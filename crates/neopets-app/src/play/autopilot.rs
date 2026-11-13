//! The autopilot (this build's test tool, not part of the game): it plays the current world the
//! way a simple player would, so a long run can be watched in the window (F7 or `--autoplay`)
//! or traced headless (NEOPETS_TRACE_AUTO). Each tick it answers boxes and prompts, drinks a
//! potion when hurt, and otherwise works toward one target, chosen in this order: a hostile
//! common enemy close by (fought with `fight_rules`), a world item lying about, a townsperson
//! it has not talked to, a place of the world (an entry record) it has not been near, and last
//! a door it has not used; on the farm, Whinnies not yet penned come before places. It walks by
//! the route search over the ground (`route_avoiding`); when it stops making progress it backs
//! off sideways, remembers the blocked step so later routes go round it, jumps, and gives up on
//! a target it cannot reach. A place counts as seen once the hero passes within 6 of it.
//! Nothing here reads anything a player could not see on screen except to choose a target and
//! a route.
use super::*;
use std::collections::{HashMap, HashSet};

/// What the autopilot is working toward.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Target {
    #[default]
    None,
    Enemy(u64),
    Item(usize),
    Talk(u64),
    Place(usize),
    /// A script trigger volume's middle (a pump, a chest, a gate: where a prompt comes up).
    Spot(usize),
    Door(usize),
    /// Herding: get behind the nearest Whinny that is not penned.
    Herd,
    /// The overworld: walk on into a neighbouring area not loaded yet (the route found when
    /// chosen).
    Onward,
}

/// The autopilot's memory.
#[derive(Clone, Debug, Default)]
pub struct Autopilot {
    /// One line for the screen: what it is doing now.
    pub doing: String,
    target: Target,
    route: Vec<[f32; 2]>,
    chosen_at: u64,
    /// The last tick the hero got 1.0 nearer the next waypoint, and that distance.
    progress_at: u64,
    best: f32,
    jumps: u32,
    /// Targets given up on, by world.
    /// Targets given up on, by world, until a tick (a minute on).
    skip: HashMap<(u32, Target), u64>,
    /// Townsfolk talked to, by world and name, with the story-flag count at that talk: a talk
    /// is due again once the story has moved on. Places, spots and doors visited, by world.
    talked: HashMap<(u32, String), usize>,
    places: HashSet<(u32, usize)>,
    /// Spots visited, with the progress mark (`progress_mark`) at that visit: a spot is due
    /// again once the story or the inventory has changed (the pump once the bucket is held).
    spots: HashMap<(u32, usize), usize>,
    /// The tick each door was last used (or arrived by).
    doors: HashMap<(u32, usize), u64>,
    /// Route lattice cells found blocked, by world.
    blocked: HashMap<u32, HashSet<(i32, i32)>>,
    /// Ticks at a Talk prompt with no conversation opening yet.
    talk_wait: u32,
    /// A sideways back-off in progress: until this tick, with this stick.
    backoff: Option<(u64, (f32, f32))>,
    /// The end point of the route chosen for `Onward`, and the worlds where none was found.
    onward: Option<[f32; 3]>,
    no_onward: HashSet<u32>,
    onward_rest: u64,
    /// The last tick this world's places and spots were forgotten.
    refreshed: u64,
    /// Targets in a row with no route from here (the hero may stand on something the route
    /// search does not see): five start a short wander.
    no_route: u32,
    /// Prompts pressed, by world, prompt label and spot (the hero's 4-unit cell), with the
    /// progress mark then: the same prompt there is pressed again only after progress.
    pressed: HashMap<(u32, u16, i32, i32), usize>,
    pack: u32,
    /// Counts for the summary.
    pub fights: u32,
    pub items: u32,
    pub talks: u32,
    pub worlds: Vec<u32>,
}

impl Play {
    /// Turn the autopilot on or off.
    pub fn toggle_autopilot(&mut self) {
        self.autopilot = match self.autopilot.take() {
            Some(_) => None,
            None => Some(Autopilot { doing: "starting".into(), ..Default::default() }),
        };
    }

