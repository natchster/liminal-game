# placer-diff: Rust placers vs. the original JS

Checks that every ported prop placer (`placers/pNN`) does exactly what its
original in `backrooms.js` does: the same number of seeded `Math.random()`
draws, the same scene graph, the same `*Positions` lists and interactables, the
same scene after animator frames and after every interactable is used.

## Running it

```
cd native
node tools/placer-diff/run.mjs all                 # every crate, 12 seeds per (level, weather)
node tools/placer-diff/run.mjs p03,p30 --seeds 100 # some crates, more seeds
node tools/placer-diff/run.mjs p28 --only placeFloorBooks --seeds 300
node tools/placer-diff/run.mjs all --seeds 100 --coverage   # also report JS lines no case executed
```

It builds `placers_difftest` in release mode first (`--no-build` skips that), runs
crates in parallel (`--jobs K`) and prints one row per crate. Exit code 0 means no
mismatch. Per-crate logs go to `--out DIR` (default `$TMPDIR/placer-diff`):
`DIR/<crate>.log` has one line per placer and, for each failing placer, the
first differing cases with the first differing line on each side.

To look at whole cases, pass the case numbers from the log to `--dump`, with the
same `--seeds`, `--levels`/`--out` and `--only` as the run that reported them:

```
node tools/placer-diff/run.mjs p30 --seeds 100 --dump 12718,12801
```

Options: `--seeds N` (per level and weather), `--sizes 21,29,37` (synthetic grids,
`--sizes ''` for none), `--levels FILE` (grid file, see below), `--synthetic-only`
(no archgen grids), `--no-uses`, `--details K` (cases printed per failing placer),
`--jobs K`, `--out DIR`, `--coverage`, `--strict` (no noise tolerance: every textual difference fails). Environment: `BACKROOMS_REPO` (the repo
with `backrooms.js`, `vendor/three.module.min.js`, `src/archgen.js`; default
`/Users/nohwan/Documents/Personal_Projects/OpenCarole`), `CARGO_TARGET_DIR`.

A full run (`all --seeds 100`, 3.9M cases) takes about a minute on 12 cores; with
`--coverage` about two; `all --seeds 300` (11.7M cases) about four.

`cargo test -p placers_difftest` runs the Rust half alone (every placer runs
without panicking and a second pass in the same process gives the same hashes);
it is part of `cargo test --workspace`.

## How it works

* **Cases.** For every placer, for every grid, for every weather (all six
  `WEATHER_BIASES`), for every seed `s`, `seed = s * 2654435761 ^ level << 24 ^
  weather << 20 ^ 0x01234567`. Grids: three synthetic ones
  (`Game::test_level`) and the grids in the level file, which `levels.mjs` makes
  with the game's own generator (`src/archgen.js`, levels 1..9, sizes 35..53),
  two of them with half the zones hidden from the placer (tiles = 2), as
  `themed()` does. `dist` is the BFS from (1,1), taken before masking, as in
  `buildWorld()`. Odd seeds start with two positions in every `*Positions` list
  (as if earlier placers had run), even seeds with empty lists. Both sides run
  the cases in the same order in one long-lived process, so module-level caches
  behave as in a browser session.
* **One case.** Reset the per-level state `buildWorld()` resets (world,
  animators, interactables, every `*Positions` list, panels, papers, light
  switches, ...), the player, `perfT`, `currentSeed = seed`, the timers. Then:
  1. run the placer under `withSeed(seed)` and count the seeded draws; describe
     the world (every object, recursively), the `*Positions` lists, the
     interactables, `gameplayR1State` and any tile the placer changed;
     entries of a `*Positions` list that carry more than x/y/z in JS (e.g.
     `{ x, y, z, period }`) are compared with the Rust `Vec<f64>` field whose
     doc comment in `game/src/positions.rs` reads `` `fooPositions[i].period` ``
     (picked up by `placers/difftest/build.rs`); a JS field with no such list
     is reported as `!! JS position fields with no Rust counterpart` and fails
     the run;
  2. run six animator frames (`dt`/`t` from 1/60 s to 16 s, `perfT = t`) under
     another seed; describe the world after frames 1, 3 and 6;
  3. twice: use every enabled interactable, then one frame; describe the world,
     interactables, player and last message; then fire every pending
     `setTimeout` and describe again.
