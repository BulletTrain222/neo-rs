//! The save file (SPEC-0025): four 0x3100-byte slots in a 0xC400-byte file, the slot field map
//! with its empty template and validity rules, the world-key lookups, the progress tier, the
//! date text, the hero maxima rules of a load, the status-timer conversions, the picker cursor
//! of SPEC-0035 section 7.1 and the per-tick save-point step.
//!
//! The slot is a plain byte image (little-endian fields). The writer changes only the fields it
//! sets, so bytes nothing writes keep their old values, as in the original.

pub const SLOT_SIZE: usize = 0x3100;
pub const FILE_SIZE: usize = 0xC400;
pub const SLOTS: usize = 4;
/// Progress flags 0 to 742 are valid; a flag at or above this reads as clear.
pub const FLAGS: u16 = 743;
/// Rows of the per-world tables and bits of the once-per-area sets.
pub const WORLDS: usize = 193;
/// Item ids with a count byte.
pub const COUNTS: usize = 191;

/// Slot field offsets (Layout, Slot field map).
pub mod at {
    pub const POSITION: usize = 0x000;
    pub const FACING: usize = 0x010;
    pub const ACTIVE_HERO: usize = 0x020;
    pub const KEYS: usize = 0x024;
    pub const FLAGS: usize = 0x030;
    pub const MOVIES: usize = 0x090;
    pub const WALLET: usize = 0x200;
    pub const STASH: usize = 0x204;
    pub const LUCK: usize = 0x208;
    pub const TOR_HEALTH: usize = 0x20C;
    pub const TOR_MAGIC: usize = 0x210;
    pub const PET_TYPE: usize = 0x214;
    pub const PET_TIME: usize = 0x218;
    pub const PET_FULL: usize = 0x21C;
    pub const CONTAINER_A: usize = 0x220;
    pub const CONTAINER_B: usize = 0x540;
    pub const HEALTH: usize = 0x860;
    pub const MAGIC: usize = 0x868;
    pub const STATUS: usize = 0x870;
    pub const STATUS_WORDS: usize = 0x8E0;
    pub const T1: usize = 0x900;
    pub const T2: usize = 0x15D1;
    pub const ONCE: usize = 0x2EB2;
    pub const TREASURE: usize = 0x2F2F;
    pub const MAPS: usize = 0x2F4F;
    pub const LOCATION: usize = 0x2F6F;
    pub const GUARD: usize = 0x2FAA;
    pub const DATE: usize = 0x2FAB;
    pub const TIER: usize = 0x2FBA;
    pub const COUNTS: usize = 0x2FBB;
    pub const SWAP_MODE: usize = 0x307C;
    pub const CLOAK: usize = 0x3080;
    pub const OPTIONS: usize = 0x3084;
    pub const MOTES: usize = 0x30A4;
    pub const ROBERTA_HEALTH: usize = 0x30BC;
    pub const ROBERTA_MAGIC: usize = 0x30C0;
}

/// The six entry groups of one item container, in save and load order (Item lists): offset in
/// the container and capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// Kind 0, the item grid (every category without its own list).
    General,
    /// Kind 7, quest items.
    Quest,
    /// Kind 8, charms.
    Charm,
    /// Kind 1, the weapon family (entry h is hero h's).
    Weapon,
    /// The tail: hero 0's then hero 1's body outfit entry.
    Tail,
    /// Kind 2, the shield family.
    Shield,
}

impl Group {
    pub const ORDER: [Group; 6] = [Group::General, Group::Quest, Group::Charm, Group::Weapon, Group::Tail, Group::Shield];

    pub fn offset(self) -> usize {
        match self {
            Group::General => 0x000,
            Group::Quest => 0x0A0,
            Group::Charm => 0x280,
            Group::Weapon => 0x2F0,
            Group::Tail => 0x300,
            Group::Shield => 0x310,
        }
    }

    pub fn capacity(self) -> usize {
        match self {
            Group::General => 20,
            Group::Quest => 60,
            Group::Charm => 14,
            Group::Weapon | Group::Tail | Group::Shield => 2,
        }
    }

