//! Skeletons, animation clips and posing (SPEC-0005). Clips store, per animated bone, a
//! rotation quaternion and a translation as 16-bit fixed point at every frame; some entries are
//! one LZO1X block that expands to the same layout.
use crate::lzo;

/// Sample scale: the single-precision constant with bit pattern 0x3A800EAE (about 1/1024).
pub const SAMPLE_SCALE: f32 = f32::from_bits(0x3A80_0EAE);
const ENTRY_HEADER: usize = 0x3C;
/// Bounds that the disc stays well inside (74 bones, 146 frames, 51,000 entries per set).
const MAX_BONES: usize = 128;
const MAX_FRAMES: usize = 4096;
const MAX_ENTRIES: usize = 1 << 16;

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn s16_at(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

/// A skeleton definition (chunk types 22–24, or a headerless family header): per joint, its
/// parent (None for the root) and mirror partner.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonDef {
    pub parents: Vec<Option<u32>>,
    pub partners: Vec<u32>,
    /// The per-bone translation scale sets (SPEC-0005 Per-bone translation scale): K floats per
    /// joint (header word 0x14), joint i's at the end of the records (0x2C) plus its record's
    /// third field. Empty when K is 0.
    pub scales: Vec<Vec<f32>>,
}

impl SkeletonDef {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        let n = u32_at(b, 0).ok_or("Truncated skeleton definition")? as usize;
        let records = u32_at(b, 0x18).ok_or("Truncated skeleton definition")? as usize;
        if n == 0 || n > MAX_BONES {
            return Err(format!("{n} joints is outside 1..={MAX_BONES}"));
        }
        let k = u32_at(b, 0x14).unwrap_or(0) as usize;
        let end = u32_at(b, 0x2C).unwrap_or(0) as usize;
        let mut def = SkeletonDef {
            parents: Vec::with_capacity(n),
            partners: Vec::with_capacity(n),
            scales: Vec::new(),
        };
        for i in 0..n {
            let at = records + 12 * i;
            let parent = u32_at(b, at).ok_or("Skeleton record past the end")?;
            def.parents.push((parent != 0xFF).then_some(parent));
            def.partners.push(u32_at(b, at + 4).ok_or("Skeleton record past the end")?);
            if (1..=64).contains(&k) {
                let block = end + u32_at(b, at + 8).unwrap_or(0) as usize;
                def.scales.push((0..k).map(|j| u32_at(b, block + 4 * j).map_or(1.0, f32::from_bits)).collect());
            }
        }
        Ok(def)
    }

    /// The translation factor of joint i for scale selector `sel` (1-based; 0 or negative, or a
    /// set the joint lacks, is no scaling).
    pub fn translation_scale(&self, i: usize, sel: i16) -> f32 {
        usize::try_from(sel)
            .ok()
            .and_then(|s| s.checked_sub(1))
            .and_then(|s| self.scales.get(i)?.get(s).copied())
            .unwrap_or(1.0)
    }

    pub fn len(&self) -> usize {
        self.parents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parents.is_empty()
    }
}

/// The mesh's bone table (a flat type-18 chunk): parents, rest local translations and rest
/// positions in model space. Skin bones index this table.
#[derive(Clone, Debug, PartialEq)]
pub struct BoneTable {
    pub parents: Vec<Option<usize>>,
    pub rest_local: Vec<[f32; 3]>,
    pub rest_position: Vec<[f32; 3]>,
    /// Hurt capsules (SPEC-0005 bone record +0x30, +0x40, +0x44): bone-local base point,
    /// length (above 0: a capsule) and radius.
    pub hurt: Vec<([f32; 3], f32, f32)>,
    /// Pose entries (attach points; 9 of 0x30 bytes): bone index (None: none), angles (x, y,
    /// z, radians) and offset.
    pub attach: Vec<(Option<usize>, [f32; 3], [f32; 3])>,
}

