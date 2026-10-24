# Porting guide: backrooms.js → Rust

This is the contract for translating the JavaScript game into the native
crates. It is written for the prop placers (the ~650 `placeXxx(dist)`
functions) but the same rules hold for the core.

## Layout

| Crate | What it is |
|---|---|
| `engine/` | three.js-compatible scene graph (`Obj`, `Geo`, `Mat`, `Tex`), r160-exact geometry generators, the HTML canvas on CoreGraphics (`Canvas`), a WebAudio-style graph (`AudioContext`), the Metal renderer. |
| `game/` | The game: `Game` (every JS global), RNG (`rnd()` = `Math.random()`), helpers (`canvas_tex`, `pg`, ...), shared tables (`game::shared`), the core systems. |
| `placers/pNN/` | Prop placers, ported chunk by chunk from backrooms.js. Each crate exports `register(&mut Vec<(&'static str, PlacerFn)>)`. |
| `app/` | The `backrooms` executable. |

Placer crates depend on `engine` and `game` only, and **must not edit them**.
If something is missing, write a local helper in your own crate. If that is
not possible, leave a `// TODO(port): ...` comment, keep the rest working,
and say so in your report. Never silently drop behaviour.

Start every placer crate with:

```rust
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use game::prelude::*;
```

## The three rules

1. **Same draws, same order.** Levels are built under a seeded
   `Math.random` and must come out identical to the JS build. So every
   `Math.random()` becomes exactly one `rnd()`, in the same order and under
   the same conditions. Short-circuit `&&`/`||`, ternaries, early `return`s,
   loop bounds and argument order all decide which draws happen. Keep them
   exactly. Do not hoist, cache, reorder or "simplify" random draws. (Rust
   evaluates function arguments left to right, like JS.)
2. **Same numbers.** All real-valued math is `f64` (JS numbers). Copy
   constants literally. `Math.floor(x)` → `x.floor()`; cast with `as usize`
   / `as i32` only where JS uses the value as an index or a tile coordinate.
   `Math.round` → `js_round(x)`. `Math.sign` → `js_sign(x)`. `x | 0` →
   `to_i32(x)`.
3. **Everything ported.** Every placer, every mesh, every animator, every
   texture, every interactable. Keep a `// line NNNN` comment above each
   function with its line in backrooms.js. Keep short comments where they
   explain intent.

## JS → Rust cheat sheet

### Globals and grid

