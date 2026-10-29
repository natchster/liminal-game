# The rooms: surfaces, walls and the eye

What the rooms (levels 4 and down) look like up close, and the plan that
makes them read as a real, badly kept building rather than a game: the
surfaces photographed, the walls built as walls are, and the camera seeing
as a person does. The look is held by [`STYLE.md`](STYLE.md); the Building
above them (levels 1–3) went through the same for its surfaces in
[`MEGASTRUCTURE.md`](MEGASTRUCTURE.md) B9, whose pipeline this reuses.

## The brief (2026-10-29)

"Make backrooms textures and wall angles more realistic." Asked what was
wrong with the walls, the developer named all four of:

1. **Stretched perspective**: walls toward the sides of the screen skewed
   and too close, like a wide-angle lens (their screen is 3440×1440).
2. **Razor-sharp corners**: walls meeting each other, the floor and the
   ceiling at perfect, unshaded edges, like paper cut-outs.
3. **Walls short of the ceiling**: in some rooms the walls (3.1 m) stop
   below a higher ceiling, leaving a gap over them.
4. **Too regular a layout**: every wall on a perfect grid.

and, for the textures, **CC0 photographs**, as the Building's.

## The analysis

Read from the code and from frames at 16:10 and 21:9:

- The camera's field of view was 74° vertical at every aspect
  (`perspective_camera(74, …)`, `Settings::fov`): about 101° across 16:10
  and 121° across 21:9. A flat projection that wide stretches whatever is
  near the screen's sides.
- The Lobby's walls are a flat, smooth terracotta (`STYLE.md` asks for
  mono-yellow wallpaper); its ceiling a flat beige under a drawn grid; its
  carpet speckled, its stains flat ellipses. Every surface of the rooms is
  painted in code (`canvas_tex`).
- Walls meet each other, the floor and the ceiling with no change of light
  at the seam: the light map is per tile, and nothing darkens a corner.

## The plan

