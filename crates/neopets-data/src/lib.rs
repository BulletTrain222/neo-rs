//! Read-only access to a user-supplied disc. No emulator or executable interpreter.
pub mod fingerprint;
mod iso;
mod mods;
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
