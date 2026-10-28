//! The monster table (SPEC-0029 section 4): resource 530 (rows of 0x504 bytes) and resource 531
//! (companion rows of 0x60 bytes) share a header of u32 N = 112 types and N u32 first-row
//! indices; rows start at 0x1D0. A placement's row is first[type] + variant.

/// SPEC-0038 Behavior 18: the band factors F1 and F2 (seven f32 each) in the boot program.
pub const BAND_F1: u32 = 0x002D_D5A4;
pub const BAND_F2: u32 = 0x002D_D5C0;

const HEADER_TYPES: usize = 112;
const ROWS_AT: usize = 0x1D0;
const ROW: usize = 0x504;
const COMPANION: usize = 0x60;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

fn i16_at(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

/// The row fields used so far (offsets in SPEC-0029 section 4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub radius: f32,
    pub height: f32,
    pub hit_points: u16,
    /// Default behavior class: the row's (+0x36) or, when none, the companion row's (+0x10).
    pub class: Option<i16>,
    pub hostile: bool,
    pub weight: f32,
    /// Can fly (+0x94): 0 never, 1 while grounded countdown ≤ 0, 2 always.
    pub can_fly: i32,
    /// Enemy defence (Damage Reduction, +0x114).
    pub defence: f32,
    /// Death disappear timer (+0x80).
    pub disappear: u16,
    /// The villager row's emote voice set (resource 531 row +0x14): -1 none, 0 to 15.
    pub voice_set: i16,
    /// The target type T (Alignment, +0x3C; SPEC-0029 Layout 4): 0 none, 1 to 7 the Mote
    /// element classes of the match rule (SPEC-0036 Damage).
    pub target_type: i16,
}

/// One attack slot k of a row (SPEC-0038 Layout section 2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AttackSlot {
    /// 0 melee, 1 missile, 0xFFFF unused (+0x16C + 2k).
    pub kind: u16,
    pub orient: bool,
    pub trigger_max: f32,
    pub trigger_min: f32,
    pub chance: f32,
    pub special: f32,
    pub wind_speed: f32,
    pub strike_speed: f32,
    pub recover_speed: f32,
    pub arc: f32,
    pub range: f32,
    pub delay_max: f32,
    pub delay_min: f32,
    pub charge: i32,
    pub charge_max: f32,
    pub charge_min: f32,
    pub damage: f32,
    pub knockback: f32,
    pub flags: u32,
    pub spawn_cap: u16,
    pub loopable: bool,
    pub loop_min: i32,
    pub loop_max: i32,
    pub loop_hold: f32,
    /// Missile values (+0x2BC flight, +0x2CC speed, +0x2DC end speed, +0x2EC radius, +0x2FC
    /// range, +0x30C inaccuracy, +0x31C spin, +0x33C spawn count, +0x36C/+0x37C status id and
    /// chance, +0x38C/+0x39C status range, +0x43C visual kind, +0x44C/+0x45C area duration and
    /// radius, +0x3CC + 2k spawn key).
    pub blocked_charge: i32,
    /// Counter Attack Trigger (+0x22C, codes 1 to 6) and Counter Attack Chance (+0x23C, a
    /// percent; SPEC-0038 Layout section 5).
    pub counter_trigger: i32,
    pub counter_chance: f32,
    pub stun_max: f32,
    pub stun_min: f32,
    pub stun_type: i32,
    pub flight: i32,
    pub speed: f32,
    pub end_speed: f32,
    pub radius: f32,
    pub missile_range: f32,
    pub inaccuracy: f32,
    pub spin: f32,
    pub spawn_count: i32,
    pub status: i32,
    pub status_chance: f32,
    pub status_range: (f32, f32),
    pub visual: i32,
    pub area_duration: f32,
    pub area_radius: f32,
    pub spawn_key: u16,
    /// The damage alignment of the attack (+0x3BC + 4k; SPEC-0029 Layout 4): its element.
    pub alignment: i32,
    /// The six effect ids of a shot (u16 at +0x46C + 2k + 8j, j = 0 to 5; 0xFFFF is -1):
    /// flight 0 and 1, blocked hit, expiry, world impact, unblocked hit (SPEC-0029 Behavior 9).
    pub effects: [i32; 6],
}

