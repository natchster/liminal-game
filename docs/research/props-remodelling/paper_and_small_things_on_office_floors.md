# r15: paper and small things on the office floors (Props P2)

Eight placers the census graded `cartoon`
(`docs/research/props-census/review1.md`, `review4.md`, `review6.md`),
remodelled from the real things (`docs/design/PROPS.md`, "How a prop is
remodelled"). The parts are placed as modelled (`prop_part_as_modelled`, a
scale of one), each in its prop's own frame; each placer keeps its draws
call for call. Two reuse other batches' parts: the crumpled receipt is
r08's balled sheet (`Real__paperBall`, `Real__paperWad`, as
`place_scrap_papers` places them), the shut paperback r02's
(`Real__bookPbCovers`, `…Spine`, `…Pages`, as `build_book_stack` places
them), each with print of its own.

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Tissue box (`place_tissue_boxes`, p26 `toys.rs`) | A flat box of 160 two-ply facial tissues (the Kleenex Trusted Care class): a folding carton of printed board, its end flaps glued, a perforated oval torn out of its top, the tissues interfolded so the next one stands up out of the slot | 8.875 x 4.75 x 3.4675 in (225 x 121 x 88 mm); a sheet 8.2 x 8.4 in (20.8 x 21.3 cm) | https://www.kleenex.com/en-ca/products/trusted-care-facial-tissues-flat-box (the box); https://stopandshop.com/groceries/product/stop-shop-soft-gentle-unscented-2-ply-facial-tissue-flat-box-160-ct-box/158147 (the sheet) |
| Tape roll (`place_tape_rolls`, p19) | PVC electrical insulating tape (the 3M Temflex 1700 class), 3/4 in wide, 7 mil, 60 ft on a 1.5 in paperboard core; this one half used | 19 mm wide, 0.18 mm, the core 38.1 mm inside, 41.3 mm outside; the roll 60 mm across (a full one about 76 mm: 18.3 m x 0.18 mm wound on the core) | https://kele.com/product/electrical-wiring-materials/tape/3m/1700 (3/4 in x 60 ft, 1.5 in core); https://uk.rs-online.com/web/p/electrical-tapes/2469540 (19 mm x 20 m, 0.13 mm; the colours) |
| Playing cards (`place_small_card_stacks`, p24 `games.rs`) | A deck of poker-size playing cards: 52 and two jokers, 0.3 mm card, the corners rounded 1/8 in; a lattice back in red, pip cards with serif indices | 2.5 x 3.5 in (63.5 x 88.9 mm), 0.3 mm a card | https://www.qpmarketnetwork.com/card-deck/dimensions-of-a-playing-card/ |
| Crumpled receipt (`place_crumpled_receipts`, p19) | A till receipt on 80 mm thermal paper (55 g/m2) balled in a hand | a strip of 80 x 200 mm is a quarter of A4's area, and a crumpled ball's size goes as its area to about the 0.4: r08's ball of A4 at 0.57 (37 mm), its loose wad at 0.55 (44 mm) | https://neotech.ae/products/thermal-paper-roll-80mmx80m (80 mm, 55 gsm) |
| Paperback (`place_paperback_books`, p19) | r02's trade paperback (216 x 140 mm, 22 mm of pages, 0.7 mm cover card); open, it lies face up, its covers flat, the pages diving into the gutter | as r02 (docs/research/props-remodelling/book_stack_suitcase_coat_stand_and_pile.md) | docs/research/props-remodelling/book_stack_suitcase_coat_stand_and_pile.md |
| Newspaper stack (`place_newspaper_stacks`, p20) | A daily in the Berliner format folded once across, as it is sold: three to six of them kept in a pile | a page 315 x 470 mm, folded 315 x 235 mm, 6 mm thick at the fold | https://en.wikipedia.org/wiki/Berliner_(format); https://papersizes.io/newspaper/berliner |
| Paper aeroplane (`place_paper_airplanes`, p03) | The classic dart folded from a sheet of A4: folded in half lengthwise, the top corners to the middle twice (the edges then at 22.5 degrees to the middle, meeting the sheet's sides 253 mm down), folded in half, the wings folded down along a line from the nose to the tail | 297 mm long (the sheet's length), the keel 44 mm deep at the tail and nothing at the nose, the wings 61 mm each | ISO 216 (A4 210 x 297 mm); the folds: https://onelittleproject.com/wprm_print/how-to-make-a-paper-airplane-the-classic |
| Paper napkin (`place_small_paper_napkins`, p26 `party.rs`) | A two-ply luncheon napkin, 33 x 33 cm, sold folded in four, its border embossed | 330 mm; folded 165 mm square | https://www.packlinq.co.uk/products/napkin-2-ply-paper-1-4-fold-33x33cm-white-2-000-pcs-tk-9987623 |

