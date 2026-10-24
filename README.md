# The Backrooms — native macOS build

A native Apple Silicon port of the browser game in the parent folder
(`backrooms.html` / `backrooms.js`, which stay as they are). Everything is
rewritten in Rust:

- **Rendering:** a Metal renderer that draws the game's scene graph the way
  three.js r160 does: same culling and sorting, the same Lambert / Phong /
  Basic / points / sprite shading, FogExp2, and the game's shader patches.
- **2D:** the game's procedural canvas textures and its HTML/CSS HUD are
  drawn with CoreGraphics and CoreText.
- **Audio:** the synthesized soundscape runs on a WebAudio-compatible node
  graph rendered through CoreAudio.
- **Platform:** windowing and input use AppKit through winit.

The level generator is bit-exact: a run seed and a level number produce the
same level as the JS build.

## Requirements

- A Mac with Apple Silicon (arm64), macOS 12 or newer.
- Rust 1.85+ (`curl https://sh.rustup.rs -sSf | sh`).
- The Xcode Command Line Tools (`xcode-select --install`), for the linker and `codesign`.

No other SDKs are needed. The Metal shaders compile at run time from the
source embedded in the binary.

## Build and run

```sh
cd native
./build.sh            # release build → dist/Backrooms.app and dist/backrooms
./build.sh --run      # build, then launch the app
open dist/Backrooms.app
```

You can also use cargo directly:

```sh
cargo run --release -p backrooms                 # windowed
cargo run --release -p backrooms -- --fullscreen # full screen
```

### Command-line options

| Option | What it does |
|---|---|
| `--fullscreen` / `--windowed` | Start full screen (borderless) or in a window (default 1280×800) |
| `--width W --height H` | Window size in points |
| `--hash '#flags'` | The JS build's URL switches, e.g. `#seed=4242`, `#nosound`, `#scp=loop@2,store@3` |
| `--screenshot out.png [--frames N] [--level L] [--seed S] [--play]` | Headless: build a level, simulate N frames and save the last one |
| `--dump-level DIR --seed S --levels 1,2,3 [--screenshot x]` | Headless: start a run like New Game, then dump each level (draw trace, grid, props, objects) as JSON (and a frame as PNG) for `tools/levelcheck/compare.mjs` |
| `--dpr 2` | Headless modes: render at this device pixel ratio (Retina) |

Saves go to `~/Library/Application Support/Backrooms/localStorage.txt` (the
browser build's `localStorage`).

## Controls

Same as the browser build:

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

## How close is it to the browser build?

Checked against the original page running in headless Chromium:

* **Levels are identical.** For run seeds 4242 (levels 1–12) and 77 with forced
  anomalies (levels 1–8), every level matches the browser exactly: all 687
  per-phase / per-placer seeded-draw marks, the tiles and zones, the exit and
  key props, the entities, and every one of the 200–1,300 placed objects
  (position and bounds). That covers all four anomalies (Stairwell, Showroom,
  Borrowed House, Loop).
* **It plays the same.** The browser's playthrough bot, ported
  (`app/tests/playthrough.rs`), finishes 13-level runs from New Game through
  every anomaly with the real movement and collision, and for seed 101 it walks
  each level in exactly as many seconds as the browser bot does (52, 81, 86, 72,
  88, 156 s).
* **It looks the same.** Spawn-point frames agree with the browser's to 1–2/255
  on average after averaging out film grain; the remaining differences are of
  the same size as between two reloads of the browser page (wallpaper patterns,
  grime and panel flicker are randomised per session in both builds). The HUD,
  menu and overlays match at 1× and 2× (Retina).
* **The parts:** ~20 differential test suites generated from the original JS
  (`cargo test -p game`), 650 prop placers checked against the JS over 11.7M
  cases (`tools/placer-diff`), and the renderer checked against three.js r160
  (mean 0.21/255).
* **Speed:** 5–7 ms per frame (simulation + rendering) at 2560×1600 on Apple
  Silicon.

What is not bit-identical: `Math.sin`/`cos`/`atan2` round differently in the
last bit between the platform libm and V8 (and between V8 versions), so long
free-running simulations drift apart at the 1e-15 level; level generation does
not depend on it. The browser's touch controls (phones/tablets) are not ported.

## Testing and comparing with the browser build

```sh
cargo test -p game                    # fixture tests generated from the original JS
cargo test -p placers_p05             # a placer crate's smoke tests (p00..p32)

cargo test --release -p backrooms --test playthrough -- --nocapture   # the bot plays 2 × 13 levels

# whole levels, browser vs native (needs Node and the repo's Playwright: `npm install` in the repo root)
node tools/levelcheck/js-capture.mjs --seed 4242 --levels 1,2,3 --shots
./target/release/backrooms --dump-level target/levelcheck --seed 4242 --levels 1,2,3 --screenshot x
node tools/levelcheck/compare.mjs --seed 4242
node tools/levelcheck/tone.mjs target/levelcheck/js_4242_L1.png target/levelcheck/rs_4242_L1.png --map
```

`compare.mjs` reports, per level, the first build phase or prop placer whose
seeded random draws differ, and compares the tiles, zones, exit, key props,
counts, entities and every placed object. The PNGs are the first frame at the
spawn point from each build. See `docs/ARCHITECTURE.md` for how the port is
organised and `docs/PORTING.md` / `docs/CORE_PORTING.md` for the JS → Rust
conventions.

## Layout

| Path | What it is |
|---|---|
| `engine/` | The platform layer: math, a three.js-compatible scene graph (`Obj`, `Geo`, `Mat`, `Tex`), r160-exact geometry generators, the Canvas 2D API on CoreGraphics, the WebAudio-style graph on CoreAudio, the Metal renderer (`render.rs`, `shaders.metal`, `composite.metal`), and the window / input loop (`app.rs`). |
| `game/` | The game: `Game` (every JS global as a field), level generation (`archgen`), the world build, lighting, entities, player, ambient sound, HUD, game flow, the Stairwell and the Loop, the Showroom and the Borrowed House. |
| `placers/p00`–`p32` | The ~650 prop placers, split into crates so they compile in parallel. |
| `app/` | The `backrooms` executable. |
| `docs/` | `PORTING.md` and `CORE_PORTING.md`: how the JS maps to Rust. |
| `tools/` | Asset conversion and the render-check harness that compares against three.js. |
| `packaging/` | `Info.plist` (and the icon) for the app bundle. |