| JS | Rust |
|---|---|
| `function placeFoo(dist)` | `fn place_foo(g: &mut Game, dist: &[i32])`, registered as `r.push(("placeFoo", place_foo))` |
| `W`, `H` | `g.w`, `g.h` (`i32`) |
| `tiles[tIdx(x, z)]` | `g.tiles[g.t_idx(x, z)]` (`Vec<u8>`; 0 open, 1 wall, 2 hidden) |
| `tIdx(x, z)` / `inGrid(x, z)` / `isWallTile(x, z)` | `g.t_idx(x, z)` / `g.in_grid(x, z)` / `g.is_wall_tile(x, z)` (any numeric args) |
| `wx(t)` / `wz(t)` / `tx(w)` / `tz(w)` | `g.wx(t)` / `g.wz(t)` / `g.tx(w)` / `g.tz(w)` |
| `dist[tIdx(x, z)]` | `dist[g.t_idx(x, z)]` (`i32`, -1 unreachable) |
| `CELL`, `WALL_H`, `EYE`, `SAN_MAX`, `STAM_MAX`, `BAT_MAX` | same names (in the prelude) |
| `world.add(o)` | `g.world.add(&o)` |
| `levelWeather?.key` | `g.weather_key()` (`&'static str`, `"normal"` by default) |
| `perfT` | `g.perf_t` |
| `currentSeed` | `g.current_seed` |
| `player.sanity`, `.bottles`, `.batteries`, `.pos`, `.yaw`, `.flashlightOn`, `.speedBoostUntil`, ... | `g.player.sanity`, `.bottles` (`i32`), `.batteries` (`i32`), `.pos`, `.yaw`, `.flashlight_on`, `.speed_boost_until`, ... |
| `woodMat`, `paperMat`, `glassMat`, `metalMat`, `darkMat`, `chromeMat`, `rustMat`, `cardboardMat`, `plasticRedMat`, ... | `g.mats.wood`, `g.mats.paper`, `g.mats.glass`, `g.mats.metal`, `g.mats.dark`, `g.mats.chrome`, `g.mats.rust`, `g.mats.cardboard`, `g.mats.plastic_red`, ... (clone or borrow: `&g.mats.wood`) |
| `dustMat` / `flashlight` (the player's lamp) | `g.dust_mat` / `g.flashlight` |
| `fooBarPositions.push({ x, y, z })` | `g.pp.foo_bar_positions.push(v3(x, y, z))` (`y` = 0 when absent). Name = snake_case of the JS name, e.g. `cuckooClockPositionsR741` → `cuckoo_clock_positions_r741` |
| `showMsg(text, dur)` | `g.show_msg(text, dur)` |
| `blip(f, d, type='sine', gain=0.2, slideTo=null, delay=0, pan=0)` | `g.blip(f, d, "sine", 0.2, None, 0.0, 0.0)`: pass every argument and fill in the JS defaults |
| `noiseBurst(d, f, q, gain, type='bandpass', slideTo=null, delay=0, pan=0)` | `g.noise_burst(d, f, q, gain, "bandpass", None, 0.0, 0.0)` |
| `if (AC)` / `AC.currentTime` | `if g.ac.is_some()` / `g.ac_time()` |
| `clamp`, `lerp` | `clamp(v, a, b)`, `lerp(a, b, t)` (f64) |
| shared tables (`FLOOR_STAIN_BY_WEATHER[key] ?? ...normal`, `CLOCK_TEXTURES`, ...) | `game::shared::*` (marked `[SHARED]` in your chunk) |

### Random

| JS | Rust |
|---|---|
| `Math.random()` | `rnd()` |
| `rand(a)` / `rand(a, b)` | `rand1(a)` / `rand2(a, b)` |
| `irand(a, b)` | `irand(a, b)` (`i64`, inclusive) |
| `choice(arr)` | `choice(&arr)` |
| `arr[Math.floor(Math.random() * arr.length)]` | `arr[(rnd() * arr.len() as f64).floor() as usize]` |
| `mulberry32(seed)` | `Mulberry32::new(seed)`, then `.next()` |

### three.js objects

Handles (`Obj`, `Geo`, `Mat`, `Tex`) are `Rc` handles. `.clone()` copies
the **handle**, which is a JS reference assignment. three's `x.clone()` (a
new object) is `.duplicate()`.

