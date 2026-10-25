"""b14 — bathroom clutter and the first toys.

Bathroom: bobby pins, cotton pads, makeup brushes, hair claws, tweezers, nail
files, smudged wall mirror, hand mirror, bird bath, eyeliner pencils, makeup
sponges, nail polish, perfume atomiser, hairbrush, vanity mirror, hairdryer,
pill bottle.
Toys: plush bear, rubber ball, crayons, children's shoes, marbles, building
blocks, toy car, doll, kite tail, pinwheel, yo-yo, toy drum, pacifier,
skipping rope, picture book, tambourine, maracas.
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
    """A shape built along +Y, laid along +X (-Y end goes to -X)."""
    return s.rot(z=-math.pi / 2)


def ax_z(s):
    """A shape built along +Y, laid along +Z."""
    return s.rot(x=math.pi / 2)


def flat(outline, t, bevel_r=0.0, seg=1):
    """Extrude a top-view outline [(x, z), ...] t thick along Y, centred."""
    return prism([(x, -z) for x, z in outline], t, bevel_r, seg).rot(x=-math.pi / 2)


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


def sweep(points, sec, scale=None, up=(0, 0, 1), caps=True):
    """Sweep a closed 2D section [(a, b), ...] along a 3D polyline
    (parallel-transport frame). scale(t) -> k or (ka, kb)."""
    pts = [Vector(p) for p in points]
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
        ka, kb = k if isinstance(k, tuple) else (k, k)
        for (a, c) in sec:
            v.append(pts[i] + nrm * (a * ka) + b * (c * kb))
    for i in range(n - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, (i + 1) * m + j2, (i + 1) * m + j))
    if caps:
        f.append(tuple(reversed(range(m))))
        f.append(tuple((n - 1) * m + j for j in range(m)))
    return _fw(Shape(v, f))


def ring_section(R, sec, seg=32, arc=TAU):
    """Sweep a closed section [(dr, dz), ...] around the Z axis in the XY
    plane (a TorusGeometry with any cross-section)."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        for dr, dz in sec:
            v.append(((R + dr) * ca, (R + dr) * sa, dz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


def torus_arc_bounds(R, r, arc):
    """Bounds of THREE.TorusGeometry(R, r, .., .., arc)."""
    xs, ys = [], []
    for i in range(361):
        u = arc * i / 360
        for j in range(24):
            w = TAU * j / 24
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r))


def rib_warp(n, amt, y0, y1):
    """Vertical grip ribs: radius * (1 - amt) in n grooves between y0..y1."""
    def w(p):
        if y0 <= p.y <= y1:
            k = 1 - amt * (0.5 + 0.5 * math.cos(n * math.atan2(p.x, p.z)))
            p.x *= k
            p.z *= k
        return p
    return w


# ============================================================ bobby pins ==
# Local +Y of a prong is the closed (bent) end, -Y the open end with the
# plastic ball tip. The wave of the top prong lies in the pin's plane (Z).
F_BP_PRONG = f_cyl(0.0007, 0.0007, 0.024)
part('BobbyPins__prong', F_BP_PRONG, lambda: fitf(lathe(
    [(0, -0.012), (0.0006, -0.0119), (0.0009, -0.0112), (0.0008, -0.0104), (0.00058, -0.0099),
     (0.00058, 0.0117), (0, 0.012)], seg=6), F_BP_PRONG), smooth=60)


def bobby_wavy():
    pts = [(0, -0.012, 0), (0, -0.0112, 0), (0, -0.0098, 0)]
    for i in range(9):
        y = -0.008 + 0.016 * i / 8
        pts.append((0, y, 0.00045 * math.sin(i * math.pi / 2)))
    pts.append((0, 0.012, 0))
    def sc(t):
        return 1.45 if t < 0.1 else 1.0
    return fitf(sweep(pts, circ(0.00058, 4, phase=math.pi / 4), scale=sc, up=(1, 0, 0)), F_BP_PRONG)

part('BobbyPins__wavy', F_BP_PRONG, bobby_wavy, smooth=60)

F_BP_BEND = torus_arc_bounds(0.002, 0.0007, math.pi)
part('BobbyPins__bend', F_BP_BEND, lambda: fitf(torus(0.002, 0.0006, seg=8, tseg=5, arc=math.pi), F_BP_BEND),
     smooth=60)


# =========================================================== cotton pads ==
def cotton_pad():
    # puffy disc, pressed seam ring near the edge, soft fibrous rim
    s = lathe([(0, -0.0013), (0.010, -0.0014), (0.0142, -0.0010), (0.0156, -0.0012), (0.0171, -0.0008),
               (0.018, 0.0), (0.0171, 0.0008), (0.0156, 0.0012), (0.0142, 0.0010), (0.010, 0.0014),
               (0, 0.0013)], seg=14)
    return noisy(s, 0.00025, scale=260, seed=3)

part('CottonPads__pad', f_cyl(0.018, 0.018, 0.003), cotton_pad, smooth=180)


# ======================================================== makeup brushes ==
# handle: -Y end (r 0.008) meets the ferrule, +Y end is the tail
part('MakeupBrushes__handle', f_cyl(0.005, 0.008, 0.085), lambda: lathe(
    [(0, -0.0425), (0.0072, -0.0425), (0.0078, -0.0405), (0.0080, -0.034), (0.0072, -0.020),
     (0.0062, 0.0), (0.0055, 0.020), (0.0050, 0.032), (0.0046, 0.038), (0.0034, 0.0415), (0, 0.0425)],
    seg=12), smooth=50)

# crimped metal ferrule: rolled edge and crimp groove at the handle end (+Y)
part('MakeupBrushes__ferrule', f_cyl(0.009, 0.009, 0.014), lambda: lathe(
    [(0, -0.007), (0.0087, -0.007), (0.009, -0.0058), (0.0084, 0.0035), (0.0077, 0.0045),
     (0.0083, 0.0055), (0.0077, 0.007), (0, 0.007)], seg=8), smooth=50)


def powder_dome():
    # pinched where it leaves the ferrule (-X), full round dome at the tip
    s = lathe([(0, -0.011), (0.0062, -0.011), (0.0078, -0.0085), (0.0098, -0.004), (0.011, 0.0005),
               (0.0104, 0.0048), (0.0085, 0.0078), (0.0055, 0.0098), (0.0022, 0.0108), (0, 0.011)], seg=14)
    s = noisy(s, 0.00035, scale=330, seed=5)
    return fitf(ax_x(s), f_sphere(0.011))

part('MakeupBrushes__dome', f_sphere(0.011), powder_dome, smooth=180)


def contour_tuft():
    # tapered bullet tuft with a swept, slightly angled tip
    s = lathe([(0, -0.012), (0.0078, -0.012), (0.0092, -0.009), (0.010, -0.004), (0.0096, 0.001),
               (0.0078, 0.005), (0.0050, 0.0085), (0.0022, 0.011), (0, 0.012)], seg=14)
    s.warp(lambda p: Vector((p.x - 0.004 * max(0.0, (p.y + 0.004) / 0.016) ** 2, p.y, p.z)))
    s = noisy(s, 0.0003, scale=330, seed=6)
    return fitf(s, f_cone(0.010, 0.024))

part('MakeupBrushes__cone', f_cone(0.010, 0.024), contour_tuft, smooth=180)


