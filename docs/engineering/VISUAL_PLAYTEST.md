# Visual playtest: looking where nobody looks

A playtest of what the game looks like where a player can go and nobody
has framed it: standing as close as a body can to every edge of the
walkable floor (a wall, a parapet, the void, a step), looking at it, along
it, up and down, standing and crouched. It is run by hand on an area, a
level or a set of levels (never by `cargo xtask check`), and after any
major change on every level or place that change touched (`CLAUDE.md`,
definition of done). Its findings are fixed and the place toured again.

## The work

How this was planned and done (the analysis, milestones, progress): [`../plan/VISUAL_PLAYTEST.md`](../plan/VISUAL_PLAYTEST.md).

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

[`../research/visual-qa.md`](../research/visual-qa.md) (kept whole; summed up in
[`SOURCES.md`](../research/SOURCES.md) section 18). Studios shoot many fixed cameras
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
   look: a solid other than the void's, taken 3 cm inside its faces and,
   further off, half a pixel's span more, as a far silhouette's edge pixel
   is part background (`sight::SPREAD`: 9 cm at 40 m, so a hole in anything
   thicker than 17 cm is found to the reach), and in the rooms a wall tile,
   the floor and the ceiling), *flicker* (pixels
   both turned renders change by more than 30% of their brightness, and 8
   levels at least: a hard edge crosses a sample point one way only, two
   surfaces fighting for one depth flip either way, and in this game's
   dark a fight is a few levels; counted only in a patch 5 px thick, as a
   fight flips a patch and a thin far rail a line) with *coplanar* (two
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
   run, read, inspect, record (`docs/research/playtests/`), reproduce each finding
   with `--shots-at`, fix, tour the place again; and when to run it.
