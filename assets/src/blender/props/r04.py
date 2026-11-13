"""r04: three of the everyday core props remodelled at their real sizes
(docs/design/PROPS.md, P1): the torn note's sheet, the shop mannequin and the
wall clock. Sources and measurements: `r04.notes.md`.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part's frame is
given with it.

The torn note: a US letter sheet of typewriter bond (8.5 x 11 in, 0.1 mm),
folded in thirds for an envelope and once more in half for a pocket, the
lower third torn away, partly along its fold. Frame: x across the page from
its left edge (x = 0) to its right (x = W), z down the page from its head
(z = 0), y up from the floor; it lies with its folds sprung up a little. Its
top right corner is cut off along the line a dog-ear folds on (A each way):
that corner is its own part, so the game can curl it up or fold it down.

noteSheet   the sheet (its top side only: it lies on the floor), the two
            creases sharp, the torn edge lifting a little
noteCorner  the dog-ear: the corner folded over flat onto the sheet, the
            fold along x through the origin, the corner toward -z; it
            springs up a little toward its tip (both sides, 0.2 mm)

The mannequin: a fibreglass male display mannequin with an egg head (1.89 m
standing on its plate; chest 0.99 m, waist 0.79 m, hips 0.92 m), its arms
off at the shoulders, its hands off at the wrists, its torso off at the
waist, its head off at the neck, on a round black steel base plate with a
chrome calf rod. Frame: x across its front (its left hand at +x), y up from
the floor (the plate's underside), z out of its front.

manTorso    the torso from the crotch to the neck (fibreglass)
manHead     the egg head, a jaw, a chin and the hint of a nose
manLegL/R   a leg and its bare sculpted foot
manArmL/R   an arm from the shoulder (its deltoid) to the wrist
manHandL/R  a hand, fingers together, curled a little, palm to the thigh
manSeams    the join lines round the waist, the neck and the wrists
manJoins    the join lines where the arms and the legs go into the torso
            (traced along where they meet)
manChips    the paint knocked off its edges (knees, elbows, nose, toes,
            heels, knuckles, shoulders), the grey under it
manPlate    the base plate, 400 mm, 10 mm, its edge rounded, the boss the
            rod screws into
manRod      the calf rod, 12 mm chrome, up behind the right calf and into
            its socket, and the socket's plate

The wall clock: the 12-inch school and office clock (the Lathem kind of the
1980s: 34 cm across, 5 cm deep, a black steel case, a domed glass, a metal
dial). Frame: x across its face, y up, z out of the wall; its back at
z = 0, its axis through x = y = 0. The game paints the dial on a disc.

clockCase   the case: the back, the drum and the rolled bezel, open to the
            dial seat at z = 0.030
clockGlass  the domed glass, seated in the bezel, 7.5 mm proud
clockHour   the hour hand, clockMinute the minute hand, clockSecond the
            sweep second hand and its counterweight: stamped steel, each
            pointing up (+y) about its pivot at the origin, its back at z = 0
clockHub    the cap nut over the hands
"""
from propkit import *
import propkit
from mathutils.bvhtree import BVHTree


def hrand(i, salt=0):
    """A repeatable number in [0, 1) from two integers (no Blender state)."""
    x = (i * 2654435761 + salt * 40503 + 12345) & 0xffffffff
    x ^= x >> 15
    x = (x * 2246822519) & 0xffffffff
    x ^= x >> 13
    x = (x * 3266489917) & 0xffffffff
    x ^= x >> 16
    return x / 4294967296.0


def nz(x, y=0.0, z=0.0):
    return noise.noise(Vector((x, y, z)))


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


# =================================================================== note ===
W, LETTER = 0.2159, 0.2794          # US letter
L3 = LETTER / 3                     # the first fold of three
L23 = LETTER * 2 / 3                # the second (the tear runs partly along it)
A = 0.045                           # the dog-ear's legs
BASE = 0.0015                       # the sheet's underside above the floor


