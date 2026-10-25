//! Minimal ELF32 little-endian program-header reader, from the public ELF format. Used only to
//! turn a virtual address named by a spec into a file offset inside the boot program.

const PT_LOAD: u32 = 1;
const MAX_SEGMENTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Segment {
    offset: u32,
    vaddr: u32,
    filesz: u32,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn loadable_segments(elf: &[u8]) -> Result<Vec<Segment>, String> {
    if elf.get(..6) != Some(b"\x7fELF\x01\x01") {
        return Err("Not a little-endian ELF32 file".into());
    }
    let bad = || "Truncated ELF program header table".to_string();
    let phoff = u32_at(elf, 0x1c).ok_or_else(bad)? as usize;
    let phentsize = u16_at(elf, 0x2a).ok_or_else(bad)? as usize;
    let phnum = u16_at(elf, 0x2c).ok_or_else(bad)? as usize;
    if phentsize < 32 || phnum > MAX_SEGMENTS {
        return Err("Unsupported ELF program header table".into());
    }
    let mut out = Vec::new();
    for i in 0..phnum {
        let at = phoff
            .checked_add(i * phentsize)
            .ok_or_else(bad)?;
        let ph = elf.get(at..at + 32).ok_or_else(bad)?;
        if u32_at(ph, 0) == Some(PT_LOAD) {
            out.push(Segment {
                offset: u32_at(ph, 4).ok_or_else(bad)?,
                vaddr: u32_at(ph, 8).ok_or_else(bad)?,
                filesz: u32_at(ph, 16).ok_or_else(bad)?,
            });
        }
    }
    Ok(out)
}

/// File offset of `len` bytes at virtual address `vaddr`, which must lie wholly inside the
/// file-backed part of one loadable segment.
pub fn file_offset(elf: &[u8], vaddr: u32, len: u32) -> Result<u64, String> {
    for s in loadable_segments(elf)? {
        let end = s.vaddr as u64 + s.filesz as u64;
        if vaddr >= s.vaddr && vaddr as u64 + len as u64 <= end {
            return Ok(s.offset as u64 + (vaddr - s.vaddr) as u64);
        }
    }
    Err(format!(
        "Address {vaddr:#010x} (+{len} bytes) is not in a loadable ELF segment"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic ELF: one header, one PT_LOAD mapping file 0x80.. to 0x00100000.., 0x100 bytes.
    fn elf(segments: &[(u32, u32, u32, u32)]) -> Vec<u8> {
        let mut b = vec![0u8; 0x34 + 32 * segments.len()];
        b[..6].copy_from_slice(b"\x7fELF\x01\x01");
        b[0x1c..0x20].copy_from_slice(&0x34u32.to_le_bytes());
        b[0x2a..0x2c].copy_from_slice(&32u16.to_le_bytes());
        b[0x2c..0x2e].copy_from_slice(&(segments.len() as u16).to_le_bytes());
        for (i, &(kind, offset, vaddr, filesz)) in segments.iter().enumerate() {
            let p = 0x34 + 32 * i;
            for (field, v) in [(0, kind), (4, offset), (8, vaddr), (16, filesz)] {
                b[p + field..p + field + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        b
    }

    #[test]
    fn maps_addresses_inside_a_load_segment() {
        let e = elf(&[(6, 0, 0, 0), (PT_LOAD, 0x80, 0x0010_0000, 0x100)]);
        assert_eq!(file_offset(&e, 0x0010_0000, 4), Ok(0x80));
        assert_eq!(file_offset(&e, 0x0010_00f8, 8), Ok(0x178));
    }

    #[test]
    fn rejects_ranges_outside_file_backed_data() {
        let e = elf(&[(PT_LOAD, 0x80, 0x0010_0000, 0x100)]);
        assert!(file_offset(&e, 0x0010_00f8, 9).is_err());
        assert!(file_offset(&e, 0x000f_fffc, 4).is_err());
        let not_load = elf(&[(6, 0x80, 0x0010_0000, 0x100)]);
        assert!(file_offset(&not_load, 0x0010_0000, 4).is_err());
    }

    #[test]
    fn rejects_malformed_headers() {
        assert!(file_offset(b"not an elf", 0, 0).is_err());
        let mut e = elf(&[(PT_LOAD, 0x80, 0x0010_0000, 0x100)]);
        e.truncate(0x40);
        assert!(file_offset(&e, 0x0010_0000, 4).is_err());
        let mut e = elf(&[(PT_LOAD, 0x80, 0x0010_0000, 0x100)]);
        e[0x2c..0x2e].copy_from_slice(&1000u16.to_le_bytes());
        assert!(file_offset(&e, 0x0010_0000, 4).is_err());
    }
}
