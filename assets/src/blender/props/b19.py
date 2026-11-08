"""b19: the "victorian" theme. Brass mechanisms, turned wood, wire cages,
upholstery and glass showpieces.

place_tripod_telescopes, place_orrery_models, place_birdcages, place_aquariums,
place_ornate_cuckoo_clocks, place_fireplaces, place_log_baskets,
place_wingback_armchairs, place_chair_arm_books, place_silhouette_portraits,
place_ottomans, place_humidors, place_brandy_snifters.

Glass, liquid, lenses, glows (basic material), flat faces/decals and
sub-millimetre ink lines are left as primitives (see b19.notes.md).
"""
import propkit
from propkit import *
from mathutils import Vector, noise

HPI = math.pi / 2


# ================================================================ helpers ==
def _fw(s):
    return propkit._fix_winding(s)


def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


def ax_x(s):
    """A shape built along +Y, laid along +X (+Y end goes to +X)."""
    return s.rot(z=-HPI)


def ax_z(s):
    """A shape built along +Y, laid along +Z (+Y end goes to +Z)."""
    return s.rot(x=HPI)


def flutes(s, n, depth, y0, y1, rmin=0.0):
    """Knurl / flute a lathe body: radius pulled in by depth*(0.5+0.5cos(n a))
    between y0 and y1 (only where r > rmin)."""
    def w(p):
        r = math.hypot(p.x, p.z)
        if r < max(rmin, 1e-7) or not (y0 - 1e-7 <= p.y <= y1 + 1e-7):
            return p
        k = max(0.0, r - depth * (0.5 + 0.5 * math.cos(n * math.atan2(p.x, p.z)))) / r
        p.x *= k
        p.z *= k
        return p
    return s.warp(w)


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
    """Sweep a closed section [(dr, dz), ...] around the Z axis in the XY
    plane (a torus_geo! with any cross-section). rmod(a) -> (k, dz)."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        kr, dzz = rmod(a) if rmod else (1.0, 0.0)
        for dr, dz in sec:
            v.append(((R + dr * kr) * ca, (R + dr * kr) * sa, dz * kr + dzz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


def circ(r, n=8, sx=1.0, sy=1.0, phase=0.0):
    return [(r * sx * math.cos(TAU * i / n + phase), r * sy * math.sin(TAU * i / n + phase)) for i in range(n)]


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


def rrect_pts(w, d, r, cseg=3):
    """Rounded rectangle outline [(a, b), ...], counter-clockwise."""
    r = max(1e-5, min(r, w / 2 - 1e-5, d / 2 - 1e-5))
    out = []
    for cx, cz, a0 in ((w / 2 - r, d / 2 - r, 0), (-w / 2 + r, d / 2 - r, 90),
                       (-w / 2 + r, -d / 2 + r, 180), (w / 2 - r, -d / 2 + r, 270)):
        for i in range(cseg + 1):
            a = math.radians(a0 + 90 * i / cseg)
            out.append((cx + r * math.cos(a), cz + r * math.sin(a)))
    return out


def yloft(profile, w, d, rc, cseg=3):
    """Moulding along +Y: rounded rectangles, profile = [(y, inset), ...]."""
    rings = []
    for y, ins in profile:
        pts = rrect_pts(w - 2 * ins, d - 2 * ins, max(0.0003, rc - ins * 0.6), cseg)
        rings.append([(x, y, z) for x, z in pts])
    return loft(rings)


def zloft(profile, w, h, rc, cseg=3):
    """Moulding along +Z (wall things): profile = [(z, inset), ...]."""
    rings = []
    for z, ins in profile:
        pts = rrect_pts(w - 2 * ins, h - 2 * ins, max(0.0003, rc - ins * 0.6), cseg)
        rings.append([(x, y, z) for x, y in pts])
    return loft(rings)


def frame_ring(w, d, prof, rc_out, cseg=2):
    """A rectangular frame ring lying in XZ (loop section): prof is a closed
    list of (inset, y) points going round the moulding's section."""
    rings = []
    for ins, y in prof:
        pts = rrect_pts(w - 2 * ins, d - 2 * ins, max(0.0003, rc_out - ins), cseg)
        rings.append([(x, y, z) for x, z in pts])
    n = len(rings[0])
    v, f = [], []
    for ring in rings:
        v += [Vector(p) for p in ring]
    R = len(rings)
    for k in range(R):
        a, b = k * n, ((k + 1) % R) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    return _fw(Shape(v, f))


def rod(r, h, seg=6):
    """A thin round rod on +Y with eased ends."""
    e = min(r * 0.6, h * 0.1)
    return lathe([(0, -h / 2), (r * 0.7, -h / 2), (r, -h / 2 + e), (r, h / 2 - e), (r * 0.7, h / 2), (0, h / 2)], seg=seg)


# ===================================================== tripod telescopes ==
# Brass-and-mahogany refractor on a wooden field tripod.
F_TT_LEG = f_cyl(0.008, 0.010, 0.85)


def tt_leg():
    # two-stage sliding leg: slim inner leg in a clamp collar, ferrule foot,
    # rounded hinge end on top; a wing screw on the clamp collar
    s = lathe([(0, -0.425), (0.0055, -0.425), (0.0075, -0.417), (0.0088, -0.405), (0.0100, -0.400),
               (0.0100, -0.378), (0.0086, -0.372), (0.0074, -0.362), (0.0072, -0.10), (0.0094, -0.096),
               (0.0100, -0.088), (0.0100, -0.060), (0.0094, -0.054), (0.0090, -0.050), (0.0082, 0.380),
               (0.0090, 0.386), (0.0090, 0.402), (0.0074, 0.414), (0.0050, 0.422), (0, 0.425)], seg=10)
    screw = lathe([(0, 0), (0.0022, 0), (0.0022, 0.004), (0.0016, 0.0045), (0, 0.0046)], seg=6)
    wing = rbox(0.0026, 0.0012, 0.009, r=0.0005, seg=1).move(y=0.0052)
    return s + ax_x(screw + wing).move(x=0.0088, y=-0.074)


part('TripodTelescopes__leg', F_TT_LEG, tt_leg)
# rubber foot cap: flat tread, bulging sides, dished socket rim on top
part('TripodTelescopes__foot', f_cyl(0.012, 0.012, 0.008), lambda: lathe(
    [(0, -0.004), (0.0105, -0.004), (0.0118, -0.0032), (0.0120, -0.0010), (0.0116, 0.0018), (0.0106, 0.0034),
     (0.0092, 0.0040), (0.0082, 0.0036), (0, 0.0036)], seg=14), smooth=50)


def tt_head():
    # tripod crown: round leg plate, chamfered clamp block, azimuth boss on top
    crown = lathe([(0, -0.025), (0.0285, -0.025), (0.0300, -0.0235), (0.0300, -0.0175),
                   (0, -0.0175)], seg=12)
    block = rbox(0.060, 0.022, 0.060, r=0.004, seg=1).move(y=-0.006)
    boss = lathe([(0.022, 0.005), (0.022, 0.012), (0.019, 0.014), (0.019, 0.018), (0.014, 0.023),
                  (0, 0.025)], seg=12, cap_bottom=True)
    return crown + block + boss


part('TripodTelescopes__head', f_box(0.060, 0.050, 0.060), tt_head)

# knurled brass thumb wheel, the same both faces (used on three axes)
part('TripodTelescopes__knob', f_cyl(0.014, 0.014, 0.010), lambda: flutes(lathe(
    [(0, -0.005), (0.0100, -0.0045), (0.0135, -0.0030), (0.0140, -0.0018),
     (0.0140, 0.0018), (0.0135, 0.0030), (0.0100, 0.0045), (0, 0.005)], seg=22),
    11, 0.0010, -0.0031, 0.0031, rmin=0.013))


def tt_tube():
    # -Y end is the objective (dew shield with a lens recess), +Y the rear;
    # two reinforcing bands round the barrel
    return lathe([(0, -0.330), (0.0285, -0.330), (0.0295, -0.348), (0.0330, -0.350), (0.0350, -0.346),
                   (0.0350, -0.232), (0.0336, -0.226), (0.0316, -0.222), (0.0316, -0.120), (0.0334, -0.117),
                   (0.0334, -0.101), (0.0316, -0.098), (0.0316, 0.100), (0.0334, 0.103), (0.0334, 0.119),
                   (0.0316, 0.122), (0.0316, 0.300), (0.0300, 0.306), (0.0300, 0.350), (0, 0.350)], seg=22)


part('TripodTelescopes__tube', f_cyl(0.035, 0.035, 0.70), tt_tube)

# objective cell bezel: front (+Z) chamfered towards the lens, bead at the back
part('TripodTelescopes__objRing', f_torus(0.035, 0.005), lambda: ring_section(0.035, [
    (-0.0050, -0.0040), (-0.0050, 0.0020), (-0.0035, 0.0050), (0.0010, 0.0050), (0.0050, 0.0015),
    (0.0050, -0.0030), (0.0030, -0.0050)], seg=18))

# rear cell: flange on the tube (-Y), stepped down to the draw-tube collar (+Y)
part('TripodTelescopes__rearCap', f_cyl(0.040, 0.040, 0.030), lambda: lathe(
    [(0, -0.015), (0.0385, -0.015), (0.0400, -0.013), (0.0400, -0.005), (0.0360, -0.002), (0.0300, 0.000),
     (0.0290, 0.004), (0.0200, 0.009), (0.0165, 0.010), (0.0165, 0.015), (0, 0.015)], seg=14))


def tt_eyepiece():
    s = lathe([(0, -0.030), (0.0085, -0.030), (0.0085, -0.013), (0.0100, -0.011), (0.0100, 0.002),
               (0.0092, 0.005), (0.0140, 0.021), (0.0140, 0.027), (0.0122, 0.030),
               (0.0092, 0.029), (0, 0.028)], seg=14)
    return flutes(s, 8, 0.0012, -0.0111, 0.0021, rmin=0.0095)


