"""t06: the garden things, remodelled at their real sizes (docs/design/PROPS.md,
P3): a pothos in a plastic hanging pot on a wall bracket, a canvas garden
apron dropped on the floor, a garden gnome, a seed catalogue framed, a bag
of potting soil, houseplant fertilizer spikes, a bundle of herbs hung to
dry, and a cast concrete bird bath. Sources and sizes: docs/research/props-remodelling/garden_things.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
own frame: x across its front, y up, z out of its front. A floor thing's
floor is the plane y = 0 (its lowest point on it); a wall thing's wall is
the plane z = 0, its y the height above the floor where it is said.

fertSpike         a houseplant fertilizer spike (52 mm, compressed
                  granules, octagonal, tapered to a blunt point), lying
                  along +x on one of its flats, its head at -x
fertSpikeHead     a spike snapped in two: the head end, its break rough
fertSpikeTip      its point end, lying a little apart
fertCrumbs        crumbs of a spike that crumbled, round the origin
soilBag           an 8 dry quart bag of potting soil (a tube of printed
                  polyethylene 305 mm flat, 430 mm long, heat sealed at
                  both ends) lying on its back: slumped, its film wrinkled
                  toward the seals; x across, z along it
soilBagSeals      its two seals: fins of double film, crimped across
soilBagSpill      soil that ran out of a split at one corner (+x, +z): a
                  low heap, its crumbs
birdBathPedestal  a cast concrete bird bath's pedestal (430 mm): a stepped
                  round plinth, a tapering column, its collars, the socket
                  at its top (a cylinder's texture coordinates)
birdBathBowlOut   its basin (410 mm across), outside: the underside, the
                  spigot that sits in the socket, the rim's outer edge
birdBathBowlIn    its basin, inside: the dish and the rim's top (the game
                  lays its stains on from above)
"""
import math
import random
import zlib

import bmesh
from propkit import *
import propkit

CYL_FRAMED = {'Real__birdBathPedestal', 'Real__birdBathBowlOut', 'Real__hangPlantPot', 'Real__hangPlantSaucer'}


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name (the
    shape is built here, at registration)."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    if not shape.v:
        return                      # nothing of it (a vine with no dead leaves): the game skips it
    lo, hi = shape.bounds()
    kind = 'cyl' if name in CYL_FRAMED else 'box'
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, smooth=smooth, flat=flat, budget=budget)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def on_floor(s, centre=True):
    """Lowest point to y = 0, its middle (x, z) over the origin."""
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2 if centre else 0, -lo.y, -(lo.z + hi.z) / 2 if centre else 0)