/// The values the common-enemy class reads (SPEC-0038 Layout section 2) and the five weapon
/// volumes (SPEC-0029 Behavior 10.2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tuning {
    pub approach: u16,
    /// Alignment (+0x3C), the element of the enemy's projectiles.
    pub element: u16,
    pub display_default: bool,
    pub search_max: f32,
    pub search_min: f32,
    pub attack_timer_max: f32,
    pub attack_timer_min: f32,
    pub approach_timer_max: f32,
    pub approach_timer_min: f32,
    pub turn: f32,
    /// Flying turn speed (0x88), Flying Idle and Agro Velocity (0x8C, 0x90), the ground timer
    /// max and min (0x98, 0x9C) and Max Fly Height (0xB4) (SPEC-0038 Layout 2; SPEC-0040 14D).
    pub fly_turn: f32,
    pub fly_idle: f32,
    pub fly_agro: f32,
    pub ground_max: f32,
    pub ground_min: f32,
    pub max_fly: f32,
    /// Distance hide flag (u16 +0xA0).
    pub distance_hide: bool,
    /// Difficulty value (u16 0x3A, the attack-manager weight) and the wait-in-line stand-off
    /// spread R (f32 0xB0; SPEC-0038 Behavior 45).
    pub weight: u16,
    pub standoff: f32,
    /// Distance hide distance and chance (f32 +0xA4, +0xA8; SPEC-0038 Behavior 24).
    pub hide_distance: f32,
    pub hide_chance: f32,
    /// The reanimation countdown range in ticks (int +0x160, +0x164) when the reanimation flag
    /// (+0x15C) is set (SPEC-0038 Behavior 34 and 35).
    pub reanimate: Option<(i32, i32)>,
    pub flying_hit: i32,
    pub anger: bool,
    pub view_angle: f32,
    pub awareness: i32,
    pub awareness_timer: f32,
    pub chance_far: f32,
    pub chance_near: f32,
    pub range_far: f32,
    pub range_near: f32,
    pub always_sight: bool,
    pub rush: (f32, f32),
    pub desired: (f32, f32),
    pub retreat: (f32, f32),
    pub strafe: (f32, f32),
    pub backbiter: f32,
    /// The defence words (0xE0 missile defend time; 0xE4, 0xE8 missile dodge and block; 0xEC to
    /// 0xF4 melee block; 0xF8 to 0x100 melee dodge; 0x104, 0x108 defence timer min and max).
    pub defend_time: i32,
    pub missile_defence: (f32, f32),
    pub melee_block: [f32; 3],
    pub melee_dodge: [f32; 3],
    pub defence_timer: (f32, f32),
    /// Blocking Angle (0x10C): a guarding target blocks a hit when dot(target facing, hero
    /// facing) is below it (SPEC-0036 Hit detection step 4).
    pub blocking_angle: f32,
    pub slots: [AttackSlot; 4],
    /// Weapon N: (radius +4(N-1), full length +0x14 + 4(N-1)), clamped to 40.0.
    pub weapons: [(f32, f32); 5],
}

#[derive(Clone, Debug, Default)]
pub struct MonsterTable {
    rows: Vec<u8>,
    companions: Vec<u8>,
    first: Vec<u32>,
}

impl MonsterTable {
    /// From the bytes of resources 530 and 531.
    pub fn parse(rows: &[u8], companions: &[u8]) -> Result<Self, String> {
        let n = u32_at(rows, 0).ok_or("Truncated monster table")? as usize;
        if n != HEADER_TYPES {
            return Err(format!("Monster table has {n} types, expected {HEADER_TYPES}"));
        }
        let first = (0..n).map(|t| u32_at(rows, 4 + 4 * t).ok_or("Truncated header")).collect::<Result<_, _>>()?;
        Ok(Self { rows: rows.to_vec(), companions: companions.to_vec(), first })
    }

    pub fn row_count(&self) -> usize {
        self.rows.len().saturating_sub(ROWS_AT) / ROW
    }

    /// The row index of (type, variant), when the type is in range.
    pub fn index(&self, monster_type: u16, variant: u16) -> Option<usize> {
        Some(*self.first.get(monster_type as usize)? as usize + variant as usize)
    }

