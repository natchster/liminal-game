# Architecture of the native build

The browser game is one large ES module (`backrooms.js`, ~71k lines) plus a
few helper modules (`src/*.js`), three.js r160, an HTML page and a stylesheet.
The native build keeps that structure and reproduces each layer the browser
provided with a macOS equivalent:

| Browser | Native |
|---|---|
| three.js r160 (scene graph, geometry, materials, WebGLRenderer) | `engine::{scene, geometry, material, texture, math}` + the Metal renderer `engine::render` |
| WebGL shaders (`ShaderLib` chunks, the game's `onBeforeCompile` patches) | `engine/src/shaders.metal` (one uber-shader specialised with function constants) |
| `<canvas>` 2D contexts (procedural textures, the HUD) | `engine::canvas` on CoreGraphics / CoreText |
| HTML + CSS (HUD, overlays, blend modes, transitions, the canvas filter) | `game::dom` (the page's elements as data), `game::hud_draw` (CSS drawn on canvases), the composite pass (`engine/src/composite.metal`) |
| WebAudio | `engine::audio` (a WebAudio-compatible node graph on CoreAudio via cpal) |
| `requestAnimationFrame`, pointer lock, keyboard | `engine::app` (winit on AppKit, a CAMetalLayer, raw mouse deltas) |
| `localStorage` | a tab-separated file in `~/Library/Application Support/Backrooms/` |
| `Math.random` / `withSeed` | `game::rng` (bit-exact mulberry32 stack + a native generator) |

## Crates

* **`engine`** knows nothing about the game. Its API mirrors three.js so the
  game code reads like the JS: `Obj` (Object3D/Mesh/Group/lights/camera),
  `Geo` (BufferGeometry with r160-exact generators), `Mat` (Lambert / Phong /
  Basic / Points / Sprite / Line materials), `Tex` (canvas and data textures).
  Handles are `Rc<RefCell<..>>`; the renderer keeps GPU resources in caches
  keyed by id + version and sweeps them through weak references.
* **`game`** is backrooms.js: `Game` holds every JS global as a field (the JS
  global `fooBar` is `g.foo_bar`), and every JS function is a method in the
  module that owns its lines (`world.rs`, `lighting.rs`, `entities.rs`,
  `player.rs`, `hud.rs`, `flow.rs`, ...). The helper modules of `src/` are
  their own Rust modules (`archgen`, `scp`, `props`, `creatures`,
  `placement`, `instancing`, `stairwell`, `loop_mode`, `store`, `surfaces`,
  `soundscape`).
* **`placers/p00`–`p32`**: the ~650 prop placer functions (55k lines of JS),
  split into crates so they compile in parallel. Each registers its functions
  by JS name; `buildWorld` calls them through that registry.
* **`app`**: the `backrooms` executable (window, input routing, the render
  call, headless screenshot / level-dump modes).

## Rendering

`Renderer::render(scene, camera, globals, overlays, target)` does what
`WebGLRenderer.render` does for this game:

1. `projectObject`: visible-only traversal, frustum culling (bounding
   spheres, InstancedMesh spheres computed once like three's), opaque /
   transparent lists sorted the same way (painter's / reverse painter's,
   `renderOrder`, group order, material id, z).
2. Lights: ambient, hemisphere, point, spot, directional in three's
   physically-correct units (`decay`, `distance` cut-off, spot penumbra), with
   per-draw light lists.
3. Shading: MeshLambert (r160 is per-fragment), MeshPhong, MeshBasic, Points,
   Sprites, Lines — chunk for chunk, including FogExp2 applied after the
   sRGB output encode (as r160 does), vertex colours, alpha test, skinning,
   morph targets, instancing (matrices and colours).
4. The game's shader patches (`Patch` in `material.rs`): the place-based light
   map (`localLightPatch`), triplanar arch UVs, skin detail bump, the
   smiler's eyes, the watcher's sheet, the shadows' smoke edge, the
   stairwell face.
5. 4× MSAA into a scene texture at the quality governor's pixel ratio, then
   the composite pass: the canvas's CSS `filter`, then every HUD/CSS layer
   with its `mix-blend-mode` computed exactly (Apple GPUs read the framebuffer
   in the fragment shader).

The renderer was validated against three.js r160 in Chromium on a reference
scene (`tools/rendercheck`): mean difference 0.21/255, 0.44 % of pixels off by
more than 8/255 (edge antialiasing).

## Determinism

Levels are built under `with_seed(seedForLevel(runSeed, n))`, as in JS. Every
`Math.random()` is `rnd()` at the same place, so the same run seed builds the
same level. `canvasTex` and caches draw from the native generator (as the JS
does since its determinism fixes), so texture caching never shifts a level's
stream. Numbers are f64 like JS; data the JS keeps in typed arrays stays f32.
`Math.hypot`, `toFixed` and friends have V8-exact ports.

## Verification tools

* `cargo test -p game`: fixture tests generated from the original JS
  (archgen, scp, props, creatures, placement, instancing, stairwell, loop,
  store, audio, soundscape, player, ambient, ...). The `tools/*.mjs`
  dumpers run the JS modules under Node with the vendored three.js.
* Placer crates: each has smoke tests (`cargo test -p placers_p05`), and
  `node tools/placer-diff/run.mjs all --seeds 100` runs every placer in Rust
  and in the original JS (vendored three.js, same seeds) and compares the
  whole resulting world, the animators and the interactables
  (`placers/difftest`, `tools/placer-diff/README.md`).
* `app/tests/playthrough.rs`: the browser build's playthrough bot, ported. It
  plays whole runs from New Game with the real movement and collision and
  goes through every anomaly; for the same seed its per-level walking time
  equals the JS bot's.
* `tools/levelcheck`: whole-level comparison with the browser build. Run
  `node tools/levelcheck/js-capture.mjs --seed S --levels 1,2,3 --shots`
  (headless Chromium on the original page), then
  `backrooms --dump-level target/levelcheck --seed S --levels 1,2,3 --screenshot x`,
  then `node tools/levelcheck/compare.mjs --seed S`. It reports the first
  phase / placer whose seeded draw count differs, and compares tiles, zones,
  exit, key props, counts, entities and every placed object.
  `tone.mjs` compares two frames after box-averaging (so film grain and the
  per-session random wallpaper speckle cancel out); `js-menu.mjs` captures
  the start menu.
* `tools/rendercheck`: renderer-vs-three.js image comparison.

### What "the same" means here

The browser build itself randomises some things per page load with the
unseeded `Math.random` (wallpaper patterns and palettes, grime, panel
flicker, the level shown behind the start menu, menu flavour text), and the
native build does the same with its own generator. Everything a run seed
determines is identical. `Math.sin`/`cos`/`atan2`/... are not bit-identical
between the platform libm and V8 (V8 itself changed implementations between
versions: Node 25 and Chromium 153 disagree on a few percent of inputs), so
values computed from them can differ in the last bit; this never changed a
level (draw counts, placements) in any comparison.