def sweep(pts, radii, seg=6, cap0=True, cap1=True, rough=None, up=None):
    """A tube along `pts` whose radius at each point is `radii[i]`
    (parallel-transport frame). `rough(i, j, p, d) -> dr` pushes a vertex
    out along its ring's direction d. cap0/cap1: 'dome' rounds the end,
    True closes it flat, False leaves it open."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    tang = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tang.append(t.normalized() if t.length > 1e-12 else Vector((1, 0, 0)))
    upv = Vector(up) if up else (Vector((0, 1, 0)) if abs(tang[0].y) < 0.9 else Vector((1, 0, 0)))
    nrm = tang[0].cross(upv).normalized()
    v, f = [], []
    for i in range(n):
        if i:
            q = tang[i - 1].rotation_difference(tang[i])
            nrm = (q @ nrm).normalized()
        b = tang[i].cross(nrm).normalized()
        for j in range(seg):
            a = TAU * j / seg
            d = nrm * math.cos(a) + b * math.sin(a)
            r = radii[i] + (rough(i, j, pts[i], d) if rough else 0.0)
            v.append(pts[i] + d * r)
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    s = Shape(v, f)
    for end, cap in ((0, cap0), (n - 1, cap1)):
        if not cap:
            continue
        ring = [end * seg + j for j in range(seg)]
        if cap == 'dome':
            out = -tang[0] if end == 0 else tang[-1]
            s.v.append(pts[end] + out * radii[end] * 0.8)
        else:
            s.v.append(pts[end].copy())
        c = len(s.v) - 1
        for a, b in zip(ring, ring[1:] + ring[:1]):
            s.f.append((b, a, c) if end == 0 else (a, b, c))
    return propkit._fix_winding(s)


def chaikin(pts, k=2):
    pts = [Vector(p) for p in pts]
    for _ in range(k):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    return pts


def hull(points):
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


def lump(rng, c, r, k=9, squash=1.0):
    """A small irregular stone or crumb: the hull of points round c."""
    pts = []
    for _ in range(k):
        d = Vector((rng.gauss(0, 1), rng.gauss(0, 1) * squash, rng.gauss(0, 1))).normalized()
        pts.append(Vector(c) + d * r * (0.6 + 0.5 * rng.random()))
    return hull(pts)


def grid_surface(fn, nu, nv, wrap_u=False):
    """A surface from fn(u, v) -> Vector over u in [0, 1] (nu steps, wrapped
    if wrap_u) and v in [0, 1] (nv steps): an open grid of quads."""
    cols = nu if wrap_u else nu + 1
    v, f = [], []
    for j in range(nv + 1):
        for i in range(cols):
            v.append(fn(i / nu, j / nv))
    for j in range(nv):
        for i in range(nu):
            i2 = (i + 1) % cols if wrap_u else i + 1
            f.append((j * cols + i, j * cols + i2, (j + 1) * cols + i2, (j + 1) * cols + i))
    return Shape(v, f)


def floor_clamp(s, y0=0.0):
    """Nothing under the floor (the placement pass would lift the prop)."""
    return s.warp(lambda p: Vector((p.x, max(p.y, y0), p.z)))


def n1(x, y=0.0, z=0.0):
    """Scalar Perlin noise (repeats in every Blender process)."""
    return noise.noise(Vector((x, y, z)))


# =================================================== fertilizer spikes ===
# A houseplant fertilizer spike (Jobe's houseplant spikes 13-4-5: 50 to a
# 2.2 oz card, 1.2 g each): compressed granules and binder pressed in a die
# to a tapering octagonal stick, 52 mm long, 8 mm across its head, its
# point blunt; chalky, the granules showing, its edges crumbled.
SPIKE_L, SPIKE_R = 0.052, 0.0043


def _spike_r(t):
    """Its circumradius at t (0 the head, 1 the point)."""
    if t < 0.05:
        return SPIKE_R - 0.0007 * (1 - t / 0.05) ** 2          # the chamfered head
    if t < 0.58:
        return SPIKE_R - 0.0005 * (t - 0.05) / 0.53
    k = (t - 0.58) / 0.42
    return max(0.0012, (SPIKE_R - 0.0005) * (1 - 0.72 * k ** 1.15))


def _spike(t0, t1, rough0, rough1, seed):
    """The spike from t0 to t1 of its length, along +x (its head at x = 0),
    an octagon lying on a flat; an end that is a break is rough."""
    rng = random.Random(seed)
    steps = max(3, int(round((t1 - t0) * 10)))
    ts = [t0 + (t1 - t0) * i / steps for i in range(steps + 1)]
    prof = []
    for k, t in enumerate(ts):
        prof.append((_spike_r(t), t * SPIKE_L))
    s = lathe([(0, prof[0][1])] + prof + [(0, prof[-1][1])], seg=8)
    off = Vector((seed * 1.3, seed * 0.7, 0.2))

    def warp(p):
        t = p.y / SPIKE_L
        r = math.hypot(p.x, p.z)
        # granular surface, crumbled edges
        g = 0.00012 * noise.noise(p * 2600 + off) + 0.0001 * noise.noise(p * 900 + off)
        q = Vector((p.x, p.y, p.z))
        if r > 1e-6:
            q.x += p.x / r * g
            q.z += p.z / r * g
        # a break: the end face torn, not flat
        for tb, rough in ((t0, rough0), (t1, rough1)):
            if rough and abs(t - tb) < 1e-6:
                q.y += 0.0011 * noise.noise(p * 700 + off) + 0.0006 * (p.x / SPIKE_R)
        return q
    s = s.warp(warp)
    # along +x, a flat down (the octagon's lathe puts a vertex at a = 0)
    s.rot(0, math.pi / 8, 0).rot(0, 0, -math.pi / 2)
    return s


def fert_spike():
    return on_floor(_spike(0.0, 1.0, False, False, 3), centre=False).move(0, 0, 0)


def fert_spike_head():
    return on_floor(_spike(0.0, 0.58, False, True, 5), centre=False)


def fert_spike_tip():
    s = _spike(0.58, 1.0, True, False, 7)
    lo, hi = s.bounds()
    # lying turned on the floor a little apart from where it broke
    s.move(-lo.x, 0, 0).rot(0, 0.5, 0).move(0.004, 0, 0.012)
    return on_floor(s, centre=False)


def fert_crumbs():
    rng = random.Random(11)
    out = Shape()
    for k in range(9):
        a = rng.random() * TAU
        d = 0.004 + 0.022 * rng.random() ** 1.5
        r = 0.0005 + 0.0011 * rng.random()
        out = out + lump(rng, (math.cos(a) * d, r * 0.6, math.sin(a) * d), r, k=8, squash=0.7)
    return on_floor(out, centre=False)


auto('Real__fertSpike', fert_spike, 200)
auto('Real__fertSpikeHead', fert_spike_head, 160)
auto('Real__fertSpikeTip', fert_spike_tip, 140)
auto('Real__fertCrumbs', fert_crumbs, 200)


# =========================================================== soil bag ===
# An 8 dry quart bag of potting soil (Miracle-Gro's 8 qt potting mix: 17 x
# 12 x 4 in on the shelf): a tube of printed LDPE film 305 mm flat, heat
# sealed across both ends with crimped seals 25 mm deep, 430 mm long over
# all, 8.8 litres of peat, bark and perlite in it. Lying on its back the
# soil spreads it: the film keeps its perimeter (610 mm round), so the
# middle is a low lens 85 mm high and narrower than the film; toward the
# seals it flattens and widens, the film there bunched in wrinkles running
# into the seal. Its contents have settled to one side and one end.
BAG_W, BAG_L, SEAL = 0.305, 0.430, 0.025
BAG_H = 0.085


def _lens_half_width(b, per):
    """The half width a of a lens (two arcs) of half height b whose
    perimeter is `per`."""
    lo, hi = 0.0, per / 2
    for _ in range(50):
        a = (lo + hi) / 2
        # a lens of half chord a, sagitta b: arc radius R, angle 2 phi
        if b < 1e-6:
            p = 4 * a
        else:
            R = (a * a + b * b) / (2 * b)
            phi = math.asin(min(1.0, a / R))
            p = 4 * R * phi
        if p > per:
            hi = a
        else:
            lo = a
    return (lo + hi) / 2


def _bag_point(u, v):
    """The film at u (-1..1 along the body between the seals) and v (0..1
    round the tube, 0 at the right fold, 0.5 at the left): its top and
    bottom are arcs, the bottom pressed flat on the floor."""
    L = BAG_L / 2 - SEAL
    au = abs(u)
    # the height falls to nothing at the seals (rounded pillow ends); the
    # soil has slid toward +z and toward +x
    hb = BAG_H * (1 - au ** 2.6) ** 0.55 * (1 + 0.10 * u) / 1.0
    a = _lens_half_width(hb / 2, BAG_W * 2)
    ang = v * TAU
    s = math.cos(ang)            # across: +1 the right fold
    top = math.sin(ang) >= 0
    x = a * s
    # the lens: each side an arc, the bottom one flattened against the floor
    yy = (hb / 2) * math.sqrt(max(0.0, 1 - s * s)) ** 0.8
    y = hb / 2 + (yy if top else -yy * 0.12) if hb > 1e-5 else 0.0
    # its bottom lies on the floor: the lower arc pressed flat
    if not top:
        y = max(0.0, y - hb * 0.38 * (1 - abs(s) ** 4))
    z = u * L
    # the soil settled to its right: that side fuller, the left slack
    y *= 1 + 0.08 * s * (1 - au)
    # wrinkles where the tube bunches into each seal: ridges along z,
    # fading into the body; small dents all over where the soil settled
    wr = smoothstep(0.55, 1.0, au)
    if top:
        y += wr * 0.004 * math.sin(s * 9.5 + 2.0 * u) * (1 - abs(s) ** 2)
        y += 0.0025 * n1(x * 22, z * 22, 0.3) * (1 - au)
    x += wr * 0.0025 * math.sin(s * 7 + u * 3)
    if top:
        # creases running in from the seals' corners, where the film folded
        for cu, cs, ang in ((1, 1, 2.3), (1, -1, -2.2), (-1, 1, 1.0), (-1, -1, -0.9), (1, 0.2, 1.7)):
            dx, dz = x - cs * BAG_W / 2 * 0.85, z - cu * L
            along = dx * math.cos(ang) + dz * math.sin(ang)
            across = -dx * math.sin(ang) + dz * math.cos(ang)
            if 0 < along < 0.12:
                y -= 0.0035 * math.exp(-(across / 0.004) ** 2) * (1 - along / 0.12)
    return Vector((x, max(0.0, y), z))


def soil_bag():
    nu, nv = 30, 40
    v, f = [], []
    for i in range(nu + 1):
        u = -1 + 2 * i / nu
        for j in range(nv):
            v.append(_bag_point(u, j / nv))
    for i in range(nu):
        for j in range(nv):
            j2 = (j + 1) % nv
            f.append((i * nv + j, i * nv + j2, (i + 1) * nv + j2, (i + 1) * nv + j))
    s = Shape(v, f)
    return propkit._fix_winding(weld(s, 1e-5))


def soil_bag_seals():
    out = Shape()
    L = BAG_L / 2 - SEAL
    for sg in (-1, 1):
        # the seal: two films welded, 0.35 mm, crimped across in fine ridges,
        # bent a little as it lies; one corner (+x, +z) torn away
        nx, nz = 40, 5
        v, f = [], []
        for j in range(nz + 1):
            for i in range(nx + 1):
                x = -BAG_W / 2 + BAG_W * i / nx
                z = sg * (L + SEAL * j / nz)
                y = 0.0012 + 0.0004 * math.sin(x * 900) * (0.3 < j / nz < 0.95) + 0.003 * (x / BAG_W) ** 2 + 0.002 * n1(x * 9, z * 9)
                v.append(Vector((x, y, z)))
        for j in range(nz):
            for i in range(nx):
                a = j * (nx + 1) + i
                if sg > 0 and i >= nx - 6 and j >= nz - 1 - (i - (nx - 6)) // 2:
                    continue
                f.append((a, a + 1, a + nx + 2, a + nx + 1))
        sh = Shape(v, f)
        top = sh.copy().move(0, 0.00018, 0)
        bot = sh.copy().move(0, -0.00018, 0)
        bot.f = [tuple(reversed(ff)) for ff in bot.f]
        out = out + top + bot
    return out


def soil_bag_spill():
    """Soil run out of the split at the +x, +z corner: a low ragged heap
    fanned out on the floor from under the seal's torn corner (8 mm at its
    deepest), its edge broken into lumps and crumbs, a few bits of bark and
    perlite beyond it."""
    rng = random.Random(23)
    L = BAG_L / 2 - SEAL
    c = Vector((BAG_W / 2 - 0.045, 0.0, L + 0.008))

    def reach(a):
        out = 0.5 + 0.5 * math.cos(a - 0.9)
        return (0.03 + 0.085 * out ** 1.5) * (1 + 0.32 * n1(math.cos(a) * 2.1, math.sin(a) * 2.1, 4.2) + 0.12 * n1(math.cos(a) * 7, math.sin(a) * 7, 2.6))

    def mound(x, z):
        dx, dz = x - c.x, z - c.z
        return 1 - math.hypot(dx, dz) / reach(math.atan2(dz, dx))
    na, nr = 44, 10
    v, f = [Vector((c.x, 0.008, c.z))], []
    for i in range(1, nr + 1):
        t = i / nr
        for j in range(na):
            a = TAU * j / na
            r = reach(a) * t
            x, z = c.x + math.cos(a) * r, c.z + math.sin(a) * r
            h = 0.008 * (1 - t * t) ** 1.4 + 0.0024 * max(0.0, n1(x * 120, z * 120, 0.7)) * (1 - t) ** 0.6
            v.append(Vector((x, max(0.0003, h if i < nr else 0.0003), z)))
    for j in range(na):
        f.append((0, 1 + (j + 1) % na, 1 + j))
    for i in range(nr - 1):
        for j in range(na):
            j2 = (j + 1) % na
            a0, b0 = 1 + i * na, 1 + (i + 1) * na
            f.append((a0 + j, a0 + j2, b0 + j2, b0 + j))
    s = Shape(v, f)
    # lumps over it and crumbs flung beyond its edge
    for k in range(46):
        a = 0.9 + rng.gauss(0, 0.8)
        r = (0.02 + 0.11 * rng.random()) * (0.6 + 0.4 * (0.5 + 0.5 * math.cos(a - 0.9)))
        x, z = c.x + math.cos(a) * r, c.z + math.sin(a) * r
        rr = 0.0012 + 0.0035 * rng.random() ** 2
        m = max(0.0, mound(x, z))
        base = 0.008 * (1 - (1 - m) ** 2) ** 1.4 if m > 0 else 0.0
        s = s + lump(rng, (x, base + rr * 0.4, z), rr, k=7, squash=0.6)
    return propkit._fix_winding(s)


auto('Real__soilBag', soil_bag, 2600, smooth=50.0)
auto('Real__soilBagSeals', lambda: floor_clamp(soil_bag_seals()), 1600)
auto('Real__soilBagSpill', lambda: floor_clamp(soil_bag_spill()), 1400, smooth=60.0)


# ========================================================== bird bath ===
# A small cast concrete pedestal bird bath (the Sunnydaze Grecian, 16 in
# across and 20 in tall; Massarelli's are the same shape larger): two
# castings, a pedestal (a stepped round plinth 260 mm across, a column
# tapering from 120 to 95 mm with a collar at its foot and its head, a
# socket at its top) and a basin 410 mm across, its rim rolled, 45 mm of
# water at its middle, a spigot under it sitting in the socket.
BB_PED_H = 0.430
BB_BOWL_R = 0.205
BB_RIM_Y = BB_PED_H + 0.088        # the rim's top
BB_DISH_Y = BB_PED_H + 0.040       # the dish's lowest point inside


def _concrete(s, seed, amount=0.0007):
    """Cast concrete: a little lumpy, pitted where air was trapped."""
    off = Vector((seed * 3.3, seed * 1.1, 0.0))

    def w(p):
        r = math.hypot(p.x, p.z)
        g = amount * noise.noise(p * 30 + off) + amount * 0.6 * max(0.0, noise.noise(p * 140 + off) - 0.3)
        if r < 1e-6:
            return p
        return Vector((p.x + p.x / r * g, p.y + 0.3 * g, p.z + p.z / r * g))
    return s.warp(w)


def bird_bath_pedestal():
    y0 = 0.0
    prof = [(0.0, 0.0), (0.128, 0.0), (0.130, 0.004), (0.130, 0.032), (0.124, 0.038),       # the plinth
            (0.104, 0.040), (0.102, 0.058), (0.094, 0.064),                                  # its step
            (0.072, 0.068), (0.074, 0.080), (0.068, 0.090), (0.060, 0.094),                  # the foot collar
            (0.058, 0.110), (0.054, 0.250), (0.049, 0.360),                                  # the column
            (0.054, 0.368), (0.058, 0.378), (0.056, 0.390),                                  # its head collar
            (0.066, 0.396), (0.072, 0.410), (0.072, BB_PED_H - 0.004), (0.068, BB_PED_H),    # the socket's cup
            (0.050, BB_PED_H), (0.048, BB_PED_H - 0.020), (0.0, BB_PED_H - 0.020)]
    s = lathe(prof, seg=36)
    return _concrete(s, 3)


def _bowl_profile():
    """The basin's section (r, y), from the spigot's foot round under the
    basin and over the rim into the dish to the middle: outside then
    inside."""
    yb = BB_PED_H - 0.018
    outside = [(0.0, yb), (0.044, yb), (0.046, yb + 0.004), (0.046, BB_PED_H + 0.002),       # the spigot
               (0.07, BB_PED_H + 0.006), (0.12, BB_PED_H + 0.020), (0.165, BB_PED_H + 0.040),  # under the basin
               (0.192, BB_PED_H + 0.058), (0.203, BB_PED_H + 0.070), (BB_BOWL_R, BB_RIM_Y - 0.010),
               (0.2035, BB_RIM_Y - 0.003)]
    inside = [(0.199, BB_RIM_Y), (0.190, BB_RIM_Y - 0.002), (0.180, BB_RIM_Y - 0.010),           # the rim's roll
              (0.160, BB_DISH_Y + 0.030), (0.120, BB_DISH_Y + 0.014), (0.070, BB_DISH_Y + 0.004), (0.0, BB_DISH_Y)]
    return outside, inside


def _chip_rim(s, seed):
    """Two chips knocked out of the rim's edge."""
    rng = random.Random(seed)
    for ang in (0.7, 3.9):
        c = box(0.03, 0.02, 0.03).rot(rng.random(), rng.random(), rng.random())
        c.move(math.cos(ang) * (BB_BOWL_R + 0.008), BB_RIM_Y - 0.002, math.sin(ang) * (BB_BOWL_R + 0.008))
        s = cut(s, c)
    return s


