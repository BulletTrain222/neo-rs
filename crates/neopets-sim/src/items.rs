//! World items, drops and pickups (SPEC-0021 World items, Drop and money flow, Pickup timing
//! gate, Pickup, Auto-collect, Coin pickups, Wallet, Pickup dispatch and the luck gate): the
//! 50-slot pool of a world, the spawn steps, the per-tick step (placed items, fade, clock, the
//! hover and bob of plain items, the charm hop, the coin flight, the scale grow), the nearest
//! ready item and the collect distance, the money and item rolls of a drop group, the greedy
//! coin split, and the luck gate.
//!
//! Not covered: the item text database (carry limits; the important-item give at expiry and the
//! range collect are run with the host's important test, the double give is not), the
//! game-state code (items never mature in play, as the spec notes), the shop,
//! the petpet reservation, the pickup hook and popup, the quick-item auto-select, cues, item
//! display models, clover patches, and the region-load point move of byte +0x21.

use crate::hero::Vec3;

const DT: f32 = 1.0 / 60.0;
pub const SLOTS: usize = 50;

/// The coin words of the boot program (SPEC-0021 Where it lives): launch speed centre, gravity,
/// restitution, stop speed, rest height, horizontal and spin damping, horizontal speed centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoinWords {
    pub launch: f32,
    pub gravity: f32,
    pub restitution: f32,
    pub stop: f32,
    pub rest: f32,
    pub damping: f32,
    pub spin_damping: f32,
    pub horizontal: f32,
}

impl CoinWords {
    pub fn from_words(w: &[f32]) -> Option<Self> {
        Some(Self {
            launch: *w.first()?,
            gravity: *w.get(1)?,
            restitution: *w.get(2)?,
            stop: *w.get(3)?,
            rest: *w.get(4)?,
            damping: *w.get(5)?,
            spin_damping: *w.get(6)?,
            horizontal: *w.get(7)?,
        })
    }
}

pub fn is_coin(id: u16) -> bool {
    (135..=144).contains(&id)
}

/// A spawn request (SPEC-0021 Spawning).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spawn {
    pub id: u16,
    pub point: Vec3,
    pub placed: bool,
    pub charm: bool,
    pub s0: f32,
    pub s1: f32,
    pub delay: f32,
    /// Lifetime argument in seconds; negative takes the default.
    pub larg: f32,
    pub direction: Option<Vec3>,
    /// The placement's flag byte (+0x20) and map byte (+0x2C).
    pub flag: u8,
    pub map: u8,
    /// The placement's fixed count (+0x2A): the buy price of a flag-1 shop item (SPEC-0050).
    pub price: i16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldItem {
    pub id: u16,
    pub position: Vec3,
    pub state: u8,
    pub spin: f32,
    pub vy: f32,
    pub hs: f32,
    pub life: f32,
    pub lifetime: f32,
    pub age: f32,
    pub reference: f32,
    pub angle: f32,
    pub direction: Vec3,
    pub scale: f32,
    pub s0: f32,
    pub s1: f32,
    pub delay: f32,
    pub token: u32,
    pub flag: u8,
    pub map: u8,
    pub charm: bool,
    /// The buy price (record +0x8A; SPEC-0050).
    pub price: i16,
    /// Food the hero dropped for a petpet (SPEC-0047 Feeding): reserved, never collected.
    pub reserved: bool,
}

impl WorldItem {
    /// SPEC-0021 Pickup timing gate.
    pub fn ready(&self) -> bool {
        match self.state {
            1 => true,
            0 | 2 => false,
            _ => self.age > if is_coin(self.id) { 0.2 } else { 0.8 },
        }
    }

    /// The drawn scale: 0 while the delay runs.
    pub fn drawn_scale(&self) -> f32 {
        if self.delay > 0.0 { 0.0 } else { self.scale }
    }
}

/// Uniform draws the pool asks for: a float in [lo, hi).
pub type Draw<'a> = &'a mut dyn FnMut(f32, f32) -> f32;

#[derive(Clone, Debug, PartialEq)]
pub struct ItemPool {
    pub slots: Vec<Option<WorldItem>>,
    counter: u32,
}

impl Default for ItemPool {
    fn default() -> Self {
        Self { slots: vec![None; SLOTS], counter: 1 }
    }
}

