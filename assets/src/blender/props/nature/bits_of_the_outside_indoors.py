"""t05: bits of the outside indoors, remodelled at their real sizes
(docs/design/PROPS.md P3): oak twiglets shed by the tree, a robin's nest fallen
with its eggs, dead beech and oak leaves blown in, moss cushions on a damp
floor, a tuft of annual meadow grass, a butterfly's wing left where a mouse
ate the rest, an empty garden snail's shell, a cellar spider's web in the
angle of a wall and the ceiling with the spider dead in it. Sources and
sizes: docs/research/props-remodelling/bits_of_the_outside_indoors.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
own frame: x across, y up from the floor it rests on (y = 0), z out of its
front; a thing that lies on the floor lies in its frame as it lies, its
lowest point at y = 0 and its middle over the origin.

twigA..F          oak twiglets shed whole (the rounded scar at the base) or
                  snapped, 45-98 mm: bark, nodes, lateral buds and the
                  terminal cluster, a side shoot; lying as they lie
twigWoodA, C, F   the pale wood where those twiglets snapped
nestCup           a robin's nest (150 mm across, 75 mm tall): grass and
                  twigs over a cup of mud, its outside and rim
nestLining        the fine dry grass lining its cup
nestStrands       loose grass stems wound round it, some trailing
nestTwigs         twigs worked into its base and sticking out
nestStray         grass ends standing out of it every way (its shag)
nestEgg           an egg (28 x 21 mm), lying on its side
nestEggBroken     an egg's broken half shell
leafBeechA..C     dead beech leaves (blades 56-72 mm), cupped, wavy, folded
leafOakA, B       dead oak leaves (70-86 mm), rolled under, twisted (each
                  over its outline's envelope: the game cuts the outline)
mossCushionA..C   moss cushions (22-40 mm across): domes of shoots
mossSetae         stalks (setae) with their nodding capsules, a cluster
grassTillerA..E   tillers of annual meadow grass, 50-120 mm: a sheath and
                  two to four blades (2.5 mm, folded, boat-tipped), arching
grassPanicle      a flowering stem with its open panicle
grassDead         dead blades lying round the tuft's foot
wingFore          a forewing of Aglais (peacock, small tortoiseshell): a
                  gently creased sheet 30.5 x 19 mm, its outline's box (the
                  game cuts the outline out and paints its scales)
wingHind          a hindwing, 24.5 x 21.5 mm, the same way
snailShell        an empty garden snail's shell (Cornu aspersum, about
                  30 mm across and 33 mm tall, 4.3 whorls), apex up
snailLip          its thickened, turned-back lip
snailInside       the inside of its last whorl and a half, seen through the
                  aperture (its bands are painted by the game, in the
                  sphere frame of snailShell)
spiderWebThreads  a cellar spider's tangle in a wall-ceiling angle: guy
                  lines, the tangle, dusty strings (the wall is z = 0, the
                  ceiling y = 0, the web below and in front of both)
spiderBody        the dead cellar spider hanging in it, legs drawn in
spiderWrap        two prey wrapped in silk, each hung on a short thread
"""
import math
import random

import bmesh
from propkit import *
import propkit


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. `smooth`: the angle (degrees) under which an
    edge is shaded smooth (a twig's few sides are round, not faceted)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, smooth=smooth, flat=flat, budget=budget)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def on_floor(s, centre=True):
    """Lowest point to y = 0, its middle (x, z) over the origin."""
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2 if centre else 0, -lo.y, -(lo.z + hi.z) / 2 if centre else 0)


