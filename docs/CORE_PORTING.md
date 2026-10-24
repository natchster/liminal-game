# Porting the core (backrooms.js minus the placers)

Read `PORTING.md` first. Every rule there still applies: same draws in
the same order, f64 for real values, the JS→Rust cheat sheet, the object
guard rules. This file covers what is specific to the core.

## The state: one `Game`, one field per JS global

`game/src/game.rs` defines `Game`. **The JS global `fooBar` is the field
`g.foo_bar`.** Each field's doc comment names its JS global, so
`grep -n '`fooBar`' game/src/game.rs` finds it.

A few subsystems keep their own structs:

| JS | Rust |
|---|---|
| textures section globals (`wallpaperVariants`, `carpetVariants`, `lightTex`, `dustSprite`, `staticTex`, ...) | `g.tex.*` (`game::textures::Textures`) |
| audio nodes (`droneGain`, `humGain`, `staminaBreath*`, `roomTone*`, ...) | `g.audio.*` (`game::audio_sys::AudioSys`) |
| `soundscape` | `g.soundscape()` (`Option<Rc<game::soundscape::Soundscape>>`); see `game/src/soundscape.rs` |
| `surfaces` | `g.surfaces` |
| `stairwell` / `loopRooms` | `g.stairwell` / `g.loop_rooms` |
| `layout`, `solids` (archgen) | `g.layout`, `g.solids` |
| `*Positions` lists | `g.pp.*` |
| `woodMat`, `paperMat`, ... | `g.mats.*` |
| DOM elements (`vignetteEl`, `msgEl`, `overlayEl`, ...) | `g.dom.*` (see below) |
| `player` | `g.player` (`state::Player`) |
| `keys[code]` | `g.key(code)` / `g.keys` |

Record-like JS objects shared between modules are structs in
`game/src/records.rs`: `Panel`, `RegionInfo`, `Pickup`, `Entity`,
`StoreState`, `HouseState`, `SafeRoom`, `BrokenElevator`, `VacuumBot`,
`LightSwitch`, `CandleState`, and so on. `Entity` types the common fields;
other per-entity state goes in its maps (`e.num["poseYaw"]`,
`e.flag["aimOk"]`, `e.vec[..]`, `e.obj[..]`). Use `e.n(k)` / `e.set_n(k, v)`.

Pure data tables (`LORE_ENTRIES`, `TEXT_POOLS`, `HUMAN_RIGS`, `SM_LEG`, ...)
are not fields. Define them as items (`const`, `static`, or a `fn`) in the
module that owns their JS lines; other modules reach them by path. The
per-weather tables of lines 69324–70002 are already generated in
`game/src/tables.rs`: `FOO_BY_WEATHER[key]` is `tables::foo_by_weather(key)`,
which returns `Option<T>`. Write the JS fallback explicitly:
`foo_by_weather(wk).or(foo_by_weather("normal")).unwrap()` for
`?? FOO.normal`, or `.unwrap_or(1.0)` for `?? 1.0`.

**If a JS global you need has no field** (it was missed), add it to `Game`
in `game.rs` and initialise it in `new_bare()`. Put it inside a block
marked `// ---- added by <your module>` and list it in your report.

## Audio API (already ported)

* `audioInit()` → `g.audio_init()`; `sfx.doorSlam()` → `g.sfx_door_slam()` (every `sfx.*` entry is `g.sfx_<snake>()`; ones with arguments: `sfx_step(run)`, `sfx_thump(v)`, `sfx_tick(v)`, `sfx_splash(loud)`, `sfx_scrape(loudness)`); by name: `g.sfx_by_name("doorSlam")`.
* `blip(...)` / `noiseBurst(...)` → `g.blip(...)` / `g.noise_burst(...)` with every argument explicit.
* `x.gain.value = v` → `g.audio.x.as_ref().map(|n| n.gain().set_value(v))`; reading `x.gain.value` → `n.gain().value()` (the last value set from the main thread). A biquad's `gain` is `filter_gain()`, `.Q` is `q()`.
* `soundscape.screech(...)` etc. → `if let Some(s) = g.soundscape() { s.screech(...) }` (see the method table in soundscape.rs).

## Functions: methods on `Game`, with a contract

Every top-level JS function becomes a `pub fn snake_name(&mut self, ...)`
(or `&self`) on `Game`, in its module's file. Pure helpers can be free
functions. The functions other modules call already exist as **stubs with
fixed signatures** in each module file: `world.rs`, `lighting.rs`,
`anomalies.rs`, `core_props.rs`, `entities.rs`, `ambient.rs`, `player.rs`,
`hud.rs`, `flow.rs`, `modes.rs`, `main_loop.rs`.

* In your own module: replace each stub body with the real port. Keep the
  signature. If it truly must change, change it and report it.
* Calling another module's function: use the stub's signature as it is. If
  you need a function another module will own but it has no stub yet, add a
  stub to *that module's file* inside `// ---- stubs added by <your module>`
  (body: `crate::stub!("jsName")` plus a default return), and report it.
