//! The quick-item panel and consumable heals (SPEC-0021 Quick-item selection marker, Quick-item
//! panel, Consumable healing): four entries with their boot candidate rows and the 8-step
//! refill, opening on an Up edge, direction edges, Cross (set the marker, close, use), Circle
//! (set the marker, close), Triangle (close, or use the marker with the panel closed), and the
//! pool heal trunc(T x p / 100 + 1), applied when it is smaller than the missing amount or the
//! use is forced.
//!
//! The second quick panel, the Mote picker (SPEC-0021 Second quick panel and Mote equipping):
//! `MotePanel`.
//!
//! Not covered: cues, the three panel-control words, the in-play gate's hero predicate and
//! game-global words, the charm-only gate and the use hook.

/// The panel's button edges this tick: a direction (0 Right, 1 Up, 2 Left, 3 Down), Cross,
/// Circle and Triangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelInput {
    pub right: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
    pub cross: bool,
    pub circle: bool,
    pub triangle: bool,
}

impl PanelInput {
    /// The direction with Left first, then Up, Down and Right.
    fn direction(&self) -> Option<usize> {
        if self.left {
            Some(2)
        } else if self.up {
            Some(1)
        } else if self.down {
            Some(3)
        } else if self.right {
            Some(0)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuickPanel {
    pub open: bool,
    pub entries: [i32; 4],
    /// 0 to 3, or 4 for no selection.
    pub selection: usize,
    /// The selection marker: an item id, or -1.
    pub marker: i32,
}

impl Default for QuickPanel {
    fn default() -> Self {
        Self { open: false, entries: [-1; 4], selection: 4, marker: -1 }
    }
}

/// What a panel tick asks of the game.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelOut {
    /// Run the use step on this id.
    pub use_item: Option<i32>,
    /// The panel took the Cross and Circle edges (the hero does not see them).
    pub consumed: bool,
}

impl QuickPanel {
    /// The effective marker: the marker when the party owns at least one, else -1.
    pub fn effective(&self, owns: &dyn Fn(i32) -> bool) -> i32 {
        if self.marker >= 0 && owns(self.marker) { self.marker } else { -1 }
    }

    /// The refill of entry k: up to 8 steps through its row (8 slots, ending in -1).
    pub fn refill(&mut self, k: usize, rows: &[[i32; 8]; 4], owns: &dyn Fn(i32) -> bool) {
        let row = &rows[k];
        for _ in 0..8 {
            let v = self.entries[k];
            let next = if v == -1 {
                row[0]
            } else {
                match row.iter().position(|&x| x == v) {
                    Some(7) => row[0],
                    Some(i) => row[i + 1],
                    None => v,
                }
            };
            self.entries[k] = next;
            if next >= 0 && owns(next) {
                return;
            }
        }
    }

    /// One tick. `gate` is the in-play gate for opening (action state not 0x19, ...).
    pub fn tick(&mut self, input: PanelInput, gate: bool, rows: &[[i32; 8]; 4], owns: &dyn Fn(i32) -> bool) -> PanelOut {
        let mut out = PanelOut::default();
        if !self.open {
            if input.up && gate {
                self.open = true;
                for k in [2, 1, 3, 0] {
                    self.refill(k, rows, owns);
                }
                self.selection = 1;
            } else if input.triangle {
                let m = self.effective(owns);
                if m >= 0 {
                    out.use_item = Some(m);
                }
            }
            return out;
        }
        out.consumed = input.cross || input.circle;
        if input.triangle {
            self.open = false;
            return out;
        }
        if let Some(d) = input.direction() {
            if d == self.selection {
                self.refill(d, rows, owns);
            }
            self.selection = d;
        }
        if (input.cross || input.circle) && self.selection < 4 {
            let id = self.entries[self.selection];
            if id >= 0 && owns(id) && id != self.marker {
                self.marker = id;
            }
            self.open = false;
            if input.cross {
                let m = self.effective(owns);
                if m >= 0 {
                    out.use_item = Some(m);
                }
            }
        }
        out
    }

    /// After a use that leaves the item's count at 0 the marker is cleared.
    pub fn used(&mut self, id: i32, left: u32) {
        if left == 0 && self.marker == id {
            self.marker = -1;
        }
    }
}

/// SPEC-0021 Second quick panel: the boot direction table at 0x2DCE58, 15 rows of 8 bytes (row
/// (selection + 1) mod 256; columns Left, Right, Up, Down, Up+Left, Up+Right, Down+Left,
/// Down+Right).
pub const MOTE_TABLE_AT: u32 = 0x002D_CE58;

/// The Mote panel's input this tick: the direction edges and the button edges.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoteInput {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub cross: bool,
    pub square: bool,
    pub circle: bool,
    pub triangle: bool,
}

/// What a Mote panel tick asks of the game.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoteOut {
    /// Set equipment word k to this Mote (Some) or clear it (None).
    pub word: Option<(usize, Option<u16>)>,
    /// A direction edge moved the selection (cue 0x9A).
    pub moved: bool,
}

