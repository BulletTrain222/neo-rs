//! Motes in combat (SPEC-0036 Damage, Mote elements and type match, Mote charge use by attacks,
//! Magic, Enemy attack damage to a hero; SPEC-0021 Equipment slots). Word 2 is the attack Mote,
//! word 1 the shield Mote and word 0 the armour Mote. Every product and sum is the console's
//! single-precision operation (truncation toward zero), then a truncating conversion.

use ps2_float::fpu::{add, mul, sub};

/// A Mote is an item id 160 to 173.
pub fn is_mote(id: u16) -> bool {
    (160..=173).contains(&id)
}

/// The odd family (161, 163, ..., 173); the even family is 160, 162, ..., 172.
pub fn odd(id: u16) -> bool {
    id % 2 == 1
}

/// The element kind of an item (SPEC-0036 Mote elements): 0 for anything that is not a Mote.
pub fn element(id: Option<u16>) -> i32 {
    match id {
        Some(160 | 161) => 1,
        Some(162 | 163) => 4,
        Some(164 | 165) => 2,
        Some(166 | 167) => 5,
        Some(168 | 169) => 6,
        Some(170 | 171) => 3,
        Some(172 | 173) => 7,
        _ => 0,
    }
}

/// Match(E_a, T): 1-4, 2-5 and 3-6 in either order; 7 matches every T; 0 matches none.
pub fn matches(ea: i32, t: i32) -> bool {
    match ea {
        7 => true,
        1..=6 => t == (ea + 2) % 6 + 1,
        _ => false,
    }
}

/// The element rule of the shield block and the armour Mote: the attack's element is 0, equals
/// the Mote's element kind, or is 7.
pub fn element_passes(attack: i32, mote: i32) -> bool {
    attack == 0 || attack == mote || attack == 7
}

/// The cost and power words of the Magic table (boot data 0x2DD47C to 0x2DD507, s32).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Magic {
    words: Vec<i32>,
}

pub const MAGIC_AT: u32 = 0x002D_D47C;
pub const MAGIC_WORDS: usize = 35;

impl Magic {
    pub fn new(words: Vec<i32>) -> Self {
        Self { words }
    }

    fn w(&self, va: u32) -> i32 {
        self.words.get((va.wrapping_sub(MAGIC_AT) / 4) as usize).copied().unwrap_or(0)
    }

    /// (cost, power) of a family by its two word pairs; no Mote: (0, 0).
    fn pick(&self, mote: Option<u16>, odd_words: (u32, u32), even_words: (u32, u32)) -> (i32, i32) {
        match mote {
            Some(id) if odd(id) => (self.w(odd_words.0), self.w(odd_words.1)),
            Some(_) => (self.w(even_words.0), self.w(even_words.1)),
            None => (0, 0),
        }
    }

    /// Tor level 0, the normal swing (word-2 Mote).
    pub fn tor_swing(&self, mote: Option<u16>) -> (i32, i32) {
        self.pick(mote, (0x002D_D4B8, 0x002D_D4F8), (0x002D_D4A8, 0x002D_D4E8))
    }

    /// Tor level 1, the power attack release (word-2 Mote).
    pub fn tor_power(&self, mote: Option<u16>) -> (i32, i32) {
        self.pick(mote, (0x002D_D4C4, 0x002D_D504), (0x002D_D4B4, 0x002D_D4F4))
    }

    /// Roberta's quick shot at launch class 0 to 2 (and the default row, class 4, as level 1):
    /// the family's cost and power, or with no Mote 3, 5 or 10 MP by class and power 0.
    pub fn roberta_quick(&self, class: u8, mote: Option<u16>) -> (i32, i32) {
        match mote {
            Some(_) => self.pick(mote, (0x002D_D498, 0x002D_D4D8), (0x002D_D488, 0x002D_D4C8)),
            None => (self.w(match class {
                0 => 0x002D_D47C,
                2 => 0x002D_D484,
                _ => 0x002D_D480,
            }), 0),
        }
    }

    /// Roberta's full release (level 3): the family's cost and power, or 10 MP and power 0.
    pub fn roberta_full(&self, mote: Option<u16>) -> (i32, i32) {
        match mote {
            Some(_) => self.pick(mote, (0x002D_D4A4, 0x002D_D4E4), (0x002D_D494, 0x002D_D4D4)),
            None => (self.w(0x002D_D484), 0),
        }
    }

    /// The armour Mote (word 0) of hero 0 (Tor) or 1 (Roberta): cost and power Q0.
    pub fn armour(&self, hero: usize, mote: Option<u16>) -> (i32, i32) {
        if hero == 0 {
            self.pick(mote, (0x002D_D4BC, 0x002D_D4FC), (0x002D_D4AC, 0x002D_D4EC))
        } else {
            self.pick(mote, (0x002D_D49C, 0x002D_D4DC), (0x002D_D48C, 0x002D_D4CC))
        }
    }
}