def note_tear(x):
    """How far down the page the torn edge runs at x: along the second fold
    for the left part, then tearing up and across, ragged."""
    t = x / W
    z = L23 + 0.0012 * nz(x * 60, 3.1)
    if t > 0.40:
        z -= (t - 0.40) / 0.60 * 0.021
    # a tongue left where the tear turned
    z += 0.0055 * math.exp(-((t - 0.47) / 0.05) ** 2)
    z += 0.0026 * nz(x * 160, 7.7) + 0.0011 * nz(x * 520, 1.9)
    return z


def note_height(x, z):
    """The sheet's height: each quarter a flat facet sloping down from the
    two creases (both sprung up, mountains), sagging onto the floor where
    it reaches it; a little cockle; the torn lip lifting."""
    zt = note_tear(x)
    k = 0.0068
    sx = k / (W / 2) * 1.35
    sz = k / L3 * 1.2 if z <= L3 else k / max(zt - L3, 0.03) * 1.05
    v = k - sx * abs(x - W / 2) - sz * abs(z - L3)
    s = 0.0012
    tent = s * math.log1p(math.exp(v / s)) if v / s < 30 else v
    cw = smoothstep(0.03, 0.075, math.hypot(x - W, z))       # the corner lies flat
    cockle = 0.0006 * nz(x * 34, z * 34, 2.2) * cw
    lip = 0.0016 * smoothstep(0.014, 0.0, zt - z)
    return BASE + tent * cw + cockle + lip


def note_sheet():
    xs = [0, 0.024, 0.048, 0.072, 0.09, W / 2, 0.126, 0.15, 0.165, W - A, 0.198, W]
    rows = [0, 0.02, 0.04, 0.065, L3, 0.113, 0.132, 0.15]
    head = 0.065

    def z_at(x, r):
        top = max(0.0, x - (W - A))              # the cut the dog-ear leaves
        if r < head and top > 0:
            return top + r * (head - top) / head
        return r

    v, f = [], []
    grid = []
    for r in rows:
        ids = []
        for x in xs:
            v.append(Vector((x, 0.0, z_at(x, r))))
            ids.append(len(v) - 1)
        grid.append(ids)
    for a, b in zip(grid, grid[1:]):
        for i in range(len(xs) - 1):
            f.append((a[i], b[i], b[i + 1], a[i + 1]))
    # the torn edge, finer than the grid, zipped onto the last row
    n = 44
    tear = []
    for k in range(n + 1):
        x = W * k / n
        v.append(Vector((x, 0.0, note_tear(x))))
        tear.append(len(v) - 1)
    last = grid[-1]
    i = j = 0
    while i < len(last) - 1 or j < len(tear) - 1:
        xi = v[last[i + 1]].x if i < len(last) - 1 else 9
        xj = v[tear[j + 1]].x if j < len(tear) - 1 else 9
        if xi <= xj:
            f.append((last[i], tear[j], last[i + 1]))
            i += 1
        else:
            f.append((last[i], tear[j], tear[j + 1]))
            j += 1
    for p in v:
        p.y = note_height(p.x, p.z)
    s = Shape(v, f)
    # every face up (+y)
    out = []
    for face in s.f:
        a, b, c = (s.v[k] for k in face[:3])
        if (b - a).cross(c - a).y < 0:
            face = tuple(reversed(face))
        out.append(face)
    s.f = out
    return s


def note_corner():
    """The dog-ear folded over: a right triangle, its long side the fold (x,
    through the origin), its tip at z = -D; it springs up toward the tip."""
    D = A / math.sqrt(2)
    T = 0.0002
    rows = 5
    top, bot = [], []
    v = []

    def y_of(d):
        return 0.0042 * (d / D) ** 2 + 0.0002 * nz(d * 90, 0.3)

    for k in range(rows + 1):
        d = D * k / rows
        half = D - d
        pts = [(-half, -d), (half, -d)] if k < rows else [(0.0, -D)]
        rt, rb = [], []
        for (x, z) in pts:
            y = y_of(d)
            v.append(Vector((x, y + T / 2, z)))
            rt.append(len(v) - 1)
            v.append(Vector((x, y - T / 2, z)))
            rb.append(len(v) - 1)
        top.append(rt)
        bot.append(rb)
    f = []
    for k in range(rows):
        a, b = top[k], top[k + 1]
        c, d = bot[k], bot[k + 1]
        if len(b) == 2:
            f.append((a[0], b[0], b[1], a[1]))
            f.append((c[1], d[1], d[0], c[0]))
            f.append((a[0], c[0], d[0], b[0]))      # the left rim
            f.append((b[1], d[1], c[1], a[1]))      # the right rim
        else:
            f.append((a[0], b[0], a[1]))
            f.append((c[1], d[0], c[0]))
            f.append((a[0], c[0], d[0], b[0]))
            f.append((b[0], d[0], c[1], a[1]))
    f.append((top[0][1], bot[0][1], bot[0][0], top[0][0]))   # the fold
    return propkit._fix_winding(Shape(v, f))


