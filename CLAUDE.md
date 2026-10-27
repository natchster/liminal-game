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
| [`docs/STYLE.md`](docs/STYLE.md) | the art bible; its bands are checked by `cargo xtask shots` |
| [`docs/content.json`](docs/content.json) | the content ledger: each family and the checks it must pass |
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
cargo xtask bench                                # performance vs docs/PERF.md and perf/history.csv (records a run)
cargo xtask drift                                # every gate → docs/content.json passes, docs/QUALITY.md Latest sweep
cargo run --release            # play
cargo xtask bundle             # dist/Backrooms.app (signed)
```

Headless evidence (never capture the screen — render offscreen):

```sh
cargo run --release -- --fresh --screenshot out.png --level 1 --seed 4242 --play --frames 90
cargo run --release -- --dump-level DIR --seed 4242 --levels 1,2 --screenshot x --views   # + exit, camp, trail views
cargo run --release -- --screenshot out.png --seed 4242 --play --showcase hound:3 [--death]
cargo run --release -- --fresh --screenshot out.png --frames 120                      # the title over the live Well (level 1)
cargo run --release -- --fresh --screenshot out.png --seed 4242 --play --frames 90 --screen pause   # or leave, settings, brightness, controls, dead, ending, notebook, notebook-typed; on levels 1-3 also fog-below, fog-in, figure; on level 1 lift
cargo test --release -p backrooms --test listen -- --nocapture    # WAVs + loudness PNGs in target/listen/
cargo test --release -p backrooms --test pacing -- --nocapture    # director phases with a cautious bot
cargo test --release -p backrooms --test personas -- --nocapture  # explorer, rusher, hider; the shape of the fear
```

Re-recording reference data (only after an intended change; say why in the commit):

```sh
cargo xtask shots --accept NAME,…|all --reason "…"       # after looking at the frames and error maps
cargo xtask shots --accept-look NAME,…|all --reason "…"  # rarely: the look itself is meant to change
cargo xtask range --accept --reason "…"                  # after reading target/xtask/range/report.md
REGEN=1 cargo test --release -p game --test entities
cargo test --release -p game --test archgen --test world_queries --test world_build -- --ignored rerecord
cargo test --release -p backrooms --test reference_levels -- --ignored rerecord
cargo test --release -p backrooms --test loudness -- --ignored rerecord   # after listening
```

## Invariants (each has cost real time when broken)

1. **Levels are a pure function of `(run_seed, level)`.** In a level build,
   randomness comes only from `rnd()` and its helpers; per-frame code uses
   the native stream (`with_native`, `native_rnd`). No clock, no other RNG, no
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
4. **Never modify the HTML/JS version in the parent folder** (`../`). It is a
   read-only reference.
5. **Layering.** `engine` never depends on `game`; `game` never touches the
   window, Metal or CoreAudio devices. The game runs headless; tests rely on it.
6. **Shader cost.** A texture read added to a global shader patch multiplies
   pipeline compile time; judge shader changes by level load plus first frame.
7. **Fixtures change only through REGEN / rerecord**, never by hand edits, and
   only for an intended change whose diff you have reviewed.

## Hooks

`.claude/settings.json` runs `cargo xtask hook …`: edits to the parent folder
and to recorded fixtures are refused, edited `.rs` files are formatted, and a
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

Content work has a skill each (`.claude/skills/`): `add-zone`, `add-prop`,
`add-creature`, `add-anomaly`, `add-sound`, `tune-pacing`, `visual-review`,
`drift-sweep` (weekly, in a fresh session: `cargo xtask drift`), and
`regen-fixtures` (the only way to re-record; invoked by name). Each ends
in its gates, evidence and a `docs/content.json` entry.

Never grade your own work. The reviewers (`.claude/agents/`) are read-only
and see only their evidence: `art-critic` (blind A/B pairs from
`cargo xtask ab prepare`, scored with `cargo xtask ab score`), `lore-keeper`,
`level-analyst`, `audio-analyst`, `code-reviewer`. Fear, sound quality and
taste are finally decided by people: `docs/PLAYTEST.md` is the session
script, and their verdicts calibrate the reviewers.
