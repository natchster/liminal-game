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
- **Canvas textures draw from the native stream** (`canvas_tex` switches to
  it); never take build randomness while painting.
- **Shader cost:** a texture read in a global patch multiplies pipeline
  compile time. Judge by level load plus first frame.
- **Evidence:** zone views (`--dump-level … --screenshot x --views`) and
  creature showcases before and after; a blind review for any change of
  look (both orders, a fresh reviewer, ties on disagreement).
