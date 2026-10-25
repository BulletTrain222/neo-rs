//! Foliage patches (SPEC-0060 Behavior 1 to 3): the family and models a key selects, and the
//! placement of each plant (jitter, ground snap, yaw, scale) with the patch distances.
//!
//! Not covered: the zone gate, the near/far fade weights and the far batches (the LOD-node
//! distances are not identified), the family logic (grass sway, bush cutting, drops and regrowth,
//! the sparkle, tree trunks, reed bending and the cut-piece fall) and the once-per-area gates.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Grass,
    Flowers,
    Bush,
    Tree,
    Reed,
    Bed,
}

/// What a key builds: the family, its near (main) model and far model registry ids, whether
/// the plants snap to the ground, and the lower bound of the random scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kind {
    pub family: Family,
    pub near: Option<u16>,
    pub far: Option<u16>,
    pub snap: bool,
    pub s0: f32,
}

/// The family table of Behavior 1 (keys 15 and 40 and up build nothing).
pub fn kind(key: u32) -> Option<Kind> {
    use Family::*;
    let k = |family, near, far, s0| Some(Kind { family, near, far, snap: true, s0 });
    match key {
        0 => k(Grass, None, Some(283), 1.0),
        12 => Some(Kind { snap: false, ..kind(0)? }),
        1 | 7 => k(Flowers, Some(194), None, 0.6),
        6 => k(Flowers, Some(193), None, 0.6),
        3 | 9 => k(Bush, Some(196), None, 0.6),
        2 | 29 => k(Bush, Some(195), None, 0.6),
        5 => k(Bush, Some(198), None, 0.6),
        16 => k(Bush, Some(222), None, 0.6),
        17 => k(Bush, Some(223), None, 0.6),
        18 => k(Bush, Some(224), None, 0.6),
        19 => k(Bush, Some(226), None, 0.6),
        20 => k(Bush, Some(227), None, 0.6),
        21 => k(Bush, Some(229), None, 0.6),
        22 => k(Bush, Some(231), None, 0.6),
        28 => k(Bush, Some(248), None, 0.6),
        30 => k(Bush, Some(253), None, 0.6),
        31 => k(Bush, Some(256), None, 0.6),
        32 => k(Bush, Some(259), None, 0.6),
        33 => k(Bush, Some(262), None, 0.6),
        34 => k(Bush, Some(265), None, 1.0),
        36 => k(Bush, Some(271), None, 0.6),
        37 => k(Bush, Some(274), None, 0.6),
        38 => k(Bush, Some(277), None, 0.6),
        39 => k(Bush, Some(280), None, 0.6),
        4 => k(Tree, Some(200), None, 0.85),
        25 => k(Tree, Some(200), Some(199), 0.85),
        13 => k(Tree, None, Some(220), 0.85),
        8 => k(Reed, Some(202), Some(203), 0.73),
        10 => k(Bed, Some(197), None, 0.85),
        11 => k(Bed, Some(204), None, 0.85),
        14 => k(Bed, Some(221), None, 0.85),
        23 => k(Bed, Some(233), None, 0.85),
        24 => k(Bed, Some(234), None, 0.85),
        26 => k(Bed, Some(235), None, 0.85),
        27 => k(Bed, Some(239), None, 0.85),
        35 => k(Bed, Some(268), None, 0.85),
        _ => None,
    }
}

/// The upper bound of the random scale: 3.5 for the bushes of model 265, else 1.0.
pub fn scale_max(kind: &Kind) -> f32 {
    if kind.near == Some(265) { 3.5 } else { 1.0 }
}

/// The jitter half-range J (Behavior 2 step 4): 0.9 for grass, 0 for reeds; with the jitter flag
/// 0.4 x depth / rows for bushes, beds and trees and 0.27 x depth / rows for flowers.
pub fn jitter(family: Family, flag: bool, depth: f32, rows: usize) -> f32 {
    let per_row = if rows == 0 { 0.0 } else { depth / rows as f32 };
    match family {
        Family::Grass => 0.9,
        Family::Reed => 0.0,
        _ if !flag => 0.0,
        Family::Flowers => 0.27 * per_row,
        _ => 0.4 * per_row,
    }
}

