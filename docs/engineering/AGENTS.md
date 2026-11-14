# The agent tooling: hooks, rules, skills and reviewers

**This game is built with coding agents, and the agents' tooling is built
with the game.** Everything under `.claude/` was written for The Backrooms
and nothing else: hooks that hold the game's invariants, rules that load
when an agent touches a subsystem, a skill for each kind of content the game
has, and read-only reviewers that judge one kind of evidence each. They are
not stock tools: each was added when the game grew a system that needed it
(the voice skills came with the voices, `docs/design/VOICES.md`), each names the
game's own commands, files and gates, and each is changed when the game
changes. When the same mistake happens twice, it becomes a rule, a lint, a
test or a step in a skill, so it cannot happen a third time (the *steering
loop*, `docs/plan/TOOLING_ROADMAP.md`). `CLAUDE.md` is the map an agent reads
first; this file is the inventory.

All of it is committed. Nothing under `.claude/` holds a secret, a token, an
address or a personal path (checked 2026-11-09, the files and their git
history); per-user settings stay in a local settings file that git ignores.
`tests/structure.rs` fails when a path a skill, rule or doc names does not
exist.

## Hooks

`.claude/settings.json` runs `.claude/hooks/xtask-hook.sh`, which runs
`cargo xtask hook <event>` (`xtask/src/hook.rs`; the built binary directly
when it is current, so an edit never waits on cargo's build lock).

| Event | What it does |
|---|---|
| before an edit | refuses an edit outside this repository (`CLAUDE.md`, invariant 4) and a hand edit of recorded reference data (fixtures, reference levels, goldens): those change only through REGEN, rerecord or `--accept`; and, while the session works from GitHub words that are not all maintainers' (the trust tool's task marker), an edit of the protected paths ([`TRUST.md`](TRUST.md)) |
| after an edit | rustfmt on the edited `.rs` file; a file it cannot parse is reported back |
| on stop | `cargo xtask check --fast` when the code changed since the last passing check; a failure sends the agent back with its `ERROR` lines (at most three times in a row) |
| after compaction | recent commits, uncommitted files and the roadmap's progress re-injected |

## Rules

`.claude/rules/*.md` load only when an agent touches the files their
`paths:` name: the subsystem's invariants where they matter, kept short, and
moved into a test or a lint as soon as one can check them.

| Rule | Loads for | Holds |
|---|---|---|
| `.claude/rules/world.md` | level generation, placers, core props | determinism (draw order is output), one purpose a zone, walls off the grid, every level walkable; the re-record commands |
| `.claude/rules/entities.md` | creatures, the director, sessions | one brain, fairness in `creature.rs`, per-frame draws from the native stream, the director owns pressure, the ecology, what they learn, bodies |
| `.claude/rules/visuals.md` | the renderer, shaders, materials, textures, visuals | light from the light map, realistic albedo (not cartoonish), canvas textures on streams of their own, photographed surfaces from CC0 sources, shader compile cost, the goldens and the albedo lint |
| `.claude/rules/audio.md` | the audio code | every sound from a place (your own voice the one exception), carry, the hush, the loudness gate, you cannot hear |
| `.claude/rules/cutscenes.md` | the cutscene engine, its sources, the test engine, the bars | it borrows your eyes, never your place; play never eases from its pose; no random draw in the engine; cues in place, each once; hands taken means no harm; the bundle is compiled; every cutscene keeps the contract on its stages |

## Skills

`.claude/skills/<name>/SKILL.md`: a playbook for each kind of content, from
the brief against the canon to the gates, the evidence and the ledger entry
(`docs/plan/content.json`).