    /// The autopilot's line for the screen, when it is on.
    pub fn autopilot_line(&self) -> Option<String> {
        let a = self.autopilot.as_ref()?;
        Some(format!("AUTOPLAY (F7 stops): {}  |  {} fights, {} items, {} talks, {} worlds", a.doing, a.fights, a.items, a.talks, a.worlds.len()))
    }

    /// One tick of the autopilot: fills the stick and buttons of `input` (and the quick panel's
    /// latches).
    pub fn autopilot_input(&mut self, input: &mut HeroInput) {
        let Some(mut a) = self.autopilot.take() else { return };
        self.autopilot_tick(&mut a, input);
        self.autopilot = Some(a);
    }

    fn autopilot_tick(&mut self, a: &mut Autopilot, input: &mut HeroInput) {
        let t = self.ticks;
        let pack = self.world.pack;
        let hero = self.player.loco.position();
        let keep = HeroInput { square_held: input.square_held, ..Default::default() };
        *input = keep;
        if a.pack != pack {
            a.pack = pack;
            a.onward = None;
            // The door it arrived by (the nearest within 12) counts as used, so it does not
            // walk straight back out.
            if let Some((k, _)) = self.world.planes.iter().enumerate().map(|(k, pl)| (k, ((pl.p[0] + 0.5 * pl.w * pl.d[0]) - hero[0]).hypot((pl.p[2] + 0.5 * pl.w * pl.d[2]) - hero[2]))).filter(|(_, d)| *d < 12.0).min_by(|x, y| x.1.total_cmp(&y.1)) {
                a.doors.insert((pack, k), self.ticks);
            }
            a.target = Target::None;
            a.route.clear();
            if !a.worlds.contains(&pack) {
                a.worlds.push(pack);
            }
        }
        // The game-over page: Continue.
        if self.death.page.is_some() {
            a.doing = "game over: continuing".into();
            input.jump = t % 60 == 0;
            return;
        }
        // Boxes and prompts: a question gets Yes, a box gets Square, a prompt is taken (not the
        // save point's or a shop's, which open screens of their own).
        if self.textbox.asking() {
            a.doing = "answering a question".into();
            input.attack = t % 10 == 5;
            return;
        }
        if self.dialogue_open {
            // A conversation with someone counts as their talk (at the story's mark now).
            if let Some(n) = self.talk_partner.clone() {
                a.talked.insert((pack, n), self.state.flags.len());
            }
            a.talk_wait = 0;
            a.doing = format!("reading{}", self.talk_partner.as_ref().map_or(String::new(), |n| format!(" ({n})")));
            input.square = t % 10 == 0;
            return;
        }
        // A door's Enter (13) or Exit (26) only when a door is the target, so walking past a
        // door does not take it.
        let door_prompt = matches!(self.prompt_shown, Some(13 | 26));
        if let Some(id) = self.prompt_shown.filter(|_| !self.save_prompt && self.buyable.is_none() && (!door_prompt || matches!(a.target, Target::Door(_)))) {
            let key = (pack, id, (hero[0] / 4.0).round() as i32, (hero[2] / 4.0).round() as i32);
            let mark = self.progress_mark();
            // A door it means to use is always taken.
            if (door_prompt || a.pressed.get(&key).is_none_or(|&m| m < mark)) && t % 10 == 0 {
                input.square = true;
                a.pressed.insert(key, mark);
            }
        }
        // A script holding the hero (a scene) or a load: wait.
        if self.held || self.job_pack().is_some() || self.cut.is_some() {
            a.doing = "watching a scene".into();
            return;
        }
        // Below 40 % health: a potion from the quick panel when one is owned.
        if self.player.hero_hp > 0 && self.player.hero_hp * 5 < self.player.hero_max * 2 {
            let owned = (0..4).find(|&k| {
                let id = self.quick.entries[k];
                id >= 0 && self.state.items.get(&(id as u16)).is_some_and(|&n| n > 0)
            });
            if let Some(k) = owned {
                a.doing = "drinking a potion".into();
                if !self.quick.open {
                    self.panel_latch.up |= t % 20 == 0;
                } else if self.quick.selection == k {
                    input.jump = t % 10 == 0;
                } else {
                    match k {
                        0 => self.panel_latch.right = true,
                        1 => self.panel_latch.up = true,
                        2 => self.panel_latch.left = true,
                        _ => self.panel_latch.down = true,
                    }
                }
                return;
            }
        }
        if self.quick.open {
            self.panel_latch.triangle = true;
        }
        // Places passed within 6 count as seen.
        for (k, e) in self.world.entries.iter().enumerate() {
            let across = (e.0[0] - hero[0]).hypot(e.0[2] - hero[2]);
            if (across < 6.0 && (e.0[1] - hero[1]).abs() < 4.0) || across < 3.0 {
                a.places.insert((pack, k));
            }
        }
        let mark = self.progress_mark();
        for (k, v) in self.world.all_volumes.iter().enumerate() {
            let c = Self::volume_middle(v);
            if (c[0] - hero[0]).hypot(c[2] - hero[2]) < 3.0 && (c[1] - hero[1]).abs() < 6.0 {
                a.spots.insert((pack, k), mark);
            }
        }
        if let Some((until, stick)) = a.backoff {
            if t < until {
                (input.forward, input.side) = stick;
                return;
            }
            a.backoff = None;
        }
        // Keep the target while it is valid; look again every 2 s when it is not a fight.
        // Herding re-aims every tick (its point moves with the Whinny).
        let valid = self.autopilot_valid(a, a.target);
        let fighting = matches!(a.target, Target::Enemy(_) | Target::Herd);
        if !valid || (!fighting && t >= a.chosen_at + 120) {
            let next = self.autopilot_choose(a);
            // A walk to a place, a door or on is kept until reached or given up, unless
            // something better (a fight, an item, a talk, herding) turns up.
            let better = matches!(next, Target::Enemy(_) | Target::Item(_) | Target::Talk(_) | Target::Herd);
            let next = if valid && !better && matches!(a.target, Target::Place(_) | Target::Spot(_) | Target::Door(_) | Target::Onward) { a.target } else { next };
            if next != a.target {
                // NEOPETS_TRACE_AUTO=2: each change of target.
                if std::env::var("NEOPETS_TRACE_AUTO").is_ok_and(|v| v == "2") {
                    println!("      AUTO target on tick {t}: {:?} -> {:?} ({})", a.target, next, self.autopilot_describe(next, 0.0));
                }
                a.target = next;
                a.route.clear();
                a.chosen_at = t;
                a.progress_at = t;
                a.best = f32::MAX;
                a.jumps = 0;
                match next {
                    Target::Enemy(_) => a.fights += 1,
                    Target::Item(_) => a.items += 1,
                    Target::Talk(_) => a.talks += 1,
                    _ => {}
                }
            } else {
                a.chosen_at = t;
            }
        }
        let Some(goal) = self.autopilot_point(a.target, a.onward) else {
            // Nothing left: forget the places and spots seen here (at most every 30 s) and look
            // round again; the way out of a script-entered room is often a spot already seen.
            if t >= a.refreshed + 1800 {
                a.refreshed = t;
                a.places.retain(|(p, _)| *p != pack);
                a.spots.retain(|(p, _), _| *p != pack);
                a.skip.retain(|(p, _), _| *p != pack);
                a.doing = "seen everything here: looking round again".into();
            } else {
                a.doing = "nothing left to do here".into();
            }
            if t % 600 == 0 && std::env::var("NEOPETS_TRACE_AUTO").is_ok_and(|v| v == "2") {
                let doors: Vec<_> = a.doors.iter().filter(|((p, _), _)| *p == pack).collect();
                let skips: Vec<_> = a.skip.iter().filter(|((p, _), _)| *p == pack).collect();
                println!("      AUTO idle on tick {t} in {pack}: doors used {doors:?}, given up {skips:?}, planes {}, held {} job {:?} cut {}", self.world.planes.len(), self.held, self.job_pack(), self.cut.is_some());
            }
            input.forward = 0.0;
            return;
        };
        let d = (goal[0] - hero[0]).hypot(goal[2] - hero[2]);
        // A fight close by uses the fighter's rules (no route).
        if let Target::Enemy(id) = a.target {
            if let Some(c) = self.characters.iter().find(|c| c.id == id) {
                if d < 14.0 && (c.position[1] - hero[1]).abs() < 6.0 {
                    let plan = self.fight_rules(c);
                    a.doing = format!("fighting {} ({} HP)", c.name, c.hp.max(0));
                    input.guard = plan.guard;
                    input.attack = plan.attack;
                    input.attack_held = plan.attack_held;
                    input.jump = plan.jump;
                    input.run = plan.run;
                    if !plan.close {
                        (input.forward, input.side) = self.stick_toward(plan.goal[0], plan.goal[1]);
                    }
                    self.autopilot_unstick(a, input, d);
                    return;
                }
            }
        }
        a.doing = self.autopilot_describe(a.target, d);
        // Herding: straight at the point behind the Whinny once near (the Whinny moves).
        if a.target == Target::Herd && d < 20.0 {
            (input.forward, input.side) = self.stick_toward(goal[0], goal[2]);
            input.run = d > 6.0;
            self.autopilot_unstick(a, input, d);
            return;
        }
        // Talk: the Talk label is up and the target is within talking range (5.8), facing it, so
        // press Square (the talk is counted when the conversation opens with them).
        if matches!(a.target, Target::Talk(_)) && self.talk_prompt && (goal[0] - hero[0]).hypot(goal[2] - hero[2]) < 4.5 {
            // Someone with nothing to say shows the prompt and opens nothing: give up on them.
            a.talk_wait += 1;
            if a.talk_wait > 240 {
                a.talk_wait = 0;
                a.skip.insert((pack, a.target), t + 3600);
                a.target = Target::None;
                return;
            }
            input.square = t % 15 == 0;
            if t % 15 != 0 {
                (input.forward, input.side) = self.stick_toward(goal[0], goal[2]);
                input.forward *= 0.15;
                input.side *= 0.15;
            }
            return;
        }
        // The route: planned once per target (again after 10 s), followed waypoint by waypoint.
        if a.route.is_empty() || t >= a.chosen_at + 600 && t % 600 == 0 {
            let avoid = a.blocked.get(&pack).cloned().unwrap_or_default();
            match self.route_avoiding(60_000, Goal::PointAt([goal[0], goal[2], goal[1]]), &avoid).or_else(|| self.route_avoiding(60_000, Goal::Point([goal[0], goal[2]]), &avoid)) {
                Some(mut r) => {
                    a.no_route = 0;
                    // The route ends on a lattice point up to 3 short: finish at the exact point
                    // (an item is taken by touch).
                    r.push([goal[0], goal[2]]);
                    a.route = r;
                }
                // Close but off the walkable ground (a door's middle sits in its wall): straight
                // at it.
                None if d < 8.0 => a.route = vec![[goal[0], goal[2]]],
                None => {
                    a.skip.insert((pack, a.target), t + 3600);
                    a.target = Target::None;
                    a.doing = "no way there; choosing again".into();
                    a.no_route += 1;
                    if a.no_route >= 5 {
                        // Wander a second in a direction from the tick count, with a jump.
                        a.no_route = 0;
                        let ang = (t % 628) as f32 / 100.0;
                        a.backoff = Some((t + 60, (ang.cos(), ang.sin())));
                        input.jump = true;
                        a.doing = "no routes from here: wandering".into();
                    }
                    return;
                }
            }
        }
        while a.route.len() > 1 && (a.route[0][0] - hero[0]).hypot(a.route[0][1] - hero[2]) < 2.0 {
            a.route.remove(0);
        }
        let Some(&w) = a.route.first() else { return };
        let dw = (w[0] - hero[0]).hypot(w[1] - hero[2]);
        // A door is walked into, not stopped at; the edge on to a neighbour is reached within 4
        // (the ground often ends short of it).
        let reached = match a.target {
            Target::Door(_) => dw < 0.6,
            Target::Onward => d < 4.0,
            // A spot's or place's point can lie inside something solid: close by and getting no
            // nearer counts as there.
            Target::Spot(_) | Target::Place(_) => dw < 1.5 || (dw < 4.5 && t > a.progress_at + 30),
            _ => dw < 1.5,
        };
        if a.route.len() == 1 && reached {
            // Arrived: a place or a door counts as visited; an item is taken by touch.
            match a.target {
                Target::Place(k) => {
                    a.places.insert((pack, k));
                }
                Target::Spot(k) => {
                    a.spots.insert((pack, k), self.progress_mark());
                }
                Target::Door(k) => {
                    a.doors.insert((pack, k), self.ticks);
                    input.forward = 0.6;
                }
                Target::Onward => {
                    // At the edge the neighbour streams in (the world changes); if it does not
                    // (a way the story has not opened), this world's edges rest for a minute.
                    a.onward = None;
                    a.no_onward.insert(pack);
                    a.onward_rest = t + 3600;
                }
                _ => {}
            }
            a.target = Target::None;
            return;
        }
        (input.forward, input.side) = self.stick_toward(w[0], w[1]);
        input.run = d > 10.0;
        self.autopilot_unstick(a, input, dw + 4.0 * a.route.len() as f32);
    }

