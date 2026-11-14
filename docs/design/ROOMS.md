# The rooms: surfaces, walls and the eye

What the rooms (levels 4 and down) look like up close, read as a real,
badly kept building rather than a game: the
surfaces photographed, the walls built as walls are, and the camera seeing
as a person does. The look is held by [`STYLE.md`](STYLE.md); the Building
above them (levels 1–3) went through the same for its surfaces in
[`MEGASTRUCTURE.md`](MEGASTRUCTURE.md) B9, whose pipeline this reuses.

## The design

- **The eye is a person's.** The field-of-view setting is the vertical
  view on a screen up to 16:9; a wider screen sees further to the sides
  only, up to 100°, and less above and below. A new player walks at 64°
  (91° across 16:10), where the old 74° had been a fisheye at 21:9.
- **Walls run to their own ceiling**, and a bulkhead closes a step between
  two ceilings; only low partitions stop short, by design (the office's
  cubicles at 1.35–1.6 m, the Showroom's under its slab).
- **Corners are darker** where walls meet each other, the floor and the
  ceiling, as a room's corners are: read from how open each place is and
  how high its ceiling (the light map's G and B, `visuals::occlusion`),
  from the sample the indirect light already takes, so no texture read is
  added. Plain walls and outside corners are untouched.
- **Walls are not all square.** A thin partition that ends free may lean
  (about 45% of those that can, 2–4.5°; a few 6–10°), inside its own wall
  tiles, so the paths and what the placers chose stay where they were;
  what you walk into is a staircase of boxes inside the drawn wall
  (`world::angled`).
- **The surfaces are photographed, over their paint.** The rooms keep
  their painted surfaces (palettes, grime, motifs, moss), and a grey
  height map from a CC0 photograph is laid flat on each face in world
  space over them (`Patch::ArchUV` with an `ArchDetail`): woodchip paper
  (a repeat in 1.2 m), carpet (1.7 m), acoustic tiles, plaster laid by
  hand (3.2 m), rough concrete (1.23 m), a worn concrete floor (3 m); each
  evened and quilted from its own patches, so no streak or pit repeats on
  a grid. Brick and glazed tile keep their painted pattern alone.
- **Each change is kept only where a blind review prefers it** (both
  orders, a fresh reviewer), with light from the light map and no lamp
  cones, and no texture read in a global shader patch without its cost
  measured.

## The work

How this was planned and done (the brief, the analysis, the plan, progress): [`../plan/ROOMS.md`](../plan/ROOMS.md).

## Sources

CC0 photographs found for the rooms (to be listed in
`assets/src/textures/sources.json` as they are used):

- ambientCG (CC0): woodchip wallpapers `Wallpaper001A`–`C`, `002A`–`C`
  (plain, peeling, mouldy), carpet `Carpet016` (beige, photo scan).
- Poly Haven (CC0): `decrepit_wallpaper`, `beige_wall_001`/`002`,
  `dirty_carpet`, `long_white_tiles`, `interior_tiles`, `worn_tile_floor`,
  `dirty_tiles`, `rounded_square_tiled_wall`, `old_linoleum_flooring_01`,
  `linoleum_brown`, `old_wooden_floor_01`, `laminate_floor_02`,
  `terrazzo_tiles`, `painted_plaster_wall`, `asbestos_sheet`.
- Wikimedia Commons (CC0): *Dropped ceiling (USSR, 1980s) (01)*, mineral
  fibre tiles in a grid, for the ceiling tiles' face.
- Not used: ambientCG's `OfficeCeiling001`–`006` (procedural, the flat CG
  look this replaces).

## What only people can tell

Whether the leaning walls make the rooms read less like a grid as you walk
through them (no reviewer could tell from still frames);
whether the narrower view feels like being there or like tunnel vision;
whether the photographed rooms still feel like the Backrooms (the uncanny
sameness) or merely like an old office.