impl BoneTable {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        let records = u32_at(b, 0x20).ok_or("Truncated bone table")? as usize;
        let n = u16_at(b, 0x38).ok_or("Truncated bone table")? as usize;
        if n == 0 || n > MAX_BONES {
            return Err(format!("{n} bones is outside 1..={MAX_BONES}"));
        }
        let mut t = BoneTable {
            parents: Vec::new(),
            rest_local: Vec::new(),
            rest_position: Vec::new(),
            hurt: Vec::new(),
            attach: Vec::new(),
        };
        for i in 0..n {
            let r = records + 0x50 * i;
            let parent = u32_at(b, r + 0x08).ok_or("Bone record past the end")?;
            t.parents.push((parent != u32::MAX).then_some(parent as usize / 0x50));
            let v = |at: usize| -> Option<[f32; 3]> {
                Some([f32_at(b, at)?, f32_at(b, at + 4)?, f32_at(b, at + 8)?])
            };
            t.rest_local.push(v(r + 0x10).ok_or("Bone record past the end")?);
            t.rest_position.push(v(r + 0x20).ok_or("Bone record past the end")?);
            t.hurt.push((
                v(r + 0x30).ok_or("Bone record past the end")?,
                f32_at(b, r + 0x40).unwrap_or(0.0),
                f32_at(b, r + 0x44).unwrap_or(0.0),
            ));
        }
        let entries = u32_at(b, 0x24).unwrap_or(0) as usize;
        let count = u16_at(b, 0x3A).unwrap_or(0).min(16) as usize;
        for k in 0..count {
            let e = entries + 0x30 * k;
            let v = |at: usize| -> Option<[f32; 3]> { Some([f32_at(b, at)?, f32_at(b, at + 4)?, f32_at(b, at + 8)?]) };
            let (Some(bone), Some(angles), Some(offset)) = (u32_at(b, e + 4), v(e + 0x10), v(e + 0x20)) else { break };
            let bone = (bone as i32 >= 0 && (bone as usize) < n).then_some(bone as usize);
            t.attach.push((bone, angles, offset));
        }
        Ok(t)
    }

    pub fn len(&self) -> usize {
        self.parents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parents.is_empty()
    }
}

/// One decoded clip: per frame, per animated bone, a rotation (x, y, z, w) and translation.
#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub frames: usize,
    pub looping: bool,
    /// Speed byte S: frames per tick at play rate R is R × 0.5 / S (R × 0.5 when S is 0).
    pub speed_byte: u8,
    pub fade_in: u8,
    pub compressed: bool,
    pub mode: u8,
    /// Mask bit numbers of the animated bones, ascending.
    pub bones: Vec<u32>,
    /// `frames × bones.len()` values, frame-major.
    pub rotations: Vec<[f32; 4]>,
    pub translations: Vec<[f32; 3]>,
    /// Root record per frame (x, y, z scaled; table A).
    pub root: Vec<[f32; 3]>,
    /// Table-B frame bits (event windows) per frame.
    pub frame_bits: Vec<u32>,
}

impl Clip {
    pub fn rotation(&self, frame: usize, bone: usize) -> [f32; 4] {
        self.rotations[frame * self.bones.len() + bone]
    }

    pub fn translation(&self, frame: usize, bone: usize) -> [f32; 3] {
        self.translations[frame * self.bones.len() + bone]
    }
}

/// Header fields of an entry, with its offsets relative to the buffer they are read from.
struct EntryHeader {
    mask: u64,
    nb: usize,
    frames: usize,
    looping: bool,
    fade_in: u8,
    mode: u8,
    speed_byte: u8,
    compressed: bool,
    table_a: usize,
    table_b: usize,
    next: usize,
}

fn entry_header(b: &[u8], at: usize) -> Result<EntryHeader, String> {
    let h = b
        .get(at..at + ENTRY_HEADER)
        .ok_or("Clip entry header past the end")?;
    let frames = u32_at(h, 0x14).unwrap() as usize;
    let mask = u64_at(h, 0).unwrap();
    if frames > MAX_FRAMES {
        return Err(format!("{frames} frames is implausible"));
    }
    Ok(EntryHeader {
        mask,
        nb: mask.count_ones() as usize,
        frames,
        looping: h[0x27] != 0,
        fade_in: h[0x24],
        mode: h[0x29],
        speed_byte: h[0x2A],
        compressed: h[0x2B] != 0,
        table_a: u32_at(h, 0x30).unwrap() as usize,
        table_b: u32_at(h, 0x34).unwrap() as usize,
        next: u32_at(h, 0x38).unwrap() as usize,
    })
}

