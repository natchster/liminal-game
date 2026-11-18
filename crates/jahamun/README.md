# Jahamun Tunnel (자하문터널), a set

Seoul, Jongno-gu, at night. You come down the long stairs on the polygonal
stone retaining wall to the red walkway beside the road, past the glass
balustrade, under the stone-faced arch, and into the bore: a 1.6 m walkway
of worn red paint between a white-tiled wall and the glass noise barrier,
the road lit beyond the glass, the same 2 m bay of light and steel
repeating for 485 m into a pale haze. At the far end a locked gate.

Built apart from the game first (in a workspace of its own), and now
moved into it: a crate of the game's workspace, read only by the engine,
its file and photographs embedded (*Moving into the game*, below).

```sh
cargo run --release -p jahamun                         # walk it, from the top of the stairs
cargo run --release -p jahamun -- --spawn inside       # top, plaza, portal, inside, deep, end
cargo run --release -p jahamun -- --shots DIR          # every named view, headless, and contact sheets
cargo run --release -p jahamun -- --shot F.png --at video   # one view (a name, or x,y,z,yaw,pitch[,fov])
cargo run --release -p jahamun -- --bench              # each view's frame time at 1920 x 1080 (CPU, GPU)
cargo run --release -p jahamun -- --frames 300         # the window, closed after 300 frames, its frame rate
cargo test --release                                   # the walk: rules, and the whole route on the set as built
```

Keys: W A S D or the arrows walk, Shift runs, the mouse looks (click to
capture it; Esc lets it go, Esc again quits), F fullscreen, 1-6 the spawns,
F2 writes the view headless to `target/jahamun-F2.png`.

## The place

What the set is measured from (`assets/src/jahamun/sources.json` lists every
source):

- **The tunnel.** Twin bores, each about 485 m, opened 30 August 1986,
  7.7 m high; the carriageway 16.8 m and the walkways 3 m, both bores
  together (Korean Wikipedia). So one bore: two lanes, 8.4 m, and a
  walkway of about 1.5 m. The walkways were made in 2010, the one toward
  Buam-dong behind a noise barrier (Namu Wiki): the glass.
- **The walkway**, from the developer's own phone video (private: looked
  at, never put in the repository): red-brown anti-slip paint worn to dark
  damp concrete along the middle and at the glass's foot, square steel
  hatches by the wall; on the left white pillow-glazed tiles (about 20 x 9
  cm, running bond) to about 2.2 m, a dark steel trim, then the grey lining
  curving over, two galvanised pipes on brackets; on the right the
  barrier's white steel foot with two access doors a bay, a 1.2 m LED bar
  on its sill in every bay (the walkway's own light), tall panes between
  dark mullions every 2 m, and the barrier going on overhead as a sloped
  glass lean-to on steel rafters; beyond the glass the road, its lamps
  near the crown, its far wall of white tile with a green band. The way-out
  signs, the blue plates numbering the escape points, a red-lit hydrant
  cabinet.
- **Outside**, from the photographs of the stairs (Commons, *Parasite
  filming location*, CC BY-SA 4.0) and of the place at night (Seoul Media
  Hub, 2022): the same red paint on the walkway and the plaza, a glass
  balustrade with an orange top rail and an LED strip under it, retaining
  walls faced with flat polygonal stones, the long stairs down the wall in
  two flights, the arch with its smooth ring, the lane signals over it,
  warm street lamps.

What is the set's own: the exact section (`assets/src/jahamun/set.py`,
*the section*), the far end's gate, the names and numbers on the signs, the
houses above the left wall, the variants' order along the bore.

## How it is made

```
assets/src/jahamun/textures.py   CC0 photographs -> crates/jahamun/assets/textures/*.jpg (albedo) and *_gloss.jpg
assets/src/jahamun/set.py        Blender: the place built from numbers, its light baked by Cycles -> crates/jahamun/assets/set.bin
crates/jahamun/src/              the set at run time (the engine only, its assets embedded) and its binary
```

