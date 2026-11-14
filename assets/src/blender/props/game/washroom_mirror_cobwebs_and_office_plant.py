"""r05: the props remodelled at their real sizes (docs/design/PROPS.md): a
washroom mirror, the cobwebs of a still floor, a neglected office plant.
Sources and sizes: docs/research/props-remodelling/washroom_mirror_cobwebs_and_office_plant.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part's frame is
given with it. Randomness comes from a `random.Random` of the part's own
(the module is run once per build, whatever else is loaded before it).

The mirror (`build_broken_mirror`): a stainless channel-frame washroom
mirror, 24 x 36 in (Bobrick B-165 2436). x across its face, y up from its
centre, z out of the wall (z = 0 the wall). The glass, its silvering and its
cracks are the game's (`Patch::Glass`, painted).

mirrorFrame   the one-piece channel frame, 13 mm face, mitred corners, its
              hem round the glass, the two locking screws under it

The cobwebs (`tail_still_cobwebs`): dust-laden tangle webs (Theridiidae,
Pholcus) as they are left in a room's corners. Two kinds of place:

  corner  where two walls meet the ceiling: the walls are z = 0 (behind)
          and x = 0 (to the left), the ceiling y = 0; the web hangs in the
          quarter x > 0, z > 0, y < 0
  edge    where a wall meets the ceiling: the wall is z = 0, the ceiling
          y = 0, the web runs along x, centred

Nothing of a web goes into the surfaces it hangs from, nor lower than the
old decal hung (DEEP under the ceiling).

cobwebCornerSheetA/B, cobwebEdgeSheetA/B      the dust packed into the angle
                                              (sheets across the corner, or
                                              along the wall's top), the
                                              sagging, torn veils, the dusty
                                              tails hanging from them (the
                                              game paints them and frays every
                                              edge out)
cobwebCornerStrandsA/B, cobwebEdgeStrandsA/B  the threads: guy lines to the
                                              walls and the ceiling, strings
                                              of silk thick with dust hanging
                                              from the veils, clumps on them

The office plant (`build_floor_plant`): a Dracaena fragrans ('Janet Craig')
on three staggered canes in a moulded plastic planter, left for years in a
corridor: the canes' new shoots grown leggy toward the light, the older
leaves yellowing and hanging, a skirt of dead ones, brown dry tips, the soil
shrunk from the pot. x, z across, y up from the floor, the pot centred. No
leaf comes lower than LEAF_LOW (what stands near the planter keeps its
place: the placement pass moves the smaller of two props whose parts' boxes
meet), except the dead ones lying flat on the soil and the floor.

dracPot        the planter (17 in), its rolled rim and moulded band
dracSaucer     the saucer under it
dracSoil       the dry soil, shrunk from the pot's wall
dracCanes      the three canes (1.30, 1.02, 0.80 m above the soil), their
               leaf-scar rings, cut flat at the top
dracWax        the red wax sealed over each cane's cut
dracShoots     the shoots out of the canes' tops
dracLeaves     the living leaves of the four crowns
dracTips       their dry brown tips
dracYellow     the oldest living leaves, yellowing, hanging
dracYellowTips their dry tips
dracDead       the dead leaves hanging under each crown
dracLitter     the dead leaves lying on the soil
dracFallen     one dead leaf lying on the floor (the game lays out four)
dracWeb        spider mites' webbing between the top crown's leaves
"""
from propkit import *
import propkit
import random
from mathutils import Vector


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


def prune(s):
    """Drop the vertices no face uses (the bounds are the part's frame)."""
    used = sorted({i for f in s.f for i in f})
    remap = {o: n for n, o in enumerate(used)}
    return Shape([s.v[i] for i in used], [tuple(remap[i] for i in f) for f in s.f])