def sweep(pts, radii, seg=6, cap0=True, cap1=True, rough=None):
    """A tube along `pts` whose radius at each point is `radii[i]`
    (parallel-transport frame). `rough(i, j, p, n) -> dr` pushes a vertex
    out along its ring's normal. cap0/cap1: 'dome' rounds the end over
    `radii`, True closes it flat, False leaves it open."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    tang = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tang.append(t.normalized() if t.length > 1e-12 else Vector((1, 0, 0)))
    up = Vector((0, 1, 0)) if abs(tang[0].y) < 0.9 else Vector((1, 0, 0))
    nrm = tang[0].cross(up).normalized()
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


def ovoid(length, width, seg=6, rings=4, point=0.0):
    """A small ovoid along +x from the origin (a bud), `point` sharpening
    its far end."""
    prof = [(0.0, 0.0)]
    for i in range(1, rings + 1):
        t = i / (rings + 1)
        r = width / 2 * math.sin(math.pi * t) ** (0.8 + point * t)
        prof.append((r, length * t))
    prof.append((0.0, length))
    return lathe(prof, seg).rot(0, 0, -math.pi / 2)


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


# ================================================================ twigs ===
# Oak twiglets (Quercus robur): an oak sheds its small twigs whole, at an
# abscission layer at their base (cladoptosis), leaving a smooth rounded
# end; others are snapped. One-year wood 2.5-5 mm across, grey-brown, its
# lenticels pale; alternate buds at the nodes, each over its leaf scar, a
# cluster of buds at the tip; a short side shoot.
TWIGS = [
    # (seed, length, base radius, tip radius, side shoots, snapped)
    (501, 0.046, 0.0022, 0.0015, 0, True),
    (502, 0.056, 0.0026, 0.0016, 1, False),
    (503, 0.066, 0.0024, 0.0015, 1, True),
    (504, 0.076, 0.0029, 0.0017, 1, False),
    (505, 0.086, 0.0030, 0.0018, 2, False),
    (506, 0.098, 0.0033, 0.0019, 1, True),
]


def _bark_rough(seed, r0):
    off = Vector((seed * 3.1, seed * 1.7, seed * 0.9))

    def f(i, j, p, d):
        q = p * 900.0 + d * 2.0 + off
        # fine ridges along it, and lenticels (small raised spots)
        return r0 * (0.06 * noise.noise(q) + 0.05 * max(0.0, noise.noise(p * 2500.0 + d * 3.0 + off) - 0.35) * 3.0)
    return f


def _axis(rng, length, kinks):
    """A twig's axis along +x from the origin: a slight zigzag at each node,
    bowed a little (in x-z: it lies flat)."""
    pts = [Vector((0, 0, 0))]
    step = length / kinks
    ang = rng.uniform(-0.1, 0.1)
    for k in range(kinks):
        ang += rng.uniform(-0.22, 0.22) * (1 if k % 2 else -1)
        last = pts[-1]
        pts.append(last + Vector((math.cos(ang), rng.uniform(-0.02, 0.02), math.sin(ang))) * step)
    return pts


def _bud_cluster(tip, direction, rng, r):
    """The terminal buds: three to five ovoid buds at the tip, splayed."""
    out = Shape()
    d0 = direction.normalized()
    side = d0.cross(Vector((0, 1, 0))).normalized()
    n = rng.randrange(3, 6)
    for k in range(n):
        L = rng.uniform(0.0032, 0.0048) * (1.15 if k == 0 else 1.0)
        W = L * rng.uniform(0.48, 0.58)
        b = ovoid(L, W, seg=5, rings=2, point=0.6)
        if k == 0:
            dirn = d0
        else:
            a = TAU * k / (n - 1) + rng.uniform(-0.3, 0.3)
            dirn = (d0 + (side * math.cos(a) + Vector((0, 1, 0)) * math.sin(a)) * rng.uniform(0.5, 0.8)).normalized()
        q = Vector((1, 0, 0)).rotation_difference(dirn)
        b.apply(q.to_matrix().to_4x4())
        b.move(*(tip - d0 * r * 0.6))
        out = out + b
    return out


def twig(seed, length, r0, r1, shoots, snapped):
    rng = random.Random(seed)
    kinks = 4 + int(length / 0.025)
    axis = chaikin(_axis(rng, length, kinks), 1)
    n = len(axis)
    radii = []
    for i in range(n):
        t = i / (n - 1)
        r = r0 + (r1 - r0) * t
        # the base swells to its rounded scar; nodes swell a little
        r *= 1.0 + 0.18 * math.exp(-((t * length) / 0.003) ** 2)
        radii.append(r)
    body = sweep(axis, radii, seg=6, cap0='dome', cap1=(True if snapped else 'dome'), rough=_bark_rough(seed, r0))
    parts = [body]
    wood = Shape()
    if snapped:
        # the snapped end: splinters standing out of the break, its wood pale
        tip = axis[-1]
        d = (axis[-1] - axis[-2]).normalized()
        for k in range(5):
            a = TAU * k / 5 + rng.uniform(-0.4, 0.4)
            side = d.cross(Vector((0, 1, 0))).normalized()
            off = (side * math.cos(a) + Vector((0, 1, 0)) * math.sin(a)) * r1 * 0.45
            L = rng.uniform(0.0008, 0.0026)
            sp = cone(r1 * 0.38, L, seg=4, bevel_r=0).rot(0, 0, -math.pi / 2).move(L / 2, 0, 0)
            q = Vector((1, 0, 0)).rotation_difference((d + off * 40).normalized())
            sp.apply(q.to_matrix().to_4x4())
            wood = wood + sp.move(*(tip + off - d * 0.0002))
        wood = wood + cyl(r1 * 0.92, h=0.0004, seg=6, bevel_r=0).rot(0, 0, math.pi / 2).apply(
            Vector((1, 0, 0)).rotation_difference(d).to_matrix().to_4x4()).move(*(tip + d * 0.0001))
    else:
        parts.append(_bud_cluster(axis[-1], axis[-1] - axis[-2], rng, r1))
    # lateral buds at the nodes (alternate, each with a raised leaf scar)
    nodes = [int(n * f) for f in (0.3, 0.48, 0.64, 0.8)]
    for k, i in enumerate(nodes):
        if i >= n - 2:
            continue
        d = (axis[i + 1] - axis[i - 1]).normalized()
        side = d.cross(Vector((0, 1, 0))).normalized() * (1 if k % 2 else -1)
        up = rng.uniform(-0.4, 0.6)
        out = (side * math.cos(up) + Vector((0, 1, 0)) * math.sin(up)).normalized()
        L = rng.uniform(0.0024, 0.0034)
        b = ovoid(L, L * 0.55, seg=5, rings=2, point=0.6)
        q = Vector((1, 0, 0)).rotation_difference((d * 0.8 + out * 0.6).normalized())
        b.apply(q.to_matrix().to_4x4())
        parts.append(b.move(*(axis[i] + out * radii[i] * 0.7)))
        scar = ellipsoid(radii[i] * 0.55, radii[i] * 0.35, radii[i] * 0.55, seg=6)
        parts.append(scar.move(*(axis[i] + out * radii[i] * 0.85 - d * L * 0.25)))
    # side shoots from a node, in the plane it lies in
    for k in range(shoots):
        i = int(n * (0.38 + 0.25 * k))
        d = (axis[i + 1] - axis[i - 1]).normalized()
        sgn = -1 if (k + seed) % 2 else 1
        side = d.cross(Vector((0, 1, 0))).normalized() * sgn
        L = length * rng.uniform(0.28, 0.42)
        start = axis[i]
        dirn = (d * 0.65 + side * 0.76).normalized()
        sp = [start, start + dirn * L * 0.35, start + (dirn + d * 0.25).normalized() * L * 0.7, start + (dirn + d * 0.4).normalized() * L]
        sp = chaikin(sp, 1)
        sr = [radii[i] * 0.72 - (radii[i] * 0.72 - r1 * 0.75) * t / (len(sp) - 1) for t in range(len(sp))]
        parts.append(sweep(sp, sr, seg=5, cap0=False, cap1='dome', rough=_bark_rough(seed + 7 * k, r0)))
        parts.append(_bud_cluster(sp[-1], sp[-1] - sp[-2], rng, sr[-1]))
    s = sum(parts[1:], parts[0])
    # lying as it lies: its middle over the origin, its lowest point on the floor
    lo, hi = (s + wood).bounds() if wood.v else s.bounds()
    dx, dy, dz = -(lo.x + hi.x) / 2, -lo.y, -(lo.z + hi.z) / 2
    s.move(dx, dy, dz)
    if wood.v:
        wood.move(dx, dy, dz)
    return s, wood


_TWIG = {}


def twig_part(i, which):
    if i not in _TWIG:
        _TWIG[i] = twig(*TWIGS[i])
    return _TWIG[i][which]


for _i in range(len(TWIGS)):
    auto(f'Real__twig{"ABCDEF"[_i]}', lambda _i=_i: twig_part(_i, 0), 1000, smooth=80.0)
    if TWIGS[_i][5]:
        auto(f'Real__twigWood{"ABCDEF"[_i]}', lambda _i=_i: twig_part(_i, 1), 100)


# =========================================================== snail shell ===
# The garden snail (Cornu aspersum): a globose shell of 4.5 whorls, 28-38 mm
# across and 25-35 mm tall, the last whorl large and descending in front of
# the aperture, the lip thickened, turned back and white, the umbilicus shut.
# A helicospiral: the whorl's section (an ellipse a little taller than wide)
# swept round the axis (+y, the apex up), growing by SH_W each turn; each
# whorl wraps over the one before, and where they meet is the suture. The
# game paints its bands on it in the sphere's frame (the same formula,
# `t05.rs`), so the section and the sweep are written plainly here.
SH_W = 2.3            # growth per whorl
SH_N = 4.3            # whorls
SH_T = SH_N * TAU
SH_K = math.log(SH_W) / TAU
SH_RC, SH_RHO, SH_H, SH_B = 0.0085, 0.0100, 0.0175, 1.12   # at the aperture: centre radius, section radius, drop, section's height / width
SH_OBL = 0.25         # the aperture is oblique: its foot reaches this much further round (radians) than its top


def sh_obl(phi):
    """How much further round the last whorl goes at section angle phi, as
    first guessed (its foot further than its top)."""
    return SH_OBL * (0.5 - 0.5 * math.sin(phi))


_AP = []


def sh_end(phi):
    """Where the last whorl ends at section angle phi: on the aperture's
    plane (a real peristome is flat, so a look into the aperture always
    meets the inside), through its top, its outer edge and its foot as
    `sh_obl` places them; the crossing nearest that guess. Across the whorl
    before (the parietal side, |phi| past 2.4), where the section may not
    cross the plane, it runs evenly between its two sides' ends."""
    w = math.atan2(math.sin(phi), math.cos(phi))
    if abs(w) > 2.4:
        a, b = _sh_end_root(2.4), _sh_end_root(-2.4)
        u = ((w if w > 0 else w + TAU) - 2.4) / (TAU - 4.8)
        return a + (b - a) * u
    return _sh_end_root(phi)