/// The shield Mote's cost (2 odd, 1 even) and power Q1 = 1 (SPEC-0036 Magic; no data words).
pub fn shield_cost(mote: u16) -> (i32, i32) {
    (if odd(mote) { 2 } else { 1 }, 1)
}

/// Tor's charge amount for power Q and weapon stat S: trunc(Q x (S x 0.01)), raised to 1 when it
/// is below 1 and Q is at least 1 (also the shield block's rule with Q = 1).
pub fn charge_tor(q: i32, s: f32) -> i32 {
    let n = mul(q as f32, mul(s, 0.01)) as i32;
    if n < 1 && q >= 1 { 1 } else { n }
}

/// The armour Mote's charge amount: trunc(Q0 x (body stat B x 0.01)), no minimum.
pub fn charge_armour(q0: i32, body_b: f32) -> i32 {
    mul(q0 as f32, mul(body_b, 0.01)) as i32
}

/// The factors of the attack Mote in word 2: its stats F and G and its element kind.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AttackMote {
    pub f: f32,
    pub g: f32,
    pub kind: i32,
}

/// A hero's melee hit (SPEC-0036 Damage): x = P A B C D E, y = (x - defence) x G, floored at 0
/// and truncated. Returns (damage, absorbed); absorbed when x > 0 and x - defence <= 0.
pub fn melee(p: f32, a: f32, mote: Option<AttackMote>, target_type: i32, d: f32, e: f32, defence: f32, g: f32) -> (i32, bool) {
    let (b, c) = match mote {
        Some(m) => {
            let c = if matches(m.kind, target_type) { add(1.0, mul(m.g, 0.01)) } else { 1.0 };
            (add(1.0, mul(m.f, 0.01)), c)
        }
        None => (1.0, 1.0),
    };
    let x = mul(mul(mul(mul(mul(p, a), b), c), d), e);
    let y = sub(x, defence);
    let absorbed = x > 0.0 && y <= 0.0;
    let y = mul(y, g).max(0.0);
    (y as i32, absorbed)
}

/// 1 + v x 0.01 (factors B and C, and the charm C(id)).
pub fn one_plus_percent(v: f32) -> f32 {
    add(1.0, mul(v, 0.01))
}

/// d x f (the petpet intake factor and other single-precision scales).
pub fn scaled(d: f32, f: f32) -> f32 {
    mul(d, f)
}

/// The shield block b of a blocked hit: the shield's stat A, times 1.25 with a shield Mote.
pub fn shield_block(stat_a: f32, mote: bool) -> f32 {
    if mote { mul(1.25, stat_a) } else { stat_a }
}

/// d0 - b, the hit less the shield block.
pub fn less_block(d0: f32, b: f32) -> f32 {
    sub(d0, b)
}

/// Armour step 1 (the armour Mote): d = (d x 0.01) x (100 - H).
pub fn armour_mote(d: f32, h: f32) -> f32 {
    mul(mul(d, 0.01), sub(100.0, h))
}

/// Armour step 2 (the body item): d = d x (1 - A_body x 0.01).
pub fn armour_body(d: f32, a_body: f32) -> f32 {
    mul(d, sub(1.0, mul(a_body, 0.01)))
}

/// The end of an enemy hit on a hero: y = max(0, d1 x G), truncated (no minimum).
pub fn taken(d1: f32, g: f32) -> i32 {
    mul(d1, g).max(0.0) as i32
}

/// The attack power table record of (hero, Mote id) (SPEC-0036 Layout, the index map).
pub fn power_index(hero: usize, mote: u16) -> Option<usize> {
    const TOR: [usize; 14] = [0, 4, 8, 9, 11, 15, 16, 18, 20, 22, 24, 26, 28, 29];
    let i = TOR.get(mote.checked_sub(160)? as usize)?;
    Some(if hero == 0 { *i } else { i + 30 })
}

