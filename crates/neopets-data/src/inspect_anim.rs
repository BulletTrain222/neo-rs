//! Inspector check for SPEC-0005: decode every animation clip on the disc (chunk sets, nested
//! sets and the headerless banks) and check each frame's rotations are unit quaternions.
use neopets_data::archive::ArchiveIndex;
use neopets_data::resource::{self, Family};
use neopets_data::{Disc, anim};
use std::collections::BTreeMap;

/// The headerless clip banks (SPEC-0005, Headerless resources).
pub const BANKS: [u32; 7] = [493, 495, 497, 499, 501, 503, 526];

#[derive(Default)]
struct Census {
    sets: BTreeMap<String, u32>,
    entries: u64,
    compressed: u64,
    by_mode: BTreeMap<u8, u64>,
    bone_frames: u64,
    worst_norm_error: f32,
    errors: Vec<String>,
}

impl Census {
    fn set(&mut self, label: String, b: &[u8]) {
        *self.sets.entry(label.split(' ').next().unwrap_or("").to_string()).or_default() += 1;
        let list = match anim::entries(b) {
            Ok(l) => l,
            Err(e) => return self.errors.push(format!("{label}: {e}")),
        };
        for at in list {
            self.entries += 1;
            match anim::decode_entry(b, at) {
                Ok(c) => {
                    self.compressed += c.compressed as u64;
                    *self.by_mode.entry(c.mode).or_default() += 1;
                    for q in &c.rotations {
                        let n = (q.iter().map(|v| v * v).sum::<f32>()).sqrt();
                        self.worst_norm_error = self.worst_norm_error.max((n - 1.0).abs());
                    }
                    self.bone_frames += c.rotations.len() as u64;
                }
                Err(e) => self.errors.push(format!("{label} entry {at:#x}: {e}")),
            }
        }
    }
}

pub fn run(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    let mut c = Census::default();
    for n in 0..index.len() as u32 {
        let bytes = index.read(disc, n)?;
        if BANKS.contains(&n) {
            c.set(format!("bank {n}"), &bytes);
            continue;
        }
        let family = resource::classify(&bytes);
        if !matches!(family, Family::ChunkFile(_)) {
            continue;
        }
        let chain = resource::walk_chunks(&bytes, resource::chunk_limit(n, family))?;
        for ch in &chain.chunks {
            let p = &bytes[ch.payload.clone()];
            match ch.kind {
                25..=27 => c.set(format!("top-{} r{n}", ch.kind), p),
                // Type-32 containers hold a nested chunk file whose children include clip sets.
                32 => {
                    if let Ok(nested) = resource::walk_chunks(p, None) {
                        for k in nested.chunks.iter().filter(|k| (25..=27).contains(&k.kind)) {
                            c.set(format!("nested-{} r{n}", k.kind), &p[k.payload.clone()]);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    println!("Clip sets: {:?}", c.sets);
    disc_vector(disc, index)?;
    println!(
        "Clip entries: {} ({} compressed); by evaluation mode: {:?}",
        c.entries, c.compressed, c.by_mode
    );
    println!(
        "Bone-frames: {}; largest quaternion norm error: {:.5}",
        c.bone_frames, c.worst_norm_error
    );
    println!("Decode errors: {}", c.errors.len());
    for e in c.errors.iter().take(15) {
        println!("  {e}");
    }
    Ok(())
}

/// SPEC-0005 disc vector: resource 541, type-25 entry 6, bone 0 sampled at cursor 19.5 (keys 19
/// and 0, fraction 0.5). The spec's values are sample / 1024; the runtime scale is 0.045 %
/// larger, so the comparison allows 0.001.
fn disc_vector(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    let bytes = index.read(disc, 541)?;
    let chain = resource::walk_chunks(&bytes, None)?;
    let set = chain.chunks.iter().find(|c| c.kind == 25).ok_or("Resource 541 has no type-25 chunk")?;
    let p = &bytes[set.payload.clone()];
    let at = *anim::entries(p)?.get(6).ok_or("Type-25 set has no entry 6")?;
    let clip = anim::decode_entry(p, at)?;
    let (q, t) = anim::blend_key(clip.rotation(19, 0), clip.rotation(0, 0), clip.translation(19, 0), clip.translation(0, 0), 0.5);
    let want_q = [0.0630, 0.0630, -0.0752, 0.9927];
    let want_t = [-0.0332, 0.6372, 0.0];
    let ok = (clip.bones.len(), clip.frames, clip.looping) == (21, 20, true)
        && q.iter().zip(want_q).all(|(a, b)| (a - b).abs() < 1e-3)
        && t.iter().zip(want_t).all(|(a, b)| (a - b).abs() < 1e-3);
    println!(
        "SPEC-0005 disc vector (541, entry 6, cursor 19.5): rotation {:.4?}, translation {:.4?}: {}",
        q,
        t,
        if ok { "matches" } else { "MISMATCH" }
    );
    Ok(())
}