# ============================================================== mannequin ===
PLATE_T = 0.010


def ring2(a, bf, bb, n, power=2.2):
    """A section: (x, z) points round it, x to a, z forward to bf and back to bb."""
    out = []
    for i in range(n):
        t = TAU * i / n
        c, s = math.cos(t), math.sin(t)
        x = a * math.copysign(abs(c) ** (2.0 / power), c)
        z = math.copysign(abs(s) ** (2.0 / power), s) * (bf if s > 0 else bb)
        out.append((x, z))
    return out


def loft(rings, bottom=None, top=None):
    """Join rings of points (each the same count) and close the ends on
    the given points."""
    n = len(rings[0])
    v, f, idx = [], [], []
    for r in rings:
        ids = []
        for p in r:
            v.append(Vector(p))
            ids.append(len(v) - 1)
        idx.append(ids)
    for a, b in zip(idx, idx[1:]):
        for i in range(n):
            j = (i + 1) % n
            f.append((a[i], a[j], b[j], b[i]))
    if bottom is not None:
        v.append(Vector(bottom))
        c = len(v) - 1
        for i in range(n):
            f.append((c, idx[0][(i + 1) % n], idx[0][i]))
    if top is not None:
        v.append(Vector(top))
        c = len(v) - 1
        for i in range(n):
            f.append((c, idx[-1][i], idx[-1][(i + 1) % n]))
    return propkit._fix_winding(Shape(v, f))


def column(specs, n, power=2.2, bottom=None, top=None):
    """A loft up y: specs (y, cx, cz, a, bf, bb)."""
    rings = [[(cx + x, y, cz + z) for x, z in ring2(a, bf, bb, n, power)] for (y, cx, cz, a, bf, bb) in specs]
    return loft(rings, bottom, top)


def limb_rings(path, secs, n, power=2.0):
    """Sections along a path (bottom to top), each square to it, their
    frames carried along it (no twist where it bends over a shoulder)."""
    pts = [Vector(p) for p in path]
    tans = [(pts[min(k + 1, len(pts) - 1)] - pts[max(k - 1, 0)]).normalized() for k in range(len(pts))]
    side = Vector((1, 0, 0))
    side = (side - tans[0] * side.dot(tans[0])).normalized()
    rings = []
    for k, p in enumerate(pts):
        if k:
            side = (tans[k - 1].rotation_difference(tans[k]) @ side).normalized()
        front = side.cross(tans[k])
        a, bf, bb = secs[k]
        rings.append([p + side * x + front * z for x, z in ring2(a, bf, bb, n, power)])
    return pts, rings


def limb(path, secs, n, power=2.0):
    pts, rings = limb_rings(path, secs, n, power)
    t0 = (pts[1] - pts[0]).normalized()
    t1 = (pts[-1] - pts[-2]).normalized()
    return loft(rings, pts[0] - t0 * secs[0][0] * 0.45, pts[-1] + t1 * secs[-1][0] * 0.45)


