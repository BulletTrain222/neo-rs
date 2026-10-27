//! Shader lists (chunk type 2) and how a material group names its texture and palette
//! (SPEC-0003, "Materials: the shader list"). Only the texture-resolution fields are read here;
//! blend, filter and UV behaviour belong to the model and render milestones.
use crate::texture::{PSMCT32, Tex0, TextureChunk};

pub const CHUNK_TYPE: u16 = 2;
const GROUP_SIZE: usize = 0x270;
const STATE_BLOCK_0: usize = 0xD0;
const STATE_BLOCK_LEN: usize = 0xD0;
/// GS register numbers of TEX0 for drawing contexts 1 and 2.
const TEX0_1: u8 = 0x06;
const TEX0_2: u8 = 0x07;
/// GS register numbers of CLAMP for drawing context 2 (the first register set) and context 1
/// (the second set, render pass 1).
const CLAMP_2: u8 = 0x09;
const CLAMP_1: u8 = 0x08;
/// GS register numbers of ALPHA for drawing contexts 1 and 2 (public GS documentation).
const ALPHA_1: u8 = 0x42;
const ALPHA_2: u8 = 0x43;
/// Groups per list stay far below this on disc.
const MAX_GROUPS: u32 = 4096;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Group {
    /// 0 = no texture state, 1 = one texture context, 2 = two.
    pub mode: u8,
    /// Which of the list's texture chunks (0..B) the addresses refer to.
    pub texture_chunk: u8,
    /// The character colour slot (group byte 0x27, 0 to 14; SPEC-0003): a character part of
    /// this group is drawn with colour word j of the character's record (SPEC-0040 Behavior 26).
    pub colour_slot: u8,
    /// Pass 0's UV mode (group offset 0x4C: 0 none, 1 environment map, 2 scroll, 3 flipbook)
    /// and, for scroll, its speeds per main-loop pass (0x60, 0x64; SPEC-0003 UV modes).
    pub uv_mode: u32,
    /// The speeds' float bits (see `uv_speed`).
    pub uv_speed_bits: [u32; 2],
    /// TEX0 of the first register set (render pass 0), when the state block is well formed.
    pub tex0: Option<Tex0>,
    /// TEX0 of the second register set (render pass 1, mode 2 only).
    pub tex0_pass1: Option<Tex0>,
    /// CLAMP of the first register set: bits 0–1 the U wrap, 2–3 the V wrap (0 repeat, 1 clamp).
    pub clamp: Option<u64>,
    /// The ALPHA register of the state block (context 2's, else context 1's): fields A, B, C
    /// and D in bits 0 to 7 and FIX in bits 32 to 39 (SPEC-0034 2.8).
    pub alpha: Option<u64>,
    /// Pass-0 vertex program (material record, group offset 0x44): 0 and 4 are unlit, 1, 2 and
    /// 6 are lit, 3 is the billboard program; it also sets the vertex colour scale.
    pub program: u32,
    /// Render pass 1 (SPEC-0003 Material record; mode 2 groups only): its vertex program
    /// (group offset 0x54, 0xFFFF for none), UV mode (0x5C) and UV-mode data's scroll speeds
    /// (0x70, 0x74), and the second register set's CLAMP and ALPHA.
    pub program_pass1: u32,
    pub uv_mode_pass1: u32,
    pub uv_speed_pass1_bits: [u32; 2],
    pub clamp_pass1: Option<u64>,
    pub alpha_pass1: Option<u64>,
}

/// The UV scroll offset of one axis (SPEC-0003 UV modes, mode 2): s x N - trunc(s x N), with N
/// the 16-bit pass counter (it wraps at 65536).
pub fn scroll_offset(speed: f32, n: u16) -> f32 {
    let x = speed * n as f32;
    x - x.trunc()
}

impl Group {
    /// Pass 0's UV scroll speeds (su, sv).
    pub fn uv_speed(&self) -> [f32; 2] {
        self.uv_speed_bits.map(f32::from_bits)
    }

    /// The blend's B field is 2 (zero): source x alpha + destination, the additive rule of
    /// SPEC-0034 2.8. Every other setting (or none) blends by alpha.
    pub fn additive(&self) -> bool {
        self.alpha.is_some_and(|a| (a >> 2) & 3 == 2)
    }

