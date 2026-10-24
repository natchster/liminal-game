# The Backrooms

A first-person exploration horror game for macOS on Apple Silicon, written in
Rust. Wander an endless maze of yellowed office rooms that is different for
every run, keep your sanity and your flashlight battery up, avoid what walks
the halls, and find the way down — through doors, holes in the floor, a
stairwell that goes too deep, a shop that closes at night, a house that is
not yours, and a corridor that loops.

- **Rendering:** a Metal renderer with per-fragment Lambert / Phong / unlit
  shading, exponential fog, a place-based light map, skinned and morphing
  creatures, instancing, 4× MSAA and a compositor for the HUD's blend modes.
- **2D:** procedural textures and the HUD are drawn with CoreGraphics and
  CoreText.
- **Audio:** a fully synthesized soundscape on an audio node graph (oscillators,
  filters, convolution reverb, spatial panners) rendered through CoreAudio.
- **Platform:** windowing and input through winit on AppKit.
- **Deterministic levels:** a run seed and a level number always build the
  same level, checked against recorded reference levels.

## Requirements

- A Mac with Apple Silicon (arm64), macOS 12 or newer.
- Rust 1.85 or newer (`curl https://sh.rustup.rs -sSf | sh`).
- The Xcode Command Line Tools (`xcode-select --install`) for the linker and
  `codesign`.

No other SDKs are needed: the Metal shaders are compiled at start-up from
source embedded in the binary, and all meshes and data are embedded too.

## Build and run

```sh
cargo run --release                   # build and play, windowed
cargo run --release -- --fullscreen   # full screen

cargo xtask bundle                    # → dist/Backrooms.app (signed) and dist/backrooms
cargo xtask bundle --run              # … and open it
open dist/Backrooms.app
```

`cargo xtask bundle --debug` packages a debug build (faster to compile,
slower to run).

### Command-line options

| Option | What it does |
|---|---|
| `--fullscreen` / `--windowed` | Start full screen (borderless) or in a window (default 1280×800) |
| `--width W --height H` | Window size in points |
| `--flags 'seed=4242&nosound&scp=loop@2,store@3'` | Launch flags: a fixed run seed, sound off, anomalies forced onto levels, … |
| `--screenshot out.png [--frames N] [--level L] [--seed S] [--play]` | Headless: build a level, simulate N frames and save the last one |
| `--dump-level DIR --seed S --levels 1,2,3 [--screenshot x]` | Headless: start a run like New Game and write each level's build trace, grid, props and objects as JSON (and the spawn view as PNG) |
| `--scale 2` | Headless modes: render at this backing scale (2 = Retina) |

Saves go to `~/Library/Application Support/Backrooms/storage.txt`.

## Controls

| Key | Action |
|---|---|
| WASD / arrows | Move |
| Mouse | Look (click the window to capture the pointer; Esc releases it) |
| Shift | Run |
| Ctrl | Crouch |
| F | Flashlight |
| E | Interact |
| Q | Drop paper |
| G / 1 | Drink almond water |
| R / 2 | Swap battery |
| H | Cry for help |
| J | Journal |
| K | Debug panel |
| M | Mute |
| F11 / Ctrl+Cmd+F | Toggle full screen |
| Cmd+Q | Quit |

## Testing

```sh
cargo test --workspace --release      # everything (≈1 minute in release)
cargo test -p game                    # the game crate's unit and fixture tests
cargo test --release -p backrooms --test reference_levels           # 20 levels vs recordings
cargo test --release -p backrooms --test playthrough -- --nocapture  # a bot plays 2 × 13 levels
```

- **Reference levels** (`crates/backrooms/tests/reference/`): 20 recorded
  levels over two run seeds, including every anomaly. Each must rebuild
  exactly: the seeded-draw count at all 687 build checkpoints (so a
  difference names the pass that changed), the tiles and zones, the exit and
  key props, the entities, the spawn pose and every placed object.
- **Playthrough:** a bot plays whole runs from New Game with the real
  movement and collision, deals with each level's anomaly and goes through
  the door, and checks the anomaly rules of a run.
- **Fixture tests** (`crates/game/tests/`): level generation, the anomaly
  plan, prop and creature meshes, placement and instancing, the world build,
  core props, creatures and their AI, the anomalies, the Stairwell and the
  Loop, the player, ambient sound, the HUD and textures, each against
  recorded reference data.
- **Placer smoke tests:** every placer crate builds its props across 12 seeds
  × 24 repetitions (`cargo test -p placers_p05`).

## Project layout

| Path | What it is |
|---|---|
| `crates/engine` | Platform and rendering: math, scene graph, geometry, materials and textures, the Metal renderer and shaders, the CoreGraphics canvas, the CoreAudio node graph, the window and input loop |
| `crates/game` | The game, platform-independent: state, level generation, the world build, props, creatures, anomalies, player, sound, HUD and game flow (see `docs/ARCHITECTURE.md`) |
| `crates/placers/placers_p00`–`p32` | The ~650 decoration placers, split into crates that compile in parallel |
| `crates/backrooms` | The executable: window, input routing, rendering, headless tools, and the whole-game tests |
| `xtask` | Project automation (`cargo xtask bundle`) |
| `packaging` | `Info.plist` and the app icon |
| `docs` | `ARCHITECTURE.md` (how it works), `DEVELOPMENT.md` (conventions and workflows) |