def _sh_end_root(phi):
    if not _AP:
        top, out, foot = sh_point(SH_T, math.pi / 2), sh_point(SH_T + sh_obl(0.0), 0.0), sh_point(SH_T + SH_OBL, -math.pi / 2)
        _AP.append((top, (out - top).cross(foot - top).normalized()))
    p0, n = _AP[0]
    guess = SH_T + sh_obl(phi)
    f = lambda t: (sh_point(t, phi) - p0).dot(n)
    best = None
    t = guess - 1.2
    while t < guess + 1.2:
        a, b = f(t), f(t + 0.01)
        if a == 0 or a * b < 0:
            lo, hi = t, t + 0.01
            for _ in range(30):
                mid = (lo + hi) / 2
                if f(lo) * f(mid) <= 0:
                    hi = mid
                else:
                    lo = mid
            r = (lo + hi) / 2
            if best is None or abs(r - guess) < abs(best - guess):
                best = r
        t += 0.01
    return guess if best is None else best


def sh_s(t):
    return math.exp(SH_K * (t - SH_T))


def sh_point(t, phi, dr=0.0):
    """The shell's surface at turn t (0 the apex, SH_T the aperture) and
    angle phi round the whorl's section (0 out from the axis, pi/2 up)."""
    s = sh_s(t)
    drop = SH_H * s * (1 + 0.10 * smoothstep(SH_T - 1.4, SH_T, t))
    flare = 1 + 0.07 * smoothstep(SH_T - 0.3, SH_T, t)
    rho = SH_RHO * s * flare + dr
    R = Vector((math.cos(t), 0, -math.sin(t)))
    return Vector((SH_RC * s * math.cos(t), -drop, -SH_RC * s * math.sin(t))) + R * (rho * math.cos(phi)) + Vector((0, 1, 0)) * (rho * SH_B * math.sin(phi))


def _sh_ts(t0=0.9):
    """Turns along the sweep: closer together as the whorl grows."""
    ts, t = [], t0
    while t < SH_T:
        ts.append(t)
        t += TAU / (9 + 22 * sh_s(t) ** 0.6)
    ts.append(SH_T)
    return ts


def _sh_rough(t, phi):
    s = sh_s(t)
    # growth lines (fine ribs across the whorl) and the wrinkled surface
    ribs = 0.000045 * math.sin(t * 95) + 0.00003 * math.sin(t * 37 + 1.3)
    wr = 0.00006 * noise.noise(Vector((t * 9.0, phi * 3.0, 0.5)))
    return (ribs + wr) * min(1.0, s * 1.6)


def sh_centre(t):
    s = sh_s(t)
    return Vector((SH_RC * s * math.cos(t), -SH_H * s * (1 + 0.10 * smoothstep(SH_T - 1.4, SH_T, t)), -SH_RC * s * math.sin(t)))


def snail_shell(inside=False, nphi=14):
    """The shell's outer face (closed at the apex, open at the aperture), or
    `inside`, the inner face of its last whorl and a half (seen through the
    aperture), facing in."""
    ts = _sh_ts()
    if inside:
        ts = [t for t in ts if t > SH_T - 4.0]
    v, f = [], []
    for t in ts:
        ramp = smoothstep(SH_T - 1.2, SH_T, t)
        for j in range(nphi):
            phi = TAU * j / nphi
            tt = t + (sh_end(phi) - SH_T) * ramp
            dr = -0.00045 * min(1.0, sh_s(tt) * 2) if inside else _sh_rough(tt, phi)
            v.append(sh_point(tt, phi, dr))
    for i in range(len(ts) - 1):
        for j in range(nphi):
            j2 = (j + 1) % nphi
            f.append((i * nphi + j, i * nphi + j2, (i + 1) * nphi + j2, (i + 1) * nphi + j))
    if not inside:
        # the apex: a point a little above the first ring's middle
        v.append(sh_centre(ts[0]) + Vector((0, SH_RHO * sh_s(ts[0]) * 0.8, 0)))
        ap = len(v) - 1
        for j in range(nphi):
            f.append((ap, (j + 1) % nphi, j))
    out = Shape(v, f)
    # one grid, one way round: its faces out of the shell (in, for the
    # inside), checked on a face of the last whorl's outermost row (its
    # section's angle 0 faces straight out from the axis)
    i0 = len(ts) - 4
    a, b, c = out.v[i0 * nphi], out.v[i0 * nphi + 1], out.v[(i0 + 1) * nphi + 1]
    n = (b - a).cross(c - b)
    t0 = ts[i0]
    outward = Vector((math.cos(t0), 0, -math.sin(t0)))
    if (n.dot(outward) > 0) == inside:
        out.f = [tuple(reversed(x)) for x in out.f]
    return out


def snail_lip():
    """The lip: rolled outward and thickened round the aperture from the
    top of its inner side over its outer edge to its foot, thinning out at
    both ends where it meets the whorl before (across which the shell is
    only glazed)."""
    n = 26
    pts, radii = [], []
    for j in range(n + 1):
        phi = -2.4 + 4.8 * j / n
        t = sh_end(phi)
        p = sh_point(t, phi, 0.0002)
        c = sh_point(t, phi, -0.0006)
        pts.append(p.lerp(c, 0.4))
        k = j / n
        radii.append(0.00085 * math.sin(math.pi * k) ** 0.6 + 0.00015)
    return sweep(pts, radii, seg=6, cap0=True, cap1=True)


auto('Real__snailShell', lambda: snail_shell(False), 2200, smooth=60.0)
auto('Real__snailInside', lambda: snail_shell(True, 14), 900, smooth=60.0)
auto('Real__snailLip', snail_lip, 400, smooth=70.0)


# ================================================================= nest ===
# An American robin's nest (or a blackbird's, the same build): an open cup
# of dry grass and twigs over a layer of mud, lined with fine dry grass.
# Measured: 113-151 mm across outside, a cup 87-102 mm across inside and
# 50-69 mm deep; this one has fallen and sits on its flattened base, 150 mm
# across and 75 mm tall, its cup 100 mm across and 50 mm deep.
NEST_OUT = [(0.000, 0.000), (0.036, 0.000), (0.055, 0.008), (0.068, 0.024), (0.0745, 0.044), (0.0735, 0.060),
            (0.068, 0.071), (0.060, 0.0755), (0.052, 0.0735)]
NEST_IN = [(0.0495, 0.068), (0.048, 0.055), (0.042, 0.040), (0.030, 0.030), (0.015, 0.0265), (0.000, 0.026)]
NEST_FLOOR = 0.026     # the cup's floor (the eggs lie on it)


