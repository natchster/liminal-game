# Architecture

## Crates

```
engine  ←  game  ←  placers_p00 … placers_p32
   ↖        ↖            ↙
        backrooms (the executable)            xtask (automation)
```

* **`engine`** knows nothing about the game. It provides:
  * `math`: vectors, quaternions, Euler angles, matrices, colours (linear
    RGB with sRGB conversion), boxes and spheres;
  * `scene`: the scene graph. `Obj` is a shared handle to a node (group,
    mesh, instanced or skinned mesh, points, line, sprite, light, camera,
    bone) with position / rotation / quaternion / scale, children, user data
    and world matrices;
  * `geometry`: `Geo` buffers and generators (box, plane, circle, cylinder,
    cone, sphere, torus, ring, lathe, capsule, polyhedra, extruded shapes via
    `earcut`), normals, bounding volumes, morph targets, skin weights;
  * `material` / `texture`: Lambert, Phong, unlit, points, sprite and line
    materials with blending, sides, alpha test and the game's shader patches;
    image and canvas textures with wrapping, repeat, offset, filtering;
  * `canvas` (on `cg`, CoreGraphics / CoreText): a 2D drawing API with paths,
    arcs, gradients, text, shadows, compositing and pixel access, used for
    procedural textures and the HUD; `css` parses colour and font strings;
  * `audio`: an audio node graph (gain, oscillator, biquad, buffer source,
    stereo and 3D panners, wave shaper, convolver, delay, compressor,
    constant source) with sample-accurate parameter automation, rendered on
    CoreAudio through cpal, or offline on demand
    (`AudioContext::new_offline`, `render_offline`; `wav` writes the result);
  * `render`: the Metal renderer (below); `app`: the winit window, the
    CAMetalLayer, raw mouse input and pointer capture; `png`: PNG encoding.
* **`game`** is the game itself, with no platform code: it runs headless,
  which is how every test drives it.
* **`placers_pNN`** hold the ~650 decoration placers (`place_*` functions
  that add props to a level). They are plain functions over `&mut Game`,
  registered by name, and split across crates so they compile in parallel.
* **`backrooms`** is the executable: it registers the placers, runs the
  window loop, turns the game's state into render calls, and has the
  headless modes (`--screenshot`, `--dump-level`) and the whole-game tests.

## The game crate

| Module | Responsibility |
|---|---|
| `base` | `Game` (all state in one struct), constants (`config`), the random streams (`rng`), numeric helpers, shared record types, per-weather tables, the prop position registry |
| `level` | `archgen`: the zoned layout of a level (archetypes, walls, partitions, rooms, ceiling heights, solids for collision and sight); `scp`: which anomaly appears on which level of a run |
| `world` | Grid and collision queries, zone purposes (`purpose`), regions, and the world build (`build`): floors, panels, core props, the themed placer passes, settling, placement, walls and ceilings, freezing static objects, instancing, culling |
| `assets` | The embedded prop and creature meshes (`assets/*.bin`), fitted to each placer's size |
| `placers` | The kit placers build with (`canvas_tex`, `pg`, joints, shared materials and tables) |
| `core_props` | The props every level needs: the exit, pickups, notes, switches, the pool, the vacuum, the safe room, elevators, holes, barricades, weather extras |
| `entities` | The creatures: rigs, skinning, animation, AI, voices, and the pacing director (`director`) |
| `anomalies` | The Showroom (a store that closes at night), the Borrowed House, the human rig they share |
| `modes` | The Stairwell and the Loop: self-contained spaces with their own movement, entered from a level |
| `player` | Movement, stamina, sanity, flashlight, items and actions |
| `audio` | Sound effects and persistent beds (`sys`), the spatial soundscape, the ambient event driver |
| `visuals` | Lighting (the place-based light map, pooled lights, fog), procedural textures and surfaces, the props' material pass (`prop_look`: muted colours, real highlights instead of glowing "shine", wear), the low-sanity colour filter |
| `ui` | The HUD and menus: an element model with CSS-like styles and classes (`model`), the HUD logic (`hud`), the journal's lore (`lore`), and the painter that turns elements into composited layers (`draw`) |
| `session` | Start-up (`boot`), the per-frame update, input handling, and the game flow: levels, saves, death, the menu |
| `debug` | Level dumps for tools and tests, and test helpers |

### One frame

