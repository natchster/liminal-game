"""Family `wood_metal_and_plastic_clutter`: industrial and domestic clutter.

Wood, rusted/painted metal and plastic are told apart by shape: planks with
gaps and chamfers, rolled rims and ribs on moulded plastic, crimped beads
and chimes on sheet metal, soft folds on cloth.
"""
from propkit import *
import propkit


# ------------------------------------------------------------ local kit --
def _fw(s):
    return propkit._fix_winding(s)


def ring_section(R, sec, seg=32, arc=TAU, a0=0.0):
    """Sweep a closed 2D section [(dr, dz), ...] around the Z axis in the XY
    plane (a torus_geo! with any cross-section). arc < TAU caps the ends."""
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
    return _fw(Shape(v, f))


def circ_sec(r, n=8, sx=1.0, sy=1.0):
    return [(r * sx * math.cos(TAU * i / n), r * sy * math.sin(TAU * i / n)) for i in range(n)]


def rr_sec(w, h, r, seg=2):
    return rounded_rect(w, h, r, seg)


def sweep(points, sec, scale=None, smooth=0, up=(0, 0, 1), caps=True):
    """Sweep a closed 2D section along a 3D polyline (parallel-transport
    frame; section x = t x up, section y = t x x). scale(t) -> k or (kx, ky)."""
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


def arc_pts(R, a0, a1, n, z=0.0):
    return [(R * math.cos(a0 + (a1 - a0) * i / n), R * math.sin(a0 + (a1 - a0) * i / n), z) for i in range(n + 1)]


def torus_arc_bounds(R, r, arc):
    """Bounds of torus_geo!(R, r, .., .., arc) (y >= 0 for arc <= PI)."""
    xs, ys = [], []
    for i in range(721):
        u = arc * i / 720
        for j in range(24):
            w = TAU * j / 24
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r))


def rim_ring(R, r, seg=36, n=7, flat_in=0.7):
    """A rolled rim bead: round outside, flatter inside."""
    sec = [(r * math.cos(a) * (1.0 if math.cos(a) > 0 else flat_in), r * math.sin(a))
           for a in (TAU * i / n for i in range(n))]
    return ring_section(R, sec, seg)


def bail(R, r_wire, r_grip, grip=0.22, n=22, sec_n=6):
    """A wire bucket bail over the top (XY plane, y >= 0) with a thicker
    grip roll in the middle and the ends turned inward into the ears."""
    hook = 0.06
    path = [(R - 0.007, r_wire * 0.6, 0), (R - 0.002, r_wire * 1.2, 0)]
    path += arc_pts(R, hook, math.pi - hook, n)
    path += [(-R + 0.002, r_wire * 1.2, 0), (-R + 0.007, r_wire * 0.6, 0)]
    def sc(t):
        d = abs(t - 0.5)
        k = r_wire + (r_grip - r_wire) * max(0.0, min(1.0, (grip - d) / 0.05))
        return k / r_wire
    return sweep(path, circ_sec(r_wire, sec_n), scale=sc)


# ------------------------------------------------------------ crates ------
def crate_body():
    """Slatted freight crate: corner battens, four boards a side with gaps
    where the slat decals sit (1/4, 1/2, 3/4 height), a planked lid."""
    S = 0.40
    h = S / 2
    rnd = random.Random(8)
    post, rec, t, gap = 0.042, 0.003, 0.016, 0.006
    out = Shape()
    for sx in (-1, 1):
        for sz in (-1, 1):
            out += rbox(post, S, post, r=0.004, seg=1).move(x=sx * (h - post / 2), z=sz * (h - post / 2))
    blen = S - 2 * post + 0.006
    bh = S / 4 - gap
    for k in range(4):
        yc = -h + S / 8 + k * S / 4
        for side in range(4):
            d = h - rec - t / 2 - rnd.uniform(0, 0.0015)
            b = rbox(blen, bh - rnd.uniform(0, 0.002), t, r=0.003, seg=1)
            b.rot(z=rnd.uniform(-0.006, 0.006))
            b.move(y=yc, z=d).rot(y=side * math.pi / 2)
            out += b
    lid_t = 0.018
    for k in range(3):
        w = S / 3 - gap
        b = rbox(S, lid_t, w, r=0.003, seg=1).move(y=h - lid_t / 2, z=-S / 3 + k * S / 3)
        out += b
    core = S - 2 * (rec + t) + 0.003
    out += box(core, S - 0.004, core)
    return out

part('WoodenCrates__body', f_box(0.40, 0.40, 0.40), crate_body)
part('WoodenCrates__pellet', f_sphere(0.010),
     lambda: noisy(sphere(0.010, seg=6, rings=4), 0.0014, scale=180, seed=3), smooth=180)


# ------------------------------------------------------------ firewood ----
def firewood_log():
    """Split-bark log along +Y. The -Y end is a clean saw cut (the core decal
    sits on it); the +Y end is chamfered and a little uneven."""
    R, L = 0.030, 0.15
    seg, rings = 28, 7
    v, f = [], []
    def rad(a, y):
        ridge = 0.0055 * abs(math.sin(3.5 * a + 0.5 * math.sin(2 * a) + 5 * y)) ** 0.22
        lump = 0.0016 * noise.noise(Vector((math.cos(a) * 2.2, math.sin(a) * 2.2, y * 26)))
        return R - 0.0058 + ridge + lump
    bendx = lambda y: 0.0018 * math.cos(math.pi * y / L)
    ids = []
    for i in range(rings):
        y = -L / 2 + L * i / (rings - 1)
        row = []
        for j in range(seg):
            a = TAU * j / seg
            r = rad(a, y)
            v.append((r * math.sin(a) + bendx(y), y, r * math.cos(a)))
            row.append(len(v) - 1)
        ids.append(row)
    for a_, b_ in zip(ids, ids[1:]):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((a_[j], b_[j], b_[j2], a_[j2]))
    # saw-cut end (-Y): bark rim, flat face
    inner = []
    for j in range(seg):
        a = TAU * j / seg
        r = rad(a, -L / 2) * 0.86
        v.append((r * math.sin(a) + bendx(-L / 2), -L / 2, r * math.cos(a)))
        inner.append(len(v) - 1)
    v.append((bendx(-L / 2), -L / 2, 0))
    c0 = len(v) - 1
    for j in range(seg):
        j2 = (j + 1) % seg
        f.append((ids[0][j], ids[0][j2], inner[j2], inner[j]))
        f.append((inner[j], inner[j2], c0))
    # rough end (+Y): chamfered bark, slightly slanted face
    top = []
    for j in range(seg):
        a = TAU * j / seg
        r = rad(a, L / 2) * 0.84
        v.append((r * math.sin(a) + bendx(L / 2), L / 2 - 0.004 - 0.003 * math.sin(a), r * math.cos(a)))
        top.append(len(v) - 1)
    v.append((bendx(L / 2) + 0.002, L / 2 - 0.004, 0.001))
    c1 = len(v) - 1
    for j in range(seg):
        j2 = (j + 1) % seg
        f.append((ids[-1][j], ids[-1][j2], top[j2], top[j]))
        f.append((top[j], top[j2], c1))
    return _fw(Shape(v, f))

part('FirewoodStacks__log', f_cyl(0.030, 0.030, 0.15), firewood_log, smooth=28)


# ------------------------------------------------------------ buckets -----
def plastic_bucket(rt, rb, h, wall=0.004, ribs=(0.32, 0.62), collar=0.03, seg=28):
    """Moulded plastic bucket, open at the top: foot bead, two stiffening
    ribs, a stepped collar under a rolled rim, thin wall down to a floor."""
    y0, y1 = -h / 2, h / 2
    rr = lambda y: rb + (rt - rb) * (y - y0) / h
    prof = [(0.0, y0), (rb - 0.006, y0), (rb - 0.001, y0 + 0.003), (rr(y0 + 0.012), y0 + 0.012)]
    for fy in ribs:
        y = y0 + fy * h
        prof += [(rr(y - 0.006), y - 0.006), (rr(y) + 0.0028, y), (rr(y + 0.006), y + 0.006)]
    yc = y1 - collar
    prof += [(rr(yc), yc), (rt - 0.0035, yc + 0.004), (rt - 0.003, y1 - 0.007),
             (rt, y1 - 0.004), (rt - 0.0012, y1), (rt - wall - 0.001, y1 - 0.0005)]
    prof += [(rt - wall - 0.002, y1 - 0.006), (rb - wall, y0 + wall + 0.012),
             (rb - wall - 0.012, y0 + wall), (0.0, y0 + wall)]
    return lathe(prof, seg, cap_bottom=False, cap_top=False)

part('CleaningMopBuckets__bucket', f_cyl(0.13, 0.11, 0.20),
     lambda: plastic_bucket(0.13, 0.11, 0.20, seg=28))
