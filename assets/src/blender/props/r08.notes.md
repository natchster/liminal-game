# r08: the everyday litter (Props P1)

Six placers the census graded `cartoon`, remodelled from the real things
(`docs/design/PROPS.md`). The parts are placed as modelled
(`prop_part_as_modelled`, a scale of one); each placer keeps its draws call
for call (checked with `--dump-level` on its evidence floors: the same draw
count, every other placer's objects the same).

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Scrap paper (`place_scrap_papers`, p01) | A sheet of office copy paper, ISO 216 A4, 80 g/m2 (about 0.1 mm thick), balled in a fist, or crumpled and smoothed flat again | A4 210 x 297 mm; a tight ball 65 mm, a loose one 80 mm | ISO 216 (A series); 80 g/m2 copy paper is about 100 um: https://www.cartridgesave.co.uk/find-paper-and-labels.html, https://resource.download.wjec.co.uk/vtc/2016-17/16-17_1-4/website/category/5/1papers_and_boards/category_and_standards/index.html |
| Chalk marks (`place_chalk_marks`, p01) | Blackboard chalk (calcium carbonate) dragged over a wall: a stick 10 mm across, 80 mm long, its stroke 6-8 mm wide, broken where the wall's tooth skips it, its dust falling below | stroke 8 mm; a mark 19-25 cm across | Dixon Prang dustless chalk, 3.25 x 0.38 in (82.5 x 9.5 mm): https://supply-essentials.classwallet.com/chalks/11514-hygieia-dustless-board-chalk-3-1-4-x-3-8-white-12-box-072067311443.html; Giotto Robercolor 9.6 x 80 mm: https://eezee.sg/product/giotto-robercolor-premium-blackboard-chalk-100-pack-c60421286d |
| Trash can (`place_trash_cans`, p31 `floor.rs`) | A round steel office wastebasket, Rubbermaid Commercial WB26 class (fire-safe, powder-coated steel, 26 qt / 6.5 gal), with a black liner turned over its rim; a small tied bin bag beside it | 13.5 in (343 mm) across, 14.5 in (368 mm) tall | WB26BK: https://aztekc.com/product/1i5c95vidl4; WB26AL: https://www.bluedogink.com/rubbermaid-commercial-fire-safe-wastebasket-round-steel-65-gal-almond-wb26al.html |
| Single sock (`place_single_socks`, p19) | A men's cotton crew sock, lost and lying limp: knit body, reinforced heel and toe, ribbed cuff (a sports sock's two bands) | foot 23 cm, leg (heel to cuff) 20 cm, 8.6 cm across flat, cuff 5 cm | crew height about 10.5 in heel to cuff: https://theoutfitters.nf.ca/products/the-standard-crew-lightweight-mens; 6-10 in: https://custom.sockclub.com/blogs/crew-socks-height-guide |
| Bottle caps (`place_bottle_caps`, p19) | Crown caps (the "26 mm" crown for the 26 mm bottle finish): tinplate, 21 flutes, a plastic liner, the printed top lacquered | 32.1 mm across and 6.0 mm tall as made (inside 26.8 mm); prised off a bottle, its skirt still crimped, about 29 mm across | Tapon Corona finished-product sheet, 26 mm pry-off crown, 21 flutes, height 0.236 in, outside 1.262 in: https://beermex.odoo.com/web/content/product.template/7382/x_studio_binary_field_1kikp?download=1; 21 flutes standard: https://patents.google.com/patent/US5458253 |
| Crushed soda can (`place_crushed_soda_cans`, p20) | A 355 ml (12 fl oz) two-piece aluminium can, 211 body necked to a 202 end, crushed underfoot | body 66 mm across (2 11/16 in nominal), 122 mm tall, end 54 mm across the double seam | Sencon DWI can dimensions: https://www.sencon.com/en/support/techtips/Canmaking_dimensions_DWI.php; TricorBraun can FAQ: https://www.tricorbraun.com/blog/aluminum-can-dimensions-faq.html |

## The parts (`r08.py`)

- `Real__paperBall`, `Real__paperWad`: a hull of about a hundred points
  pushed in and out, pockets folded in, crease planes, two to four corners of
  the sheet lifting; shaded flat (paper has no round in it). Within their
  size budgets (252 and 296 triangles).
- `Real__paperSheet`: A4 crumpled and smoothed out: straight creases, the
  old middle fold, a corner curled up. Printed in the game (a stock count
  sheet, a shoe's dusty tread across it).
- `Real__wastebasket`: one sheet of steel: the bottom raised in its chime,
  two pressed beads, the wall rolled over a wire at the rim, the locked side
  seam standing proud, a kick's dent mid-wall (rows inside and out so the
  dent bends the wall and nothing pokes through). Its own budget, 1800: the
  rim, chime and beads are what read as a wastebasket.
