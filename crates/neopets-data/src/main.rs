mod inspect_anim;
mod inspect_mesh;

use neopets_data::archive::ArchiveIndex;
use neopets_data::resource::{self, ChainEnd, Family};
use neopets_data::material;
use neopets_data::texture::{self, TextureChunk};
use neopets_data::{Disc, fingerprint, iso_path, png};
use std::collections::BTreeMap;

const USAGE: &str = "usage: neopets-data [--iso <path>] [--sha256 <disc file>]... \
                     [--resources] [--resource <n>]... [--texture <n>]... [--materials] [--meshes] [--clips] [--ground] [--areas] [--scene <n>]... [--coverage] [--lights]";

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut args = std::env::args().skip(1);
    let mut explicit = None;
    let mut hash = Vec::new();
    let mut survey = false;
    let mut coverage = false;
    let mut materials = false;
    let mut meshes = false;
    let mut clips = false;
    let mut ground = false;
    let mut areas = false;
    let mut effects = false;
    let mut show = Vec::new();
    let mut textures = Vec::new();
    let mut scenes = Vec::new();
    let mut registries = Vec::new();
    let mut lights = false;
    let mut blocks = false;
    let mut panels = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--iso" => explicit = Some(args.next().ok_or("--iso needs a path")?.into()),
            "--sha256" => hash.push(args.next().ok_or("--sha256 needs a disc file name")?),
            "--resources" => survey = true,
            "--coverage" => coverage = true,
            "--materials" => materials = true,
            "--meshes" => meshes = true,
            "--clips" => clips = true,
            "--ground" => ground = true,
            "--areas" => areas = true,
            "--effects" => effects = true,
            "--lights" => lights = true,
            "--blocks" => blocks = true,
            "--panels" => panels = true,
            "--registry" => {
                let n = args.next().ok_or("--registry needs a resource number")?;
                registries.push(n.parse::<u32>().map_err(|_| format!("Bad resource number {n}"))?);
            }
            "--scene" => {
                let n = args.next().ok_or("--scene needs a resource number")?;
                scenes.push(n.parse::<u32>().map_err(|_| format!("Bad resource number {n}"))?);
            }
            "--texture" => {
                let n = args.next().ok_or("--texture needs a resource number")?;
                textures.push(n.parse::<u32>().map_err(|_| format!("Bad resource number {n}"))?);
            }
            "--resource" => {
                let n = args.next().ok_or("--resource needs a number")?;
                show.push(n.parse::<u32>().map_err(|_| format!("Bad resource number {n}"))?);
            }
            _ => return Err(format!("Unknown argument {arg}; {USAGE}")),
        }
    }
    let path = iso_path(&root, explicit)?;
    let mut disc = Disc::open(&path)?;
    println!("{}", disc.verify_neopets()?);
    println!(
        "{} files indexed; ISO opened read-only.",
        disc.files().len()
    );
    for key in ["SYSTEM.CNF", "SCUS_973.67", "GAMEFILE.DAT"] {
        let entry = disc.files().get(key).ok_or("Missing required file")?;
        println!("{key}: {} bytes", entry.size);
    }
    for name in &hash {
        let digest = fingerprint::sha256_file(&mut disc, name)?;
        println!("SHA-256 {name}: {}", fingerprint::to_hex(&digest));
    }
    println!("Disc identity checks passed. This does not validate gameplay or all asset formats.");
    if !(panels || blocks || lights || effects || survey || coverage || materials || meshes || clips || ground || areas || !show.is_empty() || !textures.is_empty() || !scenes.is_empty() || !registries.is_empty()) {
        return Ok(());
    }
    let index = ArchiveIndex::from_disc(&mut disc)?;
    println!(
        "Boot program fingerprint matches SPEC-0001; archive index: {} resources, invariants hold.",
        index.len()
    );
    for &n in &show {
        describe(&mut disc, &index, n)?;
    }
    for &n in &textures {
        export_textures(&root, &mut disc, &index, n)?;
    }
    if materials {
        material_census(&mut disc, &index)?;
    }
    if effects {
        effect_census(&mut disc, &index)?;
    }
    if lights {
        light_census(&mut disc, &index)?;
    }
    for &n in &registries {
        // SPEC-0060 / SPEC-0004: the trees of a resource with their kind, depth and the u16 at
        // chunk +0x0E (the registry id of a type-8 child).
        let bytes = index.read(&mut disc, n)?;
        let chain = resource::walk_chunks(&bytes, None)?;
        let (trees, _) = neopets_data::model::find_trees(&bytes, &chain.chunks);
        let list: Vec<String> = trees.iter().filter(|t| t.depth > 0).map(|t| format!("{}:{}", t.kind, t.role_id)).collect();
        println!("Resource {n}: {} trees; children (kind:id) {}", trees.len(), list.join(" "));
        let patches = neopets_data::foliage::patches(&bytes, &chain.chunks);
        if let Some(p) = patches.first() {
            let rows: Vec<String> = p.rows.iter().map(|r| format!("({}, {:.4})", r.plants.len(), r.offset)).collect();
            println!(
                "  foliage: {} patches, {} plants; patch 0 box {:?} key {} height {:.4} rows {} plant 1 of row 0 at {:?}",
                patches.len(),
                patches.iter().flat_map(|p| &p.rows).map(|r| r.plants.len()).sum::<usize>(),
                p.area,
                p.key,
                p.height,
                rows.join(" "),
                p.rows.first().and_then(|r| r.plants.get(1)).map(|q| q.offset)
            );
        }
    }
    for &n in &scenes {
        inspect_mesh::scene(&mut disc, &index, n)?;
    }
    if meshes {
        inspect_mesh::run(&mut disc, &index)?;
    }
    if clips {
        inspect_anim::run(&mut disc, &index)?;
    }
    if areas {
        inspect_mesh::areas(&mut disc, &index)?;
    }
    if ground {
        inspect_mesh::ground(&mut disc, &index)?;
    }
    if blocks {
        block_census(&mut disc, &index)?;
    }
    if panels {
        panel_census(&mut disc, &index)?;
    }
    if survey || coverage {
        let report = survey_all(&mut disc, &index)?;
        if survey {
            report.print();
        }
        if coverage {
            write_coverage(&root, &report)?;
        }
    }
    Ok(())
}