def bird_bath_bowl_out():
    outside, _ = _bowl_profile()
    s = lathe(outside, seg=48, cap_bottom=False, cap_top=False)
    return _concrete(s, 5)


def bird_bath_bowl_in():
    outside, inside = _bowl_profile()
    s = lathe([outside[-1]] + inside, seg=48, cap_bottom=False, cap_top=False)
    return _concrete(s, 5, amount=0.0005)


auto('Real__birdBathPedestal', lambda: floor_clamp(bird_bath_pedestal()), 1800, smooth=40.0)
auto('Real__birdBathBowlOut', bird_bath_bowl_out, 1100, smooth=40.0)
auto('Real__birdBathBowlIn', bird_bath_bowl_in, 900, smooth=40.0)


# ====================================================== hanging plant ===
# A golden pothos (Epipremnum aureum) left for years in a 6.5 in moulded
# plastic hanging pot (165 mm across its rim, 115 mm deep, its saucer
# clipped under it) on a three-wire hanger, hung from a wrought iron wall
# bracket (Panacea's 8 in curved bracket: a flat wall plate, a square arm
# curled up at its end, a scroll under it). The pothos has gone leggy: bare
# vines half a metre long, a leaf at every few nodes, most at the tips,
# some yellow, some dried brown, aerial roots at the bare nodes.
# The wall frame: the wall z = 0, y up from the floor.
HP_HOOK = Vector((0.0, 1.958, 0.190))     # where the hanger hangs on the arm
HP_POT = Vector((0.0, 1.645, 0.190))      # the rim's top, on the pot's axis
HP_RIM_R = 0.0825
HP_POT_H = 0.115
HP_SOIL_Y = HP_POT.y - 0.024


def hang_plant_bracket():
    out = Shape()
    # the wall plate: a flat bar 22 x 5 mm, 160 mm, its ends rounded
    plate = prism(rounded_rect(0.022, 0.160, 0.010, seg=3), 0.005, bevel_r=0.0012).move(0, 1.910, 0.0025)
    holes = [cyl(0.0028, h=0.03, seg=10).rot(math.pi / 2, 0, 0).move(0, y, 0) for y in (1.848, 1.972)]
    out = out + cut(plate, *holes)
    # the arm: 10 mm square bar out from the plate's top, its end curled up
    # into the hook the hanger rides in
    pts = [(0, 1.958, 0.004), (0, 1.958, 0.06), (0, 1.958, 0.150), (0, 1.957, 0.185)]
    for k in range(1, 9):
        a = math.pi * 1.15 * k / 8
        pts.append((0, 1.958 + 0.012 - 0.012 * math.cos(a) + 0.0, 0.185 + 0.012 * math.sin(a) + 0.008 * (k / 8)))
    pts = chaikin(pts, 1)
    out = out + sweep(pts, [0.0052] * (len(pts) - 1) + [0.0045], seg=4, cap0=True, cap1='dome', up=(1, 0, 0))
    # the scroll: a flat bar from the plate's foot out and up under the arm,
    # ending in a curl
    sc = [(0, 1.852, 0.004), (0, 1.86, 0.03), (0, 1.885, 0.07), (0, 1.915, 0.10), (0, 1.94, 0.115)]
    cx, cy, r0 = 0.0, 1.928, 0.022
    for k in range(1, 14):
        a = math.pi * 0.5 - k * 0.42
        rr = r0 * (1 - k / 18)
        sc.append((0, cy + rr * math.sin(a) + 0.0, 0.118 + rr * math.cos(a) - 0.0))
    sc = chaikin(sc, 1)
    out = out + sweep(sc, [0.0035] * len(sc), seg=4, cap0=True, cap1='dome', up=(1, 0, 0))
    # forging: a little uneven
    off = Vector((3.1, 0.4, 1.7))
    return out.warp(lambda p: p + Vector((0.0004 * noise.noise(p * 60 + off), 0.0003 * noise.noise(p * 60 + off * 2), 0.0)))


def hang_plant_screws():
    out = Shape()
    for y in (1.848, 1.972):
        out = out + screw_head(0.0042, 0.0022).rot(math.pi / 2, 0, 0).move(0, y, 0.005).rot(0, 0, 0)
    return out


def _hanger_ends():
    """The three wires' ends on the rim (the clips), round the pot's axis."""
    return [Vector((HP_POT.x + math.sin(a) * (HP_RIM_R - 0.004), HP_POT.y + 0.001, HP_POT.z + math.cos(a) * (HP_RIM_R - 0.004)))
            for a in (math.radians(20), math.radians(140), math.radians(260))]


def hang_plant_hanger():
    out = Shape()
    ring_c = Vector((HP_HOOK.x, HP_HOOK.y - 0.055, HP_HOOK.z))
    # the S hook over the arm's curl down to the ring
    s_pts = []
    for k in range(10):
        a = math.pi * 1.3 * k / 9 - 0.15
        s_pts.append(HP_HOOK + Vector((0, 0.010 + 0.011 * math.cos(a), -0.011 * math.sin(a) + 0.0)))
    s_pts += [HP_HOOK + Vector((0, -0.012, 0.004)), HP_HOOK + Vector((0, -0.030, 0.002)), ring_c + Vector((0, 0.013, 0))]
    out = out + sweep(chaikin(s_pts, 1), [0.0016] * (len(chaikin(s_pts, 1))), seg=6, cap0='dome', cap1='dome')
    out = out + torus(0.011, 0.0018, seg=16, tseg=6).rot(0, math.pi / 2, 0).move(*ring_c)
    # three wires from the ring to the rim's clips, each hooked at its end
    for e in _hanger_ends():
        top = ring_c + (e - ring_c).normalized() * 0.006 + Vector((0, -0.006, 0))
        d = (e - top)
        pts = [top + d * (k / 6) for k in range(7)]
        out_dir = Vector((e.x - HP_POT.x, 0, e.z - HP_POT.z)).normalized()
        pts += [e + out_dir * 0.004 + Vector((0, -0.002, 0)), e + out_dir * 0.006 + Vector((0, -0.010, 0))]
        out = out + sweep(pts, [0.0013] * len(pts), seg=5, cap0='dome', cap1='dome')
    return out


def hang_plant_pot():
    """The pot and its clipped-on saucer: moulded polypropylene 2.5 mm,
    its rim rolled out, three slots for the hanger's clips."""
    y0, y1 = HP_POT.y - HP_POT_H, HP_POT.y
    prof = [(0.0, y0 + 0.004), (0.056, y0 + 0.004), (0.060, y0), (0.068, y0 + 0.010), (0.0755, y1 - 0.016),
            (0.0775, y1 - 0.010), (0.083, y1 - 0.008), (0.0838, y1 - 0.002), (0.0825, y1), (0.0790, y1 - 0.001),
            (0.0752, y1 - 0.006), (0.0735, y1 - 0.016), (0.0680, y0 + 0.020), (0.0, y0 + 0.020)]
    pot = lathe(prof, seg=40, cap_bottom=False, cap_top=False)
    # moulded ribs under the rim (stiffeners) and the foot ring
    ribs = Shape()
    for k in range(12):
        a = TAU * k / 12 + 0.13
        ribs = ribs + box(0.003, 0.010, 0.006).move(0, y1 - 0.013, 0.0785).rot(0, a, 0)
    pot = pot + ribs
    # the saucer: a shallow dish clipped under it
    sy = y0 - 0.014
    sp = [(0.0, sy), (0.064, sy), (0.068, sy + 0.002), (0.072, sy + 0.020), (0.0745, sy + 0.022), (0.0735, sy + 0.024),
          (0.0700, sy + 0.022), (0.0665, sy + 0.004), (0.0, sy + 0.004)]
    pot = pot + lathe(sp, seg=40, cap_bottom=False, cap_top=False)
    pot = pot.move(HP_POT.x, 0, HP_POT.z)
    return pot


def hang_plant_soil():
    """The soil: dried and shrunk from the pot's wall, cracked, sunk at the
    middle where the vines come out."""
    na, nr = 32, 6
    c = HP_POT
    v, f = [Vector((c.x, HP_SOIL_Y - 0.004, c.z))], []
    for i in range(1, nr + 1):
        t = i / nr
        for j in range(na):
            a = TAU * j / na
            r = (0.0705 - 0.004 * (0.5 + 0.5 * n1(math.cos(a) * 2, math.sin(a) * 2, 1.3))) * t
            y = HP_SOIL_Y - 0.004 * (1 - t) + 0.0015 * n1(math.cos(a) * r * 60, math.sin(a) * r * 60, 2.1)
            if i == nr:
                y -= 0.006
            v.append(Vector((c.x + math.sin(a) * r, y, c.z + math.cos(a) * r)))
    for j in range(na):
        f.append((0, 1 + j, 1 + (j + 1) % na))
    for i in range(nr - 1):
        for j in range(na):
            j2 = (j + 1) % na
            a0, b0 = 1 + i * na, 1 + (i + 1) * na
            f.append((a0 + j, b0 + j, b0 + j2, a0 + j2))
    return propkit._fix_winding(Shape(v, f))


# ---- the pothos ----
def leaf_blade(rng, L, state):
    """A pothos leaf blade, base at the origin, along +y (tip), its upper
    face +z: cordate, acuminate, folded a little along the midrib, arched;
    `state` 'green', 'yellow' (limp) or 'dead' (shrunk, curled, rolled)."""
    W = L * (0.66 if state != 'dead' else 0.5)
    nu, nv = 9, 4
    fold = 0.18 + 0.1 * rng.random() + (0.5 if state == 'dead' else 0.0)
    arch = (0.10 + 0.08 * rng.random()) * (1.6 if state == 'yellow' else 1.0)
    curl = 0.0 if state != 'dead' else 0.35 + 0.2 * rng.random()
    twist = (rng.random() - 0.5) * 0.4
    off = Vector((rng.random() * 50, rng.random() * 50, 0))
    v, f = [], []
    for i in range(nu + 1):
        a = i / nu
        # the cordate outline: the lobes round the sinus, widest at a quarter,
        # drawn to an acuminate tip
        e = (math.sin(math.pi * min(1.0, (a + 0.12) / 1.12) ** 0.75) ** 0.9) * (1 - 0.2 * a ** 3)
        e = max(e, 0.03)
        for j in range(nv * 2 + 1):
            b = -1 + j / nv
            x = b * e * W / 2
            y = a * L - 0.10 * L * (1 - a) ** 3 * abs(b) ** 1.5     # the lobes reach back
            z = fold * abs(x) - arch * L * (a * a) - curl * W * (abs(b) ** 2) * (0.6 + a)
            # a twist along it, and the margin's waves
            ang = twist * a
            x, z = x * math.cos(ang) - z * math.sin(ang), x * math.sin(ang) + z * math.cos(ang)
            z += 0.0012 * noise.noise(Vector((x * 80, y * 80, 0)) + off) * abs(b)
            v.append(Vector((x, y, z)))
    for i in range(nu):
        for j in range(nv * 2):
            a0 = i * (nv * 2 + 1) + j
            f.append((a0, a0 + 1, a0 + nv * 2 + 2, a0 + nv * 2 + 1))
    return Shape(v, f)


