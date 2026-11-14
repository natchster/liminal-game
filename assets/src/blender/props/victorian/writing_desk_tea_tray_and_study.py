"""b20 — victorian: the writing desk, the tea tray, the study walls.

Inkwell and letter, silver tea tray (cups, saucers, spoon, biscuits, sugar
bowl), antique map and botanical print frames, open pocket watch on its
chain, wrought-iron fireside tool stand, glass-dome specimens (bird,
butterfly, beetles), secretary desk, tarnished gilt mirror, dust-sheeted
furniture and the servant's bell pull.

Glass (inkwell jar, specimen dome), liquids, DoubleSide canvases/wings,
flat decals and sub-centimetre ticks stay primitives; see docs/research/props-remodelling/writing_desk_tea_tray_and_study.md.
"""
import propkit as _pk
from propkit import *


# ================================================================ helpers ==
def _fw(s):
    return _pk._fix_winding(s)


def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


def ax_x(s):
    """A shape built along +Y, laid along +X (+Y end goes to +X)."""
    return s.rot(z=-math.pi / 2)


def loft(rings, cap0=True, cap1=True):
    """Skin closed loops of 3D points (equal counts) into one solid."""
    n = len(rings[0])
    v, f = [], []
    for ring in rings:
        v += [Vector(p) for p in ring]
    for k in range(len(rings) - 1):
        a, b = k * n, (k + 1) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    if cap0:
        f.append(tuple(range(n)))
    if cap1:
        f.append(tuple(range((len(rings) - 1) * n, len(rings) * n)))
    return _fw(Shape(v, f))


def circ(r, n=8, sx=1.0, sy=1.0, phase=0.0):
    return [(r * sx * math.cos(TAU * i / n + phase), r * sy * math.sin(TAU * i / n + phase)) for i in range(n)]


def ring_section(R, sec, seg=32, arc=TAU, rmod=None, twist=0.0):
    """Sweep a closed section [(dr, dz), ...] around the Z axis in the XY
    plane (a torus_geo! with any cross-section). rmod(a) -> (k, dz, dR)
    wobbles the section along the ring; twist turns it (turns per ring)."""
    full = abs(arc - TAU) < 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        k, dzz, dR = rmod(a) if rmod else (1.0, 0.0, 0.0)
        th = twist * a
        ct, st = math.cos(th), math.sin(th)
        for (dr, dz) in sec:
            r2, z2 = (dr * ct - dz * st) * k, (dr * st + dz * ct) * k
            v.append(((R + dR + r2) * ca, (R + dR + r2) * sa, z2 + dzz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


def torus_arc_bounds(R, r, arc, radial=24, tubular=128):
    """Ideal bounds of torus_geo!(R, r, .., .., arc) as the game
    computes them (24 x 128 samples)."""
    xs, ys = [], []
    for i in range(tubular + 1):
        u = arc * i / tubular
        for j in range(radial + 1):
            w = TAU * j / radial
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r))


def rrect_pts(w, d, r, cseg=3):
    """Rounded rectangle (x, z) outline with 4*(cseg+1) points."""
    r = max(1e-5, min(r, w / 2 - 1e-5, d / 2 - 1e-5))
    out = []
    for cx, cz, a0 in ((w / 2 - r, d / 2 - r, 0), (-w / 2 + r, d / 2 - r, 90),
                       (-w / 2 + r, -d / 2 + r, 180), (w / 2 - r, -d / 2 + r, 270)):
        for i in range(cseg + 1):
            a = math.radians(a0 + 90 * i / cseg)
            out.append((cx + r * math.cos(a), cz + r * math.sin(a)))
    return out


def flutes(n, amt, y0, y1, phase=0.0):
    """Warp: radius * (1 - amt * ...) in n vertical flutes between y0..y1."""
    def w(p):
        if y0 <= p.y <= y1:
            k = 1 - amt * (0.5 + 0.5 * math.cos(n * math.atan2(p.x, p.z) + phase))
            p.x *= k
            p.z *= k
        return p
    return w


# ========================================================== inkwell letter ==
def letter_sheet():
    # a tri-folded letter opened out: one mountain and one valley crease
    # across it, the long edges lifting a little off the floor
    s = slab(0.16, 0.20, 0.0010, 6, 12).rot(x=-math.pi / 2)
    def w(p):
        z, x = p.z, p.x
        y = 0.0006 * math.exp(-((z - 0.0333) / 0.012) ** 2)
        y += 0.0005 * max(0.0, abs(z) - 0.06) / 0.04
        y += 0.0004 * (x / 0.08) ** 2 * (abs(z) / 0.1)
        p.y += y
        return p
    s.warp(w)
    lo, hi = s.bounds()
    return s.move(y=-0.001 - lo[1])

part('InkwellLetters__sheet', f_box(0.16, 0.002, 0.20), letter_sheet)

# barrel: +Y end (r 6 mm) is the cap end, -Y end meets the nib section
part('InkwellLetters__barrel', f_cyl(0.006, 0.005, 0.11), lambda: lathe(
    [(0, -0.055), (0.0038, -0.055), (0.0041, -0.052), (0.0045, -0.040), (0.0051, -0.034),
     (0.0049, -0.031), (0.0054, -0.028), (0.0058, 0.0), (0.0060, 0.040), (0.0057, 0.052),
     (0.0040, 0.0548), (0, 0.055)], seg=10), smooth=50)

# metal end cap: open lip towards the barrel (-Y), engine-turned band, domed end (+Y)
part('InkwellLetters__capband', f_cyl(0.0065, 0.0065, 0.020), lambda: lathe(
    [(0, -0.010), (0.0062, -0.010), (0.0065, -0.0085), (0.0061, -0.004), (0.0065, 0.004),
     (0.0055, 0.0088), (0, 0.010)], seg=10), smooth=45)


def pen_clip():
    # a spring clip rising from the cap end (-X), ball foot at the free end
    out = [(-0.008, -0.0015), (-0.0065, 0.0015), (0.0045, 0.0012), (0.0055, 0.0004),
           (0.0045, -0.0001), (-0.0055, 0.0002), (-0.0065, -0.0015)]
    strip = prism(out, 0.0019)
    ball = sphere(0.0015, seg=8, rings=4).move(x=0.0065, y=-0.0001)
    return strip + ball

part('InkwellLetters__clip', f_box(0.016, 0.003, 0.003), pen_clip, smooth=50)


def pen_nib():
    # the section collar (-Y) and a curved, pointed gold nib whose convex
    # side faces local -X (world up once the pen is laid along +X)
    collar = lathe([(0, -0.007), (0.0040, -0.0068), (0.0036, -0.0046), (0, -0.0046)], seg=8)
    rings = []
    for i in range(5):
        t = i / 4
        y = -0.0048 + 0.0118 * t
        r = 0.0035 * (1 - t) + 0.0003 * t
        half = math.radians(95 * (1 - t) + 18 * t)
        th = 0.00045 * (1 - 0.5 * t)
        ring = []
        for a in (-half, 0.0, half):
            ring.append((-r * math.cos(a), y, r * math.sin(a)))
        for a in (half, 0.0, -half):
            ring.append((-(r - th) * math.cos(a), y, (r - th) * math.sin(a)))
        rings.append(ring)
    return collar + loft(rings)

part('InkwellLetters__nib', f_cyl(0.0015, 0.004, 0.014), pen_nib, smooth=40)

# the ink drop swelling under the nib: pointed where it clings (+Y)
part('InkwellLetters__drop', f_sphere(0.002), lambda: lathe(
    [(0, -0.002), (0.0012, -0.0017), (0.0019, -0.0009), (0.0020, -0.0001), (0.0014, 0.0009),
     (0.0006, 0.0017), (0, 0.002)], seg=8), smooth=180)

# brass collar round the jar mouth: flat band with a rounded bead
part('InkwellLetters__neck', f_torus(0.024, 0.002), lambda: ring_section(
    0.024, [(-0.0012, -0.002), (0.0010, -0.002), (0.002, -0.0007), (0.002, 0.0007),
            (0.0010, 0.002), (-0.0012, 0.002)], seg=22))


def ink_cap():
    # screw cap set down beside the jar: knurled skirt, stepped top, button
    s = lathe([(0, -0.004), (0.0235, -0.004), (0.025, -0.0032), (0.025, 0.0016), (0.0238, 0.0026),
               (0.0205, 0.0030), (0.0060, 0.0031), (0.0048, 0.0040), (0, 0.0040)], seg=20)
    return s.warp(flutes(10, 0.035, -0.0035, 0.0017))

part('InkwellLetters__cap', f_cyl(0.025, 0.025, 0.008), ink_cap)


# ============================================================ silver tray ==
# round salver: flat top (the tea things stand on it), a shallow step to the
# border and a foot ring underneath; the rim torus covers the edge
part('SilverTrays__tray', f_cyl(0.14, 0.14, 0.008), lambda: lathe(
    [(0, -0.0028), (0.108, -0.0028), (0.112, -0.004), (0.118, -0.004), (0.121, -0.0022),
     (0.138, -0.0015), (0.140, 0.0005), (0.137, 0.004), (0.128, 0.004), (0.124, 0.0036),
     (0, 0.0036)], seg=30))


