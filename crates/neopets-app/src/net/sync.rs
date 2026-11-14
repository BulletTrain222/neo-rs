//! Online play's state sync (docs/MULTIPLAYER.md; this build's feature): what the host sends of
//! its game after a tick, and how the joiner puts it over its own.

use super::*;

impl crate::play::Play {
    /// Host: what the joiner draws after this tick.
    pub fn net_state(&self, generation: u32) -> State {
        let hero = |p: &crate::play::Player, who: usize| HeroState {
            who: who as u8,
            position: p.loco.position(),
            facing: p.loco.facing,
            state: p.loco.state,
            clip: p.hero.clip_state().action,
            hp: p.hero_hp,
            hp_max: p.hero_max,
            mp: p.hero_mp,
            mp_max: p.hero_mp_max,
            flash: p.hero_flash,
            down: p.down.is_some(),
            rate: p.loco.rate * p.loco.haste_rate(),
        };
        let p2 = self.others.first();
        let mut items: Vec<(u16, u32)> = self.state.items.iter().map(|(&k, &v)| (k, v)).collect();
        items.sort_unstable();
        State {
            tick: self.ticks,
            generation,
            pack: self.world.pack,
            heroes: [hero(&self.player, self.hero_index), p2.map(|o| hero(o, o.who)).unwrap_or_default()],
            eye: p2.map_or([0.0; 3], |o| o.camera.eye),
            focus: p2.map_or([0.0; 3], |o| o.camera.focus),
            characters: self
                .characters
                .iter()
                .map(|c| CharState { pack: c.pack, slot: c.slot, ordinal: c.ordinal as u32, position: c.position, facing: c.facing, clip: c.anim.clip_state().action, hp: c.hp, dead: c.dying.is_some() || c.enemy.as_ref().is_some_and(|e| e.is_dead()), inactive: c.inactive })
                .collect(),
            flags: self.state.flags.iter().copied().collect(),
            items,
            neopoints: self.state.neopoints,
            talk: (self.dialogue_open || self.textbox.open).then(|| Talk {
                open: self.textbox.open,
                text: self.textbox.text.clone(),
                start: self.textbox.start as u32,
                end: self.textbox.end as u32,
                p: self.textbox.p,
                choice: self.textbox.choice,
                speaker: self.speaker.0.clone(),
                colour: self.speaker.1,
                labels: self.box_labels.clone(),
            }),
            cut: self.cut.filter(|c| c.state != 0 && c.last.is_some()).map(|_| self.view()),
            events: Vec::new(),
            shots: self.shots.slots.iter().enumerate().filter_map(|(i, s)| Some((i as u8, (*s)?))).collect(),
            story: 0,
            has_story: true,
            panels: p2.map_or((false, [-1; 4], 4, -1, false, 255), |o| (o.quick.open, o.quick.entries, o.quick.selection as u8, o.quick.marker, o.mote_panel.open, o.mote_panel.selection)),
            p2_prompt: p2.and_then(|o| o.captions.0),
            world_rev: 0,
            world: Some(WorldSync {
                pack: self.world.pack,
                breakables: self.world.breakables.iter().map(|b| b.state.clamp(0, 255) as u8).collect(),
                items: {
                    let mut v: Vec<(u32, u16, NetItem)> = self
                        .pools
                        .iter()
                        .flat_map(|(&pack, pool)| {
                            pool.slots.iter().enumerate().filter_map(move |(k, it)| {
                                let it = it.as_ref()?;
                                Some((pack, k as u16, NetItem { id: it.id, position: it.position, state: it.state, angle: it.angle, scale: it.drawn_scale(), direction: it.direction, price: it.price, reference: it.reference }))
                            })
                        })
                        .collect();
                    v.sort_by_key(|e| (e.0, e.1));
                    v
                },
                chars: {
                    let mut c: Vec<(u32, u8, u32)> = self.characters.iter().map(|c| (c.pack, c.slot, c.ordinal as u32)).collect();
                    c.sort_unstable();
                    c
                },
                objects: self.world.object_anims.iter().enumerate().map(|(i, o)| (i as u16, o.state, o.speed, o.drawn as u8 | (o.scripted as u8) << 1)).collect(),
            }),
            anim_times: self.world.object_anims.iter().enumerate().filter(|(_, o)| o.animating()).map(|(i, o)| (i as u16, o.time)).collect(),
            moving: self
                .pools
                .iter()
                .flat_map(|(&pack, pool)| pool.slots.iter().enumerate().filter_map(move |(k, it)| it.as_ref().filter(|it| !matches!(it.state, 1 | 3 | 4)).map(|it| (pack, k as u16, it.position, it.angle))))
                .collect(),
            show: Show {
                music: self.music.channels.map(|c| (c.stream, c.volume)),
                bank: (self.bank_music.playing, self.bank_music.volume, self.bank_music.started),
                intensity: (self.intensity.register2, self.intensity.excite),
                fade: (self.fade.colour, self.fade.alpha, self.fade.target, self.fade.step),
                bars: (self.bars.progress, self.bars.target, self.bars.step, self.bars.done, self.bars.held),
                monster: self.monster_bar.target.map(|t| {
                    let m = &self.monster_bar;
                    (m.t, t, m.name.as_bytes().to_vec(), m.name_width, m.frame, m.fill)
                }),
                message: self.message.map(|(line, at)| (line, self.ticks.saturating_sub(at).min(u32::MAX as u64) as u32)),
                shake: self.shake_now.map(|o| (o.side, o.up, o.aim)),
            },
            page: if self.death.page.is_some() || self.death.flag_e { 2 } else { 0 },
            rtt: 0,
            cmd_ack: 0,
            equip: (self.equipment.entries, self.equipment.motes, self.cloak_worn),
            pet: self.petpet.as_ref().map(|p| (p.type_id, p.t, p.f)),
        }
    }

