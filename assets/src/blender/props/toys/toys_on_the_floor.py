"""r17: toys left on the floor, remodelled at their real sizes (docs/design/PROPS.md),
P2: the rubber ball, the toy car, the pinwheel, the jigsaw piece and the
magnetic letters. Sources and sizes: docs/research/props-remodelling/toys_on_the_floor.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its own
frame: x across its front, y up from the floor (y = 0 the floor it rests on),
z out of its front.

rubberBallHalfA, rubberBallHalfB
        a hollow rubber ball (4 in) of two moulded halves, each its own
        colour, joined at a seam tilted from level; soft with age: its base
        sagged flat where it rests, a shallow dent in its shoulder
toyCarBody      a 1:64 die-cast saloon's painted zinc body (z out of its nose):
                the lower body lofted with its wheel arches, the roof and the
                pillars round the glass
toyCarGlass     its window glazing, the glasshouse under the roof
toyCarBright    the silver-painted bumpers, grille and headlamps, the steel
                hubs and axles
toyCarTail      the tail lamps' red paint
toyCarBase      the black plastic baseplate under it, its two rivets
toyCarTyres     the four plastic tyres
pinwheelVane    one of the wheel's four vanes, lying face up (the game turns
                it four ways about the hub, each its own colour): a quarter of
                the square sheet, its cut corner curled up over to the pin, its
                flat corner drooping to the floor
pinwheelPin     the pearl-headed pin through the wheel and the bead under it
pinwheelStick   the birch dowel lying on the floor, its end under the hub
puzzlePieceA/B, puzzlePieceFaceA/B
        a children's jigsaw piece (2 mm board), A a border piece (a tab at
        +x, blanks at +z and -x, its straight side at -z), B an inner piece
        (tabs at +x and +z, blanks at -z and -x); the board's edges and
        back, and its printed face
fridgeLetter<L>, fridgeLetterMagnet<L>
        a moulded plastic fridge letter (38 mm capitals) lying face up, its
        top toward -z; the round ferrite magnet set into a pocket in its back
"""
from propkit import *
from propkit import _mesh_obj, _eval_to_shape, _drop
import propkit


def tri_count(s):
    return sum(len(f) - 2 for f in s.f)


def decimate(s, tris):
    """Collapse-decimate to about `tris` triangles."""
    n = tri_count(s)
    if n <= tris:
        return s
    ob = _mesh_obj('_dec', s)
    m = ob.modifiers.new('d', 'DECIMATE')
    m.decimate_type = 'COLLAPSE'
    m.ratio = tris / n
    m.use_collapse_triangulate = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def _fix_out(s, c):
    """Every face wound to face away from c."""
    out = []
    for q in s.f:
        p0, p1, p2 = s.v[q[0]], s.v[q[1]], s.v[q[2]]
        n = (p1 - p0).cross(p2 - p0)
        out.append(q if n.dot(p0 - c) >= 0 else tuple(reversed(q)))
    s.f = out
    return s


def hull(points, bevel_r=0.0, seg=1):
    """The convex hull of the points (a closed shape), its edges bevelled."""
    bm = bmesh.new()
    for p in points:
        bm.verts.new(Vector(p))
    bmesh.ops.convex_hull(bm, input=list(bm.verts))
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context='VERTS')
    bmesh.ops.dissolve_limit(bm, angle_limit=math.radians(1), verts=bm.verts, edges=bm.edges)
    s = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    c = sum(s.v, Vector((0, 0, 0))) / len(s.v)
    s = _fix_out(s, c)
    return bevel(s, bevel_r, seg=seg) if bevel_r > 0 else s


# =========================================================== rubber ball ===
# A 4 in (102 mm) hollow rubber play ball: two half shells moulded and
# joined at the equator (the seam a low ridge where the flash was trimmed),
# each half its own colour. Years on a floor have let the air out of it: it
# sags flat where it rests (a soft ball's contact patch, 3.5 cm across) and
# keeps a shallow dent in its shoulder. The seam is tilted 34 degrees from
# level, as a ball comes to rest any way up.
BALL_R = 0.0508
BALL_SAG = 0.0030                          # how far its base has sagged flat
BALL_TILT = math.radians(34)               # the seam's plane from level
BALL_DENT = (Vector((0.55, 0.62, -0.56)).normalized(), 0.0022, 0.30)   # where, how deep, how wide (rad)


def ball_frame():
    """The seam plane's normal n (half A's pole) and two axes in the plane."""
    n = Vector((0, math.cos(BALL_TILT), math.sin(BALL_TILT)))
    e1 = Vector((1, 0, 0))
    e2 = n.cross(e1).normalized()
    return n, e1, e2