def gadroon_rim():
    # a gadrooned border: 18 swelling lobes round the edge
    sec = circ(0.006, 6, sy=0.85, phase=math.pi / 6)
    def mod(a):
        u = (a * 18 / TAU) % 1.0
        return (0.80 + 0.22 * math.sin(math.pi * u) ** 0.8, 0.0, 0.0)
    return ring_section(0.14, sec, seg=90, rmod=mod)

part('SilverTrays__rim', f_torus(0.14, 0.006), gadroon_rim)

F_TRAY_HANDLE = torus_arc_bounds(0.020, 0.004, math.pi)


def tray_handle():
    # cast loop handle: D section, swelling into leaf bosses at both feet
    sec = [(-0.0034, -0.0028), (0.0, -0.0036), (0.0036, -0.0024), (0.0036, 0.0024),
           (0.0, 0.0036), (-0.0034, 0.0028)]
    def mod(a):
        e = min(a, math.pi - a) / math.pi
        return (1.0 + 0.12 * max(0.0, 1 - e / 0.12) - 0.1 * math.sin(a) ** 6, 0.0, 0.0)
    return fitf(ring_section(0.020, sec, seg=16, arc=math.pi, rmod=mod), F_TRAY_HANDLE)

part('SilverTrays__handle', F_TRAY_HANDLE, tray_handle)

# engraved line round the centre: a fine twisted rope
part('SilverTrays__engraving', f_torus(0.063, 0.001), lambda: ring_section(
    0.063, [(-0.001, -0.001), (0.001, -0.001), (0.001, 0.001), (-0.001, 0.001)], seg=60, twist=30))


def teacup():
    # footed porcelain cup, hollow so the tea shows 3 mm under the lip
    return vessel([(0.0125, -0.016), (0.0130, -0.0135), (0.0165, -0.0110), (0.0200, -0.0060),
                   (0.0225, 0.0030), (0.0236, 0.0110), (0.0240, 0.0155)], wall=0.0016, seg=14,
                  floor=0.026)

part('SilverTrays__cup', f_cyl(0.024, 0.020, 0.032), teacup, smooth=50)

# saucer: sunk ring for the cup, marly rising to the rim, foot ring below
part('SilverTrays__saucer', f_cyl(0.038, 0.038, 0.003), lambda: lathe(
    [(0, -0.0009), (0.0165, -0.0009), (0.0185, -0.0015), (0.021, -0.0008), (0.037, 0.0002),
     (0.038, 0.0012), (0.0368, 0.0015), (0.0215, 0.0002), (0, 0.0)], seg=18), smooth=40)


def cup_handle():
    # porcelain ear: thicker thumb rest at the top, thin lower terminal
    sec = [(-0.0017, 0.0), (0.0, -0.002), (0.0017, 0.0), (0.0, 0.002)]
    def mod(a):
        return (0.85 + 0.25 * max(0.0, math.sin(a)), 0.0, 0.0)
    return ring_section(0.008, sec, seg=12, rmod=mod)

part('SilverTrays__cupHandle', f_torus(0.008, 0.002), cup_handle, smooth=60)

# teaspoon stem: thin neck at the bowl (-Y), flat fiddle terminal (+Y)
part('SilverTrays__spoon', f_cyl(0.0015, 0.0015, 0.030), lambda: lathe(
    [(0, -0.015), (0.0007, -0.015), (0.0006, -0.011), (0.0008, -0.004), (0.0011, 0.006),
     (0.0015, 0.0115), (0.0013, 0.0142), (0, 0.015)], seg=8).scale(1, 1, 0.62), smooth=60)


def spoon_bowl():
    # a shallow egg-shaped bowl, narrow end towards the stem (-X); built at
    # its true size (the game squashes the sphere 1.4 x 0.4 x 1.0)
    s = lathe([(0, -0.0016), (0.6, -0.0012), (1.0, 0.0), (0.93, 0.0016), (0.55, 0.0007), (0, 0.0004)], seg=8)
    s.warp(lambda p: Vector((p.x * 0.0056, p.y, p.z * 0.004 * (1 - 0.3 * max(0.0, -p.x)))))
    return fitf(s, f_sphere(0.004))

part('SilverTrays__spoonBowl', f_sphere(0.004), spoon_bowl, smooth=70)

# side plate: flat well, wide marly, rolled edge, foot ring
part('SilverTrays__plate', f_cyl(0.040, 0.040, 0.003), lambda: lathe(
    [(0, -0.0010), (0.0225, -0.0010), (0.024, -0.0015), (0.0265, -0.0008), (0.039, 0.0004),
     (0.040, 0.0012), (0.0388, 0.0015), (0.027, 0.0003), (0, 0.0)], seg=18), smooth=40)


def biscuit():
    # shortbread round: fluted edge, softly domed and crumbly top
    s = lathe([(0, -0.0020), (0.0100, -0.0020), (0.0110, -0.0011), (0.0110, 0.0009),
               (0.0100, 0.0018), (0.0060, 0.0020), (0, 0.0021)], seg=24)
    def sc(p):
        a = math.atan2(p.x, p.z)
        r = math.hypot(p.x, p.z)
        if r > 0.0085:
            k = 1 - 0.07 * abs(math.sin(6 * a))
            p.x *= k
            p.z *= k
        return p
    return noisy(s.warp(sc), 0.00018, scale=900, seed=4)

part('SilverTrays__biscuit', f_cyl(0.011, 0.011, 0.004), biscuit, smooth=70)

# sugar bowl: foot ring, bellied body, shoulder and a gallery for the lid
part('SilverTrays__sugar', f_cyl(0.018, 0.020, 0.024), lambda: lathe(
    [(0, -0.012), (0.0120, -0.012), (0.0125, -0.0105), (0.0112, -0.0092), (0.0160, -0.0065),
     (0.0198, -0.0010), (0.0190, 0.0055), (0.0162, 0.0090), (0.0155, 0.0105), (0.0170, 0.0120),
     (0.0150, 0.0118), (0, 0.0110)], seg=14), smooth=50)

F_SUGAR_LID = f_bounds((-0.019021, 0.0061803, -0.019021), (0.019021, 0.020, 0.019021))
part('SilverTrays__lid', F_SUGAR_LID, lambda: lathe(
    [(0, 0.00618), (0.0190, 0.00618), (0.0190, 0.0074), (0.0168, 0.0086), (0.0152, 0.0125),
     (0.0110, 0.0164), (0.0055, 0.0186), (0.0040, 0.0194), (0, 0.0200)], seg=16), smooth=50)

# turned knob finial with a neck
part('SilverTrays__knob', f_sphere(0.004), lambda: lathe(
    [(0, -0.004), (0.0016, -0.004), (0.0011, -0.0026), (0.0032, -0.0012), (0.0040, 0.0006),
     (0.0028, 0.0030), (0, 0.004)], seg=8), smooth=60)


def napkin():
    # linen napkin folded in quarters: two soft layers, the top one shorter
    low = rbox(0.045, 0.0026, 0.030, r=0.0012, seg=2).move(y=-0.0012)
    top = rbox(0.034, 0.0024, 0.029, r=0.0011, seg=2).move(x=-0.0055, y=0.0013)
    return low + top

part('SilverTrays__napkin', f_box(0.045, 0.005, 0.030), napkin, smooth=60)


# ========================================================== frame helpers ==
def _interp(tab, e):
    for (e0, h0), (e1, h1) in zip(tab, tab[1:]):
        if e <= e1:
            t = (e - e0) / (e1 - e0) if e1 > e0 else 0.0
            t = t * t * (3 - 2 * t)
            return h0 + (h1 - h0) * t
    return tab[-1][1]


def crown_bar(w, h, t, tab):
    """A picture-frame bar. One constructor makes both the long top/bottom
    bars and the tall side bars (the game stretches the part per axis), so
    the face is symmetric in X and Y: height = H(u) * H(v), where
    H(edge distance) is the moulding profile `tab` [(e, h), ...] with e = 0
    at the edge and 1 on the centre line. Back face flat at -t/2."""
    es = sorted(set(e for e, _ in tab))
    us = [-(1 - e) for e in es] + [1 - e for e in reversed(es) if e < 1]
    n = len(us)
    H = lambda u: _interp(tab, 1 - abs(u))
    v, f = [], []
    for j, vv in enumerate(us):
        for i, uu in enumerate(us):
            v.append((uu * w / 2, vv * h / 2, -t / 2 + t * H(uu) * H(vv)))
    idx = lambda i, j: j * n + i
    for j in range(n - 1):
        for i in range(n - 1):
            f.append((idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1)))
    rim = [idx(i, 0) for i in range(n)] + [idx(n - 1, j) for j in range(1, n)] + \
          [idx(i, n - 1) for i in range(n - 2, -1, -1)] + [idx(0, j) for j in range(n - 2, 0, -1)]
    base = len(v)
    for k in rim:
        x, y, _ = v[k]
        v.append((x, y, -t / 2))
    m = len(rim)
    for a in range(m):
        b = (a + 1) % m
        f.append((rim[b], rim[a], base + a, base + b))
    f.append(tuple(base + a for a in reversed(range(m))))
    return _fw(Shape(v, f))


