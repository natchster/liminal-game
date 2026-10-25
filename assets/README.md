# Asset sources

The game embeds two mesh files, `crates/game/assets/prop_meshes.bin` and
`crates/game/assets/creature_meshes.bin`. This folder holds what they are
built from: Blender (bpy) scripts, no binary models. Everything is procedural
and scripted; no generative model is used (see `crates/game/assets/MANIFEST.json`).

| Path | What |
|---|---|
| `src/blender/props.py` | builds every prop part: runs each batch module in `src/blender/props/` (`bNN.py`, `s01.py`; `zz_selftest.py` is a test batch) |
| `src/blender/propkit.py` | the shape kit the batches use (boxes, lathes, sweeps, noise, booleans, the `finish` step that bakes normals) and the triangle budgets |
| `src/blender/props/*.map.json`, `*.notes.md` | which placer primitive each part replaces, and the batch's notes |
| `src/blender/creatures.py`, `skinned.py`, `skin_*.py`, `SKINNED.md` | the creatures: rigid parts, and skinned bodies built over rigs |

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