impl ItemPool {
    pub fn live(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    /// SPEC-0021 Spawning, steps 1 and 3 to 10 (no carry limit: the text database is not
    /// read). `ground(x, z, ceiling)` is the ground probe. Returns false when nothing spawns.
    pub fn spawn(&mut self, s: Spawn, ground: &dyn Fn(f32, f32, f32) -> Option<f32>, draw: Draw) -> bool {
        if s.id > 322 {
            return false;
        }
        let Some(slot) = self.slots.iter_mut().find(|x| x.is_none()) else { return false };
        self.counter += 1;
        let token = self.counter;
        let p = s.point;
        let lifetime = if s.larg >= 0.0 {
            s.larg
        } else if s.placed {
            f32::INFINITY
        } else if s.charm {
            15.2
        } else {
            11.2
        };
        let reference = if s.placed { p[1] } else { ground(p[0], p[2], p[1] + 8.0).unwrap_or(p[1]) + 0.2 };
        let (mut spin, mut vy) = (0.0, 0.0);
        if s.charm {
            let sign = if draw(0.0, 2.0) < 1.0 { -1.0 } else { 1.0 };
            spin = sign * draw(0.4 * std::f32::consts::PI, 0.9 * std::f32::consts::PI);
            vy = draw(2.0, 6.0);
        } else if !is_coin(s.id) {
            spin = 1.2566;
        }
        let state = if s.placed && !s.charm {
            1
        } else if is_coin(s.id) {
            5
        } else {
            2
        };
        let direction = match s.direction {
            Some(d) => d,
            None => {
                let (x, z) = (draw(-1.0, 1.0), draw(-1.0, 1.0));
                let l = x.hypot(z);
                if l > 0.0 { [x / l, 0.0, z / l] } else { [0.0; 3] }
            }
        };
        *slot = Some(WorldItem {
            id: s.id,
            position: p,
            state,
            spin,
            vy,
            hs: 0.0,
            life: 1.0,
            lifetime,
            age: 0.0,
            reference,
            angle: 0.0,
            direction,
            scale: s.s0,
            s0: s.s0,
            s1: s.s1,
            delay: s.delay,
            token,
            flag: s.flag,
            price: s.price,
            map: s.map,
            charm: s.charm,
            reserved: false,
        });
        true
    }

    /// SPEC-0021 Per-tick step (game-state code 0: items never mature).
    /// Returns the ids of the dropped items (finite lifetime) whose fade ended this tick, for
    /// the important-item give (SPEC-0021 Important-item give).
    pub fn tick(&mut self, coins: &CoinWords, ground: &dyn Fn(f32, f32, f32) -> Option<f32>, draw: Draw) -> Vec<u16> {
        let mut expired = Vec::new();
        for slot in &mut self.slots {
            let Some(it) = slot.as_mut() else { continue };
            if it.state == 1 {
                continue;
            }
            if it.age > it.lifetime {
                it.life -= 2.12 * DT;
                if it.life < 0.0 {
                    if it.lifetime.is_finite() {
                        expired.push(it.id);
                    }
                    *slot = None;
                    continue;
                }
            }
            if it.delay > 0.0 {
                it.delay -= DT;
            } else {
                it.age += DT;
            }
            if is_coin(it.id) {
                coin_step(it, coins, ground, draw);
            } else if it.charm {
                charm_step(it, draw);
            } else {
                hover_step(it);
            }
            it.angle += it.spin * DT;
            if it.scale != it.s1 {
                it.scale = if it.age > 0.5 { it.s1 } else { it.s0 + 2.0 * (it.s1 - it.s0) * it.age };
            }
        }
        expired
    }

    /// Pickup steps 1, 3 and 5: the nearest ready item, collected within base + radius, or an
    /// important item (by `important`) at any distance below 25 once its age is above 0.4 x L
    /// (the range collect).
    pub fn collectable(&self, p: Vec3, base: f32, radius: f32, important: &dyn Fn(u16) -> bool) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, it) in self.slots.iter().enumerate() {
            let Some(it) = it else { continue };
            if !it.ready() || it.token == 0 || it.reserved {
                continue;
            }
            let d = ((it.position[0] - p[0]).powi(2) + (it.position[1] - p[1]).powi(2) + (it.position[2] - p[2]).powi(2)).sqrt();
            if best.is_none_or(|b| d < b.1) {
                best = Some((i, d));
            }
        }
        let (i, d) = best?;
        let it = self.slots[i].as_ref()?;
        let range = important(it.id) && it.age > 0.4 * it.lifetime;
        (it.flag != 1 && d < 25.0 && (d < base + radius || range)).then_some(i)
    }

    pub fn take(&mut self, i: usize) -> Option<WorldItem> {
        self.slots.get_mut(i)?.take()
    }
}

