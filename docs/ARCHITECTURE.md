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
    procedural textures and the HUD, and JPEG decoding (`from_jpeg`) for the
    photographed ones; `css` parses colour and font strings;
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
| `level` | `archgen`: the zoned layout of a level (archetypes, walls, partitions, rooms, ceiling heights, solids for collision and sight; the ground a tile stands on where floors have height: a floor, a void, a stair, a mass); `building`: the Building's floors (levels 1–3): a lattice of platforms, walks, flights and voids, its cells and door, and the navigation measures; `scp`: which anomaly appears on which level of a run |
| `world` | Grid and collision queries, zone purposes (`purpose`), regions, and the world build (`build`): floors, panels, core props, the themed placer passes, settling, placement, walls and ceilings, freezing static objects, instancing, culling; the Building's floors as built (`building`: slabs, flights, parapets, masses, lamps, the door, pickups) and dressed (`building_dress`: cells and their hermetic doors, sealed doors, the galleries above and below, windows, the door's lit tower, storey numbers, loudspeakers, the Well's lift cage, each place's own); the self-assembly (`assembly`) and the figure across the void (`far_figures`: its perches, where it can be seen from, and its standing, turning and going, on a stream of its own) |
| `assets` | The embedded prop and creature meshes (`assets/*.bin`), fitted to each placer's size; the creatures' bodies (`assets::bodies`: skeletons and the meshes bound to them, `bodies.bin`) |
| `placers` | The kit placers build with (`canvas_tex`, `pg`, joints, shared materials and tables) |
| `core_props` | The props every level needs: the exit, pickups, notes, switches, the pool, the vacuum, the safe room, elevators, holes, barricades, weather extras |
| `entities` | The creatures: rigs, skinning, animation, AI, voices, and the pacing director (`director`); a body built from `bodies.bin` and posed by aiming bones (`body`); the brain every creature of the redesign shares (`brain`: modes, percepts, awareness, memory, orders; `docs/ENTITIES.md`) |
| `anomalies` | The Showroom (a store that closes at night), the Borrowed House, the human rig they share |
| `modes` | The Stairwell and the Loop: self-contained spaces with their own movement, entered from a level |
| `player` | Movement, stamina, sanity, flashlight, items and actions |
| `audio` | Sound effects and persistent beds (`sys`), the spatial soundscape, the ambient event driver |
| `visuals` | Lighting (the place-based light map, pooled lights, fog), procedural textures and surfaces, the Building's photographed surfaces (`photo`: JPEGs decoded by `Canvas::from_jpeg`), the props' material pass (`prop_look`: muted colours, real highlights instead of glowing "shine", wear), the low-sanity colour filter |
| `ui` | The HUD and menus: an element model with CSS-like styles and classes (`model`), the HUD logic (`hud`), the journal's lore (`lore`), the three typefaces and their CoreText metrics (`faces`), the menus and their painter (`menu`, `screens`), the HUD at rest (`rest`: what shows when, and the building's sentence case), the notebook as paper (`paper`), and the painter that turns elements into composited layers (`draw`) |
| `session` | Start-up (`boot`), the per-frame update, input handling (`input`: the one binding table, held keys firing once, the crouch latch, the spent-run lockout, the early-E buffer; `settings`: what the player set, kept apart from the save, and the keys as rebound; `docs/UI.md`), and the game flow: levels, saves, death, the menu |
| `story` | M.'s descent: where her camp is on each floor, the walk she took and the trails that are not hers (`plan_story`, pure), her camp and her sheets (`dress`), her notebook (`notebook`), and the end of her story on level 13 (`ending`) |
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
one random stream seeded with `seed_for_level(run_seed, n)`. Levels 1–3 are
the Building's floors (`level::building::generate_building`, then
`world::building`; the dressing draws from a salted stream of its own, and
nothing walks there); from level 4 the rooms are built as below, their
layout, rot and creatures counted by `rooms_depth(n)` (the Lobby is depth 1):

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
   then her camp and her trail (`story`: once settling has made the walls
   final, so its sight lines hold; on a salted stream of its own, so no
   checkpoint moves), resolving overlaps (`world::placement`), walls (a seam wall between two
   regions drawn in two halves, one per side) and ceilings, then freezing
   static objects, merging repeated meshes into instanced meshes
   (`world::instancing`) and building the cull list.
