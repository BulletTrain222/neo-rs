//! Inspector check for SPEC-0004: decode every node tree on the disc and compare each leaf with
//! its own header (vertex and batch counts) and its exact bounding box.
use neopets_data::archive::ArchiveIndex;
use neopets_data::mesh::{self, Batch, Layout, Leaf};
use neopets_data::resource::{self, Family};
use neopets_data::{Disc, model};
use std::collections::BTreeMap;

#[derive(Default)]
struct Census {
    trees: BTreeMap<u16, u32>,
    tree_errors: Vec<String>,
    leaves: u32,
    other_packet_kinds: u32,
    batches: BTreeMap<&'static str, u32>,
    two_pass_batches: u32,
    triangles: u64,
    vertices: u64,
    count_mismatches: Vec<String>,
    bounds_checked: u32,
    bounds_mismatches: Vec<String>,
    decode_errors: Vec<String>,
}

fn layout_name(l: Layout) -> &'static str {
    match l {
        Layout::Skinned => "A skinned",
        Layout::Plain => "B plain",
        Layout::Lit => "C lit",
    }
}

/// Exact box check: the leaf's minimum corner and size equal its vertices' extent. Allows only
/// float rounding of `max - min`.
fn bounds_match(leaf: &Leaf, batches: &[Batch]) -> bool {
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for p in batches.iter().flat_map(|b| &b.positions) {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    (0..3).all(|k| {
        let tol = 1e-4 * (1.0 + hi[k].abs().max(lo[k].abs()));
        (lo[k] - leaf.bounds.min[k]).abs() <= tol
            && (hi[k] - lo[k] - leaf.bounds.size[k]).abs() <= tol
    })
}

pub fn run(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    let mut c = Census::default();
    for n in 0..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        let family = resource::classify(&bytes);
        if !matches!(family, Family::ChunkFile(_)) {
            continue;
        }
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        let (trees, _) = model::find_trees(&bytes, &chain.chunks);
        for t in trees {
            *c.trees.entry(t.kind).or_default() += 1;
            let payload = &bytes[t.payload.clone()];
            let tree = match mesh::walk_tree(payload) {
                Ok(tree) => tree,
                Err(e) => {
                    c.tree_errors.push(format!("resource {n} tree at {:#x}: {e}", t.payload.start));
                    continue;
                }
            };
            for leaf in &tree.leaves {
                c.leaves += 1;
                if leaf.packet_kind != 0 {
                    c.other_packet_kinds += 1;
                    continue;
                }
                let batches = match mesh::decode_packet(payload, leaf) {
                    Ok(b) => b,
                    Err(e) => {
                        c.decode_errors.push(format!("resource {n} leaf {:#x}: {e}", leaf.offset));
                        continue;
                    }
                };
                let verts: usize = batches.iter().map(|b| b.positions.len()).sum();
                if verts as u32 != leaf.vertex_count || batches.len() as u32 != leaf.batch_count {
                    c.count_mismatches.push(format!(
                        "resource {n} leaf {:#x}: {verts} vertices in {} batches, header says {} in {}",
                        leaf.offset,
                        batches.len(),
                        leaf.vertex_count,
                        leaf.batch_count
                    ));
                }
                c.vertices += verts as u64;
                for b in &batches {
                    *c.batches.entry(layout_name(b.layout)).or_default() += 1;
                    c.two_pass_batches += !b.uvs_pass1.is_empty() as u32;
                    c.triangles += b.triangles().len() as u64;
                }
                if !batches.is_empty() {
                    c.bounds_checked += 1;
                    if !bounds_match(leaf, &batches) {
                        c.bounds_mismatches.push(format!("resource {n} leaf {:#x}", leaf.offset));
                    }
                }
            }
        }
    }
    println!("Node trees by holding chunk type: {:?}", c.trees);
    println!(
        "Mesh leaves: {} ({} with a packet kind other than 0, not decoded)",
        c.leaves, c.other_packet_kinds
    );
    println!("Batches by layout: {:?}; two-pass: {}", c.batches, c.two_pass_batches);
    println!("Vertices: {}; strip triangles: {}", c.vertices, c.triangles);
    println!(
        "Leaf boxes equal to their vertex extent: {} of {}",
        c.bounds_checked as usize - c.bounds_mismatches.len(),
        c.bounds_checked
    );
    for (label, list) in [
        ("Tree walk errors", &c.tree_errors),
        ("Packet decode errors", &c.decode_errors),
        ("Vertex/batch count mismatches", &c.count_mismatches),
        ("Bounding box mismatches", &c.bounds_mismatches),
    ] {
        println!("{label}: {}", list.len());
        for line in list.iter().take(10) {
            println!("  {line}");
        }
    }
    Ok(())
}