def ball_point(phi, th):
    """The ball's surface at polar angle phi from half A's pole (pi/2 the
    seam) and azimuth th, before it sagged: the seam's ridge, 0.45 mm
    proud and 1.6 mm wide, and its two shallow mould lines either side."""
    n, e1, e2 = ball_frame()
    d = n * math.cos(phi) + (e1 * math.cos(th) + e2 * math.sin(th)) * math.sin(phi)
    a = (phi - math.pi / 2) * BALL_R                  # metres from the seam along the surface
    r = BALL_R + 0.00045 * math.exp(-(a / 0.0008) ** 2) - 0.00012 * math.exp(-((abs(a) - 0.0019) / 0.0005) ** 2)
    # the dent: pushed in, soft-edged
    dd, depth, width = BALL_DENT
    ang = math.acos(max(-1.0, min(1.0, d.dot(dd))))
    r -= depth * math.exp(-(ang / width) ** 2)
    return d * r


def ball_sag(p):
    """A soft ball resting: what is under the contact plane is pressed onto
    it (a smooth max, its knee 1.8 mm soft) and bulges out a little round
    its foot."""
    plane = -BALL_R + BALL_SAG
    d = p.y - plane
    k = 0.0018
    y = plane + (d + math.sqrt(d * d + k * k)) / 2
    lift = y - p.y
    bulge = 1.0 + 0.55 * lift / BALL_R
    return Vector((p.x * bulge, y, p.z * bulge))


def ball_low():
    """The lowest point after the sag (the ball's bottom), which rests on
    the floor."""
    return ball_sag(Vector((0, -BALL_R, 0))).y


def ball_half(side):
    """One half shell (A: about the pole n, B: the other), the seam its
    edge; rings dense at the seam for its ridge."""
    ns = 34
    ring_a = [0.0, 0.22, 0.45, 0.68, 0.91, 1.13, 1.32, 1.45, 1.52, 1.552, math.pi / 2]
    phis = ring_a if side == 'A' else [math.pi - p for p in ring_a]
    v, f = [], []
    rows = []
    for j, phi in enumerate(phis):
        if j == 0:
            v.append(ball_point(phi, 0.0))
            rows.append([len(v) - 1])
            continue
        row = []
        for i in range(ns):
            v.append(ball_point(phi, TAU * i / ns))
            row.append(len(v) - 1)
        rows.append(row)
    for a, b in zip(rows, rows[1:]):
        if len(a) == 1:
            for i in range(ns):
                f.append((a[0], b[i], b[(i + 1) % ns]))
        else:
            for i in range(ns):
                j = (i + 1) % ns
                f.append((a[i], b[i], b[j], a[j]))
    s = Shape([ball_sag(p) for p in v], f)
    s.move(0, -ball_low(), 0)
    return _fix_out(s, Vector((0, -ball_low(), 0)))


# ============================================================= toy car ===
# A 1:64 die-cast saloon of the 1970s (the Matchbox 1-75 size: about 3 x
# 1.25 x 1 in): a painted zinc (Mazak) body, clear plastic glazing, a black
# plastic baseplate held by two spun rivets, thin plastic wheels on steel
# axles with silver hubs, the bumpers, grille and headlamps cast in and
# painted silver, the tail lamps red. Its frame: x across, y up, z out of
# its nose. 76 x 30 x 23 mm.
CAR_L, CAR_W = 0.076, 0.030
WHEEL_R, WHEEL_W = 0.0056, 0.0042
WHEEL_Z = 0.0245
WHEEL_X = CAR_W / 2 - 0.0026
ARCH_R = 0.0064
SILL_Y = 0.0042
GLASS = dict(y0=0.0146, y1=0.0226, z0=(-0.0185, 0.0135), z1=(-0.0105, 0.0045), w0=0.0134, w1=0.0110)


def car_top(z):
    """The lower body's top line: the bonnet rising to the cowl, the waist,
    the boot falling to its tail."""
    if z > 0.013:
        t = (z - 0.013) / (CAR_L / 2 - 0.013)
        return 0.0152 - 0.0022 * t ** 1.6
    if z < -0.018:
        t = (-0.018 - z) / (CAR_L / 2 - 0.018)
        return 0.0152 - 0.0012 * t ** 1.4
    return 0.0152


def car_bottom(z):
    """The body's bottom edge: the sill, rising over each wheel's arch."""
    y = SILL_Y
    for zc in (-WHEEL_Z, WHEEL_Z):
        dz = abs(z - zc)
        if dz < ARCH_R:
            y = max(y, WHEEL_R + math.sqrt(ARCH_R * ARCH_R - dz * dz) * 0.94)
    # the nose and tail tuck under a little
    y += 0.0012 * smoothstep(CAR_L / 2 - 0.004, CAR_L / 2, abs(z))
    return y


def car_half_w(z):
    """The body's half width along it: its plan's corners rounded (5 mm at
    the nose, 6 mm at the tail)."""
    w = CAR_W / 2
    rc = 0.005 if z > 0 else 0.006
    e = CAR_L / 2 - rc
    if abs(z) > e:
        dz = abs(z) - e
        return w - rc + math.sqrt(max(0.0, rc * rc - dz * dz))
    return w