part('CleaningMopBuckets__rim', f_torus(0.13, 0.006), lambda: rim_ring(0.13, 0.006, seg=36))
part('CleaningMopBuckets__bail', torus_arc_bounds(0.13 * 1.02, 0.004, math.pi),
     lambda: bail(0.13 * 1.02, 0.0022, 0.004))
part('CleaningMopBuckets__nub', f_sphere(0.005),
     lambda: lathe([(0, -0.005), (0.0035, -0.0045), (0.005, -0.002), (0.005, 0.001),
                    (0.0038, 0.0038), (0.0015, 0.005), (0, 0.005)], seg=8), smooth=180)


def mop_stick():
    """Mop handle: threaded bottom end, long dowel, rounded top cap."""
    prof = [(0, -0.55), (0.0065, -0.55), (0.0085, -0.547)]
    y = -0.543
    for k in range(6):
        prof += [(0.0095, y), (0.0083, y + 0.003)]
        y += 0.006
    prof += [(0.0097, -0.505), (0.010, -0.50), (0.010, 0.515), (0.0094, 0.532),
             (0.0078, 0.543), (0.0045, 0.549), (0, 0.55)]
    return lathe(prof, seg=10)

part('CleaningMopBuckets__stick', f_cyl(0.010, 0.010, 1.10), mop_stick)
part('CleaningMopBuckets__grip', f_torus(0.011, 0.003),
     lambda: ring_section(0.011, rr_sec(0.006, 0.006, 0.0018, seg=1), seg=16))


def mop_strand():
    """A twisted cotton strand: thick where it is clamped (+Y), thin and
    wavy toward the frayed tip (-Y)."""
    pts = [(0.0004 * math.sin(i * 1.7), 0.04 - 0.08 * i / 5, 0.0004 * math.cos(i * 2.3)) for i in range(6)]
    return sweep(pts, circ_sec(0.0016, 5), scale=lambda t: 1.0 - 0.45 * t, smooth=1, up=(1, 0, 0))

part('CleaningMopBuckets__strand', f_cyl(0.002, 0.001, 0.08), mop_strand, smooth=180)
part('CleaningMopBuckets__clamp', f_cyl(0.014, 0.014, 0.020),
     lambda: lathe([(0, -0.01), (0.0122, -0.01), (0.014, -0.0085), (0.014, -0.004), (0.0131, -0.003),
                    (0.0131, 0.003), (0.014, 0.004), (0.0136, 0.008), (0.0112, 0.01), (0, 0.01)], seg=12))

part('OverturnedBuckets__body', f_cyl(0.14, 0.11, 0.22),
     lambda: plastic_bucket(0.14, 0.11, 0.22, ribs=(0.28, 0.55), collar=0.034, seg=28))
part('OverturnedBuckets__rim', f_torus(0.14, 0.006), lambda: rim_ring(0.14, 0.006, seg=38))


def revolve_loop(sec, seg=32):
    """Revolve a closed (r, y) section around +Y (rings, sleeves, collars)."""
    return ring_section(0.0, sec, seg).rot(x=-math.pi / 2)


# ------------------------------------------------------------ ladders -----
def ladder_rail():
    """Rectangular wooden stile, eased edges, the top end rounded over."""
    s = rbox(0.024, 1.6, 0.024, r=0.004, seg=2)
    def ends(p):
        if p.y > 0.79:
            p.x *= 0.9
        return p
    return s.warp(ends)


def ladder_rung():
    """Turned rung with shallow anti-slip flutes and tenons into the stiles."""
    prof = [(0, -0.175), (0.0074, -0.175), (0.0078, -0.164), (0.0094, -0.160), (0.0100, -0.150),
            (0.0100, 0.150), (0.0094, 0.160), (0.0078, 0.164), (0.0074, 0.175), (0, 0.175)]
    s = lathe(prof, seg=16)
    def flute(p):
        if abs(p.y) < 0.151:
            a = math.atan2(p.z, p.x)
            k = 1 - 0.09 * max(0.0, math.cos(4 * a)) ** 3
            p.x *= k
            p.z *= k
        return p
    return s.warp(flute)

part('SmallLadders__rail', f_cyl(0.012, 0.012, 1.6), ladder_rail)
part('SmallLadders__rung', f_cyl(0.010, 0.010, 0.35), ladder_rung)


def paint_pail():
    """Sheet-metal paint pail with its lid on: rolled bottom chime, two
    stiffening beads, lid seated in a channel inside the top chime."""
    prof = [(0, -0.06), (0.066, -0.06), (0.0705, -0.0575), (0.0712, -0.0545), (0.0703, -0.051),
            (0.0735, -0.022), (0.0752, -0.019), (0.0745, -0.016),
            (0.0765, 0.018), (0.0782, 0.021), (0.0775, 0.024),
            (0.0786, 0.052), (0.0800, 0.0565), (0.0785, 0.060), (0.0748, 0.0598),
            (0.0735, 0.0565), (0.0705, 0.0565), (0.0695, 0.0585), (0, 0.0585)]
    return lathe(prof, seg=16)

part('SmallLadders__pail', f_cyl(0.08, 0.07, 0.12), paint_pail)
part('SmallLadders__pailRim', f_torus(0.08, 0.004), lambda: rim_ring(0.08, 0.004, seg=32, n=6))
part('SmallLadders__pailBail', torus_arc_bounds(0.075, 0.003, math.pi),
     lambda: bail(0.075, 0.0017, 0.003, grip=0.18, n=18))


# ------------------------------------------------------------ ceiling tile -
def ceiling_tile():
    """Mineral-fibre ceiling tile: tegular (stepped) face panel with a
    fissured, lumpy surface, a slightly crushed edge and a snapped corner."""
    base = rbox(0.60, 0.012, 0.60, r=0.002, seg=1).move(y=-0.004)
    top = slab(0.57, 0.57, 0.008, 10, 10).rot(x=-math.pi / 2).move(y=0.006)
    def fissure(p):
        if p.y > 0.008:
            n = noise.noise(Vector((p.x * 40, p.z * 40, 1.7)))
            p.y -= 0.0012 * max(0.0, n) + 0.0006 * abs(noise.noise(Vector((p.x * 90, p.z * 90, 3.1))))
        return p
    top = top.warp(fissure)
    snap = prism([(0.12, 0.31), (0.31, 0.31), (0.31, 0.15), (0.29, 0.172), (0.262, 0.166),
                  (0.245, 0.2), (0.21, 0.214), (0.19, 0.25), (0.158, 0.262), (0.14, 0.295)], 0.05).rot(x=-math.pi / 2)
    tile = cut(base, snap) + cut(top, snap)
    def crush(p):
        p.y += 0.0025 * ((p.x / 0.3) ** 2 + (p.z / 0.3) ** 2) * 0.5     # a little dished
        if p.x < -0.27:
            p.y -= 0.002 * (1 + noise.noise(Vector((p.z * 30, 0.5, 0.5))))
        return p
    return tile.warp(crush)

part('FallenCeilingTiles__tile', f_box(0.60, 0.02, 0.60), ceiling_tile)


# ------------------------------------------------------------ traffic cone -
def cone_base():
    """Square rubber base with rounded corners, bevelled top edge and a
    raised collar the cone foot sits in."""
    slab_ = prism(rounded_rect(0.20, 0.20, 0.026, seg=3), 0.010, bevel_r=0.002).rot(x=-math.pi / 2).move(y=-0.0025)
    collar = lathe([(0, 0.0015), (0.089, 0.0015), (0.0885, 0.0055), (0.0862, 0.0075), (0, 0.0075)], seg=32)
    return slab_ + collar


def cone_body():
    """Moulded cone: flared foot, straight taper, open top with a lip."""
    prof = [(0, -0.15), (0.080, -0.15), (0.0785, -0.146), (0.0715, -0.137), (0.0655, -0.122),
            (0.0215, 0.141), (0.0200, 0.148), (0.0165, 0.150), (0.0132, 0.146), (0.0122, 0.13), (0, 0.12)]
    return lathe(prof, seg=28)


def cone_band():
    """Reflective sleeve: a hollow collar round the cone, softened edges."""
    sec = [(0.0405, -0.015), (0.0540, -0.015), (0.0551, -0.0138), (0.0598, 0.0135),
           (0.0592, 0.015), (0.0375, 0.015)]
    return revolve_loop(sec, seg=28)

part('TrafficCones__base', f_box(0.20, 0.015, 0.20), cone_base)
part('TrafficCones__body', f_cyl(0.02, 0.08, 0.30), cone_body)
part('TrafficCones__band', f_cyl(0.06, 0.055, 0.03), cone_band)


# ------------------------------------------------------------ drain grate --
def drain_plate():
    """Cast frame round the drain hole (flat annulus, bevelled outer edge);
    thickness goes down (behind the plane)."""
    sec = [(0.085, -0.003), (0.110, -0.003), (0.110, -0.0018), (0.1065, 0.0), (0.0875, 0.0), (0.085, -0.0012)]
    return ring_section(0.0, sec, seg=36)