fn hover_step(it: &mut WorldItem) {
    let h = it.position[1] - it.reference;
    match it.state {
        2 => {
            let step = 2.08 - h;
            it.position[1] += step;
            if (0.0..4.8 * DT).contains(&step) || (-7.8 * DT < step && step < 0.0) {
                it.state = 3;
            }
        }
        3 => {
            if h > 2.08 {
                it.state = 4;
            } else {
                let v = 0.75 * (1.3 - ((h - 1.9).abs() * 5.5556).min(1.0));
                it.position[1] += v * DT;
            }
        }
        _ => {
            if h < 1.72 {
                it.state = 3;
            } else {
                let v = (0.75 * ((h - 1.9).abs() * 5.5556 - 1.3)).max(-8.75);
                it.position[1] += v * DT;
            }
        }
    }
}

fn charm_step(it: &mut WorldItem, draw: Draw) {
    let h = it.position[1] - it.reference;
    if it.state == 2 {
        let step = 1.78 - h;
        it.position[1] += step;
        if (0.0..4.8 * DT).contains(&step) || (-7.8 * DT < step && step < 0.0) {
            it.state = 3;
        }
        return;
    }
    it.vy -= 12.0 * DT;
    if h < 0.0 && it.vy < 0.0 {
        let sign = if draw(0.0, 2.0) < 1.0 { -1.0 } else { 1.0 };
        it.spin = sign * draw(0.4 * std::f32::consts::PI, 0.9 * std::f32::consts::PI);
        let v = draw(2.0, 6.0);
        it.vy = if it.flag == 1 { v.min(3.5) } else { v };
        it.state = 4;
    }
    it.position[1] += it.vy * DT;
}

fn coin_step(it: &mut WorldItem, c: &CoinWords, ground: &dyn Fn(f32, f32, f32) -> Option<f32>, draw: Draw) {
    it.vy += c.gravity * DT;
    let p = it.position;
    let g = ground(p[0], p[2], p[1] + 2.0).unwrap_or(f32::MIN);
    let h = p[1] - g;
    if it.state == 5 {
        if it.delay <= 0.0 {
            it.vy = draw(c.launch - 2.0, c.launch + 2.0);
            it.hs = draw(0.5 * c.horizontal, 1.5 * c.horizontal);
            it.spin = draw(0.3, 0.5) * std::f32::consts::PI;
            it.state = 2;
        } else {
            it.vy = 0.0;
            it.hs = 0.0;
            it.spin = 0.0;
        }
    } else if h <= c.rest && it.vy <= 0.0 {
        it.vy *= c.restitution;
        if it.vy.abs() < c.stop {
            it.vy = 0.0;
        }
        it.spin *= c.spin_damping;
        it.hs *= c.damping;
    }
    if it.vy.abs() < c.stop && h <= c.rest * 1.1 {
        it.position[1] = g + c.rest;
    }
    if it.vy < 0.0 && matches!(it.state, 0 | 2) {
        it.state = 3;
    }
    it.position[1] += it.vy * DT;
    for k in [0, 2] {
        it.position[k] += it.direction[k] * it.hs * DT;
    }
}

/// One drop group of resource 529 (money chance, range and its weighted sub-table entries).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub chance: i32,
    pub min: i32,
    pub max: i32,
    pub entries: Vec<(i16, i16)>,
}

/// What a drop action yields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drop {
    Coins(i32),
    Item(u16),
    Nothing,
}

/// SPEC-0021 Drop and money flow. `index(n)` draws 0..n-1 and `range(a, b)` a..=b.
pub fn drop_roll(group: Option<&Group>, subs: &[Vec<(u16, i16)>], count: i16, index: &mut dyn FnMut(u32) -> u32, range: &mut dyn FnMut(i32, i32) -> i32) -> Drop {
    let mut amount = 0;
    if let Some(g) = group {
        let r = index(100) as i32;
        if r < g.chance {
            amount = range(g.min, g.max);
        }
    }
    if amount <= 0 && count > 0 {
        amount = count as i32;
    }
    if amount > 0 {
        return Drop::Coins(amount);
    }
    item_roll(group, subs, index)
}

