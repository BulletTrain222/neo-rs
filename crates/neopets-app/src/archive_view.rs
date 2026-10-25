//! Text summary of one archive resource for the browser (SPEC-0001, SPEC-0002). Shows layout
//! facts only; contents are not decoded until the M3+ decoders land.
use neopets_data::Disc;
use neopets_data::archive::ArchiveIndex;
use neopets_data::resource::{self, ChainEnd, Family};

/// Chunk types as a compact chain, collapsing runs: "2, 13 ×33, 4".
pub fn chain_summary(kinds: &[u16]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < kinds.len() {
        let run = kinds[i..].iter().take_while(|&&k| k == kinds[i]).count();
        parts.push(if run > 1 {
            format!("{} x{run}", kinds[i])
        } else {
            kinds[i].to_string()
        });
        i += run;
    }
    parts.join(", ")
}

pub fn describe(disc: &mut Disc, index: &ArchiveIndex, n: u32) -> Result<String, String> {
    let e = index.get(n)?;
    let mut text = format!(
        "Resource #{n} of {}: sectors {}+{} ({} KiB)",
        index.len(),
        e.start_sector,
        e.sector_count,
        e.readable_len() / 1024
    );
    if e.is_empty() {
        text.push_str("\nEmpty placeholder: no data on the disc.");
        return Ok(text);
    }
    let bytes = index.read(disc, n)?;
    let family = resource::classify(&bytes);
    text.push_str(&format!("\nFamily: {}", family.name()));
    if let Family::ChunkFile(_) = family {
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        let kinds: Vec<u16> = chain.chunks.iter().map(|c| c.kind).collect();
        text.push_str(&format!(
            "\nChunks ({}): {}",
            kinds.len(),
            chain_summary(&kinds)
        ));
        if let ChainEnd::Malformed(why) = chain.end {
            text.push_str(&format!("\nChain stops early: {why}"));
        }
    }
    text.push_str("\nNot decoded yet.");
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_runs_of_chunk_types() {
        assert_eq!(chain_summary(&[]), "");
        assert_eq!(chain_summary(&[2, 13, 13, 13, 4]), "2, 13 x3, 4");
        assert_eq!(chain_summary(&[4, 4, 18, 4]), "4 x2, 18, 4");
    }
}
