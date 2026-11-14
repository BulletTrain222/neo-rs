//! Online play's messages (docs/MULTIPLAYER.md; this build's feature): the byte writer and
//! reader, and the encoding of each message's records (state, input, events, the world and story
//! syncs).

use super::*;

#[derive(Default)]
pub(super) struct Writer(pub(super) Vec<u8>);

impl Writer {
    pub(super) fn new(kind: u8) -> Self {
        let mut w = Self(MAGIC.to_vec());
        w.u16(PROTOCOL);
        w.u8(kind);
        w
    }
    pub(super) fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub(super) fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(super) fn v3(&mut self, v: [f32; 3]) {
        for x in v {
            self.f32(x);
        }
    }
    /// A direction (each part within -1 to 1) in three 16-bit parts (a step of 1/32767).
    pub(super) fn dir(&mut self, v: [f32; 3]) {
        for x in v {
            self.u16((x.clamp(-1.0, 1.0) * 32767.0).round() as i16 as u16);
        }
    }
    /// A clip or other id: 16 bits, or 0xFFFF and 32 bits when larger.
    pub(super) fn id(&mut self, v: u32) {
        if v < 0xFFFF {
            self.u16(v as u16);
        } else {
            self.u16(0xFFFF);
            self.u32(v);
        }
    }
    /// A health or magic number: 16 bits, clamped.
    pub(super) fn small(&mut self, v: i32) {
        self.u16(v.clamp(i16::MIN as i32, i16::MAX as i32) as i16 as u16);
    }
    pub(super) fn bytes(&mut self, b: &[u8]) {
        self.u32(b.len() as u32);
        self.0.extend_from_slice(b);
    }
    pub(super) fn text(&mut self, t: &str) {
        self.bytes(t.as_bytes());
    }
}

pub(super) struct Reader<'a> {
    pub(super) b: &'a [u8],
    pub(super) at: usize,
}

impl<'a> Reader<'a> {
    /// The message kind, if the bytes start with the magic and this protocol.
    pub(super) fn open(b: &'a [u8]) -> Option<(u8, Self)> {
        if b.len() < 7 || b[..4] != MAGIC || u16::from_le_bytes([b[4], b[5]]) != PROTOCOL {
            return None;
        }
        Some((b[6], Self { b, at: 7 }))
    }
    pub(super) fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.at..self.at + n)?;
        self.at += n;
        Some(s)
    }
    pub(super) fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    pub(super) fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    pub(super) fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    pub(super) fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    pub(super) fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    pub(super) fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    pub(super) fn v3(&mut self) -> Option<[f32; 3]> {
        Some([self.f32()?, self.f32()?, self.f32()?])
    }
    pub(super) fn dir(&mut self) -> Option<[f32; 3]> {
        let mut d = || self.u16().map(|v| v as i16 as f32 / 32767.0);
        Some([d()?, d()?, d()?])
    }
    pub(super) fn id(&mut self) -> Option<u32> {
        match self.u16()? {
            0xFFFF => self.u32(),
            v => Some(v as u32),
        }
    }
    pub(super) fn small(&mut self) -> Option<i32> {
        Some(self.u16()? as i16 as i32)
    }
    pub(super) fn bytes(&mut self) -> Option<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    pub(super) fn text(&mut self) -> Option<String> {
        Some(String::from_utf8_lossy(self.bytes()?).into_owned())
    }
}

/// One hero in a state message.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeroState {
    pub who: u8,
    pub position: [f32; 3],
    pub facing: [f32; 3],
    pub state: u8,
    pub clip: u32,
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub flash: u8,
    pub down: bool,
    /// The clip's playback rate (its walk or run speed), so the joiner plays it as fast.
    pub rate: f32,
    /// The weapon in hand (not stowed) and the shield raised, for the held items drawn.
    pub drawn: bool,
    pub raised: bool,
}

/// One character in a state message, matched on the joiner by world, slot and ordinal.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CharState {
    pub pack: u32,
    pub slot: u8,
    pub ordinal: u32,
    pub position: [f32; 3],
    pub facing: [f32; 3],
    pub clip: u32,
    /// The clip's play rate (an enemy's walk or run speed), so the joiner plays it as fast.
    pub rate: f32,
    pub hp: i32,
    pub dead: bool,
    pub inactive: bool,
}

