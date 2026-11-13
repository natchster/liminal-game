# t02 notes: P3, the rare: the maps, charts, prints and posters

The census graded seven placers `cartoon`, and they are made again here as
the real things they stand for (`docs/design/PROPS.md` 1, "its picture a
picture"): the cabin's framed map (`place_vintage_maps`, p06), the parlour's
antique map (`place_antique_maps`, p15), the office's framed map
(`place_framed_maps`, p13), the anatomy chart (`place_anatomy_charts`,
p13), the botanical print (`place_botanical_prints`, p16), the home-front
poster (`place_propaganda_posters`, p13) and the devotional triptych
(`place_religious_triptychs`, p22). Each placer lives in a module of the
batch's own (`t02.rs` in its crate; p06 `rustic/t02.rs`, p22
`placers/t02.rs`); the old texture painters they alone used are gone (the
p13 painters with them; p22's old triptych painter stays for its
crate's texture test). The parts are placed as modelled
(`prop_part_as_modelled`, a scale of one), every part in the frame of the
wall it hangs on (x across its face, y up from its middle, z out of the
wall, z = 0 the wall's face). Each placer keeps its draws call for call.

The census's verdicts: the vintage map "a framed white sheet with three
pale yellow Pac-Man-like polygons and a compass star ... clip art, not a
map" (review6); the antique map "ORBIS TERRARVM with two smooth yellow blobs
for land and a compass cross" (review3); the office's map "WORLD POLITICAL
over a white sheet with three pale yellow polygons and a disc for
continents" (review6); the anatomy chart "a stick outline (a circle head, a
rectangle torso, line limbs) with red dots and leader lines. It shows no
anatomy" (review3); the botanical print "clip art: a pentagon flower head,
a stick stem and two oval leaves ... pure clean white" (review7); the
poster "DUTY CALLS ... three box-bodied, round-headed pictograms like a
restroom sign" (review5); the triptych "halo-and-robe pictograms instead of
painted saints, in a flat yellow frame" (review7).

## The pictures

Every map, plate and painting is a real one: seventeen cells of the
pictures atlas batch `t01` began (`crates/game/assets/textures/pictures.jpg`,
now 1600 x 2940), from fifteen files released as CC0 by the Rijksmuseum and
the Cleveland Museum of Art, fetched from Wikimedia Commons and checked by
the SHA-1 Commons publishes (`assets/src/textures/sources.json`,
`pictures`: each one's file, page, record, maker, crop and cell; the
triptych's three panels are cut from one photograph, `same_as`). A plate
narrower than its frame's opening is fitted whole on its own paper (`pad`)
rather than cut. `visuals::pictures` paints each as it hangs
(`print_tex`), aged as an engraving on rag paper decades on a wall
(`Age::engraving`): browned all over to the tan of old paper (the museums'
scans are of sheets kept in the dark), yellowed more at its edges, foxed,
now and then a damp tide line from its foot or a crease across a corner;
the triptych as an oil (`Age::oil`: amber varnish, fine craquelure, grime).
Every pixel is computed on every core (`par_rows`), the strokes after on
the one thread, from a stream mixed with where the thing hangs
(`vary_of`), so no two prints are the same and no level draw is taken.

| Picture | What | Its record |
|---|---|---|
| map_paris | Paris and its villages, the siege of 1590, the Hogenberg workshop, Cologne, etched; plate 280 x 210 mm | Rijksmuseum RP-P-OB-78.709 |
| map_trier | Les Environs de Treves et de Consarbruch, for de Fer and Mortier, c. 1702, etched; plate 283 x 199 mm | RP-P-OB-83.034-131 |
| map_world | Blaeu's Nova totius terrarum orbis tabula, its border of planets, elements, seasons and wonders, coloured by hand, 1635-49; plate 545 x 413 mm | RP-P-1958-66 |
| map_dvina | van Keulen's sea chart of the Dvina at Archangel (Jan Luyken), coloured by hand; plate 593 x 515 mm | RP-P-1896-A-19368-2997 |
| map_sky | the Nieuwe hemel-kaart / Planisphère céleste after Pieter le Clerc, Covens & Mortier, 1780, coloured by hand; plate 583 x 668 mm | RP-P-AO-29-13A |
| map_mexico | Homann's Regni Mexicani seu Novae Hispaniae, coloured by hand, 1710-24; sheet 572 x 478 mm | RP-P-1896-A-19368-2990 |
| map_cook | the world in Mercator's projection with Cook's tracks (Cornelis van Baarsel after Henry Roberts); plate 884 x 580 mm | RP-P-2018-824 |
| map_antwerp | Plan de la Ville et Citadelle d'Anvers, lithographed, coloured by hand, 1830; sheet 581 x 434 mm | RP-P-OB-88.300 |
| anat_front, anat_back | Bidloo's Anatomia humani corporis (Amsterdam, 1685), plates T. 87 and T. 88: the skeleton with an hourglass by a tomb, and from behind; Pieter van Gunst after Gerard de Lairesse, etched and engraved; plate 276 x 442 mm | RP-P-OB-70.703, RP-P-OB-70.704 |
| bot_rose | Redouté's Rosa gallica agatha (Les Roses), stipple engraving printed in colour and finished by hand, 1817-24 | Cleveland Museum of Art 1943.139 |
| bot_pulmonaria, bot_magnolia | Ehret's Plantae Selectae nos. 42 and 33, engraved, coloured by hand, 1750-73 | Cleveland 1957.194, 1955.333 |
| bot_poppy | the opium poppy, Legrand after van Spaendonck (Fleurs dessinées d'après nature), stipple engraving, 1799-1801; sheet 333 x 508 mm | RP-P-1909-4229 |
| trip_l, trip_c, trip_r | Jan Provoost's triptych: the Virgin and Child, St John the Evangelist, Mary Magdalene, oil on panel, c. 1505-25, photographed open in its engaged frames; the centre 45 x 57 cm | SK-A-2570 |

None is a famous image. (The poster's rule, that it names no country,
party or person, is the poster's: an old map names its places.)

## What each is, and where its size comes from

| Prop | The real one | Size used | Source |
|---|---|---|---|
| Vintage map (a cabin's) | A loose plate from a 16th-17th century atlas or news-sheet series, the country round a town (Paris, 1590; Trier, 1702), framed plain: a 16 x 19 mm pine moulding, its outer edge rounded, stained (walnut-dark, or old shellac gone brown: by where it hangs), under 2 mm glass on a backing board, on a wire between two screw eyes from a hook | the plate at its own size, 276 x 208 mm under the glass; 300 x 232 overall (the old frame's 300 x 220 width kept); the eyes a third down, the wire's apex halfway to the top | the plate sizes: Rijksmuseum records above; hanging hardware a third down, the wire's slack halfway to the top: https://www.metroframe.com/framing-advice/framing-advice-final-assembly/painting-on-barspanels/, https://picturehangsolutions.com/blogs/info/wood-frame-hangers-buyer-guide |
| Antique map (a parlour's) | A hand-coloured atlas map of the 17th-18th century as the 19th century reprinted them, reduced, under a gilt fillet and glass in a 25 mm cushion moulding (walnut, ebonised, oak or mahogany: the old frame colour's draw), on its wire from a hook, leaning out at the top | the sheet 404 x 336 mm, 6 mm of it under the fillet; 460 x 392 overall (the old 460 x 390) | the plates' sizes: as above (545 x 413 to 593 x 515 mm); a gilt fillet (slip) inside a frame: https://www.chairish.com/product/36418871/antique-victorian-eastlake-deep-well-frame-with-gilt-liner-floral-detail |
| Office's framed map | A reprint of an old map on modern paper in an office frame: a flat 17 mm gallery moulding (walnut or oak) or an extruded aluminium section (black or brass-anodised), its print faded by the room's light more than foxed, flush on a sawtooth over a nail | the print 274 x 178 mm; wood 300 x 204 overall, metal 286 x 190 (the old 300 x 220); the aluminium face 10 mm, 20 deep | an aluminium picture frame profile, 9.5 mm face, 26 deep: https://new.pictoonline.fr/en/service/framed-print/custom-size-nielsen-frame; Nielsen Style 15 sections: https://www.dickblick.com/products/nielsen-metal-frame-sections-anodic-black-style-15/ |
| Anatomy chart | A plate of Bidloo's anatomy, reduced, mounted on linen between two turned rails as a school wall chart is (linen-backed, wooden rods, a cord), hung from a nail; varnished, gone amber, its foot curling where the linen pulled, slack between the rails | the sheet 215 x 300 mm; the rails 13 mm, 232 long with their finials (the old 230); the cord a V to the nail 15 cm above the top rail. A real school chart is 0.8-1.5 m; one that size would not fit the old prop's wall | school charts on linen with wooden rods and cord: https://www.chairish.com/product/25608170/large-human-skeleton-school-wall-chart-in-linen-from-german-health-museum-cologne-germany-1950s, https://www.proantic.com/en/1843026-large-anatomical-chart-for-educational-use-human-skeleton-deutsches-hygiene-museum-in-d.html; the plate's size: RP-P-OB-70.703 |
| Botanical print | A hand-coloured botanical plate as such plates were reprinted, reduced, in a cream mat cut with a bevel (its foot the wider), under glass, in a 16 mm half-round moulding (rosewood, mahogany, walnut or bird's-eye maple: the old frame colour's draw), on its wire; the mat's bevel gone brown with acid | the paper 208 x 292 mm, the window 196 x 280 (raised 4 mm); 306 x 386 overall (the old 308 x 388) | the plates are folio, the sheet about 50 x 34 cm: https://heatons-of-tisbury.co.uk/botanicaltrew.html; bottom weighting and a bevel gone brown (mat burn): https://en.wikipedia.org/wiki/Mat_(picture_framing), https://sflac.net/portfolio-view/mat-burn/ |
| Poster | A home-front bill of 1939-42 (three real slogans: FREEDOM IS IN PERIL, cream on red, as the Ministry's 1939 type-only bill with its crown gone; CARELESS TALK COSTS LIVES in black and red; DIG FOR VICTORY in slab capitals on ochre), lithographed in one or two inks on cheap paper, browned and faded, a printer's code in the margin, behind glass in a cheap 12 mm frame (stained, ebonised, varnished pine or grey paint: the old frame colour's draw); none names a country, a party or a person | the sheet 184 x 264 mm, near the small pub format of the Careless Talk series (32 x 20 cm); 200 x 280 overall (the old one's) | the 1939 bills, eleven sizes from 15 x 10 in, lettering like Gill Sans: https://en.wikipedia.org/wiki/Keep_Calm_and_Carry_On; small Careless Talk posters, 32 x 20 cm: https://antikbar.co.uk/products/careless-talk-costs-lives-wwii-pub-fougasse-pw0967 |
| Triptych | A devotional reproduction of a Flemish triptych of about 1510 (Provoost's), as religious shops sell them: the photograph printed on three wooden panels, each in its engaged frame (a flat band, a gilt cove), the wings on iron hinges at the centre's front edges, opened forward; hung flush; its varnish gone amber, finely cracked, grimed | the centre 216 x 280 mm, 28 deep; the wings 109 x 280, 18 deep, opened 17-34 degrees (by where it hangs); about 0.41 m across open (the old 0.47). The original is 45 x 57 cm; one that size would not fit the wall | the original's size and panels: Rijksmuseum SK-A-2570 (https://hdl.handle.net/10934/RM0001.COLLECT.7855) |

## The parts (`t02.py`)

Every frame is a profile swept round its outline, mitred at the corners
(`rect_sweep`: `d` in from the outer edge, `z` out from the wall); under
its lip the stack a framer makes: glass, a mat or a fillet, the paper, the
backing board. The paper is cockled a little, seeded by its part's name.

- `Real__vintageMapFrame` (the rounded pine moulding, a bevel to the
  sight), `Real__vintageMapGlass`, `Real__vintageMapPrint`,
  `Real__vintageMapBack`, `Real__vintageMapWire` (two screw eyes in the
  side rails, the wire drawn up to its apex behind the frame, the ends
  twisted back).
- `Real__antiqueMapFrame` (the cushion: a round rising to its crown, a
  hollow and a bead before the sight), `Real__antiqueMapFillet` (the gilt
  slip: a cove and a bead), `Real__antiqueMapGlass`,
  `Real__antiqueMapPrint`, `Real__antiqueMapBack`, `Real__antiqueMapWire`.
- `Real__officeMapFrameWood` (the flat gallery moulding),
  `Real__officeMapFrameMetal` (the aluminium section and its channel),
  `Real__officeMapGlass`, `Real__officeMapPrint`, `Real__officeMapBack`.
- `Real__anatomyChartSheet` (the plate on linen, slack between the rails,
  waved across, its lower corners curling), `Real__anatomyChartRail` (a
  turned dowel and its finials; top and bottom), `Real__anatomyChartCord`
  (a turn round the top rail at each end, a knot, the V to the nail, over
  it), `Real__anatomyChartNail`.
- `Real__botanicalPrintFrame` (the half-round with a sight bead),
  `Real__botanicalPrintMat` (1.5 mm card, its window cut with a bevel),
  `Real__botanicalPrintGlass`, `Real__botanicalPrintPaper`,
  `Real__botanicalPrintBack`, `Real__botanicalPrintWire`.
- `Real__warPosterFrame`, `Real__warPosterGlass`, `Real__warPosterPaper`
  (thin, buckled where damp got under the glass), `Real__warPosterBack`.
- `Real__triptychCentre` (the panel in its engaged frame and its back),
  `Real__triptychWing` (both wings), `Real__triptychHinge` (three iron
  knuckles on a pin, its domed heads). The centre and the wing are `plane`
  frames: their picture is laid on from the front, so a wing's edge takes
  the picture's edge (its photographed frame) where a `box` frame showed the
  whole painting squeezed across the wing's depth.

The hook every wired picture hangs from is `t01`'s `Real__oldPortraitHook`
(`visuals::pictures::hang`), and the glass is the windows' glass
(`glass_mat`); nothing else was modelled again.

## The finishes and the print (`visuals::pictures`)

Each moulding's finish is painted as its front is mapped, from where on its
profile each pixel lies (`moulding_tex`, the profile's face mirrored in the
placer: `FACE`): stain with its grain along each rail, rubbed paler on the
crests and the outer arris; ebonised black rubbed brown there; gilding gone
brown, burnished on the crests, rubbed to the bole; paint chipped to the
ground. The mat yellows toward its window and its bevel's core browns
(`mat_tex`). The aluminium sections and the rails, cord, nail and wire are
plain materials at their albedo (phong where they shine a little). The
poster's design is set by the game (`sheet_tex`, its painter on its own
stream): each line centred and fitted to its share of the width in a real
face (Gill Sans Bold, Rockwell Bold), the inks a hair out of register and
starved in soft patches, a white margin round the field, the printer's code
in Helvetica; then aged as a print (`age_sheet`, shared with `print_tex`):
browned more than a plate (cheap paper), its inks faded, its edges dark.
Which picture or design, which finish's shade, how far the wings are open
and how the paper aged come from where the thing hangs (`vary_of`), never
from a level draw.

## The draws

Each placer makes the same draws in the same order as the old one; what
the old drew for its look now chooses the new one's: the frame colour
(`choice` or `ri(4)`) the moulding's finish or the rails' wood, every tilt
the swing about its nail or hook (the old turned the whole group about its
origin, which for four of them was the floor under it). The antique map's and the
botanical print's places along the wall and the triptych's
(`mount_on_wall`) are drawn as before. The old canvases' own draws were on
their painters' streams and go with them.

Checked with `--dump-level` (the old code is `main` at `45c4000`): on each
prop's census floor (seed/level 555/10, 7/7, 303/13, 2/11, 202/5, 2/7) and a
second floor for each (77/11, 15/8, 3/8, 27/12, 27/10, 1/9, 19/7; and 42/11,
68/7), the draws, the counts (`solidsHash` among them) and every other
placer's objects (placer, kind, position, bounds, turn, meshes) are the
same before and after, and each prop stands where the old one stood. (On
68/7 the botanical print hangs on a wall a few degrees off the grid, and
the settling slides it 0.3 mm along the wall from where the old one went:
its own outline, not a draw.) Building a floor with two of them costs about
15-35 ms more (`--bench`, the least of four runs: 555/10 746 to 782 ms, 2/7
726 to 743 ms, 202/5 569 to 580 ms).