part('TripodTelescopes__eyepiece', f_cyl(0.014, 0.010, 0.060), tt_eyepiece)

# finder: objective cell at -Y (front), slim body, eyepiece collar at +Y
part('TripodTelescopes__finder', f_cyl(0.010, 0.010, 0.16), lambda: lathe(
    [(0, -0.076), (0.0082, -0.076), (0.0086, -0.0795), (0.0098, -0.0800), (0.0100, -0.0775),
     (0.0100, -0.050), (0.0086, -0.047), (0.0072, -0.045), (0.0072, 0.050), (0.0080, 0.052),
     (0.0080, 0.062), (0.0064, 0.064), (0.0056, 0.080), (0, 0.080)], seg=12))

# finder post: slim stem with a saddle foot on the tube
part('TripodTelescopes__bracket', f_cyl(0.002, 0.002, 0.020), lambda: lathe(
    [(0, -0.010), (0.0020, -0.010), (0.0020, -0.0085), (0.0011, -0.0065), (0.0011, 0.0080),
     (0.0016, 0.0090), (0.0016, 0.0100), (0, 0.0100)], seg=6))


# ======================================================== orrery models ==
# turned wooden plinth: ogee edge, bead, incised ring on the top
part('OrreryModels__base', f_cyl(0.10, 0.11, 0.014), lambda: lathe(
    [(0, -0.007), (0.1100, -0.0065), (0.1100, -0.0030), (0.1060, -0.0010),
     (0.1040, 0.0010), (0.1040, 0.0030), (0.1010, 0.0055), (0.0980, 0.0070), (0.0620, 0.0070),
     (0.0610, 0.0060), (0.0580, 0.0060), (0.0570, 0.0070), (0, 0.0070)], seg=24))

# brass column: foot moulding, knop, slender shaft, collar, cup under the hub
part('OrreryModels__column', f_cyl(0.010, 0.012, 0.20), lambda: lathe(
    [(0, -0.100), (0.0120, -0.100), (0.0120, -0.095), (0.0092, -0.092), (0.0072, -0.086),
     (0.0086, -0.079), (0.0072, -0.072), (0.0056, -0.066), (0.0050, 0.060), (0.0074, 0.065),
     (0.0082, 0.070), (0.0060, 0.076), (0.0054, 0.088), (0.0090, 0.093), (0.0100, 0.097),
     (0.0100, 0.100), (0, 0.100)], seg=12))


def orrery_gear():
    # spur gear (12 teeth) with a raised hub boss
    n, rr, rt = 12, 0.0205, 0.0240
    out = []
    for i in range(n):
        a = TAU * i / n
        for da, r in ((-0.30, rr), (-0.14, rt), (0.14, rt), (0.30, rr)):
            aa = a + da * TAU / n
            out.append((r * math.cos(aa), r * math.sin(aa)))
    g = prism(out, 0.006).rot(x=-HPI).move(y=-0.002)
    boss = lathe([(0.012, 0.001), (0.012, 0.0035), (0.010, 0.005), (0, 0.005)], seg=12, cap_bottom=True)
    return g + boss


part('OrreryModels__hub', f_cyl(0.024, 0.024, 0.010), orrery_gear)
part('OrreryModels__arm', f_cyl(0.001, 0.001, 0.10), lambda: rod(0.001, 0.10))
part('OrreryModels__planet', f_sphere(0.010), lambda: sphere(0.010, seg=14, rings=9), smooth=180)


# ============================================================= birdcages ==
# cast-iron stand and a wire cage; bars are twisted square iron
part('Birdcages__base', f_cyl(0.10, 0.12, 0.014), lambda: lathe(
    [(0, -0.007), (0.1180, -0.007), (0.1200, -0.0055), (0.1200, -0.0040), (0.1130, -0.0022),
     (0.1070, -0.0002), (0.1040, 0.0018), (0.1000, 0.0030), (0.0960, 0.0036), (0.0600, 0.0052),
     (0.0300, 0.0064), (0.0150, 0.0070), (0, 0.0070)], seg=24))

part('Birdcages__pole', f_cyl(0.008, 0.010, 1.10), lambda: lathe(
    [(0, -0.550), (0.0100, -0.550), (0.0100, -0.534), (0.0070, -0.528), (0.0060, -0.170), (0.0092, -0.158),
     (0.0098, -0.148), (0.0092, -0.138), (0.0060, -0.126), (0.0056, 0.300), (0.0080, 0.308), (0.0056, 0.316),
     (0.0052, 0.522), (0.0080, 0.532), (0.0080, 0.550), (0, 0.550)], seg=10))

# cage floor: a tray with a turned-up lip (rises above the frame, as a tray does)
part('Birdcages__floor', f_cyl(0.11, 0.11, 0.006), lambda: lathe(
    [(0, -0.003), (0.1060, -0.003), (0.1095, -0.0020), (0.1100, 0.0010), (0.1100, 0.0060), (0.1085, 0.0068),
     (0.1065, 0.0062), (0.1060, 0.0030), (0, 0.0030)], seg=32))

# conical top plate with a beaded edge
part('Birdcages__top', f_cyl(0.055, 0.099, 0.006), lambda: lathe(
    [(0, -0.003), (0.0970, -0.003), (0.0990, -0.0015), (0.0990, 0.0000), (0.0955, 0.0010),
     (0.0750, 0.0018), (0.0580, 0.0030), (0, 0.0030)], seg=28))


def cage_dome():
    # ribbed onion dome with a finial knob on top
    s = lathe([(0.0550, 0.000), (0.0545, 0.008), (0.0500, 0.020), (0.0420, 0.031), (0.0300, 0.040),
               (0.0160, 0.045), (0.0070, 0.047), (0.0060, 0.049), (0.0075, 0.0515), (0.0055, 0.0540),
               (0, 0.0550)], seg=24)
    return flutes(s, 12, 0.0022, 0.0, 0.044, rmin=0.012)


part('Birdcages__dome', f_bounds((-0.055, 0, -0.055), (0.055, 0.055, 0.055)), cage_dome)
part('Birdcages__hook', f_torus(0.014, 0.002), lambda: torus(0.014, 0.002, seg=20, tseg=6), smooth=60)


def twisted_bar(r, h, steps=12, turns=1.5):
    """Square iron bar twisted along +Y, ends eased."""
    rings = []
    for i in range(steps + 1):
        t = i / steps
        y = -h / 2 + h * t
        k = 0.8 if i in (0, steps) else 1.0
        a0 = turns * TAU * t
        rings.append([(r * k * math.cos(a0 + HPI * j), y, r * k * math.sin(a0 + HPI * j)) for j in range(4)])
    return loft(rings)


part('Birdcages__bar', f_cyl(0.002, 0.002, 0.30), lambda: twisted_bar(0.002, 0.30), smooth=50)

# flat strap hoop round the bars
part('Birdcages__ring', f_torus(0.088, 0.002), lambda: ring_section(0.088, [
    (-0.0010, -0.0020), (0.0010, -0.0020), (0.0014, 0.0), (0.0010, 0.0020), (-0.0010, 0.0020),
    (-0.0014, 0.0)], seg=36))

# turned wooden perch with beaded ends
part('Birdcages__perch', f_cyl(0.005, 0.005, 0.176), lambda: lathe(
    [(0, -0.088), (0.0035, -0.088), (0.0050, -0.0855), (0.0050, -0.0810), (0.0036, -0.0790),
     (0.0042, -0.0700), (0.0045, 0.0), (0.0042, 0.0700), (0.0036, 0.0790), (0.0050, 0.0810),
     (0.0050, 0.0855), (0.0035, 0.088), (0, 0.088)], seg=8), smooth=50)


def feather():
    # built at its in-game size (sphere scaled 0.5, 0.15, 2.5): vane with a
    # split, quill running out at -Z, arched along its length
    pts = [(0.0005, -0.0200), (0.0007, -0.0145), (0.0026, -0.0128), (0.0038, -0.0080), (0.0040, 0.0),
           (0.0035, 0.0080), (0.0022, 0.0150), (0.0008, 0.0192), (0.0, 0.0200), (-0.0010, 0.0188),
           (-0.0028, 0.0140), (-0.0036, 0.0060), (-0.0024, 0.0042), (-0.0034, 0.0020), (-0.0038, -0.0060),
           (-0.0030, -0.0120), (-0.0007, -0.0145), (-0.0005, -0.0200)]
    s = prism([(x, -z) for x, z in pts], 0.0005).rot(x=-HPI)
    s = s.warp(lambda p: Vector((p.x, p.y + 0.0016 * (1 - (p.z / 0.02) ** 2) - 0.0004 * (p.x / 0.004) ** 2, p.z)))
    s = s.scale(1 / 0.5, 1 / 0.15, 1 / 2.5)
    return fitf(s, f_sphere(0.008))


part('Birdcages__feather', f_sphere(0.008), feather, smooth=60)

# glazed seed / water dish
part('Birdcages__dish', f_cyl(0.020, 0.018, 0.008), lambda: vessel(
    [(0.0170, -0.004), (0.0182, -0.0036), (0.0192, -0.0015), (0.0198, 0.0015), (0.0200, 0.0035)],
    wall=0.0018, seg=14), smooth=50)

part('Birdcages__seed', f_sphere(0.002), lambda: fitf(lathe(
    [(0, -0.0020), (0.0010, -0.0016), (0.0017, -0.0004), (0.0015, 0.0010), (0.0008, 0.0018), (0, 0.0020)],
    seg=6).rot(z=0.3), f_sphere(0.002)), smooth=180)