/// The host's game after one tick, as the joiner applies it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    pub tick: u64,
    pub generation: u32,
    pub pack: u32,
    pub heroes: [HeroState; 2],
    /// Online with more than one joiner: the other joiners' heroes (players 3 and 4 as this
    /// joiner sees them; `heroes[1]` is always the receiving joiner's own).
    pub more: Vec<HeroState>,
    /// Player 2's follow camera: its eye and focus.
    pub eye: [f32; 3],
    pub focus: [f32; 3],
    pub characters: Vec<CharState>,
    pub flags: Vec<u16>,
    pub items: Vec<(u16, u32)>,
    pub neopoints: i32,
    /// The talk box: open, its text and page, the reveal, choice mode, the speaker's name and
    /// colour and the box labels (the joiner shows the host's, not its own).
    pub talk: Option<Talk>,
    /// A scripted camera running on the host: its view (eye, forward, up, focal).
    pub cut: Option<([f32; 3], [f32; 3], [f32; 3], f32)>,
    /// The host's events the joiner has not acknowledged yet, numbered (net.rs event channel).
    pub events: Vec<(u32, Event)>,
    /// The host's live shots by pool slot.
    pub shots: Vec<(u8, neopets_sim::projectile::Shot)>,
    /// The story revision (flags, items and Neopoints, counted up at each change) and whether
    /// this state carries them (only until the joiner acknowledges the revision).
    pub story: u32,
    pub has_story: bool,
    /// Player 2's quick-item panel (open, entries, selection, marker) and Mote panel (open,
    /// selection), for the joiner's HUD.
    pub panels: (bool, [i32; 4], u8, i32, bool, u8),
    /// Player 2's door prompt (its Square caption) for the joiner's HUD.
    pub p2_prompt: Option<u16>,
    /// The world's breakables and items (revision counted up at each change; carried until the
    /// joiner acknowledges it), and the moving dropped items' places every tick.
    pub world_rev: u32,
    pub world: Option<WorldSync>,
    pub moving: Vec<(u32, u16, [f32; 3], f32)>,
    /// The current world's animating objects' times (index, seconds).
    pub anim_times: Vec<(u16, f32)>,
    /// What the host's screen and music show that the joiner shows too.
    pub show: Show,
    /// The host's page while its states run: 0 play, 2 the game-over page (a paused host
    /// sends pings instead).
    pub page: u8,
    /// The receiving joiner's seat (0: player 2), for the players' numbers it shows.
    pub seat: u8,
    /// The round trip the host measures, in milliseconds (shown on both sides).
    pub rtt: u16,
    /// The last joiner item-screen command the host carried out (its number).
    pub cmd_ack: u8,
    /// The party's equipment (each hero's weapon, shield and armour, each hero's Motes) and the
    /// cloak, for the joiner's item screen.
    pub equip: ([[Option<u16>; 2]; 3], [[Option<u16>; 3]; 2], bool),
    /// The host's live petpet (type, time left T, full time F): the joiner keeps the same one.
    pub pet: Option<(i32, i32, i32)>,
    /// The current world's movable blocks away from their home (index, position): moving ones
    /// every state, the others now and then.
    pub blocks: Vec<(u16, [f32; 3])>,
    /// The host's multiplayer colours by player number (hero_colour.rs).
    pub colours: [u8; 4],
}

/// The breakables of the current world (their states) and the live items of each resident world
/// (pack, slot, and the item as the joiner draws it: id, place, state, angle, scale, facing).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldSync {
    pub pack: u32,
    pub breakables: Vec<u8>,
    pub items: Vec<(u32, u16, NetItem)>,
    /// The host's characters (pack, slot, ordinal): the joiner hides any other.
    pub chars: Vec<(u32, u8, u32)>,
    /// The current world's animated objects (SPEC-0028): index, state, speed, drawn and scripted
    /// flags (a gate player 2 opened opens on the joiner too).
    pub objects: Vec<(u16, i32, f32, u8)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NetItem {
    pub id: u16,
    pub position: [f32; 3],
    pub state: u8,
    pub angle: f32,
    pub scale: f32,
    pub direction: [f32; 3],
    pub price: i16,
    /// The ground height a hovering item bobs about (the joiner bobs it itself).
    pub reference: f32,
}

impl WorldSync {
    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.pack);
        w.u16(self.breakables.len() as u16);
        w.0.extend_from_slice(&self.breakables);
        w.u16(self.items.len() as u16);
        for (pack, slot, it) in &self.items {
            w.u32(*pack);
            w.u16(*slot);
            w.u16(it.id);
            w.v3(it.position);
            w.u8(it.state);
            w.f32(it.angle);
            w.f32(it.scale);
            w.v3(it.direction);
            w.u16(it.price as u16);
            w.f32(it.reference);
        }
        w.u16(self.chars.len() as u16);
        for &(p, s, o) in &self.chars {
            w.u32(p);
            w.u8(s);
            w.u32(o);
        }
        w.u16(self.objects.len() as u16);
        for &(i, st, speed, bits) in &self.objects {
            w.u16(i);
            w.i32(st);
            w.f32(speed);
            w.u8(bits);
        }
    }

    pub(super) fn read(r: &mut Reader) -> Option<Self> {
        let pack = r.u32()?;
        let n = r.u16()? as usize;
        let breakables = r.take(n)?.to_vec();
        let n = r.u16()? as usize;
        let mut items = Vec::with_capacity(n.min(512));
        for _ in 0..n {
            let (p, s) = (r.u32()?, r.u16()?);
            items.push((p, s, NetItem { id: r.u16()?, position: r.v3()?, state: r.u8()?, angle: r.f32()?, scale: r.f32()?, direction: r.v3()?, price: r.u16()? as i16, reference: r.f32()? }));
        }
        let n = r.u16()? as usize;
        let mut chars = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            chars.push((r.u32()?, r.u8()?, r.u32()?));
        }
        let n = r.u16()? as usize;
        let mut objects = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            objects.push((r.u16()?, r.i32()?, r.f32()?, r.u8()?));
        }
        Some(Self { pack, breakables, items, chars, objects })
    }
}