```sh
uv run --project assets/src/jahamun python -I assets/src/jahamun/textures.py   # ~40 s
blender -b --factory-startup -P assets/src/jahamun/set.py -- [--samples 384]      # ~4 min on an M4 Pro (Metal)
blender -b --factory-startup -P assets/src/jahamun/set.py -- --manifest-only 1    # the manifest again (walk, placements, spawns), the bake kept
blender -b --factory-startup -P assets/src/jahamun/set.py -- --render F.png --view=x,y,z,yaw,pitch,fov [--piece outside]   # a Cycles look at the geometry
```

**The surfaces** are Poly Haven photographs (CC0), each fetched once and
checked by its SHA-1: *long white tiles* turned to run in rows; *painted
concrete*, its green paint recoloured to the walkway's red-brown and its
bare concrete darkened (worn paint shows dark in the tunnel), laid over a
whole module with the place's own wear (feet down the middle, damp at the
glass's foot) so it does not repeat every 2 m; *plastered wall 04* for the
lining and as a neutral paint (the steel, the barrier's foot, painted with
its doors and grime); *asphalt 02*; *rock wall 11*, graded grey, for the
retaining walls and the portal. Each is graded as an albedo, to the colours
the video shows once lit.

**The place** is made of numbers in `set.py`: the section (the walkway at
-x, the vault half an ellipse springing from the tiles' top), a 6 m module
of three bays, its seven variants (A plain; B a hatch and a way-out sign; C
the hydrant cabinet, an escape-point plate and a down pipe; D a dead bar and
a hatch; E two jet fans over the road; F the escape door in the tiled wall,
shut, under its plate; G the same door changed for the way in: an older
leaf standing a hand's width open on a concrete passage with a cold light
deep in it, her note taped beside it), the approach outside, the far gate.
No model file: the geometry is the script.

**Two layouts** (`Layout`): `Real`, the tunnel as it is (485 m from the
portal to the far gate; this binary's), and `Ring`, the game's way in
(`src/ring.rs`): past the entrance, a period of twenty modules (with the
escape door once) laid over and over, 252 m past either seam, under a
thicker haze that leaves 1% of what is there at that reach; at a seam you
are carried a whole period back (or on), which cannot be seen, and once
you are past the near seam the way back goes on too. The door changes in
every copy at once (`Set::set_door_changed`).

**The light** is Cycles': every lamp is an area light where its fixture is
(a 1.2 m bar a bay at 5500 K, the road's lamps, warm street lamps at 3300 K,
the balustrade's strip, the cabinet's red, the lane signals, a few lit
windows), and each piece's diffuse light, direct and indirect, without the
surface's own colour, is baked into its corners (the meshes are made fine
enough, 8-25 cm, to hold it). A module is baked in the middle of nine, so
its light is an endless tunnel's; a plain module's two ends are made to
agree, and every other piece is eased to the plain module's light at the
ends where it meets one, so any module may follow any other without a seam.
The entrance (the outside and the first 12 m of both bores) and the far end
are baked whole.

**The set file** (`assets/set.bin`, `src/data.rs`): the manifest (materials,
pieces and their lights, where each piece stands, the walkable ground, the
lit places, the spawns) and the meshes in the engine's frame (x across, y
up, -z into the tunnel; the portal at z = 0), deflated.

**At run time** (`src/lib.rs`, `src/look.rs`): a baked surface is drawn
unlit, its photograph times its baked light, the light tone-mapped once at
load on a film shoulder judged at the material's mean albedo, and darkened
by the dirt of years where it would be (`look::grime`: the tiles at their
foot and under the pipes, the floor at both edges, soot on the road's far
wall, the wheel paths; a lattice-hash noise of the place, no stream). What
moves with the eye is drawn live: the bars' highlights on the wet paint and
the glaze (a second, additive Phong pass of the same mesh, its gloss from a
map, lit by point lights at the bars and the street lamps only: every light
within reach is looped over at each pixel, and with the road's lamps too
the pass cost 13 ms of a frame); the glass, a milky film that shows more of
the lit tunnel the flatter you see it (Fresnel), with the bars doubled in it
(their faces mirrored in the barrier's plane, seen only from the walkway: a
reflection of the surfaces too, without their pictures, read as ghosts);
glows round the lamps, two along each bar; the lane signals and the signs
(painted on the engine's canvas). The fog is by place: a light map marks
the bores lit, so inside they fade to a pale haze and outside to the night,
whatever the eye is. The other bore is let go from deep inside, where it
cannot be seen. The walkable ground is boxes at heights (the walkway, the
plaza, each tread, the street above; `src/walk.rs`).

**The camera** is the binary's, not the set's (the game has its own): a
phone's grain (`src/lens.rs`, a hashed tile moved by the frame's number), a
cool grade with a little more contrast, and headless frames supersampled
1.5 times.

