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

pub mod apart;
mod codec;
pub mod upnp;
mod sync;
pub use codec::*;
// These two names are also in Bevy's prelude: the messages' own come first.
pub use codec::{Event, State};

/// The UDP port the host listens on.
pub const PORT: u16 = 47817;
const MAGIC: [u8; 4] = *b"NPCO";
const PROTOCOL: u16 = 34;
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
/// The most characters in one state (27 bytes each when packed; with a long talk box and the
/// monster bar a state then stays under the limit).
const CHARS_A_STATE: usize = 18;
/// Events that did not fit in a state, in up to two more messages a tick.
const EVENTS: u8 = 11;
/// The joiner's world has differed from the host's for a while: a new welcome, please.
const RESYNC: u8 = 12;
/// The host sent the joiner away (the Multiplayer page's Remove row).
const REMOVED: u8 = 13;
/// Players in separate worlds (net/apart.rs): each side's world, vitals and story changes.
const APART: u8 = 14;
/// The game finder: a search for games on this home network (sent to everyone on it), and a
/// hosting game's answer (its version, players, password and port).
const FIND: u8 = 15;
const HERE: u8 = 16;

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

/// A game the finder heard of: where, its players, whether it needs a password, whether its
/// version is this one, whether it is in a game yet, and when it last answered.
#[derive(Clone, Debug)]
pub struct Found {
    pub addr: SocketAddr,
    pub players: u8,
    pub max: u8,
    pub password: bool,
    pub same_build: bool,
    pub ready: bool,
    pub id: u64,
    pub seen: Instant,
}

impl Found {
    /// The address to join it with (the port only when not the usual one).
    pub fn join_text(&self) -> String {
        if self.addr.port() == PORT { self.addr.ip().to_string() } else { self.addr.to_string() }
    }
    /// Joinable now (the same version, in a game, not full).
    pub fn joinable(&self) -> bool {
        self.same_build && self.ready && self.players < self.max
    }
}

/// The game finder: its socket, the games heard of, and when it last asked.
#[derive(Default)]
pub struct Finder {
    socket: Option<UdpSocket>,
    pub found: Vec<Found>,
    asked: Option<Instant>,
}

/// How often the finder asks, and how long a game that stops answering stays listed.
const FIND_EVERY: Duration = Duration::from_secs(2);
const FIND_KEEP: Duration = Duration::from_secs(7);

impl Finder {
    /// One step: while `on`, ask everyone on this home network every FIND_EVERY (and this
    /// computer, for a game hosted here), and list the games that answer; off, close.
    pub fn step(&mut self, on: bool, port: u16) {
        if !on {
            self.socket = None;
            self.found.clear();
            self.asked = None;
            return;
        }
        if self.socket.is_none() {
            let s = UdpSocket::bind("0.0.0.0:0").and_then(|s| s.set_nonblocking(true).and(s.set_broadcast(true)).map(|_| s));
            match s {
                Ok(s) => self.socket = Some(s),
                Err(e) => {
                    info!("net: the game finder could not start: {e}");
                    return;
                }
            }
        }
        let Some(s) = self.socket.as_ref() else { return };
        let now = Instant::now();
        if self.asked.is_none_or(|t| now.duration_since(t) >= FIND_EVERY) {
            self.asked = Some(now);
            let mut w = Writer::new(FIND);
            w.text(&build_id());
            let mut to: Vec<SocketAddr> = vec![SocketAddr::from(([255, 255, 255, 255], port)), SocketAddr::from(([127, 0, 0, 1], port))];
            // This network's own broadcast address too (some systems send 255.255.255.255 out of
            // one network card only); a home network is nearly always x.y.z.0 to x.y.z.255.
            if let Some(std::net::IpAddr::V4(ip)) = local_address() {
                let o = ip.octets();
                to.push(SocketAddr::from(([o[0], o[1], o[2], 255], port)));
            }
            for a in to {
                let _ = s.send_to(&w.0, a);
            }
        }
        let mut buf = [0u8; 512];
        while let Ok((n, from)) = s.recv_from(&mut buf) {
            let b = &buf[..n];
            if !upnp::on_this_network(from.ip()) || b.len() < 7 || b[..4] != MAGIC || b[6] != HERE {
                continue;
            }
            let other_version = u16::from_le_bytes([b[4], b[5]]) != PROTOCOL;
            let parsed = if other_version {
                None
            } else {
                Reader::open(b).and_then(|(_, mut r)| Some((r.text()?, r.u8()?, r.u8()?, r.u8()?, r.u16()?, r.u8()?, r.u64()?)))
            };
            let (same_build, players, max, password, port, ready, id) = match parsed {
                Some((build, players, max, password, port, ready, id)) => (build == build_id(), players, max.max(1), password != 0, port, ready != 0, id),
                None => (false, 0, 4, false, from.port(), true, u64::from(from.port()) ^ u64::from(u32::from(match from.ip() { std::net::IpAddr::V4(v) => v, _ => std::net::Ipv4Addr::UNSPECIFIED }))),
            };
            let addr = SocketAddr::new(from.ip(), port);
            let f = Found { addr, players, max, password, same_build, ready, id, seen: now };
            // One game heard at two addresses (this computer's own game answers both its
            // network address and 127.0.0.1): the network address is kept.
            match self.found.iter_mut().find(|g| g.id == f.id) {
                Some(g) => {
                    let keep_addr = if g.addr.ip().is_loopback() { f.addr } else { g.addr };
                    *g = Found { addr: keep_addr, ..f };
                }
                None => {
                    info!("net: the game finder heard of a game at {addr}");
                    self.found.push(f);
                }
            }
        }
        self.found.retain(|g| now.duration_since(g.seen) < FIND_KEEP);
        self.found.sort_by_key(|g| (!g.joinable(), g.addr));
        self.found.truncate(crate::display::GAME_ROWS.len());
    }
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
    /// Joiner: the state applied before `state` (the movement a missing state carries on).
    prev_state: Option<State>,
    /// Joiner, for the trace: the host's hero's last place and step here, the stalls and the
    /// largest step since the last trace line.
    pub smooth: (Option<[f32; 3]>, f32, u32, f32),
    /// Joiner (diagnostic, NEOPETS_TRACE_NETEV): this side's own hero: its last place, last
    /// step, and the ticks it stood still right after moving (a stall).
    pub own_smooth: (Option<[f32; 3]>, f32, u32),
    /// Joiner (diagnostic): this side's own hero's last step and the ticks its step jerked
    /// (turned back, or changed length by more than half, while moving).
    pub own_jerk: ([f32; 3], u32),
    /// Joiner: the tick count of the jitter buffer's gradual catch-up (`next_state`).
    catch_up: u32,
    /// Joiner, for the trace: the most states waiting in the buffer since the last line.
    pub queue_most: usize,
    /// For the trace: microseconds spent on the online work since the last trace line (the
    /// host's state made and sent, the joiner's state applied).
    pub cpu_us: u64,
    /// Joiner: the recent ticks with no new state (fading), and the states kept in hand.
    jitter: f32,
    pub hold: usize,
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
    /// Host: when each recent state tick went out, and the last one the joiner reported seen.
    /// Both: the round trip (milliseconds, smoothed), the traffic each way a second (bytes) and
    /// when it was last sampled, and the hosting line with the address to type.
    sent_at: std::collections::VecDeque<(u32, Instant)>,
    last_seen: u32,
    pub seen_tick: u32,
    pub rtt_ms: f32,
    pub rate: (f32, f32),
    rate_at: Option<(Instant, u64, u64)>,
    host_line: String,
    /// Joiner: the address joined (for joining again after the host is lost), the address to
    /// join again next frame, and until when a join again waits for the host.
    join_text: String,
    pub retry: Option<String>,
    retry_until: Option<Instant>,
    /// Host: player 2's health and magic when it went (the game generation, HP, MP): a player 2
    /// joining the same game again gets them back (docs/MULTIPLAYER.md, Default).
    kept: Option<(u32, i32, i32)>,
    /// Joiner: item-screen commands waiting for the host, and how many the host has carried
    /// out. Host: the number of the joiner's last command carried out.
    pub cmds: std::collections::VecDeque<(u8, u16, u16)>,
    pub cmds_done: u8,
    cmd_done: u8,
    /// The other side's player name (the joiner's on the host, the host's on the joiner).
    pub peer_name: String,
    /// Joiner: the host's interior named by the welcome just received (entered once its game
    /// has loaded).
    pub load_interior: Option<u32>,
    /// Host: the router's answer being waited for (UPnP), and the port it forwards.
    upnp_rx: Option<std::sync::Mutex<std::sync::mpsc::Receiver<(u16, upnp::Outcome)>>>,
    /// Host: refusals and finder answers sent to strangers in the current second (at most
    /// REFUSALS_A_SECOND, so the game cannot be used to send floods at a third party's address).
    refusals: (Option<Instant>, u32),
    /// Host: the joiner's input number last applied, and how many ticks more it was applied
    /// (a state's `input_extra`).
    input_last: u32,
    input_repeats: u8,
    /// Host: when the host was last told of a refused join (at most one line every 10 seconds).
    refuse_note: Option<Instant>,
    /// Host: this hosting's number (the game finder tells one game seen twice apart).
    host_id: u64,
    /// The game finder (net_step): games hosting on this home network.
    pub finder: Finder,
    pub upnp_port: Option<u16>,
    /// Players in separate worlds (net/apart.rs): whether this side runs its own world apart
    /// from the other; host: the door the joiner is to take, until it answers; joiner: the door
    /// to take now (step_sim); the story as last agreed; the numbered batches of changes not yet
    /// acknowledged, the next number, the other side's last batch put in and its changes waiting
    /// for step_sim; the other side's world and vitals; this side's (step_sim fills it); when
    /// this side last sent.
    pub apart: bool,
    apart_go: Option<(u16, u32, [f32; 3])>,
    /// Host: the world apart began in.
    apart_from: Option<u32>,
    pub apart_start: Option<(u16, [f32; 3])>,
    pub story_base: Option<apart::StoryBase>,
    ops_out: std::collections::VecDeque<(u32, Vec<apart::StoryOp>)>,
    ops_next: u32,
    ops_in_last: u32,
    pub ops_in: Vec<apart::StoryOp>,
    pub other_pack: u32,
    /// Host: the next input from the joiner is the new starting point of its press counters.
    reseed: bool,
    /// Host: the joiner has been in another world since going apart (only then can they meet).
    pub other_left: bool,
    other_vitals: (i32, i32),
    pub apart_me: (u32, i32, i32, [f32; 3]),
    /// Host: where the joiner's hero last was (it stays there when they meet in this world).
    other_pos: [f32; 3],
    apart_sent: Option<Instant>,
    /// Joiner: when this join began (a join with no answer gives up after 20 seconds).
    join_started: Option<Instant>,
    /// Host, for the trace: events made since the last trace line, by kind (`kind_of`).
    kinds: [u32; 10],
    /// Host: the voices sent with a place (the joiner works out their volume and pan itself
    /// each tick, so their volume and pan changes are not sent).
    pub placed: std::collections::HashSet<usize>,
    /// Host: this session's joiner plays `others[seat]` (seat 0 is player 2).
    pub seat: usize,
    /// Host: the sessions of the other joiners (players 3 and 4), sharing this one's socket.
    pub extra: Vec<Net>,
    /// Host: the Multiplayer page's Remove row was pressed (the last joiner goes).
    remove_wanted: bool,
    /// Host: the hero this joiner's player had when it went apart (it comes back with it).
    apart_who: usize,
}

