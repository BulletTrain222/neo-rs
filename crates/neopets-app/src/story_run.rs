//! The story checker's runner, a test tool of this build (scripts/story_check.py writes its goal file from SPEC-0058's
//! goal lists): one continuous headless run through the goals. The hero is teleported to each goal, so scenes, level
//! words and flags carry on from goal to goal as in play, but walking is not tested. A fight is ended by the developer
//! defeat. A goal that fails has its Done flags and items forced so that the next goal can run.

use crate::play::Play;

#[derive(Debug, Default, PartialEq)]
pub struct Goal {
    n: u32,
    kind: String,
    pack: Option<u32>,
    spots: Vec<(f32, f32, Option<f32>)>,
    name: String,
    ids: Vec<usize>,
    needs_on: Vec<u16>,
    needs_off: Vec<u16>,
    needs_items: Vec<u16>,
    done: Vec<String>,
    force_on: Vec<u16>,
    force_off: Vec<u16>,
    force_held: Vec<u16>,
    force_gone: Vec<u16>,
    limit: u64,
    /// Other packs the row names, tried when the goal's point has no floor in this one.
    others: Vec<u32>,
    /// Script triggers to walk through first, which load the characters of a talk (the 1098 hall's).
    visits: Vec<u16>,
    /// The script trigger the row names: the hero stands on a point of its volume read from the world, because a row's
    /// centre may lie outside a turned volume.
    trigger: Option<u16>,
}

fn list<T: std::str::FromStr>(s: &str) -> Vec<T> {
    s.split(',').filter_map(|t| t.trim().parse().ok()).collect()
}

/// One tab-separated line of the goal file.
pub fn parse_goal(line: &str) -> Option<Goal> {
    let c: Vec<&str> = line.split('\t').collect();
    if c.len() < 15 {
        return None;
    }
    let spots = c[3]
        .split(';')
        .filter_map(|s| {
            let n: Vec<&str> = s.split(',').collect();
            Some((n.first()?.parse().ok()?, n.get(1)?.parse().ok()?, n.get(2).and_then(|y| y.parse().ok())))
        })
        .collect();
    Some(Goal {
        n: c[0].parse().ok()?,
        kind: c[1].to_string(),
        pack: c[2].parse().ok(),
        spots,
        name: c[4].to_string(),
        ids: list(c[5]),
        needs_on: list(c[6]),
        needs_off: list(c[7]),
        needs_items: list(c[8]),
        done: c[9].split(',').filter(|t| !t.is_empty()).map(str::to_string).collect(),
        force_on: list(c[10]),
        force_off: list(c[11]),
        force_held: list(c[12]),
        force_gone: list(c[13]),
        limit: c[14].trim().parse().ok()?,
        trigger: c.get(15).and_then(|t| t.trim().parse().ok()),
        others: c.get(16).map_or(Vec::new(), |t| list(t)),
        visits: c.get(17).map_or(Vec::new(), |t| list(t)),
    })
}

