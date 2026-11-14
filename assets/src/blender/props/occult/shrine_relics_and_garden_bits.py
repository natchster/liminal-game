"""Batch b23: the "occult" and "nature" themes.

Carved wall crucifixes, iron stained-glass frames, a relic chest, a hanging
thurible, a gilt triptych, a memorial shrine, a dress form, mounted skulls,
a brass candelabra, a heavy chain, a crow, an hourglass, a planchette, wax
candles; then pots, stems, leaves, clods, pebbles, twigs, acorns, pine cones,
a quill and mushrooms. Glass, crystal, flames, glows, decals and DoubleSide
cards/leaves/petals stay primitives (see docs/research/props-remodelling/shrine_relics_and_garden_bits.md).
"""
from propkit import *
import propkit
from mathutils import Vector, noise

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


def stack(pts, prof, ref=None):
    """Moulded plate: the XY outline inset by `inset` at depth z for every
    (inset, z) in prof (back to front), rings bridged, ends capped. With
    `ref` the rings are scaled about the origin instead (inset/ref), which
    never self-intersects on a star-shaped outline."""
    base = _ccw(pts)
    if ref:
        rings = [[(x * (1 - i / ref), y * (1 - i / ref)) for x, y in base] for i, _ in prof]
    else:
        rings = [base if abs(i) < 1e-9 else offset2d(base, i) for i, _ in prof]
    n = len(base)
    v, f = [], []
    for ring, (_, z) in zip(rings, prof):
        v += [(x, y, z) for x, y in ring]
    R = len(prof)
    for r in range(R - 1):
        a, b = r * n, (r + 1) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    f.append(tuple(reversed(range(n))))
    f.append(tuple((R - 1) * n + i for i in range(n)))
    return fixw(Shape(v, f))


def ring_sec(R, sec, seg=32, arc=TAU, a0=0.0):
    """Sweep a closed section [(dr, dz), ...] around Z in the XY plane (a
    torus_geo! with any cross-section)."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        for dr, dz in sec:
            v.append(((R + dr) * math.cos(a), (R + dr) * math.sin(a), dz))
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


def circ(r, n=8, sx=1.0, sy=1.0, phase=0.0):
    return [(r * sx * math.cos(TAU * i / n + phase), r * sy * math.sin(TAU * i / n + phase)) for i in range(n)]


def lobes(s, n, depth, y0, y1, phase=0.0, sharp=1.0):
    """Gadroons / ribs: push the radius in n times around Y between y0 and y1."""
    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6 or not (y0 - 1e-7 <= p.y <= y1 + 1e-7):
            return p
        t = (p.y - y0) / ((y1 - y0) or 1)
        fade = max(0.0, math.sin(math.pi * min(1.0, max(0.0, t)))) ** 0.5
        a = math.atan2(p.x, p.z)
        c = (0.5 - 0.5 * math.cos(n * a + phase)) ** sharp
        k = max(0.0, r - depth * fade * c) / r
        p.x *= k
        p.z *= k
        return p
    return s.warp(w)


def bn(s, p):
    """Bullnose: 1 in the middle, rolling to 0 at s = +-1."""
    return max(0.0, 1 - abs(s) ** p) ** 0.5


def pillow(w, h, t, zfn, nx=16, ny=6):
    """A bar/board with a flat back at -t/2 and a front z = zfn(u, v)
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


def link_loop(L, W, r, n=6, tseg=3):
    """A chain link: a closed loop L long (Y) and W wide (X), wire radius r,
    the path an n-gon stretched into a stadium."""
    pts = []
    for i in range(n):
        a = TAU * (i + 0.5) / n
        pts.append(((W / 2 - r) * math.sin(a), (L / 2 - r) * math.cos(a)))
    v, f = [], []
    for i, (x, y) in enumerate(pts):
        px, py = pts[i - 1]
        nx_, ny_ = pts[(i + 1) % n]
        tx, ty = nx_ - px, ny_ - py
        tl = math.hypot(tx, ty)
        ox, oy = ty / tl, -tx / tl
        for j in range(tseg):
            b = TAU * j / tseg
            v.append((x + ox * r * math.cos(b), y + oy * r * math.cos(b), r * math.sin(b)))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(tseg):
            j2 = (j + 1) % tseg
            f.append((i * tseg + j, i2 * tseg + j, i2 * tseg + j2, i * tseg + j2))
    return fixw(Shape(v, f))


def chain_y(L, n, W, r, lseg=6, tseg=3):
    """n links along Y over length L, alternating 90 degrees."""
    p = (L - 4 * r) / n
    lo = p + 4 * r
    out = Shape()
    for i in range(n):
        lk = link_loop(lo, W, r, lseg, tseg)
        if i % 2:
            lk.rot(y=HPI)
        out = out + lk.move(y=-L / 2 + lo / 2 + i * p)
    return out