def drain_rim():
    """Chunky cast rim: wide foot (frame +Z is world down), chamfered crown."""
    sec = [(-0.006, 0.006), (0.006, 0.006), (0.0055, 0.0), (0.0035, -0.0052),
           (0.0015, -0.006), (-0.0015, -0.006), (-0.0035, -0.0052), (-0.0055, 0.0)]
    return ring_section(0.10, sec, seg=34)

part('FloorDrainGrates__plate', f_bounds((-0.11, -0.10724, 0), (0.11, 0.10724, 0), 'plane'), drain_plate)
part('FloorDrainGrates__rim', f_torus(0.10, 0.006), drain_rim)
part('FloorDrainGrates__bar', f_box(0.18, 0.006, 0.006),
     lambda: taper(rbox(0.18, 0.006, 0.006, r=0.0016, seg=2), top=0.8, along='y', axes='z'))


# ------------------------------------------------------------ coat hooks --
def x_extrude(outline_zy, length, bevel_r=0.0):
    """Extrude a (z, y) cross-section along X (mouldings, rails)."""
    return prism([(-z, y) for z, y in outline_zy], length, bevel_r=bevel_r).rot(y=math.pi / 2)


def hook_rail():
    """Wall rail with a moulded front: bead along the top, flat face,
    chamfered underside; two countersunk screws."""
    sec = round_outline([(-0.02, -0.015), (0.011, -0.015), (0.02, -0.006), (0.02, 0.005),
                         (0.0175, 0.0085), (0.0195, 0.0118), (0.0165, 0.015), (-0.02, 0.015)], 0.0015, seg=2)
    s = x_extrude(sec, 0.70, bevel_r=0.0015)
    for sx in (-0.225, 0.225):
        s += screw_head(0.0035, 0.0012).rot(x=math.pi / 2).move(x=sx, y=-0.0005, z=0.0196)
    return s

part('CoatHookRows__rail', f_box(0.70, 0.03, 0.04), hook_rail)
part('CoatHookRows__hook', f_cyl(0.006, 0.006, 0.05),
     lambda: lathe([(0, -0.025), (0.0046, -0.025), (0.006, -0.0228), (0.0056, -0.0205), (0.0034, -0.017),
                    (0.0029, 0.011), (0.0042, 0.0155), (0.0060, 0.0195), (0.0053, 0.0235), (0.0028, 0.025),
                    (0, 0.025)], seg=10))


def coat_torso():
    """A coat hanging off a hook: turned-up collar, rounded shoulders,
    a button line down the front (+Z), soft vertical folds that deepen
    toward an uneven hem; hollow underneath."""
    seg, rows = 24, 14
    H = 0.45
    v, f = [], []
    rings = []
    for i in range(rows):
        t = i / (rows - 1)                       # 0 = hem, 1 = collar
        y = -H / 2 + H * t
        if t < 0.84:
            base = 0.128 - 0.036 * (t / 0.84)
        elif t < 0.95:
            u = (t - 0.84) / 0.11
            base = 0.092 - 0.030 * u ** 1.6
        else:
            base = 0.040
        amp = 0.11 * (1 - t) ** 1.1 + 0.015
        row = []
        for j in range(seg):
            a = TAU * j / seg
            fold = math.sin(6 * a + 0.7 * math.sin(2 * a + 3 * t))
            r = base * (1 + fold * amp * 0.5)
            da = (a + math.pi) % TAU - math.pi          # 0 at the front
            r *= 1 - 0.07 * math.exp(-(da / 0.16) ** 2) * (1.0 if t < 0.95 else 0.0)
            if 0.8 < t < 0.95:
                r *= 1 - 0.05 * math.exp(-(abs(da) - 0.45) ** 2 / 0.03)   # lapel notch
            yy = y
            if i == 0:
                yy += 0.012 * noise.noise(Vector((math.cos(a) * 3, math.sin(a) * 3, 0.3)))
            if 0.84 < t < 0.97:
                yy -= 0.01 * abs(math.sin(a))       # shoulders fall away from the hook
            v.append((r * math.sin(a), yy, r * math.cos(a)))
            row.append(len(v) - 1)
        rings.append(row)
    for a_, b_ in zip(rings, rings[1:]):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((a_[j], b_[j], b_[j2], a_[j2]))
    v.append((0, H / 2 - 0.012, 0))
    top = len(v) - 1
    for j in range(seg):
        f.append((rings[-1][j], top, rings[-1][(j + 1) % seg]))
    inner = []
    for j in range(seg):
        p = Vector(v[rings[0][j]])
        v.append((p.x * 0.9, p.y + 0.015, p.z * 0.9))
        inner.append(len(v) - 1)
    v.append((0, -H / 2 + 0.07, 0))
    c = len(v) - 1
    for j in range(seg):
        j2 = (j + 1) % seg
        f.append((rings[0][j], rings[0][j2], inner[j2], inner[j]))
        f.append((inner[j], inner[j2], c))
    return noisy(_fw(Shape(v, f)), 0.0025, scale=14, seed=5)


def coat_sleeve():
    """Hanging sleeve: tube with compression creases, a turned-back cuff
    and a hollow opening at the bottom."""
    seg, rows = 14, 10
    L = 0.25
    v, f, rings = [], [], []
    for i in range(rows):
        t = i / (rows - 1)
        y = -L / 2 + L * t
        r = 0.036 - 0.008 * t
        if t < 0.14:
            r += 0.004                          # cuff
        crease = 0.0025 * math.sin(t * 22)
        row = []
        for j in range(seg):
            a = TAU * j / seg
            rr = r + crease * (0.5 + 0.5 * math.cos(a * 2 + t * 4))
            v.append((rr * math.sin(a) + 0.004 * math.sin(t * math.pi), y, rr * math.cos(a)))
            row.append(len(v) - 1)
        rings.append(row)
    for a_, b_ in zip(rings, rings[1:]):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((a_[j], b_[j], b_[j2], a_[j2]))
    v.append((0, L / 2, 0))
    top = len(v) - 1
    for j in range(seg):
        f.append((rings[-1][j], top, rings[-1][(j + 1) % seg]))
    inner = []
    for j in range(seg):
        p = Vector(v[rings[0][j]])
        v.append((p.x * 0.78, p.y + 0.01, p.z * 0.78))
        inner.append(len(v) - 1)
    v.append((0, -L / 2 + 0.05, 0))
    c = len(v) - 1
    for j in range(seg):
        j2 = (j + 1) % seg
        f.append((rings[0][j], rings[0][j2], inner[j2], inner[j]))
        f.append((inner[j], inner[j2], c))
    return noisy(_fw(Shape(v, f)), 0.0015, scale=30, seed=9)

part('CoatHookRows__torso', f_cyl(0.09, 0.13, 0.45), coat_torso, smooth=180)
part('CoatHookRows__sleeve', f_cyl(0.03, 0.04, 0.25), coat_sleeve, smooth=180)


# ------------------------------------------------------------ umbrellas ---
def umbrella_shaft():
    """Thin steel shaft, a collar where the handle joins (-Y) and the
    runner sleeve just below the folded canopy."""
    prof = [(0, -0.425), (0.0088, -0.425), (0.010, -0.421), (0.010, -0.405), (0.0068, -0.398),
            (0.0052, -0.39), (0.0052, 0.074), (0.0085, 0.08), (0.010, 0.086), (0.010, 0.118),
            (0.0078, 0.125), (0.0052, 0.13), (0.0045, 0.42), (0, 0.425)]
    return lathe(prof, seg=10)


def umbrella_canopy():
    """Folded canopy: eight fabric folds between the ribs, twisted, bulging
    over the gathered rib tips at the bottom, tapering to the tip, with a
    tie strap round the bundle."""
    seg, rows = 24, 11
    H = 0.30
    v, f, rings = [], [], []
    for i in range(rows):
        t = i / (rows - 1)
        y = -H / 2 + H * t
        if t < 0.12:
            base = 0.02 + 0.025 * math.sin(t / 0.12 * math.pi / 2)
        else:
            base = 0.045 - 0.022 * ((t - 0.12) / 0.88) ** 1.1
        if t > 0.94:
            base *= 0.6
        strap = 0.45 < t < 0.56
        amp = 0.05 if strap else 0.2 * (1 - 0.6 * t)
        row = []
        for j in range(seg):
            a = TAU * j / seg + 0.9 * t
            r = base * (1 - amp * (0.5 + 0.5 * math.cos(8 * a)))
            if strap:
                r = base * 0.97
            v.append((r * math.sin(a), y, r * math.cos(a)))
            row.append(len(v) - 1)
        rings.append(row)
    for a_, b_ in zip(rings, rings[1:]):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((a_[j], b_[j], b_[j2], a_[j2]))
    for ring, yc in ((rings[0], -H / 2 + 0.01), (rings[-1], H / 2)):
        v.append((0, yc, 0))
        c = len(v) - 1
        for j in range(seg):
            f.append((ring[j], ring[(j + 1) % seg], c))
    return _fw(Shape(v, f))