/// A Done token: f110 a flag on, i6 an item held, p951 the current world pack, ! the opposite.
fn token_holds(t: &str, flag: &dyn Fn(u16) -> bool, item: &dyn Fn(u16) -> bool, pack: u32) -> bool {
    let (want, t) = t.strip_prefix('!').map_or((true, t), |r| (false, r));
    let (kind, n) = t.split_at(1);
    let n: u32 = n.parse().unwrap_or(u32::MAX);
    want == match kind {
        "f" => flag(n as u16),
        "i" => item(n as u16),
        _ => pack == n,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Begin,
    Run,
    Settle(u64),
}

/// What the host does for the runner this tick.
#[derive(Default)]
pub struct Order {
    pub warp: Option<u32>,
    pub square: bool,
    pub jump: bool,
    /// A goal that needs a real attack: the attack button, three quick presses and a rest.
    pub attack: bool,
    /// An exact-count goal: the trace's own prompt presses are off.
    pub quiet: bool,
    /// The stick held forward: an exit rectangle is walked through, not pressed.
    pub walk: bool,
    pub finished: bool,
}

pub struct StoryRun {
    goals: Vec<Goal>,
    at: usize,
    started: u64,
    phase: Phase,
    saved_flags: Vec<u16>,
    saved_items: Vec<(u16, u32)>,
}

impl StoryRun {
    pub fn from_file(path: &str) -> Option<Self> {
        let goals: Vec<Goal> = std::fs::read_to_string(path).ok()?.lines().filter_map(parse_goal).collect();
        Some(Self { goals, at: 0, started: 0, phase: Phase::Begin, saved_flags: Vec::new(), saved_items: Vec::new() })
    }

    fn finish_goal(&mut self, play: &mut Play, status: &str) {
        let g = &self.goals[self.at];
        println!("GOAL {} {status} tick {} pack {}", g.n, play.ticks, play.world.pack);
        if status != "pass" && status != "nocheck" && status != "already" {
            for &f in &g.force_on {
                play.state.set(f, true);
            }
            for &f in &g.force_off {
                play.state.set(f, false);
            }
            for &i in &g.force_held {
                if !play.state.has_item(i) {
                    play.state.give(i, 1);
                }
            }
            for &i in &g.force_gone {
                play.state.take(i, 1);
            }
        }
        self.at += 1;
        self.phase = Phase::Begin;
    }

    /// One tick. A game over tears the world down to a new game: the flags and items are put back and the next goal
    /// warps to its own pack.
    pub fn step(&mut self, play: &mut Play) -> Order {
        let mut order = Order::default();
        loop {
            let Some(g) = self.goals.get(self.at) else {
                order.finished = true;
                return order;
            };
            // Around exact-count presses the trace's own prompt presses stay off, between them too.
            order.quiet = g.kind == "PRESS" || self.at.checked_sub(1).is_some_and(|k| self.goals[k].kind == "PRESS");
            if self.phase != Phase::Begin && play.state.flags.len() + 20 < self.saved_flags.len() {
                play.state.flags = self.saved_flags.iter().copied().collect();
                play.state.items = self.saved_items.iter().copied().collect();
                self.finish_goal(play, "gameover");
                continue;
            }
            let met = !g.done.is_empty() && g.done.iter().all(|t| token_holds(t, &|f| play.state.flag(f), &|i| play.state.has_item(i) || play.equipment.entries.iter().flatten().any(|e| *e == Some(i)), play.world.pack));
            let t = play.ticks.saturating_sub(self.started);
            match self.phase {
                Phase::Begin => {
                    let missing: Vec<String> = g.needs_on.iter().filter(|&&f| !play.state.flag(f)).map(|f| f.to_string()).chain(g.needs_items.iter().filter(|&&i| !play.state.has_item(i)).map(|i| format!("item{i}"))).collect();
                    if !missing.is_empty() {
                        println!("NEEDED {} {}", g.n, missing.join(","));
                    }
                    let (on, off, its) = (g.needs_on.clone(), g.needs_off.clone(), g.needs_items.clone());
                    for f in on {
                        play.state.set(f, true);
                    }
                    for f in off {
                        play.state.set(f, false);
                    }
                    for i in its {
                        if !play.state.has_item(i) {
                            play.state.give(i, 1);
                        }
                    }
                    if !matches!(g.kind.as_str(), "WAIT" | "TALK" | "PICK" | "SQUARE" | "PRESS" | "DOOR" | "WALK" | "FIGHT" | "SWING") {
                        // Forced: a scene its flags start runs out before the next goal.
                        println!("GOAL {} manual tick {} pack {}", g.n, play.ticks, play.world.pack);
                        let g = &self.goals[self.at];
                        for &f in &g.force_on {
                            play.state.set(f, true);
                        }
                        for &f in &g.force_off {
                            play.state.set(f, false);
                        }
                        for &i in &g.force_held {
                            if !play.state.has_item(i) {
                                play.state.give(i, 1);
                            }
                        }
                        for &i in &g.force_gone {
                            play.state.take(i, 1);
                        }
                        self.started = play.ticks;
                        self.phase = Phase::Settle(2);
                        return order;
                    }
                    if met {
                        self.finish_goal(play, "already");
                        continue;
                    }
                    // Shop goals are about the buying, not the saving up.
                    play.state.neopoints = play.state.neopoints.max(5000);
                    self.saved_flags = play.state.flags.iter().copied().collect();
                    self.saved_items = play.state.items.iter().map(|(a, b)| (*a, *b)).collect();
                    self.phase = Phase::Run;
                    self.started = play.ticks;
                    if let Some(p) = g.pack.filter(|&p| p != play.world.pack) {
                        order.warp = Some(p);
                        self.started = 0;
                    }
                    return order;
                }
                Phase::Run => {
                    if met {
                        self.finish_goal_run(play, "pass");
                        continue;
                    }
                    let waiting = g.done.is_empty() && t > g.limit.min(240);
                    if waiting {
                        self.finish_goal_run(play, "nocheck");
                        continue;
                    }
                    if t > g.limit {
                        self.finish_goal(play, "fail");
                        continue;
                    }
                    // No floor at the goal's point here: it lies in the next pack the row names.
                    if t == 5 {
                        if let Some(&(x, z, y)) = g.spots.first() {
                            if play.world.ground_at(x, z, y.map_or(1.0e4, |y| y + 8.0)).is_none() {
                                if let Some(next) = self.goals[self.at].others.first().copied() {
                                    self.goals[self.at].others.remove(0);
                                    order.warp = Some(next);
                                    self.started = 0;
                                    return order;
                                }
                            }
                        }
                    }
                    if !g.visits.is_empty() && t >= 10 && t < 10 + 30 * g.visits.len() as u64 && (t - 10) % 30 == 0 {
                        let i = g.visits[((t - 10) / 30) as usize];
                        if let Some(v) = play.world.volume_points().into_iter().find(|v| v.0 == i) {
                            play.place_at(v.1[0] as f32, v.1[2] as f32, v.1[1] as f32 + 8.0);
                        }
                    }
                    if g.kind != "WAIT" && t >= 10 && (t - 10) % 150 == 0 {
                        let k = ((t - 10) / 150) as usize;
                        let inside = g.trigger.and_then(|i| play.world.volume_points().into_iter().find(|v| v.0 == i)).map(|v| v.1);
                        if let Some(p) = inside.filter(|_| k % 2 == 0) {
                            play.place_at(p[0] as f32, p[2] as f32, p[1] as f32 + 8.0);
                        } else if !g.spots.is_empty() {
                            let (x, z, y) = g.spots[k % g.spots.len()];
                            // A shop's item is bought from beside it, facing it: every other visit.
                            let beside = if g.kind == "PICK" && k % 2 == 1 { 2.0 } else { 0.0 };
                            play.place_at(x + beside, z, y.map_or(1.0e4, |y| y + 8.0));
                            if beside > 0.0 {
                                play.face_point(x, z);
                            }
                        }
                    }
                    if !(g.name.is_empty() && g.ids.is_empty()) && t >= 20 && (t - 20) % 150 == 0 {
                        play.dev_go_near(&g.name, &g.ids);
                    }
                    // A door: each visit also walks a little in each of four directions from its point.
                    if g.kind == "DOOR" && !g.spots.is_empty() && t >= 40 {
                        let phase = (t - 40) % 150;
                        if phase % 25 == 0 && phase < 100 {
                            let p = play.loco.position();
                            let (x, z) = [(4.0, 0.0), (-4.0, 0.0), (0.0, 4.0), (0.0, -4.0)][(phase / 25) as usize];
                            let (sx, sz, sy) = g.spots[0];
                            play.place_at(sx, sz, sy.map_or(1.0e4, |y| y + 8.0));
                            let _ = p;
                            play.face_point(sx + x, sz + z);
                        }
                        order.walk = phase < 100;
                    }
                    if g.kind == "FIGHT" && t % 600 == 200 {
                        play.dev_kill_enemies();
                    }
                    // Square also buys a shop's item, which a touch does not take.
                    order.square = (matches!(g.kind.as_str(), "TALK" | "SQUARE" | "DOOR" | "PICK") && t >= 30 && t % 15 == 0) || (g.kind == "PRESS" && t == 40);
                    order.jump = g.kind == "PICK" && t % 60 == 40;
                    order.attack = g.kind == "SWING" && t >= 30 && matches!((t - 30) % 150, 0 | 32 | 56);
                    return order;
                }
                Phase::Settle(status) => {
                    // A scene the goal started runs out before the next goal, within reason.
                    let held = !play.scene_status().contains("held false") || play.dialogue_open;
                    if !held || t > 600 {
                        if status < 2 {
                            println!("GOAL {} {} tick {} pack {}", g.n, if status == 1 { "pass" } else { "nocheck" }, play.ticks, play.world.pack);
                        }
                        self.at += 1;
                        self.phase = Phase::Begin;
                        continue;
                    }
                    return order;
                }
            }
        }
    }

    fn finish_goal_run(&mut self, play: &Play, status: &str) {
        self.started = play.ticks;
        self.phase = Phase::Settle(u64::from(status == "pass"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_goal_line_parses() {
        let g = parse_goal("6\tSQUARE\t909\t-1949.4,1058.1,45.5;1,2\tPatricia\t37,38\t111\t112\t6\tf112,i7,!i6\t112\t\t7\t6\t1500\t5").unwrap();
        assert_eq!(g.trigger, Some(5));
        assert_eq!((g.n, g.kind.as_str(), g.pack, g.limit), (6, "SQUARE", Some(909), 1500));
        assert_eq!(g.spots, vec![(-1949.4, 1058.1, Some(45.5)), (1.0, 2.0, None)]);
        assert_eq!((g.needs_on, g.needs_off, g.needs_items), (vec![111], vec![112], vec![6]));
        assert_eq!(g.done, ["f112", "i7", "!i6"]);
        assert!(parse_goal("too\tshort").is_none());
    }

    #[test]
    fn done_tokens() {
        let flag = |f: u16| f == 1;
        let item = |i: u16| i == 2;
        for (t, want) in [("f1", true), ("!f1", false), ("f3", false), ("i2", true), ("!i3", true), ("p9", true), ("p8", false)] {
            assert_eq!(token_holds(t, &flag, &item, 9), want, "{t}");
        }
    }
}
