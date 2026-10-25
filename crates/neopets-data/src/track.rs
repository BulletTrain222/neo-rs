//! Camera tracks: the type-28 chunk of a world pack (SPEC-0052 Camera track table and record;
//! SPEC-0028 section 1). The payload holds the count at +0x00 and record offsets from +0x10; a
//! record is a 0x40-byte head (quaternion, position, key count N at +0x20, key interval at
//! +0x24, per-key focal select at +0x2B, focal length at +0x2C, key offset at +0x34) followed by
//! N keys of 0x30 bytes (quaternion, position, focal at +0x24). The track id is the record index.

use crate::resource::Chunk;

/// The type-28 chunk.
pub const CAMERA_TRACKS: u16 = 28;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

fn v4(b: &[u8], at: usize) -> Option<[f32; 4]> {
    Some([f32_at(b, at)?, f32_at(b, at + 4)?, f32_at(b, at + 8)?, f32_at(b, at + 12)?])
}

/// One key: quaternion (x, y, z, w), position and focal length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub q: [f32; 4],
    pub p: [f32; 3],
    pub focal: f32,
}

/// One track record.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    /// The head pose (used when the track has no keys) and the head focal length.
    pub head: Key,
    pub dt: f32,
    pub per_key_focal: bool,
    pub keys: Vec<Key>,
}

/// The camera tracks of a pack (empty without a type-28 chunk). A record that does not fit
/// ends the list.
pub fn camera_tracks(bytes: &[u8], top: &[Chunk]) -> Vec<Track> {
    let Some(c) = top.iter().find(|c| c.kind == CAMERA_TRACKS) else { return Vec::new() };
    let Some(p) = bytes.get(c.payload.clone()) else { return Vec::new() };
    let n = u32_at(p, 0).unwrap_or(0).min(1 << 10) as usize;
    let table = u32_at(p, 4).unwrap_or(0x10) as usize;
    let key = |at: usize, focal_at: usize| -> Option<Key> {
        let q = v4(p, at)?;
        let pos = v4(p, at + 0x10)?;
        Some(Key { q, p: [pos[0], pos[1], pos[2]], focal: f32_at(p, at + focal_at)? })
    };
    (0..n)
        .map_while(|i| {
            let r = u32_at(p, table + 4 * i)? as usize;
            let head = key(r, 0x2C)?;
            let count = u32_at(p, r + 0x20)? as i32;
            let dt = f32_at(p, r + 0x24)?;
            let per_key_focal = *p.get(r + 0x2B)? != 0;
            let keys = if count > 0 {
                let base = r + u32_at(p, r + 0x34)? as usize;
                (0..count.min(4096) as usize).map(|k| key(base + 0x30 * k, 0x24)).collect::<Option<Vec<_>>>()?
            } else {
                Vec::new()
            };
            Some(Track { head, dt, per_key_focal, keys })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(b: &mut Vec<u8>, at: usize, v: &[u8]) {
        if b.len() < at + v.len() {
            b.resize(at + v.len(), 0);
        }
        b[at..at + v.len()].copy_from_slice(v);
    }

    #[test]
    fn reads_a_static_and_a_keyed_track() {
        let mut p = vec![0u8; 0x20];
        put(&mut p, 0, &2u32.to_le_bytes());
        put(&mut p, 4, &0x10u32.to_le_bytes());
        put(&mut p, 0x10, &0x20u32.to_le_bytes());
        put(&mut p, 0x14, &0x60u32.to_le_bytes());
        // Track 0: static, head position (1, 2, 3), focal 512.
        for (k, v) in [0.0f32, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 1.0].iter().enumerate() {
            put(&mut p, 0x20 + 4 * k, &v.to_le_bytes());
        }
        put(&mut p, 0x20 + 0x2C, &512.0f32.to_le_bytes());
        // Track 1: two keys at 1/15 s, focal 791.45.
        let r = 0x60;
        put(&mut p, r + 0x20, &2u32.to_le_bytes());
        put(&mut p, r + 0x24, &(1.0f32 / 15.0).to_le_bytes());
        put(&mut p, r + 0x2C, &791.45f32.to_le_bytes());
        put(&mut p, r + 0x34, &0x40u32.to_le_bytes());
        for k in 0..2 {
            let at = r + 0x40 + 0x30 * k;
            put(&mut p, at + 12, &1.0f32.to_le_bytes());
            put(&mut p, at + 0x10, &(10.0 * k as f32).to_le_bytes());
            put(&mut p, at + 0x24, &791.45f32.to_le_bytes());
        }
        let len = p.len();
        let top = vec![Chunk { offset: 0, tag: [0; 4], kind: CAMERA_TRACKS, extra: 0, size: len as u32, role_id: 0, payload: 0..len }];
        let t = camera_tracks(&p, &top);
        assert_eq!(t.len(), 2);
        assert!(t[0].keys.is_empty());
        assert_eq!((t[0].head.p, t[0].head.focal), ([1.0, 2.0, 3.0], 512.0));
        assert_eq!(t[1].keys.len(), 2);
        assert_eq!(t[1].keys[1].p, [10.0, 0.0, 0.0]);
        assert_eq!(t[1].head.focal, 791.45);
    }
}