def _nest_lump(p, amount, scale, seed):
    q = p * scale + Vector((seed * 1.3, seed * 0.7, seed * 2.1))
    return noise3(q) * amount


def nest_cup():
    """The nest's outside and rim, down into the cup to where the lining
    covers it: lumpy, a strand's thickness rough."""
    prof = NEST_OUT + NEST_IN[:2]
    s = lathe(chaikin_2d(prof, 1), 34, cap_bottom=False, cap_top=False)
    out = s.copy()
    for i, p in enumerate(out.v):
        r = math.hypot(p.x, p.z)
        # lumps (where the bird pressed material in), coarse and fine
        d = _nest_lump(p, 0.0045, 40.0, 1) + _nest_lump(p, 0.0016, 160.0, 2)
        # the rim sags and rises where the bird worked it
        if p.y > 0.06:
            d.y += 0.004 * noise.noise(Vector((math.atan2(p.z, p.x) * 1.7, 3.3, 0.7)))
        # the base is pressed flat by the floor it fell on
        if p.y < 0.003:
            d.y = 0.0
        out.v[i] = p + d
        out.v[i].y = max(0.0, out.v[i].y)
    return out


def nest_lining():
    """The fine dry grass lining the cup, a little in from its wall."""
    s = lathe(chaikin_2d(NEST_IN, 1), 26, cap_bottom=False, cap_top=False)
    out = s.copy()
    for i, p in enumerate(out.v):
        out.v[i] = p + _nest_lump(p, 0.0012, 120.0, 3) + Vector((0, 0.0004, 0))
    # its faces into the cup (up at its bottom), where it is seen from
    def up_at_bottom(sh):
        for f in sh.f:
            c = sum((sh.v[i] for i in f), Vector()) / len(f)
            if math.hypot(c.x, c.z) < 0.01:
                a, b, d = sh.v[f[0]], sh.v[f[1]], sh.v[f[2]]
                return (b - a).cross(d - b).y > 0
        return True
    if not up_at_bottom(out):
        out.f = [tuple(reversed(f)) for f in out.f]
    return out


def chaikin_2d(pts, k=2):
    for _ in range(k):
        q = [pts[0]]
        for (ax, ay), (bx, by) in zip(pts, pts[1:]):
            q += [(ax * 0.75 + bx * 0.25, ay * 0.75 + by * 0.25), (ax * 0.25 + bx * 0.75, ay * 0.25 + by * 0.75)]
        q.append(pts[-1])
        pts = q
    return pts


def _nest_radius(y):
    """The outside's radius at height y (from NEST_OUT)."""
    for (r0, y0), (r1, y1) in zip(NEST_OUT, NEST_OUT[1:]):
        if y0 <= y <= y1 and y1 > y0:
            return r0 + (r1 - r0) * (y - y0) / (y1 - y0)
    return NEST_OUT[-2][0]


def ribbon(pts, width, normal_hint=None, twist=0.0):
    """A flat strip along `pts` (a grass blade: flat, width tapering to its
    tip), its face turned to `normal_hint` (else up)."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    v, f = [], []
    for i in range(n):
        t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
        h = Vector(normal_hint(pts[i]) if normal_hint else (0, 1, 0))
        side = t.cross(h)
        if side.length < 1e-9:
            side = t.cross(Vector((1, 0, 0)))
        side.normalize()
        a = twist * i / max(1, n - 1)
        side = side * math.cos(a) + t.cross(side) * math.sin(a)
        w = width[i] if isinstance(width, (list, tuple)) else width
        v += [pts[i] - side * w / 2, pts[i] + side * w / 2]
    for i in range(n - 1):
        f.append((2 * i, 2 * i + 1, 2 * i + 3, 2 * i + 2))
    return Shape(v, f)


def nest_strands(seed=41, n=64):
    """Dry grass blades and stems wound round the outside (lying on it),
    a few come loose and trailing."""
    rng = random.Random(seed)
    out = Shape()
    cup = nest_cup()
    lo, hi = cup.bounds()
    for k in range(n):
        y = rng.uniform(0.006, 0.072)
        a0 = rng.uniform(0, TAU)
        span = rng.uniform(0.6, 2.2)
        loose = k % 7 == 0
        pts = []
        steps = 8
        lean = rng.uniform(-0.012, 0.012)
        for i in range(steps + 1):
            u = i / steps
            a = a0 + span * u
            yy = y + rng.uniform(-0.0008, 0.0008) + (u - 0.5) * lean
            r = _nest_radius(min(max(yy, 0.001), 0.074)) + 0.0025 + rng.uniform(-0.0008, 0.0012)
            p = Vector((math.cos(a) * r, yy, math.sin(a) * r))
            if loose and u > 0.55:
                # off the nest and drooping to the floor
                q = (u - 0.55) / 0.45
                p = p + Vector((math.cos(a), 0, math.sin(a))) * 0.03 * q
                p.y = max(0.0008, p.y * (1 - q) + 0.0008 * q)
            pts.append(p)
        w0 = rng.uniform(0.0014, 0.0030)
        pts = chaikin(pts, 1)
        widths = [w0 * (1 - 0.7 * smoothstep(0.6, 1.0, i / (len(pts) - 1))) for i in range(len(pts))]
        out = out + ribbon(pts, widths, normal_hint=lambda p: Vector((p.x, 0, p.z)).normalized() if p.y > 0.002 else Vector((0, 1, 0)), twist=rng.uniform(-1, 1))
    return out


def nest_twigs(seed=43, n=16):
    """Twigs worked into the base and the outer wall, their ends standing out."""
    rng = random.Random(seed)
    out = Shape()
    for k in range(n):
        y = rng.uniform(0.004, 0.05) if k % 3 else rng.uniform(0.002, 0.012)
        a = rng.uniform(0, TAU)
        r = _nest_radius(y) - 0.004
        p0 = Vector((math.cos(a) * r, y, math.sin(a) * r))
        # mostly along the wall (tangent), one end standing out
        tang = Vector((-math.sin(a), 0, math.cos(a)))
        outw = Vector((math.cos(a), 0, math.sin(a)))
        L = rng.uniform(0.035, 0.075)
        d = (tang * rng.choice((-1, 1)) + outw * rng.uniform(0.3, 0.8) + Vector((0, rng.uniform(-0.15, 0.25), 0))).normalized()
        pts = [p0 - d * L * 0.2]
        for i in range(1, 5):
            q = p0 + d * L * (i / 4 - 0.2) + Vector((rng.uniform(-1, 1), rng.uniform(-0.5, 0.5), rng.uniform(-1, 1))) * 0.0015
            pts.append(q)
        for p in pts:
            p.y = max(p.y, 0.0012)
            # never in through the wall to the cup: at least 12 mm out from its inside
            rr = math.hypot(p.x, p.z)
            lim = _nest_radius(min(max(p.y, 0.001), 0.074)) - 0.012
            if rr < lim:
                p.x, p.z = p.x / rr * lim, p.z / rr * lim
        r0 = rng.uniform(0.0008, 0.0014)
        out = out + sweep(pts, [r0, r0, r0 * 0.9, r0 * 0.8, r0 * 0.7], seg=4, cap0=True, cap1=True)
    return out


def egg_profile(L, W, n=9):
    """An egg's profile from its blunt end (y = 0) to its point (y = L)."""
    prof = [(0.0, 0.0)]
    for i in range(1, n):
        x = -1 + 2 * i / n
        r = W / 2 * math.sqrt(max(0.0, 1 - x * x)) * (1 - 0.13 * x)
        prof.append((r, L * i / n))
    prof.append((0.0, L))
    return prof


