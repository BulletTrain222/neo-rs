use crate::{Disc, iso::normalize};
use std::path::{Path, PathBuf};

/// Explicit priority: later roots override earlier roots, then fall back to the ISO.
pub struct AssetSource {
    pub disc: Disc,
    roots: Vec<PathBuf>,
}
impl AssetSource {
    pub fn new(disc: Disc, roots: &[PathBuf]) -> Result<Self, String> {
        let roots = roots
            .iter()
            .map(|p| {
                p.canonicalize()
                    .map_err(|e| format!("Mod {}: {e}", p.display()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { disc, roots })
    }
    pub fn read(&mut self, name: &str) -> Result<Vec<u8>, String> {
        let key = normalize(name)?;
        for root in self.roots.iter().rev() {
            let candidate = root.join(Path::new(&key));
            match candidate.canonicalize() {
                Ok(path) => {
                    if !path.starts_with(root) {
                        return Err("Mod asset resolves outside its mod folder".into());
                    }
                    let size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
                    if size > 64 * 1024 * 1024 {
                        return Err("Mod asset exceeds the 64 MiB read limit".into());
                    }
                    return std::fs::read(path).map_err(|e| e.to_string());
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        self.disc.read(&key)
    }
}