/// The most joiners a host takes (players 2 to 4).
pub const MAX_JOINERS: usize = crate::coop::MAX_PLAYERS - 1;
/// The most refusals the host sends a second (to all strangers together).
const REFUSALS_A_SECOND: u32 = 10;

/// NEOPETS_NET_FUZZ=percent (diagnostic): the share of messages sent damaged.
fn fuzz_percent() -> Option<u64> {
    static P: std::sync::OnceLock<Option<u64>> = std::sync::OnceLock::new();
    *P.get_or_init(|| std::env::var("NEOPETS_NET_FUZZ").ok().and_then(|v| v.trim().parse::<u64>().ok()).filter(|&p| p > 0))
}

impl Net {
    /// A pseudo-random number (the diagnostic loss and damage).
    fn next_random(&mut self) -> u64 {
        self.lossy = self.lossy.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.lossy >> 33
    }

    /// A damaged copy of a message (diagnostic): its header kept so it is read, then a few bytes
    /// changed, a run set to 0xFF (huge lengths and indices), cut short, or junk to the end.
    fn damage(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut b = bytes.to_vec();
        if b.len() <= 7 {
            return b;
        }
        match self.next_random() % 10 {
            0..=5 => {
                for _ in 0..1 + self.next_random() % 4 {
                    let at = 7 + (self.next_random() as usize) % (b.len() - 7);
                    b[at] = self.next_random() as u8;
                }
            }
            6 | 7 => {
                let at = 7 + (self.next_random() as usize) % (b.len() - 7);
                for x in b.iter_mut().skip(at).take(4) {
                    *x = 0xFF;
                }
            }
            8 => {
                let n = 7 + (self.next_random() as usize) % (b.len() - 7);
                b.truncate(n);
            }
            _ => {
                let n = 7 + (self.next_random() as usize) % 1400;
                b.truncate(7);
                while b.len() < n {
                    b.push(self.next_random() as u8);
                }
            }
        }
        b
    }
}

/// When the game closes while online: the others are told (a goodbye), and a router forward
/// this game opened is closed before the game ends (waiting up to a few seconds).
pub fn leave_on_exit(mut exit: MessageReader<AppExit>, mut net: ResMut<Net>) {
    if exit.read().next().is_none() || net.role == Role::Off {
        return;
    }
    let (port, pending) = (net.upnp_port.take(), net.upnp_rx.take().and_then(|m| m.into_inner().ok()));
    net.stop("");
    if port.is_some() || pending.is_some() {
        info!("net: closing the router's forward before the game ends");
        upnp::release(port, pending, true);
    }
}

