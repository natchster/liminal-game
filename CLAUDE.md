# The Backrooms (native) — guide for coding agents

A first-person liminal-horror game for macOS on Apple Silicon, in Rust: a
Metal renderer, a synthesized CoreAudio soundscape, and levels generated from
a seed. This file is a map; the detail lives in the docs it points to.

| Read | For |
|---|---|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | crates, modules, one frame, level build, creatures and pacing, rendering, audio |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) | conventions, determinism rules, adding a placer, tests and fixtures, headless tools |
| [`docs/TOOLING_ROADMAP.md`](docs/TOOLING_ROADMAP.md) | the quality tooling being built, and why |
| [`docs/SOURCES.md`](docs/SOURCES.md) | the research behind the game's design |

## Commands

```sh
cargo xtask check --fast       # fmt, clippy, the quick tests — run before you say you are done
cargo xtask check              # everything (several minutes, release build)
cargo run --release            # play
cargo xtask bundle             # dist/Backrooms.app (signed)
```

Headless evidence (never capture the screen — render offscreen):

```sh
cargo run --release -- --screenshot out.png --level 1 --seed 4242 --play --frames 90
cargo run --release -- --dump-level DIR --seed 4242 --levels 1,2 --screenshot x --views
cargo run --release -- --screenshot out.png --seed 4242 --play --showcase hound:3 [--death]
cargo test --release -p backrooms --test listen -- --nocapture    # WAVs + loudness PNGs in target/listen/
cargo test --release -p backrooms --test pacing -- --nocapture    # director phases with a cautious bot
```

Re-recording reference data (only after an intended change; say why in the commit):

```sh
REGEN=1 cargo test --release -p game --test entities
cargo test --release -p game --test archgen --test world_queries --test world_build -- --ignored rerecord
cargo test --release -p backrooms --test reference_levels -- --ignored rerecord
```

## Invariants (each has cost real time when broken)

1. **Levels are a pure function of `(run_seed, level)`.** In a level build,
   randomness comes only from `rnd()` and its helpers; per-frame code uses
   the native stream (`with_native`, `native_rnd`). No clock, no other RNG, no
   `HashMap` iteration order, nothing left over from the previous level. A new
   random draw in a placer shifts every later placer: re-record and say so.
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

## Reviews

Never grade your own work. For look or feel, use a blind review: a fresh
subagent that sees only the images (both orders, keys hidden) and a rubric,
and counts disagreement as a tie. Fear, sound quality and taste are finally
decided by people.