/// Walk one entry's 7 × nb channels (mode 2) starting at `pos`.
fn channels(b: &[u8], mut pos: usize, h: &EntryHeader) -> Result<Vec<Vec<f32>>, String> {
    let mut out = Vec::with_capacity(7 * h.nb);
    for _ in 0..7 * h.nb {
        let flag = u32_at(b, pos).ok_or("Channel stream past the end")?;
        if flag == 0 {
            let v = s16_at(b, pos + 4).ok_or("Channel stream past the end")? as f32 * SAMPLE_SCALE;
            out.push(vec![v; h.frames]);
            pos += 12;
        } else if flag & 0xFFFF == 1 {
            let samples = (0..h.frames)
                .map(|f| s16_at(b, pos + 4 + 2 * f).map(|s| s as f32 * SAMPLE_SCALE))
                .collect::<Option<Vec<_>>>()
                .ok_or("Animated channel past the end")?;
            out.push(samples);
            pos += 4 + 2 * ((h.frames + 3) & !3);
        } else {
            return Err(format!("Unknown channel flag {flag:#x}"));
        }
    }
    Ok(out)
}

fn assemble(
    h: &EntryHeader,
    b: &[u8],
    table_a: usize,
    table_b: usize,
) -> Result<Clip, String> {
    let bones: Vec<u32> = (0..64).filter(|i| h.mask >> i & 1 == 1).collect();
    let n = h.frames * h.nb;
    let mut clip = Clip {
        frames: h.frames,
        looping: h.looping,
        speed_byte: h.speed_byte,
        fade_in: h.fade_in,
        compressed: h.compressed,
        mode: h.mode,
        bones,
        rotations: Vec::with_capacity(n),
        translations: Vec::with_capacity(n),
        root: Vec::with_capacity(h.frames),
        frame_bits: (0..h.frames)
            .map(|f| u32_at(b, table_b + 4 * f).unwrap_or(0))
            .collect(),
    };
    match h.mode {
        2 => {
            for f in 0..h.frames {
                let r = |k: usize| s16_at(b, table_a + 8 * f + 2 * k).unwrap_or(0) as f32 * SAMPLE_SCALE;
                clip.root.push([r(0), r(1), r(2)]);
            }
            let ch = channels(b, table_a + 8 * h.frames, h)?;
            for f in 0..h.frames {
                for bone in 0..h.nb {
                    let c = |k: usize| ch[7 * bone + k][f];
                    clip.rotations.push([c(0), c(1), c(2), c(3)]);
                    clip.translations.push([c(4), c(5), c(6)]);
                }
            }
        }
        0 => {
            let stride = 16 * h.nb + 8;
            for f in 0..h.frames {
                let frame = table_a + f * stride;
                let s = |at: usize| s16_at(b, at).map(|v| v as f32 / 4096.0).ok_or("Mode-0 frame past the end");
                for bone in 0..h.nb {
                    let at = frame + 16 * bone;
                    clip.rotations.push([s(at)?, s(at + 2)?, s(at + 4)?, s(at + 6)?]);
                    clip.translations.push([s(at + 8)?, s(at + 10)?, s(at + 12)?]);
                }
                let root = frame + 16 * h.nb;
                let r = |k: usize| s16_at(b, root + 2 * k).unwrap_or(0) as f32 * SAMPLE_SCALE;
                clip.root.push([r(0), r(1), r(2)]);
            }
        }
        other => return Err(format!("Evaluation mode {other} is not supported")),
    }
    Ok(clip)
}

/// Decode the entry at `at`, whose offsets are relative to `b` (a chunk payload, or a whole
/// headerless bank).
pub fn decode_entry(b: &[u8], at: usize) -> Result<Clip, String> {
    let h = entry_header(b, at)?;
    if h.nb > MAX_BONES {
        return Err("Too many animated bones".into());
    }
    if !h.compressed {
        return assemble(&h, b, h.table_a, h.table_b);
    }
    let block = b.get(h.table_b..).ok_or("Compressed block past the end")?;
    let u = u32_at(block, 0).ok_or("Truncated compressed block")? as usize;
    let c = u32_at(block, 4).ok_or("Truncated compressed block")? as usize;
    if u > 1 << 24 {
        return Err(format!("Expanded size {u} is implausible"));
    }
    let data = block.get(8..8 + c).ok_or("Compressed block past the end")?;
    let (expanded, used) = lzo::decompress(data, u)?;
    if used != c {
        return Err(format!("LZO1X block used {used} of {c} bytes"));
    }
    // Expanded layout: table B at 0, table A at (table A - table B), channels after table A.
    let a = h
        .table_a
        .checked_sub(h.table_b)
        .ok_or("Table A before table B in a compressed entry")?;
    assemble(&h, &expanded, a, 0)
}

