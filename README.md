# The Backrooms

A first-person exploration horror game for macOS on Apple Silicon, written in
Rust. You follow someone down an endless building of yellowed office rooms
that is different for every run: M., who went in after E. and left a trail
of folded paper, a camp on every floor and a page of her notebook at each.
Nothing points the way; the door is found by its light and the sound it
makes. Keep your sanity and your flashlight battery up, avoid what walks the
halls (taking her page wakes them), don't trust every trail, and go down —
through doors, holes in the floor, a stairwell that goes too deep, a shop
that closes at night, a house that is not yours, and a corridor that loops —
to her last camp on level 13, and on (`docs/GAMEPLAY.md`).

- **Rendering:** a Metal renderer with per-fragment Lambert / Phong / unlit
  shading, exponential fog, a place-based light map, floor occlusion and a
  soft highlight roll-off, procedural wear on props, a camera grade and lens
  vignette, skinned and morphing creatures, instancing, 4× MSAA and a
  compositor for the HUD's blend modes.
- **Creatures and pacing:** creatures that see ahead of them and search for
  what they only heard, and a pacing director that sends one after you,
  announces it through the place (its steps, failing lights, then silence)
  and backs everything off after a close call.
- **2D:** procedural textures and the HUD are drawn with CoreGraphics and
  CoreText.
- **Audio:** a fully synthesized soundscape on an audio node graph (oscillators,
  filters, convolution reverb, spatial panners) rendered through CoreAudio;
  every sound comes from somewhere: props from where they stand, a few
  signature sounds per level from fixed places.
- **Platform:** windowing and input through winit on AppKit.
- **Deterministic levels:** a run seed and a level number always build the
  same level, checked against recorded reference levels. Each area of a
  level has one purpose (an office wing, washrooms, a maintenance run, or
  nothing at all), and its wallpaper, grime and darkness change at its
  walls.

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
| `--showcase smiler[:dist] [--death]` | With `--screenshot`: keep a creature (`smiler`, `hound`, `watcher`) in front of the camera; `--death` lets it kill you just before the shot |
| `--dump-level DIR --seed S --levels 1,2,3 [--screenshot x]` | Headless: start a run like New Game and write each level's build trace, grid, props and objects as JSON (and the spawn view as PNG) |
| `--scale 2` | Headless modes: render at this backing scale (2 = Retina) |
| `--record FILE` | Play and record the session (inputs, telemetry, the director's events) to FILE, on this machine only (playtests) |
| `--replay FILE` | Play a recorded session back; the Up / Down arrows mark tension rising / falling (`<FILE>.trace.json`), Escape stops |
| `--bench FILE --seed S --levels 1,2,3 [--bench-frames N]` | Headless: time each level's build, first frame, and the CPU update and GPU time per frame along a fixed path (used by `cargo xtask bench`) |
| `--metrics FILE --seed S --levels 1,2,3` | Headless: build the levels as a run would and write their expressive-range measures as JSON (used by `cargo xtask range`) |
| `--fresh` | Headless modes: ignore the saved settings and progress, and fix the per-frame randomness (the same frame every time) |
| `--screen NAME` | With `--screenshot --play`: put a list up for the last frames (`pause`, `leave`, `settings`, `brightness`, `controls`, `title`) |

Saves go to `~/Library/Application Support/Backrooms/storage.txt`.

## Controls

| Key | Action |
|---|---|
| WASD / arrows | Move (with the arrows, crouch with C: Ctrl+arrows is Mission Control's) |
| Mouse | Look (click the window to capture the pointer) |
| Shift (hold) | Run; a spent run comes back at 30 stamina |
| Ctrl (hold) / C (toggle) | Crouch |
| E / left click | Use what you face (an E a moment early still lands) |
| F / right click | Flashlight |
| Q | Drop a sheet of paper |
| G / 1 | Drink almond water |
| R / 2 | Change the battery |
| V / H | Call out |
| Tab / J | Notebook (the arrows, the wheel or a click turn its pages) |
| X (hold) | Look at what you carry and how you are (the screen is otherwise empty at rest) |
| Esc / P | Pause and resume |
| F3 | Debug panel |
| M | Mute |

Settings (from the title or the pause list) change the look speed, invert,
field of view, head bob, camera shake, film grain, the run key (hold or
press), volume, brightness and whether leaving the window pauses; Controls
rebinds any key (Enter on a line, then the key; keys swap). They are kept
apart from the save.

Everything a run needs is under the left hand with W and Shift held, or on
the mouse; the table lives in `crates/game/src/session/input.rs` and
`tests/controls.rs` holds it to that.
| F11 / Ctrl+Cmd+F | Toggle full screen |
| Cmd+Q | Quit |

## Testing

```sh
cargo test --workspace --release      # everything (≈1 minute in release)
cargo test -p game                    # the game crate's unit and fixture tests
cargo test --release -p backrooms --test reference_levels           # 20 levels vs recordings
cargo test --release -p backrooms --test playthrough -- --nocapture  # a bot plays 2 × 13 levels
cargo test --release -p backrooms --test pacing -- --nocapture       # a cautious bot plays with death on
cargo test --release -p backrooms --test listen -- --nocapture       # levels recorded to target/listen/*.wav
```

- **Reference levels** (`crates/backrooms/tests/reference/`): 20 recorded
  levels over two run seeds, including every anomaly. Each must rebuild
  exactly: the seeded-draw count at all 687 build checkpoints (so a
  difference names the pass that changed), the tiles and zones, the exit and
  key props, the entities, the spawn pose and every placed object.
- **Playthrough:** a bot plays whole runs from New Game with the real
  movement and collision, deals with each level's anomaly and goes through
  the door, and checks the anomaly rules of a run.
- **Pacing:** a bot that plays like a cautious player (explores, holds a
  Smiler's gaze and backs away, runs from the rest, keeps still while
  something searches nearby, faces a Watcher it sees) plays four levels for
  five minutes each with death on. The quiet must lead to a build-up within
  2.5 minutes, the calm and the relief stay within their lengths, things
  reach you, and close calls are survived.
- **Listening:** levels are played with the audio rendered offline,
  recorded to `target/listen/` (WAV, plus a loudness timeline PNG) and
  measured in 50 ms windows: a level opens quietly, its signature sounds
  stand out of the room's bed, a creature's notice is loud against the
  quiet before it, and the director's hush silences the ambience.
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
| `docs` | `ARCHITECTURE.md` (how it works), `DEVELOPMENT.md` (conventions and workflows), `SOURCES.md` (the research behind the design and the structure, and where each source was used) |
