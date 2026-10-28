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
* **Skills and reviewers.** `.claude/skills/<name>/SKILL.md` are the
  content playbooks (zones, props, creatures, anomalies, sounds, pacing,
  blind visual review, re-recording); `.claude/agents/*.md` are read-only
  reviewers (`art-critic` with a checked-in memory of people's verdicts in
  `.claude/agent-memory/art-critic/`, `lore-keeper`, `level-analyst`,
  `audio-analyst`, `code-reviewer`). `tests/structure.rs` checks that the
  paths they name exist.
* **Blind reviews.** `cargo xtask ab prepare --before DIR --after DIR`
  writes every image pair twice, once in each order, under shuffled names
  (`target/xtask/ab/pairs/`, key hidden in `target/xtask/ab/.key.json`);
  `cargo xtask ab score ANSWERS.json --reviewer NAME` combines the two
  orders per image (a preference needs both; otherwise a tie) and appends
  the verdicts to `crates/backrooms/tests/review/verdicts.jsonl`.
* **Calibrating reviewers.** `cargo xtask calibrate prepare` damages ten
  goldens with the regressions the gates exist for (cartoon colour,
  washed-out light, a hole, smeared detail) and adds identical controls,
  then prepares them as blind pairs (the golden is the right answer, a tie
  for the controls). Give a fresh reviewer a copy of the pairs folder, then
  `cargo xtask calibrate score ANSWERS.json --reviewer NAME` reports
  accuracy and Cohen's kappa by kind and appends them to
  `crates/backrooms/tests/review/calibration.jsonl`. Once people's verdicts
  exist (`docs/PLAYTEST.md`), they are the truth a reviewer is held to.
* **Playtests.** `docs/PLAYTEST.md` is the script for sessions with fresh
  players (brief, silent observation, debrief, tension trace, one Enjoyment
  item, A/B by ranking); each session is a file in `docs/playtests/` from
  `TEMPLATE.md`, and its findings go into the verdict log, the critic's
  memory, `QUALITY.md` and the ledger.
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
* **Golden frames.** `cargo xtask shots` renders 33 fixed frames headless
  with `--fresh` (each creature at 2, 4 and 8 m and its death scene; the
  spawn view of eight levels and one view per zone purpose or place, which together
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
* **Expressive range.** `cargo xtask range` builds 72 levels (12 seeds ×
  levels 1–6) with `backrooms --metrics` and measures each on properties
  other than the generator's inputs (`level_metrics`: loops, dead ends, depth
  from spawn, integration, room size, sightlines, light and prop density,
  themes per zone, dark share, grime). It compares the set with
  `crates/backrooms/tests/recorded/range.json`: the energy distance with a
  permutation test catches drift (p < 0.01), and a measure whose spread
  halves or doubles catches sameness or chaos. It writes
  `target/xtask/range/report.md`, three scatter plots and a seed archive
  (`target/xtask/range/archive.json`: per cell of depth × light density, the level with the
  longest views). `--look` also renders every zone view of four runs and
  holds each purpose to its `STYLE.md` band. After an intended change to
  generation, read the report and the plots, then
  `cargo xtask range --accept --reason "…"` (logged in
  `tests/recorded/ACCEPTED.md`). Part of the full check.
* **The art bible as a gate.** `docs/STYLE.md` ends with a JSON block of
  look bands: a colourfulness ceiling for every frame, a luminance ceiling
  for level views, and a band per zone purpose. `cargo xtask shots` holds the
  goldens to it, so a purpose whose view drifts out of its band fails even
  when its frame and look anchor are re-accepted.
