//! GAMEFILE.DAT resource index (SPEC-0001). The archive has no header of its own; its index
//! is a table inside the boot program, trusted only after the boot program's fingerprint
//! matches the build the spec describes.
use crate::{Disc, elf, fingerprint};

pub const BOOT_PROGRAM: &str = "SCUS_973.67";
pub const ARCHIVE: &str = "GAMEFILE.DAT";
pub const SECTOR: u64 = 2048;

/// SPEC-0001: SHA-256 of SCUS_973.67 for SCUS-97367 v1.00.
const BOOT_SHA256: &str = "a8cc770cf6157f7a67672eea71ed1b631bf780e7d684802956ccab24b71d8f25";
/// SPEC-0001: virtual address of the index table in the boot program, for this build only.
const INDEX_VADDR: u32 = 0x002D_7618;
/// SPEC-0001: the entry count is a built-in constant of this build, not stored on the disc.
const ENTRY_COUNT: usize = 1434;
/// SPEC-0001, invariant 4: placeholder entries on this build.
const EMPTY_ENTRIES: usize = 76;
const ENTRY_SIZE: usize = 8;

/// The boot program's bytes, after checking its SHA-256 against SPEC-0001. Every fixed address
/// in the boot program is trusted only through this check.
pub fn verified_boot_program(disc: &mut Disc) -> Result<Vec<u8>, String> {
    let expected = fingerprint::parse_sha256(BOOT_SHA256)?;
    fingerprint::verify_sha256(disc, BOOT_PROGRAM, &expected).map_err(|e| {
        format!("{e}. Fixed locations are only known for SCUS-97367 v1.00.")
    })?;
    disc.read(BOOT_PROGRAM)
}

