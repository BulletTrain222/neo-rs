//! SPEC-0028 section 11: the per-frame atmosphere zone selection from the camera eye. Each
//! frame the zones of the current world give a working record (overlay colour, post-pass
//! colour, render scalar) and the blend factor b that the sky dome's role-10 layer uses as its
//! opacity (SPEC-0046 Sky domes, step 6).

/// One zone (type 41 record, as rebased at load).
#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub depth: f32,
    pub key: u32,
    pub centre: [f32; 3],
    pub radius: f32,
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
}

/// The zone output: vectors A and B, the render scalar, and the blend factor b.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Output {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub depth: f32,
    pub blend: f32,
}

/// The zone range and world-change crossfade distance (0x2DB4CC) and the gated-zone blend
/// distance (0x2DB4D0).
pub const RANGE: f32 = 50.0;
pub const GATED: f32 = 25.0;

/// The gate key test: key 13 always passes; keys 1 to 7 and 9 to 12 need their first flag set
/// and their second clear; key 8 needs flag 248; other keys fail (key 0 is not a flag key).
pub fn key_passes(key: u32, flag: &dyn Fn(u16) -> bool) -> bool {
    let pair = |set: u16, clear: u16| flag(set) && !flag(clear);
    match key {
        1 => pair(83, 5),
        2 => pair(10, 322),
        3 => pair(64, 283),
        4 => pair(57, 58),
        5 => pair(74, 263),
        6 => pair(85, 298),
        7 => pair(92, 94),
        8 => flag(248),
        9 => pair(332, 100),
        10 => pair(332, 102),
        11 => pair(332, 104),
        12 => pair(332, 106),
        13 => true,
        _ => false,
    }
}

fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

fn lerp(a: &Output, b: &Output, t: f32) -> Output {
    Output { a: lerp4(a.a, b.a, t), b: lerp4(a.b, b.b, t), depth: a.depth + (b.depth - a.depth) * t, blend: a.blend }
}

fn record(z: &Zone) -> Output {
    Output { a: z.a, b: z.b, depth: z.depth, blend: 0.0 }
}

/// The horizontal distance g from q to a polygon zone and whether q is inside (step 3), or
/// None when its bounding circle is more than 50 away.
fn polygon(z: &Zone, q: [f32; 3]) -> Option<(f32, bool)> {
    let (dx, dz) = (q[0] - z.centre[0], q[2] - z.centre[2]);
    if dx.hypot(dz) - z.radius > RANGE {
        return None;
    }
    let n = z.points.len();
    if n == 0 {
        return None;
    }
    let d = |p: [f32; 2]| (q[0] - p[0]).hypot(q[2] - p[1]);
    let (k, _) = z.points.iter().enumerate().map(|(i, &p)| (i, d(p))).fold((0, f32::MAX), |m, (i, v)| if v < m.1 { (i, v) } else { m });
    if d(z.points[k]) <= 0.5 {
        return Some((0.0, true));
    }
    // The edges that meet at the nearest vertex (one at an open polyline's end).
    let mut edges = Vec::new();
    if k + 1 < n || z.closed {
        edges.push((z.points[k], z.points[(k + 1) % n]));
    }
    if k > 0 || z.closed {
        edges.push((z.points[(k + n - 1) % n], z.points[k]));
    }
    let mut best: Option<(f32, bool)> = None;
    for (s, e) in edges {
        let (ex, ez) = (e[0] - s[0], e[1] - s[1]);
        let len2 = (ex * ex + ez * ez).max(1e-12);
        let t = (((q[0] - s[0]) * ex + (q[2] - s[1]) * ez) / len2).clamp(0.0, 1.0);
        let g = (q[0] - (s[0] + t * ex)).hypot(q[2] - (s[1] + t * ez));
        // The y component of (edge direction) x (q - edge start).
        let cross_y = ez * (q[0] - s[0]) - ex * (q[2] - s[1]);
        if best.is_none_or(|b| g < b.0) {
            best = Some((g, cross_y > 0.0));
        }
    }
    best
}

/// The selection's kept state: the world the output belongs to, the eye there, and the
/// previous output (for the world-change crossfade).
#[derive(Clone, Debug, Default)]
pub struct Selector {
    world: Option<u32>,
    eye: [f32; 3],
    previous: Output,
}