    /// The colour byte that means 1.0 for this group's pass-0 program (SPEC-0004, Colour).
    pub fn colour_unit(&self) -> f32 {
        if matches!(self.program, 1 | 2 | 6) { 16.0 } else { 128.0 }
    }

    /// U and V repeat (true) or clamp (false); repeat when the state block is unreadable.
    pub fn repeats(&self) -> (bool, bool) {
        let c = self.clamp.unwrap_or(0);
        (c & 3 == 0, (c >> 2) & 3 == 0)
    }

    /// Whether the group draws a second pass: mode 2, a pass-1 TEX0 and a pass-1 program
    /// other than 0xFFFF (SPEC-0003 How a pass is drawn).
    pub fn two_pass(&self) -> bool {
        self.mode == 2 && self.tex0_pass1.is_some() && self.program_pass1 != 0xFFFF
    }

    /// Pass 1's scroll speeds (su, sv).
    pub fn uv_speed_pass1(&self) -> [f32; 2] {
        self.uv_speed_pass1_bits.map(f32::from_bits)
    }

    /// Pass 1's U and V repeat (true) or clamp (false).
    pub fn repeats_pass1(&self) -> (bool, bool) {
        let c = self.clamp_pass1.unwrap_or(0);
        (c & 3 == 0, (c >> 2) & 3 == 0)
    }
}

