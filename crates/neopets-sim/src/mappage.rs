//! The map page and the quest journal (SPEC-0051; SPEC-0035 sections 14.3 and 14.4): the
//! per-area projection, the two pulses, the map's icon list and cursor, the pending target, the
//! journal's rows, its layout and scrolling. The host reads the quest and treasure rows from the
//! item-table file and supplies the flags; drawing and the button words belong to the front end.

use std::f32::consts::PI;

pub const AREAS: usize = 77;
/// The icon list's capacity (SPEC-0051 section 2).
pub const ICONS: usize = 32;

/// The projection records (SPEC-0051 Layout, projection values): per area (a, b, c, d, tx, tz,
/// yaw), with a = G0.x, b = G0.z, c = G2.x, d = G2.z, tx = G3.x, tz = G3.z and the yaw at +0x40.
/// The game builds these at start-up in code (no data table holds them); the spec publishes
/// them by the user's decision as constants for the clean side.
pub const PROJECTION: [[f32; 7]; AREAS] = [
    [1.04305, 0.0, 0.0, 1.21474, 192.63324, 331.59692, 0.0],
    [0.73112, -0.00029, 0.00025, 0.85362, -957.2571, 1710.0322, 0.000345],
    [0.73208, -0.00029, 0.00025, 0.85412, 253.54434, 190.18715, 0.000345],
    [0.65231, -0.00026, 0.00023, 0.75849, 234.58755, 172.76564, 0.000345],
    [0.44305, 0.0, 0.0, 0.5143, 165.83887, 193.02713, 0.0],
    [0.57417, 0.0, 0.0, 0.66975, -171.63916, 1615.2902, 0.0],
    [0.70419, -0.0004, 0.00034, 0.82166, -220.28339, 2016.964, 0.000488],
    [0.617, -0.00035, 0.0003, 0.72022, -313.33118, 1651.3778, 0.000488],
    [0.35059, 0.0, 0.0, 0.40805, 718.5642, -256.51343, 0.0],
    [0.64429, -0.00025, 0.00022, 0.73077, 1505.665, -597.9626, 0.000345],
    [1.18491, 0.0, 0.0, 1.38002, 283.44287, 232.33742, 0.0],
    [0.45427, 0.0, 0.0, 0.53013, 215.2739, 214.32294, 0.0],
    [0.9572, -0.00041, 0.00033, 1.18978, 286.25964, 334.36316, 0.000345],
    [0.38711, 0.0, 0.0, 0.45204, -797.09705, 430.53616, 0.0],
    [0.72885, -0.0003, 0.00025, 0.85459, -1365.1035, 958.83716, 0.000345],
    [0.37604, 0.0, 0.0, 0.43823, -465.04755, 101.71341, 0.0],
    [0.36839, -0.00021, 0.00018, 0.43094, -431.25006, 389.7728, 0.000488],
    [0.38184, -0.00015, 0.00013, 0.44318, -737.1537, 151.77046, 0.000345],
    [0.32714, -0.00013, 0.00011, 0.38245, -95.92188, 521.6359, 0.000345],
    [0.38953, 0.0, 0.0, 0.45489, -387.0746, 574.9614, 0.0],
    [1.30511, 0.0, 0.0, 1.51934, -131.36536, -957.3627, 0.0],
    [0.22305, -0.00009, 0.00008, 0.26017, 150.11417, 1.81174, 0.000345],
    [0.32771, 0.0, 0.0, 0.37879, -386.68848, 581.7608, 0.0],
    [-0.10139, 1.18223, -1.18223, -0.10139, 315.84497, 229.77393, -1.656349],
    [0.68568, -0.00048, 0.00041, 0.79675, 308.82764, 211.34525, 0.000598],
    [0.92241, 0.0, 0.0, 1.07279, 250.90172, 238.49026, 0.0],
    [0.38036, 0.0, 0.0, 0.44343, 294.1, 576.4526, 0.0],
    [0.44053, 0.0, 0.0, 0.5157, 120.11545, 273.18677, 0.0],
    [0.56961, 0.0, 0.0, 0.66719, 248.41087, 400.9008, 0.0],
    [0.33847, -0.00019, 0.00017, 0.39741, 244.25713, 311.15204, 0.000488],
    [0.35651, 0.0, 0.0, 0.41585, 169.96088, 225.18387, 0.0],
    [0.34931, 0.0, 0.0, 0.4053, 331.83493, 226.06445, 0.0],
    [0.35237, 0.0, 0.0, 0.40996, 258.24988, 137.00436, 0.0],
    [0.34904, 0.0, 0.0, 0.40672, 258.58884, -105.53748, 0.0],
    [0.65156, 0.0, 0.0, 0.76094, 293.32556, 269.6815, 0.0],
    [0.38399, 0.0, 0.0, 0.44641, 265.47998, 201.6826, 0.0],
    [0.87737, 0.0, 0.0, 1.02212, 257.01053, 239.55505, 0.0],
    [0.28616, 0.0, 0.0, 0.33129, 194.7616, 276.60623, 0.0],
    [0.41515, 0.0, 0.0, 0.48363, 635.60187, -6.03427, 0.0],
    [0.45944, -0.00047, 0.00022, 0.95824, 671.56085, -236.12561, 0.000488],
    [0.63658, 0.0, 0.0, 0.7439, -111.51642, 28.27921, 0.0],
    [0.60065, -0.00024, 0.00021, 0.69987, 357.0287, 194.13304, 0.000345],
    [-0.71084, -0.0553, 0.0553, -0.71085, -150.3432, 229.58147, 3.063948],
    [0.35617, 0.0, 0.0, 0.41576, 328.5676, 203.492, 0.0],
    [0.25196, -0.00014, 0.00012, 0.29312, 265.36176, 400.99683, 0.000488],
    [0.67641, 0.0, 0.0, 0.78877, 5.35699, 632.25604, 0.0],
    [1.09091, 0.0, 0.0, 1.27516, 861.72815, -1127.2908, 0.0],
    [0.44157, 0.0, 0.0, 0.51565, 203.73636, -87.88193, 0.0],
    [1.14567, -0.00046, 0.0004, 1.33554, 263.67035, 222.63002, 0.000345],
    [1.14191, 0.0, 0.0, 1.33554, 263.21747, 222.5679, 0.0],
    [2.46535, 0.0, 0.0, 2.95745, 238.80197, 709.2766, 0.0],
    [2.53535, 0.0, 0.0, 2.91765, 240.48485, 682.02356, 0.0],
    [2.55844, -0.00099, 0.00088, 2.86813, 252.73619, 669.1474, 0.000345],
    [0.57457, -0.00023, 0.0002, 0.66891, 208.85158, -131.62158, 0.000345],
    [0.64883, -0.00026, 0.00022, 0.7538, 533.3732, 658.18097, 0.000345],
    [0.60477, -0.00034, 0.0003, 0.70501, -349.73358, 142.95248, 0.000488],
    [0.78823, 0.0, 0.0, 0.91798, 255.75012, 220.86972, 0.0],
    [0.42387, 0.0, 0.0, 0.49466, 255.50443, 254.9581, 0.0],
    [1.72955, -0.00086, 0.0006, 2.4998, 278.14407, 217.48892, 0.000345],
    [1.0397, 0.0, 0.0, 1.21344, 266.86786, 208.42456, 0.0],
    [0.43539, 0.0, 0.0, 0.50876, 271.8101, 203.07576, 0.0],
    [0.5098, -0.00029, 0.00025, 0.59404, -1092.0647, 730.4917, 0.000488],
    [0.47727, -0.00018, 0.00016, 0.525, -1598.9265, 166.53557, 0.000345],
    [0.29053, 0.0, 0.0, 0.33881, -694.8701, 410.01135, 0.0],
    [0.32949, 0.0, 0.0, 0.38438, 602.2047, 221.44446, 0.0],
    [0.64631, -0.00026, 0.00022, 0.75147, 907.9389, 908.72943, 0.000345],
    [0.27838, 0.0, 0.0, 0.32583, 224.31708, -324.46503, 0.0],
    [0.28334, 0.0, 0.0, 0.33004, 293.4334, -501.8175, 0.0],
    [1.69042, 0.0, 0.0, 1.9805, 1378.4886, -4444.102, 0.0],
    [0.29846, 0.0, 0.0, 0.34723, 461.61646, -457.36328, 0.0],
    [0.67599, 0.0, 0.0, 0.78785, -585.1721, 454.28915, 0.0],
    [0.33507, 0.0, 0.0, 0.39156, 354.892, 458.7405, 0.0],
    [0.36866, -0.00014, 0.00013, 0.4041, -238.93832, 894.6209, 0.000345],
    [0.42619, 0.0, 0.0, 0.50336, 539.8159, -145.75848, 0.0],
    [0.57686, 0.0, 0.0, 0.67002, 273.63312, 1180.992, 0.0],
    [1.5625, 0.0, 0.0, 1.8125, 261.0, 802.0, 0.0],
    [0.84251, 0.0, 0.0, 0.98111, 254.48405, 227.11746, 0.0],
];