    /// The group an item of a text-table category goes to (SPEC-0021 Party inventory lists);
    /// category 9 (the cloak entries) is not stored as a list, and category 5 is the tail.
    pub fn of_category(category: u32) -> Option<Group> {
        match category {
            7 => Some(Group::Quest),
            8 => Some(Group::Charm),
            1 => Some(Group::Weapon),
            2 => Some(Group::Shield),
            5 => Some(Group::Tail),
            9 => None,
            _ => Some(Group::General),
        }
    }
}

/// One slot's bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot(pub Vec<u8>);

impl Default for Slot {
    fn default() -> Self {
        Self::empty()
    }
}

impl Slot {
    /// The empty-slot template (Layout, Empty-slot template).
    pub fn empty() -> Self {
        let mut s = Slot(vec![0; SLOT_SIZE]);
        for k in 0..3 {
            s.set_key(k, -1);
        }
        s.set_i32(at::PET_TYPE, -1);
        s.set_text(at::LOCATION, 59, b"Empty");
        s.0[at::DATE] = b' ';
        s.0[at::DATE + 1] = 0;
        s.set_u32(at::SWAP_MODE, 2);
        for h in 0..2 {
            for i in 0..3 {
                s.set_mote(h, i, -1);
            }
        }
        s
    }

    pub fn u32(&self, o: usize) -> u32 {
        u32::from_le_bytes(self.0[o..o + 4].try_into().unwrap())
    }

    pub fn i32(&self, o: usize) -> i32 {
        self.u32(o) as i32
    }

    pub fn f32(&self, o: usize) -> f32 {
        f32::from_bits(self.u32(o))
    }

