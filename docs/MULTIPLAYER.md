# Multiplayer plan (this build's feature; not in the original game)

Status (2026-11-13): steps 1 to 3 built (local co-op, split screen, co-op rules, the Multiplayer page); step 4 built on the home network (net.rs): host and join, shared talk boxes and cutscene camera, world changes, joining from the title screen, shared sounds, effects, music, items and breakables, pausing and quitting. Player 2 has its own panels, doors, guard, climbing and rumble. Written from the user's decisions; docs/STATUS.md records each increment.

## How to play together

**On one computer (split screen).** Plug in a second controller and start a game. Press
Start on the second controller and player 2 appears beside player 1, on the bottom (or right)
half of the screen. Press Select on that controller to take player 2 out again. Player 2 has its
own health and magic bars, item panel (D-pad up), Mote panel (D-pad down), camera and rumble.

**On two computers (home network).** Both computers need their own copy of the game disc and
the same build of this game.
1. The host starts or loads a game, then opens Options, Multiplayer and chooses **Host a game**.
   The help line shows the address to give your friend (for example 192.168.1.20).
2. The friend opens Options, Multiplayer (from the title screen is fine), types that address
   on **Join address** with the number keys and the dot (Backspace deletes), and chooses
   **Join a game**. They go straight into the host's game as player 2, full screen.
3. The first time the host does this, Windows asks whether to let the game through the
   firewall: allow it on **Private networks**.
4. To stop, either player can Quit from the pause page. If the host quits, the friend goes back
   to the title screen.

**The Multiplayer page.**
| Row | Choices | What it does |
|---|---|---|
| Knocked out | Revive, Game over | Revive: a fallen player gets up when the other stands beside them for 2 seconds, or by themselves after 10. Game over: any fall ends the game |
| Split screen | Top and bottom, Side by side | How one computer's screen is shared |
| Player 2 | Partner, Second Tor | Partner: Roberta when the story has her; Second Tor: always Tor |
| Host a game | | Host the game you are playing (press again to stop) |
| Join address | | The host's address |
| Join a game | | Join that host (press again to leave) |

**What the two players share.** One world and story, one bag of items and Neopoints, the
enemies, talk boxes (either player's Square moves a box on; only player 1 answers questions,
talks to people and shops) and cutscenes. Either player's door takes both. Each player has
their own hero, health, magic, camera, item and Mote panels, and rumble.