def _orient(s, origin, dir_y, face_z):
    """Turn a shape built along +y (face +z) so +y runs along dir_y, its
    face as near face_z as it can, its origin at `origin`."""
    y = Vector(dir_y).normalized()
    z = Vector(face_z)
    z = (z - y * z.dot(y))
    if z.length < 1e-6:
        z = Vector((1, 0, 0)) - y * y.x
    z.normalize()
    x = y.cross(z)
    m = Matrix(((x.x, y.x, z.x, 0), (x.y, y.y, z.y, 0), (x.z, y.z, z.z, 0), (0, 0, 0, 1)))
    return s.apply(m).move(*origin)


# Each vine: (seed, length, how far it wanders, how far out it drifts)
VINES = {'A': (31, 0.62, 0.05, 0.035), 'B': (32, 0.46, 0.07, 0.05), 'C': (33, 0.34, 0.04, 0.03), 'D': (34, 0.54, 0.06, 0.06)}
_VINE_CACHE = {}


def vine(which):
    """A vine in its own frame: the rim's top where it crosses at the
    origin, its outward +z (the pot's axis at z = -HP_RIM_R), up +y; from
    the soil up over the rim and hanging down. Returns its parts: stem,
    green leaves, yellow leaves, dead leaves (each a Shape), and roots."""
    if which in _VINE_CACHE:
        return _VINE_CACHE[which]
    seed, L, wander, drift = VINES[which]
    rng = random.Random(seed)
    soil = Vector((0.0, HP_SOIL_Y - HP_POT.y - 0.002, -HP_RIM_R + 0.030 + 0.02 * rng.random()))
    pts = [soil, soil + Vector((0, 0.020, 0.012)), Vector((0, 0.016, -0.018)), Vector((0, 0.011, 0.002)),
           Vector((0, 0.002, 0.018)), Vector((0, -0.022, 0.030))]
    # down the vine: a gentle wander, drifting out and back
    n = 16
    ph = rng.random() * 6
    for k in range(1, n + 1):
        t = k / n
        y = -0.022 - L * t
        x = wander * math.sin(t * 2.3 + ph) * t + 0.01 * math.sin(t * 9 + ph)
        z = 0.030 + drift * math.sin(t * 1.7 + ph * 0.5) * t + 0.008 * math.sin(t * 7)
        pts.append(Vector((x, y, z)))
    pts = chaikin(pts, 1)
    # the stem thins toward its tip
    nn = len(pts)
    radii = [0.0024 - 0.0009 * (k / (nn - 1)) for k in range(nn)]
    stem = sweep(pts, radii, seg=5, cap0=True, cap1='dome')
    # arc length along it
    acc = [0.0]
    for a, b in zip(pts, pts[1:]):
        acc.append(acc[-1] + (b - a).length)
    total = acc[-1]

    def at(s):
        for k in range(1, nn):
            if acc[k] >= s:
                t = (s - acc[k - 1]) / max(1e-9, acc[k] - acc[k - 1])
                p = pts[k - 1].lerp(pts[k], t)
                d = (pts[k] - pts[k - 1]).normalized()
                return p, d
        return pts[-1], (pts[-1] - pts[-2]).normalized()
    green, yellow, dead, roots = Shape(), Shape(), Shape(), Shape()
    s = 0.06 + 0.02 * rng.random()
    side = 1
    while s < total - 0.01:
        p, d = at(s)
        frac = s / total
        # leggy: the old half of the vine mostly bare
        keep = rng.random() < (0.25 + 0.85 * frac ** 1.2)
        side = -side
        out = Vector((side * 0.9, 0.25, 0.7)).normalized()
        if keep:
            r = rng.random()
            state = 'green' if r < 0.72 - 0.25 * (1 - frac) else ('yellow' if r < 0.88 else 'dead')
            Lb = (0.075 - 0.035 * frac) * (0.85 + 0.3 * rng.random())
            if state == 'dead':
                Lb *= 0.8
            # the petiole: out from the node, bending so the blade hangs
            lp = 0.025 + 0.02 * rng.random()
            c1 = p + out * lp * 0.5 + Vector((0, 0.004, 0))
            c2 = p + out * lp + Vector((0, -0.006, 0)) + Vector((0, 0, 0.004))
            pp = chaikin([p, c1, c2], 1)
            pet = sweep(pp, [0.0011 - 0.0003 * k / (len(pp) - 1) for k in range(len(pp))], seg=4, cap0=False, cap1=True)
            blade = leaf_blade(rng, Lb, state)
            hang = Vector((out.x * 0.5, -0.85 - 0.2 * rng.random(), 0.35)).normalized()
            face = Vector((side * 0.3, 0.4, 1.0))
            blade = _orient(blade, c2, hang, face)
            if state == 'green':
                green = green + blade
                stem = stem + pet
            elif state == 'yellow':
                yellow = yellow + blade
                stem = stem + pet
            else:
                dead = dead + blade + pet
        else:
            # a bare node: the scar and a short brown aerial root
            rd = Vector((-side * 0.4, -0.5, 0.6)).normalized()
            rl = 0.006 + 0.012 * rng.random()
            roots = roots + sweep([p, p + rd * rl * 0.5, p + rd * rl + Vector((0, -0.003, 0))], [0.0012, 0.001, 0.0007], seg=4, cap0=False, cap1='dome')
        s += 0.04 + 0.025 * rng.random()
    _VINE_CACHE[which] = (stem, green, yellow, dead, roots)
    return _VINE_CACHE[which]


for _w in VINES:
    auto(f'Real__hangPlantVine{_w}Stem', lambda _w=_w: vine(_w)[0], 900, smooth=60.0)
    auto(f'Real__hangPlantVine{_w}Green', lambda _w=_w: vine(_w)[1], 1300, smooth=60.0)
    auto(f'Real__hangPlantVine{_w}Yellow', lambda _w=_w: vine(_w)[2], 600, smooth=60.0)
    auto(f'Real__hangPlantVine{_w}Dead', lambda _w=_w: vine(_w)[3] + vine(_w)[4], 600, smooth=60.0)


def crown(which):
    """What grows at the pot's top: short shoots from the soil with a few
    leaves held up over the rim, and dead leaves dropped on the soil (the
    pot's frame)."""
    rng = random.Random(41)
    stems, green, yellow, dead = Shape(), Shape(), Shape(), Shape()
    states = ['green', 'green', 'yellow', 'green', 'dead', 'green', 'green', 'yellow', 'green', 'green', 'dead', 'green']
    n = 0
    for k in range(7):
        a = TAU * k / 7 + rng.random() * 0.6
        r0 = 0.010 + 0.03 * rng.random()
        base = Vector((HP_POT.x + math.sin(a) * r0, HP_SOIL_Y - 0.002, HP_POT.z + math.cos(a) * r0))
        out = Vector((math.sin(a), 0, math.cos(a)))
        upright = k % 3 == 0
        if upright:
            # a shoot standing up from the soil, its leaf held up and out
            path = [base, base + Vector((0, 0.03, 0)) + out * 0.008, base + Vector((0, 0.055 + 0.02 * rng.random(), 0)) + out * 0.02]
        else:
            # a shoot grown up and over the rim, its leaves hanging outside
            rim = Vector((HP_POT.x, HP_POT.y, HP_POT.z)) + out * (HP_RIM_R + 0.004)
            path = [base, base.lerp(rim, 0.5) + Vector((0, 0.03, 0)), rim + Vector((0, 0.012, 0)), rim + out * 0.012 + Vector((0, 0.002, 0))]
        path = chaikin(path, 1)
        stems = stems + sweep(path, [0.0025 - 0.0006 * j / (len(path) - 1) for j in range(len(path))], seg=5, cap0=False, cap1='dome')
        for m in range(2 if k < 5 else 1):
            state = states[n % len(states)]
            n += 1
            bl = leaf_blade(rng, 0.055 + 0.03 * rng.random(), state)
            side = Vector((math.cos(a), 0, -math.sin(a))) * (m - 0.5) * 0.9
            if upright:
                hang = (out * 0.8 + side + Vector((0, 0.35 + 0.3 * rng.random(), 0))).normalized()
            else:
                hang = (out * 0.45 + side * 0.6 + Vector((0, -0.9, 0))).normalized()
            at_ = path[-1] if m == 0 else path[-3]
            tip = at_ + hang * 0.012 + Vector((0, 0.004, 0)) + out * 0.004
            pet = sweep([at_, tip], [0.0011, 0.0009], seg=4, cap0=False, cap1=True)
            bl = _orient(bl, tip, hang, out + Vector((0, 0.6, 0)))
            if state == 'green':
                green = green + bl
                stems = stems + pet
            elif state == 'yellow':
                yellow = yellow + bl
                stems = stems + pet
            else:
                dead = dead + bl + pet
    # leaves fallen on the soil, dried flat and curled
    for k in range(3):
        a = rng.random() * TAU
        r0 = 0.035 + 0.02 * rng.random()
        p = Vector((HP_POT.x + math.sin(a) * r0, HP_SOIL_Y + 0.002, HP_POT.z + math.cos(a) * r0))
        bl = leaf_blade(rng, 0.045, 'dead')
        bl = _orient(bl, p, Vector((-math.sin(a + 0.4), 0.04, -math.cos(a + 0.4))), Vector((0, 1, 0)))
        dead = dead + bl
    return {'Stem': stems, 'Green': green, 'Yellow': yellow, 'Dead': dead}[which]


auto('Real__hangPlantBracket', hang_plant_bracket, 1100, smooth=40.0)
auto('Real__hangPlantScrews', hang_plant_screws, 200)
auto('Real__hangPlantHanger', hang_plant_hanger, 1200, smooth=60.0)
auto('Real__hangPlantPot', hang_plant_pot, 1800, smooth=40.0)
auto('Real__hangPlantSoil', hang_plant_soil, 500, smooth=60.0)
for _k in ('Stem', 'Green', 'Yellow', 'Dead'):
    auto(f'Real__hangPlantCrown{_k}', lambda _k=_k: crown(_k), 1200, smooth=60.0)