    pub fn set_u32(&mut self, o: usize, v: u32) {
        self.0[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }

    pub fn set_i32(&mut self, o: usize, v: i32) {
        self.set_u32(o, v as u32);
    }

    pub fn set_f32(&mut self, o: usize, v: f32) {
        self.set_u32(o, v.to_bits());
    }

    pub fn vec4(&self, o: usize) -> [f32; 4] {
        std::array::from_fn(|k| self.f32(o + 4 * k))
    }

    pub fn set_vec4(&mut self, o: usize, v: [f32; 4]) {
        for (k, x) in v.iter().enumerate() {
            self.set_f32(o + 4 * k, *x);
        }
    }

    /// Key A, B or C (k = 0, 1, 2): a saved world id, -1 for none.
    pub fn key(&self, k: usize) -> i32 {
        self.i32(at::KEYS + 4 * k)
    }

    pub fn set_key(&mut self, k: usize, v: i32) {
        self.set_i32(at::KEYS + 4 * k, v);
    }

    /// The picker's empty test: key A exactly -1 (Validity rules).
    pub fn is_empty(&self) -> bool {
        self.key(0) == -1
    }

    /// The save writer's present flag: the sign bit of key A is clear.
    pub fn present(&self) -> bool {
        self.key(0) >= 0
    }

    /// Progress flag n: bit (n AND 7) of byte (n >> 3); flags from 743 read as clear.
    pub fn flag(&self, n: u16) -> bool {
        n < FLAGS && self.0[at::FLAGS + (n as usize >> 3)] & (1 << (n & 7)) != 0
    }

    pub fn set_flag(&mut self, n: u16, on: bool) {
        if n >= FLAGS {
            return;
        }
        let b = &mut self.0[at::FLAGS + (n as usize >> 3)];
        if on {
            *b |= 1 << (n & 7);
        } else {
            *b &= !(1 << (n & 7));
        }
    }

    /// Clear all 743 flags (the 93-byte bitmap).
    pub fn clear_flags(&mut self) {
        self.0[at::FLAGS..at::FLAGS + 93].fill(0);
    }

    /// Hero h's maximum health and magic words (0x20C and 0x210 for Tor, 0x30BC and 0x30C0 for
    /// Roberta).
    pub fn maximum_words(&self, h: usize) -> (i32, i32) {
        if h == 0 {
            (self.i32(at::TOR_HEALTH), self.i32(at::TOR_MAGIC))
        } else {
            (self.i32(at::ROBERTA_HEALTH), self.i32(at::ROBERTA_MAGIC))
        }
    }

    pub fn set_maximum_words(&mut self, h: usize, health: i32, magic: i32) {
        let (a, b) = if h == 0 { (at::TOR_HEALTH, at::TOR_MAGIC) } else { (at::ROBERTA_HEALTH, at::ROBERTA_MAGIC) };
        self.set_i32(a, health);
        self.set_i32(b, magic);
    }

    /// Entry `index` of a group of container c (0 = A, the party; 1 = B, the stash): (id, value).
    pub fn entry(&self, c: usize, g: Group, index: usize) -> (i32, i32) {
        let o = Self::entry_at(c, g, index);
        (self.i32(o), self.i32(o + 4))
    }

    /// Write an entry: the value always, the id only when there is an item (Item lists, Save
    /// rule: an entry with no item record keeps its old id word).
    pub fn set_entry(&mut self, c: usize, g: Group, index: usize, id: Option<u16>, value: i32) {
        let o = Self::entry_at(c, g, index);
        if let Some(id) = id {
            self.set_i32(o, id as i16 as i32);
        }
        self.set_i32(o + 4, value);
    }

    fn entry_at(c: usize, g: Group, index: usize) -> usize {
        assert!(index < g.capacity());
        let base = if c == 0 { at::CONTAINER_A } else { at::CONTAINER_B };
        base + g.offset() + 8 * index
    }

    /// The re-add order of a load (Loading, step 8): each group in order, containers A and B
    /// interleaved by entry; pairs with a value below 1 are skipped. Items: (container, group,
    /// entry index, id, value).
    pub fn adds(&self) -> Vec<(usize, Group, usize, i32, i32)> {
        let mut out = Vec::new();
        for g in Group::ORDER {
            for i in 0..g.capacity() {
                for c in 0..2 {
                    let (id, v) = self.entry(c, g, i);
                    if v >= 1 {
                        out.push((c, g, i, id, v));
                    }
                }
            }
        }
        out
    }

    /// Current health and magic of hero h.
    pub fn pools(&self, h: usize) -> (i32, i32) {
        (self.i32(at::HEALTH + 4 * h), self.i32(at::MAGIC + 4 * h))
    }

    pub fn set_pools(&mut self, h: usize, health: i32, magic: i32) {
        self.set_i32(at::HEALTH + 4 * h, health);
        self.set_i32(at::MAGIC + 4 * h, magic);
    }

    /// Status timer i of hero h, in seconds.
    pub fn status(&self, h: usize, i: usize) -> f32 {
        self.f32(at::STATUS + 0x38 * h + 4 * i)
    }

    pub fn set_status(&mut self, h: usize, i: usize, seconds: f32) {
        self.set_f32(at::STATUS + 0x38 * h + 4 * i, seconds);
    }

    /// Hero h's status word k (0 fire amount, 1 fire period, 2 poison amount, 3 poison period).
    pub fn status_word(&self, h: usize, k: usize) -> i32 {
        self.i32(at::STATUS_WORDS + 8 * k + 4 * h)
    }

    pub fn set_status_word(&mut self, h: usize, k: usize, v: i32) {
        self.set_i32(at::STATUS_WORDS + 8 * k + 4 * h, v);
    }

    /// Once-per-area set s (0 to 4: items 19, 20, 151, 174, 154), bit = world index.
    pub fn once(&self, s: usize, world: usize) -> bool {
        world < WORLDS && self.0[at::ONCE + 25 * s + (world >> 3)] & (1 << (world & 7)) != 0
    }

    pub fn set_once(&mut self, s: usize, world: usize, on: bool) {
        if world >= WORLDS {
            return;
        }
        let b = &mut self.0[at::ONCE + 25 * s + (world >> 3)];
        if on {
            *b |= 1 << (world & 7);
        } else {
            *b &= !(1 << (world & 7));
        }
    }

    /// A zero-terminated text field (up to `len` bytes).
    pub fn text(&self, o: usize, len: usize) -> &[u8] {
        let f = &self.0[o..o + len];
        &f[..f.iter().position(|&c| c == 0).unwrap_or(len)]
    }

    /// Write a text with its zero terminator, cut to the field.
    pub fn set_text(&mut self, o: usize, len: usize, t: &[u8]) {
        let n = t.len().min(len - 1).min(len);
        self.0[o..o + n].copy_from_slice(&t[..n]);
        self.0[o + n] = 0;
    }

    pub fn location(&self) -> &[u8] {
        self.text(at::LOCATION, 59)
    }

    /// The location name: up to 59 characters, then the guard byte 0x2FAA is zeroed.
    pub fn set_location(&mut self, t: &[u8]) {
        let n = t.len().min(59);
        self.0[at::LOCATION..at::LOCATION + n].copy_from_slice(&t[..n]);
        if n < 59 {
            self.0[at::LOCATION + n] = 0;
        }
        self.0[at::GUARD] = 0;
    }

    pub fn date(&self) -> &[u8] {
        self.text(at::DATE, 15)
    }

    pub fn set_date(&mut self, t: &[u8]) {
        self.set_text(at::DATE, 15, t);
    }

    /// The progress tier byte, read as a signed byte (the picker's row type is this plus 2).
    pub fn tier(&self) -> i8 {
        self.0[at::TIER] as i8
    }

    pub fn mote(&self, h: usize, i: usize) -> i32 {
        self.i32(at::MOTES + 12 * h + 4 * i)
    }

    pub fn set_mote(&mut self, h: usize, i: usize, v: i32) {
        self.set_i32(at::MOTES + 12 * h + 4 * i, v);
    }
}

/// The whole save image: four slots in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveFile {
    pub slots: [Slot; SLOTS],
}

