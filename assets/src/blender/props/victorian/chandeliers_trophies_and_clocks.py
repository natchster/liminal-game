"""Batch b18: the "victorian" theme, the showpieces of the Maze's hoard.

Crystal chandeliers, mounted deer heads and antlers, family and pet
portraits, barometers, a mounted fish, cuckoo and grandfather clocks,
candle lanterns, horseshoes, cast-iron kettles, wagon wheels, bellows, a
rocking chair, a music box, records and a turntable, floor and moon globes.
Carved wood, turned brass, cast iron and taxidermy. Crystal, glass, flames,
glows and flat faces/labels are transparent, basic material or decals in the game
and stay primitives (see docs/research/props-remodelling/chandeliers_trophies_and_clocks.md).
"""
from propkit import *
import propkit
from mathutils import Vector

fixw = propkit._fix_winding
HPI = math.pi / 2


# ------------------------------------------------------------ local kit --
def _ccw(pts):
    a = sum(p[0] * q[1] - q[0] * p[1] for p, q in zip(pts, pts[1:] + pts[:1]))
    return list(pts) if a > 0 else list(reversed(pts))


def offset2d(pts, d):
    """Offset a CCW outline inward by d (mitred corners)."""
    n, out = len(pts), []
    for i in range(n):
        p0, p1, p2 = Vector(pts[i - 1]), Vector(pts[i]), Vector(pts[(i + 1) % n])
        e0, e1 = (p1 - p0).normalized(), (p2 - p1).normalized()
        n0, n1 = Vector((-e0.y, e0.x)), Vector((-e1.y, e1.x))
        m = n0 + n1
        m = (m if m.length > 1e-9 else n0).normalized()
        q = p1 + m * (d / max(0.35, m.dot(n0)))
        out.append((q.x, q.y))
    return out


def stack(pts, prof, loop=False):
    """Moulding: the XY outline `pts`, inset by `inset` at depth z for every
    (inset, z) in prof, rings bridged, ends capped (loop=True closes a ring)."""
    base = _ccw(pts)
    rings = [base if abs(i) < 1e-9 else offset2d(base, i) for i, _ in prof]
    n = len(base)
    v, f = [], []
    for ring, (_, z) in zip(rings, prof):
        v += [(x, y, z) for x, y in ring]
    R = len(prof)
    for r in range(R - 1 + (1 if loop else 0)):
        a, b = r * n, ((r + 1) % R) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    if not loop:
        f.append(tuple(reversed(range(n))))
        f.append(tuple((R - 1) * n + i for i in range(n)))
    return fixw(Shape(v, f))


def ring_lathe(profile, seg=24, arc=TAU):
    """Revolve a CLOSED profile loop [(r, y), ...] around +Y."""
    v, f, n = [], [], len(profile)
    for r, y in profile:
        for i in range(seg):
            a = arc * i / seg
            v.append((r * math.sin(a), y, r * math.cos(a)))
    for k in range(n):
        a, b = k * seg, ((k + 1) % n) * seg
        for i in range(seg):
            j = (i + 1) % seg
            f.append((a + i, b + i, b + j, a + j))
    return fixw(Shape(v, f))


