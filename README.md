# neo-rs

Open-source Rust and Bevy recreation of **Neopets: The Darkest Faerie** (PlayStation 2), playing
from your own copy of the game disc.

A from-scratch reimplementation of **The Darkest Faerie's gameplay behaviour** in **Rust**, using
the **Bevy** engine. It runs natively on Windows: no emulator and no PS2 BIOS.

neo-rs aims to play like the original: the same rules, timings, menus, story and fights,
wherever they have been implemented.

**Important**

neo-rs is an independent project. It is **not** a decompiled copy of the game, **not** a
distribution of the game, **not** an emulator, and **not** an automated recompilation of the PS2
program.

The project contains no game files. It reads models, textures, levels, text, sound and movies at
runtime from **an ISO image of your own disc**, so a legally obtained copy of the game is
required.

> Neopets is a trademark of Neopets, Inc. Neopets: The Darkest Faerie was developed by Idol Minds
> and published by Sony Computer Entertainment America. neo-rs is not affiliated with or
> endorsed by any of them.

---

## Project status

neo-rs is **experimental and incomplete**. You can start a new game from the title screen,
play through much of the story as Tor and Roberta, and save and load. Many parts are still
unverified against the original, and some are missing. Full compatibility is **not** claimed.

| | |
|---|---|
| Supported disc | **USA 1.00, SCUS-97367** (other versions and regions are rejected) |
| Platforms | Windows (tested). Linux, macOS and consoles have never been tried. |
| Engine | Rust (stable, edition 2024) + Bevy 0.19.1 |
| Needs | Your own disc image. No BIOS, emulator or extraction step. |

**Working today (in outline):** the title screen, the front-end menus and 8 save slots; worlds
streamed from the disc, with characters, animation, lighting, shadows, water and foliage; Tor
and Roberta, with movement, climbing, ledges, ladders, ropes, combat, magic and
switching heroes; common enemies and most bosses; the level scripts and story through all four
acts; dialogue, the journal, the map page, shops, items and petpets; cutscenes and the
original's Bink movies (with a custom decoder); music, sound effects and voices; local
split-screen co-op for 2 to 4 players and online play for up to 4 (this build's own
feature); display and graphics options; rebindable keyboard, mouse and gamepad controls.

**Not there yet:** some boss and story details are still waiting on research. A headless run
of the story's goals flags 40 of them for a closer look. The Cloud Beast is cut content: the disc
has no model, animation or arena for it, so this build uses stand-ins. Mod support has a file
resolver but no menu. Smooth motion above 60 Hz is not built. Online play between two real
computers has not been tested yet.

See [docs/STATUS.md](docs/STATUS.md) for the detailed list.

---

## How it works

- **Original game data at runtime.**
  neo-rs reads the disc image directly: the archive, models, textures, animations,
  levels, text, sound banks, streams and movies. Nothing is extracted to disk and the image is
  never modified.

- **Behaviour independently implemented in Rust.**
  Game systems are written from scratch from plain-language specifications. The original game is
  the behavioural reference; its code is not copied.

- **Clean-room process.**
  A separate research workspace, which is not part of this repository, studies the original and
  writes specifications in [specs/](specs/): data layouts, where data lives on the disc, behaviour
  described in words and maths, constants and test vectors. The implementation side builds only
  from those specs and never sees the original's code. The specs never contain decompiled code,
  code addresses or copied game data, and `scripts/audit-specs.ps1` checks for these. See
  [docs/CLEAN_ROOM.md](docs/CLEAN_ROOM.md).

- **Simulation separate from presentation.**
  The game rules live in `neopets-sim`, which does not depend on Bevy, files or the wall clock and
  runs at a fixed 60 ticks a second. The Bevy application handles drawing, input, audio and
  menus.

- **Your files stay outside the repository.**
  The disc image, its path, saves, settings and logs all live in `private/`, which git ignores.

---

## Getting started

### 1. Install the prerequisites

- **Neopets: The Darkest Faerie**, USA version (SCUS-97367), as an ISO image made from your own
  disc. Nothing in this project downloads it.
