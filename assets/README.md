# Asset sources

The game embeds two mesh files, `crates/game/assets/prop_meshes.bin` and
`crates/game/assets/creature_meshes.bin`, and the Building's photographed
surfaces, `crates/game/assets/textures/*.jpg`. This folder holds what they
are built from: Blender (bpy) scripts, no binary models; and the scripts that
make textures of CC0 photographs, no images. No generative model is used (see
`crates/game/assets/MANIFEST.json`).

| Path | What |
|---|---|
| `src/blender/props.py` | builds every prop part: runs each batch module in `src/blender/props/` (`bNN.py`, `s01.py`; `zz_selftest.py` is a test batch) |
| `src/blender/propkit.py` | the shape kit the batches use (boxes, lathes, sweeps, noise, booleans, the `finish` step that bakes normals) and the triangle budgets |
| `src/blender/props/*.map.json`, `*.notes.md` | which placer primitive each part replaces, and the batch's notes |
| `src/blender/creatures.py`, `skinned.py`, `skin_*.py`, `SKINNED.md` | the creatures: rigid parts, and skinned bodies built over rigs |
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

**Creatures** need the rig dumps of the HTML version (its
`tools/rig-dump.mjs` writes `.cache/rigs/<type>.json`), which are not ported
yet. Rebuild them there (`creatures.py`, then `skinned.py` per type, then
`creature-pack.mjs`), and convert the module it writes:

```sh
cargo xtask assets import-js --creatures ../src/creature-meshes.js
```

(`--props ../src/prop-meshes.js` converts the HTML version's prop module the
same way; both conversions reproduce the files byte for byte.)

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