| JS | Rust |
|---|---|
| `new THREE.Group()` / `new THREE.Object3D()` | `group()` / `object3d()` |
| `new THREE.Mesh(geo, mat)` | `mesh(geo, &mat)` or `mesh(geo, mat)`; an array of materials: `mesh(geo, vec![a, b])` |
| `new THREE.BoxGeometry(w, h, d)` | `box_geo!(w, h, d)`. Every primitive is a macro with three's defaults for missing args: `cyl_geo!`, `cone_geo!`, `sphere_geo!`, `plane_geo!`, `circle_geo!`, `torus_geo!`, `capsule_geo!`, `octa_geo!`, `icosa_geo!`, `dodeca_geo!`, `tetra_geo!`, `ring_geo!`, `lathe_geo!(points_vec, segs)`, `shape_geo!(shape, segs)` |
| `PG('Name__part', geo)` / `PG(name, geo, true)` | `pg("Name__part", geo)` / `pg_own(name, geo)` |
| `new THREE.MeshLambertMaterial({ color: 0x8a6a40, transparent: true })` | `lambert!({ color: 0x8a6a40, transparent: true })`: same keys, same values. Also `basic_mat!`, `phong!`, `standard_mat!`, `sprite_mat!`, `points_mat!`, `line_mat!`. Keys: `color emissive emissiveIntensity specular shininess map alphaMap bumpMap bumpScale opacity transparent alphaTest side depthWrite depthTest blending vertexColors fog polygonOffset polygonOffsetFactor polygonOffsetUnits size sizeAttenuation rotation visible` (`roughness`/`metalness` accepted and ignored) |
| colour values | hex int (`0xff0000`), `Color`, or a CSS string |
| `THREE.DoubleSide`, `THREE.BackSide`, `THREE.AdditiveBlending`, `THREE.NormalBlending` | `DoubleSide`, `BackSide`, `AdditiveBlending`, `NormalBlending` |
| `new THREE.PointLight(c, i, d, decay)` | `point_light!(c, i, d, decay)` (missing args = three's defaults) |
| `new THREE.SpotLight(c, i, d, angle, penumbra, decay)` | `spot_light!(...)`; `.target` → `l.target()` |
| `new THREE.Sprite(mat)` / `new THREE.Points(geo, mat)` / `new THREE.Line(geo, mat)` | `sprite(&mat)` / `points(geo, &mat)` / `line(geo, &mat)` |
| `new THREE.InstancedMesh(geo, mat, n)`; `setMatrixAt(i, m)`; `instanceMatrix.needsUpdate = true` | `instanced_mesh(geo, &mat, n)`; `o.set_matrix_at(i, &m)`; (automatic) |
| `new THREE.Vector3(x, y, z)` | `v3(x, y, z)` (a `Copy` value, methods mutate in place like three: `v.normalize()`, `v.multiply_scalar(s)`, `v.add(&o)`, `v.sub(&o)`, `v.distance_to(&o)`, `v.length()`, `v.apply_axis_angle(&axis, a)`); `v.clone()` → `v` |
| `new THREE.Vector2(x, y)` | `v2(x, y)` |
| `new THREE.Color(0xff8800)` | `Color::hex(0xff8800)`; `.setHex(h)` → `.set_hex(h)`, `.setHSL(h,s,l)` → `.set_hsl(h,s,l)`, `.lerp(&c, t)`, `.multiplyScalar(s)` |
| `new THREE.Euler(x, y, z)` / `Quaternion` / `Matrix4` | `Euler::new(x, y, z)` / `Quat` / `Mat4` with three's method names in snake_case |
| `new THREE.Shape()`; `shape.moveTo(...)`; `lineTo`; `quadraticCurveTo`; `absarc(...)` | `let mut s = Shape::new();` `s.move_to(..)`; `s.line_to(..)`; `s.quadratic_curve_to(..)`; `s.absarc(x, y, r, a0, a1, cw)` |

### Object3D properties

The accessors return guards. **Never hold a guard and access the same
object again in the same statement**, or the RefCell panics:
`o.position().y = o.position().y + 1.0` panics. Write
`o.position().y += 1.0`, or read with the copy getters (`pos()`, `rot()`,
`scl()`, `quat()`), which are always safe: `o.position().y = o.pos().y * 2.0`.

| JS | Rust |
|---|---|
| `o.position.set(x, y, z)` | `o.position().set(x, y, z)` (numeric args of any type) |
| `o.position.y = v` / `+= v` | `o.position().y = v` / `+= v` |
| read `o.position.x` / the vector | `o.pos().x` / `o.pos()` |
| `o.position.copy(v)` | `o.position().copy(&v)` |
| `o.rotation.y = a`, `o.rotation.set(x, y, z)` | `o.rotation().y = a`, `o.rotation().set(x, y, z)` |
| read `o.rotation.y` | `o.rot().y` |
| `o.rotation.order = 'YXZ'` | `o.set_rotation_order(EulerOrder::YXZ)` |
| `o.scale.set(..)` / `setScalar(s)` / `o.scale.y = v` | `o.scale().set(..)` / `o.scale().set_scalar(s)` / `o.scale().y = v` |
| `o.quaternion.setFromAxisAngle(axis, a)` | `o.quaternion().set_from_axis_angle(&axis, a)` |
| `o.lookAt(x, y, z)` | `o.look_at(x, y, z)` |
| `o.rotateX(a)` / `translateZ(d)` | `o.rotate_x(a)` / `o.translate_z(d)` |
| `o.visible = b` / read | `o.set_visible(b)` / `o.visible()` |
| `o.renderOrder = n` / `o.frustumCulled = b` / `o.name = s` | `o.set_render_order(n)` / `o.set_frustum_culled(b)` / `o.set_name(s)` |
| `o.userData.phase = v` / read | `o.user().phase = v` / `o.r().user.phase`. Typed fields: `placer static_prop floats gameplay no_instance hide phase base_x base_y base_z w`; anything else: `o.user().set("key", v)` / `o.r().user.get("key")` |
| `o.add(c)` / `o.remove(c)` / `o.children` | `o.add(&c)` / `o.remove(&c)` / `o.children()` |
| `o.traverse((c) => ...)` | `o.traverse(&mut \|c\| { ... })` |
| `o.material` / `o.material = m` | `o.mat()` / `o.set_mat(&m)` |
| `o.geometry` / `o.geometry = g` | `o.geo()` / `o.set_geo(&g)` |
| `o.getWorldPosition(v)` | `o.world_position()` |
| `light.intensity = v` / read | `l.set_intensity(v)` / `l.intensity()` |
| `light.color.setHex(h)`, `light.distance = d` | `l.light().color.set_hex(h)`, `l.light().distance = d` |

### Materials and textures at run time

| JS | Rust |
|---|---|
| `mat.opacity = v` | `mat.m().opacity = v` (or `mat.set_opacity(v)`) |
| read `mat.opacity` | `mat.opacity_v()` |
| `mat.color.setHex(h)` / `mat.emissive.setHex(h)` | `mat.m().color.set_hex(h)` / `mat.m().emissive.set_hex(h)` |
| `mat.emissiveIntensity = v` | `mat.m().emissive_intensity = v` |
| `mat.needsUpdate = true` | `mat.needs_update()` |
| `mat.map.offset.y += d` | `if let Some(t) = mat.map_v() { t.t().offset.y += d; }` |
| `tex.repeat.set(a, b)` / `tex.wrapS = tex.wrapT = THREE.RepeatWrapping` | `tex.t().repeat.set(a, b)` / `tex.set_repeat_wrapping()` |
| `tex.needsUpdate = true` | `tex.needs_update()` |

### Geometry

| JS | Rust |
|---|---|
| `geo.translate(x, y, z)`, `rotateX(a)`, `scale(x, y, z)` | `geo.translate(x, y, z)`, `geo.rotate_x(a)`, `geo.scale(x, y, z)` (chainable) |
| `geo.computeVertexNormals()` / `computeBoundingBox()` | `geo.compute_vertex_normals()` / `geo.compute_bounding_box()` |
| `geo.parameters.radius` | `geo.params().radius()` (also `width()`, `height()`, `depth()`) |
| `new THREE.BufferGeometry()` + `setAttribute('position', new THREE.BufferAttribute(new Float32Array(a), 3))` + `setIndex(idx)` | `let geo = Geo::new(); geo.set_position(a_f32); geo.set_index(idx_u32);` also `set_normal`, `set_uv`, `set_color(v, 3)` |
| `geo.attributes.position.array[i] = v; ...needsUpdate = true` | `{ let mut gg = geo.g(); gg.position[i] = v; gg.touch(); }` |
| `geo.attributes.position.count` | `geo.vertex_count()` |

### Canvas textures

| JS | Rust |
|---|---|
| `canvasTex(w, h, (ctx, w, h) => {...})` | `canvas_tex(w, h, \|ctx, w, h\| {...})` (`w`, `h` are `f64` inside) |
| `canvasTex(w, h, draw, false)` | `canvas_tex_opt(w, h, false, \|ctx, w, h\| {...})` |
| module-level `const T = canvasTex(...)` | `game::lazy_static_tex!(t_tex, canvas_tex(...));` then call `t_tex()` (built on first use, on the native RNG) |
| module-level array of textures | a fn with a `thread_local! { static V: OnceCell<Vec<Tex>> }` (see `game/src/shared.rs` `clock_textures`) |
| `ctx.fillStyle = s` / `strokeStyle` | `ctx.set_fill_style(s)` (`&str`, `&String`, `&Gradient`) / `ctx.set_stroke_style(s)` |
| `ctx.lineWidth`, `font`, `textAlign`, `textBaseline`, `globalAlpha`, `globalCompositeOperation`, `lineCap`, `lineJoin`, `shadowBlur`, `shadowColor` | `ctx.set_line_width(v)`, `set_font(s)`, `set_text_align(s)`, `set_text_baseline(s)`, `set_global_alpha(v)`, `set_global_composite_operation(s)`, `set_line_cap(s)`, `set_line_join(s)`, `set_shadow_blur(v)`, `set_shadow_color(s)` |
| `beginPath moveTo lineTo quadraticCurveTo bezierCurveTo closePath rect fill stroke clip` | `begin_path move_to line_to quadratic_curve_to bezier_curve_to close_path rect fill stroke clip` |
| `ctx.arc(x, y, r, a0, a1)` / with `anticlockwise` | `ctx.arc(x, y, r, a0, a1)` / `ctx.arc_dir(x, y, r, a0, a1, ccw)` |
| `ctx.ellipse(x, y, rx, ry, rot, a0, a1)` / with `ccw` | `ctx.ellipse(...)` / `ctx.ellipse_dir(..., ccw)` |
| `ctx.fill('evenodd')` | `ctx.fill_evenodd()` |
| `fillRect strokeRect clearRect` | `fill_rect stroke_rect clear_rect` |
| `ctx.fillText(t, x, y)` / with `maxWidth` / `strokeText` | `ctx.fill_text(&t, x, y)` / `fill_text_max(&t, x, y, mw)` / `stroke_text(&t, x, y)` |
| `ctx.measureText(t).width` | `ctx.measure_text(&t).width` |
| `save restore translate rotate scale setTransform` | `save restore translate rotate scale set_transform` |
| `const gr = ctx.createLinearGradient(...)`; `gr.addColorStop(o, c)`; `ctx.fillStyle = gr` | `let gr = ctx.create_linear_gradient(...)`; `gr.add_color_stop(o, c)`; `ctx.set_fill_style(&gr)` (also `create_radial_gradient`) |
| `ctx.setLineDash([4, 2])` | `ctx.set_line_dash(&[4.0, 2.0])` |
| `getImageData` / `putImageData` / `createImageData` | same, snake_case (`ImageData { width, height, data }`) |
| `` `rgba(20,10,5,${a})` `` | `&format!("rgba(20,10,5,{})", a)` (Rust prints f64 the way JS does) |

Canvas methods take any numeric type, so `ctx.fill_rect(0, 0, w, h)` works.
Plain Rust arithmetic does not: write `w / 2.0`, not `w / 2`.

### Closures (animators, interactables)

```rust
// animators.push((dt, t) => { ember.scale.set(f, f, f); ... });
let ember2 = ember.clone();               // handles you still need afterwards
g.animate(move |g, dt, t| { ember2.scale().set(f, f, f); });

// interactables.push({ pos, r: 1.4, prompt: '[E] OPEN', enabled: () => !opened, use: () => {...} });
let opened = Rc::new(Cell::new(false));
let o2 = opened.clone();
g.interact(pos, 1.4, "[E] OPEN", move |_g| !o2.get(), move |g| { opened.set(true); g.show_msg("...", 2.0); });
```

* Closures are `'static`: they own what they capture (`move`). Clone `Obj`/`Mat`/`Tex` handles first if you need them later. They are cheap.
* State shared between closures, or between a closure and later code, uses `Rc<Cell<T>>` / `Rc<RefCell<T>>`.
* Inside a closure use its own `g` parameter for game state, never an outer `g`.
* An interactable's `prompt` is a fixed `String`. If JS changes it later, keep the closures' state in an `Rc` and note it.

### Things to skip

* `typeof window !== 'undefined' && (window.x = ...)` and other devtools exposure.
* `try { ... } catch (e) {}`: keep the body.
* `geometry.dispose()` / `material.dispose()`: call `.dispose()` (no-op) or drop the line.

## Types and idioms

* Tile coordinates: `i32`. Lists of tiles: `Vec<(i32, i32)>` or small structs.
* JS objects used as records: a local `struct` (define it inside the function).
* `arr.push(x)` → `vec.push(x)`, `arr.length` → `vec.len()`; `for (const x of arr)` → `for x in &v` / `for x in v.iter()`.
* `arr.sort((a, b) => a.k - b.k)` → `v.sort_by(|a, b| a.k.partial_cmp(&b.k).unwrap())`. Both sorts are stable.
* `Math.max(a, b)` on f64 → `a.max(b)`; on ints → `a.max(b)` with integer types.
* Destructuring: `const { x, z } = t` → `let (x, z) = t;` or field access.
* `x ?? y` / `x || y` on optional values → `Option` / `unwrap_or`.

## Testing your crate

Each crate's `src/lib.rs` ends with:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }
}
```

`smoke_placers` runs every placer many times on synthetic levels of several
sizes and weathers, then runs the animators and uses every interactable.
Run it with your own target directory, so agents don't block each other:

```
cd native && CARGO_TARGET_DIR=<your dir> cargo test -p placers_pNN
```

It must pass with no panics, and `cargo build` must have no errors. Warnings
about unused things are allowed.