/// The status a Mote's attack sends (SPEC-0036 Layout, Mote status words; SPEC-0021): the
/// code by id, and the duration, amount and period from the attack power table records after
/// the Mote's power index (whole numbers, -1 for a field the Mote does not have). None for the
/// Motes with no status. `table` reads record i of the table.
pub fn status_request(hero: usize, id: u16, table: impl Fn(usize) -> Option<f32>) -> Option<[i32; 4]> {
    let code = match id {
        160 | 161 => 7,
        163 => 0,
        164 | 165 => 2,
        166 | 167 => 3,
        168 | 169 => 1,
        170 | 171 => 6,
        _ => return None,
    };
    let k = power_index(hero, id)?;
    let word = |n: usize| table(k + n).map_or(-1, |v| v as i32);
    Some(match id {
        160 | 161 | 164 => [code, word(1), word(2), word(3)],
        165 => [code, -1, -1, -1],
        _ => [code, word(1), -1, -1],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elements_and_match() {
        assert_eq!((element(Some(160)), element(Some(163)), element(Some(173)), element(Some(98)), element(None)), (1, 4, 7, 0, 0));
        for (a, b) in [(1, 4), (2, 5), (3, 6)] {
            assert!(matches(a, b) && matches(b, a));
        }
        assert!(matches(7, 0) && matches(7, 7) && !matches(1, 1) && !matches(0, 0) && !matches(1, 0) && !matches(4, 7));
        assert!(element_passes(0, 1) && element_passes(7, 4) && element_passes(4, 4) && !element_passes(1, 4));
    }

    #[test]
    fn melee_vectors() {
        // SPEC-0036 Platform arithmetic: sword stat A 4 with Mote 161 (F 25) deals 4
        // (4.9999995), not 5; without a Mote it deals 4.
        let m = AttackMote { f: 25.0, g: 100.0, kind: 1 };
        assert_eq!(melee(1.0, 4.0, Some(m), 0, 1.0, 1.0, 0.0, 1.0), (4, false));
        assert_eq!(melee(1.0, 4.0, None, 0, 1.0, 1.0, 0.0, 1.0), (4, false));
        // Defence at least x: absorbed with 0 damage.
        assert_eq!(melee(1.0, 4.0, None, 0, 1.0, 1.0, 5.0, 1.0), (0, true));
        // Unarmed: x = 0, nothing absorbed.
        assert_eq!(melee(1.0, 0.0, Some(m), 4, 1.0, 1.0, 0.0, 1.0), (0, false));
        // A matched target (T = 4 for kind 1) raises the hit through C.
        assert!(melee(1.0, 10.0, Some(m), 4, 1.0, 1.0, 0.0, 1.0).0 > melee(1.0, 10.0, Some(m), 0, 1.0, 1.0, 0.0, 1.0).0);
    }

    #[test]
    fn charge_vectors() {
        // SPEC-0036 Mote charge use: Tor swings use 1 (lesser, Q 2) and 3 (greater, Q 4) with
        // S = 100; power attacks 24 and 39; the shield block 1.
        assert_eq!((charge_tor(2, 100.0), charge_tor(4, 100.0)), (1, 3));
        assert_eq!((charge_tor(25, 100.0), charge_tor(40, 100.0)), (24, 39));
        assert_eq!((charge_tor(1, 0.0), charge_tor(0, 100.0)), (1, 0));
        // Armour: 3 for a lesser (Q0 4) and 5 for a greater (Q0 6) Mote with body stat B 100.
        assert_eq!((charge_armour(4, 100.0), charge_armour(6, 100.0)), (3, 5));
    }

    #[test]
    fn enemy_hit_vectors() {
        // SPEC-0036 Enemy attack damage: book 178 (stat A 40), a blocked hit of 60: word 1
        // empty gives 20, a shield Mote gives 10.
        assert_eq!(taken(60.0 - shield_block(40.0, false), 1.0), 20);
        assert_eq!(taken(60.0 - shield_block(40.0, true), 1.0), 10);
        assert_eq!(taken(sub(10.0, 20.0), 1.0), 0, "no minimum, floored at 0");
        // A greater armour Mote (H 75) takes a hit of 40 to about a quarter; a body item of 50
        // halves it.
        assert_eq!(taken(armour_mote(40.0, 75.0), 1.0), 9);
        assert_eq!(taken(armour_body(40.0, 50.0), 1.0), 20);
    }

    #[test]
    fn magic_words_and_power_index() {
        // Synthetic words: each word holds its own index.
        let m = Magic::new((0..MAGIC_WORDS as i32).collect());
        let at = |va: u32| ((va - MAGIC_AT) / 4) as i32;
        assert_eq!(m.tor_swing(Some(161)), (at(0x002D_D4B8), at(0x002D_D4F8)));
        assert_eq!(m.tor_swing(Some(160)), (at(0x002D_D4A8), at(0x002D_D4E8)));
        assert_eq!(m.tor_power(None), (0, 0));
        assert_eq!(m.armour(1, Some(163)), (at(0x002D_D49C), at(0x002D_D4DC)));
        assert_eq!((power_index(0, 160), power_index(0, 173), power_index(1, 163), power_index(0, 98)), (Some(0), Some(29), Some(39), None));
        assert_eq!(shield_cost(161), (2, 1));
    }

    #[test]
    fn status_words() {
        // Synthetic table: record i holds i. Tor's Lava (index 4): fire with records 5, 6, 7;
        // Water (index 9): stun for record 10; Rock: poison with the defaults; Nova: none.
        let t = |i: usize| (i < 60).then_some(i as f32);
        assert_eq!(status_request(0, 161, t), Some([7, 5, 6, 7]));
        assert_eq!(status_request(0, 163, t), Some([0, 10, -1, -1]));
        assert_eq!(status_request(1, 164, t), Some([2, 42, 43, 44]));
        assert_eq!(status_request(0, 165, t), Some([2, -1, -1, -1]));
        assert_eq!(status_request(0, 172, t), None);
        assert_eq!(status_request(0, 162, t), None);
    }
}