/// The host's music, fade, letterbox, monster bar, message and camera shake after a tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Show {
    /// The two stream channels (stream, volume).
    pub music: [(Option<u32>, i32); 2],
    /// The bank music (playing, volume, started this tick) and the intensity's register 2 and G.
    pub bank: (Option<u32>, i32, bool),
    pub intensity: (u8, u8),
    /// The fade (colour, alpha, target, step) and the letterbox (progress, target, step, done, held).
    pub fade: ([u8; 3], f32, f32, f32),
    pub bars: (f32, f32, f32, bool, bool),
    /// The monster bar (time left, target, name, name width, frame and fill widths).
    pub monster: Option<(i32, u64, Vec<u8>, i32, i32, i32)>,
    /// The level message line and how many ticks it has shown.
    pub message: Option<(u16, u32)>,
    /// The camera shake (side, up, aim).
    pub shake: Option<(f32, f32, Option<f32>)>,
}

impl Show {
    pub(super) fn write(&self, w: &mut Writer) {
        for (stream, volume) in self.music {
            w.u32(stream.unwrap_or(u32::MAX));
            w.i32(volume);
        }
        w.u32(self.bank.0.unwrap_or(u32::MAX));
        w.i32(self.bank.1);
        w.u8(self.bank.2 as u8);
        w.u8(self.intensity.0);
        w.u8(self.intensity.1);
        w.0.extend_from_slice(&self.fade.0);
        w.f32(self.fade.1);
        w.f32(self.fade.2);
        w.f32(self.fade.3);
        w.f32(self.bars.0);
        w.f32(self.bars.1);
        w.f32(self.bars.2);
        w.u8(self.bars.3 as u8 | (self.bars.4 as u8) << 1);
        match &self.monster {
            Some((t, target, name, nw, frame, fill)) => {
                w.u8(1);
                w.i32(*t);
                w.u64(*target);
                w.bytes(name);
                w.i32(*nw);
                w.i32(*frame);
                w.i32(*fill);
            }
            None => w.u8(0),
        }
        match self.message {
            Some((line, age)) => {
                w.u8(1);
                w.u16(line);
                w.u32(age);
            }
            None => w.u8(0),
        }
        match self.shake {
            Some((side, up, aim)) => {
                w.u8(1);
                w.f32(side);
                w.f32(up);
                write_opt_f32(w, aim);
            }
            None => w.u8(0),
        }
    }

    pub(super) fn read(r: &mut Reader) -> Option<Self> {
        let stream = |v: u32| (v != u32::MAX).then_some(v);
        let music = [(stream(r.u32()?), r.i32()?), (stream(r.u32()?), r.i32()?)];
        let bank = (stream(r.u32()?), r.i32()?, r.u8()? != 0);
        let intensity = (r.u8()?, r.u8()?);
        let fade = ([r.u8()?, r.u8()?, r.u8()?], r.f32()?, r.f32()?, r.f32()?);
        let (a, b, c, bits) = (r.f32()?, r.f32()?, r.f32()?, r.u8()?);
        let bars = (a, b, c, bits & 1 != 0, bits & 2 != 0);
        let monster = match r.u8()? {
            1 => Some((r.i32()?, r.u64()?, r.bytes()?.to_vec(), r.i32()?, r.i32()?, r.i32()?)),
            _ => None,
        };
        let message = match r.u8()? {
            1 => Some((r.u16()?, r.u32()?)),
            _ => None,
        };
        let shake = match r.u8()? {
            1 => Some((r.f32()?, r.f32()?, read_opt_f32(r)?)),
            _ => None,
        };
        Some(Self { music, bank, intensity, fade, bars, monster, message, shake })
    }
}

/// Something that happened on the host for the joiner to play once: numbered, resent with each
/// state until the joiner's input acknowledges it.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// A sound request (start, change or stop of an instance), as the host's sound manager made it.
    Sound(neopets_sim::sound::Request),
    /// An operation on the host's gameplay effects (its handles; effect.rs `Share`).
    Fx(neopets_sim::effect::Op),
    /// A stinger clip, and an emote clip (bank, start, length).
    Stinger(i32),
    Emote(u32, u32, u32),
    /// A movie the host's scripts queued.
    Movie(u16),
    /// Where the host's voice `id` sounds from (emitter, base volume, radii), sent just before its
    /// start: the joiner works out the volume and pan from its own listener.
    Voice { id: u32, emitter: [f32; 3], base: i32, r: f32, r0: f32 },
    /// Player 2's motor send (level, countdown ticks) for the joiner's controller.
    Rumble(u8, i32),
    /// A line for the joiner (`NOTICE_SAVED`: the host saved the game).
    Notice(u8),
}

/// `Event::Notice` lines.
pub const NOTICE_SAVED: u8 = 1;

/// The joiner's words for a host notice.
pub fn notice_text(n: u8) -> Option<&'static str> {
    match n {
        NOTICE_SAVED => Some("The host saved the game."),
        _ => None,
    }
}

/// The joiner's listener (sound.rs): its view's eye with the rows right, up and forward.
pub fn listener_of(eye: [f32; 3], forward: [f32; 3]) -> neopets_sim::sound::Listener {
    let n = (forward[0] * forward[0] + forward[1] * forward[1] + forward[2] * forward[2]).sqrt().max(1e-6);
    let f = forward.map(|v| v / n);
    let up = [0.0, 1.0, 0.0];
    let r = [f[1] * up[2] - f[2] * up[1], f[2] * up[0] - f[0] * up[2], f[0] * up[1] - f[1] * up[0]];
    let rn = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt().max(1e-6);
    let r = r.map(|v| v / rn);
    let u = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
    neopets_sim::sound::Listener::from_camera(eye, [r, u, f])
}