# ============================================================ hair claws ==
def claw_jaw(w, h, d, nf, flip):
    """One comb of a claw clip seen from the side: hinge at the back (-X),
    the shell sags into a U and its fingers rise at the front (+X); slots
    between the fingers. flip mirrors it for the upper jaw."""
    t = 0.0028
    n = 8
    yb, yt = -h / 2 + t / 2, h / 2 - t / 2
    cl = []
    for i in range(n + 1):
        u = i / n
        g = ((u - 0.4) / 0.4) ** 2 if u < 0.4 else 0.85 * ((u - 0.4) / 0.6) ** 2.2
        cl.append(Vector((-w / 2 + t / 2 + (w - t) * u, yb + (yt - yb) * g)))
    up, lo = [], []
    for i, p in enumerate(cl):
        tg = (cl[min(i + 1, n)] - cl[max(i - 1, 0)]).normalized()
        nm = Vector((-tg.y, tg.x))
        up.append(tuple(p + nm * (t / 2)))
        lo.append(tuple(p - nm * (t / 2)))
    s = prism(up + lo[::-1], d)
    sw = 0.003
    fw = (d - (nf - 1) * sw) / nf
    cutters = [box(w * 0.62, h * 2, sw).move(x=w / 2 - w * 0.31 + 0.002, z=-d / 2 + fw * (k + 1) + sw * (k + 0.5))
               for k in range(nf - 1)]
    s = cut(s, *cutters)
    if flip:
        s.scale(1, -1, 1)
    return fitf(s, f_box(w, h, d))

part('HairClaws__jaw', f_box(0.048, 0.012, 0.024), lambda: claw_jaw(0.048, 0.012, 0.024, 4, False))
part('HairClaws__topJaw', f_box(0.044, 0.010, 0.022), lambda: claw_jaw(0.044, 0.010, 0.022, 5, True))
part('HairClaws__tooth', f_box(0.003, 0.004, 0.010), lambda: taper(
    rbox(0.003, 0.004, 0.010, r=0.0009, seg=1), top=0.7, along='z', axes='x'))
# hinge rivet: domed heads at both ends, spring barrel in the middle
part('HairClaws__hinge', f_cyl(0.003, 0.003, 0.024), lambda: lathe(
    [(0, -0.012), (0.0026, -0.0115), (0.003, -0.0095), (0.0019, -0.0088), (0.0026, 0.0),
     (0.0019, 0.0088), (0.003, 0.0095), (0.0026, 0.0115), (0, 0.012)], seg=7), smooth=50)


# ============================================================== tweezers ==
def tweezer_arm():
    # flat spring-steel leg: wide grip at the back, tapering to a slant tip,
    # four grip ridges across the top
    ol = [(-0.0275, -0.002), (0.010, -0.002), (0.020, -0.0011), (0.0275, -0.0007),
          (0.0262, 0.0008), (0.020, 0.0011), (0.010, 0.002), (-0.0275, 0.002)]
    s = flat(ol, 0.002, bevel_r=0.0003)
    grooves = [box(0.0006, 0.001, 0.01).move(x=-0.012 + 0.0032 * k, y=0.0012) for k in range(4)]
    return fitf(cut(s, *grooves), f_box(0.055, 0.002, 0.004))

part('Tweezers__arm', f_box(0.055, 0.002, 0.004), tweezer_arm)
# the joined back end, rounded off
part('Tweezers__base', f_box(0.005, 0.003, 0.008), lambda: fitf(flat(
    [(0.0025, -0.004), (0.0025, 0.004), (-0.0006, 0.004), (-0.0019, 0.0034), (-0.0025, 0.0019),
     (-0.0025, -0.0019), (-0.0019, -0.0034), (-0.0006, -0.004)],
    0.003, bevel_r=0.0004), f_box(0.005, 0.003, 0.008)))


# ============================================================ nail files ==
part('NailFiles__board', f_box(0.100, 0.003, 0.014), lambda: fitf(flat(
    rounded_rect(0.100, 0.014, 0.0069, 5), 0.003, bevel_r=0.0006), f_box(0.100, 0.003, 0.014)))
part('NailFiles__grit', f_box(0.088, 0.001, 0.012), lambda: fitf(flat(
    rounded_rect(0.088, 0.012, 0.0059, 5), 0.001, bevel_r=0.0003), f_box(0.088, 0.001, 0.012)))


# ======================================================== smudge mirror ==
def rect_loft(w, h, prof, ky=1.0):
    """Moulded rectangular frame: prof = [(inset, z), ...] from the back
    outer edge over the moulding to the centre panel (mitred corners)."""
    rings = []
    for i, z in prof:
        ix, iy = i, i * ky
        rings.append([(-w / 2 + ix, -h / 2 + iy, z), (w / 2 - ix, -h / 2 + iy, z),
                      (w / 2 - ix, h / 2 - iy, z), (-w / 2 + ix, h / 2 - iy, z)])
    return loft(rings)

# back flat on the wall, outer round, bead, cove, flat lip, then the panel
# the smudged glass lies on (glass plane at z = +0.005)
part('SmudgeMirrors__frame', f_box(0.42, 0.55, 0.008), lambda: rect_loft(0.42, 0.55, [
    (0.0, -0.004), (0.0, 0.0006), (0.0008, 0.0024), (0.0025, 0.0036), (0.005, 0.004), (0.0075, 0.0037),
    (0.0092, 0.0027), (0.0105, 0.0019), (0.0125, 0.0021), (0.0145, 0.0031), (0.0165, 0.004),
    (0.0192, 0.004), (0.0198, 0.0035)], ky=1.25))


# =========================================================== hand mirror ==
# round back with a bevelled bezel; the glass disc lies 0.5 mm above it
part('HandMirrors__frame', f_cyl(0.030, 0.030, 0.005), lambda: lathe(
    [(0, -0.0025), (0.025, -0.0025), (0.0296, -0.0014), (0.030, 0.0005), (0.0296, 0.0019),
     (0.0285, 0.0025), (0.0272, 0.0025), (0.0262, 0.0016), (0.0255, 0.0012), (0, 0.0012)], seg=16))


def hand_mirror_handle():
    # flat neck where it leaves the frame (-Z), slim waist, fuller grip; the
    # underside stays flat on the floor
    secs = [(-0.035, 0.0080, 0.0040), (-0.028, 0.0068, 0.0042), (-0.015, 0.0058, 0.0048),
            (0.0, 0.0062, 0.0055), (0.020, 0.0072, 0.0060), (0.032, 0.0070, 0.0058), (0.035, 0.0058, 0.0048)]
    rings = []
    for z, w, h in secs:
        yc = -0.003 + h / 2
        ring = []
        for j in range(10):
            a = TAU * j / 10
            c, s = math.cos(a), math.sin(a)
            ring.append((w / 2 * math.copysign(abs(c) ** 0.6, c), yc + h / 2 * math.copysign(abs(s) ** 0.6, s), z))
        rings.append(ring)
    return fitf(loft(rings), f_box(0.008, 0.006, 0.070))

part('HandMirrors__handle', f_box(0.008, 0.006, 0.070), hand_mirror_handle, smooth=60)
# turned finial along the handle axis: collar towards the handle, ball end
part('HandMirrors__knob', f_sphere(0.008), lambda: fitf(ax_z(lathe(
    [(0.0048, -0.008), (0.0044, -0.0055), (0.0072, -0.0022), (0.008, 0.001), (0.0066, 0.0048),
     (0.0036, 0.0074), (0, 0.008)], seg=8)), f_sphere(0.008)), smooth=60)


# ============================================================ bird baths ==
# cast stone: stepped plinth, turned baluster, dished bowl with a rolled rim
part('BirdBaths__base', f_cyl(0.10, 0.12, 0.020), lambda: noisy(lathe(
    [(0, -0.01), (0.1185, -0.01), (0.12, -0.0088), (0.12, -0.0045), (0.1175, -0.0028), (0.110, -0.0022),
     (0.108, -0.0005), (0.1045, 0.0035), (0.1015, 0.0085), (0.099, 0.01), (0.056, 0.01), (0, 0.01)],
    seg=24), 0.0007, scale=45, seed=11))

