# r13a notes: P2, the common: six things that hang on a wall

Six placers the census graded `cartoon`, made again as the real things they
stand for (`docs/design/PROPS.md` 1), after batch `r05`'s washroom mirror: the
domestic wall mirror (`place_wall_mirrors`, p29), the small cracked mirror
(`place_cracked_mirrors`, p22), the wall chalkboard
(`place_wall_chalkboards`, p22), the rustic sign (`place_rustic_signs`,
p06), the biohazard sign (`place_biohazard_signs`, p21) and the
cross-stitch (`place_cross_stitches`, p05). The parts are placed as
modelled (`prop_part_as_modelled`, a scale of one); every part is in the
frame of the wall it hangs on (x across its face, y up, z out of the wall,
z = 0 the wall's face), so a part's back is on the wall's real face wherever
`settle_props` slides the prop. Each placer keeps its draws call for call.

The census's verdicts: the wall mirror "a moulded frame around a flat, dull
tan panel with no reflection and no sheen ... an empty picture frame or a
corkboard" (review3); the cracked mirror "a neat spider-web drawing of
concentric rings and spokes with dots, and the glass reflects nothing"
(review6); the chalkboard "the writing ... in a bold typewriter face
(Courier-like) in perfect rows: no hand, no chalk grain" (review2); the
rustic sign "three orange wood planks ... two grey bars where the lettering
should be" (review2); the biohazard sign "three rings round a dot, not the
real trefoil of interlocking crescents, and the plate is pristine"
(review5); the cross-stitch "a flat terracotta ring around cream cloth with
nine big red squares ... pixel-icon blocks" (review1).

## What each is, and where its size comes from

| Prop | The real one | Size used | Source |
|---|---|---|---|
| Wall mirror | A domestic framed wall mirror: 4 mm silvered float glass in the rebate of a moulded frame on a hardboard back, hung flat on brass mirror plates (two at the top, two at the foot); a stained hardwood ogee moulding or a gilt cushion moulding (gold leaf on red bole over gesso) | 42 x 60 cm overall (the old frame's), a 34 mm moulding 20 mm deep, glass 352 x 532 mm seen | gilt framed mirrors of 22.5 x 30.5 x 2 in and 24 x 34 x 1.5 in: https://www.1stdibs.com/buy/gilt-framed-wall-mirrors/, https://www.chairish.com/product/36584211/vintage-english-gilt-beveled-wall-mirror; 3-hole mirror plates, 38 mm, 0.9 mm brass, used in pairs: https://www.lionpic.co.uk/p/10209/3-Hole-Mirror-Plates-38mm-Brass-Plated-100-pack |
| ... its washroom | When the old placer laid out a washroom round it: a wall-hung vitreous china basin with pillar taps, its chrome bottle trap; a 12 in school clock beside it | the basin 500 wide, 400 out, 175 deep, its bowl 380 across, its tap holes 52 mm off the wall, its rim 0.895 m up (where the old soap and toothbrush lay); the clock 32 cm (batch `r04`'s case at 0.944) | a 500 mm two-tap-hole wall-hung basin (Armitage Shanks Sandringham 21: 500 wide, 440 out, 190 high, bowl 400): https://www.manomano.co.uk/p/armitage-shanks-sandringham-21-wall-hung-basin-500mm-wide-2-tap-hole-17169721; the school clock: `docs/research/props-remodelling/torn_note_mannequin_and_wall_clock.md` |
| Cracked mirror | A small mirror in a stained half-round moulding, hung from one nail by a sawtooth hanger (so it hangs askew), struck once | glass 30 x 40 cm (the old one's), a 24 mm moulding 18 mm deep | as the wall mirror; how struck glass breaks (radials from the blow, rings between them, the crushed spot): https://www.nist.gov/document/glassfracturespdf |
| Chalkboard | The common 30 x 40 cm wall chalkboard: a painted board in a varnished solid pine frame, flat on the wall | 40 x 30 cm overall, the frame 22 mm wide and 14 mm deep, the writing face 356 x 256 mm | Securit Woody WBW-TE-30-40, 12 x 16 in, wooden frame: https://www.staples.com/securit-woody-wall-chalkboard-wooden-frame-12-x-16-wbw-te-30-40/product_24411647; 300 x 10 x 400 mm: https://p3.tbsg.co.uk/ProductDetails/Wall-Notice-Boards/Drywipe-Boards/1162715/Securit-Woody-Chalkboard-with-White-Chalk-Marker-and-Mounting-Kit-300xx10x400mm-Black-WBW-BL-30-40/Master.aspx; a pine frame 1.4 cm thick: https://www.galaxus.ch/en/s14/product/vidaxl-kreidetafel-bulletin-board-pinboards-53800413 |
| Rustic sign | A hand-made cabin sign: three pine boards nailed to two battens, a name burned in with a poker, hung by a jute cord from two screw eyes over one nail | 30 x 15 cm (the old panel's): boards 300 x 47 x 14 mm, 2.5 mm apart; the cord 5 mm | hanging wood signs of 12 x 6 in with rope: https://www.walmart.com/ip/Hanging-Wood-Sign-Home-Decorative-You-Loved-Hanging-Wood-Sign-Printed-Wall-Plaque-Wood-Signs-Hanging-Rustic-Family-Signs-Home-Decor-Gift-12X6-Inch/16161922873; three pine planks on a cord: https://store.unity.edu/wood-plank-hanging-sign |
| Biohazard sign | The ISO 7010 W009 warning sign (a signal-yellow triangle, its black band, the biohazard trefoil) over BIOHAZARD, printed on 1 mm rigid PVC with rounded corners, screwed through its four corners | A4 portrait, 210 x 297 mm; the triangle 180 mm a side, its band 5.3% of its side (the W009 drawing); holes 15 mm in from each side; 8 mm pan-head screws | the trefoil's construction (outer circles 300 units across, centres 110 out; inner 210 across, centres 150 out; the centre's hole 60; the ring 100-135, cut 10 short; slits 10 wide; gaps 40): https://commons.wikimedia.org/wiki/File:Biohazard_Symbol_Specification.svg (Baldwin and Runkle, Science 1967); W009: https://commons.wikimedia.org/wiki/File:ISO_7010_W009.svg; vertical W009 signs with a text panel are sold at 297 x 210 mm in polypropylene, the symbol alone at 200 x 200 mm (laboratory and electronics suppliers' listings, e.g. https://www.fishersci.ie/shop/products/iso-vertical-safety-sign-warning-biological-hazard/15700059) |
| Cross-stitch | An alphabet sampler: cotton cross stitches on linen, 14 to the inch (28-count over two), stretched and framed: a border, the alphabet, the numerals, a house between two trees, a motto, the maker's initials and the year | 22 x 28 cm overall (fits where the old 22 cm hoop hung), a 25 mm oak scoop moulding, the linen 17 x 23 cm seen | a sampler framed 16 3/8 x 12 3/4 in, worked in cross stitch on plain-woven linen: https://emuseum.chs.org/emuseum/objects/10633/sampler; house and alphabet samplers: https://www.chairish.com/product/29359242/vintage-framed-cross-stitch-sampler-alphabet-house-folk-art-cottage-farmhouse, https://www.historicnewengland.org/explore/collections-access/capobject/?gusn=7648 |

## The parts (`r13a.py`)

- `Real__wallMirrorFrameWood`, `Real__wallMirrorFrameGilt`: a profile swept
  round the 42 x 60 cm rectangle, mitred: the wood one's rounded outer
  edge, a low ogee crest, a quirk, a cove down to the sight bead; the gilt
  one's outer bead, raised cushion, sight bead and slip; each with its
  rebate behind (the glass's face 9 mm off the wall), the back standing on
  the plates. `Real__wallMirrorPlates`: four 38 mm brass plates on the wall
  under the frame's back, their heads standing 13 mm past its top and foot,
  each with its countersunk slotted screw.
- `Real__wallMirrorBasin` (the body: a loft of the plan, back straight,
  front a half ellipse, tapering to its foot, its rim rolled; the deck with
  two tap holes, the plug hole), `Real__wallMirrorBasinBowl` (the bowl's
  inside, split off to be painted apart), `Real__wallMirrorBasinTrap` (the
  chrome waste and its grate, the tail, the bottle trap and the pipe back to
  a rose on the wall), `Real__wallMirrorBasinTaps` (two pillar taps: base,
  body, spout, crosshead and its index button).
- `Real__crackedMirrorFrame`: a half-round moulding round 30 x 40 cm, its
  rebate.
- `Real__chalkboardFrame`: a flat eased frame, its groove for the board.
- `Real__rusticSignBoards` (three boards, each its own length, sawn a little
  off square, cupped, the sawn face lumpy, edges eased),
  `Real__rusticSignBattens`, `Real__rusticSignNails` (twelve heads, set
  unevenly), `Real__rusticSignEyes`, `Real__rusticSignCord` (three plies
  twisted round it, sagging from the eyes to the nail, knotted at each eye,
  the ends hanging), `Real__rusticSignNail` (angled up into the wall).
- `Real__biohazardPlate`, `Real__biohazardPlateLifted` (the top right
  corner sprung 4 mm off the wall where its screw is gone),
  `Real__biohazardScrews`, `Real__biohazardScrews3`.
- `Real__samplerFrame`: an oak scoop moulding round 17 x 23 cm.
- Reused: `Real__glassShard` (batch `r09`'s) for the cracked wall mirror's
  splinters; `Real__clockCase` and `Real__clockGlass` (batch `r04`'s) round
  the washroom clock's dial.

## In the game

- **The mirrors' glass** is the washroom mirror's (`build_broken_mirror`,
  copied into each placer crate): `Patch::Glass`, nearly all the light back,
  the room in it; its map where the silver has gone black from the edges
  (deepest along the foot and in the corners, spots ahead of the front,
  pinholes); a transparent film over it of dust, the last cleaning's
  swirls, the frame seen again round the edge, and the breakage. The wall
  mirror is struck one time in ten (the old crack overlay's draw), smeared
  by hands three times in ten (the old smudge overlay's draw); the small
  mirror always struck: radial cracks from the blow (some stopping short,
  some branching), rings between them near it, the crushed spot, each piece
  between two radials giving back the room a little differently, the silver
  flaked along every crack. The breakage comes from a generator seeded by
  where the mirror hangs (no draws), so silver and film agree. The wood
  frames are painted rail by rail, the grain along each, the mitres opened
  a hair, the stain rubbed thin along both edges, knocks, dust on the top
  rail; the gilt frame pixel by pixel from where on its profile a point
  lies: burnished on the beads and the cushion's crest and rubbed through
  there to the red bole, chipped to the white gesso along its outer edge,
  grime in the quirks. Which frame: gilt about one in three, from the
  height's draw. The splinters (the old shards' draws, five each) lie in
  front of the wall, silvered face up or painted back up. The old unlit
  overlays and shards (which glowed in the dark) are gone.
- **The washroom:** the old dark bar of a "sink outline" is now the
  basin under the mirror (its draw, 60%, says whether its taps could still
  be on it; the old tap draw, 50% of those, whether they are; else the
  deck shows the two holes they were taken from). The soap bar and the
  toothbrush that floated at 0.905 m now lie on its deck (the soap's offset
  kept to the deck, 0.205 m either side). Its glaze is grey, crazed, brown
  pooled toward the plug, runs under the taps, scale round the plug. The
  clock has the school clock's black case and glass round its old dial. The
  drip stain on the floor is lit, soft and ragged (it was an unlit disc
  that glowed).
- **The chalkboard:** one of six notes of the week's business (an order
  list, a meeting, someone out till the 14th, a broken photocopier, the
  coffee list, a delivery), in someone's chalk capitals: each letter a
  stroke at a time from a hand's glyphs (smoothed between its sharp turns),
  each a little bigger or smaller, leaning, off the line, the lines
  drifting; the chalk caught on the board's grain and skipping; over the
  ghost of the last note, wiped and smeared by the eraser's swipes, dust
  low on the board. No arrow, tally or circle (her marks and the secrets'
  sign), no rule and no hint. Which note and the hand come from where it
  hangs.
- **The rustic sign:** the boards grey, dry pine, each its own grain
  (flat-sawn rings in nested cathedrals, a knot, checks at the ends), the
  gaps dark, rust run from the nails; a family's or a cabin's name (or
  GONE FISHING, HOME SWEET HOME) burned in Copperplate, the char soft round
  each letter and uneven. No company's name.
- **The biohazard sign:** the trefoil tested pixel by pixel against Baldwin
  and Runkle's construction (4 x 4 samples), the W009 triangle with its
  rounded outer corners, BIOHAZARD in a bold grotesque; the yellow faded
  (more at its top), the ink gone brown-grey, scratches through the print to
  the white PVC (most on its lower half), scrapes, grime, rust run from each
  screw, dust on its top edge; one in three with a screw gone and that
  corner sprung (from where it hangs). Lit (`lambert`): the old plate was
  unlit and glowed.
- **The sampler:** each stitch a cross of two strands laid corner to corner
  (the top one always the same way), round and lit along its length, on
  linen with its weave and holes, 6 px a stitch; the colours (madder,
  indigo, green, ochre, brown, black) faded toward the linen, most at its
  top; the linen yellowed, foxed, darker under the lip. Its initials never
  carry an M (hers is M.).

## Draws

Every old draw is made in the same order and number: the wall mirror's
(the chance, the candidate, the smudge and crack chances, the height, the
slide, the sway and its phase, the shards' count and five each, the
washroom's chance, the clock's texture, side, offset, height and tilt, the
sink and tap chances, the dispenser's, the soap's, the stain's, the sign's,
the toothbrush's), the cracked mirror's (candidate, slide, tilt), the
chalkboard's (candidate, slide), the rustic sign's and the sampler's (wall
tile, tilt), the biohazard sign's (candidate, slide). Painters draw only on
their own streams, varied by where each thing hangs (no level draws).
Checked on all 79 of the census's 272 floors that have one of the six (the
census's seeds and levels): the same draw count on every floor, and every
other placer's objects (placer, kind, position, bounds) the same, against
the old placers built with the same prop meshes.

What changed of their own: the tilted ones (the rustic sign, the sampler,
the small mirror) still turn about the old point; the props' bounds are
their real sizes; the wall mirror's washroom adds the basin (one object)
and loses the bar and the two tap boxes; the splinters are instanced (as
the other placers' identical parts are), so they leave the objects list.