    /// Joiner: the host's state after this side's tick. Player 1 is the host's hero; player 2
    /// (`others[0]`) is this side's, followed by the camera.
    pub fn apply_net_state(&mut self, s: &State) {
        let put = |p: &mut crate::play::Player, h: &HeroState| {
            p.loco.teleport(h.position, h.facing);
            p.loco.state = h.state;
            if p.hero.clip_state().action != h.clip {
                p.hero.request(h.clip, true);
            }
            (p.hero_hp, p.hero_max, p.hero_mp, p.hero_mp_max, p.hero_flash) = (h.hp, h.hp_max, h.mp, h.mp_max, h.flash);
            p.hero.set_rate(h.rate);
            p.down = h.down.then_some(0);
            // The joiner is not sent the timer: "Down" alone.
            p.down_line = h.down.then(|| b"Down".to_vec());
        };
        put(&mut self.player, &s.heroes[0]);
        if self.others.is_empty() {
            self.add_player();
        }
        if let Some(o) = self.others.first_mut() {
            o.who = s.heroes[1].who as usize;
            put(o, &s.heroes[1]);
            o.camera.eye = s.eye;
            o.camera.focus = s.focus;
            // Player 2 is not run on this side: its animation steps here (player 1's in the tick).
            let _ = o.hero.tick();
            // Player 2's panels as the host has them (the joiner's HUD shows them).
            let pn = &s.panels;
            (o.quick.open, o.quick.entries, o.quick.selection, o.quick.marker) = (pn.0, pn.1, pn.2 as usize, pn.3);
            (o.mote_panel.open, o.mote_panel.selection) = (pn.4, pn.5);
            o.captions.0 = s.p2_prompt;
        }
        for c in self.characters.iter_mut() {
            // States carry only the characters that changed (and a few in turn); a character the
            // host's set lacks (made by this side's own game) is hidden.
            let Some(h) = s.characters.iter().find(|h| h.pack == c.pack && h.slot == c.slot && h.ordinal as usize == c.ordinal) else {
                let known = s.world.as_ref().is_none_or(|w| w.chars.binary_search(&(c.pack, c.slot, c.ordinal as u32)).is_ok());
                if !known && Some(c.id) != self.pet_char {
                    c.inactive = true;
                }
                continue;
            };
            c.inactive = h.inactive;
            c.position = h.position;
            c.facing = h.facing;
            if let Some(w) = c.walker.as_mut() {
                w.body.position = h.position;
            }
            if c.anim.clip_state().action != h.clip {
                c.anim.request(h.clip, true);
            }
            c.hp = h.hp;
            if h.dead {
                if let Some(e) = c.enemy.as_mut().filter(|e| !e.is_dead()) {
                    e.die(&mut self.rng);
                }
            }
        }
        // The host's talk box in place of this side's own, and its scripted camera.
        match &s.talk {
            Some(t) => {
                self.dialogue_open = true;
                self.textbox.open = t.open;
                self.textbox.text = t.text.clone();
                self.textbox.start = t.start as usize;
                self.textbox.end = t.end as usize;
                self.textbox.p = t.p;
                self.textbox.choice = t.choice;
                self.speaker = (t.speaker.clone(), t.colour);
                self.box_labels = t.labels.clone();
            }
            None => {
                self.dialogue_open = false;
                self.textbox.open = false;
            }
        }
        self.net_cut = s.cut;
        // The host's music, fade, letterbox, monster bar, message and shake.
        {
            let sh = &s.show;
            for (c, &(stream, volume)) in self.music.channels.iter_mut().zip(sh.music.iter()) {
                c.stream = stream;
                c.volume = volume;
            }
            (self.bank_music.playing, self.bank_music.volume, self.bank_music.started) = sh.bank;
            (self.intensity.register2, self.intensity.excite) = sh.intensity;
            self.fade = neopets_sim::scene::Fade { colour: sh.fade.0, alpha: sh.fade.1, target: sh.fade.2, step: sh.fade.3 };
            self.bars = neopets_sim::scene::Bars { progress: sh.bars.0, target: sh.bars.1, step: sh.bars.2, done: sh.bars.3, held: sh.bars.4 };
            match &sh.monster {
                Some((t, target, name, nw, frame, fill)) => {
                    let m = &mut self.monster_bar;
                    m.t = *t;
                    m.target = Some(*target);
                    m.name = String::from_utf8_lossy(name).into_owned();
                    m.name_width = *nw;
                    m.frame = *frame;
                    m.fill = *fill;
                }
                None => self.monster_bar.target = None,
            }
            self.message = sh.message.map(|(line, age)| (line, self.ticks.saturating_sub(age as u64)));
            self.shake_now = sh.shake.map(|(side, up, aim)| neopets_sim::shake::Offset { side, up, aim });
        }
        // The host's breakables (a break here as the host's) and world items.
        if let Some(ws) = s.world.as_ref() {
            if ws.pack == self.world.pack {
                for (i, &st) in ws.breakables.iter().enumerate() {
                    let here = self.world.breakables.get(i).map(|b| b.state);
                    match (here, st) {
                        (Some(h), 2) if h != 2 => self.net_break(i),
                        (Some(h), st) if h != st as i32 && st != 2 => self.world.breakables[i].state = st as i32,
                        _ => {}
                    }
                }
            }
            let packs: Vec<u32> = self.pools.keys().copied().collect();
            for pack in packs {
                if pack != ws.pack && self.other.as_ref().is_none_or(|o| o.pack != pack) {
                    continue;
                }
                let pool = self.pools.entry(pack).or_default();
                let mine: Vec<(usize, u16)> = pool.slots.iter().enumerate().filter_map(|(k, it)| it.as_ref().map(|it| (k, it.id))).collect();
                for (k, id) in mine {
                    if !ws.items.iter().any(|e| e.0 == pack && e.1 as usize == k && e.2.id == id) {
                        pool.slots[k] = None;
                    }
                }
            }
            for (pack, k, it) in &ws.items {
                if *pack != ws.pack && self.other.as_ref().is_none_or(|o| o.pack != *pack) {
                    continue;
                }
                let pool = self.pools.entry(*pack).or_default();
                let k = *k as usize;
                if pool.slots.len() <= k {
                    continue;
                }
                if pool.slots[k].is_none_or(|w| w.id != it.id) {
                    pool.slots[k] = Some(neopets_sim::items::WorldItem {
                        id: it.id,
                        position: it.position,
                        state: it.state,
                        spin: 0.0,
                        vy: 0.0,
                        hs: 0.0,
                        life: 0.0,
                        lifetime: 0.0,
                        age: 10.0,
                        reference: it.reference,
                        angle: it.angle,
                        direction: it.direction,
                        scale: it.scale,
                        s0: it.scale,
                        s1: it.scale,
                        delay: 0.0,
                        token: 0,
                        flag: 0,
                        map: 0,
                        charm: false,
                        price: it.price,
                        reserved: false,
                    });
                }
            }
        }
        // The host's animated objects: states, speeds and flags from the world sync, the moving
        // ones' times from each state.
        if let Some(ws) = s.world.as_ref().filter(|w| w.pack == self.world.pack) {
            for &(i, st, speed, bits) in &ws.objects {
                if let Some(o) = self.world.object_anims.get_mut(i as usize) {
                    (o.state, o.speed, o.drawn, o.scripted) = (st, speed, bits & 1 != 0, bits & 2 != 0);
                }
            }
        }
        if s.pack == self.world.pack {
            for &(i, t) in &s.anim_times {
                if let Some(o) = self.world.object_anims.get_mut(i as usize) {
                    o.time = t;
                }
            }
        }
        for &(pack, k, at, angle) in &s.moving {
            if let Some(Some(it)) = self.pools.get_mut(&pack).and_then(|p| p.slots.get_mut(k as usize)) {
                it.position = at;
                it.angle = angle;
            }
        }
        // The host's shots in place of this side's.
        self.shots.slots = Default::default();
        for (i, sh) in &s.shots {
            if let Some(slot) = self.shots.slots.get_mut(*i as usize) {
                *slot = Some(*sh);
            }
        }
        // The host's petpet: the same type here (its package loads and its character is then
        // placed by the host's), its times kept to the host's so it leaves only with the host's;
        // none when the host has none.
        match (s.pet, self.petpet.as_mut()) {
            (Some((ty, t, f)), Some(p)) if p.type_id == ty => (p.t, p.f) = (t, f),
            (Some((ty, t, f)), _) => self.activate_pet(ty, t, f),
            (None, Some(_)) => self.deactivate_pet(),
            (None, None) => {}
        }
        // The party's equipment and cloak (the joiner's item screen shows them).
        (self.equipment.entries, self.equipment.motes, self.cloak_worn) = s.equip;
        self.state.flags = s.flags.iter().copied().collect();
        self.state.items = s.items.iter().copied().collect();
        self.state.neopoints = s.neopoints;
    }
}