/// List a world pack's placed trees with their source chunk type and transformed bounds.
pub fn scene(disc: &mut Disc, index: &ArchiveIndex, n: u32) -> Result<(), String> {
    use neopets_data::world;
    let bytes = index.read(disc, n)?;
    let family = resource::classify(&bytes);
    let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
    let s = world::scene(&bytes, &chain.chunks);
    println!("Resource {n}: {} placed trees, {} unresolved references", s.trees.len(), s.unresolved);
    for (i, t) in s.trees.iter().enumerate() {
        let payload = &bytes[t.payload.clone()];
        let Ok(tree) = mesh::walk_tree(payload) else {
            println!("  {i:>4} type {:>2}: does not walk", t.source);
            continue;
        };
        let b = tree.root_bounds;
        let (lo, hi) = (b.min, [b.min[0] + b.size[0], b.min[1] + b.size[1], b.min[2] + b.size[2]]);
        let (lo, hi) = match &t.matrix {
            Some(m) => (world::transform_point(m, lo), world::transform_point(m, hi)),
            None => (lo, hi),
        };
        println!(
            "  {i:>4} type {:>2}: {:>4} leaves, box ({:.0}, {:.0}, {:.0}) .. ({:.0}, {:.0}, {:.0})",
            t.source,
            tree.leaves.len(),
            lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]
        );
    }
    Ok(())
}

/// Check the ground probe against each world pack's level-mesh vertices for both cell orders: a
/// vertex counts as a hit when the probe from just above it lands within 0.5 units. Walls and
/// props also have vertices, so a correct order still misses many; the comparison is relative.
pub fn ground(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    use neopets_data::ground::{CellOrder, Ground, MISS, navmesh_vertices, spawn, spawn_height};
    let mut totals = [(0u64, 0u64); 2];
    // x-major breakdown: no cell (0.0), miss, |dh| < 0.5, < 2, < 10, larger.
    let mut kinds = [0u64; 6];
    for n in 533..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        if !matches!(resource::classify(&bytes), Family::ChunkFile(_)) {
            continue;
        }
        let chain = resource::walk_chunks(&bytes, None)?;
        if chain.chunks.first().map(|c| c.kind) != Some(31) {
            continue;
        }
        let _ = navmesh_vertices;
        // Level display tree vertices (world space), sampled: terrain lies on the patches.
        let mut verts = Vec::new();
        for c in chain.chunks.iter().filter(|c| c.kind == 3) {
            let payload = &bytes[c.payload.clone()];
            let Ok(tree) = mesh::walk_tree(payload) else { continue };
            for leaf in tree.leaves.iter().step_by(3) {
                if let Ok(batches) = mesh::decode_packet(payload, leaf) {
                    verts.extend(batches.iter().flat_map(|b| b.positions.iter().copied()).step_by(5));
                }
            }
        }
        for (k, order) in [CellOrder::XMajor, CellOrder::ZMajor].into_iter().enumerate() {
            let Some(g) = Ground::of(&bytes, &chain.chunks, order) else { continue };
            for v in verts.iter() {
                let h = g.probe(v[0], v[2], v[1] + 0.5);
                totals[k].1 += 1;
                totals[k].0 += (h != MISS && (h - v[1]).abs() < 0.5) as u64;
                if k == 0 {
                    let d = (h - v[1]).abs();
                    kinds[match () {
                        _ if h == 0.0 => 0,
                        _ if h == MISS => 1,
                        _ if d < 0.5 => 2,
                        _ if d < 2.0 => 3,
                        _ if d < 10.0 => 4,
                        _ => 5,
                    }] += 1;
                }
            }
            if n == 909 {
                if let Some((s, f)) = spawn(&bytes, &chain.chunks) {
                    println!(
                        "Pack 909 spawn ({order:?}): record ({:.2}, {:.2}, {:.2}) facing ({:.2}, {:.2}, {:.2}); ground height {:.3}",
                        s[0], s[1], s[2], f[0], f[1], f[2], spawn_height(&g, s)
                    );
                }
            }
        }
    }
    // Rectangle census (SPEC-0028 6.1): records, pushing boxes, and the axes' unit length.
    let (mut rects, mut pushing, mut worst) = (0usize, 0usize, 0f32);
    for n in 533..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        let Ok(chain) = resource::walk_chunks(&bytes, None) else { continue };
        if chain.chunks.first().map(|c| c.kind) != Some(31) {
            continue;
        }
        for r in neopets_data::ground::rectangles(&bytes, &chain.chunks) {
            rects += 1;
            pushing += r.pushes(0) as usize;
            for a in r.axes {
                worst = worst.max(((a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() - 1.0).abs());
            }
        }
    }
    println!("Rectangles: {rects} (SPEC-0028: 6,524), {pushing} pass the box-push gate at mask 0; largest axis length error {worst:.2e}");
    // Frame records referenced around pack 909's spawn: check the assumed axis layout.
    {
        let bytes = index.read(disc, 909)?;
        let chain = resource::walk_chunks(&bytes, None)?;
        if let Some(g) = Ground::of(&bytes, &chain.chunks, CellOrder::XMajor) {
            let (mut n, mut unit, mut vertical, mut kinds) = (0, 0, 0, std::collections::BTreeMap::new());
            let (mut on_planes, mut top_above) = (0, 0);
            let len = |a: [f32; 4]| (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            for dx in -10..10 {
                for dz in -10..10 {
                    for f in g.frames_at(-1852.8 + dx as f32 * 8.0, 1137.5 + dz as f32 * 8.0) {
                        n += 1;
                        unit += ((len(f.face) - 1.0).abs() < 1e-3 && (len(f.run) - 1.0).abs() < 1e-3) as usize;
                        vertical += (f.face[1].abs() <= 0.2) as usize;
                        *kinds.entry(f.kind).or_insert(0usize) += 1;
                        let pv = |q: [f32; 4]| q[0] * f.start[0] + q[1] * f.start[1] + q[2] * f.start[2] + q[3];
                        on_planes += (pv(f.face).abs() < 1e-2 && pv(f.run).abs() < 1e-2) as usize;
                        top_above += ((f.depth_at(f.start) - f.height).abs() < 1e-2 && f.height > 0.0) as usize;
                    }
                }
            }
            println!("Frame records near 909's spawn (with repeats): {n}; unit face and run {unit}; near-vertical face {vertical}; start on the face and run planes {on_planes}; top-plane value H at the start (H > 0) {top_above}; types {kinds:?}");
        }
    }
    println!("x-major breakdown: no cell {}, miss {}, <0.5 {}, <2 {}, <10 {}, farther {}", kinds[0], kinds[1], kinds[2], kinds[3], kinds[4], kinds[5]);
    for (k, label) in ["x-major", "z-major"].iter().enumerate() {
        let (hit, all) = totals[k];
        println!("Probe on level-mesh vertices, {label} cells: {hit} of {all} within 0.5 units");
    }
    Ok(())
}

