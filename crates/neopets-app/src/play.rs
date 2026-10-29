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
    /// The world breakables (type-29 second list, SPEC-0048), each with its live state word.
    pub breakables: Vec<neopets_data::world::Breakable>,
    /// The movable blocks (SPEC-0048 Movable blocks) and each one's inline mesh.
    pub blocks: Vec<neopets_sim::block::Block>,
    /// The poses of the drawn placed objects that own ground patches, for the probe's owner
    /// placement (SPEC-0018 Ground probe steps 2 and 4), refreshed each tick.
    pub owner_poses: std::collections::HashMap<usize, neopets_data::ground::OwnerPose>,
    /// The ladders of the type-10 groups (SPEC-0006 Type 10, records of type -3).
    pub rails: Vec<neopets_sim::ladder::Rail>,
    /// The grab lines (SPEC-0033 Behavior 9): bit-1 cylinders of the type-11 list.
    pub grab_lines: Vec<neopets_sim::grab::Line>,
    /// Every type-10 record with its links (the climb panels are those of type -2).
    pub panels: Vec<neopets_sim::climb::Panel>,
    pub block_meshes: Vec<Option<std::ops::Range<usize>>>,
    pub motions: Vec<neopets_sim::object::Motion>,
    pub object_anims: Vec<neopets_sim::object::ObjAnim>,
    /// The placed objects' inline type-11 headers (+0x1D8): their trigger volumes (rest frame)
    /// and locators.
    pub headers: Vec<Option<neopets_data::trigger::ObjectHeader>>,
    /// MoveObjToAnimatedLocator links: object to (host, locator).
    pub obj_links: std::collections::HashMap<usize, (usize, usize)>,
    /// RegisterScriptAnimatedTrigger: record index to (host object, routine).
    /// RegisterScriptAnimatedTrigger records by (host object, record index): the routine.
    pub anim_routines: std::collections::BTreeMap<(usize, u16), u8>,
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
    /// The developer test stage's floor height (this build's own flat world; no disc pack).
    pub flat: Option<f32>,
    /// Rectangle primitives that push characters out (SPEC-0018 box push) under the world's
    /// collision mask, and all of them with their gate masks.
    boxes: Vec<BoxShape>,
    all_boxes: Vec<(u16, BoxShape)>,
    /// The pushing collision spheres (SPEC-0018 Sphere contact): those whose mask passes, and
    /// all of them with their masks.
    push_spheres: Vec<([f32; 3], f32)>,
    all_spheres: Vec<(u16, [f32; 3], f32)>,
    /// SetWorldCollisMask's gate word (0 at load).
    collis_mask: u16,
    neighbours: Neighbours,
    /// Teleport planes (doors), the arrival-door list (tag 0x0E) and the entry records.
    pub planes: Vec<TeleportPlane>,
    arrivals: Neighbours,
    pub entries: Vec<([f32; 3], [f32; 3])>,
    /// The entry records' locator tags (u16 +0x1E), by list position.
    pub tags: Vec<u16>,
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
    /// The book stations (SPEC-0028 Book station) with their 1-based book values.
    books: Vec<(neopets_sim::trigger::Volume, u16)>,
    /// Every volume that fires on contact or from a hero shot under some collision mask, and those
    /// a hero shot fires under the current mask.
    pub all_volumes: Vec<neopets_sim::trigger::Volume>,
    shot_volumes: Vec<neopets_sim::trigger::Volume>,
    routines: HashMap<u16, u8>,
    /// Script-group type registrations: trigger index to its owner group (SPEC-0048 Script
    /// groups, type registration); a later RegisterTrigger on the index replaces it.
    group_routes: HashMap<u16, usize>,
    /// Triggers of deactivated script groups (DeactivateScriptGroup): they start nothing.
    disabled_triggers: std::collections::HashSet<u16>,
    records: HashMap<u16, neopets_sim::trigger::Record>,
    pub groups: Vec<neopets_data::trigger::ScriptGroup>,
    /// The aim targets (type-11 n2 entries; SPEC-0030 row 126), each active at load.
    pub aim_targets: Vec<neopets_sim::rope::Target>,
    placements: Vec<(u8, u8)>,
    item_points: Vec<neopets_data::region::ItemPoint>,
    locks: Vec<(i32, i32, i32)>,
    opened: std::collections::BTreeSet<usize>,
    /// Camera tracks (type 28; SPEC-0052), by track id.
    pub tracks: Vec<neopets_sim::scene::Track>,
}

impl World {
    /// SetWorldCollisMask: the trigger volumes and push boxes whose gate mask is 0 or meets it.
    pub fn set_collis_mask(&mut self, mask: u16) {
        self.collis_mask = mask;
        if let Some(g) = self.ground.as_mut() {
            g.mask = mask;
        }
        self.volumes = self.all_volumes.iter().filter(|v| v.fires_on_contact(mask)).copied().collect();
        self.shot_volumes = self.all_volumes.iter().filter(|v| v.fires_on_shot(mask)).copied().collect();
        self.boxes = self.all_boxes.iter().filter(|(m, _)| *m == 0 || m & mask != 0).map(|(_, b)| *b).collect();
        self.push_spheres = self.all_spheres.iter().filter(|(m, ..)| *m == 0 || m & mask != 0).map(|&(_, c, r)| (c, r)).collect();
    }

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

    /// The developer test stage (this build's own world, no disc pack and not the original's):
    /// a flat floor at height 0 everywhere, one entry record at the origin facing +Z, and
    /// nothing else (no level geometry, objects, triggers, script, lines or music).
    pub fn test_stage(pack: u32) -> Self {
        World {
            pack,
            kind: Kind::Exterior,
            buffer: Buffer::Exterior,
            placed_effects: Vec::new(),
            objects: Vec::new(),
            breakables: Vec::new(),
            blocks: Vec::new(),
            owner_poses: Default::default(),
            rails: Vec::new(),
            grab_lines: Vec::new(),
            panels: Vec::new(),
            block_meshes: Vec::new(),
            motions: Vec::new(),
            object_anims: Vec::new(),
            headers: Vec::new(),
            obj_links: Default::default(),
            anim_routines: Default::default(),
            patches: Vec::new(),
            lights: Vec::new(),
            world_id: -1,
            zones: None,
            children: Vec::new(),
            ground: None,
            flat: Some(0.0),
            boxes: Vec::new(),
            all_boxes: Vec::new(),
            push_spheres: Vec::new(),
            all_spheres: Vec::new(),
            collis_mask: 0,
            neighbours: Neighbours::default(),
            planes: Vec::new(),
            arrivals: Neighbours::default(),
            entries: vec![([0.0; 3], [0.0, 0.0, 1.0])],
            tags: vec![0],
            scripted_doors: false,
            lines: Default::default(),
            entities: Vec::new(),
            navmeshes: Vec::new(),
            pens: Vec::new(),
            save_points: Vec::new(),
            map_area: None,
            music: None,
            ambients: Vec::new(),
            ambient_counters: Vec::new(),
            spheres: Vec::new(),
            capacity: 0,
            volumes: Vec::new(),
            books: Vec::new(),
            all_volumes: Vec::new(),
            shot_volumes: Vec::new(),
            routines: HashMap::new(),
            group_routes: HashMap::new(),
            disabled_triggers: Default::default(),
            records: HashMap::new(),
            groups: Vec::new(),
            aim_targets: Vec::new(),
            placements: Vec::new(),
            item_points: Vec::new(),
            locks: Vec::new(),
            opened: Default::default(),
            tracks: Vec::new(),
        }
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
            breakables: Vec::new(),
            blocks: Vec::new(),
            owner_poses: Default::default(),
            rails: Vec::new(),
            grab_lines: Vec::new(),
            panels: Vec::new(),
            block_meshes: Vec::new(),
            motions: Vec::new(),
            object_anims: Vec::new(),
            headers: Vec::new(),
            obj_links: Default::default(),
            anim_routines: Default::default(),
            children: neopets_data::model::find_trees(&bytes, &chain.chunks).0.iter().filter(|t| t.depth > 0).map(|t| t.role_id).collect(),
            ground: Ground::of(&bytes, &chain.chunks, CellOrder::XMajor),
            flat: None,
            boxes: Vec::new(),
            push_spheres: Vec::new(),
            all_spheres: Vec::new(),
            all_boxes: ground::rectangles(&bytes, &chain.chunks)
                .into_iter()
                .filter(|r| r.pushes(0xFFFF))
                .map(|r| (r.mask, BoxShape { origin: r.origin, axes: r.axes, extents: r.extents }))
                .collect(),
            collis_mask: 0,
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
            books: Vec::new(),
            all_volumes: Vec::new(),
            shot_volumes: Vec::new(),
            routines: HashMap::new(),
            group_routes: HashMap::new(),
            disabled_triggers: Default::default(),
            records: HashMap::new(),
            groups: neopets_data::trigger::script_groups(&bytes, &chain.chunks),
            aim_targets: neopets_data::ground::aim_targets(&bytes, &chain.chunks).iter().map(|t| neopets_sim::rope::Target { point: t.point, active: true }).collect(),
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
        // SPEC-0048 Start mode and reset: an entry keeps its pack state (1 on disc) at each load;
        // modes 1 and 2 apply only to an entry whose broken bit is set, and no disc entry has
        // them, so nothing changes here.
        world.breakables = neopets_data::world::breakables(&bytes, &chain.chunks);
        // NEOPETS_TRACE_BREAK_KEYS: the entries whose break spawns a character by key, or that
        // carry an element seal.
        if std::env::var_os("NEOPETS_TRACE_BREAK_KEYS").is_some() {
            for (i, b) in world.breakables.iter().enumerate().filter(|(_, b)| b.spawn_key >= 0 || !matches!(b.element, 0 | 7)) {
                println!("      break key: pack {} breakable {i} key {} element {} state {} at {:?}", world.pack, b.spawn_key, b.element, b.state, b.position());
            }
        }
        // SPEC-0048 Switch groups, set-up: in each kind-4 entry members 1 to n-1 are disabled.
        for g in world.groups.iter().filter(|g| g.kind == 4) {
            for &m in g.breakables.iter().skip(1) {
                if let Some(b) = usize::try_from(m).ok().and_then(|m| world.breakables.get_mut(m)) {
                    b.state = 0;
                }
            }
        }
        let records = neopets_data::trigger::panels(&bytes, &chain.chunks);
        world.rails = records.iter().filter(|p| p.kind == -3).map(|p| neopets_sim::ladder::Rail { normal: p.normal, u: p.u, v: p.v, lu: p.lu, lv: p.lv }).collect();
        world.panels = records.iter().map(|p| neopets_sim::climb::Panel { normal: p.normal, u: p.u, v: p.v, lu: p.lu, lv: p.lv, kind: p.kind, links: p.links }).collect();
        // SPEC-0018 Patch-id table: each placed object's inline model set is registered under
        // its patch id.
        if let Some(g) = world.ground.as_mut() {
            for (i, o) in world.objects.iter().enumerate() {
                if let Some(r) = o.as_ref().and_then(|o| o.model_set.clone()) {
                    g.add_object_set(i, &bytes[r]);
                }
            }
        }
        world.motions = world
            .objects
            .iter()
            .map(|o| o.as_ref().and_then(|o| neopets_sim::object::Motion::parse(&bytes[o.stream.clone()?], o.keys as usize)).unwrap_or_default())
            .collect();
        world.object_anims = vec![Default::default(); world.objects.len()];
        world.headers = world.objects.iter().map(|o| o.as_ref().and_then(|o| o.header.clone()).map(|r| neopets_data::trigger::object_header(&bytes[r]))).collect();
        let (ambients, spheres) = ground::sound_emitters(&bytes, &chain.chunks);
        world.ambient_counters = vec![0; ambients.len()];
        world.ambients = ambients;
        world.spheres = spheres;
        let (capacity, volumes) = neopets_data::trigger::volumes(&bytes, &chain.chunks);
        world.capacity = capacity;
        world.grab_lines = volumes
            .iter()
            .filter(|v| v.kind == 3 && v.flags & 0x0002 != 0)
            .filter_map(|v| match v.shape {
                neopets_data::trigger::Shape::Cylinder { base, axis, length, radius } => Some(neopets_sim::grab::Line { a: base, u: axis, length: length.abs(), radius, mask: v.mask }),
                _ => None,
            })
            .collect();
        let all: Vec<(neopets_sim::trigger::Volume, u16)> = volumes
            .into_iter()
            .map(|v| (sim_volume(&v), v.class))
            .collect();
        world.books = all.iter().filter(|(v, _)| neopets_sim::book::is_station(v.kind, v.flags)).copied().collect();
        world.all_spheres = all
            .iter()
            .filter(|(v, _)| v.kind == 1 && neopets_sim::contact::sphere_pushes(v.flags))
            .filter_map(|(v, _)| match v.shape {
                neopets_sim::trigger::Shape::Sphere { centre, radius } => Some((v.mask, centre, radius)),
                _ => None,
            })
            .collect();
        world.volumes = all.into_iter().map(|(v, _)| v).filter(|v| v.fires_on_contact(0xFFFF) || v.fires_on_shot(0xFFFF)).collect();
        world.all_volumes = world.volumes.clone();
        // SPEC-0048 Source and Placement: each n1 entry with a bit-1 box is a block, placed on
        // the ground. Its box is the block's own solid (it moves with it), so it is not also a
        // static push box.
        let sources = neopets_data::trigger::movable_blocks(&bytes, &chain.chunks);
        let blocks: Vec<neopets_sim::block::Block> = sources
            .iter()
            .map(|s| {
                neopets_sim::block::Block::new(s.size, s.height, s.corner, s.axes, s.extents, |x, z, c| {
                    let h = world.probe(x, z, c);
                    h.found.then_some(h.height)
                })
            })
            .collect();
        world.blocks = blocks;
        world.block_meshes = sources.iter().map(|s| s.mesh.clone()).collect();
        world.all_boxes.retain(|(_, b)| !sources.iter().any(|s| s.corner == b.origin));
        world.set_collis_mask(0);
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
        // The test stage: one level floor everywhere, found from at or above it.
        if let Some(h) = self.flat.filter(|&h| c >= h - 0.5) {
            return ProbeHit { height: h, normal: UP, kind: None, found: true, respawn: None, surface: None, zone: None };
        }
        self.ground.as_ref().map_or(ProbeHit { height: ground::MISS, normal: UP, kind: None, found: false, respawn: None, surface: None, zone: None }, |g| g.probe_owned(x, z, c, &|o| self.owner_poses.get(&o).copied()))
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
    /// The same, on the floor within 2.5 of height y (stacked floors): (x, z, y).
    PointAt([f32; 3]),
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
    /// SPEC-0010: the Cloud Beast's controller (the test spawn only).
    pub cloud: Option<neopets_sim::cloudbeast::Cloud>,
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
    /// The 15 colour slots (SPEC-0040 Behavior 26): (red, green, blue) / 255 of the record's
    /// colour words; a part draws with the slot of its material group.
    pub slots: [[f32; 3]; 15],
    /// The hit flash's frames left (SPEC-0046 Per-frame draw order, step 6): while above 0
    /// every part draws with (1.5, 0.5, 0.5) in place of its slot colour.
    pub flash: u8,
    /// The kind-8 record its drop placement ordinal names (SPEC-0038 Behavior 34, part B).
    drop: Option<neopets_data::region::ItemPoint>,
    /// The carried Neopoints (SPEC-0038 Behavior 34 part D; enemy 0xC28) and the stolen item
    /// with its carry timer in ticks (SPEC-0029 Behavior 11b).
    carried_np: i32,
    carried_item: Option<(u16, u32)>,
    root_step: [f32; 3],
    /// A script walk to a point (SPEC-0030 rows 106 and 108; SPEC-0056 classes 23 and 25), with
    /// the final facing of the orienting form.
    script_walk: Option<([f32; 3], Option<[f32; 3]>, f32)>,
    /// SetAI_MovementType: 0 walk, 1 run, 2 fast run, 3 and 4 idle (SPEC-0040 Behavior 14D).
    move_type: u8,
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
    /// The Gnarfas boss controller (behaviour class 14; SPEC-0020) and the hero's hit on it this
    /// tick, which the boss takes through its own flinch.
    pub gnarfas: Option<neopets_sim::gnarfas::Gnarfas>,
    pub pending_hit: Option<i32>,
    /// The Werelupe King controller (behaviour class 32; SPEC-0019).
    pub king: Option<neopets_sim::werelupe::King>,
    /// The Ixi Chief's controller (SPEC-0014; class 11) and the summed damage of the hero shots
    /// that hit it this tick (its melee hit is `pending_hit`).
    pub ixi: Option<neopets_sim::ixi::Ixi>,
    pub pending_shot: Option<i32>,
    /// A Faerie Sister's controller (behaviour classes 13, 15 and 16; SPEC-0011) and the hero's
    /// hit on her this tick (damage, and whether it was a shot).
    pub sister: Option<neopets_sim::sisters::Sister>,
    pub sister_hit: Option<(i32, bool)>,
    /// The Undead Draik controller (behaviour class 12; SPEC-0029 Behavior 6).
    pub draik: Option<neopets_sim::draik::Draik>,
    /// The Darkest Faerie controller (behaviour class 37; SPEC-0054).
    pub faerie: Option<neopets_sim::darkest::Faerie>,
    /// The row's attack-slot spawn keys (+0x3CC + 2k; SPEC-0029 Behavior 31).
    pub spawn_keys: [u16; 4],
    /// The Queen Fyora controller (behaviour class 38; SPEC-0029 Behavior 30).
    pub fyora: Option<neopets_sim::fyora::Fyora>,
    /// The Plague Serpent controller (behaviour class 2; SPEC-0016) and the hero's hit on it.
    pub serpent: Option<neopets_sim::serpent::Serpent>,
    pub serpent_hit: Option<(i32, bool)>,
    /// The Mortog controller (behaviour class 39; SPEC-0029 Behavior 3).
    pub mortog: Option<neopets_sim::mortog::Mortog>,
    /// The wild petpet loop (class 33; SPEC-0047) and the record's pet type id (+0xAC).
    pub wild: Option<neopets_sim::petpet::Wild>,
    pub pet_type: i32,
    /// The second petpet behaviour (class 20; SPEC-0047).
    pub class20: Option<neopets_sim::petpet::Class20>,
    /// The Black Knight's per-fight record (behaviour class 8; SPEC-0012).
    pub knight: Option<neopets_sim::knight::Knight>,
    /// The Gelert Assassin (behaviour class 1; SPEC-0013), its clone slot (block ordinal) and
    /// the hit waiting for its controller (damage, from a shot).
    pub gelert: Option<neopets_sim::gelert::Gelert>,
    pub gelert_slot: usize,
    pub gelert_hit: Option<(i32, bool)>,
    /// SetEntityInvulnerable (SPEC-0030 row 132; +0x7F0): never a hit candidate while set.
    pub invulnerable: bool,
}

impl Character {
    /// The Cloud Beast stand-in's body (it has no rig): a cylinder of the row's radius and
    /// height around the floating cloud, the base raised by the hover. None for anyone else.
    pub fn stand_in_body(&self) -> Option<neopets_sim::combat::Cylinder> {
        self.cloud.as_ref()?;
        let r = self.row.map_or(3.0, |r| r.radius);
        let h = self.row.map_or(3.5, |r| r.height);
        Some(neopets_sim::combat::Cylinder { center: add(self.position, [0.0, crate::placeholder::CLOUD_HOVER + 0.5 * h, 0.0]), axis: [0.0, 1.0, 0.0], radius: r, length: h })
    }

    /// SPEC-0047 Layout / SPEC-0036 Hittable predicate tests 4 and 5: a character whose pet type
    /// id has a kind in either table is skipped by hero swings, shots and the Mote walks.
    pub fn pet_like(&self) -> bool {
        neopets_sim::petpet::kind(self.pet_type).is_some() || neopets_sim::petpet::second_kind(self.pet_type).is_some()
    }

    /// Skipped by hero swings, shots and the Mote walks: pet-like, or invulnerable by script
    /// (SPEC-0040 Behavior 17 base rule, the flag at +0x7F0).
    /// A boss whose controller sets the unhittable word (+0x7F0) and tests the hero's swing and
    /// shots in its own hit routine: Gnarfas, the Plague Serpent, the Faerie Sisters, the Black
    /// Knight and the Undead Draik (SPEC-0036 Hittable predicate). The Mote power-attack objects
    /// and the target finder never select them.
    /// The Juppie is excluded from every hero hit scan as well: only its stun loop counts swings
    /// and only its dazed loop shots (SPEC-0015 Hero actions); so is the Darkest Faerie, whose
    /// own tests take the flinch, the form-2 hit and the absorbed shots (SPEC-0054 Hero actions).
    pub fn own_hit_check(&self) -> bool {
        self.gnarfas.is_some() || self.serpent.is_some() || self.sister.is_some() || self.knight.is_some() || self.draik.is_some() || self.boss.is_some() || self.faerie.is_some()
    }

    /// The guard query (base state 20; SPEC-0029 Open question 171): a blocking common enemy
    /// (outer 7 or 8), the Black Knight outside an open stun gate, the guarding Undead Draik and
    /// Werelupe King, and the Ixi Chief in idle, hold, the stage transition and its blocks. A hero
    /// shot meets such a target as a guarding target. (Gnarfas's missile block removes the shot
    /// in its own check instead, SPEC-0020.)
    pub fn guards(&self) -> bool {
        use neopets_sim::ixi::State as Ixi;
        self.enemy.as_ref().is_some_and(|e| e.blocking)
            || self.knight.as_ref().is_some_and(|k| !k.exposed && !k.defeated())
            || self.draik.as_ref().is_some_and(|d| d.guarding())
            || self.king.as_ref().is_some_and(|k| k.guarding())
            || self.ixi.as_ref().is_some_and(|x| matches!(x.state, Ixi::Idle | Ixi::Hold | Ixi::Transition | Ixi::BlockMelee | Ixi::BlockMissile))
    }

    /// An enemy kind is also not hittable in its melee or missile dodge (agro outer sub-state 5
    /// or 6) or, when not script-held, while animation 74, 75 or 76 plays (SPEC-0036 Hittable
    /// predicate, tests 2 and 6).
    pub fn unhittable(&self) -> bool {
        self.pet_like()
            || self.invulnerable
            || self.enemy.as_ref().is_some_and(|e| e.lying() || (e.state == 19 && matches!(e.outer, 5 | 6)) || (!self.held && matches!(self.anim.clip_state().action, 74..=76)))
    }
}

/// A summon-reserved record of a character block (SPEC-0029 Behavior 31), with the character it
/// is bound to.
struct Reserve {
    pack: u32,
    slot: u8,
    region: u32,
    pl: neopets_data::region::Placement,
    model: ModelData,
    anim: HeroAnim,
    row: Option<neopets_data::monster::Row>,
    tuning: Option<neopets_data::monster::Tuning>,
    bound: Option<u64>,
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
    /// The reeds of a reed patch (Behavior 9), one per plant.
    pub reeds: Vec<neopets_sim::foliage::Reed>,
    /// The zone number (+0x20, low byte) and whether this pass's zone query passed.
    pub zone: u8,
    pub on: bool,
    /// The draw-distance overrides A, B, C, D (SPEC-0060 section 3).
    pub overrides: [f32; 4],
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
    /// Until the hero holds an item.
    Item(u16),
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
    pub root_step: [f32; 3],
    /// Diagnostic block slides (block, target x and z) for traces.
    pub diag_slides: Vec<(usize, [f32; 2])>,
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
    /// Square held on the last input (the level scripts' Square release tests).
    square_held: bool,
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
    /// The target of a target play (SPEC-0027 Target plays): the pack and the target.
    cut_target: Option<(u32, neopets_sim::level::CameraTarget)>,
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
    /// The Rope Gun (SPEC-0036 Right-stick aim mode; SPEC-0033 State 27): the aim mode, the
    /// world's rope manager, state 27's phases and the right stick's converted Y this tick.
    pub aim: neopets_sim::rope::AimMode,
    pub rope: neopets_sim::rope::Rope,
    rope_hero: Option<neopets_sim::rope::RopeHero>,
    rope_y: f32,
    /// The world the rope manager belongs to (a world change resets it).
    rope_pack: u32,
    /// Diagnostic (NEOPETS_TRACE_ROPE): the scan's camera forward points at this aim target.
    pub rope_aim: Option<usize>,
    /// The equipped weapon (item id), the item table, the hero's hit list and the weapon axis
    /// of the previous tick (D2 of the swept test).
    /// The heroes' family entries and Mote words (SPEC-0021 Party inventory lists, Equipment
    /// slots); `weapon` and `shield` mirror Tor's entries for the combat code.
    pub equipment: neopets_sim::inventory::Equipment,
    /// The Tattered Cloak is worn (the cloak swap; SPEC-0021 Tattered Cloak, partly).
    pub cloak_worn: bool,
    /// The party's weapon-drawn and shield-raised flags (SPEC-0040 Behavior 29), shared by both
    /// heroes.
    pub weapon_drawn: bool,
    pub shield_raised: bool,
    pub weapon: Option<u16>,
    /// The equipped shield (item id; Tor's shields are type 5, SPEC-0036).
    pub shield: Option<u16>,
    pub items: neopets_data::item::ItemTable,
    reach_tor: Vec<f32>,
    reach_roberta: Vec<f32>,
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
    /// The hero's block state (state 18, SPEC-0033 Behavior 4A.2).
    hold: Option<neopets_sim::block::Hold>,
    /// States 19 and 58 (SPEC-0033 Behavior 51), the side-touch latch (Behavior 15A) and the
    /// push-into-face counter (Behavior 15).
    hang: Option<neopets_sim::block::Hang>,
    /// The grab-line hang (state 28) and its cooldown (SPEC-0033 Behavior 9).
    line_hang: Option<neopets_sim::grab::Hang>,
    line_cooldown: u32,
    /// The hero's touched flag from the last motion step (see `record_tick`'s contact pass).
    touched: bool,
    /// The ledge state (30, 35 or 47) and its held edge (SPEC-0033 Behavior 51).
    ledge: Option<neopets_sim::ledge::Ledge>,
    climb: Option<neopets_sim::block::Climb>,
    /// State 41 and the grab cooldown (Behavior 20) the jump off sets.
    ladder: Option<neopets_sim::ladder::Ladder>,
    grab_cooldown: u32,
    /// State 21 (SPEC-0033 Behavior 51).
    panel: Option<neopets_sim::climb::Climb>,
    side_touch: bool,
    face_count: u32,
    stick_centred: bool,
    /// Hero switching (SPEC-0053): the selected hero (0 Tor, 1 Roberta), the parked hero, the
    /// swap mode, the running swap task and the L1 and R1 press edges kept for the next tick.
    pub hero_index: usize,
    pub parked: Option<Parked>,
    pub swap_mode: u32,
    /// The next switch request is the script's forced form (SPEC-0023 section 12).
    forced_switch: bool,
    pub swap: Option<neopets_sim::swap::Task>,
    pub l1_latch: bool,
    pub r1_latch: bool,
    pub last_swap: String,
    /// Roberta's attack step, her six shot slots and the shot rows (SPEC-0036).
    r_attack: Option<neopets_sim::roberta::RobertaAttack>,
    pub hero_shots: Vec<neopets_sim::roberta::Shot>,
    /// The hero's candidate list and its lock (SPEC-0040 Behavior 20).
    pub candidates: neopets_sim::lock::Candidates,
    pub lock: Option<u64>,
    /// Diagnostic: the fight trace's target by character name (NEOPETS_TRACE_TARGET).
    pub fight_target: Option<String>,
    shot_rows: neopets_sim::roberta::Rows,
    /// Forced binds waiting for the embedded region: (pack, ordinal).
    rebind: Vec<(u32, usize)>,
    /// Entities deactivated before their characters loaded.
    pending_off: std::collections::HashSet<(u32, usize)>,
    /// The Juppie tuning words and the boss hits of this tick's swing (character id, damage).
    juppie_words: neopets_sim::juppie::Words,
    gnarfas_words: neopets_sim::gnarfas::Words,
    king_words: neopets_sim::werelupe::Words,
    /// The Faerie Sisters' tuning words (overwritten by alternate loop A) and their shared group.
    sister_words: neopets_sim::sisters::Words,
    sister_group: neopets_sim::sisters::Group,
    /// The Mote-eater drain (SPEC-0034 3.13): the drain, its body effect and its slots' effects
    /// with their offsets.
    drain: Option<(neopets_sim::sisters::Drain, usize, Vec<(usize, [f32; 3])>)>,
    drain_words: neopets_sim::sisters::DrainWords,
    draik_words: neopets_sim::draik::Words,
    ixi_words: neopets_sim::ixi::Words,
    faerie_words: neopets_sim::darkest::Words,
    reserves: Vec<Reserve>,
    /// The duel's shared slots (SPEC-0029 Behavior 19), its words, and the loop-B summons.
    duel: neopets_sim::fyora::Slots,
    duel_words: neopets_sim::fyora::Words,
    serpent_words: neopets_sim::serpent::Words,
    mortog_words: neopets_sim::mortog::Words,
    /// The Mortog zone of a world (InitMortogBehavior): the pack, the zone and the target index.
    mortog_zone: Option<(u32, neopets_sim::mortog::Zone, i32)>,
    duel_summons: Vec<u64>,
    boss_hits: Vec<(u64, i32)>,
    pub last_boss: String,
    /// The last spawn-by-key request's outcome.
    pub last_summon: String,
    /// The live petpet (SPEC-0047 class 34), its character once its package has loaded, the
    /// pending save values, the bonus words and the low-time mark.
    pub petpet: Option<neopets_sim::petpet::Live>,
    pet_char: Option<u64>,
    pet_spawn: Option<[f32; 3]>,
    pub pet_pending: neopets_sim::petpet::Pending,
    pet_bonus: neopets_sim::petpet::Bonus,
    pet_low: i32,
    pub last_pet: String,
    /// SPEC-0047 food: the base words and sets, the shared reservation and the food the hero is
    /// feeding (hero state 49).
    pet_food: neopets_sim::petpet::Food,
    pet_reserve: neopets_sim::petpet::Reservation,
    feeding: Option<u16>,
    /// The class 20 waits (4 to 5 s) in ticks.
    class20_waits: (i32, i32),
    /// The Black Knight's tuning block (SPEC-0012, 0x2DA3E8).
    knight_words: Option<neopets_sim::knight::Words>,
    /// The Gelert Assassin's tuning words and data constants (SPEC-0013).
    gelert_words: Option<neopets_sim::gelert::Words>,
    /// The live Mote power-attack objects and their boot words (SPEC-0036 Charm attack hits).
    /// The Undead Draik's spit flight effects (fire, ice).
    spit_fx: [Option<usize>; 2],
    mote_objects: Vec<neopets_sim::charm::Object>,
    charm_words: neopets_sim::charm::Words,
    /// The Lava orbit's angle, 0 at the first activation and continuing across spawns.
    orbit_angle: f32,
    hit_list: Vec<u64>,
    /// Bumped with each clear of the hit list: one melee swing (the thorn hazards' strike gate).
    swing_serial: u32,
    /// L1 held this tick (pad slot 0; SPEC-0038 Behavior 14 code 5).
    l1_held: bool,
    /// SPEC-0060 Cut: last tick's weapon point 2.5 along the blade's frame axis.
    reed_point: Option<[f32; 3]>,
    /// SPEC-0040 Behavior 23A: the hero's stored look yaw and pitch.
    head_look: neopets_sim::look::Look,
    /// The Plague Serpent's neck joints (boot list 0x320F48; SPEC-0016 6A).
    neck_joints: Vec<usize>,
    /// SPEC-0010 Test spawn: with NEOPETS_CLOUD_BEAST set, monster rows 1 (the Cloud Beast) and
    /// 2 (its companion), spawned in each world with the stand-in models (no model on disc).
    cloud_rows: Option<(neopets_data::monster::Row, neopets_data::monster::Row)>,
    /// The test spawn is on (NEOPETS_CLOUD_BEAST, or the developer menu).
    cloud_on: bool,
    /// The next spawn also places the hero in the middle of the arena (the menu's fight entry).
    cloud_place_hero: bool,
    cloud_words: Option<neopets_sim::cloudbeast::Words>,
    /// The arena: the level's locators 101 to 106 when it has all six, else a stand-in ring of
    /// radius 16 around where the hero stood at the spawn, each point facing the centre.
    cloud_arena: Option<(u32, [([f32; 3], [f32; 3]); 6])>,
    /// Worlds whose Cloud Beast was defeated (no respawn there).
    cloud_defeated: std::collections::HashSet<u32>,
    /// Recent ball bursts and reply hit flashes for the stand-in effects: (point, ticks left).
    pub cloud_bursts: Vec<([f32; 3], u32)>,
    /// SPEC-0038 Behavior 45: the shared attack-manager list.
    attack_mgr: neopets_sim::attackmgr::Manager,
    /// SPEC-0038 Behavior 3: the broadcast count of each alarm group (by pack and group).
    alarms: HashMap<(u32, u8), u32>,
    /// SPEC-0011 Thorn hits: the five-record hazard pool, its contact radius, and the thorn
    /// shots' positions by pool slot as of the last tick.
    thorns: neopets_sim::thorn::Pool,
    thorn_radius: f32,
    thorn_shots: [Option<[f32; 3]>; neopets_sim::projectile::SLOTS],
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
    /// The enemy and breakable payouts due: the pack, the point, the drop placement and the
    /// carried Neopoints (SPEC-0038 Behavior 34 parts A, B and D).
    payouts: Vec<(u32, [f32; 3], Option<neopets_data::region::ItemPoint>, i32)>,
    /// Switch-group stages shown in state 3 that become breakable (state 1) on a tick (SPEC-0048
    /// Switch groups, step 3): (pack, breakable, tick).
    chain_arms: Vec<(u32, usize, u64)>,
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
    /// The second quick panel, the Mote picker (SPEC-0021), its boot direction table, and the
    /// Square edge it reads (latched like the first panel's buttons).
    pub mote_panel: neopets_sim::quick::MotePanel,
    mote_table: [u8; 120],
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
    /// the state-60 penalty due on the next tick.
    pub death: neopets_sim::death::DeathFlow,
    penalty_due: bool,
    /// The game-over page's exit with no save or load pending: the host tears the world down
    /// and the front end goes to record 0 (SPEC-0026 Game over flag; SPEC-0035 state 26).
    pub teardown_wanted: bool,
    /// The in-play gate on story bit 368 fired (SPEC-0026 Credits): the host opens the credits.
    pub credits_wanted: bool,
    credits_fired: bool,
    /// The game's random generator (SPEC-0008): one stream, seeded once.
    pub rng: neopets_sim::rng::Rng,
    /// Effects (SPEC-0034): the resource-9 definition table, its material images, the shared
    /// pools and the footstep dust table (28 ids at 0x3150A8).
    pub effect_table: std::sync::Arc<neopets_sim::effect::EffectTable>,
    pub effect_images: std::sync::Arc<Vec<neopets_data::effect::GroupImage>>,
    pub effects: neopets_sim::effect::Effects,
    /// The item showcase in its full mode (SPEC-0021 Item showcase; SPEC-0033 Behavior 42).
    pub showcase: Option<Showcase>,
    /// The screen-anchored showcase of modes 1 to 3 (SPEC-0021 Item showcase).
    pub corner: Option<Corner>,
    /// The ids picked up before, for the first-pickup text.
    showcased: std::collections::HashSet<u16>,
    /// The swing-trail manager (SPEC-0034 3.8), the hero's held-weapon trail and each enemy's.
    pub trails: neopets_sim::trail::Trails,
    hero_trail: Option<usize>,
    enemy_trails: HashMap<u64, usize>,
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
    /// The health and pool bars' fill points as last drawn (SPEC-0023 section 3, Loss
    /// response): the fill's right edge and vertical centre as fractions of the window, and the
    /// window's aspect; written by the HUD view.
    pub hud_fill_points: [Option<([f32; 2], f32)>; 2],
    /// Each bar's loss effect (0xFA2 health, 0xFA3 pool; SPEC-0034 Fixed users of group 3):
    /// its object and saved screen point, and the values of the previous tick.
    hud_loss: [Option<(usize, [f32; 2], f32)>; 2],
    hud_prev: [i32; 2],
    /// Character light states (SPEC-0046 Character lights): the hero's (key None) and each
    /// region character's, by id.
    char_lights: HashMap<Option<u64>, neopets_sim::charlight::State>,
    /// The heroes' shadow hulls (Tor's outfit, Roberta's 512; SPEC-0004 type 33), by hero index.
    pub shadow_hulls: [Vec<neopets_sim::shadow::Hull>; 2],
    /// The drop direction of a dig reward while it pays out (SPEC-0051 section 10 step 11).
    dig_drop: Option<[f32; 3]>,
    /// The hero's position on the last bush logic step (for the bend's speed).
    bend_last: Option<[f32; 3]>,
    /// Where a swap's effects were placed (the outgoing hero's position at step 2).
    swap_at: [f32; 3],
    /// The hero attack-effect driver (SPEC-0034 3.17): its tables and live effects.
    attack_fx: AttackFx,
    /// Added object yaw (SetAnimObjAdditionRotY), degrees, by (pack, object).
    obj_yaw: HashMap<(u32, usize), f32>,
    /// Added vertical offset (SetAnimObjPositionOffset), by (pack, object).
    obj_offset: HashMap<(u32, usize), f32>,
    /// The camera shake record and this frame's movement (SPEC-0027 Camera shake).
    pub shake: neopets_sim::shake::Shake,
    pub shake_now: Option<neopets_sim::shake::Offset>,
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
    /// The item placements (kind 8) of the region loaded in each slot, for ItemActivate.
    slot_items: HashMap<(u32, u8), Vec<neopets_data::region::ItemPoint>>,
    /// Script slots whose last load has arrived (for the slot's living count).
    slot_arrived: std::collections::HashSet<(u32, u8)>,
    /// The attack window was open on the last tick (for the strike edge).
    window_was: bool,
    /// The current world's pending slot (SetWorldLoadOverRide), None for -1.
    pending_slot: Option<u32>,
    pub region_wanted: Vec<(u32, u8, u32)>,
    next_character: u64,
    /// Set once the hero's meshes exist.
    pub spawned: bool,
    /// Which game this is: a rebuild (New Game after a Quit, or a load) counts up, so the
    /// renderer drops every drawn character (the new game's ids start again).
    pub generation: u32,
    /// A front-end page other than play is up (the world is frozen and the HUD hidden).
    pub menu_open: bool,
    /// SPEC-0033 Behavior 1C: the rumble manager, its presets and Tor's leap-landing request
    /// (preset, seconds, magnitude) from the boot program, and the motor send of this tick for
    /// the window to pass to the first controller's large motor.
    rumble: neopets_sim::rumble::Rumble,
    rumble_presets: Vec<neopets_sim::rumble::Preset>,
    /// The Mote colour rows (SPEC-0023 section 25; 0x2DCA60, one RGB row per Mote 160 to 173).
    mote_glow_rgb: [[u8; 3]; 14],
    landing_rumble: (usize, f32, f32),
    pub motor: Option<neopets_sim::rumble::Send>,
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
    /// The developer menu's cheats (a test tool of this build; no spec).
    pub cheats: Cheats,
    /// The hero's hit flash frames left (SPEC-0046 step 6).
    pub hero_flash: u8,
    /// The book words (SPEC-0035 section 6.1) and whether the chooser shows "Read Book".
    pub books: neopets_sim::book::Books,
    book_prompt: bool,
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
        // The outfits' shadow hulls (SPEC-0004 Optional per-model block, type 33).
        let mut hulls = |n: u32| -> Vec<neopets_sim::shadow::Hull> {
            let Ok(bytes) = index.read(disc, n) else { return Vec::new() };
            let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) else { return Vec::new() };
            neopets_data::shadow::find(&bytes, &chain.chunks)
                .unwrap_or_default()
                .into_iter()
                .map(|u| neopets_sim::shadow::Hull { bank: u.bank, edges: u.edges, triangles: u.triangles, points: u.points })
                .collect()
        };
        let shadow_hulls_loaded = [hulls(outfit), hulls(512)];
        let boot = BootTravel::from_disc(disc)?;
        let items = neopets_data::item::ItemTable::parse(&index.read(disc, neopets_data::item::ITEM_CONFIG)?)?;
        let boot_bytes = neopets_data::archive::verified_boot_program(disc)?;
        let reach_tor = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::item::REACH_TOR, 11)?
            .into_iter()
            .map(f32::from_bits)
            .collect();
        let reach_roberta = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::item::REACH_ROBERTA, 8)?.into_iter().map(f32::from_bits).collect();
        let hero_max = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::item::HEALTH_TOR, 1)?[0] as i32;
        let floats = |at: u32| -> Result<Vec<f32>, String> {
            Ok(neopets_data::archive::boot_u32_table(&boot_bytes, at, 7)?.into_iter().map(f32::from_bits).collect())
        };
        let bands = (floats(neopets_data::monster::BAND_F1)?, floats(neopets_data::monster::BAND_F2)?);
        let words = |at: u32, n: usize| neopets_data::archive::boot_u32_table(&boot_bytes, at, n);
        let rows = words(neopets_data::item::QUICK_ROWS, 32)?;
        let quick_rows: [[i32; 8]; 4] = std::array::from_fn(|r| std::array::from_fn(|k| rows[8 * r + k] as i32));
        let mote_words = words(neopets_sim::quick::MOTE_TABLE_AT, 30)?;
        let mote_table: [u8; 120] = std::array::from_fn(|k| mote_words[k / 4].to_le_bytes()[k % 4]);
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
        let knight_words = words(neopets_sim::knight::BASE, neopets_sim::knight::WORDS).ok().and_then(|w| neopets_sim::knight::Words::read(&w));
        let gelert_words = words(neopets_sim::gelert::BASE, neopets_sim::gelert::WORDS).ok().and_then(|w| neopets_sim::gelert::Words::read(&w, &|at| words(at, 1).ok().map(|v| v[0])));
        let class20_waits = {
            let w = words(neopets_sim::petpet::CLASS20_WAIT_AT, 2)?;
            ((f32::from_bits(w[0]) * 60.0) as i32, (f32::from_bits(w[1]) * 60.0) as i32)
        };
        let pet_food = {
            use neopets_sim::petpet::*;
            let set = |at: u32| words(at, 8).map(|w| w.iter().map(|&v| v as i32).collect::<Vec<i32>>());
            let base = words(BASE_AT, 9)?;
            Food { base: std::array::from_fn(|k| f32::from_bits(base[k])), sets: Sets { a: set(SET_A)?, b: set(SET_B)?, c: set(SET_C)? } }
        };
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
                dazed_cone: float(DAZED_CONE)?,
                spit_damage: int(SPIT_DAMAGE)? as f32,
            }
        };
        let gnarfas_words = {
            use neopets_sim::gnarfas::*;
            let int = |at: u32| words(at, 1).map(|v| v[0] as i32);
            let pair = |at: u32| words(at, 2).map(|v| (f32::from_bits(v[0]), f32::from_bits(v[1])));
            let ints = |at: u32| words(at, 2).map(|v| [v[0] as i32, v[1] as i32]);
            Words {
                laugh_chance: int(LAUGH_CHANCE)?,
                rage_percent: int(RAGE_PERCENT)?,
                block_chance: ints(BLOCK_CHANCE)?,
                dodge_chance: int(DODGE_CHANCE)?,
                reaction: pair(REACTION)?,
                missile_jump_chance: int(MISSILE_JUMP_CHANCE)?,
                melee_dodge_chance: int(MELEE_DODGE_CHANCE)?,
                whip_cooldown: pair(WHIP_COOLDOWN)?,
                whip_range: float(WHIP_RANGE)?,
                whip_damage: int(WHIP_DAMAGE)?,
                rapid_cooldown: pair(RAPID_COOLDOWN)?,
                rapid_range: float(RAPID_RANGE)?,
                rapid_damage: int(RAPID_DAMAGE)?,
                rapid_exhaust_chance: int(RAPID_EXHAUST)?,
                power_range: float(POWER_RANGE)?,
                power_cooldown: pair(POWER_COOLDOWN)?,
                power_chance: ints(POWER_CHANCE)?,
                power_damage: int(POWER_DAMAGE)?,
                jump_hero_chance: int(JUMP_HERO_CHANCE)?,
                slam_damage: int(SLAM_DAMAGE)?,
                butt_range: float(BUTT_RANGE)?,
                butt_cooldown: pair(BUTT_COOLDOWN)?,
                butt_chance: ints(BUTT_CHANCE)?,
                approach_range: float(APPROACH_RANGE)?,
                jump_time: float(JUMP_TIME)?,
                exhaust: [pair(EXHAUST)?, pair(EXHAUST + 8)?],
                push: float(PUSH)?,
                gravity: float(GRAVITY)?,
                run_walk: float(RUN_WALK)?,
            }
        };
        let king_words = {
            use neopets_sim::werelupe::*;
            let i = |at: u32, n: usize| words(at, n).map(|v| v.iter().map(|&x| x as i32).collect::<Vec<_>>());
            let f = |at: u32, n: usize| words(at, n).map(|v| v.iter().map(|&x| f32::from_bits(x)).collect::<Vec<_>>());
            let three = |at: u32| i(at, 3).map(|v| [v[0], v[1], v[2]]);
            let (sw, lu, co, mv, dm, sh) = (words(SWING, 5)?, words(LUNGE, 5)?, words(COMBO, 5)?, words(MOVEMENT, 4)?, words(DAMAGES, 8)?, f(SHARED, 5)?);
            let fl = |v: u32| f32::from_bits(v);
            Words {
                block_chance: three(BLOCK_CHANCE)?,
                jump_distance: float(JUMP_DISTANCE)?,
                jump_chance: { let v = i(JUMP_CHANCE, 2)?; (v[0], v[1]) },
                roll_chance: i(ROLL_CHANCE, 1)?[0],
                roll_counter_distance: f(ROLL_COUNTER, 1)?[0],
                roll_counter_chance: i(ROLL_COUNTER + 4, 1)?[0],
                overhead_distance: f(OVERHEAD, 1)?[0],
                overhead_chance: i(OVERHEAD + 4, 1)?[0],
                boomerang_range: { let v = f(BOOMERANG_RANGE, 2)?; (v[0], v[1]) },
                boomerang_cooldown: i(BOOMERANG_COOLDOWN, 1)?[0],
                boomerang_chance: three(BOOMERANG_CHANCE)?,
                boomerang_speed: float(BOOMERANG_SPEED)?,
                boomerang_damage: dm[1] as i32,
                flame_range: { let v = f(FLAME_RANGE, 2)?; (v[0], v[1]) },
                flame_cooldown: i(FLAME_COOLDOWN, 1)?[0],
                flame_chance: three(FLAME_CHANCE)?,
                flame_wait: { let v = i(FLAME_WAIT, 2)?; (v[0], v[1]) },
                flame_damage: dm[2] as i32,
                swing_distance: fl(sw[0]),
                swing_cooldown: sw[1] as i32,
                swing_chance: [sw[2] as i32, sw[3] as i32, sw[4] as i32],
                swing_damage: dm[0] as i32,
                lunge_distance: fl(lu[0]),
                lunge_cooldown: lu[1] as i32,
                lunge_chance: [lu[2] as i32, lu[3] as i32, lu[4] as i32],
                lunge_damage: dm[3] as i32,
                combo_distance: fl(co[0]),
                combo_cooldown: co[1] as i32,
                combo_chance: [co[2] as i32, co[3] as i32, co[4] as i32],
                combo_damage: [dm[4] as i32, dm[5] as i32, dm[6] as i32],
                walk_distance: fl(mv[0]),
                orbit_distance: fl(mv[1]),
                hop_distance: fl(mv[2]),
                hop_chance: mv[3] as i32,
                overhead_damage: dm[7] as i32,
                phase2: i(PHASES, 1)?[0],
                phase3: i(PHASES + 4, 1)?[0],
                tick_rate: sh[0],
                jump_step: sh[1],
                boomerang_timeout: sh[2],
                far_weight: sh[4],
            }
        };
        let drain_words = neopets_sim::sisters::DrainWords::read(&words(neopets_sim::sisters::DRAIN_WORDS, 7)?, &words(neopets_sim::sisters::DRAIN_TABLE, 14)?).ok_or("The drain words are short")?;
        let thorn_radius = f32::from_bits(words(neopets_sim::thorn::RADIUS_WORD, 1)?[0]);
        // The Cloud Beast's rows and tuning words, read always so the developer menu can spawn it.
        let cloud_rows = match (index.read(disc, 530), index.read(disc, 531)) {
            (Ok(a), Ok(b)) => neopets_data::monster::MonsterTable::parse(&a, &b).ok().and_then(|t| t.row(1, 0).zip(t.row(2, 0))),
            _ => None,
        };
        let cloud_words = neopets_sim::cloudbeast::Words::read(&words(neopets_sim::cloudbeast::BASE, neopets_sim::cloudbeast::WORDS)?);
        let neck_joints: Vec<usize> = words(neopets_sim::look::NECK_JOINTS_AT, 6)?.into_iter().map(|w| w as usize).collect();
        let sister_words = neopets_sim::sisters::Words::read(&words(neopets_sim::sisters::BASE, neopets_sim::sisters::WORDS)?, words(neopets_sim::sisters::LEACH_DRAIN, 1)?[0]).ok_or("The Faerie Sisters' tuning block is short")?;
        let draik_words = neopets_sim::draik::Words::read(&words(neopets_sim::draik::BASE, neopets_sim::draik::WORDS)?).ok_or("The Draik's tuning block is short")?;
        let ixi_words = neopets_sim::ixi::Words::read(&words(neopets_sim::ixi::BASE, neopets_sim::ixi::WORDS)?).ok_or("The Ixi Chief's tuning block is short")?;
        let mortog_words = neopets_sim::mortog::Words::read(&words(neopets_sim::mortog::BASE, neopets_sim::mortog::WORDS)?).ok_or("The Mortog words are short")?;
        let serpent_words = neopets_sim::serpent::Words::read(&words(neopets_sim::serpent::BASE, neopets_sim::serpent::WORDS)?).ok_or("The Plague Serpent's tuning block is short")?;
        let duel_words = neopets_sim::fyora::Words::read(&words(neopets_sim::fyora::WORDS, 2)?).ok_or("The duel words are short")?;
        let faerie_words = {
            use neopets_sim::darkest::*;
            Words::read(&words(BASE, WORDS)?, &words(BASE_B, WORDS_B)?).ok_or("The Darkest Faerie's tuning words are short")?
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
            diag_slides: Vec::new(),
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
            square_held: false,
            talk_prompt: false,
            talk_partner: None,
            talk_with: None,
            dialogue_open: false,
            threads: Vec::new(),
            cut: None,
            cut_target: None,
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
            trails: Default::default(),
            showcase: None,
            corner: None,
            showcased: Default::default(),
            hero_trail: None,
            enemy_trails: HashMap::new(),
            dust_ids,
            shot_fx: vec![None; neopets_sim::projectile::SLOTS],
            clip_fx: HashMap::new(),
            last_fx: String::new(),
            save_glow: None,
            placed_fx: HashMap::new(),
            r_displays: Default::default(),
            r_fx: [None; 2],
            hud_fill_points: [None; 2],
            hud_loss: [None; 2],
            hud_prev: [i32::MAX; 2],
            char_lights: HashMap::new(),
            shadow_hulls: shadow_hulls_loaded,
            dig_drop: None,
            bend_last: None,
            swap_at: [0.0; 3],
            attack_fx: AttackFx::default(),
            obj_yaw: HashMap::new(),
            obj_offset: HashMap::new(),
            shake: Default::default(),
            shake_now: None,
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
            aim: Default::default(),
            rope: Default::default(),
            rope_hero: None,
            rope_y: 0.0,
            rope_pack: 0,
            rope_aim: None,
            weapon_drawn: false,
            shield_raised: false,
            weapon: None,
            shield: None,
            items,
            reach_tor,
            reach_roberta,
            magic,
            last_mote: String::new(),
            spit_fx: [None; 2],
            mote_objects: Vec::new(),
            orbit_angle: 0.0,
            charm_words,
            last_status: String::new(),
            rebind: Vec::new(),
            r_attack: None,
            hero_shots: Vec::new(),
            candidates: Default::default(),
            lock: None,
            fight_target: None,
            shot_rows,
            hero_index: 0,
            parked,
            swap_mode: neopets_sim::swap::rule_m(flags.contains(&220), flags.contains(&248)),
            forced_switch: false,
            swap: None,
            l1_latch: false,
            r1_latch: false,
            last_swap: String::new(),
            catch: Default::default(),
            carried: None,
            hold: None,
            hang: None,
            ledge: None,
            line_hang: None,
            line_cooldown: 0,
            touched: false,
            climb: None,
            ladder: None,
            grab_cooldown: 0,
            panel: None,
            side_touch: false,
            face_count: 0,
            stick_centred: true,
            gesture: Default::default(),
            gesture_words,
            pending_off: Default::default(),
            juppie_words,
            gnarfas_words,
            king_words,
            sister_words,
            sister_group: Default::default(),
            drain: None,
            drain_words,
            draik_words,
            ixi_words,
            faerie_words,
            reserves: Vec::new(),
            duel: [None; 4],
            duel_words,
            serpent_words,
            mortog_words,
            mortog_zone: None,
            duel_summons: Vec::new(),
            boss_hits: Vec::new(),
            last_boss: String::new(),
            last_summon: String::new(),
            petpet: None,
            pet_char: None,
            pet_spawn: None,
            pet_pending: Default::default(),
            pet_bonus,
            pet_low,
            last_pet: String::new(),
            pet_food,
            pet_reserve: Default::default(),
            feeding: None,
            class20_waits,
            knight_words,
            gelert_words,
            hit_list: Vec::new(),
            swing_serial: 0,
            l1_held: false,
            reed_point: None,
            head_look: Default::default(),
            neck_joints,
            cloud_rows,
            cloud_on: std::env::var_os("NEOPETS_CLOUD_BEAST").is_some(),
            cloud_place_hero: false,
            cloud_words,
            cloud_arena: None,
            cloud_defeated: Default::default(),
            cloud_bursts: Vec::new(),
            attack_mgr: Default::default(),
            alarms: HashMap::new(),
            thorns: Default::default(),
            thorn_radius,
            thorn_shots: [None; neopets_sim::projectile::SLOTS],
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
            chain_arms: Vec::new(),
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
            mote_panel: Default::default(),
            mote_table,
            textbox: Default::default(),
            speaker: (Vec::new(), [128; 3]),
            hero_colour,
            box_labels,
            labels,
            box_words,
            death: neopets_sim::death::DeathFlow::new(),
            penalty_due: false,
            teardown_wanted: false,
            credits_wanted: false,
            credits_fired: false,
            characters: Vec::new(),
            slot_regions: HashMap::new(),
            slot_items: HashMap::new(),
            slot_arrived: Default::default(),
            window_was: false,
            pending_slot: None,
            region_wanted: Vec::new(),
            next_character: 0,
            spawned: false,
            generation: 0,
            menu_open: false,
            rumble: Default::default(),
            rumble_presets: Vec::new(),
            mote_glow_rgb: [[0; 3]; 14],
            landing_rumble: (5, 0.0, 0.0),
            motor: None,
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
            cheats: Cheats::default(),
            hero_flash: 0,
            books: Default::default(),
            book_prompt: false,
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
        play.entity_setup();
        play.activated();
        // Flags already set when play starts are not risings (no stingers for them).
        play.flags_seen = play.state.flags.clone();
        play.load_attack_tables(&boot_bytes);
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
        // Cells are keyed by height too (2-unit bands), so stacked floors are separate nodes.
        let key = |x: f32, z: f32, y: f32| ((x / step).round() as i32, (z / step).round() as i32, (y / 2.0).round() as i32);
        let ground = |x: f32, z: f32, y: f32| {
            let h = self.world.probe(x, z, y + 2.5);
            if h.found {
                return Some((h.height, false));
            }
            let h = o?.probe(x, z, y + 2.5);
            h.found.then_some((h.height, true))

        };
        let start = key(p[0], p[2], p[1]);
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
                let (nx, nz) = ((k.0 + dx) as f32 * step, (k.1 + dz) as f32 * step);
                let Some((h, other)) = ground(nx, nz, y) else { continue };
                if h > y + 1.5 {
                    continue;
                }
                let n = key(nx, nz, h);
                if came.contains_key(&n) {
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
                    Goal::PointAt(t) => !other && (t[0] - nx).hypot(t[1] - nz) <= 3.0 && (h - t[2]).abs() < 2.5,
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
        // NEOPETS_NO_OTHER: the other world is never drawn (diagnostic).
        if std::env::var_os("NEOPETS_NO_OTHER").is_some() {
            return false;
        }
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
    /// The pack of a world id (eW), unless it pairs with the current pack.
    fn world_pack(&self, id: u32) -> Option<u32> {
        let areas = &self.boot.areas;
        let slot = areas.world_ids.iter().position(|&w| w == id)?;
        let target = areas.packs[slot];
        (!self.boot.paired(self.world.pack, target)).then_some(target)
    }

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
        // The pending slot (SetWorldLoadOverRide) stands in for the cell's exit value, and is
        // set back to -1 whatever the step decides.
        let target = match self.pending_slot.take() {
            Some(slot) => self.world_pack(slot),
            None => self.neighbour_at(p[0], p[2]),
        };
        let Some(target) = target else { return };
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
        // The traversal holds of the world left behind end with the arrival.
        (self.ladder, self.ledge, self.hang, self.climb, self.panel, self.hold) = (None, None, None, None, None, None);
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

    /// SPEC-0027 Camera initialisation at a world load: a scripted play of the world left
    /// behind ends with it (its track belongs to that world's table, SPEC-0030 row 294), and
    /// the camera is attached to the hero. The old world's hold on the hero ends too (the
    /// arrival places the hero afresh; this build's hold flag stands for the scripted state).
    /// Without this, 1176's won or lost round (HoldHero and camera 5 at end mode 1, then the
    /// load of 1023) left the hero held in the village: 1023's zone-loader thread waits there
    /// for good, so the no-threads release never came.
    fn drop_camera_play(&mut self) {
        self.cut = None;
        self.cut_target = None;
        self.held = false;
        self.hero_walk = None;
    }

    /// The current world's Init (a world loaded by a door job).
    fn run_init(&mut self) {
        let mut init = level::on_init(self.world.pack, &mut self.state);
        init.extend(self.group_setup());
        self.apply(&init);
        self.entity_setup();
    }

    /// The module's shared entity-control table (SPEC-0043), set up at its Init.
    fn entity_setup(&mut self) {
        let pack = self.world.pack;
        let cmds = level::entity_setup(pack, &mut self.state);
        self.entity_commands(pack, cmds);
    }

    /// The host side of the shared entity-control routine's requests: walks (WalkEntity or
    /// WalkEntityAndOrient, no wait), one-shot plays and loops; an idle loop asked for again
    /// while its clip already plays does not restart it.
    fn entity_commands(&mut self, pack: u32, cmds: Vec<neopets_sim::entityctl::Command>) {
        use neopets_sim::entityctl::Command;
        for c in cmds {
            let a = match c {
                Command::Walk { entity, locator, stop, orient } => Action::WalkEntity { entity, locator, wait: false, orient, stop },
                Command::Play { entity, anim } => Action::EntityAnim { entity, anim, looped: false, wait: false },
                Command::Loop { entity, anim, seconds } => {
                    let playing = self.characters.iter().find(|ch| ch.pack == pack && ch.entity == Some(entity)).is_some_and(|ch| ch.anim.clip_state().action == anim);
                    if seconds == 0.0 && playing {
                        continue;
                    }
                    Action::EntityAnim { entity, anim, looped: true, wait: false }
                }
            };
            self.apply(&[a]);
        }
    }

    /// SPEC-0048 set-up of the current world's type-40 groups at its load: a kind-1 chest's
    /// object 0 is drawn at its start (state 9), or at its last frame (finished) when its
    /// collected entry is set.
    fn group_setup(&self) -> Vec<Action> {
        (0..self.world.groups.len()).flat_map(|i| self.group_setup_one(i)).collect()
    }

    /// The built-in set-up of one script group by its kind (also ReActivateScriptGroup).
    fn group_setup_one(&self, i: usize) -> Vec<Action> {
        let mut out = Vec::new();
        if let Some(g) = self.world.groups.get(i) {
            let Some(obj) = g.objects.first().and_then(|&o| usize::try_from(o).ok()) else { return out };
            match g.kind {
                1 => out.push(Action::InitObjOn { obj, end: self.state.collected.contains(&g.key) }),
                // Kind 2 (the element-sealed chest, SPEC-0048 Element gate on hits, Set-up): object
                // 0 drawn, object 1 drawn stopped; once opened (its key row set) both breakables
                // disabled, object 1 hidden and effect 0 stopped.
                2 => {
                    out.push(Action::InitObjOn { obj, end: false });
                    let o1 = g.objects.get(1).and_then(|&o| usize::try_from(o).ok());
                    if self.state.collected.contains(&g.key) {
                        out.extend(g.breakables.iter().take(2).filter_map(|&b| usize::try_from(b).ok()).map(|id| Action::SetBreakableActive { id, on: false }));
                        out.extend(o1.map(|obj| Action::DeactivateObj { obj }));
                        out.extend(g.effects.first().and_then(|&e| usize::try_from(e).ok()).map(|slot| Action::StopEffect { slot }));
                    } else {
                        out.extend(o1.map(|obj| Action::InitObjOn { obj, end: false }));
                    }
                }
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
            self.entity_setup();
            self.other = Some(std::mem::replace(&mut self.world, current));
        }
    }

    /// SPEC-0006 Make current: the new world's script runs its activate-world routine.
    fn activated(&mut self) {
        // An Update's per-tick undamageable set belongs to the world that was current.
        self.state.undamageable_now.clear();
        // The world's own region (its embedded type-32 characters), slot 255, once per load.
        let pack = self.world.pack;
        if !self.slot_regions.contains_key(&(pack, EMBEDDED_SLOT)) {
            self.slot_regions.insert((pack, EMBEDDED_SLOT), pack);
            self.region_wanted.push((pack, EMBEDDED_SLOT, pack));
        }
        if self.seen_pack != self.world.pack {
            // The old current world's Exit (its leave step), when it is still resident.
            let old = self.seen_pack;
            if let Some(w) = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == old) {
                let anims = w.object_anims.clone();
                level::on_exit(old, &mut self.state, &|o| anims.get(o).is_some_and(|a| a.drawn && a.state != 4));
            }
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
        self.thorns.reset();
        self.thorn_shots = [None; neopets_sim::projectile::SLOTS];
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
            Wait::Item(item) => self.state.has_item(item),
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
                    self.state.reset_levels();
                    self.arrive(landing);
                    if let Some(&pack) = self.boot.areas.packs.get(slot) {
                        self.job = Some(Job::ReloadTransition(pack));
                    }
                    self.last_door = format!("script: left the interior to {}, reloading slot {slot}", self.world.pack);
                    self.activated();
                }
                // SPEC-0030 rows 18, 22 and 63 on the current world's movable blocks (an index
                // out of range does nothing).
                Action::ActivateBlock { block, on } => {
                    if let Some(b) = self.world.blocks.get_mut(block) {
                        b.active = on as i32;
                    }
                }
                Action::MoveBlock { block, locator } => {
                    // The placement probes the ground with the objects as they stand now.
                    self.owner_pose_step();
                    if let (Some(&(x, _)), true) = (self.world.entries.get(locator), block < self.world.blocks.len()) {
                        let w = &mut self.world;
                        let mut b = w.blocks[block].clone();
                        b.move_to(x, |px, pz, c| {
                            let h = w.probe(px, pz, c);
                            h.found.then_some(h.height)
                        });
                        w.blocks[block] = b;
                    }
                }
                Action::ReacquireBlock { block, ceiling } => {
                    self.owner_pose_step();
                    if block < self.world.blocks.len() {
                        let w = &mut self.world;
                        let mut b = w.blocks[block].clone();
                        b.reacquire(
                            |px, pz, c| {
                                let h = w.probe(px, pz, c);
                                h.found.then_some(h.height)
                            },
                            ceiling,
                        );
                        w.blocks[block] = b;
                    }
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
                    self.cut_target = None;
                    self.held = true;
                    return (wait && keyed).then_some(Wait::Camera);
                }
                Action::PlayTarget { track, wait, end, speed, target } => {
                    let keyed = !self.world.tracks.get(track)?.keys.is_empty();
                    self.cut = Some(neopets_sim::scene::CameraPlay::new(track, end, speed));
                    self.cut_target = Some((pack, target));
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
                Action::PlayFmv { movie } => {
                    self.state.queue_movie(movie);
                }
                Action::StartEffect { slot, camera } => self.start_placed(pack, slot, camera),
                Action::SwitchEffect { slot, on } => self.switch_placed(pack, slot, on),
                Action::AddAnimObjSound { obj, cue, a, b } => {
                    let w = if self.world.pack == pack { Some(&mut self.world) } else { self.other.as_mut().filter(|o| o.pack == pack) };
                    if let Some(w) = w.filter(|w| obj < w.object_anims.len()) {
                        w.object_anims[obj].add_sound(&w.motions[obj], cue, a, b);
                    }
                }
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
                    self.display_mask.insert(pack, 2 * w.mask(neopets_sim::wheel::copy_of(pack).shift));
                }
                Action::WheelPattern => {
                    let mut w = level::wheel_of(pack, &self.state);
                    let rng = &mut self.rng;
                    w.new_pattern(&mut || rng.range(0, 7));
                    level::set_wheel(pack, &mut self.state, &w);
                    self.display_mask.insert(pack, 2 * w.mask(neopets_sim::wheel::copy_of(pack).shift));
                }
                Action::WheelSpin => {
                    self.wheel_spin = Some((pack, neopets_sim::wheel::Spin::default()));
                    return Some(Wait::WheelSpin);
                }
                Action::WheelResult => {
                    let mut w = level::wheel_of(pack, &self.state);
                    let won = w.result();
                    level::set_wheel(pack, &mut self.state, &w);
                    self.display_mask.insert(pack, 2 * w.mask(neopets_sim::wheel::copy_of(pack).shift));
                    let (win, loss) = neopets_sim::wheel::copy_of(pack).sounds;
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
                Action::AttachObj { obj, host, locator } => {
                    let w = if self.world.pack == pack { Some(&mut self.world) } else { self.other.as_mut().filter(|o| o.pack == pack) };
                    if let Some(w) = w.filter(|w| obj < w.objects.len() && w.headers.get(host).and_then(|h| h.as_ref()).is_some_and(|h| locator < h.locators.len())) {
                        w.obj_links.insert(obj, (host, locator));
                    }
                }
                Action::GroupEffectsToObjLocators { group, host, locator } => {
                    let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
                    let fx: Vec<usize> = w.and_then(|w| w.groups.get(group)).map(|g| g.effects.iter().filter_map(|&e| usize::try_from(e).ok()).collect()).unwrap_or_default();
                    for (j, slot) in fx.into_iter().enumerate() {
                        if let Some(at) = self.anim_locator(pack, host, locator + j, 0) {
                            if let Some(h) = self.placed_object(pack, slot) {
                                self.effects.set_position(h, at);
                                if let Some(Some(o)) = self.effects.objects.get_mut(h) {
                                    o.position = at;
                                }
                            }
                        }
                    }
                }
                Action::SpeedFactor { obj, factor } => {
                    let w = if self.world.pack == pack { Some(&mut self.world) } else { self.other.as_mut().filter(|o| o.pack == pack) };
                    if let Some(o) = w.and_then(|w| w.object_anims.get_mut(obj)) {
                        o.speed = factor;
                    }
                }
                Action::RegisterAnimTrigger { host, index, routine } => {
                    if pack == self.world.pack && self.world.headers.get(host).and_then(|h| h.as_ref()).is_some_and(|h| h.volumes.iter().any(|v| v.index == index)) {
                        self.world.anim_routines.insert((host, index), routine);
                    }
                }
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
                Action::MoveType { entity, kind } => {
                    if let Some(c) = self.entity_mut(pack, entity) {
                        c.move_type = kind;
                    }
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
                Action::LaunchFireMote { from, to, speed } => {
                    let Some((start, _)) = self.locator(pack, from) else { return None };
                    let hero = self.loco.position();
                    let target = self.locator(pack, to).map_or([hero[0], hero[1] + 1.5, hero[2]], |(p, _)| p);
                    self.shots.submit(neopets_sim::projectile::fire_mote(start, target, speed));
                }
                Action::RandomLoop7 { entity, anims } => {
                    let a = anims[self.rng.range(0, 6).clamp(0, 6) as usize];
                    let c = self.entity_mut(pack, entity)?;
                    c.held = true;
                    c.anim.play(a);
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
                    // ReleaseEntityControl (row 107) requests no animation; ReturnEntityAnimToIdle
                    // (row 110) asks for the entity's own idle, which is animation 0 for
                    // townsfolk. A boss's controller picks its own clips (the Juppie's 0 is a
                    // death clip, and playing it at the intro's release made the hub sink it
                    // again; SPEC-0015 Start of the fight), so a boss keeps its clip.
                    let boss = c.boss.is_some() || c.gnarfas.is_some() || c.king.is_some() || c.knight.is_some() || c.gelert.is_some() || c.sister.is_some() || c.draik.is_some() || c.faerie.is_some() || c.serpent.is_some() || c.ixi.is_some();
                    if !boss {
                        c.anim.play(0);
                    }
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
                Action::TargetActive { target, on } => {
                    let world = if self.world.pack == pack { &mut self.world } else { self.other.as_mut().filter(|o| o.pack == pack)? };
                    if let Some(t) = world.aim_targets.get_mut(target) {
                        t.active = on;
                    }
                }
                Action::TriggerBaggus { index } => {
                    // SPEC-0030 row 64 (R-0116): one record per kind-3 group of the world's
                    // type-40 list, in order; a group with a locator (+0x0C) places the next
                    // record there, its height the ground below the locator's height + 4.0. The
                    // list's shared model is present when a kind-3 group names a candidate
                    // (+0x2C). Not built: the shared model's draw and move (the chunk-29
                    // model-handle array is not specified; R-0119), the burst 2.0 above the
                    // record (its effect is not named) and the breakable-object path. An
                    // unplaced record (pack 735's sixth) spawns nothing here.
                    let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
                    let kind3: Vec<&neopets_data::trigger::ScriptGroup> = world.groups.iter().filter(|g| g.kind == 3).collect();
                    if index >= kind3.len() {
                        return None;
                    }
                    let model = kind3.iter().any(|g| !g.breakables.is_empty());
                    let placed: Vec<i32> = kind3.iter().filter_map(|g| g.locators.first().copied()).collect();
                    let l = placed.get(index).and_then(|&i| world.entries.get(usize::try_from(i).ok()?)).map(|e| e.0);
                    if model {
                        self.cue(178);
                    }
                    let Some(l) = l else { return None };
                    let y = self.ground_in(pack, l[0], l[2], l[1] + 4.0).unwrap_or(l[1]);
                    let spawn = neopets_sim::items::Spawn {
                        id: 145,
                        point: [l[0], y, l[2]],
                        placed: false,
                        charm: false,
                        s0: 0.25,
                        s1: 1.0,
                        delay: 0.0,
                        larg: -1.0,
                        direction: None,
                        flag: 0,
                        map: 0,
                        price: 0,
                    };
                    let spawned = self.spawn_item(pack, spawn);
                    let line = format!("bagguss {index} broken at {:.1},{:.1},{:.1} (item 145 spawned: {spawned}) on tick {}", l[0], y, l[2], self.ticks);
                    self.last_fx = if self.last_fx.starts_with("bagguss") { format!("{}; {line}", self.last_fx) } else { line };
                }
                Action::AiBehavior { entity, code: _ } => {
                    let c = self.entity_mut(pack, entity)?;
                    c.held = false;
                    c.script_walk = None;
                    if let Some(e) = c.enemy.as_mut() {
                        e.calm();
                    }
                }
                Action::MainCue { cue } => self.cue(cue),
                Action::EffectAt { id, at, dir } => {
                    let f = unit([dir[0], 0.0, dir[2]]);
                    let f = if f == [0.0; 3] { [0.0, 0.0, 1.0] } else { f };
                    let table = self.effect_table.clone();
                    self.effects.one_shot(&table, id, at, [[f[2], 0.0, -f[0]], [0.0, 1.0, 0.0], f], &mut self.rng);
                }
                Action::DisableInvisibility { entity } => {
                    if let Some(e) = self.entity_mut(pack, entity).and_then(|c| c.enemy.as_mut()) {
                        e.visible = true;
                    }
                }
                Action::SetInvulnerable { entity, on } => {
                    if let Some(c) = self.entity_mut(pack, entity) {
                        c.invulnerable = on;
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
                    if let Some(c) = self.characters.iter_mut().find(|c| c.pack == pack && c.entity == Some(entity) && c.dying.is_none() && c.hp > 0 && !c.enemy.as_ref().is_some_and(|e| e.is_dead())) {
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
                Action::RefillHealth => self.hero_hp = self.hero_max,
                Action::HeroHpAtLeast { percent } => {
                    let floor = (self.hero_max as f32 * percent / 100.0) as i32;
                    self.hero_hp = self.hero_hp.max(floor);
                }
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
                Action::GroupEffectsStop { group } => {
                    let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
                    let fx: Vec<usize> = w.and_then(|w| w.groups.get(group)).map(|g| g.effects.iter().filter_map(|&e| usize::try_from(e).ok()).collect()).unwrap_or_default();
                    for slot in fx {
                        self.apply(&[Action::StopEffect { slot }]);
                    }
                }
                Action::DamageHero { amount } => {
                    let facing = self.loco.facing;
                    self.hero_hit(amount as f32, 0.0, false, facing, "script".into());
                }
                Action::KillHero => {
                    if self.loco.state != hero_states::DEAD {
                        self.default_death();
                    }
                }
                Action::RegisterGroupTriggers { group, routine } => {
                    let members: Vec<u16> = self.world.groups.get(group).map(|g| g.triggers.iter().filter_map(|&t| u16::try_from(t).ok()).collect()).unwrap_or_default();
                    for index in members {
                        if (index as usize) < self.world.capacity {
                            self.world.routines.insert(index, routine);
                            self.world.group_routes.remove(&index);
                        }
                    }
                }
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
                Action::WaitItem { item } => return Some(Wait::Item(item)),
                Action::ActivateTimer { start, limit, up } => self.state.timer.activate(start, limit, up),
                Action::DeactivateTimer => self.state.timer.deactivate(),
                Action::StopTimer => self.state.timer.running = false,
                Action::GameOver => self.death.flag_e = true,
                Action::BossFight { instance } => {
                    // The first hero is hero slot 0 (Tor); the inactive hero is the parked one.
                    let refill_parked = |p: &mut Parked| {
                        p.hp = p.hp_max;
                        p.mp = p.mp_max;
                    };
                    if instance <= 3 {
                        if self.hero_index == 0 {
                            self.hero_hp = self.hero_max;
                            self.hero_mp = self.hero_mp_max;
                        } else if let Some(p) = self.parked.as_mut() {
                            refill_parked(p);
                        }
                    } else {
                        if let Some(p) = self.parked.as_mut() {
                            refill_parked(p);
                        }
                        self.hero_hp = self.hero_max;
                        self.hero_mp = self.hero_mp_max;
                    }
                }
                Action::LongFall => {
                    self.attack = None;
                    self.loco.long_fall_start();
                }
                Action::InitMortog { target, centre, edge, hold } => {
                    // SPEC-0030 row 84: both locators must exist; the zone count starts at 0.
                    if let (Some((a, _)), Some((b, _))) = (self.locator(pack, centre), self.locator(pack, edge)) {
                        let l = (a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2);
                        self.mortog_zone = Some((pack, neopets_sim::mortog::Zone { centre: a, l, hold }, target));
                        self.state.mortogs_on_target = 0;
                    }
                }
                Action::NavmeshAssign { entity, mesh } => {
                    let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
                    let meshes = &world.navmeshes;
                    if let Some(c) = self.characters.iter_mut().find(|c| c.pack == pack && c.entity == Some(entity)) {
                        if let Some(w) = c.walker.as_mut() {
                            w.assign_mesh(usize::try_from(mesh).ok().filter(|&m| m < meshes.len()), meshes);
                        }
                    }
                }
                Action::QueryNavmesh { entity, word } => {
                    let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
                    if let Some(c) = self.characters.iter().find(|c| c.pack == pack && c.entity == Some(entity)) {
                        let kind = if c.row.is_some_and(|r| r.can_fly != 0) { 2 } else { 1 };
                        let id = neopets_sim::nav::closest(&world.navmeshes, c.position, kind);
                        self.state.set_word(pack, word, id + 1);
                    }
                }
                Action::MortogTimes { hold } => {
                    if let Some((_, z, _)) = self.mortog_zone.as_mut() {
                        z.hold = hold;
                    }
                }
                Action::WaitObj { obj } => return Some(Wait::ObjAnim(pack, obj)),
                Action::GroupEffects { group } => {
                    let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
                    let fx: Vec<usize> = w.and_then(|w| w.groups.get(group)).map(|g| g.effects.iter().filter_map(|&e| usize::try_from(e).ok()).collect()).unwrap_or_default();
                    for slot in fx {
                        self.start_placed(pack, slot, true);
                    }
                }
                Action::LoadOverride { slot } => {
                    if (0..193).contains(&slot) && pack == self.world.pack {
                        self.pending_slot = Some(slot as u32);
                        self.last_door = format!("pending slot {slot} on tick {}", self.ticks);
                    }
                }
                // SPEC-0030 rows 46, 47 and 55 on the current world's breakables.
                Action::Break { id } => {
                    if self.world.breakables.get(id).is_some_and(|b| b.state == 1) {
                        self.break_one(id);
                    }
                }
                Action::SetBreakableActive { id, on } => {
                    if let Some(b) = self.world.breakables.get_mut(id) {
                        b.state = on as i32;
                    }
                }
                Action::MakeBreakable { id, on } => {
                    if let Some(b) = self.world.breakables.get_mut(id) {
                        b.state = match (b.state, on) {
                            (1, false) => 3,
                            (3, true) => 1,
                            (s, _) => s,
                        };
                    }
                }
                Action::HeroStatus { code, seconds } => {
                    self.hero_status(code, seconds);
                }
                Action::ItemActivate { slot, ordinal, snap, placed } => {
                    if let Some(p) = self.slot_items.get(&(pack, slot)).and_then(|v| v.get(ordinal)).copied() {
                        self.scripted_add(pack, p, snap, placed);
                    }
                }
                Action::AddItem { locator, item } => {
                    if let Some(&(p, _)) = self.world.entries.get(locator) {
                        let charm = self.item_kind(item) == 17;
                        let spawn = neopets_sim::items::Spawn {
                            id: item,
                            point: p,
                            placed: true,
                            charm,
                            s0: 1.0,
                            s1: 1.0,
                            delay: 0.0,
                            larg: -1.0,
                            direction: None,
                            flag: 0,
                            map: 0,
                            price: 0,
                        };
                        self.spawn_item(pack, spawn);
                        self.last_fx = format!("item {item} added at locator {locator} on tick {}", self.ticks);
                    }
                }
                Action::ObjOffset { obj, y } => {
                    self.obj_offset.insert((pack, obj), y);
                }
                Action::FreezeTrial => {
                    let t = self.state.timer.value;
                    self.state.set_word(pack, 3, t.to_bits() as i32);
                    self.state.timer.activate(t, t, false);
                    self.last_fx = format!("time trial finished in {t:.2} s on tick {}", self.ticks);
                }
                Action::DisplayMask { mask } => {
                    self.display_mask.insert(pack, 2 * mask);
                }
                Action::CollisMask { mask } => {
                    let w = if self.world.pack == pack { Some(&mut self.world) } else { self.other.as_mut().filter(|o| o.pack == pack) };
                    if let Some(w) = w {
                        w.set_collis_mask(mask as u16);
                    }
                }
                Action::ObjYaw { obj, degrees } => {
                    self.obj_yaw.insert((pack, obj), degrees);
                }
                Action::CameraShake { t, a, s, p4, r, phi, z } => self.shake.start(t, a, s, p4, r, phi, z),
                Action::CameraShakeMode { kind, duration, strength } => self.shake.start_mode(kind, duration, strength),
                Action::DeactivateGroup { group } => {
                    if let Some(g) = self.world.groups.get(group).cloned() {
                        for t in g.triggers.iter().filter_map(|&t| u16::try_from(t).ok()) {
                            self.world.routines.remove(&t);
                            self.world.group_routes.remove(&t);
                            self.world.disabled_triggers.insert(t);
                        }
                        let objs: Vec<Action> = g.objects.iter().filter_map(|&o| usize::try_from(o).ok()).map(|obj| Action::DeactivateObj { obj }).collect();
                        self.apply(&objs);
                    }
                }
                Action::ReactivateGroup { group } => {
                    if let Some(g) = self.world.groups.get(group).cloned() {
                        for t in g.triggers.iter().filter_map(|&t| u16::try_from(t).ok()) {
                            self.world.disabled_triggers.remove(&t);
                        }
                    }
                    let setup = self.group_setup_one(group);
                    self.apply(&setup);
                }
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
                    // SetWhinnySwitches lists several pens (1398: four), each call one entry.
                    self.world.pens.push(neopets_sim::whinny::Pen { inner, entrance, point, occupied: false });
                    // NEOPETS_TRACE_PEN: the pen's meshes as SetWhinnyPenmesh sets them (diagnostic).
                    if std::env::var_os("NEOPETS_TRACE_PEN").is_some() {
                        let m = &self.world.navmeshes;
                        let info = |i: usize| m.get(i).map(|n| (n.kind, n.len(), n.locate_full(point).is_some()));
                        println!("      Whinny pen: point {point:?}; inner {inner} (kind, triangles, holds point) {:?}; entrance {entrance} {:?}", info(inner), info(entrance));
                    }
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
                    self.slot_arrived.remove(&(pack, slot));
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

    /// A target play's anchor point: the hero at half its height, or an animated object's origin.
    fn target_anchor(&self, pack: u32, t: neopets_sim::level::CameraTarget) -> Option<[f32; 3]> {
        match t {
            neopets_sim::level::CameraTarget::Hero => Some(add(self.loco.position(), [0.0, 0.5 * HERO_HEIGHT, 0.0])),
            neopets_sim::level::CameraTarget::Object(i) => self.object_matrix(pack, i).map(|m| [m[12], m[13], m[14]]),
        }
    }

    /// The view the window draws: the camera play's pose (while it plays or holds its last
    /// pose) or the follow camera; eye, forward, up and focal length (SPEC-0027 Projection).
    /// SPEC-0023 section 3, Loss response: the camera-relative world point of a saved screen
    /// point (window fractions `n`, aspect `aspect`): E + 1.1D + (0.5 - x) w R + (0.5 - y) h U
    /// with R the camera's left and U its up. The projection record's q and a terms are taken as
    /// this view's own frustum at depth 1.1 (w and h its full width and height there), so the
    /// point sits under the bar's fill end on screen (this build's reading of P).
    fn hud_point(&self, n: [f32; 2], aspect: f32) -> [f32; 3] {
        let (eye, f, up, focal) = self.view();
        let h = 2.0 * 1.1 * (neopets_sim::camera::SCREEN.1 / 2.0) / focal.max(1.0);
        let w = h * aspect;
        let left = [up[1] * f[2] - up[2] * f[1], up[2] * f[0] - up[0] * f[2], up[0] * f[1] - up[1] * f[0]];
        std::array::from_fn(|k| eye[k] + 1.1 * f[k] + (0.5 - n[0]) * w * left[k] + (0.5 - n[1]) * h * up[k])
    }

    /// The bars' loss effects (SPEC-0023 section 3; SPEC-0034 Fixed users of group 3): when a
    /// bar's value falls and its slot is idle (no live spawner), its effect starts at the world
    /// point of the fill end drawn on the previous tick; a live slot follows its saved screen
    /// point each tick. A fall while the slot is busy keeps the old anchor.
    fn hud_loss_step(&mut self) {
        let values = [self.hero_hp, self.hero_mp];
        let table = self.effect_table.clone();
        for k in 0..2 {
            if let Some((h, ..)) = self.hud_loss[k] {
                let live = self.effects.objects.get(h).and_then(|o| o.as_ref()).is_some_and(|o| o.slots.iter().flatten().any(|&s| self.effects.spawners[s].as_ref().is_some_and(|sp| sp.owner == Some(h))));
                if !live {
                    self.effects.release(h);
                    self.hud_loss[k] = None;
                }
            }
            match self.hud_loss[k] {
                Some((h, n, a)) => {
                    let p = self.hud_point(n, a);
                    self.effects.set_position(h, p);
                }
                None if values[k] < self.hud_prev[k] => {
                    if let Some((n, a)) = self.hud_fill_points[k] {
                        let id = if k == 0 { 0xFA2 } else { 0xFA3 };
                        let p = self.hud_point(n, a);
                        let h = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, id, p, neopets_sim::effect::IDENTITY));
                        self.effects.update(&table, h, None, &mut self.rng);
                        self.hud_loss[k] = Some((h, n, a));
                    }
                }
                None => {}
            }
        }
        self.hud_prev = values;
    }

    pub fn view(&self) -> ([f32; 3], [f32; 3], [f32; 3], f32) {
        let (eye, forward, up, focal) = match self.cut.filter(|c| c.state != 0).and_then(|c| c.last) {
            Some(p) => match self.cut_target.and_then(|(pack, t)| self.target_anchor(pack, t)) {
                // The hero's anchor is taken at half its height and an object's at its origin
                // (the anchors are not given).
                Some(at) => {
                    let (f, u) = neopets_sim::scene::look_at(p.eye, at);
                    (p.eye, f, u, p.focal)
                }
                None => (p.eye, p.rows[2], p.rows[1], p.focal),
            },
            None => {
                let (e, f) = (self.camera.eye, self.camera.focus);
                ([e[0], e[1], e[2]], [f[0] - e[0], f[1] - e[1], f[2] - e[2]], [0.0, 1.0, 0.0], neopets_sim::camera::FOCAL)
            }
        };
        // SPEC-0027 Per-frame use of the shake: the view (not the camera) moves by the frame's
        // amounts along its level right axis and the world up; with an aim distance it keeps
        // looking at the point that far ahead of the unshaken eye.
        let Some(o) = self.shake_now else { return (eye, forward, up, focal) };
        let f = unit(forward);
        let h = unit([-f[2], 0.0, f[0]]);
        let moved = [eye[0] + o.side * h[0], eye[1] + o.up, eye[2] + o.side * h[2]];
        let forward = match o.aim {
            Some(l) => [eye[0] + l * f[0] - moved[0], eye[1] + l * f[1] - moved[1], eye[2] + l * f[2] - moved[2]],
            None => forward,
        };
        (moved, forward, up, focal)
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
                    self.state.reset_levels();
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
                if pack == 1139 {
                    self.state.talk_five = self.rng.range(1, 5);
                }
                if pack == 1358 || pack == 934 {
                    self.state.talk_raw = self.rng.next_u32();
                }
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
            self.draw_items(true);
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
            self.swing_serial = self.swing_serial.wrapping_add(1);
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
        // Steps 1 to 4 (SPEC-0036 Roberta shot launch), per shot: modes 0 and 1 aim at the
        // lock, modes 2 to 5 at a random visible character within 60 and fan the direction;
        // else the default point 20 ahead and 2 up. The target is the shot's homing target.
        // S = A x D x B x G x E; T is 1, or the attack power table value for a full release.
        let weapon_a = self.weapon.and_then(|i| self.items.get(i)).map_or(0.0, |i| i.stats[0] as f32);
        let b = mote.and_then(|id| self.items.get(id)).map_or(1.0, |i| neopets_sim::mote::one_plus_percent(i.stats[5] as f32));
        let e = self.pet_bonus.damage_factor(self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t));
        let scale = neopets_sim::charm::scale(weapon_a, d, b, self.statuses.damage_scale(), e);
        let t = if full { mote.and_then(|id| neopets_sim::mote::power_index(1, id)).and_then(|k| self.design.attack_power(k)).unwrap_or(1.0) } else { 1.0 };
        let status = (full && row.reaction != -1)
            .then(|| mote.and_then(|id| neopets_sim::mote::status_request(1, id, |k| self.design.attack_power(k))).map(|r| [row.reaction, r[1], r[2], r[3]]).unwrap_or([row.reaction, -1, -1, -1]));
        // The shot's visual record (SPEC-0034): kind = the launch class. SPEC-0036 (R-0114): the
        // step is the main clip's family-position field minus 111, and every shipped Roberta clip
        // has 0 there, so the lookup fails and no table-backed flight or impact is attached.
        // This build keeps step 0's ids as a stand-in so the shots are seen (without them a shot
        // draws only the debug marker); what the original draws for them is asked in R-0120.
        let visual = self.attack_fx.shot_tables.ids(class, 0, mote);
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
            let target = if row.homing <= 1 { self.lock } else { self.shot_target() };
            let aim = target.and_then(|id| self.aim_point(id)).unwrap_or_else(|| default_aim(p, f));
            let mut dir = launch_dir(anchor, aim);
            if (2..=5).contains(&row.homing) {
                let (r1, r2) = (fan_draw(&mut self.rng), fan_draw(&mut self.rng));
                dir = fan(dir, r1, r2);
            }
            self.hero_shots.push(Shot {
                position: anchor,
                dir,
                speed: row.speed,
                radius: row.radius,
                life: row.life,
                homing: row.homing,
                target,
                element: neopets_sim::mote::element(mote),
                scale,
                t,
                class,
                status,
                id: self.attack_fx.next_shot,
                arming: 0,
            });
            if let Some(v) = visual {
                self.attack_fx.shots.insert(self.attack_fx.next_shot, (v, [None; 2], anchor));
            }
            self.attack_fx.next_shot = self.attack_fx.next_shot.wrapping_add(1);
            launched += 1;
        }
        self.last_hit = format!("Roberta launched {launched} shot(s) (class {class}, held {held}, homing {}) on tick {}", row.homing, self.ticks);
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
            let aim = s.target.and_then(|id| self.characters.iter().find(|c| c.id == id && c.dying.is_none())).map(|c| c.id).and_then(|id| self.aim_point(id));
            let before = s.position;
            if !s.step(aim) {
                continue;
            }
            // The ground end: the shot's centre strictly below the probed height; the entry's
            // surface code picks the cue, played once from the shooter's weapon bank.
            let hit = self.world.probe(s.position[0], s.position[2], s.position[1] + 3.0);
            if hit.found && s.position[1] < hit.height {
                if let Some(code) = hit.surface {
                    let cue = neopets_sim::roberta::surface_cue(code as u32);
                    self.sound.play(neopets_sim::sound::WEAPON, cue, neopets_sim::sound::Play { emitter: Some(s.position), ..Default::default() });
                }
                continue;
            }
            // The first character whose bone capsule meets the shot sphere.
            // The Black Knight is never a shot target here (SPEC-0012 G15): its weak-point test
            // reads the shot slots itself, and a shot off the guarding knight pays nothing. Nor is
            // the Juppie (SPEC-0015 Hero actions): shots pass through its body; its mouth bounce
            // and the Act 3 dazed loop are not built. Nor the Darkest Faerie, whose absorb test
            // reads the shot slots itself (SPEC-0054).
            // SPEC-0036 Hit on characters: a candidate passes the eligibility test, or guards
            // while the shot is armed (a guarding target is tested even when not hittable).
            let armed = s.armed();
            let hit = self.characters.iter().enumerate().filter(|(_, c)| c.pack == pack && !c.inactive && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|e| e.is_dead()) && c.townsfolk.is_none() && c.row.is_some()).filter(|(_, c)| (c.guards() && armed) || (!c.guards() && !c.unhittable() && c.knight.is_none() && c.boss.is_none() && c.faerie.is_none() && c.king.as_ref().is_none_or(|k| k.hittable()))).find_map(|(i, c)| {
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
                // The Cloud Beast stand-in: the shot sphere against its body cylinder.
                let cloud = c.stand_in_body().is_some_and(|b| {
                    let (lo, hi) = (b.center[1] - 0.5 * b.length, b.center[1] + 0.5 * b.length);
                    (s.position[0] - b.center[0]).hypot(s.position[2] - b.center[2]) <= b.radius + s.radius && s.position[1] >= lo - s.radius && s.position[1] <= hi + s.radius
                });
                (touches || cloud).then_some(i)
            });
            let Some(i) = hit else {
                // SPEC-0036 World contact: the first solid collider of the cell disarms the shot
                // (its surface byte picks the cue, at the collider); then the cell's surface
                // records (byte +0x4C not 6) met by the shot's path, the last match's byte
                // +0x4D picking the cue. The cue plays once from the weapon bank.
                let mask = self.world.collis_mask;
                let collider = self.world.all_volumes.iter().find(|v| v.solid_for_shot(mask) && v.shot_hits(s.position, s.radius)).map(|v| {
                    let at = match v.shape {
                        neopets_sim::trigger::Shape::Sphere { centre, .. } => centre,
                        neopets_sim::trigger::Shape::Cylinder { base, .. } => base,
                        neopets_sim::trigger::Shape::Rectangle { origin, .. } => origin,
                    };
                    (neopets_sim::roberta::collider_cue(v.surface), at)
                });
                let record = || {
                    let g = self.world.ground.as_ref()?;
                    g.frames_at(s.position[0], s.position[2]).into_iter().filter(|f| f.kind != 6 && f.crossed_by(before, s.position)).last().map(|f| (neopets_sim::roberta::surface_cue(f.surface as u32), s.position))
                };
                match collider.or_else(record) {
                    Some((cue, at)) => {
                        self.sound.play(neopets_sim::sound::WEAPON, cue, neopets_sim::sound::Play { emitter: Some(at), ..Default::default() });
                    }
                    None => keep.push(s),
                }
                continue;
            };
            // SPEC-0036 Quick shot meets a guarding target: it bounces (no damage, nothing paid)
            // and flies on; a full-release shot is not blocked: a hittable target takes it as
            // below, any other is passed with the shot disarmed.
            if self.characters[i].guards() {
                if s.class < 3 {
                    s.bounce(&mut self.rng);
                    keep.push(s);
                    continue;
                }
                let c = &self.characters[i];
                if c.unhittable() || c.knight.is_some() || c.draik.is_some() || c.king.is_some() || c.gnarfas.is_some() {
                    continue;
                }
            }
            // SPEC-0020 Hero actions: Gnarfas's own check removes a shot that meets him before
            // asking for its damage, so it flinches him (49 ticks) with damage 0 and nothing is
            // paid (SPEC-0036 Quick-shot payment).
            if self.characters[i].gnarfas.is_some() {
                self.characters[i].pending_shot = Some(0);
                continue;
            }
            let (t_type, defence) = self.characters[i].row.map_or((0, 0.0), |r| (r.target_type as i32, r.defence));
            let mote = self.equipment.motes[1][2];
            let gst = mote.and_then(|id| self.items.get(id)).map_or(0.0, |i| i.stats[6] as f32);
            let c = if neopets_sim::mote::matches(s.element, t_type) { neopets_sim::mote::one_plus_percent(gst) } else { 1.0 };
            // trunc(max(0, S x C x T - defence)), absorbed when S x C x T > 0 and the result <= 0.
            let (damage, absorbed) = neopets_sim::charm::damage(neopets_sim::charm::scale(s.scale, c, s.t, 1.0, 1.0), 1.0, defence, 1.0);
            // The Plague Serpent and the Undead Draik are not hittable: a shot that meets them
            // pays nothing (SPEC-0036 Quick-shot payment; SPEC-0016 Hero actions).
            let unpaid = self.characters[i].serpent.is_some() || self.characters[i].draik.is_some();
            if s.class < 3 && !unpaid {
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
                // SPEC-0033 Behavior 1C source 2: a positional-mode-0 entry's start requests
                // preset 0 for its duration with the intensity by the camera's distance.
                let s = if e.start * d == 0.0 { l } else { e.start * d };
                if e.positional == 0 && t >= s && t < s + l {
                    let (c, eye) = (self.characters[i].position, self.camera.eye);
                    let d2 = (0..3).map(|k| (c[k] - eye[k]) * (c[k] - eye[k])).sum::<f32>();
                    let m = neopets_sim::rumble::falloff(e.intensity, e.inner, e.outer, d2);
                    if m > 0.0 {
                        self.rumble.request(&self.rumble_presets, 0, e.rumble_seconds, m);
                    }
                }
                if e.id == -1 || e.trail {
                    continue;
                }
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
        // SPEC-0023 section 12: every request starts the portrait swing (a forced, script request
        // is always accepted and never blocked); the gate is the in-play controls gate.
        let forced = std::mem::take(&mut self.forced_switch);
        let accepted = forced || neopets_sim::swap::allowed(self.swap_mode, self.hero_index);
        let blocked = !forced && parked.hp <= 0;
        // The in-play controls gate (SPEC-0053 3.13), as far as this build tracks it: state not
        // 0x19, no box (dialogue) or menu, and game-state code 0 (no swap or door job running).
        // Group A (3.12) is not tested here.
        let gate = self.loco.state != 25 && !self.dialogue_open && !self.menu_open && self.swap.is_none() && self.job.is_none();
        self.hud.swing.start(if self.hero_index == 0 { 1 } else { 0 }, accepted, blocked, gate);
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
        self.forced_switch = true;
        self.request_switch(false, effects, flash);
    }

    /// TriggerActStart(stage) (SPEC-0053 3.7): the swap mode, the hero (stage 1 by a normal
    /// switch request, stages 2 and 3 selected directly with no task and both heroes placed at
    /// the world's entry record 0), the story flag and the heal of the heroes in the mask (hit
    /// points to the maximum, status timers cleared), the Act 2 stash and kit and the Act 3
    /// stash merge (R-0042; SPEC-0025 Item lists).
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
                // Stages 2 and 3 (R-0042): the named hero is bound and spawned as at a new game
                // at entry 0 of the current world; nothing writes its action state.
                self.swap = None;
                if h != self.hero_index {
                    self.exchange_heroes();
                }
                if let Some(&e) = self.world.entries.first() {
                    self.arrive(e);
                }
            }
            None => {}
        }
        match stage {
            // Stage 2 (R-0042): the party inventory and wallet to the stash, the party cleared,
            // then the kit through the normal add: items 150 x 3, 149 x 3, 127 x 1 and coin 135
            // x 1150 (1150 Neopoints to the wallet).
            2 => {
                let s = &mut self.state;
                s.stash = std::mem::take(&mut s.items);
                s.stash_neopoints = std::mem::replace(&mut s.neopoints, 0);
                self.equipment_step();
                for (id, n) in [(150, 3), (149, 3), (127, 1)] {
                    for _ in 0..n {
                        self.give(id);
                    }
                }
                self.state.neopoints += 1150;
            }
            // Stage 3 (SPEC-0025 Item lists): the stash merges back (an item the party does not
            // hold takes the stash's count; Medium for the per-entry reading), the wallets add,
            // and the stash is cleared.
            3 => {
                let s = &mut self.state;
                for (id, n) in std::mem::take(&mut s.stash) {
                    let e = s.items.entry(id).or_insert(0);
                    if *e == 0 {
                        *e = n;
                    }
                }
                s.neopoints += std::mem::take(&mut s.stash_neopoints);
                self.equipment_step();
            }
            _ => {}
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
    /// A diagnostic selection of a hero at a trace's start (the exchange with the parked hero).
    pub fn trace_select_hero(&mut self, hero: usize) {
        if hero != self.hero_index {
            self.exchange_heroes();
        }
    }

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
        self.draw_items(false);
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
            Some(neopets_sim::swap::Step::Start { flash, effects }) => {
                // Step 2: the hero attack pools are reset; the first flag plays sound 0xB0 and
                // starts the first swap effect (4025) at the outgoing hero.
                self.mote_objects.clear();
                self.hero_shots.clear();
                self.r_attack = None;
                if effects {
                    self.cue(0xB0);
                    let table = self.effect_table.clone();
                    let at = self.loco.position();
                    self.swap_at = at;
                    self.effects.one_shot(&table, 0xFB9, at, neopets_sim::effect::IDENTITY, &mut self.rng);
                }
                if flash {
                    self.fade.out([255; 3], 37);
                }
            }
            Some(neopets_sim::swap::Step::Exchange) => self.exchange_heroes(),
            Some(neopets_sim::swap::Step::EndFlags { flash, effects }) => {
                // Step 10: the third swap effect (4027) where the instances were placed (the
                // outgoing hero's position); the second (4026) is never started.
                if effects {
                    let table = self.effect_table.clone();
                    let at = self.swap_at;
                    self.effects.one_shot(&table, 0xFBB, at, neopets_sim::effect::IDENTITY, &mut self.rng);
                }
                if flash {
                    self.fade.fade_in([255; 3], 8);
                }
            }
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
    /// SPEC-0022 Square chooser, the block rows: in state 18 with the stick centred and clip 40
    /// "Let go" (label 20), whose press returns the hero to state 1; otherwise "Grab block"
    /// (label 10) while the face finder finds a face of the hero's current block, whose press
    /// enters state 18 (SPEC-0033 Behaviors 4A.2 and 15A).
    fn block_step(&mut self) -> bool {
        // SPEC-0033 state 21 exits: "Stop Climbing" (label 18) in any sub-mode but 7; Square
        // drops the hero off (state 7, air phase 5, request 67).
        if self.loco.state == hero_states::PANEL {
            if self.panel.is_some_and(|c| c.sub != 7) {
                self.door_prompt = Some(18);
                if self.square_now {
                    self.panel = None;
                    self.loco.drop_off();
                }
            }
            return true;
        }
        // SPEC-0022: on the ladder "Stop Climbing" (label 18); Square returns to state 1.
        if self.loco.state == hero_states::LADDER {
            self.door_prompt = Some(18);
            if self.square_now {
                self.leave_ladder();
            }
            return true;
        }
        if self.loco.state == hero_states::HANG {
            if self.hero.clip_state().action == 73 {
                self.door_prompt = Some(20);
                if self.square_now {
                    self.leave_edge(hero_states::STANCE, 0.0);
                }
            }
            return true;
        }
        if self.loco.state == hero_states::CLIMB {
            return true;
        }
        // State 30's let go: Square with label 20 while hanging (Behavior 44), to state 1.
        if self.loco.state == hero_states::LEDGE {
            if self.hero.clip_state().action == 73 {
                self.door_prompt = Some(20);
                if self.square_now {
                    self.leave_ledge(hero_states::STANCE);
                }
            }
            return true;
        }
        if matches!(self.loco.state, hero_states::LEDGE_UP | hero_states::LEDGE_RISE) {
            return true;
        }
        if self.loco.state == hero_states::BLOCK {
            if self.hero.clip_state().action == neopets_sim::block::ANIM_STILL && self.stick_centred {
                self.door_prompt = Some(20);
                if self.square_now {
                    self.loco.state = hero_states::STANCE;
                    self.hold = None;
                }
            }
            return true;
        }
        if neopets_sim::hero::busy_state(self.loco.state) || !(1..=6).contains(&self.loco.state) {
            return false;
        }
        let (hero, f) = (self.loco.position(), self.loco.facing);
        let Some((k, d)) = neopets_sim::block::current(&self.world.blocks, hero) else { return false };
        let Some(m) = neopets_sim::block::face(&self.world.blocks[k], d, hero, f) else { return false };
        self.door_prompt = Some(10);
        if self.square_now {
            self.loco.enter_block();
            self.hold = Some(neopets_sim::block::Hold { block: k, face: m, latched: false });
        }
        true
    }

    /// State 18 each tick (SPEC-0033 Behavior 4A.2): with no block record the hero goes to state
    /// 1; else the hold's step sets the clip, the facing (the approach walk keeps its stick
    /// motion; the grab clears the motion) and starts the block moves, with cue 11 and the
    /// strain voice (Tor 33, Roberta 57) on an accepted push.
    fn hold_step(&mut self, input: &HeroInput) {
        self.stick_centred = neopets_sim::hero::speed_value(input) <= 0.0;
        self.edge_step(input);
        if self.loco.state != hero_states::BLOCK {
            self.hold = None;
            return;
        }
        let Some(mut hold) = self.hold.filter(|h| h.block < self.world.blocks.len()) else {
            self.hold = None;
            self.loco.state = hero_states::STANCE;
            return;
        };
        let v = neopets_sim::hero::speed_value(input);
        let stick = self.loco.target.map(|c| c * v);
        let clip = self.hero.hold_clip();
        let step = hold.tick(&mut self.world.blocks, self.loco.position(), self.loco.facing, stick, clip);
        self.hold = Some(hold);
        if let Some((dir, rate)) = step.steer {
            self.loco.steer_toward(dir, rate);
        }
        if let Some(p) = step.place {
            self.loco.body.velocity = [0.0; 3];
            self.loco.body.position = p;
        }
        self.loco.block_request(step.request);
        if step.pushed {
            self.cue(11);
            let voice = if self.hero_index == 0 { 33 } else { 57 };
            self.sound.play(neopets_sim::sound::VOICE, voice, neopets_sim::sound::Play::default());
            if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
                let b = &self.world.blocks[hold.block];
                println!("      block {} push toward {} for {} ticks on tick {} from {:?}", hold.block, b.dir, b.n, self.ticks, b.position);
            }
        }
    }

    /// Diagnostic: each block (index, position, active) and each volume a block can fire (index,
    /// kind, flags, shape) in the current world.
    pub fn block_survey(&self) -> Vec<String> {
        let mut v: Vec<String> = self.world.blocks.iter().enumerate().map(|(k, b)| format!("block {k}: at {:?} size {} active {}", b.position, b.size, b.active)).collect();
        if let Some(g) = &self.world.ground {
            v.push(format!("object patch sets (id, object): {:?}; drawn owners {:?}", g.object_sets(), self.world.owner_poses.keys().collect::<Vec<_>>()));
        }
        v.push(format!("probe at (-19.082, 11.697) ceiling 36.076: {}", self.world.probe(-19.082, 11.697, 36.076).height));
        for (g, grp) in self.world.groups.iter().enumerate() {
            v.push(format!("group {g}: kind {} key {} objects {:?} effects {:?} triggers {:?}", grp.kind, grp.key, grp.objects, grp.effects, grp.triggers));
        }
        for (g, grp) in self.world.groups.iter().enumerate().filter(|(_, g)| matches!(g.kind, 1 | 2)) {
            for &o in &grp.objects {
                let at = usize::try_from(o).ok().and_then(|i| self.world.objects.get(i)?.as_ref()).map(|ob| [ob.rest[12], ob.rest[13], ob.rest[14]]);
                v.push(format!("chest group {g} (kind {}) object {o} at {:?}", grp.kind, at));
            }
        }
        for (pack, pool) in &self.pools {
            for it in pool.slots.iter().flatten() {
                v.push(format!("world item {} of {pack} at {:?}", it.id, it.position));
            }
        }
        // The ladders whose up axis is vertical: the foot point in front of the face, its
        // ground and the ladder's base and top heights.
        for (k, r) in self.world.rails.iter().enumerate().filter(|(_, r)| r.v[1] > 0.99) {
            let base = -r.v[3];
            let (n, u) = ([r.normal[0], r.normal[1], r.normal[2]], [r.u[0], r.u[1], r.u[2]]);
            // A point on the face plane at the U start and the base height, then 1.0 out and
            // 0.8 along U.
            let o = [-(r.normal[3]) * n[0] - (r.u[3]) * u[0], base, -(r.normal[3]) * n[2] - (r.u[3]) * u[2]];
            let foot = [o[0] + n[0] + 0.8 * u[0], base, o[2] + n[2] + 0.8 * u[2]];
            let g = self.world.probe(foot[0], foot[2], base + 3.0).height;
            v.push(format!("ladder {k}: foot ({:.2}, {:.2}) ground {:.2} base {:.2} top {:.2} into {:?}", foot[0], foot[2], g, base, base + r.lv, n.map(|c| -c)));
        }
        v.extend(self.world.all_volumes.iter().filter(|x| x.fires_on_block(self.world.collis_mask)).map(|x| format!("block volume {}: kind {} flags {:#x} routine {:?} {:?}", x.index, x.kind, x.flags, self.world.routines.get(&x.index), x.shape)));
        v
    }

    /// Diagnostic: the hero's state, clip, prompt and hold, and the nearest block's position and
    /// motion (positions rounded to 0.1).
    pub fn block_trace(&self) -> String {
        let r = |v: [f32; 3]| v.map(|c| (c * 10.0).round() / 10.0);
        let hero = self.loco.position();
        let near = neopets_sim::block::current(&self.world.blocks, hero).map(|(k, d)| {
            let b = &self.world.blocks[k];
            (k, (d * 10.0).round() / 10.0, r(b.position), b.moving, neopets_sim::block::face(b, d, hero, self.loco.facing))
        });
        format!("state {} touch {} clip {} prompt {:?} hold {:?} hero {:?} facing {:?} root {:?} block {:?}", self.loco.state, self.side_touch, self.hero.clip_state().action, self.prompt_shown, self.hold.map(|h| (h.block, h.face, h.latched)), r(hero), r(self.loco.facing), self.root_step, near)
    }

    /// Diagnostic: the full stick (forward, side) that points the hero straight at a point.
    pub fn stick_toward(&self, x: f32, z: f32) -> (f32, f32) {
        let p = self.loco.position();
        let (dx, dz) = (x - p[0], z - p[2]);
        let d = dx.hypot(dz).max(1e-6);
        let (f, l) = self.camera.rows();
        let fl = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
        ((dx * f[0] + dz * f[2]) / d / fl, (dx * l[0] + dz * l[2]) / d)
    }

    /// Diagnostic: the stick (forward, side) that points into the held block face once the grab
    /// has latched (the camera-relative form of the scripted walk).
    /// SPEC-0046 Culling, the story layers: bit 20 + i for a visible layer i of 1 to 8 and
    /// 17 + (i - 8) for 9 to 12 (layers 1 and 12 share bit 21), from the story flags.
    pub fn story_layers(&self) -> u32 {
        let f = |n: u16| self.state.flag(n);
        let on = |a: u16, b: u16| f(a) && !f(b);
        let layers = [on(83, 5), on(10, 322), on(64, 283), on(57, 58), on(74, 263), on(85, 298), on(92, 94), f(248), on(332, 100), on(332, 102), on(332, 104), on(332, 106)];
        layers.iter().enumerate().filter(|(_, v)| **v).fold(0, |m, (k, _)| {
            let i = k as u32 + 1;
            m | 1 << if i <= 8 { 20 + i } else { 17 + (i - 8) }
        })
    }

    /// Diagnostic: the frame records of gated model-set entries (gate, start, length), on an
    /// 8-unit lattice over the grid.
    pub fn gated_walls(&mut self) -> Vec<String> {
        let Some(g) = self.world.ground.as_mut() else { return Vec::new() };
        let before = g.mask;
        g.mask = 0xFFFF;
        let g = &*g;
        let [x0, z0, x1, z1] = g.bounds();
        let mut seen = Vec::new();
        let mut out = Vec::new();
        let mut x = x0;
        while x < x1 {
            let mut z = z0;
            while z < z1 {
                for f in g.frames_at(x, z).into_iter().filter(|f| f.gate != 0) {
                    let key = f.start.map(|v| v as i32);
                    if !seen.contains(&key) {
                        seen.push(key);
                        out.push(format!("gate {} kind {} start ({:.1}, {:.1}, {:.1}) L {:.1} H {:.1}", f.gate, f.kind, f.start[0], f.start[1], f.start[2], f.length, f.height));
                    }
                }
                z += 8.0;
            }
            x += 8.0;
        }
        if let Some(g) = self.world.ground.as_mut() {
            g.mask = before;
        }
        out
    }

    /// Diagnostic: what pushes the hero where it stands: the push boxes and spheres that move a
    /// point 1.0 around it, and the frame records whose wall push does.
    pub fn blockers(&self) -> Vec<String> {
        let p = self.loco.position();
        let mut out = Vec::new();
        for (k, (m, b)) in self.world.all_boxes.iter().enumerate() {
            let mut q = p;
            contact::resolve(&mut q, std::slice::from_ref(b), contact::HERO_RADIUS + 0.3);
            if q != p {
                out.push(format!("box {k} mask {m} active {} origin {:?} extents {:?}", self.world.boxes.contains(b), b.origin, b.extents));
            }
        }
        if let Some(g) = &self.world.ground {
            for f in g.frames_at(p[0], p[2]) {
                let w = WallEdge { face: f.face, run: f.run, top: f.top, length: f.length, height: f.height };
                if w.push(p, contact::HERO_RADIUS + 0.3).is_some() {
                    out.push(format!("frame gate {} entry-mask {} kind {} start {:?} face {:?} L {} H {}", f.gate, g.mask, f.kind, f.start, f.face, f.length, f.height));
                }
            }
        }
        for (m, c, r) in &self.world.all_spheres {
            let d = ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt();
            if d < r + 2.0 {
                out.push(format!("sphere mask {m} centre {c:?} radius {r}"));
            }
        }
        out
    }

    /// Diagnostic: block 0 of the current world centred on (x, z), placed on the ground there;
    /// returns its top.
    pub fn diag_block_at(&mut self, x: f32, z: f32) -> Option<f32> {
        let w = &mut self.world;
        let mut b = w.blocks.first()?.clone();
        b.move_to([x, b.position[1], z], |px, pz, cl| {
            let h = w.probe(px, pz, cl);
            h.found.then_some(h.height)
        });
        w.blocks[0] = b;
        Some(w.blocks[0].top())
    }

    /// Diagnostic: the hero put on ladder `k` (state 41, the mount), 0.9 out from its face and
    /// `up` above its base.
    pub fn diag_ladder(&mut self, k: usize, up: f32) {
        let Some(r) = self.world.rails.get(k).copied() else { return };
        let n = [r.normal[0], r.normal[1], r.normal[2]];
        let (u, v) = ([r.u[0], r.u[1], r.u[2]], [r.v[0], r.v[1], r.v[2]]);
        let p = self.loco.position();
        let plane = |q: [f32; 4], p: [f32; 3]| q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3];
        let (a, b, c) = (0.9 - plane(r.normal, p), 0.8 - plane(r.u, p), up - plane(r.v, p));
        self.loco.body.position = std::array::from_fn(|i| p[i] + a * n[i] + b * u[i] + c * v[i]);
        self.loco.facing = [-n[0], 0.0, -n[2]];
        self.ladder = Some(neopets_sim::ladder::Ladder::new(k));
        self.loco.enter_state(hero_states::LADDER, 93);
    }

    pub fn block_trace_stick(&self) -> Option<(f32, f32)> {
        let h = self.hold.filter(|h| h.latched)?;
        let n = self.world.blocks.get(h.block)?.direction(h.face as i32)?;
        let (f, l) = self.camera.rows();
        let fl = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
        Some(((n[0] * f[0] + n[2] * f[2]) / fl, n[0] * l[0] + n[2] * l[2]))
    }

    /// Diagnostic: move block 0 of the current world so that the first walking character of the
    /// world stands 0.5 inside its west face; returns the character and its position.
    pub fn block_onto_npc(&mut self) -> Option<(u64, [f32; 3])> {
        let pack = self.world.pack;
        let c = self.characters.iter().find(|c| c.pack == pack && c.walker.is_some())?;
        let (id, p) = (c.id, c.position);
        let b = self.world.blocks.first()?.clone();
        let x = [p[0] + 0.5 * b.size - 0.5, p[1], p[2]];
        let w = &mut self.world;
        let mut nb = b;
        nb.move_to(x, |px, pz, cl| {
            let h = w.probe(px, pz, cl);
            h.found.then_some(h.height)
        });
        w.blocks[0] = nb;
        Some((id, p))
    }

    /// Leave state 19 or 58 to `state`, moved `push` along the held face, with gravity back on.
    fn leave_edge(&mut self, state: u8, push: f32) {
        let face = self.hang.map(|h| (h.block, h.face)).or(self.climb.map(|c| (c.block, c.face)));
        if let Some(d) = face.and_then(|(k, m)| self.world.blocks.get(k)?.direction(m as i32)) {
            for k in [0, 2] {
                self.loco.body.position[k] += d[k] * push;
            }
        }
        self.hang = None;
        self.climb = None;
        self.loco.body.no_gravity = false;
        self.loco.enter_state(state, if state == hero_states::RUN { 4 } else { 0 });
    }

    /// SPEC-0033 Behaviors 15, 15A and 51, after the hero update: in states 19 and 58 their
    /// step (no gravity; the clips' root motion lifts the hero); otherwise the block-edge grab
    /// (the side-touch latch and a face, on Cross from the ground or in air phases 0 to 5)
    /// enters state 19, and the push-into-face counter with the fit test enters state 58.
    fn edge_step(&mut self, input: &HeroInput) {
        // The grab cooldown (Behavior 20) the panel and ladder grabs share.
        self.grab_cooldown = self.grab_cooldown.saturating_sub(1);
        if self.rope_step(input) {
            return;
        }
        self.panel_step(input);
        if self.loco.state == hero_states::PANEL {
            return;
        }
        self.ladder_step(input);
        if self.loco.state == hero_states::LADDER {
            return;
        }
        self.line_step(input);
        if self.loco.state == hero_states::LINE {
            return;
        }
        let hero = self.loco.position();
        let c = self.hero.clip_state();
        let clip = neopets_sim::block::EdgeClip { action: c.action, progress: c.progress };
        let v = neopets_sim::hero::speed_value(input);
        let stick = self.loco.target.map(|x| x * v);
        let f = self.loco.facing;
        let side_axis = neopets_sim::hero::basis(f)[0];
        let (a, b) = (stick[0] * f[0] + stick[2] * f[2], stick[0] * side_axis[0] + stick[2] * side_axis[2]);
        let forward = a > 0.0 && a.abs() > b.abs();
        if self.ledge_step(input.side, forward) {
            return;
        }
        let step = match self.loco.state {
            hero_states::HANG => self.hang.as_mut().map(|h| h.tick(&self.world.blocks, hero, input.side, forward, self.loco.body.gap, clip)),
            hero_states::CLIMB => {
                let touched = self.touched;
                self.climb.as_mut().map(|h| {
                    h.touched = touched;
                    h.tick(&self.world.blocks, hero, forward, clip)
                })
            }
            _ => {
                self.hang = None;
                self.climb = None;
                let found = neopets_sim::block::current(&self.world.blocks, hero).and_then(|(k, d)| Some((k, neopets_sim::block::face(&self.world.blocks[k], d, hero, f)?)));
                let grab = (input.jump && (1..=10).contains(&self.loco.state)) || (self.loco.airborne() && (0..=5).contains(&self.loco.phase));
                if let (true, true, Some((k, m))) = (grab, self.side_touch, found) {
                    self.hang = Some(neopets_sim::block::Hang::new(k, m));
                    self.loco.enter_state(hero_states::HANG, 72);
                    self.trace_edge("hang", k, m);
                } else if (1..=6).contains(&self.loco.state) {
                    // The counter: reset with no face, up to 15 at one; from 15 the fit test runs
                    // every tick and a pass enters state 58.
                    match found {
                        None => self.face_count = 0,
                        Some((k, m)) => {
                            if self.face_count < neopets_sim::block::FACE_COUNT {
                                self.face_count += 1;
                            } else if neopets_sim::block::fits(&self.world.blocks[k], m, hero, f, a, b) {
                                self.climb = Some(neopets_sim::block::Climb::new(k, m));
                                self.loco.enter_state(hero_states::CLIMB, 70);
                                self.face_count = 0;
                                self.trace_edge("climb", k, m);
                            }
                        }
                    }
                }
                None
            }
        };
        if !matches!(self.loco.state, hero_states::HANG | hero_states::CLIMB) {
            return;
        }
        let Some(step) = step.or(Some(neopets_sim::block::EdgeStep { request: self.loco.action(), ..Default::default() })) else { return };
        self.loco.body.no_gravity = true;
        self.loco.body.velocity = [0.0; 3];
        if let Some(d) = step.facing {
            self.loco.steer_toward(d, 0.5);
        }
        if let Some(p) = step.place {
            self.loco.body.position = p;
        }
        self.loco.block_request(step.request);
        if let Some((state, push)) = step.exit {
            self.leave_edge(state, push);
            self.trace_edge("leave", state as usize, 0);
        }
    }

    /// SPEC-0033 Behaviors 22A and 51: the wall-panel grab from the ground and while jumping
    /// (the cooldown 0), and state 21 each tick: no gravity, the motion cleared, the climb's
    /// hold, requests, clip rate and exits.
    fn panel_step(&mut self, input: &HeroInput) {
        let hero = self.loco.position();
        if self.loco.state != hero_states::PANEL {
            self.panel = None;
            let ground = (1..=6).contains(&self.loco.state);
            let jumping = (7..=10).contains(&self.loco.state) || (43..=46).contains(&self.loco.state);
            if self.grab_cooldown > 0 || !(ground || jumping) || self.world.panels.is_empty() {
                return;
            }
            let v = neopets_sim::hero::speed_value(input);
            let Some(k) = neopets_sim::climb::grab_wall(&self.world.panels, hero, self.loco.facing, self.loco.target, v) else { return };
            self.panel = Some(neopets_sim::climb::Climb::new(k));
            self.loco.enter_state(hero_states::PANEL, 121);
            self.trace_edge("panel", k, 1);
        }
        let Some(mut c) = self.panel else {
            self.leave_ladder();
            return;
        };
        let clip = self.hero.clip_state();
        // In the top-edge hang (sub-mode 17) the stick values are the camera-relative stick
        // vector projected onto the panel: f into the wall, s along the hero's left (SPEC-0033
        // Top-edge hang), so a pure-forward push is blocked only when the camera is square on.
        let (mut f, mut s) = (input.forward, input.side);
        if c.sub == 17 {
            if let Some(t) = self.world.panels.get(c.target) {
                let (cf, cl) = self.camera.rows();
                let flat = |v: [f32; 3]| unit([v[0], 0.0, v[2]]);
                let (cf, cl) = (flat(cf), flat(cl));
                let w: [f32; 3] = std::array::from_fn(|k| input.forward * cf[k] + input.side * cl[k]);
                let n = t.n();
                let h = unit([-n[0], 0.0, -n[2]]);
                f = w[0] * h[0] + w[2] * h[2];
                s = w[0] * h[2] - w[2] * h[0];
            }
        }
        let view = neopets_sim::climb::View { hero, f, s, gap: self.loco.body.gap, clip: (clip.action, clip.progress), cooldown: self.grab_cooldown };
        let sub = c.sub;
        let step = c.tick(&self.world.panels, view);
        self.panel = Some(c);
        if c.sub != sub {
            self.trace_edge("panel sub", c.target, c.sub);
        }
        self.loco.body.no_gravity = true;
        self.loco.body.velocity = [0.0; 3];
        if let Some((f, w)) = step.facing {
            self.loco.steer_toward(f, w);
        }
        if let Some(p) = step.place {
            self.loco.body.position = p;
        }
        if let Some(r) = step.request {
            self.loco.block_request(r);
        }
        if let Some(k) = step.cooldown {
            self.grab_cooldown = k;
        }
        self.loco.rate = step.rate;
        if step.leave {
            self.panel = None;
            self.leave_ladder();
            self.trace_edge("panel off", c.target, 0);
        }
    }

    /// Leave the ladder to state 1 with gravity back on.
    /// SPEC-0033 Behavior 9: the grab-line entry from the jump states (7 to 10, 43 to 46) with
    /// the cooldown expired, the first line of the world whose gate passes and whose reach
    /// passes; then state 28 each tick: no gravity, facing along the line, the attach, the
    /// slide and the let-go (Square shows the let-go label 20).
    fn line_step(&mut self, input: &HeroInput) {
        use neopets_sim::grab;
        self.line_cooldown = self.line_cooldown.saturating_sub(1);
        let hero = self.loco.position();
        if self.loco.state != hero_states::LINE {
            self.line_hang = None;
            let jumping = (7..=10).contains(&self.loco.state) || (43..=46).contains(&self.loco.state);
            if !jumping || self.line_cooldown > 0 {
                return;
            }
            let mask = self.world.collis_mask;
            let Some(k) = self.world.grab_lines.iter().position(|l| (l.mask == 0 || l.mask & mask != 0) && l.reaches(hero)) else { return };
            self.line_hang = Some(grab::Hang::new(k));
            self.loco.enter_state(hero_states::LINE, 112);
            self.trace_edge("grab line", k, 0);
        }
        let Some(mut h) = self.line_hang else {
            self.loco.drop_off();
            return;
        };
        let square = std::mem::take(&mut self.square_now);
        // The search: Cross held with the side stick outside the dead band (40 of 127.5);
        // pushed right seeks on the -B1 side (side word 0), left on the +B1 side (1).
        let mut launch = None;
        if input.cross_held && input.side.abs() > 40.0 / 127.5 {
            let f = self.loco.facing;
            let b1 = neopets_sim::hero::basis(f)[0];
            let side = if input.side > 0.0 { 1 } else { 0 };
            launch = h.seek(&self.world.grab_lines, hero, f, b1, side);
        }
        h.cross = input.cross_held;
        h.playing = self.hero.clip_state().action;
        // Test 4: the touched flag in the slide with the cooldown expired lets go, then the
        // long-fall landing (state 48); the attach's 60-tick cooldown keeps it off at first.
        if launch.is_none() && h.mode == 1 && self.touched && self.line_cooldown < 1 {
            if let Some(line) = self.world.grab_lines.get(h.line).copied() {
                self.line_hang = None;
                self.line_cooldown = grab::COOLDOWN;
                self.loco.line_let_go(line.u.map(|c| c * 15.0));
                self.loco.long_fall_start();
                self.trace_edge("line touched: long fall", h.line, 0);
                return;
            }
        }
        let mut step = h.tick(&self.world.grab_lines, hero, self.loco.body.gap, square);
        if let Some(r) = launch {
            step.request = r;
        }
        self.line_hang = Some(h);
        let Some(line) = self.world.grab_lines.get(h.line).copied() else { return };
        if step.let_go {
            self.line_hang = None;
            self.line_cooldown = grab::COOLDOWN;
            self.loco.line_let_go(line.u.map(|c| c * 15.0));
            self.trace_edge("let go of line", h.line, 0);
            return;
        }
        self.door_prompt = Some(20);
        // Cue 141 is kept playing at the hero while Square is not pressed (started when it is
        // not already playing).
        if !square {
            self.sound.play(neopets_sim::sound::MAIN, 141, neopets_sim::sound::Play { do_not_restart: true, ..Default::default() });
        }
        self.loco.body.no_gravity = true;
        self.loco.body.velocity = [0.0; 3];
        self.loco.steer_toward([line.u[0], 0.0, line.u[2]], 1.0);
        if let Some(p) = step.place {
            self.loco.body.position = p;
        }
        self.loco.block_request(step.request);
    }

    fn leave_ladder(&mut self) {
        self.ladder = None;
        self.loco.body.no_gravity = false;
        self.loco.enter_state(hero_states::STANCE, 0);
    }

    /// SPEC-0033 Behaviors 13 and 22: the ladder grab test from the ground and while jumping
    /// (the cooldown 0), and state 41 each tick: facing the ladder, motion cleared, no gravity,
    /// the mount move, the rung clips, the climb off the top, stepping off, and the jump off.
    fn ladder_step(&mut self, input: &HeroInput) {
        let hero = self.loco.position();
        let v = neopets_sim::hero::speed_value(input);
        if self.loco.state != hero_states::LADDER {
            self.ladder = None;
            let ground = (1..=6).contains(&self.loco.state);
            let jumping = (7..=10).contains(&self.loco.state) || (43..=46).contains(&self.loco.state);
            if self.grab_cooldown > 0 || !(ground || jumping) || self.world.rails.is_empty() {
                return;
            }
            let stick = self.loco.target;
            if let Some(k) = neopets_sim::ladder::grab(&self.world.rails, hero, self.loco.facing, stick, v, false) {
                self.ladder = Some(neopets_sim::ladder::Ladder::new(k));
                self.loco.enter_state(hero_states::LADDER, 93);
                self.trace_edge("ladder", k, 0);
            } else {
                return;
            }
        }
        let Some(mut l) = self.ladder else {
            self.leave_ladder();
            return;
        };
        let c = self.hero.clip_state();
        let step = l.tick(&self.world.rails, hero, input.forward, input.jump, self.loco.body.gap, (c.action, c.progress));
        if std::env::var_os("NEOPETS_TRACE_CLIMB").is_some() {
            let r = self.world.rails[l.rail];
            let pl = |q: [f32; 4]| q[0] * hero[0] + q[1] * hero[1] + q[2] * hero[2] + q[3];
            println!("      ladder tick {} sub {} clip {} {:.2} gap {:.2} forward {} y {:.2} step {:?} d {:.3} x {:.3} v {:.3} place {:?}", self.ticks, l.sub, c.action, c.progress, self.loco.body.gap, input.forward, hero[1], step.request, pl(r.normal), pl(r.u), pl(r.v), step.place);
        }
        self.ladder = Some(l);
        self.loco.body.no_gravity = true;
        self.loco.body.velocity = [0.0; 3];
        self.loco.steer_toward(step.facing, 1.0);
        if let Some(p) = step.place {
            self.loco.body.position = p;
        }
        if let Some(r) = step.request {
            self.loco.block_request(r);
        }
        if step.cue {
            self.cue(140);
        }
        match step.leave {
            Some(true) => {
                // The jump off: -0.5 times the facing, +2.0 up, and the 30-tick cooldown.
                let f = self.loco.facing;
                self.ladder = None;
                self.grab_cooldown = neopets_sim::ladder::COOLDOWN;
                self.loco.jump_off([-neopets_sim::ladder::JUMP_BACK * f[0], neopets_sim::ladder::JUMP_UP, -neopets_sim::ladder::JUMP_BACK * f[2]]);
                self.trace_edge("ladder jump", l.rail, 0);
            }
            Some(false) => {
                self.leave_ladder();
                self.trace_edge("ladder off", l.rail, 0);
            }
            None => {}
        }
    }

    /// SPEC-0033 Behaviors 32, 37 and 51: in the jump states' air phases 0 to 5 the in-air ledge
    /// chooser reads the edge contact (the nearby frame records the acceptance test passes, the
    /// one facing the hero most) and enters state 30, 35 or 47; in those states their step (no
    /// gravity, the motion cleared). Reports whether a ledge state owns the tick.
    fn ledge_step(&mut self, side: f32, forward: bool) -> bool {
        use neopets_sim::ledge::{self, Class, Edge, Ledge};
        let hero = self.loco.position();
        if !matches!(self.loco.state, ledge::HANG | ledge::CLIMB_FALL | ledge::CLIMB_RISE | ledge::DROP) {
            self.ledge = None;
            self.loco.body.no_snap = false;
            if (1..=4).contains(&self.loco.state) && !self.drop_check() {
                return false;
            }
            if self.loco.state != ledge::DROP && !(self.loco.airborne() && (0..=5).contains(&self.loco.phase)) {
                return false;
            }
        }
        if !matches!(self.loco.state, ledge::HANG | ledge::CLIMB_FALL | ledge::CLIMB_RISE | ledge::DROP) {
            let Some(g) = &self.world.ground else { return false };
            let (f, v) = (self.loco.facing, self.loco.body.velocity);
            let toward = |e: &Edge| -(e.face[0] * f[0] + e.face[2] * f[2]);
            let edge = g
                .frames_at(hero[0], hero[2])
                .into_iter()
                .map(|r| Edge { start: r.start, face: r.face, run: r.run, top: r.top, length: r.length, height: r.height, kind: r.kind })
                .filter(|e| e.accepts(hero, f, v, contact::HERO_RADIUS, self.loco.state))
                .max_by(|a, b| toward(a).total_cmp(&toward(b)));
            let Some(edge) = edge else { return false };
            let ground = self.world.ground_at(hero[0], hero[2], hero[1] + 8.0).unwrap_or(ground::MISS);
            let Some(class) = ledge::choose(edge.h(hero), hero[1], ground, self.loco.body.normal[1], true) else { return false };
            let (state, request) = match class {
                Class::Hang => (ledge::HANG, 72),
                Class::Climb if v[1] > 0.0 => (ledge::CLIMB_RISE, 134),
                Class::Climb => (ledge::CLIMB_FALL, 75),
            };
            self.ledge = Some(Ledge::new(state, edge));
            self.loco.enter_state(state, request);
            self.trace_edge("ledge", state as usize, 0);
        }
        let c = self.hero.clip_state();
        // State 37's depth check: the ground 2.0 and 2.5 ahead more than 4.5 below (or none) at
        // both points makes the edge deep.
        let f = self.loco.facing;
        let deep = [2.0, 2.5].iter().all(|&k| self.world.ground_at(hero[0] + f[0] * k, hero[2] + f[2] * k, hero[1] + 1.0).is_none_or(|g| hero[1] - g > 4.5));
        let Some(step) = self.ledge.as_mut().map(|l| {
            if l.state == ledge::DROP {
                l.drop_tick(hero, f, deep, self.loco.body.gap)
            } else {
                l.tick(hero, side, forward, self.loco.body.gap, ledge::Clip { action: c.action, progress: c.progress })
            }
        }) else {
            self.leave_ledge(hero_states::STANCE);
            return true;
        };
        if std::env::var_os("NEOPETS_TRACE_LEDGES").is_some() {
            println!("      ledge tick {} sub {:?} clip {} {:.2} gap {:.2} place {:?} bone0 {:?} facing {:?}", self.ticks, self.ledge.map(|l| l.sub), c.action, c.progress, self.loco.body.gap, step.place, self.hero.main_translations().first(), self.loco.facing);
        }
        self.loco.body.no_gravity = true;
        self.loco.body.velocity = [0.0; 3];
        // While hanging the hero is below the top it holds, inside the step-up reach of the
        // ground constraint; the no-snap flag keeps it there (this build's choice: the spec does
        // not name the flag's writer; R-0106).
        self.loco.body.no_snap = self.loco.state == ledge::HANG;
        if let Some(d) = step.facing {
            self.loco.steer_toward(d, step.rate.unwrap_or(0.5));
        }
        if let Some(p) = step.place {
            self.loco.body.position = p;
        }
        if let Some(r) = step.request {
            self.loco.block_request(r);
        }
        if step.hang {
            if let Some(l) = self.ledge.as_mut() {
                *l = Ledge::new(ledge::HANG, l.edge);
            }
            self.loco.enter_state(ledge::HANG, 72);
            self.trace_edge("ledge", ledge::HANG as usize, 0);
        }
        if let Some(s) = step.exit {
            self.leave_ledge(s);
            self.trace_edge("leave", s as usize, 0);
        }
        true
    }

    /// SPEC-0033 Behavior 49 in the ground states 1 to 4: the ledge-drop test on the frame
    /// records of the hero's cell, in order; the first accepted record decides. Walking or
    /// running (3, 4) toward its outward normal requests the running jump (state 10) unless its
    /// type is 1 or 6; standing or slow walking toward it, or standing with the back to it, drops
    /// into a hang (state 37) unless its type is 3 or 6. Reports whether state 37 was entered.
    /// Not kept: the ground-object kind gates (the hero is taken as on no ground object), the
    /// edge offer's boosted jump, the camera side-offset event and states 5 and 6 (not entered in
    /// this build).
    fn drop_check(&mut self) -> bool {
        use neopets_sim::ledge::{self, Edge, Ledge};
        let Some(g) = &self.world.ground else { return false };
        let (hero, f, v, state) = (self.loco.position(), self.loco.facing, self.loco.body.velocity, self.loco.state);
        for r in g.frames_at(hero[0], hero[2]) {
            let e = Edge { start: r.start, face: r.face, run: r.run, top: r.top, length: r.length, height: r.height, kind: r.kind };
            let Some(negative) = e.drop_gates(hero, v, f, contact::HERO_RADIUS, state) else { continue };
            let accepted = negative || {
                let g0 = self.world.ground_at(hero[0], hero[2], hero[1] + 8.0).unwrap_or(ground::MISS);
                let p = e.drop_probe();
                let hit = self.world.probe(p[0], p[2], hero[1] + 6.0);
                ledge::drop_rule(g0, hit.found.then_some((hit.height, hit.kind)))
            };
            if !accepted {
                continue;
            }
            let toward = e.face[0] * f[0] + e.face[2] * f[2] > 0.0;
            if toward && (3..=4).contains(&state) && !matches!(e.kind, 1 | 6) {
                self.loco.edge_jump();
                self.trace_edge("edge jump", state as usize, 0);
                return false;
            }
            if (toward || state == 1) && !matches!(e.kind, 3 | 6) {
                self.ledge = Some(Ledge::drop_from(e, state));
                self.loco.enter_state(ledge::DROP, 0);
                self.trace_edge("drop", state as usize, 0);
                return true;
            }
            return false;
        }
        false
    }

    /// The drawn hero's offset from its position while hanging from a ledge (state 30): 1.4
    /// along the edge's outward normal, easing to 0 over the pull-up (clip 59). The spec's hang
    /// point is 0.7 behind the face, which draws the body inside the wall (playtest); the
    /// motion keeps the spec's point (the pull-up then lands well onto the top) and only the
    /// drawing moves out (this build's choice; R-0108).
    pub fn hero_draw_offset(&self) -> [f32; 3] {
        let Some(l) = self.ledge.filter(|l| l.state == neopets_sim::ledge::HANG && self.loco.state == hero_states::LEDGE) else { return [0.0; 3] };
        let n = l.edge.normal();
        let len = n[0].hypot(n[2]).max(1e-6);
        let c = self.hero.clip_state();
        let w = if c.action == 59 { (1.0 - c.progress).clamp(0.0, 1.0) } else { 1.0 };
        [1.4 * w * n[0] / len, 0.0, 1.4 * w * n[2] / len]
    }

    /// Diagnostic: frame records whose top is 1.75 to 6.5 above the ground 1.5 out from the
    /// middle of their face, with that stand point (the jump-to-ledge candidates).
    pub fn ledge_survey(&self) -> Vec<String> {
        let Some(g) = &self.world.ground else { return Vec::new() };
        let [x0, z0, x1, z1] = g.bounds();
        let mut seen: Vec<[i32; 3]> = Vec::new();
        let mut out = Vec::new();
        let (mut x, step) = (x0, 2.0);
        while x <= x1 && out.len() < 40 {
            let mut z = z0;
            while z <= z1 && out.len() < 40 {
                for r in g.frames_at(x, z) {
                    let key = r.start.map(|v| (v * 10.0) as i32);
                    if seen.contains(&key) || r.face[1].abs() > 0.2 || r.kind == 3 || r.kind == 6 || r.length < 2.0 {
                        continue;
                    }
                    seen.push(key);
                    let n = [r.face[0], 0.0, r.face[2]];
                    let mid: [f32; 3] = std::array::from_fn(|k| r.start[k] + r.run[k] * 0.5 * r.length + n[k] * 1.5);
                    let top = r.start[1];
                    let Some(gy) = self.world.ground_at(mid[0], mid[2], top) else { continue };
                    let d = top - 0.25 - gy;
                    // The ground just behind the face (on top when the record is a cliff edge).
                    let back: [f32; 3] = std::array::from_fn(|k| mid[k] - n[k] * 2.1);
                    let behind = self.world.ground_at(back[0], back[2], top + 1.0).map_or(f32::NAN, |b| b - top);
                    let cliff = behind.abs() < 0.6;
                    if (1.75..12.0).contains(&d) && (cliff || std::env::var("NEOPETS_TRACE_LEDGES").as_deref() != Ok("cliff")) {
                        out.push(format!("ledge kind {} drop {d:.2} stand {:.1},{:.1} y {gy:.2} face {:.2},{:.2} L {:.1} top-behind {behind:.2}", r.kind, mid[0], mid[2], -n[0], -n[2], r.length));
                    }
                }
                z += step;
            }
            x += step;
        }
        out
    }

    fn leave_ledge(&mut self, state: u8) {
        self.ledge = None;
        self.loco.body.no_gravity = false;
        self.loco.body.no_snap = false;
        self.loco.enter_state(state, if state == hero_states::RUN { 4 } else { 0 });
    }

    fn trace_edge(&self, what: &str, a: usize, b: u8) {
        if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
            println!("      {what} {a} {b} on tick {} at {:?}", self.ticks, self.loco.position());
        }
    }

    /// SPEC-0048 Solid for characters for the region characters (SPEC-0040 Behavior 14A branch
    /// 1): after their motion, each walking character of the current world is pushed out of the
    /// active blocks (not in owner states 18, 19, 30, 41 and 58, which no region character uses).
    fn npc_block_step(&mut self) {
        if self.world.blocks.is_empty() {
            return;
        }
        let pack = self.world.pack;
        let blocks = &self.world.blocks;
        for c in self.characters.iter_mut().filter(|c| c.pack == pack) {
            let Some(wk) = c.walker.as_mut() else { continue };
            let mut p = wk.body.position;
            if neopets_sim::block::contact(blocks, &mut p).hit {
                if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
                    println!("      character {} ({}) against a block on tick {} at {:?}", c.id, c.name, self.ticks, p);
                }
                wk.body.position = p;
                c.position = p;
            }
        }
    }

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

    /// Attach point k of a character as a world frame (SPEC-0005 Attach points).
    fn character_attach(c: &Character, k: usize) -> Option<neopets_data::anim::Affine> {
        let frames = c.anim.bone_frames();
        let &(bone, angles, offset) = c.anim.bones().attach.get(k)?;
        Some(placement(c.position, c.facing).mul(frames.get(bone?)?).mul(&attach_rotation(offset, angles)))
    }

    /// SPEC-0046 Second walk and attachment slots: for each drawn, active, living character, its
    /// slot 0 model in the hand (axes Z0, Y0, -X0 at T0), its slot 6 model at W5, and its slot 1
    /// model while the clip's event bit 0x1000 is on and its selected attack's projectile type is
    /// below 5, at W0 to W4 by the clip's event bits 1, 2, 4, 8 and 0x10. Each entry: the
    /// character id, the slot and the frame. The thrown-model word is reproduced for the
    /// Werelupe King's boomerang (the Black Knight is not in play).
    pub fn character_attachments(&self) -> Vec<(u64, u8, neopets_data::anim::Affine)> {
        let mut out = Vec::new();
        for c in self.characters.iter().filter(|c| !c.inactive && self.pack_drawn(c.pack) && c.enemy.as_ref().is_none_or(|e| !e.dead)) {
            for &(slot, _) in &c.model.attachments {
                // The thrown-model word (SPEC-0046 Held items step 1): while the Werelupe King's
                // boomerang flies (SPEC-0019 Boomerang flight) or the Black Knight's flying sword
                // is out of his hand (sub-states 0 to 4, SPEC-0012 Flying swords), the slot 0
                // model is drawn at it instead of in the hand, spinning 0.6 radians per tick; the
                // spin axis is taken as the vertical (the spec's eased axis is open).
                let thrown = c
                    .king
                    .as_ref()
                    .and_then(|k| k.boomerang)
                    .map(|b| b.position)
                    .or_else(|| c.knight.as_ref().filter(|k| k.sword.sub < 5).map(|k| k.sword.position));
                let frame = match slot {
                    0 if thrown.is_some() => Self::character_attach(c, 0).map(|w| {
                        let (sn, cs) = (0.6 * (self.ticks % 10_000) as f32).sin_cos();
                        let turn = |v: [f32; 3]| [cs * v[0] + sn * v[2], v[1], -sn * v[0] + cs * v[2]];
                        neopets_data::anim::Affine { cols: [turn(w.cols[2]), turn(w.cols[1]), turn(w.cols[0].map(|v| -v))], t: thrown.unwrap_or(w.t) }
                    }),
                    0 => Self::character_attach(c, 0).map(|w| neopets_data::anim::Affine { cols: [w.cols[2], w.cols[1], w.cols[0].map(|v| -v)], t: w.t }),
                    6 => Self::character_attach(c, 5),
                    1 => {
                        let bits = c.anim.frame_bits();
                        let kind = c.enemy.as_ref().and_then(|e| e.selected.and_then(|k| e.p.slots.get(k))).map(|s| s.missile.visual);
                        if bits & 0x1000 != 0 && kind.is_some_and(|v| v < 5) {
                            let k = [1, 2, 4, 8, 0x10].iter().position(|&b| bits & b != 0).unwrap_or(0);
                            Self::character_attach(c, k)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(f) = frame {
                    out.push((c.id, slot, f));
                }
            }
        }
        out
    }

    /// SPEC-0040 Behavior 29: set or clear both flags.
    fn draw_items(&mut self, on: bool) {
        self.set_weapon_drawn(on);
        self.shield_raised = on;
    }

    /// The weapon-drawn flag; a change plays the weapon slot's unsheathe (cue 0) or sheathe
    /// (cue 1) sound (SPEC-0040 Behavior 29; SPEC-0031 Weapon slot uses).
    fn set_weapon_drawn(&mut self, on: bool) {
        if self.weapon_drawn != on && self.weapon.is_some() {
            self.sound.play(neopets_sim::sound::WEAPON, if on { 0 } else { 1 }, neopets_sim::sound::Play::default());
        }
        self.weapon_drawn = on;
    }

    /// SPEC-0040 Behavior 29 per tick: the L1 guard with a shield sets both flags; states 25 and
    /// 28 and a talk clear them.
    fn items_flags_step(&mut self) {
        if self.loco.state == hero_states::GUARD && self.shield.is_some() {
            self.draw_items(true);
        }
        if matches!(self.loco.state, 25 | 28) || self.dialogue_open {
            self.draw_items(false);
        }
    }

    /// The hero's attach point k as a world frame (SPEC-0005 Attach points).
    fn hero_attach(&self, k: usize) -> Option<neopets_data::anim::Affine> {
        let (p, f) = (self.loco.position(), self.loco.facing);
        let frames = self.hero.bone_frames();
        let &(bone, angles, offset) = self.hero.bones().attach.get(k)?;
        Some(placement(p, f).mul(frames.get(bone?)?).mul(&attach_rotation(offset, angles)))
    }

    /// SPEC-0046 Held items, focus hero, steps 1 and 2: the shield (resource 560 + s; on the arm
    /// at W5 while raised, else stowed on the back with axes (-X8, Z8, Y8) at T8, Tor only) and
    /// the weapon (resource 544 + w; in the hand at W0 while drawn, else sheathed with axes (Y6,
    /// Z6, X6) at T6, which only Tor has). Nothing while the cloak is worn or the hero is not
    /// drawn. Each entry: the resource, the frame and the rescale rule of the stowed forms.
    /// SPEC-0046 Hero-item effect matrices; SPEC-0023 section 25: Tor's weapon and shield Mote
    /// glow. With Tor selected and a Mote in word 2 (weapon) or word 1 (shield), common model
    /// 299 + (weapon id - 98) for weapons 98 to 105, or 307 + (shield id - 106), at the item's
    /// drawn frame, tinted with the Mote's colour row. Roberta has none. Each entry is the glow
    /// model, the held item's display resource (whose drawn matrix the glow takes) and the RGB.
    pub fn hero_glows(&self) -> Vec<(u32, u32, [u8; 3])> {
        if self.hero_index != 0 {
            return Vec::new();
        }
        let rgb = |m: Option<u16>| m.and_then(|m| self.mote_glow_rgb.get((m as usize).wrapping_sub(160)).copied());
        let (weapon, shield) = (rgb(self.equipment.motes[0][2]), rgb(self.equipment.motes[0][1]));
        self.hero_items()
            .into_iter()
            .filter_map(|(res, _, _)| match res {
                544..=551 => weapon.map(|c| (299 + res - 544, res, c)),
                560..=564 => shield.map(|c| (307 + res - 560, res, c)),
                _ => None,
            })
            .collect()
    }

    pub fn hero_items(&self) -> Vec<(u32, neopets_data::anim::Affine, HeroItemScale)> {
        let mut out = Vec::new();
        if self.equipment.entries[2][self.hero_index] == Some(126) || self.loco.state == hero_states::DEAD {
            return out;
        }
        let neg = |v: [f32; 3]| v.map(|c| -c);
        if let (Some(id), 0) = (self.shield, self.hero_index) {
            let s = id as i32 - 106;
            if (0..=4).contains(&s) {
                let frame = if self.shield_raised {
                    self.hero_attach(5).map(|w| (w, HeroItemScale::None))
                } else {
                    self.hero_attach(8).map(|w| (neopets_data::anim::Affine { cols: [neg(w.cols[0]), w.cols[2], w.cols[1]], t: w.t }, HeroItemScale::Shield))
                };
                if let Some((w, scale)) = frame {
                    out.push((560 + s as u32, w, scale));
                }
            }
        }
        if let Some(id) = self.weapon.filter(|_| self.hero_index == 0 || self.weapon_drawn) {
            let w = if id <= 110 { id as i32 - 98 } else { id as i32 - 103 };
            if (0..=15).contains(&w) {
                let frame = if self.weapon_drawn {
                    self.hero_attach(0).map(|a| (a, HeroItemScale::None))
                } else {
                    self.hero_attach(6).map(|a| (neopets_data::anim::Affine { cols: [a.cols[1], a.cols[2], a.cols[0]], t: a.t }, HeroItemScale::Weapon))
                };
                if let Some((a, scale)) = frame {
                    out.push((544 + w as u32, a, scale));
                }
            }
        }
        out
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

    /// The weapon reach W (SPEC-0021): Tor's table value for swords 98 to 108, Roberta's for
    /// 111 to 118, 1.5 with no valid weapon.
    /// SPEC-0021 Item showcase, start with quiet 0: when the item has showcase text and the
    /// hero is not busy, the full showcase (mode 4) and its pose task begin. Not built: the
    /// showcase effect, the camera call (0.9425, 10.0), the dropped stinger request 42 / 66 and
    /// the quiet starts of the two give paths. Without text, or with the hero busy, the
    /// screen-anchored showcase of mode 1 starts instead.
    /// Diagnostic: the showcase started for item `id` in the current world.
    pub fn diag_showcase(&mut self, id: u16) {
        let pack = self.world.pack;
        self.start_showcase(id, pack);
        println!("      showcase {id}: {:?}, corner {:?}", self.showcase.as_ref().map(|s| (s.pose, s.floating, String::from_utf8_lossy(&s.text).into_owned())), self.corner);
    }

    fn start_showcase(&mut self, id: u16, pack: u32) {
        // Nothing happens while the full showcase (mode 4) runs.
        if self.showcase.is_some() {
            return;
        }
        let first = self.showcased.insert(id);
        let flags = &self.state;
        let text = self.design.item_showcase_text(id, first, |f| flags.flag(f));
        let busy = neopets_sim::hero::busy_state(self.loco.state) || self.dialogue_open;
        let Some(text) = text.filter(|_| !busy) else {
            // Mode 1 (grow) at (0.5, 0.6, 5.0) with scale 0.
            self.corner = Some(Corner { id, pack, mode: 1, x: 0.5, y: 0.6, z: 5.0, s: 0.0, timer: 0.0, angle: 0.0 });
            return;
        };
        self.corner = None;
        // SPEC-0033 Behavior 42: the pose code and stinger by hero and item id.
        let tor = self.hero_index == 0;
        let (pose, stinger, floating) = match id {
            98..=105 if tor => (0xA4, Some(8), false),
            111..=118 if !tor => (0xA4, Some(8), false),
            106..=110 if tor => (0xA6, None, false),
            119..=125 if tor => (0xA5, Some(6), false),
            127..=134 if !tor => (0xA5, Some(6), false),
            _ => (0xA3, Some(7), true),
        };
        // Swords, wands and shields: both drawn flags set (they stay set after the showcase).
        if pose == 0xA4 || pose == 0xA6 {
            self.set_weapon_drawn(true);
        }
        if let Some(k) = stinger {
            self.stinger(k);
        }
        self.loco.play_one_shot(pose, true);
        self.hero.request(pose, true);
        self.showcase = Some(Showcase { id, pack, pose, floating, text, ticks: 0, opened: None, angle: 0.0 });
    }

    /// The full-showcase task, each tick: the pose re-requested (code 0xA3 with text holds 1.0 s
    /// before the text box opens; the other codes open it at once), the text box with the item
    /// text titled with the hero's name, and the end once 2.0 s have passed since the box opened
    /// and it is closed: the hero back to state 1.
    fn showcase_step(&mut self) {
        let Some(mut s) = self.showcase.take() else { return };
        s.ticks += 1;
        // The angle advances 2 pi / 5 rad per second and wraps.
        s.angle = (s.angle + 1.2566372 / 60.0) % std::f32::consts::TAU;
        let hold = if s.pose == 0xA3 { 60 } else { 0 };
        if s.opened.is_none() && s.ticks > hold {
            let name = self.hero_name();
            self.open_text(name, &s.text);
            s.opened = Some(s.ticks);
        }
        let done = s.opened.is_some_and(|t| s.ticks >= t + 120) && !self.dialogue_open;
        if done {
            if self.loco.state == neopets_sim::hero::ONE_SHOT {
                self.loco.state = hero_states::STANCE;
            }
            return;
        }
        if self.loco.state != neopets_sim::hero::ONE_SHOT {
            self.loco.play_one_shot(s.pose, true);
        }
        self.hero.request(s.pose, false);
        self.showcase = Some(s);
    }

    /// The floating showcase item (mode 4): its id, world and transform (2.8 up and 1.1 ahead of
    /// the hero, turned by the showcase angle), unless the pose hides it.
    /// SPEC-0021 Item showcase, modes 1 to 3, each frame (dt = 1/60 here): the angle advances
    /// by 2 pi / 5 rad/s and wraps; grow raises s by 1.5 dt (coins and Tal, 135 to 144, stop at
    /// 0.4 and shrink; others move by (+0.5, -0.5) dt and stop at 0.8 to hold); hold lasts 1.3 s;
    /// shrink lowers s by 2.5 dt and moves x by 0.5 dt, and ends below 0.
    fn corner_step(&mut self) {
        let Some(c) = self.corner.as_mut() else { return };
        let dt = 1.0 / 60.0;
        c.angle += 1.256_637_2 * dt;
        if c.angle > std::f32::consts::TAU {
            c.angle -= std::f32::consts::TAU;
        }
        match c.mode {
            1 => {
                c.s += 1.5 * dt;
                if (135..=144).contains(&c.id) {
                    if c.s > 0.4 {
                        c.s = 0.4;
                        c.mode = 3;
                    }
                } else {
                    c.x += 0.5 * dt;
                    c.y -= 0.5 * dt;
                    if c.s > 0.8 {
                        c.s = 0.8;
                        c.mode = 2;
                        c.timer = 0.0;
                    }
                }
            }
            2 => {
                c.timer += dt;
                if c.timer > 1.3 {
                    c.mode = 3;
                }
            }
            _ => {
                c.s -= 2.5 * dt;
                c.x += 0.5 * dt;
                if c.s < 0.0 {
                    self.corner = None;
                }
            }
        }
    }

    pub fn showcase_item(&self) -> Option<(u16, u32, [f32; 3], f32)> {
        let s = self.showcase.as_ref().filter(|s| s.floating)?;
        let (p, f) = (self.loco.position(), self.loco.facing);
        Some((s.id, s.pack, [p[0] + 1.1 * f[0], p[1] + 2.8, p[2] + 1.1 * f[2]], s.angle))
    }

    /// SPEC-0034 3.8, held-weapon trails: on every tick with the hero's attack-active bit (event
    /// bit 0x1) its trail is kept going with the held-weapon record, from attach point 0 (base
    /// B, direction D its third axis) with L = 0.6 W; an enemy's on every tick with any of event
    /// bits 0x1 to 0x10, with L = 1.5. Then every trail ages once. Not covered: the clip-list
    /// trails of +0x48 = 0 entries (element-coloured) and the Black Knight's own trail.
    fn trail_step(&mut self) {
        use neopets_sim::trail;
        if self.hero.frame_bits() & 1 != 0 {
            let place = placement(self.loco.position(), self.loco.facing);
            let frames = self.hero.bone_frames();
            if let Some(&(Some(bone), angles, offset)) = self.hero.bones().attach.first() {
                if let Some(bw) = frames.get(bone) {
                    let w = place.mul(bw).mul(&attach_rotation(offset, angles));
                    let (p1, p2) = trail::held_points(w.t, unit(w.cols[2]), 0.6 * self.weapon_reach());
                    self.hero_trail = self.trails.hold(self.hero_trail, trail::HELD, p1, p2);
                }
            }
        }
        let pack = self.world.pack;
        for c in self.characters.iter().filter(|c| c.pack == pack && !c.inactive && c.enemy.is_some() && c.anim.frame_bits() & 0x1F != 0) {
            let Some(&(Some(bone), angles, offset)) = c.anim.bones().attach.first() else { continue };
            let frames = c.anim.bone_frames();
            let Some(f) = frames.get(bone) else { continue };
            let w = placement(c.position, c.facing).mul(f).mul(&attach_rotation(offset, angles));
            let (p1, p2) = trail::held_points(w.t, unit(w.cols[2]), 1.5);
            match self.trails.hold(self.enemy_trails.get(&c.id).copied(), trail::HELD, p1, p2) {
                Some(h) => self.enemy_trails.insert(c.id, h),
                None => self.enemy_trails.remove(&c.id),
            };
        }
        self.trails.tick();
        // NEOPETS_TRACE_TRAILS: the live trails and their live samples (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_TRAILS").is_some() && self.trails.iter().next().is_some() {
            let t: Vec<String> = self.trails.iter().map(|t| format!("{}/{}", t.segments().count(), t.samples.len())).collect();
            println!("      trails on tick {}: {}", self.ticks, t.join(" "));
        }
    }

    fn weapon_reach(&self) -> f32 {
        self.weapon
            .and_then(|id| match id {
                98..=108 => self.reach_tor.get((id - 98) as usize).copied(),
                111..=118 => self.reach_roberta.get((id - 111) as usize).copied(),
                _ => None,
            })
            .unwrap_or(1.5)
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
        let mut knight_block = false;
        let hero_at = self.loco.position();
        // Factor E: the live petpet of kind 0 or 7 (SPEC-0047 Passive bonuses).
        let e_factor = self.pet_bonus.damage_factor(self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t));
        let mut hits = Vec::new();
        for (i, c) in self.characters.iter_mut().enumerate().filter(|(_, c)| c.pack == pack && !c.inactive && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|e| e.is_dead()) && !c.unhittable()) {
            if self.hit_list.contains(&c.id) {
                continue;
            }
            let d = sub_v(c.position, p);
            if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > 65.0 {
                continue;
            }
            let cplace = placement(c.position, c.facing);
            let cframes = c.anim.bone_frames();
            // The Cloud Beast stand-in has no rig: its body is a cylinder of the row's radius and
            // height around the floating cloud (stand-in hurt shape).
            let cloud_body = c.stand_in_body();
            let hit = cloud_body.is_some_and(|b| neopets_sim::combat::overlap(&weapon, &b)) || c.anim.bones().hurt.iter().zip(&cframes).any(|(&(base, length, radius), f)| {
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
            // SPEC-0036 Hit detection step 4: a guarding target never takes damage from a swing
            // that is not a power hit (Tor's power step, sub-state 10, with a Mote in word 2).
            // With dot(target facing, hero facing) below its Blocking Angle the swing is blocked
            // (the block plays its hit reaction, SPEC-0038 Behavior 44, outer 7) and counts as
            // landed; otherwise the swing has no effect at all and the target stays out of the
            // hit list, so a later window tick can still be blocked.
            let hf = self.loco.facing;
            let power_hit = sub == 10 && mote_id.is_some();
            // SPEC-0012 G15: the Black Knight is unhittable and guarding on every tick its stun
            // gate does not pass: the swing is blocked whatever the facing (Blocking Angle 1.0):
            // no damage, impact cue 34, one block per swing, the magic paid as for a landed hit,
            // and Tor's sword recoils (sub-state 13); a power step with a Mote passes through
            // with no effect. With the gate passing the swing passes through; the damage comes
            // only from the knight's weak-point routine.
            // SPEC-0029 G11: the Undead Draik the same way while it guards (outside its stun and
            // hit reactions, and before stage 5, Blocking Angle 2.0); exposed, the swing passes
            // through to its own hit check.
            // SPEC-0019 G14: the Werelupe King cannot be hit in its flinch, catch, roll and
            // jump; while it blocks, a swing from its front (dot of the facings below 0) is
            // blocked (counted as landed, the blocked word set, Tor's recoil) and one from beside
            // or behind does nothing; a power step with a Mote passes through the guard.
            if let Some(k) = c.king.as_mut() {
                if !k.hittable() {
                    continue;
                }
                if k.guarding() && !power_hit {
                    if c.facing[0] * hf[0] + c.facing[1] * hf[1] + c.facing[2] * hf[2] < 0.0 {
                        self.hit_list.push(c.id);
                        landed += 1;
                        knight_block = true;
                        k.blocked = true;
                    }
                    continue;
                }
            }
            let guards = match (c.knight.as_ref(), c.draik.as_ref()) {
                (Some(k), _) => Some(!k.exposed),
                (_, Some(d)) => Some(d.guarding()),
                _ => None,
            };
            if let Some(guarding) = guards {
                if guarding && !power_hit {
                    self.hit_list.push(c.id);
                    landed += 1;
                    knight_block = true;
                }
                if guarding || c.knight.is_some() {
                    continue;
                }
            }
            if let Some(e) = c.enemy.as_mut().filter(|e| e.blocking && !power_hit) {
                if c.facing[0] * hf[0] + c.facing[1] * hf[1] + c.facing[2] * hf[2] < e.p.defence.angle {
                    self.hit_list.push(c.id);
                    e.block_hit = true;
                    landed += 1;
                }
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
            // The bosses with their own hit checks are skipped by the swing resolver that pays the
            // magic and Mote charge and sends the power step's status (SPEC-0011 and SPEC-0020
            // Hero actions); their routines take the hit. A Faerie Sister's takes the power step
            // at power 1.0 (the attack power table is not used); Gnarfas's uses the table.
            let own = c.own_hit_check();
            let power = if c.sister.is_some() || c.serpent.is_some() { 1.0 } else { power };
            let (damage, absorbed) = neopets_sim::mote::melee(power, stat_a as f32, mote, t, d_factor, e_factor, c.row.map_or(0.0, |r| r.defence), g);
            if (damage > 0 || absorbed) && !own {
                landed += 1;
            }
            // A boss takes its hits through its own controller (SPEC-0015 counted hits).
            // SPEC-0036 Unarmed melee: with no weapon-family item (A = 0) a swing calls no
            // take-hit.
            if stat_a == 0 {
                continue;
            }
            if c.boss.is_some() {
                self.boss_hits.push((c.id, damage));
                continue;
            }
            hits.push((i, damage, own));
        }
        // The knight's block (SPEC-0012 G15): the impact cue and, for Tor, the sword bounce.
        if knight_block {
            self.sound.play(neopets_sim::sound::WEAPON, 34, neopets_sim::sound::Play { emitter: Some(hero_at), ..Default::default() });
            if self.hero_index == 0 {
                if let Some(a) = self.attack.as_mut() {
                    a.recoil();
                }
            }
        }
        // Step 6: the breakable walk on every tick of the window.
        let tip = add(p0, scale(axis, l));
        let prev = d2.map_or(tip, |d| add(p0, scale(d, l)));
        self.breakable_walk(p0, tip, prev);
        // SPEC-0036 Damage, take-hit: in the power step a status request of the word-2 Mote
        // also goes to the target (SPEC-0029 Behavior 27.1).
        let request = mote_id.filter(|_| sub == 10).and_then(|id| neopets_sim::mote::status_request(0, id, |k| self.design.attack_power(k)));
        for &(i, _, _) in &hits {
            if let Some(e) = self.characters.get(i).and_then(|c| c.entity) {
                self.state.hero.hits.push(e);
            }
        }
        for (i, damage, own) in hits {
            self.character_take_hit(i, damage, hero_at, "");
            if let Some(r) = request.filter(|_| !own) {
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
        // SPEC-0046 step 6: a hit through the enemy class's hit method starts the 14-frame flash.
        if let Some(c) = self.characters.get_mut(i) {
            c.flash = HIT_FLASH;
        }
        // The script's undamageable entities (SPEC-0044 1056 Update) take no damage.
        let pack = self.world.pack;
        // SPEC-0019: the Werelupe King takes the damage at once and flinches on its next tick; its
        // own controller handles its death.
        if let Some(c) = self.characters.get_mut(i).filter(|c| c.king.is_some()) {
            // Status ticks (poison, fire) hurt it in any state (G14).
            let Some(k) = c.king.as_mut().filter(|k| k.hittable() || by.starts_with("status")) else { return };
            let damage = k.hit_damage(damage);
            c.hp -= damage;
            c.pending_hit = Some(damage);
            return;
        }
        // SPEC-0014 Incoming hits: the Ixi Chief's controller applies the hit (the hold, lava and
        // death take none; a melee hit and the summed shots reach it separately).
        if let Some(c) = self.characters.get_mut(i).filter(|c| c.ixi.is_some()) {
            if by.contains("shot") {
                // Several shots in one tick: the last one's damage stands (Edge cases 2).
                c.pending_shot = Some(damage.max(0));
            } else {
                c.pending_hit = Some(damage.max(0));
            }
            return;
        }
        // SPEC-0010: the Cloud Beast keeps the shared NPC take-hit; its combat word is never agro,
        // so the damage is doubled before the subtraction. The monster bar is armed as for the
        // common enemies (a guess: the spec names no bar for the test spawn).
        if self.characters.get(i).is_some_and(|c| c.cloud.is_some() && c.dying.is_none()) {
            let width = self.name_font.as_ref().map_or(10, |(f, sp)| f.measure(self.characters[i].name.as_bytes(), *sp));
            let c = &mut self.characters[i];
            let max = c.row.map_or(1, |r| r.hit_points as i32);
            self.monster_bar.arm(c.id, &c.name, width, c.hp, max, &self.monster_words);
            c.hp -= 2 * damage.max(0);
            self.last_hit = format!("{by}hit {} for {} (hp {})", c.name, 2 * damage.max(0), c.hp.max(0));
            return;
        }
        // SPEC-0016: the Plague Serpent takes a hit only as its stun strike (its own controller).
        if let Some(c) = self.characters.get_mut(i).filter(|c| c.serpent.is_some()) {
            c.serpent_hit = Some((damage, by.contains("shot")));
            return;
        }
        // SPEC-0013: the Gelert Assassin's controller takes a hit only in its hittable states.
        if let Some(c) = self.characters.get_mut(i).filter(|c| c.gelert.is_some()) {
            // G7: a melee contact takes the tick; otherwise the last overlapping shot's damage
            // stands (each slot is still removed).
            let shot = by.contains("shot");
            if !(shot && c.gelert_hit.is_some_and(|(_, s)| !s)) {
                c.gelert_hit = Some((damage.max(0), shot));
            }
            return;
        }
        // SPEC-0011: a Faerie Sister's own controller applies the hit (the shield, the flinch and
        // death); a hero shot is told apart from the swing and the Mote objects.
        if let Some(c) = self.characters.get_mut(i).filter(|c| c.sister.is_some()) {
            c.sister_hit = Some((damage, by.contains("shot")));
            return;
        }
        // SPEC-0029 Behavior 6.4: the Undead Draik registers hits on its own bone groups; the
        // Darkest Faerie is invulnerable for her whole life (SPEC-0054 Hit points).
        // Queen Fyora's take-hit is not given; she takes no hit here.
        if self.characters.get(i).is_some_and(|c| c.draik.is_some() || c.faerie.is_some() || c.fyora.is_some()) {
            return;
        }
        // SPEC-0020: Gnarfas takes a hit through its own flinch (the damage lands when it ends).
        if let Some(c) = self.characters.get_mut(i).filter(|c| c.gnarfas.is_some()) {
            c.pending_hit = Some(damage);
            return;
        }
        let entity = self.characters.get(i).and_then(|c| c.entity);
        if let Some(e) = entity.filter(|_| self.characters[i].pack == pack) {
            self.state.hit_now.insert(e);
        }
        let damage = if entity.is_some_and(|e| level::undamageable(pack).contains(&e) || self.state.undamageable_now.contains(&e)) { 0 } else { damage };
        let Some(c) = self.characters.get_mut(i) else { return };
        {
            if let Some(e) = c.enemy.as_mut() {
                // SPEC-0023 10.1: the take-hit arms the monster bar before the damage.
                if !e.is_dead() {
                    let width = self.name_font.as_ref().map_or(10, |(f, sp)| f.measure(c.name.as_bytes(), *sp));
                    let max = c.row.map_or(1, |r| r.hit_points as i32);
                    self.monster_bar.arm(c.id, &c.name, width, c.hp, max, &self.monster_words);
                }
                let (applied, toward) = e.take_hit(damage, hero_at, &mut self.rng);
                if let Some(w) = c.walker.as_mut() {
                    w.steer_to(toward, 1.0);
                }
                c.hp -= applied;
                if c.hp <= 0 && !e.is_dead() {
                    c.hp = 0;
                    e.die(&mut self.rng);
                    self.payouts.push((c.pack, c.position, c.drop, std::mem::take(&mut c.carried_np)));
                }
            } else if damage > 0 && c.townsfolk.is_none() {
            // SPEC-0056: the take-hit of a townsfolk character returns at once.
                c.hp -= damage;
                if c.hp <= 0 && c.dying.is_none() {
                    c.dying = Some(c.row.map_or(60, |r| r.disappear.max(1) as u32));
                    // A class-17 flyer (its AI is not built) still pays its drop placement on
                    // death (SPEC-0021 enemy drops).
                    if c.row.is_some_and(|r| r.class == Some(17)) {
                        self.payouts.push((c.pack, c.position, c.drop, std::mem::take(&mut c.carried_np)));
                    }
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
        // Wind's centre: the ground probed from 8 above the hero.
        let ground = self.world.ground_at(hero[0], hero[2], hero[1] + 8.0);
        objects.retain_mut(|o| {
            let live = o.tick(hero, ground);
            if o.id == 161 {
                self.orbit_angle = o.angle;
            }
            live
        });
        for o in objects.iter_mut() {
            let p = neopets_sim::mote::power_index(0, o.id).and_then(|k| self.design.attack_power(k)).unwrap_or(1.0);
            let g = self.items.get(o.id).map_or(0.0, |i| i.stats[6] as f32);
            let mut hits = Vec::new();
            for (i, c) in self.characters.iter().enumerate().filter(|(_, c)| c.pack == pack && !c.inactive && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|e| e.is_dead()) && !c.unhittable() && !c.own_hit_check()) {
                let Some(r) = c.row else { continue };
                let base = c.stand_in_body().map_or(c.position, |b| [b.center[0], b.center[1] - 0.5 * b.length, b.center[2]]);
                if !o.touches(base, r.radius, r.height) || !o.list(c.id) {
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
        if cost > 0 && !self.cheats.magic {
            self.hero_mp = (self.hero_mp - cost).max(0);
        }
    }

    /// SPEC-0036 Mote charge use: a use of n below the remaining charge subtracts n; otherwise
    /// the Mote leaves the inventory and every equipment word holding it is cleared.
    fn spend_charge(&mut self, word: usize, id: u16, n: i32) {
        if n <= 0 || self.cheats.charge {
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
            let spawned = neopets_sim::charm::spawn_tor(id, &self.charm_words, at, hero, self.orbit_angle, scale);
            if let Some(o) = spawned.first() {
                self.last_mote = format!("Mote {id} power-attack object x{} (scale {scale}, {} ticks) on tick {}", spawned.len(), o.life, self.ticks);
            }
            self.mote_objects.extend(spawned);
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
        let enemies = self.characters.iter().any(|c| c.pack == self.world.pack && !c.inactive && c.enemy.as_ref().is_some_and(|e| !e.is_dead()) && c.dying.is_none());
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
            .filter(|c| c.pack == self.world.pack && c.dying.is_none() && !c.inactive && !c.enemy.as_ref().is_some_and(|e| e.is_dead()))
            // A stunned boss is fought first; otherwise the nearest class-17 enemy.
            .filter(|c| self.fight_target.as_ref().is_none_or(|n| *n == c.name))
            .filter(|c| c.row.is_some_and(|r| r.class == Some(17)) || c.gnarfas.is_some() || c.king.is_some() || c.knight.as_ref().is_some_and(|k| !k.defeated()) || c.gelert.as_ref().is_some_and(|g| g.state != neopets_sim::gelert::State::Death && g.state != neopets_sim::gelert::State::CloneWait) || c.sister.as_ref().is_some_and(|s| !s.dead()) || c.draik.as_ref().is_some_and(|k| k.stage < 6) || c.faerie.is_some() || c.serpent.is_some() || c.ixi.as_ref().is_some_and(|k| k.state != neopets_sim::ixi::State::Death) || c.boss.as_ref().is_some_and(|b| matches!(b.top, neopets_sim::juppie::Top::Slam(2 | 3))))
            .min_by(|a, b| {
                let d = |c: &Character| if c.boss.is_some() { 0.0 } else { (c.position[0] - p[0]).hypot(c.position[2] - p[2]) };
                d(a).total_cmp(&d(b))
            })?;
        let d = (c.position[0] - p[0]).hypot(c.position[2] - p[2]);
        // Roberta fights from range: close within 16 (her shots fly along her facing).
        Some(([c.position[0], c.position[2]], d < if self.hero_index == 1 { 16.0 } else { 4.5 }))
    }

    /// Diagnostic herding goal: 3.5 behind the nearest Whinny that is not penned, on the far
    /// side from the pen point.
    pub fn herd_goal(&self) -> Option<[f32; 2]> {
        let p = self.loco.position();
        let w = self
            .characters
            .iter()
            .filter(|c| c.pack == self.world.pack && c.whinny.as_ref().is_some_and(|w| !w.penned()))
            .min_by(|a, b| {
                let d = |c: &Character| (c.position[0] - p[0]).hypot(c.position[2] - p[2]);
                d(a).total_cmp(&d(b))
            })?;
        // The free pen nearest that Whinny (the first pen when all are taken).
        let hd = |q: [f32; 3]| (w.position[0] - q[0]).hypot(w.position[2] - q[2]);
        let pen = self.world.pens.iter().filter(|pn| !pn.occupied).min_by(|a, b| hd(a.point).total_cmp(&hd(b.point))).or(self.world.pens.first())?.point;
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
        if slot != REBIND_SLOT && slot != PET_SLOT && slot != EMBEDDED_SLOT {
            self.slot_arrived.insert((pack, slot));
        }
        if slot != REBIND_SLOT {
            self.slot_items.insert((pack, slot), items.clone());
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
        // A record whose entity the embedded region has bound since the request is not bound twice.
        let bound = |o: usize| self.entity_of(pack, pack, o).is_some_and(|e| self.characters.iter().any(|c| c.pack == pack && c.entity == Some(e)));
        let loaded: Vec<_> = if slot == REBIND_SLOT { loaded.into_iter().filter(|(pl, ..)| wanted.contains(&pl.ordinal) && !bound(pl.ordinal)).collect() } else { loaded };
        let rebind = slot == REBIND_SLOT;
        let (slot, region) = if slot == REBIND_SLOT {
            self.slot_regions.remove(&(pack, REBIND_SLOT));
            (EMBEDDED_SLOT, pack)
        } else {
            (slot, region)
        };
        // The block's summon-reserved records are kept for spawn by key, not bound.
        if !rebind {
            self.reserves.retain(|r| !(r.pack == pack && r.slot == slot));
        }
        let live_type = self.petpet.as_ref().map(|p| p.type_id);
        for (pl, model, anim, row, tuning) in loaded {
            // SPEC-0047 Acquisition: while a petpet of type t is live, no record of type t binds.
            if live_type == Some(pl.key as i32) {
                continue;
            }
            if pl.summon {
                if !rebind {
                    self.reserves.push(Reserve { pack, slot, region, pl, model, anim, row, tuning, bound: None });
                }
                continue;
            }
            // The pack's AI-player activate callback refuses some embedded records.
            if slot == EMBEDDED_SLOT && level::admits(pack, pl.ordinal, &self.state) == Some(false) {
                continue;
            }
            let drop = usize::try_from(pl.drop).ok().and_then(|i| items.get(i).copied());
            self.bind_character(pack, slot, region, pl, (model, anim, row, tuning), drop);
        }
    }

    /// Bind a record of a block as a character: on the ground under its point (the ground step
    /// probes 2.5 above; SPEC-0018; the fall itself is not simulated), with the controllers of
    /// its class. Returns its id.
    #[allow(clippy::type_complexity)]
    fn bind_character(
        &mut self,
        pack: u32,
        slot: u8,
        region: u32,
        pl: neopets_data::region::Placement,
        (model, anim, row, tuning): (ModelData, HeroAnim, Option<neopets_data::monster::Row>, Option<neopets_data::monster::Tuning>),
        drop: Option<neopets_data::region::ItemPoint>,
    ) -> u64 {
        {
            self.next_character += 1;
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
                walker: row.filter(|r| matches!(r.class, Some(1 | 7 | 8 | 11 | 12 | 14 | 20 | 21 | 32 | 33 | 39)) || (r.class == Some(17) && tuning.is_some())).map(|r| {
                    neopets_sim::walker::Walker::new(position, pl.facing, r.radius, r.weight.max(0.1), self.next_character as u32)
                }),
                whinny: row.filter(|r| r.class == Some(7)).map(|_| neopets_sim::whinny::Whinny::default()),
                enemy: match (row, tuning) {
                    (Some(r), Some(t)) if r.class == Some(17) => Some(Enemy::new(
                        params(&r, &t),
                        enemy::Spawn { default_state: pl.ai_code as u16, home: position, emerge_distance: pl.emerge_distance, selector: pl.selector, alarm: (pl.alarm != 0xFF).then_some(pl.alarm) },
                        &mut self.rng,
                    )),
                    _ => None,
                },
                weapons: tuning.map_or([(0.0, 0.0); 5], |t| t.weapons),
                drop,
                carried_np: 0,
                carried_item: None,
                colour: pl.colour,
                slots: slot_colours(&pl.colours),
                flash: 0,
                root_step: [0.0; 3],
                script_walk: None,
                move_type: 0,
                held: false,
                inactive: self.entity_of(pack, region, pl.ordinal).is_some_and(|e| self.pending_off.contains(&(pack, e))),
                deactivated: self.entity_of(pack, region, pl.ordinal).is_some_and(|e| self.pending_off.contains(&(pack, e))),
                // SPEC-0015 Variant: the spit flag from the Act 3 marker (story flag 0x53).
                boss: row.filter(|r| r.class == Some(18)).map(|_| {
                    let mut j = neopets_sim::juppie::Juppie::default();
                    j.spit = self.state.flag(0x53);
                    j
                }),
                miamouse: row.filter(|r| r.class == Some(21)).map(|_| neopets_sim::miamouse::Miamouse::new(position)),
                gnarfas: row.filter(|r| r.class == Some(14)).map(|_| neopets_sim::gnarfas::Gnarfas::new(&self.gnarfas_words, &mut self.rng)),
                pending_hit: None,
                king: row.filter(|r| r.class == Some(32)).map(|_| neopets_sim::werelupe::King::new(position)),
                ixi: row.filter(|r| r.class == Some(11)).map(|_| neopets_sim::ixi::Ixi::new(neopets_sim::ixi::fight_mode(self.state.flag(358), self.state.flag(60)), &self.ixi_words)),
                pending_shot: None,
                sister: row.and_then(|r| r.class).filter(|c| matches!(c, 13 | 15 | 16)).map(|c| neopets_sim::sisters::Sister::new(c as u8)),
                sister_hit: None,
                draik: row.filter(|r| r.class == Some(12)).map(|_| neopets_sim::draik::Draik::new(&self.draik_words, &mut self.rng)),
                faerie: row.filter(|r| r.class == Some(37)).map(|_| neopets_sim::darkest::Faerie::default()),
                spawn_keys: tuning.map_or([0; 4], |t| std::array::from_fn(|k| t.slots[k].spawn_key)),
                fyora: row.filter(|r| r.class == Some(38)).map(|_| neopets_sim::fyora::Fyora::default()),
                serpent: row.filter(|r| r.class == Some(2)).map(|_| neopets_sim::serpent::Serpent::default()),
                serpent_hit: None,
                mortog: row.filter(|r| r.class == Some(39)).map(|_| neopets_sim::mortog::Mortog::default()),
                wild: row.filter(|r| r.class == Some(33)).map(|_| neopets_sim::petpet::Wild::default()),
                pet_type: pl.key as i32,
                class20: row.filter(|r| r.class == Some(20)).map(|_| neopets_sim::petpet::Class20::new(position)),
                knight: row.filter(|r| r.class == Some(8)).and_then(|_| self.knight_words.as_ref()).map(neopets_sim::knight::Knight::new),
                gelert: row.filter(|r| r.class == Some(1)).and_then(|_| self.gelert_words.as_ref()).map(|w| neopets_sim::gelert::Gelert::new(w, self.state.flag(258), pl.ordinal > 0, &mut self.rng)),
                gelert_slot: pl.ordinal,
                gelert_hit: None,
                invulnerable: false,
                hp: row.map_or(1, |r| r.hit_points as i32),
                dying: None,
                sound_action: None,
                emotes: Default::default(),
                sound_blocks: anim.sound_blocks(),
                row,
                cloud: None,
                anim,
                model,
            });
        }
        self.next_character
    }

    /// A spawn-by-key request (SPEC-0029 Behavior 31): the block of a character's request, or
    /// for a world-level request (no block) the embedded block and then each slot in order; the
    /// first reserved record of the key not bound to a live character is bound at the point and
    /// facing, with the drop item as its direct drop, and starts in agro (state 19) or, for the
    /// minion keys and the arrival flag, with animation 82 in state 21. Returns the summoned
    /// character. The block's free-object and cap tests and the summoner id are not built.
    fn summon(&mut self, pack: u32, block: Option<u8>, key: i16, at: [f32; 3], facing: [f32; 3], drop: Option<u16>, arrive: bool) -> Option<u64> {
        let live: std::collections::HashSet<u64> = self.characters.iter().filter(|c| c.dying.is_none() && !c.enemy.as_ref().is_some_and(|e| e.is_dead())).map(|c| c.id).collect();
        for r in &mut self.reserves {
            if r.bound.is_some_and(|id| !live.contains(&id)) {
                r.bound = None;
            }
        }
        let blocks: Vec<u8> = match block {
            Some(s) => vec![s],
            None => {
                let mut v: Vec<u8> = self.reserves.iter().filter(|r| r.pack == pack).map(|r| r.slot).collect();
                v.sort_by_key(|&s| (s != EMBEDDED_SLOT, s));
                v.dedup();
                v
            }
        };
        for s in blocks {
            let recs: Vec<usize> = (0..self.reserves.len()).filter(|&i| self.reserves[i].pack == pack && self.reserves[i].slot == s).collect();
            let keyed: Vec<neopets_sim::summon::Reserved> = recs.iter().map(|&i| neopets_sim::summon::Reserved { key: self.reserves[i].pl.key, bound: self.reserves[i].bound.is_some() }).collect();
            let Some(k) = neopets_sim::summon::find(&keyed, key) else { continue };
            let i = recs[k];
            let r = &self.reserves[i];
            let mut pl = r.pl.clone();
            pl.position = at;
            pl.facing = facing;
            let parts = (r.model.clone(), r.anim.clone(), r.row, r.tuning);
            let region = r.region;
            let drop = drop.map(|id| neopets_data::region::ItemPoint { position: at, facing, flags: 0, group: -1, key: id as i16, luck: 0, count: 0, map: 0, snap: false });
            let id = self.bind_character(pack, s, region, pl, parts, drop);
            self.reserves[i].bound = Some(id);
            let hero = self.loco.position();
            let rng = &mut self.rng;
            if let Some(c) = self.characters.iter_mut().find(|c| c.id == id) {
                c.inactive = false;
                c.deactivated = false;
                if let Some(a) = c.enemy.as_mut().and_then(|e| e.summoned(arrive || neopets_sim::summon::minion(key), hero, rng)) {
                    c.anim.play(a);
                }
            }
            self.last_summon = format!("summoned key {key} at {:.1},{:.1},{:.1} on tick {}", at[0], at[1], at[2], self.ticks);
            return Some(id);
        }
        self.last_summon = format!("no reserved record for key {key} on tick {}", self.ticks);
        None
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
            carried_np: 0,
            carried_item: None,
            colour: pl.colour,
            slots: slot_colours(&pl.colours),
            flash: 0,
            root_step: [0.0; 3],
            script_walk: None,
            move_type: 0,
            held: false,
            inactive: false,
            deactivated: false,
            boss: None,
            miamouse: None,
            gnarfas: None,
            pending_hit: None,
            king: None,
            ixi: None,
            pending_shot: None,
            sister: None,
            sister_hit: None,
            draik: None,
            faerie: None,
            spawn_keys: [0; 4],
            fyora: None,
            serpent: None,
            serpent_hit: None,
            mortog: None,
            wild: None,
            pet_type: pl.key as i32,
            class20: None,
            knight: None,
            gelert: None,
            gelert_slot: 0,
            gelert_hit: None,
            invulnerable: false,
            hp: row.map_or(1, |r| r.hit_points as i32),
            dying: None,
            sound_action: None,
            emotes: Default::default(),
            sound_blocks: anim.sound_blocks(),
            row,
            cloud: None,
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

    /// Diagnostic: the bosses' HP (trace only).
    pub fn diag_gelert_hp(&mut self, hp: i32) {
        let pack = self.world.pack;
        for c in self.characters.iter_mut().filter(|c| c.pack == pack && !c.inactive) {
            let boss = c.gelert.as_ref().is_some_and(|g| !g.clone) || c.ixi.is_some() || c.king.is_some() || c.serpent.is_some() || c.gnarfas.is_some() || c.draik.is_some() || c.boss.is_some() || c.sister.is_some();
            if boss {
                c.hp = hp;
            }
        }
    }

    /// Diagnostic: load a region into a slot of the current world (trace only).
    pub fn diag_load_region(&mut self, slot: u8, region: u32) {
        let pack = self.world.pack;
        self.slot_regions.insert((pack, slot), region);
        self.region_wanted.push((pack, slot, region));
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
        let food = self.characters.iter().find(|c| c.id == id).and_then(|c| self.nearest_food(c.position, false));
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
            clip_action: clip.action,
            clip_done: clip.progress >= 0.99,
            food,
        };
        let act = live.tick(&v, self.pet_low, &mut self.rng);
        if let Some(t) = act.reserve {
            self.pet_reserve.reserve(t);
        }
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
        if let Some((token, food)) = act.eat {
            self.remove_food(token);
            self.pet_reserve.clear();
            if let Some(p) = self.petpet.as_mut() {
                p.eat(self.pet_food.value(p.kind, food));
                self.last_pet = format!("petpet ate item {food} on tick {}: T {} F {}", self.ticks, p.t, p.f);
            }
        }
        if act.leave {
            self.last_pet = format!("petpet left on tick {}", self.ticks);
            self.deactivate_pet();
        }
    }

    /// The nearest world item of category 12 in the current world (position, id, token); with
    /// `anchor`, the nearest Bagguss (145) only.
    fn nearest_food(&self, at: [f32; 3], anchor: bool) -> Option<([f32; 3], u16, u32)> {
        let pool = self.pools.get(&self.world.pack)?;
        let items = &self.items;
        pool.slots
            .iter()
            .flatten()
            .filter(|it| items.get(it.id).is_some_and(|i| i.kind == 12) && (it.id == neopets_sim::petpet::BAGGUSS) == anchor)
            .map(|it| (it.position, it.id, it.token, (it.position[0] - at[0]).hypot(it.position[2] - at[2])))
            .min_by(|a, b| a.3.total_cmp(&b.3))
            .map(|(p, id, t, _)| (p, id, t))
    }

    /// Remove the world item with this token from the current world; returns its id.
    fn remove_food(&mut self, token: u32) -> Option<u16> {
        let pool = self.pools.get_mut(&self.world.pack)?;
        let i = pool.slots.iter().position(|s| s.as_ref().is_some_and(|it| it.token == token))?;
        pool.take(i).map(|it| it.id)
    }

    /// SPEC-0047 Strike step 5: the reservation's fallback, once per tick: past 5 s the food is
    /// removed and the live petpet (T at least 1) gets its value.
    fn food_fallback(&mut self) {
        let Some(token) = self.pet_reserve.tick() else { return };
        let Some(id) = self.remove_food(token) else { return };
        if let Some(p) = self.petpet.as_mut().filter(|p| p.t >= 1) {
            p.eat(self.pet_food.value(p.kind, id));
            self.last_pet = format!("petpet fed item {id} by the fallback on tick {}: T {}", self.ticks, p.t);
        }
    }

    /// SPEC-0022 idle list "Feed petpet" (label 14) and SPEC-0047 Feeding a petpet: the prompt
    /// while the hero's own petpet is within 2.5 units or a wild petpet within 4.5, in front of
    /// the hero (horizontal dot above 0.5); Square with a food item selected enters state 49,
    /// without one plays cue 0xA2. The food is the selected quick item (which item the feed
    /// takes is not given; R-0094). The 4.5 own-petpet range while a timer is below 45 s and
    /// the class-20 creatures (25 units) are not built.
    fn feed_step(&mut self) {
        if neopets_sim::hero::busy_state(self.loco.state) || !(1..=6).contains(&self.loco.state) {
            return;
        }
        let (hero, f) = (self.loco.position(), self.loco.facing);
        let pack = self.world.pack;
        let front = |p: [f32; 3], r: f32| {
            let d = [p[0] - hero[0], p[2] - hero[2]];
            let l = d[0].hypot(d[1]);
            l <= r && (l == 0.0 || (d[0] * f[0] + d[1] * f[2]) / l > 0.5)
        };
        let pet = self.pet_char;
        // The live petpet's range is 2.5, or 4.5 while its time T is below 45 s, with no food
        // reserved; wild petpets 4.5 (SPEC-0047 Prompt and meter).
        let own = if self.petpet.as_ref().is_some_and(|p| p.t < 2700) { 4.5 } else { 2.5 };
        let reserved = self.pet_reserve.food.is_some();
        let near = self.characters.iter().any(|c| c.pack == pack && !c.inactive && ((Some(c.id) == pet && !reserved && front(c.position, own)) || (c.wild.is_some() && front(c.position, 4.5)) || (c.class20.is_some() && front(c.position, 25.0))));
        if !near {
            return;
        }
        self.door_prompt = Some(14);
        if !self.square_now {
            return;
        }
        let items = &self.state.items;
        let food = self.quick.effective(&|id| u16::try_from(id).ok().and_then(|id| items.get(&id)).is_some_and(|&n| n > 0));
        // With no selected quick item the first category-12 item held (SPEC-0047 Feeding
        // selection; the general list's order is taken as item id order).
        let food = u16::try_from(food).ok().or_else(|| {
            let mut held: Vec<u16> = items.iter().filter(|&(&id, &n)| n > 0 && self.items.get(id).is_some_and(|i| i.kind == 12) && id != neopets_sim::petpet::BAGGUSS).map(|(&id, _)| id).collect();
            held.sort_unstable();
            held.first().copied()
        });
        match food.filter(|&id| neopets_sim::petpet::feedable(id) && self.items.get(id).is_some_and(|i| i.kind == 12)) {
            Some(id) => {
                self.loco.feed();
                self.feeding = Some(id);
            }
            None => self.cue(0xA2),
        }
    }

    /// SPEC-0047 Feeding, at progress 0.99 of clip 138: the fed flag goes to the nearest of the
    /// own petpet and the wild petpets within 10.5 (a wild one also sends the live petpet away,
    /// T = -1); one of the food is dropped 2.8 units in front of the hero at model scale 0.45,
    /// reserved and not collectable, and one is removed from the inventory.
    fn feed_finish(&mut self) {
        let Some(food) = self.feeding else { return };
        let clip = self.hero.clip_state();
        let done = clip.action == neopets_sim::hero::ANIM_FEED && clip.progress >= 0.99 || self.loco.state == hero_states::STANCE;
        if !done {
            if self.loco.state != neopets_sim::hero::FEED {
                self.feeding = None;
            }
            return;
        }
        self.feeding = None;
        let (hero, f) = (self.loco.position(), self.loco.facing);
        let pack = self.world.pack;
        let pet = self.pet_char;
        let best = self
            .characters
            .iter()
            .filter(|c| c.pack == pack && !c.inactive && (Some(c.id) == pet || c.wild.is_some()))
            .map(|c| (c.id, (c.position[0] - hero[0]).hypot(c.position[2] - hero[2])))
            .filter(|&(_, d)| d <= 10.5)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id);
        match best {
            Some(id) if Some(id) == pet => {
                if let Some(p) = self.petpet.as_mut() {
                    p.pending_event = true;
                }
            }
            Some(id) => {
                if let Some(w) = self.characters.iter_mut().find(|c| c.id == id).and_then(|c| c.wild.as_mut()) {
                    w.fed = true;
                }
                if let Some(p) = self.petpet.as_mut() {
                    p.t = -1;
                }
            }
            None => {}
        }
        let at = [hero[0] + 2.8 * f[0], hero[1], hero[2] + 2.8 * f[2]];
        let y = self.world.ground_at(at[0], at[2], at[1] + 8.0).unwrap_or(hero[1]);
        let spawn = neopets_sim::items::Spawn { id: food, point: [at[0], y, at[2]], placed: true, charm: false, s0: 0.45, s1: 0.45, delay: 0.0, larg: -1.0, direction: Some([0.0; 3]), flag: 0, map: 0, price: 0 };
        if self.spawn_item(pack, spawn) {
            if let Some(it) = self.pools.get_mut(&pack).and_then(|pl| pl.slots.iter_mut().flatten().max_by_key(|it| it.token)) {
                it.reserved = true;
                self.pet_reserve.reserve(it.token);
            }
        }
        let left = {
            let n = self.state.items.entry(food).or_insert(0);
            *n = n.saturating_sub(1);
            *n
        };
        if left == 0 {
            self.state.items.remove(&food);
        }
        self.quick.used(food as i32, left);
        self.last_pet = format!("fed item {food} on tick {}: target {:?}", self.ticks, best);
    }

    /// SPEC-0047 class 20 for each such creature of the current world: the nearest world item
    /// other than the anchor within 25 units is approached and consumed (removed); the seek
    /// sets the reservation's slow flag; otherwise it wanders about its spawn point.
    fn class20_step(&mut self) {
        let pack = self.world.pack;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.class20.is_some() && !c.inactive && !c.held).map(|c| c.id).collect();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let at = self.characters[i].position;
            let object = self.pools.get(&pack).and_then(|pool| {
                pool.slots
                    .iter()
                    .flatten()
                    .filter(|it| it.id != neopets_sim::petpet::BAGGUSS)
                    .map(|it| (it.position, it.token, (it.position[0] - at[0]).hypot(it.position[2] - at[2])))
                    .filter(|o| o.2 <= 25.0)
                    .min_by(|a, b| a.2.total_cmp(&b.2))
                    .map(|(p, t, _)| (p, t))
            });
            let world = &self.world;
            let c = &mut self.characters[i];
            let clip = c.anim.clip_state();
            let v = neopets_sim::petpet::Class20View { position: c.position, object, clip_action: clip.action, clip_done: clip.progress >= 0.99 };
            let Some(k) = c.class20.as_mut() else { continue };
            let act = k.tick(&v, self.class20_waits, &mut self.rng);
            if let Some(a) = act.anim {
                if clip.action != a {
                    c.anim.play(a);
                }
            }
            if let Some(wk) = c.walker.as_mut() {
                if let Some(t) = act.toward {
                    wk.steer_to(t, 0.1);
                }
                let ground = &world.ground;
                wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                c.position = wk.position();
                c.facing = wk.facing;
            }
            if act.seeking && self.pet_reserve.food.is_some() {
                self.pet_reserve.slow = true;
            }
            if let Some(token) = act.consume {
                if let Some(item) = self.remove_food(token) {
                    self.last_pet = format!("class 20 creature {id} consumed item {item} on tick {}", self.ticks);
                }
            }
        }
    }

    /// SPEC-0047 class 33 for each wild petpet of the current world: the loop, its animation,
    /// walk and turn, the food reservation, and on eating the join (Activation by id): its
    /// meal's time becomes the live petpet's T and F, it is removed from the world, and the
    /// live petpet spawns at its place. Whether the food can be reached is not tested (every
    /// food counts as reachable).
    fn wild_step(&mut self) {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.wild.is_some() && !c.inactive && !c.held).map(|c| c.id).collect();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let at = self.characters[i].position;
            let (food, anchor) = (self.nearest_food(at, false), self.nearest_food(at, true).map(|f| f.0));
            let world = &self.world;
            let c = &mut self.characters[i];
            let clip = c.anim.clip_state();
            // The food's reach (SPEC-0047 Food reach): its point located in the pet's current
            // navmesh; no line of sight, and the next-nearest food is not tried.
            let reachable = food.is_some_and(|f| c.walker.as_ref().and_then(|w| w.mesh).and_then(|m| world.navmeshes.get(m)).is_some_and(|m| m.first_holding(f.0).is_some()));
            let home = *c.wild.as_mut().map(|w| w.home.get_or_insert(c.position)).unwrap_or(&mut c.position);
            let v = neopets_sim::petpet::WildView { position: c.position, facing: c.facing, hero, centre: home, food, reachable, anchor, clip_action: clip.action, clip_done: clip.progress >= 0.99 };
            let Some(w) = c.wild.as_mut() else { continue };
            let act = w.tick(&v, &mut self.rng);
            if let Some(a) = act.anim {
                if clip.action != a {
                    c.anim.play(a);
                }
            }
            if let Some((t, rate)) = act.turn {
                c.facing = neopets_sim::hero::steer(c.facing, [t[0] - c.position[0], 0.0, t[2] - c.position[2]], rate);
                if let Some(wk) = c.walker.as_mut() {
                    wk.facing = c.facing;
                }
            }
            if let Some(wk) = c.walker.as_mut() {
                if let Some((t, rate)) = act.toward {
                    wk.steer_to(t, rate);
                }
                let ground = &world.ground;
                wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                c.position = wk.position();
                c.facing = wk.facing;
            }
            if let Some(t) = act.reserve {
                self.pet_reserve.reserve(t);
            }
            if let Some((token, fid)) = act.eat {
                let c = &mut self.characters[i];
                let (ty, pos) = (c.pet_type, c.position);
                let Some(kind) = neopets_sim::petpet::kind(ty) else { continue };
                let Some(w) = c.wild.as_mut() else { continue };
                w.eat(self.pet_food.value(kind, fid));
                let t = w.t;
                self.remove_food(token);
                self.pet_reserve.clear();
                if t < 1 {
                    continue;
                }
                self.characters.remove(i);
                self.activate_pet(ty, t, t);
                self.pet_spawn = Some(pos);
                self.last_pet = format!("wild petpet {ty} ate item {fid} on tick {} and joins with T {t}", self.ticks);
            }
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
            // The mouth point (bone 25) and its horizontal forward, and the hero's armed shot
            // slots with their damage by the hero shot rule (trunc(S x C x T), defence 0).
            let mouth_frame = frames.get(25).map(|f| place.mul(f));
            let mouth = mouth_frame.as_ref().map_or(head, |m| m.t);
            let mouth_forward = mouth_frame.as_ref().map_or(c.facing, |m| {
                let z = m.cols[2];
                let l = z[0].hypot(z[2]).max(1e-6);
                [z[0] / l, 0.0, z[2] / l]
            });
            let t_type = c.row.map_or(0, |r| r.target_type as i32);
            let mote = self.equipment.motes[1][2];
            let gst = mote.and_then(|m| self.items.get(m)).map_or(0.0, |it| it.stats[6] as f32);
            let shot_slots: Vec<(usize, [f32; 3], f32, i32)> = self
                .hero_shots
                .iter()
                .enumerate()
                .filter(|(_, s)| s.armed())
                .map(|(k, s)| {
                    let cf = if neopets_sim::mote::matches(s.element, t_type) { neopets_sim::mote::one_plus_percent(gst) } else { 1.0 };
                    let damage = neopets_sim::charm::damage(neopets_sim::charm::scale(s.scale, cf, s.t, 1.0, 1.0), 1.0, 0.0, 1.0).0;
                    (k, s.position, s.radius, damage)
                })
                .collect();
            let clip = c.anim.clip_state();
            // NEOPETS_TRACE_JUPPIE: the state, clip, progress and the root and head heights.
            if std::env::var_os("NEOPETS_TRACE_JUPPIE").is_some() {
                let root = frames.first().map_or(0.0, |f| place.mul(f).t[1]);
                println!("      juppie tick {}: {:?} stage {} clip {} {:.2} root y {:.2} head y {:.2}", self.ticks, c.boss.as_ref().map(|b| b.top), c.boss.as_ref().map_or(0, |b| b.stage), clip.action, clip.progress, root, head[1]);
            }
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
                release: c.anim.frame_bits() & 0x20 != 0,
                mouth,
                mouth_forward,
                shots: &shot_slots,
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
                        if !e.is_dead() {
                            o.hp = 0;
                            e.die(&mut self.rng);
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
            // SPEC-0015 Spit: one projectile from the mouth at the hero's feet into the pool.
            if act.spit {
                let setup = neopets_sim::juppie::spit_setup(mouth, hero, self.juppie_words.spit_damage, id);
                self.shots.submit(setup);
            }
            // The hit helper: bounced shots fly on; counted ones are disarmed.
            for &k in &act.bounced {
                if let Some(s) = self.hero_shots.get_mut(k) {
                    s.bounce(&mut self.rng);
                }
            }
            let mut gone = act.absorbed.clone();
            gone.sort_unstable();
            for &k in gone.iter().rev() {
                if k < self.hero_shots.len() {
                    self.hero_shots.remove(k);
                }
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
                self.state.reset_levels();
                self.drop_camera_play();
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
                self.state.reset_levels();
                self.drop_camera_play();
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
        // SPEC-0028 Book station: on each tick the body overlaps one, the pending book index
        // takes its book value.
        for (v, value) in &self.world.books {
            if v.touches(p) {
                self.books.pending = *value as i32;
            }
        }
        fired.sort_unstable();
        fired.dedup();
        for &i in &fired {
            self.world.records.entry(i).or_default().fire(frame, true, -1);
        }
        // SPEC-0028 Hero shot pass: each live shot fires the volumes its point and radius touch,
        // with object 0 (the shot word set); a shot can fire several triggers in one tick.
        let mut shot: Vec<u16> = self.hero_shots.iter().flat_map(|s| self.world.shot_volumes.iter().filter(|v| v.shot_hits(s.position, s.radius)).map(|v| v.index)).collect();
        shot.sort_unstable();
        shot.dedup();
        for &i in &shot {
            self.world.records.entry(i).or_default().fire(frame, false, -1);
        }
        self.state.shot_hits = shot.iter().copied().collect();
        fired.extend(shot.iter().filter(|i| !fired.contains(i)).copied().collect::<Vec<_>>());
        // SPEC-0048 Trigger volumes: a block that moved this tick (or stopped on it) clears its
        // last-fired volume and fires every volume its cylinder touches with object 0 and its
        // number; an idle block fires its last volume again each tick.
        let mask = self.world.collis_mask;
        let mut by_block: Vec<(u16, i32)> = Vec::new();
        for (k, b) in self.world.blocks.iter_mut().enumerate() {
            if b.testing {
                b.last_fired = None;
                for v in self.world.all_volumes.iter().filter(|v| v.fires_on_block(mask) && v.block_hits(b.position, b.size, b.height)) {
                    by_block.push((v.index, k as i32));
                    b.last_fired = Some(v.index);
                }
            } else if let Some(i) = b.last_fired {
                by_block.push((i, k as i32));
            }
        }
        for &(i, k) in &by_block {
            self.world.records.entry(i).or_default().fire(frame, false, k);
            if !fired.contains(&i) {
                fired.push(i);
            }
        }
        // SPEC-0030 row 276, CollideEntityWithTriggers for the entities the module's Update names:
        // each fires the volumes its position touches with object 0; the firer is kept for the
        // callback.
        let mut firers: HashMap<u16, usize> = HashMap::new();
        let pack = self.world.pack;
        for &e in level::collide_entities(pack) {
            let Some(c) = self.characters.iter().find(|c| c.pack == pack && c.entity == Some(e) && !c.inactive && c.dying.is_none()) else { continue };
            for v in self.world.volumes.iter().filter(|v| v.touches(c.position)) {
                self.world.records.entry(v.index).or_default().fire(frame, false, -1);
                firers.entry(v.index).or_insert(e);
                if !fired.contains(&v.index) {
                    fired.push(v.index);
                }
            }
        }
        // A record a block fired that is decaying (the block left) still runs its callback, so
        // the scripts see the leaving state 3.
        let leaving: Vec<u16> = self.world.records.iter().filter(|(i, r)| r.state == 3 && r.block >= 0 && !fired.contains(i)).map(|(i, _)| *i).collect();
        fired.extend(leaving);
        if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() && !by_block.is_empty() {
            let states: Vec<(u16, i32, u8)> = by_block.iter().map(|&(i, k)| (i, k, self.world.records.get(&i).map_or(0, |r| r.state))).collect();
            if self.ticks % 30 == 0 || states.iter().any(|s| s.2 == 1) {
                println!("      block fires on tick {}: {states:?}", self.ticks);
            }
        }
        // The callbacks read the pad's newly-pressed word themselves; an open box keeps it.
        let square = self.square_now && !self.dialogue_open;
        self.anim_trigger_step(frame, p, square);
        for i in fired {
            if self.world.disabled_triggers.contains(&i) {
                continue;
            }
            let rec = self.world.records.get(&i).copied().unwrap_or_default();
            if let Some(g) = self.world.groups.iter().position(|g| g.kind == 1 && g.triggers.first() == Some(&(i as i32))) {
                self.chest_step(g, rec, square);
                continue;
            }
            if let Some(g) = self.world.groups.iter().position(|g| g.kind == 6 && g.triggers.first() == Some(&(i as i32))) {
                self.dig_step(g, rec);
                continue;
            }
            // SPEC-0048 Element gate on hits, Prompt thread: on repeated fires of the sealed
            // chest's trigger 0 while its body is intact, context prompt 29 while the seal's
            // element and the slot-2 Mote's kind differ (Nova not exempt), else 30, until Square
            // is held.
            if let Some(g) = self.world.groups.iter().find(|g| g.kind == 2 && g.triggers.first() == Some(&(i as i32))).cloned() {
                let body = g.breakables.first().and_then(|&b| usize::try_from(b).ok()).and_then(|b| self.world.breakables.get(b));
                let e_t = g.breakables.get(1).and_then(|&b| usize::try_from(b).ok()).and_then(|b| self.world.breakables.get(b)).map_or(0, |b| b.element as i32);
                if rec.state == 2 && body.is_some_and(|b| matches!(b.state, 1 | 3)) && !square && self.door_prompt.is_none() && !self.talk_prompt {
                    let e_a = neopets_sim::mote::element(self.equipment.motes[0][2]);
                    self.door_prompt = Some(if e_t != 0 && e_a != e_t { 29 } else { 30 });
                }
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
                let call = level::TriggerCall { index: i, state: rec.state, object: rec.object, square, block: rec.block };
                let out = level::group_run(self.world.pack, g.kind, call, &objects);
                self.apply(&out);
                continue;
            }
            let Some(&routine) = self.world.routines.get(&i) else { continue };
            let call = level::TriggerCall { index: i, state: rec.state, object: rec.object, square, block: rec.block };
            self.state.talking = self.dialogue_open;
            if self.world.pack == 1066 {
                self.state.trigger_raw = self.rng.next_u32();
            }
            self.state.trigger_firer = firers.get(&i).copied();
            let out = level::on_trigger(self.world.pack, routine, call, &mut self.state);
            self.state.trigger_firer = None;
            if out.prompt.is_some() && self.door_prompt.is_none() && !self.talk_prompt {
                self.door_prompt = out.prompt;
            }
            self.apply(&out.actions);
        }
        for r in self.world.records.values_mut() {
            r.decay(frame);
        }
    }

    /// RegisterScriptAnimatedTrigger records (SPEC-0030 row 266): each registered record of a
    /// host object's inline list is tested in the host's current pose (the hero point taken into
    /// the host's rest frame), fires its own record and runs its routine with its index.
    fn anim_trigger_step(&mut self, frame: i64, p: [f32; 3], square: bool) {
        const BASE: u16 = 0x8000;
        let pack = self.world.pack;
        let list: Vec<(u16, usize, u8)> = self.world.anim_routines.iter().map(|(&(h, k), &r)| (k, h, r)).collect();
        for (k, h, routine) in list {
            let Some(v) = self.world.headers.get(h).and_then(|x| x.as_ref()).and_then(|x| x.volumes.iter().find(|v| v.index == k)).map(sim_volume) else { continue };
            let (Some(m), Some(rest)) = (self.object_pose(pack, h, 0), self.world.objects.get(h).and_then(|o| o.as_ref()).map(|o| o.rest)) else { continue };
            let d = [p[0] - m[12], p[1] - m[13], p[2] - m[14]];
            let col = |c: usize| [m[4 * c], m[4 * c + 1], m[4 * c + 2]];
            let dot = |a: [f32; 3]| (a[0] * d[0] + a[1] * d[1] + a[2] * d[2]) / (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).max(1e-12);
            let local = [rest[12] + dot(col(0)), rest[13] + dot(col(1)), rest[14] + dot(col(2))];
            if !v.touches(local) {
                continue;
            }
            // Each host's records keep their own state (host x 64 + record).
            let rec = self.world.records.entry(BASE + (h as u16) * 64 + k).or_default();
            rec.fire(frame, true, -1);
            let rec = *rec;
            let call = level::TriggerCall { index: k, state: rec.state, object: rec.object, square, block: -1 };
            self.state.talking = self.dialogue_open;
            let out = level::on_trigger(pack, routine, call, &mut self.state);
            if out.prompt.is_some() && self.door_prompt.is_none() && !self.talk_prompt {
                self.door_prompt = out.prompt;
            }
            self.apply(&out.actions);
        }
    }

    /// An object's current pose regardless of the draw gate: the motion sample at its time on its
    /// base (the linked locator's current position, else the rest translation), or its rest
    /// matrix while no script has touched it.
    fn object_pose(&self, pack: u32, i: usize, depth: u8) -> Option<[f32; 16]> {
        let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
        let obj = w.objects.get(i)?.as_ref()?;
        let a = w.object_anims.get(i)?;
        let base = self.obj_base(pack, i, depth);
        let m = w.motions.get(i)?;
        if !a.scripted || m.keys == 0 {
            let mut r = obj.rest;
            r[12..15].copy_from_slice(&base);
            return Some(r);
        }
        let (q, root, scale) = m.sample(a.time);
        let r = neopets_sim::object::rotation(q);
        let c = |k: usize| r[k].map(|v| v * scale[k]);
        let (c0, c1, c2) = (c(0), c(1), c(2));
        Some([c0[0], c0[1], c0[2], 0.0, c1[0], c1[1], c1[2], 0.0, c2[0], c2[1], c2[2], 0.0, base[0] + root[0], base[1] + root[1], base[2] + root[2], 1.0])
    }

    /// An object's base: its linked locator's current position (SPEC-0028 section 7.1 step 5),
    /// else its rest translation.
    fn obj_base(&self, pack: u32, i: usize, depth: u8) -> [f32; 3] {
        let Some(w) = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack) else { return [0.0; 3] };
        let rest = w.objects.get(i).and_then(|o| o.as_ref()).map_or([0.0; 3], |o| [o.rest[12], o.rest[13], o.rest[14]]);
        match w.obj_links.get(&i) {
            Some(&(h, k)) if depth < 4 && h != i => self.anim_locator(pack, h, k, depth + 1).unwrap_or(rest),
            _ => rest,
        }
    }

    /// Formula A (SPEC-0030 Note on animated locators): the current position of locator k of
    /// host h, P = T + r + R (L - T) with T the rest translation.
    fn anim_locator(&self, pack: u32, h: usize, k: usize, depth: u8) -> Option<[f32; 3]> {
        let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
        let l = w.headers.get(h)?.as_ref()?.locators.get(k)?.0;
        let rest = w.objects.get(h)?.as_ref()?.rest;
        let m = self.object_pose(pack, h, depth)?;
        let d = [l[0] - rest[12], l[1] - rest[13], l[2] - rest[14]];
        Some(std::array::from_fn(|j| m[12 + j] + m[j] * d[0] + m[4 + j] * d[1] + m[8 + j] * d[2]))
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
    /// sequence as a script thread. Not built: SetHeroEquippedState 0 and the HUD reset (step 2).
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
        // Step 6: the camera shake (2.0, 0.3, 25.0, 0.5, 0.8, 1.5708, 0.2).
        v.push(Action::CameraShake { t: 2.0, a: 0.3, s: 25.0, p4: 0.5, r: 0.8, phi: 1.5708, z: 0.2 });
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
        self.l1_held = input.guard;
        self.rope_y = -neopets_sim::camera::converter_a(pad.right_y);
        self.ticks += 1;
        // The hit flashes run down one frame a tick (60 ticks a second).
        self.hero_flash = self.hero_flash.saturating_sub(1);
        for c in &mut self.characters {
            c.flash = c.flash.saturating_sub(1);
        }
        // Developer cheats: full health and magic every tick.
        if self.cheats.god {
            self.hero_hp = self.hero_max;
        }
        if self.cheats.magic {
            self.hero_mp = self.hero_mp_max;
        }
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
                // applies the snapshot; otherwise the world is torn down (the title flow).
                if self.snapshot.is_some() {
                    self.reload_wanted = true;
                } else {
                    self.teardown_wanted = true;
                    self.last_hurt = format!("game over: the world is torn down on tick {}", self.ticks);
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
        self.square_held = input.square_held;
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
        self.state.hero.fire = input.attack;
        self.state.hero.fire_held = input.attack_held;
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
        let was_open = self.quick.open;
        let marker = self.quick.marker;
        // SPEC-0021 Second quick panel: a Down edge opens it when both panels are closed and the
        // in-play gate passes (cue 0x9A); while it is open (the first closed) its handler runs,
        // takes Cross, Square and Circle, and re-arms the equipment panels each call.
        let mote_was_open = self.mote_panel.open;
        if !self.quick.open && !self.mote_panel.open && panel.down && self.loco.state != 0x19 && !self.dialogue_open {
            self.mote_panel.open = true;
            panel.down = false;
            self.cue_fixed(0x9A);
        } else if self.mote_panel.open && !self.quick.open {
            let mi = neopets_sim::quick::MoteInput {
                left: panel.left,
                right: panel.right,
                up: panel.up,
                down: panel.down,
                cross: input.jump,
                square: input.square,
                circle: input.attack,
                triangle: panel.triangle,
            };
            let h = self.hero_index.min(1);
            let words = self.equipment.motes[h];
            let items = self.state.items.clone();
            let held = |id: u16| items.get(&id).is_some_and(|&n| n > 0);
            let r = self.mote_panel.tick(&mi, &self.mote_table, self.shield.is_some(), self.weapon.is_some(), &held, words);
            if std::env::var_os("NEOPETS_TRACE_MOTE_PANEL").is_some() && (mi.circle || mi.cross || mi.square || mi.left || mi.right || mi.up || mi.down) {
                println!("mote panel tick {}: {mi:?} -> {r:?}, selection {}", self.ticks, self.mote_panel.selection);
            }
            panel = neopets_sim::quick::PanelInput::default();
            if r.moved {
                self.cue_fixed(0x9A);
            }
            match r.word {
                Some((k, Some(id))) => {
                    self.item_command(&crate::menus::Command::EquipMote { word: k, id });
                    self.hud.tween_requests.push(neopets_sim::hud::TweenRequest::Mote { selection: self.mote_panel.selection, word: k });
                }
                Some((k, None)) => {
                    if self.equipment.clear_mote(h, k) {
                        self.cue_fixed(0xA4);
                    }
                }
                None => {}
            }
            for w in 0..3 {
                self.hud.rearm_word(w);
            }
        }
        let inv = &self.state.items;
        let owns = |id: i32| id >= 0 && inv.get(&(id as u16)).is_some_and(|&n| n > 0);
        let out = self.quick.tick(panel, self.loco.state != 0x19 && !self.mote_panel.open, &self.quick_rows, &owns);
        if self.quick.open {
            self.mote_panel.open = false;
        }
        // SPEC-0023 section 4: selecting or using a quick item re-arms G0, G1 and C; L3 re-arms
        // every group.
        // SPEC-0023 section 11.1: the marker moved to a chosen entry: its icon flies in.
        if self.quick.marker != marker && self.quick.marker >= 0 && self.quick.selection < 4 {
            self.hud.tween_requests.push(neopets_sim::hud::TweenRequest::Quick { entry: self.quick.selection });
        }
        if out.use_item.is_some() || self.quick.marker != marker {
            self.hud.rearm_quick();
        }
        if std::mem::take(&mut self.l3_latch) {
            self.hud.rearm_all();
            // Section 9: L3 also sets the D-pad hint's timer.
            self.hud.d = 240;
        }
        if out.consumed || was_open {
            input.jump = false;
            input.attack = false;
        }
        // The Mote panel takes Cross, Square and Circle while it is open.
        if mote_was_open || self.mote_panel.open {
            input.jump = false;
            input.attack = false;
            input.square = false;
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
        self.attack_fx_step();
        self.shot_fx_step();
        self.items_flags_step();
        if self.hero_index == 0 && self.attack.is_none() && input.attack && (1..=6).contains(&self.loco.state) {
            self.attack = Some(neopets_sim::attack::TorAttack::start_with(self.gesture.mode));
            self.draw_items(true);
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
            self.swing_serial = self.swing_serial.wrapping_add(1);
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
            // SPEC-0036 Tor's sub-state 13 (the recoil): while the current action is not 26, the
            // recoil voice cue (13 for material 4, the only material this build requests;
            // SPEC-0048 Element recoil) and the rumble request.
            if sub == 13 && self.hero.clip_state().action != 26 {
                self.sound.play(VOICE, 13, Cue::default());
                let (p, t, m) = self.landing_rumble;
                self.rumble.request(&self.rumble_presets, p, t, m);
            }
        }
        let hits = self.hit_list.len();
        self.hero_swing();
        self.trail_step();
        self.showcase_step();
        self.corner_step();
        self.mote_objects_step();
        self.shots_step();
        // A swing hit (SPEC-0031 Creature blocks): power cue 2 in sub-state 10; else a Tor hit on
        // one of the four special creature kinds (the record's kind at +0xAC) plays that
        // creature's "impact special" cue on its block at its position (nothing when the block is
        // not held); any other victim, and every Roberta hit, weapon cue 14 (no material impact).
        if self.hit_list.len() > hits {
            let power = self.attack.as_ref().is_some_and(|a| a.sub == 10);
            let special = |kind: i32| match kind {
                240 => Some((365, 34)),
                480 => Some((391, 1)),
                405 => Some((355, 15)),
                1941 | 1942 | 1949 | 1950 => Some((361, 22)),
                _ => None,
            };
            let specials: Vec<(u32, i32, [f32; 3])> = if self.hero_index == 0 {
                self.hit_list[hits..].iter().filter_map(|id| self.characters.iter().find(|c| c.id == *id)).filter_map(|c| special(c.pet_type).map(|(b, q)| (b, q, c.position))).collect()
            } else {
                Vec::new()
            };
            if power {
                self.sound.play(neopets_sim::sound::POWER, 2, neopets_sim::sound::Play::default());
            } else if !specials.is_empty() {
                for (block, cue, at) in specials {
                    if let Some(slot) = self.sound.slots.iter().position(|&b| b == Some(block)) {
                        self.sound.play(slot, cue, neopets_sim::sound::Play { emitter: Some(at), ..Default::default() });
                    }
                }
            } else {
                self.sound.play(neopets_sim::sound::WEAPON, 14, neopets_sim::sound::Play::default());
            }
        }
        self.loco.shield = self.shield.is_some();
        self.status_step();
        self.loco.update(&input, forward, left, self.hero.clip_state());
        self.hold_step(&input);
        self.pending_step();
        // While a conversation runs the emote consumer owns the hero's main id.
        if !self.dialogue_open {
            self.hero.request(self.loco.action(), force);
        }
        self.hero.set_rate(self.loco.rate);
        let (world, other) = (&self.world, self.other.as_ref());
        let search_other = buffers::may_swap(world.buffer, self.boot.neighbour_ok(world.pack, world.kind, world.pack));
        let noclip = self.cheats.noclip;
        let hero_state = self.loco.state;
        let block_contact = !noclip && !matches!(self.loco.state, 18 | 19 | 30 | 41 | 58);
        let wall_contact = !noclip && !matches!(self.loco.state, 21 | 23 | 30 | 35 | 37 | 38 | 41 | 47);
        let side_touch = Cell::new(false);
        // The motion record's touched flag (+0x238; SPEC-0018 Slide response step 6): cleared
        // by the hero branch each step and set by any contact that writes a normal; here any
        // push out of a wall, box, block or sphere, or a block top's floor contact.
        let touched = Cell::new(false);
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
                if let (Some(g), true) = (&world.ground, wall_contact) {
                    let walls: Vec<WallEdge> = g
                        .frames_at(p[0], p[2])
                        .into_iter()
                        .map(|f| WallEdge { face: f.face, run: f.run, top: f.top, length: f.length, height: f.height })
                        .collect();
                    contact::resolve_walls(p, &walls, contact::HERO_RADIUS);
                }
                // On a ladder (state 41) the box primitives are skipped too: the mount's point 0.9
                // out from a ladder on a wall lies inside the wall's box by the hero radius, so
                // the push would stop the mount from ever aligning (this build's choice; the
                // spec skips only the walls and blocks there; R-0108).
                let before = *p;
                if !noclip && hero_state != hero_states::LADDER {
                    contact::resolve(p, &world.boxes, contact::HERO_RADIUS);
                }
                // SPEC-0048 Solid for characters: the movable blocks' boxes; a top face sets the
                // floor-contact flag and a side face of a block 1 to 5 taller the side-touch latch.
                let c = if block_contact { neopets_sim::block::contact(&world.blocks, p) } else { Default::default() };
                side_touch.set(c.side_touch);
                // SPEC-0018 Sphere contact: out of the pushing spheres to r + 1.25.
                if !noclip {
                    contact::resolve_spheres(p, &world.push_spheres);
                }
                let moved = (0..3).any(|k| (p[k] - before[k]).abs() > 1e-5);
                if moved || c.floor || c.side_touch {
                    touched.set(true);
                }
                c.floor
            },
        );
        self.touched = touched.get();
        // The side-touch latch the next hero update reads (SPEC-0033 Behavior 15A).
        self.side_touch = side_touch.get();
        // SPEC-0055 8A: the deadly-ground kill (kind 2, gap 0.085 or less; no health change),
        // then the death-state handler of state 16.
        let hp = {
            let p = self.loco.position();
            self.world.probe(p[0], p[2], p[1] + neopets_sim::motion::PROBE_CEILING)
        };
        let deadly = (hp.found && hp.kind == Some(2))
            .then(|| neopets_sim::death::Deadly { respawn: hp.respawn.unwrap_or(0xFF), surface: hp.surface.unwrap_or(0) });
        // The probed gap (a block top under the hero is not deadly ground).
        let probed_gap = self.loco.position()[1] - hp.height;
        if neopets_sim::death::deadly_kill(deadly, probed_gap) {
            // The death effect of the surface (0x81 hot burst, 0xBC water splash; SPEC-0055 8A,
            // SPEC-0034 section 37) when the hero enters the death state.
            if self.loco.state != hero_states::DEAD {
                if let Some(e) = neopets_sim::death::death_clip(deadly).1 {
                    let table = self.effect_table.clone();
                    self.effects.one_shot(&table, e as i32, self.loco.position(), neopets_sim::effect::IDENTITY, &mut self.rng);
                }
            }
            self.death_hook();
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
        self.book_prompt = false;
        if !self.dialogue_open && !self.buy_step() && !self.talk_step() && !self.catch_step() && !self.block_step() {
            self.door_step();
            // SPEC-0022 the idle list's Feed petpet (after the door prompt; a caption set
            // earlier in the pass stops it).
            if self.door_prompt.is_none() && !self.save_prompt {
                self.feed_step();
            }
            // SPEC-0022 the idle list's Read Book (after the door prompt; a caption set earlier
            // in the pass stops it): the prompt, the pending index cleared, a Square edge takes
            // the book (cue 0xAF).
            if self.door_prompt.is_none() && self.prompt_shown.is_none() && !self.save_prompt {
                let (shown, took) = self.books.step(self.square_now);
                self.book_prompt = shown;
                if took {
                    self.cue(0xAF);
                }
            }
            // The chooser's last fallback (SPEC-0040 Behavior 29 item 3): Square with no target
            // enters the sneak while the weapon is sheathed, and sheathes it while drawn. A
            // "Save Game" caption shown this pass stops it too (SPEC-0025 Save points, order
            // against the hero's Square handling).
            if self.square_now && !self.book_prompt && self.door_prompt.is_none() && self.prompt_shown.is_none() && !self.save_prompt {
                if self.weapon_drawn {
                    self.draw_items(false);
                } else {
                    self.loco.sneak();
                }
            }
        }
        if self.job.is_none() {
            self.state.hero.swap_mode = self.swap_mode;
            self.state.hero.selected = self.hero_index;
            let parked_dead = self.parked.as_ref().is_some_and(|p| p.hp <= 0);
            let live_dead = self.loco.state == hero_states::DEAD;
            self.state.hero.dead = if self.hero_index == 0 { [live_dead, parked_dead] } else { [parked_dead, live_dead] };
            self.state.hero.swapping = self.swap.is_some();
            let window = self.attack.is_some() && self.hero.frame_bits() & 1 != 0;
            self.state.hero.strike = window && !self.window_was;
            self.window_was = window;
            if self.state.carry.is_empty() {
                self.state.carry = (0..=180u16).map(|id| self.design.carry_limit(id).unwrap_or(0)).collect();
            }
            // SPEC-0026 Credits: on the play pass after story bit 368 is set, the credits open
            // (once; the teardown at their end clears every flag).
            if self.state.flag(368) && !self.credits_fired {
                self.credits_fired = true;
                self.credits_wanted = true;
                self.last_door = format!("the credits gate (flag 368) on tick {}", self.ticks);
                if std::env::var_os("NEOPETS_TRACE_CREDITS").is_some() {
                    println!("      credits opened from play (flag 368) on tick {}", self.ticks);
                }
            }
            self.trigger_step();
            let pack = self.world.pack;
            self.state.entity_hp = self.characters.iter().filter(|c| c.pack == pack).filter_map(|c| Some((c.entity?, c.hp.max(0)))).collect();
            self.state.hero.sub = self.attack.as_ref().map_or(0, |a| a.sub);
            self.state.hero.occupied = neopets_sim::hero::busy_state(self.loco.state) && self.loco.state != hero_states::ATTACK || self.held;
            self.state.hero.attack_mote = self.equipment.motes[0][2];
            self.state.hero.hp = self.hero_hp;
            self.state.hero.hp_max = self.hero_max;
            self.state.miamice_caged = self.catch.count;
            self.state.hero.shield_mote = self.equipment.motes[0][1];
            self.state.hero.square_held = self.square_held;
            self.state.hero.deflected = self.shots.deflected();
            self.state.hero.tick = self.ticks;
            self.state.hero.position = self.loco.position();
            self.state.obj_times = self.world.object_anims.iter().enumerate().map(|(i, a)| (i, a.time)).collect();
            self.state.anim_locators = level::tracked_locators(pack)
                .iter()
                .map(|&h| (h, (0..self.world.headers.get(h).and_then(|x| x.as_ref()).map_or(0, |x| x.locators.len())).filter_map(|k| self.anim_locator(pack, h, k, 0)).collect()))
                .collect();
            // The 1096 dispensers' draw (the module's random-draw routine is not given).
            if matches!(pack, 1096 | 1397 | 1023) {
                self.state.trigger_raw = self.rng.next_u32();
            }
            self.state.cloak_worn = self.cloak_worn;
            let actions = level::on_update(self.world.pack, &mut self.state, self.dialogue_open);
            // NEOPETS_TRACE_WORDS: the current module's words, object 0's time and the hero
            // every 30 ticks.
            if std::env::var_os("NEOPETS_TRACE_WORDS").is_some() && self.ticks % 30 == 0 {
                let mut w: Vec<(u8, i32)> = self.state.words.iter().filter(|((p, _), v)| *p == pack && **v != 0).map(|((_, k), v)| (*k, *v)).collect();
                w.sort_unstable();
                let p = self.loco.position();
                println!("      words of {pack} on tick {}: {w:?}; object 0 at {:.2}; hero ({:.1}, {:.1}, {:.1}) hp {} state {} held {}", self.ticks, self.state.obj_times.get(&0).copied().unwrap_or(-1.0), p[0], p[1], p[2], self.hero_hp, self.loco.state, self.held);
            }
            self.state.hero.hits.clear();
            self.state.hit_now.clear();
            self.apply(&actions);
            // The shared entity-control routine (SPEC-0043), called by the module's Update.
            let talking: Vec<usize> = self.characters.iter().filter(|c| c.pack == pack && self.dialogue_open && self.talk_with == Some(c.id)).filter_map(|c| c.entity).collect();
            let walking: Vec<usize> = self.characters.iter().filter(|c| c.pack == pack && c.script_walk.is_some()).filter_map(|c| c.entity).collect();
            let rng = &mut self.rng;
            let cmds = level::entity_step(pack, &mut self.state, &|e| talking.contains(&e), &|e| walking.contains(&e), &mut |lo, hi| rng.range(lo, hi));
            self.entity_commands(pack, cmds);
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
        self.head_look_step();
        let (at, facing) = (self.loco.position(), self.loco.facing);
        if std::env::var_os("NEOPETS_NO_CHAINS").is_none() {
            self.hero.chain_step(at, facing);
        }
        self.sound_step();
        self.clip_effects_step();
        // SPEC-0033 Behavior 1C per pass (not while the menu is up, game status 9): entries run
        // down with Vibration off, but nothing is sent then.
        let send = self.rumble.tick(self.menu_open);
        if self.options.block.vibration != 0 {
            if let Some(s) = send {
                self.motor = Some(s);
            }
        }
        if let (Some(s), true) = (send, std::env::var_os("NEOPETS_TRACE_RUMBLE").is_some()) {
            println!("      rumble on tick {}: level {} for {} ticks", self.ticks, s.level, s.count);
        }
        // SPEC-0028 7.1: the resident worlds' animated objects step before the hero update.
        let mut object_sounds = Vec::new();
        let current = self.world.pack;
        for w in [&mut self.world].into_iter().chain(self.other.as_mut()) {
            for (o, m) in w.object_anims.iter_mut().zip(&w.motions) {
                o.tick(m);
            }
            // SPEC-0031 Animated-object sounds: a slot's start plays once (do-not-restart) on
            // the area slot at the object (its rest position stands in for the reference centre),
            // and its end stops that cue there; only the current world's area slot sounds.
            if w.pack == current {
                for (i, o) in w.object_anims.iter().enumerate() {
                    if o.sound_start.is_some() || o.sound_stop.is_some() {
                        let at = w.objects.get(i).and_then(|ob| ob.as_ref()).map(|ob| [ob.rest[12], ob.rest[13], ob.rest[14]]);
                        object_sounds.push((o.sound_start, o.sound_stop, at));
                    }
                }
            }
        }
        for (start, stop, at) in object_sounds {
            if let Some(cue) = start {
                self.sound.play(neopets_sim::sound::AREA, cue, neopets_sim::sound::Play { emitter: at, do_not_restart: true, radius: Some((200.0, 0.0)), ..Default::default() });
            }
            if let Some(cue) = stop {
                self.sound.stop(neopets_sim::sound::AREA, cue, at);
            }
        }
        for w in [&mut self.world].into_iter().chain(self.other.as_mut()) {
            // SPEC-0048 Per tick: the movable-block list after the object animations.
            for b in &mut w.blocks {
                b.tick();
            }
        }
        // Diagnostic (trace only): blocks slid toward a point at 0.2 units a tick, firing the
        // volumes they touch as a push does.
        let pack = self.world.pack;
        self.diag_slides.retain(|&(k, t)| {
            let Some(b) = self.world.blocks.get_mut(k) else { return false };
            let d = [t[0] - b.position[0], t[1] - b.position[2]];
            let l = d[0].hypot(d[1]);
            let step = l.min(0.2);
            if l > 0.0 {
                b.position[0] += d[0] / l * step;
                b.position[2] += d[1] / l * step;
            }
            b.testing = true;
            let _ = pack;
            l > 0.2
        });
        self.owner_pose_step();
        self.status_display_step();
        self.shake_now = self.shake.step();
        // SPEC-0023 section 31: the countdown timer steps in play, not while a box is up.
        if !self.dialogue_open {
            self.state.timer.tick();
        }
        self.zone_step();
        self.foliage_step();
        self.placed_release_step();
        self.placed_zone_step();
        let table = self.effect_table.clone();
        self.effects.tick(&table, &mut self.rng);
        self.light_step();
        // IsAI_Dead (SPEC-0030 row 45): a bound character's dead test, else the kill memory of
        // this load for its entity id.
        for c in self.characters.iter().filter(|c| c.dying.is_some() || c.enemy.as_ref().is_some_and(|e| e.is_dead())) {
            if let Some(e) = c.entity {
                self.killed.insert((c.pack, e));
            }
        }
        let pack = self.world.pack;
        self.state.ai_dead = self.killed.iter().filter(|(p, _)| *p == pack).map(|&(_, e)| e).collect();
        self.state.entity_mesh = self.characters.iter().filter(|c| c.pack == pack && !c.inactive).filter_map(|c| Some((c.entity?, c.walker.as_ref()?.mesh.map_or(-1, |m| m as i32)))).collect();
        self.state.ai_dead.retain(|&e| !self.characters.iter().any(|c| c.pack == pack && c.entity == Some(e) && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|x| x.is_dead()) && c.hp > 0));
        self.state.aggressive = self.characters.iter().filter(|c| c.pack == pack && !c.inactive && c.dying.is_none() && c.enemy.as_ref().is_some_and(|e| e.state == 19 && !e.is_dead())).filter_map(|c| c.entity).collect();
        self.slot_arrived.retain(|(p, _)| *p == pack || self.other.as_ref().is_some_and(|o| o.pack == *p));
        // The switch-group stages whose 0.4 s has passed become breakable.
        let (now, cur) = (self.ticks, self.world.pack);
        let due: Vec<usize> = self.chain_arms.iter().filter(|a| a.0 == cur && a.2 <= now).map(|a| a.1).collect();
        self.chain_arms.retain(|a| a.0 == cur && a.2 > now);
        for i in due {
            if let Some(b) = self.world.breakables.get_mut(i).filter(|b| b.state == 3) {
                b.state = 1;
            }
        }
        self.state.breakables = self.world.breakables.iter().enumerate().map(|(i, b)| (i, b.state)).collect();
        self.state.slot_alive = self
            .slot_arrived
            .iter()
            .filter(|(p, _)| *p == pack)
            .map(|&(_, slot)| (slot, self.characters.iter().filter(|c| c.pack == pack && c.slot == slot && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|x| x.is_dead()) && c.hp > 0).count()))
            .collect();
        self.state.animating = self.world.object_anims.iter().enumerate().filter(|(_, a)| a.animating()).map(|(i, _)| i).collect();
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
        // A released world's pool resets every slot: its important dropped items go to the
        // party (SPEC-0021 Pool lifetime and Important-item give).
        let released: Vec<u32> = self.pools.keys().copied().filter(|p| !resident.contains(p)).collect();
        for pack in released {
            if let Some(mut pool) = self.pools.remove(&pack) {
                for id in pool.release() {
                    if self.design.item_important(id) {
                        self.state.give(id, 1);
                    }
                }
            }
        }
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
        // Mortogs (SPEC-0029 Behavior 3): the routine against the world's zone (none: the data
        // defaults, L = 0), the shared zone count, the motion-record tick.
        let zone = match self.mortog_zone {
            Some((p, z, _)) if p == world.pack => z,
            _ => neopets_sim::mortog::Zone { centre: [0.0; 3], l: 0.0, hold: self.mortog_words.hold },
        };
        for c in self.characters.iter_mut().filter(|c| c.pack == world.pack && !c.held && !c.inactive) {
            let (Some(wk), Some(m)) = (c.walker.as_mut(), c.mortog.as_mut()) else { continue };
            let before = m.state;
            let id = m.tick(wk, &world.navmeshes, &zone, &self.mortog_words, &mut self.state.mortogs_on_target, hero, &mut self.rng);
            if m.state != before {
                self.last_boss = format!("Mortog {} {:?} -> {:?} (zone count {}) on tick {}", c.id, before, m.state, self.state.mortogs_on_target, self.ticks);
            }
            c.anim.play(id);
            let ground = &world.ground;
            wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
            c.position = wk.position();
            c.facing = wk.facing;
        }
        self.feed_finish();
        self.wild_step();
        self.class20_step();
        self.pet_step();
        self.food_fallback();
        self.boss_step();
        self.gnarfas_step();
        self.king_step();
        self.knight_step();
        self.gelert_step();
        self.ixi_step();
        self.sisters_step();
        self.draik_step();
        self.faerie_step();
        self.fyora_step();
        self.serpent_step();
        self.mice_step();
        self.cloud_beast_step();
        self.enemies_step();
        self.lock_step();
        self.npc_block_step();
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
            self.hud_loss_step();
            self.hud.tick_icons();
            // Section 12: the swing ticks in the in-play state, not under a dialogue box.
            if !self.dialogue_open {
                self.hud.swing.tick();
            }
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
                // SPEC-0040 Behavior 14D item 8: the townsfolk speeds and walk ids by movement
                // type (walk 1 at 0.08337 a tick, run 6 at 0.22504, fast run 5 at 0.32160).
                let (speed, clip) = match c.move_type {
                    1 => (0.225_04, 6),
                    2 => (0.321_60, 5),
                    3 | 4 => (0.0, 3),
                    _ => (0.083_37, 1),
                };
                if d >= stop {
                    c.facing = neopets_sim::hero::steer(c.facing, [dx, 0.0, dz], 0.3);
                    let s = speed / d;
                    let (x, z) = (c.position[0] + dx * s, c.position[2] + dz * s);
                    let y = if c.pack == world.pack { world.ground_at(x, z, c.position[1] + 2.5) } else { None };
                    c.position = [x, y.unwrap_or(c.position[1]), z];
                    if c.anim.clip_state().action != clip {
                        c.anim.play(clip);
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
    /// SPEC-0040 Behavior 20 items 1 to 5 each tick: the candidates are the current world's
    /// living characters the hero's attacks can hit (not townsfolk, not petpet types; the
    /// spawn-record foe test is read as this), engaged by an attack (state 13) while facing them.
    fn lock_step(&mut self) {
        let pack = self.world.pack;
        let chars: Vec<(u64, [f32; 3])> = self
            .characters
            .iter()
            .filter(|c| c.pack == pack && !c.inactive && c.dying.is_none() && c.hp > 0 && !c.enemy.as_ref().is_some_and(|e| e.is_dead() || e.lock_vetoed(c.anim.clip_state().action)) && c.townsfolk.is_none() && c.row.is_some() && !c.unhittable())
            .map(|c| (c.id, c.position))
            .collect();
        let attacking = self.loco.state == hero_states::ATTACK;
        self.lock = self.candidates.tick(&chars, self.loco.position(), self.loco.facing, attacking);
    }

    /// The target point of a character (SPEC-0040 Behavior 20 item 6, R-0102): the shared
    /// non-hero default, render attachment point 7 posed on the character; the Cloud Beast's
    /// override, its companion's pose point 1 (the stand-in's orb); a character without point 7
    /// gives its position (the spec's default: no aim point from body height).
    fn aim_point(&self, id: u64) -> Option<[f32; 3]> {
        let c = self.characters.iter().find(|c| c.id == id)?;
        if c.cloud.is_some() {
            let orb = self.characters.iter().find(|o| o.pack == c.pack && o.slot == CLOUD_SLOT && o.cloud.is_none() && o.row.is_some_and(|r| r.class == Some(10)));
            return Some(orb.map_or(c.position, |o| add(o.position, [0.0, 0.6, 0.0])));
        }
        Some(Self::attach_point(c, 7).unwrap_or(c.position))
    }

    /// Render attachment point n of a character in the world.
    fn attach_point(c: &Character, n: usize) -> Option<[f32; 3]> {
        let &(Some(bone), angles, offset) = c.anim.bones().attach.get(n)? else { return None };
        let frames = c.anim.bone_frames();
        let m = placement(c.position, c.facing).mul(frames.get(bone)?).mul(&attach_rotation(offset, angles));
        Some(m.t)
    }

    /// SPEC-0036 Roberta shot launch step 1 for homing modes 2 to 5: the first ten hittable
    /// characters within 60 horizontally whose aim point is visible along three parallel rays
    /// (from the hero + 1.5 up, and 1.0 to each side), one chosen uniformly.
    fn shot_target(&mut self) -> Option<u64> {
        let pack = self.world.pack;
        let h = self.loco.position();
        let eye = add(h, [0.0, 1.5, 0.0]);
        let mut found = Vec::new();
        for c in self.characters.iter().filter(|c| c.pack == pack && !c.inactive && c.dying.is_none() && c.hp > 0 && !c.enemy.as_ref().is_some_and(|e| e.is_dead()) && c.townsfolk.is_none() && c.row.is_some() && !c.unhittable()) {
            if found.len() >= 10 {
                break;
            }
            if (c.position[0] - h[0]).hypot(c.position[2] - h[2]) >= 60.0 {
                continue;
            }
            let at = c.stand_in_body().map(|b| b.center).or_else(|| Self::attach_point(c, 7)).unwrap_or(c.position);
            let d = unit([at[0] - eye[0], 0.0, at[2] - eye[2]]);
            let side = [d[2], 0.0, -d[0]];
            let clear = |o: f32| self.world.ground.as_ref().is_none_or(|g| g.segment_clear(add(eye, scale(side, o)), add(at, scale(side, o)), true));
            if clear(0.0) && clear(1.0) && clear(-1.0) {
                found.push(c.id);
            }
        }
        (!found.is_empty()).then(|| found[self.rng.index(found.len() as u32) as usize])
    }

    /// SPEC-0010 Test spawn (NEOPETS_CLOUD_BEAST): the cut Cloud Beast (type 1, variant 0, row
    /// 1, class 9) 12 units in front of the hero and its companion (type 2, row 2, class 10)
    /// riding on it, both with the stand-in models of `placeholder` (the disc has no model,
    /// skeleton or clip bank for either) and no animation families. Respawned when missing from
    /// the current world. The class-9 controller is not built: the boss stays where it spawned.
    fn cloud_beast_step(&mut self) {
        let Some((boss_row, companion_row)) = self.cloud_rows.filter(|_| self.cloud_on) else { return };
        let pack = self.world.pack;
        let find = |s: &Self, class: i16| s.characters.iter().position(|c| c.pack == pack && c.slot == CLOUD_SLOT && c.row.is_some_and(|r| r.class == Some(class)));
        if self.cloud_defeated.contains(&pack) {
            return;
        }
        if find(self, 9).is_none() {
            let f = self.loco.facing;
            // The stand-in arena: a ring of radius 16 around the hero, point 0 ahead of the hero.
            let centre = self.loco.position();
            let a0 = f[0].atan2(f[2]);
            let ring: [([f32; 3], [f32; 3]); 6] = std::array::from_fn(|k| {
                let a = a0 + k as f32 * std::f32::consts::FRAC_PI_3;
                let d = [a.sin(), 0.0, a.cos()];
                (add(centre, scale(d, 16.0)), [-d[0], 0.0, -d[2]])
            });
            // SPEC-0010 Arena: the level locators with user ids 101 to 106 when all six exist (the
            // Juppie and Plague Serpent arenas); otherwise the ring.
            let w = &self.world;
            let tagged: Vec<([f32; 3], [f32; 3])> = (101..=106).filter_map(|k| w.tags.iter().position(|&t| t == k).and_then(|i| w.entries.get(i)).copied()).collect();
            let ring = <[([f32; 3], [f32; 3]); 6]>::try_from(tagged).unwrap_or(ring);
            self.cloud_arena = Some((pack, ring));
            let at = ring[0].0;
            // The menu's fight entry: the hero to the middle of the six points, facing point 0.
            if std::mem::take(&mut self.cloud_place_hero) {
                let c = ring.iter().fold([0.0; 3], |a, (p, _)| add(a, scale(*p, 1.0 / 6.0)));
                let d = sub_v(at, c);
                let l = d[0].hypot(d[2]).max(1e-3);
                self.arrive((c, [d[0] / l, 0.0, d[2] / l]));
            }
            if std::env::var_os("NEOPETS_TRACE_CLOUD").is_some() {
                println!("CLOUD arena {:?}", ring.map(|(p, _)| p));
            }
            let back = [-f[0], 0.0, -f[2]];
            let bones = neopets_data::anim::BoneTable { parents: Vec::new(), rest_local: Vec::new(), rest_position: Vec::new(), hurt: Vec::new(), attach: Vec::new() };
            let placement = |ordinal: usize, name: &str, monster_type: u16| neopets_data::region::Placement {
                ordinal,
                position: at,
                facing: back,
                name: name.into(),
                skeletons: [-1; 3],
                tables: [-1; 3],
                model_set: -1,
                monster_type,
                variant: 0,
                ai_code: 0,
                scale_set: 0,
                summon: false,
                key: -1,
                emerge_distance: 0.0,
                selector: 0,
                alarm: 0xFF,
                drop: -1,
                colour: [128; 3],
                colours: [0x00FF_FFFF; 15],
            };
            self.characters.retain(|c| !(c.pack == pack && c.slot == CLOUD_SLOT));
            let anim = || HeroAnim::new(bones.clone(), Vec::new());
            self.bind_character(pack, CLOUD_SLOT, pack, placement(usize::MAX - 1, "Cloud Beast (stand-in)", 1), (crate::placeholder::cloud_beast(), anim(), Some(boss_row), None), None);
            self.bind_character(pack, CLOUD_SLOT, pack, placement(usize::MAX, "Cloud Beast companion (stand-in)", 2), (crate::placeholder::cloud_companion(), anim(), Some(companion_row), None), None);
            if let (Some(w), Some(c)) = (self.cloud_words, find(self, 9)) {
                self.characters[c].cloud = Some(neopets_sim::cloudbeast::Cloud::new(&w, &mut self.rng));
                // NEOPETS_CLOUD_HP=n: the boss starts with n hit points (trace only).
                if let Some(hp) = std::env::var("NEOPETS_CLOUD_HP").ok().and_then(|v| v.parse::<i32>().ok()) {
                    self.characters[c].hp = hp;
                }
            }
            self.last_boss = format!("Cloud Beast stand-in spawned at {at:?} on tick {}", self.ticks);
            // A boss fight's opening stinger (4, as the level scripts' fight starts use; a guess).
            self.stinger(4);
        }
        self.cloud_fight(pack);
        // The companion rides on the boss.
        if let (Some(b), Some(c)) = (find(self, 9), find(self, 10)) {
            // SPEC-0010 Companion: pinned to the boss (bone 11 of the stand-in: above the cloud)
            // and turned toward the hero with weight 0.1 along the horizontal direction.
            let p = self.characters[b].position;
            let hero = self.loco.position();
            let k = &mut self.characters[c];
            k.position = [p[0], p[1] + crate::placeholder::COMPANION_RIDE, p[2]];
            let to = [hero[0] - k.position[0], 0.0, hero[2] - k.position[2]];
            k.facing = neopets_sim::hero::steer(k.facing, to, 0.1);
        }
    }

    /// SPEC-0010, one controller tick of the spawned Cloud Beast: the view (the stand-in arena,
    /// the companion's orb as pose point 1, the hero's attack, guard and Fog Mote, Roberta's
    /// shots), then the boss's place and facing, its hits on the hero (not blocked, no armour),
    /// the gust's knock-back, the reply's self-damage and the death. Guesses where the disc has
    /// nothing: the beam's wall sweep finds its hit by halving the segment against the level's
    /// segment query and takes the reversed beam direction as the face normal; at death the
    /// companion goes with the boss and the world is marked so the boss does not respawn.
    fn cloud_fight(&mut self, pack: u32) {
        let Some(w) = self.cloud_words else { return };
        self.cloud_bursts.retain_mut(|(_, t)| {
            *t = t.saturating_sub(1);
            *t > 0
        });
        let find = |s: &Self, class: i16| s.characters.iter().position(|c| c.pack == pack && c.slot == CLOUD_SLOT && c.row.is_some_and(|r| r.class == Some(class)));
        let Some(b) = find(self, 9) else { return };
        let companion = find(self, 10);
        if self.characters[b].hp <= 0 || self.characters[b].dying.is_some() {
            if self.cloud_defeated.insert(pack) {
                let disappear = self.characters[b].row.map_or(510, |r| r.disappear.max(1) as u32);
                self.characters[b].dying.get_or_insert(disappear);
                self.characters[b].cloud = None;
                if let Some(c) = companion {
                    let own = self.characters[c].row.map_or(disappear, |r| r.disappear.max(1) as u32);
                    self.characters[c].dying.get_or_insert(own);
                }
                self.last_boss = format!("Cloud Beast defeated on tick {}", self.ticks);
                // Stinger 1, the Werelupe King's death stinger (SPEC-0019; a guess for this boss).
                self.stinger(1);
                // A reward (a guess: the disc gives the boss none): 200 Neopoints in coins and the
                // Wind Mote (167), whose pickup text likens it to a tiny thundercloud.
                let p = self.characters[b].position;
                let spawn = |id: u16, charm: bool| neopets_sim::items::Spawn {
                    id,
                    point: [p[0], p[1] + 1.0, p[2]],
                    placed: false,
                    charm,
                    s0: 0.25,
                    s1: 1.0,
                    delay: 0.0,
                    larg: -1.0,
                    direction: None,
                    flag: 0,
                    map: 0,
                    price: 0,
                };
                for id in neopets_sim::items::coin_split(200, &self.coins.0, &self.coins.1) {
                    self.spawn_item(pack, spawn(id, false));
                }
                let charm = self.item_kind(167) == 17;
                self.spawn_item(pack, spawn(167, charm));
                if std::env::var_os("NEOPETS_TRACE_CLOUD").is_some() {
                    println!("CLOUD defeated on tick {}", self.ticks);
                }
            }
            return;
        }
        let Some(mut cloud) = self.characters[b].cloud.take() else { return };
        let hero = self.loco.position();
        let shots: Vec<([f32; 3], [f32; 3])> = if self.hero_index == 1 { self.hero_shots.iter().map(|s| (s.position, scale(s.dir, s.speed / 60.0))).collect() } else { Vec::new() };
        let world = &self.world;
        let ground = |x: f32, z: f32, c: f32| world.ground_at(x, z, c).unwrap_or(0.0);
        let wall = |a: [f32; 3], e: [f32; 3]| -> Option<(f32, [f32; 3])> {
            let g = world.ground.as_ref()?;
            if g.segment_clear(a, e, false) {
                return None;
            }
            let (mut lo, mut hi) = (0.0f32, 1.0f32);
            for _ in 0..12 {
                let m = 0.5 * (lo + hi);
                if g.segment_clear(a, add(a, scale(sub_v(e, a), m)), false) {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            Some((hi, unit(sub_v(a, e))))
        };
        let c = &self.characters[b];
        let v = neopets_sim::cloudbeast::View {
            position: c.position,
            forward: c.facing,
            hp: c.hp,
            max_hp: c.row.map_or(240, |r| r.hit_points as i32),
            hero,
            hero_forward: self.loco.facing,
            hero_radius: contact::HERO_RADIUS,
            hero_height: HERO_HEIGHT,
            variant: self.hero_index.min(1) as u8,
            in_attack: self.hero_index == 0 && self.attack.as_ref().is_some_and(|a| (2..=12).contains(&a.sub)),
            guard: self.loco.state == hero_states::GUARD,
            fog: self.equipment.motes[self.hero_index.min(1)][1] == Some(166),
            shots: &shots,
            companion: companion.map(|k| add(self.characters[k].position, [0.0, 0.6, 0.0])),
            points: self.cloud_arena.filter(|(p, _)| *p == pack).map(|(_, r)| r),
            ground: &ground,
            wall: &wall,
        };
        let before = (cloud.state, cloud.stage);
        let fired = cloud.beams[0].active;
        let act = cloud.tick(&v, &w, &mut self.rng);
        cloud.advance_clip();
        // NEOPETS_TRACE_CLOUD: the controller's state changes and hits (trace only).
        if std::env::var_os("NEOPETS_TRACE_CLOUD").is_some() && (before != (cloud.state, cloud.stage) || !act.hits.is_empty() || act.knock.is_some() || act.self_damage > 0) {
            println!("CLOUD tick {} {:?} stage {} -> {:?} stage {} clip {} hp {} hero hp {} dist {:.1} hits {:?} knock {:?} self {}", self.ticks, before.0, before.1, cloud.state, cloud.stage, cloud.clip.id, v.hp, self.hero_hp, (v.position[0] - hero[0]).hypot(v.position[2] - hero[2]), act.hits, act.knock, act.self_damage);
        }
        let c = &mut self.characters[b];
        if let Some(p) = act.position {
            c.position = p;
        }
        c.facing = act.forward;
        if act.self_damage > 0 {
            c.hp -= act.self_damage;
            c.flash = HIT_FLASH;
        }
        let at = add(c.position, [0.0, crate::placeholder::CLOUD_HOVER + 1.8, 0.0]);
        if act.hit_fx {
            c.flash = HIT_FLASH;
            self.cloud_bursts.push((at, 20));
        }
        // The controller makes no cue calls; these are guesses from cues this build already uses:
        // the Mote-use cue 0xA2 when the shock starts and when a bolt first fires.
        let shock_started = before.0 != neopets_sim::cloudbeast::State::Shock && cloud.state == neopets_sim::cloudbeast::State::Shock;
        let bolt_started = !fired && cloud.beams[0].active;
        self.characters[b].cloud = Some(cloud);
        if shock_started || bolt_started {
            self.cue(0xA2);
        }
        let facing = self.characters[b].facing;
        for h in act.hits {
            match h {
                neopets_sim::cloudbeast::Hit::Damage(d) => {
                    self.hero_hit(d as f32, 0.0, false, facing, "the Cloud Beast".into());
                }
                neopets_sim::cloudbeast::Hit::Blocked => {
                    self.hero_hit(0.0, 0.0, true, facing, "the Cloud Beast".into());
                }
            }
        }
        if let Some(k) = act.knock {
            if self.loco.state != hero_states::DEAD {
                self.loco.knock(k);
            }
        }
        for p in act.bursts {
            self.cloud_bursts.push((p, 20));
        }
    }

    /// The stand-in effects of the Cloud Beast for drawing (its own effect definitions 447 to
    /// 454 are not on the disc): the gust column (O, E, fade 0 to 128), the live beams, the
    /// balls, the shock (centre and radius) and the bursts.
    #[allow(clippy::type_complexity)]
    pub fn cloud_fx(&self) -> Option<(Option<([f32; 3], [f32; 3], i32)>, Vec<([f32; 3], [f32; 3])>, Vec<[f32; 3]>, Option<([f32; 3], f32)>)> {
        let c = self.characters.iter().find(|c| c.pack == self.world.pack && c.cloud.is_some())?;
        let cl = c.cloud.as_ref()?;
        let beams = cl.beams.iter().filter(|b| b.active && b.len > 0.0).map(|b| (b.start, b.end())).collect();
        let balls = cl.balls.iter().flatten().map(|b| b.position).collect();
        let shock = cl.shock.then(|| (add(c.position, [0.0, crate::placeholder::CLOUD_HOVER + 1.0, 0.0]), self.cloud_words.map_or(8.0, |w| w.shock_radius)));
        Some((cl.gust.map(|(o, e)| (o, e, cl.fade)), beams, balls, shock))
    }

    /// The stand-in body's pose for a Cloud Beast character (boss or companion): a bob, a
    /// forward lean (radians) and a scale, from the boss's current clip (the disc has no clips).
    pub fn cloud_pose(&self, c: &Character) -> Option<(f32, f32, f32)> {
        if c.slot != CLOUD_SLOT {
            return None;
        }
        // Defeated: the cloud shrinks and sinks over its first two seconds of the corpse timer.
        if let (Some(t), Some(r)) = (c.dying, c.row) {
            let k = ((t as f32 - (r.disappear.max(1) as f32 - 120.0)) / 120.0).clamp(0.02, 1.0);
            return Some((-1.5 * (1.0 - k), 0.0, k));
        }
        let boss = self.characters.iter().find(|b| b.pack == c.pack && b.slot == CLOUD_SLOT && b.cloud.is_some())?;
        let clip = boss.cloud.as_ref()?.clip;
        let t = self.ticks as f32 / 60.0;
        let p = clip.progress();
        let bob = 0.35 * (t * 1.7).sin();
        let pi = std::f32::consts::PI;
        Some(match clip.id {
            5 => (bob + 0.5 * (p * pi).sin(), 0.0, 1.0 + 0.08 * (p * 4.0 * pi).sin()),
            6 => (bob, -0.25 * p, 1.0 + 0.15 * p),
            7 => (bob, 0.3, 1.15 - 0.2 * p),
            22 => (bob, 0.15 * (1.0 - p), 0.95 + 0.05 * p),
            8 => (bob + 0.05 * (t * 40.0).sin(), -0.1, 1.05),
            10 => (bob, -0.1 * (1.0 - p), 1.0),
            11 => (bob - 0.3 * p, 0.0, 1.0 - 0.1 * p),
            12 => (bob, 0.0, 1.0 + 0.12 * (t * 25.0).sin().abs()),
            13 => (bob, 0.0, 0.9 + 0.1 * p),
            14 => (bob, 0.2 * (p * pi).sin(), 1.0 + 0.1 * (p * pi).sin()),
            15 | 16 => (bob + 0.4 * (p * pi).sin(), -0.45 * (p * pi).sin(), 0.95),
            23 => (bob, -0.2, 0.92 + 0.03 * (t * 12.0).sin()),
            _ => (bob, 0.0, 1.0),
        })
    }

    /// Developer menu: spawn the Cloud Beast in this world around the hero (removing one already
    /// here, and clearing a defeat).
    /// Developer menu, after the warp to the Cloud Beast fight: the test spawn on, its defeat
    /// mark cleared, and the hero placed in the arena at the spawn.
    pub fn dev_cloud_fight(&mut self) {
        if self.cloud_rows.is_none() || self.cloud_words.is_none() {
            return;
        }
        self.cloud_on = true;
        self.cloud_place_hero = true;
        self.cloud_defeated.remove(&self.world.pack);
    }

    /// Developer menu: make the flat test stage (`World::test_stage`, this build's own world)
    /// current in place of the loaded one, dropping that world, its characters, scenes and
    /// threads, and put the hero at the stage's origin facing +Z.
    pub fn enter_test_stage(&mut self) {
        self.world = World::test_stage(STAGE_PACK);
        self.other = None;
        self.kinds.insert(STAGE_PACK, Kind::Exterior);
        self.state.reset_levels();
        self.threads.clear();
        self.characters.clear();
        self.drop_camera_play();
        // A fade or letterbox the dropped world's scene had started would never be undone.
        self.fade = Default::default();
        self.bars = Default::default();
        // The stage has no region of its own to load.
        self.slot_regions.insert((STAGE_PACK, EMBEDDED_SLOT), STAGE_PACK);
        self.arrive(([0.0; 3], [0.0, 0.0, 1.0]));
        self.activated();
    }

    /// Whether the current world is the developer test stage.
    pub fn on_test_stage(&self) -> bool {
        self.world.flat.is_some()
    }

    /// The progress flags a boss fight starts with: its start index's stage rows (table 11, in
    /// stage order), then its walkthrough flags.
    pub fn boss_flags(&self, b: &BossStart) -> Vec<u16> {
        let mut f: Vec<u16> = START_STAGES[b.start.min(8)].iter().flat_map(|&s| self.design.stage_flags(s)).collect();
        f.extend(b.flags);
        f
    }

    /// After the boss world's load: the stage item grants (table 12; item 135 adds Neopoints, 136
    /// to 139 add 5, 10, 25 and 100), the extra gear, the equipment, full health and magic, the
    /// hero, the placement and, for the Cloud Beast, the test spawn.
    pub fn boss_setup(&mut self, b: &BossStart) {
        let grants: Vec<(u16, u32)> = START_STAGES[b.start.min(8)].iter().flat_map(|&s| self.design.stage_items(s)).collect();
        for (item, n) in grants {
            match item {
                135 => self.state.neopoints += n as i32,
                136 => self.state.neopoints += 5,
                137 => self.state.neopoints += 10,
                138 => self.state.neopoints += 25,
                139 => self.state.neopoints += 100,
                _ => self.state.give(item, n),
            }
        }
        for &id in b.items {
            if !self.state.items.get(&id).is_some_and(|&n| n > 0) {
                self.give(id);
            }
        }
        self.equipment_step();
        self.weapon = self.equipment.entries[0][self.hero_index.min(1)];
        if b.roberta {
            self.trace_select_hero(1);
        }
        (self.hero_hp, self.hero_mp) = (self.hero_max, self.hero_mp_max);
        if let Place::Trigger(k) = b.place {
            let at = self.world.all_volumes.iter().find(|v| v.index == k).map(|v| match v.shape {
                neopets_sim::trigger::Shape::Sphere { centre, .. } => centre,
                neopets_sim::trigger::Shape::Cylinder { base, axis, length, .. } => add(base, scale(axis, 0.5 * length)),
                neopets_sim::trigger::Shape::Rectangle { origin, axes, extents } => add(origin, add(add(scale(axes[0], 0.5 * extents[0]), scale(axes[1], 0.5 * extents[1])), scale(axes[2], 0.5 * extents[2]))),
            });
            if let Some(p) = at {
                let y = self.world.ground_at(p[0], p[2], p[1] + 8.0).unwrap_or(p[1]);
                let facing = self.loco.facing;
                self.arrive(([p[0], y, p[2]], facing));
            }
        }
        if b.cloud {
            self.enter_test_stage();
            self.dev_cloud_fight();
        }
        self.last_boss = format!("developer menu: boss fight {} in {} (start index {}, {} flags)", b.name, b.pack, b.start, self.state.flags.len());
    }

    pub fn dev_cloud_beast(&mut self) -> String {
        if self.cloud_rows.is_none() || self.cloud_words.is_none() {
            return "The Cloud Beast's monster rows are not on this disc".into();
        }
        let pack = self.world.pack;
        self.cloud_on = true;
        self.cloud_defeated.remove(&pack);
        self.characters.retain(|c| !(c.pack == pack && c.slot == CLOUD_SLOT));
        let tags = (101..=106).all(|k| self.world.tags.contains(&k));
        if tags { "Cloud Beast spawned on this arena's points 101 to 106".into() } else { "Cloud Beast spawned on a ring around the hero".into() }
    }

    /// SPEC-0040 Behavior 23A, the hero's head look: on a tick whose main animation has the
    /// head-look bit, the target is the talk partner's or the locked target's look point, else
    /// the nearest live character in front (dot above 0.5) closer than the hero's range (12.0
    /// Tor, 25.0 Roberta) raised 1.5, else 5.0 ahead of the head (the animation-chosen side and
    /// reset targets are not built; Medium); the controller steps the yaw and pitch and the
    /// look point turns head joint 6. Without the bit the head keeps its animated pose.
    fn head_look_step(&mut self) {
        self.hero.set_look(None);
        // NEOPETS_TRACE_LOOK: the main id and its head-look bit once a second (trace only).
        if std::env::var_os("NEOPETS_TRACE_LOOK").is_some() && self.ticks % 60 == 0 {
            println!("LOOK tick {} action {} allowed {} state {} yaw {:.2} pitch {:.2}", self.ticks, self.hero.action(), self.hero.head_look_allowed(), self.loco.state, self.head_look.yaw, self.head_look.pitch);
        }
        // NEOPETS_NO_LOOK: no head look (comparison traces only).
        if !self.hero.head_look_allowed() || std::env::var_os("NEOPETS_NO_LOOK").is_some() {
            return;
        }
        let place = placement(self.loco.position(), self.loco.facing);
        let Some(head) = self.hero.bone_frames().get(crate::character::HEAD_JOINT).map(|f| place.apply(f.t)) else { return };
        let (side, fwd) = (place.cols[0], place.cols[2]);
        let pack = self.world.pack;
        let live = |c: &&Character| c.pack == pack && !c.inactive && c.dying.is_none();
        let partner = self.talk_partner.as_ref().filter(|_| self.dialogue_open).and_then(|n| self.characters.iter().filter(live).find(|c| &c.name == n));
        let locked = self.lock.and_then(|id| self.characters.iter().filter(live).find(|c| c.id == id));
        let range = if self.hero_index == 0 { 12.0 } else { 25.0 };
        let ahead = || {
            self.characters
                .iter()
                .filter(live)
                .filter(|c| c.row.is_some() && c.hp > 0)
                .map(|c| (c, sub_v(c.position, head)))
                .map(|(c, d)| (c, d, (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()))
                .filter(|&(_, d, l)| l < range && l > 1e-3 && (d[0] * fwd[0] + d[1] * fwd[1] + d[2] * fwd[2]) / l > 0.5)
                .min_by(|a, b| a.2.total_cmp(&b.2))
                .map(|(c, ..)| add(c.position, [0.0, 1.5, 0.0]))
        };
        let target = partner.or(locked).and_then(|c| self.aim_point(c.id)).or_else(ahead).unwrap_or(add(head, scale(fwd, 5.0)));
        // NEOPETS_TRACE_LOOK_SIDE: the target 5 units to the side and 2 up (trace only).
        let target = if std::env::var_os("NEOPETS_TRACE_LOOK_SIDE").is_some() { add(add(head, scale(side, 5.0)), [0.0, 2.0, 0.0]) } else { target };
        let point = self.head_look.step(head, target, fwd, side, false);
        if std::env::var_os("NEOPETS_TRACE_LOOK").is_some() && self.ticks % 60 == 0 {
            println!("LOOK head {head:?} hero {:?} target {target:?} fwd {fwd:?} side {side:?} point {point:?}", self.loco.position());
        }
        let v = sub_v(point, self.loco.position());
        let local = [v[0] * side[0] + v[2] * side[2], v[1], v[0] * fwd[0] + v[2] * fwd[2]];
        self.hero.set_look(Some(local));
        if std::env::var_os("NEOPETS_TRACE_LOOK").is_some() && self.ticks % 60 == 0 {
            if let Some(f) = self.hero.bone_frames().get(crate::character::HEAD_JOINT) {
                let z = place.rotate(f.cols[2]);
                let l = (z[0] * z[0] + z[1] * z[1] + z[2] * z[2]).sqrt();
                let d = sub_v(point, place.apply(f.t));
                let ld = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                println!("LOOK head forward . look direction {:.3}", (z[0] * d[0] + z[1] * d[1] + z[2] * d[2]) / (l * ld));
            }
        }
    }

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
        // SPEC-0038 Behavior 44 defences: the hero's swing class (attack sub-state +0xF90 by
        // hero) and the live hero shots as (position, velocity per tick).
        let swing_sub = match self.hero_index {
            0 => self.attack.as_ref().map(|a| a.sub),
            _ => self.r_attack.as_ref().map(|a| a.sub),
        };
        let hero_swing = swing_sub.and_then(|s| enemy::swing_class(self.hero_index, s));
        let hero_recover = self.hero.frame_bits() & 0x800 != 0;
        let shot_paths: Vec<([f32; 3], [f32; 3])> = self.hero_shots.iter().map(|s| (s.position, scale(s.dir, s.speed / 60.0))).collect();
        let guarding = self.loco.state == hero_states::GUARD;
        // SPEC-0007 Hit reaction and invulnerability / SPEC-0036 Hittable predicate: no common
        // enemy picks an attack (and its cooldowns wait) while the hero is not hittable: in
        // states 16, 17, 25, 33, 36, 48, 50 to 55, 57, 59 and 60, in Roberta's leap and landing
        // sub-states (8 and 13), under the script hold, or while a dialogue box is up.
        let roberta_leap = self.hero_index == 1 && self.r_attack.as_ref().is_some_and(|a| matches!(a.sub, 8 | 13));
        let attackable = !matches!(self.loco.state, 16 | 17 | 25 | 33 | 36 | 48 | 50..=55 | 57 | 59 | 60) && !roberta_leap && !self.held && !self.dialogue_open;
        // SPEC-0038 Behavior 45: the attack manager's frame, before the enemies read their wait
        // flags; the capacity from the selected hero's weapon and body item.
        let h = self.hero_index.min(1);
        let capacity = neopets_sim::attackmgr::capacity(h, self.equipment.entries[0][h], self.equipment.entries[2][h]);
        let members: HashMap<u64, neopets_sim::attackmgr::Member> = self
            .characters
            .iter()
            .filter_map(|c| {
                let e = c.enemy.as_ref()?;
                let d = sub_v(c.position, hero);
                let weight = c.row.map_or(0, |_| e.p.weight as i16 as i32);
                Some((c.id, neopets_sim::attackmgr::Member { agro: e.state == 19 && !e.is_dead(), dist: (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(), movable: e.movable(), weight }))
            })
            .collect();
        self.attack_mgr.run(capacity, &|id| members.get(&id).copied());
        for c in self.characters.iter_mut() {
            if let Some(e) = c.enemy.as_mut().filter(|e| e.joined) {
                e.joined = false;
                self.attack_mgr.grant(c.id);
            }
            // A group member entering agro posts its group's broadcast.
            if let Some(e) = c.enemy.as_mut().filter(|e| e.alarmed) {
                e.alarmed = false;
                if let Some(g) = e.spawn.alarm {
                    *self.alarms.entry((c.pack, g)).or_default() += 1;
                }
            }
        }
        // The notice gate's inputs: the groups of the Display-in-Default 0 enemies in state 19 or 2.
        let roused: Vec<(u32, Option<u8>)> = self.characters.iter().filter_map(|c| c.enemy.as_ref().filter(|e| !e.p.display_default && matches!(e.state, 19 | 2) && !e.is_dead()).map(|e| (c.pack, e.spawn.alarm))).collect();
        let world = &self.world;
        let mgr = &self.attack_mgr;
        // SPEC-0038 Behavior 35's clear-area test reads the live pool's positions.
        let pool: Vec<(u64, [f32; 3])> = self.characters.iter().filter(|c| c.dying.is_none() && c.hp > 0 && !c.inactive).map(|c| (c.id, c.position)).collect();
        // SPEC-0038 Behaviors 9 and 39: the current world's items for the map search, and the
        // items enemies consume this tick.
        let world_items: Vec<(usize, u16, [f32; 3])> = self.pools.get(&world.pack).map(|pool| pool.slots.iter().enumerate().filter_map(|(i, s)| s.as_ref().map(|it| (i, it.id, it.position))).collect()).unwrap_or_default();
        let mut consumed = Vec::new();
        let mut strikes = Vec::new();
        let mut dots = Vec::new();
        for c in self.characters.iter_mut().filter(|c| c.pack == world.pack && !c.held && !c.inactive) {
            let Character { id, name, enemy: Some(e), walker: Some(wk), anim, row: Some(row), weapons, position, facing, root_step, hp, .. } = c else { continue };
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
            // The navmesh pick: the first mesh holding the enemy, of kind 2 (flying) when the
            // row can fly, else 1 (SPEC-0038 Behavior 6); a flyer with no flying mesh under it
            // takes a ground mesh. A grounding or the retarget step picks again.
            let fly_kind = if row.can_fly != 0 { 2 } else { 1 };
            if e.take_repick() || wk.mesh.and_then(|i| world.navmeshes.get(i)).is_none_or(|m| m.locate_full(p).is_none()) {
                let find = |kind: u32| world.navmeshes.iter().position(|m| m.kind == kind && m.locate_full(p).is_some());
                let pick = find(fly_kind).or_else(|| find(1));
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
            // The missile defence: each live hero shot's path from its position to position +
            // velocity x the defend time (ticks) against the enemy's body capsule.
            v.hero_swing = hero_swing;
            v.hero_recover = hero_recover;
            v.l1_held = self.l1_held;
            v.waits = mgr.waits(*id);
            let own = e.spawn.alarm;
            v.alarm_serial = own.and_then(|g| self.alarms.get(&(world.pack, g)).copied()).unwrap_or(0);
            v.alarm_block = own.is_some() && roused.iter().any(|&(pk, g)| pk == world.pack && g.is_some_and(|g| Some(g) != own));
            let near = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() < 5.0;
            v.clear = !near(p, hero) && !pool.iter().any(|&(o, q)| o != *id && near(p, q));
            let locate = |q: [f32; 3]| mesh.is_none_or(|m| m.locate_full(q).is_some());
            v.locate = Some(&locate);
            v.items = &world_items;
            v.hero_shot_live = !shot_paths.is_empty();
            let body = Cylinder::from_base(p, [0.0, 1.0, 0.0], row.radius, row.height);
            let look = e.p.defence.hold.max(0) as f32;
            v.missile_threat = shot_paths.iter().any(|&(s, vel)| neopets_sim::combat::segment_meets(s, add(s, scale(vel, look)), &body));
            let outer_before = e.outer;
            let before = (e.outer, e.inner, e.seq, e.selected);
            let act = e.tick(&v, &mut self.rng);
            // NEOPETS_TRACE_AGRO: each change of an enemy's agro sub-states (diagnostic).
            if std::env::var("NEOPETS_TRACE_AGRO").as_deref() == Ok("clip") && e.outer == 2 && self.ticks % 5 == 0 {
                println!("      agro clip {name} tick {}: seq {} clip {} progress {:.3} finished {} window {:?}", self.ticks, e.seq, v.clip.action, v.clip.progress, v.clip.finished, anim.layer0_window());
            }
            if before != (e.outer, e.inner, e.seq, e.selected) && std::env::var_os("NEOPETS_TRACE_AGRO").is_some() {
                let d = sub_v(hero, p);
                println!("      agro {name} tick {}: outer {} inner {} seq {} slot {:?} d {:.1} attackable {} recovery {}", self.ticks, e.outer, e.inner, e.seq, e.selected, (d[0] * d[0] + d[2] * d[2]).sqrt(), v.hero_attackable, e.recovery);
            }
            // The flyer's goal height (SPEC-0040 Behavior 21), with the ground under it.
            let fly_y = e.airborne().then(|| e.fly_target(&v, world.ground_at(p[0], p[2], p[1] + 8.0)));
            // NEOPETS_TRACE_DEFENCE: a defence starting (outers 5 to 8).
            if e.outer != outer_before && (5..=8).contains(&e.outer) && std::env::var_os("NEOPETS_TRACE_DEFENCE").is_some() {
                println!("      defence: {name} enters outer {} (swing class {:?}) on tick {}", e.outer, v.hero_swing, self.ticks);
            }
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
                if let Some(mut setup) = neopets_sim::projectile::spawn(&slot.missile, e.p.element, *id, launch, hero, (out_of_cone, self.attack_mgr.waits(*id)), &mut self.rng) {
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
            let mut step = if act.stop { [0.0; 3] } else { *root_step };
            let ground = &world.ground;
            // SPEC-0040 Behavior 14D item 7: a flyer adds the row speed along its class axis and
            // its height keeper moves y 0.1 a tick toward the goal height (set at once within
            // 0.1); a can-fly 1 flyer has no gravity and no ground projection.
            let airborne = e.airborne();
            wk.float = airborne && row.can_fly == 1;
            if airborne {
                if let Some(f) = act.fly {
                    step = add(step, f);
                }
                if let Some(target) = fly_y {
                    let dy = target - wk.position()[1];
                    wk.body.position[1] += if dy.abs() > 0.1 { 0.1 * dy.signum() } else { dy };
                }
            }
            if let Some(p) = act.snap {
                wk.body.position = p;
                step = [0.0; 3];
            }
            wk.motion_tick(step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
            *position = wk.position();
            *facing = wk.facing;
            if let Some(s) = act.strike {
                let m = e.p.slots[s.slot].missile;
                strikes.push((*id, s, wk.facing, (m.status, m.status_chance, m.status_range), e.strike_scale(), e.status(11)));
            }
            // SPEC-0038 Behavior 35: the rise sets health to its maximum.
            if act.heal {
                *hp = row.hit_points as i32;
            }
            for amount in act.status_hits.into_iter().flatten() {
                dots.push((*id, amount));
            }
            consumed.extend(act.consume);
        }
        if let Some(pool) = self.pools.get_mut(&self.world.pack) {
            for i in consumed {
                pool.destroy(i);
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
                poisoned: self.statuses.active(neopets_sim::status::POISON),
                owner_exists: &owner_exists,
                owner_body: &owner_body,
                ground: &ground,
                wall: &wall,
            };
            self.shots.tick(&env)
        };
        self.thorn_step(hero);
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
                            if c.hp <= 0 && !e.is_dead() {
                                c.hp = 0;
                                e.die(&mut self.rng);
                                self.payouts.push((c.pack, c.position, c.drop, std::mem::take(&mut c.carried_np)));
                            }
                            self.last_hit = format!("reflected shot hit {} for {applied} (hp {})", c.name, c.hp);
                        }
                    }
                }
                neopets_sim::projectile::Event::Status { code, chance, seconds } => {
                    // SPEC-0029 Behavior 9: int[0, 100] at most the chance; no stun while stunned.
                    let r = self.rng.range(0, 100);
                    if r <= chance && !(code == 0 && self.statuses.active(neopets_sim::status::STUN)) {
                        self.hero_status(code, seconds);
                    }
                }
                neopets_sim::projectile::Event::Summon { key, at } => {
                    // SPEC-0029 Behavior 31 source 3: a world-level request at the landing point,
                    // facing the hero (the shot's forward axis is not kept).
                    let to = [hero[0] - at[0], 0.0, hero[2] - at[2]];
                    let pack = self.world.pack;
                    self.summon(pack, None, key as i16, at, unit(to), None, false);
                }
                neopets_sim::projectile::Event::MoteEater => {
                    // The drain starts when none is active: its body effect 464 above the hero.
                    if self.drain.is_none() {
                        let d = neopets_sim::sisters::Drain::start(&self.drain_words, &mut self.rng);
                        let at = add(self.loco.position(), [0.0, self.drain_words.body_height, 0.0]);
                        let body = self.effects.add_object(neopets_sim::effect::EffectObject::new(&self.effect_table, 464, at, neopets_sim::effect::IDENTITY));
                        self.drain = Some((d, body, Vec::new()));
                    }
                }
                neopets_sim::projectile::Event::Leach => {
                    // SPEC-0011 Leach: effect 4039 at the hero's model point, then 50 MP drained
                    // (not below 0; the MP-empty rule is not built).
                    let table = self.effect_table.clone();
                    let p = add(self.loco.position(), [0.0, 1.5, 0.0]);
                    self.effects.one_shot(&table, 4039, p, neopets_sim::effect::IDENTITY, &mut self.rng);
                    self.hero_mp = (self.hero_mp - self.sister_words.leach_drain).max(0);
                }
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
                self.hero_status(neopets_sim::status::POISON as i32, -1);
            }
            let died = self.hero_hit(damage, s.knockback, s.blocked, f, format!("slot {}{}", s.slot, if s.blocked { ", blocked" } else { "" }));
            // SPEC-0029 Behavior 11a: a landed hit of a slot with the steal bit (0x04) and
            // selector 0 may take Neopoints, dropped at once as coins at the hero (the Thief, the
            // Rogue and the Thug). A blocked hit is taken as failing the impact gate (Open
            // question 157).
            let slot = self.characters.iter().find(|c| c.id == at).and_then(|c| c.enemy.as_ref()).map(|e| e.p.slots[s.slot]);
            if let Some(sl) = slot.filter(|sl| !s.blocked && sl.flags & 0x04 != 0 && sl.steal_selector == 0) {
                if let Some(taken) = neopets_sim::items::steal_amount(sl.steal_chance, sl.steal_min, sl.steal_max, self.state.neopoints, &mut self.rng).filter(|&t| t > 0) {
                    self.state.neopoints -= taken;
                    let pack = self.world.pack;
                    let p = self.loco.position();
                    let spawn = |id: u16| neopets_sim::items::Spawn { id, point: [p[0], p[1] + 1.0, p[2]], placed: false, charm: false, s0: 0.25, s1: 1.0, delay: 0.0, larg: -1.0, direction: None, flag: 0, map: 0, price: 0 };
                    for id in neopets_sim::items::coin_split(taken, &self.coins.0, &self.coins.1) {
                        self.spawn_item(pack, spawn(id));
                    }
                    self.last_fx = format!("stole {taken} Neopoints (slot {}) on tick {}", s.slot, self.ticks);
                }
            }
            // SPEC-0029 Behavior 11b: selectors 1 to 3 take one held item of the selector's pair
            // instead; the enemy carries it for 180 ticks and its carried Neopoints grow by the
            // item's catalogue price. Selectors outside 0 to 3 end with no effect.
            if let Some(sl) = slot.filter(|sl| !s.blocked && sl.flags & 0x04 != 0 && sl.steal_selector != 0) {
                let items = &self.state.items;
                if let Some(id) = neopets_sim::items::steal_item(sl.steal_selector, |id| items.get(&id).is_some_and(|&n| n > 0), &mut self.rng) {
                    let left = self.state.items.get(&id).copied().unwrap_or(0);
                    if left > 1 {
                        self.state.items.insert(id, left - 1);
                    } else {
                        self.state.items.remove(&id);
                    }
                    let price = self.design.catalogue_price(id).unwrap_or(0);
                    if let Some(c) = self.characters.iter_mut().find(|c| c.id == at) {
                        c.carried_item = Some((id, 180));
                        c.carried_np += price;
                    }
                    self.last_fx = format!("stole item {id} (price {price}, slot {}) on tick {}", s.slot, self.ticks);
                }
            }
            // SPEC-0038 Behavior 13 strike test: the status roll int[0, 100] strictly below the
            // chance, for int[int(range max), int(range min)] / 60 whole seconds.
            if status.0 != -1 && !s.blocked {
                let r = self.rng.range(0, 100);
                if (r as f32) < status.1 {
                    let ticks = self.rng.range(status.2 .1 as i32, status.2 .0 as i32);
                    self.hero_status(status.0, ticks / 60);
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
        } else if self.book_prompt {
            Some(31)
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
        // SPEC-0021 Use-request gate item 2: a registered use hook diverts the use (the item is
        // not used or removed) and starts the hook thread.
        if let Some(actions) = level::on_hero_use_item(self.world.pack, &mut self.state) {
            self.last_pickup = format!("use of item {id} diverted by the use hook of {} on tick {}", self.world.pack, self.ticks);
            self.apply(&actions);
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
            // SPEC-0006 Kind 8 +0x21: a snapping placement spawns at the ground (probed from its
            // y + 4.0; 0.0 with no ground) + 0.5.
            let mut point = p.position;
            if p.snap {
                let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
                point[1] = w.and_then(|w| w.ground_at(point[0], point[2], point[1] + 4.0)).unwrap_or(0.0) + 0.5;
            }
            let spawn = neopets_sim::items::Spawn {
                id,
                point,
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

    /// SPEC-0021 Drop routes 3, a scripted add of a placement: the luck gate, then for keys 0 or
    /// more the vetoes and the item itself; a negative key takes the group's item roll only (no
    /// money). The point follows the region-load rule (byte +0x21: the ground + 0.5); snap then
    /// puts it on the ground (SPEC-0030 note on rows 201 and 364); placed items never expire.
    fn scripted_add(&mut self, pack: u32, p: neopets_data::region::ItemPoint, snap: bool, placed: bool) {
        if p.luck != 0 {
            let r = self.rng.range(0, 100);
            if !neopets_sim::items::luck_passes(self.luck, r) {
                return;
            }
        }
        let id = if p.key >= 0 {
            if self.vetoed(p.key as u16, pack) {
                return;
            }
            p.key as u16
        } else {
            let group = usize::try_from(p.group).ok().and_then(|g| self.items.groups.get(g)).map(|g| neopets_sim::items::Group { chance: g.chance as i32, min: g.min as i32, max: g.max as i32, entries: g.entries.clone() });
            let rng = std::cell::RefCell::new(&mut self.rng);
            let mut index = |n: u32| rng.borrow_mut().index(n);
            match neopets_sim::items::item_roll(group.as_ref(), &self.items.sub_tables, &mut index) {
                neopets_sim::items::Drop::Item(id) => id,
                _ => return,
            }
        };
        let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
        let mut point = p.position;
        if p.snap {
            point[1] = w.and_then(|w| w.ground_at(point[0], point[2], point[1] + 4.0)).unwrap_or(0.0) + 0.5;
        }
        if snap {
            point[1] = w.and_then(|w| w.ground_at(point[0], point[2], point[1] + 4.0)).unwrap_or(point[1]);
        }
        let spawn = neopets_sim::items::Spawn {
            id,
            point,
            placed,
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
        let line = format!("item {id} activated from placement {:?} on tick {}", p.position, self.ticks);
        // Two activations on one tick are both kept in the trace line.
        self.last_fx = if self.last_fx.contains("activated from") && self.last_fx.ends_with(&format!("tick {}", self.ticks)) { format!("{}; {line}", self.last_fx) } else { line };
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

    /// SPEC-0021 Important-item give: the important ids among a pool's slot-reset gives go to
    /// the party, count 1 each (capped at the carry limit).
    fn reset_gives(&mut self, pack: u32) {
        // SPEC-0021 Coins: each coin ground bounce plays cue 0xA7 at the coin.
        let bounces = self.pools.get_mut(&pack).map(|p| std::mem::take(&mut p.bounces)).unwrap_or_default();
        for at in bounces {
            self.sound.play(neopets_sim::sound::MAIN, 0xA7, neopets_sim::sound::Play { emitter: Some(at), ..Default::default() });
        }
        let gives = self.pools.get_mut(&pack).map(|p| std::mem::take(&mut p.gives)).unwrap_or_default();
        for id in gives {
            if self.design.item_important(id) {
                self.state.give(id, 1);
            }
        }
    }

    /// The world items' per-tick step for the resident worlds' pools, the enemy payouts due,
    /// and the hero's pickup in the current world (SPEC-0021 Pickup, Auto-collect, Coin pickups).
    fn items_step(&mut self) {
        for (pack, at, drop, carried) in std::mem::take(&mut self.payouts) {
            self.pay_out(pack, at, drop);
            // Part D: the carried Neopoints split into coins at the enemy, with no spacing.
            if carried > 0 {
                let spawn = |id: u16| neopets_sim::items::Spawn { id, point: at, placed: false, charm: false, s0: 0.25, s1: 1.0, delay: 0.0, larg: -1.0, direction: None, flag: 0, map: 0, price: 0 };
                for id in neopets_sim::items::coin_split(carried, &self.coins.0, &self.coins.1) {
                    self.spawn_item(pack, spawn(id));
                }
            }
        }
        // SPEC-0029 Behavior 11b: a stolen item's carry timer runs down one tick at a time; below
        // 1 the carried flag clears.
        for c in &mut self.characters {
            if let Some((_, t)) = c.carried_item.as_mut() {
                *t = t.saturating_sub(1);
                if *t < 1 {
                    c.carried_item = None;
                }
            }
        }
        let Some(coins) = self.coins.2 else { return };
        let packs: Vec<u32> = self.pools.keys().copied().collect();
        for pack in packs {
            let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
            let ground = |x: f32, z: f32, c: f32| world.and_then(|w| w.ground_at(x, z, c));
            let rng = &mut self.rng;
            let mut draw = |lo: f32, hi: f32| rng.frange(lo, hi);
            let expired = self.pools.get_mut(&pack).map(|pool| pool.tick(&coins, &ground, &mut draw)).unwrap_or_default();
            // SPEC-0021 Important-item give: an important dropped item that fades out goes to
            // the party (count 1).
            for id in expired {
                if self.design.item_important(id) {
                    self.state.give(id, 1);
                    self.last_pickup = format!("important item {id} given at its expiry on tick {}", self.ticks);
                }
            }
            // The slot resets since the last tick (an expired drop's slot reused).
            self.reset_gives(pack);
        }
        let p = self.loco.position();
        let pack = self.world.pack;
        let base = self.items.get(146).map_or(1.5, |i| i.pickup_base);
        let Some(i) = self.pools.get(&pack).and_then(|pool| pool.collectable(p, base, contact::HERO_RADIUS, &|id| self.design.item_important(id))) else { return };
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
        // The take's slot reset gives an important dropped item first (SPEC-0021 Important-item
        // give); the pickup's own add follows, so such a pickup adds 2 (capped).
        self.reset_gives(pack);
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
            // The normal add, capped at the carry limit.
            self.state.give(it.id, count);
            if matches!(it.id, 19 | 20 | 151 | 154 | 174) {
                self.once.insert((it.id, pack));
            }
            self.last_pickup = format!("picked up item {} (held {}) on tick {}", it.id, self.state.items[&it.id], self.ticks);
            let hook = level::on_pickup(pack, it.id, &mut self.state);
            self.apply(&hook);
            // The world-item pickup starts the showcase with quiet 0.
            self.start_showcase(it.id, pack);
        }
    }

    /// A common-enemy strike on the hero: SPEC-0036 Enemy attack damage to a hero (no guard,
    /// so no block; no armour, Motes or petpet; status scale 1), Taking a hit (the dialogue box
    /// discards hits; a dead hero takes none; Tor's charge sub-states 1 and 10 are busy), the
    /// deduct and the default death, and the knockback. Returns whether the hero died.
    fn hero_hit(&mut self, damage: f32, knockback: f32, blocked: bool, attacker_facing: [f32; 3], what: String) -> bool {
        if self.dialogue_open || self.loco.state == hero_states::DEAD || self.cheats.god {
            return false;
        }
        let damage = damage.max(0.0).trunc() as i32;
        // SPEC-0040 Behavior 29 item 2: a hit (not a poison tick, attacker -2) with the weapon
        // sheathed draws Tor's weapon when he has one and raises the hero's shield.
        if !self.weapon_drawn && what != "poison" {
            let tor_armed = self.equipment.entries[0][0].is_some();
            self.set_weapon_drawn(tor_armed);
            self.shield_raised = self.shield.is_some();
        }
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
        self.hero_flash = HIT_FLASH;
        self.last_hurt = format!("took {damage} ({what}) on tick {}, hp {}", self.ticks, self.hero_hp.max(0));
        // SPEC-0031 Hero impact: melee or projectile with no armour plays 170; the voice slot
        // plays hit big (1) on a fair draw of 0, else hit small (0).
        self.cue(170);
        let big = self.rng.next_u32() % 2 == 0;
        self.sound.play(neopets_sim::sound::VOICE, if big { 1 } else { 0 }, neopets_sim::sound::Play::default());
        if self.hero_hp <= 0 {
            self.hero_hp = 0;
            self.death_hook();
            return true;
        }
        if damage > 0 && knockback > 0.0 {
            let d = unit([9.0 * attacker_facing[0], 9.0 * attacker_facing[1] + 12.0, 9.0 * attacker_facing[2]]);
            self.attack = None;
            self.loco.knock(scale(d, knockback));
        }
        false
    }

    /// SPEC-0055 Death hook with the gate allowed (a death from damage or deadly ground): a hero
    /// not yet dead consults the current area's death callback, whose task actions start and
    /// whose flag 1 skips the default death.
    fn death_hook(&mut self) {
        if self.loco.state != hero_states::DEAD {
            if let Some((skip, actions)) = level::on_hero_death(self.world.pack, self.hero_index, &mut self.state) {
                self.last_fx = format!("death callback of {} on tick {} (skip {skip})", self.world.pack, self.ticks);
                self.apply(&actions);
                if skip {
                    return;
                }
            }
        }
        self.default_death();
    }

    /// SPEC-0020 per tick for Gnarfas (class 14) in the current world: the host's measurements
    /// (the hero's hit, the missile paths against the body cylinder, the bone-chain test of the
    /// frame's event bits, the tree points, the ground gap), the controller tick and its
    /// requests (the clip, the steering and travel, the hits and knockback on the hero, the
    /// level events, the phase 2 heal).
    fn gnarfas_step(&mut self) {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.gnarfas.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let world = &self.world;
            let locator = |k: usize| world.tags.iter().position(|&t| t as usize == k).and_then(|i| world.entries.get(i)).map(|e| e.0);
            let trees = neopets_sim::gnarfas::TREES.map(locator);
            let shots: Vec<([f32; 3], [f32; 3], f32)> = self.hero_shots.iter().map(|s| (s.position, scale(s.dir, s.speed), s.radius)).collect();
            // SPEC-0020: "the hero is down" is action state 16 only.
            let busy = self.loco.state == 16;
            let st = self.loco.state;
            let c = &mut self.characters[i];
            let body = c.row.map_or(1.0, |r| r.radius);
            // A missile path (its point to point + n x velocity) against the body cylinder (radius
            // r, 5.0 tall); the spec tests the bone cylinders.
            let near = |n: f32| shots.iter().any(|&(p, v, r)| {
                let end = add(p, scale(v, n));
                let (a, b) = (c.position, add(c.position, [0.0, 5.0, 0.0]));
                neopets_sim::gnarfas::segment_distance(p, end, a, b) <= body + r
            });
            let (near10, near120) = (near(10.0), near(120.0));
            let clip = c.anim.clip_state();
            let bits = c.anim.frame_bits();
            let to_boss = [c.position[0] - hero[0], 0.0, c.position[2] - hero[2]];
            let tl = to_boss[0].hypot(to_boss[2]).max(1e-6);
            let faces = (hf[0] * to_boss[0] + hf[2] * to_boss[2]) / tl > 0.5;
            let attacking = c.gnarfas.as_ref().is_some_and(|g| matches!(g.state, neopets_sim::gnarfas::State::Whip { .. } | neopets_sim::gnarfas::State::Rapid { .. } | neopets_sim::gnarfas::State::Power { .. }));
            let chain_hit = if attacking {
                let place = placement(c.position, c.facing);
                let frames = c.anim.bone_frames();
                neopets_sim::gnarfas::RUNS.iter().filter(|r| bits & r.0 != 0).find_map(|&(_, first, last)| {
                    (first..=last).find(|&b| frames.get(b).is_some_and(|f| {
                        let m = place.mul(f);
                        neopets_sim::gnarfas::bone_hits(m.t, unit(m.cols[1]), hero)
                    })).map(|_| last)
                })
            } else {
                None
            };
            let gap = world.ground_at(c.position[0], c.position[2], c.position[1] + 2.5).map_or(0.0, |g| c.position[1] - g);
            let v = neopets_sim::gnarfas::View {
                position: c.position,
                facing: c.facing,
                hero,
                hero_hp: self.hero_hp,
                hero_max: self.hero_max,
                hero_busy: busy,
                hero_blocks: st == 20 && faces,
                hero_down: matches!(st, 7..=11 | 0x2B..=0x2E),
                hero_attack_start: self.state.hero.fire,
                shot: c.pending_hit.is_none() && c.pending_shot.is_some(),
                hit: c.pending_hit.take().or(c.pending_shot.take()),
                missile_near: near10,
                missile_ahead: near120,
                chain_hit,
                frame_bits: bits,
                clip: (clip.action, clip.progress, clip.progress >= 0.99),
                hp: c.hp,
                max_hp: c.row.map_or(1, |r| r.hit_points as i32),
                level: &level,
                trees,
                ground_gap: gap,
            };
            let Some(g) = c.gnarfas.as_mut() else { continue };
            let before = g.state;
            let act = g.tick(&v, &self.gnarfas_words, &mut self.rng);
            let after = g.state;
            if let Some(a) = act.anim {
                if clip.action != a || (clip.progress >= 0.99 && std::mem::discriminant(&before) != std::mem::discriminant(&after)) {
                    c.anim.play(a);
                }
            }
            if let Some(rate) = act.face {
                let to = [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]];
                c.facing = neopets_sim::hero::steer(c.facing, to, rate);
            }
            match act.travel {
                neopets_sim::gnarfas::Move::Fly(vel) => {
                    c.position = add(c.position, vel);
                    if vel[0] != 0.0 || vel[2] != 0.0 {
                        c.facing = neopets_sim::hero::steer(c.facing, [vel[0], 0.0, vel[2]], 0.2);
                    }
                    if let Some(wk) = c.walker.as_mut() {
                        wk.body.position = c.position;
                        wk.facing = c.facing;
                    }
                }
                neopets_sim::gnarfas::Move::Still => {}
                neopets_sim::gnarfas::Move::Steer { mode, target, weight } => {
                    let m = [target[0] - c.position[0], 0.0, target[2] - c.position[2]];
                    if mode != 255 && (m[0] != 0.0 || m[2] != 0.0) {
                        // The shared steering rule: modes 5 and 6 face across the move.
                        let (dir, w) = match mode {
                            5 => {
                                let p = [m[2], 0.0, -m[0]];
                                let l = p[0].hypot(p[2]).max(1e-6);
                                (p, if (c.facing[0] * p[0] + c.facing[2] * p[2]) / l > 0.9 { 1.0 } else { 0.1 })
                            }
                            6 => {
                                let p = [-m[2], 0.0, m[0]];
                                let l = p[0].hypot(p[2]).max(1e-6);
                                (p, if (c.facing[0] * p[0] + c.facing[2] * p[2]) / l > 0.9 { 1.0 } else { 0.1 })
                            }
                            _ => (m, weight),
                        };
                        c.facing = neopets_sim::hero::steer(c.facing, dir, w);
                    }
                    let ground = &world.ground;
                    if let Some(wk) = c.walker.as_mut() {
                        wk.facing = c.facing;
                        wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                        c.position = wk.position();
                    }
                }
            }
            c.hp -= act.hp_loss;
            if before != after && std::mem::discriminant(&before) != std::mem::discriminant(&after) {
                self.last_boss = format!("Gnarfas {:?} -> {:?} (hp {}, strikes {}) on tick {}", before, after, c.hp, g.strikes, self.ticks);
            }
            let tip = act.spark.and_then(|b| c.anim.bone_frames().get(b).map(|f| placement(c.position, c.facing).mul(f).t));
            let from = c.position;
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            if act.heal_hero {
                self.hero_hp = self.hero_max;
            }
            let d = {
                let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                let l = dx.hypot(dz).max(1e-6);
                [dx / l, 0.0, dz / l]
            };
            if let Some(p) = tip {
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, 710, p, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            if let Some((damage, blocked, s)) = act.hero_damage {
                self.hero_hit(damage as f32, self.gnarfas_words.push * s, blocked, d, "Gnarfas".into());
            }
            if let Some(s) = act.knockback {
                let k = unit([9.0 * d[0], 12.0, 9.0 * d[2]]);
                self.attack = None;
                self.loco.knock(scale(k, self.gnarfas_words.push * s));
            }
        }
    }

    /// SPEC-0019 per tick for the Werelupe King (class 32) in the current world: the host's
    /// measurements (the hit, the reach test, the hand point, the ground), the controller tick and
    /// its requests (the clip, the steering and travel, the hits, the rings, the phase events and
    /// flag 273). The reach test's weapon cylinder (radius 2.5, length 7.0, from 1.5 behind the
    /// anchor) is taken from a chest-high anchor along the facing, since the anchor is not given;
    /// the hero's body records are taken as one 1.0 by 3.5 upright cylinder.
    /// SPEC-0014 per tick for the Ixi Chief (class 11) in the current world: the view (the hero's
    /// attack, guard and state; the missile contact of a hero shot's 120-tick path with the boss's
    /// body; the hits it took; the monster weapon tests of attach points 0 to 4 on the clip's
    /// event bits; the navigation check against its mesh; the level flags), then the controller's
    /// clip, facing, travel by its locomotion mode on the clip's root motion, hits, push, level
    /// flags, HP loss and hit flash (effect 222).
    fn ixi_step(&mut self) {
        use neopets_sim::combat::Cylinder;
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.ixi.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        if ids.is_empty() {
            return;
        }
        let hplace = placement(hero, hf);
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
        let st = self.loco.state;
        let attacking = self.attack.is_some() || self.r_attack.is_some();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let world = &self.world;
            let c = &mut self.characters[i];
            let (radius, height) = c.row.map_or((1.5, 3.5), |r| (r.radius, r.height.max(1.0)));
            let body = Cylinder { center: add(c.position, [0.0, 0.5 * height, 0.0]), axis: UP, radius, length: height };
            let missile_contact = self.hero_shots.iter().any(|s| {
                let end = add(s.position, scale(s.dir, s.speed * 120.0));
                neopets_sim::combat::segment_meets(s.position, end, &Cylinder { radius: radius + s.radius, ..body })
            });
            let place = placement(c.position, c.facing);
            let frames = c.anim.bone_frames();
            let bits = c.anim.frame_bits();
            let weapons: [bool; 5] = std::array::from_fn(|n| {
                if bits & (1 << n) == 0 {
                    return false;
                }
                let Some(&(Some(b), angles, offset)) = c.anim.bones().attach.get(n) else { return false };
                let Some(bw) = frames.get(b) else { return false };
                let w = place.mul(bw).mul(&attach_rotation(offset, angles));
                let (r, length) = c.weapons[n];
                let weapon = Cylinder { center: w.t, axis: unit(w.cols[2]), radius: r, length };
                caps.iter().any(|cap| neopets_sim::combat::overlap(&weapon, cap))
            });
            // The navmesh pick (SPEC-0029 Behavior 2): the first ground mesh holding the boss,
            // again whenever the stored one no longer holds it.
            if let Some(wk) = c.walker.as_mut() {
                let p = wk.position();
                if wk.mesh.and_then(|i| world.navmeshes.get(i)).is_none_or(|m| m.locate_full(p).is_none()) {
                    let pick = world.navmeshes.iter().position(|m| m.kind == 1 && m.locate_full(p).is_some());
                    if pick.is_some() {
                        wk.assign_mesh(pick, &world.navmeshes);
                    }
                }
            }
            let mesh = c.walker.as_ref().and_then(|wk| wk.mesh).and_then(|m| world.navmeshes.get(m));
            let valid = |p: [f32; 3]| mesh.is_none_or(|m| m.locate_full(p).is_some());
            let clip = c.anim.clip_state();
            let v = neopets_sim::ixi::View {
                position: c.position,
                facing: c.facing,
                hero,
                hero_facing: hf,
                hero_attacking: attacking,
                hero_guard: st == hero_states::GUARD,
                hero_state: st,
                hp: c.hp,
                max_hp: c.row.map_or(1, |r| r.hit_points as i32),
                missile_contact,
                melee_hit: c.pending_hit.take(),
                shot_hit: c.pending_shot.take(),
                weapons,
                frame_bits: bits,
                clip: (clip.action, clip.progress >= 0.99),
                blocked: c.walker.as_ref().is_some_and(|wk| wk.blocked),
                hero_knocked: st == hero_states::LONG_FALL,
                valid: &valid,
                level: &level,
            };
            let Some(k) = c.ixi.as_mut() else { continue };
            let before = k.state;
            let act = k.tick(&v, &self.ixi_words, &mut self.rng);
            let after = k.state;
            if let Some(a) = act.anim {
                if clip.action != a || (clip.progress >= 0.99 && std::mem::discriminant(&before) != std::mem::discriminant(&after)) {
                    c.anim.play(a);
                }
            }
            if let Some(rate) = act.face {
                let to = [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]];
                c.facing = neopets_sim::hero::steer(c.facing, to, rate);
            }
            if let Some((mode, target)) = act.travel {
                let m = [target[0] - c.position[0], 0.0, target[2] - c.position[2]];
                if matches!(mode, 5 | 6) && (m[0] != 0.0 || m[2] != 0.0) {
                    let p = if mode == 5 { [m[2], 0.0, -m[0]] } else { [-m[2], 0.0, m[0]] };
                    let l = p[0].hypot(p[2]).max(1e-6);
                    let w = if (c.facing[0] * p[0] + c.facing[2] * p[2]) / l > 0.9 { 1.0 } else { 0.1 };
                    c.facing = neopets_sim::hero::steer(c.facing, p, w);
                } else if let Some(wk) = c.walker.as_mut() {
                    // SPEC-0029 Behavior 2 Movement: the other modes run the navigation step of
                    // SPEC-0056 toward the move point (its route and weights), modes 3 and 4
                    // steering toward minus the direction.
                    wk.facing = c.facing;
                    wk.reverse = matches!(mode, 3 | 4);
                    wk.nav_step(&world.navmeshes, target, hero);
                    wk.reverse = false;
                    c.facing = wk.facing;
                }
            }
            // The boss moves only by its clip's root travel (SPEC-0014 Movement).
            let ground = &world.ground;
            if let Some(wk) = c.walker.as_mut() {
                wk.facing = c.facing;
                wk.lock = neopets_sim::ixi::height_lock(c.position[1], hero[1]);
                wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                c.position = wk.position();
            }
            c.hp -= act.hp_loss.max(0);
            if std::mem::discriminant(&before) != std::mem::discriminant(&after) {
                self.last_boss = format!("Ixi Chief {:?} -> {:?} (hp {}, mode {}) on tick {}", before, after, c.hp, k.mode, self.ticks);
            }
            let from = c.position;
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            if act.flash {
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, 222, add(from, [0.0, 2.0, 0.0]), neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            let d = {
                let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                let l = dx.hypot(dz).max(1e-6);
                [dx / l, 0.0, dz / l]
            };
            if let Some((damage, blocked)) = act.hero_damage {
                self.hero_hit(damage as f32, 0.0, blocked, d, "the Ixi Chief".into());
            }
            if let Some(p) = act.push {
                self.attack = None;
                self.loco.knock(p);
            }
        }
    }

    fn king_step(&mut self) {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.king.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let world = &self.world;
            let ground = |x: f32, z: f32| world.ground_at(x, z, 1.0e6).unwrap_or(0.0);
            let st = self.loco.state;
            let attacking = self.attack.is_some();
            let c = &mut self.characters[i];
            let fwd = unit([c.facing[0], 0.0, c.facing[2]]);
            let anchor = add(c.position, [0.0, 2.0, 0.0]);
            let p0 = add(anchor, scale(fwd, -1.5));
            let e1 = add(p0, scale(fwd, 7.0));
            let dist3 = (0..3).map(|k| (hero[k] - c.position[k]).powi(2)).sum::<f32>().sqrt();
            let reach = dist3 <= 65.0 && neopets_sim::gnarfas::segment_distance(p0, e1, hero, add(hero, [0.0, 3.5, 0.0])) <= 2.5 + 1.0;
            let clip = c.anim.clip_state();
            let v = neopets_sim::werelupe::View {
                position: c.position,
                facing: c.facing,
                hero,
                hero_facing: hf,
                hero_guarding: st == 20,
                hero_down: st == hero_states::DEAD || matches!(st, 7..=11 | 0x2B..=0x2E),
                hero_attacking: attacking,
                hero_attack_state: st == 13,
                was_hit: c.pending_hit.take().is_some(),
                reach,
                frame_bits: c.anim.frame_bits(),
                clip: (clip.action, clip.progress, clip.progress >= 0.99),
                hp: c.hp,
                max_hp: c.row.map_or(1, |r| r.hit_points as i32),
                hand: add(add(c.position, [0.0, 3.0, 0.0]), scale(fwd, 1.1)),
                ground: &ground,
            };
            let Some(k) = c.king.as_mut() else { continue };
            let before = k.state;
            let act = k.tick(&v, &self.king_words, &mut self.rng);
            let after = k.state;
            if let Some(a) = act.anim {
                if clip.action != a || (clip.progress >= 0.99 && std::mem::discriminant(&before) != std::mem::discriminant(&after)) {
                    c.anim.play(a);
                }
            }
            if let Some(rate) = act.face {
                let to = [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]];
                c.facing = neopets_sim::hero::steer(c.facing, to, rate);
            }
            match act.travel {
                neopets_sim::werelupe::Move::Fly(vel) => {
                    c.position = add(c.position, vel);
                    if vel[0] != 0.0 || vel[2] != 0.0 {
                        c.facing = neopets_sim::hero::steer(c.facing, [vel[0], 0.0, vel[2]], 0.2);
                    }
                    if let Some(wk) = c.walker.as_mut() {
                        wk.body.position = c.position;
                        wk.facing = c.facing;
                    }
                }
                neopets_sim::werelupe::Move::Still => {}
                neopets_sim::werelupe::Move::Steer { mode, target, weight } => {
                    let m = [target[0] - c.position[0], 0.0, target[2] - c.position[2]];
                    if mode != 255 && (m[0] != 0.0 || m[2] != 0.0) {
                        let (dir, w) = match mode {
                            5 | 6 => {
                                let p = if mode == 5 { [m[2], 0.0, -m[0]] } else { [-m[2], 0.0, m[0]] };
                                let l = p[0].hypot(p[2]).max(1e-6);
                                (p, if (c.facing[0] * p[0] + c.facing[2] * p[2]) / l > 0.9 { 1.0 } else { 0.1 })
                            }
                            _ => (m, weight),
                        };
                        c.facing = neopets_sim::hero::steer(c.facing, dir, w);
                    }
                    let ground = &world.ground;
                    if let Some(wk) = c.walker.as_mut() {
                        wk.facing = c.facing;
                        wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                        c.position = wk.position();
                    }
                }
            }
            if std::mem::discriminant(&before) != std::mem::discriminant(&after) {
                self.last_boss = format!("Werelupe King {:?} -> {:?} (hp {}, phase {}) on tick {}", before, after, c.hp, k.phase + 1, self.ticks);
            }
            let from = c.position;
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            for &f in &act.set_flag {
                self.state.set(f, true);
            }
            let d = {
                let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                let l = dx.hypot(dz).max(1e-6);
                [dx / l, 0.0, dz / l]
            };
            for centre in &act.rings {
                // The ring effect 415 at the boss's position.
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, 415, *centre, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            if let Some((damage, blocked)) = act.hero_damage {
                self.hero_hit(damage as f32, 0.0, blocked, d, "the Werelupe King".into());
            }
            for &damage in &act.raw_damage {
                self.hero_hit(damage as f32, 0.0, false, d, "the Werelupe King's flame or boomerang".into());
            }
        }
    }

    /// SPEC-0012 per tick for the Black Knight of the current world: the measurements (the
    /// monster weapon tests of attach points 1 to 5 against the hero's hurt capsules, the hero's
    /// body and shots against the six weak-point cylinders, the shots near the body, the hero on
    /// the knight's ground mesh, body contact, the ground), the controller tick, then its clip,
    /// turn, travel (mode 0 toward the target, modes 5 and 6 sideways, on the walker), hits on
    /// the hero with the knock-back, level states, the spark at a hit part and the health
    /// display (350 at most, the part sum with its quirk). Interpretations (Medium-Low, R-0097):
    /// a part's frame kind is taken as the bone with that index, its cylinder along the bone's
    /// up axis; the walk-mesh edge contact is not measured (false); the hand is 1.1 ahead of the
    /// body at 3.0 up and the sword's return point the body at 3.0 up.
    fn knight_step(&mut self) {
        use neopets_sim::combat::Cylinder;
        use neopets_sim::knight as kn;
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.knight.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        let Some(words) = self.knight_words else { return };
        if ids.is_empty() {
            return;
        }
        let hplace = placement(hero, hf);
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
        // The hero body cylinder (SPEC-0012, R-0097): radius 1.0, length W + 2.0 by the weapon
        // equipped (3.5 with none).
        let body_len = self.weapon_reach() + 2.0;
        let body = Cylinder { center: add(hero, [0.0, 0.5 * body_len, 0.0]), axis: UP, radius: 1.0, length: body_len };
        let shots: Vec<([f32; 3], f32)> = self.hero_shots.iter().map(|s| (s.position, s.radius)).collect();
        // The hero's damage query for the knight (argument 1.0), rounded down.
        let stat_a = self.weapon.and_then(|id| self.items.get(id)).filter(|i| i.is_weapon()).map_or(1, |i| i.stats[0]);
        let e_factor = self.pet_bonus.damage_factor(self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t));
        let hero_damage = neopets_sim::mote::melee(1.0, stat_a as f32, None, -1, 1.0, e_factor, 0.0, self.statuses.damage_scale()).0;
        let hero_mesh = neopets_sim::nav::closest(&self.world.navmeshes, hero, 1);
        let (hero_index, attacking, guarding) = (self.hero_index as u8, self.loco.state == hero_states::ATTACK, self.loco.state == hero_states::GUARD);
        // SPEC-0012 G9, the strike-active predicate: Tor in the attack state with sub-state 11 or
        // 12 or in jump state 7 in air phase 8 or 9; Roberta in the attack state with 5 or 12.
        let striking = match self.hero_index {
            0 => (attacking && self.attack.as_ref().is_some_and(|a| matches!(a.sub, 11 | 12))) || (self.loco.state == 7 && matches!(self.loco.phase, 8 | 9)),
            _ => attacking && self.r_attack.as_ref().is_some_and(|a| matches!(a.sub, 5 | 12)),
        };
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let world = &self.world;
            let c = &mut self.characters[i];
            let place = placement(c.position, c.facing);
            let frames = c.anim.bone_frames();
            let bits = c.anim.frame_bits();
            let weapons: [bool; 5] = std::array::from_fn(|n| {
                if bits & (1 << n) == 0 {
                    return false;
                }
                let Some(&(Some(b), angles, offset)) = c.anim.bones().attach.get(n) else { return false };
                let Some(bw) = frames.get(b) else { return false };
                let w = place.mul(bw).mul(&attach_rotation(offset, angles));
                let (radius, length) = c.weapons[n];
                let weapon = Cylinder { center: w.t, axis: unit(w.cols[2]), radius, length };
                caps.iter().any(|cap| neopets_sim::combat::overlap(&weapon, cap))
            });
            // The weak points: (joint, radius, length) by part, the selectors 37, 34, 24, 9, 4 and
            // 7 in the order left knee, right knee, left under-arm, right under-arm, back, neck
            // (SPEC-0012 Open question 21); the cylinder runs from the joint along the frame's
            // second basis vector.
            let part_shape = |k: usize| -> Option<Cylinder> {
                let (bone, radius, length) = match k {
                    kn::LEFT_KNEE => (37, 1.5, 2.0),
                    kn::RIGHT_KNEE => (34, 1.5, 2.0),
                    kn::LEFT_ARM => (24, 2.0, 2.5),
                    kn::RIGHT_ARM => (9, 2.0, 2.5),
                    kn::BACK => (0x04, 1.5, 2.0),
                    _ => (0x07, 2.5, 2.0),
                };
                let m = place.mul(frames.get(bone)?);
                Some(Cylinder::from_base(m.t, unit(m.cols[1]), radius, length))
            };
            let part_hit = |k: usize| part_shape(k).is_some_and(|cyl| neopets_sim::combat::overlap(&body, &cyl));
            let part_shot = |k: usize| part_shape(k).is_some_and(|cyl| shots.iter().any(|&(p, r)| (0..3).map(|j| (p[j] - cyl.center[j]).powi(2)).sum::<f32>().sqrt() < 5.0 + r));
            let r = c.row.map_or(1.5, |r| r.radius);
            let height = c.row.map_or(6.0, |r| r.height);
            let shot_overlap = shots.first().is_some_and(|&(p, sr)| neopets_sim::gnarfas::segment_distance(c.position, add(c.position, [0.0, height, 0.0]), p, p) <= r + sr);
            let hd = (hero[0] - c.position[0]).hypot(hero[2] - c.position[2]);
            let fwd = unit([c.facing[0], 0.0, c.facing[2]]);
            let sword_hit = |p: [f32; 3]| neopets_sim::gnarfas::segment_distance(hero, add(hero, [0.0, 3.5, 0.0]), p, p) <= 2.0;
            let own_mesh = neopets_sim::nav::closest(&world.navmeshes, c.position, 1);
            let clip = c.anim.clip_state();
            let v = kn::View {
                position: c.position,
                facing: c.facing,
                hero,
                hero_facing: hf,
                hero_index,
                hero_attack: attacking,
                hero_guard: guarding,
                hero_strike: striking,
                shots: !shots.is_empty(),
                shot_overlap,
                weapons,
                frame_bits: bits,
                clip: (clip.action, clip.progress >= 0.99),
                hero_on_mesh: hero_mesh >= 0 && hero_mesh == own_mesh,
                hero_on_mesh_full: hero_mesh >= 0 && hero_mesh == own_mesh,
                contact: hd < r + 1.0,
                edge: false,
                part_hit: &part_hit,
                part_shot: &part_shot,
                hero_damage,
                second_measure: hd,
                ground: world.ground_at(c.position[0], c.position[2], c.position[1] + 8.0),
                // Attachment point 0, the hand grip: the sword's attach and return point
                // (SPEC-0012 Bone table and attachment points).
                hand: Self::attach_point(c, 0).unwrap_or(add(add(c.position, [0.0, 3.0, 0.0]), scale(fwd, 1.1))),
                return_point: Self::attach_point(c, 0).unwrap_or(add(c.position, [0.0, 3.0, 0.0])),
                sword_hit: &sword_hit,
                level: &level,
            };
            let Some(k) = c.knight.as_mut() else { continue };
            let before = k.state;
            let act = k.tick(&v, &words, &mut self.rng);
            let after = k.state;
            if let Some(a) = act.anim {
                if clip.action != a || (clip.progress >= 0.99 && before != after) {
                    c.anim.play(a);
                }
            }
            if let Some(rate) = act.face {
                let to = [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]];
                let to = if rate < 0.0 { [-to[0], 0.0, -to[2]] } else { to };
                c.facing = neopets_sim::hero::steer(c.facing, to, rate.abs());
            }
            if let Some((mode, target)) = act.travel {
                let m = [target[0] - c.position[0], 0.0, target[2] - c.position[2]];
                if m[0] != 0.0 || m[2] != 0.0 {
                    let dir = match mode {
                        5 => [m[2], 0.0, -m[0]],
                        6 => [-m[2], 0.0, m[0]],
                        _ => m,
                    };
                    c.facing = neopets_sim::hero::steer(c.facing, dir, 0.1);
                }
            }
            let ground = &world.ground;
            if let Some(wk) = c.walker.as_mut() {
                wk.facing = c.facing;
                wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                c.position = wk.position();
            }
            c.hp = kn::display_hp(&k.parts);
            if before != after {
                let centres: Vec<[i32; 3]> = (0..6).filter_map(|p| part_shape(p)).map(|c| c.center.map(|x| x.round() as i32)).collect();
                self.last_boss = format!("Black Knight {:?} -> {:?} (parts {:?}, part centres {:?}, hero at {:?}) on tick {}", before, after, k.parts, centres, hero.map(|x| x.round() as i32), self.ticks);
            }
            let spark = act.part_effect.and_then(|p| part_shape(p)).map(|cyl| cyl.center);
            let from = c.position;
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            if act.consume_shot && !self.hero_shots.is_empty() {
                self.hero_shots.remove(0);
            }
            if let Some(p) = spark {
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, 305, p, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            if let Some((damage, knock)) = act.hero_damage {
                let d = {
                    let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                    let l = dx.hypot(dz).max(1e-6);
                    [dx / l, 0.0, dz / l]
                };
                if self.hero_hit(damage as f32, 0.0, false, d, "the Black Knight".into()) {
                    if let Some(kv) = knock {
                        self.loco.body.velocity = kv;
                    }
                }
            }
        }
    }

    /// SPEC-0013 per tick for the Gelert Assassins of the current world (the boss and its
    /// clones): the measurements (weapon 1 of attach point 0 against the hero's hurt capsules
    /// while event bit 0x1 is on; the melee reaction's reach and facing, as the common enemies'
    /// defence, 8.0 and 0.5; a hero shot whose 180-tick path passes within the body; a shot
    /// within 10; the hits waiting from the hero's swing and shots; the walker's navmesh
    /// blocked flag; a landing point's walkability on the boss's mesh; locators 200 to 207),
    /// the controller tick, then its clip and rate, turn, travel (approach and the sideways
    /// modes on the walker), the straight rainbow step, the placement, unreduced hits on the
    /// hero, the dagger launches into the enemy projectile pool, the level states, the flinch
    /// flash 222, the heal and its aura 428, the light arc's missile wall, the clone wake and
    /// a hit clone's reset (effect 559). Interpretations (R-0098): the missile-reaction and hit
    /// tests use the body cylinder for the bone cylinders; the clones are the block's records
    /// after the first; a hidden clone is inactive.
    fn gelert_step(&mut self) {
        use neopets_sim::combat::Cylinder;
        use neopets_sim::gelert as ge;
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let Some(words) = self.gelert_words.clone() else { return };
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.gelert.is_some() && !c.held).map(|c| c.id).collect();
        if ids.is_empty() {
            return;
        }
        if std::env::var_os("NEOPETS_TRACE_GELERT").is_some() && self.ticks % 120 == 0 {
            let all: Vec<String> = self.characters.iter().filter(|c| c.pack == pack && c.gelert.is_some()).map(|c| format!("[id {} entity {:?} slot {} inactive {} held {} {:?} anim {}]", c.id, c.entity, c.gelert_slot, c.inactive, c.held, c.gelert.as_ref().map(|g| g.state), c.anim.action())).collect();
            println!("      gelerts on tick {}: {}", self.ticks, all.join(" "));
        }
        let hplace = placement(hero, hf);
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
        let shots: Vec<([f32; 3], [f32; 3], f32)> = self.hero_shots.iter().map(|s| (s.position, scale(s.dir, s.speed), s.radius)).collect();
        let attacking = self.attack.is_some();
        let guarding = self.loco.state == hero_states::GUARD;
        let hero_anim = self.hero.clip_state().action;
        let ratio = self.hero_hp as f32 / self.hero_max.max(1) as f32;
        let clones_alive = self.characters.iter().filter(|c| c.pack == pack && c.gelert.as_ref().is_some_and(|g| g.clone && g.state != ge::State::CloneWait)).count();
        let level_flags = self.state.level_flags.clone();
        let level = |f: u16| level_flags.contains(&f);
        let world = &self.world;
        let rainbow: [Option<[f32; 3]>; 8] = std::array::from_fn(|k| world.tags.iter().position(|&t| t as usize == 200 + k).and_then(|i| world.entries.get(i)).map(|e| e.0));
        let mut wake = None;
        let mut new_shots: Vec<neopets_sim::projectile::Setup> = Vec::new();
        let mut hero_hits: Vec<(i32, [f32; 3])> = Vec::new();
        let mut walls: Vec<([f32; 3], [f32; 3])> = Vec::new();
        let mut effects: Vec<(u32, [f32; 3])> = Vec::new();
        let mut levels: Vec<u16> = Vec::new();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let c = &mut self.characters[i];
            let place = placement(c.position, c.facing);
            let frames = c.anim.bone_frames();
            let bits = c.anim.frame_bits();
            let weapon_touch = bits & 1 != 0
                && c.anim.bones().attach.first().and_then(|&(bone, angles, offset)| Some(place.mul(frames.get(bone?)?).mul(&attach_rotation(offset, angles)))).is_some_and(|w| {
                    let (radius, length) = c.weapons[0];
                    let weapon = Cylinder { center: w.t, axis: unit(w.cols[2]), radius, length };
                    caps.iter().any(|cap| neopets_sim::combat::overlap(&weapon, cap))
                });
            let off = [c.position[0] - hero[0], 0.0, c.position[2] - hero[2]];
            let dist = off[0].hypot(off[2]);
            let melee_reach = dist < neopets_sim::enemy::DEFENCE_REACH && (hf[0] * off[0] + hf[2] * off[2]) / dist.max(1e-6) > neopets_sim::enemy::DEFENCE_ARC;
            // SPEC-0013 4.13 missile reaction (R-0098): each hero missile's path (point to point +
            // 180 x velocity) against every posed bone cylinder of positive radius (full length,
            // along posed local Y, centred halfway from its base).
            let bones: Vec<Cylinder> = c
                .anim
                .bones()
                .hurt
                .iter()
                .zip(&frames)
                .filter(|(h, _)| h.2 > 0.0)
                .map(|(&(base, length, radius), f)| {
                    let w = place.mul(f);
                    Cylinder::from_base(w.apply(base), unit(w.cols[1]), radius, length)
                })
                .collect();
            let missile_threat = shots.iter().any(|&(p, v, _)| bones.iter().any(|b| neopets_sim::combat::segment_meets(p, add(p, scale(v, 180.0)), b)));
            let missile_near = shots.iter().any(|&(p, _, _)| (0..3).map(|k| (p[k] - c.position[k]).powi(2)).sum::<f32>().sqrt() < 10.0);
            let pending = c.gelert_hit.take();
            // The landing search accepts a point on any navmesh of the region (SPEC-0013, R-0098).
            let walkable = |p: [f32; 3]| world.navmeshes.iter().any(|m| m.first_holding(p).is_some());
            let blocked = c.walker.as_ref().is_some_and(|w| w.blocked);
            let clip = c.anim.clip_state();
            let frame = c.anim.layer0().map_or(0, |(_, f)| f.max(0) as u32);
            let max_hp = c.row.map_or(800, |r| r.hit_points as i32);
            let v = ge::View {
                position: c.position,
                facing: c.facing,
                hero,
                hero_facing: hf,
                hero_guard: guarding,
                hero_attacking: attacking,
                hero_anim,
                hero_hp_ratio: ratio,
                hero_radius: 1.0,
                hero_height: 3.5,
                hp: c.hp,
                max_hp,
                clip: (clip.action, clip.progress, clip.progress >= 0.99),
                frame,
                bits,
                weapon_touch,
                melee_reach,
                missile_threat,
                missile_near,
                hit_melee: pending.is_some_and(|(_, shot)| !shot),
                hit_missiles: pending.is_some_and(|(_, shot)| shot) as u32,
                hit_damage: pending.map_or(0, |(d, _)| d),
                nav_blocked: blocked,
                level: &level,
                walkable: &walkable,
                rainbow,
                clones_alive,
            };
            let Some(g) = c.gelert.as_mut() else { continue };
            let before = g.state;
            // G7: damage only when the hit makes the boss flinch, times the time factor.
            let act = g.tick(&v, &words, &mut self.rng);
            c.hp = (c.hp - act.hp_loss).max(0);
            let after = g.state;
            let is_clone = g.clone;
            c.anim.set_rate(act.rate);
            if let Some(a) = act.anim {
                if clip.action != a || (clip.progress >= 0.99 && before != after) {
                    c.anim.play(a);
                }
            }
            if let Some(rate) = act.face {
                let to = [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]];
                c.facing = neopets_sim::hero::steer(c.facing, to, rate);
            }
            if let Some(p) = act.place {
                c.position = p;
                if let Some(wk) = c.walker.as_mut() {
                    wk.body.position = p;
                }
            }
            if let Some(s) = act.step {
                c.position = add(c.position, s);
                if let Some(wk) = c.walker.as_mut() {
                    wk.body.position = c.position;
                }
            }
            if let Some((mode, target)) = act.travel {
                let m = [target[0] - c.position[0], 0.0, target[2] - c.position[2]];
                if m[0] != 0.0 || m[2] != 0.0 {
                    let dir = match mode {
                        5 => [m[2], 0.0, -m[0]],
                        6 => [-m[2], 0.0, m[0]],
                        _ => m,
                    };
                    c.facing = neopets_sim::hero::steer(c.facing, dir, 0.1);
                }
                if let Some(a) = neopets_sim::enemy::slot_entry(&ge::SLOTS, mode) {
                    if clip.action != a {
                        c.anim.play(a);
                    }
                }
            }
            let ground = &world.ground;
            if act.place.is_none() && act.step.is_none() {
                if let Some(wk) = c.walker.as_mut() {
                    wk.facing = c.facing;
                    wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                    c.position = wk.position();
                }
            }
            c.hp = (c.hp + act.heal).min(max_hp);
            // A clone waiting in base state 42 is not drawn.
            c.inactive = g.state == ge::State::CloneWait;
            if act.clone_reset {
                c.inactive = true;
                effects.push((559, c.position));
            }
            if before != after {
                self.last_boss = format!("Gelert Assassin{} {:?} -> {:?} (hp {}, pattern {}) on tick {}", if is_clone { " clone" } else { "" }, before, after, c.hp, g.pattern, self.ticks);
            }
            for &e in &act.effects {
                effects.push((e, c.position));
            }
            if let Some(on) = act.aura {
                effects.push((if on { 428 } else { 0 }, c.position));
            }
            levels.extend(act.set_level.iter().copied());
            if let Some(d) = act.hero_damage {
                hero_hits.push((d, c.position));
            }
            if let Some(ab) = act.arc {
                walls.push(ab);
            }
            if act.wake_clones {
                wake = Some((c.position, c.facing));
            }
            // Daggers from attach point 1 (or the body middle).
            if !act.daggers.is_empty() {
                let origin = c.anim.bones().attach.get(1).and_then(|&(bone, angles, offset)| Some(place.mul(frames.get(bone?)?).mul(&attach_rotation(offset, angles)).t)).unwrap_or([c.position[0], c.position[1] + 3.0, c.position[2]]);
                for d in &act.daggers {
                    let to = sub_v(d.aim, origin);
                    let dist = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
                    new_shots.push(neopets_sim::projectile::Setup {
                        kind: 0,
                        visual: 0,
                        element: 0,
                        owner: Some(id),
                        start: origin,
                        velocity: scale(unit(to), d.speed),
                        lifetime: ((dist + 10.0) / d.speed) as i32,
                        damage: d.damage,
                        radius: d.radius,
                        spin: 0.0,
                        area_radius: 0.0,
                        area_duration: 0,
                        key: -1,
                        ignore_world: false,
                        status: (-1, 0, 0),
                        effects: [-1; 6],
                    });
                }
            }
        }
        // The clone wake: each hidden clone placed at its slot offset and woken.
        if let Some((boss, facing)) = wake {
            for c in self.characters.iter_mut().filter(|c| c.pack == pack) {
                let slot = c.gelert_slot;
                if let Some(g) = c.gelert.as_mut().filter(|g| g.clone && g.state == ge::State::CloneWait) {
                    g.wake();
                    c.inactive = false;
                    c.position = ge::clone_place(boss, facing, slot.saturating_sub(1));
                    c.facing = facing;
                    if let Some(wk) = c.walker.as_mut() {
                        wk.body.position = c.position;
                    }
                }
            }
        }
        for f in levels {
            self.state.set_level(f, true);
        }
        for s in new_shots {
            self.shots.submit(s);
        }
        for (a, b) in walls {
            self.hero_shots.retain(|s| !ge::wall_stops(a, b, s.position, scale(s.dir, s.speed)));
        }
        let table = self.effect_table.clone();
        for (e, p) in effects {
            if e != 0 {
                self.effects.one_shot(&table, e as i32, p, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
        }
        for (damage, from) in hero_hits {
            let d = {
                let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                let l = dx.hypot(dz).max(1e-6);
                [dx / l, 0.0, dz / l]
            };
            self.hero_hit(damage as f32, 0.0, false, d, "the Gelert Assassin".into());
        }
    }

    /// SPEC-0011 per tick for the Faerie Sisters in the current world: the group's recount (level
    /// event 2 once at 0), then each sister's measurements (the hit, the hero shots whose next
    /// 10 ticks pass within her body, the clip and its event bits), her controller tick and its
    /// requests (the clip and its rate 2 - f, the turn toward the hero, the travel, the pool
    /// launch, the effect and the level events). The travel moves her straight at the path
    /// step's speed with her height kept (the route step, collision and the navmesh blocked flag
    /// are not built, so she never flips on a block).
    fn sisters_step(&mut self) {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.sister.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        if ids.is_empty() {
            self.sister_group = Default::default();
            return;
        }
        let alive = self.characters.iter().filter(|c| ids.contains(&c.id) && c.sister.as_ref().is_some_and(|s| !s.dead())).count() as i32;
        if self.sister_group.recount(alive) {
            self.state.set_level(2, true);
        }
        let shots: Vec<([f32; 3], [f32; 3], f32)> = self.hero_shots.iter().map(|s| (s.position, scale(s.dir, s.speed), s.radius)).collect();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let c = &mut self.characters[i];
            let body = c.row.map_or(1.0, |r| r.radius);
            let height = c.row.map_or(3.0, |r| r.height);
            let (a, b) = (c.position, add(c.position, [0.0, height, 0.0]));
            let near = shots.iter().take(6).filter(|&&(p, v, r)| neopets_sim::gnarfas::segment_distance(p, add(p, scale(v, 10.0)), a, b) <= body + r).count();
            let clip = c.anim.clip_state();
            // The alternate loops' avoid point: the far centroid pick of her ground mesh, 10 from
            // the hero, while the hero is within 5.0 of her.
            let close = (c.position[0] - hero[0]).hypot(c.position[2] - hero[2]) < 5.0;
            let far = if close && c.sister.as_ref().is_some_and(|s| s.alternate && s.avoid.is_none()) {
                let meshes = &self.world.navmeshes;
                usize::try_from(neopets_sim::nav::closest(meshes, c.position, 1)).ok().and_then(|m| meshes.get(m)).and_then(|m| m.far_centroid(hero, 10.0, &mut self.rng))
            } else {
                None
            };
            let v = neopets_sim::sisters::View {
                position: c.position,
                hero,
                hp: c.hp,
                max_hp: c.row.map_or(1, |r| r.hit_points as i32),
                hit: c.sister_hit.take(),
                missiles_near: near,
                clip: (clip.action, clip.progress >= 0.99),
                progress: clip.progress,
                frame_bits: c.anim.frame_bits(),
                blocked: false,
                level: &level,
                far,
            };
            let Some(s) = c.sister.as_mut() else { continue };
            let before = s.state;
            let act = s.tick(&v, &mut self.sister_group, &mut self.sister_words, id, &mut self.rng);
            // Each sister loop still stepped advances the drain once (SPEC-0034 3.13).
            let step = self.drain.as_mut().map(|(d, ..)| d.tick(&self.drain_words, &mut self.rng));
            let after = s.state;
            if let Some(n) = act.anim {
                if clip.action != n || (clip.progress >= 0.99 && before != after) {
                    c.anim.play(n);
                }
            }
            c.anim.set_rate(act.rate);
            if let Some(w) = act.face {
                let to = [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]];
                c.facing = neopets_sim::hero::steer(c.facing, to, w);
            }
            if let Some((p, speed)) = act.travel {
                let m = [p[0] - c.position[0], 0.0, p[2] - c.position[2]];
                let l = m[0].hypot(m[2]);
                if l > 1e-6 {
                    c.position = add(c.position, scale(m, speed / l));
                }
            }
            c.hp -= act.hp_loss;
            if act.remove {
                c.dying = Some(1);
            }
            if std::mem::discriminant(&before) != std::mem::discriminant(&after) {
                self.last_boss = format!("{} {:?} -> {:?} (hp {}, alive {}, shield {}) on tick {}", c.name, before, after, c.hp, self.sister_group.alive, self.sister_group.shielded, self.ticks);
            }
            let at = c.position;
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            if let Some((fx, up)) = act.effect {
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, fx, add(at, [0.0, up, 0.0]), neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            if let Some(setup) = act.launch {
                self.shots.submit(setup);
            }
            if let Some(step) = step {
                self.drain_step(step);
            }
        }
    }

    /// SPEC-0011 Thorn hits (Medium): on the first tick a thorn shot's pool slot is free again,
    /// a live Malice (kind selector 0) claims a hazard record at the shot's last position, its
    /// height from the ground probe started 8 above (Medium-Low). The hazards then tick: contact
    /// with the hero's body (radius word 0x2DCB20) deals the thorn damage (10) and waits 120
    /// ticks; after the 60-tick growth each swing whose weapon volume (radius 1.0, length 2.5
    /// from the grip) reaches one counts a strike, and the third removes it. Not built: any
    /// drawn hazard (the spec names no model or effect).
    fn thorn_step(&mut self, hero: [f32; 3]) {
        let pack = self.world.pack;
        let malice = self.characters.iter().any(|c| c.pack == pack && c.dying.is_none() && c.sister.as_ref().is_some_and(|s| s.class == 13 && !s.dead()));
        for (k, slot) in self.shots.slots.iter().enumerate() {
            let now = slot.as_ref().filter(|s| s.s.kind == neopets_sim::thorn::SHOT_KIND).map(|s| s.position);
            if let (Some(last), None) = (self.thorn_shots[k], slot.as_ref()) {
                if malice {
                    let y = self.world.ground_at(last[0], last[2], last[1] + 8.0).unwrap_or(last[1]);
                    self.thorns.spawn([last[0], y, last[2]]);
                }
            }
            self.thorn_shots[k] = now;
        }
        if self.thorns.live().next().is_none() {
            return;
        }
        let r = self.thorn_radius;
        let touches = |p: [f32; 3]| neopets_sim::projectile::sphere_meets_body(p, r, hero, contact::HERO_RADIUS, HERO_HEIGHT);
        let blade = self.weapon_segment().map(|c| {
            let p0 = add(c.center, scale(c.axis, -0.5 * c.length));
            (p0, add(p0, scale(c.axis, 2.5)))
        });
        let reaches = move |p: [f32; 3]| blade.is_some_and(|(a, b)| neopets_sim::combat::segment_distance(a, b, p, p) <= 1.0);
        let swing = blade.map(|_| (self.swing_serial, &reaches as &dyn Fn([f32; 3]) -> bool));
        let events = self.thorns.tick(&touches, swing, self.sister_words.thorn_max_hits);
        for ev in events {
            match ev {
                neopets_sim::thorn::Event::Contact(_) => {
                    let d = self.enemy_damage(self.sister_words.special_damage[0] as f32, false, -1, 1.0);
                    self.hero_hit(d, 0.0, false, [0.0, 0.0, 1.0], "a thorn".into());
                }
                neopets_sim::thorn::Event::Removed(i) => self.last_boss = format!("thorn hazard {i} removed on tick {}", self.ticks),
            }
        }
    }

    /// One call of the Mote-eater drain: the effects follow the hero; a reveal starts its slot's
    /// effect and uses its Mote's charge (a Mote not held or spent is cleared from the
    /// equipment); the stop ends every effect. The slot effects' 0.5 scale is not applied.
    fn drain_step(&mut self, step: neopets_sim::sisters::DrainStep) {
        use neopets_sim::sisters::DrainStep;
        let hero = self.loco.position();
        let table = self.effect_table.clone();
        let Some((_, body, slots)) = self.drain.as_mut() else { return };
        match step {
            DrainStep::Stop => {
                let (body, slots) = (*body, std::mem::take(slots));
                for h in std::iter::once(body).chain(slots.iter().map(|s| s.0)) {
                    self.effects.stop(&table, h);
                    self.effects.release(h);
                }
                self.drain = None;
                return;
            }
            DrainStep::Reveal { mote, effect, offset, charge, .. } => {
                if effect >= 0 {
                    let h = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, effect, add(hero, offset), neopets_sim::effect::IDENTITY));
                    slots.push((h, offset));
                }
                if charge > 0 {
                    let word = self.equipment.motes[self.hero_index.min(1)].iter().position(|&m| m == Some(mote)).unwrap_or(0);
                    self.spend_charge(word, mote, charge);
                }
            }
            DrainStep::Follow => {}
        }
        if let Some((_, body, slots)) = self.drain.as_ref() {
            self.effects.set_position(*body, add(hero, [0.0, self.drain_words.body_height, 0.0]));
            for &(h, o) in slots {
                self.effects.set_position(h, add(hero, o));
            }
        }
    }

    /// SPEC-0029 Behavior 6 per tick for the Undead Draik (class 12) in the current world: the
    /// host's measurements and the controller's requests. The mouth is the head part's bone (7)
    /// and the chest the chest part's bone (1); the hit groups are the broken-part bones (the
    /// left and right legs 25 and 26, the wings 38 and 5, the arms 9 and 17, the body 1 with
    /// radius 2.0, the head 7), each a 1.0 by 3.0 cylinder along the bone's Y axis from half its
    /// length; the beam touches the hero when its segment comes within the hero's radius of a
    /// 3.5-tall hero axis; the claw and bite use the monster weapon test of the strike.
    fn draik_step(&mut self) {
        use neopets_sim::combat::Cylinder;
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.draik.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        if ids.is_empty() {
            return;
        }
        let hplace = placement(hero, hf);
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
        let swing = self.weapon_segment();
        let guarding = self.loco.state == hero_states::GUARD;
        let mote = self.equipment.motes[0][1];
        let reflects = [
            mote.is_some_and(|m| matches!(m, 160..=161 | 172..=173)),
            mote.is_some_and(|m| matches!(m, 162..=163 | 172..=173)),
        ];
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let c = &mut self.characters[i];
            let Some(stage) = c.draik.as_ref().map(|k| k.stage) else { continue };
            let place = placement(c.position, c.facing);
            let frames = c.anim.bone_frames();
            let bone = |b: usize| frames.get(b).map(|f| place.mul(f));
            let pos0 = c.position;
            let point = |b: usize| bone(b).map_or([pos0[0], pos0[1] + 2.0, pos0[2]], |m| m.t);
            // The hit groups' bones (R-0066): left leg 25, 27, 28; right leg 26, 32, 33; wings 37,
            // 47 to 51; left arm 9 to 11; right arm 17 to 19; body 1 to 4, 7, 54, 57; head 54, 57.
            // Stage 1 tests all seven, stage 3 all but the legs, stage 5 the head.
            let groups: &[usize] = match stage {
                1 => &[25, 27, 28, 26, 32, 33, 37, 47, 48, 49, 50, 51, 9, 10, 11, 17, 18, 19, 1, 2, 3, 4, 7, 54, 57],
                3 => &[37, 47, 48, 49, 50, 51, 9, 10, 11, 17, 18, 19, 1, 2, 3, 4, 7, 54, 57],
                _ => &[54, 57],
            };
            let melee_group = swing.as_ref().is_some_and(|s| {
                groups.iter().any(|&b| {
                    bone(b).is_some_and(|m| {
                        let axis = unit(m.cols[1]);
                        let cyl = Cylinder { center: add(m.t, scale(axis, 1.5)), axis, radius: if b == 7 { 2.0 } else { 1.0 }, length: 3.0 };
                        neopets_sim::combat::overlap(s, &cyl)
                    })
                })
            });
            let near_shot = |p: [f32; 3], r: f32| groups.iter().any(|&b| { let q = sub_v(p, point(b)); (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() } < 5.0 + r);
            let shot_group = self.hero_shots.iter().any(|s| near_shot(s.position, s.radius));
            let bits = c.anim.frame_bits();
            let contact = (0..5).any(|n| {
                if bits & (1 << n) == 0 {
                    return false;
                }
                let Some(&(Some(b), angles, offset)) = c.anim.bones().attach.get(n) else { return false };
                let Some(bw) = frames.get(b) else { return false };
                let w = place.mul(bw).mul(&attach_rotation(offset, angles));
                let (radius, length) = c.weapons[n];
                let weapon = Cylinder { center: w.t, axis: unit(w.cols[2]), radius, length };
                caps.iter().any(|cap| neopets_sim::combat::overlap(&weapon, cap))
            });
            // The mouth is bone 52; the beam test is the segment from it to the moving end point
            // against a vertical cylinder around the hero raised 1.5 (height 1.75, radius 5.0).
            let mouth = point(52);
            let beam_cyl = Cylinder { center: add(hero, [0.0, 1.5, 0.0]), axis: UP, radius: 5.0, length: 1.75 };
            let touches = |a: [f32; 3], b: [f32; 3]| neopets_sim::combat::segment_meets(a, b, &beam_cyl);
            let clip = c.anim.clip_state();
            let v = neopets_sim::draik::View {
                position: c.position,
                facing: c.facing,
                hero,
                hp: c.hp,
                max_hp: c.row.map_or(1, |r| r.hit_points as i32),
                mouth,
                chest: point(1),
                melee: swing.is_some(),
                shots: !self.hero_shots.is_empty(),
                melee_group,
                shot_group,
                weapon_contact: contact,
                beam_touches: &touches,
                hero_facing: hf,
                reflects: reflects.map(|r| r && guarding),
                clip: (clip.action, clip.progress >= 0.99),
                frame_bits: bits,
                level: &level,
            };
            let Some(k) = c.draik.as_mut() else { continue };
            let before = k.state;
            let act = k.tick(&v, &self.draik_words, id, &mut self.rng);
            let (after, stage_now) = (k.state, k.stage);
            if let Some(n) = act.anim {
                if clip.action != n || (clip.progress >= 0.99 && std::mem::discriminant(&before) != std::mem::discriminant(&after)) {
                    c.anim.play(n);
                }
            }
            let world = &self.world;
            let walk = |c: &mut Character| {
                let ground = &world.ground;
                if let Some(wk) = c.walker.as_mut() {
                    wk.facing = c.facing;
                    wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
                    c.position = wk.position();
                }
            };
            match act.travel {
                neopets_sim::draik::Move::Toward(p, w) => {
                    c.facing = neopets_sim::hero::steer(c.facing, [p[0] - c.position[0], 0.0, p[2] - c.position[2]], w);
                    walk(c);
                }
                neopets_sim::draik::Move::Turn(w) => {
                    c.facing = neopets_sim::hero::steer(c.facing, [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]], w);
                    walk(c);
                }
                neopets_sim::draik::Move::Still => {}
            }
            c.hp -= act.hp_loss;
            if std::mem::discriminant(&before) != std::mem::discriminant(&after) || act.hp_loss != 0 {
                self.last_boss = format!("Undead Draik stage {} {:?} -> {:?} (hp {}) on tick {}", stage_now, before, after, c.hp, self.ticks);
            }
            let parts: Vec<[f32; 3]> = act.breaks.iter().map(|&p| point(neopets_sim::draik::PART_BONES[p])).collect();
            let from = c.position;
            if act.consume_shot {
                self.hero_shots.retain(|s| !near_shot(s.position, s.radius));
            }
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            let table = self.effect_table.clone();
            for p in parts {
                self.effects.one_shot(&table, neopets_sim::draik::BREAK_EFFECT, p, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            if let Some(setup) = act.launch {
                self.shots.submit(setup);
            }
            // The spit records: a 0.5 sphere against the hero's body capsules, the ground from 8
            // above.
            let released = c.draik.as_ref().map_or([None; 2], |k| k.spits.map(|s| s.map(|s| s.position)));
            let spit_ends = {
                let ground = |p: [f32; 3]| world.ground_at(p[0], p[2], p[1] + 8.0);
                let hits = |p: [f32; 3], r: f32| caps.iter().any(|cap| neopets_sim::combat::overlap(&Cylinder { center: p, axis: UP, radius: r, length: 0.0 }, cap));
                c.draik.as_mut().map_or(Vec::new(), |k| k.spit_step(&self.draik_words, &hits, &ground))
            };
            let flying = c.draik.as_ref().map_or([None; 2], |k| k.spits.map(|s| s.map(|s| s.position)));
            // SPEC-0029 Spit appearance: the flight effect starts at the mouth on the release
            // tick and follows the spit; at the end it stops and the impact bursts there.
            for k in 0..2 {
                let flight = neopets_sim::draik::SPIT_FX[k].0;
                if self.spit_fx[k].is_none() {
                    if let Some(at) = released[k] {
                        let h = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, flight, at, neopets_sim::effect::IDENTITY));
                        self.effects.update(&table, h, None, &mut self.rng);
                        self.spit_fx[k] = Some(h);
                    }
                }
                if let (Some(h), Some(at)) = (self.spit_fx[k], flying[k]) {
                    self.effects.set_position(h, at);
                    self.effects.update(&table, h, None, &mut self.rng);
                }
            }
            for e in &spit_ends {
                if let Some(h) = self.spit_fx[e.record].take() {
                    self.effects.set_position(h, e.position);
                    self.effects.stop(&table, h);
                    self.effects.release(h);
                }
                self.effects.one_shot(&table, neopets_sim::draik::SPIT_FX[e.record].1, e.position, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            let spit_hits: Vec<i32> = spit_ends.iter().filter_map(|e| e.damage).collect();
            let d = {
                let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                let l = dx.hypot(dz).max(1e-6);
                [dx / l, 0.0, dz / l]
            };
            for damage in spit_hits {
                self.hero_hit(damage as f32, 0.0, false, d, "the Undead Draik's spit".into());
            }
            for (damage, throw) in act.hero_damage {
                self.hero_hit(damage as f32, 0.0, false, d, "the Undead Draik".into());
                if let Some((u, speed)) = throw {
                    // R-0066: the throw is speed x unit((0, 12, 0) + 9 u).
                    self.attack = None;
                    self.loco.knock(scale(unit([9.0 * u[0], 12.0, 9.0 * u[2]]), speed));
                }
            }
        }
    }

    /// SPEC-0054 per tick for the Darkest Faerie (class 37) in the current world: the host's
    /// measurements (the ground 8.0 above her, the hero's guard, slot-1 kind, in-attack test,
    /// attack form 2 and weapon reach within 65.0 against her body cylinder, the armed hero shots)
    /// and her requests (the clip, the turn, her height, the shove, the orb hits, the level
    /// states, the absorbed shots and the impact effects). The launch point is taken 0.85 of her
    /// height up (her mouth bone is not given); the summons by key, the slots' flight effects and
    /// the cast glow are not built.
    fn faerie_step(&mut self) {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.faerie.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        let swing = self.weapon_segment();
        let guarding = self.loco.state == hero_states::GUARD;
        let kind = neopets_sim::darkest::attack_kind(self.equipment.motes[self.hero_index.min(1)][1]);
        let attacking = if self.hero_index == 0 { self.attack.is_some() } else { self.r_attack.is_some() };
        let form2 = if self.hero_index == 0 { self.attack.as_ref().is_some_and(|a| matches!(a.sub, 4 | 7 | 8 | 12)) } else { self.r_attack.as_ref().is_some_and(|a| a.sub == 4) };
        let shots: Vec<([f32; 3], f32)> = self.hero_shots.iter().map(|s| (s.position, s.radius)).collect();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let progress = self.state.flag(364);
            let partner = self.characters.iter().find(|o| o.pack == pack && o.row.is_some_and(|r| r.class == Some(38))).map(|o| o.position);
            let summons_alive = self.duel_summons.iter().any(|id| self.characters.iter().any(|o| o.id == *id && o.dying.is_none() && !o.enemy.as_ref().is_some_and(|e| e.is_dead())));
            let c = &mut self.characters[i];
            let (radius, height) = c.row.map_or((1.0, 3.0), |r| (r.radius, r.height));
            let near = (hero[0] - c.position[0]).hypot(hero[2] - c.position[2]) < 65.0;
            let body = neopets_sim::combat::Cylinder::from_base(c.position, UP, radius, height);
            let reaches = near && swing.as_ref().is_some_and(|s| neopets_sim::combat::overlap(s, &body));
            // NEOPETS_TRACE_FAERIE: in the absorb window, her height and the nearest hero shot
            // (diagnostic).
            if std::env::var_os("NEOPETS_TRACE_FAERIE").is_some() && c.faerie.as_ref().is_some_and(|f| f.code == 7) && self.ticks % 30 == 0 {
                let near = shots.iter().map(|&(p, _)| ((p[0] - c.position[0]).powi(2) + (p[1] - c.position[1] - 0.5 * height).powi(2) + (p[2] - c.position[2]).powi(2)).sqrt()).fold(f32::MAX, f32::min);
                println!("      faerie window: at {:?} (h {height}, r {radius}), hero {:?}, {} shots, nearest {near:.1} on tick {}", c.position, hero, shots.len(), self.ticks);
            }
            // NEOPETS_TRACE_FAERIE: the stun-time measurements (diagnostic).
            if std::env::var_os("NEOPETS_TRACE_FAERIE").is_some() && c.faerie.as_ref().is_some_and(|f| f.code == 5) && attacking {
                let sub = if self.hero_index == 0 { self.attack.as_ref().map(|a| a.sub) } else { self.r_attack.as_ref().map(|a| a.sub) };
                println!("      faerie stun: attacking sub {sub:?} form2 {form2} swing {} reaches {reaches} on tick {}", swing.is_some(), self.ticks);
            }
            let clip = c.anim.clip_state();
            let ground = self.world.ground_at(c.position[0], c.position[2], c.position[1] + 8.0);
            let v = neopets_sim::darkest::View {
                position: c.position,
                facing: c.facing,
                hero,
                hero_facing: hf,
                hero_guard: guarding,
                hero_kind: kind,
                hero_attacking: attacking,
                form2,
                weapon_reaches: reaches,
                radius,
                height,
                hero_radius: contact::HERO_RADIUS,
                ground,
                shots: &shots,
                mouth: add(c.position, [0.0, 0.85 * height, 0.0]),
                clip: (clip.action, clip.progress >= 0.99),
                frame_bits: c.anim.frame_bits(),
                progress,
                level: &level,
            };
            let Some(f) = c.faerie.as_mut() else { continue };
            let duel = !progress && !level(18);
            let before = if duel { 20 + f.b_state } else { f.code };
            let act = if duel { f.loop_b(&v, &self.faerie_words, partner, &mut self.duel, summons_alive, &mut self.rng) } else { f.tick(&v, &self.faerie_words, &mut self.rng) };
            let after = if duel { 20 + f.b_state } else { f.code };
            if let Some(n) = act.anim {
                if clip.action != n || (clip.progress >= 0.99 && before != after) {
                    c.anim.play(n);
                }
            }
            if let Some(w) = act.face {
                c.facing = neopets_sim::hero::steer(c.facing, [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]], w);
            }
            if let Some((p, w)) = act.turn_to {
                c.facing = neopets_sim::hero::steer(c.facing, [p[0] - c.position[0], 0.0, p[2] - c.position[2]], w);
            }
            if let Some(y) = act.height {
                c.position[1] = y;
            }
            if before != after {
                self.last_boss = format!("Darkest Faerie code {before} -> {after} (absorbs {}, stuns {}) on tick {}", f.absorbs, f.stuns, self.ticks);
            }
            let from = c.position;
            let (keys, slot, facing) = (c.spawn_keys, c.slot, c.facing);
            for &(flag, on) in &act.levels {
                self.state.set_level(flag, on);
            }
            let mut gone: Vec<usize> = act.absorbed.clone();
            gone.sort_unstable();
            for k in gone.into_iter().rev() {
                if k < self.hero_shots.len() {
                    self.hero_shots.remove(k);
                }
            }
            let table = self.effect_table.clone();
            for &(fx, at) in &act.impacts {
                self.effects.one_shot(&table, fx, at, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            for &(burst, at) in &act.summons {
                // The burst by the slot-1 key facing her forward; the bolt by the slot-0 key
                // facing away from her; both drop item 172.
                let key = keys[burst as usize] as i16;
                let face = if burst { facing } else { unit([at[0] - from[0], 0.0, at[2] - from[2]]) };
                self.summon(pack, Some(slot), key, at, face, Some(172), false);
            }
            let d = {
                let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                let l = dx.hypot(dz).max(1e-6);
                [dx / l, 0.0, dz / l]
            };
            for &damage in &act.hero_damage {
                self.hero_hit(damage as f32, 0.0, false, d, "the Darkest Faerie's orb".into());
            }
            if let Some(p) = act.push {
                self.attack = None;
                let at = add(self.loco.position(), p.offset);
                self.loco.body.position = at;
                self.loco.knock(p.velocity);
            }
        }
    }

    /// SPEC-0029 Behavior 30 per tick for Queen Fyora (class 38) in the current world: she waits
    /// for a class-37 object, advances the duel's shared slots, faces it and runs her state; a
    /// deflected orb that reaches the hero hits for the word's 5 with no attacker, and a type-2
    /// orb that reaches the hero summons the Faerie's slot-0 key at the hero. Her sample point is
    /// taken 0.75 of her height up (her hand or staff point is not given).
    fn fyora_step(&mut self) {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.fyora.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        if ids.is_empty() {
            self.duel = [None; 4];
            return;
        }
        let faerie = self.characters.iter().find(|o| o.pack == pack && o.row.is_some_and(|r| r.class == Some(37))).map(|o| (o.position, o.spawn_keys[0], o.slot));
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level2 = self.state.level(2);
            let c = &mut self.characters[i];
            let height = c.row.map_or(3.0, |r| r.height);
            let clip = c.anim.clip_state();
            let v = neopets_sim::fyora::View {
                position: c.position,
                facing: c.facing,
                faerie: faerie.map(|f| f.0),
                hero,
                sample: add(c.position, [0.0, 0.75 * height, 0.0]),
                clip: (clip.action, clip.progress >= 0.99),
                frame_bits: c.anim.frame_bits(),
                level2,
            };
            let Some(q) = c.fyora.as_mut() else { continue };
            let before = q.state;
            let act = q.tick(&v, &mut self.duel, &self.duel_words, &mut self.rng);
            let after = q.state;
            if let Some(n) = act.anim {
                if clip.action != n || (clip.progress >= 0.99 && before != after) {
                    c.anim.play(n);
                }
            }
            if let Some(f) = act.face {
                c.facing = f;
            }
            if before != after {
                self.last_boss = format!("Queen Fyora state {before} -> {after} (slots {:?}) on tick {}", self.duel.iter().map(|s| s.map(|s| s.ty)).collect::<Vec<_>>(), self.ticks);
            }
            let table = self.effect_table.clone();
            for &(fx, at) in &act.effects {
                self.effects.one_shot(&table, fx, at, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            for &damage in &act.events.hero_damage {
                self.hero_hit(damage as f32, 0.0, false, hf, "a deflected orb".into());
            }
            if let (true, Some((_, key, slot))) = (act.events.summon, faerie) {
                if let Some(s) = self.summon(pack, Some(slot), key as i16, hero, hf, None, false) {
                    self.duel_summons.push(s);
                }
            }
        }
    }

    /// SPEC-0016 per tick for the Plague Serpent (class 2) in the current world: the anchors are
    /// the world's locators of the anchor ids, the lunge's hit is weapon 3 of the monster weapon
    /// test (event bit 0x4) against the hero's bone capsules, the stun strike is any hero hit
    /// that reaches it (the head cylinder is not tested), and the mouth is taken 2.5 above its
    /// position (the head bone is not given). A landed lunge is unreduced damage with the hit
    /// flash 222 at the hero; the hero's reaction state 0x36 is not built (the neck look is).
    fn serpent_step(&mut self) {
        use neopets_sim::combat::Cylinder;
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.serpent.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        if ids.is_empty() {
            return;
        }
        let hplace = placement(hero, hf);
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
        let world = &self.world;
        let locator = |k: usize| world.tags.iter().position(|&t| t as usize == k).and_then(|i| world.entries.get(i)).map(|e| e.0);
        let anchors = neopets_sim::serpent::ANCHORS.map(|ids| ids.map(locator));
        let (code, blocking, stunned) = (self.loco.state, self.loco.state == hero_states::GUARD, self.statuses.active(neopets_sim::status::STUN));
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let world = &self.world;
            let ground = |x: f32, z: f32, from: f32| world.ground_at(x, z, from);
            let c = &mut self.characters[i];
            let bits = c.anim.frame_bits();
            let place = placement(c.position, c.facing);
            let frames = c.anim.bone_frames();
            let lunge_contact = bits & 0x4 != 0
                && c.anim.bones().attach.get(2).and_then(|&(b, angles, offset)| {
                    let bw = frames.get(b?)?;
                    let w = place.mul(bw).mul(&attach_rotation(offset, angles));
                    let (radius, length) = c.weapons[2];
                    let weapon = Cylinder { center: w.t, axis: unit(w.cols[2]), radius, length };
                    Some(caps.iter().any(|cap| neopets_sim::combat::overlap(&weapon, cap)))
                }) == Some(true);
            let clip = c.anim.clip_state();
            let v = neopets_sim::serpent::View {
                position: c.position,
                facing: c.facing,
                hero,
                hero_facing: hf,
                hero_code: code,
                hero_blocking: blocking,
                hero_stunned: stunned,
                hp: c.hp,
                max_hp: c.row.map_or(1, |r| r.hit_points as i32),
                anchors,
                ground: &ground,
                lunge_contact,
                strike: c.serpent_hit.take(),
                mouth: add(c.position, [0.0, 2.5, 0.0]),
                clip: (clip.action, clip.progress >= 0.99),
                frame_bits: bits,
                level: &level,
            };
            let Some(sp) = c.serpent.as_mut() else { continue };
            let before = sp.state;
            let act = sp.tick(&v, &self.serpent_words, &mut self.rng);
            let after = sp.state;
            // SPEC-0016 6A: in state 0 the neck look at the active hero (joints 6 and 17 of the
            // animated pose); on any other tick the six angles go back to 0.
            c.anim.set_neck(None);
            if after == neopets_sim::serpent::State::Idle {
                let place = placement(c.position, c.facing);
                let frames = c.anim.bone_frames();
                if let (Some(a), Some(b)) = (self.neck_joints.first().and_then(|&j| frames.get(j)), self.neck_joints.last().and_then(|&j| frames.get(j))) {
                    neopets_sim::look::neck_step(&mut sp.neck, place.apply(a.t), place.apply(b.t), hero);
                }
            } else {
                sp.neck = [0.0; 6];
            }
            c.anim.set_neck(Some((self.neck_joints.clone(), sp.neck)));
            // NEOPETS_TRACE_NECK: the serpent's neck angles once a second (trace only).
            if std::env::var_os("NEOPETS_TRACE_NECK").is_some() && self.ticks % 60 == 0 {
                println!("NECK tick {} state {:?} angles {:?}", self.ticks, sp.state, sp.neck);
            }
            if let Some(n) = act.anim {
                if clip.action != n || (clip.progress >= 0.99 && before != after) {
                    c.anim.play(n);
                }
            }
            c.anim.set_rate(act.rate);
            if let Some((dir, w)) = act.face {
                c.facing = neopets_sim::hero::steer(c.facing, dir, w);
            }
            if let Some(p) = act.place {
                c.position = p;
            }
            c.hp = (c.hp - act.hp_loss).max(0);
            if std::mem::discriminant(&before) != std::mem::discriminant(&after) || before != after && matches!(after, neopets_sim::serpent::State::Bite(_)) {
                self.last_boss = format!("Plague Serpent {:?} -> {:?} (hp {}, phase {}, slot {}) on tick {}", before, after, c.hp, sp.phase, sp.slot, self.ticks);
            }
            let from = c.position;
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            for &(code, seconds) in &act.statuses {
                self.hero_status(code, seconds);
            }
            if let Some(setup) = act.launch {
                self.shots.submit(setup);
            }
            if act.consume_shot && !self.hero_shots.is_empty() {
                self.hero_shots.remove(0);
            }
            if let Some(damage) = act.lunge {
                let d = {
                    let (dx, dz) = (hero[0] - from[0], hero[2] - from[2]);
                    let l = dx.hypot(dz).max(1e-6);
                    [dx / l, 0.0, dz / l]
                };
                if self.hero_hit(damage as f32, 0.0, false, d, "the Plague Serpent's lunge".into()) {
                    let table = self.effect_table.clone();
                    self.effects.one_shot(&table, 222, add(hero, [0.0, 1.5, 0.0]), neopets_sim::effect::IDENTITY, &mut self.rng);
                }
            }
        }
    }

    /// SPEC-0036 Health and death, step 3: state 16, the melee hit list cleared.
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
                let reeds = if kind.family == neopets_sim::foliage::Family::Reed { plants.iter().map(|q| neopets_sim::foliage::Reed::new(q.yaw)).collect() } else { Vec::new() };
                built.push(LivePatch { kind, model, centre, plants, bushes, reeds, zone: p.zone as u8, on: true, overrides: p.overrides });
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
        // The hero's speed for the bush bend: this build takes its movement this tick (units
        // per second), as root motion moves the hero rather than the body velocity.
        let moved = self.bend_last.map_or(0.0, |l: [f32; 3]| ((hero[0] - l[0]).powi(2) + (hero[2] - l[2]).powi(2)).sqrt() * 60.0);
        self.bend_last = Some(hero);
        let facing = self.loco.facing;
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
                    // The body test (a segment with radius 0.75): read here as the hero's feet
                    // within 0.75 plus the hero radius of the bush base, horizontally, at a
                    // similar height.
                    let distance = (q.position[0] - hero[0]).hypot(q.position[2] - hero[2]);
                    let contact = distance < 0.75 + neopets_sim::contact::HERO_RADIUS && (q.position[1] - hero[1]).abs() < 3.0;
                    b.bend_step(p.kind.near == Some(198), &neopets_sim::foliage::BendHero { distance, contact, speed: moved, facing });
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
        // Behavior 9, reeds within 100 units of the eye: the bend away from the hero within 5
        // units, the cut by a weapon hit in the attack window (cue 22 for Tor, 28 for Roberta;
        // the piece launched along the weapon point's motion; the weapon test is taken as the bush capsule's, a
        // radius 0.5 and length 6.0 up from the base), the cut piece's fall and the drop roll.
        let mut reed_cuts = 0;
        // SPEC-0060 Cut set-up: the launch is the change of the weapon point 2.5 along its frame
        // axis between two samples (here this tick and the last); `Reed::cut` keeps its
        // horizontal part, normalised (a zero change gives a piece that never moves off).
        let point = weapon.as_ref().map(|w| add(w.center, scale(w.axis, 2.5 - 0.5 * w.length - 0.25)));
        let launch = point.zip(self.reed_point).map(|(a, b)| sub_v(a, b));
        self.reed_point = point;
        let worlds = [&self.world].into_iter().chain(self.other.as_ref());
        let worlds: Vec<_> = worlds.collect();
        for (&pack, patches) in self.foliage.iter_mut() {
            let Some(world) = worlds.iter().find(|w| w.pack == pack) else { continue };
            for p in patches.iter_mut().filter(|p| !p.reeds.is_empty() && p.on) {
                let d = [p.centre[0] - eye[0], p.centre[1] - eye[1], p.centre[2] - eye[2]];
                if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] >= 100.0 * 100.0 {
                    continue;
                }
                for (q, r) in p.plants.iter().zip(p.reeds.iter_mut()) {
                    match r.state {
                        0 => {
                            // Reed angles: the spring-back, the body contact and bend, then the
                            // weapon test inside the 5-unit gate with the latch clear, against a
                            // capsule of radius 0.5 and length 6 along the reed's current up.
                            let cuttable = r.standing_step(q.position, hero);
                            if let (true, Some(w)) = (cuttable, weapon.as_ref()) {
                                let reed = neopets_sim::combat::Cylinder::from_base(q.position, r.up, 0.5, 6.0);
                                if neopets_sim::combat::overlap(w, &reed) && r.cut(launch.unwrap_or([0.0; 3])) {
                                    reed_cuts += 1;
                                    if let Some(id) = neopets_sim::foliage::reed_drop(self.rng.range(0, 100)) {
                                        drops.push((pack, q.position, vec![id]));
                                    }
                                }
                            }
                        }
                        1 => {
                            let at = [q.position[0] + r.offset[0], q.position[1] + r.offset[1], q.position[2] + r.offset[2]];
                            r.fall_step(q.position[1], world.ground_at(at[0], at[2], at[1] + 8.0));
                        }
                        _ => {}
                    }
                }
            }
        }
        if reed_cuts > 0 && std::env::var_os("NEOPETS_TRACE_PATCHES").is_some() {
            println!("      {reed_cuts} reed(s) cut on tick {}", self.ticks);
        }
        for _ in 0..reed_cuts {
            self.cue(if self.hero_index == 0 { 22 } else { 28 });
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

    /// The world-shadow sun (this build's option): the brightest directional (type 2) light of
    /// the current world's light set (SPEC-0028 section 2), by colour times strength, else the
    /// scene default's main light (SPEC-0046: (0.666, -0.524, 0.531)); one that points level or
    /// upward is passed over. The direction is the light's travel direction, held at least 0.5
    /// downward so shadows stay a sensible length.
    pub fn sun_direction(&self) -> [f32; 3] {
        let brightness = |e: &neopets_sim::charlight::Entry| (e.colour[0] + e.colour[1] + e.colour[2]) * e.strength.abs().max(1e-3);
        let best = self.world.lights.iter().flat_map(|r| r.a.iter().chain(&r.b).chain(&r.c)).filter(|e| e.kind == 2 && unit(e.direction)[1] < -0.2).max_by(|a, b| brightness(a).total_cmp(&brightness(b)));
        let mut d = unit(best.map_or(neopets_sim::charlight::defaults()[1].direction, |e| e.direction));
        if d[1] > -0.5 {
            let h = (d[0] * d[0] + d[2] * d[2]).sqrt().max(1e-6);
            let k = (1.0f32 - 0.25).sqrt() / h;
            d = [d[0] * k, -0.5, d[2] * k];
        }
        d
    }

    /// SPEC-0046 Character shadow volumes for this frame: the hero is the one caster (the
    /// Gnarfas, the other kind on disc, is not built), taken while within 50 units of the eye;
    /// its hulls moved by its posed bones (the bank index taken as the skin index across the
    /// families), its light direction from its character light context (straight down without
    /// one) and the count signs from the camera eye.
    pub fn shadow_volume(&self) -> neopets_sim::shadow::Volume {
        use neopets_sim::shadow as sh;
        let mut v = sh::Volume::default();
        let hulls = &self.shadow_hulls[self.hero_index.min(1)];
        let (eye, ..) = self.view();
        let p = add(self.loco.position(), self.hero_draw_offset());
        let d = sub(p, eye);
        if hulls.is_empty() || sh::choose(&[d[0] * d[0] + d[1] * d[1] + d[2] * d[2]]).is_empty() {
            return v;
        }
        let place = placement(p, self.loco.facing);
        let frames = self.hero.bone_frames();
        let slots: Vec<Option<sh::LightSlot>> = self.char_lights.get(&None).map_or(Vec::new(), |s| {
            s.slots.iter().map(|x| x.entry.map(|e| sh::LightSlot { kind: e.kind, weight: x.weight, direction: e.direction, position: e.position })).collect()
        });
        let l = sh::light_direction(&slots, p, slots.is_empty()).unwrap_or([0.0, -1.0, 0.0]);
        let frame = |b: u8| frames.get(b as usize).map(|f| {
            let m = place.mul(f);
            sh::Frame { cols: m.cols, t: m.t }
        });
        sh::volume(hulls, frame, l, eye, &mut v);
        v
    }

    /// The Rope Gun's muzzle M = T + 0.37F - S of weapon attach frame 0 (SPEC-0033 State 27),
    /// or the hero's position 1.2 up when the frame is missing.
    fn rope_muzzle(&self) -> [f32; 3] {
        let p = self.loco.position();
        let frame = (|| {
            let place = placement(p, self.loco.facing);
            let frames = self.hero.bone_frames();
            let &(Some(bone), angles, offset) = self.hero.bones().attach.first()? else { return None };
            Some(place.mul(frames.get(bone)?).mul(&attach_rotation(offset, angles)))
        })();
        frame.map_or(add(p, [0.0, 1.2, 0.0]), |w| sub(add(w.t, scale(unit(w.cols[2]), 0.37)), unit(w.cols[0])))
    }

    /// The rope's drawn strand: the muzzle M and the endpoint Q while the manager is out or
    /// attached (SPEC-0033 State 27).
    pub fn rope_line(&self) -> Option<([f32; 3], [f32; 3])> {
        (self.rope.state != neopets_sim::rope::RopeState::Idle && self.loco.state == neopets_sim::rope::STATE).then(|| (self.rope_muzzle(), self.rope.q))
    }

    /// SPEC-0036 Right-stick aim mode and SPEC-0033 State 27, after the hero update: the aim
    /// mode and, while it is on with item 36 held, the anchor pass of the aim scan (from the
    /// camera eye, the sight line from 1.5 below it, stopped 1.0 short of the target); R2 in
    /// idle with an anchor selected enters state 27 (the spec's state-13 hop, taken at once);
    /// then state 27's phases. Returns whether the hero is in state 27.
    fn rope_step(&mut self, input: &HeroInput) -> bool {
        use neopets_sim::rope as rp;
        if self.rope_pack != self.world.pack {
            self.rope_pack = self.world.pack;
            self.rope.reset();
            if self.rope_hero.take().is_some() && self.loco.state == rp::STATE {
                self.loco.body.no_gravity = false;
                self.loco.enter_state(hero_states::STANCE, 0);
            }
        }
        let has = self.state.has_item(rp::ITEM);
        let scan = self.aim.step(self.loco.state, None, self.rope_y, input.r2_held, input.circle_held);
        if self.loco.state != rp::STATE {
            self.rope_hero = None;
            if scan && has {
                let (eye, mut f, _, _) = self.view();
                if let Some(t) = self.rope_aim.and_then(|k| self.world.aim_targets.get(k)) {
                    f = unit(sub(t.point, eye));
                }
                let from = [eye[0], eye[1] - rp::SIGHT_DROP, eye[2]];
                let targets = &self.world.aim_targets;
                let ground = self.world.ground.as_ref();
                // The sight sweep (this build's per-sample test; R-0121): the line from 1.5
                // below the eye to the target, stopped 1.0 short, is tested against the frame
                // records of the cell under each sample point (0, 5, 15, 30, 50, ...).
                let sight = |to: [f32; 3]| {
                    let d = sub(to, from);
                    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
                    let end = add(from, scale(d, ((l - 1.0) / l).max(0.0)));
                    ground.is_none_or(|g| rp::sight_samples(l - 1.0).iter().all(|&k| {
                        let q = add(from, scale(d, k / l));
                        !g.frames_at(q[0], q[2]).iter().any(|r| r.crossed_by(from, end))
                    }))
                };
                let pick = rp::scan(targets, eye, f, |i| sight(targets[i].point));
                for (i, t) in self.world.aim_targets.iter().enumerate() {
                    if Some(i) == pick && self.rope.anchor != Some(i) {
                        self.last_fx = format!("rope anchor {i} at {:.1},{:.1},{:.1} selected on tick {}", t.point[0], t.point[1], t.point[2], self.ticks);
                    }
                }
                if self.rope_aim.is_some() && self.ticks % 30 == 0 {
                    let near: Vec<String> = targets.iter().map(|t| {
                        let d = sub(t.point, eye);
                        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                        format!("{:.0}{}{}", l, if t.active { "" } else { " off" }, if sight(t.point) { "" } else { " blocked" })
                    }).collect();
                    println!("      rope scan tick {}: eye {:?} pick {pick:?} targets {near:?}", self.ticks, eye.map(|v| v.round()));
                    if let (Some(g), Some(t)) = (ground, targets.first()) {
                        let d = sub(t.point, from);
                        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                        let end = add(from, scale(d, (l - 1.0) / l));
                        for &k in &rp::sight_samples(l - 1.0) {
                            let q = add(from, scale(d, k / l));
                            for r in g.frames_at(q[0], q[2]).iter().filter(|r| r.crossed_by(from, end)).take(2) {
                                println!("        sample {k}: kind {} start {:?} face {:?} len {} h {}", r.kind, r.start.map(|v| v.round()), r.face.map(|v| (v * 100.0).round() / 100.0), r.length, r.height);
                            }
                        }
                    }
                }
                self.rope.select(pick.map(|i| (i, self.world.aim_targets[i].point)));
            } else {
                self.rope.select(None);
            }
            if !(self.aim.on && has && self.rope.anchor.is_some() && input.r2_held && self.loco.state == hero_states::STANCE) {
                return false;
            }
            self.rope_hero = Some(rp::RopeHero::default());
            self.loco.enter_state(rp::STATE, rp::FIRE_CLIP);
            self.trace_edge("rope", self.rope.anchor.unwrap_or(0), 0);
        }
        let Some(mut h) = self.rope_hero else {
            self.loco.enter_state(hero_states::STANCE, 0);
            return false;
        };
        let m = self.rope_muzzle();
        self.rope.update(m);
        let c = self.hero.clip_state();
        let view = rp::View { p: self.loco.position(), m, clip: (c.action, c.progress), frame_bits: self.hero.frame_bits(), touched: self.touched, square: input.square };
        let step = h.tick(view, &mut self.rope);
        self.rope_hero = Some(h);
        if step.holster {
            self.set_weapon_drawn(false);
        }
        if let Some(r) = step.request {
            self.loco.block_request(r);
        }
        if let Some((d, w)) = step.face {
            self.loco.steer_toward(d, w);
        }
        if let Some(cue) = step.cue {
            self.cue(cue);
        }
        self.loco.extra = step.push;
        self.loco.body.no_gravity = step.no_gravity;
        if step.no_gravity {
            self.loco.body.velocity = [0.0; 3];
        }
        if step.exit {
            self.rope_hero = None;
            self.loco.body.no_gravity = false;
            self.loco.enter_state(hero_states::STANCE, 0);
            self.trace_edge("rope off", 0, 0);
            return false;
        }
        true
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
    /// The draw zone gate (SPEC-0006 Binary grids; SPEC-0004, SPEC-0028): column `c` of world
    /// `pack` with key -1 (the active block). The current world uses block 0 with the camera's
    /// zone row; the second world's draws use the block keyed by its world id (SPEC-0006 Zone
    /// manager: the main loop makes that block active while the second world draws), which passes
    /// everything when the current grid has no such block. Ungated, the second world's whole
    /// backdrop drew over the current world's (1023 with 696: the far mountains fought, a
    /// flicker).
    pub fn zone_visible(&self, pack: u32, c: u8) -> bool {
        if pack == self.world.pack {
            return self.zones.query(self.world.zones.as_ref(), c as i32, -1);
        }
        match self.other.as_ref().filter(|o| o.pack == pack) {
            Some(o) => self.zones.query(self.world.zones.as_ref(), c as i32, o.world_id),
            None => true,
        }
    }

    /// Read the attack-family tables (SPEC-0034 Where it lives; 14 ids each, index m - 160).
    pub fn load_attack_tables(&mut self, boot: &[u8]) {
        use neopets_sim::rumble as r;
        self.rumble_presets = neopets_data::archive::boot_u32_table(boot, r::PRESETS, 40).map(|w| r::Preset::table(&w)).unwrap_or_default();
        let word = |at: u32| neopets_data::archive::boot_u32_table(boot, at, 1).ok().and_then(|w| w.first().copied());
        if let (Some(p), Some(t), Some(m)) = (word(r::LANDING_PRESET), word(r::LANDING_SECONDS), word(r::LANDING_MAGNITUDE)) {
            self.landing_rumble = (p as usize, f32::from_bits(t), f32::from_bits(m));
        }
        if std::env::var_os("NEOPETS_TRACE_RUMBLE").is_some() {
            println!("      rumble presets {:?}, landing {:?}", self.rumble_presets, self.landing_rumble);
        }
        if let Ok(w) = neopets_data::archive::boot_u32_table(boot, 0x002D_CA60, 14) {
            self.mote_glow_rgb = std::array::from_fn(|k| {
                let b = w[k].to_le_bytes();
                [b[0], b[1], b[2]]
            });
        }
        let t = |at: u32| neopets_data::archive::boot_u32_table(boot, at, 14).map(|w| std::array::from_fn(|k| w[k] as i32)).unwrap_or([-1; 14]);
        self.attack_fx.tables = AttackTables {
            line: t(0x0031_5280),
            aura: t(0x0031_53D0),
            charge: t(0x0031_5398),
            burst: [t(0x0031_5328), t(0x0031_5360)],
            hold: [t(0x0031_52B8), t(0x0031_52F0)],
            light: t(0x0031_5588),
        };
        // SPEC-0034 Roberta's shot visuals.
        {
            use neopets_sim::roberta as rb;
            let t8 = |at: u32| neopets_data::archive::boot_u32_table(boot, at, 8).map(|w| std::array::from_fn(|k| w[k] as i32)).unwrap_or([-1; 8]);
            self.attack_fx.shot_tables = rb::VisualTables {
                flight: rb::FLIGHT_AT.map(t8),
                full: t(rb::FULL_AT),
                aura: t(0x0031_53D0),
                impact_mote: t(rb::IMPACT_MOTE_AT),
                impact: rb::IMPACT_AT.map(t8),
            };
        }
        if std::env::var_os("NEOPETS_TRACE_FX").is_some() {
            eprintln!("[fx] tables {:?}", self.attack_fx.tables);
        }
    }

    /// SPEC-0034 3.17, once per pass for the active hero: the near-hand point N = position + 0.48
    /// Z0 and the tip T = position + R Z0 (Z0 attach point 0's third axis, R the weapon reach),
    /// both moved 0.2 toward the camera eye. Tor: the Mote weapon line (frame at N, target T),
    /// the tip light, and the swing glows 244, 243 and 246 (0.25, 0.2 and 0.25 s into sub-states
    /// 2, 3 and 4, at T, followed while the sub-state stays). Roberta: the Mote aura and charge-up
    /// effect at the hero, the spin glow 242 at T in sub-states 6 and 7. Both: on the first pass
    /// of sub-state 1 past the full-press threshold (50 Tor, 80 Roberta) the burst (detached; 243
    /// for Tor and 240 for Roberta with no Mote) and the charged-hold loop (N, target T; 245 for
    /// Roberta with no Mote) until sub-state 1 ends. Roberta's shot visuals: shot_fx_step.
    fn attack_fx_step(&mut self) {
        let table = self.effect_table.clone();
        let h = self.hero_index;
        let pos = self.loco.position();
        let m = self.equipment.motes[h.min(1)][2].map(|id| id as i32);
        let idx = m.and_then(|id| usize::try_from(id - 160).ok()).filter(|&k| k < 14);
        let (sub, f) = match h {
            0 => self.attack.as_ref().map_or((0, 0), |a| (a.sub, a.f)),
            _ => self.r_attack.as_ref().map_or((0, 0), |a| (a.sub, a.f)),
        };
        let charged = f > if h == 0 { 50 } else { 80 };
        let place = placement(pos, self.loco.facing);
        let frames = self.hero.bone_frames();
        let z0 = self.hero.bones().attach.first().and_then(|&(bone, angles, offset)| Some(unit(place.mul(frames.get(bone?)?).mul(&attach_rotation(offset, angles)).cols[2]))).unwrap_or(self.loco.facing);
        let eye = self.camera.eye;
        let toward = |p: [f32; 3]| add(p, scale(unit(sub_v(eye, p)), 0.2));
        let n = toward(add(pos, scale(z0, 0.48)));
        let t = toward(add(pos, scale(z0, self.weapon_reach())));
        let fw = unit([self.loco.facing[0], 0.0, self.loco.facing[2]]);
        let frame = [[fw[2], 0.0, -fw[0]], [0.0, 1.0, 0.0], fw];
        let fx = &mut self.attack_fx;
        let (effects, rng) = (&mut self.effects, &mut self.rng);
        let tb = fx.tables;
        let mut keep = |slot: &mut Option<(i32, usize)>, want: Option<i32>, at: [f32; 3], target: Option<[f32; 3]>| keep_fx(effects, &table, rng, frame, slot, want, at, target);
        if h == 0 {
            keep(&mut fx.line, idx.map(|k| tb.line[k]), n, Some(t));
            keep(&mut fx.light, idx.map(|k| tb.light[k]), t, None);
            keep(&mut fx.aura, None, pos, None);
            keep(&mut fx.charge, None, pos, None);
        } else {
            keep(&mut fx.line, None, n, None);
            keep(&mut fx.light, None, t, None);
            keep(&mut fx.aura, idx.map(|k| tb.aura[k]), pos, None);
            keep(&mut fx.charge, idx.filter(|_| !charged).map(|k| tb.charge[k]), pos, None);
        }
        // The full-charge burst and the charged-hold loop.
        if sub == 1 && charged && !fx.burst_done {
            fx.burst_done = true;
            let burst = idx.map_or(if h == 0 { 243 } else { 240 }, |k| tb.burst[h.min(1)][k]);
            let hold = idx.map_or(if h == 0 { -1 } else { 245 }, |k| tb.hold[h.min(1)][k]);
            keep(&mut fx.hold, Some(hold), n, Some(t));
            fx.burst = Some(burst);
        } else if sub == 1 && fx.burst_done {
            let id = fx.hold.map(|(id, _)| id);
            keep(&mut fx.hold, id, n, Some(t));
        } else if sub != 1 {
            fx.burst_done = false;
            keep(&mut fx.hold, None, n, None);
        }
        if let Some(burst) = fx.burst.take().filter(|&b| b >= 0) {
            self.effects.one_shot(&table, burst, pos, frame, &mut self.rng);
        }
        // The swing glows (Tor) and the spin glow (Roberta): on a sub-state change the timer and
        // the started flag reset and the old glow is released to play out; the glow starts once
        // its time is reached and follows T while the sub-state lasts.
        let glow = match (h, sub) {
            (0, 2) => Some((0.25, 244)),
            (0, 3) => Some((0.2, 243)),
            (0, 4) => Some((0.25, 246)),
            (1, 6 | 7) => Some((0.0, 242)),
            _ => None,
        };
        let fx = &mut self.attack_fx;
        if fx.glow_sub != sub {
            fx.glow_sub = sub;
            fx.glow_timer = 0.0;
            fx.glow_started = false;
            if let Some(hd) = fx.glow.take() {
                self.effects.release(hd);
            }
        }
        if let Some((at_time, id)) = glow {
            fx.glow_timer = fx.glow_timer + 1.0 / 60.0;
            if !fx.glow_started && fx.glow_timer >= at_time {
                fx.glow_started = true;
                let hd = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, id, t, frame));
                self.effects.update(&table, hd, None, &mut self.rng);
                fx.glow = Some(hd);
            }
            if let Some(hd) = fx.glow {
                self.effects.set_position(hd, t);
            }
        }
        // NEOPETS_TRACE_FX: the driver's roles whenever they change.
        if std::env::var_os("NEOPETS_TRACE_FX").is_some() {
            let fx = &self.attack_fx;
            let now = format!("mote {m:?} sub {sub} f {f} line {:?} light {:?} aura {:?} charge {:?} hold {:?} glow {:?}", fx.line, fx.light, fx.aura, fx.charge, fx.hold, fx.glow);
            if now != fx.trace {
                eprintln!("[fx] tick {}: {now}", self.ticks);
                self.attack_fx.trace = now;
            }
        }
    }

    /// Whether pool slot `k` draws flight effects (its marker is then not drawn).
    pub fn pool_slot_has_fx(&self, k: usize) -> bool {
        self.shot_fx.get(k).copied().flatten().is_some_and(|(_, h)| h.iter().any(Option::is_some))
    }

    /// SPEC-0034 Roberta's shot visuals, once per pass: each live shot's flight effects A and B
    /// follow it; a shot that has ended (by any route) stops them and plays C once at its last
    /// position.
    fn shot_fx_step(&mut self) {
        let table = self.effect_table.clone();
        let live: std::collections::HashMap<u32, ([f32; 3], [f32; 3])> = self.hero_shots.iter().map(|s| (s.id, (s.position, s.dir))).collect();
        let frame_of = flat_frame;
        let ids: Vec<u32> = self.attack_fx.shots.keys().copied().collect();
        for id in ids {
            let Some((v, mut handles, last)) = self.attack_fx.shots.get(&id).copied() else { continue };
            match live.get(&id) {
                Some(&(at, dir)) => {
                    let frame = frame_of(dir);
                    for (k, h) in handles.iter_mut().enumerate() {
                        if v[k] < 0 {
                            continue;
                        }
                        let hd = *h.get_or_insert_with(|| {
                            let hd = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, v[k], at, frame));
                            self.effects.update(&table, hd, None, &mut self.rng);
                            hd
                        });
                        self.effects.set_position(hd, at);
                    }
                    self.attack_fx.shots.insert(id, (v, handles, at));
                }
                None => {
                    for hd in handles.into_iter().flatten() {
                        self.effects.stop(&table, hd);
                        self.effects.release(hd);
                    }
                    if v[2] >= 0 {
                        self.effects.one_shot(&table, v[2], last, frame_of([0.0, 0.0, 1.0]), &mut self.rng);
                    }
                    if std::env::var_os("NEOPETS_TRACE_FX").is_some() {
                        eprintln!("[fx] tick {}: shot {id} ended at {last:?}; flight {:?}, impact {}", self.ticks, &v[..2], v[2]);
                    }
                    self.attack_fx.shots.remove(&id);
                }
            }
        }
    }

    /// The drawn placed objects that own ground patches, with their current and rest frames,
    /// for each resident world's probe (an object that is not drawn gives no patches).
    fn owner_pose_step(&mut self) {
        for other in [false, true] {
            let Some(w) = (if other { self.other.as_ref() } else { Some(&self.world) }) else { continue };
            let pack = w.pack;
            let owners: Vec<usize> = w.ground.as_ref().map_or(Vec::new(), |g| g.object_sets().into_iter().map(|(_, o)| o).collect());
            let poses: std::collections::HashMap<usize, neopets_data::ground::OwnerPose> = owners
                .into_iter()
                .filter_map(|i| {
                    let rest = w.objects.get(i)?.as_ref()?.rest;
                    Some((i, neopets_data::ground::OwnerPose { current: self.object_matrix(pack, i)?, rest }))
                })
                .collect();
            if let Some(w) = if other { self.other.as_mut() } else { Some(&mut self.world) } {
                w.owner_poses = poses;
            }
        }
    }

    /// A movable block's draw matrix (column-major): axes B, vertical and A, and its position.
    pub fn block_matrix(&self, pack: u32, i: usize) -> Option<[f32; 16]> {
        let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
        let b = w.blocks.get(i)?;
        let (x, y, z, p) = (b.b, b.up, b.a, b.position);
        Some([x[0], x[1], x[2], 0.0, y[0], y[1], y[2], 0.0, z[0], z[1], z[2], 0.0, p[0], p[1], p[2], 1.0])
    }

    /// A breakable's draw matrix while it is drawn: states 1 and 3 (SPEC-0048 Breakable draw).
    pub fn breakable_matrix(&self, pack: u32, i: usize) -> Option<[f32; 16]> {
        let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
        let b = w.breakables.get(i)?;
        matches!(b.state, 1 | 3).then_some(b.matrix)
    }

    /// SPEC-0048 Break routes, the hero melee route (SPEC-0036 Hit detection, step 6): each tick of
    /// the window, the swing triangle (P0 and the current and previous-tick tips) against every
    /// intact breakable's volume of the current world. A hit runs the element gate (the entry's
    /// class 0 or 7, or the slot-2 Mote's kind), the break effect from state 1 and the contact
    /// cue by material; on an element mismatch Tor recoils (SPEC-0048 Element recoil: attack
    /// sub-state 13 with material 4). The break's spawn by key is in `break_one_with`. The
    /// script hook, the piece spawns and the timed effect are not built.
    fn breakable_walk(&mut self, p0: [f32; 3], tip: [f32; 3], prev: [f32; 3]) {
        use neopets_sim::combat::{Cylinder, segment_distance, segment_meets};
        let kind = neopets_sim::mote::element(self.equipment.motes[0][2]);
        let edges = [(p0, tip), (p0, prev), (tip, prev)];
        let mut broken = Vec::new();
        let mut chained = Vec::new();
        let mut cues = Vec::new();
        let mut recoil = false;
        let mut seals = Vec::new();
        // The element-sealed chests (kind 2): (group, chest body breakable, seal breakable).
        let sealed: Vec<(usize, usize, usize)> = self
            .world
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| g.kind == 2)
            .filter_map(|(gi, g)| Some((gi, usize::try_from(*g.breakables.first()?).ok()?, usize::try_from(*g.breakables.get(1)?).ok()?)))
            .collect();
        // The chain hook: members 0 to n-2 of a kind-4 group (the group's index with it).
        let hooked: Vec<(usize, usize)> = self
            .world
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| g.kind == 4)
            .flat_map(|(gi, g)| g.breakables.iter().take(g.breakables.len().saturating_sub(1)).filter_map(move |&m| usize::try_from(m).ok().map(|m| (m, gi))))
            .collect();
        for (i, b) in self.world.breakables.iter_mut().enumerate() {
            let Some(v) = b.volume else { continue };
            if !matches!(b.state, 1 | 3) {
                continue;
            }
            let hit = match v.kind {
                1 => edges.iter().any(|&(a, c)| segment_distance(a, c, v.centre, v.centre) <= v.radius),
                3 => {
                    let cyl = Cylinder::from_base(v.centre, unit(v.axis), v.radius, v.height);
                    edges.iter().any(|&(a, c)| segment_meets(a, c, &cyl))
                }
                4 | 7 => edges.iter().any(|&(a, c)| segment_box(a, c, v.centre, v.axes, v.extents)),
                _ => false,
            };
            if !hit {
                continue;
            }
            // SPEC-0048 Element gate on hits: the chest body's hook only recoils Tor; the seal's
            // hook passes when its element is 0, matches the slot-2 Mote's kind, or the Mote is
            // Nova (7), and opens the chest; otherwise Tor recoils.
            if let Some(&(gi, body, seal)) = sealed.iter().find(|s| s.1 == i || s.2 == i) {
                let e_t = b.element as i32;
                if i == seal && (e_t == 0 || kind == e_t || kind == 7) {
                    if b.state == 1 && !seals.contains(&gi) {
                        seals.push(gi);
                    }
                } else if i == body || i == seal {
                    recoil = true;
                }
                continue;
            }
            let e = b.element as i32;
            if !(e == 0 || e == 7 || e == kind) {
                // A mismatch: Tor recoils with material 4, with no break and no cue.
                recoil = true;
                if std::env::var_os("NEOPETS_TRACE_BREAK").is_some() {
                    println!("      breakable {i}: element {e} against Mote kind {kind}: recoil");
                }
                continue;
            }
            if let Some(&(_, gi)) = hooked.iter().find(|h| h.0 == i) {
                // The hook replaces the contact route: no cue, no push.
                if !chained.contains(&gi) {
                    chained.push(gi);
                }
                continue;
            }
            if b.state == 1 {
                broken.push(i);
            } else {
                cues.push(break_cue(b.material));
            }
        }
        for gi in seals {
            self.seal_open(gi);
        }
        if recoil && self.hero_index == 0 {
            if let Some(a) = self.attack.as_mut() {
                a.recoil();
            }
        }
        for gi in chained {
            self.chain_hook(gi);
        }
        for i in broken {
            self.break_one(i);
        }
        for c in cues {
            self.cue(c);
        }
    }

    /// SPEC-0048 Element gate on hits, Open: the sealed chest's sequence as a script thread (the
    /// waits in seconds as ticks): the body made active, the seal broken, effect 0 stopped, the
    /// weapon put away; 0.3 s, camera 0 (waited for), cue 182; 1.3 s, object 0 hidden and object
    /// 1 shown and looped; 2.0 s, the display mask stepped 0x10000, 0x20000, 0x40000 (0.8 s each)
    /// and back to 0; effect 791 at X + 2.9 d, the body broken, object 1 hidden and object 0
    /// shown, hero animation 192; 3.3 s, camera 1, hero animation 186; the reward (the drop of
    /// pair 0, as a lock chest's) and the key row; 2.0 s and the normal camera. Not built: the
    /// HUD reset, the reward's 2.2 d launch point (the lock chest's 2.4 d is used) and pair 1's
    /// character.
    fn seal_open(&mut self, gi: usize) {
        let Some(g) = self.world.groups.get(gi).cloned() else { return };
        let b = |k: usize| g.breakables.get(k).and_then(|&b| usize::try_from(b).ok());
        let o = |k: usize| g.objects.get(k).and_then(|&o| usize::try_from(o).ok());
        let camera = |k: usize| g.cameras.get(k).and_then(|&c| usize::try_from(c).ok());
        let (x, d) = g.locators.first().and_then(|&k| usize::try_from(k).ok()).and_then(|k| self.world.entries.get(k).copied()).unwrap_or((self.loco.position(), self.loco.facing));
        let fx = [x[0] + 2.9 * d[0], x[1] + 2.9 * d[1], x[2] + 2.9 * d[2]];
        let secs = |s: f32| Action::Sleep { ticks: (s * 60.0).round() as u32 };
        let mut v: Vec<Action> = Vec::new();
        v.extend(b(0).map(|id| Action::SetBreakableActive { id, on: true }));
        v.extend(b(1).map(|id| Action::Break { id }));
        v.extend(g.effects.first().and_then(|&e| usize::try_from(e).ok()).map(|slot| Action::StopEffect { slot }));
        v.extend([Action::Equip { on: false }, secs(0.3)]);
        v.extend(camera(0).map(|track| Action::PlayCamera { track, wait: true, end: 1, speed: 1.0 }));
        v.extend([Action::MainCue { cue: 182 }, secs(1.3)]);
        v.extend(o(0).map(|obj| Action::DeactivateObj { obj }));
        v.extend(o(1).map(|obj| Action::InitObjOn { obj, end: false }));
        v.extend(o(1).map(|obj| Action::LoopAnim { obj, speed: 1.0 }));
        v.push(secs(2.0));
        // SetWorldDisplayMask stores twice its argument; the raw masks are passed halved.
        for m in [0x10000u32, 0x20000, 0x40000] {
            v.extend([Action::DisplayMask { mask: m / 2 }, secs(0.8)]);
        }
        v.push(Action::DisplayMask { mask: 0 });
        v.push(Action::EffectAt { id: 791, at: fx, dir: d });
        v.extend(b(0).map(|id| Action::Break { id }));
        v.extend(o(1).map(|obj| Action::DeactivateObj { obj }));
        v.extend(o(0).map(|obj| Action::InitObjOn { obj, end: false }));
        v.extend([Action::HeroAnim { anim: 192, wait: false }, secs(2.3), secs(1.0)]);
        v.extend(camera(1).map(|track| Action::PlayCamera { track, wait: false, end: 1, speed: 1.0 }));
        v.extend([Action::HeroAnim { anim: 186, wait: false }, Action::ChestReward { group: gi as u16 }, secs(2.0), Action::ReturnCamera { preset: -1, release: true }]);
        self.last_door = format!("sealed chest group {gi} opening on tick {}", self.ticks);
        self.apply(&v);
    }

    /// SPEC-0048 Switch groups, the chain hook of kind-4 entry `gi`: the first member in state 1
    /// or 3 breaks when it is in state 1 (no cue), and the next member is shown in state 3 and
    /// becomes breakable 24 ticks later.
    fn chain_hook(&mut self, gi: usize) {
        let Some(members) = self.world.groups.get(gi).map(|g| g.breakables.clone()) else { return };
        let members: Vec<usize> = members.into_iter().filter_map(|m| usize::try_from(m).ok()).collect();
        let Some(k) = members.iter().position(|&m| self.world.breakables.get(m).is_some_and(|b| matches!(b.state, 1 | 3))) else { return };
        if self.world.breakables[members[k]].state == 3 {
            return;
        }
        self.break_one_with(members[k], false);
        if let Some(&next) = members.get(k + 1) {
            if let Some(b) = self.world.breakables.get_mut(next) {
                b.state = 3;
                self.chain_arms.push((self.world.pack, next, self.ticks + 24));
            }
        }
    }

    /// The break of breakable i of the current world from state 1 (SPEC-0048 Break effect): state
    /// 2, the drop action for its placement (no luck gate), the spawn by key (step 3) and the
    /// contact cue by material.
    /// Diagnostic: break breakable i of the current world when it is intact (trace only).
    pub fn diag_break(&mut self, i: usize) {
        if self.world.breakables.get(i).is_some_and(|b| b.state == 1) {
            self.break_one(i);
        }
    }

    fn break_one(&mut self, i: usize) {
        self.break_one_with(i, true);
    }

    fn break_one_with(&mut self, i: usize, with_cue: bool) {
        let pack = self.world.pack;
        let Some(b) = self.world.breakables.get_mut(i) else { return };
        b.state = 2;
        let (at, drop, cue) = (b.position(), b.drop, break_cue(b.material));
        let (key, facing) = (b.spawn_key, unit([b.matrix[8], b.matrix[9], b.matrix[10]]));
        // The drop id is a placement id: entry p of the placement-id table (tag 0x13) names the
        // block and the ordinal among its item records (SPEC-0006, SPEC-0048 Placed drops on
        // break). Only the embedded block (0xFF) is resolved.
        let point = usize::try_from(drop)
            .ok()
            .and_then(|d| self.world.placements.get(d))
            .filter(|(region, _)| *region == 0xFF)
            .and_then(|&(_, ord)| self.world.item_points.get(ord as usize).copied());
        if point.is_some() {
            self.payouts.push((pack, at, point, 0));
        }
        // Step 3: a spawn key of 0 or more runs a world-level spawn by key at the break point
        // with the entry's facing, the arrival flag set (SPEC-0029 Behavior 31).
        if key >= 0 {
            self.summon(pack, None, key, at, facing, None, true);
            if std::env::var_os("NEOPETS_TRACE_BREAK").is_some() {
                println!("      breakable {i} spawn key {key}: {}", self.last_summon);
            }
        }
        self.last_pickup = format!("breakable {i} of {pack} broken on tick {}", self.ticks);
        if std::env::var_os("NEOPETS_TRACE_BREAK").is_some() {
            println!("      breakable {i} of {pack} broken at {at:?} on tick {} (drop {drop})", self.ticks);
        }
        if with_cue {
            self.cue(cue);
        }
    }

    pub fn object_matrix(&self, pack: u32, i: usize) -> Option<[f32; 16]> {
        let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
        let obj = w.objects.get(i)?.as_ref()?;
        // NEOPETS_HIDE_OBJ="a-b": objects a to b are never drawn (diagnostic).
        if std::env::var("NEOPETS_HIDE_OBJ").ok().and_then(|v| {
            let (a, b) = v.split_once('-')?;
            Some((a.trim().parse::<usize>().ok()?..=b.trim().parse::<usize>().ok()?).contains(&i))
        }) == Some(true)
        {
            return None;
        }
        // SPEC-0028 section 7: an object whose zone query fails is not drawn.
        if !self.zone_visible(pack, obj.zone) {
            return None;
        }
        let a = w.object_anims.get(i)?;
        // An object a type-40 group names (the group kinds whose setup is not built) is also
        // drawn at rest until a script touches it. A lock chest's set-up poses only its object 0
        // (SPEC-0048 Set-up poses by kind), so its other objects keep the disc state 3 and are
        // not drawn (SPEC-0028 Draw gate) until a script turns them on.
        // In every pack, an object of a group whose set-up is built (kinds 1, 5 and 6) that the
        // set-up did not pose keeps the disc state 3 and is not drawn until a script turns it on
        // (a lock chest's lid block, a dig site's dug pieces).
        if !a.scripted && w.groups.iter().any(|g| matches!(g.kind, 1 | 2 | 5 | 6) && g.objects.contains(&(i as i32))) {
            return None;
        }
        let grouped = w.groups.iter().any(|g| g.objects.contains(&(i as i32)) && !(g.kind == 1 && g.objects.first() != Some(&(i as i32))));
        let gated = level::objects_scripted(pack) || (pack == 1217 && neopets_sim::burrows::named(i));
        // The 758 wheel turns even while no script has touched it (its pack's objects are drawn
        // at rest).
        if !a.scripted && (!gated || grouped) && !(pack == 758 && i == 0) {
            return Some(obj.rest);
        }
        // SPEC-0059 Layout: the 1358 and 758 wheels are object 0 turned to A about locator 1's
        // (1358) or locator 4's (758) forward axis (through the object's origin).
        if matches!(pack, 1358 | 758) && i == 0 && (a.visible() || (pack == 758 && !a.scripted)) {
            if let Some((_, f)) = w.entries.get(if pack == 758 { 4 } else { 1 }).copied() {
                let k = unit(f);
                let angle = level::wheel_of(pack, &self.state).angle.to_radians();
                let (sn, cs) = angle.sin_cos();
                let m = obj.rest;
                let turn = |v: [f32; 3]| {
                    let d = k[0] * v[0] + k[1] * v[1] + k[2] * v[2];
                    let c = [k[1] * v[2] - k[2] * v[1], k[2] * v[0] - k[0] * v[2], k[0] * v[1] - k[1] * v[0]];
                    std::array::from_fn::<f32, 3, _>(|j| v[j] * cs + c[j] * sn + k[j] * d * (1.0 - cs))
                };
                let (x, y, z) = (turn([m[0], m[1], m[2]]), turn([m[4], m[5], m[6]]), turn([m[8], m[9], m[10]]));
                return Some([x[0], x[1], x[2], 0.0, y[0], y[1], y[2], 0.0, z[0], z[1], z[2], 0.0, m[12], m[13], m[14], 1.0]);
            }
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
        let lift = self.obj_offset.get(&(pack, i)).copied().unwrap_or(0.0);
        let base = if w.obj_links.contains_key(&i) { self.obj_base(pack, i, 0) } else { [obj.rest[12], obj.rest[13], obj.rest[14]] };
        let t = [base[0] + root[0], base[1] + root[1] + lift, base[2] + root[2]];
        let c = |k: usize| r[k].map(|v| v * scale[k]);
        let (mut c0, c1, mut c2) = (c(0), c(1), c(2));
        // SetAnimObjAdditionRotY: the added yaw turns the frame about its own Y axis.
        if let Some(&deg) = self.obj_yaw.get(&(pack, i)) {
            let (sn, cs) = deg.to_radians().sin_cos();
            let (x, z) = (c0, c2);
            c0 = std::array::from_fn(|k| cs * x[k] - sn * z[k]);
            c2 = std::array::from_fn(|k| sn * x[k] + cs * z[k]);
        }
        Some([c0[0], c0[1], c0[2], 0.0, c1[0], c1[1], c1[2], 0.0, c2[0], c2[1], c2[2], 0.0, t[0], t[1], t[2], 1.0])
    }

    /// The id the status displays use for the selected hero.
    const HERO_DISPLAY: u64 = u64::MAX;

    /// The hero's status apply from a hit, a shot or a script, with the immunity block's
    /// one-shot (SPEC-0036 Status display and cues) at the hero's origin (R-0045).
    fn hero_status(&mut self, code: i32, seconds: i32) -> bool {
        if let Some(fx) = self.statuses.immunity_block(code) {
            let table = self.effect_table.clone();
            self.effects.one_shot(&table, fx, self.loco.position(), neopets_sim::effect::IDENTITY, &mut self.rng);
        }
        self.statuses.apply(code, seconds)
    }

    /// SPEC-0036 Status display and cues for the hero and the common enemies: a status that
    /// became active this tick runs the display start (codes 8 to 10 play their one-shot), the
    /// rotation steps, the shown status's effect follows the character (curse 1.5 above), and a
    /// stun that ended plays 0xCD. The effects sit at the character's origin: the main attach
    /// point is not identified (R-0045).
    fn status_display_step(&mut self) {
        let table = self.effect_table.clone();
        let mut who: Vec<(u64, [bool; 14], [f32; 3])> = vec![(Self::HERO_DISPLAY, std::array::from_fn(|c| self.statuses.active(c)), self.loco.position())];
        for c in &self.characters {
            if let Some(e) = c.enemy.as_ref().filter(|e| !e.is_dead() && !c.inactive) {
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
        // NEOPETS_SKIP_FX="a,b": these placed records are never started (diagnostic).
        if std::env::var("NEOPETS_SKIP_FX").is_ok_and(|v| v.split(',').any(|t| t.trim().parse::<usize>().ok() == Some(slot))) {
            return;
        }
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
        // Independently, low magic: cue 158 while the magic pool is below 0.15 of its maximum
        // (SPEC-0036 Low-health warning; the same hero and the same do-not-restart rule).
        if self.hero_mp_max > 0 && (self.hero_mp as f32) < 0.15 * self.hero_mp_max as f32 {
            self.sound.play(snd::MAIN, 158, Play { do_not_restart: true, ..Default::default() });
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
        if std::env::var_os("NEOPETS_TRACE_SLOTS").is_some() && self.ticks % 100 == 0 {
            println!("      sound slots {:?}; kinds {:?}", self.sound.slots, self.characters.iter().filter(|c| c.pack == self.world.pack).map(|c| (c.pet_type, c.sound_blocks.clone())).collect::<Vec<_>>());
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
            if let Some((kind, v)) = self.design.flag_cue(n) {
                if v >= 1 {
                    self.stinger(v - 1);
                }
                // SPEC-0023 section 8: a flag with a journal record sets the notification.
                if kind != 0 {
                    self.hud.notify_journal();
                }
            }
        }
        // Intensity: k counts the characters in AI state 19 (agro) that are not dead.
        let k = self.characters.iter().filter(|c| c.enemy.as_ref().is_some_and(|e| e.state == 19 && !e.is_dead())).count();
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

    /// Map area m's name (design table 5), for the developer menu's warp list.
    pub fn area_name(&self, m: i32) -> Option<String> {
        let m = usize::try_from(m).ok()?;
        self.design.location_name(m).map(|n| String::from_utf8_lossy(&n).into_owned())
    }

    fn world_index(&self, pack: u32) -> usize {
        neopets_sim::save::world_index(&self.world_keys.0, &self.world_keys.1, pack as i32)
    }

    /// Rebuild a slot from the live game (SPEC-0025 Saving, step 3): both heroes' pools and
    /// maxima, the equipped Motes, the party and the stash, the petpet, the sets and the live
    /// options. The parked hero's status timers are written inactive, the status words -1, and
    /// the per-world tables and count table empty (no live state for them yet).
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
        // The played words: one u32 per movie index, 1 once the movie has played (SPEC-0026).
        s.0[at::MOVIES..at::MOVIES + 368].fill(0);
        for &m in self.state.movies.iter().filter(|&&m| (m as usize) < 92) {
            s.set_u32(at::MOVIES + 4 * m as usize, 1);
        }
        s.set_i32(at::WALLET, self.state.neopoints);
        s.set_i32(at::STASH, self.state.stash_neopoints);
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
        // Container B (the stash) by each item's group, in id order.
        let mut stash_lists: HashMap<Group, Vec<(u16, u32)>> = HashMap::new();
        let mut stashed: Vec<(&u16, &u32)> = self.state.stash.iter().filter(|(_, n)| **n > 0).collect();
        stashed.sort();
        for (&id, &n) in stashed {
            let g = self.design.item_category(id).and_then(Group::of_category).unwrap_or(Group::General);
            stash_lists.entry(g).or_default().push((id, n));
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
                match stash_lists.get(&g).and_then(|l| l.get(i)) {
                    Some(&(id, n)) => s.set_entry(1, g, i, Some(id), n as i32),
                    None => s.set_entry(1, g, i, None, 0),
                }
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
            // SPEC-0025 Equipment Mote words: armour, shield and weapon Mote of hero h.
            for i in 0..3 {
                s.set_mote(h, i, self.equipment.motes[h][i].map_or(-1, i32::from));
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
    /// placement at vectors A and B, the options with their apply rule, the stash, the petpet,
    /// both heroes' pools, the swap mode and the equipped Motes. Not applied: Roberta's status
    /// timers and status words, the per-world tables and the count table.
    pub fn apply_slot(&mut self, s: &neopets_sim::save::Slot) {
        use neopets_sim::save::{self as sv, Group, at};
        self.state.flags.clear();
        for n in 0..sv::FLAGS {
            if s.flag(n) {
                self.state.flags.insert(n);
            }
        }
        self.state.movies = (0..92u16).filter(|&m| s.u32(at::MOVIES + 4 * m as usize) != 0).collect();
        self.state.movie_queue.clear();
        self.state.items.clear();
        self.weapon = None;
        self.shield = None;
        self.state.neopoints = s.i32(at::WALLET);
        self.state.stash.clear();
        self.state.stash_neopoints = s.i32(at::STASH);
        for (c, g, i, id, v) in s.adds() {
            let Ok(id) = u16::try_from(id) else { continue };
            if c != 0 {
                // Container B, the stash: every entry by its count.
                *self.state.stash.entry(id).or_insert(0) += v as u32;
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
        // Loading step 12g: each hero's three Mote words through the equip setter with no cue;
        // a changed word re-arms its equipment-panel timer (240) for the active hero. Tor's Mote
        // follow-up (Medium) is not run.
        for h in 0..2 {
            for i in 0..3 {
                let changed = match u16::try_from(s.mote(h, i)) {
                    Ok(id) => self.equipment.set_mote(h, i, id),
                    Err(_) => self.equipment.clear_mote(h, i),
                };
                if changed && h == self.hero_index {
                    self.hud.rearm_word(i);
                }
            }
        }
        let (a, b) = (s.vec4(at::POSITION), s.vec4(at::FACING));
        let pose = ([a[0], a[1], a[2]], [b[0], b[1], b[2]]);
        self.restart = Some(pose);
        self.arrive(pose);
        // SPEC-0055 2A step 6 / SPEC-0025 Loading: the selected hero arrives in state 59 (clip
        // 175, the arrival pose, to progress 0.99, then state 1); it reads no stick meanwhile.
        self.loco.set_end_state(hero_states::ARRIVAL);
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
    /// Developer menu: every living enemy of the current world takes a defeating hit (bosses that
    /// refuse damage keep refusing it). Returns how many were hit.
    pub fn dev_kill_enemies(&mut self) -> usize {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let targets: Vec<usize> = (0..self.characters.len())
            .filter(|&i| {
                let c = &self.characters[i];
                let flyer = c.enemy.is_none() && c.hp > 0 && c.row.is_some_and(|r| r.class == Some(17) && r.can_fly != 0);
                c.pack == pack && !c.inactive && c.dying.is_none() && (c.enemy.as_ref().is_some_and(|e| !e.is_dead()) || flyer)
            })
            .collect();
        for &i in &targets {
            self.character_take_hit(i, 1_000_000, hero, "the developer menu ");
        }
        // The Gelert Assassin (not a clone): HP to 0 (its death runs on its next free tick).
        let mut gelerts = 0;
        for c in self.characters.iter_mut().filter(|c| c.pack == pack && c.gelert.as_ref().is_some_and(|g| !g.clone)) {
            c.hp = 0;
            gelerts += 1;
        }
        // The Black Knight: its six weak points to 0 (its defeat test then runs).
        let mut knights = 0;
        for c in self.characters.iter_mut().filter(|c| c.pack == pack && !c.inactive) {
            if let Some(k) = c.knight.as_mut().filter(|k| !k.defeated()) {
                k.parts = [0; 6];
                knights += 1;
            }
        }
        // The Undead Draik: straight to its death state (the death animation, then level
        // event 12).
        let mut bosses = 0;
        for c in self.characters.iter_mut().filter(|c| c.pack == pack && !c.inactive) {
            if let Some(d) = c.draik.as_mut().filter(|d| d.stage < 6 && d.state != neopets_sim::draik::State::Dead) {
                d.state = neopets_sim::draik::State::Dead;
                c.hp = 0;
                bosses += 1;
            }
        }
        // The Faerie Sisters: each to its death state (the group's end then runs).
        for c in self.characters.iter_mut().filter(|c| c.pack == pack && !c.inactive) {
            if let Some(sis) = c.sister.as_mut().filter(|s| s.state != neopets_sim::sisters::State::Dead) {
                sis.state = neopets_sim::sisters::State::Dead;
                c.hp = 0;
                bosses += 1;
            }
        }
        targets.len() + knights + gelerts + bosses
    }

    /// Developer menu: the current world's start points, and a move to start point k.
    pub fn dev_start_points(&self) -> usize {
        self.world.entries.len()
    }

    pub fn dev_go_start(&mut self, k: usize) -> bool {
        let Some(&pose) = self.world.entries.get(k) else { return false };
        self.arrive(pose);
        true
    }

    /// Developer menu: a 141 NP coin split (one coin of each denomination) and the chosen item,
    /// dropped as a drop action would drop them, 6 units in front of the hero. Returns how many
    /// world items were spawned.
    pub fn dev_drop(&mut self, item: u16) -> usize {
        let pack = self.world.pack;
        let p = self.loco.position();
        let f = self.loco.facing;
        let at = [p[0] + 6.0 * f[0], p[1] + 1.0, p[2] + 6.0 * f[2]];
        let spawn = |id: u16, charm: bool| neopets_sim::items::Spawn {
            id,
            point: at,
            placed: false,
            charm,
            s0: 0.25,
            s1: 1.0,
            delay: 0.0,
            larg: -1.0,
            direction: None,
            flag: 0,
            map: 0,
            price: 0,
        };
        let mut n = 0;
        for id in neopets_sim::items::coin_split(141, &self.coins.0, &self.coins.1) {
            n += self.spawn_item(pack, spawn(id, false)) as usize;
        }
        let charm = self.item_kind(item) == 17;
        n + self.spawn_item(pack, spawn(item, charm)) as usize
    }

    /// Developer menu: both heroes to full health and magic.
    pub fn dev_heal(&mut self) {
        self.hero_hp = self.hero_max;
        self.hero_mp = self.hero_mp_max;
        if let Some(p) = self.parked.as_mut() {
            p.hp = p.hp_max;
            p.mp = p.mp_max;
        }
    }

    /// Developer menu: an item's name from the item table, or "(no name)".
    /// Diagnostic: each named item's id, name, text category and carry limit.
    pub fn item_limits(&self) -> Vec<String> {
        (0..181u16).filter_map(|id| Some(format!("item {id} {} cat {:?} limit {:?}", String::from_utf8_lossy(&self.design.item_name(id)?), self.design.item_category(id), self.design.carry_limit(id)))).collect()
    }

    pub fn item_label(&self, id: u16) -> String {
        self.design.item_name(id).map_or_else(|| "(no name)".to_string(), |n| String::from_utf8_lossy(&n).into_owned())
    }

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
    /// Diagnostic: the current world's book stations (value, trigger index).
    /// The current world's script triggers: (index, kind, a middle point) for traces.
    pub fn trigger_points(&self) -> Vec<(u16, u8, [i32; 3])> {
        use neopets_sim::trigger::Shape;
        self.world
            .all_volumes
            .iter()
            .map(|v| {
                let c = match v.shape {
                    Shape::Sphere { centre, .. } => centre,
                    Shape::Cylinder { base, axis, length, .. } => [base[0] + 0.5 * length * axis[0], base[1] + 0.5 * length * axis[1], base[2] + 0.5 * length * axis[2]],
                    Shape::Rectangle { origin, axes, extents } => std::array::from_fn(|k| origin[k] + 0.5 * (extents[0] * axes[0][k] + extents[1] * axes[1][k])),
                };
                (v.index, v.kind, c.map(|x| x.round() as i32))
            })
            .collect()
    }

    pub fn book_stations(&self) -> Vec<(u16, u16)> {
        self.world.books.iter().map(|(v, value)| (*value, v.index)).collect()
    }

    /// Book i's directory entry in resource 576 (the books' path record; SPEC-0024 Book index).
    pub fn book_entry(&self, i: usize) -> Option<(u32, u32)> {
        self.design.paths.get(2)?.get(i).copied()
    }

    /// The map page's and the journal's tables (SPEC-0051 Layout): the quest rows (table 1),
    /// the treasure rows (table 3) and the area names (table 5), with their texts.
    /// The map's tables: quest rows (table 1), treasure rows (3), area names, and the area id of
    /// each collected-table row (table 6, 248 rows; SPEC-0051 section 3).
    #[allow(clippy::type_complexity)]
    pub fn map_tables(&self) -> (Vec<neopets_sim::mappage::Quest>, Vec<neopets_sim::mappage::Treasure>, Vec<Vec<u8>>, Vec<i32>) {
        let t = &self.design.tables;
        let text = |k: usize| move |p: u32| t.get(k).and_then(|tb| tb.string(p)).map(<[u8]>::to_vec).unwrap_or_default();
        let rows = |k: usize| t.get(k).map(|tb| (0..tb.count).filter_map(|i| tb.record(i)).collect::<Vec<_>>()).unwrap_or_default();
        let quests = rows(1).into_iter().filter_map(|r| neopets_sim::mappage::Quest::read(r, &text(1))).collect();
        let treasures = rows(3).into_iter().filter_map(|r| neopets_sim::mappage::Treasure::read(r, &text(3))).collect();
        let names = (0..neopets_sim::mappage::AREAS).map(|m| self.design.location_name(m).unwrap_or_default()).collect();
        let areas = rows(6).into_iter().map(|r| r.get(0..4).map_or(-1, |b| i32::from_le_bytes(b.try_into().unwrap()))).collect();
        (quests, treasures, names, areas)
    }

    /// The map and journal's view of the game this pass. The hero map area is the current
    /// world's, else the transition map (SPEC-0051 section 1), else -1.
    pub fn map_view(&self) -> crate::menus::MapView {
        crate::menus::MapView {
            hero_area: self.world.map_area.or(self.state.transition_map).unwrap_or(-1),
            hero: self.loco.position(),
            facing: self.loco.facing,
            flags: self.state.flags.iter().copied().collect(),
            maps: self.state.maps.iter().copied().collect(),
            collected: self.state.collected.iter().copied().collect(),
        }
    }

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
            Command::UseItem(id) => {
                // SPEC-0021 Use-request gate item 3: the item screen plays the refusal cue for a
                // diverted use.
                if level::on_hero_use_item(self.world.pack, &mut self.state.clone()).is_some() {
                    self.cue(0xA2);
                }
                self.use_item(id)
            }
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
            Command::Cloak => {
                self.cloak_worn = !self.cloak_worn;
                self.state.cloak_worn = self.cloak_worn;
            }
            _ => {}
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
                Some(t) => format!("{} [mode {} id {} at {:.0},{:.0}]", c.name, t.mode, c.anim.action(), c.position[0], c.position[2]),
                None if c.enemy.is_some() => {
                    let e = c.enemy.as_ref().unwrap();
                    let what = if e.is_dead() { "dead".to_string() } else { format!("state {} outer {} inner {} seq {}", e.state, e.outer, e.inner, e.seq) };
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
                    Some(r) => format!(
                        "{} [class {} hp {}/{}{} at {:.0},{:.1},{:.0}{}]",
                        if c.name.is_empty() { "(unnamed)" } else { &c.name },
                        r.class.unwrap_or(-1),
                        c.hp,
                        r.hit_points,
                        c.ixi.as_ref().map_or(String::new(), |k| format!(" ixi {:?}", k.state)) + &c.wild.as_ref().map_or(String::new(), |w| format!(" wild type {} state {} fed {} anim {}", c.pet_type, w.state, w.fed, c.anim.action())) + &c.class20.as_ref().map_or(String::new(), |k| format!(" class20 state {} anim {}", k.state, c.anim.action())),
                        c.position[0],
                        c.position[1],
                        c.position[2],
                        if c.inactive { " inactive" } else if c.held { " held" } else { "" }
                    ),
                    None => c.name.clone(),
                },
            })
            .collect();
        names.dedup();
        if self.shots.live() > 0 {
            names.push(format!("enemy shots {}", self.shots.live()));
        }
        if let Some(pool) = self.pools.get(&self.world.pack).filter(|p| p.live() > 0) {
            let ids: Vec<String> = pool.slots.iter().flatten().map(|i| format!("{} at {:.0},{:.1},{:.0}", i.id, i.position[0], i.position[1], i.position[2])).collect();
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

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    a.map(|c| c * s)
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}

/// The rescale of a stowed hero item (SPEC-0046 Held items): the shield's axes by 1.6 / |bx -
/// sx| and the sheathed weapon's by min(2.6 / |bz - sz|, 0.6), from the model's box minimum and
/// size along X or Z.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HeroItemScale {
    None,
    Shield,
    Weapon,
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
        counter_trigger: s.counter_trigger,
        counter_chance: s.counter_chance,
        steal_min: s.steal_min,
        steal_max: s.steal_max,
        steal_selector: s.steal_selector,
        steal_chance: s.steal_chance,
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
        fly_turn: t.fly_turn,
        fly_idle: t.fly_idle,
        fly_agro: t.fly_agro,
        ground_max: t.ground_max,
        ground_min: t.ground_min,
        max_fly: t.max_fly,
        search_max: t.search_max,
        search_min: t.search_min,
        attack_timer_max: t.attack_timer_max,
        attack_timer_min: t.attack_timer_min,
        approach_timer_max: t.approach_timer_max,
        approach_timer_min: t.approach_timer_min,
        turn: t.turn,
        distance_hide: t.distance_hide,
        reanimate: t.reanimate,
        weight: t.weight,
        standoff: t.standoff,
        hide_distance: t.hide_distance,
        hide_chance: t.hide_chance,
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
        defence: enemy::Defence { hold: t.defend_time, missile: t.missile_defence, block: t.melee_block, dodge: t.melee_dodge, exit: t.defence_timer, angle: t.blocking_angle },
        map_search: t.map_search.map(|m| enemy::MapSearch { chance: m.chance, timer: m.timer, kind: m.kind, greedy: m.greedy }),
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
        hero_swing: None,
        hero_recover: false,
        l1_held: false,
        missile_threat: false,
        hero_shot_live: false,
        spawn_bits: 0,
        radii: 0.0,
        clip: enemy::Clip { action: clip.action, progress: clip.progress, finished: clip.finished },
        has,
        waits: false,
        alarm_block: false,
        alarm_serial: 0,
        clear: true,
        items: &[],
        locate: None,
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
    // SPEC-0033 Behavior 7: the camera follows the drawn copy, whose height lags the world
    // height (so the 0.1 ground-constraint steps on a slope do not shake it).
    let shown = [p[0], loco.follow.shown, p[2]];
    Followed { position: shown, facing: loco.facing, height: HERO_HEIGHT, moving: loco.moved(), ground_kind }
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

/// The hit flash: 14 frames (a 4-frame and a 10-frame stage) with colour-override slot 1 at
/// (1.5, 0.5, 0.5) (SPEC-0046 Per-frame draw order, step 6).
pub const HIT_FLASH: u8 = 14;
pub const FLASH_COLOUR: [[f32; 3]; 15] = [[1.5, 0.5, 0.5]; 15];

/// The colour slot vectors of a record's 15 colour words: (byte 0, byte 1, byte 2) / 255.
/// The block slot of the Cloud Beast test spawn (no region load uses it).
const CLOUD_SLOT: u8 = 240;
/// Where a developer-menu boss fight puts the hero after the load.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    /// The world's entry record (the load's own arrival).
    Entry(usize),
    /// The middle of the volume of this trigger index (its fight starts on contact).
    Trigger(u16),
}

/// A developer-menu boss fight (this build's tool, not part of the game): the world, the
/// start-point index whose stages set flags and grant items as the game's own chooser does
/// (SPEC-0035 start-point confirm; SPEC-0049 and SPEC-0024 tables 11 and 12), the walkthrough
/// flags between that checkpoint and the fight (SPEC-0058, the step cited), gear the stages do
/// not grant, the hero, and where the hero goes.
#[derive(Clone, Copy, Debug)]
pub struct BossStart {
    pub name: &'static str,
    pub pack: u32,
    pub start: usize,
    pub flags: &'static [u16],
    pub items: &'static [u16],
    pub roberta: bool,
    pub place: Place,
    pub cloud: bool,
}

/// SPEC-0035 start-point confirm: the stage ids each start index sets (index 0 sets none).
pub const START_STAGES: [&[u16]; 9] = [&[], &[0], &[0, 1], &[0, 1, 2], &[4], &[4, 5], &[0, 1, 3], &[0, 1, 3, 6], &[0, 1, 2, 7]];

/// The fights. Act 1 fights take the enlistment gear of step 7 (sword 99, armour 120).
/// The developer test stage's pack number (none on the disc uses it).
pub const STAGE_PACK: u32 = 90_000;

pub const BOSSES: &[BossStart] = &[
    // Step 9: the Act 1 Juppie fight is due with 145 set and 82 and 83 clear; trigger 0.
    BossStart { name: "Juppie (Act 1)", pack: 1217, start: 0, flags: &[1, 2, 17, 143, 145], items: &[99, 120], roberta: false, place: Place::Trigger(0), cloud: false },
    // Steps 13 to 15: Cogham's raiders, then the raider leader's staged fight (trigger 1, 55 clear).
    BossStart { name: "Raider leader (Act 1)", pack: 800, start: 5, flags: &[193, 194, 195, 197, 198, 199], items: &[99, 120], roberta: false, place: Place::Trigger(1), cloud: false },
    // Steps 16 to 19: the glade freed (211), the load to 985 needs 212 on and 214 off; 281 clear.
    BossStart { name: "Gnarfas", pack: 985, start: 5, flags: &[193, 194, 195, 197, 198, 199, 200, 55, 56, 201, 202, 203, 151, 204, 205, 29, 206, 208, 209, 210, 623, 211, 212, 213], items: &[99, 120], roberta: false, place: Place::Entry(0), cloud: false },
    // Steps 28 to 33: Brightvale and Bogshot; trigger 23 starts the fight (263 clear).
    BossStart { name: "Plague Serpent", pack: 735, start: 2, flags: &[249, 250, 68, 251, 259, 266, 260, 264, 76], items: &[], roberta: false, place: Place::Trigger(23), cloud: false },
    // Step 35: 296 and 297 on, the load into 989 needs 298 off.
    BossStart { name: "Black Knight", pack: 989, start: 2, flags: &[249, 250, 68, 251, 296, 297], items: &[], roberta: false, place: Place::Entry(0), cloud: false },
    // Step 36: the Act 3 Juppie fight (276, 277; 1217's Init sets 81 and 83 with 248 on).
    BossStart { name: "Juppie (Act 3)", pack: 1217, start: 2, flags: &[249, 250, 276, 277], items: &[], roberta: false, place: Place::Trigger(0), cloud: false },
    // Step 37: the mines (286); trigger 1 starts the intro.
    BossStart { name: "Ixi Chieftain (Act 3)", pack: 864, start: 2, flags: &[249, 250, 286], items: &[], roberta: false, place: Place::Trigger(1), cloud: false },
    // Step 38: the lair's entrance sphere (trigger 1) walks the hero into the arena.
    BossStart { name: "Werelupe King", pack: 1246, start: 2, flags: &[249, 250, 271, 272, 273], items: &[], roberta: false, place: Place::Trigger(1), cloud: false },
    // Step 39: the second load into 985 (66 on, 283 off, 281 set).
    BossStart { name: "Gelert Assassin", pack: 985, start: 2, flags: &[249, 250, 64, 214, 216, 66, 281], items: &[], roberta: false, place: Place::Entry(0), cloud: false },
    // Steps 41 and 42: the dungeons (305, 35); the vault's arrival with 45 clear starts the fight.
    BossStart { name: "Undead Draik", pack: 1096, start: 7, flags: &[305, 35], items: &[], roberta: false, place: Place::Entry(0), cloud: false },
    // Step 43: the food chute (319); trigger 6 with 320 set starts movie 52 and the fight.
    BossStart { name: "Faerie Sisters", pack: 1122, start: 7, flags: &[305, 35, 319, 320], items: &[], roberta: false, place: Place::Trigger(6), cloud: false },
    // Step 46: the final fight in 1432.
    BossStart { name: "Darkest Faerie", pack: 1432, start: 8, flags: &[], items: &[], roberta: false, place: Place::Entry(0), cloud: false },
    // Cut content (SPEC-0010): this build's stand-in, fought on the flat test stage (the Act 3
    // kit of start index 2 is given in the Juppie arena's load, which the stage then replaces).
    BossStart { name: "Cloud Beast (cut, flat test stage)", pack: 1217, start: 2, flags: &[249, 250, 276, 277, 278, 84, 5, 6], items: &[], roberta: false, place: Place::Entry(0), cloud: true },
];


pub fn slot_colours(words: &[u32; 15]) -> [[f32; 3]; 15] {
    words.map(|w| [(w & 0xFF) as f32 / 255.0, ((w >> 8) & 0xFF) as f32 / 255.0, ((w >> 16) & 0xFF) as f32 / 255.0])
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

/// The developer menu's cheats: no damage (and full health each tick), no magic cost (and full
/// magic each tick), no Mote charge use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cheats {
    pub god: bool,
    pub magic: bool,
    pub charge: bool,
    /// The developer menu's no clip: the hero's wall, box, block and sphere contacts are off.
    pub noclip: bool,
}

/// The screen-anchored showcase (modes 1 grow, 2 hold, 3 shrink): the item, its screen
/// position (x, y as fractions of a 640 x 480 screen, z units in front of the camera), scale,
/// hold timer and angle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corner {
    pub id: u16,
    pub pack: u32,
    pub mode: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub s: f32,
    pub timer: f32,
    pub angle: f32,
}

/// The full item showcase in progress (SPEC-0021 Item showcase).
#[derive(Clone, Debug, PartialEq)]
pub struct Showcase {
    pub id: u16,
    pub pack: u32,
    pub pose: u32,
    /// The item model floats above the hero (code 0xA3).
    pub floating: bool,
    pub text: Vec<u8>,
    pub ticks: u32,
    /// The tick the text box opened.
    pub opened: Option<u32>,
    pub angle: f32,
}

/// A disc trigger volume as the trigger step tests it.
fn sim_volume(v: &neopets_data::trigger::Volume) -> neopets_sim::trigger::Volume {
    use neopets_data::trigger::Shape as D;
    use neopets_sim::trigger::Shape as S;
    let shape = match v.shape {
        D::Sphere { centre, radius } => S::Sphere { centre, radius },
        D::Cylinder { base, axis, length, radius } => S::Cylinder { base, axis, length, radius },
        D::Rectangle { origin, axes, extents } => S::Rectangle { origin, axes, extents },
    };
    neopets_sim::trigger::Volume { kind: v.kind, flags: v.flags, mask: v.mask, index: v.index, surface: v.surface, shape }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_colour_vector() {
        // SPEC-0004 Colour test vector: colour word 1 bytes (255, 152, 83), vertex byte 20 (1.25)
        // lit at 128 per channel: (160, 95, 52), an orange part; with no slot colour (160, 160,
        // 160).
        let mut words = [0x80FF_FFFFu32; 15];
        words[1] = 255 | (152 << 8) | (83 << 16);
        let s = slot_colours(&words);
        let out = s[1].map(|c| (128.0 * 1.25 * c) as u32);
        assert_eq!(out, [160, 95, 52]);
        assert_eq!(s[0].map(|c| (128.0 * 1.25 * c) as u32), [160, 160, 160]);
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A segment against an oriented box (origin, three axes, extents along them; the rectangle of
/// SPEC-0018): the slab test in the box's frame.
/// The breakable contact cue by material (SPEC-0036 Hit detection step 6).
fn break_cue(material: i16) -> i32 {
    match material {
        2 => 143,
        3 => 145,
        4 => 146,
        _ => 144,
    }
}

fn segment_box(a: [f32; 3], b: [f32; 3], origin: [f32; 3], axes: [[f32; 3]; 3], ext: [f32; 3]) -> bool {
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for k in 0..3 {
        let ax = unit(axes[k]);
        let (pa, pb) = (dot(sub_v(a, origin), ax), dot(sub_v(b, origin), ax));
        let (lo, hi) = (ext[k].min(0.0), ext[k].max(0.0));
        let d = pb - pa;
        if d.abs() < 1e-9 {
            if pa < lo || pa > hi {
                return false;
            }
        } else {
            let (u, w) = ((lo - pa) / d, (hi - pa) / d);
            t0 = t0.max(u.min(w));
            t1 = t1.min(u.max(w));
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// An effect frame from a direction and world up: its flat forward as the third axis (+Z when
/// the direction is vertical or zero).
fn flat_frame(d: [f32; 3]) -> neopets_sim::effect::Frame {
    let fw = unit([d[0], 0.0, d[2]]);
    let fw = if fw == [0.0; 3] || fw.iter().any(|c| c.is_nan()) { [0.0, 0.0, 1.0] } else { fw };
    [[fw[2], 0.0, -fw[0]], [0.0, 1.0, 0.0], fw]
}

/// The attack-family tables the driver reads (SPEC-0034 3.17), 14 ids each by Mote index; the
/// burst and hold tables are by hero.
#[derive(Clone, Copy, Debug)]
struct AttackTables {
    line: [i32; 14],
    aura: [i32; 14],
    charge: [i32; 14],
    burst: [[i32; 14]; 2],
    hold: [[i32; 14]; 2],
    light: [i32; 14],
}

impl Default for AttackTables {
    fn default() -> Self {
        Self { line: [-1; 14], aura: [-1; 14], charge: [-1; 14], burst: [[-1; 14]; 2], hold: [[-1; 14]; 2], light: [-1; 14] }
    }
}

/// The driver's live effects: (effect id, handle) per role, the glow of the current sub-state
/// with its timer and started flag, and the burst latch of sub-state 1.
#[derive(Default)]
struct AttackFx {
    tables: AttackTables,
    /// Roberta's shot visuals: the tables, the next shot id, and per live shot its ids (A, B,
    /// C), its flight handles and its last position.
    shot_tables: neopets_sim::roberta::VisualTables,
    next_shot: u32,
    shots: std::collections::HashMap<u32, ([i32; 3], [Option<usize>; 2], [f32; 3])>,
    line: Option<(i32, usize)>,
    light: Option<(i32, usize)>,
    aura: Option<(i32, usize)>,
    charge: Option<(i32, usize)>,
    hold: Option<(i32, usize)>,
    glow: Option<usize>,
    glow_sub: u8,
    glow_timer: f32,
    glow_started: bool,
    burst_done: bool,
    burst: Option<i32>,
    trace: String,
}

/// One driver role: start (or replace) its effect when the wanted id changes, stop and release it
/// when none is wanted, and move it (and its spawners' target) every pass.
#[allow(clippy::too_many_arguments)]
fn keep_fx(effects: &mut neopets_sim::effect::Effects, table: &neopets_sim::effect::EffectTable, rng: &mut neopets_sim::rng::Rng, frame: neopets_sim::effect::Frame, slot: &mut Option<(i32, usize)>, want: Option<i32>, at: [f32; 3], target: Option<[f32; 3]>) {
    let want = want.filter(|&id| id >= 0);
    if slot.map(|(id, _)| id) != want {
        if let Some((_, hd)) = slot.take() {
            effects.stop(table, hd);
            effects.release(hd);
        }
        if let Some(id) = want {
            let hd = effects.add_object(neopets_sim::effect::EffectObject::new(table, id, at, frame));
            if let Some(tg) = target {
                effects.set_target(hd, tg);
            }
            effects.update(table, hd, None, rng);
            *slot = Some((id, hd));
        }
    }
    if let Some((_, hd)) = *slot {
        effects.set_position(hd, at);
        if let Some(tg) = target {
            effects.set_target(hd, tg);
        }
    }
}