EGG_L, EGG_W = 0.0285, 0.0210   # a robin's egg


def nest_egg():
    """An egg lying on its side (its long axis along x), on y = 0."""
    s = lathe(egg_profile(EGG_L, EGG_W), 14)
    s.rot(0, 0, -math.pi / 2)
    return on_floor(s)


def nest_egg_broken(seed=47):
    """The blunt half of an egg's shell, broken along a ragged line, lying on
    its side open to one side: its shell 0.25 mm thick."""
    rng = random.Random(seed)
    prof = egg_profile(EGG_L, EGG_W, 12)
    nseg = 16
    # how far up the egg (in profile steps) the break runs at each angle
    cut = [6 + rng.choice((0, 1, 1, 2)) - (1 if j % 4 == 0 else 0) for j in range(nseg)]
    v, f = [], []
    rows = max(cut) + 1
    for i in range(rows):
        for j in range(nseg):
            k = min(i, cut[j])
            r, y = prof[k] if k else (prof[1][0] * 0.3, 0.0)
            a = TAU * j / nseg
            v.append(Vector((r * math.sin(a), y, r * math.cos(a))))
    inner = [p * 0.975 + Vector((0, 0.0003, 0)) for p in v]
    n0 = len(v)
    v += inner
    for i in range(rows - 1):
        for j in range(nseg):
            j2 = (j + 1) % nseg
            a, b, c, d = i * nseg + j, i * nseg + j2, (i + 1) * nseg + j2, (i + 1) * nseg + j
            f.append((a, b, c, d))
            f.append((n0 + d, n0 + c, n0 + b, n0 + a))
    # the broken rim joins outside to inside
    top = rows - 1
    for j in range(nseg):
        j2 = (j + 1) % nseg
        a, b = top * nseg + j, top * nseg + j2
        f.append((b, a, n0 + a, n0 + b))
    # close the blunt end's tiny hole
    s = Shape(v, f)
    s = weld(s, 1e-6)
    s.rot(0, 0, -math.pi / 2).rot(rng.uniform(0.3, 0.6), 0, 0)
    return on_floor(s)


def nest_stray(seed=45, n=70):
    """Grass ends standing out of the outside and the rim every way: the
    shag of a real nest's silhouette."""
    rng = random.Random(seed)
    out = Shape()
    for k in range(n):
        y = rng.uniform(0.008, 0.074)
        a = rng.uniform(0, TAU)
        r = _nest_radius(y) - 0.002
        base = Vector((math.cos(a) * r, y, math.sin(a) * r))
        outw = Vector((math.cos(a), 0, math.sin(a)))
        tang = Vector((-math.sin(a), 0, math.cos(a)))
        up = 0.6 if y > 0.065 else rng.uniform(-0.3, 0.4)
        d = (outw * rng.uniform(0.3, 1.0) + tang * rng.uniform(-1, 1) + Vector((0, up, 0))).normalized()
        L = rng.uniform(0.008, 0.03)
        bend = Vector((0, -rng.uniform(0.0, 0.25), 0))
        pts = [base, base + d * L * 0.5, base + (d + bend).normalized() * L]
        for p in pts:
            p.y = max(p.y, 0.0008)
        w0 = rng.uniform(0.0010, 0.0022)
        out = out + ribbon(pts, [w0, w0 * 0.8, w0 * 0.3], normal_hint=lambda p: Vector((0, 1, 0)), twist=rng.uniform(-1.5, 1.5))
    return out


auto('Real__nestCup', nest_cup, 1600, smooth=60.0)
auto('Real__nestStray', nest_stray, 400, smooth=60.0)
auto('Real__nestLining', nest_lining, 900, smooth=60.0)
auto('Real__nestStrands', nest_strands, 2200, smooth=60.0)
auto('Real__nestTwigs', nest_twigs, 700, smooth=80.0)
auto('Real__nestEgg', nest_egg, 300, smooth=60.0)
auto('Real__nestEggBroken', nest_egg_broken, 600, smooth=60.0)


# ================================================================ leaves ===
# Dead leaves blown in: beech (Fagus sylvatica: ovate, 50-100 mm long and
# 38-64 mm wide, 6-9 pairs of straight veins, the margin wavy, a petiole of
# about 8 mm; dead, they stay brown and crisp) and pedunculate oak (Quercus
# robur: obovate, 70-120 mm, 4-6 rounded lobes a side, a 2-5 mm petiole and
# two ear-like lobes at its base). Each blade is modelled over its outline's
# envelope (a smooth oval a millimetre outside it), lying in x-z, the midrib
# along +x from its base (x = 0) to its tip; its curl and crumple raise it in
# y only, so seen from above it is its flat outline: the game paints the
# leaf (its outline cut out of the envelope, lobes, tears and holes, veins)
# in the frame's top view and the two agree (LEAVES in `t05.rs`).
LEAVES = {
    # name: (seed, blade length, envelope half width, petiole, oak?, how it lies)
    'BeechA': (611, 0.072, 0.0225, 0.009, False, 'cup'),
    'BeechB': (612, 0.062, 0.0195, 0.008, False, 'wave'),
    'BeechC': (613, 0.056, 0.0180, 0.007, False, 'fold'),
    'OakA': (621, 0.086, 0.0265, 0.004, True, 'dome'),
    'OakB': (622, 0.070, 0.0220, 0.003, True, 'twist'),
}


def leaf_env(a, oak):
    """The envelope's half width (a fraction of the widest) at a fraction a
    of the blade's length from its base."""
    if oak:
        # obovate: widest at two thirds, the ears at its base
        return max(0.0, math.sin(math.pi * min(1.0, a) ** 0.62)) ** 0.75 * (1 - 0.15 * (1 - a)) + 0.18 * math.exp(-((a - 0.03) / 0.05) ** 2)
    # ovate: widest at two fifths, an acute tip
    return max(0.0, math.sin(math.pi * min(1.0, a) ** 1.15)) ** 0.8


