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
  - core props → the `core_props_head` / `core_props_tail` fixtures
- **Zones have one purpose each** (`world/purpose.rs`); themed placers only
  see floor whose purpose welcomes their theme (`THEME_COMPAT`,
  `PURPOSE_EMPTY`). A new placer goes into `THEMED_ORDER` and
  `PLACER_THEME` in `world/tables.rs`.
- **Every level stays walkable.** Archgen walls up unreachable pockets on the
  tile grid; keep that true, and keep props off the paths a body needs.
- **Gates:** `tests/level_sweep.rs` (walkable, reachable, repeatable) and
  `cargo xtask range --look` (the spread of levels against its baseline,
  the purposes against their `STYLE.md` bands). Accept a new range baseline
  only after reading `target/xtask/range/report.md`.
- **Evidence:** `--dump-level DIR --seed S --levels … --screenshot x --views`
  before and after, and look at the zone views.