/// The joiner: a host sound request with the volume and pan worked out again from this side's
/// listener, for a voice whose source is known (`voices`; a stop forgets it).
pub fn hear(r: neopets_sim::sound::Request, voices: &mut std::collections::HashMap<usize, ([f32; 3], i32, f32, f32)>, l: &neopets_sim::sound::Listener) -> neopets_sim::sound::Request {
    use neopets_sim::sound::Request;
    let here = |id: usize| voices.get(&id).map(|&(e, base, r, r0)| (l.volume(e, base, r, r0).unwrap_or(0).max(0), Some(l.pan(e))));
    match r {
        Request::Play { id, block, cue, volume, pan } => {
            let (volume, pan) = here(id).unwrap_or((volume, pan));
            Request::Play { id, block, cue, volume, pan }
        }
        Request::Set { id, volume, pan } => {
            let (volume, pan) = here(id).unwrap_or((volume, pan));
            Request::Set { id, volume, pan }
        }
        Request::Stop { id } => {
            voices.remove(&id);
            r
        }
    }
}

pub(super) fn write_frame(w: &mut Writer, f: &[[f32; 3]; 3]) {
    for row in f {
        w.v3(*row);
    }
}

pub(super) fn read_frame(r: &mut Reader) -> Option<[[f32; 3]; 3]> {
    Some([r.v3()?, r.v3()?, r.v3()?])
}

pub(super) fn write_opt_f32(w: &mut Writer, v: Option<f32>) {
    match v {
        Some(x) => {
            w.u8(1);
            w.f32(x);
        }
        None => w.u8(0),
    }
}

pub(super) fn read_opt_f32(r: &mut Reader) -> Option<Option<f32>> {
    Some(match r.u8()? {
        1 => Some(r.f32()?),
        _ => None,
    })
}

pub(super) fn write_op(w: &mut Writer, op: &neopets_sim::effect::Op) {
    use neopets_sim::effect::Op;
    match op {
        Op::Add { handle, def, position, frame, state, cull, scale } => {
            w.u8(10);
            w.u32(*handle as u32);
            w.u32(def.map_or(u32::MAX, |d| d as u32));
            w.v3(*position);
            write_frame(w, frame);
            w.u8(*state);
            write_opt_f32(w, *cull);
            write_opt_f32(w, *scale);
        }
        Op::OneShot { id, position, frame } => {
            w.u8(11);
            w.i32(*id);
            w.v3(*position);
            write_frame(w, frame);
        }
        Op::Update { handle, listener } => {
            w.u8(12);
            w.u32(*handle as u32);
            match listener {
                Some((a, b)) => {
                    w.u8(1);
                    w.v3(*a);
                    w.v3(*b);
                }
                None => w.u8(0),
            }
        }
        Op::Stop { handle } => {
            w.u8(13);
            w.u32(*handle as u32);
        }
        Op::Release { handle } => {
            w.u8(14);
            w.u32(*handle as u32);
        }
        Op::Target { handle, at } => {
            w.u8(15);
            w.u32(*handle as u32);
            w.v3(*at);
        }
        Op::Position { handle, at } => {
            w.u8(16);
            w.u32(*handle as u32);
            w.v3(*at);
        }
    }
}

pub(super) fn read_op(kind: u8, r: &mut Reader) -> Option<neopets_sim::effect::Op> {
    use neopets_sim::effect::Op;
    Some(match kind {
        10 => Op::Add {
            handle: r.u32()? as usize,
            def: Some(r.u32()?).filter(|&d| d != u32::MAX).map(|d| d as usize),
            position: r.v3()?,
            frame: read_frame(r)?,
            state: r.u8()?,
            cull: read_opt_f32(r)?,
            scale: read_opt_f32(r)?,
        },
        11 => Op::OneShot { id: r.i32()?, position: r.v3()?, frame: read_frame(r)? },
        12 => Op::Update {
            handle: r.u32()? as usize,
            listener: match r.u8()? {
                1 => Some((r.v3()?, r.v3()?)),
                _ => None,
            },
        },
        13 => Op::Stop { handle: r.u32()? as usize },
        14 => Op::Release { handle: r.u32()? as usize },
        15 => Op::Target { handle: r.u32()? as usize, at: r.v3()? },
        16 => Op::Position { handle: r.u32()? as usize, at: r.v3()? },
        _ => return None,
    })
}

/// A shot of the host's pool, for the joiner to draw.
pub(super) fn write_shot(w: &mut Writer, i: u8, sh: &neopets_sim::projectile::Shot) {
    let s = &sh.s;
    w.u8(i);
    w.i32(s.kind);
    w.i32(s.visual);
    w.i32(s.element);
    w.u64(s.owner.map_or(u64::MAX, |o| o));
    w.v3(s.start);
    w.v3(s.velocity);
    w.i32(s.lifetime);
    w.f32(s.damage);
    w.f32(s.radius);
    w.f32(s.spin);
    w.f32(s.area_radius);
    w.i32(s.area_duration);
    w.i32(s.key);
    w.u8(s.ignore_world as u8);
    w.i32(s.status.0);
    w.i32(s.status.1);
    w.i32(s.status.2);
    for e in s.effects {
        w.i32(e);
    }
    w.i32(s.trail);
    w.v3(sh.position);
    w.v3(sh.velocity);
    w.i32(sh.life);
    w.u8(sh.area as u8 | (sh.at_owner as u8) << 1);
}