# the torso: (y, cz, a, bf, bb)
TORSO = [
    (0.862, 0.000, 0.100, 0.060, 0.070),
    (0.900, 0.000, 0.140, 0.085, 0.100),
    (0.925, -0.002, 0.158, 0.093, 0.114),
    (0.950, -0.004, 0.172, 0.099, 0.124),
    (0.975, -0.004, 0.173, 0.100, 0.123),
    (1.000, -0.004, 0.168, 0.098, 0.118),
    (1.050, -0.002, 0.157, 0.097, 0.100),
    (1.100, 0.000, 0.146, 0.098, 0.096),
    (1.150, 0.002, 0.145, 0.102, 0.094),
    (1.210, 0.004, 0.150, 0.108, 0.095),
    (1.270, 0.004, 0.161, 0.118, 0.100),
    (1.330, 0.004, 0.176, 0.130, 0.110),
    (1.380, 0.004, 0.184, 0.127, 0.113),
    (1.420, 0.000, 0.185, 0.111, 0.108),
    (1.460, -0.006, 0.191, 0.097, 0.100),
    (1.490, -0.010, 0.194, 0.082, 0.088),
    (1.515, -0.010, 0.170, 0.066, 0.076),
    (1.535, -0.008, 0.118, 0.058, 0.066),
    (1.552, -0.004, 0.068, 0.055, 0.060),
    (1.600, 0.000, 0.054, 0.052, 0.054),
    (1.650, 0.006, 0.050, 0.048, 0.050),
    (1.685, 0.008, 0.040, 0.040, 0.040),
]
TORSO_N = 24


def torso_power(y):
    return 2.4 if y > 1.44 else 2.25


def man_torso():
    rings = []
    for (y, cz, a, bf, bb) in TORSO:
        rings.append([(x, y, cz + z) for x, z in ring2(a, bf, bb, TORSO_N, torso_power(y))])
    s = loft(rings, (0, 0.846, 0.0), (0, 1.694, 0.008))
    return s.move(0, PLATE_T, 0)


def torso_ring(y, grow=0.0):
    """The torso's section at y (between two specs), grown outward."""
    for (y0, *p0), (y1, *p1) in zip(TORSO, TORSO[1:]):
        if y0 <= y <= y1:
            t = (y - y0) / (y1 - y0)
            cz, a, bf, bb = (u + (w - u) * t for u, w in zip(p0, p1))
            return [Vector((x, y + PLATE_T, cz + z)) for x, z in ring2(a + grow, bf + grow, bb + grow, TORSO_N, torso_power(y))]
    raise ValueError(y)


HEAD = [
    (1.648, 0.040, 0.022, 0.018, 0.018),
    (1.665, 0.026, 0.045, 0.044, 0.036),
    (1.690, 0.012, 0.060, 0.068, 0.060),
    (1.720, 0.005, 0.068, 0.083, 0.080),
    (1.750, 0.000, 0.073, 0.090, 0.092),
    (1.780, -0.005, 0.076, 0.090, 0.100),
    (1.810, -0.008, 0.075, 0.085, 0.100),
    (1.840, -0.012, 0.066, 0.072, 0.088),
    (1.865, -0.014, 0.050, 0.053, 0.066),
    (1.880, -0.015, 0.028, 0.029, 0.038),
]


def nose_rise(y):
    """The face's one feature: the hint of a nose under the hint of a brow."""
    return 0.0065 * math.exp(-((y - 1.738) / 0.016) ** 2) + 0.0022 * math.exp(-((y - 1.772) / 0.010) ** 2)


def man_head():
    n = 24
    rings = []
    for (y, cz, a, bf, bb) in HEAD:
        ring = []
        for x, z in ring2(a, bf, bb, n, 2.05):
            if z > 0:
                z += nose_rise(y) * math.exp(-(x / 0.017) ** 2)
            ring.append((x, y, cz + z))
        rings.append(ring)
    s = loft(rings, (0, 1.641, 0.042), (0, 1.887, -0.016))
    return s.move(0, PLATE_T, 0)