/// Check the area table against SPEC-0006's sample rows (slot, pack, world id).
pub fn areas(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    let t = neopets_data::area::AreaTable::from_disc(disc)?;
    let rows = [(0usize, 678u32, 0u32), (5, 691, 11), (68, 909, 602), (107, 1121, 1362), (192, 1433, 11070)];
    let ok = rows.iter().all(|&(s, p, w)| t.packs[s] == p && t.world_ids[s] == w);
    let distinct: std::collections::BTreeSet<u32> = t.packs.iter().copied().collect();
    println!(
        "Area table: {} slots, {} distinct packs, world ids {}..{}; SPEC-0006 sample rows {}",
        t.packs.len(),
        distinct.len(),
        t.world_ids[0],
        t.world_ids[t.world_ids.len() - 1],
        if ok { "match" } else { "DO NOT MATCH" }
    );
    for dome in 1..=3 {
        let packs: Vec<u32> = (0..t.packs.len())
            .filter(|&s| neopets_data::area::dome_for_slot(Some(s)) == dome)
            .map(|s| t.packs[s])
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        if dome > 1 {
            println!("Dome {dome}: packs {packs:?}");
        }
    }
    travel_census(disc, index)
}

/// SPEC-0006 travel data over every game pack: module families by host-record group, the
/// neighbour lists' shape, and the boot tables.
fn travel_census(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    use neopets_data::travel::{BootTravel, Neighbours};
    let boot = BootTravel::from_disc(disc)?;
    let names: Vec<usize> = boot.family_names.iter().map(Vec::len).collect();
    let distinct = boot.family_names.iter().collect::<std::collections::BTreeSet<_>>().len();
    println!(
        "Travel tables: {} area pairs ({} non-zero), {} no-swap packs, {} block pairs; family names: {} distinct, lengths {names:?}",
        boot.area_pairs.len(),
        boot.area_pairs.iter().filter(|p| p.0 != 0).count(),
        boot.no_swap.len(),
        boot.block_pairs.len(),
        distinct
    );
    let mut kinds = std::collections::BTreeMap::new();
    let mut of_pack = std::collections::BTreeMap::new();
    let (mut tables, mut zero_unused, mut contiguous, mut ids) = (0, 0, 0, 0);
    let (mut line_tables, mut line_records, mut resolved) = (0, 0, 0);
    let (mut planes, mut plane_packs, mut default_doors, mut scripted_packs, mut arrivals_ok) = (0, 0, 0, 0, 0);
    for pack in 533..=1433u32 {
        let Ok(bytes) = index.read(disc, pack) else { continue };
        let Ok(chain) = resource::walk_chunks(&bytes, None) else { continue };
        if chain.chunks.first().map(|c| c.kind) != Some(31) {
            continue;
        }
        let kind = boot.kind(pack, &bytes, &chain.chunks);
        let module = !boot.module_entries(&bytes, &chain.chunks).is_empty();
        *kinds.entry((format!("{kind:?}"), module)).or_insert(0usize) += 1;
        of_pack.insert(pack, kind);
        let lt = neopets_data::dialogue::LineTable::of(&bytes, &chain.chunks);
        if !lt.is_empty() {
            line_tables += 1;
            line_records += lt.len();
            resolved += (0..lt.len()).filter(|&i| lt.line(i).is_some()).count();
        }
        let tp = neopets_data::ground::teleport_planes(&bytes, &chain.chunks);
        if !tp.is_empty() {
            planes += tp.len();
            plane_packs += 1;
            if boot.imports(&bytes, &chain.chunks, "RegisterTeleportCallback") {
                scripted_packs += 1;
            } else {
                default_doors += tp.len();
                println!("  default doors: pack {pack} ({kind:?}) {} door(s), arrivals {:?}", tp.len(), (0..tp.len() as i32).map(|k| Neighbours::arrivals(&bytes, &chain.chunks).get(k)).collect::<Vec<_>>());
            }
            let arr = Neighbours::arrivals(&bytes, &chain.chunks);
            arrivals_ok += (0..tp.len() as i32).all(|n| arr.get(n) >= 0) as usize;
        }
        if chain.chunks.iter().any(|c| c.kind == 15) {
            let n = Neighbours::of(&bytes, &chain.chunks);
            let used: Vec<i32> = (0..200).filter(|&k| n.get(k) != -1).collect();
            if !used.is_empty() || n != Neighbours::default() {
                tables += 1;
                zero_unused += (n.get(0) == -1) as usize;
                contiguous += used.iter().enumerate().all(|(i, &k)| k == i as i32 + 1) as usize;
                ids += used.len();
            }
        }
    }
    // SPEC-0029 section 4 / SPEC-0056: monster rows and the class of every placement.
    let table = neopets_data::monster::MonsterTable::parse(&index.read(disc, 530)?, &index.read(disc, 531)?)?;
    let (mut placements, mut townsfolk, mut minus7_not6, mut classes) = (0, 0, 0, std::collections::BTreeMap::new());
    for res in 533..=1433u32 {
        let Ok(b) = index.read(disc, res) else { continue };
        let Ok(ch) = resource::walk_chunks(&b, None) else { continue };
        let regions: Vec<neopets_data::region::Region> = if ch.chunks.first().map(|c| c.kind) == Some(31) {
            ch.chunks
                .iter()
                .filter(|c| c.kind == 32)
                .filter_map(|c| {
                    let p = &b[c.payload.clone()];
                    let n = resource::walk_chunks(p, None).ok()?;
                    Some(neopets_data::region::Region::of(p, &n.chunks))
                })
                .collect()
        } else if ch.chunks.iter().any(|c| c.kind == 36) {
            vec![neopets_data::region::Region::of(&b, &ch.chunks)]
        } else {
            continue;
        };
        for r in regions {
            for pl in &r.records.placements {
                placements += 1;
                let class = table.row(pl.monster_type, pl.variant).and_then(|row| row.class);
                *classes.entry(class).or_insert(0usize) += 1;
                townsfolk += (class == Some(6)) as usize;
                minus7_not6 += (pl.skeletons[0] == -7 && class != Some(6)) as usize;
            }
        }
    }
    println!(
        "Monster table: {} rows; {placements} placements, {townsfolk} of class 6, {minus7_not6} main -7 placements not class 6; by class {classes:?}",
        table.row_count()
    );
    // SPEC-0018 navigation meshes and SPEC-0056 route tables over every game pack.
    {
        let (mut meshes, mut kinds_nav, mut verts, mut short, mut one_way) = (0, std::collections::BTreeMap::new(), 0, 0, 0);
        let mut pack722 = None;
        for pack in 533..=1433u32 {
            let Ok(b) = index.read(disc, pack) else { continue };
            let Ok(ch) = resource::walk_chunks(&b, None) else { continue };
            if ch.chunks.first().map(|c| c.kind) != Some(31) {
                continue;
            }
            let ms = neopets_data::navmesh::navmeshes(&b, &ch.chunks);
            for m in &ms {
                meshes += 1;
                *kinds_nav.entry(m.kind).or_insert(0usize) += 1;
                let t = m.triangles.len();
                verts += 3 * t;
                short += (m.table.len() < (2 * t * t).div_ceil(8)) as usize;
                for (a, nb) in m.neighbours.iter().enumerate() {
                    for &n in nb {
                        if n >= 0 && !m.neighbours.get(n as usize).is_some_and(|x| x.contains(&(a as i32))) {
                            one_way += 1;
                        }
                    }
                }
            }
            if pack == 722 {
                pack722 = ms.get(2).map(|m| (m.kind, m.neighbours.clone(), (0..5).map(|a| (0..5).map(|b| m.route(a, b).unwrap_or(9)).collect::<Vec<_>>()).collect::<Vec<_>>()));
            }
        }
        {
            let b = index.read(disc, 909)?;
            let ch = resource::walk_chunks(&b, None)?;
            let e8 = neopets_data::ground::entry_record(&b, &ch.chunks, 8);
            println!("  909 locator 8 (pen point): {e8:?}");
            for (i, m) in neopets_data::navmesh::navmeshes(&b, &ch.chunks).iter().enumerate() {
                let xs = m.triangles.iter().flatten().map(|v| v[0]);
                let zs = m.triangles.iter().flatten().map(|v| v[2]);
                let (x0, x1) = xs.fold((f32::MAX, f32::MIN), |a, x| (a.0.min(x), a.1.max(x)));
                let (z0, z1) = zs.fold((f32::MAX, f32::MIN), |a, z| (a.0.min(z), a.1.max(z)));
                println!("  909 mesh {i}: kind {}, {} triangles, x {x0:.1}..{x1:.1}, z {z0:.1}..{z1:.1}", m.kind, m.triangles.len());
            }
        }
        println!("Navigation meshes: {meshes} (by kind {kinds_nav:?}), {verts} vertices, {short} route tables short of 2T^2/8 bytes, {one_way} one-way links");
        println!("Pack 722 mesh 2 (kind, neighbours, route rows): {pack722:?}");
    }
    // SPEC-0021 item table and SPEC-0005 hurt capsules (Tor's outfit 504 for attach points).
    {
        let items = neopets_data::item::ItemTable::parse(&index.read(disc, 529)?)?;
        let mut by_type = std::collections::BTreeMap::new();
        for it in &items.items {
            *by_type.entry(it.kind).or_insert(0usize) += 1;
        }
        let weapons: Vec<(u16, i16)> = items.items.iter().filter(|i| i.is_weapon()).map(|i| (i.id, i.stats[0])).collect();
        println!("Item table: {} records, by type {by_type:?}; weapons (id, stat A) {weapons:?}", items.items.len());
        println!(
            "Drop tables: {} sub-tables {:?}; {} groups {:?}",
            items.sub_tables.len(),
            items.sub_tables.iter().map(Vec::len).collect::<Vec<_>>(),
            items.groups.len(),
            items.groups.iter().map(|g| (g.chance, g.min, g.max, g.entries.clone())).collect::<Vec<_>>()
        );
        let b = index.read(disc, 504)?;
        let ch = resource::walk_chunks(&b, None)?;
        if let Some(c) = ch.chunks.iter().find(|c| c.kind == 18 && !b[c.payload.clone()].starts_with(b"IDM")) {
            let t = neopets_data::anim::BoneTable::parse(&b[c.payload.clone()])?;
            let caps = t.hurt.iter().filter(|h| h.1 > 0.0).count();
            println!("Outfit 504 bones: {}, hurt capsules {caps}, attach points {:?}", t.len(), t.attach.iter().take(2).collect::<Vec<_>>());
        }
    }
    // SPEC-0036 start values and SPEC-0038 band factors; class-17 tuning census.
    {
        let boot_bytes = neopets_data::archive::verified_boot_program(disc)?;
        let hp = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::item::HEALTH_TOR, 4)?;
        let f1: Vec<f32> = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::monster::BAND_F1, 7)?.into_iter().map(f32::from_bits).collect();
        let f2: Vec<f32> = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::monster::BAND_F2, 7)?.into_iter().map(f32::from_bits).collect();
        println!("Hero start words (health Tor, Roberta, magic Tor, Roberta) {hp:08x?}; band F1 {f1:?}, F2 {f2:?}");
        let t = neopets_data::monster::MonsterTable::parse(&index.read(disc, 530)?, &index.read(disc, 531)?)?;
        let (mut c17, mut by_approach, mut slot_kinds) = (0, std::collections::BTreeMap::new(), std::collections::BTreeMap::new());
        for i in 0..t.row_count() {
            let (Some(r), Some(u)) = (t.row_at(i), t.tuning_at(i)) else { continue };
            if r.class != Some(17) {
                continue;
            }
            c17 += 1;
            *by_approach.entry(u.approach).or_insert(0usize) += 1;
            for s in u.slots.iter().filter(|s| s.kind != 0xFFFF) {
                *slot_kinds.entry(s.kind).or_insert(0usize) += 1;
            }
        }
        let flower = t.tuning_at(149).map(|u| (u.approach, u.slots.map(|s| (s.kind, s.trigger_min, s.trigger_max, s.range, s.damage, s.knockback, s.wind_speed, s.strike_speed)), u.weapons));
        println!("Class-17 rows: {c17}, by approach {by_approach:?}, used slots by kind {slot_kinds:?}; row 149 {flower:?}");
        // Packs whose embedded region (type 32) places ground class-17 enemies with a melee slot.
        let mut melee_packs = Vec::new();
        let (mut item_points, mut drop_points) = (0usize, 0usize);
        for pack in 533..=1433u32 {
            let Ok(bytes) = index.read(disc, pack) else { continue };
            let Ok(chain) = resource::walk_chunks(&bytes, None) else { continue };
            if chain.chunks.first().map(|c| c.kind) != Some(31) {
                continue;
            }
            let Some(c) = chain.chunks.iter().find(|c| c.kind == 32) else { continue };
            let Ok(nested) = resource::walk_chunks(&bytes[c.payload.clone()], None) else { continue };
            let r = neopets_data::region::Region::of(&bytes[c.payload.clone()], &nested.chunks);
            item_points += r.records.items.len();
            drop_points += r.records.items.iter().filter(|i| i.key == -1).count();
            let names: Vec<String> = r
                .records
                .placements
                .iter()
                .filter(|pl| {
                    t.row(pl.monster_type, pl.variant).is_some_and(|row| row.class == Some(17) && row.can_fly == 0)
                        && t.tuning(pl.monster_type, pl.variant).is_some_and(|u| u.slots.iter().any(|s| s.kind == 0))
                })
                .map(|pl| format!("{} (state {})", pl.name, pl.ai_code))
                .collect();
            if !names.is_empty() {
                melee_packs.push((pack, names.len(), names.first().cloned().unwrap_or_default()));
            }
        }
        println!("Packs with embedded ground melee class-17 enemies: {} {:?}", melee_packs.len(), melee_packs.iter().take(12).collect::<Vec<_>>());
        println!("Embedded kind-8 item points: {item_points}, of them {drop_points} with key -1");
        // SPEC-0055 Layout: deadly ground entries (kind 2), with and without a respawn index.
        let (mut deadly, mut indexed, mut deadly_packs, mut surfaces) = (0, 0, 0, std::collections::BTreeMap::new());
        let mut first = None;
        // SPEC-0025 Save points: kind-4 records of each world pack's own region.
        let (mut save_points, mut save_packs, mut save_flags, mut save_dropped, mut save_first) = (0usize, 0usize, 0usize, 0usize, None);
        for pack in 533..=1433u32 {
            let Ok(bytes) = index.read(disc, pack) else { continue };
            let Ok(chain) = resource::walk_chunks(&bytes, None) else { continue };
            if chain.chunks.first().map(|c| c.kind) != Some(31) {
                continue;
            }
            if let Some(r) = neopets_data::region::embedded_records(&bytes, &chain.chunks) {
                if !r.save_points.is_empty() {
                    save_packs += 1;
                    save_points += r.save_points.len();
                    save_flags += r.save_points.iter().filter(|s| s.flag != 0).count();
                    save_first.get_or_insert((pack, r.save_points[0].position));
                }
                save_dropped += r.save_points_dropped;
            }
            let Some(g) = neopets_data::ground::Ground::of(&bytes, &chain.chunks, neopets_data::ground::CellOrder::XMajor) else { continue };
            let d = g.deadly_entries();
            if d.is_empty() {
                continue;
            }
            deadly_packs += 1;
            deadly += d.len();
            indexed += d.iter().filter(|e| e.0 != 0xFF).count();
            for e in &d {
                *surfaces.entry(e.1).or_insert(0usize) += 1;
            }
            first.get_or_insert((pack, d[0]));
        }
        println!("Deadly ground entries: {deadly} in {deadly_packs} packs, {indexed} with a respawn index; surfaces {surfaces:?}; first {first:?}");
        println!("Save points: {save_points} in {save_packs} world packs, {save_flags} with a nonzero flag, {save_dropped} dropped past eight; first {save_first:?}");
        // SPEC-0051 Map area names and SPEC-0021 categories, as the save writer reads them.
        if let Ok(d) = index.read(disc, neopets_data::design::RESOURCE).and_then(|b| neopets_data::design::DesignTables::parse(&b)) {
            let name = |m: usize| d.location_name(m).map(|t| String::from_utf8_lossy(&t).into_owned());
            let mut area = |pack: u32| index.read(disc, pack).ok().and_then(|b| {
                let c = resource::walk_chunks(&b, None).ok()?;
                neopets_data::travel::map_area(&b, &c.chunks)
            });
            let (a909, a691, a678) = (area(909), area(691), area(678));
            println!(
                "Map areas: 909 {a909:?} {:?}, 691 {a691:?}, 678 {a678:?}; names 5 {:?}, 7 {:?}, 43 {:?}; categories 98 {:?}, 126 {:?}, 164 {:?}, 146 {:?}; limits 146 {:?}",
                a909.and_then(|m| name(m as usize)), name(5), name(7), name(43),
                d.item_category(98), d.item_category(126), d.item_category(164), d.item_category(146), d.carry_limit(146)
            );
        }
        // SPEC-0035 section 14.3: the row drawer's words, raw.
        if let Ok(boot) = neopets_data::archive::verified_boot_program(disc) {
            if let Ok(w) = neopets_data::archive::boot_u32_table(&boot, 0x002D_CDA8, 12) {
                let shown: Vec<String> = w.iter().map(|v| format!("{v:#x}/{}", f32::from_bits(*v))).collect();
                println!("Row drawer words 0x2DCDA8..: {shown:?}");
            }
        }
        // SPEC-0025 Lookups: the world key table (boot 0x31EFA8 keys, 0x31F2B0 saved ids).
        if let Ok(boot) = neopets_data::archive::verified_boot_program(disc) {
            if let (Ok(keys), Ok(ids)) = (neopets_data::archive::boot_u32_table(&boot, 0x0031_EFA8, 193), neopets_data::archive::boot_u32_table(&boot, 0x0031_F2B0, 193)) {
                let f = |k: i32| keys.iter().position(|&v| v as i16 == k as i16).map(|i| (i, ids[i]));
                let distinct = ids.iter().collect::<std::collections::BTreeSet<_>>().len();
                println!(
                    "World keys: 678 {:?}, 691 {:?}, 909 {:?}, 1391 {:?}, 1414 {:?}, 1433 {:?}; {distinct} distinct ids, max {:?}",
                    f(678), f(691), f(909), f(1391), f(1414), f(1433), ids.iter().max()
                );
            }
        }
    }
    // SPEC-0035 / SPEC-0023: the front-end UI database and the HUD's art.
    {
        let f = neopets_data::frontend::Frontend::parse(&index.read(disc, neopets_data::frontend::RESOURCE)?)?;
        let mut by_kind = std::collections::BTreeMap::new();
        for n in &f.nodes {
            *by_kind.entry(n.kind).or_insert(0usize) += 1;
        }
        let sheets = f.assets.iter().filter(|a| matches!(a, neopets_data::frontend::Asset::Sheet(_))).count();
        let resolved = (0..f.assets.len() as u32).filter(|&i| f.sheet_entries(i).is_some()).count();
        let pictures = f.assets.iter().filter(|a| matches!(a, neopets_data::frontend::Asset::Picture(_))).count();
        let hud = f.screens.get(9).map(|s| (s.node, s.state));
        let keys: Vec<(u32, Option<usize>)> = [0x01, 0x02, 0x06, 0x07, 0x08, 0x09, 0x31, 0x3B, 0x12, 0xD1].iter().map(|&k| (k, f.key(k))).collect();
        println!(
            "Front end: {} nodes by kind {by_kind:?}, {} screens, {} sprites, {} value texts, {} keys; assets: {sheets} sheets ({resolved} resolve to the texture chunk), {pictures} pictures; HUD record 9 (node, state) {hud:?}; keys {keys:?}",
            f.nodes.len(),
            f.screens.len(),
            f.sprites.len(),
            f.values.len(),
            f.keys.len()
        );
        // The HUD's bars group 0x31 subtree: depth, node, kind, mode, handler, arg, key, x, y, w, h.
        fn walk(f: &neopets_data::frontend::Frontend, n: usize, depth: usize, out: &mut Vec<String>) {
            let d = f.nodes[n];
            out.push(format!(
                "{}{n} k{} m{} h{} a{} key{:x} ({},{}) {}x{} s{} al{}{}",
                "  ".repeat(depth),
                d.kind,
                d.mode,
                d.handler,
                d.arg,
                d.key,
                d.x,
                d.y,
                d.width,
                d.height,
                d.scale_x,
                d.align,
                if d.absolute { " abs" } else { "" }
            ));
            for c in f.children(n) {
                walk(f, c, depth + 1, out);
            }
        }
        if std::env::var_os("NEOPETS_PAGE_TREES").is_some() {
            // The load (17), save (18) and journal (22) records' trees.
            for r in [17usize, 18, 22] {
                let mut out = Vec::new();
                if let Some(n) = f.screens.get(r).and_then(|s| s.node) {
                    walk(&f, n, 0, &mut out);
                }
                let _ = std::fs::write(format!("private/tree-{r}.txt"), out.join("\n"));
            }
        }
        let row_keys: Vec<String> = [0xD3u32, 0x37, 0x36, 0xDD, 0x34, 0x33]
            .iter()
            .map(|&k| match f.key(k) {
                Some(n) => {
                    let d = f.nodes[n];
                    format!("{k:#04x} -> node {n} kind {} ({}, {}) {}x{}{} children {}", d.kind, d.x, d.y, d.width, d.height, if d.absolute { " abs" } else { "" }, f.children(n).len())
                }
                None => format!("{k:#04x} -> none"),
            })
            .collect();
        println!("Journal row parts as section C keys: {row_keys:?}");
        for k in [0x34u32, 0x33] {
            if let Some(n) = f.key(k) {
                println!("  row text {k:#04x}: {:?}", f.label(n).map(|l| (l.font, l.spacing, l.colour, String::from_utf8_lossy(&l.text).into_owned())));
            }
        }
        if std::env::var_os("NEOPETS_HUD_TREE").is_some() {
            let mut out = Vec::new();
            walk(&f, 322, 0, &mut out);
            let _ = std::fs::write("private/hud-tree.txt", out.join("
"));
        }
        let title: Vec<String> = f
            .children(759)
            .iter()
            .filter(|&&n| f.nodes[n].kind == 4)
            .map(|&n| {
                let label = f.children(n).iter().find_map(|&c| f.label(c)).map(|l| String::from_utf8_lossy(&l.text).into_owned()).unwrap_or_default();
                format!("{n} {label:?} {:?}", f.targets.get(&n))
            })
            .collect();
        println!(
            "Front-end text: {} label bindings ({} with text), {} pages {:?}, {} item targets; title items {title:?}",
            f.labels.len(),
            f.labels.values().filter(|l| !l.text.is_empty()).count(),
            f.pages.len(),
            { let mut v: Vec<_> = f.pages.iter().map(|(n, pg)| (*n, pg.list)).collect(); v.sort(); v },
            f.targets.len()
        );
        let logical: Vec<String> = [0x19u32, 0x1B, 0x78, 0x79]
            .iter()
            .map(|&k| match f.key(k) {
                Some(n) => {
                    let d = f.nodes[n];
                    format!("{k:#04x} -> node {n} kind {} {}x{} sprite {:?}", d.kind, d.width, d.height, f.sprite(n).map(|s| (s.asset, s.columns, s.rows)))
                }
                None => format!("{k:#04x} -> none"),
            })
            .collect();
        println!("Logical sprites as section C keys: {logical:?}");
        let fonts: Vec<(usize, u8, i32, i32)> = f.fonts.iter().map(|ft| (ft.glyphs.len(), ft.mode, ft.line_height(), ft.glyph(32).map_or(-1, |g| g.advance as i32))).collect();
        let sample = "Flower Monster";
        println!("Fonts (glyphs, mode, line height, space advance): {fonts:?}; {sample:?} in font 1 is {} px", f.fonts.get(1).map_or(-1, |ft| ft.measure(sample.as_bytes(), 0)));
        if let Some(mem) = f.memory() {
            if let Some((img, base)) = f.text_image(&mem, 0, "Neopets: The Darkest Faerie".as_bytes(), 0, [128; 4]) {
                std::fs::write("private/textures/font0-sample.png", neopets_data::png::encode_rgba(img.width, img.height, &img.rgba)?).map_err(|e| e.to_string())?;
                println!("  font 0 sample: {}x{}, baseline {base}", img.width, img.height);
            }
        }
        if let Some(mem) = f.memory() {
            for k in [0x01u32, 0x07, 0x06] {
                let Some(n) = f.key(k) else { continue };
                let node = f.nodes[n];
                if let Some(img) = f.sprite_image(&mem, n, node.width.max(1) as u32, node.height.max(1) as u32) {
                    let path = format!("private/textures/hud-key-{k:02x}.png");
                    let _ = std::fs::create_dir_all("private/textures");
                    let _ = std::fs::write(&path, neopets_data::png::encode_rgba(img.width, img.height, &img.rgba)?);
                    println!("  HUD key {k:#04x}: node {n} {}x{} at ({}, {}), colour {:?} -> {path}", node.width, node.height, node.x, node.y, f.sprite(n).map(|s| s.colour));
                }
            }
        }
    }
    println!("Pack kinds (kind, has module): {kinds:?}");
    {
        let bytes = index.read(disc, 909)?;
        let chain = resource::walk_chunks(&bytes, None)?;
        let e: Vec<_> = (0..5).filter_map(|i| neopets_data::ground::entry_record(&bytes, &chain.chunks, i)).collect();
        println!("909 entry records 0..4 (position, facing): {e:?}");
        if let Some(c) = chain.chunks.iter().find(|c| c.kind == 32) {
            let nested = resource::walk_chunks(&bytes[c.payload.clone()], None)?;
            println!("909 type-32 payload: {} bytes, starts {:02x?}; nested kinds {:?}", c.payload.len(), &bytes[c.payload.start..c.payload.start + 32], nested.chunks.iter().map(|k| k.kind).collect::<Vec<_>>());
            let r = neopets_data::region::Region::of(&bytes[c.payload.clone()], &nested.chunks);
            println!("909 embedded region: {} placements, {} tables, {} model sets, skeletons {:?}, clip sets {:?}", r.records.placements.len(), r.records.tables.len(), r.model_sets.len(), r.skeletons.each_ref().map(Vec::len), r.clip_sets.each_ref().map(Vec::len));
            for pl in r.records.placements.iter().take(6) {
                println!("  {:?} at {:?} type {} skel {:?} tables {:?} model {} summon {}", pl.name, pl.position, pl.monster_type, pl.skeletons, pl.tables, pl.model_set, pl.summon);
            }
        }
        for res in [910u32, 914, 920, 926] {
            let b = index.read(disc, res)?;
            let ch = resource::walk_chunks(&b, None)?;
            let r = neopets_data::region::Region::of(&b, &ch.chunks);
            for t in &r.records.tables {
                let n = u32::from_le_bytes(b[t.start..t.start + 4].try_into().unwrap());
                println!("  table at {:#x}..{:#x}: N {n}, slice {} bytes (needs {})", t.start, t.end, t.len(), 4 + 8 * n);
            }
            println!("Region {res}: {} placements, {} tables, {} model sets, skeletons {:?}, clip sets {:?}", r.records.placements.len(), r.records.tables.len(), r.model_sets.len(), r.skeletons.each_ref().map(Vec::len), r.clip_sets.each_ref().map(Vec::len));
            for pl in &r.records.placements {
                println!("  {:?} at {:?} type {} skel {:?} tables {:?} model {} summon {}", pl.name, pl.position, pl.monster_type, pl.skeletons, pl.tables, pl.model_set, pl.summon);
            }
        }
        let ents = neopets_data::travel::entity_table(&bytes, &chain.chunks);
        println!("909 entity table: {} entries; 31..=50: {:?}", ents.len(), ents.get(31..51));
        let lines = neopets_data::dialogue::LineTable::of(&bytes, &chain.chunks);
        println!("909 dialogue table: {} lines; line 0 resolves: {}", lines.len(), lines.line(0).is_some());
    }
    let g = |p: u32| of_pack.get(&p).copied();
    println!("Kinds of 691: {:?}, 909: {:?}, 1429: {:?}, 1008: {:?}, 1121: {:?}", g(691), g(909), g(1429), g(1008), g(1121));
    println!(
        "Teleport planes: {planes} in {plane_packs} packs; {scripted_packs} of those packs import RegisterTeleportCallback; {default_doors} doors in packs that do not; {arrivals_ok} packs give every door an arrival (tag 0x0E)"
    );
    println!("Dialogue line tables: {line_tables} with {line_records} records, {resolved} resolve to speaker and text");
    println!("Neighbour lists (tag 0x0B): {tables} with values; position 0 unused in {zero_unused}; used positions 1..m in {contiguous}; {ids} world ids");
    Ok(())
}
