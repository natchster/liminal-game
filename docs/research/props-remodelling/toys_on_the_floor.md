# r17 notes: P2, the common: toys on the floor

Five placers the census graded `cartoon` (the rubber ball, the toy car, the
pinwheel, the jigsaw piece, the magnetic letters), made again as the real
toys they stand for (`docs/design/PROPS.md` 1, "toys stay toys": moulded, faded,
scuffed, in the muted colours a toy has after years on a floor), after
batch `r03`'s duck and doll. The parts are placed as modelled
(`prop_part_as_modelled`, a scale of one). Each placer keeps its draws call
for call.

The census's verdicts: the ball "a sphere split by a hard horizontal line
into a dark-red top and a grey bottom, no seam, scuffs or rubber sheen...
a toy Poke Ball" (review1); the car "two boxes and four black cylinder
wheels" (review2); the pinwheel "four flat triangles in toy primaries...
upright on the bare tile on the point of its stick, with nothing holding
it" (review2, also listed as broken); the puzzle piece "one flat blue-grey
slab with a square tab, no round knobs, no printed picture" (review5); the
magnetic letters "identical rounded-square tiles with a white disc... no
letter shapes, they read as buttons or sweets" (review7).

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Rubber ball (`place_rubber_balls`, p26) | A 4 in hollow rubber play ball: two half shells moulded and joined at the equator (a low seam where the flash was trimmed), each its own colour | 101.6 mm across; sagged 3 mm at its base (a contact patch 3.5 cm across), a 2 mm dent | 4 in vinyl play balls: https://www.marchants.com/softex-4-ball-yellow, the hollow 4 in stage ball https://www.SynergyFlowArts.com/collections/sales/products/stage-ball; half shells joined: https://m.itftennis.com/media/2167/balls-ball-manufacture.pdf, https://patents.google.com/patent/US3825457A |
| Toy car (`place_toy_cars`, p26) | A 1:64 die-cast saloon of the 1970s (a Matchbox 1-75 car): a zamak (zinc) body in enamel, clear plastic glazing, a black plastic baseplate held by spun rivets, thin plastic wheels on steel axles, the bumpers, grille and lamps cast in and painted silver | 76 x 30 x 23 mm, wheels 11 mm | about 3 x 1.25 x 1 in: https://www.newegg.com/p/028-0001-00V48; 87 x 28 x 32 mm for a current one: https://www.onbuy.com/gb/p/matchbox-164-diecast-car-assortment-c0859~p232482053/; zamak, spigots through the baseplate: https://en.wikipedia.org/wiki/Die-cast_toy; a baseplate riveted on, plastic wheels and axles: https://www.vintagevolkswagenmodels.com/vintage-bus-toys-bs/lesney-matchbox/lesney-matchbox-vw-bus-v1 |
| Pinwheel (`place_small_pinwheels`, p26) | A small paper pinwheel (a party favour): a square of printed paper cut in from its corners, every other corner curled to the middle and pinned through a bead into a wooden dowel | a 10 cm sheet (its flat corners 7 cm from the hub), a 6 mm birch dowel 28 cm long (the old stick's length) | a 7 in square: https://www.hellokids.com/c_31745/kids-crafts-and-activities/two-colored-paper-pinwheel; a 9 in wheel on an 11 in pole: https://www.mysweetmuffin.com/collections/toys/products/djeco-dyi-pinwheels-sweet; a pin through the middle, a bead behind to let it turn: https://gerberlife.com/blog/make-paper-pinwheel |
| Jigsaw piece (`place_puzzle_pieces`, p05) | A piece of a children's 35-piece jigsaw: 2 mm greyboard faced with printed paper, die cut with round knobs on necks | a 38 mm body (the old one's), knobs 11.6 mm across, 2 mm thick | 35 pieces in 11.5 x 8.25 in: https://boutique.larevanche.ca/en/les-reines-de-locean.html; pieces of about 4-5 cm: https://www.skroutz.gr/s/22044753/Puzzle-pentru-Copii-Frozen-Intors-II-pentru-4-ani-Ani-35buc-05074-Ravensburger.html; 2 mm board: https://shop.rom.on.ca/products/arcadia-puzzles-1000-pieces-canada-eh-jigsaw-puzzle |
| Magnetic letters (`place_magnetic_letters`, p24) | Plastic fridge letters, the capitals of an alphabet set: a bold rounded sans moulded in polypropylene, a round ferrite magnet set into the back | capitals 38 mm tall, 8 mm thick, a 6 mm magnet | Lakeshore's capitals are 1 3/8 in: https://www.lakeshorelearning.com/products/language/alphabet/magnetic-letters-uppercase/p/LC206; 2 in jumbo ones: https://stemfinity.com/products/jumbo-magnetic-uppercase-letters; ABS letters, a magnet on the back: https://www.hktdc.com/event/hktoyfair/en/product/1Z03F9YTN |

