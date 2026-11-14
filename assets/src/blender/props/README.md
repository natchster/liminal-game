# Prop parts: the modelling contract

Every prop in the game is built by a *placer* (a `place_xxx(g, dist)` in
its theme's crate, `crates/placers/placers_<theme>/src/xxx.rs`, about 650
of them, or a core-props builder in `crates/game/src/core_props/`) from the
engine's primitives (`box_geo!`, `cyl_geo!`, `sphere_geo!`, …). This
directory makes those props finer in Blender, a folder per theme
(`domestic/`, `office/`, …, and `game/` for what the game places itself),
a file per *family*: the props modelled together, sharing their kit of
shapes, named for what they are (`domestic/kitchen_hall_and_floor_things.py`).
A family goes in the folder of the theme most of its parts serve; the index
at the end says which others it serves. There are two kinds of family:

- **Fitted parts** (the families made first, and the Showroom): each primitive that matters gets a
  real modelled part, swapped in where the placer wraps it,
  `pg("Name__role", box_geo!(…))` (or `pg_own`, a private copy the placer
  may change afterwards; `crates/game/src/placers/kit.rs`), and **fitted to
  the primitive's bounds** (`assets::props::prop_part`), so the placer's
  layout, sizes, randomness, colours, animation and gameplay stay exactly as
  they are. Only the shape gets better. This file is their contract.
