//! The browser's view of one archive resource: a text summary (SPEC-0001, SPEC-0002) and its
//! decoded textures (SPEC-0003).
use neopets_data::Disc;
use neopets_data::archive::ArchiveIndex;
use neopets_data::material;
use neopets_data::model::{self, ModelData};
use neopets_data::resource::{self, ChainEnd, Family};
use neopets_data::texture::{Image, TextureChunk};

/// Chunk type of a world pack's header (SPEC-0006).
const WORLD_HEADER: u16 = 31;

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
    // A world pack (first chunk type 31, SPEC-0006) draws as its placed static scene.
    if chunks.first().is_some_and(|c| c.kind == WORLD_HEADER) {
        return world_scene(disc, index, n, &bytes, &chunks, true);
    }
    let shared = if (544..=575).contains(&n) { chunk_file(disc, 7)? } else { None };
    let data = model::build(&bytes, &chunks, shared.as_ref().map(|(b, c)| (b.as_slice(), c.as_slice())))?;
    Ok((!data.parts.is_empty()).then_some(data))
}

/// A sky dome resource's layer A (role 1) only: layer D (role 10, the purple cap and the
/// two-pass layer) draws only while the atmosphere blend factor is above 0 (SPEC-0046 Sky
/// domes, step 6), and play has no atmosphere zone pass yet, so the factor is 0.
pub fn dome(disc: &mut Disc, index: &ArchiveIndex, d: u32) -> Result<Option<ModelData>, String> {
    if index.get(d)?.is_empty() {
        return Ok(None);
    }
    let bytes = index.read(disc, d)?;
    let family = resource::classify(&bytes);
    let chain = resource::walk_chunks(&bytes, resource::chunk_limit(d, family))?;
    let data = model::build_roles(&bytes, &chain.chunks, None, Some(&[1]))?;
    Ok((!data.parts.is_empty()).then_some(data))
}

/// A world pack's placed static scene (mask 0, no story layer; SPEC-0046), with its sky dome's
/// layer A when `dome` is set (only the current world's dome is drawn in play).
pub fn world_scene(
    disc: &mut Disc,
    index: &ArchiveIndex,
    n: u32,
    bytes: &[u8],
    chunks: &[resource::Chunk],
    dome: bool,
) -> Result<Option<ModelData>, String> {
    let mut world = model::build_world_parts(bytes, chunks, 0, false)?;
    if dome {
        // The world's sky dome by its area slot; only layer A (role 1) draws without an
        // atmosphere zone (SPEC-0046 Sky domes).
        let slot = neopets_data::area::AreaTable::from_disc(disc)?.slot_of(n);
        let d = neopets_data::area::dome_for_slot(slot);
        if !index.get(d)?.is_empty() {
            let dome_bytes = index.read(disc, d)?;
            let family = resource::classify(&dome_bytes);
            let chain = resource::walk_chunks(&dome_bytes, resource::chunk_limit(d, family))?;
            world.append(model::build_roles(&dome_bytes, &chain.chunks, None, Some(&[1]))?);
        }
    }
    Ok((!world.parts.is_empty()).then_some(world))
}