impl Default for SaveFile {
    fn default() -> Self {
        Self::empty()
    }
}

impl SaveFile {
    /// Four empty slots (start-up, and before each scan).
    pub fn empty() -> Self {
        Self { slots: std::array::from_fn(|_| Slot::empty()) }
    }

    /// A read must give exactly 0xC400 bytes (Validity rules; record 14 otherwise).
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != FILE_SIZE {
            return Err(format!("Save file is {} bytes, not {FILE_SIZE}", bytes.len()));
        }
        Ok(Self { slots: std::array::from_fn(|k| Slot(bytes[k * SLOT_SIZE..(k + 1) * SLOT_SIZE].to_vec())) })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.slots.iter().flat_map(|s| s.0.iter().copied()).collect()
    }

    /// A slot by index; 4 or more has no slot.
    pub fn slot(&self, k: usize) -> Option<&Slot> {
        self.slots.get(k)
    }

    pub fn filled(&self) -> [bool; SLOTS] {
        std::array::from_fn(|k| !self.slots[k].is_empty())
    }
}

/// Forward lookup: the saved id at the first entry whose key equals `key` compared as a 16-bit
/// signed value; 0xFFFF when none matches (Lookups).
pub fn forward(keys: &[u32], ids: &[u32], key: i32) -> u32 {
    keys.iter()
        .position(|&k| k as i16 == key as i16)
        .and_then(|i| ids.get(i).copied())
        .unwrap_or(0xFFFF)
}

/// Reverse lookup: the position of the first entry holding the saved id; the table length (193)
/// when none does.
pub fn reverse(ids: &[u32], id: u32) -> usize {
    ids.iter().position(|&v| v == id).unwrap_or(ids.len())
}

/// The world index of a world key: the reverse lookup of the forward lookup.
pub fn world_index(keys: &[u32], ids: &[u32], key: i32) -> usize {
    reverse(ids, forward(keys, ids, key))
}

/// The progress tier (the act number): 4 for flag 332, 3 for 248, 2 for 220, else 1.
pub fn tier(flag: impl Fn(u16) -> bool) -> u8 {
    if flag(0x14C) {
        4
    } else if flag(0xF8) {
        3
    } else if flag(0xDC) {
        2
    } else {
        1
    }
}

/// The date text "%2d.%02d.%02d %2d:%02d": month, day, year (two digits), hour, minute.
pub fn date_text(month: u32, day: u32, year: u32, hour: u32, minute: u32) -> String {
    format!("{month:2}.{day:02}.{:02} {hour:2}:{minute:02}", year % 100)
}

