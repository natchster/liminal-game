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

## What the research says

[`research/visual-qa.md`](research/visual-qa.md) (kept whole; summed up in
[`SOURCES.md`](SOURCES.md) section 18). Studios shoot many fixed cameras
per level with the noisy features off and compare at strict thresholds;
nobody documents bots taking pictures for art review, so a tour placed by
rule is ours. Z-fighting is found by counting coplanar fragments per pixel
or by testing faces for coplanarity; a jittered render diffed is our own,
so it is paired with the geometric test. A see-through wall is the wall
crossing the near face of the frustum: test the face's corners, not the eye
alone. Finding unlit or glowing things in a lit scene automatically has no
source: read the materials. Vision models find under half the glitches in a
frame and hallucinate clipping at near contact: the inspector is a filter,
never the gate. What the design took:

- the renders deterministic: the world frozen, the grain and the lamps'
  flicker held between a view's renders, and the bare ones without the HUD;
- *clip* tests the near face's four corners and the eye, not the eye alone;
- *flicker* is two detectors: the jitter diff (pixels) and *coplanar*
  (two faces in reach, of one thing or two, parallel, within 2 mm, overlapping);
- a thing is flagged when several views agree, not on one;
- every finding is reproduced with `--shots-at` and looked at before it is
  fixed; the report shows each flagged view as a strip (the look, the gaps,
  the flicker) for a fast read.

## The design

