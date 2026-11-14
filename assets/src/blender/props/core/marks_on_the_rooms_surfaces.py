"""r07: the marks on the rooms' surfaces (docs/design/PROPS.md), and what falls from
them. The stains, water streaks, cracks, cobwebs and puddles are painted in
the game (soft, irregular textures on transparent planes: `docs/research/props-remodelling/marks_on_the_rooms_surfaces.md`);
the only solid things are the bits a cracked ceiling drops on the floor
under it, placed as modelled (a scale of one, `prop_part_as_modelled`) and
turned at random, so each part's frame is its own middle.

ceilingCracksTileBitA..C  pieces broken off a mineral-fibre ceiling tile: a slab of
                    the tile's 15.9 mm (5/8 in) thickness, its painted face
                    flat (+y), its broken edges ragged with fibre, 3-6 cm
                    across
ceilingCracksChipA..C     chips of concrete or plaster spalled from a slab or a
                    plaster ceiling: a flat face (the surface it was), the
                    rest fractured facets, 2-5 cm across
"""
import math
import random

import bmesh
from propkit import *
import propkit

TILE_T = 0.0159   # a 5/8 in mineral-fibre tile


def hull(points):
    """The convex hull of `points`, its faces outward."""
    bm = bmesh.new()
    for p in points:
        bm.verts.new(p)
    bmesh.ops.convex_hull(bm, input=list(bm.verts))
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context='VERTS')
    s = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return propkit._fix_winding(s)


def tile_bit(seed, size):
    """A piece broken off a ceiling tile, `size` m across: an irregular
    outline, ragged along every broken edge (each edge a run of small
    zigzags), its painted face flat and a shade proud of the fibre core."""
    rnd = random.Random(seed)
    corners = 5 + rnd.randrange(2)
    pts = []
    for k in range(corners):
        a = TAU * k / corners + rnd.uniform(-0.35, 0.35)
        r = size / 2 * rnd.uniform(0.6, 1.0)
        pts.append((r * math.cos(a), r * math.sin(a)))
    outline = []
    for k in range(corners):
        (x0, y0), (x1, y1) = pts[k], pts[(k + 1) % corners]
        n = 4
        for i in range(n):
            t = i / n
            nx, ny = -(y1 - y0), x1 - x0
            ln = math.hypot(nx, ny) or 1.0
            j = rnd.uniform(-0.0018, 0.0018) if i else 0.0
            outline.append((x0 + (x1 - x0) * t + nx / ln * j, y0 + (y1 - y0) * t + ny / ln * j))
    slab_ = prism(outline, TILE_T, bevel_r=0.0008)
    # the fibre core crumbles back from the painted face at the break
    core = noisy(slab_, 0.0012, scale=260.0, seed=seed)
    # thickness along y, the painted face up
    return core.rot(math.pi / 2, 0, 0)


def chip(seed, size):
    """A chip of concrete or plaster `size` m across: one flat face (the
    ceiling's), the rest fractured facets."""
    rnd = random.Random(seed)
    h = size * 0.55
    pts = []
    for k in range(6):
        a = TAU * k / 6 + rnd.uniform(-0.35, 0.35)
        r = size / 2 * rnd.uniform(0.7, 1.0)
        pts.append((r * math.cos(a), h / 2, r * math.sin(a)))
    for k in range(18):
        a, b = rnd.uniform(0, TAU), rnd.uniform(-1.2, 0.5)
        r = size / 2 * rnd.uniform(0.7, 1.0)
        pts.append((r * math.cos(b) * math.cos(a), h / 2 * math.sin(b), r * math.cos(b) * math.sin(a)))
    return hull(pts)


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


for i, (seed, size) in enumerate([(701, 0.052), (702, 0.040), (703, 0.060)]):
    auto('Real__ceilingCracksTileBit' + 'ABC'[i], lambda seed=seed, size=size: tile_bit(seed, size), 240)
for i, (seed, size) in enumerate([(711, 0.042), (712, 0.032), (713, 0.050)]):
    auto('Real__ceilingCracksChip' + 'ABC'[i], lambda seed=seed, size=size: chip(seed, size), 120)