/// Entry offsets of an animation set (types 25–27) or headerless bank: `E` entries chained by
/// their next links, starting at the offset at +0x24.
pub fn entries(b: &[u8]) -> Result<Vec<usize>, String> {
    let count = u32_at(b, 0x0C).ok_or("Truncated animation set")? as usize;
    let mut at = u32_at(b, 0x24).ok_or("Truncated animation set")? as usize;
    if count > MAX_ENTRIES {
        return Err(format!("{count} clip entries is implausible"));
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        out.push(at);
        if i + 1 < count {
            at = entry_header(b, at)?.next;
        }
    }
    Ok(out)
}

/// A quaternion (x, y, z, w) to a column-major 3 × 3 rotation, without normalising: the
/// original applies the standard formula to the raw, possibly non-unit values.
pub fn quat_to_matrix(q: [f32; 4]) -> [[f32; 3]; 3] {
    let [x, y, z, w] = q;
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w)],
        [2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w)],
        [2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

/// Blend two keys: translations linearly; rotations componentwise after flipping the second
/// when their dot product is negative (no spherical blend, no normalising).
pub fn blend_key(qa: [f32; 4], qb: [f32; 4], ta: [f32; 3], tb: [f32; 3], t: f32) -> ([f32; 4], [f32; 3]) {
    let dot: f32 = (0..4).map(|i| qa[i] * qb[i]).sum();
    let qb = if dot < 0.0 { qb.map(|v| -v) } else { qb };
    (
        std::array::from_fn(|i| qa[i] * (1.0 - t) + qb[i] * t),
        std::array::from_fn(|i| ta[i] * (1.0 - t) + tb[i] * t),
    )
}

/// A column-major 4 × 4 affine transform as rotation columns and translation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub cols: [[f32; 3]; 3],
    pub t: [f32; 3],
}

impl Affine {
    pub const IDENTITY: Self = Self {
        cols: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        t: [0.0; 3],
    };

    pub fn apply(&self, p: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|r| {
            self.cols[0][r] * p[0] + self.cols[1][r] * p[1] + self.cols[2][r] * p[2] + self.t[r]
        })
    }

    pub fn rotate(&self, v: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|r| self.cols[0][r] * v[0] + self.cols[1][r] * v[1] + self.cols[2][r] * v[2])
    }

    pub fn mul(&self, o: &Affine) -> Affine {
        Affine {
            cols: o.cols.map(|c| self.rotate(c)),
            t: self.apply(o.t),
        }
    }
}

/// A sampled joint value: rotation and translation.
pub type Joint = ([f32; 4], [f32; 3]);

/// Combine a sample into a joint (SPEC-0005, Combining a sample into the pose): with full
/// weight the sample replaces the joint; otherwise output ← (1 − φ) output + φ sample,
/// componentwise, after flipping the sample's rotation toward the output's. Nothing is
/// normalised. A joint with no value yet takes the sample.
pub fn combine(joint: &mut Option<Joint>, sample: Joint, replace: bool, phi: f32) {
    *joint = Some(match (*joint, replace) {
        (Some((q, t)), false) => {
            let dot: f32 = (0..4).map(|i| q[i] * sample.0[i]).sum();
            let qs = if dot < 0.0 { sample.0.map(|v| -v) } else { sample.0 };
            (
                std::array::from_fn(|i| (1.0 - phi) * q[i] + phi * qs[i]),
                std::array::from_fn(|i| (1.0 - phi) * t[i] + phi * sample.1[i]),
            )
        }
        _ => sample,
    });
}