`backrooms` calls `Game::frame(dt)` (`session::frame`), which runs the
scheduled callbacks and then the update in a fixed order: player, entities,
the pacing director, anomaly, prop culling, lights, HUD (which also runs the
prop animators), dust, film grain, reality glitches, the flicker wave, ghost
footsteps, guide arrows, the vacuum and the broken elevator, sanity effects,
ambient sound and the soundscape. Inside the Stairwell or the Loop the mode's update replaces
most of that. The app then asks for the render globals (background, fog, the
light map, the colour filter) and the HUD layers, and renders.

### Building a level

`init_level(n)` (`session::flow`) draws everything a level is made of from
one random stream seeded with `seed_for_level(run_seed, n)`:

1. `generate_layout` (`level::archgen`): zones, walls, rooms, ceiling
   heights (neighbouring zones of one archetype share theirs).
2. `build_world` (`world::build`): zone purposes (`world::purpose`: zones of
   one archetype that open into each other form an *area*, and each area is
   given one purpose — a primary prop theme weighted by what its archetype
   welcomes, sometimes a compatible second theme, whether the building's own
   fittings are there, or nothing at all), regions and their lighting and
   grime (one per area, a big lone zone split in two along an inner wall, so
   wallpaper, grime and darkness change at walls between places), floor and
   ceiling panels, the core props (each kind in the zones it belongs in,
   some in rows), the store or house if this is their level, the themed
   placer passes in a fixed order (each placer only sees the zones whose
   purpose welcomes its theme), settling props on what is below them,
   resolving overlaps (`world::placement`), walls (a seam wall between two
   regions drawn in two halves, one per side) and ceilings, then freezing
   static objects, merging repeated meshes into instanced meshes
   (`world::instancing`) and building the cull list.
3. Spawning the creatures.

The build records a checkpoint (label + draws so far) after every phase and
placer pass when tracing is on; the reference-level test compares these.

## Creatures and pacing

Each creature (`entities::ai`) is in one of three states:

* **wander**: it roams toward dark goals far from you. Its eyes look
  forward: it sees you only inside a wide cone ahead of it (or right beside
  it, or when your flashlight is aimed at it), so you can pass behind one.
  Seeing you starts a hunt; only *hearing* you (`player.audible_range`:
  silent standing, quiet crouching, loud running) starts a search.
* **search**: it walks to where it heard you, stops and listens, then tries
  spots ever closer around that point, quiet but for its footfalls, until
  it sees you or gives up.
* **hunt**: the notice comes first (it stops dead, its head snapped to you,
  a shriek from where it stands, a dissonant sting), then a charge. Losing
  sight and sound of you for a few seconds sends it to search where it last
  had you. Touch is death; the death scene shows the creature that did it.

Above them, the **director** (`entities::director`) shapes each level into a
cycle, after the AI director designs of horror games (a menace gauge that
backs the threat off after a peak; build-up, peak and relax phases):

| Phase | What happens |
|---|---|
| Calm | Nothing is sent after you (45–80 s on the first floors, shorter deeper down) |
| Build | One creature is sent (the nearest Smiler or Hound, now and then the Watcher; not the one sent last time when another can go). It is steered toward your part of the level, to tiles you cannot see in a circle round you that tightens as the build-up goes on (16 m out at first, 3–8 m at the end). Its footsteps far off, then the lights over it stuttering, then quiet as it closes in. If the time runs out it is told where you are (and again every few seconds) and gets a last stretch of up to 25 s to come and look |
| Peak | Something hunts you, or searches close by. The menace gauge fills with closeness and sight; once it is full and you have broken away (nothing hunting you within 8 m or in sight) for a few seconds, or it is half full again and the hunter is not on top of you, everything after you withdraws |
| Relax | The creatures keep away and hear less for 25–45 s |

The Watcher keeps the rhythm too: unless it is the one sent, it stays still
within 18 m of you and only creeps slowly when further off, never teleports,
and drains half as much. Sent, it creeps and closes the gap while you look
away; its touch (a blow to your nerve) is that build-up's climax, after
which it falls back. Its touch kills only when your nerve is already gone.

The director publishes `tension` (how close the threat feels: the drone, the
heartbeat and the vignette follow it) and `hush` (how quiet the ambience
should be), and plays the sting the first time you lay eyes on each kind of
creature on a level. It runs on the native random stream, so it never
affects a level's build.