- `Real__wastebasketLiner`: the black liner lining it, turned out over the
  rim and hanging down outside, its hem uneven; dented deeper than
  the wall so it never shows through it.
- `Real__wastebasketTrash`: what is in it when it stands: crumpled paper
  pressed down to a little over half its height, balled sheets on top.
- `Real__trashBag`: a small bin bag full and tied: slumped, its film drawn
  into pleats at the neck, the knot and its two ears.
- `Real__sock`, `Real__sockHeelToe`, `Real__sockBands`: a flattened tube
  along an L (cuff, leg, heel turned at its outer corner, foot to a blunt
  toe), wrinkles across it, the cuff's mouth turned in; the reinforced heel
  and toe as a skin over the knit; a sports sock's two bands.
- `Real__capCrown`, `Real__capTop`, `Real__capLiner` (and `…Bent`): the
  fluted skirt (21 flutes), the knuckle, the printed top panel, the liner
  inside; the bent ones levered up by the opener about a crease, the flute
  it caught opened out.
- `Real__canCrushedBody`, `Real__canCrushedEnds`: stood on its end and
  stamped flat: the wall folded into three rings out of step round it, the
  lid slid off centre and tipped, the bottom dome; the lid with its double
  seam, countersink, the drinking hole pushed in and the tab.
- `Real__canFlatBody`, `Real__canFlatEnds`: stepped on lying down: the
  middle pressed flat, the stiff ends half squashed.

Parts over the size budget (`budget_for`) carry their own on a `part('…'`
line, so the asset lint reads it.

## In the game

- **Scrap paper**: the old crumple's x size picks a ball, a loose wad or a
  flat sheet; its z size scales a ball within ±10%. Paper `#e4e0d4` with
  fragments of printed type turned every way, crease grime, floor dust.
- **Chalk**: a stranger's hand, never hers or the place's (LORE: her marks
  are a lone 8 cm stroke low on a wall, her tallies, open arrows; the
  secrets' sign is a small circle with a point, scratched): a number
  underlined, a looping scribble where someone tried the chalk, or a
  fitter's measure (a line with a tick at each end, its length in
  centimetres over it). The old streak's width picks the kind, its height
  the size. Strokes are chalk grains caught on a fixed wall tooth, heavier
  where pressed, broken where they skipped; faint dust under them. White,
  tinted by `chalk_color_by_weather` as before. The mark is laid in upright
  strips each under 8 cm wide, each at the old streak's place: settling
  probes a wall face only at the middle of anything narrower than 8 cm, so
  every strip settles exactly as the old streak did and no wall face is
  pinned that was not before (a single 22 cm plane pinned one on seed 202
  level 10 and moved the mark 1.4 m). Where the wall's face steps within the
  mark's width near the wall, the mark is drawn smaller.
- **Trash can**: with two pieces of paper by it (the old spill count) it
  stands, full of crumpled paper; with three it lies knocked over, the paper
  spilled out of its mouth. Paint by `vent_color_by_weather`, mottled,
  scratched low, rust at the chime, the rim and down from the seam; the
  liner in the weather's bag film. The bag beside it as before (its distance
  kept; the placement pass keeps it clear of the can).
- **Sock**: the old colour picks a sock: a white sports sock gone grey
  (darker heel and toe, navy bands), black, burgundy, navy, grey marl
  (grey bands). A knit texture: wales, fibre flecks, grey washes, the dirt of
  a floor heaviest at heel and toe.
- **Bottle caps**: the old colour picks one of five printed tops (no real
  brand: LAGER, PILSNER, SODA, EXPORT, BEER), faded, rubbed, scratched to
  the tin, the bottle's lip pressed in a ring; the skirt's flute crests
  rubbed bright, rust at the cut edge. The old 25% edge-on caps lie on their
  backs showing the liner; a third are bent (from where they fell, no new
  draw); each is moved clear of those already down.
- **Crushed can**: the old colour picks a plain label (COLA, CLUB SODA,
  LEMON-LIME, ORANGE, SPARKLING WATER; no real brand), its band, 12 FL OZ
  (355 mL), a nutrition panel and a barcode; faded, scuffed, scratched to
  the aluminium and cracked bare along its folds. Where it lies picks
  stamped on its end or stepped on lying down (no new draw).

## Left as it was

- Seed 202 level 10: settling moves the chalk mark (as it moved the old
  streak) 1.37 m off the wall into the wall tile, where it hangs in the air
  facing away; the settle pass, not this placer, puts it there.
- Seed 1 level 12: the placement pass now finds room for the bottle caps it
  used to leave out, and nudges the larger sock 11 cm; no draw changes.