def car_section(z, n_top=3):
    """A ring round the body at z: a rounded rectangle, its shoulders round
    (2.6 mm), its foot nearly square (a 0.7 mm chamfer), the sides a little
    tumbled in toward the top."""
    w, y0, y1 = car_half_w(z), car_bottom(z), car_top(z)
    rt = min(0.0026, (y1 - y0) * 0.45, w * 0.5)
    rb = min(0.0007, (y1 - y0) * 0.2)
    right = [(0.0, y0), (w - rb, y0), (w, y0 + rb)]
    for k in range(n_top + 1):
        a = (math.pi / 2) * k / n_top
        right.append((w - 0.0004 - rt + rt * math.cos(a), y1 - rt + rt * math.sin(a)))
    right.append((0.0, y1))
    left = [(-x, y) for x, y in reversed(right[1:-1])]
    return [Vector((x, y, z)) for x, y in right + left]


def car_plan_z(x, front):
    """The nose's (or tail's) face in plan at x across: flat, its corners
    rounded back."""
    w = CAR_W / 2
    rc = 0.005 if front else 0.006
    e = CAR_L / 2 - rc
    dx = abs(x) - (w - rc)
    z = CAR_L / 2 if dx <= 0 else e + math.sqrt(max(0.0, rc * rc - dx * dx))
    return z if front else -z


def car_lower():
    zs = {-CAR_L / 2, CAR_L / 2}
    for k in range(-9, 10):
        zs.add(k * 0.004)
    for zc in (-WHEEL_Z, WHEEL_Z):
        for d in (-1.0, -0.72, -0.38, 0.0, 0.38, 0.72, 1.0):
            zs.add(zc + d * ARCH_R)
    for e in (0.0007, 0.0022, 0.0045):
        zs.add(CAR_L / 2 - e)
        zs.add(-CAR_L / 2 + e)
    zs = sorted(z for z in zs if abs(z) <= CAR_L / 2 + 1e-9)
    merged = [zs[0]]
    for z in zs[1:]:
        if z - merged[-1] > 0.0007:
            merged.append(z)
    merged[-1] = CAR_L / 2
    rings = [car_section(z, n_top=2) for z in merged]
    n = len(rings[0])
    v = [p for r in rings for p in r]
    f = []
    for k in range(len(rings) - 1):
        for i in range(n):
            j = (i + 1) % n
            f.append((k * n + i, k * n + j, (k + 1) * n + j, (k + 1) * n + i))
    f.append(tuple(range(n)))
    f.append(tuple((len(rings) - 1) * n + i for i in range(n)))
    s = Shape(v, f)
    s = propkit._fix_winding(s)
    return bevel(s, 0.0007, seg=1, angle=50)


def glass_points(grow=0.0, dy=0.0):
    g = GLASS
    pts = []
    for sx in (-1, 1):
        for z in g['z0']:
            pts.append((sx * (g['w0'] + grow), g['y0'], z + (grow if z > 0 else -grow)))
        for z in g['z1']:
            pts.append((sx * (g['w1'] + grow), g['y1'] + dy, z + (grow if z > 0 else -grow)))
    return pts


def car_glass():
    return hull(glass_points(grow=-0.0004), bevel_r=0.0012, seg=2)


def car_upper():
    """The roof and its pillars: a shell a shade larger than the glass, its
    windows cut out of it (the windscreen and the rear window through it
    along its length, the side windows across it), so the glass shows
    through them set in a little."""
    shell = hull(glass_points(grow=0.0004, dy=0.0006), bevel_r=0.0014, seg=2)
    g = GLASS
    ya, yb = g['y0'] + 0.0019, g['y1'] - 0.0011
    # through it along z: the windscreen and the rear window (inside the A
    # and C pillars), and across it along x: the side windows, in front of
    # and behind the B pillar
    wz = prism([(-g['w1'] + 0.0012, ya), (g['w1'] - 0.0012, ya), (g['w1'] - 0.0016, yb), (-g['w1'] + 0.0016, yb)], 0.2)
    # side windows in side view (z, y): the front door's and the rear's
    def side(z0b, z1b, z0t, z1t):
        # (a prism turned about y: its outline's x becomes -z)
        return prism([(-z0b, ya), (-z1b, ya), (-z1t, yb), (-z0t, yb)], 0.2).rot(0, math.pi / 2, 0)
    zA_b, zA_t = g['z0'][1] - 0.0016, g['z1'][1] - 0.0002       # behind the A pillar
    zC_b, zC_t = g['z0'][0] + 0.0044, g['z1'][0] + 0.0020       # in front of the wide C pillar
    zB = -0.0026
    front = side(zB + 0.0009, zA_b, zB + 0.0008, zA_t)
    rear = side(zC_b, zB - 0.0009, zC_t, zB - 0.0010)
    return cut(shell, wz, front, rear)


