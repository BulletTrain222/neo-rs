# Multiplayer plan (this build's feature; not in the original game)

Status (2026-11-13): steps 1 to 3 built (local co-op, split screen, co-op rules, the Multiplayer page); step 4 built on the home network (net.rs): host and join, shared talk boxes and cutscene camera, world changes, joining from the title screen. Written from the user's decisions; docs/STATUS.md records each increment.

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
| Doors and world exits | Either player; both go; player 2 placed beside | [ ] player 2's doors | [ ] |
| Talk boxes | Player 1 opens and answers; both see the box | [x] | [x] shown on the joiner |
| Talk box advance | Default: either player can press Cross to go on (not choose) | [ ] | [ ] |
| Cutscenes (scripted camera, bars, fades) | Both watch; player 2 holds still | [x] | [x] camera and bars; [ ] fades |
| Movies (FMV) | Both watch; default: player 1 skips for both | [x] | [ ] joiner plays the host's movie |
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
| Game over and Continue | Only when both are down (or one, with revive off); player 1 chooses | [x] | [ ] joiner follows the reload |
| Pause, item screen, map | Default: either player opens it and the game pauses for both | [ ] player 2 | [ ] |
| Game options (text speed, difficulty) | The host's | [x] | [x] |
| Time, atmosphere, lighting | From the world | [x] | [x] |

### B. Each player's own
| Thing | Rule | One PC | Online |
|---|---|---|---|
| Hero | Player 2: Roberta when the story has her, else a second Tor (Multiplayer page) | [x] | [x] |
| Place, movement, jumping, collisions | Own | [x] | [x] |
| Health, magic, poison and other statuses | Own | [x] | [x] |
| Down and revive | Own (revive or game over, Multiplayer page) | [x] | [x] |
| Weapon, shield, armour | Own hero's equipment row (a second Tor uses Tor's) | [x] | [x] |
| Attacks, combos | Own | [x] | [x] |
| Guard and reflect (shield) | Own | [ ] player 2 | [ ] |
| Power attacks, spells, magic items | Own magic bar | [ ] player 2 | [ ] |
| Quick items (D-pad) | Own panel over the shared bag; the item goes to the user | [ ] player 2 | [ ] |
| Ledges, climbing, ladders, swimming, rope gun | Own | [?] player 2 | [?] |
| Lock-on and target | Own | [?] | [?] |
| Camera and its controls | Own follow camera; invert and speed per computer | [x] | [x] |
| HUD (portrait, health and magic bars) | Own; Neopoints shown on both | [x] | [x] |
| Talk prompt ("Talk" over an NPC) | Shown to player 1 only (player 1 talks) | [x] | [x] |
| Rumble | Own controller only | [ ] player 2 | [ ] joiner's pad |
| Controls and bindings | Per computer; on one PC, player 2 uses the second pad | [x] | [x] |
| Display, audio volume, graphics | Per computer | [x] | [x] |
| Sound listener | One PC: player 1. Online: each computer hears from its own player | [x] | [ ] joiner hears from the host's camera for now |
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
- [ ] Send flags and items only when they change (acknowledged), to keep states small.
- [ ] State size check: keep each state below about 1,200 bytes, or split it, for the internet.
- [ ] In-game notices: "Player 2 joined", "Player 2 left", "Lost the host".
- [ ] Pause menu: "Leave game" for the joiner, "Stop hosting" for the host. The host going to
      the title screen sends Bye, and the joiner returns to the title with a message.
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
