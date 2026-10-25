//! The front-end UI database (SPEC-0035 Layout): resource 532's type-38 payload (header,
//! section table, key-to-node map C, screen records D, sprite records E, label bindings G, the
//! UI node tree J, value-text records K and asset paths O) and its type-4 texture sheet, whose
//! sub-textures the sheet assets name by their TEX0 words (SPEC-0023 HUD screen and art).
//! Strings, pages, item targets, animation tracks and slots, and fonts are not read yet.

use crate::resource::{self, Chunk};
use crate::gsmem::GsMemory;
use crate::texture::{Image, Tex0, TextureChunk};
use std::collections::HashMap;

pub const RESOURCE: u32 = 532;
const DATABASE: u16 = 38;
const TEXTURES: u16 = 4;
const NODE: usize = 52;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn i16_at(b: &[u8], at: usize) -> Option<i16> {
    u16_at(b, at).map(|v| v as i16)
}

/// A section J node (52 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Node {
    /// The node's data as a payload offset (a sprite's E record, a text's G, a value's K).
    pub data: u32,
    pub child: Option<usize>,
    pub sibling: Option<usize>,
    pub parent: Option<usize>,
    pub x: i16,
    pub y: i16,
    /// Scale, 4.12 fixed point.
    pub scale_x: i16,
    pub scale_y: i16,
    pub height: i16,
    pub width: i16,
    /// Predicate handler index (-1 none), search key (0xFFFF none), row spacing.
    pub handler: i16,
    pub key: u16,
    pub spacing: u8,
    pub kind: u8,
    /// Condition mode, absolute flag, alignment, predicate argument.
    pub mode: u8,
    pub absolute: bool,
    pub align: u8,
    pub arg: u8,
}

/// A section E sprite record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sprite {
    pub colour: [u8; 4],
    pub asset: u32,
    pub columns: u8,
    pub rows: u8,
    pub frames: u8,
    pub frame: u8,
}

/// A section K value-text record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValueText {
    pub colour: [u8; 4],
    pub variable: u16,
    pub font: u16,
    pub spacing: i16,
    pub gate: u8,
    pub format: u8,
}

/// A section D screen record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Screen {
    pub node: Option<usize>,
    pub back: i16,
    pub state: u16,
}

/// A section O asset: a sheet asset's TEX0 (kind word 2), or a full-screen picture number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asset {
    Sheet(Tex0),
    Picture(u32),
    Other,
}

#[derive(Clone, Debug, Default)]
pub struct Frontend {
    pub nodes: Vec<Node>,
    /// Section C: key to node (the last record wins).
    pub keys: HashMap<u32, usize>,
    pub screens: Vec<Screen>,
    /// Sprite and value-text records by the payload offset a node's data names.
    pub sprites: HashMap<u32, Sprite>,
    pub values: HashMap<u32, ValueText>,
    pub assets: Vec<Asset>,
    pub texture: Option<TextureChunk>,
}