* **The content ledger.** `docs/content.json` lists every content family
  (creatures, anomalies, zone purposes, prop themes, sound) with the checks
  it must pass; `tests/structure.rs` fails when the code has content the
  ledger lacks, or the ledger names a check that does not exist. Grades and
  open issues per subsystem live in `docs/QUALITY.md`.
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
* `crates/backrooms/tests/level_sweep.rs` checks generated levels through
  the real collision geometry (a few fixed seeds, then 12 proptest cases;
  `SWEEP_CASES=500` for a long sweep): spawn is clear, the exit can be walked
  to, every open tile is reachable by a player-sized body (a flood fill over
  a 0.25 m grid with `circle_hits_wall`) unless a barricade closes it off (by
  design: barricades keep only spawn → exit solvable, and about 6 levels in
  20 have rooms behind them), every pickup can be reached, and a level built
  after another is the same as one built alone. Failing cases are saved in
  `tests/level_sweep.proptest-regressions` (commit it). The ignored helpers
  `census` (pocket statistics over fixed seeds, or `SEED=… LEVEL=…`) and
  `show_pocket` (an ASCII map around a tile) help when it fails. **Open
  issue:** a barricade can lock a pickup away (about 1 level in 20); the sweep
  reports these while `PICKUPS_BEHIND_BARRICADES` is true and fails on
  pickups lost any other way.
* `crates/backrooms/tests/structure.rs` checks the conventions mechanically
  and says how to fix a failure: the engine never depends on the game and
  the game on no platform crate; level-build code takes no clock or foreign
  randomness; every themed placer has a theme; every creature has goldens
  and behaviour tests; no source file passes 2,400 lines; paths named in the
  docs exist; the fast check skips only tests that exist.
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

## Assets

The embedded meshes are built from the bpy scripts in `assets/src/blender/`,
the photographed textures from the scripts in `assets/src/textures/` (see
`assets/README.md`):

* `cargo xtask assets props [--batch …]` builds the props in Blender headless
  and compares every part with `crates/game/assets/prop_meshes.bin` by its
  decoded geometry; `--write` (a full build) replaces the file, keeping the
  bytes of unchanged parts. The build repeats exactly: every part's
  randomness is seeded from its name and noise vectors come from
  `propkit.noise3` (`mathutils.noise.noise_vector` differs in every Blender
  process). 123 *frozen* parts, built before that, keep their stored shapes
  until a deliberate `--rebuild-frozen` with re-recorded prop fixtures.
* `cargo xtask assets textures [--write]` builds the Building's photographed
  surfaces (`crates/game/assets/textures/`) with
  `assets/src/textures/build.py` from the CC0 photographs listed in its
  `sources.json` (each checked by its SHA-1) and compares them; `--write`
  replaces them and records their fingerprints. It needs a Python with
  Pillow, NumPy and SciPy (`$PYTHON`).
* Creatures still rebuild in the HTML version (they need its rig dumps);
  `cargo xtask assets import-js` converts its generated modules (byte for
  byte).
* `crates/game/assets/MANIFEST.json` records each file's sources, generator,
  licence and fingerprint; `tests/structure.rs` fails on a file whose
  fingerprint changed without its entry, a licence or generator not on the
  allowlist, or a generative model not allowed (non-commercial or
  territory-limited weights).
* `crates/game/tests/asset_lint.rs` lints every part (indices, bounds,
  normals, slivers, distance from its frame, the triangle budget from
  `propkit` or its batch script, skin weights) and keeps
  `crates/game/assets/parts.tsv` current; after a mesh change,
  `REGEN=1 cargo test --release -p game --test asset_lint` rewrites the
  table so the diff shows what changed.

## Performance and replays

* `cargo xtask bench` times twelve levels headless (build, load plus first
  frame, CPU update and Metal GPU time per frame along a fixed path, draw
  calls, triangles; medians of three repeats) against the budgets in
  `docs/PERF.md`, and looks for lasting steps in `perf/history.csv` (a
  change-point search with a permutation test, once eight runs are on
  record). Record a run after any change to rendering, level building or
  per-frame code, and commit the history with it; the full check runs
  `--no-record`.
* `crates/backrooms/tests/replay.rs` records a bot's inputs, replays them in
  a fresh game and compares a hash of the state (player, creatures,
  director, world; `game::debug::replay`) after every frame: a
  nondeterminism shows at the frame it enters. Each run gets its own thread,
  because the caches and the per-frame random stream are thread-local.

