//! Book records (SPEC-0024 Book records and Book index): resource 576 holds 88 records, each a
//! chunk chain whose type-2000 chunk has the title and the body paragraphs. Record i lies at
//! directory entry i of the books' path record (start and length in 2,048-byte units).

use crate::resource;

pub const RESOURCE: u32 = 576;
const TEXT: u16 = 2000;
const LAYOUT: u16 = 2001;

/// A book (SPEC-0024 Book records): its style index, title and body string from the text chunk
/// (type 2000); its glyph font from the layout chunk (type 2001, front-end font 2's descriptor
/// with the book's own glyph records); and its paper picture (128 x 128) and glyph sheet
/// (256 x 256) from the record's second texture chunk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    pub style: u32,
    pub title: Vec<u8>,
    /// The body: one string; newline bytes end lines.
    pub body: Vec<u8>,
    pub font: Option<crate::frontend::Font>,
    pub paper: Option<crate::texture::Image>,
    pub sheet: Option<crate::texture::Image>,
    /// The record's bytes, for its page sheet and pictures (`models`).
    pub record: Vec<u8>,
}

/// The type-2000 payload: title length (with its NUL) at +0x00, style at +0x04, the title at
/// +0x10, then the body string (up to its zero byte).
pub fn parse_text(p: &[u8]) -> Option<Book> {
    let w = |o: usize| p.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let len = w(0)? as usize;
    let title = p.get(0x10..0x10 + len)?;
    let title = title[..title.iter().position(|&b| b == 0).unwrap_or(title.len())].to_vec();
    let rest = &p[0x10 + len..];
    let body = rest[..rest.iter().position(|&b| b == 0).unwrap_or(rest.len())].to_vec();
    Some(Book { style: w(4)?, title, body, ..Default::default() })
}

/// Book i from resource 576's bytes and the directory entry (start, length) in sectors.
pub fn read(resource_bytes: &[u8], entry: (u32, u32)) -> Option<Book> {
    let (start, len) = (entry.0 as usize * 2048, entry.1 as usize * 2048);
    let rec = resource_bytes.get(start..(start + len).min(resource_bytes.len()))?;
    let chain = resource::walk_chunks(rec, None).ok()?;
    let c = chain.chunks.iter().find(|c| c.kind == TEXT)?;
    let mut book = parse_text(rec.get(c.payload.clone())?)?;
    book.record = rec.to_vec();
    if let Some(l) = chain.chunks.iter().find(|c| c.kind == LAYOUT) {
        book.font = rec.get(l.payload.clone()).and_then(|p| crate::frontend::parse_font(p, 0));
    }
    // The second type-4 chunk: the paper (128 x 128, 8-bit) and the glyph sheet (256 x 256).
    if let Some(t) = chain.chunks.iter().filter(|c| c.kind == 4).nth(1) {
        if let Some(tex) = rec.get(t.payload.clone()).and_then(|p| crate::texture::TextureChunk::parse(p).ok()) {
            let mem = tex.gs_memory();
            for k in tex.textures().collect::<Vec<_>>() {
                let Some(img) = tex.first_palette_for(k).and_then(|pal| tex.decode(&mem, k, pal).ok()) else { continue };
                match (img.width, img.height) {
                    (128, 128) if book.paper.is_none() => book.paper = Some(img),
                    (256, 256) if book.sheet.is_none() => book.sheet = Some(img),
                    _ => {}
                }
            }
        }
    }
    Some(book)
}

/// The record's page sheet (model child 100) and decoration mesh (child 200, its switch parts by the style mask;
/// SPEC-0024 Page data of a record; SPEC-0035 section 6.1 rule 12).
pub fn models(record: &[u8], mask: u32) -> Option<(crate::model::ModelData, crate::model::ModelData)> {
    let chain = resource::walk_chunks(record, None).ok()?;
    let child = |role: u16, m: Option<u32>| crate::model::build_masked(record, &chain.chunks, None, Some(&[role]), m).ok();
    Some((child(100, None)?, child(200, Some(mask))?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_payload() {
        let mut p = vec![0u8; 0x10];
        p[0] = 6;
        p[4] = 2;
        p.extend_from_slice(b"Title\0First\nline\n\nAfter\0\0");
        let b = parse_text(&p).unwrap();
        assert_eq!((b.style, b.title.as_slice()), (2, &b"Title"[..]));
        assert_eq!(b.body, b"First\nline\n\nAfter".to_vec(), "one body string up to its zero byte");
        assert!(parse_text(&[0; 8]).is_none());
    }
}