# ============================================================ antique map ==
# walnut frame: rounded outer edge, a bead, a small cove, then a crowned face
part('AntiqueMaps__bar', f_box(0.46, 0.02, 0.010), lambda: crown_bar(
    0.46, 0.02, 0.010, [(0.0, 0.30), (0.025, 0.66), (0.06, 0.80), (0.11, 0.70), (0.2, 0.86),
                        (0.45, 0.98), (1.0, 1.0)]), smooth=40)


# =========================================================== pocket watch ==
# case: domed back (-Y), middle band, stepped bezel seat at the front (+Y)
part('PocketWatches__body', f_cyl(0.030, 0.030, 0.010), lambda: lathe(
    [(0, -0.005), (0.016, -0.0047), (0.026, -0.0033), (0.0300, -0.0008), (0.0300, 0.0012),
     (0.0285, 0.0042), (0.0262, 0.0050), (0, 0.0050)], seg=22), smooth=40)


def watch_bezel():
    # half-round bezel with a milled (coin-edge) outer face
    sec = [(-0.0015, -0.002), (0.0012, -0.0017), (0.002, 0.0), (0.0012, 0.0017), (-0.0015, 0.002)]
    def mod(a):
        return (1.0 if int(round(a * 28 / TAU)) % 2 == 0 else 0.9, 0.0, 0.0)
    return ring_section(0.030, sec, seg=28, rmod=mod)

part('PocketWatches__rim', f_torus(0.030, 0.002), watch_bezel)


def flat(outline, t):
    """Extrude a top-view outline [(x, z), ...] t thick along Y, centred."""
    return prism([(x, -z) for x, z in outline], t).rot(x=-math.pi / 2)


def hand(len_, w, spade):
    """A watch hand lying flat: pivot boss at +Z, tip at -Z."""
    z0, z1 = len_ / 2, -len_ / 2
    rb = w / 2
    cz = z0 - rb
    boss = [(rb * math.cos(a), cz + rb * math.sin(a)) for a in
            [math.radians(d) for d in (-60, -20, 20, 60, 90, 120, 160, 200, 240)]]
    sw = w * 0.16
    if spade:
        body = [(-sw, cz - rb * 0.7), (-sw, z1 + len_ * 0.42), (-w * 0.5, z1 + len_ * 0.28),
                (-sw * 0.6, z1 + len_ * 0.08), (0.0, z1), (sw * 0.6, z1 + len_ * 0.08),
                (w * 0.5, z1 + len_ * 0.28), (sw, z1 + len_ * 0.42), (sw, cz - rb * 0.7)]
    else:
        body = [(-sw * 1.4, cz - rb * 0.7), (-sw * 0.4, z1 + len_ * 0.05), (0.0, z1),
                (sw * 0.4, z1 + len_ * 0.05), (sw * 1.4, cz - rb * 0.7)]
    out = body + [p for p in boss if p[1] > cz - rb * 0.7 or abs(p[0]) > sw * 1.4]
    return flat(out, 0.0008)

part('PocketWatches__hourHand', f_box(0.0018, 0.0008, 0.014), lambda: hand(0.014, 0.0018, True))
part('PocketWatches__minHand', f_box(0.0012, 0.0008, 0.020), lambda: hand(0.020, 0.0012, False))

# cannon pinion: a domed brass boss
part('PocketWatches__pin', f_cyl(0.0015, 0.0015, 0.002), lambda: lathe(
    [(0, -0.001), (0.0015, -0.001), (0.0015, -0.0002), (0.0009, 0.0007), (0, 0.001)], seg=8), smooth=50)


def watch_crown():
    # fluted onion crown on a short neck
    s = lathe([(0, -0.0025), (0.0021, -0.0025), (0.0033, -0.0012), (0.0035, 0.0004),
               (0.0029, 0.0018), (0, 0.0025)], seg=10)
    return s.warp(flutes(5, 0.12, -0.0015, 0.0022))

part('PocketWatches__crown', f_cyl(0.0035, 0.0035, 0.005), watch_crown)

# hunter lid: domed outside (-Y), snap lip round the inside face (+Y)
part('PocketWatches__cover', f_cyl(0.030, 0.030, 0.003), lambda: lathe(
    [(0, -0.0015), (0.016, -0.0012), (0.026, -0.0006), (0.0300, 0.0004), (0.0300, 0.0015),
     (0.0282, 0.0010), (0, 0.0008)], seg=20), smooth=40)

# curb link: a round link with a twisted, flattened section
part('PocketWatches__link', f_torus(0.004, 0.001), lambda: ring_section(
    0.004, [(-0.001, -0.0006), (0.001, -0.0006), (0.001, 0.0006), (-0.001, 0.0006)], seg=10,
    twist=1.0), smooth=60)

# T-bar: turned bar with ball ends and a centre boss, laid along X
part('PocketWatches__fob', f_box(0.014, 0.004, 0.004), lambda: fitf(ax_x(lathe(
    [(0, -0.007), (0.0017, -0.0058), (0.0009, -0.0043), (0.0009, -0.0014), (0.0015, 0.0),
     (0.0009, 0.0014), (0.0009, 0.0043), (0.0017, 0.0058), (0, 0.007)], seg=6)),
    f_box(0.014, 0.004, 0.004)), smooth=60)

# watch key: pipe end (-Y) and a collar under the bow (+Y)
part('PocketWatches__keyShaft', f_cyl(0.0012, 0.0012, 0.014), lambda: lathe(
    [(0, -0.007), (0.0010, -0.007), (0.0011, -0.0055), (0.0008, -0.0040), (0.0008, 0.0040),
     (0.0012, 0.0055), (0.0010, 0.0068), (0, 0.007)], seg=6), smooth=50)


def key_bow():
    # stamped trefoil bow: flat section swelling into three lobes
    sec = [(-0.001, -0.0006), (0.001, -0.0006), (0.001, 0.0006), (-0.001, 0.0006)]
    def mod(a):
        return (0.85 + 0.15 * math.cos(3 * a), 0.0, 0.0003 * math.cos(3 * a))
    return fitf(ring_section(0.005, sec, seg=12, rmod=mod), f_torus(0.005, 0.001))

part('PocketWatches__keyHead', f_torus(0.005, 0.001), key_bow)
part('PocketWatches__keyBit', f_box(0.0025, 0.002, 0.0025), lambda: rbox(0.0025, 0.002, 0.0025, r=0.0004, seg=1))


# ======================================================= botanical print ==
# slim oak frame: rounded edge, fine bead, flat face
part('BotanicalPrints__bar', f_box(0.308, 0.014, 0.010), lambda: crown_bar(
    0.308, 0.014, 0.010, [(0.0, 0.35), (0.03, 0.72), (0.07, 0.86), (0.12, 0.78), (0.25, 0.95),
                         (1.0, 1.0)]), smooth=40)


# ======================================================= fireside tools ==
def star_loft(stations, n=8):
    """Loft of n-point sections along +Y: stations [(y, r, angle, k)], the
    odd points pulled in to k * r (k < 1 gives a square-ish bar that twists
    when the angle turns; k = 1 is round)."""
    rings = []
    for y, r, ang, k in stations:
        rings.append([(r * (1 if i % 2 == 0 else k) * math.sin(ang + TAU * i / n), y,
                       r * (1 if i % 2 == 0 else k) * math.cos(ang + TAU * i / n)) for i in range(n)])
    return loft(rings)


def twisted_rod(L, r, twist_rate, ends, n_tw, k=0.72):
    """A wrought-iron bar: round turned ends (stations given as [(y, r)]
    for the bottom end, mirrored at the top) and a twisted square middle."""
    lo = [(y, rr) for y, rr in ends]
    y0 = lo[-1][0] + (lo[-1][0] - lo[-2][0])
    y1 = -y0
    st = []
    for y, rr in lo:
        st.append((y, rr, twist_rate * y0, 1.0))
    for i in range(n_tw + 1):
        y = y0 + (y1 - y0) * i / n_tw
        st.append((y, r, twist_rate * y, k))
    for y, rr in reversed(lo):
        st.append((-y, rr, twist_rate * y1, 1.0))
    return star_loft(st)

# the stand's upright: turned foot collar, twisted square bar, hub collar
# where the arms leave (0.27 up) and a neck for the finial
part('FireToolSets__rod', f_cyl(0.006, 0.006, 0.60), lambda: star_loft(
    [(-0.300, 0.0052, -0.26 * 40, 1.0), (-0.290, 0.0060, -0.26 * 40, 1.0), (-0.276, 0.0060, -0.26 * 40, 1.0),
     (-0.268, 0.0045, -0.26 * 40, 1.0)] +
    [(y, 0.0057, y * 40, 0.72) for y in [-0.26 + 0.475 * i / 36 for i in range(37)]] +
    [(0.226, 0.0045, 0.215 * 40, 1.0), (0.236, 0.0055, 0.215 * 40, 1.0), (0.248, 0.0045, 0.215 * 40, 1.0),
     (0.258, 0.0060, 0.215 * 40, 1.0), (0.282, 0.0060, 0.215 * 40, 1.0), (0.290, 0.0045, 0.215 * 40, 1.0),
     (0.300, 0.0040, 0.215 * 40, 1.0)]))

