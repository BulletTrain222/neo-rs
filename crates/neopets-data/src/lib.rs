//! Read-only access to a user-supplied disc. No emulator or executable interpreter.
pub mod anim;
pub mod area;
pub mod archive;
pub mod audio;
pub mod bank;
pub mod block;
mod elf;
pub mod design;
pub mod dialogue;
pub mod effect;
pub mod foliage;
pub mod fingerprint;
pub mod frontend;
pub mod gs;
pub mod ground;
pub mod gsmem;
pub mod item;
mod iso;
pub mod light;
pub mod lzo;
pub mod material;
pub mod mesh;
pub mod model;
pub mod monster;
pub mod navmesh;
mod mods;
pub mod png;
pub mod region;
pub mod resource;
pub mod texture;
pub mod track;
pub mod travel;
pub mod trigger;
pub mod world;
pub use iso::{Disc, Entry};
pub use mods::AssetSource;

use std::path::{Path, PathBuf};

/// An explicit path wins, then an environment override, then the private local setting.
pub fn iso_path(root: &Path, explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    if let Some(p) = std::env::var_os("NEOPETS_ISO") {
        return Ok(p.into());
    }
    let setting = root.join("private/iso-path.txt");
    let text = std::fs::read_to_string(&setting)
        .map_err(|_| "Choose your ISO with Set-ISO.cmd or pass --iso <path>.".to_string())?;
    let text = text.trim().trim_start_matches('\u{feff}');
    if text.is_empty() {
        return Err("The private ISO path is empty.".into());
    }
    let p = PathBuf::from(text);
    Ok(if p.is_absolute() { p } else { root.join(p) })
}