Each part ends in frames before and after and a blind review (both
orders, a fresh reviewer, a second to confirm), and keeps what the game
already holds: light from the light map and no lamp cones; levels a pure
function of `(run_seed, level)` (a visual change takes no draw from the
level's stream, or one from a salted stream of its own); no new texture
read in a global shader patch without its cost measured; every gate of
`cargo xtask check`.

| Part | What | Gate |
|---|---|---|
| R1 | **The eye.** The field of view a person's: the setting is the vertical view up to 16:9; a wider screen sees further to the sides only up to 100°; a new player walks at 64° (91° across 16:10) | blind review at 16:10 and 21:9; `tests/settings.rs` |
| R2 | **Walls to the ceiling** where a room's ceiling is higher than its walls (low partitions stay low) | frames; collision and the level sweep unchanged |
| R3 | **Corners**: darker where walls meet each other, the floor and the ceiling; worn, slightly rounded edges and trim | frames; shader cost; blind review |
| R4 | **Less regular walls**: slightly off-square and now and then angled, without moving what you walk into | the level sweep, reference levels re-recorded with a reason; blind review |
| R5 | **The Lobby photographed**: wallpaper, carpet and ceiling tiles from CC0 photographs, graded to the surfaces they replace | `cargo xtask assets textures`; `STYLE.md` bands; blind review |
| R6 | **The other zones photographed**: tiles, linoleum, wood, plaster, terrazzo | as R5, zone by zone |

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

## Progress

| Part | Commit | What was done |
|---|---|---|
| R1 | (this commit) | The eye. `session::settings::vertical_fov`: the setting is the vertical view on a screen up to 16:9 (a narrower one sees less to the sides); past 16:9 the view to the sides grows only to `H_FOV_MAX`, 100° (or what the setting gives at 16:9, if wider), and less is seen above and below; a new player walks at 64° (`FOV_DEFAULT`; 91° across 16:10, 97° across 16:9, 100° across 21:9, where 74° had been 101°, 106° and 121°), the setting 50–90°. A store from before that never moved it (74, no `fovRev`) takes the new default; a chosen one stays. The culling cone follows the camera (`lighting.rs`, from its fov and aspect). Blind reviews (both orders, two fresh reviewers): the new view preferred in all nine frames by both, eight zone frames at 16:10 and the Lobby at 21:9 (the old one there: "a huge smeared left wall, a tiny partition, a ceiling grid fanning out"). Tests: `tests/settings.rs` `a_wide_screen_does_not_stretch_the_walls` (the view at 4:3 to 16:9 is the setting; 100° across at 21:9 and 32:9; a wider setting keeps its 16:9 view; the camera follows it; the old default migrates). Re-recorded: all 48 goldens and their look anchors (every frame's composition tighter); the expressive range holds. Seen on the way, for later parts: the Lobby's partitions are terracotta, not `STYLE.md`'s mono-yellow; a thin dark strip where a poolroom wall ends; a hard seam where a mustard carpet meets a grey floor in an office hall |
| R2 | (this commit) | Walls to the ceiling: found already so. Every full wall box runs from the floor to its own tile's ceiling (`build_solids_view`: `top = ceil[t]`), and where two tiles' ceilings differ a bulkhead closes the step (`world/arch.rs`); only the low partitions stop short, by design (the office's cubicles at 1.35–1.6 m, the Showroom's at 2.35–2.5 m under its slab). What read as a gap was the bare join: nothing darkened it, and the wallpaper's painted shadow at its top shows only on walls over about 2.87 m (most of the Lobby's are 2.7–3.05). That is R3's |
| R3 | (this commit) | Corners. The light map is RGBA now at 8 texels a tile (0.5 m; `visuals::occlusion`): R the light as before (sampled between the tiles as the linear filter sampled the old map), G how open each place is (the distance to the nearest wall that runs up to its ceiling, to 0.8 m; low partitions left out), B its ceiling (to 10 m). The shader reads G and B from the sample it already took 0.8 m out from each surface for its indirect light, so no texture read is added: a wall face beside another wall, a floor or a ceiling beside a wall, finds a wall nearer than 0.8 m and keeps 45–100% of its indirect light; a wall within 0.6 m of its ceiling keeps 60–100%. Plain walls (their own face 0.8 m off) and outside corners are untouched. Off where the map does not hold the place's shape (`corner_occlusion`: the Building's stacked floors, the Stairwell, the Loop). G and B are made once a level (`shape_key`) and kept while panels switch. Blind reviews (two fresh reviewers, both orders): the corners preferred in 5 and 7 of 9 frames, the rest ties (the dim pool hall, the dark office), the old never. Found on the way: the rooms' surfaces (drywall, concrete, brick, tile; `visuals::surfaces`) had painted from the per-frame stream the first time a level needed them, so a run's frames hung on which level that was (a hole E10's fix had left: those painters called the native generator directly); they paint from their painters' own streams now (`tests/story.rs` `a_levels_build_takes_nothing_from_the_frames_whatever_came_before`, which fails without it). Their stains are laid out anew once: the concrete floors and slab ceilings came out more mottled (two reviewers saw it; R6 photographs them). Re-recorded: all 48 goldens (the corners; the surfaces' new layouts) |
| R4 | — | Less regular walls: not done; what was tried and what it would take. Walls that are not quite true (plaster bowing a few millimetres over a metre, as a bump in the shader, so nothing anyone walks into or hangs on a wall moves) changed no frame by more than 2 levels in 255, even at 2 cm: the rooms are lit mostly by their ambient and hemisphere light, which a small tilt of a surface barely changes; dropped. Walls truly off-square or at an angle cannot be drawn alone: collision, the line of sight (sound and the creatures' eyes), the light map's shape and the sixteen placers that hang things on walls (`wall_adj_spot` and kin) all read walls as boxes on the grid, so a wall drawn 2° off would leave a poster floating 14 cm off it or a hand through it. As a milestone of its own: angled boxes in the solids (from archgen, on a salted stream so no level's draws move), the circle, segment and ray queries over them, the placers asked for a wall's own face, the light map's G from them, and the archgen, world and reference fixtures re-recorded, the level sweep holding |
| R5 | (this commit) | The Lobby photographed, as details over its painted surfaces (which keep their palettes, grime, motifs and moss). The engine's `Patch::ArchUV` takes an `ArchDetail`: a grey height map laid flat on each face in world space (from its world position and normal), one sample a pixel shading the colour and lit as relief, the relief fading as its texels shrink under a pixel, bound in the bump slot so only the materials that have one compile the branch. Made by `assets/src/textures/build.py` (`detail`, `acoustic`) from CC0 photographs, each checked by its SHA-1 (`sources.json`): woodchip paper (ambientCG `Wallpaper001A`; its height and its colour's shading without what changes over 2 cm), 1.2 m a repeat (a coarse paper: at a fine one's 3 mm the chips were lost past an arm's length, even at 3440×1440); carpet (ambientCG `Carpet016`, 1.7 m as photographed); an acoustic tile's face (three tiles of a Soviet dropped ceiling of the 1980s, Wikimedia Commons, Georg Pik, quilted seamless), a tile a repeat. The carpet and the tile ceilings are world-space materials now (one repeat a tile, as their planes' UVs had had it). The ceiling at a real tile's size: 6 × 6 tiles a 4 m tile (0.67 m; 2 ft is 0.61) on a 3 px T-bar grid, half a tile off along z so the fittings, hung at the tiles' centres, fill 2 × 1 tiles exactly (`panel_geo` 1.33 × 0.67 m, a 2 × 4 ft troffer, where it was 1.9 × 1.25); a missing tile is one tile, never where a fitting hangs; a fallen tile is one of them, grimy (near black, seen from below against a lit fitting it had been taken for a hole by four reviewers). The peeling and mouldy papers were tried and dropped (their marks repeated every half metre). Blind reviews (two fresh reviewers, both orders): preferred in 7 of 9 frames by both, ties in the two with no tile ceiling, the old never ("huge panels … fittings that look oversized" against "a believable drop ceiling: about 600 mm tiles"). Re-recorded: all 48 goldens; the look anchors of the three frames that look up at a fitting (smaller, so less bright and contrasted). Bench after R1–R5: within budget, no slowdown in the history; a level's build 2–7 ms longer (the shape channels), GPU p50 about 0.1 ms more. Seen for R6: the Showroom's slab and the pool's plaster ceilings are mottled noise with no material; the floors' stains are perfect ellipses |
| R6 | (this commit) | The other zones photographed, as R5's details over their painted surfaces (`photo::detail_for`): plaster laid by hand (Poly Haven `plastered_wall_04`, Rob Tuytel; 3.2 m a repeat) on the drywall, painted walls and plaster ceilings; rough concrete (`rough_concrete`, Dimitrios Savva; 1.23 m) on the concrete walls and slab ceilings; a worn concrete floor (`concrete_floor_worn_001`, Dimitrios Savva and Rico Cilliers; 3 m); the carpet on the office's carpet tiles. Brick and glazed tile keep their painted pattern alone. `build.py` `detail` evens the contrast (`even`) and lays each again from its own patches (`quilt`): as photographed, a streak in the plaster, a rough patch in the concrete and a cluster of pits in the floor repeated on a grid; a concrete wall with its own form-tie grid was tried and dropped (it fought the painted seams). The painted floors and ceilings are world-space materials now (`arch::plane_material`), laid from the tiles' edges as their planes' UVs had laid them (from the centres, the concrete floor's saw-cut ran down the middle of every corridor, the camera's tile included). Blind reviews (two fresh reviewers, both orders; frames at 1280×800, the nine views it changed most): preferred in 8 of 9 by both, a tie, the old never ("visible weathering on the concrete and drywall" against "smoother and more even … closer to a flat game surface"; "no obvious repeat or tiling pattern"). Re-recorded: the goldens that show them |

## What only people can tell

Whether the narrower view feels like being there or like tunnel vision;
whether the photographed rooms still feel like the Backrooms (the uncanny
sameness) or merely like an old office.
