# r05: the washroom mirror, the cobwebs, the office plant

Batch `r05.py` (Props P1, docs/design/PROPS.md). What each real thing is, where
its size and construction come from, and what the game does with the parts.

## 1. The wall mirror (`core_wall_mirrors`: `build_broken_mirror`)

**The real one:** the stainless channel-frame washroom mirror that hangs
over the basins of every office and school washroom, here the Bobrick
B-165 2436.

- Size 24 x 36 in (610 x 914 mm), outside of frame to outside of frame.
- Frame: one-piece type 430 stainless steel channel, 1/2 x 1/2 x 3/8 in
  (13 x 13 x 9.5 mm), bright polished, 90 degree mitred corners; a frame
  screw at the bottom locks it to its concealed wall hanger (and lets the
  glass be replaced).
- Glass: 1/4 in (6 mm) float glass mirror, silvered and copper-backed,
  painted on its back.
- Sources: Bobrick technical data,
  <https://www.specifiedby.com/bobrick-uk/b-165-channel-frame-mirror/Bobrick-UK_B-165-Channel-Frame-Mirror_Technical_PDS.pdf>,
  <https://cdn11.bigcommerce.com/s-6en05n3m/content/pdf/B-165_td.pdf>;
  <https://www.ferguson.com/product/bobrick-36-x-24-in.-stainless-steel-channel-frame-glass-mirror-bb1652436/1661308.html>
  (frame 1/2 x 1/2 x 3/8 in, glass 1/4 in).

