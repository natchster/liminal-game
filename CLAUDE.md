# The Backrooms (native) — guide for coding agents

A first-person liminal-horror game for macOS on Apple Silicon, in Rust: a
Metal renderer, a synthesized CoreAudio soundscape, and levels generated from
a seed. This file is a map; the detail lives in the docs it points to.

| Read | For |
|---|---|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | crates, modules, one frame, level build, creatures and pacing, rendering, audio |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) | conventions, determinism rules, adding a placer, tests and fixtures, headless tools |
| [`docs/LORE.md`](docs/LORE.md) | the canon: places, creatures, anomalies, and how lore may reach the player |
| [`docs/PILLARS.md`](docs/PILLARS.md) | what the game is for; quote the rule a content change serves |
| [`docs/GAMEPLAY.md`](docs/GAMEPLAY.md) | what a level asks of the player (her trail, her camp, the notebook) and the plan behind it |
| [`docs/UI.md`](docs/UI.md) | the controls (one binding table, what a run needs in reach) and the screens, and the plan behind them |
| [`docs/MEGASTRUCTURE.md`](docs/MEGASTRUCTURE.md) | the Building: the vast floors above the rooms (levels 1–3), the self-assembly, and the plan behind them |
| [`docs/ENTITIES.md`](docs/ENTITIES.md) | the creatures: five rules, one brain, fairness, bodies, how each is beaten, and the plan behind them ([`docs/research/`](docs/research/README.md) for the research) |
| [`docs/STYLE.md`](docs/STYLE.md) | the art bible; its bands are checked by `cargo xtask shots` |
| [`docs/ROOMS.md`](docs/ROOMS.md) | the rooms up close: the field of view, corners, the photographed details, the ceiling's grid, and the plan behind them |
| [`docs/VENDING.md`](docs/VENDING.md) | the vending machine: a real one, its rules (the coin, the silence, rocking it), and the plan behind them |
| [`docs/WINDOWS.md`](docs/WINDOWS.md) | the windows: cut into real walls, the outside they show, the one behind the glass, and the plan behind them |
| [`docs/PROPS.md`](docs/PROPS.md) | the props made real (the census of every kind, how one is remodelled), barricades that close the way, the props a hand can use, and the plan behind them |
| [`docs/SECRETS.md`](docs/SECRETS.md) | her marks (one hand, many kinds), the place's quiet answers, the hiding places and what is put away in them (what stays with you), and the plan behind them |
| [`docs/VOICES.md`](docs/VOICES.md) | the voices: personas, the text-to-speech pipeline (models licensed for sale, run here), the evaluation of words, tone and emotion, your asides and the director that chooses them, and the plan behind them |
| [`docs/CUTSCENES.md`](docs/CUTSCENES.md) | cutscenes: the moments that borrow your eyes and give them back (the way in from the title, a broken lift's shaft, dying), the bars that say your hands are taken, the engine that stores, plays and tests them, its pipeline, and the plan behind them |
| [`docs/AGENTS.md`](docs/AGENTS.md) | this tooling itself: the hooks, rules, skills and reviewers under `.claude/`, written for this game and grown with it |
| [`docs/content.json`](docs/content.json) | the content ledger: each family and the checks it must pass |
| [`docs/MILESTONES.md`](docs/MILESTONES.md) | every milestone still open across the docs, in one queue, done one by one |
| [`docs/QUALITY.md`](docs/QUALITY.md) | grades, evidence and open issues per subsystem |
| [`docs/TOOLING_ROADMAP.md`](docs/TOOLING_ROADMAP.md) | the quality tooling being built, and why |
| [`docs/SOURCES.md`](docs/SOURCES.md) | the research behind the game's design |

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
cargo xtask bench                                # performance vs docs/PERF.md, perf/history.csv and perf/creatures.csv (records a run)
cargo xtask drift                                # every gate → docs/content.json passes, docs/QUALITY.md Latest sweep
cargo xtask voice doctor [--smoke]               # the voice toolchain: uv, tools/voice/{synth,eval}, 14 pinned models, licences
cargo xtask voice reference --persona P [--emotion E] --candidates N [--write]   # a persona's voice (and an emotion's) from its description
cargo xtask voice build --persona P [--model M]  # every take of P's lines (text-to-speech on MLX), cleaned
cargo xtask voice eval --persona P [--model M]   # words, emotion, tone of voice, voice: report.md and the gate
cargo xtask voice write --persona P              # the chosen takes into crates/game/assets/voice/ and MANIFEST.json
cargo xtask voice selftest                       # the evaluation against known answers
cargo xtask cutscene check|write|list            # cutscenes: assets/src/cutscenes/*.toml → crates/game/assets/cutscenes.json, the game's rules
cargo xtask cutscene preview ID [--stage S]     # the sources as they stand, played on each stage: frames, a contact sheet, timeline, contract (target/xtask/cutscene/ID/)
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
back with its `ERROR` lines). See `docs/DEVELOPMENT.md`.

## Definition of done

- `cargo xtask check --fast` passes (and the full check for anything touching
  level generation, creatures, rendering or audio).
- A change you can see or hear ends in evidence: a headless PNG you looked
  at, a `listen` WAV/PNG, or a `pacing` log — not a description.
- Docs that describe what you changed are updated in the same commit.
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
the game and changed with it (`docs/AGENTS.md`): when the game grows a
system, it grows a skill; when a mistake repeats, it becomes a check.

Content work has a skill each (`.claude/skills/`): `add-zone`, `add-prop`,
`add-creature`, `add-anomaly`, `add-sound`, `tune-pacing`, `visual-review`,
`drift-sweep` (weekly, in a fresh session: `cargo xtask drift`),
`regen-fixtures` (the only way to re-record; invoked by name), and the
voices: `voice-persona` (who speaks: the persona, its reference voice),
`voice-lines` (its lines, made, judged and into the game) and `voice-eval`
(reading the evaluation, a bake-off, changing a threshold only through
known answers). Each ends in its gates, evidence and a `docs/content.json`
entry.

Never grade your own work. The reviewers (`.claude/agents/`) are read-only
and see only their evidence: `art-critic` (blind A/B pairs from
`cargo xtask ab prepare`, scored with `cargo xtask ab score`), `lore-keeper`,
`level-analyst`, `audio-analyst`, `code-reviewer`. Fear, sound quality and
taste are finally decided by people: `docs/PLAYTEST.md` is the session
script, and their verdicts calibrate the reviewers.
