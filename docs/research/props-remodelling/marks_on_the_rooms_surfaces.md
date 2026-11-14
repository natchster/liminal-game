# r07: the marks on the rooms' surfaces

Six placers whose props are marks, not things: the stains on the walls and
the ceilings, the water streaks, the cracks in the ceiling, the cobwebs and
the puddles. A mark is painted, not modelled: a transparent texture
(`canvas_tex_opt`, its own stream) on a plane laid on the surface, with the
decal material (`lambert`, `transparent`, `depthWrite: false`,
`polygonOffset -2/-2`), soft and uneven, with a source and a direction;
never a polygon's or an ellipse's hard edge. The only solid things are the
bits a cracked ceiling drops (`r07.py`).

What every plane keeps of the old prop (so `settle_props` probes the wall
where it did and nothing else moves): its middle, its width along the wall
(8 cm where the old one was narrower: `settle_probe` looks at a face's sides
only past 8 cm) and its vertical middle. The decal materials do not write
depth, so the placement pass (`mesh_boxes`) never sees them.

## The real things

**A water stain (wall and ceiling).** Water spreads through the paint,
paper or tile; as it dries from the rim inward, what it carried (dirt,
tannins from the board) is carried to the rim and left there: the tide line,
the dark sharp wavy edge round a paler middle, one more for every time it
got wet again (the coffee-ring effect, Deegan et al. 1997). Brown-yellow on
paint, paper and mineral-fibre tiles; on concrete and brick a dark damp patch
with a white crust of salts (efflorescence) along its rims; mould in soft
speckled clusters where it stayed wet.
- https://kanopibyarmstrong.com/blogs/news/water-stains-on-ceiling (ring
  stains on ceiling tiles; dark crusty ring = an old, dried leak)
- https://www.mit.edu/~azlokapa/questions/coffee.html (why the edge is darker)
- https://en.wikipedia.org/wiki/Efflorescence,
  https://inspectapedia.com/Wet_Basements/Effloresence_White.php (salts on masonry)

**The suspended ceiling** the stains and cracks are on: Armstrong Fine
Fissured 928 / 1728, 24 x 24 x 5/8 in (610 x 610 x 15.9 mm) mineral-fibre
lay-in tiles on a 15/16 in exposed T-bar grid; the game's grid is 6 tiles a
4 m tile (667 mm). A leak wets the tile it drips onto and runs onto the ones
round it; each tile stains on its own, the stain stopped by the grid, whose
steel rusts where the water ran along it. A tile cracks within itself (the
grid holds each one apart): edge to edge, or part way, its halves a little
out of line, the fibre crumbling at the break.
- https://direct-www.armstrongceilings.com/pdbupimages-clg/1728BL225268.pdf/download/data-sheet-fine-fissured.pdf

**A crack in plaster or a slab**: jagged and branching, widest along its
middle and thinning to nothing at its ends (hairline under 1 mm, a working
crack 2-5 mm), on a slab often white with salts leached out along it, in
plaster with the paint lifting at its lips.
- https://eppconcrete.com/types-of-ceiling-cracks-with-pictures
- https://inspectandtest.net/guides/ceiling-cracks/

**What falls from it**: crumbs of the tile's fibre (the tile is 15.9 mm
thick: bits of it are slabs that thick, a painted face and a ragged fibrous
break), or chips of concrete or plaster (one flat face, the rest fractured).

**Water streaks down a wall**: water coming through at the ceiling's edge
runs straight down, narrow where it set off, wider as it slows and spreads,
several runs side by side of different lengths, each ending in the drop it
dried in; the same tide line along each run's edges, a fainter wash between;
white salts on masonry. Where it dripped and stood on the floor below, a
dried ring.

**A cobweb**: the loose tangle of a cobweb spider (Theridiidae), built
between two adjoining surfaces (the angle of a wall and the ceiling), long
abandoned; its sticky silk catches airborne dust and lint, so an old one is a
grey, torn veil with a few heavier, dust-laden strands and anchor lines.
- https://blogs.ifas.ufl.edu/nassauco/2017/07/01/q-dust-gathering-cobwebs-caused-spiders
- https://www.mentalfloss.com/article/600360/where-do-cobwebs-come-from