    /// The class-17 values of (type, variant).
    pub fn tuning(&self, monster_type: u16, variant: u16) -> Option<Tuning> {
        self.tuning_at(self.index(monster_type, variant)?)
    }

    /// Row i's raw bytes.
    pub fn row_bytes(&self, i: usize) -> Option<&[u8]> {
        let r = ROWS_AT + ROW * i;
        self.rows.get(r..r + ROW)
    }

    /// The class-17 values of row i.
    pub fn tuning_at(&self, i: usize) -> Option<Tuning> {
        let r = ROWS_AT + ROW * i;
        let b = self.rows.get(r..r + ROW)?;
        let f = |o: usize| f32_at(b, o).unwrap_or(0.0);
        let u = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        let w = |o: usize| u32_at(b, o).unwrap_or(0);
        let slot = |k: usize| AttackSlot {
            kind: u(0x16C + 2 * k),
            orient: u(0x174 + 2 * k) != 0,
            trigger_max: f(0x17C + 4 * k),
            trigger_min: f(0x18C + 4 * k),
            chance: f(0x19C + 4 * k),
            special: f(0x1AC + 4 * k),
            wind_speed: f(0x1BC + 4 * k),
            strike_speed: f(0x1CC + 4 * k),
            recover_speed: f(0x1DC + 4 * k),
            arc: f(0x1EC + 4 * k),
            range: f(0x1FC + 4 * k),
            delay_max: f(0x20C + 4 * k),
            delay_min: f(0x21C + 4 * k),
            charge: w(0x24C + 4 * k) as i32,
            charge_max: f(0x25C + 4 * k),
            charge_min: f(0x26C + 4 * k),
            damage: f(0x34C + 4 * k),
            knockback: f(0x35C + 4 * k),
            flags: w(0x3AC + 4 * k),
            spawn_cap: u(0x3D4 + 2 * k),
            loopable: w(0x49C + 4 * k) != 0,
            loop_min: w(0x4AC + 4 * k) as i32,
            loop_max: w(0x4BC + 4 * k) as i32,
            loop_hold: f(0x4CC + 4 * k),
            blocked_charge: w(0x2AC + 4 * k) as i32,
            counter_trigger: w(0x22C + 4 * k) as i32,
            counter_chance: f(0x23C + 4 * k),
            stun_max: f(0x27C + 4 * k),
            stun_min: f(0x28C + 4 * k),
            stun_type: w(0x29C + 4 * k) as i32,
            flight: w(0x2BC + 4 * k) as i32,
            speed: f(0x2CC + 4 * k),
            end_speed: f(0x2DC + 4 * k),
            radius: f(0x2EC + 4 * k),
            missile_range: f(0x2FC + 4 * k),
            inaccuracy: f(0x30C + 4 * k),
            spin: f(0x31C + 4 * k),
            spawn_count: w(0x33C + 4 * k) as i32,
            status: w(0x36C + 4 * k) as i32,
            status_chance: f(0x37C + 4 * k),
            status_range: (f(0x38C + 4 * k), f(0x39C + 4 * k)),
            visual: w(0x43C + 4 * k) as i32,
            area_duration: f(0x44C + 4 * k),
            area_radius: f(0x45C + 4 * k),
            spawn_key: u(0x3CC + 2 * k),
            alignment: w(0x3BC + 4 * k) as i32,
            effects: std::array::from_fn(|j| u(0x46C + 2 * k + 8 * j) as i16 as i32),
        };
        Some(Tuning {
            approach: u(0x38),
            element: u(0x3C),
            display_default: u(0x82) != 0,
            search_max: f(0x5C),
            search_min: f(0x60),
            attack_timer_max: f(0x64),
            attack_timer_min: f(0x68),
            approach_timer_max: f(0x6C),
            approach_timer_min: f(0x70),
            turn: f(0x84),
            fly_turn: f(0x88),
            fly_idle: f(0x8C),
            fly_agro: f(0x90),
            ground_max: f(0x98),
            ground_min: f(0x9C),
            max_fly: f(0xB4),
            distance_hide: u(0xA0) != 0,
            weight: u(0x3A),
            standoff: f(0xB0),
            hide_distance: f(0xA4),
            reanimate: (w(0x15C) != 0).then(|| (w(0x160) as i32, w(0x164) as i32)),
            hide_chance: f(0xA8),
            flying_hit: w(0xAC) as i32,
            anger: w(0x134) != 0,
            view_angle: f(0xC0),
            awareness: w(0xC4) as i32,
            awareness_timer: f(0xC8),
            chance_far: f(0xCC),
            chance_near: f(0xD0),
            range_far: f(0xD4),
            range_near: f(0xD8),
            always_sight: w(0xDC) != 0,
            rush: (f(0x4DC), f(0x4E0)),
            desired: (f(0x4E4), f(0x4E8)),
            retreat: (f(0x4EC), f(0x4F0)),
            strafe: (f(0x4F4), f(0x4F8)),
            backbiter: f(0x500),
            defend_time: w(0xE0) as i32,
            missile_defence: (f(0xE4), f(0xE8)),
            melee_block: [f(0xEC), f(0xF0), f(0xF4)],
            melee_dodge: [f(0xF8), f(0xFC), f(0x100)],
            defence_timer: (f(0x104), f(0x108)),
            blocking_angle: f(0x10C),
            slots: std::array::from_fn(slot),
            weapons: std::array::from_fn(|n| (f(4 * n).min(40.0), f(0x14 + 4 * n).min(40.0))),
        })
    }