/// SPEC-0034 Layout section 2: where the resource-9 effect table is, and the blend of the shader
/// list's groups (the records are read by the sim's parser; the app's `--effects` checks them).
/// SPEC-0028 section 2: the type-30 light sets of every chunk file, with the spec's counts and
/// resource 685's records 0 and 1; and the type-35 binary grids (SPEC-0006).
fn light_census(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    use neopets_data::light;
    let (mut packs, mut records, mut gated, mut counts, mut exact) = (0u32, 0usize, 0usize, [0usize; 3], 0u32);
    let (mut grids, mut blocks, mut square, mut unreadable) = (0u32, 0usize, 0usize, 0u32);
    for n in 0..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        let family = resource::classify(&bytes);
        if !matches!(family, Family::ChunkFile(_)) {
            continue;
        }
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        if let Some(g) = neopets_data::zone::zone_grid(&bytes, &chain.chunks) {
            grids += 1;
            blocks += g.len();
            square += usize::from(g.first().is_some_and(|b| b.width == b.height && b.key == -1));
        } else if chain.chunks.iter().any(|c| c.kind == neopets_data::zone::ZONE_CHUNK) {
            unreadable += 1;
        }
        let Some(c) = chain.chunks.iter().find(|c| c.kind == light::LIGHT_CHUNK) else { continue };
        let sets = light::light_sets(&bytes, &chain.chunks);
        // The chain ends exactly at the payload end when every record is read.
        let p = &bytes[c.payload.clone()];
        let n_words = u32::from_le_bytes(p[0..4].try_into().unwrap()) as usize;
        let mut at = (u32::from_le_bytes(p[4..8].try_into().unwrap()) as usize + 4 * n_words).next_multiple_of(16);
        for s in &sets {
            at = (at + 0x20 + 0x50 * (s.a.len() + s.b.len() + s.c.len())).next_multiple_of(16);
        }
        if sets.len() == n_words && at == p.len() {
            exact += 1;
        }
        packs += 1;
        records += sets.len();
        gated += sets.iter().filter(|s| s.gate != 0).count();
        for s in &sets {
            counts[0] += s.a.len();
            counts[1] += s.b.len();
            counts[2] += s.c.len();
        }
        if n == 685 {
            let zones: Vec<String> = sets.iter().map(|s| format!("{}{}", s.a.iter().chain(&s.b).chain(&s.c).next().map_or(-1, |e| e.zone as i32), if s.gate != 0 { "g" } else { "" })).collect();
            println!("  685: {} records, first-entry zones {}", sets.len(), zones.join(" "));
            if let Some(b) = neopets_data::zone::zone_grid(&bytes, &chain.chunks).and_then(|g| g.into_iter().next()) {
                let rows: Vec<String> = b.cells.chunks(b.width.max(1) as usize).map(|r| r.iter().map(|c| c.to_string()).collect::<String>()).collect();
                println!("  685 block 0 ({} x {}): {}", b.width, b.height, rows.join(" "));
            }
            for (i, s) in sets.iter().take(2).enumerate() {
                println!("    record {i}: gate {} counts ({}, {}, {})", s.gate, s.a.len(), s.b.len(), s.c.len());
                for e in s.a.iter().chain(&s.c) {
                    println!(
                        "      type {} flag {} strength {} position {:?} direction {:?} colour {:?} radius {} zone {}",
                        e.kind, e.flag, e.strength, e.position, e.direction, e.colour, e.radius, e.zone
                    );
                }
            }
        }
    }
    println!(
        "Light sets: {packs} packs ({exact} end exactly), {records} records ({gated} gated), entries A {} B {} C {} (total {})",
        counts[0],
        counts[1],
        counts[2],
        counts.iter().sum::<usize>()
    );
    println!("Binary grids (type 35): {grids} chunks ({unreadable} unreadable), {blocks} blocks, {square} with a square block 0 keyed -1");
    Ok(())
}

