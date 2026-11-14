"""Family `vents_sockets_signs_and_litter` — the "core" theme: things found everywhere (vents, sockets, signs,
fixtures, litter). Decals, transparent and vertex-edited primitives are left
alone (see docs/research/props-remodelling/vents_sockets_signs_and_litter.md); every solid one gets a modelled part here.
"""
import propkit
from propkit import *
from mathutils import Vector, noise
import bmesh

fixw = propkit._fix_winding


# ------------------------------------------------------------- helpers ---
def _ccw(pts):
    a = sum(p[0] * q[1] - q[0] * p[1] for p, q in zip(pts, pts[1:] + pts[:1]))
    return list(pts) if a > 0 else list(reversed(pts))


def offset2d(pts, d):
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


def rr(w, h, r, seg=2):
    """Rounded-rect outline generator for stack(): inset -> points."""
    return lambda i: rounded_rect(w - 2 * i, h - 2 * i, max(r - i, 0.0003), seg)


def outl(pts):
    base = _ccw(pts)
    return lambda i: base if abs(i) < 1e-9 else offset2d(base, i)


def stack(outline, prof, loop=False):
    """Moulding: for each (inset, z) in prof the outline, inset, at depth z;
    consecutive rings are bridged. Ends are capped, or with loop=True the last
    ring joins the first (a frame/bezel with a hole)."""
    v, f = [], []
    rings = [outline(i) for i, _ in prof]
    n = len(rings[0])
    for ring, (_, z) in zip(rings, prof):
        v += [(x, y, z) for x, y in _ccw(ring)]
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


def ring_lathe(profile, seg=24):
    """Revolve a CLOSED profile loop [(r, y), ...] around +Y (rings, rims, sleeves)."""
    v, f, n = [], [], len(profile)
    for r, y in profile:
        for i in range(seg):
            a = TAU * i / seg
            v.append((r * math.sin(a), y, r * math.cos(a)))
    for k in range(n):
        a, b = k * seg, ((k + 1) % n) * seg
        for i in range(seg):
            j = (i + 1) % seg
            f.append((a + i, b + i, b + j, a + j))
    return fixw(Shape(v, f))


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


def hull(points):
    bm = bmesh.new()
    for p in points:
        bm.verts.new(p)
    bmesh.ops.convex_hull(bm, input=list(bm.verts))
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context='VERTS')
    s = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return fixw(s)


