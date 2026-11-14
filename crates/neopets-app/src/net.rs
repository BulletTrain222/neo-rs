//! Online co-op on the home network (docs/MULTIPLAYER.md step 4; this build's feature, not in
//! the original game). Peer-hosted, with no server: one player hosts (the game listens on UDP port
//! `PORT`), the other types the host's address and joins. Each player needs their own copy of the
//! disc: only controller input and game numbers cross the network, never game data.
//!
//! The host runs the game. The joiner's controller drives the host's player 2 (as a second
//! controller does in local co-op, coop.rs). The joiner loads the host's game from a save slot
//! the host sends (`Welcome`), then every tick takes the host's `State`: both heroes (place,
//! facing, action, clip, health and magic, player 2's camera), every character (place, facing,
//! clip, health), the story flags, items and Neopoints. The joiner's own game still runs (its
//! loading, scripts, animation and effects) but its heroes and characters are put where the
//! host has them after each of its ticks, and it takes no hits of its own. A world change on the
//! host sends a new `Welcome`.
//!
//! Messages are small binary records (`Writer`, `Reader`): a 4-byte magic, the protocol version,
//! a kind byte, then the kind's fields, little-endian. Not reliable in general: `Hello` is sent
//! until a `Welcome` or `Refuse` comes back, the `Welcome` until input arrives with its generation,
//! and the input and state are sent every tick, each newer one replacing the last.
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use neopets_sim::camera::CameraPad;
use neopets_sim::hero::HeroInput;

use crate::PlayState;

/// The UDP port the host listens on.
pub const PORT: u16 = 47817;
const MAGIC: [u8; 4] = *b"NPCO";
const PROTOCOL: u16 = 16;
/// No message for this long ends the session.
const TIMEOUT: Duration = Duration::from_secs(6);

const HELLO: u8 = 1;
const WELCOME: u8 = 2;
const REFUSE: u8 = 3;
const INPUT: u8 = 4;
const STATE: u8 = 5;
const BYE: u8 = 6;
/// Sent when nothing else has gone out for 200 ms (a paused game sends no states or inputs), with
/// whether this side is paused.
const PING: u8 = 7;
/// A welcome goes in parts of at most `PART` bytes (the internet splits larger datagrams into IP
/// fragments, which some routers drop); the joiner acknowledges the parts it holds.
const WELCOME_PART: u8 = 8;
const PART_ACK: u8 = 9;
const PART: usize = 1000;
/// The story (channel 0) and the world sync (channel 1) go in parts too, outside the states, each
/// resent every 100 ms until the joiner's input acknowledges its revision.
const BLOB_PART: u8 = 10;
/// A state stays under this many bytes (events fill the room left), so it never needs fragments.
const STATE_LIMIT: usize = 1150;
/// Events that did not fit in a state, in up to two more messages a tick.
const EVENTS: u8 = 11;

/// This build's identity, checked at joining: the crate version and the build's own number.
fn build_id() -> String {
    format!("{} {}", env!("CARGO_PKG_VERSION"), option_env!("NEOPETS_BUILD").unwrap_or("dev"))
}

#[derive(Default)]
struct Writer(Vec<u8>);