/// The registers this crate reads from a state block.
#[derive(Default)]
struct StateRegs {
    tex0: Option<Tex0>,
    tex0_pass1: Option<Tex0>,
    clamp: Option<u64>,
    alpha: Option<u64>,
    clamp_pass1: Option<u64>,
    alpha_pass1: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderList {
    pub groups: Vec<Group>,
    /// How many type-4 chunks after this list in the same chain belong to it (B).
    pub texture_chunks: u32,
}

/// Read a state block's packed A+D register list. A block that does not start with a GIF tag
/// of 6 or 12 registers is malformed (5 groups of resource 528 on disc) and yields None.
fn state_registers(block: &[u8]) -> Option<StateRegs> {
    let nloop = (u64_at(block, 0)? & 0x7fff) as usize;
    if nloop != 6 && nloop != 12 {
        return None;
    }
    let mut regs = StateRegs::default();
    for i in 0..nloop {
        let at = 16 + i * 16;
        let value = u64_at(block, at)?;
        match *block.get(at + 8)? {
            // Mode 1 lists only the first set, which uses registers 0x07 and 0x09; mode 2 adds
            // the second set (0x06, 0x08).
            TEX0_2 => regs.tex0 = Some(Tex0::decode(value)),
            TEX0_1 => regs.tex0_pass1 = Some(Tex0::decode(value)),
            CLAMP_2 => regs.clamp = Some(value),
            CLAMP_1 => regs.clamp_pass1 = Some(value),
            ALPHA_2 => regs.alpha = Some(value),
            ALPHA_1 => {
                regs.alpha = regs.alpha.or(Some(value));
                regs.alpha_pass1 = Some(value);
            }
            _ => {}
        }
    }
    Some(regs)
}

impl ShaderList {
    /// Parse a type-2 chunk payload.
    pub fn parse(payload: &[u8]) -> Result<Self, String> {
        let bad = || "Truncated shader list header".to_string();
        let table = u32_at(payload, 0x10).ok_or_else(bad)? as usize;
        let count = u32_at(payload, 0x14).ok_or_else(bad)?;
        let texture_chunks = u32_at(payload, 0x1C).ok_or_else(bad)?;
        if count > MAX_GROUPS {
            return Err(format!("{count} material groups is implausible"));
        }
        let groups = (0..count as usize)
            .map(|g| {
                let at = table + g * GROUP_SIZE;
                let group = payload
                    .get(at..at + GROUP_SIZE)
                    .ok_or_else(|| format!("Material group {g} runs past the shader list"))?;
                let mode = group[0x26];
                let block = &group[STATE_BLOCK_0..STATE_BLOCK_0 + STATE_BLOCK_LEN];
                let regs = if mode == 0 {
                    StateRegs::default()
                } else {
                    state_registers(block).unwrap_or_default()
                };
                Ok(Group {
                    mode,
                    texture_chunk: group[0x28],
                    colour_slot: group[0x27].min(14),
                    uv_mode: u32_at(group, 0x4C).unwrap_or(0),
                    uv_speed_bits: [u32_at(group, 0x60).unwrap_or(0), u32_at(group, 0x64).unwrap_or(0)],
                    tex0: regs.tex0,
                    tex0_pass1: regs.tex0_pass1,
                    clamp: regs.clamp,
                    alpha: regs.alpha,
                    program: u32_at(group, 0x44).unwrap_or(0),
                    program_pass1: u32_at(group, 0x54).unwrap_or(0xFFFF),
                    uv_mode_pass1: u32_at(group, 0x5C).unwrap_or(0),
                    uv_speed_pass1_bits: [u32_at(group, 0x70).unwrap_or(0), u32_at(group, 0x74).unwrap_or(0)],
                    clamp_pass1: regs.clamp_pass1,
                    alpha_pass1: if mode == 2 { regs.alpha_pass1 } else { None },
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            groups,
            texture_chunks,
        })
    }
}

/// Where a material's pass-0 texture lives: indexes into the list's texture chunks and that
/// chunk's sub-texture table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureRef {
    pub chunk: usize,
    pub texture: usize,
    pub palette: usize,
}

/// Resolve a group's pass-0 texture against the texture chunks that follow its shader list:
/// the texture is the entry in chunk k whose TBP and PSM equal TEX0's, and the palette is the
/// PSMCT32 entry whose TBP equals CBP. Searching by address alone across chunks is ambiguous.
pub fn resolve(group: &Group, chunks: &[TextureChunk]) -> Option<TextureRef> {
    resolve_tex0(group, group.tex0?, chunks)
}

/// Resolve the pass-1 texture (the second register set's TEX0) the same way.
pub fn resolve_pass1(group: &Group, chunks: &[TextureChunk]) -> Option<TextureRef> {
    resolve_tex0(group, group.tex0_pass1?, chunks)
}

fn resolve_tex0(group: &Group, t: Tex0, chunks: &[TextureChunk]) -> Option<TextureRef> {
    let k = group.texture_chunk as usize;
    let chunk = chunks.get(k)?;
    let texture = chunk
        .entries
        .iter()
        .position(|e| !e.is_palette() && e.tbp == t.tbp && e.psm == t.psm)?;
    let palette = chunk
        .entries
        .iter()
        .position(|e| e.psm == PSMCT32 && e.tbp == t.cbp)?;
    Some(TextureRef {
        chunk: k,
        texture,
        palette,
    })
}

/// A shader list and the texture chunks that belong to it, by position in the walked chain.
#[derive(Clone, Debug)]
pub struct ListTextures {
    /// None for texture chunks with no shader list before them (the front-end chunks).
    pub list: Option<ShaderList>,
    pub chunks: Vec<TextureChunk>,
}

/// Group the top-level chain's texture chunks under the shader list before them: each list owns
/// the next B type-4 chunks. Extra chunks are dropped, as the original drops them.
pub fn lists_in_chain(
    bytes: &[u8],
    chain: &[crate::resource::Chunk],
) -> Result<Vec<ListTextures>, String> {
    let mut out: Vec<ListTextures> = Vec::new();
    for c in chain {
        let payload = &bytes[c.payload.clone()];
        match c.kind {
            CHUNK_TYPE => out.push(ListTextures {
                list: Some(ShaderList::parse(payload)?),
                chunks: Vec::new(),
            }),
            crate::texture::CHUNK_TYPE => {
                let chunk = TextureChunk::parse(payload)?;
                match out.last_mut() {
                    Some(l) if l.list.is_none() => l.chunks.push(chunk),
                    Some(l) => {
                        let room = l.list.as_ref().map_or(0, |s| s.texture_chunks as usize);
                        if l.chunks.len() < room {
                            l.chunks.push(chunk);
                        }
                    }
                    None => out.push(ListTextures {
                        list: None,
                        chunks: vec![chunk],
                    }),
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {

    #[test]
    fn scroll_vectors() {
        // SPEC-0046 pack 1414: speeds (0, +0.00125) and (0, -0.00125): +0.75 and -0.75 after 600
        // passes, 0 after 800.
        assert!((scroll_offset(0.00125, 600) - 0.75).abs() < 1e-4);
        assert!((scroll_offset(-0.00125, 600) + 0.75).abs() < 1e-4);
        assert!(scroll_offset(0.00125, 800).abs() < 1e-4);
    }

    use super::*;

    fn tex0(tbp: u64, psm: u64, cbp: u64) -> u64 {
        tbp | 4 << 14 | psm << 20 | 5 << 26 | 5 << 30 | 1 << 34 | cbp << 37
    }

    /// A list with three groups: mode 1, mode 2, and one without the GIF tag.
    fn list() -> Vec<u8> {
        let mut b = vec![0u8; 0x20 + 3 * GROUP_SIZE + 4];
        b[0x10..0x14].copy_from_slice(&0x20u32.to_le_bytes());
        b[0x14..0x18].copy_from_slice(&3u32.to_le_bytes());
        b[0x18..0x1C].copy_from_slice(&(0x20 + 3 * GROUP_SIZE as u32).to_le_bytes());
        b[0x1C..0x20].copy_from_slice(&2u32.to_le_bytes());
        let mut put = |g: usize, mode: u8, k: u8, regs: &[(u64, u8)], tag: bool| {
            let at = 0x20 + g * GROUP_SIZE;
            b[at + 0x20] = g as u8;
            b[at + 0x26] = mode;
            b[at + 0x28] = k;
            let block = at + STATE_BLOCK_0;
            let mut pos = block;
            if tag {
                b[block..block + 8].copy_from_slice(&(regs.len() as u64 | 1 << 15).to_le_bytes());
                pos += 16;
            }
            for &(value, reg) in regs {
                b[pos..pos + 8].copy_from_slice(&value.to_le_bytes());
                b[pos + 8] = reg;
                pos += 16;
            }
        };
        let filler = |n: usize| vec![(0u64, 0x15u8); n];
        let mut one = filler(5);
        one.insert(0, (tex0(0x40, 0x13, 0x80), TEX0_2));
        put(0, 1, 1, &one, true);
        let mut two = filler(10);
        two.insert(0, (tex0(0x10, 0x14, 0x20), TEX0_2));
        two.insert(6, (tex0(0x30, 0x13, 0x80), TEX0_1));
        put(1, 2, 0, &two, true);
        put(2, 1, 0, &one, false);
        b
    }

    #[test]
    fn reads_groups_registers_and_chunk_count() {
        let l = ShaderList::parse(&list()).unwrap();
        assert_eq!(l.texture_chunks, 2);
        let [a, b, c] = &l.groups[..] else { panic!() };
        assert_eq!((a.mode, a.texture_chunk), (1, 1));
        let t = a.tex0.unwrap();
        assert_eq!((t.tbp, t.psm, t.cbp, t.tw), (0x40, 0x13, 0x80, 5));
        assert_eq!(a.tex0_pass1, None);
        assert_eq!(b.tex0.unwrap().tbp, 0x10);
        assert_eq!(b.tex0_pass1.unwrap().tbp, 0x30);
        assert_eq!((c.tex0, c.tex0_pass1), (None, None), "no GIF tag: no texture state");
    }

    #[test]
    fn resolves_texture_and_palette_in_the_groups_chunk_only() {
        use crate::texture::SubTexture;
        let entry = |tbp, psm| SubTexture {
            tbp,
            tbw: 4,
            width: 32,
            height: 32,
            psm,
            tex0: Tex0::decode(0),
        };
        // Both chunks hold a texture at 0x40; only chunk 1 also has the palette at 0x80.
        let c0 = TextureChunk::with_entries(vec![entry(0x40, 0x13), entry(0x60, 0x00)]);
        let c1 = TextureChunk::with_entries(vec![entry(0x00, 0x14), entry(0x40, 0x13), entry(0x80, 0x00)]);
        let l = ShaderList::parse(&list()).unwrap();
        let chunks = [c0, c1];
        assert_eq!(
            resolve(&l.groups[0], &chunks),
            Some(TextureRef { chunk: 1, texture: 1, palette: 2 })
        );
        // Group 1 names chunk 0, PSMT4 at 0x10: no such entry there.
        assert_eq!(resolve(&l.groups[1], &chunks), None);
        assert_eq!(resolve(&l.groups[2], &chunks), None, "no texture state");
        assert_eq!(resolve(&l.groups[0], &chunks[..1]), None, "chunk k missing");
    }

    #[test]
    fn rejects_truncated_lists() {
        let l = list();
        assert!(ShaderList::parse(&l[..0x18]).is_err());
        assert!(ShaderList::parse(&l[..0x20 + GROUP_SIZE]).is_err());
        let mut huge = l.clone();
        huge[0x14..0x18].copy_from_slice(&1_000_000u32.to_le_bytes());
        assert!(ShaderList::parse(&huge).is_err());
    }
}
