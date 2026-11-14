# The docs

Sorted by kind; a doc, or a section of one, goes where its kind is (the
rule is in `CLAUDE.md`, and `crates/backrooms/tests/structure.rs` holds it):

| Folder | Holds | Not |
|---|---|---|
| [`design/`](design/) | the game: what it is, how it looks, sounds and plays, and why | milestones, progress (its plan, `plan/<TOPIC>.md` by the same name) |
| [`research/`](research/README.md) | what the design rests on: outside sources, research notes, censuses, playtests (people's sessions; the visual playtest's records in [`research/playtests/`](research/playtests/visual/README.md)) | decisions (they go in design) |
| [`plan/`](plan/MILESTONES.md) | the work: the milestone queue, each topic's milestones and progress, the tooling roadmap, the quality scorecard, the content ledger | how the game should be (design) |
| [`engineering/`](engineering/ARCHITECTURE.md) | the code: how it is built, its conventions and tools, its performance | game design |

A design doc names its plan, and a plan names the design it serves; a
design doc may sum up the research it rests on and list its sources, but
the research itself is in `research/`, kept as found and summed up in
[`research/SOURCES.md`](research/SOURCES.md). A dated heading is the work's or
a study's, never a design's (`tests/structure.rs` fails on one).

**Design**

| Read | For |
|---|---|
| [`design/PILLARS.md`](design/PILLARS.md) | what the game is for; quote the rule a content change serves |
| [`design/LORE.md`](design/LORE.md) | the canon: places, creatures, anomalies, and how lore may reach the player |
| [`design/GAMEPLAY.md`](design/GAMEPLAY.md) | what a level asks of the player (her trail, her camp, the notebook) |
| [`design/UI.md`](design/UI.md) | the controls (one binding table, what a run needs in reach) and the screens |
| [`design/STYLE.md`](design/STYLE.md) | the art bible; its bands are checked by `cargo xtask shots` |
| [`design/MEGASTRUCTURE.md`](design/MEGASTRUCTURE.md) | the Building: the vast floors above the rooms (levels 1–3), the self-assembly |
| [`design/ROOMS.md`](design/ROOMS.md) | the rooms up close: the field of view, corners, the photographed details, the ceiling's grid |
| [`design/ENTITIES.md`](design/ENTITIES.md) | the creatures: five rules, one brain, fairness, bodies, how each is beaten ([`research/`](research/README.md) for the research) |
| [`design/PROPS.md`](design/PROPS.md) | the props made real (the census of every kind, how one is remodelled), barricades that close the way, the props a hand can use |
| [`design/SECRETS.md`](design/SECRETS.md) | her marks (one hand, many kinds), the place's quiet answers, the hiding places and what is put away in them (what stays with you) |
| [`design/VENDING.md`](design/VENDING.md) | the vending machine: a real one, its rules (the coin, the silence, rocking it) |
| [`design/WINDOWS.md`](design/WINDOWS.md) | the windows: cut into real walls, the outside they show, the one behind the glass |
| [`design/VOICES.md`](design/VOICES.md) | the voices: who speaks (personas), what is said and when (your asides and the director that chooses them), and how the pipeline's models are chosen: licensed for sale, run here |
| [`design/CUTSCENES.md`](design/CUTSCENES.md) | cutscenes: the moments that borrow your eyes and give them back (the way in from the title, a broken lift's shaft, dying), the bars that say your hands are taken, the engine that stores, plays and tests them, its pipeline |

**Engineering**

| Read | For |
|---|---|
| [`engineering/ARCHITECTURE.md`](engineering/ARCHITECTURE.md) | crates, modules, one frame, level build, creatures and pacing, rendering, audio |
| [`engineering/DEVELOPMENT.md`](engineering/DEVELOPMENT.md) | conventions, determinism rules, adding a placer, tests and fixtures, headless tools |
| [`engineering/AGENTS.md`](engineering/AGENTS.md) | this tooling itself: the hooks, rules, skills and reviewers under `.claude/`, written for this game and grown with it |
| [`engineering/VOICES.md`](engineering/VOICES.md) | the voice pipeline: the toolchain, how lines are made, the evaluation of words, tone and emotion, the files the game embeds |
| [`engineering/PERF.md`](engineering/PERF.md) | the performance budgets `cargo xtask bench` holds the game to, and what each speed-up found |
| [`engineering/VISUAL_PLAYTEST.md`](engineering/VISUAL_PLAYTEST.md) | the visual playtest: a tour of the edges of the walkable floor (looked at, along, up and down), its detectors (gaps, flicker, clipping, out of place), its report and its reviewer; run by hand, and after any major change on the places it touched |

**Plan**

| Read | For |
|---|---|
| [`plan/MILESTONES.md`](plan/MILESTONES.md) | every milestone still open across the docs, in one queue, done one by one |
| `plan/<TOPIC>.md` | each design topic's work: its brief and analysis, milestones and progress, what it left open (`ENTITIES.md`, `MEGASTRUCTURE.md`, `PROPS.md`, `VOICES.md`, …, `VISUAL_PLAYTEST.md`) |
| [`plan/QUALITY.md`](plan/QUALITY.md) | grades, evidence and open issues per subsystem |
| [`plan/content.json`](plan/content.json) | the content ledger: each family and the checks it must pass |
| [`plan/TOOLING_ROADMAP.md`](plan/TOOLING_ROADMAP.md) | the quality tooling being built, and why |

**Research**

| Read | For |
|---|---|
| [`research/SOURCES.md`](research/SOURCES.md) | the research behind the game's design |
| [`research/README.md`](research/README.md) | the research notes, each answering a brief, with every source linked |
| [`research/PLAYTEST.md`](research/PLAYTEST.md) | the playtest session script: what people are asked, and what they found (records in `research/playtests/`) |