# cast plinth: rounded foot edge, cove, ogee up to the top
part('FireToolSets__base', f_cyl(0.075, 0.083, 0.012), lambda: lathe(
    [(0, -0.006), (0.0815, -0.006), (0.0830, -0.0045), (0.0826, -0.0028), (0.0785, -0.0012),
     (0.0775, 0.0015), (0.0758, 0.0045), (0.0730, 0.0060), (0, 0.0060)], seg=30))

# acorn finial on a neck
part('FireToolSets__finial', f_sphere(0.012), lambda: lathe(
    [(0, -0.012), (0.0048, -0.012), (0.0042, -0.0092), (0.0085, -0.0068), (0.0116, -0.0028),
     (0.0120, 0.0012), (0.0102, 0.0055), (0.0068, 0.0082), (0.0030, 0.0098), (0.0022, 0.0112),
     (0, 0.012)], seg=12), smooth=50)

# hanging arm: plain where it leaves the hub (+Y), knob at the free end (-Y)
part('FireToolSets__arm', f_cyl(0.0025, 0.0025, 0.030), lambda: lathe(
    [(0, -0.015), (0.0017, -0.0148), (0.0025, -0.0135), (0.0024, -0.0118), (0.0013, -0.0105),
     (0.0016, 0.0), (0.0019, 0.0135), (0.0021, 0.015), (0, 0.015)], seg=8), smooth=50)

F_FT_HOOK = torus_arc_bounds(0.004, 0.001, math.pi)
part('FireToolSets__hook', F_FT_HOOK, lambda: fitf(ring_section(
    0.004, [(0.0, -0.001), (0.001, 0.0), (0.0, 0.001), (-0.001, 0.0)], seg=8, arc=math.pi,
    rmod=lambda a: (1.0 - 0.3 * a / math.pi, 0.0, 0.0)), F_FT_HOOK), smooth=60)

# tool shaft: turned ends with collars, twisted square middle
part('FireToolSets__shaft', f_cyl(0.003, 0.003, 0.32), lambda: star_loft(
    [(-0.160, 0.0024, -0.12 * 70, 1.0), (-0.152, 0.0030, -0.12 * 70, 1.0), (-0.142, 0.0030, -0.12 * 70, 1.0),
     (-0.133, 0.0021, -0.12 * 70, 1.0)] +
    [(y, 0.0029, y * 70, 0.72) for y in [-0.124 + 0.248 * i / 22 for i in range(23)]] +
    [(0.133, 0.0021, 0.124 * 70, 1.0), (0.142, 0.0030, 0.124 * 70, 1.0), (0.152, 0.0030, 0.124 * 70, 1.0),
     (0.160, 0.0024, 0.124 * 70, 1.0)]))

# forged poker head: neck (+Y) swelling into a quarter-twisted square spike
part('FireToolSets__pokerTip', f_cyl(0.001, 0.005, 0.020), lambda: fitf(star_loft(
    [(0.010, 0.0010, 0.0, 1.0), (0.0065, 0.0016, 0.0, 1.0), (0.003, 0.0024, 0.2, 0.75),
     (-0.003, 0.0040, 0.5, 0.72), (-0.0085, 0.0050, 0.78, 0.72), (-0.010, 0.0030, 0.8, 0.8)]),
    f_cyl(0.001, 0.005, 0.020)))

# turned handle: collar at the shaft (-Y), swelling grip, ball end
part('FireToolSets__grip', f_cyl(0.006, 0.006, 0.030), lambda: lathe(
    [(0, -0.015), (0.0034, -0.015), (0.0046, -0.0132), (0.0034, -0.0112), (0.0047, -0.0060),
     (0.0058, 0.0030), (0.0060, 0.0090), (0.0050, 0.0125), (0.0030, 0.0145), (0, 0.015)], seg=10),
     smooth=50)


def tong_arm():
    # flat forged arm widening into a gripping pad at the bottom (-Y)
    def sec(y, a, b):
        return [(a, y, b * 0.55), (a * 0.55, y, b), (-a * 0.55, y, b), (-a, y, b * 0.55),
                (-a, y, -b * 0.55), (-a * 0.55, y, -b), (a * 0.55, y, -b), (a, y, -b * 0.55)]
    return loft([sec(0.0275, 0.0011, 0.0015), sec(0.020, 0.0012, 0.0016), sec(-0.012, 0.0013, 0.0017),
                 sec(-0.018, 0.0016, 0.0022), sec(-0.0235, 0.0024, 0.0025), sec(-0.0268, 0.0020, 0.0024),
                 sec(-0.0275, 0.0011, 0.0016)])

part('FireToolSets__pincer', f_cyl(0.0025, 0.0025, 0.055), tong_arm)

# tongs' bow handle: twisted square iron ring
part('FireToolSets__loop', f_torus(0.010, 0.002), lambda: ring_section(
    0.010, [(-0.0017, -0.0017), (0.0017, -0.0017), (0.0017, 0.0017), (-0.0017, 0.0017)], seg=24, twist=2.0),
     smooth=40)

# shovel pan: plate with the side lips turned up
part('FireToolSets__scoop', f_box(0.035, 0.005, 0.030), lambda: prism(
    [(-0.016, -0.0025), (0.016, -0.0025), (0.0175, 0.0025), (0.0165, 0.0025), (0.0151, -0.0013),
     (-0.0151, -0.0013), (-0.0165, 0.0025), (-0.0175, 0.0025)], 0.030, bevel_r=0.0003))


def scoop_back():
    # arched back plate of the shovel
    arch = [(0.0175 * math.cos(math.pi * i / 8), 0.002 + 0.008 * math.sin(math.pi * i / 8)) for i in range(9)]
    return prism([(-0.0175, -0.010), (0.0175, -0.010)] + arch, 0.003, bevel_r=0.0004)

part('FireToolSets__scoopBack', f_box(0.035, 0.020, 0.003), scoop_back)


def brush_head():
    # hearth brush: a dense, slightly splayed bundle of bristle tufts
    s = lathe([(0, -0.014), (0.0100, -0.0138), (0.0120, -0.0118), (0.0118, -0.0060), (0.0126, 0.0),
               (0.0133, 0.0060), (0.0140, 0.0115), (0.0124, 0.0138), (0, 0.014)], seg=16)
    s.warp(flutes(8, 0.07, -0.012, 0.013))
    return noisy(s, 0.0006, scale=500, seed=11)

part('FireToolSets__bristles', f_cyl(0.014, 0.012, 0.028), brush_head, smooth=70)

# cinder: a faceted clinker lump
part('FireToolSets__lump', f_sphere(0.004), lambda: fitf(noisy(
    sphere(0.004, seg=7, rings=5), 0.0009, scale=300, seed=3), f_sphere(0.004)), flat=True)


# ========================================================= specimen dome ==
# ebonised turned plinth: rounded foot bead, cove, ogee to the top
part('SpecimenDomes__base', f_cyl(0.075, 0.081, 0.015), lambda: lathe(
    [(0, -0.0075), (0.0790, -0.0075), (0.0810, -0.0058), (0.0808, -0.0035), (0.0782, -0.0018),
     (0.0765, 0.0010), (0.0768, 0.0035), (0.0752, 0.0065), (0.0738, 0.0075), (0, 0.0075)], seg=28))


def bead_ring(R, r, beads, per=4):
    sec = [(0.0, -r), (r, 0.0), (0.0, r), (-r, 0.0)]
    def mod(a):
        u = (a * beads / TAU) % 1.0
        return (0.62 + 0.38 * math.sin(math.pi * u) ** 0.7, 0.0, 0.0)
    return ring_section(R, sec, seg=beads * per, rmod=mod)

# gilt bead moulding round the plinth top
part('SpecimenDomes__baseRing', f_torus(0.075, 0.004), lambda: bead_ring(0.075, 0.004, 17), smooth=60)

# gilt collar the glass sits in: upright band with a rounded top bead
part('SpecimenDomes__domeRim', f_torus(0.067, 0.002), lambda: ring_section(
    0.067, [(-0.0012, -0.002), (0.0010, -0.002), (0.0020, -0.0010), (0.0020, 0.0010),
            (0.0010, 0.0020), (-0.0012, 0.0020)], seg=40))


def plaque():
    # canted-corner brass plate with a bevelled face and two pins
    c = 0.0022
    o = [(-0.018 + c, -0.005), (0.018 - c, -0.005), (0.018, -0.005 + c), (0.018, 0.005 - c),
         (0.018 - c, 0.005), (-0.018 + c, 0.005), (-0.018, 0.005 - c), (-0.018, -0.005 + c)]
    p = prism(o, 0.0016, bevel_r=0.0005).move(z=-0.0002)
    pins = sum((lathe([(0, 0.0), (0.0008, 0.0), (0.0005, 0.0005), (0, 0.0006)], seg=6)
                .rot(x=math.pi / 2).move(x=sx * 0.0155, z=0.0006) for sx in (-1, 1)), Shape())
    return p + pins

