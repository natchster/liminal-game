# t03: the washroom shelf and the dressing table's leavings (Props P3)

Seven rare placers the census graded `cartoon`, remodelled from the real
things (`docs/design/PROPS.md`, "How a prop is remodelled"). The parts are placed as
modelled (`prop_part_as_modelled`, a scale of one), each in its prop's own
frame; each placer keeps its draws call for call. Checked with `--dump-level`
against the old code on every floor where one of them stood among the
census's 272 (seeds 4242, 77, 7, 202, 303, 1999, 11, 99, 555, 31337 over
levels 4–14; seeds 1–20 less 7 and 11 over levels 6–14): 43 floors, the
same draw count, the same trace marks, the same `solidsHash`, and every
other placer's objects (placer, position, bounds, turn, meshes) the same.
Each prop's own position is unchanged.

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Shampoo shelf (`place_shampoo_shelves`, p02) | A bathroom shelf of toughened glass, its edges polished, held at its back edge in two chrome-plated zinc clamp brackets (a wall plate, the jaw the glass sits in, a grub screw under it) | 500 × 120 mm of 8 mm glass, its top at the old plank's 1.21 m; the clamps 24 mm wide, 55 mm deep | https://www.manomano.co.uk/p/milano-arvo-modern-wall-mounted-bathroom-glass-shelf-with-square-chrome-brackets-500mm-length-2575300 (W 500 × D 120 × H 20 mm, 8 mm toughened glass, chromed brass brackets), https://weinmann-schanz.de/gb/en/Plumbing/Bathroom-accessories/Shelves/Eight-shelf/Shelf-bracket-Eight-L-520-mm-clear-glass-chrome-plated-brass-sid437409.html (520 × 120 mm, 6 mm clear glass, chrome-plated brass) |
| … its oval bottles | A 400 ml oval HDPE bottle of the "orchid oval" kind, a 24/410 neck, a flip-top cap; printed on the plastic | 76 × 45 mm, 201 mm to the neck's top; the cap 30 mm across, 23.5 mm | https://berryglobal.com/en/product/product-item/400ml-orchid-oval-hdpe-13823160 (400 ml Orchid Oval: 76 × 45 × 201 mm), https://packagingsolutions.amcor.com/en/product/product-item/400ml-orchid-oval-hdpe-13823160 (24/410 neck, shallow flip-top cap) |
| … its round bottles | A 250 ml Boston round PET bottle, a 24/410 neck, a disc-top cap; a paper label | 58 mm across, 129 mm to the neck's top; the cap 27 mm across, 18.5 mm | https://berlinpackaging.eu/en-international/products/boston-round-pet-bottles-250-ml-24-410 (250 ml, 24/410: 129 mm high, 58 mm across) |
| Shower cap (`place_shower_caps`, p02) | A reusable PEVA shower cap with an elasticated frilled edge, dropped on the floor opening down | 26 cm across laid flat; its elastic gathered to a ring 13 cm across (41 cm round unstretched), the frill 2 cm; lying, about 21 × 3 cm | https://www.walmart.com/ip/9405324584 (PEVA shower cap, 26 cm across), https://www.walmart.com/ip/14760969109 (PEVA, 26 cm); marketplace listings give 24.5–28 cm across and an elastic about 41 cm round before it is stretched |
| Powder compact (`place_makeup_compacts`, p03) | A pressed powder compact for a standard 59 mm pan: a moulded case, its mirrored lid on a three-knuckle hinge, a push catch; the aluminium pan glued in its well | 76 mm across, 15 mm shut (3 × 0.6 in); the pan 59 mm, 5 mm deep | https://www.qstomize.com/products/custom-compact-powder (ABS with mirror, 59 mm pan, 3 in across, 0.6 in thick), https://thevintagecompactshop.com/products/compact-case-for-pressed-powder-1960s-stratton-60-67-mm-diameter (cases for 59–67 mm pans) |
| Nail polish (`place_nail_polish`, p03) | Batch `r18`'s bottle (`place_small_nail_polishes`): the 13.5 ml square "retro" bottle | 25 × 25 mm, 52 mm to its neck, the cap 17 × 27 mm | `docs/research/props-remodelling/washroom_janitor_classroom_and_plant.md` |
| Perfume bottle (`place_perfume_bottles`, p03) | A 30 ml rectangular perfume flacon of the "Vermeer" kind: heavy-bottomed glass, a 15 mm crimp (FEA 15) neck, a spray pump crimped on, an overcap | a 42 × 25 mm base, 78 mm to the neck's top, its bottom 8 mm thick; the cap 25 × 25 × 32 mm | https://eurovetrocap.com/en/prodotto/products/fragrance/33578vermeeren (Vermeer 30 ml: rectangular, Ø 15 aerosol neck, 78 mm high, base 42 × 25 mm) |
| Small perfume (`place_small_perfumes`, p25 `t03.rs`) | A dressing table's perfume atomizer of the 1950s–70s French kind: pressed glass, a gilt metal collar and spray head, a rubber bulb in a silk net on a covered hose, a tassel | a 48 × 25 mm base, 72 mm to the neck's top, 86 mm with the collar; the bulb 20 × 34 mm | https://chrysler.emuseum.com/objects/7686/perfume-atomizer (French, c. 1950–1970: pressed glass, metal, rubber, tassel; 11.4 cm overall, base 4.8 × 2.5 cm; as the museum's index gives it, the page refused a direct fetch), https://www.1stdibs.com/furniture/decorative-objects/vases-vessels/bottles/art-deco-fan-design-cut-glass-perfume-atomizer-circa-1930/id-f_32924472 (cut-glass atomizers, 8–12 cm) |
| Makeup sponge (`place_makeup_sponges`, p08) | A teardrop blending sponge of hydrophilic polyurethane foam | 45 × 45 × 65 mm (dry) | https://www.desertcart.co.za/products/171729872-beautyblender-original-pink-beauty-blender-makeup-sponge-super-soft-foam (45 × 45 × 65 mm, 30 g) |