**Not yet.** Playing over the internet (outside the home network), and checking everything on
screen with two real computers.

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
- Pressing Start on a second controller adds player 2 beside player 1: Roberta when the story
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
| Story triggers | Fired by player 1 only (keeps story order) | [x] | [ ] joiner's own triggers must not fire |
| Doors and world exits | Either player; both go; player 2 placed beside | [x] (barn traced both ways) | [x] (the host runs player 2's) |
| Talk boxes | Player 1 opens and answers; both see the box | [x] | [x] shown on the joiner |
| Talk box advance | Default: either player's Square goes on (held Cross speeds the text); only player 1 answers | [x] | [x] |
| Cutscenes (scripted camera, bars, fades) | Both watch; player 2 holds still | [x] | [x] camera and bars; [ ] fades |
| Movies (FMV) | Both watch; one PC: player 1 skips; online each side skips its own | [x] | [?] the host's movies are events (not seen in play) |
| Shops | Player 1 drives; shared purse | [x] | [ ] joiner sees the shop page |
| Neopoints, items, keys, quest items | One shared bag | [x] | [x] sent each state |
| Health and magic pickups | Go to whoever touches them | [x] | [x] host decides |
| Chests, switches, push blocks, levers | Either player; one state | [ ] player 2 | [ ] joiner sees their state |
| Enemies, bosses, their health | One set; each targets the nearest standing player | [x] | [x] place, clip, health |
| Enemy health bar (monster bar) | Shown to both | [x] | [x] |
| Shots (projectiles) | One pool; hit either player | [x] | [?] joiner draws the host's (not yet seen in play) |
| Effects, sparks | One set | [x] | [x] the host's effect log |
| Debris, weapon trails | One set | [x] | [?] from the joiner's own copy (clips and breaks match) |
| Music and stingers | One selection | [x] | [x] the host's channels, bank and clips |
| Sound effects | One set of events | [x] | [x] the host's requests |
| Fades, letterbox bars | Shared | [x] | [x] |
| Screen shake | Default: both views shake | [x] | [x] (twice in a host cutscene) |
| Messages ("Got X", notices) | Shown to both | [x] | [x] the level message line |
| Petpet | Follows player 1 | [x] | [ ] joiner draws it |
| Saving | The host's save only; default: the joiner keeps none, and sees "Saved" | [x] | [ ] notice |
| Game over and Continue | Only when both are down (or one, with revive off); player 1 chooses | [x] | [?] joiner told "Game over", follows the reload (restart tested by warp) |
| Pause, item screen, map | Default: either player opens it; on one PC the game pauses for both; online the host's pause pauses both, the joiner's pauses only the joiner (player 2 lets go) | [ ] player 2 | [x] pause; [ ] item screen and map for player 2 |
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
| Guard and reflect (shield) | Own | [x] guard (reflect unchecked) | [x] (the host runs player 2) |
| Power attacks, spells, magic items | Own magic bar; own Mote panel | [x] Mote panel; [?] casting unchecked | [x] Mote panel |
| Quick items (D-pad) | Own panel over the shared bag; the item goes to the user | [x] | [x] |
| Ledges, climbing, ladders, swimming | Own | [x] (ladder traced) | [x] |
| Rope gun | Player 1's (one rope per game) | [x] | [x] |
| Lock-on and target | Own | [?] | [?] |
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
- [ ] Joiner puppet, rest: world items (pickups taken by player 2 still show), switches,
      chests and blocks moved by player 2, enemy decisions that make effects outside the log.
- [x] **Event channel:** numbered events, resent until acknowledged (sounds and effects so far;
      music, messages, shake, fades and rumble to come).
- [x] Shots: each state carries the host's live shots.
- [x] Send flags and items only when they change (acknowledged), to keep states small.
- [x] Events capped at about 900 bytes a state. States measured at 863 to 1,046 bytes early on.
- [ ] Characters only when changed, for busy towns over the internet.
- [x] In-game notices: "Player 2 joined", "Player 2 left", "Lost the host", "The host paused the game".
- [x] Quit on the pause page ends the session; the host's quit sends the joiner to the title
      screen with a message. Pings keep a paused session alive.
- [ ] Pause-page rows "Leave game" and "Stop hosting" (today: Options, Multiplayer page).
- [ ] Smoothing on the joiner if movement looks jumpy.
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
8. [ ] Weapon trails and debris on the joiner: counts compared; sent if they differ.
9. [ ] The petpet online: tried again with a petpet set by the save, compared on both sides.

### Player 2 does more
10. [x] (chest and lever traced; player 1 is brought over for the scene; push blocks not traced) Chests, switches, levers and push blocks: player 2 can use them (one PC and online).
11. [x] (one PC; online the joiner's item screen does not reach the host yet) The item screen opened by player 2 shows player 2's own hero and equipment (**Default**).
12. [x] Player 2's shield reflects shots (traced).
13. [~] (uses go to player 2; healing traced, statuses not) Player 2 casting magic items (traced).
14. [x] (its target list now steps; a lock was not seen set for either player) Lock-on for player 2 (traced).
15. [x] (no swimming in this build; water is deadly ground, item 16) Player 2 swimming (traced where there is water).
16. [x] (lava traced; falling below was already covered) Player 2 falling out of the world comes back beside player 1 (**Default**).
17. [x] (already built in step 3: 60 across or 25 up or down) Player 2 left far behind (more than a set distance, or in an unloaded area) is brought
    beside player 1 (**Default**).
18. [x] After a cutscene, player 2 stands beside player 1.

### Menus
19. [ ] Pause page rows: "Leave game" on the joiner and "Stop hosting" on the host.
20. [ ] The Multiplayer page shows this computer's home-network address while hosting.
21. [ ] The Multiplayer page shows the connection while online: round trip in ms and traffic.
22. [ ] The host can send player 2 away ("Remove player 2" row).
23. [ ] After "Lost the host", the title screen offers to join the same address again
    (**Default**).
24. [ ] A joiner that drops and rejoins in the same session gets player 2's health and magic
    back (**Default**).

### Smoothness
25. [ ] Smoothing on the joiner: heroes and characters drawn between the last two states;
    jumps measured before and after.
26. [ ] The joiner's input delay measured (press to the host acting on it).
27. [ ] Clock drift: the joiner's buffer depth over a long session stays steady.

### Holding up
28. [ ] A long session: 20 minutes online on the autopilot with no stall, a steady buffer and
    steady memory.
29. [ ] An outage: 3 seconds with nothing getting through; the session comes back by itself.
30. [ ] Joining late: in the middle of a talk box, a cutscene and a boss fight.
31. [ ] The host pauses while the joiner is in a menu; the host quits while the joiner loads.
32. [ ] The broken-message test covers the newer messages (parts, events, resync).
33. [ ] Two hosts on one computer: the second gets a clear notice (port in use).

### Split screen
34. [ ] Frame time with split screen measured against one player (perf views).
35. [ ] **Default** layout: side by side; an option for top and bottom on the Multiplayer page.

### Tests and words
36. [ ] Co-op scenarios for the new player 2 actions (chest or switch, fall back, far behind).
37. [ ] net_check.py modes: `--long`, `--outage`, `--late`.
38. [ ] Tests for the co-op rules that need no window (synthetic).
39. [ ] The shared/own checklist above brought up to date (many rows are done).
40. [ ] docs/STATUS.md entries and the multiplayer summary updated.
41. [ ] "How to play together" updated for the new rows and options.
42. [ ] The morning report.
