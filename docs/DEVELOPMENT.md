# Development guide

## Workspace conventions

* **Layout.** A virtual workspace: every crate lives under `crates/`, named
  after its directory (the placer crates are grouped in `crates/placers/`),
  and project automation is the `xtask` crate. `cargo run` builds and runs
  the game (`default-members`); use `--workspace` for everything.
* **Shared settings** are declared once in the root `Cargo.toml`:
  `[workspace.package]` (version, edition, MSRV, license),
  `[workspace.dependencies]` (every external and internal dependency, used
  as `foo = { workspace = true }`), and `[workspace.lints]` (each crate opts
  in with `[lints] workspace = true`).
* **Automation in Rust.** `cargo xtask bundle` builds and signs the app; add
  new tasks to `xtask/src/main.rs` rather than shell scripts.
* **Layering.** `engine` never depends on `game`; `game` never touches the
  window, Metal or CoreAudio devices directly; the executable glues them.
  This keeps the whole game runnable headless, which is what the tests do.
* **Modules.** One subsystem per folder in `crates/game/src` (see
  `ARCHITECTURE.md`); a folder's `mod.rs` documents what it owns. Game logic
  is written as `impl Game` blocks in the module that owns it, so `Game`
  stays one plain struct of state.
* **Formatting and lints.** `cargo fmt` (settings in `rustfmt.toml`) and
  `cargo clippy --workspace --all-targets`; both should be clean before a
  change goes in.

## Determinism rules

Levels must be a pure function of `(run_seed, level)`:

* Inside a level build, take randomness only from `rnd()` and its helpers
  (`rand1`, `rand2`, `irand`, `choice`). Never use another RNG, the clock,
  hash-map iteration order, or anything left over from the previous level.
* Code that runs per frame, or caches results across levels, must draw from
  the native stream: `with_native(|| …)` or `native_rnd()`. `canvas_tex`
  already switches to it while drawing.
* Keep draw order stable: a new random draw in a placer shifts every later
  placer. If you change a placer on purpose, re-record the reference levels
  (below) and say so in the commit.
* Use `base::numeric` where exact results matter (`round_half_up`,
  `nan_min` / `nan_max`, `hypot2` / `hypot3`, `fmt_fixed`, `round_decimals`).

## Adding a placer

1. Write `pub fn place_things(g: &mut Game, dist: &[i32])` in a placer crate
   (`use game::prelude::*;` brings the scene, geometry, material, kit and
   random helpers). Pick candidate tiles from `g.tiles` / `dist`, build the
   meshes, add them to `g.world`, and register animators (`g.animate`) or
   interactables (`g.interact`) if it moves or can be used.
2. Register it in the crate's `register()` under its name
   (`"place_things"`).
3. Add the name to `THEMED_ORDER` in `crates/game/src/world/tables.rs` at the
   point where it should run, and its theme to `PLACER_THEME`.
4. `cargo test -p placers_pNN` runs the crate's smoke tests (12 seeds × 24
   repetitions, every animator and interactable exercised).

## Tests and reference data

* `cargo test --workspace --release` runs everything.
* `crates/game/tests/fixtures/` hold recorded reference data for the game's
  subsystems; the tests compare against it with tight tolerances. After an
  intended change to the creatures' builds, animation or behaviour,
  `REGEN=1 cargo test --release -p game --test entities` re-records the
  affected sections of `entities.json` (keeping their inputs); review the
  diff. The creatures' senses and the director also have behaviour tests in
  `tests/entities.rs`.
* `crates/backrooms/tests/reference/` hold 20 recorded levels. If a change is
  meant to alter level generation, rebuild those levels with
  `backrooms --dump-level DIR --seed S --levels …` and review the
  difference before replacing them.
* The level-generation fixtures re-record themselves from the current code
  (keeping each case's seeds and query points); run the ones a deliberate
  change affects, then review the diff:
  `cargo test --release -p game --test archgen --test world_queries --test world_build -- --ignored rerecord`
  and `cargo test --release -p backrooms --test reference_levels -- --ignored rerecord`.
* `crates/game/tests/zone_purpose.rs` checks the zone purposes and regions
  as properties (one purpose per area, themed placers only see welcoming
  floor, regions never cut through a room, determinism).
* `crates/backrooms/tests/playthrough.rs` plays whole runs; run it with
  `--nocapture` to see the per-level walking times.
* `crates/backrooms/tests/pacing.rs` plays levels with death on, with a bot
  that behaves like a cautious player, and logs the director's phases
  (`--nocapture` prints each run's calm → build → peak → relax sequence,
  close calls and deaths). The creatures' per-frame decisions draw from the
  native stream, so runs differ; its thresholds leave room for that.
* `crates/backrooms/tests/listen.rs` renders the game's audio offline and
  writes each run to `target/listen/*.wav` with a loudness timeline
  (`*.png`, dB per 50 ms window, red marks at events): listen to them after
  a change to the sound. Its checks are measured against the room's bed.

## Profiling and debugging

* `backrooms --screenshot out.png --level L --seed S --play --frames N`
  renders headless and prints draw calls, triangles, lights and pipeline
  counts; add `--scale 2` for Retina resolution.
* `backrooms --dump-level DIR --seed S --levels 1,2 --screenshot x --views`
  also renders one PNG from inside each zone (`rs_S_LN_zNN.png`); the JSON
  lists each zone's purpose and the regions.
* `--showcase smiler[:dist]` (or `hound`, `watcher`) keeps a creature
  standing in front of the camera for the shot; add `--death` to be killed
  by it 15 frames before the shot (the death scene).
* The K key shows the in-game debug panel (positions, timers, the ambient
  log, entity states).
* `cargo run --release -p engine --example perftest` measures the renderer
  alone; `--example rendertest` renders a reference scene to PNG.
