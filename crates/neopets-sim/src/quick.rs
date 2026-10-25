//! The quick-item panel and consumable heals (SPEC-0021 Quick-item selection marker, Quick-item
//! panel, Consumable healing): four entries with their boot candidate rows and the 8-step
//! refill, opening on an Up edge, direction edges, Cross (set the marker, close, use), Circle
//! (set the marker, close), Triangle (close, or use the marker with the panel closed), and the
//! pool heal trunc(T x p / 100 + 1), applied when it is smaller than the missing amount or the
//! use is forced.
//!
//! Not covered: cues, the second (Mote) panel, the three panel-control words, the in-play gate's
//! hero predicate and game-global words, the charm-only gate and the use hook.

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