/// The growth words (SPEC-0036 Hero growth; boot 0x319BC0 to 0x319BEC): starts, gains and caps,
/// each in the order Tor health, Roberta health, Tor magic, Roberta magic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Growth {
    pub start: [i32; 4],
    pub gain: [i32; 4],
    pub cap: [i32; 4],
}

impl Growth {
    pub fn from_words(w: &[u32]) -> Option<Self> {
        if w.len() < 12 {
            return None;
        }
        let row = |r: usize| std::array::from_fn(|k| w[4 * r + k] as i32);
        Some(Self { start: row(0), gain: row(1), cap: row(2) })
    }
}

/// The maxima a load sets (Hero maxima and restore, rules 1 to 4): [hero][health, magic].
pub fn maxima(slot: &Slot, g: &Growth) -> [[i32; 2]; 2] {
    let within = |v: i32, cap: i32| (1..=cap).contains(&v);
    let (tor_h, tor_m) = slot.maximum_words(0);
    let (rob_h, rob_m) = slot.maximum_words(1);
    let tor_health = if within(tor_h, g.cap[0]) { tor_h } else { g.cap[0] };
    let tor_magic = if within(tor_m, g.cap[2]) { tor_m } else { g.cap[2] };
    // Roberta's fallbacks read Tor's words, banded by Tor's start and gain words.
    let banded = |own: i32, cap: i32, x: i32, x_cap: i32, start: i32, gain: i32, reference: i32, step: i32| {
        if within(own, cap) {
            own
        } else if within(x, x_cap) && step != 0 {
            start + gain * ((x - reference) / step)
        } else {
            cap
        }
    };
    let rob_health = banded(rob_h, g.cap[1], tor_h, g.cap[1], g.start[1], g.gain[1], g.start[0], g.gain[0]);
    let rob_magic = banded(rob_m, g.cap[3], tor_m, g.cap[3], g.start[3], g.gain[3], g.start[2], g.gain[2]);
    [[tor_health, tor_magic], [rob_health, rob_magic]]
}

/// A live status timer as saved: ticks x 1/60 s (an inactive -1 writes -1/60). Computed as a
/// division, which gives the spec's 1.5 for 90 ticks; a multiply by the f32 of 1/60 would give
/// 1.5000001 (the original's rounding is not confirmed).
pub fn status_seconds(ticks: i32) -> f32 {
    ticks as f32 / 60.0
}

/// The whole seconds a load passes to the status apply (truncated toward zero).
pub fn restore_seconds(saved: f32) -> i32 {
    saved.trunc() as i32
}

/// The luck a load sets on each hero: 0 plus the saved value, clamped to 0 to 24.
pub fn restore_luck(saved: i32) -> i32 {
    saved.clamp(0, 24)
}

/// The load screen's starting cursor (SPEC-0035 section 7.2): the first filled slot, or 4.
pub fn load_start(filled: [bool; SLOTS]) -> usize {
    filled.iter().position(|&f| f).unwrap_or(SLOTS)
}

/// One cursor move (section 7.1). Save mode moves by one within 0 to 3; load mode skips empty
/// slots and stays put when no filled slot lies that way.
pub fn cursor_step(cursor: usize, down: bool, load: bool, filled: [bool; SLOTS]) -> usize {
    let next = |c: usize| if down { c.checked_add(1).filter(|&n| n < SLOTS) } else { c.checked_sub(1) };
    if !load {
        return next(cursor.min(SLOTS - 1)).unwrap_or(cursor);
    }
    let mut at = cursor;
    while let Some(n) = next(at) {
        if filled[n] {
            return n;
        }
        at = n;
    }
    cursor
}

/// A save point: a kind-4 record of a world pack's own region (SPEC-0006).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavePoint {
    pub position: [f32; 3],
    pub flag: u32,
}

