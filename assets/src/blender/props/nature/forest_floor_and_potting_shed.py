"""b24: the "nature" theme. Forest-floor litter (dandelions, moss, ferns,
snail shells, pine needles, bird nests, chestnuts), potting-shed plants and
tools (hanging planters, seedlings, trowels, watering cans, gloves, shears,
hoes, rakes, hoses, shovels, soil bags, kneeling pads, twine, bird feeders,
gnomes, fertilizer spikes, herb bundles, a dead potted plant) and framed
catalogue pages. Organic, irregular forms, soft shading.

Leaves, grass blades, butterfly wings, webs, seed packets, the apron fabric
and the herb stems are DoubleSide/transparent/custom geometry, and papers,
glass and dots are decals: they stay primitives (see docs/research/props-remodelling/forest_floor_and_potting_shed.md).
"""
import propkit
from propkit import *
from mathutils import Vector, noise

HPI = math.pi / 2
_fw = propkit._fix_winding


# ================================================================ helpers ==
def skin(rings, cap0=True, cap1=True):
    """Skin 3D rings (equal counts; a 1-point ring is a pole) into a solid."""
    v, f, ids = [], [], []
    for ring in rings:
        ids.append(list(range(len(v), len(v) + len(ring))))
        v += [Vector(p) for p in ring]
    for a, b in zip(ids, ids[1:]):
        if len(a) == 1 and len(b) == 1:
            continue
        if len(a) == 1:
            f += [(a[0], b[i], b[(i + 1) % len(b)]) for i in range(len(b))]
        elif len(b) == 1:
            f += [(a[i], b[0], a[(i + 1) % len(a)]) for i in range(len(a))]
        else:
            n = len(a)
            f += [(a[i], a[(i + 1) % n], b[(i + 1) % n], b[i]) for i in range(n)]
    if cap0 and len(ids[0]) > 2:
        f.append(tuple(reversed(ids[0])))
    if cap1 and len(ids[-1]) > 2:
        f.append(tuple(ids[-1]))
    return _fw(Shape(v, f))


def chaikin(pts, k):
    pts = [Vector(p) for p in pts]
    for _ in range(k):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    return pts