fn effect_census(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    use neopets_data::effect;
    let bytes = index.read(disc, effect::RESOURCE)?;
    let chain = resource::walk_chunks(&bytes, None)?;
    let range = effect::table_in_chain(&bytes, &chain.chunks).ok_or("Resource 9 has no type-34 chunk")?;
    let chunk = chain.chunks.iter().find(|c| c.kind == effect::CHUNK_TYPE).map_or(0, |c| c.offset);
    println!("Effect table: type-34 chunk at {chunk:#x}, payload {} bytes", range.len());
    let materials = neopets_data::material::lists_in_chain(&bytes, &chain.chunks)?;
    if let Some(list) = materials.iter().find_map(|l| l.list.as_ref()) {
        let additive = list.groups.iter().filter(|g| g.additive()).count();
        println!("  shader list: {} groups, {additive} additive", list.groups.len());
    }
    Ok(())
}

fn describe(disc: &mut Disc, index: &ArchiveIndex, n: u32) -> Result<(), String> {
    let e = index.get(n)?;
    let bytes = index.read(disc, n)?;
    let family = resource::classify(&bytes);
    println!(
        "Resource {n}: sectors {}+{} ({} bytes), {}",
        e.start_sector,
        e.sector_count,
        e.readable_len(),
        family.name()
    );
    if let Family::ChunkFile(_) = family {
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        for c in &chain.chunks {
            println!(
                "  chunk at {:#08x}: type {:>4}  extra {:>3}  size {:#x}",
                c.offset, c.kind, c.extra, c.size
            );
        }
        println!("  end: {:?}", chain.end);
    }
    Ok(())
}

#[derive(Default)]
struct Survey {
    families: BTreeMap<Family, u32>,
    ends: BTreeMap<String, u32>,
    chunk_types: BTreeMap<u16, u32>,
    malformed: Vec<(u32, String)>,
    /// Top-level texture chunks whose every texture decodes, and their total.
    texture_chunks: (u32, u32),
    /// Texture packs (front-end pictures) that decode.
    texture_packs_decoded: u32,
}

