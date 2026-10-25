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