# =============================================================== mirror ===
M_W, M_H = 0.610, 0.914        # overall, 24 x 36 in
M_FACE = 0.013                 # the channel's face, 1/2 in
M_PROJ = 0.019                 # out from the wall, 3/4 in
M_BACK = 0.006                 # the hanger behind the frame
M_SHEET = 0.0009               # the stainless sheet
M_GLASS_FRONT = M_PROJ - 0.0018


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle: d in from
    its outer edge, z out from the wall; the corners mitred."""
    v, f = [], []
    n = len(profile)
    for (d, z) in profile:
        x, y = w / 2 - d, h / 2 - d
        v += [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    for i in range(n):
        j = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, j * 4 + k2, j * 4 + k))
    return propkit._fix_winding(Shape(v, f))


def arc(cx, cz, r, a0, a1, n):
    return [(cx + r * math.cos(a0 + (a1 - a0) * i / n), cz + r * math.sin(a0 + (a1 - a0) * i / n)) for i in range(n + 1)]


def mirror_frame():
    r1, r2 = 0.0016, 0.0008
    lip_back = M_GLASS_FRONT
    prof = [(0.0, M_BACK)]
    # the outer return rolls over into the face
    prof += [(r1 - r1 * math.cos(a), M_PROJ - r1 + r1 * math.sin(a)) for a in (0.0, math.pi / 4, math.pi / 2)]
    # the face, to the hem folded under at its inner edge
    prof += [(M_FACE - r2, M_PROJ)]
    prof += [(M_FACE - r2 + r2 * math.sin(a), M_PROJ - r2 + r2 * math.cos(a)) for a in (math.pi / 2, math.pi)]
    prof += [(M_FACE - r2 - 0.0004, lip_back)]
    # back along the hem's underside, down the inside of the return
    prof += [(M_SHEET, lip_back), (M_SHEET, M_BACK + M_SHEET), (0.0095, M_BACK + M_SHEET), (0.0095, M_BACK)]
    # the rolled corner gives duplicate points at its ends: drop them
    clean = []
    for p in prof:
        if not clean or abs(p[0] - clean[-1][0]) + abs(p[1] - clean[-1][1]) > 1e-6:
            clean.append(p)
    s = rect_sweep(clean, M_W, M_H)
    # the two locking screws under the bottom rail, into the hanger
    for x in (-0.19, 0.19):
        sc = screw_head(r=0.0024, h=0.0011).rot(math.pi, 0, 0).move(x, -M_H / 2, M_BACK + 0.0065)
        s = s + sc
    return s


# ============================================================== cobwebs ===
def membrane(P, ns, nt, rng, ragged=None, holes=0.0, jitter=0.0, cut=2):
    """A sheet over P(s, t), s and t in [0, 1]: ns x nt quads. `ragged` is
    the free edge's side ('s1', 's0', 't0', 't1' or a set of them): the rows
    there are cut back unevenly; `holes` the share of torn-out quads."""
    ragged = set([ragged] if isinstance(ragged, str) else (ragged or ()))
    keep_s = [ns] * (nt + 1)
    lo_s = [0] * (nt + 1)
    for j in range(nt + 1):
        if 's1' in ragged:
            keep_s[j] = ns - rng.choice((0, 0, 1, 1, 2)[:cut + 3])
        if 's0' in ragged:
            lo_s[j] = rng.choice((0, 0, 1, 1, 2)[:cut + 3])
    v, idx = [], {}
    f = []
    for j in range(nt):
        for i in range(ns):
            if i < max(lo_s[j], lo_s[j + 1]) or i >= min(keep_s[j], keep_s[j + 1]):
                continue
            if ('t0' in ragged and j == 0 or 't1' in ragged and j == nt - 1) and rng.random() < 0.45:
                continue
            if 0 < i < ns - 1 and rng.random() < holes:
                continue
            quad = []
            for (a, b) in ((i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)):
                if (a, b) not in idx:
                    p = Vector(P(a / ns, b / nt))
                    if jitter:
                        p = p + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * jitter
                    idx[(a, b)] = len(v)
                    v.append(p)
                quad.append(idx[(a, b)])
            f.append(tuple(quad))
    return Shape(v, f)


def catenary(a, b, sag, n=5, wob=0.0, rng=None):
    a, b = Vector(a), Vector(b)
    pts = []
    for i in range(n + 1):
        t = i / n
        p = a.lerp(b, t) + Vector((0, -sag * 4 * t * (1 - t), 0))
        if wob and 0 < i < n:
            p = p + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * wob
        pts.append(p)
    return pts


def strand(pts, r):
    return tube(pts, r, seg=3, caps=False)


def dust_ball(c, r, rng):
    """A clump of fluff caught on a thread, drawn out along it"""
    b = sphere(r, seg=5, rings=3).scale(rng.uniform(0.7, 1.0), rng.uniform(1.8, 3.2), rng.uniform(0.7, 1.0))
    return b.move(c[0], c[1] + r, c[2])


def tail(top, L, w0, rng, n=6):
    """A tail of silk hanging from a sheet: a narrow ribbon thick with dust,
    tapering, swaying and twisting as it hangs (three across: the game frays
    its two edges out, its middle stays)."""
    top = Vector(top)
    a0 = rng.uniform(0, math.tau)
    tw = rng.uniform(-2.0, 2.0)
    sway = Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * (0.12 * L)
    v, f = [], []
    for k in range(n + 1):
        t = k / n
        c = top + Vector((0, -L * t, 0)) + sway * t * t
        a = a0 + tw * t
        half = w0 * (1 - 0.7 * t) / 2
        d = Vector((math.cos(a), 0, math.sin(a))) * half
        v += [c - d, c + Vector((-d.z, 0, d.x)) * 0.25, c + d]
    for k in range(n):
        i = 3 * k
        f.append((i, i + 1, i + 4, i + 3))
        f.append((i + 1, i + 2, i + 5, i + 4))
    return Shape(v, f)


DEEP = 0.44        # how far under the ceiling a web hangs at most (the old one's depth)


def rope(top, L, r0, rng, n=6):
    """A string of silk hanging from a web, thick with the dust it caught,
    swaying a little, thinning to its end, a clump or two of fluff on it."""
    top = Vector(top)
    sway = Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * (0.14 * L)
    pts = []
    for k in range(n):
        t = k / (n - 1)
        kink = Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * (0.05 * L * math.sin(math.pi * t))
        pts.append(top + Vector((0, -L * t, 0)) + sway * t * t + kink)
    s = tube(pts, r0, seg=4, caps=False, r_end=r0 * 0.4)
    for _ in range(rng.choice((0, 1, 1, 2))):
        t = rng.uniform(0.35, 0.95)
        c = top + Vector((0, -L * t, 0)) + sway * t * t
        s = s + dust_ball(c, r0 * rng.uniform(1.3, 2.2), rng)
    return s


def off_walls(s, corner):
    """Nothing of a web goes into the walls or the ceiling it hangs from
    (the corner's two walls x = 0 and z = 0, or the one wall z = 0)."""
    for p in s.v:
        p.y = min(p.y, -0.0006)
        p.z = max(p.z, 0.0012)
        if corner:
            p.x = max(p.x, 0.0012)
    return s


def corner_plug(d, bulge, rng, ns=6, nt=6, holes=0.0):
    """A sheet across the very corner, its three edges on the two walls and
    the ceiling (d out along each), sagging out of the corner in its middle:
    the dust packed into a corner's angle."""
    # its edges stand a little off the walls (the game frays them out: no line drawn on a wall)
    A = Vector((d * rng.uniform(0.8, 1.2), -0.008, 0.008))       # where wall z=0 meets the ceiling
    B = Vector((0.008, -0.008, d * rng.uniform(0.8, 1.2)))       # where wall x=0 meets the ceiling
    C = Vector((0.008, -d * rng.uniform(0.7, 1.05), 0.008))      # down the walls' corner
    out = Vector((1, -1, 1)).normalized()

    def P(sv, tv):
        foot = C + Vector((1 - tv, 0, tv)) * (d * 0.08)
        p = foot.lerp(A.lerp(B, tv), sv)
        return p + out * (bulge * math.sin(math.pi * tv) * math.sin(math.pi * min(1.0, sv * 1.15)))
    return membrane(P, ns, nt, rng, holes=holes)


def corner_web(seed, strands):
    """The web where two walls meet the ceiling (see the module's notes)."""
    rng = random.Random(seed)
    s = Shape()
    if not strands:
        # the dust packed into the corner's angle, layer over layer
        for d, bulge in ((0.07, 0.012), (0.13, 0.025), (0.2, 0.04)):
            s = s + corner_plug(d * rng.uniform(0.85, 1.15), bulge, rng, holes=0.08)
        # the main hammock, wall to wall below the corner, its front sagging
        a, b = rng.uniform(0.36, 0.44), rng.uniform(0.33, 0.41)
        ya, yb = -rng.uniform(0.17, 0.24), -rng.uniform(0.14, 0.2)
        sag = rng.uniform(0.07, 0.1)

        def ham(sv, tv, a=a, b=b, ya=ya, yb=yb, sag=sag, y0=-0.012, s0=0.05):
            u = s0 + (1 - s0) * sv
            pa = Vector((a * u, y0 + (ya - y0) * u, 0.002))
            pb = Vector((0.002, y0 + (yb - y0) * u, b * u))
            p = pa.lerp(pb, tv)
            k = math.sin(math.pi * tv)
            return p + Vector((0.0, -sag * k * u ** 1.3, 0.0)) + Vector((1, 0, 1)) * (0.015 * k * u)
        s = s + membrane(ham, 14, 12, rng, ragged='s1', holes=0.07)
        # a smaller, denser one higher in the corner
        a2, b2 = rng.uniform(0.17, 0.23), rng.uniform(0.16, 0.22)
        s = s + membrane(lambda sv, tv: ham(sv, tv, a=a2, b=b2, ya=-0.08, yb=-0.07, sag=0.03, y0=-0.004, s0=0.1),
                         8, 8, rng, ragged='s1', holes=0.04)
        # curtains from each wall up to the ceiling
        for side in (0, 1):
            L = rng.uniform(0.24, 0.34)
            d = rng.uniform(0.09, 0.14)
            h = rng.uniform(0.07, 0.11)

            def cur(sv, tv, L=L, d=d, h=h, side=side):
                x = 0.03 + L * sv
                taper = math.sin(math.pi * min(1.0, 0.15 + 0.85 * sv)) ** 0.5
                wall = Vector((x, -h * taper - 0.004, 0.002))
                ceil = Vector((x, -0.002, d * taper + 0.004))
                p = wall.lerp(ceil, tv) + Vector((0, -0.028 * math.sin(math.pi * tv) * taper, 0))
                return p if side == 0 else Vector((p.z, p.y, p.x))
            s = s + membrane(cur, 9, 6, rng, ragged='s1', holes=0.05)
        # tails hanging from the hammock's front, heavy with dust
        for _ in range(5):
            top = ham(rng.uniform(0.7, 0.95), rng.uniform(0.15, 0.85))
            s = s + tail(top, min(rng.uniform(0.06, 0.3), DEEP + top.y), rng.uniform(0.012, 0.026), rng)
        return off_walls(prune(s), True)
    # threads
    parts = []
    walls = lambda: rng.choice(('a', 'b', 'c'))

    def on(surface, reach=0.44):
        if surface == 'a':
            return (rng.uniform(0.01, reach), -rng.uniform(0.0, reach * 0.6), 0.001)
        if surface == 'b':
            return (0.001, -rng.uniform(0.0, reach * 0.6), rng.uniform(0.01, reach))
        return (rng.uniform(0.0, reach * 0.7), -0.001, rng.uniform(0.0, reach * 0.7))
    # guy lines between the surfaces
    for _ in range(14):
        s1 = walls()
        s2 = rng.choice([w for w in ('a', 'b', 'c') if w != s1])
        p, q = on(s1), on(s2)
        d = (Vector(p) - Vector(q)).length
        parts.append(strand(catenary(p, q, d * rng.uniform(0.06, 0.2), n=5, wob=0.004, rng=rng), rng.uniform(0.0005, 0.0009)))
    # strings hanging from under the sheets, thick with dust (no lower than the
    # old web hung: DEEP under the ceiling)
    for _ in range(7):
        x, z = rng.uniform(0.04, 0.32), rng.uniform(0.04, 0.3)
        top = (x, -0.04 - 0.45 * (x + z) / 2, z)
        L = min(rng.uniform(0.05, 0.3), DEEP + top[1] - 0.01)
        parts.append(rope(top, L, rng.uniform(0.0008, 0.0015), rng))
    # the anchor lines run out along the two wall-ceiling edges and the corner
    for (dx, dz) in ((1, 0), (0, 1)):
        L = rng.uniform(0.45, 0.6)
        pts = [Vector((dx * L * k / 4 + 0.004 * dz, -0.004 - 0.003 * (k % 2), dz * L * k / 4 + 0.004 * dx)) for k in range(5)]
        parts.append(strand(pts, 0.0008))
    pts = [Vector((0.004, -0.45 * k / 4, 0.004)) for k in range(5)]
    parts.append(strand(pts, 0.0007))
    return off_walls(prune(sum(parts, Shape())), True)


def edge_web(seed, strands, L):
    """The web where a wall meets the ceiling, along x, about L long (see
    the notes)."""
    rng = random.Random(seed)
    if not strands:
        s = Shape()
        hams = []
        for k, (h, d, sag, ns) in enumerate(((rng.uniform(0.2, 0.27), rng.uniform(0.17, 0.23), 0.09, 13),
                                             (rng.uniform(0.1, 0.14), rng.uniform(0.09, 0.12), 0.035, 9))):
            Lk = L * (1.0 if k == 0 else rng.uniform(0.45, 0.6))
            x0 = -L / 2 + (0 if k == 0 else rng.uniform(0.05, L - Lk - 0.05))

            def ham(sv, tv, Lk=Lk, x0=x0, h=h, d=d, sag=sag):
                x = x0 + Lk * sv
                taper = math.sin(math.pi * (0.06 + 0.88 * sv)) ** 0.6
                wall = Vector((x, -h * taper - 0.003, 0.002))
                ceil = Vector((x, -0.002, d * taper + 0.003))
                return wall.lerp(ceil, tv) + Vector((0, -sag * taper * math.sin(math.pi * tv), 0.0))
            hams.append(ham)
            s = s + membrane(ham, ns, 8, rng, ragged={'s0', 's1'}, holes=0.05, cut=1)
        # the dust packed along the angle where the wall meets the ceiling
        f0, f1 = rng.uniform(0.0, 0.25), rng.uniform(0.0, 0.25)

        def fillet(sv, tv, f0=f0, f1=f1):
            x = -L / 2 + L * (0.04 + 0.92 * sv)
            k = math.sin(math.pi * (0.04 + 0.92 * sv)) ** 0.5
            hh = (0.035 + 0.045 * (f0 + (f1 - f0) * sv) / 0.25) * k + 0.006
            wall = Vector((x, -hh, 0.002))
            ceil = Vector((x, -0.002, hh * 0.9))
            return wall.lerp(ceil, tv) + Vector((0, -1, 1)).normalized() * (0.012 * k * math.sin(math.pi * tv))
        s = s + membrane(fillet, max(8, round(L / 0.035)), 5, rng, holes=0.06)
        # tails from the big one's lowest line
        for _ in range(4):
            top = hams[0](rng.uniform(0.2, 0.8), rng.uniform(0.35, 0.65))
            s = s + tail(top, min(rng.uniform(0.07, 0.26) * L / 0.56, DEEP + top.y), rng.uniform(0.012, 0.026), rng)
        return off_walls(prune(s), False)
    parts = []
    for _ in range(14):
        x1, x2 = rng.uniform(-L / 2, L / 2), rng.uniform(-L / 2, L / 2)
        p = (x1, -rng.uniform(0.0, 0.24), 0.001)
        q = (x2, -0.001, rng.uniform(0.0, 0.25))
        d = (Vector(p) - Vector(q)).length
        parts.append(strand(catenary(p, q, d * rng.uniform(0.06, 0.2), n=5, wob=0.004, rng=rng), rng.uniform(0.0005, 0.0009)))
    # strings hanging from under the sheets, thick with dust
    for _ in range(6):
        x = rng.uniform(-L * 0.4, L * 0.4)
        top = (x, -0.06 - rng.uniform(0.0, 0.08), rng.uniform(0.04, 0.12))
        Lh = min(rng.uniform(0.06, 0.3) * L / 0.56, DEEP + top[1] - 0.01)
        parts.append(rope(top, Lh, rng.uniform(0.0008, 0.0015), rng))
    pts = [Vector((-L / 2 - 0.06 + (L + 0.12) * k / 5, -0.004 - 0.002 * (k % 2), 0.004)) for k in range(6)]
    parts.append(strand(pts, 0.0008))
    return off_walls(prune(sum(parts, Shape())), False)


# ================================================================ plant ===
POT_Y0 = 0.005                   # the pot stands in its saucer
SOIL_Y = 0.322
CANES = [  # (x, z, radius, height above the soil, lean x, lean z)
    (-0.045, 0.030, 0.029, 1.30, -0.05, 0.02),
    (0.055, 0.035, 0.025, 1.02, 0.06, 0.03),
    (0.005, -0.060, 0.022, 0.80, 0.01, -0.07),
]
# Nothing of the leaves comes lower than this: whatever stands near the
# planter (its little potted neighbours stand 0.82 m at most) keeps its place
# (the game's placement pass moves the smaller of two props whose parts' boxes
# meet). Only the dead leaves on the soil and the floor (flat) go lower.
LEAF_LOW = 0.9
# the shoots: (cane, start height below its top, direction yaw, rise, length)
SHOOTS = [(0, 0.035, 2.6, 1.25, 0.30), (1, 0.03, 0.4, 1.0, 0.18), (1, 0.05, 4.1, 0.85, 0.12), (2, 0.03, 5.2, 0.9, 0.15)]


def cane_axis(c):
    """A cane's foot (buried 6 cm) and the middle of its cut top."""
    x, z, r, h, lx, lz = c
    H = h + 0.06
    base = Vector((x, SOIL_Y - 0.06, z))
    top = Vector((x + lx * H, SOIL_Y + h, z + lz * H))
    return base, top


def pot():
    prof = [(0.150, POT_Y0), (0.154, POT_Y0 + 0.012), (0.197, 0.300), (0.203, 0.307),
            (0.207, 0.350), (0.215, 0.357), (0.216, 0.368)]
    return vessel(prof, 0.006, seg=28, floor=0.008)


def saucer():
    return vessel([(0.160, 0.0), (0.166, 0.004), (0.190, 0.030), (0.194, 0.035)], 0.003, seg=28, floor=0.004)


def soil():
    rng = random.Random(11)
    r = 0.192
    s = cyl(r, r - 0.002, 0.03, seg=22, bevel_r=0.004)
    s = s.move(0, SOIL_Y - 0.015, 0)
    out = s.copy()
    for i, p in enumerate(out.v):
        if p.y > SOIL_Y - 0.004:
            # dry crumbs and the old watering hollow by the canes
            q = p * 30.0
            out.v[i] = p + Vector((0, 0.006 * noise.noise(q) - 0.008 * math.exp(-((p.x + 0.01) ** 2 + p.z ** 2) / 0.006), 0))
    return out


def cane(c, rng):
    x, z, r, h, lx, lz = c
    base, top = cane_axis(c)
    H = h + 0.06
    prof = [(0.0, 0.0)]
    n = int(H / 0.028)
    for i in range(n + 1):
        y = H * i / n
        rr = r * (1.0 + (0.045 if i % 2 else 0.0) + 0.02 * rng.uniform(-1, 1))
        prof.append((rr, y))
    prof.append((r * 0.96, H + 0.002))
    prof.append((0.0, H + 0.002))
    s = lathe(prof, seg=8, cap_bottom=False, cap_top=False)
    # its lean: sheared along its height
    s = s.warp(lambda p: Vector((p.x + lx * p.y, p.y, p.z + lz * p.y)))
    return s.move(base.x, base.y, base.z)


def canes():
    rng = random.Random(3)
    return sum((cane(c, rng) for c in CANES), Shape())


def wax():
    out = Shape()
    for c in CANES:
        base, top = cane_axis(c)
        r = c[2]
        cap = lathe([(0.0, 0.0), (r * 1.03, 0.0), (r * 1.02, 0.004), (r * 0.7, 0.007), (0.0, 0.008)], seg=8)
        out = out + cap.move(top.x, top.y + 0.001, top.z)
    return out


def shoot_path(sh):
    ci, below, yaw, rise, L = sh
    base, top = cane_axis(CANES[ci])
    r = CANES[ci][2]
    d = Vector((math.sin(yaw), 0, math.cos(yaw)))
    start = Vector((top.x, top.y - below, top.z)) + d * (r * 0.8)
    pts = [start]
    p = start.copy()
    for k in range(1, 6):
        t = k / 5
        a = 0.35 + (rise - 0.35) * t           # out first, then up toward the light
        p = p + (d * math.cos(a) + Vector((0, 1, 0)) * math.sin(a)) * (L / 5)
        pts.append(p.copy())
    return pts


def shoots():
    out = Shape()
    for sh in SHOOTS:
        pts = shoot_path(sh)
        out = out + tube(pts, 0.012, seg=7, caps=True, r_end=0.010)
    return out


def leaf_strip(base, yaw, elev, L, W, droop, twist, fold, wave, rng, n=8, tip=0.0, narrow=1.0, phases=None):
    """A strap leaf from `base`: its centreline rises at `elev` toward `yaw`
    and bends down by `droop` over its length; three points across (the
    edges and the midrib, folded by `fold`). Returns (body, tip) shapes, the
    tip being the last `tip` of its length."""
    hd = Vector((math.sin(yaw), 0, math.cos(yaw)))
    up = Vector((0, 1, 0))
    pts, tans = [], []
    p = Vector(base)
    for i in range(n):
        s = i / (n - 1)
        a = max(elev - droop * s ** 1.35, -1.50)
        t = hd * math.cos(a) + up * math.sin(a)
        pts.append(p.copy())
        tans.append(t)
        p = p + t * (L / (n - 1))
    rows = []
    for i, (c, t) in enumerate(zip(pts, tans)):
        s = i / (n - 1)
        if s <= 0.42:
            w = W * (0.42 + 0.58 * math.sin(math.pi / 2 * s / 0.42))
        else:
            w = W * max(0.03, 1 - ((s - 0.42) / 0.58) ** 1.7)
        w *= narrow
        side = t.cross(up)
        if side.length < 1e-6:
            side = hd.cross(up)
        side.normalize()
        nrm = side.cross(t).normalized()
        q = twist * s
        side, nrm = side * math.cos(q) + nrm * math.sin(q), nrm * math.cos(q) - side * math.sin(q)
        wv = wave * math.sin(s * 19.0 + (phases[i] if phases else rng.uniform(0, 6))) * w
        l = c - side * (w / 2) + nrm * (fold * w + wv)
        r = c + side * (w / 2) + nrm * (fold * w - wv)
        rows.append((l, c, r))
    k = n - 1 - (round(tip * (n - 1)) if tip > 0 else 0)
    k = max(1, min(n - 1, k))

    def strip(i0, i1):
        v, f = [], []
        for i in range(i0, i1 + 1):
            v += list(rows[i])
        for i in range(i1 - i0):
            a = i * 3
            f.append((a, a + 1, a + 4, a + 3))
            f.append((a + 1, a + 2, a + 5, a + 4))
        return Shape(v, f)
    body = strip(0, k)
    tipshape = strip(k, n - 1) if k < n - 1 else Shape()
    return body, tipshape


def high_leaf(make):
    """A leaf from `make(k)` (k from 1 down: how far it reaches as grown),
    shortened and lifted until none of it is lower than LEAF_LOW."""
    for step in range(16):
        k = 1.0 - step * 0.05
        body, tp = make(k)
        lo = min(p.y for p in (body + tp).v)
        if lo >= LEAF_LOW:
            break
    return body, tp


def crowns():
    """Every crown's leaves, by what they are: (green, their tips, yellow,
    their tips, dead hanging, dead on the soil)."""
    rng = random.Random(7)
    green, tips, yellow, ytips, dead, litter = Shape(), Shape(), Shape(), Shape(), Shape(), Shape()
    golden = math.radians(137.5)
    for ci, sh in enumerate(SHOOTS):
        pts = shoot_path(sh)
        tip = pts[-1]
        axis = (pts[-1] - pts[-2]).normalized()
        n_green = (15, 12, 9, 11)[ci]
        n_yellow = (2, 2, 1, 2)[ci]
        n_dead = (5, 4, 3, 4)[ci]
        scale = (1.0, 0.9, 0.75, 0.82)[ci]
        yaw0 = rng.uniform(0, math.tau)
        total = n_dead + n_yellow + n_green
        for k in range(total):
            age = 1 - k / (total - 1)                      # 1 the oldest, 0 the newest
            yaw = yaw0 + golden * k
            # the leaves clasp the top 9 cm of the shoot, the oldest lowest
            along = 0.09 * age
            b = tip - axis * along + Vector((math.sin(yaw), 0, math.cos(yaw))) * 0.011
            ph = [rng.uniform(0, 6) for _ in range(8)]
            if k < n_dead:
                L = scale * rng.uniform(0.30, 0.46)
                el, dr, tw = rng.uniform(-1.1, -0.6), rng.uniform(0.4, 0.9), rng.uniform(1.5, 3.5)
                body, tp = high_leaf(lambda f: leaf_strip(b, yaw, el * f, L * f, 0.05, dr * f, tw, 0.45, 0.12, rng, n=6,
                                                          narrow=0.55, phases=ph))
                dead = dead + body + tp
            elif k < n_dead + n_yellow:
                L = scale * rng.uniform(0.50, 0.62)
                el, dr, tw, tf = rng.uniform(-0.15, 0.15), rng.uniform(2.2, 2.7), rng.uniform(-0.5, 0.5), rng.uniform(0.15, 0.3)
                body, tp = high_leaf(lambda f: leaf_strip(b, yaw, el + 0.5 * (1 - f), L * f, 0.062, dr * f, tw, 0.22, 0.05, rng,
                                                          n=8, tip=tf, phases=ph))
                yellow = yellow + body
                ytips = ytips + tp
            else:
                young = (k - n_dead - n_yellow) / max(1, n_green - 1)     # 0 the oldest green
                L = scale * (0.62 - 0.30 * young ** 1.5) * rng.uniform(0.9, 1.08)
                W = (0.07 - 0.022 * young) * rng.uniform(0.9, 1.1)
                elev = 0.15 + 1.15 * young + rng.uniform(-0.1, 0.1)
                droop = (2.0 - 1.3 * young) * rng.uniform(0.85, 1.15)
                tipf = rng.choice((0.0, 0.1, 0.14, 0.14, 0.2, 0.28)) if young < 0.85 else rng.choice((0.0, 0.0, 0.1))
                tw = rng.uniform(-0.6, 0.6)
                body, tp = high_leaf(lambda f: leaf_strip(b, yaw, elev + 0.3 * (1 - f), L * (0.6 + 0.4 * f), W, droop * f, tw,
                                                          0.18 + 0.1 * young, 0.03 * (1 - young), rng, n=8, tip=tipf, phases=ph))
                green = green + body
                tips = tips + tp
    # the dead lying on the soil
    for k in range(4):
        a = rng.uniform(0, math.tau)
        rr = rng.uniform(0.07, 0.15)
        b = Vector((math.cos(a) * rr, SOIL_Y + 0.004, math.sin(a) * rr))
        body, tp = leaf_strip(b, a + rng.uniform(-1, 1), 0.0, rng.uniform(0.2, 0.32), 0.045, 0.0,
                              rng.uniform(2, 4), 0.5, 0.1, rng, n=6, narrow=0.6)
        litter = litter + body + tp
    return green, tips, yellow, ytips, dead, litter


_CROWNS = []


def crown_part(i):
    if not _CROWNS:
        _CROWNS.extend(crowns())
    return _CROWNS[i]


def fallen():
    rng = random.Random(5)
    body, tp = leaf_strip(Vector((-0.2, 0.012, 0.0)), math.pi / 2, 0.06, 0.42, 0.05, 0.14, 1.2, 0.3, 0.12, rng,
                          n=7, narrow=0.85)
    s = body + tp
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2, -lo.y, -(lo.z + hi.z) / 2)


