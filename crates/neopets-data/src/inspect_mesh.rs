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
    println!("x-major breakdown: no cell {}, miss {}, <0.5 {}, <2 {}, <10 {}, farther {}", kinds[0], kinds[1], kinds[2], kinds[3], kinds[4], kinds[5]);
    for (k, label) in ["x-major", "z-major"].iter().enumerate() {
        let (hit, all) = totals[k];
        println!("Probe on level-mesh vertices, {label} cells: {hit} of {all} within 0.5 units");
    }
    Ok(())
}

/// Check the area table against SPEC-0006's sample rows (slot, pack, world id).
pub fn areas(disc: &mut Disc) -> Result<(), String> {
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
    Ok(())
}