1. **A tour** (`game::debug::tour`): on each level, every walkable place
   at an edge (a tile beside a wall, a void, a step or the grid's end), the
   point nearest that edge a body can stand on, and from it views: facing
   the edge, along it both ways (grazing), up at its top and down at its
   foot; standing and crouched; corners too. Deterministic (no draw from
   any stream), sampled across the level to a budget, filtered to an area
   when asked.
2. **Five renders a view** (`backrooms --tour`): the look as a player sees
   it (the HUD's grain and vignette on, the lamp lit where it is dark); and,
   bare (no HUD layer, so no grain to change between them), the scene
   cleared to magenta, cleared to green, and cleared to magenta again with
   the camera turned by 0.02° (about a tenth of a pixel) one way and then
   the other. The colour filter runs over the whole frame, the clear colour
   too, so the background is found where the two clears differ, not by
   matching a colour: a magenta surface cannot pass for a hole.
3. **Detectors**, each a number and a picture: *gaps* (the background
   where a ray from the eye meets, within 40 m, what of the level stops a
   look: a solid other than the void's, taken 3 cm inside its faces, and
   in the rooms a wall tile, the floor and the ceiling), *flicker* (pixels
   both turned renders change: a hard edge crosses a sample point one way
   only, two surfaces fighting for one depth flip either way) with *coplanar* (two
   faces in reach, of one thing or two, that lie on one plane and overlap),
   *clip* (the near face's corners or the eye inside a thing; the
   architecture's merged meshes and the solids are not asked), *plain* (in view and near: a flat-coloured material, or one
   unlit or glowing where the place is dark; lit by itself where the place
   is lit it is `selflit`, a lamp or a window as meant, listed only), each
   with the object's name where there is one; a plain thing flagged when
   two stands agree, the others at once.
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
| VP2 | **Renders and the pixel detectors**: `backrooms --tour`, five renders a view; *gaps* (the background where a ray from the eye meets the level's own walls, floors, masses or ceiling within reach: a hole, not the open void) and *flicker* (changed by a turn either way) | known answers: a hole cut into a wall and a fighting pair put into a level are each found; a clean view, and the void seen over a parapet, are not |
| VP3 | **The geometric detectors**: *clip* (the near face's corners or the eye in a solid or a thing; the body standing inside a thing), *coplanar* (two faces, of one thing or two, on one plane, overlapping), *plain* (a flat-coloured, unlit or glowing material in view and near), a thing flagged when several views agree | known answers: a camera inside a thing, a coplanar pair and a flat-coloured thing are each found; a clean level's counts recorded |
| VP4 | **The report**: `cargo xtask playtest visual`, ranked findings, a strip a flagged view (the look, the gaps, the flicker), contact sheets, `findings.tsv` | a run on a floor of the rooms and one of the Building, looked at |
| VP5 | **The skill and the reviewer**: `.claude/skills/visual-playtest`, `.claude/agents/visual-inspector.md`, `CLAUDE.md` (the commands; after a major change, the places it touched), `AGENTS.md`, `DEVELOPMENT.md` | the structure test; the skill followed by a fresh agent on one area |
| review A | Code and design review of VP0–VP5 | committed with what it found |
| VP6 | **The Building toured**: levels 1–3 on several runs, the inspector's review, `docs/playtests/` | the report written |
| VP7+ | **Its findings fixed**, a milestone for each kind, each place toured again | the finding gone on a new tour; fixtures and goldens only through `regen-fixtures` |
| review B | Code and design review of VP6 on | committed with what it found |

## Progress

| # | Commit | Notes |
|---|---|---|
| VP0 | 7c98223 | the plan; the research (`research/visual-qa.md`, `SOURCES.md` 18) folded in with VP1 |
| VP1 | 95b92a6 | `game::debug::tour`: `Game::standable` (on the grid, off the wall tiles, on walked ground, the body's 0.20 m circle clear of every solid), `tour_stands` (each tile walked along both axes from its middle and 1.2 m either side, in 2 cm steps, until a body is stopped; the edge named by what is beyond: `wall`, `mass`, `void`, `step`, `grid`, or `solid` for a thing inside the tile; a corner where it is stopped again within 0.6 m sideways), `tour_views` (stands chosen farthest-first to a budget; six looks from an edge, three from a corner). Seed 4242: the Building's level 1 has 1190 stands (865 at a mass, 249 over the void, 76 at a solid), level 5 of the rooms 3104 (all walls: no prop there stops a body). Tests: `crates/backrooms/tests/visual_tour.rs` |
| VP2 | 04bc52b | `backrooms --tour DIR` (`crates/backrooms/src/tour.rs`; `--tour-budget`, `--tour-area`, `--tour-plant hole\|fight`; `TOUR_DEBUG=VIEW` traces the gap rays): five renders a view, `DIR/L<n>/views.tsv` (each view's `--shots-at` camera, its background, gaps and flicker) and the looks, the flagged views' masks painted (`_gaps.png`, `_flicker.png`); `game::debug::sight` (the background from two clears, the gap rays, the flicker of two turns, blobs; `View` now carries its edge). What it took to make it true, each seen in the frames: the solids' void box stops a body and no look (asking it, every look up in the Well was a hole), a solid stands a few cm proud of what is drawn of it (a ray at the sky's rim grazed it: taken 3 cm inside), the gap rays every second pixel (a crack a pixel wide), and one turn alone flagged every hard bright edge (a lamp's outline, a door's lit gap: 47 views on the three floors; two turns, 7). Seed 4242, 120 views a floor (29 s for four): the Building's floors no gaps, flicker in 1, 6 and 0 views (small: a plaque, a ledge; to be judged on the tour of VP6); level 5 of the rooms a hairline crack along a ceiling seam seen from three views, a hole over a wall where two ceilings meet (5,482 px) and the background through a far window's edge: the first real findings. Known answers (`tests/visual_tour.rs`): the hole planted found (78,720 px, the clean view 0), the fight found looking up past it (1,669 px; 0 clean), the Works' eight views into the void with the background in them free of gaps |
| VP3 | 5f6d5a7 | `game::debug::things`: every mesh of the world a part (an instanced mesh's instances each one; a placed thing named by its placer or core kind, the architecture `level`, each described by its nearest named ancestor, its geometry, its size in the world and its colour: `box 4.00x0.13x0.26 #ffffff map`), judged by its own box when no side is over 6 m. *clip*: the eye or a corner of the near plane's face (padded half again) inside a part, measured in metres; *body*: the body's column, knee to eye, inside a part taller than 0.3 m; *plain*: within 5 m and in view, a lit material with no picture, no vertex colour and no wear (`plain`), an unlit one (`unlit`), a glowing one (`glow`), flagged when two views agree; *coplanar*: faces within 6 m of the stands, parallel, within 2 mm and overlapping 4 cm² or more (sweep and prune in plane buckets), one finding a pair of things, `coplanar_alike` (one material on both: drawn alike where they overlap; listed, not flagged: no flicker was seen near any) apart from `coplanar`. `--tour` writes `DIR/L<n>/findings.tsv` (kind, thing, mesh, how many views, the first, where, how much, flagged, the flagged first) and `views.tsv` gains clip, body and plain; `--tour-plant inside\|flat`. Seed 4242, 120 views a floor: the Stair Halls (L2) a coping 13 cm by 26 cm the crouched eye stands 4–6 cm inside, six times; the Well (L1) its doors and jambs flat `#47504a` with no picture (33 plain), door jambs flush with a facade pier's face (12 coplanar); the Works (L3) ochre railings and posts plain (47); the rooms (L5) two walls doubled on one plane and a locker's door on its body. Known answers: a box round the eye a clip and a body, a flat box beside the stand plain, the fighting squares coplanar, none of them at the first stand as built (made to fail once: the overlap threshold out of reach); unit tests: two triangles' shared area, the near face before the eye, a frame's share of a box, two views to flag a plain thing |
| VP4 | 5480112 | `cargo xtask playtest visual [--seeds 4242,7] [--levels 1,2,3] [--budget 180] [--area x0,z0,x1,z1] [--size 640x400]` (`xtask/src/playtest.rs`): builds the game, runs `--tour` a seed into `target/xtask/playtest/visual/s<seed>/`, and writes `report.md` (a row a floor; each floor's gaps and flicker views, the largest first, each with its painted mask and its `--shots-at` camera; its flagged things by kind, each named, with how many views saw it and the camera of the first; how many were listed and not flagged) and `findings.tsv` (every finding of every floor with its seed and level). `--tour` writes the sheets (`crates/backrooms/src/sheet.rs`, each tile named under it with the HUD's canvas): `sheet_gaps.png` and `sheet_flicker.png` (the look beside its mask), `sheet_things.png` (the first view of each flagged thing, named with what it is) and `sheet_all_NN.png` (every view, 48 a page: what no detector measures). Looked at on seed 4242's Well (L1) and level 5 of the rooms, 120 views each, 18 s: the cell doors flat `#47504a` among the photographed concrete; a corner of the Well where the concrete photograph is stretched and smeared (`L1_002c`, seen only on the all-views sheet); in the rooms the hole over a wall and the ceiling's seam both plain in the player's own look (a pale panel, a bright hairline). Test: the report ranks the flagged and keeps every finding |
| VP5 | d63f3d1 | `.claude/skills/visual-playtest` (when, the scope, the run, the report, the sheets, the inspector by brief, reproducing, the record, the fix a kind at a time, the tour again, what it cannot see, the gates), `.claude/agents/visual-inspector.md`, `CLAUDE.md`'s definition of done (after a major change, the playtest on every place it touched), `AGENTS.md`, `DEVELOPMENT.md`; the records in `docs/playtests/visual/` (its README the record's layout; the people's sessions stay a folder up). **The skill followed end to end by a fresh agent** on an area of the Stair Halls (seed 4242, 60 views, the inspector briefed as the skill says: 21 min). It confirmed the crouched eye inside the balustrade's rail (the rail vanishes: you look down the shaft) and found the flicker there all on far lanterns' glazing bars (false), and listed what the skill and the tools could not do; each is fixed: the lamps' pooled light was a second's fade behind a camera put somewhere, so a reproduction did not match the tour (`Game::snap_pooled_lights` in the tour and `--shots-at`: they now match by eye; the goldens' `--views` path untouched); a flicker cannot be seen in one still (`--tour-at`: every detector on an exact camera, the bare and both turned renders written to flip between); a clip cannot be seen from inside the thing (the report gives a camera stepped back); a budget left stands out silently (`meta.tsv`, "60 of 84 views"); the run deleted the one before (kept as `visual.prev/`); the report's columns, colours and sizes unexplained (a legend); the sheets paged at twelve; the skill's steps rewritten (where a picture lands, the game's flags beside the task's, the inspector's brief: read-only, the root and the folder, the time it takes, wait for it; the clip's fix on the Building is the solids' boxes; a detector's false flag is tuned against a known answer; the stands move after a change to the solids) |
| review A1 | (this commit) | The code and design review of VP0–VP5, by two fresh agents (the code reviewer by brief; a design reviewer given the request and a run of the Building's three floors). About 25 findings, fixed in four commits; this one the safety and the small ones. **`--out` could delete the working folder**: `out.with_extension("prev")` leaves `.` as it is, so `--out .` removed the folder's contents (`.git` with them) and `--out docs/…` lost the records on the next run; now only a folder the task made (its `.visual-playtest` mark) is ever moved aside or removed, any other existing folder that is not empty refused (`only_a_folder_it_made_is_ever_moved_or_removed`). A corner stood up to 0.6 m outside an area (tested on the area that showed it). A malformed `--tour-area`, `--tour-at` or `--tour-budget` toured a whole level instead: now it stops. The report reads the flag threshold from the tour (`meta.tsv`), not a copy. A plain thing is flagged when two *stands* agree (a stand's looks share an eye; findings.tsv gains `stands`). Unlit and glowing things are flagged only where the place is dark: lit by themselves where it is lit (the rooms' ceiling panels, a lamp, a lit window) they are `selflit`, listed only. Docs made true: clip does not ask the solids or the merged meshes, coplanar pairs faces of one thing too, `debug` holds the playtest. The rule where agents act: `add-prop`, `add-zone`, `add-anomaly` and the world and visuals rules end in the visual playtest of what they touched. The skill: a run its own `--out`, an inspector a floor in parallel, the eye budget, a plain fix accepted only by a blind look (any wear clears the flag), a coplanar fix only for a fight seen |