def car_body():
    return car_lower() + car_upper()


def car_bright():
    """The silver paint and steel: bumpers wrapping the corners, the grille,
    the headlamps; each wheel's hub and the axles."""
    out = Shape()
    zf, zr = CAR_L / 2, -CAR_L / 2
    for front, sgn in ((True, 1), (False, -1)):
        pts = []
        for k in range(11):
            x = -0.0146 + 0.0292 * k / 10
            pts.append((x, 0.0062, car_plan_z(x, front) + sgn * 0.0003))
        out = out + tube(pts, 0.0009, seg=5)
    # the grille and the headlamps on the nose's flat face
    out = out + rbox(0.0084, 0.0030, 0.0008, r=0.0002, seg=1).move(0, 0.0097, zf - 0.0001)
    for sx in (-1, 1):
        out = out + cyl(0.0016, h=0.0008, seg=10, bevel_r=0).rot(math.pi / 2, 0, 0).move(sx * 0.0074, 0.0100, zf - 0.0002)
        for zc in (-WHEEL_Z, WHEEL_Z):
            hub = lathe([(0.0, 0.0), (0.0033, 0.0), (0.0033, 0.0003), (0.0022, 0.0008), (0.0, 0.0009)], seg=10)
            hub.rot(0, 0, -sx * math.pi / 2).move(sx * (WHEEL_X + WHEEL_W / 2 - 0.0007), WHEEL_R, zc)
            out = out + hub
    for zc in (-WHEEL_Z, WHEEL_Z):
        out = out + cyl(0.0005, h=2 * WHEEL_X, seg=6, bevel_r=0).rot(0, 0, math.pi / 2).move(0, WHEEL_R, zc)
    return out


def car_tail():
    out = Shape()
    for sx in (-1, 1):
        out = out + rbox(0.0034, 0.0022, 0.0008, r=0.0002, seg=1).move(sx * 0.0068, 0.0116, -CAR_L / 2 + 0.0001)
    return out


def car_base():
    plate = rbox(0.0196, 0.0010, 0.066, r=0.0003, seg=1).move(0, 0.0041, 0)
    for zc in (-0.029, 0.029):
        plate = plate + lathe([(0.0, 0.0), (0.0016, 0.0), (0.0014, 0.0004), (0.0, 0.0006)], seg=10).rot(math.pi, 0, 0).move(0, 0.0036, zc)
    return plate


def car_tyres():
    """A thin plastic wheel: its tread flat, its shoulders rounded, a
    recess in its outer face where the hub sits, its inner face closed."""
    out = Shape()
    hw = WHEEL_W / 2
    prof = [(0.0, -hw + 0.0004), (0.0032, -hw + 0.0004), (WHEEL_R - 0.0007, -hw), (WHEEL_R, -hw + 0.0010),
            (WHEEL_R, hw - 0.0010), (WHEEL_R - 0.0007, hw), (0.0035, hw), (0.0035, hw - 0.0006), (0.0, hw - 0.0006)]
    w = lathe(prof, seg=14)
    for sx in (-1, 1):
        for zc in (-WHEEL_Z, WHEEL_Z):
            out = out + w.copy().rot(0, 0, -sx * math.pi / 2).move(sx * WHEEL_X, WHEEL_R, zc)
    return out


# ============================================================ pinwheel ===
# A small paper pinwheel (a party favour): a 10 cm square of printed paper
# cut from each corner three quarters of the way in, every other corner curled
# to the middle and pinned with a pearl-headed pin through a wooden bead
# into a 6 mm birch dowel 28 cm long. It lies on the floor face up, the
# dowel on the floor, the wheel over its end, the vanes' curls standing up
# and the flat corners drooping to the floor.
PW_S = 0.100                     # the sheet's side
PW_D = PW_S / math.sqrt(2)       # centre to corner
PW_SC = 0.27                     # the cuts stop this far out (a fraction of PW_D)
STICK_R = 0.003
STICK_L = 0.280
STICK_TIP = 0.012                # the dowel runs on past the pin
PW_HUB_Y = 2 * STICK_R + 0.0045  # the sheet's middle: over the dowel and the bead


PW_LEAN = math.radians(32)       # the curl's loop leans over toward the flat corner


def pw_droop(r):
    """The flat sheet's height at radius r: level over the dowel and the
    bead, its corners bending down toward the floor."""
    t = smoothstep(0.040, PW_D, r)
    return PW_HUB_Y - (PW_HUB_Y - 0.0012) * t ** 1.4