def mite_web():
    rng = random.Random(9)
    sh = SHOOTS[0]
    tip = shoot_path(sh)[-1]
    parts = []
    for _ in range(18):
        a, b = rng.uniform(0, math.tau), rng.uniform(0, math.tau)
        p = tip + Vector((math.sin(a) * rng.uniform(0.04, 0.16), rng.uniform(-0.02, 0.12), math.cos(a) * rng.uniform(0.04, 0.16)))
        q = tip + Vector((math.sin(b) * rng.uniform(0.04, 0.16), rng.uniform(-0.02, 0.12), math.cos(b) * rng.uniform(0.04, 0.16)))
        parts.append(strand(catenary(p, q, 0.01, n=3), 0.0005))
    return sum(parts, Shape())


# ================================================================ parts ===
auto('Real__mirrorFrame', mirror_frame, 400)
auto('Real__cobwebCornerSheetA', lambda: corner_web(101, False), 1100)
auto('Real__cobwebCornerSheetB', lambda: corner_web(202, False), 1100)
auto('Real__cobwebCornerStrandsA', lambda: corner_web(103, True), 1200)
auto('Real__cobwebCornerStrandsB', lambda: corner_web(204, True), 1200)
auto('Real__cobwebEdgeSheetA', lambda: edge_web(301, False, 0.56), 800)
auto('Real__cobwebEdgeSheetB', lambda: edge_web(402, False, 0.34), 800)
auto('Real__cobwebEdgeStrandsA', lambda: edge_web(303, True, 0.56), 900)
auto('Real__cobwebEdgeStrandsB', lambda: edge_web(404, True, 0.34), 900)
auto('Real__dracPot', pot, 1000)
auto('Real__dracSaucer', saucer, 600)
auto('Real__dracSoil', soil, 560)
auto('Real__dracCanes', canes, 2000)
auto('Real__dracWax', wax, 200)
auto('Real__dracShoots', shoots, 560)
auto('Real__dracLeaves', lambda: crown_part(0), 1800)
auto('Real__dracTips', lambda: crown_part(1), 1100)
auto('Real__dracYellow', lambda: crown_part(2), 1100)
auto('Real__dracYellowTips', lambda: crown_part(3), 400)
auto('Real__dracDead', lambda: crown_part(4), 1100)
auto('Real__dracLitter', lambda: crown_part(5), 400)
auto('Real__dracFallen', fallen, 280)
auto('Real__dracWeb', mite_web, 560)
