# Skinned creatures: the contract

The creatures used to be a rigid part per joint, which read as an action
figure: a visible ball at every knee and elbow. Now each creature's body is
**one continuous skinned mesh** bound to the game's own rig, and the old
procedural animation drives the same joints, bending skin instead of moving
separate pieces. Heads, teeth, eyes, hair and similar stay rigid parts on
their bones (`tools/blender/creatures.py`, as before).

The Smiler is the worked example: `makeSmiler` in `backrooms.js`, and
`tools/blender/skin_smiler.py`.

## The loop

```sh
T=smiler                                   # smiler | hound | staff | shadow | watcher
node build-standalone.mjs backrooms        # the game, with your rig changes
node tools/rig-dump.mjs $T                 # its rest pose -> .cache/rigs/$T.json
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
    --python tools/blender/skinned.py -- $T          # -> <name>Body in src/creature-meshes.js
node tools/creature-pack.mjs               # deflate what the Blender scripts wrote plain
node build-standalone.mjs backrooms        # again, with the new body
node tools/entity-shot.mjs --only smiler --out shots/skin/$T                       # portraits
node tools/entity-shot.mjs --only smiler --out shots/skin/$T --gap 1.3 --pitch 0.32 --views front,34,side --tag close
node tools/entity-motion.mjs --levels 5,7  # smoothness numbers (turn, accel, zigzag, joint speed)
```

Rigid parts (heads etc.): `Blender ... --python tools/blender/creatures.py --
src/creature-meshes.js partA,partB` rebuilds only those parts. Both scripts
lock `src/creature-meshes.js` while they rewrite it, so several builds can
run at once.

## The rig (backrooms.js)

- Build joints with `joint(geo, mat, parent, x, y, z, name)`. It makes a
  named `THREE.Bone`; `geo` is the rigid part it carries, or `null` when the
  creature has a skinned body (see `rigid()` in `makeSmiler`).
- Every bone the body bends at needs a unique name (`thigh.L`, `finger21.R`).
- `userData.tail = [x, y, z]` (local) gives a leaf bone its direction;
  `userData.main = 'childName'` picks which child a bone points at. These
  only matter for weighting (a bone's segment runs from its head to its tail).
- The **rest pose** is the pose the make-function builds, before any
  animation. The body is modelled in it and bound in it, so set every rest
  rotation there (not in the animation).
- After the rig is built, and before `g.position` is set:
  `if (body) P.skin = skinBody(g, body, mat);` (`body` is
  `creatureGeometry(THREE, '<name>Body')`, null until you have built one).
- `skinDetail`: wrap the skin material in `withSkinDetail(mat, scale, strength)`
  for pores, creases and varied shine (Smiler: `3.1, 1.05`).
- Keep what the tests read: `parts.legs`, `parts.arms` (Smiler: hip/knee,
  shoulder/elbow), `parts.head` whose first small `SphereGeometry` child is
  an eye, `parts.neck`, `parts.jaw` (null for the Watcher), `parts.tail` for
  the Hound, and `window.__game.rigs`.

## The body (tools/blender/skin_<type>.py)

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
parts on top. Up to five creatures can be on screen. Check with
`node tools/bench.mjs --levels 3,5,7` that frame times stay where they were.