## The parts (`r15.py`)

- `Real__tissueBoxTop`, `…Sides`, `…Ends`: the carton cut by face, so each
  takes its own cell of one atlas by its box's texture coordinates (the top
  panel 225 x 121 mm, a long side 225 x 88, an end 121 x 88): the top with
  the oval torn out along its perforation, the torn edge turned down into
  the box; the sides and the bottom and the corners' folds; each end the
  panel and the outer flap glued over it 0.5 mm proud, its edge 14.5 mm
  above the foot. One top corner trodden down (all three follow the same
  crush). `Real__tissueBoxInside`: the board's inside, facing in, seen
  through the oval. `Real__tissueBoxStack`: the tissues still in it, a
  little over half full, domed. `Real__tissueBoxSheet`: the tissue standing
  out of the slot, gone limp: pleated where the oval gathers it, leaning,
  one corner fallen onto the top, the other curled back, its edge uneven.
- `Real__tapeRollFaces` (the two wound faces, the inner turns 0.3 mm proud:
  telescoped), `Real__tapeRollWrap` (the outer wrap, its edges rounded),
  `Real__tapeRollCore` (the paperboard core, its top edge crushed),
  `Real__tapeRollTail` (the loose end peeled off and standing, curling out,
  its last 12 mm stretched to a point where it was torn). The roll lies on
  its face (the old one stood on edge and read as a red block); the faces,
  wrap and core are cylinder-mapped parts, so the faces take the disc.
- `Real__cardSingle`: a card, its corners rounded. `Real__cardDeck`: the
  deck under the loose cards, 46 cards in four bunches not quite squared
  (the edges' texture lines a card each).
- `Real__pbOpenCovers`, `Real__pbOpenBlock`, `Real__pbOpenSpread`: the
  paperback open face up: its covers flat (flush with the pages but for a
  sliver), the pages' two halves (head, tail and fore-edge faces, the
  fore-edge bulging), the spread over them dipping into the gutter, cockled,
  the recto's leaves lifting toward the fore-edge.