# ============================================================= aquariums ==
def aq_stand():
    # cabinet: flush front for the drawer fronts, shadow groove under the
    # top, recessed door panels below the drawers, fielded side panels,
    # cut-out plinth leaving four block feet
    s = rbox(0.55, 0.60, 0.30, r=0.005, seg=1)
    cutters = [
        rbox(0.230, 0.150, 0.02, r=0.003, seg=1).move(x=-0.128, y=-0.160, z=0.155),
        rbox(0.230, 0.150, 0.02, r=0.003, seg=1).move(x=0.128, y=-0.160, z=0.155),
        rbox(0.02, 0.44, 0.24, r=0.003, seg=1).move(x=0.280, y=0.0),
        rbox(0.02, 0.44, 0.24, r=0.003, seg=1).move(x=-0.280, y=0.0),
        box(0.60, 0.005, 0.008).move(y=0.268, z=0.150),
        box(0.008, 0.005, 0.32).move(x=0.275, y=0.268),
        box(0.008, 0.005, 0.32).move(x=-0.275, y=0.268),
        box(0.46, 0.035, 0.03).move(y=-0.290, z=0.150),
        box(0.03, 0.035, 0.22).move(x=0.275, y=-0.290),
        box(0.03, 0.035, 0.22).move(x=-0.275, y=-0.290),
    ]
    return cut(s, *cutters)


part('Aquariums__stand', f_box(0.55, 0.60, 0.30), aq_stand)

# fielded drawer front: flat border, bevel up to a raised field
part('Aquariums__drawer', f_box(0.24, 0.16, 0.003), lambda: zloft(
    [(-0.0015, 0.0), (0.0000, 0.0), (0.0004, 0.0008), (0.0004, 0.012), (0.0015, 0.020)], 0.24, 0.16, 0.004,
    cseg=1))

# turned mushroom knob, its neck at -Z on the drawer
part('Aquariums__knob', f_sphere(0.006), lambda: fitf(ax_z(lathe(
    [(0, -0.006), (0.0030, -0.006), (0.0022, -0.0040), (0.0054, 0.0008), (0.0060, 0.0030),
     (0.0040, 0.0056), (0, 0.0060)], seg=8)), f_sphere(0.006)), smooth=60)

# black angle trim round the tank: an open rectangle, rounded outer edge
part('Aquariums__frame', f_box(0.506, 0.010, 0.286), lambda: frame_ring(0.506, 0.286, [
    (0.0, -0.005), (0.0, 0.0038), (0.0012, 0.0050), (0.0150, 0.0050), (0.0160, 0.0040), (0.0160, -0.005)],
    0.004, cseg=2))


def heightfield(w, d, nx, nz, hfn, ybot):
    """A block whose top is a height field hfn(x, z) over a w x d grid, with
    straight sides down to a flat bottom at ybot."""
    xs = [-w / 2 + w * i / nx for i in range(nx + 1)]
    zs = [-d / 2 + d * j / nz for j in range(nz + 1)]
    v, f = [], []
    idx = {}
    for j, z in enumerate(zs):
        for i, x in enumerate(xs):
            idx[(i, j)] = len(v)
            v.append((x, hfn(x, z), z))
    for j in range(nz):
        for i in range(nx):
            f.append((idx[(i, j)], idx[(i + 1, j)], idx[(i + 1, j + 1)], idx[(i, j + 1)]))
    rim = [(i, 0) for i in range(nx)] + [(nx, j) for j in range(nz)] + \
          [(i, nz) for i in range(nx, 0, -1)] + [(0, j) for j in range(nz, 0, -1)]
    base = len(v)
    for key in rim:
        x, _, z = v[idx[key]]
        v.append((x, ybot, z))
    n = len(rim)
    for k in range(n):
        k2 = (k + 1) % n
        f.append((idx[rim[k]], idx[rim[k2]], base + k2, base + k))
    f.append(tuple(base + k for k in range(n)))
    return _fw(Shape(v, f))


def aq_gravel():
    # gravel bed banked up towards the back, lumpy with pebbles
    def h(x, z):
        u = noise.noise(Vector((x * 45, z * 45, 0.3))) * 0.0035 + noise.noise(Vector((x * 120, z * 120, 1.7))) * 0.0016
        return 0.006 - 0.004 * (z / 0.13) + u
    return fitf(heightfield(0.48, 0.26, 24, 13, h, -0.014), f_box(0.48, 0.028, 0.26))


part('Aquariums__gravel', f_box(0.48, 0.028, 0.26), aq_gravel, smooth=60)


def aq_blade():
    # strap-leaved water plant: pinched base, widest low down, tapering to a
    # point; a midrib, a gentle lean and a lazy wave through the thin axis
    rings, n = [], 12
    for i in range(n + 1):
        t = i / n
        y = -0.09 + 0.18 * t
        half = 0.003 * (0.55 + 0.45 * math.sin(math.pi * min(1.0, t * 1.6 + 0.2))) * (1 - t ** 3)
        half = max(half, 0.0002)
        th = 0.001 * (1 - 0.6 * t)
        cx = 0.0025 * t * t
        cz = 0.0022 * math.sin(t * 5.0)
        rings.append([(cx - half, y, cz), (cx, y, cz + th), (cx + half, y, cz), (cx, y, cz - th)])
    return loft(rings)


part('Aquariums__blade', f_box(0.006, 0.18, 0.002), aq_blade, smooth=60)


def aq_fish():
    # built at its in-game size (sphere scaled 1.4, 0.7, 0.5): head at +X,
    # tapering to the tail root at -X, a dorsal fin ridge on top
    rings = []
    st = [(-0.0252, 0.0020, 0.0012, 0.0), (-0.0200, 0.0040, 0.0022, 0.0), (-0.0120, 0.0082, 0.0055, 0.0010),
          (-0.0040, 0.0112, 0.0080, 0.0022), (0.0040, 0.0118, 0.0090, 0.0012), (0.0120, 0.0105, 0.0085, 0.0),
          (0.0180, 0.0082, 0.0070, 0.0), (0.0225, 0.0050, 0.0045, 0.0), (0.0252, 0.0012, 0.0012, 0.0)]
    for x, ry, rz, fin in st:
        ring = []
        for k in range(10):
            a = TAU * k / 10
            yy = ry * math.sin(a) * (0.92 if math.sin(a) < 0 else 1.0)
            if k == 3 or k == 2:
                yy += fin * (0.6 if k == 2 else 1.0)
            ring.append((x, yy, rz * math.cos(a)))
        rings.append(ring)
    s = loft(rings).scale(1 / 1.4, 1 / 0.7, 1 / 0.5)
    return fitf(s, f_sphere(0.018))


part('Aquariums__fish', f_sphere(0.018), aq_fish, smooth=70)


def aq_tail():
    # fantail: two forked fins splayed either side of the body plane, joined
    # at a slim root towards the apex (+Y, the body end)
    out = [(0.0015, 0.007), (0.0060, 0.001), (0.0125, -0.007), (0.0070, -0.0052), (0.0, -0.0030),
           (-0.0070, -0.0052), (-0.0125, -0.007), (-0.0060, 0.001), (-0.0015, 0.007)]
    fin = prism(out, 0.0008)
    root = lathe([(0, 0.002), (0.0018, 0.003), (0.0016, 0.007), (0, 0.0072)], seg=6)
    s = fin.copy().rot(y=0.75) + fin.copy().rot(y=-0.75) + root
    return fitf(s, f_cone(0.010, 0.014))


part('Aquariums__tail', f_cone(0.010, 0.014), aq_tail, smooth=40)


def aq_canopy():
    # light hood: chamfered top, a hinged feeding flap cut into the lid
    s = yloft([(-0.010, 0.0), (0.004, 0.0), (0.0075, 0.0025), (0.0100, 0.0080)], 0.514, 0.294, 0.006, cseg=2)
    return cut(s, box(0.49, 0.003, 0.003).move(y=0.010, z=-0.03),
               box(0.003, 0.003, 0.11).move(x=0.12, y=0.010, z=0.028),
               box(0.003, 0.003, 0.11).move(x=-0.12, y=0.010, z=0.028),
               rbox(0.05, 0.01, 0.012, r=0.003, seg=1).move(y=0.0105, z=0.11))


part('Aquariums__canopy', f_box(0.514, 0.020, 0.294), aq_canopy)


# ================================================== ornate cuckoo clocks ==
def leaf(L, W, T, seg=6):
    """A carved leaf: flattened, pointed ellipsoid along +Y, thin in Z."""
    s = lathe([(0, -L / 2), (W * 0.45, -L * 0.30), (W * 0.50, 0.0), (W * 0.32, L * 0.28), (0, L / 2)], seg=seg)
    return s.scale(1, 1, T / W)


def cc_body():
    # Black Forest case: moulded base and cornice, raised front border, a
    # carved leaf wreath round the dial (open at the top for the cuckoo
    # door) and leaf sprays in the top corners
    core = rbox(0.206, 0.30, 0.094, r=0.004, seg=1).move(y=0.0, z=-0.003)
    base = rbox(0.220, 0.026, 0.100, r=0.005, seg=1).move(y=-0.147)
    corn = rbox(0.220, 0.020, 0.100, r=0.004, seg=1).move(y=0.150)
    border = frame_ring(0.206, 0.274, [(0.0, 0.0), (0.0, 0.0035), (0.004, 0.0048), (0.010, 0.0048),
                                       (0.012, 0.0)], 0.004, cseg=1).rot(x=HPI).move(y=0.0, z=0.044)
    wreath = Shape()
    fy = 0.048
    for i in range(11):
        a = math.radians(-60 + 300 * i / 10) if False else math.radians(120 + 300 * i / 10)
        lf = leaf(0.024, 0.012, 0.004).rot(z=a - HPI + 0.5).move(x=0.079 * math.cos(a), y=fy + 0.079 * math.sin(a),
                                                                z=0.050)
        wreath = wreath + lf
    corners = Shape()
    for sx in (-1, 1):
        for k, (ang, dx, dy) in enumerate(((0.3, 0.0, 0.0), (1.2, 0.010, -0.006), (-0.6, -0.004, -0.010))):
            corners = corners + leaf(0.022, 0.010, 0.004).rot(z=sx * (ang + 0.6)).move(
                x=sx * (0.080 - dx), y=0.118 + dy, z=0.049)
    bottom = Shape()
    for k in range(-2, 3):
        bottom = bottom + leaf(0.026, 0.011, 0.004).rot(z=k * 0.45).move(x=k * 0.012, y=-0.112 + abs(k) * 0.004,
                                                                          z=0.049)
    return core + base + corn + border + wreath + corners + bottom


