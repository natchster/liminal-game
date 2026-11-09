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
(`docs/DEVELOPMENT.md`, "A build of an older commit plays aloud"): a build
from before `03e4b80` benches out loud.

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

## The update's tail (milestone 14, 2026-11-09)

The release binary's bench modes (`--bench`, `--bench-creatures`) are the
same functions across the window, so each commit was built and its binary
timed in rounds with the others: one run of each binary in turn, the order
reversed every round, on a quiet machine (load 2–3, no other build or
bench). The tables give each binary's update p50, the median of its p99s
with each run's p99, and the median of its GPU p99s.

**The endpoints first** (6 rounds). The pack's step is real: 2a35e3b
measured 0.081 ms, then 0.93 [1.91 0.99 0.96 0.89 0.89 0.89], then 2.23
ms. 7b9117d measured 0.126, then 1.80 [1.65 2.21 1.79 2.15 1.81 1.44],
then 5.24 ms. The levels' step is not real. Over the nine levels without a
lit vending machine the mean p99 is 0.864 ms on 2a35e3b and 0.858 ms on
7b9117d. (At 2a35e3b the machines still repainted far away, 1.6 ms on
4242 L5–6 and 7 L6. The fix was in the tree that recorded the `2a35e3b+`
rows and was committed as 6c37ebb.)

**The pack, bisected.** The window has 75 first-parent commits. Tested
first were the suspect, `03e4b80` ("Playtesting never sounds through the
speakers": the headless modes get a silent context instead of the
device), and its parent. The step lies exactly between them. `03e4b80` is
not a merge, and its only change at run time is the bench's audio
context.

| Commit | Audio in the bench | p50 | p99 (runs) | GPU p99 |
|---|---|---|---|---|
| `2a35e3b` | the device: CoreAudio's output thread renders the graph | 0.081 | 0.93 [1.91 0.99 0.96 0.89 0.89 0.89] | 2.23 |
| `bb2b1c1` (#55) | the device | 0.081 | 0.94 [0.92 0.94 0.96 0.95 0.95 0.94 0.94] | 2.25 |
| `03e4b80` (#56) | the silent clock: its own thread renders the graph, unheard | 0.108 | 1.72 [1.72 1.95 1.56 1.11 1.36 1.76 2.12] | 5.58 |
| `7b9117d` | the silent clock | 0.126 | 1.80 [1.65 2.21 1.79 2.15 1.81 1.44] | 5.24 |

The draw calls and triangles are the same on both sides, and so are the
process's CPU time and context switches. A sample (`sample` on a long
pack run) showed the device's thread rendering about 23% of a core and the
silent clock's thread about 12%. The cost is in the update and the GPU,
not in the audio thread.

**Why, on HEAD.** One build of `a443d89` had its silent clock switched by
an environment variable (8 rounds):

| Silent clock | p50 | p99 (runs) | GPU p99 |
|---|---|---|---|
| renders the graph (as `03e4b80` to `f0b82aa`) | 0.087 | 1.29 [0.98 1.28 1.30 1.08 1.51 1.42 1.35 0.92] | 3.13 |
| renders it, paced to the wall clock | 0.089 | 1.19 [1.19 1.19 1.13 1.22 1.24 1.24 1.16 1.20] | 2.83 |
| renders it, at user-interactive QoS | 0.086 | 1.18 [1.68 1.20 1.31 1.16 0.99 0.95 1.45 0.91] | 3.13 |
| keeps time, renders nothing | 0.079 | 0.90 [1.00 0.89 0.91 0.88 1.27 0.89 0.91 0.88] | 2.24 |
| no thread at all | 0.084 | 0.97 [0.96 0.94 1.28 0.92 0.98 1.02 1.01 0.97] | 2.24 |
| **this commit** | 0.080 | 0.92 [0.93 0.92 0.97 0.91 0.92 1.27 0.90 1.31] | 2.23 |

On an ordinary thread, rendering the graph costs the tail however it is
paced or prioritized. Rendering it nowhere takes the cost away. Before
`03e4b80`, CoreAudio's real-time output thread rendered the same graph
without the cost. How an ordinary thread's rendering on another core slows
the main thread and the GPU is not known. The pack shows it most: nine
creatures hunt and send many one-shots, and the bench runs the game about
five times faster than the audio clock. Nothing hears a silent context,
and nothing reads its graph back (the engine only takes commands and
gives back its clock). So `Engine::silent_clock` now keeps the clock
moving as a device would ask for frames: 10 ms at a time, the clock at the
end of the last whole quantum begun, as `fill` leaves it. It drops what
the game sends. Tests and headless runs keep the same clock; no draw,
reference level or golden moves.

**The levels.** The `2a35e3b+` rows were recorded with 6c37ebb's tree.
Built with the speaker guard, 6c37ebb measures the same as 7b9117d today
(6 rounds, each binary's mean p99 over the twelve levels):

| Binary | mean p99 | 4242 L5 | the Building (L1–3) | pack p50 / p99 / GPU p99 |
|---|---|---|---|---|
| `6c37ebb` + the guard | 0.878 | 0.934 | 0.858 | 0.088 / 1.28 / 3.67 |
| `7b9117d` | 0.874 | 0.879 | 0.865 | 0.090 / 1.25 / 3.24 |
| `f0b82aa` | 0.896 | 0.889 | 0.902 | 0.091 / 1.24 / 3.05 |
| `f0b82aa` + this commit | 0.852 | 0.878 | 0.829 | 0.081 / 0.93 / 2.24 |

On November 3 that tree's runs gave 0.79 (0.77–0.85). A level's p99 is
its film grain's frames: every fourth frame repaints 480×300 at one draw
a pixel, about 0.75 ms. So the p99 moves with the machine's state (load,
heat), and here it moved 0.06–0.09 ms with no code changed. The old tree
with the guard taken in also renders on the silent clock, so its pack is
as slow as `7b9117d`'s. Rendering on the silent clock also put
occasional runs of slow grain frames on the Building's floors (7 L3 1.20
ms on `f0b82aa`, 0.82 ms with this commit). The recorded run after this
commit: levels 0.81–0.91 ms, the pack 0.08 / 0.89 / 2.23 ms (was
0.086 / 1.24 / 4.34 at `6efc5ef+`).

To bench an old commit against today's, take in this commit's
`Engine::silent_clock` along with the speaker guard. Otherwise the old
build renders on the silent clock as `03e4b80` did, and its pack measures
slower than HEAD for that alone.

## What the first run measured (2026-10-25, Apple Silicon, macOS 27)

| Measure | Range over the twelve levels |
|---|---|
| level build | 98–257 ms |
| load + first frame | 9–25 ms |
| update p50 / p99 | 0.06–0.10 / 0.75–1.27 ms |
| GPU p50 / p99 | 0.74–1.34 / 1.38–2.88 ms |
| draw calls (max) | 604–1,167 |
| triangles (max) | 110–190 k; the Showroom (4242 L4) 1.31 M |

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

## Budgets

About twice the worst level today, and the GPU well inside a 60 fps frame
(16.7 ms) at four times the pixels (Retina). The Showroom sets the triangle
budget; a maze level that grows toward it shows first as a step in the
history. Raise a budget only with a reason in the commit.

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
  }
}
```
<!-- perf-budgets:end -->
