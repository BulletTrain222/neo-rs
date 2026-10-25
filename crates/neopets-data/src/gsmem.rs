//! PS2 Graphics Synthesizer local memory, addressed the way the hardware stores PSMCT32,
//! PSMT8 and PSMT4 pixels (public GS documentation: pages, blocks and columns). The game uploads
//! texture data as 32-bit pixels and samples the same memory as indexed pixels (SPEC-0003), so
//! decoding means writing with one addressing mode and reading with another.

pub const SIZE: usize = 4 << 20;
const BLOCK_BYTES: usize = 256;
const BLOCKS: usize = SIZE / BLOCK_BYTES;

/// Block numbers within a page, in block-grid order (8 wide, 4 tall) for PSMCT32 and PSMT8.
const BLOCKS_32: [[u8; 8]; 4] = [
    [0, 1, 4, 5, 16, 17, 20, 21],
    [2, 3, 6, 7, 18, 19, 22, 23],
    [8, 9, 12, 13, 24, 25, 28, 29],
    [10, 11, 14, 15, 26, 27, 30, 31],
];

/// Block numbers within a page for PSMT4 (4 wide, 8 tall).
const BLOCKS_4: [[u8; 4]; 8] = [
    [0, 2, 8, 10],
    [1, 3, 9, 11],
    [4, 6, 12, 14],
    [5, 7, 13, 15],
    [16, 18, 24, 26],
    [17, 19, 25, 27],
    [20, 22, 28, 30],
    [21, 23, 29, 31],
];

/// Word order inside one PSMCT32 column (8 × 2 pixels).
const COLUMN_32: [[u8; 8]; 2] = [[0, 1, 4, 5, 8, 9, 12, 13], [2, 3, 6, 7, 10, 11, 14, 15]];

fn block_base(bp: u32, page: u32, block: u8) -> usize {
    (bp as usize + page as usize * 32 + block as usize) % BLOCKS * BLOCK_BYTES
}

/// Byte offset of the word for pixel (x, y) inside an 8 × 8 PSMCT32 block.
fn ct32_in_block(x: u32, y: u32) -> usize {
    ((y / 2) * 16 + COLUMN_32[(y % 2) as usize][x as usize] as u32) as usize * 4
}

/// The indexed formats pack several pixels into the words of one PSMCT32 block. For a pixel at
/// (x, y) inside a PSMT8 (16 × 16) or PSMT4 (32 × 16) block, this gives the PSMCT32 pixel of the
/// same block whose word holds it, and the byte (PSMT8) or nibble (PSMT4) within that word.
fn indexed_in_block(x: u32, y: u32, wide: bool) -> (usize, u32) {
    let swap = ((y + 2) >> 2) & 1;
    let cx = (x & 7) ^ (swap << 2);
    let cy = ((y & !3) >> 1) | (y & 1);
    let sub = ((y >> 1) & 1) | ((x >> 2) & 2) | if wide { (x >> 2) & 4 } else { 0 };
    (ct32_in_block(cx, cy), sub)
}

/// Buffer width in 64-pixel units, as pages per row for the 128-pixel-wide indexed pages.
fn indexed_pages_per_row(bw: u32) -> u32 {
    (bw / 2).max(1)
}

pub fn ct32_offset(bp: u32, bw: u32, x: u32, y: u32) -> usize {
    let page = x / 64 + (y / 32) * bw.max(1);
    let block = BLOCKS_32[((y % 32) / 8) as usize][((x % 64) / 8) as usize];
    block_base(bp, page, block) + ct32_in_block(x % 8, y % 8)
}

pub fn t8_offset(bp: u32, bw: u32, x: u32, y: u32) -> usize {
    let page = x / 128 + (y / 64) * indexed_pages_per_row(bw);
    let block = BLOCKS_32[((y % 64) / 16) as usize][((x % 128) / 16) as usize];
    let (word, byte) = indexed_in_block(x % 16, y % 16, false);
    block_base(bp, page, block) + word + byte as usize
}

/// Byte offset and whether the pixel is the high nibble.
pub fn t4_offset(bp: u32, bw: u32, x: u32, y: u32) -> (usize, bool) {
    let page = x / 128 + (y / 128) * indexed_pages_per_row(bw);
    let block = BLOCKS_4[((y % 128) / 16) as usize][((x % 128) / 32) as usize];
    let (word, nibble) = indexed_in_block(x % 32, y % 16, true);
    (
        block_base(bp, page, block) + word + (nibble / 2) as usize,
        nibble % 2 == 1,
    )
}

/// One 4 MiB local-memory image, zero at creation.
pub struct GsMemory {
    bytes: Vec<u8>,
}

impl Default for GsMemory {
    fn default() -> Self {
        Self {
            bytes: vec![0; SIZE],
        }
    }
}

