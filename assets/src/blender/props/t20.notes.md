# t20 notes: P3, the rare: a study's things

Eight placers of the victorian theme, made again as the real things they
stand for (`docs/design/PROPS.md`). None was framed by the 272-floor census (they
are rare, and some are instanced out of the dump): their floors were found
by a probe (seeds 5001-5040, levels 4-14) and, for the orrery and the
chess table, by scanning seeds 5041-5241 (levels 5-14); every dump, census
and shot ran with `--flags noinstance`. The parts are placed as modelled
(`prop_part_as_modelled`, a scale of one). What an earlier batch modelled
is placed, not modelled again: the chess table's brandy snifter is t10's
(`Real__snifterBowl`, `…Foot`, `…BrandyLow`), the pictures are
`visuals::pictures`' (t01, t02). Each placer lives in a `t20.rs` of its
own (`placers_p01`, `p13`, `p14`, `p15`, `p17`, `p06/src/rustic/`,
`p24/src/toys/`) and keeps its draws call for call.

What the old ones were (from their code): the bellows a 10 cm toy of two
boxes and four slabs; the globe a blue ball with 8-12 round green and
brown patches stuck on it on a 3-leg stick tripod; the orrery a glowing
yellow sun with a point light, wire arms and coloured balls, Jupiter with
a ring; the chess stand a plain round table, 64 coloured planes for its
squares and pieces of cylinders and spheres, a glowing red cigar tip; the
abacus an upright frame of toy-primary beads (red, blue, yellow, green);
the humidor a box lined in velvet with the cigars hovering over it; the
pocket watch a disc with boxes for its ticks and a faintly glowing case;
the music box a cube (pink or yellow among its colours) with a crank.

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Bellows (`place_bellows`, p06) | An English oak and leather fireside bellows, about 1880: two pear-shaped boards with their handles, the top one carved with a roundel, a gusset of leather nailed round their edges with brass round-headed nails, a turned brass nozzle in the nose block | 450 mm overall (the nozzle's tip to the handles' ends), 190 mm across the boards, 62 mm high lying (its leather a little open) | English oak and leather bellows c. 1880, max length 45 cm, 7.5 in wide, 2.25 in deep: https://www.chairish.com/product/29044025/antique-fireplace-bellows-english-oak-leather-fire-tool-victorian-c1880; early 19th century pair with turned brass nozzles, 15.5 x 5.75 in: https://lapada.org/art-and-antiques/antique-pair-of-early-19th-century-fire-bellows |
| Floor globe (`place_floor_globes`, p13) | A 12 in terrestrial globe on a floor stand: a full brass meridian, axis pins, an hour circle at the pole; a library stand of three turned legs with stretchers to a hub and a wooden horizon ring papered with its calendar, or a turned pedestal on a cross of four feet (the old stand draw) | the ball 305 mm (r 152.5 mm), its middle 720 mm up, 930-946 mm to the top of the hour circle; the horizon 436 mm across | C.F. Weber / Andrews 12 in floor globe, 34.75 in high, nickel-plated brass meridian, horizon with a printed calendar and zodiac: https://www.georgeglazer.com/wpmain/product/globe-american-andrews-cf-weber-terrestrial-12-inch-table-globe-nickel-plated-tripod-stand-antique-chicago-c-1890-1907-reserved/; Nims 12 in globe, calibrated brass meridian, papered horizon: https://www.1stdibs.com/creators/hb-nims-and-co/furniture/more-furniture-collectibles/home-accents/globes |
| Orrery (`place_orrery_models`, p14) | A London planetary orrery of about 1820 (after Newton's): a turned drum on three feet, a printed and hand-coloured paper dial (the zodiac by degrees, the calendar, the seasons), a brass pillar and sun, ivory planets on brass arms on their own sleeves, the Earth a small globe with its moon, a crank in the base | the base 300 mm across, the sun's top 301 mm up; orbits 50-178 mm (the old one's), the planets 8-29 mm | Whipple Museum 9243, Newton (London, 1794-1846): an oak base on three turned legs, 326 mm, 402 mm high, a printed hand-coloured dial, a brass sun, ivory planets on brass arms, a handle in the side: https://collections.whipplemuseum.cam.ac.uk/objects/9243; RMG portable orrery, ivory planets, brass arms and sun: https://www.rmg.co.uk/collections/objects/rmgc-object-11222 |
| Chess table (`place_chess_side_stands`, p17) | A Victorian tripod table (a moulded round top on its bearers, a turned column, three cabriole legs on pad feet), a folding board of boxwood and rosewood squares in a walnut border, Staunton chessmen of boxwood and ebony; a small cast brass cigar ashtray with two rests | the top 520 mm across, 700 mm high; the board 300 mm, its squares 35 mm (the old one's); a 64 mm (2.5 in) king, 26 mm at its foot, a 33 mm pawn | Victorian pedestal games tables, 39 x 39 x 80 cm and tripod tables of 28-32 in: https://hemswell-antiques.com/antiques/period-furniture/antique-tables/victorian-coromandel-and-satin-wood-inlaid-mahogany-games-table-108899.html, https://foundersantiques.co.uk/products/antique-victorian-mahogany-games-table-chess-table; Staunton pattern (1849), the square about 1.3 to 1.6 times the king's foot: https://thechessstore.com/british-staunton-chess-set-with-ebony-boxwood-pieces-3-5-king/ |
| Abacus (`place_small_abacuses`, p01) | A small Chinese abacus (suanpan): 13 bamboo rods, two beads above the beam and five below, a hardwood frame strapped with brass at its corners, lying on its back | 232 x 133 x 22 mm; rods 16 mm apart; beads 13.6 mm across, 8.4 mm thick | Lotus-Flower 13-rod suanpan, 32 cm, brass corners: https://mail.plusmodels.com/store/Vintage-Chinese-Lotus-Flower-Brand-Abacus-Suanpan-13-Rods-Red-Beads-1950s-70s_318761091795.html; a suanpan of 21.5 x 13 cm and 13 rods of 5 and 2 beads: https://www.lastdodo.com/en/items/5356009-suanpan-chinees-telraam, https://www.chairish.com/product/15596458/vintage-1900-chinese-wood-abacus |
| Humidor (`place_humidors`, p15) | A Victorian table humidor: a veneered pine case (mahogany, burr walnut, rosewood or ebonised: the old colour's draw) on a moulded plinth, lined with Spanish cedar (fresh or darkened) or zinc (the old lining's draw) standing proud as a lip, two brass butt hinges, a brass escutcheon, a brass name plate let into the lid; Corona cigars with paper bands; a steel guillotine cutter; a paper matchbook | 260 x 190 mm, 72 mm to the case's top, 102 mm shut; the lid open 99 degrees; the cigars 140 mm by a 42 ring (16.5 mm) | Victorian humidors cedar-lined, with lock and key: https://www.1stdibs.com/buy/antique-humidor; a Victorian rosewood box, 4.73 x 10.52 x 7.68 in: https://www.1stdibs.com/furniture/decorative-objects/boxes/cigar-boxes/material/rosewood/; the Corona, 5 1/2 in by a 42 ring |
| Pocket watch (`place_pocket_watches`, p15) | A 16-size hunter-case pocket watch of about 1900: the domed back, the band, the bezel and crystal, the front cover open on its hinge at IX, the pendant, knurled crown and bow at III; a white enamel dial with Roman numerals and a seconds subdial at VI, blued spade and whip hands; an Albert chain of curb links to a T-bar or a winding key (the old chain-end draw); gold-filled, coin silver, rose gold or gunmetal (the old case draw) | the case 50 mm across, 12 mm thick; the dial 43 mm; the links 5.8 mm | Waltham and Knapp 16-size hunters, 50.0 mm: https://www.1stdibs.com/jewelry/watches/pocket-watches/1910s-waltham-gold-filled-antique-full-hunter-pocket-watch-size-16/id-v_25171752; Elgin 16-size hunter, Roman dial, seconds at VI: https://thewatchpreserve.com/products/antique-16-size-elgin-hunter-pocket-watch-grade-114-gold-filled-fancy-box-hinge |
| Music box (`place_small_music_boxes`, p24) | A Swiss child's hand-crank (manivelle) music box of about 1900: a small box of beech, stained or painted (walnut, mahogany, dark oak, red, yellow ochre: the old colour's draw), a print pasted inside its lid, under a glass the works on a brass bedplate (the pinned cylinder, an 18-tooth steel comb, the wheel), the crank on its side with a porcelain knob | 100 x 70 mm, 44 mm high shut; the cylinder 40 mm long; the lid open 103 degrees | Swiss child's hand-crank box c. 1900, wood, metal and porcelain, 10 x 7 x 4 cm, an illustrated paper: https://www.elcoleccionistaeclectico.com/en/swiss-childrens-hand-crank-music-box-wood-1900s-p23846; Thorens manivelle (hand-cranked, no spring): https://emuseum.its.iastate.edu/objects/11962/manivelles-music-box |

## The parts (`t20.py`)

- **The bellows** (`Real__bellowsBoards`, `…Leather`, `…Tacks`,
  `…Nozzle`): the boards a pear outline with its handle (a swell at the
  grip) extruded and bevelled, the top one turned up 4 degrees about the
  hinge line and carved with a raised roundel; the nose block; the leather
  a band round the body from one side of the hinge to the other, nailed at
  both boards' edges, sagging out between them in eleven pleats, a strip
  over the nose for the hinge; round-headed brass nails every 17.5 mm; the
  nozzle lathed (a flange, a collar with two beads, the cone, its lip).
- **The globe** (`Real__globeBall` (a `sphere` frame, so its map wraps it),
  `…Meridian`, `…Pins`, `…Tripod`, `…Horizon`, `…HorizonPaper`,
  `…Pedestal`): the meridian a 3.5 x 10 mm band; the pins, their nuts, the
  hour circle and its pointer; the tripod's legs swept along their axes
  with a turner's profile (bead, vase, ring, taper), bun feet, stretchers
  to a hub with a hanging finial, the column up to a slotted socket the
  meridian stands in; the horizon ring a moulded profile, its paper a thin
  ring on it; the pedestal a turned column on a cross of four arms with
  bun feet.
- **The orrery** (`Real__orreryBase`, `…Dial`, `…Pillar`, `…Sun`,
  `…Arm0`-`4`, `…Planet0`-`4`, `…Moon`, `…Crank`, `…CrankKnob`): the drum
  turned with its mouldings on three turned feet; the dial a disc of
  paper, the brass bezel round it; each arm a sleeve on the pillar at its
  own height, a flat bar out to its orbit, a boss, a post up to the
  planet's cup; the Earth's arm carries the moon's; the crank's boss, arm
  and ivory knob.
- **The chess table** (`Real__chessTableTop`, `…TableBase`, `…Board`,
  `…BoardBody`, `…BoardHinges`, `…King`, `…Queen`, `…Bishop`, `…Knight`,
  `…Rook`, `…Pawn`, `…Ashtray`): the top a turned thumbnail moulding, its
  bearers and block; the column a baluster, the legs swept S-curves
  tapering to pad feet; the board's inlaid face apart from its body (so
  the squares do not run down its edges), the fold; the chessmen lathed
  from Staunton profiles (the king's cross, the queen's coronet of eight
  points and ball, the bishop's mitre cut, the rook's crenellations, the
  pawn's ball), the knight's head cut from its profile and rounded, its
  muzzle narrowed, its mane and ears; the ashtray a cast dish with two
  rests.
- **The abacus** (`Real__abacusFrame`, `…Rods`, `…Brass`, `…BeadsA`,
  `…BeadsB`): the rails and stiles rounded, the beam; bamboo rods with a
  slight bow; brass straps folded round each corner with their pins; the
  91 bicone beads cleared (A) or set to a sum (B).
- **The humidor** (`Real__humidorBox`, `…Lining`, `…Lid`, `…LidLining`,
  `…Brass`, `…LidBrass`, `…Cigar`, `…CigarBand`, `…CigarFoot`,
  `…CigarAsh`, `…Cutter`, `…Matchbook`): the case hollowed, its plinth a
  moulding lofted round it; the cedar lining standing 6 mm proud; the lid
  cushioned, hollow below, modelled shut about its hinge line; the hinge
  leaves, knuckles and screws, the escutcheon cut to its keyhole, the
  canted name plate; the cigar lathed (its cap rounded, its foot cut, the
  wrapper's seam a ridge, a slight bend; a `cyl` frame so the leaf wraps
  it), its band a ring of paper (also `cyl`), the cut foot's filler, a
  smoked one's ash; the cutter's body with its hole, the blade and finger
  loop; the matchbook's cover, flap and staple.
- **The pocket watch** (`Real__pocketWatchCase`, `…Cover`, `…Dial`,
  `…Crystal`, `…Hour`, `…Minute`, `…Second`, `…Link`, `…TBar`, `…Key`): the
  case lathed (the domed back, the band, the bezel), the case's knuckles,
  the pendant, the knurled crown, the bow fallen onto the floor; the cover
  modelled open and turned past upright till its rim rests on the floor (a
  part tilted in the game is measured by its tilted box); the crystal a
  thin dome; the hands cut flat (a spade, a whip, a needle) with their
  bosses; a curb link a twisted oval of wire; the T-bar and the key lying
  flat.
- **The music box** (`Real__musicBoxCase`, `…Lid`, `…Paper`, `…Brass`,
  `…Comb`, `…Glass`, `…Knob`): the box hollowed with the fillets the glass
  rests on and a foot moulding; the lid hollow, a small knob on its front,
  modelled shut about its hinge line; the print on the hollow's ceiling;
  the bedplate on pillars, the brackets, the cylinder and its arbor, the
  great wheel's 36 teeth, the shaft through the right wall, the crank's
  boss, arm and pin, the hinges; the comb's base, 18 teeth (the bass ones
  longer and thicker), its screws, the cylinder's pins.

## The game side

- **Bellows**: oak gone dark under old wax, its rays' flecks, rubbed pale
  at the handles, soot near the nose, dust; the leather dried, crazed into
  islands gone grey at their lips, scuffed; the nozzle's brass gone brown,
  a little verdigris. Draws: the chance, the tile, its place (two) and
  turn, as the old. It now lies on the floor (the old one floated 7 mm and
  was set down by the placement pass).
- **Floor globe**: the gores are the Blaeus' world map of 1635-1649
  (`map_world`, CC0, Rijksmuseum RP-P-1958-66: Willem and Joan Blaeu made
  globes as well as maps) laid back onto the sphere through Mercator's
  projection, measured on the atlas (the polar circles at rows 139.5 and
  325 of its cell, the tropic of Capricorn at 257: 59.05 px a unit of
  Mercator's y; its neatlines 53 px and 370 px apart for 360 degrees; the
  map from 80.5 N to 70.7 S); the sheet's two polar insets (they lie over
  the South Sea) taken out, the sea's paper and the graticule carried
  through; polar caps beyond its edges printed with their meridians and
  circles; the twelve gores' joins; amber varnish, crazed, rubbed through
  round its middle by the hands that turned it, foxed, dust on its top.
  The horizon's paper printed with the rings of a library globe's horizon
  (degrees, the compass's points, the zodiac's signs washed by hand, the
  calendar's months), its varnish amber and rubbed through where it was
  leant on. The meridian's degrees engraved. The stand stained as the old
  wood's draw (mahogany, walnut, ebonised, golden oak, fumed oak). Draws:
  the chance, the tile, the wood, the stand, the land patches' count and
  each patch's six (the first one's longitude is where the ball stopped),
  the spin's, its place and turn. **Dropped**: the slow spin (40%): the
  ball turned by itself forever with nothing turning it; it stands still
  where it was last turned, the spin's draw kept (`let _ = rnd();`).
