# Prop parts: the modelling contract

Every prop in the game is built by a *placer* (a `place_xxx(g, dist)` in a
placer crate under `crates/placers/`, about 650 of them, or a core-props
builder in `crates/game/src/core_props/`) from the engine's primitives
(`box_geo!`, `cyl_geo!`, `sphere_geo!`, …). This directory makes those
props finer in Blender. There are two kinds of batch:

- **Fitted parts** (`bNN.py`, `s01.py`): each primitive that matters gets a
  real modelled part, swapped in where the placer wraps it,
  `pg("Name__role", box_geo!(…))` (or `pg_own`, a private copy the placer
  may change afterwards; `crates/game/src/placers/kit.rs`), and **fitted to
  the primitive's bounds** (`assets::props::prop_part`), so the placer's
  layout, sizes, randomness, colours, animation and gameplay stay exactly as
  they are. Only the shape gets better. This file is their contract.
- **Real things** (`rNN.py`, `tNN.py`, `v01.py`, `w01.py`, `x01.py`): the
  prop remodelled as the real object at its real size, each part registered
  by the batch's own `auto(name, build, budget)` and used as modelled. Their
  brief is `docs/PROPS.md`, "How a prop is remodelled".

The pilot batch `b00.py` (radiator, overturned chair) is the worked example
of a fitted batch. Read it first.

## Files per batch

| File | What |
|---|---|
| `bNN.py` | The parts: `part(name, frame, build)` calls (Blender Python, `from propkit import *`) |
| `bNN.notes.md` | Per placer: what was rebuilt, what was left and why, placement problems seen |

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
B=b07
# 1. build the batch in Blender headless: errors, warnings, triangles, and
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
buried in its box, a wrong rotation) in the batch's notes.

**Work in small steps.** Write `bNN.py` a few placers at a time (a header
plus the first 4–6 placers, then the rest, at most ~200 lines per step),
and build after each step.

Look at every image, fix and rebuild until each part reads as the real
object, fits its frame, and sits right in the game. If a census shot shows
nothing useful (a wall, the ceiling), try another `--seed`. Never run two
`cargo xtask assets props --write` at once, and never edit another batch's
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