# ================================================================ occult ==
# ------------------------------------------------------ wall crucifixes --
def ridge_bar(w, h, t, rise, long='y'):
    """Carved bar: flat back, square sides, a hipped roof rising `rise` to a
    ridge along the long axis. Two of them crossing mitre into an X."""
    hw, hh = w / 2, h / 2
    zs, zt = t / 2 - rise, t / 2
    if long == 'y':
        a = min(hw, hh)
        r0, r1 = (0, -(hh - a)), (0, hh - a)
    else:
        a = min(hw, hh)
        r0, r1 = (-(hw - a), 0), (hw - a, 0)
    v = [(-hw, -hh, -t / 2), (hw, -hh, -t / 2), (hw, hh, -t / 2), (-hw, hh, -t / 2),
         (-hw, -hh, zs), (hw, -hh, zs), (hw, hh, zs), (-hw, hh, zs),
         (r0[0], r0[1], zt), (r1[0], r1[1], zt)]
    f = [(3, 2, 1, 0), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
    if long == 'y':
        f += [(5, 6, 9, 8), (7, 4, 8, 9), (4, 5, 8), (6, 7, 9)]
    else:
        f += [(6, 7, 8, 9), (4, 5, 9, 8), (7, 4, 8), (5, 6, 9)]
    s = fixw(Shape(v, f))
    return bevel(s, min(w, h) * 0.035, seg=1, angle=12)

part('WallCrucifixes__upright', f_box(0.026, 0.28, 0.020),
     lambda: ridge_bar(0.026, 0.28, 0.020, 0.0055, 'y'), smooth=20)
part('WallCrucifixes__crossbar', f_box(0.16, 0.024, 0.020),
     lambda: ridge_bar(0.16, 0.024, 0.020, 0.0055, 'x'), smooth=20)


# ------------------------------------------------ stained glass windows --
# one constructor makes all four iron bars (long rails and tall stiles), so
# the forged bar crowns towards all four edges; a light hammered surface
def glass_bar():
    t = 0.012
    s = pillow(0.34, 0.020, t, lambda u, v: t / 2 - 0.0035 + 0.0035 * bn(u, 9) * bn(v, 9), nx=20, ny=8)
    return noisy(s, 0.00025, scale=260, seed=4, axis=(0, 0, 1))

part('StainedGlassWindows__bar', f_box(0.34, 0.020, 0.012), glass_bar, smooth=50)


# ------------------------------------------------------ reliquary boxes --
def arch_cutter(w, h, depth, z0):
    """A round-headed niche outline extruded along Z, face at z0."""
    r = w / 2
    pts = [(-r, -h / 2), (r, -h / 2)]
    for i in range(6):
        a = math.pi * i / 5
        pts.append((r * math.cos(a), h / 2 - r + r * math.sin(a)))
    return prism(pts[:2] + pts[2:], depth).move(z=z0)


def reliquary_body():
    # wooden chest: eased edges, a lid seam under the top gilt band, round
    # arched niches between the bands (three on the long sides, two on the
    # ends), a sunk panel on the lid
    W, H, D = 0.14, 0.06, 0.09
    s = rbox(W, H, D, r=0.0025, seg=1)
    ys = 0.0185
    cutters = [box(W + 0.01, 0.0012, 0.0016).move(y=ys, z=sz * D / 2) for sz in (-1, 1)]
    cutters += [box(0.0016, 0.0012, D + 0.01).move(x=sx * W / 2, y=ys) for sx in (-1, 1)]
    for sz in (-1, 1):
        for x in (-0.042, 0, 0.042):
            c = arch_cutter(0.026, 0.034, 0.006, 0).move(x=x, y=-0.0005)
            cutters.append(c.move(z=sz * (D / 2 + 0.0015)))
    for sx in (-1, 1):
        for z in (-0.02, 0.02):
            c = arch_cutter(0.026, 0.034, 0.006, 0).rot(y=HPI).move(x=sx * (W / 2 + 0.0015), y=-0.0005, z=z)
            cutters.append(c)
    lid = prism(rounded_rect(0.096, 0.052, 0.006), 0.004).rot(x=HPI).move(y=H / 2 + 0.0012)
    return cut(s, *cutters, lid)

part('ReliquaryBoxes__body', f_box(0.14, 0.06, 0.09), reliquary_body, smooth=30)


def reliquary_trim():
    # gilt band: a half-round bead between two fillets, standing 2 mm proud
    out = rounded_rect(0.144, 0.094, 0.0015, seg=2)
    prof = [(0.0012, -0.003), (0.0006, -0.0026), (0.0006, -0.0017), (0.0001, -0.0010), (0.0, 0.0),
            (0.0001, 0.0010), (0.0006, 0.0017), (0.0006, 0.0026), (0.0012, 0.003)]
    return stack(out, prof).rot(x=-HPI)

part('ReliquaryBoxes__trim', f_box(0.144, 0.006, 0.094), reliquary_trim, smooth=45)


# ------------------------------------------------------ hanging censers --
# thurible chain: long oval links turned 90 degrees one to the next
part('HangingCensers__chain', f_cyl(0.002, 0.002, 0.75),
     lambda: chain_y(0.75, 30, 0.0064, 0.0007, lseg=6, tseg=3), smooth=180)


def censer_bowl():
    # the fire bowl under the rim: a gadrooned belly on a short stem and a
    # spreading foot ring; closed flat under the lid
    s = lathe([(0, -0.035), (0.0165, -0.035), (0.0180, -0.0336), (0.0150, -0.0318), (0.0090, -0.0290),
               (0.0100, -0.0262), (0.0200, -0.0215), (0.0290, -0.0140), (0.0335, -0.0060),
               (0.0350, -0.0015), (0.0342, 0.0), (0, 0.0)], seg=14)
    return lobes(s, 12, 0.0018, -0.0215, -0.004)

part('HangingCensers__bowl', f_bounds((-0.035, -0.035, -0.035), (0.035, 0.0, 0.035)), censer_bowl, smooth=50)

# the band round the joint: a flat-topped flange with a rounded outer edge
part('HangingCensers__rim', f_torus(0.035, 0.004), lambda: ring_sec(0.035, [
    (-0.0036, -0.0026), (0.0010, -0.0040), (0.0034, -0.0022), (0.0040, 0.0),
    (0.0034, 0.0022), (0.0010, 0.0040), (-0.0036, 0.0026)], seg=20), smooth=50)


def censer_lid():
    # pierced cover: a flange, a ribbed first tier, a stepped second tier, a
    # small cupola
    s = lathe([(0, 0.0), (0.0300, 0.0), (0.0300, 0.0030), (0.0280, 0.0045), (0.0270, 0.0090),
               (0.0240, 0.0135), (0.0205, 0.0145), (0.0195, 0.0185), (0.0160, 0.0215),
               (0.0120, 0.0225), (0.0105, 0.0260), (0.0060, 0.0292), (0, 0.030)], seg=12)
    return lobes(s, 8, 0.0022, 0.0045, 0.0135, sharp=2.0)

part('HangingCensers__lid', f_bounds((-0.03, 0.0, -0.03), (0.03, 0.03, 0.03)), censer_lid, smooth=45)

# finial: a collar on the lid, a ball, a pointed pip
part('HangingCensers__knob', f_sphere(0.008), lambda: lathe(
    [(0, -0.008), (0.0046, -0.008), (0.0068, -0.0035), (0.0072, -0.0005),
     (0.0052, 0.0032), (0.0016, 0.0056), (0, 0.008)], seg=8), smooth=60)


# -------------------------------------------------- religious triptychs --
# gilt frame: all four bars share the constructor, so a crowned cushion
# moulding with a flat outer fillet reads right on rails and stiles alike
part('ReligiousTriptychs__bar', f_box(0.468, 0.024, 0.020), lambda: pillow(
    0.468, 0.024, 0.020,
    lambda u, v: 0.004 + 0.006 * bn(u, 10) * bn(v, 5) - 0.0012 * math.exp(-((abs(v) - 0.55) / 0.12) ** 2) * bn(u, 10),
    nx=22, ny=10), smooth=40)


def triptych_divider():
    # colonnette between the panels: half-round shaft, square base and capital
    t = 0.020
    def z(u, v):
        if abs(v) > 0.93:
            return t / 2
        if abs(v) > 0.90:
            return t / 2 - 0.0015
        return t / 2 - 0.004 + 0.004 * math.sqrt(max(0.0, 1 - u * u))
    return pillow(0.008, 0.34, t, z, nx=6, ny=26)

part('ReligiousTriptychs__divider', f_box(0.008, 0.34, 0.020), triptych_divider, smooth=40)


# ---------------------------------------------------------- more kit --
def rsphere(fn, seg=16, rings=10, zpole=False):
    """UV sphere whose radius follows fn(unit direction) -> radius (zpole:
    poles on the Z axis, rings round the long axis of a skull)."""
    b = sphere(1.0, seg, rings)
    if zpole:
        b.rot(x=HPI)
    return b.warp(lambda p: p.normalized() * fn(p.normalized()) if p.length > 1e-9 else p)


def gauss(d, c, w):
    """1 at direction c, falling off with the angle to it (width w radians)."""
    c = Vector(c).normalized()
    a = math.acos(max(-1.0, min(1.0, d.dot(c))))
    return math.exp(-(a / w) ** 2)


def sweep(points, sec, scale=None, smooth=0, up=(0, 0, 1), caps=True):
    """Sweep a closed 2D section [(x, y), ...] along a 3D polyline; section x
    runs along up x tangent, y along the binormal. scale(t) -> k or (kx, ky)."""
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


def board(w, d, prof, r=0.004):
    """Plank in XZ: rounded-rect outline, moulded edge prof [(inset, y)...]."""
    return stack(rounded_rect(w, d, r, seg=2), prof).rot(x=-HPI)


def taper_candle(r, h, seed=0, crater=0.0025, drip=0.14):
    """Wax taper along Y: softened top, a shallow melt crater round the wick,
    one run of drip down the side."""
    y0, y1 = -h / 2, h / 2
    s = lathe([(0, y0), (r * 0.97, y0), (r, y0 + 0.0015), (r, y1 - 0.004), (r * 0.93, y1 - 0.0012),
               (r * 0.78, y1), (r * 0.5, y1 - crater * 0.6), (r * 0.28, y1 - crater),
               (r * 0.14, y1 - crater * 0.55), (0, y1 - crater * 0.25)], seg=10)

    def run(p):
        a = math.atan2(p.x, p.z) - seed
        a = (a + math.pi) % TAU - math.pi
        rr = math.hypot(p.x, p.z)
        if rr > r * 0.8 and p.y > y0 + h * 0.35:
            k = math.exp(-(a / 0.35) ** 2) * min(1.0, (p.y - y0 - h * 0.35) / (h * 0.3))
            p.x *= 1 + drip * k
            p.z *= 1 + drip * k
        return p
    return s.warp(run)


# --------------------------------------------------- memorial shrines --
part('MemorialShrines__platform', f_box(0.35, 0.020, 0.20), lambda: board(0.35, 0.20, [
    (0.0015, -0.010), (0.0, -0.0086), (0.0, 0.0030), (0.0008, 0.0062), (0.0026, 0.0087),
    (0.0050, 0.0098), (0.0068, 0.0100)], r=0.006), smooth=40)
part('MemorialShrines__step', f_box(0.21, 0.015, 0.14), lambda: board(0.21, 0.14, [
    (0.0008, -0.0075), (0.0, -0.0065), (0.0, 0.0030), (0.0012, 0.0062), (0.0032, 0.0075)], r=0.004),
    smooth=40)

# small standing photo frame: bead-and-cove moulding round a sunk centre
part('MemorialShrines__frame', f_box(0.092, 0.112, 0.008), lambda: stack(
    rounded_rect(0.092, 0.112, 0.0015, seg=2),
    [(0.0, -0.004), (0.0, 0.0012), (0.0010, 0.0032), (0.0024, 0.0040), (0.0038, 0.0036),
     (0.0046, 0.0024), (0.0054, 0.0012), (0.0060, 0.0006)]), smooth=40)

part('MemorialShrines__plate', f_box(0.056, 0.006, 0.001),
     lambda: rbox(0.056, 0.006, 0.001, r=0.00035, seg=1), smooth=30)

# brass drip dish: rolled rim, dished floor, socket collar for the candle
part('MemorialShrines__holder', f_cyl(0.015, 0.015, 0.006), lambda: lathe(
    [(0, -0.003), (0.0138, -0.003), (0.0150, -0.0022), (0.0148, 0.0012), (0.0140, 0.0030),
     (0.0128, 0.0026), (0.0122, 0.0002), (0.0100, -0.0008), (0.0094, 0.0002), (0.0092, 0.0030),
     (0.0080, 0.0030), (0, 0.0030)], seg=14), smooth=45)

part('MemorialShrines__candle', f_cyl(0.008, 0.008, 0.065),
     lambda: taper(taper_candle(0.008, 0.065, seed=0.5), top=0.88, along='y', axes='xz'), smooth=60)

# pewter bud vase: foot ring, full belly, tapering to the lip ring above it
part('MemorialShrines__vase', f_cyl(0.018, 0.018, 0.045), lambda: lathe(
    [(0, -0.0225), (0.0160, -0.0225), (0.0172, -0.0212), (0.0158, -0.0190), (0.0170, -0.0160),
     (0.0180, -0.0090), (0.0176, 0.0010), (0.0162, 0.0110), (0.0148, 0.0190), (0.0141, 0.0225),
     (0, 0.0225)], seg=14), smooth=50)
part('MemorialShrines__mouth', f_cyl(0.014, 0.014, 0.004), lambda: vessel(
    [(0.0128, -0.002), (0.0136, -0.0008), (0.0140, 0.0012)], 0.0022, seg=14), smooth=50)


def dried_stem(L, r, seed=0):
    # a dry, slightly kinked stalk with two nodes
    pts = [(0.0, -L / 2, 0.0), (r * 0.5, -L / 6, -r * 0.3), (-r * 0.4, L / 6, r * 0.3), (r * 0.3, L / 2, 0.0)]
    s = tube(pts, r * 0.8, seg=4, smooth=1)
    for y in (-L * 0.1, L * 0.25):
        s = s + lathe([(0, y - r * 1.2), (r * 1.05, y - r * 0.4), (r * 1.05, y + r * 0.4), (0, y + r * 1.2)], seg=4)
    return s

part('MemorialShrines__stem', f_cyl(0.0008, 0.0008, 0.075), lambda: dried_stem(0.075, 0.0008), smooth=60)


def dried_bud():
    # a dried rosebud: five petal lobes closed to a point, sepals below
    s = lathe([(0, -0.006), (0.0022, -0.0050), (0.0056, -0.0022), (0.0061, 0.0008),
               (0.0040, 0.0040), (0.0012, 0.0058), (0, 0.006)], seg=10)
    s = lobes(s, 5, 0.0012, -0.004, 0.006, sharp=0.6)
    return noisy(s, 0.00015, scale=500, seed=2)

part('MemorialShrines__bud', f_sphere(0.006), dried_bud, smooth=70)

# the little gilt standing cross: carved ridge bars like the wall crucifix
part('MemorialShrines__crossV', f_box(0.006, 0.055, 0.003),
     lambda: ridge_bar(0.006, 0.055, 0.003, 0.0009, 'y'), smooth=20)
part('MemorialShrines__crossH', f_box(0.030, 0.006, 0.003),
     lambda: ridge_bar(0.030, 0.006, 0.003, 0.0009, 'x'), smooth=20)


def ribbon_fold():
    # black ribbon lying on the platform, folded back over itself at one end
    th, w = 0.0011, 0.030
    path = [(0.035, -0.0034, 0), (0.010, -0.0036, 0), (-0.015, -0.0033, 0), (-0.028, -0.0034, 0)]
    for i in range(1, 8):
        a = -HPI + math.pi * i / 8
        path.append((-0.028 - 0.0034 * math.cos(a), 0.0034 * math.sin(a), 0))
    path += [(-0.028, 0.0034, 0), (-0.012, 0.0030, 0), (0.004, 0.0036, 0)]
    sec = [(-th / 2, -w / 2), (th / 2, -w / 2), (th / 2, w / 2), (-th / 2, w / 2)]
    s = sweep(path, sec, up=(0, 0, 1))
    # soft ripples across the width
    return s.warp(lambda p: Vector((p.x, p.y + 0.0006 * math.sin(p.x * 180 + p.z * 90), p.z)))

part('MemorialShrines__ribbon', f_box(0.070, 0.008, 0.030), ribbon_fold, smooth=40)


def ribbon_tail():
    # the hanging end: a gentle wave, cut in a swallowtail
    s = slab(0.030, 0.050, 0.0011, nx=6, ny=8)

    def w(p):
        t = (0.025 - p.y) / 0.05
        p.y += 0.009 * max(0.0, 1 - abs(p.x) / 0.015) * t * t
        p.z += 0.0024 * math.sin(p.y * 70 + 0.6) * (0.4 + 0.6 * t)
        return p
    return s.warp(w).rot(y=HPI)

part('MemorialShrines__tail', f_box(0.006, 0.05, 0.030), ribbon_tail, smooth=50)


# --------------------------------------------------------- dress forms --
# tripod stand: turned legs, pad feet, a turned hub, the column
part('DressForms__leg', f_cyl(0.020, 0.020, 0.30), lambda: lathe(
    [(0, -0.15), (0.0172, -0.15), (0.0200, -0.1470), (0.0200, -0.1400), (0.0176, -0.1360),
     (0.0180, -0.1300), (0.0168, -0.0400), (0.0150, 0.0800), (0.0140, 0.1250), (0.0158, 0.1300),
     (0.0158, 0.1380), (0.0140, 0.1420), (0.0120, 0.1500), (0, 0.15)], seg=10), smooth=50)
part('DressForms__foot', f_cyl(0.024, 0.024, 0.008), lambda: lathe(
    [(0, -0.004), (0.0226, -0.004), (0.0240, -0.0028), (0.0240, -0.0005), (0.0226, 0.0015),
     (0.0200, 0.0030), (0.0160, 0.0040), (0, 0.004)], seg=16), smooth=45)
part('DressForms__hub', f_sphere(0.024), lambda: lathe(
    [(0, -0.024), (0.0130, -0.024), (0.0150, -0.0205), (0.0145, -0.0170), (0.0215, -0.0110),
     (0.0240, -0.0020), (0.0225, 0.0080), (0.0160, 0.0150), (0.0150, 0.0190), (0.0135, 0.0240),
     (0, 0.024)], seg=14), smooth=45)
part('DressForms__pole', f_cyl(0.012, 0.012, 0.60), lambda: lathe(
    [(0, -0.30), (0.0120, -0.30), (0.0120, -0.288), (0.0100, -0.284), (0.0098, 0.284),
     (0.0120, 0.288), (0.0120, 0.300), (0, 0.300)], seg=12), smooth=50)


def knurled_wheel():
    # brass hand wheel: a flat ring with a knurled rim of 18 grips
    sec = [(-0.0030, -0.0024), (0.0026, -0.0030), (0.0040, 0.0), (0.0026, 0.0030), (-0.0030, 0.0024)]
    s = ring_sec(0.016, sec, seg=28)

    def knurl(p):
        a = math.atan2(p.y, p.x)
        r = math.hypot(p.x, p.y)
        if r > 0.0165:
            k = 1 - 0.07 * max(0.0, math.cos(14 * a)) ** 2 * (r - 0.0165) / 0.0035
            p.x *= k
            p.y *= k
        return p
    return s.warp(knurl)

part('DressForms__wheel', f_torus(0.016, 0.004), knurled_wheel, smooth=50)


def torso_piece(r, sy, sz, y_lo, y_hi, prof, bust=0.0, seg=20):
    """A section of the padded torso, built in the placer's unscaled sphere
    frame (radius r, later scaled (1, sy, sz)). prof: world-size [(radius, y)]
    bottom to top; bust pushes two forward lobes out on the +Z side."""
    pts = [(0, y_lo / sy)] + [(rr, yy / sy) for rr, yy in prof] + [(0, y_hi / sy)]
    s = lathe(pts, seg=seg)

    def shape(p):
        rr = math.hypot(p.x, p.z)
        if rr < 1e-9:
            return p
        a = math.atan2(p.x, p.z)
        # a flatter back and sides that stay full: a softened superellipse
        k = 1.0 - 0.06 * max(0.0, -math.cos(a)) ** 2
        if bust:
            yw = p.y * sy
            for cx in (-0.62, 0.62):
                g = math.exp(-((a - cx) / 0.36) ** 2) * math.exp(-((yw - 0.002) / 0.026) ** 2)
                k += bust * g
        p.x *= k
        p.z *= k
        return p
    return s.warp(shape)

# hip: a flat base, full hips, drawing in towards the waist (reaches 9 mm
# above its sphere to close on the waist)
part('DressForms__hip', f_sphere(0.14), lambda: torso_piece(0.14, 0.45, 0.85, -0.063, 0.070, [
    (0.114, -0.063), (0.124, -0.060), (0.131, -0.052), (0.137, -0.036), (0.140, -0.014), (0.138, 0.008),
    (0.130, 0.030), (0.118, 0.048), (0.106, 0.062), (0.098, 0.070)]), smooth=60)
# waist: a cinched band, flared top and bottom
part('DressForms__waist', f_sphere(0.10), lambda: torso_piece(0.10, 0.35, 0.85, -0.041, 0.041, [
    (0.090, -0.041), (0.098, -0.036), (0.100, -0.028), (0.093, -0.012), (0.090, 0.004),
    (0.094, 0.022), (0.100, 0.034), (0.096, 0.041)]), smooth=60)
# bust: under-bust drawn in, two forward lobes, rising to the shoulders
part('DressForms__bust', f_sphere(0.13), lambda: torso_piece(0.13, 0.50, 0.85, -0.078, 0.077, [
    (0.092, -0.078), (0.104, -0.070), (0.116, -0.052), (0.124, -0.030), (0.126, -0.006),
    (0.124, 0.018), (0.118, 0.040), (0.110, 0.060), (0.104, 0.077)], bust=0.15), smooth=60)


def shoulder_yoke():
    # sloping shoulders rounding over to the neck, narrower front to back
    s = lathe([(0, -0.030), (0.106, -0.030), (0.110, -0.024), (0.108, -0.012), (0.100, 0.000),
               (0.086, 0.012), (0.066, 0.022), (0.046, 0.029), (0.038, 0.0325), (0, 0.0335)], seg=20)
    return s.scale(1, 1, 0.86)

part('DressForms__shoulder', f_cyl(0.11, 0.11, 0.06), shoulder_yoke, smooth=60)
part('DressForms__collar', f_cyl(0.040, 0.040, 0.030), lambda: lathe(
    [(0, -0.015), (0.0400, -0.015), (0.0360, -0.0115), (0.0318, -0.0060), (0.0300, 0.0040),
     (0.0302, 0.0110), (0.0290, 0.0142), (0.0250, 0.0150), (0, 0.0150)], seg=16), smooth=50)


def pincushion():
    # tomato pincushion: eight puffed segments, dimpled top and bottom
    s = lathe([(0, -0.0150), (0.0060, -0.0158), (0.0130, -0.0130), (0.0172, -0.0065), (0.0180, 0.0010),
               (0.0168, 0.0080), (0.0128, 0.0140), (0.0060, 0.0160), (0, 0.0132)], seg=16)
    return lobes(s, 8, 0.0022, -0.016, 0.016, sharp=2.0)

part('DressForms__cushion', f_sphere(0.018), pincushion, smooth=70)


def drape():
    # cloth laid over the bust: loose folds running front to back, the free
    # edges falling away
    s = slab(0.24, 0.15, 0.0014, nx=14, ny=8).rot(x=-HPI)

    def w(p):
        u, v = p.x / 0.12, p.z / 0.075
        p.y += 0.0020 * math.sin(u * 7.0 + 0.5) + 0.0010 * math.sin(u * 13 + v * 3)
        p.y -= 0.0025 * max(0.0, abs(v) - 0.4) ** 2 / 0.36
        return p
    return noisy(s.warp(w), 0.0005, scale=40, seed=5, axis=(0, 1, 0))

part('DressForms__drape', f_box(0.24, 0.005, 0.15), drape, smooth=60)


def fabric_flow():
    # the loose length hanging at the side: vertical folds that open towards
    # the hem, the hem uneven
    s = slab(0.10, 0.22, 0.0014, nx=12, ny=6)

    def w(p):
        t = (0.11 - p.y) / 0.22
        p.z += 0.0026 * math.sin(p.x / 0.05 * 7.5 + 0.4) * (0.5 + 0.5 * t)
        if t > 0.95:
            p.y += 0.004 * math.sin(p.x * 90)
        return p
    return s.warp(w).rot(y=HPI)

part('DressForms__flow', f_box(0.008, 0.22, 0.10), fabric_flow, smooth=60)


# -------------------------------------------------------- skull plaques --
PW, PH = 0.18, 0.22
JOINT = -0.0335           # where the shield's upper board meets its point


def shield_pts():
    """Shield outline in plaque coordinates: a scalloped crest, straight
    sides, drawing in to 0.126 wide at the joint, curving to a point."""
    hw = PW / 2
    top = 0.1095
    tp = [(-hw, top - 0.015), (-hw * 0.9, top - 0.0095), (-hw * 0.72, top - 0.0060), (-hw * 0.48, top - 0.0030),
          (-hw * 0.24, top - 0.0008), (0.0, top)]
    rt = [(-x, y) for x, y in reversed(tp[:-1])]
    side = [(hw, 0.030), (hw * 0.985, 0.006), (hw * 0.93, -0.012), (hw * 0.82, -0.025), (0.063, JOINT),
            (0.056, -0.052), (0.044, -0.072), (0.028, -0.090), (0.012, -0.104), (0.0, -0.110)]
    lside = [(-x, y) for x, y in reversed(side[:-1])]
    return round_outline(tp + rt + side + lside, 0.002, seg=1)


def ogee_prof(t):
    z0 = -t / 2
    return [(0, z0), (0, z0 + t * 0.55), (0.0016, t / 2 - t * 0.2), (0.0036, t / 2 - t * 0.14),
            (0.0048, t / 2 - t * 0.02), (0.0066, t / 2)]


def shield_half(upper):
    s = stack(shield_pts(), ogee_prof(0.012), ref=0.092)
    if upper:
        s = boolean(s, box(0.3, 0.2, 0.05).move(y=JOINT + 0.1), 'INTERSECT')
        return s.move(y=-PH * 0.175)
    s = boolean(s, box(0.3, 0.2, 0.05).move(y=JOINT - 0.1), 'INTERSECT')
    return s.move(y=PH * 0.325)

part('SkullPlaques__shieldTop', f_box(0.18, 0.143, 0.012), lambda: shield_half(True), smooth=40)
part('SkullPlaques__shieldPoint', f_box(0.126, 0.077, 0.012), lambda: shield_half(False), smooth=40)
# the round variant: the moulded face towards +Y, which the placer turns to
# face the room
part('SkullPlaques__oval', f_cyl(0.09, 0.09, 0.012), lambda: stack(
    circ(0.09, 40), ogee_prof(0.012)).rot(x=-HPI), smooth=40)


def id_plate():
    # engraved brass label with a screw at each end
    s = rbox(0.09, 0.014, 0.0008, r=0.0003, seg=1).move(z=-0.0001)
    for sx in (-1, 1):
        s = s + lathe([(0, 0), (0.0016, 0), (0.0012, 0.0004), (0, 0.0005)], seg=6).rot(x=HPI).move(x=sx * 0.041, z=0.0003)
    return s

part('SkullPlaques__idPlate', f_box(0.09, 0.014, 0.001), id_plate, smooth=40)


def fox_cranium():
    # canid braincase in the sphere's unscaled frame (snout towards +Z):
    # sagittal crest, pinched behind the eyes, flared cheekbones, deep orbits,
    # a flatter underside
    def fn(d):
        r = 0.030
        r *= 1 - 0.20 * max(0.0, -d.y) ** 2
        r *= 1 - 0.20 * math.exp(-((d.z - 0.55) / 0.35) ** 2) * (1 - abs(d.y)) * (0.4 + 0.6 * max(0.0, d.y))
        r += 0.0030 * math.exp(-(d.x / 0.2) ** 2) * max(0.0, d.y) ** 3
        for sx in (-1, 1):
            r += 0.0045 * gauss(d, (sx * 0.86, -0.25, 0.45), 0.30)
            r -= 0.0085 * gauss(d, (sx * 0.52, 0.22, 0.82), 0.30)
        r -= 0.0030 * gauss(d, (0, -0.2, -0.98), 0.35)
        return r
    return rsphere(fn, seg=14, rings=10, zpole=True).fit((-0.03, -0.03, -0.03), (0.03, 0.03, 0.03))

part('SkullPlaques__foxCranium', f_sphere(0.030), fox_cranium, smooth=70)


def muzzle(w, h, d, tip_w, tip_h, nose=0.35):
    """Bony muzzle along +Z: flat palate, top sloping down to the nose, a
    nasal opening cut in the front."""
    rings = []
    for i in range(5):
        t = i / 4
        z = -d / 2 + d * t
        ww = w + (tip_w - w) * t ** 0.8
        hh = h + (tip_h - h) * t
        yb = -h / 2
        sec = []
        for x, y in [(-1, 0), (-0.8, 0.55), (-0.35, 1), (0.35, 1), (0.8, 0.55), (1, 0), (0.85, -0.05), (-0.85, -0.05)]:
            sec.append((x * ww / 2, yb + y * hh, z))
        rings.append(sec)
    s = loft(rings)
    return cut(s, ellipsoid(tip_w * 0.28, tip_h * nose, 0.004, seg=8).move(y=-h / 2 + tip_h * 0.55, z=d / 2))

part('SkullPlaques__foxSnout', f_box(0.025, 0.020, 0.045), lambda: muzzle(0.025, 0.020, 0.045, 0.012, 0.011), smooth=70)


def tooth_row():
    # incisors: one scalloped plate, six little crowns along the bottom
    pts = [(-0.010, 0.001), (0.010, 0.001)]
    for i in range(6):
        x0 = 0.010 - i * (0.020 / 6)
        x1 = x0 - 0.020 / 6
        pts += [(x0 - 0.0003, -0.0004), ((x0 + x1) / 2, -0.001), (x1 + 0.0003, -0.0004)]
    return prism(pts, 0.004)

part('SkullPlaques__teeth', f_box(0.020, 0.002, 0.004), tooth_row, smooth=30)
part('SkullPlaques__canine', f_cone(0.002, 0.006), lambda: bend(lathe(
    [(0, -0.003), (0.0019, -0.003), (0.0017, -0.0012), (0.0010, 0.0012), (0, 0.003)], seg=6),
    0.35, 'x', 'y'), smooth=60)


def deer_cranium():
    # cervid braincase: rounded vault, pedicle bosses up top, bony orbit rims
    # standing out at the front corners round deep sockets
    def fn(d):
        r = 0.032
        r *= 1 - 0.16 * max(0.0, -d.y) ** 2
        r *= 1 - 0.10 * math.exp(-((d.z - 0.6) / 0.35) ** 2) * max(0.0, d.y)
        for sx in (-1, 1):
            r += 0.0040 * gauss(d, (sx * 0.55, 0.78, -0.15), 0.26)
            r += 0.0045 * gauss(d, (sx * 0.80, 0.10, 0.58), 0.30)
            r -= 0.0085 * gauss(d, (sx * 0.58, 0.14, 0.80), 0.24)
        return r
    return rsphere(fn, seg=14, rings=10, zpole=True).fit((-0.032, -0.032, -0.032), (0.032, 0.032, 0.032))

part('SkullPlaques__deerCranium', f_sphere(0.032), deer_cranium, smooth=70)
part('SkullPlaques__deerFace', f_box(0.028, 0.018, 0.055), lambda: muzzle(0.028, 0.018, 0.055, 0.013, 0.012, 0.30),
     smooth=70)


def antler_piece(h, r0, r1, burr=0.0, seed=0, seg=7, curve=0.25):
    """Beam with a pearled surface and a slight forward curve; burr ring at
    the base."""
    n = 7
    prof = [(0, -h / 2)]
    if burr:
        prof += [(burr * 0.9, -h / 2), (burr, -h / 2 + 0.0015), (burr * 0.86, -h / 2 + 0.0035)]
    else:
        prof += [(r0, -h / 2)]
    for i in range(1, n):
        t = i / n
        prof.append((r0 + (r1 - r0) * t ** 0.8, -h / 2 + h * t))
    prof += [(r1 * 0.6, h / 2 - r1 * 0.3), (0, h / 2)]
    s = lathe(prof, seg=seg)
    s = noisy(s, r0 * 0.12, scale=700, seed=seed)
    s = bend(s, curve, 'x', 'y')
    lo, hi = s.bounds()
    return s.move(z=-(lo.z + hi.z) / 2, x=-(lo.x + hi.x) / 2)

part('SkullPlaques__antler', f_cyl(0.008, 0.008, 0.035),
     lambda: antler_piece(0.035, 0.0068, 0.0040, burr=0.0080, seed=1), smooth=70)
part('SkullPlaques__fork', f_cyl(0.005, 0.005, 0.018),
     lambda: antler_piece(0.018, 0.0048, 0.0020, seed=3, seg=6, curve=0.12), smooth=70)


def raven_cranium():
    # corvid skull: round braincase behind, huge orbits on the sides, the
    # front drawing to the beak root (+Z)
    def fn(d):
        r = 0.020
        r *= 1 - 0.30 * max(0.0, d.z) ** 2 * (1 - 0.6 * max(0.0, d.y))
        for sx in (-1, 1):
            r -= 0.0055 * gauss(d, (sx * 0.78, 0.10, 0.50), 0.36)
        r += 0.0012 * gauss(d, (0, 0.62, 0.78), 0.35)
        return r
    return rsphere(fn, seg=14, rings=10, zpole=True).fit((-0.02, -0.02, -0.02), (0.02, 0.02, 0.02))

part('SkullPlaques__ravenCranium', f_sphere(0.020), raven_cranium, smooth=70)


def raven_beak():
    # long pointed beak (apex +Y), taller than wide, culmen ridge, nostril
    # grooves, the tip hooked a little towards +X (down once placed)
    n = 8
    rings = []
    for i in range(n + 1):
        t = i / n
        y = -0.0275 + 0.055 * t
        hh = 0.0080 * (1 - t) ** 0.85 + 0.0004
        ww = 0.0066 * (1 - t) ** 0.95 + 0.0003
        cx = 0.0016 * t ** 3
        sec = []
        for k in range(8):
            a = TAU * k / 8
            x, z = math.cos(a), math.sin(a)
            ridge = 1.0 + 0.12 * max(0.0, -x) ** 4
            sec.append((cx + x * hh * ridge, y, z * ww))
        rings.append(sec)
    return loft(rings)

part('SkullPlaques__ravenBeak', f_cone(0.008, 0.055), raven_beak, smooth=45)


# --------------------------------------------------- small candelabras --
# brass: a stepped, domed foot; a turned stem with collars under each knop;
# flattened cushion knops; slim arms; drip cups; wax tapers
part('SmallCandelabras__base', f_cyl(0.055, 0.055, 0.014), lambda: lathe(
    [(0, -0.007), (0.0535, -0.007), (0.0550, -0.0058), (0.0548, -0.0040), (0.0505, -0.0030),
     (0.0490, -0.0015), (0.0440, 0.0005), (0.0350, 0.0035), (0.0240, 0.0058), (0.0150, 0.0066),
     (0.0120, 0.0070), (0, 0.0070)], seg=20), smooth=45)
part('SmallCandelabras__column', f_cyl(0.010, 0.010, 0.14), lambda: lathe(
    [(0, -0.070), (0.0100, -0.070), (0.0100, -0.066), (0.0078, -0.063), (0.0072, -0.052),
     (0.0092, -0.049), (0.0070, -0.046), (0.0068, -0.012), (0.0090, -0.009), (0.0068, -0.006),
     (0.0066, 0.028), (0.0088, 0.031), (0.0066, 0.034), (0.0064, 0.066), (0.0090, 0.068),
     (0.0086, 0.070), (0, 0.070)], seg=12), smooth=45)
part('SmallCandelabras__knop', f_sphere(0.018), lambda: lathe(
    [(0, -0.018), (0.0080, -0.018), (0.0110, -0.0150), (0.0165, -0.0060), (0.0180, 0.0),
     (0.0165, 0.0060), (0.0110, 0.0150), (0.0080, 0.018), (0, 0.018)], seg=16), smooth=40)
part('SmallCandelabras__arm', f_cyl(0.004, 0.004, 0.061), lambda: lathe(
    [(0, -0.0305), (0.0040, -0.0305), (0.0040, -0.0280), (0.0028, -0.0250), (0.0026, -0.0040),
     (0.0038, 0.0), (0.0026, 0.0040), (0.0024, 0.0260), (0.0036, 0.0290), (0.0034, 0.0305),
     (0, 0.0305)], seg=8), smooth=50)
part('SmallCandelabras__cup', f_cyl(0.012, 0.012, 0.006), lambda: lathe(
    [(0, -0.003), (0.0070, -0.003), (0.0060, -0.0018), (0.0105, 0.0012), (0.0120, 0.0024),
     (0.0115, 0.0030), (0.0100, 0.0022), (0.0074, 0.0014), (0.0070, 0.0030), (0, 0.0030)], seg=14),
     smooth=45)
part('SmallCandelabras__candle', f_cyl(0.006, 0.006, 0.05),
     lambda: taper_candle(0.006, 0.05, seed=1.2, crater=0.0018), smooth=60)


# ------------------------------------------------------ small chain links --
def forged_link():
    # heavy forged link: slightly oval, the section a little flattened on the
    # faces, a weld swelling on one side
    R, r = 0.024, 0.005

    def rm(a):
        return 1.0 + 0.10 * math.exp(-((a - math.pi) / 0.35) ** 2)
    sec = [(r * math.cos(TAU * j / 6), r * 0.9 * math.sin(TAU * j / 6)) for j in range(6)]
    v, f = [], []
    n = 22
    for i in range(n):
        a = TAU * i / n
        k = rm(a)
        ex, ey = 1.0, 0.92
        for dr, dz in sec:
            rr = R + dr * k
            v.append((rr * math.cos(a) * ex, rr * math.sin(a) * ey, dz * k))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(6):
            j2 = (j + 1) % 6
            f.append((i * 6 + j, i2 * 6 + j, i2 * 6 + j2, i * 6 + j2))
    s = fixw(Shape(v, f))
    return noisy(s, 0.0003, scale=150, seed=7)

part('SmallChainLinks__link', f_torus(0.024, 0.005), forged_link, smooth=60)


# ---------------------------------------------------------- small crows --
def crow_body():
    # in the unscaled sphere frame (x later stretched 1.4, head end +X): a
    # full breast, the back sloping and narrowing to the tail root, folded
    # wings bulging high on the flanks, a flatter belly
    def w(p):
        t = max(0.0, -p.x / 0.03)
        k = 1 - 0.50 * t ** 1.6
        p.y *= k
        p.z *= k
        p.y += 0.004 * t * t
        if p.x > 0 and p.y < 0.005:
            p.x *= 1 + 0.06 * (1 - abs(p.z) / 0.03)
        if p.y < 0:
            p.y *= 0.88
        side = abs(p.z) / 0.03
        if p.y > -0.006 and side > 0.4:
            g = math.exp(-((p.y - 0.008) / 0.012) ** 2) * math.exp(-((p.x + 0.004) / 0.018) ** 2)
            p.z *= 1 + 0.08 * g
        return p
    s = sphere(0.03, 16, 9).warp(w)
    return s.fit((-0.03, -0.03, -0.03), (0.03, 0.03, 0.03))

part('SmallCrows__body', f_sphere(0.030), crow_body, smooth=70)


def crow_head():
    # a flat crown sloping to a steep forehead at +X, a shaggy throat, the
    # cheeks drawn in towards the bill
    def w(p):
        if p.y > 0:
            p.y *= 0.84 - 0.10 * max(0.0, -p.x / 0.02)
        if p.y < 0 and p.x > -0.008:
            p.y *= 1.0 + 0.08 * max(0.0, p.x / 0.02 + 0.4)
        p.z *= 1 - 0.14 * max(0.0, p.x / 0.02) ** 2
        return p
    return sphere(0.02, 14, 9).warp(w)

part('SmallCrows__head', f_sphere(0.020), crow_head, smooth=70)


def crow_beak():
    # heavy corvid bill along +Y (apex): deep at the root, arched culmen,
    # laterally pinched, a mouth line; the arch bows towards +X (up once the
    # placer lays it along the head)
    n = 7
    rings = []
    for i in range(n + 1):
        t = i / n
        y = -0.009 + 0.018 * t
        hh = 0.0046 * (1 - t) ** 0.9 + 0.0003
        ww = 0.0056 * (1 - t) ** 1.1 + 0.0002
        cx = -0.0008 * math.sin(math.pi * t * 0.9) * (1 - t) - 0.0005 * t * t
        sec = []
        for k in range(6):
            a = TAU * k / 6
            x, z = math.cos(a), math.sin(a)
            sec.append((cx + x * hh, y, z * ww * (1 - 0.25 * max(0.0, -x))))
        rings.append(sec)
    return loft(rings)

part('SmallCrows__beak', f_cone(0.005, 0.018), crow_beak, smooth=80)
part('SmallCrows__leg', f_cyl(0.002, 0.002, 0.024), lambda: lathe(
    [(0, -0.012), (0.0022, -0.012), (0.0020, -0.0100), (0.0015, -0.0085), (0.0016, -0.004),
     (0.0014, 0.004), (0.0019, 0.0075), (0.0020, 0.012), (0, 0.012)], seg=6), smooth=50)


# ---------------------------------------------------- small hourglasses --
# turned end plates (occ 0 is the top plate, its seat for the glass facing
# down; occ 1 the bottom plate, seat facing up)
def hg_plate(flip):
    s = lathe([(0, -0.003), (0.0330, -0.003), (0.0350, -0.0018), (0.0350, -0.0002), (0.0338, 0.0006),
               (0.0342, 0.0016), (0.0325, 0.0027), (0.0200, 0.0030), (0.0185, 0.0022),
               (0, 0.0022)], seg=16)
    return s.scale(1, -1 if flip else 1, 1)

part('SmallHourglasses__topPlate', f_cyl(0.035, 0.035, 0.006), lambda: hg_plate(True), smooth=45)
part('SmallHourglasses__botPlate', f_cyl(0.035, 0.035, 0.006), lambda: hg_plate(False), smooth=45)
# bead-and-reel turned post
part('SmallHourglasses__post', f_cyl(0.003, 0.003, 0.15), lambda: lathe(
    [(0, -0.075), (0.0024, -0.075), (0.0030, -0.071), (0.0020, -0.066), (0.0017, -0.050),
     (0.0028, -0.045), (0.0017, -0.040), (0.0015, -0.010), (0.0030, -0.004), (0.0030, 0.004),
     (0.0015, 0.010), (0.0017, 0.040), (0.0028, 0.045), (0.0017, 0.050), (0.0020, 0.066),
     (0.0030, 0.071), (0.0024, 0.075), (0, 0.075)], seg=8), smooth=50)


def sand_heap():
    # a heap of fine sand at its angle of repose: slightly hollow flanks, a
    # soft crest, a slightly domed base, a grainy surface
    s = lathe([(0, -0.0175), (0.0120, -0.0180), (0.0200, -0.0175), (0.0192, -0.0140), (0.0158, -0.0060),
               (0.0116, 0.0030), (0.0070, 0.0110), (0.0028, 0.0160), (0, 0.0175)], seg=12)
    return noisy(s, 0.0004, scale=300, seed=11)

part('SmallHourglasses__sand', f_cone(0.020, 0.035), sand_heap, smooth=70)


# --------------------------------------------------------- ouija boards --
def planchette():
    # heart-shaped planchette (pointer towards +Z, as the 3-sided cylinder's
    # corner): a domed body 6 mm thick with a round window near the point,
    # standing on three felt-tipped pegs
    out = []
    for i in range(18):
        a = TAU * i / 18
        # cardioid-like: round lobes at the back, drawn to a point at +Z
        x = math.sin(a)
        z = math.cos(a)
        k = 1.0 - 0.10 * math.cos(2 * a) + 0.10 * math.cos(a)
        px, pz = x * k, z * k
        if pz > 0:
            px *= 1 - 0.55 * pz ** 1.5
        out.append((px, pz))
    xs = [p[0] for p in out]
    zs = [p[1] for p in out]
    sx = 0.030 / max(abs(min(xs)), max(xs))
    cz = (max(zs) + min(zs)) / 2
    sz = 0.030 / ((max(zs) - min(zs)) / 2)
    pts = [(x * sx, (z - cz) * sz) for x, z in out]
    body = stack(pts, [(0, 0.0), (0, 0.0024), (0.0030, 0.0052), (0.0090, 0.0060)],
                 ref=0.030).rot(x=-HPI)
    body = cut(body, cyl(0.0060, 0.0060, 0.02, seg=8).move(z=0.0115))
    legs = Shape()
    for x, z in ((0.0, 0.020), (-0.017, -0.016), (0.017, -0.016)):
        legs = legs + lathe([(0, -0.006), (0.0024, -0.006), (0.0015, -0.0042), (0.0015, 0.0002)],
                            seg=5).move(x=x, z=z)
    return body + legs

part('OuijaBoards__planchette', f_cyl(0.030, 0.030, 0.012), planchette, smooth=40)


# -------------------------------------------------------- small candles --
def pillar_candle():
    # a pillar that has burned down: a crater inside a soft, uneven rim, two
    # runs of wax down the side, a slight lean in the melt
    h, r = 0.10, 0.020
    s = lathe([(0, -h / 2), (r * 0.98, -h / 2), (r, -h / 2 + 0.002), (r, h / 2 - 0.006), (r * 0.99, h / 2 - 0.0015),
               (r * 0.90, h / 2), (r * 0.80, h / 2 - 0.0012), (r * 0.55, h / 2 - 0.0040),
               (r * 0.22, h / 2 - 0.0048), (r * 0.08, h / 2 - 0.0020), (0, h / 2 - 0.0008)], seg=15)

    def w(p):
        a = math.atan2(p.x, p.z)
        rr = math.hypot(p.x, p.z)
        if rr > r * 0.7:
            k = 0.0
            for a0, top, bot in ((0.6, 0.05, 0.010), (2.9, 0.05, 0.028)):
                da = (a - a0 + math.pi) % TAU - math.pi
                if p.y > bot - h / 2:
                    k += math.exp(-(da / 0.28) ** 2) * min(1.0, (p.y - bot + h / 2) / 0.02)
            p.x *= 1 + 0.07 * min(1.0, k)
            p.z *= 1 + 0.07 * min(1.0, k)
            if p.y > h / 2 - 0.003:
                p.y += 0.0012 * math.sin(a * 3 + 0.8)
        return p
    return s.warp(w).fit((-r, -h / 2, -r), (r, h / 2, r))

part('SmallCandles__body', f_cyl(0.020, 0.020, 0.10), pillar_candle, smooth=60)


def wax_drip():
    # a run of wax: flattened, broad where it left the rim, ending in a bead
    s = lathe([(0, -0.015), (0.0010, -0.0150), (0.0014, -0.0138), (0.0012, -0.0120), (0.0014, -0.004),
               (0.0020, 0.006), (0.0028, 0.013), (0.0030, 0.015), (0, 0.015)], seg=8)
    return s.scale(1, 1, 0.8)

part('SmallCandles__drip', f_cyl(0.003, 0.003, 0.03), wax_drip, smooth=60)


# ================================================================ nature ==
def flower_pot(rt, rb, h, rim_h, lip=0.06, soil=0.08, seg=20):
    """Terracotta pot along Y: a thick rolled collar at the top, the body
    tapering to a foot with a bevel, the soil level sunk inside the collar."""
    y0, y1 = -h / 2, h / 2
    ry = y1 - rim_h
    body_top = rt * 0.93
    t = rt * lip
    prof = [(0, y0), (rb * 0.94, y0), (rb, y0 + h * 0.03), (body_top, ry - h * 0.02),
            (body_top + t * 0.2, ry), (rt * 0.985, ry + rim_h * 0.10), (rt, ry + rim_h * 0.3),
            (rt, y1 - rim_h * 0.15), (rt * 0.985, y1), (rt - t, y1), (rt - t * 1.05, y1 - h * 0.02),
            (rt - t * 1.1, y1 - h * soil), (0, y1 - h * soil)]
    return lathe(prof, seg=seg)

part('DeadPlants__pot', f_cyl(0.16, 0.16, 0.20),
     lambda: noisy(flower_pot(0.16, 0.12, 0.20, 0.045, lip=0.07, soil=0.06, seg=24), 0.0008, scale=25, seed=3),
     smooth=40)


def dead_stalk(L, r0, r1, nodes=3, seed=0, kink=0.006):
    """A dry stalk along Y: tapering, gently kinked at nodes (within `kink`),
    swollen nodes, a snapped tip."""
    rnd = random.Random(seed)
    pts, n = [], nodes + 1
    for i in range(n + 1):
        t = i / n
        off = 0.0 if i == 0 else kink * (rnd.random() - 0.5) * 2 * t
        pts.append((off, -L / 2 + L * t, kink * 0.4 * (rnd.random() - 0.5) * t))
    s = tube(pts, r0, seg=6, smooth=2, r_end=r1)
    for i in range(1, n):
        t = i / n
        x, y, z = pts[i]
        rr = r0 + (r1 - r0) * t
        s = s + lathe([(0, -rr * 1.5), (rr * 1.25, -rr * 0.3), (rr * 1.25, rr * 0.3), (0, rr * 1.5)], seg=6).move(x, y, z)
    return s

part('DeadPlants__stem', f_cyl(0.008, 0.008, 0.32), lambda: dead_stalk(0.32, 0.0074, 0.0050, 3, seed=2, kink=0.010),
     smooth=60)


def leaf_blade(L, W, pts_n=10, curl=0.0, cup=0.0, droop=0.0, crinkle=0.0, th=0.0006, midrib=0.0, seed=0):
    """A leaf along X (tip at +X, stalk at -X), W wide along Z, a thin slab
    with a pointed tip, curled edges (cup, +Y), drooping ends and crinkles."""
    nx, nz = pts_n, 4
    v, f = [], []

    def half_w(u):          # u 0..1 along the blade
        return W / 2 * (math.sin(math.pi * min(1.0, u * 1.08)) ** 0.8) * (1 - 0.15 * u)

    rnd = random.Random(seed)
    grid = []
    for side in (1, -1):
        for i in range(nx + 1):
            u = i / nx
            x = -L / 2 + L * u
            hw = max(half_w(u), W * 0.02)
            for j in range(nz + 1):
                zz = -hw + 2 * hw * j / nz
                e = abs(zz) / (W / 2)
                y = cup * e * e + droop * (2 * u - 1) ** 2 - midrib * math.exp(-(zz / (W * 0.06)) ** 2)
                y += crinkle * noise.noise(Vector((x * 90 + seed, zz * 90, seed * 3.1)))
                grid.append((x, y + side * th / 2, zz))
    k = nz + 1
    off = (nx + 1) * k
    for i in range(nx):
        for j in range(nz):
            a = i * k + j
            f.append((a, a + 1, a + k + 1, a + k))
            b = off + a
            f.append((b, b + k, b + k + 1, b + 1))
    # stitch the rim
    rim = [i * k for i in range(nx + 1)] + [nx * k + j for j in range(1, nz + 1)] + \
          [i * k + nz for i in range(nx - 1, -1, -1)] + [j for j in range(nz - 1, 0, -1)]
    for a, b in zip(rim, rim[1:] + rim[:1]):
        f.append((b, a, off + a, off + b))
    s = fixw(Shape(grid, f))
    if curl:
        s = bend(s, curl, 'z', 'x')
    return s


def dry_leaf(L, W, t, seed=0):
    # a withered leaf: edges curled up, a sunk midrib, crinkled
    s = leaf_blade(L, W, 10, cup=t * 0.55, droop=-t * 0.25, crinkle=t * 0.12, th=t * 0.18, midrib=t * 0.2,
                   seed=seed)
    return s.fit((-L / 2, -t / 2, -W / 2), (L / 2, t / 2, W / 2))

part('DeadPlants__leaf', f_box(0.04, 0.005, 0.025), lambda: dry_leaf(0.04, 0.025, 0.005, seed=1), smooth=60)


def fallen_leaf(w, h, seed=0):
    # a dead leaf flat on the floor, facing +Z; its curled rim reaches z = 0
    # and the rest sits just behind (under) it
    s = leaf_blade(w, h, 10, cup=0.00055, droop=0.0002, crinkle=0.0002, th=0.0003, midrib=0.0002, seed=seed)
    s.rot(x=HPI)            # leaf plane XZ -> XY, cup +Y -> +Z
    lo, hi = s.bounds()
    return s.move(z=-hi.z)

part('DeadPlants__fallenLeaf', f_bounds((-0.025, -0.0175, 0), (0.025, 0.0175, 0), 'plane'),
     lambda: fallen_leaf(0.05, 0.035, seed=4), smooth=60)


# ----------------------------------------------------- dead flower vases --
part('DeadFlowerVases__body', f_cyl(0.035, 0.035, 0.070), lambda: lathe(
    [(0, -0.035), (0.0262, -0.035), (0.0280, -0.0335), (0.0270, -0.0310), (0.0268, -0.0290),
     (0.0290, -0.0230), (0.0320, -0.0120), (0.0342, 0.0020), (0.0350, 0.0140), (0.0340, 0.0240),
     (0.0310, 0.0310), (0.0285, 0.0350), (0, 0.0350)], seg=20), smooth=50)
part('DeadFlowerVases__neck', f_cyl(0.028, 0.028, 0.020), lambda: vessel(
    [(0.0280, -0.010), (0.0228, -0.0030), (0.0214, 0.0040), (0.0245, 0.0085), (0.0262, 0.0100)],
    0.0030, seg=12), smooth=50)
part('DeadFlowerVases__stem', f_cyl(0.0015, 0.0015, 0.11), lambda: dead_stalk(0.11, 0.0014, 0.0009, 2, seed=5,
     kink=0.0025), smooth=60)


# -------------------------------------------------------- potted plants --
part('PottedPlants__pot', f_cyl(0.055, 0.055, 0.08),
     lambda: flower_pot(0.055, 0.045, 0.08, 0.020, lip=0.08, soil=0.05, seg=20), smooth=40)


# -------------------------------------------------------- flower stems --
# a cut stem lying on the floor: a slight curve along the floor, two nodes
def cut_stem():
    s = bend(dead_stalk(0.20, 0.0019, 0.0015, 2, seed=8, kink=0.0), 0.08, 'x', 'y')
    lo, hi = s.bounds()
    return s.move(y=-(lo.y + hi.y) / 2, z=-(lo.z + hi.z) / 2)

part('FlowerStems__stem', f_cyl(0.002, 0.002, 0.20), cut_stem, smooth=60)


def lance_leaf():
    # a fresh leaf lying flat (plane frame, facing +Z): lance-shaped, a sunk
    # midrib, the edges lifting a little
    s = leaf_blade(0.030, 0.014, 8, cup=0.00042, droop=0.0001, crinkle=0.0, th=0.00018, midrib=0.00015)
    s.rot(x=HPI)
    lo, hi = s.bounds()
    return s.move(z=-hi.z)

part('FlowerStems__leaf', f_bounds((-0.015, -0.007, 0), (0.015, 0.007, 0), 'plane'), lance_leaf, smooth=60)


# ----------------------------------------------------------- dirt clods --
def clod():
    # a crumbly lump of dry soil: lumpy, flattened a little, with sharp
    # broken facets
    s = subdivide(sphere(0.014, 8, 4), 1)
    s = noisy(s, 0.0035, scale=110, seed=13)
    s = noisy(s, 0.0012, scale=420, seed=2)
    return s.fit((-0.014, -0.014, -0.014), (0.014, 0.014, 0.014))

part('DirtClods__clod', f_bounds((-0.014, -0.014, -0.014), (0.014, 0.014, 0.014)), clod, smooth=55)


# -------------------------------------------------------------- pebbles --
def pebble():
    # a water-worn pebble: smooth, oblong, one side flatter
    s = subdivide(sphere(0.01, 8, 4), 1)
    s = s.warp(lambda p: Vector((p.x * 1.0, p.y * (0.8 if p.y > 0 else 0.6), p.z * 0.9)))
    s = noisy(s, 0.0012, scale=90, seed=21)
    return s.fit((-0.01, -0.01, -0.01), (0.01, 0.01, 0.01))

part('Pebbles__pebble', f_sphere(0.010), pebble, smooth=180)


# ---------------------------------------------------------------- twigs --
def twig():
    # a dead twig: tapering, a bend or two, bark knobs, a side stub, both ends
    # snapped
    L, r0, r1 = 0.07, 0.0033, 0.0023
    pts = [(0.0, -L / 2, 0.0), (0.0006, -L / 6, 0.0004), (-0.0006, L / 6, -0.0003), (0.0004, L / 2, 0.0)]
    s = tube(pts, r0, seg=6, smooth=2, r_end=r1)
    s = noisy(s, 0.00025, scale=900, seed=6)
    stub = tube([(0.0, 0.004, 0.0), (0.0024, 0.0085, 0.0), (0.0035, 0.011, 0.0)], 0.0011, seg=5, r_end=0.0008)
    knot = lathe([(0, -0.0035), (r0 * 1.12, -0.0012), (r0 * 1.12, 0.0012), (0, 0.0035)], seg=6).move(y=-0.018)
    return s + stub + knot

part('Twigs__twig', f_cyl(0.0023, 0.0033, 0.07), twig, smooth=60)


# --------------------------------------------------------------- acorns --
def acorn_nut():
    # the nut, in the unscaled sphere frame (stretched 1.3 in y): full
    # shoulders under the cap, drawing to a point with a small nipple
    s = lathe([(0, -0.011), (0.0008, -0.0108), (0.0016, -0.0098), (0.0050, -0.0088), (0.0085, -0.0060),
               (0.0104, -0.0025), (0.0110, 0.0015), (0.0106, 0.0055), (0.0092, 0.0085), (0.0060, 0.0104),
               (0, 0.011)], seg=12)
    return s

part('Acorns__nut', f_sphere(0.011), acorn_nut, smooth=60)


def acorn_cap():
    # the cupule: rows of overlapping scales stepping up a cup with a thick
    # rim, hollow underneath where the nut sits, a stalk scar on top
    out = [(0.0090, 0.0), (0.0110, 0.0004), (0.0112, 0.0022), (0.0104, 0.0032), (0.0106, 0.0048),
           (0.0094, 0.0060), (0.0092, 0.0074), (0.0074, 0.0086), (0.0066, 0.0098), (0.0036, 0.0106),
           (0.0016, 0.0110), (0, 0.0110)]
    s = lathe(out, seg=10, cap_bottom=False)
    inner = lathe([(0.0090, 0.0), (0.0078, 0.0034), (0.0052, 0.0064), (0, 0.0074)], seg=10, cap_bottom=False)
    inner.f = [tuple(reversed(f)) for f in inner.f]
    s = weld(s + inner)
    s = noisy(s, 0.00025, scale=900, seed=9).fit((-0.011, 0.0, -0.011), (0.011, 0.0094, 0.011))
    # the stalk stub the placer's own stalk (buried in the cup) should be
    stalk = tube([(0, 0.0086, 0), (0.0005, 0.0110, 0.0002)], 0.0011, seg=5, r_end=0.0008)
    return s + stalk

part('Acorns__cap', f_bounds((-0.011, 0.0, -0.011), (0.011, 0.011, 0.011)), acorn_cap, smooth=75)


# ------------------------------------------------------------ pine cones --
def cone_core():
    # the cone's body under the scales (axis +Y = the fat end, as the placer
    # lays it): spiralled rows of scale tips, a stalk at the fat end, the
    # narrow end closing to a point
    s = lathe([(0, -0.0275), (0.0030, -0.0270), (0.0060, -0.0230), (0.0080, -0.0140), (0.0098, -0.0040),
               (0.0112, 0.0060), (0.0120, 0.0150), (0.0116, 0.0220), (0.0080, 0.0260), (0.0030, 0.0270),
               (0.0022, 0.0275), (0, 0.0275)], seg=14)

    def scales(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6 or p.y > 0.024 or p.y < -0.026:
            return p
        a = math.atan2(p.x, p.z)
        u = math.cos(7 * a + p.y * 520) * math.cos(7 * a - p.y * 520)
        k = 1 + 0.10 * max(0.0, u)
        p.x *= k
        p.z *= k
        return p
    return s.warp(scales).fit((-0.012, -0.0275, -0.012), (0.012, 0.0275, 0.012))

part('PineCones__core', f_cyl(0.012, 0.012, 0.055), cone_core, smooth=60)


def cone_scale():
    # one woody scale (local +Y points out from the cone): a thin stalk
    # widening into a thick diamond apophysis with a transverse keel and a
    # small umbo in the middle
    rings = []
    for y, sx, sz in ((-0.0025, 0.0022, 0.0030), (-0.0005, 0.0030, 0.0040), (0.0012, 0.0030, 0.0040),
                      (0.0022, 0.0020, 0.0028), (0.0025, 0.0006, 0.0010)):
        ring = []
        for k in range(8):
            a = TAU * k / 8
            c, sn = math.cos(a), math.sin(a)
            # a diamond (rhombus) with softened corners
            d = 1.0 / (abs(c) + abs(sn)) ** 0.7
            ring.append((c * sx * d, y, sn * sz * d))
        rings.append(ring)
    s = loft(rings)
    return s.fit((-0.003, -0.0025, -0.004), (0.003, 0.0025, 0.004))

part('PineCones__scale', f_box(0.006, 0.005, 0.008), cone_scale, smooth=50)


# ------------------------------------------------------------- feathers --
def quill():
    # the feather's shaft (tip +Y): a hollow-looking bare calamus at the
    # base, the rachis tapering to a fine tip, curving slightly
    s = lathe([(0, -0.050), (0.0011, -0.050), (0.0015, -0.0485), (0.0015, -0.030), (0.0013, -0.026),
               (0.0012, 0.0), (0.0008, 0.030), (0.0004, 0.049), (0, 0.050)], seg=6)
    s = bend(s, 0.10, 'x', 'y')
    lo, hi = s.bounds()
    return s.move(z=-(lo.z + hi.z) / 2, y=-(lo.y + hi.y) / 2)

part('Feathers__shaft', f_cyl(0.0015, 0.0015, 0.10), quill, smooth=60)


# ------------------------------------------------------------ mushrooms --
def mush_stem():
    # the stipe: a swollen base, fibrous, narrowing up to a ragged ring
    # (annulus) under the cap
    s = lathe([(0, -0.015), (0.0055, -0.015), (0.0070, -0.0125), (0.0068, -0.0085), (0.0056, -0.0030),
               (0.0050, 0.0060), (0.0048, 0.0095), (0.0062, 0.0112), (0.0064, 0.0118), (0.0048, 0.0126),
               (0.0046, 0.0150), (0, 0.0150)], seg=10)
    return noisy(s, 0.00018, scale=700, seed=4)

part('Mushrooms__stem', f_cyl(0.007, 0.007, 0.030), mush_stem, smooth=60)


def mush_cap():
    # convex cap with an in-rolled margin; underneath, radiating gills round
    # the stipe apex
    R = 0.014
    s = lathe([(0, -0.0003), (0.0060, 0.0016), (0.0112, 0.0010), (0.0134, 0.0004), (0.0140, 0.0024),
               (0.0134, 0.0060), (0.0112, 0.0096), (0.0072, 0.0124), (0.0026, 0.0138), (0, 0.0140)], seg=16)

    def gills(p):
        if p.y < 0.0021:
            r = math.hypot(p.x, p.z)
            if 0.0035 < r < 0.0128:
                a = math.atan2(p.x, p.z)
                p.y -= 0.0007 * max(0.0, math.cos(8 * a)) ** 2
        return p
    return s.warp(gills)

part('Mushrooms__cap', f_bounds((-0.014, 0.0, -0.014), (0.014, 0.014, 0.014)), mush_cap, smooth=60)
