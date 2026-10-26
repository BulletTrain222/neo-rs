//! The lit draw (SPEC-0046, Light slots and the lit draw; Scene default lights): an ambient
//! colour plus three directional lights, applied per vertex by the lit programs 1, 2 and 6.
//! The default rig is a set of code constants of the original (SPEC-0046 says they are not
//! disc data). The area's character light sets (type 30, SPEC-0028 section 2) are read here;
//! the slot walk that applies them is `neopets_sim::charlight`.

/// A light context after its slots are filled: ambient A and, per directional slot, the unit
/// direction toward the light (the negated travel direction) and its weighted colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lights {
    pub ambient: [f32; 3],
    pub directional: [([f32; 3], [f32; 3]); 3],
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 { v.map(|c| c / l) } else { v }
}

impl Lights {
    /// SPEC-0046 Scene default lights: ambient (50, 50, 50) and three type-2 lights of strength
    /// 0.25 with the travel directions and colours of its table.
    pub fn scene_default() -> Self {
        let slot = |travel: [f32; 3], colour: [f32; 3]| {
            (normalise(travel.map(|c| -c)), colour.map(|c| c * 0.25))
        };
        Self {
            ambient: [50.0; 3],
            directional: [
                slot([0.666, -0.524, 0.531], [255.0, 255.0, 195.0]),
                slot([0.193, -0.598, -0.778], [205.0, 205.0, 255.0]),
                slot([-0.649, -0.665, 0.368], [205.0, 205.0, 255.0]),
            ],
        }
    }

    /// The lit colour L for a normal, per RGB channel in 0–255 units.
    pub fn lit(&self, n: [f32; 3]) -> [f32; 3] {
        let mut l = self.ambient;
        for (t, c) in &self.directional {
            let d = (n[0] * t[0] + n[1] * t[1] + n[2] * t[2]).clamp(0.0, 1.0);
            for k in 0..3 {
                l[k] += d * c[k];
            }
        }
        l
    }

    /// Lit vertex output (programs 1, 2 and 6, draw colour scale 1): RGB = min(255, L × byte
    /// / 16) and alpha = min(255, 128 × alpha byte / 16), truncated.
    pub fn vertex(&self, n: [f32; 3], colour: [u8; 4]) -> [u8; 4] {
        let l = self.lit(n);
        let ch = |k: usize| (l[k] * colour[k] as f32 / 16.0).min(255.0) as u8;
        [ch(0), ch(1), ch(2), (128.0 * colour[3] as f32 / 16.0).min(255.0) as u8]
    }
}

/// The light-set chunk of a world pack (SPEC-0028 section 2).
pub const LIGHT_CHUNK: u16 = 30;

/// A light-set entry (0x50 bytes): position (+0x00), unit travel direction (+0x10), colour
/// R, G, B, alpha (+0x20), type (+0x30; the array decides it), strength (+0x34), flag (+0x38),
/// radius (+0x3C), radius squared (+0x40) and the zone key's low byte (+0x44).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightEntry {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub colour: [f32; 4],
    pub kind: i32,
    pub strength: f32,
    pub flag: i32,
    pub radius: f32,
    pub radius_sq: f32,
    pub zone: u8,
}

/// A light-set record: its gate word (+0x02; nonzero means character lighting skips it) and
/// arrays A (type 2), B (type 3) and C (type 1) in that order after the 0x20-byte header.
#[derive(Clone, Debug, PartialEq)]
pub struct LightRecord {
    pub gate: i16,
    pub a: Vec<LightEntry>,
    pub b: Vec<LightEntry>,
    pub c: Vec<LightEntry>,
}

fn entry(p: &[u8], at: usize, kind: i32) -> Option<LightEntry> {
    let f = |o: usize| p.get(at + o..at + o + 4).map(|b| f32::from_le_bytes(b.try_into().unwrap()));
    let i = |o: usize| p.get(at + o..at + o + 4).map(|b| i32::from_le_bytes(b.try_into().unwrap()));
    Some(LightEntry {
        position: [f(0)?, f(4)?, f(8)?],
        direction: [f(0x10)?, f(0x14)?, f(0x18)?],
        colour: [f(0x20)?, f(0x24)?, f(0x28)?, f(0x2C)?],
        kind,
        strength: f(0x34)?,
        flag: i(0x38)?,
        radius: f(0x3C)?,
        radius_sq: f(0x40)?,
        zone: *p.get(at + 0x44)?,
    })
}

/// The light-set records of a walked world pack chain, in table order. Records follow the u32
/// table (at the payload's +0x04 offset, N words) in sequence from the table end rounded up to
/// 16, each 0x20 + 0x50 × (A + B + C) bytes rounded up to 16; the table values are not used.
/// The type of each entry is taken from its array.
pub fn light_sets(bytes: &[u8], chain: &[crate::resource::Chunk]) -> Vec<LightRecord> {
    let Some(c) = chain.iter().find(|c| c.kind == LIGHT_CHUNK) else { return Vec::new() };
    let Some(p) = bytes.get(c.payload.clone()) else { return Vec::new() };
    light_sets_of(p)
}