- **Rust (stable)** from [rustup.rs](https://rustup.rs), with the MSVC toolchain.
- **Visual Studio Build Tools** with the "Desktop development with C++" workload, including a
  Windows SDK. `scripts\cargo.cmd` finds them by itself.
- **Git**, and **Python 3** if you want to run the test scripts in `scripts/`.

### 2. Get the code

```powershell
git clone https://github.com/BulletTrain222/neo-rs.git
cd neo-rs
```

### 3. Tell neo-rs where your ISO is

Run **`Set-ISO.cmd`** and choose your ISO. The path is saved in `private/iso-path.txt`, which
git ignores, so your local path is never committed. The ISO itself is never copied.

The ISO path is looked up in this order:

1. **Command line:** `--iso "<path to ISO>"`
2. **Environment variable:** `NEOPETS_ISO="<path to ISO>"`
3. **Saved path:** `private/iso-path.txt`, written by `Set-ISO.cmd`

### 4. Check that your disc is found

```powershell
.\Inspect-ISO.cmd
```

This opens the disc, indexes its files and prints what it found, without opening a window. The
game itself checks the disc's boot program against the supported version's fingerprint when it
loads.

### 5. Build and run it

```powershell
scripts\cargo.cmd build --release --locked -p neopets-app
target\release\neopets-app.exe --play --title
```

The first build downloads the crates and compiles Bevy, which takes a while; later builds are
fast. Use the release build, because debug builds are much slower.

Run it **from the project folder**. Saves and settings are read from `private/` in the current
folder. Started with no arguments, the program opens the resource viewer, not the game.

---

## Playing

The game reads a virtual PS2 controller. Every button can be rebound for keyboard, mouse and
gamepad under **Options → Keyboard & Gamepad**. Xbox and PlayStation pads work, and the
on-screen button pictures can show PlayStation, Xbox or keyboard keys.

| Action (PS2 button) | Keyboard and mouse (default) |
|---|---|
| Move | `W` `A` `S` `D` |
| Camera | Mouse, or `I` `J` `K` `L` |
| Jump, confirm (✕) | `Space` / `Enter` |
| Attack, answer (○) | `F` / left click |
| Use, talk (□) | `E` / right click |
| Item panel, back (△) | `T` / `Backspace` |
| Guard (L1) | `Q` |
| Run (R1) | `Shift` |
| Switch hero (L1 + R1) | `Q` + `Shift` |
| Page left / page right, attack (L2 / R2) | `Z` / `C` |
| Show the HUD (L3) | `X` |
| Centre the camera (R3) | `R` / middle click |
| Item screen (Start) | `Tab` |
| Pause (Select) | `Esc` |
| Quick items, map, journal (D-pad) | Arrow keys / number pad |

Menus also work with the mouse, and `Esc` closes them.

### Options added by this build

Beyond the original's options, the Options menu has **Display** (window, resolution, aspect
ratio, render scale, graphics preset, Vulkan or DX12, frame-rate limit, shadows, ambient
occlusion, water reflections and more), **Gameplay** (movie cutscenes, button pictures, attack
turning), **Sound** (a volume for each kind of sound) and **Multiplayer**. Settings are saved in
`private/display.txt`.

### Multiplayer (this build's feature)

- **One computer:** 2 to 4 players in split screen. Open **Options → Multiplayer**, set
  **Players**, and choose **Local co-op**.
- **Home network:** one player starts or loads a game and chooses **Host a game**. The others
  open **Options → Multiplayer → Online** and pick the game under **Games on this network**, or
  type the host's address. Every player needs their own disc image and the same build. Allow the
  game through the Windows firewall on **Private networks**.

See [docs/MULTIPLAYER.md](docs/MULTIPLAYER.md) for the rules and every option.

### Developer tools

