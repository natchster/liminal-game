"""Family `instruments_nursery_toys_and_leftovers`: the "toys" theme. Toy instruments (recorder, triangle,
xylophone, baton, metronome, castanets), nursery toys (spinning top, hula
hoop, plane, skateboard, boat, jacks, letters, piggy bank, dino, phone,
robot, microphone, piano, wand, sword) and the sadder nursery leftovers
(doll heads and limbs, crib side, child shoe, teddies, tricycle, colouring
book). Bright, chunky, rounded plastic and wood.

Decals, DoubleSide/transparent/basic material primitives, custom buffer geometry
and 1 mm cords are left alone (see docs/research/props-remodelling/instruments_nursery_toys_and_leftovers.md).
"""
import propkit
from propkit import *
from mathutils import Vector

fixw = propkit._fix_winding
HPI = math.pi / 2


# ------------------------------------------------------------- helpers ---
def loft(rings, caps=True):
    """Bridge a list of equal-length 3D rings; cap both ends."""
    v, f = [], []
    n = len(rings[0])
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


def _ccw(pts):
    a = sum(p[0] * q[1] - q[0] * p[1] for p, q in zip(pts, pts[1:] + pts[:1]))
    return list(pts) if a > 0 else list(reversed(pts))


def ystack(outline, prof):
    """A plan outline (x, z) stacked in y: prof = [(inset_fn_arg, y), ...];
    outline(i) returns the outline inset by i."""
    rings = []
    for i, y in prof:
        rings.append([(x, y, z) for x, z in _ccw(outline(i))])
    return loft(rings)


def rr(w, d, r, seg=3):
    """Rounded-rect plan outline generator: inset -> points."""
    return lambda i: rounded_rect(w - 2 * i, d - 2 * i, max(r - i, 0.0002), seg)


def angular(s, fn):
    """Warp each vertex radially around +Y: fn(theta, y, r) -> new r."""
    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-7:
            return p
        k = fn(math.atan2(p.x, p.z), p.y, r) / r
        return Vector((p.x * k, p.y, p.z * k))
    return s.warp(w)