part('BirdBaths__pedestal', f_cyl(0.045, 0.055, 0.32), lambda: noisy(lathe(
    [(0, -0.16), (0.055, -0.16), (0.055, -0.150), (0.051, -0.147), (0.048, -0.140), (0.050, -0.133),
     (0.048, -0.127), (0.042, -0.118), (0.040, -0.08), (0.037, -0.02), (0.034, 0.05), (0.032, 0.10),
     (0.033, 0.118), (0.038, 0.124), (0.036, 0.130), (0.040, 0.138), (0.045, 0.148), (0.045, 0.16),
     (0, 0.16)], seg=14), 0.0008, scale=35, seed=12))


def bird_bowl():
    # the transparent water disc (r 0.128 at y +0.013) meets the inner rim
    s = lathe([(0, -0.015), (0.108, -0.015), (0.11, -0.0135), (0.126, -0.004), (0.136, 0.005),
               (0.1395, 0.0105), (0.14, 0.0135), (0.1375, 0.015), (0.1315, 0.015), (0.1275, 0.0128),
               (0.118, 0.004), (0.09, -0.003), (0.05, -0.006), (0, -0.0065)], seg=22,
              cap_bottom=False, cap_top=False)
    return fitf(noisy(s, 0.0007, scale=40, seed=13), f_cyl(0.14, 0.11, 0.030))

part('BirdBaths__bowl', f_cyl(0.14, 0.11, 0.030), bird_bowl)


# ====================================================== eyeliner pencils ==
# body: +Y end under the cap, -Y end at the sharpened tip; foil band grooves
part('PencilEyeliners__body', f_cyl(0.004, 0.004, 0.075), lambda: lathe(
    [(0, -0.0375), (0.0036, -0.0375), (0.004, -0.0369), (0.004, 0.025), (0.0037, 0.0256), (0.0037, 0.0264),
     (0.004, 0.027), (0.004, 0.0368), (0.0036, 0.0375), (0, 0.0375)], seg=12))
# sharpened cone and a soft rounded kohl point
part('PencilEyeliners__tip', f_cone(0.004, 0.014), lambda: lathe(
    [(0, -0.007), (0.004, -0.007), (0.0037, -0.004), (0.0016, 0.0025), (0.0014, 0.0045),
     (0.0009, 0.0065), (0, 0.007)], seg=10), smooth=50)
# cap: open end towards the body (-Y), rounded closed end (+Y)
part('PencilEyeliners__cap', f_cyl(0.005, 0.005, 0.020), lambda: lathe(
    [(0, -0.01), (0.0045, -0.01), (0.005, -0.009), (0.005, 0.0075), (0.0046, 0.0095), (0.003, 0.01),
     (0, 0.01)], seg=10), smooth=50)


# ======================================================== makeup sponges ==
# teardrop blender sponge: full round bottom narrowing into the tip cone
part('MakeupSponges__body', f_sphere(0.030), lambda: fitf(noisy(lathe(
    [(0, -0.03), (0.012, -0.027), (0.024, -0.02), (0.0295, -0.01), (0.030, 0.0), (0.027, 0.01),
     (0.021, 0.018), (0.0145, 0.024), (0.008, 0.028), (0, 0.030)], seg=16), 0.0004, scale=150, seed=4),
    f_sphere(0.030)), smooth=180)
part('MakeupSponges__tip', f_cone(0.014, 0.024), lambda: fitf(noisy(lathe(
    [(0, -0.012), (0.014, -0.012), (0.0135, -0.008), (0.0118, -0.003), (0.009, 0.003), (0.0055, 0.008),
     (0.0025, 0.011), (0, 0.012)], seg=14), 0.0003, scale=150, seed=5), f_cone(0.014, 0.024)), smooth=180)


# ========================================================== nail polish ==
# black collar with a thread under the cap, tall ribbed cap with a soft top
part('NailPolishes__neck', f_cyl(0.008, 0.010, 0.008), lambda: lathe(
    [(0.010, -0.004), (0.0095, -0.0025), (0.0082, -0.0005), (0.0086, 0.0015), (0.0079, 0.004)], seg=10))
part('NailPolishes__cap', f_cyl(0.012, 0.012, 0.036), lambda: lathe(
    [(0, -0.018), (0.0115, -0.018), (0.012, -0.0172), (0.012, 0.0155), (0.0114, 0.0175), (0.0098, 0.018),
     (0, 0.018)], seg=20).warp(rib_warp(10, 0.06, -0.0173, 0.0156)), smooth=50)


# ============================================================== perfumes ==
# crimped atomiser collar (gold)
part('Perfumes__neck', f_cyl(0.008, 0.010, 0.016), lambda: lathe(
    [(0.0098, -0.008), (0.010, -0.005), (0.0088, -0.0035), (0.0085, 0.005), (0.0078, 0.008)], seg=10))
# flat metal band ring
part('Perfumes__ring', f_torus(0.010, 0.002), lambda: ring_section(
    0.010, [(0.002, -0.0011), (0.0013, -0.002), (-0.0013, -0.002), (-0.002, -0.0011), (-0.002, 0.0011),
            (-0.0013, 0.002), (0.0013, 0.002), (0.002, 0.0011)], seg=15))


def perfume_bulb():
    # rubber squeeze bulb: pear with a necked nozzle towards the tube (-X)
    s = lathe([(0, -0.020), (0.0042, -0.020), (0.004, -0.015), (0.0055, -0.013), (0.010, -0.009),
               (0.016, -0.004), (0.0195, 0.002), (0.020, 0.006), (0.0185, 0.011), (0.014, 0.016),
               (0.0075, 0.0192), (0, 0.020)], seg=12)
    return fitf(ax_x(s), f_sphere(0.020))

part('Perfumes__bulb', f_sphere(0.020), perfume_bulb, smooth=70)


def braided_tube():
    prof = [(0, -0.015)]
    for i in range(9):
        y = -0.015 + 0.00333 * i
        prof += [(0.0027, y + 0.0002), (0.003, y + 0.0017)]
    prof += [(0.0027, 0.0149), (0, 0.015)]
    return lathe(prof, seg=7)

part('Perfumes__tube', f_cyl(0.003, 0.003, 0.030), braided_tube, smooth=60)
part('Perfumes__cord', f_cyl(0.001, 0.001, 0.006), lambda: lathe(
    [(0, -0.003), (0.0008, -0.003), (0.001, -0.0015), (0.0008, 0), (0.001, 0.0015), (0.0008, 0.003),
     (0, 0.003)], seg=5), smooth=60)
# tassel: bound head on top, fringe skirt below
part('Perfumes__tassel', f_sphere(0.003), lambda: fitf(lathe(
    [(0, -0.003), (0.0026, -0.003), (0.0028, -0.0024), (0.0017, 0.0), (0.0010, 0.0006),
     (0.0014, 0.0016), (0.0010, 0.0027), (0, 0.003)], seg=8).warp(rib_warp(4, 0.25, -0.0031, -0.0002)),
    f_sphere(0.003)), smooth=60)


# ============================================================ hairbrushes ==
# paddle head: convex back shell, raised rim, domed rubber cushion (+Y)
part('Hairbrushes__pad', f_cyl(0.045, 0.045, 0.014), lambda: lathe(
    [(0, -0.007), (0.030, -0.0068), (0.040, -0.0058), (0.0445, -0.004), (0.045, -0.001), (0.0445, 0.003),
     (0.043, 0.0045), (0.0405, 0.0045), (0.0395, 0.003), (0.037, 0.0045), (0.030, 0.0062), (0.015, 0.0069),
     (0, 0.007)], seg=22))
