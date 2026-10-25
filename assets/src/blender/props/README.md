# Prop parts: the modelling contract

Every prop in the game is built by a *placer* in `backrooms.js` (about 650
`function placeXxx(dist)`) from three.js primitives. This directory rebuilds
those props in Blender, finer: each primitive that matters gets a real modelled
part. A part is swapped in at runtime by `PG('Name__role', new THREE.XGeometry(...))`
(see `src/props.js`) and **fitted to the primitive's bounds**, so the placer's
layout, sizes, randomness, colours, animation and gameplay stay exactly as they
are. Only the shape gets better.

The pilot batch `b00.py` / `b00.map.json` (radiator, overturned chair) is the
worked example. Read it first.

## Files per batch

| File | What |
|---|---|
| `bNN.py` | The parts: `part(name, frame, build)` calls (Blender Python, `from propkit import *`) |
| `bNN.map.json` | Which constructor in which placer becomes which part |
| `bNN.notes.md` | Per placer: what you rebuilt, what you left and why, placement problems you saw |

`BATCHES.json` says which placers belong to which batch.

## A part

```python
part('Radiators__body', f_box(0.70, 0.50, 0.10), radiator_body)   # build is a function returning a Shape
```

- **Name:** `<PlacerNameWithoutPlace>__<role>`, e.g. `PocketWatches__case`. Use
  letters, digits and `_` only. A part may be shared by several constructors
  (same object, same frame kind), even across placers in your batch.
- **Frame:** the primitive it replaces, with that primitive's *canonical*
  arguments as written in the placer. If an argument is an expression, use a
  typical value. The part is modelled **in the primitive's frame**:

| three.js primitive | frame | where the part must sit |
|---|---|---|
| `BoxGeometry(w, h, d)` | `f_box(w, h, d)` | centred on the origin |
| `CylinderGeometry(rt, rb, h)` | `f_cyl(rt, rb, h)` | axis +Y, centred, y from −h/2 to h/2, top radius rt |
| `ConeGeometry(r, h)` | `f_cone(r, h)` | axis +Y, centred, apex at +h/2 |
| `SphereGeometry(r)` | `f_sphere(r)` | centred |
| `TorusGeometry(R, r)` | `f_torus(R, r)` | ring in the **XY plane** around the Z axis |
| `CapsuleGeometry(r, len)` | `f_capsule(r, len)` | axis +Y, centred, height len + 2r |
| partial sphere/cylinder, Lathe, Shape, Octahedron… | `f_bounds(lo, hi)` | the primitive's own bounds |
| `PlaneGeometry(w, h)` / `CircleGeometry(r)` | `f_bounds((-w/2,-h/2,0),(w/2,h/2,0), 'plane')` | front face at z = 0 facing +Z; any thickness goes **behind** (−Z, into the wall) |

- Coordinates are three.js: **+Y up**, metres. Blender does not care; the kit
  builds straight in these coordinates.
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
  details. The whole game ships these meshes, so cheap is good.
- **Shading:** `part(..., smooth=35)` is the crease angle. Hard surfaces keep
  the default (weighted normals make flat faces flat and bevels soft). For
  organic or soft things use `smooth=180`; use `flat=True` for a faceted look.

## What to replace, what to leave

- Replace every solid primitive a player would look at: box, cylinder,
  sphere, torus, cone, capsule, and the Octahedron and Lathe ones where a
  better shape exists.
- **Leave** decals and flat textured quads (stains, posters, labels, clock
  faces, screens, shadows, glows) and anything with `transparent: true`,
  `side: THREE.DoubleSide`, `depthWrite: false`, `MeshBasicMaterial` glows, or
  `blending: THREE.AdditiveBlending`. They are texture work, not geometry. A
  *solid* body behind a textured face is fine as a `plane` frame part, but
  only on an opaque material.
- **Leave** primitives whose geometry the code later reads or edits vertex by
  vertex (`.attributes.position`, `setXYZ`). Geometries changed with
  `.scale/.rotateX/.translate` are fine: the codemod gives them a private copy.
- Tiny bits (under ~1 cm: rivets, ticks, pins) only need a rounded or chamfered
  version (cheap), or can stay primitives. Say which in the notes.
- Gameplay-critical objects (the exit door, pickups, notes, switches, phones)
  may be refined, but must keep exactly the same bounds. They do, by
  construction.

## The map

```json
[
  { "placer": "placeRadiators", "ctor": "new THREE.BoxGeometry(0.70, 0.50, 0.10)", "part": "Radiators__body" },
  { "placer": "placeLamp", "ctor": "new THREE.CylinderGeometry(0.02, 0.02, h, 8)", "part": "Lamp__pole", "occ": 1 }
]
```

- `ctor` is the constructor text **copied from the placer**. Matching ignores
  whitespace, but everything else must be exact, including the arguments.
- Without `occ`, every identical constructor text in that placer is mapped.
  With `occ: k`, only the k-th (0-based) occurrence is.
- Every entry must match (the gallery build prints "map problems").

## Workflow (run from the project root)

```sh
B=b07
# 1. build your parts in Blender (fast): errors, warnings, triangles
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
    --python tools/blender/props.py -- --batch $B          # add --parts A,B to rebuild a few
# 2. contact sheet: the old primitive (left) vs your part (right) in every cell
node tools/prop-preview.mjs --batch $B --out shots/props/$B.png      # --page 1 … for >48 parts, --uv checks UVs
# 3. in the game: a private build with only your batch applied, each placer
#    forced to run on a real level and framed by a camera
node tools/prop-gallery.mjs --batch $B --out shots/props/${B}_game.png --from 0 --count 12
node tools/prop-gallery.mjs --batch $B --level 2 ...  # other levels have other zones and lighting
```

**Also handled centrally:** the level's placement pass (`src/placement.js`)
lifts whole objects that sink into the floor, sets down hovering ones and
pulls props out of walls and out of each other, and the gallery shows that.
Log *part-to-part* problems (a handle off its mug, a lid buried in its box, a
wrong rotation).

**Already fixed centrally, don't log it:** wall props that hung ~1.5 m off the
wall facing it (`wx(x) + dx * 0.485`, `atan2(dx, dz)`) were moved onto the
walls by `tools/fix-wall-offsets.mjs`. Keep private scratch scripts in your
own subfolder of the session scratchpad (e.g. `scratchpad/b15w/`); other
agents share the scratchpad root.

**Work in small steps.** Write `bNN.py` a few placers at a time: `Write` a
header plus the first 4–6 placers, then append the rest with `Edit`, at most
~200 lines per step, and build after each step. A single huge write can stall
the session and lose everything.

Look at every image with the Read tool, fix and rebuild until each part reads
as the real object, fits its frame, and sits right in the game. If a gallery
shot shows nothing useful (a wall, the ceiling), try another `--seed`.

**Never** run `node tools/prop-assemble.mjs`, `node tools/prop-codemod.mjs`
without `--check`, `node build-standalone.mjs`, `npm run build` or `npm test`,
and never edit `backrooms.js`, `src/*` or another batch's files. Other batches
are being built at the same time; the gallery and preview tools give you a
private build instead. Integration is done centrally afterwards.

## Kit reference (`tools/blender/propkit.py`)

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
- Check in the preview that normals are not inverted (a part that renders dark
  or inside-out). Constructors fix winding; `.scale(-1, ...)` flips it too.
