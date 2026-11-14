# t07 notes: P3, the rare: kitchen, hall and floor things

Eight placers the census graded `cartoon`, made again as the real things
they stand for (`docs/design/PROPS.md`). The parts are placed as modelled
(`prop_part_as_modelled`, a scale of one). What an earlier batch already
modelled is placed, not modelled again: the opened suitcase is r02's case
and clothes, the coats on the rack r06's overcoat and r13b's scarf, the
umbrellas in the stand r09's, the boots on the tray r14a's. Each placer
lives in a `t07.rs` of its own crate (`placers_p00`, `p08`, `p10`, `p11`,
`p21/src/placers/`) and keeps its draws call for call.

The census's verdicts: the ashtray "a pale pink-grey ashtray ... a
translucent flat oval for smoke, and an orange-red glow thrown across the
carpet ... the ember lights the floor like a lamp" (review2); the sieve "the
mesh drawn as an olive disc crossed by two lines each way (a tic-tac-toe
grid)" (review1); the coat rack "each coat is a flat slab cut straight
across the top, with no shoulders, collar or hanger" (review4); the boot
tray "four identical orange-brown boots ... a bare tube with a rounded bulge
at the foot ... like test tubes" (review6); the colander "a pristine white
bowl ... its perforations are five irregular black blotches" (review6); the
opened suitcase "a black slab base with a slab lid ... a saturated magenta
lump. It reads as a jewellery box of primitives" (review5); the umbrella
stand "three thin bare rods ... It reads as a quiver of arrows" (review5);
the vacuum "a saturated blue short body on a base, a pencil-thin rod for a
handle ... Wrong proportions and a toy colour" (review5).

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Smoking ashtray (`place_smoking_ashtrays`, p00) | A round pressed-glass restaurant ashtray, four rests across its rim; a king-size filter cigarette left burning in a rest | 108 mm across, 25 mm tall; the cigarette 84 x 7.8 mm, its cork tipping 25 mm (35 mm of tobacco left) | Libbey 5156, 4 1/4 in, 0.98 in tall, rests in the rim: https://www.webstaurantstore.com/libbey-5156-4-1-4-round-ashtray-case/5515156.html; Libbey 5154 Nob Hill, 4 in, 1 1/4 in, four rests: https://www.tigerchef.com/libbey-5154-ashtray.html; king size 84 mm, 8 mm, a 25 mm filter: https://www.paperguru.de/OCB_1:1-1/Dark-Horse-King-Size-Filter-Tubes-Extra-Long-Filter-24-mm-long-Filter-200-Tubes-per-Box |
| Hanging colander (`place_hanging_colanders`, p08) | A two-handled colander on a foot ring, stamped and punched in rings: graniteware enamel, stainless or tin-lined copper, hung by an ear's hole on a nail | 24 cm across, 11.6 cm tall with its foot; holes 4.8 mm about 10 mm apart; ears 40 mm with a 13.6 mm hole | Enamel colander, 24 x 12 cm, two handles and a base: https://www.dille-kamille.com/colander-enamel-black-white-00020806.html; Emalia footed enamel colander, 24.5 cm, 26.5 cm over the handles: https://www.thebigkitchen.co.uk/?p=1247019 |
| Herb sieve (`place_herb_sieves`, p08) | A fine-mesh strainer: a tinned band round a woven wire bowl, a looped two-rod handle, a pan hook opposite; hung by the loop | 7 in (178 mm) across the band, 78 mm deep; the handle 135 mm to its loop | Matfer Bourgeat 020426, 7 in mesh strainer, looped handle and pan hook: https://www.webstaurantstore.com/matfer-bourgeat-7-stainless-steel-fine-mesh-strainer-020426/98020426.html; a 7.67 in (19.5 cm) mesh, a loop to rest it: https://www.canadiantire.ca/en/pdp/stainless-steel-long-handle-strainer-7-in-0421408p.0421408.html |
| Coat rack (`place_coat_racks`, p10) | A 24 in board of stained hardwood with five cast brass hooks, screwed to the wall | 610 x 90 x 19 mm, hooks 120 mm apart projecting 95 mm, the cradle 77 mm out from the wall | Richelieu 5-hook wooden rack: 24.0 in wide, 3.7 in projection, 5.57 in tall: https://build.com/richelieu-rh111210590195/s1231157; a 24 in (610 mm) board, 5 hooks: https://www.homedepot.com/p/301412542 |
| Vacuum cleaner (`place_vacuum_cleaners`, p10) | An upright of the Hoover Convertible kind: a hood with a headlight and a vinyl bumper over the brush, a chromed handle on a bail, a cloth outer bag on a hook, the cord wound on two hooks | 45 x 13 x 12 in (1.14 x 0.33 x 0.30 m); the hood 0.32 x 0.28 m, the bag 0.78 m long | Hoover Convertible with headlight, 45 in H x 13 in W x 12 in D, a reusable cloth bag, a metal handle with a cord wrap: https://spencertified.com/products/hoover-convertible-with-headlight-vintage-upright-vacuum-cleaner; Convertible bag jackets: https://vacuumland.org/threads/post-wwii-eureka-f-g-and-hoover-convertible-bag-jacket-sizes.45203 |
| Boot tray (`place_boot_trays`, p11) | A moulded rubber boot tray, a raised lip, ribs across its floor; a pair or two of work boots on it | 32 x 16 in (810 x 410 mm), 24 mm deep; the boots r14a's (310 mm) | Mohawk Home rubber boot tray, 32 x 16 x 0.31 in, raised ridges inside: https://homedepot.com/p/Mohawk-Home-Rubber-Black-16-in-x-32-in-Boot-Tray-Mat-838447/322838775; Lee Valley, 32 x 16 x 1 1/4 in: https://www.leevalley.com/en-us/shop/home/clothing/footwear/accessories/50056-rubber-boot-tray; Project Source, natural and recycled rubber, 16 x 32 in: https://www.lowes.com/pd/Project-Source-16x32-Rubber-Boot-Tray/5014496617 |
| Umbrella stand (`place_umbrella_stands`, p11) | A rolled sheet steel umbrella stand, a bead at its top, swaged ribs, its lap seam down the back, a pressed bottom | 200 mm across, 490 mm tall | vidaXL steel umbrella stand, 20 x 48.5 cm: https://vidaxl.com.au/e/vidaxl-umbrella-stand-umbrellas-steel-white/8718475613145.html; SONGMICS steel stand, 17.5 x 49 cm: https://www.therange.co.uk/furniture/hallway-furniture/hallway-storage/round-metal-umbrella-stand-in-black |
| Opened suitcase (`place_opened_suitcases`, p21) | r02's 1960s moulded hard-shell case (Samsonite Silhouette class) lying open, gone through | 0.584 x 0.406 m, as r02 | docs/research/props-remodelling/book_stack_suitcase_coat_stand_and_pile.md |

