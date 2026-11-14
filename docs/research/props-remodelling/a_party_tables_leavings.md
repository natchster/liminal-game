# t13 notes: P3, the rare: a party table's leavings

Four placers the census graded `cartoon`, made again as what a party
leaves when nobody comes back for years (`docs/design/PROPS.md` 1, "the party and
the shrine"): printed paper cups and plates, a pressed glass cake stand
and the cake nobody ate on it, cornbread muffins in their paper cases;
food gone as food goes when it is left (dried hard, shrunk, cracked,
mould in soft colonies, crumbs), never a cartoon of food. The parts are
placed as modelled (`prop_part_as_modelled`, a scale of one). The placers
live in modules of their own: `placers_p25/src/music/t13.rs` (the cup),
`placers_p26/src/party/t13.rs` (the plate), `placers_p07/src/t13.rs` (the
cake stand and the muffins, beside `r16`'s stale loaf, whose mould painter
`loaf_mould` they reuse); each keeps its draws, call for call.

The census's verdicts: the cup "a plain grey tapered cylinder with a
closed flat top ... no open mouth, no rim roll, no paper white or print"
(review1); the plate "a flat tan ellipse ... no raised rim, no fluted edge,
no dish" (review1); the cake stand "two stacked plain red cylinders with
white studs: a toy cake in a saturated red" (review4); the muffins "four
identical smooth tan domes, each with a dark dot on top ... no crumb, no
cracked crust and no paper cases" (review7).

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Paper cup (`place_small_paper_cups`, p25) | A 9 oz printed paper hot/cold party cup: PE-coated cupstock about 0.4 mm, a conical wall rolled from a fan blank with an overlapped side seam, the rim curled outward into a bead, a bottom disc set up inside and the wall's foot folded in round its skirt | 76 mm across the curl, 51 mm across the base, 89 mm tall | https://www.webstaurantstore.com/creative-converting-563131b-9-oz-turquoise-blue-poly-paper-hot-cold-cup-pack/999CP9TQ.html (3 in top, 3 1/2 in tall), https://hotpackglobal.com/product/white-paper-cup-9oz (75.2 / 52.2 / 88.2 mm), https://www.webstaurantstore.com/dart-solo-r9bb-jd110-bare-eco-forward-9-oz-wax-treated-printed-paper-cold-cup-pack/999R9BARE.html (3 in, 2 in, 3 11/16 in) |
| Paper plate (`place_small_paper_plates`, p26) | A 7 in coated paper dessert plate, pressed from board 0.45 mm: a flat well, a wall at about 30 degrees, a flange with its edge turned down, the pleats a press leaves round the wall and flange; printed on the coated face only | 178 mm across, 12 mm deep, the well 110 mm, 32 pleats | https://www.webstaurantstore.com/creative-converting-79161b-7-ivory-paper-plate-pack/999PP7EC.html (7 in, coated), https://www.imperialdade.com/catalog/product-detail/sturdystyle-plate-7-in-coated-paper-white-round-heavy-duty-1000-case?id=17158; depth about 1/2 in from 7 in party plates' listings |
| Cake stand (`place_cake_stands`, p07) | An 11 in pressed glass cake stand of the mid-century kind (L.E. Smith, Federal, Indiana): a hollow trumpet foot pressed in panels, its rim ground flat, a knop where the plate was fused on, the plate rayed underneath and its rim turned up and scalloped; in clear glass or milk glass | 281 mm across, 120 mm tall, the foot 125 mm | https://www.chairish.com/product/33986235/mid-20th-century-l-e-smith-heritage-collection-quintec-pattern-pedestal-cake-stand (11 in, 4 1/2 - 5 in), https://www.chairish.com/product/32442001/mid-20th-century-federal-glass-windsor-pattern-cake-stand (11 in, 4 3/4 in) |
| The cake on it | A two-tier buttercream cake, an 8 in tier under a 5 in, each tier two layers (4 in tall fresh), piped shell borders and rosettes; left for years: dried and shrunk, slumped, the frosting cracked in a net to the sponge | tiers 196 and 126 mm across, 79 and 64 mm tall (shrunk about 7%) | https://veenaazmanov.com/cake-serving-chart-guide/ (Wilton's charts: tiers 4 in tall, two 2 in layers) |
| Birthday candle in it | A twisted (spiral) birthday candle, 2 1/2 in by 3/16 - 1/4 in, burnt down | 5.2 mm across, 34 mm left above the frosting (of 64) | https://www.walmart.com/ip/44901699 (2.5 x 0.25 in), https://www.heb.com/product-detail/product/15550061 (0.19 x 2.5 in) |
| Cornbread muffins (`place_cornbread_muffins`, p07) | Cornbread muffins baked in standard pleated paper baking cups (2 in base, 1 1/4 - 1 3/8 in wall, opened by the batter to about 2 3/4 in), years on: dried hard and shrunk away from the case, the crown cracked across | the case 50 mm at the base, 67 mm at its rim, 33 mm tall; the crowns 66 mm across, 51 and 54 mm tall | https://www.webstaurantstore.com/guide/629/types-and-styles-of-baking-cups.html, https://www.webstaurantstore.com/white-fluted-baking-cup-1-7-8-x-1-5-16-pack/999602450187.html (1 7/8 in base, 1 5/16 in wall) |

## The parts (`t13.py`)

- **Cup** (`Real__paperCupOuter`, `…Inner`, `…Rim`, `…Bottom`): one lathe
  each, angles from +z toward +x as the game's cylinder texture coordinates
  run. The outside: the printed wall, its side seam a board proud from 0.7
  to 11.5 degrees (a cut edge's step), the foot folded in under the bottom
  disc; the inside of the wall; the curl (a 2.9 mm bead rolled outward);
  the floor with its crease into the wall and the bottom disc's underside,
  6 mm up in the foot. `Real__paperCupSqueezed…`: the same four squeezed
  in a fist (`cup_squeeze`: pressed flat across x toward the rim while the
  disc holds the base round, sharp inward creases, the board's crumple, the
  rim tipped down on one side; a point moves by its angle and height, so
  the inside follows the outside).
- **Plate** (`Real__paperPlateTop`, `…Under`): the coated face with the cut
  edge, and the underside a board's thickness under it along its normal;
  32 pleats across the wall and flange (a zigzag in the surface, a wave on
  the edge). `Real__paperPlateBent…`: trodden on (`plate_tread`): creased
  across 22 mm off the middle, the far side bent up 28 degrees over 8 mm,
  crumpled where the crease crosses the wall and flange, the near flange
  pressed down by the heel.
- **Cake stand** (`Real__cakeStandFoot`, `…Plate`): the foot one closed
  lathe in 24 angles (the trumpet's twelve flat panels, its ground rim, the
  knop, closed over the top where the plate was fused on); the plate in 72
  angles (24 rays pressed up into the underside, 24 scallops notched into
  the rim, the top flat); sharp at 25 degrees so the pressed edges catch.
  `Real__cakeStandCake`: both tiers (the top tier sunk 3 mm into the bottom
  one, off its middle and leaning 1.6 degrees about its own foot, the
  bottom tier's side bulged), shell borders round both feet (28 and 18
  shells, each a head tapering into the next), six star-piped rosettes on
  the bottom tier's shoulder (the old six swirls), the buttercream's lumps.
  `Real__cakeStandCandle` (four flutes twisted round it, a crater burnt
  into its top, two runs of wax down it and a pool on the frosting) and
  `Real__cakeStandWick`, posed with the top tier. The flame is `t09`'s
  (`Real__brassStickFlameB`) at 0.55.
- **Muffins** (`Real__cornMuffinCase`, `Real__cornMuffinA`, `…B`): the case
  a single pleated sheet (24 pleats, its upper wall loosened and splayed
  where the muffin shrank from it); the muffin one lathe, its body in the
  case with a gap opening toward the rim, its crown pebbled (the coarse
  meal) and split along two (A) or three (B) cracks (`MUFFIN_CRACKS`,
  copied into `placers_p07/src/t13.rs` for their paint).

## The game side

- **Texture coordinates:** a part's caps and sides need different paint
  (the floor of a cup and its wall, a cake's top and its side), and the game
  lays both on the same map. `part_split`/`put_split` folds them apart (the
  sides round and up into the lower half, the caps from above into the
  upper left quarter) and doubles a corner a cap triangle shares with a side
  triangle, so no triangle runs across the map between the two (which had
  streaked the rims). The cup's bottom and the plates are laid from above
  at their real scale (`put_top`).
- **Cup:** the old colour draw picks the print: white with confetti, pink
  with white dots, blue with white stripes, yellow with white zigzags, green
  with small dots and a band, cream with "Happy Birthday" in a printer's
  script twice round (no brand, no character). The print faded, rubbed
  off where it was held and at the foot, the seam misregistered, a run of
  the drink dried down from the rim; inside, the drink (cola, punch or
  squash, by where it stands) dried down the wall from its tide line and a
  dark crust crackled on the floor. The old 25% tips it over: it lies on
  its curl and its base's edge, its parts turned in their geometry (the
  placement measures a turned part by its turned box, whose corner dipped
  into the floor and had lifted the cup 9.7 mm), the drink dried along the
  side it lay on and run out of its mouth into the floor (a soft stain).
  Standing (no draw: from where it is): drunk from and left; two or three
  still nested as they came, clean and dusty; or squeezed in a fist.
- **Plate:** the old colour draws pick the board's white and (60%) the ink
  round the flange (raspberry, royal blue, gold, green, purple): a band
  with white dots, rays between the pleats, or confetti; plain ones keep an
  embossed line. From where it lies: eaten from (frosting smeared with a
  fork and dried, cracked; grease soaked in round it and through to the
  underside; specks; `r16`'s crumbs moved onto the well), a stack of three
  or four as they came (dusty), dropped face down, or trodden on (bent,
  a sole's lugs in grime where it pressed).
- **Cake stand:** the old colour draw: white is milk glass (opaque, a dull
  shine, dust), cream clear glass (`Patch::Glass`, the wine glass's: the
  room by Fresnel, a little of it held in the thick glass, dust from the
  map's grey: thick on the plate round the cake and none under it, a ring
  of grime at the cake's foot, smears on the rim); its foot writes depth.
  The old frosting draw picks vanilla, pink, lilac or chocolate
  buttercream, all gone yellow-grey (the chocolate bloomed), cracked in a
  net down to the dried sponge, crazed between, fat seeped dark round each
  tier's foot, mould colonies (`loaf_mould`) mostly low on the sides, dust
  thick on the tops. The candle's and the flame's draws are the old ones;
  lit, it burns with the old light and the old two-sine flicker.
- **Muffins:** the old golden colour draw picks each muffin's crown (A or
  B, each its own painted cracks); the old ring's draws place them, the
  ring opened until real cases clear each other (the old one let 7 cm
  muffins sit 2 cm apart). The crust golden gone dull and dusty, pebbled,
  browner at the shoulder, the crumb paler at the cracks' lips, mould; the
  cases plain white, kraft or printed with small dots, grease soaked
  through in blotches with dark tide edges.

Every painter is seeded by where its thing is (`salt`), so what is painted
differs from floor to floor without a level draw.

## Checks

The draws, the settled walls (`solidsHash`) and every other placer's
objects (placer, position, bounds, turn, meshes, order) are the same as the
old placers' (built against the same `prop_meshes.bin`) on 14 floors:
202/8, 4242/14, 13/10, 14/9 (cups: squeezed, standing, lying); 555/8,
14/9, 6/7, 6/10, 18/9 (plates: stacks, eaten, trodden, face down); 31337/8,
13/9, 15/10, 17/13 (stands: clear glass with a cake, clear and empty, milk
glass with a cake, a cake with its candle lit); 20/8, 19/11 (muffins: four
each, white and kraft cases). The nested cups land on none of these: they
were shot from a local build that forced them (not committed).
