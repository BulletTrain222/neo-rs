use neopets_data::archive::ArchiveIndex;
use neopets_data::resource::{self, ChainEnd, Family};
use neopets_data::{Disc, fingerprint, iso_path};
use std::collections::BTreeMap;

const USAGE: &str = "usage: neopets-data [--iso <path>] [--sha256 <disc file>]... \
                     [--resources] [--resource <n>]... [--coverage]";

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
    let mut show = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--iso" => explicit = Some(args.next().ok_or("--iso needs a path")?.into()),
            "--sha256" => hash.push(args.next().ok_or("--sha256 needs a disc file name")?),
            "--resources" => survey = true,
            "--coverage" => coverage = true,
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
    if !(survey || coverage || !show.is_empty()) {
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
            for c in &chain.chunks {
                *s.chunk_types.entry(c.kind).or_default() += 1;
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

/// docs/PROGRESS.md asset coverage: one line per type, "name decoded total". Nothing is
/// decoded for display yet, so every decoded count is 0 until the M3+ decoders land.
fn write_coverage(root: &std::path::Path, s: &Survey) -> Result<(), String> {
    let mut text = String::new();
    println!("Asset coverage (decoded / total):");
    for (family, total) in &s.families {
        if *family == Family::Empty {
            continue;
        }
        println!("  {:<16} 0 / {total}", family.name());
        text.push_str(&format!("{} 0 {total}\n", family.name()));
    }
    let path = root.join("private/asset-coverage.txt");
    std::fs::create_dir_all(root.join("private")).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| format!("Write {}: {e}", path.display()))?;
    println!("Wrote {}", path.display());
    Ok(())
}
