//! Resource families and the IDM chunk container (SPEC-0002). The game never probes bytes to
//! pick a loader, so classification here is for tooling and validation; loaders will treat
//! resource numbers as typed slots.
use std::ops::Range;

const CHUNK_HEADER: usize = 16;
/// Enough bytes to classify any family by its first bytes.
pub const CLASSIFY_BYTES: u64 = 0x24;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Family {
    Empty,
    /// "IDM" container; the byte is the first chunk's format byte (1, 4 for texture packs,
    /// 0xCD for the design-table resource).
    ChunkFile(u8),
    /// First u32 = 1 and "SBv2" at +0x20.
    SoundBank,
    /// First u32 = 3 and "SBlk" at +0x18.
    SoundBlock,
    /// "VAGp" mono ADPCM.
    Vag,
    /// " KPV" stereo ADPCM stream.
    StereoStream,
    /// No magic: clip banks, skeleton headers, tables, text. Identified by resource number in
    /// later specs, not by bytes.
    Headerless,
}

impl Family {
    pub fn name(self) -> String {
        match self {
            Family::Empty => "empty".into(),
            Family::ChunkFile(1) => "chunk-file".into(),
            Family::ChunkFile(4) => "texture-pack".into(),
            Family::ChunkFile(b) => format!("chunk-file-{b:#04x}"),
            Family::SoundBank => "sound-bank".into(),
            Family::SoundBlock => "sound-block".into(),
            Family::Vag => "vag".into(),
            Family::StereoStream => "stereo-stream".into(),
            Family::Headerless => "headerless".into(),
        }
    }
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// Classify from the first bytes of a resource (at least [`CLASSIFY_BYTES`] for every family
/// to be recognizable). An empty slice is an empty resource.
pub fn classify(bytes: &[u8]) -> Family {
    let tag = |at: usize, t: &[u8]| bytes.get(at..at + t.len()) == Some(t);
    if bytes.is_empty() {
        Family::Empty
    } else if tag(0, b"IDM") && bytes.len() > 3 {
        Family::ChunkFile(bytes[3])
    } else if tag(0, b"VAGp") {
        Family::Vag
    } else if tag(0, b" KPV") {
        Family::StereoStream
    } else if u32_at(bytes, 0) == Some(1) && tag(0x20, b"SBv2") {
        Family::SoundBank
    } else if u32_at(bytes, 0) == Some(3) && tag(0x18, b"SBlk") {
        Family::SoundBlock
    } else {
        Family::Headerless
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    /// Offset of the 16-byte header within the walked buffer.
    pub offset: usize,
    pub tag: [u8; 4],
    pub kind: u16,
    /// Extra header bytes between the 16-byte header and the payload.
    pub extra: u16,
    /// Distance to the next sibling, header included.
    pub size: u32,
    /// u16 at header byte 14. Only meaningful for children of a type-8 container, where it is
    /// the role id; elsewhere it is build-tool fill.
    pub role_id: u16,
    pub payload: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainEnd {
    /// A header with chunk type 0.
    Terminator,
    /// No room for another header before the end of the buffer.
    BufferEnd,
    /// Stopped after the caller's chunk limit.
    Limit,
    /// An implausible header; the chunks before it are still valid.
    Malformed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkChain {
    pub chunks: Vec<Chunk>,
    pub end: ChainEnd,
}

/// One-chunk walk limits from SPEC-0002: texture packs and the held-item display models
/// 544–575 are followed by bytes that are not chunks.
pub fn chunk_limit(resource: u32, family: Family) -> Option<usize> {
    if family == Family::ChunkFile(4) || (544..=575).contains(&resource) {
        Some(1)
    } else {
        None
    }
}

/// Walk a chain of sibling chunks starting at offset 0. Only the first chunk's tag is checked,
/// and only its first three bytes ("IDM"), as the original does.
pub fn walk_chunks(bytes: &[u8], limit: Option<usize>) -> Result<ChunkChain, String> {
    if bytes.get(..3) != Some(b"IDM") {
        return Err("Not an IDM chunk file".into());
    }
    let mut chunks = Vec::new();
    let mut pos = 0usize;
    let end = loop {
        if limit.is_some_and(|l| chunks.len() >= l) {
            break ChainEnd::Limit;
        }
        if pos + CHUNK_HEADER > bytes.len() {
            break ChainEnd::BufferEnd;
        }
        let h = &bytes[pos..pos + CHUNK_HEADER];
        let kind = u16_at(h, 4);
        if kind == 0 {
            break ChainEnd::Terminator;
        }
        let extra = i16::from_le_bytes([h[6], h[7]]);
        let size = u32_at(h, 8).unwrap_or(0);
        let fail = |why: &str| ChainEnd::Malformed(format!("chunk at {pos:#x}: {why}"));
        if extra < 0 || extra % 4 != 0 {
            break fail("extra header length is negative or not a multiple of 4");
        }
        if size == 0 || size % 4 != 0 {
            break fail("size is zero or not a multiple of 4");
        }
        let next = pos as u64 + size as u64;
        if next > bytes.len() as u64 {
            break fail("size runs past the end of the resource");
        }
        let payload_start = pos + CHUNK_HEADER + extra as usize;
        if payload_start as u64 > next {
            break fail("extra header runs past the chunk");
        }
        chunks.push(Chunk {
            offset: pos,
            tag: h[0..4].try_into().unwrap(),
            kind,
            extra: extra as u16,
            size,
            role_id: u16_at(h, 14),
            payload: payload_start..next as usize,
        });
        pos = next as usize;
    };
    Ok(ChunkChain { chunks, end })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(tag: &[u8; 4], kind: u16, extra: i16, size: u32) -> Vec<u8> {
        let mut h = tag.to_vec();
        h.extend(kind.to_le_bytes());
        h.extend(extra.to_le_bytes());
        h.extend(size.to_le_bytes());
        h.extend([0; 4]);
        h
    }

    /// SPEC-0002 synthetic vector: a type-18 chunk, a type-4 chunk with E = 8, a terminator.
    fn spec_vector() -> Vec<u8> {
        let mut b = header(b"IDM\x01", 18, 0, 0x20);
        b.extend([0xAA; 16]);
        b.extend(header(b"IDM\x01", 4, 8, 0x30));
        b.extend([0xBB; 0x20]);
        b.extend(header(b"\0\0\0\0", 0, 0, 0));
        b
    }

    #[test]
    fn walks_spec_test_vector() {
        let chain = walk_chunks(&spec_vector(), None).unwrap();
        assert_eq!(chain.end, ChainEnd::Terminator);
        let [a, b] = &chain.chunks[..] else {
            panic!("expected two chunks")
        };
        assert_eq!((a.kind, a.extra, a.size, a.payload.clone()), (18, 0, 0x20, 0x10..0x20));
        assert_eq!((b.offset, b.kind, b.extra, b.size), (0x20, 4, 8, 0x30));
        assert_eq!((b.payload.start, b.payload.len()), (0x38, 0x18));
    }

    #[test]
    fn checks_only_three_tag_letters_and_honours_limits() {
        let mut v = spec_vector();
        v[3] = 7;
        assert_eq!(classify(&v), Family::ChunkFile(7));
        assert_eq!(walk_chunks(&v, None).unwrap().chunks.len(), 2);
        let limited = walk_chunks(&v, Some(1)).unwrap();
        assert_eq!((limited.chunks.len(), limited.end), (1, ChainEnd::Limit));
        assert!(walk_chunks(b"IDX\x01", None).is_err());
        assert!(walk_chunks(b"ID", None).is_err());
    }

    #[test]
    fn stops_at_buffer_end_and_on_malformed_headers() {
        let v = spec_vector();
        assert_eq!(walk_chunks(&v[..0x50], None).unwrap().end, ChainEnd::BufferEnd);
        let malformed = |patch: &dyn Fn(&mut Vec<u8>)| {
            let mut v = spec_vector();
            patch(&mut v);
            let chain = walk_chunks(&v, None).unwrap();
            assert_eq!(chain.chunks.len(), 1, "first chunk stays valid");
            matches!(chain.end, ChainEnd::Malformed(_))
        };
        assert!(malformed(&|v| v[0x28..0x2c].copy_from_slice(&0u32.to_le_bytes())));
        assert!(malformed(&|v| v[0x28..0x2c].copy_from_slice(&0x32u32.to_le_bytes())));
        assert!(malformed(&|v| v[0x28..0x2c].copy_from_slice(&0x1000u32.to_le_bytes())));
        assert!(malformed(&|v| v[0x26..0x28].copy_from_slice(&(-4i16).to_le_bytes())));
        assert!(malformed(&|v| v[0x26..0x28].copy_from_slice(&6i16.to_le_bytes())));
        assert!(malformed(&|v| v[0x26..0x28].copy_from_slice(&0x40i16.to_le_bytes())));
    }

    #[test]
    fn classifies_families_by_first_bytes() {
        let at = |len: usize, at: usize, bytes: &[u8]| {
            let mut v = vec![0u8; len];
            v[at..at + bytes.len()].copy_from_slice(bytes);
            v
        };
        assert_eq!(classify(&[]), Family::Empty);
        assert_eq!(classify(b"IDM\x04"), Family::ChunkFile(4));
        assert_eq!(classify(b"IDM"), Family::Headerless, "no format byte");
        assert_eq!(classify(b"VAGp\0\0\0\x20"), Family::Vag);
        assert_eq!(classify(b" KPV"), Family::StereoStream);
        let mut bank = at(0x24, 0x20, b"SBv2");
        bank[0] = 1;
        assert_eq!(classify(&bank), Family::SoundBank);
        bank[0] = 2;
        assert_eq!(classify(&bank), Family::Headerless);
        let mut block = at(0x1c, 0x18, b"SBlk");
        block[0] = 3;
        assert_eq!(classify(&block), Family::SoundBlock);
        assert_eq!(classify(&block[..0x1a]), Family::Headerless, "tag cut off");
        assert_eq!(classify(&[0xd0, 0, 0, 0]), Family::Headerless);
    }

    #[test]
    fn limits_follow_spec_rules() {
        assert_eq!(chunk_limit(600, Family::ChunkFile(4)), Some(1));
        assert_eq!(chunk_limit(544, Family::ChunkFile(1)), Some(1));
        assert_eq!(chunk_limit(575, Family::ChunkFile(1)), Some(1));
        assert_eq!(chunk_limit(7, Family::ChunkFile(1)), None);
        assert_eq!(chunk_limit(576, Family::ChunkFile(1)), None);
    }
}
