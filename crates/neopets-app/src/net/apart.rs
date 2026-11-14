//! Players in separate worlds online (docs/MULTIPLAYER.md, Plan: players in separate worlds,
//! phase 1; this build's feature): while apart, each side runs its own world, and the story both
//! games share (flags, items, Neopoints, the collected rows, treasure maps) goes between them as
//! small operations. Each side keeps the story as it last agreed with the other (`StoryBase`);
//! what changed since is sent, and what arrives is put into both the game and the base, so it is
//! not sent back.

use super::*;
use std::collections::{BTreeSet, HashMap};

/// The shared story as last agreed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StoryBase {
    pub flags: BTreeSet<u16>,
    pub items: HashMap<u16, u32>,
    pub neopoints: i32,
    pub collected: BTreeSet<u32>,
    pub maps: BTreeSet<u32>,
}

/// One change to the shared story.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoryOp {
    FlagSet(u16),
    FlagClear(u16),
    /// An item's count changed by this much (gained or used).
    Item(u16, i32),
    Neopoints(i32),
    Collected(u32),
    Map(u32),
}

impl StoryBase {
    pub fn of(s: &neopets_sim::level::GameState) -> Self {
        Self { flags: s.flags.clone(), items: s.items.clone(), neopoints: s.neopoints, collected: s.collected.clone(), maps: s.maps.clone() }
    }

    /// What `s` changed since this base, oldest kinds first.
    pub fn changes(&self, s: &neopets_sim::level::GameState) -> Vec<StoryOp> {
        let mut out = Vec::new();
        out.extend(s.flags.difference(&self.flags).map(|&f| StoryOp::FlagSet(f)));
        out.extend(self.flags.difference(&s.flags).map(|&f| StoryOp::FlagClear(f)));
        let mut ids: Vec<u16> = s.items.keys().chain(self.items.keys()).copied().collect();
        ids.sort_unstable();
        ids.dedup();
        for id in ids {
            let d = s.items.get(&id).copied().unwrap_or(0) as i64 - self.items.get(&id).copied().unwrap_or(0) as i64;
            if d != 0 {
                out.push(StoryOp::Item(id, d.clamp(i32::MIN as i64, i32::MAX as i64) as i32));
            }
        }
        if s.neopoints != self.neopoints {
            out.push(StoryOp::Neopoints(s.neopoints - self.neopoints));
        }
        out.extend(s.collected.difference(&self.collected).map(|&c| StoryOp::Collected(c)));
        out.extend(s.maps.difference(&self.maps).map(|&m| StoryOp::Map(m)));
        out
    }
}

/// Apply the other side's change to a story (the game's, or the base).
pub fn apply_to(op: StoryOp, flags: &mut BTreeSet<u16>, items: &mut HashMap<u16, u32>, neopoints: &mut i32, collected: &mut BTreeSet<u32>, maps: &mut BTreeSet<u32>) {
    match op {
        StoryOp::FlagSet(f) => {
            flags.insert(f);
        }
        StoryOp::FlagClear(f) => {
            flags.remove(&f);
        }
        StoryOp::Item(id, d) => {
            let n = (items.get(&id).copied().unwrap_or(0) as i64 + d as i64).max(0) as u32;
            if n == 0 {
                items.remove(&id);
            } else {
                items.insert(id, n);
            }
        }
        // The purse never goes below 0 (both sides spending at once: the host's order).
        StoryOp::Neopoints(d) => *neopoints = (*neopoints + d).max(0),
        StoryOp::Collected(c) => {
            collected.insert(c);
        }
        StoryOp::Map(m) => {
            maps.insert(m);
        }
    }
}

/// Put the other side's changes into the game's story and the base (not sent back).
pub fn apply_ops(ops: &[StoryOp], s: &mut neopets_sim::level::GameState, base: &mut StoryBase) {
    for &op in ops {
        apply_to(op, &mut s.flags, &mut s.items, &mut s.neopoints, &mut s.collected, &mut s.maps);
        apply_to(op, &mut base.flags, &mut base.items, &mut base.neopoints, &mut base.collected, &mut base.maps);
    }
}

pub(super) fn write_op(w: &mut Writer, op: &StoryOp) {
    match *op {
        StoryOp::FlagSet(f) => {
            w.u8(1);
            w.u16(f);
        }
        StoryOp::FlagClear(f) => {
            w.u8(2);
            w.u16(f);
        }
        StoryOp::Item(id, d) => {
            w.u8(3);
            w.u16(id);
            w.i32(d);
        }
        StoryOp::Neopoints(d) => {
            w.u8(4);
            w.i32(d);
        }
        StoryOp::Collected(c) => {
            w.u8(5);
            w.u32(c);
        }
        StoryOp::Map(m) => {
            w.u8(6);
            w.u32(m);
        }
    }
}

pub(super) fn read_op(r: &mut Reader) -> Option<StoryOp> {
    Some(match r.u8()? {
        1 => StoryOp::FlagSet(r.u16()?),
        2 => StoryOp::FlagClear(r.u16()?),
        3 => StoryOp::Item(r.u16()?, r.i32()?),
        4 => StoryOp::Neopoints(r.i32()?),
        5 => StoryOp::Collected(r.u32()?),
        6 => StoryOp::Map(r.u32()?),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two games apart from one story: each one's changes, sent to the other, leave both with
    /// the same story; a change applied from the other side is not sent back.
    #[test]
    fn two_games_keep_one_story() {
        let mut a = neopets_sim::level::GameState::default();
        a.flags.insert(5);
        a.items.insert(57, 2);
        a.neopoints = 100;
        let mut b = a.clone();
        let (mut base_a, mut base_b) = (StoryBase::of(&a), StoryBase::of(&b));
        // Side A sets a flag and drinks one potion; side B clears flag 5, finds an item and
        // spends 30 Neopoints and gets a treasure map.
        a.flags.insert(9);
        a.items.insert(57, 1);
        b.flags.remove(&5);
        b.items.insert(146, 1);
        b.neopoints -= 30;
        b.maps.insert(3);
        let (from_a, from_b) = (base_a.changes(&a), base_b.changes(&b));
        base_a = StoryBase::of(&a);
        base_b = StoryBase::of(&b);
        apply_ops(&from_b, &mut a, &mut base_a);
        apply_ops(&from_a, &mut b, &mut base_b);
        assert_eq!(StoryBase::of(&a), StoryBase::of(&b));
        assert_eq!((a.flags.iter().copied().collect::<Vec<_>>(), a.items.get(&57), a.items.get(&146), a.neopoints), (vec![9], Some(&1), Some(&1), 70));
        assert!(base_a.changes(&a).is_empty() && base_b.changes(&b).is_empty(), "nothing sent back");
        // The operations travel.
        let mut w = Writer::new(1);
        for op in &from_b {
            write_op(&mut w, op);
        }
        let (_, mut r) = Reader::open(&w.0).unwrap();
        let back: Vec<StoryOp> = (0..from_b.len()).map_while(|_| read_op(&mut r)).collect();
        assert_eq!(back, from_b);
    }
}