# ========================================================= herb bundle ===
# Herbs hung to dry (three to eight stems a bunch, tied at their cut ends
# with cotton string and hung head down from a cup hook; dry in two weeks,
# and left hanging for years): garden sage, rosemary or lavender. Each stem
# in its own frame: the binding at the origin, the stem hanging down -y,
# its cut end up through the binding; leaves facing about +z (the room).
# The wall frame for the hook and the string: the wall z = 0.
HB_HOOK = Vector((0.0, 1.918, 0.0))
HB_TIE = Vector((0.0, 1.874, 0.026))     # the binding's middle


def herb_hook():
    """A brass cup hook (1 in): its collar against the wall, the wire out
    and turned up into the hook."""
    collar = cyl(0.0042, h=0.0022, seg=12, bevel_r=0.0006).rot(math.pi / 2, 0, 0).move(0, 0, 0.0011)
    pts = [(0, 0, 0.002), (0, 0, 0.010), (0, -0.002, 0.016)]
    for k in range(1, 8):
        a = math.pi * k / 7
        pts.append((0, -0.002 - 0.0055 * math.sin(a), 0.0215 - 0.0055 * math.cos(a) + 0.0))
    pts.append((0, 0.004, 0.026))
    wire = sweep(chaikin(pts, 1), [0.00125] * len(chaikin(pts, 1)), seg=6, cap0=True, cap1='dome')
    return (collar + wire).move(*HB_HOOK)


def herb_twine():
    """Cotton string: wound six turns round the cut ends, knotted, its loop
    up over the hook, a loose end hanging."""
    out = Shape()
    c = HB_TIE
    for k in range(6):
        y = c.y + 0.004 - k * 0.0018
        r = 0.0092 + 0.0004 * math.sin(k * 2.1)
        pts = [Vector((c.x + math.cos(a) * r, y - 0.0018 * a / TAU, c.z + math.sin(a) * r * 0.85)) for a in [TAU * i / 14 for i in range(15)]]
        out = out + sweep(pts, [0.0009] * len(pts), seg=4, cap0=False, cap1=False)
    # the knot at the front, the loop up over the hook's bend
    knot = Vector((c.x + 0.002, c.y + 0.003, c.z + 0.0085))
    out = out + lump(random.Random(5), knot, 0.0022, k=10, squash=0.8)
    hook_bend = HB_HOOK + Vector((0, -0.0075, 0.0215))
    loop = [knot + Vector((-0.002, 0.002, -0.001)), knot.lerp(hook_bend, 0.5) + Vector((-0.004, 0, -0.004)), hook_bend + Vector((-0.002, -0.001, -0.0015)),
            hook_bend + Vector((0, 0.0012, 0.0)), hook_bend + Vector((0.002, -0.001, 0.0015)), knot.lerp(hook_bend, 0.5) + Vector((0.004, 0, -0.002)),
            knot + Vector((0.002, 0.002, -0.001))]
    out = out + sweep(chaikin(loop, 2), [0.0009] * len(chaikin(loop, 2)), seg=4, cap0='dome', cap1='dome')
    tail = [knot, knot + Vector((0.004, -0.012, 0.004)), knot + Vector((0.003, -0.028, 0.006))]
    out = out + sweep(chaikin(tail, 1), [0.0008] * len(chaikin(tail, 1)), seg=4, cap0=False, cap1='dome')
    return out


def _stem_path(rng, L, bend):
    """A stem from its cut end (above the binding) down: a gentle curve."""
    pts = []
    n = 10
    ph = rng.random() * 6
    for k in range(n + 1):
        t = k / n
        y = 0.022 - (L + 0.022) * t
        x = bend * math.sin(t * 2.2 + ph) * t
        z = bend * 0.6 * math.sin(t * 1.7 + ph * 1.3) * t
        pts.append(Vector((x, y, z)))
    return pts


def _along(pts, s):
    acc = 0.0
    for a, b in zip(pts, pts[1:]):
        d = (b - a).length
        if acc + d >= s:
            t = (s - acc) / max(d, 1e-9)
            return a.lerp(b, t), (b - a).normalized()
        acc += d
    return pts[-1], (pts[-1] - pts[-2]).normalized()


def _plen(pts):
    return sum((b - a).length for a, b in zip(pts, pts[1:]))


def sage_leaf(rng, L):
    """A dried sage leaf, base at the origin, along +y: oblong, its margin
    rolled under, crinkled (the rugose surface), drooping."""
    W = L * 0.32
    nu, nv = 7, 3
    v, f = [], []
    off = Vector((rng.random() * 40, rng.random() * 40, 0))
    roll = 0.5 + 0.3 * rng.random()
    droop = 0.15 + 0.15 * rng.random()
    for i in range(nu + 1):
        a = i / nu
        e = max(0.05, math.sin(math.pi * min(1.0, a ** 0.85)) ** 0.7)
        for j in range(nv * 2 + 1):
            b = -1 + j / nv
            x = b * e * W / 2
            y = a * L
            z = -roll * W * 0.5 * b * b - droop * L * a * a + 0.0007 * noise.noise(Vector((x * 300, y * 300, 0)) + off)
            v.append(Vector((x, y, z)))
    for i in range(nu):
        for j in range(nv * 2):
            a0 = i * (nv * 2 + 1) + j
            f.append((a0, a0 + 1, a0 + nv * 2 + 2, a0 + nv * 2 + 1))
    return Shape(v, f)


def herb_sage(seed):
    rng = random.Random(seed)
    L = 0.20 + 0.05 * rng.random()
    pts = _stem_path(rng, L, 0.012)
    stem = sweep(pts, [0.0016 - 0.0006 * k / (len(pts) - 1) for k in range(len(pts))], seg=4, cap0=True, cap1='dome')
    total = _plen(pts)
    s = 0.06 + 0.02 * rng.random()
    turn = rng.random() * TAU
    while s < total - 0.004:
        p, d = _along(pts, s)
        frac = s / total
        for sg in (-1, 1):
            if rng.random() < 0.18:
                continue          # a leaf dropped
            # the pair's leaves out to each side, turned 90 degrees a node
            side = Vector((math.cos(turn) * sg, 0, math.sin(turn) * sg * 0.6 + 0.4))
            Ll = (0.032 + 0.022 * rng.random()) * (1.0 - 0.35 * frac)
            hang = (d * 0.75 + side * 0.55).normalized()
            pet = sweep([p, p + hang * 0.006], [0.0007, 0.0006], seg=3, cap0=False, cap1=True)
            lf = _orient(sage_leaf(rng, Ll), p + hang * 0.006, hang, Vector((0, 0, 1)) + side * 0.3)
            stem = stem + pet + lf
        turn += math.pi / 2 + 0.2 * rng.random()
        s += 0.022 + 0.012 * rng.random()
    return stem


def _needle(rng, L):
    """A rosemary needle along +y: a narrow strip folded down its midrib,
    curved a little."""
    v = []
    n = 3
    for i in range(n + 1):
        t = i / n
        w = 0.0011 * (1 - 0.7 * t ** 2)
        y = L * t
        z = -0.0025 * t * t
        v += [Vector((-w, y, z - 0.0003)), Vector((0, y, z + 0.0004)), Vector((w, y, z - 0.0003))]
    f = []
    for i in range(n):
        a = i * 3
        f += [(a, a + 1, a + 4, a + 3), (a + 1, a + 2, a + 5, a + 4)]
    return Shape(v, f)


def herb_rosemary(seed):
    """Returns (wood, needles)."""
    rng = random.Random(seed)
    L = 0.17 + 0.05 * rng.random()
    pts = _stem_path(rng, L, 0.010)
    wood = sweep(pts, [0.0021 - 0.0011 * k / (len(pts) - 1) for k in range(len(pts))], seg=5, cap0=True, cap1='dome',
                 rough=lambda i, j, p, d: 0.00015 * noise.noise(p * 400))
    needles = Shape()
    total = _plen(pts)
    s = 0.032
    ang = rng.random() * TAU
    while s < total:
        p, d = _along(pts, s)
        frac = s / total
        if rng.random() < 0.85:
            # a whorl of two, angled toward the tip (now down)
            for k in range(2):
                a = ang + k * math.pi
                out = Vector((math.cos(a), 0, math.sin(a)))
                out = (out - d * out.dot(d)).normalized()
                dirn = (d * (0.75 + 0.15 * frac) + out * 0.6).normalized()
                nl = _orient(_needle(rng, 0.016 + 0.01 * rng.random()), p + out * 0.0015, dirn, out)
                needles = needles + nl
        ang += 2.4
        s += 0.0034 + 0.0015 * rng.random()
    return wood, needles


def herb_lavender(seed):
    """Returns (stem, spike): a long bare stem, its flower spike at the
    hanging end, a pair of narrow leaves near the cut."""
    rng = random.Random(seed)
    L = 0.21 + 0.06 * rng.random()
    pts = _stem_path(rng, L, 0.008)
    stem = sweep(pts, [0.0011 - 0.0003 * k / (len(pts) - 1) for k in range(len(pts))], seg=4, cap0=True, cap1=False)
    total = _plen(pts)
    for s in (0.03, 0.055):
        p, d = _along(pts, s)
        for sg in (-1, 1):
            out = Vector((sg, 0, 0.3)).normalized()
            hang = (d * 0.85 + out * 0.4).normalized()
            stem = stem + _orient(sage_leaf(rng, 0.025).scale(0.45, 1, 1), p, hang, Vector((0, 0, 1)))
    # the spike: whorls of calyces at the stem's end, the lower whorls apart
    end, d = _along(pts, total)
    spike = Shape()
    sl = 0.045 + 0.015 * rng.random()
    nw = 11
    for k in range(nw):
        t = k / (nw - 1)
        # the lowest whorls (nearest the stem) stand apart
        c = end + d * (-sl * (1 - t) ** 0.8)
        r = 0.0042 * math.sin(math.pi * (0.12 + 0.82 * t)) ** 0.5 + 0.0012
        for m in range(7):
            a = TAU * m / 7 + k * 0.9
            o = Vector((math.cos(a), 0, math.sin(a)))
            o = (o - d * o.dot(d)).normalized()
            spike = spike + lump(rng, c + o * r * 0.75, r * 0.6, k=6, squash=1.4)
    return stem, spike