part('OrnateCuckooClocks__body', f_box(0.22, 0.32, 0.10), cc_body)


def cc_roof():
    # fish-scale shingles on the top face (symmetric: the same panel is used
    # for both slopes), scalloped barge edge at the front (+Z)
    def h(x, z):
        row = x / 0.022
        ph = 0.5 if int(math.floor(row)) % 2 else 0.0
        u = (row - math.floor(row))
        w = abs(math.sin(math.pi * (z / 0.022 + ph)))
        return 0.0025 + 0.0025 * (1 - u) * (0.5 + 0.5 * w)
    s = heightfield(0.198, 0.12, 18, 10, h, -0.005)

    def scallop(p):
        if p.z > 0.0599:
            p.z -= 0.004 * (1 - abs(math.sin(math.pi * p.x / 0.033)))
        return p
    return fitf(s.warp(scallop), f_box(0.198, 0.010, 0.12))


part('OrnateCuckooClocks__roof', f_box(0.198, 0.010, 0.12), cc_roof)

# ridge cap: rounded beam running front to back
part('OrnateCuckooClocks__ridge', f_box(0.010, 0.012, 0.12), lambda: prism(
    [(-0.005, -0.006), (0.005, -0.006), (0.005, 0.0), (0.0036, 0.0044), (0.0, 0.006), (-0.0036, 0.0044),
     (-0.005, 0.0)], 0.12))

# scalloped pendant bracket under the eaves
part('OrnateCuckooClocks__trim', f_box(0.014, 0.010, 0.006), lambda: prism(
    [(-0.007, 0.005), (0.007, 0.005), (0.007, 0.001), (0.0055, -0.0006), (0.0035, -0.0010), (0.0015, -0.0020),
     (0.0, -0.005), (-0.0015, -0.0020), (-0.0035, -0.0010), (-0.0055, -0.0006), (-0.007, 0.001)], 0.006))

# brass dial bezel: flat back, beaded front (+Z)
part('OrnateCuckooClocks__rim', f_torus(0.065, 0.004), lambda: ring_section(0.065, [
    (-0.0040, -0.0040), (0.0040, -0.0040), (0.0040, 0.0000), (0.0026, 0.0030), (0.0000, 0.0040),
    (-0.0030, 0.0026), (-0.0040, 0.0000)], seg=32))


def clock_hand(w, L):
    """Spade hand lying in XZ, pivot boss at -Z, tip at +Z."""
    z0, z1 = -L / 2, L / 2
    rb = w / 2
    pts = [(rb * math.cos(a), z0 + rb + rb * math.sin(a)) for a in
           (math.radians(d) for d in (-20, -60, -90, -120, -160))]
    pts = [(x, z) for x, z in pts]
    pts = [(rb * 0.35, z0 + 2 * rb)] + [(-x, z) for x, z in pts][::-1][:0] + pts
    body = [(rb * 0.35, z0 + 2 * rb), (rb * 0.25, z1 - L * 0.30), (rb * 0.95, z1 - L * 0.20),
            (rb * 0.25, z1 - L * 0.08), (0.0, z1), (-rb * 0.25, z1 - L * 0.08), (-rb * 0.95, z1 - L * 0.20),
            (-rb * 0.25, z1 - L * 0.30), (-rb * 0.35, z0 + 2 * rb)]
    boss = [(rb * math.cos(a), z0 + rb + rb * math.sin(a)) for a in
            (math.radians(d) for d in (200, 240, 270, 300, 340))]
    out = body + boss
    return prism([(x, -z) for x, z in out], 0.001).rot(x=-HPI)


part('OrnateCuckooClocks__hourHand', f_box(0.005, 0.001, 0.028), lambda: clock_hand(0.005, 0.028))
part('OrnateCuckooClocks__minuteHand', f_box(0.003, 0.001, 0.042), lambda: clock_hand(0.003, 0.042))


def cc_door():
    # cuckoo door: backing plate with an arched raised panel
    plate = rbox(0.032, 0.028, 0.0012, r=0.0004, seg=1).move(z=-0.0009)
    arch = [(-0.011, -0.011), (0.011, -0.011), (0.011, 0.002)]
    arch += [(0.011 * math.cos(math.pi * k / 8), 0.002 + 0.010 * math.sin(math.pi * k / 8)) for k in range(1, 8)]
    arch += [(-0.011, 0.002)]
    panel = prism(arch, 0.0020).move(z=0.0005)
    return plate + panel


part('OrnateCuckooClocks__door', f_box(0.032, 0.028, 0.003), cc_door)
part('OrnateCuckooClocks__rod', f_cyl(0.001, 0.001, 0.20), lambda: rod(0.001, 0.20))


def cc_chain():
    # small wire links, each at right angles to the next
    n, L = 22, 0.18
    pitch = L / n
    ll, rc, rw = pitch * 1.25, 0.0006, 0.0004
    shape = Shape()
    for i in range(n):
        y = -L / 2 + pitch * (i + 0.5)
        cl = [(rc, 0.0), (0.0, ll / 2 - rw), (-rc, 0.0), (0.0, -ll / 2 + rw)]
        v, f = [], []
        for k, (a, b) in enumerate(cl):
            for j in range(3):
                t = TAU * j / 3
                ang = math.atan2(b, a)
                v.append((a + rw * math.cos(t) * math.cos(ang), b + rw * math.cos(t) * math.sin(ang),
                          rw * math.sin(t)))
        for k in range(4):
            k2 = (k + 1) % 4
            for j in range(3):
                j2 = (j + 1) % 3
                f.append((k * 3 + j, k2 * 3 + j, k2 * 3 + j2, k * 3 + j2))
        lk = _fw(Shape(v, f))
        if i % 2:
            lk = lk.rot(y=HPI)
        shape = shape + lk.move(y=y)
    return fitf(shape, f_cyl(0.001, 0.001, 0.18))


part('OrnateCuckooClocks__chain', f_cyl(0.001, 0.001, 0.18), cc_chain, smooth=60)