## The parts (`t07.py`)

- **The ashtray** (`Real__ashtrayGlass`, `…Ash`, `…ButtFilters`,
  `…ButtPapers`, `…ButtChar`, `…CigFilter`, `…CigPaper`, `…CigEmber`,
  `…CigAsh`): the glass a lathe of its section (the foot and the dish
  under it, the thick bottom, the bowl, the flat rim with its rounded
  edges), its four rests cut across the rim; the ash a lumpy bed in the
  bowl with flakes and a fallen column; three butts stubbed out, their
  paper crushed flat and kinked, their ends charred; the cigarette lying in
  the rest at 45 degrees, its filter out over the rim, tipped 10.6 degrees
  down so its ash rests on the ash, a ring of ember between its paper and
  its column of ash (cracked in rings, drooping).
- **The colander** (`Real__colanderBowl`, `…Rim`, `…Nail`): the bowl one
  sheet (a lathe of its profile, rings across its bottom so the holes laid
  there stay round), a knock in its shoulder; registered in a 'sphere'
  frame so its texture is laid round its axis: the game punches the holes
  out of that texture in rings, as they were punched (alpha tested). The
  rim: the rolled bead, the foot ring, the two stamped ears (their sides
  pressed up into a rib, their roots bent down onto the wall and riveted
  twice). The nail: a wire nail, its head, driven 8 degrees up.
- **The sieve** (`Real__sieveMesh`, `…Frame`): the mesh one sheet (a
  dome, sagged where things were pressed through it), laid from the front
  so its weave is a square grid; the band (rolled, 7 mm), the handle's two
  rods converging into its loop, the pan hook bent in a U under the band.
  It hangs as a sieve does on a nail: the loop over it, the band against
  the wall, the bowl out.
- **The coat rack** (`Real__coatRackBoard`, `…Screws`, `…Pegs`): the board
  rounded on its edges and ends, a bead routed along its face, cupped a
  little; two countersunk screws; five cast hooks (a round rose, an arm
  out and down to its cradle, turned up to a ball). The cradle is where
  r06's hook has its own, so r06's coat and r13b's scarf hang there as
  modelled.
- **The vacuum** (`Real__vacuumHood`, `…Rubber`, `…Lens`, `…Metal`,
  `…Grip`, `…Bag`, `…FillTube`, `…Cord`, `…Plug`, `…PlugBlades`): the hood
  lofted over its plan (rounded at its nose, low over the brush and up over
  the motor), the headlight's housing, the bail's sockets, the motor's vent
  ribs and the fill tube's collar; the bumper round it and four wheels; the
  bail, the tube (leaning back 5.3 degrees), the bag hook and its clamp, the
  two cord hooks; the grip bent back, ribbed, the switch housing at its
  foot; the bag lofted from its foot on the fill tube to its hem on the
  hook, flat against the handle behind, a sack in front, sagging full low
  down, its slack in long folds, an eyelet in its hem; the fill tube's
  elbow and clamp; the cord wound four times round the hooks and let down
  from the lower one (`CORD_FREE`); a moulded plug and its blades.
- **The boot tray** (`Real__bootTray`): the lip a band round its rounded
  plan, bevelled; the floor; 19 ribs across it; trodden a little dished, a
  corner kinked up.