def pw_curl_arc():
    """C1's cut edge, from where the cut stops to its corner pinned at the
    middle, is a strip of paper (1 - PW_SC) * PW_D long: it rises off the
    sheet in a loop over to the pin. The loop as a circular arc of that
    length through its two ends, in its own plane's (a out along the cut,
    b up the plane) coordinates: the centre, radius, start angle and the
    angle it turns through."""
    rc = PW_SC * PW_D
    p0 = Vector((rc, 0.0))
    p2 = Vector((0.0030, 0.0042))
    L = (1 - PW_SC) * PW_D
    c = (p2 - p0).length
    lo, hi = 1e-3, TAU * 0.98                 # L / c = th / (2 sin(th / 2))
    for _ in range(60):
        th = (lo + hi) / 2
        if th / (2 * math.sin(th / 2)) < L / c:
            lo = th
        else:
            hi = th
    th = (lo + hi) / 2
    R = c / (2 * math.sin(th / 2))
    mid = (p0 + p2) / 2
    ch = (p2 - p0).normalized()
    up = Vector((-ch.y, ch.x))
    if up.y < 0:
        up = -up
    centre = mid + up * (-R * math.cos(th / 2))
    a0 = math.atan2(p0.y - centre.y, p0.x - centre.x)
    a_top = math.atan2(up.y, up.x)
    sgn = 1.0 if ((a_top - a0) % TAU) < math.pi else -1.0
    return centre, R, a0, sgn * th


def pinwheel_vane():
    """One vane, the quarter of the sheet between the cuts from corners C0
    (at 45 deg, its corner left flat) and C1 (at 135 deg, its corner curled
    over to the pin), in the -z quarter. A grid over (s, t): s out from the
    middle, t across from C0's cut to C1's. The strip along C1's cut is the
    loop (pw_curl_arc), its plane leaning over the vane toward C0; across
    the vane the paper sweeps from the flat sheet up into it."""
    ns, nt = 16, 11
    a0, a1 = math.radians(45), math.radians(135)
    d0 = Vector((math.cos(a0), 0, -math.sin(a0)))
    d1 = Vector((math.cos(a1), 0, -math.sin(a1)))
    e_tan = (d0 - d1 * d0.dot(d1)).normalized()
    plane_b = Vector((0, 1, 0)) * math.cos(PW_LEAN) + e_tan * math.sin(PW_LEAN)
    centre, R, b0, span = pw_curl_arc()
    hub = Vector((0, PW_HUB_Y, 0))

    def loop(u):
        a = b0 + span * u
        pa, pb = centre.x + R * math.cos(a), centre.y + R * math.sin(a)
        return hub + d1 * pa + plane_b * pb

    v, f = [], []
    for j in range(nt + 1):
        t = j / nt
        for i in range(ns + 1):
            s = i / ns
            flat = (d0 * (1 - t) + d1 * t) * s * PW_D
            p = Vector((flat.x, pw_droop(flat.length), flat.z))
            if s > PW_SC:
                u = (s - PW_SC) / (1 - PW_SC)
                w = smoothstep(0.25, 1.0, t) ** 0.9
                # the paper near the cut's end is held flat by the sheet
                w *= smoothstep(0.0, 0.10, u) if t < 0.999 else 1.0
                p = p.lerp(loop(u), w)
            v.append(p)
    for j in range(nt):
        for i in range(ns):
            a = j * (ns + 1) + i
            f.append((a, a + 1, a + ns + 2, a + ns + 1))
    s = Shape(v, f)
    a, b, c = (s.v[x] for x in s.f[0][:3])
    if (b - a).cross(c - a).y < 0:
        s.f = [tuple(reversed(q)) for q in s.f]
    return s


def pinwheel_pin():
    """The pearl-headed pin (its head over the corners pinned together), its
    shaft down through the sheet, and the wooden bead under the sheet."""
    head = sphere(0.0022, seg=10, rings=6).move(0, PW_HUB_Y + 0.0058, 0)
    shaft = cyl(0.0004, h=0.008, seg=6, bevel_r=0).move(0, PW_HUB_Y + 0.001, 0)
    bead = lathe([(0.0, 0.0), (0.0016, 0.0), (0.0030, 0.0008), (0.0032, 0.0020), (0.0030, 0.0032), (0.0016, 0.0040), (0.0, 0.0040)], seg=10)
    bead.move(0, 2 * STICK_R - 0.0002, 0)
    return head + shaft + bead


def pinwheel_stick():
    """The dowel lying on the floor along -z from just past the hub."""
    s = cyl(STICK_R, h=STICK_L, seg=10, bevel_r=0.0007, bevel_seg=1)
    s.rot(math.pi / 2, 0, 0).move(0, STICK_R, STICK_TIP - STICK_L / 2)
    # a dowel is not quite straight
    def bow(p):
        t = (p.z - STICK_TIP) / -STICK_L
        p.x += 0.0025 * math.sin(math.pi * t)
        return p
    return s.warp(bow)