# handle: flared neck into the head (-Y), slim throat, swelling grip
part('Hairbrushes__handle', f_cyl(0.012, 0.015, 0.10), lambda: lathe(
    [(0, -0.05), (0.0142, -0.05), (0.015, -0.0475), (0.0125, -0.040), (0.0108, -0.028), (0.0112, -0.012),
     (0.0122, 0.008), (0.0124, 0.025), (0.0116, 0.040), (0.011, 0.05), (0, 0.05)], seg=12), smooth=50)


def brush_knob():
    # rounded end cap of the handle (handle side +X) with a hanging hole
    s = ax_x(lathe([(0, -0.016), (0.007, -0.0152), (0.0125, -0.011), (0.0155, -0.005), (0.016, 0.0),
                    (0.0152, 0.005), (0.0128, 0.0095), (0.0112, 0.0135), (0.0112, 0.016), (0, 0.016)], seg=12))
    s = cut(s, cyl(0.0036, 0.0036, 0.05, seg=10, bevel_r=0).move(x=-0.005))
    return fitf(s, f_sphere(0.016))

part('Hairbrushes__knob', f_sphere(0.016), brush_knob, smooth=50)
# ball-tipped nylon pin
part('Hairbrushes__bristle', f_cyl(0.001, 0.001, 0.012), lambda: lathe(
    [(0, -0.006), (0.0007, -0.006), (0.0007, 0.0036), (0.001, 0.0044), (0.001, 0.0054), (0.0006, 0.006),
     (0, 0.006)], seg=5), smooth=60)


# ======================================================= standing mirrors ==
# vanity mirror: ogee foot, turned stem, moulded bezel, domed back cover
part('StandingMirrors__base', f_cyl(0.055, 0.065, 0.010), lambda: lathe(
    [(0, -0.005), (0.064, -0.005), (0.065, -0.0042), (0.065, -0.0028), (0.062, -0.0015), (0.059, 0.0005),
     (0.0585, 0.002), (0.0565, 0.0038), (0.053, 0.005), (0, 0.005)], seg=28))
part('StandingMirrors__stem', f_cyl(0.005, 0.005, 0.070), lambda: lathe(
    [(0, -0.035), (0.005, -0.035), (0.005, -0.032), (0.0035, -0.029), (0.003, -0.02), (0.003, 0.024),
     (0.0042, 0.028), (0.003, 0.031), (0.0028, 0.035), (0, 0.035)], seg=10), smooth=50)
# bezel section (dr, dz): flat back, inner face, half-round bead in front
# of the glass (glass at z = +0.002)
part('StandingMirrors__frame', f_torus(0.055, 0.006), lambda: fitf(ring_section(0.055, [
    (0.006, -0.003), (0.0045, -0.0055), (0.0015, -0.006), (-0.0045, -0.006), (-0.006, -0.0042),
    (-0.006, 0.0035), (-0.0045, 0.0055), (-0.0015, 0.006), (0.0025, 0.0052), (0.0055, 0.002)], seg=26),
    f_torus(0.055, 0.006)))
F_SM_BACK = f_bounds((-0.055, -0.055, 0), (0.055, 0.055, 0), 'plane')
part('StandingMirrors__back', F_SM_BACK, lambda: ax_z(lathe(
    [(0, -0.0015), (0.055, -0.0015), (0.055, 0.0), (0.050, 0.0006), (0.035, 0.0012), (0, 0.0015)], seg=24)))
# pivot knobs (same part on both sides, so symmetric)
part('StandingMirrors__peg', f_cyl(0.005, 0.005, 0.010), lambda: lathe(
    [(0.0042, -0.005), (0.005, -0.0035), (0.0043, 0.0), (0.005, 0.0035), (0.0042, 0.005)], seg=10))


# ============================================================ hairdryers ==
def rrect_pts(w, d, r, cseg=2):
    """Rounded rectangle (a, b) outline with 4*(cseg+1) points."""
    r = max(1e-5, min(r, w / 2 - 1e-5, d / 2 - 1e-5))
    out = []
    for cx, cz, a0 in ((w / 2 - r, d / 2 - r, 0), (-w / 2 + r, d / 2 - r, 90),
                       (-w / 2 + r, -d / 2 + r, 180), (w / 2 - r, -d / 2 + r, 270)):
        for i in range(cseg + 1):
            a = math.radians(a0 + 90 * i / cseg)
            out.append((cx + r * math.cos(a), cz + r * math.sin(a)))
    return out

# barrel: +Y is the back (intake recess under the back grille disc),
# -Y the front face the nozzle sits on; a shell seam round the middle
part('Hairdryers__barrel', f_cyl(0.045, 0.045, 0.14), lambda: lathe(
    [(0, -0.07), (0.040, -0.07), (0.0435, -0.069), (0.045, -0.064), (0.045, -0.003), (0.0438, -0.0012),
     (0.0438, 0.0012), (0.045, 0.003), (0.045, 0.062), (0.0435, 0.068), (0.041, 0.07), (0.0385, 0.07),
     (0.037, 0.066), (0, 0.066)], seg=22))
# nozzle (the placer's frustum: narrow end on the barrel, wide end out front)
part('Hairdryers__nozzle', f_cyl(0.0315, 0.045, 0.025), lambda: lathe(
    [(0.0315, 0.0125), (0.033, 0.009), (0.043, -0.0075), (0.045, -0.0105), (0.0443, -0.0125),
     (0.040, -0.0125), (0.0385, -0.0115), (0.035, -0.0115), (0.0335, -0.0125), (0, -0.0125)], seg=22))
# pistol grip: flared butt, ribbed grip, collar under the barrel
part('Hairdryers__handle', f_cyl(0.018, 0.020, 0.075), lambda: lathe(
    [(0, -0.0375), (0.017, -0.0375), (0.0198, -0.0352), (0.020, -0.031), (0.0172, -0.025), (0.0158, -0.018),
     (0.0167, -0.010), (0.0158, -0.002), (0.0167, 0.006), (0.0158, 0.014), (0.0168, 0.024), (0.018, 0.032),
     (0.018, 0.0375), (0, 0.0375)], seg=11), smooth=50)
# rocker switch with a pivot groove
part('Hairdryers__switch', f_box(0.010, 0.016, 0.006), lambda: fitf(cut(
    rbox(0.010, 0.016, 0.006, r=0.0015, seg=1), box(0.02, 0.0012, 0.004).move(z=0.0035)),
    f_box(0.010, 0.016, 0.006)))
# round flex segment with eased ends
part('Hairdryers__cord', f_cyl(0.003, 0.003, 0.027), lambda: lathe(
    [(0, -0.0135), (0.0026, -0.0135), (0.003, -0.0125), (0.003, 0.0125), (0.0026, 0.0135), (0, 0.0135)],
    seg=8), smooth=60)


def dryer_plug():
    # moulded plug: strain-relief boot at the cord end (-X), square face (+X)
    rings = []
    for x, h, d, r in ((-0.009, 0.006, 0.007, 0.0028), (-0.005, 0.0082, 0.010, 0.003),
                       (-0.0025, 0.010, 0.014, 0.003), (0.009, 0.010, 0.014, 0.0025)):
        rings.append([(x, a, b) for a, b in rrect_pts(h, d, r, 2)])
    return fitf(loft(rings), f_box(0.018, 0.010, 0.014))

part('Hairdryers__plug', f_box(0.018, 0.010, 0.014), dryer_plug)
part('Hairdryers__prong', f_box(0.010, 0.002, 0.003), lambda: fitf(flat(
    [(-0.005, -0.0015), (0.0035, -0.0015), (0.0048, -0.0009), (0.005, 0.0), (0.0048, 0.0009),
     (0.0035, 0.0015), (-0.005, 0.0015)], 0.002, bevel_r=0.0003), f_box(0.010, 0.002, 0.003)))


# ========================================================== pill bottles ==
def ring_lathe(profile, seg=24):
    """Revolve a closed (r, y) loop (no point on the axis): bands, sleeves."""
    s = lathe(list(profile) + [profile[0]], seg, cap_bottom=False, cap_top=False)
    return _fw(weld(s))