def pinecone(seg=12, tiers=5):
    """Pine-cone weight: tiers of overlapping scales, each tier flaring out to
    a scalloped lip on its apex side (it hangs apex-down), flat top at -Y."""
    R, H = 0.016, 0.030
    env = [(0.0, 0.66), (0.10, 0.98), (0.35, 1.0), (0.65, 0.82), (0.88, 0.48), (1.0, 0.0)]

    def r_at(t):
        for (t0, r0), (t1, r1) in zip(env, env[1:]):
            if t <= t1:
                return r0 + (r1 - r0) * (t - t0) / (t1 - t0)
        return 0.0
    rings = []
    t0, t1 = 0.0, 0.93
    for k in range(tiers):
        ta = t0 + (t1 - t0) * k / tiers
        tb = t0 + (t1 - t0) * (k + 1) / tiers
        ph = (k % 2) * math.pi / (seg // 2)
        root = [(R * r_at(ta) * 0.80 * math.sin(TAU * j / seg), -H / 2 + H * ta, R * r_at(ta) * 0.80 * math.cos(TAU * j / seg))
                for j in range(seg)]
        lip = []
        for j in range(seg):
            a = TAU * j / seg
            sc = 0.5 + 0.5 * math.cos((seg // 2) * a + ph)
            r = R * r_at(tb - 0.02) * (0.76 + 0.24 * sc)
            lip.append((r * math.sin(a), -H / 2 + H * (tb - 0.02) + 0.0022 * sc, r * math.cos(a)))
        rings += [root, lip]
    s = loft(rings, cap0=True, cap1=False)
    v, f = list(s.v), list(s.f)
    last = len(rings) - 1
    base = len(v)
    tip = Vector((0.0, H / 2, 0.0))
    v.append(tip)
    for j in range(seg):
        f.append((last * seg + j, last * seg + (j + 1) % seg, base))
    return fitf(_fw(Shape(v, f)), f_cone(R, H))


part('OrnateCuckooClocks__pinecone', f_cone(0.016, 0.030), pinecone, smooth=50)


# ============================================================ fireplaces ==
def brick(x0, x1, y0, y1, z0, z1, c=0.004):
    """A brick whose front (+Z) edges are chamfered, so the joints read."""
    r0 = [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0)]
    r1 = [(x0, y0, z1 - c), (x1, y0, z1 - c), (x1, y1, z1 - c), (x0, y1, z1 - c)]
    r2 = [(x0 + c, y0 + c, z1), (x1 - c, y0 + c, z1), (x1 - c, y1 - c, z1), (x0 + c, y1 - c, z1)]
    return loft([r0, r1, r2])


def brick_course(x0, x1, y0, y1, z0, z1, bl=0.225, gap=0.007, phase=0.0):
    """One course of plain bricks along X between x0..x1 (bond offset
    `phase`), mortar gaps left open to the core behind."""
    out = Shape()
    x = x0 - phase
    while x < x1 - 1e-6:
        a, b = max(x, x0), min(x + bl - gap, x1)
        if b - a > 0.02:
            out = out + brick(a, b, y0, y1, z0, z1)
        x += bl
    return out


def brick_block(w, h, d, y_rows, hole=None):
    """A brick mass w x h x d centred on the origin: courses of bricks in
    running bond over a mortar core inset 5 mm. y_rows = list of (y0, y1,
    kind) with kind 'run' or 'soldier'. hole = (x0, x1, y0, y1, zb): a recess
    open to the front (+Z) down to zb."""
    g = 0.007
    core = Shape()
    zc0, zc1 = -d / 2, d / 2 - 0.005
    if hole:
        hx0, hx1, hy0, hy1, zb = hole
        core = box(w - 0.01, h / 2 - hy1, zc1 - zc0).move(0, (hy1 + h / 2) / 2, (zc0 + zc1) / 2)
        if hy0 > -h / 2 + 1e-6:
            core = core + box(w - 0.01, hy0 + h / 2, zc1 - zc0).move(0, (hy0 - h / 2) / 2, (zc0 + zc1) / 2)
        core = core + box(hx0 + w / 2 - 0.005, hy1 - hy0, zc1 - zc0).move((hx0 - w / 2) / 2 + 0.0025, (hy0 + hy1) / 2,
                                                                         (zc0 + zc1) / 2)
        core = core + box(w / 2 - hx1 - 0.005, hy1 - hy0, zc1 - zc0).move((hx1 + w / 2) / 2 - 0.0025, (hy0 + hy1) / 2,
                                                                         (zc0 + zc1) / 2)
        core = core + box(hx1 - hx0, hy1 - hy0, zb - zc0).move((hx0 + hx1) / 2, (hy0 + hy1) / 2, (zc0 + zb) / 2)
    else:
        core = box(w - 0.01, h, zc1 - zc0).move(z=(zc0 + zc1) / 2)
    out = core
    for k, (y0, y1, kind) in enumerate(y_rows):
        ya, yb = y0 + g / 2, y1 - g / 2
        if kind == 'soldier':
            spans = [(-w / 2, w / 2)]
            for a, b in spans:
                out = out + brick_course(a, b, ya, yb, -d / 2, d / 2, bl=0.075, gap=g, phase=0.0)
            continue
        ph = 0.1125 if k % 2 else 0.0
        if hole and y0 < hole[3] - 1e-6 and y1 > hole[2] + 1e-6:
            spans = [(-w / 2, hole[0]), (hole[1], w / 2)]
        else:
            spans = [(-w / 2, w / 2)]
        for a, b in spans:
            out = out + brick_course(a, b, ya, yb, -d / 2, d / 2, phase=ph - (a + w / 2))
    return out


def fp_body():
    # brick chimney-piece: running bond, a soldier course over the firebox,
    # and the firebox recess itself cut 0.2 m into the brickwork
    rows = [(-0.55, -0.502, 'run')]
    y = -0.502
    for _ in range(7):
        rows.append((y, y + 0.55 / 7, 'run'))
        y += 0.55 / 7
    rows.append((y, y + 0.1045, 'soldier'))
    y += 0.1045
    for _ in range(5):
        rows.append((y, y + 0.3975 / 5, 'run'))
        y += 0.3975 / 5
    return brick_block(0.90, 1.10, 0.30, rows, hole=(-0.25, 0.25, -0.502, 0.048, -0.049))


part('Fireplaces__body', f_box(0.90, 1.10, 0.30), fp_body, flat=True)


def fp_chimney():
    rows = []
    y = -0.5
    for _ in range(13):
        rows.append((y, y + 1.0 / 13, 'run'))
        y += 1.0 / 13
    return brick_block(0.54, 1.0, 0.30, rows)


part('Fireplaces__chimney', f_box(0.54, 1.00, 0.30), fp_chimney, flat=True)


def fp_firebox():
    # the dark firebox, open at the front: back, sides, floor, smoke shelf,
    # and a cast fireback plate with an arched top on the back wall
    t = 0.02
    back = box(0.50, 0.55, t).move(z=-0.10 + t / 2)
    sides = box(t, 0.55, 0.20).move(x=-0.25 + t / 2) + box(t, 0.55, 0.20).move(x=0.25 - t / 2)
    floor = box(0.50, 0.006, 0.20).move(y=-0.275 + 0.003)
    top = box(0.50, 0.03, 0.20).move(y=0.275 - 0.015)
    arch = [(-0.15, -0.255), (0.15, -0.255), (0.15, 0.05)]
    arch += [(0.15 * math.cos(math.pi * k / 8), 0.05 + 0.12 * math.sin(math.pi * k / 8)) for k in range(1, 8)]
    arch += [(-0.15, 0.05)]
    plate = prism(arch, 0.012, bevel_r=0.003).move(z=-0.10 + t + 0.004)
    return back + sides + floor + top + plate


part('Fireplaces__opening', f_box(0.50, 0.55, 0.20), fp_firebox)


def fp_hearth():
    # three stone slabs, eased top edges, fine joints
    out = Shape()
    w = 0.96 / 3
    for i in range(3):
        out = out + rbox(w - 0.003, 0.030, 0.14, r=0.004, seg=1).move(x=-0.48 + w * (i + 0.5))
    return noisy(out, 0.0006, scale=40, seed=4)


part('Fireplaces__hearth', f_box(0.96, 0.030, 0.14), fp_hearth)

# mantel shelf: bullnose top edge over a cove, returned round the ends
part('Fireplaces__mantel', f_box(0.96, 0.026, 0.20), lambda: yloft(
    [(-0.013, 0.012), (-0.006, 0.012), (-0.004, 0.009), (-0.001, 0.004), (0.001, 0.0012), (0.004, 0.0),
     (0.009, 0.0), (0.0115, 0.0012), (0.013, 0.004)], 0.96, 0.20, 0.006, cseg=2))

# andiron upright: splayed foot, vase, collars, neck under the finial
part('Fireplaces__post', f_cyl(0.010, 0.014, 0.18), lambda: lathe(
    [(0, -0.090), (0.0140, -0.090), (0.0140, -0.084), (0.0090, -0.078), (0.0070, -0.060), (0.0100, -0.050),
     (0.0105, -0.030), (0.0080, -0.012), (0.0060, 0.000), (0.0085, 0.006), (0.0060, 0.012), (0.0050, 0.060),
     (0.0080, 0.068), (0.0080, 0.076), (0.0055, 0.082), (0.0060, 0.090), (0, 0.090)], seg=10))

# ball finial with a collar underneath
part('Fireplaces__finial', f_sphere(0.014), lambda: lathe(
    [(0, -0.014), (0.0055, -0.014), (0.0068, -0.0125), (0.0055, -0.0108)] +
    [(0.0138 * math.cos(math.radians(a)), 0.0006 + 0.0134 * math.sin(math.radians(a)))
     for a in (-50, -25, 0, 25, 50, 72)] + [(0, 0.014)], seg=12), smooth=60)

# forged billet bar: square section, chamfered, hammered
part('Fireplaces__bar', f_cyl(0.006, 0.006, 0.13), lambda: noisy(
    rbox(0.0092, 0.13, 0.0092, r=0.0015, seg=1).rot(y=math.pi / 4), 0.0004, scale=90, seed=2))


def split_log(r, h, seed=0):
    """A firewood log along +Y: bark with long fissures, a branch stub,
    flat sawn ends (for the end-grain discs)."""
    ys = [-h / 2 + h * k / 14 for k in range(15)]
    prof = [(0, -h / 2), (r * 0.90, -h / 2)] + [(r, y) for y in ys[1:-1]] + [(r * 0.90, h / 2), (0, h / 2)]
    s = lathe(prof, seg=14)

    def bark(p):
        rr = math.hypot(p.x, p.z)
        if rr < r * 0.95 or abs(p.y) > h / 2 - 1e-6:
            return p
        a = math.atan2(p.x, p.z)
        k = 1 - 0.20 * (0.5 + 0.5 * math.cos(7 * a + 3 * noise.noise(Vector((p.y * 8, seed, 0.2))))) ** 4 \
            + 0.05 * noise.noise(Vector((p.x * 90, p.y * 25, p.z * 90 + seed))) \
            + 0.04 * noise.noise(Vector((0.3, p.y * 9, seed + 0.7)))
        p.x *= k
        p.z *= k
        return p
    s = s.warp(bark)
    stub = lathe([(0, 0), (r * 0.30, 0), (r * 0.26, r * 0.22), (0, r * 0.25)], seg=6).rot(x=-HPI * 0.8)
    return s + stub.move(y=h * 0.18, z=r * 0.88)


part('Fireplaces__log', f_cyl(0.024, 0.024, 0.275), lambda: split_log(0.024, 0.275), smooth=32)


# ============================================================ log baskets ==
def loop_rings(rings):
    """Bridge a closed loop of equal-length 3D rings (last joins first)."""
    n = len(rings[0])
    v, f = [], []
    for ring in rings:
        v += [Vector(p) for p in ring]
    R = len(rings)
    for k in range(R):
        a, b = k * n, ((k + 1) % R) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    return _fw(Shape(v, f))


def wicker_body():
    # woven wicker wall (weavers in and out of the stakes), wall thickness,
    # a rolled rim at the top; open top and bottom (the floor disc is separate)
    N, K = 24, 14
    rb, rt, H = 0.126, 0.140, 0.24
    rad = lambda y: rb + (rt - rb) * (y + H / 2) / H
    y0, y1 = -0.12, 0.104
    amp = 0.0032

    def ring(r, y, off=None):
        out = []
        for j in range(N):
            a = TAU * j / N
            rr = r + (off(j) if off else 0.0)
            out.append((rr * math.sin(a), y, rr * math.cos(a)))
        return out
    rings = []
    for k in range(K + 1):
        y = y0 + (y1 - y0) * k / K
        ro = rad(y) - 0.0045
        rings.append(ring(ro, y, lambda j, k=k: amp * (1 if (j + k) % 2 == 0 else -1) * (0.4 if k in (0, K) else 1.0)))
    ro = rad(0.11) - 0.0045
    for dr, y in ((0.0040, 0.108), (0.0045, 0.1155), (0.0005, 0.1200), (-0.0045, 0.1185), (-0.0080, 0.1120)):
        rings.append(ring(ro + dr, y))
    rings.append(ring(rad(0.104) - 0.0125, 0.104))
    rings.append(ring(rad(-0.12) - 0.0125, -0.12))
    return loop_rings(rings)


part('LogBaskets__body', f_cyl(0.14, 0.126, 0.24), wicker_body, smooth=60)

# flat iron strap hoop
part('LogBaskets__ring', f_torus(0.14, 0.003), lambda: ring_section(0.14, [
    (-0.0012, -0.0030), (0.0010, -0.0030), (0.0014, -0.0015), (0.0014, 0.0015), (0.0010, 0.0030),
    (-0.0012, 0.0030)], seg=40))

F_LB_HANDLE = torus_arc_bounds(0.026, 0.005, math.pi, radial=24)


def rope_arc(R, r, arc, seg=16, m=8, strands=3, twist=5.0):
    """A twisted-rope (or twisted-iron) half ring in the XY plane."""
    v, f = [], []
    for i in range(seg + 1):
        a = arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        for j in range(m):
            ph = TAU * j / m
            rr = r * (0.82 + 0.18 * math.cos(strands * ph - twist * a * 2))
            v.append(((R + rr * math.cos(ph)) * ca, (R + rr * math.cos(ph)) * sa, rr * math.sin(ph)))
    for i in range(seg):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, (i + 1) * m + j, (i + 1) * m + j2, i * m + j2))
    f.append(tuple(reversed(range(m))))
    f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