impl Survey {
    fn print(&self) {
        println!("Families (classified by bytes, SPEC-0002):");
        for (family, count) in &self.families {
            println!("  {:<16} {count}", family.name());
        }
        println!("Chunk-chain ends:");
        for (end, count) in &self.ends {
            println!("  {end:<16} {count}");
        }
        println!(
            "Top-level texture chunks fully decodable: {} of {}",
            self.texture_chunks.0, self.texture_chunks.1
        );
        println!("Top-level chunk types:");
        for (kind, count) in &self.chunk_types {
            println!("  type {kind:>4}: {count}");
        }
        for (n, why) in &self.malformed {
            println!("  malformed resource {n}: {why}");
        }
    }
}

/// Read every resource once (about 1.9 GB) to classify it and walk its chunk chain.
fn survey_all(disc: &mut Disc, index: &ArchiveIndex) -> Result<Survey, String> {
    let mut s = Survey::default();
    for n in 0..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        let family = resource::classify(&bytes);
        *s.families.entry(family).or_default() += 1;
        if let Family::ChunkFile(_) = family {
            let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
            let mut all_decode = true;
            for c in &chain.chunks {
                *s.chunk_types.entry(c.kind).or_default() += 1;
                if c.kind == texture::CHUNK_TYPE {
                    s.texture_chunks.1 += 1;
                    if texture_chunk_decodes(&bytes[c.payload.clone()]) {
                        s.texture_chunks.0 += 1;
                    } else {
                        all_decode = false;
                    }
                }
            }
            if family == Family::ChunkFile(4) && all_decode {
                s.texture_packs_decoded += 1;
            }
            let end = match &chain.end {
                ChainEnd::Terminator => "terminator",
                ChainEnd::BufferEnd => "buffer-end",
                ChainEnd::Limit => "limit",
                ChainEnd::Malformed(why) => {
                    s.malformed.push((n, why.clone()));
                    "malformed"
                }
            };
            *s.ends.entry(end.into()).or_default() += 1;
        }
    }
    Ok(s)
}

/// Every texture in the chunk decodes with some palette of the right size.
fn texture_chunk_decodes(payload: &[u8]) -> bool {
    let Ok(chunk) = TextureChunk::parse(payload) else {
        return false;
    };
    let mem = chunk.gs_memory();
    chunk.textures().collect::<Vec<_>>().into_iter().all(|t| {
        chunk
            .first_palette_for(t)
            .is_some_and(|p| chunk.decode(&mem, t, p).is_ok())
    })
}

/// docs/PROGRESS.md asset coverage: one line per type, "name decoded total". Texture chunks and
/// texture packs count as decoded once every texture in them decodes (SPEC-0003); other
/// families stay at 0 until their decoders land.
fn write_coverage(root: &std::path::Path, s: &Survey) -> Result<(), String> {
    let mut lines = vec![("texture-chunk".to_string(), s.texture_chunks.0, s.texture_chunks.1)];
    for (family, total) in &s.families {
        if *family == Family::Empty {
            continue;
        }
        let decoded = if *family == Family::ChunkFile(4) { s.texture_packs_decoded } else { 0 };
        lines.push((family.name(), decoded, *total));
    }
    let mut text = String::new();
    println!("Asset coverage (decoded / total):");
    for (name, decoded, total) in lines {
        println!("  {name:<16} {decoded} / {total}");
        text.push_str(&format!("{name} {decoded} {total}\n"));
    }
    let path = root.join("private/asset-coverage.txt");
    std::fs::create_dir_all(root.join("private")).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| format!("Write {}: {e}", path.display()))?;
    println!("Wrote {}", path.display());
    Ok(())
}

