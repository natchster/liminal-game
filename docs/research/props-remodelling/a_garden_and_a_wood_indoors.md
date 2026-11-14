# t17: bits of a garden and a wood indoors (Props P3)

Eight placers no census floor had shown (they are on 1-7 of 440 probe
floors), remodelled from the real things (`docs/design/PROPS.md`): the acorns,
the chestnuts, the pine cone and the pine needles, the fern
(`placers_p03`, now in `crates/placers/placers_p03/src/t17.rs`), the framed
botanical (`placers_p06`, `src/garden/t17.rs`), the gardening gloves and the
garden rake (`placers_p05`, `src/garden/t17.rs`). The parts are placed as
modelled (`prop_part_as_modelled`, a scale of one). Each placer keeps its
draws call for call (below).

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Sources |
|---|---|---|---|
| Acorns (`place_acorns`) | Pedunculate oak (*Quercus robur*): the acorn oblong-ovoid, 19-35 mm (2.7 cm on average), 12-20 mm across, a quarter to a third of it in the cup; the cup hemispherical, 15-20 mm, grey-brown, of small appressed felted scales; the nut with a short point (the style) at its tip and a pale scar at its base. Most fall bare (the cups stay on the stalks). A weevil larva (*Curculio glandium*) leaves by a round hole of about 2 mm | nut 25 x 15 mm, cup 18 mm across and 8.4 mm deep, hole 2 mm | https://floraseries.landcareresearch.co.nz/taxa/33c4fa06-52c8-48f9-bb3e-4b5bdbf150aa (2-4 cm, 1/4-1/3 in the cup); https://www.idpan.poznan.pl/images/stories/dendrobiology/vol47suppl/47s_25_31.pdf (5,780 acorns: 1.9-3.5 cm, mean 2.71); https://turkiyebitkileri.com/fotoğraf-galerisi/fagaceae-kayıngiller/quercus-meşe/quercus-robur.html (cupule 15-20 mm, greyish-brown); https://elmedinaturaldelbages.cat/en/species/acorns-and-chestnut-weevil-curculio-elephas-curculio-elephas-en/ (the larva's 2 mm exit hole) |
| Chestnuts (`place_chestnuts`) | Horse chestnut (*Aesculus hippocastanum*): the fruit a leathery capsule about 6 cm across, set with stout prickles 4-8 mm, splitting into three valves; usually one seed (the conker), 2-4 cm, glossy chestnut brown, a large pale scar (hilum); dried, the husk goes brown and hard and the conker dull, dark and wrinkled | seed 31 x 24 x 28 mm; capsule 51 mm across, its wall 5 mm, prickles 4-7.5 mm | https://publication.plazi.org/GgServer/html/03B0402CFEC7E26EFEC2FA6AD0F3FA5A (capsule c. 6 cm, spiny; seeds 2-4 cm with a large white hilum); https://plantdatabase.kpu.ca/Plant/aehi (capsule 2.5-6 cm, one seed, rarely 2-3); https://www.monaconatureencyclopedia.com/aesculus-hippocastanum/?lang=en (the hilum big, white or greyish) |
| Pine cone (`place_pine_cones`) | Scots pine (*Pinus sylvestris*): the ripe cone 3-7.5 cm, ovoid-conic, short-stalked, opening when dry; its apophyses rhombic, flat to pyramidal (most at the base), keeled across, a small umbo; dull grey-brown, the scales' hidden faces red-brown | 56 mm long, 38 mm across its open scales, 58 scales on the golden angle | https://conifersociety.org/conifers/pinus-sylvestris/ (cones 3-7.5 cm; apophysis flat to pyramidal, small prickle on the umbo); https://dendro.cnre.vt.edu/dendrology/syllabus/factsheet.cfm?ID=112 (pyramidal mostly on the basal scales); https://acorn.mortonarb.org/Detail/objects/79819 (ovoid-conic, short-stalked; umbo small) |
| Pine needles (`place_pine_needles`) | Scots pine's needles: in pairs held in a persistent papery sheath 5-10 mm, 2.5-7 cm long, twisted, half-round in section; shed brown | pairs of 44-52 mm needles, 1.6 mm wide, in a 7 mm sheath; singles 47 mm; a drift of broken needles and dust 0.3 m across | https://conifersociety.org/conifers/pinus-sylvestris/ (needles in pairs 2.5-5 cm, sheath 5-10 mm); https://dendro.cnre.vt.edu/dendrology/syllabus/factsheet.cfm?ID=112 (1.5-3 in, twisted) |
| Fern (`place_ferns`) | Common polypody (*Polypodium vulgare*), the fern of walls, rocks and old trees: fronds 10-40 cm, the blade oblong, deeply pinnatifid into 10-18 pairs of narrow oblong blunt segments with finely notched margins, shortening to the tip; naked round sori, yellow to orange-brown going dark, in a row each side of the midvein, halfway to the margin | a frond broken off: blade 150 mm, 13 pairs of segments to 36 mm long and 7-8 mm wide, the blade about 70 mm across; its stalk 55 mm | https://fermanagh.bsbi.org/polypodium-vulgare-l-s-s; https://en.wikipedia.org/wiki/Polypodium_vulgare (10-18 pairs of segments, sori between midrib and margin, yellow to orange) |
| Botanical (`place_botanicals`) | A small framed botanical plate: one of the CC0 plates `visuals::pictures` paints (Ehret's Pulmonaria and Magnolia, Redouté's Rosa gallica, van Spaendonck's poppy), reduced, in a cream mat with a bevelled window (its foot wider), under glass, in a "20 x 30" frame of a narrow flat moulding (16-20 mm faces, 14-20 mm deep), on screw eyes and a wire | 200 x 300 mm overall (the old frame's), a 16 mm moulding 18 mm deep, the window 118 x 188 mm raised 4 mm | https://www.masterfoto.lv/en/photo-frames/18709-focus-tango-wood-black-20x30.html (a 20x30 frame: 19 mm face, 14 mm deep); https://artphotolimited.com/us-en/frames/20x30/oak-wooden-frame-20x30cm (oak, 20 mm rods, 20 deep); the plates: `assets/src/textures/sources.json` (`pictures`) |
| Gardening gloves (`place_gardening_gloves`) | A pair of cotton jersey gardening gloves with knitted wrists and PVC grip dots on the palm and fingers (Town & Country's "Cotton Grip" and the like; a medium 9 in long) | 230 mm from the wrist's mouth to the middle fingertip, the hand 90 mm across flat, fingers 19-22 mm, the knitted wrist 62 mm; dots 2.4 mm in staggered rows 5.5 mm apart | https://www.webbsdirect.co.uk/town-country-cotton-grip-gloves-red-medium/ (washable cotton, PVC dots on palm and fingers, a knitted wrist); https://www.walmart.com/ip/740829101 (cotton knit-wrist garden gloves, medium, 9 in) |
| Garden rake (`place_garden_rakes`) | A bow rake: a tempered steel head on two bows, 14 curved tines, a steel ferrule, a 54 in white ash handle (Ames/UnionTools 63107 and its like) | handle 1372 mm, 28.6 mm across; head 349 mm wide; tines 70 mm; 1.50 m overall | https://www.hanessupply.com/uniontools-63107 (14 tines, 54 in hardwood handle, 13 3/4 in head); https://www.lawsonproducts.com/products/54-bow-rake-white-ash-straight-handle-14-tines-13-3-4-width-1283127 (white ash, 2 3/4 in curved tines, a steel ferrule) |

## The parts (`t17.py`)

- `Real__acornNut` (a bare nut lying along x: a lathe of the real profile,
  blunt at its scar, widest a quarter of the way, a rounded cone to its
  style), `Real__acornPairNut` and `Real__acornPairCup` (a nut seated in
  its cup, laid down at the turn that holds its centre of mass lowest: the
  build prints the turn and the cup foot's place, `PAIR_*` in `t17.rs`),
  `Real__acornCup` (an empty cup mouth up, tipped 0.22 rad). The cup's
  inside follows the nut's profile; its outside carries 6 rows of 13
  shingled scales (raised toward their tips), the stalk's stub under it.
- `Real__conker` (the seed on its flatter side, lumpy as a seed pressed in
  its capsule, the hilum a little flattened with a low ridge round it, a
  small beak at its edge), `Real__conkerSeated` (the same, sitting in an
  opened husk), `Real__conkerHuskOut` (three valves split along their
  meridians and turned back 0.55-0.78 rad about the stalk's pole, the
  husk tipped 0.3 rad onto one; their warty outsides, 11 stout prickles a
  valve curving toward the apex, the stalk), `Real__conkerHuskIn` (the
  insides and the split edges, 5 mm of wall).
- `Real__pineCone` (the axis, the stalk and 58 woody scales on the golden
  angle, opened 68 degrees at the base to 22 at the tip) and
  `Real__pineConeApo` (each scale's rhombic end, keeled across, the umbo
  and its prickle in its middle, pyramidal toward the base): lying on its
  side.
- `Real__pineNeedlesA..C` (two pairs and a single, each needle a twisted
  half-round sweep tapering to its point, lying flat), `Real__pineSheathA,
  B` (their papery sheaths).
- `Real__fernFrond`: the blade over its outline's envelope (the segments'
  layout, `fern_seg`, is the game's too), lying in x-z, raised in y only (a
  ridge along the rachis, the segments arching to their tips on the floor,
  their margins waved, crumpled): seen from above it is the flat outline
  the game paints. `Real__fernStipe`: the rachis on it and the stalk to
  where it broke.
- `Real__botFrame` (the moulding's profile swept round 200 x 300 mm,
  mitred; its face `BT_FACE`, which the game's finish follows),
  `Real__botMat` (1.5 mm card, its window cut with a bevel),
  `Real__botGlass`, `Real__botPaper`, `Real__botBack`, `Real__botWire`
  (two screw eyes, the wire drawn up to the hook, its ends twisted back).
- The gloves: one glove is a closed shell over its outline (a signed
  distance field of the palm, the wrist and each digit, traced by marching
  squares and filled by a constrained Delaunay triangulation, 6.2 mm
  apart), its rest shape as a hand fills it, the rim (its seam) shared by
  the palm and the back. Blender's cloth solver (headless, 150 and 200
  frames, self-collision) drops the right glove fingers first from a hand's
  height, flattened to a quarter of its rest thickness and a little lumpy,
  onto the floor; then the left (the right mirrored and turned palm up)
  across the right's knuckles at 2.2 rad to it, meeting the right's back.
  Each settles limp, its slack in soft folds. `Real__gloveRBody`,
  `Real__gloveRCuff`, `Real__gloveLBody`, `Real__gloveLCuff`: the jersey
  and the knitted wrist of each.
- `Real__rakeHandle` (the ash handle, its end in the ferrule, its butt
  rounded) and `Real__rakeSteel` (the ferrule crimped onto the bow's stem
  and riveted, the two bows from the stem out and up to the head's ends,
  the head bar, 14 tines curving back to their points): the rake built in
  its tool frame, then laid on its tines' points and its handle's end
  (tipped 0.043 rad), as one dropped lies.

Every part is within its budget (three a few percent over their declared
budget, warnings only); 19,555 triangles in all.

## In the game

- **Acorns**: the old acorn's colour picks the nut (ripe brown and a
  little glossy, old and dark, dried pale), its tilt whether it lies bare
  (two in five, one in three of those holed by a weevil), in its cup (two
  in five) or is an empty cup; where it lies and its turn as drawn, kept a
  little apart from those already down (no draw). Each part's map is laid
  on by its own axis (`uv_part`): the nut's from the middle of its scar
  round its side to its style (fine stripes along it, the scar pale and
  rough inside a dark ring, the tip's felt grey, blotches of decay, dust on
  top); the cup's round it and up its outside or inside (the scales as the
  model's, each a little out of line, paler toward its tip, its lower edge
  shadowed; the inside pale and silky, a ring where the nut sat).