def interp(tab, x):
    """Piecewise-linear lookup in [(x, y), ...]."""
    if x <= tab[0][0]:
        return tab[0][1]
    for (x0, y0), (x1, y1) in zip(tab, tab[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * (x - x0) / ((x1 - x0) or 1)
    return tab[-1][1]


def small_screw(r, h, seg=8):
    """A cheaper slotted screw head (8 sides) for small plates."""
    s = lathe([(0, 0), (r, 0), (r, h * 0.35), (r * 0.65, h * 0.9), (0, h)], seg=seg)
    return cut(s, box(r * 2.4, h, r * 0.35).move(y=h * 0.95))


def face_z(s):
    """Rotate a +Y-facing detail (screw head, lathe) to face +Z."""
    return s.rot(x=math.pi / 2)


def crumple(w, h, d, seed, jag=0.14, sub=2):
    """A crumpled paper / wrapper wad: a subdivided ball, every vertex pushed
    in or out at random (sharp creases) plus a broad lumpy warp."""
    s = subdivide(box(1, 1, 1), sub)
    rnd = random.Random(seed)
    s.warp(lambda p: p.normalized() * 0.5 * (1 - jag + 2 * jag * rnd.random()))
    s = noisy(s, 0.09, scale=2.2, seed=seed)
    return s.fit((-w / 2, -h / 2, -d / 2), (w / 2, h / 2, d / 2))


# ------------------------------------------------------------ scrap paper --
part('ScrapPapers__wad', f_box(0.085, 0.055, 0.085), lambda: crumple(0.085, 0.055, 0.085, 11), flat=True)


# -------------------------------------------------------------- wall vent --
def vent_grille():
    # stamped flange around a 0.24 × 0.14 opening, a duct sleeve behind it and
    # six angled louvres (the dark opening plane shows between them)
    flange = stack(rr(0.30, 0.20, 0.012, 2),
                   [(0, 0.004), (0, 0.013), (0.003, 0.018), (0.008, 0.020), (0.020, 0.020),
                    (0.026, 0.017), (0.030, 0.012), (0.030, 0.004)], loop=True)
    sleeve = stack(rr(0.246, 0.146, 0.003, 1), [(0, -0.02), (0, 0.006), (0.003, 0.006), (0.003, -0.02)], loop=True)
    blades = Shape()
    for k in range(6):
        y = -0.07 + 0.14 * (k + 0.5) / 6
        blades = blades + box(0.24, 0.024, 0.0022).rot(x=math.radians(48)).move(y=y, z=0.006)
    screws = sum((face_z(screw_head(0.0035, 0.0014)).move(x=sx * 0.136, z=0.0194) for sx in (-1, 1)), Shape())
    return flange + sleeve + blades + screws

part('WallVents__grille', f_box(0.30, 0.20, 0.04), vent_grille)


# ------------------------------------------------------------------ shoes --
def _sole_outline():
    tab = [(-0.14, 0.036), (-0.09, 0.040), (-0.03, 0.035), (0.03, 0.049), (0.07, 0.055), (0.11, 0.050), (0.14, 0.040)]
    N = 13
    xs = [-0.14 + 0.28 * (1 - math.cos(math.pi * i / (N - 1))) / 2 for i in range(N)]
    env = lambda x: math.sqrt(max(0.0, 1 - (x / 0.14) ** 6))
    top = [(x, interp(tab, x) * env(x)) for x in xs]
    bot = [(x, -(interp(tab, x) - (0.006 if -0.07 < x < 0.03 else 0.0)) * env(x)) for x in reversed(xs[1:-1])]
    return top + bot

def shoe_sole():
    s = stack(outl(_sole_outline()),
              [(0.004, -0.015), (0.001, -0.0145), (0, -0.0125), (0, -0.004), (0.0016, -0.003),
               (0.0016, -0.0015), (0, -0.0005), (0, 0.0135), (0.0015, 0.015)])
    s.rot(x=-math.pi / 2)
    def spring(p):                       # toe and heel lift off the floor a little
        if p.y < 0:
            p.y += 0.005 * max(0.0, (p.x - 0.06) / 0.08) ** 2 + 0.002 * max(0.0, (-p.x - 0.1) / 0.04) ** 2
        return p
    return s.warp(spring)

def _dring(x, hw, ytop, ybot, m=10, p=0.55):
    pts = []
    for k in range(m):
        a = math.pi * k / (m - 1)
        pts.append((x, ybot + (ytop - ybot) * abs(math.sin(a)) ** p, hw * math.cos(a)))
    return pts

def shoe_upper():
    xs = [-0.12, -0.117, -0.11, -0.095, -0.07, -0.04, -0.01, 0.02, 0.05, 0.08, 0.10, 0.12]
    hw = [(-0.12, 0.022), (-0.117, 0.033), (-0.11, 0.040), (-0.095, 0.044), (-0.05, 0.046), (0.0, 0.048), (0.06, 0.050), (0.12, 0.047)]
    yt = [(-0.12, 0.036), (-0.11, 0.040), (-0.095, 0.039), (-0.06, 0.026), (-0.02, 0.036), (0.0, 0.037),
          (0.04, 0.029), (0.08, 0.020), (0.10, 0.016), (0.12, 0.011)]
    s = loft([_dring(x, interp(hw, x), interp(yt, x), -0.04) for x in xs])
    opening = cyl(1, 1, 0.08, seg=20, bevel_r=0).scale(0.048, 1, 0.031).move(x=-0.052, y=0.035)
    s = cut(s, opening)
    laces = Shape()
    for x in (0.012, 0.03, 0.048, 0.066):
        y0, y1 = interp(yt, x - 0.01), interp(yt, x + 0.01)
        laces = laces + rbox(0.005, 0.0035, 0.052, r=0.0012, seg=1).rot(z=math.atan2(y1 - y0, 0.02)).move(x=x, y=interp(yt, x) - 0.0006)
    return s + laces

def shoe_toe():
    xs = [-0.04, -0.02, 0.0, 0.015, 0.026, 0.034, 0.039, 0.0405]
    hw = [0.040, 0.040, 0.038, 0.034, 0.028, 0.020, 0.012, 0.005]
    yt = [0.025, 0.023, 0.018, 0.012, 0.006, 0.0, -0.006, -0.009]
    return loft([_dring(x, w, y, -0.025, m=12, p=0.5) for x, w, y in zip(xs, hw, yt)]).fit((-0.04, -0.025, -0.04), (0.04, 0.025, 0.04))

part('DroppedShoes__sole', f_box(0.28, 0.03, 0.11), shoe_sole)
part('DroppedShoes__upper', f_box(0.24, 0.08, 0.10), shoe_upper, smooth=60)
part('DroppedShoes__toe', f_box(0.08, 0.05, 0.08), shoe_toe, smooth=180)


# ------------------------------------------------------------ wall socket --
def socket_plate():
    plate = stack(rr(0.10, 0.07, 0.008, 2), [(0, -0.0075), (0, -0.003), (0.0015, 0.0), (0.004, 0.0015), (0.008, 0.002)])
    boss = stack(rr(0.064, 0.048, 0.010, 3), [(0, 0.0), (0, 0.0055), (0.0015, 0.0075), (0.004, 0.0075)])
    slots = [rbox(0.0045, 0.014, 0.012, r=0.0008, seg=1).move(x=sx * 0.018, z=0.0075) for sx in (-1, 1)]
    ground = cyl(0.0032, 0.0032, 0.012, seg=10, bevel_r=0).rot(x=math.pi / 2).move(y=-0.015, z=0.0075)
    return plate + cut(boss, *slots, ground)

part('WallSockets__plate', f_box(0.10, 0.07, 0.015), socket_plate)


# --------------------------------------------------------- cigarette butts --
def butt_burnt():
    s = lathe([(0.0, -0.0074), (0.0017, -0.008), (0.0024, -0.0071), (0.0025, -0.004), (0.0025, 0.008), (0.0, 0.008)], seg=8)
    rnd = random.Random(5)
    s.warp(lambda p: Vector((p.x, p.y + (rnd.uniform(0, 0.0014) if p.y < -0.006 else 0.0), p.z * 0.88)))
    return s.rot(z=-math.pi / 2)

def butt_filter():
    s = lathe([(0.0, -0.005), (0.0025, -0.005), (0.0025, 0.0041), (0.0021, 0.005), (0.0009, 0.0047), (0.0, 0.0045)], seg=8)
    return s.scale(1, 1, 0.9).rot(z=-math.pi / 2)

part('CigaretteButts__burnt', f_box(0.016, 0.005, 0.005), butt_burnt, smooth=60)
part('CigaretteButts__filter', f_box(0.010, 0.005, 0.005), butt_filter, smooth=60)


# ------------------------------------------------------- cracked wall hole --
def wall_hole_rim():
    # broken drywall: a jagged torn outline, a paper lip, crumbly gypsum
    # dished back towards the wall (thickness only behind z = 0)
    n, rnd = 32, random.Random(7)
    jag = [0.87 + 0.13 * rnd.random() for _ in range(n)]
    rings = [(1.0, -0.0034), (0.93, 0.0), (0.85, -0.0012), (0.70, -0.0024), (0.50, -0.0033), (0.26, -0.0038)]
    v, f = [], []
    for k, (rho, z) in enumerate(rings):
        for i in range(n):
            a = TAU * i / n
            j = jag[i] * (1 + (0.08 * (rnd.random() - 0.5) if k > 1 else 0))
            zz = z + (rnd.uniform(-0.0006, 0.0006) if k > 1 else 0)
            v.append((0.09 * rho * j * math.cos(a), 0.075 * rho * j * math.sin(a), zz))
    v.append((0.0, 0.0, -0.0039))
    c = len(v) - 1
    for k in range(len(rings) - 1):
        for i in range(n):
            a, b = k * n + i, k * n + (i + 1) % n
            f.append((a, b, b + n, a + n))
    last = (len(rings) - 1) * n
    for i in range(n):
        f.append((last + i, last + (i + 1) % n, c))
    f.append(tuple(reversed(range(n))))           # flat back against the wall
    s = fixw(Shape(v, f))
    lo, hi = s.bounds()
    return s.fit((-0.09, -0.075, lo.z), (0.09, 0.075, hi.z))

part('CrackedWallHoles__rim', f_bounds((-0.09, -0.075, 0), (0.09, 0.075, 0), 'plane'), wall_hole_rim)


# ------------------------------------------------------ fire extinguisher --
def ext_body():
    # drawn steel cylinder: domed shoulder up to the valve, a rolled foot ring
    return lathe([(0, -0.203), (0.05, -0.205), (0.078, -0.21), (0.086, -0.209), (0.09, -0.203), (0.09, -0.186),
                  (0.0886, -0.183), (0.0886, -0.179), (0.09, -0.176), (0.09, 0.13), (0.088, 0.155), (0.082, 0.175),
                  (0.071, 0.192), (0.055, 0.204), (0.04, 0.209), (0.03, 0.21), (0, 0.21)], seg=28)

def ext_valve():
    # threaded collar on the neck, the valve body, a small pressure gauge
    v = lathe([(0, -0.03), (0.05, -0.03), (0.05, -0.021), (0.047, -0.017), (0.036, -0.016), (0.034, -0.008),
               (0.030, -0.004), (0.030, 0.026), (0.027, 0.03), (0, 0.03)], seg=24)
    gauge = cyl(0.011, 0.011, 0.008, seg=12, bevel_r=0.0015).rot(z=-math.pi / 2).move(x=0.036, y=0.012)
    return v + gauge

def ext_lever():
    # squeeze lever over the carry handle, on a pivot pin
    top = prism([(-0.045, 0.001), (-0.02, 0.006), (0.02, 0.0093), (0.05, 0.010), (0.05, 0.005), (0.02, 0.0038),
                 (-0.02, 0.0008), (-0.045, -0.003)], 0.024, bevel_r=0.0015)
    low = prism([(-0.05, -0.004), (-0.02, -0.0035), (0.02, -0.005), (0.05, -0.006), (0.05, -0.010), (0.02, -0.0092),
                 (-0.02, -0.0078), (-0.05, -0.008)], 0.03, bevel_r=0.0015)
    pin = cyl(0.0042, 0.0042, 0.03, seg=10, bevel_r=0.001).rot(x=math.pi / 2).move(x=-0.036, y=-0.001)
    return top + low + pin

def ext_hose():
    # discharge hose stub: crimped ferrule at the valve end, a flared horn tip
    return lathe([(0, -0.066), (0.0042, -0.07), (0.0066, -0.069), (0.008, -0.062), (0.008, -0.05), (0.0066, -0.047),
                  (0.006, -0.044), (0.006, 0.055), (0.0072, 0.057), (0.0074, 0.07), (0, 0.07)], seg=14)

def ext_bracket():
    # wall hook: back plate, an arm with an upturned lip, a brace, two screws
    plate = rbox(0.008, 0.12, 0.03, r=0.002, seg=1).move(x=-0.071)
    arm = rbox(0.14, 0.009, 0.024, r=0.002, seg=1).move(x=0.0, y=0.01)
    lip = rbox(0.009, 0.036, 0.024, r=0.002, seg=1).move(x=0.0705, y=0.0235)
    brace = rbox(0.085, 0.008, 0.02, r=0.002, seg=1).rot(z=math.atan2(0.05, 0.062)).move(x=-0.036, y=-0.019)
    screws = sum((screw_head(0.0045, 0.0018).rot(z=-math.pi / 2).move(x=-0.067, y=sy) for sy in (-0.045, 0.045)), Shape())
    return plate + arm + lip + brace + screws

part('FireExtinguishers__body', f_cyl(0.09, 0.09, 0.42), ext_body)
part('FireExtinguishers__valve', f_cyl(0.03, 0.05, 0.06), ext_valve)
part('FireExtinguishers__lever', f_box(0.10, 0.02, 0.03), ext_lever)
part('FireExtinguishers__hose', f_cyl(0.008, 0.008, 0.14), ext_hose)
part('FireExtinguishers__bracket', f_box(0.15, 0.12, 0.03), ext_bracket)


# -------------------------------------------------------------- exit sign --
def exit_housing():
    # moulded housing: back box, a parting groove, a raised bezel round an
    # inset face panel, two screws in the lower bezel
    s = stack(rr(0.42, 0.20, 0.014, 3),
              [(0.003, -0.02), (0, -0.017), (0, 0.004), (0.0012, 0.005), (0.0012, 0.0065), (0, 0.0075), (0, 0.0165),
               (0.003, 0.0195), (0.006, 0.02), (0.013, 0.02), (0.016, 0.0175), (0.018, 0.017)])
    screws = sum((face_z(screw_head(0.003, 0.0012)).move(x=sx * 0.19, y=-0.0905, z=0.0192) for sx in (-1, 1)), Shape())
    return s + screws

part('ExitSigns__housing', f_box(0.42, 0.20, 0.04), exit_housing)


# ------------------------------------------------------------- thermostat --
def thermo_housing():
    # rounded-square wall thermostat: pillowed cover over a base, a parting
    # line, a circular groove round the dial
    s = stack(rr(0.12, 0.12, 0.022, 4),
              [(0.002, -0.015), (0, -0.013), (0, -0.002), (0.0012, -0.001), (0, 0.0),
               (0, 0.004), (0.004, 0.010), (0.010, 0.0135), (0.018, 0.015)])
    groove = face_z(ring_lathe([(0.0425, -0.004), (0.046, -0.004), (0.046, 0.004), (0.0425, 0.004)], seg=20)).move(z=0.015)
    return cut(s, groove)

part('Thermostats__housing', f_box(0.12, 0.12, 0.03), thermo_housing)


# -------------------------------------------------------------- trash can --
def _can_r(y):
    return 0.15 + 0.01 * (y + 0.25) / 0.5

def can_body():
    # galvanised can lying on its side: built upright (base down, mouth up)
    # with two pressed bands, then flipped so the open mouth is at -Y
    prof = [(0, -0.244), (0.135, -0.244), (0.14, -0.25), (0.148, -0.249), (0.1505, -0.24)]
    for yc in (-0.115, 0.1):
        prof += [(_can_r(yc - 0.016), yc - 0.016), (_can_r(yc) + 0.0045, yc - 0.008), (_can_r(yc) + 0.0045, yc + 0.008),
                 (_can_r(yc + 0.016), yc + 0.016)]
    prof += [(0.16, 0.25), (0.1555, 0.25), (0.1525, 0.24), (0.146, -0.236), (0, -0.236)]
    return lathe(prof, seg=28).rot(x=math.pi)

def can_rim():
    # rolled bead round the mouth (hollow, so the can stays open)
    return ring_lathe([(0.147, -0.006), (0.152, -0.01), (0.157, -0.009), (0.16, -0.005), (0.16, 0.005),
                       (0.157, 0.009), (0.152, 0.01), (0.147, 0.006)], seg=32)

def bag_body():
    # a full bin bag: sagging bulge, pleats gathered up into the neck, lumps
    s = lathe([(0, -0.09), (0.07, -0.09), (0.10, -0.082), (0.11, -0.06), (0.11, -0.03), (0.104, 0.0),
               (0.09, 0.03), (0.066, 0.055), (0.04, 0.074), (0.02, 0.085), (0, 0.09)], seg=22)
    def pleat(p):
        t = min(1.0, max(0.0, (p.y + 0.02) / 0.1))
        a = math.atan2(p.x, p.z)
        k = 1 + 0.12 * t * math.sin(7 * a + 0.6)
        return Vector((p.x * k, p.y, p.z * k))
    s = noisy(s.warp(pleat), 0.007, scale=16, seed=3)
    return s.fit((-0.11, -0.09, -0.10), (0.11, 0.09, 0.10))

def bag_knot():
    # gathered neck, the tie, a ruffled tuft of plastic above it
    s = lathe([(0, -0.03), (0.03, -0.03), (0.018, -0.015), (0.011, 0.0), (0.013, 0.006), (0.032, 0.02),
               (0.04, 0.029), (0.026, 0.03), (0, 0.022)], seg=16)
    def ruffle(p):
        a = math.atan2(p.x, p.z)
        t = min(1.0, max(0.0, (p.y - 0.004) / 0.02))
        k = 1 + 0.12 * t * math.sin(5 * a) + 0.10 * (1 - t) * math.sin(9 * a)
        return Vector((p.x * k, p.y + 0.003 * t * math.sin(5 * a + 1.0), p.z * k))
    s = noisy(s.warp(ruffle), 0.0025, scale=60, seed=4)
    return s.fit((-0.04, -0.03, -0.04), (0.04, 0.03, 0.04))

def bag_lump():
    s = sphere(0.5, seg=16).scale(1, 0.8, 1)
    s = noisy(s, 0.06, scale=2.5, seed=9)
    s.warp(lambda p: Vector((p.x, max(p.y, -0.34 + 0.05 * p.y), p.z)))
    return s.fit((-0.07, -0.05, -0.06), (0.07, 0.05, 0.06))

part('TrashCans__body', f_cyl(0.15, 0.16, 0.50), can_body)
part('TrashCans__rim', f_cyl(0.16, 0.16, 0.02), can_rim)
part('TrashCans__litter', f_box(0.08, 0.05, 0.08), lambda: crumple(0.08, 0.05, 0.08, 23, jag=0.18), flat=True)
part('TrashCans__bag', f_box(0.22, 0.18, 0.20), bag_body, smooth=180)
part('TrashCans__knot', f_cyl(0.04, 0.03, 0.06), bag_knot, smooth=180)
part('TrashCans__lump', f_box(0.14, 0.10, 0.12), bag_lump, smooth=180)


# ---------------------------------------------------------- motion sensor --
def sensor_base():
    # ceiling mount: flat top, rounded skirt stepping in to the lens
    return lathe([(0, 0.01), (0.08, 0.01), (0.08, 0.005), (0.0785, 0.0015), (0.075, -0.0045), (0.07, -0.0085),
                  (0.064, -0.0098), (0.062, -0.01), (0, -0.01)], seg=32)

def sensor_lens():
    # faceted Fresnel dome on a short collar (the placer flips it to face down)
    prof = [(0, 0.0), (0.06, 0.0), (0.06, 0.004)]
    for k in range(1, 6):
        a = math.pi / 2 * k / 6
        prof.append((0.058 * math.cos(a), 0.004 + 0.056 * math.sin(a)))
    prof += [(0.012, 0.0595), (0, 0.06)]
    return lathe(prof, seg=16)

part('MotionSensors__base', f_cyl(0.08, 0.08, 0.02), sensor_base)
part('MotionSensors__lens', f_bounds((-0.06, 0, -0.06), (0.06, 0.06, 0.06), 'sphere'), sensor_lens, flat=True)


# ------------------------------------------------------------- coffee cup --
def paper_cup():
    # takeaway cup: recessed base in a skirt, tapered wall, rolled rim, inside
    return lathe([(0, -0.042), (0.024, -0.042), (0.026, -0.045), (0.028, -0.045), (0.0284, -0.041),
                  (0.0365, 0.037), (0.0373, 0.040), (0.038, 0.0425), (0.0374, 0.045), (0.0355, 0.0445),
                  (0.034, 0.041), (0.0265, -0.038), (0, -0.038)], seg=22)

part('CoffeeCups__cup', f_cyl(0.038, 0.028, 0.09), paper_cup)


# ------------------------------------------------------ missing photo hook --
def photo_hook():
    # brass picture hook: a tapered plate on the wall, a hook curled out at
    # the bottom, the pin driven diagonally into the wall through the top
    plate = prism([(-0.0075, 0.01), (0.0075, 0.01), (0.005, -0.006), (-0.005, -0.006)], 0.001).move(z=0.0005)
    hook = tube([(0, -0.0055, 0.001), (0, -0.0094, 0.0025), (0, -0.0085, 0.0045), (0, -0.0048, 0.0051)], 0.0009, seg=5)
    head = cyl(0.0022, 0.0022, 0.0008, seg=8, bevel_r=0).rot(x=math.pi / 2).move(y=0.0058, z=0.0014)
    pin = tube([(0, 0.0058, 0.001), (0, 0.001, -0.006)], 0.0006, seg=4)
    return plate + hook + head + pin

part('MissingPhotoHooks__hook', f_box(0.015, 0.02, 0.012), photo_hook)


# -------------------------------------------------------------- fire alarm --
def alarm_body():
    # pull station: back box, a parting line, rounded cover, a pocket for the pull bar
    s = stack(rr(0.12, 0.16, 0.01, 3),
              [(0.002, -0.02), (0, -0.018), (0, 0.002), (0.0012, 0.003), (0.0012, 0.0045), (0, 0.0055),
               (0, 0.016), (0.003, 0.0195), (0.007, 0.02)])
    return cut(s, rbox(0.10, 0.03, 0.012, r=0.003, seg=1).move(y=-0.02, z=0.02))

def alarm_handle():
    bar = rbox(0.09, 0.015, 0.009, r=0.004, seg=2).move(z=0.003)
    stems = sum((rbox(0.012, 0.011, 0.008, r=0.002, seg=1).move(x=sx * 0.034, z=-0.0035) for sx in (-1, 1)), Shape())
    return bar + stems

part('FireAlarms__body', f_box(0.12, 0.16, 0.04), alarm_body)
part('FireAlarms__handle', f_box(0.09, 0.015, 0.015), alarm_handle)


# ------------------------------------------------------- emergency light --
def elight_housing():
    # twin-head emergency unit: rounded box, parting groove, a louvred speaker
    # grille between the heads and a test button
    s = stack(rr(0.30, 0.10, 0.014, 2),
              [(0.002, -0.05), (0, -0.048), (0, -0.001), (0.0012, 0.0), (0.0012, 0.0015), (0, 0.0025),
               (0, 0.038), (0.004, 0.047), (0.009, 0.05)])
    slots = [rbox(0.05, 0.0038, 0.012, r=0.0012, seg=1).move(y=y, z=0.05) for y in (-0.012, 0.0, 0.012, 0.024)]
    s = cut(s, *slots)
    button = cyl(0.005, 0.005, 0.006, seg=12, bevel_r=0.0012).rot(x=math.pi / 2).move(y=-0.03, z=0.05)
    return s + button

def elight_head():
    # lamp head: swivel collar at the base, a slightly tapered shell, domed lens
    return lathe([(0, -0.025), (0.045, -0.025), (0.045, -0.019), (0.0435, -0.017), (0.0435, -0.012), (0.045, -0.01),
                  (0.043, 0.008), (0.042, 0.012), (0.04, 0.015), (0.035, 0.019), (0.026, 0.023), (0.013, 0.0248),
                  (0, 0.025)], seg=20)

def elight_bezel():
    return ring_lathe([(0.037, -0.004), (0.046, -0.004), (0.048, -0.002), (0.048, 0.002), (0.046, 0.004),
                       (0.039, 0.004), (0.036, 0.001)], seg=18)

part('EmergencyLights__housing', f_box(0.30, 0.10, 0.10), elight_housing)
part('EmergencyLights__head', f_cyl(0.04, 0.045, 0.05), elight_head)
part('EmergencyLights__bezel', f_cyl(0.048, 0.048, 0.008), elight_bezel)


# -------------------------------------------------------- wet floor sign --
def wet_sign_panel():
    # blow-moulded A-frame leaf: tapered, a raised border round a recessed
    # field, an embossed warning triangle, a carry slot at the top
    pts = round_outline([(-0.14, -0.21), (0.14, -0.21), (0.11, 0.21), (-0.11, 0.21)], 0.02, seg=3)
    s = stack(outl(pts), [(0.0, -0.007), (0, -0.001), (0.001, 0.0), (0.012, 0.0), (0.014, -0.0015)])
    tri = round_outline([(-0.075, -0.02), (0.075, -0.02), (0.0, 0.10)], 0.012, seg=2)
    emb = stack(outl(tri), [(0, -0.003), (0, 0.0), (0.009, 0.0), (0.009, -0.003)], loop=True)
    slot = prism(rounded_rect(0.08, 0.022, 0.011, seg=3), 0.03).move(y=0.17)
    return cut(s, slot) + emb

part('WetFloorSigns__panel', f_bounds((-0.14, -0.21, 0), (0.14, 0.21, 0), 'plane'), wet_sign_panel)


# -------------------------------------------------------------- handrail --
def rail_tube():
    return lathe([(0, -1.55), (0.012, -1.549), (0.017, -1.546), (0.0195, -1.541), (0.02, -1.535),
                  (0.02, 1.535), (0.0195, 1.541), (0.017, 1.546), (0.012, 1.549), (0, 1.55)], seg=20)

def rail_bracket():
    # clamp block (split in two halves) with a hex bolt on top
    block = rbox(0.03, 0.054, 0.06, r=0.007, seg=2).move(y=-0.003)
    ring = stack(rr(0.034, 0.064, 0.004, 1), [(0, -0.00075), (0, 0.00075), (0.003, 0.00075), (0.003, -0.00075)], loop=True)
    block = cut(block, ring.rot(x=math.pi / 2))
    bolt = cyl(0.0065, 0.0065, 0.0045, seg=6, bevel_r=0.0008).move(y=0.0262)
    dome = lathe([(0, 0.0284), (0.0035, 0.0284), (0.0028, 0.0295), (0, 0.03)], seg=8)
    return block + bolt + dome

def rail_worn():
    return lathe([(0, -0.15), (0.018, -0.15), (0.021, -0.148), (0.022, -0.144), (0.022, 0.144), (0.021, 0.148),
                  (0.018, 0.15), (0, 0.15)], seg=20)

part('Handrails__rail', f_cyl(0.02, 0.02, 3.1), rail_tube)
part('Handrails__bracket', f_box(0.03, 0.06, 0.06), rail_bracket)
part('Handrails__worn', f_cyl(0.022, 0.022, 0.30), rail_worn)


# -------------------------------------------------------- ceiling speaker --
def speaker_trim():
    return lathe([(0, 0.01), (0.11, 0.01), (0.11, 0.006), (0.1085, 0.002), (0.104, -0.002), (0.097, -0.005),
                  (0.093, -0.0065), (0.091, -0.009), (0.09, -0.01), (0, -0.01)], seg=28)

def speaker_grille():
    # pressed grille: concentric V grooves on the face, a rolled edge
    prof = [(0, -0.0025)]
    for rc in (0.024, 0.045, 0.066):
        prof += [(rc - 0.002, -0.0025), (rc, -0.0012), (rc + 0.002, -0.0025)]
    prof += [(0.086, -0.0025), (0.089, -0.0018), (0.09, 0.0), (0.089, 0.0022), (0.087, 0.0025), (0, 0.0025)]
    return lathe(prof, seg=18)

part('CeilingSpeakers__trim', f_cyl(0.11, 0.11, 0.02), speaker_trim)
part('CeilingSpeakers__grille', f_cyl(0.09, 0.09, 0.005), speaker_grille)


# -------------------------------------------------------- vending machine --
def _groove(w, h, r, x=0.0, y=0.0, z=0.275, width=0.004, depth=0.004):
    """A ring cutter: cuts a panel line of `width` round a w × h rounded rect."""
    return stack(rr(w, h, r, 2), [(0, -depth), (0, depth), (width, depth), (width, -depth)], loop=True).move(x=x, y=y, z=z)

def vend_cabinet():
    # carcass, a full-width door proud of it (panel lines round the display,
    # the control and coin panels and the delivery flap, a lock), side vents,
    # and a kick recess above the plinth
    carcass = rbox(0.89, 1.79, 0.48, r=0.01, seg=2).move(y=0.03, z=-0.035)
    vents = [rbox(0.03, 0.009, 0.14, r=0.002, seg=1).move(x=sx * 0.445, y=0.55 + k * 0.03, z=-0.11)
             for sx in (-1, 1) for k in range(6)]
    carcass = cut(carcass, *vents)
    kick = rbox(0.86, 0.07, 0.45, r=0.004, seg=1).move(y=-0.89, z=-0.05)
    door = rbox(0.90, 1.77, 0.072, r=0.014, seg=3).move(y=0.03, z=0.239)
    door = cut(door,
               _groove(0.83, 1.05, 0.02, y=0.275),
               _groove(0.29, 0.30, 0.012, x=-0.14, y=-0.185),
               _groove(0.15, 0.36, 0.012, x=0.33, y=-0.13),
               _groove(0.42, 0.19, 0.015, y=-0.645),
               cyl(0.012, 0.012, 0.012, seg=14, bevel_r=0).rot(x=math.pi / 2).move(x=0.33, y=-0.25, z=0.275))
    return carcass + kick + door

def vend_button():
    # square selection button, dished face, eased edge
    return stack(rr(0.06, 0.06, 0.008, 2), [(0, -0.004), (0, 0.0015), (0.0025, 0.0038), (0.006, 0.004), (0.012, 0.0034)])

def vend_plinth():
    # base with a louvred intake at the front, standing on four levelling feet
    body = rbox(0.90, 0.048, 0.55, r=0.004, seg=1).move(y=0.006)
    slots = [rbox(0.14, 0.006, 0.014, r=0.0015, seg=1).move(x=x, y=y, z=0.275)
             for x in (-0.3, -0.1, 0.1, 0.3) for y in (0.016, 0.0)]
    feet = sum((cyl(0.018, 0.02, 0.013, seg=12, bevel_r=0.002).move(x=sx * 0.40, y=-0.0235, z=sz * 0.23)
                for sx in (-1, 1) for sz in (-1, 1)), Shape())
    return cut(body, *slots) + feet

def coin():
    return lathe([(0, -0.0007), (0.0098, -0.001), (0.011, -0.0004), (0.011, 0.0004), (0.0098, 0.001), (0, 0.0007)], seg=12)

part('DeadVendingMachines__cabinet', f_box(0.90, 1.85, 0.55), vend_cabinet)
part('DeadVendingMachines__button', f_box(0.06, 0.06, 0.008), vend_button)
part('DeadVendingMachines__plinth', f_box(0.90, 0.06, 0.55), vend_plinth)
part('DeadVendingMachines__coin', f_cyl(0.011, 0.011, 0.002), coin)


# --------------------------------------------------------- keycard reader --
def reader_body():
    # reader housing with a swipe band (the slot plane sits on it) and a keypad
    s = stack(rr(0.10, 0.14, 0.012, 3), [(0.002, -0.015), (0, -0.013), (0, 0.004), (0.003, 0.009), (0.006, 0.0115), (0.01, 0.012)])
    band = rbox(0.092, 0.03, 0.008, r=0.003, seg=2).move(y=0.03, z=0.011)
    keys = sum((box(0.014, 0.0095, 0.004).move(x=x, y=y, z=0.0125) for x in (-0.02, 0.0, 0.02) for y in (0.002, -0.011, -0.024)), Shape())
    return s + band + keys

part('KeycardReaders__body', f_box(0.10, 0.14, 0.03), reader_body)


# --------------------------------------------------- hanging cable + plug --
def cable():
    # a limp flex with memory kinks; both ends on the axis (ceiling, plug)
    pts = [(0, 0.425, 0), (0.002, 0.33, 0.004), (0.009, 0.22, -0.003), (0.004, 0.1, -0.01), (-0.006, -0.02, -0.006),
           (-0.01, -0.15, 0.004), (-0.002, -0.28, 0.008), (0.003, -0.37, 0.002), (0, -0.425, 0)]
    return tube(pts, 0.0055, seg=6, smooth=2)

def plug():
    # moulded plug: body, strain-relief boot up to the cable, two blades down
    body = rbox(0.035, 0.022, 0.028, r=0.004, seg=2).move(y=-0.0005)
    boot = lathe([(0, 0.0105), (0.009, 0.0105), (0.0075, 0.016), (0.006, 0.0225), (0, 0.0225)], seg=10)
    blades = sum((box(0.0016, 0.012, 0.0065).move(x=sx * 0.0063, y=-0.0165) for sx in (-1, 1)), Shape())
    return body + boot + blades

part('CeilingCables__cable', f_cyl(0.006, 0.006, 0.85), cable, smooth=180)
part('CeilingCables__plug', f_box(0.035, 0.045, 0.028), plug)


# ----------------------------------------------------- dead ceiling light --
def light_canopy():
    # ceiling rose: flat face, rounded edge back up towards the ceiling, 2 screws
    s = lathe([(0, -0.0054), (0.11, -0.0054), (0.109, -0.0036), (0.104, -0.0018), (0.095, -0.0006), (0.08, 0.0), (0, 0.0)], seg=32)
    s = face_z(s)
    screws = sum((face_z(screw_head(0.0035, 0.0014)).move(x=sx * 0.07) for sx in (-1, 1)), Shape())
    return s + screws

def light_socket():
    # ribbed bakelite lamp holder with a hollow skirt
    s = lathe([(0, 0.015), (0.028, 0.015), (0.028, 0.0105), (0.025, 0.0085), (0.025, -0.012), (0.023, -0.015),
               (0.018, -0.015), (0.018, -0.006), (0, -0.006)], seg=16)
    def ribs(p):
        a = math.atan2(p.x, p.z)
        r = math.hypot(p.x, p.z)
        if -0.0121 < p.y < 0.0086 and abs(r - 0.025) < 1e-4 and round(a / (TAU / 16)) % 2:
            k = 0.0235 / r
            return Vector((p.x * k, p.y, p.z * k))
        return p
    return s.warp(ribs)

def light_bulb():
    # the screw base of a smashed bulb with a jagged crown of glass
    s = lathe([(0, 0.0125), (0.0135, 0.0125), (0.015, 0.009), (0.0135, 0.006), (0.015, 0.003), (0.0125, 0.0),
               (0.017, -0.007), (0.020, -0.0125), (0.0185, -0.0125), (0.0155, -0.007), (0, -0.003)], seg=12)
    jag = [0.0, 0.006, 0.002, 0.0075, 0.001, 0.004, 0.0, 0.0065, 0.003, 0.008, 0.0015, 0.005]
    def crown(p):
        if p.y < -0.012:
            i = round(math.atan2(p.x, p.z) / (TAU / 12)) % 12
            p.y += jag[i]
        return p
    return s.warp(crown)

part('DeadCeilingLights__canopy', f_bounds((-0.11, -0.11, 0), (0.11, 0.11, 0), 'plane'), light_canopy)
part('DeadCeilingLights__socket', f_cyl(0.028, 0.028, 0.03), light_socket)
part('DeadCeilingLights__bulb', f_cyl(0.015, 0.020, 0.025), light_bulb)


# ----------------------------------------------------------- ceiling chunk --
def ceiling_chunk():
    # a broken lump of plaster: one flat painted face, the rest fractured facets
    rnd = random.Random(17)
    pts = []
    for k in range(5):
        a = TAU * k / 5 + rnd.uniform(-0.3, 0.3)
        pts.append((0.024 * math.cos(a) * rnd.uniform(0.75, 1.0), 0.015, 0.024 * math.sin(a) * rnd.uniform(0.75, 1.0)))
    for k in range(22):
        a, b = rnd.uniform(0, TAU), rnd.uniform(-1.0, 0.45)
        r = rnd.uniform(0.8, 1.0)
        pts.append((0.025 * r * math.cos(b) * math.cos(a), 0.015 * math.sin(b) * 1.0 - 0.002, 0.025 * r * math.cos(b) * math.sin(a)))
    return hull(pts).fit((-0.025, -0.015, -0.025), (0.025, 0.015, 0.025))

part('CeilingCracks__chunk', f_box(0.05, 0.03, 0.05), ceiling_chunk, flat=True)


# -------------------------------------------------------- wall thermostat --
def wthermo_body():
    s = stack(rr(0.08, 0.12, 0.01, 3), [(0.0015, -0.01), (0, -0.0085), (0, -0.001), (0.001, 0.0), (0, 0.001),
                                         (0, 0.004), (0.003, 0.0085), (0.007, 0.01)])
    slots = [rbox(0.0032, 0.01, 0.008, r=0.001, seg=1).move(x=-0.0225 + 0.009 * k, y=0.045, z=0.01) for k in range(6)]
    return cut(s, *slots)

def wthermo_knob():
    # knurled setpoint dial with a pointer groove across the face
    s = lathe([(0, -0.003), (0.018, -0.003), (0.018, 0.001), (0.0168, 0.0026), (0.014, 0.003), (0, 0.003)], seg=24)
    def knurl(p):
        r = math.hypot(p.x, p.z)
        if r > 0.0179 and round(math.atan2(p.x, p.z) / (TAU / 24)) % 2:
            return Vector((p.x * 0.017 / r, p.y, p.z * 0.017 / r))
        return p
    return cut(s.warp(knurl), box(0.0025, 0.004, 0.013).move(y=0.0036, z=0.0085))

part('WallThermostats__body', f_box(0.08, 0.12, 0.02), wthermo_body)
part('WallThermostats__knob', f_cyl(0.018, 0.018, 0.006), wthermo_knob)


# ------------------------------------------------------------ chalk stick --
def chalk_stick():
    # one end square with a soft arris, the used end worn to a slanted nub
    s = lathe([(0, -0.0275), (0.0054, -0.0275), (0.006, -0.0269), (0.006, 0.019), (0.0057, 0.0225), (0.0047, 0.0258),
               (0.0028, 0.0273), (0, 0.0275)], seg=12)
    def wear(p):
        t = min(1.0, max(0.0, (p.y - 0.017) / 0.0105))
        p.y -= 0.004 * (p.x / 0.006 + 1) * t
        return p
    return noisy(s.warp(wear), 0.00015, scale=900, seed=2)

part('SmallChalkSticks__stick', f_cyl(0.006, 0.006, 0.055), chalk_stick, smooth=60)


# ---------------------------------------------------------- mirror frame --
def mirror_bar():
    # one constructor makes both the horizontal and the vertical bars, so the
    # part is square in XY and only its front edges are moulded (a stepped
    # bead and a chamfer); stretched along a bar they stay crisp
    return stack(rr(0.024, 0.024, 0.0004, 1), [(0, -0.01), (0, 0.006), (0.0008, 0.0068), (0.0008, 0.0082), (0.0018, 0.01)])

part('CrackedMirrors__bar', f_box(0.024, 0.024, 0.02), mirror_bar)


# --------------------------------------------------------- overturned chair --
def chair_seat():
    s = slab(0.34, 0.30, 0.024, 8, 8).rot(x=-math.pi / 2)
    def dish(p):
        if p.y > 0:
            p.y -= 0.004 * max(0.0, 1 - (p.x / 0.17) ** 2 - (p.z / 0.15) ** 2)
        return p
    return bevel(s.warp(dish), 0.006, seg=2, angle=40)

def chair_leg():
    return lathe([(0, -0.2), (0.0085, -0.2), (0.0092, -0.197), (0.0095, -0.15), (0.0105, -0.10), (0.012, -0.06),
                  (0.0105, -0.052), (0.012, -0.044), (0.012, 0.14), (0.0108, 0.15), (0.0115, 0.16), (0.0115, 0.2), (0, 0.2)], seg=12)

def chair_spindle():
    return lathe([(0, -0.14), (0.0055, -0.14), (0.0055, -0.12), (0.0065, -0.1), (0.008, -0.02), (0.008, 0.02),
                  (0.0065, 0.1), (0.0055, 0.12), (0.0055, 0.14), (0, 0.14)], seg=10)

def chair_rail():
    # crest rail: rounded section, bowed back towards its ends, rounded ends
    sec = rounded_rect(0.02, 0.024, 0.007, seg=3)
    xs = [(-0.15, 0.6), (-0.148, 0.88), (-0.144, 1.0), (-0.1, 1.0), (-0.05, 1.0), (0.0, 1.0), (0.05, 1.0), (0.1, 1.0),
          (0.144, 1.0), (0.148, 0.88), (0.15, 0.6)]
    rings = [[(x, v * k, u * k + 0.0025 - 0.005 * (x / 0.15) ** 2) for u, v in sec] for x, k in xs]
    return loft(rings)

part('OverturnedChairs__seat', f_box(0.34, 0.024, 0.30), chair_seat)
part('OverturnedChairs__leg', f_cyl(0.012, 0.012, 0.40), chair_leg)
part('OverturnedChairs__spindle', f_cyl(0.008, 0.008, 0.28), chair_spindle)
part('OverturnedChairs__rail', f_box(0.30, 0.024, 0.020), chair_rail)


# ----------------------------------------------------- overturned wastebasket --
def basket_bin():
    # moulded bin shell (open, as the placer builds it): foot bead, tapered
    # wall with thickness, rolled rim at the wide end
    return ring_lathe([(0.089, -0.125), (0.0915, -0.122), (0.0915, -0.116), (0.0905, -0.113), (0.1085, 0.112),
                       (0.11, 0.116), (0.11, 0.123), (0.1085, 0.125), (0.106, 0.1235), (0.1055, 0.118),
                       (0.0865, -0.12)], seg=24)

def basket_base():
    s = lathe([(0, -0.004), (0.09, -0.004), (0.09, -0.0015), (0.0875, 0.0), (0.078, 0.0), (0.0755, -0.0012), (0, -0.0012)], seg=24)
    return face_z(s)

part('OverturnedWastebaskets__bin', f_cyl(0.11, 0.09, 0.25), basket_bin)
part('OverturnedWastebaskets__base', f_bounds((-0.09, -0.09, 0), (0.09, 0.09, 0), 'plane'), basket_base)
part('OverturnedWastebaskets__paper', f_sphere(0.019), lambda: crumple(0.038, 0.038, 0.038, 31, jag=0.16), flat=True)


# ------------------------------------------------------- elevator buttons --
def elev_plate():
    # brushed faceplate: chamfered edge, raised collars round the buttons, two countersunk screws
    s = stack(rr(0.06, 0.12, 0.004, 2), [(0, -0.006), (0, 0.0045), (0.0015, 0.006)])
    collar = face_z(ring_lathe([(0.0165, -0.0005), (0.02, -0.0005), (0.02, 0.0003), (0.0185, 0.001), (0.0165, 0.0008)], seg=16))
    collars = collar.copy().move(y=0.025, z=0.006) + collar.copy().move(y=-0.025, z=0.006)
    screws = sum((face_z(small_screw(0.0028, 0.0009)).move(y=sy * 0.052, z=0.0056) for sy in (-1, 1)), Shape())
    return s + collars + screws

def elev_button():
    return lathe([(0, -0.0025), (0.016, -0.0025), (0.016, -0.001), (0.0148, 0.0015), (0.0135, 0.0025), (0.011, 0.0022),
                  (0, 0.0018)], seg=20)

part('ElevatorButtons__plate', f_box(0.06, 0.12, 0.012), elev_plate)
part('ElevatorButtons__button', f_cyl(0.016, 0.016, 0.005), elev_button)


# ---------------------------------------------------- broken light switch --
def switch_plate():
    # pillowed switch plate, the toggle opening, two screws, a corner broken
    # out where the wires come through
    s = stack(rr(0.06, 0.10, 0.004, 1), [(0, -0.003), (0, 0.0005), (0.002, 0.0025), (0.005, 0.003)])
    hole = rbox(0.011, 0.025, 0.004, r=0.0015, seg=1).move(y=0.004, z=0.003)
    chip = box(0.012, 0.016, 0.02).rot(z=0.5).move(x=-0.031, y=-0.016)
    screws = sum((face_z(small_screw(0.0028, 0.001)).move(y=sy * 0.038, z=0.0026) for sy in (-1, 1)), Shape())
    return cut(s, hole, chip) + screws

def _side_prism(pts_zy, width, bevel_r):
    """Extrude a side profile [(z, y), ...] across X (width), centred."""
    return prism([(-z, y) for z, y in pts_zy], width, bevel_r=bevel_r).rot(y=math.pi / 2)

def switch_toggle():
    # the toggle bat: low at its pivot, rising to a rounded tip, narrowing
    s = _side_prism([(-0.005, -0.014), (-0.002, -0.014), (0.0, -0.004), (0.005, 0.012), (0.004, 0.014), (0.0, 0.012),
                     (-0.005, 0.0)], 0.014, 0.0012)
    s = taper(s, top=0.75, along='y', axes='x')
    return s.fit((-0.007, -0.014, -0.005), (0.007, 0.014, 0.005))

def switch_wire():
    # a stripped wire end with a slight kink
    ins = tube([(0, -0.025, 0), (0.0006, -0.013, 0.0003), (0.0008, 0.0, -0.0004), (0.0002, 0.012, 0.0005), (0, 0.017, 0.0)], 0.001, seg=6, smooth=1)
    core = tube([(0, 0.0165, 0.0), (-0.0003, 0.021, 0.0001), (-0.0004, 0.025, 0.0003)], 0.00055, seg=5)
    return ins + core

part('BrokenLightSwitches__plate', f_box(0.060, 0.10, 0.006), switch_plate)
part('BrokenLightSwitches__toggle', f_box(0.014, 0.028, 0.010), switch_toggle)
part('BrokenLightSwitches__wire', f_cyl(0.001, 0.001, 0.05), switch_wire, smooth=70)


# ------------------------------------------------------- crushed soda can --
def soda_can():
    # an axially crushed can: domed base, accordion folds, lid rim and tab
    s = lathe([(0, -0.0125), (0.0165, -0.015), (0.018, -0.0135), (0.0195, -0.009), (0.0165, -0.004), (0.0205, 0.002),
               (0.017, 0.0075), (0.0185, 0.0135), (0.02, 0.015), (0.0165, 0.0145), (0, 0.0142)], seg=13)
    s = noisy(s, 0.0012, scale=110, seed=6)
    tab = box(0.006, 0.0008, 0.0045).move(y=0.0148, z=0.006)
    return s + tab

part('CrushedSodaCans__can', f_cyl(0.020, 0.018, 0.030), soda_can, smooth=40)


# ------------------------------------------------------- fire alarm pull --
def pull_body():
    # unlit red plate: only its outline shows, so it stays cheap (rounded corners, chamfer)
    return stack(rr(0.10, 0.14, 0.008, 2), [(0, -0.01), (0, 0.007), (0.003, 0.01)])

def pull_handle():
    # pull-down lever: sloped front, a finger lip at the bottom
    return _side_prism([(-0.005, -0.01), (0.005, -0.01), (0.005, -0.005), (0.0015, -0.0035), (0.0, 0.01), (-0.005, 0.01)],
                       0.06, 0.0015)

part('FireAlarmPulls__body', f_box(0.10, 0.14, 0.020), pull_body)
part('FireAlarmPulls__handle', f_box(0.06, 0.020, 0.010), pull_handle)


# -------------------------------------------------- crushed water bottle --
def bottle_cap():
    # ribbed screw cap; its closed top faces -Y (outwards on the bottle)
    s = lathe([(0, -0.004), (0.0088, -0.004), (0.010, -0.0028), (0.010, 0.004), (0, 0.004)], seg=12)
    def ribs(p):
        r = math.hypot(p.x, p.z)
        if r > 0.0099 and round(math.atan2(p.x, p.z) / (TAU / 12)) % 2:
            return Vector((p.x * 0.0093 / r, p.y, p.z * 0.0093 / r))
        return p
    return s.warp(ribs)

def bottle_label():
    # thin wrap-around label sleeve, slightly crinkled with the crushed bottle
    s = ring_lathe([(0.021, -0.015), (0.021, -0.005), (0.021, 0.005), (0.021, 0.015), (0.0205, 0.015), (0.0205, -0.005),
                    (0.0205, 0.005), (0.0205, -0.015)], seg=16)
    def crinkle(p):
        r = math.hypot(p.x, p.z)
        k = 1 + 0.025 * noise.noise(Vector((p.x, p.y, p.z)) * 160)
        return Vector((p.x * k, p.y, p.z * k))
    return s.warp(crinkle)

part('CrushedWaterBottles__cap', f_cyl(0.010, 0.010, 0.008), bottle_cap)
part('CrushedWaterBottles__label', f_cyl(0.021, 0.021, 0.030), bottle_label)
