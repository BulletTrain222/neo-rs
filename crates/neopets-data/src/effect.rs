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