/// The Mote picker: open flag and selection (0 to 13 for Motes 160 to 173, 255 "No Mote").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotePanel {
    pub open: bool,
    pub selection: u8,
}

impl Default for MotePanel {
    fn default() -> Self {
        Self { open: false, selection: 255 }
    }
}

impl MotePanel {
    /// The direction column, by the first matching rule of SPEC-0021 item 7 (None with no
    /// direction).
    pub fn column(i: &MoteInput) -> Option<usize> {
        if !(i.left || i.right || i.up || i.down) {
            return None;
        }
        Some(if i.up && i.left {
            4
        } else if i.up && i.right {
            5
        } else if i.down && i.left {
            6
        } else if i.right && i.down {
            7
        } else if !i.left && i.right {
            1
        } else if !i.left && !i.right {
            if i.up { 2 } else { 3 }
        } else {
            0
        })
    }

    /// One handler call while the panel is open (the first panel closed). Triangle closes it;
    /// Cross equips into word 0, Square into word 1 when a shield is held, Circle into word 2
    /// when a weapon is held (buttons take priority over the direction); otherwise a direction
    /// moves the selection through the table. `owns` is the owned count test of a Mote id and
    /// `words` the hero's three Mote words.
    pub fn tick(&mut self, i: &MoteInput, table: &[u8; 120], shield: bool, weapon: bool, owns: &dyn Fn(u16) -> bool, words: [Option<u16>; 3]) -> MoteOut {
        let mut out = MoteOut::default();
        if i.triangle {
            self.open = false;
            return out;
        }
        let slot = if i.cross {
            Some(0)
        } else if i.square && shield {
            Some(1)
        } else if i.circle && weapon {
            Some(2)
        } else {
            None
        };
        if i.cross || i.square || i.circle {
            if let Some(k) = slot {
                let id = 160 + self.selection as u16;
                let equip = self.selection != 255 && owns(id) && words[k] != Some(id);
                out.word = Some((k, equip.then_some(id)));
            }
            return out;
        }
        if let Some(c) = Self::column(i) {
            let row = (self.selection as usize + 1) % 256;
            if let Some(&next) = table.get(row * 8 + c) {
                self.selection = next;
            }
            out.moved = true;
        }
        out
    }
}