_HERBS = {}
for _k in range(3):
    auto(f'Real__herbSage{"ABC"[_k]}', lambda _k=_k: herb_sage(201 + _k), 1200, smooth=55.0)
    auto(f'Real__herbRosemaryWood{"ABC"[_k]}', lambda _k=_k: herb_rosemary(301 + _k)[0], 400, smooth=60.0)
    auto(f'Real__herbRosemaryNeedles{"ABC"[_k]}', lambda _k=_k: herb_rosemary(301 + _k)[1], 1100, smooth=60.0)
    auto(f'Real__herbLavenderStem{"ABC"[_k]}', lambda _k=_k: herb_lavender(401 + _k)[0], 450, smooth=60.0)
    auto(f'Real__herbLavenderSpike{"ABC"[_k]}', lambda _k=_k: herb_lavender(401 + _k)[1], 1100, smooth=60.0)
auto('Real__herbHook', herb_hook, 420, smooth=60.0)
auto('Real__herbTwine', herb_twine, 1000, smooth=60.0)


# ============================================================== gnome ===
# A garden gnome of painted cast concrete, 12 in (Alpine's and Design House's
# 12 in gnomes: 305 mm tall, 125-150 mm across): a rotund old man standing
# on a rock, his hands folded on his belly over his belt, his beard to his
# belt, a pointed cap with its tip falling back; each colour its own part
# (the game paints them, faded and chipped). Facing +z.
GN_BELT_Y = 0.112


def _soft(s, levels=1):
    return subdivide(s, levels)


def gnome_base():
    """The rock he stands on: a lumpy slab, 135 x 115 mm, 24 mm high."""
    rng = random.Random(61)
    pts = []
    for k in range(26):
        a = TAU * k / 26 + rng.random() * 0.1
        r = 1 + 0.08 * math.sin(a * 3 + 1) + 0.05 * rng.random()
        pts.append(Vector((math.cos(a) * 0.066 * r, 0.0, math.sin(a) * 0.056 * r)))
        pts.append(Vector((math.cos(a) * 0.060 * r, 0.018 + 0.006 * rng.random(), math.sin(a) * 0.050 * r)))
    s = hull(pts)
    s = _soft(s, 1)
    off = Vector((1.3, 2.2, 0.4))
    s = s.warp(lambda p: Vector((p.x, max(0.0, p.y + 0.0025 * noise.noise(p * 70 + off) * min(1.0, p.y / 0.006)), p.z)))
    return s


def gnome_boots():
    out = Shape()
    for sg in (-1, 1):
        b = ellipsoid(0.017, 0.014, 0.030, seg=14).move(sg * 0.024, 0.032, 0.012)
        b = b.warp(lambda p: Vector((p.x, max(p.y, 0.022), p.z + 0.006 * max(0.0, (p.z - 0.012) / 0.03) * 0)))
        # the toe turned up a little
        b = b.warp(lambda p: Vector((p.x, p.y + 0.006 * max(0.0, (p.z - 0.025) / 0.017) ** 2, p.z)))
        out = out + b
    return out


def gnome_trousers():
    out = Shape()
    for sg in (-1, 1):
        out = out + cyl(0.016, 0.018, 0.03, seg=12).move(sg * 0.023, 0.057, 0.0)
    return out


def _coat_r(y):
    """The coat's radius at height y: the hem flared, the belly round, the
    shoulders sloping to the neck."""
    if y < 0.075:
        return 0.049 + 0.002 * (0.075 - y) / 0.02
    if y < 0.16:
        t = (y - 0.075) / 0.085
        return 0.049 + 0.010 * math.sin(math.pi * t * 0.9) - 0.010 * t ** 3
    t = (y - 0.16) / 0.02
    return 0.049 - 0.002 - 0.018 * t ** 1.5


def gnome_coat():
    ys = [0.054, 0.056, 0.065, 0.075, 0.085, 0.095, 0.105, 0.115, 0.125, 0.135, 0.145, 0.155, 0.162, 0.168, 0.174, 0.180]
    prof = [(0.0, 0.054)] + [(_coat_r(y), y) for y in ys] + [(0.0, 0.181)]
    s = lathe(prof, seg=28)
    # the belly forward, the back flat; the hem's rolled edge
    def w(p):
        r = math.hypot(p.x, p.z)
        k = max(0.0, p.z / max(r, 1e-6))
        p = Vector((p.x, p.y, p.z * 0.92))
        p.z += 0.012 * k * math.exp(-((p.y - 0.11) / 0.03) ** 2)
        # folds hanging from the belt down to the hem
        if p.y < GN_BELT_Y:
            ang = math.atan2(p.x, p.z)
            p += Vector((p.x, 0, p.z)).normalized() * 0.0016 * math.sin(ang * 7) * (GN_BELT_Y - p.y) / 0.04
        return p
    return s.warp(w)


def gnome_sleeves():
    out = Shape()
    for sg in (-1, 1):
        pts = [Vector((sg * 0.036, 0.165, -0.004)), Vector((sg * 0.052, 0.150, 0.004)), Vector((sg * 0.054, 0.132, 0.022)),
               Vector((sg * 0.038, 0.122, 0.048)), Vector((sg * 0.018, 0.120, 0.058))]
        pts = chaikin(pts, 2)
        n = len(pts)
        out = out + sweep(pts, [0.015 - 0.003 * k / (n - 1) for k in range(n)], seg=12, cap0=True, cap1=True)
        # the cuff
        out = out + torus(0.0125, 0.0025, seg=14, tseg=6).rot(0, math.atan2(pts[-1].x - pts[-2].x, pts[-1].z - pts[-2].z), 0).move(*pts[-1])
    return out


def gnome_belt():
    s = lathe([(0.0, GN_BELT_Y - 0.008)] + [(_coat_r(y) + 0.0025, y) for y in (GN_BELT_Y - 0.008, GN_BELT_Y + 0.008)] + [(0.0, GN_BELT_Y + 0.008)], seg=28)
    def w(p):
        r = math.hypot(p.x, p.z)
        k = max(0.0, p.z / max(r, 1e-6))
        p = Vector((p.x, p.y, p.z * 0.92))
        p.z += 0.012 * k * math.exp(-((p.y - 0.11) / 0.03) ** 2)
        return p
    return s.warp(w)


def gnome_buckle():
    zf = (_coat_r(GN_BELT_Y) + 0.0025) * 0.92 + 0.012 * math.exp(-((GN_BELT_Y - 0.11) / 0.03) ** 2)
    outer = prism(rounded_rect(0.024, 0.020, 0.003), 0.003, bevel_r=0.0008)
    inner = prism(rounded_rect(0.014, 0.010, 0.002), 0.01)
    b = cut(outer, inner)
    # the hands cover its top half, so it shows below them
    return b.move(0, GN_BELT_Y - 0.002, zf + 0.0012)


def gnome_skin():
    """The face (cheeks, nose, the brow under the cap), the ears and the
    hands folded on the belly."""
    out = Shape()
    head = ellipsoid(0.033, 0.034, 0.032, seg=20).move(0, 0.206, 0.006)
    out = out + head
    for sg in (-1, 1):
        # the cheeks: round, rosy, set out beside the nose and half under the
        # moustache (not as big as the nose)
        out = out + ellipsoid(0.010, 0.0085, 0.008, seg=12).move(sg * 0.019, 0.200, 0.029)
        out = out + ellipsoid(0.006, 0.010, 0.004, seg=10).rot(0, sg * 0.3, sg * 0.2).move(sg * 0.033, 0.207, 0.004)   # ears
    out = out + ellipsoid(0.0115, 0.0125, 0.012, seg=14).move(0, 0.203, 0.041)                  # the nose, bulbous
    # the hands: mittens with a thumb, one over the other
    for sg, dz, dy in ((-1, 0.0, 0.0), (1, 0.004, 0.006)):
        h = ellipsoid(0.014, 0.010, 0.0085, seg=12).rot(0, 0, sg * 0.4).move(sg * 0.008, 0.121 + dy, 0.060 + dz)
        th = ellipsoid(0.004, 0.007, 0.004, seg=8).rot(0, 0, -sg * 0.5).move(sg * 0.002, 0.128 + dy, 0.066 + dz)
        out = out + h + th
    return out


def _gnome_front(x, y):
    """The z of the body's front behind the beard at (x, y): the coat and
    its belly, or the face."""
    if y < 0.172:
        r = _coat_r(y) * 0.92 + 0.012 * math.exp(-((y - 0.11) / 0.03) ** 2)
        return math.sqrt(max(0.0, r * r - x * x))
    q = 1 - (x / 0.033) ** 2 - ((y - 0.206) / 0.034) ** 2
    return 0.006 + 0.032 * math.sqrt(max(0.0, q))


def gnome_beard():
    """The beard: from the cheeks down over the chest to a rounded point at
    the belt, in locks, thick at the chin; the moustache under the nose;
    the eyebrows."""
    nu, nv = 12, 12
    v, f = [], []
    for side in (0, 1):
        for j in range(nv + 1):
            t = j / nv                       # 0 at the cheeks, 1 at the point
            y = 0.197 - 0.080 * t
            hw = 0.035 * (1 - t ** 1.7) + 0.003
            for i in range(nu + 1):
                u = -1 + 2 * i / nu
                x = u * hw
                zb = max(_gnome_front(x, y), _gnome_front(x, min(y + 0.006, 0.24)) - 0.004)
                if side == 0:
                    th = (0.004 + 0.018 * math.sin(math.pi * min(1.0, t * 1.6 + 0.08)) ** 0.7 * (1 - 0.55 * t)) * (1 - u * u) ** 0.4 + 0.0015
                    locks = 0.0022 * math.sin(u * 10 + t * 3) * (1 - abs(u) * 0.3)
                    v.append(Vector((x, y, zb + th + locks)))
                else:
                    v.append(Vector((x * 0.97, y, zb - 0.003)))
    n = (nu + 1) * (nv + 1)
    for j in range(nv):
        for i in range(nu):
            a = j * (nu + 1) + i
            f.append((a, a + 1, a + nu + 2, a + nu + 1))
            f.append((n + a, n + a + nu + 1, n + a + nu + 2, n + a + 1))
    # stitch the rims
    rim = [i for i in range(nu + 1)] + [j * (nu + 1) + nu for j in range(1, nv + 1)] + \
          [nv * (nu + 1) + i for i in range(nu - 1, -1, -1)] + [j * (nu + 1) for j in range(nv - 1, 0, -1)]
    for a, b in zip(rim, rim[1:] + rim[:1]):
        f.append((b, a, n + a, n + b))
    beard = propkit._fix_winding(Shape(v, f))
    # strands: grooves along the locks
    off = Vector((2.0, 1.0, 0.0))
    beard = beard.warp(lambda p: p + Vector((0, 0, 0.0009 * noise.noise(Vector((p.x * 400, p.y * 40, 0)) + off))))
    out = beard
    # the moustache: two curled lobes under the nose
    for sg in (-1, 1):
        pts = [Vector((sg * 0.003, 0.194, 0.046)), Vector((sg * 0.014, 0.191, 0.043)), Vector((sg * 0.024, 0.188, 0.036)), Vector((sg * 0.029, 0.193, 0.030))]
        mp = chaikin(pts, 2)
        out = out + sweep(mp, [0.0058 * (1 - 0.65 * (k / (len(mp) - 1)) ** 1.3) for k in range(len(mp))], seg=10, cap0='dome', cap1='dome')
        # the eyebrows, bushy
        out = out + ellipsoid(0.010, 0.004, 0.005, seg=10).rot(0, sg * 0.35, -sg * 0.25).move(sg * 0.013, 0.222, 0.033)
    return out


