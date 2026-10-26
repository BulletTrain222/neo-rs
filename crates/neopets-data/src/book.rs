//! Book records (SPEC-0024 Book records and Book index): resource 576 holds 88 records, each a
//! chunk chain whose type-2000 chunk has the title and the body paragraphs. Record i lies at
//! directory entry i of the books' path record (start and length in 2,048-byte units).

use crate::resource;

pub const RESOURCE: u32 = 576;
const TEXT: u16 = 2000;

/// A book's title and its NUL-terminated body paragraphs (a newline inside one is a line end).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    pub style: u32,
    pub title: Vec<u8>,
    pub paragraphs: Vec<Vec<u8>>,
}

/// The type-2000 payload: title length (with its NUL) at +0x00, style at +0x04, the title at
/// +0x10, then the paragraphs to the end, split at NUL.
pub fn parse_text(p: &[u8]) -> Option<Book> {
    let w = |o: usize| p.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let len = w(0)? as usize;
    let title = p.get(0x10..0x10 + len)?;
    let title = title[..title.iter().position(|&b| b == 0).unwrap_or(title.len())].to_vec();
    let paragraphs = p[0x10 + len..].split(|&b| b == 0).filter(|s| !s.is_empty()).map(<[u8]>::to_vec).collect();
    Some(Book { style: w(4)?, title, paragraphs })
}

/// Book i from resource 576's bytes and the directory entry (start, length) in sectors.
pub fn read(resource_bytes: &[u8], entry: (u32, u32)) -> Option<Book> {
    let (start, len) = (entry.0 as usize * 2048, entry.1 as usize * 2048);
    let rec = resource_bytes.get(start..(start + len).min(resource_bytes.len()))?;
    let chain = resource::walk_chunks(rec, None).ok()?;
    let c = chain.chunks.iter().find(|c| c.kind == TEXT)?;
    parse_text(rec.get(c.payload.clone())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_payload() {
        let mut p = vec![0u8; 0x10];
        p[0] = 6;
        p[4] = 2;
        p.extend_from_slice(b"Title\0First\nline\0Second\0");
        let b = parse_text(&p).unwrap();
        assert_eq!((b.style, b.title.as_slice()), (2, &b"Title"[..]));
        assert_eq!(b.paragraphs, vec![b"First\nline".to_vec(), b"Second".to_vec()]);
        assert!(parse_text(&[0; 8]).is_none());
    }
}
