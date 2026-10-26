//! The object records of pack 1217, the Juppie burrows (SPEC-0045 1217 Main records, Gate
//! records and Cage records): eleven main records (rise, idle loop and hit groups with an inner
//! record), two gates and the cages, each stepped every tick with the request rule (a request of
//! 8 changes nothing, a request equal to the state is dropped, any other becomes the state and
//! that pass is the state's entry; the request is then 8).
//!
//! Not covered: moving effects 21, 22 and 23 to their locators (MoveEffect is not built; they
//! start where the level places them) and the Act 3 records.
use crate::level::Action;

/// The request that changes nothing.
pub const NONE: i32 = 8;

fn take(state: &mut i32, request: &mut i32) -> bool {
    let entry = *request != NONE && *request != *state;
    if entry {
        *state = *request;
    }
    *request = NONE;
    entry
}

fn on(objs: &[usize], end: bool) -> impl Iterator<Item = Action> + '_ {
    objs.iter().map(move |&obj| Action::InitObjOn { obj, end })
}

fn off(objs: &[usize]) -> impl Iterator<Item = Action> + '_ {
    objs.iter().map(|&obj| Action::DeactivateObj { obj })
}

fn play(objs: &[usize], speed: f32) -> impl Iterator<Item = Action> + '_ {
    objs.iter().map(move |&obj| Action::PlayAnim { obj, speed, wait: false })
}

/// Set to frame 0 and played forward at `speed` (a replay of the whole group).
fn replay(objs: &[usize], speed: f32) -> impl Iterator<Item = Action> + '_ {
    objs.iter().flat_map(move |&obj| [Action::InitObjOn { obj, end: false }, Action::PlayAnim { obj, speed, wait: false }])
}

fn sound(cue: i32, locator: usize) -> Option<usize> {
    let _ = cue;
    Some(locator)
}

/// A gate record: groups A and B of 27 objects, the single object and the sound locator.
#[derive(Clone, Debug, PartialEq)]
pub struct Gate {
    pub a: Vec<usize>,
    pub b: Vec<usize>,
    pub single: usize,
    pub locator: usize,
    pub state: i32,
    pub request: i32,
}

/// The inner record of a main record.
#[derive(Clone, Debug, PartialEq)]
pub struct Inner {
    pub p: usize,
    pub q: usize,
    pub r: [usize; 2],
    pub s: Vec<usize>,
    pub t: usize,
    pub r_locator: usize,
    pub s_locator: usize,
    pub state: i32,
    pub request: i32,
    stamp: Option<u64>,
    sounded: bool,
}

/// A main record: rise A, idle B, hit C, effect E and its inner record.
#[derive(Clone, Debug, PartialEq)]
pub struct Main {
    pub a: Vec<usize>,
    pub b: Vec<usize>,
    pub c: Vec<usize>,
    pub e: usize,
    pub inner: Inner,
    pub state: i32,
    pub request: i32,
    /// The close has played A backward (state 4).
    closing: bool,
}

/// A cage record: groups A and B of 7 objects, its entity, its extra object and locator.
#[derive(Clone, Debug, PartialEq)]
pub struct Cage {
    pub a: Vec<usize>,
    pub b: Vec<usize>,
    pub entity: usize,
    pub extra: usize,
    pub locator: usize,
    pub state: i32,
    pub request: i32,
    prev: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Burrows {
    pub gates: Vec<Gate>,
    pub mains: Vec<Main>,
    pub cages: Vec<Cage>,
}

/// The objects a record of 1217 names (the record objects follow the draw gate; the module's
/// other objects are switched by helpers whose calls are not listed one by one).
pub fn named(i: usize) -> bool {
    (2..=309).contains(&i) || (364..=718).contains(&i) || i <= 1
}

impl Burrows {
    /// The Act 1 records of the fight-due branch: eleven main records, the two gates and the Act 1
    /// cage. Returns the fill's calls (effects 10 to 20 and 21 to 23 stopped, the cage's extra
    /// object at its start pose).
    pub fn act1() -> (Self, Vec<Action>) {
        let range = |a: usize, n: usize| (a..a + n).collect::<Vec<_>>();
        let mains = (0..11)
            .map(|r| Main {
                a: range(13 + 27 * r, 9),
                b: range(22 + 27 * r, 9),
                c: range(31 + 27 * r, 9),
                e: 10 + r,
                inner: Inner {
                    p: 364 + 4 * r,
                    q: 365 + 4 * r,
                    r: [366 + 4 * r, 367 + 4 * r],
                    s: range(408 + 13 * r, 13),
                    t: 2 + r,
                    r_locator: 56 + 3 * r,
                    s_locator: 57 + 3 * r,
                    state: 8,
                    request: 0,
                    stamp: None,
                    sounded: false,
                },
                state: 8,
                request: 0,
                closing: false,
            })
            .collect();
        let gates = vec![
            Gate { a: range(611, 27), b: range(638, 27), single: 0, locator: 38, state: 8, request: 0 },
            Gate { a: range(665, 27), b: range(692, 27), single: 1, locator: 39, state: 8, request: 0 },
        ];
        let cages = vec![Cage { a: range(597, 7), b: range(604, 7), entity: 3, extra: 554, locator: 40, state: 0, request: NONE, prev: 0 }];
        let mut fill: Vec<Action> = (10..=23).map(|slot| Action::StopEffect { slot }).collect();
        fill.push(Action::InitObjOn { obj: 554, end: false });
        (Self { gates, mains, cages }, fill)
    }

