# Building, running and the command line

How to build the game, run it, and drive it from the command line. How
it is made inside is [`ARCHITECTURE.md`](ARCHITECTURE.md); how to work on
it, its tests and its tools, [`DEVELOPMENT.md`](DEVELOPMENT.md); its
controls, [`../design/UI.md`](../design/UI.md) (*The keys*).

## Requirements

- A Mac with Apple Silicon (arm64), macOS 12 or newer.
- Rust 1.100.0, pinned by `rust-toolchain.toml`: install rustup
  (`curl https://sh.rustup.rs -sSf | sh`), and the first build fetches
  the pinned compiler. A release, not a channel: replays and the
  reference levels hold the game's arithmetic to one compiler.
- The Xcode Command Line Tools (`xcode-select --install`) for the linker
  and `codesign`.

No other SDK is needed: the Metal shaders are compiled at start-up from
source embedded in the binary, and every mesh, texture, voice and table
is embedded too.

## Build and run

```sh
cargo run --release                   # build and play, in a window
cargo run --release -- --fullscreen   # full screen

cargo xtask bundle                    # → dist/Backrooms.app (signed) and dist/backrooms
cargo xtask bundle --run              # … and open it
open dist/Backrooms.app
```

`cargo xtask bundle --debug` packages a debug build (faster to compile,
slower to run).

## Where things are kept

Under `~/Library/Application Support/Backrooms/`:

| Path | What |
|---|---|
| `storage.txt` | the player's settings, kept apart from the save ([`SAVES.md`](SAVES.md), *The storage file*) |
| `saves/<name>.ocsave` | the run, saved on entering a floor, at a death and when something is taken (`default`; [`SAVES.md`](SAVES.md)) |
| `replays/` | the stretches recorded with F9, listed on the title's Replays screen ([`REPLAYS.md`](REPLAYS.md)) |

A headless run, a `--record` session and a replay read these at most,
and never write them.

## The command line

`backrooms [options]` (`cargo run --release -- [options]`). Every option
is listed here; `crates/backrooms/tests/structure.rs` fails on one the
executable reads and this file does not name.

### Playing

| Option | What it does |
|---|---|
| `--fullscreen` / `--windowed` | Start full screen (borderless) or in a window (default 1280×800) |
| `--width W --height H` | The window's size in points (or a headless render's) |
| `--flags 'seed=4242&nosound&scp=loop@2,store@3'` | Launch flags: a fixed run seed, sound off, anomalies forced onto levels, … (`prologue=1` / `prologue=0`: New descent goes through the way in, the tunnel, or not; by default it does, but for a run on a chosen seed or level) |
| `--save NAME` | The save play reads and writes, `saves/NAME.ocsave` (`default`) |
| `--from-save FILE` | Play on from a save, or from a recording's in-point; with a headless mode, that mode starts from it |
| `--help` | The options, in short |

### Recordings and replays

([`REPLAYS.md`](REPLAYS.md))

