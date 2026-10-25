//! The game's random generator (SPEC-0008 Random numbers): one MT19937 stream with the 1998
//! seeding rule (word i = 69069 × word i−1), and the three draw rules every system uses:
//! Range (closed integer range, unsigned remainder), Index (below n) and FRange (real range),
//! each with its no-draw shortcut. Seeded once (from the clock in play; a fixed seed in tests).

const N: usize = 624;
const M: usize = 397;
const MATRIX: u32 = 0x9908_B0DF;
const UPPER: u32 = 0x8000_0000;
const LOWER: u32 = 0x7FFF_FFFF;

#[derive(Clone, Debug)]
pub struct Rng {
    words: [u32; N],
    index: usize,
}

impl Rng {
    /// Seed by the 1998 rule; the first output regenerates first.
    pub fn seeded(seed: u32) -> Self {
        let mut words = [0u32; N];
        words[0] = seed;
        for i in 1..N {
            words[i] = words[i - 1].wrapping_mul(69069);
        }
        Self { words, index: N }
    }

    /// SPEC-0008 seed from a clock: (second + 1)(minute + 1)(hour + 1)(day + 1)(month + 1) + year,
    /// in 32 bits, with the raw clock bytes (BCD in the original).
    pub fn clock_seed(second: u8, minute: u8, hour: u8, day: u8, month: u8, year: u8) -> u32 {
        [second, minute, hour, day, month]
            .iter()
            .fold(1u32, |acc, &b| acc.wrapping_mul(b as u32 + 1))
            .wrapping_add(year as u32)
    }

    fn twist(&mut self) {
        for i in 0..N {
            let y = (self.words[i] & UPPER) | (self.words[(i + 1) % N] & LOWER);
            let mag = if y & 1 == 0 { 0 } else { MATRIX };
            self.words[i] = self.words[(i + M) % N] ^ (y >> 1) ^ mag;
        }
        self.index = 0;
    }

    /// One tempered output.
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.words[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9D2C_5680;
        y ^= (y << 15) & 0xEFC6_0000;
        y ^ (y >> 18)
    }

    /// Range(lo, hi): both ends included; swapped when hi < lo; no draw when equal.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let (lo, hi) = if hi < lo { (hi, lo) } else { (lo, hi) };
        if lo == hi {
            return lo;
        }
        let span = (hi as i64 - lo as i64 + 1) as u32;
        lo.wrapping_add((self.next_u32() % span) as i32)
    }

    /// Index(n): below n; no draw for n = 0.
    pub fn index(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        self.next_u32() % n
    }

    /// FRange(a, b): a0 + float(u) × (b0 − a0) × 2^−32 after ordering; no draw when a = b.
    pub fn frange(&mut self, a: f32, b: f32) -> f32 {
        if a == b {
            return a;
        }
        let (a0, b0) = if b < a { (b, a) } else { (a, b) };
        let s = (b0 - a0) * (1.0 / 4_294_967_296.0f32);
        self.next_u32() as f32 * s + a0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_outputs() {
        let first = |seed: u32| {
            let mut r = Rng::seeded(seed);
            (0..10).map(|_| r.next_u32()).collect::<Vec<_>>()
        };
        assert_eq!(first(1), [3796174982, 4182529786, 2180050607, 3809380472, 2960793660, 4051247694, 3989107140, 2094500863, 3349580919, 3767281029]);
        assert_eq!(first(4357), [3510405877, 4290933890, 2191955339, 564929546, 152112058, 4262624192, 2687398418, 268830360, 1763988213, 578848526]);
        assert_eq!(first(860282), [1830660608, 153682920, 3398361290, 1057550717, 3031632102, 2934766831, 3404529811, 574691166, 610019998, 1706036388]);
    }

    #[test]
    fn regeneration_boundary() {
        let mut r = Rng::seeded(1);
        let outs: Vec<u32> = (0..625).map(|_| r.next_u32()).collect();
        assert_eq!((outs[623], outs[624]), (751518116, 1141860845));
    }

    #[test]
    fn wrapper_vectors() {
        let five = |f: &mut dyn FnMut(&mut Rng) -> i64| {
            let mut r = Rng::seeded(1);
            (0..5).map(|_| f(&mut r)).collect::<Vec<_>>()
        };
        assert_eq!(five(&mut |r| r.range(0, 100) as i64), [92, 0, 48, 34, 72]);
        assert_eq!(five(&mut |r| r.index(100) as i64), [82, 86, 7, 72, 60]);
        assert_eq!(five(&mut |r| r.range(1, 100) as i64), [83, 87, 8, 73, 61]);
        let mut r = Rng::seeded(1);
        let f: Vec<f32> = (0..3).map(|_| r.frange(0.0, 1.0)).collect();
        assert!((f[0] - 0.8838659).abs() < 1e-6 && (f[1] - 0.9738211).abs() < 1e-6 && (f[2] - 0.5075826).abs() < 1e-6);
        let mut r = Rng::seeded(1);
        let f: Vec<f32> = (0..3).map(|_| r.frange(1.0, -1.0)).collect();
        assert!((f[0] - 0.7677318).abs() < 1e-6 && (f[1] - 0.9476422).abs() < 1e-6 && (f[2] - 0.0151652).abs() < 1e-6);
        let mut r = Rng::seeded(860282);
        assert_eq!((0..5).map(|_| r.range(0, 100)).collect::<Vec<_>>(), [56, 7, 49, 18, 43]);
        let mut r = Rng::seeded(1);
        assert!((r.frange(0.0, 10.0) - 8.8387).abs() < 1e-3);
    }

    #[test]
    fn no_draw_shortcuts() {
        let mut r = Rng::seeded(1);
        assert_eq!((r.range(5, 5), r.index(0), r.frange(0.0, -0.0)), (5, 0, 0.0));
        assert_eq!(r.next_u32(), 3796174982, "the stream is untouched");
    }
}
