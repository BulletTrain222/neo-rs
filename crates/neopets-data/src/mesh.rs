//! Node trees and mesh packets (SPEC-0004). A node tree is a hierarchy of group nodes and mesh
//! leaves; each leaf owns a packet, a VIF command stream (public PS2 VIF documentation) holding
//! triangle-strip batches whose attribute arrays follow one of three layouts.
use std::collections::HashSet;

const HEADER: usize = 0x50;
const NODE_GROUP: u32 = 0;
const NODE_LEAF: u32 = 1;
const NODE_VISIBILITY: u32 = 3;
const NODE_LOD: u32 = 4;
/// Node trees on disc hold at most a few thousand nodes; this bounds hostile input.
const MAX_NODES: usize = 1 << 16;

fn u8_at(b: &[u8], at: usize) -> Option<u8> {
    b.get(at).copied()
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

fn vec3_at(b: &[u8], at: usize) -> Option<[f32; 3]> {
    Some([f32_at(b, at)?, f32_at(b, at + 4)?, f32_at(b, at + 8)?])
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: [f32; 3],
    /// Box size, not the maximum corner.
    pub size: [f32; 3],
    pub center: [f32; 3],
    pub radius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    /// Offset of the leaf node in the tree's payload.
    pub offset: usize,
    pub bounds: Bounds,
    pub material_group: u16,
    /// 0 = one packet; 1 = CPU-animated double-buffered leaf; 2 = raw program record.
    pub packet_kind: u8,
    pub vertex_count: u32,
    pub batch_count: u32,
    /// Offset of the packet in the payload.
    pub packet: usize,
    /// The zone column of the nearest enclosing type-3 node (+0x50, low byte), if any: the
    /// subtree draws only while the zone query passes for it (SPEC-0004 Drawing a node tree,
    /// item 3).
    pub zone: Option<u8>,
    /// The pass byte of the most recently entered type-4 node (+0x62): the sub-list of its material bucket the leaf
    /// joins (SPEC-0004 Drawing a node tree, 3b).
    pub pass: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeTree {
    pub leaves: Vec<Leaf>,
    /// Nodes visited (all types).
    pub nodes: usize,
    pub root_bounds: Bounds,
    /// The first distance-cull (type-4) node's floats at +0x50, +0x54, +0x58 (fade length) and
    /// +0x64 (fade end), for the foliage distances of SPEC-0060.
    pub lod: Option<[f32; 4]>,
}

fn bounds_at(b: &[u8], at: usize) -> Option<Bounds> {
    Some(Bounds {
        min: vec3_at(b, at + 0x10)?,
        size: vec3_at(b, at + 0x20)?,
        center: vec3_at(b, at + 0x30)?,
        radius: f32_at(b, at + 0x08)?,
    })
}

/// Walk the tree rooted at payload offset 0, following first-child and sibling links and the
/// three extra child links of distance-cull nodes. Offsets of 0 mean "none".
pub fn walk_tree(payload: &[u8]) -> Result<NodeTree, String> {
    walk_tree_layers(payload, None)
}

/// As `walk_tree`, but with a world layer mask: a distance-cull (type-4) node whose 32-bit layer
/// flags (+0x5C) are nonzero and share no bit with the mask is skipped with its subtree; its
/// sibling is still visited (SPEC-0046, Culling; SPEC-0004 node table).
pub fn walk_tree_layers(payload: &[u8], layer_mask: Option<u32>) -> Result<NodeTree, String> {
    let root_bounds = bounds_at(payload, 0).ok_or("Node tree shorter than one node")?;
    let mut leaves = Vec::new();
    let mut lod = None;
    let mut seen = HashSet::new();
    // Each entry carries the zone column of its nearest enclosing type-3 node.
    let mut stack: Vec<(usize, Option<u8>)> = vec![(0, None)];
    let mut pass = 0;
    while let Some((at, zone)) = stack.pop() {
        if !seen.insert(at) {
            continue;
        }
        if seen.len() > MAX_NODES {
            return Err("Node tree has too many nodes".into());
        }
        let bad = || format!("Node at {at:#x} runs past the payload");
        let kind = u32_at(payload, at).ok_or_else(bad)?;
        let size = match kind {
            NODE_GROUP => HEADER,
            NODE_VISIBILITY => 0x60,
            NODE_LOD => 0x80,
            NODE_LEAF => 0x90,
            other => return Err(format!("Node at {at:#x} has unknown type {other}")),
        };
        if at + size > payload.len() {
            return Err(bad());
        }
        if kind == NODE_LOD && lod.is_none() {
            lod = Some([0x50, 0x54, 0x58, 0x64].map(|o| f32_at(payload, at + o).unwrap_or(0.0)));
        }
        let hidden = kind == NODE_LOD
            && layer_mask.is_some_and(|m| {
                let flags = u32_at(payload, at + 0x5C).unwrap();
                flags != 0 && flags & m == 0
            });
        if kind == NODE_LOD && !hidden {
            pass = u8_at(payload, at + 0x62).unwrap();
        }
        // The sibling keeps the enclosing column; a type-3 node's own subtree takes its column.
        let inner = if kind == NODE_VISIBILITY { Some(u8_at(payload, at + 0x50).unwrap()) } else { zone };
        let mut links = vec![(u32_at(payload, at + 0x44).unwrap(), zone)];
        if !hidden {
            links.push((u32_at(payload, at + 0x40).unwrap(), inner));
            if kind == NODE_LOD {
                links.extend([0x70, 0x74, 0x78].map(|o| (u32_at(payload, at + o).unwrap(), inner)));
            }
        }
        for (link, z) in links {
            let link = link as usize;
            if link != 0 {
                if link % 4 != 0 || link >= payload.len() {
                    return Err(format!("Node at {at:#x} links outside the payload ({link:#x})"));
                }
                stack.push((link, z));
            }
        }
        if kind == NODE_LEAF {
            let packet = u32_at(payload, at + 0x50).unwrap() as usize;
            if packet >= payload.len() {
                return Err(format!("Leaf at {at:#x} has its packet outside the payload"));
            }
            leaves.push(Leaf {
                offset: at,
                bounds: bounds_at(payload, at).unwrap(),
                material_group: u16_at(payload, at + 0x5C).unwrap(),
                packet_kind: u8_at(payload, at + 0x5E).unwrap(),
                vertex_count: u32_at(payload, at + 0x64).unwrap(),
                batch_count: u32_at(payload, at + 0x68).unwrap(),
                packet,
                zone,
                pass,
            });
        }
    }
    leaves.sort_by_key(|l| l.offset);
    Ok(NodeTree {
        leaves,
        nodes: seen.len(),
        root_bounds,
        lod,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Position, colour, normal, skin weights, bone slots, UV.
    Skinned,
    /// Position, colour, UV.
    Plain,
    /// Position, colour, normal, UV.
    Lit,
    /// The CPU-animated leaves' draw packets (SPEC-0004 Multi-packet leaves): V4-32 positions
    /// (w = 1), a V4-32 colour array that is zero on disc (the vertex animator writes it each
    /// frame) and V4-32 UVs (z = w = 0). Decoded colours are neutral (128).
    Animated,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    pub layout: Layout,
    pub positions: Vec<[f32; 3]>,
    /// True where the vertex completes no triangle (flag bit 7).
    pub no_triangle: Vec<bool>,
    pub colors: Vec<[u8; 4]>,
    pub normals: Vec<[f32; 3]>,
    pub weights: Vec<[f32; 3]>,
    /// Matrix-palette addresses; bone index = byte / 3.
    pub bones: Vec<[u8; 4]>,
    pub uvs: Vec<[f32; 2]>,
    /// The second half of a two-pass batch's UV array.
    pub uvs_pass1: Vec<[f32; 2]>,
}

impl Batch {
    /// Strip triangles: vertex i (from 2) with a clear flag emits (i - 2, i - 1, i).
    pub fn triangles(&self) -> Vec<[u32; 3]> {
        (2..self.positions.len())
            .filter(|&i| !self.no_triangle[i])
            .map(|i| [i as u32 - 2, i as u32 - 1, i as u32])
            .collect()
    }
}

/// One UNPACK: format (vn, vl), the masked bit, its element count and raw data.
struct Unpack<'a> {
    vn: u32,
    vl: u32,
    masked: bool,
    num: usize,
    data: &'a [u8],
}

impl Unpack<'_> {
    fn is(&self, vn: u32, vl: u32) -> bool {
        self.vn == vn && self.vl == vl && !self.masked
    }

    fn f32s<const K: usize>(&self, n: usize) -> Vec<[f32; K]> {
        (0..n)
            .map(|i| std::array::from_fn(|k| f32_at(self.data, (i * K + k) * 4).unwrap_or(0.0)))
            .collect()
    }

    fn bytes4(&self, n: usize) -> Vec<[u8; 4]> {
        (0..n)
            .map(|i| std::array::from_fn(|k| self.data.get(i * 4 + k).copied().unwrap_or(0)))
            .collect()
    }
}

/// Byte length of an UNPACK's data: num elements of (vn + 1) components of (32 >> vl) bits,
/// padded to whole words. V4-5 (vl 3) packs a 16-bit element.
fn unpack_len(vn: u32, vl: u32, num: usize) -> usize {
    let bits = if vl == 3 { 16 } else { (vn as usize + 1) * (32 >> vl) };
    (num * bits).div_ceil(32) * 4
}

/// Split a packet's VIF stream into per-batch UNPACK lists. Each batch starts at a STMASK.
fn batches_of(stream: &[u8]) -> Result<Vec<Vec<Unpack<'_>>>, String> {
    let mut out: Vec<Vec<Unpack>> = Vec::new();
    let mut pos = 0;
    while pos + 4 <= stream.len() {
        let code = u32_at(stream, pos).unwrap();
        let cmd = (code >> 24) & 0x7f;
        let num = ((code >> 16) & 0xff) as usize;
        let imm = code & 0xffff;
        pos += 4;
        match cmd {
            0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x10 | 0x11 | 0x13 | 0x14
            | 0x15 | 0x17 => {}
            0x20 => {
                pos += 4;
                out.push(Vec::new());
            }
            0x30 | 0x31 => pos += 16,
            0x60..=0x7f => {
                let (vn, vl) = ((cmd >> 2) & 3, cmd & 3);
                let n = if num == 0 { 256 } else { num };
                let len = unpack_len(vn, vl, n);
                let data = stream
                    .get(pos..pos + len)
                    .ok_or("UNPACK data runs past the packet")?;
                pos += len;
                let addr = imm & 0x3ff;
                // The V4-32 to address 0 between batches carries the next batch's GIF tags.
                if vn == 3 && vl == 0 && addr == 0 {
                    continue;
                }
                if let Some(batch) = out.last_mut() {
                    batch.push(Unpack {
                        vn,
                        vl,
                        masked: cmd & 0x10 != 0,
                        num: n,
                        data,
                    });
                }
            }
            // Stream padding (zero or tool fill) or an unknown command ends the packet.
            _ => break,
        }
    }
    Ok(out)
}

fn decode_batch(unpacks: &[Unpack]) -> Result<Batch, String> {
    let flag = unpacks
        .iter()
        .find(|u| u.masked)
        .ok_or("Batch has no masked flag array")?;
    let arrays: Vec<&Unpack> = unpacks.iter().filter(|u| !u.masked).collect();
    let shape: Vec<(u32, u32)> = arrays.iter().map(|u| (u.vn, u.vl)).collect();
    const V3_32: (u32, u32) = (2, 0);
    const V2_32: (u32, u32) = (1, 0);
    const V4_8: (u32, u32) = (3, 2);
    const V4_32: (u32, u32) = (3, 0);
    if let [V4_32, V4_32, V4_32] = shape.as_slice() {
        let n = arrays[0].num;
        if arrays.iter().any(|u| u.num != n) || flag.num != n {
            return Err("Attribute arrays of one batch differ in length".into());
        }
        return Ok(Batch {
            layout: Layout::Animated,
            positions: arrays[0].f32s::<4>(n).into_iter().map(|v| [v[0], v[1], v[2]]).collect(),
            no_triangle: flag.data[..n].iter().map(|&f| f & 0x80 != 0).collect(),
            colors: vec![[128; 4]; n],
            normals: Vec::new(),
            weights: Vec::new(),
            bones: Vec::new(),
            uvs: arrays[2].f32s::<4>(n).into_iter().map(|v| [v[0], v[1]]).collect(),
            uvs_pass1: Vec::new(),
        });
    }
    let layout = match shape.as_slice() {
        [V3_32, V4_8, V3_32, V3_32, V4_8, V2_32] => Layout::Skinned,
        [V3_32, V4_8, V2_32] => Layout::Plain,
        [V3_32, V4_8, V3_32, V2_32] => Layout::Lit,
        other => return Err(format!("Unrecognised attribute layout {other:?}")),
    };
    let n = arrays[0].num;
    if arrays.iter().any(|u| u.num != n && !u.is(1, 0)) || flag.num != n {
        return Err("Attribute arrays of one batch differ in length".into());
    }
    let uv = arrays.last().unwrap();
    if uv.num != n && uv.num != 2 * n {
        return Err("UV array is neither N nor 2N entries".into());
    }
    let all_uvs = uv.f32s::<2>(uv.num);
    let mut b = Batch {
        layout,
        positions: arrays[0].f32s::<3>(n),
        no_triangle: flag.data[..n].iter().map(|&f| f & 0x80 != 0).collect(),
        colors: arrays[1].bytes4(n),
        normals: Vec::new(),
        weights: Vec::new(),
        bones: Vec::new(),
        uvs: all_uvs[..n].to_vec(),
        uvs_pass1: all_uvs[n..].to_vec(),
    };
    if layout != Layout::Plain {
        b.normals = arrays[2].f32s::<3>(n);
    }
    if layout == Layout::Skinned {
        b.weights = arrays[3].f32s::<3>(n);
        b.bones = arrays[4].bytes4(n);
    }
    Ok(b)
}

/// Decode a leaf's batches. A kind-1 leaf (CPU-animated, double-buffered; SPEC-0004 Multi-packet
/// leaves) holds the offsets of two byte-identical draw packets at packet + 0x3C (even parity)
/// and + 0x40 (odd), relative to the packet itself (on disc the two copies fill the payload's
/// end); the even copy is decoded as a one-packet leaf (the animation itself is the host's). Kind 2 is not supported (no leaf on disc uses it).
pub fn decode_packet(payload: &[u8], leaf: &Leaf) -> Result<Vec<Batch>, String> {
    if leaf.packet_kind == 1 {
        let rel = u32_at(payload, leaf.packet + 0x3C).ok_or("Truncated kind-1 packet header")? as usize;
        let draw = leaf.packet + rel;
        if rel == 0 || draw >= payload.len() {
            return Err(format!("Kind-1 draw packet {draw:#x} is outside the payload"));
        }
        return decode_packet(payload, &Leaf { packet_kind: 0, packet: draw, ..leaf.clone() });
    }
    if leaf.packet_kind != 0 {
        return Err(format!("Packet kind {} is not supported", leaf.packet_kind));
    }
    let qwc = u16_at(payload, leaf.packet + 2).ok_or("Truncated packet header")? as usize;
    // The header holds the first batch's GIF tags: two for two-pass (mode-2) meshes.
    let tags = u8_at(payload, leaf.packet + 0x0D).ok_or("Truncated packet header")?.max(1);
    let end = leaf.packet + qwc * 16;
    let stream = payload
        .get(leaf.packet + 16 * tags as usize..end)
        .ok_or("Packet runs past the payload")?;
    batches_of(stream)?
        .iter()
        .map(|u| decode_batch(u))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(kind: u32, child: u32, sibling: u32) -> Vec<u8> {
        let size = match kind {
            NODE_LOD => 0x80,
            NODE_VISIBILITY => 0x60,
            NODE_LEAF => 0x90,
            _ => HEADER,
        };
        let mut b = vec![0u8; size];
        b[0..4].copy_from_slice(&kind.to_le_bytes());
        b[0x40..0x44].copy_from_slice(&child.to_le_bytes());
        b[0x44..0x48].copy_from_slice(&sibling.to_le_bytes());
        b
    }

    /// SPEC-0004 synthetic layout-B batch of 3 vertices, as VIF codes.
    fn plain_batch(flags: [u8; 3]) -> Vec<u8> {
        let mut s = Vec::new();
        let word = |s: &mut Vec<u8>, w: u32| s.extend(w.to_le_bytes());
        word(&mut s, 0x2000_0000);
        word(&mut s, 0x3F3F_3F3F);
        word(&mut s, 0x6803_8001);
        for v in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            for c in v {
                s.extend(c.to_le_bytes());
            }
        }
        word(&mut s, 0x7203_8001);
        s.extend(flags);
        s.push(0);
        word(&mut s, 0x6E03_C004);
        s.extend([128, 64, 32, 128, 128, 128, 128, 128, 1, 2, 3, 4]);
        word(&mut s, 0x6403_8007);
        for uv in [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]] {
            for c in uv {
                s.extend(c.to_le_bytes());
            }
        }
        s
    }

    fn leaf_payload(stream: &[u8]) -> (Vec<u8>, Leaf) {
        leaf_payload_with_tags(stream, 1)
    }

    fn leaf_payload_with_tags(stream: &[u8], tags: u8) -> (Vec<u8>, Leaf) {
        let mut p = node(NODE_LEAF, 0, 0);
        p[0x50..0x54].copy_from_slice(&0x90u32.to_le_bytes());
        let header = 16 * tags as usize;
        let mut packet = vec![0u8; header];
        // A second tag's bytes must not be read as VIF codes.
        packet[header - 4..].copy_from_slice(&0x302E_4400u32.to_le_bytes());
        packet[0x0D] = tags;
        let qwc = (header + stream.len()).div_ceil(16);
        packet[2..4].copy_from_slice(&(qwc as u16).to_le_bytes());
        packet.extend(stream);
        packet.resize(qwc * 16, 0);
        p.extend(packet);
        let tree = walk_tree(&p).unwrap();
        (p, tree.leaves[0].clone())
    }

    #[test]
    fn strip_rule_matches_spec_vector() {
        let b = Batch {
            layout: Layout::Plain,
            positions: vec![[0.0; 3]; 6],
            no_triangle: [true, true, false, false, true, false].to_vec(),
            colors: vec![],
            normals: vec![],
            weights: vec![],
            bones: vec![],
            uvs: vec![],
            uvs_pass1: vec![],
        };
        assert_eq!(b.triangles(), [[0, 1, 2], [1, 2, 3], [3, 4, 5]]);
    }

    #[test]
    fn decodes_spec_plain_batch() {
        let mut stream = plain_batch([0x80, 0x80, 0x00]);
        stream.extend(plain_batch([0x80, 0x80, 0x80]));
        let (p, leaf) = leaf_payload(&stream);
        let batches = decode_packet(&p, &leaf).unwrap();
        assert_eq!(batches.len(), 2);
        let b = &batches[0];
        assert_eq!(b.layout, Layout::Plain);
        assert_eq!(b.positions[1], [1.0, 0.0, 0.0]);
        assert_eq!(b.colors[0], [128, 64, 32, 128]);
        assert_eq!(b.uvs[2], [0.0, 1.0]);
        assert_eq!(b.triangles(), [[0, 1, 2]]);
        assert!(batches[1].triangles().is_empty());
    }

    #[test]
    fn two_tag_headers_are_skipped_whole() {
        let (p, leaf) = leaf_payload_with_tags(&plain_batch([0x80, 0x80, 0x00]), 2);
        let batches = decode_packet(&p, &leaf).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].triangles(), [[0, 1, 2]]);
    }

    #[test]
    fn rejects_unknown_layouts_and_truncation() {
        let mut stream = plain_batch([0; 3]);
        // Turn the UV UNPACK (V2-32) into V3-32: no layout matches.
        let at = stream.len() - 4 * 7;
        stream[at + 3] = 0x68;
        let (p, leaf) = leaf_payload(&stream);
        assert!(decode_packet(&p, &leaf).is_err());
        let (p, leaf) = leaf_payload(&plain_batch([0; 3]));
        assert!(decode_packet(&p[..p.len() - 40], &leaf).is_err());
    }

    #[test]
    fn walks_links_including_lod_children() {
        // Root group -> visibility group -> LOD group whose +0x78 child is a group of two leaves.
        let mut p = node(NODE_GROUP, 0x50, 0);
        p.extend(node(NODE_VISIBILITY, 0xB0, 0));
        let mut lod = node(NODE_LOD, 0, 0);
        lod[0x78..0x7C].copy_from_slice(&0x130u32.to_le_bytes());
        p.extend(lod);
        p.extend(node(NODE_GROUP, 0x180, 0));
        let mut a = node(NODE_LEAF, 0, 0x210);
        a[0x50..0x54].copy_from_slice(&0x210u32.to_le_bytes());
        a[0x5C..0x5E].copy_from_slice(&7u16.to_le_bytes());
        p.extend(a);
        let mut b = node(NODE_LEAF, 0, 0);
        b[0x50..0x54].copy_from_slice(&0x2A0u32.to_le_bytes());
        p.extend(b);
        p.extend([0u8; 16]);
        let t = walk_tree(&p).unwrap();
        assert_eq!(t.nodes, 6);
        assert_eq!(t.leaves.iter().map(|l| l.offset).collect::<Vec<_>>(), [0x180, 0x210]);
        assert_eq!(t.leaves[0].material_group, 7);
        // A link past the end and an unknown node type are errors; a cycle is not followed.
        let mut bad = p.clone();
        bad[0x40..0x44].copy_from_slice(&0x9000u32.to_le_bytes());
        assert!(walk_tree(&bad).is_err());
        let mut odd = p.clone();
        odd[0x50] = 9;
        assert!(walk_tree(&odd).is_err());
        // A layer-flagged LOD node hides its subtree when the mask shares no bit with it.
        let mut layered = p.clone();
        layered[0xB0 + 0x5C..0xB0 + 0x60].copy_from_slice(&0x0020_0000u32.to_le_bytes());
        assert_eq!(walk_tree_layers(&layered, Some(0)).unwrap().leaves.len(), 0);
        assert_eq!(walk_tree_layers(&layered, Some(0x0020_0000)).unwrap().leaves.len(), 2);
        assert_eq!(walk_tree_layers(&layered, None).unwrap().leaves.len(), 2);
        // SPEC-0004 item 3: the leaves under the type-3 node carry its zone column (+0x50).
        assert_eq!(t.leaves.iter().map(|l| l.zone).collect::<Vec<_>>(), [Some(0), Some(0)]);
        let mut zoned = p.clone();
        zoned[0x50 + 0x50] = 5;
        assert_eq!(walk_tree(&zoned).unwrap().leaves.iter().map(|l| l.zone).collect::<Vec<_>>(), [Some(5), Some(5)]);
        let mut cycle = p.clone();
        cycle[0x180 + 0x44..0x180 + 0x48].copy_from_slice(&0u32.to_le_bytes());
        cycle[0x130 + 0x44..0x130 + 0x48].copy_from_slice(&0x50u32.to_le_bytes());
        assert!(walk_tree(&cycle).is_ok());
    }
}
