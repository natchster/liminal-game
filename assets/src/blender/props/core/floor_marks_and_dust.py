"""r11a: the marks of the common floors and the dust that gathers on them
(docs/design/PROPS.md, P2). Sources and sizes: docs/research/props-remodelling/floor_marks_and_dust.md.

The marks (a puddle under a drip, claw marks gouged into a wall, a chalk
arrow, a hand's grime on a wall) are painted in the game, soft and uneven,
on transparent planes; the only solid things are the dust bunnies, placed
as modelled (a scale of one, `prop_part_as_modelled`), each in its own
frame: x and z across the floor, y up from it, its foot at y = 0.

A dust bunny is a loose clump of what a floor sheds (textile lint, hair,
skin, grit) felted together by static and its own fibres where no air
moves: lumps of grey fluff run into each other, ropes of lint and a hair or
two straying out of it and lying on the floor round it.

dustBunnyA      a loose ball of fluff about 60 mm across, 20 mm high, a hair
                trailing out to 105 mm
dustBunnyB      a flat sprawl about 68 mm across, 15 mm high, ropes trailing
dustBunnyC      a small wisp about 38 mm across, 14 mm high
dustBunnyFuzzA..C  the loose fuzz round each one's lumps, a little larger
                than them: the game cuts its fibres out of it (alpha test),
                so its edge is a fringe of fibres, not a surface
"""
import math
import random
import zlib

from mathutils import Vector
from propkit import *
import propkit


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Shaded smooth all over (fluff has no edges)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget, smooth=180.0)


# ========================================================== dust bunny ===
def lump(rnd, c, rx, ry, rz, seed, fluff=1.0):
    """A lump of felted fluff round `c`: an ellipsoid pushed into lobes, then
    roughened into tufts a few millimetres across (`fluff` more of both for
    the loose fuzz round it)."""
    s = sphere(1.0, seg=12, rings=7).scale(rx, ry, rz).move(*c)
    r = min(rx, rz)
    s = noisy(s, r * 0.32 * fluff, scale=1.3 / r, seed=seed)
    return noisy(s, 0.0007 * fluff, scale=380.0, seed=seed + 50)


def strand(rnd, start, length, r0, r1, n=6, curl=0.7, rise=0.35):
    """A rope of lint (or a hair) from `start`, `length` long: a wandering
    line that droops and lies along the floor where it meets it, thinning
    from `r0` to `r1`."""
    p = Vector(start)
    pts = [p.copy()]
    yaw = rnd.uniform(0, TAU)
    pitch = rnd.uniform(-0.2, rise)
    step = length / n
    for _ in range(n):
        yaw += rnd.uniform(-curl, curl)
        pitch += rnd.uniform(-0.45, 0.18)
        d = Vector((math.cos(pitch) * math.cos(yaw), math.sin(pitch), math.cos(pitch) * math.sin(yaw)))
        p = p + d * step
        if p.y < r1 * 1.2:
            p.y = r1 * 1.2
            pitch = max(pitch, 0.0) * 0.3
        pts.append(p.copy())
    return tube(pts, r0, seg=3, caps=False, r_end=r1)


def dust_bunny(size, height, lumps, ropes, hairs, seed):
    """A dust bunny `size` m across and `height` high: `lumps` lumps of fluff
    run together along a crooked line, `ropes` ropes of lint and `hairs`
    hairs straying out of it, its foot flattened on the floor; and the loose
    fuzz round its lumps (drawn with its fibres cut out). (body, fuzz)"""
    rnd = random.Random(seed)
    body, fuzz, centres = [], [], []
    a = rnd.uniform(0, TAU)
    x = z = 0.0
    for k in range(lumps):
        r = size / 2 * rnd.uniform(0.45, 0.62) * (1.0 if k == 0 else rnd.uniform(0.55, 0.9))
        h = height * rnd.uniform(0.38, 0.5) * (1.0 if k == 0 else rnd.uniform(0.7, 0.95))
        if k:
            a += rnd.uniform(-1.1, 1.1)
            x += math.cos(a) * size * rnd.uniform(0.16, 0.26)
            z += math.sin(a) * size * rnd.uniform(0.16, 0.26)
        c = (x, h * 0.7, z)
        rz = r * rnd.uniform(0.75, 1.0)
        body.append(lump(rnd, c, r * 0.82, h * 0.82, rz * 0.82, seed + 7 * k))
        fuzz.append(lump(rnd, c, r * 1.12, h * 1.15, rz * 1.12, seed + 7 * k + 3, fluff=1.8))
        centres.append((Vector(c), r, h))
    for k in range(ropes):
        c, r, h = rnd.choice(centres)
        a = rnd.uniform(0, TAU)
        st = c + Vector((math.cos(a) * r * 0.7, rnd.uniform(-0.3, 0.3) * h, math.sin(a) * r * 0.7))
        body.append(strand(rnd, st, size * rnd.uniform(0.2, 0.45), rnd.uniform(0.0003, 0.00042), 0.00012))
    for k in range(hairs):
        c, r, h = rnd.choice(centres)
        a = rnd.uniform(0, TAU)
        st = c + Vector((math.cos(a) * r * 0.5, 0.0, math.sin(a) * r * 0.5))
        body.append(strand(rnd, st, size * rnd.uniform(0.8, 1.2), 0.0002, 0.00012, n=8, curl=0.5, rise=0.1))
    body, fuzz = sum(body), sum(fuzz)
    # the foot on the floor: nothing below it, the fluff pressed flat there
    for s in (body, fuzz):
        s.warp(lambda p: Vector((p.x, max(p.y, 0.0), p.z)))
    lo, hi = body.bounds()
    off = (-(lo.x + hi.x) / 2, -lo.y, -(lo.z + hi.z) / 2)
    return body.move(*off), fuzz.move(*off)


BUNNIES = {
    'A': dust_bunny(0.060, 0.024, 4, 7, 1, 1101),
    'B': dust_bunny(0.068, 0.017, 5, 7, 1, 1102),
    'C': dust_bunny(0.040, 0.014, 2, 5, 1, 1103),
}
auto('Real__dustBunnyA', lambda: BUNNIES['A'][0], 1300)
auto('Real__dustBunnyB', lambda: BUNNIES['B'][0], 1400)
auto('Real__dustBunnyC', lambda: BUNNIES['C'][0], 750)
auto('Real__dustBunnyFuzzA', lambda: BUNNIES['A'][1], 700)
auto('Real__dustBunnyFuzzB', lambda: BUNNIES['B'][1], 850)
auto('Real__dustBunnyFuzzC', lambda: BUNNIES['C'][1], 400)