part('Umbrellas__shaft', f_cyl(0.010, 0.010, 0.85), umbrella_shaft)
part('Umbrellas__canopy', f_cyl(0.025, 0.045, 0.30), umbrella_canopy, smooth=50)
part('Umbrellas__handle', f_cyl(0.012, 0.012, 0.06),
     lambda: lathe([(0, -0.03), (0.0088, -0.03), (0.0115, -0.027), (0.012, -0.021), (0.0104, -0.014),
                    (0.0112, -0.007), (0.0104, 0.0), (0.0112, 0.007), (0.0104, 0.014), (0.0098, 0.022),
                    (0.0075, 0.026), (0.0072, 0.03), (0, 0.03)], seg=10))


# ------------------------------------------------------------ wall mirror -
def rect_rings(w, h, rings):
    """Rectangular rings [(inset, z), ...] from the outer edge inward, mitred
    at the corners and joined by quads; the innermost ring is filled (front)
    and the outermost closed at the back. Front faces +Z."""
    v, f = [], []
    for u, z in rings:
        x, y = w / 2 - u, h / 2 - u
        v += [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    for k in range(len(rings) - 1):
        for c in range(4):
            c2 = (c + 1) % 4
            f.append((k * 4 + c, k * 4 + c2, (k + 1) * 4 + c2, (k + 1) * 4 + c))
    n = len(rings) - 1
    f.append((n * 4, n * 4 + 1, n * 4 + 2, n * 4 + 3))
    f.append((3, 2, 1, 0))
    return _fw(Shape(v, f))


def mirror_frame():
    """Moulded frame: rounded outer edge, a bead, a cove and an inner step
    down to the mirror rebate; the back sits flat on the wall (-Z)."""
    prof = [(0.0, -0.008), (0.0, -0.0036), (0.0012, -0.0016), (0.0035, -0.0004), (0.0065, 0.0),
            (0.0095, -0.0006), (0.0118, -0.0022), (0.0145, -0.0036), (0.0172, -0.0034),
            (0.0192, -0.0017), (0.0212, -0.0006), (0.0236, -0.0010), (0.0256, -0.0026),
            (0.0268, -0.0046), (0.0300, -0.0046)]
    return rect_rings(0.42, 0.60, prof)

part('WallMirrors__frame', f_bounds((-0.21, -0.30, 0), (0.21, 0.30, 0), 'plane'), mirror_frame)
part('WallMirrors__glass', f_bounds((-0.18, -0.27, 0), (0.18, 0.27, 0), 'plane'),
     lambda: rect_rings(0.36, 0.54, [(0.0, -0.006), (0.0, -0.0024), (0.0015, -0.0016), (0.013, 0.0)]),
     smooth=20)


def faucet_handle():
    """Chrome cross-head tap: round escutcheon on the wall (-Z), stem,
    a four-spoke head and a domed hub."""
    esc = lathe([(0, 0), (0.0148, 0), (0.0150, 0.0015), (0.012, 0.0036), (0.0055, 0.0045), (0, 0.0045)], seg=12)
    esc = esc.rot(x=math.pi / 2).move(z=-0.015)
    stem = cyl(0.0042, 0.0042, 0.018, seg=8, bevel_r=0).rot(x=math.pi / 2).move(z=-0.003)
    spokes = Shape()
    for a in (0, math.pi / 2):
        spokes += rbox(0.030, 0.0058, 0.006, r=0.0024, seg=1).rot(z=a + math.pi / 4).move(z=0.0075)
    hub = lathe([(0, 0), (0.0068, 0), (0.0062, 0.003), (0.0038, 0.0058), (0, 0.0066)], seg=10)
    hub = hub.rot(x=math.pi / 2).move(z=0.0084)
    s = esc + stem + spokes + hub
    return s.fit((-0.015, -0.015, -0.015), (0.015, 0.015, 0.015))

part('WallMirrors__tap', f_box(0.03, 0.03, 0.03), faucet_handle)


def soap_dispenser():
    """Wall soap dispenser: rounded base, lid seam near the top, domed
    push cap and a recessed level window on the room side (+Z)."""
    prof = [(0, -0.08), (0.0175, -0.08), (0.0228, -0.077), (0.0248, -0.071), (0.025, -0.062),
            (0.025, 0.043), (0.0236, 0.0455), (0.0236, 0.0485), (0.025, 0.051), (0.025, 0.066),
            (0.0232, 0.0745), (0.017, 0.0785), (0.008, 0.08), (0, 0.08)]
    s = lathe(prof, seg=18)
    return cut(s, rbox(0.011, 0.07, 0.02, r=0.004, seg=1).move(y=-0.005, z=0.0295))

part('WallMirrors__dispenser', f_cyl(0.025, 0.025, 0.16), soap_dispenser)
part('WallMirrors__nozzle', f_cyl(0.006, 0.006, 0.02),
     lambda: lathe([(0, -0.01), (0.0022, -0.01), (0.0028, -0.006), (0.0034, -0.003), (0.0055, 0.0005),
                    (0.006, 0.004), (0.006, 0.01), (0, 0.01)], seg=8), smooth=50)


def soap_bar():
    """Worn soap bar: soft pillow edges, top domed and rubbed smooth."""
    s = rbox(0.06, 0.02, 0.035, r=0.0075, seg=3)
    def pillow(p):
        if p.y > 0:
            p.y *= 1 - 0.18 * ((p.x / 0.03) ** 2 + (p.z / 0.0175) ** 2) * 0.5
        return p
    return s.warp(pillow)

part('WallMirrors__soap', f_box(0.06, 0.020, 0.035), soap_bar, smooth=60)
part('WallMirrors__brush', f_cyl(0.004, 0.004, 0.10),
     lambda: lathe([(0, -0.05), (0.0022, -0.05), (0.0024, -0.032), (0.0030, -0.022), (0.0038, -0.008),
                    (0.0040, 0.004), (0.0034, 0.018), (0.0040, 0.034), (0.0034, 0.047), (0, 0.05)], seg=8),
     smooth=60)


def bristles():
    """Toothbrush head: a thin backing plate with 3 x 2 upright tufts."""
    s = box(0.012, 0.0018, 0.008).move(y=-0.0021)
    for i in range(3):
        for k in range(2):
            hgt = 0.0042 - 0.0004 * ((i + k) % 2)
            s += box(0.0033, hgt, 0.0033).move(x=-0.004 + 0.004 * i, y=-0.0012 + hgt / 2, z=-0.002 + 0.004 * k)
    return s

part('WallMirrors__bristles', f_box(0.012, 0.006, 0.008), bristles, flat=True)


# ------------------------------------------------------------ luggage -----
def outline_tube(outline, sec, z=0.0):
    """Sweep a 2D section (d_out, d_z) round a closed XY outline (piping,
    beads round a face). Corners are mitred."""
    n, m = len(outline), len(sec)
    pts = [Vector((x, y, 0)) for x, y in outline]
    v, f = [], []
    for i in range(n):
        a, b, c = pts[i - 1], pts[i], pts[(i + 1) % n]
        e1 = (b - a).normalized()
        e2 = (c - b).normalized()
        n1 = Vector((e1.y, -e1.x, 0))
        n2 = Vector((e2.y, -e2.x, 0))
        nb = (n1 + n2).normalized()
        k = 1.0 / max(0.3, nb.dot(n1))
        for d, dz in sec:
            v.append(b + nb * d * k + Vector((0, 0, z + dz)))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    return _fw(Shape(v, f))


def superquad(a, b, d, e_lat=0.3, e_lon=0.3, nlon=20, nlat=12):
    """Rounded box / pillow: a superellipsoid with half-sizes a, b, d."""
    sp = lambda w, e: math.copysign(abs(w) ** e, w)
    v, f = [(0, -b, 0)], []
    for i in range(1, nlat):
        th = -math.pi / 2 + math.pi * i / nlat
        for j in range(nlon):
            ph = TAU * j / nlon
            cl, sl = sp(math.cos(th), e_lat), sp(math.sin(th), e_lat)
            v.append((a * cl * sp(math.cos(ph), e_lon), b * sl, d * cl * sp(math.sin(ph), e_lon)))
    v.append((0, b, 0))
    top = len(v) - 1
    ring = lambda i: [1 + (i - 1) * nlon + j for j in range(nlon)]
    for j in range(nlon):
        f.append((0, ring(1)[(j + 1) % nlon], ring(1)[j]))
        f.append((top, ring(nlat - 1)[j], ring(nlat - 1)[(j + 1) % nlon]))
    for i in range(1, nlat - 1):
        r0, r1 = ring(i), ring(i + 1)
        for j in range(nlon):
            j2 = (j + 1) % nlon
            f.append((r0[j], r0[j2], r1[j2], r1[j]))
    return _fw(Shape(v, f))


def suitcase_body():
    """Leather suitcase: lid and base shells split by a seam near the
    front, piping round both big faces, leather corner caps."""
    W, H, D = 0.40, 0.28, 0.14
    base = rbox(W - 0.008, H - 0.008, 0.086, r=0.012, seg=2).move(z=-D / 2 + 0.004 + 0.043)
    lid = rbox(W - 0.008, H - 0.008, 0.044, r=0.012, seg=2).move(z=D / 2 - 0.004 - 0.022)
    core = box(W - 0.02, H - 0.02, 0.012).move(z=0.024)
    out = base + lid + core
    rr = rounded_rect(W - 0.006, H - 0.006, 0.016, seg=2)
    for zz in (-D / 2 + 0.006, D / 2 - 0.006):
        out += outline_tube(rr, circ_sec(0.003, 5), z=zz)
    for sx in (-1, 1):
        for sy in (-1, 1):
            for sz in (-1, 1):
                out += rbox(0.034, 0.034, 0.03, r=0.009, seg=1).move(
                    x=sx * (W / 2 - 0.017), y=sy * (H / 2 - 0.017), z=sz * (D / 2 - 0.015))
    return out

part('Suitcases__body', f_box(0.40, 0.28, 0.14), suitcase_body)


def leather_handle(R, r):
    """Carry handle: a padded grip arch, thinner towards two mounting
    blocks at the ends."""
    path = arc_pts(R - r * 0.3, 0.0, math.pi, 16)
    s = sweep(path, circ_sec(r, 8, sx=0.72), scale=lambda t: 0.7 + 0.3 * math.sin(math.pi * t) ** 0.6)
    for sx in (-1, 1):
        s += rbox(r * 2.2, r * 1.6, r * 2.0, r=r * 0.4, seg=1).move(x=sx * (R - r * 0.3), y=r * 0.8)
    return s

part('Suitcases__handle', torus_arc_bounds(0.06, 0.008, math.pi), lambda: leather_handle(0.06, 0.008))


def suitcase_latch():
    """Brass draw latch: rounded housing with a slot and a thumb slide."""
    s = rbox(0.03, 0.009, 0.024, r=0.0028, seg=2).move(y=-0.0015)
    s = cut(s, box(0.014, 0.004, 0.004).move(y=0.003, z=0.004))
    s += rbox(0.009, 0.004, 0.006, r=0.0012, seg=1).move(y=0.004, z=-0.004)
    return s

part('Suitcases__latch', f_box(0.03, 0.012, 0.024), suitcase_latch)
part('Suitcases__plate', f_bounds((-0.03, -0.02, 0), (0.03, 0.02, 0), 'plane'),
     lambda: prism(rounded_rect(0.06, 0.04, 0.005, seg=2), 0.0011, bevel_r=0.0004).move(z=-0.0009)
     + screw_head(0.0026, 0.0008, slot=False).rot(x=math.pi / 2).move(x=-0.022, z=-0.0008)
     + screw_head(0.0026, 0.0008, slot=False).rot(x=math.pi / 2).move(x=0.022, z=-0.0008))


def backpack_body():
    """Soft pack: pillowy superellipsoid, front bulging, top slumped, a
    zip track arching over the front and a sagging, lumpy skin."""
    s = superquad(0.14, 0.18, 0.08, e_lat=0.35, e_lon=0.3, nlon=24, nlat=14)
    def slump(p):
        if p.z > 0:
            p.z *= 1 + 0.1 * max(0.0, 1 - (p.x / 0.14) ** 2) * max(0.0, 1 - (p.y / 0.18) ** 2)
        if p.y > 0.1:
            p.y -= 0.03 * (p.x / 0.14) ** 2 * (p.y - 0.1) / 0.08
        return p
    s = noisy(s.warp(slump), 0.003, scale=12, seed=2)
    zip_ = [(0.118 * math.cos(a), -0.03 + 0.17 * math.sin(a), 0.0) for a in
            (math.pi * i / 14 for i in range(15))]
    zp = []
    for x, y, _ in zip_:
        best = max((p for p in s.v if abs(p.x - x) < 0.02 and abs(p.y - y) < 0.02 and p.z > 0),
                   key=lambda p: p.z, default=Vector((x, y, 0.08)))
        zp.append((x, y, best.z - 0.001))
    return s + sweep(zp, circ_sec(0.0028, 5), up=(0, 0, 1))

part('Backpacks__body', f_box(0.28, 0.36, 0.16), backpack_body, smooth=70)
part('Backpacks__strap', f_bounds((-0.02, -0.175, 0), (0.02, 0.175, 0), 'plane'),
     lambda: bevel(slab(0.04, 0.35, 0.005, 3, 10), 0.0012, seg=1).warp(
         lambda p: Vector((p.x, p.y, p.z - 0.0025 - 0.0035 * max(0.0, 1 - (p.y / 0.175) ** 2)))), smooth=50)
part('Backpacks__handle', torus_arc_bounds(0.04, 0.006, math.pi),
     lambda: sweep(arc_pts(0.046 - 0.00175, 0.0, math.pi, 16), rr_sec(0.0035, 0.012, 0.0012, seg=1)))


def backpack_pocket():
    """Front pocket: padded pouch (behind the plane) under a flap with
    rounded lower corners."""
    pouch = superquad(0.078, 0.047, 0.0019, e_lat=0.3, e_lon=0.25, nlon=16, nlat=8).move(z=-0.0021)
    flap = prism(round_outline([(-0.08, 0.05), (0.08, 0.05), (0.08, 0.012), (0.0, 0.004), (-0.08, 0.012)],
                               0.012, seg=2), 0.0014, bevel_r=0.0005).move(z=-0.0007)
    return pouch + flap

part('Backpacks__pocket', f_bounds((-0.08, -0.05, 0), (0.08, 0.05, 0), 'plane'), backpack_pocket, smooth=50)


# ------------------------------------------------------------ hats --------
def hat_brim():
    """Felt brim: thin, dished, a bound edge rolled up, snapped a little
    down at the front and up at the back."""
    prof = [(0, -0.0012), (0.05, -0.0018), (0.085, -0.0032), (0.095, -0.004), (0.0995, -0.0018),
            (0.1, 0.0012), (0.0975, 0.0035), (0.0935, 0.0022), (0.075, 0.0002), (0.05, 0.0), (0, 0.0006)]
    s = lathe(prof, seg=30)
    return s.warp(lambda p: Vector((p.x, p.y - 0.0009 * (p.x / 0.1) * (math.hypot(p.x, p.z) / 0.1), p.z)))


def hat_crown():
    """Fedora crown: slight taper, rounded top edge, a lengthwise crease
    in the top and two pinches at the front."""
    prof = [(0, -0.04), (0.055, -0.04), (0.0545, -0.01), (0.0525, 0.018), (0.050, 0.030),
            (0.046, 0.0365), (0.038, 0.0398), (0.024, 0.0405), (0.011, 0.0405), (0, 0.040)]
    s = lathe(prof, seg=28)
    def shape(p):
        r = math.hypot(p.x, p.z)
        if p.y > 0.02:
            k = (p.y - 0.02) / 0.02
            p.y -= 0.013 * k * math.exp(-(p.z / 0.013) ** 2) * max(0.0, 1 - (p.x / 0.052) ** 2)
            a = math.atan2(p.z, p.x)
            pinch = math.exp(-((abs(a) - 0.6) / 0.28) ** 2) * k
            p.x *= 1 - 0.14 * pinch
            p.z *= 1 - 0.14 * pinch
        return p
    return s.warp(shape)


def hat_band():
    """Grosgrain band with a flat bow knot at the side."""
    s = ring_section(0.055, [(-0.0038, -0.005), (0.0045, -0.005), (0.005, -0.0042), (0.005, 0.0042),
                             (0.0045, 0.005), (-0.0038, 0.005)], seg=24)
    knot = rbox(0.004, 0.0075, 0.0095, r=0.0012, seg=1).move(x=-0.0575)
    loops = sum((ellipsoid(0.0022, 0.009, 0.0045, seg=8).rot(z=sy * 0.35).move(x=-0.0572, y=sy * 0.0085, z=-0.0005)
                 for sy in (-1, 1)), Shape())
    return s + knot + loops

part('SmallHats__brim', f_cyl(0.10, 0.10, 0.008), hat_brim, smooth=50)
part('SmallHats__crown', f_cyl(0.055, 0.055, 0.08), hat_crown, smooth=60)
part('SmallHats__band', f_torus(0.055, 0.005), hat_band)


# ------------------------------------------------------------ jewelry box -
def jewel_body():
    """Box body: moulded plinth, walls with rounded top edges and an open
    top recess the felt insert sits in."""
    plinth = rbox(0.10, 0.007, 0.07, r=0.002, seg=1).move(y=-0.0165)
    walls = rbox(0.097, 0.034, 0.067, r=0.0025, seg=2).move(y=0.003)
    walls = cut(walls, rbox(0.086, 0.03, 0.056, r=0.002, seg=1).move(y=0.0205))
    esc = rbox(0.008, 0.01, 0.002, r=0.0008, seg=1).move(y=0.008, z=0.0342)
    return plinth + walls + esc


def felt_insert():
    """Felt ring-roll insert: four padded rolls along the box (plane faces
    +Z = up), with slits between them."""
    s = box(0.09, 0.06, 0.0008).move(z=-0.0017)
    for k in range(4):
        y = -0.0225 + 0.015 * k
        s += cyl(0.0071, 0.0071, 0.089, seg=10, bevel_r=0).rot(z=math.pi / 2).scale(1, 1, 0.15).move(y=y, z=-0.00107)
    return s

part('SmallJewelryBoxes__body', f_box(0.10, 0.04, 0.07), jewel_body)
part('SmallJewelryBoxes__felt', f_bounds((-0.045, -0.03, 0), (0.045, 0.03, 0), 'plane'), felt_insert, smooth=60)
part('SmallJewelryBoxes__lidOpen', f_box(0.10, 0.03, 0.07),
     lambda: cut(rbox(0.10, 0.03, 0.07, r=0.004, seg=2), rbox(0.088, 0.03, 0.058, r=0.002, seg=1).move(y=-0.009)))
part('SmallJewelryBoxes__lid', f_box(0.10, 0.014, 0.07),
     lambda: rbox(0.10, 0.009, 0.07, r=0.0025, seg=2).move(y=-0.0025)
     + rbox(0.084, 0.006, 0.054, r=0.002, seg=2).move(y=0.004))
part('SmallJewelryBoxes__hinge', f_cyl(0.003, 0.003, 0.02),
     lambda: lathe([(0, -0.01), (0.0026, -0.01), (0.003, -0.0092), (0.003, -0.0038), (0.0022, -0.0034),
                    (0.0022, 0.0034), (0.003, 0.0038), (0.003, 0.0092), (0.0026, 0.01), (0, 0.01)], seg=6))


# ------------------------------------------------------------ magnifier ---
part('SmallMagnifyingGlasses__rim', f_torus(0.040, 0.005),
     lambda: ring_section(0.040, [(-0.0032, -0.0036), (0.0032, -0.005), (0.005, -0.0034), (0.005, -0.0012),
                                  (0.0041, 0.0), (0.005, 0.0012), (0.005, 0.0034), (0.0032, 0.005),
                                  (-0.0032, 0.0036), (-0.0036, 0.0)], seg=26))
part('SmallMagnifyingGlasses__handle', f_cyl(0.008, 0.010, 0.10),
     lambda: lathe([(0, -0.05), (0.0082, -0.05), (0.0097, -0.047), (0.010, -0.04), (0.0091, -0.028),
                    (0.0095, -0.012), (0.0086, 0.01), (0.0074, 0.03), (0.0070, 0.036), (0.0080, 0.038),
                    (0.0080, 0.046), (0.0064, 0.0485), (0.0040, 0.05), (0, 0.05)], seg=10))
part('SmallMagnifyingGlasses__knob', f_sphere(0.012),
     lambda: lathe([(0, -0.012), (0.0078, -0.012), (0.0072, -0.0092), (0.0102, -0.0052), (0.012, 0.0),
                    (0.0112, 0.0048), (0.0082, 0.0088), (0.0042, 0.0114), (0, 0.012)], seg=12).rot(z=-math.pi / 2))


# ------------------------------------------------------------ coat hooks --
def hook_board():
    """Wall board: chamfered front edges, eased ends, two screws."""
    sec = round_outline([(-0.007, -0.025), (0.002, -0.025), (0.007, -0.02), (0.007, 0.02),
                         (0.002, 0.025), (-0.007, 0.025)], 0.0012, seg=2)
    s = x_extrude(sec, 0.30, bevel_r=0.0015)
    for sx in (-0.135, 0.135):
        s += screw_head(0.0032, 0.0011).rot(x=math.pi / 2).move(x=sx, z=0.0067)
    return s

part('CoatHooks__board', f_box(0.30, 0.05, 0.014), hook_board)
part('CoatHooks__post', f_cyl(0.004, 0.004, 0.030),
     lambda: lathe([(0, -0.015), (0.004, -0.015), (0.004, -0.0125), (0.0026, -0.0105), (0.0024, 0.011),
                    (0.0030, 0.0135), (0.0022, 0.015), (0, 0.015)], seg=8))
part('CoatHooks__tip', f_cyl(0.003, 0.003, 0.014),
     lambda: lathe([(0, -0.007), (0.0021, -0.007), (0.0021, 0.0015), (0.003, 0.0035), (0.003, 0.0055),
                    (0.0019, 0.007), (0, 0.007)], seg=8), smooth=60)


def outline_rings(w, h, r, rings, seg=3):
    """Like rect_rings but every ring is a rounded rectangle (radius r - inset)."""
    v, f = [], []
    cnt = None
    for u, z in rings:
        ol = rounded_rect(w - 2 * u, h - 2 * u, max(r - u, 0.0004), seg)
        cnt = len(ol)
        v += [(x, y, z) for x, y in ol]
    for k in range(len(rings) - 1):
        for c in range(cnt):
            c2 = (c + 1) % cnt
            f.append((k * cnt + c, k * cnt + c2, (k + 1) * cnt + c2, (k + 1) * cnt + c))
    n = len(rings) - 1
    f.append(tuple(n * cnt + c for c in range(cnt)))
    f.append(tuple(reversed(range(cnt))))
    return _fw(Shape(v, f))


# ------------------------------------------------------------ small debris -
part('DeadMoths__body', f_cyl(0.002, 0.001, 0.014),
     lambda: lathe([(0, -0.007), (0.0009, -0.0062), (0.0007, -0.0048), (0.0018, -0.0025), (0.0015, 0.0),
                    (0.002, 0.002), (0.0014, 0.0052), (0, 0.007)], seg=6), smooth=180)


def vase_stub():
    """Bottom of a smashed vase: foot ring, bellied wall, a jagged broken
    top edge (teeth reach the full height)."""
    prof = [(0.0, -0.01), (0.031, -0.01), (0.0345, -0.0072), (0.035, -0.001), (0.0322, 0.01)]
    wall = 0.0034
    inner = [(0.0288, 0.01), (0.0312, -0.001), (0.0296, -0.0062), (0.0, -0.0062)]
    s = lathe(prof + [(0.0305, 0.0105)] + inner, seg=14, cap_bottom=False, cap_top=False)
    teeth = [0.0, 0.006, 0.002, 0.0075, 0.0005, 0.005, 0.0015, 0.0068, 0.0, 0.0045, 0.0025, 0.007, 0.001, 0.0055]
    def jag(p):
        if p.y > 0.004:
            a = (math.atan2(p.x, p.z) % TAU) / TAU * len(teeth)
            i = int(a) % len(teeth)
            fr = a - int(a)
            d = teeth[i] * (1 - fr) + teeth[(i + 1) % len(teeth)] * fr
            p.y = 0.004 + (p.y - 0.004) * (0.006 - d) / 0.006 if d < 0.006 else 0.004 + (p.y - 0.004) * 0.1
        return p
    return s.warp(jag)

part('ShatteredVases__base', f_cyl(0.030, 0.035, 0.020), vase_stub, smooth=45)
part('ShatteredVases__shard', f_box(0.010, 0.019, 0.004),
     lambda: bend(prism([(-0.005, -0.0095), (0.005, -0.0095), (0.005, 0.0015), (0.0028, 0.0095), (0.0008, 0.0035),
                         (-0.0018, 0.0068), (-0.005, -0.0008)], 0.0036, bevel_r=0.0005), 0.3, 'y', 'x')
     .fit((-0.005, -0.0095, -0.002), (0.005, 0.0095, 0.002)))


# ------------------------------------------------------------ CRT TV ------
def crt_case():
    """CRT television: full-size front shell (+Z) with a speaker grille
    in the lower margin, a tapered rear housing over the tube with vent
    slots, a seam between them, four little feet."""
    W, H, D = 0.28, 0.22, 0.24
    front = rbox(W, H - 0.006, 0.10, r=0.009, seg=2).move(y=0.003, z=D / 2 - 0.05)
    slots = [box(0.0028, 0.013, 0.012).move(x=0.05 + 0.009 * i, y=-0.095, z=D / 2) for i in range(7)]
    front = cut(front, *slots)
    rear = rbox(W - 0.018, H - 0.024, 0.15, r=0.012, seg=2).move(y=-0.004, z=-D / 2 + 0.075)
    zf, zb = 0.02, -D / 2
    def taper_(p):
        t = max(0.0, min(1.0, (zf - p.z) / (zf - zb)))
        p.x *= 1 - 0.32 * t
        p.y = -0.109 + (p.y + 0.109) * (1 - 0.34 * t)
        return p
    rear = rear.warp(taper_)
    vents = []
    for i in range(5):
        z = -0.03 - 0.018 * i
        t = (zf - z) / (zf - zb)
        top = -0.109 + (0.099 + 0.109) * (1 - 0.34 * t)
        vents.append(box(0.10, 0.012, 0.005).rot(x=-0.45).move(y=top, z=z))
    rear = cut(rear, *vents)
    seam = rbox(W - 0.012, H - 0.018, 0.006, r=0.006, seg=1).move(y=0.0, z=0.0205)
    feet = sum((rbox(0.03, 0.006, 0.026, r=0.002, seg=1).move(x=sx * (0.105 if sz > 0 else 0.066), y=-0.107, z=sz * 0.08)
                for sx in (-1, 1) for sz in (-1, 1)), Shape())
    return front + rear + seam + feet

part('OldCRTTVs__body', f_box(0.28, 0.22, 0.24), crt_case)
part('OldCRTTVs__bezel', f_box(0.238, 0.165, 0.005),
     lambda: outline_rings(0.238, 0.165, 0.014, [(0, -0.0025), (0, 0.0012), (0.0018, 0.0025), (0.0075, 0.0025),
                                                 (0.0135, 0.0002), (0.0155, -0.0012)], seg=3))


def crt_knob():
    """Knurled dial; its face (with a pointer ridge) is -Y, the case side +Y."""
    s = lathe([(0, -0.0026), (0.0068, -0.0026), (0.008, -0.0016), (0.008, 0.003), (0, 0.003)], seg=12)
    def knurl(p):
        r = math.hypot(p.x, p.z)
        if r > 0.0075:
            i = round(math.atan2(p.x, p.z) / (TAU / 12))
            if i % 2:
                p.x *= 0.9
                p.z *= 0.9
        return p
    return s.warp(knurl) + box(0.0012, 0.0006, 0.0055).move(y=-0.0029, z=0.0032)

part('OldCRTTVs__knob', f_cyl(0.008, 0.008, 0.006), crt_knob, smooth=40)
part('OldCRTTVs__antenna', f_cyl(0.002, 0.002, 0.18),
     lambda: lathe([(0, -0.09), (0.0018, -0.09), (0.002, -0.088), (0.002, -0.03), (0.0015, -0.028),
                    (0.0015, 0.03), (0.0011, 0.032), (0.0011, 0.084), (0.0019, 0.0855), (0.002, 0.088),
                    (0.0014, 0.0898), (0, 0.09)], seg=8))


# ------------------------------------------------------------ ashtray -----
def ashtray():
    """Heavy glass-style ashtray: thick sloped wall, flat rim with three
    cigarette rests, a shallow bowl."""
    prof = [(0, -0.0075), (0.038, -0.0075), (0.040, -0.0066), (0.046, -0.0015), (0.0495, 0.0055),
            (0.0500, 0.0068), (0.0486, 0.0075), (0.0432, 0.0075), (0.0412, 0.0062), (0.0385, 0.0035),
            (0.034, 0.0022), (0, 0.0022)]
    s = lathe(prof, seg=18)
    rests = [cyl(0.0035, 0.0035, 0.03, seg=8, bevel_r=0).rot(z=math.pi / 2).move(x=0.045, y=0.0082).rot(y=k * TAU / 3)
             for k in range(3)]
    return cut(s, *rests)

part('OverflowingAshtrays__tray', f_cyl(0.050, 0.040, 0.015), ashtray)
part('OverflowingAshtrays__ash', f_cone(0.030, 0.015),
     lambda: noisy(lathe([(0, -0.0075), (0.030, -0.0075), (0.026, -0.0052), (0.020, -0.0016), (0.013, 0.0022),
                          (0.007, 0.0055), (0.002, 0.0075), (0, 0.0075)], seg=16), 0.0012, scale=160, seed=4),
     smooth=180)
part('OverflowingAshtrays__paper', f_cyl(0.002, 0.002, 0.014),
     lambda: sweep([(0, -0.007, 0), (0.0002, -0.002, 0), (0.0005, 0.003, 0.0002), (0.0002, 0.0068, 0.0004)],
                   circ_sec(0.0019, 6), scale=lambda t: 1.0 if t < 0.8 else (1.0 - 0.5 * (t - 0.8) / 0.2, 0.85)),
     smooth=60)
part('OverflowingAshtrays__filter', f_cyl(0.002, 0.002, 0.006),
     lambda: lathe([(0, -0.003), (0.0015, -0.003), (0.002, -0.0023), (0.002, 0.003), (0, 0.003)], seg=6),
     smooth=60)


# ------------------------------------------------------------ open case ---
def open_case_half():
    """Opened suitcase half (open side +Y): leather shell with rounded
    corners, a lining recessed 3 mm inside the rim, piping round the rim."""
    s = rbox(0.22, 0.05, 0.14, r=0.008, seg=2)
    s = cut(s, rbox(0.206, 0.02, 0.126, r=0.005, seg=1).move(y=0.032))
    pipe = outline_tube(rounded_rect(0.216, 0.136, 0.008, seg=2), circ_sec(0.0022, 5)).rot(x=-math.pi / 2).move(y=0.0228)
    return s + pipe

part('OpenedSuitcases__base', f_box(0.22, 0.05, 0.14), open_case_half)
part('OpenedSuitcases__lid', f_box(0.22, 0.05, 0.14), lambda: open_case_half().rot(x=math.pi))
part('OpenedSuitcases__stud', f_cyl(0.008, 0.008, 0.008),
     lambda: lathe([(0, -0.004), (0.0078, -0.004), (0.008, -0.0022), (0.0062, 0.0012), (0.0034, 0.0034),
                    (0, 0.004)], seg=10), smooth=50)


def folded_cloth():
    """A folded garment: soft slab, one rolled fold edge, a crease across
    the top and rumpled noise."""
    s = superquad(0.035, 0.0055, 0.0225, e_lat=0.25, e_lon=0.25, nlon=16, nlat=8)
    def fold(p):
        if p.y > 0:
            p.y -= 0.0022 * math.exp(-((p.x - 0.01) / 0.004) ** 2)
        if p.x < -0.025:
            p.y += 0.0012 * (1 - abs(p.y) / 0.0055)
        return p
    return noisy(s.warp(fold), 0.0012, scale=60, seed=7)

part('OpenedSuitcases__cloth', f_box(0.07, 0.011, 0.045), folded_cloth, smooth=180)


# ------------------------------------------------------------ picture frame
def frame_bar(w=0.268, h=0.024, t=0.015):
    """Frame moulding bar. One constructor makes both the long top/bottom
    bars and the tall side bars, so the profile is symmetric in X and Y:
    a crowned face (+Z) that rolls off towards every edge, which reads as
    a rounded moulding whichever way the bar is stretched."""
    us = [-1, -0.975, -0.92, -0.8, -0.55, -0.2, 0.2, 0.55, 0.8, 0.92, 0.975, 1]
    n = len(us)
    f_ = lambda u: max(0.0, 1 - abs(u) ** 5) ** 0.35
    v, f = [], []
    for j, vv in enumerate(us):
        for i, uu in enumerate(us):
            z = -t / 2 + 0.55 * t + 0.45 * t * f_(uu) * f_(vv)
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

part('BrokenPictureFrames__bar', f_box(0.268, 0.024, 0.015), frame_bar, smooth=40)


# ------------------------------------------------------------ vinyl -------
part('VinylRecordDiscs__disc', f_cyl(0.078, 0.078, 0.002),
     lambda: lathe([(0, -0.001), (0.0775, -0.001), (0.078, -0.0006), (0.078, 0.0006), (0.0775, 0.001),
                    (0.0745, 0.0007), (0.061, 0.0007), (0.0602, 0.00052), (0.0594, 0.0007), (0.046, 0.0007),
                    (0.0452, 0.00052), (0.0444, 0.0007), (0.031, 0.0007), (0.0275, 0.0006), (0.0262, 0.001),
                    (0, 0.001)], seg=20), smooth=25)


# ------------------------------------------------------------ mouse trap --
part('MouseTraps__base', f_box(0.06, 0.006, 0.038),
     lambda: cut(rbox(0.06, 0.006, 0.038, r=0.0012, seg=1),
                 box(0.064, 0.0012, 0.0008).move(y=0.003, z=-0.008), box(0.064, 0.0012, 0.0008).move(y=0.003, z=0.009)))
part('MouseTraps__snap', f_torus(0.016, 0.001),
     lambda: outline_tube(rounded_rect(0.032, 0.032, 0.006, seg=2), circ_sec(0.001, 5)))
part('MouseTraps__bar', f_cyl(0.001, 0.001, 0.036),
     lambda: lathe([(0, -0.018), (0.0007, -0.0178), (0.001, -0.0172), (0.001, 0.0172), (0.0007, 0.0178),
                    (0, 0.018)], seg=6))
part('MouseTraps__trigger', f_box(0.006, 0.002, 0.014),
     lambda: prism(round_outline([(-0.003, -0.007), (0.003, -0.007), (0.003, 0.004), (0.0015, 0.007),
                                  (-0.0015, 0.007), (-0.003, 0.004)], 0.001, seg=1), 0.002)
     .rot(x=-math.pi / 2))
part('MouseTraps__cheese', f_box(0.006, 0.005, 0.005),
     lambda: prism([(-0.003, -0.0025), (0.003, -0.0025), (0.003, 0.0025), (0.0012, 0.0025), (-0.003, -0.0009)],
                   0.005, bevel_r=0.0003))


# ------------------------------------------------------------ eyeglasses --
part('ShatteredEyeglasses__rim', f_torus(0.020, 0.002),
     lambda: ring_section(0.020, [(-0.0014, -0.0016), (0.0008, -0.002), (0.002, -0.0009), (0.002, 0.0009),
                                  (0.0008, 0.002), (-0.0014, 0.0016)], seg=22))
part('ShatteredEyeglasses__bridge', f_cyl(0.001, 0.001, 0.010),
     lambda: lathe([(0, -0.005), (0.0009, -0.0048), (0.001, -0.0038), (0.0007, 0.0), (0.001, 0.0038),
                    (0.0009, 0.0048), (0, 0.005)], seg=6))
part('ShatteredEyeglasses__temple', f_cyl(0.001, 0.001, 0.10),
     lambda: lathe([(0, -0.05), (0.0008, -0.0495), (0.001, -0.046), (0.0009, -0.036), (0.0007, -0.03),
                    (0.0007, 0.044), (0.001, 0.046), (0.001, 0.0495), (0, 0.05)], seg=6))
part('ShatteredEyeglasses__broken', f_cyl(0.001, 0.001, 0.06),
     lambda: lathe([(0, -0.03), (0.0008, -0.0295), (0.001, -0.026), (0.0009, -0.018), (0.0007, -0.012),
                    (0.0007, 0.028), (0.0005, 0.03)], seg=6, cap_top=True)
     .warp(lambda p: Vector((p.x, p.y + (0.0012 * p.x / 0.0007 if p.y > 0.027 else 0), p.z)))
     .fit((-0.001, -0.03, -0.001), (0.001, 0.03, 0.001)))


# ------------------------------------------------------------ wristwatch --
part('ForgottenWristwatches__case', f_cyl(0.020, 0.020, 0.006),
     lambda: lathe([(0, -0.003), (0.0172, -0.003), (0.0196, -0.0017), (0.020, 0.0), (0.0194, 0.0024),
                    (0.0186, 0.003), (0.0178, 0.0024), (0, 0.0024)], seg=18))


def watch_strap(holes):
    """Leather strap curling off the case: flat section, square-ish edges,
    punched holes (one side) or a keeper loop near the case (other side)."""
    R = 0.035 - 0.0014
    arc = math.pi * 0.7
    s = sweep(arc_pts(R, 0.0, arc, 11), rr_sec(0.0028, 0.010, 0.0009, seg=1),
              scale=lambda t: (1.0, 1.0 if t < 0.85 else 1.0 - 0.3 * (t - 0.85) / 0.15))
    if holes:
        cutters = []
        for u in (0.52, 0.6, 0.68):
            a = arc * u
            cutters.append(cyl(0.0011, 0.0011, 0.01, seg=6, bevel_r=0).rot(z=a - math.pi / 2)
                           .move(x=R * math.cos(a), y=R * math.sin(a)))
        s = cut(s, *cutters)
    else:
        a = arc * 0.16
        s += rbox(0.0046, 0.004, 0.0108, r=0.0012, seg=1).rot(z=a).move(x=R * math.cos(a), y=R * math.sin(a))
    return s.fit(*torus_arc_bounds(0.030, 0.005, arc)[1:])

part('ForgottenWristwatches__strapHoles', torus_arc_bounds(0.030, 0.005, math.pi * 0.7), lambda: watch_strap(True))
part('ForgottenWristwatches__strapKeeper', torus_arc_bounds(0.030, 0.005, math.pi * 0.7), lambda: watch_strap(False))


# ------------------------------------------------------------ cup & saucer -
part('PorcelainCupSaucers__saucer', f_cyl(0.045, 0.045, 0.004),
     lambda: lathe([(0, -0.0013), (0.02, -0.0013), (0.022, -0.002), (0.025, -0.002), (0.027, -0.0012),
                    (0.044, 0.0011), (0.045, 0.002), (0.0432, 0.002), (0.028, -0.0002), (0.0255, -0.0005),
                    (0, -0.0005)], seg=24), smooth=40)
part('PorcelainCupSaucers__saucerRim', f_torus(0.043, 0.002), lambda: rim_ring(0.043, 0.002, seg=40, n=6))
part('PorcelainCupSaucers__cup', f_cyl(0.024, 0.020, 0.030),
     lambda: lathe([(0, -0.015), (0.0188, -0.015), (0.0212, -0.0115), (0.0232, -0.002), (0.024, 0.0142),
                    (0.0229, 0.015), (0.0218, 0.0138), (0.021, -0.003), (0.0175, -0.011), (0, -0.0115)], seg=16),
     smooth=45)
part('PorcelainCupSaucers__handle', torus_arc_bounds(0.010, 0.002, math.pi * 1.4),
     lambda: sweep(arc_pts(0.010, 0.0, math.pi * 1.4, 16), circ_sec(0.002, 6, sx=0.7),
                   scale=lambda t: 1.0 - 0.25 * t).fit(*torus_arc_bounds(0.010, 0.002, math.pi * 1.4)[1:]),
     smooth=60)


# ------------------------------------------------------------ alarm bell --
def bell_plate():
    """Wooden back plate: rounded corners, ogee edge, four screws."""
    s = outline_rings(0.12, 0.16, 0.012, [(0, -0.007), (0, -0.0022), (0.0018, 0.0016), (0.0045, 0.0038),
                                          (0.0085, 0.0048)], seg=3)
    for sx in (-1, 1):
        for sy in (-1, 1):
            s += screw_head(0.0036, 0.0021).rot(x=math.pi / 2).move(x=sx * 0.044, y=sy * 0.064, z=0.0048)
    return s

part('AlarmBells__plate', f_box(0.12, 0.16, 0.014), bell_plate)
part('AlarmBells__gong', f_bounds((-0.055, 0, -0.055), (0.055, 0.055, 0.055)),
     lambda: revolve_loop([(0.052, 0.0), (0.055, 0.0), (0.0552, 0.003), (0.0535, 0.0065), (0.0508, 0.0155),
                           (0.0515, 0.019), (0.047, 0.0265), (0.036, 0.041), (0.02, 0.051),
                           (0.0006, 0.055), (0.0006, 0.052), (0.02, 0.048), (0.035, 0.038), (0.0475, 0.021),
                           (0.0512, 0.006)], seg=18).fit((-0.055, 0, -0.055), (0.055, 0.055, 0.055)))


def hex_nut():
    """Hex nut with chamfered top (a 6-sided lathe turned 15 deg so it
    spans the frame equally in X and Z) and the bolt end poking out."""
    rr = 0.008 / math.cos(math.radians(15))
    nut = lathe([(0, -0.004), (rr, -0.004), (rr, 0.0004), (rr * 0.84, 0.0015), (0, 0.0015)], seg=6).rot(y=math.radians(15))
    bolt = lathe([(0, 0.0012), (0.0038, 0.0012), (0.0036, 0.0032), (0.002, 0.004), (0, 0.004)], seg=10)
    return (nut + bolt).fit((-0.008, -0.004, -0.008), (0.008, 0.004, 0.008))


part('AlarmBells__nut', f_cyl(0.008, 0.008, 0.008), hex_nut)
part('AlarmBells__arm', f_cyl(0.001, 0.001, 0.04),
     lambda: lathe([(0, -0.02), (0.0007, -0.0198), (0.001, -0.0192), (0.001, 0.0192), (0.0007, 0.0198),
                    (0, 0.02)], seg=6))
part('AlarmBells__ball', f_sphere(0.006),
     lambda: lathe([(0, -0.006), (0.0042, -0.0042), (0.006, 0.0), (0.0045, 0.0038), (0.002, 0.0052),
                    (0.0018, 0.006), (0, 0.006)], seg=8).fit((-0.006, -0.006, -0.006), (0.006, 0.006, 0.006)),
     smooth=60)


# ------------------------------------------------------------ ring --------
part('WeddingRings__band', f_torus(0.010, 0.002),
     lambda: ring_section(0.010, [(-0.0016, -0.0017), (-0.0002, -0.002), (0.0014, -0.0014), (0.002, 0.0),
                                  (0.0014, 0.0014), (-0.0002, 0.002), (-0.0016, 0.0017)], seg=20), smooth=50)