- **Orrery**: mahogany dull and blooming; the paper dial printed (the
  sun's face, the orbits named, the zodiac's signs and degrees washed by
  hand in three colours, the calendar's months on a pale blue ring), its
  varnish gone brown, foxed, a ring of dust; brass gone brown, rubbed;
  the planets ivory, painted (Mercury grey, Venus cream, the Earth seas
  and land of no exact shape with white poles, Mars red-brown, Jupiter
  banded), yellowed. Draws: the chance, the tile, the turning draw, its
  place and turn. **Dropped**: the planets going round by themselves
  (60%; no hand on the crank): they stand where they were left, the draw
  kept; the sun's glow and its point light (a lamp where there is none;
  "the sun is brass, never a glow"); Jupiter's ring (a toy's; Jupiter has
  none to see).
- **Chess table**: the stand's stain from the old wood's draw; the top's
  figure with faint white rings of wet glasses, scratches, dust; the
  board's veneers laid each its own way, the stringing, worn at the
  players' two edges, dust. Each piece's square, side, kind and turn are
  the old draws (the square drawn again while taken, up to 30 times, as
  the old); the taken pawns lie on their sides along the board, resting on
  their foot's rim and their ball. The snifter (30%) is t10's glass with a
  little brandy; the ashtray (30%) cast brass, a cigar (the humidor's
  part) on its rest, its ash grey. Draws: all of the old, in order.
  **Kept**: `chess_set_positions` (the clicks of a move). **Dropped**: the
  cigar's glowing tip (a cigar left on a rest goes out in minutes; nothing
  burns here).
