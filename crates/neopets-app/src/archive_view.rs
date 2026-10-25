//! The browser's view of one archive resource: a text summary (SPEC-0001, SPEC-0002) and its
//! decoded textures (SPEC-0003).
use neopets_data::Disc;
use neopets_data::archive::ArchiveIndex;
use neopets_data::material;
use neopets_data::model::{self, ModelData};
use neopets_data::resource::{self, ChainEnd, Family};
use neopets_data::texture::{Image, TextureChunk};

/// Chunk types as a compact chain, collapsing runs: "2, 13 ×33, 4".
pub fn chain_summary(kinds: &[u16]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < kinds.len() {
        let run = kinds[i..].iter().take_while(|&&k| k == kinds[i]).count();
        parts.push(if run > 1 {
            format!("{} x{run}", kinds[i])
        } else {
            kinds[i].to_string()
        });
        i += run;
    }
    parts.join(", ")
}

pub fn describe(disc: &mut Disc, index: &ArchiveIndex, n: u32) -> Result<String, String> {
    let e = index.get(n)?;
    let mut text = format!(
        "Resource #{n} of {}: sectors {}+{} ({} KiB)",
        index.len(),
        e.start_sector,
        e.sector_count,
        e.readable_len() / 1024
    );
    if e.is_empty() {
        text.push_str("\nEmpty placeholder: no data on the disc.");
        return Ok(text);
    }
    let bytes = index.read(disc, n)?;
    let family = resource::classify(&bytes);
    text.push_str(&format!("\nFamily: {}", family.name()));
    if let Family::ChunkFile(_) = family {
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        let kinds: Vec<u16> = chain.chunks.iter().map(|c| c.kind).collect();
        text.push_str(&format!(
            "\nChunks ({}): {}",
            kinds.len(),
            chain_summary(&kinds)
        ));
        if let ChainEnd::Malformed(why) = chain.end {
            text.push_str(&format!("\nChain stops early: {why}"));
        }
    }
    Ok(text)
}

/// Every texture of a resource's top-level chain, labelled. Textures under a shader list come
/// once per distinct material pairing; chunks without one (front-end pictures) use each palette
/// of the right size, since nothing on the disc pairs them.
pub fn textures(
    disc: &mut Disc,
    index: &ArchiveIndex,
    n: u32,
) -> Result<Vec<(String, Image)>, String> {
    if index.get(n)?.is_empty() {
        return Ok(Vec::new());
    }
    let bytes = index.read(disc, n)?;
    let family = resource::classify(&bytes);
    if !matches!(family, Family::ChunkFile(_)) {
        return Ok(Vec::new());
    }
    let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
    let mut out = Vec::new();
    for l in material::lists_in_chain(&bytes, &chain.chunks)? {
        let memories: Vec<_> = l.chunks.iter().map(TextureChunk::gs_memory).collect();
        match &l.list {
            Some(list) => {
                let mut seen = std::collections::HashSet::new();
                for (g, group) in list.groups.iter().enumerate() {
                    let Some(r) = material::resolve(group, &l.chunks) else {
                        continue;
                    };
                    if seen.insert(r) {
                        let image =
                            l.chunks[r.chunk].decode(&memories[r.chunk], r.texture, r.palette)?;
                        out.push((format!("material group {g}"), image));
                    }
                }
            }
            None => {
                for (ci, chunk) in l.chunks.iter().enumerate() {
                    for t in chunk.textures().collect::<Vec<_>>() {
                        let palettes = chunk.palettes_for(t);
                        for &p in &palettes {
                            let label = if palettes.len() == 1 {
                                format!("chunk {ci} texture {t}")
                            } else {
                                format!("chunk {ci} texture {t}, palette {p} (pairing unknown)")
                            };
                            out.push((label, chunk.decode(&memories[ci], t, p)?));
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Every node tree of a resource as mesh parts (SPEC-0004), or None when it has none. The
/// held-item models 544–575 have no shader list and draw with resource 7's (SPEC-0004).
pub fn model(disc: &mut Disc, index: &ArchiveIndex, n: u32) -> Result<Option<ModelData>, String> {
    let chunk_file = |disc: &mut Disc, n: u32| -> Result<Option<(Vec<u8>, Vec<resource::Chunk>)>, String> {
        if index.get(n)?.is_empty() {
            return Ok(None);
        }
        let bytes = index.read(disc, n)?;
        let family = resource::classify(&bytes);
        if !matches!(family, Family::ChunkFile(_)) {
            return Ok(None);
        }
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        Ok(Some((bytes, chain.chunks)))
    };
    let Some((bytes, chunks)) = chunk_file(disc, n)? else {
        return Ok(None);
    };
    let shared = if (544..=575).contains(&n) { chunk_file(disc, 7)? } else { None };
    let data = model::build(
        &bytes,
        &chunks,
        shared.as_ref().map(|(b, c)| (b.as_slice(), c.as_slice())),
    )?;
    Ok((!data.parts.is_empty()).then_some(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_runs_of_chunk_types() {
        assert_eq!(chain_summary(&[]), "");
        assert_eq!(chain_summary(&[2, 13, 13, 13, 4]), "2, 13 x3, 4");
        assert_eq!(chain_summary(&[4, 4, 18, 4]), "4 x2, 18, 4");
    }
}
