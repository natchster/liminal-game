---
paths:
  - "crates/game/src/world/**"
  - "crates/game/src/level/**"
  - "crates/game/src/core_props/**"
  - "crates/game/src/placers/**"
  - "crates/game/src/modes/**"
  - "crates/game/src/anomalies/**"
  - "crates/placers/**"
---

# Level generation rules

- **Determinism.** Inside a build, draw only from `rnd()` / `rand1` / `rand2` /
  `irand` / `choice`. Per-level choices that must not shift the build's
  stream use a `SplitMix64` seeded from `current_seed` with a salt (see
  `SIG_SALT`, `BED_SALT`). Never the clock, `thread_rng`, or `HashMap`
  iteration order. Building `(seed, level)` must give the same result
  whatever level was built before it.
- **Draw order is part of the output.** Adding, removing or reordering a
  random draw shifts every later placer. If that is intended, re-record and
  say so in the commit:
  - archgen / world layout → `cargo test --release -p game --test archgen --test world_queries --test world_build -- --ignored rerecord`
  - anything that changes the built level → `cargo test --release -p backrooms --test reference_levels -- --ignored rerecord`
  - core props → `cargo test --release -p game --test core_props_head -- --ignored rerecord`
    (their second half: `--test core_props_tail -- --ignored rerecord`)
- **Zones have one purpose each** (`world/purpose.rs`); themed placers only
  see floor whose purpose welcomes their theme (`THEME_COMPAT`,
  `PURPOSE_EMPTY`). A new placer goes into `THEMED_ORDER` and
  `PLACER_THEME` in `world/tables.rs`.
- **Walls are not all on the grid.** A partition may lean (`world::angled`):
  in `g.solids` it is a staircase of `BOX_ANGLED` steps inside the drawn
  wall, and `g.angled` holds the wall as drawn. Ask the solids for a wall's
  real face (`ray_axis`, `circle_hits`), never the tile grid; a placer that
  hangs a thing at a tile edge is moved onto the face, and turned with it, by
  `settle_props`. Set the solids with `set_solids_now`, which keeps the two
  in step.
- **Every level stays walkable.** Archgen walls up unreachable pockets on the
  tile grid; keep that true, and keep props off the paths a body needs.
- **Gates:** `tests/level_sweep.rs` (walkable, reachable, repeatable) and
  `cargo xtask range --look` (the spread of levels against its baseline,
  the purposes against their `STYLE.md` bands). Accept a new range baseline
  only after reading `target/xtask/range/report.md`.
- **Evidence:** `--dump-level DIR --seed S --levels … --screenshot x --views`
  before and after, and look at the zone views; after a major change, the
  visual playtest of the levels it touched (`visual-playtest`).