def leaf(seed, L, W, P, oak, how):
    rng = random.Random(seed)
    nu, nv = 14, 6
    v, f = [], []
    off = Vector((seed * 0.37, seed * 0.11, 0))

    def height(x, z):
        a, b = x / L, z / W
        y = 0.0
        if how == 'cup':      # margins curled up, the tip down a little
            y = 0.30 * W * abs(b) ** 1.5 - 0.004 * math.sin(math.pi * a)
        elif how == 'wave':   # nearly flat, its margin wavy between the veins
            y = 0.0016 * math.sin(a * 7 * math.pi) * abs(b) + 0.004 * a * a
        elif how == 'fold':   # folded up along the midrib, as dried
            y = 0.75 * abs(z) + 0.003 * math.sin(math.pi * a)
        elif how == 'dome':   # its margins rolled under
            y = -0.28 * W * abs(b) ** 1.8 + 0.006 * math.sin(math.pi * a)
        elif how == 'twist':  # twisted along its length
            y = 0.55 * z * (a - 0.4) + 0.004 * math.sin(math.pi * a)
        # crumple
        return y + 0.0012 * noise.noise(Vector((x * 70.0, z * 70.0, 0.0)) + off)

    for i in range(nu + 1):
        a = i / nu
        e = max(leaf_env(a, oak), 0.04) * W + 0.001
        for j in range(nv + 1):
            b = -1 + 2 * j / nv
            x, z = a * L, b * e
            v.append(Vector((x, height(x, z), z)))
    for i in range(nu):
        for j in range(nv):
            a0 = i * (nv + 1) + j
            f.append((a0, a0 + 1, a0 + nv + 2, a0 + nv + 1))
    s = Shape(v, f)
    # the petiole: from the blade's base back, curving up a little
    y0 = height(0.0, 0.0)
    pts = [Vector((-P * k / 3, y0 + 0.0012 * (k / 3) ** 2 + 0.0005, 0.0)) for k in range(4)]
    s = s + sweep(pts, [0.0007, 0.0006, 0.00055, 0.0005], seg=4, cap0=False, cap1=True)
    return on_floor(s)


for _name, _p in LEAVES.items():
    auto(f'Real__leaf{_name}', lambda _p=_p: leaf(*_p), 300)


# ================================================================== wing ===
# A detached butterfly's wing: the peacock (Aglais io, wingspan 50-55 mm:
# its forewing about 30 mm from root to apex) and the small tortoiseshell
# (Aglais urticae) hibernate in sheds, lofts and outbuildings, and their
# wings are what is left when mice or a wren take them over the winter.
# The membrane is modelled as a gently creased sheet over the wing's
# bounding box, lying in x-z (x from the root along the costa to the apex,
# z back to the inner margin); the game cuts the wing's outline out of it
# and paints its scales (WINGS in `t05.rs`), so the box is the outline's.
WING_FORE = (0.0305, 0.0190)       # root to apex, costa to tornus
WING_HIND = (0.0245, 0.0215)


def wing(size, seed):
    rng = random.Random(seed)
    w, d = size
    nu, nv = 10, 7
    v, f = [], []
    for i in range(nu + 1):
        for j in range(nv + 1):
            x, z = w * i / nu, d * j / nv
            # it lies a little curled along the costa, creased across
            y = 0.0016 * (1 - j / nv) ** 2 + 0.0007 * math.sin(x / w * 5.0 + seed) * (j / nv) + 0.0004 * noise.noise(Vector((x * 300, z * 300, seed)))
            v.append(Vector((x, y, z)))
    for i in range(nu):
        for j in range(nv):
            a = i * (nv + 1) + j
            f.append((a, a + 1, a + nv + 2, a + nv + 1))
    return on_floor(Shape(v, f))


auto('Real__wingFore', lambda: wing(WING_FORE, 3), 200)
auto('Real__wingHind', lambda: wing(WING_HIND, 5), 200)


# ================================================================== moss ===
# Moss on a floor kept damp (silver moss, Bryum argenteum, and its kin of
# pavements and roofs): shoots under 1 cm tall, their leaves about 1 mm,
# packed into cushions and mats a few centimetres across; the setae 7-20 mm
# tall, each with a capsule 1-2.5 mm long, nodding, red-brown.
def moss_cushion(seed, rx, rz, h):
    """A cushion: a low dome of packed shoots, its top furred (lumpy at the
    shoots' scale), its foot spreading into the mat."""
    rng = random.Random(seed)
    nr, na = 8, 26
    v, f = [], []
    off = Vector((seed * 1.1, seed * 2.3, seed * 0.7))
    for i in range(nr + 1):
        t = i / nr
        for j in range(na):
            a = TAU * j / na
            wob = 1 + 0.18 * noise.noise(Vector((math.cos(a) * 2.0, math.sin(a) * 2.0, seed)))
            r = t * wob
            x, z = rx * r * math.cos(a), rz * r * math.sin(a)
            y = h * (1 - t ** 2.2) ** 0.8
            p = Vector((x, y, z))
            fur = noise.noise(p * 900.0 + off) * 0.0012 + noise.noise(p * 260.0 + off) * 0.0022
            p.y = max(0.0, p.y + fur * (1 - t ** 4))
            v.append(p)
    v.append(Vector((0, h + 0.0004, 0)))
    c = len(v) - 1
    for i in range(nr):
        for j in range(na):
            j2 = (j + 1) % na
            a, b = i * na + j, i * na + j2
            f.append((a, b, b + na, a + na))
    # the centre: the first ring is at r = 0 (degenerate): fan instead
    f = f[na:]
    for j in range(na):
        f.append((c, na + (j + 1) % na, na + j))
    s = Shape(v, f)
    s = weld(s, 1e-7)
    return on_floor(s)


def moss_setae(seed=731, n=9):
    """A cluster of setae, each a hair-thin stalk curving up from the
    cushion, its capsule nodding at the top."""
    rng = random.Random(seed)
    out = Shape()
    for k in range(n):
        a, r = rng.uniform(0, TAU), rng.uniform(0.0, 0.009)
        base = Vector((math.cos(a) * r, 0.0, math.sin(a) * r))
        H = rng.uniform(0.008, 0.016)
        lean = Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * 0.25 * H
        pts = [base + Vector((0, H * t, 0)) + lean * t * t for t in (0, 0.33, 0.66, 1.0)]
        out = out + sweep(pts, [0.00018, 0.00016, 0.00015, 0.00014], seg=3, cap0=False, cap1=False)
        # the capsule: hanging off the top, nodding
        top = pts[-1]
        d = (lean.normalized() if lean.length > 1e-6 else Vector((1, 0, 0))) + Vector((0, -0.9, 0))
        cap = ovoid(rng.uniform(0.0014, 0.0022), 0.0008, seg=5, rings=2, point=0.2)
        q = Vector((1, 0, 0)).rotation_difference(d.normalized())
        cap.apply(q.to_matrix().to_4x4())
        out = out + cap.move(*top)
    return on_floor(out)


auto('Real__mossCushionA', lambda: moss_cushion(711, 0.020, 0.017, 0.010), 500, smooth=70.0)
auto('Real__mossCushionB', lambda: moss_cushion(712, 0.013, 0.011, 0.008), 500, smooth=70.0)
auto('Real__mossCushionC', lambda: moss_cushion(713, 0.009, 0.008, 0.006), 500, smooth=70.0)
auto('Real__mossSetae', moss_setae, 400, smooth=80.0)


