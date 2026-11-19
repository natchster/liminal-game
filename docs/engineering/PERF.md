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

- **The assets, A2-A7** (issue 13's milestones, after `5946106+`, the last
  run recorded: no run recorded since, so written down before a step is
  flagged). 208 placers gone, a pass that places things by their rules
  after the floor settles, and the new almond water and battery (a bottle
  about 6.8k triangles in four meshes, see-through). Measured on
  2026-11-16 with two virtual machines running beside it (about 2.7 cores
  and 40 GB between them), so in turn, three rounds, the order reversed
  each round: `5946106`, A6 (`e7fd09e`) and A7's tree. On that machine
  every binary's GPU p99 was 4-8 ms on the rooms' floors and the Well (the
  old one's too: 6.9 ms on 7 L1, recorded at 2.4 the day before), over the
  6 ms budget: the machine's, not the work's, and no run is recorded until
  it is quiet. Against `5946106`, A7: GPU p50 the same within 0.1 ms;
  builds -8 to -11% on 7 L4-L6 (fewer placers) and +6 to +10% on 4242
  L5-L6 (the rules pass); peak triangles +26k on 4242 L5 (223k → 249k,
  the bottles in view) and -7k on 4242 L4, far within the budget. Against
  A6, A7 the same or lower on 11 of the 12 GPU p99s (one shared water
  copy, no render order). Kept; record a run on a quiet machine.
- **The facility's F5** (`a2d5e30+` in the history, flagged by F7's full
  check, the third run after it): peak triangles on level 6, the Utility
  Halls, 202k → 437k (7 L6) and 243k → 499k (4242 L6). The tunnels' lining
  is meshes now (the haunches at their crease, each a merged mesh with its
  own UVs and its row of bolts) and the runs on the straight corridors'
  crown (pipes on hangers, couplings, cable trays and their risers). Draw
  calls +50 on those floors, the GPU p50 as it was (1.07 → 1.12 ms, 1.04 →
  1.16); far within the 2.5M triangle budget. Kept.
- **F7, the drops** (not recorded: on 2026-11-18 the machine ran another
  session's Blender at eight cores and a virtual machine, load 38; record
  a run on a quiet one). Against `e2949cc`, the counts, which no load moves:
  every floor of the bench the same but 7 L6, the one with a drop (of
  water). There, draw calls 566 → 864 and triangles 437k → 401k: the drop's
  own are a handful (its sides, bottom, water and two instanced railings),
  the rest the floor's placement drawn anew round the drop's tiles (meshes
  1733 → 1817), seen differently by the bench's cameras; 864 is 7 L4-L5's
  876-913, within the 2000 budget.
- **F9, the pit hall** (not recorded: on 2026-11-18 a virtual machine and
  other sessions held the machine, load 25). The bench's floors (1-6) have
  no pit hall; what reaches them is the bulkheads closing a step between
  two walls where either touches open floor (3 to 165 quads a floor of
  the `world_build` fixture's, instanced, in groups already drawn but
  where a new surface's group appears: a draw call). Its full check
  flagged F5's step again, named `cb0d9c5+` this time (the best split moves
  as runs are added): the same peak triangles on level 6, explained above
  (7 L6 401k and 4242 L6 499k, as F7's tree). Record runs on a quiet
  machine.

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
A body may be detailed close and cheap far (E14): from 6 m out its far
meshes are drawn (`Node::lod`; the Hound 101k triangles near, 22.7k far),
and the skin's relief takes three texture reads a pixel (nine before). The
pack drew 438k triangles at most (559k without the far bodies), its GPU
p50 1.86–2.18 ms as before (the frame is bound by its pixels, not its
vertices); brought within 1.5–3 m of you, the skin's three reads were
1.78–1.89 ms against nine's 1.96–1.99.

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

## The way in

And **the way in** (`backrooms --bench-tunnel`, `docs/design/WAY_IN.md`):
the Jahamun Tunnel entered as New descent enters it (the place built and
shown, its first floor already built behind it), its first frame, and 240
frames over seven views, each turned a little a frame (the street above,
the plaza, the stairs' head, a few bays in, deep in looking on and looking
back, the open door), held to the `tunnel` budgets and to
`perf/tunnel.csv`. The place is far heavier than a floor of the Building:
its surfaces are baked meshes fine enough to hold their light (8-25 cm),
106 modules of them built for the ring, a highlight pass over the wet
paint and the glaze, and its own lights.

What the first measures found (W7, 2026-11-19, 1280×800, on a machine
other sessions kept at a load of 40-100, so the absolute times move with
it and the ratios are the measure): 1.40 M triangles and 1,608 draw calls
at most, GPU 5.0 / 9.7 ms (p50 / p99); the plaza's view the worst (p99
11.9 ms). Two things were let go:

- a module's highlights past 60 m (`HIGHLIGHT_REACH`): there a glint is a
  pixel or two, and its pass draws the densest meshes again: 1.40 M → 1.15
  M triangles, 1,608 → 1,470 calls;
- the point lights of the LED strip under the balustrade's rail (54 of
  them, its glow kept): every pixel of the plaza's highlight looped over
  all of them; the plaza's view went from the dearest to below the
  walkway's.

After them, measured in five pairs with the Well (4242 L1) run in turn, so
each pair shares the load: the way in's GPU 4.8 / 9.0 ms (p50 / p99)
where the Well's is 2.8 / 5.3, about 1.7 times a floor of the Building,
inside a 60 fps frame at the worst; its update 0.03-0.13 ms at p50 (the
ring, the voice and the set's look for the eye); entering it 0.2 s with
the machine lighter, up to 1.8 s with it loaded (the set file inflated,
its meshes and photographs made: under the black of New descent). Its
history's first row waits for a run of `cargo xtask bench` on a quiet
machine.

## Budgets

About twice the worst level today, and the GPU well inside a 60 fps frame
(16.7 ms) at four times the pixels (Retina). The Showroom sets the triangle
budget; a maze level that grows toward it shows first as a step in the
history. A replayed frame within 0.5 ms keeps a seek over ten minutes of
play under 20 s; the log within 0.5 MB a minute keeps a long floor's in
tens of megabytes; the file within 25 KB a minute is the format's own
target (`docs/engineering/REPLAYS.md`). The way in's are the same kind
of margin over what it measured with the machine loaded (its entry is
under the black of New descent). Raise a budget only with a reason in the commit.

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
  },
  "tunnel": {
    "enter_ms": 2000,
    "first_frame_ms": 600,
    "update_p99_ms": 4,
    "gpu_p50_ms": 8,
    "gpu_p99_ms": 14,
    "calls_max": 2500,
    "tris_max": 2500000
  }
}
```
<!-- perf-budgets:end -->