/// Sample a clip at key `k`, next key `k2` and fraction `t`, and combine each animated joint
/// into the joint list: mask bit b drives element b − `base` (0 for main clips, N_main for
/// sub-part and face clips). With `mirror` (the joints' mirror partners; Mirrored clips) the
/// sample of joint i goes to joint partner(i), its rotation (x, y, z, w) as (x, −y, −z, w) and
/// its translation (x, y, z) as (−x, y, z).
#[allow(clippy::too_many_arguments)]
pub fn sample_into(
    clip: &Clip,
    k: usize,
    k2: usize,
    t: f32,
    base: u32,
    joints: &mut [Option<Joint>],
    replace: bool,
    phi: f32,
    scale: &dyn Fn(usize) -> f32,
    mirror: Option<&[u32]>,
) {
    if k >= clip.frames || k2 >= clip.frames {
        return;
    }
    for (i, &bit) in clip.bones.iter().enumerate() {
        let Some(j) = bit.checked_sub(base).map(|j| j as usize).filter(|&j| j < joints.len()) else {
            continue;
        };
        // Both keys' translations are scaled before the blend (Per-bone translation scale).
        let f = scale(j);
        let (mut q, mut tr) = blend_key(
            clip.rotation(k, i),
            clip.rotation(k2, i),
            clip.translation(k, i).map(|v| v * f),
            clip.translation(k2, i).map(|v| v * f),
            t,
        );
        let mut out = j;
        if let Some(partners) = mirror {
            match partners.get(j).map(|&p| p as usize).filter(|&p| p < joints.len()) {
                Some(p) => out = p,
                None => continue,
            }
            q = [q[0], -q[1], -q[2], q[3]];
            tr = [-tr[0], tr[1], tr[2]];
        }
        combine(&mut joints[out], (q, tr), replace, phi);
    }
}

/// Word A bit 12: the entry's clip plays mirrored (Mirrored clips).
pub const MIRROR_BIT: u32 = 1 << 12;

/// One animation family of a hero (SPEC-0005, Hero animation data): its skeleton header, its
/// clip bank decoded in chain order, and word A of each action-table entry.
#[derive(Clone, Debug)]
pub struct FamilyData {
    pub def: SkeletonDef,
    pub clips: Vec<Clip>,
    pub actions: Vec<u32>,
    /// Word B of each entry: the packed sound word (SPEC-0031 Layout section 10).
    pub sounds: Vec<u32>,
}

impl FamilyData {
    /// Parse a headerless header, clip bank and action table (raw resources).
    pub fn parse(header: &[u8], bank: &[u8], table: &[u8]) -> Result<Self, String> {
        let def = SkeletonDef::parse(header)?;
        let clips = entries(bank)?
            .into_iter()
            .map(|at| decode_entry(bank, at))
            .collect::<Result<Vec<_>, _>>()?;
        let n = u32_at(table, 0).ok_or("Truncated action table")? as usize;
        let actions = (0..n.min(4096))
            .map(|i| u32_at(table, 4 + 8 * i).ok_or("Action table past the end"))
            .collect::<Result<Vec<_>, _>>()?;
        let sounds = (0..n.min(4096)).map(|i| u32_at(table, 8 + 8 * i).unwrap_or(0xFFFF_FFFF)).collect();
        Ok(Self { def, clips, actions, sounds })
    }

    /// The clip for an action id: word A bits 0–11; an id past the table or 0xFFF is a lookup
    /// miss, which gives clip 0. None when the index is not a clip of the bank.
    /// The clip the table names for the action, with no fallback: None for an id at or past
    /// the table's end or a clip field of 0xFFF (SPEC-0038 Layout section 6: such a slot phase
    /// counts as missing and is skipped).
    pub fn named_clip(&self, action: u32) -> Option<usize> {
        let index = (self.actions.get(action as usize).copied()? & 0xFFF) as usize;
        (index != 0xFFF && index < self.clips.len()).then_some(index)
    }

    pub fn clip_for(&self, action: u32) -> Option<usize> {
        let index = match self.actions.get(action as usize) {
            Some(&a) if a & 0xFFF != 0xFFF => (a & 0xFFF) as usize,
            _ => 0,
        };
        (index < self.clips.len()).then_some(index)
    }
}

/// Resource numbers of a hero's families (SPEC-0005 table): (header, bank, action table) for
/// main, sub-part and face; hero 0 is Tor, 1 is Roberta.
pub fn hero_resources(hero: usize) -> [(u32, u32, u32); 3] {
    let h = (hero.min(1) * 6) as u32;
    [(492 + h, 493 + h, 520 + h / 2), (494 + h, 495 + h, 521 + h / 2), (496 + h, 497 + h, 522 + h / 2)]
}

