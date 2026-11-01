//! The Bink version 1 (revision i) movie decoder (SPEC-0008 Movie decoder build guide): the
//! frame container, the per-plane lookup maps and value bundles, the eleven block types, the
//! coefficient and residue grammars, the integer 8 × 8 inverse transform and the BT.601 colour
//! conversion, and the revision-i audio codec. The fixed tables are read from the verified boot
//! program at run time (`Tables::read`); nothing of the disc is bundled.
use crate::{elf, movie::Header};

/// The boot program's fixed tables (SPEC-0008 Fixed tables).
#[derive(Clone, Debug)]
pub struct Tables {
    /// Per code family: the template bytes (2^width entries) and the index width.
    templates: Vec<Vec<u8>>,
    widths: [u8; 16],
    /// Coefficient permutation: coding index to natural position.
    perm: [u8; 64],
    scans: [[u8; 64]; 16],
    intra: [[i32; 64]; 16],
    inter: [[i32; 64]; 16],
    /// Audio: segment lengths (in units of 8), band edges (Hz), band multipliers.
    segments: [u8; 16],
    edges: [u32; 25],
    multipliers: Vec<f32>,
}

fn boot_bytes(boot: &[u8], vaddr: u32, len: usize) -> Result<&[u8], String> {
    let at = elf::file_offset(boot, vaddr, len as u32)? as usize;
    boot.get(at..at + len).ok_or_else(|| format!("Boot data at {vaddr:#x} is short"))
}

impl Tables {
    pub fn read(boot: &[u8]) -> Result<Self, String> {
        let mut widths = [0u8; 16];
        widths.copy_from_slice(boot_bytes(boot, 0x0032_5DD8, 16)?);
        let pointers = crate::archive::boot_u32_table(boot, 0x002F_7F80, 16)?;
        let mut templates = Vec::with_capacity(16);
        for (f, &p) in pointers.iter().enumerate() {
            let w = widths[f];
            if !(1..=8).contains(&w) {
                return Err(format!("Bink code family {f} has index width {w}"));
            }
            templates.push(boot_bytes(boot, p, 1 << w)?.to_vec());
        }
        let mut perm = [0u8; 64];
        perm.copy_from_slice(boot_bytes(boot, 0x0032_7AE0, 64)?);
        let scan_bytes = boot_bytes(boot, 0x0032_7B40, 16 * 64)?;
        let scans = std::array::from_fn(|q| std::array::from_fn(|k| scan_bytes[q * 64 + k]));
        let weights = |vaddr: u32| -> Result<[[i32; 64]; 16], String> {
            let b = boot_bytes(boot, vaddr, 16 * 64 * 4)?;
            Ok(std::array::from_fn(|q| std::array::from_fn(|k| i32::from_le_bytes(b[(q * 64 + k) * 4..(q * 64 + k) * 4 + 4].try_into().unwrap()))))
        };
        let mut segments = [0u8; 16];
        segments.copy_from_slice(boot_bytes(boot, 0x002F_9178, 16)?);
        let e = crate::archive::boot_u32_table(boot, 0x002F_9188, 25)?;
        let m = boot_bytes(boot, 0x002F_91F0, 256 * 4)?;
        Ok(Self {
            templates,
            widths,
            perm,
            scans,
            intra: weights(0x0032_7F60)?,
            inter: weights(0x0032_8F80)?,
            segments,
            edges: std::array::from_fn(|k| e[k]),
            multipliers: (0..256).map(|k| f32::from_le_bytes(m[k * 4..k * 4 + 4].try_into().unwrap())).collect(),
        })
    }
}

/// Least-significant-bit-first reader over little-endian bytes.
struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn peek(&self, n: u32) -> u32 {
        let mut v = 0u32;
        for i in 0..n as usize {
            let p = self.pos + i;
            let bit = self.data.get(p / 8).map_or(0, |b| (b >> (p % 8)) & 1);
            v |= (bit as u32) << i;
        }
        v
    }

    fn read(&mut self, n: u32) -> u32 {
        let v = self.peek(n);
        self.pos += n as usize;
        v
    }

    fn bit(&mut self) -> bool {
        self.read(1) == 1
    }

    /// The byte offset after rounding the bit position up to a 32-bit word.
    fn word_end(&self) -> usize {
        self.pos.div_ceil(32) * 4
    }

    fn overrun(&self) -> bool {
        self.pos > self.data.len() * 8
    }
}

/// A lookup map: its family (template) and the 16 symbols.
#[derive(Clone, Copy)]
struct Map {
    family: usize,
    symbols: [u8; 16],
}