def stick(points, rfn, seg=6, smooth=0, cap0='flat', cap1='flat'):
    """Tube along a polyline; rfn(t, a) -> radius (t 0..1 along, a angle).
    cap 'point' closes to a pole a little beyond the end, 'round' domes it."""
    pts = chaikin(points, smooth)
    n = len(pts)
    tan = [(pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized() for i in range(n)]
    up = Vector((0, 0, 1)) if abs(tan[0].z) < 0.9 else Vector((1, 0, 0))
    nrm = tan[0].cross(up).normalized()
    L = [0.0]
    for a, b in zip(pts, pts[1:]):
        L.append(L[-1] + (b - a).length)
    rings = []
    for i in range(n):
        if i:
            nrm = (tan[i - 1].rotation_difference(tan[i]) @ nrm).normalized()
        bn = tan[i].cross(nrm).normalized()
        t = L[i] / L[-1]
        rings.append([pts[i] + (nrm * math.cos(a) + bn * math.sin(a)) * rfn(t, a)
                      for a in (TAU * j / seg for j in range(seg))])
    for end, cap in ((0, cap0), (1, cap1)):
        if cap in ('point', 'round'):
            i = 0 if end == 0 else n - 1
            d = -tan[0] if end == 0 else tan[-1]
            r = rfn(float(end), 0.0)
            tip = [pts[i] + d * r * (1.0 if cap == 'point' else 0.9)]
            if cap == 'round':
                mid = [pts[i] + (p - pts[i]) * 0.7 + d * r * 0.6 for p in rings[i]]
                extra = [mid, tip]
            else:
                extra = [tip]
            rings = (list(reversed(extra)) + rings) if end == 0 else (rings + extra)
    return skin(rings, cap0 == 'flat', cap1 == 'flat')


def ring_section(R, sec, seg=32, arc=TAU, a0=0.0, rmod=None):
    """Sweep a closed section [(dr, dz), ...] round the Z axis in the XY plane
    (a torus_geo! of any section). rmod(a) -> (k, dz) scales/offsets it."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        k, dzz = rmod(a) if rmod else (1.0, 0.0)
        for dr, dz in sec:
            v.append(((R + dr * k) * math.cos(a), (R + dr * k) * math.sin(a), dz * k + dzz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


def oval(rx, rz, n=8, phase=0.0):
    return [(rx * math.cos(TAU * i / n + phase), rz * math.sin(TAU * i / n + phase)) for i in range(n)]


def arc_bounds(R, r, arc):
    """Frame of torus_geo!(R, r, .., .., arc) (ideal bounds)."""
    xs, ys = [], []
    for i in range(361):
        u = arc * i / 360
        for j in range(24):
            w = TAU * j / 24
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r))


def nz(p, sc, seed=0.0):
    return noise.noise(Vector((p.x * sc + seed * 7.1, p.y * sc + seed * 3.3, p.z * sc + seed * 5.7)))


def ax_x(s):
    """Built along +Y, laid along +X."""
    return s.rot(z=-HPI)


# ============================================================ dandelions ==
def dandelion_stem():
    # hollow scape, a slight bow, a little swollen under the flower head
    pts = [(0.0012 * math.sin(math.pi * t), -0.03 + 0.06 * t, 0.0004 * math.sin(TAU * t))
           for t in (i / 8 for i in range(9))]
    return stick(pts, lambda t, a: 0.00085 - 0.00017 * t + (0.00028 if t > 0.94 else 0.0), seg=6, smooth=1)


part('Dandelions__stem', f_cyl(0.001, 0.001, 0.06), dandelion_stem, smooth=70)
# one seed of the clock: achene at the bottom, fine beak, pappus tuft on top
part('Dandelions__wisp', f_cyl(0.0004, 0.0004, 0.010), lambda: lathe(
    [(0, -0.005), (0.00022, -0.0047), (0.00034, -0.0036), (0.0003, -0.0026), (0.00012, -0.0019),
     (0.0001, 0.0036), (0.00028, 0.0043), (0.0006, 0.0049), (0.0003, 0.0051), (0, 0.005)], seg=5), smooth=70)


# ========================================================== moss patches ==
def moss_mat():
    # low cushion of moss hugging the floor: lumpy dome, scalloped edge; the
    # plane's face is z = 0 (0.003 m up), the underside reaches the floor
    s = lathe([(0, -0.0028), (0.14, -0.0028), (0.14, -0.0021), (0.132, -0.0006), (0.115, 0.0011),
               (0.085, 0.0023), (0.05, 0.003), (0.02, 0.0033), (0, 0.0034)], seg=28).rot(x=HPI)

    def w(p):
        a = math.atan2(p.y, p.x)
        k = 0.86 + 0.09 * math.sin(3 * a + 0.7) + 0.05 * math.sin(7 * a + 2.0)
        p.x *= k
        p.y *= k
        if p.z > -0.0027:
            p.z += 0.0014 * nz(p, 40.0, 1) + 0.0008 * nz(p, 110.0, 2)
        return p
    s.warp(w)
    lo, hi = s.bounds()
    return s.warp(lambda p: Vector((-0.14 + (p.x - lo.x) / (hi.x - lo.x) * 0.28,
                                    -0.14 + (p.y - lo.y) / (hi.y - lo.y) * 0.28, p.z)))


part('MossPatches__mat', f_bounds((-0.14, -0.14, 0), (0.14, 0.14, 0), 'plane'), moss_mat, smooth=180)


def moss_tuft():
    # a cushion of moss: flat underside, lumpy starry top (the placer squashes it to half height)
    s = sphere(1.0, seg=12, rings=8)

    def w(p):
        p = p * (1.0 + 0.16 * nz(p, 2.6, 3) + 0.06 * nz(p, 7.0, 4))
        if p.y < -0.45:
            p.y = -0.45 + (p.y + 0.45) * 0.35
        return p
    return s.warp(w).fit((-0.013, -0.013, -0.013), (0.013, 0.013, 0.013))


part('MossPatches__tuft', f_sphere(0.013), moss_tuft, smooth=180)


# ================================================================= ferns ==
def fern_stem():
    # rachis: thick at the frond base (+Y, world -X where the big pinnae
    # are), fine at the tip, grooved on its upper face (+X is world up)
    pts = [(0.0, 0.07 - 0.14 * t, 0.0016 * math.sin(math.pi * t) - 0.0006 * t) for t in (i / 7 for i in range(8))]

    def r(t, a):
        base = 0.0024 - 0.0016 * t
        return base * (1.0 - 0.22 * max(0.0, math.cos(a)) ** 4)
    return stick(pts, r, seg=7, smooth=1, cap1='point')


part('Ferns__stem', f_cyl(0.0015, 0.0025, 0.14), fern_stem, smooth=60)


# ========================================================== snail shells ==
# a whorl of the spire: outer face sloping out towards the bottom (+Z is
# world down after rotation.x = PI/2), sutures tucked in top and bottom,
# fine growth ribs across the whorl
WHORL = [(1.0, 0.4), (0.62, 0.97), (-0.35, 0.98), (-1.0, 0.3), (-0.8, -0.72), (0.05, -1.0), (0.66, -0.62)]


def snail_ring():
    rib = lambda a: (1.0 + 0.035 * math.sin(3 * a + 0.5) + 0.02 * math.sin(5 * a), 0.0)
    return ring_section(0.020, [(dr * 0.007, dz * 0.007) for dr, dz in WHORL], seg=20, rmod=rib)


part('SnailShells__ring', f_torus(0.020, 0.007), snail_ring, smooth=70)

# aperture lip: flared, rolled outer edge, tapering into the whorl at both ends
part('SnailShells__lip', arc_bounds(0.018, 0.006, math.pi), lambda: ring_section(
    0.018, [(0.006, 0.0), (0.0042, 0.0045), (-0.0012, 0.006), (-0.0055, 0.0026), (-0.006, -0.0022),
            (-0.002, -0.0058), (0.0035, -0.0045)], seg=12, arc=math.pi,
    rmod=lambda a: (0.45 + 0.55 * math.sin(a) ** 0.5, 0.0)), smooth=70)


# =========================================================== pine needles ==
def pine_needle():
    # half-round needle (flat face down, -X is world down), sharp tip at -Y,
    # a sliver of papery sheath at the base, a faint curve
    rings = []
    for i in range(9):
        t = i / 8
        y = -0.015 + 0.03 * t
        r = 0.0007 * (0.25 + 0.75 * min(1.0, t * 1.6)) * (1.12 if t > 0.9 else 1.0)
        zc = 0.0005 * math.sin(math.pi * t)
        rings.append([(r * math.cos(a) - r * 0.45, y, zc + r * math.sin(a)) for a in
                      (-HPI, -HPI * 0.5, 0.0, HPI * 0.5, HPI)] + [(-r * 0.45, y, zc)])
    return skin([[(-0.00012, -0.0158, 0.0)]] + rings)


part('PineNeedles__needle', f_cyl(0.0007, 0.0005, 0.030), pine_needle, smooth=60)


# ============================================================= bird nests ==
def nest_twig(L, r, seed):
    """A crooked twig: kinks, bark lumps, two nodes, a snapped side stub,
    broken ends."""
    rnd = random.Random(seed)
    pts = [(r * rnd.uniform(-0.45, 0.45), -L / 2 + L * i / 5, r * rnd.uniform(-0.45, 0.45)) for i in range(6)]
    pts[0] = (0.0, -L / 2, 0.0)

    def rf(t, a):
        node = 0.14 * (math.exp(-((t - 0.33) / 0.05) ** 2) + math.exp(-((t - 0.71) / 0.05) ** 2))
        return r * 0.72 * (1.0 + node + 0.09 * math.sin(3 * a + seed + 9 * t))
    s = stick(pts, rf, seg=6, smooth=1)
    p = Vector(chaikin(pts, 1)[4])
    d = Vector((rnd.choice((-1, 1)) * 0.8, 0.6, 0.3)).normalized()
    stub = stick([p, p + d * r * 1.9], lambda t, a: r * (0.42 - 0.12 * t), seg=5)
    return (s + stub).fit((-r, -L / 2, -r), (r, L / 2, r))


part('BirdNests__twig', f_cyl(0.003, 0.003, 0.055), lambda: nest_twig(0.055, 0.003, 3), smooth=60)
part('BirdNests__twigThin', f_cyl(0.002, 0.002, 0.040), lambda: nest_twig(0.040, 0.002, 8), smooth=60)


def nest_egg():
    # blunt end down, pointed end up (the placer stretches it 1.3x in y)
    prof = [(0, -1.0)]
    for i in range(1, 9):
        a = math.pi * i / 9
        y = -math.cos(a)
        prof.append((math.sin(a) * (1.0 - 0.2 * y) * (1.0 - 0.04 * y * y), y))
    prof.append((0, 1.0))
    return lathe(prof, seg=12).fit((-0.01, -0.01, -0.01), (0.01, 0.01, 0.01))


part('BirdNests__egg', f_sphere(0.010), nest_egg, smooth=180)


# ============================================================== chestnuts ==
def chestnut_nut():
    # plump nut, broad pale-scar base, rounded shoulder, drawn up to the tuft;
    # one face flattened where it lay against its neighbours in the husk
    s = lathe([(0, -1.0), (0.5, -0.97), (0.82, -0.86), (0.97, -0.58), (1.0, -0.22), (0.94, 0.18),
               (0.77, 0.52), (0.52, 0.78), (0.26, 0.93), (0, 1.0)], seg=14)

    def w(p):
        if p.z > 0:
            p.z *= 0.78
        p.x += 0.03 * nz(p, 2.0, 5)
        return p
    return s.warp(w).fit((-0.014, -0.014, -0.014), (0.014, 0.014, 0.014))


part('Chestnuts__nut', f_sphere(0.014), chestnut_nut, smooth=180)
# the tuft: a dried stub of the style drawn to a wispy point
part('Chestnuts__tip', f_cone(0.004, 0.005), lambda: lathe(
    [(0, -0.0025), (0.0034, -0.0025), (0.004, -0.0019), (0.0037, -0.0009), (0.0024, -0.0001),
     (0.0012, 0.0008), (0.0005, 0.0019), (0, 0.0025)], seg=7), smooth=70)


def husk_spike():
    # a little burr of three spines from a common base
    s = lathe([(0, -0.004), (0.0012, -0.004), (0.001, -0.0031), (0, -0.0025)], seg=5)
    for k in range(3):
        a = TAU * k / 3 + 0.4
        d = Vector((0.0011 * math.cos(a), 0.0, 0.0011 * math.sin(a)))
        s = s + stick([(0, -0.0032, 0), d * 0.4 + Vector((0, -0.0002, 0)), d + Vector((0, 0.0038, 0))],
                      lambda t, a: 0.00042 * (1 - t) + 0.00005, seg=4, cap1='point')
    return s.fit((-0.0015, -0.004, -0.0015), (0.0015, 0.004, 0.0015))


part('Chestnuts__spike', f_cone(0.0015, 0.008), husk_spike, smooth=60)


# ========================================================= hanging plants ==
def hp_arm():
    # forged bracket: wall plate at -Z, twisted square bar, an eye at the
    # far end the chain hangs from
    plate = rbox(0.010, 0.014, 0.005, r=0.0012, seg=1).move(z=-0.0675)
    rings = []
    for i in range(25):
        z = -0.065 + 0.117 * i / 24
        tw = 0.0 if z < -0.045 or z > 0.035 else (z + 0.045) / 0.08 * TAU * 1.5
        rings.append([(0.0034 * math.cos(tw + a), 0.0034 * math.sin(tw + a), z)
                      for a in (math.pi / 4 + HPI * k for k in range(4))])
    bar = skin(rings)
    eye = ring_section(0.0052, oval(0.0017, 0.0017, 6), seg=16).rot(y=HPI).move(z=0.0598)
    return plate + bar + eye


part('HangingPlants__arm', f_box(0.010, 0.014, 0.14), hp_arm, smooth=40)


def hp_chain():
    s = Shape()
    for i in range(5):
        link = torus(0.00105, 0.00045, seg=8, tseg=3).scale(1.0, 5.1, 1.0).rot(y=HPI * (i % 2))
        s = s + link.move(y=-0.0240 + i * 0.0120)
    return s


part('HangingPlants__chain', f_cyl(0.0015, 0.0015, 0.06), hp_chain, smooth=180)

# hanging planter bowl: foot recess, rounded belly, rolled rim; the soil
# face sits just under the dirt disc
part('HangingPlants__pot', f_cyl(0.055, 0.045, 0.048), lambda: lathe(
    [(0, -0.0225), (0.030, -0.0225), (0.033, -0.024), (0.040, -0.024), (0.0445, -0.0215), (0.0466, -0.017),
     (0.050, -0.002), (0.052, 0.012), (0.0525, 0.0155), (0.0548, 0.0167), (0.055, 0.021), (0.054, 0.0238),
     (0.052, 0.024), (0.0503, 0.0228), (0.0495, 0.0215), (0, 0.0215)], seg=18), smooth=40)


# ============================================================== seedlings ==
part('Seedlings__pot', f_cyl(0.028, 0.024, 0.030), lambda: lathe(
    [(0, -0.015), (0.0215, -0.015), (0.024, -0.0138), (0.0258, 0.0055), (0.0278, 0.0068), (0.028, 0.0088),
     (0.028, 0.0138), (0.0268, 0.015), (0.0244, 0.0146), (0, 0.0146)], seg=14), smooth=40)


def seedling_stem():
    # hypocotyl: a slight crook, swelling where the seed leaves spring
    pts = [(0.0004 * math.sin(math.pi * t * 1.3), -0.012 + 0.024 * t, 0.0002 * t) for t in (i / 4 for i in range(5))]
    return stick(pts, lambda t, a: 0.0011 + 0.0004 * t ** 3, seg=5, smooth=1)


part('Seedlings__stem', f_cyl(0.0015, 0.0015, 0.024), seedling_stem, smooth=70)


# ================================================================ trowels ==
def trowel_blade():
    # frame: x 0..0.02 is the convex back of the scoop (world +z), y the
    # length (+y toward the tip), z the width; a thin dished sheet from a
    # narrow heel at the shank to a full belly, the hollow facing -x
    rings = []
    th = [math.pi * k / 5 for k in range(6)]
    for i in range(11):
        t = i / 10
        y = -0.04 + 0.08 * t
        w = 0.0045 + 0.0153 * math.sin(min(1.0, t / 0.75) * HPI) - 0.0025 * max(0.0, (t - 0.8) / 0.2)
        d = 0.0035 + 0.0163 * math.sin(min(1.0, t / 0.7) * HPI)
        out = [(d * math.sin(a), y, w * math.cos(a)) for a in th]
        inn = [((d - 0.0021) * math.sin(a), y, (w - 0.0014) * math.cos(a)) for a in reversed(th)]
        rings.append(out + inn)
    return skin(rings)


part('Trowels__blade', f_bounds((0.0, -0.04, -0.02), (0.02, 0.04, 0.02)), trowel_blade, smooth=50)
# the point: an ogive drawn out from the blade end (+y) to a point (-y)
part('Trowels__tip', f_cone(0.015, 0.020), lambda: lathe(
    [(0, -0.010), (0.0022, -0.0088), (0.0062, -0.0055), (0.0105, -0.001), (0.0136, 0.0045), (0.015, 0.0085),
     (0.0146, 0.010), (0, 0.010)], seg=10), smooth=50)
# shank: ferrule collar into the handle (-y, world +x), neck up to the blade heel
part('Trowels__shaft', f_cyl(0.005, 0.005, 0.040), lambda: lathe(
    [(0, -0.02), (0.0046, -0.02), (0.005, -0.018), (0.005, -0.0125), (0.0038, -0.0105), (0.0031, -0.008),
     (0.0029, 0.013), (0.0034, 0.0175), (0.0041, 0.02), (0, 0.02)], seg=8), smooth=40)
# grip: ferrule ring at the shank end (+y), waist, swelling palm, flared butt
part('Trowels__handle', f_cyl(0.012, 0.014, 0.060), lambda: lathe(
    [(0, -0.03), (0.0118, -0.03), (0.0138, -0.0275), (0.014, -0.022), (0.0128, -0.012), (0.0118, -0.002),
     (0.0121, 0.012), (0.0109, 0.021), (0.0112, 0.0235), (0.012, 0.025), (0.012, 0.029), (0, 0.03)], seg=12), smooth=50)


def trowel_cap():
    # end knob with a hanging hole; handle side at -x
    s = ax_x(lathe([(0, -0.014), (0.0118, -0.014), (0.0136, -0.0095), (0.014, -0.0025), (0.0128, 0.0055),
                    (0.0096, 0.0108), (0.0052, 0.0134), (0, 0.014)], seg=12))
    return cut(s, cyl(0.0032, 0.0032, 0.04, seg=8, bevel_r=0).rot(x=HPI).move(x=0.0045))


part('Trowels__cap', f_sphere(0.014), trowel_cap, smooth=50)


# ========================================================= watering cans ==
def can_body():
    # galvanised can: rolled foot bead, two swaged ribs, rolled top seam, a
    # low domed top with the fill opening towards the back (+x)
    s = lathe([(0, -0.04), (0.0472, -0.04), (0.0502, -0.0362), (0.0493, -0.033),
               (0.0487, -0.013), (0.0494, -0.0115), (0.0484, -0.010), (0.0469, 0.013), (0.0476, 0.0145),
               (0.0466, 0.016), (0.0455, 0.0335), (0.0459, 0.0368), (0.0448, 0.0392), (0.041, 0.0388),
               (0, 0.040)], seg=18)
    return cut(s, cyl(0.014, 0.014, 0.02, seg=12, bevel_r=0).move(x=0.021, y=0.036))


part('WateringCans__body', f_cyl(0.045, 0.050, 0.080), can_body, smooth=40)
# spout: soldered flange at the wide end (-y), rolled lip at the narrow end
part('WateringCans__spout', f_cyl(0.008, 0.010, 0.095), lambda: lathe(
    [(0, -0.0475), (0.0086, -0.0475), (0.010, -0.0462), (0.010, -0.0412), (0.0091, -0.039), (0.0087, -0.035),
     (0.0072, 0.042), (0.0079, 0.0448), (0.008, 0.0466), (0.0072, 0.0475), (0.0048, 0.0468), (0, 0.0455)], seg=11),
    smooth=40)
# rose: conical back (-y), a slightly domed face inside a beaded rim (+y)
part('WateringCans__rose', f_cyl(0.012, 0.014, 0.006), lambda: lathe(
    [(0, -0.003), (0.0124, -0.003), (0.014, -0.0021), (0.014, -0.0006), (0.0129, 0.0014), (0.0121, 0.003),
     (0.0109, 0.0025), (0.006, 0.0027), (0, 0.0029)], seg=16), smooth=40)
# strap handle: folded sheet strip, flat across its width (z), rolled edges
part('WateringCans__handle', arc_bounds(0.038, 0.005, math.pi), lambda: ring_section(
    0.038, [(0.0016, -0.005), (0.0023, -0.0036), (0.0023, 0.0036), (0.0016, 0.005), (-0.0011, 0.0047),
            (-0.0017, 0.003), (-0.0017, -0.003), (-0.0011, -0.0047)], seg=16, arc=math.pi), smooth=40)


# ======================================================= gardening gloves ==
def glove_palm():
    # stuffed-looking fabric palm: flat underside, puffed back, welted edge
    s = slab(0.045, 0.055, 0.010, 5, 6).rot(x=HPI)

    def w(p):
        p.x *= (1 - 0.16 * min(1.0, abs(p.z) / 0.0275) ** 4) * (1 - 0.07 * max(0.0, -p.z / 0.0275))
        fx = 1 - min(1.0, abs(p.x) / 0.0225) ** 2
        fz = 1 - min(1.0, abs(p.z) / 0.0275) ** 2
        k = 0.3 + 0.7 * (fx * fz) ** 0.5
        if p.y > 0:
            p.y = -0.001 + (p.y + 0.001) * k * 1.2 + 0.0006 * nz(p, 90.0, 6)
        else:
            p.y = -0.001 + (p.y + 0.001) * (0.55 + 0.45 * k)
        return p
    return s.warp(w).fit((-0.0225, -0.005, -0.0275), (0.0225, 0.005, 0.0275))


part('GardeningGloves__palm', f_box(0.045, 0.010, 0.055), glove_palm, smooth=180)


def glove_thumb():
    # stuffed thumb pointing +z (towards the fingers), knuckle crease,
    # tip curling in towards the palm (+x)
    s = sphere(1.0, seg=12, rings=8)

    def w(p):
        k = 1.0 - 0.22 * max(0.0, p.z) - 0.07 * math.exp(-((p.z - 0.15) / 0.12) ** 2)
        p.x *= k
        p.y *= k
        p.x += 0.12 * max(0.0, p.z) ** 2
        return p
    return s.warp(w).fit((-0.01, -0.01, -0.01), (0.01, 0.01, 0.01))


part('GardeningGloves__thumb', f_sphere(0.010), glove_thumb, smooth=180)


def glove_finger():
    # finger of the glove: rounded tip at +z, a knuckle crease
    rings = []
    for z, k in ((-0.010, 0.92), (-0.004, 0.97), (0.0005, 0.9), (0.005, 0.96), (0.0085, 0.8)):
        rings.append([(0.004 * k * c, 0.005 * k * s_, z) for c, s_ in oval(1.0, 1.0, 8)])
    return skin(rings + [[(0.0, 0.0, 0.010)]])


part('GardeningGloves__finger', f_box(0.008, 0.010, 0.020), glove_finger, smooth=180)


def glove_cuff():
    # knit cuff: a soft band with rib ridges running towards the fingers
    rings = []
    for i in range(15):
        x = -0.024 + 0.048 * i / 14
        k = (1.0 if i % 2 else 0.9) * (0.8 if i in (0, 14) else 1.0)
        rings.append([(x, 0.006 * k * s_, 0.006 * k * c) for c, s_ in oval(1.0, 1.0, 8, math.pi / 8)])
    return skin(rings)


part('GardeningGloves__cuff', f_box(0.048, 0.012, 0.012), glove_cuff, smooth=60)


# ======================================================== pruning shears ==
def shear_blade(side, hook):
    """Bypass blade lying flat (thin axis y): curved spine on one side, the
    edge on the other (side +1: edge towards +z). hook=True is the broad,
    blunt anvil hook; otherwise a sharp cutting blade. -x is the tip end."""
    rings = []
    for i in range(12):
        u = i / 11
        x = -0.0225 + 0.045 * u
        zs = -0.005 + (0.0016 if hook else 0.0022) * (1 - u) ** 2
        ze = 0.005 - (0.0026 if hook else 0.0036) * (1 - u) ** 1.3
        heel = 1 - 0.3 * max(0.0, (u - 0.82) / 0.18) ** 2
        zs, ze = zs * heel, ze * heel
        tk = 0.0025 * (0.6 + 0.4 * u)
        te = 0.0011 if hook else 0.00035
        zm = zs + (ze - zs) * 0.45
        sec = [(zs, tk), (zm, tk * 0.85), (ze, te), (ze, -te), (zm, -tk * 0.85), (zs, -tk)]
        rings.append([(x, y, z * side) for z, y in sec])
    return skin(rings)


part('PruningShears__blade', f_box(0.045, 0.005, 0.010), lambda: shear_blade(1, False), smooth=40)
part('PruningShears__hook', f_box(0.045, 0.005, 0.010), lambda: shear_blade(-1, True), smooth=40)


def shear_tip():
    # the blade's point: +y joins the blade, drawn to a hooked point at -y
    rings = []
    for i in range(5):
        t = i / 5
        k = (1 - t) ** 0.75
        rings.append([(0.0055 * k * c, 0.007 - 0.014 * t, 0.0016 * t * t + 0.006 * k * s_)
                      for c, s_ in oval(1.0, 1.0, 6)])
    return skin(rings + [[(0.0, -0.007, 0.0018)]])


part('PruningShears__tip', f_cone(0.006, 0.014), shear_tip, smooth=50)
# pivot bolt: domed head (+y), shank, flat nut (-y)
part('PruningShears__pivot', f_cyl(0.004, 0.004, 0.020), lambda: lathe(
    [(0, -0.01), (0.0036, -0.01), (0.004, -0.0075), (0.0025, -0.007), (0.0025, 0.007), (0.004, 0.0075),
     (0.0036, 0.0093), (0, 0.01)], seg=8), smooth=50)


def shear_handle():
    # moulded grip: slim neck at the pivot end (-x), swelling grip with
    # shallow rings, rounded butt at +x
    rings = []
    for i in range(12):
        u = i / 11
        x = -0.0275 + 0.0535 * u
        g = math.sin(min(1.0, u / 0.35) * HPI)
        dip = 1 - 0.06 * sum(math.exp(-((u - c) / 0.03) ** 2) for c in (0.5, 0.63, 0.76))
        butt = 1 - 0.28 * max(0.0, (u - 0.85) / 0.15) ** 2
        h, w = (0.0045 + 0.0025 * g) * dip * butt, (0.0042 + 0.0018 * g) * dip * butt
        rings.append([(x, h * s_, w * c) for c, s_ in oval(1.0, 1.0, 8)])
    return skin(rings + [[(0.0275, 0.0, 0.0)]])


part('PruningShears__handle', f_box(0.055, 0.014, 0.012), shear_handle, smooth=60)


# ================================================== hoes, rakes, shovels ==
def tool_handle(r, L):
    # ash handle: faint swell and taper, subtly out of round with the grain,
    # rounded butt (-y, the knob end), a shoulder into the ferrule (+y)
    prof = [(0, -L / 2), (r * 0.8, -L / 2)]
    for i in range(14):
        t = i / 13
        prof.append((r * (0.985 + 0.015 * math.sin(math.pi * t * 0.8)) * (1 - 0.06 * t ** 3), -L / 2 + 0.008 + (L - 0.016) * t))
    prof += [(r * 0.9, L / 2), (0, L / 2)]
    s = lathe(prof, seg=12)
    return s.warp(lambda p: Vector((p.x * (1 + 0.025 * math.sin(p.y * 40)), p.y, p.z * (1 - 0.02 * math.sin(p.y * 40))
                                    + 0.00025 * nz(p, 30.0, 11))))


part('GardenHoes__handle', f_cyl(0.012, 0.012, 0.55), lambda: tool_handle(0.012, 0.55), smooth=50)


def tool_ferrule():
    # crimped steel socket ferrule with a rivet
    s = lathe([(0, -0.01), (0.0128, -0.01), (0.014, -0.0088), (0.014, -0.0065), (0.0131, -0.0055),
               (0.0131, 0.0055), (0.014, 0.0065), (0.014, 0.0088), (0.0128, 0.01), (0, 0.01)], seg=14)
    riv = lathe([(0, 0), (0.0021, 0), (0.0019, 0.0006), (0.0011, 0.0011), (0, 0.0012)], seg=8)
    return s + riv.rot(z=-HPI).move(x=0.0128)


part('GardenHoes__collar', f_cyl(0.014, 0.014, 0.020), tool_ferrule, smooth=40)


def hoe_head():
    # draw-hoe blade lying flat: rounded back corners, slightly convex
    # front edge ground to a bevel (-x), socket boss at the back centre (+x)
    out = [(-0.008 + 0.0009 * (z / 0.035) ** 2, z) for z in (0.035 - 0.07 * i / 8 for i in range(9))]
    pts = out + [p for p in round_outline([(-0.0071, -0.035), (0.008, -0.035), (0.008, 0.035), (-0.0071, 0.035)], 0.004)
                 if p[0] > -0.005]
    s = prism(_ccw2(pts), 0.0036).rot(x=HPI)
    s.warp(lambda p: Vector((p.x, p.y * (0.4 + 0.6 * min(1.0, (p.x + 0.008) / 0.009)), p.z)))
    boss = rbox(0.0065, 0.006, 0.013, r=0.0012, seg=1).move(x=0.0048)
    return s + boss


def _ccw2(pts):
    # order an outline around its centroid (convex outlines only)
    cx = sum(p[0] for p in pts) / len(pts)
    cy = sum(p[1] for p in pts) / len(pts)
    return sorted(pts, key=lambda p: math.atan2(p[1] - cy, p[0] - cx))


part('GardenHoes__head', f_box(0.016, 0.006, 0.070), hoe_head, smooth=35)
# turned end knob: neck into the handle at -x, domed end at +x
part('GardenHoes__cap', f_sphere(0.014), lambda: ax_x(lathe(
    [(0, -0.014), (0.0112, -0.014), (0.0118, -0.0112), (0.0112, -0.008), (0.0132, -0.0055), (0.014, -0.001),
     (0.0133, 0.0055), (0.0105, 0.0105), (0.0058, 0.0134), (0, 0.014)], seg=12)), smooth=50)


def rake_bar():
    # rake head bar: flat steel bar on edge, eased edges, rounded ends
    rings = []
    for i in range(9):
        t = i / 8
        z = -0.05 + 0.1 * t
        e = 1 - 0.35 * max(0.0, abs(t - 0.5) * 2 - 0.9) / 0.1
        rings.append([(0.0025 * e * c, 0.004 * e * s_, z) for c, s_ in
                      ((1, 0.8), (0.8, 1), (-0.8, 1), (-1, 0.8), (-1, -0.8), (-0.8, -1), (0.8, -1), (1, -0.8))])
    return skin(rings)


part('GardenRakes__bar', f_box(0.005, 0.008, 0.100), rake_bar, smooth=40)


def rake_tine():
    # tine: thick root at the top (+y), curving slightly, drawn to a point
    pts = [(0.0007 * (1 - t) ** 2 - 0.0003, 0.02 - 0.04 * t, 0.0) for t in (i / 4 for i in range(5))]
    return stick(pts, lambda t, a: 0.0017 - 0.0011 * t, seg=6, smooth=1, cap1='point').fit(
        (-0.0018, -0.02, -0.0018), (0.0018, 0.02, 0.0018))


part('GardenRakes__tine', f_cyl(0.001, 0.0018, 0.040), rake_tine, smooth=50)
part('GardenRakes__cap', f_sphere(0.013), lambda: ax_x(lathe(
    [(0, -0.013), (0.0098, -0.013), (0.0106, -0.0104), (0.0126, -0.0062), (0.013, -0.0012), (0.0122, 0.0048),
     (0.0098, 0.0094), (0.0055, 0.0122), (0, 0.013)], seg=12)), smooth=50)


# ============================================================ garden hoses ==
def loop_tube(center, r, seg, m, rfn=None, twist=0.0):
    """Closed tube round a loop: center(a) -> (point, radial unit); the
    section is swept in the (radial, z) plane; rfn(a, phi) scales it."""
    v, f = [], []
    for i in range(seg):
        a = TAU * i / seg
        c, rad = center(a)
        for j in range(m):
            ph = TAU * j / m + twist * a
            k = rfn(a, ph) if rfn else 1.0
            v.append(c + (rad * math.cos(ph) + Vector((0, 0, 1)) * math.sin(ph)) * r * k)
    for i in range(seg):
        i2 = (i + 1) % seg
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    return _fw(Shape(v, f))


def fit_xy(s, rx, ry):
    lo, hi = s.bounds()
    return s.warp(lambda p: Vector((-rx + (p.x - lo.x) / (hi.x - lo.x) * 2 * rx,
                                    -ry + (p.y - lo.y) / (hi.y - lo.y) * 2 * ry, p.z)))


def hose_loop():
    # one turn of the coil: not quite round, rising and falling a little off
    # the turn below (frame -z is world up), a moulded stripe along the top
    def center(a):
        R = 0.090 * (1 + 0.018 * math.sin(2 * a + 1.0) + 0.01 * math.sin(3 * a + 0.3))
        rad = Vector((math.cos(a), math.sin(a), 0.0))
        return rad * R + Vector((0, 0, -0.0028 * (0.5 + 0.5 * math.sin(a + 0.4)))), rad
    s = loop_tube(center, 0.008, 34, 7, rfn=lambda a, ph: 1.07 if abs(ph - 5 * TAU / 7) < 0.1 else 1.0)
    return fit_xy(s, 0.098, 0.098)


part('GardenHoses__loop', f_torus(0.090, 0.008), hose_loop, smooth=60)


def flutes(s, n, depth, y0, y1, rmin=0.0):
    def w(p):
        r = math.hypot(p.x, p.z)
        if r < max(rmin, 1e-7) or not (y0 - 1e-7 <= p.y <= y1 + 1e-7):
            return p
        k = max(0.0, r - depth * (0.5 + 0.5 * math.cos(n * math.atan2(p.x, p.z)))) / r
        return Vector((p.x * k, p.y, p.z * k))
    return s.warp(w)


# brass nozzle: knurled hose coupling (+y, towards the coil), barrel with a
# grip ring, a flared front (-y) that takes the spray tip
part('GardenHoses__nozzle', f_cyl(0.010, 0.012, 0.060), lambda: flutes(lathe(
    [(0, -0.03), (0.0108, -0.03), (0.012, -0.0275), (0.012, -0.019), (0.0108, -0.016), (0.0106, -0.003),
     (0.0112, -0.001), (0.0101, 0.004), (0.0101, 0.024), (0.0082, 0.027), (0.0078, 0.03), (0, 0.03)], seg=12),
    6, 0.0009, 0.0039, 0.0241, rmin=0.0095), smooth=45)
# adjustable spray tip: ribbed collar, cone, orifice
part('GardenHoses__tip', f_cone(0.008, 0.014), lambda: lathe(
    [(0, -0.007), (0.0078, -0.007), (0.008, -0.005), (0.0052, 0.002), (0.0035, 0.0062), (0.0022, 0.007),
     (0.0012, 0.0062), (0, 0.006)], seg=8), smooth=45)


# ========================================================== garden shovels ==
part('GardenShovels__handle', f_cyl(0.014, 0.014, 0.55), lambda: tool_handle(0.014, 0.55), smooth=50)


def shovel_blade():
    # spade blade lying flat: dished across its width, thinning to the
    # front edge (-x) with rounded front corners, the socket strap running
    # down the middle from the back (+x)
    s = slab(0.070, 0.090, 0.003, 6, 6).rot(x=HPI)

    def w(p):
        u = p.x / 0.035
        p.z *= 1 - 0.13 * max(0.0, -u - 0.55) ** 2 / 0.2
        p.y *= 0.55 + 0.45 * min(1.0, (u + 1) / 0.8)
        p.y += 0.0025 * (p.z / 0.045) ** 2 - 0.0012
        return p
    s.warp(w)
    strap = rbox(0.034, 0.003, 0.013, r=0.001, seg=1).move(x=0.018, y=-0.0006)
    return (s + strap).fit((-0.035, -0.004, -0.045), (0.035, 0.004, 0.045))


part('GardenShovels__blade', f_box(0.070, 0.008, 0.090), shovel_blade, smooth=40)
# D-grip yoke: round rod thickening into the hand grip at the crown
part('GardenShovels__grip', arc_bounds(0.024, 0.006, math.pi), lambda: ring_section(
    0.024, [(0.006 * c, 0.006 * s_) for c, s_ in oval(1.0, 1.0, 8)], seg=16, arc=math.pi,
    rmod=lambda a: (0.72 + 0.28 * math.sin(a) ** 3, 0.0)), smooth=60)


# ============================================================== soil bags ==
def superellipse(w, h, n, p=3.0):
    out = []
    for i in range(n):
        a = TAU * i / n
        c, s_ = math.cos(a), math.sin(a)
        out.append((w * math.copysign(abs(c) ** (2 / p), c), h * math.copysign(abs(s_) ** (2 / p), s_)))
    return out


def soil_body():
    # a filled sack: sides and top bulge, slumped a little, creased plastic
    rings = []
    for i in range(9):
        t = i / 8
        z = -0.1 + 0.2 * t
        b = math.sin(math.pi * (0.1 + 0.8 * t))
        rings.append([(x, y - 0.004 * (1 - b), z) for x, y in superellipse(0.054 + 0.006 * b, 0.035 + 0.005 * b, 16, 2.8)])
    s = skin(rings).warp(lambda p: p + Vector((0.0012 * nz(p, 25.0, 12), 0.0015 * nz(p, 25.0, 13), 0.0)))
    return s.fit((-0.06, -0.04, -0.1), (0.06, 0.04, 0.1))


part('SoilBags__body', f_box(0.12, 0.08, 0.20), soil_body, smooth=70)


def soil_cap():
    # bag end: the sack draws in to a flat heat-sealed seam across its width
    # at both ends of the frame (the inner one hides inside the body)
    rings = []
    for i in range(11):
        t = i / 10
        z = -0.055 + 0.11 * t
        e = abs(2 * t - 1)
        h = 0.055 * max(0.05, math.sqrt(max(0.0, 1 - e ** 2.2)))
        w = 0.055 * (0.93 + 0.07 * (1 - e ** 2))
        rings.append([(x, y, z) for x, y in superellipse(w, h, 12, 2.4)])
    s = skin(rings).warp(lambda p: p + Vector((0.0, 0.0016 * nz(p, 30.0, 14), 0.0)))
    return s.fit((-0.055, -0.055, -0.055), (0.055, 0.055, 0.055))


part('SoilBags__cap', f_sphere(0.055), soil_cap, smooth=70)


# ========================================================== kneeling pads ==
def stadium(L, W, n=5):
    r = W / 2
    pts = [(L / 2 - r + r * math.cos(-HPI + math.pi * i / n), r * math.sin(-HPI + math.pi * i / n)) for i in range(n + 1)]
    return pts + [(-x, -y) for x, y in pts]


def pad_body():
    # closed-cell foam: generously rounded edges, a shallow recessed
    # kneeling panel, the carry slot cut through at +x
    s = rbox(0.35, 0.030, 0.20, r=0.009, seg=3)
    slot = prism(stadium(0.086, 0.025, 5), 0.06).rot(x=HPI).move(x=0.115)
    panel = prism(rounded_rect(0.20, 0.15, 0.025, 3), 0.004).rot(x=HPI).move(x=-0.045, y=0.0158)
    return cut(s, slot, panel)


part('KneelingPads__body', f_box(0.35, 0.030, 0.20), pad_body, smooth=40)


def pad_cutout():
    # the shadow in the carry slot: a stadium plug, its top dished down
    st = stadium(0.080, 0.020, 5)
    inset = stadium(0.072, 0.013, 5)
    return skin([[(x, -0.016, z) for x, z in st], [(x, 0.016, z) for x, z in st],
                 [(x, 0.0125, z) for x, z in inset]])


part('KneelingPads__cutout', f_box(0.080, 0.032, 0.020), pad_cutout, smooth=40)


# ============================================================ twine balls ==
def twine_ball():
    # wound ball: crossing bands of wraps stand proud, a little lumpy
    s = sphere(1.0, seg=18, rings=12)
    axes = [Vector(a).normalized() for a in ((0.2, 1, 0.3), (1, 0.1, -0.4), (-0.3, 0.5, 1), (0.8, -0.6, 0.5))]

    def w(p):
        q = p.normalized()
        b = sum(math.exp(-(q.dot(n) / 0.22) ** 2) for n in axes)
        return q * (1.0 + 0.034 * min(1.3, b) + 0.016 * nz(q, 3.0, 15))
    return s.warp(w).fit((-0.038, -0.038, -0.038), (0.038, 0.038, 0.038))


part('TwineBalls__ball', f_sphere(0.038), twine_ball, smooth=180)


def twine_wrap():
    # a turn of 2-ply twine lying on the ball
    return loop_tube(lambda a: (Vector((0.038 * math.cos(a), 0.038 * math.sin(a), 0.0)),
                                Vector((math.cos(a), math.sin(a), 0.0))), 0.001, 24, 4,
                     rfn=lambda a, ph: 1.0 + 0.22 * math.cos(2 * ph), twist=6.0)


part('TwineBalls__wrap', f_torus(0.038, 0.001), twine_wrap, smooth=70)


def twine_end():
    # the loose end: twisted, wandering, drooping a little (-x is world
    # down) and frayed at the free end (-y)
    pts = [(-0.0013 * (1 - t) ** 2 + 0.0003 * math.sin(9 * t), 0.035 - 0.07 * t, 0.0012 * math.sin(6 * t + 0.5))
           for t in (i / 9 for i in range(10))]
    return stick(pts, lambda t, a: 0.001 * (1 + 0.25 * math.cos(2 * a + 40 * t)) * (1.5 if t > 0.97 else 1.0),
                 seg=4, smooth=1)


part('TwineBalls__end', f_cyl(0.001, 0.001, 0.070), twine_end, smooth=70)


# ======================================================= framed pictures ==
def rrect_pts(w, h, r, cseg=3):
    r = max(1e-5, min(r, w / 2 - 1e-5, h / 2 - 1e-5))
    out = []
    for cx, cy, a0 in ((w / 2 - r, h / 2 - r, 0), (-w / 2 + r, h / 2 - r, 90),
                       (-w / 2 + r, -h / 2 + r, 180), (w / 2 - r, -h / 2 + r, 270)):
        for i in range(cseg + 1):
            a = math.radians(a0 + 90 * i / cseg)
            out.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return out


def frame_loop(w, h, sec, rc, ky=1.0, cseg=2):
    """A picture-frame moulding: sec is a closed loop of (inset, z) points
    going round the moulding's section; insets on y are scaled by ky."""
    rings = [[(x, y, z) for x, y in rrect_pts(w - 2 * ins, h - 2 * ins * ky, max(0.0008, rc - ins), cseg)]
             for ins, z in sec]
    n, R = len(rings[0]), len(rings)
    v, f = [], []
    for ring in rings:
        v += [Vector(p) for p in ring]
    for k in range(R):
        a, b = k * n, ((k + 1) % R) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    return _fw(Shape(v, f))


def catalog_frame():
    # dark oak ogee moulding round a recessed backing board
    sec = [(0.0, -0.006), (0.0, 0.0015), (0.0012, 0.0046), (0.0035, 0.006), (0.0065, 0.0055), (0.0095, 0.0033),
           (0.0135, 0.0028), (0.0175, 0.0042), (0.0205, 0.0036), (0.0205, -0.006)]
    board = rbox(0.182, 0.252, 0.006, r=0.0008, seg=1).move(z=-0.001)
    return frame_loop(0.22, 0.30, sec, 0.004, ky=1.2) + board


part('SeedCatalogs__frame', f_box(0.22, 0.30, 0.012), catalog_frame, smooth=40)


def botanical_frame():
    # flat reeded frame: raised outer bead, two reeds, a step down to the
    # sight edge, backing board behind
    sec = [(0.0, -0.007), (0.0, 0.004), (0.0015, 0.007), (0.004, 0.0065), (0.0055, 0.0048), (0.008, 0.0058),
           (0.0105, 0.0048), (0.013, 0.0058), (0.0155, 0.0046), (0.0172, 0.0034), (0.0205, 0.0034),
           (0.0205, -0.007)]
    board = rbox(0.162, 0.262, 0.006, r=0.0008, seg=1).move(z=-0.002)
    return frame_loop(0.20, 0.30, sec, 0.003) + board


part('Botanicals__frame', f_box(0.20, 0.30, 0.014), botanical_frame, smooth=40)


def pressed_frame():
    # painted shadow-box: rounded outer face, deep inner well to the mount
    sec = [(0.0, -0.009), (0.0, 0.006), (0.0012, 0.0085), (0.0035, 0.009), (0.0105, 0.009), (0.0132, 0.0078),
           (0.0152, 0.0062), (0.0152, -0.009)]
    board = rbox(0.152, 0.212, 0.006, r=0.0008, seg=1).move(z=-0.0015)
    return frame_loop(0.18, 0.24, sec, 0.006) + board


part('PressedFlowers__frame', f_box(0.18, 0.24, 0.018), pressed_frame, smooth=40)


# ========================================================= garden aprons ==
def apron_pocket():
    # canvas patch pocket on the folded apron: hemmed opening towards the
    # waist (-z), a stitched divider down the middle, a little puckered
    s = slab(0.12, 0.05, 0.0016, 12, 5).rot(x=HPI)

    def w(p):
        hem = math.exp(-((p.z + 0.023) / 0.004) ** 2)
        div = math.exp(-(p.x / 0.0025) ** 2) * (1 - hem)
        if p.y > 0:
            p.y += 0.0012 * hem - 0.0006 * div + 0.0004 * nz(p, 60.0, 16)
        p.y += 0.0004 * nz(p, 25.0, 17)
        return p
    return s.warp(w).fit((-0.06, -0.001, -0.025), (0.06, 0.0015, 0.025))


part('GardenAprons__pocket', f_box(0.12, 0.002, 0.05), apron_pocket, smooth=60)


# ========================================================== bird feeders ==
def feeder_seeds():
    # seed column seen through the tube: seeds packed against the wall,
    # a heaped, uneven top
    s = lathe([(0, -0.05), (0.0235, -0.05), (0.025, -0.047), (0.025, -0.03), (0.025, -0.012), (0.025, 0.006),
               (0.025, 0.024), (0.0248, 0.038), (0.021, 0.043), (0.014, 0.0475), (0.006, 0.0495), (0, 0.05)], seg=14)

    def w(p):
        r = math.hypot(p.x, p.z)
        if r > 1e-6:
            k = 1 + 0.035 * nz(p, 380.0, 18)
            p.x *= k
            p.z *= k
        if p.y > 0.035:
            p.y += 0.0018 * nz(p, 150.0, 19)
        return p
    return s.warp(w).fit((-0.025, -0.05, -0.025), (0.025, 0.05, 0.025))


part('BirdFeeders__seeds', f_cyl(0.025, 0.025, 0.10), feeder_seeds, smooth=180)
# roof: overhanging skirt, low dome, a boss for the hanger
part('BirdFeeders__cap', f_cyl(0.036, 0.036, 0.012), lambda: lathe(
    [(0, -0.0045), (0.031, -0.0045), (0.0318, -0.006), (0.0356, -0.006), (0.036, -0.0048), (0.0352, -0.0012),
     (0.028, 0.0028), (0.016, 0.0049), (0.0058, 0.0056), (0.0046, 0.006), (0, 0.006)], seg=14), smooth=45)
# seed tray: rolled rim round a shallow dish
part('BirdFeeders__base', f_cyl(0.045, 0.045, 0.008), lambda: lathe(
    [(0, -0.004), (0.042, -0.004), (0.045, -0.0028), (0.045, 0.0028), (0.0436, 0.004), (0.0412, 0.0034),
     (0.0405, 0.0004), (0.038, -0.0014), (0, -0.0014)], seg=20), smooth=45)
# perch: steel rod, collar where it enters the tube (+y), ball end (-y)
part('BirdFeeders__perch', f_cyl(0.002, 0.002, 0.040), lambda: lathe(
    [(0, -0.02), (0.0014, -0.0198), (0.002, -0.0188), (0.0017, -0.0171), (0.0012, -0.0163), (0.0012, 0.016),
     (0.002, 0.0168), (0.002, 0.0186), (0.0013, 0.0191), (0.0013, 0.02), (0, 0.02)], seg=6), smooth=50)


def feeder_loop():
    # wire hanger ring, the wire ends twisted together at one side
    def c(a):
        rad = Vector((math.cos(a), math.sin(a), 0.0))
        return rad * 0.010, rad
    return loop_tube(c, 0.002, 18, 6, rfn=lambda a, ph: 1.0 + 0.3 * math.exp(-((a - 4.7) / 0.35) ** 2)
                     * (0.6 + 0.4 * math.cos(2 * ph + 9 * a)))


part('BirdFeeders__loop', f_torus(0.010, 0.002), feeder_loop, smooth=60)


# ========================================================= garden gnomes ==
# the gnome's face is towards -z (its nose sits there)
def gnome_boot():
    # chunky boot, toe cap lower than the shaft, toe towards -z
    s = rbox(0.014, 0.016, 0.020, r=0.003, seg=1)

    def w(p):
        if p.y > -0.004 and p.z < 0.0:
            p.y = -0.004 + (p.y + 0.004) * (1 - 0.45 * min(1.0, -p.z / 0.008))
        return p
    return s.warp(w).fit((-0.007, -0.008, -0.010), (0.007, 0.008, 0.010))


part('GardenGnomes__boot', f_box(0.014, 0.016, 0.020), gnome_boot, smooth=40)


def gnome_body():
    # pot-bellied robe: flared hem, belly, belted waist, round shoulders;
    # sleeves down the sides meeting in front of the belly
    s = lathe([(0, -0.035), (0.0244, -0.0335), (0.0262, -0.015), (0.0272, 0.004),
               (0.0258, 0.009), (0.0262, 0.0135), (0.0245, 0.026), (0.019, 0.033), (0, 0.035)], seg=12)
    arms = Shape()
    for sx in (-1, 1):
        arms = arms + stick([(sx * 0.0135, 0.027, 0.002), (sx * 0.0195, 0.019, -0.001), (sx * 0.0212, 0.008, -0.007),
                             (sx * 0.018, 0.0, -0.016), (sx * 0.010, -0.003, -0.0232)],
                            lambda t, a: 0.0056 - 0.0008 * t, seg=5)
    return (s + arms).fit((-0.030, -0.035, -0.030), (0.030, 0.035, 0.030))


part('GardenGnomes__body', f_cyl(0.030, 0.024, 0.070), gnome_body, smooth=70)


def gnome_head():
    # round head: brow over sunk eyes, apple cheeks, a hollow where the nose
    # ball sits, ears at the sides
    s = sphere(1.0, seg=14, rings=10)
    feats = [((-0.33, 0.2, -0.92), -0.07, 0.28), ((0.33, 0.2, -0.92), -0.07, 0.28),
             ((-0.42, -0.2, -0.88), 0.06, 0.32), ((0.42, -0.2, -0.88), 0.06, 0.32),
             ((0.0, 0.0, -1.0), -0.09, 0.3), ((1.0, 0.02, 0.05), 0.09, 0.3), ((-1.0, 0.02, 0.05), 0.09, 0.3)]

    def w(p):
        q = p.normalized()
        k = 1.0
        for c, amt, wd in feats:
            d = (q - Vector(c).normalized()).length
            k += amt * math.exp(-(d / wd) ** 2)
        return q * k
    return s.warp(w).fit((-0.024, -0.024, -0.024), (0.024, 0.024, 0.024))


part('GardenGnomes__head', f_sphere(0.024), gnome_head, smooth=180)


def gnome_beard():
    # beard (the placer flips it apex-down): wavy locks from a broad root to
    # a curling point
    s = lathe([(0, -0.015), (0.0165, -0.015), (0.018, -0.0115), (0.0165, -0.004), (0.0125, 0.004),
               (0.0072, 0.0105), (0.0026, 0.0142), (0, 0.015)], seg=18)
    s = flutes(s, 9, 0.0022, -0.0125, 0.012, rmin=0.0015)

    def w(p):
        t = (p.y + 0.015) / 0.03
        p.z -= 0.0035 * t ** 2
        return p
    return s.warp(w).fit((-0.018, -0.015, -0.018), (0.018, 0.015, 0.018))


part('GardenGnomes__beard', f_cone(0.018, 0.030), gnome_beard, smooth=70)


def gnome_hat():
    # felt cap: slight flare at the band, concave flanks, the point flopping
    # towards the back (+z), a few soft wrinkles
    s = lathe([(0, -0.035), (0.026, -0.035), (0.0252, -0.03), (0.0222, -0.02), (0.0178, -0.008), (0.0128, 0.004),
               (0.0082, 0.015), (0.0045, 0.024), (0.002, 0.031), (0, 0.035)], seg=14)

    def w(p):
        t = max(0.0, (p.y + 0.01) / 0.045)
        p.z += 0.012 * t * t
        p.y -= 0.004 * t ** 3
        p.x += 0.0006 * nz(p, 90.0, 20)
        return p
    return s.warp(w).fit((-0.026, -0.035, -0.026), (0.026, 0.035, 0.026))


part('GardenGnomes__hat', f_cone(0.026, 0.070), gnome_hat, smooth=70)
# rolled felt band round the hat
part('GardenGnomes__brim', f_torus(0.022, 0.005), lambda: ring_section(
    0.022, [(0.005, 0.0), (0.0035, 0.0042), (-0.001, 0.0048), (-0.0046, 0.0022), (-0.0046, -0.0022),
            (-0.001, -0.0048), (0.0035, -0.0042)], seg=20, rmod=lambda a: (1.0 + 0.04 * math.sin(5 * a), 0.0)),
     smooth=70)


def gnome_nose():
    # bulbous nose: bridge up top, round bulb low and forward (-z)
    s = sphere(1.0, seg=8, rings=6)
    s.warp(lambda p: Vector((p.x * (1 - 0.18 * max(0.0, p.y)), p.y, p.z * (1 - 0.12 * max(0.0, p.y)) - 0.12 * max(0.0, -p.y))))
    return s.fit((-0.005, -0.005, -0.005), (0.005, 0.005, 0.005))


part('GardenGnomes__nose', f_sphere(0.005), gnome_nose, smooth=180)


# ===================================================== fertilizer spikes ==
# compressed-fertilizer stake: fluted shank, a collar under the flat head
# (+y), drawn to a fluted point (the tip part, -y end meets it)
part('FertilizerSpikes__body', f_cyl(0.003, 0.004, 0.055), lambda: flutes(lathe(
    [(0, -0.0275), (0.004, -0.0275), (0.0037, -0.004), (0.0032, 0.0225), (0.0036, 0.0238), (0.0036, 0.0262),
     (0.003, 0.0275), (0, 0.0275)], seg=16), 8, 0.0008, -0.0275, 0.0226, rmin=0.002), smooth=50)
part('FertilizerSpikes__tip', f_cone(0.004, 0.012), lambda: flutes(lathe(
    [(0, -0.006), (0.004, -0.006), (0.0027, -0.0008), (0.0011, 0.0038), (0, 0.006)], seg=16),
    8, 0.0008, -0.0061, 0.004, rmin=0.0006), smooth=50)


# ========================================================== herb bundles ==
def herb_hook():
    # cup hook: rosette at the wall (-y), wire out and turned up (-z is up)
    rose = lathe([(0, -0.005), (0.003, -0.005), (0.0022, -0.0038), (0, -0.0035)], seg=6)
    wire = stick([(0, -0.0038, 0.0), (0, 0.0012, 0.0), (0, 0.0035, -0.0008), (0, 0.0042, -0.0026)],
                 lambda t, a: 0.00095, seg=4, smooth=1, cap1='round')
    return (rose + wire).fit((-0.003, -0.005, -0.003), (0.003, 0.005, 0.003))


part('HerbBundles__hook', f_cyl(0.003, 0.003, 0.010), herb_hook, smooth=50)


def herb_twine():
    # two turns of twine round the stems, the knot at the front
    s = Shape()
    for dz, ph in ((-0.0011, 0.0), (0.0011, 1.3)):
        c = (lambda dz: lambda a: (Vector((0.0102 * math.cos(a), 0.0102 * math.sin(a), dz)),
                                   Vector((math.cos(a), math.sin(a), 0.0))))(dz)
        s = s + loop_tube(c, 0.0012, 14, 4, rfn=lambda a, p, ph=ph: 1.0 + 0.2 * math.cos(2 * p + 5 * a + ph)
                          + 0.45 * math.exp(-((a - 4.71) / 0.3) ** 2))
    return s.fit((-0.012, -0.012, -0.002), (0.012, 0.012, 0.002))


part('HerbBundles__twine', f_torus(0.010, 0.002), herb_twine, smooth=60)


# ===================================================== dead potted plants ==
def dead_pot():
    # old terracotta pot: foot ring, tapered body, thick collar, soil face
    # just under the soil disc; a little weathered and uneven
    s = lathe([(0, -0.07), (0.074, -0.07), (0.0792, -0.0682), (0.0801, -0.0645), (0.0795, -0.061),
               (0.0873, 0.0275), (0.0888, 0.0302), (0.0968, 0.0322), (0.0995, 0.0355), (0.100, 0.0655),
               (0.0978, 0.0696), (0.0935, 0.0697), (0.0902, 0.0682), (0, 0.0675)], seg=20)
    return s.warp(lambda p: p + Vector((0.0005 * nz(p, 40.0, 21), 0.0, 0.0005 * nz(p, 40.0, 22))))


part('DeadPottedPlants__pot', f_cyl(0.10, 0.08, 0.14), dead_pot, smooth=40)


def soil_face(r, n, h, sc, depth, seed, m=1.0):
    # dry potting soil filling a pot: mounded, cloddy, sinking under the rim
    s = lathe([(0, -depth), (r, -depth), (r, -min(0.0016, depth * 0.8)), (r * 0.94, 0.0003 * m), (r * 0.78, 0.0014 * m),
               (r * 0.45, 0.0019 * m), (0, 0.0022 * m)], seg=n).rot(x=HPI)
    return s.warp(lambda p: p + Vector((0.0, 0.0, h * nz(p, sc, seed) if p.z > -0.001 else 0.0)))


part('DeadPottedPlants__soil', f_bounds((-0.09, -0.09, 0), (0.09, 0.09, 0), 'plane'),
     lambda: soil_face(0.09, 24, 0.0016, 110.0, 0.004, 23), smooth=180)
part('Seedlings__dirt', f_bounds((-0.026, -0.026, 0), (0.026, 0.026, 0), 'plane'),
     lambda: soil_face(0.026, 14, 0.0004, 220.0, 0.0012, 24, m=0.35), smooth=180)


def dead_stem():
    # withered stalk: crooked, nodes, two snapped side shoots, the tip
    # flopping over
    pts = [(0.0, -0.11, 0.0), (0.0008, -0.07, 0.0004), (-0.0012, -0.03, -0.0006), (0.0006, 0.01, 0.0006),
           (-0.0004, 0.045, -0.0002), (0.0010, 0.075, 0.0), (0.0034, 0.095, 0.0006), (0.006, 0.104, 0.0008),
           (0.0076, 0.11, 0.0006)]

    def rf(t, a):
        node = 0.18 * sum(math.exp(-((t - c) / 0.025) ** 2) for c in (0.18, 0.42, 0.63))
        return (0.0026 - 0.0015 * t) * (1 + node + 0.08 * math.sin(2 * a + 30 * t))
    s = stick(pts, rf, seg=6, smooth=1, cap1='point')
    for y, sx, sz in ((-0.035, 1, 0.3), (0.018, -1, -0.4)):
        base = Vector((sx * 0.0012, y, 0.0))
        s = s + stick([base, base + Vector((sx * 0.0048, 0.0065, sz * 0.004))], lambda t, a: 0.0011 - 0.0004 * t, seg=5)
    return s


part('DeadPottedPlants__stem', f_cyl(0.004, 0.004, 0.22), dead_stem, smooth=60)