## Determinism

* Everything a level is made of comes from the seeded stream (`rnd()`,
  `rand1`, `rand2`, `irand`, `choice` in `base::rng`) inside `with_seed`.
* Per-frame randomness (flicker, AI wobble, sound variation) and anything
  cached across levels (textures, surfaces, the canvas inside `canvas_tex`)
  draw from the native stream, so they can never shift a level's draws.
* Numbers are `f64`; buffers the GPU reads (positions, instance data) are
  `f32`. Helpers whose exact result matters are in `base::numeric`
  (round-half-up, NaN-propagating min / max, a compensated hypotenuse, exact
  fixed-point formatting).

## Rendering

`Renderer::render(scene, camera, globals, overlays, target)`:

1. **Collect:** a visible-only traversal with frustum culling (bounding
   spheres; instanced meshes use the union of their instances' spheres),
   opaque and transparent lists sorted by render order, depth and material.
2. **Lights:** ambient, hemisphere, point, spot and directional lights with
   physical falloff and spot penumbra; each draw gets the lights that reach
   it.
3. **Shading:** one Metal uber-shader specialised with function constants
   (material kind, maps, vertex colours, instancing, skinning, morph
   targets, fog, the light map, patches, wear). Lambert and Phong are per
   fragment; lit surfaces lose indirect light toward the floor and roll
   their highlights off with a soft shoulder; materials with `wear` (the
   props, see `visuals::prop_look`) get value-noise mottling and grime near
   the floor. Fog is exponential-squared and applied after the sRGB encode.
4. **Patches** (`material::Patch`): the place-based light map, triplanar
   architecture UVs, skin detail, the smiler's eyes, the watcher's sheet,
   the shadows' smoke edge, the stairwell face.
5. **Composite:** 4× MSAA into a scene texture at the quality governor's
   resolution, then the camera grade (a little less saturation, a gentle
   S-curve, the lens vignette), then the colour filter, then each HUD layer
   with its blend mode computed exactly in the fragment shader (Apple GPUs
   read the framebuffer there).

GPU resources are cached by id and version and released through weak
references when the scene drops them.

## UI

The HUD is an element model (`ui::model::UiModel`): text, classes, inline
styles in CSS syntax (`background`, `opacity`, `filter`, `transform`, …) and
display state, changed by game code exactly like any other state. `ui::draw`
lays the elements out, paints each one into its own cached canvas (repainted
only when its content changes), runs style transitions and keyframe
animations, and hands the app a list of `HudLayer`s (canvases, fills and
radial gradients with blend modes) for the compositor. Clicks are answered
by `hud_hit_test`.

## Audio

`game::audio::sys::AudioSys` owns the persistent beds (drone, hum, room
tone, stamina breath, rain) and every sound effect (`sfx_*`), all
synthesized. `audio::soundscape` handles spatial voices, zone reverbs and
footsteps; `audio::ambient` schedules the ambient events of each weather,
plays each prop sound from the nearest prop in earshot (level by distance,
pan by direction, muffled without line of sight), gives every level a few
signature sounds anchored to fixed spots on rhythms of their own (stable
per run seed and level), and starts every cadence part-way through its
interval so a level never opens with everything at once. A signature carries
as far as it would: a slam, a scream or footsteps from 20–55 m, a phone or a
radio from the next rooms, a drip or paper only from close by; each lands
some 6–14 dB over the room's bed (measured by `tests/listen.rs`). The
weather's own sounds come from places too: each level puts a draught (the
wind, the echoing whisper), a duct (the clatter), a dying fan (the grinder)
and an occupied room (whispers, steps) somewhere of its own, and the
electric crackle and surges come from the nearest light fitting. The rare
events (a knock, a door, a voice, a reality shift…) belong to the run: their
countdowns carry from level to level. The pacing director sets
`Game::amb_hush` to quiet all of it while something comes: the one-shots
soften and hold, the room tone and the lights' own hum sink away, and the
reality glitch and the flicker wave wait. The engine renders the graph in
128-frame quanta on CoreAudio's thread; the game talks to it through a
command queue.

## Saves

`Game::storage_get / storage_set` keep a small key-value store in
`~/Library/Application Support/Backrooms/storage.txt`; the run (seed, level,
items, lore found, medals) is saved on every level change.
