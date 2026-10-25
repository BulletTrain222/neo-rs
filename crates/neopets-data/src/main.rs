mod inspect_anim;
mod inspect_mesh;

use neopets_data::archive::ArchiveIndex;
use neopets_data::resource::{self, ChainEnd, Family};
use neopets_data::material;
use neopets_data::texture::{self, TextureChunk};
use neopets_data::{Disc, fingerprint, iso_path, png};
use std::collections::BTreeMap;

const USAGE: &str = "usage: neopets-data [--iso <path>] [--sha256 <disc file>]... \
                     [--resources] [--resource <n>]... [--texture <n>]... [--materials] [--meshes] [--clips] [--ground] [--areas] [--scene <n>]... [--coverage]";

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
    let mut show = Vec::new();
    let mut textures = Vec::new();
    let mut scenes = Vec::new();
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
    if !(survey || coverage || materials || meshes || clips || ground || areas || !show.is_empty() || !textures.is_empty() || !scenes.is_empty()) {
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
        inspect_mesh::areas(&mut disc)?;
    }
    if ground {
        inspect_mesh::ground(&mut disc, &index)?;
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