/// Round half away from zero to a whole pixel.
fn round(v: f32) -> i32 {
    v.round() as i32
}

/// The screen point of a world point in an area's picture (SPEC-0051 section 7); none for an
/// area outside 0 to 76.
pub fn project(area: i32, x: f32, z: f32) -> Option<(i32, i32)> {
    let [a, b, c, d, tx, tz, _] = *PROJECTION.get(usize::try_from(area).ok()?)?;
    Some((round(a * x + c * z + tx), round(b * x + d * z + tz)))
}

/// The hero marker's angle: the facing minus the area's yaw offset, plus pi.
pub fn marker_angle(area: i32, facing: f32) -> f32 {
    let yaw = usize::try_from(area).ok().and_then(|k| PROJECTION.get(k)).map_or(0.0, |p| p[6]);
    facing - yaw + PI
}

/// The hero facing as an angle from +Z: the arc cosine of the dot product, negated when the
/// facing's x is positive. The spec leaves the side at Medium; the screen settles it: world +X is
/// screen right and the marker turns clockwise, so a hero facing +X needs pi / 2 after the pi term
/// (user's report 22: the pointer showed the opposite way).
pub fn facing_angle(f: [f32; 3]) -> f32 {
    let l = (f[0] * f[0] + f[2] * f[2]).sqrt();
    if l == 0.0 {
        return 0.0;
    }
    let a = (f[2] / l).clamp(-1.0, 1.0).acos();
    if f[0] > 0.0 { -a } else { a }
}

