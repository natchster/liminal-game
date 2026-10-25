# Tooling roadmap

Status: **being implemented** (started 2026-10-25); the [progress
table](#progress) says what exists.

The game is written mostly by an AI coding agent, and it will keep growing:
new zones, props, creatures, anomalies, sounds. This file lists the
skills, tools and checks to add first so that growth does not wear the
quality down. It is based on a research pass over agent harnesses, content
pipelines, automated game QA and design evaluation. The sources are at the
end; [SOURCES.md](SOURCES.md) covers the research behind the game itself.

## Progress

| Milestone | Items | State |
|---|---|---|
| M1 | 0.1 `CLAUDE.md`, 0.2 path rules, 0.4 `cargo xtask check` | done |
| M2 | 0.5 hooks: refuse parent-folder and fixture edits, rustfmt after edits, the fast check on stop, progress after compaction | done |
| M3 | 1.1 golden frames (`cargo xtask shots`, 31 frames, exact then ꟻLIP), 1.2 look statistics (per frame and set-wide anchors), 1.3 material lint (`tests/materials.rs`); `--fresh` reproducible headless renders | done; prop turntable sheets move to 3.1, goldens are kept in the repository at 480×300 (no Git LFS here) |
| M4 | 1.7 audio gates (`tests/loudness.rs`: EBU R128 integrated loudness and range against a recording, −1 dBTP true peak, one-shot repetition) | done; the 30-minute whole-mix measurement is left for when the mix gets music or voices |
| M5 | 1.5 level sweep through the collision geometry (`tests/level_sweep.rs`, proptest), 1.10 structural tests (`tests/structure.rs`) | done; the sweep found that barricades can lock a pickup away (about 1 level in 20), reported as an open issue until `tail_barricades` checks pickups too |
| M6 | 0.3 `LORE.md` (ported, with delivery channels), `PILLARS.md`, `STYLE.md` (with look bands checked by `cargo xtask shots`); 0.6 content ledger `docs/content.json` (kept in step by `tests/structure.rs`); 0.7 scorecard `docs/QUALITY.md` | done; the bands start wide (one view per purpose) until the range report tightens them |
| M7 | 1.6 expressive range (`cargo xtask range`: 72 levels measured by `backrooms --metrics`, energy distance + permutation test, spread checks, scatter plots, a MAP-Elites seed archive; `--look` measures 631 zone views per purpose) | done; the `STYLE.md` bands are now measured percentiles, which showed that darkness follows region light rather than purpose |

## Contents

- [Progress](#progress)
- [What can drift, and what catches it today](#what-can-drift-and-what-catches-it-today)
- [Principles](#principles)
- [Phase 0 — make the repository teach the agent](#phase-0--make-the-repository-teach-the-agent)
- [Phase 1 — computational sensors (the gates)](#phase-1--computational-sensors-the-gates)
- [Phase 2 — skills and reviewers](#phase-2--skills-and-reviewers)
- [Phase 3 — the asset pipeline](#phase-3--the-asset-pipeline)
- [Phase 4 — playtesting and measuring fear](#phase-4--playtesting-and-measuring-fear)
- [Phase 5 — continuous drift control](#phase-5--continuous-drift-control)
- [Order and effort](#order-and-effort)
- [What not to build](#what-not-to-build)
- [How to know the tooling works](#how-to-know-the-tooling-works)
- [Sources](#sources)

## What can drift, and what catches it today

| Dimension | What drift looks like here | Caught today by | Missing |
|---|---|---|---|
| **Code** | Layering broken (game touching Metal), determinism rules broken, modules swelling (`ui/draw.rs` 2,264 lines, `level/archgen.rs` 2,128) | `cargo fmt`, `clippy`, recorded fixtures, 20 reference levels | One check command, CI, structural tests, file-size caps |
| **Look** | Props drift back to saturated and flat, lighting creeps brighter, creatures lose their menace | Material values inside fixtures; ad-hoc PNG review; blind A/B reviews run from scratch folders | Golden frames with a perceptual diff, look statistics per zone, material lint |
| **Sound** | Mix gets louder or quieter, events pile up, sounds stop coming from places | `listen.rs` (bed-relative checks), ambient tests | Loudness and true-peak gates, repetition checks |
| **Space** | Levels feel random, unreadable or samey; pockets a body cannot reach | archgen / world fixtures, `zone_purpose.rs`, tile-grid reachability inside archgen, `playthrough.rs` | Many-seed expressive-range report, reachability through the real collision walls |
| **Fear** | Scares repeat, cues always mean the same thing, the director gets too punishing or too soft | `pacing.rs` bands, creature behaviour tests | Event-log analysis, telemetry, human playtest data |
| **Canon** | New things that do not belong in the Backrooms | `docs/LORE.md` — but only in the HTML version's folder | Canon in this repo, a canon check |
| **Performance** | Frame time, load time and memory creep | `engine` `perftest` example, counts printed by `--screenshot` | Budgets, a history, regression detection |
| **Assets** | Meshes that cannot be rebuilt, unknown provenance | Magic-number check on `prop_meshes.bin` / `creature_meshes.bin` | The Blender sources (they live in the HTML version's `tools/blender`), a rebuild and check command, mesh lint, a licence manifest |

Two more observations from the last pass. The metrics that proved the
realism and sound work (colourfulness, deep-shadow share, theme coherence,
share of events from a place, the blind A/B harness) were one-off scripts
in a scratch folder, so nothing will catch a regression in them. And the
human play-test verdict is still pending: everything so far has been judged
by measurements and by agents.

## Principles

1. **Computational sensors first, inferential second, humans for taste.**
   Tests, lints and metrics are cheap and deterministic, so run them on
   every change; model judges are slower and less reliable, so use them on
   what the sensors flag (Böckeler). Vision models detect visual regressions
   poorly (45% in VideoGameQA-Bench) and invent glitches, and Anthropic's own
   multi-agent game experiments concluded that "models have poor taste in
   this arena". Claude cannot hear at all. Whether something is scary is a
   human call, made with fresh players.
2. **The maker never grades its own work.** Self-evaluation is lenient,
   and an evaluator can "talk itself into" approving bugs it found
   (Anthropic, harness design). Judges favour their own outputs
   (Panickssery et al.) and the order they are shown things (Wang et al.).
   Reviews run in a fresh context, blind, in both orders, ideally with a
   different model family, and disagreement counts as a tie.
3. **Compare with a frozen baseline, not an absolute.** Loudness relative to
   the game's own mix, level metrics as a distribution compared with the
   accepted one (energy distance, Summerville), performance as a history
   with change-point detection (Daly et al.). Absolute targets hide drift;
   relative ones expose it.
4. **A golden is accepted on purpose.** wgpu's harness silently writes a
   missing reference image, which lets an agent "pass" by deleting one. Here
   a missing golden fails, and accepting a new one is an explicit command
   whose reason goes into the commit.
5. **Make the repository teach the agent.** A short root map, rules that
   load only for the files being touched, skills that load only when used,
   and lint messages that say how to fix the problem (Claude Code docs,
   harness-engineering write-ups).
6. **Every content change ends in evidence.** A rendered sheet, a WAV and
   its loudness figures, a bot log — kept, not described. Agents otherwise
   mark work done after unit tests alone (Anthropic, long-running harnesses).
7. **Small units, tracked in a ledger, fresh sessions.** One content piece
   per session, with its required checks recorded in a file the next session
   reads; long sessions "drift toward tunnel vision" (Cursor).
8. **Generated, not authored** stays the house rule: textures drawn in code,
   sounds synthesized, meshes from scripts. Any AI-generated asset enters
   only through the pipeline in Phase 3, with its provenance recorded.
9. **Feed recurring failures back.** When the same mistake shows up twice,
   change a rule, a lint or a test so it cannot show up a third time.

## Phase 0 — make the repository teach the agent

Days of work; everything later builds on it.

**0.1 `CLAUDE.md` (under 200 lines).** A map, not a manual: what the
workspace is, the commands (`cargo xtask check`, `REGEN=1`, `--ignored
rerecord`, `--dump-level --views`, `--showcase`, `listen.rs`, `pacing.rs`),
and the invariants that have already cost time when broken:
- levels are a pure function of `(run_seed, level)`;
- lighting and ambience are tied to places, never to the player's position;
- never capture the user's screen (offscreen renders and PNG readback only);
- never modify the HTML/JS version in the parent folder;
- commit only when asked, with the project's identity.

Everything else stays in `ARCHITECTURE.md` and `DEVELOPMENT.md`, linked from
it. The docs recommend under 200 lines because longer files lower adherence.
*Done when:* a fresh session can build, test and re-record from it alone.

**0.2 Path-scoped rules (`.claude/rules/*.md` with `paths:`).** These load
only when matching files are touched:

| Rule file | Applies to | What it holds |
|---|---|---|
| `world.md` | `world/**`, `level/**`, `crates/placers/**` | determinism, draw order, which fixtures to re-record |
| `audio.md` | `audio/**` | place-based sound, hush, carry table, the loudness gate |
| `visuals.md` | `visuals/**`, `crates/engine/src/*.metal` | albedo ranges, place-based light, shader compile cost |
| `entities.md` | `entities/**` | the director's invariants, the senses tests, the showcase shots |

**0.3 Canon and taste, written down in this repo.**
- `docs/LORE.md`: port the canon from the HTML version as a living bible
  (Megill): a short core summary, the canon list (levels, entities, objects,
  events), and the *delivery channels* the canon may use — notes, sound,
  architecture. The `lore-keeper` reviewer checks new text against both.
- `docs/PILLARS.md`: three to five pillars, each broken into goals and then
  into rules with a one-line test (Koster's pillars → goals → rules).
  Pillars name experiences, not features (Tarczykowski); Duskers, for
  example, aimed at isolation rather than fear and left out music to get it
  (Keenan). Candidates here: *familiar but wrong*, *the hum never stops*,
  *you are never shown everything*, *emptiness is the threat*. Every
  content change quotes the rule it serves; Koster admits it is "easy to
  forget" to check work against pillars, so the skills in Phase 2 make it a
  step.
- `docs/STYLE.md`: an art bible written as data, one entry per zone purpose
  (Control's sector-by-sector deviation from one base is the model):
  - materials and their albedo, roughness and shininess ranges;
  - ceiling module, fixtures and motifs;
  - the deviation each purpose is allowed;
  - target bands for mean luminance, deep-shadow share and colourfulness.

  The checks in Phase 1 read the numeric part, so the bible and the gates
  cannot disagree. Each entry also names its wayfinding cues and whether a
  threat belongs there at all: Pools has no monsters, and its wet footprints
  tell players when they are walking in circles. A zone with no creature is
  a legitimate design, not a gap to fill.

**0.4 `cargo xtask check`.** One command, two modes:
- `--fast`, under 3 minutes: fmt check, clippy with `-D warnings`, unit
  tests, and a fixed random sample of the seed sweeps and fixtures;
- full: everything.

Output is terse, with one grep-able `ERROR` line per failure; a long log
wastes the agent's context (Carlini). Every later gate plugs into it.

**0.5 Hooks (`.claude/settings.json`).**

| Hook | What it does |
|---|---|
| `PostToolUse` on `*.rs` edits | runs `rustfmt` on the edited file |
| `Stop` | runs `cargo xtask check --fast`; honours `stop_hook_active` so it cannot loop |
| `SessionStart` (matcher `compact`) | re-injects the invariants after compaction |
| `PreToolUse` | blocks writes to the parent folder; blocks hand edits to fixtures and goldens, so they change only through the REGEN / accept commands |

**0.6 Content ledger (`docs/content.json`).** One entry per creature, prop
family, zone purpose, anomaly and sound signature. Each entry holds its
required checks, a `passes` flag per check, the evidence paths and the date
of its last human review. JSON, because agents rewrite it less freely than
Markdown (Anthropic). A session starts by reading the ledger and picking one
entry.

**0.7 Quality scorecard (`docs/QUALITY.md`).** A grade (A–D) per subsystem
and per content family, with the evidence behind each grade and the open
regressions. The drift sweep (5.2) rewrites it. This is the
"quality score" idea from the harness-engineering write-ups, adapted.

## Phase 1 — computational sensors (the gates)

One to three weeks. Each item becomes part of `cargo xtask check`.

**1.1 Golden frames with a perceptual diff.** `cargo xtask shots` renders a
fixed shot list:
- one view per zone for each reference seed (`--dump-level --views`);
- each creature at 2, 4 and 8 m (`--showcase`), plus its death scene;
- a turntable sheet per prop family.

Each shot is compared with its golden using FLIP (`nv-flip` crate): the
mean, the 99th percentile, and the worst 64×64 tile, because a missing wall
is a small, local change that a global mean hides (Unreal's local-error
check). Tolerances are set per shot from N repeat renders of the same scene
(Chromium Gold), not one global threshold.

Goldens are pinned to one Mac and OS version and stored outside the code
tree (a separate repo or Git LFS); on any other machine the result is
advisory (Filament). A missing golden fails. `xtask shots --accept NAME
--reason …` is the only way in. Failures write the FLIP error map next to
the frame, so a person or a reviewer agent can see where it changed.

**1.2 Look statistics per zone.** Promote the scratch metrics to Rust, run
over the same shots, and gate each value on the per-purpose bands in
`STYLE.md`:
- colourfulness (Hasler–Süsstrunk);
- deep-shadow share;
- luminance histogram and mean.

Goldens are re-accepted whenever a change is intended; these bands still
catch the slow slide back to bright, saturated, "cartoonish" frames.

**1.3 Material lint.** Every material and canvas texture the game builds is
checked across N seeds; levels build headless, so this is cheap:
- base-colour maximum channel between 0.032 and 0.87 for non-metals (the
  OpenMoonray validity range; see also Unreal's measured values and
  physicallybased.info);
- shininess, specular and emissive within the material class's range;
- texture tiling seams.

Lint messages name the material and the placer that built it.

**1.4 Mesh lint for the embedded parts.** Run over every part in
`prop_meshes.bin` and `creature_meshes.bin`:
- triangle budget per prop class;
- no zero-area faces, no unwelded seams, no degenerate normals (rules after
  the Omniverse asset validator);
- consistent up axis and unit scale, pivot at the base;
- skin weights normalised.

A per-part stats table is written, so a commit's diff shows what changed.

**1.5 Level property sweep (proptest).** Over thousands of seeds at night,
and a fixed sample in `--fast`. Proptest stores the seeds of failing cases
in committed regression files and replays them first. The properties:
- every walkable tile is reachable from spawn by a **player-sized body
  through the real collision walls**, not only on the logical tile grid
  that archgen already checks (the HTML version had this flood test);
- spawn is clear;
- exits and items are reachable;
- no prop sinks into or overlaps another;
- building the same `(seed, level)` twice, or after another level, gives
  the same hash.

Wherever the collision map and the logical grid disagree, that is a bug.

**1.6 Expressive-range report.** `cargo xtask range` builds N seeds per zone
purpose. It measures properties *other than* the generator's inputs (the PCG
book's rule):
- loop ratio and dead-end ratio;
- depth from spawn and integration of hub rooms (space-syntax measures,
  which Bıyık and Sürer used to evaluate a generated haunted-house game);
- room-size distribution and longest sightline;
- light and prop density;
- distinct themes per zone.

It writes 2D density plots as PNG (Smith & Whitehead), compares the joint
distribution with the accepted baseline by energy distance (Summerville),
and fails on a jump (drift) or a variance collapse (sameness). The baseline
is re-accepted like a golden. This is the main guard for *"less random, but
still varied from run to run"*. Wider is not automatically better
(Summerville), and no single "fitness" score stands in for quality
(Withington et al.), so the report shows the distributions and the
developer decides what an acceptable spread is.

The report also keeps a **seed archive** in the MAP-Elites manner (Mouret
and Clune; Alvarez et al. for rooms): a grid over two or three of the
metrics — say navigation depth × light density per zone purpose — holding
the best seed found for each cell. Empty cells show what the generator
cannot make; one overfull cell shows sameness. The archive's seeds double
as the shot list for 1.1 and the bot levels for 4.1, so tests cover the
whole range, not just a handful of familiar seeds.

**1.7 Audio gates.** Measured with the `ebur128` crate on the `listen.rs`
renders:
- integrated loudness within ±1 LU of the accepted baseline per scenario;
- true peak at or below −1 dBTP;
- loudness range within a band;
- each signature's level relative to the room bed.

Two repetition checks are added: the same one-shot never twice within a set
time, and no bed loop repeated within a level (Sonancia's rule). Published
game targets such as −23 LUFS and Sony's ±2 LU are for whole mixes over
long play, so the absolute target is measured on a 30-minute bot walk and
the per-scenario checks stay relative.

**1.8 Performance budgets and history.** `cargo xtask bench` walks a fixed
camera path through each reference seed. It records:
- GPU time (`MTLCommandBuffer` `gpuStartTime` / `gpuEndTime`);
- CPU frame p50 and p99;
- draw calls and triangles;
- level build plus first-frame time (the shader-compile cost lands there);
- peak heap during the build (`dhat`).

Each run appends a row to `perf/history.csv`. Regressions are found by
change-point detection over the history (Daly et al., Hunter); a fixed
threshold fires on noise. Budgets live in `docs/PERF.md`.

**1.9 Replay and divergence.** Bot runs record their input stream and a
per-tick state hash. A replay must reproduce the hashes, and a mismatch
reports the first divergent tick (Factorio's per-tick CRC). This catches a
nondeterminism the moment it enters, instead of when a fixture fails three
changes later.

**1.10 Structural fitness tests.** Each failure message says how to fix it:
- `engine` never depends on `game` (`cargo metadata`);
- no `Instant::now`, `thread_rng` or `HashMap` iteration inside level-build
  modules;
- every placer appears in `THEMED_ORDER` and `PLACER_THEME`;
- every ambient sound has a `sig_carry` entry;
- every creature has showcase shots and senses tests;
- file-size caps on new modules;
- every module named in `ARCHITECTURE.md` exists.

## Phase 2 — skills and reviewers

Ongoing; one skill per content type, each written once the gates it relies
on exist.

**Skills (`.claude/skills/<name>/SKILL.md`).** Each one:
- stays under 500 lines, with detail in linked files;
- ends with an evidence step and a ledger update;
- has `disable-model-invocation: true` if it re-records or accepts
  anything.

| Skill | What it walks through |
|---|---|
| `add-zone` | The Level Design Book's order: layout, then blockout metrics (range, reachability), then a bot walk, then lighting, then set dressing last. A `STYLE.md` entry comes first, and reference levels are re-recorded with the reason given. |
| `add-prop` | Purpose and theme, then real-world reference values, then the Blender part through 3.1, material and mesh lint, a turntable sheet, a golden accepted, and the ledger. |
| `add-creature` | A brief checked against the pillars and the canon, a silhouette test at 8 m, the rig and animation through 3.1, showcase sheets at 2/4/8 m plus the death scene, senses and behaviour tests, director integration, and a `pacing.rs` run. |
| `add-anomaly` | The anomaly rules (at most one per level, never twice in a run), a canon check, and its own shots and tests. |
| `add-sound` | Synthesis recipe, place anchor, carry entry, the loudness gate, a `listen.rs` scenario and its timeline PNG. Then a **human listening checkpoint**, because the agent cannot judge the sound itself. |
| `tune-pacing` | Run `pacing.rs` N times, read the phase statistics, change one parameter, and compare the distributions. This is the loop NVIDIA's Fly, Fail, Fix uses: playtester metrics in, configuration changes out. |
| `visual-review` | The blind A/B harness promoted from the scratch folder: random sides in both orders, hidden keys, a reviewer subagent that sees only the images, the rubric and the anchors, and ties on disagreement. Verdicts are appended to a log for 4.5. |
| `regen-fixtures` | The only sanctioned way to re-record fixtures or accept goldens. It summarises the diff and writes the reason into the commit message. |
| `drift-sweep` | Run every gate, the range report and the bench history, rewrite `QUALITY.md`, and list the regressions (5.2). |

**Reviewer subagents (`.claude/agents/`).** All are read-only, and none sees
the author's reasoning:

| Reviewer | Sees | Checks | Notes |
|---|---|---|---|
| `art-critic` | images, the `STYLE.md` rubric, anchor images | look against the rubric | Keeps project memory of human verdicts. Calibrated on the problem screenshots already in the HTML version's folder (`bad_walls.png`, `broken_entity.png`, `light_wrong.png`, `perspective_wrong.png`, `pool_wrong.png`, `pool_blocks.png`, `vending_wrong.png`, `window_wrong.png` and others). They are copied into a labelled set, each with a note saying what was wrong. |
| `lore-keeper` | new text, creatures and anomalies | contradictions with `LORE.md` and `PILLARS.md`, and canon delivered through channels the bible does not allow | Also tracks the variety of the agent's own notes and descriptions: expressive-range analysis applies to generated text too (Boucher et al.), and same-model writers repeat themselves. |
| `level-analyst` | the range report, reachability and the zone views | explains drift; does not approve it | |
| `audio-analyst` | loudness timelines, spectra, event logs | numbers only | Never asked whether a sound is good. |
| `code-reviewer` | the diff | layering, determinism rules, module size; flags only real gaps | The Claude Code docs warn that open-ended reviewers push over-engineering. |

Same-model reviewers share blind spots (Anthropic, multi-agent systems), so
their agreement with people is measured (4.5) before any of them blocks
anything.

## Phase 3 — the asset pipeline

Weeks; do it before the next big batch of props or creatures.

**3.1 Bring the Blender sources here.** Today `prop_meshes.bin` and
`creature_meshes.bin` come from scripts in the HTML version's
`tools/blender`:
- `props/b00.py` … with their `.map.json` and notes;
- `creatures.py`, `skinned.py`, `skin_*.py`;
- the packers `creature-pack.mjs` and `prop-assemble.mjs`.

Copy them into `assets/src/`. Then add `cargo xtask assets`: it runs a
pinned Blender version headless from an empty scene, one `.blend` per
process (the `bpy` docs), and packs the `.bin` files deterministically.
`--check` proves the committed `.bin` files decode to what the sources
produce. *Done when:* a prop can be changed and rebuilt from this repo
alone, and 1.4 runs on the result.

**3.2 Asset manifest and licence gate.** One record per part: its source
script or tool, the model and version if one was used, the licence, and the
human edits made. A check against an allowlist blocks:
- non-commercial weights (AudioGen and MMAudio checkpoints are CC-BY-NC);
- licences with territory exclusions (Hunyuan3D 2.1 excludes the EU, the UK
  and South Korea).

The manifest also records what Steam's content survey would need disclosed
(pre-generated AI content that players see). Prompts alone do not make a
human author (US Copyright Office), so the record of human edits matters.

**3.3 Blender MCP for inspection, scripts for changes.** Use the official
Blender Lab server (Blender 5.1+) or the community `blender-mcp` to look at
scenes and render multi-angle review sheets while modelling. Both run model
code "without any guards", so run them in a VM or sandbox, keep them
localhost-only, and turn telemetry off. What gets committed is always a
script in `assets/src/`, never an opaque edit to a `.blend` file.

**3.4 Optional: generated meshes as raw material for background props.**
Image-to-3D models (TRELLIS.2 is MIT-licensed; others need the licence
check) produce about 1M faces with possible holes and no scale. So each
output goes through:
1. decimation with a bounded error (`meshoptimizer`);
2. a UV unwrap (`xatlas`);
3. a bake;
4. the 1.4 lint;
5. a pairwise turntable comparison against golden props (GPTEval3D style);
6. human approval.

Creatures stay hand-directed; deforming meshes need edge flow no generator
gets right. Whether this fits "generated, not authored" is a decision for
the developer, so it stays off until made.

**3.5 LOD chains** with `meshoptimizer`, built at pack time, each level made
from the previous one, gated on the returned error rather than on triangle
count. Only if 1.8 shows prop triangles as the bottleneck.

**3.6 Optional: reference sounds.** Commercially licensed generators
(ElevenLabs on a paid plan, Stable Audio Open within its community licence)
produce *targets* that the synthesized sounds are compared against: their
envelope, spectrum and loudness. They are never shipped. Moffat and Reiss
found synthesized effects can match recordings in blind tests, and a
reference makes the comparison concrete.

## Phase 4 — playtesting and measuring fear

Ongoing; this is where taste comes in.

**4.1 Bot personas.** Next to the cautious bot in `pacing.rs`, three more
(Holmgård's procedural personas):
- an *explorer* drawn to unvisited space, which yields per-seed coverage
  heatmaps (EA SEED's curiosity agents);
- a *rusher*;
- a *hider*.

Every persona shares the same oracles, defined before any bot is written
(Politowski et al.): stuck for over N s, outside the world, NaN position,
an exit not found, a death the player had no way to avoid. Deterministic
scripted bots beat RL here, because the only movement is walking.

An LLM agent playing from structured state can add a *clarity* check — can
it find the exit, does it understand a note — since LLM players' results
track the difficulty humans report (Xiao and Yang). It is not asked whether
anything is fun or frightening: model judgments of fun follow human ones
less consistently than judgments of fairness (Collins et al.).

**4.2 Run-log analysis.** From the director's and the ambient system's logs
across many runs:
- time between scares, and the longest stretch at peak menace (Alien:
  Isolation withdraws its alien "backstage" at the top of the gauge);
- relief length (Booth's relax phase is 30–45 s): horror needs a cycle of
  dread and release, since constant attacks numb and tension that never
  releases bores (Bycer, opinion rather than data);
- how often the same cue repeats;
- whether one cue leads to different outcomes (Dead Space's intensity
  director: the same vent sound sometimes means nothing);
- a cap on each scare type per session ("in horror, repetition is a deadly
  sin", Grip);
- intended against computed tension per room (Sonancia).

Scares are judged as **sequences**, not one at a time: in a modified P.T.,
the order of events and the build-up changed measured tension, and players
habituated to repeated mechanics (Graja et al.); repeated play dulls
arousal even when players do not notice it themselves (Grizzard et al.).
So the report looks at what came before each scare, and the per-type caps
apply across a whole run, not only per level.

The report gates on bands, like `pacing.rs` does now.

**4.3 Telemetry in the human build.** Opt-in, written to a local file, never
sent anywhere. It records:
- positions;
- director phases;
- events, deaths and quits.

A replay viewer draws heatmaps and paths over the level map. Human runs then
feed the same analysis as the bots.

**4.4 Human playtest kit.**
- A script after The Level Design Book: brief, observe silently, debrief
  with comprehension questions and subjective ones.
- Five or six fresh players per milestone (Bromley). Fresh matters: the
  developer and the agent are both used to the scares, which is why Until
  Dawn's team used new testers.
- Players annotate tension over a replay of their own run (RankTrace); the
  direction of change in a trace predicted skin-conductance responses.
- Players rank build A against build B rather than rate each one (the PCG
  book's advice).
- At most one questionnaire item, Enjoyment from the miniPXI. The GEQ's
  subscales did not hold up in validation, and the miniPXI items are only
  moderately stable.
- Results go into the ledger and the scorecard.

**4.5 Calibrate the judges.** Every human verdict, from the playtest kit or
the developer, becomes a labelled example. Cohen's kappa between each
reviewer agent and the humans is reported in `QUALITY.md`. Raw agreement
overstates it by 33–41 points ("Reliability without Validity"), and a judge
can be consistent without being right. A reviewer gets to block changes only
above an agreed kappa.

## Phase 5 — continuous drift control

**5.1 CI.**

| Trigger | Runs |
|---|---|
| Every push | `cargo xtask check --fast`, on a GitHub-hosted macOS runner (Metal works there; Bevy renders its examples on one) or on the developer's Mac |
| Nightly | the full seed sweep, the range report, goldens, the bench history, `pacing.rs` ×N, `listen.rs` |

The golden comparison stays advisory on hosted runners and binding on the
pinned machine.

**5.2 Scheduled drift sweep.** Weekly, in a fresh session, the `drift-sweep`
skill:
- runs everything and rewrites `QUALITY.md`;
- checks the docs against the code;
- lists the regressions and the files that grew past their caps.

A person reads it: the "garbage collection day" from the harness-engineering
write-ups.

**5.3 The steering loop.** Each regression a person catches that no gate
caught becomes a new test, lint, rule or rubric line. Count these;
a falling count is the sign that the harness is working.

## Order and effort

S = up to a day, M = 2–5 days, L = 1–3 weeks.

| # | Item | Effort | Why this order |
|---|---|---|---|
| 1 | 0.1 `CLAUDE.md`, 0.2 path rules, 0.4 `xtask check` | S + S + M | Everything else plugs into them |
| 2 | 0.5 hooks | S | Makes the check run without being asked |
| 3 | 1.1 golden frames, 1.2 look statistics | M + S | Protects the realism and creature work just done |
| 4 | 1.7 audio gates | S | Protects the place-based sound just done |
| 5 | 1.5 property sweep with collision reachability, 1.10 structural tests | M + S | Cheapest guards for the generator and the architecture |
| 6 | 0.3 `LORE` / `PILLARS` / `STYLE`, 0.6 ledger, 0.7 scorecard | M | Needed before new content arrives |
| 7 | 1.6 expressive range | M | Guards "less random, still varied" |
| 8 | 2.x skills `add-zone`, `add-prop`, `add-creature`, `add-sound`; reviewers | M each | Written once their gates exist |
| 9 | 4.4 human playtest kit, 4.5 calibration | M | The pending play-test is the cheapest signal there is |
| 10 | 3.1 Blender sources and `xtask assets`, 3.2 manifest, 1.4 mesh lint | L | Before the next batch of props or creatures |
| 11 | 1.8 performance history, 1.9 replay | M + M | Before levels get heavier |
| 12 | 4.1 personas, 4.2 run-log analysis, 4.3 telemetry | L | Scales the fear measurement |
| 13 | 5.1 CI, 5.2 drift sweep | M | Once enough gates exist to be worth running |
| 14 | 3.3 Blender MCP, 3.4–3.6 generated assets, 3.5 LODs | L, optional | Only after the pipeline and the licence policy are in place |

## What not to build

- **A vision model as the gate.** Use it to explain frames a metric has
  flagged. Measured models detect 45% of visual regressions at best, and
  find the right frame of a glitchy video in only 19 of 100 cases
  (VideoGameQA-Bench).
- **An MCP server for this engine.** The CLI already exposes rendering,
  dumps and showcases, and the Claude Code docs call CLI tools the most
  context-efficient option. Wrap it in skills.
- **Dozens of role agents and process documents.** Anthropic found role
  prompts and hierarchies "did not make much difference" to the games their
  swarms made. A community studio template reports its 30-document "rigor"
  build ranked last in its own blind review (self-reported). A few gates
  and a few reviewers are enough.
- **RL playtesters.** Movement is walking, reward hacking is common (Ubisoft
  La Forge's bunny-hopping agents), and scripted bots with oracles are
  cheaper and deterministic.
- **Self-refreshing goldens or fixtures.** They turn every regression into a
  new baseline.
- **Shipping audio from non-commercial models, or meshes from models whose
  licence excludes where the game is sold.**
- **Agent-only taste.** Agents on the same model share blind spots and
  habituate to nothing; people habituate too, which is why each milestone
  needs fresh players.

## How to know the tooling works

| Measure | Healthy sign |
|---|---|
| Regressions caught by a person that no gate caught (5.3) | falling |
| Gate false alarms per week | low enough that nobody overrides them by habit |
| Time from "new prop family" to an accepted golden with a ledger entry | falling |
| Kappa between each reviewer agent and human verdicts (4.5) | stable or rising |
| The range report's distance from baseline between releases | small and explained |
| The scorecard (`QUALITY.md`) | no grade drops without an entry saying why |

## Sources

Pages marked † were re-checked while writing this file; the rest were opened
during the research passes. *[unverified]* marks claims seen only second
hand.

### Agent harnesses and Claude Code

- † Anthropic, *Effective harnesses for long-running agents* (2025-11-26) — initializer agent, progress file, JSON feature list, one feature per session, end-to-end testing: https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- † Anthropic, *Harness design for long-running application development* (2026-03-24) — planner / generator / evaluator, hard thresholds per criterion, few-shot calibration, evaluator leniency, "Claude can't hear", the retro game maker example: https://www.anthropic.com/engineering/harness-design-long-running-apps
- † Anthropic Frontier Red Team, *Patterns and problems in emerging multiagent systems* (2026-08-13) — "models have poor taste in this arena", role prompts made little difference, same-model homogeneity: https://www.anthropic.com/research/multiagent-systems
- Anthropic, *Building effective agents* (2024-12-19): https://www.anthropic.com/engineering/building-effective-agents
- Anthropic, *Demystifying evals for AI agents* (2026-01-09) — start from real failures, separate regression and capability evals, calibrate model graders: https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents
- Anthropic, *Building a C compiler with a team of parallel Claudes* (2026-02-05) — terse test output, fast deterministic sample mode, stricter CI as features broke each other: https://www.anthropic.com/engineering/building-c-compiler
- Anthropic, *Effective context engineering for AI agents* (2025-09-29): https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
- Anthropic, *Equipping agents for the real world with Agent Skills* (2025-10-16): https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills
- † Claude Code docs — memory (CLAUDE.md under 200 lines, path-scoped rules): https://code.claude.com/docs/en/memory; skills (SKILL.md under 500 lines, progressive disclosure, `disable-model-invocation`): https://code.claude.com/docs/en/skills; † `/goal` (the evaluator reads only the transcript): https://code.claude.com/docs/en/goal; hooks: https://code.claude.com/docs/en/hooks-guide; subagents: https://code.claude.com/docs/en/sub-agents; best practices: https://code.claude.com/docs/en/best-practices
- † Böckeler, *Harness engineering for coding agent users* (2026-04-02) — guides and sensors, computational and inferential, behaviour checks least solved: https://martinfowler.com/articles/harness-engineering.html
- OpenAI, *Harness engineering* (2026-02-11) *[unverified: page not readable; described by]* Böckeler's memo https://www.martinfowler.com/articles/exploring-gen-ai/harness-engineering-memo.html and https://zenml.io/llmops-database/harness-engineering-building-software-where-humans-steer-and-agents-execute — custom linters with fix instructions, structural tests, garbage-collection days
- Cursor, *Scaling long-running autonomous coding* (2026-01-14) — long runs drift toward tunnel vision: https://cursor.com/blog/scaling-agents
- Community game-dev setups for Claude Code: Claude Code Game Studios (screenshot evidence for visible changes; the author's self-reported finding on process documents) https://github.com/Donchitos/Claude-Code-Game-Studios; game-creator (QA subagent after every step) https://github.com/opusgamelabs/game-creator

### Agents building and testing games

- † GameDevBench (arXiv 2602.11103, 2026) — 333 Godot tasks; best agent 53.8%; success drops from 51.4% on gameplay to 33.0% on 2D graphics; visual feedback raised one model from 41.1% to 52.0%: https://arxiv.org/abs/2602.11103
- † Play2Code / PlaytestArena (arXiv 2605.28258, 2026) — a rubric-blind GUI playtester iterating with the builder; 66.8% rubric pass rate, +37.1 points over single pass: https://arxiv.org/abs/2605.28258
- VideoGameQA-Bench (NeurIPS 2025) — glitch detection about 82%, visual regression at most 45.2%, false positives: https://asgaardlab.github.io/videogameqa-bench/ and https://arxiv.org/html/2505.15952
- GlitchBench (CVPR 2024): https://arxiv.org/html/2312.05291
- VideoGameBench (2025): https://arxiv.org/abs/2505.18134
- ArtifactsBench (2025) — per-task checklists judged over frame sequences: https://arxiv.org/abs/2507.04952
- NVIDIA, *Fly, Fail, Fix* (2025) — playtester metrics drive configuration edits: https://research.nvidia.com/publication/2025-08_fly-fail-fix-iterative-game-repair-reinforcement-learning-and-large-multimodal
- EA SEED, automated testing with RL and curiosity: https://arxiv.org/abs/2103.15819, https://arxiv.org/abs/2103.13798
- Ubisoft La Forge, navigation agents and reward hacking: https://arxiv.org/abs/2011.04764, https://news.ubisoft.com/en-us/article/6Mv4hZqUMJoY1xpf1yiQPi/ubisoft-la-forge-pushing-stateoftheart-ai-in-games-to-create-the-next-generation-of-npcs
- Politowski et al., developers' view of automated testing agents (2022) — define the oracle first: https://arxiv.org/abs/2202.12777
- Holmgård et al., procedural personas (2018): https://ar5iv.arxiv.org/html/1802.06881
- Blender MCP: † official Blender Lab server (Blender 5.1+, runs model code "without any guards") https://www.blender.org/lab/mcp-server/; community server https://github.com/ahujasid/blender-mcp

### Visual, audio and performance checks

- FLIP: https://github.com/NVlabs/flip, https://research.nvidia.com/labs/rtr/publication/andersson2020flip/; † Rust crate `nv-flip` (mean, percentiles, 67 px/degree default): https://docs.rs/nv-flip/latest/nv_flip/
- wgpu image tests (mean and percentile checks; a missing reference is written silently): https://github.com/gfx-rs/wgpu/blob/trunk/tests/src/image.rs
- Unreal screenshot comparison (global and local error): https://dev.epicgames.com/documentation/en-us/unreal-engine/screenshot-comparison-tool-in-unreal-engine; Gauntlet: https://dev.epicgames.com/documentation/en-us/unreal-engine/gauntlet-automation-framework-in-unreal-engine
- Filament render diff (goldens kept apart, advisory off the reference host): https://google.github.io/filament/dup/test_ci_renderdiff.html
- Chromium pixel tests with Skia Gold (per-test inexact matching, several baselines): https://chromium.googlesource.com/chromium/src/+/main/docs/gpu/gpu_pixel_testing_with_gold.md
- Bevy example runs on macOS / Metal in CI: https://github.com/bevyengine/bevy/blob/main/.github/workflows/example-run.yml
- LPIPS https://github.com/richzhang/PerceptualSimilarity and DISTS https://github.com/dingkeyan93/DISTS for "same look" after intended texture changes
- † `ebur128` crate (EBU R128 M/S/I, loudness range, true peak): https://docs.rs/ebur128/latest/ebur128/; EBU R128: https://tech.ebu.ch/publications/r128
- Game loudness: Sony's Garry Taylor https://designingsound.org/2012/07/30/video-games-and-loudness-standards-interview-with-sonys-garry-taylor/; G.A.N.G. https://designingsound.org/2013/02/loudness-in-game-audio/
- criterion.rs analysis: https://bheisler.github.io/criterion.rs/book/analysis.html; change-point detection: Daly et al. https://arxiv.org/abs/2003.00584, Hunter https://arxiv.org/abs/2301.03034
- Metal GPU timing: https://developer.apple.com/documentation/metal/mtlcommandbuffer/gpustarttime; `dhat` heap tests: https://docs.rs/dhat/latest/dhat/
- proptest failure persistence: https://proptest-rs.github.io/proptest/proptest/failure-persistence.html
- Determinism checks: Gaffer On Games https://gafferongames.com/post/deterministic_lockstep/; Factorio FFF #47 https://factorio.com/blog/post/fff-47

### Judges

- Zheng et al., *Judging LLM-as-a-Judge* (2023): https://arxiv.org/abs/2306.05685
- Wang et al., *Large Language Models are not Fair Evaluators* (2023) — order swaps flip verdicts: https://arxiv.org/abs/2305.17926
- Panickssery et al. (2024) — self-recognition and self-preference: https://arxiv.org/abs/2404.13076
- MLLM-as-a-Judge (2024) — pairwise is closer to humans than scoring: https://arxiv.org/abs/2402.04788
- † Norman et al., *Reliability without Validity* (arXiv 2606.19544, 2026) — kappa 33–41 points below raw agreement; consistent yet position-biased judges: https://arxiv.org/abs/2606.19544
- GPTEval3D (CVPR 2024) — pairwise multi-view judging of 3D assets: https://arxiv.org/abs/2401.04092

### Content pipelines and assets

- TRELLIS.2: https://github.com/microsoft/TRELLIS.2; Hunyuan3D 2.1 and its licence: https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1, https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1/blob/main/LICENSE
- `meshoptimizer`: https://meshoptimizer.org/; `xatlas`: https://github.com/jpcy/xatlas; glTF-Validator: https://github.com/KhronosGroup/glTF-Validator; glTF-Transform: https://gltf-transform.dev/cli
- Blender as a Python module: https://docs.blender.org/api/current/info_advanced_blender_as_bpy.html
- † OpenMoonray PBR validity (albedo 0.032–0.87): https://docs.openmoonray.org/user-reference/how-to-guides/pbr-validity/; Unreal physically based materials: https://dev.epicgames.com/documentation/en-us/unreal-engine/physically-based-materials-in-unreal-engine; https://physicallybased.info/
- Unreal data validation: https://dev.epicgames.com/documentation/en-us/unreal-engine/data-validation-in-unreal-engine; Omniverse asset validator rules: https://docs.omniverse.nvidia.com/kit/docs/asset-validator/1.1.6/source/extensions/omni.asset_validator.core/docs/rules.html
- Moffat and Reiss, perceptual evaluation of synthesized sound effects (ACM TAP 2018): https://doi.org/10.1145/3165287
- AI audio licences: Stable Audio Open https://huggingface.co/stabilityai/stable-audio-open-1.0; AudioGen https://huggingface.co/facebook/audiogen-medium; MMAudio https://github.com/hkchengrex/MMAudio; ElevenLabs publishing terms https://elevenlabs.io/docs/help-center/legal/can-i-publish-the-content-i-generate-on-the-platform/llms.txt
- Steamworks content survey: https://partner.steamgames.com/doc/gettingstarted/contentsurvey; US Copyright Office, Part 2 (2025): https://www.copyright.gov/newsnet/2025/1060.html

### Design quality

- Koster, *Stars Reach pillars* (2024): https://www.raphkoster.com/?p=46260
- Tarczykowski, *Design pillars* (2020) — pillars as experiences: https://gamedeveloper.com/business/agile-game-development-part-2-design-pillars; Keenan on Duskers (2017): https://gamecenter.nyu.edu/2017-igf-interviews-duskers/
- Megill, *Building a basic story bible* (2019): https://www.gamedeveloper.com/design/building-a-basic-story-bible-for-your-game
- *The real buildings that inspired Control's Oldest House* (2019): https://gamedeveloper.com/art/the-real-buildings-that-inspired-i-control-i-s-oldest-house
- The Level Design Book — process https://book.leveldesignbook.com/process/overview and playtesting https://book.leveldesignbook.com/process/blockout/playtesting
- Smith and Whitehead, *Analyzing the expressive range of a level generator* (2010): https://www.pcgworkshop.com/archive/smith2010analyzing.pdf
- Shaker, Smith and Yannakakis, *PCG book* ch. 12 (2016): https://www.pcgbook.com/chapter12.pdf
- Summerville, *Expanding expressive range* (AIIDE 2018): https://ojs.aaai.org/index.php/AIIDE/article/view/13012
- Withington, Cook and Tokarchuk, PCG evaluation survey (FDG 2024): https://arxiv.org/html/2404.18657v1
- Boucher, Smith and Telliel, expressive-range taxonomy extended to generative AI (2024): https://ceur-ws.org/Vol-3926/paper9.pdf
- MAP-Elites: https://arxiv.org/abs/1504.04909; in PCG: https://arxiv.org/abs/1907.04053; constrained MAP-Elites for rooms (Alvarez et al. 2020): https://arxiv.org/abs/2003.03377
- Bıyık and Sürer, space-syntax evaluation of generated levels (2020, abstract only): https://avesis.metu.edu.tr/yayin/568c5026-3dfe-457e-b84a-0a417e8b6d2f/developing-a-space-syntax-based-evaluation-method-for-procedurally-generated-game-levels
- † Lopes, Liapis and Yannakakis, *Targeting horror via level and soundscape generation* (AIIDE 2015): https://ojs.aaai.org/index.php/AIIDE/article/view/12778; Sonancia: https://antoniosliapis.com/papers/sonancia_a_multi-faceted_generator_for_horror.pdf; RankTrace: https://www.antoniosliapis.com/papers/ranktrace_relative_and_unbounded_affect_annotation.pdf
- Booth, *The AI systems of Left 4 Dead* (2009): https://steamcdn-a.akamaihd.net/apps/valve/2009/ai_systems_of_l4d_mike_booth.pdf
- Thompson, *The perfect organism: the AI of Alien: Isolation* (2017): https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation
- EA, *Inside Dead Space: the intensity director*: https://www.ea.com/technology/news/inside-dead-space-4-the-intensity-director
- Until Dawn playtesting with GSR (2015): https://blog.playstation.com/?p=148419; Grip on repetition (2020): https://blog.playstation.com/?p=221993
- Graja, Lopes and Chanel, tension in a modified P.T. (IEEE ToG 2020): https://archive-ouverte.unige.ch/unige:139638
- Grizzard et al., habituation to repeated play (2015): https://pure.ewha.ac.kr/en/publications/the-thrill-is-gone-but-you-might-not-know-habituation-and-general/
- Bycer, *The balancing act of tension in horror game design* (2015): https://www.gamedeveloper.com/design/the-balancing-act-of-tension-in-horror-game-design
- *Pools* — tension without monsters, footprints as wayfinding: https://en.wikipedia.org/wiki/Pools_(video_game)
- Liapis et al., modelling affect for horror soundscapes (crowdsourced pairwise sound rankings): https://antoniosliapis.com/papers/modelling_affect_for_horror_soundscapes.pdf; PAGAN annotation tool: https://arxiv.org/abs/1907.01008
- LLM players as testers: Xiao and Yang (2024) https://arxiv.org/abs/2410.02829v1; Collins et al. (2025) https://arxiv.org/abs/2510.10930
- Bromley, *How many players do I need for a playtest?* (2022): https://gamesuserresearch.com/how-many-players-do-i-need-for-a-playtest/
- miniPXI test–retest (2024): https://arxiv.org/html/2407.19516v1; GEQ validation (CHI PLAY 2018) *[partly unverified]*: https://researcher.itu.dk/en/publications/systematic-review-and-validation-of-the-game-experience-questionn/