* Conventions: numbers are `f64`, except tile coordinates (`i32`), indices
  (`usize`) and the level number (`i32`). Strings are `&str` in and
  `String` out. An entity parameter `e` is its index `ei: usize` into
  `g.entities`. BFS fields are `Rc<Vec<i32>>`. Optional JS arguments are
  explicit (`Option<T>`, or the JS default value written at the call site).
* `setTimeout(fn, ms)` → `g.set_timeout(ms, move |g| { ... })` (returns an
  id; `clearTimeout(id)` → `g.clear_timeout(id)`). `performance.now()` →
  `g.performance_now()`. `localStorage.getItem/setItem/removeItem` →
  `g.storage_get/set/remove`. `location.hash.includes('x')` → `g.hash_has("x")`.
* `AC.currentTime` → `g.ac_time()`, and `if (AC)` → `if g.ac.is_some()`.

## The DOM

`game/src/dom.rs` models every element of `backrooms.html`. Code that pokes
the page does the same to the model:

| JS | Rust |
|---|---|
| `vignetteEl.style.opacity = v` | `g.dom.vignette.set_opacity(v)` |
| `el.style.background = s` / `.boxShadow` / `.filter` / `.transform` / ... | `g.dom.x.set_style("background", s)` (any CSS property; camelCase or CSS names) |
| `el.textContent = s` / `el.innerHTML = s` | `g.dom.x.set_text(s)` / `g.dom.x.set_html(s)` |
| `el.classList.add/remove/toggle(c, on)` | `add_class` / `remove_class` / `toggle_class(c, on)` / `flip_class(c)` |
| `el.style.display = 'none'` | `g.dom.x.set_display("none")` |
| `document.body.classList` | `g.dom.body` |

The HUD module (`hud.rs` plus a new `hud_draw.rs`) owns drawing the model.
It ports `backrooms.css` and turns the DOM state into `dom::HudLayer`s
(CoreGraphics canvases plus fills and radial gradients) through
`hud_layers()`, and answers clicks through `hud_hit_test()`.

## Order of work at start-up and per frame

