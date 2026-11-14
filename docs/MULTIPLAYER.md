# Multiplayer plan (this build's feature; not in the original game)

Status (2026-11-13): steps 1 to 3 built (local co-op, split screen, co-op rules, the Multiplayer page); step 4 built on the home network (net.rs): host and join, shared talk boxes and cutscene camera, world changes, joining from the title screen, shared sounds, effects, music, items and breakables, pausing and quitting. Player 2 has its own panels, doors, guard, climbing and rumble. Written from the user's decisions; docs/STATUS.md records each increment.

## How to play together

**On one computer (split screen, 2 to 4 players).** Plug in the controllers, then open Options,
Multiplayer. Set **Players** (2, 3 or 4) and, if you like, each player's controller, then press
Cross on **Local co-op**. In play, players 2 to 4 appear beside player 1, each with their own
part of the screen: two players top and bottom (or side by side), three with player 1 on the
top half and players 2 and 3 on the bottom quarters, four with a quarter each. Cross on Local
co-op again takes them out. A controller press never starts it. Each other player has their own
health and magic bars, item panel (D-pad up), Mote panel (D-pad down), camera and rumble; their
Start opens the pause page with their own item screen (their hero's Motes, and an item used goes
to them). Player 2 is Roberta when the story has her waiting (the Player 2 row); players 3 and 4
are always Tor. Local co-op is off while you host or join online.

**Controllers.** Each player's controller row picks Auto or Controller 1 to 4, numbered in the
order the controllers were plugged in. Auto: player 1 has the keyboard and every controller no
other player has; players 2 to 4 take the next free controllers in order. Player 1 always keeps
the keyboard. In the menus every controller works.

**On two to four computers (home network).** Up to three friends can join one host (players 2,
3 and 4), each on their own computer with their own copy of the game disc and the same build of
this game. The first to join is player 2 (Roberta when they chose Partner and the story has
her); players 3 and 4 play player 1's hero. If a player leaves, the ones after them move up
(player 4 becomes player 3). With Doors take **Only you**, any friend can go off into their own
world; each one comes back when they walk into the host's world (they come back as the next
player number). Two friends who are both away from the host do not see each other.
1. The host starts or loads a game, then opens Options, Multiplayer and chooses **Host a game**.
   The help line shows the address to give your friend (for example 192.168.1.20).
2. The friend opens Options, Multiplayer (from the title screen is fine), types that address
   on **Join address** with the number keys and the dot (Backspace deletes), and chooses
   **Join a game**. They go straight into the host's game as the next player, full screen.
   More friends join the same way.
3. The first time the host does this, Windows asks whether to let the game through the
   firewall: allow it on **Private networks**.
4. To stop, either player can Quit from the pause page. If the host quits, the friend goes back
   to the title screen. While you play, the Multiplayer page shows the round trip (how long a
   message takes there and back) and the traffic each way.
5. If the connection drops, the friend's game goes back to the title screen and joins the same
   host again by itself for up to a minute (press **Join a game** to stop it). Player 2 comes
   back with the health and magic it had. The host can send the last player who joined away
   with **Remove a player**.
6. If the host's computer says the port is in use, another copy of the game is hosting there.
   Change the **Port** row (for example to 47818). The address to give your friend then ends in
   `:47818`, or the friend sets the same port on their Port row and types the address alone.

**Over the internet (not tried yet).** The friend needs a way in to the host's computer
through the host's router:
1. Easiest: before hosting, set **Open the port (UPnP)** to On. While you host, the game asks
   the router to let your friend in, and the help line shows the internet address to give them.
   The way in closes when you stop hosting, or by itself after two hours. Some routers have
   UPnP turned off; then the help line says the router could not open the port.
2. By hand: in the router's settings, forward **UDP port 47817** (or the Port row's) to the host's computer (the
   address the help line shows, for example 192.168.1.20). The friend then joins with the
   host's internet address (search "what is my IP" on the host's computer).
3. Setting a **Password** keeps strangers out.

What it needs: about 25 to 30 KB/s from the host to the friend and 5 KB/s back (measured at
rest, walking and in a boss fight). A round trip under about 100 ms plays well; the game rides
out lost messages and uneven delays.

**The Multiplayer page.**
| Row | Choices | What it does |
|---|---|---|
| Local co-op | | Start or stop play on this computer (press again to stop) |
| Players | 2, 3, 4 | How many play on this computer |
| Player 1 to 4 controller | Auto, Controller 1 to 4 | Each player's controller (shown with the one it has now) |
| Knocked out | Revive, Game over | Revive: a fallen player gets up when the other stands beside them for 2 seconds, or by themselves after 10. Game over: any fall ends the game |
| Split screen | Top and bottom, Side by side | How two players share one computer's screen (three or four always have quarters) |
| Player 2 | Partner, Second Tor | Partner: Roberta when the story has her; Second Tor: always Tor (players 3 and 4 are always Tor) |
| Host a game | | Host the game you are playing (press again to stop) |
| Join address | | The host's address |
| Join a game | | Join that host (press again to leave) |
| Remove a player | | Host only: send the last player who joined back to their title screen and keep hosting |
| Password | None | Optional: the host types one before hosting, and the friend the same before joining |
| Open the port (UPnP) | Off, On | While hosting, ask the router to let a friend on the internet in |
| Port | 47817 (default) | The UDP port to host on, and to join on when the address has none; type it or step it with Left and Right |
| Stay together | On, Off | On: player 2 is brought back when more than 60 units away. Off: each goes where they like in the area; doors and loads still take both |
| Doors take | Both, Only you | Online: Only you lets player 2 go through a door or walk into the next area alone, into its own world; the two meet again when one walks into the other's world. Both: a door takes both, and player 2 stepping into the next area comes back beside player 1 |

**What the two players share.** One world and story, one bag of items and Neopoints, the
enemies, talk boxes (either player's Square moves a box on; only player 1 answers questions,
talks to people and shops) and cutscenes. Either player's door takes both. Either player can
open a chest or pull a lever; when player 2 does, player 1 is brought over to play the scene.
Bosses go after whichever standing player is nearer. Each player has their own hero, health,
magic, shield (blocks and reflections), camera, item and Mote panels, and rumble. A player who
lands in lava or water comes back beside the other, a little hurt.

**Not yet.** Playing over the internet has not been tried (the UPnP row was tried only against
a stand-in router), nor everything on screen with two real computers.

