use neopets_data::{Disc, fingerprint, iso_path};
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
    while let Some(arg) = args.next() {
        if arg == "--iso" {
            explicit = Some(args.next().ok_or("--iso needs a path")?.into());
        } else if arg == "--sha256" {
            hash.push(args.next().ok_or("--sha256 needs a disc file name")?);
        } else {
            return Err(format!(
                "Unknown argument {arg}; usage: neopets-data [--iso <path>] [--sha256 <disc file>]..."
            ));
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
    if !hash.is_empty() {
        println!("Digests are not compared: the expected fingerprint awaits a spec for R-0001.");
    }
    println!("Disc identity checks passed. This does not validate gameplay or all asset formats.");
    Ok(())
}
