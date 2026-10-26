//! Spawn by key (SPEC-0029 Behavior 31): the search of one character block for the first
//! summon-reserved record, in record order, with the requested key that is not bound to a
//! character, and the minion keys that arrive by animation 82 and state 21 instead of agro.
//!
//! The host keeps each block's reserved records (the summon byte at +0xB6 is set) with the
//! character each is bound to, binds a found record at the request's point and facing, and
//! orders the blocks of a world-level request (the embedded block first, then each slot).

/// One reserved record of a block: its key (+0xAC) and whether a live character holds it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reserved {
    pub key: i16,
    pub bound: bool,
}

/// The first unbound record with the key, in record order.
pub fn find(records: &[Reserved], key: i16) -> Option<usize> {
    records.iter().position(|r| r.key == key && !r.bound)
}

/// The minion keys 2801 to 2807, 2811 to 2817, ..., 2871 to 2877: these play animation 82 and
/// take state 21; every other summon takes the agro state 19.
pub fn minion(key: i16) -> bool {
    (2801..=2877).contains(&key) && (1..=7).contains(&(key % 10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_and_minions() {
        let r = [Reserved { key: 2864, bound: true }, Reserved { key: 2844, bound: false }, Reserved { key: 2864, bound: false }];
        assert_eq!(find(&r, 2864), Some(2));
        assert_eq!(find(&r, 2844), Some(1));
        assert_eq!(find(&r, 619), None);
        assert!(minion(2801) && minion(2807) && minion(2844) && minion(2877));
        assert!(!minion(2800) && !minion(2808) && !minion(2810) && !minion(2878) && !minion(1960));
    }
}