/// SPEC-0021 Drop and money flow steps 4 and 5, the item roll alone (a scripted add of a
/// negative-key placement rolls only this: no money roll, no coins).
pub fn item_roll(group: Option<&Group>, subs: &[Vec<(u16, i16)>], index: &mut dyn FnMut(u32) -> u32) -> Drop {
    let Some(g) = group else { return Drop::Nothing };
    let span = 100 - g.chance;
    if span <= 0 {
        return Drop::Nothing;
    }
    let t = index(span as u32) as i32;
    let mut acc = 0;
    let Some(&(sub, _)) = g.entries.iter().find(|e| {
        acc += e.1 as i32;
        acc > t
    }) else {
        return Drop::Nothing;
    };
    let Some(table) = subs.get(sub.max(0) as usize) else { return Drop::Nothing };
    let u = index(100) as i32;
    let mut acc = 0;
    table
        .iter()
        .find(|e| {
            acc += e.1 as i32;
            acc > u
        })
        .map_or(Drop::Nothing, |e| Drop::Item(e.0))
}

/// The exact greedy coin split over the denominations (100, 25, 10, 5, 1), as coin ids.
pub fn coin_split(amount: i32, denominations: &[i32], ids: &[u16]) -> Vec<u16> {
    let mut out = Vec::new();
    let mut left = amount.max(0);
    for (&d, &id) in denominations.iter().zip(ids) {
        if d <= 0 {
            continue;
        }
        for _ in 0..left / d {
            out.push(id);
        }
        left %= d;
    }
    out
}

/// The luck gate: r = int[0, 100]; vetoed when trunc(luck x 4.166667) <= r.
pub fn luck_passes(luck: i32, r: i32) -> bool {
    ((luck as f32 * 4.166667) as i32) > r
}

#[cfg(test)]
mod tests {
    use super::*;

    const COINS: CoinWords = CoinWords { launch: 16.0, gravity: -40.0, restitution: -0.65, stop: 1.0, rest: 0.6, damping: 0.98, spin_damping: 0.98, horizontal: 2.0 };

    fn flat(_: f32, _: f32, _: f32) -> Option<f32> {
        Some(0.0)
    }

    fn spawn_one(id: u16, charm: bool, direction: Option<Vec3>, delay: f32, draws: &[f32]) -> (ItemPool, Vec<f32>) {
        let mut pool = ItemPool::default();
        let mut q: Vec<f32> = draws.to_vec();
        let mut d = |_: f32, _: f32| if q.is_empty() { 0.5 } else { q.remove(0) };
        let s = Spawn { id, point: [0.0, 1.0, 0.0], placed: false, charm, s0: 0.25, s1: 1.0, delay, larg: -1.0, direction, flag: 0, map: 0, price: 0 };
        assert!(pool.spawn(s, &flat, &mut d));
        (pool, q)
    }

    #[test]
    fn coin_flight_vector() {
        // SPEC-0021 vector 23: coin 137 at (0, 1, 0), direction (0.7071, 0, -0.7071), launch
        // vy 16, hs 2, spin 0.4 pi.
        let (mut pool, _) = spawn_one(137, false, Some([0.7071, 0.0, -0.7071]), 0.0, &[]);
        let mut draws = vec![16.0, 2.0, 0.4];
        let mut d = |_: f32, _: f32| if draws.is_empty() { 0.0 } else { draws.remove(0) };
        pool.tick(&COINS, &flat, &mut d);
        let c = pool.slots[0].unwrap();
        assert_eq!(c.state, 2);
        assert!((c.position[1] - 1.2667).abs() < 1e-3 && (c.position[0] - 0.0236).abs() < 1e-3, "{c:?}");
        for _ in 2..=24 {
            pool.tick(&COINS, &flat, &mut d);
        }
        assert_eq!(pool.slots[0].unwrap().state, 2);
        pool.tick(&COINS, &flat, &mut d);
        let c = pool.slots[0].unwrap();
        assert_eq!(c.state, 3, "the apex on tick 25");
        assert!((c.position[1] - 4.3333).abs() < 1e-2, "{c:?}");
        for _ in 26..=52 {
            pool.tick(&COINS, &flat, &mut d);
        }
        let c = pool.slots[0].unwrap();
        assert!((c.vy - 11.70).abs() < 0.02 && (c.hs - 1.96).abs() < 1e-3, "first contact on tick 52: {c:?}");
        for _ in 53..=160 {
            pool.tick(&COINS, &flat, &mut d);
        }
        let c = pool.slots[0].unwrap();
        assert!((c.position[1] - 0.6).abs() < 1e-4 && c.vy == 0.0, "at rest by tick 160: {c:?}");
        assert!((c.hs - 1.6341).abs() < 2e-3, "{c:?}");
    }

