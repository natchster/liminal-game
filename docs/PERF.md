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
that roams the rooms, and the one in the corner, are placed 6–14 m round
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