part('LogBaskets__handle', F_LB_HANDLE, lambda: fitf(rope_arc(0.026, 0.005, math.pi), F_LB_HANDLE), smooth=60)


# ================================================== wingback armchairs ==
def sheet(w, d, nx, nz, ftop, fbot, xz=None):
    """A closed soft slab over a w x d grid in XZ: top at ftop(x, z), bottom
    at fbot(x, z); xz(x, z) -> (x, z) optionally reshapes the outline."""
    v, f = [], []
    xs = [-w / 2 + w * i / nx for i in range(nx + 1)]
    zs = [-d / 2 + d * j / nz for j in range(nz + 1)]
    T, B = {}, {}
    for j, z in enumerate(zs):
        for i, x in enumerate(xs):
            px, pz = xz(x, z) if xz else (x, z)
            T[(i, j)] = len(v)
            v.append((px, ftop(x, z), pz))
            B[(i, j)] = len(v)
            v.append((px, fbot(x, z), pz))
    for j in range(nz):
        for i in range(nx):
            q = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)]
            f.append(tuple(T[k] for k in q))
            f.append(tuple(B[k] for k in reversed(q)))
    rim = [(i, 0) for i in range(nx)] + [(nx, j) for j in range(nz)] + \
          [(i, nz) for i in range(nx, 0, -1)] + [(0, j) for j in range(nz, 0, -1)]
    for k in range(len(rim)):
        a, b = rim[k], rim[(k + 1) % len(rim)]
        f.append((T[a], T[b], B[b], B[a]))
    return _fw(Shape(v, f))


def wa_seat():
    # loose seat cushion: piped top and bottom edges, soft bulging band,
    # crowned top
    return yloft([(-0.060, 0.030), (-0.058, 0.012), (-0.054, 0.004), (-0.049, 0.0), (-0.044, 0.004),
                  (0.000, 0.001), (0.044, 0.004), (0.049, 0.0), (0.054, 0.004), (0.057, 0.014),
                  (0.059, 0.050), (0.060, 0.120)], 0.55, 0.48, 0.045, cseg=3)


part('WingbackArmchairs__seat', f_box(0.55, 0.12, 0.48), wa_seat, smooth=60)


def wa_base():
    # upholstered base: tailored skirt with inverted pleats at the corners
    # and the side centres, a welt where the skirt meets the body
    s = yloft([(-0.180, 0.004), (-0.176, 0.0), (-0.062, 0.0), (-0.059, 0.006), (-0.055, 0.0015),
               (-0.046, 0.0015), (-0.042, 0.005), (0.150, 0.003), (0.172, 0.005), (0.180, 0.016)],
              0.57, 0.50, 0.012, cseg=2)
    cutters = []
    for x in (-0.285, 0.0, 0.285):
        for z in (-0.25, 0.25):
            if x == 0.0:
                cutters.append(box(0.006, 0.12, 0.012).move(x=0.0, y=-0.121, z=z))
            else:
                cutters.append(box(0.012, 0.12, 0.012).move(x=x, y=-0.121, z=z))
    for x in (-0.285, 0.285):
        cutters.append(box(0.012, 0.12, 0.006).move(x=x, y=-0.121, z=0.0))
    return cut(s, *cutters)


part('WingbackArmchairs__base', f_box(0.57, 0.36, 0.50), wa_base)


def wa_back():
    # tall back, diamond-buttoned on the front (+Z), rolled edges, gently
    # arched top
    W, H, D = 0.55, 0.75, 0.16
    btn = []
    for r in (3, 6, 9, 12, 15, 18):
        cols = (4, 8, 12, 16) if r % 2 else (6, 10, 14)
        for c in cols:
            btn.append((-W / 2 + W * c / 20, -H / 2 + H * r / 20))

    def front(x, y):
        ex = min(1.0, (W / 2 - abs(x)) / 0.05)
        ey = min(1.0, (H / 2 - abs(y)) / 0.05)
        e = math.sin(HPI * min(ex, ey)) ** 0.7
        z = 0.03 + 0.05 * e
        for bx, by in btn:
            d2 = ((x - bx) ** 2 + (y - by) ** 2) / 0.022 ** 2
            if d2 < 9:
                z -= 0.016 * math.exp(-d2) * e
        return z
    s = heightfield(W, H, 20, 20, lambda x, z: front(x, -z), -0.08)
    s = s.rot(x=HPI)

    def arch(p):
        if p.y > 0:
            p.y -= 0.05 * (p.x / (W / 2)) ** 2 * (p.y / (H / 2)) ** 2
        return p
    return fitf(s.warp(arch), f_box(W, H, D))


part('WingbackArmchairs__back', f_box(0.55, 0.75, 0.16), wa_back, smooth=70)


def wa_wing():
    # the wing seen from the side: full height at the back (-Z), sweeping
    # forward and down to the arm; padded edges
    out = [(-0.14, -0.281), (0.14, -0.281), (0.14, -0.06), (0.132, 0.05), (0.108, 0.14), (0.068, 0.208),
           (0.012, 0.255), (-0.06, 0.278), (-0.11, 0.281), (-0.134, 0.272), (-0.14, 0.25)]
    s = prism([(-z, y) for z, y in out], 0.11)
    s = bevel(s, 0.03, seg=2, angle=50)
    return fitf(s.rot(y=HPI), f_box(0.11, 0.5625, 0.28))


part('WingbackArmchairs__wing', f_box(0.11, 0.5625, 0.28), wa_wing, smooth=60)


def wa_arm():
    # padded arm block, rounded, with a raised scroll facing on the front
    body = rbox(0.15, 0.26, 0.43, r=0.03, seg=2).move(z=-0.005)
    face = prism(rrect_pts(0.11, 0.19, 0.03, 2), 0.012, bevel_r=0.004).move(y=-0.025, z=0.214)
    return body + face


part('WingbackArmchairs__arm', f_box(0.15, 0.26, 0.44), wa_arm, smooth=50)


def wa_roll():
    # scroll arm roll: +Y is the front end, gathered into radial pleats
    # round a covered button; plain rounded back end
    s = lathe([(0, -0.220), (0.062, -0.220), (0.072, -0.215), (0.075, -0.203), (0.075, 0.190), (0.073, 0.204),
               (0.068, 0.2135), (0.058, 0.2185), (0.042, 0.2170), (0.024, 0.2150), (0.012, 0.2140),
               (0.009, 0.2175), (0.005, 0.2200), (0, 0.2200)], seg=20)

    def pleat(p):
        r = math.hypot(p.x, p.z)
        if p.y > 0.205 and 0.013 < r < 0.066:
            a = math.atan2(p.x, p.z)
            p.y -= 0.0035 * (0.5 + 0.5 * math.cos(10 * a)) * math.sin(math.pi * (r - 0.013) / 0.053)
        return p
    return s.warp(pleat)


part('WingbackArmchairs__roll', f_cyl(0.075, 0.075, 0.44), wa_roll, smooth=60)

# turned bun foot
part('WingbackArmchairs__foot', f_cyl(0.014, 0.020, 0.030), lambda: lathe(
    [(0, -0.015), (0.0125, -0.015), (0.0180, -0.0115), (0.0200, -0.0055), (0.0190, 0.0005), (0.0150, 0.0060),
     (0.0120, 0.0085), (0.0140, 0.0105), (0.0140, 0.0150), (0, 0.0150)], seg=10))


def wa_throw_top():
    # the folded throw lying over the arm roll: arched across, soft ripples
    def mid(x, z):
        return 0.004 - 0.012 * (x / 0.065) ** 2 + 0.0015 * math.sin(z * 38 + x * 9)
    return sheet(0.13, 0.336, 6, 14, lambda x, z: mid(x, z) + 0.003, lambda x, z: mid(x, z) - 0.003)


part('WingbackArmchairs__throwTop', f_box(0.13, 0.010, 0.336), wa_throw_top, smooth=70)


def wa_throw_hang():
    # the hanging end: vertical folds deepening towards a rolled hem
    def off(y, z):
        t = (0.10 - y) / 0.20
        return (0.0015 + 0.0045 * t) * math.sin(z * TAU / 0.072 + 0.6)
    s = sheet(0.20, 0.288, 5, 18, lambda y, z: off(y, z) + 0.002, lambda y, z: off(y, z) - 0.002)
    s = s.rot(z=HPI)          # grid X -> +Y, thickness Y -> -X
    return s


part('WingbackArmchairs__throwHang', f_box(0.008, 0.20, 0.288), wa_throw_hang, smooth=70)

# fringe tassel: bound head on top, neck, flared fringe
part('WingbackArmchairs__tassel', f_box(0.008, 0.020, 0.008), lambda: flutes(lathe(
    [(0, -0.010), (0.0040, -0.0095), (0.0030, 0.0020), (0.0020, 0.0045), (0.0031, 0.0070), (0.0024, 0.0090),
     (0, 0.0100)],
    seg=8), 4, 0.0008, -0.0096, 0.0015), smooth=60)