    pub fn row(&self, monster_type: u16, variant: u16) -> Option<Row> {
        self.row_at(self.index(monster_type, variant)?)
    }

    /// Row i.
    pub fn row_at(&self, i: usize) -> Option<Row> {
        let r = ROWS_AT + ROW * i;
        let c = ROWS_AT + COMPANION * i;
        if r + ROW > self.rows.len() {
            return None;
        }
        let own = i16_at(&self.rows, r + 0x36)?;
        let class = if own >= 0 { Some(own) } else { i16_at(&self.companions, c + 0x10).filter(|&k| k >= 0) };
        Some(Row {
            voice_set: i16_at(&self.companions, c + 0x14).unwrap_or(-1),
            target_type: i16_at(&self.rows, r + 0x3C)?,
            radius: f32_at(&self.rows, r + 0x28)?,
            height: f32_at(&self.rows, r + 0x2C)?,
            hit_points: u16::from_le_bytes(self.rows.get(r + 0x30..r + 0x32)?.try_into().ok()?),
            class,
            hostile: i16_at(&self.rows, r + 0x3E)? != 0,
            weight: f32_at(&self.rows, r + 0x118)?,
            can_fly: u32_at(&self.rows, r + 0x94)? as i32,
            defence: f32_at(&self.rows, r + 0x114)?,
            disappear: u16::from_le_bytes(self.rows.get(r + 0x80..r + 0x82)?.try_into().ok()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_lookup_and_companion_class() {
        // Two types: type 0 starts at row 0, type 1 at row 1; row 1 has no class of its own and
        // its companion row gives 6.
        let mut rows = vec![0u8; ROWS_AT + 2 * ROW];
        rows[0..4].copy_from_slice(&112u32.to_le_bytes());
        rows[8..12].copy_from_slice(&1u32.to_le_bytes());
        let r1 = ROWS_AT + ROW;
        rows[r1 + 0x30..r1 + 0x32].copy_from_slice(&50u16.to_le_bytes());
        rows[r1 + 0x36..r1 + 0x38].copy_from_slice(&(-1i16).to_le_bytes());
        rows[r1 + 0x28..r1 + 0x2C].copy_from_slice(&1.175f32.to_le_bytes());
        let mut comp = vec![0u8; ROWS_AT + 2 * COMPANION];
        comp[ROWS_AT + COMPANION + 0x10..ROWS_AT + COMPANION + 0x12].copy_from_slice(&6i16.to_le_bytes());
        let t = MonsterTable::parse(&rows, &comp).unwrap();
        assert_eq!(t.row_count(), 2);
        let row = t.row(1, 0).unwrap();
        assert_eq!((row.hit_points, row.class, row.radius), (50, Some(6), 1.175));
        assert!(t.row(1, 1).is_none(), "past the end");
        assert_eq!(t.tuning(1, 0).map(|u| u.slots[0].kind), Some(0));
    }
}