/// Read `count` little-endian u32 values at a virtual address of the verified boot program.
pub fn boot_u32_table(boot: &[u8], vaddr: u32, count: usize) -> Result<Vec<u32>, String> {
    let len = (count * 4) as u32;
    let at = elf::file_offset(boot, vaddr, len)? as usize;
    Ok(boot[at..at + len as usize]
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
        .collect())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceEntry {
    /// First sector, relative to the start of GAMEFILE.DAT.
    pub start_sector: u32,
    pub sector_count: u32,
}

impl ResourceEntry {
    /// Zero-count entries are placeholders; reading at their start would return the next
    /// resource's bytes.
    pub fn is_empty(&self) -> bool {
        self.sector_count == 0
    }

    pub fn byte_offset(&self) -> u64 {
        self.start_sector as u64 * SECTOR
    }

    /// Whole sectors. The true payload length, if any, comes from the resource's own header.
    pub fn readable_len(&self) -> u64 {
        self.sector_count as u64 * SECTOR
    }
}

#[derive(Clone, Debug)]
pub struct ArchiveIndex {
    entries: Vec<ResourceEntry>,
}

impl ArchiveIndex {
    /// Check the boot program's fingerprint, then read and validate the index from the disc.
    pub fn from_disc(disc: &mut Disc) -> Result<Self, String> {
        let boot = verified_boot_program(disc)?;
        let table_len = (ENTRY_COUNT * ENTRY_SIZE) as u32;
        let at = elf::file_offset(&boot, INDEX_VADDR, table_len)? as usize;
        let table = boot
            .get(at..at + table_len as usize)
            .ok_or("Index table runs past the boot program")?;
        let archive_size = disc.size(ARCHIVE)?;
        if archive_size % SECTOR != 0 {
            return Err(format!("{ARCHIVE} is not a whole number of sectors"));
        }
        let index = Self::parse(table, archive_size / SECTOR)?;
        let empty = index.entries.iter().filter(|e| e.is_empty()).count();
        if empty != EMPTY_ENTRIES {
            return Err(format!(
                "Index has {empty} empty entries; SPEC-0001 expects {EMPTY_ENTRIES} on this build"
            ));
        }
        Ok(index)
    }

    /// Parse back-to-back 8-byte entries and check SPEC-0001 invariants 1–3 against an
    /// archive of `archive_sectors` sectors.
    pub fn parse(table: &[u8], archive_sectors: u64) -> Result<Self, String> {
        if table.is_empty() || table.len() % ENTRY_SIZE != 0 {
            return Err("Index table must be a non-empty multiple of 8 bytes".into());
        }
        let entries: Vec<ResourceEntry> = table
            .chunks_exact(ENTRY_SIZE)
            .map(|e| ResourceEntry {
                start_sector: u32::from_le_bytes(e[0..4].try_into().unwrap()),
                sector_count: u32::from_le_bytes(e[4..8].try_into().unwrap()),
            })
            .collect();
        if entries[0].start_sector != 0 {
            return Err("Index does not start at sector 0".into());
        }
        let mut next = 0u64;
        for (n, e) in entries.iter().enumerate() {
            if e.start_sector as u64 != next {
                return Err(format!("Index entry {n} is not contiguous with the one before"));
            }
            next += e.sector_count as u64;
        }
        if next != archive_sectors {
            return Err(format!(
                "Index covers {next} sectors but the archive has {archive_sectors}"
            ));
        }
        Ok(Self { entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[ResourceEntry] {
        &self.entries
    }

    /// The original accepts one past the end in some checks; we reject anything out of range.
    pub fn get(&self, resource: u32) -> Result<ResourceEntry, String> {
        self.entries
            .get(resource as usize)
            .copied()
            .ok_or_else(|| format!("Resource {resource} is out of range 0..{}", self.len()))
    }

    /// All sectors of a resource; empty placeholders read as no bytes.
    pub fn read(&self, disc: &mut Disc, resource: u32) -> Result<Vec<u8>, String> {
        let e = self.get(resource)?;
        if e.is_empty() {
            return Ok(Vec::new());
        }
        disc.read_range(ARCHIVE, e.byte_offset(), e.readable_len())
    }

    /// Read only the first `len` bytes (clamped to the resource), for cheap classification.
    pub fn read_head(&self, disc: &mut Disc, resource: u32, len: u64) -> Result<Vec<u8>, String> {
        let e = self.get(resource)?;
        disc.read_range(ARCHIVE, e.byte_offset(), len.min(e.readable_len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(pairs: &[(u32, u32)]) -> Vec<u8> {
        pairs
            .iter()
            .flat_map(|&(s, c)| [s.to_le_bytes(), c.to_le_bytes()].concat())
            .collect()
    }

    #[test]
    fn parses_spec_test_vector() {
        // SPEC-0001 synthetic vector: three entries covering five sectors.
        let index = ArchiveIndex::parse(&table(&[(0, 3), (3, 0), (3, 2)]), 5).unwrap();
        assert_eq!(index.len(), 3);
        let r0 = index.get(0).unwrap();
        assert_eq!((r0.byte_offset(), r0.readable_len()), (0, 6144));
        assert!(index.get(1).unwrap().is_empty());
        let r2 = index.get(2).unwrap();
        assert_eq!((r2.byte_offset(), r2.readable_len()), (6144, 4096));
        assert!(index.get(3).is_err());
        assert!(index.get(u32::MAX).is_err());
    }

    #[test]
    fn rejects_broken_invariants() {
        assert!(ArchiveIndex::parse(&table(&[(1, 2)]), 3).is_err(), "must start at 0");
        assert!(ArchiveIndex::parse(&table(&[(0, 2), (3, 1)]), 4).is_err(), "gap");
        assert!(ArchiveIndex::parse(&table(&[(0, 2), (1, 1)]), 3).is_err(), "overlap");
        assert!(ArchiveIndex::parse(&table(&[(0, 2)]), 3).is_err(), "short of archive");
        assert!(ArchiveIndex::parse(&table(&[(0, u32::MAX), (u32::MAX, u32::MAX)]), 0).is_err());
        assert!(ArchiveIndex::parse(&[0; 12], 0).is_err(), "partial entry");
        assert!(ArchiveIndex::parse(&[], 0).is_err());
    }
}