# ================================================================= grass ===
# Annual meadow grass (Poa annua), the grass of pavement cracks and damp
# corners: tufts of tillers, blades 1.3-14 cm long and about 2.5 mm wide,
# folded along the midrib, their tips shaped like a boat's prow, light
# green; grown here in the dark from a drift of silt it has gone pale and
# lank. Each tiller is a sheath with two to four blades arching out of it;
# the flowering stems carry a small open panicle.
TILLERS = [
    # (seed, height, blades)
    (801, 0.050, 2),
    (802, 0.065, 3),
    (803, 0.080, 3),
    (804, 0.095, 4),
    (805, 0.115, 3),
]


def blade(base, yaw, rise, L, w, droop, rng, n=7):
    """A grass blade from `base`: it rises at `rise` toward `yaw` and arches
    over by `droop`; folded along its midrib (a shallow V, three across),
    tapering to a boat-prow tip."""
    d0 = Vector((math.cos(yaw) * math.cos(rise), math.sin(rise), math.sin(yaw) * math.cos(rise)))
    side = Vector((-math.sin(yaw), 0, math.cos(yaw)))
    pts = []
    p = Vector(base)
    d = d0.copy()
    for k in range(n + 1):
        pts.append(p.copy())
        # bend down as it goes, a little sideways
        d = (d + Vector((0, -droop / n, 0)) + side * rng.uniform(-0.04, 0.04)).normalized()
        p = p + d * (L / n)
    v, f = [], []
    for k, c in enumerate(pts):
        t = k / n
        half = w / 2 * (1 - 0.15 * t) * (1 - smoothstep(0.8, 1.0, t) * 0.92)
        t_dir = (pts[min(k + 1, n)] - pts[max(k - 1, 0)]).normalized()
        up = t_dir.cross(side).normalized()
        fold = half * 0.35
        v += [c + side * half + up * fold, c - up * 0.0001, c - side * half + up * fold]
    for k in range(n):
        i = 3 * k
        f.append((i, i + 1, i + 4, i + 3))
        f.append((i + 1, i + 2, i + 5, i + 4))
    for p in v:
        p.y = max(p.y, 0.004)
    return Shape(v, f)


def tiller(seed, H, nb):
    rng = random.Random(seed)
    out = Shape()
    # the sheath: a short tube, the blades leaving it
    sh = 0.012 + 0.08 * H
    out = out + sweep([(0, 0, 0), (0, sh * 0.5, 0), (0.0006, sh, 0.0002)], [0.0013, 0.0012, 0.0010], seg=5, cap0=False, cap1=False)
    yaw0 = rng.uniform(0, TAU)
    for k in range(nb):
        yaw = yaw0 + TAU * k / nb + rng.uniform(-0.5, 0.5)
        L = H * rng.uniform(0.8, 1.15) * (1.15 if k == 0 else 1.0)
        rise = rng.uniform(1.0, 1.35)
        droop = rng.uniform(0.6, 1.5) * (L / 0.08)
        out = out + blade(Vector((0, sh * rng.uniform(0.6, 1.0), 0)), yaw, rise, L, rng.uniform(0.0022, 0.0029), droop, rng)
    return out


def panicle(seed=811):
    """A flowering stem: up out of a sheath, its open panicle of spikelets
    on fine branches, nodding."""
    rng = random.Random(seed)
    H = 0.10
    pts = [Vector((0, H * t, 0)) + Vector((0.012 * t * t, 0, 0.004 * t)) for t in (0, 0.25, 0.5, 0.75, 1.0)]
    out = sweep(pts, [0.0007, 0.0006, 0.0005, 0.00045, 0.0004], seg=4, cap0=False, cap1=True)
    for k in range(9):
        t = 0.62 + 0.36 * k / 8
        at = pts[0].lerp(pts[-1], t) + Vector((0.012 * t * t - 0.012 * t, 0, 0))
        a = rng.uniform(0, TAU)
        L = 0.006 + 0.012 * (1 - (t - 0.62) / 0.36)
        tip = at + Vector((math.cos(a) * L, -L * 0.3 + rng.uniform(-0.002, 0.004), math.sin(a) * L))
        out = out + sweep([at, at.lerp(tip, 0.5) + Vector((0, 0.001, 0)), tip], [0.00022, 0.0002, 0.00018], seg=3, cap0=False, cap1=False)
        sp = ovoid(rng.uniform(0.0035, 0.0048), 0.0016, seg=4, rings=2, point=0.5)
        q = Vector((1, 0, 0)).rotation_difference((tip - at).normalized())
        sp.apply(q.to_matrix().to_4x4())
        out = out + sp.move(*tip)
    return out


def grass_dead(seed=821, n=7):
    """Dead blades lying round the tuft's foot, flattened, straw-coloured."""
    rng = random.Random(seed)
    out = Shape()
    for k in range(n):
        a = rng.uniform(0, TAU)
        L = rng.uniform(0.04, 0.08)
        start = Vector((math.cos(a), 0, math.sin(a))) * rng.uniform(0.002, 0.01)
        d = Vector((math.cos(a + rng.uniform(-0.6, 0.6)), 0, math.sin(a + rng.uniform(-0.6, 0.6))))
        pts = []
        for i in range(6):
            t = i / 5
            pts.append(start + d * L * t + Vector((0, 0.0012 + 0.006 * (1 - t) ** 2 + 0.0008 * math.sin(t * 9 + k), 0)) + Vector((-d.z, 0, d.x)) * 0.008 * math.sin(t * 2.5 + k))
        ww = rng.uniform(0.0018, 0.0026)
        out = out + ribbon(pts, [ww * (1 - 0.8 * (i / 5) ** 3) for i in range(6)], twist=rng.uniform(-2, 2))
    return on_floor(out, centre=False)


for _i, _t in enumerate(TILLERS):
    auto(f'Real__grassTiller{"ABCDE"[_i]}', lambda _t=_t: tiller(*_t), 400, smooth=60.0)
auto('Real__grassPanicle', panicle, 500, smooth=70.0)
auto('Real__grassDead', grass_dead, 200, smooth=60.0)


# ================================================================== web ===
# A cellar spider's web (Pholcus phalangioides, the daddy-long-legs spider
# of cellars and ceiling corners): an untidy, loose tangle of fine, not
# sticky threads hung in the angle of a wall and the ceiling, the spider
# hanging upside down in it; its body 8-10 mm, its legs five or six times
# that (a span of 45-70 mm). An old web is grey with the dust its threads
# caught, a few strands heavy with it, prey wrapped and hung in it.
# The frame: the wall is z = 0 (the web in front of it, +z), the ceiling
# y = 0 (the web below it); x along the wall. It stays within the old web's
# box (x 0.35 m wide, 0.35 m down at most: the wall's probes and the
# ceiling's follow see the same).
WEB_W, WEB_D, WEB_OUT = 0.34, 0.34, 0.13


