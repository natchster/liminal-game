---
paths:
  - "crates/game/src/visuals/**"
  - "crates/engine/src/*.metal"
  - "crates/engine/src/render.rs"
  - "crates/engine/src/material.rs"
  - "crates/engine/src/texture.rs"
  - "crates/game/src/assets/**"
---

# Rendering and look rules

- **Light is place-based.** The light map (`visuals/lighting.rs`) decides
  ambient and fog per tile; never tie them to the camera, and no lamp cones.
- **Realistic, not cartoonish.** Props get their settled look in
  `visuals/prop_look.rs` (`Material::look_settled`). Keep base colours
  inside physically plausible albedo (max channel about 0.03–0.87 for
  non-metals) and avoid saturated, flat colour.
- **Canvas textures draw from a stream of their own** (`canvas_tex`:
  `with_own_stream`, seeded by the painter's place in the code, its size and
  how many it has made since the level began); never the level's stream nor
  the per-frame one. So moving a painter's code repaints its textures, and
  so does handing a fn over in place of its closure (`|c, w, h| paint(c,
  w, h)` → `paint`, as clippy's `redundant_closure` would): the placers'
  fingerprints say which; keep the closure, or re-record with the reason.
  The name is coarse: the closures written in one fn share it and its
  count, and so do placers painting through one shared helper (fixed by
  the build's order, so a level repeats, but one placer's paints move
  another's pixels); and `type_name` is not promised to stay the same from
  one compiler to the next (queued: explicit painter keys). Never key a
  stream by something a cache hides (a placer's name, as O3b did): a cached
  texture paints once a thread, and a level's pixels would hang on what was
  built before (the fingerprints start each placement as a level does,
  `kit::level_begins`, and run it on a fresh thread too, and fail on that). Never draw on a canvas from two threads at once (CoreGraphics
  rasterizes differently then); a pure per-pixel loop may use `par_rows`. The Building's surfaces
  are photographs (`visuals/photo.rs`, built by `assets/src/textures/`) and
  draw nothing; a new one comes from a CC0 photograph listed in
  `sources.json` (the manifest allows MIT, CC0, CC BY 4.0 only), graded to
  the surface it replaces. The rooms keep their painted surfaces, with a
  photographed detail laid over them in world space (`photo::arch_detail`,
  `detail_for`; `docs/design/ROOMS.md`): a new one is a grey map from a CC0
  photograph (`build.py` `detail`), at the material's real size, quilted if
  a feature of it would repeat on a grid, kept only where a blind review
  prefers it.
- **Shader cost:** a texture read in a global patch multiplies pipeline
  compile time. Judge by level load plus first frame.
- **Gates:** `cargo xtask shots` (golden frames, look anchors per frame and
  over the set) and `tests/materials.rs` (albedo lint). Accept new goldens
  only after looking at the frames and the error maps in
  `target/xtask/shots/`, with a `--reason`; move a look anchor only when the
  look is meant to change.
- **Evidence:** zone views (`--dump-level … --screenshot x --views`) and
  creature showcases before and after; a blind review for any change of
  look (both orders, a fresh reviewer, ties on disagreement); after a
  change to materials, textures or lighting, the visual playtest of the
  places it touched (`visual-playtest`).