    /// Progress toward the target: 1.0 nearer within 1 s, or a jump (up to three), then the
    /// target is given up.
    fn autopilot_unstick(&mut self, a: &mut Autopilot, input: &mut HeroInput, distance: f32) {
        let t = self.ticks;
        if distance < a.best - 1.0 {
            a.best = distance;
            a.progress_at = t;
            return;
        }
        if t > a.progress_at + 120 && matches!(a.target, Target::Enemy(_)) && distance < 6.0 {
            // A fight in place is progress enough.
            a.progress_at = t;
            return;
        }
        if t > a.progress_at + 60 {
            if a.jumps < 3 {
                a.jumps += 1;
                a.progress_at = t;
                // Remember the step it could not take, back off sideways (alternating sides)
                // for half a second with a jump, and plan again round it.
                if let Some(w) = a.route.first() {
                    a.blocked.entry(self.world.pack).or_default().insert(((w[0] / 4.0).round() as i32, (w[1] / 4.0).round() as i32));
                }
                let side = if a.jumps % 2 == 0 { 1.0 } else { -1.0 };
                a.backoff = Some((t + 30, (-0.5, side)));
                input.jump = true;
                a.doing = format!("stuck: backing off ({})", a.jumps);
                a.route.clear();
            } else {
                a.skip.insert((self.world.pack, a.target), t + 3600);
                if a.target == Target::Onward {
                    a.onward = None;
                    a.no_onward.insert(self.world.pack);
                }
                a.doing = "stuck: giving up on it".into();
                a.target = Target::None;
            }
        }
    }