/// The marker's colour channel: 128 times the float pulse, rounded half away from zero.
pub fn marker_channel(pulse: f32) -> u8 {
    round(128.0 * pulse).clamp(0, 255) as u8
}

/// The pulse words (boot data 0x2DD310 to 0x2DD32F, 8 words).
pub const PULSE_WORDS: u32 = 0x002D_D310;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PulseWords {
    pub int_start: i32,
    pub int_step: i32,
    pub int_hi: i32,
    pub int_lo: i32,
    pub start: f32,
    pub step: f32,
    pub hi: f32,
    pub lo: f32,
}

impl Default for PulseWords {
    /// Neutral words for a missing boot table (the scale stays 4096 and the pulse 1.0).
    fn default() -> Self {
        Self { int_start: 4096, int_step: 0, int_hi: 4096, int_lo: 4096, start: 1.0, step: 0.0, hi: 1.0, lo: 1.0 }
    }
}

impl PulseWords {
    /// The words from 0x2DD310: the integer upper and lower bounds (16 bits each at 0x2DD310 and
    /// 0x2DD312), the step (16 bits at 0x2DD314), the float lower and upper bounds (0x2DD31C,
    /// 0x2DD320), the float step (0x2DD324), the integer start (0x2DD328) and the float start
    /// (0x2DD32C).
    pub fn read(w: &[u32]) -> Option<Self> {
        if w.len() < 8 {
            return None;
        }
        let half = |k: usize, hi: bool| (if hi { (w[k] >> 16) as u16 } else { w[k] as u16 }) as i16 as i32;
        Some(Self {
            int_hi: half(0, false),
            int_lo: half(0, true),
            int_step: half(1, false),
            lo: f32::from_bits(w[3]),
            hi: f32::from_bits(w[4]),
            step: f32::from_bits(w[5]),
            int_start: w[6] as i32,
            start: f32::from_bits(w[7]),
        })
    }
}

