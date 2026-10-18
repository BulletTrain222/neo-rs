use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const SECTOR: u64 = 2048;
const MAX_DIRECTORY: u64 = 16 * 1024 * 1024;
const MAX_READ: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Entry {
    pub offset: u64,
    pub size: u64,
}

/// Minimal bounded ISO9660 reader; unsupported layouts fail instead of being guessed.
pub struct Disc {
    file: File,
    length: u64,
    entries: BTreeMap<String, Entry>,
}

fn le32(b: &[u8], at: usize) -> u64 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as u64
}

pub(crate) fn normalize(name: &str) -> Result<String, String> {
    let n = name.replace('\\', "/");
    let mut parts = Vec::new();
    for p in n.trim_start_matches('/').split('/') {
        if p.is_empty() || p == "." || p == ".." || p.contains(':') || p.contains('\0') {
            return Err("Invalid asset path".into());
        }
        let p = p.split(';').next().unwrap();
        if p.is_empty() {
            return Err("Empty asset name".into());
        }
        parts.push(p.to_ascii_uppercase());
    }
    Ok(parts.join("/"))
}

impl Disc {
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Open ISO {}: {e}", path.display()))?;
        let length = file.metadata().map_err(|e| e.to_string())?.len();
        let mut disc = Self {
            file,
            length,
            entries: BTreeMap::new(),
        };
        let mut primary = None;
        for sector in 16..80 {
            let vd = disc.bytes(sector * SECTOR, SECTOR)?;
            if &vd[1..6] != b"CD001" || vd[6] != 1 {
                return Err("Expected an uncompressed ISO9660 image with 2048-byte sectors".into());
            }
            if vd[0] == 1 {
                primary = Some(vd);
                break;
            }
            if vd[0] == 255 {
                break;
            }
        }
        let pvd = primary.ok_or("No ISO9660 primary volume descriptor")?;
        if u16::from_le_bytes([pvd[128], pvd[129]]) != 2048 {
            return Err("Unsupported logical block size".into());
        }
        if pvd[156] < 34 {
            return Err("Invalid root directory record".into());
        }
        let mut visited = BTreeSet::new();
        disc.walk(
            le32(&pvd, 158) * SECTOR,
            le32(&pvd, 166),
            "",
            0,
            &mut visited,
        )?;
        Ok(disc)
    }

    fn bytes(&mut self, offset: u64, size: u64) -> Result<Vec<u8>, String> {
        if size > MAX_READ || offset.checked_add(size).is_none_or(|end| end > self.length) {
            return Err("ISO read is outside the file or exceeds the 64 MiB read limit".into());
        }
        let mut data = vec![0; size as usize];
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        self.file.read_exact(&mut data).map_err(|e| e.to_string())?;
        Ok(data)
    }

    fn walk(
        &mut self,
        offset: u64,
        size: u64,
        prefix: &str,
        depth: usize,
        visited: &mut BTreeSet<u64>,
    ) -> Result<(), String> {
        if depth > 16 || size > MAX_DIRECTORY || !visited.insert(offset) {
            return Err("Invalid or cyclic ISO directory".into());
        }
        let data = self.bytes(offset, size)?;
        let mut pos = 0;
        while pos < data.len() {
            let len = data[pos] as usize;
            if len == 0 {
                pos = (pos / 2048 + 1) * 2048;
                continue;
            }
            if len < 34 || pos + len > data.len() || pos % 2048 + len > 2048 {
                return Err("Truncated ISO directory record".into());
            }
            let r = &data[pos..pos + len];
            let name_len = r[32] as usize;
            if 33 + name_len > len {
                return Err("Invalid ISO name length".into());
            }
            let name = &r[33..33 + name_len];
            if name != [0] && name != [1] {
                if r[1] != 0 || r[25] & 0x80 != 0 || r[26] != 0 || r[27] != 0 {
                    return Err(
                        "Extended, multi-extent, or interleaved files are unsupported".into(),
                    );
                }
                let leaf = normalize(std::str::from_utf8(name).map_err(|_| "Non-ASCII ISO name")?)?;
                if leaf.contains('/') {
                    return Err("Invalid ISO directory entry name".into());
                }
                let path = format!("{prefix}{leaf}");
                let entry = Entry {
                    offset: le32(r, 2) * SECTOR,
                    size: le32(r, 10),
                };
                if entry
                    .offset
                    .checked_add(entry.size)
                    .is_none_or(|end| end > self.length)
                {
                    return Err(format!("ISO entry outside image: {path}"));
                }
                if r[25] & 2 != 0 {
                    self.walk(
                        entry.offset,
                        entry.size,
                        &format!("{path}/"),
                        depth + 1,
                        visited,
                    )?;
                } else if self.entries.insert(path, entry).is_some() {
                    return Err("Duplicate ISO path".into());
                }
            }
            pos += len;
        }
        Ok(())
    }

    pub fn files(&self) -> &BTreeMap<String, Entry> {
        &self.entries
    }

    pub fn read(&mut self, name: &str) -> Result<Vec<u8>, String> {
        let key = normalize(name)?;
        let e = self
            .entries
            .get(&key)
            .ok_or_else(|| format!("Missing disc file: {key}"))?
            .clone();
        self.bytes(e.offset, e.size)
    }

    /// Stream archive blocks without loading the roughly 1.9 GB GAMEFILE.DAT into RAM.
    pub fn read_range(&mut self, name: &str, offset: u64, size: u64) -> Result<Vec<u8>, String> {
        let key = normalize(name)?;
        let e = self
            .entries
            .get(&key)
            .ok_or_else(|| format!("Missing disc file: {key}"))?
            .clone();
        if offset.checked_add(size).is_none_or(|end| end > e.size) {
            return Err(format!("Read extends past {key}"));
        }
        self.bytes(e.offset + offset, size)
    }

    pub fn verify_neopets(&mut self) -> Result<String, String> {
        let cnf = self.read("SYSTEM.CNF")?;
        let text = std::str::from_utf8(&cnf).map_err(|_| "Invalid SYSTEM.CNF text")?;
        let value = |key: &str| {
            text.lines()
                .filter_map(|line| line.split_once('='))
                .find(|(k, _)| k.trim().eq_ignore_ascii_case(key))
                .map(|(_, v)| v.trim())
        };
        let boot = value("BOOT2").ok_or("Missing BOOT2")?;
        let name = boot
            .rsplit(['\\', '/', ':'])
            .next()
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        if !name.eq_ignore_ascii_case("SCUS_973.67") || value("VER") != Some("1.00") {
            return Err(
                "Expected Neopets: The Darkest Faerie, USA SCUS-97367, version 1.00".into(),
            );
        }
        if !self.entries.contains_key("GAMEFILE.DAT") {
            return Err("Missing GAMEFILE.DAT".into());
        }
        let head = self.read_range("SCUS_973.67", 0, 20)?;
        if &head[..7] != b"\x7fELF\x01\x01\x01" || head[18..20] != [8, 0] {
            return Err("Boot program is not a little-endian MIPS ELF32".into());
        }
        Ok("Neopets: The Darkest Faerie • USA • SCUS-97367 • 1.00".into())
    }
}