- `Real__newsBody` (the bottom and the fold's round), `Real__newsEdges` (the
  open edge, the top half 1.5 mm short of the bottom, its leaves not
  squared; the cut ends), `Real__newsFront` (the front page's top half over
  it), `Real__newsFrontCurl` (the same, its open corner curling up: the top
  paper's).
- `Real__paperPlane`: the dart: the keel's two walls opening a little up
  from the crease, the wings from the nose, drooping toward their tips (the
  left more); the nose crumpled back and folded over, the left wing's
  trailing edge curled up, the right tip curled down; pitched until its
  crease lies on the floor and rolled onto the left wing's tip.
- `Real__napkinCrumpled`: the folded napkin crumpled in a hand and
  dropped: seven soft folds drawing it in, the top flap lifting at its open
  corner. `Real__napkinWad`: scrunched into a loose wad (r08's crumple,
  looser and flatter), four corners coming away from it.

Every part is within its budget (5,366 triangles in all).

## In the game

- **Tissue box**: the old colour draw picks the print: cream with
  grey-green sprigs, pale blue with white waves, lilac with white flowers,
  grey with a fine stripe (the repeat drawn once and laid like a printing
  cylinder's). "Facial Tissues" in a serif italic on a label panel on both
  long sides, "160" and an EAN-13 bar code on the ends, the oval's printed
  ring and its torn perforation; the board worn at every fold, creased at
  the trodden corner, the floor's grime and a tide line up its foot, dust
  over all of it, heaviest on top. The old tissue's tilt draw turns the limp
  tissue in its slot. No maker's name.
- **Tape**: the old colour draw picks white, red, yellow or blue PVC, gone
  dull (phong, a vinyl's low sheen); the faces' turns in fine rings, grit
  and lint stuck to the adhesive edges, thickest at the outer turns; the
  core grey-brown kraft.
- **Cards**: with the old fan draw (three in ten) the cards are spread in a
  fan, backs up, the one under them face up; else the deck lies unsquared
  with the old loose cards slid on it by the old jitter draws, and the old
  suit draws turn its top card up (red or black). The back: a white border,
  a red lattice with a rosette in an oval (no maker's mark); a face: an ace
  to a ten, its indices in Times, its pips laid out as a deck's are, those
  below the middle turned. Grime at the edges, a crease across a corner.
- **Receipt**: thermal paper gone cream, patches of its print (items,
  prices, a total, "THANK YOU") in Courier on facets turned every way,
  faded brown; the creases' grime, floor dust.
- **Paperback**: with the old open draw (six in ten) it lies open; else
  shut, r02's paperback with a cover of its own: the old colour draw's
  ground, a light panel with the title in a serif, the author above, a
  small tree in a ring for the publisher's mark (no real imprint), faded
  from the head, creased at the spine, a cup's ring, "40p" in pencil; its
  spine printed; its page edges tanned. Open, its two pages are a quiet
  novel's (written for the game: a seaside boarding house; nothing in it
  about the place), set justified in Times with running heads (the author,
  the title) and folios, tanned toward the edges and the gutter, foxed, a
  tide mark at the recto's foot, grey with the dust of lying open. The text
  is set once a process (every copy lies open at the same pages); the rest
  is painted for each.
- **Newspapers**: each paper's old colour draw yellows it; the old slide and
  turn draws lay it askew, and turn it round when its slide went one way
  (its fold to the other side). The front page (the top paper's, and seen
  at the edges of those under it): an invented local daily ("The Saltham
  Gazette"), its number and price in the ears, the day's dateline and a
  weather line; a mundane lead across four columns (a bus timetable, a
  bridge repair, the market hall's roof, the rain), its photograph (a
  street of terraced houses, a tree, a parked car, a bus shelter) drawn in
  greys and screened into halftone dots, its caption; a second story and a
  short one; columns of body type in Times. Yellowed from the edges in, a
  cup's ring, dust.
- **Paper aeroplane**: the old colour draw picks the sheet: white is a typed
  page (minutes of a staff meeting: the kitchen rota, stationery orders),
  else blue, pink or green copy paper; dust, the nose grey where it hit.
  The old tilt draw is kept (it now lies on its keel and a wing tip).
- **Napkin**: the old colour draw picks white, cream, pink, blue, yellow or
  green tissue; the old crumples' count picks the napkin dropped (three) or
  scrunched into a wad (four), and their draws are kept; the tissue's fine
  emboss, the border pressed in a band, spots printed on a coloured one,
  the quarter folds, a smear of chocolate cake and a grease spot, dust.

Every painter is seeded by where its thing lies (a salt from its position),
so what is painted differs floor to floor without a level draw. Paper that
lay on the floor still lies there as it did (the napkin, the receipt, the
dart, the books, the newspapers); nothing is added to `Game::papers`, and no
loose sheet is added.

## Checks

- **Draws**: `--dump-level` on 272 floors (seeds 4242, 77, 7, 202, 303,
  1999, 11, 99, 555, 31337 over levels 4-14; seeds 1-20 less 7 and 11 over
  levels 6-14): the old placers and the new, both on the same
  `prop_meshes.bin`, give the same draw count on every floor, and every
  object of every other placer (placer, kind, position, turn, bounds, mesh
  count) the same. The eight props are placed where they were and turned as
  they were (the dart's old tilt alone is no longer applied); one open
  paperback (seed 2, level 8, by a wall) is settled 11 cm further along the
  wall than the old one, by the settle pass measuring the real book's
  larger bounds against the wall's face.
- **Evidence**: each prop's census camera and a closer one on its census
  floor, before and after; four sides of the variants the census did not
  show (the open book, the fanned cards, the deck with its top card turned
  up, the dropped napkin, a lilac tissue box, another stack of papers, the
  receipt's wad, the dart, the tape).
- **Build time** (`--bench`, warm builds, the old and new placers on the
  same meshes, interleaved): each adds 2-10 ms to its floor (the tissue box
  about 7, the newspapers about 10, the open book about 4: its text is set
  once a process).