- **Abacus**: the hardwood near black, rubbed red at its edges, dust; the
  beads red-brown, polished; cleared or with a sum left on it, chosen by
  where it lies (no draw). Draws: the chance, the tile, its place and
  turn, as the old.
- **Humidor**: the veneer (ribbon mahogany, burr walnut, streaked
  rosewood, ebonised: the old box colour's draw), its polish dull and
  crazed, rubbed through at its edges, dust; the lining (Spanish cedar
  fresh or darkened, or zinc gone grey with a white bloom: the old lining
  colour's draw). The cigars: the old wrapper colour's draw picks its
  shade (claro to oscuro), the old length's draw scales the whole cigar a
  little (0.92-1.06), the old radius's draw how dried it is, the old turn
  its turn, the old place where it lies (three inside, on the one under it
  where they cross; the rest on the floor beside the box), the old band
  colour's draw one of four printed bands (FLOR FINA, HABANA, LA
  PREFERIDA, REGALIA: invented, in gold and ink). The cutter (40%) steel
  going to rust; the matchbook (30%, its turn the old draw) printed SAFETY
  MATCHES and aged as a print. **Dropped**: the spilled cigars rocking by
  themselves (50% each, nothing moving them): they lie still where they
  fell, each one's test and its three draws (phase, speed, swing) kept.
  The lid opens to 99 degrees on its hinges (the old one stood at 60 with
  nothing to hold it).
- **Pocket watch**: the case's metal (the old draw: gold-filled worn to
  its base metal, coin silver blackened in its hollows, rose gold,
  gunmetal worn grey), tarnished and scratched by a pocket; the enamel
  dial (Roman numerals, the minute track, the seconds' subdial at VI, a
  maker's WARRANTED), its enamel crazed, grime under the bezel; the
  crystal the windows' glass (`pictures::glass_mat`). The hands stop at a
  time: the minutes from the old minute hand's draw, the hour from the old
  hour hand's, the hour hand where it is at that minute. The chain's end
  (the old draws) as the old; a T-bar (60%), a key (30%) or nothing. The
  engraving (30%, the old draw) an inscription inside the cover in a
  copperplate hand. **Kept**: `hunter_watch_positions` (its tick).
  **Dropped**: the cases' emissive (metal does not glow).
- **Music box**: the finish from the old colour's draw (beech stained
  walnut, mahogany or dark oak, or painted red or yellow ochre), worn to
  the beech at its corners; the print inside the lid a picture of the
  atlas (a rose, a poppy, a landscape, a garden photograph), aged as an
  engraving; the glass (`pictures::glass_mat`), brass, steel, the
  porcelain knob. Draws: the chance, the tile, the colour, its place and
  turn, as the old.

## Checked

Every evidence floor dumped with the old placers and the new (the same
prop meshes in both), `--flags noinstance`: the same draws to the last,
the same `solidsHash` and counts, every other placer's objects the same
(placer, pos, min, max) on 5003/8, 5028/8 (bellows), 5017/12, 5050/14
(globe), 5047/11, 5055/10 (orrery), 5043/8, 5056/10 (chess), 5026/12,
5036/8 (abacus), 5023/9, 5043/5 (humidor), 5011/14, 5041/13 (watch),
5013/10, 5044/8 (music box). Each prop's lowest point is on the floor
(0.0, not under it). Shot from the census cameras and close cameras of
our own on those floors and on 5078/6 (the pedestal globe), 5114/6 (the
chess table with the snifter) and 5172/14 (with the ashtray).