No bottle carries a real maker's name: the shampoos are an invented
Clearwell (anti-dandruff), Linden (conditioner), Meadow (herbal), Brightwell
(egg & lemon) and Velour (bath and shower gel), and the perfumes carry no
name at all.

## The parts (`t03.py`)

- **Shelf** (`Real__shampooShelf…`): `Glass` (its front corners rounded, its
  edges arrissed), `GlassEdge` (the polished edge, green as float glass's
  edge is: a skin over its sides), `Brackets` (each a side profile extruded
  24 mm: the wall plate, the jaw under the glass and the lip over it, the
  grub screw's socket head under the jaw, the plate's screw cover). The wall
  at z = 0, the glass's top at y = 0.
- **Oval bottle** (`Real__shampooOval…`): `Body` (lofted superellipses:
  the rounded foot and its push-up, a slight waist, the shoulder blending to
  the round neck and its thread ring), `Label` (a skin over its front, 30 to
  140 mm up, as far round as it faces forward), `Cap` (the flip-top: the
  lid's line, the hinge strap, the thumb recess, the orifice ring).
- **Round bottle** (`Real__shampooRound…`): `Body` (lathed: the foot, the
  shoulder, the neck), `Label` (a wrap round it, framed as a cylinder),
  `Cap` (the disc-top: 30 grip ribs, the tipping disc, its nub).
- **Shower cap** (`Real__showerCapFilm`): one film. The crown a height
  field over a disc grid (no pole at its middle): a low dome off centre,
  its edge on the frill or, in one lobe, spilled over it onto the floor;
  folds running in to the elastic (ridged noise, sharp along their ridges),
  creases every way at two scales, a side given way, a fold across. The
  frill round the gathered elastic in soft uneven waves.
- **Compact** (`Real__makeupCompact…`): `Base` (lathed: the rounded foot,
  the rim's step for the lid, the well; two hinge knuckles and their pin's
  ends, the catch's boss and push button), `Lid` (its skirt, the mirror's
  recess, the domed top; the middle knuckle, the thumb tab), `Trim` (a gilt
  ring round the lid's top), `Mirror`, `Pan` (the aluminium godet),
  `Powder` (the pressed cake, worn into a hollow by the puff, its lip). The
  hinge pin's axis is printed (`t03 compact: hinge …`) and copied into
  `T03_HINGE`.
- **Flacon** (`Real__perfumeFlacon…`): `Glass` and its thick `GlassBase`
  (one bottle cut at its inside floor: rounded upright edges, the shoulder,
  the crimp neck's bead), `ScentLow`/`Mid`/`High` (the inside below a fifth,
  a half, three quarters of it), `DipTube`, `Pump` (the ferrule crimped under
  the bead, its collar, the actuator and its nozzle), `Cap` (square, hollow,
  down on the shoulder).
- **Atomizer** (`Real__perfumeAtomizer…`): `Glass` and `GlassBase` (the
  same bottle, 48 × 25 mm, seven pressed flutes down its front and back),
  `ScentLow`/`Mid`/`High`, `Collar` (the knurled screw collar, the stem, the
  spray head across its top and the spout for the hose), `Bulb` (lathed, its
  neck and its cap, the net's knots as a fine noise; lying on the floor
  beside the bottle), `Hose` (arched from the spout down to the bulb's neck)
  and `HoseFlacon` (the same from the flacon's neck, 6 mm higher), `Tassel`
  (its cord, the bound head, 13 strands lying in a loose bunch).
- **Sponge** (`Real__makeupSpongeFoam`): lathed (a round base to a rounded
  point), the foam never quite true (noise at two scales), a dent where it
  was squeezed; framed as a cylinder (u round it, v up it). Modelled
  standing; the turn that lays it on its side at rest (its volume's centre
  lowest over the floor) is printed (`t03 sponge: lay …`) and copied into
  `T03_SPONGE_LAY`.

Not modelled again: the nail polish (`r18`'s parts, placed as they are).
Patch::Glass (the windows' glass) for the shelf, the flacon and the
atomizer, and (silvered, as the wall mirror's) for the compact's mirror.

## The game side, and the draws kept

Each batch module lives in a file of its own (`t03.rs` beside the placer's
crate's `lib.rs`; the atomizer's beside `dressing.rs`), declared where the
old function was.

- **Shampoo shelf** (p02): the gate, the wall tile, then the bottle count
  and, for each bottle, its colour pick and its height draw, as before. The
  colour picks the bottle: pale blue anti-dandruff shampoo, rose conditioner
  (ovals), green herbal shampoo, amber egg shampoo (rounds), plum shower gel
  (oval). The height draw sizes it uniformly (0.92–1.08) and turns it on the
  glass (±0.25 rad). Painted: each bottle's print (the ovals' straight on
  the plastic, the rounds' on a paper label round their front, small print
  on its sides: what is in it, how to use it), rubbed and water-stained, a
  torn corner; the plastic mottled, a scum ring at its foot, dried runs from
  its shoulder; the glass's dust, lime splashes and the rings of the bottles
  (where these stand, and two taken away); the chrome pitted, bloomed,
  rusting where the plating went. The shelf's width is the old plank's: the
  settled walls are the same (`solidsHash`).
- **Shower cap** (p02): the gate, the tile, the colour, x, z, the turn. The
  colour picks the cap: white with small faded blue flowers, pink with white
  dots, plain lavender, pale yellow with daisies, or a hotel's clear
  disposable one gone milky. Painted from above: its print, yellowed and
  dirty in blotches, mildew specks in clusters. Satin phong (the clear one
  translucent, opaque enough for the placement to measure it).
- **Compact** (p03): the gate, the tile, the colour, open (40%), the
  jitter's two, the turn. The colour picks the case: rose, black, gold-tone
  brass (its lid turned in rings, the plating rubbed through at its edge),
  ivory or teal; the plastic ones have the gilt trim. Open, the lid turns
  105° about the hinge pin; the mirror's silver gone black from its edge and
  in spots; the powder (light beige, medium or tan, from where it lies) worn
  into a hollow to the pan's metal and cracked where it was dropped.
- **Nail polish** (p03): the gate, the tile, the colour, the jitter's two,
  the turn, the tip (25%), as before. The colour picks the lacquer, dried
  dark (red, pink, plum, teal, a silver with its shimmer, black, yellow); a
  white cap (the old one was white); tipped, `r18`'s pooled lacquer and its
  lift.
- **Perfume bottle** (p03): the gate, the tile, the colour, the bulb (40%),
  the jitter's two, the turn. The colour is the scent's (amber, rose, pale
  blue, gold, coral), how much is left (a fifth now and then, mostly a half
  or three quarters) from where it lies; the glass's dust and the ring where
  the scent stood higher. Without the bulb: the pump, and the cap on it, or
  (one in three, from where it lies) off and lying by it; the pump and cap
  gold-tone, black or silver. With it: the atomizer's collar, hose, bulb and
  tassel on the flacon's neck (a refill), the silk crimson, faded rose,
  black or old gold.
- **Small perfume** (p25): the gate, the tile, the colour, x, z, the turn.
  The colour is the scent's (cologne, lavender, rose, lily of the valley,
  champagne); the rest as the perfume bottle's atomizer.
- **Makeup sponge** (p08): the gate, the tile, the colour, the smudge
  (55%) and its colour, x, z, the turn, the tilt, as before. The colour is
  the foam's, faded and greyed, its pores dark with dust, dirtier on the
  side it lies on; the smudge means used: foundation of the smudge colour's
  shade dried into its point and down the side that faces up, a tide at its
  edge. The tilt draw rolls it about its own length before it is laid (the
  stain turned up or partly under); the turn that lays it is in a copy of
  the geometry (`duplicate`, `rotate_y`, `rotate_z`, `translate`), so it is
  measured by its own box and its lowest point is on the floor.

Every painter that strokes with its own stream first skips a number of
draws set by its prop's salt (where it lies), or mixes the salt into its
seed, so its strokes differ from floor to floor; the per-pixel painters fill
their rows with `par_rows`. The extra build time is within the noise of a
level's build (a floor with three bottles on the shelf: under 40 ms in four
runs of each).

## Sizes against the old

The bottles are their real sizes: the ovals 19–22 cm with their caps (the
old height draw's uniform 0.92–1.08) where the old were 16–22, the rounds
12–14. The compact is 76 mm across where the old
was 56; the flacon 78 mm high where the old was 65; the cap lies 21 cm across
and 3 cm high where the old dome stood 9 cm; the sponge lies on its side, 65
mm long, where the old stood 10 cm tall.
