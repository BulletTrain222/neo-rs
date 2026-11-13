# Multiplayer plan (this build's feature; not in the original game)

Status: planned, nothing built. Written 2026-11-13 from the user's decisions.

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
