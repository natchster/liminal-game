"""Batch b13: bathroom and dressing-table clutter.

Compacts, perfumes, nail polish, hairbrushes, pill bottles, spray bottles,
wall fixtures (dispensers, towel bars, mirrors, faucets, shelves), and the
small cosmetic things that end up on the floor (lipsticks, mascaras, clippers,
cotton, hair ties). Glass bodies and bulbs are transparent in the game and
stay primitives; their caps, collars, pumps and nozzles are modelled here.
"""
from propkit import *
import propkit


# ------------------------------------------------------------ local kit --
def _fw(s):
    return propkit._fix_winding(s)


def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


def knurl(s, seg, depth, y0, y1, odd=True):
    """Ribs on a lathe body built with `seg` segments: every other vertex
    ring between y0 and y1 is pulled in by `depth` (grip ribs, flutes)."""
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


def ring_section(R, sec, seg=32, arc=TAU, a0=0.0, rmod=None):
    """Sweep a closed 2D section [(dr, dz), ...] around the Z axis in the XY
    plane (a torus_geo! with any cross-section). arc < TAU caps the ends.
    rmod(angle, j) -> (k_r, dz_add) wobbles the section along the ring."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        m_ = rmod(a, i) if rmod else (1.0, 0.0)
        kr, dzz = m_[0], m_[1]
        dR = m_[2] if len(m_) > 2 else 0.0
        for j, (dr, dz) in enumerate(sec):
            v.append(((R + dR + dr * kr) * ca, (R + dR + dr * kr) * sa, dz * kr + dzz))
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


def sweep(points, sec, scale=None, smooth=0, up=(0, 0, 1), caps=True):
    """Sweep a closed 2D section along a 3D polyline (parallel-transport
    frame). scale(t) -> k or (kx, ky)."""
    pts = [Vector(p) for p in points]
    for _ in range(smooth):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    n = len(pts)
    tan = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tan.append(t.normalized())
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
    return _fw(Shape(v, f))


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


def sellipse_pts(a, b, n=20, p=2.5):
    """Superellipse (x, z) outline: p=2 ellipse, larger = squarer."""
    out = []
    for i in range(n):
        t = TAU * i / n
        c, s = math.cos(t), math.sin(t)
        out.append((a * math.copysign(abs(c) ** (2 / p), c), b * math.copysign(abs(s) ** (2 / p), s)))
    return out


def yloft(profile, w, d, rc, cseg=3):
    """Stack of rounded rectangles along +Y: profile = [(y, inset), ...]."""
    rings = []
    for y, ins in profile:
        pts = rrect_pts(w - 2 * ins, d - 2 * ins, max(0.0003, rc - ins * 0.6), cseg)
        rings.append([(x, y, z) for x, z in pts])
    return loft(rings)


def oval_loft(profile, a, b, n=20, p=2.5):
    """Stack of superellipses along +Y: profile = [(y, inset), ...]."""
    rings = []
    for y, ins in profile:
        rings.append([(x, y, z) for x, z in sellipse_pts(a - ins, b - ins, n, p)])
    return loft(rings)


def torus_arc_bounds(R, r, arc, radial=4):
    """Bounds of torus_geo!(R, r, radial, .., arc) (XY plane)."""
    xs, ys = [], []
    for i in range(721):
        u = arc * i / 720
        for j in range(radial):
            w = TAU * j / radial
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r))


def x_axis(s):
    """Lathe/Y-axis shape laid along +X (its +Y end goes to +X)."""
    return s.rot(z=-math.pi / 2)


# ======================================================= small compacts ==
R_SC = 0.038

# powder pan: rounded foot, bead rim, shallow well under the powder disc
part('SmallCompacts__base', f_cyl(R_SC, R_SC, 0.010), lambda: lathe(
    [(0, -0.005), (0.0362, -0.005), (0.038, -0.0034), (0.038, 0.0026), (0.0369, 0.005),
     (0.0333, 0.005), (0.0326, 0.0036), (0, 0.0036)], seg=20))

# open lid lying beside it, inside face up: bezel ring round the mirror seat
part('SmallCompacts__lid', f_cyl(R_SC, R_SC, 0.008), lambda: lathe(
    [(0, -0.004), (0.0362, -0.004), (0.038, -0.0024), (0.038, 0.0022), (0.0369, 0.004),
     (0.0334, 0.004), (0.0327, 0.003), (0, 0.003)], seg=20))

# closed clamshell: two halves meeting at a V seam, domed lid, rounded foot
part('SmallCompacts__body', f_cyl(R_SC, R_SC, 0.018), lambda: lathe(
    [(0, -0.009), (0.0355, -0.009), (0.038, -0.0068), (0.038, -0.0009), (0.0369, 0.0),
     (0.038, 0.0009), (0.038, 0.0046), (0.0352, 0.0078), (0.022, 0.009), (0, 0.009)], seg=16))


def compact_hinge():
    # two hinge knuckles on a pin, axis along X
    k = lathe([(0, -0.003), (0.0021, -0.003), (0.003, -0.0021), (0.0024, 0.0), (0.003, 0.0021),
               (0.0021, 0.003), (0, 0.003)], seg=8)
    return x_axis(k)

part('SmallCompacts__hinge', f_sphere(0.003), compact_hinge, smooth=50)


# ======================================================= small perfumes ==
# atomizer collar: octagonal faceted gold ferrule, shoulder and pump button
part('SmallPerfumes__cap', f_cyl(0.009, 0.009, 0.010), lambda: lathe(
    [(0, -0.005), (0.009, -0.005), (0.009, 0.0012), (0.0072, 0.0029), (0.0040, 0.0031),
     (0.0040, 0.005), (0, 0.005)], seg=8), smooth=30)

# tassel cord: thin twisted strand with a knot near the top, frayed end
part('SmallPerfumes__cord', f_cyl(0.001, 0.001, 0.014), lambda: lathe(
    [(0, -0.007), (0.0009, -0.0066), (0.0006, -0.0056), (0.00055, 0.004), (0.001, 0.0053),
     (0.0006, 0.0066), (0, 0.007)], seg=5), smooth=70)


# =================================================== small nail polishes ==
def polish_brush_cap():
    # tall screw cap: 8 grip ribs, slight taper, domed top
    s = lathe([(0, -0.012), (0.0097, -0.012), (0.010, -0.011), (0.0096, 0.009), (0.0091, 0.0112),
               (0.0078, 0.012), (0, 0.012)], seg=16)
    return knurl(s, 16, 0.0015, -0.0112, 0.0092)

part('SmallNailPolishes__cap', f_cyl(0.010, 0.010, 0.024), polish_brush_cap, smooth=40)


# ===================================================== small hairbrushes ==
def cushion_pad():
    # oval brush head: moulded shell with a rolled edge, a rubber cushion
    # bulging slightly out of the top
    prof = [(-0.006, 0.0035), (-0.0045, 0.0005), (-0.0025, 0.0), (0.002, 0.0), (0.0038, 0.0012),
            (0.0048, 0.0024), (0.006, 0.0042)]
    return oval_loft(prof, 0.0275, 0.0175, n=20, p=2.3)

part('SmallHairbrushes__pad', f_box(0.055, 0.012, 0.035), cushion_pad, smooth=50)

# nylon pin bristle with a ball tip
part('SmallHairbrushes__bristle', f_cyl(0.002, 0.002, 0.010), lambda: lathe(
    [(0, -0.005), (0.0011, -0.005), (0.0009, 0.0018), (0.0017, 0.0024), (0.002, 0.0036),
     (0.0015, 0.0047), (0, 0.005)], seg=6), smooth=70)

# handle: flared where it meets the head, a waist, grip rings, round end
part('SmallHairbrushes__handle', f_cyl(0.008, 0.012, 0.10), lambda: lathe(
    [(0, -0.05), (0.0106, -0.05), (0.012, -0.0472), (0.0112, -0.036), (0.0098, -0.018),
     (0.0093, 0.002), (0.0087, 0.004), (0.0092, 0.007), (0.0089, 0.034), (0.0080, 0.0458),
     (0.0058, 0.0492), (0, 0.05)], seg=12), smooth=50)


# ================================================== spilled pill bottles ==
def pill_cap():
    # child-resistant cap: ribbed skirt, flat top with an eased edge,
    # hollow underside (it lies open-end towards the bottle)
    s = lathe([(0, 0.0015), (0.0136, 0.0015), (0.0136, -0.005), (0.0153, -0.005),
               (0.016, -0.0042), (0.016, 0.0036), (0.0152, 0.005), (0, 0.005)], seg=16)
    return knurl(s, 16, 0.0007, -0.0043, 0.0037)

part('SpilledPillBottles__cap', f_cyl(0.016, 0.016, 0.010), pill_cap, smooth=40)

# caplet pill: round-ended
part('SpilledPillBottles__pill', f_cyl(0.003, 0.003, 0.008), lambda: lathe(
    [(0, -0.004), (0.0019, -0.0035), (0.0029, -0.0019), (0.003, 0.0), (0.0029, 0.0019),
     (0.0019, 0.0035), (0, 0.004)], seg=8), smooth=70)


# ========================================================= spray bottles ==
def trigger_head():
    # trigger-sprayer shroud in side view: screw collar at the bottom, rounded
    # back, snout forward (+X) where the nozzle plugs in
    out = [(-0.0118, -0.015), (0.0118, -0.015), (0.0118, -0.0088), (0.0175, -0.0025),
           (0.0175, 0.0085), (0.0122, 0.0118), (0.0, 0.015), (-0.0115, 0.015),
           (-0.0175, 0.0105), (-0.0175, -0.0035), (-0.0118, -0.0088)]
    s = prism(round_outline(out, 0.0022, seg=2), 0.020)
    # the shroud narrows above the collar

    def w(p):
        t = max(0.0, min(1.0, (p.y + 0.008) / 0.023))
        p.z *= 1.0 - 0.22 * t
        return p
    s = s.warp(w)
    return bevel(s, 0.0012, seg=1, angle=40)

part('SprayBottles__head', f_box(0.035, 0.030, 0.020), trigger_head, smooth=40)

# finger trigger: flat lever with two finger scallops underneath
part('SprayBottles__lever', f_box(0.024, 0.006, 0.010), lambda: prism(round_outline(
    [(-0.012, -0.0008), (-0.0085, -0.003), (-0.0015, -0.0016), (0.0045, -0.003), (0.012, -0.0012),
     (0.012, 0.003), (-0.012, 0.003)], 0.0011, seg=2), 0.010), smooth=40)


def spray_nozzle():
    # round stem into a square adjustable nozzle cap at the outer end (-Y)
    stem = lathe([(0, -0.003), (0.0018, -0.003), (0.0018, 0.0065), (0.0015, 0.007), (0, 0.007)], seg=6)
    head = rbox(0.006, 0.005, 0.006, r=0.0009, seg=1).move(y=-0.0045)
    return stem + head

part('SprayBottles__nozzle', f_cyl(0.003, 0.003, 0.014), spray_nozzle)


def ring_lathe(profile, seg=24):
    """Revolve a *closed* (r, y) loop (no point on the axis): bands, sleeves."""
    s = lathe(list(profile) + [profile[0]], seg, cap_bottom=False, cap_top=False)
    return _fw(weld(s))


# ======================================================= soap dispensers ==
# pump spout under the unit: collar, waisted stem, flared tip with outlet hole
part('SoapDispensers__nozzle', f_cyl(0.010, 0.012, 0.020), lambda: lathe(
    [(0, -0.0072), (0.0034, -0.0072), (0.0034, -0.01), (0.0112, -0.01), (0.012, -0.0091),
     (0.012, -0.0062), (0.0088, -0.0032), (0.0082, 0.0062), (0.010, 0.0078), (0.010, 0.01), (0, 0.01)],
    seg=12), smooth=40)

# push pad on top: eased edges and a shallow finger dish
part('SoapDispensers__plunger', f_box(0.08, 0.014, 0.06), lambda: yloft(
    [(-0.007, 0.0012), (-0.0056, 0.0), (0.0038, 0.0), (0.0062, 0.0016), (0.007, 0.0042), (0.0060, 0.0098)],
    0.08, 0.06, 0.012, cseg=3), smooth=40)


# ======================================================== air fresheners ==
# aerosol can: domed bottom inside a chime, straight wall, rolled shoulder
part('AirFresheners__can', f_cyl(0.020, 0.020, 0.14), lambda: lathe(
    [(0, -0.0655), (0.011, -0.0668), (0.0168, -0.0692), (0.0186, -0.07), (0.0198, -0.0691),
     (0.020, -0.0672), (0.020, 0.0632), (0.0192, 0.0662), (0.0176, 0.0676), (0.0183, 0.0688),
     (0.0176, 0.07), (0.0125, 0.07), (0, 0.0697)], seg=20), smooth=40)

# spray cap: tapered, recessed finger well with the actuator button
part('AirFresheners__cap', f_cyl(0.018, 0.020, 0.018), lambda: lathe(
    [(0, -0.009), (0.0197, -0.009), (0.020, -0.0081), (0.0182, 0.0074), (0.0168, 0.009),
     (0.0092, 0.009), (0.0084, 0.0062), (0.0058, 0.0062), (0.0054, 0.0084), (0, 0.0084)], seg=16), smooth=40)

# label: a thin paper sleeve round the can
part('AirFresheners__label', f_cyl(0.0205, 0.0205, 0.030), lambda: ring_lathe(
    [(0.0199, -0.015), (0.0205, -0.0147), (0.0205, 0.0147), (0.0199, 0.015)], seg=24))


# ======================================================= hand towel bars ==
part('HandTowelBars__bar', f_cyl(0.006, 0.006, 0.36), lambda: lathe(
    [(0, -0.18), (0.0042, -0.18), (0.0056, -0.1791), (0.006, -0.1774), (0.006, 0.1774),
     (0.0056, 0.1791), (0.0042, 0.18), (0, 0.18)], seg=12))


def towel_bar_post():
    # side view: wall flange at the back, a neck, and a round collar that
    # the bar runs through; extruded across the bar's width
    cz, r = 0.0035, 0.0065
    arc = [(cz + r * math.cos(a), r * math.sin(a))
           for a in (math.radians(d) for d in range(-150, 151, 30))]
    out = [(-0.010, -0.007), (-0.0076, -0.007), (-0.0076, -0.0031), (-0.0042, -0.0031)] + arc + \
          [(-0.0042, 0.0031), (-0.0076, 0.0031), (-0.0076, 0.007), (-0.010, 0.007)]
    return prism(out, 0.010).rot(y=-math.pi / 2)

part('HandTowelBars__bracket', f_box(0.010, 0.014, 0.020), towel_bar_post, smooth=40)


# ===================================================== bathroom mirrors ==
def crown_bar(w, h, t, back=0.55):
    """A frame bar whose face crowns and rolls off towards all four edges,
    so it reads as a rounded moulding when stretched either way (one
    constructor makes both the long and the tall bars)."""
    us = [-1, -0.975, -0.92, -0.8, -0.55, -0.2, 0.2, 0.55, 0.8, 0.92, 0.975, 1]
    n = len(us)
    f_ = lambda u: max(0.0, 1 - abs(u) ** 5) ** 0.35
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

part('BathroomMirrors__bar', f_box(0.328, 0.014, 0.012), lambda: crown_bar(0.328, 0.014, 0.012), smooth=40)


# ========================================================== sink faucets ==
def faucet_plate():
    # stadium escutcheon with a chamfered face and two fixing screws
    s = yloft([(-0.004, 0.0), (0.0008, 0.0), (0.0031, 0.0013), (0.004, 0.0036)], 0.16, 0.06, 0.0299, cseg=4)
    s.rot(x=math.pi / 2)
    for sx in (-1, 1):
        s = s + screw_head(0.0028, 0.0011).rot(x=math.pi / 2).move(x=sx * 0.0712, z=0.0034)
    return s

part('SinkFaucets__plate', f_box(0.16, 0.06, 0.008), faucet_plate)

# spout riser: foot collar, waisted body, ring under the spout joint
part('SinkFaucets__base', f_cyl(0.012, 0.012, 0.05), lambda: lathe(
    [(0, -0.025), (0.0106, -0.025), (0.012, -0.0236), (0.012, -0.0192), (0.0092, -0.0158),
     (0.0092, 0.0168), (0.0118, 0.0192), (0.0118, 0.025), (0, 0.025)], seg=14))

F_SPOUT = f_bounds((0.0, 0.0, -0.010), (0.070, 0.070, 0.010))


def faucet_spout():
    # quarter-bend spout tube (r 8.5 mm) with a joint collar at the riser end
    # (theta = 90 deg) and an aerator collar at the outlet (theta = 0)
    angs = [0.0, 0.05, 0.075, 0.18, 0.36, 0.56, 0.78, 1.0, 1.2, 1.38, 1.495, 1.52, math.pi / 2]
    ks = [1.0, 1.0, 0.85, 0.85, 0.85, 0.85, 0.85, 0.85, 0.85, 0.85, 0.85, 1.0, 1.0]
    pts = [(0.06 * math.cos(a), 0.06 * math.sin(a), 0.0) for a in angs]
    n = len(pts)
    s = sweep(pts, circ_sec(0.010, 8), scale=lambda t: ks[int(round(t * (n - 1)))])
    return fitf(s, F_SPOUT)

part('SinkFaucets__spout', F_SPOUT, faucet_spout, smooth=50)


def faucet_knob():
    # handle hub: fluted skirt against the wall (-Y), domed top
    s = lathe([(0, -0.004), (0.0136, -0.004), (0.014, -0.0033), (0.0138, -0.0012), (0.0121, 0.0012),
               (0.0096, 0.0032), (0.0084, 0.004), (0, 0.004)], seg=16)
    return knurl(s, 16, 0.0012, -0.0034, -0.0011)

part('SinkFaucets__knob', f_cyl(0.014, 0.014, 0.008), faucet_knob, smooth=40)


def cross_arm():
    # one arm of the cross handle, plan view: waisted bar with round ends
    out = [(-0.010, 0.0), (-0.0093, -0.0015), (-0.0077, -0.0015), (-0.0058, -0.001), (0.0058, -0.001),
           (0.0077, -0.0015), (0.0093, -0.0015), (0.010, 0.0), (0.0093, 0.0015), (0.0077, 0.0015),
           (0.0058, 0.001), (-0.0058, 0.001), (-0.0077, 0.0015), (-0.0093, 0.0015)]
    return prism(out, 0.002).rot(x=math.pi / 2)

part('SinkFaucets__cross1', f_box(0.020, 0.002, 0.003), cross_arm, smooth=50)
part('SinkFaucets__cross2', f_box(0.003, 0.002, 0.020), lambda: cross_arm().rot(y=math.pi / 2), smooth=50)


# ============================================================ soap bars ==
# worn pillow bar: rounded sides and a domed top
part('SoapBars__bar', f_box(0.07, 0.020, 0.04), lambda: oval_loft(
    [(-0.010, 0.0042), (-0.0084, 0.0008), (-0.0058, 0.0), (0.0042, 0.0), (0.0077, 0.0018),
     (0.0095, 0.0048), (0.010, 0.0105)], 0.035, 0.020, n=20, p=3.2), smooth=60)


def soap_dish():
    # moulded dish: foot, rolled rim, sloped inner wall, floor with drain ribs
    rings = []
    for y, ins in ((-0.004, 0.0055), (-0.0028, 0.0012), (0.0024, 0.0), (0.004, 0.0013),
                   (0.004, 0.0046), (-0.0012, 0.0068)):
        pts = rrect_pts(0.09 - 2 * ins, 0.06 - 2 * ins, max(0.003, 0.012 - ins * 0.6), 3)
        rings.append([(x, y, z) for x, z in pts])
    s = loft(rings)
    for i in range(4):
        s = s + box(0.0026, 0.003, 0.042).move(x=-0.0255 + i * 0.017, y=-0.0008)
    return s

part('SoapBars__dish', f_box(0.09, 0.008, 0.06), soap_dish, smooth=40)


# ============================================================= bath mats ==
def bath_mat():
    # plush mat: rounded corners, bound edge with a stitched groove inside
    # it, soft pile field with a slight nap
    W, D = 0.60, 0.40
    prof = [(-0.004, 0.004), (-0.0031, 0.0006), (-0.0008, 0.0), (0.0022, 0.0006), (0.0031, 0.0028),
            (0.0026, 0.011), (0.0028, 0.014), (0.0036, 0.021), (0.0039, 0.05), (0.0040, 0.10),
            (0.0040, 0.16), (0.0040, 0.19)]
    rings = []
    for y, ins in prof:
        pts = rrect_pts(W - 2 * ins, D - 2 * ins, max(0.006, 0.05 - ins * 0.8), 5)
        rings.append([(x, y, z) for x, z in pts])
    s = loft(rings)
    def nap(p):
        if p.y > 0.0015:
            p.y += 0.0006 * noise.noise(Vector((p.x * 40, 0.3, p.z * 40)))
        return p
    return s.warp(nap)

part('BathMats__mat', f_box(0.60, 0.008, 0.40), bath_mat, smooth=60)


# =========================================================== hair dryers ==
def dryer_plate():
    # wall holder plate: rounded, chamfered face, two fixing screws
    s = yloft([(-0.010, 0.0), (0.0058, 0.0), (0.0091, 0.0024), (0.010, 0.0064)], 0.10, 0.14, 0.014, cseg=3)
    s.rot(x=math.pi / 2)
    for sy in (-1, 1):
        s = s + screw_head(0.0042, 0.0016).rot(x=math.pi / 2).move(y=sy * 0.055, z=0.0092)
    return s

part('HairDryers__plate', f_box(0.10, 0.14, 0.02), dryer_plate)

# barrel: front (-Y) necks down to the nozzle, seam ring, rear (+Y) intake
# cap with two concentric grille grooves
part('HairDryers__body', f_cyl(0.045, 0.045, 0.16), lambda: lathe(
    [(0, -0.08), (0.031, -0.08), (0.0345, -0.0772), (0.0405, -0.067), (0.0443, -0.053), (0.045, -0.04),
     (0.045, 0.050), (0.0437, 0.0526), (0.045, 0.0552), (0.045, 0.0668), (0.0424, 0.0756), (0.0372, 0.08),
     (0.0335, 0.0782), (0.0275, 0.0782), (0.0235, 0.08), (0.019, 0.08), (0.0165, 0.0782), (0, 0.0782)], seg=16), smooth=40)


def dryer_nozzle():
    # flared concentrator: lip at the outlet (-Y), recessed outlet, 2 vanes
    s = lathe([(0, -0.0165), (0.0288, -0.0165), (0.0302, -0.02), (0.0366, -0.02), (0.038, -0.0187),
               (0.0376, -0.0148), (0.0334, -0.002), (0.0309, 0.0125), (0.030, 0.0185), (0.0282, 0.02),
               (0, 0.02)], seg=14)
    for dz in (-0.0095, 0.0095):
        s = s + box(0.058, 0.004, 0.0024).move(y=-0.0178, z=dz)
    return s

part('HairDryers__nozzle', f_cyl(0.030, 0.038, 0.04), dryer_nozzle, smooth=40)

# grip: finger grooves, rounded butt
part('HairDryers__handle', f_cyl(0.025, 0.025, 0.10), lambda: lathe(
    [(0, -0.05), (0.0175, -0.05), (0.0228, -0.0472), (0.025, -0.0418), (0.025, -0.034), (0.0226, -0.0285),
     (0.025, -0.0225), (0.0226, -0.017), (0.025, -0.0112), (0.0236, 0.0), (0.0232, 0.03), (0.0236, 0.05),
     (0, 0.05)], seg=14), smooth=45)

F_DRYER_CORD = torus_arc_bounds(0.025, 0.005, math.pi, radial=4)


def dryer_cord():
    # rubber flex loop with a strain-relief sleeve at each end
    angs = [0.0, 0.1, 0.13, 0.3, 0.55, 0.85, 1.2, 1.5708, 1.94, 2.29, 2.59, 2.84, 3.01, 3.04, math.pi]
    ks = [1.0, 1.0, 0.7] + [0.62] * 9 + [0.7, 1.0, 1.0]
    pts = [(0.025 * math.cos(a), 0.025 * math.sin(a), 0.0) for a in angs]
    n = len(pts)
    s = sweep(pts, circ_sec(0.005, 8), scale=lambda t: ks[int(round(t * (n - 1)))])
    return fitf(s, F_DRYER_CORD)

part('HairDryers__cord', F_DRYER_CORD, dryer_cord, smooth=50)


# =============================================== toilet paper dispensers ==
def tp_housing():
    # moulded jumbo-roll housing: flat back, pillowed cover with a parting
    # seam, a sight-window slot and a key lock at the top
    s = yloft([(-0.07, 0.0), (0.026, 0.0), (0.028, 0.0016), (0.030, 0.0), (0.046, 0.0), (0.058, 0.0032),
               (0.0655, 0.0092), (0.07, 0.019)], 0.24, 0.20, 0.035, cseg=3)
    s.rot(x=math.pi / 2)
    s = cut(s, rbox(0.11, 0.013, 0.012, r=0.004, seg=1).move(y=-0.015, z=0.0705))
    lock = lathe([(0, -0.003), (0.0085, -0.003), (0.0085, 0.0008), (0.0072, 0.0025), (0, 0.0025)], seg=12)
    return s + lock.rot(x=math.pi / 2).move(y=0.058, z=0.0675)

part('ToiletPaperDispensers__housing', f_box(0.24, 0.20, 0.14), tp_housing)

# paper roll: soft rounded edges, open centre around the core
part('ToiletPaperDispensers__roll', f_cyl(0.055, 0.055, 0.10), lambda: ring_lathe(
    [(0.019, -0.05), (0.051, -0.05), (0.0538, -0.0492), (0.055, -0.0465), (0.055, 0.0465),
     (0.0538, 0.0492), (0.051, 0.05), (0.019, 0.05)], seg=24), smooth=45)

# cardboard core tube
part('ToiletPaperDispensers__core', f_cyl(0.015, 0.015, 0.11), lambda: ring_lathe(
    [(0.0134, -0.055), (0.015, -0.055), (0.015, 0.055), (0.0134, 0.055)], seg=12), smooth=50)


def tear_bar():
    # serrated tear bar: side view has a straight top and a toothed edge
    n = 14
    L = 0.14
    out = [(-L / 2, 0.005)]
    for i in range(n):
        z0 = -L / 2 + L * i / n
        out += [(z0 + L / n * 0.1, -0.0018), (z0 + L / n * 0.5, -0.005)]
    out += [(L / 2, -0.0018), (L / 2, 0.005)]
    return prism(out[::-1], 0.020).rot(y=-math.pi / 2)

part('ToiletPaperDispensers__lip', f_box(0.020, 0.010, 0.14), tear_bar)


# ====================================================== shampoo shelves ==
def shampoo_shelf():
    # side view: bullnose front (+Z), undercut cove, square back to the wall
    arc = [(0.061 + 0.009 * math.cos(math.radians(a)), 0.001 + 0.009 * math.sin(math.radians(a)))
           for a in range(-60, 91, 30)]
    out = [(-0.07, -0.01), (0.052, -0.01), (0.057, -0.0082)] + arc + [(0.061, 0.01), (-0.07, 0.01)]
    s = prism(out, 0.50).rot(y=-math.pi / 2)
    return bevel(s, 0.0015, seg=1, angle=50)

part('ShampooShelves__shelf', f_box(0.50, 0.020, 0.14), shampoo_shelf)

# shampoo bottle: rounded foot, recessed label panel, shoulder into the neck
part('ShampooShelves__bottle', f_cyl(0.030, 0.032, 0.17), lambda: lathe(
    [(0, -0.085), (0.0288, -0.085), (0.0317, -0.0826), (0.032, -0.0778), (0.0315, -0.0552),
     (0.0308, -0.0535), (0.0302, 0.0292), (0.0309, 0.0312), (0.0302, 0.0495), (0.0282, 0.0618),
     (0.0236, 0.0706), (0.0205, 0.0742), (0.0205, 0.085), (0, 0.085)], seg=16), smooth=45)


def flip_cap():
    # flip-top cap: lid parting line, domed lid, thumb notch at the front
    s = lathe([(0, -0.01), (0.0195, -0.01), (0.020, -0.0092), (0.020, 0.0028), (0.0187, 0.0041),
               (0.020, 0.0054), (0.0194, 0.0082), (0.0168, 0.0099), (0, 0.01)], seg=14)
    return cut(s, box(0.011, 0.0046, 0.006).move(y=0.0036, z=0.0205))

part('ShampooShelves__cap', f_cyl(0.020, 0.020, 0.020), flip_cap, smooth=40)


# ============================================================ towel bars ==
def towel_bracket():
    # wall rosette at the back, tapered post, round boss holding the bar
    ros = yloft([(-0.003, 0.0), (0.0008, 0.0), (0.003, 0.0022)], 0.03, 0.06, 0.013, cseg=3)
    ros.rot(x=math.pi / 2).move(z=-0.027)
    post = lathe([(0.0105, -0.024), (0.0088, 0.004), (0.0092, 0.008)], seg=12, cap_bottom=False, cap_top=False)
    post.rot(x=math.pi / 2)
    boss = sphere(0.013, seg=12, rings=6).move(z=0.017)
    return ros + post + boss

part('TowelBars__bracket', f_box(0.03, 0.06, 0.06), towel_bracket, smooth=45)

part('TowelBars__bar', f_cyl(0.010, 0.010, 0.48), lambda: lathe(
    [(0, -0.24), (0.007, -0.24), (0.0093, -0.2386), (0.010, -0.2362), (0.010, 0.2362),
     (0.0093, 0.2386), (0.007, 0.24), (0, 0.24)], seg=14))


# =========================================================== hair brushes ==
# paddle head: rounded rim, flat face for the cushion
part('HairBrushes__pad', f_box(0.06, 0.010, 0.11), lambda: oval_loft(
    [(-0.005, 0.0032), (-0.0038, 0.0004), (-0.002, 0.0), (0.0018, 0.0), (0.0038, 0.0009), (0.005, 0.003)],
    0.030, 0.055, n=24, p=3.0), smooth=50)


def paddle_bristles():
    # rubber cushion with rows of tapered bristle tufts
    base = oval_loft([(-0.006, 0.0), (-0.0035, 0.0)], 0.0275, 0.05, n=16, p=3.0)
    tuft = lathe([(0.0024, -0.0036), (0.0011, 0.0058), (0, 0.006)], seg=3, cap_bottom=False)
    s = base
    for ix in range(4):
        for iz in range(6):
            s = s + tuft.copy().rot(y=0.5 * (ix + iz)).move(x=-0.0207 + ix * 0.0138, z=-0.0395 + iz * 0.0158)
    return s

part('HairBrushes__bristles', f_box(0.055, 0.012, 0.10), paddle_bristles, flat=True)

# handle: neck at the head (-Y), swelling to a rounded butt (+Y)
part('HairBrushes__handle', f_cyl(0.014, 0.012, 0.10), lambda: lathe(
    [(0, -0.05), (0.0106, -0.05), (0.012, -0.0482), (0.0106, -0.04), (0.0099, -0.022), (0.0114, 0.0),
     (0.0131, 0.024), (0.014, 0.038), (0.0134, 0.0458), (0.0108, 0.0492), (0.006, 0.05), (0, 0.05)], seg=12),
    smooth=50)


# =========================================================== stall walls ==
def stall_foot():
    # stainless pedestal shoe: anchored flange, tapered body, top bead
    s = yloft([(-0.075, 0.0), (-0.067, 0.0), (-0.065, 0.0062), (-0.062, 0.0092), (0.060, 0.0132),
               (0.0655, 0.0112), (0.0715, 0.0112), (0.075, 0.0132)], 0.10, 0.08, 0.008, cseg=3)
    bolt = lathe([(0, 0.0), (0.0048, 0.0), (0.0048, 0.0022), (0.0036, 0.0034), (0, 0.0034)], seg=6)
    for sx in (-1, 1):
        for sz in (-1, 1):
            s = s + bolt.copy().move(x=sx * 0.0405, y=-0.0672, z=sz * 0.0305)
    return s

part('StallWalls__foot', f_box(0.10, 0.15, 0.08), stall_foot)


# ==================================================== toothbrush holders ==
def bullnose_shelf(w, h, d):
    """Side view: rounded nosing at the front (+Z), undercut, square back."""
    rn = h * 0.45
    arc = [(d / 2 - rn + rn * math.cos(math.radians(a)), h * 0.05 + rn * math.sin(math.radians(a)))
           for a in range(-60, 91, 30)]
    out = [(-d / 2, -h / 2), (d / 2 - h * 0.6, -h / 2), (d / 2 - h * 0.3, -h * 0.41)] + arc + \
          [(d / 2 - rn, h / 2), (-d / 2, h / 2)]
    s = prism(out, w).rot(y=-math.pi / 2)
    return bevel(s, min(0.0015, h * 0.08), seg=1, angle=50)

part('ToothbrushHolders__shelf', f_box(0.16, 0.015, 0.10), lambda: bullnose_shelf(0.16, 0.015, 0.10))

# tumbler: foot ring, flared wall, rolled rim, real inside and floor
part('ToothbrushHolders__cup', f_cyl(0.035, 0.030, 0.10), lambda: vessel(
    [(0.0262, -0.05), (0.0294, -0.0492), (0.030, -0.0462), (0.0292, -0.0432), (0.0298, -0.040),
     (0.0336, 0.036), (0.0349, 0.0493)], 0.0026, seg=20, floor=0.009), smooth=40)


def toothbrush_handle():
    # contoured handle: soft grip below, thin neck, flat back of the head on top
    rings = []
    for y, w, d in ((-0.055, 0.0052, 0.0046), (-0.0525, 0.0074, 0.0068), (-0.045, 0.008, 0.008),
                    (-0.012, 0.0078, 0.0074), (0.012, 0.0062, 0.0056), (0.030, 0.0042, 0.0044),
                    (0.043, 0.0048, 0.0046), (0.049, 0.0068, 0.0052), (0.0545, 0.0064, 0.0048),
                    (0.055, 0.0046, 0.0034)):
        rings.append([(x, y, z) for x, z in rrect_pts(w, d, min(w, d) * 0.42, 2)])
    return loft(rings)

part('ToothbrushHolders__handle', f_box(0.008, 0.11, 0.008), toothbrush_handle, smooth=50)


def toothbrush_head():
    # bristle block: thin backing, five trimmed tufts rising towards +Z
    out = [(-0.006, -0.010), (0.0035, -0.010)]
    for i in range(5):
        y0 = -0.010 + 0.004 * i
        out += [(0.006, y0 + 0.0006), (0.006, y0 + 0.0034)]
        if i < 4:
            out += [(0.0028, y0 + 0.004)]
    out += [(0.0035, 0.010), (-0.006, 0.010)]
    return prism(out, 0.010).rot(y=-math.pi / 2)

part('ToothbrushHolders__head', f_box(0.010, 0.020, 0.012), toothbrush_head, flat=True)


# ====================================================== single earrings ==
def hoop_loop():
    # hoop with a small opening and the snap post bridging it (gap at 45 deg
    # so the ring still reaches its full bounds on both axes)
    gap = 0.26
    a0 = math.pi / 4 + gap / 2
    s = ring_section(0.010, circ_sec(0.0015, 6), seg=20, arc=TAU - gap, a0=a0)
    p0 = (0.010 * math.cos(a0 - gap - 0.05), 0.010 * math.sin(a0 - gap - 0.05), 0)
    p1 = (0.010 * math.cos(a0 + 0.05), 0.010 * math.sin(a0 + 0.05), 0)
    return s + tube([p0, p1], 0.0006, seg=5)

part('SingleEarrings__loop', f_torus(0.010, 0.0015), hoop_loop, smooth=50)

F_GEM = f_bounds((-0.005, -0.005, -0.005), (0.005, 0.005, 0.005))
# round brilliant: pointed pavilion, girdle, crown and table
part('SingleEarrings__gem', F_GEM, lambda: lathe(
    [(0, -0.005), (0.005, 0.0010), (0.005, 0.0017), (0.0029, 0.005), (0, 0.005)], seg=8), flat=True)


# ======================================================== nail clippers ==
def clip_strip(L, H, W, rise, w_rear, thick=0.0012, n=14):
    """A spring-steel jaw strip in side view: flat at the rear (-X), curving
    by `rise` (up +, down -) into the jaw at the front, widening to W."""
    pts, ks = [], []
    for i in range(n):
        t = i / (n - 1)
        x = -L / 2 + L * t
        c = max(0.0, (t - 0.55) / 0.45)
        y = rise * (c * c * (3 - 2 * c))
        pts.append((x, y, 0.0))
        ks.append(w_rear + (1.0 - w_rear) * min(1.0, t / 0.7))
    sec = rounded_rect(thick, W, thick * 0.45, seg=1)
    s = sweep(pts, sec, scale=lambda t: (1.0, ks[int(round(t * (n - 1)))]))
    return fitf(s, f_box(L, H, W))

part('NailClippers__bottom', f_box(0.055, 0.007, 0.020), lambda: clip_strip(0.055, 0.007, 0.020, 0.0056, 0.68))
part('NailClippers__top', f_box(0.048, 0.005, 0.018), lambda: clip_strip(0.048, 0.005, 0.018, -0.0036, 0.7))


def clip_lever():
    # stamped lever: narrow pivot end with a notch (-X), wide thumb pad (+X)
    # with grip ridges, arched along its length
    top = [(-0.025, 0.0), (-0.0238, 0.0024), (-0.018, 0.0027), (-0.010, 0.0033), (-0.002, 0.0039),
           (0.008, 0.004), (0.018, 0.004), (0.0232, 0.0034), (0.025, 0.0014)]
    out = top + [(x, -z) for x, z in reversed(top[1:-1])]
    out = [(x, z) for x, z in out]
    s = prism(out, 0.0012).rot(x=math.pi / 2)

    def arch(p):
        p.y += 0.0024 * (1 - (p.x / 0.025) ** 2)
        return p
    s = s.warp(arch)
    for gx in (0.010, 0.0145, 0.019):
        s = s + box(0.0012, 0.0008, 0.0062).move(x=gx, y=0.0024 * (1 - (gx / 0.025) ** 2) + 0.0008)
    return fitf(s, f_box(0.050, 0.004, 0.008))

part('NailClippers__lever', f_box(0.050, 0.004, 0.008), clip_lever, smooth=40)

# pivot pin: domed head on top, peened end below
part('NailClippers__rivet', f_cyl(0.003, 0.003, 0.018), lambda: lathe(
    [(0, -0.009), (0.0024, -0.009), (0.0026, -0.0082), (0.0014, -0.0075), (0.0014, 0.0072), (0.003, 0.0078),
     (0, 0.009)], seg=8), smooth=50)


# =========================================================== shower caps ==
# gathered elastic band: the section swells and dips in 18 pleats
part('ShowerCaps__band', f_torus(0.088, 0.005), lambda: ring_section(
    0.088, circ_sec(0.005, 5), seg=54,
    rmod=lambda a, i: (1.0 + 0.18 * math.cos(18 * a), 0.0015 * math.sin(18 * a + 0.8),
                       0.0028 * math.cos(18 * a))), smooth=70)


# ============================================================= hair ties ==
# fabric-covered elastic: slightly crinkled section, a seam bulge
part('HairTies__tie', f_torus(0.020, 0.004), lambda: ring_section(
    0.020, circ_sec(0.004, 6), seg=22,
    rmod=lambda a, i: (1.0 + 0.07 * math.sin(7 * a) + 0.05 * math.sin(13 * a + 1.0)
                       + 0.12 * max(0.0, math.cos(a - 2.0)) ** 16, 0.0)), smooth=70)


# ==================================================== contact lens cases ==
def lens_case_base():
    # twin-well body: a peanut-shaped plinth, a shoulder under each cap and a
    # threaded collar the caps screw onto
    c, r = 0.017, 0.018
    out = [(c + r * math.cos(math.radians(a)), r * math.sin(math.radians(a))) for a in range(-120, 121, 30)]
    out += [(0.0, 0.0138)]
    out += [(-c + r * math.cos(math.radians(a)), r * math.sin(math.radians(a))) for a in range(60, 301, 30)]
    out += [(0.0, -0.0138)]
    s = prism(out, 0.006).rot(x=math.pi / 2).move(y=-0.003)
    well = lathe([(0.0145, -0.0005), (0.0145, 0.0024), (0.0121, 0.0031), (0.0121, 0.006), (0, 0.006)],
                 seg=12, cap_bottom=False)
    return s + well.copy().move(x=-0.016) + well.copy().move(x=0.016)

part('ContactLensCases__base', f_box(0.070, 0.012, 0.036), lens_case_base, smooth=40)


def lens_cap():
    s = lathe([(0, -0.005), (0.0127, -0.005), (0.013, -0.0042), (0.013, 0.0032), (0.012, 0.0048),
               (0.0104, 0.005), (0.0097, 0.0043), (0, 0.0043)], seg=20)
    return knurl(s, 20, 0.0008, -0.0043, 0.0033)

part('ContactLensCases__cap', f_cyl(0.013, 0.013, 0.010), lens_cap, smooth=40)


# ========================================================= cotton swabs ==
part('CottonSwabs__stick', f_cyl(0.0012, 0.0012, 0.040), lambda: lathe(
    [(0, -0.02), (0.0011, -0.0199), (0.0012, -0.0192), (0.0012, 0.0192), (0.0011, 0.0199), (0, 0.02)], seg=6),
    smooth=70)

part('CottonSwabs__tuft', f_sphere(0.0035), lambda: noisy(sphere(0.0035, seg=8, rings=6), 0.00035, 900, seed=3),
     smooth=180)


# ========================================================== cotton balls ==
def cotton_puff():
    s = sphere(0.013, seg=14, rings=9)
    s = noisy(s, 0.0024, 130, seed=5)
    return noisy(s, 0.0008, 420, seed=9)

part('CottonBalls__puff', f_sphere(0.013), cotton_puff, smooth=180)


# ================================================ hand towel dispensers ==
def towel_dispenser_body():
    # moulded housing: flat back, pillowed cover with a parting seam, a
    # paper-level sight slot and a key lock on the top of the cover
    s = yloft([(-0.07, 0.0), (0.034, 0.0), (0.036, 0.0018), (0.038, 0.0), (0.051, 0.0), (0.0615, 0.0042),
               (0.0668, 0.0112), (0.07, 0.0225)], 0.30, 0.40, 0.045, cseg=3)
    s.rot(x=math.pi / 2)
    s = cut(s, rbox(0.014, 0.17, 0.012, r=0.005, seg=1).move(x=0.095, y=-0.01, z=0.0705))
    lock = lathe([(0, -0.003), (0.010, -0.003), (0.010, 0.0008), (0.0086, 0.0026), (0, 0.0026)], seg=12)
    return s + lock.rot(x=math.pi / 2).move(y=0.14, z=0.0674)

part('HandTowelDispensers__body', f_box(0.30, 0.40, 0.14), towel_dispenser_body)


def towel_throat():
    # dispensing lip in side view: a plate curling down at the front (+Z)
    top = [(-0.05, 0.007), (0.018, 0.007), (0.031, 0.0055), (0.041, 0.002), (0.047, -0.0025), (0.05, -0.007)]
    bot = [(0.0465, -0.007), (0.0435, -0.003), (0.0375, 0.0003), (0.029, 0.0028), (0.018, 0.004), (-0.05, 0.004)]
    s = prism(round_outline(top + bot, 0.0012, seg=1), 0.24).rot(y=-math.pi / 2)
    return bevel(s, 0.0015, seg=1, angle=60)

part('HandTowelDispensers__lip', f_box(0.24, 0.014, 0.10), towel_throat, smooth=40)

F_TONGUE = f_bounds((-0.10, -0.07, 0.0), (0.10, 0.07, 0.0), 'plane')


def paper_tongue():
    # hanging paper towel: thin sheet, soft waves, torn bottom edge;
    # thickness and waves go behind the front plane
    s = slab(0.20, 0.14, 0.0006, 12, 8).move(z=-0.0003)
    rnd = [0.0, 0.004, 0.001, 0.006, 0.002, 0.0, 0.005, 0.0015, 0.0045, 0.0, 0.003, 0.001, 0.0]

    def w(p):
        u = (p.x + 0.10) / 0.20
        t = (0.07 - p.y) / 0.14          # 0 at the slot, 1 at the torn edge
        p.z -= 0.0022 * t * t + 0.0009 * (1 + math.sin(u * 9.0 + 0.6)) * t
        if p.y < -0.069:
            p.y += rnd[int(round(u * 12))]
        return p
    return fitf(s.warp(w), (None, (-0.10, -0.07, -0.0037), (0.10, 0.07, 0.0)))

part('HandTowelDispensers__tongue', F_TONGUE, paper_tongue, smooth=60)


def pull_strip():
    # dark "pull" badge: rounded plate with three raised down-chevrons
    s = rbox(0.06, 0.010, 0.003, r=0.001, seg=1).move(z=-0.001)
    chev = [(-0.0045, 0.0032), (-0.003, 0.0032), (0.0, -0.0008), (0.003, 0.0032), (0.0045, 0.0032), (0.0, -0.0032)]
    for cx in (-0.018, 0.0, 0.018):
        s = s + prism(chev, 0.002).move(x=cx, z=0.0015)
    return s

part('HandTowelDispensers__arrow', f_box(0.06, 0.010, 0.005), pull_strip, flat=True)


# ===================================================== floss containers ==
# lower shell: a rounded pebble, flat top where the lid closes
part('FlossContainers__body', f_box(0.045, 0.014, 0.038), lambda: yloft(
    [(-0.007, 0.0042), (-0.0058, 0.0009), (-0.0038, 0.0), (0.0062, 0.0), (0.007, 0.0006)],
    0.045, 0.038, 0.013, cseg=3), smooth=50)


def floss_lid():
    # flip lid: domed, with a thumb dimple at the front (+X) by the cutter
    s = yloft([(-0.002, 0.0), (0.0004, 0.0), (0.0014, 0.0011), (0.002, 0.0032)], 0.042, 0.035, 0.012, cseg=3)
    return cut(s, sphere(0.006, seg=12, rings=6).scale(1.0, 0.5, 1.0).move(x=0.0175, y=0.0035))

part('FlossContainers__lid', f_box(0.042, 0.004, 0.035), floss_lid, smooth=45)

# cutter: small metal tab with a V notch in its outer edge
part('FlossContainers__notch', f_box(0.010, 0.004, 0.006), lambda: prism(
    [(-0.005, -0.003), (0.005, -0.003), (0.005, -0.0011), (0.0022, 0.0), (0.005, 0.0011), (0.005, 0.003),
     (-0.005, 0.003)], 0.004).rot(x=math.pi / 2), flat=True)


# ====================================================== makeup compacts ==
R_MC = 0.028
F_MC = f_cyl(R_MC, R_MC, 0.006)
# powder pan base: rounded foot, bead rim, pan well under the powder disc
part('MakeupCompacts__base', F_MC, lambda: lathe(
    [(0, -0.003), (0.0265, -0.003), (0.028, -0.0018), (0.028, 0.0016), (0.0272, 0.003), (0.0232, 0.003),
     (0.0226, 0.0022), (0, 0.0022)], seg=20))

# open lid standing up: +Y faces the powder (mirror seat and bezel), the
# outside (-Y) is domed
part('MakeupCompacts__lidOpen', F_MC, lambda: lathe(
    [(0, -0.003), (0.019, -0.0027), (0.0258, -0.0019), (0.028, -0.0006), (0.028, 0.0018), (0.0272, 0.003),
     (0.0222, 0.003), (0.0216, 0.0024), (0, 0.0024)], seg=16))

# closed lid: domed top with an engraved ring
part('MakeupCompacts__lid', F_MC, lambda: lathe(
    [(0, -0.003), (0.0268, -0.003), (0.028, -0.0018), (0.028, 0.0006), (0.0266, 0.0022), (0.0215, 0.0028),
     (0.0204, 0.0024), (0.0192, 0.0029), (0, 0.003)], seg=16))


# ============================================================= lipsticks ==
# tube: base (+Y) with a chamfer, seam ring, reduced collar at the open end (-Y)
part('Lipsticks__body', f_cyl(0.012, 0.012, 0.055), lambda: lathe(
    [(0, -0.0275), (0.0108, -0.0275), (0.0112, -0.027), (0.0112, -0.023), (0.012, -0.022), (0.012, 0.017),
     (0.0113, 0.0177), (0.012, 0.0184), (0.012, 0.0268), (0.0113, 0.0275), (0, 0.0275)], seg=14))

# decorative band: flat-faced metal ring
part('Lipsticks__band', f_torus(0.0125, 0.0025), lambda: ring_section(
    0.0125, rounded_rect(0.005, 0.005, 0.0012, seg=1), seg=16), smooth=40)

# twist collar: ribbed grip ring
part('Lipsticks__twist', f_torus(0.012, 0.003), lambda: ring_section(
    0.012, rounded_rect(0.006, 0.006, 0.0018, seg=1)[::2], seg=20,
    rmod=lambda a, i: (1.0 if i % 2 == 0 else 0.84, 0.0)), smooth=40)

# bullet: slim where it leaves the tube (+Y), full and rounded at the tip (-Y)
part('Lipsticks__stick', f_cyl(0.008, 0.010, 0.024), lambda: lathe(
    [(0, -0.012), (0.0074, -0.012), (0.0092, -0.0113), (0.010, -0.0098), (0.0099, -0.005), (0.0086, 0.008),
     (0.008, 0.012), (0, 0.012)], seg=12), smooth=45)

# cap: hollow, open end (+Y) with a grip groove, flat closed end (-Y)
part('Lipsticks__cap', f_cyl(0.014, 0.014, 0.030), lambda: lathe(
    [(0, -0.015), (0.0132, -0.015), (0.014, -0.0138), (0.014, 0.0098), (0.0133, 0.0108), (0.014, 0.0118),
     (0.014, 0.015), (0.0129, 0.015), (0.0129, -0.0139), (0, -0.0139)], seg=14), smooth=40)


# ================================================== eyeshadow palettes ==
def palette_base():
    # tray: eased edges, two hinge knuckles along the back (-Z)
    s = yloft([(-0.003, 0.0012), (-0.0022, 0.0), (0.0018, 0.0), (0.0026, 0.0004), (0.003, 0.0012)],
              0.080, 0.050, 0.006, cseg=3)
    kn = lathe([(0, -0.006), (0.0016, -0.006), (0.002, -0.0052), (0.002, 0.0052), (0.0016, 0.006), (0, 0.006)],
               seg=6)
    for sx in (-1, 1):
        s = s + kn.copy().rot(z=math.pi / 2).move(x=sx * 0.022, y=0.001, z=-0.023)
    return s

part('EyeshadowPalettes__base', f_box(0.080, 0.006, 0.050), palette_base, smooth=40)


def palette_lid():
    # closed lid: crowned top, thumb notch in the front edge (+Z)
    s = yloft([(-0.003, 0.0006), (-0.0024, 0.0), (0.0012, 0.0), (0.0024, 0.0012), (0.003, 0.0034)],
              0.080, 0.050, 0.006, cseg=3)
    return cut(s, cyl(0.006, 0.006, 0.004, seg=12, bevel_r=0).rot(x=math.pi / 2).move(y=0.0005, z=0.026))

part('EyeshadowPalettes__lid', f_box(0.080, 0.006, 0.050), palette_lid, smooth=40)


def shadow_pan():
    # metal pan with pressed powder sitting below the rim, a used dip
    rings = []
    for y, ins in ((-0.002, 0.0003), (0.0018, 0.0), (0.0018, 0.0006), (0.0010, 0.0012), (0.0003, 0.0045)):
        rings.append([(x, y, z) for x, z in rrect_pts(0.020 - 2 * ins, 0.014 - 2 * ins, max(0.0006, 0.003 - ins * 0.5), 1)])
    return loft(rings)

part('EyeshadowPalettes__pan', f_box(0.020, 0.004, 0.014), shadow_pan, smooth=45)

F_PAL_MIRROR = f_bounds((-0.038, -0.0225, 0.0), (0.038, 0.0225, 0.0), 'plane')
# mirror plate: flat face with a narrow bevel, 1.6 mm thick behind the face
part('EyeshadowPalettes__mirror', F_PAL_MIRROR, lambda: yloft(
    [(-0.0016, 0.0003), (-0.0006, 0.0), (0.0, 0.0006)], 0.076, 0.045, 0.004, cseg=2).rot(x=math.pi / 2))


# ======================================================= perfume bottles ==
def perfume_cap():
    # fluted gold cap with a chamfered top
    s = lathe([(0, -0.005), (0.0138, -0.005), (0.014, -0.0043), (0.0124, 0.0034), (0.0110, 0.005), (0, 0.005)],
              seg=16)
    return knurl(s, 16, 0.0011, -0.0044, 0.0035)

part('PerfumeBottles__cap', f_cyl(0.012, 0.014, 0.010), perfume_cap, smooth=40)

# atomizer tube with a ferrule at each end
part('PerfumeBottles__tube', f_cyl(0.001, 0.001, 0.018), lambda: lathe(
    [(0, -0.009), (0.001, -0.009), (0.001, -0.0076), (0.0007, -0.007), (0.0007, 0.007), (0.001, 0.0076),
     (0.001, 0.009), (0, 0.009)], seg=6), smooth=50)


# =========================================================== nail polish ==
# squat bottle: thick rounded base, straight wall, shoulder into the neck
part('NailPolish__body', f_cyl(0.014, 0.014, 0.020), lambda: lathe(
    [(0, -0.01), (0.0128, -0.01), (0.0138, -0.0093), (0.014, -0.008), (0.014, 0.004), (0.0132, 0.0072),
     (0.0112, 0.0088), (0.0084, 0.0095), (0.0082, 0.01), (0, 0.01)], seg=16), smooth=40)


def polish_cap():
    s = lathe([(0, -0.0125), (0.0118, -0.0125), (0.012, -0.0115), (0.0112, 0.011), (0.0102, 0.0122),
               (0.009, 0.0125), (0, 0.0125)], seg=16)
    return knurl(s, 16, 0.0012, -0.0116, 0.0111)

part('NailPolish__cap', f_cyl(0.012, 0.012, 0.025), polish_cap, smooth=40)


# ============================================================== mascaras ==
# tube: rounded base (+Y), shoulder, threaded neck towards the cap (-Y)
part('Mascaras__tube', f_cyl(0.007, 0.007, 0.060), lambda: lathe(
    [(0, -0.03), (0.0045, -0.03), (0.0045, -0.0282), (0.0053, -0.0276), (0.0045, -0.027), (0.0046, -0.0252),
     (0.0068, -0.0236), (0.007, -0.022), (0.007, 0.0284), (0.0062, 0.03), (0, 0.03)], seg=12), smooth=45)

# octagonal cap: open end (+Y) with the wand socket, rounded closed end
part('Mascaras__cap', f_cyl(0.009, 0.009, 0.015), lambda: lathe(
    [(0, -0.0075), (0.0081, -0.0075), (0.009, -0.0062), (0.009, 0.0075), (0.0072, 0.0075), (0.0072, 0.0035),
     (0, 0.0035)], seg=8), smooth=30)

# wand: collar where it plugs into the cap (-Y), thinning to the brush (+Y)
part('Mascaras__wand', f_cyl(0.0015, 0.0015, 0.030), lambda: lathe(
    [(0, -0.015), (0.0015, -0.015), (0.0015, -0.012), (0.0011, -0.011), (0.0009, 0.015), (0, 0.015)], seg=6),
    smooth=50)


def mascara_brush():
    # spiral-bristle brush: ridged spindle along X (stretched further in game)
    s = lathe([(0, -0.006), (0.004, -0.005), (0.006, -0.0033), (0.0045, -0.0017), (0.006, 0.0),
               (0.0045, 0.0017), (0.006, 0.0033), (0.004, 0.005), (0, 0.006)], seg=6)
    return fitf(x_axis(s), f_sphere(0.006))

part('Mascaras__brush', f_sphere(0.006), mascara_brush, smooth=60)
