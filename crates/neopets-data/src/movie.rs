//! Movies: the boot program's movie name table (SPEC-0026 Movie name table) and the Bink file
//! header (SPEC-0008 Movie files). The frame data is not decoded: no spec describes the Bink
//! stream yet (specs/REQUESTS.md R-0112).
use crate::{Disc, elf};

/// SPEC-0026: 92 pointers to NUL-terminated names, in the boot program's data.
const NAME_TABLE_VADDR: u32 = 0x002E_B458;
pub const MOVIE_COUNT: usize = 92;
/// The longest name accepted (the disc names are short folder\stem.BIK strings).
const NAME_LIMIT: usize = 64;

/// The movie names by index, read from the verified boot program (`archive::verified_boot_program`).
/// Each is relative to the MOVIES folder, with a backslash, for example `ACT1\100A.BIK`.
pub fn names(boot: &[u8]) -> Result<Vec<String>, String> {
    crate::archive::boot_u32_table(boot, NAME_TABLE_VADDR, MOVIE_COUNT)?
        .into_iter()
        .map(|p| c_string(boot, p))
        .collect()
}

fn c_string(boot: &[u8], vaddr: u32) -> Result<String, String> {
    let at = elf::file_offset(boot, vaddr, 1)? as usize;
    let tail = &boot[at..boot.len().min(at + NAME_LIMIT)];
    let end = tail.iter().position(|&b| b == 0).ok_or_else(|| format!("Movie name at {vaddr:#x} is not terminated"))?;
    std::str::from_utf8(&tail[..end]).map(str::to_owned).map_err(|_| format!("Movie name at {vaddr:#x} is not ASCII"))
}

/// The disc path of a table name: under the MOVIES folder (SPEC-0026; the `;1` suffix dropped).
pub fn disc_path(name: &str) -> String {
    format!("MOVIES/{}", name.replace('\\', "/"))
}

/// The first 0x34 bytes of a Bink file (SPEC-0008 Movie files).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// The revision letter after `BIK` (f, g, h or i are accepted; every disc movie is i).
    pub revision: u8,
    pub frames: u32,
    pub largest_frame: u32,
    pub width: u32,
    pub height: u32,
    /// Frame time: `rate_num / rate_den` frames per second (10,000,000 / 333,333 for 30 fps).
    pub rate_num: u32,
    pub rate_den: u32,
    pub flags: u32,
    pub audio_tracks: u32,
    /// The first track's sample rate (low 16 bits at 0x30) and flags (high 16 bits).
    pub audio_rate: u16,
    pub audio_flags: u16,
}

pub const HEADER_SIZE: usize = 0x34;

impl Header {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        if b.len() < HEADER_SIZE {
            return Err("Movie header is short".into());
        }
        if &b[0..3] != b"BIK" || !matches!(b[3], b'f' | b'g' | b'h' | b'i') {
            return Err("Not a Bink revision f to i movie".into());
        }
        let u = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let audio = u(0x30);
        Ok(Self {
            revision: b[3],
            frames: u(0x08),
            largest_frame: u(0x0C),
            width: u(0x14),
            height: u(0x18),
            rate_num: u(0x1C),
            rate_den: u(0x20),
            flags: u(0x24),
            audio_tracks: u(0x28),
            audio_rate: audio as u16,
            audio_flags: (audio >> 16) as u16,
        })
    }

    pub fn fps(&self) -> f64 {
        if self.rate_den == 0 { 30.0 } else { self.rate_num as f64 / self.rate_den as f64 }
    }

    /// The play time, from the frame count and the frame rate.
    pub fn seconds(&self) -> f64 {
        self.frames as f64 / self.fps()
    }

    pub fn read(disc: &mut Disc, name: &str) -> Result<Self, String> {
        Self::parse(&disc.read_range(&disc_path(name), 0, HEADER_SIZE as u64)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movie_header_fields() {
        // Synthetic header with the SPEC-0008 disc values of a 30 fps movie.
        let mut b = vec![0u8; HEADER_SIZE];
        b[0..4].copy_from_slice(b"BIKi");
        let mut put = |o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        put(0x08, 990);
        put(0x0C, 40_996);
        put(0x10, 990);
        put(0x14, 512);
        put(0x18, 448);
        put(0x1C, 10_000_000);
        put(0x20, 333_333);
        put(0x28, 1);
        put(0x30, 0x1000 << 16 | 44_100);
        let h = Header::parse(&b).unwrap();
        assert_eq!((h.revision, h.frames, h.width, h.height, h.audio_rate, h.audio_flags), (b'i', 990, 512, 448, 44_100, 0x1000));
        // SPEC-0026: movie 0 is 990 frames, 33.0 s.
        assert!((h.seconds() - 33.0).abs() < 0.01);
        b[3] = b'j';
        assert!(Header::parse(&b).is_err());
        assert_eq!(disc_path("ACT1\\100A.BIK"), "MOVIES/ACT1/100A.BIK");
    }
}