# ======================================================== jigsaw piece ===
# A piece of a children's jigsaw (a 35-piece puzzle, about 30 x 21 cm: its
# pieces 4-5 cm): 2 mm greyboard faced with a printed paper, die cut with
# round knobs on necks. A is a border piece (a tab, two blanks, a straight
# side), B an inner one (two tabs, two blanks). Lying face up, x and z across
# it (its outline is drawn in (x, y) and laid down: y becomes -z).
PZ_B = 0.038                    # the body's side
PZ_T = 0.0020                   # the board
PZ_KNOB = dict(r=0.0058, c=0.0078, neck=0.0034)


def pz_edge(p0, p1, kind, shift=0.0, lean=0.0):
    """The points along one side from p0 to p1 (p0 not included): straight,
    a tab (out) or a blank (in); the knob a little off the middle and
    leaning, as dies are cut by hand."""
    p0, p1 = Vector(p0), Vector(p1)
    u = (p1 - p0).normalized()
    n = Vector((u.y, -u.x))                     # outward for a counter-clockwise outline
    if kind == 'flat':
        return [p1]
    w = n * (1.0 if kind == 'tab' else -1.0)    # toward the knob
    L = (p1 - p0).length
    m = p0 + u * (L / 2 + shift)
    r, cv, nh = PZ_KNOB['r'], PZ_KNOB['c'], PZ_KNOB['neck']
    root = cv - math.sqrt(r * r - nh * nh)      # where the neck meets the bulb
    pts = [m - u * nh * 1.45, m - u * nh + w * root * 0.35]
    a_l = math.atan2(-math.sqrt(r * r - nh * nh), -nh)
    a_r = math.atan2(-math.sqrt(r * r - nh * nh), nh)
    span = (a_l - a_r) % TAU                    # the long way: round the far side of the bulb
    k = 12
    for i in range(k + 1):
        a = a_l - span * i / k
        pts.append(m + u * (lean + r * math.cos(a)) + w * (cv + r * math.sin(a)))
    pts += [m + u * nh + w * root * 0.35, m + u * nh * 1.45, p1]
    return pts


def round_sharp(outline, r, seg=2, min_turn=40.0):
    """Round the outline's corners that turn by more than min_turn degrees."""
    out = []
    n = len(outline)
    for i in range(n):
        a, p, b = Vector(outline[i - 1]), Vector(outline[i]), Vector(outline[(i + 1) % n])
        d0, d1 = (p - a), (b - p)
        if d0.length < 1e-9 or d1.length < 1e-9:
            continue
        turn = math.degrees(d0.angle(d1))
        if turn > min_turn:
            out += propkit._round_corner(outline[i], outline[i - 1], outline[(i + 1) % n], r, seg)
        else:
            out.append(tuple(outline[i]))
    return out


def pz_outline(sides):
    """sides: the four sides' kinds going counter-clockwise round the
    outline's (x, y) plane from its -y side (laid down, -y is +z): +z, +x,
    -z, -x."""
    h = PZ_B / 2
    corners = [(-h, -h), (h, -h), (h, h), (-h, h)]
    shifts = [(0.0012, -0.0006), (-0.0009, 0.0007), (0.0006, 0.0005), (-0.0010, -0.0004)]
    pts = [Vector(corners[0])]
    for i in range(4):
        pts += pz_edge(corners[i], corners[(i + 1) % 4], sides[i], *shifts[i])
    pts = pts[:-1]
    out = [(p.x, p.y) for p in pts]
    # the die's corners and the necks' joins are a little round
    return round_sharp(out, 0.0006, seg=2)


PZ_SIDES = {'A': ('blank', 'tab', 'flat', 'blank'), 'B': ('tab', 'tab', 'blank', 'blank')}
_pz = {}


def pz_piece(which):
    """The board, split into its printed face (the top) and the rest."""
    if which in _pz:
        return _pz[which]
    outline = pz_outline(PZ_SIDES[which])
    s = prism(outline, PZ_T)                    # along z, centred
    # lay it down: the outline's y becomes -z (away), its thickness y
    s.rot(-math.pi / 2, 0, 0).move(0, PZ_T / 2, 0)
    top, rest = [], []
    for q in s.f:
        p0, p1, p2 = s.v[q[0]], s.v[q[1]], s.v[q[2]]
        nrm = (p1 - p0).cross(p2 - p0)
        (top if nrm.y > 0 and nrm.normalized().y > 0.9 else rest).append(q)

    def sub(faces):
        used = sorted({i for q in faces for i in q})
        remap = {i: k for k, i in enumerate(used)}
        return Shape([s.v[i].copy() for i in used], [tuple(remap[i] for i in q) for q in faces])
    _pz[which] = (sub(top), sub(rest))
    return _pz[which]


# ======================================================= fridge letters ===
# Moulded plastic fridge letters, the capitals of an alphabet set (38 mm
# tall, 8 mm thick: Lakeshore's are 1 3/8 in): a bold, rounded sans with its
# edges rounded over, a flat back with a round ferrite magnet (6 mm) set
# flush into a pocket. Each lies face up, its top toward -z.
LETTER_SIZE = 0.0500            # the font size that (made bold) makes a 38 mm capital
LETTER_EXTRUDE = 0.0029
LETTER_BEVEL = 0.0011
LETTER_OFFSET = 0.0015          # the strokes made bolder
MAGNET_R, MAGNET_H = 0.0030, 0.0016
_letters = {}