/// The map's two pulses (SPEC-0051 section 5): the cursor icon's integer scale (4.12) and the
/// hero marker's float brightness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pulses {
    pub scale: i32,
    scale_step: i32,
    pub bright: f32,
    bright_step: f32,
}

impl Pulses {
    pub fn new(w: &PulseWords) -> Self {
        Self { scale: w.int_start, scale_step: w.int_step, bright: w.start, bright_step: w.step }
    }

    pub fn step(&mut self, w: &PulseWords) {
        let v = self.scale + self.scale_step;
        if v > w.int_hi {
            self.scale = w.int_hi;
            self.scale_step = -w.int_step.abs();
        } else if v < w.int_lo {
            self.scale = w.int_lo;
            self.scale_step = w.int_step.abs();
        } else {
            self.scale = v;
        }
        self.bright += self.bright_step;
        if self.bright > w.hi {
            self.bright = w.hi;
            self.bright_step = -self.bright_step;
        } else if self.bright < w.lo {
            self.bright = w.lo;
            self.bright_step = -self.bright_step;
        }
    }
}

/// A quest row (table index 1; SPEC-0051 Layout) with its texts resolved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quest {
    pub kind: u32,
    pub start: i32,
    pub done: i32,
    pub title: Vec<u8>,
    pub objective: Vec<u8>,
    pub stage: u32,
    pub x: f32,
    pub z: f32,
    pub scene: i32,
}

fn word(r: &[u8], o: usize) -> Option<u32> {
    r.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
}

impl Quest {
    /// A 36-byte row, with the text pointers resolved by `text`.
    pub fn read(r: &[u8], text: &dyn Fn(u32) -> Vec<u8>) -> Option<Self> {
        Some(Self {
            kind: word(r, 0)?,
            start: word(r, 4)? as i32,
            done: word(r, 8)? as i32,
            title: text(word(r, 0x0C)?),
            objective: text(word(r, 0x10)?),
            stage: word(r, 0x14)?,
            x: f32::from_bits(word(r, 0x18)?),
            z: f32::from_bits(word(r, 0x1C)?),
            scene: word(r, 0x20)? as i32,
        })
    }

    /// Accepted: neither flag id -1, the start flag set and the done flag clear.
    pub fn accepted(&self, flag: &dyn Fn(i32) -> bool) -> bool {
        self.start != -1 && self.done != -1 && flag(self.start) && !flag(self.done)
    }
}

/// A treasure row (table index 3).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Treasure {
    pub name: Vec<u8>,
    pub x: f32,
    pub z: f32,
    pub area: i32,
}

impl Treasure {
    pub fn read(r: &[u8], text: &dyn Fn(u32) -> Vec<u8>) -> Option<Self> {
        let name = word(r, 0)?;
        Some(Self { name: if name == 0 { Vec::new() } else { text(name) }, x: f32::from_bits(word(r, 4)?), z: f32::from_bits(word(r, 8)?), area: word(r, 0x0C)? as i32 })
    }
}

/// A row reference (the pending target, a journal row, an icon's items).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Quest(usize),
    Treasure(usize),
}

/// A map icon: world x and z, marker kind (0 primary, 1 secondary, 2 treasure), its row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Icon {
    pub x: f32,
    pub z: f32,
    pub kind: u8,
    pub target: Target,
}

/// The game's flags as the lists need them: a progress flag, and treasure i's held map and
/// collected flag (entry 200 + i).
pub struct Flags<'a> {
    pub flag: &'a dyn Fn(i32) -> bool,
    pub held: &'a dyn Fn(usize) -> bool,
    pub collected: &'a dyn Fn(usize) -> bool,
}

impl Flags<'_> {
    /// A listed treasure: its map held, its area not -1, its collected flag clear.
    fn listed(&self, t: &Treasure, i: usize) -> bool {
        (self.held)(i) && t.area != -1 && !(self.collected)(i)
    }
}