## Evidence

- **Against the video.** `--shots` renders `video` and `video_back` from
  the video's own viewpoints (portrait, its phone's field of view); the
  mean colour of the same regions, video / set (sRGB): lit tile 174, 180,
  177 / 177, 191, 195; barrier foot 113 / 105; lining 103 / 99; lean-to
  83 / 71; the view through the glass 137 / 108; the floor near the eye
  134, 66, 49 / 102, 66, 56 (the set's floor is still the duller: the
  video's phone saturates the red).
- **A reviewer's read** (an agent shown the renders beside the video,
  nothing of how they were made) ranked what gave them away; acted on:
  the glass too clear, the floor's wear (the walked lane paler, both edges
  dark), the corners not dark enough, the tiles' grout, the road's vault
  stains, the wheel paths, the night sky's glow, the stone's outlined
  joints, the camera (grain, grade). Not acted on: the tiles' own
  highlights (pillowed tiles need normals the engine's surfaces do not
  have), traffic, trees.
- **Speed** (`--bench`, M4 Pro, 1920 x 1080, 4x MSAA): 2.4-7.9 ms of GPU
  and 0.3-3.6 ms of encoding a frame, every view; the window 116 fps at
  1280 x 720 (`--frames`).
- **The walk** (`cargo test`): the rules (walls stop and slide, stairs
  climb and descend, no stepping off a flight) and the whole route on the
  set as built: the street above, the stairs' head through the coping's
  gap, both flights, the plaza, the portal, 485 m of walkway to the gate
  (and not through it); the glass, the tiled wall and the retaining wall
  stop you.

## Moving into the game

The set is shaped like the game's own self-contained places (the Stairwell,
the Loop: `crates/game/src/modes/`): the level world hidden, the place
running its own movement, glue mapping its camera, fog and sound onto the
game's. What moving in takes:

1. **The crate** (done): `crates/jahamun`, a member of the game's
   workspace; it reads only `engine`, has no window or device, draws
   nothing random and reads no clock.
2. **A mode** (done): `crates/game/src/modes/tunnel.rs`, the run's way in
   (`docs/design/WAY_IN.md`): the level world hidden, `Set::root` in the
   scene, movement with `Set::walk` and the ring's carry, `Set::update`
   with the eye each frame, the frame's fog and light from `Set::globals`;
   the door that changes, her note, the door into the first floor.
3. **The assets** (done): the pipeline and its sources in
   `assets/src/jahamun/` (the photographs all CC0, each with its SHA-1),
   the set file and textures embedded (`include_bytes!`, `src/assets.rs`)
   as the game embeds its own, not read from the crate's folder.
4. **Its meaning** (done, in part): not a borrowed place but the real one
   the run begins in (`docs/design/WAY_IN.md`, `LORE.md` *The way in*,
   checked by `lore-keeper`; its plan `docs/plan/WAY_IN.md`); a
   `content.json` entry (`story/way_in`); its tests
   (`crates/backrooms/tests/tunnel.rs`); the look into it (`tunnel_in`),
   your voice in it (`you_tunnel`, `modes::tunnel_voice`), its sound (the
   hum, the cars going quiet, the open door's air: `modes::tunnel_sound`).

## What is not here yet

In this binary, sound (the game's way in has its own: the bars' hum, the
cars behind the glass, steps, the open door's air); the jet fans' roar;
the road's traffic seen (it is only heard, in the game); trees and the
hill's green; the far portal seen from outside (the
gate is shut); the tiles' pillowed glaze catching a highlight each (no
normals on baked surfaces); the bake's light at its finest only near the
walkway (8-10 cm; the vault's is 22 cm). The set file and its textures are
15 MB, embedded in the binary.