    /// The gates only (both branches of the fight-due test fill them).
    pub fn gates_only() -> Self {
        let (b, _) = Self::act1();
        Self { gates: b.gates, mains: Vec::new(), cages: Vec::new() }
    }

    /// One pass of the record thread: the eleven records (each with its inner record), the cages
    /// and the gates. `animating` asks whether an object animates; `tick` is the game clock.
    pub fn step(&mut self, animating: &dyn Fn(usize) -> bool, tick: u64) -> Vec<Action> {
        let mut out = Vec::new();
        let any = |v: &[usize]| v.iter().any(|&o| animating(o));
        for m in &mut self.mains {
            let req = m.request;
            let entry = take(&mut m.state, &mut m.request);
            match m.state {
                0 | 5 => {}
                1 => {
                    if entry {
                        out.extend(on(&m.a, false));
                        out.extend(off(&m.b));
                        out.extend(off(&m.c));
                        out.extend(play(&m.a, 1.0));
                        out.push(Action::StartEffect { slot: m.e, camera: false });
                        out.push(Action::LoopSound { cue: 8, locator: Some(0) });
                        m.inner.request = 1;
                    } else if any(&m.a) {
                        out.push(Action::VisitLoopSound { cue: 8, locator: Some(0) });
                    } else {
                        out.extend(off(&m.a));
                        out.extend(on(&m.b, false));
                        out.extend([Action::KillSound { cue: 8, locator: Some(0) }, Action::StopEffect { slot: m.e }]);
                        m.state = 2;
                    }
                }
                3 => {
                    if entry {
                        out.extend(off(&m.a));
                        out.extend(off(&m.b));
                        out.extend(on(&m.c, false));
                        out.extend(play(&m.c, 1.0));
                        out.extend([Action::StartEffect { slot: m.e, camera: false }, Action::PlaySound { cue: 6, locator: Some(0) }]);
                        if !matches!(m.inner.state, 5..=7) {
                            m.inner.request = 3;
                        }
                    } else if !any(&m.c) {
                        out.extend(on(&m.b, false));
                        out.extend(off(&m.a));
                        out.extend(off(&m.c));
                        out.push(Action::StopEffect { slot: m.e });
                        m.state = 2;
                    }
                }
                4 => {
                    if entry {
                        m.closing = false;
                        if !matches!(m.inner.state, 5..=7) {
                            m.inner.request = 3;
                        }
                    } else if !m.closing && matches!(m.inner.state, 5..=7) {
                        out.extend(off(&m.a));
                        out.extend(off(&m.b));
                        out.extend(off(&m.c));
                        m.state = 5;
                    } else if !m.closing && m.inner.state == 2 {
                        // Once the inner record is back in state 2: A at its end pose, played
                        // backward, E, the loop sound and inner request 4.
                        out.extend(on(&m.a, true));
                        out.extend(play(&m.a, -1.0));
                        out.extend([Action::StartEffect { slot: m.e, camera: false }, Action::LoopSound { cue: 9, locator: Some(0) }]);
                        m.inner.request = 4;
                        m.closing = true;
                    } else if m.closing && !any(&m.a) {
                        out.extend(off(&m.a));
                        out.extend(off(&m.b));
                        out.extend(off(&m.c));
                        out.extend([Action::KillSound { cue: 9, locator: Some(0) }, Action::StopEffect { slot: m.e }]);
                        m.state = 5;
                    }
                }
                _ => {
                    // State 2 and the others: the idle loop.
                    if entry && req == 2 {
                        m.inner.request = 2;
                    }
                    if !any(&m.b) {
                        out.extend(replay(&m.b, 1.0));
                        out.push(Action::StopEffect { slot: m.e });
                    }
                }
            }
            out.extend(m.inner.step(animating, tick));
        }
        for c in &mut self.cages {
            let entry = take(&mut c.state, &mut c.request);
            match c.state {
                0 => {
                    if c.prev != 0 {
                        out.extend(off(&c.a));
                        out.extend(off(&c.b));
                    }
                }
                4 => {
                    if entry {
                        out.extend(on(&c.b, false));
                        out.extend(off(&c.a));
                        out.extend(play(&c.b, 1.0));
                        out.push(Action::PlayAnim { obj: c.extra, speed: 1.0, wait: false });
                        out.push(Action::EntityAnim { entity: c.entity, anim: 0, looped: false, wait: false });
                        out.push(Action::LoopSound { cue: 9, locator: sound(9, c.locator) });
                    } else if c.b.iter().any(|&o| animating(o)) {
                        out.push(Action::VisitLoopSound { cue: 9, locator: sound(9, c.locator) });
                    } else {
                        out.push(Action::KillSound { cue: 9, locator: sound(9, c.locator) });
                        out.extend(off(&c.a));
                        out.extend(off(&c.b));
                        c.state = 5;
                    }
                }
                5 => {}
                _ => {
                    if !c.a.iter().any(|&o| animating(o)) {
                        out.extend(replay(&c.a, 2.5));
                    }
                }
            }
            c.prev = c.state;
        }
        for g in &mut self.gates {
            let entry = take(&mut g.state, &mut g.request);
            match g.state {
                0 | 5 => {}
                1 => {
                    if entry {
                        out.extend(on(&g.b, true));
                        out.extend(off(&g.a));
                        out.push(Action::DeactivateObj { obj: g.single });
                        out.push(Action::LoopSound { cue: 8, locator: Some(g.locator) });
                        out.extend(play(&g.b, -1.0));
                        out.push(Action::InitObjOn { obj: g.single, end: false });
                    } else if g.b.iter().any(|&o| animating(o)) {
                        out.push(Action::VisitLoopSound { cue: 8, locator: Some(g.locator) });
                    } else {
                        out.push(Action::KillSound { cue: 8, locator: Some(g.locator) });
                        out.extend(off(&g.b));
                        out.extend(on(&g.a, false));
                        out.push(Action::InitObjOn { obj: g.single, end: false });
                        g.state = 2;
                    }
                }
                4 => {
                    if entry {
                        out.extend(on(&g.b, false));
                        out.extend(off(&g.a));
                        out.push(Action::LoopSound { cue: 9, locator: Some(g.locator) });
                        out.extend(play(&g.b, 1.0));
                        out.push(Action::DeactivateObj { obj: g.single });
                    } else if g.b.iter().any(|&o| animating(o)) {
                        out.push(Action::VisitLoopSound { cue: 9, locator: Some(g.locator) });
                    } else {
                        out.push(Action::KillSound { cue: 9, locator: Some(g.locator) });
                        out.extend(off(&g.b));
                        out.extend(off(&g.a));
                        out.push(Action::DeactivateObj { obj: g.single });
                        g.state = 5;
                    }
                }
                _ => {
                    if entry {
                        out.extend(on(&g.a, false));
                        out.push(Action::InitObjOn { obj: g.single, end: false });
                        out.extend(off(&g.b));
                    }
                    if !g.a.iter().any(|&o| animating(o)) {
                        out.extend(replay(&g.a, 1.0));
                    }
                }
            }
        }
        out
    }
}

impl Inner {
    fn step(&mut self, animating: &dyn Fn(usize) -> bool, tick: u64) -> Vec<Action> {
        let mut out = Vec::new();
        let request = self.request;
        let from = self.state;
        let entry = take(&mut self.state, &mut self.request);
        let all = |s: &Inner| {
            let mut v = vec![s.p, s.q, s.r[0], s.r[1], s.t];
            v.extend(&s.s);
            v
        };
        match self.state {
            1 => {
                if entry {
                    out.push(Action::InitObjOn { obj: self.p, end: false });
                    out.extend(off(&[self.q, self.r[0], self.r[1]]));
                    out.extend(off(&self.s));
                    out.extend([Action::StopEffect { slot: 23 }, Action::StopEffect { slot: 22 }]);
                    out.push(Action::PlayAnim { obj: self.p, speed: 1.0, wait: false });
                } else if !animating(self.p) {
                    out.extend(on(&self.r, false));
                    out.extend(off(&[self.p, self.q]));
                    out.extend(off(&self.s));
                    self.state = 2;
                }
            }
            2 => {
                if entry && request == 2 {
                    out.extend(off(&all(self)));
                    self.state = 0;
                } else if entry || matches!(from, 1 | 3) {
                    out.extend(on(&self.r, false));
                }
            }
            3 => {
                if entry {
                    out.push(Action::InitObjOn { obj: self.q, end: false });
                    out.push(Action::StartEffect { slot: 23, camera: false });
                    out.extend(off(&[self.p, self.r[0], self.r[1]]));
                    out.extend(off(&self.s));
                    out.push(Action::PlayAnim { obj: self.q, speed: 1.0, wait: false });
                } else if !animating(self.q) {
                    out.extend(on(&self.r, false));
                    out.extend(off(&[self.p, self.q]));
                    out.extend(off(&self.s));
                    self.state = 2;
                }
            }
            4 => {
                if entry {
                    out.extend(on(&self.r, false));
                    out.extend(off(&[self.p, self.q, self.t]));
                    out.extend(off(&self.s));
                    out.extend(play(&self.r, 1.0));
                    self.stamp = Some(tick);
                    self.sounded = false;
                } else {
                    if !self.sounded && self.stamp.is_some_and(|s| tick >= s + 24) {
                        self.sounded = true;
                        out.push(Action::PlaySound { cue: 10, locator: Some(self.r_locator) });
                        out.push(Action::StartEffect { slot: 21, camera: false });
                    }
                    if self.sounded && !self.r.iter().any(|&o| animating(o)) {
                        out.push(Action::InitObjOn { obj: self.t, end: false });
                        self.state = 6;
                    }
                }
            }
            7 => {
                if entry {
                    out.extend(on(&self.s, false));
                    out.push(Action::StartEffect { slot: 22, camera: false });
                    out.extend(off(&[self.p, self.q, self.r[0], self.r[1], self.t]));
                    out.push(Action::PlaySound { cue: 11, locator: Some(self.s_locator) });
                    out.extend(play(&self.s, 1.0));
                } else if !self.s.iter().any(|&o| animating(o)) {
                    out.extend(off(&all(self)));
                    self.state = 5;
                }
            }
            _ => {}
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC-0045 test vector (record 3): A 94 to 102, B 103 to 111, C 112 to 120, E 13, P 376,
    /// Q 377, R 378 and 379, S 447 to 459, T 5; request 1 rises and loops, request 3 hits, the
    /// death step ends in inner state 6 with T on.
    #[test]
    fn record_three() {
        let (mut b, fill) = Burrows::act1();
        assert!(fill.contains(&Action::InitObjOn { obj: 554, end: false }));
        let m = &b.mains[3];
        assert_eq!((m.a[0], m.a[8], m.b[0], m.c[8], m.e), (94, 102, 103, 120, 13));
        assert_eq!((m.inner.p, m.inner.q, m.inner.r, m.inner.s[0], m.inner.s[12], m.inner.t), (376, 377, [378, 379], 447, 459, 5));
        let busy = std::cell::RefCell::new(std::collections::HashSet::<usize>::new());
        let animating = |o: usize| busy.borrow().contains(&o);
        // The first pass takes request 0 (state 0).
        b.step(&animating, 0);
        assert_eq!(b.mains[3].state, 0);
        b.mains[3].request = 1;
        busy.borrow_mut().extend(94..=102);
        busy.borrow_mut().insert(376);
        let out = b.step(&animating, 1);
        assert!(out.contains(&Action::PlayAnim { obj: 94, speed: 1.0, wait: false }) && out.contains(&Action::StartEffect { slot: 13, camera: false }));
        assert_eq!(b.mains[3].inner.state, 1);
        busy.borrow_mut().clear();
        b.step(&animating, 2);
        assert_eq!((b.mains[3].state, b.mains[3].inner.state), (2, 2));
        // The hit.
        b.mains[3].request = 3;
        let out = b.step(&animating, 3);
        assert!(out.contains(&Action::PlayAnim { obj: 112, speed: 1.0, wait: false }) && out.contains(&Action::PlaySound { cue: 6, locator: Some(0) }));
        // The death step: inner request 4, the sound 0.4 s later, then T on and state 6.
        b.mains[3].inner.request = 4;
        b.step(&animating, 10);
        let out = b.step(&animating, 34);
        assert!(out.contains(&Action::PlaySound { cue: 10, locator: Some(65) }));
        assert!(out.contains(&Action::InitObjOn { obj: 5, end: false }));
        assert_eq!(b.mains[3].inner.state, 6);
    }

    #[test]
    fn gate_rule() {
        let mut b = Burrows::gates_only();
        let animating = |_: usize| false;
        b.step(&animating, 0);
        assert_eq!(b.gates[0].state, 0);
        b.gates[1].request = 1;
        let out = b.step(&animating, 1);
        assert!(out.contains(&Action::PlayAnim { obj: 692, speed: -1.0, wait: false }));
        // No B object animates: the A loop starts (state 2).
        b.step(&animating, 2);
        assert_eq!(b.gates[1].state, 2);
        let out = b.step(&animating, 3);
        assert!(out.contains(&Action::PlayAnim { obj: 665, speed: 1.0, wait: false }));
        b.gates[1].request = 4;
        b.step(&animating, 4);
        b.step(&animating, 5);
        assert_eq!(b.gates[1].state, 5);
    }
}