pub(super) fn read_shot(r: &mut Reader) -> Option<(u8, neopets_sim::projectile::Shot)> {
    let i = r.u8()?;
    let s = neopets_sim::projectile::Setup {
        kind: r.i32()?,
        visual: r.i32()?,
        element: r.i32()?,
        owner: Some(r.u64()?).filter(|&o| o != u64::MAX),
        start: r.v3()?,
        velocity: r.v3()?,
        lifetime: r.i32()?,
        damage: r.f32()?,
        radius: r.f32()?,
        spin: r.f32()?,
        area_radius: r.f32()?,
        area_duration: r.i32()?,
        key: r.i32()?,
        ignore_world: r.u8()? != 0,
        status: (r.i32()?, r.i32()?, r.i32()?),
        effects: [r.i32()?, r.i32()?, r.i32()?, r.i32()?, r.i32()?, r.i32()?],
        trail: r.i32()?,
    };
    let (position, velocity, life, bits) = (r.v3()?, r.v3()?, r.i32()?, r.u8()?);
    Some((i, neopets_sim::projectile::Shot::remote(s, position, velocity, life, bits & 1 != 0, bits & 2 != 0)))
}

pub(super) fn write_pan(w: &mut Writer, pan: Option<i32>) {
    match pan {
        Some(p) => {
            w.u8(1);
            w.i32(p);
        }
        None => w.u8(0),
    }
}

pub(super) fn read_pan(r: &mut Reader) -> Option<Option<i32>> {
    Some(match r.u8()? {
        1 => Some(r.i32()?),
        _ => None,
    })
}

impl Event {
    pub(super) fn write(&self, w: &mut Writer) {
        use neopets_sim::sound::Request;
        match self {
            Event::Sound(Request::Play { id, block, cue, volume, pan }) => {
                w.u8(1);
                w.u32(*id as u32);
                w.u32(*block);
                w.i32(*cue);
                w.i32(*volume);
                write_pan(w, *pan);
            }
            Event::Sound(Request::Set { id, volume, pan }) => {
                w.u8(2);
                w.u32(*id as u32);
                w.i32(*volume);
                write_pan(w, *pan);
            }
            Event::Sound(Request::Stop { id }) => {
                w.u8(3);
                w.u32(*id as u32);
            }
            Event::Fx(op) => write_op(w, op),
            Event::Stinger(k) => {
                w.u8(20);
                w.i32(*k);
            }
            Event::Movie(m) => {
                w.u8(22);
                w.u16(*m);
            }
            Event::Notice(n) => {
                w.u8(23);
                w.u8(*n);
            }
            Event::Rumble(level, count) => {
                w.u8(5);
                w.u8(*level);
                w.i32(*count);
            }
            Event::Voice { id, emitter, base, r, r0 } => {
                w.u8(4);
                w.u32(*id);
                w.v3(*emitter);
                w.i32(*base);
                w.f32(*r);
                w.f32(*r0);
            }
            Event::Emote(bank, start, len) => {
                w.u8(21);
                w.u32(*bank);
                w.u32(*start);
                w.u32(*len);
            }
        }
    }

