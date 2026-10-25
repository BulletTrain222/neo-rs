//! Dialogue lines (SPEC-0022 Dialogue line table and Text encoding; SPEC-0028 section 5): the
//! IDMX table of a world pack's type-1000 chunk, its 12-byte records (speaker offset, text
//! offset, colour), and the text with its control codes removed for plain display.
use crate::resource::Chunk;

pub const LINE_TABLE: u16 = 1000;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn c_string(b: &[u8], at: usize) -> Option<&[u8]> {
    let tail = b.get(at..)?;
    Some(&tail[..tail.iter().position(|&c| c == 0)?])
}

/// One dialogue line: the speaker name, the text with control codes removed (a page stop
/// becomes " / "), and the line colour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub speaker: String,
    pub text: String,
    pub colour: [u8; 3],
}

/// Plain text of a line: every control code (marker 0x01, length at +2) is skipped; the page
/// stop (type 4) becomes " / ". A code running past the end ends the text.
pub fn plain_text(raw: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == 0x01 {
            let (Some(&len), Some(&kind)) = (raw.get(i + 2), raw.get(i + 3)) else { break };
            if len == 0 || i + len as usize > raw.len() {
                break;
            }
            if kind == 4 {
                out.push_str(" / ");
            }
            i += len as usize;
        } else {
            out.push(raw[i] as char);
            i += 1;
        }
    }
    out.trim_end_matches(" / ").to_string()
}

/// A pack's dialogue line table, kept as the table bytes.
#[derive(Clone, Debug, Default)]
pub struct LineTable {
    table: Vec<u8>,
    count: usize,
}

impl LineTable {
    /// The table of the pack's first type-1000 chunk; empty when there is none or it is not
    /// an IDMX table.
    pub fn of(bytes: &[u8], top: &[Chunk]) -> Self {
        let Some(c) = top.iter().find(|c| c.kind == LINE_TABLE) else { return Self::default() };
        let t = &bytes[c.payload.clone()];
        if t.get(..4) != Some(b"IDMX") {
            return Self::default();
        }
        let count = t.get(0x0C..0x0E).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as usize);
        Self { table: t.to_vec(), count }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Line i, or None outside 0 to count − 1.
    pub fn line(&self, i: usize) -> Option<Line> {
        if i >= self.count {
            return None;
        }
        let r = 0x10 + 12 * i;
        let t = &self.table;
        let speaker = c_string(t, u32_at(t, r)? as usize)?;
        let text = c_string(t, u32_at(t, r + 4)? as usize)?;
        let c = t.get(r + 8..r + 11)?;
        Some(Line {
            speaker: plain_text(speaker),
            text: plain_text(text),
            colour: [c[0], c[1], c[2]],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_codes_are_removed_and_page_stops_marked() {
        // "Hi" + colour code (n 3, L 9, type 2, V 3, args R G B A) + "there" + page stop + "Bye".
        let mut raw = b"Hi ".to_vec();
        raw.extend([0x01, 3, 9, 2, 3, 128, 128, 128, 128]);
        raw.extend(b"there");
        raw.extend([0x01, 2, 5, 4, 6]);
        raw.extend(b"Bye");
        assert_eq!(plain_text(&raw), "Hi there / Bye");
        // A code running past the end ends the text.
        assert_eq!(plain_text(&[b'A', 0x01, 3, 20, 2]), "A");
    }

    #[test]
    fn table_records_resolve_to_strings() {
        let mut t = b"IDMX".to_vec();
        t.resize(0x10, 0);
        t[0x0C] = 1;
        t[0x0E] = 12;
        t.extend(28u32.to_le_bytes());
        t.extend(32u32.to_le_bytes());
        t.extend([10, 20, 30, 0]);
        t.extend(b"Tor\0Hi!\0");
        let table = LineTable { count: 1, table: t };
        assert_eq!(table.line(0), Some(Line { speaker: "Tor".into(), text: "Hi!".into(), colour: [10, 20, 30] }));
        assert_eq!(table.line(1), None);
    }
}