/// The ceiling the ground probe starts above the patch height: 8 for the bushes of models 198
/// and 223 (SPEC-0018 Ceilings supplied by callers); None for the default column probe.
pub fn probe_ceiling(kind: &Kind) -> Option<f32> {
    matches!(kind.near, Some(198 | 223)).then_some(8.0)
}

/// The far distance F of a patch (Behavior 3): the override D when A or D is nonzero, else
/// 4000 (the LOD node's value when the model has one is not read).
pub fn far_distance(overrides: [f32; 4]) -> f32 {
    if overrides[0] != 0.0 || overrides[3] != 0.0 { overrides[3] } else { 4000.0 }
}

/// The patch radius (Behavior 2 step 2): 0.7072 x the larger box side.
pub fn radius(area: [f32; 4]) -> f32 {
    0.7072 * (area[1] - area[0]).max(area[3] - area[2])
}

/// A placed plant: position, yaw (radians), scale K and colour bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub position: [f32; 3],
    pub yaw: f32,
    pub scale: f32,
    pub colour: [u8; 4],
}

/// Place plant `offset` of a row at `row` (Behavior 2 steps 3 to 8): jitter x and z in [-J, J],
/// snap to the ground (`ground` returns None when there is no ground: the plant is not placed),
/// a random yaw and a scale in [s0, max].
#[allow(clippy::too_many_arguments)]
pub fn place(kind: &Kind, area: [f32; 4], height: f32, row: f32, offset: f32, j: f32, colour: [u8; 4], ground: &dyn Fn(f32, f32) -> Option<f32>, r: &mut crate::rng::Rng) -> Option<Placed> {
    let mut x = area[0] + offset;
    let mut z = area[2] + row;
    if j > 0.0 {
        x += r.frange(-j, j);
        z += r.frange(-j, j);
    }
    let y = if kind.snap { ground(x, z).filter(|&h| h > -1.0e8)? } else { height };
    let yaw = r.frange(-std::f32::consts::PI, std::f32::consts::PI);
    let scale = r.frange(kind.s0, scale_max(kind));
    Some(Placed { position: [x, y, z], yaw, scale, colour })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_and_rules() {
        assert_eq!(kind(0).unwrap().far, Some(283));
        assert!(!kind(12).unwrap().snap);
        assert_eq!(kind(33).unwrap().near, Some(262));
        assert_eq!((kind(15), kind(40)), (None, None));
        assert_eq!(probe_ceiling(&kind(5).unwrap()), Some(8.0));
        assert_eq!(jitter(Family::Grass, false, 10.0, 2), 0.9);
        assert_eq!(jitter(Family::Bush, false, 10.0, 2), 0.0);
        assert!((jitter(Family::Bush, true, 10.0, 2) - 2.0).abs() < 1e-6);
        assert!((jitter(Family::Flowers, true, 10.0, 2) - 1.35).abs() < 1e-6);
        assert_eq!((far_distance([0.0; 4]), far_distance([200.0, 32.0, 50.0, 150.0])), (4000.0, 150.0));
        // SPEC-0006 check: pack 678 record 0's plant 1 of row 0 lies at x = -1015.50 + 1.2959.
        let k = kind(9).unwrap();
        let mut r = crate::rng::Rng::seeded(1);
        let p = place(&k, [-1015.5, -1012.81, 1320.54, 1325.01], 74.1661, 2.073, 1.2959, 0.0, [64, 64, 64, 128], &|_, _| Some(74.0), &mut r).unwrap();
        assert!((p.position[0] + 1014.2041).abs() < 1e-3 && (p.position[2] - 1322.613).abs() < 1e-3);
        assert!((0.6..=1.0).contains(&p.scale));
        assert!(place(&k, [0.0; 4], 0.0, 0.0, 0.0, 0.0, [0; 4], &|_, _| None, &mut r).is_none());
    }
}