/// The icon list of an area (SPEC-0051 section 2): quest icons (accepted, stage nonzero, scene =
/// area) in table order, then treasure icons in table order; at most 32.
pub fn icons(quests: &[Quest], treasures: &[Treasure], area: i32, f: &Flags) -> Vec<Icon> {
    let q = quests.iter().enumerate().filter(|(_, q)| q.accepted(f.flag) && q.stage != 0 && q.scene == area).map(|(i, q)| Icon { x: q.x, z: q.z, kind: (q.stage - 1).min(255) as u8, target: Target::Quest(i) });
    let t = treasures.iter().enumerate().filter(|(i, t)| f.listed(t, *i) && t.area == area).map(|(i, t)| Icon { x: t.x, z: t.z, kind: 2, target: Target::Treasure(i) });
    q.chain(t).take(ICONS).collect()
}

/// The journal's rows (SPEC-0035 section 14.3): accepted quests, then listed treasures.
pub fn journal_rows(quests: &[Quest], treasures: &[Treasure], f: &Flags) -> Vec<Target> {
    let q = quests.iter().enumerate().filter(|(_, q)| q.accepted(f.flag)).map(|(i, _)| Target::Quest(i));
    let t = treasures.iter().enumerate().filter(|(i, t)| f.listed(t, *i)).map(|(i, _)| Target::Treasure(i));
    q.chain(t).collect()
}

/// A row's area: the quest's scene, else the treasure's area; none when negative.
pub fn area_of(t: Target, quests: &[Quest], treasures: &[Treasure]) -> Option<i32> {
    match t {
        Target::Quest(i) => quests.get(i).map(|q| q.scene),
        Target::Treasure(i) => treasures.get(i).map(|t| t.area),
    }
    .filter(|&a| a >= 0)
}

/// The map cursor's move (SPEC-0051 section 4, step 3): Up or Right to the next icon, else Down
/// or Left to the previous one, wrapping; 0 with an empty list.
pub fn map_cursor(cursor: usize, n: usize, next: bool, prev: bool) -> usize {
    if n == 0 {
        return 0;
    }
    if next {
        (cursor + 1) % n
    } else if prev {
        (cursor + n - 1) % n
    } else {
        cursor.min(n - 1)
    }
}

/// One drawn journal row: its index, offset p and the objective lines shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Laid {
    pub row: usize,
    pub p: i32,
    pub lines: usize,
}

/// The journal layout from the top index (SPEC-0035 section 14.3): each row starts where the
/// previous one ended; its objective shows up to trunc((360 - (128 + p)) / line) + 2 lines
/// (none at a limit of 0 or less); its height is max(79, 75 + line x L); a row ending at 360 or
/// more is not drawn, one ending at 360 or less counts as fully shown. Returns the drawn rows,
/// the last fully shown row and the more-below flag.
pub fn layout(top: usize, lines: &[usize], line: i32) -> (Vec<Laid>, Option<usize>, bool) {
    let mut out = Vec::new();
    let mut full = None;
    let mut p = 0i32;
    let mut cut = false;
    let mut past = false;
    for (row, &n) in lines.iter().enumerate().skip(top) {
        let limit = (360 - (128 + p)) / line.max(1) + 2;
        let shown = if limit <= 0 { 0 } else { n.min(limit as usize) };
        let next = p + 79.max(75 + line * shown as i32);
        if next <= 360 {
            full = Some(row);
        }
        if next >= 360 {
            past = true;
            break;
        }
        cut |= shown < n;
        out.push(Laid { row, p, lines: shown });
        p = next;
    }
    let further = out.last().map_or(top, |l| l.row + 1) < lines.len();
    (out, full, further && (past || cut))
}

/// The journal's cursor, top index and bar pulse (SPEC-0035 section 14.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Journal {
    pub cursor: i32,
    pub top: usize,
    pub pulse: f32,
    step: f32,
}

impl Default for Journal {
    fn default() -> Self {
        Self { cursor: 0, top: 0, pulse: 1.8, step: 0.0 }
    }
}