def gnome_eyes():
    out = Shape()
    for sg in (-1, 1):
        out = out + ellipsoid(0.0035, 0.004, 0.002, seg=10).rot(0, sg * 0.3, 0).move(sg * 0.012, 0.213, 0.0345)
    return out


def gnome_hat():
    """The cap: a cone from a rolled band round the head (low over the
    ears and the back of the head) up to a tip that falls back and to his
    left, soft folds where it bends."""
    n = 16
    pts = []
    for k in range(n + 1):
        t = k / n
        bend = t ** 2.2
        pts.append(Vector((0.012 * bend, 0.215 + 0.09 * t - 0.016 * bend, 0.002 - 0.036 * bend)))
    radii = [0.0385 * (1 - t) ** 0.9 + 0.0025 for t in [k / n for k in range(n + 1)]]
    hat = sweep(pts, radii, seg=24, cap0=True, cap1='dome')
    band = torus(0.0365, 0.0042, seg=28, tseg=8).rot(math.pi / 2, 0, 0).move(0, 0.215, 0.002)
    hat = hat + band
    # the brim tipped back: down over the back of the head, up over the brow
    def tip(p):
        if p.y < 0.25:
            k = min(1.0, (0.25 - p.y) / 0.03)
            back = max(0.0, -p.z / 0.038)
            side = abs(p.x) / 0.038
            p = Vector((p.x, p.y - 0.03 * k * back ** 1.2 - 0.008 * k * side ** 2 + 0.006 * k * max(0.0, p.z / 0.04), p.z - 0.004 * k * back))
        return Vector((p.x, p.y, p.z * 0.96))
    return hat.warp(tip)


auto('Real__gnomeBase', gnome_base, 900, smooth=50.0)
auto('Real__gnomeBoots', gnome_boots, 900, smooth=60.0)
auto('Real__gnomeTrousers', gnome_trousers, 300, smooth=60.0)
auto('Real__gnomeCoat', gnome_coat, 1100, smooth=60.0)
auto('Real__gnomeSleeves', gnome_sleeves, 1300, smooth=60.0)
auto('Real__gnomeBelt', gnome_belt, 300, smooth=60.0)
auto('Real__gnomeBuckle', gnome_buckle, 220, smooth=40.0)
auto('Real__gnomeSkin', gnome_skin, 1800, smooth=60.0)
auto('Real__gnomeBeard', gnome_beard, 1550, smooth=60.0)
auto('Real__gnomeEyes', gnome_eyes, 200, smooth=60.0)
auto('Real__gnomeHat', gnome_hat, 1300, smooth=60.0)


# ======================================================= seed catalogue ===
# A seed house's catalogue of the 1930s (Burpee's of 1938: 9 x 6 in, 229 x
# 152 mm), its cover framed: a stained pine moulding 20 mm wide, 220 x 300
# mm over all, glass, a cream mat with a bevelled window, the cover, a
# hardboard back. The picture frame's frame (as `t01`): x across, y up from
# its middle, z out of the wall (the wall z = 0, where the frame's back
# lies).
SC_W, SC_H = 0.220, 0.300
SC_M = 0.020                     # the moulding's face
SC_REB = 0.012                   # the rabbet's edge, in from the outside
SC_ZL = 0.0085                   # the lip's underside
SC_COVER = (0.152, 0.229)
SC_WINDOW = (0.136, 0.212)