    #[test]
    fn hovering_potion_vector() {
        // Vector 25: potion 146 at (0, 1, 0), reference 0.2.
        let (mut pool, _) = spawn_one(146, false, None, 0.0, &[0.3, 0.4]);
        let mut d = |_: f32, _: f32| 0.0;
        pool.tick(&COINS, &flat, &mut d);
        let p = pool.slots[0].unwrap();
        assert!((p.position[1] - 2.28).abs() < 1e-5 && p.state == 2);
        pool.tick(&COINS, &flat, &mut d);
        assert_eq!(pool.slots[0].unwrap().state, 3);
        pool.tick(&COINS, &flat, &mut d);
        pool.tick(&COINS, &flat, &mut d);
        assert_eq!(pool.slots[0].unwrap().state, 4);
        for _ in 5..=48 {
            pool.tick(&COINS, &flat, &mut d);
        }
        assert!(!pool.slots[0].unwrap().ready(), "not ready after 48 ticks");
        pool.tick(&COINS, &flat, &mut d);
        assert!(pool.slots[0].unwrap().ready(), "ready from tick 49");
        for _ in 50..=672 {
            pool.tick(&COINS, &flat, &mut d);
        }
        assert_eq!(pool.slots[0].unwrap().life, 1.0);
        pool.tick(&COINS, &flat, &mut d);
        assert!((pool.slots[0].unwrap().life - 0.9647).abs() < 1e-3, "the fade starts on tick 673");
        for _ in 674..=700 {
            pool.tick(&COINS, &flat, &mut d);
        }
        assert!(pool.slots[0].is_some());
        pool.tick(&COINS, &flat, &mut d);
        assert!(pool.slots[0].is_none(), "freed on tick 701");
    }

    #[test]
    fn hopping_charm_vector() {
        // Vector 26: charm 172 with spawn draws spin +2.0420 (sign draw 1.5 = +) and vy 3.0;
        // the first contact draws a negative sign and a bounce speed 4.0.
        let (mut pool, _) = spawn_one(172, true, None, 0.0, &[1.5, 2.0420, 3.0, 0.2, 0.2]);
        let mut draws = vec![0.5, 2.0420, 4.0];
        let mut d = |_: f32, _: f32| if draws.is_empty() { 3.0 } else { draws.remove(0) };
        pool.tick(&COINS, &flat, &mut d);
        assert!((pool.slots[0].unwrap().position[1] - 1.98).abs() < 1e-5);
        pool.tick(&COINS, &flat, &mut d);
        assert_eq!(pool.slots[0].unwrap().state, 3);
        let mut contact = None;
        for t in 3..=60 {
            pool.tick(&COINS, &flat, &mut d);
            if contact.is_none() && pool.slots[0].unwrap().state == 4 {
                contact = Some(t);
            }
        }
        assert_eq!(contact, Some(54));
    }