def pill_label():
    # paper sleeve with the overlapping end lifted a little at the seam
    s = ring_lathe([(0.0201, -0.01925), (0.0205, -0.01925), (0.0205, 0.01925), (0.0201, 0.01925)], seg=24)
    def lift(p):
        a = math.atan2(p.x, p.z)
        if 0.0 < a < 0.55:
            k = 1 + 0.018 * (1 - a / 0.55)
            p.x *= k
            p.z *= k
        return p
    return s.warp(lift)

part('PillBottles__label', f_cyl(0.0205, 0.0205, 0.0385), pill_label)
# child-proof cap: hollow skirt with grip ribs, recessed top ring
part('PillBottles__cap', f_cyl(0.021, 0.021, 0.010), lambda: lathe(
    [(0, 0.0025), (0.0185, 0.0025), (0.0188, -0.005), (0.0203, -0.005), (0.021, -0.0042), (0.021, 0.0038),
     (0.0203, 0.005), (0.016, 0.005), (0.015, 0.0043), (0, 0.0043)], seg=16).warp(
    rib_warp(8, 0.035, -0.0043, 0.0039)))
part('PillBottles__tab', f_box(0.002, 0.008, 0.004), lambda: rbox(0.002, 0.008, 0.004, r=0.0005, seg=1))
# capsule shells and domed ends, tablets with bevelled faces
part('PillBottles__half', f_cyl(0.002, 0.002, 0.004), lambda: cyl(0.002, 0.002, 0.004, seg=10, bevel_r=0),
     smooth=60)
part('PillBottles__end', f_sphere(0.002), lambda: sphere(0.002, seg=8, rings=5), smooth=180)
part('PillBottles__tablet', f_cyl(0.003, 0.003, 0.002), lambda: lathe(
    [(0, -0.001), (0.0022, -0.0009), (0.003, -0.0004), (0.003, 0.0004), (0.0022, 0.0009), (0, 0.001)],
    seg=10), smooth=50)


# ============================================================ plush bears ==
def seam(depth, zsign=1):
    """Pull the vertices on the x = 0 meridian in: a stitched centre seam."""
    def w(p):
        if abs(p.x) < 1e-5 and p.z * zsign > 0:
            p.z -= zsign * depth
        return p
    return w


def bear_body():
    # sitting pear-shaped torso with a front seam and two stubby feet
    torso = lathe([(0, -0.058), (0.03, -0.056), (0.048, -0.045), (0.057, -0.028), (0.06, -0.01),
                   (0.057, 0.01), (0.05, 0.03), (0.038, 0.046), (0.022, 0.056), (0, 0.06)], seg=16)
    torso.warp(seam(0.0018, 1)).warp(seam(0.0012, -1))
    feet = Shape()
    for sx in (-1, 1):
        feet += sphere(1, seg=10, rings=6).scale(0.018, 0.015, 0.024).move(x=sx * 0.029, y=-0.044, z=0.034)
    s = noisy(torso + feet, 0.0012, scale=40, seed=21)
    return fitf(s, f_sphere(0.06))

part('PlushBears__body', f_sphere(0.06), bear_body, smooth=180)


def bear_head():
    # round head (the eyes sit on its surface), fuller lower face / muzzle,
    # gusset seam from the crown down to the muzzle
    s = sphere(0.045, seg=16, rings=10)
    def muzzle(p):
        if p.z > 0 and -0.034 < p.y < 0.006:
            fy = 1 - abs(p.y + 0.014) / 0.02
            fx = max(0.0, 1 - abs(p.x) / 0.026)
            p.z = min(0.045, p.z + 0.006 * max(0.0, fy) * fx)
        return p
    s.warp(muzzle).warp(lambda p: Vector((p.x, p.y, p.z - (0.0012 if abs(p.x) < 1e-5 and p.z > 0 and p.y > 0.004 else 0))))
    return fitf(noisy(s, 0.0008, scale=45, seed=22), f_sphere(0.045))

part('PlushBears__head', f_sphere(0.045), bear_head, smooth=180)


def bear_ear():
    # round ear with a rolled rim and a cupped front (+Z); the rounded back
    # half sinks into the head
    s = ax_z(lathe([(0, -0.015), (0.007, -0.0135), (0.0115, -0.0095), (0.014, -0.004), (0.015, 0.002),
                    (0.0142, 0.0095), (0.0118, 0.0138), (0.0098, 0.0128), (0.0075, 0.0105), (0.004, 0.0092),
                    (0, 0.009)], seg=12))
    return fitf(noisy(s, 0.0004, scale=90, seed=24), f_sphere(0.015))

part('PlushBears__ear', f_sphere(0.015), bear_ear, smooth=180)


def bear_arm():
    # stubby limb hanging down and forward, paw end fuller (symmetric in x,
    # the same part serves both sides)
    s = lathe([(0, -0.03), (0.012, -0.028), (0.0175, -0.02), (0.019, -0.008), (0.0175, 0.006),
               (0.015, 0.018), (0.010, 0.027), (0, 0.03)], seg=12).rot(x=-0.75)
    return fitf(noisy(s, 0.0008, scale=50, seed=23), f_sphere(0.028))

part('PlushBears__arm', f_sphere(0.028), bear_arm, smooth=180)


# =========================================================== rubber balls ==
F_BALL_HALF = f_bounds((-0.05, 0, -0.05), (0.05, 0.05, 0.05))


def ball_half():
    # hemisphere shell: moulding bead at the equator, valve dimple at the pole
    prof = [(0, 0), (0.0492, 0), (0.0497, 0.0008), (0.0494, 0.0017)]
    for i in range(1, 8):
        a = math.radians(4 + 80 * i / 7)
        prof.append((0.05 * math.cos(a), 0.05 * math.sin(a)))
    prof += [(0.0028, 0.0499), (0.0018, 0.0492), (0, 0.0492)]
    return fitf(lathe(prof, seg=22), F_BALL_HALF)

part('RubberBalls__half', F_BALL_HALF, ball_half, smooth=50)


# ========================================================== small crayons ==
# body: paper wrapper with printed band ridges, flat back end (+Y);
# -Y end meets the tip cone
part('SmallCrayons__body', f_cyl(0.008, 0.008, 0.06), lambda: lathe(
    [(0, 0.03), (0.0072, 0.03), (0.0078, 0.0293), (0.0078, 0.026), (0.008, 0.0255), (0.008, 0.0225),
     (0.0078, 0.022), (0.0078, -0.014), (0.008, -0.0145), (0.008, -0.0175), (0.0078, -0.018),
     (0.0078, -0.03), (0, -0.03)], seg=12), smooth=50)


def crayon_tip():
    # sharpened cone with a blunt, slightly worn point
    s = lathe([(0, -0.007), (0.008, -0.007), (0.0078, -0.0055), (0.0032, 0.0045), (0.0021, 0.0063),
               (0, 0.007)], seg=10)
    s.warp(lambda p: Vector((p.x + 0.0009 * max(0.0, (p.y + 0.001) / 0.008), p.y, p.z)))
    return fitf(s, f_cone(0.008, 0.014))

part('SmallCrayons__tip', f_cone(0.008, 0.014), crayon_tip, smooth=50)


# ======================================================= children's shoes ==
# top-view last: rounded heel (-X), narrow waist, widest at the ball, toe (+X)
_SHOE_W = [(0.0, 0.0), (0.015, 0.55), (0.05, 0.76), (0.10, 0.83), (0.20, 0.84), (0.32, 0.78), (0.45, 0.76),
           (0.58, 0.88), (0.70, 0.97), (0.80, 0.97), (0.88, 0.88), (0.94, 0.72), (0.98, 0.45), (1.0, 0.0)]