/// One family's joint list: its skeleton definition, the skin bone of its first joint, and the
/// sampled joints (None = rest pose).
pub struct Family<'a> {
    pub def: &'a SkeletonDef,
    pub skin_base: usize,
    pub joints: &'a [Option<Joint>],
}

/// Compose world transforms for the main family then the sub-part and face families, and return
/// one skin matrix per skin bone: S_i = W_i × translate(−rest position_i). Unanimated joints use
/// the mesh's rest local translation and no rotation. Sub-part and face parents below N_main
/// are main joints; others are elements of their own list.
pub fn skin_matrices(bones: &BoneTable, main: &Family, others: &[Family]) -> Vec<Affine> {
    let mut world = vec![Affine::IDENTITY; bones.len()];
    let n_main = main.def.len();
    let local = |skin: usize, sampled: Option<Joint>| -> Affine {
        match sampled {
            Some((q, t)) => Affine { cols: quat_to_matrix(q), t },
            None => Affine {
                cols: Affine::IDENTITY.cols,
                t: bones.rest_local.get(skin).copied().unwrap_or([0.0; 3]),
            },
        }
    };
    for (i, parent) in main.def.parents.iter().enumerate().take(bones.len()) {
        let l = local(i, main.joints.get(i).copied().flatten());
        world[i] = match parent {
            Some(p) if (*p as usize) < i => world[*p as usize].mul(&l),
            _ => l,
        };
    }
    for f in others {
        for (i, parent) in f.def.parents.iter().enumerate() {
            let skin = f.skin_base + i;
            if skin >= bones.len() {
                break;
            }
            let l = local(skin, f.joints.get(i).copied().flatten());
            let parent_world = match parent {
                Some(p) if (*p as usize) < n_main => Some(world[*p as usize]),
                Some(p) => {
                    let j = *p as usize - n_main;
                    (j < i).then(|| world[f.skin_base + j])
                }
                None => None,
            };
            world[skin] = parent_world.map_or(l, |w| w.mul(&l));
        }
    }
    world
        .iter()
        .zip(&bones.rest_position)
        .map(|(w, r)| w.mul(&Affine { cols: Affine::IDENTITY.cols, t: r.map(|v| -v) }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-joint main definition with K = 2: joint 0's set at block offset 0, joint 1's at 8.
    #[test]
    fn translation_scale_sets() {
        let mut b = vec![0u8; 0x30 + 24 + 16];
        b[0..4].copy_from_slice(&2u32.to_le_bytes());
        b[0x14..0x18].copy_from_slice(&2u32.to_le_bytes());
        b[0x18..0x1C].copy_from_slice(&0x30u32.to_le_bytes());
        b[0x2C..0x30].copy_from_slice(&0x48u32.to_le_bytes());
        b[0x30..0x34].copy_from_slice(&0xFFu32.to_le_bytes());
        b[0x3C..0x40].copy_from_slice(&0u32.to_le_bytes());
        b[0x44..0x48].copy_from_slice(&8u32.to_le_bytes());
        for (o, v) in [(0x48, 1.046f32), (0x4C, 2.0), (0x50, 0.5), (0x54, 3.0)] {
            b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        let def = SkeletonDef::parse(&b).unwrap();
        assert_eq!(def.parents, [None, Some(0)]);
        // SPEC-0005 vector: selector 1, joint 0 translation y 1.145 gives 1.145 x 1.046.
        assert!((1.145 * def.translation_scale(0, 1) - 1.198).abs() < 1e-3);
        assert_eq!(def.translation_scale(1, 2), 3.0);
        assert_eq!(def.translation_scale(1, 0), 1.0, "selector 0: no scaling");
        assert_eq!(def.translation_scale(1, -1), 1.0, "a negative selector: no scaling");
        assert_eq!(def.translation_scale(1, 3), 1.0, "a set past K: no scaling");
    }

    #[test]
    fn quaternion_matches_spec_example() {
        // (0, sin 45°, 0, cos 45°) turns 90° about +Y and maps +X to -Z.
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let m = Affine { cols: quat_to_matrix([0.0, h, 0.0, h]), t: [0.0; 3] };
        let x = m.rotate([1.0, 0.0, 0.0]);
        assert!((x[0]).abs() < 1e-6 && (x[2] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn blend_flips_opposite_quaternions_and_does_not_normalise() {
        let (q, t) = blend_key([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, -1.0], [0.0; 3], [2.0, 0.0, 0.0], 0.5);
        assert_eq!(q, [0.0, 0.0, 0.0, 1.0], "the second key is negated first");
        assert_eq!(t, [1.0, 0.0, 0.0]);
        let (q, _) = blend_key([1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0; 3], [0.0; 3], 0.5);
        assert_eq!(q, [0.5, 0.5, 0.0, 0.0], "componentwise, no normalising");
    }

    #[test]
    fn sample_scale_is_the_spec_constant() {
        assert_eq!(SAMPLE_SCALE.to_bits(), 0x3A80_0EAE);
        assert!((1024.0 * SAMPLE_SCALE - 1.000448).abs() < 1e-6);
    }

    /// A one-bone (bit 0), three-frame plain mode-2 entry: one animated channel (rotation x)
    /// and six constant channels, with root records.
    fn plain_entry() -> Vec<u8> {
        let mut b = vec![0u8; 0x30];
        b[0x0C..0x10].copy_from_slice(&1u32.to_le_bytes());
        b[0x24..0x28].copy_from_slice(&0x30u32.to_le_bytes());
        let e = b.len();
        let mut h = vec![0u8; ENTRY_HEADER];
        h[0] = 1;
        h[0x10] = 1;
        h[0x14] = 3;
        h[0x24] = 4;
        h[0x27] = 1;
        h[0x29] = 2;
        h[0x2A] = 2;
        let table_b = e + ENTRY_HEADER;
        let table_a = table_b + 12;
        h[0x30..0x34].copy_from_slice(&(table_a as u32).to_le_bytes());
        h[0x34..0x38].copy_from_slice(&(table_b as u32).to_le_bytes());
        b.extend(h);
        b.extend([1, 0, 0, 0, 0, 0, 0, 0, 0x80, 0, 0, 0]); // frame bits 1, 0, 0x80
        for f in 0..3i16 {
            for v in [f * 100, 0, 1024, 0] {
                b.extend(v.to_le_bytes());
            }
        }
        // Rotation x animated: 0, 512, 1024 (+ one padding sample).
        b.extend([1u8, 0, 0, 0]);
        for s in [0i16, 512, 1024, 0] {
            b.extend(s.to_le_bytes());
        }
        // Constant channels: y = 0, z = 0, w = 1024, tx = 0, ty = 2048, tz = 0.
        for v in [0i16, 0, 1024, 0, 2048, 0] {
            b.extend([0u8; 4]);
            b.extend(v.to_le_bytes());
            b.extend([0u8; 6]);
        }
        b
    }

    #[test]
    fn decodes_a_plain_entry() {
        let b = plain_entry();
        let list = entries(&b).unwrap();
        assert_eq!(list, [0x30]);
        let c = decode_entry(&b, 0x30).unwrap();
        assert_eq!((c.frames, c.looping, c.speed_byte, c.fade_in, c.bones.clone()), (3, true, 2, 4, vec![0]));
        assert_eq!(c.frame_bits, [1, 0, 0x80]);
        assert_eq!(c.rotation(1, 0)[0], 512.0 * SAMPLE_SCALE);
        assert_eq!(c.translation(2, 0), [0.0, 2048.0 * SAMPLE_SCALE, 0.0]);
        assert_eq!(c.root[2][0], 200.0 * SAMPLE_SCALE);
    }

    #[test]
    fn decodes_a_compressed_entry_as_its_expansion() {
        // Store the plain entry's tables and channels as a literal-only LZO1X block.
        let plain = plain_entry();
        let body = &plain[0x30 + ENTRY_HEADER..];
        let mut lzo = vec![17 + body.len() as u8];
        lzo.extend(body);
        lzo.extend([0x11, 0, 0]);
        let mut b = plain[..0x30 + ENTRY_HEADER].to_vec();
        b[0x30 + 0x2B] = 1;
        b.extend((body.len() as u32).to_le_bytes());
        b.extend((lzo.len() as u32).to_le_bytes());
        b.extend(lzo);
        assert_eq!(decode_entry(&b, 0x30).unwrap(), {
            let mut c = decode_entry(&plain, 0x30).unwrap();
            c.compressed = true;
            c
        });
    }

    #[test]
    fn combine_matches_spec_blend_vector() {
        let mut j = Some(([0.0, 0.0, 0.0, 1.0], [0.0; 3]));
        combine(&mut j, ([0.0, 0.7071, 0.0, 0.7071], [4.0, 0.0, 0.0]), false, 0.25);
        let (q, t) = j.unwrap();
        assert!((q[1] - 0.1768).abs() < 1e-4 && (q[3] - 0.9268).abs() < 1e-4, "{q:?}");
        assert_eq!(t, [1.0, 0.0, 0.0]);
        // A negated sample gives the same result.
        let mut k = Some(([0.0, 0.0, 0.0, 1.0], [0.0; 3]));
        combine(&mut k, ([0.0, -0.7071, 0.0, -0.7071], [4.0, 0.0, 0.0]), false, 0.25);
        assert_eq!(j, k);
        // Full weight replaces.
        combine(&mut k, ([1.0, 0.0, 0.0, 0.0], [0.0; 3]), true, 0.0);
        assert_eq!(k, Some(([1.0, 0.0, 0.0, 0.0], [0.0; 3])));
    }

    #[test]
    fn hero_resources_match_spec_table() {
        assert_eq!(hero_resources(0), [(492, 493, 520), (494, 495, 521), (496, 497, 522)]);
        assert_eq!(hero_resources(1), [(498, 499, 523), (500, 501, 524), (502, 503, 525)]);
    }

    #[test]
    fn skin_matrix_is_identity_at_rest_and_follows_parents() {
        let bones = BoneTable {
            parents: vec![None, Some(0)],
            rest_local: vec![[0.0, 1.0, 0.0], [0.0, 2.0, 0.0]],
            rest_position: vec![[0.0, 1.0, 0.0], [0.0, 3.0, 0.0]],
            hurt: Vec::new(),
            attach: Vec::new(),
        };
        let def = SkeletonDef { parents: vec![None, Some(0)], partners: vec![0, 1], scales: Vec::new() };
        let rest = skin_matrices(&bones, &Family { def: &def, skin_base: 0, joints: &[None, None] }, &[]);
        assert!(rest.iter().all(|m| m.apply([5.0, 6.0, 7.0]) == [5.0, 6.0, 7.0]));
        // Turn the root 90° about +Y: a point at bone 1's rest position stays on bone 1.
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let joints = [Some(([0.0, h, 0.0, h], [0.0, 1.0, 0.0])), None];
        let s = skin_matrices(&bones, &Family { def: &def, skin_base: 0, joints: &joints }, &[]);
        let p = s[1].apply([1.0, 3.0, 0.0]);
        assert!((p[0]).abs() < 1e-6 && (p[1] - 3.0).abs() < 1e-6 && (p[2] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn mirrored_sample_goes_to_the_partner() {
        // SPEC-0005 Mirrored clips: joint 1's sample lands on its partner 2, rotation (x, -y,
        // -z, w) and translation (-x, y, z); without the flag it stays on joint 1.
        let clip = Clip {
            frames: 1,
            looping: false,
            speed_byte: 0,
            fade_in: 0,
            compressed: false,
            mode: 0,
            bones: vec![1],
            rotations: vec![[0.1, 0.2, 0.3, 0.9]],
            translations: vec![[1.0, 2.0, 3.0]],
            root: vec![[0.0; 3]],
            frame_bits: vec![0],
        };
        let mut joints = vec![None; 3];
        sample_into(&clip, 0, 0, 0.0, 0, &mut joints, true, 1.0, &|_| 1.0, Some(&[0, 2, 1]));
        assert_eq!((joints[1], joints[2]), (None, Some(([0.1, -0.2, -0.3, 0.9], [-1.0, 2.0, 3.0]))));
        let mut joints = vec![None; 3];
        sample_into(&clip, 0, 0, 0.0, 0, &mut joints, true, 1.0, &|_| 1.0, None);
        assert_eq!(joints[1], Some(([0.1, 0.2, 0.3, 0.9], [1.0, 2.0, 3.0])));
    }
}