impl Map {
    fn read(b: &mut Bits) -> Self {
        let family = b.read(4) as usize;
        let mut symbols: [u8; 16] = std::array::from_fn(|k| k as u8);
        if family == 0 {
            return Self { family, symbols };
        }
        if b.bit() {
            // List form: n + 1 listed symbols, then the rest in ascending order.
            let n = b.read(3) as usize + 1;
            let mut out = Vec::with_capacity(16);
            for _ in 0..n {
                let s = b.read(4) as u8;
                if !out.contains(&s) {
                    out.push(s);
                }
            }
            let rest: Vec<u8> = (0..16u8).filter(|s| !out.contains(s)).collect();
            out.extend(rest);
            symbols.copy_from_slice(&out[..16]);
        } else {
            // Merge form: passes for group sizes 1, 2, 4 ... 2^depth.
            let depth = b.read(2);
            let mut size = 1;
            for _ in 0..=depth {
                let mut out = [0u8; 16];
                let mut o = 0;
                for start in (0..16).step_by(2 * size) {
                    let (first, second) = (&symbols[start..start + size], &symbols[start + size..start + 2 * size]);
                    let (mut i, mut j) = (0, 0);
                    while i < size && j < size {
                        if b.bit() {
                            out[o] = second[j];
                            j += 1;
                        } else {
                            out[o] = first[i];
                            i += 1;
                        }
                        o += 1;
                    }
                    for &s in first[i..].iter().chain(&second[j..]) {
                        out[o] = s;
                        o += 1;
                    }
                }
                symbols = out;
                size *= 2;
            }
        }
        Self { family, symbols }
    }

    fn symbol(&self, b: &mut Bits, t: &Tables) -> u8 {
        let w = t.widths[self.family] as u32;
        let byte = t.templates[self.family][b.peek(w) as usize];
        b.pos += (byte >> 4) as usize;
        self.symbols[(byte & 15) as usize]
    }
}

/// A value bundle: the queue, its read position, the count width and the open flag.
struct Bundle {
    values: Vec<i32>,
    next: usize,
    width: u32,
    open: bool,
}

impl Bundle {
    fn new(n: u32) -> Self {
        Self { values: Vec::new(), next: 0, width: 32 - n.leading_zeros(), open: true }
    }

    /// The count of the next refill, or None when the bundle is still holding values or is
    /// closed (a count of 0 closes it).
    fn due(&mut self, b: &mut Bits) -> Option<usize> {
        if !self.open || self.next < self.values.len() {
            return None;
        }
        self.values.clear();
        self.next = 0;
        let n = b.read(self.width) as usize;
        if n == 0 {
            self.open = false;
            return None;
        }
        Some(n)
    }

    fn take(&mut self) -> Result<i32, String> {
        let v = *self.values.get(self.next).ok_or("Bink bundle ran out of values")?;
        self.next += 1;
        Ok(v)
    }
}

/// The nine bundles of a plane, in refill order.
const TYPES: usize = 0;
const NESTED: usize = 1;
const COLOURS: usize = 2;
const PATTERN: usize = 3;
const XOFF: usize = 4;
const YOFF: usize = 5;
const INTRA_DC: usize = 6;
const INTER_DC: usize = 7;
const RUN: usize = 8;

/// A plane of 8-bit samples.
#[derive(Clone, Debug, PartialEq)]
pub struct Plane {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Plane {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height, data: vec![0; width * height] }
    }

    fn block(&self, x: isize, y: isize) -> [u8; 64] {
        std::array::from_fn(|k| {
            let (px, py) = (x + (k % 8) as isize, y + (k / 8) as isize);
            if px < 0 || py < 0 || px as usize >= self.width || py as usize >= self.height {
                0
            } else {
                self.data[py as usize * self.width + px as usize]
            }
        })
    }

    fn put(&mut self, x: usize, y: usize, v: &[u8; 64]) {
        for r in 0..8 {
            let row = (y + r) * self.width + x;
            self.data[row..row + 8].copy_from_slice(&v[r * 8..r * 8 + 8]);
        }
    }
}

/// A decoded picture: Y (luma), U (the red difference, Cr) and V (the blue difference, Cb).
#[derive(Clone, Debug, PartialEq)]
pub struct Picture {
    pub planes: [Plane; 3],
}

impl Picture {
    pub fn new(width: usize, height: usize) -> Self {
        Self { planes: [Plane::new(width, height), Plane::new(width / 2, height / 2), Plane::new(width / 2, height / 2)] }
    }

    /// RGBA pixels by the BT.601 limited-range matrix (1.164 luma factor; SPEC-0008 Colour
    /// conversion), each luma with the chroma of its 2 × 2 cell.
    pub fn rgba(&self) -> Vec<u8> {
        let [y, cr, cb] = &self.planes;
        let mut out = Vec::with_capacity(y.width * y.height * 4);
        for py in 0..y.height {
            for px in 0..y.width {
                let l = (y.data[py * y.width + px] as f32 - 16.0).max(0.0) * 1.164;
                let c = (py / 2) * cr.width + px / 2;
                let (r, b) = (cr.data[c] as f32 - 128.0, cb.data[c] as f32 - 128.0);
                let px = |v: f32| v.round().clamp(0.0, 255.0) as u8;
                out.extend([px(l + 1.596 * r), px(l - (3209.0 / 8192.0) * b - 0.813 * r), px(l + 2.018 * b), 255]);
            }
        }
        out
    }
}