/// Decode a resource's textures to PNGs under private/textures/. Textures under a shader list
/// are exported once per material that uses them, with the material's palette
/// (r<n>_l<list>_g<group>.png). Chunks with no shader list (front-end pictures) use every palette
/// of the right size, since nothing on the disc pairs them (r<n>_c<chunk>_t<entry>[_p<pal>].png).
fn export_textures(
    root: &std::path::Path,
    disc: &mut Disc,
    index: &ArchiveIndex,
    n: u32,
) -> Result<(), String> {
    let bytes = index.read(disc, n)?;
    let family = resource::classify(&bytes);
    let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
    let lists = material::lists_in_chain(&bytes, &chain.chunks)?;
    let dir = root.join("private/textures");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let write = |name: String, image: &texture::Image| -> Result<(), String> {
        let file = png::encode_rgba(image.width, image.height, &image.rgba)?;
        std::fs::write(dir.join(&name), file).map_err(|e| e.to_string())
    };
    for (li, l) in lists.iter().enumerate() {
        let memories: Vec<_> = l.chunks.iter().map(TextureChunk::gs_memory).collect();
        let Some(list) = &l.list else {
            for (ci, chunk) in l.chunks.iter().enumerate() {
                for t in chunk.textures().collect::<Vec<_>>() {
                    let palettes = chunk.palettes_for(t);
                    for &pal in &palettes {
                        let image = chunk.decode(&memories[ci], t, pal)?;
                        let suffix = if palettes.len() > 1 { format!("_p{pal}") } else { String::new() };
                        let name = format!("r{n}_c{ci}_t{t}{suffix}.png");
                        println!("  {}x{} texture {t}, palette {pal} -> {name}", image.width, image.height);
                        write(name, &image)?;
                    }
                }
            }
            continue;
        };
        println!(
            "Resource {n} list {li}: {} groups, {} texture chunks",
            list.groups.len(),
            l.chunks.len()
        );
        let mut seen = std::collections::HashSet::new();
        for (g, group) in list.groups.iter().enumerate() {
            let Some(r) = material::resolve(group, &l.chunks) else {
                continue;
            };
            if !seen.insert(r) {
                continue;
            }
            let image = l.chunks[r.chunk].decode(&memories[r.chunk], r.texture, r.palette)?;
            let name = format!("r{n}_l{li}_g{g}.png");
            println!(
                "  group {g}: {}x{} (chunk {}, texture {}, palette {}) -> {name}",
                image.width, image.height, r.chunk, r.texture, r.palette
            );
            write(name, &image)?;
        }
    }
    Ok(())
}

/// Resolve every top-level material on the disc and decode each distinct texture once, to check
/// SPEC-0003's resolution rule and the decoder across every texture chunk.
fn material_census(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    let (mut lists, mut by_mode, mut resolved, mut decoded) = (0, [0u32; 3], 0u32, 0u32);
    let mut unresolved = Vec::new();
    let mut loose_textures = 0u32;
    for n in 0..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        let family = resource::classify(&bytes);
        if !matches!(family, Family::ChunkFile(_)) {
            continue;
        }
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        for l in material::lists_in_chain(&bytes, &chain.chunks).map_err(|e| format!("Resource {n}: {e}"))? {
            let memories: Vec<_> = l.chunks.iter().map(TextureChunk::gs_memory).collect();
            let Some(list) = &l.list else {
                for (ci, chunk) in l.chunks.iter().enumerate() {
                    for t in chunk.textures().collect::<Vec<_>>() {
                        if let Some(p) = chunk.first_palette_for(t) {
                            chunk.decode(&memories[ci], t, p).map_err(|e| format!("Resource {n}: {e}"))?;
                            loose_textures += 1;
                        }
                    }
                }
                continue;
            };
            lists += 1;
            let mut seen = std::collections::HashSet::new();
            for (g, group) in list.groups.iter().enumerate() {
                by_mode[(group.mode as usize).min(2)] += 1;
                if group.mode == 0 {
                    continue;
                }
                match material::resolve(group, &l.chunks) {
                    Some(r) => {
                        resolved += 1;
                        if seen.insert(r) {
                            l.chunks[r.chunk]
                                .decode(&memories[r.chunk], r.texture, r.palette)
                                .map_err(|e| format!("Resource {n} group {g}: {e}"))?;
                            decoded += 1;
                        }
                    }
                    None => unresolved.push((n, g, group.tex0.is_some())),
                }
            }
        }
    }
    println!("Top-level shader lists: {lists}");
    println!(
        "Groups by mode: none {}, one context {}, two contexts {}",
        by_mode[0], by_mode[1], by_mode[2]
    );
    println!("Textured groups resolved in their chunk: {resolved}; distinct textures decoded: {decoded}");
    println!("Textures in chunks without a shader list decoded: {loose_textures}");
    println!("Textured groups not resolved: {}", unresolved.len());
    for (n, g, well_formed) in unresolved.iter().take(20) {
        let why = if *well_formed { "no matching entry" } else { "malformed state block" };
        println!("  resource {n} group {g}: {why}");
    }
    Ok(())
}