def ring_section(R, sec, seg=32, arc=TAU, a0=0.0):
    """Sweep a closed section [(dr, dz), ...] around Z in the XY plane
    (a torus_geo! with any cross-section); arc < TAU caps the ends."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
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
    return fixw(Shape(v, f))


def circ(r, n=8, sx=1.0, sy=1.0, a0=0.0):
    return [(r * sx * math.cos(a0 + TAU * i / n), r * sy * math.sin(a0 + TAU * i / n)) for i in range(n)]


def xrod(x0, x1, r0, r1=None, seg=8):
    """Short round rod along X with domed ends (pins, stubs)."""
    r1 = r0 if r1 is None else r1
    L = x1 - x0
    s = lathe([(0, 0), (r0 * 0.6, 0), (r0, L * 0.12), (r1, L * 0.88), (r1 * 0.6, L), (0, L)], seg=seg)
    return s.rot(z=-HPI).move(x=x0)


# ------------------------------------------------------------- recorder --
# the recorder lies along world x; frame +y points at the mouth end and
# frame +x is world up (rotation.z = PI/2)
REC_HOLES = (0.06, 0.032, 0.004, -0.024, -0.052)


def recorder_body():
    # middle joint: joint bead below the head, gentle taper to the foot,
    # five finger holes on top and the thumb hole underneath
    s = lathe([(0, -0.12), (0.0108, -0.12), (0.0110, -0.097), (0.0116, -0.095), (0.0116, -0.089), (0.0110, -0.087),
               (0.0116, 0.092), (0.0120, 0.095), (0.0120, 0.104), (0.0114, 0.107), (0.0114, 0.12), (0, 0.12)], seg=14)
    holes = [cyl(0.0031, 0.0031, 0.008, seg=8, bevel_r=0).rot(z=HPI).move(x=0.0141, y=y) for y in REC_HOLES]
    thumb = cyl(0.0029, 0.0029, 0.008, seg=10, bevel_r=0).rot(z=HPI).move(x=-0.0141, y=0.078)
    return cut(s, *holes, thumb)


def recorder_beak():
    # head joint swelling to the head, the underside cut away to a flattened
    # beak tip with the windway slot; the window notch sits on top
    s = lathe([(0, -0.010), (0.0100, -0.010), (0.0106, -0.0082), (0.0136, -0.0056), (0.0140, -0.0030),
               (0.0140, 0.0015), (0.0130, 0.0062), (0.0116, 0.010), (0, 0.010)], seg=14)
    n = Vector((0.640, -0.768))
    c = Vector((-0.008, 0.005)) - n * 0.015
    under = box(0.03, 0.06, 0.04).rot(z=math.atan2(n.y, n.x)).move(x=c.x, y=c.y)
    windway = box(0.0013, 0.012, 0.0075).move(x=0.0036, y=0.0105)
    window = box(0.006, 0.0042, 0.0058).move(x=0.0165, y=-0.0036)
    return cut(s, under, windway, window)


def recorder_foot():
    # foot joint: thick socket ring toward the body, a waist, a small lip at
    # the end round the open bore
    return lathe([(0, -0.004), (0.0050, -0.004), (0.0052, -0.010), (0.0108, -0.010), (0.0122, -0.0086),
                  (0.0122, -0.0066), (0.0114, -0.0046), (0.0118, 0.0025), (0.0156, 0.0048), (0.0160, 0.0085),
                  (0.0148, 0.010), (0, 0.010)], seg=12)


part('SmallRecorders__body', f_cyl(0.012, 0.012, 0.24), recorder_body)
part('SmallRecorders__beak', f_cyl(0.014, 0.010, 0.020), recorder_beak)
part('SmallRecorders__foot', f_cyl(0.016, 0.012, 0.020), recorder_foot)


# ------------------------------------------------------------- triangle --
def tri_rod():
    # round steel rod with domed ends
    h, r = 0.05, 0.003
    return lathe([(0, -h), (r * 0.55, -h), (r * 0.9, -h + 0.0008), (r, -h + 0.002), (r, h - 0.002),
                  (r * 0.9, h - 0.0008), (r * 0.55, h), (0, h)], seg=10)


def tri_beater():
    # striker: thin rod with a turned handle on the +y end
    return lathe([(0, -0.04), (0.0012, -0.04), (0.0019, -0.0393), (0.0019, 0.006), (0.0026, 0.0085),
                  (0.0030, 0.012), (0.0030, 0.036), (0.0026, 0.0393), (0.0012, 0.040), (0, 0.040)], seg=10)


part('SmallTriangles__rod', f_cyl(0.003, 0.003, 0.10), tri_rod)
part('SmallTriangles__beater', f_cyl(0.003, 0.003, 0.08), tri_beater)


# ------------------------------------------------------------ xylophone --
def xylo_rail():
    # wooden rail with eased edges and a shallow felt channel along the top
    s = rbox(0.24, 0.010, 0.010, r=0.0016, seg=2)
    return cut(s, box(0.232, 0.002, 0.0045).move(y=0.0052))


def xylo_bar():
    # tone bar: rounded edges, arched undercut, two countersunk pin holes
    w, h, d = 0.024, 0.006, 0.072
    s = rbox(w, h, d, r=0.0012, seg=2)
    arch = cyl(0.023, 0.023, w * 1.2, seg=16, bevel_r=0).rot(z=HPI).scale(1, 0.09, 1).move(y=-h / 2)
    pins = [cyl(0.0016, 0.0016, 0.004, seg=8, bevel_r=0).move(y=h / 2, z=sz * 0.020) for sz in (-1, 1)]
    return cut(s, arch, *pins)


def xylo_shaft():
    # mallet handle: slim toward the head (-y), rounded butt (+y)
    return lathe([(0, -0.04), (0.0018, -0.04), (0.0021, -0.030), (0.0027, 0.010), (0.0030, 0.034),
                  (0.0026, 0.0388), (0.0014, 0.040), (0, 0.040)], seg=8)


def xylo_head():
    # rubber mallet ball, the handle entering at a pole (-x)
    return sphere(0.008, seg=10, rings=6).rot(z=HPI)


part('SmallXylophones__rail', f_box(0.24, 0.010, 0.010), xylo_rail)
part('SmallXylophones__bar', f_box(0.024, 0.006, 0.072), xylo_bar)
part('SmallXylophones__shaft', f_cyl(0.003, 0.003, 0.08), xylo_shaft)
part('SmallXylophones__head', f_sphere(0.008), xylo_head, smooth=180, weighted=False)


# --------------------------------------------------------------- baton --
def baton_shaft():
    # white baton: thick where it leaves the cork (+y, world x = 0), fine
    # rounded tip at the far end
    return lathe([(0, -0.15), (0.0008, -0.15), (0.0012, -0.1485), (0.0016, -0.10), (0.0023, 0.0),
                  (0.0029, 0.12), (0.0030, 0.1485), (0.0020, 0.15), (0, 0.15)], seg=8)


def baton_cork():
    # pear-shaped cork grip: bulb at the butt (+y), tapering to a small
    # ferrule round the shaft (-y)
    return lathe([(0, -0.015), (0.0030, -0.015), (0.0034, -0.0125), (0.0040, -0.0118), (0.0046, -0.008),
                  (0.0062, 0.000), (0.0076, 0.007), (0.0080, 0.0105), (0.0074, 0.0135), (0.0050, 0.0150),
                  (0, 0.015)], seg=14)


part('ConductorBatons__shaft', f_cyl(0.002, 0.003, 0.30), baton_shaft)
part('ConductorBatons__cork', f_cyl(0.008, 0.008, 0.030), baton_cork, smooth=50)


# ------------------------------------------------------------ metronome --
def metronome_body():
    # wooden pyramid, square to the room with a flat front (+z): a skirt at
    # the foot, eased edges, a small overhanging cap on top
    ring = lambda hw, y, r: [(x, y, z) for x, z in _ccw(rounded_rect(2 * hw, 2 * hw, r, 3))]
    return loft([ring(0.0455, -0.075, 0.003), ring(0.0455, -0.069, 0.003), ring(0.0438, -0.0675, 0.0028),
                 ring(0.0425, -0.066, 0.0027), ring(0.0145, 0.063, 0.0012), ring(0.0168, 0.0645, 0.0014),
                 ring(0.0170, 0.0700, 0.0014), ring(0.0150, 0.075, 0.001)])


def metronome_base():
    # plinth with a rounded top edge and a small bottom chamfer
    return ystack(rr(0.10, 0.10, 0.006), [(0.0006, -0.004), (0, -0.0034), (0, 0.0006), (0.0010, 0.0028),
                                           (0.0022, 0.0038), (0.0032, 0.004)])


def metronome_arm():
    # flat steel pendulum: round pivot end, pointed top
    pts = [(0.0019 * math.cos(a), -0.048 + 0.0019 * math.sin(a))
           for a in [math.pi + math.pi * i / 6 for i in range(7)]]
    pts += [(0.0012, -0.046), (0.0012, 0.044), (0, 0.050), (-0.0012, 0.044), (-0.0012, -0.046)]
    return prism(pts, 0.0014, bevel_r=0.0003)


def metronome_weight():
    # brass slide weight, narrowing to the top
    return taper(rbox(0.010, 0.006, 0.008, r=0.0009, seg=1), top=0.72, along='y', axes='x')


part('SmallMetronomes__body', f_cone(0.055, 0.15), metronome_body)
part('SmallMetronomes__base', f_box(0.10, 0.008, 0.10), metronome_base)
part('SmallMetronomes__arm', f_cyl(0.002, 0.002, 0.10), metronome_arm)
part('SmallMetronomes__weight', f_box(0.010, 0.006, 0.008), metronome_weight)


# ------------------------------------------------------------ castanets --
def castanet_shell():
    # thick wooden shell: domed back, cupped hollow in the flat face (y = 0)
    # with a rounded rim, a little fuller toward the cord ear (+x)
    s = lathe([(0, 0.028), (0.0125, 0.0268), (0.0215, 0.0215), (0.0272, 0.0135), (0.0298, 0.0050),
               (0.0292, 0.0010), (0.0270, 0.0), (0.0240, 0.0), (0.0226, 0.0024), (0.0175, 0.0082),
               (0.0090, 0.0118), (0, 0.0128)], seg=14, cap_bottom=False, cap_top=False)
    return angular(s, lambda a, y, r: r * (0.94 + 0.06 * max(0.0, math.sin(a)) ** 3))


part('SmallCastanets__shell', f_bounds((-0.03, 0, -0.03), (0.03, 0.03, 0.03)), castanet_shell, smooth=50)


# --------------------------------------------------------- spinning top --
# the body cone is flipped by the placer: frame -y (wide) is the top face,
# where the six colour wedges lie; frame +y is the point
def top_body():
    # rounded shoulder with a raised rim round a slightly sunk top face (the
    # wedge discs sit on it), then a full curve down to a small flat that
    # meets the tip
    return lathe([(0, -0.0172), (0.0262, -0.0172), (0.0262, -0.018), (0.0280, -0.0160), (0.0280, -0.0125),
                  (0.0268, -0.0098), (0.0225, -0.0035), (0.0155, 0.0045), (0.0085, 0.0115),
                  (0.0030, 0.018), (0, 0.018)], seg=14)


def top_stem():
    # spindle with a knob to spin it by; reaches 1.5 mm below its frame
    # toward the body face
    return lathe([(0, -0.0095), (0.0020, -0.0095), (0.0021, 0.001), (0.0029, 0.0035), (0.0030, 0.0062),
                  (0.0024, 0.0078), (0, 0.008)], seg=10)


def top_tip():
    # hard point: shoulder under the body, rounded ball point
    return lathe([(0, -0.004), (0.0030, -0.004), (0.0027, -0.0015), (0.0015, 0.0022), (0.0009, 0.0034),
                  (0, 0.004)], seg=10)


part('SmallSpinningTops__body', f_cone(0.028, 0.036), top_body)
part('SmallSpinningTops__stem', f_cyl(0.003, 0.003, 0.016), top_stem, smooth=50)
part('SmallSpinningTops__tip', f_cone(0.003, 0.008), top_tip, smooth=50)


# ------------------------------------------------------------ hula hoop --
def arc_loft(R, stations):
    """Sweep sections around Z at radius R: stations = [(angle, section)]."""
    m = len(stations[0][1])
    v, f = [], []
    for a, sec in stations:
        ca, sa = math.cos(a), math.sin(a)
        v += [((R + dr) * ca, (R + dr) * sa, dz) for dr, dz in sec]
    for i in range(len(stations) - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, (i + 1) * m + j, (i + 1) * m + j2, i * m + j2))
    f.append(tuple(reversed(range(m))))
    f.append(tuple((len(stations) - 1) * m + j for j in range(m)))
    return fixw(Shape(v, f))


def hoop_ring():
    # smooth round plastic tube
    return ring_section(0.30, circ(0.012, 8), seg=60)


def hoop_stripe():
    # wrapped tape band a hair proud of the tube, its ends eased down
    arc = math.pi / 12
    ks = [0.985, 1.0, 1.0, 1.0, 1.0, 1.0, 0.985]
    return arc_loft(0.30, [(arc * i / 6, circ(0.0126 * k, 8)) for i, k in enumerate(ks)])


part('SmallHulaHoops__hoop', f_torus(0.30, 0.012), hoop_ring, smooth=60)
part('SmallHulaHoops__stripe', f_bounds((0.2777, 0.0, -0.0125), (0.3125, 0.08088, 0.0125)), hoop_stripe, smooth=60)


# ------------------------------------------------------------ toy plane --
# fuselage: frame +y = nose (world x = 0), frame +x = world up
def plane_fuselage():
    s = lathe([(0, -0.06), (0.0080, -0.06), (0.0086, -0.056), (0.0104, -0.035), (0.0122, -0.010),
               (0.0134, 0.015), (0.0139, 0.040), (0.0140, 0.0465), (0.0132, 0.0480), (0.0140, 0.0495),
               (0.0140, 0.06), (0, 0.06)], seg=16)
    canopy = ellipsoid(0.0058, 0.012, 0.0068, seg=12).move(x=0.0110, y=0.024)
    return s + canopy


def plane_nose():
    # blunt engine cowling: cylinder behind (inside the fuselage), rounded
    # front with a cooling groove; front is world -x
    s = lathe([(0, -0.014), (0.0128, -0.014), (0.0130, -0.002), (0.0140, -0.0006), (0.0140, 0.0015), (0.0133, 0.0028),
               (0.0136, 0.0045), (0.0126, 0.0085), (0.0100, 0.0120), (0.0062, 0.0137), (0, 0.014)], seg=14)
    return s.rot(z=HPI)


def plane_tail():
    # tail cone closing the fuselage, back is world +x
    s = lathe([(0, -0.008), (0.0080, -0.008), (0.0080, -0.002), (0.0068, 0.0028), (0.0042, 0.0063),
               (0.0014, 0.0078), (0, 0.008)], seg=10)
    return s.rot(z=-HPI)


def _airfoil(c, t, n=8, te=0.0003):
    """Symmetric section, leading edge at -c/2, chord along x: [(x, y)]."""
    up = []
    for i in range(n + 1):
        u = (1 - math.cos(math.pi * i / n)) / 2
        yt = 5 * t * (0.2969 * math.sqrt(u) - 0.126 * u - 0.3516 * u ** 2 + 0.2843 * u ** 3 - 0.1036 * u ** 4)
        up.append((-c / 2 + c * u, max(yt, te)))
    lo = [(x, -y) for x, y in reversed(up[1:-1])]
    return [(x, y) for x, y in up] + lo


def plane_wing():
    # airfoil-section wing, rounded tips (span along z)
    sec = _airfoil(0.025, 0.005 / 0.98)
    zs = [-0.07, -0.0692, -0.0672, -0.064, -0.059, -0.03, 0.0, 0.03, 0.059, 0.064, 0.0672, 0.0692, 0.07]
    rings = []
    for z in zs:
        u = max(0.0, (abs(z) - 0.058) / 0.012)
        k = max(0.22, math.sqrt(max(0.0, 1 - u * u)))
        rings.append([(x * k, y * (0.4 + 0.6 * k), z) for x, y in sec])
    s = loft(rings)
    return s.fit((-0.0125, -0.0025, -0.07), (0.0125, 0.0025, 0.07))


def plane_fin():
    # swept fin with a rudder line (leading edge toward the nose, -x)
    pts = round_outline([(-0.010, -0.012), (0.010, -0.012), (0.010, 0.012), (0.002, 0.012)], 0.0025, seg=2)
    s = prism(pts, 0.004, bevel_r=0.0012, bevel_seg=2)
    return cut(s, box(0.0006, 0.022, 0.001).move(x=0.0055, y=0.001, z=0.002),
               box(0.0006, 0.022, 0.001).move(x=0.0055, y=0.001, z=-0.002))


def plane_stab():
    # swept tailplane seen from above (span along z)
    pts = round_outline([(-0.010, 0.0), (-0.004, -0.0225), (0.010, -0.0225), (0.010, 0.0225), (-0.004, 0.0225)],
                        0.0025, seg=2)
    return prism(pts, 0.003, bevel_r=0.0010, bevel_seg=1).rot(x=-HPI)


def plane_blade():
    # two-bladed paddle propeller on a hub (blades along y, thin along x)
    prof = [(0.0, 0.0024), (0.0035, 0.0017), (0.0085, 0.0025), (0.0128, 0.0019), (0.0146, 0.0009), (0.015, 0.0)]
    right = [(w, y) for y, w in prof] + [(-w, y) for y, w in reversed(prof[:-1])]
    pts = right + [(-x, -y) for x, y in right[1:-1]]
    s = prism(pts, 0.0011, bevel_r=0.0003).rot(y=HPI)
    hub = cyl(0.0027, 0.0027, 0.0032, seg=8, bevel_r=0.0006, bevel_seg=1).rot(z=HPI)
    return s + hub


part('SmallToyPlanes__fuselage', f_cyl(0.014, 0.008, 0.12), plane_fuselage)
part('SmallToyPlanes__nose', f_sphere(0.014), plane_nose)
part('SmallToyPlanes__tail', f_sphere(0.008), plane_tail)
part('SmallToyPlanes__wing', f_box(0.025, 0.005, 0.14), plane_wing, smooth=50)
part('SmallToyPlanes__fin', f_box(0.020, 0.024, 0.004), plane_fin)
part('SmallToyPlanes__stab', f_box(0.020, 0.003, 0.045), plane_stab)
part('SmallToyPlanes__blade', f_box(0.002, 0.030, 0.005), plane_blade)


# ----------------------------------------------------------- skateboard --
def skate_deck():
    # popsicle deck: round nose and tail kicked up, slight concave, eased
    # rails; lofted as cross-sections along x
    half = [0.0, 0.06, 0.108, 0.12, 0.13, 0.14, 0.148, 0.154, 0.158, 0.1598]
    xs = [-x for x in reversed(half[1:])] + half
    rings = []
    for x in xs:
        u = abs(x)
        hw = 0.04 if u <= 0.124 else 0.04 * math.sqrt(max(0.0, 1 - ((u - 0.124) / 0.036) ** 2))
        hw = max(hw, 0.0035)
        kick = 0.009 * max(0.0, (u - 0.108) / 0.052) ** 1.6
        e = min(0.0012, hw * 0.3)
        d = -0.0008
        sec = [(-hw, -0.0011), (-hw + e, -0.0038), (-hw * 0.5, -0.0040 + d * 0.75), (0, -0.0040 + d),
               (hw * 0.5, -0.0040 + d * 0.75), (hw - e, -0.0038), (hw, -0.0011), (hw - e, 0.0018),
               (hw * 0.5, 0.0020 + d * 0.75), (0, 0.0020 + d), (-hw * 0.5, 0.0020 + d * 0.75), (-hw + e, 0.0018)]
        rings.append([(x, y + kick, z) for z, y in sec])
    return loft(rings)


def skate_truck():
    # baseplate against the deck, bushing and kingpin, cast hanger, axle
    base = rbox(0.012, 0.0022, 0.024, r=0.0006, seg=1).move(y=0.0049)
    bush = cyl(0.0040, 0.0044, 0.0040, seg=8, bevel_r=0.0008, bevel_seg=1).move(y=0.0018)
    hanger = prism(round_outline([(-0.029, -0.0048), (0.029, -0.0048), (0.029, -0.0024), (0.007, -0.0004),
                                  (-0.007, -0.0004), (-0.029, -0.0024)], 0.0012, seg=1), 0.0078,
                   bevel_r=0.0006).rot(y=HPI)
    axle = cyl(0.0015, 0.0015, 0.070, seg=6, bevel_r=0).rot(x=HPI).move(y=-0.0034)
    return base + bush + hanger + axle


def skate_wheel():
    # urethane wheel: rounded tread lips, flat faces, bearing recess
    return lathe([(0, 0.0028), (0.0046, 0.0028), (0.0050, 0.0048), (0.0112, 0.0048), (0.0134, 0.0036),
                  (0.0140, 0.0012), (0.0140, -0.0012), (0.0134, -0.0036), (0.0112, -0.0048), (0.0050, -0.0048),
                  (0.0046, -0.0028), (0, -0.0028)], seg=14)


part('SmallSkateboards__deck', f_box(0.32, 0.008, 0.08), skate_deck)
part('SmallSkateboards__truck', f_box(0.012, 0.010, 0.07), skate_truck)
part('SmallSkateboards__wheel', f_cyl(0.014, 0.014, 0.010), skate_wheel, smooth=50)


# ------------------------------------------------------------- toy boat --
def boat_hull():
    # flared hull (wider at the top), round stern, rubbing strake under the
    # gunwale, a waterline groove, rounded bilge
    fl = lambda y: 0.0055 * (0.012 - y) / 0.027
    prof = [(fl(-0.015) + 0.0014, -0.015), (fl(-0.0142) + 0.0005, -0.0142), (fl(-0.0125), -0.0125),
            (fl(-0.0052), -0.0052), (fl(-0.0046) + 0.0007, -0.0046), (fl(-0.0034) + 0.0007, -0.0034),
            (fl(-0.0028), -0.0028), (fl(0.0105), 0.0105), (-0.0008, 0.0112), (-0.0008, 0.0134),
            (0.0, 0.0140), (0.0006, 0.015)]
    return ystack(rr(0.12, 0.045, 0.011, 4), prof)


def boat_bow():
    # the bow: hull sections narrowing to the stem, keel rising and sheer
    # lifting toward the tip. Frame +y = forward (world +x), frame +x = down.
    st = [(-0.020, 0.0222, -0.0165, 0.0180), (-0.012, 0.0215, -0.0168, 0.0165), (-0.006, 0.0200, -0.0172, 0.0140),
          (0.0, 0.0168, -0.0180, 0.0112), (0.006, 0.0124, -0.0189, 0.0080), (0.011, 0.0080, -0.0199, 0.0042),
          (0.015, 0.0046, -0.0208, 0.0004), (0.018, 0.0022, -0.0215, -0.0038), (0.020, 0.0008, -0.0220, -0.0080)]
    rings = []
    for y, hw, top, bot in st:
        ring = []
        for k in range(11):
            t = math.pi * k / 10
            ring.append((top + (bot - top) * math.sin(t) ** 0.6, y, -hw * math.cos(t)))
        rings.append(ring)
    return loft(rings)


def boat_deck():
    # planked deck board (front face up), 1.6 mm thick into the hull
    s = prism(rounded_rect(0.10, 0.040, 0.005, 3), 0.0016, bevel_r=0.0004).move(z=-0.0008)
    return cut(s, *[box(0.104, 0.0008, 0.0012).move(y=y) for y in (-0.01, 0.0, 0.01)])


def boat_mast():
    # dowel mast with a cap
    return lathe([(0, -0.05), (0.0027, -0.05), (0.0028, -0.048), (0.0026, 0.044), (0.0032, 0.0455),
                  (0.0032, 0.0485), (0.0022, 0.050), (0, 0.050)], seg=8)


part('SmallToyBoats__hull', f_box(0.12, 0.030, 0.045), boat_hull)
part('SmallToyBoats__bow', f_cone(0.022, 0.04), boat_bow)
part('SmallToyBoats__deck', f_bounds((-0.05, -0.02, 0), (0.05, 0.02, 0), 'plane'), boat_deck)
part('SmallToyBoats__mast', f_cyl(0.003, 0.003, 0.10), boat_mast)


# ---------------------------------------------------------------- jacks --
def jack_pin():
    # arm of a jack: swells into the hub at the centre
    return lathe([(0, -0.013), (0.0013, -0.013), (0.0015, -0.0045), (0.0021, -0.0012), (0.0021, 0.0012),
                  (0.0015, 0.0045), (0.0013, 0.013), (0, 0.013)], seg=8)


def jack_ball():
    # rubber ball with a moulding seam round the middle
    R = 0.010
    prof = []
    for t in (0, 22, 44, 66, 84, 90, 96, 114, 136, 158, 180):
        k = 0.975 if t == 90 else 1.0
        a = math.radians(t)
        prof.append((R * k * math.sin(a) if 0 < t < 180 else 0.0, -R * math.cos(a)))
    return lathe(prof, seg=14)


part('SmallJacks__pin', f_cyl(0.0018, 0.0018, 0.026), jack_pin, smooth=50)
part('SmallJacks__knob', f_sphere(0.003), lambda: sphere(0.003, seg=10, rings=6), smooth=180, weighted=False)
part('SmallJacks__ball', f_sphere(0.010), jack_ball, smooth=60)


# ---------------------------------------------------- magnetic letters --
def letter_tile():
    # pillowy plastic tile with a raised bezel round the letter disc
    body = ystack(rr(0.028, 0.028, 0.006, 2), [(0.0008, -0.005), (0, -0.0042), (0, 0.0028), (0.0010, 0.0046),
                                                 (0.0022, 0.005)])
    bezel = ring_section(0.0112, [(-0.0006, 0.0), (0.0006, 0.0), (0.0005, 0.0009), (-0.0005, 0.0009)],
                         seg=16).rot(x=-HPI).move(y=0.0048)
    return body + bezel


part('MagneticLetters__tile', f_box(0.028, 0.010, 0.028), letter_tile)


# ----------------------------------------------------------- piggy bank --
def _bump(p, c, r, h):
    """Push p outward (radially from the origin) by a soft bump of height h
    and radius r centred on direction c."""
    d = (p.normalized() - c).length
    return p + p.normalized() * h * math.exp(-(d / r) ** 2)


def piggy_body():
    # round ceramic pig (the placer stretches it 1.3x along x): flat belly,
    # haunches over the legs, a soft brow toward the snout (+x)
    s = sphere(0.045, seg=20, rings=14)
    legs = [Vector((sx * 0.0308, -0.030, sz * 0.020)).normalized() for sx in (-1, 1) for sz in (-1, 1)]
    brow = Vector((0.75, 0.55, 0.0)).normalized()
    def w(p):
        q = p.copy()
        for c in legs:
            q = _bump(q, c, 0.35, 0.0022)
        q = _bump(q, brow, 0.30, 0.0015)
        if q.y < -0.031:
            q.y = -0.031 - (-0.031 - q.y) * 0.72
        return q
    return s.warp(w)


def piggy_leg():
    # stubby tapered leg with a cloven hoof notch at the front (+x)
    s = lathe([(0, -0.009), (0.0064, -0.009), (0.0074, -0.0072), (0.0080, 0.009), (0, 0.009)], seg=10)
    return cut(s, box(0.006, 0.0042, 0.0011).move(x=0.0074, y=-0.0072))


def piggy_snout():
    # snout disc: flat front face (frame -y, world +x) for the nostrils,
    # rounded rim, flaring slightly into the face
    return lathe([(0, -0.007), (0.0116, -0.007), (0.0130, -0.0062), (0.0137, -0.0045), (0.0136, 0.001),
                  (0.0140, 0.007), (0, 0.007)], seg=16)


def piggy_slot():
    # coin slot with rounded ends
    return prism(rounded_rect(0.024, 0.005, 0.0024, 3), 0.001).rot(x=-HPI)


def piggy_tail():
    # curly tail: tapering, curling inward at the tip
    pts = []
    for a, r in ((0, 0.008), (40, 0.008), (85, 0.008), (130, 0.008), (180, 0.008), (225, 0.008), (270, 0.0079),
                 (305, 0.0068), (335, 0.0056), (355, 0.0046)):
        t = math.radians(a)
        pts.append((r * math.cos(t), r * math.sin(t), 0.0))
    return tube(pts, 0.0021, seg=5, r_end=0.0012)


part('SmallPiggyBanks__body', f_sphere(0.045), piggy_body, smooth=180, weighted=False)
part('SmallPiggyBanks__leg', f_cyl(0.008, 0.008, 0.018), piggy_leg, smooth=50)
part('SmallPiggyBanks__snout', f_cyl(0.014, 0.014, 0.014), piggy_snout, smooth=50)
part('SmallPiggyBanks__eye', f_sphere(0.003), lambda: sphere(0.003, seg=10, rings=6), smooth=180, weighted=False)
part('SmallPiggyBanks__slot', f_box(0.024, 0.001, 0.005), piggy_slot)
part('SmallPiggyBanks__tail', f_bounds((-0.010, -0.010, -0.002), (0.010, 0.010, 0.002)), piggy_tail,
     smooth=180, weighted=False)


# -------------------------------------------------------------- toy dino --
def dino_torso():
    # T-rex torso (stretched 1.6x along x by the placer): a row of scutes
    # down the spine, soft hips over the legs, flatter belly
    s = sphere(0.028, seg=14, rings=10).rot(z=HPI)
    def w(p):
        q = p.copy()
        for k in range(5):
            c = Vector((math.cos(math.radians(60 + 15 * k)), math.sin(math.radians(60 + 15 * k)), 0.0))
            q = _bump(q, c, 0.12, 0.0022)
        for sz in (-1, 1):
            q = _bump(q, Vector((0.05, -0.45, sz * 0.9)).normalized(), 0.45, 0.0016)
        if q.y < -0.02:
            q.y = -0.02 - (-0.02 - q.y) * 0.8
        return q
    return s.warp(w)


def dino_head():
    # head (stretched 1.5x along x): brow ridges over sunk eye sockets, a
    # flatter jaw line underneath
    s = sphere(0.020, seg=14, rings=10).rot(z=HPI)
    eyes = [Vector((0.0033, 0.010, sz * 0.014)).normalized() for sz in (-1, 1)]
    brows = [Vector((0.002, 0.016, sz * 0.011)).normalized() for sz in (-1, 1)]
    def w(p):
        q = p.copy()
        for c in eyes:
            q = _bump(q, c, 0.22, -0.0016)
        for c in brows:
            q = _bump(q, c, 0.25, 0.0012)
        if q.y < -0.013:
            q.y = -0.013 - (-0.013 - q.y) * 0.7
        return q
    return s.warp(w)


def dino_snout():
    # rounded muzzle with the mouth slit between the jaws and two nostrils
    s = rbox(0.020, 0.014, 0.014, r=0.0042, seg=1)
    return cut(s, box(0.013, 0.0009, 0.02).move(x=0.0045, y=-0.0022))


def dino_arm():
    # little arm, bent forward, two claws on the +y end
    s = lathe([(0, -0.006), (0.0030, -0.006), (0.0036, -0.004), (0.0032, 0.0015), (0.0034, 0.0035),
               (0.0022, 0.0052), (0, 0.0056)], seg=8)
    claws = [cone(0.0011, 0.003, seg=4).rot(z=0.25).move(x=0.0016, y=0.0045, z=sz * 0.0016) for sz in (-1, 1)]
    return s + claws[0] + claws[1]


def _ell(cx, rx, rz, y, n=12, lobes=0.0):
    pts = []
    for i in range(n):
        a = TAU * i / n
        k = 1.0 + (lobes * max(0.0, math.cos(3 * a)) if math.cos(a) > 0.3 else 0.0)
        pts.append((cx + rx * math.cos(a) * k, y, rz * math.sin(a)))
    return pts


def dino_leg():
    # big thigh, shin, a three-toed foot forward (+x); the sole reaches
    # 2.9 mm below the frame toward the floor
    return loft([_ell(0.0012, 0.0055, 0.0050, -0.0179, lobes=0.12), _ell(0.0012, 0.0059, 0.0056, -0.0165, lobes=0.12),
                 _ell(0.0005, 0.0052, 0.0052, -0.0128, lobes=0.08), _ell(-0.0008, 0.0040, 0.0042, -0.0105),
                 _ell(-0.0006, 0.0042, 0.0044, -0.002), _ell(0.0, 0.0068, 0.0066, 0.005),
                 _ell(0.0, 0.0066, 0.0064, 0.011), _ell(0.0, 0.0050, 0.0050, 0.0150)])


def dino_tail():
    # tapering tail (frame +y = tip, +x = world up), gently S-curved, with
    # the spine scutes carried on
    rings = []
    for i in range(10):
        t = i / 9
        y = -0.03 + 0.06 * t
        r = 0.014 * (1 - t) ** 0.9 + 0.0012
        cx = -0.0035 * math.sin(math.pi * t) + 0.004 * t ** 3
        ring = []
        for k in range(10):
            a = TAU * k / 10
            bump = 0.0016 * (1 - t) if k == 0 and i % 2 == 1 else 0.0
            ring.append((cx + (r + bump) * math.cos(a), y, r * math.sin(a)))
        rings.append(ring)
    return loft(rings).fit((-0.014, -0.03, -0.014), (0.014, 0.03, 0.014))


part('SmallToyDinos__torso', f_sphere(0.028), dino_torso, smooth=180, weighted=False)
part('SmallToyDinos__head', f_sphere(0.020), dino_head, smooth=180, weighted=False)
part('SmallToyDinos__snout', f_box(0.020, 0.014, 0.014), dino_snout, smooth=60)
part('SmallToyDinos__arm', f_cyl(0.004, 0.004, 0.012), dino_arm, smooth=60)
part('SmallToyDinos__leg', f_box(0.014, 0.030, 0.014), dino_leg, smooth=70)
part('SmallToyDinos__tail', f_cone(0.014, 0.06), dino_tail, smooth=70)


# ------------------------------------------------------------- toy phone --
def phone_base():
    # chunky moulded base: big corner radii, a shell seam round the sides,
    # soft top edge, small foot chamfer
    return ystack(rr(0.09, 0.07, 0.013, 3), [(0.0015, -0.010), (0, -0.0085), (0, 0.0040), (0.0007, 0.0047),
                                               (0, 0.0054), (0, 0.0074), (0.0012, 0.0092), (0.0030, 0.010)])


def phone_button():
    # domed push button
    return lathe([(0, -0.001), (0.0050, -0.001), (0.0050, 0.0), (0.0041, 0.0008), (0.0022, 0.001), (0, 0.001)],
                 seg=12)


def phone_handset():
    # handset grip (frame +x = world up): oval section, arching 2 mm up in
    # the middle between the ear and mouth cups
    sec = rounded_rect(0.016, 0.019, 0.006, 2)
    rings = []
    for i in range(9):
        y = -0.0375 + 0.075 * i / 8
        c = -0.002 + 0.004 * math.cos(math.pi * y / 0.075)
        rings.append([(x + c, y, z) for x, z in sec])
    return loft(rings)


def phone_cup():
    # ear/mouth cup: flat underside resting on the base, domed back with a
    # seam groove
    return lathe([(0, -0.010), (0.0104, -0.010), (0.0117, -0.0086), (0.0120, -0.0062), (0.0114, -0.0050),
                  (0.0118, -0.0040), (0.0112, 0.0025), (0.0092, 0.0074), (0.0058, 0.0108), (0, 0.012)], seg=14)


part('SmallToyPhones__base', f_box(0.09, 0.020, 0.07), phone_base)
part('SmallToyPhones__button', f_cyl(0.005, 0.005, 0.002), phone_button, smooth=50)
part('SmallToyPhones__handset', f_cyl(0.010, 0.010, 0.075), phone_handset, smooth=60)
part('SmallToyPhones__cup', f_sphere(0.012), phone_cup, smooth=50)


# ------------------------------------------------------------ toy robot --
def robot_torso():
    # tin robot chest: chamfered box, chest panel line with four rivets,
    # a waist groove
    s = rbox(0.036, 0.040, 0.024, r=0.0022, seg=1)
    panel = cut(box(0.026, 0.024, 0.002), box(0.024, 0.022, 0.004)).move(y=0.003, z=0.012)
    waist = cut(box(0.04, 0.0012, 0.03), box(0.0344, 0.004, 0.0224)).move(y=-0.0115)
    s = cut(s, panel, waist)
    rivet = lathe([(0, 0), (0.0011, 0), (0.0008, 0.0006), (0, 0.0008)], seg=6).rot(x=HPI)
    for sx in (-1, 1):
        for sy in (-1, 1):
            s = s + rivet.copy().move(x=sx * 0.0103, y=0.003 + sy * 0.0093, z=0.0119)
    return s


def robot_head():
    # boxy head: sunk visor band across the eyes, bolt "ears" either side
    s = rbox(0.024, 0.024, 0.022, r=0.0018, seg=1)
    s = cut(s, rbox(0.018, 0.0085, 0.004, r=0.0012, seg=1).move(y=0.003, z=0.0115))
    ear = lathe([(0, 0), (0.0040, 0), (0.0040, 0.0010), (0.0028, 0.0020), (0, 0.0020)], seg=10)
    return s + ear.copy().rot(z=-HPI).move(x=0.0119) + ear.rot(z=HPI).move(x=-0.0119)


def robot_antenna():
    # wire antenna with a collar where it leaves the head
    return lathe([(0, -0.010), (0.00145, -0.010), (0.00145, -0.0082), (0.0010, -0.0074), (0.0010, 0.010),
                  (0, 0.010)], seg=6)


def robot_arm():
    # ball shoulder, ringed arm, pincer claw at the bottom
    shoulder = sphere(0.0048, seg=8, rings=5).move(y=0.0092)
    arm = lathe([(0, -0.0090), (0.0036, -0.0090), (0.0042, -0.0070), (0.0038, -0.0012), (0.0042, 0.0002),
                 (0.0040, 0.0090), (0, 0.0090)], seg=8)
    claw = ring_section(0.0034, circ(0.0014, 4, a0=math.pi / 4), seg=8, arc=TAU * 0.72,
                        a0=-math.pi / 2 + TAU * 0.14).rot(y=HPI).move(y=-0.0106)
    return shoulder + arm + claw


def robot_leg():
    # ribbed leg on a chamfered boot
    boot = rbox(0.014, 0.0065, 0.014, r=0.0016, seg=1).move(y=-0.00675)
    leg = lathe([(0.0052, -0.004), (0.0052, 0.0035), (0.0045, 0.0042), (0.0052, 0.0049), (0.0052, 0.010)],
                seg=7, cap_bottom=False, cap_top=False)     # ends hidden in the boot and the torso
    return boot + leg


part('SmallToyRobots__torso', f_box(0.036, 0.040, 0.024), robot_torso)
part('SmallToyRobots__head', f_box(0.024, 0.024, 0.022), robot_head)
part('SmallToyRobots__antenna', f_cyl(0.001, 0.001, 0.020), robot_antenna)
part('SmallToyRobots__arm', f_box(0.010, 0.028, 0.010), robot_arm)
part('SmallToyRobots__leg', f_box(0.014, 0.020, 0.014), robot_leg)


# ------------------------------------------------------- toy microphone --
def mic_handle():
    # tapered handle (frame +y = head end, +x = world up): rounded butt,
    # grip rings, a slide switch on top
    s = lathe([(0, -0.05), (0.0098, -0.05), (0.0118, -0.047), (0.0120, -0.030), (0.0116, -0.028),
               (0.0119, -0.024), (0.0115, -0.022), (0.0118, -0.018), (0.0112, 0.030), (0.0100, 0.05), (0, 0.05)],
              seg=12)
    switch = rbox(0.004, 0.011, 0.0055, r=0.0012, seg=1).move(x=0.0104, y=0.022)
    return s + switch


def mic_grille():
    # ball grille, ribbed like a wound wire basket (axis along the handle)
    R = 0.020
    prof = [(0, -R)]
    for i in range(1, 9):
        a = math.pi * i / 9
        k = 0.955 if i % 2 == 0 else 1.0
        prof.append((R * k * math.sin(a), -R * k * math.cos(a)))
    return lathe(prof + [(0, R)], seg=16).rot(z=HPI)


def mic_collar():
    # joint ring between grille and handle: rounded edges and a bead
    return lathe([(0, -0.002), (0.0126, -0.002), (0.0136, -0.0014), (0.0134, -0.0004), (0.0140, 0.0),
                  (0.0134, 0.0004), (0.0136, 0.0014), (0.0126, 0.002), (0, 0.002)], seg=16)


part('SmallToyMicrophones__handle', f_cyl(0.010, 0.012, 0.10), mic_handle)
part('SmallToyMicrophones__grille', f_sphere(0.020), mic_grille, smooth=70)
part('SmallToyMicrophones__collar', f_cyl(0.014, 0.014, 0.004), mic_collar, smooth=50)


# ------------------------------------------------------------ toy piano --
def piano_case():
    # rounded case: key cheeks rising at both ends, a key slip along the
    # front edge, a row of little speaker holes in the front
    s = ystack(rr(0.22, 0.08, 0.008, 3), [(0.0012, -0.012), (0, -0.0105), (0, 0.0105), (0.0012, 0.012)])
    for sx in (-1, 1):
        s = s + rbox(0.0095, 0.012, 0.070, r=0.0022, seg=2).move(x=sx * 0.10525, y=0.011, z=0.004)
    s = s + rbox(0.201, 0.0045, 0.011, r=0.0014, seg=1).move(y=0.0118, z=0.0335)
    holes = [cyl(0.0022, 0.0022, 0.006, seg=8, bevel_r=0).rot(x=HPI).move(x=0.012 * k, y=-0.0015, z=0.040)
             for k in range(-2, 3)]
    return cut(s, *holes)


def piano_back():
    # upright back panel: arched top, a sunk panel on the player's side (+z)
    # and two round sound holes
    arch = [(0.11 * math.cos(math.pi * i / 10), 0.017 + 0.008 * math.sin(math.pi * i / 10)) for i in range(11)]
    pts = [(-0.11, -0.025), (0.11, -0.025)] + arch
    s = prism(pts, 0.014, bevel_r=0.0018, bevel_seg=1)
    inner = [(-0.095, -0.012), (0.095, -0.012)] + [(0.095 * math.cos(math.pi * i / 8),
                                                     0.010 + 0.007 * math.sin(math.pi * i / 8)) for i in range(9)]
    panel = prism(inner, 0.004).move(z=0.0078)
    holes = [cyl(0.0065, 0.0065, 0.02, seg=12, bevel_r=0).rot(x=HPI).move(x=sx * 0.062, y=0.0035)
             for sx in (-1, 1)]
    return cut(s, panel, *holes)


def piano_white_key():
    # white key: eased sides, rounded top edge
    return ystack(rr(0.02707, 0.054, 0.0014, 2), [(0.0004, -0.004), (0, -0.0034), (0, 0.0026), (0.0006, 0.0036),
                                                   (0.0014, 0.004)])


def piano_black_key():
    # raised black key with sloping sides and a rounded top
    return ystack(rr(0.01714, 0.033, 0.0012, 2), [(0.0, -0.005), (0.0006, 0.0025), (0.0012, 0.0042),
                                                   (0.0022, 0.005)])


part('SmallToyPianos__case', f_box(0.22, 0.024, 0.08), piano_case)
part('SmallToyPianos__back', f_box(0.22, 0.05, 0.014), piano_back)
part('SmallToyPianos__whitekey', f_box(0.02707, 0.008, 0.054), piano_white_key)
part('SmallToyPianos__blackkey', f_box(0.01714, 0.010, 0.033), piano_black_key)


# -------------------------------------------------------- princess wand --
def wand_stick():
    # twisted plastic rod: four-lobed section turning along the length, a
    # collar under the star (frame -y) and a round knob at the handle end
    stations = [(-0.090, 0.0030, 0), (-0.0885, 0.0040, 0), (-0.0815, 0.0040, 0), (-0.0800, 0.0032, 1)]
    stations += [(-0.078 + 0.156 * i / 13, 0.0033, 1) for i in range(14)]
    stations += [(0.0800, 0.0034, 0), (0.0835, 0.0040, 0), (0.0875, 0.0037, 0), (0.0900, 0.0016, 0)]
    rings = []
    for y, r, tw in stations:
        ring = []
        for k in range(12):
            a = TAU * k / 12
            rr_ = r * (1 + (0.18 * math.cos(4 * a + y * 70.0) if tw else 0.0))
            ring.append((rr_ * math.cos(a), y, rr_ * math.sin(a)))
        rings.append(ring)
    return loft(rings).fit((-0.004, -0.09, -0.004), (0.004, 0.09, 0.004))


part('SmallPrincessWands__stick', f_cyl(0.004, 0.004, 0.18), wand_stick, smooth=60)


# ------------------------------------------------------------ toy sword --
def _blade_sec(k=1.0, t=1.0):
    """Blade section (y, z): blunt toy edges, bevels, a raised mid-ridge.
    k scales the height, t the thickness."""
    return [(0.010 * k, 0.0008 * t), (0.0045 * k, 0.0026 * t), (0.0, 0.0040 * t), (-0.0045 * k, 0.0026 * t),
            (-0.010 * k, 0.0008 * t), (-0.010 * k, -0.0008 * t), (-0.0045 * k, -0.0026 * t), (0.0, -0.0040 * t),
            (0.0045 * k, -0.0026 * t), (0.010 * k, -0.0008 * t)]


def sword_blade():
    # ridged plastic blade on its edge (height y, thickness z)
    rings = [[(x, y, z) for y, z in _blade_sec(kk, tt)]
             for x, kk, tt in ((-0.14, 0.95, 0.8), (-0.138, 1.0, 1.0), (0.14, 1.0, 1.0))]
    return loft(rings)


def sword_tip():
    # the point, continuing the blade section; frame +y = the point,
    # frame -x = world up (the blade is 5 mm thick, so the tip stays thin)
    rings = []
    for y, k in ((-0.010, 1.0), (-0.005, 0.86), (0.0, 0.66), (0.005, 0.40), (0.0085, 0.16), (0.010, 0.03)):
        rings.append([(-yy, y, zz) for yy, zz in _blade_sec(k, 0.25 + 0.75 * k)])
    return loft(rings)


def sword_guard():
    # crossguard: chamfered centre block, round arms with ball ends
    block = rbox(0.008, 0.012, 0.010, r=0.0015, seg=1)
    arms = lathe([(0, -0.018), (0.0025, -0.0172), (0.0040, -0.0157), (0.0044, -0.0136), (0.0038, -0.0116),
                  (0.0027, -0.0102), (0.0027, 0.0102), (0.0038, 0.0116), (0.0044, 0.0136), (0.0040, 0.0157),
                  (0.0025, 0.0172), (0, 0.018)], seg=10)
    return block + arms


def sword_grip():
    # moulded grip rings along the handle
    prof = [(0, -0.025), (0.0074, -0.025)]
    for k in range(6):
        prof.append((0.0080, -0.0215 + k * 0.0086))
        if k < 5:
            prof.append((0.0071, -0.0172 + k * 0.0086))
    return lathe(prof + [(0.0074, 0.025), (0, 0.025)], seg=10)


def sword_pommel():
    # onion pommel with a neck toward the grip (+x) and a button (-x)
    s = lathe([(0, -0.010), (0.0060, -0.010), (0.0064, -0.0085), (0.0060, -0.0072), (0.0086, -0.0042),
               (0.0100, -0.0004), (0.0092, 0.0036), (0.0062, 0.0068), (0.0030, 0.0082), (0.0030, 0.0092),
               (0.0016, 0.010), (0, 0.010)], seg=12)
    return s.rot(z=HPI)


part('SmallToySwords__blade', f_box(0.28, 0.020, 0.005), sword_blade)
part('SmallToySwords__tip', f_cone(0.010, 0.020), sword_tip)
part('SmallToySwords__guard', f_box(0.008, 0.036, 0.010), sword_guard)
part('SmallToySwords__grip', f_cyl(0.008, 0.008, 0.05), sword_grip, smooth=180, weighted=False)
part('SmallToySwords__pommel', f_sphere(0.010), sword_pommel, smooth=50)


# ------------------------------------------------------------ doll head --
# head sphere is scaled (1, 1.1, 0.95) by the placer; face at +z
def doll_head():
    # porcelain head: sunk eye sockets for the glass eyes, brows, a small
    # nose, round cheeks, the lips brought forward to the mouth quad, chin
    s = sphere(0.045, seg=20, rings=14)
    d = lambda x, y, z: Vector((x, y, z)).normalized()
    bumps = [(d(sx * 0.014, 0.0064, 0.040), 0.17, -0.0020) for sx in (-1, 1)]
    bumps += [(d(sx * 0.014, 0.0145, 0.037), 0.16, 0.0009) for sx in (-1, 1)]
    bumps += [(d(0, -0.004, 1), 0.12, 0.0022), (d(0, -0.30, 0.95), 0.14, 0.0019), (d(0, -0.62, 0.78), 0.2, 0.0010)]
    bumps += [(d(sx * 0.55, -0.35, 0.76), 0.33, 0.0014) for sx in (-1, 1)]
    def w(p):
        q = p.copy()
        for c, r, h in bumps:
            q = _bump(q, c, r, h)
        return q
    return s.warp(w)


def doll_hair():
    # sculpted hair cap: full crown (covers the head top), strand ridges,
    # a fringe dipping over the forehead, rolled rim inside
    prof = [(0.0, 0.0412)]
    for ph in (80, 62, 44, 26, 10, 0):
        a = math.radians(ph)
        prof.append((0.0402 * math.cos(a) ** 0.8, 0.0412 * math.sin(a)))
    prof.append((0.0372, 0.0012))
    s = lathe(prof, seg=20, cap_bottom=False, cap_top=False)
    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6:
            return p
        a = math.atan2(p.x, p.z)
        k = 1.0 + (0.022 * math.cos(10 * a) if p.y > 0.004 else 0.0)
        y = p.y
        if p.y < 0.012:
            y -= 0.0034 * max(0.0, math.cos(a)) ** 3 * (1 - p.y / 0.012)
        return Vector((p.x * k, y, p.z * k))
    return s.warp(w)


def doll_neck():
    # torn cloth neck stub, ragged at the bottom
    s = lathe([(0, 0.005), (0.0140, 0.005), (0.0128, -0.001), (0.0104, -0.0038), (0, -0.0038)], seg=12)
    return s.warp(lambda p: Vector((p.x, p.y - (0.0012 * abs(math.sin(7 * math.atan2(p.x, p.z))) if p.y < -0.003 else 0), p.z)))


part('DollHeads__head', f_sphere(0.045), doll_head, smooth=180, weighted=False)
part('DollHeads__hair', f_bounds((-0.04, 0, -0.04), (0.04, 0.04, 0.04)), doll_hair, smooth=70)
part('DollHeads__neck', f_cyl(0.014, 0.010, 0.010), doll_neck, smooth=50)


# ------------------------------------------------------------ crib side --
def crib_base_rail():
    # bottom rail: eased edges, a bead line along both faces
    s = rbox(0.44, 0.016, 0.020, r=0.0025, seg=2)
    return cut(s, *[box(0.45, 0.0012, 0.0012).move(y=0.003, z=sz * 0.010) for sz in (-1, 1)])


def crib_top_rail():
    # top (teething) rail: flat underside, round-topped section
    sec = [(-0.010, -0.008), (0.010, -0.008)] + \
          [(0.010 * math.cos(math.pi * i / 8), -0.0015 + 0.0095 * math.sin(math.pi * i / 8)) for i in range(9)]
    return prism(round_outline(sec, 0.0015, seg=1), 0.44, bevel_r=0.0015).rot(y=HPI)


def crib_spindle():
    # turned spindle with beads near both ends; a tenon runs 5 mm past the
    # bottom of the frame into the base rail
    return lathe([(0, -0.205), (0.0030, -0.205), (0.0030, -0.192), (0.0048, -0.190), (0.0050, -0.186),
                  (0.0040, -0.181), (0.0036, -0.150), (0.0043, -0.050), (0.0045, 0.0), (0.0043, 0.050),
                  (0.0036, 0.150), (0.0040, 0.181), (0.0050, 0.186), (0.0048, 0.190), (0.0036, 0.194),
                  (0.0036, 0.200), (0, 0.200)], seg=10)


def crib_post():
    # square corner post (eased edges) turned into a vase neck above the top
    # rail, with a collar under the finial
    sq = rbox(0.026, 0.43, 0.026, r=0.0035, seg=2).move(y=-0.035)
    neck = lathe([(0, 0.176), (0.0110, 0.176), (0.0122, 0.182), (0.0090, 0.192), (0.0078, 0.205),
                  (0.0100, 0.222), (0.0086, 0.236), (0.0120, 0.242), (0.0126, 0.2475), (0.0110, 0.250),
                  (0, 0.250)], seg=16)
    return sq + neck


def crib_finial():
    # turned knob: neck into the post, collar, ball, small tip
    return lathe([(0, -0.018), (0.0060, -0.018), (0.0060, -0.0140), (0.0086, -0.0122), (0.0062, -0.0098),
                  (0.0110, -0.0060), (0.0146, -0.0010), (0.0150, 0.0030), (0.0132, 0.0090), (0.0080, 0.0145),
                  (0.0034, 0.0162), (0.0024, 0.0180), (0, 0.018)], seg=12)


part('BabyCribFragments__baserail', f_box(0.44, 0.016, 0.020), crib_base_rail)
part('BabyCribFragments__toprail', f_box(0.44, 0.016, 0.020), crib_top_rail)
part('BabyCribFragments__spindle', f_cyl(0.005, 0.005, 0.40), crib_spindle, smooth=50)
part('BabyCribFragments__post', f_cyl(0.014, 0.014, 0.50), crib_post, smooth=50)
part('BabyCribFragments__finial', f_sphere(0.018), crib_finial, smooth=50)


# ------------------------------------------------------ doll body parts --
# the same parts serve arms (r 10 mm) and legs (r 14 mm); frame +y is the
# socket end (world -x after rotation.z = PI/2), -y the hand/foot end
def doll_limb():
    # porcelain limb: round joint end, full upper part, elbow/knee waist,
    # forearm/calf swell, slim wrist/ankle; slightly bent
    tab = [(-0.036, 0.0074), (-0.032, 0.0078), (-0.022, 0.0092), (-0.010, 0.0102), (0.002, 0.0094),
           (0.012, 0.0108), (0.022, 0.0120), (0.029, 0.0118), (0.0335, 0.0098), (0.0355, 0.0060), (0.036, 0.0)]
    rings = []
    for y, r in tab:
        r = max(r, 0.0012)
        dz = 0.0022 * math.sin(math.pi * (y + 0.036) / 0.072)
        rings.append([(r * math.cos(TAU * k / 12), y, dz + r * math.sin(TAU * k / 12)) for k in range(12)])
    return loft(rings)


def doll_socket():
    # dark torn socket: a rim round a hollow, ragged on its outer face (+y)
    s = lathe([(0, -0.003), (0.0134, -0.003), (0.0140, -0.0022), (0.0140, 0.0016), (0.0128, 0.003),
               (0.0076, 0.003), (0.0066, 0.0006), (0, 0.0)], seg=16)
    def rag(p):
        if p.y > 0.0025 and math.hypot(p.x, p.z) > 0.007:
            a = math.atan2(p.x, p.z)
            p.y -= 0.0014 * abs(math.sin(5 * a) * math.cos(3 * a))
        return p
    return s.warp(rag)


def doll_hand():
    # mitten hand/foot: rounded lump, flatter toward the fingers (+x), a
    # thumb bulge, narrower at the wrist (-x) where it meets the limb
    s = sphere(0.017, seg=14, rings=10).rot(z=HPI)
    thumb = Vector((0.25, 0.35, 0.9)).normalized()
    def w(p):
        q = p.copy()
        if q.x > 0:
            q.y *= 1 - 0.18 * (q.x / 0.017) ** 2
        if q.x < -0.008:
            k = 1 - 0.22 * ((-q.x - 0.008) / 0.009)
            q.y *= k
            q.z *= k
        return _bump(q, thumb, 0.35, 0.0016)
    return s.warp(w)


part('DollBodyParts__limb', f_cyl(0.012, 0.012, 0.072), doll_limb, smooth=180, weighted=False)
part('DollBodyParts__socket', f_cyl(0.014, 0.014, 0.006), doll_socket, smooth=40)
part('DollBodyParts__hand', f_sphere(0.017), doll_hand, smooth=180, weighted=False)


# ----------------------------------------------------- single child shoe --
def _offset2d(pts, d):
    """Offset a CCW outline inward by d (mitred)."""
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


def _foot(L, W, n=9):
    """Plan outline of a small shoe sole, heel at -x, toe at +x."""
    base = [(-1.0, 0.80), (-0.6, 0.86), (-0.2, 0.80), (0.3, 1.0), (0.65, 0.98), (1.0, 0.86)]
    def w(u):
        for (a, wa), (b, wb) in zip(base, base[1:]):
            if u <= b:
                return wa + (wb - wa) * (u - a) / (b - a)
        return base[-1][1]
    xs = [-math.cos(math.pi * i / n) for i in range(n + 1)]
    right = [(u * L, W * w(u) * (1 - abs(u) ** 2.6) ** (1 / 2.6)) for u in xs]
    return _ccw(right + [(x, -z) for x, z in reversed(right[1:-1])])


def shoe_sole():
    # rubber outsole: foot outline, rounded bottom edge, tread grooves
    o = _foot(0.05, 0.025)
    s = ystack(lambda i: _offset2d(o, i), [(0.0012, -0.005), (0, -0.0036), (0, 0.0042), (0.0008, 0.005)])
    return cut(s, *[box(0.0022, 0.003, 0.06).move(x=x, y=-0.0056) for x in (-0.036, -0.026, 0.012, 0.022, 0.032)])


def shoe_toe():
    # vamp and toe box (the placer squashes it to 0.6 in y): open lace throat
    # with the tongue at its floor, so the lace X shows; round toe at +x
    st = [(-0.028, 0.026, 0.027), (-0.018, 0.028, 0.0278), (-0.006, 0.0282, 0.028), (0.005, 0.027, 0.0272),
          (0.013, 0.023, 0.0255), (0.019, 0.017, 0.0228), (0.0235, 0.009, 0.0188), (0.0265, 0.000, 0.0132),
          (0.0278, -0.009, 0.0072), (0.028, -0.016, 0.0025)]
    rings = []
    for x, top, hw in st:
        thr = x < 0.012
        ring = []
        for k in range(13):
            t = math.pi * k / 12
            z = hw * math.cos(t)
            y = -0.028 + (top + 0.028) * math.sin(t) ** 0.45
            if thr and abs(z) < 0.0105:
                u = abs(z) / 0.0105
                y = 0.0145 + (y - 0.0145) * u ** 3 if y > 0.0145 else y
            ring.append((x, y, z))
        rings.append(ring)
    return loft(rings)


def shoe_heel():
    # heel counter and padded collar round the foot opening (open to the
    # front where the throat continues), a pull tab at the back
    back = [(-0.003 + 0.017 * math.cos(a), 0.024 * math.sin(a))
            for a in [math.pi / 2 + math.pi * i / 10 for i in range(11)]]
    o = _ccw(back + [(0.020, -0.024), (0.020, 0.024)])
    s = ystack(lambda i: _offset2d(o, i), [(0.0006, -0.012), (0, -0.011), (0, 0.009), (0.0012, 0.0115),
                                           (0.0030, 0.012)])
    hole = ystack(lambda i: _offset2d(o, i), [(0.0052, -0.002), (0.0052, 0.02)]).move(x=0.012)
    tab = rbox(0.004, 0.009, 0.010, r=0.0015, seg=1).move(x=-0.0185, y=0.0095)
    return cut(s, hole) + tab


def shoe_lace():
    # flat cord with rounded ends
    s = lathe([(0, -0.010), (0.0006, -0.0098), (0.0010, -0.009), (0.0010, 0.009), (0.0006, 0.0098), (0, 0.010)],
              seg=8)
    return s.rot(z=HPI).scale(1, 0.5, 1)


part('SingleChildShoes__sole', f_box(0.10, 0.010, 0.05), shoe_sole)
part('SingleChildShoes__toe', f_sphere(0.028), shoe_toe, smooth=60)
part('SingleChildShoes__heel', f_box(0.040, 0.024, 0.048), shoe_heel, smooth=50)
part('SingleChildShoes__lace', f_box(0.020, 0.001, 0.002), shoe_lace, smooth=60)


# ---------------------------------------------------------- teddy bears --
def _plush(s, amount=0.0005, seed=0):
    return noisy(s, amount, scale=90, seed=seed)


def teddy_body():
    # plush body (the placer stretches it to 1.2 in y): pear-shaped, belly
    # forward (+z), a seam down the front and back, flat seat so it sits on
    # the floor instead of sinking
    s = sphere(0.045, seg=20, rings=14)
    belly = Vector((0, -0.2, 1)).normalized()
    def w(p):
        q = p.copy()
        if q.y < 0:
            k = 1 + 0.05 * (-q.y / 0.045)
            q.x *= k
            q.z *= k
        q = _bump(q, belly, 0.45, 0.0022)
        if abs(q.x) < 0.001:
            q.z -= math.copysign(0.0011, q.z)
        if q.y < -0.0375:
            q.y = -0.0375 - (-0.0375 - q.y) * 0.25
        return q
    return _plush(s.warp(w), seed=1)


def teddy_head():
    # plush head: gusset seam over the crown (front to back), fuller cheeks,
    # the eye band a little flatter so the button eye and the empty socket
    # both show
    s = sphere(0.035, seg=20, rings=14)
    def w(p):
        q = p.copy()
        for sx in (-1, 1):
            q = _bump(q, Vector((sx * 0.6, -0.25, 0.75)).normalized(), 0.35, 0.0016)
        if abs(q.x) < 0.001 and q.y > 0.012:
            q.y -= 0.0010
        if q.z > 0.028 and 0.0 < q.y < 0.017:
            q.z = 0.028 + (q.z - 0.028) * 0.45
        return q
    return _plush(s.warp(w), 0.0004, seed=2)


def teddy_muzzle():
    # muzzle with the stitched groove running down from the nose
    s = sphere(0.018, seg=14, rings=10)
    def w(p):
        q = p.copy()
        if abs(q.x) < 0.001 and q.z > 0.004 and q.y < 0.002:
            q.z -= 0.0010
        return q
    return s.warp(w)


def teddy_nose():
    # stitched nose pad: rounded triangle, broad on top
    s = sphere(0.005, seg=10, rings=6)
    return s.warp(lambda p: Vector((p.x * (0.70 + 0.45 * (p.y + 0.005) / 0.010), p.y, p.z * 0.92)))


def teddy_ear():
    # thick round ear, cupped on the front (+z), flat where it meets the head
    s = lathe([(0, 0.0), (0.0120, 0.0), (0.0116, 0.0032), (0.0104, 0.0062), (0.0082, 0.0088),
               (0.0050, 0.0110), (0, 0.012)], seg=20)
    def w(p):
        q = Vector((p.x, p.y, p.z * 0.82))
        if q.z > 0:
            q.z -= 0.0045 * max(0.0, 1 - (q.x / 0.0082) ** 2 - ((q.y - 0.0062) / 0.0056) ** 2)
        return q
    return s.warp(w)


def teddy_eye():
    # safety-eye button: domed front (+z) with a small rim
    s = lathe([(0, -0.004), (0.0034, -0.0020), (0.0040, 0.0005), (0.0033, 0.0012), (0.0026, 0.0032),
               (0, 0.004)], seg=10)
    return s.rot(x=HPI)


def teddy_limb():
    # stuffed limb (stretched 1.3 in y): bean-shaped, the paw end curling
    # forward, soft lumps
    s = sphere(0.014, seg=14, rings=10)
    def w(p):
        q = p.copy()
        if q.y < 0:
            q.z += 0.0022 * (-q.y / 0.014) ** 2
            k = 1 + 0.06 * (-q.y / 0.014)
            q.x *= k
        return q
    return _plush(s.warp(w), 0.0004, seed=3).fit((-0.014, -0.014, -0.014), (0.014, 0.014, 0.014))


part('WornTeddyBears__body', f_sphere(0.045), teddy_body, smooth=180, weighted=False)
part('WornTeddyBears__head', f_sphere(0.035), teddy_head, smooth=180, weighted=False)
part('WornTeddyBears__muzzle', f_sphere(0.018), teddy_muzzle, smooth=180, weighted=False)
part('WornTeddyBears__nose', f_sphere(0.005), teddy_nose, smooth=180, weighted=False)
part('WornTeddyBears__ear', f_bounds((-0.012, 0, -0.012), (0.012, 0.012, 0.012)), teddy_ear, smooth=180,
     weighted=False)
part('WornTeddyBears__eye', f_sphere(0.004), teddy_eye, smooth=60)
part('WornTeddyBears__limb', f_sphere(0.014), teddy_limb, smooth=180, weighted=False)


# ------------------------------------------------------------ teddy paw --
def teddy_paw():
    # torn-off plush paw (stretched 1.4 in y): paw pad bulging at the
    # bottom front, the sole flattened where it rests, the top ripped open
    # into a ragged crater
    s = sphere(0.020, seg=14, rings=10)
    pad = Vector((0, -0.75, 0.66)).normalized()
    def w(p):
        q = _bump(p.copy(), pad, 0.35, 0.0015)
        if q.y < -0.0162:
            q.y = -0.0162 - (-0.0162 - q.y) * 0.2
        if q.y > 0.0165:
            a = math.atan2(q.x, q.z)
            q.y = 0.0165 + 0.0035 * abs(math.sin(3.5 * a) * math.cos(2 * a))
        if math.hypot(q.x, q.z) < 0.004:
            q.y = 0.0105
        return q
    return _plush(s.warp(w), 0.0005, seed=4)


part('TeddyPaws__paw', f_sphere(0.020), teddy_paw, smooth=180, weighted=False)


# ------------------------------------------------------------- tricycle --
def trike_main_bar():
    # painted steel tube with a weld collar at each end
    return lathe([(0, -0.1), (0.0084, -0.1), (0.0094, -0.0988), (0.0094, -0.0935), (0.0087, -0.0915),
                  (0.0087, 0.0915), (0.0094, 0.0935), (0.0094, 0.0988), (0.0084, 0.1), (0, 0.1)], seg=12)


def trike_fork():
    # steerer tube with the head-tube collar near the top, crown, and the
    # blade flattened into a dropout at the bottom (flat faces toward the
    # wheel axle, z)
    s = lathe([(0, -0.07), (0.0050, -0.07), (0.0066, -0.0665), (0.0066, 0.030), (0.0080, 0.0335),
               (0.0080, 0.0465), (0.0070, 0.050), (0.0060, 0.052), (0.0060, 0.0665), (0.0048, 0.070), (0, 0.070)],
              seg=12)
    def flat(p):
        if p.y < -0.045:
            k = 1 - 0.45 * min(1.0, (-0.045 - p.y) / 0.02)
            p.z *= k
            p.x *= 1 + 0.12 * min(1.0, (-0.045 - p.y) / 0.02)
        return p
    return s.warp(flat).fit((-0.008, -0.07, -0.008), (0.008, 0.07, 0.008))


def trike_handlebar():
    # bar with the stem clamp in the middle, rolled ends under the grips
    return lathe([(0, -0.06), (0.0058, -0.06), (0.0066, -0.0575), (0.0066, -0.013), (0.0080, -0.0105),
                  (0.0080, 0.0105), (0.0066, 0.013), (0.0066, 0.0575), (0.0058, 0.06), (0, 0.06)], seg=12)


def trike_grip():
    # soft grip: finger ribs, flared ends
    return lathe([(0, -0.012), (0.0090, -0.012), (0.0100, -0.0105), (0.0100, -0.0082), (0.0086, -0.0070),
                  (0.0092, -0.0045), (0.0086, -0.0018), (0.0092, 0.0010), (0.0086, 0.0038), (0.0092, 0.0062),
                  (0.0086, 0.0074), (0.0100, 0.0086), (0.0100, 0.0105), (0.0090, 0.012), (0, 0.012)], seg=9)


def trike_seat():
    # moulded saddle: broad at the back (-x), narrow nose, dished top
    st = [(-0.035, 0.018), (-0.0335, 0.0228), (-0.030, 0.0248), (-0.020, 0.025), (-0.005, 0.0235),
          (0.010, 0.0200), (0.022, 0.0160), (0.031, 0.0118), (0.0345, 0.0075), (0.035, 0.004)]
    rings = []
    for x, hw in st:
        dish = 0.0026 * max(0.0, 1 - ((x + 0.008) / 0.03) ** 2)
        sec = [(-hw, -0.0065), (-hw * 0.5, -0.007), (0, -0.007), (hw * 0.5, -0.007), (hw, -0.0065),
               (hw * 1.0, 0.003), (hw * 0.92, 0.0064), (hw * 0.6, 0.007 - dish * 0.7), (0, 0.007 - dish),
               (-hw * 0.6, 0.007 - dish * 0.7), (-hw * 0.92, 0.0064), (-hw * 1.0, 0.003)]
        rings.append([(x, y, z) for z, y in sec])
    return loft(rings)


def _trike_wheel(R, seg, groove=True):
    """Solid plastic trike wheel: hub boss, recessed web, rim, tyre with
    chunky tread lugs (axis +y)."""
    prof = [(0, 0.010), (0.012, 0.010), (0.016, 0.0055), (R - 0.015, 0.0055), (R - 0.009, 0.010),
            (R - 0.004, 0.0095), (R - 0.001, 0.0065), (R, 0.0035)]
    if groove:
        prof.append((R - 0.0007, 0.0012))
    mid = [(r, -y) for r, y in reversed(prof[1:])]
    s = lathe(prof + mid + [(0, -0.010)], seg=seg)
    def lug(p):
        r = math.hypot(p.x, p.z)
        if r > R - 0.0022:
            a = math.atan2(p.x, p.z)
            i = round(a / (TAU / seg))
            if i % 2:
                k = (r - 0.0012) / r
                p.x *= k
                p.z *= k
        return p
    return s.warp(lug)


part('OverturnedTricycles__mainbar', f_cyl(0.010, 0.010, 0.20), trike_main_bar)
part('OverturnedTricycles__fork', f_cyl(0.008, 0.008, 0.14), trike_fork)
part('OverturnedTricycles__handlebar', f_cyl(0.008, 0.008, 0.12), trike_handlebar)
part('OverturnedTricycles__grip', f_cyl(0.010, 0.010, 0.024), trike_grip, smooth=50)
part('OverturnedTricycles__seat', f_box(0.070, 0.014, 0.050), trike_seat, smooth=50)
part('OverturnedTricycles__frontwheel', f_cyl(0.055, 0.055, 0.020), lambda: _trike_wheel(0.055, 16), smooth=45)
part('OverturnedTricycles__rearwheel', f_cyl(0.035, 0.035, 0.020), lambda: _trike_wheel(0.035, 10, False),
     smooth=45)


# ------------------------------------------------------- colouring book --
def book_cover():
    # soft card cover: rounded corners, a spine crease across the middle,
    # the outer edges curling up a little
    s = slab(0.22, 0.14, 0.003, 12, 8).rot(x=-HPI)
    def w(p):
        ax, az = abs(p.x), abs(p.z)
        if ax > 0.105 and az > 0.065:
            p.x -= math.copysign(0.0025, p.x)
            p.z -= math.copysign(0.0025, p.z)
        p.y += 0.0016 * max(0.0, (ax - 0.09) / 0.02) ** 2 + 0.0008 * max(0.0, (az - 0.055) / 0.015) ** 2
        if ax < 0.004:
            p.y -= 0.0005
        return p
    return s.warp(w)


def book_crayon():
    # crayon: flat butt, paper wrapper with rolled edges, bare wax, worn
    # conical tip at +y
    return lathe([(0, -0.025), (0.0035, -0.025), (0.0040, -0.0243), (0.0040, -0.0190), (0.0037, -0.0185),
                  (0.0040, -0.0180), (0.0040, 0.0120), (0.0036, 0.0126), (0.0037, 0.0160), (0.0017, 0.0234),
                  (0.0011, 0.025), (0, 0.025)], seg=10)


part('ColoringBooks__cover', f_box(0.22, 0.003, 0.14), book_cover)
part('ColoringBooks__crayon', f_cyl(0.004, 0.004, 0.05), book_crayon, smooth=50)