def _font_mesh(ch, extrude, bevel_depth, offset):
    cu = bpy.data.curves.new('_letter', 'FONT')
    cu.body = ch
    cu.size = LETTER_SIZE
    cu.extrude = extrude
    cu.bevel_depth = bevel_depth
    cu.bevel_resolution = 0
    cu.offset = offset
    cu.resolution_u = 4
    cu.fill_mode = 'BOTH'
    ob = bpy.data.objects.new('_letter', cu)
    bpy.context.scene.collection.objects.link(ob)
    s = _eval_to_shape(ob)
    bpy.data.objects.remove(ob)
    bpy.data.curves.remove(cu)
    return weld(s, 1e-6)


def _magnet_spot(ch):
    """Where in the letter's outline (its 2D fill, x and y) a magnet sits
    farthest from any edge: on a 0.4 mm grid, the inside point whose
    nearest outline edge is farthest."""
    flat = _font_mesh(ch, 0.0, 0.0, LETTER_OFFSET)
    bm = propkit.to_bmesh(flat)
    edges = [(e.verts[0].co.copy(), e.verts[1].co.copy()) for e in bm.edges if len(e.link_faces) == 1]
    tris = []
    for fc in bm.faces:
        vs = [v.co.copy() for v in fc.verts]
        for k in range(1, len(vs) - 1):
            tris.append((vs[0], vs[k], vs[k + 1]))
    bm.free()
    lo, hi = flat.bounds()

    def inside(x, y):
        for a, b, c in tris:
            d1 = (x - b.x) * (a.y - b.y) - (a.x - b.x) * (y - b.y)
            d2 = (x - c.x) * (b.y - c.y) - (b.x - c.x) * (y - c.y)
            d3 = (x - a.x) * (c.y - a.y) - (c.x - a.x) * (y - a.y)
            neg = d1 < 0 or d2 < 0 or d3 < 0
            pos = d1 > 0 or d2 > 0 or d3 > 0
            if not (neg and pos):
                return True
        return False

    def dist(x, y):
        best = 1e9
        for a, b in edges:
            ab = b - a
            L2 = ab.x * ab.x + ab.y * ab.y
            t = 0.0 if L2 < 1e-14 else max(0.0, min(1.0, ((x - a.x) * ab.x + (y - a.y) * ab.y) / L2))
            dx, dy = x - (a.x + ab.x * t), y - (a.y + ab.y * t)
            best = min(best, dx * dx + dy * dy)
        return math.sqrt(best)
    cx, cy = (lo.x + hi.x) / 2, (lo.y + hi.y) / 2
    best, spot = -1.0, (cx, cy)
    step = 0.0006
    nx, ny = int((hi.x - lo.x) / step), int((hi.y - lo.y) / step)
    for i in range(nx + 1):
        for j in range(ny + 1):
            x, y = lo.x + i * step, lo.y + j * step
            if not inside(x, y):
                continue
            # prefer the middle a little (a magnet goes near the centre)
            d = dist(x, y) - 0.04 * math.hypot(x - cx, y - cy)
            if d > best:
                best, spot = d, (x, y)
    return spot


def boolean_self(a, b):
    """a - b with the exact solver resolving a's own overlaps (a bold
    font's strokes run into each other)."""
    oa, ob = _mesh_obj('_ba', a), _mesh_obj('_bb', b)
    m = oa.modifiers.new('bool', 'BOOLEAN')
    m.operation = 'DIFFERENCE'
    m.object = ob
    m.solver = 'EXACT'
    m.use_self = True
    ob.hide_viewport = True
    out = _eval_to_shape(oa)
    _drop(oa, ob)
    return out


