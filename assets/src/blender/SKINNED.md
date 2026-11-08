# The old creatures' skinned bodies: how they were made

The old creatures' bodies in `crates/game/assets/creature_meshes.bin` are
kept as stored (see `assets/README.md`): no build here makes them again,
because these scripts build over dumps of each rig's rest pose that nothing
in this repository writes. This is the record of how they were made. The
creatures of the redesign are built by `assets/src/creatures/` instead
(`cargo xtask assets creatures`); of this file's parts the game still uses
the Borrowed House's shadows' body (`shBody`) and the Stairwell's face
(`stairFace`).

Each body was **one continuous skinned mesh** bound to the creature's rig,
so the animation bends skin instead of moving separate pieces (a rigid part
per joint had read as an action figure: a visible ball at every knee and
elbow). Heads, teeth, eyes, hair and similar stayed rigid parts on their
bones (`creatures.py`). The Smiler was the worked example
(`skin_smiler.py`).

## The build

```sh
T=smiler                                   # smiler | hound | staff | shadow | watcher
# the rig's rest pose, dumped to .cache/rigs/$T.json (every bone's name,
# parent, rest matrix, head and tail)
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
    --python assets/src/blender/skinned.py -- $T          # -> the part <name>Body
```

Rigid parts (heads etc.): `Blender ... --python assets/src/blender/creatures.py
-- OUT partA,partB` rebuilds only those parts. Both scripts lock their
output while they rewrite it, so several builds can run at once. The parts
were then packed into `creature_meshes.bin` (`assets::creatures` decodes
it).

## The rig

- Every bone the body bends at needs a unique name (`thigh.L`, `finger21.R`).
- A leaf bone's tail (local `[x, y, z]`) gives it its direction, and a bone
  with several children points at the one named its main child. These only
  matter for weighting (a bone's segment runs from its head to its tail).
- The **rest pose** is the pose the rig is built in, before any animation.
  The body is modelled in it and bound in it, so every rest rotation is set
  there (not in the animation).
- In the game, a body is bound to the built rig with
  `entities::skin_body(root, geo, mat)` (`geo` from
  `assets::creatures::creature_geometry("<name>Body")`), and its material
  wrapped in `entities::with_skin_detail(mat, scale, strength)` for pores,
  creases and varied shine (the Smiler's was `3.1, 1.05`).

## The body (`skin_<type>.py`)

Run by `skinned.py` in its namespace: `C` is creatures.py's namespace (all
its helpers — `ell`, `tube`, `chain`, `sweep`, `loft_y`, `M`, `xform` — and
part builders such as `C['sm_thigh']`), plus `Piece`, `V`, `BONES`, `math`.
It defines `BODY`:

```python
BODY = dict(name='smBody', pieces=smiler_pieces, paint=smiler_paint,
            body=dict(voxel=0.0034, smooth=(4, 0.5), noise=0.0011, tris=15000),
            fine=dict(voxel=0.0022, smooth=(2, 0.5), noise=0.0004, tris=5200))
```

- A `Piece(build, frame, bones=None, group='body', local=None, sig=0.04)` is
  `build(bm)` modelled in bone `frame`'s rest frame (bone at the origin; a
  limb hangs down −Y, +Z forward, as creatures.py's limb builders already
  are), placed by that bone's rest matrix. It weights to `bones` (default:
  the frame bone); with several bones, a vertex is split along their
  segments, softly over `sig` metres (a torso over the spine bones).
- Split a piece at every joint that should bend inside it (one piece per
  finger bone), or give it several bones.
- Pieces must overlap where they meet: the voxel remesh fuses them. Where
  they don't, add a bridge (an ellipsoid at the joint).
- Groups: `body` and `fine` are remeshed separately (fine voxels keep
  fingers and toes apart) and joined; give each group a triangle budget.
- `paint(p, n, cav)` returns sRGB albedo; `cav` is 0 on a dome, 1 in a crease
  (darken creases; it is the only ambient occlusion there is).
- Weights: each vertex follows the pieces it lies on, blended over ~1 cm
  where pieces meet, and only between pieces whose bones meet at a joint
  (the same bone, parent and child, or two root bones). A claw curled by
  the knee or two fingers side by side never borrow each other's weight.
  So a joint that should bend smoothly needs its two pieces on parent and
  child bones (or one piece with both bones). Look at bends in the game (a
  knee at 90 degrees, a clenched fist): a fold that collapses means a piece
  is missing or a bone segment is wrong.

## Budgets

About 20,000 triangles for a body (both groups), the head and other rigid
parts on top. Up to five creatures can be on screen; `cargo xtask bench`
says whether frame times stay where they were.