/// The one-dimensional transform of the build guide (Inverse transform), 32-bit integers.
fn idct8(s: [i32; 8]) -> [i32; 8] {
    let m = |k: i32, v: i32| k.wrapping_mul(v) >> 11;
    let a0 = s[0].wrapping_add(s[4]);
    let a1 = s[0].wrapping_sub(s[4]);
    let a2 = s[2].wrapping_add(s[6]);
    let a3 = m(2896, s[2].wrapping_sub(s[6]));
    let a4 = s[5].wrapping_add(s[3]);
    let a5 = s[5].wrapping_sub(s[3]);
    let a6 = s[1].wrapping_add(s[7]);
    let a7 = s[1].wrapping_sub(s[7]);
    let b0 = a4.wrapping_add(a6);
    let b1 = m(3784, a5.wrapping_add(a7));
    let b2 = m(-5352, a5).wrapping_sub(b0).wrapping_add(b1);
    let b3 = m(2896, a6.wrapping_sub(a4)).wrapping_sub(b2);
    let b4 = m(2217, a7).wrapping_add(b3).wrapping_sub(b1);
    [
        a0.wrapping_add(a2).wrapping_add(b0),
        a1.wrapping_add(a3).wrapping_sub(a2).wrapping_add(b2),
        a1.wrapping_sub(a3).wrapping_add(a2).wrapping_add(b3),
        a0.wrapping_sub(a2).wrapping_sub(b4),
        a0.wrapping_sub(a2).wrapping_add(b4),
        a1.wrapping_sub(a3).wrapping_add(a2).wrapping_sub(b3),
        a1.wrapping_add(a3).wrapping_sub(a2).wrapping_sub(b2),
        a0.wrapping_add(a2).wrapping_sub(b0),
    ]
}

/// The inverse transform of a dequantised block (natural order): columns, then rows; each
/// output the low byte of ⌊(o + 127) / 256⌋.
pub fn inverse_transform(c: &[i32; 64]) -> [u8; 64] {
    let mut t = [0i32; 64];
    for col in 0..8 {
        let s: [i32; 8] = std::array::from_fn(|r| c[r * 8 + col]);
        let o = if s[1..].iter().all(|&v| v == 0) { [s[0]; 8] } else { idct8(s) };
        for r in 0..8 {
            t[r * 8 + col] = o[r];
        }
    }
    let mut out = [0u8; 64];
    for row in 0..8 {
        let s: [i32; 8] = std::array::from_fn(|k| t[row * 8 + k]);
        let o = idct8(s);
        for k in 0..8 {
            out[row * 8 + k] = (o[k].wrapping_add(127) >> 8) as u8;
        }
    }
    out
}

/// Dequantisation: the low 32 bits of c × weight, arithmetic-shifted right by 11.
pub fn dequantise(c: i32, weight: i32) -> i32 {
    c.wrapping_mul(weight) >> 11
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    A,
    B,
    C,
    S,
}

/// The coefficient or residue list walk (build guide Coefficients and Residue blocks). `value`
/// reads one coded value at the current level and returns it; `event` is told of each value
/// and returns false to end the walk at once (the residue's event counter).
struct Walk {
    list: Vec<(usize, Kind)>,
}

impl Walk {
    fn empty(e: (usize, Kind)) -> bool {
        e == (0, Kind::A)
    }