**How it wears:** the silver goes black from the edges in ("edge creep",
"black edge"): water from the basin and from cleaning gets behind the edges
and corrodes the backing, worst along the bottom and in the corners, with
spots ahead of the front
(<https://civicscreens.com.au/InfoCentre/Mirroredgecreep>,
<https://mirrormate.com/blogs/news/mirror-deterioration-why-it-happens-and-how-to-fix-it>).
Struck, a pane held in its frame breaks in radial cracks from the point of
impact, with concentric cracks between them near it, the crushed spot at
the blow (SWGMAT's glass fracture guideline,
<https://www.nist.gov/document/glassfracturespdf>).

**Modelled:** `Real__mirrorFrame`: the channel swept round the glass with
mitred corners, its outer return rolled into the 13 mm face, the face's
inner edge hemmed over the glass, standing 6 mm off the wall on its hanger;
the two locking screws under the bottom rail.

**In the game:** the frame bright-polished steel gone dull (phong). The
glass is the windows' glass (`Patch::Glass`, docs/design/WINDOWS.md): Fresnel
nearly all reflection (silvered), the room's light in it (the irradiance
where it hangs), the lamp's highlight; its map is where the silver has gone
to the black paint behind (the black edge in stretches with a wandering,
feathered front, deepest along the bottom and in the corners; spots ahead of
it; pinholes; flaking along the cracks and round the blow), and, faintly,
the darker room low in it. Over it a transparent film: dust, dried splashes
and runs on the lower half, wiped arcs, the frame's lip seen again round
the edge, the cracks (a dark line where the silver broke and a bright one
where the edge catches the light) and the crushed spot of the blow, now and
then a shard gone to the back. The breakage is worked out from the old
cracks' draws (four starts and five steps each) by a generator seeded by
them, so the silver and the film agree and no crack can leave the glass
(both are painted inside the glass's own rectangle). The old placer set it
at 1.4 m but its 1.4 m slab was let down to the floor by the placement pass;
the new one hangs where the placer puts it, its centre at 1.4 m (a
washroom mirror's top at about 1.86 m).

## 2. The cobwebs (`core_weather_extras`, still floors: `tail_still_cobwebs`)

**The real thing:** the dusty webs in a room's corners are mostly the
abandoned tangle webs of cobweb spiders (Theridiidae, the common house
spider) and cellar spiders (Pholcidae), built where two surfaces meet: the
sticky silk goes on catching dust after the spider has gone, sags and tears,
and single dragline threads collect dust into hanging strings
(<https://blogs.ifas.ufl.edu/nassauco/2017/07/01/q-dust-gathering-cobwebs-caused-spiders>,
<https://www.mentalfloss.com/animals/invertebrates/where-do-cobwebs-come-from>).
Old ones are grey to sooty, densest in the angle itself; the fine threads
show only where they catch the light. Never a wheel (an orb web is a
garden spider's, outdoors).

**Modelled** (two of each, A and B):

- `Real__cobwebCornerSheet*`: dust packed into the corner's angle (sheets
  across the corner, bulging out of it), a hammock sagging wall to wall
  below it and a smaller one above, curtains from each wall up to the
  ceiling, dusty tails hanging from the hammock's front; torn through in
  places, their free edges ragged.
- `Real__cobwebEdgeSheet*` (along a wall's top, where there is no corner;
  0.56 m and 0.34 m long): the dust packed along the angle, two veils
  sagging from wall to ceiling, tails.
- `Real__cobweb*Strands*`: guy lines between the surfaces (catenaries),
  strings of silk thick with dust hanging from the veils (tapering, kinked,
  a clump or two of fluff on them), the anchor lines along the edges.
- Nothing goes into the walls or the ceiling, and nothing hangs lower than
  the old decal did (0.44 m under the ceiling).

**In the game:** a corner is looked for beside the spot (a wall tile on one
side whose neighbour into the wall is a wall too); the old quarter turn's
draw picks which web and which side is tried first. The corner web's group
stands in the middle of the web so the settling pass finds both walls'
faces beside it. The sheets are painted with a repeating silk (a grey film,
threads in two or three runs, dust drifts, fluff and grit, torn holes),
laid on at its real scale (0.3 m a repeat); how thick the silk is (vertex
alpha) falls off from the corner (or the wall's top) and frays to nothing
over 3.5 cm at every edge of a sheet, so no sheet shows an outline. The
silk is lit as one surface turned out into the room from its corner,
whichever side is seen. The threads are pale, half transparent.

## 3. The office plant (`core_plant_zone`: `build_floor_plant`)

**The real one:** *Dracaena fragrans* 'Janet Craig', the dark green,
plain-leaved corn plant of offices and malls, grown as staggered canes in
a floor planter.

- Leaves strap-shaped, glossy dark green, up to 3 in (7.6 cm) wide and
  10-20 in (25-50 cm) long, in rosettes at the tops of the canes; the
  canes 3-6 ft as sold (NC State Extension,
  <https://plants.ces.ncsu.edu/plants/dracaena-fragrans/common-name/dracaena/>;
  <https://www.houseplant411.com/houseplant/dracaena-janet-craig-plant-how-to-grow-care>,
  at least 3-5 stalks of different heights in a floor plant).
- Neglect: the lower leaves yellow and drop, baring the stalk; brown dry
  tips from dry soil, low humidity, the salts and fluoride of tap water
  (<https://www.houseplant411.com/?p=49114>,
  <https://www.ukhouseplants.com/plants/dracaena-compacta-janet-craig?c=dracaena>);
  the white crust of hard water on the pot and the saucer.

**Modelled:** a 17 in (0.43 m) moulded plastic planter with a rolled rim and
a moulded band, its saucer, the dry soil shrunk from the pot's wall; three
canes 1.30, 1.02 and 0.80 m above the soil (leaf-scar rings, cut flat, red
wax on the cuts); four shoots out of the canes' tops grown leggy toward the
light; four crowns of leaves in a 137.5 degree spiral (15, 12, 9 and 11
living leaves, the youngest upright, the older arching over and drooping;
1-2 yellowing ones hanging; 3-5 dead ones in a skirt under each crown),
brown dry tips; dead leaves on the soil; one dry curled leaf for the floor;
spider mites' webbing in the top crown. No leaf comes lower than 0.9 m
(only the flat dead ones on the soil and the floor): the placement pass
moves the smaller of two props whose parts' boxes meet, and the little
potted plants round it stand 0.82 m at most.

**In the game:** the planter charcoal plastic gone grey with dust, the white
crust where the saucer stood full and runs of it down from the rim, scuffs;
the saucer black; the soil dark peat with perlite grains and bits of dead
leaf; the leaves' maps (dark green filmed with dust, streaks along their
length, brown blotches; tips, yellow and dead leaves the same in their own
colours) laid on at 0.2 m a repeat. The old placer's draws are all kept:
the moss patches' (spent), the eight leaves' (the first one's spin turns
the plant about its pot), the two yellowing leaves', the four dropped
leaves' (their shade, place and turn lay out the four dry leaves on the
floor, out past the saucer), the web's (now and then the mites' webbing).