def plump_pillow(w, h, d, nx=10, nz=8, ear=0.006, pinch=0.008):
    """Knife-edge throw pillow: plump middle, thin seam, corner ears."""
    def f(x, z):
        u, v = x / (w / 2), z / (d / 2)
        return (h / 2) * max(0.0, (1 - u * u)) ** 0.32 * max(0.0, (1 - v * v)) ** 0.32 + 0.0015

    def xz(x, z):
        u, v = x / (w / 2), z / (d / 2)
        px = x * (1 - pinch / (w / 2) * (1 - v * v) * abs(u) ** 3) + math.copysign(ear, x) * (u * v) ** 4
        pz = z * (1 - pinch / (d / 2) * (1 - u * u) * abs(v) ** 3) + math.copysign(ear, z) * (u * v) ** 4
        return px, pz
    s = sheet(w, d, nx, nz, f, lambda x, z: -f(x, z), xz)
    return fitf(s, f_box(w, h, d))


part('WingbackArmchairs__pillow', f_box(0.28, 0.06, 0.20), lambda: plump_pillow(0.28, 0.06, 0.20), smooth=70)


# ===================================================== chair arm books ==
# side stand: moulded top on turned legs with a turned stretcher
part('ChairArmBooks__top', f_box(0.34, 0.014, 0.34), lambda: yloft(
    [(-0.007, 0.007), (-0.0045, 0.0045), (-0.0020, 0.0012), (0.0000, 0.0), (0.0045, 0.0), (0.0065, 0.0015),
     (0.0070, 0.0035)], 0.34, 0.34, 0.022, cseg=3))

part('ChairArmBooks__leg', f_cyl(0.010, 0.014, 0.543), lambda: lathe(
    [(0, -0.2715), (0.0110, -0.2715), (0.0135, -0.266), (0.0140, -0.258), (0.0115, -0.250), (0.0090, -0.240),
     (0.0105, -0.200), (0.0120, -0.160), (0.0100, -0.110), (0.0080, -0.040), (0.0072, 0.080), (0.0092, 0.090),
     (0.0072, 0.100), (0.0068, 0.200), (0.0090, 0.212), (0.0100, 0.222), (0.0100, 0.2715), (0, 0.2715)],
    seg=10))

part('ChairArmBooks__brace', f_cyl(0.006, 0.006, 0.255), lambda: lathe(
    [(0, -0.1275), (0.0045, -0.1275), (0.0045, -0.112), (0.0060, -0.106), (0.0042, -0.098), (0.0036, -0.020),
     (0.0055, -0.008), (0.0060, 0.0), (0.0055, 0.008), (0.0036, 0.020), (0.0042, 0.098), (0.0060, 0.106),
     (0.0045, 0.112), (0.0045, 0.1275), (0, 0.1275)], seg=8))


def book_page(spine_sign=1):
    # open page block: the top dips into the gutter at the spine end,
    # fore-edge corners lift a little
    def h(x, z):
        u = x * spine_sign
        y = 0.0025
        if u > 0.015:
            y -= 0.0035 * ((u - 0.015) / 0.045) ** 2
        if u < -0.035:
            y += 0.0008 * ((-0.035 - u) / 0.025) * (abs(z) / 0.075) ** 3
        return y
    return heightfield(0.12, 0.15, 10, 6, h, -0.0025)


part('ChairArmBooks__pageL', f_box(0.12, 0.005, 0.15), lambda: book_page(1), smooth=40)
part('ChairArmBooks__pageR', f_box(0.12, 0.005, 0.15), lambda: book_page(-1), smooth=40)

# rounded leather spine with raised bands and headcaps
part('ChairArmBooks__spine', f_cyl(0.003, 0.003, 0.15), lambda: lathe(
    [(0, -0.075), (0.0024, -0.075), (0.0030, -0.0730), (0.0026, -0.0700), (0.0026, -0.0520),
     (0.0030, -0.0490), (0.0026, -0.0460), (0.0026, -0.0170), (0.0030, -0.0140), (0.0026, -0.0110),
     (0.0026, 0.0110), (0.0030, 0.0140), (0.0026, 0.0170), (0.0026, 0.0460), (0.0030, 0.0490),
     (0.0026, 0.0520), (0.0026, 0.0700), (0.0030, 0.0730), (0.0024, 0.075), (0, 0.075)], seg=6), smooth=50)

part('ChairArmBooks__cover', f_box(0.003, 0.007, 0.15), lambda: rbox(0.003, 0.007, 0.15, r=0.0012, seg=2))


def book_ribbon():
    # silk bookmark with a swallowtail end at +Z, lying in a soft wave
    n = 8
    right = [(0.004, -0.025 + 0.044 * i / n) for i in range(n + 1)] + [(0.004, 0.025)]
    left = [(-x, z) for x, z in reversed(right)]
    out = right + [(0.0, 0.0185)] + left
    s = prism([(x, -z) for x, z in out], 0.0008).rot(x=-HPI)
    return s.warp(lambda p: Vector((p.x, p.y + 0.0005 * math.sin(p.z * 110), p.z)))


part('ChairArmBooks__ribbon', f_box(0.008, 0.002, 0.050), book_ribbon, smooth=50)

# round wire spectacle rim (a rounded band, as tall as the frame)
part('ChairArmBooks__rim', f_torus(0.018, 0.002), lambda: ring_section(0.018, [
    (-0.0008, -0.0020), (0.0008, -0.0020), (0.0010, 0.0), (0.0008, 0.0020), (-0.0008, 0.0020),
    (-0.0010, 0.0)], seg=22), smooth=60)

part('ChairArmBooks__bridge', f_box(0.012, 0.002, 0.003), lambda: fitf(tube(
    [(-0.006, 0, -0.0012), (-0.003, 0, 0.0008), (0.0, 0, 0.0012), (0.003, 0, 0.0008), (0.006, 0, -0.0012)],
    0.0008, seg=5, smooth=1), f_box(0.012, 0.002, 0.003)), smooth=60)

# temple wire with a ball tip at each end
part('ChairArmBooks__temple', f_cyl(0.001, 0.001, 0.038), lambda: lathe(
    [(0, -0.019), (0.0008, -0.0186), (0.0010, -0.0178), (0.0006, -0.0168), (0.0006, 0.0168), (0.0010, 0.0178),
     (0.0008, 0.0186), (0, 0.019)], seg=6), smooth=60)


# =============================================== silhouette portraits ==
def sp_frame():
    # moulded frame: flat back on the wall, rounded outer edge, a bead that
    # stands proud of the gilt mat, small rosettes on the corners
    s = zloft([(-0.007, 0.0), (0.004, 0.0), (0.0075, 0.0015), (0.0095, 0.0030), (0.0085, 0.0045),
               (0.0070, 0.0052)], 0.18, 0.22, 0.004, cseg=2)
    ros = lathe([(0, 0.0), (0.0042, 0.0), (0.0040, 0.0010), (0.0026, 0.0022), (0, 0.0028)], seg=8)
    ros = flutes(ros, 4, 0.0012, 0.0, 0.0012, rmin=0.003).rot(x=HPI)
    for sx in (-1, 1):
        for sy in (-1, 1):
            s = s + ros.copy().move(x=sx * 0.0865, y=sy * 0.1065, z=0.0082)
    return s


part('SilhouettePortraits__frame', f_box(0.18, 0.22, 0.014), sp_frame)

# gilt slip: flat margin and a raised bead round the sight edge
part('SilhouettePortraits__mat', f_box(0.17, 0.21, 0.002), lambda: zloft(
    [(-0.001, 0.0), (0.0010, 0.0), (0.0012, 0.0010), (0.0012, 0.0040), (0.0025, 0.0055), (0.0015, 0.0068),
     (0.0012, 0.0072)], 0.17, 0.21, 0.003, cseg=1))


# ============================================================= ottomans ==
def ot_drum():
    # round deep-buttoned ottoman top: dimples at the five buttons, crowned
    # top, piped edge, softly bulging band
    N = 24
    btn = [(0.0, 0.0), (0.132, 0.0), (-0.132, 0.0), (0.0, 0.132), (0.0, -0.132)]

    def top_y(x, z, r):
        y = 0.086 - 0.010 * (r / 0.23) ** 2
        for bx, bz in btn:
            d2 = ((x - bx) ** 2 + (z - bz) ** 2) / 0.024 ** 2
            y -= 0.020 * math.exp(-d2)
        return y
    rings = []
    for r, y in ((0.200, -0.090), (0.222, -0.088), (0.229, -0.082), (0.232, -0.074), (0.228, -0.069),
                 (0.237, -0.010), (0.238, 0.062), (0.236, 0.067), (0.240, 0.073), (0.236, 0.080)):
        rings.append([(r * math.cos(TAU * j / N), y, r * math.sin(TAU * j / N)) for j in range(N)])
    for r in (0.225, 0.205, 0.175, 0.150, 0.132, 0.114, 0.085, 0.055, 0.030, 0.014):
        ring = []
        for j in range(N):
            x, z = r * math.cos(TAU * j / N), r * math.sin(TAU * j / N)
            ring.append((x, top_y(x, z, r), z))
        rings.append(ring)
    return fitf(loft(rings), f_cyl(0.24, 0.23, 0.18))


part('Ottomans__drum', f_cyl(0.24, 0.23, 0.18), ot_drum, smooth=70)


def box_pleats(s, seg, every, depth, y0, y1):
    """Pull every `every`-th vertex ring column of a lathe in by depth
    between y0 and y1: inverted box pleats round a skirt."""
    step = TAU / seg

    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6 or not (y0 - 1e-7 <= p.y <= y1 + 1e-7):
            return p
        i = int(round(math.atan2(p.x, p.z) / step)) % seg
        if i % every == 0:
            k = (r - depth) / r
            p.x *= k
            p.z *= k
        return p
    return s.warp(w)