The original game is single-player. Every co-op rule here is this build's own choice. It is
recorded in docs/STATUS.md as such when built, and single-player play stays as it is today.
No spec covers it; spec requests are only for original behaviour the co-op rules rest on.

## The user's decisions (2026-11-13)

- Local co-op comes first: two controllers on one PC.
- Split screen: each player has half the window and their own camera.
- Player 2 plays Roberta when she is in the story, and a second Tor when she is not.
- Online play is peer-hosted, with no dedicated server: one player clicks Host, the other types the
  host's IP address and clicks Join. Home network (LAN) first; internet play (port forwarding)
  comes later.

## How it fits the game as built

- The game already has both heroes and hero switching (SPEC-0053). The hero not in use is
  "parked". Player 2 wakes the parked hero, or a second Tor, and gives it a second controller.
- The game logic assumes one hero throughout (`Play::loco` is used about 570 times in play.rs,
  `Play::hero` about 80). Enemies, triggers, pickups, the camera (`Play::camera`) and the HUD
  all ask for "the hero".
- The simulation runs at a fixed 60 ticks a second from one `HeroInput` with a seeded RNG.
  Report replays re-feed recorded input, but they are close, not exact (the snapshot misses
  enemies, and some hash maps iterate in a different order each run).
- All content comes from the ISO. Each player needs their own copy, and the network carries only
  controller input and numbers, never game data. The disc fingerprint (neopets-data, sha2)
  checks that both players have the same disc, and a build id checks the same build.

## Steps

### 1. Groundwork: a list of players (nothing visible)
- Move the hero's state (movement record, animation, HP and MP, held items, target list, input
  edges) into a `Player`, with `Play` holding the players and player 0 being today's hero.
- Verification: a per-tick state checksum (trace switch). Scenario checks and report replays
  must give the same checksums before and after.
- Optional: exact replays (ordered maps where order affects play, a complete snapshot). These
  help bug reports, but host-hosted online play does not need them.

### 2. Local co-op prototype with split screen
- (Replaced on 2026-11-14 by the Local co-op row; see How to play together.) Pressing Start on a second controller adds player 2 beside player 1: Roberta when the story
  has her, otherwise a second Tor (same outfit for now). Pressing Start again in the pause menu
  takes player 2 out.
- Split screen: two 3D cameras, each with half the window (top and bottom by default; a Display
  option for left and right). Each follows its own player with the game's follow camera (a
  second `FollowCamera`). The HUD is drawn per half.
- Player 2 walks, jumps, attacks, collides, is drawn and animated, and takes hits.
- Hero swapping is off while player 2 is in.
- Cost: the scene is drawn twice, about 4 ms a frame rising to an estimated 7 to 8 ms, with
  shadows and reflections per view. To be measured.