    #[test]
    fn rolls_split_and_gates() {
        let subs: Vec<Vec<(u16, i16)>> = vec![vec![(146, 30)], vec![], vec![(152, 100)], vec![], vec![], vec![], vec![(172, 75), (173, 25)]];
        let g1 = Group { chance: 40, min: 300, max: 800, entries: vec![(5, 25), (2, 20), (6, 15)] };
        // Vector 4: r = 73 (no money), t = 30: sub-table 2, item 152.
        let mut seq = vec![73u32, 30, 10];
        let mut index = |_: u32| seq.remove(0);
        let mut range = |a: i32, _: i32| a;
        assert_eq!(drop_roll(Some(&g1), &subs, 0, &mut index, &mut range), Drop::Item(152));
        // Vector 5: t = 50 (sub-table 6), u = 80: item 173.
        let mut seq = vec![73u32, 50, 80];
        let mut index = |_: u32| seq.remove(0);
        assert_eq!(drop_roll(Some(&g1), &subs, 0, &mut index, &mut range), Drop::Item(173));
        // Vector 3: group 0 money 62 splits into 25, 25, 10, 1, 1.
        let den = [100, 25, 10, 5, 1];
        let ids = [139u16, 138, 137, 136, 135];
        assert_eq!(coin_split(62, &den, &ids), vec![138, 138, 137, 135, 135]);
        // A fixed count pays coins when the money roll fails; group -1 has no rolls.
        let mut index = |_: u32| 99;
        assert_eq!(drop_roll(None, &subs, 150, &mut index, &mut range), Drop::Coins(150));
        assert_eq!(coin_split(150, &den, &ids), vec![139, 138, 138]);
        assert_eq!(drop_roll(None, &subs, 0, &mut index, &mut range), Drop::Nothing);
        // Luck vectors 1 and 2.
        assert!(luck_passes(12, 49) && !luck_passes(12, 50));
        assert!(!luck_passes(0, 0) && !luck_passes(24, 100) && luck_passes(24, 99));
    }

    #[test]
    fn important_range_collect_and_expiry() {
        // An important dropped item (L 11.2 s) 10 units away: collected only once its age is
        // above 0.4 x L; its fade's end reports it for the give.
        let (mut pool, _) = spawn_one(18, false, None, 0.0, &[0.3, 0.4]);
        let l = pool.slots[0].unwrap().lifetime;
        {
            let it = pool.slots[0].as_mut().unwrap();
            it.state = 3;
            it.age = 0.4 * l;
            it.position = [10.0, 0.0, 0.0];
        }
        let imp = |id: u16| id == 18;
        assert_eq!(pool.collectable([0.0; 3], 1.5, 1.175, &imp), None);
        pool.slots[0].as_mut().unwrap().age = 0.4 * l + 0.1;
        assert_eq!(pool.collectable([0.0; 3], 1.5, 1.175, &imp), Some(0));
        assert_eq!(pool.collectable([0.0; 3], 1.5, 1.175, &|_| false), None);
        let mut d = |_: f32, _: f32| 0.5;
        let flat = |_: f32, _: f32, _: f32| Some(0.0);
        pool.slots[0].as_mut().unwrap().age = l + 1.0;
        let mut expired = Vec::new();
        for _ in 0..40 {
            expired.extend(pool.tick(&COINS, &flat, &mut d));
        }
        assert_eq!(expired, vec![18]);
    }

    #[test]
    fn pickup_gate_and_collect_distance() {
        let (mut pool, _) = spawn_one(146, false, None, 0.0, &[0.3, 0.4]);
        {
            let it = pool.slots[0].as_mut().unwrap();
            it.state = 3;
            it.age = 0.81;
            it.position = [2.0, 1.7, 0.0];
        }
        assert_eq!(pool.collectable([0.0; 3], 1.5, 1.175, &|_| false), Some(0), "d = 2.625 < 2.675");
        pool.slots[0].as_mut().unwrap().position = [2.0, 1.8, 0.0];
        assert_eq!(pool.collectable([0.0; 3], 1.5, 1.175, &|_| false), None, "d = 2.691");
        pool.slots[0].as_mut().unwrap().age = 0.8;
        pool.slots[0].as_mut().unwrap().position = [1.0, 0.0, 0.0];
        assert_eq!(pool.collectable([0.0; 3], 1.5, 1.175, &|_| false), None, "age 0.8 is not ready");
        let mut full = ItemPool::default();
        let mut d = |_: f32, _: f32| 0.5;
        let s = Spawn { id: 146, point: [0.0; 3], placed: true, charm: false, s0: 1.0, s1: 1.0, delay: 0.0, larg: -1.0, direction: None, flag: 0, map: 0, price: 0 };
        for _ in 0..SLOTS {
            assert!(full.spawn(s, &flat, &mut d));
        }
        assert!(!full.spawn(s, &flat, &mut d), "a 51st drop spawns nothing");
    }
}