* **Canonical text.** One line per object: kind, name, position, Euler angles +
  order, quaternion, scale, visibility, render order, frustum culling, matrix
  auto-update; for meshes the geometry type and parameters (`GeoParams`),
  vertex and index counts, sums of the position/normal/uv/colour attributes and
  a position-weighted sum, group count, and per material kind, colour,
  emissive, opacity, transparency, side, depth flags, blending, alpha test,
  vertex colours, fog, visibility, polygon offset, map (repeat, offset,
  rotation, wrap), alpha/bump/emissive maps, Phong specular/shininess, point
  size, sprite rotation; instancing (count and matrix sums); for lights colour,
  intensity, distance, decay, angle, penumbra, ground colour, target. Numbers
  are quantised to 1e-5 and written `~N` (`Math.floor(v * 1e5 + 0.5)` on both
  sides). The Rust side is `placers/difftest/src/lib.rs`, the JS side
  `canon.mjs`; they must stay byte-identical.
* **Comparison.** The Rust binary (`placers/difftest`, `placer-diff <crate>`)
  prints per case the draw counts and a 64-bit hash of each section. The node
  side (`run.mjs`) runs the original JS under the vendored three.js r160 and
  compares. For mismatching cases it re-runs Rust with `--detail` to get the
  full text and prints the first differing line. Differences that are only
  rounding noise are counted as `noise`, not as failures: a quantised number off
  by at most 3e-5, a colour channel off by 1/255, or the same orientation
  written differently (Euler angles 2π apart and the quaternion negated; see
  "Known engine differences").
* **The JS side.** `jsbundle.mjs` lexes `backrooms.js` into top-level
  statements and pulls in each placer with every top-level declaration it
  references, transitively (tables, materials, helpers, `wx`/`wz`, `player`,
  ...). Stand-ins: `PG` is the identity (props off, as in the Rust build),
  `canvasTex` returns an empty texture without drawing (texture contents are not
  compared; in the game they draw on the native RNG, so they never touch the
  level's stream), `showMsg` records the text, `setTimeout` is a virtual timer.
  `globalThis.__threeUuidRandom` is the unseeded `Math.random`, as in the game.
* **Coverage.** With `--coverage` the workers run under `NODE_V8_COVERAGE` and
  `coverage.mjs` lists, per placer, the JS lines (and branches inside a line)
  that no case executed: code the comparison did not check. The full run
  executes all but 13 of 32105 placer lines; what is left is unreachable in the
  game or audio-only (`?? TABLE.normal` fallbacks for unknown weathers,
  `if (AC) blip(...)` with no audio context, a `THREE.OctahedronGeometry ?`
  feature test).

## Protocol of `placer-diff`

`placer-diff <crate> [--seeds N] [--sizes a,b] [--levels FILE] [--only a,b] [--detail i,j] [--no-uses]`,
`placer-diff --list`. Lines on stdout, tab-separated:

* `H <json>`: crate, placers, weathers, `*Positions` names, frames, grids;
* `C idx placer level weather s seed drawsBuild hashBuild drawsFrames hashFrames drawsUses hashUses`;
* `P idx placer level weather s seed <json message>`: the Rust placer panicked;
* `D idx <json build> <json frames> <json uses>`: full texts (for `--detail`);
* `E count`.

Level file (`levels.mjs`): one grid per line, `w h tiles dist`, tiles one digit
per tile (row-major), dist comma-separated.

## Known engine differences (not placer bugs)

* `f64::sin/cos/tan/atan2/asin` (macOS libm) differ from V8's `Math.*`
  (fdlibm) in the last ulp for a few percent of inputs
  (`cargo run --release -p placers_difftest --example trig_repro > t.txt;
  node tools/placer-diff/repro/trig_repro.mjs t.txt`). Positions differ by
  ~1e-16, which the quantisation hides, but a value that should be zero can come
  out ±tiny, so `atan2` lands on the other side of ±π: after `Object3D.clone()`
  (quaternion → Euler) a rotation can read `z = π` in one build and `-π` in the
  other, with the quaternion negated. Same orientation, so it is classed as
  noise (`placeFloorBooks` shows it in ~0.1% of its cases;
  `--example euler_repro` + `repro/euler_repro.mjs` measures it).