/// An event's kind for the trace: sound play, set, stop; effect add, one-shot, update, other;
/// voice; music and movies; rumble and notices.
fn kind_of(e: &Event) -> usize {
    use neopets_sim::effect::Op;
    use neopets_sim::sound::Request;
    match e {
        Event::Sound(Request::Play { .. }) => 0,
        Event::Sound(Request::Set { .. }) => 1,
        Event::Sound(Request::Stop { .. }) => 2,
        Event::Fx(Op::Add { .. }) => 3,
        Event::Fx(Op::OneShot { .. }) => 4,
        Event::Fx(Op::Update { .. }) => 5,
        Event::Fx(_) => 6,
        Event::Voice { .. } => 7,
        Event::Stinger(_) | Event::Emote(..) | Event::Movie(_) => 8,
        Event::Rumble(..) | Event::Notice(_) => 9,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    Host,
    Join(String),
    Leave,
    /// Host: send player 2 away and keep hosting.
    Remove,
}

impl Net {
    /// Whether this side follows a host (a joiner with a loaded game).
    pub fn following(&self) -> bool {
        self.role == Role::Join && self.loaded > 0
    }

    /// Host: a refusal to a hello, at most REFUSALS_A_SECOND a second (a hello's sender
    /// address is not checked, so unlimited answers could be aimed at someone else).
    fn refuse(&mut self, to: SocketAddr, why: &str) {
        let mut w = Writer::new(REFUSE);
        w.text(why);
        self.answer_stranger(to, &w.0);
    }

    /// Host: a message to someone not in the game (a refusal, a finder's answer), within the
    /// budget of REFUSALS_A_SECOND.
    fn answer_stranger(&mut self, to: SocketAddr, bytes: &[u8]) {
        let now = Instant::now();
        if self.refusals.0.is_none_or(|t| now.duration_since(t) >= Duration::from_secs(1)) {
            self.refusals = (Some(now), 0);
        }
        if self.refusals.1 >= REFUSALS_A_SECOND {
            return;
        }
        self.refusals.1 += 1;
        if let Some(s) = self.socket.as_ref() {
            let _ = s.send_to(bytes, to);
        }
    }

    /// Host: a line to the host that someone could not join (at most one every 10 seconds).
    fn note_refusal(&mut self, line: &str) {
        let now = Instant::now();
        if self.refuse_note.is_none_or(|t| now.duration_since(t) >= Duration::from_secs(10)) {
            self.refuse_note = Some(now);
            self.notice = Some(line.to_string());
        }
    }

    /// Joiner, while joining (not yet in the host's game): what is happening, for the line on
    /// screen.
    pub fn join_line(&self) -> Option<String> {
        if self.role != Role::Join || self.loaded > 0 {
            return None;
        }
        let at = self.target.map_or(String::new(), |a| if a.port() == PORT { a.ip().to_string() } else { a.to_string() });
        if let Some((_, parts)) = self.parts.as_ref() {
            let have = parts.iter().filter(|p| p.is_some()).count();
            return Some(format!("Loading the host's game ({have} of {})...", parts.len()));
        }
        if self.retry_until.is_some() {
            return Some(format!("Lost the host. Joining {at} again..."));
        }
        let secs = self.join_started.map_or(0, |t| t.elapsed().as_secs());
        Some(if self.last_heard.is_some() { format!("Joining {at}: the host answered...") } else { format!("Joining {at}... ({secs} s)") })
    }

    /// Joiner: the host sent something this game could not use (main.rs caught it): leave its
    /// game for the title screen.
    pub fn drop_host(&mut self) {
        warn!("net: the host sent something this game could not use; leaving its game");
        if self.loaded > 0 {
            self.ended = true;
        }
        self.notice = Some("The host sent something this game could not use. You left the game.".into());
        self.stop("Left: the host sent something this game could not use.");
    }

    fn send(&mut self, bytes: &[u8]) {
        if outage() {
            return;
        }
        // NEOPETS_NET_FUZZ=percent (diagnostic): that share of the messages sent go out damaged,
        // to show the other side passes them over without harm.
        let damaged;
        let bytes = match fuzz_percent() {
            // (A hello goes whole, so the damaged messages reach a session.)
            Some(p) if bytes.get(6) != Some(&HELLO) && self.next_random() % 100 < p => {
                damaged = self.damage(bytes);
                &damaged[..]
            }
            _ => bytes,
        };
        if let (Some(s), Some(p)) = (self.socket.as_ref(), self.peer.or(self.target)) {
            let _ = s.send_to(bytes, p);
            self.bytes_out += bytes.len() as u64 + 28;
            self.last_any = Some(Instant::now());
        }
    }

    /// Host: a new joiner starts every channel afresh (events from 1, the story, the world sync
    /// and the characters sent again, the round trip measured again). Before, a joiner coming
    /// back to the same host waited for event 1 while the host's numbering went on, and the
    /// events piled up unplayed.
    fn reset_peer(&mut self) {
        self.remote = None;
        self.seen = Input::default();
        self.events.clear();
        self.next_event = 0;
        self.acked = 0;
        self.story_last = None;
        self.story_acked = 0;
        self.world_last = None;
        self.world_acked = 0;
        self.chars_last.clear();
        self.moving_last.clear();
        self.cursor = (0, 0);
        self.blob_sent = Default::default();
        self.sent_at.clear();
        self.last_seen = 0;
        self.rtt_ms = 0.0;
        self.resync = false;
        self.rate_at = None;
        self.cmd_done = 0;
        self.placed.clear();
    }

    fn stop(&mut self, status: &str) {
        for e in &mut self.extra {
            e.stop("");
        }
        if self.socket.is_some() && self.peer.is_some() {
            let w = Writer::new(BYE);
            self.send(&w.0);
        }
        // The router's forward ends with the hosting, however it ended (an answer still on its
        // way closes the forward it reports), on its own thread.
        let (port, pending) = (self.upnp_port.take(), self.upnp_rx.take().and_then(|m| m.into_inner().ok()));
        upnp::release(port, pending, false);
        *self = Self { disc: self.disc, status: status.to_string(), notice: self.notice.take(), ended: self.ended, kept: self.kept, finder: std::mem::take(&mut self.finder), ..Default::default() };
    }

    /// Host: the state after a tick, sent to the joiner with the events not yet acknowledged
    /// (at most 96, oldest first).
    pub fn send_state(&mut self, s: &State) {
        if self.role == Role::Host && self.peer.is_some() && !self.apart {
            self.sent_at.push_back((s.tick as u32, Instant::now()));
            while self.sent_at.len() > 240 {
                self.sent_at.pop_front();
            }
            let acked = self.acked;
            while self.events.front().is_some_and(|e| e.0 <= acked) {
                self.events.pop_front();
            }
            let mut s = s.clone();
            s.rtt = self.rtt_ms.round().clamp(0.0, 65535.0) as u16;
            s.cmd_ack = self.cmd_done;
            let seq = self.remote.map_or(0, |i| i.seq);
            self.input_repeats = if seq == self.input_last { self.input_repeats.saturating_add(1) } else { 0 };
            self.input_last = seq;
            s.input_seq = seq;
            s.input_extra = self.input_repeats;
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
                // At most 8 a state, from the turning start (a burst of drops cannot take a
                // state past the limit; one left out stays changed for a following state).
                let mut keep = vec![false; s.moving.len()];
                let mut sent = 0;
                for k in 0..s.moving.len() {
                    let i = (start + k) % n;
                    let (pack, slot, at, _) = s.moving[i];
                    let changed = last.get(&(pack, slot)).is_none_or(|p| (0..3).any(|j| (p[j] - at[j]).abs() > 1e-3));
                    if (changed || k < 4) && sent < 8 {
                        if changed {
                            last.insert((pack, slot), at);
                        }
                        keep[i] = true;
                        sent += 1;
                    }
                }
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
                // At most CHARS_A_STATE a state, taken from the turning start, so a crowd never
                // takes a state past the limit; a changed character left out stays changed and
                // goes with a following state.
                let mut keep = vec![false; s.characters.len()];
                let mut sent = 0;
                for k in 0..s.characters.len() {
                    let i = (start + k) % n;
                    let c = &s.characters[i];
                    let key = (c.pack, c.slot, c.ordinal);
                    let changed = last.get(&key).is_none_or(|p| {
                        p.clip != c.clip || (p.rate - c.rate).abs() > 1e-3 || p.hp != c.hp || p.dead != c.dead || p.inactive != c.inactive || (0..3).any(|j| (p.position[j] - c.position[j]).abs() > 1e-3 || (p.facing[j] - c.facing[j]).abs() > 1e-3)
                    });
                    if (changed || k < 4) && sent < CHARS_A_STATE {
                        if changed {
                            last.insert(key, *c);
                        }
                        keep[i] = true;
                        sent += 1;
                    }
                }
                let mut it = keep.iter();
                s.characters.retain(|_| *it.next().unwrap_or(&false));
                self.cursor.0 = start + 4;
                if s.tick % 120 == 0 {
                    last.clear();
                }
            }
            // The events not yet acknowledged, oldest first, up to 96 and in the room the state
            // leaves under the limit; with no room left (a large state) none, and the events
            // messages below carry them (before, at least one went in and a state could pass the
            // limit: 1,201 bytes in a 20-minute session).
            let room = STATE_LIMIT.saturating_sub(s.encode().len());
            let mut size = 0;
            s.events = Vec::new();
            for e in self.events.iter().take(96) {
                let mut w = Writer(Vec::new());
                e.1.write(&mut w);
                size += w.0.len() + 4;
                if size > room {
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
                println!("NETEV host tick {}: {} events made, {} acknowledged, {} waiting; state {} bytes ({} characters, {} flags, {} items, {} events, {} moving, {} shots, talk {}); sent {} received {} bytes; round trip {:.1} ms; {}; made by kind (play set stop add oneshot update fx voice clip other) {:?}; state work {:.1} us a tick", s.tick, self.next_event, self.acked, self.events.len(), bytes.len(), s.characters.len(), s.flags.len(), s.items.len(), s.events.len(), s.moving.len(), s.shots.len(), s.talk.is_some(), self.bytes_out, self.bytes_in, self.rtt_ms, self.status, std::mem::take(&mut self.kinds), std::mem::take(&mut self.cpu_us) as f32 / 300.0);
            }
            self.send(&bytes);
        }
    }

    /// Joiner: the state for this tick from the jitter buffer: one a tick in order, keeping one
    /// in hand when it can (states arrive unevenly), catching up when more than three wait, and
    /// the last one again when none came.
    pub fn next_state(&mut self) -> Option<State> {
        let loaded = self.loaded;
        // About the ticks with nothing new over the last few seconds (it fades by 1% a tick).
        self.jitter *= 0.99;
        while self.queue.front().is_some_and(|s| s.generation != loaded) && self.queue.iter().any(|s| s.generation == loaded) {
            self.queue.pop_front();
        }
        // A burst (the host's states sent several to a frame, or this side's frame late) is kept
        // and played down gradually: beyond the hold plus two, one extra state is skipped every
        // fourth tick (a small step), and only past 12 are the oldest dropped at once. Before,
        // everything past 3 was dropped in one go, and the heroes jumped on several ticks at
        // once: with four games on one computer the joiners' walking jerked, most while the
        // host ran about (user report 154).
        while self.queue.len() > 12 {
            self.queue.pop_front();
        }
        self.catch_up = self.catch_up.wrapping_add(1);
        if self.queue.len() > self.hold + 2 && self.catch_up % 4 == 0 {
            self.queue.pop_front();
        }
        // The states kept in hand follow how unevenly they arrive (Default): one, two after
        // repeated ticks with nothing new, three when that goes on; back down when they come
        // evenly. NEOPETS_NET_HOLD=n (diagnostic) fixes it.
        let hold = std::env::var("NEOPETS_NET_HOLD").ok().and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(if self.jitter > 6.0 {
            3
        } else if self.jitter > 2.0 {
            2
        } else {
            1
        });
        self.hold = hold;
        if self.queue.len() > hold || (!self.queue.is_empty() && self.starve >= 1) {
            let next = self.queue.pop_front();
            self.prev_state = std::mem::replace(&mut self.state, next);
            self.starve = 0;
        } else {
            self.starve += 1;
            self.repeats += 1;
            // Only a tick with nothing at all waiting counts as uneven arrival (not a tick held
            // on purpose to build the buffer up).
            if self.queue.is_empty() {
                self.jitter += 1.0;
            }
            // No state for this tick: the heroes and characters carry on by their last tick's
            // movement (at most 4 ticks, and only movement under 2 units a tick), and the next
            // state puts them right. NEOPETS_NET_NOSMOOTH (diagnostic) holds them instead.
            if self.starve <= 4 && std::env::var_os("NEOPETS_NET_NOSMOOTH").is_none() {
                if let (Some(s), Some(p)) = (self.state.as_ref(), self.prev_state.as_ref()) {
                    return Some(carried(s, p, self.starve as f32));
                }
            }
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

    /// Whether any joiner is in (host).
    pub fn any_peer(&self) -> bool {
        self.peer.is_some() || self.extra.iter().any(|e| e.peer.is_some())
    }

    /// How many joiners are in (host).
    pub fn peers(&self) -> usize {
        self.peer.is_some() as usize + self.extra.iter().filter(|e| e.peer.is_some()).count()
    }

    /// Host: an event for every joiner.
    pub fn push_event(&mut self, e: Event) {
        for x in &mut self.extra {
            x.push_own(e.clone());
        }
        self.push_own(e);
    }

    /// Host: an event for this session's joiner only (kept while it is in; at most 4,000
    /// waiting, then a new welcome).
    pub fn push_own(&mut self, e: Event) {
        if self.role != Role::Host || self.peer.is_none() || self.apart {
            return;
        }
        self.next_event += 1;
        let n = self.next_event;
        self.kinds[kind_of(&e)] += 1;
        self.events.push_back((n, e));
        // More waiting than the channel can carry: dropping the oldest would leave the joiner
        // waiting for it for ever (the 20-minute test's event channel stopped this way), so the
        // queue is emptied and the joiner gets a new welcome (its game and running effects as
        // now), counting events on from there.
        // NEOPETS_NET_QUEUE=n (diagnostic): a smaller limit, to test it.
        static LIMIT: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let limit = *LIMIT.get_or_init(|| std::env::var("NEOPETS_NET_QUEUE").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(4000));
        if self.events.len() > limit {
            self.events.clear();
            self.resync = true;
            info!("net: more than {limit} events waiting: a new welcome instead");
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
    /// The story as it stands is the agreed one; nothing waits in either direction.
    pub fn start_story(&mut self, s: &neopets_sim::level::GameState) {
        self.story_base = Some(apart::StoryBase::of(s));
        self.ops_out.clear();
        (self.ops_next, self.ops_in_last) = (0, 0);
        self.ops_in.clear();
        self.apart_sent = None;
    }

    /// Apart, each tick: the other side's story changes put in, this side's since the last
    /// agreed story queued to go (numbered, sent until acknowledged).
    pub fn story_sync(&mut self, s: &mut neopets_sim::level::GameState) {
        let Some(base) = self.story_base.as_mut() else { return };
        let incoming = std::mem::take(&mut self.ops_in);
        if !incoming.is_empty() {
            info!("net: the other world's story changes: {incoming:?}");
        }
        apart::apply_ops(&incoming, s, base);
        let changes = base.changes(s);
        if !changes.is_empty() {
            info!("net: this world's story changes: {changes:?}");
            *base = apart::StoryBase::of(s);
            self.ops_next += 1;
            self.ops_out.push_back((self.ops_next, changes));
        }
    }

    /// Host: player 2 went through door `door` alone (Doors take Only you): it leaves this game,
    /// and the joiner is told to take that door with its own game.
    pub fn go_apart_host(&mut self, door: usize, play: &mut crate::play::Play) {
        if self.role != Role::Host || self.peer.is_none() || self.apart {
            return;
        }
        keep_p2(self, play);
        // The player's place goes with the door (the host's last state did not have it there
        // yet); a walk into the next area travels as door 0xFFFF.
        let at = play.others.get(self.seat).map_or([0.0; 3], |o| o.loco.position());
        self.apart_who = play.others.get(self.seat).map_or(0, |o| o.who.min(1));
        play.remove_seat(self.seat);
        self.remote = None;
        self.apart = true;
        self.apart_go = Some((if door == crate::play::WALKED_APART { u16::MAX } else { door as u16 }, play.world.pack, at));
        self.other_pack = play.world.pack;
        self.other_left = false;
        self.apart_from = Some(play.world.pack);
        self.start_story(&play.state);
        self.notice = Some(format!("{} went off on their own.", if self.peer_name.is_empty() { "Player 2" } else { self.peer_name.as_str() }));
        if door == crate::play::WALKED_APART {
            info!("net: {} walks into the next area alone; apart", self.peer_name);
        } else {
            info!("net: {} takes door {door} of {} alone; apart", self.peer_name, play.world.pack);
        }
    }

    /// Host: the joiner of `seat` went through door `door` alone (Doors take Only you): its
    /// session goes apart and the players after it move up a seat.
    pub fn go_apart_seat(&mut self, seat: usize, door: usize, play: &mut crate::play::Play) {
        let mut extra = std::mem::take(&mut self.extra);
        let found = std::iter::once(&mut *self).chain(extra.iter_mut()).find(|n| n.peer.is_some() && !n.apart && n.seat == seat).map(|n| {
            n.go_apart_host(door, play);
        });
        if found.is_some() {
            renumber(self, &mut extra, Some(seat));
        }
        self.extra = extra;
    }

    /// Host, each tick: every joiner apart meets the host again when it reports the host's
    /// world, or a story moment here (a movie, a hero switch, game over) calls it back.
    pub fn host_rejoins(&mut self, play: &mut crate::play::Play, story: bool) {
        let mut extra = std::mem::take(&mut self.extra);
        for n in std::iter::once(&mut *self).chain(extra.iter_mut()) {
            if n.apart_settled() {
                let met = n.other_left && n.other_pack == play.world.pack && play.job.is_none();
                if met || story {
                    n.rejoin_host(play, if met { "met in this world" } else { "a story moment" });
                }
            }
        }
        self.extra = extra;
    }

    /// Each tick while apart: the story changes each way, and this side's world, health, magic
    /// and place (`me`) for the other side (every joiner apart, on a host).
    pub fn apart_sync(&mut self, state: &mut neopets_sim::level::GameState, me: (u32, i32, i32, [f32; 3])) {
        let mut extra = std::mem::take(&mut self.extra);
        for n in std::iter::once(&mut *self).chain(extra.iter_mut()) {
            if n.apart {
                n.story_sync(state);
                n.apart_me = me;
            }
        }
        self.extra = extra;
    }

    /// Whether this side (or, on a host, any joiner) is in separate worlds.
    pub fn any_apart(&self) -> bool {
        self.apart || self.extra.iter().any(|e| e.apart)
    }

    /// Host: the joiners playing in the host's world (not apart).
    pub fn with_host(&self) -> usize {
        std::iter::once(self).chain(self.extra.iter()).filter(|n| n.peer.is_some() && !n.apart).count()
    }

    /// Host: the players meet again (or a story moment calls player 2 back): player 2 comes
    /// back into this game beside player 1 with its health and magic, and a new welcome follows.
    pub fn rejoin_host(&mut self, play: &mut crate::play::Play, why: &str) {
        if !self.apart {
            return;
        }
        self.apart = false;
        self.apart_go = None;
        self.story_base = None;
        self.ops_out.clear();
        play.coop_remote = true;
        // Back as the next player (after those in the host's world).
        self.seat = play.others.len();
        self.peer_name = format!("Player {}", self.seat + 2);
        play.add_player();
        // Back with the hero it left with (the partner came back as Tor when it was not player 2).
        if play.others.get(self.seat).is_some_and(|o| o.who.min(1) != self.apart_who) {
            play.set_player_hero(self.seat, self.apart_who);
        }
        let (hp, mp) = self.other_vitals;
        if let Some(o) = play.others.get_mut(self.seat).filter(|_| hp > 0) {
            (o.hero_hp, o.hero_mp) = (hp.min(o.hero_max), mp.max(0).min(o.hero_mp_max));
        }
        // Met in this world: the player stays where it walked in (not put beside player 1).
        if self.other_pack == play.world.pack && self.other_pos != [0.0; 3] {
            if let Some(o) = play.others.get_mut(self.seat) {
                let f = o.loco.facing;
                o.loco.teleport(self.other_pos, f);
            }
        }
        // A new welcome (the joiner loads this game again), even when the host walked into this
        // world this tick: the walk-through rule would skip the welcome (a following joiner
        // makes the same move), but this joiner is not following (user report 147).
        self.welcome_pack = u32::MAX;
        self.resync = true;
        // The joiner's input from before it went apart is not played again (player 2 ran on
        // into a wall with it); player 2 stands until the joiner's next input, which is taken
        // as the new starting point of its press counters (they kept counting while apart, and
        // read against the old ones they made a Square press that took player 2 back through
        // the barn door it had just left; user report 149).
        self.remote = None;
        self.reseed = true;
        self.notice = Some(format!("{} is back.", if self.peer_name.is_empty() { "Player 2" } else { self.peer_name.as_str() }));
        info!("net: player 2 back ({why}); a new welcome");
    }

    /// Whether the joiner has taken the door it was handed (the host is waiting otherwise).
    pub fn apart_settled(&self) -> bool {
        self.apart && self.apart_go.is_none()
    }

    /// The last event a welcome made now covers: the joiner plays the events after it.
    fn event_base(&self) -> u32 {
        self.events.front().map_or(self.next_event, |e| e.0 - 1)
    }

    /// Host, each tick: every session's welcome (after a world change, a new game or a joiner's
    /// request) and its state, built for its joiner's seat. `walked`: the host walked into the
    /// loaded neighbour (a following joiner makes the same move: no welcome).
    pub fn host_tick(&mut self, play: &mut crate::play::Play, walked: bool) {
        let mut slot: Option<neopets_sim::save::Slot> = None;
        let mut extra = std::mem::take(&mut self.extra);
        for n in std::iter::once(&mut *self).chain(extra.iter_mut()) {
            if n.peer.is_none() || n.apart {
                continue;
            }
            if walked && play.generation == n.welcome_game && play.world.pack != n.welcome_pack && !n.resync {
                n.welcome_pack = play.world.pack;
                info!("net: world {} by walking; the joiner follows without a welcome", play.world.pack);
            }
            if std::mem::take(&mut n.resync) {
                n.welcome_pack = u32::MAX;
            }
            if play.world.pack != n.welcome_pack || play.generation != n.welcome_game {
                let slot = slot.get_or_insert_with(|| {
                    let mut s = neopets_sim::save::Slot::empty();
                    play.write_slot(&mut s, b"NETPLAY");
                    s
                });
                let fx = play.effects.snapshot();
                n.rewelcome(slot, play.world.pack, play.generation, &fx, play.interior_pack());
            }
            // Each joiner's rumble goes to it alone.
            if let Some(s) = play.others.get_mut(n.seat).and_then(|o| o.motor.take()) {
                n.push_own(Event::Rumble(s.level, s.count));
            }
            let started = std::time::Instant::now();
            let s = play.net_state(n.generation, n.seat);
            n.send_state(&s);
            n.cpu_us = n.cpu_us.saturating_add(started.elapsed().as_micros() as u64);
        }
        self.extra = extra;
    }

    /// The Multiplayer page's line: the session's status, and the other joiners' round trips.
    pub fn status_line(&self) -> String {
        let more: Vec<String> = self.extra.iter().filter(|e| e.peer.is_some()).map(|e| format!("{} (round trip about {} ms)", e.peer_name, e.rtt_ms.round())).collect();
        if more.is_empty() { self.status.clone() } else { format!("{} Also in: {}.", self.status, more.join(", ")) }
    }

    pub fn rewelcome(&mut self, slot: &neopets_sim::save::Slot, pack: u32, game: u32, fx: &[neopets_sim::effect::Op], interior: u32) {
        if self.role != Role::Host || self.peer.is_none() || self.apart || (pack == self.welcome_pack && game == self.welcome_game) {
            return;
        }
        self.welcome_pack = pack;
        self.welcome_game = game;
        self.generation += 1;
        self.welcome = Some(welcome(self.generation, slot, fx, self.event_base(), interior));
        self.welcome_mask = 0;
        self.last_sent = None;
        info!("net: world {pack}, welcome {}", self.generation);
    }
}

/// A welcome: the generation, the host's game as a save slot, and its gameplay effects running
/// now (effect operations, applied after the load).
fn welcome(generation: u32, slot: &neopets_sim::save::Slot, fx: &[neopets_sim::effect::Op], base: u32, interior: u32) -> Vec<u8> {
    let host = my_name();
    let mut w = Writer::new(WELCOME);
    w.u32(generation);
    w.bytes(&slot.0);
    w.u16(fx.len().min(512) as u16);
    for op in fx.iter().take(512) {
        write_op(&mut w, op);
    }
    w.u32(base);
    w.text(&host);
    // The interior the host is in (0 outdoors): a slot loads the outdoor world.
    w.u32(interior);
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
    // A welcome while apart: the players meet again in the host's game.
    if net.apart {
        net.apart = false;
        net.story_base = None;
        net.ops_out.clear();
        net.notice = Some("Back with player 1.".into());
        info!("net: back in the host's game");
    }
    net.load = Some(neopets_sim::save::Slot(bytes.to_vec()));
    net.load_fx = match r.u16() {
        Some(n) => (0..n).map_while(|_| read_op(r.u8()?, r)).collect(),
        None => Vec::new(),
    };
    // The events this welcome covers (its game and effects are as after them).
    if let Some(base) = r.u32() {
        if base > net.applied {
            info!("net: events {}..={base} covered by the welcome", net.applied + 1);
            net.applied = base;
        }
    }
    // The host is called Player 1 (its name is no longer sent; the user's choice).
    let _ = r.text();
    net.peer_name = "Player 1".into();
    net.load_interior = r.u32().filter(|&p| p != 0);
    net.loaded = generation;
    net.state = None;
    net.queue.clear();
    net.status = format!("Joined {from}.");
    if first {
        net.notice = Some(if net.peer_name.trim().is_empty() { "Joined the host's game.".into() } else { format!("Joined {}'s game.", net.peer_name) });
    }
    info!("net: welcome {generation} from {from}");
}

/// The port to host on: NEOPETS_NET_PORT, else a line `port=N` in private/net.txt (the older
/// way), else the Multiplayer page's Port row (`setting`; 47817 by default).
pub fn host_port(setting: u16) -> u16 {
    if let Some(p) = std::env::var("NEOPETS_NET_PORT").ok().and_then(|v| v.trim().parse().ok()) {
        return p;
    }
    std::fs::read_to_string("private/net.txt")
        .ok()
        .and_then(|t| t.lines().find_map(|l| l.trim().strip_prefix("port=").and_then(|v| v.trim().parse().ok())))
        .unwrap_or(if setting > 0 { setting } else { PORT })
}

/// NEOPETS_NET_OUTAGE="from:seconds" (diagnostic): nothing sent or received from `from`
/// seconds after this program started, for that long (a cable pulled).
fn outage() -> bool {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    static WINDOW: std::sync::OnceLock<Option<(f32, f32)>> = std::sync::OnceLock::new();
    let start = *START.get_or_init(Instant::now);
    let Some((a, len)) = *WINDOW.get_or_init(|| {
        let v = std::env::var("NEOPETS_NET_OUTAGE").ok()?;
        let (a, b) = v.split_once(':')?;
        Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
    }) else {
        return false;
    };
    let t = start.elapsed().as_secs_f32();
    t >= a && t < a + len
}

/// State `s` carried on `k` ticks by the movement from `p` to `s` (heroes and characters; a
/// step of 2 units or more is a teleport and is not carried).
fn carried(s: &State, p: &State, k: f32) -> State {
    let step = |a: [f32; 3], b: [f32; 3]| {
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < 4.0 { [a[0] + d[0] * k, a[1] + d[1] * k, a[2] + d[2] * k] } else { a }
    };
    let mut out = s.clone();
    for (h, q) in out.heroes.iter_mut().zip(p.heroes.iter()) {
        h.position = step(h.position, q.position);
    }
    for c in out.characters.iter_mut() {
        if let Some(q) = p.characters.iter().find(|q| (q.pack, q.slot, q.ordinal) == (c.pack, c.slot, c.ordinal)) {
            c.position = step(c.position, q.position);
        }
    }
    out
}

/// An apart message from the other side: its world and vitals, its acknowledgement of this
/// side's changes, its next changes in order, and (from the host) the door the joiner takes.
fn read_apart(net: &mut Net, r: &mut Reader, host: bool) {
    let (Some(pack), Some(hp), Some(mp), Some(pos), Some(ack), Some(go)) = (r.u32(), r.i32(), r.i32(), r.v3(), r.u32(), r.u8()) else { return };
    let door = if go == 1 {
        match (r.u16(), r.u32(), r.v3()) {
            (Some(d), Some(p), Some(at)) => Some((d, p, at)),
            _ => return,
        }
    } else {
        None
    };
    while net.ops_out.front().is_some_and(|b| b.0 <= ack) {
        net.ops_out.pop_front();
    }
    let n = r.u8().unwrap_or(0);
    for _ in 0..n {
        let (Some(num), Some(count)) = (r.u32(), r.u16()) else { break };
        let mut ops = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let Some(op) = apart::read_op(r) else { return };
            ops.push(op);
        }
        if num == net.ops_in_last + 1 {
            net.ops_in_last = num;
            net.ops_in.extend(ops);
        }
    }
    if pack != net.other_pack {
        info!("net: the other side is now in world {pack}");
    }
    net.other_pack = pack;
    net.other_vitals = (hp, mp);
    net.other_pos = pos;
    // A world (not 0, which is a load still running) other than the one apart began in.
    if host && pack != 0 && net.apart_go.is_none() && Some(pack) != net.apart_from {
        net.other_left = true;
    }
    if host {
        // The joiner answered: it has taken its door.
        if net.apart_go.take().is_some() {
            info!("net: player 2 is in its own world ({pack})");
        }
    } else if let Some((d, p, at)) = door {
        if !net.apart {
            net.apart = true;
            net.apart_start = Some((d, at));
            net.other_pack = p;
            net.notice = Some("You went off on your own. Walk into player 1's world to meet again.".into());
            info!("net: taking door {d} of {p} alone; apart");
        }
    }
}

/// Host: the joiner's player's health and magic kept as it goes (`Net::kept`).
fn keep_p2(net: &mut Net, play: &crate::play::Play) {
    if let Some(o) = play.others.get(net.seat) {
        net.kept = Some((play.generation, o.hero_hp, o.hero_mp));
    }
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
#[allow(clippy::too_many_arguments)]
pub fn net_step(mut net: ResMut<Net>, mut play: Option<ResMut<PlayState>>, mut archive: Option<ResMut<crate::Archive>>, mut coop: ResMut<crate::coop::Coop>, menu: Option<Res<crate::menu_view::Menu>>, password: Res<NetPassword>, settings: Res<crate::display::DisplaySettings>) {
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
    // Joining again after the host was lost (Default: for up to a minute; the Join row stops it).
    let retrying = net.retry.take();
    if let Some(a) = retrying.as_ref().filter(|_| net.role == Role::Off && net.request.is_none()) {
        net.request = Some(Request::Join(a.clone()));
    }
    match net.request.take() {
        Some(Request::Host) => {
            net.stop("");
            let port = host_port(settings.net_port);
            match UdpSocket::bind(("0.0.0.0", port)).and_then(|s| s.set_nonblocking(true).map(|_| s)) {
                Ok(s) => {
                    net.socket = Some(s);
                    net.role = Role::Host;
                    net.host_id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64) ^ u64::from(port);
                    // Another port than the usual one goes in the address the friend types.
                    let shown = |ip: std::net::IpAddr| if port == PORT { ip.to_string() } else { format!("{ip}:{port}") };
                    net.status = match local_address() {
                        Some(ip) => format!("Hosting. Your friend joins with {}.", shown(ip)),
                        None => format!("Hosting on port {port}."),
                    };
                    net.host_line = net.status.clone();
                    // The Multiplayer page's UPnP row (or NEOPETS_UPNP, diagnostic): ask the
                    // router to forward the port, on its own thread.
                    if settings.upnp || std::env::var_os("NEOPETS_UPNP").is_some() {
                        if let Some(ip) = local_address() {
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                let _ = tx.send((port, upnp::open(port, ip)));
                            });
                            net.upnp_rx = Some(std::sync::Mutex::new(rx));
                            net.status = format!("{} Asking the router to open the port...", net.host_line);
                            info!("net: asking the router to forward port {port}");
                        }
                    }
                    info!("net: hosting on port {port}");
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    net.status = format!("Could not host: port {port} is in use (is another copy of the game hosting on this computer?). Put port=N in private/net.txt to use another port.");
                    info!("net: could not host: port {port} in use");
                }
                Err(e) => net.status = format!("Could not host: {e}"),
            }
        }
        Some(Request::Join(text)) => {
            net.stop("");
            // An address with no port joins on the Port row's (47817 by default).
            let port = host_port(settings.net_port);
            let addr = if text.contains(':') { text.parse::<SocketAddr>().ok() } else { format!("{text}:{port}").parse::<SocketAddr>().ok() };
            match (addr, UdpSocket::bind("0.0.0.0:0").and_then(|s| s.set_nonblocking(true).map(|_| s))) {
                (Some(a), Ok(s)) => {
                    net.socket = Some(s);
                    net.role = Role::Join;
                    net.target = Some(a);
                    net.join_text = text.clone();
                    net.join_started = Some(Instant::now());
                    net.status = format!("Joining {a}...");
                    info!("net: joining {a}");
                }
                (None, _) => {
                    net.status = format!("\"{text}\" is not an address. Type it like 192.168.1.20 (or 192.168.1.20:47818 for another port).");
                    net.notice = Some(net.status.clone());
                }
                (_, Err(e)) => net.status = format!("Could not join: {e}"),
            }
        }
        Some(Request::Remove) => {
            if net.role == Role::Host && net.any_peer() {
                net.remove_wanted = true;
            }
        }
        Some(Request::Leave) => {
            // (The router's forward ends with the hosting: `stop`.)
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
    if retrying.is_some() && net.role == Role::Join {
        net.retry_until = Some(Instant::now() + Duration::from_secs(60));
        net.status = "Lost the host. Joining again (the Join row stops)...".into();
        info!("net: joining the host again");
    }
    // The game finder: while the Multiplayer page's Online tab shows and this computer neither
    // hosts nor joins (NEOPETS_NET_FIND=1, diagnostic: always, with each game heard printed).
    let online_tab = menu.as_ref().is_some_and(|m| m.flow.current == crate::menus::DISPLAY && m.flow.display_page == 2 && m.flow.display_tab == 1);
    let find = net.role == Role::Off && (online_tab || std::env::var_os("NEOPETS_NET_FIND").is_some());
    net.finder.step(find, host_port(settings.net_port));
    if net.role == Role::Off {
        return;
    }
    // The router's answer (UPnP).
    if let Some((port, r)) = net.upnp_rx.as_ref().and_then(|rx| rx.lock().ok()?.try_recv().ok()) {
        net.upnp_rx = None;
        match r {
            upnp::Outcome::Opened(ip) => {
                net.upnp_port = Some(port);
                let shown = ip.map_or_else(|| "your internet address".to_string(), |ip| if port == PORT { ip } else { format!("{ip}:{port}") });
                net.host_line = format!("{} A friend on the internet joins with {shown}.", net.host_line);
                info!("net: the router forwards port {port}");
            }
            upnp::Outcome::Failed(e) => {
                net.host_line = format!("{} The router could not open the port: {e}.", net.host_line);
                info!("net: the router could not forward port {port}: {e}");
            }
            upnp::Outcome::Closed => {}
        }
        net.status = net.host_line.clone();
    }
    let now = Instant::now();
    // The messages waiting. NEOPETS_NET_DROP=percent and NEOPETS_NET_DELAY=ms (diagnostic): a
    // lossy, slow network, made here by dropping that share of received messages and holding
    // the rest that long.
    let mut buf = vec![0u8; 65536];
    let drop = std::env::var("NEOPETS_NET_DROP").ok().and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or(0);
    let delay = Duration::from_millis(std::env::var("NEOPETS_NET_DELAY").ok().and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0));
    let jitter_ms = std::env::var("NEOPETS_NET_JITTER").ok().and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0);
    while let Some(s) = net.socket.as_ref() {
        let Ok((n, from)) = s.recv_from(&mut buf) else { break };
        net.bytes_in += n as u64 + 28;
        if outage() {
            continue;
        }
        if drop > 0 {
            net.lossy = net.lossy.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            if ((net.lossy >> 33) % 100) < drop as u64 {
                continue;
            }
        }
        // NEOPETS_NET_JITTER=ms (diagnostic): each message held a further 0 to ms, at random
        // (messages may then arrive out of order, as on the internet).
        let extra = if jitter_ms > 0 {
            net.lossy = net.lossy.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            Duration::from_millis((net.lossy >> 33) % (jitter_ms + 1))
        } else {
            Duration::ZERO
        };
        net.held.push_back((now + delay + extra, buf[..n].to_vec(), from));
    }
    let mut ready = Vec::new();
    // Every held message now due, in the order they fall due.
    let mut due: Vec<(Instant, Vec<u8>, SocketAddr)> = Vec::new();
    net.held.retain(|m| {
        if m.0 <= now {
            due.push(m.clone());
            false
        } else {
            true
        }
    });
    due.sort_by_key(|m| m.0);
    ready.extend(due.into_iter().map(|(_, b, from)| (b, from)));
    if net.role != Role::Host {
        guarded_step(&mut net, ready, now, &mut play, &mut coop, &menu, &password, &settings, 0);
        return;
    }
    host_route(&mut net, ready, now, &mut play, &mut coop, &menu, &password, &settings);
}

/// Host: each message to its joiner's session (a newcomer's hello to a free one: a new session
/// while fewer than three joiners are in), then each session's step; a seat freed moves the
/// later joiners up one (player 4 becomes player 3).
#[allow(clippy::too_many_arguments)]
fn host_route(net: &mut Net, ready: Vec<(Vec<u8>, SocketAddr)>, now: Instant, play: &mut Option<ResMut<PlayState>>, coop: &mut crate::coop::Coop, menu: &Option<Res<crate::menu_view::Menu>>, password: &NetPassword, settings: &crate::display::DisplaySettings) {
    let mut extra = std::mem::take(&mut net.extra);
    let mut inbox: Vec<Vec<(Vec<u8>, SocketAddr)>> = vec![Vec::new(); 1 + extra.len()];
    for (bytes, from) in ready {
        // The game finder's question from this home network: this game's version, players,
        // password, port and whether it is in a game (an internet stranger is not answered).
        if bytes.len() >= 7 && bytes[..4] == MAGIC && bytes[6] == FIND {
            if upnp::on_this_network(from.ip()) {
                let players = 1 + net.peer.is_some() as u8 + extra.iter().filter(|e| e.peer.is_some()).count() as u8;
                let mut w = Writer::new(HERE);
                w.text(&build_id());
                w.u8(players);
                w.u8(crate::coop::MAX_PLAYERS as u8);
                w.u8(!password.0.is_empty() as u8);
                w.u16(net.socket.as_ref().and_then(|s| s.local_addr().ok()).map_or(PORT, |a| a.port()));
                w.u8(play.as_ref().is_some_and(|p| p.play.spawned) as u8);
                w.u64(net.host_id);
                let b = w.0;
                net.answer_stranger(from, &b);
            }
            continue;
        }
        let known = if net.peer == Some(from) { Some(0) } else { extra.iter().position(|e| e.peer == Some(from)).map(|k| k + 1) };
        let k = match known {
            Some(k) => k,
            None => {
                let hello = bytes.len() >= 7 && bytes[..4] == MAGIC && bytes[6] == HELLO && u16::from_le_bytes([bytes[4], bytes[5]]) == PROTOCOL;
                if !hello || net.peer.is_none() {
                    0
                } else if let Some(k) = extra.iter().position(|e| e.peer.is_none()) {
                    k + 1
                } else if extra.len() + 1 < MAX_JOINERS {
                    extra.push(Net { role: Role::Host, socket: net.socket.as_ref().and_then(|s| s.try_clone().ok()), disc: net.disc, host_line: net.host_line.clone(), ..Default::default() });
                    inbox.push(Vec::new());
                    extra.len()
                } else {
                    // Full: the first session refuses it.
                    0
                }
            }
        };
        inbox[k].push((bytes, from));
    }
    // The Remove row: the last joiner goes.
    if std::mem::take(&mut net.remove_wanted) {
        let last = std::iter::once(&*net).chain(extra.iter()).enumerate().filter(|(_, e)| e.peer.is_some()).max_by_key(|(_, e)| e.seat).map(|(k, _)| k);
        if let Some(k) = last {
            let n = if k == 0 { &mut *net } else { &mut extra[k - 1] };
            // Sent three times (no acknowledgement; the joiner's timeout covers a loss).
            let w = Writer::new(REMOVED);
            for _ in 0..3 {
                n.send(&w.0);
            }
            let (seat, apart) = (n.seat, n.apart);
            if let Some(p) = play.as_deref_mut().filter(|_| !apart) {
                keep_p2(n, &p.play);
                p.play.remove_seat(seat);
            }
            let name = n.peer_name.clone();
            n.peer = None;
            n.remote = None;
            n.welcome = None;
            n.last_heard = None;
            n.apart = false;
            net.status = format!("{name} removed. Still hosting.");
            net.notice = Some(format!("{name} was removed."));
            info!("net: {name} removed by the host");
            if !apart {
                renumber(net, &mut extra, Some(seat));
            }
        }
    }
    let linked = |n: &Net| n.peer.is_some() as usize;
    let others = extra.iter().map(linked).sum();
    let mine = std::mem::take(&mut inbox[0]);
    let freed = guarded_step(net, mine, now, play, coop, menu, password, settings, others);
    renumber(net, &mut extra, freed);
    for k in 0..extra.len() {
        let others = linked(net) + extra.iter().enumerate().filter(|(j, _)| *j != k).map(|(_, e)| linked(e)).sum::<usize>();
        let mine = std::mem::take(&mut inbox[k + 1]);
        let freed = guarded_step(&mut extra[k], mine, now, play, coop, menu, password, settings, others);
        renumber(net, &mut extra, freed);
        if let Some(n) = extra[k].notice.take() {
            net.notice = Some(match net.notice.take() {
                Some(o) => format!("{o} {n}"),
                None => n,
            });
        }
    }
    // Sessions whose joiner is gone close (their socket is a copy of this one's), and the seats
    // past the last joiner have no input.
    extra.retain(|e| e.peer.is_some());
    net.extra = extra;
    let seats = net.with_host();
    for c in coop.remote.iter_mut().skip(seats) {
        *c = None;
    }
}

/// Host: seat `freed` is empty now (its player left the game): the joiners after it move up one.
fn renumber(net: &mut Net, extra: &mut [Net], freed: Option<usize>) {
    let Some(k) = freed else { return };
    for n in std::iter::once(net).chain(extra.iter_mut()) {
        if n.peer.is_some() && !n.apart && n.seat > k {
            n.seat -= 1;
            n.peer_name = format!("Player {}", n.seat + 2);
        }
    }
}

/// A session's step, with a safety net: should a message from the other side make it fail (a
/// damaged or hostile message this game's checks missed), the game goes on. The host sends that
/// joiner away (its seat freed); a joiner leaves the host's game.
#[allow(clippy::too_many_arguments)]
fn guarded_step(net: &mut Net, ready: Vec<(Vec<u8>, SocketAddr)>, now: Instant, play: &mut Option<ResMut<PlayState>>, coop: &mut crate::coop::Coop, menu: &Option<Res<crate::menu_view::Menu>>, password: &NetPassword, settings: &crate::display::DisplaySettings, others: usize) -> Option<usize> {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| session_step(net, ready, now, play, coop, menu, password, settings, others)));
    match r {
        Ok(freed) => freed,
        Err(_) if net.role == Role::Join => {
            net.drop_host();
            None
        }
        Err(_) => {
            let name = if net.peer_name.is_empty() { "A player".to_string() } else { net.peer_name.clone() };
            warn!("net: {name} sent something this game could not use; sent away");
            let w = Writer::new(REMOVED);
            net.send(&w.0);
            let (seat, apart) = (net.seat, net.apart);
            net.peer = None;
            net.remote = None;
            net.welcome = None;
            net.last_heard = None;
            net.apart = false;
            net.notice = Some(format!("{name} was sent away (a message this game could not use)."));
            if apart || net.seat == usize::MAX {
                return None;
            }
            let p = play.as_deref_mut()?;
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.play.remove_seat(seat))).ok()?;
            Some(seat)
        }
    }
}