### 3. Co-op rules
| Area | Rule |
|---|---|
| Enemy targets | The nearest player, or the one that hit it last; either player can be hit |
| Health | Each player has their own HP and MP bars |
| Knocked out | The player is down; the partner revives by standing close for a few seconds, or the downed player returns near the partner after a delay; game over only when both are down |
| Friendly fire | Off |
| Pickups | Neopoints and items go into one shared purse; a health or magic pickup goes to whoever touches it |
| Doors and world changes | When one player goes through, both go; player 2 is placed beside player 1 |
| Talking, shops, cutscenes | Player 1 drives them; player 2 steps aside during a cutscene and returns after |
| Story triggers | Fired by player 1 (story order stays as in single-player); doors by either |
| Too far apart | Each has their own camera, but they stay in one world; a player outside a set distance is pulled back to the other |
| Saving | One save (the host's); player 2's hero is saved as the parked hero is today |
| Audio | One listener, at player 1 |

### 4. Host and join on the home network
- Main menu or pause menu: **Host** shows the host's IP address and waits. **Join** asks for an
  IP address. Nothing else is needed: no account and no server.
- Each PC shows a full screen: the host sees player 1, the joiner sees player 2. Split screen
  is for two players on one PC.
- The model is host-authoritative. Each tick the joiner sends its controller; the host runs the
  game with both controllers and sends back what the joiner must draw: characters (position,
  facing, model, animation and time), effects and shots, HUD numbers, sounds, dialogue, the
  world and its loaded neighbour, and player 2's camera. The joiner draws it with the models
  from its own disc.
- Delay: the joiner's input takes one round trip, unnoticeable on a home network.
- World changes: the host tells the joiner which world to load and pauses until the joiner says
  it is ready.
- Transport: plain UDP written in this workspace (no networking crate; such crates lag the
  pinned Bevy 0.19.1), with a small reliable channel for joins, world changes and dialogue.
  One fixed port, and Windows Firewall asks the host to allow the game the first time.
- On joining: same disc fingerprint and same build, or the join is refused with a message.
- Disconnect: player 2 leaves and the host plays on alone.
- Testing: two game windows on one PC, joined over 127.0.0.1.

### 5. Later: internet play
- Port forwarding: instructions in the Host screen, and optionally UPnP to open the port
  automatically. Tools like Tailscale or ZeroTier already work with step 4 unchanged.
- Prediction of the joiner's own movement if the delay is felt.

## Risks
- The one-hero assumption reaches into ledges, climbing, swimming, the rope gun, camera modes,
  boss scripts and scripted cutscenes. Player 2 skips some of these at first (for example,
  stepping aside while player 1 climbs a scripted route).
- Bosses were tuned for one hero. Scaling is left alone unless play shows a need.
- Split screen doubles the drawing work.

## Checklist: what is shared and what each player has (2026-11-13)

Written to work from overnight. **[x]** done, **[ ]** to do, **[?]** built but unchecked.
"Default" marks a rule chosen here because no decision had been made yet. Each default is easy to
change, so say if one should go another way.

### The model in one paragraph
There is one game, the host's. It owns the world, the story, the enemies, the shared bag and the
save. Each player has their own hero, health and magic, camera, HUD, controller, rumble and screen.
On one PC the two players share a window (split screen). Online, the joiner's computer is a
**viewer and a controller**: it sends its controller to the host and draws what the host sends.
It must not make game decisions of its own (enemy choices, triggers, doors, pickups, scripts),
because its copy would drift from the host's. Everything the joiner sees or hears comes from the
host's state or from events the host sends: effects, shots, sounds, music and fades.

### A. Shared by both players (one copy, the host's)
| Thing | Rule | One PC | Online |
|---|---|---|---|
| World and loaded neighbour | Both always in the same world | [x] | [x] world change sends a new welcome |
| Story flags and quests | One set | [x] | [x] sent each state |
| Story triggers | Fired by player 1 only (keeps story order) | [x] | [x] the joiner's scripts load no world; its story is the host's in each state (plan 4) |
| Doors and world exits | Either player; both go; player 2 placed beside | [x] (barn traced both ways) | [x] (the host runs player 2's) |
| Talk boxes | Player 1 opens and answers; both see the box | [x] | [x] shown on the joiner |
| Talk box advance | Default: either player's Square goes on (held Cross speeds the text); only player 1 answers | [x] | [x] |
| Cutscenes (scripted camera, bars, fades) | Both watch; player 2 holds still; player 2 comes beside player 1 when a scene ends | [x] | [x] camera, bars and fades (each state) |
| Movies (FMV) | Both watch; one PC: player 1 skips; online each side skips its own | [x] | [x] queued on the joiner (plan 4; not seen playing) |
| Shops | Priced items in the world; shared purse | [x] | [x] prices and buy lines come with the world sync and the message line |
| Neopoints, items, keys, quest items | One shared bag | [x] | [x] sent each state |
| Health and magic pickups | Go to whoever touches them | [x] | [x] host decides |
| Chests, switches, push blocks, levers | Either player (Default: player 1 is brought over for the scene); one state | [x] chests and levers; [?] push blocks | [x] gates, objects and breakables from the host |
| Enemies, bosses, their health | One set; each targets the nearest standing player | [x] | [x] place, clip, health |
| Enemy health bar (monster bar) | Shown to both | [x] | [x] |
| Shots (projectiles) | One pool; hit, blocked or reflected by either player | [x] | [x] the host's (matched in a fight) |
| Effects, sparks | One set | [x] | [x] the host's effect log |
| Debris, weapon trails | One set | [x] | [?] from the joiner's own copy (clips and breaks match) |
| Music and stingers | One selection | [x] | [x] the host's channels, bank and clips |
| Sound effects | One set of events | [x] | [x] the host's requests |
| Fades, letterbox bars | Shared | [x] | [x] |
| Screen shake | Default: both views shake | [x] | [x] (twice in a host cutscene) |
| Messages ("Got X", notices) | Shown to both | [x] | [x] the level message line |
| Petpet | Follows player 1 | [x] | [x] the host's petpet sent with each state |
| Saving | The host's save only; default: the joiner keeps none, and sees "The host saved the game." | [x] | [x] notice |
| Game over and Continue | Only when both are down (or one, with revive off); player 1 chooses | [x] | [x] the game-over line on the joiner; Continue reloads both (plan 4) |
| Pause, item screen, map | Default: either player opens it; on one PC the game pauses for both and the item screen is the opener's; online the host's pause pauses both, the joiner's pauses only the joiner (player 2 lets go) | [x] | [x] pause; the joiner's item screen commands go to the host |
| Game options (text speed, difficulty) | The host's | [x] | [x] |
| Time, atmosphere, lighting | From the world | [x] | [x] |

### B. Each player's own
| Thing | Rule | One PC | Online |
|---|---|---|---|
| Hero | Player 2: Roberta when the story has her, else a second Tor (Multiplayer page) | [x] | [x] |
| Place, movement, jumping, collisions | Own | [x] | [x] |
| Health, magic, poison and other statuses | Own; each trickles back | [x] | [x] |
| Down and revive | Own (revive or game over, Multiplayer page) | [x] | [x] |
| Weapon, shield, armour | Own hero's equipment row (a second Tor uses Tor's) | [x] | [x] |
| Attacks, combos | Own | [x] | [x] |
| Guard and reflect (shield) | Own | [x] guard and reflect | [x] (the host runs player 2) |
| Power attacks, spells, magic items | Own magic bar; own Mote panel | [x] Mote panel, power attack; items go to the user | [x] Mote panel |
| Quick items (D-pad) | Own panel over the shared bag; the item goes to the user | [x] | [x] |
| Ledges, climbing, ladders | Own (no swimming in this build; deadly ground puts player 2 back beside player 1) | [x] (ladder and lava traced) | [x] |
| Rope gun | Player 1's (one rope per game) | [x] | [x] |
| Lock-on and target | Own | [x] target list (a lock not seen) | [x] (the host runs player 2) |
| Camera and its controls | Own follow camera; invert and speed per computer | [x] | [x] |
| HUD (portrait, health and magic bars) | Own; Neopoints shown on both | [x] | [x] |
| Talk prompt ("Talk" over an NPC) | Shown to player 1 only (player 1 talks); door prompts to each player | [x] | [x] |
| Rumble | Own controller only | [x] | [x] |
| Controls and bindings | Per computer; on one PC, player 2 uses the second pad | [x] | [x] |
| Display, audio volume, graphics | Per computer | [x] | [x] |
| Sound listener | One PC: player 1. Online: each computer hears from its own player | [x] | [x] |
| Pickup and damage flashes | Own | [x] | [x] |

### C. Online plumbing
- [x] Host, join by address, refusal for a different build or disc, timeout, Bye.
- [x] Joining from the title screen.
- [x] A world change sends a new welcome; old-world states wait.
- [x] **Joiner as a puppet, first part:** no hits, deaths, doors or input of its own; its sounds
      and gameplay effects are muted (the host's play instead).
- [x] Joiner puppet, rest: world items, switches, chests, gates and breakables from the host;
      the joiner's scripts load no world (plans 2 to 4).
- [x] **Event channel:** numbered events, resent until acknowledged (sounds and effects so far;
      music, messages, shake, fades and rumble to come).
- [x] Shots: each state carries the host's live shots.
- [x] Send flags and items only when they change (acknowledged), to keep states small.
- [x] Events capped at about 900 bytes a state. States measured at 863 to 1,046 bytes early on.
- [x] Characters only when changed, for busy towns over the internet (plan 3).
- [x] In-game notices: "Player 2 joined", "Player 2 left", "Lost the host", "The host paused the game".
- [x] Quit on the pause page ends the session; the host's quit sends the joiner to the title
      screen with a message. Pings keep a paused session alive.
- [x] Leaving, stopping and Remove player 2 are Multiplayer page rows (the pause page keeps the
      original rows; its Quit ends the session). The page shows the round trip and traffic.
- [x] Smoothing on the joiner: a jitter buffer (plan 3) and movement carried on over a missing state (plan 4).
- [x] Joining again by itself after the host is lost; player 2's health kept for the same game.
- [ ] Test on two real computers on one home network (needs the user).
- [ ] Internet play: port forwarding instructions on the Multiplayer page, optional UPnP (later).

### D. Order of work tonight
1. Joiner as a puppet (stops the drift at the root).
2. Event channel, with sound effects first (heard from the joiner's own player).
3. Shots and effects on the joiner.
4. Music, fades, shake, messages and the monster bar.
5. Pause, leave and stop-hosting menu rows, host quit, joined and left notices.
6. Game over and Continue online; movies on both.
7. Player 2's own actions: doors, chests and switches, guard, quick items, magic, item screen.
8. Smaller states (flags and items on change).
Each item is tested with two windows over 127.0.0.1, recorded in docs/STATUS.md, checked with
Check.cmd and committed on its own. Anything unclear goes in the morning report.

## Night plan 2 (2026-11-13, the display is off)

Every item is tested from logs and traces (no screen). Each one is recorded in docs/STATUS.md,
checked with Check.cmd (and scripts/scenarios.py when single-player code changes) and
committed on its own. **Default** marks a rule chosen without the user; the morning report lists
them all.

### Tools first
1. [x] `scripts/net_check.py`: starts a host and a joiner window over 127.0.0.1 (host on the
   autopilot), waits, reads both logs and reports: joined, welcomes, events made and
   acknowledged, effect objects on both sides, music on both, state size. Fails loudly.
2. [x] Lossy network switches: NEOPETS_NET_DROP=percent (drop that share of received messages)
   and NEOPETS_NET_DELAY=ms (hold received messages that long). Test: at 20% loss and 100 ms,
   every event is still played once and in order, and the session holds.
3. [x] A decode test: random and truncated bytes never panic any decoder (synthetic bytes).
4. [x] Session tests: a second joiner is refused while one is in; a join before the host is in
   a game is refused; a joiner can leave and join again; the host keeps hosting after a leave.

### Player 2 can do more (one PC and online)
5. [x] Doors: player 2 walking through a door or exit takes both players. Player 1 arrives
   with player 2 beside. Doors that need Square show the prompt on player 2's half, and player
   2's Square uses them. **Default:** either player's door moves both.
6. [x] Talk boxes: either player's Cross moves a box on (Default); only player 1 chooses an
   answer.
7. [~] (audited: no shared state clashes; not traced) Player 2's Mote magic and power attacks: traced on one PC.
8. [x] (ladder traced; ledges and edges run the same steps) Player 2 on ledges, ladders, climbing and swimming: traced where the scenarios go, and
   gaps recorded.
9. [x] Rumble: each player's own controller; the joiner's comes as an event.
10. [x] One PC: "Player 2 joined" and "Player 2 left" in game, as online.

### The joiner sees the same world
11. [x] World items: pickups taken (by either player) vanish on the joiner, and dropped ones
    appear.
12. [~] Breakables and moving objects (breakables done; gates and platforms still the joiner's own scripts, matched at the farm where nothing moved): broken crates, opened doors and gates, and platforms
    as on the host.
13. [x] Sounds heard from the joiner's own player: the host sends where a sound is, and the
    joiner works out volume and pan from its own camera.
14. [x] Effects already running when the joiner arrives are sent at the welcome.
15. [~] (measured: 667 bytes in Meridell, 970 at the farm; delta left until needed) Characters only when they change (states smaller in busy places), and a state-size
    reading in a busy town.
16. [x] (no change needed: no cursor in question boxes; shop prompts are player 1's) Talk box choice cursor, shop prices and the buy prompt on the joiner.
17. [x] (player 2's door prompt is sent; talk prompts are player 1's) Prompts on the joiner are player 2's own (door and talk prompts where player 2 stands).
18. [x] The double shake in host cutscenes on the joiner.

### Guards against breaking it later
19. [x] A co-op scenario in scripts/scenarios.py: player 2 joins, walks, attacks, is hit, goes
    down and is revived.
20. [x] net_check.py run after each online change.

### Words
21. [x] "How to play together" at the top of this file: one PC, home network, what each option
    does, what to do when Windows asks about the firewall.
22. [x] The morning report: what was done, what was not, every Default, what needs the screen
    or a second computer.

### Not tonight (needs the user)
- Looking at it on screen; two real computers; internet play (port forwarding, UPnP).

### Night plan 2 report (2026-11-14, morning)
Done and committed, each with Check passing (and the scenarios where single-player code moved;
44 pass): test tools (net_check.py, lossy-network switches, a broken-message test, session
tests), player 2's doors, talk boxes moved on by either player, world items, breakables and
stray characters following the host, sounds heard from the joiner's own player, rumble per
player, one-PC joined and left lines, no double shake, co-op scenarios, player 2 climbing and
ladders, running effects in the welcome, this guide.

Defaults chosen without the user (say if any should change):
- Either player's door moves both players.
- Either player's Square moves a talk box on; only player 1 answers, talks and shops.
- Online, the joiner's pause pauses only the joiner (player 2 lets go); the host's pauses both.
- The rope gun is player 1's alone.
- Loading the next world follows player 1.

Left for later: player 2's Mote magic and power attacks were audited but not traced;
moving objects are the joiner's own (matched where tested); characters are sent in full each
tick (small enough so far); nothing was seen on screen (the display was off); two real
computers and internet play need the user.

## Night plan 3 (2026-11-14, the display is off)

Same rules as plan 2: tested from logs and traces, recorded in docs/STATUS.md, Check.cmd (and
the scenarios when single-player code moves) before each commit, **Default** for every rule
chosen without the user.

### Ready for the internet
1. [x] The welcome (a 12.5 KB save slot) goes in parts of about 1,000 bytes, each acknowledged,
   so it never needs IP fragments (which some routers drop). Tested with loss switched on.
2. [x] (34 KB/s normal, 53 KB/s in a burst; all messages under 1,150 bytes) Bandwidth: bytes a second each way, measured on the farm and in a fight; positions
   packed smaller if it is over about 60 KB/s.
3. [x] A port other than 47817 (private/net.txt, or `host:port` in the join address).

### Smoother joiner
4. [x] Walking into a neighbouring area: the joiner follows without a full reload when its own
   game already made the same move (today every world change reloads the joiner).
5. [x] (a one-state jitter buffer) State arrival timing on the joiner (gaps between states) measured; smoothing if
   movement would stutter.
6. [x] (35 ms, scenes 120 ms more) How long a joiner's reload takes, measured.

### Co-op that keeps going
7. [x] One PC: player 2 stays in through a load, a Continue after game over and a developer
   warp (today player 2 has to press Start again).
8. [x] (byte for byte the same) Saving with player 2 in writes the same slot as without (player 2 never ends up saved
   as player 1, or as the parked hero).
9. [x] Story hero switches (Tor to Roberta and back) with player 2 in: traced, and player 2
   kept sensible (Partner becomes the other hero).
10. [x] (not tried with a real second controller) One PC: the second controller's Start opens the pause page once player 2 is in
    (Select still takes player 2 out). **Default.**

### Fights
11. [x] Bosses: a boss that only ever looks at player 1 is found and, where its code allows,
    turned to the nearest standing player like other enemies.
12. [x] (effects 31 and 31, boss health equal; hits not seen) A boss fight online: host warped into a developer boss fight; the joiner's boss health
    bar, effects and shots compared.
13. [x] Player 2's Mote magic and power attacks traced.
14. [x] (none missing, in a boss fight and with crates breaking) Characters the host makes that the joiner lacks (spawned in fights) counted, and made
    on the joiner if the counts show it matters.

### The joiner's world, complete
15. [x] Moving objects (gates, lifts, platforms): the host's object animations sent when they
    change, the moving ones' times every tick.
16. [x] Characters sent only when changed (positions every tick only for those that move).
17. [~] (travels as a character; a test petpet did not take in the window) The petpet online: compared on both sides.

### Keeping it tidy
18. [x] net.rs split into parts (messages and codecs, the session, the state sync) with no
    change in behaviour (net_check and the codec tests the same before and after).
19. [x] net_check.py checks more by itself: world revisions equal on both sides, object lists
    equal, events all played.
20. [x] A short multiplayer summary entry in docs/STATUS.md pointing at each increment.
21. [x] The morning report.

### Night plan 3 report (2026-11-14, morning)
Done and committed, each with Check passing (and the 45 scenarios where single-player code
moved):
- Internet-ready messages: every message stays under 1,150 bytes (the welcome, the story and the
  world sync go in acknowledged parts; states carry only changed characters and falling items;
  extra event messages when they back up). Normal play is about 34 KB/s from host to joiner (was
  58), a busy moment about 53 KB/s (was 116). Hosting on another port works.
- A smoother joiner: no reload when the host walks into the next area (the joiner asks for a
  welcome only if it ends up elsewhere), a one-state jitter buffer (no repeated frames on a clean
  network), reloads measured at 35 ms.
- Co-op that keeps going: player 2 comes back after a load, Continue or warp on one PC; saves are
  the same with or without player 2; story hero switches go ahead (the partner becomes the other
  hero); player 2's Start opens the pause page.
- Fights: bosses turn on the nearest standing player; a boss fight online matched on both sides
  (31 effect objects, the boss's health); player 2's Mote power attack works; the joiner was
  missing no characters.
- The joiner's world: gates, lifts and platforms come from the host.
- Tidy: net.rs is now net/mod.rs, codec.rs and sync.rs; net_check compares world revisions,
  objects and missing characters by itself; a multiplayer summary heads docs/STATUS.md's
  multiplayer entries.

Defaults chosen without the user (say if any should change):
- The second controller's Start opens the pause page once player 2 is in; in menus both
  controllers drive them.
- A story's hero switch goes ahead with player 2 in; a partner player 2 becomes the other hero.
- Bosses go after whichever standing player is nearer.

Left for later: the petpet was not seen online (a test petpet did not take in the window);
the second-controller pause and re-join were not tried with a real second controller; nothing
was seen on screen; two real computers and the internet need the user.

## Night plan 4 (2026-11-14 night, the display is off)

Same rules as plans 2 and 3: tested from logs and traces (no screen), recorded in
docs/STATUS.md, Check.cmd (and the scenarios when single-player code moves) before each commit,
**Default** for every rule chosen without the user. Items that turn out to need the screen, a real
second controller or a second computer are marked and left for the morning.

### The joiner only watches (finishing the puppet)
1. [x] (nothing changed by the joiner's own game in the traces; its scripts load no world now) The joiner's own story triggers and scripts: count what its game runs that changes shared
   state (flags, items, characters, doors) and stop it, so only the host's game decides.
2. [x] (no change needed: shops are priced items in the world, already synced) A shop on the joiner: while player 1 shops, the joiner sees a "Player 1 is shopping" line
   (**Default**: not the shop page itself).
3. [x] (sent by a diagnostic; not tried through the book online) Saving: the joiner sees "Saved" when the host saves.
4. [x] (also fixed: the game-over and paused lines on the joiner) Game over online: a forced game over on the host shows the joiner the game-over page and
   the joiner follows the Continue (traced with a debug switch).
5. [x] (queued on the joiner; movies were off in the test) Movies online: a host movie reaches the joiner as an event and plays (traced).
6. [~] (the fade comes with every state; not compared frame by frame) Cutscene fades on the joiner: compared with the host's, frame for frame, in a scripted scene.
7. [x] (matched at the same ticks in the Plague Serpent fight) Shots on the joiner: the host's and the joiner's shot lists compared in a fight.
8. [ ] (not tried tonight) Weapon trails and debris on the joiner: counts compared; sent if they differ.
9. [ ] (not tried again tonight) The petpet online: tried again with a petpet set by the save, compared on both sides.

### Player 2 does more
10. [x] (chest and lever traced; player 1 is brought over for the scene; push blocks not traced) Chests, switches, levers and push blocks: player 2 can use them (one PC and online).
11. [x] (one PC; online the joiner's commands now go to the host) The item screen opened by player 2 shows player 2's own hero and equipment (**Default**).
12. [x] Player 2's shield reflects shots (traced).
13. [~] (uses go to player 2; healing traced, statuses not) Player 2 casting magic items (traced).
14. [x] (its target list now steps; a lock was not seen set for either player) Lock-on for player 2 (traced).
15. [x] (no swimming in this build; water is deadly ground, item 16) Player 2 swimming (traced where there is water).
16. [x] (lava traced; falling below was already covered) Player 2 falling out of the world comes back beside player 1 (**Default**).
17. [x] (already built in step 3: 60 across or 25 up or down) Player 2 left far behind (more than a set distance, or in an unloaded area) is brought
    beside player 1 (**Default**).
18. [x] After a cutscene, player 2 stands beside player 1.

### Menus
19. [~] (the pause page keeps the original rows, its Quit ends the session; the Multiplayer page's Host and Join rows stop and leave) Pause page rows: "Leave game" on the joiner and "Stop hosting" on the host.
20. [x] (built in plan 2; now with the connection) The Multiplayer page shows this computer's home-network address while hosting.
21. [x] The Multiplayer page shows the connection while online: round trip in ms and traffic.
22. [x] The host can send player 2 away ("Remove player 2" row).
23. [x] (it joins again by itself for up to a minute) After "Lost the host", the title screen offers to join the same address again
    (**Default**).
24. [x] A joiner that drops and rejoins in the same session gets player 2's health and magic
    back (**Default**).

### Smoothness
25. [x] (stalls at 20% loss 492 to 167) Smoothing on the joiner: heroes and characters drawn between the last two states;
    jumps measured before and after.
26. [x] (12 ms on one computer, 65 ms with 50 ms each way) The joiner's input delay measured (press to the host acting on it).
27. [x] (2 to 6 states over 20 minutes) Clock drift: the joiner's buffer depth over a long session stays steady.

### Holding up
28. [x] (found and fixed: the event channel stopping after an overflow, sounds never started piling up, and memory growing about 45 MB a minute in any window) A long session: 20 minutes online on the autopilot with no stall, a steady buffer and
    steady memory.
29. [x] (8 seconds; also fixed the event numbering for a returning joiner) An outage: 3 seconds with nothing getting through; the session comes back by itself.
30. [x] Joining late: in the middle of a talk box, a cutscene and a boss fight.
31. [x] (also: a join with no answer now gives up after 20 seconds) The host pauses while the joiner is in a menu; the host quits while the joiner loads.
32. [x] The broken-message test covers the newer messages (parts, events, resync).
33. [x] Two hosts on one computer: the second gets a clear notice (port in use).

### Split screen
34. [ ] (needs the screen on: frame times are not real with the display off) Frame time with split screen measured against one player (perf views).
35. [x] (already a Multiplayer page row since step 2; Top and bottom stays the default) **Default** layout: side by side; an option for top and bottom on the Multiplayer page.

### Tests and words
36. [x] (six new) Co-op scenarios for the new player 2 actions (chest or switch, fall back, far behind).
37. [x] (`--seconds 1200`, `--outage`, `--join-delay`) net_check.py modes: `--long`, `--outage`, `--late`.
38. [x] Tests for the co-op rules that need no window (synthetic).
39. [x] The shared/own checklist above brought up to date (many rows are done).
40. [x] docs/STATUS.md entries and the multiplayer summary updated.
41. [x] "How to play together" updated for the new rows and options.
42. [x] The morning report.

### Night plan 4 report (2026-11-14, morning)
Done and committed, each with Check passing (and the scenarios, now 51, where single-player code
moved). 36 of the 42 items are done, 3 partly (cutscene fades not compared frame by frame, magic
items' statuses not traced, the pause page keeps its original rows), 3 left (weapon trails and
debris compared on both sides, the petpet online, and split-screen frame times, which need the
screen on).

**The joiner only watches.** Its scripts no longer load worlds (the host's load comes as a new
welcome); a trace found its own game changing nothing of the story. Online now: the game-over
line and Continue (traced with a forced fall), "The host saved the game.", the host's movies,
shots matching in a fight, and joining late into a boss fight.

**Player 2 does more.** Chests and levers (player 1 is brought over for the scene), its own item
screen (one PC, and online its commands now go to the host), shield blocks and reflections, its
own target list, coming back from lava, and coming beside player 1 when a scene ends. Six new
co-op scenarios.

**The Multiplayer page.** The round trip and traffic while online, a Remove player 2 row, a
joiner that joins again by itself after the host is lost (player 2 keeps its health), a clear
line when the port is taken or a join gets no answer.

**Smoother and sturdier.** Movement carries on over a missing state (stalls at 20% loss: 492 down
to 167 a minute). A press reaches the host in about 12 ms on one computer. An outage of 8
seconds recovers by itself.

**Bugs found and fixed tonight:**
- The game-over line never showed on the joiner, and "The host paused the game." stayed up after
  the host carried on.
- A joiner coming back to the same host waited for an event that never came (events piled up).
- After a joiner left or was removed, the host still said "Lost player 2." six seconds later.
- The event channel stopped for good once 4,000 events backed up (now a fresh welcome).
- Script launchers' shots passed straight through player 2, and player 2 never blocked or
  reflected; player 2's target list never updated.
- A join with no answer waited for ever.
- **Single player too:** sounds the audio side never started were kept for good, each sent a
  volume and pan every tick (they piled up to about 40 a tick in world 947).
- **Single player too:** the game's memory grew about 45 MB a minute for as long as it ran (the
  never-ending sound sources were played as looping, which keeps every sample). It now stays
  level (445 MB in play).

Defaults chosen without the user (say if any should change):
- When player 2 opens a chest or pulls a lever, player 1 is brought to player 2's place to play
  the scene, with player 2 beside (as at doors).
- The pause page opened by player 2's Start has player 2's item screen.
- Player 2 in lava or water comes back beside player 1, losing up to 5 health.
- When a scripted scene ends, player 2 more than 8 units away is put beside player 1.
- After "Lost the host" the joiner joins again by itself for up to a minute; a player 2 back in
  the same game gets its health and magic back.
- A join with no answer gives up after 20 seconds.
- More than 4,000 events waiting sends a fresh welcome; a joiner sitting in its own pause menu
  catches up when it returns.

Not checked: anything on screen or heard (the display and speakers were off), a real second
controller, two computers, the internet, the petpet online, split-screen frame times.

The 20-minute session again, after the fixes: it passed every comparison (120 of 120 world
revisions equal, music, objects and characters matching). The host made 388,107 events, against
2.4 million before, with 8 waiting at the end. Traffic finished at 33 KB/s, against 137. Memory
levelled off at about 600 MB for the host and 550 MB for the joiner (before: past 1.4 GB and
still climbing). A state of 1,201 bytes there also showed that a large state could pass the
1,150-byte limit; it now leaves its events to the events messages.

## Night plan 5 (2026-11-14, after plan 4)

Same rules as plans 2 to 4: tested from logs and traces, recorded in docs/STATUS.md, Check.cmd
(and the scenarios when single-player code moves) before each commit, **Default** for every
rule chosen without the user. Nothing here changes the user's router or anything outside this
folder: features that would (opening a port with UPnP) are built but tried only against a
stand-in on this computer, and left for the user to switch on.

### Stress and leaks (plan 4's long session found three bugs; look for more)
1. [x] (passed; memory level; moving items now capped) A one-hour online session on the autopilot: memory, buffer, events, every comparison.
2. [ ] (left at the user's word) A 20-minute online session at 10% loss and 80 ms delay.
3. [x] (565 MB, level) A one-hour single-player session: memory and the asset and entity counts level.
4. [ ] (left at the user's word; the switch is built) 30 world loads in a row (developer warps) in one window: memory and counts after each.
5. [ ] (left at the user's word) The same 30 loads online: a welcome each, no stall, no stray characters.
6. [ ] (left at the user's word) A joiner that joins and leaves 20 times: the host's memory and channels stay clean.
7. [ ] (left at the user's word; the voice count is now traced) Many sounds at once (a busy fight) for 10 minutes: the voice count stays bounded.
8. [ ] (needs a game loaded from the disc; decoding is fuzzed) The joiner's apply step fed random states (synthetic): no panic.
9. [x] (host about 80 us, joiner 12 us) Each window's CPU time per tick for the online work (host's state, joiner's apply),
   from the perf spans (CPU spans are real with the display off; frame times are not).

### Smaller, faster messages (for the internet)
10. [x] (facings, clips and health packed; places kept exact; 13 to 32% less traffic) Positions and facings packed into 16 bits each where the loss is under a hundredth of
    a unit; traffic measured before and after.
11. [x] (left as is: heroes change nearly every tick) Heroes sent only when they change, as characters are.
12. [x] (placed voices' changes not sent at all; the joiner works them out) Sound volume and pan changes (the most common event) sent only when they change by
    more than a step a listener would hear.
13. [x] (not needed: 0.22 s at 100 ms each way) The welcome's save slot packed (runs of zeros), and the join time measured.
14. [x] Bandwidth at rest, walking and in a fight, written in "How to play together".

### How the joiner feels
15. [ ] (not built: needs the screen to judge; see the design note in the report) Player 2 moved at once on the joiner from its own controller (prediction), drawn
    toward the host's place as states arrive; the gap measured with 50 and 100 ms delay.
16. [x] (stalls at 40 ms jitter 142 to 87) The jitter buffer that grows a state when states come unevenly and shrinks when they
    are even; the delay and stalls measured.
17. [ ] (with item 15) The joiner's camera follows the predicted player 2 (no jump when the state arrives).
18. [ ] (not built: a sound heard twice cannot be checked without the speakers) Sounds of player 2's own swings and jumps played at once on the joiner (Default),
    not after the round trip.

### Joining
19. [x] An optional password on the Multiplayer page (Default: none); a wrong one is refused
    with a line.
20. [x] Player names (Default: the computer's name, shortened): "Sam joined.". Replaced on 2026-11-14 at the user's word: players are called Player 1 and Player 2, and no name is sent.
21. [~] (the Multiplayer page's line shows each refusal, where the join starts) The joiner's title screen says "Joining..." and why a join failed (a refusal shows
    on the title screen, not only on the Multiplayer page).
22. [x] (another protocol version told apart too; not traced with two builds) A friendly refusal for a different build ("The host has a newer build").
23. [x] (tried against a stand-in router only) UPnP (Default: off): a Multiplayer page row that asks the router to forward the port
    for the session and removes it at the end; tried only against a stand-in router on this
    computer.
24. [x] "How to play together" gets an internet section (port forwarding by hand, or UPnP).

### Co-op extras
25. [x] (in the Square caption's place; the joiner shows "Down") A down player shows how long until they get up, and a revive bar while the other
    stands beside (HUD, both views).
26. [x] (they did not move: the Square rows were player 1's; fixed) Player 2's push blocks: traced, and fixed if they do not move.
27. [~] (the same swap as the healing traced; statuses not traced) Player 2's magic items' statuses (buffs, cures) traced.
28. [~] (the panels' opening was traced in plan 2; an item used from the joiner's D-pad not traced) Player 2's quick panel and Mote panel on the joiner: a quick item used from the
    joiner's D-pad traced end to end.
29. [ ] (not tried) Player 2 in a boss grab (a boss that holds the hero): traced, freed as player 1 is.
30. [x] (it fell off: no riding step; fixed) Player 2 riding a moving platform (822) traced.
31. [~] (the host runs player 2's ledges as on one PC, traced there; not online) Player 2's ledges online traced.
32. [x] Both players down at once with revive on: game over, as the rules say (traced).
33. [~] (stays player 1's; no line added) Player 2 talking to an NPC: Default stays player 1's; player 2 gets a "Player 1
    talks" line instead of nothing.

### The joiner's world, last pieces
34. [x] (it was missing on the joiner; now sent) The petpet online: given by a host diagnostic after the welcome, compared.
35. [ ] (not tried) Weapon trails and debris compared on both sides.
36. [ ] (not tried; the fade comes with every state) Cutscene fades compared frame by frame in a scripted scene.
37. [~] (both traced in plan 2; not in a boss fight) The host's camera shake and rumble in a boss fight compared.
38. [~] (boss health matched; the bar shows only after a hit, not compared) The monster health bar compared during a fight with hits (a host diagnostic hurts
    the boss).

### Tests and tools
39. [~] (`--seconds 3600` and the joins script; no new flags) net_check.py `--long` (an hour) and `--soak` (joins and leaves) modes.
40. [x] (four new: block, ladder, platform, both down) A scenario per new co-op rule.
41. [x] Synthetic tests for the packing (round trip within its step), the password, names.
42. [x] (and at most 18 characters a state) A test that a state with every field at its largest stays under the limit.
43. [x] scripts/net_check.py reports CPU per tick.

### Single player, found along the way
44. [x] (the looping streams, item 45) Any leak or growth the stress items find in single play, fixed and traced.
45. [x] (the title screen stays at 411 MB) The looping disc streams: one copy of a stream instead of two (the loop keeps a
    second copy as floats); memory measured.

### Words
46. [x] docs/STATUS.md entries and the multiplayer summary.
47. [x] The checklists brought up to date.
48. [x] The morning report.

### Design note: prediction for the joiner (items 15 and 17, not built)
Today the joiner's own player 2 moves only when the host's state comes back: about 30 to 45 ms
on a home network (the press reaches the host in about 12 ms, the host's next tick, the state's
way back, one state held), and the round trip plus about two frames over the internet. A
prediction would run player 2's movement on the joiner from its own controller at once (the
movement update, ledges, ladders and contacts of player 2's co-op step, without attacks, doors
or Square things), and keep each tick's input numbered. The host would send, with each state,
the number of the last input it used for player 2 and player 2's full movement record (not only
its place). On each state the joiner would put player 2 back to the host's record and run the
inputs the host has not used yet again (rewind and replay), then ease the drawn player 2 toward
the result over a few frames when the two differ by under a unit, and snap when more. The camera
would follow the drawn player 2. The risks are visible only on screen: rubber-banding when the
two sides disagree (pushes by enemies, moving platforms, scripted holds), and the camera
swinging. It is best built with the screen on, behind a Multiplayer page row (Default off), and
compared with today's at 50 and 150 ms.

### Night plan 5 report (2026-11-14)
Done and committed, each with Check passing and the scenarios (now 55) where single-player code
moved. 28 of the 48 items are done, 8 partly, 12 left (the stress items 2 and 4 to 7 at the
user's word, item 8, prediction and local sounds for the screen, a boss grab, trails and fades).

**Smaller and steadier online.** States are packed (a character 27 bytes instead of 42, at most
18 characters and 8 moving items a state, so a state stays under 1,150 bytes): 28.9 KB/s on the
farm road (was about 34), 30.5 in a boss fight (was 44.6). The joiner works out the volume and
pan of placed host sounds itself, so those changes are no longer sent (24 KB/s in world 947).
The jitter buffer follows the network (stalls at 40 ms of jitter 142 to 87 a minute), and a late
state no longer steps characters back. The online work is small: about 80 microseconds a tick on
the host, 12 on the joiner.

**Joining.** A Password row (none by default), player names in the joined and left lines (the
computer's name, or `name=` in private/net.txt), a joiner of another build told so. A UPnP row
(Off by default) asks the router to let a friend in while you host; it was tried only against a
stand-in router on this computer, never your router. "How to play together" has an internet
section.

**Player 2.** Grabs and pushes blocks and steps off ladders with Square (the Square rows were
player 1's), rides moving platforms (it fell off before), and its view shows "Down: up in 7 s"
or "Reviving 50%". The host's petpet now shows on the joiner (it was missing). Both players
down ends the game.

**Long runs.** An hour online passed every comparison with memory level (about 610 and 550 MB);
an hour alone stayed at 565 MB. The looping music streams no longer keep a second copy (the title
screen stays at 411 MB; it grew 137 MB in 3.5 minutes before).

Defaults chosen without the user: the jitter buffer holds one to three states as the network
needs; player names default to the computer's name; no password; UPnP off; the down line's
words.

Questions for the original game: R-0284 (part way up a ladder, Stop Climbing takes hold again at
once, for either player).

Not checked: anything on screen or heard, UPnP with a real router, two computers, the internet.

## Plan: players in separate worlds (2026-11-14, not built)

**Today.** There is one game. It runs the current world and the neighbour it streams in, with
their scripts, triggers, characters, enemies, items and effects. The play code uses "the current
world" in about 720 places, and 71 systems read the one game to draw it, play its sounds and show
its HUD. So both players are always in the same world: a door or a load takes both. With
**Stay together** Off they can already roam the current area and its neighbour.

### Three ways to do it
1. **Many worlds in one game.** The game would keep a list of running worlds and each player
   would be in one of them. Every piece of code that means "the current world" (those ~720
   places) would have to ask "which world?". It is one simulation, so it would suit one PC and
   online alike, but it is the biggest change and the likeliest to break single-player play.
2. **Two games side by side, sharing one story** (recommended). While the players are apart,
   player 2 has a game of its own: the same story, bag and purse, but its own world, with that
   world's scripts and enemies. The two games pass each other the story changes they make, as
   small operations (a flag set or cleared, an item gained or used, Neopoints spent). When the
   players meet in one world again, player 2's game is dropped and it joins player 1's as today.
   Online this is most of the way there already: the joiner's computer runs a full game of its
   own, which today only follows the host. While apart, it would run its own world instead.
3. **Stay as today.** Both players in one world, roaming it freely with Stay together Off.

### Phase 1: online (way 2), built 2026-11-14 (steps 1 to 7 and 9; the whereabouts line is a notice for now)
1. A **Doors take** row on the Multiplayer page: Both (as today) or Only you. With Only you and
   Stay together Off, a door player 2 takes leads only player 2 out.
2. **Apart:** the joiner's game takes charge of its own world. Its scripts, doors, triggers,
   enemies and pickups run, where today they are turned off. The host stops sending that world's
   characters and objects, and each side shows a line such as "Player 2 is in the Meridell
   market".
3. **One story, two games:** each side sends the story changes it makes as numbered operations
   (flags, items, Neopoints, collected rows, treasure maps, kill memory). The host puts them in
   one order and sends the result back, so both games hold the same story. A clash, such as
   both spending the last Neopoints, is settled in the host's order (rare).
4. **Meeting again:** when one player's door leads into the other's world, player 2 is welcomed
   into the host's game as it is today, beside player 1.
5. **Story moments while apart** (act starts, hero switches, movies, game over): Default, player
   2 is brought back to player 1 first, so the story runs only once.
6. **Down while apart:** revive on, player 2 gets up by itself after 10 seconds. The game is over
   only when player 1 falls.
7. **Saving while apart:** the host saves its own world; on loading, player 2 starts beside
   player 1.
8. **Traffic while apart:** only story operations and each player's whereabouts, so very little.
9. **Tests:** two windows with player 2 walking into another world, then flags and items
   changed on both sides and compared, then the two meeting again; a 10-minute run.

### Phase 2: one PC, split screen, several nights
10. A second game held next to the first when the players part, made from the first's save as it
    stands, and dropped when they meet.
11. **Drawing:** each half of the screen shows its own game's world, characters, effects and HUD,
    each world on its own render layer, so two worlds that use the same coordinates do not draw
    over each other. The 71 systems that read the game would learn which game a view shows.
12. **Sound:** each game's sounds, heard from its own half's camera.
13. **Cost:** two worlds running and drawn at once, roughly double the work while apart. It would
    be measured on this laptop.

### Decisions for you
- **Which way:** 2, two games sharing one story (recommended), or 1, many worlds in one game.
- **Story moments while apart:** bring player 2 back first (recommended), or let each player's
  world run its own.
- **Player 2's say while apart:** player 2 talks to people, shops and opens chests in its own
  world (recommended; today only player 1 talks and shops).
- **Order:** online first (recommended; most of it is already there), or one PC first.

### Risks
- **The story in two places:** two scripts setting flags in different orders. Kept to a minimum
  by bringing player 2 back for story moments.
- **Original scripts that assume one current world** (worlds replaced or left by scripts) only
  work in the game that runs them, which is the reason for two games and not one.
- **One PC:** memory and frame time with two worlds drawn at once.