impl Frontend {
    /// Parse resource 532's chunk chain.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let chain = resource::walk_chunks(bytes, None)?;
        let db = chain.chunks.iter().find(|c| c.kind == DATABASE).ok_or("No front-end database chunk")?;
        let mut f = Self::parse_database(&bytes[db.payload.clone()])?;
        f.texture = chain.chunks.iter().find(|c: &&Chunk| c.kind == TEXTURES).and_then(|c| TextureChunk::parse(&bytes[c.payload.clone()]).ok());
        Ok(f)
    }

    /// The type-38 payload: the version check (1.05), then the sections.
    pub fn parse_database(p: &[u8]) -> Result<Self, String> {
        let bad = || "Truncated front-end database".to_string();
        if (u16_at(p, 0x10), u16_at(p, 0x12)) != (Some(1), Some(5)) {
            return Err("Front-end database version is not 1.05".into());
        }
        let count = |at: usize| u32_at(p, at).map(|v| v as usize).ok_or_else(bad);
        // Section starts in file order A, T, C, D, E, F, G, H, I, J, K, L, M, N, O, tail.
        let start = |i: usize| u32_at(p, 0x54 + 4 * i).map(|v| v as usize).ok_or_else(bad);
        let (n_d, n_e, n_j, n_k, n_o, n_c) = (count(0x14)?, count(0x18)?, count(0x2C)?, count(0x30)?, count(0x40)?, count(0x50)?);
        let (c0, d0, e0, j0, k0, o0) = (start(2)?, start(3)?, start(4)?, start(9)?, start(10)?, start(14)?);
        let node_of = |off: u32| -> Option<usize> {
            let off = off as usize;
            (off >= j0 && (off - j0) % NODE == 0 && (off - j0) / NODE < n_j).then(|| (off - j0) / NODE)
        };
        let mut nodes = Vec::with_capacity(n_j);
        for i in 0..n_j {
            let r = j0 + NODE * i;
            let w = |o: usize| u32_at(p, r + o).ok_or_else(bad);
            let s = |o: usize| i16_at(p, r + o).ok_or_else(bad);
            let b = |o: usize| p.get(r + o).copied().ok_or_else(bad);
            nodes.push(Node {
                data: w(0x00)?,
                child: node_of(w(0x04)?),
                sibling: node_of(w(0x08)?),
                parent: node_of(w(0x0C)?),
                x: s(0x14)?,
                y: s(0x16)?,
                scale_x: s(0x18)?,
                scale_y: s(0x1A)?,
                height: s(0x20)?,
                width: s(0x22)?,
                handler: s(0x24)?,
                key: s(0x28)? as u16,
                spacing: b(0x2A)?,
                kind: b(0x2B)?,
                mode: b(0x2C)?,
                absolute: b(0x2D)? != 0,
                align: b(0x2E)?,
                arg: b(0x2F)?,
            });
        }
        let mut keys = HashMap::new();
        for i in 0..n_c {
            let (k, n) = (u32_at(p, c0 + 8 * i).ok_or_else(bad)?, u32_at(p, c0 + 8 * i + 4).ok_or_else(bad)?);
            if (n as usize) < n_j {
                keys.insert(k, n as usize);
            }
        }
        let screens = (0..n_d)
            .map(|i| {
                let r = d0 + 20 * i;
                Some(Screen { node: node_of(u32_at(p, r)?), back: i16_at(p, r + 4)?, state: u16_at(p, r + 0x0A)? })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(bad)?;
        let mut sprites = HashMap::new();
        for i in 0..n_e {
            let r = e0 + 28 * i;
            let g = p.get(r + 0x0E..r + 0x11).ok_or_else(bad)?;
            sprites.insert(
                r as u32,
                Sprite {
                    colour: p.get(r + 4..r + 8).ok_or_else(bad)?.try_into().unwrap(),
                    asset: u32_at(p, r + 8).ok_or_else(bad)?,
                    columns: p[r + 0x0C],
                    rows: p[r + 0x0D],
                    frames: g.iter().map(|&v| v as u32).sum::<u32>().min(255) as u8,
                    frame: p.get(r + 0x14).copied().ok_or_else(bad)?,
                },
            );
        }
        let mut values = HashMap::new();
        for i in 0..n_k {
            let r = k0 + 20 * i;
            values.insert(
                r as u32,
                ValueText {
                    colour: p.get(r + 4..r + 8).ok_or_else(bad)?.try_into().unwrap(),
                    variable: u16_at(p, r + 8).ok_or_else(bad)?,
                    font: u16_at(p, r + 0x0A).ok_or_else(bad)?,
                    spacing: i16_at(p, r + 0x0C).ok_or_else(bad)?,
                    gate: p.get(r + 0x0E).copied().ok_or_else(bad)?,
                    format: p.get(r + 0x0F).copied().ok_or_else(bad)?,
                },
            );
        }
        let assets = (0..n_o)
            .map(|i| {
                let r = o0 + 136 * i;
                let kind = u32_at(p, r + 8)?;
                Some(match kind {
                    2 => Asset::Sheet(Tex0::decode(u64::from_le_bytes(p.get(r..r + 8)?.try_into().ok()?))),
                    0xFFFF_FFFF => Asset::Picture(u32_at(p, r + 0x0C)?),
                    _ => Asset::Other,
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(bad)?;
        Ok(Self { nodes, keys, screens, sprites, values, assets, texture: None })
    }

    /// The node a section C key names.
    pub fn key(&self, k: u32) -> Option<usize> {
        self.keys.get(&k).copied()
    }

    /// A sprite node's record.
    pub fn sprite(&self, node: usize) -> Option<&Sprite> {
        let n = self.nodes.get(node)?;
        if n.kind != 1 {
            return None;
        }
        self.sprites.get(&n.data)
    }

    /// The texture and palette entries of a sheet asset: the sub-texture whose base block and
    /// format equal the TEX0's, and the 32-bit entry at its palette base.
    pub fn sheet_entries(&self, asset: u32) -> Option<(usize, usize)> {
        let Some(Asset::Sheet(t)) = self.assets.get(asset as usize).copied() else { return None };
        let tex = self.texture.as_ref()?;
        let texture = tex.entries.iter().position(|e| !e.is_palette() && e.tbp == t.tbp && e.psm == t.psm)?;
        let palette = tex.entries.iter().position(|e| e.is_palette() && e.tbp == t.cbp)?;
        Some((texture, palette))
    }

    /// The art of a sprite node: `width` x `height` texels from its frame cell's top-left,
    /// clamped at the sub-texture's edges (SPEC-0023: clamp on both axes).
    pub fn sprite_image(&self, mem: &GsMemory, node: usize, width: u32, height: u32) -> Option<Image> {
        let n = self.nodes.get(node)?;
        let s = self.sprite(node)?;
        let (texture, palette) = self.sheet_entries(s.asset)?;
        let tex = self.texture.as_ref()?;
        let sub = tex.entries.get(texture)?;
        let full = tex.decode(mem, texture, palette).ok()?;
        let cols = s.columns.max(1) as u32;
        let f = s.frame as u32;
        let (cx, cy) = ((f % cols).min(cols - 1), (f / cols).min(s.rows.max(1) as u32 - 1));
        let (w0, h0) = (n.width.max(0) as u32, n.height.max(0) as u32);
        let (ox, oy) = (cx * w0, cy * h0);
        let (width, height) = (width.clamp(1, 1024), height.clamp(1, 1024));
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let sx = (ox + x).min(sub.width.max(1) - 1);
                let sy = (oy + y).min(sub.height.max(1) - 1);
                let i = ((sy * full.width + sx) * 4) as usize;
                rgba.extend_from_slice(full.rgba.get(i..i + 4)?);
            }
        }
        Some(Image { width, height, rgba })
    }

    /// The texture sheet's emulated GS memory (build once, pass to `sprite_image`).
    pub fn memory(&self) -> Option<GsMemory> {
        self.texture.as_ref().map(|t| t.gs_memory())
    }

    /// The children of a node in sibling order.
    pub fn children(&self, node: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut c = self.nodes.get(node).and_then(|n| n.child);
        while let Some(i) = c {
            if out.contains(&i) {
                break;
            }
            out.push(i);
            c = self.nodes[i].sibling;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_header_and_tree() {
        // A synthetic payload: header with version 1.05, one D record naming node 0, a C key
        // 0x31 for node 1, and two J nodes (0 the parent of 1).
        let mut p = vec![0u8; 0x400];
        p[0x10..0x12].copy_from_slice(&1u16.to_le_bytes());
        p[0x12..0x14].copy_from_slice(&5u16.to_le_bytes());
        let put32 = |p: &mut Vec<u8>, at: usize, v: u32| p[at..at + 4].copy_from_slice(&v.to_le_bytes());
        let (c0, d0, e0, j0, k0, o0) = (0xA0usize, 0xB0, 0xD0, 0x100, 0x200, 0x220);
        put32(&mut p, 0x14, 1);
        put32(&mut p, 0x2C, 2);
        put32(&mut p, 0x50, 1);
        for (i, s) in [(2, c0), (3, d0), (4, e0), (9, j0), (10, k0), (14, o0)] {
            put32(&mut p, 0x54 + 4 * i, s as u32);
        }
        put32(&mut p, c0, 0x31);
        put32(&mut p, c0 + 4, 1);
        put32(&mut p, d0, j0 as u32);
        put32(&mut p, j0 + 0x04, (j0 + NODE) as u32);
        put32(&mut p, j0 + NODE + 0x0C, j0 as u32);
        p[j0 + NODE + 0x14..j0 + NODE + 0x16].copy_from_slice(&71i16.to_le_bytes());
        p[j0 + NODE + 0x2B] = 1;
        let f = Frontend::parse_database(&p).unwrap();
        assert_eq!(f.nodes.len(), 2);
        assert_eq!(f.screens[0].node, Some(0));
        assert_eq!(f.key(0x31), Some(1));
        assert_eq!((f.nodes[1].parent, f.nodes[1].x, f.nodes[1].kind), (Some(0), 71, 1));
        assert_eq!(f.children(0), vec![1]);
        p[0x12] = 6;
        assert!(Frontend::parse_database(&p).is_err(), "version 1.06 is refused");
    }
}