- **Real things** (the remodelled families, the vending machine, the
  windows, the barricades): the prop remodelled as the real object at its
  real size, each part registered by the family's own `auto(name, build,
  budget)` and used as modelled. Their brief is `docs/design/PROPS.md`,
  "How a prop is remodelled".

The pilot family `core/radiators_and_overturned_chairs.py` (radiator,
overturned chair) is the worked example of fitted parts. Read it first.

## Files per family

| File | What |
|---|---|
| `<theme>/<family>.py` | The parts: `part(name, frame, build)` calls (Blender Python, `from propkit import *`); a family's file name is its batch name in the build (`--batch`) and must be unique |
| `docs/research/props-remodelling/<family>.md` | Its notes: per placer, what was rebuilt, what was left and why, sources and sizes, placement problems seen (a record of the work) |

## A part

```python
part('Radiators__body', f_box(0.70, 0.50, 0.10), radiator_body)   # build is a function returning a Shape
```

- **Name:** `<PlacerNameWithoutPlace>__<role>` in CamelCase, e.g.
  `PocketWatches__case` for `place_pocket_watches`. Use letters, digits and
  `_` only. A part may be shared by several primitives (same object, same
  frame kind), even across placers in the batch.
- **Frame:** the primitive it replaces, with that primitive's *canonical*
  arguments as written in the placer. If an argument is an expression, use a
  typical value. The part is modelled **in the primitive's frame**:

| primitive | frame | where the part must sit |
|---|---|---|
| `box_geo!(w, h, d)` | `f_box(w, h, d)` | centred on the origin |
| `cyl_geo!(rt, rb, h)` | `f_cyl(rt, rb, h)` | axis +Y, centred, y from −h/2 to h/2, top radius rt |
| `cone_geo!(r, h)` | `f_cone(r, h)` | axis +Y, centred, apex at +h/2 |
| `sphere_geo!(r)` | `f_sphere(r)` | centred |
| `torus_geo!(R, r)` | `f_torus(R, r)` | ring in the **XY plane** around the Z axis |
| `capsule_geo!(r, len)` | `f_capsule(r, len)` | axis +Y, centred, height len + 2r |
| partial sphere/cylinder, `lathe_geo!`, `shape_geo!`, `octa_geo!`… | `f_bounds(lo, hi)` | the primitive's own bounds |
| `plane_geo!(w, h)` / `circle_geo!(r)` | `f_bounds((-w/2,-h/2,0),(w/2,h/2,0), 'plane')` | front face at z = 0 facing +Z; any thickness goes **behind** (−Z, into the wall) |

- Coordinates are the game's: **+Y up**, metres. Blender does not care; the
  kit builds straight in these coordinates.
- **The part must fill its frame.** The game maps the frame's bounds onto the
  primitive's bounds, so a part that is smaller than its frame shrinks the prop,
  and one that is rotated or off-centre ends up wrong. The build checks the fit
  (error above 35 %, warning above 10 %). A thin axis (paper, plates) may bow or
  crumple by up to about a quarter of the part's size.
- **One part is one material.** Whatever the placer colours differently is
  already a separate mesh. Don't model it again inside your part.
- **Faithful, then finer.** Read the placer's comments and code: know what the
  thing *is* and what each primitive stands for (a cylinder may be a mug, a
  table leg or a pipe). Then model that real object at arm's-length quality:
  - chamfered or rounded hard edges (`rbox`, `bevel`)
  - rims, lips and hollows (`vessel`, `lathe`)
  - seams, grooves and panel lines cut as geometry (`cut`)
  - screw heads, hinges and handles (`screw_head`, `tube`)
  - turned profiles (`lathe`)
  - soft, lumpy forms for soft things (`subdivide`, `noisy`, `bend`)
  - paper that curls and cloth that folds (`slab`, `grid_plane` + `bend`/`noisy`)

  Keep the silhouette the placer intended.
- **Budget:** `budget_for(frame)` (from ~100 triangles for a key to ~1800 for
  big furniture). Over budget is a warning; over 125 % is an error. Spend
  triangles on silhouette and edges people see, not on hidden faces or tiny
  details. The whole game ships these meshes, so cheap is good. The asset
  lint reads a `budget=` given to `part()` from the call's text: write it as
  a literal number.
- **Shading:** `part(..., smooth=35)` is the crease angle. Hard surfaces keep
  the default (weighted normals make flat faces flat and bevels soft). For
  organic or soft things use `smooth=180`; use `flat=True` for a faceted look.

## What to replace, what to leave

- Replace every solid primitive a player would look at: box, cylinder,
  sphere, torus, cone, capsule, and the octahedron and lathe ones where a
  better shape exists.
- **Leave** decals and flat textured quads (stains, posters, labels, clock
  faces, screens, shadows, glows) and anything with `transparent: true`,
  `side: DoubleSide`, `depth_write: false`, `basic_mat!` glows, or
  `blending: AdditiveBlending`. They are texture work, not geometry. A
  *solid* body behind a textured face is fine as a `plane` frame part, but
  only on an opaque material.
- **Leave** primitives whose geometry the code later reads or edits vertex by
  vertex (its position attribute). Geometry the placer scales, rotates or
  translates afterwards is fine through `pg_own`, which gives it a private
  copy.
- Tiny bits (under ~1 cm: rivets, ticks, pins) only need a rounded or chamfered
  version (cheap), or can stay primitives. Say which in the notes.
- Gameplay-critical objects (the exit door, pickups, notes, switches, phones)
  may be refined, but must keep exactly the same bounds. They do, by
  construction.

## Workflow

```sh
B=pipes_boxes_and_janitors_things   # a family, or a whole theme: B=industrial
# 1. build it in Blender headless: errors, warnings, triangles, and
#    each part compared with crates/game/assets/prop_meshes.bin
cargo xtask assets props --batch $B
# 2. a full build writes prop_meshes.bin (unchanged parts keep their bytes),
#    then refresh the parts table and review its diff
cargo xtask assets props --write
REGEN=1 cargo test --release -p game --test asset_lint
# 3. in the game: every kind of prop on real levels, framed by a camera
cargo run --release -- --dump-level DIR --seed 4242 --levels 4,5 --screenshot x \
    --prop-census 1 --census-only place_radiators
