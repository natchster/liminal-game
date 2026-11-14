# Replays

A replay is a session's inputs played again into the same simulation: the
game is deterministic, so the same start and the same inputs, frame by
frame, make the same play. This doc is the format (`OCREPLAY`), what makes
it repeat, and the tools that read it. The work that built it, and what is
still to come (recording any stretch, the Replays screen, saves in the same
container), is [`../plan/REPLAYS.md`](../plan/REPLAYS.md); the research
behind it is [`../research/replays.md`](../research/replays.md).

## The commands

```sh
backrooms --record FILE          # play; the session goes to FILE (written every 300 frames and on quitting)
backrooms --replay FILE          # play FILE back in the window; ↑ / ↓ mark tension (FILE.trace.json)
backrooms --replay-verify FILE   # headless: every segment replayed, every kept state compared
backrooms --dump-replay FILE     # the whole file as JSON on stdout
cargo xtask heatmap FILE…        # where players went, per level (through --dump-replay)
```

A recorded session starts like a fresh install (no save read or written,
default settings), with the per-frame stream seeded from the clock (and
the seed kept). `--replay-verify` prints one line a segment and fails if any
leaves the recording; a file made by another build is said first (it may
not repeat: see *What makes it repeat*). `tests/replay.rs` records a
session from the title through a floor's door with the app's own start,
checks it from its start and from the floor's anchor in-process and in the
built game, and writes the sample `target/replays/recorded_77_L3.ocreplay`.

## What makes it repeat

- **Levels** are a pure function of (run seed, level) (`CLAUDE.md`,
  invariant 1).
- **Time is integer ticks** of 1/65,536 s (`CLOCK_TICKS_PER_SECOND`). The
  window's interval is turned into ticks where it enters
  (`clock_ticks`), and both the step and the game's clock (`now_ms`,
  `perf_t`) come from them (`Game::advance_clock`). A recording keeps each
  frame's ticks, so the replay's step is the recording's to the bit.
- **The mouse is rounded where it enters** to 1/256 of a point
  (`Quantizer`, `INPUT_GRID`), with the remainder carried into the next
  move (as Doom carries its mouse): what the game reads is exactly what a
  frame keeps, and slow moves are not lost.
- **The math is the game's own** (`engine::dmath`, correctly rounded),
  so a transcendental gives the same bits on every Mac and every macOS.
- **Chance** comes from the level's stream in a level build and the
  per-frame stream in play; whatever is made lazily draws from a stream of
  its own (`with_own_stream`), and a zone's reverb from one keyed by its
  name (`ir_stream`).