- **Chestnuts**: the old 60% husk draw now sets the conker in its husk
  split open; where each lies (the old spread doubled: a husk is 6 cm
  across) and its turn as drawn, kept apart. The conker's map is laid on by
  the direction from its middle (glossy mahogany with paler wavy bands
  round the scar like a grain, the scar pale buff and pitted inside a dark
  edge, dust on top), fresh and glossy, dried dull or old and wrinkled by
  where it lies; the husk's outside dried brown leather with olive left in
  patches, warty; its inside pale tan browned in patches.
- **Pine cone**: the old colour's draw now weathers it (brown, darker, grey
  and dry, dark with damp) as a tint of the scales' red-brown wood and
  their grey-brown ends.
- **Pine needles**: each old needle is a pair in its sheath or, every third,
  a single broken from its pair, where and turned as drawn, its old green
  now the shed needle's colour (rust brown, grey-brown, dark with damp);
  under them a drift of broken needles and dust painted as a mark (`r07`),
  soft-edged, from where it lies (its strokes on a stream of their own,
  mixed with the place).
- **Fern**: the old colour picks how far it has dried (green browning at
  its tips, withered olive, dead brown); the twelve old sway phases are
  drawn as before, the first now turning it onto its back one in three
  (its sori up), the second its shade. The frond is painted once a state
  and side in its blade's top view: the segments' outline cut out (alpha
  test), their midveins and forked side veins paler, the margins browned,
  the tips dried, holes; on its back the sori in rows. **Kept behaviour**:
  the old leaflets swayed in a draught; the frond lying on the
  floor now turns a hair (0.012 rad) about its blade's base at the old
  rate, its phase from the third old phase draw.