/// One session's step: its messages, its sending and its timeouts. Returns the seat freed when
/// a host's joiner left. `others`: how many other joiners are in (host).
#[allow(clippy::too_many_arguments)]
fn session_step(net: &mut Net, ready: Vec<(Vec<u8>, SocketAddr)>, now: Instant, play: &mut Option<ResMut<PlayState>>, coop: &mut crate::coop::Coop, menu: &Option<Res<crate::menu_view::Menu>>, password: &NetPassword, settings: &crate::display::DisplaySettings, others: usize) -> Option<usize> {
    let mut freed = None;
    for (bytes, from) in ready {
        // Another version of this game (its messages cannot be read here): a joiner is told; a
        // host answers a hello with a refusal the joiner can tell apart by its version.
        if bytes.len() >= 7 && bytes[..4] == MAGIC && u16::from_le_bytes([bytes[4], bytes[5]]) != PROTOCOL {
            if net.role == Role::Join && net.target == Some(from) {
                info!("net: the host runs another version of the game");
                let line = "Could not join: your game is a different version from the host's. Use the same version on both computers.";
                net.notice = Some(line.into());
                net.stop(line);
            } else if net.role == Role::Host && bytes[6] == HELLO {
                net.refuse(from, "your game is a different version from the host's. Use the same version on both computers");
            }
            continue;
        }
        let Some((kind, mut r)) = Reader::open(&bytes) else { continue };
        match (net.role, kind) {
            (Role::Host, HELLO) => {
                let (build, disc) = (r.text(), r.u64());
                let (name, pass) = (r.text().unwrap_or_default(), r.u64().unwrap_or(0));
                let partner = r.u8().map(|v| v != 0);
                // Each reason as the joiner reads it ("Could not join: ..."), and the host's own
                // line for the ones the host can help with.
                let (refuse, note) = if build.as_deref() != Some(build_id().as_str()) {
                    (Some(format!("your game is a different version from the host's ({} there, {} here). Use the same version on both computers", build.unwrap_or_default(), build_id())), Some("Someone could not join: their game is a different version."))
                } else if pass != password_hash(&password.0) {
                    if pass == 0 {
                        (Some("the host's game has a password. Type it in Password on the Multiplayer page (Online tab), then join again".to_string()), Some("Someone tried to join without the password."))
                    } else {
                        (Some("the password is wrong. Check it with the host, type it in Password on the Multiplayer page, then join again".to_string()), Some("Someone tried to join with a wrong password."))
                    }
                } else if disc != net.disc {
                    (Some("your disc image is not the same as the host's (both need the same game disc)".to_string()), Some("Someone could not join: their disc image is different."))
                } else if net.peer.is_some_and(|p| p != from) {
                    (Some("the host's game is full (4 players)".to_string()), None)
                } else if play.as_ref().is_none_or(|p| !p.play.spawned) {
                    (Some("the host is not playing yet. Ask them to start or load a game, then join again".to_string()), Some("Someone is trying to join: start or load a game to let them in."))
                } else {
                    (None, None)
                };
                if let Some(why) = refuse {
                    net.refuse(from, &why);
                    if let Some(n) = note {
                        net.note_refusal(n);
                    }
                    continue;
                }
                let Some(p) = play.as_deref_mut() else { continue };
                if net.peer.is_none() {
                    net.reset_peer();
                    net.peer = Some(from);
                    net.last_heard = Some(now);
                    // Player 2 is the joiner's (a local second controller lets go of it), the hero
                    // the joiner chose on its Multiplayer page.
                    // The first joiner is player 2 (local players beyond player 2 go; player 2's
                    // hero is the one the joiner chose); later ones are players 3 and 4.
                    if others == 0 {
                        p.play.coop_partner_remote = partner;
                        p.play.keep_players(1);
                        net.seat = 0;
                    } else {
                        net.seat = p.play.others.len();
                    }
                    p.play.coop_remote = true;
                    if p.play.others.len() <= net.seat {
                        p.play.add_player();
                    }
                    info!("net: player {} is hero {:?}", net.seat + 2, p.play.others.get(net.seat).map(|o| o.who));
                    // The same game's player back: its health and magic as it left them.
                    if let (Some((g, hp, mp)), Some(o)) = (net.kept, p.play.others.get_mut(net.seat)) {
                        if g == p.play.generation && hp > 0 {
                            (o.hero_hp, o.hero_mp) = (hp.min(o.hero_max), mp.min(o.hero_mp_max));
                            info!("net: player 2 back with {hp} health and {mp} magic");
                        }
                    }
                    coop.release_local();
                    net.generation += 1;
                    net.welcome_pack = p.play.world.pack;
                    net.welcome_game = p.play.generation;
                    let mut slot = neopets_sim::save::Slot::empty();
                    p.play.write_slot(&mut slot, b"NETPLAY");
                    let base = net.event_base();
                    net.welcome = Some(welcome(net.generation, &slot, &p.play.effects.snapshot(), base, p.play.interior_pack()));
                    net.welcome_mask = 0;
                    // Players are called by number (Player 2), not by the computer's name.
                    let _ = name;
                    net.peer_name = format!("Player {}", net.seat + 2);
                    net.status = format!("{} joined from {from}.", net.peer_name);
                    net.notice = Some(format!("{} joined.", net.peer_name));
                    info!("net: {from} joined ({})", net.peer_name);
                }
            }
            (Role::Host, INPUT) if net.peer == Some(from) => {
                net.last_heard = Some(now);
                if let Some(i) = Input::decode(&mut r) {
                    if i.generation == net.generation {
                        net.welcome = None;
                    }
                    // The round trip: the state tick the joiner last saw, against when it went.
                    if i.seen > net.last_seen {
                        net.last_seen = i.seen;
                        if let Some(&(_, t)) = net.sent_at.iter().find(|e| e.0 == i.seen) {
                            let ms = now.duration_since(t).as_secs_f32() * 1000.0;
                            net.rtt_ms = if net.rtt_ms == 0.0 { ms } else { net.rtt_ms * 0.9 + ms * 0.1 };
                        }
                    }
                    // The joiner's item-screen command, once: run as player 2's item screen.
                    if i.cmd.0 == net.cmd_done.wrapping_add(1) {
                        net.cmd_done = i.cmd.0;
                        if let (Some(c), Some(p)) = (command_of(i.cmd.1, i.cmd.2, i.cmd.3), play.as_deref_mut()) {
                            let was = std::mem::replace(&mut p.play.menu_player, net.seat + 1);
                            p.play.as_menu_player(|q| q.item_command(&c));
                            p.play.menu_player = was;
                            info!("net: player {}'s item screen: {c:?}", net.seat + 2);
                        }
                    }
                    if std::mem::take(&mut net.reseed) {
                        net.seen = i;
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
                    if host && !net.apart {
                        keep_p2(net, &p.play);
                        p.play.remove_seat(net.seat);
                        freed = Some(net.seat);
                    } else if !host {
                        p.play.remove_player();
                        p.play.net_follow = false;
                    }
                }
                if host {
                    // The host keeps hosting for the next joiner.
                    net.peer = None;
                    net.remote = None;
                    net.welcome = None;
                    // No timeout for a joiner that said goodbye.
                    net.last_heard = None;
                    net.status = format!("{} left. Still hosting.", net.peer_name);
                    net.notice = Some(format!("{} left.", if net.peer_name.is_empty() { "Player 2" } else { net.peer_name.as_str() }));
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
            (Role::Host, APART) if net.peer == Some(from) => {
                net.last_heard = Some(now);
                read_apart(net, &mut r, true);
            }
            (Role::Join, APART) if net.target == Some(from) => {
                net.last_heard = Some(now);
                read_apart(net, &mut r, false);
            }
            (Role::Join, REMOVED) if net.target == Some(from) => {
                if let Some(p) = play.as_deref_mut() {
                    p.play.remove_player();
                    p.play.net_follow = false;
                }
                if net.loaded > 0 {
                    net.ended = true;
                }
                net.notice = Some("The host removed you from the game.".into());
                info!("net: removed by the host");
                net.stop("The host removed you from the game.");
            }
            (_, PING) if net.peer == Some(from) || net.target == Some(from) => {
                net.last_heard = Some(now);
                let page = r.u8().unwrap_or(0);
                let paused = page != 0;
                if net.peer_page != page {
                    info!("net: the other side's page is now {page} (ping)");
                }
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
                take_welcome(net, &mut r, from);
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
                                take_welcome(net, &mut wr, from);
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
                let line = format!("Could not join: {why}.");
                net.notice = Some(line.clone());
                net.stop(&line);
            }
            (Role::Join, STATE) if net.target == Some(from) => {
                net.last_heard = Some(now);
                if let Some(t) = net.last_state_at {
                    net.gap_max = net.gap_max.max(now.duration_since(t).as_millis() as u32);
                }
                net.last_state_at = Some(now);
                net.gap_states += 1;
                if let Some(mut s) = State::decode(&mut r) {
                    // A running host is not paused (its pings say so only while quiet); its
                    // game-over page comes with each state.
                    if net.peer_page != s.page {
                        info!("net: the host's page is now {} (state)", s.page);
                    }
                    net.peer_paused = s.page != 0;
                    net.peer_page = s.page;
                    net.seen_tick = s.tick as u32;
                    // The host carried out the item-screen command sent: the next one goes.
                    if !net.cmds.is_empty() && s.cmd_ack == net.cmds_done.wrapping_add(1) {
                        net.cmds.pop_front();
                        net.cmds_done = net.cmds_done.wrapping_add(1);
                    }
                    net.rtt_ms = s.rtt as f32;
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
                    if s.generation > net.acc_gen {
                        net.acc_gen = s.generation;
                        net.chars_acc.clear();
                        net.moving_acc.clear();
                    }
                    // Into the jitter buffer, in order: an older or repeated state (one that
                    // arrived late, out of order) is dropped before its characters go into the
                    // running copies. A newer world (a later welcome) restarts the host's ticks.
                    let newest = net.queue.back().or(net.state.as_ref()).map(|o| (o.generation, o.tick));
                    if newest.is_none_or(|n| (s.generation, s.tick) > n) {
                        for c in &s.characters {
                            net.chars_acc.insert((c.pack, c.slot, c.ordinal), *c);
                        }
                        for m in &s.moving {
                            net.moving_acc.insert((m.0, m.1), *m);
                        }
                        s.characters = net.chars_acc.values().copied().collect();
                        s.moving = net.moving_acc.values().copied().collect();
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
        w.text(&my_name());
        w.u64(password_hash(&password.0));
        // This side's Player 2 choice: player 2 is this side's player.
        // NEOPETS_NET_PARTNER=0/1 (diagnostic) in place of the setting (two windows share it).
        let partner = std::env::var("NEOPETS_NET_PARTNER").ok().map_or(settings.coop_partner, |v| v.trim() == "1");
        w.u8(partner as u8);
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
    // Apart (net/apart.rs): every 100 ms this side's world, vitals and unacknowledged story
    // changes (and, from the host until the joiner answers, the door to take).
    let linked_apart = net.apart && ((net.role == Role::Host && net.peer.is_some()) || (net.role == Role::Join && net.target.is_some()));
    if linked_apart && net.apart_sent.is_none_or(|t| now.duration_since(t) >= Duration::from_millis(100)) {
        net.apart_sent = Some(now);
        let mut w = Writer::new(APART);
        w.u32(net.apart_me.0);
        w.i32(net.apart_me.1);
        w.i32(net.apart_me.2);
        w.v3(net.apart_me.3);
        w.u32(net.ops_in_last);
        match net.apart_go {
            Some((door, pack, at)) => {
                w.u8(1);
                w.u16(door);
                w.u32(pack);
                w.v3(at);
            }
            None => w.u8(0),
        }
        let mut batches = Vec::new();
        let mut size = 0;
        for (num, ops) in net.ops_out.iter() {
            let mut b = Writer(Vec::new());
            b.u32(*num);
            b.u16(ops.len() as u16);
            for op in ops {
                apart::write_op(&mut b, op);
            }
            if size + b.0.len() > 900 && !batches.is_empty() {
                break;
            }
            size += b.0.len();
            batches.push(b.0);
        }
        w.u8(batches.len() as u8);
        for b in batches {
            w.0.extend_from_slice(&b);
        }
        let bytes = w.0;
        net.send(&bytes);
    }
    // Once a second while linked: the traffic each way, and the Multiplayer page's line with the
    // round trip and the traffic.
    let linked_now = (net.role == Role::Host && net.peer.is_some()) || net.following();
    if linked_now && net.rate_at.is_none_or(|r| now.duration_since(r.0) >= Duration::from_secs(1)) {
        if let Some((t, o, i)) = net.rate_at {
            let secs = now.duration_since(t).as_secs_f32().max(0.001);
            net.rate = ((net.bytes_out - o) as f32 / secs, (net.bytes_in - i) as f32 / secs);
            let (out, inn) = (net.rate.0 / 1024.0, net.rate.1 / 1024.0);
            let rtt = net.rtt_ms.round();
            net.status = if net.role == Role::Host {
                format!("{} {} is in: round trip about {rtt} ms; sending {out:.0} KB/s, receiving {inn:.0} KB/s.", net.host_line, net.peer_name)
            } else {
                format!("Joined. Round trip about {rtt} ms; receiving {inn:.0} KB/s, sending {out:.0} KB/s.")
            };
        }
        net.rate_at = Some((now, net.bytes_out, net.bytes_in));
    } else if !linked_now {
        net.rate_at = None;
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
    // The host's input for the joiner's player from its controller (its presses since the last
    // frame).
    // The latest input stays for every tick until a newer one comes (a frame that ran two ticks
    // gave the second none, and the player stood still for it: with four windows on one
    // computer the joiners' players stuttered), and a press stays until a tick takes it (a
    // frame that ran no tick lost the press to the next frame's input).
    if net.role == Role::Host && net.peer.is_some() && !net.apart && net.seat < coop.remote.len() {
        let before = coop.remote[net.seat].map(|(h, _)| (h.jump, h.attack, h.square));
        coop.remote[net.seat] = net.remote.map(|i| {
            let (mut h, c) = i.to_hero(&net.seen);
            // NEOPETS_TRACE_NETINPUT (diagnostic): each jump press as it reaches the host.
            if h.jump && std::env::var_os("NEOPETS_TRACE_NETINPUT").is_some() {
                info!("net: jump {} arrived", i.jumps);
            }
            if let Some((j, a, q)) = before {
                (h.jump, h.attack, h.square) = (h.jump || j, h.attack || a, h.square || q);
            }
            (h, c)
        });
        if let Some(i) = net.remote {
            let pl = i.to_panel(&net.seen);
            let q = &mut coop.seats[net.seat].panel;
            (q.up, q.left, q.right, q.down, q.triangle) = (q.up | pl.up, q.left | pl.left, q.right | pl.right, q.down | pl.down, q.triangle | pl.triangle);
            net.seen = i;
        }
    }
    // Timeouts.
    if net.last_heard.is_some_and(|t| now.duration_since(t) > TIMEOUT) {
        if let Some(p) = play.as_deref_mut() {
            if net.role == Role::Host && !net.apart {
                keep_p2(net, &p.play);
                p.play.remove_seat(net.seat);
                freed = Some(net.seat);
            } else if net.role != Role::Host {
                p.play.remove_player();
                p.play.net_follow = false;
            }
        }
        if net.role == Role::Host {
            net.peer = None;
            net.remote = None;
            net.welcome = None;
            net.last_heard = None;
            net.status = format!("Lost {}. Still hosting.", net.peer_name);
            net.notice = Some(format!("Lost {}.", net.peer_name));
            info!("net: the joiner timed out");
        } else {
            info!("net: the host timed out");
            if net.loaded > 0 {
                net.ended = true;
                net.notice = Some("Lost the host. Joining again...".into());
            }
            let again = std::mem::take(&mut net.join_text);
            net.stop("Lost the host.");
            if !again.is_empty() {
                net.retry = Some(again);
            }
        }
    }
    // A first join with no answer at all for 20 seconds stops (before, it waited for ever).
    if net.role == Role::Join && net.loaded == 0 && net.last_heard.is_none() && net.retry_until.is_none() && net.join_started.is_some_and(|t| now.duration_since(t) > Duration::from_secs(20)) {
        info!("net: no answer from the host");
        let at = net.target.map_or(String::new(), |a| if a.port() == PORT { a.ip().to_string() } else { a.to_string() });
        let line = format!("Could not join {at}: no answer. Check the address, that the host chose Host, and that the host's firewall lets the game in.");
        net.notice = Some(line.clone());
        net.stop(&line);
    }
    // A join again that has not reached the host within its minute stops.
    if net.role == Role::Join && net.loaded == 0 && net.retry_until.is_some_and(|t| now > t) {
        info!("net: the host did not come back");
        net.notice = Some("Could not reach the host again. You can join again from the Multiplayer page.".into());
        net.stop("Could not reach the host again.");
    } else if net.role == Role::Join && net.loaded == 0 && net.last_heard.is_none() {
        // Still waiting for an answer.
    }
    freed
}

/// This computer's player name as sent: none (players are called Player 1, Player 2 and so on
/// by their place, not by the computer's name; the user's choice).
pub fn my_name() -> String {
    String::new()
}

/// A password's hash as it travels (0 for none).
pub fn password_hash(p: &str) -> u64 {
    if p.is_empty() {
        return 0;
    }
    // FNV-1a, so both computers get the same number for the same password.
    p.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

/// The Multiplayer page's password (Default: none): the host's for the game it hosts, the
/// joiner's for the game it joins. Typed with letters and digits; kept in private/password.txt.
#[derive(Resource)]
pub struct NetPassword(pub String);

impl Default for NetPassword {
    fn default() -> Self {
        // NEOPETS_NET_PASSWORD (diagnostic) in place of the file, so two windows on one computer
        // can differ.
        if let Ok(p) = std::env::var("NEOPETS_NET_PASSWORD") {
            return Self(p);
        }
        Self(std::fs::read_to_string("private/password.txt").map(|t| t.trim().to_string()).unwrap_or_default())
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
pub fn type_address(menu: Option<Res<crate::menu_view::Menu>>, keys: Res<ButtonInput<KeyCode>>, mut address: ResMut<JoinAddress>, mut password: ResMut<NetPassword>, mut settings: ResMut<crate::display::DisplaySettings>) {
    let Some(menu) = menu else { return };
    if menu.flow.current != crate::menus::DISPLAY || menu.flow.display_page != 2 {
        return;
    }
    // The password row: letters and digits, Backspace deletes (kept for next time).
    if menu.flow.display_focus == crate::display::PASSWORD_ROW {
        let before = password.0.clone();
        for k in keys.get_just_pressed() {
            let c = match k {
                KeyCode::Digit0 | KeyCode::Numpad0 => Some('0'),
                KeyCode::Backspace => {
                    password.0.pop();
                    None
                }
                _ => {
                    let name = format!("{k:?}");
                    name.strip_prefix("Key").or_else(|| name.strip_prefix("Digit")).or_else(|| name.strip_prefix("Numpad")).filter(|r| r.len() == 1).and_then(|r| r.chars().next()).map(|c| c.to_ascii_lowercase())
                }
            };
            if let Some(c) = c.filter(|_| password.0.len() < 20) {
                password.0.push(c);
            }
        }
        if password.0 != before {
            let _ = std::fs::write("private/password.txt", &password.0);
        }
        return;
    }
    // The port row: digits type (at most 65535), Backspace deletes the last; an empty or
    // too small port is kept as typed while typing and taken as the default when used.
    if menu.flow.display_focus == crate::display::PORT_ROW {
        let mut p = settings.net_port as u32;
        for (d, (a, b)) in [(KeyCode::Digit0, KeyCode::Numpad0), (KeyCode::Digit1, KeyCode::Numpad1), (KeyCode::Digit2, KeyCode::Numpad2), (KeyCode::Digit3, KeyCode::Numpad3), (KeyCode::Digit4, KeyCode::Numpad4), (KeyCode::Digit5, KeyCode::Numpad5), (KeyCode::Digit6, KeyCode::Numpad6), (KeyCode::Digit7, KeyCode::Numpad7), (KeyCode::Digit8, KeyCode::Numpad8), (KeyCode::Digit9, KeyCode::Numpad9)].into_iter().enumerate() {
            if (keys.just_pressed(a) || keys.just_pressed(b)) && p * 10 + d as u32 <= 65535 {
                p = p * 10 + d as u32;
            }
        }
        if keys.just_pressed(KeyCode::Backspace) {
            p /= 10;
        }
        if p != settings.net_port as u32 {
            settings.net_port = p as u16;
            settings.save();
        }
        return;
    }
    if menu.flow.display_focus != crate::display::ADDRESS_ROW {
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
            page: 2,
            rtt: 31,
            cmd_ack: 3,
            input_seq: 9,
            input_extra: 2,
            pet: Some((450, 30, 40)),
            equip: ([[Some(1), None], [None, Some(2)], [None, None]], [[Some(160), None, Some(161)], [None; 3]], true),
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
                (13, Event::Notice(NOTICE_SAVED)),
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
            for kind in [HELLO, WELCOME, REFUSE, INPUT, STATE, BYE, PING, WELCOME_PART, PART_ACK, BLOB_PART, EVENTS, RESYNC, REMOVED, APART] {
                b[6] = kind;
                if let Some((_, mut r)) = Reader::open(&b) {
                    let _ = State::decode(&mut r);
                }
                if let Some((_, mut r)) = Reader::open(&b) {
                    let _ = Input::decode(&mut r);
                }
                // The newer messages' records: an events list, and the world sync of a blob.
                if let Some((_, mut r)) = Reader::open(&b) {
                    while Event::read(&mut r).is_some() {}
                }
                if let Some((_, mut r)) = Reader::open(&b) {
                    let _ = WorldSync::read(&mut r);
                }
                for host in [true, false] {
                    if let Some((_, mut r)) = Reader::open(&b) {
                        read_apart(&mut Net::default(), &mut r, host);
                    }
                }
            }
        }
        // Every cut of a world sync is refused too.
        let ws = WorldSync { pack: 909, breakables: vec![1, 2], items: vec![(909, 3, NetItem { id: 5, ..Default::default() })], chars: vec![(909, 1, 0)], objects: vec![(4, 2, 1.5, 3)] };
        let mut w = Writer::new(BLOB_PART);
        ws.write(&mut w);
        for cut in 7..w.0.len() {
            if let Some((_, mut r)) = Reader::open(&w.0[..cut]) {
                assert!(WorldSync::read(&mut r).is_none(), "a world sync cut at {cut} decoded");
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

    /// No password travels as 0; the same password gives the same number on both computers,
    /// another password another number.
    #[test]
    fn password_hashes() {
        assert_eq!(password_hash(""), 0);
        assert_eq!(password_hash("apple7"), password_hash("apple7"));
        assert_ne!(password_hash("apple7"), password_hash("pear"));
        assert_ne!(password_hash("a"), 0);
    }

    /// The game finder: a hosting socket answers a search sent to this computer, and the game
    /// is listed once with its version, players and password.
    #[test]
    fn the_finder_lists_a_game() {
        let host = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = host.local_addr().unwrap().port();
        let mut f = Finder::default();
        f.step(true, port);
        let mut buf = [0u8; 512];
        host.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let (n, from) = host.recv_from(&mut buf).unwrap();
        assert_eq!((&buf[..4], buf[6]), (&MAGIC[..], FIND));
        let _ = n;
        let mut w = Writer::new(HERE);
        w.text(&build_id());
        w.u8(2);
        w.u8(4);
        w.u8(1);
        w.u16(port);
        w.u8(1);
        w.u64(77);
        host.send_to(&w.0, from).unwrap();
        host.send_to(&w.0, from).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        f.step(true, port);
        assert_eq!(f.found.len(), 1, "one game, heard twice");
        let g = &f.found[0];
        assert_eq!((g.players, g.max, g.password, g.same_build, g.ready, g.joinable()), (2, 4, true, true, true, true));
        assert_eq!(g.join_text(), format!("127.0.0.1:{port}"));
        f.step(false, port);
        assert!(f.found.is_empty());
    }

    /// Junk, and damaged copies of real messages, are read without a panic by every reader of
    /// the other side's messages, and the item-screen commands they carry stay in range.
    #[test]
    fn junk_messages_are_harmless() {
        let mut n = Net::default();
        let mut s = State::default();
        s.characters = vec![CharState { pack: 909, slot: 3, ordinal: 7, position: [1.0, 2.0, 3.0], ..Default::default() }];
        s.events = vec![(1, Event::Sound(Request::Play { id: 1, block: 2, cue: 3, volume: 4, pan: Some(5) })), (2, Event::Movie(7)), (3, Event::Stinger(2))];
        let real = [s.encode(), Input::default().encode()];
        for k in 0..40_000u32 {
            let b = if k % 2 == 0 {
                let len = (n.next_random() % 300) as usize;
                let mut b: Vec<u8> = MAGIC.to_vec();
                b.extend(PROTOCOL.to_le_bytes());
                b.push((n.next_random() % 32) as u8);
                b.extend((0..len).map(|_| n.next_random() as u8));
                b
            } else {
                let r = real[(k / 2 % 2) as usize].clone();
                n.damage(&r)
            };
            let Some((_, r)) = Reader::open(&b) else { continue };
            let _ = State::decode(&mut r.clone());
            let _ = Input::decode(&mut r.clone());
            let _ = Event::read(&mut r.clone());
            let _ = WorldSync::read(&mut r.clone());
            let _ = read_op((k % 20) as u8, &mut r.clone());
            let _ = apart::read_op(&mut r.clone());
            let mut w = Net { role: Role::Join, ..Default::default() };
            take_welcome(&mut w, &mut r.clone(), "127.0.0.1:1".parse().unwrap());
            read_apart(&mut w, &mut r.clone(), k % 3 == 0);
            if let Some(i) = Input::decode(&mut r.clone()) {
                assert!(i.side.abs() <= 1.0 && i.forward.abs() <= 1.0);
                match command_of(i.cmd.1, i.cmd.2, i.cmd.3) {
                    Some(crate::menus::Command::EquipMote { word, .. } | crate::menus::Command::ClearMote(word)) => assert!(word < 3),
                    _ => {}
                }
            }
        }
        // A mote word out of range changes nothing.
        let mut e = neopets_sim::inventory::Equipment::default();
        assert!(!e.set_mote(0, 9, 3) && !e.clear_mote(5, 0));
    }

    /// The packed records come back within their step: a facing within 1/32767, a large clip
    /// or ordinal exactly, health clamped to 16 bits.
    #[test]
    fn packed_records_round_trip() {
        let mut s = State::default();
        s.heroes[1] = HeroState { who: 1, position: [1234.5, -3.25, 99.0], facing: [0.6, 0.0, -0.8], state: 7, clip: 70_000, hp: 150, hp_max: 40_000, mp: -3, mp_max: 9, flash: 2, down: true, rate: 1.25, drawn: true, raised: false };
        s.characters = vec![
            CharState { pack: 909, slot: 3, ordinal: 70_000, position: [1.0, 2.0, 3.0], facing: [0.0, 1.0, 0.0], clip: 162, rate: 1.5, hp: 35, dead: true, inactive: false },
            CharState { pack: 1066, slot: 0, ordinal: 2, position: [-1.5, 0.0, 8.0], facing: [-0.70710678, 0.0, 0.70710678], clip: 0xFFFF, rate: 0.0, hp: -5, dead: false, inactive: true },
        ];
        let b = s.encode();
        let (_, mut r) = Reader::open(&b).unwrap();
        let d = State::decode(&mut r).unwrap();
        let h = &d.heroes[1];
        assert_eq!((h.who, h.down, h.position, h.state, h.clip, h.hp, h.hp_max, h.mp, h.flash), (1, true, [1234.5, -3.25, 99.0], 7, 70_000, 150, 32_767, -3, 2));
        assert_eq!((h.drawn, h.raised), (true, false), "the held items' flags travel");
        assert!((0..3).all(|k| (h.facing[k] - s.heroes[1].facing[k]).abs() <= 1.0 / 32767.0));
        for (a, c) in s.characters.iter().zip(&d.characters) {
            assert_eq!((a.pack, a.slot, a.ordinal, a.position, a.clip, a.hp, a.dead, a.inactive), (c.pack, c.slot, c.ordinal, c.position, c.clip, c.hp, c.dead, c.inactive));
            assert!((0..3).all(|k| (a.facing[k] - c.facing[k]).abs() <= 1.0 / 32767.0));
        }
    }

    /// A busy state (both heroes, the most characters a state takes, a long talk box, the monster
    /// bar, shots and moving items) stays under the limit before any events go in.
    #[test]
    fn a_busy_state_stays_under_the_limit() {
        let mut s = State::default();
        s.characters = (0..CHARS_A_STATE as u32).map(|k| CharState { pack: 909 + k % 2, slot: (k % 7) as u8, ordinal: k, ..Default::default() }).collect();
        s.talk = Some(Talk { open: true, text: vec![b'x'; 160], speaker: b"Hubert".to_vec(), labels: (b"Next".to_vec(), b"Finish".to_vec()), ..Default::default() });
        s.show.monster = Some((300, 7, b"Plague Serpent".to_vec(), 90, 1, 2));
        s.moving = vec![(909, 1, [0.0; 3], 0.0); 4];
        let size = s.encode().len();
        let base = State::default().encode().len();
        let mut no_chars = s.clone();
        no_chars.characters.clear();
        let mut no_talk = s.clone();
        no_talk.talk = None;
        assert!(size < STATE_LIMIT, "{size} bytes; empty {base}, without characters {}, without talk {}", no_chars.encode().len(), no_talk.encode().len());
    }

    /// A missing state carries the heroes and characters on by their last step (k ticks), and
    /// leaves a teleport (2 units or more in a tick) where it is.
    #[test]
    fn a_missing_state_carries_movement_on() {
        let mut p = State::default();
        let mut s = State::default();
        p.heroes[0].position = [0.0, 0.0, 0.0];
        s.heroes[0].position = [0.5, 0.0, 0.0];
        p.heroes[1].position = [0.0; 3];
        s.heroes[1].position = [5.0, 0.0, 0.0];
        p.characters = vec![CharState { pack: 9, slot: 1, ordinal: 2, position: [1.0, 0.0, 1.0], ..Default::default() }];
        s.characters = vec![CharState { pack: 9, slot: 1, ordinal: 2, position: [1.0, 0.0, 1.25], ..Default::default() }];
        let c = carried(&s, &p, 2.0);
        assert_eq!(c.heroes[0].position, [1.5, 0.0, 0.0]);
        assert_eq!(c.heroes[1].position, [5.0, 0.0, 0.0], "a teleport is not carried");
        assert_eq!(c.characters[0].position, [1.0, 0.0, 1.75]);
    }

    /// A new joiner starts the host's channels afresh: events from 1, nothing acknowledged, the
    /// story and world sync to send again.
    #[test]
    fn a_new_joiner_starts_the_channels_afresh() {
        let mut n = Net { role: Role::Host, ..Default::default() };
        n.events.push_back((41, Event::Stinger(1)));
        (n.next_event, n.acked, n.story_acked, n.world_acked, n.last_seen, n.rtt_ms) = (41, 40, 3, 2, 900, 20.0);
        n.reset_peer();
        assert!(n.events.is_empty());
        assert_eq!((n.next_event, n.acked, n.story_acked, n.world_acked, n.last_seen), (0, 0, 0, 0, 0));
        assert_eq!(n.rtt_ms, 0.0);
    }

    #[test]
    fn input_round_trips_with_ack() {
        let i = Input { generation: 4, ack: 99, story: 6, world: 2, side: 0.5, forward: -1.0, held: RUN | GUARD, right: [1, 255], jumps: 3, attacks: 4, squares: 5, dpad: [1, 2, 3, 4, 5], seen: 7, cmd: (2, 3, 1, 160), seq: 12_345 };
        let bytes = i.encode();
        let (kind, mut r) = Reader::open(&bytes).expect("header");
        assert_eq!(kind, INPUT);
        assert_eq!(Input::decode(&mut r), Some(i));
    }
}