def _interp(tab, u):
    for (a, fa), (b, fb) in zip(tab, tab[1:]):
        if a <= u <= b:
            return fa + (fb - fa) * (u - a) / (b - a)
    return tab[-1][1]


def shoe_outline(L, W, inset=0.0):
    """Closed (x, z) outline, optionally offset inwards."""
    top = [(-L / 2 + L * u, W / 2 * f) for u, f in _SHOE_W[1:-1]]
    pts = [(-L / 2, 0.0)] + top + [(L / 2, 0.0)] + [(x, -z) for x, z in reversed(top)]
    if inset <= 0:
        return pts
    n = len(pts)
    out = []
    for i in range(n):
        a, b, c = Vector(pts[i - 1]), Vector(pts[i]), Vector(pts[(i + 1) % n])
        e1, e2 = (b - a).normalized(), (c - b).normalized()
        nm = Vector((e1.y + e2.y, -(e1.x + e2.x)))
        nm = nm.normalized() if nm.length > 1e-9 else Vector((0, 0))
        if nm.dot(b) > 0:
            nm = -nm
        out.append(tuple(b + nm * inset))
    return out


def shoe_sole():
    # rubber sole: chamfered bottom edge, foxing groove round the sidewall
    L, W = 0.14, 0.06
    rings = []
    for y, ins in ((-0.004, 0.0012), (-0.0032, 0.0), (0.0004, 0.0), (0.0011, 0.0007), (0.0018, 0.0),
                   (0.0034, 0.0), (0.004, 0.0009)):
        rings.append([(x, y, z) for x, z in shoe_outline(L, W, ins)])
    return fitf(loft(rings), f_box(0.14, 0.008, 0.06))

part('ChildrenShoes__sole', f_box(0.14, 0.008, 0.06), shoe_sole)

_SHOE_H = [(0.0, 0.045), (0.08, 0.050), (0.18, 0.049), (0.30, 0.042), (0.42, 0.040), (0.52, 0.046),
           (0.62, 0.045), (0.75, 0.041), (0.88, 0.036), (1.0, 0.030)]


def shoe_upper():
    # heel counter, collar dipping at the ankle, tongue/instep, vamp sloping
    # into the toe cap; foot opening cut in; strap over the instep
    L, W, y0 = 0.14, 0.06, -0.025
    H = lambda x: _interp(_SHOE_H, (x + L / 2) / L)
    # tucked inside the sole below its top (y = 0.008), full last above it
    low = ((0.0005, 0.0024), (0.0105, 0.0))
    levels = ((0.6, 0.0), (0.84, 0.0022), (0.95, 0.0065), (1.0, 0.012))
    rings = [[(x, y0 + h, z) for x, z in shoe_outline(L, W, ins)] for h, ins in low]
    for t, ins in levels:
        rings.append([(x, y0 + t * H(x), z) for x, z in shoe_outline(L, W, ins)])
    s = loft(rings)
    s = cut(s, sphere(1, seg=12, rings=8).scale(0.028, 0.030, 0.0195).move(x=-0.033, y=0.019))
    x0 = 0.004
    hw = W / 2 * _interp(_SHOE_W, (x0 + L / 2) / L)
    path = []
    side = [(0.0125 / H(x0), 0.0)] + list(levels)
    for t, ins in side[:-1]:
        path.append((x0, y0 + t * H(x0), -(hw - ins) - 0.0009))
    path.append((x0, y0 + H(x0) + 0.0009, 0.0))
    for t, ins in reversed(side[:-1]):
        path.append((x0, y0 + t * H(x0), (hw - ins) + 0.0009))
    strap = sweep(path, [(-0.0012, -0.0075), (0.0012, -0.0075), (0.0012, 0.0075), (-0.0012, 0.0075)],
                  up=(1, 0, 0))
    return fitf(s + strap, f_box(0.14, 0.05, 0.06))

part('ChildrenShoes__upper', f_box(0.14, 0.05, 0.06), shoe_upper)


def shoe_toe():
    # toe cap: dome forward (+X), a rubber-edge ridge where it meets the
    # upper, flattened where it rests on the sole
    s = ax_x(lathe([(0, -0.03), (0.02, -0.027), (0.027, -0.018), (0.029, -0.006), (0.0296, -0.002),
                    (0.030, 0.0), (0.0292, 0.003), (0.0282, 0.008), (0.024, 0.017), (0.016, 0.0245),
                    (0.008, 0.0285), (0, 0.03)], seg=13))
    s.warp(lambda p: Vector((p.x, p.y if p.y > -0.021 else -0.021 + (p.y + 0.021) * 0.35, p.z)))
    # toe box: the top slopes down towards the front instead of a ball
    s.warp(lambda p: Vector((p.x, p.y * (1 - 0.45 * max(0.0, p.x + 0.004) / 0.034) if p.y > 0 else p.y, p.z)))
    return fitf(s, f_sphere(0.03))

part('ChildrenShoes__toe', f_sphere(0.03), shoe_toe, smooth=60)


# =============================================================== marbles ==
part('MarbleScatters__marble', f_sphere(0.010), lambda: sphere(0.010, seg=12, rings=8), smooth=180)


# ======================================================= building blocks ==
def block_body():
    # moulded 2x2 brick: eased edges, hollow underside with the centre tube
    s = rbox(0.06, 0.03, 0.06, r=0.0012, seg=1)
    s = cut(s, box(0.0556, 0.03, 0.0556).move(y=-0.0035))
    tube_ = ring_lathe([(0.0048, -0.015), (0.0066, -0.015), (0.0066, 0.0118), (0.0048, 0.0118)], seg=12)
    return s + tube_

part('BuildingBlocks__body', f_box(0.06, 0.03, 0.06), block_body)
part('BuildingBlocks__stud', f_cyl(0.008, 0.008, 0.006), lambda: lathe(
    [(0.008, -0.003), (0.008, 0.0021), (0.0077, 0.0028), (0.007, 0.003), (0, 0.003)], seg=14,
    cap_bottom=False))


# ============================================================== toy cars ==
def car_body():
    # die-cast lower body: rear deck, hood sloping to the front (+X),
    # bumpers, wheel arches cut over the axles
    ol = [(-0.040, -0.0065), (-0.040, 0.004), (-0.0375, 0.0085), (-0.032, 0.010), (0.016, 0.010),
          (0.030, 0.0082), (0.0375, 0.0055), (0.040, 0.0015), (0.040, -0.0065), (0.0375, -0.010),
          (-0.0375, -0.010)]
    s = prism(ol, 0.05, bevel_r=0.002, bevel_seg=1)
    arches = [cyl(0.0118, 0.0118, 0.07, seg=12, bevel_r=0).rot(x=math.pi / 2).move(x=sx * 0.028, y=-0.013)
              for sx in (-1, 1)]
    return fitf(cut(s, *arches), f_box(0.08, 0.02, 0.05))

part('ToyCars__body', f_box(0.08, 0.02, 0.05), car_body)


def car_roof():
    # cabin: raked windscreen (+X), rear window, tumblehome sides, shallow
    # window recesses
    rings = []
    for y, x0, x1, hd, r in ((-0.009, -0.020, 0.020, 0.0225, 0.003), (0.005, -0.0165, 0.0105, 0.021, 0.004),
                             (0.0082, -0.0145, 0.0078, 0.0195, 0.005), (0.009, -0.0125, 0.0055, 0.0175, 0.005)):
        rings.append([((x0 + x1) / 2 + a, y, b) for a, b in rrect_pts(x1 - x0, 2 * hd, r, 2)])
    s = loft(rings)
    win = [box(0.024, 0.0085, 0.004).rot(x=sz * 0.11).move(x=-0.003, y=0.0005, z=sz * 0.0232) for sz in (-1, 1)]
    win += [box(0.004, 0.0085, 0.034).rot(z=-0.26).move(x=0.0167, y=0.0005),
            box(0.004, 0.0085, 0.034).rot(z=0.2).move(x=-0.0206, y=0.0005)]
    return fitf(cut(s, *win), f_box(0.04, 0.018, 0.045))