- **Botanical**: `visuals::pictures` paints it as `t02`'s print: the plate
  (by where it hangs) laid in its paper and aged as an engraving (browned,
  foxed, now and then a tide line), the mat yellowed toward its window and
  its bevel browned, the glass's dust and smears, the moulding's finish
  (stained oak, walnut, mahogany or ebonised, rubbed on its crests) by
  where it hangs. It hangs on its wire from a brass hook at 1.65 m, as
  the old one's middle; the old tilt's draw now swings it about the hook
  (the old turned the whole group about the floor under it).
- **Gloves**: the old colour picks the jersey's dye (sage green, brick
  red, olive, brown, oatmeal), faded; the wrists a darker rib. Each glove's
  jersey is painted from above over its part's box (the knit's wales,
  fibres, fading, soil dried in and ground into the fingers' end, grime
  where it rubbed); the left's palm (up) carries its PVC dots in staggered
  rows, worn away in patches; the wrists' 1x1 rib. All from where it lies.
- **Rake**: its enamel (green, black, red oxide, or none left) by where it
  lies, chipped to rust; the handle's ash gone grey, its grain in long
  streaks, dark where the hands held it, stained at the ferrule.

## Draws

Every old draw is kept, in order (the comments at each say what it now
chooses, or that it is kept for its place in the stream). Checked with
`--dump-level --flags noinstance` on two floors a placer (acorns 5010/13,
5030/5; chestnuts 5014/14, 5029/12; pine cone 5022/14, 5023/13; needles
5028/5, 5028/13, 5023/13; fern 5018/11, 5028/10; botanical 5002/13, 5008/9;
gloves 5007/9, 5009/11, 5014/14, 5023/13; rake 5005/10, 5010/11): the same
draws at every mark, the same `solidsHash`, every other placer's objects
(placer, position, bounds, turn) the same. (On 5028/13 the paper plates of
batch `t13` differ in their bounds before and after: their parts were not
yet in the committed `prop_meshes.bin`, and `--write` built them; not a
draw.)

## What the look cannot do

- The acorns, the conkers and the needles are a few centimetres: from the
  census's camera they are a few pixels; the close cameras show them.
- The jersey's knit and the cup's felt are painted, not modelled; at arm's
  length the knit reads as fine lines.
- The camera's near plane is 5 cm: a nut seen from closer is cut open.