part('SpecimenDomes__plaque', f_box(0.036, 0.010, 0.002), plaque)


def twig():
    # natural twig perch: kinked, knobbly, with a snapped side stub
    pts = [(0.0003, -0.020, 0.0), (-0.0004, -0.012, 0.0002), (0.0002, -0.004, -0.0003),
           (-0.0002, 0.006, 0.0002), (0.0004, 0.014, -0.0002), (0.0, 0.020, 0.0)]
    t = tube(pts, 0.0024, seg=7, smooth=1)
    stub = lathe([(0, 0.0), (0.0013, 0.0), (0.0010, 0.0022), (0, 0.0026)], seg=5).rot(z=-1.1).move(x=0.0012, y=0.004)
    return noisy(t, 0.00022, scale=900, seed=2) + stub

part('SpecimenDomes__perch', f_cyl(0.003, 0.003, 0.040), twig, smooth=70)


def bird_body():
    # plump songbird torso: full breast forward (+X) and low, the back
    # rising to the shoulders, a tapered rump at -X. Built at its true
    # shape (the game stretches the sphere 1.0 x 0.85 x 1.3)
    s = lathe([(0, -1.0), (0.45, -0.88), (0.78, -0.55), (0.96, -0.1), (0.98, 0.25), (0.86, 0.6),
               (0.55, 0.88), (0, 1.0)], seg=12)
    s = ax_x(s).scale(-1, 1, 1)                      # blunt breast end to +X
    def w(p):
        x = p.x
        p.y -= 0.18 * max(0.0, x) ** 2 * (1 if p.y < 0.2 else 0.3)   # breast hangs low
        p.y += 0.12 * max(0.0, -x) ** 2                              # rump lifts
        return p
    s.warp(w)
    return fitf(noisy(s, 0.03, scale=6, seed=7), f_sphere(0.014))

part('SpecimenDomes__birdBody', f_sphere(0.014), bird_body, smooth=180)


def bird_head():
    # round crown, the face drawn forward to the beak root (+X)
    s = sphere(1.0, seg=12, rings=7)
    def w(p):
        if p.x > 0.35:
            t = (p.x - 0.35) / 0.65
            p.z *= 1 - 0.45 * t * t
            p.y = p.y * (1 - 0.35 * t * t) - 0.05 * t
        if p.y > 0.3:
            p.y -= 0.08 * (p.y - 0.3)                # a flatter crown
        return p
    return fitf(s.warp(w), f_sphere(0.009))

part('SpecimenDomes__birdHead', f_sphere(0.009), bird_head, smooth=180)


def beak():
    # conical seed-eater's beak, a little hooked; tip bends to local +X
    # (world down once the cone is turned to point along +X)
    s = lathe([(0.002, -0.003), (0.0016, -0.001), (0.0009, 0.0012), (0.0003, 0.0027), (0, 0.003)], seg=8)
    def w(p):
        t = (p.y + 0.003) / 0.006
        p.x += 0.0006 * t * t
        p.z *= 0.8
        return p
    return fitf(s.warp(w), f_cone(0.002, 0.006))

part('SpecimenDomes__beak', f_cone(0.002, 0.006), beak, smooth=50)


def tail():
    # bundle of long tail feathers tapering to a pointed tip (+Y), three
    # raised quills on the broad side
    def sec(y, r, kz):
        pts = []
        for i in range(10):
            a = TAU * i / 10
            k = 1.0 + (0.12 if i in (0, 5) else 0.0)
            pts.append((r * math.sin(a) * k, y, r * math.cos(a) * kz))
        return pts
    s = loft([sec(-0.008, 0.0068, 0.7), sec(-0.004, 0.0070, 0.6), sec(0.0015, 0.0055, 0.5),
              sec(0.0060, 0.0028, 0.45), sec(0.0080, 0.0006, 0.4)])
    return fitf(s, f_cone(0.007, 0.016))

part('SpecimenDomes__tail', f_cone(0.007, 0.016), tail, smooth=60)


def wing():
    # folded wing: a feathered leaf along Z with rows of covert tips
    # stepping out from the middle towards both ends (either side uses it)
    s = lathe([(0, -1.0), (0.35, -0.9), (0.62, -0.7), (0.84, -0.42), (0.96, -0.14), (1.0, 0.0),
               (0.96, 0.14), (0.84, 0.42), (0.62, 0.7), (0.35, 0.9), (0, 1.0)], seg=12)
    s = s.rot(x=math.pi / 2)
    def w(p):
        u = abs(p.z) * 3.0
        k = 1.0 - 0.07 * (u - math.floor(u))
        p.x *= k
        p.y *= k
        return p
    return fitf(s.warp(w), f_sphere(0.010))

part('SpecimenDomes__wing', f_sphere(0.010), wing, smooth=180)

# glass eye / pin-head bead
part('SpecimenDomes__bead', f_sphere(0.0015), lambda: sphere(0.0015, seg=8, rings=5), smooth=180)

# entomological pin: fine needle with a sharp point at the bottom
part('SpecimenDomes__pin', f_cyl(0.0005, 0.0005, 0.030), lambda: lathe(
    [(0, -0.015), (0.0003, -0.0138), (0.0005, -0.012), (0.0005, 0.0148), (0, 0.015)], seg=6), smooth=60)

# butterfly body: furry thorax at the head end (-Y), ringed abdomen to +Y
part('SpecimenDomes__bfBody', f_cyl(0.001, 0.001, 0.016), lambda: lathe(
    [(0, -0.008), (0.0008, -0.0074), (0.0010, -0.0055), (0.0007, -0.0036), (0.0009, -0.0026),
     (0.0008, 0.0006), (0.0007, 0.0036), (0.0004, 0.0068), (0, 0.008)], seg=6), smooth=60)


def bf_head():
    # small round head with two bulging compound eyes (sides, front +X)
    s = sphere(0.002, seg=8, rings=5)
    def w(p):
        if p.x > -0.0005:
            p.z *= 1.0 + 0.25 * math.exp(-((p.y) / 0.0011) ** 2)
        return p
    return fitf(s.warp(w), f_sphere(0.002))

part('SpecimenDomes__bfHead', f_sphere(0.002), bf_head, smooth=120)

# clubbed antenna, curving slightly forward
part('SpecimenDomes__antenna', f_cyl(0.0004, 0.0004, 0.010), lambda: lathe(
    [(0, -0.005), (0.00022, -0.005), (0.00022, 0.0026), (0.0004, 0.0036), (0.00036, 0.0046),
     (0, 0.005)], seg=4).warp(lambda p: Vector((p.x + 0.00025 * ((p.y + 0.005) / 0.01) ** 2 - 0.0001, p.y, p.z))),
     smooth=60)


def beetle():
    # beetle at its true size (x 12, y 4.8, z 19 mm; head end +Z): domed
    # elytra with a sunken seam, a waist, then the narrower pronotum
    s = lathe([(0, -0.0096), (0.0042, -0.0084), (0.0060, -0.0045), (0.0058, 0.0012),
               (0.0045, 0.0034), (0.0052, 0.0060), (0.0040, 0.0090), (0, 0.0096)], seg=8)
    s = s.rot(x=math.pi / 2)                         # axis to Z (+Y end -> +Z)
    def w(p):
        p.y *= 0.40 if p.y > 0 else 0.22
        if p.y > 0 and abs(p.x) < 1e-4 and p.z < 0.003:
            p.y *= 0.8                               # the elytral seam
        return p
    s.warp(w)
    return fitf(s, f_sphere(0.006))

part('SpecimenDomes__beetle', f_sphere(0.006), beetle, smooth=70)

# beetle head: rounded, flattened, narrowing to the mouthparts (+Z)
part('SpecimenDomes__beetleHead', f_sphere(0.003), lambda: fitf(lathe(
    [(0, -0.003), (0.0024, -0.0022), (0.0030, 0.0), (0.0024, 0.0018), (0.0012, 0.0028), (0, 0.003)],
    seg=8).rot(x=math.pi / 2).scale(1, 0.7, 1), f_sphere(0.003)), smooth=70)

# specimen label: a little curled card
part('SpecimenDomes__label', f_box(0.010, 0.0008, 0.004), lambda: slab(
    0.010, 0.004, 0.0006, 3, 2).rot(x=-math.pi / 2).warp(
    lambda p: Vector((p.x, p.y + 0.0003 * (p.x / 0.005) ** 2, p.z))))