    /// How far the story and the inventory have come: the story flags set plus the items held.
    fn progress_mark(&self) -> usize {
        self.state.flags.len() + self.state.items.values().map(|&n| n as usize).sum::<usize>()
    }

    /// A trigger volume's middle.
    fn volume_middle(v: &neopets_sim::trigger::Volume) -> [f32; 3] {
        use neopets_sim::trigger::Shape;
        match v.shape {
            Shape::Sphere { centre, .. } => centre,
            Shape::Cylinder { base, axis, length, .. } => std::array::from_fn(|k| base[k] + 0.5 * length * axis[k]),
            Shape::Rectangle { origin, axes, extents } => std::array::from_fn(|k| origin[k] + 0.5 * (extents[0] * axes[0][k] + extents[1] * axes[1][k])),
        }
    }

    fn autopilot_valid(&self, a: &Autopilot, target: Target) -> bool {
        let pack = self.world.pack;
        if a.skip.get(&(pack, target)).is_some_and(|&until| self.ticks < until) {
            return false;
        }
        match target {
            Target::None => false,
            Target::Enemy(id) => self.characters.iter().any(|c| c.id == id && Self::autopilot_foe(c)),
            Target::Item(k) => self.pools.get(&pack).is_some_and(|p| p.slots.get(k).is_some_and(|s| s.is_some())),
            Target::Talk(id) => self.characters.iter().any(|c| c.id == id && !c.inactive && a.talked.get(&(pack, c.name.clone())).is_none_or(|&n| n < self.state.flags.len())),
            Target::Place(k) => !a.places.contains(&(pack, k)),
            Target::Spot(k) => a.spots.get(&(pack, k)).is_none_or(|&m| m < self.progress_mark()) && k < self.world.all_volumes.len(),
            // A door is due when unused, or not used for 30 s (so it does not bounce between
            // two worlds).
            Target::Door(k) => a.doors.get(&(pack, k)).is_none_or(|&u| self.ticks > u + 1800),
            Target::Herd => self.herd_goal().is_some(),
            Target::Onward => a.onward.is_some(),
        }
    }

