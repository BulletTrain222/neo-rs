//! The shared entity-control routine of the level modules (SPEC-0043, Shared entity-control
//! routine): a table of up to nine records, one per townsperson, that picks idle animations
//! and walks the entity round its waypoints, stepped once per tick by the module's Update.

/// A waypoint: target locator, dwell and orient flags, stop distance and point animation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub locator: usize,
    pub dwell: bool,
    pub orient: bool,
    pub stop: f32,
    pub anim: Option<(u32, f32)>,
}

impl Point {
    /// A point with the dwell and orient flags, stop 1.1 and no point animation.
    pub fn new(locator: usize, dwell: bool, orient: bool) -> Self {
        Self { locator, dwell, orient, stop: 1.1, anim: None }
    }
}

/// One record.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub entity: usize,
    pub active: bool,
    pub hold: u32,
    pub anims: Vec<(u32, f32)>,
    pub idle: u32,
    pub current: Option<u32>,
    pub timer: u32,
    pub points: Vec<Point>,
    pub index: usize,
    pub anim_range: (f32, f32),
    pub dwell_range: (f32, f32),
    /// None until the first step draws it (the original draws it at the add).
    pub anim_delay: Option<u32>,
    pub move_delay: u32,
    pub anim_clock: u32,
    pub move_clock: u32,
}

/// What a step asks the host to do for an entity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// GrabEntityControl, WalkEntityAndOrient (orient) or WalkEntity, ReleaseEntityControl.
    Walk { entity: usize, locator: usize, stop: f32, orient: bool },
    /// PlayEntityAnim (one shot, no wait).
    Play { entity: usize, anim: u32 },
    /// LoopEntityAnim for `seconds` (0 = the idle loop asked for again).
    Loop { entity: usize, anim: u32, seconds: f32 },
}

/// The table (at most nine records: an add is refused when count + 1 is not below 10).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub records: Vec<Record>,
}

fn ticks(s: f32) -> i32 {
    (60.0 * s) as i32
}

impl Table {
    /// Add(entity, idle, anim min, anim max, dwell min, dwell max): the next record, active, its
    /// idle loop started (LoopEntityAnim(entity, idle, 30 s)). None when the table is full.
    pub fn add(&mut self, entity: usize, idle: u32, anim: (f32, f32), dwell: (f32, f32)) -> Option<Command> {
        if self.records.len() + 1 >= 10 {
            return None;
        }
        self.records.push(Record {
            entity,
            active: true,
            hold: 0,
            anims: Vec::new(),
            idle,
            current: None,
            timer: 0,
            points: Vec::new(),
            index: 0,
            anim_range: anim,
            dwell_range: dwell,
            anim_delay: None,
            move_delay: 10,
            anim_clock: 0,
            move_clock: 0,
        });
        Some(Command::Loop { entity, anim: idle, seconds: 30.0 })
    }

    fn record(&mut self, entity: usize) -> Option<&mut Record> {
        self.records.iter_mut().find(|r| r.entity == entity)
    }

    /// Add animation: appended to the entity's record (ignored when 9 are held).
    pub fn add_anim(&mut self, entity: usize, anim: u32, seconds: f32) {
        if let Some(r) = self.record(entity).filter(|r| r.anims.len() + 1 < 10) {
            r.anims.push((anim, seconds));
        }
    }

    /// Add point: appended to the entity's record; path on from the first point.
    pub fn add_point(&mut self, entity: usize, p: Point) {
        if let Some(r) = self.record(entity).filter(|r| r.points.len() + 1 < 10) {
            r.points.push(p);
        }
    }

    /// Set active (parks or wakes a record).
    pub fn set_active(&mut self, entity: usize, on: bool) {
        if let Some(r) = self.record(entity) {
            r.active = on;
        }
    }