| Skill | For |
|---|---|
| `add-zone` | a zone archetype or purpose: the Level Design Book order, blockout checks, bot walk, lighting, set dressing last |
| `add-prop` | a prop placer: theme and purpose, real materials, its Blender part, the look settle, the material lint, goldens |
| `add-creature` | a creature: canon, body (Blender, MakeHuman CC0), brain, fairness, what it learns, ecology, rules and arena tests |
| `add-anomaly` | an anomaly level: canon and attribution, its run rules, its own notes, tests and goldens |
| `add-sound` | a sound: synthesis, place, carry, the loudness and listening gates, a person's ears |
| `tune-pacing` | the director: measure from bot runs, change one parameter, compare distributions |
| `visual-review` | a blind A/B review of frames with a fresh reviewer, both orders, the verdict logged |
| `regen-fixtures` | the only way to re-record reference data, the diff reviewed and the reason kept |
| `drift-sweep` | the weekly sweep: every gate, the ledger and the scorecard, what people caught turned into checks |
| `voice-persona` | a voice persona: the canon check, the persona file, its designed reference voice and emotional references |
| `voice-lines` | a persona's lines: made, judged, into the game's assets and code, heard, a person's ears |
| `voice-eval` | the voice evaluation: reading the report, a bake-off between models, thresholds changed only through known answers |
| `visual-playtest` | the visual playtest of an area, a level or a set: when (by hand, and after any major change on the places it touched), the run, the report and sheets, the inspector, reproducing with `--shots-at`, the record in `docs/research/playtests/visual/`, the fix a kind at a time, the tour again, what it cannot see (followed end to end by a fresh agent before it was committed) |
| `add-cutscene` | a cutscene: whether it should be one, its source, its trigger and stage, the preview, the contract suite, goldens, the canon (followed end to end by a fresh agent before it was committed) |
| `implement-issue` | a GitHub issue (or a pull request's comments): read only through its trusted brief, acted on only where maintainers wrote or approved it, the protected paths left alone when any words came from outside, the pull request the trust gate expects ([`TRUST.md`](TRUST.md)) |

## Trust

The repository is public, and the agents implement its issues: whose words
they act on is settled before they read anything ([`TRUST.md`](TRUST.md)).
The owner's and maintainers' words are trusted; anyone else's only once a
maintainer approves them with `/trust`, and only as they stood then. The
*protected paths* (this tooling: `.claude/`, the workflows and the trust
tool under `.github/`, `CLAUDE.md`, the hook) change only by maintainers:
the trust gate refuses anyone else's pull request touching them, and holds
a maintainer's, when it implements others' words, until a maintainer
approves its exact commit (`/trust-protected <sha>`).

## Reviewers

`.claude/agents/*.md`: read-only agents that see only their evidence and
never grade their own author's work. Each judges one kind of thing; people
have the last word, and their verdicts calibrate the reviewers.

| Reviewer | Sees | Says |
|---|---|---|
| `art-critic` | blind A/B pairs of frames (`cargo xtask ab prepare`), `docs/design/STYLE.md` | A, B or tie a pair, with its evidence; flags anything broken. Its memory (`.claude/agent-memory/art-critic/MEMORY.md`) keeps people's verdicts and its calibration on known answers (`cargo xtask calibrate`), so its taste is corrected |
| `lore-keeper` | a diff or new text, `docs/design/LORE.md`, `docs/design/PILLARS.md` | contradictions with the canon and broken rules of how lore reaches the player, with quotes |
| `level-analyst` | the expressive-range report, the level sweep, the zone views | what changed in level generation and why; it explains, it does not approve |
| `audio-analyst` | the loudness gate, `listen` renders, ambient logs | the numbers; never whether a sound is good (that is for ears) |
| `visual-inspector` | a visual playtest's folder (its report.md, the sheets, the flagged pictures), `docs/engineering/VISUAL_PLAYTEST.md`, `docs/design/STYLE.md` | each detector's flag confirmed, false or unsure, and what the sheets show (out of place, clip, flicker, gap) with its view, severity and confidence: a filter for people, never the gate (vision models miss half and imagine clipping at near contact) |
| `code-reviewer` | a diff, `CLAUDE.md`, `docs/engineering/DEVELOPMENT.md` | layering, determinism and module-size gaps; no style preferences |

## How they grow with the game

- **A new system gets its skill** when it is built, written from what was
  learned building it (the voice skills from the voice pipeline's bake-off
  and known answers), and **followed end to end by a fresh agent** before it
  is committed: what that agent could not do from the skill alone is fixed
  in the skill or in the code.
- **A mistake made twice becomes a check**: a hook, a rule, a lint, a test
  or a skill step (the hook's refusal of hand-edited fixtures, the structure
  test that skills name only files that exist, the voice evaluation's known
  answers).
- **A reviewer is calibrated before it is trusted**: on damaged goldens with
  known answers (`cargo xtask calibrate`), then on people's verdicts
  (`docs/research/PLAYTEST.md`); its accuracy is logged
  (`crates/backrooms/tests/review/calibration.jsonl`).
- **Adding one**: a skill is `.claude/skills/<name>/SKILL.md` with a `name`
  and a `description` that says when to use it, numbered steps that end in
  gates, evidence and the ledger; a reviewer is `.claude/agents/<name>.md`
  with read-only tools and the one kind of evidence it may see; list it
  here, in `CLAUDE.md` and in `docs/engineering/DEVELOPMENT.md`.