part('ToyCars__roof', f_box(0.04, 0.018, 0.045), car_roof)
# tyre with rounded shoulders and a dished hub on both faces
part('ToyCars__wheel', f_cyl(0.010, 0.010, 0.008), lambda: lathe(
    [(0, -0.0036), (0.0085, -0.004), (0.010, -0.0026), (0.010, 0.0026), (0.0085, 0.004), (0, 0.0036)],
    seg=12), smooth=50)


# ============================================================ small dolls ==
def pleats(n, amt, y_top, y_hem):
    """Skirt folds that deepen towards the hem."""
    def w(p):
        if p.y < y_top:
            k = min(1.0, (y_top - p.y) / (y_top - y_hem))
            f = 1 - amt * k * (0.5 + 0.5 * math.cos(n * math.atan2(p.x, p.z)))
            p.x *= f
            p.z *= f
        return p
    return w

# dress: rolled hem, pleated skirt flaring from the waist, bodice, shoulders
part('SmallDolls__dress', f_cyl(0.020, 0.020, 0.07), lambda: fitf(lathe(
    [(0, -0.035), (0.0185, -0.035), (0.020, -0.0325), (0.0165, -0.012), (0.013, 0.002), (0.0135, 0.018),
     (0.0125, 0.027), (0.0095, 0.032), (0.006, 0.035), (0, 0.035)], seg=16).warp(pleats(8, 0.09, 0.0, -0.034)),
    f_cyl(0.020, 0.020, 0.07)), smooth=60)


def doll_head():
    # round skull, narrower chin, small ears (eyes sit on the upper face)
    s = sphere(0.028, seg=12, rings=9)
    def chin(p):
        if p.y < -0.008:
            k = 1 - 0.14 * min(1.0, (-0.008 - p.y) / 0.02)
            p.x *= k
            p.z *= k
        return p
    s.warp(chin)
    ears = sum((sphere(1, seg=6, rings=4).scale(0.0025, 0.005, 0.0035).move(x=sx * 0.0268, y=-0.001)
                for sx in (-1, 1)), Shape())
    return fitf(s + ears, f_sphere(0.028))

part('SmallDolls__head', f_sphere(0.028), doll_head, smooth=70)
# vinyl arm: rounded ends, slimmer between (symmetric: both sides share it)
part('SmallDolls__arm', f_cyl(0.008, 0.008, 0.06), lambda: lathe(
    [(0, -0.03), (0.005, -0.0295), (0.0075, -0.027), (0.008, -0.024), (0.0068, -0.018), (0.0062, -0.006),
     (0.0064, 0.0), (0.0062, 0.006), (0.0068, 0.018), (0.008, 0.024), (0.0075, 0.027), (0.005, 0.0295),
     (0, 0.03)], seg=10), smooth=70)


def doll_leg():
    # thigh, knee, slim ankle and a small foot pointing forward (+Z)
    leg = lathe([(0, -0.026), (0.0052, -0.024), (0.0058, -0.018), (0.0070, -0.008), (0.0068, -0.002),
                 (0.0078, 0.008), (0.0088, 0.02), (0.0085, 0.027), (0.006, 0.0295), (0, 0.03)], seg=10)
    leg.move(z=-0.0015)
    foot = sphere(1, seg=8, rings=5).scale(0.0055, 0.0042, 0.0075).move(y=-0.0255, z=0.0022)
    return fitf(leg + foot, f_cyl(0.009, 0.009, 0.06))

part('SmallDolls__leg', f_cyl(0.009, 0.009, 0.06), doll_leg, smooth=70)


# ============================================================ small kites ==
def kite_tail():
    # tail string with the three bows' knots (at local y 0.09, 0.03, -0.03)
    prof = [(0, -0.125), (0.0012, -0.124)]
    for k in (-0.03, 0.03, 0.09):
        prof += [(0.0012, k - 0.004), (0.002, k - 0.0015), (0.002, k + 0.0015), (0.0012, k + 0.004)]
    prof += [(0.0012, 0.124), (0, 0.125)]
    return fitf(lathe(prof, seg=6), f_cyl(0.002, 0.002, 0.25))

part('SmallKites__tail', f_cyl(0.002, 0.002, 0.25), kite_tail, smooth=60)
F_KITE_BOW = f_bounds((-0.01, -0.0075, 0), (0.01, 0.0075, 0), 'plane')


def kite_bow():
    # ribbon bow: two pinched wings with notched ends, a knot in the middle
    # (kept thin: the plane frame's thickness goes down into the floor)
    ol = [(-0.01, -0.0075), (-0.0015, -0.0022), (0.0015, -0.0022), (0.01, -0.0075), (0.0085, 0.0),
          (0.01, 0.0075), (0.0015, 0.0022), (-0.0015, 0.0022), (-0.01, 0.0075), (-0.0085, 0.0)]
    wings = prism(ol, 0.0004).move(z=-0.0002)
    knot = rbox(0.0042, 0.0052, 0.00085, r=0.0003, seg=1).move(z=0.0)
    return wings + knot

part('SmallKites__bow', F_KITE_BOW, kite_bow)


# ========================================================= pinwheels ==
part('SmallPinwheels__stick', f_cyl(0.005, 0.005, 0.28), lambda: lathe(
    [(0, -0.14), (0.0045, -0.14), (0.005, -0.1392), (0.005, 0.1392), (0.0045, 0.14), (0, 0.14)], seg=10),
    smooth=50)
# pin head: dome towards the viewer (+Y), washer behind
part('SmallPinwheels__hub', f_cyl(0.010, 0.010, 0.008), lambda: lathe(
    [(0, -0.004), (0.0088, -0.004), (0.010, -0.0022), (0.0082, -0.0008), (0.0070, 0.0017), (0.0042, 0.0035),
     (0, 0.004)], seg=10), smooth=50)


# ============================================================ small yo-yos ==
# a yo-yo half, symmetric (the same part is used both ways round): rounded
# rim, both faces with a shallow inset cap
part('SmallYoYos__half', f_cyl(0.024, 0.024, 0.006), lambda: lathe(
    [(0, -0.0022), (0.0125, -0.0022), (0.0205, -0.003), (0.0235, -0.0019), (0.024, 0.0),
     (0.0235, 0.0019), (0.0205, 0.003), (0.0125, 0.0022), (0, 0.0022)], seg=18), smooth=50)
part('SmallYoYos__axle', f_cyl(0.005, 0.005, 0.020), lambda: lathe(
    [(0, -0.01), (0.0045, -0.01), (0.005, -0.0092), (0.005, 0.0092), (0.0045, 0.01), (0, 0.01)], seg=12))


def yoyo_string():
    # twisted string with a knot at the free end (+Y)
    prof = [(0, -0.05)]
    for i in range(9):
        y = -0.05 + 0.0105 * i
        prof += [(0.00075, y + 0.0005), (0.00095, y + 0.0055)]
    prof += [(0.00075, 0.0445), (0.0016, 0.0465), (0.0014, 0.049), (0, 0.05)]
    return fitf(lathe(prof, seg=5), f_cyl(0.001, 0.001, 0.10))

part('SmallYoYos__string', f_cyl(0.001, 0.001, 0.10), yoyo_string, smooth=60)


