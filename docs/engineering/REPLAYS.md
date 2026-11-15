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
backrooms --record FILE          # play; the whole session goes to FILE (as if F9 were pressed at once)
backrooms --replay FILE          # play FILE back in the window, from its in-point; ↑ / ↓ mark tension (FILE.trace.json)
backrooms --replay-verify FILE   # headless: every segment replayed, every kept state compared
backrooms --dump-replay FILE     # the whole file as JSON on stdout
backrooms --record-bot DIR --seed S --levels 2,5 [--bot-seconds 60]   # a recording a floor, the wanderer playing (record-gameplay)
backrooms --replay-frames FILE --sheet DIR [--every 5]   # frames rendered headless from the in-point, and their sheet
backrooms --save-from-replay FILE --at S --save-to OUT   # a recording as a save that resumes S seconds in (SAVES.md)
backrooms --from-save FILE       # play on from a save (or a recording, from its in-point); --save NAME for play's own
backrooms --bench-replay F --seed S --bench-frames N   # what recording and replaying cost (cargo xtask bench)
cargo xtask heatmap FILE…        # where players went, per level (through --dump-replay)
```

A `--record` session starts like a fresh install (no save read or written,
default settings). Any session's per-frame stream is seeded from the clock,
and the seed kept.

## Recording any stretch

Play keeps a **log** (`Recorder`, in the app) from the moment it starts:
each frame and its input, the state every 60 frames, the telemetry. Until
a recording begins it holds only the current segment, from the start a
replay of it would stand at: the session's start (the per-frame seed, the
launch flags, what storage held before the boot read it (the settings),
and the save it read) until the first floor is entered
through its door, the lift, a fall, the Stairwell, her story's end, a
waking after a death or Continue (`docs/engineering/SAVES.md`); that
floor's anchor after. So a recording can begin at any moment of play.
New descent does not start a segment (the new run's first floor is built
as play goes on from the last anchor or the session's start), so after
one the log, and what a replay's seek simulates again, reaches back that
far, at about 250 KB a minute of play. The frames are kept in
blocks of a minute: a full block is shared by every copy, so a recording
taken from the log, and its copy a minute, cost the open block alone.

**F9** (`Record`, rebindable on the Controls page) or the pause list's
**Record** begins one with the next frame, its **in-point**; the same key
or **Stop recording** ends it after the frame it is pressed in, its
**out-point**. While one is made the log keeps every segment from the one
its in-point is in, and a small red **● REC 1:23** stands at the top right
over every screen; when it ends, the file's name stands there for four
seconds once its file is written (or that it was not; or that the disk
had no room, *Room on the disk*). The key works on
every screen and card (as Mute does). The file holds the segment its
in-point is in and every one after; its header names the in- and
out-points. It goes to `~/Library/Application
Support/Backrooms/replays/<UTC date and time>_L<floor>.ocreplay`, written
off the frame (a thread, beside it first and then in its place, so a
reader never sees half of one), with a copy every minute of the clock
while it is made (a crash loses at most a minute) and the whole of it, once,
on quitting. What a recording keeps of the build (the executable's hash
among it) is worked out once, on a thread, as the game starts.

The game's answer to the key is the same whether a recording begins or
ends (it asks the app: `Game::want_record`), so a replay of a session in
which the key was pressed runs as play did; the REC mark and the pause
list's words are the app's (`Game::rec_mark`), drawn and never read by
play.

### Room on the disk

A recording needs room on the disk (GitHub issue #9). As one begins (F9,
Record, `--record`) the app measures the disk the replays folder is on
(`engine::disk::free_share`: `statvfs`, what the game may still write
over the whole volume, at the nearest folder that exists), and while it
runs it measures again every so often; the game's guard says when, and
what the share means (`game::replay::disk::DiskGuard`). With less free
than the line, a recording does not begin, and the mark says so for
eight seconds ("Not recording: 3% of the disk free"); one that runs
ends, kept as it stands (written as any ended recording is), and the
mark says why ("Recording stopped: 3% of the disk free") instead of its
name. The line and the interval are the player's (Settings › Replays:
5% of the disk, from 1% to 20%; every minute, up to every 30); the look
as a recording begins is always made. A disk that cannot be measured
refuses nothing. The interval runs on the game's clock, as the mark's
own time does, and nothing of it is read by play, so a replay never
depends on the disk. The guard's tests pretend to see the disk (40%,
3.4%) and run minutes of the clock in a moment (`replay::disk`'s unit
tests); `engine::disk`'s measure the real one; `--screen
recording-refused` and `recording-stopped` show the mark's warnings.

## Watching

The title's **Replays** (`ui::menu`, `Screen::Replays`) lists the
recordings in the folder, newest first, from their headers alone
(`replays::list`: the floor each began on, `level`; when, in this Mac's
time; how long, `seconds`; "another build" when the executable's hash
differs), and offers Watch and Delete for one. Watching makes way for
the recording's game (the session's ends, and a recording being made is
written first); Escape gives back a fresh session at the title. A
`--replay` is the same viewer in a window of its own, and quits at the
end.

The list is read from the folder at a session's start and again when a
recording is written, and it reaches play as an input (`Listed`, through
the log), since it comes from outside the game: a replay of a session
has the title's rows play had. Deleting takes the line out in play
(the app removes the file). A `--record` playtest lists none.

A **`Viewer`** (`replay::viewer`) plays a recording by the window's
clock, not a recorded frame per window frame: a recording made at 120 Hz
plays at its own pace on a 60 Hz window. Space pauses; − and + step the
speed (¼, ½, 1, 2, 4); ← and → move ten seconds; Enter takes control:
play goes on from where the replay is, its log what the recording held up
to there (`Recorder::resume`; the same file is a save there,
`ReplayFile::cut`, `docs/engineering/SAVES.md`), the keys the recording
held let go as play's first input, the device's mute (the viewer's)
lifted. A run taken over from the Replays screen keeps its saves in
memory (the player's own save is not the recording's run); one taken
over from `--replay` or `--from-save` writes as that launch says. On is simulated from
where it is; back starts a fresh game at the anchor before the place (the
session's start, or a floor's) and simulates to it. A seek runs a slice
of each window frame (10 ms), so the window stays alive and the line at
the foot of the screen (`PlaybackBar`, drawn by `paint_playback`) shows
how far it has come. The sound is hushed while seeking, paused or not at
its speed. A window that falls behind plays at most 16 recorded frames
a frame: the replay slows, it does not race.

A replay starts at the file's first segment and reaches its in-point by
**simulating**, unseen and unheard (the audio device's own mute, not the
game's: that is play's state), the one-shots it makes cut as it goes
(`AudioContext::cut_one_shots`: a minute of footsteps would otherwise
sound at once; nothing in play listens for a sound's end), then plays to
the out-point. The line at the foot of the window says how far a seek has
come (*Watching*); a seek simulates about 85
times faster than play (`PERF.md`, *Recording and replaying*). `--replay-verify` prints one line a segment and fails if any
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
- **Not the speakers.** What a game draws from the per-frame stream as it
  starts (its noise) is made at a fixed 48 kHz (`NOISE_RATE`), whatever
  the output device's rate; the engine plays it at the device's. A session
  played on a 44.1 kHz headset replays on the silent 48 kHz clock that
  `--replay-verify` and every test run on (`tests/replay.rs`).
- **Not the thread.** The per-frame stream's draw count is the stream's
  since its seed, so a replay in the thread that played (as a replay
  chosen in the game will be) counts alike; `tests/replay.rs` replays in
  the playing thread, and `--bench-replay` does.
- **A floor's start is an anchor** (`session::anchor`): the state play
  had just before it entered a floor (the save, the per-frame stream, the
  clocks, the settings, the view, the keys held, what crosses floors
  unsaved, the sound's preset and hum, the dust and pooled effects, the
  film grain's refresh, the mouse's move in that frame, the camera's lens;
  the view is applied, its aspect with it). A fresh game handed the anchor
  enters the floor exactly as play did (`tests/anchors.rs`).

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

A reader steps over a chunk whose tag it does not know (a newer writer's)
without inflating it, and refuses a file whose version is newer than its
own, whose CRC does not match, that has no `END_`, or that would hand a
game what it cannot stand at: a chunk inflating past 256 MB, segments not
one after another, a floor numbered outside 1–9,999, a view of no size,
an in-point outside its frames (`ReplayFile::check`). The `ANCR` chunks are the index: each names
its segment's first frame, and chunks can be stepped over by their length
without inflating them.

### The header (`HEAD`)

```json
{
  "created_ms": 1763140000000,
  "build": { "version": "1.0.0", "commit": "eed15b59aa1c+", "rustc": "rustc 1.100.0 (…)", "exe": "<FNV-1a 64 of the executable, hex>", "macos": "27.0" },
  "flags": "seed=77&level=3",
  "level": 3,
  "seconds": 25.0,
  "in": 1291,
  "out": 2789,
  "format": 1,
  "strings": ["KeyW", "ShiftLeft", "menu:new", …],
  "frames": 2789,
  "hash_every": 60
}
```

`commit` ends in `+` when the tree differed from it. `flags` are the launch
flags (`?seed=…&level=…` as the game reads them). `level` is the floor
it began on and `seconds` how long it plays from its in-point (the
Replays screen lists them without reading further). `in` is the first frame
the recording shows (frames before it, from its first segment's start, are
simulated unseen), `out` the frame after its last; a file without them
shows everything it holds. `strings` is the table
the frames' keys and targets index into; `frames` (the count of every
segment's frames) and `hash_every` are written by the writer and are for
reading only.

### A segment's start (`ANCR`)

```json
{ "first_frame": 0, "start": { "kind": "session", "native_seed": 12648430, "flags": "…", "storage": { "settings": "{…}" }, "saved": "4f435245504c4159…", "settings": {…}, "view": [<u64>, <u64>, <u64>] } }
{ "first_frame": 1328, "start": { "kind": "floor", "anchor": { "level": 4, "how": "door", "ticks": 1092, "state": {…} } } }
```

- `session`: a fresh process. The per-frame stream is seeded with
  `native_seed`; the game is made with `flags`, its storage filled with
  `storage` (what the app read from disk before booting, the settings:
  empty for a fresh install), handed `saved` (the bytes of the save it
  read, `SAVES.md`, as hex; none for a fresh install), booted in `view`
  (width and height in points, scale factor) as the window boots it
  (`Game::boot_in_view`), and given `settings` (`Settings::to_json`). A
  replay writes nothing to disk.
- `floor`: a floor's anchor. `how` is the way in (`door`, `lift`, `fall`,
  `stairwell`, `ending`, `wake`, `continue`); `ticks` is the interval of the frame it was
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
| 11 | the recordings listed (the Replays screen's lines, from outside the game) | a count (varint), then each line's path, label and detail (string indices) |

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
  "rows":   [ { "f": 720, "t": 12.0, "level": 3, "seed": 77, "state": "playing", "x": 1.25, "z": -3.5, "phase": "Build", "menace": 0.2, "sanity": 98.1, "stamina": 100.0, "battery": 97.0 }, … ],
  "events": [ { "f": 1896, "t": 31.6, "level": 3, "seed": 77, "what": "build" }, … ]
}
```

