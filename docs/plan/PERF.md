# The work on performance

What each speed-up and each measurement found, as it was done. The budgets
the bench holds the game to, and how to measure, are
[`../engineering/PERF.md`](../engineering/PERF.md).

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
gives back its clock). So the silent clock now keeps the clock moving as
a device would ask for frames: 10 ms at a time, the clock at the end of
the last whole quantum begun, as `fill` leaves it, and what the game sends
is dropped. Tests and headless runs keep the same clock; no draw,
reference level or golden moves. (Made in `2b6e022` as
`Engine::silent_clock`; since `833db43` it is the free `silent_clock`,
started by `CtxInner::run_silent`, and what is sent is dropped before it
is queued, in `CtxInner::send`.)

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

To bench an old commit against today's, take in `2b6e022`'s silent clock
(`git cherry-pick -n 2b6e022`: there `Engine::silent_clock`, which keeps
time and renders nothing; today the free `silent_clock`, started by
`CtxInner::run_silent`, with what is sent dropped in `CtxInner::send`)
along with the speaker guard (`03e4b80`). Otherwise the old build renders
on the silent clock as `03e4b80` did, and its pack measures slower than
HEAD for that alone.

## The light map's rebuilds (code review 4, 2026-11-10)

The light map is rebuilt whenever a lamp goes out or comes back
(`update_lights`), a vending machine's light follows its rhythm, or her
lamp gutters out; on the Building every lamp the fog passes goes out as it
rises and comes back as it falls (`update_fog_surface`): 32 rebuilds over
a rise and fall on 4242 L1, 18 on L3. Each rebuild made the floor under
every texel (64 a tile, `floor_y_at` each) and hashed the level's shape
anew, though neither changes within a level. The floors, their base and
the shape are made once a level now (`occlusion::LightShape`, dropped by
`build_world`), and a rebuild repacks the light. Timed by a scratch timer
around `build_light_map` over a fog pass (the fog from a metre under the
lowest lamp to a metre over the highest and back, 400 steps each way) and,
on the rooms, a panel switched 40 times; release build, the mean (median)
of a rebuild:

| Level | Rebuilds | Before | After |
|---|---|---|---|
| 4242 L1, the Well (28 × 28 tiles, 41 lamps) | 32 | 0.389 (0.381) ms | 0.152 (0.148) ms |
| 4242 L3, the Works (22 lamps) | 18 | 0.370 (0.357) | 0.148 (0.148) |
| 7 L1 | 28 | 0.367 (0.358) | 0.150 (0.146) |
| 7 L3 | 28 | 0.375 (0.376) | 0.149 (0.146) |
| 4242 L9, the rooms (53 × 53) | 40 | 1.219 (1.200) | 0.570 (0.552) |
| 4242 L5, the rooms (39 × 39) | 40 | 0.669 (0.664) | 0.300 (0.293) |

The bytes of every rebuild (hashed in order) were the same before and
after on all six, measured before the same commit gave the Building's B
the floors' gaps; `tests/building_floors.rs`,
`the_light_maps_shape_is_made_once_and_kept`, holds it since: at each
rebuild of a fog's rise and fall the kept shape packs what one made from
nothing does. The bench recorded with it is within budget with no step;
the Building's first frames 7.5–9.7 ms on L2–3 and 23.8–25.8 on L1, as
before (no texture read was added: the floor's B is read from the sample
the shader already takes).

## What the first run measured (2026-10-25, Apple Silicon, macOS 27)

| Measure | Range over the twelve levels |
|---|---|
| level build | 98–257 ms |
| load + first frame | 9–25 ms |
| update p50 / p99 | 0.06–0.10 / 0.75–1.27 ms |
| GPU p50 / p99 | 0.74–1.34 / 1.38–2.88 ms |
| draw calls (max) | 604–1,167 |
| triangles (max) | 110–190 k; the Showroom (4242 L4) 1.31 M |
