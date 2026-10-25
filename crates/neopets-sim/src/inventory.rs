//! Equipment and the item screen's rules (SPEC-0021 Equipment families and ranking, Party
//! inventory lists, Equipment slots; SPEC-0035 section 14.1).
//!
//! The weapon, shield and body items are the heroes' entries in the category 1, 2 and 5 lists:
//! types 6, 5 and 4 go to slot 0 (Tor), types 3, 2 and 1 to slot 1 (Roberta); an empty entry is
//! filled and a filled one is replaced only by a greater id. The three Mote words per hero hold
//! a charm id or nothing.

/// The family (0 weapon, category 1; 1 shield, category 2; 2 body, category 5) and the hero
/// slot of an equipment config type.
pub fn family(config_type: i16) -> Option<(usize, usize)> {
    match config_type {
        6 => Some((0, 0)),
        3 => Some((0, 1)),
        5 => Some((1, 0)),
        2 => Some((1, 1)),
        4 => Some((2, 0)),
        1 => Some((2, 1)),
        _ => None,
    }
}

/// The heroes' equipment: the family entries [family][hero] and the Mote words [hero][word].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Equipment {
    pub entries: [[Option<u16>; 2]; 3],
    pub motes: [[Option<u16>; 3]; 2],
}

impl Equipment {
    /// The add of an equipment item: returns the id it replaced (or None when the entry was
    /// empty), or Err(()) when the add leaves the entry unchanged (the id is not greater).
    pub fn add(&mut self, id: u16, config_type: i16) -> Result<Option<u16>, ()> {
        let Some((f, h)) = family(config_type) else { return Err(()) };
        match self.entries[f][h] {
            Some(old) if id <= old => Err(()),
            old => {
                self.entries[f][h] = Some(id);
                Ok(old)
            }
        }
    }

    /// Set Mote word k of a hero: returns whether it changed (setting the held id does nothing).
    pub fn set_mote(&mut self, hero: usize, word: usize, id: u16) -> bool {
        let w = &mut self.motes[hero][word];
        if *w == Some(id) {
            return false;
        }
        *w = Some(id);
        true
    }

    /// Clear Mote word k: returns whether it held an id.
    pub fn clear_mote(&mut self, hero: usize, word: usize) -> bool {
        self.motes[hero][word].take().is_some()
    }
}

/// SPEC-0035 section 14.1 Items-tab cursor: index 0 to 19, row = index / 5 (wrapping 0 to 3),
/// column = index % 5 (wrapping 0 to 4).
pub fn grid_move(cursor: usize, up: bool, down: bool, left: bool, right: bool) -> usize {
    let (mut r, mut c) = (cursor / 5, cursor % 5);
    if up {
        r = (r + 3) % 4;
    }
    if down {
        r = (r + 1) % 4;
    }
    if left {
        c = (c + 4) % 5;
    }
    if right {
        c = (c + 1) % 5;
    }
    r * 5 + c
}

/// SPEC-0035 section 14.1 item name line: with a carry limit of 2 or more and the item obtained
/// before, "name (obtained/limit)"; with a limit below 2 and more than one held, "name (held)";
/// otherwise the name. No entry (or none held) gives a single space.
pub fn name_line(name: Option<&[u8]>, limit: u32, obtained: u32, held: u32) -> Vec<u8> {
    let Some(name) = name.filter(|_| held > 0) else { return b" ".to_vec() };
    let mut out = name.iter().take(31).copied().collect::<Vec<u8>>();
    if limit >= 2 && obtained > 0 {
        out.extend(format!(" ({obtained}/{limit})").bytes());
    } else if limit < 2 && held > 1 {
        out.extend(format!(" ({held})").bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_add_rule() {
        let mut e = Equipment::default();
        // The Wooden Sword (98, type 6) fills Tor's weapon entry; a greater sword replaces it;
        // a lesser one does not.
        assert_eq!(e.add(98, 6), Ok(None));
        assert_eq!(e.add(100, 6), Ok(Some(98)));
        assert_eq!(e.add(99, 6), Err(()));
        assert_eq!(e.entries[0], [Some(100), None]);
        // A shield (106, type 5) and a wand (111, type 3, Roberta's slot).
        assert_eq!(e.add(106, 5), Ok(None));
        assert_eq!(e.add(111, 3), Ok(None));
        assert_eq!((e.entries[1][0], e.entries[0][1]), (Some(106), Some(111)));
        assert_eq!(e.add(54, 12), Err(()), "not equipment");
        // Mote words: setting the same id changes nothing.
        assert!(e.set_mote(0, 2, 160) && !e.set_mote(0, 2, 160));
        assert!(e.clear_mote(0, 2) && !e.clear_mote(0, 2));
    }

    #[test]
    fn items_tab_cursor() {
        // SPEC-0035 14.1: from 19, Right gives 15 and Down gives 4; from 0, Up gives 15 and
        // Left gives 4.
        assert_eq!(grid_move(19, false, false, false, true), 15);
        assert_eq!(grid_move(19, false, true, false, false), 4);
        assert_eq!(grid_move(0, true, false, false, false), 15);
        assert_eq!(grid_move(0, false, false, true, false), 4);
    }

    #[test]
    fn name_line_vectors() {
        // SPEC-0035 14.1 (synthetic name Apple).
        assert_eq!(name_line(Some(b"Apple"), 10, 3, 1), b"Apple (3/10)");
        assert_eq!(name_line(Some(b"Apple"), 10, 0, 1), b"Apple");
        assert_eq!(name_line(Some(b"Apple"), 1, 0, 4), b"Apple (4)");
        assert_eq!(name_line(Some(b"Apple"), 1, 0, 1), b"Apple");
        assert_eq!(name_line(None, 1, 0, 1), b" ");
    }
}