def letter_parts(ch):
    """The letter lying face up (its back on the floor at y = 0, its top
    toward -z), centred on its bounds; and its magnet in the pocket."""
    if ch in _letters:
        return _letters[ch]
    s = _font_mesh(ch, LETTER_EXTRUDE, LETTER_BEVEL, LETTER_OFFSET)
    mx, my = _magnet_spot(ch)
    lo, hi = s.bounds()
    c = Vector(((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, lo.z))
    # the pocket: a round recess in the back, the magnet set 0.3 mm inside it
    pocket = cyl(MAGNET_R + 0.0002, h=MAGNET_H * 2, seg=10, bevel_r=0).rot(math.pi / 2, 0, 0).move(mx, my, lo.z)
    s = boolean_self(s, pocket)
    magnet = cyl(MAGNET_R, h=MAGNET_H - 0.0003, seg=10, bevel_r=0).rot(math.pi / 2, 0, 0)
    magnet.move(mx, my, lo.z + 0.0003 + (MAGNET_H - 0.0003) / 2)

    def lay(sh):
        sh = sh.copy().move(-c.x, -c.y, -c.z)
        return sh.rot(-math.pi / 2, 0, 0)          # text +y -> -z, its face +z -> +y
    _letters[ch] = (decimate(lay(s), 470), lay(magnet))
    return _letters[ch]


# ---------------------------------------------------------------- parts ---
# the angle (degrees) below which each part's faces are shaded smooth, by
# its name's start (soft and round things high, cut and moulded edges lower)
SMOOTH = (('Real__rubberBall', 60), ('Real__pinwheelVane', 60), ('Real__pinwheelPin', 60),
          ('Real__pinwheelStick', 50), ('Real__toyCarBright', 50), ('Real__toyCarTyres', 50),
          ('Real__toyCar', 40), ('Real__puzzlePiece', 30), ('Real__fridgeLetter', 40))


def auto(name, build, budget, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. `kind` is the frame the game derives its
    texture coordinates from ('box': each face from its own side, 'plane':
    straight along z). (Each call's line ends with its budget: the asset
    lint reads it there.)"""
    shape = build()
    lo, hi = shape.bounds()
    smooth = next((a for k, a in SMOOTH if name.startswith(k)), 35.0)
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth)


auto('Real__rubberBallHalfA', lambda: ball_half('A'), 700)
auto('Real__rubberBallHalfB', lambda: ball_half('B'), 700)
auto('Real__toyCarBody', car_body, 1300)
auto('Real__toyCarGlass', car_glass, 280)
auto('Real__toyCarBright', car_bright, 760)
auto('Real__toyCarTail', car_tail, 100)
auto('Real__toyCarBase', car_base, 140)
auto('Real__toyCarTyres', car_tyres, 1000)
auto('Real__pinwheelVane', pinwheel_vane, 400)
auto('Real__pinwheelPin', pinwheel_pin, 280)
auto('Real__pinwheelStick', pinwheel_stick, 280)
auto('Real__puzzlePieceA', lambda: pz_piece('A')[1], 400)
auto('Real__puzzlePieceFaceA', lambda: pz_piece('A')[0], 280)
auto('Real__puzzlePieceB', lambda: pz_piece('B')[1], 400)
auto('Real__puzzlePieceFaceB', lambda: pz_piece('B')[0], 280)
auto('Real__fridgeLetterA', lambda: letter_parts('A')[0], 480)
auto('Real__fridgeLetterB', lambda: letter_parts('B')[0], 480)
auto('Real__fridgeLetterC', lambda: letter_parts('C')[0], 480)
auto('Real__fridgeLetterD', lambda: letter_parts('D')[0], 480)
auto('Real__fridgeLetterE', lambda: letter_parts('E')[0], 480)
auto('Real__fridgeLetterF', lambda: letter_parts('F')[0], 480)
auto('Real__fridgeLetterG', lambda: letter_parts('G')[0], 480)
auto('Real__fridgeLetterH', lambda: letter_parts('H')[0], 480)
auto('Real__fridgeLetterI', lambda: letter_parts('I')[0], 480)
auto('Real__fridgeLetterJ', lambda: letter_parts('J')[0], 480)
auto('Real__fridgeLetterK', lambda: letter_parts('K')[0], 480)
auto('Real__fridgeLetterL', lambda: letter_parts('L')[0], 480)
auto('Real__fridgeLetterM', lambda: letter_parts('M')[0], 480)
auto('Real__fridgeLetterN', lambda: letter_parts('N')[0], 480)
auto('Real__fridgeLetterO', lambda: letter_parts('O')[0], 480)
auto('Real__fridgeLetterP', lambda: letter_parts('P')[0], 480)
auto('Real__fridgeLetterQ', lambda: letter_parts('Q')[0], 480)
auto('Real__fridgeLetterR', lambda: letter_parts('R')[0], 480)
auto('Real__fridgeLetterS', lambda: letter_parts('S')[0], 480)
auto('Real__fridgeLetterT', lambda: letter_parts('T')[0], 480)
auto('Real__fridgeLetterU', lambda: letter_parts('U')[0], 480)
auto('Real__fridgeLetterV', lambda: letter_parts('V')[0], 480)
auto('Real__fridgeLetterW', lambda: letter_parts('W')[0], 480)
auto('Real__fridgeLetterX', lambda: letter_parts('X')[0], 480)
auto('Real__fridgeLetterY', lambda: letter_parts('Y')[0], 480)
auto('Real__fridgeLetterZ', lambda: letter_parts('Z')[0], 480)
for _ch in 'ABCDEFGHIJKLMNOPQRSTUVWXYZ':
    auto(f'Real__fridgeLetterMagnet{_ch}', lambda ch=_ch: letter_parts(ch)[1], 100)
