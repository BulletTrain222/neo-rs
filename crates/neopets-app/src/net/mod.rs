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

mod codec;
mod sync;
pub use codec::*;
// These two names are also in Bevy's prelude: the messages' own come first.
pub use codec::{Event, State};

/// The UDP port the host listens on.
pub const PORT: u16 = 47817;
const MAGIC: [u8; 4] = *b"NPCO";
const PROTOCOL: u16 = 18;
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
/// The joiner's world has differed from the host's for a while: a new welcome, please.
const RESYNC: u8 = 12;

/// This build's identity, checked at joining: the crate version and the build's own number.
fn build_id() -> String {
    format!("{} {}", env!("CARGO_PKG_VERSION"), option_env!("NEOPETS_BUILD").unwrap_or("dev"))
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
    /// Host: the joiner asked for a new welcome. Joiner: ticks its world has differed from the
    /// host's, and when it last asked.
    pub resync: bool,
    /// Joiner: when the last state arrived, and the largest gap between states and the number of
    /// states since the last trace line (milliseconds; for NEOPETS_TRACE_NETEV).
    last_state_at: Option<Instant>,
    pub gap_max: u32,
    pub gap_states: u32,
    /// Joiner: states waiting to be applied, one a tick (a one-state jitter buffer), the ticks
    /// that applied no new state, and the ticks since one was.
    pub queue: std::collections::VecDeque<State>,
    pub repeats: u32,
    starve: u32,
    pub mismatch: u32,
    resync_asked: Option<Instant>,
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
    world_last: Option<(u32, Vec<u8>, Vec<(u32, u16, u16, u8)>, Vec<(u32, u8, u32)>, Vec<(u16, i32, u32, u8)>)>,
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
            let sig = s.world.as_ref().map(|w| (w.pack, w.breakables.clone(), w.items.iter().map(|(p, k, it)| (*p, *k, it.id, it.state)).collect::<Vec<_>>(), w.chars.clone(), w.objects.iter().map(|o| (o.0, o.1, o.2.to_bits(), o.3)).collect::<Vec<_>>()));
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

    /// Joiner: the state for this tick from the jitter buffer: one a tick in order, keeping one
    /// in hand when it can (states arrive unevenly), catching up when more than three wait, and
    /// the last one again when none came.
    pub fn next_state(&mut self) -> Option<State> {
        let loaded = self.loaded;
        while self.queue.front().is_some_and(|s| s.generation != loaded) && self.queue.iter().any(|s| s.generation == loaded) {
            self.queue.pop_front();
        }
        while self.queue.len() > 3 {
            self.queue.pop_front();
        }
        if self.queue.len() >= 2 || (!self.queue.is_empty() && self.starve >= 1) {
            self.state = self.queue.pop_front();
            self.starve = 0;
        } else {
            self.starve += 1;
            self.repeats += 1;
        }
        self.state.clone()
    }

    /// Joiner: ask the host for a new welcome (at most every 2 seconds).
    pub fn ask_resync(&mut self) {
        let now = Instant::now();
        if self.role != Role::Join || self.resync_asked.is_some_and(|t| now.duration_since(t) < Duration::from_secs(2)) {
            return;
        }
        self.resync_asked = Some(now);
        let w = Writer::new(RESYNC);
        self.send(&w.0);
        info!("net: this side's world differs from the host's; asked for a new welcome");
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
    net.queue.clear();
    net.status = format!("Joined {from}.");
    if first {
        net.notice = Some("Joined the host's game.".into());
    }
    info!("net: welcome {generation} from {from}");
}

/// The port to host on: NEOPETS_NET_PORT, else a line `port=N` in private/net.txt, else `PORT`.
pub fn host_port() -> u16 {
    if let Some(p) = std::env::var("NEOPETS_NET_PORT").ok().and_then(|v| v.trim().parse().ok()) {
        return p;
    }
    std::fs::read_to_string("private/net.txt")
        .ok()
        .and_then(|t| t.lines().find_map(|l| l.trim().strip_prefix("port=").and_then(|v| v.trim().parse().ok())))
        .unwrap_or(PORT)
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
            let port = host_port();
            match UdpSocket::bind(("0.0.0.0", port)).and_then(|s| s.set_nonblocking(true).map(|_| s)) {
                Ok(s) => {
                    net.socket = Some(s);
                    net.role = Role::Host;
                    // Another port than the usual one goes in the address the friend types.
                    let shown = |ip: std::net::IpAddr| if port == PORT { ip.to_string() } else { format!("{ip}:{port}") };
                    net.status = match local_address() {
                        Some(ip) => format!("Hosting. Your friend joins with {}.", shown(ip)),
                        None => format!("Hosting on port {port}."),
                    };
                    info!("net: hosting on port {port}");
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
            (Role::Host, RESYNC) if net.peer == Some(from) => {
                net.last_heard = Some(now);
                net.resync = true;
                info!("net: the joiner asked for a new welcome");
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
                if let Some(t) = net.last_state_at {
                    net.gap_max = net.gap_max.max(now.duration_since(t).as_millis() as u32);
                }
                net.last_state_at = Some(now);
                net.gap_states += 1;
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
                    // Into the jitter buffer, in order (an older or repeated state is dropped).
                    let newest = net.queue.back().or(net.state.as_ref()).map(|o| (o.generation, o.tick));
                    if newest.is_none_or(|n| (s.generation, s.tick) > n) {
                        net.queue.push_back(s);
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
            world: Some(WorldSync { pack: 909, breakables: vec![1, 2, 0], items: vec![(909, 3, NetItem { id: 160, position: [1.0, 2.0, 3.0], state: 1, angle: 0.5, scale: 1.0, direction: [0.0, 0.0, 1.0], price: -2, reference: 0.2 })], chars: vec![(909, 1, 0), (909, 2, 3)], objects: vec![(4, 2, 1.5, 3)] }),
            anim_times: vec![(4, 0.75)],
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