    pub(super) fn read(r: &mut Reader) -> Option<Self> {
        use neopets_sim::sound::Request;
        Some(match r.u8()? {
            1 => Event::Sound(Request::Play { id: r.u32()? as usize, block: r.u32()?, cue: r.i32()?, volume: r.i32()?, pan: read_pan(r)? }),
            2 => Event::Sound(Request::Set { id: r.u32()? as usize, volume: r.i32()?, pan: read_pan(r)? }),
            3 => Event::Sound(Request::Stop { id: r.u32()? as usize }),
            k @ 10..=16 => Event::Fx(read_op(k, r)?),
            20 => Event::Stinger(r.i32()?),
            21 => Event::Emote(r.u32()?, r.u32()?, r.u32()?),
            22 => Event::Movie(r.u16()?),
            23 => Event::Notice(r.u8()?),
            5 => Event::Rumble(r.u8()?, r.i32()?),
            4 => Event::Voice { id: r.u32()?, emitter: r.v3()?, base: r.i32()?, r: r.f32()?, r0: r.f32()? },
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Talk {
    pub open: bool,
    pub text: Vec<u8>,
    pub start: u32,
    pub end: u32,
    pub p: f32,
    pub choice: bool,
    pub speaker: Vec<u8>,
    pub colour: [u8; 3],
    pub labels: (Vec<u8>, Vec<u8>),
}

pub(super) fn write_hero(w: &mut Writer, h: &HeroState) {
    w.u8(h.who & 0x1F | (h.down as u8) << 7 | (h.drawn as u8) << 6 | (h.raised as u8) << 5);
    w.v3(h.position);
    w.dir(h.facing);
    w.u8(h.state);
    w.id(h.clip);
    w.small(h.hp);
    w.small(h.hp_max);
    w.small(h.mp);
    w.small(h.mp_max);
    w.u8(h.flash);
    w.u16((h.rate * 1000.0).round().clamp(0.0, 65535.0) as u16);
}

pub(super) fn read_hero(r: &mut Reader) -> Option<HeroState> {
    let who = r.u8()?;
    Some(HeroState { who: who & 0x1F, down: who & 0x80 != 0, drawn: who & 0x40 != 0, raised: who & 0x20 != 0, position: r.v3()?, facing: r.dir()?, state: r.u8()?, clip: r.id()?, hp: r.small()?, hp_max: r.small()?, mp: r.small()?, mp_max: r.small()?, flash: r.u8()?, rate: r.u16()? as f32 / 1000.0 })
}

impl State {
    pub(super) fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new(STATE);
        w.u64(self.tick);
        w.u32(self.generation);
        w.u32(self.pack);
        for h in &self.heroes {
            write_hero(&mut w, h);
        }
        w.u8(self.more.len().min(2) as u8);
        for h in self.more.iter().take(2) {
            write_hero(&mut w, h);
        }
        w.v3(self.eye);
        w.v3(self.focus);
        // The characters: the worlds they belong to once, then each by its world's index.
        let mut packs: Vec<u32> = Vec::new();
        for c in &self.characters {
            if !packs.contains(&c.pack) {
                packs.push(c.pack);
            }
        }
        w.u8(packs.len().min(255) as u8);
        for &p in packs.iter().take(255) {
            w.u32(p);
        }
        let shown: Vec<&CharState> = self.characters.iter().filter(|c| packs.iter().take(255).any(|&p| p == c.pack)).collect();
        w.u16(shown.len() as u16);
        for c in shown {
            w.u8(packs.iter().position(|&p| p == c.pack).unwrap_or(0) as u8);
            w.u8(c.slot);
            w.id(c.ordinal);
            w.v3(c.position);
            w.dir(c.facing);
            w.id(c.clip);
            w.u16((c.rate * 1000.0).round().clamp(0.0, 65535.0) as u16);
            w.small(c.hp);
            w.u8(c.dead as u8 | (c.inactive as u8) << 1);
        }
        w.u32(self.story);
        w.u8(self.has_story as u8);
        if self.has_story {
            w.u32(self.flags.len() as u32);
            for &f in &self.flags {
                w.u16(f);
            }
            w.u32(self.items.len() as u32);
            for &(id, n) in &self.items {
                w.u16(id);
                w.u32(n);
            }
            w.i32(self.neopoints);
        }
        match &self.talk {
            Some(t) => {
                w.u8(1);
                w.u8(t.open as u8);
                w.bytes(&t.text);
                w.u32(t.start);
                w.u32(t.end);
                w.f32(t.p);
                w.u8(t.choice as u8);
                w.bytes(&t.speaker);
                w.0.extend_from_slice(&t.colour);
                w.bytes(&t.labels.0);
                w.bytes(&t.labels.1);
            }
            None => w.u8(0),
        }
        match self.cut {
            Some((e, f, u, focal)) => {
                w.u8(1);
                w.v3(e);
                w.v3(f);
                w.v3(u);
                w.f32(focal);
            }
            None => w.u8(0),
        }
        w.u16(self.events.len() as u16);
        for (n, e) in &self.events {
            w.u32(*n);
            e.write(&mut w);
        }
        w.u8(self.shots.len() as u8);
        for (i, sh) in &self.shots {
            write_shot(&mut w, *i, sh);
        }
        self.show.write(&mut w);
        let pn = &self.panels;
        w.u8(pn.0 as u8 | (pn.4 as u8) << 1);
        for e in pn.1 {
            w.i32(e);
        }
        w.u8(pn.2);
        w.i32(pn.3);
        w.u8(pn.5);
        w.u16(self.p2_prompt.map_or(u16::MAX, |p| p));
        w.u32(self.world_rev);
        match &self.world {
            Some(ws) => {
                w.u8(1);
                ws.write(&mut w);
            }
            None => w.u8(0),
        }
        w.u16(self.moving.len() as u16);
        for (pack, slot, at, angle) in &self.moving {
            w.u32(*pack);
            w.u16(*slot);
            w.v3(*at);
            w.f32(*angle);
        }
        w.u16(self.anim_times.len() as u16);
        for &(i, t) in &self.anim_times {
            w.u16(i);
            w.f32(t);
        }
        w.u8(self.page);
        w.u8(self.seat);
        w.u16(self.rtt);
        w.u8(self.cmd_ack);
        let opt = |w: &mut Writer, v: Option<u16>| w.u16(v.unwrap_or(u16::MAX));
        for row in self.equip.0 {
            for v in row {
                opt(&mut w, v);
            }
        }
        for row in self.equip.1 {
            for v in row {
                opt(&mut w, v);
            }
        }
        w.u8(self.equip.2 as u8);
        match self.pet {
            Some((ty, t, f)) => {
                w.u8(1);
                w.i32(ty);
                w.i32(t);
                w.i32(f);
            }
            None => w.u8(0),
        }
        w.u8(self.blocks.len().min(255) as u8);
        for &(i, p) in self.blocks.iter().take(255) {
            w.u16(i);
            w.v3(p);
        }
        for c in self.colours {
            w.u8(c);
        }
        w.0
    }

    pub(super) fn decode(r: &mut Reader) -> Option<Self> {
        let tick = r.u64()?;
        let generation = r.u32()?;
        let pack = r.u32()?;
        let heroes = [read_hero(r)?, read_hero(r)?];
        let n = r.u8()?.min(2);
        let more = (0..n).map(|_| read_hero(r)).collect::<Option<Vec<_>>>()?;
        let (eye, focus) = (r.v3()?, r.v3()?);
        let np = r.u8()? as usize;
        let mut packs = Vec::with_capacity(np);
        for _ in 0..np {
            packs.push(r.u32()?);
        }
        let n = r.u16()? as usize;
        let mut characters = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            let pack = *packs.get(r.u8()? as usize)?;
            let (slot, ordinal, position, facing, clip, rate, hp, bits) = (r.u8()?, r.id()?, r.v3()?, r.dir()?, r.id()?, r.u16()?, r.small()?, r.u8()?);
            characters.push(CharState { pack, slot, ordinal, position, facing, clip, rate: rate as f32 / 1000.0, hp, dead: bits & 1 != 0, inactive: bits & 2 != 0 });
        }
        let story = r.u32()?;
        let has_story = r.u8()? != 0;
        let (mut flags, mut items, mut neopoints) = (Vec::new(), Vec::new(), 0);
        if has_story {
            let n = r.u32()? as usize;
            flags = (0..n.min(65536)).map(|_| r.u16()).collect::<Option<Vec<_>>>()?;
            let n = r.u32()? as usize;
            items = (0..n.min(65536)).map(|_| Some((r.u16()?, r.u32()?))).collect::<Option<Vec<_>>>()?;
            neopoints = r.i32()?;
        }
        let talk = match r.u8()? {
            1 => Some(Talk {
                open: r.u8()? != 0,
                text: r.bytes()?.to_vec(),
                start: r.u32()?,
                end: r.u32()?,
                p: r.f32()?,
                choice: r.u8()? != 0,
                speaker: r.bytes()?.to_vec(),
                colour: [r.u8()?, r.u8()?, r.u8()?],
                labels: (r.bytes()?.to_vec(), r.bytes()?.to_vec()),
            }),
            _ => None,
        };
        let cut = match r.u8()? {
            1 => Some((r.v3()?, r.v3()?, r.v3()?, r.f32()?)),
            _ => None,
        };
        let count = r.u16()? as usize;
        let mut events = Vec::with_capacity(count.min(256));
        for _ in 0..count {
            events.push((r.u32()?, Event::read(r)?));
        }
        let count = r.u8()? as usize;
        let mut shots = Vec::with_capacity(count.min(16));
        for _ in 0..count {
            shots.push(read_shot(r)?);
        }
        let show = Show::read(r)?;
        let bits = r.u8()?;
        let entries = [r.i32()?, r.i32()?, r.i32()?, r.i32()?];
        let panels = (bits & 1 != 0, entries, r.u8()?, r.i32()?, bits & 2 != 0, r.u8()?);
        let p2_prompt = Some(r.u16()?).filter(|&p| p != u16::MAX);
        let world_rev = r.u32()?;
        let world = match r.u8()? {
            1 => Some(WorldSync::read(r)?),
            _ => None,
        };
        let n = r.u16()? as usize;
        let mut moving = Vec::with_capacity(n.min(256));
        for _ in 0..n {
            moving.push((r.u32()?, r.u16()?, r.v3()?, r.f32()?));
        }
        let n = r.u16()? as usize;
        let mut anim_times = Vec::with_capacity(n.min(256));
        for _ in 0..n {
            anim_times.push((r.u16()?, r.f32()?));
        }
        let page = r.u8()?;
        let seat = r.u8()?;
        let rtt = r.u16()?;
        let cmd_ack = r.u8()?;
        let mut opt = || r.u16().map(|v| (v != u16::MAX).then_some(v));
        let mut entries = [[None; 2]; 3];
        for row in entries.iter_mut() {
            for v in row.iter_mut() {
                *v = opt()?;
            }
        }
        let mut motes = [[None; 3]; 2];
        for row in motes.iter_mut() {
            for v in row.iter_mut() {
                *v = opt()?;
            }
        }
        let equip = (entries, motes, r.u8()? != 0);
        let pet = match r.u8()? {
            0 => None,
            _ => Some((r.i32()?, r.i32()?, r.i32()?)),
        };
        let n = r.u8()? as usize;
        let blocks = (0..n).map(|_| Some((r.u16()?, r.v3()?))).collect::<Option<Vec<_>>>()?;
        let colours = [r.u8()?, r.u8()?, r.u8()?, r.u8()?];
        Some(Self { tick, generation, pack, heroes, more, eye, focus, characters, flags, items, neopoints, talk, cut, events, shots, show, story, has_story, panels, p2_prompt, world_rev, world, moving, anim_times, page, seat, rtt, cmd_ack, equip, pet, blocks, colours })
    }
}

/// The joiner's controller for one tick: the held buttons and stick, the right stick, and press
/// counters (so a press is not lost with a packet).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub generation: u32,
    /// The last host event played here, in order (the event channel's acknowledgement), and the
    /// story revision held here.
    pub ack: u32,
    pub story: u32,
    pub world: u32,
    pub side: f32,
    pub forward: f32,
    pub held: u16,
    pub right: [u8; 2],
    pub jumps: u8,
    pub attacks: u8,
    pub squares: u8,
    /// Press counters of the D-pad (up, left, right, down) and Triangle, for player 2's panels.
    pub dpad: [u8; 5],
    /// The last host state tick received here (the host times the round trip by it).
    pub seen: u32,
    /// The item-screen command waiting for the host (number, kind, two values; `ItemCmd`),
    /// sent with every input until a state acknowledges its number.
    pub cmd: (u8, u8, u16, u16),
}

/// A joiner's item-screen command as it travels (kind, two values).
pub fn cmd_of(c: &crate::menus::Command) -> Option<(u8, u16, u16)> {
    use crate::menus::Command;
    Some(match *c {
        Command::UseItem(id) => (1, id, 0),
        Command::Marker(m) => (2, m as i16 as u16, 0),
        Command::EquipMote { word, id } => (3, word as u16, id),
        Command::MoteSelect(s) => (4, s as u16, 0),
        Command::ClearMote(word) => (5, word as u16, 0),
        Command::Cloak => (6, 0, 0),
        _ => return None,
    })
}

/// The item-screen command a joiner sent (`cmd_of` back).
pub fn command_of(kind: u8, a: u16, b: u16) -> Option<crate::menus::Command> {
    use crate::menus::Command;
    Some(match kind {
        1 => Command::UseItem(a),
        2 => Command::Marker(a as i16 as i32),
        3 => Command::EquipMote { word: a as usize, id: b },
        4 => Command::MoteSelect(a as u8),
        5 => Command::ClearMote(a as usize),
        6 => Command::Cloak,
        _ => return None,
    })
}

pub(super) const RUN: u16 = 1;
pub(super) const SQUARE_HELD: u16 = 2;
pub(super) const CROSS_HELD: u16 = 4;
pub(super) const GUARD: u16 = 8;
pub(super) const ATTACK_HELD: u16 = 16;
pub(super) const R2: u16 = 32;
pub(super) const CIRCLE: u16 = 64;
pub(super) const L2: u16 = 128;

impl Input {
    pub fn from_hero(i: &HeroInput, pad: &CameraPad, generation: u32, counts: (u8, u8, u8), ack: (u32, u32)) -> Self {
        let bit = |b: bool, v: u16| if b { v } else { 0 };
        Self {
            generation,
            ack: ack.0,
            story: ack.1,
            world: 0,
            side: i.side,
            forward: i.forward,
            held: bit(i.run, RUN) | bit(i.square_held, SQUARE_HELD) | bit(i.cross_held, CROSS_HELD) | bit(i.guard, GUARD) | bit(i.attack_held, ATTACK_HELD) | bit(i.r2_held, R2) | bit(i.circle_held, CIRCLE) | bit(i.l2, L2),
            right: [pad.right_x, pad.right_y],
            jumps: counts.0,
            attacks: counts.1,
            squares: counts.2,
            dpad: [0; 5],
            seen: 0,
            cmd: (0, 0, 0, 0),
        }
    }