impl Journal {
    /// The entry: cursor and top at 0, or the cursor on the pending row.
    pub fn open(&mut self, rows: &[Target], pending: Option<Target>) {
        self.top = 0;
        self.cursor = pending.and_then(|p| rows.iter().position(|&r| r == p)).map_or(0, |k| k as i32);
    }

    /// The keys: Down first (+1, clamped to the last row; -1 with no rows), else Up (-1, clamped
    /// to 0); a move sets the pulse to its maximum. Returns whether a key moved the cursor.
    pub fn keys(&mut self, n: usize, down: bool, up: bool, hi: f32) -> bool {
        if down {
            self.cursor = (self.cursor + 1).min(n as i32 - 1);
        } else if up {
            self.cursor = (self.cursor - 1).max(0);
        } else {
            return false;
        }
        self.pulse = hi;
        true
    }

    /// The pulse step: add the step, bounce at the bounds (the step keeps its sign).
    pub fn pulse(&mut self, lo: f32, hi: f32, step: f32) {
        if self.step == 0.0 {
            self.step = step;
        }
        self.pulse += self.step;
        if self.pulse > hi {
            self.pulse = hi;
            self.step = -self.step;
        } else if self.pulse < lo {
            self.pulse = lo;
            self.step = -self.step;
        }
    }