impl Selector {
    /// One frame's output for the current world `world` and its zones, from the eye `q`.
    pub fn select(&mut self, world: u32, zones: &[Zone], q: [f32; 3], flag: &dyn Fn(u16) -> bool) -> Output {
        // 1. World change crossfade.
        let mut cross = None;
        if self.world != Some(world) {
            let d = (q[0] - self.eye[0]).hypot(q[1] - self.eye[1]).hypot(q[2] - self.eye[2]);
            if self.world.is_none() || d > RANGE {
                self.world = Some(world);
                self.eye = q;
            } else {
                cross = Some(1.0 - d / RANGE);
            }
        }
        // 2. Base zone: the first ungated one is the fallback; a gated one that passes is
        // taken at once.
        let mut base: Option<&Zone> = None;
        for z in zones.iter().filter(|z| z.points.is_empty()) {
            if z.key == 0 {
                base = base.or(Some(z));
            } else if key_passes(z.key, flag) {
                base = Some(z);
                break;
            }
        }
        let gated_base = base.is_some_and(|z| z.key != 0);
        let mut work = base.map_or(Output::default(), record);
        let mut b = if gated_base { 1.0f32 } else { 0.0 };
        // 3. Polygon zones.
        for z in zones.iter().filter(|z| !z.points.is_empty() && z.key != 0 && key_passes(z.key, flag)) {
            let Some((g, inside)) = polygon(z, q) else { continue };
            if inside || g < 1.0 {
                work = Output { blend: work.blend, ..record(z) };
            } else if g <= RANGE {
                work = lerp(&work, &record(z), 1.0 - (g - 1.0) / 49.0);
            }
            let term = if inside {
                if g > GATED || base.is_some_and(|z| z.key == 1) { 1.0 } else { g / GATED }
            } else if z.key == 0 && gated_base {
                g / GATED
            } else {
                0.0
            };
            b = b.max(term);
        }
        work.blend = b;
        // 4. Output.
        let out = match cross {
            Some(w) => Output { blend: work.blend + (self.previous.blend - work.blend) * w, ..lerp(&work, &self.previous, w) },
            None => work,
        };
        self.previous = out;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(key: u32, b: [f32; 4]) -> Zone {
        Zone { a: [0.0; 4], b, depth: 100000.0, key, centre: [0.0; 3], radius: 1.0, points: Vec::new(), closed: false }
    }

    #[test]
    fn gated_base_gives_full_blend_when_its_key_passes() {
        // Pack 1230's single zone: key 2 (flag 10 set, 322 clear), purple, alpha 28.
        let zones = [base(2, [35.0, 0.0, 55.0, 28.0])];
        let mut s = Selector::default();
        let on = |f: u16| f == 10;
        let out = s.select(1230, &zones, [0.0; 3], &on);
        assert_eq!((out.blend, out.b), (1.0, [35.0, 0.0, 55.0, 28.0]));
        let off = |_: u16| false;
        let out = Selector::default().select(1230, &zones, [0.0; 3], &off);
        assert_eq!((out.blend, out.b), (0.0, [0.0; 4]));
    }

    #[test]
    fn an_ungated_fallback_is_kept_when_the_gated_zone_fails() {
        // Pack 956: zone 0 ungated, zone 1 key 3 (flag 64 set, 283 clear).
        let zones = [base(0, [128.0, 128.0, 128.0, 0.0]), base(3, [35.0, 0.0, 55.0, 28.0])];
        let out = Selector::default().select(956, &zones, [0.0; 3], &|_| false);
        assert_eq!((out.blend, out.b[0]), (0.0, 128.0));
        let out = Selector::default().select(956, &zones, [0.0; 3], &|f| f == 64);
        assert_eq!((out.blend, out.b[0]), (1.0, 35.0));
    }

    #[test]
    fn key_tests() {
        assert!(key_passes(13, &|_| false));
        assert!(!key_passes(0, &|_| true));
        assert!(key_passes(8, &|f| f == 248));
        assert!(!key_passes(2, &|f| f == 10 || f == 322));
    }

    #[test]
    fn world_change_crossfade() {
        let a = [base(0, [100.0, 0.0, 0.0, 10.0])];
        let b = [base(0, [0.0, 0.0, 100.0, 10.0])];
        let mut s = Selector::default();
        s.select(1, &a, [0.0; 3], &|_| false);
        // 10 units from the stored eye in a new world: w = 0.8 of the previous output.
        let out = s.select(2, &b, [10.0, 0.0, 0.0], &|_| false);
        assert!((out.b[0] - 80.0).abs() < 1e-3 && (out.b[2] - 20.0).abs() < 1e-3);
        // 60 away: the crossfade ends and the new world is stored.
        let out = s.select(2, &b, [60.0, 0.0, 0.0], &|_| false);
        assert_eq!(out.b, [0.0, 0.0, 100.0, 10.0]);
    }
}
