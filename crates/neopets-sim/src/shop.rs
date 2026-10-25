//! Shops (SPEC-0050 sections 3, 8 and 9; SPEC-0023 section 15; SPEC-0035 section 15.2): the
//! buyable item choice, the buy decision, the take-or-copy rule and the fixed-width number field
//! of the price caption.

/// SPEC-0023 section 15: an integer in a field of n cells, right-aligned, with a comma before
/// each group of three digits and blank leading cells. Values of 10,000,000 or more and
/// negative values give the original's non-digit cells.
pub fn number_field(value: i32, n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n);
    let mut v = value as i64;
    let mut p: i64 = 10i64.pow((n - 1 - n / 4) as u32);
    let mut started = false;
    for i in 0..n {
        let r = n - 1 - i;
        if r % 4 == 3 {
            out.push(if started { b',' } else { b' ' });
            continue;
        }
        let d = v / p;
        v -= d * p;
        p /= 10;
        if d != 0 {
            out.push((48 + d) as u8);
            started = true;
        } else {
            out.push(if started || i == n - 1 { b'0' } else { b' ' });
        }
    }
    out
}

/// A world item as the buy chooser sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub position: [f32; 3],
    pub flag: u8,
    pub ready: bool,
}

/// SPEC-0050 section 3, Choosing the buyable object: ready items with a vertical gap under 5.0,
/// a horizontal distance under 4.0 and a facing cosine above 0.8; the lowest distance minus
/// 2 x cosine wins; the winner is buyable only with flag byte 1.
pub fn buyable(items: &[Candidate], hero: [f32; 3], facing: [f32; 3]) -> Option<usize> {
    let flen = facing[0].hypot(facing[2]).max(1e-6);
    let mut best: Option<(usize, f32)> = None;
    for (i, c) in items.iter().enumerate() {
        if !c.ready || (c.position[1] - hero[1]).abs() >= 5.0 {
            continue;
        }
        let (dx, dz) = (c.position[0] - hero[0], c.position[2] - hero[2]);
        let d = dx.hypot(dz);
        let cos = if d > 1e-6 { (dx * facing[0] + dz * facing[2]) / (d * flen) } else { 1.0 };
        if d >= 4.0 || cos <= 0.8 {
            continue;
        }
        let score = d - 2.0 * cos;
        if best.is_none_or(|(_, s)| score < s) {
            best = Some((i, score));
        }
    }
    best.map(|(i, _)| i).filter(|&i| items[i].flag == 1)
}

/// The price that is never affordable.
pub const EXORBITANT: i16 = 0x7FFF;

/// The outcome of the buy action after the hook step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buy {
    /// Price 0x7FFF: the never-affordable message.
    NeverAffordable,
    /// The price is above the balance: the need-more message for this amount.
    NeedMore(i32),
    /// The add failed: the can't-buy-now message.
    CantBuyNow,
    /// Bought: the balance falls by the price.
    Bought,
}

/// SPEC-0050 section 3 steps 3 to 5 (the price is read as signed; a negative price passes the
/// balance test and raises the balance).
pub fn decide(price: i16, balance: i32, add_ok: bool) -> Buy {
    if price == EXORBITANT {
        Buy::NeverAffordable
    } else if price as i32 > balance {
        Buy::NeedMore(price as i32 - balance)
    } else if !add_ok {
        Buy::CantBuyNow
    } else {
        Buy::Bought
    }
}

/// SPEC-0050 section 8 Take or copy: equipment (config types 1 to 6), the stat-up items (Golden,
/// Red and Starry Neggs and the two clovers) and treasure maps are taken from the level; other
/// purchases add a copy and leave the item on display.
pub fn takes(id: u16, config_type: i16) -> bool {
    (1..=6).contains(&config_type) || matches!(id, 19 | 20 | 151 | 154 | 174 | 175)
}

/// SPEC-0050 section 5: a price scaled by the multiplier, capped at 32767 (no lower clamp).
pub fn scaled(price: i16, factor: f32) -> i16 {
    ((price as f32 * factor) as i32).min(32767) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_field_vectors() {
        // SPEC-0023 section 15 and SPEC-0035 test vectors.
        assert_eq!(number_field(0, 9), b"        0");
        assert_eq!(number_field(150, 9), b"      150");
        assert_eq!(number_field(1234, 9), b"    1,234");
        assert_eq!(number_field(9_999_999, 9), b"9,999,999");
        assert_eq!(number_field(12_345_678, 9), b"<,345,678");
        assert_eq!(number_field(-5, 9), b"        +");
    }

    #[test]
    fn buy_path_vectors() {
        // SPEC-0050 Buy path (balance 500).
        assert_eq!(decide(EXORBITANT, 500, true), Buy::NeverAffordable);
        assert_eq!(decide(EXORBITANT, 40000, true), Buy::NeverAffordable);
        assert_eq!(decide(600, 500, true), Buy::NeedMore(100));
        assert_eq!(decide(300, 500, true), Buy::Bought);
        assert_eq!(decide(300, 500, false), Buy::CantBuyNow);
        assert_eq!(decide(-50, 500, true), Buy::Bought);
    }

    #[test]
    fn multiplier_vectors() {
        assert_eq!(scaled(30, 4.0), 120);
        assert_eq!(scaled(20000, 4.0), 32767);
        assert_eq!(scaled(-1000, 4.0), -4000);
    }

    #[test]
    fn chooser_picks_the_best_score_and_needs_flag_1() {
        let c = |x: f32, z: f32, flag| Candidate { position: [x, 0.0, z], flag, ready: true };
        let hero = [0.0, 0.0, 0.0];
        let f = [0.0, 0.0, 1.0];
        // Ahead at 2 (score 2 - 2 = 0) beats ahead at 3 (score 1).
        assert_eq!(buyable(&[c(0.0, 3.0, 1), c(0.0, 2.0, 1)], hero, f), Some(1));
        // Behind or beyond 4: no candidate.
        assert_eq!(buyable(&[c(0.0, -2.0, 1), c(0.0, 4.5, 1)], hero, f), None);
        // The winner has flag 0: nothing is buyable even with a flag-1 item in range.
        assert_eq!(buyable(&[c(0.0, 3.0, 1), c(0.0, 2.0, 0)], hero, f), None);
        assert!(takes(98, 4) && takes(175, 0) && takes(151, 7) && !takes(54, 7));
    }
}