/// A region character (SPEC-0006 kind-3 placement; SPEC-0005 families): the mesh of its model
/// set, drawn with the region's materials, and its animation families. The main family is the
/// region's own k-th type-22/25 pair, or a global set for a negative index (-7: header 492 and
/// body 526; -6: Tor's main set; -5: Roberta's); sub-part and face families are the region's
/// types 23/26 and 24/27. Each family's action table is the region's kind-2 record that the
/// placement names. Families that are missing or out of range are left out.
pub fn region_character(
    disc: &mut Disc,
    index: &ArchiveIndex,
    bytes: &[u8],
    region: &neopets_data::region::Region,
    placement: &neopets_data::region::Placement,
) -> Result<(ModelData, crate::character::HeroAnim), String> {
    use neopets_data::anim::{BoneTable, FamilyData};
    let set = usize::try_from(placement.model_set)
        .ok()
        .and_then(|k| region.model_sets.get(k))
        .ok_or("Model set out of range")?;
    // The region chain with only the chosen model set: shared chunks (materials, textures)
    // stay; the other model sets drop out.
    let chain: Vec<resource::Chunk> = region
        .chain
        .iter()
        .filter(|c| !(c.kind == 18 && bytes[c.payload.clone()].starts_with(b"IDM")) || c.payload == set.payload)
        .cloned()
        .collect();
    // SPEC-0004 3a: a region character draws with the region's mask, bit 30 (the colour
    // override that selects bit 31 needs the world's variant key, whose writer is open).
    // SPEC-0046 Second walk and attachment slots: a child with role 10 + k is the set's
    // attachment model k, drawn at its own attach point, not part of the body.
    let (trees, _) = model::find_trees(bytes, &chain);
    let mut roles: Vec<u16> = trees.iter().map(|t| t.role_id).collect();
    roles.sort_unstable();
    roles.dedup();
    let body: Vec<u16> = roles.iter().copied().filter(|r| !(10..=20).contains(r)).collect();
    let mut model = model::build_masked(bytes, &chain, None, Some(&body), Some(model::REGION_MASK))?;
    for r in roles.iter().copied().filter(|r| (10..=20).contains(r)) {
        let m = model::build_masked(bytes, &chain, None, Some(&[r]), Some(model::REGION_MASK))?;
        if !m.parts.is_empty() {
            model.attachments.push(((r - 10) as u8, m));
        }
    }
    let nested = resource::walk_chunks(&bytes[set.payload.clone()], None)?;
    let flat = nested
        .chunks
        .iter()
        .find(|c| c.kind == 18 && !bytes[set.payload.start + c.payload.start..].starts_with(b"IDM"))
        .ok_or("Model set has no bone table")?;
    let bones = BoneTable::parse(&bytes[set.payload.start + flat.payload.start..set.payload.start + flat.payload.end])?;
    let table = |i: i16| -> Option<&[u8]> {
        let r = region.records.tables.get(usize::try_from(i).ok()?)?;
        Some(&bytes[r.clone()])
    };
    let mut families = Vec::new();
    for f in 0..3 {
        let (s, t) = (placement.skeletons[f], placement.tables[f]);
        let Some(table) = table(t) else { continue };
        let family = if s >= 0 {
            let (Some(h), Some(b)) = (region.skeletons[f].get(s as usize), region.clip_sets[f].get(s as usize)) else { continue };
            FamilyData::parse(&bytes[h.payload.clone()], &bytes[b.payload.clone()], table)?
        } else if f == 0 && matches!(s, -7..=-5) {
            let (h, b) = match s {
                -7 => (492, 526),
                -6 => (492, 493),
                _ => (498, 499),
            };
            FamilyData::parse(&index.read(disc, h)?, &index.read(disc, b)?, table)?
        } else {
            continue;
        };
        families.push((f as u8, family));
    }
    if families.is_empty() {
        return Err("No animation family".into());
    }
    // NEOPETS_TRACE_RIG: each region character's families (role, skeleton, joints) and its bone
    // table size (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_RIG").is_some() {
        let f: Vec<(u8, i16, usize)> = families.iter().map(|(r, d)| (*r, placement.skeletons[*r as usize], d.def.len())).collect();
        println!("      rig {}: bones {}, families (role, skeleton, joints) {f:?}", placement.name, bones.len());
    }
    let mut anim = crate::character::HeroAnim::with_roles(bones, families);
    anim.set_scale_set(placement.scale_set);
    anim.play(0);
    Ok((model, anim))
}

/// A hero outfit (resources 504–519) with its hero's animation families, or None for any
/// other resource. The bone table is the outfit's flat type-18 chunk (SPEC-0005).
pub fn hero(
    disc: &mut Disc,
    index: &ArchiveIndex,
    n: u32,
) -> Result<Option<crate::character::HeroAnim>, String> {
    use neopets_data::anim::{BoneTable, FamilyData, hero_resources};
    if !(504..=519).contains(&n) {
        return Ok(None);
    }
    let bytes = index.read(disc, n)?;
    let chain = resource::walk_chunks(&bytes, None)?;
    let table = chain
        .chunks
        .iter()
        .find(|c| c.kind == 18 && !bytes[c.payload.clone()].starts_with(b"IDM"))
        .ok_or("Outfit has no bone table")?;
    let bones = BoneTable::parse(&bytes[table.payload.clone()])?;
    let hero = if n >= 512 { 1 } else { 0 };
    let families = hero_resources(hero)
        .into_iter()
        .map(|(h, b, t)| {
            FamilyData::parse(&index.read(disc, h)?, &index.read(disc, b)?, &index.read(disc, t)?)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut anim = crate::character::HeroAnim::new(bones, families);
    // Tor's spawn sets chain mode 1 (his tail), Roberta's mode 2 (two head chains).
    let dropped = anim.set_chains(if hero == 0 { neopets_sim::chain::tor() } else { neopets_sim::chain::roberta() });
    if dropped > 0 {
        bevy::log::warn!("Outfit {n}: {dropped} chain links do not hang from their link joints; left out");
    }
    Ok(Some(anim))
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