    /// One tick (steps 1 to 7) with the host's IsEntityTalking and IsEntityWalkToPointActive
    /// answers and its Random(lo, hi) (both ends included).
    pub fn step(&mut self, talking: &dyn Fn(usize) -> bool, walking: &dyn Fn(usize) -> bool, random: &mut dyn FnMut(i32, i32) -> i32) -> Vec<Command> {
        let mut out = Vec::new();
        for r in self.records.iter_mut() {
            if r.anim_delay.is_none() {
                r.anim_delay = Some(random(ticks(r.anim_range.0), ticks(r.anim_range.1)).max(0) as u32);
            }
            if !r.active || talking(r.entity) {
                continue;
            }
            r.timer = r.timer.saturating_sub(1);
            r.hold = r.hold.saturating_sub(1);
            if walking(r.entity) {
                continue;
            }
            let path = !r.points.is_empty();
            if path && r.move_clock >= r.move_delay {
                r.move_clock = 0;
                r.index = (r.index + 1) % r.points.len();
                let p = r.points[r.index];
                r.move_delay = if p.dwell { random(ticks(r.dwell_range.0), ticks(r.dwell_range.1)).max(0) as u32 } else { 0 };
                r.timer += 1;
                out.push(Command::Walk { entity: r.entity, locator: p.locator, stop: p.stop, orient: p.orient });
            }
            if r.timer == 0 {
                r.anim_clock += 1;
                r.move_clock += 1;
            }
            let point_anim = if path { r.points[r.index].anim } else { None };
            if r.hold == 0 && r.anim_clock >= r.anim_delay.unwrap_or(0) && (!r.anims.is_empty() || point_anim.is_some()) {
                r.anim_delay = Some(random(ticks(r.anim_range.0), ticks(r.anim_range.1)).max(0) as u32);
                r.anim_clock = 0;
                let (id, mut s) = match point_anim {
                    Some(a) => a,
                    None => r.anims[random(0, r.anims.len() as i32 - 1).clamp(0, r.anims.len() as i32 - 1) as usize],
                };
                if s == -1.0 {
                    out.push(Command::Play { entity: r.entity, anim: id });
                    s = 1.5;
                } else {
                    out.push(Command::Loop { entity: r.entity, anim: id, seconds: s });
                }
                r.current = Some(id);
                r.timer = ticks(s).max(0) as u32;
            }
            if r.timer == 0 {
                out.push(Command::Loop { entity: r.entity, anim: r.idle, seconds: 0.0 });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC-0043 vector: 745 entity 3 (anims 51 (2.5 s) and 147 (1.2 s), anim 2 to 5 s, dwell 5
    /// to 7 s, points 8, 9, 7 with dwell and orient).
    #[test]
    fn walks_the_points_and_plays_idles() {
        let mut t = Table::default();
        assert_eq!(t.add(3, 0, (2.0, 5.0), (5.0, 7.0)), Some(Command::Loop { entity: 3, anim: 0, seconds: 30.0 }));
        t.add_anim(3, 51, 2.5);
        t.add_anim(3, 147, 1.2);
        for l in [8, 9, 7] {
            t.add_point(3, Point::new(l, true, true));
        }
        let mut walking = None::<usize>;
        let mut walks = Vec::new();
        let mut anims = Vec::new();
        let mut draw = 0;
        let mut random = |lo: i32, hi: i32| {
            draw += 1;
            if draw % 2 == 0 { hi } else { lo }
        };
        for tick in 0..2000 {
            // A walk takes 50 ticks here.
            let busy = walking.is_some_and(|start| tick < start + 50);
            let cmds = t.step(&|_| false, &|_| busy, &mut random);
            for c in cmds {
                match c {
                    Command::Walk { locator, stop, orient, .. } => {
                        assert!(orient && stop == 1.1);
                        walks.push((tick, locator));
                        walking = Some(tick);
                    }
                    Command::Loop { anim, seconds, .. } if seconds > 0.0 && anim != 0 => anims.push(anim),
                    _ => {}
                }
            }
        }
        // The first move (move clock 10) goes from index 0 to point 9, then 7, 8, 9 again.
        assert_eq!(walks[0], (10, 9));
        assert_eq!(walks.iter().take(4).map(|w| w.1).collect::<Vec<_>>(), vec![9, 7, 8, 9]);
        // Each dwell is 300 to 420 idle ticks after the 50-tick walk; the clocks stop while an
        // idle animation owns the entity, so the wall-clock gap is longer.
        let gap = walks[1].0 - walks[0].0;
        assert!(gap >= 350, "{gap}");
        assert!(anims.iter().all(|a| [51, 147].contains(a)) && !anims.is_empty());
    }

    #[test]
    fn talking_freezes_and_the_table_holds_nine() {
        let mut t = Table::default();
        for e in 0..12 {
            t.add(e, 0, (1.0, 1.0), (3.0, 4.0));
        }
        assert_eq!(t.records.len(), 9);
        t.add_anim(0, 40, 2.5);
        let mut random = |lo: i32, _| lo;
        let before = t.records[0].clone();
        let cmds = t.step(&|e| e == 0, &|_| false, &mut random);
        assert!(cmds.iter().all(|c| !matches!(c, Command::Loop { entity: 0, .. } | Command::Play { entity: 0, .. })));
        assert_eq!((t.records[0].anim_clock, t.records[0].timer), (before.anim_clock, before.timer));
    }
}