def _clean(profile):
    out = []
    for p in profile:
        if not out or abs(p[0] - out[-1][0]) + abs(p[1] - out[-1][1]) > 1e-7:
            out.append(p)
    if abs(out[0][0] - out[-1][0]) + abs(out[0][1] - out[-1][1]) < 1e-7:
        out.pop()
    return out


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle: d in from
    its outer edge, z out from the wall; the corners mitred."""
    prof = _clean(profile)
    v, f = [], []
    n = len(prof)
    for (d, z) in prof:
        x, y = w / 2 - d, h / 2 - d
        v += [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    for i in range(n):
        j = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, j * 4 + k2, j * 4 + k))
    return propkit._fix_winding(Shape(v, f))


def seed_cat_frame():
    prof = [(0.0, 0.0), (0.0, 0.0145), (0.0012, 0.0168), (0.0040, 0.0182), (0.0080, 0.0187), (0.0120, 0.0180),
            (0.0150, 0.0160), (0.0178, 0.0122), (SC_M, 0.0112), (SC_M, SC_ZL), (SC_REB, SC_ZL), (SC_REB, 0.0)]
    s = rect_sweep(list(reversed(prof)), SC_W, SC_H)
    # knocks along its outer edges
    off = Vector((0.3, 1.2, 2.1))
    return s.warp(lambda p: p + Vector((0, 0, 0.00025 * noise.noise(p * 90 + off))))


def seed_cat_glass():
    w, h = SC_W - 2 * SC_REB - 0.001, SC_H - 2 * SC_REB - 0.001
    return rbox(w, h, 0.002, r=0.0006, seg=1).move(0, 0, SC_ZL - 0.001)


def seed_cat_mat():
    """Mat board 1.5 mm, cream, its window cut with a 45 degree bevel."""
    w, h = SC_W - 2 * SC_REB - 0.001, SC_H - 2 * SC_REB - 0.001
    z0, z1 = SC_ZL - 0.0035, SC_ZL - 0.002
    ww, wh = SC_WINDOW
    outer = box(w, h, z1 - z0).move(0, 0, (z0 + z1) / 2)
    # the window, wider at the face than at the back (the bevel)
    n = 4
    lo = [(-ww / 2 - 0.0015, -wh / 2 - 0.0015), (ww / 2 + 0.0015, -wh / 2 - 0.0015), (ww / 2 + 0.0015, wh / 2 + 0.0015), (-ww / 2 - 0.0015, wh / 2 + 0.0015)]
    hi_ = [(-ww / 2, -wh / 2), (ww / 2, -wh / 2), (ww / 2, wh / 2), (-ww / 2, wh / 2)]
    v = [(x, y, z0 - 0.001) for x, y in hi_] + [(x * 1.0, y * 1.0, z1 + 0.001) for x, y in lo]
    # (a frustum: narrow at the back, wide at the face)
    f = [tuple(reversed(range(n))), tuple(range(n, 2 * n))]
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
    cutter = propkit._fix_winding(Shape([Vector(p) for p in v], f))
    return cut(outer, cutter)


def seed_cat_cover():
    """The catalogue's cover, pressed flat under the mat but cockled a
    little where it is free in the window."""
    w, h = SC_COVER
    z = SC_ZL - 0.0037
    s = slab(w, h, 0.0004, nx=10, ny=14)
    off = Vector((5.1, 0.7, 0.0))
    return s.warp(lambda p: Vector((p.x, p.y, p.z + z + 0.0003 * max(0.0, noise.noise(Vector((p.x * 18, p.y * 18, 0)) + off) + 0.2))))


def seed_cat_back():
    w, h = SC_W - 2 * SC_REB - 0.001, SC_H - 2 * SC_REB - 0.001
    return rbox(w, h, 0.003, r=0.0005, seg=1).move(0, 0, SC_ZL - 0.0035 - 0.0004 - 0.0015)


auto('Real__seedCatFrame', seed_cat_frame, 300)
auto('Real__seedCatGlass', seed_cat_glass, 60)
auto('Real__seedCatMat', seed_cat_mat, 120)
auto('Real__seedCatCover', seed_cat_cover, 680)
auto('Real__seedCatBack', seed_cat_back, 60)


# ============================================================= apron ===
# A gardener's waist apron of cotton duck (the New York Botanical Garden
# shop's: 26 x 10.5 in, 10 oz canvas, a centre pocket and two side
# pockets across its front, twill tape ties): 560 x 290 mm, its top
# folded into a waistband, a pocket panel 170 mm deep stitched into three
# across its lower part, 25 mm cotton twill tape ties. Dropped on the
# floor: it fell in a heap, folded over itself, the ties lying loose.
# Draped by Blender's cloth solver (headless): the apron let fall from
# hanging by its waistband, forward onto the floor. Its frame: the floor
# y = 0, the heap's middle over the origin.
AW_W, AW_L = 0.28, 0.29          # half the width; the length (FV 0 .. -AW_L)
AW_STEP = 0.016
AW_POCKET = -0.115               # the pocket panel's top (FV)
AW_BAND = -0.030                 # the waistband's stitching (FV)


def apron_flat_grid():
    nx = int(round(2 * AW_W / AW_STEP))
    nv = int(round(AW_L / AW_STEP))
    pts, faces = [], []
    for j in range(nv + 1):
        for i in range(nx + 1):
            pts.append((-AW_W + 2 * AW_W * i / nx, -AW_L * j / nv))
    for j in range(nv):
        for i in range(nx):
            a = j * (nx + 1) + i
            faces.append((a, a + 1, a + nx + 2, a + nx + 1))
    return pts, faces, nx, nv


def apron_drop():
    import bpy
    pts, faces, nx, nv = apron_flat_grid()
    init = []
    FOLD = -0.07                 # its left end folded back over along x = FOLD
    for k, (x, v) in enumerate(pts):
        d = -v
        # lying, its front up, its length along -z, bunched a little across
        # (soft waves the solver lays down as folds)
        X = x * 0.74 + 0.02 * math.sin(d * 13.0 + 0.5)
        Z = -(d - AW_L / 2) * 0.8 + 0.035 * math.sin(x * 6.0)
        Y = 0.05 + 0.045 * max(0.0, math.sin(X * 15.0 + d * 5.0)) + 0.03 * max(0.0, math.sin(d * 19.0 + x * 7.0)) + 0.006 * noise.noise(Vector((x * 9, v * 9, 0.21)))
        if X < FOLD:
            # over the fold: turned back over the rest, a roll at the fold,
            # askew
            e = FOLD - X
            ang = math.pi * min(1.0, e / 0.012)
            X = FOLD - 0.006 * math.sin(ang) + max(0.0, e - 0.012) * 0.97
            Y = Y + 0.012 * (1 - math.cos(ang)) + 0.03
            Z = Z + max(0.0, e - 0.012) * 0.25
        init.append((X, Y, Z))
    # its rest shape is the apron laid flat; it starts from `init`
    flat = [(x, 0.3, (-v - AW_L / 2) * -1.0) for x, v in pts]
    me = bpy.data.meshes.new('apron_drop')
    me.from_pydata(flat, [], faces)
    me.update()
    ob = bpy.data.objects.new('apron_drop', me)
    bpy.context.scene.collection.objects.link(ob)
    basis = ob.shape_key_add(name='Basis')
    start = ob.shape_key_add(name='start')
    for i, q in enumerate(init):
        start.data[i].co = q
    start.value = 1.0
    fl = bpy.data.meshes.new('apron_floor')
    fl.from_pydata([(-2, 0, -2), (2, 0, -2), (2, 0, 2), (-2, 0, 2)], [], [(3, 2, 1, 0)])    # facing +y: it collides on its face's side
    fl.update()
    floor = bpy.data.objects.new('apron_floor', fl)
    bpy.context.scene.collection.objects.link(floor)
    floor.modifiers.new('col', 'COLLISION')
    floor.collision.thickness_outer = 0.002
    floor.collision.cloth_friction = 12.0
    sc = bpy.context.scene
    grav = tuple(sc.gravity)
    sc.gravity = (0.0, -9.81, 0.0)
    cl = ob.modifiers.new('cloth', 'CLOTH')
    s = cl.settings
    s.quality = 8
    s.mass = 0.08                  # duck: heavier than twill
    s.tension_stiffness = 25.0
    s.compression_stiffness = 25.0
    s.shear_stiffness = 8.0
    s.bending_stiffness = 0.8      # 10 oz duck: its folds broad and round
    s.air_damping = 2.0
    s.rest_shape_key = basis
    c = cl.collision_settings
    c.use_collision = True
    c.distance_min = 0.003
    c.use_self_collision = True
    c.self_distance_min = 0.0025
    c.self_friction = 8.0
    c.collision_quality = 4
    frames = 120
    sc.frame_start, sc.frame_end = 1, frames
    cl.point_cache.frame_start, cl.point_cache.frame_end = 1, frames
    for fr in range(1, frames + 1):
        sc.frame_set(fr)
    dg = bpy.context.evaluated_depsgraph_get()
    em = ob.evaluated_get(dg).to_mesh()
    out = [Vector(vv.co) for vv in em.vertices]
    ob.evaluated_get(dg).to_mesh_clear()
    sc.frame_set(1)
    sc.gravity = grav
    bpy.data.objects.remove(ob)
    bpy.data.meshes.remove(me)
    bpy.data.objects.remove(floor)
    bpy.data.meshes.remove(fl)
    return out, faces, pts, nx, nv


_APRON = None


def apron_sim():
    global _APRON
    if _APRON is None:
        _APRON = apron_drop()
    return _APRON


def _front_normals(P, faces):
    """Vertex normals of the cloth's front (its winding's side: +z in the
    flat), area weighted."""
    N = [Vector((0, 0, 0)) for _ in P]
    for f in faces:
        a, b, c = P[f[0]], P[f[1]], P[f[2]]
        n = (b - a).cross(c - a)
        if len(f) == 4:
            n += (c - a).cross(P[f[3]] - a)
        for i in f:
            N[i] += n
    return [n.normalized() if n.length > 1e-12 else Vector((0, 1, 0)) for n in N]


def _apron_centre():
    P, faces, flat, nx, nv = apron_sim()
    lo = Vector((min(p.x for p in P), 0, min(p.z for p in P)))
    hi = Vector((max(p.x for p in P), 0, max(p.z for p in P)))
    return Vector(((lo.x + hi.x) / 2, 0.0, (lo.z + hi.z) / 2))


def _relaxed():
    P, faces, flat, nx, nv = apron_sim()
    P = [p.copy() for p in P]
    # down onto the floor (the solver keeps it its collision distance off)
    y0 = min(p.y for p in P) - 0.0010
    for p in P:
        p.y = max(p.y - y0, 0.0010)
    c = _apron_centre()
    return [p - c for p in P], faces, flat, nx, nv


def _edge_ring(nx, nv):
    top = list(range(nx + 1))
    right = [j * (nx + 1) + nx for j in range(1, nv + 1)]
    bottom = [nv * (nx + 1) + i for i in range(nx - 1, -1, -1)]
    left = [j * (nx + 1) for j in range(nv - 1, 0, -1)]
    return top + right + bottom + left


def apron_cloth():
    P, faces, flat, nx, nv = _relaxed()
    s = Shape([p.copy() for p in P], faces)
    N = _front_normals(P, faces)
    # the hem rolled round its edge, thicker along the waistband's fold
    ring = _edge_ring(nx, nv)
    pts = [P[i] for i in ring]
    v, f = [], []
    n = len(pts)
    for i in range(n):
        t = (pts[(i + 1) % n] - pts[i - 1]).normalized()
        nn = N[ring[i]]
        b = t.cross(nn).normalized()
        r = 0.0022 if flat[ring[i]][1] > -1e-6 else 0.0016
        for k in range(4):
            a = TAU * k / 4
            v.append(pts[i] + (nn * math.cos(a) + b * math.sin(a)) * r)
    for i in range(n):
        i2 = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, i2 * 4 + k2, i2 * 4 + k))
    s = s + Shape(v, f)
    return s


def apron_pocket():
    """The pocket panel over the apron's lower part (its front), its top
    hemmed, the two seams that make three pockets, a little proud of the
    apron and gaping at its mouths."""
    P, faces, flat, nx, nv = _relaxed()
    N = _front_normals(P, faces)
    rows = [j for j in range(nv + 1) if flat[j * (nx + 1)][1] <= AW_POCKET + 1e-6]
    pv, pf = [], []
    nr = len(rows)
    for jj, j in enumerate(rows):
        for i in range(nx + 1):
            k = j * (nx + 1) + i
            u = flat[k][0] / AW_W
            gape = 0.0035 * (1.0 if jj == 0 else 0.4 if jj == 1 else 0.0) * abs(math.sin(u * math.pi * 1.5))
            pv.append(P[k] + N[k] * (0.0022 + gape))
    for jj in range(nr - 1):
        for i in range(nx):
            a = jj * (nx + 1) + i
            pf.append((a, a + 1, a + nx + 2, a + nx + 1))
    pocket = Shape(pv, pf)
    # its top's hem
    top = [pv[i] for i in range(nx + 1)]
    pocket = pocket + sweep(top, [0.0017] * len(top), seg=4, cap0=True, cap1=True)
    # the seams: two lines of stitching down it, at a third each side
    for xs in (-AW_W / 3, AW_W / 3):
        i = int(round((xs + AW_W) / (2 * AW_W) * nx))
        line = [pv[jj * (nx + 1) + i] + N[rows[jj] * (nx + 1) + i] * 0.0008 for jj in range(nr)]
        pocket = pocket + sweep(line, [0.0007] * len(line), seg=3, cap0=True, cap1=True)
    # the waistband's stitching across the top
    j = int(round(-AW_BAND / AW_L * nv))
    band = [P[j * (nx + 1) + i] + N[j * (nx + 1) + i] * 0.0008 for i in range(nx + 1)]
    pocket = pocket + sweep(band, [0.0007] * len(band), seg=3, cap0=True, cap1=True)
    return pocket


def apron_ties():
    """Twill tape, 25 mm: from each end of the waistband, lying flat and
    loose on the floor out from the heap, in curves, over the cloth where it
    crosses it."""
    P, faces, flat, nx, nv = _relaxed()
    out = Shape()
    for corner, sg in ((0, -1), (nx, 1)):
        c0 = P[corner]
        # away from the heap's middle, then curling round
        ang = math.atan2(c0.z, c0.x)
        tp = [c0.copy()]
        L = 0.36
        n = 30
        x, z = c0.x, c0.z
        a = ang
        a += sg * 0.5
        for i in range(1, n + 1):
            t = i / n
            # it lies as it fell: bending one way and the other
            a += sg * (0.05 + 0.16 * n1(t * 3.0 + sg * 5.0, 0.3, 1.7)) + 0.3 * n1(t * 9.0, sg * 3.0, 0.9)
            x += math.cos(a) * L / n
            z += math.sin(a) * L / n
            near = [q.y for q in P if abs(q.x - x) < 0.018 and abs(q.z - z) < 0.018]
            # crinkles where it buckled
            y = max([0.0] + near) + 0.0016 + 0.0025 * max(0.0, n1(t * 9.0, sg, 2.2))
            tp.append(Vector((x, y, z)))
        # down off the cloth's edge gently
        for k in range(1, len(tp) - 1):
            tp[k].y = max(tp[k].y, min(tp[k - 1].y, tp[k + 1].y) * 0.5 + 0.0016)
        tp = chaikin(tp, 1)
        sides = []
        m = len(tp)
        for k in range(m):
            d = tp[min(k + 1, m - 1)] - tp[max(k - 1, 0)]
            sd = Vector((-d.z, 0, d.x)).normalized()
            # one turn over on itself (a half twist), where it folded
            t = k / (m - 1)
            tw = math.pi * smoothstep(0.55, 0.62, t) if sg > 0 else math.pi * smoothstep(0.25, 0.3, t)
            dn = d.normalized()
            sides.append(sd * math.cos(tw) + dn.cross(sd) * math.sin(tw) * 0.25)
        out = out + _band(tp, sides, 0.022, 0.0011)
    return out


def _band(pts, sides, width, thick):
    """A tape along `pts`, `width` across toward `sides[i]`, `thick` through."""
    v, f = [], []
    n = len(pts)
    for i in range(n):
        p = Vector(pts[i])
        t = (Vector(pts[min(i + 1, n - 1)]) - Vector(pts[max(i - 1, 0)])).normalized()
        s = Vector(sides[i])
        s = (s - t * s.dot(t)).normalized()
        nn = t.cross(s).normalized()
        if nn.y < 0:
            nn = -nn
        for a, b in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            v.append(p + s * (a * width / 2) + nn * (b * thick / 2))
    for i in range(n - 1):
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, (i + 1) * 4 + k2, (i + 1) * 4 + k))
    f.append((3, 2, 1, 0))
    f.append(((n - 1) * 4, (n - 1) * 4 + 1, (n - 1) * 4 + 2, (n - 1) * 4 + 3))
    return propkit._fix_winding(Shape(v, f))


auto('Real__apronGardenCloth', lambda: floor_clamp(apron_cloth()), 2600, smooth=60.0)
auto('Real__apronGardenPocket', lambda: floor_clamp(apron_pocket()), 1700, smooth=60.0)
auto('Real__apronGardenTies', lambda: floor_clamp(apron_ties()), 1000, smooth=60.0)
