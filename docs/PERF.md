# Performance

Measure, then change. `cargo xtask bench` times twelve levels (runs 4242 and
7, levels 1–6) headless at 1280×800: the level build, the load plus first
frame (pipeline compiles land there), and 240 frames along a fixed path
through every zone — the game update on the CPU and the frame on the GPU
(Metal's command-buffer GPU start and end times), with draw calls and
triangles. Each measure is the median of three repeats.

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
  }
}
```
<!-- perf-budgets:end -->