# a leg (its right, at -x): path and (a, bf, bb)
LEG = [
    ((-0.105, 0.085, -0.020), (0.026, 0.030, 0.030)),
    ((-0.105, 0.120, -0.018), (0.028, 0.028, 0.031)),
    ((-0.104, 0.180, -0.016), (0.032, 0.031, 0.037)),
    ((-0.103, 0.260, -0.014), (0.040, 0.034, 0.048)),
    ((-0.102, 0.340, -0.010), (0.048, 0.038, 0.058)),
    ((-0.100, 0.420, -0.004), (0.046, 0.040, 0.050)),
    ((-0.098, 0.500, 0.005), (0.047, 0.050, 0.044)),
    ((-0.097, 0.560, 0.006), (0.052, 0.053, 0.047)),
    ((-0.096, 0.660, 0.004), (0.062, 0.062, 0.057)),
    ((-0.094, 0.760, 0.002), (0.071, 0.070, 0.065)),
    ((-0.091, 0.860, 0.000), (0.079, 0.075, 0.075)),
    ((-0.086, 0.940, -0.004), (0.074, 0.074, 0.080)),
    ((-0.078, 1.000, -0.006), (0.062, 0.060, 0.066)),
]
# the foot along z from the heel: (z, y_bottom, y_top, half width)
FOOT = [
    (-0.068, 0.006, 0.040, 0.022),
    (-0.058, 0.000, 0.058, 0.031),
    (-0.035, 0.000, 0.080, 0.035),
    (0.000, 0.000, 0.078, 0.038),
    (0.040, 0.000, 0.062, 0.041),
    (0.080, 0.000, 0.047, 0.045),
    (0.120, 0.000, 0.036, 0.048),
    (0.148, 0.000, 0.029, 0.046),
    (0.172, 0.001, 0.024, 0.041),
    (0.188, 0.003, 0.018, 0.032),
]


def man_foot():
    n = 16
    rings = []
    for (z, y0, y1, a) in FOOT:
        mid, half = (y0 + y1) / 2, (y1 - y0) / 2
        ring = []
        for (x, w) in ring2(a, 1.0, 1.0, n, 2.5):
            # a flat sole: the bottom squarer than the instep
            ww = w if w > 0 else -abs(w) ** 0.7
            ring.append((x, mid + ww * half, z))
        rings.append(ring)
    s = loft(rings, (0, 0.022, -0.072), (0, 0.010, 0.196))
    # its toe out a little, about the ankle, under the leg
    return s.rot(0, -0.13, 0).move(-0.105, 0, -0.012)


def man_leg_r():
    path = [p for p, _ in LEG]
    secs = [q for _, q in LEG]
    return (limb(path, secs, 20) + man_foot()).move(0, PLATE_T, 0)


def mirror(s):
    return s.copy().scale(-1, 1, 1)


# an arm (its right, at -x), from the wrist up to the shoulder cap
ARM = [
    ((-0.242, 0.912, 0.040), (0.019, 0.027, 0.027)),
    ((-0.241, 0.960, 0.033), (0.023, 0.031, 0.031)),
    ((-0.239, 1.020, 0.020), (0.029, 0.034, 0.034)),
    ((-0.236, 1.100, 0.002), (0.038, 0.039, 0.038)),
    ((-0.233, 1.155, -0.010), (0.035, 0.034, 0.037)),
    ((-0.229, 1.210, -0.010), (0.038, 0.039, 0.038)),
    ((-0.223, 1.300, -0.006), (0.043, 0.046, 0.043)),
    ((-0.215, 1.380, 0.000), (0.048, 0.050, 0.048)),
    ((-0.208, 1.430, 0.000), (0.050, 0.055, 0.054)),
    ((-0.200, 1.462, -0.002), (0.046, 0.052, 0.050)),
    ((-0.188, 1.480, -0.004), (0.039, 0.046, 0.044)),
    ((-0.172, 1.488, -0.005), (0.029, 0.036, 0.034)),
    ((-0.156, 1.486, -0.006), (0.019, 0.025, 0.023)),
]


def man_arm_r():
    path = [p for p, _ in ARM]
    secs = [q for _, q in ARM]
    return limb(path, secs, 18).move(0, PLATE_T, 0)