    /// One level of the walk. Returns false when `coded` asked to stop.
    fn level(&mut self, b: &mut Bits, coded: &mut dyn FnMut(&mut Bits, usize) -> bool) -> bool {
        let mut i = 0;
        while i < self.list.len() {
            let (idx, kind) = self.list[i];
            if Self::empty((idx, kind)) || !b.bit() {
                i += 1;
                continue;
            }
            match kind {
                Kind::A => {
                    self.list[i] = (idx + 4, Kind::B);
                    match self.group(b, idx, &mut i, coded) {
                        true => {}
                        false => return false,
                    }
                }
                Kind::B => {
                    self.list[i] = (idx, Kind::C);
                    self.list.extend([(idx + 4, Kind::C), (idx + 8, Kind::C), (idx + 12, Kind::C)]);
                }
                Kind::C => {
                    self.list[i] = (0, Kind::A);
                    i += 1;
                    let mut at = i - 1;
                    if !self.group(b, idx, &mut at, coded) {
                        return false;
                    }
                    i = at + 1;
                }
                Kind::S => {
                    self.list[i] = (0, Kind::A);
                    i += 1;
                    if !coded(b, idx) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Code the four indices of a group: 1 puts (index, S) at the front (moving the current
    /// position on by one), 0 reads a value.
    fn group(&mut self, b: &mut Bits, idx: usize, cur: &mut usize, coded: &mut dyn FnMut(&mut Bits, usize) -> bool) -> bool {
        for j in idx..idx + 4 {
            if b.bit() {
                self.list.insert(0, (j, Kind::S));
                *cur += 1;
            } else if !coded(b, j) {
                return false;
            }
        }
        true
    }
}

/// Coefficients of a transform block (coding order); coefficient 0 is set by the caller.
fn read_coefficients(b: &mut Bits, coef: &mut [i32; 64]) {
    let mut w = Walk { list: vec![(4, Kind::A), (24, Kind::A), (44, Kind::A), (1, Kind::S), (2, Kind::S), (3, Kind::S)] };
    let m = b.read(4);
    for v in (0..m).rev() {
        let mut coded = |b: &mut Bits, k: usize| {
            let value = if v == 0 {
                1
            } else {
                (1i32 << v) + b.read(v) as i32
            };
            coef[k] = if b.bit() { -value } else { value };
            true
        };
        w.level(b, &mut coded);
    }
}

/// A residue block (type 4) at natural positions, ended after `limit` + 1 events.
fn read_residue(b: &mut Bits, perm: &[u8; 64], limit: i32) -> [i32; 64] {
    let mut res = [0i32; 64];
    let mut w = Walk { list: vec![(4, Kind::A), (24, Kind::A), (44, Kind::A), (0, Kind::C)] };
    let e = b.read(3);
    let mut counter = limit;
    let mut nonzero: Vec<usize> = Vec::new();
    for step in (0..=e).rev() {
        let s = 1i32 << step;
        let mut k = 0;
        while k < nonzero.len() {
            let p = perm[nonzero[k]] as usize;
            if b.bit() {
                res[p] += if res[p] < 0 { -s } else { s };
                counter -= 1;
                if counter < 0 {
                    return res;
                }
            }
            k += 1;
        }
        let mut stopped = false;
        let mut coded = |b: &mut Bits, k: usize| {
            let v = if b.bit() { -s } else { s };
            res[perm[k] as usize] = v;
            nonzero.push(k);
            counter -= 1;
            if counter < 0 {
                stopped = true;
                return false;
            }
            true
        };
        if !w.level(b, &mut coded) || stopped {
            return res;
        }
    }
    res
}

/// One plane's decoding state.
struct PlaneDecoder<'a> {
    t: &'a Tables,
    maps: [Map; 23],
    bundles: [Bundle; 9],
    high: u8,
}

const MAP_TYPES: usize = 0;
const MAP_NESTED: usize = 1;
const MAP_LOW: usize = 18;
const MAP_PATTERN: usize = 19;
const MAP_XOFF: usize = 20;
const MAP_YOFF: usize = 21;
const MAP_RUN: usize = 22;

impl<'a> PlaneDecoder<'a> {
    fn new(t: &'a Tables, b: &mut Bits, width: usize) -> Self {
        let maps = std::array::from_fn(|_| Map::read(b));
        let bpr = (width / 8) as u32;
        let w8 = width as u32 / 8 + 511;
        let bundles = [
            Bundle::new(w8),
            Bundle::new(width as u32 / 16 + 511),
            Bundle::new(bpr * 64 + 511),
            Bundle::new(bpr * 8 + 511),
            Bundle::new(w8),
            Bundle::new(w8),
            Bundle::new(w8),
            Bundle::new(w8),
            Bundle::new(bpr * 48 + 511),
        ];
        Self { t, maps, bundles, high: 0 }
    }

    fn sym(&self, b: &mut Bits, map: usize) -> u8 {
        self.maps[map].symbol(b, self.t)
    }

    fn refill(&mut self, b: &mut Bits) {
        for k in 0..9 {
            let Some(n) = self.bundles[k].due(b) else { continue };
            let mut v: Vec<i32> = Vec::with_capacity(n);
            match k {
                TYPES | NESTED => {
                    let map = if k == TYPES { MAP_TYPES } else { MAP_NESTED };
                    if b.bit() {
                        let x = b.read(4) as i32;
                        v.resize(n, x);
                    } else {
                        let mut last = 0;
                        while v.len() < n {
                            let s = self.sym(b, map) as i32;
                            if s < 12 {
                                v.push(s);
                                last = s;
                            } else {
                                let r = [4, 8, 12, 32][(s - 12) as usize];
                                for _ in 0..r {
                                    if v.len() < n {
                                        v.push(last);
                                    }
                                }
                            }
                        }
                    }
                }
                COLOURS => {
                    let colour = |d: &mut Self, b: &mut Bits| {
                        let h = d.maps[2 + d.high as usize].symbol(b, d.t);
                        d.high = h;
                        let l = d.sym(b, MAP_LOW);
                        (h as i32) * 16 + l as i32
                    };
                    if b.bit() {
                        let c = colour(self, b);
                        v.resize(n, c);
                    } else {
                        for _ in 0..n {
                            v.push(colour(self, b));
                        }
                    }
                }
                PATTERN => {
                    for _ in 0..n {
                        let lo = self.sym(b, MAP_PATTERN) as i32;
                        let hi = self.sym(b, MAP_PATTERN) as i32;
                        v.push(hi * 16 + lo);
                    }
                }
                XOFF | YOFF => {
                    let map = if k == XOFF { MAP_XOFF } else { MAP_YOFF };
                    let signed = |m: i32, b: &mut Bits| if m != 0 && b.bit() { -m } else { m };
                    if b.bit() {
                        let m = b.read(4) as i32;
                        let x = signed(m, b);
                        v.resize(n, x);
                    } else {
                        for _ in 0..n {
                            let m = self.sym(b, map) as i32;
                            v.push(signed(m, b));
                        }
                    }
                }
                INTRA_DC | INTER_DC => {
                    let first = if k == INTRA_DC {
                        b.read(11) as i32
                    } else {
                        let m = b.read(10) as i32;
                        if m != 0 && b.bit() { -m } else { m }
                    };
                    v.push(first);
                    let mut prev = first;
                    while v.len() < n {
                        let group = (n - v.len()).min(8);
                        let w = b.read(4);
                        for _ in 0..group {
                            if w != 0 {
                                let m = b.read(w) as i32;
                                prev += if m != 0 && b.bit() { -m } else { m };
                            }
                            v.push(prev);
                        }
                    }
                }
                _ => {
                    if b.bit() {
                        let x = b.read(4) as i32;
                        v.resize(n, x);
                    } else {
                        for _ in 0..n {
                            v.push(self.sym(b, MAP_RUN) as i32);
                        }
                    }
                }
            }
            self.bundles[k].values = v;
        }
    }

    fn take(&mut self, k: usize) -> Result<i32, String> {
        self.bundles[k].take()
    }

    fn colour(&mut self) -> Result<u8, String> {
        Ok(self.take(COLOURS)? as u8)
    }

    /// An 8 × 8 block of a pixel-domain type that needs no prediction (3, 5 intra, 6, 8, 9).
    fn own_block(&mut self, b: &mut Bits, kind: i32) -> Result<[u8; 64], String> {
        let mut out = [0u8; 64];
        match kind {
            3 => {
                let scan = self.t.scans[b.read(4) as usize];
                let mut filled = 0usize;
                while filled < 63 {
                    let n = self.take(RUN)? as usize + 1;
                    let from = filled;
                    filled += n;
                    if filled > 64 {
                        return Err("Bink run block overran its 64 pixels".into());
                    }
                    if b.bit() {
                        let c = self.colour()?;
                        for &p in &scan[from..filled] {
                            out[p as usize] = c;
                        }
                    } else {
                        for &p in &scan[from..filled] {
                            out[p as usize] = self.colour()?;
                        }
                    }
                }
                if filled == 63 {
                    out[scan[63] as usize] = self.colour()?;
                }
            }
            5 => {
                let mut coef = [0i32; 64];
                coef[0] = self.take(INTRA_DC)?;
                read_coefficients(b, &mut coef);
                let q = b.read(4) as usize;
                let mut nat = [0i32; 64];
                for k in 0..64 {
                    let p = self.t.perm[k] as usize;
                    nat[p] = dequantise(coef[k] as i16 as i32, self.t.intra[q][p]);
                }
                out = inverse_transform(&nat);
            }
            6 => out = [self.colour()?; 64],
            8 => {
                let (a, c) = (self.colour()?, self.colour()?);
                for r in 0..8 {
                    let row = self.take(PATTERN)? as u8;
                    for k in 0..8 {
                        out[r * 8 + k] = if row >> k & 1 == 1 { c } else { a };
                    }
                }
            }
            9 => {
                for p in &mut out {
                    *p = self.colour()?;
                }
            }
            _ => return Err(format!("Bink block type {kind} cannot stand alone")),
        }
        Ok(out)
    }

    /// Decode the plane into `cur` from `prev`.
    fn decode(&mut self, b: &mut Bits, cur: &mut Plane, prev: &Plane) -> Result<(), String> {
        let (bw, bh) = (cur.width / 8, cur.height / 8);
        for by in 0..bh {
            self.refill(b);
            let mut bx = 0;
            while bx < bw {
                let kind = self.take(TYPES)?;
                let (x, y) = (bx * 8, by * 8);
                match kind {
                    0 => cur.put(x, y, &prev.block(x as isize, y as isize)),
                    1 => {
                        if by % 2 == 0 {
                            let nested = self.take(NESTED)?;
                            if !matches!(nested, 3 | 5 | 6 | 8 | 9) {
                                return Err(format!("Bink nested type {nested}"));
                            }
                            let small = self.own_block(b, nested)?;
                            for r in 0..16 {
                                for c in 0..16 {
                                    let (px, py) = (x + c, y + r);
                                    if px < cur.width && py < cur.height {
                                        cur.data[py * cur.width + px] = small[(r / 2) * 8 + c / 2];
                                    }
                                }
                            }
                        }
                        bx += 1;
                    }
                    2 | 4 | 7 => {
                        let dx = self.take(XOFF)? as isize;
                        let dy = self.take(YOFF)? as isize;
                        let pred = prev.block(x as isize + dx, y as isize + dy);
                        let out = match kind {
                            2 => pred,
                            4 => {
                                let limit = b.read(7) as i32;
                                let res = read_residue(b, &self.t.perm, limit);
                                std::array::from_fn(|k| pred[k].wrapping_add(res[k] as u8))
                            }
                            _ => {
                                let mut coef = [0i32; 64];
                                coef[0] = self.take(INTER_DC)?;
                                read_coefficients(b, &mut coef);
                                let q = b.read(4) as usize;
                                let mut nat = [0i32; 64];
                                for k in 0..64 {
                                    let p = self.t.perm[k] as usize;
                                    nat[p] = dequantise(coef[k] as i16 as i32, self.t.inter[q][p]);
                                }
                                let add = inverse_transform(&nat);
                                std::array::from_fn(|k| pred[k].wrapping_add(add[k]))
                            }
                        };
                        cur.put(x, y, &out);
                    }
                    3 | 5 | 6 | 8 | 9 => {
                        let out = self.own_block(b, kind)?;
                        cur.put(x, y, &out);
                    }
                    _ => {}
                }
                bx += 1;
            }
            if b.overrun() {
                return Err(format!("Bink plane overran its data in block row {by}"));
            }
        }
        Ok(())
    }
}

/// The movie: its header, the frame index and the decoding state.
pub struct Movie {
    pub header: Header,
    /// Frame starts (masked), with the file length as the last entry.
    pub offsets: Vec<u32>,
    pub keyframes: Vec<bool>,
    pub picture: Picture,
    audio: Option<AudioState>,
    /// Audio packets decoded, and those whose blocks ended exactly at the packet end.
    pub audio_packets: (usize, usize),
}

/// One decoded frame's audio and the plane ends found (for checks).
pub struct Frame {
    pub samples: Vec<i16>,
    /// Byte ends of Y, U and V within the frame (V's should be the frame length).
    pub plane_ends: [usize; 3],
    pub u_start: usize,
}

impl Movie {
    /// Open a movie from its whole file.
    pub fn open(file: &[u8]) -> Result<Self, String> {
        let header = Header::parse(file)?;
        let tracks = header.audio_tracks as usize;
        let index_at = 0x2C + 12 * tracks;
        let n = header.frames as usize + 1;
        let mut offsets = Vec::with_capacity(n);
        let mut keyframes = Vec::with_capacity(n);
        for k in 0..n {
            let o = index_at + 4 * k;
            let w = u32::from_le_bytes(file.get(o..o + 4).ok_or("Bink frame index is short")?.try_into().unwrap());
            offsets.push(w & !1);
            keyframes.push(w & 1 == 1);
        }
        let (w, h) = (header.width as usize, header.height as usize);
        let audio = (tracks > 0).then(|| AudioState::new(header.audio_rate as u32, header.audio_flags & 0x2000 != 0));
        Ok(Self { header, offsets, keyframes, picture: Picture::new(w, h), audio, audio_packets: (0, 0) })
    }

    pub fn frames(&self) -> usize {
        self.header.frames as usize
    }

    /// Decode frame `i` (frames must be decoded in order): the picture is replaced and the
    /// frame's audio returned.
    pub fn decode(&mut self, file: &[u8], i: usize, t: &Tables) -> Result<Frame, String> {
        let (s, e) = (self.offsets[i] as usize, self.offsets[i + 1] as usize);
        let frame = file.get(s..e).ok_or("Bink frame is outside the file")?;
        let u32_at = |o: usize| -> Result<usize, String> { Ok(u32::from_le_bytes(frame.get(o..o + 4).ok_or("Bink frame is short")?.try_into().unwrap()) as usize) };
        let mut cursor = 0;
        let mut samples = Vec::new();
        for _ in 0..self.header.audio_tracks {
            let d = u32_at(cursor)?;
            if d != 0 {
                let want = u32_at(cursor + 4)?;
                let packet = frame.get(cursor + 8..cursor + 4 + d).ok_or("Bink audio packet is short")?;
                if let Some(a) = self.audio.as_mut() {
                    let (pcm, exact) = a.packet(packet, want, t)?;
                    samples.extend(pcm);
                    self.audio_packets.0 += 1;
                    self.audio_packets.1 += usize::from(exact);
                }
            }
            cursor += 4 + d;
        }
        let wdisp = u32_at(cursor)?;
        let y_start = cursor + 4;
        let u_start = cursor + wdisp;
        let prev = self.picture.clone();
        let mut cur = Picture::new(prev.planes[0].width, prev.planes[0].height);
        let mut ends = [0usize; 3];
        // Y.
        let mut b = Bits::new(frame.get(y_start..).ok_or("Bink Y plane is outside the frame")?);
        PlaneDecoder::new(t, &mut b, cur.planes[0].width).decode(&mut b, &mut cur.planes[0], &prev.planes[0])?;
        ends[0] = y_start + b.word_end();
        // U (Cr), then V (Cb) from the word after U's last bit.
        let mut b = Bits::new(frame.get(u_start..).ok_or("Bink U plane is outside the frame")?);
        PlaneDecoder::new(t, &mut b, cur.planes[1].width).decode(&mut b, &mut cur.planes[1], &prev.planes[1])?;
        ends[1] = u_start + b.word_end();
        let v_start = ends[1];
        let mut b = Bits::new(frame.get(v_start..).ok_or("Bink V plane is outside the frame")?);
        PlaneDecoder::new(t, &mut b, cur.planes[2].width).decode(&mut b, &mut cur.planes[2], &prev.planes[2])?;
        ends[2] = v_start + b.word_end();
        self.picture = cur;
        Ok(Frame { samples, plane_ends: ends, u_start })
    }

    /// The audio's output rate and channel count, when the movie has a track.
    pub fn audio_format(&self) -> Option<(u32, u16)> {
        self.audio.as_ref().map(|a| (a.rate, a.channels as u16))
    }
}

/// The revision-i audio decoder state (build guide Audio).
struct AudioState {
    rate: u32,
    channels: usize,
    n: usize,
    bounds: Vec<usize>,
    scale: f32,
    prev: Vec<i16>,
    first: bool,
    /// The cosine table for the transform, n × n.
    cos: Vec<f32>,
}

impl AudioState {
    fn new(rate: u32, stereo: bool) -> Self {
        let channels = if stereo { 2 } else { 1 };
        let n = if rate >= 44_100 { 2048 } else if rate >= 22_050 { 1024 } else { 512 };
        Self { rate, channels, n, bounds: Vec::new(), scale: 2.0 / (n as f32).sqrt(), prev: Vec::new(), first: true, cos: Vec::new() }
    }

    fn setup(&mut self, t: &Tables) {
        if !self.bounds.is_empty() {
            return;
        }
        let half = self.n / 2;
        let r2 = (self.rate + 1) / 2;
        let nb = if t.edges[0] >= r2 { 0 } else { 1 + t.edges[1..].iter().take(24).take_while(|&&e| e < r2).count() };
        self.bounds = (0..nb).map(|k| ((t.edges[k] as u64 * half as u64 / r2 as u64) as usize).max(1)).collect();
        self.bounds.push(half);
        let n = self.n;
        self.cos = (0..n * n).map(|i| {
            let (x, k) = (i / n, i % n);
            (std::f64::consts::PI * k as f64 * (2 * x + 1) as f64 / (2 * n) as f64).cos() as f32
        }).collect();
    }

    fn float29(b: &mut Bits) -> f32 {
        let k = b.read(5) as i32;
        let m = b.read(23) as f32;
        let v = m * 2f32.powi(k - 23);
        if b.bit() { -v } else { v }
    }

    /// Decode blocks until `want` bytes of samples are made.
    /// Also returns whether the blocks ended exactly at the packet end.
    fn packet(&mut self, data: &[u8], want: usize, t: &Tables) -> Result<(Vec<i16>, bool), String> {
        self.setup(t);
        let mut b = Bits::new(data);
        let mut out = Vec::with_capacity(want / 2);
        let (n, ch) = (self.n, self.channels);
        let v = n * ch / 16;
        while out.len() * 2 < want {
            let start = b.pos;
            b.read(2);
            let mut pcm = vec![0i16; n * ch];
            let nb = self.bounds.len() - 1;
            for c in 0..ch {
                let mut coef = vec![0f32; n];
                coef[0] = Self::float29(&mut b);
                coef[1] = Self::float29(&mut b);
                let mult: Vec<f32> = (0..nb).map(|_| t.multipliers[b.read(8) as usize]).collect();
                let mut k = 2usize;
                let mut band = 0usize;
                let mut m = mult.first().copied().unwrap_or(1.0);
                while k < n {
                    let len = if b.bit() { 8 * t.segments[b.read(4) as usize] as usize } else { 8 };
                    let end = (k + len).min(n);
                    let w = b.read(4);
                    if w == 0 {
                        // A zero segment: the multiplier jumps to the last band whose boundary is
                        // below the segment end.
                        while band + 1 < nb && 2 * self.bounds[band + 1] < end {
                            band += 1;
                        }
                        m = mult[band];
                        k = end;
                        continue;
                    }
                    while k < end {
                        while band + 1 < nb && k >= 2 * self.bounds[band + 1] {
                            band += 1;
                            m = mult[band];
                        }
                        let mag = b.read(w);
                        if mag != 0 {
                            let s = if b.bit() { -(mag as f32) } else { mag as f32 };
                            coef[k] = s * m;
                        }
                        k += 1;
                    }
                }
                for x in 0..n {
                    let row = &self.cos[x * n..x * n + n];
                    let mut sum = coef[0];
                    for kk in 1..n {
                        sum += coef[kk] * row[kk];
                    }
                    let s = (sum * self.scale).trunc().clamp(-32768.0, 32767.0) as i16;
                    pcm[x * ch + c] = s;
                }
            }
            let used = b.pos - start;
            b.pos = start + used.div_ceil(32) * 32;
            if !self.first {
                for i in 0..v {
                    pcm[i] = ((pcm[i] as i32 * i as i32 + self.prev[i] as i32 * (v - i) as i32) / v as i32) as i16;
                }
            }
            self.first = false;
            self.prev = pcm[n * ch - v..].to_vec();
            out.extend_from_slice(&pcm[..n * ch - v]);
            if b.overrun() {
                return Err("Bink audio packet overran".into());
            }
        }
        Ok((out, b.pos == data.len() * 8))
    }
}

/// The movie library's 48 kHz mixer stage (SPEC-0008 Movie audio internals): decoded PCM of
/// one track becomes 48 kHz output. A 48 kHz track passes through unchanged; another rate takes
/// output frame k from source frame floor(k x rate / 48,000), repeating or skipping frames (no
/// interpolation), with the position carried across calls. A single track's signed-16 samples
/// need no saturation (the stage saturates only sums of tracks).
#[derive(Clone, Debug)]
pub struct Mixer {
    rate: u64,
    channels: usize,
    /// Output frames produced so far and source frames dropped from `pending`.
    out: u64,
    dropped: u64,
    pending: Vec<i16>,
}

/// The mixer's output rate.
pub const MIX_RATE: u32 = 48_000;

impl Mixer {
    pub fn new(rate: u32, channels: u16) -> Self {
        Self { rate: rate.max(1) as u64, channels: channels.max(1) as usize, out: 0, dropped: 0, pending: Vec::new() }
    }

    /// The 48 kHz frames for this block of interleaved source frames.
    pub fn convert(&mut self, input: &[i16]) -> Vec<i16> {
        if self.rate == MIX_RATE as u64 {
            return input.to_vec();
        }
        let c = self.channels;
        self.pending.extend_from_slice(input);
        let have = (self.pending.len() / c) as u64;
        let mut output = Vec::new();
        loop {
            let src = self.out * self.rate / MIX_RATE as u64;
            let Some(at) = src.checked_sub(self.dropped).filter(|&at| at < have) else { break };
            let at = at as usize * c;
            output.extend_from_slice(&self.pending[at..at + c]);
            self.out += 1;
        }
        // Keep the frames a later output frame can still select.
        let next = self.out * self.rate / MIX_RATE as u64;
        let drop = next.saturating_sub(self.dropped).min(have);
        self.pending.drain(..drop as usize * c);
        self.dropped += drop;
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixer_rate_conversion() {
        // 48 kHz passes through.
        let mut m = Mixer::new(48_000, 2);
        assert_eq!(m.convert(&[1, 2, 3, 4]), vec![1, 2, 3, 4]);
        // 44.1 kHz mono: 441 source frames give 480 output frames, frame k from floor(k 441 /
        // 480), the same whether the block is fed whole or in pieces.
        let src: Vec<i16> = (0..441).collect();
        let whole = Mixer::new(44_100, 1).convert(&src);
        assert_eq!(whole.len(), 480);
        assert!(whole.iter().enumerate().all(|(k, &v)| v as usize == k * 441 / 480));
        let mut m = Mixer::new(44_100, 1);
        let mut pieces = Vec::new();
        for chunk in src.chunks(37) {
            pieces.extend(m.convert(chunk));
        }
        assert_eq!(pieces, whole);
        // Stereo frames stay together.
        let mut m = Mixer::new(44_100, 2);
        let out = m.convert(&[10, -10, 20, -20]);
        assert!(out.chunks(2).all(|f| f[0] == -f[1]));
    }

    #[test]
    fn dequantise_vectors() {
        // SPEC-0008 8x8 inverse transform checks: 2 × 65,536 >> 11 = 64; −96,582 >> 11 = −48.
        assert_eq!(dequantise(2, 65_536), 64);
        assert_eq!(dequantise(-1, 96_582), -48);
    }

    #[test]
    fn transform_of_a_flat_block() {
        // A DC-only block: the column pass passes s0 down column 0, the row formulas spread it
        // to all eight outputs, and each pixel is (s0 + 127) >> 8.
        let mut c = [0i32; 64];
        c[0] = 2048;
        let out = inverse_transform(&c);
        assert!(out.iter().all(|&v| v == out[0]));
        assert_eq!(out[0], ((2048 + 127) >> 8) as u8);
    }

    #[test]
    fn merge_map_forms() {
        // SPEC-0008 Code-map construction forms, synthetic: identity pairs (eight 0 bits), then
        // all-one merge bits.
        fn bits(v: &[u8]) -> Vec<u8> {
            let mut out = vec![0u8; v.len().div_ceil(8)];
            for (i, &b) in v.iter().enumerate() {
                out[i / 8] |= b << (i % 8);
            }
            out
        }
        let run = |depth: u8, ones: usize| {
            // family 1 (4 bits), merge (0), depth (2 bits), pass 1: eight 0s, then ones.
            let mut v = vec![1, 0, 0, 0, 0, depth & 1, depth >> 1 & 1];
            v.extend([0; 8]);
            v.extend(std::iter::repeat_n(1, ones));
            let data = bits(&v);
            Map::read(&mut Bits::new(&data)).symbols
        };
        assert_eq!(run(1, 64), [2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13]);
        assert_eq!(run(2, 64), [6, 7, 4, 5, 2, 3, 0, 1, 14, 15, 12, 13, 10, 11, 8, 9]);
        assert_eq!(run(3, 64), [14, 15, 12, 13, 10, 11, 8, 9, 6, 7, 4, 5, 2, 3, 0, 1]);
    }

    #[test]
    fn colour_check_values() {
        // Limited-range luma 16 with neutral chroma gives 0; 235 gives 255.
        let mut p = Picture::new(16, 16);
        p.planes[1].data.fill(128);
        p.planes[2].data.fill(128);
        p.planes[0].data[0] = 16;
        p.planes[0].data[1] = 235;
        let rgba = p.rgba();
        assert_eq!(&rgba[0..3], &[0, 0, 0]);
        assert_eq!(&rgba[4..7], &[255, 255, 255]);
    }
}
