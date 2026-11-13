# The Backrooms (native) — guide for coding agents

A first-person liminal-horror game for macOS on Apple Silicon, in Rust: a
Metal renderer, a synthesized CoreAudio soundscape, and levels generated from
a seed. This file is a map; the detail lives in the docs it points to.

The docs are sorted by kind ([`docs/README.md`](docs/README.md)). Put a new doc, or a new
section, where its kind is, and nowhere else:

- **`docs/design/`: the game.** What it is, how it looks, sounds and plays,
  and why. No milestones or progress: those are its plan, `docs/plan/<TOPIC>.md`
  under the same name, which its *The work* section links.
- **`docs/research/`: what the design rests on.** Outside sources, research
  notes, censuses, and playtests (people's sessions, the visual playtest's
  records). Kept as found.
- **`docs/plan/`: the work.** The milestone queue, each topic's milestones and
  progress, the tooling roadmap, the quality scorecard and the content ledger.
- **`docs/engineering/`: the code.** How it is built and how to work on it.

`tests/structure.rs` holds this: the docs root has only the index and the four
folders, and every `docs/…` path the repository names exists.

**Design**

| Read | For |
|---|---|
| [`docs/design/PILLARS.md`](docs/design/PILLARS.md) | what the game is for; quote the rule a content change serves |
| [`docs/design/LORE.md`](docs/design/LORE.md) | the canon: places, creatures, anomalies, and how lore may reach the player |
| [`docs/design/GAMEPLAY.md`](docs/design/GAMEPLAY.md) | what a level asks of the player (her trail, her camp, the notebook) |
| [`docs/design/UI.md`](docs/design/UI.md) | the controls (one binding table, what a run needs in reach) and the screens |
| [`docs/design/STYLE.md`](docs/design/STYLE.md) | the art bible; its bands are checked by `cargo xtask shots` |
| [`docs/design/MEGASTRUCTURE.md`](docs/design/MEGASTRUCTURE.md) | the Building: the vast floors above the rooms (levels 1–3), the self-assembly |
| [`docs/design/ROOMS.md`](docs/design/ROOMS.md) | the rooms up close: the field of view, corners, the photographed details, the ceiling's grid |
| [`docs/design/ENTITIES.md`](docs/design/ENTITIES.md) | the creatures: five rules, one brain, fairness, bodies, how each is beaten ([`docs/research/`](docs/research/README.md) for the research) |
| [`docs/design/PROPS.md`](docs/design/PROPS.md) | the props made real (the census of every kind, how one is remodelled), barricades that close the way, the props a hand can use |
| [`docs/design/SECRETS.md`](docs/design/SECRETS.md) | her marks (one hand, many kinds), the place's quiet answers, the hiding places and what is put away in them (what stays with you) |
| [`docs/design/VENDING.md`](docs/design/VENDING.md) | the vending machine: a real one, its rules (the coin, the silence, rocking it) |
| [`docs/design/WINDOWS.md`](docs/design/WINDOWS.md) | the windows: cut into real walls, the outside they show, the one behind the glass |
| [`docs/design/VOICES.md`](docs/design/VOICES.md) | the voices: personas, the text-to-speech pipeline (models licensed for sale, run here), the evaluation of words, tone and emotion, your asides and the director that chooses them |
| [`docs/design/CUTSCENES.md`](docs/design/CUTSCENES.md) | cutscenes: the moments that borrow your eyes and give them back (the way in from the title, a broken lift's shaft, dying), the bars that say your hands are taken, the engine that stores, plays and tests them, its pipeline |

**Engineering**

| Read | For |
|---|---|
| [`docs/engineering/ARCHITECTURE.md`](docs/engineering/ARCHITECTURE.md) | crates, modules, one frame, level build, creatures and pacing, rendering, audio |
| [`docs/engineering/DEVELOPMENT.md`](docs/engineering/DEVELOPMENT.md) | conventions, determinism rules, adding a placer, tests and fixtures, headless tools |
| [`docs/engineering/AGENTS.md`](docs/engineering/AGENTS.md) | this tooling itself: the hooks, rules, skills and reviewers under `.claude/`, written for this game and grown with it |
| [`docs/engineering/PERF.md`](docs/engineering/PERF.md) | the performance budgets `cargo xtask bench` holds the game to, and what each speed-up found |
| [`docs/engineering/VISUAL_PLAYTEST.md`](docs/engineering/VISUAL_PLAYTEST.md) | the visual playtest: a tour of the edges of the walkable floor (looked at, along, up and down), its detectors (gaps, flicker, clipping, out of place), its report and its reviewer; run by hand, and after any major change on the places it touched |

**Plan**

| Read | For |
|---|---|
| [`docs/plan/MILESTONES.md`](docs/plan/MILESTONES.md) | every milestone still open across the docs, in one queue, done one by one |
| `docs/plan/<TOPIC>.md` | each design topic's work: its brief and analysis, milestones and progress, what it left open (`ENTITIES.md`, `MEGASTRUCTURE.md`, `PROPS.md`, `VOICES.md`, …, `VISUAL_PLAYTEST.md`) |
| [`docs/plan/QUALITY.md`](docs/plan/QUALITY.md) | grades, evidence and open issues per subsystem |
| [`docs/plan/content.json`](docs/plan/content.json) | the content ledger: each family and the checks it must pass |
| [`docs/plan/TOOLING_ROADMAP.md`](docs/plan/TOOLING_ROADMAP.md) | the quality tooling being built, and why |

**Research**

| Read | For |
|---|---|
| [`docs/research/SOURCES.md`](docs/research/SOURCES.md) | the research behind the game's design |
| [`docs/research/README.md`](docs/research/README.md) | the research notes, each answering a brief, with every source linked |
| [`docs/research/PLAYTEST.md`](docs/research/PLAYTEST.md) | the playtest session script: what people are asked, and what they found (records in `docs/research/playtests/`) |


## Commands

```sh
cargo xtask check --fast       # fmt, clippy, the quick tests — run before you say you are done
cargo xtask check              # everything (several minutes, release build)
cargo xtask shots              # golden frames + look statistics (part of the check)
cargo xtask range --look       # the generator's expressive range vs its baseline (full check)
cargo xtask ab prepare --before DIR --after DIR   # blind pairs (both orders); then `ab score`
cargo xtask calibrate prepare [--mild]            # known-answer pairs to measure a reviewer
cargo xtask assets props [--write]               # rebuild the prop meshes in Blender, compare part by part
cargo xtask assets textures [--write]            # rebuild the Building's photographed surfaces (CC0 photographs), compare
cargo xtask assets creatures [--write]           # rebuild the creatures' bodies in Blender (MakeHuman CC0 bodies), compare
cargo xtask bench                                # performance vs docs/engineering/PERF.md, perf/history.csv and perf/creatures.csv (records a run)
cargo xtask drift                                # every gate → docs/plan/content.json passes, docs/plan/QUALITY.md Latest sweep
cargo xtask voice doctor [--smoke]               # the voice toolchain: uv, tools/voice/{synth,eval}, 14 pinned models, licences
cargo xtask voice reference --persona P [--emotion E] --candidates N [--write]   # a persona's voice (and an emotion's) from its description
cargo xtask voice build --persona P [--model M]  # every take of P's lines (text-to-speech on MLX), cleaned
cargo xtask voice eval --persona P [--model M]   # words, emotion, tone of voice, voice: report.md and the gate
cargo xtask voice write --persona P              # the chosen takes into crates/game/assets/voice/ and MANIFEST.json
cargo xtask voice selftest                       # the evaluation against known answers
cargo xtask cutscene check|write|list            # cutscenes: assets/src/cutscenes/*.toml → crates/game/assets/cutscenes.json, the game's rules
cargo xtask cutscene preview ID [--stage S]     # the sources as they stand, played on each stage: frames, a contact sheet, timeline, contract (target/xtask/cutscene/ID/)
cargo xtask playtest visual [--seeds 4242,7] [--levels 1,2,3] [--area x0,z0,x1,z1]   # by hand, never in check: the edges toured, gaps, flicker, clip, plain, coplanar; report.md, sheets (target/xtask/playtest/visual/)
cargo run --release            # play
cargo xtask bundle             # dist/Backrooms.app (signed)
```

Headless evidence (never capture the screen — render offscreen):

```sh
cargo run --release -- --fresh --screenshot out.png --level 1 --seed 4242 --play --frames 90
cargo run --release -- --dump-level DIR --seed 4242 --levels 1,2 --screenshot x --views   # + exit, camp, trail views
cargo run --release -- --dump-level DIR --seed 4242 --levels 4,5 --screenshot x --prop-census 1 [--census-only core_wet_floor_sign] [--census-use]   # every kind of prop, framed (as --fresh: the same shot every time); DIR/census/census.tsv keeps each camera; --census-use uses every verb first
cargo run --release -- --dump-level DIR --seed 4242 --levels 5 --screenshot x --shots-at "name:x,y,z,yaw,pitch;…"   # those cameras again (a prop before and after)
cargo run --release -- --dump-level DIR --seed 4242 --levels 1,2,3 --screenshot x --far-figure   # the figure across the void from 4 m, 12 m, 25 m and its own view, each also without it (_none); its cameras printed for --shots-at
cargo run --release -- --screenshot out.png --level 4 --seed 4242 --play --showcase hound:3 [--death]   # a floor of the rooms: nothing walks the Building
cargo run --release -- --fresh --screenshot out.png --frames 120                      # the title over the live Well (level 1)
cargo run --release -- --fresh --screenshot out.png --seed 4242 --play --frames 90 --screen pause   # or leave, settings, brightness, controls, dead, ending, notebook, notebook-typed, cutscene, cutscene-skip; on levels 1-3 also fog-below, fog-in, figure, cell-answered, cell, cell-gave; on level 1 lift
cargo test --release -p backrooms --test listen -- --nocapture    # WAVs + loudness PNGs in target/listen/
cargo test --release -p backrooms --test pacing -- --nocapture    # director phases with a cautious bot
cargo test --release -p backrooms --test personas -- --nocapture  # explorer, rusher, hider; the shape of the fear
cargo test --release -p backrooms --test asides -- --nocapture    # your asides on real floors: what was weighed, said, let go, and why
cargo test --release -p backrooms --test cutscenes                 # the cutscene player, and every cutscene's contract on each of its stages
cargo run --release -- --cutscene ID [--stage S] --sheet DIR [--every 0.25]   # one cutscene's frames, sheet, timeline, contract (or --screenshot F --cut-at T)
cargo run --release -- --tour DIR --seed 4242 --levels 1,2,3 [--tour-budget 180] [--tour-area x0,z0,x1,z1] --width 640 --height 400   # the visual playtest's renders: the edges' views, gaps and flicker (DIR/L<n>/views.tsv)
```

Re-recording reference data (only after an intended change; say why in the commit):

```sh
cargo xtask shots --accept NAME,…|all --reason "…"       # after looking at the frames and error maps
cargo xtask shots --accept-look NAME,…|all --reason "…"  # rarely: the look itself is meant to change
cargo xtask range --accept --reason "…"                  # after reading target/xtask/range/report.md
REGEN=1 cargo test --release -p game --test entities
cargo test --release -p game --test archgen --test world_queries --test world_build -- --ignored rerecord
cargo test --release -p backrooms --test reference_levels -- --ignored rerecord
cargo test --release -p game --test core_props_head -- --ignored rerecord   # the core props
cargo test --release -p game --test core_props_tail -- --ignored rerecord   # their second half
cargo test --release -p backrooms --test loudness -- --ignored rerecord   # after listening
```

## Invariants (each has cost real time when broken)

1. **Levels are a pure function of `(run_seed, level)`.** In a level build,
   randomness comes only from `rnd()` and its helpers; per-frame code uses
   the native stream (`with_native`, `native_rnd`); what is made once
   whenever first needed (`canvas_tex`) draws from a stream of its own
   (`with_own_stream`). No clock, no other RNG, no
   `HashMap` iteration order, nothing left over from the previous level, and
   nothing from the save (the build runs with nothing read; what depends on
   the run's progress is fitted afterwards, like `fit_lore_notes`). A new
   random draw in a placer shifts every later placer: re-record and say so.
   Likewise a native draw added or removed at a level's start shifts every
   frame after it (grain, animation, every golden): reuse an existing draw.
2. **Light and sound belong to places.** Never tie ambient light, fog or
   ambience to the player's position, and no lamp cones; light comes from the
   light map, and sounds come from props or fixed anchors (`play_from`).
3. **Never capture the user's screen.** Use the headless modes above and read
   the PNG.
4. **Never modify anything outside this repository**, whatever is outside
   it: edits stay inside the repository (the pre-edit hook refuses the rest).
5. **Layering.** `engine` never depends on `game`; `game` never touches the
   window, Metal or CoreAudio devices. The game runs headless; tests rely on it.
6. **Shader cost.** A texture read added to a global shader patch multiplies
   pipeline compile time; judge shader changes by level load plus first frame.
7. **Fixtures change only through REGEN / rerecord**, never by hand edits, and
   only for an intended change whose diff you have reviewed.

## Hooks

`.claude/settings.json` runs `cargo xtask hook …`: edits outside this
repository and to recorded fixtures are refused, edited `.rs` files are formatted, and a
stop runs `cargo xtask check --fast` when code changed (a failure sends you
back with its `ERROR` lines). See `docs/engineering/DEVELOPMENT.md`.

## Definition of done

- `cargo xtask check --fast` passes (and the full check for anything touching
  level generation, creatures, rendering or audio).
- A change you can see or hear ends in evidence: a headless PNG you looked
  at, a `listen` WAV/PNG, or a `pacing` log — not a description.
- Docs that describe what you changed are updated in the same commit.
- After a major change (level generation, the architecture, a placer or
  core prop, materials, textures, lighting, the camera), the visual
  playtest on every level or place it touched (`visual-playtest`: `cargo
  xtask playtest visual --levels … [--area …]`), its findings read and
  recorded in `docs/research/playtests/visual/`, and fixed or said why not.
- If the same mistake happens twice, add a rule, lint or test so it cannot
  happen a third time (see the roadmap's "steering loop").

## Commits

Commit only when asked. Use the project identity per command, without
changing git config:

```sh
git -c user.name="Pavel Krivoruchko" -c user.email="pavel@lilium.me" commit …
```

End the message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Skills and reviewers

The skills, reviewers, rules and hooks are this project's own, written for
the game and changed with it (`docs/engineering/AGENTS.md`): when the game grows a
system, it grows a skill; when a mistake repeats, it becomes a check.

Content work has a skill each (`.claude/skills/`): `add-zone`, `add-prop`,
`add-creature`, `add-anomaly`, `add-sound`, `tune-pacing`, `visual-review`,
`drift-sweep` (weekly, in a fresh session: `cargo xtask drift`),
`regen-fixtures` (the only way to re-record; invoked by name), and the
voices: `voice-persona` (who speaks: the persona, its reference voice),
`voice-lines` (its lines, made, judged and into the game) and `voice-eval`
(reading the evaluation, a bake-off, changing a threshold only through
known answers), `add-cutscene` (a moment that borrows your eyes: its
source, trigger, stage, preview, contract and goldens), and
`visual-playtest` (the edges of an area, a level or a set toured: gaps,
flicker, clipping, things out of place; reproduced, recorded, fixed, toured
again). Each ends in its gates, evidence and a `docs/plan/content.json` entry
(the playtest in `docs/research/playtests/visual/`).

Never grade your own work. The reviewers (`.claude/agents/`) are read-only
and see only their evidence: `art-critic` (blind A/B pairs from
`cargo xtask ab prepare`, scored with `cargo xtask ab score`), `lore-keeper`,
`level-analyst`, `audio-analyst`, `code-reviewer`, `visual-inspector` (a
visual playtest's report and sheets: a filter, never the gate). Fear, sound quality and
taste are finally decided by people: `docs/research/PLAYTEST.md` is the session
script, and their verdicts calibrate the reviewers.
