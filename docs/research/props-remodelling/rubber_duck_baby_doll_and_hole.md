# r03 notes — P1, the everyday: the rubber duck, the baby doll, the hole in the floor

Three core props (`crates/game/src/core_props/builders.rs`), each made
again as the real thing at its real size (docs/design/PROPS.md). The census's
verdicts (research/props-census/review1, 3, 5): the doll "a clay
figurine", the duck "a sphere on a blob, no eyes, no shaped tail", the
floor hole BROKEN ("a black half-cylinder 1.2 m tall stands up out of the
carpet").

## The rubber duck (`build_rubber_duck`)

**The real one.** The classic bath duck: blow-moulded soft PVC, hollow,
flat-bottomed, 8–10 cm long. Teka Squibbles yellow duck 9 x 8 x 8 cm
(https://www.galaxus.fr/en/s5/product/teka-squibbles-badespielzeug-gelbe-ente-bath-toys-47995463);
MidOcean PVC duck 8 x 8 x 7 cm
(https://www.galaxus.ch/en/s5/product/midocean-pvc-duck-toy-bath-toys-58539472);
Diaqua classic duck 75 x 60 x 80 mm
(https://www.buchmann.ch/en/diaqua-classic-rubber-duck-yellow-p-485803.html).
The body and head one moulding with a parting line, the tail turned up to
a point, the wings in low relief, a broad flat bill (painted orange, upper
mandible over a lower one), painted eyes.

**Made.** 9.8 cm bill to tail, 7.3 cm tall, 7.2 cm wide (the old one was
28 cm tall).
- `Real__duckBody` — the moulding as one surface (metaballs: hull, breast,
  rump rising to the turned-up tail, neck, head, cheeks, the wings in
  relief), its base flattened where it sits; 540 tris.
- `Real__duckBill` — two lofted mandibles sunk 5 mm into the face: the
  upper broad, flat, rounded and overhanging, turned up at its tip; the
  lower narrower and thinner; the gape the groove between.
- `Real__duckEyeWhite`, `Real__duckEyePupil` — the painted eyes as thin
  patches laid on the head (ray-cast onto the body), the pupil toward the
  front.
- Materials: the vinyl a faded yellow (0xd2b866) under a grime map (faded
  and darker patches, grime gathered toward its base, specks); the bill's
  paint worn thin and chipped back to the yellow; the eye paint an
  off-white and a near-black (0x2a2622, inside the albedo floor).

**Draws.** The old builder made none (`let _ = cx`); the new one makes
none outside its painters. It is also built unplaced by the stacked-top
replay in `scatter`, which only needs its draws: none.

## The baby doll (`build_baby_doll`)

**The real one.** A bent-limb play baby of the 1970s–80s, 38 cm: the V&A's
Palitoy Tiny Tears of 1978 (15 in, 38 cm; soft vinyl, bent limbs,
blue sleeping eyes with dark nylon lashes, single-line brows, an open
mouth for its bottle, wrinkles at the wrists;
https://collections.vam.ac.uk/item/O35359/tiny-tears-baby-doll-palitoy/);
the boxed one of 1977–81 (https://collections.vam.ac.uk/item/O35354/tiny-tears-doll-palitoy);
the cheaper babies with a cloth body, vinyl head and limbs, moulded and
painted hair and sleep eyes (a 15 in Bitty Baby of the early 1990s:
https://alaniazapoved.ru/products/vintage-pleasant-company-american-girl-bitty-baby-doll-15-mo/17827027).
Sleep eyes close when the doll lies down (weighted balls carrying the
lids); one stuck half open is the common fault.

**Made.** Lying on its back, its head rolled toward its right: 34 cm crown
to heel with the legs bent, the head 10 cm wide and as wide as the
shoulders, about 30 % of its length.
- The head is sculpted in its own frame as a radial field about its centre
  (an ellipsoid with soft bumps and dents: the full forehead, the fat
  cheeks and jowls, a button nose and its wings, the pursed lips, the eye
  sockets and the folds over them, the ears, the short lower face, a neck
  stub set back into the body), its eye openings cut as tilted almonds and
  the bottle's hole bored between the lips; decimated once (1045 tris) and
  split in two: `Real__dollFace` (in front of the ears, under the
  hairline: its paint laid straight on along z, a plane frame) and
  `Real__dollHead` (the skull, ears and neck, laid into the body's frame).
- `Real__dollEyeR`, `Real__dollEyeL` — the balls, their fronts just behind
  the face's surface, the iris painted forward (sphere frame).
- `Real__dollLids` — each lid a shell on the ball filling its opening and
  tucked under the rim, the edge rolled: the right shut, the left stuck
  half open. `Real__dollLashes` — the nylon lash strip: fine blades from
  each lid's edge, lying over the cheek from the shut eye, standing out
  from the half-open one.
- `Real__dollHair` — moulded hair: a shell 1.5–2.5 mm proud of the scalp
  over the crown, thinning to nothing at a hairline high on the brow with a
  lock brought forward, swept over the ears and low at the nape, in broad
  combed waves.
- `Real__dollRomper` — the romper on the cloth body (metaballs: chest,
  belly, seat, the neck opening, puffed sleeves to mid upper arm, the legs
  gathered at mid thigh), soft creases; `Real__dollCollar` — the Peter Pan
  collar's two flaps and two buttons, laid on it.
- `Real__dollArms`, `Real__dollLegs` — chubby bent vinyl limbs out of the
  sleeves and leg bands: the left arm thrown up beside the head, the right
  down by its side; frog legs; mitten hands with the fingers moulded
  together and a thumb, creased wrists; feet with toes.
- What rests on the floor is flattened onto it (y = 0.5 mm), so nothing
  dips under the floor.
- Materials: the vinyl yellowed (#d6b19a), faintly uneven, grimed toward
  the floor; the face's paint (a canvas laid on along z): sprayed blush rubbed
  off the cheeks' crowns, fine light-brown brows, painted lower lashes, rose
  lips round the dark bottle hole, the nostrils, dirt in the creases round
  the eye openings, the nose and the mouth's corners, specks, a grey smudge
  on the right cheek, a child's faded blue ballpoint on the brow; the eyes
  an ivory gone yellow with grey-blue irises (phong: they catch the light);
  the lids a painted flesh; the lashes a dark brown; the hair paint a mid
  brown, combed in streaks, rubbed back to the vinyl in places; the romper
  a powder blue faded toward grey, its weave, old stains, the floor's grey
  where it lay; the collar an off-white cotton.

**Draws.** None, as before (also built by the core props' second half,
`seam.rs`, and unplaced by the stacked-top replay).

## The hole in the floor (`build_floor_hole`)

**Why it stood up.** The old builder drew a 0.82 m open cylinder 1.2 m
tall centred 0.6 m *under* the floor, a black disc at the floor and 14
blocks round the rim. The placement pass (`world::placement`) rests every
floor prop on the floor: the cylinder's box reached 1.2 m under it, so the
whole group was lifted 1.2 m (the dump showed it at y = 1.2), and the
black disc (a circle, which the pass ignores) went up with it; from above
its back faces showed as a black half-cylinder with the carpet inside.
`tail_floor_hole` (tail.rs) only sets the group at y = 0 and records the
fall trigger (r 0.75 m); it needs no change.

**The real one.** A hole broken through an office floor: the floor
covering (carpet or vinyl, about 10 mm) torn back; a composite slab of
lightweight concrete on a 2 in (51 mm) ribbed steel deck, about 150 mm in
all, with 6x6-W1.4xW1.4 welded wire fabric near its top (Vulcraft /
Bekaert data sheet, lightweight concrete on 2 in deck with
6x6-W1.4xW1.4: https://vulcraft.com/catalogs/Deck/VulcaftBekaertDataSheet-Nov2024.pdf;
2 in minimum cover over the deck:
https://engineering.purdue.edu/~ramirez/CE479/FALL05/Ce479Fall05-Composite-FloorDecks.pdf);
the deck bearing on open-web steel joists, a 16K being 16 in deep, its
chords double angles, its web round bar (SJI standard specifications:
https://steeljoist.org/wp-content/uploads/2021/05/StandardSpecs_K_LH_DLH-Series_0820.pdf;
https://en.wikipedia.org/wiki/Open_web_steel_joist). Broken, not cut: a
ragged opening with bites where pieces came away, the concrete short of
the covering's tear in places, the deck torn and its lip bent down, the
wires cut and bent, rubble and dust round it.

**Made.** The opening 1.2–1.55 m across (the fall trigger is r 0.75).
- How it is drawn: the floor cannot be cut, so the hole is drawn through
  it. What is under the floor goes first, deepest first, without writing
  depth (a ray from above into the hole meets what is higher first, so
  that order is right): `Real__floorHoleShaft` (the dark: a box open at the
  top, faces inward), `Real__floorHoleJoist`, `Real__floorHoleDeck`,
  `Real__floorHoleFace` and `Real__floorHoleLedge`, `Real__floorHoleWire`
  and `Real__floorHoleCover`; then `Real__floorHoleMask`, the opening at
  the floor's surface in depth only, which keeps the floor (drawn after)
  out of it. Nothing under the floor writes depth, so the placement pass
  never sees it (it is what lifted the old shaft); the old prop's boxes
  (the cylinder and the 14 blocks, where the pass had lifted them) are kept
  as an invisible footprint, seen only by the placement pass, so nothing
  else is placed over the hole and nothing else moves.
- `Real__floorHoleCover` — the covering's torn edge, 10 mm, frayed;
  `Real__floorHoleLedge` — the concrete's top bared where the covering
  tore back further than the slab broke; `Real__floorHoleFace` — the
  slab's broken face down to the deck, following its ribs, rough;
  `Real__floorHoleDeck` — the ribbed deck (12 in pitch), torn at the hole,
  its lip bent down into it (at least 1 cm of lip everywhere, so no row
  of it collapses); `Real__floorHoleWire` — the mesh's wires (3.4 mm, 6 in
  grid) cut where the slab broke, some short, some long, bent down;
  `Real__floorHoleJoist` — a 16K joist under one side (top chord of two
  angles the deck bears on, bottom chord, the round-bar web zigzagging
  between); `Real__floorHoleRubble` — 44 broken pieces (convex hulls, one
  face flat where it broke off the slab's top) from 1.4 to 13 cm and 20
  bits of grit, more on the side where the slab gave first, each resting
  on the floor on one of its faces; `Real__floorHoleDust` — the dust
  round the opening, a decal.
- Materials: broken concrete grey with its aggregate showing, grimed, the
  face going dark toward its foot with rust run down from the wires; the
  bared slab top with black adhesive smears and grit; the deck a dull
  spangled galvanised grey rusting along the tear; the joist's primer
  rusting and going dark with depth; the wires rust brown; the dust thick
  at the rim and thinning out, patchy, grit in it.

**Draws.** None, as before (also built by `seam.rs`).

## Checked

- Draws: `--dump-level` on (303, 14), (1, 6), (11, 5), (1, 7), (11, 14),
  (1, 12): the same draw counts as before on all six, and every object
  the same but the remodelled props' own (their bounds and mesh counts;
  a doll the placement pass once stepped aside from the wheelchair it is
  set on now fits under the seat, and stays at the chair's place).
- Nothing under the floor: every part that writes depth is at y >= 0
  (the duck's base -0.08 mm, inside the pass's tolerance).
- `cargo xtask assets props --write` keeps a stored part's old bounds when
  its quantized bytes match, so a part that only moved (the doll's eye
  balls) kept its old place; a fresh write over the committed
  `prop_meshes.bin` puts every part where it is built.