    /// The scroll: one row per tick toward a cursor below the last fully shown row or above
    /// the top.
    pub fn scroll(&mut self, last_full: Option<usize>) {
        if self.cursor < 0 {
            return;
        }
        let c = self.cursor as usize;
        if last_full.is_some_and(|l| c > l) {
            self.top += 1;
        } else if c < self.top {
            self.top -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_marker_and_pulses() {
        // Area 75: (-100, -400) gives (104.75, 77.0), rounded to (105, 77).
        assert_eq!(project(75, -100.0, -400.0), Some((105, 77)));
        assert_eq!(project(77, 0.0, 0.0), None);
        // The marker angle subtracts the yaw.
        assert!((marker_angle(0, 0.2) - (0.2 + PI)).abs() < 1e-6);
        assert!((marker_angle(23, 0.0) - (1.656349 + PI)).abs() < 1e-5);
        assert_eq!((marker_channel(0.8), marker_channel(1.0), marker_channel(1.5)), (102, 128, 192));
        // Pulse words as the boot bytes would hold them, and the spec's vectors.
        let w = PulseWords::read(&[4300 | (3900 << 16), 50, 0, 0.8f32.to_bits(), 1.5f32.to_bits(), 0.03f32.to_bits(), 4096, 1.0f32.to_bits()]).unwrap();
        assert_eq!((w.int_hi, w.int_lo, w.int_step, w.int_start), (4300, 3900, 50, 4096));
        let mut p = Pulses::new(&w);
        p.scale = 4290;
        p.step(&w);
        assert_eq!(p.scale, 4300);
        p.step(&w);
        assert_eq!(p.scale, 4250);
        let mut p = Pulses::new(&w);
        p.bright = 1.45;
        p.step(&w);
        assert!((p.bright - 1.48).abs() < 1e-5);
        p.step(&w);
        assert_eq!(p.bright, 1.5);
        p.step(&w);
        assert!((p.bright - 1.47).abs() < 1e-5);
    }

    fn quest(start: i32, done: i32, stage: u32, scene: i32) -> Quest {
        Quest { start, done, stage, scene, title: b"Q".to_vec(), ..Default::default() }
    }

    #[test]
    fn icons_and_rows() {
        let flag = |f: i32| f == 107;
        let held = |i: usize| i == 13 || i == 1;
        let none = |_: usize| false;
        let f = Flags { flag: &flag, held: &held, collected: &none };
        let mut treasures = vec![Treasure { area: -1, ..Default::default() }; 14];
        treasures[13].area = 3;
        treasures[1].area = 63;
        // Stage 1, scene 9: one kind-0 icon; stage 2: kind 1; stage 0: none.
        assert_eq!(icons(&[quest(107, 108, 1, 9)], &treasures, 9, &f).iter().map(|i| i.kind).collect::<Vec<_>>(), vec![0]);
        assert_eq!(icons(&[quest(107, 108, 2, 9)], &treasures, 9, &f)[0].kind, 1);
        assert!(icons(&[quest(107, 108, 0, 9)], &treasures, 9, &f).is_empty());
        // The done flag set: no icon; an id of -1: never.
        let both = |x: i32| x == 107 || x == 108;
        assert!(icons(&[quest(107, 108, 1, 9)], &treasures, 9, &Flags { flag: &both, held: &held, collected: &none }).is_empty());
        assert!(icons(&[quest(107, -1, 1, 9)], &treasures, 9, &f).is_empty());
        // Treasure 13 in area 3, none once collected; treasure 1 only in area 63.
        assert_eq!(icons(&[], &treasures, 3, &f)[0].target, Target::Treasure(13));
        let c13 = |i: usize| i == 13;
        assert!(icons(&[], &treasures, 3, &Flags { flag: &flag, held: &held, collected: &c13 }).is_empty());
        assert_eq!(icons(&[], &treasures, 63, &f).len(), 1);
        // Order: the quest, then the treasure; the journal has no area filter.
        let q = [quest(107, 108, 1, 3)];
        assert_eq!(icons(&q, &treasures, 3, &f).iter().map(|i| i.target).collect::<Vec<_>>(), vec![Target::Quest(0), Target::Treasure(13)]);
        assert_eq!(journal_rows(&q, &treasures, &f), vec![Target::Quest(0), Target::Treasure(1), Target::Treasure(13)]);
        assert_eq!(area_of(Target::Treasure(1), &q, &treasures), Some(63));
        assert_eq!(area_of(Target::Treasure(0), &q, &treasures), None);
        // Cursor: A, B, C on A, Down wraps to C; Right from C to A; both pressed: next wins.
        assert_eq!(map_cursor(0, 3, false, true), 2);
        assert_eq!(map_cursor(2, 3, true, false), 0);
        assert_eq!(map_cursor(1, 3, true, true), 2);
        assert_eq!(map_cursor(0, 0, true, false), 0);
    }

    #[test]
    fn journal_layout_and_keys() {
        // Line height 14: a treasure row is 79 high, quest rows with 1 to 3 lines 89, 103, 117.
        let (rows, full, more) = layout(0, &[0, 1, 2, 3, 0], 14);
        // The fourth row starts at 271, where the line limit is 0: no objective, 79 high, so it
        // ends at 350; the fifth would end at 429: not drawn, and more below.
        assert_eq!(rows.iter().map(|l| (l.p, l.lines)).collect::<Vec<_>>(), vec![(0, 0), (79, 1), (168, 2), (271, 0)]);
        assert_eq!((full, more), (Some(3), true));
        assert!(!layout(0, &[0, 1], 14).2);
        // Four treasure rows end at 316; a fifth would end at 395.
        let (rows, full, _) = layout(0, &[0, 0, 0, 0, 0], 14);
        assert_eq!((rows.len(), full), (4, Some(3)));
        // A row ending exactly at 360 is hidden but counts as fully shown (103 + 89 + 89 + 79).
        let (rows, full, _) = layout(0, &[2, 1, 1, 0], 14);
        assert_eq!((rows.len(), full), (3, Some(3)));
        // Keys: Down clamps to the last row, Up to 0; with no rows Down gives -1.
        let mut j = Journal::default();
        assert!(j.keys(2, true, false, 1.8));
        assert!(j.keys(2, true, false, 1.8));
        assert_eq!(j.cursor, 1);
        j.keys(2, false, true, 1.8);
        j.keys(2, false, true, 1.8);
        assert_eq!(j.cursor, 0);
        let mut j = Journal::default();
        j.keys(0, true, false, 1.8);
        assert_eq!(j.cursor, -1);
        j.keys(0, false, true, 1.8);
        assert_eq!(j.cursor, 0);
        // A pending row: the cursor starts on it.
        j.open(&[Target::Quest(4), Target::Treasure(2)], Some(Target::Treasure(2)));
        assert_eq!(j.cursor, 1);
        // Scroll one row per tick.
        j.cursor = 5;
        j.scroll(Some(3));
        assert_eq!(j.top, 1);
        j.cursor = 0;
        j.scroll(Some(6));
        assert_eq!(j.top, 0);
    }
}
