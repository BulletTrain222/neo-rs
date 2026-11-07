//! Where the effect definition table lives (SPEC-0034 Layout section 2): the type-34 chunk of
//! resource 9. Its records are read by `neopets_sim::effect::EffectTable::parse`.
pub const CHUNK_TYPE: u16 = 34;
/// The resource that holds the shared group (group 3) and its shader list (2.8).
pub const RESOURCE: u32 = 9;

/// The first type-34 chunk payload in a walked top-level chain.
pub fn table_in_chain(bytes: &[u8], chain: &[crate::resource::Chunk]) -> Option<std::ops::Range<usize>> {
    chain.iter().find(|c| c.kind == CHUNK_TYPE).map(|c| c.payload.clone()).filter(|r| r.end <= bytes.len())
}

/// One material group of the resource-9 shader list as the effect sprites draw it (2.8): its
/// decoded texture (None when it does not resolve) and whether it blends additively.
#[derive(Clone, Debug)]
pub struct GroupImage {
    pub image: Option<crate::texture::Image>,
    pub repeat: (bool, bool),
    pub additive: bool,
}

/// Decode every group of the first shader list in a walked chain with its texture chunks.
pub fn group_images(bytes: &[u8], chain: &[crate::resource::Chunk]) -> Result<Vec<GroupImage>, String> {
    let lists = crate::material::lists_in_chain(bytes, chain)?;
    let Some(l) = lists.into_iter().find(|l| l.list.is_some()) else { return Ok(Vec::new()) };
    let list = l.list.unwrap();
    let mut memories: Vec<Option<crate::gsmem::GsMemory>> = (0..l.chunks.len()).map(|_| None).collect();
    Ok(list
        .groups
        .iter()
        .map(|g| {
            let image = crate::material::resolve(g, &l.chunks).zip(g.tex0).and_then(|(r, t0)| {
                let mem = memories[r.chunk].get_or_insert_with(|| l.chunks[r.chunk].gs_memory());
                l.chunks[r.chunk].decode_region(mem, r.texture, r.palette, 1 << t0.tw.min(10), 1 << t0.th.min(10)).ok()
            });
            GroupImage { image, repeat: g.repeats(), additive: g.additive() }
        })
        .collect())
}

/// The placed-effect chunk of a world pack (SPEC-0028 section 4; SPEC-0034 section 6).
pub const PLACED_CHUNK: u16 = 39;

/// A placed effect record (0x90 bytes): its frame rows (+0x00, +0x10, the direction +0x20),
/// position (+0x30), effect id (+0x40), zone number (+0x44) and cull (+0x84).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub frame: [[f32; 3]; 3],
    pub position: [f32; 3],
    pub id: i32,
    pub zone: u32,
    pub cull: f32,
    /// The per-placement motion scale S (+0x80; SPEC-0028 section 4, SPEC-0034 section 6).
    pub scale: f32,
}

/// The placed-effect records of a walked world pack chain, in slot order (a u32 count at +0,
/// records from +0x10, stride 0x90).
pub fn placed(bytes: &[u8], chain: &[crate::resource::Chunk]) -> Vec<Placed> {
    let Some(c) = chain.iter().find(|c| c.kind == PLACED_CHUNK) else { return Vec::new() };
    let Some(p) = bytes.get(c.payload.clone()) else { return Vec::new() };
    let f = |at: usize| p.get(at..at + 4).map(|b| f32::from_le_bytes(b.try_into().unwrap()));
    let u = |at: usize| p.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let n = u(0).unwrap_or(0) as usize;
    (0..n.min(4096))
        .map_while(|i| {
            let r = 0x10 + 0x90 * i;
            let v = |at: usize| Some([f(r + at)?, f(r + at + 4)?, f(r + at + 8)?]);
            Some(Placed { frame: [v(0)?, v(0x10)?, v(0x20)?], position: v(0x30)?, id: u(r + 0x40)? as i32, zone: u(r + 0x44)?, cull: f(r + 0x84)?, scale: f(r + 0x80).unwrap_or(1.0) })
        })
        .collect()
}