A row a second of the game's clock; `events` is the director's log as it
grew; `f` is the frame each was taken after (a recording keeps those from
its in-point). Neither is read by a replay: they are for playtests and the
heatmap, kept so a recording answers "where" and "when" without being
replayed.

## Size

A minute of play adds about 18 KB (the bot's runs; mostly the look,
as small varints), the state hashes about 2 KB of it. A session's start is
a few hundred bytes (and its storage: a save is a few KB); a floor's anchor
about 6 KB, most of it the dust's motes. `tests/replay.rs` holds the growth
to 25 KB a minute. In memory the log takes about 250 KB a minute and its
work after a frame about 0.1 µs (`cargo xtask bench`, `PERF.md`).

## The code

| Where | What |
|---|---|
| `crates/game/src/replay/file.rs` | the format: `ReplayFile`, `Segment`, `Start`, `encode`, `decode`, `Quantizer` |
| `crates/game/src/replay/mod.rs` | `InputEvent`, `FrameInput`, `Recorder` (the log: frames, hashes, segments at each floor, telemetry; a recording's in-point, letting go of floors no recording needs), `Player` (a segment to the end, checking; a replay keeps no floor anchor of its own), `StateHash`, `Game::boot_in_view` |
| `crates/game/src/session/anchor.rs` | the anchor: `Game::enter_level`, `anchor_state`, `start_from_anchor` |
| `crates/backrooms/src/replays.rs` | the build a recording keeps, where recordings go and the thread that writes them, `--replay-verify`, `--dump-replay`, `--bench-replay` |
| `crates/backrooms/src/main.rs` | the log of play (the window's input, rounded, into it), F9 and the pause list's Record (`toggle_recording`), `--record`, `--replay` and its seek |
| `crates/game/src/replay/viewer.rs` | watching: `Timeline` (each frame's time), `Viewer` (by the clock, the speed, pause, ten seconds either way), `PlaybackBar`, `ReplayLine` |
| `crates/game/src/session/input.rs`, `ui/menu.rs`, `ui/draw.rs` | the `Record` action, the pause list's row, the REC mark (`paint_rec`); the Replays screens; the line under a replay (`paint_playback`) |
| `crates/backrooms/tests/replay.rs`, `tests/anchors.rs` | a run replays to the same states; a recording survives its file; one begun mid-floor or on the first floor replays from its in-point; between recordings the log keeps one floor; an anchor equals play |

A change that alters what a frame does (a new draw from the per-frame
stream, a field that crosses floors) leaves old recordings behind: they
may no longer repeat, and the build in their header says why. A field
that crosses floors and steers play must be in the anchor, or a replay
from it leaves the recording (`tests/replay.rs` catches it from the
title's way in, `tests/anchors.rs` on each way into a floor).