## The parts (`r17.py`)

- `Real__rubberBallHalfA`, `Real__rubberBallHalfB`: each half a shell of
  rings about its pole, dense at the equator for the seam (a ridge 0.45 mm
  proud and 1.6 mm wide, a faint mould line either side); the seam's plane
  tilted 34 degrees from level (a ball rests any way up); its base pressed
  flat onto the floor by a smooth max and bulging a little round its foot
  (it has lost air); a soft 2 mm dent in its shoulder. 646 triangles each.
- `Real__toyCarBody`: the lower body lofted from sections along its length
  (a rounded rectangle: round shoulders, a near-square foot, the sides
  tumbled in), its bottom line rising over each wheel's arch, its plan's
  corners rounded (5 mm at the nose, 6 at the tail), the bonnet falling to
  the nose and the boot to the tail, the nose's and tail's edges chamfered;
  the roof and pillars a shell over the glasshouse with the windscreen, the
  rear window and the side windows cut out of it (A, wide C and thin B
  pillars left). 1190 triangles (a budget of its own, 1300: the arches and
  the glasshouse are what read as a car at 7 cm).
- `Real__toyCarGlass`: the glasshouse (a bevelled hull) set 0.8 mm in from
  the shell, showing through the openings.
- `Real__toyCarBright`: the bumpers following the plan round the corners,
  the grille and headlamps on the nose's face, the four hubs (domed discs in
  the tyres' recesses) and the two axles.
- `Real__toyCarTail`, `Real__toyCarBase` (the plate between the wheels and
  its two rivets), `Real__toyCarTyres` (four lathed wheels: a flat tread,
  rounded shoulders, a recess in the outer face for the hub, the inner face
  closed).
- `Real__pinwheelVane`: one quarter of the sheet as a grid from the middle
  out and across between two cuts; the strip along one cut (from where the
  cut stops to its corner) is a circular arc of the paper's own length
  through the cut's end and the pin, its plane leaning over the vane, and
  the paper sweeps from the flat sheet up into it; the flat corner droops
  to the floor. The game places it four times, turned a quarter each, each
  in its own paper. One-sided (drawn both sides).
- `Real__pinwheelPin` (the pearl head over the pinned corners, the shaft,
  the bead under the sheet), `Real__pinwheelStick` (the dowel along -z from
  just past the hub, bowed 2.5 mm).
- `Real__puzzlePieceA`, `Real__puzzlePieceB`: the outline (the body's sides
  straight, flat, a tab or a blank: a round bulb on a neck, each a little
  off its side's middle and leaning, as hand-made dies are), its sharp turns
  rounded, extruded 2 mm; split into the printed top
  (`Real__puzzlePieceFaceA/B`) and the board's edges and back. A: a border
  piece (a tab at +x, blanks at +z and -x, the straight side at -z); B: an
  inner piece (tabs at +x and +z, blanks at -z and -x).
- `Real__fridgeLetterA` ... `Z`: each capital from Blender's own font
  (`Bfont Regular`, built in, so the build repeats anywhere) made bolder by
  1.5 mm, extruded 8 mm with chamfered edges, laid face up with its top
  toward -z; a round pocket bored in its back (an exact boolean that
  resolves the bold strokes' own overlaps) where the letter is widest from
  its edges (searched on a 0.6 mm grid), the magnet
  (`Real__fridgeLetterMagnet<L>`, 6 x 1.3 mm) set in it 0.3 mm deep.
  Decimated to 470 triangles at most (Q, B); a budget of 480 each.

Each `auto(...)` line ends with its budget (the asset lint reads the last
argument), so the smoothing angles are a table by name (`SMOOTH`).

## In the game

- **Rubber ball**: the old top colour picks half A's rubber (red, blue,
  mustard, green, faded), the old bottom colour half B's (off-white, black,
  mustard, purple, faded). One wear map for both (a chalky bloom where the
  light found it, grime rolled in, scuffs, grit), a faint rubber sheen.