/// [`light_sets`] on a type-30 payload.
pub fn light_sets_of(p: &[u8]) -> Vec<LightRecord> {
    let word = |at: usize| p.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let half = |at: usize| p.get(at..at + 2).map(|b| i16::from_le_bytes(b.try_into().unwrap()));
    let (Some(n), Some(table)) = (word(0), word(4)) else { return Vec::new() };
    let n = (n as i32).clamp(0, 4096) as usize;
    let mut at = (table as usize + 4 * n).next_multiple_of(16);
    let mut out = Vec::new();
    for _ in 0..n {
        let (Some(gate), Some(na), Some(nb), Some(nc)) = (half(at + 2), half(at + 4), half(at + 6), half(at + 8)) else { break };
        let counts = [na, nb, nc].map(|k| k.max(0) as usize);
        let mut e = at + 0x20;
        let mut arrays: [Vec<LightEntry>; 3] = Default::default();
        for (k, kind) in [2, 3, 1].into_iter().enumerate() {
            for _ in 0..counts[k] {
                let Some(x) = entry(p, e, kind) else { return out };
                arrays[k].push(x);
                e += 0x50;
            }
        }
        let [a, b, c] = arrays;
        out.push(LightRecord { gate, a, b, c });
        at = e.next_multiple_of(16);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_sets_walk_records_in_sequence() {
        // Two records: N = 2, table at 0x10 (two unused words), first record at 0x20.
        let mut p = vec![0u8; 0x20];
        p[0..4].copy_from_slice(&2u32.to_le_bytes());
        p[4..8].copy_from_slice(&0x10u32.to_le_bytes());
        p[0x10..0x18].copy_from_slice(&[0xEE; 8]);
        let record = |gate: i16, a: i16, b: i16, c: i16| {
            let mut r = vec![0xCDu8; 0x20];
            r[0..2].copy_from_slice(&1i16.to_le_bytes());
            r[2..4].copy_from_slice(&gate.to_le_bytes());
            r[4..6].copy_from_slice(&a.to_le_bytes());
            r[6..8].copy_from_slice(&b.to_le_bytes());
            r[8..10].copy_from_slice(&c.to_le_bytes());
            for k in 0..(a + b + c) {
                let mut e = vec![0xCDu8; 0x50];
                let fl = |e: &mut Vec<u8>, at: usize, v: f32| e[at..at + 4].copy_from_slice(&v.to_le_bytes());
                fl(&mut e, 0, k as f32);
                fl(&mut e, 0x10, 0.0);
                fl(&mut e, 0x14, -1.0);
                fl(&mut e, 0x18, 0.0);
                fl(&mut e, 0x20, 255.0);
                e[0x30..0x34].copy_from_slice(&9i32.to_le_bytes());
                fl(&mut e, 0x34, 0.5);
                e[0x38..0x3C].copy_from_slice(&0i32.to_le_bytes());
                fl(&mut e, 0x3C, 10.0);
                fl(&mut e, 0x40, 100.0);
                e[0x44..0x48].copy_from_slice(&3u32.to_le_bytes());
                r.extend(e);
            }
            r
        };
        p.extend(record(1, 1, 0, 1));
        p.extend(record(0, 0, 2, 0));
        let sets = light_sets_of(&p);
        assert_eq!(sets.len(), 2);
        assert_eq!((sets[0].gate, sets[0].a.len(), sets[0].b.len(), sets[0].c.len()), (1, 1, 0, 1));
        // The array decides the type, not the stored word.
        assert_eq!((sets[0].a[0].kind, sets[0].c[0].kind, sets[1].b[1].kind), (2, 1, 3));
        assert_eq!(sets[0].c[0].position[0], 1.0);
        assert_eq!(sets[1].b[1].position[0], 1.0);
        assert_eq!((sets[1].b[0].zone, sets[1].b[0].radius_sq, sets[1].b[0].direction), (3, 100.0, [0.0, -1.0, 0.0]));
    }

    #[test]
    fn default_lights_match_spec_vectors() {
        let lights = Lights::scene_default();
        // SPEC-0004: normal up with bytes 16 gives (148, 148, 156, 128); normal down gives the
        // ambient alone.
        assert_eq!(lights.vertex([0.0, 1.0, 0.0], [16; 4]), [148, 148, 156, 128]);
        assert_eq!(lights.vertex([0.0, -1.0, 0.0], [16; 4])[..3], [50, 50, 50]);
        // SPEC-0046: the slot colours are strength × colour.
        assert_eq!(lights.directional[0].1, [63.75, 63.75, 48.75]);
    }

    #[test]
    fn output_saturates_at_255() {
        let lights = Lights::scene_default();
        assert_eq!(lights.vertex([0.0, 1.0, 0.0], [128, 128, 128, 128]), [255; 4]);
    }
}