    /// The D-pad and Triangle presses since `last` (the counters last seen).
    pub fn to_panel(&self, last: &Input) -> neopets_sim::quick::PanelInput {
        let p = |k: usize| self.dpad[k] != last.dpad[k];
        neopets_sim::quick::PanelInput { up: p(0), left: p(1), right: p(2), down: p(3), triangle: p(4), ..Default::default() }
    }

    /// The hero input and camera pad, with the presses since `last` (the counters last seen).
    pub fn to_hero(&self, last: &Input) -> (HeroInput, CameraPad) {
        let h = |v: u16| self.held & v != 0;
        let i = HeroInput {
            side: self.side,
            forward: self.forward,
            run: h(RUN),
            jump: self.jumps != last.jumps,
            attack_held: h(ATTACK_HELD),
            attack: self.attacks != last.attacks,
            square: self.squares != last.squares,
            square_held: h(SQUARE_HELD),
            guard: h(GUARD),
            r2_held: h(R2),
            circle_held: h(CIRCLE),
            cross_held: h(CROSS_HELD),
            l2: h(L2),
        };
        (i, CameraPad { right_x: self.right[0], right_y: self.right[1], r3_held: false, r3_pressed: false })
    }

    pub(super) fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new(INPUT);
        w.u32(self.generation);
        w.u32(self.ack);
        w.u32(self.story);
        w.u32(self.world);
        w.f32(self.side);
        w.f32(self.forward);
        w.u16(self.held);
        w.u8(self.right[0]);
        w.u8(self.right[1]);
        w.u8(self.jumps);
        w.u8(self.attacks);
        w.u8(self.squares);
        w.0.extend_from_slice(&self.dpad);
        w.u32(self.seen);
        w.u8(self.cmd.0);
        w.u8(self.cmd.1);
        w.u16(self.cmd.2);
        w.u16(self.cmd.3);
        w.0
    }

    pub(super) fn decode(r: &mut Reader) -> Option<Self> {
        Some(Self { generation: r.u32()?, ack: r.u32()?, story: r.u32()?, world: r.u32()?, side: r.f32()?, forward: r.f32()?, held: r.u16()?, right: [r.u8()?, r.u8()?], jumps: r.u8()?, attacks: r.u8()?, squares: r.u8()?, dpad: [r.u8()?, r.u8()?, r.u8()?, r.u8()?, r.u8()?], seen: r.u32()?, cmd: (r.u8()?, r.u8()?, r.u16()?, r.u16()?) })
    }
}