**A puddle**: on carpet water soaks in: a dark wet patch with a soft ragged
edge and a darker rim where the dirt wicked out to it. On a hard floor it
stands: the floor under a film of water looks darker (less light scattered
back from the filled pores), its edge sharp and irregular, a damp margin
round it, and a sheen.
- https://news.ncsu.edu/?p=934367,
  https://engineerfix.com/does-concrete-lighten-as-it-dries/ (why wet is darker)

## What each placer makes

| Placer | Plane | Painted |
|---|---|---|
| `place_wall_stains` (p32) | from just under the ceiling to as far below the old foot, as wide as the old streak (8 cm at least) | the wetted patch hanging from the ceiling's edge in lobes, its tide lines (2-3, the earlier inside the last), paler middle, a run or two on down ending in a drop; brown-yellow, or damp grey with salts on concrete and brick, faint rust on tile; mould speckled in soft clusters in damp weathers |
| `place_ceiling_stains` (p31) | on a suspended ceiling one plane a tile (the grid's tiles, the one it dripped onto and those it ran onto, never a fitting's); on a slab or plaster one plane | ring stains with 1-5 lobed tide lines, each tile's own, stopped at the grid's flanges; rust on the bars near the leak; salts on a slab |
| `place_wall_water_streaks` (p28) | as the stain's | 2-6 runs from a wetted patch at the ceiling's edge, narrow at the top, wider, narrowing to their drops; the floor ring below on the old disc |
| `place_ceiling_cracks` (p28) | the one tile (tiles), or a plane round the old path (slab, plaster) | in a tile: a jagged break edge to edge or part way along the old heading and kinks, a lit lip and a shadow, crumbs, a branch or two; in a slab or plaster: the old three-run path jagged, tapering to nothing, branches, salts or lifted paint |
| `place_ceiling_cobwebs` (p28) | a sagging sheet across the angle, as long as the old web and as deep (seen from both sides) | a torn grey veil, felted long strands along two directions, anchor lines to the wall and the ceiling (some dusty, heavy, sagging), dust specks and clumps, the old caught insect |
| `place_floor_puddles` (p28) | flat on the floor, 2.5 x the old radius | carpet: soaked-in dark patch, soft ragged edge, darker rim; hard floor: dark film, sharp lobed edge, damp margin, satellite pools, a phong sheen (the drip's tremble animates it) |

The parts (`r07.py`), placed as modelled and scaled 0.87-1.13 by the old
chunk's two size draws:

| Part | What | Size | Material (game) |
|---|---|---|---|
| `Real__ceilingCracksTileBitA..C` | a bit of a mineral-fibre tile: 15.9 mm thick, ragged outline, fibre crumbled at the break | 40-60 mm | aged mineral fibre `0xb0a998` |
| `Real__ceilingCracksChipA..C` | a chip of concrete or plaster: one flat face, fractured facets | 32-50 mm | concrete `0x8e897e` / plaster `0xc2baa8` |

## Draws

Every old draw is kept, in order, and now chooses something of the new mark
(the comments at each say what). Checked with `--dump-level` on every census
floor of these props (seed 99 levels 5, 6, 12, 14; seed 1 levels 6, 8, 10;
seed 11 level 5; seed 2 level 6) and on 90 more (seeds 9, 10, 12-17, 21, 42,
levels 4-12): the same draws, and every other placer's objects the same.

## What the look cannot do

- The engine has no reflections: a hard floor's puddle gets its sheen only
  from the phong highlight of the lights near it, never a mirror of the
  ceiling.
- The wall stains and streaks keep the old prop's narrow plane (8-10 cm):
  wider, `settle_props` would probe the wall's face across it and pin faces
  the old one did not, moving the walls and everything after them.
- The large translucent ellipses many reviewers saw on the walls are not
  this placer: they are the wallpaper's own grime (`visuals::textures`,
  `grime()` in `make_wallpaper` and `make_moss_wallpaper`), drawn as filled
  ellipses.