# ========================================================= secretary desk ==
def sec_cabinet():
    # carcass with a moulded top, a chamfered plinth, a fielded panel on
    # the front above the drawers and on each side (drawer fronts are
    # separate meshes on the front face at z 0.175)
    top = rbox(0.55, 0.022, 0.35, r=0.006, seg=2).move(y=0.289)
    plinth = rbox(0.55, 0.034, 0.35, r=0.005, seg=1).move(y=-0.283)
    body = box(0.536, 0.548, 0.349).move(y=0.004, z=-0.0005)
    front = rbox(0.47, 0.21, 0.006, r=0.002, seg=1).move(y=0.17, z=0.1735)
    field = rbox(0.42, 0.165, 0.004, r=0.0015, seg=1).move(y=0.17, z=0.1755)
    sides = sum((rbox(0.006, 0.44, 0.27, r=0.002, seg=1).move(x=sx * 0.2685, y=0.004) for sx in (-1, 1)), Shape())
    return top + plinth + body + front + field + sides

part('SecretaryDesks__cabinet', f_box(0.55, 0.60, 0.35), sec_cabinet)

# turned leg: collar under the carcass (+Y), baluster swell, bun foot (-Y)
part('SecretaryDesks__leg', f_cyl(0.014, 0.020, 0.20), lambda: lathe(
    [(0, -0.100), (0.0160, -0.100), (0.0200, -0.092), (0.0200, -0.083), (0.0160, -0.073),
     (0.0110, -0.067), (0.0130, -0.061), (0.0105, -0.053), (0.0120, -0.020), (0.0145, 0.020),
     (0.0128, 0.052), (0.0105, 0.070), (0.0132, 0.079), (0.0140, 0.086), (0.0140, 0.100),
     (0, 0.100)], seg=14), smooth=45)


def drawer_front(w, h, t, bead):
    # flat drawer front framed by a cock bead round its edge
    plate = rbox(w - 0.002, h - 0.002, t * 0.7, r=0.0012, seg=1).move(z=-t * 0.15)
    b = bead
    strips = [rbox(w, b, t * 0.55, r=0.001, seg=1).move(y=sy * (h / 2 - b / 2), z=t * 0.225) for sy in (-1, 1)]
    strips += [rbox(b, h - 2 * b, t * 0.55, r=0.001, seg=1).move(x=sx * (w / 2 - b / 2), z=t * 0.225) for sx in (-1, 1)]
    return plate + sum(strips, Shape())

part('SecretaryDesks__drawer', f_box(0.5225, 0.168, 0.005), lambda: drawer_front(0.5225, 0.168, 0.005, 0.006))

# brass knob, rose against the drawer (-Y), mushroom face (+Y)
part('SecretaryDesks__pull', f_cyl(0.008, 0.008, 0.006), lambda: lathe(
    [(0, -0.003), (0.0045, -0.003), (0.0028, -0.0012), (0.0078, 0.0008), (0.0055, 0.0030), (0, 0.003)],
    seg=10), smooth=50)

# oval escutcheon behind the knob
part('SecretaryDesks__backplate', f_box(0.020, 0.010, 0.001), lambda: prism(
    [(0.010 * math.cos(TAU * i / 12), 0.005 * math.sin(TAU * i / 12)) for i in range(12)], 0.001))


def drop_panel():
    # fall front: rounded edges, the writing field sunk 1 mm into the top face
    core = rbox(0.4675, 0.010, 0.30, r=0.003, seg=2)
    return cut(core, box(0.40, 0.004, 0.24).move(y=0.005 + 0.001))

part('SecretaryDesks__dropPanel', f_box(0.4675, 0.010, 0.30), drop_panel)


def hinge():
    # brass butt-hinge leaf with its knuckle along the hinge edge (-Y)
    leaf = box(0.012, 0.0098, 0.0011).move(y=0.0011, z=-0.0003)
    knuckle = lathe([(0, -0.006), (0.0009, -0.0058), (0.0010, 0.0058), (0, 0.006)], seg=6).rot(z=math.pi / 2).move(y=-0.005)
    screws = sum((lathe([(0, 0.0), (0.0012, 0.0), (0, 0.0005)], seg=5).rot(x=math.pi / 2).move(x=sx * 0.0035, y=0.002, z=0.00025)
                  for sx in (-1, 1)), Shape())
    return leaf + knuckle + screws

part('SecretaryDesks__hinge', f_box(0.012, 0.012, 0.002), hinge)

# folding brass stay: eyes at the ends, knuckle joint in the middle
part('SecretaryDesks__stay', f_cyl(0.001, 0.001, 0.14), lambda: lathe(
    [(0, -0.070), (0.0009, -0.069), (0.0010, -0.066), (0.0006, -0.064), (0.0006, -0.002),
     (0.0010, -0.001), (0.0010, 0.001), (0.0006, 0.002), (0.0006, 0.064), (0.0010, 0.066),
     (0.0009, 0.069), (0, 0.070)], seg=6), smooth=50)


def upper_case():
    # the upper case: six pigeonholes cut into its front (their dark backs
    # are separate planes 1.75 mm behind the front), eased edges
    w, h, d = 0.4675, 0.30, 0.2625
    s = rbox(w, h, d, r=0.003, seg=1)
    pitch = w / 6
    hw, hh = pitch * 0.85, h * 0.55
    holes = [box(hw, hh, 0.012).move(x=-w / 2 + (i + 0.5) * pitch, y=0.015, z=d / 2 - 0.004 + 0.006)
             for i in range(6)]
    return cut(s, *holes)

part('SecretaryDesks__upper', f_box(0.4675, 0.30, 0.2625), upper_case)


def paper_roll():
    # a rolled document: the sheet's spiral shows at both ends
    outer, inner = [], []
    n = 18
    for i in range(n + 1):
        th = 3.2 * math.pi * i / n
        r = 0.0040 - 0.0022 * i / n
        outer.append((r * math.cos(th), r * math.sin(th)))
        inner.append(((r - 0.00045) * math.cos(th), (r - 0.00045) * math.sin(th)))
    s = prism(outer + inner[::-1], 0.04636).rot(x=-math.pi / 2)
    return fitf(s, f_cyl(0.004, 0.004, 0.04636))

part('SecretaryDesks__roll', f_cyl(0.004, 0.004, 0.04636), paper_roll, smooth=40)

part('SecretaryDesks__sdrawer', f_box(0.09934, 0.05, 0.005), lambda: drawer_front(0.09934, 0.05, 0.005, 0.004))

# tiny brass knob
part('SecretaryDesks__knob', f_sphere(0.003), lambda: lathe(
    [(0, -0.003), (0.0012, -0.003), (0.0010, -0.0015), (0.0030, 0.0003), (0.0022, 0.0024), (0, 0.003)],
    seg=8), smooth=60)


def desk_paper():
    s = slab(0.10, 0.14, 0.0006, 5, 7).rot(x=-math.pi / 2)
    def w(p):
        p.y += 0.0012 * (p.z / 0.07) ** 2 * (1 if p.z > 0 else 0.4) + 0.0003 * (p.x / 0.05) ** 2
        return p
    s.warp(w)
    lo, hi = s.bounds()
    return s.move(y=-0.0005 - lo[1])

part('SecretaryDesks__paper', f_box(0.10, 0.001, 0.14), desk_paper)


def cornice():
    # cap moulding: cove and ogee stepping out at the front and both
    # sides, flush at the back (-Z)
    w, d = 0.57, 0.3175
    rings = []
    for y, ins in [(-0.010, 0.013), (-0.0075, 0.012), (-0.0045, 0.009), (-0.0025, 0.0052), (-0.0015, 0.0042),
                   (0.0, 0.0040), (0.0015, 0.0018), (0.0045, 0.0), (0.0085, 0.0), (0.010, 0.0015)]:
        x0, z1, z0 = w / 2 - ins, d / 2 - ins, -d / 2
        c = 0.002
        rings.append([(x0, y, z0), (x0, y, z1 - c), (x0 - c, y, z1), (-x0 + c, y, z1), (-x0, y, z1 - c), (-x0, y, z0)])
    return loft(rings)

part('SecretaryDesks__cornice', f_box(0.57, 0.020, 0.3175), cornice)


# ======================================================= tarnished mirror ==
# gilt frame: rounded outer edge, a bead, cove, a second bead, a sunk
# channel and a crowned face (symmetric so it serves both bar directions)
part('TarnishedMirrors__bar', f_box(0.472, 0.036, 0.020), lambda: crown_bar(
    0.472, 0.036, 0.020, [(0.0, 0.28), (0.018, 0.56), (0.04, 0.70), (0.065, 0.58), (0.09, 0.64),
                          (0.12, 0.78), (0.16, 0.86), (0.2, 0.76), (0.26, 0.92), (0.45, 1.0),
                          (1.0, 1.0)]), smooth=40)


def rosette():
    # carved rosette boss facing +Z: eight rounded petals round a button
    s = sphere(1.0, seg=16, rings=9).rot(x=math.pi / 2)
    def w(p):
        phi = math.atan2(p.y, p.x)
        if p.z > -0.2:
            t = min(1.0, (p.z + 0.2) / 0.6)
            k = 1 - 0.16 * t * (0.5 - 0.5 * math.cos(8 * phi))
            p.x *= k
            p.y *= k
        if p.z > 0.75:
            p.z += 0.08                              # raised centre button
        return p
    return fitf(s.warp(w), f_sphere(0.020))

