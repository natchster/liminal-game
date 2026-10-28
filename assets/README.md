# Asset sources

The game embeds three mesh files, `crates/game/assets/prop_meshes.bin`,
`crates/game/assets/creature_meshes.bin` and
`crates/game/assets/bodies.bin`, and the Building's photographed surfaces,
`crates/game/assets/textures/*.jpg`. This folder holds what they are built
from: Blender (bpy) scripts, no binary models; and the scripts that
make textures of CC0 photographs, no images. No generative model is used (see
`crates/game/assets/MANIFEST.json`).

| Path | What |
|---|---|
| `src/blender/props.py` | builds every prop part: runs each batch module in `src/blender/props/` (`bNN.py`, `s01.py`; `zz_selftest.py` is a test batch) |
| `src/blender/propkit.py` | the shape kit the batches use (boxes, lathes, sweeps, noise, booleans, the `finish` step that bakes normals) and the triangle budgets |
| `src/blender/props/*.map.json`, `*.notes.md` | which placer primitive each part replaces, and the batch's notes |
| `src/blender/creatures.py`, `skinned.py`, `skin_*.py`, `SKINNED.md` | the old creatures: rigid parts, and skinned bodies built over the HTML version's rigs |
| `src/creatures/build.py`, `kit.py`, `hm08.py`, `body_*.py`, `sources.json` | the creatures' bodies (`bodies.bin`): skeletons, MakeHuman CC0 bodies and their targets, bone-heat weights |
| `src/textures/sources.json` | the photographs the Building's surfaces are made of: where each is, its author, licence (CC0) and SHA-1 |
| `src/textures/build.py`, `texlib.py` | fetch and check the photographs, and make each texture: rectify, cut along joints, quilt (Efros and Freeman), wrap along least-error seams, take off the slow light, grade to the surface it replaced |

## Rebuilding

**Props** (reproducible here):

```sh
cargo xtask assets props                 # build every batch in Blender headless, compare part by part
cargo xtask assets props --batch b01     # one batch (compare only)
cargo xtask assets props --write         # a full build replaces prop_meshes.bin (unchanged parts keep their bytes)
REGEN=1 cargo test --release -p game --test asset_lint   # then refresh the parts table, and review its diff
```

Blender is found as `$BLENDER`, else `blender` on the path, else
`/Applications/Blender.app`. Every part's randomness is seeded from its name,
and noise vectors come from scalar Perlin samples (`propkit.noise3`;
`mathutils.noise.noise_vector` gives different values in every Blender
process), so two builds are identical and a rebuild matches the committed
file exactly.

**Creatures** (reproducible here; the redesign of `docs/ENTITIES.md`):

```sh
cargo xtask assets creatures                  # build every body in Blender headless, compare body by body
cargo xtask assets creatures --only hound     # some (compare only)
cargo xtask assets creatures --write          # a full build replaces bodies.bin and records its fingerprint
```

`src/creatures/build.py` runs every `body_<name>.py` beside it in an empty
scene. A body script builds a skeleton (`kit.Skel`: bones with a parent, a
head and a tail, in the game's frame), its meshes (MakeHuman's base body
with its targets (`hm08.py`), or metaballs, then voxel remesh,
displacement, decimation and vertex paint), and binds them with Blender's
bone heat (`kit.Body.skinned`, four bones a vertex) or to one bone
(`kit.Body.rigid`). Posing (`human.pose`) records which bone each vertex
follows (`posed.owner`); `anatomy.carry_values` carries it, or any mask,
through a remesh or decimation, so wasting a limb (`anatomy.waste`) thins
the arm's own skin and not the chest beside it, and `anatomy.relax_offsets`
smooths the offsets so no part steps against its neighbour. Faces are
sculpted in the head bone's rest frame, where MakeHuman's landmarks are
(the Smiler's orbits, cheeks and rictus: `body_smiler.sculpt_face`), and
teeth are built one by one along a dental arch fitted to MakeHuman's
(`anatomy.Arch`, `anatomy.teeth_row`: incisors, canines, premolars and
molars, each a little different). The xtask packs what it wrote (int16 positions, int8
normals, sRGB bytes, bone slots and weights, morph offsets; see
`crates/game/src/assets/bodies.rs`) into `crates/game/assets/bodies.bin`.
Workbench previews of every body are written beside its JSON in
`target/xtask/assets/creatures/`; judge a body in the game, under the lamp,
not there. MakeHuman's files (CC0 1.0 since September 2020; its
`LICENSE.md`, section C) are listed in `src/creatures/sources.json` with
their SHA-1s and fetched once into `target/creature-sources/`. Noise is
`kit.noise3` (hashed value noise in NumPy), so the build repeats byte for
byte.

The **old creatures** (`creature_meshes.bin`) need the rig dumps of the
HTML version (its `tools/rig-dump.mjs` writes `.cache/rigs/<type>.json`),
which were never ported. Rebuild them there (`creatures.py`, then
`skinned.py` per type, then `creature-pack.mjs`), and convert the module it
writes:

```sh
cargo xtask assets import-js --creatures ../src/creature-meshes.js
```

(`--props ../src/prop-meshes.js` converts the HTML version's prop module the
same way; both conversions reproduce the files byte for byte.) They go as
each creature moves to the new bodies.

## Gates

- `crates/game/tests/asset_lint.rs` — mesh lint (indices, bounds, normals,
  degenerate triangles, distance from the modelling frame, triangle budgets,
  skin weights) and the parts table `crates/game/assets/parts.tsv`.
- `crates/backrooms/tests/structure.rs` — the manifest covers every embedded
  file, its fingerprint matches, and its licence and generator are allowed.

**Textures** (reproducible here; Python with Pillow, NumPy and SciPy):

```sh
cargo xtask assets textures              # fetch the photographs once (target/texture-sources/), build, compare
cargo xtask assets textures --write      # replace crates/game/assets/textures/ and record the fingerprints
PYTHON=path/to/venv/bin/python cargo xtask assets textures   # a Python that has them
```

Each photograph is checked against its SHA-1 (Commons publishes one; the
Panelka files are checked inside the pack's zip) before anything is built.
Every step is seeded, so a rebuild with the same libraries gives the same
pixels; a different Pillow, NumPy or SciPy can move a texture by a level or
two, which the compare reports. The builds write a 3x3 tiled preview of each
texture to `target/textures/` beside it: look at it for seams.