def man_hand_r():
    # the palm, edge on to the front (palm toward +x, the thigh)
    palm = column([
        (0.812, -0.240, 0.045, 0.012, 0.036, 0.034),
        (0.840, -0.242, 0.045, 0.016, 0.043, 0.038),
        (0.870, -0.243, 0.044, 0.017, 0.039, 0.036),
        (0.900, -0.243, 0.042, 0.016, 0.030, 0.030),
        (0.925, -0.242, 0.040, 0.016, 0.022, 0.022),
    ], 14, 2.4, bottom=(-0.240, 0.804, 0.046), top=(-0.242, 0.931, 0.040))
    fingers = []
    # index, middle, ring, little: their knuckles along z, their lengths
    for dz, ln, r in ((0.028, 0.078, 0.0090), (0.009, 0.088, 0.0094), (-0.010, 0.082, 0.0090), (-0.027, 0.064, 0.0080)):
        x0, y0, z0 = -0.241, 0.826, 0.045 + dz
        pts = [(x0, y0, z0), (x0 + 0.002, y0 - ln * 0.40, z0), (x0 + 0.008, y0 - ln * 0.72, z0 + 0.001), (x0 + 0.019, y0 - ln * 0.95, z0 + 0.002)]
        fingers.append(tube(pts, r, seg=6, caps=True, r_end=r * 0.8))
    thumb = tube([(-0.236, 0.896, 0.072), (-0.229, 0.872, 0.083), (-0.224, 0.848, 0.085), (-0.222, 0.833, 0.082)], 0.0102, seg=6, caps=True, r_end=0.0082)
    return (palm + sum(fingers) + thumb).move(0, PLATE_T, 0)


def seam_loop(pts, r=0.0011):
    pts = list(pts) + [pts[0]]
    return tube(pts, r, seg=3, caps=False)


def traced_join(table, n, start, steps=2):
    """Where a limb (its path and sections, `table`) goes into the torso:
    up each line of its surface (n round it, and halfway between) from the
    section `start` to the first point inside the torso, then halved down
    to the surface; a loop of points on the limb, grown a little."""
    torso = man_torso()
    bvh = BVHTree.FromPolygons([p.copy() for p in torso.v], torso.f)

    def inside(p):
        loc, nrm, _, _ = bvh.find_nearest(p)
        return (p - loc).dot(nrm) < 0

    path = [p for p, _ in table]
    secs = [(a + 0.0005, bf + 0.0005, bb + 0.0005) for _, (a, bf, bb) in table]
    _, rings = limb_rings(path, secs, n)
    lift = Vector((0, PLATE_T, 0))
    out = []
    for i in range(n * steps):
        j, part = divmod(i, steps)

        def at(k):
            return rings[k][j].lerp(rings[k][(j + 1) % n], part / steps) + lift

        k = start
        while k < len(rings) - 1 and not inside(at(k)):
            k += 1
        lo, hi = at(k - 1), at(k)
        for _ in range(16):
            mid = lo.lerp(hi, 0.5)
            if inside(mid):
                hi = mid
            else:
                lo = mid
        out.append(lo)
    return out


def man_seams():
    loops = []
    # the waist and the neck
    loops.append(seam_loop(torso_ring(1.100, 0.0006)))
    loops.append(seam_loop(torso_ring(1.585, 0.0006)))
    # the wrists
    path = [p for p, _ in ARM]
    secs = [(a + 0.0006, bf + 0.0006, bb + 0.0006) for _, (a, bf, bb) in ARM]
    _, rings = limb_rings(path, secs, 18)
    wrist = [p + Vector((0, PLATE_T, 0)) for p in rings[0]]
    loops.append(seam_loop(wrist))
    loops.append(mirror(seam_loop(wrist)))
    return sum(loops[1:], loops[0])


def man_joins(table, n, start, steps):
    """Where an arm or a leg goes into the torso, both sides."""
    ring = traced_join(table, n, start, steps)
    return seam_loop(ring, 0.0013) + mirror(seam_loop(ring, 0.0013))


def body_parts():
    """Every fibreglass part, as placed (for where the chips go)."""
    if not hasattr(body_parts, 'memo'):
        legs = man_leg_r()
        arm = man_arm_r()
        hand = man_hand_r()
        body_parts.memo = [man_torso(), man_head(), legs, mirror(legs), arm, mirror(arm), hand, mirror(hand)]
    return body_parts.memo