- **Toy car**: the old colour picks the enamel (red, blue, mustard, green,
  silver-grey, black), painted with ragged chips to the zinc (a dark edge
  round each), fine scratches, dulled patches and dust; the glazing dark
  grey, the silver paint worn. The old quarter on its side lie rolled about
  their length onto a flank (raised by the parts' half width), showing the
  baseplate and wheels.
- **Pinwheel**: the old red, blue, yellow and green as four printed papers
  (white dots in offset rows, some rubbed off; faded toward white; a tide
  line; floor dirt; a crease), the dowel grey-brown birch, the pin's head
  off-white. It lies face up, the dowel on the floor (never on its point,
  the census's broken case). The old lean (±0.075) turns the wheel on its
  pin within ±20 degrees of the dowel lying under a side's middle, where the
  sheet clears the dowel (a drooping corner over it would pass through it).
  The old 30% spin animator (its period and direction drawn as before) now
  stirs the wheel ±4 degrees on its pin: a wheel lying on its corners
  cannot turn round.
- **Jigsaw piece**: the old colour picks what the fragment of the picture
  shows, in flat colours with dark outlines as a child's puzzle is printed:
  sky and a cloud's edge (blue-grey), a fence's planks and a nail (brown),
  grass and daisies (green), straw (ochre), a tiled roof (rose). Faded,
  rubbed, dirty, the print worn back to the grey board at the body's
  corners and along its straight edges (not across a knob's neck), all
  along a border piece's straight side. The old second tab picks the inner
  piece (B), else a border piece (A). Painted per piece at 128 px over its
  own bounds.
- **Magnetic letters**: the old colours as the set's polypropylene, faded;
  one wear map for all (scuffs, scratches, grime). Which letter each is,
  and whether it landed on its back (about one in five, showing the magnet
  in its pocket), comes from where it fell (its drawn position and turn; no
  new draw), and no letter comes twice in a spill, so the letters are random
  and spell nothing. The old spacing test runs on the drawn positions
  exactly as before (it decided how many draws each letter made); each
  letter is then pushed off those already down by its reach (no draw), so
  none passes through another.

## Draws

Every placer makes the old calls in the old order, no more and no fewer:
the chance, the tile, the colour pick(s), the old second tab, the old
retry loop's draws (the letters), the position, the turn, the old lean and
the spin's three (the pinwheel), the flip and its side (the car); their
values pick the new variants. The painters draw on their own streams
(`canvas_tex`). Checked with `--dump-level` against the old build on each
prop's census floor and a second where it appears (202/8 and 1999/11,
1999/9 and 303/14 (on its side), 202/5 and 303/13, 303/13 and 303/7 and
1/15 (both shapes), 2/12 and 2/15): the same draw count on each, and every
other placer's objects (pos, min, max, rot, kind, meshes), the marks, the
entities, the exit and the player the same; only these five placers' own
objects differ.

## Left as it was

- Seed 303 level 13: the pinwheel lies in a pool room, on the floor under
  the pool's few centimetres of water, so it is seen through the water's
  tint (the old one stood 32 cm tall, most of it out of the water).
- The car's group no longer carries the old 25 mm lift on its side (the
  rolled body is raised inside it); its own object's `pos` moves by that
  much in the dumps.