| Key | Tool |
|---|---|
| `F1` (pad: LB + L3 + A) | Developer menu: cheats, warps, boss teleports, story flags, character spawner, debug views |
| `F2` | Free camera |
| `F3` | Performance overlay |
| `F4` / `F5` | Pause / step one tick |
| `F7` | Autopilot |
| `F9` | Named save states |
| `F12` | Name what is at the centre of the screen |
| `End` | Save a bug report (screenshot, game state, recent input and the hero's last 10 seconds) to `private/reports/` |

### Command-line options

| Option | Meaning |
|---|---|
| `--play` | Start in the game instead of the resource viewer |
| `--title` | Start at the title screen (with `--play`) |
| `--iso <path>` | Disc image to read (overrides `NEOPETS_ISO` and `private/iso-path.txt`) |
| `--pack <n>` / `--start x,z` | Start in a given world pack, at a given spot |
| `--seed <n>` | Random seed |
| `--flag <n>` / `--give <n>` | Set a story flag / give an item at the start (repeatable) |
| `--original-pacing` | Slow down like the PS2 when a frame runs long, instead of catching up |
| `--autoplay` | Let the autopilot play |
| `--screenshot-at <s>` `--screenshot <png>` | Save a screenshot after some seconds and exit |
| `--hide-hud` | Hide the HUD |
| `--play-trace <ticks>` | Run without a window and print the game state each tick |
| `--resource <n>` | Open the resource viewer at a resource number |

Diagnostics are switched on with `NEOPETS_*` environment variables. `NEOPETS_TRACE_<AREA>`
prints a subsystem's trace.

---

## Running the tests

```powershell
.\Check.cmd
```

This runs, in order:
1. The data and sim tests (`scripts\cargo.cmd test --locked`).
2. The app tests and the Bevy build (`scripts\cargo.cmd test --locked -p neopets-app`).
3. The source audit, which checks that only allowed file types are present.
4. The spec audit, the clean-room tripwire.

- Committed tests never need the disc. They use synthetic bytes or the specs' own test vectors.
- `scripts/scenarios.py` runs named in-game checks on your real disc with the release build,
  one for each fixed bug. Run it after gameplay changes.

---

## Troubleshooting

- **"Choose your ISO with Set-ISO.cmd or pass --iso <path>."** No disc image is configured.
  Run `Set-ISO.cmd`.
- **"fingerprint mismatch".** The image is not the USA 1.00 release (SCUS-97367), or it is
  damaged. Make a fresh image of your disc.
- **A sky dome with no game.** That is the resource viewer. Start with `--play --title`.
- **Saves or settings seem to be missing.** They live in `private/` in the folder you run from.
  Start the game from the project folder.
- **Linker errors on Windows.** Install the Visual Studio Build Tools (C++ workload with a
  Windows SDK) and build through `scripts\cargo.cmd`.
- **The game won't start because of a missing DLL.** Reinstall the Visual C++ redistributable:
  `winget install Microsoft.VCRedist.2015+.x64`.
- **The game runs slowly.** Use the release build, and try a lower **Graphics preset** or
  **Render scale** on the Display page.
- **Friends can't join.** Allow the game through the firewall on Private networks, and check that
  everyone uses the same build. If the port is in use, change **Port** on the Multiplayer page.

---

## Repository layout

```text
neo-rs/
├── Cargo.toml
├── crates/
│   ├── neopets-data/   # ISO reader, archive, and decoders for every kind of disc data
│   ├── neopets-sim/    # engine-independent game rules: heroes, enemies, levels, story, UI rules, sound
│   └── neopets-app/    # Bevy app: drawing, input, audio, menus, multiplayer, developer tools
├── specs/              # the approved clean-room specifications (the only source about the original)
├── docs/               # status, architecture, clean-room process, multiplayer, provenance
├── scripts/            # build launcher, audits and test scripts
├── third_party/
│   └── ps2-float/      # PS2 float arithmetic (MIT OR Apache-2.0)
├── assets/             # art made for this build (the Cloud Beast model), not from the disc
├── mods/               # your local replacement files (ignored by git)
└── *.cmd               # Set-ISO, Inspect-ISO, Run, Check, Progress
```

## Documentation

- [docs/STATUS.md](docs/STATUS.md): what is implemented, against which spec, and what is
  unverified
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how the crates fit together
- [docs/CLEAN_ROOM.md](docs/CLEAN_ROOM.md): the two-room process and what a spec may contain
- [specs/README.md](specs/README.md): the index of specs and their status
- [docs/MULTIPLAYER.md](docs/MULTIPLAYER.md): co-op and online play
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md): setup in detail
- [docs/PROVENANCE.md](docs/PROVENANCE.md): third-party code and what may be committed

## Contributing

Contributions are welcome. To keep the repository publishable and clean:

- **Never commit anything from the disc** (models, textures, sound, movies, executable bytes),
  disc images, BIOS files, save states, captures or decompiler output. Keep local files in
  `private/`.
- **Build only from `specs/`** and public documentation. Never disassemble or decompile the game
  or work from someone else's decompilation. If a spec is missing or unclear, add a question to
  `specs/REQUESTS.md` instead of guessing.
- Mark anything a spec does not cover as "this build's choice" or "stand-in", and record each
  change in `docs/STATUS.md`.
- Run `Check.cmd` before sending changes.

`AGENTS.md` and `START_HERE.md` hold the same rules for AI coding assistants. Much of this code
was written with one.

## License

No license has been chosen yet for this project's own code, so it is all rights reserved for
now. `third_party/ps2-float` is MIT OR Apache-2.0. See [docs/PROVENANCE.md](docs/PROVENANCE.md).

Neopets: The Darkest Faerie and all of its content remain the property of their owners.