part('TarnishedMirrors__rosette', f_sphere(0.020), rosette, smooth=60)


def pediment():
    # scrolled crest: an arch rising to the centre, C-scroll ends, with a
    # raised carved panel applied on its face (+Z)
    right = [(0.120, -0.0125), (0.120, -0.0035), (0.113, 0.0005), (0.104, 0.0), (0.099, -0.0035),
             (0.090, -0.0015), (0.075, 0.0025), (0.055, 0.0052), (0.035, 0.0080), (0.018, 0.0108),
             (0.0, 0.0125)]
    out = right + [(-x, y) for x, y in reversed(right[:-1])]
    back = prism(out, 0.010, bevel_r=0.0012).move(z=-0.002)
    inner = [(x * 0.82, (y + 0.0125) * 0.62 - 0.0125 + 0.003) for x, y in out]
    face = prism(inner, 0.004, bevel_r=0.0010).move(z=0.005)
    return back + face

part('TarnishedMirrors__pediment', f_box(0.24, 0.025, 0.014), pediment)

# acorn drop under the frame: cupped top, pointed tip
part('TarnishedMirrors__pendant', f_sphere(0.012), lambda: lathe(
    [(0, -0.012), (0.0022, -0.0100), (0.0062, -0.0058), (0.0092, -0.0010), (0.0110, 0.0030),
     (0.0120, 0.0058), (0.0090, 0.0080), (0.0060, 0.0095), (0.0045, 0.0112), (0, 0.012)], seg=12),
     smooth=50)


# ======================================================= servant bell pull ==
def bell_bracket():
    # wall plate (-Z, thin at the screw holes) and a cast arm projecting
    # to +Z with a scrolled nose
    plate = rbox(0.08, 0.020, 0.0035, r=0.0010, seg=1).move(z=-0.015 + 0.00175)
    side = [(-0.0115, -0.010), (-0.0115, 0.010), (0.004, 0.010), (0.011, 0.0085), (0.015, 0.0045),
            (0.015, 0.0005), (0.012, -0.0018), (0.007, -0.0015), (0.002, -0.0035), (-0.005, -0.0075)]
    arm = prism([(-z, y) for z, y in side], 0.016, bevel_r=0.0012).rot(y=math.pi / 2)
    return plate + arm

part('ServantBellPulls__bracket', f_box(0.08, 0.020, 0.030), bell_bracket)

# the bell hangs mouth down: crown at the top (r 3 cm, closed by the cap),
# waist, flaring lip at the bottom (r 4.5 cm); a 3 mm shell, open below
part('ServantBellPulls__bell', f_cyl(0.045, 0.030, 0.075), lambda: lathe(
    [(0, 0.0375), (0.0280, 0.0375), (0.0300, 0.0345), (0.0310, 0.0200), (0.0330, 0.0000),
     (0.0370, -0.0180), (0.0420, -0.0300), (0.0450, -0.0362), (0.0442, -0.0375), (0.0410, -0.0368),
     (0.0380, -0.0300), (0.0336, -0.0180), (0.0296, 0.0000), (0.0272, 0.0200), (0, 0.0330)], seg=20),
     smooth=50)


def bell_cap():
    # domed crown cap with a hex nut on top
    dome = lathe([(0, -0.004), (0.0300, -0.004), (0.0300, -0.0016), (0.0265, 0.0008), (0.0170, 0.0022),
                  (0.0080, 0.0027), (0, 0.0027)], seg=20)
    nut = lathe([(0, 0.0024), (0.0060, 0.0024), (0.0060, 0.0040), (0, 0.0040)], seg=6)
    return dome + nut

part('ServantBellPulls__bellTop', f_cyl(0.030, 0.030, 0.008), bell_cap)

# clapper: a ball on a stub of its stem
part('ServantBellPulls__clapper', f_sphere(0.008), lambda: lathe(
    [(0, -0.008), (0.0045, -0.0068), (0.0074, -0.0028), (0.0066, 0.0018), (0.0030, 0.0050),
     (0.0014, 0.0058), (0.0014, 0.008), (0, 0.008)], seg=7), smooth=60)


def slotted_screw():
    # domed brass head standing proud of the plate, slot across it
    head = lathe([(0, -0.0015), (0.0030, -0.0015), (0.0030, -0.0005), (0.0021, 0.0008), (0, 0.0015)], seg=8)
    return cut(head, box(0.007, 0.0012, 0.0006).move(y=0.0013))

part('ServantBellPulls__screw', f_cyl(0.003, 0.003, 0.003), slotted_screw)


def split_ring():
    # a small coiled split ring: 1.25 turns of wire rising through the ring
    arc = 1.25 * TAU
    return ring_section(0.008, circ(0.0005, 3), seg=16, arc=arc,
                        rmod=lambda a: (1.0, -0.0005 + 0.001 * a / arc, 0.0))

part('ServantBellPulls__spring', f_torus(0.008, 0.001), split_ring, smooth=70)


def rope(L, r, n_st, step):
    """Three-strand laid cord along +Y: a 9-point section with three
    lobes that turn a little at every station."""
    rings = []
    for k in range(n_st):
        y = -L / 2 + L * k / (n_st - 1)
        th = step * k
        ring = []
        for i in range(9):
            a = TAU * i / 9 + th
            rr = r * (0.80 + 0.20 * math.cos(3 * (a - th) + 0.0))
            ring.append((rr * math.sin(a), y, rr * math.cos(a)))
        rings.append(ring)
    return loft(rings)

part('ServantBellPulls__cord', f_cyl(0.005, 0.005, 1.08), lambda: rope(1.08, 0.005, 60, math.radians(40)),
     smooth=50)

# whipping band round the cord: twisted square wire
part('ServantBellPulls__braid', f_torus(0.006, 0.001), lambda: ring_section(
    0.006, [(-0.0008, -0.0008), (0.0008, -0.0008), (0.0008, 0.0008), (-0.0008, 0.0008)], seg=12, twist=1.0),
     smooth=60)


def tassel_head():
    # thread-wrapped tassel mould at its true size (5 x 6.5 cm; the game
    # stretches the sphere 1.3 tall): neck and collar where the cord goes
    # in, shoulder, full bulb, skirt flare where the fringe hangs
    s = lathe([(0, -0.0325), (0.0120, -0.0312), (0.0205, -0.0262), (0.0245, -0.0160), (0.0250, -0.0060),
               (0.0232, 0.0020), (0.0178, 0.0098), (0.0095, 0.0140), (0.0060, 0.0180), (0.0072, 0.0232),
               (0.0050, 0.0300), (0, 0.0325)], seg=12)
    s.warp(flutes(6, 0.05, -0.030, 0.012))
    return fitf(s, f_sphere(0.025))

part('ServantBellPulls__bulb', f_sphere(0.025), tassel_head, smooth=70)

# gilt gimp band round the tassel: twisted five-lobed section
part('ServantBellPulls__band', f_torus(0.021, 0.003), lambda: ring_section(
    0.021, circ(0.003, 5), seg=24, twist=2.0), smooth=50)


def fringe_strand():
    # a silk fringe cord with a gentle wave and a knotted, frayed end (-Y)
    s = lathe([(0, -0.0275), (0.0009, -0.0268), (0.0010, -0.0250), (0.0006, -0.0236), (0.0006, -0.012),
               (0.0006, 0.0), (0.0006, 0.012), (0.0006, 0.0262), (0, 0.0275)], seg=5)
    return s.warp(lambda p: Vector((p.x + 0.0003 * math.sin(p.y / 0.0275 * math.pi), p.y, p.z)))

part('ServantBellPulls__strand', f_cyl(0.001, 0.001, 0.055), fringe_strand, smooth=60)


# ================================================= dust-sheeted furniture ==
def rr_point(w, d, r, s):
    """Point and outward normal (x, z, nx, nz) at arc length s along a
    rounded rectangle w x d with corner radius r (starts mid +X side)."""
    a, b = w / 2 - r, d / 2 - r
    segs = [('l', (w / 2, 0), (0, 1), b), ('c', (a, b), 0.0), ('l', (a, d / 2), (-1, 0), 2 * a),
            ('c', (-a, b), math.pi / 2), ('l', (-w / 2, b), (0, -1), 2 * b), ('c', (-a, -b), math.pi),
            ('l', (-a, -d / 2), (1, 0), 2 * a), ('c', (a, -b), 1.5 * math.pi), ('l', (w / 2, -b), (0, 1), b)]
    arc = math.pi / 2 * r
    for sg in segs:
        L = sg[3] if sg[0] == 'l' else arc
        if s <= L + 1e-9:
            if sg[0] == 'l':
                (x0, z0), (dx, dz) = sg[1], sg[2]
                return x0 + dx * s, z0 + dz * s, dz, -dx
            (cx, cz), a0 = sg[1], sg[2]
            t = a0 + s / r
            return cx + r * math.cos(t), cz + r * math.sin(t), math.cos(t), math.sin(t)
        s -= L
    return w / 2, 0.0, 1.0, 0.0