    /// A common enemy worth fighting: hostile, alive, active, not held by a script, and not one
    /// waiting hidden for its alarm or a script (emerge distance 1 or less).
    fn autopilot_foe(c: &Character) -> bool {
        !c.inactive
            && !c.held
            && c.dying.is_none()
            && c.hp > 0
            && c.row.is_some_and(|r| r.hostile && r.class == Some(17))
            && c.enemy.as_ref().is_some_and(|e| !e.is_dead() && !(e.state == 8 && e.spawn.emerge_distance <= 1.0))
    }

    fn autopilot_choose(&self, a: &mut Autopilot) -> Target {
        let pack = self.world.pack;
        let hero = self.player.loco.position();
        let d3 = |p: [f32; 3]| (p[0] - hero[0]).hypot(p[2] - hero[2]) + 2.0 * (p[1] - hero[1]).abs();
        // The validity test reads a copy, so the onward search and the door reset can write.
        let snap = a.clone();
        let ok = |t: Target| self.autopilot_valid(&snap, t);
        let armed = self.weapon.is_some() || self.hero_index == 1;
        let nearest = |list: Vec<(Target, [f32; 3])>, range: f32| list.into_iter().filter(|(t, p)| ok(*t) && d3(*p) < range).min_by(|x, y| d3(x.1).total_cmp(&d3(y.1))).map(|(t, _)| t);
        if armed {
            let foes = self.characters.iter().filter(|c| c.pack == pack && Self::autopilot_foe(c)).map(|c| (Target::Enemy(c.id), c.position)).collect();
            if let Some(t) = nearest(foes, 30.0) {
                return t;
            }
        }
        // The farm's Whinnies (SPEC-0049): herding them is the first task there.
        if ok(Target::Herd) {
            return Target::Herd;
        }
        let items = self.pools.get(&pack).map(|p| p.slots.iter().enumerate().filter_map(|(k, s)| s.as_ref().map(|i| (Target::Item(k), i.position))).collect()).unwrap_or_default();
        if let Some(t) = nearest(items, 60.0) {
            return t;
        }
        // Anyone friendly: townsfolk and the other named characters with no fight in them (not
        // enemies, bosses, Whinnies or the petpet).
        let pet = self.pet_char;
        let folk = self.characters.iter().filter(|c| c.pack == pack && !c.inactive && c.enemy.is_none() && c.whinny.is_none() && Some(c.id) != pet && !Self::is_boss(c) && c.row.is_none_or(|r| !r.hostile)).map(|c| (Target::Talk(c.id), c.position)).collect();
        // Any townsperson in the world with a talk due (the story moves on through their talks).
        if let Some(t) = nearest(folk, f32::MAX) {
            return t;
        }
        let spots = self.world.all_volumes.iter().enumerate().map(|(k, v)| (Target::Spot(k), Self::volume_middle(v))).collect();
        if let Some(t) = nearest(spots, 80.0) {
            return t;
        }
        let places = self.world.entries.iter().enumerate().map(|(k, e)| (Target::Place(k), e.0)).collect();
        if let Some(t) = nearest(places, f32::MAX) {
            return t;
        }
        if armed {
            let foes = self.characters.iter().filter(|c| c.pack == pack && Self::autopilot_foe(c)).map(|c| (Target::Enemy(c.id), c.position)).collect();
            if let Some(t) = nearest(foes, f32::MAX) {
                return t;
            }
        }
        // An exterior: on into a neighbouring area not loaded yet (again after a rest).
        if a.onward_rest != 0 && self.ticks > a.onward_rest {
            a.onward_rest = 0;
            a.no_onward.clear();
        }
        if !a.no_onward.contains(&pack) && a.onward.is_none() {
            match self.route_avoiding(120_000, Goal::NewNeighbour, a.blocked.get(&pack).unwrap_or(&HashSet::new())) {
                Some(r) => {
                    let e = *r.last().unwrap_or(&[hero[0], hero[2]]);
                    a.onward = Some([e[0], hero[1], e[1]]);
                    return Target::Onward;
                }
                None => {
                    a.no_onward.insert(pack);
                }
            }
        }
        if a.onward.is_some() && ok(Target::Onward) {
            return Target::Onward;
        }
        // Doors (scripted ones too: their prompt is taken): the one used longest ago, never one
        // used in the last 30 s; a door given up on is tried again after a minute.
        let used = |k: usize| a.doors.get(&(pack, k)).copied();
        self.world
            .planes
            .iter()
            .enumerate()
            .map(|(k, _)| k)
            .filter(|&k| ok(Target::Door(k)))
            .min_by_key(|&k| used(k).unwrap_or(0))
            .map_or(Target::None, Target::Door)
    }