impl GsMemory {
    /// Host-to-local transfer of a `width × height` PSMCT32 rectangle at (0, 0) of buffer
    /// (`bp`, `bw`). `pixels` is row-major, 4 bytes per pixel.
    pub fn upload_ct32(&mut self, bp: u32, bw: u32, width: u32, height: u32, pixels: &[u8]) {
        for y in 0..height {
            for x in 0..width {
                let src = ((y * width + x) * 4) as usize;
                let dst = ct32_offset(bp, bw, x, y);
                self.bytes[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
            }
        }
    }

    pub fn ct32(&self, bp: u32, bw: u32, x: u32, y: u32) -> [u8; 4] {
        let at = ct32_offset(bp, bw, x, y);
        self.bytes[at..at + 4].try_into().unwrap()
    }

    pub fn t8(&self, bp: u32, bw: u32, x: u32, y: u32) -> u8 {
        self.bytes[t8_offset(bp, bw, x, y)]
    }

    pub fn t4(&self, bp: u32, bw: u32, x: u32, y: u32) -> u8 {
        let (at, high) = t4_offset(bp, bw, x, y);
        if high {
            self.bytes[at] >> 4
        } else {
            self.bytes[at] & 0x0f
        }
    }

    pub fn set_ct32(&mut self, bp: u32, bw: u32, x: u32, y: u32, c: [u8; 4]) {
        let at = ct32_offset(bp, bw, x, y);
        self.bytes[at..at + 4].copy_from_slice(&c);
    }

    /// Store an index the way the GS would for a PSMT8 write (used by tests and tooling).
    pub fn set_t8(&mut self, bp: u32, bw: u32, x: u32, y: u32, v: u8) {
        self.bytes[t8_offset(bp, bw, x, y)] = v;
    }

    pub fn set_t4(&mut self, bp: u32, bw: u32, x: u32, y: u32, v: u8) {
        let (at, high) = t4_offset(bp, bw, x, y);
        let b = &mut self.bytes[at];
        *b = if high {
            (*b & 0x0f) | (v << 4)
        } else {
            (*b & 0xf0) | (v & 0x0f)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ct32_page_is_a_bijection_onto_8_kib() {
        let offsets: HashSet<usize> = (0..32)
            .flat_map(|y| (0..64).map(move |x| ct32_offset(0, 1, x, y)))
            .collect();
        assert_eq!(offsets.len(), 64 * 32);
        assert!(offsets.iter().all(|&o| o < 8192 && o % 4 == 0));
        // First block: the first column is two rows of eight words.
        assert_eq!(ct32_offset(0, 1, 0, 0), 0);
        assert_eq!(ct32_offset(0, 1, 1, 0), 4);
        assert_eq!(ct32_offset(0, 1, 0, 1), 8);
        assert_eq!(ct32_offset(0, 1, 2, 0), 16);
        assert_eq!(ct32_offset(0, 1, 0, 2), 64);
        // The second block in a row is block 1; the second block row starts with block 2.
        assert_eq!(ct32_offset(0, 1, 8, 0), 256);
        assert_eq!(ct32_offset(0, 1, 0, 8), 512);
    }

    #[test]
    fn indexed_pages_are_bijections_onto_8_kib() {
        let t8: HashSet<usize> = (0..64)
            .flat_map(|y| (0..128).map(move |x| t8_offset(0, 2, x, y)))
            .collect();
        assert_eq!(t8.len(), 128 * 64);
        assert!(t8.iter().all(|&o| o < 8192));
        let t4: HashSet<(usize, bool)> = (0..128)
            .flat_map(|y| (0..128).map(move |x| t4_offset(0, 2, x, y)))
            .collect();
        assert_eq!(t4.len(), 128 * 128);
        assert!(t4.iter().all(|&(o, _)| o < 8192));
    }

    #[test]
    fn pages_advance_by_buffer_width_and_base_pointer() {
        // A 256-wide PSMCT32 buffer has four pages per row.
        assert_eq!(ct32_offset(0, 4, 64, 0), 8192);
        assert_eq!(ct32_offset(0, 4, 0, 32), 4 * 8192);
        assert_eq!(ct32_offset(0x2A00, 4, 0, 0), 0x2A00 * 256);
        // A 512-wide PSMT8 buffer (TBW 8) also has four 128-pixel pages per row.
        assert_eq!(t8_offset(0, 8, 128, 0) / 8192, 1);
        assert_eq!(t8_offset(0, 8, 0, 64) / 8192, 4);
        // Addresses wrap at the end of local memory.
        assert_eq!(ct32_offset(BLOCKS as u32, 1, 0, 0), 0);
    }

    #[test]
    fn indexed_writes_read_back() {
        let mut m = GsMemory::default();
        for y in 0..20 {
            for x in 0..40 {
                m.set_t8(0x40, 2, x, y, (x * 7 + y) as u8);
                m.set_t4(0x80, 2, x, y, ((x + y) % 16) as u8);
            }
        }
        for y in 0..20 {
            for x in 0..40 {
                assert_eq!(m.t8(0x40, 2, x, y), (x * 7 + y) as u8);
                assert_eq!(m.t4(0x80, 2, x, y), ((x + y) % 16) as u8);
            }
        }
    }
}