/// SPEC-0006 Type 10: the ladders and climb panels of every chunk file, with the check that
/// each record's origin lies inside its group's radius.
fn panel_census(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    let (mut packs, mut ladders, mut panels, mut outside) = (0u32, 0usize, 0usize, 0usize);
    for n in 0..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        let family = resource::classify(&bytes);
        if !matches!(family, Family::ChunkFile(_)) {
            continue;
        }
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        let recs = neopets_data::trigger::panels(&bytes, &chain.chunks);
        if recs.is_empty() {
            continue;
        }
        packs += 1;
        let here = recs.iter().filter(|r| r.kind == -3).count();
        if here > 0 {
            print!("{n}:{here} ");
        }
        ladders += here;
        panels += recs.iter().filter(|r| r.kind == -2).count();
        let groups = neopets_data::trigger::panel_groups(&bytes, &chain.chunks);
        let mut k = 0;
        for (c, r2, count) in groups {
            for rec in recs.iter().skip(k).take(count) {
                let d2: f32 = (0..3).map(|i| (rec.origin[i] - c[i]).powi(2)).sum();
                if d2 > r2 * 1.0001 + 1e-3 {
                    outside += 1;
                }
            }
            k += count;
        }
        if std::env::var("NEOPETS_PANELS_PACK").ok().and_then(|v| v.parse::<u32>().ok()) == Some(n) {
            for r in recs.iter().filter(|r| r.kind == -3) {
                println!("  ladder origin {:?} n {:?} u {:?} v {:?} lu {} lv {}", r.origin, r.normal, r.u, r.v, r.lu, r.lv);
            }
        }
    }
    println!("type 10: {ladders} ladders and {panels} climb panels in {packs} packs; {outside} records outside their group radius");
    Ok(())
}

/// SPEC-0048 Source: the movable blocks of every chunk file (the spec counts 90 in 17 packs).
fn block_census(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    let (mut packs, mut total, mut meshes) = (0u32, 0usize, 0usize);
    for n in 0..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        let family = resource::classify(&bytes);
        if !matches!(family, Family::ChunkFile(_)) {
            continue;
        }
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        let b = neopets_data::trigger::movable_blocks(&bytes, &chain.chunks);
        if b.is_empty() {
            continue;
        }
        packs += 1;
        total += b.len();
        meshes += b.iter().filter(|b| b.mesh.is_some()).count();
        let mut flags: Vec<u16> = b.iter().map(|b| b.flags).collect();
        flags.sort_unstable();
        flags.dedup();
        print!("flags {flags:x?} ");
        let first = &b[0];
        if std::env::var("NEOPETS_BLOCKS_PACK").ok().and_then(|v| v.parse::<u32>().ok()) == Some(n) {
            for (k, b) in b.iter().enumerate() {
                println!("  block {k}: size {} height {} corner {:?} extents {:?}", b.size, b.height, b.corner, b.extents);
            }
        }
        println!("pack {n}: {} blocks; block 0 size {} height {} model {} corner {:?} axes {:?} extents {:?} mesh {}", b.len(), first.size, first.height, first.model, first.corner, first.axes, first.extents, first.mesh.is_some());
    }
    println!("movable blocks: {total} in {packs} packs, {meshes} with an inline mesh");
    Ok(())
}