`main_loop.rs` already has the page's module load (`boot()`: textures, world
caches, anomalies, renderer, entities, ambient, player, hud, flow, then
`initLevel(1)`) and `frame()` (the JS `frame()`'s update order). Each
module implements its `init_*` and update functions. The prop animators run
inside `updateHud`, as in JS (`for (const fn of animators) fn(dt, perfT)`).

## Determinism

`initLevel` builds a level under `with_seed(seedForLevel(runSeed, n))`.
Everything it calls (genLayoutLevel, buildRegions, buildWorld, placeProps,
the placers, placeStore/placeHouse, settle passes, spawnEntities...) must
draw in exactly the JS order. Code that runs per frame draws from the native
generator, so its order matters less, but keep it faithful anyway.

## Building and testing

Work in your private workspace copy. The game crate's core is behind the
`wip` feature, which the app enables:

```
cargo build -p backrooms          # the whole game
cargo test -p game                # unit tests
./target/debug/backrooms --screenshot out.png --frames 120 --level 3 --play   # headless frame
```

Compile errors in other modules' stubs can't happen if you follow the
contract. If another module's stub panics or misbehaves at run time, work
around it in your tests and mention it.

## Already-ported subsystems (use them; don't re-port)

Read the module docs for full APIs. Integration notes from their authors:

### `game::archgen`, `game::scp` (src/archgen.js, src/scp.js — bit-exact)
* `generate_layout(&LayoutOpts{size, level, cell: CELL, scp}, &mut || rnd()) -> Layout`;
  `ceiling_heights(&layout) -> Vec<f32>` (= `tileCeil`); `build_solids(&layout,
  &SolidsOpts{anchors, ceil: Some(&tile_ceil), cell})` / `build_solids_view` over an
  external tile array; queries `circle_hits`, `point_in_solid`, `segment_clear`,
  `ray_axis`; `store_cells(zone)`; `archetype(key)`; `level_schedule`.
* In JS `tiles` *is* `layout.tiles` (themed() masks tiles with 2 and the layout
  sees it). In Rust `g.tiles` is the canonical array; build solids over it with
  `build_solids_view(&LayoutView{tiles: &g.tiles, ..})` or keep `layout.tiles` in sync.
* Tile units: `toU(w) = w/CELL + W/2`, `toV(w) = w/CELL + H/2`. `circleHitsWall`
  is `circle_hits(.., r/CELL, 1.8)`; `los` is `segment_clear(.., 1.5)`.
* `+x.toFixed(d)` → `archgen::to_fixed_num(x, d)`; string `toFixed` →
  `game::js::to_fixed` (now exact, V8-identical).
* `scp_for_level(run_seed, n)`, `scp_plan_default(run_seed)`.

### `game::textures`, `game::surfaces`
* `g.tex: Textures` — `wallpaper_variants` (86 × `{tex, grime_lvl, moss, palette_idx}`),
  `carpet_variants` (5, index 4 = moss), `ceiling_variants` (5), `light_tex`,
  `dust_sprite`, `face_tex`, `static_canvas`/`static_tex` (`g.tex.update_static()` = `drawStatic()`).
  `boot()` builds it; `Game::new_bare()` has 1×1 placeholders with the real counts.
* Free functions: `make_wallpaper`, `make_moss_wallpaper`, `handwritten_tex(text, &HandOpts)`,
  `text_tex(text, &TextTexOpts)`, `make_carpet`, `make_ceiling`, `draw_hand_stroke`,
  `strokes(c)`, `WALL_PALETTES`, `pick_soft_pattern`, ... (all `Default`-able option structs).
* `g.surfaces.wall(kind, g) / floor(kind, g) / ceil(kind, g) -> Option<Tex>`, `scale(kind)`.

### `game::props`, `game::creatures`, `game::instancing`, `game::placement`
* `pg(name, fallback)` already goes through `props::prop_part`. `buildWorld` must call
  `props::clear_prop_cache()` at the start of each level; `#noprops` →
  `props::set_props_enabled(false)`; `propStats` = `props::prop_stats()`.
* settlePlacement = `placement::resolve_placement(&PlacementOpts{objects, walls, ceil_at,
  in_pool, tol: DEFAULT_TOL})`. Objects: `world.children[from_idx..]` with `user.placer`,
  not `user.arch`, not lights. `protect = gameplayObjects().has(o) || (placer !=
  "placeProps" && !user.static_prop)`. Walls: `WallGrid::from_solids(W, H, CELL,
  &solids.boxes, solids.count).query(bb)`. `#noplace` skips it.
* After placement: freezeStatic, then (unless `#noinstance`)
  `instancing::instancify_static(&g.world, &|x, z| (g.tx(x) as f64 / 10.0).floor() as i64 * 64 + (g.tz(z) as f64 / 10.0).floor() as i64)`,
  then buildPropCull — before the world is added to the scene.
* `creature_geometry(name) -> Option<Geo>` (`cpart(name, fb)` = `.unwrap_or(fb)`),
  `with_cavity_shade(&geo, strength, center)`, `watcher_geometry() -> WatcherGeo{body, morphs, morph_color}`.

### `game::stairwell`, `game::loop_mode`, `game::store` (src/*.js — fixture-exact)
* `g.stairwell: Option<Stairwell>` — create in `boot()` (module load) with
  `Stairwell::new(StairDeps::game(face_geo))` where face_geo =
  `creature_geometry("stairFace").map(|g| with_cavity_shade(&g, 0.3, None))`; add `group` to the scene.
  `update(dt, &StairInput::with_camera(fwd, strafe, speed, yaw, pitch, &g.camera)) -> StairOut{x, y, z, seg, near_exit, events}`.
* `g.loop_rooms: Option<LoopMode>` — `LoopMode::new(LoopDeps::game(LoopTextures{..}))` on entry,
  `enter(lap)`, `update(dt, &LoopInput{..}) -> LoopOut{.., events, env, lights}`; on leave
  `scene.remove(&group); dispose()`.
* Store: `store_phase_default(t)`, `store_clock(&p)`, `product_name(rng)`, `room_set(theme, rng)`,
  `StoreKit::game(mat, tag)` with `build(name, a1, a2)`, `hide_spot(&obj)`.
* `stairwell::js_hypot2/js_hypot3/js_hypot` are V8's `Math.hypot`, bit-exact; use them
  for every `Math.hypot` in seeded code.

### `game::core_props` (placeProps)
Split in two: `core_props.rs` (57003–59035) and `core_props_tail.rs` (59036–61160).
Shared: `PropsCtx{cands, taken, exit_tile}` with `cx.take(filter)`, `has_wall_n(g, c)`,
and the builders `build_x(g, cx) -> Built{g, top, drawers}`.

### `player.rs` (ported) — record-key conventions other modules must match
* `VacuumBot`: `yaw`→`dir`, `pickTimer`→`timer`, `followT`→`follow_t`; `speed` (default 0.45),
  `spinPhase`, `chaseT`, `stunT`, `beepTimer` → `f["<jsName>"]`; `homeBtn` → `objs["homeBtn"]`.
  The vacuum's E-tap interactable must write these same keys. JS interactables that hold
  `g.position` by reference (prompt follows a moving object) must keep `Interactable.pos` updated.
* `BrokenElevator`: `f["lingerT"]`, `b["descentFired"]`.
* `LightSwitch`: `x`/`z` are world coords (`wx`/`wz`), `panels` = `panelIndices`,
  `f["motionCooldown"]`, `f["armT"]`, `b["wasInRange"]`. `flipSwitch` is
  `g.flip_light_switch(i, cause)` (stub in core_props_tail.rs, owned by placeProps' tail).
* Lore notes: `whisperCooldown`, `wasNearRead` live in the note object's user data.
* Prop animators run inside `updateHud` (JS line 68211), not updatePlayer.