```

**Handled by the level, not the placer:** the placement pass
(`world::placement`) lifts whole objects that sink into the floor, sets down
hovering ones and pulls props out of walls and out of each other; the census
shots show it. Log *part-to-part* problems (a handle off its mug, a lid
buried in its box, a wrong rotation) in the family's notes.

**Work in small steps.** Write a family a few placers at a time (a header
plus the first 4–6 placers, then the rest, at most ~200 lines per step),
and build after each step.

Look at every image, fix and rebuild until each part reads as the real
object, fits its frame, and sits right in the game. If a census shot shows
nothing useful (a wall, the ceiling), try another `--seed`. Never run two
`cargo xtask assets props --write` at once, and never edit another family's
files.

## Kit reference (`assets/src/blender/propkit.py`)

Shapes are vertex/face lists. Join them with `+` (or `sum([...], Shape())`).
Transform them with `.move(x,y,z)`, `.rot(x,y,z)` (radians), `.scale(x,y,z)`,
`.warp(fn)` and `.fit(lo, hi)`. `.copy()` duplicates a shape.

| Build | |
|---|---|
| `box(w,h,d)`, `rbox(w,h,d,r=None,seg=2)` | box; rounded (seg ≥ 2) or chamfered (seg = 1) edges |
| `cyl(rt,rb,h,seg=32,bevel_r=None,open_ended=False)` | cylinder/frustum on +Y with rounded cap rims |
| `cone(r,h)`, `sphere(r,seg)`, `ellipsoid(rx,ry,rz)`, `capsule(r,len)`, `torus(R,r,seg,tseg,arc)` | |
| `lathe([(r,y),…],seg)` | revolve a profile around +Y (r = 0 closes the end) |
| `vessel([(r,y),…],wall,seg)` | open container: outer profile + rolled rim + inside + floor |
| `tube(points,r,seg,smooth=k,r_end=None)` | sweep along a polyline (Chaikin-smoothed k times): pipes, wires, handles, hooks, chains |
| `prism(outline2d,depth,bevel_r)` | extrude an XY outline along Z (keys, brackets, signs, shaped plates) |
| `rounded_rect(w,h,r)`, `round_outline(pts,r)` | 2D outlines for `prism` |
| `plane(w,h)`, `disc(r)`, `grid_plane(w,h,nx,ny)`, `slab(w,h,t,nx,ny)` | flat and thin things (slab = subdivided thin box) |
| `screw_head(r,h)` | a slotted dome head on +Y |

| Modify | |
|---|---|
| `bevel(s,r,seg,angle=30)` | round every edge sharper than `angle` |
| `cut(a,*cutters)`, `boolean(a,b,op)` | exact booleans (grooves, holes, recesses, slots) |
| `subdivide(s,levels)` | Catmull-Clark: soft, rounded forms |
| `noisy(s,amount,scale,seed,axis=None)` | lumpy, crumpled, worn |
| `bend(s,angle,axis,along)`, `taper(s,top,along,axes)` | bend and taper |
| `repeat(s,n,step,rot)`, `ring_of(s,n,radius,axis)` | arrays |
| `weld(s)` | merge doubles |

Tips:

- `cut` needs closed solids. Build the cutter slightly bigger than the depth
  you want, and poke it through the surface.
- `lathe` profiles go bottom to top. Add points near a corner to round it, or
  use `vessel` for anything open.
- `noisy` on a low-poly shape does nothing useful: subdivide first, or use
  `slab` / `grid_plane`.
- Check in the census shots that normals are not inverted (a part that
  renders dark or inside-out). Constructors fix winding; `.scale(-1, ...)`
  flips it too.

## Where each family is

A folder per theme of `world::tables::PLACER_THEME` (the theme most of a
family's parts serve; *also* names the others), and `game/` for what the
game places itself. A family is the props modelled together, sharing
their kit of shapes; its notes (what each part is, its sources and sizes,
what was left and why) are a record of that work, in
`docs/research/props-remodelling/`. *Was* is the batch it was made in,
which older notes and commits name.

**bathroom**: bathrooms and dressing tables

| File | Also | Notes | Was |
|---|---|---|---|
| [`bathroom_clutter_and_first_toys.py`](bathroom/bathroom_clutter_and_first_toys.py) | toys | [notes](../../../../docs/research/props-remodelling/bathroom_clutter_and_first_toys.md) | `b14` |
| [`dressing_table_clutter.py`](bathroom/dressing_table_clutter.py) | — | [notes](../../../../docs/research/props-remodelling/dressing_table_clutter.md) | `b13` |
| [`washroom_fittings.py`](bathroom/washroom_fittings.py) | core | [notes](../../../../docs/research/props-remodelling/washroom_fittings.md) | `t04` |
| [`washroom_shelf_and_dressing_table.py`](bathroom/washroom_shelf_and_dressing_table.py) | — | [notes](../../../../docs/research/props-remodelling/washroom_shelf_and_dressing_table.md) | `t03` |

**core**: the rooms' own fixtures, marks and litter

| File | Also | Notes | Was |
|---|---|---|---|
| [`everyday_litter.py`](core/everyday_litter.py) | — | [notes](../../../../docs/research/props-remodelling/everyday_litter.md) | `r08` |
| [`floor_marks_and_dust.py`](core/floor_marks_and_dust.py) | — | [notes](../../../../docs/research/props-remodelling/floor_marks_and_dust.md) | `r11a` |
| [`marks_on_the_rooms_surfaces.py`](core/marks_on_the_rooms_surfaces.py) | game | [notes](../../../../docs/research/props-remodelling/marks_on_the_rooms_surfaces.md) | `r07` |
| [`radiators_and_overturned_chairs.py`](core/radiators_and_overturned_chairs.py) | industrial |  | `b00` |
| [`vents_sockets_signs_and_litter.py`](core/vents_sockets_signs_and_litter.py) | — | [notes](../../../../docs/research/props-remodelling/vents_sockets_signs_and_litter.md) | `b01` |
| [`wall_and_ceiling_fixtures.py`](core/wall_and_ceiling_fixtures.py) | — | [notes](../../../../docs/research/props-remodelling/wall_and_ceiling_fixtures.md) | `b02` |
| [`wall_hole_pipe_run_posters_and_hooks.py`](core/wall_hole_pipe_run_posters_and_hooks.py) | domestic, industrial, party | [notes](../../../../docs/research/props-remodelling/wall_hole_pipe_run_posters_and_hooks.md) | `r06` |
| [`wall_marks_and_wear.py`](core/wall_marks_and_wear.py) | industrial | [notes](../../../../docs/research/props-remodelling/wall_marks_and_wear.md) | `r11b` |
| [`what_services_screw_to_walls.py`](core/what_services_screw_to_walls.py) | office, industrial | [notes](../../../../docs/research/props-remodelling/what_services_screw_to_walls.md) | `r12` |

**domestic**: a home's rooms

| File | Also | Notes | Was |
|---|---|---|---|
| [`hall_lamps_pets_and_kettles.py`](domestic/hall_lamps_pets_and_kettles.py) | bathroom | [notes](../../../../docs/research/props-remodelling/hall_lamps_pets_and_kettles.md) | `b12` |
| [`household_things_lying_or_hung.py`](domestic/household_things_lying_or_hung.py) | — | [notes](../../../../docs/research/props-remodelling/household_things_lying_or_hung.md) | `b09` |
| [`kitchen_and_hearth_left_behind.py`](domestic/kitchen_and_hearth_left_behind.py) | industrial, party | [notes](../../../../docs/research/props-remodelling/kitchen_and_hearth_left_behind.md) | `r16` |
| [`kitchen_hall_and_floor_things.py`](domestic/kitchen_hall_and_floor_things.py) | — | [notes](../../../../docs/research/props-remodelling/kitchen_hall_and_floor_things.md) | `t07` |
| [`kitchen_things_frames_and_photos.py`](domestic/kitchen_things_frames_and_photos.py) | — | [notes](../../../../docs/research/props-remodelling/kitchen_things_frames_and_photos.md) | `b10` |
| [`pictures_on_the_walls.py`](domestic/pictures_on_the_walls.py) | victorian, occult, office, game | [notes](../../../../docs/research/props-remodelling/pictures_on_the_walls.md) | `t01` |
| [`shoes_laundry_cleaning_and_travel.py`](domestic/shoes_laundry_cleaning_and_travel.py) | — | [notes](../../../../docs/research/props-remodelling/shoes_laundry_cleaning_and_travel.md) | `b11` |
| [`things_worn_and_dropped.py`](domestic/things_worn_and_dropped.py) | — | [notes](../../../../docs/research/props-remodelling/things_worn_and_dropped.md) | `r14a` |
| [`wood_metal_and_plastic_clutter.py`](domestic/wood_metal_and_plastic_clutter.py) | industrial | [notes](../../../../docs/research/props-remodelling/wood_metal_and_plastic_clutter.md) | `b08` |

**game**: what the game places itself: the core props, the vending machine, the windows, the barricades, the Showroom

| File | Also | Notes | Was |
|---|---|---|---|
| [`barricades.py`](game/barricades.py) | — |  | `x01` |
| [`book_stack_suitcase_coat_stand_and_pile.py`](game/book_stack_suitcase_coat_stand_and_pile.py) | office | [notes](../../../../docs/research/props-remodelling/book_stack_suitcase_coat_stand_and_pile.md) | `r02` |
| [`core_placement.py`](game/core_placement.py) | — | [notes](../../../../docs/research/props-remodelling/core_placement.md) | `b03` |
| [`first_everyday_core_props.py`](game/first_everyday_core_props.py) | — |  | `r01` |
| [`rubber_duck_baby_doll_and_hole.py`](game/rubber_duck_baby_doll_and_hole.py) | — | [notes](../../../../docs/research/props-remodelling/rubber_duck_baby_doll_and_hole.md) | `r03` |
| [`showroom.py`](game/showroom.py) | — |  | `s01` |
| [`torn_note_mannequin_and_wall_clock.py`](game/torn_note_mannequin_and_wall_clock.py) | — | [notes](../../../../docs/research/props-remodelling/torn_note_mannequin_and_wall_clock.md) | `r04` |
| [`vending_machine.py`](game/vending_machine.py) | — |  | `v01` |
| [`washroom_mirror_cobwebs_and_office_plant.py`](game/washroom_mirror_cobwebs_and_office_plant.py) | — | [notes](../../../../docs/research/props-remodelling/washroom_mirror_cobwebs_and_office_plant.md) | `r05` |
| [`what_a_floors_weather_leaves.py`](game/what_a_floors_weather_leaves.py) | — | [notes](../../../../docs/research/props-remodelling/what_a_floors_weather_leaves.md) | `r10` |
| [`windows.py`](game/windows.py) | — |  | `w01` |

**industrial**: plant and service spaces

| File | Also | Notes | Was |
|---|---|---|---|
| [`coat_hook_broom_apron_and_rugs.py`](industrial/coat_hook_broom_apron_and_rugs.py) | domestic | [notes](../../../../docs/research/props-remodelling/coat_hook_broom_apron_and_rugs.md) | `r13b` |
| [`crate_sash_umbrella_and_things_put_away.py`](industrial/crate_sash_umbrella_and_things_put_away.py) | game, domestic, core | [notes](../../../../docs/research/props-remodelling/crate_sash_umbrella_and_things_put_away.md) | `r09` |
| [`pipes_boxes_and_janitors_things.py`](industrial/pipes_boxes_and_janitors_things.py) | office | [notes](../../../../docs/research/props-remodelling/pipes_boxes_and_janitors_things.md) | `b07` |
| [`six_things_hung_on_a_wall.py`](industrial/six_things_hung_on_a_wall.py) | domestic, office, core | [notes](../../../../docs/research/props-remodelling/six_things_hung_on_a_wall.md) | `r13a` |
| [`washroom_janitor_classroom_and_plant.py`](industrial/washroom_janitor_classroom_and_plant.py) | office, bathroom, nature, victorian | [notes](../../../../docs/research/props-remodelling/washroom_janitor_classroom_and_plant.md) | `r18` |

**nature**: plants, gardens, what grows or nests

| File | Also | Notes | Was |
|---|---|---|---|
| [`a_garden_and_a_wood_indoors.py`](nature/a_garden_and_a_wood_indoors.py) | — | [notes](../../../../docs/research/props-remodelling/a_garden_and_a_wood_indoors.md) | `t17` |
| [`bits_of_the_outside_indoors.py`](nature/bits_of_the_outside_indoors.py) | bathroom | [notes](../../../../docs/research/props-remodelling/bits_of_the_outside_indoors.md) | `t05` |
| [`forest_floor_and_potting_shed.py`](nature/forest_floor_and_potting_shed.py) | — | [notes](../../../../docs/research/props-remodelling/forest_floor_and_potting_shed.md) | `b24` |
| [`garden_things.py`](nature/garden_things.py) | bathroom | [notes](../../../../docs/research/props-remodelling/garden_things.md) | `t06` |

**occult**: the occult and Halloween

| File | Also | Notes | Was |
|---|---|---|---|
| [`a_shrines_things.py`](occult/a_shrines_things.py) | party, victorian | [notes](../../../../docs/research/props-remodelling/a_shrines_things.md) | `t09` |
| [`halloween_things.py`](occult/halloween_things.py) | — | [notes](../../../../docs/research/props-remodelling/halloween_things.md) | `t08` |
| [`shrine_and_seance_things.py`](occult/shrine_and_seance_things.py) | — | [notes](../../../../docs/research/props-remodelling/shrine_and_seance_things.md) | `t18` |
| [`shrine_relics_and_garden_bits.py`](occult/shrine_relics_and_garden_bits.py) | nature | [notes](../../../../docs/research/props-remodelling/shrine_relics_and_garden_bits.md) | `b23` |
| [`victorian_curios_and_halloween.py`](occult/victorian_curios_and_halloween.py) | victorian | [notes](../../../../docs/research/props-remodelling/victorian_curios_and_halloween.md) | `b22` |

**office**: offices

| File | Also | Notes | Was |
|---|---|---|---|
| [`boards_tapes_lockers_and_gadgets.py`](office/boards_tapes_lockers_and_gadgets.py) | — | [notes](../../../../docs/research/props-remodelling/boards_tapes_lockers_and_gadgets.md) | `b05` |
| [`fountains_boards_cabinets_and_laptops.py`](office/fountains_boards_cabinets_and_laptops.py) | — | [notes](../../../../docs/research/props-remodelling/fountains_boards_cabinets_and_laptops.md) | `b04` |
| [`maps_charts_prints_and_posters.py`](office/maps_charts_prints_and_posters.py) | victorian, domestic, occult | [notes](../../../../docs/research/props-remodelling/maps_charts_prints_and_posters.md) | `t02` |
| [`mugs_newspapers_letters_and_phones.py`](office/mugs_newspapers_letters_and_phones.py) | — | [notes](../../../../docs/research/props-remodelling/mugs_newspapers_letters_and_phones.md) | `b06` |
| [`paper_and_small_things_on_office_floors.py`](office/paper_and_small_things_on_office_floors.py) | party, domestic | [notes](../../../../docs/research/props-remodelling/paper_and_small_things_on_office_floors.md) | `r15` |
| [`small_things_dropped_or_left.py`](office/small_things_dropped_or_left.py) | domestic, industrial | [notes](../../../../docs/research/props-remodelling/small_things_dropped_or_left.md) | `r14b` |

**party**: parties

| File | Also | Notes | Was |
|---|---|---|---|
| [`a_party_tables_leavings.py`](party/a_party_tables_leavings.py) | — | [notes](../../../../docs/research/props-remodelling/a_party_tables_leavings.md) | `t13` |
| [`party_remnants_and_antiques.py`](party/party_remnants_and_antiques.py) | victorian | [notes](../../../../docs/research/props-remodelling/party_remnants_and_antiques.md) | `b17` |
| [`small_everyday_things.py`](party/small_everyday_things.py) | core, toys, bathroom, office | [notes](../../../../docs/research/props-remodelling/small_everyday_things.md) | `t22` |
| [`what_a_party_room_leaves.py`](party/what_a_party_room_leaves.py) | — | [notes](../../../../docs/research/props-remodelling/what_a_party_room_leaves.md) | `t12` |

**toys**: toys and play

| File | Also | Notes | Was |
|---|---|---|---|
| [`a_play_areas_toys.py`](toys/a_play_areas_toys.py) | — | [notes](../../../../docs/research/props-remodelling/a_play_areas_toys.md) | `t16` |
| [`chunky_plastic_rubber_and_plush.py`](toys/chunky_plastic_rubber_and_plush.py) | party | [notes](../../../../docs/research/props-remodelling/chunky_plastic_rubber_and_plush.md) | `b16` |
| [`instruments_nursery_toys_and_leftovers.py`](toys/instruments_nursery_toys_and_leftovers.py) | — | [notes](../../../../docs/research/props-remodelling/instruments_nursery_toys_and_leftovers.md) | `b15` |
| [`soft_toys_and_dolls.py`](toys/soft_toys_and_dolls.py) | — | [notes](../../../../docs/research/props-remodelling/soft_toys_and_dolls.md) | `t14` |
| [`toys_left_on_the_floor.py`](toys/toys_left_on_the_floor.py) | — | [notes](../../../../docs/research/props-remodelling/toys_left_on_the_floor.md) | `t15` |
| [`toys_on_the_floor.py`](toys/toys_on_the_floor.py) | — | [notes](../../../../docs/research/props-remodelling/toys_on_the_floor.md) | `r17` |

**victorian**: old houses

| File | Also | Notes | Was |
|---|---|---|---|
| [`a_studys_things.py`](victorian/a_studys_things.py) | — | [notes](../../../../docs/research/props-remodelling/a_studys_things.md) | `t20` |
| [`brass_mechanisms_and_showpieces.py`](victorian/brass_mechanisms_and_showpieces.py) | — | [notes](../../../../docs/research/props-remodelling/brass_mechanisms_and_showpieces.md) | `b19` |
| [`chandeliers_trophies_and_clocks.py`](victorian/chandeliers_trophies_and_clocks.py) | — | [notes](../../../../docs/research/props-remodelling/chandeliers_trophies_and_clocks.md) | `b18` |
| [`four_larger_things.py`](victorian/four_larger_things.py) | occult | [notes](../../../../docs/research/props-remodelling/four_larger_things.md) | `t21` |
| [`hall_stand_bell_rack_and_curios.py`](victorian/hall_stand_bell_rack_and_curios.py) | — | [notes](../../../../docs/research/props-remodelling/hall_stand_bell_rack_and_curios.md) | `b21` |
| [`lodge_trophies_and_parlour_ornaments.py`](victorian/lodge_trophies_and_parlour_ornaments.py) | — | [notes](../../../../docs/research/props-remodelling/lodge_trophies_and_parlour_ornaments.md) | `t11` |
| [`parlour_and_office_big_things.py`](victorian/parlour_and_office_big_things.py) | office | [notes](../../../../docs/research/props-remodelling/parlour_and_office_big_things.md) | `t10` |
| [`things_on_a_wall_and_one_box.py`](victorian/things_on_a_wall_and_one_box.py) | occult | [notes](../../../../docs/research/props-remodelling/things_on_a_wall_and_one_box.md) | `t19` |
| [`writing_desk_tea_tray_and_study.py`](victorian/writing_desk_tea_tray_and_study.py) | — | [notes](../../../../docs/research/props-remodelling/writing_desk_tea_tray_and_study.md) | `b20` |