- **The umbrella stand** (`Real__umbrellaStandBody`): one lathe, inside
  and out, of 0.8 mm sheet: the bottom seamed on (its rolled foot bead),
  two swaged ribs, the top's rolled bead; the lap seam down its back;
  three knocks. Registered in a 'cyl' frame: its finish is laid round it.

## In the game

- **Ashtray**: the old glass colour picks the glass (clear with a green
  edge, amber, smoke, blue-grey), transparent. The ash painted (pale fluff,
  burnt flecks), the filters cork tipping. The ember glows (a flame); its
  light is a faint glow on the ash (it was a lamp's on the carpet). The
  smoke: two crossed sheets where the old two wisps were, a thread painted
  rising straight from the burning end, then wavering, widening and curling
  as it fades; each wisp's three old draws (a nudge and a phase) set where
  it stands and how it sways; the old ember phase still flickers the glow
  and the ember's colour.
- **Colander**: the old metal draw picks the kind (stainless gone dull,
  tin-lined copper tarnished brown, graniteware: blue-grey enamel speckled
  white); the old 14 holes' 28 draws place its chips (the enamel to the
  black iron, rust round them) and knocks. Hung by an ear's hole on the
  nail (1.86 m), its rim against the wall, its bottom out.
- **Sieve**: the old metal draw picks the wire (stainless, brass wire,
  tinned steel gone dull). The weave painted (over and under, about 1.4 mm),
  the wall seen through it, flour and dried leaf caught in it. Hung by its
  loop on a nail (1.84 m).
- **Coat rack**: the old wood draw picks the stain (mid oak, walnut,
  golden oak, dark); the board painted with its grain, the varnish rubbed
  pale round each hook, grime in the bead and along its foot. The old
  coat draws (the hook, its colour, the 35% sway) hang a wool overcoat on
  a middle hook (r06's, in the old colour as wool: oxblood, navy, loden,
  dark brown, burgundy, slate, camel, charcoal), one hung over another
  lying on it (pushed out below its loop); on an end hook, a knitted
  scarf instead (r13b's): an overcoat there stood out past the board, and
  where the rack hangs on a partition by its end the wider rack's probes
  missed the face and pinned it (the walls changed on 4/6 and 5/12 until
  it did not). The old optional scarf (40%) is r13b's scarf, in the old
  scarf colour.
- **Vacuum**: the old body colour picks the hood's plastic (burgundy, slate
  blue, grey, black, harvest gold, avocado), dusty, scuffed, grimed along
  its foot, and the bag's cloth (twill, two printed bands rubbed thin, the
  zip's track, grey with dust low down, dark at its foot). The old cord
  end's two draws: where its loose end, let down off the lower hook, trails
  to on the floor (a tube along a curve, built in the game), and its plug.
- **Boot tray**: the old colour picks the rubber (black, brown, charcoal,
  blue-black), painted with grit, the road salt's tide lines and mud along
  its ribs; the old 60% wet: dirty water standing between the ribs (a
  transparent painted sheet under the ribs' tops). The old pairs draw: one
  or two pairs of r14a's work boots, the left of each mirrored from the
  right (its triangles turned back out); the second pair's right one fallen
  on its side (r14a's lying boot, its lugs showing); each pair's old colour
  picks its leather (oxblood, dark brown oiled, wheat nubuck, black).
- **Umbrella stand**: the old colour picks the finish (hammered brown
  paint, charcoal, black enamel, lacquered brass gone tarnished,
  galvanised); painted chipped at its rim and foot (not the brass),
  rusting from the foot, dried drips run down, dust on its bead. Each
  umbrella's four old draws: its cloth (the old colour faded toward a dusty
  grey), its crook (dark wood, light wood, black, chrome), how far it leans
  out and which way (stood on its ferrule on the bottom, leant till its
  canopy meets the rim; less toward the wall), its crook turned to the
  room.
- **Opened suitcase**: the old leather draw picks the shell (saddle,
  oxblood, tan, black); the lining painted (weak tea, its stripe, tide
  marks, grubby edges). The old 4-6 cloths are r02's clothes in their slots
  (trousers, a jumper, a shirt on the trousers, socks in a corner; a fifth
  a cardigan pulled out over the rim; a sixth another jumper on the first),
  each cloth's old colour its palette's, its old sizes, place and turn a
  nudge and a turn, the first's tilt the side it was packed from.

Every painter draws on a stream of its own, its strokes varied by where
the thing is (no level draw). Materials are lambert or phong, their colour
painted; nothing emits but the ember.

## Checked

Every one of the 272 census floors dumped (`--dump-level`) with the build
before and after: the same draws on all 272, every other placer's objects
the same, `solidsHash` and every count the same. These props themselves
stand where they stood, but for the opened suitcase on 9/12: placed by the
floor hole there, the real case (0.58 m, where the old was 0.22) is moved
0.5 m off the hole's edge by the placement pass. Level build time the same
(best of three, 1.9-2.0 s for the dump of 31337/8, 11/11, 555/9, 11/14,
7/7 either way).