def drape(w, d, rc, rows, n, fold_len, seed, cap_top=True, pw=1.5):
    """Cloth hanging round a rounded-rectangle footprint: rows
    [(y, inset, fold_amp)] from the floor up. The folds only ever push
    outwards (whatever is under the sheet stays covered); fold_len is the
    mean spacing of the vertical folds."""
    per = 2 * (w + d - 4 * rc) + 2 * math.pi * rc
    rings = []
    for y, ins, amp in rows:
        ring = []
        for i in range(n):
            s = per * i / n
            wi, di, ri = w - 2 * ins, d - 2 * ins, max(0.004, rc - ins * 0.5)
            per_i = 2 * (wi + di - 4 * ri) + 2 * math.pi * ri
            x, z, nx, nz = rr_point(wi, di, ri, s * per_i / per)
            ph = TAU * s / fold_len + 1.7 * noise.noise(Vector((s * 3.1, seed * 1.3, 0.5)))
            f = (0.5 + 0.5 * math.sin(ph)) ** pw
            f *= 0.75 + 0.5 * (0.5 + 0.5 * noise.noise(Vector((s * 7.3, y * 5.0, seed))))
            ring.append((x + nx * amp * f, y, z + nz * amp * f))
        rings.append(ring)
    return loft(rings, cap0=True, cap1=cap_top)


def top_round(y_top, R, amp, k=4):
    """Rows for a top edge rolling over with radius R."""
    out = []
    for i in range(k + 1):
        t = (math.pi / 2) * i / k
        out.append((y_top - R + R * math.sin(t), R * (1 - math.cos(t)), amp * (1 - i / k)))
    return out


def sheet_chair():
    # dust sheet over a chair: folds deepening to the floor, fuller where
    # the sheet goes over the seat and arms, the top rolling over the back
    h = 0.85
    rows = [(-h / 2, 0.004, 0.004), (-h / 2 + 0.10, 0.004, 0.005), (-h / 2 + 0.20, -0.003, 0.010),
            (-0.10, -0.004, 0.014), (0.02, -0.005, 0.012), (0.12, 0.0, 0.009), (0.26, 0.002, 0.006)]
    rows += top_round(h / 2, 0.045, 0.004, 3)
    return drape(0.48, 0.52, 0.06, rows, 44, 0.11, 3)

part('SheetCoveredFurniture__chairBody', f_box(0.48, 0.85, 0.52), sheet_chair)


def sheet_wingback():
    h = 1.15
    rows = [(-h / 2, 0.004, 0.004), (-h / 2 + 0.13, 0.004, 0.005), (-h / 2 + 0.25, 0.002, 0.012),
            (-0.22, 0.0, 0.015), (-0.08, -0.005, 0.013), (0.05, 0.0, 0.006), (0.20, 0.002, 0.004),
            (0.36, 0.002, 0.003), (0.46, 0.002, 0.003)]
    rows += top_round(h / 2, 0.05, 0.004, 4)
    return drape(0.55, 0.50, 0.07, rows, 56, 0.12, 8)

part('SheetCoveredFurniture__wingBody', f_box(0.55, 1.15, 0.50), sheet_wingback)


def sheet_skirt():
    # the hem where the sheet meets the floor: flares out in deep folds,
    # drawn in at the top where it disappears under the draped body
    rows = [(-0.075, 0.0, 0.014), (-0.068, 0.0, 0.013), (-0.03, 0.005, 0.009), (0.02, 0.012, 0.004),
            (0.06, 0.019, 0.0015), (0.075, 0.021, 0.0)]
    return drape(0.53, 0.57, 0.06, rows, 48, 0.09, 5, pw=1.0)

part('SheetCoveredFurniture__skirt', f_box(0.53, 0.15, 0.57), sheet_skirt)

# where the sheet goes over the arms: a soft padded roll
part('SheetCoveredFurniture__arm', f_box(0.05, 0.2975, 0.364),
     lambda: rbox(0.05, 0.2975, 0.364, r=0.022, seg=3), smooth=60)

def sheet_wing():
    # the sheet over a chair wing: a padded, pillowy block, fullest in the
    # middle, rolling off to every edge, a few soft creases
    rings = []
    for x, ins in [(-0.055, 0.032), (-0.051, 0.014), (-0.043, 0.005), (-0.026, 0.001), (0.0, 0.0),
                   (0.026, 0.001), (0.043, 0.005), (0.051, 0.014), (0.055, 0.032)]:
        pts = rrect_pts(0.46 - 2 * ins, 0.30 - 2 * ins, max(0.01, 0.06 - ins * 0.5), 3)
        rings.append([(x, y, z) for y, z in pts])
    s = loft(rings)
    return noisy(s, 0.004, scale=9, seed=4, axis=(1, 0, 0))

part('SheetCoveredFurniture__wing', f_box(0.11, 0.46, 0.30), sheet_wing, smooth=60)


def heightfield(w, t, d, nx, nz, fn):
    """A soft lump lying on a surface: top y = -t/2 + t * fn(u, v)
    (u, v in -1..1), flat bottom at -t/2, edges feathered down."""
    v, f = [], []
    for j in range(nz + 1):
        for i in range(nx + 1):
            u, vv = -1 + 2 * i / nx, -1 + 2 * j / nz
            v.append((u * w / 2, -t / 2 + t * (0.18 + 0.82 * max(0.0, fn(u, vv))), vv * d / 2))
    idx = lambda i, j: j * (nx + 1) + i
    for j in range(nz):
        for i in range(nx):
            f.append((idx(i, j), idx(i, j + 1), idx(i + 1, j + 1), idx(i + 1, j)))
    rim = [idx(i, 0) for i in range(nx + 1)] + [idx(nx, j) for j in range(1, nz + 1)] + \
          [idx(i, nz) for i in range(nx - 1, -1, -1)] + [idx(0, j) for j in range(nz - 1, 0, -1)]
    base = len(v)
    for k in rim:
        x, _, z = v[k]
        v.append((x, -t / 2, z))
    m = len(rim)
    for a in range(m):
        b = (a + 1) % m
        f.append((rim[a], rim[b], base + b, base + a))
    f.append(tuple(base + a for a in range(m)))
    return _fw(Shape(v, f))


def cloth_ripple(u, v):
    # two soft creases running along X, dying out towards the ends
    end = max(0.0, 1 - abs(u) ** 3)
    r1 = math.exp(-((v + 0.28) / 0.28) ** 2)
    r2 = 0.6 * math.exp(-((v - 0.42) / 0.22) ** 2) * (1 - 0.3 * u)
    return end * (r1 + r2) * (1 - 0.8 * abs(v) ** 6)

part('SheetCoveredFurniture__fold', f_box(0.144, 0.020, 0.104), lambda: heightfield(
    0.144, 0.020, 0.104, 10, 8, cloth_ripple), smooth=70)


def sheet_table_top():
    # the sheet over a table top: flat on top, rolling over the edges
    rows = [(-0.030, 0.002, 0.006), (-0.012, 0.0, 0.004), (0.010, 0.0, 0.002)] + top_round(0.030, 0.022, 0.0015, 3)
    return drape(1.10, 0.70, 0.03, rows, 64, 0.14, 13)

part('SheetCoveredFurniture__tableTop', f_box(1.10, 0.06, 0.70), sheet_table_top)

# a table leg showing through the sheet: soft square post, a little flare
part('SheetCoveredFurniture__legBump', f_box(0.05, 0.675, 0.05), lambda: loft(
    [[(x, y, z) for x, z in rrect_pts(wd, wd, wd * 0.35, 1)] for y, wd in
     [(-0.3375, 0.050), (-0.30, 0.044), (0.30, 0.040), (0.3375, 0.030)]]), smooth=60)

# the long crease where the sheet meets across the table top
part('SheetCoveredFurniture__centerFold', f_box(0.77, 0.015, 0.04), lambda: heightfield(
    0.77, 0.015, 0.04, 16, 4, lambda u, v: max(0.0, 1 - abs(u) ** 4) * math.exp(-(v / 0.5) ** 2)
    * (0.85 + 0.15 * math.sin(u * 7.0))), smooth=70)


def sheet_bump():
    # the sheet over something small left on the table: a soft mound that
    # spreads into radial folds where it meets the table top (flat bottom)
    s = lathe([(0, -0.080), (0.080, -0.080), (0.074, -0.072), (0.062, -0.050), (0.054, -0.015),
               (0.046, 0.020), (0.036, 0.052), (0.022, 0.071), (0.009, 0.0785), (0, 0.080)], seg=28)
    def w(p):
        a = math.atan2(p.x, p.z)
        k = max(0.0, (0.02 - p.y) / 0.10)
        m = 1 + 0.10 * k * math.sin(7 * a + 0.8 * math.sin(3 * a))
        p.x *= m
        p.z *= m
        return p
    return fitf(s.warp(w), f_sphere(0.08))

part('SheetCoveredFurniture__bump', f_sphere(0.08), sheet_bump, smooth=70)
