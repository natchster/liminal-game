# Visual playtest: looking where nobody looks

A playtest of what the game looks like where a player can go and nobody
has framed it: standing as close as a body can to every edge of the
walkable floor (a wall, a parapet, the void, a step), looking at it, along
it, up and down, standing and crouched. It is run by hand on an area, a
level or a set of levels (never by `cargo xtask check`), and after any
major change on every level or place that change touched (`CLAUDE.md`,
definition of done). Its findings are fixed and the place toured again.

## The analysis (2026-11-11)

What the gates see today, and what they miss:

| Gate | Sees | Misses |
|---|---|---|
| `cargo xtask shots` | 56 fixed frames, exact | everything outside them |
| `--views` (zone views) | one view from inside each zone | the edges, where geometry meets |
| `--prop-census` | every kind of prop, framed | the prop where it stands among others, from where you stand |
| `tests/materials.rs` | albedo and saturation of materials | whether a thing looks in place next to its neighbours |
| `art-critic` (blind A/B) | which of two frames is better | a fault in a frame nobody rendered |
| `PLAYTEST.md` | people, in a session | a few minutes of a few floors |

The player's body is a circle of 0.20 m (`circle_hits_wall` in
`update_player`) and the camera's near plane is 0.05 m: walls cannot cut
the camera, but most props have no collision (only vending machines,
barricades and shut cell doors stop you), so a body can walk into a table
and see through its sides. The Building's floors are photographs
(`visuals::photo`) where the rooms are painted canvases: a flat-coloured
prop stands out there.

## What it looks for

1. **Out of place**: a flat-coloured object (no texture, no vertex
   colour) on photographed surfaces; lighting that does not match (an
   unlit or emissive thing in the dark, a thing far brighter or darker than
   what stands beside it); texture quality (blurred, stretched, tiled
   visibly, a seam in a photograph).
2. **Clipping and flicker**: two surfaces fighting for the same depth
   (z-fighting: patches that flip when the camera moves a hair), the
   camera inside an object or close enough to see through its border, a
   thing passing through a wall or another thing.
3. **Gaps**: a hole to nothing (the sky or the void where a wall or floor
   should be), a crack at a seam, a view into a part that should stay
   hidden (the inside of a wall, the back of a mass, an unbuilt side).

## The design

1. **A tour** (`game::debug::tour`): on each level, every walkable place
   at an edge (a tile beside a wall, a void, a step or the grid's end), the
   point nearest that edge a body can stand on, and from it views: facing
   the edge, along it both ways (grazing), up at its top and down at its
   foot; standing and crouched; corners too. Deterministic (no draw from
   any stream), sampled across the level to a budget, filtered to an area
   when asked.
2. **Three renders a view** (`backrooms --tour`): the look as a player sees
   it; the same with the sky and background cleared to a sentinel magenta,
   which no surface has; and the same again with the camera moved by a
   hair (2 mm and 0.02°).
3. **Detectors**, each a number and a picture: *gaps* (the sentinel where
   geometry should be: small or enclosed magenta, or any at all where the
   level is enclosed), *flicker* (pixels that change between the look and
   the jittered render away from edges: z-fighting), *clip* (the camera
   within reach of an object's bounds: inside a thing), *plain* (in view
   and near: a flat-coloured, an unlit or a glowing material), each with
   the object's name where there is one.
4. **The report** (`cargo xtask playtest visual`): per level, the views,
   each detector's flagged views ranked, contact sheets per detector and of
   a sample of all views, and `findings.tsv`; under
   `target/xtask/playtest/visual/`.
5. **Eyes**: a fresh reviewer (`visual-inspector`, by brief) looks at the
   sheets, the flagged first, and names what is wrong in each of the three
   kinds, with the view to reproduce it; detectors find, people (and the
   inspector) judge.
6. **The skill** (`visual-playtest`): an area, a level or a set of levels;
   run, read, inspect, record (`docs/playtests/`), reproduce each finding
   with `--shots-at`, fix, tour the place again; and when to run it.

## Milestones

Each is tested (`cargo xtask check --fast`), committed and pushed on its
own, with the production build made; after the fifth and at the end, a
code and design review whose findings are fixed.

| # | Milestone | Done when |
|---|---|---|
| VP0 | This plan; the research | committed |
| VP1 | **The tour**: edge stands and their views, corners, crouched, sampled to a budget, an area | tests: every stand is one a body can stand on, near its edge, the same twice, spread over the level, inside an area when given |
| VP2 | **Renders and detectors**: `backrooms --tour`, three renders a view, the gaps, flicker, clip and plain detectors | known answers: a hole, a fighting pair, a camera inside a thing and a flat-coloured thing put into a level are each found; a clean view is not |
| VP3 | **The report**: `cargo xtask playtest visual`, ranked findings, contact sheets, `findings.tsv` | a run on a floor of the rooms and one of the Building, looked at |
| VP4 | **The skill and the reviewer**: `.claude/skills/visual-playtest`, `.claude/agents/visual-inspector.md`, `CLAUDE.md` (the commands; after a major change, the places it touched), `AGENTS.md`, `DEVELOPMENT.md` | the structure test; the skill followed by a fresh agent on one area |
| review A | Code and design review of VP0–VP4 | committed with what it found |
| VP5 | **The Building toured**: levels 1–3 on several runs, the inspector's review, `docs/playtests/` | the report written |
| VP6+ | **Its findings fixed**, a milestone for each kind, each place toured again | the finding gone on a new tour; fixtures and goldens only through `regen-fixtures` |
| review B | Code and design review of VP5 on | committed with what it found |

## Progress

| # | Commit | Notes |
|---|---|---|
| VP0 | (this commit) | the plan |