    fn autopilot_point(&self, target: Target, onward: Option<[f32; 3]>) -> Option<[f32; 3]> {
        let pack = self.world.pack;
        match target {
            Target::None => None,
            Target::Enemy(id) | Target::Talk(id) => self.characters.iter().find(|c| c.id == id).map(|c| c.position),
            Target::Item(k) => self.pools.get(&pack)?.slots.get(k)?.as_ref().map(|i| i.position),
            Target::Place(k) => self.world.entries.get(k).map(|e| e.0),
            Target::Spot(k) => self.world.all_volumes.get(k).map(Self::volume_middle),
            Target::Door(k) => self.world.planes.get(k).map(|pl| [pl.p[0] + 0.5 * pl.w * pl.d[0], pl.p[1], pl.p[2] + 0.5 * pl.w * pl.d[2]]),
            Target::Herd => self.herd_goal().map(|g| [g[0], self.world.ground_at(g[0], g[1], self.player.loco.position()[1] + 3.0).unwrap_or(self.player.loco.position()[1]), g[1]]),
            Target::Onward => onward,
        }
    }

    fn autopilot_describe(&self, target: Target, d: f32) -> String {
        let name = |id: u64| self.characters.iter().find(|c| c.id == id).map_or("someone".to_string(), |c| c.name.clone());
        match target {
            Target::None => "looking around".into(),
            Target::Enemy(id) => format!("going after {} ({d:.0} away)", name(id)),
            Target::Item(_) => format!("picking up an item ({d:.0} away)"),
            Target::Talk(id) => format!("going to talk to {} ({d:.0} away)", name(id)),
            Target::Place(k) => format!("exploring: place {k} ({d:.0} away)"),
            Target::Spot(k) => format!("checking a spot (trigger {k}, {d:.0} away)"),
            Target::Door(_) => format!("heading for a door ({d:.0} away)"),
            Target::Herd => format!("herding a Whinny to the pen ({d:.0} to get behind it)"),
            Target::Onward => format!("heading on to the next area ({d:.0} away)"),
        }
    }
}
