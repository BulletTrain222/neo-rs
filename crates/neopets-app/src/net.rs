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
const PROTOCOL: u16 = 2;
/// No message for this long ends the session.
const TIMEOUT: Duration = Duration::from_secs(6);

const HELLO: u8 = 1;
const WELCOME: u8 = 2;
const REFUSE: u8 = 3;
const INPUT: u8 = 4;
const STATE: u8 = 5;
const BYE: u8 = 6;

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
            w.u8(c.dead as u8);
        }
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
            characters.push(CharState { pack: r.u32()?, slot: r.u8()?, ordinal: r.u32()?, position: r.v3()?, facing: r.v3()?, clip: r.u32()?, hp: r.i32()?, dead: r.u8()? != 0 });
        }
        let n = r.u32()? as usize;
        let flags = (0..n.min(65536)).map(|_| r.u16()).collect::<Option<Vec<_>>>()?;
        let n = r.u32()? as usize;
        let items = (0..n.min(65536)).map(|_| Some((r.u16()?, r.u32()?))).collect::<Option<Vec<_>>>()?;
        let neopoints = r.i32()?;
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
        Some(Self { tick, generation, pack, heroes, eye, focus, characters, flags, items, neopoints, talk, cut })
    }
}

/// The joiner's controller for one tick: the held buttons and stick, the right stick, and press
/// counters (so a press is not lost with a packet).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub generation: u32,
    pub side: f32,
    pub forward: f32,
    pub held: u16,
    pub right: [u8; 2],
    pub jumps: u8,
    pub attacks: u8,
    pub squares: u8,
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
    pub fn from_hero(i: &HeroInput, pad: &CameraPad, generation: u32, counts: (u8, u8, u8)) -> Self {
        let bit = |b: bool, v: u16| if b { v } else { 0 };
        Self {
            generation,
            side: i.side,
            forward: i.forward,
            held: bit(i.run, RUN) | bit(i.square_held, SQUARE_HELD) | bit(i.cross_held, CROSS_HELD) | bit(i.guard, GUARD) | bit(i.attack_held, ATTACK_HELD) | bit(i.r2_held, R2) | bit(i.circle_held, CIRCLE) | bit(i.l2, L2),
            right: [pad.right_x, pad.right_y],
            jumps: counts.0,
            attacks: counts.1,
            squares: counts.2,
        }
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
        w.f32(self.side);
        w.f32(self.forward);
        w.u16(self.held);
        w.u8(self.right[0]);
        w.u8(self.right[1]);
        w.u8(self.jumps);
        w.u8(self.attacks);
        w.u8(self.squares);
        w.0
    }

    fn decode(r: &mut Reader) -> Option<Self> {
        Some(Self { generation: r.u32()?, side: r.f32()?, forward: r.f32()?, held: r.u16()?, right: [r.u8()?, r.u8()?], jumps: r.u8()?, attacks: r.u8()?, squares: r.u8()? })
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
    pub generation: u32,
    welcome: Option<Vec<u8>>,
    /// Joiner: the host's address, the slot to load, the latest state, the generation loaded,
    /// and the press counters of this side's controller.
    target: Option<SocketAddr>,
    pub load: Option<neopets_sim::save::Slot>,
    pub state: Option<State>,
    pub loaded: u32,
    pub counts: (u8, u8, u8),
    pub outgoing: Option<Input>,
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
        }
    }

    fn stop(&mut self, status: &str) {
        if self.socket.is_some() && self.peer.is_some() {
            let w = Writer::new(BYE);
            self.send(&w.0);
        }
        *self = Self { disc: self.disc, status: status.to_string(), ..Default::default() };
    }

    /// Host: the state after a tick, sent to the joiner.
    pub fn send_state(&mut self, s: &State) {
        if self.role == Role::Host && self.peer.is_some() {
            let bytes = s.encode();
            self.send(&bytes);
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
    pub fn rewelcome(&mut self, slot: &neopets_sim::save::Slot, pack: u32) {
        if self.role != Role::Host || self.peer.is_none() || pack == self.welcome_pack {
            return;
        }
        self.welcome_pack = pack;
        self.generation += 1;
        self.welcome = Some(welcome(self.generation, slot));
        self.last_sent = None;
        info!("net: world {pack}, welcome {}", self.generation);
    }
}

fn welcome(generation: u32, slot: &neopets_sim::save::Slot) -> Vec<u8> {
    let mut w = Writer::new(WELCOME);
    w.u32(generation);
    w.bytes(&slot.0);
    w.0
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
pub fn net_step(mut net: ResMut<Net>, mut play: Option<ResMut<PlayState>>, mut archive: Option<ResMut<crate::Archive>>, mut coop: ResMut<crate::coop::Coop>) {
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
    let started = START.get_or_init(Instant::now).elapsed() > Duration::from_secs(5) && std::env::var_os("NEOPETS_NET_HOST").is_none();
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
    // The messages waiting.
    let mut buf = vec![0u8; 65536];
    loop {
        let Some(s) = net.socket.as_ref() else { break };
        let Ok((n, from)) = s.recv_from(&mut buf) else { break };
        let Some((kind, mut r)) = Reader::open(&buf[..n]) else { continue };
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
                    let mut slot = neopets_sim::save::Slot::empty();
                    p.play.write_slot(&mut slot, b"NETPLAY");
                    net.welcome = Some(welcome(net.generation, &slot));
                    net.status = format!("{from} joined.");
                    info!("net: {from} joined");
                }
            }
            (Role::Host, INPUT) if net.peer == Some(from) => {
                net.last_heard = Some(now);
                if let Some(i) = Input::decode(&mut r) {
                    if i.generation == net.generation {
                        net.welcome = None;
                    }
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
                } else {
                    net.stop("The host ended the game.");
                }
            }
            (Role::Join, WELCOME) if net.target == Some(from) => {
                net.last_heard = Some(now);
                net.peer = Some(from);
                let (Some(generation), Some(bytes)) = (r.u32(), r.bytes()) else { continue };
                if generation != net.loaded && bytes.len() == neopets_sim::save::SLOT_SIZE {
                    net.load = Some(neopets_sim::save::Slot(bytes.to_vec()));
                    net.loaded = generation;
                    net.state = None;
                    net.status = format!("Joined {from}.");
                    info!("net: welcome {generation} from {from}");
                }
            }
            (Role::Join, REFUSE) if net.target == Some(from) => {
                let why = r.text().unwrap_or_default();
                net.stop(&format!("The host said no: {why}."));
            }
            (Role::Join, STATE) if net.target == Some(from) => {
                net.last_heard = Some(now);
                if let Some(s) = State::decode(&mut r) {
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
    if let Some(w) = net.welcome.clone().filter(|_| due && net.role == Role::Host) {
        net.send(&w);
        net.last_sent = Some(now);
    }
    if net.role == Role::Join {
        if let Some(i) = net.outgoing.take() {
            net.send_input(i);
        }
    }
    // The host's player 2 input from the joiner's controller (its presses since the last frame).
    if net.role == Role::Host {
        coop.remote = net.remote.map(|i| {
            let (h, c) = i.to_hero(&net.seen);
            (h, c)
        });
        if let Some(i) = net.remote {
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
            info!("net: the joiner timed out");
        } else {
            info!("net: the host timed out");
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
                .map(|c| CharState { pack: c.pack, slot: c.slot, ordinal: c.ordinal as u32, position: c.position, facing: c.facing, clip: c.anim.clip_state().action, hp: c.hp, dead: c.dying.is_some() || c.enemy.as_ref().is_some_and(|e| e.is_dead()) })
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
        }
        for c in self.characters.iter_mut() {
            let Some(h) = s.characters.iter().find(|h| h.pack == c.pack && h.slot == c.slot && h.ordinal as usize == c.ordinal) else { continue };
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
