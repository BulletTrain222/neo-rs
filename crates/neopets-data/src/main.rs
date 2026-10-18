use neopets_data::{Disc, iso_path};
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
    while let Some(arg) = args.next() {
        if arg == "--iso" {
            explicit = Some(args.next().ok_or("--iso needs a path")?.into());
        } else {
            return Err(format!(
                "Unknown argument {arg}; usage: neopets-data [--iso <path>]"
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
    println!("Disc identity checks passed. This does not validate gameplay or all asset formats.");
    Ok(())
}