- **The view** (the window's size and scale) is part of the start and its
  changes are inputs (`View`): the camera's aspect, and so what it sees,
  hang on it.
- **A floor's start is an anchor** (`session::anchor`): the state play
  had just before it entered a floor (the save, the per-frame stream, the
  clocks, the settings, the view, the keys held, what crosses floors
  unsaved, the sound's preset and hum, the dust and pooled effects, the
  film grain's refresh). A fresh game handed the anchor enters the floor
  exactly as play did (`tests/anchors.rs`).

What does not repeat: a replay made by another build (other code, other
compiler) may leave the recording at once. Every recording keeps its
build (`build` in the header: version, commit, compiler, the executable's
hash), and a replay says so when the executable differs.

## The file

Little-endian throughout.

```
"OCREPLAY"          8 bytes, the magic
version             u16 (now 1)
reserved            u16 (0)
chunks…             each: tag (4 ASCII bytes) | length (u32) | CRC-32 (u32) | body
```

Each chunk's body is deflated (raw deflate, RFC 1951, with no zlib header; level 9); the length and
the CRC-32 (IEEE, as zip and PNG) are of the deflated bytes, so a damaged
chunk is caught before it is inflated. Chunks come in this order:

| Chunk | Body | Once |
|---|---|---|
| `HEAD` | JSON: the header | first |
| `ANCR` | JSON: a segment's start | per segment |
| `FRMS` | binary: the segment's frames | after its `ANCR` |
| `HASH` | binary: the state after every 60th frame | after its `FRMS` |
| `TELE` | JSON: telemetry | once |
| `END_` | empty | last: a file without it was cut short |

A reader skips a chunk whose tag it does not know (a newer writer's), and
refuses a file whose version is newer than its own, whose CRC does not
match, or that has no `END_`. The `ANCR` chunks are the index: each names
its segment's first frame, and chunks can be stepped over by their length
without inflating them.

### The header (`HEAD`)

```json
{
  "created_ms": 1763140000000,
  "build": { "version": "1.0.0", "commit": "eed15b59aa1c+", "rustc": "rustc 1.100.0 (…)", "exe": "<FNV-1a 64 of the executable, hex>", "macos": "27.0" },
  "flags": "seed=77&level=3",
  "format": 1,
  "strings": ["KeyW", "ShiftLeft", "menu:new", …],
  "frames": 2789,
  "hash_every": 60
}
```

`commit` ends in `+` when the tree differed from it. `flags` are the launch
flags (`?seed=…&level=…` as the game reads them). `strings` is the table
the frames' keys and targets index into; `frames` (the count of every
segment's frames) and `hash_every` are written by the writer and are for
reading only.

### A segment's start (`ANCR`)

```json
{ "first_frame": 0, "start": { "kind": "session", "native_seed": 12648430, "flags": "…", "settings": {…}, "view": [<u64>, <u64>, <u64>] } }
{ "first_frame": 1328, "start": { "kind": "floor", "anchor": { "level": 4, "how": "door", "ticks": 1092, "state": {…} } } }
```

- `session`: a fresh process. The per-frame stream is seeded with
  `native_seed`; the game is made with `flags`, booted in `view` (width
  and height in points, scale factor) as the window boots it
  (`Game::boot_in_view`), and given `settings` (`Settings::to_json`).
- `floor`: a floor's anchor. `how` is the way in (`door`, `lift`, `fall`,
  `stairwell`, `ending`); `ticks` is the interval of the frame it was
  entered in (a replay from it first runs the rest of that frame); `state`
  is `Game::anchor_state` (`session/anchor.rs`).

Every float in a start is its IEEE-754 bit pattern as an unsigned integer,
so JSON keeps it exactly. Frames are numbered across the whole recording:
a segment's frames are `first_frame`, `first_frame + 1`, …; the frame in
which a floor was entered is the previous segment's last.

### Frames (`FRMS`)

```
count               varint
per frame:
  interval          zigzag varint: this frame's ticks minus the previous frame's (0 before the first)
  events            varint, then each event: an op byte and its payload
```

| Op | Event | Payload |
|---|---|---|
| 0 | key down | string index (varint) |
| 1 | key up | string index |
| 2 | look, on the grid | dx, dy in 1/256 points (zigzag varints) |
| 3 | look, off the grid | dx, dy as f64 bits (8 bytes each) |
| 4 | click on a target | string index (a menu row or HUD id; `""` the canvas) |
| 5 | pointer lock | 1 byte: 1 locked, 0 released |
| 6 | focus lost | — |
| 7 | wheel, on the grid | delta in 1/256 (zigzag varint) |
| 8 | wheel, off the grid | f64 bits |
| 9 | hover a menu row | string index |
| 10 | the view changed | width, height, scale as f64 bits |

A varint is 7 bits a byte, low first, the high bit set on every byte but
the last; zigzag maps 0, −1, 1, −2… to 0, 1, 2, 3…. Keys are the window's
key codes (`KeyW`, `ShiftLeft`, `Mouse0`): bindings are applied by the game,
so a recording is the keys pressed, not the actions.

### State hashes (`HASH`)

```
count               varint
per kept state:
  frame             varint: the frame (whole-recording index) after which it was taken
  player, entities, director, world    u64 each
```

Kept after every frame whose index + 1 is a multiple of 60
(`HASH_EVERY`). The four parts (`Game::state_hash`) say where a replay
left: the player (position, look, stamina, sanity, battery, light, crouch), the creatures, the director
(phase, menace, tension), and the world (level, state, the per-frame
stream's draws, the clocks). A replay compares each as it passes it and
stops checking at the first that differs (`Divergence`).

### Telemetry (`TELE`)

```json
{
  "rows":   [ { "t": 12.0, "level": 3, "seed": 77, "state": "playing", "x": 1.25, "z": -3.5, "phase": "Build", "menace": 0.2, "sanity": 98.1, "stamina": 100.0, "battery": 97.0 }, … ],
  "events": [ { "t": 31.6, "level": 3, "seed": 77, "what": "build" }, … ]
}
```

A row a second of the game's clock; `events` is the director's log as it
grew. Neither is read by a replay: they are for playtests and the
heatmap, kept so a recording answers "where" and "when" without being
replayed.

## Size

A minute of play adds about 18 KB (the bot's runs; mostly the look,
as small varints), the state hashes about 2 KB of it. A session's start is
a few hundred bytes; a floor's anchor about 6 KB, most of it the dust's
motes. `tests/replay.rs` holds the growth to 25 KB a minute.

## The code

| Where | What |
|---|---|
| `crates/game/src/replay/file.rs` | the format: `ReplayFile`, `Segment`, `Start`, `encode`, `decode`, `Quantizer` |
| `crates/game/src/replay/mod.rs` | `InputEvent`, `FrameInput`, `Recorder` (frames, hashes, segments at each floor, telemetry), `Player` (a segment to the end, checking), `StateHash`, `Game::boot_in_view` |
| `crates/game/src/session/anchor.rs` | the anchor: `Game::enter_level`, `anchor_state`, `start_from_anchor` |
| `crates/backrooms/src/replays.rs` | the build a recording keeps, `--replay-verify`, `--dump-replay` |
| `crates/backrooms/src/main.rs` | `--record` (the window's input, rounded, into the recorder) and `--replay` |
| `crates/backrooms/tests/replay.rs`, `tests/anchors.rs` | a run replays to the same states; a recording survives its file; an anchor equals play |

A change that alters what a frame does (a new draw from the per-frame
stream, a field that crosses floors) leaves old recordings behind: they
may no longer repeat, and the build in their header says why. A field
that crosses floors and steers play must be in the anchor, or a replay
from it leaves the recording (`tests/replay.rs` catches it from the
title's way in, `tests/anchors.rs` on each way into a floor).