# where the paint is knocked off: (target, size)
CHIPS = [
    ((-0.098, 0.51, 0.06), 0.011), ((0.098, 0.50, 0.06), 0.008),       # knees
    ((-0.22, 1.52, 0.00), 0.010), ((0.21, 1.515, -0.02), 0.006),       # shoulder caps
    ((-0.235, 1.165, -0.06), 0.009), ((0.235, 1.15, -0.06), 0.012),     # elbows
    ((0.03, 1.895, 0.0), 0.012),                                         # the crown
    ((-0.125, 0.04, 0.21), 0.008), ((0.13, 0.04, 0.20), 0.010),          # toes
    ((-0.105, 0.04, -0.09), 0.009), ((0.108, 0.05, -0.09), 0.006),       # heels
    ((-0.24, 0.835, 0.083), 0.006), ((0.235, 0.83, 0.08), 0.007),        # knuckles
    ((-0.218, 0.75, 0.05), 0.004), ((0.22, 0.76, 0.06), 0.005),          # fingertips
    ((0.17, 0.97, 0.03), 0.010), ((-0.165, 0.93, 0.0), 0.007),            # hips
    ((0.09, 1.37, 0.13), 0.006), ((-0.06, 1.25, 0.12), 0.004),           # chest
    ((-0.10, 0.97, -0.13), 0.009),                                        # buttock
    ((0.108, 0.28, 0.04), 0.008), ((-0.10, 0.20, 0.035), 0.005),          # shins
    ((0.05, 1.12, 0.10), 0.005),                                          # the waist's edge
]


def man_chips():
    parts = body_parts()
    allv, allf = [], []
    for s in parts:
        n = len(allv)
        allv += [p.copy() for p in s.v]
        allf += [tuple(i + n for i in f) for f in s.f]
    bvh = BVHTree.FromPolygons(allv, allf)
    out = []
    for k, (target, size) in enumerate(CHIPS):
        tgt = Vector(target) + Vector((0, PLATE_T, 0))
        loc, nrm, _, _ = bvh.find_nearest(tgt)
        if loc is None:
            continue
        u = nrm.cross(Vector((0, 1, 0)) if abs(nrm.y) < 0.9 else Vector((1, 0, 0))).normalized()
        w = nrm.cross(u)
        m = 9
        ring = []
        for i in range(m):
            a = TAU * i / m + hrand(k, i) * 0.4
            r = size * (0.45 + 0.55 * hrand(k, i + 50)) * (1.0 + 0.35 * math.cos(2 * a + hrand(k, 99) * 6))
            p = loc + (u * math.cos(a) + w * math.sin(a)) * r
            l2, n2, _, _ = bvh.find_nearest(p)
            if n2.dot(nrm) < 0:
                n2 = -n2
            ring.append(l2 + n2 * 0.0005)
        v = [loc + nrm * 0.0005] + ring
        f = []
        for i in range(m):
            tri = (0, 1 + i, 1 + (i + 1) % m)
            a, b, c = v[tri[0]], v[tri[1]], v[tri[2]]
            if (b - a).cross(c - a).dot(nrm) < 0:
                tri = (tri[0], tri[2], tri[1])
            f.append(tri)
        out.append(Shape(v, f))
    return sum(out[1:], out[0])


def man_plate():
    plate = cyl(0.200, h=PLATE_T, seg=48, bevel_r=0.0028, bevel_seg=2).move(0, PLATE_T / 2, 0.030)
    boss = cyl(0.016, 0.018, h=0.012, seg=16, bevel_r=0.0015, bevel_seg=1).move(-0.103, PLATE_T + 0.006, -0.090)
    return plate + boss


def man_rod():
    rod = tube([(-0.103, PLATE_T + 0.004, -0.090), (-0.103, 0.22, -0.090), (-0.103, 0.272, -0.090),
                (-0.103, 0.292, -0.086), (-0.103, 0.303, -0.072), (-0.103, 0.306, -0.046)], 0.006, seg=10, smooth=1)
    # the socket's plate let into the back of the calf
    sock = cyl(0.013, h=0.003, seg=14, bevel_r=0.001, bevel_seg=1).rot(math.pi / 2, 0, 0).move(-0.103, 0.306, -0.0655)
    return rod + sock


