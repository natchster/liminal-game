# r02 notes: the book stack, the suitcase, the coat stand, the pile

Props P1 (docs/design/PROPS.md), four core props graded `cartoon` by the census
(`docs/research/props-census/review1.md`, `review2.md`, `review5.md`,
`review6.md`), each made again as the real thing at its real size. The
parts are placed as modelled (`prop_part_as_modelled`) by their builders in
`crates/game/src/core_props/builders.rs`; every builder keeps its old draws
call for call (checked with `--dump-level`: the same draw count on (99, 14),
(1, 6), (11, 5), (555, 10), (7, 12) and on the reference levels of seeds 4242
(4–12) and 77 (4–8), and no object of any other placer changed).

## The book stack (`build_book_stack`)

The census: red, green and blue solid boxes with cream labels; no page
block, no overhang, no wear.

The real thing: cloth hardbacks of the old octavo sizes and a trade
paperback. Binder's greyboard is 2–3 mm
([hardcover anatomy](https://qicaipress-kz8ir.kinsta.page/blog/hardcover-anatomy/));
the boards stand past the page block by a square of about their own
thickness, 3 mm in a worked example
([Brighton bookbinding worksheet](https://blogs.brighton.ac.uk/viscommtech/bookarts/worksheets-for-bookbinders/hard-cover-for-multi-section),
[Edinburgh glossary, "squares"](https://lob.is.ed.ac.uk/index.php/concept/3816));
the spine of a sewn book is rounded and backed, its fore-edge concave
([PRC, square back vs round back](https://prcbookprinting.com/?p=6560)).
Sizes: demy octavo 222 x 146 mm (A, 30 mm of pages), a larger royal-ish
240 x 164 mm (B, 44 mm), crown octavo 192 x 128 mm (C, 21 mm); a trade
paperback 216 x 140 mm (8.5 x 5.5 in), 22 mm, 0.7 mm cover card.

Made: per hardback its boards (2.6 mm, fore-edge corners bumped round, the
upper board bowed up toward the fore-edge from damp), its rounded spine
(the cloth over the stiffener, into the joints) and its page block (3 mm in
at head, tail and fore-edge, the fore-edge concave and cockled); the
paperback's covers (the front lifting at the fore-edge, its tail corner
curled), square spine and swollen page block; a torn slip of paper kept as
a marker. Each book is three meshes so that each takes its own cell of one
512 px atlas painted per stack (`book_atlas`): the boards' cloth (weave,
faded toward the fore-edge, a darker patch where another book lay, a
blind-stamped rule, sometimes a gilt title, corners bumped through to the
board, edges rubbed, stains), the spines (faded more than the boards, title
and author stamped small in gilt or ink, rules at head and tail, frayed
ends, the gilt rubbed, dust on the upper edge; a paperback's printed spine
cracked along its length), and the page edges they share (yellowed leaves,
tanned toward the boards, foxed). Titles and authors are made up.

Draws: `irand(3, 6)` books; per book its three sizes (the width picks the
kind: a small hardback, the paperback, a novel, a big one; the thickness
scales it 0.94–1.06; the depth is drawn and unused), the cloth `choice`, the
offset, the turn and the ribbon roll (now the slip of paper); then the
standing book's roll and its five draws (now one more book laid on top,
turned across the rest, a paperback or a hardback by its lean).

## The suitcase (`build_suitcase`)

The census: an open box with plain slab walls holding smooth coloured blobs
that read as stones; no handle, latch or clothes.

The real thing: a 1960s moulded hard-shell case of the Samsonite
Silhouette class, 23 x 16 x 7 in, its shell covered in textured vinyl, a
fabric lining with elastic pockets and tie-down straps
([Thrift World, 1960s Samsonite Silhouette](https://thriftworld.com/products/vintage-60s-samsonite-red-silhouette-hard-shell-suitcase-mid-century-modern));
two metal latches with small locks at the front
([Kennesaw State, 1960s Monarch hard-shell suitcase](https://aegisdigitalmuseum.kennesaw.edu/items/show/248));
an aluminium valance (tongue and groove) round both rims, hinges at the
back, feet on the hinge side it stands on.

Made (0.584 x 0.406 m, base 112 mm deep, lid 66 mm): the base and the lid
shells (rounded moulding, a moulded rib, moulded feet), their linings (the
lid's with a shirred pocket and its elastic edge; the base's with the two
check straps that hold the lid open), the trim (valances, the base's
tongue, hinge leaves and knuckles, the two latches with their thumb slides
and lock barrels, the catches on the lid, the handle's lugs), the moulded
grip hanging down the front. The lid stands open 102 degrees on its
straps. In it: trousers folded in three (waistband, crease), a shirt
folded on its card (collar standing, placket, buttons), a knitted jumper
(ribbed hem and neck, the sleeves folded under), a pair of socks rolled
into each other, and a cardigan pulled half out, one sleeve over the front
rim down to the floor with its ribbed cuff (or, by its draw, still folded
on the jumper). The lining is painted (rayon gone the colour of weak tea, a
woven stripe, tide marks, grime in the corners).

Draws: the five (`irand(0, 3)`, `rand2` x 4) of the old five blobs, made
first and in the old order: each thing's colour from its own palette, its
size a small turn, its place a nudge within its slot; the first thing's
height says which side it was packed from, the last thing's height the
shell's colour and its place whether the cardigan was pulled out.

## The coat stand (`build_coat_rack`)

The census: the coat is one flat rounded slab like a tombstone, no
shoulders, sleeves or folds.

The real thing: a Thonet-style bentwood coat and hat stand, 74 in tall, an
18 in square base, four legs, a ring, two rows of six hooks
([Chairish, early 20th c. Thonet-style bentwood coat and hat rack](https://www.chairish.com/product/29413967/early-20th-c-thonet-style-bentwood-coat-and-hat-rack);
Thonet's own model 1 of the 1904 catalogue is 200 cm,
[Dorotheum](https://dorotheum.com/en/l/9159785)). A man's wool overcoat,
size 42: 46 in round the chest, 26 in sleeves, 36 in from the collar to
the hem
([Charles Tyrwhitt wool overcoat sizes](https://www.charlestyrwhitt.com/eu/it_IT/SS25-wool-overcoat.html)).

Made: the turned pole (collars at the legs and both rows of hooks, the
finial), the four bent legs on a 0.64 m spread with their feet turned up
and the ring tying them, six coat hooks (a J with a knob) and six hat
hooks (an S scroll) above them, in dark-stained beech. The coat hangs by
the loop in its collar from the front coat hook: the collar pulled up to a
point at the hook and open at the front, the neck fallen open in a V with
the lapels turned back, the shoulders falling away steeply with the drag
of the hook running out to them, the sleeves hanging at its sides (a cap
at the shoulder, wrinkles at the elbow, narrowing to the cuff), the body
heavy with folds deepening to an uneven hem, the left front over the right,
three horn buttons; the hem turned up inside. It is modelled as a surface
of the hanging coat, not draped: a cloth simulation (Blender's solver, a
coat tube and sewn sleeves pinned at the loop, tried at three stiffnesses)
crumpled and tangled round the pole in every run.

Draws: the one coat roll, as it was; its value picks the wool (charcoal,
navy, camel, brown, loden).

## The pile (`build_pile`)

The census: three smooth blobby boulders and a bare brown cube; no rubble,
grit or broken edges.

What it is: a pile is placed where things are stored (`STORAGE`: tunnels,
electrical, halls; the industrial theme): a heap of debris swept up there.
The real pieces: a modular brick, 194 x 57 x 92 mm
([BIA Technical Note 10](https://gobrick.com:443/media/file/10-dimensioning-and-estimating-brick-masonry.pdf));
1/2 in gypsum board (12.7 mm, a tapered factory edge, paper faced;
[USG Sheetrock panels](https://www.usg.com/en-CA/p/product/sheetrock-brand-ultralight-panels-141134));
a 2 x 2 ft lay-in mineral fibre ceiling tile, 5/8 in, fine fissured
([Armstrong Fine Fissured 1728](https://penntoolco.com/91519157)).

Made: the mound (0.56 x 0.47 m, 14 cm, its edge ragged where it was swept,
grit round it) and over it the bits of concrete, of plaster and of brick
(each a broken fragment: the hull of a few points, its faces flat and its
edges sharp, half sunk, never through the floor); the pieces the game lays
on it: a brick with chipped arrises, a half brick broken across, two lumps
of concrete broken along random planes, a piece of drywall (one factory
edge, the rest snapped and jagged, sagged), a corner of a ceiling tile (two
factory edges, sagged, its fissured face painted with a water stain); and
round it the pale dust it shed (a soft decal, grit in it, broom drags).

Draws: the old loop with its bound redrawn every pass; per piece `choice`
(a flat piece or a lump), its sizes and colour (which piece: drywall, tile
or brick; concrete, half brick or a small lump; and a scale of 0.92–1.08),
its place (laid on the mound's height there or on what was laid before it,
tipped to the slope, the broad pieces less) and its turn (a yaw and a
small tilt). The old y draw is drawn and unused.

## Placement

The positions are the placers' own; the engine's placement pass settles
each prop by its meshes. With the new meshes no object of any other placer
moves on the floors checked. Some of these props themselves now sit
differently from before: the old ones were lifted (their parts went through
the floor) and some nudged sideways by it, the new ones rest on the floor;
on 4 of the 14 reference levels a pile or a suitcase stands up to 0.16 m
from where the old one had been pushed to (the same spot for three sizes of
the mound tried, so the scatter's own).