| Option | What it does |
|---|---|
| `--record FILE` | Play, and record the whole session to FILE (`OCREPLAY`: its frames, kept states, telemetry, the director's events), as if F9 were pressed at once; for playtests, on this machine only |
| `--replay FILE` | Play a recording back; the Up / Down arrows mark tension rising / falling (`FILE.trace.json`), Escape stops |
| `--replay-verify FILE` | Headless: replay a recording from its start and from each floor's anchor in it, every kept state compared |
| `--dump-replay FILE` | A recording as JSON, on stdout |
| `--record-bot DIR --seed S --levels 1,4 [--bot-seconds 60]` | Headless: a recording a floor, each a session the wanderer bot plays |
| `--replay-frames FILE --sheet DIR [--every 5]` | Headless: a recording's frames rendered every so many seconds, and their contact sheet |
| `--save-from-replay FILE --at S --save-to OUT` | A recording as a save that resumes S seconds after its in-point ([`SAVES.md`](SAVES.md)) |
| `--bench-replay FILE --seed S --bench-frames N` | Headless: what recording and replaying a bot's session cost (used by `cargo xtask bench`) |

### Pictures, headless

Rendered offscreen, never from the screen.

| Option | What it does |
|---|---|
| `--screenshot out.png [--frames N] [--level L] [--seed S] [--play]` | Build a level (or the title, without `--play`), simulate N frames and save the last one |
| `--fresh` | Ignore the saved settings and progress, and seed the per-frame randomness: the same frame every time |
| `--scale 2` | Render at this backing scale (2 = Retina) |
| `--screen NAME` | With `--play`: put a list or card up 45 frames before the shot (`pause`, `leave`, `settings`, `brightness`, `controls`, `recording`, `replays`, `dead`, `notebook`, `cutscene`, the way in's `tunnel`, `tunnel-door`, `tunnel-open`, …; `CLAUDE.md` lists them all) |
| `--showcase smiler[:dist[:walk]] [--death]` | Keep a creature (`smiler`, `hound`, `watcher`, …) in front of the camera at that distance, or place it there once and let it go about (`walk`); `--death` lets it kill you just before the shot. A floor of the rooms (level 4 on): nothing walks the Building |

### Levels

| Option | What it does |
|---|---|
| `--dump-level DIR --seed S --levels 1,2,3` | Start a run like New Game and write each level's build trace, grid, props and objects as JSON |
| `--screenshot x` | With `--dump-level`: the spawn view of each level as PNG too |
| `--views` | … and one more from inside each zone, and the exit, camp and trail views (the poses along the walls that lean printed) |
| `--views-at "x,z,yaw;…"` | … and one from each of these poses (world metres, radians): the same poses in two builds |
| `--shots-at "name:x,y,z,yaw,pitch;…"` | … and one from each of these cameras exactly (a prop before and after) |
| `--prop-census N [--census-only a,b] [--census-use]` | … and up to N framed pictures of every kind of placed prop, indexed in `census.tsv` (only the kinds named; each used first) |
| `--far-figure` | … on the Building's floors: the figure across the void from 4, 12 and 25 m and from its own view, each also without it; its cameras printed for `--shots-at` |
| `--knock-demo` | … on the Well (level 1): the first door that answers with a bottle, a battery and a note, before, as it comes out, and after |
| `--vending-demo hang\|bin` | With `--views-at`: every lit vending machine first played to that state |
| `--metrics FILE --seed S --levels 1,2,3` | Build the levels as a run would and write their expressive-range measures as JSON, no rendering (used by `cargo xtask range`) |

### Measuring

([`PERF.md`](PERF.md))

| Option | What it does |
|---|---|
| `--bench FILE --seed S --levels 1,2,3 [--bench-frames N]` | Time each level's build, its first frame, and the CPU update and GPU time a frame along a fixed path (used by `cargo xtask bench`) |
| `--bench-creatures FILE --seed S` | Time the creature pack (every kind that roams the rooms, two of each, all hunting you) on level 4 |

### Cutscenes

([`../design/CUTSCENES.md`](../design/CUTSCENES.md))

| Option | What it does |
|---|---|
| `--cutscene ID [--stage S] [--bundle FILE] --sheet DIR [--every 0.25]` | Play a cutscene on its stage (else its first), from the library or a bundle: its frames, a contact sheet, its timeline and its contract |
| `--cutscene ID --screenshot F --cut-at T` | The cutscene's frame at T seconds |
| `--cutscene-check FILE` | A compiled cutscene bundle against the game's rules |

### The visual playtest

([`VISUAL_PLAYTEST.md`](VISUAL_PLAYTEST.md); `cargo xtask playtest visual` runs it)

| Option | What it does |
|---|---|
| `--tour DIR --seed S --levels 1,2,3` | The playtest's renders: the edges of the walkable floor, looked at, along, up and down (`DIR/L<n>/views.tsv`) |
| `--tour-budget N` | The views a level (180) |
| `--tour-all` | Every stand's views rendered for the detectors (the eyes still get the budget's sample) |
| `--tour-area x0,z0,x1,z1` | Only inside this area (world metres) |
| `--tour-plant hole\|fight` | A known fault planted at the first stand (the detectors' own test) |
| `--tour-at "NAME:x,y,z,yaw,pitch;…"` | These cameras instead of the tour's: a finding seen again exactly |
