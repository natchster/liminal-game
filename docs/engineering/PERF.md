# Performance

Measure, then change. `cargo xtask bench` times twelve levels (runs 4242 and
7, levels 1–6) headless at 1280×800: the level build, the load plus first
frame (pipeline compiles land there), and 240 frames along a fixed path
through every zone — the game update on the CPU and the frame on the GPU
(Metal's command-buffer GPU start and end times), with draw calls and
triangles. Each measure is the median of three repeats. A level over a
budget gets a second look: three more runs, judged on the median of all
six. A p99 of 240 frames is about the third-worst frame, so one hitch of
the machine's can carry it (the Well, 4242 L1, has measured 1.4–6.1 ms
over the same code while its p50 held at 1.3 ms); a level that is really
slow stays slow over six. Each level's JSON (`target/xtask/bench/*.json`)
lists the frames whose update took over 2 ms (`update_slow`: which, and how
long), so a high p99 can be read, not guessed at. (At E4 one build put the
update p99 of every level, the Building's too, at 4.7 ms under `cargo xtask
bench` and at 0.75 ms when the same binary was run by hand; a change to code
after the timed loop alone took it away, and no frame over 2 ms has been
seen since. Nothing from those runs was recorded.)

It fails when a level goes over a budget below, or when a measure's history
on this machine (`perf/history.csv`) shows a lasting step up: the best split
of the series into an earlier and a later mean, more than 10% slower and
beyond chance in a permutation test (p < 0.01), with at least eight runs on
record. One slow run cannot fail it; a regression that stays does.

```sh
cargo xtask bench               # measure, check, and append the run to perf/history.csv
cargo xtask bench --no-record   # measure and check only (the full check runs this)
```

Record a run (without `--no-record`) after each change that touches
rendering, level building or per-frame code, and commit the history with it.

## Steps explained

A step the bench finds is looked at and explained here (it is flagged
while it is among the last three runs of its series). To find a step by
bisecting, build each older commit with the speaker guard taken in
(`docs/engineering/DEVELOPMENT.md`, "A build of an older commit plays aloud"): a build
from before `03e4b80` benches out loud.

- **T1–T2, Rust 1.100 and edition 2024** (`295928f+` and `6be102a+` in the
  history, flagged by T2's full check and T3's): seed 4242's levels 2 and 3
  update p99 0.80 → 1.3–1.9 ms, and once 7 L6's GPU p99 2.0 → 3.7, the
  medians unchanged (0.02–0.03 ms). Not the toolchain. Measured apart, the
  pre-T1 code built by 1.85.1 (A) and by 1.100.0 (B), and T2 (C), benched
  in turn three times each so the machine's state fell on all three alike:
  1.100 is the faster at p99 on 11 of the 12 levels (median of three, e.g.
  4242 L3 1.04 → 0.82 ms, 7 L2 1.10 → 0.84) and builds levels 1–4% sooner;
  C's medians are B's; the creature pack's ranges overlap (A 0.95–1.17, B
  0.85–1.24, C 1.01–1.47). What rose is the tail on a busy machine: the run
  recorded after T1 was taken straight after a full check (load average
  45), and that day a virtual machine and a copy of the game left playing
  for days shared the CPU and GPU; single runs gave 0.82–1.93 ms. Record a
  run on a quiet machine, never straight after the full check; kept.
- **R1, the math made ours** (flagged by R1's full check: the creature
  pack's update p50 0.08 → 0.09–0.10 ms). Real, and the price of correct
  rounding: CORE-MATH's `sin` and `cos` take about 18 ns where the
  system's took 7 (its `atan2`, `atan`, `exp` and `acos` are faster; `hypot`
  is now `√(x² + y²)`, 1 ns). Measured apart, review A (A) and R1 (B)
  benched in turn three times each: the levels' builds 2503 → 2662 ms in
  all (+6.3%: the rooms' floors 4–6 +5–9%, 20–33 ms; the Building's within
  noise), every update p50 +0.01 ms (0.03 → 0.04 on the Building, 0.06 →
  0.07 on the rooms), p99 unchanged (0.73–0.80), and the first frame, which
  sets every node's world matrix, 1–3 ms dearer (the Building's floors,
  4242 L2 6.8 → 9.6, 7 L2 8.6 → 10.6; L1 25.4 → 27.8; flagged first by R2b's
  check, at 14 ms under its load: 10.1 on a quiet machine). All within
  budget; seed 7's level 5, the slowest build, has 33 ms left of its 600
  (64 before). R2b (the anchors) costs nothing measurable: R2a and R2b
  benched in turn three times each, builds, first frames and updates the
  same within 0.4 ms. The
  kit's noise has no trigonometry, so the cost is spread over the placers'
  own geometry and painters and the engine's: if a floor nears its budget,
  look there (`sample` on `--bench`). Kept: a level now builds the same on
  every Mac.
  Flagged again by R4's full check, at its third run (update p50 on 7 L4–L6
  and 4242 L5 +0.01–0.02 ms, as above, and **7 L5's update p99 0.84 → 1.44
  ms**, which the A/B above had not shown). Review A and R4 benched in turn
  five times each on that floor: p99 0.80 → 1.58 ms every time, frames 48
  and 68 (the bench's first frame in two new zones) 2.1 and 3.8 ms where
  review A had none over 2. A count of `dmath`'s calls per frame found
  them: about 5,000 in a frame, 192,000 and 344,000 in those two, all in
  `make_ir_data`, which built a zone's reverb when it was first entered
  with an exponential a sample. The decay is now a running product (one
  exponential a channel; it drifts by 1e-11 over the longest tail, under
  what an f32 keeps; the soundscape's reference, the 17 listen tests and
  the loudness held): p99 0.79–0.87, no frame over 2 ms. The update p50
  stays +0.02 ms (0.08 → 0.10 on 7 L5: a frame's 5,000 calls). 7 L5's
  build measured 615–621 ms against 580 for review A on the loaded
  machine (570 in R4's full check): its 600 ms budget holds only on a
  quiet machine, and it is the first floor to look at if a build grows.

- **E7** (`8274df8+` in the history, flagged by E8's full check): level 4's
  build, the first floor of the rooms a run builds, 135 → 148 ms in both
  runs. The creatures' first-use setup (their shared materials and
  textures) is paid there: timing `add_entity`, the first creature of a
  process took 15 ms and the next 7, whatever the kind; before E7 part of
  it was paid on level 1 (4–6 ms dearer then). And 4242 L5's triangles,
  137k → 168k: the ecology put another creature in the camera's view
  (content). Both well within budget; kept.
- **Props P1–P6** (`2a35e3b+`, flagged by their full check): the floors of
  the rooms build slower, 4242 L5 210 → 515 ms and 7 L5 345 → 547, L4 and
  L6 by 170–260 ms; and their triangles grew (4242 L6 174k → 242k). The
  props made real are modelled parts and paint their own textures, one for
  each thing (a torn note's sheet and type, a book stack's atlas, a
  mirror's silver and film, each stain and web of the marks, the
  barricades' plywood and boards). Painted as they were, 7 L5 took 666 ms,
  over its budget. Two changes, both giving the same pixels: a radial
  gradient whose end stop is clear is shaded only inside its end circle
  (`Canvas::draw_gradient`; 25 ms), and the marks' per-pixel painters (the
  ceiling and wall stains, 15% of a build) fill their rows on every core
  (`par_rows`): 7 L5 547 ms. Painting whole canvases on a pool of threads
  was tried and dropped: CoreGraphics rasterizes a path a little
  differently while another thread rasterizes, and a golden changed from
  run to run. The rest is the content; within budget, kept.
- **The vending machine** (`49e683a+` and after, flagged by the same
  check): the update's p99 on the floors with a lit machine, 0.78 →
  1.53 ms. Its display scrolls HAVE A NICE DAY three characters a second,
  and every lit machine repainted its 350 dots (a fill and a CSS colour
  parsed for each) every third of a second, wherever you were: 0.75 ms,
  on the film grain's frames 1.5. It repaints now only within 14 m (its
  dots are a few pixels from further off; the first paint always
  happens), its two colours parsed once: the p99 back to the grain's
  0.8 ms.
- **The update's tail** (`7b9117d+` in the history, flagged by
  `MILESTONES.md` 7's full check, the third run after it): the update's
  p99, 0.76–0.81 → 0.85–1.06 ms on 7 L2, 7 L5, 4242 L2 and L3, and the
  creature pack's 0.86 → 1.23 ms (its GPU p99 2.27 → 3.2 ms), the p50s
  held. Bisected (milestone 14, below): it has two causes, and neither is
  the game's own cost. The pack's step is `03e4b80`, which stopped the
  headless modes opening the speakers. Before it, CoreAudio's output thread
  rendered the bench's audio graph. After it, the silent clock's own thread
  rendered it, though nothing heard it, and that rendering cost the main
  thread's update and the GPU their tail. The silent clock now renders
  nothing: the pack is back to 0.89 / 2.23 ms. The levels' step is the
  machine. The same code measures the same on both sides of the window
  today, 0.06–0.09 ms over what it measured on November 3. Kept.

- **The Building's small faults** (milestone 11, `0931fa3+` in the
  history: no step flagged yet, so written down before one is): the Works'
  triangles 32k → 81k on 4242 L3 and 49k → 104k on 7 L3, and 10–40 more
  draw calls on every Building floor. The Works' pipes are runs now, with
  their flanged joints, elbows back into the wall, collars and brackets,
  and the pipe itself 14-sided (8 before); the Stair Halls' lamps are
  lanterns and the Works' floodlights have hoods; the rooms off the walks
  are furnished with parts. Measured against the parent's binary in turn
  (three rounds), the builds held (37.5–42.5 → 37.9–41.6 ms) and the
  first frames moved within a millisecond (4242 L3 5.8 → 8.2 ms, 7 L2
  10.1 → 6.9). Content, far within budget; kept.
- **Code review 3** (`fa45f64+` in the history): most of that rise taken
  back. The elbows (14 × 8 segments, about 60% of it) are 8 × 4, and the
  pipe is 8-sided and open-ended (its ends are always in a flange, an
  elbow or a wall; the elbow's sides meet the pipe's): the Works' triangles
  80.9k → 52.0k on 4242 L3 and 104.3k → 72.3k on 7 L3, their draw calls
  unchanged (208, 284). The rooms' furniture shares its geometry now (a
  table's four legs one instanced draw; the stool's foot ring 4 × 12): the
  Well's draw calls 409 → 385 (4242 L1) and 333 → 325 (7 L1). On the floors
  of the rooms a contact shadow's footprint has at most 16 corners, and a
  shadow goes with its thing past its draw distance: 0.1–1% fewer
  triangles (4242 L5 224.4k → 223.4k, 7 L6 200.4k → 198.5k), 0–2 fewer
  draws. The builds and first frames held.

## What the work found

Each speed-up and measurement, as it was done: [`../plan/PERF.md`](../plan/PERF.md).

## The creatures

Beside the levels, `cargo xtask bench` times the **creature pack**
(`backrooms --bench-creatures`): on level 4 of run 4242, two of every kind
that roams the rooms (`ROAMING_KINDS`: the Smiler, the Hound, the Watcher,
the Reader), and the one in the corner, are placed 6–14 m round
you, all hunting you (god mode), and over 240 frames, while you turn once on the spot, the game
update on the CPU, the frame on the GPU, draw calls and triangles. It is
held to the `creatures` budgets below and to its own history,
`perf/creatures.csv`, by the same change-point test. The first row
(2026-10-28, before the redesign of `ENTITIES.md`): 5 creatures placed,
update 0.07 / 0.89 ms (p50 / p99), GPU 2.2 / 3.6 ms, 263k triangles.

## Recording and replaying

Last, `cargo xtask bench` times **recording and replaying**
(`backrooms --bench-replay`, `docs/engineering/REPLAYS.md`): a fresh
process at the title, New descent, the way in, then a minute (3,600
frames at 60 a second) of the restless bot on run 4242, its log kept as
play keeps it; then the session replayed headless from its start and
checked. It measures a replayed frame (what a seek costs: a minute of play
is simulated again in 60 × 60 × that), the log's work after each frame,
the log's memory a minute of play (its frames, events, kept states and
telemetry: what the game holds between recordings is one floor of it),
and the file a minute. It is held to the `replays` budgets below and to
`perf/replays.csv`. The first runs (R4, 2026-11-14): a replayed frame
0.19–0.20 ms (about 85 times real time: ten minutes are reached in about
7 s), the log 0.1 µs a frame, 253 KB a minute in memory, 16 KB a minute
in the file.

## Budgets

About twice the worst level today, and the GPU well inside a 60 fps frame
(16.7 ms) at four times the pixels (Retina). The Showroom sets the triangle
budget; a maze level that grows toward it shows first as a step in the
history. A replayed frame within 0.5 ms keeps a seek over ten minutes of
play under 20 s; the log within 0.5 MB a minute keeps a long floor's in
tens of megabytes; the file within 25 KB a minute is the format's own
target (`docs/engineering/REPLAYS.md`). Raise a budget only with a reason in the commit.

<!-- perf-budgets:begin -->
```json
{
  "levels": {
    "build_ms": 600,
    "first_frame_ms": 100,
    "update_p99_ms": 4,
    "gpu_p99_ms": 6,
    "calls_max": 2000,
    "tris_max": 2500000
  },
  "creatures": {
    "update_p50_ms": 1.5,
    "update_p99_ms": 4,
    "gpu_p50_ms": 6,
    "gpu_p99_ms": 8,
    "tris_max": 2500000
  },
  "replays": {
    "replay_frame_us": 500,
    "record_frame_us": 20,
    "log_kb_min": 512,
    "file_kb_min": 25
  }
}
```
<!-- perf-budgets:end -->