/// SPEC-0021 Consumable healing for one pool: the new current value when the use heals, else
/// None. `table` is T for the active hero, `percent` the item's stat (100 for the full
/// restores).
pub fn heal(table: i32, percent: i32, current: i32, max: i32, forced: bool) -> Option<i32> {
    if percent <= 0 {
        return None;
    }
    let h = (table as f32 * percent as f32 / 100.0 + 1.0) as i32;
    let missing = max - current;
    if h < missing || (forced && missing > 0) { Some(max.min(current + h)) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mote_panel_vectors() {
        // SPEC-0021 test vectors on a synthetic table holding the spec's entries: from 255,
        // Left 12, Right 13, Up+Left 2; from 0, Right 1.
        let mut table = [255u8; 120];
        (table[0], table[1], table[4]) = (12, 13, 2);
        table[8 + 1] = 1;
        let none = |_: u16| false;
        let mv = |sel: u8, i: MoteInput| {
            let mut p = MotePanel { open: true, selection: sel };
            p.tick(&i, &table, true, true, &none, [None; 3]);
            p.selection
        };
        assert_eq!(mv(255, MoteInput { left: true, ..Default::default() }), 12);
        assert_eq!(mv(255, MoteInput { right: true, ..Default::default() }), 13);
        assert_eq!(mv(255, MoteInput { up: true, left: true, ..Default::default() }), 2);
        assert_eq!(mv(0, MoteInput { right: true, ..Default::default() }), 1);
        // Equip and clear: Circle equips an owned Mote into word 2; pressing it again clears;
        // Square without a shield does nothing; Cross with "No Mote" clears word 0.
        let owns = |id: u16| id == 160;
        let mut p = MotePanel { open: true, selection: 0 };
        let circle = MoteInput { circle: true, ..Default::default() };
        assert_eq!(p.tick(&circle, &table, true, true, &owns, [None; 3]).word, Some((2, Some(160))));
        assert_eq!(p.tick(&circle, &table, true, true, &owns, [None, None, Some(160)]).word, Some((2, None)));
        assert_eq!(p.tick(&MoteInput { square: true, ..Default::default() }, &table, false, true, &owns, [None; 3]).word, None);
        p.selection = 255;
        assert_eq!(p.tick(&MoteInput { cross: true, ..Default::default() }, &table, true, true, &owns, [Some(160), None, None]).word, Some((0, None)));
        assert!(!{ p.tick(&MoteInput { triangle: true, ..Default::default() }, &table, true, true, &owns, [None; 3]); p.open });
    }

    const ROWS: [[i32; 8]; 4] = [
        [145, 54, 51, 52, 53, -1, -1, -1],
        [150, 146, 156, 58, -1, -1, -1, -1],
        [153, 155, 152, 57, 56, 55, 50, -1],
        [149, 157, 147, 59, -1, -1, -1, -1],
    ];

    #[test]
    fn heal_vectors() {
        // Vectors 8 to 12 and 52.
        assert_eq!(heal(500, 20, 0, 1000, false), Some(101));
        assert_eq!(heal(400, 20, 0, 1000, false), Some(81));
        assert_eq!(heal(500, 100, 100, 150, true), Some(150), "501 clamped to the missing amount");
        assert_eq!(heal(300, 50, 0, 1000, false), Some(151));
        assert_eq!(heal(500, 15, 0, 1000, false), Some(76));
        assert_eq!(heal(300, 100, 10, 90, false), None, "301 is not smaller than the missing 80");
        assert_eq!(heal(300, 100, 10, 90, true), Some(90));
        assert_eq!(heal(500, 20, 150, 150, true), None, "nothing missing");
    }

    #[test]
    fn refill_cycles_owned_candidates() {
        let owns = |id: i32| id == 146 || id == 58;
        let mut p = QuickPanel::default();
        p.refill(1, &ROWS, &owns);
        assert_eq!(p.entries[1], 146);
        p.refill(1, &ROWS, &owns);
        assert_eq!(p.entries[1], 58);
        p.refill(1, &ROWS, &owns);
        assert_eq!(p.entries[1], 146, "wraps past -1 to the first owned id");
        // No candidate owned: the entry ends where the eighth step left it.
        let none = |_: i32| false;
        let mut q = QuickPanel::default();
        q.refill(0, &ROWS, &none);
        assert_eq!(q.entries[0], 54, "145, 54, 51, 52, 53, -1, 145, 54");
    }

    #[test]
    fn open_select_and_use() {
        // Vector 41: Right selects entry 0; Cross sets the marker and uses it.
        let owns = |id: i32| id == 145 || id == 146;
        let mut p = QuickPanel::default();
        let out = p.tick(PanelInput { up: true, ..Default::default() }, true, &ROWS, &owns);
        assert!(p.open && p.selection == 1 && out.use_item.is_none());
        assert_eq!(p.entries, [145, 146, -1, 147]);
        p.tick(PanelInput { right: true, ..Default::default() }, true, &ROWS, &owns);
        assert_eq!(p.selection, 0);
        let out = p.tick(PanelInput { cross: true, ..Default::default() }, true, &ROWS, &owns);
        assert_eq!((p.marker, p.open, out.use_item, out.consumed), (145, false, Some(145), true));
        // Vector 42: no selection, Cross does nothing and the panel stays open.
        let mut q = QuickPanel { open: true, ..Default::default() };
        let out = q.tick(PanelInput { cross: true, ..Default::default() }, true, &ROWS, &owns);
        assert!(q.open && out.use_item.is_none());
        // Triangle with the panel closed uses the marker.
        let out = p.tick(PanelInput { triangle: true, ..Default::default() }, true, &ROWS, &owns);
        assert_eq!(out.use_item, Some(145));
        p.used(145, 0);
        assert_eq!(p.marker, -1);
    }
}