# ============================================================== toy drums ==
def drum_body():
    # shell between two hoops; the head sits in the top hoop (membrane disc
    # 1 mm above); a zigzag tension cord laced round the shell
    s = lathe([(0, -0.045), (0.057, -0.045), (0.06, -0.043), (0.06, -0.036), (0.0565, -0.035),
               (0.0565, 0.035), (0.06, 0.036), (0.06, 0.043), (0.057, 0.045), (0.054, 0.043), (0, 0.043)], seg=18)
    n = 6
    pts = []
    for i in range(2 * n + 1):
        a = math.pi * i / n
        y = 0.034 if i % 2 == 0 else -0.034
        pts.append((a, y))
    path = []
    rr = 0.0565 + 0.0009
    for (a0, y0), (a1, y1) in zip(pts, pts[1:]):
        path.append((rr * math.sin(a0), y0, rr * math.cos(a0)))
        am = (a0 + a1) / 2
        path.append((rr * math.sin(am), (y0 + y1) / 2, rr * math.cos(am)))
    path.append(path[0])
    cord = sweep(path, circ(0.0011, 4), up=(0, 1, 0))
    return fitf(s + cord, f_cyl(0.06, 0.06, 0.09))

part('ToyDrums__body', f_cyl(0.06, 0.06, 0.09), drum_body)


# ========================================================= pacifiers ==
# teat: flared base on the shield (+Y), thin neck, round bulb (-Y tip)
part('SmallPacifiers__nipple', f_cyl(0.008, 0.010, 0.020), lambda: lathe(
    [(0, -0.010), (0.008, -0.0085), (0.0098, -0.005), (0.0075, -0.001), (0.0045, 0.004), (0.008, 0.010)],
    seg=10), smooth=70)


def paci_shield():
    # curved shield dished towards the teat side (-Y), rolled rim, two vents
    s = lathe([(0, 0.001), (0.011, 0.0003), (0.018, -0.0019), (0.0195, -0.0025), (0.0201, -0.0016),
               (0.018, -0.0004), (0.011, 0.0017), (0, 0.0025)], seg=12)
    vents = [cyl(0.0028, 0.0028, 0.02, seg=7, bevel_r=0).move(x=sx * 0.0118) for sx in (-1, 1)]
    return fitf(cut(s, *vents), f_cyl(0.020, 0.020, 0.005))

part('SmallPacifiers__shield', f_cyl(0.020, 0.020, 0.005), paci_shield, smooth=50)


def paci_ring():
    # handle ring with a hinge knuckle where it clips to the shield (+X)
    r = torus(0.010, 0.002, seg=16, tseg=6)
    knuckle = cyl(0.0027, 0.0027, 0.0048, seg=8, bevel_r=0).rot(x=math.pi / 2).move(x=0.0102)
    return fitf(r + knuckle, f_torus(0.010, 0.002))

part('SmallPacifiers__ring', f_torus(0.010, 0.002), paci_ring, smooth=60)


# ======================================================== skipping ropes ==
def rope_coil():
    # a loose coil of one and a half turns; the two ends run into the
    # handles at angles 0 and pi; the turns ride over each other
    n = 36
    pts = []
    for i in range(n + 1):
        a = 3 * math.pi * i / n
        R = 0.0965 + 0.007 * a / (3 * math.pi)
        z = -0.0036 + 0.0072 * a / (3 * math.pi) + 0.0006 * math.sin(5 * a)
        pts.append((R * math.cos(a), R * math.sin(a), z))
    s = sweep(pts, circ(0.0042, 6), up=(0, 0, 1))
    return fitf(s, f_torus(0.10, 0.008))

part('SkippingRopes__coil', f_torus(0.10, 0.008), rope_coil, smooth=70)
# turned wooden handle: collars at both ends, swelling grip (symmetric)
part('SkippingRopes__handle', f_cyl(0.010, 0.010, 0.06), lambda: lathe(
    [(0, -0.03), (0.006, -0.03), (0.0075, -0.028), (0.0075, -0.025), (0.0068, -0.023), (0.0092, -0.012),
     (0.010, 0.0), (0.0092, 0.012), (0.0068, 0.023), (0.0075, 0.025), (0.0075, 0.028), (0.006, 0.03),
     (0, 0.03)], seg=12), smooth=50)


# ====================================================== children's books ==
def book_pages():
    # thick board-book pages: spine side (-X) flat, rounded fore-edge
    # corners, fore-edge bowed in, a groove between every board
    W, D = 0.095, 0.135
    def outline(ins):
        x1, z1, c = W / 2 - ins, D / 2 - ins, 0.0012
        r = 0.006
        pts = [(-W / 2, -z1)]
        for k in range(3):
            a = -math.pi / 2 + (math.pi / 2) * k / 2
            pts.append((x1 - r + r * math.cos(a), -z1 + r + r * math.sin(a)))
        pts += [(x1 - c, -D / 4), (x1 - c * 1.3, 0.0), (x1 - c, D / 4)]
        for k in range(3):
            a = (math.pi / 2) * k / 2
            pts.append((x1 - r + r * math.cos(a), z1 - r + r * math.sin(a)))
        pts.append((-W / 2, z1))
        return pts
    rings = []
    n = 6
    for i in range(n + 1):
        y = -0.008 + 0.016 * i / n
        if 0 < i < n:
            rings.append([(x, y - 0.0004, z) for x, z in outline(0.0)])
            rings.append([(x, y, z) for x, z in outline(0.0007)])
            rings.append([(x, y + 0.0004, z) for x, z in outline(0.0)])
        else:
            rings.append([(x, y, z) for x, z in outline(0.0004)])
    return fitf(loft(rings), f_box(0.095, 0.016, 0.135))

part('ChildrensBooks__pages', f_box(0.095, 0.016, 0.135), book_pages)


# ========================================================== tambourines ==
def tambourine_rim():
    # wooden hoop band (membrane disc meets its inner face at r 0.086),
    # six slots through the band where the jingles sit
    sec = [(0.010, -0.008), (0.008, -0.010), (-0.002, -0.010), (-0.004, -0.008), (-0.004, 0.008),
           (-0.002, 0.010), (0.008, 0.010), (0.010, 0.008)]
    s = ring_section(0.09, sec, seg=22)
    slots = [box(0.03, 0.021, 0.0105).move(x=0.093, z=0.003).rot(z=math.radians(60 * k)) for k in range(6)]
    return fitf(cut(s, *slots), f_torus(0.09, 0.010))

part('SmallTambourines__rim', f_torus(0.09, 0.010), tambourine_rim)
# a pair of domed zils face to face, rims almost touching
part('SmallTambourines__jingle', f_cyl(0.008, 0.008, 0.004), lambda: lathe(
    [(0, -0.002), (0.0045, -0.0017), (0.008, -0.0004), (0.0074, 0.0), (0.008, 0.0004), (0.0045, 0.0017),
     (0, 0.002)], seg=10), smooth=40)


# ============================================================== maracas ==
# gourd-shaped head, necked where the handle enters (-Y)
part('SmallMaracas__bulb', f_sphere(0.028), lambda: fitf(lathe(
    [(0, -0.028), (0.0078, -0.028), (0.0092, -0.0255), (0.0145, -0.0215), (0.021, -0.0145), (0.0258, -0.0055),
     (0.028, 0.0035), (0.0272, 0.012), (0.0235, 0.0195), (0.0165, 0.025), (0.0075, 0.0277), (0, 0.028)],
    seg=14), f_sphere(0.028)), smooth=70)
# turned handle: end knob (-Y), slim grip, collar flaring into the head
part('SmallMaracas__handle', f_cyl(0.007, 0.006, 0.08), lambda: lathe(
    [(0, -0.04), (0.0055, -0.04), (0.0065, -0.0375), (0.0065, -0.033), (0.0052, -0.030), (0.0047, -0.018),
     (0.0052, -0.004), (0.0062, 0.006), (0.007, 0.012), (0.0066, 0.04), (0, 0.04)], seg=10), smooth=50)
