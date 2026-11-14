# r18: the washroom and the janitor's things, the classroom, the plant nobody watered (Props P2)

Seven placers the census graded `cartoon`, remodelled from the real things
(`docs/design/PROPS.md`, "How a prop is remodelled"). The parts are placed as
modelled (`prop_part_as_modelled`, a scale of one), each in its prop's own
frame; each placer keeps its draws call for call. Checked with `--dump-level`
on 148 floors (seeds 1, 2, 3, 7, 11, 77, 99, 202, 303, 555, 1999, 4242, 31337
over levels 4–14, and the variants' floors 10/11, 15/6, 15/14, 22/8, 29/6):
the same draw count and the same trace marks on every floor, and every other
placer's objects (placer, position, bounds, turn, meshes) the same, compared
with the old code built on the same `prop_meshes.bin`. Each prop's own
position is unchanged too.

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Washcloth (`place_washcloths`, p02) | A cotton terry washcloth, 400–700 g/m², a flat-woven (dobby) band set in from a folded hem | 305 × 305 mm, the pile 3 mm, the band 14 mm, 8 mm in from a 6 mm hem | https://www.webstaurantstore.com/12-x-12-100-ring-spun-cotton-beige-wash-cloth-1-lb-pack/161WC1BGEDZ.html (12 × 12 in, dobby border, 1 lb/dozen), https://www.webstaurantstore.com/zonda-wash-cloth-13x13-1-75lb-150ea-100-combed-cotton-dobby-white-c150/766ZO01W.html (dobby border, lockstitch hems), https://www.walmart.com/ip/1280303671 (12 × 12 in, 675 g/m²) |
| Nail polish (`place_small_nail_polishes`, p25 `dressing.rs`) | A small square bottle of nail polish: heavy-bottomed glass, a threaded neck, a moulded screw cap carrying the brush; no maker's name | the glass 25 × 25 mm, 52 mm to the top of its neck, its bottom 7 mm thick; the cap 17 mm across, 27 mm long | https://tkbtrading.com/en-lb/collections/nail-polish-bottles-and-tools/products/13-5ml-prince-retro-bottle (13.5 ml: 52 mm high, 25 × 25 mm), https://tkbtrading.com/products/15ml-prince-le-barrel-bottle (15 ml: 51.7 mm, 28.5 mm square, cap 34 mm) |
| Floss container (`place_floss_containers`, p03) | A 50 m dental floss dispenser: a moulded polypropylene box and its flip lid, a stainless cutter set in the lid | 40 × 55 × 19 mm | https://www.aosonline.co.uk/prdctrl-dentaglo-waxed-nylon-dental-floss-mint-pack-of-24-per4970-gem34970-a357274c (50 m, clear PP dispenser, steel cutter, 4 × 2 × 5.5 cm) |
| Overturned bucket (`place_overturned_buckets`, p28) | A 10 l household pail: polypropylene 2.2 mm thick, a flanged rim, a stacking bead, two moulded ears, a 3.5 mm zinc-plated wire bail with a plastic grip | 281 mm across the rim, 230 mm high, the foot 212 mm | https://www.mitre10.com.au/queen-plastic-bucket-with-metal-handle-assorted-colours-10l-2595361 (10 l: 230 mm high, 280 mm across, metal handle, plastic grip), https://www.advantage-catering-equipment.co.uk/products/jantex-round-plastic-bucket-white-10ltr (10 l: 260 × 280 mm) |
| Mop bucket (`place_mop_buckets`, p21) | A 26 qt commercial mop bucket of the Rubbermaid WaveBrake class: a moulded tub on four 3 in castors, a side-press wringer hooked over its end, a 24 oz cotton wet mop on a 54 in wooden handle | the tub 460 × 372 mm at its rim, its rim 362 mm up; the wringer 210 × 300 mm, its lever's grip at 865 mm; the handle 1.37 m | https://www.katom.com/007-FG748000YEL.html (26 qt side-press combo; 16.05 × 22.83 × 34.41 in), https://argo.webstaurantstore.com/rubbermaid-2064915-wavebrake-yellow-side-press-mop-wringer/6902064915YL.html (the wringer 13 × 13 × 27 in) |
| School desk (`place_school_desks`, p13) | A combination chair desk of the Virco 9400BR class: a 24 × 18 in laminate top on particleboard in a T-moulded edge, a chromed tubular steel frame, a steel-rod book rack under the seat; here the seat and back of moulded plywood (the older desks') | the top 610 × 457 × 29 mm, its surface at 745 mm; the seat at 440 mm; the frame 7/8 in tube; 0.84 m deep overall | https://argo.webstaurantstore.com/virco-9000-series-9400br-24-x-36/9769400P1BO.html (24 × 18 in laminate top, steel rod underseat book rack, tubular chrome steel frame), https://www.ontimesupplies.com/vir9400br70084-9400-series-chair-desk-21w-x-33-1-2d-x-30h-medium-oak-red-2-carton.html (21 × 33.5 × 30 in) |
| Dead plant (`place_dead_plants`, p30) | Zonal pelargoniums (geraniums) in a terracotta azalea pot, left unwatered: their stems go woody and grey and stay standing, the leaves dry on them and fall | the pot 315 mm across, 193 mm high (its old size: 320 × 200); the stems 300 mm | https://eddiswholesale.com/q114678 (Deroma azalea half pot 31 cm: 31.5 × 19.3 cm), https://www.masterclass.com/articles/pelargonium-care-guide (pelargonium stems are woodier, they do not die back) |

## The parts (`r18.py`)

- **Pail** (`Real__overturnedPail…`): `Body` (outside: the foot ring and
  recessed base, the wall, the stacking bead, the flanged rim with a chip
  out of it, the two ears with their holes and ribs; a little out of round),
  `Inside` (its wall, five litre ticks moulded on one side), `InsideFloor`,
  `BailSide`/`GripSide` and `BailDown`/`GripDown` (the wire bail hooked
  through the ears and its sleeve grip, swung about the ears until they rest
  on the floor as the pail lies on its side or upside down: found by
  bisection in `r18.py`). Its rest on its side (the turn about x that puts the
  flange and the foot on the floor, and the lift) is worked out from the
  body's vertices and printed (`r18 pail: tilt … lift …`), copied into
  `r18_pail::TILT`/`LIFT`.
- **Mop bucket** (`Real__mopBucket…`): `Tub` (a loft of rounded rectangles,
  tapered, a stacking bead, the rim rolled out and down, the end grips, the
  castors' bosses), `Inside` (its fill lines), `Floor`, `Water` (the surface
  at the fill line, fitted to the inside), `CastorForks` (top plate, race,
  fork legs with their lead, bridge, axle; each swivelled its own way),
  `Wheels` (tyred, their hubs sunk), `Wringer` (a hopper hooked over the
  end, its ribs, lip, drain grid, the lever's pivot boss), `Plate` (the
  perforated press plate), `Lever` (the steel lever, its strap and bolt),
  `Grip`; the mop: `MopHandle` (wood, its hanging hole), `MopClamp` (the
  quick-change jaw and collar), `MopStrings` (a matted mass with 46 yarns
  over the band's edge, heaped on the tub's floor).
- **School desk** (`Real__schoolDesk…`): `Top`, `Edge`, `Core` (the corner
  knocked off: the laminate and moulding gone, the particleboard showing),
  `Frame` (two side frames of bent tube: a runner to the rear post and up it
  to carry the back; the front post, the rail under the top, the bridge down
  to the seat rail; three cross tubes; the top's plates), `Glides`, `Rack`
  (rods), `Seat` (dished, its front edge rolled down), `Back` (curved),
  `Screws`; `BookPages` (both pages rising out of the gutter, a closed block),
  `BookCover`; the pencil `Pencil` (hexagonal), `PencilWood`, `PencilLead`,
  `PencilFerrule` (crimped), `PencilEraser` (worn); `Apple` (shrunk and
  wrinkled), `AppleStem`.
- **Dead plant** (`Real__deadPlant…`): `Pot` (the azalea pot's collar, its
  foot and drainage hole, chips off the rim), `Soil` (a crust shrunk from the
  wall, its edge turned down into the gap), `StemA`/`B`/`C` (woody stems that
  zigzag at their nodes, swollen there, a dried petiole's stub at each, a
  branch, the dried flower stalk; each top node and its direction printed,
  `r18 stem …`, copied into `r18_plant::STEM_TOP`), `LeafDry` (a leaf
  shrivelled and curled on its petiole), `LeafLive`, `Fallen`, `Litter`.
- **Washcloth** (`Real__washcloth…`): the cloth mapped from its flat square
  onto the floor: folded over a line across it on a 4.5 mm round (the grid
  dense there), the flap askew, its far corner curled up off it, three rucks
  where it bunched, the edges a little out of true; `Terry` (the pile, 3.2 mm)
  and `Border` (the hem and the dobby band, thinner), each a slab.
- **Nail polish** (`Real__nailPolish…`): `Glass` (the walls, shoulder and
  threaded neck), `GlassBase` (the thick bottom), `Lacquer` (dried down in
  it, a meniscus), `LacquerLying` (pooled along its side), `Cap`, `Brush`
  (the stem), `Drip` (a ring of dried lacquer under the cap and a drip down
  one face).
- **Floss** (`Real__floss…`): `Body`, `Lid` (split from one domed rounded
  box at the parting line; the thumb notch, the floss's groove), `Cutter`
  (a serrated plate), `Thread` (the end out of the groove, curled on the
  floor).

Not reused: the book stacks' hardbacks (`Real__bookHb…`) are closed; the
desk's book is open. The office Dracaena (`Real__drac…`) is another plant in
another pot; the dead plant is modelled after the pelargonium the old
placer's stems suggested.

## The game side, and the draws kept

- **Washcloth** (p02): the old colour draw picks white (gone to cream), peach,
  blue, coral, sage or grey terry. One painter per part, laid as the part is
  seen from above (its bounds read off the geometry): the loops (a texel
  each), the band's weave, dirt in blotches, mildew in dark clusters, one
  stain with a soft tide. Draws: the gate, the tile, the colour, x, z, the
  turn, as before.
- **Nail polish** (p25): the old colour draw picks the lacquer (red, plum,
  blue, gold, green, burgundy, near black, dried dark). The glass is phong,
  transparent (0.42); its thick bottom less clear (0.62), so the placement
  pass measures the glass on the floor, not the lacquer 7 mm up in it (it
  had dropped the bottle 7 mm into the floor). The old tip draw (20%) lays it
  on its side and shows the lacquer pooled along that side. The old glowing
  `basic_mat` is gone. Draws: the gate, the tile, the colour, the tip, x, z,
  the turn.
- **Floss** (p03): the old colour draw picks the body and lid (white and
  teal, blue and white, white and lavender, white and pink, yellowed); a
  scuffed map; the label printed on its face as a decal (DENTAL FLOSS, waxed ·
  mint, 50 m, 54.6 yd, a band of the lid's colour; no maker), rubbed. Draws:
  the gate, the tile, the colour, the jitter, the turn.
- **Pail** (p28): the old colour draw picks yellow, oxide red or grey
  polypropylene. The yaw draw's fraction lays one in three upside down (no
  new draw). The turn of its rest is baked into each part's own copy of the
  geometry (`duplicate`, `rotate_x`, `translate`): a turned mesh is measured
  by its turned box, which lifted it 3 cm off the floor. Painted: the outside
  round it and up it (mottled plastic, grime up from the foot, dried runs
  from the rim, scuffs pale and dark), the inside (a tide line, the film under
  it), the bottom (muck dried and cracked in flakes). The old 40% spill (a
  glowing `basic_mat` disc) is what ran out and dried: on a hard floor a
  grey film, a dark tide line and silt in lobes; on carpet a dark soaked
  patch (p28's `mark_tex`, `paint_px`, `tide`, with a painter of its own);
  its old radius draw sets its size, its old turn draw its shape. Draws: the
  gate, the tile, the colour, the yaw, x, z, the spill's gate, radius, turn.
- **Mop bucket** (p21): the old colour draw picks blue, grey or red (the
  cleaners' colour code); the old 60% water draw leaves grey water with a
  scum and lint on it (phong, transparent), else a dried ring and silt.
  Painted: the tub and wringer (grime, dried runs from the rim, scuffs), the
  inside (the tide and the scum ring over it), the floor, the water, the
  strings (grey-brown, darker where they lay wet). Draws: the gate, the tile,
  the colour, the water, `scatter_on_floor`'s three.
- **School desk** (p13): the old book's draws are all kept and used: its
  colour picks the cloth, its turn turns the open book (and the pencil on
  it), and the 28 draws of its 14 old lines of print seed what is painted on
  the pages and the desk. Painted: the laminate (a medium oak print worn pale
  where arms lay, rings, initials and a heart carved through to the paper,
  a game of noughts and crosses and a name in ballpoint, a year, scratches;
  turned to read from the seat), the plywood (varnish worn through where sat
  on, edges dark), the chrome (dulled, rust pitting up from the feet), the
  pages (a geography textbook's chapter on the water cycle in two columns,
  its figure, questions, running heads and page numbers; yellowed, foxed, a
  tide mark at the foot). The old 70% apple and its colour draw: an apple
  dried to a wrinkled brown husk (its old colour in patches); the old 60%
  pencil and its turn: a yellow HB worn short. Draws: the gate, the tile, the
  cloth, the turn, the 28, the apple's two, the pencil's two, x, z, the turn.
- **Dead plant** (p30): every old draw kept: the gate, the tile, alive (8%),
  the stem count, and each stem's way out, its lean, its foot's depth and its
  leaf's turn; the wall bias and offsets, the turn; the glow's draws (5%,
  30%, the phase: kept, the light gone, a plant gives none); the 40% fallen
  leaves, their count and each one's size (a uniform scale of 0.85–1.15),
  tilt (kept), turn and place. Each stem part is first turned so its own lean
  points outward, then leant out (the old lean draw, scaled to keep it
  inside the pot's rim). Painted: the pot (faded terracotta, salts bloomed
  white under the collar and in patches, green-black at the foot, pits), the
  soil (a crust cracked in plates of every size, perlite grains, the dark gap
  at the wall), the leaves (dry papery tan, veined; or a living zonal leaf).
  The fallen leaves share their geometry and material, so the instancing
  pass makes one instanced mesh of them: they leave the dump's object list
  (fewer `place_dead_plants` objects on floors with fallen leaves).

Every painter that strokes with its own stream first skips a number of
draws set by its prop's salt (where it lies), so its strokes differ from
floor to floor; the per-pixel painters fill their rows with `par_rows`.
Build cost: the desk 8.7 ms (its top and pages 3 ms each), the mop bucket
2.8 ms, the rest less.

## Sizes against the old

The mop bucket and the desk are much bigger than the old (a 0.15 m pail; a
0.38 m top on stilts): the brief's real ones. The pail is the old size, the
plant's pot within 2 cm of it, the washcloth 30 cm where the old was 22, the
nail polish 7 cm tall where the old was 6.6, the floss box turned on its back.