# ================================================================== clock ===
def clock_case():
    prof = [(0.0, 0.0), (0.1630, 0.0), (0.1672, 0.0022), (0.1680, 0.0060), (0.1680, 0.0390),
            (0.1695, 0.0435), (0.1682, 0.0475), (0.1638, 0.0498), (0.1570, 0.0497),
            (0.1527, 0.0472), (0.1505, 0.0430), (0.1497, 0.0300)]
    s = lathe(prof, seg=40, cap_bottom=False, cap_top=False)
    return s.rot(math.pi / 2, 0, 0)


def clock_glass():
    R, sag, z0 = 0.1506, 0.0075, 0.0425
    rc = (R * R + sag * sag) / (2 * sag)
    prof = []
    for i in range(7):
        r = R * (1 - i / 7)
        prof.append((r, z0 + math.sqrt(rc * rc - r * r) - (rc - sag)))
    prof.append((0.0, z0 + sag))
    s = lathe(prof, seg=40, cap_bottom=False, cap_top=False)
    return s.rot(math.pi / 2, 0, 0)


def hand_blade(tail, tip, w0, w1, t):
    """A tapered blade from -tail to tip along y, w0 wide at the pivot and
    w1 near its rounded tip, t thick, its back at z = 0."""
    pts = [(-w0 * 0.8, -tail), (w0 * 0.8, -tail), (w0, 0.0), (w1, tip - w1 * 1.5), (w1 * 0.55, tip - w1 * 0.3),
           (0.0, tip), (-w1 * 0.55, tip - w1 * 0.3), (-w1, tip - w1 * 1.5), (-w0, 0.0)]
    return prism(pts, t).move(0, 0, t / 2)


def clock_hour():
    blade = hand_blade(0.020, 0.086, 0.0068, 0.0046, 0.0008)
    collet = cyl(0.0078, h=0.0012, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0006)
    return blade + collet


def clock_minute():
    blade = hand_blade(0.026, 0.127, 0.0052, 0.0031, 0.0008)
    collet = cyl(0.0062, h=0.0012, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0006)
    return blade + collet


def clock_second():
    needle = prism([(-0.0011, -0.034), (0.0011, -0.034), (0.0007, 0.134), (-0.0007, 0.134)], 0.0005).move(0, 0, 0.00025)
    weight = cyl(0.0056, h=0.0006, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, -0.031, 0.0003)
    hub = cyl(0.0042, h=0.0012, seg=12, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0006)
    return needle + weight + hub


def clock_hub():
    s = lathe([(0.0, 0.0), (0.0046, 0.0), (0.0046, 0.0016), (0.0036, 0.0032), (0.0, 0.0037)], seg=16, cap_bottom=False, cap_top=False)
    return s.rot(math.pi / 2, 0, 0)


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


auto('Real__noteSheet', note_sheet, 400)
auto('Real__noteCorner', note_corner, 100)

auto('Real__manTorso', man_torso, 1100)
auto('Real__manHead', man_head, 560)
auto('Real__manLegR', man_leg_r, 1100)
auto('Real__manLegL', lambda: mirror(man_leg_r()), 1100)
auto('Real__manArmR', man_arm_r, 1100)
auto('Real__manArmL', lambda: mirror(man_arm_r()), 1100)
auto('Real__manHandR', man_hand_r, 560)
auto('Real__manHandL', lambda: mirror(man_hand_r()), 560)
auto('Real__manSeams', man_seams, 1100)
auto('Real__manJoins', lambda: man_joins(ARM, 18, 5, 2) + man_joins(LEG, 20, 7, 3), 1200)
auto('Real__manChips', man_chips, 560)
auto('Real__manPlate', man_plate, 1100)
auto('Real__manRod', man_rod, 560)

auto('Real__clockCase', clock_case, 1100)
auto('Real__clockGlass', clock_glass, 560)
auto('Real__clockHour', clock_hour, 280)
auto('Real__clockMinute', clock_minute, 280)
auto('Real__clockSecond', clock_second, 280)
auto('Real__clockHub', clock_hub, 100)
