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
* **Automation in Rust.** `cargo xtask bundle` builds and signs the app and
  `cargo xtask check` runs the quality gate; add new tasks as modules of
  `xtask/src/` rather than shell scripts.
* **Agent guidance.** `CLAUDE.md` is the short map for coding agents (commands,
  invariants, definition of done); `.claude/rules/*.md` hold rules that load
  only when files under their `paths:` are touched. Keep both short, and move
  a rule into a test or lint when it can be checked mechanically.
* **Hooks.** `.claude/settings.json` wires Claude Code's hooks to
  `cargo xtask hook <event>` (through `.claude/hooks/xtask-hook.sh`, which
  calls the built binary directly so edits never wait on cargo's build lock):
  * before an edit: the HTML/JS version in the parent folder and the recorded
    fixtures, reference levels and goldens are refused — they change only
    through REGEN / rerecord / accept;
  * after an edit: rustfmt on the edited `.rs` file;
  * on stop: `cargo xtask check --fast` when code differs from the last
    passing check; a failure blocks the stop with its `ERROR` lines, at most
    three times in a row;
  * after compaction: recent commits, uncommitted files and the roadmap's
    progress table are re-injected (`CLAUDE.md` itself is re-read anyway).
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

* `cargo xtask check` is the gate: `cargo fmt --check`, clippy with
  `-D warnings`, the whole test suite and the golden frames (`shots`), each
  step's output written to `target/xtask/check/<step>.log` and summarised as
  one `ERROR <step> …` line per problem. `--fast` skips the slow suites that
  play levels for minutes (`pacing`, `listen`, `playthrough`, `loudness`,
  the material lint); `--only fmt,clippy` runs a subset.
* `cargo test --workspace --release` runs every test directly.
* **Golden frames.** `cargo xtask shots` renders 31 fixed frames headless
  with `--fresh` (each creature at 2, 4 and 8 m and its death scene; the
  spawn view of five levels and one view per zone purpose, which together
  cover every purpose, weather and anomaly) and compares them with
  `crates/backrooms/tests/golden/`. With `--fresh` a frame is the same bit
  for bit on the same machine, so identical pixels pass; otherwise ꟻLIP
  reports the mean, 99th percentile and worst 32×32 tile and writes an error
  map to `target/xtask/shots/<name>-diff.png`. Each frame's look statistics
  (colourfulness, luminance, contrast, deep-shadow share) are held to
  anchors in `look.json`, frame by frame and as a median over the set (a
  change of light moves every frame a little). After an intended change,
  look at the frames and error maps, then
  `cargo xtask shots --accept NAME,…|all --reason "…"`; look anchors move
  only with `--accept-look … --reason "…"`, so re-accepting frames never
  hides a slow slide toward bright, saturated frames. Every acceptance is
  logged in `golden/ACCEPTED.md`. Goldens bind to the Mac model and macOS
  version in `shots.json`; elsewhere differences are advisory. (They are
  480×300 and live in the repository; an unchanged frame adds nothing to the
  history.)
* **Material lint.** `crates/backrooms/tests/materials.rs` builds six levels
  and checks every lit, opaque, non-glowing material's effective albedo
  (colour × mean of its map, linear): brightest channel 0.008–0.87,
  saturation ≤ 0.985, and no opaque material on a mostly clear map. The
  props' look pass (`visuals/prop_look.rs`) keeps settled props inside; the
  lint guards everything else and the pass itself. Exceptions live in
  `EXEMPT` with a reason; `--nocapture` prints the distribution.
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
* `crates/backrooms/tests/loudness.rs` measures three level renders to EBU
  R128 (the `ebur128` crate): integrated loudness within 1 LU and loudness
  range within 3 LU of `tests/recorded/loudness.json`, true peak at or
  below −1 dBTP, and no one-shot ambient event twice within 8 s. The
  per-frame randomness is seeded, so the numbers repeat exactly. The mix sits
  near −30 LUFS integrated, quiet by design (published game targets of −23
  or −18 LUFS assume dialogue and music); the gate holds it where it is.
  After an intended change to the mix, listen to the `listen.rs` renders,
  then `cargo test --release -p backrooms --test loudness -- --ignored rerecord`.

## Profiling and debugging

* `backrooms --screenshot out.png --level L --seed S --play --frames N`
  renders headless and prints draw calls, triangles, lights and pipeline
  counts; add `--scale 2` for Retina resolution.
* `--fresh` (headless modes) ignores the saved settings and progress and
  seeds the per-frame randomness, so the same command renders the same frame
  bit for bit. Headless runs never write the save file.
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