## Personas, run analysis, telemetry

* `crates/backrooms/tests/personas.rs` plays three levels with death on as
  an explorer (heads for space it has not seen; coverage heatmaps in
  `target/personas/`), a rusher (sprints for the exit) and a hider
  (crouches in the darkest dead end), walking a 0.25 m grid of where a body
  fits. Oracles: never outside the world, never stuck over 12 s, the rusher
  reaches the exit, no death in the first 20 s. Then the director's log
  (`Director::log`: phases, the stalker's signs, stings, notices, strikes,
  deaths) of every run is held to the shape of the fear: a peak at most
  60 s, a relief at least 8 s, the same sign at most 8 times in a row,
  signs followed by a chase between 5% and 95% of the time, at most 6
  stings a level. The report is `target/personas/report.md`.
* `backrooms --record FILE` records a session (inputs in order, the
  window's frame times and clock, a telemetry row a second, the director's
  events) on this machine only; `backrooms --replay FILE` plays it back
  exactly and takes tension marks (↑ / ↓) into `<FILE>.trace.json`;
  `cargo xtask heatmap FILE…` draws where players went. See
  `docs/PLAYTEST.md`. `tests/replay.rs` proves a recording survives its file
  and replays to the same states.

## Continuous checks

* **CI:** `.github/workflows/check.yml` runs `cargo xtask check --fast` on
  every push and pull request on GitHub's macOS runners, and nightly the
  full check without the bench (a virtual GPU's timings mean nothing) plus a
  200-case level sweep; the golden frames are advisory there (they bind to
  the Mac that recorded them). It runs once the repository has a GitHub
  remote.
* **Drift sweep:** `cargo xtask drift [--skip-check]` runs the full check (or
  reads its last logs), writes each `docs/content.json` entry's `passes`
  and `last_checked`, and rewrites the *Latest sweep* section of
  `docs/QUALITY.md` (steps, tests, frames, range, performance, the largest
  files, failing content, and the count of regressions people caught that
  no gate did); the `drift-sweep` skill is the weekly routine around it.
* **Nightly on the reference Mac:** `cargo xtask nightly` = the drift sweep,
  a 200-case level sweep and a recorded bench run.
* `cargo xtask check --skip STEP,…` leaves steps out.

## Profiling and debugging

* `backrooms --screenshot out.png --level L --seed S --play --frames N`
  renders headless and prints draw calls, triangles, lights and pipeline
  counts; add `--scale 2` for Retina resolution.
* `backrooms --bench FILE --seed S --levels …` writes the timings
  `cargo xtask bench` reads.
* `--fresh` (headless modes) ignores the saved settings and progress and
  seeds the per-frame randomness, so the same command renders the same frame
  bit for bit. Headless runs never write the save file.
* `backrooms --dump-level DIR --seed S --levels 1,2 --screenshot x --views`
  also renders one PNG from inside each zone (`rs_S_LN_zNN.png`), one
  looking at the exit door (`_exit.png`), one of M.'s camp (`_camp.png`) and
  one at one of her sheets looking on along her trail (`_trail.png`); the
  JSON lists each zone's purpose and the regions. Each view runs the props'
  distance cull for its own camera (it otherwise runs only a few times a
  second, and every view would keep the spawn's: small props far from the
  spawn went missing from the zone views, and from the goldens, until
  2026-10-26). `cargo test --release -p backrooms --test story` draws each
  checked level's walk, sheets and camp from above into `target/story/`.
* `--showcase smiler[:dist]` (or `hound`, `watcher`) keeps a creature
  standing in front of the camera for the shot; add `--death` to be killed
  by it 15 frames before the shot (the death scene). Give it a floor of the
  rooms (`--level 4` or deeper): the Building's floors (1–3) have no
  creatures, and the arrival there stands high over y = 0.
* The K key shows the in-game debug panel (positions, timers, the ambient
  log, entity states).
* `cargo run --release -p engine --example perftest` measures the renderer
  alone; `--example rendertest` renders a reference scene to PNG.