/// What the save-point step did this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointStep {
    /// The nearest point of each branch (zero flag, nonzero flag).
    pub chosen: [Option<usize>; 2],
    /// The glow moved to a newly chosen point this tick.
    pub glow_moved: bool,
    /// The point the glow moved to (the later branch when both moved).
    pub glow_at: Option<usize>,
    /// Cue 0xA8 is kept playing (within 15 units).
    pub cue: bool,
    /// The "Save Game" prompt shows (within 5 units, gate open).
    pub prompt: bool,
    /// Square opens the save screen; with the zero-flag branch it also sets the restart point.
    pub open: bool,
    pub restart: bool,
}

/// The region's save-point state: the remembered glow index of each branch and the away flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SavePoints {
    remembered: [Option<usize>; 2],
    pub away: bool,
}

impl SavePoints {
    /// One tick (Save points, steps 1 to 6). `gate` is the prompt gate of step 4 (no in-play
    /// sub-state, game-state code 0); `square` the press edge; `busy` the hero's busy test.
    pub fn step(&mut self, points: &[SavePoint], p: [f32; 3], gate: bool, square: bool, busy: bool) -> PointStep {
        let mut out = PointStep::default();
        let mut best = [(None, 2.5e9f32); 2];
        for (k, sp) in points.iter().enumerate().take(8) {
            let d2: f32 = (0..3).map(|i| (sp.position[i] - p[i]).powi(2)).sum();
            let b = &mut best[(sp.flag != 0) as usize];
            if d2 < b.1 {
                *b = (Some(k), d2);
            }
        }
        for branch in 0..2 {
            let (Some(k), d2) = best[branch] else { continue };
            out.chosen[branch] = Some(k);
            if self.remembered[branch] != Some(k) {
                self.remembered[branch] = Some(k);
                out.glow_moved = true;
                out.glow_at = Some(k);
            }
            if d2 < 225.0 {
                out.cue = true;
            }
            if !gate {
                continue;
            }
            if d2 < 25.0 {
                out.prompt = true;
                if square && !busy {
                    out.open = true;
                    self.away = false;
                    if branch == 0 {
                        out.restart = true;
                    }
                }
            } else {
                self.away = true;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_and_template() {
        let f = SaveFile::empty();
        let b = f.to_bytes();
        assert_eq!(b.len(), 0xC400);
        assert_eq!(&b[0x024..0x030], &[0xFF; 12]);
        assert_eq!(&b[0x2F6F..0x2F75], b"Empty\0");
        assert_eq!(b[0x2FAB], 0x20);
        assert_eq!(b[0x2FAC], 0);
        assert_eq!(f.slots[3].u32(at::SWAP_MODE), 2);
        assert_eq!(f.slots[0].mote(1, 2), -1);
        assert_eq!(at::MOTES + 12 + 8, 0x30B8);
        assert_eq!(f.slots[0].i32(at::PET_TYPE), -1);
        assert!(f.slots.iter().all(|s| s.is_empty() && !s.present()));
        // Slot offsets, and a slot index of 4 or more has no slot.
        let mut g = f.clone();
        g.slots[2].set_key(0, 11);
        let gb = g.to_bytes();
        assert_eq!(gb[0x6200 + 0x24], 11);
        assert_eq!(&gb[..0x6200], &b[..0x6200]);
        assert_eq!(&gb[0x9300..], &b[0x9300..]);
        assert!(g.slot(3).is_some() && g.slot(4).is_none());
        // Size check.
        assert!(SaveFile::parse(&gb).is_ok());
        assert!(SaveFile::parse(&gb[..50_175]).is_err());
        let mut long = gb.clone();
        long.push(0);
        assert!(SaveFile::parse(&long).is_err());
        assert_eq!(SaveFile::parse(&gb).unwrap(), g);
    }

    #[test]
    fn fields() {
        let mut s = Slot::empty();
        s.0[at::FLAGS + 41] = 0x10;
        assert!(s.flag(332) && !s.flag(333));
        assert!(!s.flag(743));
        // Hero 1 timers follow hero 0's 56-byte block.
        s.set_status(1, 0, 1.5);
        assert_eq!(s.f32(0x8A8), 1.5);
        // Tails: container A hero 1 at 0x528, container B hero 0 at 0x840.
        s.set_entry(0, Group::Tail, 1, Some(130), 1);
        assert_eq!((s.i32(0x528), s.i32(0x52C)), (130, 1));
        s.set_entry(1, Group::Tail, 0, Some(121), 1);
        assert_eq!(s.i32(0x840), 121);
        assert_eq!(at::CONTAINER_A + Group::Quest.offset(), 0x2C0);
        assert_eq!(at::CONTAINER_B + Group::Weapon.offset(), 0x830);
        assert_eq!(at::CONTAINER_A + Group::Shield.offset(), 0x530);
        // An empty pair over an old id keeps the id and writes the count.
        s.set_i32(0x220, 50);
        s.set_entry(0, Group::General, 0, None, 0);
        assert_eq!((s.i32(0x220), s.i32(0x224)), (50, 0));
        assert!(s.adds().iter().all(|a| a.3 != 50));
        // Load order: groups in order, A and B interleaved.
        s.set_entry(1, Group::General, 0, Some(7), 2);
        s.set_entry(0, Group::General, 1, Some(8), 1);
        s.set_entry(0, Group::Charm, 0, Some(164), 100);
        let order: Vec<(usize, i32)> = s.adds().iter().map(|a| (a.0, a.3)).collect();
        assert_eq!(order, vec![(1, 7), (0, 8), (0, 164), (1, 121), (0, 130)]);
        // Once-per-area: Golden Negg in world index 68 is byte 0x2EEC bit 4.
        s.set_once(2, 68, true);
        assert_eq!(s.0[0x2EEC], 0x10);
        assert!(s.once(2, 68) && !s.once(2, 127));
        s.set_once(2, 193, true);
        assert!(!s.once(2, 193));
        // Location with its guard byte; a 59-character name has no terminator in its field.
        s.set_location(b"Meridell Farm");
        assert_eq!(s.location(), b"Meridell Farm");
        let long = [b'x'; 70];
        s.set_location(&long);
        assert_eq!(s.location().len(), 59);
        assert_eq!(s.0[at::GUARD], 0);
    }

    #[test]
    fn lookups() {
        let (keys, ids) = ([700u32, 701], [0u32, 5]);
        assert_eq!(reverse(&ids, 5), 1);
        assert_eq!(reverse(&ids, 9), 2);
        assert_eq!(reverse(&ids, 0), 0);
        assert_eq!(forward(&keys, &ids, 701), 5);
        assert_eq!(forward(&keys, &ids, 702), 0xFFFF);
        assert_eq!(world_index(&keys, &ids, 701), 1);
        assert_eq!(world_index(&keys, &ids, 702), 2);
    }

    #[test]
    fn tier_and_date() {
        let mut s = Slot::empty();
        s.0[at::FLAGS + 27] = 0x10;
        assert_eq!(tier(|n| s.flag(n)), 2);
        s.0[at::FLAGS + 31] |= 0x01;
        assert_eq!(tier(|n| s.flag(n)), 3);
        s.0[at::FLAGS + 41] |= 0x10;
        assert_eq!(tier(|n| s.flag(n)), 4);
        assert_eq!(tier(|_| false), 1);
        assert_eq!(date_text(3, 5, 26, 7, 4), " 3.05.26  7:04");
        assert_eq!(date_text(10, 20, 2026, 22, 13), "10.20.26 22:13");
    }

    #[test]
    fn maxima_rules() {
        // Starts, gains, caps as the spec gives them (synthetic words here).
        let g = Growth { start: [150, 120, 90, 150], gain: [25, 20, 15, 25], cap: [500, 400, 300, 500] };
        let mut s = Slot::empty();
        s.set_maximum_words(0, 275, 210);
        assert_eq!(maxima(&s, &g), [[275, 210], [220, 350]]);
        s.set_maximum_words(0, 600, 210);
        assert_eq!(maxima(&s, &g)[0][0], 500);
        assert_eq!(maxima(&s, &g)[1][0], 400);
        s.set_maximum_words(0, 300, 210);
        assert_eq!(maxima(&s, &g)[1][0], 240);
        s.set_maximum_words(0, 100, 210);
        assert_eq!(maxima(&s, &g)[1][0], 80);
        // Round trips, and the cap on a word above it.
        s.set_maximum_words(0, 225, 135);
        s.set_maximum_words(1, 180, 225);
        assert_eq!(maxima(&s, &g), [[225, 135], [180, 225]]);
        s.set_maximum_words(1, 0, 0);
        assert_eq!(maxima(&s, &g)[1], [180, 225]);
        s.set_maximum_words(0, 525, 135);
        assert_eq!(maxima(&s, &g)[0][0], 500);
        // The empty template's zero words give the caps for Tor.
        assert_eq!(maxima(&Slot::empty(), &g), [[500, 300], [400, 500]]);
    }

    #[test]
    fn status_conversions() {
        assert_eq!(status_seconds(90), 1.5);
        assert_eq!(status_seconds(0), 0.0);
        assert!((status_seconds(-1) + 1.0 / 60.0).abs() < 1e-6);
        assert_eq!(restore_seconds(1.5), 1);
        assert_eq!(restore_seconds(status_seconds(-1)), 0);
        assert_eq!(restore_seconds(35.9), 35);
        assert_eq!(restore_seconds(0.9), 0);
        assert_eq!(restore_seconds(status_seconds(2154)), 35);
        assert_eq!(restore_luck(30), 24);
        assert_eq!(restore_luck(7), 7);
    }

    #[test]
    fn picker_cursor() {
        let filled = [true, false, false, true];
        assert_eq!(load_start(filled), 0);
        assert_eq!(load_start([false, false, true, true]), 2);
        assert_eq!(load_start([false; 4]), 4);
        // Load mode skips empty slots; with none that way the cursor stays.
        assert_eq!(cursor_step(0, true, true, filled), 3);
        assert_eq!(cursor_step(3, true, true, filled), 3);
        assert_eq!(cursor_step(3, false, true, filled), 0);
        assert_eq!(cursor_step(0, false, true, filled), 0);
        // Save mode moves by one, bounded.
        assert_eq!(cursor_step(0, true, false, filled), 1);
        assert_eq!(cursor_step(3, true, false, filled), 3);
        assert_eq!(cursor_step(0, false, false, filled), 0);
    }

    #[test]
    fn save_point_step() {
        let at = |x: f32| SavePoint { position: [x, 0.0, 0.0], flag: 0 };
        let mut sp = SavePoints::default();
        // Distances 20, 4 and 4: the lower index wins the tie.
        let r = sp.step(&[at(20.0), at(4.0), at(-4.0)], [0.0; 3], true, true, false);
        assert_eq!(r.chosen, [Some(1), None]);
        assert!(r.glow_moved && r.cue && r.prompt && r.open && r.restart);
        // Distance 10: cue, no prompt, away.
        let mut sp = SavePoints::default();
        let r = sp.step(&[at(10.0)], [0.0; 3], true, false, false);
        assert!(r.cue && !r.prompt && sp.away);
        // Exactly 15 and exactly 5: both tests are strict.
        let r = sp.step(&[at(15.0)], [0.0; 3], true, false, false);
        assert!(!r.cue && !r.prompt && r.chosen[0] == Some(0));
        let r = sp.step(&[at(5.0)], [0.0; 3], true, true, false);
        assert!(r.cue && !r.prompt && !r.open);
        // Busy: the prompt shows, nothing opens.
        let r = sp.step(&[at(3.0)], [0.0; 3], true, true, true);
        assert!(r.cue && r.prompt && !r.open && !r.restart);
        // Gate closed (game-state code 6): cue, no prompt, away untouched.
        let mut sp = SavePoints { away: false, ..Default::default() };
        let r = sp.step(&[at(3.0)], [0.0; 3], false, true, false);
        assert!(r.cue && !r.prompt && !r.open && !sp.away);
        // A nonzero flag opens the screen without the restart point.
        let r = sp.step(&[SavePoint { position: [3.0, 0.0, 0.0], flag: 1 }], [0.0; 3], true, true, false);
        assert!(r.open && !r.restart);
        // Points past the first eight are not used.
        let mut many = vec![at(100.0); 8];
        many.push(at(1.0));
        let r = SavePoints::default().step(&many, [0.0; 3], true, false, false);
        assert_eq!(r.chosen[0], Some(0));
    }
}