def ring_sec(R, sec, seg=32, arc=TAU, a0=0.0, rmod=None):
    """Sweep a closed section [(dr, dz), ...] around Z in the XY plane (a
    torus_geo! with any cross-section). arc < TAU caps both ends.
    rmod(angle) -> (k_section, dz, dR) varies the section along the ring."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        k, dzz, dR = rmod(a) if rmod else (1.0, 0.0, 0.0)
        for dr, dz in sec:
            rr = R + dR + dr * k
            v.append((rr * math.cos(a), rr * math.sin(a), dz * k + dzz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return fixw(Shape(v, f))


def loft(rings, caps=True):
    """Bridge equal-length closed 3D rings; cap both ends."""
    n = len(rings[0])
    v, f = [], []
    for ring in rings:
        v += [tuple(p) for p in ring]
    for r in range(len(rings) - 1):
        a, b = r * n, (r + 1) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    if caps:
        f.append(tuple(reversed(range(n))))
        f.append(tuple((len(rings) - 1) * n + i for i in range(n)))
    return fixw(Shape(v, f))


def sweep(points, sec, scale=None, smooth=0, up=(0, 0, 1), caps=True):
    """Sweep a closed 2D section [(x, y), ...] along a 3D polyline.
    scale(t) -> k or (kx, ky), t from 0 to 1 along the path."""
    pts = [Vector(p) for p in points]
    for _ in range(smooth):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    n = len(pts)
    tan = [(pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized() for i in range(n)]
    nrm = tan[0].cross(Vector(up))
    if nrm.length < 1e-6:
        nrm = tan[0].cross(Vector((1, 0, 0)))
    nrm.normalize()
    m = len(sec)
    v, f = [], []
    for i in range(n):
        if i:
            nrm = (tan[i - 1].rotation_difference(tan[i]) @ nrm).normalized()
        b = tan[i].cross(nrm).normalized()
        k = scale(i / (n - 1)) if scale else 1.0
        kx, ky = k if isinstance(k, tuple) else (k, k)
        for (x, y) in sec:
            v.append(pts[i] + nrm * (x * kx) + b * (y * ky))
    for i in range(n - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, (i + 1) * m + j2, (i + 1) * m + j))
    if caps:
        f.append(tuple(reversed(range(m))))
        f.append(tuple((n - 1) * m + j for j in range(m)))
    return fixw(Shape(v, f))


def circ(r, n=8, sx=1.0, sy=1.0, phase=0.0):
    return [(r * sx * math.cos(TAU * i / n + phase), r * sy * math.sin(TAU * i / n + phase)) for i in range(n)]


def sell(a, b, n=20, p=2.5, phase=0.0):
    """Superellipse outline (p=2 ellipse, larger = squarer)."""
    out = []
    for i in range(n):
        t = TAU * i / n + phase
        c, s = math.cos(t), math.sin(t)
        out.append((a * math.copysign(abs(c) ** (2 / p), c), b * math.copysign(abs(s) ** (2 / p), s)))
    return out


def interp(tab, x):
    """Piecewise-linear lookup in [(x, y), ...]."""
    if x <= tab[0][0]:
        return tab[0][1]
    for (x0, y0), (x1, y1) in zip(tab, tab[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * (x - x0) / ((x1 - x0) or 1)
    return tab[-1][1]


def lobes(s, n, depth, y0, y1, phase=0.0):
    """Gadroons / flutes: push the radius in and out n times around Y between
    y0 and y1 (depth in metres at full strength, fading at the ends)."""
    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6 or not (y0 - 1e-7 <= p.y <= y1 + 1e-7):
            return p
        t = (p.y - y0) / ((y1 - y0) or 1)
        fade = max(0.0, math.sin(math.pi * min(1.0, max(0.0, t)))) ** 0.5
        a = math.atan2(p.x, p.z)
        k = max(0.0, r - depth * fade * (0.5 - 0.5 * math.cos(n * a + phase))) / r
        p.x *= k
        p.z *= k
        return p
    return s.warp(w)


def torus_bounds(R, r, arc, kind='torus'):
    """Ideal bounds of torus_geo!(R, r, .., .., arc), as `assets::props`
    samples them (24 radial x 128 tubular)."""
    xs, ys = [], []
    for i in range(129):
        u = arc * i / 128
        for j in range(25):
            w = TAU * j / 24
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r), kind)


def x_axis(s):
    """A Y-axis shape laid along +X (its +Y end goes to +X)."""
    return s.rot(z=-HPI)


# ================================================== crystal chandeliers ==
def chand_chain():
    # oval brass links, every other one turned 90 degrees, 0.40 long
    L, R, r, sy = 0.40, 0.0048, 0.0011, 1.8
    outer, inner = 2 * (R + r) * sy, 2 * (R - r) * sy
    pitch = (outer + inner) / 2
    n = int((L - outer) / pitch) + 1
    pitch = (L - outer) / (n - 1)
    link = torus(R, r, seg=8, tseg=3).scale(1, sy, 1)
    out = Shape()
    for i in range(n):
        out = out + link.copy().rot(y=HPI * (i % 2)).move(y=-L / 2 + outer / 2 + i * pitch)
    return out

part('CrystalChandeliers__chain', f_cyl(0.003, 0.003, 0.40), chand_chain, smooth=60)

# turned brass collar under the chain: a bead, then a gadrooned bell flaring
# out to a rolled lip
part('CrystalChandeliers__cap', f_cyl(0.030, 0.020, 0.010), lambda: lobes(lathe(
    [(0, -0.005), (0.018, -0.005), (0.0205, -0.0036), (0.0212, -0.0015), (0.026, 0.0004),
     (0.0296, 0.0024), (0.0296, 0.0042), (0.0262, 0.005), (0, 0.005)], seg=16),
    8, 0.0012, -0.0015, 0.0024), smooth=60)

# the brass heart: a gadrooned bowl with a drop finial below and a collared
# neck above
part('CrystalChandeliers__body', f_sphere(0.030), lambda: lobes(lathe(
    [(0, -0.030), (0.0045, -0.0285), (0.0038, -0.0238), (0.013, -0.0195), (0.024, -0.011),
     (0.0298, -0.001), (0.0272, 0.0085), (0.0175, 0.0155), (0.0108, 0.0205), (0.0135, 0.0245),
     (0.0120, 0.0290), (0, 0.030)], seg=12), 6, 0.0032, -0.0195, 0.0085), smooth=70)

# a turned arm: root collar, two knops, a flared socket under the cup
part('CrystalChandeliers__arm', f_cyl(0.003, 0.003, 0.10), lambda: lathe(
    [(0, -0.05), (0.0036, -0.05), (0.0042, -0.0465), (0.0022, -0.041), (0.0019, -0.016),
     (0.0036, -0.0115), (0.0020, -0.007), (0.0019, 0.026), (0.0032, 0.030), (0.0020, 0.034),
     (0.0020, 0.044), (0.0034, 0.0488), (0, 0.05)], seg=8), smooth=50)

# bobeche: a knopped stem swelling into a flared drip pan with a dished top
part('CrystalChandeliers__cup', f_cyl(0.008, 0.006, 0.006), lambda: lathe(
    [(0, -0.003), (0.0042, -0.003), (0.0036, -0.0012), (0.0074, 0.0017), (0.0080, 0.0030),
     (0.0054, 0.0024), (0, 0.0024)], seg=10), smooth=50)


def chand_candle():
    # wax taper with a softened, dished top, a wick and one run of drip
    s = lathe([(0, -0.012), (0.0050, -0.012), (0.0050, 0.0098), (0.0042, 0.0118), (0.0012, 0.0110),
               (0.0005, 0.0126), (0, 0.0128)], seg=8)

    def drip(p):
        a = math.atan2(p.x, p.z)
        r = math.hypot(p.x, p.z)
        if r > 0.004 and p.y > -0.004:
            k = math.exp(-(a / 0.45) ** 2) * min(1.0, (p.y + 0.004) / 0.012)
            p.x *= 1 + 0.12 * k
            p.z *= 1 + 0.12 * k
        return p
    return s.warp(drip)

part('CrystalChandeliers__candle', f_cyl(0.005, 0.005, 0.024), chand_candle, smooth=60)


# ======================================================== deer head mounts ==
def shield_outline(w, h, top='crest'):
    """Taxidermy shield: straight sides curving to a point at the bottom;
    the top either a scalloped crest peaking in the middle or straight."""
    hw, hh = w / 2, h / 2
    if top == 'crest':
        tp = [(-hw, hh * 0.80), (-hw * 0.86, hh * 0.94), (-hw * 0.6, hh * 0.95), (-hw * 0.34, hh * 0.90),
              (-hw * 0.14, hh * 0.93), (0, hh)]
    else:
        tp = [(-hw, hh * 0.96), (-hw * 0.96, hh), (0, hh)]
    rt = [(-x, y) for x, y in reversed(tp[:-1])]
    side = [(hw, hh * 0.1), (hw * 0.97, -hh * 0.3), (hw * 0.84, -hh * 0.6), (hw * 0.6, -hh * 0.83),
            (hw * 0.3, -hh * 0.95), (0, -hh)]
    lside = [(-x, y) for x, y in reversed(side[:-1])]
    return round_outline(tp + rt + side + lside, min(w, h) * 0.02, seg=2)


def ogee_plaque(w, h, t, top='crest'):
    # routed ogee edge round a raised field
    z0 = -t / 2
    return stack(shield_outline(w, h, top), [(0, z0), (0, z0 + t * 0.62), (0.0022, t / 2 - t * 0.14),
                                            (0.0048, t / 2 - t * 0.12), (0.0062, t / 2 - t * 0.02),
                                            (0.0085, t / 2)])

part('DeerHeadMounts__plaque', f_box(0.28, 0.32, 0.020), lambda: ogee_plaque(0.28, 0.32, 0.020), smooth=40)


def deer_head():
    # head and neck in world size (the mesh is scaled 0.85, 1.2, 0.9): a neck
    # rising out of the plaque, cheeks, a narrow brow ridge between the eyes,
    # the poll sloping back to the antler bases
    lv = [(-0.084, 0.030, -0.030, -0.063), (-0.075, 0.046, -0.004, -0.063), (-0.058, 0.054, 0.016, -0.063),
          (-0.040, 0.057, 0.032, -0.062), (-0.020, 0.0595, 0.046, -0.060), (-0.002, 0.058, 0.056, -0.057),
          (0.016, 0.054, 0.063, -0.053), (0.034, 0.049, 0.060, -0.048), (0.052, 0.042, 0.050, -0.041),
          (0.066, 0.033, 0.036, -0.033), (0.077, 0.022, 0.018, -0.023), (0.084, 0.010, 0.003, -0.012)]
    rings = []
    n = 22
    for y, hw, zf, zb in lv:
        zc = zb + 0.46 * (zf - zb)
        ring = []
        for i in range(n):
            t = TAU * i / n
            c, s = math.cos(t), math.sin(t)
            p = 1.55 if s > 0 else 2.6
            x = hw * math.copysign(abs(c) ** (2 / p), c)
            d = (zf - zc) if s > 0 else (zc - zb)
            z = zc + d * math.copysign(abs(s) ** (2 / p), s)
            ring.append((x, y, z))
        rings.append(ring)
    s = loft(rings)
    return s.scale(1 / 0.85, 1 / 1.2, 1 / 0.9).fit((-0.07, -0.07, -0.07), (0.07, 0.07, 0.07))

part('DeerHeadMounts__head', f_sphere(0.070), deer_head, smooth=75)


def deer_snout():
    # muzzle in world size (mesh scaled 0.7, 0.7, 1): deep at the cheeks,
    # tapering to a rounded nose, the bridge sloping down, a mouth crease
    lv = [(-0.040, 0.024, -0.026, 0.028), (-0.026, 0.028, -0.028, 0.025), (-0.010, 0.026, -0.028, 0.019),
          (0.006, 0.0225, -0.027, 0.011), (0.020, 0.0195, -0.025, 0.005), (0.031, 0.0165, -0.022, 0.000),
          (0.037, 0.0125, -0.019, -0.004), (0.040, 0.006, -0.015, -0.008)]
    rings = []
    n = 18
    for z, hw, yb, yt in lv:
        yc, hh = (yb + yt) / 2, (yt - yb) / 2
        ring = []
        for i in range(n):
            t = TAU * i / n
            c, s = math.cos(t), math.sin(t)
            x = hw * math.copysign(abs(c) ** (2 / 2.4), c)
            y = yc + hh * math.copysign(abs(s) ** (2 / 2.4), s)
            if z > -0.02 and -0.55 < s < -0.15:
                x *= 0.9                                   # mouth line
            ring.append((x, y, z))
        rings.append(ring)
    return loft(rings).scale(1 / 0.7, 1 / 0.7, 1).fit((-0.04, -0.04, -0.04), (0.04, 0.04, 0.04))

part('DeerHeadMounts__snout', f_sphere(0.040), deer_snout, smooth=75)


def deer_ear(R=0.014, H=0.030):
    # a cupped ear: U-section shell open to the front, leaf-shaped in outline
    wt = [(0, 0.55), (0.2, 0.85), (0.45, 1.0), (0.7, 0.8), (0.88, 0.45), (1.0, 0.06)]
    rings = []
    for k in range(9):
        t = k / 8
        w = R * interp(wt, t)
        th = min(0.0013, w * 0.45)
        fwd = w * 0.9
        y = -H / 2 + H * t
        dz = -0.003 * t * t
        pts = [(w, fwd)] + [(w * math.cos(math.pi * a / 4), -w * math.sin(math.pi * a / 4)) for a in range(5)] + \
              [(-w, fwd), (-(w - th), fwd)] + \
              [((w - th) * math.cos(math.pi * a / 4), -(w - th) * math.sin(math.pi * a / 4)) for a in range(4, -1, -1)] + \
              [(w - th, fwd)]
        rings.append([(x, y, z + dz) for x, z in pts])
    return loft(rings).fit((-R, -H / 2, -R), (R, H / 2, R))

part('DeerHeadMounts__ear', f_cone(0.014, 0.030), deer_ear, smooth=60)


def antler(h, r0, r1, burr, curve, seg=7, seed=0):
    """Antler beam/tine along +Y: burr ring at the base (if burr), pearled
    beam tapering from r0 to r1 and curving forward (+Z) by `curve`."""
    y0 = -h / 2 + (0.1 * h if burr else 0.0)
    n = 9
    pts = [(0, y0 + (h / 2 - y0) * k / (n - 1), curve * (k / (n - 1)) ** 1.6) for k in range(n)]
    s = tube(pts, r0, seg=seg, r_end=r1)
    s = noisy(s, r0 * 0.1, scale=900, seed=seed)
    if burr:
        b = lathe([(0, -h / 2), (burr * 0.8, -h / 2), (burr, -h / 2 + 0.03 * h), (burr * 0.92, -h / 2 + 0.08 * h),
                   (r0 * 0.9, -h / 2 + 0.11 * h), (0, -h / 2 + 0.11 * h)], seg=9)
        s = s + noisy(b, burr * 0.08, scale=1400, seed=seed + 3)
    return s

part('DeerHeadMounts__beam', f_cyl(0.005, 0.005, 0.10), lambda: antler(0.10, 0.0046, 0.0026, 0.0060, 0.004), smooth=70)
part('DeerHeadMounts__tine', f_cyl(0.003, 0.003, 0.05),
     lambda: antler(0.05, 0.0030, 0.0006, 0.0, 0.0025, seg=6, seed=2).move(z=-0.0008), smooth=70)


# ====================================================== family portraits ==
def pillow(w, h, t, zfn, nx=16, ny=6):
    """A bar/board with a flat back at -t/2 and a front surface z = zfn(u, v)
    (u, v in -1..1 across w and h), sampled denser towards the edges."""
    us = [-math.cos(math.pi * i / nx) for i in range(nx + 1)]
    vs = [-math.cos(math.pi * j / ny) for j in range(ny + 1)]
    v, f = [], []
    for j, vv in enumerate(vs):
        for i, uu in enumerate(us):
            v.append((uu * w / 2, vv * h / 2, zfn(uu, vv)))
    k = nx + 1
    for j in range(ny):
        for i in range(nx):
            a = j * k + i
            f.append((a, a + 1, a + k + 1, a + k))
    rim = [i for i in range(nx + 1)] + [j * k + nx for j in range(1, ny + 1)] + \
          [ny * k + i for i in range(nx - 1, -1, -1)] + [j * k for j in range(ny - 1, 0, -1)]
    base = len(v)
    for idx in rim:
        x, y, _ = v[idx]
        v.append((x, y, -t / 2))
    m = len(rim)
    for q in range(m):
        a, b = rim[q], rim[(q + 1) % m]
        f.append((b, a, base + q, base + (q + 1) % m))
    f.append(tuple(base + q for q in range(m)))
    return fixw(Shape(v, f))


def bullnose(s, p):
    return max(0.0, 1 - abs(s) ** p) ** 0.5

# the four frame bars share one constructor (long top/bottom bars and short
# upright stiles), so the moulding crowns towards all four edges and reads as
# a rounded moulding either way; the crest stands 3 mm proud of the photo
part('FamilyPortraits__bar', f_box(0.228, 0.014, 0.015), lambda: pillow(
    0.228, 0.014, 0.015, lambda u, v: 0.0025 + 0.0080 * bullnose(u, 6) * bullnose(v, 4), nx=18, ny=6), smooth=60)


# ============================================================ barometers ==
# turned wooden case, axis towards the room (+Y): a rounded back, a cove and
# a bead round the side, a flat bezel 5 mm wide round the dial, the dial seat
# recessed behind the printed face
part('Barometers__rim', f_cyl(0.11, 0.11, 0.030), lambda: lathe(
    [(0, -0.015), (0.103, -0.015), (0.1085, -0.0128), (0.110, -0.0085), (0.1085, -0.0045), (0.1062, -0.0015),
     (0.1075, 0.0025), (0.1100, 0.0062), (0.1092, 0.0112), (0.1060, 0.0146), (0.1010, 0.0150),
     (0.0985, 0.0135), (0.0975, 0.0118), (0, 0.0118)], seg=22), smooth=40)


def baro_needle():
    # blued-steel pointer: spade tip, fine shaft, hub ring round the pivot
    # (pivot at y = -0.030 after the placer's translate) and a short tail
    half = [(0, 0.035), (0.0017, 0.0295), (0.0025, 0.0262), (0.0015, 0.0232), (0.0005, 0.0222),
            (0.0005, -0.0262), (0.0006, -0.0345), (0, -0.035)]
    out = half + [(-x, y) for x, y in reversed(half[1:-1])]
    s = prism(out, 0.0012)
    hub = cyl(0.0025, 0.0025, 0.0020, seg=12, bevel_r=0.0004).rot(x=HPI).move(y=-0.030)
    return s + hub

part('Barometers__needle', f_box(0.005, 0.070, 0.003), baro_needle, smooth=40)


# ======================================================= mounted antlers ==
part('MountedAntlers__plaque', f_box(0.18, 0.22, 0.020),
     lambda: ogee_plaque(0.18, 0.22, 0.020, top='flat'), smooth=40)


def antler_crest():
    # carved shell crest on the plaque (the mesh is squashed 1, 0.5, 0.2 into
    # a half-lens): a gadrooned half-dome whose flutes fan out from the
    # bottom centre, a bead round the rim
    dome = lathe([(0, -0.090), (0.0900, -0.090), (0.0900, -0.010), (0.0885, -0.004), (0.0895, 0.004),
                  (0.0860, 0.012), (0.0780, 0.036), (0.0620, 0.060), (0.0380, 0.080), (0.0150, 0.088),
                  (0, 0.090)], seg=40)
    dome = lobes(dome, 10, 0.0075, 0.012, 0.086, phase=math.pi)
    dome.rot(x=HPI)
    dome = cut(dome, box(0.3, 0.2, 0.3).move(y=-0.1))
    return dome.fit((-0.09, 0, -0.09), (0.09, 0.09, 0.09))


part('MountedAntlers__crest', f_bounds((-0.09, 0.0, -0.09), (0.09, 0.09, 0.09)), antler_crest, smooth=45)
part('MountedAntlers__trunk', f_cyl(0.006, 0.010, 0.14),
     lambda: antler(0.14, 0.0072, 0.0040, 0.0100, 0.006, seg=8, seed=5).move(z=-0.002), smooth=70)
part('MountedAntlers__prong', f_cyl(0.004, 0.005, 0.055),
     lambda: antler(0.055, 0.0048, 0.0008, 0.0, 0.0024, seg=6, seed=7).move(z=-0.0004), smooth=70)


# ========================================================= mounted fish ==
def oval_plaque(w, h, t, p=2.2, n=40):
    z0 = -t / 2
    return stack(sell(w / 2, h / 2, n, p), [(0, z0), (0, z0 + t * 0.6), (0.0022, t / 2 - t * 0.16),
                                          (0.0050, t / 2 - t * 0.12), (0.0065, t / 2 - t * 0.02), (0.0090, t / 2)])

part('MountedFish__plaque', f_box(0.35, 0.15, 0.018), lambda: oval_plaque(0.35, 0.15, 0.018), smooth=40)


def fish_body():
    # a bass in world size (the mesh is scaled 2.5, 1, 0.6): jutting lower
    # jaw at -X, the gill cover as a step, deep belly, slim tail stock at +X
    lv = [(-0.1125, 0.002, -0.012, 0.006), (-0.100, 0.012, -0.020, 0.011), (-0.085, 0.022, -0.028, 0.016),
          (-0.070, 0.031, -0.034, 0.020), (-0.056, 0.037, -0.038, 0.0225), (-0.053, 0.0385, -0.0395, 0.0245),
          (-0.020, 0.045, -0.043, 0.027), (0.020, 0.043, -0.040, 0.026), (0.050, 0.034, -0.032, 0.021),
          (0.075, 0.022, -0.021, 0.014), (0.095, 0.0135, -0.0125, 0.009), (0.1125, 0.010, -0.009, 0.007)]
    rings = []
    n = 16
    for x, yt, yb, hd in lv:
        yc, hh = (yt + yb) / 2, (yt - yb) / 2
        ring = []
        for i in range(n):
            t = TAU * i / n
            c, s = math.cos(t), math.sin(t)
            ring.append((x, yc + hh * math.copysign(abs(s) ** (2 / 1.8), s), hd * math.copysign(abs(c) ** (2 / 1.8), c)))
        rings.append(ring)
    return loft(rings).scale(1 / 2.5, 1, 1 / 0.6).fit((-0.045, -0.045, -0.045), (0.045, 0.045, 0.045))

part('MountedFish__body', f_sphere(0.045), fish_body, smooth=70)


def fish_tail():
    # forked caudal fin growing out of the tail stock (frame -Y end, which
    # butts the body), mid-swish: the two lobes twist opposite ways. Frame X
    # is the fish's depth, frame Z its height.
    rings = []
    zs = [-0.028 + 0.056 * k / 8 for k in range(9)]
    for z in zs:
        az = abs(z)
        yL = -0.0238 if az <= 0.009 else -0.0238 + (az - 0.009) / 0.019 * 0.0415
        yT = 0.006 + 0.014 * (az / 0.028) ** 1.2
        yL = min(yL, yT - 0.002)
        side = []
        for i in range(6):
            y = yL + (yT - yL) * i / 5
            th = 0.0007 + 0.0055 * max(0.0, 1 - (y + 0.0245) / 0.014) * max(0.0, 1 - az / 0.012)
            tw = -0.82 * z * max(0.0, (y + 0.0245) / 0.0445) ** 1.5
            side.append((y, th, tw))
        ring = [(tw + th, y, z) for y, th, tw in side] + [(tw - th, y, z) for y, th, tw in reversed(side)]
        rings.append(ring)
    return loft(rings)

part('MountedFish__tail', f_cone(0.028, 0.040), fish_tail, smooth=50)


def fish_dorsal():
    # spiny dorsal fin: frame -Z is up, frame X runs head (-X) to tail, frame
    # Y is the fish's depth. Only its crest clears the back; the root is a
    # lozenge buried in the body.
    top = [(-0.020, 0.0), (-0.017, -0.012), (-0.013, -0.0145), (-0.0115, -0.0200), (-0.008, -0.0150),
           (-0.0055, -0.0195), (-0.002, -0.0142), (0.0005, -0.0182), (0.004, -0.0135), (0.0065, -0.0165),
           (0.010, -0.0125), (0.0125, -0.0145), (0.0200, -0.0080), (0.0200, 0.002)]
    blade = prism([(x, z) for x, z in top], 0.0016).rot(x=HPI)
    root = ellipsoid(0.017, 0.015, 0.012, seg=10).move(z=0.008)
    return blade + root

part('MountedFish__dorsal', f_cone(0.020, 0.030), fish_dorsal, smooth=45)

# glass eye: a domed cornea over a flat iris ring, looking out (+Z)
part('MountedFish__eye', f_sphere(0.006), lambda: lathe(
    [(0, -0.006), (0.0058, -0.002), (0.0056, 0.0015), (0.0044, 0.003), (0.0022, 0.0055), (0, 0.006)],
    seg=8).rot(x=HPI), smooth=60)


# ========================================================= cuckoo clocks ==
def cuckoo_body():
    # Black Forest case: bevelled box, a raised frame of stiles and rails on
    # the front, half-round corner columns and a scalloped apron at the foot
    W, H, D = 0.22, 0.20, 0.06
    case = rbox(W, H, D - 0.006, r=0.004, seg=1).move(z=-0.003)
    fr = Shape()
    for sx in (-1, 1):
        fr = fr + rbox(0.016, H - 0.012, 0.008, r=0.0018, seg=1).move(x=sx * (W / 2 - 0.016), z=D / 2 - 0.004)
        fr = fr + cyl(0.0055, 0.0055, H - 0.004, seg=8, bevel_r=0.0012).move(x=sx * (W / 2 - 0.0055), z=D / 2 - 0.0085)
    for sy in (-1, 1):
        fr = fr + rbox(W - 0.03, 0.012, 0.008, r=0.0018, seg=1).move(y=sy * (H / 2 - 0.012), z=D / 2 - 0.004)
    sc = [(-0.085, 0.0)]
    for i in range(7):
        x0 = -0.085 + i * 0.17 / 7
        for k in range(1, 5):
            a = math.pi * k / 4
            sc.append((x0 + 0.17 / 14 * (1 - math.cos(a)), -0.009 * math.sin(a)))
    sc.append((0.085, 0.0))
    apron = prism(sc[::-1] + [(0.085, 0.004), (-0.085, 0.004)][::-1], 0.006).move(
        y=-H / 2 + 0.009, z=D / 2 - 0.006)
    return case + fr + apron

part('CuckooClocks__body', f_box(0.22, 0.20, 0.06), cuckoo_body, smooth=35)


def cuckoo_roof():
    # built in world axes (x across, y up, z out of the wall) then turned
    # into the rotated 3-sided cone's frame: frame = (-z, y, x)
    teeth = 5
    top = []
    for i in range(teeth + 1):
        t = i / teeth
        x = -0.14 + 0.14 * t
        y = -0.042 + 0.092 * t
        top.append((x, y))
        if i < teeth:
            top.append((x + 0.14 / teeth * 0.15, y + 0.0045))
    right = [(-x, y) for x, y in reversed(top[:-1])]
    outline = top + right + [(0.14, -0.050), (0.0, 0.040), (-0.14, -0.050)]
    roof = prism(outline, 0.255, bevel_r=0.0008).move(z=-0.0125)
    gable = prism([(-0.112, -0.058), (0.112, -0.058), (0.0, 0.020)], 0.010).move(z=0.028)
    door = prism(round_outline([(-0.011, -0.050), (0.011, -0.050), (0.011, -0.032), (0.0, -0.024), (-0.011, -0.032)],
                               0.004, seg=2), 0.004, bevel_r=0.0006).move(z=0.034)
    sill = rbox(0.030, 0.003, 0.010, r=0.001, seg=1).move(y=-0.0515, z=0.036)
    s = roof + gable + door + sill
    return s.warp(lambda p: Vector((-p.z, p.y, p.x)))

part('CuckooClocks__roof', f_cone(0.14, 0.10), cuckoo_roof, smooth=30)

# dial bezel: turned half-round ring with an inner bead
part('CuckooClocks__rim', f_torus(0.055, 0.004), lambda: ring_sec(0.055, [
    (-0.004, -0.004), (0.004, -0.004), (0.0038, 0.0005), (0.0024, 0.0033), (0.0004, 0.0040), (-0.0016, 0.0032),
    (-0.0026, 0.0020), (-0.0036, 0.0022), (-0.0040, 0.0008)], seg=30), smooth=50)


def clock_hand(L, tip, pivot, W, T, spade=0.30):
    """Flat hand along Y: the pointer runs from the pivot to y = tip (sign
    gives direction), a spade near the tip, a ring round the pivot and a
    short tail with a round end."""
    d = 1 if tip > pivot else -1
    ln = abs(tip - pivot)
    sp = lambda k: pivot + d * ln * k
    half = [(0.0, tip), (W * 0.35, sp(0.93)), (W * 0.5, sp(1 - spade * 0.55)), (W * 0.18, sp(1 - spade)),
            (W * 0.12, sp(0.25)), (W * 0.2, sp(0.12))]
    tail_end = pivot - d * (L - ln)
    half_t = [(W * 0.14, pivot - d * (L - ln) * 0.5), (W * 0.3, tail_end + d * 0.0015), (0.0, tail_end)]
    right = half + half_t
    out = right + [(-x, y) for x, y in reversed(right[1:-1])]
    s = prism(out if d < 0 else out[::-1], T)
    hub = cyl(W * 0.5, W * 0.5, T * 1.1, seg=10, bevel_r=0.0002).rot(x=HPI).move(y=pivot)
    return s + hub

part('CuckooClocks__hour', f_box(0.004, 0.030, 0.003),
     lambda: clock_hand(0.030, -0.015, 0.005, 0.004, 0.0016, spade=0.45), smooth=40)
part('CuckooClocks__minute', f_box(0.003, 0.040, 0.003),
     lambda: clock_hand(0.040, -0.020, 0.010, 0.003, 0.0016, spade=0.3), smooth=40)


def link_chain(L, R, r, sy, seg=6, tseg=3):
    """Oval links, every other one turned 90 degrees, filling y = -L/2..L/2."""
    outer, inner = 2 * (R + r) * sy, 2 * (R - r) * sy
    pitch = (outer + inner) / 2
    n = int((L - outer) / pitch) + 1
    pitch = (L - outer) / (n - 1)
    link = torus(R, r, seg=seg, tseg=tseg).scale(1, sy, 1)
    out = Shape()
    for i in range(n):
        out = out + link.copy().rot(y=HPI * (i % 2)).move(y=-L / 2 + outer / 2 + i * pitch)
    return out

part('CuckooClocks__chain', f_cyl(0.001, 0.001, 0.12), lambda: link_chain(0.12, 0.0024, 0.0005, 1.6), smooth=60)


def pinecone():
    # cast-iron pine-cone weight (hangs apex down: the placer flips it): an
    # egg-shaped core whose rings are pushed in and out in a checkerboard, so
    # the scales read as the cone's diamond lattice; a collar and a hanging
    # eye at the top (frame -Y)
    n = 10
    prof = [(0.0, 0.0120)]
    for k in range(n):
        t = (k + 0.5) / n
        y = 0.0112 - 0.0198 * t
        r = 0.0090 * math.sin(math.pi * (0.08 + 0.64 * t)) ** 0.7
        prof.append((r, y))
    prof += [(0.0034, -0.0092), (0.0, -0.0100)]
    s = lathe(prof[::-1], seg=10)

    def scales(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-5 or p.y < -0.0088 or p.y > 0.0110:
            return p
        k = int(round((0.0112 - p.y) / 0.0198 * n - 0.5))
        i = int(round(math.atan2(p.x, p.z) / (TAU / 10))) % 10
        f = 1.13 if (i + k) % 2 == 0 else 0.90
        p.x *= f
        p.z *= f
        return p
    s.warp(scales)
    eye = torus(0.0016, 0.0005, seg=5, tseg=3).rot(y=HPI).move(y=-0.0105)
    return s + eye


part('CuckooClocks__cone', f_cone(0.010, 0.024), pinecone, smooth=45)


# ====================================================== candle lanterns ==
def lantern_base():
    # square stepped plinth, eased edges, a candle socket in the middle
    s = rbox(0.090, 0.005, 0.090, r=0.0015, seg=1).move(y=-0.0025)
    s = s + rbox(0.074, 0.004, 0.074, r=0.0014, seg=1).move(y=0.0015)
    s = s + rbox(0.064, 0.0015, 0.064, r=0.0005, seg=1).move(y=0.0042)
    return s

part('CandleLanterns__base', f_cyl(0.045, 0.045, 0.010), lantern_base, smooth=30)


def lantern_top():
    # square cap: a drip rim, a low pyramid roof with a vent collar and two
    # ears for the bail handle
    s = rbox(0.090, 0.0035, 0.090, r=0.0012, seg=1).move(y=-0.00325)
    s = s + lathe([(0.046, -0.0015), (0.0100, 0.0033), (0.0100, 0.0040), (0.0, 0.0040)], seg=4).rot(y=math.pi / 4)
    s = s + lobes(cyl(0.0085, 0.0085, 0.0035, seg=12, bevel_r=0.0006), 6, 0.0012, -0.00175, 0.00175).move(y=0.0045)
    for sz in (-1, 1):
        ear = prism(round_outline([(-0.003, 0.0), (0.003, 0.0), (0.0025, 0.0055), (-0.0025, 0.0055)], 0.0012, seg=2),
                    0.0015).rot(y=HPI).move(y=0.0008, z=sz * 0.020)
        s = s + ear
    return s

part('CandleLanterns__top', f_cyl(0.045, 0.045, 0.010), lantern_top, smooth=30)


def twisted_bar(h, a, turns, n=20):
    """Square iron bar of side a, twisted `turns` times along its height."""
    rings = []
    for k in range(n + 1):
        y = -h / 2 + h * k / n
        t = TAU * turns * k / n
        rings.append([(a / 2 * math.cos(t + q), y, a / 2 * math.sin(t + q))
                      for q in (math.pi / 4, 3 * math.pi / 4, 5 * math.pi / 4, 7 * math.pi / 4)])
    return loft(rings)

part('CandleLanterns__post', f_cyl(0.003, 0.003, 0.14),
     lambda: twisted_bar(0.14, 0.0042 * 1.414, 3.0, n=24), smooth=30)


def lantern_candle():
    # pillar candle: extends down onto the plinth and up into a wick; a
    # melted crater round the wick and drips down the side
    s = lathe([(0, -0.035), (0.0118, -0.035), (0.0120, -0.030), (0.0120, 0.026), (0.0112, 0.0298),
               (0.0080, 0.0282), (0.0040, 0.0268), (0.0009, 0.0270), (0.0007, 0.0345), (0, 0.0350)], seg=12)

    def drips(p):
        r = math.hypot(p.x, p.z)
        if r < 0.009:
            return p
        a = math.atan2(p.x, p.z)
        k = 0.0
        for a0, y0 in ((0.4, 0.004), (2.6, 0.014), (4.4, -0.006)):
            da = (a - a0 + math.pi) % TAU - math.pi
            if p.y > y0:
                k = max(k, math.exp(-(da / 0.28) ** 2) * min(1.0, (p.y - y0) / 0.01))
        p.x *= 1 + 0.06 * k
        p.z *= 1 + 0.06 * k
        return p
    return s.warp(drips)

part('CandleLanterns__candle', f_cyl(0.012, 0.012, 0.060), lantern_candle, smooth=60)


def lantern_handle():
    # wire bail: a half-round arch with a flattened grip loop on top, the ends
    # turned out into eyes that hook the cap ears
    R = 0.020
    pts = [(R + 0.0024, -0.0004), (R + 0.0012, 0.0012)]
    pts += [(R * math.cos(math.pi * k / 16), R * math.sin(math.pi * k / 16)) for k in range(1, 16)]
    pts += [(-R - 0.0012, 0.0012), (-R - 0.0024, -0.0004)]
    s = tube([(x, y, 0) for x, y in pts], 0.0016, seg=5)
    grip = x_axis(lathe([(0, -0.006), (0.0017, -0.006), (0.0024, -0.0035), (0.0024, 0.0035), (0.0017, 0.006),
                         (0, 0.006)], seg=6)).move(y=R - 0.0002)
    return s + grip

part('CandleLanterns__handle', torus_bounds(0.020, 0.002, math.pi), lantern_handle, smooth=60)


# ============================================================ horseshoes ==
def horseshoe():
    # forged shoe: flat web with a fuller groove round the front face, square
    # edges, heels tapering at the ends, a toe clip at the top of the arch
    sec = [(-0.0100, -0.0070), (0.0100, -0.0070), (0.0100, 0.0070), (0.0058, 0.0070), (0.0047, 0.0054),
           (0.0017, 0.0054), (0.0006, 0.0070), (-0.0100, 0.0070)]
    arc = math.pi * 1.3

    def heel(a):
        t = min(a, arc - a) / 0.5
        k = 0.86 + 0.14 * min(1.0, t)
        return (k, 0.0, 0.010 * (1 - k))
    s = ring_sec(0.055, sec, seg=26, arc=arc, rmod=heel)
    clip = prism([(-0.009, 0.0), (0.009, 0.0), (0.004, 0.0055), (-0.004, 0.0055)], 0.0022, bevel_r=0.0006)
    clip.move(y=0.0625, z=0.0060)
    s = s + clip.rot(z=arc / 2 - HPI)
    return noisy(s, 0.00025, scale=260, seed=4)

part('Horseshoes__arch', torus_bounds(0.055, 0.010, math.pi * 1.3), horseshoe, smooth=30)

# forged nail: square tapered shank, rounded head (head at +Y)
part('Horseshoes__hanger', f_cyl(0.002, 0.002, 0.010), lambda:
     lathe([(0, -0.005), (0.0009, -0.0045), (0.0014, 0.0022), (0.0021, 0.0026), (0.0022, 0.0036),
            (0.0016, 0.0048), (0, 0.005)], seg=4).rot(y=math.pi / 4)
     + lathe([(0.0, 0.0022), (0.0021, 0.0024), (0.0022, 0.0036), (0.0015, 0.0048), (0, 0.005)], seg=8), smooth=40)


# ===================================================== cast-iron kettles ==
def kettle_body():
    # built at world size (the mesh is squashed to 0.85 in y): a foot ring,
    # a full belly with a casting seam, a stepped shoulder and the lid seat
    s = lathe([(0, -0.051), (0.030, -0.051), (0.0335, -0.0475), (0.031, -0.044), (0.047, -0.035),
               (0.057, -0.020), (0.0602, -0.0055), (0.0596, -0.0035), (0.0575, 0.012), (0.049, 0.029),
               (0.041, 0.038), (0.0335, 0.0425), (0.030, 0.0468), (0.0278, 0.051), (0.0245, 0.051),
               (0.0235, 0.047), (0, 0.047)], seg=18)
    return s.scale(1, 1 / 0.85, 1).fit((-0.06, -0.06, -0.06), (0.06, 0.06, 0.06))

part('CastIronKettles__body', f_sphere(0.060), kettle_body, smooth=45)


def kettle_spout():
    # tapering spout with a flared root and a rolled, open mouth
    s = lathe([(0, -0.035), (0.0139, -0.035), (0.0124, -0.030), (0.0110, -0.020), (0.0096, 0.000),
               (0.0087, 0.024), (0.0093, 0.031), (0.0099, 0.0345), (0.0090, 0.035), (0.0066, 0.0345),
               (0.0058, 0.030), (0, 0.030)], seg=12)
    return s.warp(lambda p: Vector((p.x + 0.002 * ((p.y + 0.035) / 0.07) ** 2, p.y, p.z)))

part('CastIronKettles__spout', f_cyl(0.010, 0.014, 0.070), kettle_spout, smooth=50)

# lid: flange that drops into the body, a domed top ring
part('CastIronKettles__lid', f_cyl(0.020, 0.024, 0.008), lambda: lathe(
    [(0, -0.004), (0.0228, -0.004), (0.0240, -0.0028), (0.0236, -0.0012), (0.0212, -0.0006), (0.0196, 0.0014),
     (0.0150, 0.0032), (0.0080, 0.0040), (0, 0.0040)], seg=16), smooth=45)

# acorn finial
part('CastIronKettles__knob', f_sphere(0.005), lambda: lathe(
    [(0, -0.005), (0.0022, -0.0048), (0.0018, -0.0030), (0.0042, -0.0012), (0.0050, 0.0008), (0.0040, 0.0034),
     (0, 0.0050)], seg=7), smooth=50)


def kettle_handle():
    # bail handle: round iron wire, a swelled grip in the middle, the ends
    # turned out into hooks
    R = 0.055
    pts = [(R + 0.0035, -0.0030), (R + 0.0038, 0.0005)]
    pts += [(R * math.cos(math.pi * k / 20), R * math.sin(math.pi * k / 20)) for k in range(1, 20)]
    pts += [(-R - 0.0038, 0.0005), (-R - 0.0035, -0.0030)]
    n = len(pts)
    return sweep([(x, y, 0) for x, y in pts], circ(0.0034, 6),
                 scale=lambda t: 1 + 0.55 * math.exp(-((t - 0.5) / 0.13) ** 2))

part('CastIronKettles__handle', torus_bounds(0.055, 0.004, math.pi), kettle_handle, smooth=60)


# ========================================================= wagon wheels ==
def ring_sec_a(R, sec, angles, rmod=None):
    """Full ring with a closed section at explicit angles (XY plane)."""
    m = len(sec)
    v, f = [], []
    n = len(angles)
    for a in angles:
        k, dzz, dR = rmod(a) if rmod else (1.0, 0.0, 0.0)
        for dr, dz in sec:
            rr = R + dR + dr * k
            v.append((rr * math.cos(a), rr * math.sin(a), dz * k + dzz))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    return fixw(Shape(v, f))


def wheel_tyre():
    # iron tyre: a flat band shrunk onto the felloes, eased edges, a little
    # hammered unevenness
    sec = [(-0.0045, -0.0090), (0.0085, -0.0090), (0.0100, -0.0070), (0.0100, 0.0070), (0.0085, 0.0090),
           (-0.0045, 0.0090)]
    return noisy(ring_sec(0.140, sec, seg=48), 0.0003, scale=90, seed=6)

part('WagonWheels__rim', f_torus(0.140, 0.010), wheel_tyre, smooth=35)


def wheel_felloes():
    # four wooden felloes with chamfered edges; the joints between them show
    # as V seams
    sec = [(-0.0140, -0.0115), (-0.0115, -0.0140), (0.0080, -0.0140), (0.0100, -0.0120), (0.0100, 0.0120),
           (0.0080, 0.0140), (-0.0115, 0.0140), (-0.0140, 0.0115)]
    joints = [math.pi / 8 + k * math.pi / 2 for k in range(4)]
    angles = sorted(set([TAU * i / 48 for i in range(48)] + [j + d for j in joints for d in (-0.012, 0.0, 0.012)]))
    angles = [a for i, a in enumerate(angles) if i == 0 or a - angles[i - 1] > 0.004]

    def seam(a):
        d = min(abs((a - j + math.pi) % TAU - math.pi) for j in joints)
        return (0.9 if d < 0.002 else 1.0, 0.0, 0.0)
    return ring_sec_a(0.128, sec, angles, seam)

part('WagonWheels__band', f_torus(0.128, 0.014), wheel_felloes, smooth=35)


def wheel_hub():
    # nave (axis towards the room): barrel with iron hub bands, a hex axle nut
    s = lathe([(0, -0.015), (0.0205, -0.015), (0.0215, -0.0095), (0.0245, -0.0075), (0.0250, 0.0010),
               (0.0235, 0.0065), (0.0215, 0.0085), (0.0195, 0.0105), (0, 0.0105)], seg=14)
    nut = cyl(0.0095, 0.0095, 0.0045, seg=6, bevel_r=0.0008).move(y=0.0125)
    return s + nut

part('WagonWheels__hub', f_cyl(0.025, 0.025, 0.030), wheel_hub, smooth=40)


def wheel_spoke():
    # oval spoke, fullest in the middle, with square tenon shoulders at both
    # ends where it enters the nave and the felloe
    rings = []
    for y, k in ((-0.047, 0.78), (-0.036, 0.86), (-0.018, 0.96), (0.0, 1.0), (0.018, 0.96), (0.036, 0.86),
                 (0.047, 0.78)):
        rings.append([(x * k, y, z * k) for x, z in sell(0.0032, 0.0040, 10, 2.0)])
    body = loft(rings)
    ten = lambda sy: rbox(0.0056, 0.0105, 0.0064, r=0.0008, seg=1).move(y=sy * 0.05)
    return body + ten(-1) + ten(1)

part('WagonWheels__spoke', f_box(0.008, 0.115, 0.008), wheel_spoke, smooth=50)


# =============================================================== bellows ==
def pear(w, d, n=22):
    """Bellows board outline in (x, z): broad at +X, narrowing to the nozzle
    at -X."""
    out = []
    for i in range(n):
        t = TAU * i / n
        c, s = math.cos(t), math.sin(t)
        x = w / 2 * c
        u = (c + 1) / 2
        hw = d / 2 * (0.40 + 0.60 * math.sin(math.pi / 2 * u) ** 0.8)
        out.append((x, hw * s))
    return out


def bellows_paddle():
    # pear-shaped board with an eased edge and a carved oval boss on top
    pts = pear(0.10, 0.06)
    board = prism(pts, 0.008, bevel_r=0.0012).rot(x=-HPI)
    boss = lathe([(0, -0.0010), (0.0110, -0.0010), (0.0110, 0.0005), (0.0085, 0.0012), (0, 0.0012)], seg=12)
    boss = boss.scale(1.35, 1, 1).move(x=0.012, y=0.0045)
    return board + boss

part('Bellows__paddle', f_box(0.10, 0.008, 0.06), bellows_paddle, smooth=35)


def bellows_pleat():
    # one leather fold: bulging out round the middle, pinched at the boards
    pts = pear(0.095, 0.055, n=24)
    s = stack(pts, [(0.004, -0.003), (0.0012, -0.0018), (0.0, 0.0), (0.0012, 0.0018), (0.004, 0.003)])
    return s.rot(x=-HPI)

part('Bellows__pleat', f_box(0.095, 0.006, 0.055), bellows_pleat, smooth=60)

# turned handle: ball end (frame -Y, the outer end), grip swell, a collar
# and a neck where it joins the board
part('Bellows__handle', f_cyl(0.008, 0.008, 0.10), lambda: lathe(
    [(0, -0.05), (0.0040, -0.0495), (0.0075, -0.046), (0.0078, -0.042), (0.0060, -0.038), (0.0042, -0.034),
     (0.0062, -0.020), (0.0068, 0.000), (0.0058, 0.020), (0.0048, 0.035), (0.0060, 0.038), (0.0060, 0.043),
     (0.0045, 0.046), (0, 0.05)], seg=9), smooth=55)

# brass nozzle: a ferrule at the board (+Y), a tapered body and a beaded,
# bored mouth at -Y
part('Bellows__nozzle', f_cyl(0.006, 0.010, 0.024), lambda: lathe(
    [(0, -0.006), (0.0030, -0.0105), (0.0045, -0.012), (0.0085, -0.012), (0.0100, -0.0105), (0.0098, -0.0085),
     (0.0078, -0.0075), (0.0068, -0.002), (0.0060, 0.006), (0.0065, 0.0075), (0.0065, 0.0095), (0.0052, 0.0105),
     (0.0052, 0.012), (0, 0.012)], seg=10), smooth=45)


# ===================================================== grandfather clocks ==
def ystack(pts_xz, prof):
    """stack() lying flat: outline in (x, z), prof = [(inset, y), ...]."""
    return stack([(x, -z) for x, z in pts_xz], prof).rot(x=-HPI)


def gfc_base():
    # plinth: case with a fielded front panel, a wider skirting with a cut
    # arch between two bracket feet at the front
    s = rbox(0.316, 0.292, 0.176, r=0.003, seg=1).move(y=0.014)
    field = stack(rounded_rect(0.22, 0.19, 0.004, seg=1),
                  [(0.0, 0.086), (0.0, 0.0895), (0.014, 0.0935), (0.017, 0.094)]).move(y=0.02)
    skirt = rbox(0.320, 0.030, 0.180, r=0.003, seg=1).move(y=-0.145)
    arch = prism([(-0.105, -0.2), (0.105, -0.2), (0.105, -0.152), (0.075, -0.145), (0.0, -0.142), (-0.075, -0.145),
                  (-0.105, -0.152)], 0.03).move(z=0.095)
    skirt = cut(skirt, arch)
    cap = rbox(0.320, 0.010, 0.180, r=0.003, seg=1).move(y=-0.126)
    return s + field + skirt + cap

part('GrandfatherClocks__base', f_box(0.32, 0.32, 0.18), gfc_base, smooth=30)

# ogee moulding stepping in from the plinth to the trunk
part('GrandfatherClocks__baseTrim', f_box(0.33, 0.020, 0.19), lambda: ystack(
    rounded_rect(0.33, 0.19, 0.002, seg=1),
    [(0.0, -0.010), (0.0, -0.006), (0.0025, -0.0035), (0.0058, -0.0015), (0.0078, 0.0015), (0.0085, 0.0045),
     (0.0120, 0.0065), (0.0160, 0.0070), (0.0180, 0.010)]), smooth=40)


def gfc_waist():
    # the trunk is open behind the glass: a recess 5.5 cm deep so the
    # pendulum (hung 3 cm behind the front) shows; stiles, rails, a back
    # board and a beaded door surround
    W, H, D = 0.272, 0.90, 0.18
    ow, oh = 0.202, 0.78
    back = rbox(W, H, 0.125, r=0.003, seg=1).move(z=-D / 2 + 0.0625)
    st = lambda sx: rbox((W - ow) / 2, H, D, r=0.003, seg=1).move(x=sx * (ow / 2 + (W - ow) / 4))
    rl = lambda sy: rbox(ow + 0.004, (H - oh) / 2, D, r=0.003, seg=1).move(y=sy * (oh / 2 + (H - oh) / 4))
    bead = stack(rounded_rect(ow, oh, 0.002, seg=1), [(-0.011, 0.0895), (-0.011, 0.0915), (-0.006, 0.0948),
                                                     (-0.001, 0.0925), (0.0, 0.0895)], loop=True)
    return back + st(-1) + st(1) + rl(-1) + rl(1) + bead

part('GrandfatherClocks__waist', f_box(0.272, 0.90, 0.18), gfc_waist, smooth=30)


def gfc_head():
    # hood: case with turned pillars at the front corners, a round moulded
    # dial surround (the dial sits 3 cm above the hood's centre) and a
    # break-arch pediment set back above the cornice line
    W, H, D = 0.304, 0.63, 0.18
    s = rbox(W, 0.585, D, r=0.003, seg=1).move(y=-0.0225)
    ped = prism(round_outline([(-0.13, 0.26), (0.13, 0.26), (0.13, 0.285), (0.06, 0.296), (0.035, 0.312),
                               (0.0, 0.315), (-0.035, 0.312), (-0.06, 0.296), (-0.13, 0.285)], 0.004, seg=2),
                0.15, bevel_r=0.002).move(z=-0.012)
    ring = ring_sec(0.110, [(-0.007, -0.004), (0.009, -0.004), (0.009, 0.0), (0.006, 0.0045), (0.0015, 0.0058),
                            (-0.004, 0.0042), (-0.007, 0.001)], seg=32).move(y=0.0315, z=0.090)
    door = stack(rounded_rect(0.262, 0.30, 0.004, seg=1), [(-0.004, 0.0895), (-0.004, 0.0915), (0.0, 0.0935),
                                                          (0.006, 0.0935), (0.009, 0.0915), (0.009, 0.0895)],
                 loop=True).move(y=0.0315)
    pil = lathe([(0, -0.225), (0.0125, -0.225), (0.0125, -0.212), (0.0095, -0.205), (0.0085, 0.195),
                 (0.0105, 0.205), (0.0125, 0.215), (0.0125, 0.225), (0, 0.225)], seg=8)
    pils = pil.copy().move(x=-0.137, y=0.0, z=0.078) + pil.copy().move(x=0.137, y=0.0, z=0.078)
    return s + ped + ring + door + pils

part('GrandfatherClocks__head', f_box(0.304, 0.63, 0.18), gfc_head, smooth=30)

# cornice: a cove flaring out under a flat top edge
part('GrandfatherClocks__crown', f_box(0.318, 0.030, 0.19), lambda: ystack(
    rounded_rect(0.318, 0.19, 0.002, seg=1),
    [(0.0095, -0.015), (0.0085, -0.011), (0.0060, -0.0075), (0.0032, -0.0045), (0.0012, -0.001), (0.0, 0.003),
     (0.0, 0.0085), (0.0015, 0.011), (0.0045, 0.012), (0.0055, 0.015)]), smooth=40)

# brass finial: a waisted socket, an urn, a flame spire
part('GrandfatherClocks__finial', f_cone(0.014, 0.040), lambda: lathe(
    [(0, -0.020), (0.0085, -0.020), (0.0085, -0.017), (0.0050, -0.015), (0.0105, -0.0105), (0.0140, -0.0055),
     (0.0120, -0.0005), (0.0055, 0.0020), (0.0060, 0.0045), (0.0035, 0.0070), (0.0028, 0.0140), (0.0012, 0.0180),
     (0, 0.020)], seg=10), smooth=50)


def gfc_rod():
    # flat brass pendulum rod with a suspension block at the top (+Y)
    rod = rbox(0.0070, 0.62, 0.0022, r=0.0007, seg=1).move(y=-0.015)
    blk = rbox(0.0080, 0.024, 0.0080, r=0.0012, seg=1).move(y=0.313)
    spr = rbox(0.0045, 0.010, 0.0012, r=0.0004, seg=1).move(y=0.296)
    return rod + blk + spr

part('GrandfatherClocks__rod', f_cyl(0.004, 0.004, 0.65), gfc_rod, smooth=30)

# lenticular bob: lens-shaped both sides with a turned rim
part('GrandfatherClocks__bob', f_cyl(0.050, 0.050, 0.012), lambda: lathe(
    [(0, -0.006), (0.020, -0.0054), (0.038, -0.0036), (0.0475, -0.0016), (0.050, 0.0), (0.0475, 0.0016),
     (0.038, 0.0036), (0.020, 0.0054), (0, 0.006)], seg=32), smooth=60)

# dial bezel: a flat brass ring with a raised bead (the placer lays it flat)
part('GrandfatherClocks__faceRim', f_torus(0.106, 0.008), lambda: ring_sec(0.106, [
    (-0.008, -0.004), (0.008, -0.004), (0.008, 0.0005), (0.0055, 0.0032), (0.0015, 0.0040), (-0.0035, 0.0025),
    (-0.008, 0.0012)], seg=36), smooth=50)

# hands: the placer's boxes run along Z (out of the dial), so the pierced
# pointer is modelled flat in X-Z with its tip at +Z
part('GrandfatherClocks__hour', f_box(0.008, 0.002, 0.045),
     lambda: clock_hand(0.045, -0.0225, 0.0125, 0.008, 0.0016, spade=0.4).rot(x=-HPI), smooth=40)
part('GrandfatherClocks__minute', f_box(0.005, 0.002, 0.068),
     lambda: clock_hand(0.068, -0.034, 0.024, 0.005, 0.0016, spade=0.25).rot(x=-HPI), smooth=40)


# ========================================================= gentle rockers ==
def up(s):
    """pillow() builds its face towards +Z; turn it to face +Y."""
    return s.rot(x=-HPI)


def rocker_seat():
    # saddled plank seat: dished in the middle, the front edge rolled over,
    # eased all round
    def top(u, v):
        dish = 0.0045 * max(0.0, 1 - u * u) * max(0.0, 1 - ((v + 0.15) / 1.0) ** 2)
        return 0.004 + 0.014 * bullnose(u, 10) * bullnose(v, 8) - dish
    return up(pillow(0.36, 0.32, 0.036, top, nx=16, ny=12))

part('GentleRockers__seat', f_box(0.36, 0.036, 0.32), rocker_seat, smooth=50)

# turned leg: a ring below the seat, a long taper, a foot ring
part('GentleRockers__leg', f_cyl(0.010, 0.010, 0.28), lambda: lathe(
    [(0, -0.14), (0.0080, -0.14), (0.0086, -0.136), (0.0070, -0.128), (0.0076, -0.100), (0.0092, -0.060),
     (0.0100, -0.020), (0.0082, 0.012), (0.0100, 0.030), (0.0100, 0.040), (0.0078, 0.052), (0.0086, 0.110),
     (0.0092, 0.136), (0, 0.14)], seg=10), smooth=55)


def rocker_back():
    # spindle back: two stiles, an arched splat rail at the top and a bottom
    # rail; the five turned spindles (separate) run between the rails
    stile = lambda sx: taper(rbox(0.028, 0.42, 0.020, r=0.004, seg=2), top=0.85, along='y', axes='x').move(
        x=sx * 0.166)
    crest = prism(round_outline([(-0.155, 0.210), (0.155, 0.210), (0.155, 0.150), (0.10, 0.160), (0.0, 0.170),
                                 (-0.10, 0.160), (-0.155, 0.150)], 0.006, seg=2), 0.018, bevel_r=0.0025)
    low = rbox(0.31, 0.034, 0.018, r=0.004, seg=2).move(y=-0.188)
    return stile(-1) + stile(1) + crest + low

part('GentleRockers__back', f_box(0.36, 0.42, 0.020), rocker_back, smooth=40)

# spindle: swelling in the middle, a bead a third of the way up
part('GentleRockers__slat', f_cyl(0.006, 0.006, 0.40), lambda: lathe(
    [(0, -0.20), (0.0036, -0.20), (0.0040, -0.16), (0.0055, -0.07), (0.0060, 0.0), (0.0052, 0.06),
     (0.0060, 0.075), (0.0060, 0.085), (0.0045, 0.10), (0.0038, 0.16), (0.0034, 0.20), (0, 0.20)], seg=8),
     smooth=55)


def rocker_crest():
    # crest rail: rises in a shallow arch to the middle, rolled ears at the
    # ends, bowed back a little
    half = [(0.0, 0.012), (0.08, 0.010), (0.15, 0.004), (0.172, 0.006), (0.186, 0.011), (0.190, 0.005),
            (0.188, -0.006), (0.180, -0.012), (0.0, -0.012)]
    out = half + [(-x, y) for x, y in reversed(half[1:-1])]
    s = prism(round_outline(out, 0.003, seg=2), 0.018, bevel_r=0.003)
    return s.warp(lambda p: Vector((p.x, p.y, p.z - 0.004 * (p.x / 0.19) ** 2 + 0.002)))

part('GentleRockers__topRail', f_box(0.38, 0.024, 0.020), rocker_crest, smooth=40)


def rocker_arm():
    # arm rest: slim at the back, widening to a rounded hand-hold at the
    # front (+Z), eased edges
    pts = [(-0.0085, -0.112), (0.0085, -0.112), (0.0095, 0.02), (0.012, 0.075), (0.012, 0.100), (0.008, 0.112),
           (-0.008, 0.112), (-0.012, 0.100), (-0.012, 0.075), (-0.0095, 0.02)]
    s = prism(round_outline([(x, -z) for x, z in pts], 0.004, seg=2), 0.020, bevel_r=0.004).rot(x=-HPI)
    return s

part('GentleRockers__arm', f_box(0.024, 0.020, 0.224), rocker_arm, smooth=40)

# arm post: a small baluster
part('GentleRockers__armSupport', f_cyl(0.008, 0.008, 0.11), lambda: lathe(
    [(0, -0.055), (0.0070, -0.055), (0.0070, -0.048), (0.0055, -0.044), (0.0080, -0.020), (0.0072, 0.010),
     (0.0045, 0.030), (0.0060, 0.040), (0.0060, 0.047), (0.0048, 0.055), (0, 0.055)], seg=10), smooth=55)


def rocker_runner():
    # rocker: an upright board section, eased edges, the ends tapering to
    # rounded tips
    sec = [(-0.008, -0.006), (0.008, -0.006), (0.008, 0.006), (-0.008, 0.006)]
    sec = round_outline(sec, 0.003, seg=2)
    arc = math.pi * 0.75

    def tip(a):
        t = min(a, arc - a) / 0.12
        k = 0.55 + 0.45 * min(1.0, t) ** 0.5
        return (k, 0.0, 0.008 * (1 - k))
    return ring_sec(0.30, sec, seg=36, arc=arc, rmod=tip)

part('GentleRockers__runner', torus_bounds(0.30, 0.008, math.pi * 0.75), rocker_runner, smooth=40)


def cushion_top(u, v, buttons):
    h = 0.004 + 0.028 * bullnose(u, 5) * bullnose(v, 5)
    for bu, bv in buttons:
        d2 = ((u - bu) / 0.16) ** 2 + ((v - bv) / 0.18) ** 2
        h -= 0.006 * math.exp(-d2 * 2.5)
    return h - 0.016

part('GentleRockers__cushion', f_box(0.34, 0.032, 0.30), lambda: up(pillow(
    0.34, 0.30, 0.032, lambda u, v: cushion_top(u, v, [(-0.4, -0.4), (0.4, -0.4), (-0.4, 0.4), (0.4, 0.4)]),
    nx=14, ny=12)), smooth=180)
part('GentleRockers__backCushion', f_box(0.33, 0.168, 0.024), lambda: pillow(
    0.33, 0.168, 0.024, lambda u, v: 0.75 * cushion_top(u, v, [(-0.45, 0.0), (0.45, 0.0)]) + 0.004,
    nx=14, ny=8), smooth=180)


# ========================================================== pet portraits ==
# gilt frame block: an outer bead, a cove, an inner sight bead, a rabbet the
# portrait sits in (the painting is 1 mm in front of the block's face)
part('PetPortraits__frame', f_box(0.22, 0.24, 0.014), lambda: stack(
    rounded_rect(0.22, 0.24, 0.0015, seg=1),
    [(0.0, -0.007), (0.0, 0.004), (0.0012, 0.0085), (0.0030, 0.0110), (0.0045, 0.0095), (0.0060, 0.0082),
     (0.0072, 0.0092), (0.0082, 0.0096), (0.0092, 0.0078), (0.0099, 0.0062), (0.0110, 0.0062)]), smooth=40)


# ================================================== ornate music boxes ==
def mbox_body():
    # casket: eased box with a raised, bevelled panel on every side
    s = rbox(0.136, 0.055, 0.096, r=0.003, seg=2)
    for sx in (-1, 1):
        s = s + rbox(0.004, 0.030, 0.066, r=0.0012, seg=1).move(x=sx * 0.068)
    for sz in (-1, 1):
        s = s + rbox(0.106, 0.030, 0.004, r=0.0012, seg=1).move(z=sz * 0.048)
    return s

part('OrnateMusicBoxes__bottom', f_box(0.14, 0.055, 0.10), mbox_body, smooth=35)

# gilt band round the rim (and the foot): a beaded frame, open in the middle
# so the velvet shows
part('OrnateMusicBoxes__trim', f_box(0.144, 0.003, 0.104), lambda: ystack(
    rounded_rect(0.144, 0.104, 0.002, seg=1),
    [(0.0055, -0.0015), (0.0, -0.0015), (0.0, 0.0003), (0.0012, 0.0015), (0.0044, 0.0015), (0.0055, 0.0003)]
).warp(lambda p: p), smooth=40)


def mbox_lid():
    # lid as an inverted tray: a thick top with a raised, bevelled panel
    # outside (+Y) and a well inside (-Y) where the mirror is fixed
    top = rbox(0.140, 0.0075, 0.100, r=0.0025, seg=2).move(y=0.00585)
    panel = ystack(rounded_rect(0.110, 0.070, 0.006, seg=2),
                   [(0.0, 0.0090), (0.0, 0.0100), (0.008, 0.0118), (0.010, 0.0120)])
    wall = lambda w, d, x, z: rbox(w, 0.0118, d, r=0.0012, seg=1).move(x=x, y=-0.0037, z=z)
    walls = wall(0.140, 0.006, 0, 0.047) + wall(0.140, 0.006, 0, -0.047) + \
        wall(0.006, 0.092, 0.067, 0) + wall(0.006, 0.092, -0.067, 0)
    return top + panel + walls

part('OrnateMusicBoxes__lid', f_box(0.14, 0.01925, 0.10), mbox_lid, smooth=35)

# gilt rosette stud
part('OrnateMusicBoxes__orn', f_sphere(0.003), lambda: lobes(lathe(
    [(0, -0.003), (0.0030, -0.0015), (0.0030, 0.0005), (0.0018, 0.0022), (0, 0.003)], seg=8),
    4, 0.0008, -0.0015, 0.0022), smooth=50)


def mbox_lining():
    # velvet pad: soft rolled edges, quilted into four puffs
    def top(u, v):
        h = 0.0011 + 0.0012 * bullnose(u, 6) * bullnose(v, 6)
        h -= 0.0006 * (math.exp(-(u / 0.12) ** 2) + math.exp(-(v / 0.14) ** 2)) * bullnose(u, 6) * bullnose(v, 6)
        return h - 0.0003
    return up(pillow(0.13, 0.09, 0.003, top, nx=14, ny=10))

part('OrnateMusicBoxes__lining', f_box(0.13, 0.003, 0.09), mbox_lining, smooth=70)


def spring_post():
    # a coil spring under the dancer's pin
    pts = []
    turns, n = 3.5, 4
    for k in range(int(turns * n) + 1):
        a = TAU * k / n
        y = -0.0095 + 0.011 * k / (turns * n)
        pts.append((0.0010 * math.cos(a), y, 0.0010 * math.sin(a)))
    pts.append((0.0, 0.0035, 0.0))
    pts.append((0.0, 0.0100, 0.0))
    return tube(pts, 0.00035, seg=3)

part('OrnateMusicBoxes__post', f_cyl(0.001, 0.001, 0.020), spring_post, smooth=60)

# dancer's bodice: hips, waist, bust, shoulders, neck
part('OrnateMusicBoxes__body', f_cyl(0.003, 0.004, 0.014), lambda: lathe(
    [(0, -0.007), (0.0038, -0.0068), (0.0026, -0.0030), (0.0034, 0.0010), (0.0036, 0.0040), (0.0020, 0.0056),
     (0.0011, 0.0062), (0, 0.0070)], seg=7).scale(1, 1, 0.8).fit((-0.004, -0.007, -0.004), (0.004, 0.007, 0.004)),
     smooth=60)


def dancer_head():
    h = sphere(0.0036, seg=8, rings=5).move(y=-0.0004)
    bun = sphere(0.0013, seg=6, rings=4).move(y=0.0026, z=-0.0014)
    return (h + bun).fit((-0.004, -0.004, -0.004), (0.004, 0.004, 0.004))

part('OrnateMusicBoxes__head', f_sphere(0.004), dancer_head, smooth=70)


def tutu():
    # a stiff tutu: a flat frilled disc gathered up to the waist, its edge
    # waving up and down
    s = lathe([(0, -0.0016), (0.0095, -0.0026), (0.0100, -0.0020), (0.0060, 0.0002), (0.0034, 0.0030),
               (0, 0.0030)], seg=10)

    def frill(p):
        r = math.hypot(p.x, p.z)
        if r > 0.004:
            p.y += 0.0007 * math.sin(5 * math.atan2(p.x, p.z)) * (r / 0.01)
        return p
    return s.warp(frill)

part('OrnateMusicBoxes__skirt', f_cone(0.010, 0.006), tutu, smooth=60)

# slender arm with a hand
part('OrnateMusicBoxes__arm', f_cyl(0.001, 0.001, 0.010), lambda: lathe(
    [(0, -0.005), (0.0010, -0.0045), (0.0008, 0.0010), (0.0006, 0.0034), (0.0008, 0.0042), (0, 0.005)], seg=5),
     smooth=60)

# winding key: an escutcheon collar at the box (+Y) and a slim shaft
part('OrnateMusicBoxes__keyShaft', f_cyl(0.003, 0.003, 0.015), lambda: lathe(
    [(0, -0.0075), (0.0017, -0.0075), (0.0017, 0.0035), (0.0030, 0.0050), (0.0030, 0.0075), (0, 0.0075)], seg=8),
     smooth=45)


def key_wing(long_axis):
    # one pair of butterfly-bow wings: round lobes on a waisted bar
    pts = []
    for i in range(5):
        a = math.pi * (0.7 - 1.4 * i / 4)
        pts.append((0.0055 + 0.0015 * math.cos(a), 0.0015 * math.sin(a)))
    ring = [(0.0, 0.0008)] + pts + [(0.0, -0.0008)] + [(-x, -y) for x, y in pts]
    s = prism(ring, 0.0027)                             # outline in XY, thickness Z
    s = s.rot(y=HPI)                                    # thickness along X
    s = s.rot(x=HPI) if long_axis == 'y' else s        # wings along Y or Z
    return s


part('OrnateMusicBoxes__keyBar', f_box(0.003, 0.014, 0.003), lambda: key_wing('y'), smooth=40)
part('OrnateMusicBoxes__keyBar2', f_box(0.003, 0.003, 0.014), lambda: key_wing('z'), smooth=40)


# ========================================================= record stacks ==
def sleeve():
    # card sleeve: flat printed face (the labels sit 0.5 mm in front of it),
    # edges softened and worn, a little ring wear on the back where the disc
    # presses through
    def front(u, v):
        return 0.0006 + 0.0019 * bullnose(u, 14) * bullnose(v, 14)
    s = pillow(0.14, 0.1372, 0.005, front, nx=10, ny=10)

    def ring(p):
        if p.z < -0.002:
            r = math.hypot(p.x, p.y) / 0.067
            p.z -= 0.0004 * math.exp(-((r - 0.93) / 0.12) ** 2)
        return p
    return noisy(s.warp(ring), 0.00012, scale=80, seed=9)

part('RecordStacks__sleeve', f_box(0.14, 0.1372, 0.005), sleeve, smooth=50)


def record(R, t, label_r, hole=0.0036, seg=28):
    """Vinyl disc on +Y: raised rim, grooved band a touch thinner, flat
    label area, spindle hole (closed profile, revolved)."""
    h = t / 2
    top = [(hole, h * 0.95), (label_r, h * 0.95), (label_r + 0.0015, h * 0.70), (R * 0.97, h * 0.72), (R, h)]
    prof = top + [(r, -y) for r, y in reversed(top)]
    return ring_lathe(prof, seg=seg)

part('RecordStacks__record', f_cyl(0.0672, 0.0672, 0.003), lambda: record(0.0672, 0.003, 0.0224), smooth=40)


# ============================================================ turntables ==
def tt_plinth():
    # wooden plinth: eased box, a raised lip round the top deck, a chamfered
    # front and four feet
    s = rbox(0.360, 0.054, 0.280, r=0.006, seg=2).move(y=-0.002)
    lip = ystack(rounded_rect(0.356, 0.276, 0.006, seg=2), [(0.0, 0.024), (0.0, 0.029), (0.0015, 0.030),
                                                           (0.006, 0.030), (0.007, 0.0285), (0.007, 0.024)])
    feet = Shape()
    for sx in (-1, 1):
        for sz in (-1, 1):
            feet = feet + cyl(0.012, 0.014, 0.006, seg=10, bevel_r=0.0015).move(x=sx * 0.15, y=-0.027, z=sz * 0.11)
    return s + lip + feet

part('Turntables__base', f_box(0.36, 0.06, 0.28), tt_plinth, smooth=35)

# platter: bevelled top edge, a rim ring of strobe dots round the side
part('Turntables__platter', f_cyl(0.12, 0.12, 0.008), lambda: lobes(lathe(
    [(0, -0.004), (0.114, -0.004), (0.1185, -0.0032), (0.1200, -0.0012), (0.1200, 0.0012), (0.1185, 0.0030),
     (0.1150, 0.0040), (0, 0.0040)], seg=40), 20, 0.0012, -0.0012, 0.0012), smooth=35)

part('Turntables__vinyl', f_cyl(0.114, 0.114, 0.002), lambda: record(0.114, 0.002, 0.036), smooth=40)

# paper label: a thin disc with an eased edge round the spindle hole
part('Turntables__label', f_cyl(0.036, 0.036, 0.001), lambda: ring_lathe(
    [(0.0036, -0.0005), (0.0355, -0.0005), (0.0360, 0.0), (0.0355, 0.0005), (0.0036, 0.0005)], seg=24), smooth=40)

# spindle: a collar and a rounded, slightly tapered pin
part('Turntables__spindle', f_cyl(0.002, 0.002, 0.010), lambda: lathe(
    [(0, -0.005), (0.0020, -0.005), (0.0020, -0.0035), (0.0017, -0.0028), (0.0016, 0.0035), (0.0010, 0.0048),
     (0, 0.005)], seg=8), smooth=50)

# tone-arm pillar: flange, pillar, bearing housing with a cap
part('Turntables__armBase', f_cyl(0.014, 0.014, 0.020), lambda: lathe(
    [(0, -0.010), (0.0140, -0.010), (0.0140, -0.0075), (0.0110, -0.0060), (0.0065, -0.0050), (0.0060, 0.0030),
     (0.0085, 0.0045), (0.0085, 0.0085), (0.0060, 0.0100), (0, 0.0100)], seg=14), smooth=40)


def tt_arm():
    # arm tube: a bearing collar at the pivot (-Y), a slim tube, a slight
    # crank and a headshell collar at the tip (+Y)
    tubeS = tube([(0, -0.072, 0), (0, 0.040, 0), (0.0012, 0.060, 0), (0.0016, 0.074, 0)], 0.0021, seg=8, smooth=1)
    c1 = lathe([(0, -0.080), (0.0030, -0.080), (0.0030, -0.070), (0.0022, -0.068), (0, -0.068)], seg=8)
    c2 = lathe([(0, 0.072), (0.0026, 0.072), (0.0026, 0.080), (0, 0.080)], seg=8).move(x=0.0016)
    return tubeS + c1 + c2

part('Turntables__arm', f_cyl(0.003, 0.003, 0.16), tt_arm, smooth=50)


def tt_head():
    # headshell with a finger lift (+Z) over a tapered cartridge body
    shell = prism(round_outline([(-0.008, -0.0045), (0.007, -0.0045), (0.008, 0.0040), (-0.008, 0.0040)],
                                0.0012, seg=1), 0.0012).rot(x=-HPI).move(y=0.0022)
    lift = box(0.0030, 0.0009, 0.0016).move(x=0.003, y=0.0024, z=0.0042)
    cart = taper(rbox(0.012, 0.0045, 0.0070, r=0.0006, seg=1), top=1.0, along='x', axes='y').move(x=-0.0005, y=-0.0006)
    return shell + lift + cart

part('Turntables__cartridge', f_box(0.016, 0.006, 0.010), tt_head, smooth=35)

# counterweight: knurled drum with a scale ring and a stub on top
part('Turntables__weight', f_cyl(0.010, 0.010, 0.016), lambda: lobes(lathe(
    [(0, -0.008), (0.0092, -0.008), (0.0100, -0.0068), (0.0100, 0.0020), (0.0088, 0.0030), (0.0088, 0.0050),
     (0.0060, 0.0065), (0.0030, 0.0068), (0.0030, 0.0080), (0, 0.0080)], seg=16), 8, 0.0006, -0.0068, 0.0020),
     smooth=40)


def tt_knob():
    # knurled knob, its face towards the room (+Y), a pointer ridge
    s = lobes(lathe([(0, -0.003), (0.0080, -0.003), (0.0080, 0.0015), (0.0065, 0.0030), (0, 0.0030)], seg=12),
              6, 0.0007, -0.003, 0.0015)
    ptr = box(0.0012, 0.0008, 0.0055).move(y=0.0030, z=0.0022)
    return s + ptr

part('Turntables__knob', f_cyl(0.008, 0.008, 0.006), tt_knob, smooth=40)


# ========================================================== floor globes ==
# tripod leg: a ring collar at the top, a slender turned shaft, a pad foot
part('FloorGlobes__leg', f_cyl(0.006, 0.008, 0.34), lambda: lathe(
    [(0, -0.170), (0.0080, -0.170), (0.0080, -0.164), (0.0056, -0.158), (0.0050, -0.140), (0.0062, -0.060),
     (0.0058, 0.080), (0.0046, 0.140), (0.0060, 0.150), (0.0060, 0.160), (0.0050, 0.166), (0, 0.170)], seg=8),
     smooth=55)


def globe_tripod_base():
    # round plinth with three lobes reaching out under the tripod feet
    # (the legs stand at 0, 120 and 240 degrees)
    pts = []
    n = 48
    for i in range(n):
        a = TAU * i / n
        lobe = max(math.exp(-(((a - b + math.pi) % TAU - math.pi) / 0.32) ** 2) for b in (0, TAU / 3, 2 * TAU / 3))
        r = 0.128 + 0.022 * lobe
        pts.append((r * math.cos(a), -r * math.sin(a)))
    s = stack(pts, [(0.0, -0.005), (0.0, 0.0005), (0.0025, 0.0035), (0.0060, 0.0050)])
    return s.rot(x=-HPI)

part('FloorGlobes__tripodBase', f_cyl(0.13, 0.15, 0.010), globe_tripod_base, smooth=40)


def globe_square_base():
    # square plinth (the 4-sided cylinder has its corners on the axes; the
    # placer turns it 45 degrees): tapered sides, a cove and a top fillet
    sq = [(0.15, 0.0), (0.0, 0.15), (-0.15, 0.0), (0.0, -0.15)]
    sq = round_outline(sq, 0.006, seg=2)
    s = stack(sq, [(0.0, -0.007), (0.0, -0.004), (0.004, -0.0005), (0.0065, 0.0005), (0.0085, 0.0035),
                   (0.0135, 0.007)])
    return s.rot(x=-HPI)

part('FloorGlobes__squareBase', f_cyl(0.14, 0.15, 0.014), globe_square_base, smooth=35)


def baluster(h, rt, rb, seg=12):
    """Turned column: a base collar, a vase swelling low, a long taper, rings
    and a cup at the top."""
    y0, y1 = -h / 2, h / 2
    f = lambda t: y0 + (y1 - y0) * t
    return lathe([(0, y0), (rb, y0), (rb, f(0.03)), (rb * 0.72, f(0.05)), (rb * 0.95, f(0.14)), (rb * 0.82, f(0.24)),
                  (rt * 0.62, f(0.36)), (rt * 0.58, f(0.80)), (rt * 0.95, f(0.84)), (rt * 0.62, f(0.88)),
                  (rt * 0.60, f(0.93)), (rt, f(0.975)), (rt, y1), (0, y1)], seg=seg)

part('FloorGlobes__column', f_cyl(0.012, 0.014, 0.32), lambda: baluster(0.32, 0.012, 0.014), smooth=55)


def meridian(R, r, arc, seg=40):
    # flat brass band with a raised edge on each face (a graduated ring)
    sec = [(-r, -0.0025), (r, -0.0025), (r, 0.0025), (r * 0.6, 0.0030), (-r * 0.6, 0.0030), (-r, 0.0025)]
    return ring_sec(R, sec, seg=seg, arc=arc)

part('FloorGlobes__yoke', torus_bounds(0.152, 0.005, math.pi * 1.15),
     lambda: meridian(0.152, 0.005, math.pi * 1.15), smooth=35)

# axis: a brass rod with a pivot knob at each end (shared by the moon globe)
part('FloorGlobes__axis', f_cyl(0.002, 0.002, 0.336), lambda: lathe(
    [(0, -0.168), (0.0015, -0.168), (0.0024, -0.1655), (0.0020, -0.162), (0.0013, -0.158), (0.0013, 0.158),
     (0.0020, 0.162), (0.0024, 0.1655), (0.0015, 0.168), (0, 0.168)], seg=6), smooth=55)


def earth_globe():
    # smooth paper-covered sphere with the hemisphere seam at the equator
    # (land patches are decals 0.7 mm proud: nothing here rises above them)
    R = 0.14
    rings = 18
    prof = [(0, -R)]
    for i in range(1, rings):
        a = math.pi * i / rings - HPI
        prof.append((R * math.cos(a) * 0.999, R * math.sin(a)))
        if i == rings // 2 - 1:
            prof += [(R * 1.0028, -0.0014), (R * 1.0028, 0.0014)]
    prof.append((0, R))
    prof = sorted(set(prof), key=lambda p: p[1])
    return lathe(prof, seg=28)

part('FloorGlobes__globe', f_sphere(0.14), earth_globe, smooth=60)


# =========================================================== moon globes ==
# turned base: a thumbnail edge, an ogee rise and a stepped top
part('MoonGlobes__base', f_cyl(0.09, 0.11, 0.020), lambda: lathe(
    [(0, -0.010), (0.1080, -0.010), (0.1100, -0.0080), (0.1095, -0.0045), (0.1020, -0.0015), (0.0940, 0.0015),
     (0.0905, 0.0050), (0.0880, 0.0070), (0.0600, 0.0085), (0.0300, 0.0100), (0, 0.0100)], seg=28), smooth=40)

part('MoonGlobes__column', f_cyl(0.010, 0.014, 0.32), lambda: baluster(0.32, 0.010, 0.014), smooth=55)

# horizon ring (the placer lays it flat round the moon)
part('MoonGlobes__yoke', f_torus(0.12, 0.004), lambda: meridian(0.12, 0.004, TAU, seg=44), smooth=35)


def moon():
    # a gently lumpy sphere, kept just under the maria/crater decals
    s = sphere(0.1093, seg=22, rings=12)
    s = noisy(s, 0.0006, scale=22, seed=11)
    return s.fit((-0.11, -0.11, -0.11), (0.11, 0.11, 0.11)).scale(0.996)

part('MoonGlobes__moon', f_sphere(0.110), moon, smooth=70)