impl Writer {
    fn new(kind: u8) -> Self {
        let mut w = Self(MAGIC.to_vec());
        w.u16(PROTOCOL);
        w.u8(kind);
        w
    }
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn v3(&mut self, v: [f32; 3]) {
        for x in v {
            self.f32(x);
        }
    }
    fn bytes(&mut self, b: &[u8]) {
        self.u32(b.len() as u32);
        self.0.extend_from_slice(b);
    }
    fn text(&mut self, t: &str) {
        self.bytes(t.as_bytes());
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    /// The message kind, if the bytes start with the magic and this protocol.
    fn open(b: &'a [u8]) -> Option<(u8, Self)> {
        if b.len() < 7 || b[..4] != MAGIC || u16::from_le_bytes([b[4], b[5]]) != PROTOCOL {
            return None;
        }
        Some((b[6], Self { b, at: 7 }))
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.at..self.at + n)?;
        self.at += n;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn v3(&mut self) -> Option<[f32; 3]> {
        Some([self.f32()?, self.f32()?, self.f32()?])
    }
    fn bytes(&mut self) -> Option<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    fn text(&mut self) -> Option<String> {
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
    /// What the host's screen and music show that the joiner shows too.
    pub show: Show,
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
    fn write(&self, w: &mut Writer) {
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
    }

    fn read(r: &mut Reader) -> Option<Self> {
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
        Some(Self { pack, breakables, items, chars })
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
    fn write(&self, w: &mut Writer) {
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

    fn read(r: &mut Reader) -> Option<Self> {
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

fn write_frame(w: &mut Writer, f: &[[f32; 3]; 3]) {
    for row in f {
        w.v3(*row);
    }
}

fn read_frame(r: &mut Reader) -> Option<[[f32; 3]; 3]> {
    Some([r.v3()?, r.v3()?, r.v3()?])
}

fn write_opt_f32(w: &mut Writer, v: Option<f32>) {
    match v {
        Some(x) => {
            w.u8(1);
            w.f32(x);
        }
        None => w.u8(0),
    }
}

fn read_opt_f32(r: &mut Reader) -> Option<Option<f32>> {
    Some(match r.u8()? {
        1 => Some(r.f32()?),
        _ => None,
    })
}

fn write_op(w: &mut Writer, op: &neopets_sim::effect::Op) {
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

fn read_op(kind: u8, r: &mut Reader) -> Option<neopets_sim::effect::Op> {
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
fn write_shot(w: &mut Writer, i: u8, sh: &neopets_sim::projectile::Shot) {
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

fn read_shot(r: &mut Reader) -> Option<(u8, neopets_sim::projectile::Shot)> {
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

fn write_pan(w: &mut Writer, pan: Option<i32>) {
    match pan {
        Some(p) => {
            w.u8(1);
            w.i32(p);
        }
        None => w.u8(0),
    }
}

fn read_pan(r: &mut Reader) -> Option<Option<i32>> {
    Some(match r.u8()? {
        1 => Some(r.i32()?),
        _ => None,
    })
}

impl Event {
    fn write(&self, w: &mut Writer) {
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

    fn read(r: &mut Reader) -> Option<Self> {
        use neopets_sim::sound::Request;
        Some(match r.u8()? {
            1 => Event::Sound(Request::Play { id: r.u32()? as usize, block: r.u32()?, cue: r.i32()?, volume: r.i32()?, pan: read_pan(r)? }),
            2 => Event::Sound(Request::Set { id: r.u32()? as usize, volume: r.i32()?, pan: read_pan(r)? }),
            3 => Event::Sound(Request::Stop { id: r.u32()? as usize }),
            k @ 10..=16 => Event::Fx(read_op(k, r)?),
            20 => Event::Stinger(r.i32()?),
            21 => Event::Emote(r.u32()?, r.u32()?, r.u32()?),
            22 => Event::Movie(r.u16()?),
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

fn write_hero(w: &mut Writer, h: &HeroState) {
    w.u8(h.who);
    w.v3(h.position);
    w.v3(h.facing);
    w.u8(h.state);
    w.u32(h.clip);
    w.i32(h.hp);
    w.i32(h.hp_max);
    w.i32(h.mp);
    w.i32(h.mp_max);
    w.u8(h.flash);
    w.u8(h.down as u8);
}

fn read_hero(r: &mut Reader) -> Option<HeroState> {
    Some(HeroState { who: r.u8()?, position: r.v3()?, facing: r.v3()?, state: r.u8()?, clip: r.u32()?, hp: r.i32()?, hp_max: r.i32()?, mp: r.i32()?, mp_max: r.i32()?, flash: r.u8()?, down: r.u8()? != 0 })
}

impl State {
    fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new(STATE);
        w.u64(self.tick);
        w.u32(self.generation);
        w.u32(self.pack);
        for h in &self.heroes {
            write_hero(&mut w, h);
        }
        w.v3(self.eye);
        w.v3(self.focus);
        w.u32(self.characters.len() as u32);
        for c in &self.characters {
            w.u32(c.pack);
            w.u8(c.slot);
            w.u32(c.ordinal);
            w.v3(c.position);
            w.v3(c.facing);
            w.u32(c.clip);
            w.i32(c.hp);
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
        w.0
    }

    fn decode(r: &mut Reader) -> Option<Self> {
        let tick = r.u64()?;
        let generation = r.u32()?;
        let pack = r.u32()?;
        let heroes = [read_hero(r)?, read_hero(r)?];
        let (eye, focus) = (r.v3()?, r.v3()?);
        let n = r.u32()? as usize;
        let mut characters = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            let (pack, slot, ordinal, position, facing, clip, hp, bits) = (r.u32()?, r.u8()?, r.u32()?, r.v3()?, r.v3()?, r.u32()?, r.i32()?, r.u8()?);
            characters.push(CharState { pack, slot, ordinal, position, facing, clip, hp, dead: bits & 1 != 0, inactive: bits & 2 != 0 });
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
        Some(Self { tick, generation, pack, heroes, eye, focus, characters, flags, items, neopoints, talk, cut, events, shots, show, story, has_story, panels, p2_prompt, world_rev, world, moving })
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
}

const RUN: u16 = 1;
const SQUARE_HELD: u16 = 2;
const CROSS_HELD: u16 = 4;
const GUARD: u16 = 8;
const ATTACK_HELD: u16 = 16;
const R2: u16 = 32;
const CIRCLE: u16 = 64;
const L2: u16 = 128;

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

    fn encode(&self) -> Vec<u8> {
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
        w.0
    }

    fn decode(r: &mut Reader) -> Option<Self> {
        Some(Self { generation: r.u32()?, ack: r.u32()?, story: r.u32()?, world: r.u32()?, side: r.f32()?, forward: r.f32()?, held: r.u16()?, right: [r.u8()?, r.u8()?], jumps: r.u8()?, attacks: r.u8()?, squares: r.u8()?, dpad: [r.u8()?, r.u8()?, r.u8()?, r.u8()?, r.u8()?] })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    #[default]
    Off,
    Host,
    Join,
}

/// The session: this side's role, its socket, the peer, and what each side has to hand on.
#[derive(Resource, Default)]
pub struct Net {
    pub role: Role,
    socket: Option<UdpSocket>,
    pub peer: Option<SocketAddr>,
    /// What the Multiplayer page shows.
    pub status: String,
    /// Asked for by the Multiplayer page: host, join this address, or leave.
    pub request: Option<Request>,
    last_heard: Option<Instant>,
    /// When the last hello or welcome went out (each is resent until answered; the states and
    /// inputs sent every tick do not count).
    last_sent: Option<Instant>,
    /// The disc's identity (a hash of the verified boot program), worked out once.
    disc: Option<u64>,
    /// Host: the joiner's latest input and the one before its presses were counted from; the
    /// world the last welcome was for and its generation; a welcome waiting to be resent.
    pub remote: Option<Input>,
    seen: Input,
    pub welcome_pack: u32,
    /// The host's game (`Play::generation`, new at each load or new game) at the last welcome.
    pub welcome_game: u32,
    pub generation: u32,
    welcome: Option<Vec<u8>>,
    /// The parts of the welcome the joiner has acknowledged (bit k for part k).
    welcome_mask: u64,
    /// Joiner: the parts of a welcome being received (its generation and each part).
    parts: Option<(u32, Vec<Option<Vec<u8>>>)>,
    /// Joiner: the host's address, the slot to load, the latest state, the generation loaded,
    /// and the press counters of this side's controller.
    target: Option<SocketAddr>,
    pub load: Option<neopets_sim::save::Slot>,
    /// The welcome's running effects, applied once its game is loaded.
    pub load_fx: Vec<neopets_sim::effect::Op>,
    pub state: Option<State>,
    pub loaded: u32,
    pub counts: (u8, u8, u8),
    pub dpad: [u8; 5],
    pub outgoing: Option<Input>,
    /// Host: events waiting for the joiner's acknowledgement, the next event's number and the
    /// last number acknowledged. Joiner: the last event played, and the events received but not
    /// yet played (step_sim plays them on the next tick).
    events: std::collections::VecDeque<(u32, Event)>,
    next_event: u32,
    acked: u32,
    pub applied: u32,
    pub incoming: Vec<Event>,
    /// Joiner: the sources of the host's voices playing here (id: emitter, base, radii).
    pub voices: std::collections::HashMap<usize, ([f32; 3], i32, f32, f32)>,
    /// A line for the player (joined, left, lost), shown once in play; whether a joiner's game
    /// ended (it goes back to the title screen); whether the other side is paused; when anything
    /// last went out.
    pub notice: Option<String>,
    pub ended: bool,
    /// Bytes sent and received so far (for the bandwidth trace).
    pub bytes_out: u64,
    pub bytes_in: u64,
    /// Received messages held for NEOPETS_NET_DELAY, and the NEOPETS_NET_DROP generator.
    held: std::collections::VecDeque<(Instant, Vec<u8>, SocketAddr)>,
    lossy: u64,
    /// Host: the story revision, the story it stands for, and the revision the joiner holds.
    /// Joiner: the latest story received (revision, flags, items, Neopoints), put into each
    /// state that comes without one.
    story_rev: u32,
    story_last: Option<(Vec<u16>, Vec<(u16, u32)>, i32)>,
    story_acked: u32,
    pub story_held: Option<(u32, Vec<u16>, Vec<(u16, u32)>, i32)>,
    /// The world sync's revisions, as the story's: the host's count, what it stands for and the
    /// joiner's acknowledgement; the joiner's latest copy.
    world_rev: u32,
    #[allow(clippy::type_complexity)]
    world_last: Option<(u32, Vec<u8>, Vec<(u32, u16, u16, u8)>, Vec<(u32, u8, u32)>)>,
    /// Host: each character's state last sent, and the round-robin cursors of the characters and
    /// moving items resent unchanged (so a lost state heals).
    chars_last: std::collections::HashMap<(u32, u8, u32), CharState>,
    cursor: (usize, usize),
    /// Joiner: the running copies of the host's characters and moving items, and the welcome
    /// generation they are for.
    chars_acc: std::collections::HashMap<(u32, u8, u32), CharState>,
    moving_acc: std::collections::HashMap<(u32, u16), (u32, u16, [f32; 3], f32)>,
    acc_gen: u32,
    world_acked: u32,
    pub world_held: Option<(u32, WorldSync)>,
    /// Host: each channel's revision last sent and when. Joiner: each channel's parts arriving.
    blob_sent: [(u32, Option<Instant>); 2],
    #[allow(clippy::type_complexity)]
    blob_in: [Option<(u32, Vec<Option<Vec<u8>>>)>; 2],
    /// Host: the moving items' places last sent (a place goes again only when it changes).
    moving_last: std::collections::HashMap<(u32, u16), [f32; 3]>,
    pub peer_paused: bool,
    /// The other side's page: 0 in play, 1 a menu (paused), 2 game over.
    pub peer_page: u8,
    last_any: Option<Instant>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    Host,
    Join(String),
    Leave,
}

impl Net {
    /// Whether this side follows a host (a joiner with a loaded game).
    pub fn following(&self) -> bool {
        self.role == Role::Join && self.loaded > 0
    }

    fn send(&mut self, bytes: &[u8]) {
        if let (Some(s), Some(p)) = (self.socket.as_ref(), self.peer.or(self.target)) {
            let _ = s.send_to(bytes, p);
            self.bytes_out += bytes.len() as u64 + 28;
            self.last_any = Some(Instant::now());
        }
    }

    fn stop(&mut self, status: &str) {
        if self.socket.is_some() && self.peer.is_some() {
            let w = Writer::new(BYE);
            self.send(&w.0);
        }
        *self = Self { disc: self.disc, status: status.to_string(), notice: self.notice.take(), ended: self.ended, ..Default::default() };
    }

    /// Host: the state after a tick, sent to the joiner with the events not yet acknowledged
    /// (at most 96, oldest first).
    pub fn send_state(&mut self, s: &State) {
        if self.role == Role::Host && self.peer.is_some() {
            let acked = self.acked;
            while self.events.front().is_some_and(|e| e.0 <= acked) {
                self.events.pop_front();
            }
            let mut s = s.clone();
            let story = (s.flags.clone(), s.items.clone(), s.neopoints);
            if self.story_last.as_ref() != Some(&story) {
                self.story_rev += 1;
                self.story_last = Some(story);
            }
            s.story = self.story_rev;
            s.has_story = self.story_acked < self.story_rev;
            // The world's breakables and the set of live items (not their bouncing places).
            let sig = s.world.as_ref().map(|w| (w.pack, w.breakables.clone(), w.items.iter().map(|(p, k, it)| (*p, *k, it.id, it.state)).collect::<Vec<_>>(), w.chars.clone()));
            if sig != self.world_last {
                self.world_rev += 1;
                if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() {
                    println!("NETREV host {} items {:?} broken {:?}", self.world_rev, s.world.as_ref().map(|w| w.items.len()), s.world.as_ref().map(|w| w.breakables.iter().filter(|&&b| b == 2).count()));
                }
                self.world_last = sig;
            }
            s.world_rev = self.world_rev;
            if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() && s.tick % 300 == 0 {
                println!("NETW host tick {}: world rev {} acknowledged {} items {:?}", s.tick, self.world_rev, self.world_acked, s.world.as_ref().map(|w| w.items.len()));
            }
            // The story and the world sync go as parts on their own channels (below).
            let story_bytes = {
                let mut w = Writer(Vec::new());
                w.u32(s.flags.len() as u32);
                for &f in &s.flags {
                    w.u16(f);
                }
                w.u32(s.items.len() as u32);
                for &(id, n) in &s.items {
                    w.u16(id);
                    w.u32(n);
                }
                w.i32(s.neopoints);
                w.0
            };
            let world_bytes = s.world.take().map(|ws| {
                let mut w = Writer(Vec::new());
                ws.write(&mut w);
                w.0
            });
            s.has_story = false;
            (s.flags, s.items) = (Vec::new(), Vec::new());
            if self.story_acked < self.story_rev {
                self.blob_send(0, self.story_rev, &story_bytes);
            }
            if let Some(b) = world_bytes.filter(|_| self.world_acked < self.world_rev) {
                self.blob_send(1, self.world_rev, &b);
            }
            // Moving items and characters: each goes when it changed, and four unchanged ones a
            // tick in turn (a lost state heals within a few ticks).
            {
                let n = s.moving.len().max(1);
                let start = self.cursor.1 % n;
                let last = &mut self.moving_last;
                let keep: Vec<bool> = s
                    .moving
                    .iter()
                    .enumerate()
                    .map(|(i, &(pack, k, at, _))| {
                        let changed = last.get(&(pack, k)).is_none_or(|p| (0..3).any(|j| (p[j] - at[j]).abs() > 1e-3));
                        if changed {
                            last.insert((pack, k), at);
                        }
                        changed || (i + n - start) % n < 4
                    })
                    .collect();
                let mut it = keep.iter();
                s.moving.retain(|_| *it.next().unwrap_or(&false));
                self.cursor.1 = start + 4;
                if s.tick % 120 == 0 {
                    last.clear();
                }
            }
            {
                let n = s.characters.len().max(1);
                let start = self.cursor.0 % n;
                let last = &mut self.chars_last;
                let keep: Vec<bool> = s
                    .characters
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        let key = (c.pack, c.slot, c.ordinal);
                        let changed = last.get(&key).is_none_or(|p| {
                            p.clip != c.clip || p.hp != c.hp || p.dead != c.dead || p.inactive != c.inactive || (0..3).any(|j| (p.position[j] - c.position[j]).abs() > 1e-3 || (p.facing[j] - c.facing[j]).abs() > 1e-3)
                        });
                        if changed {
                            last.insert(key, *c);
                        }
                        changed || (i + n - start) % n < 4
                    })
                    .collect();
                let mut it = keep.iter();
                s.characters.retain(|_| *it.next().unwrap_or(&false));
                self.cursor.0 = start + 4;
                if s.tick % 120 == 0 {
                    last.clear();
                }
            }
            // The events not yet acknowledged, oldest first, up to 96 and in the room the state
            // leaves under the limit (at least one; the rest go with the next states).
            let room = STATE_LIMIT.saturating_sub(s.encode().len()).max(64);
            let mut size = 0;
            s.events = Vec::new();
            for e in self.events.iter().take(96) {
                let mut w = Writer(Vec::new());
                e.1.write(&mut w);
                size += w.0.len() + 4;
                if size > room && !s.events.is_empty() {
                    break;
                }
                s.events.push(e.clone());
            }
            let bytes = s.encode();
            // Events that did not fit: up to two more messages, every other tick (they go again
            // until acknowledged).
            let mut rest = self.events.iter().skip(s.events.len()).peekable();
            let mut extra = Vec::new();
            for _ in 0..if s.tick % 2 == 0 { 2 } else { 0 } {
                if rest.peek().is_none() {
                    break;
                }
                let mut m = Writer::new(EVENTS);
                let mut list = Vec::new();
                let mut size = 9;
                while let Some(e) = rest.peek() {
                    let mut w = Writer(Vec::new());
                    e.1.write(&mut w);
                    if size + w.0.len() + 4 > STATE_LIMIT && !list.is_empty() {
                        break;
                    }
                    size += w.0.len() + 4;
                    list.push((e.0, w.0));
                    rest.next();
                }
                m.u16(list.len() as u16);
                for (n, b) in list {
                    m.u32(n);
                    m.0.extend_from_slice(&b);
                }
                extra.push(m.0);
            }
            for m in extra {
                self.send(&m);
            }
            if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() && s.tick % 300 == 0 {
                println!("NETEV host tick {}: {} events made, {} acknowledged, {} waiting; state {} bytes ({} characters, {} flags, {} items, {} events, {} moving, {} shots, talk {}); sent {} received {} bytes", s.tick, self.next_event, self.acked, self.events.len(), bytes.len(), s.characters.len(), s.flags.len(), s.items.len(), s.events.len(), s.moving.len(), s.shots.len(), s.talk.is_some(), self.bytes_out, self.bytes_in);
            }
            self.send(&bytes);
        }
    }

    /// Host: a blob channel's revision in parts, at once for a new revision and every 100 ms while
    /// it is not acknowledged.
    fn blob_send(&mut self, channel: usize, rev: u32, bytes: &[u8]) {
        let now = Instant::now();
        let (sent_rev, at) = self.blob_sent[channel];
        if sent_rev == rev && at.is_some_and(|t| now.duration_since(t) < Duration::from_millis(100)) {
            return;
        }
        self.blob_sent[channel] = (rev, Some(now));
        let chunks: Vec<&[u8]> = if bytes.is_empty() { vec![&[][..]] } else { bytes.chunks(PART).collect() };
        for (k, c) in chunks.iter().enumerate() {
            let mut m = Writer::new(BLOB_PART);
            m.u8(channel as u8);
            m.u32(rev);
            m.u16(chunks.len() as u16);
            m.u16(k as u16);
            m.bytes(c);
            let b = m.0;
            self.send(&b);
        }
    }

    /// Host: an event for the joiner (kept while a joiner is in; at most 4,000 waiting, the
    /// oldest dropped).
    pub fn push_event(&mut self, e: Event) {
        if self.role != Role::Host || self.peer.is_none() {
            return;
        }
        self.next_event += 1;
        let n = self.next_event;
        self.events.push_back((n, e));
        if self.events.len() > 4000 {
            self.events.pop_front();
        }
    }

    /// Joiner: this tick's input, sent to the host.
    pub fn send_input(&mut self, i: Input) {
        if self.role == Role::Join && self.target.is_some() {
            let bytes = i.encode();
            self.send(&bytes);
        }
    }

    /// Host: a new welcome (the joiner reloads) when the host's world changed.
    pub fn rewelcome(&mut self, slot: &neopets_sim::save::Slot, pack: u32, game: u32, fx: &[neopets_sim::effect::Op]) {
        if self.role != Role::Host || self.peer.is_none() || (pack == self.welcome_pack && game == self.welcome_game) {
            return;
        }
        self.welcome_pack = pack;
        self.welcome_game = game;
        self.generation += 1;
        self.welcome = Some(welcome(self.generation, slot, fx));
        self.welcome_mask = 0;
        self.last_sent = None;
        info!("net: world {pack}, welcome {}", self.generation);
    }
}

/// A welcome: the generation, the host's game as a save slot, and its gameplay effects running
/// now (effect operations, applied after the load).
fn welcome(generation: u32, slot: &neopets_sim::save::Slot, fx: &[neopets_sim::effect::Op]) -> Vec<u8> {
    let mut w = Writer::new(WELCOME);
    w.u32(generation);
    w.bytes(&slot.0);
    w.u16(fx.len().min(512) as u16);
    for op in fx.iter().take(512) {
        write_op(&mut w, op);
    }
    w.0
}

/// Joiner: a whole welcome (the host's game as a save slot and its running effects), loaded
/// by step_sim on this frame.
fn take_welcome(net: &mut Net, r: &mut Reader, from: SocketAddr) {
    let (Some(generation), Some(bytes)) = (r.u32(), r.bytes()) else { return };
    if generation == net.loaded || bytes.len() != neopets_sim::save::SLOT_SIZE {
        return;
    }
    let first = net.loaded == 0;
    net.load = Some(neopets_sim::save::Slot(bytes.to_vec()));
    net.load_fx = match r.u16() {
        Some(n) => (0..n).map_while(|_| read_op(r.u8()?, r)).collect(),
        None => Vec::new(),
    };
    net.loaded = generation;
    net.state = None;
    net.status = format!("Joined {from}.");
    if first {
        net.notice = Some("Joined the host's game.".into());
    }
    info!("net: welcome {generation} from {from}");
}

/// The local addresses of this computer (for the Host line), from the route a UDP socket would
/// take; none when there is no network.
pub fn local_address() -> Option<std::net::IpAddr> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?;
    s.local_addr().ok().map(|a| a.ip())
}

/// Each frame: requests from the Multiplayer page, the socket's messages, joining, timeouts.
#[allow(clippy::too_many_arguments)]
pub fn net_step(mut net: ResMut<Net>, mut play: Option<ResMut<PlayState>>, mut archive: Option<ResMut<crate::Archive>>, mut coop: ResMut<crate::coop::Coop>, menu: Option<Res<crate::menu_view::Menu>>) {
    if net.disc.is_none() {
        if let Some(a) = archive.as_deref_mut() {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            match neopets_data::archive::verified_boot_program(&mut a.disc) {
                Ok(b) => b.hash(&mut h),
                Err(_) => 0u8.hash(&mut h),
            }
            net.disc = Some(h.finish());
        }
    }
    // NEOPETS_NET_HOST=1 / NEOPETS_NET_JOIN=address (diagnostic): host once play has run a
    // second; join then too, or five seconds after start (from the title screen).
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    // NEOPETS_NET_HOST=title hosts five seconds after start too, before any game.
    let early = START.get_or_init(Instant::now).elapsed() > Duration::from_secs(5);
    let started = early && std::env::var("NEOPETS_NET_HOST").map_or(true, |v| v == "title");
    if net.role == Role::Off && net.request.is_none() && (started || play.as_ref().is_some_and(|p| p.play.ticks > 60)) {
        static ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !ONCE.swap(true, std::sync::atomic::Ordering::Relaxed) {
            if std::env::var_os("NEOPETS_NET_HOST").is_some() {
                net.request = Some(Request::Host);
            } else if let Ok(a) = std::env::var("NEOPETS_NET_JOIN") {
                net.request = Some(Request::Join(a));
            }
        }
    }
    match net.request.take() {
        Some(Request::Host) => {
            net.stop("");
            match UdpSocket::bind(("0.0.0.0", PORT)).and_then(|s| s.set_nonblocking(true).map(|_| s)) {
                Ok(s) => {
                    net.socket = Some(s);
                    net.role = Role::Host;
                    net.status = match local_address() {
                        Some(ip) => format!("Hosting. Your friend joins with {ip}."),
                        None => format!("Hosting on port {PORT}."),
                    };
                    info!("net: hosting on port {PORT}");
                }
                Err(e) => net.status = format!("Could not host: {e}"),
            }
        }
        Some(Request::Join(text)) => {
            net.stop("");
            let addr = if text.contains(':') { text.parse::<SocketAddr>().ok() } else { format!("{text}:{PORT}").parse::<SocketAddr>().ok() };
            match (addr, UdpSocket::bind("0.0.0.0:0").and_then(|s| s.set_nonblocking(true).map(|_| s))) {
                (Some(a), Ok(s)) => {
                    net.socket = Some(s);
                    net.role = Role::Join;
                    net.target = Some(a);
                    net.status = format!("Joining {a}...");
                    info!("net: joining {a}");
                }
                (None, _) => net.status = format!("\"{text}\" is not an address."),
                (_, Err(e)) => net.status = format!("Could not join: {e}"),
            }
        }
        Some(Request::Leave) => {
            if net.role == Role::Join {
                if let Some(p) = play.as_deref_mut() {
                    p.play.remove_player();
                    p.play.net_follow = false;
                }
            }
            if net.role == Role::Host {
                if let Some(p) = play.as_deref_mut() {
                    p.play.remove_player();
                }
            }
            net.stop("Left.");
        }
        None => {}
    }
    if net.role == Role::Off {
        return;
    }
    let now = Instant::now();
    // The messages waiting. NEOPETS_NET_DROP=percent and NEOPETS_NET_DELAY=ms (diagnostic): a
    // lossy, slow network, made here by dropping that share of received messages and holding
    // the rest that long.
    let mut buf = vec![0u8; 65536];
    let drop = std::env::var("NEOPETS_NET_DROP").ok().and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or(0);
    let delay = Duration::from_millis(std::env::var("NEOPETS_NET_DELAY").ok().and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0));
    while let Some(s) = net.socket.as_ref() {
        let Ok((n, from)) = s.recv_from(&mut buf) else { break };
        net.bytes_in += n as u64 + 28;
        if drop > 0 {
            net.lossy = net.lossy.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            if ((net.lossy >> 33) % 100) < drop as u64 {
                continue;
            }
        }
        net.held.push_back((now + delay, buf[..n].to_vec(), from));
    }
    let mut ready = Vec::new();
    while net.held.front().is_some_and(|m| m.0 <= now) {
        if let Some((_, b, from)) = net.held.pop_front() {
            ready.push((b, from));
        }
    }
    for (bytes, from) in ready {
        let Some((kind, mut r)) = Reader::open(&bytes) else { continue };
        match (net.role, kind) {
            (Role::Host, HELLO) => {
                let (build, disc) = (r.text(), r.u64());
                let refuse = if build.as_deref() != Some(build_id().as_str()) {
                    Some(format!("different game builds ({} here, {} there)", build_id(), build.unwrap_or_default()))
                } else if disc != net.disc {
                    Some("different discs".to_string())
                } else if net.peer.is_some_and(|p| p != from) {
                    Some("a player has already joined".to_string())
                } else if play.as_ref().is_none_or(|p| !p.play.spawned) {
                    Some("the host is not in a game yet".to_string())
                } else {
                    None
                };
                if let Some(why) = refuse {
                    let mut w = Writer::new(REFUSE);
                    w.text(&why);
                    if let Some(s) = net.socket.as_ref() {
                        let _ = s.send_to(&w.0, from);
                    }
                    continue;
                }
                let Some(p) = play.as_deref_mut() else { continue };
                if net.peer.is_none() {
                    net.peer = Some(from);
                    net.last_heard = Some(now);
                    // Player 2 is the joiner's (a local second controller lets go of it).
                    if p.play.others.is_empty() {
                        p.play.add_player();
                    }
                    coop.pad = None;
                    net.generation += 1;
                    net.welcome_pack = p.play.world.pack;
                    net.welcome_game = p.play.generation;
                    let mut slot = neopets_sim::save::Slot::empty();
                    p.play.write_slot(&mut slot, b"NETPLAY");
                    net.welcome = Some(welcome(net.generation, &slot, &p.play.effects.snapshot()));
                    net.welcome_mask = 0;
                    net.status = format!("{from} joined.");
                    net.notice = Some("Player 2 joined.".into());
                    info!("net: {from} joined");
                }
            }
            (Role::Host, INPUT) if net.peer == Some(from) => {
                net.last_heard = Some(now);
                if let Some(i) = Input::decode(&mut r) {
                    if i.generation == net.generation {
                        net.welcome = None;
                    }
                    net.acked = net.acked.max(i.ack);
                    net.story_acked = net.story_acked.max(i.story);
                    net.world_acked = net.world_acked.max(i.world);
                    net.remote = Some(i);
                }
            }
            (_, BYE) if net.peer == Some(from) || net.target == Some(from) => {
                let host = net.role == Role::Host;
                if let Some(p) = play.as_deref_mut() {
                    p.play.remove_player();
                    p.play.net_follow = false;
                }
                if host {
                    // The host keeps hosting for the next joiner.
                    net.peer = None;
                    net.remote = None;
                    net.welcome = None;
                    net.status = "Your friend left. Still hosting.".into();
                    net.notice = Some("Player 2 left.".into());
                    info!("net: the joiner left");
                } else {
                    if net.loaded > 0 {
                        net.ended = true;
                        net.notice = Some("The host ended the game.".into());
                    }
                    info!("net: the host left");
                    net.stop("The host ended the game.");
                }
            }
            (_, PING) if net.peer == Some(from) || net.target == Some(from) => {
                net.last_heard = Some(now);
                let page = r.u8().unwrap_or(0);
                let paused = page != 0;
                net.peer_paused = paused;
                net.peer_page = page;
                // The joiner paused: player 2 lets go of the controls (the presses counted so
                // far stay counted).
                if paused && net.role == Role::Host {
                    if let Some(i) = net.remote.as_mut() {
                        (i.side, i.forward, i.held, i.right) = (0.0, 0.0, 0, [128, 128]);
                    }
                }
            }
            (Role::Join, WELCOME) if net.target == Some(from) => {
                net.last_heard = Some(now);
                net.peer = Some(from);
                take_welcome(&mut net, &mut r, from);
            }
            (Role::Join, WELCOME_PART) if net.target == Some(from) => {
                net.last_heard = Some(now);
                net.peer = Some(from);
                let (Some(generation), Some(total), Some(k), Some(chunk)) = (r.u32(), r.u16(), r.u16(), r.bytes()) else { continue };
                let (total, k) = (total as usize, k as usize);
                if total == 0 || total > 64 || k >= total {
                    continue;
                }
                let mut mask = u64::MAX;
                if generation != net.loaded {
                    if net.parts.as_ref().is_none_or(|p| p.0 != generation || p.1.len() != total) {
                        net.parts = Some((generation, vec![None; total]));
                    }
                    if let Some((_, parts)) = net.parts.as_mut() {
                        parts[k] = Some(chunk.to_vec());
                        mask = parts.iter().enumerate().filter(|(_, p)| p.is_some()).fold(0, |m, (i, _)| m | 1 << i);
                        if parts.iter().all(Option::is_some) {
                            let whole: Vec<u8> = parts.iter().flatten().flatten().copied().collect();
                            net.parts = None;
                            if let Some((WELCOME, mut wr)) = Reader::open(&whole) {
                                take_welcome(&mut net, &mut wr, from);
                            }
                        }
                    }
                }
                let mut w = Writer::new(PART_ACK);
                w.u32(generation);
                w.u64(mask);
                let bytes = w.0;
                net.send(&bytes);
            }
            (Role::Join, EVENTS) if net.target == Some(from) => {
                net.last_heard = Some(now);
                let Some(count) = r.u16() else { continue };
                for _ in 0..count {
                    let (Some(n), Some(e)) = (r.u32(), Event::read(&mut r)) else { break };
                    if n == net.applied + 1 {
                        net.applied = n;
                        net.incoming.push(e);
                    }
                }
            }
            (Role::Join, BLOB_PART) if net.target == Some(from) => {
                net.last_heard = Some(now);
                let (Some(ch), Some(rev), Some(total), Some(k), Some(chunk)) = (r.u8(), r.u32(), r.u16(), r.u16(), r.bytes()) else { continue };
                let (ch, total, k) = (ch as usize, total as usize, k as usize);
                let held = match ch {
                    0 => net.story_held.as_ref().map(|h| h.0),
                    1 => net.world_held.as_ref().map(|h| h.0),
                    _ => continue,
                };
                if total == 0 || total > 64 || k >= total || held.is_some_and(|h| rev <= h) {
                    continue;
                }
                if net.blob_in[ch].as_ref().is_none_or(|b| b.0 != rev || b.1.len() != total) {
                    net.blob_in[ch] = Some((rev, vec![None; total]));
                }
                let Some((_, parts)) = net.blob_in[ch].as_mut() else { continue };
                parts[k] = Some(chunk.to_vec());
                if !parts.iter().all(Option::is_some) {
                    continue;
                }
                let whole: Vec<u8> = parts.iter().flatten().flatten().copied().collect();
                net.blob_in[ch] = None;
                let mut br = Reader { b: &whole, at: 0 };
                match ch {
                    0 => {
                        let story = (|| {
                            let n = br.u32()? as usize;
                            let flags = (0..n.min(65536)).map(|_| br.u16()).collect::<Option<Vec<_>>>()?;
                            let n = br.u32()? as usize;
                            let items = (0..n.min(65536)).map(|_| Some((br.u16()?, br.u32()?))).collect::<Option<Vec<_>>>()?;
                            Some((flags, items, br.i32()?))
                        })();
                        if let Some((f, i, np)) = story {
                            net.story_held = Some((rev, f, i, np));
                        }
                    }
                    _ => {
                        if let Some(ws) = WorldSync::read(&mut br) {
                            if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() {
                                println!("NETREV joiner {} items {} broken {}", rev, ws.items.len(), ws.breakables.iter().filter(|&&b| b == 2).count());
                            }
                            net.world_held = Some((rev, ws));
                        }
                    }
                }
            }
            (Role::Host, PART_ACK) if net.peer == Some(from) => {
                net.last_heard = Some(now);
                if let (Some(generation), Some(mask)) = (r.u32(), r.u64()) {
                    if generation == net.generation {
                        net.welcome_mask |= mask;
                    }
                }
            }
            (Role::Join, REFUSE) if net.target == Some(from) => {
                let why = r.text().unwrap_or_default();
                info!("net: refused: {why}");
                net.stop(&format!("The host said no: {why}."));
            }
            (Role::Join, STATE) if net.target == Some(from) => {
                net.last_heard = Some(now);
                if let Some(mut s) = State::decode(&mut r) {
                    // The host's events in order, each once (a gap waits for the resend).
                    for (n, e) in std::mem::take(&mut s.events) {
                        if n == net.applied + 1 {
                            net.applied = n;
                            net.incoming.push(e);
                        }
                    }
                    // The story comes only when it changed: keep it, and give it to states without.
                    if s.has_story {
                        if net.story_held.as_ref().is_none_or(|h| s.story >= h.0) {
                            net.story_held = Some((s.story, s.flags.clone(), s.items.clone(), s.neopoints));
                        }
                    } else if let Some((_, f, i, n)) = net.story_held.as_ref() {
                        (s.flags, s.items, s.neopoints) = (f.clone(), i.clone(), *n);
                    }
                    // The world sync, as the story: keep the latest, give it to states without.
                    match s.world.take() {
                        Some(ws) => {
                            if net.world_held.as_ref().is_none_or(|h| s.world_rev >= h.0) {
                                if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() && net.world_held.as_ref().is_none_or(|h| h.0 != s.world_rev) {
                                    println!("NETREV joiner {} items {} broken {}", s.world_rev, ws.items.len(), ws.breakables.iter().filter(|&&b| b == 2).count());
                                }
                                net.world_held = Some((s.world_rev, ws.clone()));
                            }
                            s.world = Some(ws);
                        }
                        None => s.world = net.world_held.as_ref().map(|h| h.1.clone()),
                    }
                    // Characters and moving items come only when changed: each state's go into
                    // running copies (cleared at a welcome), and the stored state carries them all.
                    if net.acc_gen != s.generation {
                        net.acc_gen = s.generation;
                        net.chars_acc.clear();
                        net.moving_acc.clear();
                    }
                    for c in &s.characters {
                        net.chars_acc.insert((c.pack, c.slot, c.ordinal), *c);
                    }
                    for m in &s.moving {
                        net.moving_acc.insert((m.0, m.1), *m);
                    }
                    s.characters = net.chars_acc.values().copied().collect();
                    s.moving = net.moving_acc.values().copied().collect();
                    // A newer world (a later welcome) restarts the host's ticks.
                    if net.state.as_ref().is_none_or(|old| (s.generation, s.tick) >= (old.generation, old.tick)) {
                        net.state = Some(s);
                    }
                }
            }
            _ => {}
        }
    }
    // Sending: the joiner's hello until welcomed, the host's welcome until the joiner has it.
    let due = net.last_sent.is_none_or(|t| now.duration_since(t) > Duration::from_millis(400));
    if net.role == Role::Join && net.loaded == 0 && due {
        let mut w = Writer::new(HELLO);
        w.text(&build_id());
        w.u64(net.disc.unwrap_or(0));
        let bytes = w.0;
        net.send(&bytes);
        net.last_sent = Some(now);
    }
    let part_due = net.last_sent.is_none_or(|t| now.duration_since(t) > Duration::from_millis(200));
    if let Some(w) = net.welcome.clone().filter(|_| part_due && net.role == Role::Host) {
        let chunks: Vec<&[u8]> = w.chunks(PART).collect();
        for (k, c) in chunks.iter().enumerate() {
            if net.welcome_mask & (1 << k) != 0 {
                continue;
            }
            let mut m = Writer::new(WELCOME_PART);
            m.u32(net.generation);
            m.u16(chunks.len() as u16);
            m.u16(k as u16);
            m.bytes(c);
            let bytes = m.0;
            net.send(&bytes);
        }
        net.last_sent = Some(now);
    }
    if net.role == Role::Join {
        if let Some(i) = net.outgoing.take() {
            net.send_input(i);
        }
    }
    // Nothing sent for 200 ms (a paused game): a ping, with whether this side is paused.
    let quiet = net.last_any.is_none_or(|t| now.duration_since(t) > Duration::from_millis(200));
    let linked = (net.role == Role::Host && net.peer.is_some()) || (net.role == Role::Join && net.loaded > 0);
    if quiet && linked {
        let page = match menu.as_ref().map(|m| m.flow.current) {
            Some(crate::menus::PLAY) | None => 0,
            Some(crate::menus::GAME_OVER) => 2,
            Some(_) => 1,
        };
        let mut w = Writer::new(PING);
        w.u8(page);
        let bytes = w.0;
        net.send(&bytes);
    }
    // The host's player 2 input from the joiner's controller (its presses since the last frame).
    if net.role == Role::Host {
        coop.remote = net.remote.map(|i| {
            let (h, c) = i.to_hero(&net.seen);
            (h, c)
        });
        if let Some(i) = net.remote {
            let pl = i.to_panel(&net.seen);
            let q = &mut coop.panel;
            (q.up, q.left, q.right, q.down, q.triangle) = (q.up | pl.up, q.left | pl.left, q.right | pl.right, q.down | pl.down, q.triangle | pl.triangle);
            net.seen = i;
        }
    }
    // Timeouts.
    if net.last_heard.is_some_and(|t| now.duration_since(t) > TIMEOUT) {
        if let Some(p) = play.as_deref_mut() {
            p.play.remove_player();
            p.play.net_follow = false;
        }
        if net.role == Role::Host {
            net.peer = None;
            net.remote = None;
            net.welcome = None;
            net.last_heard = None;
            net.status = "Lost your friend. Still hosting.".into();
            net.notice = Some("Lost player 2.".into());
            info!("net: the joiner timed out");
        } else {
            info!("net: the host timed out");
            if net.loaded > 0 {
                net.ended = true;
                net.notice = Some("Lost the host.".into());
            }
            net.stop("Lost the host.");
        }
    } else if net.role == Role::Join && net.loaded == 0 && net.last_heard.is_none() {
        // Still waiting for an answer.
    }
}

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
            }),
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
            p.down = h.down.then_some(0);
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
        self.state.flags = s.flags.iter().copied().collect();
        self.state.items = s.items.iter().copied().collect();
        self.state.neopoints = s.neopoints;
    }
}

/// The Multiplayer page's join address being typed (digits, dots and a colon; Backspace).
#[derive(Resource)]
pub struct JoinAddress(pub String);

impl Default for JoinAddress {
    fn default() -> Self {
        Self(std::fs::read_to_string("private/join.txt").map(|t| t.trim().to_string()).unwrap_or_else(|_| "127.0.0.1".into()))
    }
}

/// Each frame: with the Multiplayer page's address row focused, the number keys, the dot and the
/// colon type into the address, Backspace deletes.
pub fn type_address(menu: Option<Res<crate::menu_view::Menu>>, keys: Res<ButtonInput<KeyCode>>, mut address: ResMut<JoinAddress>) {
    let Some(menu) = menu else { return };
    if menu.flow.current != crate::menus::DISPLAY || menu.flow.display_page != 2 || menu.flow.display_focus != crate::display::ADDRESS_ROW {
        return;
    }
    const KEYS: [(KeyCode, char); 22] = [
        (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'), (KeyCode::Digit3, '3'), (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'), (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'), (KeyCode::Digit8, '8'), (KeyCode::Digit9, '9'),
        (KeyCode::Numpad0, '0'), (KeyCode::Numpad1, '1'), (KeyCode::Numpad2, '2'), (KeyCode::Numpad3, '3'), (KeyCode::Numpad4, '4'),
        (KeyCode::Numpad5, '5'), (KeyCode::Numpad6, '6'), (KeyCode::Numpad7, '7'), (KeyCode::Numpad8, '8'), (KeyCode::Numpad9, '9'),
        (KeyCode::Period, '.'), (KeyCode::NumpadDecimal, '.'),
    ];
    for (k, c) in KEYS {
        if keys.just_pressed(k) && address.0.len() < 40 {
            address.0.push(c);
        }
    }
    if keys.just_pressed(KeyCode::Semicolon) && keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) && address.0.len() < 40 {
        address.0.push(':');
    }
    if keys.just_pressed(KeyCode::Backspace) {
        address.0.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neopets_sim::sound::Request;

    #[test]
    fn state_with_events_round_trips() {
        let s = State {
            tick: 7,
            generation: 2,
            pack: 1,
            story: 3,
            has_story: true,
            panels: (true, [57, -1, 3, 9], 1, 57, false, 255),
            p2_prompt: Some(13),
            world_rev: 4,
            world: Some(WorldSync { pack: 909, breakables: vec![1, 2, 0], items: vec![(909, 3, NetItem { id: 160, position: [1.0, 2.0, 3.0], state: 1, angle: 0.5, scale: 1.0, direction: [0.0, 0.0, 1.0], price: -2, reference: 0.2 })], chars: vec![(909, 1, 0), (909, 2, 3)] }),
            moving: vec![(909, 4, [0.0, 1.0, 0.0], 0.25)],
            flags: vec![1, 500],
            items: vec![(57, 2)],
            neopoints: 40,
            events: vec![
                (5, Event::Sound(Request::Play { id: 3, block: 9, cue: -1, volume: 1000, pan: Some(-20) })),
                (6, Event::Sound(Request::Set { id: 3, volume: 10, pan: None })),
                (7, Event::Sound(Request::Stop { id: 3 })),
                (8, Event::Fx(neopets_sim::effect::Op::Add { handle: 4, def: Some(2), position: [1.0, 2.0, 3.0], frame: neopets_sim::effect::IDENTITY, state: 1, cull: None, scale: Some(0.5) })),
                (9, Event::Fx(neopets_sim::effect::Op::Update { handle: 4, listener: Some(([0.0; 3], [0.0, 0.0, 1.0])) })),
                (10, Event::Fx(neopets_sim::effect::Op::Release { handle: 4 })),
                (11, Event::Voice { id: 3, emitter: [1.0, 2.0, 3.0], base: 1024, r: 200.0, r0: 10.0 }),
                (12, Event::Rumble(200, 45)),
            ],
            shots: vec![(3, neopets_sim::projectile::Shot::remote(
                neopets_sim::projectile::Setup { kind: 2, visual: 23, element: 1, owner: Some(77), start: [0.0; 3], velocity: [0.0, 0.0, 1.0], lifetime: 9, damage: 3.0, radius: 1.0, spin: 0.0, area_radius: 0.0, area_duration: 0, key: -1, ignore_world: false, status: (-1, 0, 0), effects: [-1; 6], trail: -1 },
                [1.0; 3], [0.0, 0.0, 1.0], 5, false, true,
            ))],
            show: Show {
                music: [(Some(5), 1024), (None, 0)],
                bank: (Some(12), 300, true),
                intensity: (4, 9),
                fade: ([1, 2, 3], 64.0, 128.0, 4.0),
                bars: (0.5, 1.0, 0.1, false, true),
                monster: Some((600, 42, b"Name".to_vec(), 30, 80, 50)),
                message: Some((7, 90)),
                shake: Some((0.1, -0.2, None)),
            },
            ..Default::default()
        };
        let bytes = s.encode();
        let (kind, mut r) = Reader::open(&bytes).expect("header");
        assert_eq!(kind, STATE);
        assert_eq!(State::decode(&mut r), Some(s));
    }

    /// Broken messages (every cut of a full state, and random bytes behind a valid header) are
    /// refused without a panic.
    #[test]
    fn broken_messages_never_panic() {
        let s = State {
            tick: 9,
            events: vec![(1, Event::Sound(Request::Play { id: 1, block: 2, cue: 3, volume: 4, pan: Some(5) })), (2, Event::Movie(7)), (3, Event::Stinger(2))],
            talk: Some(Talk { open: true, text: b"Hello".to_vec(), ..Default::default() }),
            characters: vec![CharState::default(); 3],
            has_story: true,
            flags: vec![1, 2, 3],
            ..Default::default()
        };
        let bytes = s.encode();
        for cut in 0..bytes.len() {
            if let Some((_, mut r)) = Reader::open(&bytes[..cut]) {
                assert!(State::decode(&mut r).is_none(), "a cut at {cut} decoded");
            }
        }
        let mut x: u64 = 0x1234_5678;
        for len in 0..400 {
            let mut b = bytes[..7].to_vec();
            for _ in 0..len {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                b.push((x >> 56) as u8);
            }
            for kind in [HELLO, WELCOME, REFUSE, INPUT, STATE, BYE, PING] {
                b[6] = kind;
                if let Some((_, mut r)) = Reader::open(&b) {
                    let _ = State::decode(&mut r);
                }
                if let Some((_, mut r)) = Reader::open(&b) {
                    let _ = Input::decode(&mut r);
                }
            }
        }
    }

    /// The joiner hears a host voice from its own listener: a near source loud, the same source
    /// heard from far away quiet; a voice with no known source keeps the host's numbers; a stop
    /// forgets the source.
    #[test]
    fn joiner_hears_from_its_own_listener() {
        let mut voices = std::collections::HashMap::new();
        voices.insert(7, ([0.0, 0.0, 10.0], 1000, 200.0, 0.0));
        let near = listener_of([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let far = listener_of([0.0, 0.0, -150.0], [0.0, 0.0, 1.0]);
        let host = Request::Play { id: 7, block: 1, cue: 2, volume: 5, pan: None };
        let (Request::Play { volume: a, pan: pa, .. }, Request::Play { volume: b, .. }) = (hear(host, &mut voices, &near), hear(host, &mut voices, &far)) else { panic!() };
        assert!(a > b && b > 0 && pa.is_some(), "near {a}, far {b}");
        let other = Request::Set { id: 8, volume: 33, pan: Some(4) };
        assert_eq!(hear(other, &mut voices, &near), other);
        hear(Request::Stop { id: 7 }, &mut voices, &near);
        assert!(voices.is_empty());
    }

    #[test]
    fn input_round_trips_with_ack() {
        let i = Input { generation: 4, ack: 99, story: 6, world: 2, side: 0.5, forward: -1.0, held: RUN | GUARD, right: [1, 255], jumps: 3, attacks: 4, squares: 5, dpad: [1, 2, 3, 4, 5] };
        let bytes = i.encode();
        let (kind, mut r) = Reader::open(&bytes).expect("header");
        assert_eq!(kind, INPUT);
        assert_eq!(Input::decode(&mut r), Some(i));
    }
}