def web_threads(seed=901):
    """The web hung across the angle: a loose hammock of threads from the
    wall a hand below the ceiling up to the ceiling a hand out from the
    wall, spread along the angle, sagging; guy lines out from its edges to
    both, a few long lines across the angle, strands heavy with dust. (The
    first web's threads crossed a ball every way, out on the wall's face: a
    hairball, not a web in the angle, M8.)"""
    rng = random.Random(seed)
    parts = []
    # anchors: on the wall (z = 0) and on the ceiling (y = 0)
    def on_wall(lo=0.02, hi=0.28):
        return Vector((rng.uniform(-WEB_W / 2, WEB_W / 2), -rng.uniform(lo, hi), 0.0008))

    def on_ceiling(lo=0.01, hi=WEB_OUT):
        return Vector((rng.uniform(-WEB_W / 2, WEB_W / 2), -0.0008, rng.uniform(lo, hi)))
    # the hammock: the diagonal of the angle from the wall at H down to the
    # ceiling at H out, its middle sagging away from the angle
    H, HC = 0.20, 0.125                                   # down the wall, out on the ceiling
    diag = Vector((0.0, H, HC)).normalized()              # up the hammock, wall to ceiling
    away = Vector((0.0, -HC, H)).normalized()             # out of the angle, into the room

    def on_hammock(t=None, x=None, k=1.0):
        t = rng.uniform(0.05, 0.95) if t is None else t
        x = rng.uniform(-0.15, 0.15) * k if x is None else x
        base = Vector((x, -H * (1 - t), HC * t))
        # the middle sags out of the angle, most toward the middle of its span
        sag = 0.028 * math.sin(math.pi * t) * (1.0 - (x / 0.17) ** 2)
        return base + away * (sag + rng.uniform(-0.008, 0.008)) + diag * rng.uniform(-0.006, 0.006)
    lines = []
    for _ in range(34):   # the hammock's threads, mostly along the angle, some across it
        a = on_hammock()
        if rng.random() < 0.7:
            b = on_hammock(x=max(-0.16, min(0.16, a.x + rng.uniform(-0.12, 0.12))))
        else:
            b = on_hammock(t=max(0.05, min(0.95, (a.y + H) / H * 0.5 + rng.uniform(-0.4, 0.4))))
        lines.append((a, b, rng.uniform(0.0003, 0.00045)))
    for _ in range(14):   # guy lines from its edges to the wall and the ceiling
        a = on_hammock(t=rng.choice((rng.uniform(0.0, 0.25), rng.uniform(0.75, 1.0))))
        lines.append((a, on_wall() if a.y < -H / 2 else on_ceiling(), rng.uniform(0.00035, 0.0005)))
    for _ in range(8):    # a few threads hanging under it, where the spider sits
        a = on_hammock(k=0.6)
        lines.append((a, a + away * rng.uniform(0.02, 0.05) + Vector((rng.uniform(-0.02, 0.02), -0.01, 0)), rng.uniform(0.0003, 0.0004)))
    for _ in range(6):    # long lines from wall to ceiling across the angle
        lines.append((on_wall(0.05, 0.24), on_ceiling(0.03, WEB_OUT), rng.uniform(0.0003, 0.00045)))
    for _ in range(5):    # strands heavy with dust, sagging
        lines.append((on_hammock(k=0.8), on_wall(0.06, 0.30), rng.uniform(0.0007, 0.0011)))
    for a, b, r in lines:
        d = (b - a).length
        sag = d * (rng.uniform(0.02, 0.07) if r < 0.0006 else rng.uniform(0.08, 0.15))
        pts = [a.lerp(b, t) + Vector((0, -sag * 4 * t * (1 - t), 0)) for t in (0, 0.33, 0.66, 1.0)]
        parts.append(sweep(pts, [r] * 4, seg=3, cap0=False, cap1=False))
    # dust caught on the heavy strands: small clumps of fluff
    for a, b, r in lines[-5:]:
        for t in (rng.uniform(0.3, 0.5), rng.uniform(0.6, 0.85)):
            c = a.lerp(b, t) + Vector((0, -(b - a).length * 0.1 * 4 * t * (1 - t), 0))
            parts.append(sphere(r * rng.uniform(2.0, 3.0), seg=5, rings=3).scale(1.0, 1.6, 1.0).move(*c))
    s = sum(parts[1:], parts[0])
    for p in s.v:
        p.y = min(p.y, -0.0005)
        p.z = max(p.z, 0.0005)
        p.x = max(-WEB_W / 2, min(WEB_W / 2, p.x))
        p.y = max(p.y, -WEB_D)
        p.z = min(p.z, WEB_OUT + 0.01)
    return s


WEB_SPIDER_AT = (0.012, -0.135, 0.065)   # where the spider hangs (its body's middle)


def spider_body(seed=911):
    """A cellar spider dead in its web: hanging upside down (its belly up),
    the small round cephalothorax, the long cylindrical abdomen, eight long
    hair-thin legs drawn in at the knees as a dead spider's are."""
    rng = random.Random(seed)
    parts = []
    ceph = ellipsoid(0.0014, 0.0008, 0.0013, seg=8)
    parts.append(ceph)
    abd = ellipsoid(0.0012, 0.0011, 0.0034, seg=8).move(0, 0.0003, -0.0042)
    parts.append(abd)
    for side in (-1, 1):
        for k in range(4):
            # coxa angle along the body (front to back), legs 1 and 2 the longest
            ang = (0.55 + 0.32 * k) * math.pi * (1 if side > 0 else -1) * 0.5
            L = [0.050, 0.040, 0.030, 0.036][k]
            out = Vector((math.sin(ang) * side * side, 0, math.cos(ang) * (1 if k < 2 else -1)))
            out = Vector((side * abs(math.sin((0.25 + 0.17 * k) * math.pi)), 0, math.cos((0.25 + 0.17 * k) * math.pi))).normalized()
            # dead: the femur up and out (belly up, toward the web), the rest
            # of the leg folded down and in under it
            knee = out * L * 0.30 + Vector((0, 0.012 + rng.uniform(0, 0.004), 0))
            mid = knee + out * L * 0.10 + Vector((0, -0.010, 0))
            foot = mid - out * L * 0.12 + Vector((0, -0.012 - rng.uniform(0, 0.005), 0)) + Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * 0.002
            pts = chaikin([out * 0.0012, knee, mid, foot], 1)
            parts.append(sweep(pts, [0.00028 - 0.00012 * i / (len(pts) - 1) for i in range(len(pts))], seg=3, cap0=False, cap1=True))
    s = sum(parts[1:], parts[0])
    return s.rot(math.pi, 0, 0).move(*WEB_SPIDER_AT)


def spider_wrap(seed=921):
    """Prey wrapped in silk and hung: two bundles on short threads."""
    rng = random.Random(seed)
    out = Shape()
    for at in ((-0.07, -0.16, 0.05), (0.09, -0.21, 0.07)):
        b = noisy(ellipsoid(0.0022, 0.0034, 0.0022, seg=7), 0.0004, scale=900.0, seed=seed)
        top = Vector(at) + Vector((0, 0.012, 0))
        out = out + b.move(*at) + sweep([top, Vector(at) + Vector((0, 0.003, 0))], [0.0002, 0.0002], seg=3, cap0=False, cap1=False)
    return out


auto('Real__spiderWebThreads', web_threads, 1600, smooth=80.0)
auto('Real__spiderBody', spider_body, 600, smooth=70.0)
auto('Real__spiderWrap', spider_wrap, 300, smooth=60.0)