part('Ottomans__drumSkirt', f_cyl(0.235, 0.235, 0.14), lambda: box_pleats(lathe(
    [(0, -0.070), (0.2300, -0.070), (0.2350, -0.066), (0.2350, 0.050), (0.2320, 0.054), (0.2350, 0.058),
     (0.2340, 0.065), (0.2280, 0.070), (0, 0.070)], seg=48), 48, 4, 0.008, -0.0665, 0.0505), smooth=50)

part('Ottomans__bunFoot', f_cyl(0.014, 0.020, 0.020), lambda: lathe(
    [(0, -0.010), (0.0140, -0.010), (0.0188, -0.0070), (0.0200, -0.0025), (0.0180, 0.0025), (0.0140, 0.0055),
     (0.0122, 0.0072), (0.0140, 0.0085), (0.0140, 0.0100), (0, 0.0100)], seg=10))

# square footstool: stuffed, crowned top with piped edges
part('Ottomans__cushion', f_box(0.38, 0.14, 0.30), lambda: yloft(
    [(-0.070, 0.006), (-0.066, 0.0), (-0.062, 0.004), (0.040, 0.002), (0.051, 0.006), (0.055, 0.0),
     (0.059, 0.006), (0.065, 0.028), (0.069, 0.070), (0.070, 0.110)], 0.38, 0.30, 0.030, cseg=3), smooth=60)


def ot_apron():
    # upholstered rail with a short tailored skirt, inverted pleats at the
    # corners and side centres
    s = yloft([(-0.050, 0.003), (-0.047, 0.0), (0.038, 0.0), (0.041, 0.004), (0.045, 0.0005), (0.048, 0.0005),
               (0.050, 0.006)], 0.372, 0.292, 0.010, cseg=2)
    cutters = []
    for x, z, wx, wz in ((0.186, 0.146, 0.012, 0.012), (-0.186, 0.146, 0.012, 0.012), (0.186, -0.146, 0.012, 0.012),
                         (-0.186, -0.146, 0.012, 0.012), (0.0, 0.146, 0.006, 0.012), (0.0, -0.146, 0.006, 0.012),
                         (0.186, 0.0, 0.012, 0.006), (-0.186, 0.0, 0.012, 0.006)):
        cutters.append(box(wx, 0.09, wz).move(x=x, y=-0.051 + 0.045, z=z))
    return cut(s, *cutters)


part('Ottomans__apron', f_box(0.372, 0.10, 0.292), ot_apron)

part('Ottomans__leg', f_cyl(0.010, 0.014, 0.14), lambda: lathe(
    [(0, -0.070), (0.0100, -0.070), (0.0135, -0.066), (0.0140, -0.060), (0.0120, -0.054), (0.0090, -0.046),
     (0.0105, -0.030), (0.0110, -0.010), (0.0082, 0.020), (0.0072, 0.040), (0.0092, 0.048), (0.0072, 0.055),
     (0.0100, 0.060), (0.0100, 0.070), (0, 0.070)], seg=10))

part('Ottomans__pillow', f_box(0.16, 0.04, 0.16), lambda: plump_pillow(0.16, 0.04, 0.16, nx=8, nz=8,
                                                                      ear=0.004, pinch=0.006), smooth=70)


# ============================================================= humidors ==
# cabinet-made box: moulded plinth, cove, inlaid stringing line near the top
part('Humidors__box', f_box(0.20, 0.055, 0.14), lambda: yloft(
    [(-0.0275, 0.0012), (-0.0262, 0.0), (-0.0215, 0.0), (-0.0200, 0.0014), (-0.0185, 0.0024),
     (0.0100, 0.0024), (0.0104, 0.0032), (0.0116, 0.0032), (0.0120, 0.0024), (0.0275, 0.0024)],
    0.20, 0.14, 0.004, cseg=2))

# gilt trim band: an open rectangle round the lining / round the lid
part('Humidors__trim', f_box(0.203, 0.003, 0.143), lambda: frame_ring(0.203, 0.143, [
    (0.0, -0.0015), (0.0, 0.0008), (0.0008, 0.0015), (0.0060, 0.0015), (0.0065, 0.0008), (0.0065, -0.0015)],
    0.0045, cseg=2))

# velvet lining pad with a soft rolled edge
part('Humidors__lining', f_box(0.19, 0.003, 0.13), lambda: yloft(
    [(-0.0015, 0.0), (0.0004, 0.0), (0.0012, 0.0015), (0.0015, 0.0060)], 0.19, 0.13, 0.004, cseg=2),
     smooth=60)


def hm_lid():
    # lid: raised and bevelled top panel, a recess underneath
    s = yloft([(-0.0096, 0.0012), (-0.0086, 0.0), (0.0040, 0.0), (0.0052, 0.0012), (0.0064, 0.0014),
               (0.0080, 0.0070), (0.0096, 0.0120)], 0.20, 0.14, 0.004, cseg=2)
    return cut(s, box(0.182, 0.012, 0.122).move(y=-0.0115))


part('Humidors__lid', f_box(0.20, 0.01925, 0.14), hm_lid)


def hm_hinge():
    # piano hinge barrel: knuckles with fine joints, pin heads at the ends
    prof = [(0, -0.085), (0.0024, -0.085), (0.0030, -0.0835)]
    n = 10
    for k in range(1, n):
        y = -0.085 + 0.17 * k / n
        prof += [(0.0030, y - 0.0012), (0.0024, y), (0.0030, y + 0.0012)]
    prof += [(0.0030, 0.0835), (0.0024, 0.085), (0, 0.085)]
    return lathe(prof, seg=6)


part('Humidors__hinge', f_cyl(0.003, 0.003, 0.17), hm_hinge, smooth=50)


def hm_clasp():
    # brass hasp plate with a rounded drop and a domed pin
    out = [(-0.005, 0.0025), (0.005, 0.0025), (0.005, 0.0), (0.0035, -0.0018), (0.0012, -0.0025),
           (-0.0012, -0.0025), (-0.0035, -0.0018), (-0.005, 0.0)]
    plate = prism(out, 0.0016).move(z=-0.0007)
    pin = lathe([(0, 0.0), (0.0012, 0.0), (0.0010, 0.0008), (0, 0.0012)], seg=6).rot(x=HPI).move(y=-0.0002, z=0.0002)
    return plate + pin


part('Humidors__clasp', f_box(0.010, 0.005, 0.003), hm_clasp)


def hm_cigar():
    # -Y: the head, tapered to a rounded cap with the cap seam; +Y: the cut
    # foot. A spiral wrapper seam runs up the body.
    prof = [(0, -0.0625), (0.0034, -0.0621), (0.0058, -0.0598), (0.0069, -0.0560), (0.0071, -0.0515),
            (0.0068, -0.0505), (0.0072, -0.0495)]
    prof += [(0.0072 + 0.0008 * (k / 10), -0.045 + 0.0105 * k) for k in range(11)]
    prof += [(0.0080, 0.0615), (0.0074, 0.0625), (0.0040, 0.0620), (0, 0.0618)]
    s = lathe(prof, seg=12)

    def seam(p):
        r = math.hypot(p.x, p.z)
        if r < 0.006 or not (-0.049 < p.y < 0.060):
            return p
        u = (math.atan2(p.x, p.z) / TAU + p.y / 0.034) % 1.0
        k = 1 - 0.035 * u
        p.x *= k
        p.z *= k
        return p
    return s.warp(seam)


part('Humidors__cigar', f_cyl(0.008, 0.0072, 0.125), hm_cigar, smooth=60)

# paper band: a thin ring round the cigar
part('Humidors__band', f_cyl(0.0084, 0.0084, 0.010), lambda: ring_section(0.0081, [
    (-0.0003, -0.005), (0.0003, -0.005), (0.0003, 0.005), (-0.0003, 0.005)], seg=12).rot(x=HPI), smooth=40)

# round punch cutter: chamfered disc with a through-hole
part('Humidors__cutter', f_cyl(0.020, 0.020, 0.006), lambda: ring_section(0.014, [
    (-0.0060, -0.0020), (-0.0052, -0.0030), (0.0054, -0.0030), (0.0060, -0.0022), (0.0060, 0.0022),
    (0.0054, 0.0030), (-0.0052, 0.0030), (-0.0060, 0.0020)], seg=16).rot(x=HPI))

# matchbook: cover folded round the back (-X), flap tucked at the front
part('Humidors__matchbook', f_box(0.030, 0.006, 0.024), lambda: prism(
    [(-0.0135, -0.003), (0.0145, -0.003), (0.0150, -0.0025), (0.0150, 0.0018), (0.0125, 0.0023),
     (0.0110, 0.0030), (-0.0135, 0.0030), (-0.0147, 0.0022), (-0.0150, 0.0), (-0.0147, -0.0022)], 0.024))


# ======================================================= brandy snifters ==
part('BrandySnifters__coaster', f_cyl(0.055, 0.055, 0.005), lambda: lathe(
    [(0, -0.0025), (0.0530, -0.0025), (0.0550, -0.0012), (0.0550, 0.0010), (0.0538, 0.0025),
     (0.0490, 0.0025), (0.0480, 0.0017), (0.0460, 0.0017), (0.0450, 0.0023), (0.0400, 0.0019),
     (0, 0.0019)], seg=28))

# leather coaster: rounded edge, stitched groove round the border
part('BrandySnifters__leatherCoaster', f_box(0.100, 0.005, 0.100), lambda: yloft(
    [(-0.0025, 0.0003), (0.0014, 0.0), (0.0025, 0.0012), (0.0025, 0.0050), (0.0018, 0.0056), (0.0018, 0.0066),
     (0.0025, 0.0072)], 0.100, 0.100, 0.008, cseg=3))

# raised half-round lip; flat underside at +Z (the ring is laid flat)
part('BrandySnifters__coasterRim', f_torus(0.052, 0.003), lambda: ring_section(0.052, [
    (-0.0030, 0.0030), (0.0030, 0.0030), (0.0026, 0.0004), (0.0013, -0.0022), (0.0, -0.0030),
    (-0.0015, -0.0024), (-0.0028, 0.0002)], seg=32), smooth=50)