3. Spawning the creatures.
4. Fitting the level to the run's progress, outside the seeded stream: the
   build ran as a new run would (with nothing read), so a retry or a
   Continue is the same place; then the notes already read are swapped for
   unread ones on a salted stream of their own (`fit_lore_notes`).

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

The creatures of the redesign (`docs/ENTITIES.md`; so far the Hound, the
Smiler, the Watcher and the one in the corner) run
their own brain instead (`entities::creature`, `entities::brain`): one
`Mode` at a time (idle, roam, alert, search, stalk, hunt, strike, withdraw,
...), changed only by a rule of the kind and logged with its reason;
percepts computed once a frame (hearing along the way it would walk to you,
`path_distance`, not through walls); awareness that fills from the senses
(from 0.3 the tell, at 1 sure); the director's wishes as orders. Fairness is
shared code no kind can skip: a strike needs two tells you could perceive
in the last 20 s, not the level's first 30 s, and not the 2 s after any
notice; the first contact of a kind in a run knocks you down and sends it
away for 45 s (a second in the same peak, or one on a spent nerve, kills);
chases are capped below your sprint, bursts at 7 m/s over 4 m. A kind is
its data and its own `think`, `animate` (on a body from `bodies.bin`) and
`voice` (`entities::hound`, which hears; `entities::smiler`, which sees
your lamp and loses you in the dark; `entities::watcher`, which moves only
unseen, its poses morphs of one sheet, among furniture under the same
sheets; `entities::corner`, placed after the build and never sent, woken by
your eyes on its face).

Above them, the **director** (`entities::director`) shapes each level into a
cycle, after the AI director designs of horror games (a menace gauge that
backs the threat off after a peak; build-up, peak and relax phases):

| Phase | What happens |
|---|---|
| Calm | Nothing is sent after you (45–80 s on the first floors of the rooms, shorter deeper down; 110–150 s for the first on level 4, the Lobby, while M.'s camp is there to find). Taking her page cuts it to 4–9 s (`director_wake`). Nothing walks the Building's floors (levels 1–3) |
| Build | One creature is sent (the nearest Smiler or Hound, now and then the Watcher; not the one sent last time when another can go). It is steered toward your part of the level, to tiles you cannot see in a circle round you that tightens as the build-up goes on (16 m out at first, 3–8 m at the end). Its footsteps far off, then the lights over it stuttering, then quiet as it closes in. If the time runs out it is told where you are (and again every few seconds) and gets a last stretch of up to 25 s to come and look |
| Peak | Something hunts you, or searches close by. The menace gauge fills with closeness and sight; once it is full and you have broken away (nothing hunting you within 8 m or in sight) for a few seconds, or it is half full again and the hunter is not on top of you, everything after you withdraws |
| Relax | The creatures keep away and hear less for 25–45 s |

On the Building's floors nothing walks: the director runs the
**self-assembly** there instead (`world::assembly`), in the same phases,
so the log reads alike: calm (the quiet), build (the siren from the
loudspeakers, the fog rising from far below toward the lowest walk in 42
s), peak (the fog among the walks, then at its crest), relax (it sinks;
the all-clear). Out in it the mind drains and you die (`assembly`);
sealed in a cell (its hermetic door shut by its wheel) you are safe. The
lamps under its surface go out, and with your eyes under it the fog is
what you see (the camera is in that place). Her page sounds the siren.

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
per run seed and level), lets the exit door call from where it stands (a
draught and the sign's relay every 8–10 s, heard from up to 40 m: with the
door's light in the light map, it is how the door is found, since there is
no compass), and starts every cadence part-way through its
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
