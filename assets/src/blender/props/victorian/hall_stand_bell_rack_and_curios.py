"""Batch b21: Victorian house objects.

Hall stand (turned post, hooks, hats, umbrellas, scarf), servants' bell
rack, cast-iron door stops (dog, boot, owl), framed correspondence and
heraldic crest frames, empty painting hooks, chess side stand and pieces,
miniature portrait frames, peacock feather vase, bellows camera on its
tripod, brass wall sconce, ornate gilt mirror, pocket watch, open journal,
parchment scroll.

Glass (snifter), DoubleSide paper/cloth (canvases, pages, feather eyes,
ribbons), flat painted marks and basic material flames stay primitives; see
docs/research/props-remodelling/hall_stand_bell_rack_and_curios.md.
"""
from propkit import *
import propkit


# ------------------------------------------------------------ local kit --
def _fw(s):
    return propkit._fix_winding(s)


def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


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


def ring_section(R, sec, seg=32, arc=TAU, a0=0.0, rmod=None):
    """Sweep a closed 2D section [(dr, dz), ...] round the Z axis in the XY
    plane (a torus_geo! with any cross-section). arc < TAU caps the ends.
    rmod(angle, i) -> (k_r, dz_add[, dR]) varies the section along the ring.
    A ring the game lays flat with rotation.x = PI/2 has local -Z up."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        mm = rmod(a, i) if rmod else (1.0, 0.0)
        kr, dzz = mm[0], mm[1]
        dR = mm[2] if len(mm) > 2 else 0.0
        for (dr, dz) in sec:
            rr_ = R + dR + dr * kr
            v.append((rr_ * ca, rr_ * sa, dz * kr + dzz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


def circ_sec(r, n=8, sx=1.0, sy=1.0, phase=0.0):
    return [(r * sx * math.cos(TAU * i / n + phase), r * sy * math.sin(TAU * i / n + phase)) for i in range(n)]


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


def yloft(profile, w, d, rc, cseg=3):
    """Stack of rounded rectangles along +Y: profile = [(y, inset), ...]."""
    rings = []
    for y, ins in profile:
        pts = rrect_pts(w - 2 * ins, d - 2 * ins, max(0.0003, rc - ins * 0.6), cseg)
        rings.append([(x, y, z) for x, z in pts])
    return loft(rings)


def torus_arc_bounds(R, r, arc, radial=24, tubular=128):
    """Bounds of torus_geo!(R, r, .., .., arc) as the game computes
    them (24 x 128 samples)."""
    xs, ys = [], []
    for i in range(tubular + 1):
        u = arc * i / tubular
        for j in range(radial + 1):
            w = TAU * j / radial
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r))


def knurl(s, seg, depth, y0, y1, odd=True):
    """Every other vertex of a `seg`-sided lathe between y0 and y1 is pulled
    in by `depth` (flutes, pleats, grip ribs)."""
    step = TAU / seg

    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6 or not (y0 - 1e-7 <= p.y <= y1 + 1e-7):
            return p
        i = int(round(math.atan2(p.x, p.z) / step)) % seg
        if (i % 2 == 1) == odd:
            k = max(0.0, r - depth) / r
            p.x *= k
            p.z *= k
        return p
    return s.warp(w)


def x_axis(s):
    """Y-axis shape laid along +X (its +Y end goes to +X)."""
    return s.rot(z=-math.pi / 2)


def frame_bar(w, h, t, back=0.5, edge=0.80, bead=0.10):
    """A picture-frame bar whose face crowns and rolls off towards all four
    edges, so it reads as a moulding whichever way the one constructor is
    stretched (the long and the tall bars share it). A sunk bead line runs
    round near the edge; the back is flat against the wall."""
    us = [-1, -0.985, -0.95, -0.9, -0.84, -0.76, -0.5, 0.0, 0.5, 0.76, 0.84, 0.9, 0.95, 0.985, 1]
    n = len(us)

    def f_(u):
        a = abs(u)
        base = max(0.0, 1 - a ** 6) ** 0.35
        dip = bead * math.exp(-((a - edge) / 0.045) ** 2)
        return max(0.0, base - dip)
    v, f = [], []
    for j, vv in enumerate(us):
        for i, uu in enumerate(us):
            z = -t / 2 + back * t + (1 - back) * t * f_(uu) * f_(vv)
            v.append((uu * w / 2, vv * h / 2, z))
    idx = lambda i, j: j * n + i
    for j in range(n - 1):
        for i in range(n - 1):
            f.append((idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1)))
    rim = [idx(i, 0) for i in range(n)] + [idx(n - 1, j) for j in range(1, n)] + \
          [idx(i, n - 1) for i in range(n - 2, -1, -1)] + [idx(0, j) for j in range(n - 2, 0, -1)]
    base_i = len(v)
    for k in rim:
        x, y, _ = v[k]
        v.append((x, y, -t / 2))
    m = len(rim)
    for a in range(m):
        b = (a + 1) % m
        f.append((rim[b], rim[a], base_i + a, base_i + b))
    f.append(tuple(base_i + a for a in reversed(range(m))))
    return _fw(Shape(v, f))


# ================================================= victorian hall stands ==
# umbrella drip tray: moulded foot, rounded edge, dished pan inside
part('VictorianHallStands__tray', f_cyl(0.16, 0.16, 0.020), lambda: lathe(
    [(0, -0.010), (0.150, -0.010), (0.1555, -0.0093), (0.1590, -0.0070), (0.1600, -0.0035),
     (0.1600, 0.0020), (0.1580, 0.0065), (0.1560, 0.0100), (0.1495, 0.0100), (0.1450, 0.0070),
     (0.1380, 0.0048), (0, 0.0048)], seg=36))

# raised tray rim: a bead with a flat underside (-Z is up once laid flat)
part('VictorianHallStands__rim', f_torus(0.16, 0.006), lambda: ring_section(
    0.16, [(-0.006, 0.006), (0.006, 0.006), (0.0056, -0.0012), (0.0036, -0.0048), (0.0, -0.006),
           (-0.0036, -0.0048), (-0.0056, -0.0012)], seg=64))


def hall_post():
    # turned column: plinth mouldings, a vase swell low down, slim necks
    # where the three loose rings sit, bead collars at both hook levels
    prof = [(0, -0.900), (0.0300, -0.900), (0.0300, -0.884), (0.0262, -0.878), (0.0282, -0.868),
            (0.0240, -0.856), (0.0222, -0.820), (0.0250, -0.760), (0.0262, -0.700), (0.0245, -0.640),
            (0.0205, -0.575), (0.0190, -0.530), (0.0212, -0.490), (0.0228, -0.400), (0.0214, -0.300),
            (0.0188, -0.230), (0.0210, -0.170), (0.0218, -0.020), (0.0206, 0.140), (0.0182, 0.220),
            (0.0200, 0.300), (0.0190, 0.600), (0.0178, 0.700), (0.0215, 0.714), (0.0215, 0.726),
            (0.0178, 0.740), (0.0176, 0.860), (0.0212, 0.872), (0.0212, 0.886), (0.0170, 0.894),
            (0.0120, 0.900), (0, 0.900)]
    return lathe(prof, seg=14)

part('VictorianHallStands__post', f_cyl(0.020, 0.030, 1.80), hall_post)

# turned ring collar round the post
part('VictorianHallStands__ring', f_torus(0.024, 0.005), lambda: ring_section(
    0.024, circ_sec(0.005, 6), seg=20))

# hook peg: flared where it meets the post (+Y), a collar, a rounded tip
part('VictorianHallStands__arm', f_cyl(0.004, 0.004, 0.055), lambda: lathe(
    [(0, -0.0275), (0.0022, -0.0272), (0.0030, -0.0255), (0.0027, -0.010), (0.0030, 0.008),
     (0.0040, 0.0165), (0.0033, 0.0205), (0.0034, 0.0250), (0.0040, 0.0275), (0, 0.0275)], seg=8),
     smooth=50)

# brass knob: a ball with a turned neck collar on top
part('VictorianHallStands__cap', f_sphere(0.008), lambda: lathe(
    [(0, -0.008), (0.0045, -0.0068), (0.0072, -0.0035), (0.0080, 0.0005), (0.0066, 0.0045),
     (0.0036, 0.0064), (0.0026, 0.0074), (0, 0.008)], seg=8), smooth=60)


def acorn_finial():
    # carved acorn: a neck, a scaly cup (knurled), the nut with a point
    s = lathe([(0, -0.024), (0.0110, -0.024), (0.0085, -0.0205), (0.0150, -0.0160), (0.0225, -0.0070),
               (0.0240, -0.0030), (0.0205, -0.0010), (0.0215, 0.0040), (0.0195, 0.0120), (0.0130, 0.0195),
               (0.0040, 0.0235), (0, 0.024)], seg=14)
    return knurl(s, 14, 0.0014, -0.0165, -0.0025)

part('VictorianHallStands__finial', f_sphere(0.024), acorn_finial, smooth=45)

# oval mirror surround: sight edge, flat, a bead and a cove to the back
part('VictorianHallStands__mirror', f_torus(0.09, 0.008), lambda: ring_section(
    0.09, [(-0.0080, 0.0030), (-0.0080, -0.0020), (-0.0055, -0.0045), (-0.0020, -0.0050),
           (0.0010, -0.0080), (0.0048, -0.0065), (0.0072, -0.0030), (0.0080, 0.0015),
           (0.0070, 0.0080), (-0.0060, 0.0080)], seg=28), smooth=40)


def furled_umbrella():
    # a slim furled umbrella: ferrule tip at -Y, the pleated canopy swelling
    # round the lower half, a tie strap, the bare stick up to the crook
    prof = [(0, -0.375), (0.0009, -0.3745), (0.0012, -0.368), (0.0015, -0.352), (0.0022, -0.346),
            (0.0030, -0.300), (0.0037, -0.200), (0.0040, -0.090), (0.0040, -0.070), (0.0036, -0.040),
            (0.0038, -0.030), (0.0033, 0.020), (0.0026, 0.100), (0.0020, 0.150), (0.0016, 0.165),
            (0.0015, 0.300), (0.0017, 0.372), (0, 0.375)]
    s = lathe(prof, seg=16)
    s = knurl(s, 16, 0.0009, -0.345, -0.092)
    s = knurl(s, 16, 0.0008, -0.028, 0.148)
    return s

part('VictorianHallStands__umbrella', f_cyl(0.004, 0.004, 0.75), furled_umbrella)


def crook_handle():
    # bent-cane crook: thick where it leaves the stick, easing to a rounded
    # tip; a collar at the stick end
    R, r = 0.018, 0.004
    s = ring_section(R, circ_sec(r, 8), seg=16, arc=math.pi,
                     rmod=lambda a, i: (1.0 - 0.18 * (a / math.pi) ** 1.5, 0.0))
    return fitf(s, torus_arc_bounds(R, r, math.pi))

part('VictorianHallStands__crook', torus_arc_bounds(0.018, 0.004, math.pi), crook_handle, smooth=50)


def top_hat_crown():
    # silk topper: raised hat band low down, waisted sides flaring to a
    # rounded top edge, slightly domed top
    return lathe([(0, -0.045), (0.0578, -0.045), (0.0584, -0.0440), (0.0584, -0.0230), (0.0574, -0.0222),
                  (0.0566, -0.0100), (0.0568, 0.0100), (0.0584, 0.0300), (0.0600, 0.0400), (0.0596, 0.0430),
                  (0.0578, 0.0446), (0.0300, 0.0450), (0, 0.0450)], seg=24)

part('VictorianHallStands__topHat', f_cyl(0.060, 0.058, 0.090), top_hat_crown)


def hat_brim(R, h, curl):
    # brim with a rolled bound edge, curled up at the sides and dipping at
    # front and back (the d'Orsay curve)
    y0 = -h / 2
    s = lathe([(0, y0 + 0.0006), (R * 0.72, y0 + 0.0004), (R * 0.88, y0 + 0.0002), (R * 0.965, y0 + h * 0.30),
               (R, y0 + h * 0.62), (R * 0.99, y0 + h * 0.92), (R * 0.965, y0 + h * 0.88), (R * 0.90, y0 + h * 0.46),
               (R * 0.80, y0 + h * 0.38), (0, y0 + h * 0.38)], seg=32)

    def w(p):
        rr = math.hypot(p.x, p.z) / R
        p.y += curl * math.cos(2 * math.atan2(p.z, p.x)) * rr ** 2
        return p
    return s.warp(w)

part('VictorianHallStands__topHatBrim', f_cyl(0.080, 0.080, 0.008), lambda: hat_brim(0.080, 0.008, 0.0016))
part('VictorianHallStands__bowlerBrim', f_cyl(0.065, 0.065, 0.006), lambda: hat_brim(0.065, 0.006, 0.0012))


def bowler_crown():
    # felt bowler: a round dome over a grosgrain band, open underneath
    prof = [(0.0, 0.0), (0.020, 0.0012), (0.036, 0.0068), (0.047, 0.0160), (0.0530, 0.0290),
            (0.0548, 0.0440), (0.0552, 0.0500), (0.0550, 0.0636)]
    s = vessel(prof, 0.0018, seg=20, floor=0.0014).rot(x=math.pi)
    return fitf(s, BOWLER)

BOWLER = f_bounds((-0.055, 0.055 * math.cos(math.pi * 0.55), -0.055), (0.055, 0.055, 0.055))
part('VictorianHallStands__bowler', BOWLER, bowler_crown)


def scarf_strip():
    # knitted scarf hanging from the hook: soft folds down its length,
    # a ribbed edge; thin along X
    s = slab(0.045, 0.35, 0.003, 4, 24)

    def w(p):
        p.z += 0.0011 * math.sin(p.y * 38 + 0.8) + 0.0005 * math.cos(p.x * 140 + p.y * 21)
        return p
    return fitf(s.warp(w).rot(y=math.pi / 2), f_box(0.005, 0.35, 0.045))

part('VictorianHallStands__scarf', f_box(0.005, 0.35, 0.045), scarf_strip, smooth=60)


def tassel():
    # a little bunch of loose yarn ends
    s = Shape()
    for k, (dz, dy) in enumerate(((-0.0025, 0.0), (0.0, -0.001), (0.0025, 0.0005))):
        s = s + tube([(0.0, 0.005, dz * 0.6), (0.0003 * (k - 1), 0.0, dz), (0.0006 * (k - 1), -0.005 + dy * 0.2, dz * 1.1)],
                     0.0012, seg=4)
    return fitf(s, f_box(0.005, 0.010, 0.008))

part('VictorianHallStands__fringe', f_box(0.005, 0.010, 0.008), tassel, smooth=70)


# ============================================== servants' hall bell racks ==
def moulding(profile, length):
    """Extrude a (z, y) section along X (a moulding strip). Front is +Z."""
    return prism(profile, length).rot(y=-math.pi / 2)


def bell_board():
    # raised-and-fielded board: a broad chamfered margin at the top and the
    # bottom, only a small round-over at the ends where the labels reach
    w, h = 0.52, 0.28
    lv = [(-0.010, 0.0, 0.0), (0.0035, 0.0, 0.0), (0.0060, 0.0008, 0.0050), (0.0080, 0.0020, 0.0105),
          (0.0092, 0.0032, 0.0150), (0.0100, 0.0045, 0.0172)]
    rings = []
    for z, ix, iy in lv:
        pts = rrect_pts(w - 2 * ix, h - 2 * iy, 0.004, 2)
        rings.append([(x, y, z) for x, y in pts])
    return loft(rings)

part('ServantHallBellRacks__board', f_box(0.52, 0.28, 0.020), bell_board)

# cornice on top: cove and rounded nose projecting to the board face
part('ServantHallBellRacks__topTrim', f_box(0.54, 0.020, 0.030), lambda: fitf(moulding(
    [(-0.015, -0.010), (0.006, -0.010), (0.0085, -0.0070), (0.0110, -0.0035), (0.0132, 0.0005),
     (0.0133, 0.0035), (0.0146, 0.0048), (0.0150, 0.0075), (0.0138, 0.0098), (0.0120, 0.0100),
     (-0.015, 0.0100)], 0.54), f_box(0.54, 0.020, 0.030)))

# base moulding: a rounded ovolo over a cove and a little foot
part('ServantHallBellRacks__botTrim', f_box(0.54, 0.015, 0.030), lambda: fitf(moulding(
    [(-0.015, 0.0075), (0.0120, 0.0075), (0.0142, 0.0060), (0.0150, 0.0030), (0.0146, 0.0005),
     (0.0120, -0.0020), (0.0112, -0.0045), (0.0122, -0.0058), (0.0122, -0.0075), (-0.015, -0.0075)],
    0.54), f_box(0.54, 0.015, 0.030)))


def name_plate(w, h, r, screws=True, cant=False):
    # thin plate with eased edges; brass ones get two domed screws
    outline = rounded_rect(w, h, r, seg=2) if not cant else \
        round_outline([(-w / 2 + r, -h / 2), (w / 2 - r, -h / 2), (w / 2, -h / 2 + r), (w / 2, h / 2 - r),
                       (w / 2 - r, h / 2), (-w / 2 + r, h / 2), (-w / 2, h / 2 - r), (-w / 2, -h / 2 + r)], 0.0006, 1)
    s = prism(outline, 0.0007, bevel_r=0.00025).move(z=-0.00015)
    if screws:
        for sx in (-1, 1):
            s = s + lathe([(0, 0.0), (0.0016, 0.0), (0.0013, 0.0003), (0.0006, 0.0005), (0, 0.00055)], seg=8) \
                .rot(x=math.pi / 2).move(x=sx * (w / 2 - 0.006), z=0.0002)
    return fitf(s, f_box(w, h, 0.001))

part('ServantHallBellRacks__title', f_box(0.208, 0.020, 0.001), lambda: name_plate(0.208, 0.020, 0.003))
part('ServantHallBellRacks__label', f_box(0.050, 0.020, 0.001), lambda: name_plate(0.050, 0.020, 0.003, False))
part('ServantHallBellRacks__roomPlate', f_box(0.055, 0.016, 0.001),
     lambda: name_plate(0.055, 0.016, 0.0028, False, cant=True))

SPRING = torus_arc_bounds(0.010, 0.0008, math.pi / 3)
# spring wire: a smooth rounded arc of brass wire
part('ServantHallBellRacks__spring', SPRING, lambda: fitf(ring_section(
    0.010, circ_sec(0.0008, 5), seg=8, arc=math.pi / 3), SPRING), smooth=60)


def servant_bell():
    # a real bell, mouth down: crown, waist, flaring sound bow and lip,
    # hollow inside with a clapper hanging in the mouth
    outer = [(0.0075, 0.0120), (0.0092, 0.0085), (0.0100, 0.0000), (0.0122, -0.0080), (0.0140, -0.0120)]
    inner = [(0.0128, -0.0120), (0.0105, -0.0060), (0.0086, 0.0040), (0.0060, 0.0090)]
    prof = [(0, 0.0102)] + list(reversed(inner)) + list(reversed(outer)) + [(0, 0.0120)]
    s = lathe(prof, seg=12)
    clapper = lathe([(0, -0.0112), (0.0024, -0.0096), (0.0015, -0.0074), (0.0006, 0.0100), (0, 0.0102)], seg=6)
    return s + clapper

part('ServantHallBellRacks__bell', f_cyl(0.014, 0.010, 0.024), servant_bell, smooth=50)

# crown boss on top of the bell
part('ServantHallBellRacks__cap', f_cyl(0.010, 0.010, 0.003), lambda: lathe(
    [(0, -0.0015), (0.0098, -0.0015), (0.0100, -0.0006), (0.0082, 0.0008), (0.0040, 0.0015), (0, 0.0015)],
    seg=12), smooth=50)


# ================================================ cast-iron door stops ==
def sell(a, b, n=10, p=2.5, cy=0.0):
    """Superellipse (z, y) ring points for lofts along X."""
    out = []
    for i in range(n):
        t = TAU * i / n
        c, s_ = math.cos(t), math.sin(t)
        out.append((a * math.copysign(abs(c) ** (2 / p), c), cy + b * math.copysign(abs(s_) ** (2 / p), s_)))
    return out


def xloft(stations, n=10, p=2.5):
    """Loft along X: stations = [(x, z_half, y_bottom, y_top), ...]."""
    rings = []
    for x, zh, yb, yt in stations:
        rings.append([(x, y, z) for z, y in sell(zh, (yt - yb) / 2, n, p, (yt + yb) / 2)])
    return loft(rings)


# plinth: a foot, an ovolo stepping in and a flat top for the figure
part('CastIronDoorStops__base', f_box(0.11, 0.010, 0.09), lambda: yloft(
    [(-0.005, 0.0), (-0.0032, 0.0), (-0.0016, 0.0007), (0.0000, 0.0020), (0.0016, 0.0030),
     (0.0032, 0.0040), (0.0036, 0.0048), (0.0050, 0.0052)], 0.11, 0.09, 0.009, 2))

DOG_BODY = f_sphere(0.030)


def dog_body():
    # a recumbent spaniel (the game stretches the sphere 1.5 x 1 x 0.9, so
    # it is modelled at that size): haunches at the back, deep chest, the
    # front paws laid out forward on the plinth
    st = [(-0.0450, 0.006, -0.018, -0.004), (-0.0415, 0.017, -0.027, 0.010), (-0.0330, 0.0255, -0.030, 0.017),
          (-0.0220, 0.0270, -0.030, 0.020), (-0.0100, 0.0235, -0.030, 0.021), (0.0020, 0.0215, -0.030, 0.022),
          (0.0140, 0.0220, -0.030, 0.026), (0.0260, 0.0205, -0.029, 0.030), (0.0360, 0.0165, -0.028, 0.024),
          (0.0420, 0.0110, -0.026, 0.012), (0.0450, 0.0040, -0.022, -0.002)]
    s = xloft(st, n=10, p=2.4)
    for sz in (-1, 1):
        s = s + tube([(0.012, -0.0245, sz * 0.0125), (0.030, -0.0250, sz * 0.0120), (0.0412, -0.0250, sz * 0.0115)],
                     0.0052, seg=6)
    s = s.scale(1.0, 1.0, 1.0)
    return fitf(s.scale(1 / 1.5, 1.0, 1 / 0.9), DOG_BODY)

part('CastIronDoorStops__dogBody', DOG_BODY, dog_body, smooth=60)

# head: round skull, a stop at the front where the muzzle sets in (+X)
part('CastIronDoorStops__dogHead', f_sphere(0.022), lambda: x_axis(lathe(
    [(0, -0.022), (0.012, -0.019), (0.019, -0.012), (0.022, -0.003), (0.0212, 0.006), (0.0180, 0.013),
     (0.0150, 0.0175), (0.0138, 0.0205), (0.0100, 0.0220), (0, 0.022)], seg=12)), smooth=60)


def dog_muzzle():
    # blunt muzzle (+Y forward, +X down): flatter on top, full flews below,
    # a nose pad on the upper front
    s = lathe([(0, -0.009), (0.0098, -0.009), (0.0096, -0.004), (0.0086, 0.003), (0.0074, 0.0065),
               (0.0052, 0.0086), (0.0026, 0.0090), (0, 0.009)], seg=10)

    def w(p):
        if p.x < 0:
            p.x *= 0.80
        else:
            p.x *= 1.0 + 0.10 * max(0.0, (p.y + 0.009) / 0.018)
        return p
    nose = sphere(0.0034, seg=8, rings=5).scale(1.0, 0.8, 1.2).move(-0.0040, 0.0072, 0)
    return fitf(s.warp(w) + nose, f_cone(0.010, 0.018))

part('CastIronDoorStops__dogMuzzle', f_cone(0.010, 0.018), dog_muzzle, smooth=55)


def dog_ear():
    # long spaniel ear hanging down, fuller and wavier towards its tip
    rings = []
    for y, k in ((0.0112, 0.35), (0.0080, 0.70), (0.0030, 0.92), (-0.0030, 1.0), (-0.0085, 0.88), (-0.0112, 0.35)):
        rings.append([(0.004 * k * math.cos(TAU * i / 8), y, 0.0056 * k * math.sin(TAU * i / 8)) for i in range(8)])
    return fitf(loft(rings), f_sphere(0.008))

part('CastIronDoorStops__dogEar', f_sphere(0.008), dog_ear, smooth=70)

# stubby leg ending in a paw with toe grooves
part('CastIronDoorStops__dogLeg', f_cyl(0.006, 0.006, 0.020), lambda: knurl(lathe(
    [(0, -0.010), (0.0056, -0.010), (0.0060, -0.0082), (0.0052, -0.0055), (0.0043, -0.0035),
     (0.0042, 0.0060), (0.0047, 0.0100), (0, 0.010)], seg=8), 8, 0.0008, -0.0095, -0.0060), smooth=50)

# tail: tapering and curling up
part('CastIronDoorStops__dogTail', f_cyl(0.004, 0.004, 0.020), lambda: fitf(bend(lathe(
    [(0, -0.010), (0.0034, -0.0098), (0.0040, -0.0060), (0.0036, 0.0000), (0.0026, 0.0060),
     (0.0014, 0.0094), (0, 0.010)], seg=6), 0.7, 'z', 'y'), f_cyl(0.004, 0.004, 0.020)), smooth=60)


def boot_vamp():
    # the foot of the boot: a sole with a welt, the upper sloping down from
    # the instep to the toe (+X)
    st = [(-0.0375, 0.0175, -0.0110, 0.0150), (-0.0300, 0.0182, -0.0110, 0.0150), (-0.0150, 0.0182, -0.0110, 0.0148),
          (0.0000, 0.0180, -0.0110, 0.0120), (0.0150, 0.0175, -0.0110, 0.0060), (0.0300, 0.0160, -0.0110, 0.0020),
          (0.0375, 0.0130, -0.0110, 0.0000)]
    upper = xloft(st, n=10, p=3.2)
    sole = yloft([(-0.015, 0.0006), (-0.0132, 0.0), (-0.0116, 0.0), (-0.0106, 0.0008)], 0.075, 0.038, 0.012, 2)
    return fitf(upper + sole, f_box(0.075, 0.030, 0.038))

part('CastIronDoorStops__bootFoot', f_box(0.075, 0.030, 0.038), boot_vamp, smooth=45)


def boot_toe():
    # rounded toe box (modelled at the game's 1 x 0.8 x 0.9 stretch)
    rings = []
    for x in (-0.018, -0.008, 0.0, 0.007, 0.012, 0.0155, 0.0172, 0.018):
        k = 1.0 if x <= 0 else math.sqrt(max(0.0, 1 - (x / 0.018) ** 2))
        k = max(k, 0.12)
        rings.append([(x, y - 0.0012 * (1 - k), z) for z, y in sell(0.0162 * k, 0.0144 * k, 12, 2.6)])
    s = loft(rings)
    return fitf(s.scale(1.0, 1 / 0.8, 1 / 0.9), f_sphere(0.018))

part('CastIronDoorStops__bootToe', f_sphere(0.018), boot_toe, smooth=50)

# stacked leather heel: narrower at the tread, two lift lines
part('CastIronDoorStops__bootHeel', f_box(0.018, 0.014, 0.030), lambda: yloft(
    [(-0.007, 0.0020), (-0.0045, 0.0016), (-0.0040, 0.0022), (-0.0035, 0.0014), (-0.0005, 0.0010),
     (0.0000, 0.0016), (0.0005, 0.0008), (0.0070, 0.0003)], 0.018, 0.030, 0.0045, 2))


def boot_shaft():
    # the leg of the boot: narrower at the ankle, flaring to the top, hollow
    # inside the opening
    w = d = 0.038
    outer = [(-0.0375, 0.0020), (-0.0200, 0.0030), (0.0000, 0.0020), (0.0300, 0.0003), (0.0375, 0.0)]
    rings = [[(x, y, z) for x, z in rrect_pts(w - 2 * ins, d - 2 * ins, max(0.002, 0.010 - ins), 2)]
             for y, ins in outer]
    for y, ins in ((0.0370, 0.0030), (0.0280, 0.0036), (0.0120, 0.0040)):
        rings.append([(x, y, z) for x, z in rrect_pts(w - 2 * ins, d - 2 * ins, max(0.002, 0.010 - ins), 2)])
    return loft(rings)

part('CastIronDoorStops__bootShaft', f_box(0.038, 0.075, 0.038), boot_shaft)

# lace segment: a small rounded rod
part('CastIronDoorStops__lace', f_cyl(0.0015, 0.0015, 0.006), lambda: lathe(
    [(0, -0.003), (0.0010, -0.0028), (0.0015, -0.0018), (0.0015, 0.0018), (0.0010, 0.0028), (0, 0.003)],
    seg=6), smooth=70)


def boot_cuff():
    # rolled top edge, following the shaft's rounded-square opening
    path = rrect_pts(0.030, 0.030, 0.008, 3)
    sec = circ_sec(0.003, 6)
    n, m = len(path), len(sec)
    v, f = [], []
    for i, (px, py) in enumerate(path):
        qx, qy = path[(i + 1) % n]
        ox, oy = path[i - 1]
        tx, ty = qx - ox, qy - oy
        l = math.hypot(tx, ty)
        nx, ny = ty / l, -tx / l
        if nx * px + ny * py < 0:
            nx, ny = -nx, -ny
        for dr, dz in sec:
            v.append((px + nx * dr, py + ny * dr, dz))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    return fitf(_fw(Shape(v, f)), f_torus(0.015, 0.003))

part('CastIronDoorStops__bootCuff', f_torus(0.015, 0.003), boot_cuff, smooth=60)


def owl_body():
    # egg-shaped owl (modelled at the game's 1 x 1.2 x 0.9 stretch): rows of
    # scalloped breast feathers down the front (+X), smooth back
    prof = [(0, -0.042), (0.016, -0.0405), (0.026, -0.0350), (0.0325, -0.0250), (0.0350, -0.0120),
            (0.0348, 0.0000), (0.0335, 0.0100), (0.0305, 0.0200), (0.0255, 0.0290), (0.0185, 0.0360),
            (0.0095, 0.0405), (0, 0.042)]
    s = lathe(prof, seg=18).scale(1.0, 1.0, 0.9)

    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6 or p.y < -0.036 or p.y > 0.034:
            return p
        th = math.atan2(p.z, p.x)
        front = max(0.0, math.cos(th)) ** 0.7
        ph = ((p.y + 0.042) / 0.0105 + 0.28 * math.cos(6 * th)) % 1.0
        k = 1.0 + 0.045 * front * (1.0 - ph)
        p.x *= k
        p.z *= k
        return p
    s = s.warp(w)
    return fitf(s.scale(1.0, 1 / 1.2, 1 / 0.9), f_sphere(0.035))

part('CastIronDoorStops__owlBody', f_sphere(0.035), owl_body, smooth=50)

# head: round crown, a dished facial disc with a raised rim facing +X and
# a ridge down the middle where the beak comes out
part('CastIronDoorStops__owlHead', f_sphere(0.024), lambda: x_axis(lathe(
    [(0, -0.024), (0.0120, -0.0205), (0.0200, -0.0120), (0.0240, 0.0000), (0.0225, 0.0100),
     (0.0180, 0.0180), (0.0150, 0.0236), (0.0128, 0.0232), (0.0092, 0.0188), (0.0050, 0.0186),
     (0.0022, 0.0202), (0, 0.0212)], seg=12)), smooth=50)

# eye rim: a rounded ring round each staring eye
part('CastIronDoorStops__owlEyeRing', f_torus(0.008, 0.002), lambda: ring_section(
    0.008, circ_sec(0.002, 5), seg=10), smooth=60)

# hooked beak (+Y out of the face, bending towards +X = down)
part('CastIronDoorStops__owlBeak', f_cone(0.004, 0.008), lambda: fitf(bend(lathe(
    [(0, -0.004), (0.0040, -0.004), (0.0034, -0.001), (0.0022, 0.0015), (0.0010, 0.0034), (0, 0.004)],
    seg=6), 0.9, 'z', 'y'), f_cone(0.004, 0.008)), smooth=50)


def owl_wing():
    # folded wing (modelled at the game's 0.6 x 1.2 x 0.5 stretch): a lens
    # of rounded coverts ending in three stepped flight-feather tips
    out = [(0.0, 0.0168), (0.0050, 0.0150), (0.0078, 0.0090), (0.0084, 0.0010), (0.0076, -0.0070),
           (0.0058, -0.0125), (0.0040, -0.0110), (0.0026, -0.0150), (0.0006, -0.0135), (-0.0012, -0.0168),
           (-0.0036, -0.0120), (-0.0064, -0.0060), (-0.0080, 0.0020), (-0.0070, 0.0100), (-0.0040, 0.0152)]
    rings = []
    for z, k in ((-0.0070, 0.55), (-0.0036, 0.90), (0.0, 1.0), (0.0036, 0.90), (0.0070, 0.55)):
        rings.append([(x * k, y * (0.6 + 0.4 * k), z) for x, y in out])
    s = loft(rings)
    return fitf(s.scale(1 / 0.6, 1 / 1.2, 1 / 0.5), f_sphere(0.014))

part('CastIronDoorStops__owlWing', f_sphere(0.014), owl_wing, smooth=50)


def owl_foot():
    # a leg stub and four talons gripping the plinth (three forward, +X)
    s = cyl(0.0020, 0.0024, 0.006, seg=5, bevel_r=0).move(y=0.001)
    for a in (-0.7, 0.0, 0.7, math.pi):
        ca, sa = math.cos(a), math.sin(a)
        L = 0.0046 if a != math.pi else 0.0036
        s = s + tube([(0, -0.0012, 0), (ca * L * 0.6, -0.0025, sa * L * 0.6), (ca * L, -0.0040, sa * L)],
                     0.0012, seg=4, r_end=0.0006)
    return fitf(s, f_cyl(0.005, 0.005, 0.008))

part('CastIronDoorStops__owlFoot', f_cyl(0.005, 0.005, 0.008), owl_foot, smooth=60)

# ear tuft: a feathered spike, curving a little
part('CastIronDoorStops__owlTuft', f_cone(0.004, 0.010), lambda: fitf(bend(lathe(
    [(0, -0.005), (0.0036, -0.0048), (0.0040, -0.0030), (0.0030, 0.0000), (0.0016, 0.0030), (0, 0.005)],
    seg=6), 0.4, 'z', 'y'), f_cone(0.004, 0.010)), smooth=45)


# ======================================== frames: correspondence, crests ==
part('CorrespondenceCards__bar', f_box(0.228, 0.014, 0.008),
     lambda: frame_bar(0.228, 0.014, 0.008, back=0.45, edge=0.78, bead=0.12), smooth=40)
part('HeraldicCrests__bar', f_box(0.336, 0.018, 0.012),
     lambda: frame_bar(0.336, 0.018, 0.012, back=0.40, edge=0.72, bead=0.16), smooth=40)


# ================================================= empty painting hooks ==
# picture nail: thin shank, a flat round head with a domed top (+Y is out
# of the wall)
part('EmptyPaintingOutlines__nail', f_cyl(0.0012, 0.0012, 0.006), lambda: lathe(
    [(0, -0.003), (0.0004, -0.0030), (0.0005, -0.0024), (0.0005, 0.0018), (0.0012, 0.0020),
     (0.0012, 0.0025), (0.0008, 0.0029), (0, 0.003)], seg=8), smooth=50)

HOOK = torus_arc_bounds(0.003, 0.0008, math.pi)
# the bent wire hook, thinning to its tip
part('EmptyPaintingOutlines__hook', HOOK, lambda: fitf(ring_section(
    0.003, circ_sec(0.0008, 5), seg=8, arc=math.pi,
    rmod=lambda a, i: (1.0 - 0.3 * (a / math.pi) ** 2, 0.0)), HOOK), smooth=60)


def frayed_wire():
    # two-strand twisted picture wire left on the hook, its broken lower end
    # (-Y) fraying into loose strands
    rings = []
    for k in range(17):
        y = -0.024 + 0.054 * k / 16
        ring = []
        for i in range(6):
            a = TAU * i / 6
            rr_ = 0.0006 * (0.8 + 0.2 * math.cos(2 * (a - 1.1 * k)))
            ring.append((rr_ * math.sin(a), y, rr_ * math.cos(a)))
        rings.append(ring)
    s = loft(rings)
    for a in (0.0, 2.1, 4.2):
        ca, sa = math.cos(a), math.sin(a)
        s = s + tube([(0.0002 * ca, -0.0235, 0.0002 * sa), (0.0005 * ca, -0.027, 0.0005 * sa),
                      (0.0008 * ca, -0.030, 0.0008 * sa)], 0.00022, seg=4)
    return fitf(s, f_cyl(0.0008, 0.0008, 0.06))

part('EmptyPaintingOutlines__wire', f_cyl(0.0008, 0.0008, 0.06), frayed_wire, smooth=60)


# ================================================== chess side stands ==
# round top: flat, a thumbnail-moulded edge, bevelled underneath
part('ChessSideStands__top', f_cyl(0.19, 0.19, 0.014), lambda: lathe(
    [(0, -0.007), (0.172, -0.007), (0.181, -0.0062), (0.1868, -0.0045), (0.1900, -0.0010),
     (0.1900, 0.0028), (0.1880, 0.0058), (0.1850, 0.0070), (0, 0.007)], seg=40))

# turned pedestal: collar and bead at the foot, a vase, necks and rings
part('ChessSideStands__pedestal', f_cyl(0.020, 0.030, 0.465), lambda: lathe(
    [(0, -0.2325), (0.0300, -0.2325), (0.0300, -0.2150), (0.0245, -0.2080), (0.0270, -0.1980),
     (0.0220, -0.1860), (0.0250, -0.1500), (0.0275, -0.1050), (0.0255, -0.0600), (0.0190, -0.0150),
     (0.0140, 0.0300), (0.0135, 0.0600), (0.0185, 0.0700), (0.0185, 0.0800), (0.0140, 0.0900),
     (0.0150, 0.1600), (0.0135, 0.1900), (0.0185, 0.2000), (0.0200, 0.2150), (0.0200, 0.2325),
     (0, 0.2325)], seg=14))

# splayed leg: turned taper with a ring at each end
part('ChessSideStands__leg', f_cyl(0.010, 0.015, 0.14), lambda: lathe(
    [(0, -0.070), (0.0130, -0.070), (0.0150, -0.064), (0.0125, -0.056), (0.0120, -0.030),
     (0.0100, 0.040), (0.0092, 0.056), (0.0100, 0.062), (0.0096, 0.070), (0, 0.070)], seg=10))

# bun foot
part('ChessSideStands__foot', f_cyl(0.015, 0.018, 0.008), lambda: lathe(
    [(0, -0.004), (0.0160, -0.004), (0.0180, -0.0020), (0.0178, 0.0010), (0.0160, 0.0034),
     (0.0120, 0.0040), (0, 0.004)], seg=14), smooth=50)


def chess_board():
    # board with a rounded edge and a sunk inlay line round the playing field
    s = yloft([(-0.005, 0.0015), (-0.0038, 0.0), (0.0030, 0.0), (0.0045, 0.0010), (0.005, 0.0022)],
              0.30, 0.30, 0.004, 2)
    g, d = 0.0012, 0.0012
    cutters = [box(0.2904, d * 2, g).move(z=0.1446, y=0.005), box(0.2904, d * 2, g).move(z=-0.1446, y=0.005),
               box(g, d * 2, 0.2904).move(x=0.1446, y=0.005), box(g, d * 2, 0.2904).move(x=-0.1446, y=0.005)]
    return cut(s, *cutters)

part('ChessSideStands__board', f_box(0.30, 0.010, 0.30), chess_board)

# piece base: felted foot, bead and a stepped top
part('ChessSideStands__base', f_cyl(0.010, 0.011, 0.004), lambda: lathe(
    [(0, -0.002), (0.0104, -0.002), (0.0110, -0.0012), (0.0108, -0.0002), (0.0097, 0.0004),
     (0.0099, 0.0012), (0.0086, 0.0019), (0.0055, 0.002), (0, 0.002)], seg=12), smooth=45)

# pawn stem: flared foot, waist, collar under the ball
part('ChessSideStands__pawnBody', f_cyl(0.004, 0.008, 0.018), lambda: lathe(
    [(0, -0.009), (0.0080, -0.009), (0.0062, -0.0060), (0.0038, -0.0010), (0.0031, 0.0040),
     (0.0056, 0.0062), (0.0036, 0.0085), (0, 0.009)], seg=8), smooth=45)
part('ChessSideStands__pawnHead', f_sphere(0.006), lambda: sphere(0.006, seg=10, rings=6), smooth=80)


def rook_tower():
    # tower: flared foot, slight taper, a flared parapet with a hollow top
    return lathe([(0, -0.014), (0.0080, -0.014), (0.0068, -0.0115), (0.0058, -0.0080), (0.0055, 0.0070),
                  (0.0064, 0.0090), (0.0080, 0.0110), (0.0080, 0.0140), (0.0060, 0.0140), (0.0058, 0.0115),
                  (0, 0.0115)], seg=12)

part('ChessSideStands__rookBody', f_cyl(0.008, 0.008, 0.028), rook_tower, smooth=45)
part('ChessSideStands__rookTooth', f_box(0.003, 0.004, 0.003), lambda: rbox(0.003, 0.004, 0.003, r=0.0005, seg=1))

# knight: turned stem, and a carved horse head (profile in YZ, muzzle +Z)
part('ChessSideStands__knightBody', f_cyl(0.006, 0.008, 0.020), lambda: lathe(
    [(0, -0.010), (0.0080, -0.010), (0.0068, -0.0075), (0.0050, -0.0040), (0.0045, 0.0060),
     (0.0060, 0.0080), (0.0060, 0.0100), (0, 0.010)], seg=10), smooth=45)


def knight_head():
    out = [(-0.0048, -0.0070), (0.0036, -0.0070), (0.0031, -0.0042), (0.0068, -0.0022), (0.0070, 0.0006),
           (0.0022, 0.0042), (0.0005, 0.0070), (-0.0012, 0.0062), (-0.0030, 0.0048), (-0.0052, 0.0016),
           (-0.0062, -0.0030), (-0.0056, -0.0058)]
    # outline is (z, y); loft it across X with chamfered faces
    rings = []
    for xx, k in ((-0.0040, 0.84), (-0.0028, 1.0), (0.0028, 1.0), (0.0040, 0.84)):
        rings.append([(xx, y * (0.5 + 0.5 * k) - 0.0003 * (1 - k), zz * k) for zz, y in out])
    s = loft(rings)

    def w(p):
        if p.z > 0.002:
            p.x *= 1.0 - 0.28 * min(1.0, (p.z - 0.002) / 0.005)
        return p
    return fitf(s.warp(w), f_box(0.008, 0.014, 0.014))

part('ChessSideStands__knightHead', f_box(0.008, 0.014, 0.014), knight_head, smooth=40)

# bishop: stem with a collar, and a mitre with its slit and a bead on top
part('ChessSideStands__bishopBody', f_cyl(0.005, 0.008, 0.026), lambda: lathe(
    [(0, -0.013), (0.0080, -0.013), (0.0066, -0.0100), (0.0046, -0.0050), (0.0036, 0.0060),
     (0.0058, 0.0090), (0.0058, 0.0104), (0.0042, 0.0118), (0.0040, 0.013), (0, 0.013)], seg=10), smooth=45)


def bishop_mitre():
    s = lathe([(0, -0.0072), (0.0050, -0.0050), (0.0060, -0.0005), (0.0046, 0.0034), (0.0020, 0.0060),
               (0, 0.0072)], seg=8)
    s = cut(s, box(0.0012, 0.005, 0.014).rot(z=0.55).move(x=0.0046, y=0.0018))
    return fitf(s.scale(1.0, 1 / 1.2, 1.0), f_sphere(0.006))

part('ChessSideStands__bishopHead', f_sphere(0.006), bishop_mitre, smooth=45)


def royal_stem():
    # queen / king stem: spreading foot, long waist, two collars at the top
    return lathe([(0, -0.5), (1.00, -0.5), (0.86, -0.42), (0.62, -0.26), (0.52, 0.20), (0.60, 0.32),
                  (0.74, 0.36), (0.74, 0.40), (0.60, 0.44), (0.74, 0.46), (0.78, 0.50), (0, 0.5)], seg=12)

part('ChessSideStands__royalBody', f_cyl(0.007, 0.009, 0.030), lambda: fitf(royal_stem(), f_cyl(0.007, 0.009, 0.030)),
     smooth=45)

# coronet: a flaring cup with a bead rim and a dished top
part('ChessSideStands__coronet', f_cyl(0.010, 0.008, 0.006), lambda: lathe(
    [(0, -0.003), (0.0078, -0.003), (0.0088, 0.0008), (0.0100, 0.0024), (0.0092, 0.0030),
     (0.0074, 0.0022), (0, 0.0018)], seg=8), smooth=45)

# crown point with a ball tip
part('ChessSideStands__point', f_cone(0.002, 0.005), lambda: lathe(
    [(0, -0.0025), (0.0020, -0.0025), (0.0010, 0.0006), (0.0007, 0.0012), (0.0010, 0.0019), (0, 0.0025)],
    seg=6), smooth=60)
part('ChessSideStands__crossV', f_box(0.002, 0.010, 0.002), lambda: rbox(0.002, 0.010, 0.002, r=0.0004, seg=1))
part('ChessSideStands__crossH', f_box(0.006, 0.002, 0.002), lambda: rbox(0.006, 0.002, 0.002, r=0.0004, seg=1))


def ash_tray():
    # shallow round ashtray, two cigar rests notched into the rim
    s = vessel([(0.0215, -0.002), (0.0240, -0.0012), (0.0250, 0.0008), (0.0250, 0.0016)], 0.0022, seg=16,
               floor=0.0010)
    s = cut(s, cyl(0.0035, 0.0035, 0.07, seg=8, bevel_r=0).rot(z=math.pi / 2).move(y=0.0040))
    return fitf(s, f_cyl(0.025, 0.025, 0.004))

part('ChessSideStands__tray', f_cyl(0.025, 0.025, 0.004), ash_tray)

# cigar: cut foot at -Y (the lit end), a gentle swell, band, rounded cap
part('ChessSideStands__cigar', f_cyl(0.005, 0.005, 0.055), lambda: lathe(
    [(0, -0.0275), (0.0044, -0.0275), (0.0048, -0.0265), (0.0050, -0.0100), (0.0049, 0.0120),
     (0.0051, 0.0140), (0.0051, 0.0190), (0.0048, 0.0200), (0.0046, 0.0230), (0.0036, 0.0262),
     (0.0015, 0.0274), (0, 0.0275)], seg=10), smooth=50)


# =========================================== miniature portrait clusters ==
# gilt oval frame (the game stretches it 1.3 tall): an inner sight bead, a
# cove, the rounded outer moulding, flat back; front is +Z
part('MiniaturePortraitClusters__frame', f_torus(0.05, 0.006), lambda: ring_section(
    0.05, [(-0.0060, -0.0060), (-0.0060, 0.0008), (-0.0048, 0.0030), (-0.0034, 0.0018), (-0.0016, 0.0022),
           (0.0005, 0.0050), (0.0030, 0.0060), (0.0052, 0.0042), (0.0060, 0.0010), (0.0058, -0.0060)],
    seg=28), smooth=40)

MINI_HOOK = torus_arc_bounds(0.003, 0.001, math.pi)
part('MiniaturePortraitClusters__hook', MINI_HOOK, lambda: fitf(ring_section(
    0.003, circ_sec(0.001, 5), seg=8, arc=math.pi,
    rmod=lambda a, i: (1.0 - 0.3 * (a / math.pi) ** 2, 0.0)), MINI_HOOK), smooth=60)


def bead_rod():
    # gilt bead-and-reel rod joining the frames
    prof = [(0, -0.075), (0.0012, -0.0748)]
    n = 11
    for i in range(n):
        y0 = -0.072 + 0.144 * i / n
        step = 0.144 / n
        prof += [(0.0018, y0 + step * 0.20), (0.0020, y0 + step * 0.45), (0.0017, y0 + step * 0.70),
                 (0.0010, y0 + step * 0.88)]
    prof += [(0.0012, 0.0748), (0, 0.075)]
    return fitf(x_axis(lathe(prof, seg=6)), f_box(0.15, 0.004, 0.003))

part('MiniaturePortraitClusters__decor', f_box(0.15, 0.004, 0.003), bead_rod, smooth=60)


# ============================================== peacock feather vases ==
# foot ring: widening up to a rounded shoulder under the body
part('PeacockFeatherVases__base', f_cyl(0.055, 0.0495, 0.012), lambda: lathe(
    [(0, -0.006), (0.0495, -0.006), (0.0512, -0.0045), (0.0514, -0.0020), (0.0540, 0.0010),
     (0.0550, 0.0035), (0.0538, 0.0060), (0, 0.006)], seg=32))


def vase_lower(h=0.2475):
    # lower body swelling out to the hip, open where it meets the parts
    # above and below
    prof = []
    for i in range(8):
        t = i / 7
        prof.append((0.055 + 0.020 * (1 - (1 - t) ** 1.5), -h / 2 + h * t))
    return lathe(prof, seg=28, cap_bottom=False, cap_top=False)

part('PeacockFeatherVases__lower', f_cyl(0.075, 0.055, 0.2475), vase_lower)


def vase_upper(h=0.2025):
    # full rounded shoulder drawn in to a short neck; the mouth is hollow
    ts = [(0.0, 0.0750), (0.25, 0.0746), (0.50, 0.0725), (0.667, 0.0662), (0.78, 0.0572), (0.87, 0.0455),
          (0.93, 0.0372), (0.965, 0.0336), (1.0, 0.0350)]
    prof = [(r, -h / 2 + h * t) for t, r in ts]
    prof += [(0.0318, h / 2 - 0.004), (0.0300, h / 2 - 0.030), (0, h / 2 - 0.034)]
    return lathe(prof, seg=26, cap_bottom=False)

part('PeacockFeatherVases__upper', f_cyl(0.035, 0.075, 0.2025), vase_upper)

# flared lip rolled over at the mouth (-Z is up once laid flat)
part('PeacockFeatherVases__rim', f_torus(0.035, 0.006), lambda: ring_section(
    0.035, [(-0.0060, 0.0040), (-0.0052, -0.0020), (-0.0022, -0.0060), (0.0020, -0.0058), (0.0052, -0.0038),
            (0.0060, -0.0005), (0.0040, 0.0035), (0.0005, 0.0060)], seg=17), smooth=50)

# gilt band: flat against the vase, rounded outside
part('PeacockFeatherVases__trim', f_torus(0.0675, 0.002), lambda: ring_section(
    0.0675, [(-0.0020, -0.0020), (0.0008, -0.0020), (0.0020, -0.0008), (0.0020, 0.0008), (0.0008, 0.0020),
             (-0.0020, 0.0020)], seg=40), smooth=50)

# rachis: the bare quill at the base (-Y), tapering towards the eye
part('PeacockFeatherVases__shaft', f_cyl(0.001, 0.0015, 0.42), lambda: lathe(
    [(0, -0.21), (0.0012, -0.2095), (0.0015, -0.19), (0.0013, -0.12), (0.0010, 0.10), (0.0006, 0.2095),
     (0, 0.21)], seg=5), smooth=60)


def feather_vane():
    # a tuft of herl along the rachis (modelled at the game's 0.3 x 0.5 x
    # 1.5 stretch): long along Z, fringed top and bottom edges, thin in X
    r = 0.014
    L, H = 1.5 * r, 0.5 * r
    top, bot = [], []
    n = 11
    for i in range(n + 1):
        z = -L + 2 * L * i / n
        env = math.sqrt(max(0.0, 1 - (z / L) ** 2)) ** 0.8
        fr = 1.0 if i % 2 else 0.72
        top.append((z, H * env * fr))
        bot.append((z, -H * env * (0.72 if i % 2 else 1.0)))
    out = top + list(reversed(bot[1:-1]))
    rings = []
    for x, k in ((-0.3 * r, 0.35), (0.0, 1.0), (0.3 * r, 0.35)):
        rings.append([(x, y * (0.4 + 0.6 * k), z * (0.85 + 0.15 * k)) for z, y in out])
    return fitf(loft(rings), f_sphere(r))

part('PeacockFeatherVases__barb', f_sphere(0.014), feather_vane, smooth=70)


# ===================================================== bellows cameras ==
def square_ring(half, rc, sec, cseg=2):
    """Sweep a (dr, dz) section round a rounded square in the XY plane."""
    path = rrect_pts(2 * half, 2 * half, rc, cseg)
    n, m = len(path), len(sec)
    v, f = [], []
    for i, (px, py) in enumerate(path):
        qx, qy = path[(i + 1) % n]
        ox, oy = path[i - 1]
        tx, ty = qx - ox, qy - oy
        l = math.hypot(tx, ty)
        nx, ny = ty / l, -tx / l
        if nx * px + ny * py < 0:
            nx, ny = -nx, -ny
        for dr, dz in sec:
            v.append((px + nx * dr, py + ny * dr, dz))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    return _fw(Shape(v, f))


# tripod leg: thick lower section, a clamp collar, slimmer upper section and
# the hinge knuckle at the head end (+Y)
part('BellowsCameras__leg', f_cyl(0.008, 0.014, 1.15), lambda: lathe(
    [(0, -0.575), (0.0110, -0.575), (0.0135, -0.565), (0.0140, -0.540), (0.0126, -0.050),
     (0.0138, -0.042), (0.0138, 0.000), (0.0104, 0.008), (0.0094, 0.020), (0.0084, 0.500),
     (0.0095, 0.540), (0.0095, 0.566), (0.0068, 0.575), (0, 0.575)], seg=10))

# rubber foot: ribbed cap with a rounded tread
part('BellowsCameras__foot', f_cyl(0.010, 0.012, 0.008), lambda: lathe(
    [(0, -0.004), (0.0110, -0.004), (0.0120, -0.0030), (0.0120, -0.0012), (0.0112, -0.0006),
     (0.0114, 0.0010), (0.0106, 0.0016), (0.0104, 0.0030), (0.0092, 0.0040), (0, 0.004)], seg=12), smooth=50)

# brass stay with swelled eye ends
part('BellowsCameras__brace', f_cyl(0.002, 0.002, 0.14), lambda: lathe(
    [(0, -0.070), (0.0014, -0.070), (0.0020, -0.066), (0.0020, -0.060), (0.0012, -0.056),
     (0.0012, 0.056), (0.0020, 0.060), (0.0020, 0.066), (0.0014, 0.070), (0, 0.070)], seg=6), smooth=50)

# tripod head: a cast bowl where the legs hinge, a column and platform
part('BellowsCameras__hub', f_sphere(0.020), lambda: lathe(
    [(0, -0.020), (0.0100, -0.0185), (0.0170, -0.0120), (0.0200, -0.0040), (0.0200, 0.0040),
     (0.0150, 0.0070), (0.0118, 0.0090), (0.0118, 0.0140), (0.0160, 0.0160), (0.0160, 0.0200),
     (0, 0.020)], seg=12), smooth=40)


def rear_standard():
    # rear standard: eased wooden frame, the back sunk where the ground
    # glass sits, a plate-holder slot along the top
    s = rbox(0.16, 0.14, 0.020, r=0.0025, seg=2)
    s = cut(s, box(0.112, 0.092, 0.008).move(z=-0.0115), box(0.150, 0.004, 0.010).move(y=0.0712, z=-0.001))
    return s

part('BellowsCameras__rearPlate', f_box(0.16, 0.14, 0.020), rear_standard)


def bellows():
    # square leather bellows, pleated, tapering from the rear standard (-Y)
    # to the lens board (+Y)
    rings = []
    folds = 8
    for k in range(2 * folds + 1):
        t = k / (2 * folds)
        y = -0.07 + 0.14 * t
        half = 0.055 - 0.008 * t - (0.0045 if k % 2 else 0.0)
        rings.append([(x, y, z) for x, z in rrect_pts(2 * half, 2 * half, 0.008, 2)])
    return fitf(loft(rings), f_cyl(0.055, 0.055, 0.14))

part('BellowsCameras__bellows', f_cyl(0.055, 0.055, 0.14), bellows, smooth=40)

# a fold ridge: rounded-square, like the bellows it belongs to
part('BellowsCameras__ridge', f_torus(0.056, 0.003), lambda: fitf(square_ring(
    0.056, 0.012, circ_sec(0.003, 5)), f_torus(0.056, 0.003)), smooth=40)


def lens_board():
    # lens standard: board with a chamfered front margin
    w, h = 0.14, 0.12
    lv = [(-0.007, 0.0), (0.0035, 0.0), (0.0058, 0.0020), (0.0070, 0.0045)]
    return loft([[(x, y, z) for x, y in rrect_pts(w - 2 * i, h - 2 * i, 0.003, 2)] for z, i in lv])

part('BellowsCameras__frontBoard', f_box(0.14, 0.12, 0.014), lens_board)


def brass_lens():
    # brass lens (+Y forward): flange on the board, barrel, a knurled front
    # cell and the hood lip round the glass
    s = lathe([(0, -0.0225), (0.0280, -0.0225), (0.0280, -0.0190), (0.0215, -0.0175), (0.0215, 0.0040),
               (0.0250, 0.0070), (0.0250, 0.0160), (0.0235, 0.0185), (0.0240, 0.0225), (0, 0.0225)], seg=16)
    return knurl(s, 16, 0.0010, 0.0069, 0.0161)

part('BellowsCameras__lens', f_cyl(0.028, 0.028, 0.045), brass_lens, smooth=40)

# knurled brass knob on a short neck (focus knob and the top thumbscrew)
part('BellowsCameras__knob', f_cyl(0.006, 0.006, 0.008), lambda: knurl(lathe(
    [(0, -0.004), (0.0030, -0.004), (0.0030, -0.0022), (0.0060, -0.0016), (0.0060, 0.0030),
     (0.0050, 0.0040), (0, 0.004)], seg=10), 10, 0.0007, -0.0017, 0.0031), smooth=40)


def cloth(w, h, t, nx, ny, fold, seed, hang=False):
    # dark focusing cloth: a thin slab (w x h, thickness along Z) with soft
    # folds; a hanging flap folds more towards its lower edge
    s = slab(w, h, t, nx, ny)

    def f_(p):
        k = 0.35 + 0.65 * ((h / 2 - p.y) / h) if hang else 1.0
        p.z += fold * k * (math.sin(p.x / w * 9.0 + seed) * 0.7 + math.sin(p.y / h * 5.0 + p.x / w * 3 + seed * 2) * 0.3)
        return p
    return s.warp(f_)

part('BellowsCameras__clothTop', f_box(0.20, 0.008, 0.12), lambda: fitf(
    cloth(0.20, 0.12, 0.0025, 12, 8, 0.0022, 0.4).rot(x=-math.pi / 2), f_box(0.20, 0.008, 0.12)), smooth=60)
part('BellowsCameras__clothSide', f_box(0.008, 0.20, 0.10), lambda: fitf(
    cloth(0.10, 0.20, 0.0025, 8, 10, 0.0024, 1.3, True).rot(y=math.pi / 2), f_box(0.008, 0.20, 0.10)), smooth=60)
part('BellowsCameras__clothBack', f_box(0.20, 0.15, 0.008), lambda: fitf(
    cloth(0.20, 0.15, 0.0025, 12, 8, 0.0024, 2.1, True), f_box(0.20, 0.15, 0.008)), smooth=60)

# flash gun: rod with a knurled grip and collars; T-bar with end caps
part('BellowsCameras__flashRod', f_cyl(0.003, 0.003, 0.25), lambda: knurl(lathe(
    [(0, -0.125), (0.0024, -0.125), (0.0030, -0.121), (0.0030, -0.070), (0.0022, -0.066),
     (0.0020, 0.110), (0.0030, 0.114), (0.0030, 0.125), (0, 0.125)], seg=8), 8, 0.0004, -0.119, -0.071),
     smooth=45)
part('BellowsCameras__tBar', f_cyl(0.003, 0.003, 0.08), lambda: lathe(
    [(0, -0.040), (0.0022, -0.0398), (0.0030, -0.037), (0.0026, -0.033), (0.0019, -0.030), (0.0019, 0.030),
     (0.0026, 0.033), (0.0030, 0.037), (0.0022, 0.0398), (0, 0.040)], seg=8), smooth=50)

# flash pan: shallow brass trough, widening to a lip
part('BellowsCameras__flashPan', f_cyl(0.020, 0.015, 0.008), lambda: fitf(vessel(
    [(0.0150, -0.004), (0.0175, -0.001), (0.0195, 0.0025), (0.0200, 0.0035)], 0.0012, seg=16, floor=0.0012),
    f_cyl(0.020, 0.015, 0.008)))


def powder_heap():
    # little heap of magnesium powder (modelled at the game's 1 x 0.35 x 1)
    s = lathe([(0, -0.0028), (0.0076, -0.0028), (0.0080, -0.0020), (0.0060, -0.0002), (0.0032, 0.0018),
               (0, 0.0028)], seg=10)
    s = noisy(s, 0.0005, scale=600, seed=3)
    return fitf(s.scale(1.0, 1 / 0.35, 1.0), f_sphere(0.008))

part('BellowsCameras__powder', f_sphere(0.008), powder_heap, smooth=70)


# ======================================================== brass sconces ==
def sconce_plate():
    # cast backplate: a waisted cartouche with scrolled points top and
    # bottom, bevelled, with a round boss where the arm leaves
    half = [(0.0, 0.0350), (0.0060, 0.0322), (0.0100, 0.0268), (0.0170, 0.0238), (0.0225, 0.0180),
            (0.0178, 0.0000), (0.0225, -0.0180), (0.0170, -0.0238),
            (0.0100, -0.0268), (0.0060, -0.0322), (0.0, -0.0350)]
    out = half + [(-x, y) for x, y in reversed(half[1:-1])]
    s = prism(out, 0.0045, bevel_r=0.0008).move(z=-0.00075)
    boss = lathe([(0, -0.001), (0.0100, -0.001), (0.0100, 0.0012), (0.0085, 0.0024), (0.0050, 0.0030),
                  (0, 0.0030)], seg=10).rot(x=math.pi / 2).move(y=0.008)
    return fitf(s + boss, f_box(0.045, 0.070, 0.006))

part('BrassSconces__plate', f_box(0.045, 0.070, 0.006), sconce_plate)


def slot_screw():
    s = lathe([(0, -0.0015), (0.0020, -0.0015), (0.0020, -0.0004), (0.0014, 0.0009), (0, 0.0015)], seg=8)
    return cut(s, box(0.005, 0.0012, 0.0006).move(y=0.0014))

part('BrassSconces__screw', f_cyl(0.002, 0.002, 0.003), slot_screw, smooth=50)

# turned arm section: end collars, a bead, slender shanks
part('BrassSconces__arm', f_cyl(0.006, 0.006, 0.045), lambda: lathe(
    [(0, -0.0225), (0.0056, -0.0225), (0.0060, -0.0200), (0.0044, -0.0180), (0.0037, -0.0080),
     (0.0049, -0.0040), (0.0049, 0.0000), (0.0037, 0.0040), (0.0035, 0.0170), (0.0056, 0.0200),
     (0.0060, 0.0225), (0, 0.0225)], seg=10), smooth=45)

# melon knop: gadrooned ball between two collars
part('BrassSconces__knop', f_sphere(0.010), lambda: knurl(lathe(
    [(0, -0.010), (0.0042, -0.0094), (0.0042, -0.0080), (0.0082, -0.0052), (0.0100, 0.0000),
     (0.0082, 0.0052), (0.0042, 0.0080), (0.0042, 0.0094), (0, 0.010)], seg=16), 16, 0.0012, -0.006, 0.006),
     smooth=40)

# bobeche: a dished drip pan with a flared rim and the candle socket
part('BrassSconces__dripCup', f_cyl(0.025, 0.020, 0.010), lambda: lathe(
    [(0, -0.005), (0.0130, -0.005), (0.0200, -0.0025), (0.0240, 0.0015), (0.0250, 0.0050),
     (0.0228, 0.0048), (0.0160, 0.0012), (0.0148, 0.0050), (0.0133, 0.0050), (0.0133, 0.0025),
     (0, 0.0025)], seg=14))

# scalloped edge of the drip pan (-Z is up once laid flat)
part('BrassSconces__dripRim', f_torus(0.023, 0.002), lambda: fitf(ring_section(
    0.023, circ_sec(0.002, 5), seg=24, rmod=lambda a, i: (1.0, 0.0, 0.0010 * math.cos(8 * a))),
    f_torus(0.023, 0.002)), smooth=50)


def melted_candle():
    # wax candle: a slight taper, a melted rim round a shallow crater with
    # the wick stub in the middle, two runs of wax down the side
    s = lathe([(0, -0.060), (0.0128, -0.060), (0.0130, -0.058), (0.0125, 0.050), (0.0122, 0.0560),
               (0.0118, 0.0595), (0.0104, 0.0600), (0.0090, 0.0584), (0.0040, 0.0575), (0.0014, 0.0584),
               (0, 0.0600)], seg=16)
    for a, L in ((0.6, 0.030), (2.9, 0.018)):
        ca, sa = math.sin(a), math.cos(a)
        pts = [(0.0116, 0.0590), (0.0122, 0.0560), (0.0124, 0.0590 - L * 0.5), (0.0124, 0.0590 - L)]
        s = s + tube([(r * ca, y, r * sa) for r, y in pts], 0.0010, seg=5, r_end=0.0015)
    return fitf(s, f_cyl(0.012, 0.013, 0.12))

part('BrassSconces__candle', f_cyl(0.012, 0.013, 0.12), melted_candle, smooth=55)

# charred wick curling over
part('BrassSconces__wick', f_cyl(0.0008, 0.0008, 0.004), lambda: fitf(bend(lathe(
    [(0, -0.002), (0.0008, -0.002), (0.0008, 0.0008), (0.0005, 0.0017), (0, 0.002)], seg=5),
    0.9, 'z', 'y'), f_cyl(0.0008, 0.0008, 0.004)), smooth=60)

# a run of wax: thin where it leaves the lip, a blob at the bottom
part('BrassSconces__wax', f_cyl(0.001, 0.002, 0.014), lambda: lathe(
    [(0, -0.007), (0.0014, -0.0067), (0.0020, -0.0052), (0.0016, -0.0034), (0.0011, -0.0010),
     (0.0010, 0.0050), (0.0007, 0.0068), (0, 0.007)], seg=6), smooth=70)

# chain link
part('BrassSconces__link', f_torus(0.004, 0.001), lambda: ring_section(
    0.004, circ_sec(0.001, 5), seg=10), smooth=60)


# ================================================== ornate wall mirrors ==
part('OrnateWallMirrors__bar', f_box(0.32, 0.020, 0.020),
     lambda: frame_bar(0.32, 0.020, 0.020, back=0.30, edge=0.70, bead=0.18), smooth=40)


def rosette():
    # carved corner rosette facing +Z: eight petals round a domed centre
    s = lathe([(0, -0.020), (0.0120, -0.0180), (0.0185, -0.0110), (0.0200, -0.0030), (0.0180, 0.0060),
               (0.0120, 0.0100), (0.0090, 0.0140), (0.0060, 0.0185), (0, 0.020)], seg=16)
    s = knurl(s, 16, 0.0030, -0.0035, 0.0065)
    return fitf(s.rot(x=math.pi / 2), f_sphere(0.020))

part('OrnateWallMirrors__ornament', f_sphere(0.020), rosette, smooth=45)


def urn_finial():
    # crest: a gadrooned urn on a plinth, collar, and a flame on top
    s = lathe([(0, -0.0225), (0.0200, -0.0225), (0.0200, -0.0190), (0.0140, -0.0170), (0.0160, -0.0110),
               (0.0145, -0.0040), (0.0070, 0.0010), (0.0090, 0.0040), (0.0065, 0.0070), (0.0085, 0.0120),
               (0.0035, 0.0195), (0, 0.0225)], seg=12)
    return knurl(s, 12, 0.0020, -0.0115, -0.0035)

part('OrnateWallMirrors__crown', f_cone(0.020, 0.045), urn_finial, smooth=40)


# =================================================== small pocket watch ==
# case lying face up: domed back, rounded band, bezel round the dial
part('SmallPocketWatches__case', f_cyl(0.028, 0.028, 0.010), lambda: lathe(
    [(0, -0.005), (0.0200, -0.0045), (0.0265, -0.0028), (0.0280, 0.0000), (0.0272, 0.0032),
     (0.0275, 0.0045), (0.0262, 0.0050), (0.0238, 0.0044), (0, 0.0044)], seg=16), smooth=40)

# winding crown: fluted knob (-Y, outwards) on the pendant neck (+Y)
part('SmallPocketWatches__crown', f_cyl(0.004, 0.004, 0.006), lambda: knurl(lathe(
    [(0, -0.003), (0.0040, -0.0028), (0.0040, 0.0000), (0.0032, 0.0008), (0.0020, 0.0012),
     (0.0020, 0.003), (0, 0.003)], seg=10), 10, 0.0006, -0.0029, 0.0001), smooth=40)

# chain link, slightly flattened curb section
part('SmallPocketWatches__link', f_torus(0.005, 0.001), lambda: ring_section(
    0.005, circ_sec(0.001, 5, sx=1.15, sy=0.85), seg=10), smooth=60)


# ======================================================== open journals ==
def journal_cover():
    # the open leather cover: two boards with eased edges meeting at a
    # hinge groove, the rounded spine bulging underneath
    w, d = 0.1035, 0.148
    s = Shape()
    for sx in (-1, 1):
        s = s + rbox(w, 0.0034, d, r=0.0011, seg=2).move(x=sx * (w / 2 + 0.0005), y=0.0008)
    spine = x_axis(lathe([(0, -0.5), (1, -0.5), (1, 0.5), (0, 0.5)], seg=10)).rot(y=math.pi / 2)
    spine = spine.scale(0.009, 0.004, d * 0.99).move(y=-0.0005)
    return fitf(s + spine, f_box(0.208, 0.005, 0.148))

part('OpenJournals__cover', f_box(0.208, 0.005, 0.148), journal_cover)

# quill shaft: the nib point at -Y, the rachis thinning towards +Y
part('OpenJournals__quill', f_cyl(0.001, 0.001, 0.16), lambda: lathe(
    [(0, -0.080), (0.0004, -0.0785), (0.0009, -0.0720), (0.0010, -0.0600), (0.0009, 0.0400),
     (0.0006, 0.0780), (0, 0.080)], seg=5), smooth=60)


# ===================================================== parchment scrolls ==
def rolled_parchment():
    # a sheet really rolled up: the spiral shows at both ends, the core is
    # hollow and the outer edge of the sheet steps along the roll
    turns, r0, r1, t = 3.2, 0.0068, 0.0192, 0.0010
    N = int(turns * 12)
    inner, outer = [], []
    for i in range(N + 1):
        th = TAU * turns * i / N
        r = r0 + (r1 - r0) * i / N
        outer.append(((r + t / 2) * math.cos(th), (r + t / 2) * math.sin(th)))
        inner.append(((r - t / 2) * math.cos(th), (r - t / 2) * math.sin(th)))
    s = prism(outer + list(reversed(inner)), 0.22).rot(x=math.pi / 2)
    return fitf(s, f_cyl(0.020, 0.020, 0.22))

part('ParchmentScrolls__roll', f_cyl(0.020, 0.020, 0.22), rolled_parchment, smooth=40)
