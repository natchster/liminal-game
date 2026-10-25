"""Batch b11: the "domestic" theme. Shoes, gloves, laundry, ironing, cleaning,
travel things, small personal items and the bigger household objects.
Decals, transparent and sub-millimetre ink primitives are left alone (see
b11.notes.md). Every other solid primitive gets a modelled part here.
"""
import propkit
from propkit import *
from mathutils import Vector
import bmesh

fixw = propkit._fix_winding
HPI = math.pi / 2


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
    return lambda i: rounded_rect(w - 2 * i, h - 2 * i, max(r - i, 0.0002), seg)


def outl(pts):
    base = _ccw(pts)
    return lambda i: base if abs(i) < 1e-9 else offset2d(base, i)


def stack(outline, prof, loop=False):
    """Moulding: for each (inset, z) in prof the outline, inset, at depth z;
    consecutive rings are bridged. Ends are capped, or with loop=True the last
    ring joins the first (a ring / bezel with a hole)."""
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


def ystack(outline_xz, prof, loop=False):
    """stack() lying flat: the outline is given in (x, z), prof heights are y."""
    def o(i):
        return [(x, -z) for x, z in outline_xz(i)]
    return stack(o, prof, loop).rot(x=-HPI)


def xz_outl(pts):
    """outline generator in (x, z) for ystack()."""
    base = _ccw(pts)
    return lambda i: base if abs(i) < 1e-9 else offset2d(base, i)


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


def interp(tab, x):
    """Piecewise-linear lookup in [(x, y), ...]."""
    if x <= tab[0][0]:
        return tab[0][1]
    for (x0, y0), (x1, y1) in zip(tab, tab[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * (x - x0) / ((x1 - x0) or 1)
    return tab[-1][1]


def ellipse(a, b, n=20):
    return [(a * math.cos(TAU * i / n), b * math.sin(TAU * i / n)) for i in range(n)]


def dring(z, hw, yt, yb, m=12, p=0.45):
    """A D-shaped section at depth z: flat bottom at yb, sides rising and
    rounding over to the top yt, half-width hw."""
    return [(hw * math.cos(math.pi * k / (m - 1)),
             yb + (yt - yb) * abs(math.sin(math.pi * k / (m - 1))) ** p, z) for k in range(m)]


def fitb(s, w, h, d):
    return s.fit((-w / 2, -h / 2, -d / 2), (w / 2, h / 2, d / 2))


def ring_lathe(profile, seg=24):
    """Revolve a CLOSED profile loop [(r, y), ...] around +Y (rings, rims)."""
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


def dtorus(R, r, seg=32, prof=None):
    """Ring in the XY plane (torus frame) with a custom closed cross-section
    prof [(dr, dz), ...] around the tube centre (default a circle)."""
    prof = prof or [(r * math.cos(TAU * k / 8), r * math.sin(TAU * k / 8)) for k in range(8)]
    s = ring_lathe([(R + dr, dz) for dr, dz in prof], seg)
    return s.rot(x=HPI)


# ------------------------------------------------------------------ shoes --
# a sneaker built from sole, mid upper, toe box, heel counter, dark collar
# opening and a tongue (toe at +Z)
def _sole_xz():
    tab = [(-0.12, 0.034), (-0.10, 0.040), (-0.06, 0.042), (-0.02, 0.041), (0.02, 0.044), (0.05, 0.047),
           (0.08, 0.046), (0.10, 0.042), (0.12, 0.034)]
    N = 13
    zs = [-0.12 + 0.24 * (1 - math.cos(math.pi * i / (N - 1))) / 2 for i in range(N)]
    env = lambda z: math.sqrt(max(0.0, 1 - (z / 0.12) ** 8))
    right = [(interp(tab, z) * env(z), z) for z in zs]
    left = [(-x, z) for x, z in reversed(right[1:-1])]
    return right + left


def shoe_sole():
    s = ystack(xz_outl(_sole_xz()),
               [(0.002, -0.007), (0, -0.0058), (0, -0.0022), (0.0009, -0.0015),
                (0, -0.0008), (0, 0.0064), (0.0008, 0.007)])
    grooves = [box(0.1, 0.004, 0.0026).move(y=-0.0074, z=z) for z in (0.024, 0.044, 0.064, 0.084, -0.085, -0.066)]
    s = cut(s, *grooves)
    def spring(p):                       # toe spring: the tip of the tread lifts
        if p.y < 0 and p.z > 0.07:
            p.y += 0.0035 * ((p.z - 0.07) / 0.05) ** 2
        return p
    return s.warp(spring)


def shoe_upper():
    zs = [-0.08, -0.06, -0.03, 0.0, 0.025, 0.045, 0.06, 0.072, 0.08]
    hw = [(-0.08, 0.041), (-0.05, 0.043), (0.08, 0.043)]
    yt = [(-0.08, 0.020), (0.02, 0.020), (0.045, 0.015), (0.06, 0.010), (0.072, 0.005), (0.08, 0.001)]
    return loft([dring(z, interp(hw, z), interp(yt, z), -0.02, m=14, p=0.35) for z in zs])


def shoe_toe():
    zs = [-0.04, -0.02, 0.0, 0.015, 0.026, 0.033, 0.037, 0.0395, 0.04]
    hw = [0.043, 0.043, 0.041, 0.037, 0.031, 0.024, 0.017, 0.010, 0.004]
    yt = [0.015, 0.013, 0.009, 0.005, 0.001, -0.003, -0.006, -0.008, -0.009]
    s = loft([dring(z, w, y, -0.015, m=14, p=0.45) for z, w, y in zip(zs, hw, yt)])
    return fitb(s, 0.086, 0.030, 0.08)


def shoe_heel():
    # heel counter: a U round the back, collar dipping to the front, pull tab
    zs = [-0.025, -0.0245, -0.023, -0.019, -0.012, -0.004, 0.006, 0.016, 0.025]
    hw = [0.012, 0.022, 0.030, 0.037, 0.041, 0.043, 0.043, 0.043, 0.043]
    yt = [(-0.025, 0.0225), (-0.02, 0.0215), (-0.01, 0.019), (0.025, 0.013)]
    s = loft([dring(z, w, interp(yt, z), -0.025, m=12, p=0.3) for z, w in zip(zs, hw)])
    tab = rbox(0.016, 0.012, 0.004, r=0.0015, seg=1).move(y=0.019, z=-0.023)
    return fitb(s + tab, 0.086, 0.05, 0.05)


def shoe_opening():
    # padded collar round a dark sunken footbed
    return ystack(xz_outl(ellipse(0.035, 0.05, 22)),
                  [(0, -0.005), (0, 0.001), (0.0015, 0.0038), (0.0042, 0.005), (0.0075, 0.0045),
                   (0.0100, 0.0022), (0.0115, -0.0015), (0.0122, -0.0035)])


def shoe_tongue():
    s = ystack(xz_outl(rounded_rect(0.048, 0.030, 0.009, seg=3)),
               [(0.0012, -0.0025), (0, -0.0015), (0, 0.0012), (0.0018, 0.0025)])
    return fitb(bend(s, 0.2, 'x', 'z'), 0.048, 0.005, 0.030)


part('ShoePairs__sole', f_box(0.09, 0.014, 0.24), shoe_sole)
part('ShoePairs__upper', f_box(0.086, 0.040, 0.16), shoe_upper, smooth=60)
part('ShoePairs__toe', f_box(0.086, 0.030, 0.08), shoe_toe, smooth=70)
part('ShoePairs__heel', f_box(0.086, 0.050, 0.05), shoe_heel, smooth=60)
part('ShoePairs__opening', f_box(0.070, 0.010, 0.100), shoe_opening, smooth=60)
part('ShoePairs__tongue', f_box(0.048, 0.005, 0.030), shoe_tongue, smooth=60)


# ----------------------------------------------------------------- gloves --
def glove_palm():
    # a soft stuffed back-of-hand: domed top, flatter underside, knuckles at +Z
    s = subdivide(box(0.07, 0.02, 0.07), 2)
    def puff(p):
        u, w = p.x / 0.035, p.z / 0.035
        p.y = p.y * (0.75 if p.y < 0 else 1.0) + 0.002 * (1 - u * u) * (1 - w * w)
        if p.y > 0 and p.z > 0.012:
            p.y += 0.0015 * max(0.0, math.cos(p.x / 0.0165 * math.pi * 2)) * (p.z - 0.012) / 0.023
        return p
    s.warp(puff)
    return fitb(noisy(s, 0.0006, scale=60, seed=3), 0.07, 0.02, 0.07)


def glove_digit(base=1.0):
    # a stuffed finger: soft tip at +Z, two knuckle creases
    prof = [(0.0, -0.5), (0.9 * base, -0.5), (0.97 * base, -0.42), (1.0, -0.2), (0.93, -0.1), (1.0, 0.0),
            (0.98, 0.18), (0.9, 0.22), (0.95, 0.3), (0.86, 0.4), (0.62, 0.46), (0.3, 0.495), (0.0, 0.5)]
    s = lathe([(r, y) for r, y in prof], seg=8).rot(x=HPI)
    return s


def glove_finger():
    return fitb(glove_digit(1.0).scale(1, 0.8, 1), 0.014, 0.014, 0.052)


def glove_thumb():
    return fitb(glove_digit(1.15).scale(1, 0.85, 1), 0.014, 0.014, 0.040)


def resample(pts, n):
    """A closed polyline resampled to n points evenly spaced along it."""
    P = [Vector((x, y)) for x, y in pts] + [Vector(pts[0])]
    L = [0.0]
    for a, b in zip(P, P[1:]):
        L.append(L[-1] + (b - a).length)
    out, k = [], 0
    for i in range(n):
        t = L[-1] * i / n
        while L[k + 1] < t:
            k += 1
        u = (t - L[k]) / ((L[k + 1] - L[k]) or 1)
        q = P[k].lerp(P[k + 1], u)
        out.append((q.x, q.y))
    return out


def glove_cuff():
    # knit rib cuff: ribs run along the arm (Z) over the top and sides, the
    # wrist opening sunk into the -Z end
    base = resample(_ccw(rounded_rect(0.076, 0.018, 0.0078, seg=6)), 52)
    pts = []
    n = len(base)
    for i, (x, y) in enumerate(base):
        a, b = Vector(base[i - 1]), Vector(base[(i + 1) % n])
        t = (b - a).normalized()
        nrm = Vector((t.y, -t.x))
        amp = 0.00055 * math.cos(math.pi * i / 2) if y > -0.005 else 0.0
        pts.append((x + nrm.x * amp, y + nrm.y * amp))
    s = prism(pts, 0.024)
    mouth = cyl(1, 1, 0.024, seg=14, bevel_r=0).scale(0.028, 1, 0.0052).rot(x=HPI).move(z=-0.015)
    return fitb(cut(s, mouth), 0.076, 0.018, 0.024)


part('DroppedGloves__palm', f_box(0.070, 0.020, 0.070), glove_palm, smooth=180)
part('DroppedGloves__finger', f_box(0.014, 0.014, 0.052), glove_finger, smooth=180)
part('DroppedGloves__thumb', f_box(0.014, 0.014, 0.040), glove_thumb, smooth=180)
part('DroppedGloves__cuff', f_box(0.076, 0.018, 0.024), glove_cuff, smooth=50)
part('DroppedGloves__trim', f_box(0.078, 0.004, 0.008), lambda: rbox(0.078, 0.004, 0.008, r=0.0015, seg=2), smooth=60)


# ----------------------------------------------------------- soft goods ---
def soft_fold(w, h, d, seed=0, nx=8, nz=6, fold=-1, pinch=0.55, sag=0.0012, crinkle=0.0008):
    """A folded garment / towel lying flat (w along X, h thick, d along Z):
    the fold side (x = fold * w/2) stays round and full, the open edges pinch
    soft, the middle sags a little, a light crinkle over everything."""
    s = slab(w, d, h, nx, nz).rot(x=-HPI)
    def soft(p):
        u, v = p.x / (w / 2), p.z / (d / 2)
        e_open = max(abs(v), u * -fold if u * -fold > 0 else 0.0)
        k = max(0.0, (e_open - 0.72) / 0.28)
        p.y *= 1 - pinch * k * k
        if u * fold > 0.8:                                   # round the fold
            t = min(1.0, (u * fold - 0.8) / 0.2)
            p.y *= math.sqrt(max(0.0, 1 - t * t * 0.85))
        p.y -= sag * (1 - u * u) * (1 - v * v)
        return p
    s.warp(soft)
    s = noisy(s, crinkle, scale=45, seed=seed, axis=(0, 1, 0))
    return fitb(s, w, h, d)


# --------------------------------------------------------- laundry basket --
def lattice_wall(rt, rb, H, t, rows, cols=16, slot_frac=0.44, floor=0.005):
    """Round plastic basket wall with rows of vertical slots: `rows` is a
    list of (y0, y1, holes?) bottom to top. Outer and inner skins, jambs round
    every slot, a flat top edge and a floor. Axis +Y, centred."""
    ang = []
    for c in range(cols):
        a0 = TAU * c / cols
        ang += [a0, a0 + TAU / cols * (1 - slot_frac)]
    N = len(ang)
    ys = [rows[0][0]] + [r[1] for r in rows]
    rad = lambda y: rb + (rt - rb) * (y + H / 2) / H
    v, f = [], []
    O, I = [], []
    for li, y in enumerate(ys):
        yi = y if li else y + floor
        O.append([len(v) + j for j in range(N)])
        v += [(rad(y) * math.cos(a), y, rad(y) * math.sin(a)) for a in ang]
        I.append([len(v) + j for j in range(N)])
        v += [((rad(yi) - t) * math.cos(a), yi, (rad(yi) - t) * math.sin(a)) for a in ang]
    for ri, (_, _, holes) in enumerate(rows):
        for j in range(N):
            j2 = (j + 1) % N
            if holes and j % 2 == 1:
                f.append((O[ri][j], O[ri + 1][j], I[ri + 1][j], I[ri][j]))
                f.append((O[ri][j2], I[ri][j2], I[ri + 1][j2], O[ri + 1][j2]))
                f.append((O[ri][j], I[ri][j], I[ri][j2], O[ri][j2]))
                f.append((O[ri + 1][j], O[ri + 1][j2], I[ri + 1][j2], I[ri + 1][j]))
            else:
                f.append((O[ri][j], O[ri][j2], O[ri + 1][j2], O[ri + 1][j]))
                f.append((I[ri][j], I[ri + 1][j], I[ri + 1][j2], I[ri][j2]))
    top = len(ys) - 1
    for j in range(N):
        j2 = (j + 1) % N
        f.append((O[top][j], I[top][j], I[top][j2], O[top][j2]))
    cb, ci = len(v), len(v) + 1
    v += [(0, ys[0], 0), (0, ys[0] + floor, 0)]
    for j in range(N):
        j2 = (j + 1) % N
        f.append((cb, O[0][j2], O[0][j]))
        f.append((ci, I[0][j], I[0][j2]))
    return fixw(Shape(v, f))


def basket_body():
    rows = [(-0.08, -0.050, False), (-0.050, -0.010, True), (-0.010, 0.006, False),
            (0.006, 0.046, True), (0.046, 0.08, False)]
    return lattice_wall(0.14, 0.119, 0.16, 0.0045, rows, cols=16, slot_frac=0.42)


def basket_rim():
    prof = [(0.008 * math.copysign(abs(math.cos(TAU * k / 10)) ** 0.6, math.cos(TAU * k / 10)),
             0.008 * math.copysign(abs(math.sin(TAU * k / 10)) ** 0.6, math.sin(TAU * k / 10))) for k in range(10)]
    return dtorus(0.14, 0.008, seg=32, prof=prof)


def basket_handle():
    pts = []
    for k in range(13):
        u = math.pi * k / 12
        c, s_ = math.cos(u), math.sin(u)
        pts.append((0.030 * math.copysign(abs(c) ** 0.75, c), 0.030 * abs(s_) ** 0.6, 0))
    s = tube(pts, 0.006, seg=8)
    return s.fit((-0.036, 0, -0.006), (0.036, 0.036, 0.006))


def drape_top():
    # a cloth tongue over the rim: two soft ripples, the hanging edge rolled down
    s = slab(0.07, 0.06, 0.002, 7, 6).rot(x=-HPI)
    s.warp(lambda p: Vector((p.x, p.y + 0.0014 * math.sin(p.z / 0.06 * TAU * 1.2) - 0.0016 * max(0, p.x / 0.035) ** 2, p.z)))
    return fitb(noisy(s, 0.0004, scale=60, seed=4, axis=(0, 1, 0)), 0.07, 0.006, 0.06)


def drape_hang():
    # a hanging cloth: vertical folds, a wavy hem
    s = slab(0.06, 0.10, 0.0012, 7, 7)
    s.warp(lambda p: Vector((p.x, p.y + 0.003 * math.sin(p.x / 0.06 * TAU * 1.5) * max(0, -p.y / 0.05),
                             p.z + 0.0017 * math.sin(p.x / 0.06 * TAU * 1.5 + 0.6) * (0.4 + 0.6 * (0.05 - p.y) / 0.1))))
    return fitb(s, 0.06, 0.10, 0.005)


part('LaundryBaskets__body', f_cyl(0.14, 0.119, 0.16), basket_body)
part('LaundryBaskets__rim', f_torus(0.14, 0.008), basket_rim, smooth=60)
part('LaundryBaskets__handle', f_bounds((-0.036, 0, -0.006), (0.036, 0.036, 0.006)), basket_handle, smooth=60)
part('LaundryBaskets__cloth', f_box(0.194, 0.017, 0.179), lambda: soft_fold(0.194, 0.017, 0.179, seed=2), smooth=60)
part('LaundryBaskets__drapeTop', f_box(0.07, 0.006, 0.06), drape_top, smooth=70)
part('LaundryBaskets__drapeHang', f_box(0.06, 0.10, 0.005), drape_hang, smooth=70)


# ----------------------------------------------------------- ironing board --
def half_lathe(profile, seg=12):
    """Revolve [(r, y), ...] (all r > 0) over half a turn, x >= 0 (like a
    CylinderGeometry with thetaStart 0, thetaLength PI); closed flat side."""
    v, f, rings = [], [], []
    for r, y in profile:
        rings.append([len(v) + i for i in range(seg + 1)])
        v += [(r * math.sin(math.pi * i / seg), y, r * math.cos(math.pi * i / seg)) for i in range(seg + 1)]
    for a, b in zip(rings, rings[1:]):
        for i in range(seg):
            f.append((a[i], b[i], b[i + 1], a[i + 1]))
    f.append(tuple(rings[0]))
    f.append(tuple(rings[-1]))
    f.append(tuple([r[0] for r in rings] + [r[seg] for r in reversed(rings)]))
    return fixw(Shape(v, f))


PAD = [(0.003, -0.01), (0, -0.0062), (0.0012, -0.0036), (0, -0.0012), (0, 0.0035), (0.0035, 0.0082), (0.011, 0.01)]


def board_main():
    half = [(0.0, -0.0085), (0.058, -0.0085), (0.071, -0.010), (0.0775, -0.0088), (0.080, -0.0052),
            (0.0788, -0.0036), (0.080, -0.0022), (0.080, 0.003), (0.0778, 0.0068), (0.0742, 0.0088),
            (0.069, 0.0097), (0.05, 0.010)]
    pts = half + [(-x, y) for x, y in reversed(half[1:])]
    s = prism(pts, 0.32).rot(y=HPI)
    return fitb(bevel(s, 0.0025, seg=2, angle=50), 0.32, 0.02, 0.16)


def board_nose():
    xs = [0.075 * (1 - math.cos(HPI * i / 10)) for i in range(11)]
    zf = lambda x: 0.075 * max(0.0, 1 - (x / 0.075) ** 2) ** 0.62
    pts = [(x, -zf(x)) for x in xs[:-1]] + [(0.075, 0.0)] + [(x, zf(x)) for x in reversed(xs[:-1])]
    s = ystack(xz_outl(pts), PAD)
    return s.fit((0, -0.01, -0.075), (0.075, 0.01, 0.075))


def board_back():
    pts = [(-0.08 * math.sin(math.pi * i / 16), 0.08 * math.cos(math.pi * i / 16)) for i in range(17)]
    s = ystack(xz_outl(pts), PAD)
    return s.fit((-0.08, -0.01, -0.08), (0, 0.01, 0.08))


def steel_tube(r, h, seg=8, rivet=True):
    """A thin steel leg / rod: pressed flat at both ends, a rivet in each flat."""
    y = h / 2
    s = lathe([(0, -y), (r * 0.8, -y), (r, -y + 0.0015), (r, -y + 0.016), (r, -y + 0.03),
               (r, y - 0.03), (r, y - 0.016), (r, y - 0.0015), (r * 0.8, y), (0, y)], seg=seg)
    def press(p):
        d = abs(p.y) - (y - 0.03)
        if d > 0:
            k = min(1.0, d / 0.014)
            p.z *= 1 - 0.55 * k
            p.x *= 1 + 0.1 * k
        return p
    s.warp(press)
    if rivet:
        for sy in (-1, 1):
            s = s + cyl(r * 0.45, r * 0.45, r * 1.1, seg=6, bevel_r=r * 0.1).rot(x=HPI).move(y=sy * (y - 0.009))
    return s.fit((-r, -y, -r), (r, y, r))


def iron_outline(L, W, back_r=0.006):
    """Iron soleplate plan: square heel at -X, sides running into a point at +X."""
    n = 7
    side = [(-L / 2 + L * 0.45 + L * 0.55 * i / n, -W / 2 * max(0.0, 1 - (i / n) ** 1.9) ** 0.75) for i in range(n)]
    front = side + [(L / 2, 0.0)] + [(x, -z) for x, z in reversed(side)]
    c1 = propkit._round_corner((-L / 2, W / 2), front[-1], (-L / 2, -W / 2), back_r, 2)
    c0 = propkit._round_corner((-L / 2, -W / 2), (-L / 2, W / 2), front[0], back_r, 2)
    return c0 + front + c1


def iron_base():
    s = ystack(xz_outl(iron_outline(0.09, 0.06)),
               [(0.001, -0.005), (0, -0.0038), (0.0011, -0.0016), (0.0004, -0.0008), (0.0004, 0.0038), (0.0016, 0.005)])
    return fitb(s, 0.09, 0.010, 0.06)


def scaled_rings(pts, prof, cx=0.0, cz=0.0):
    """Loft of the (x, z) outline scaled about (cx, cz) by k at height y
    for (k, y) in prof: domes and tapers without offset fold-overs."""
    base = _ccw(pts)
    return loft([[(cx + (x - cx) * k, y, cz + (z - cz) * k) for x, z in base] for k, y in prof])


def iron_upper():
    s = scaled_rings(iron_outline(0.07, 0.05, 0.012),
                     [(1.0, -0.01), (1.0, -0.002), (0.965, 0.0035), (0.9, 0.0072), (0.78, 0.0093), (0.6, 0.01)], cx=-0.008)
    return fitb(s, 0.07, 0.020, 0.05)


def iron_nose():
    s = half_lathe([(0.0048, -0.015), (0.0064, -0.0142), (0.0305, 0.0128), (0.0317, 0.0143), (0.0302, 0.015)], seg=12)
    return s.fit((0, -0.015, -0.032), (0.032, 0.015, 0.032))


def iron_handle():
    pts = [(-0.024, 0.0, 0), (-0.024, 0.009, 0), (-0.021, 0.019, 0), (-0.012, 0.024, 0), (0.012, 0.024, 0),
           (0.02, 0.02, 0), (0.024, 0.01, 0), (0.024, 0.0, 0)]
    s = tube(pts, 0.006, seg=7, smooth=1)
    return s.fit((-0.03, 0, -0.006), (0.03, 0.03, 0.006))


def iron_dial():
    s = lathe([(0, -0.0015), (0.0078, -0.0015), (0.008, -0.0006), (0.0074, 0.0006), (0, 0.0006)], seg=10)
    s = s + box(0.013, 0.0012, 0.0025).move(y=0.0009)
    return s.fit((-0.008, -0.0015, -0.008), (0.008, 0.0015, 0.008))


def cord_boot():
    return lathe([(0, -0.005), (0.005, -0.005), (0.005, -0.003), (0.0044, -0.001), (0.0038, 0.002),
                  (0.0032, 0.005), (0, 0.005)], seg=8).fit((-0.005, -0.005, -0.005), (0.005, 0.005, 0.005))


def cord_seg():
    return lathe([(0, -0.055), (0.0028, -0.055), (0.003, -0.054), (0.003, 0.054), (0.0028, 0.055), (0, 0.055)], seg=8)


def rubber_foot():
    return lathe([(0, -0.003), (0.0095, -0.003), (0.01, -0.0021), (0.0093, 0.0005), (0.0078, 0.0024),
                  (0.0055, 0.003), (0, 0.003)], seg=8)


def folded_shirt():
    body = soft_fold(0.14, 0.0068, 0.10, seed=6, nx=6, nz=5, fold=1, pinch=0.3, crinkle=0.0004).move(y=-0.0016)
    band = [(0.0, -0.0006), (0.0026, -0.0006), (0.0034, 0.0022), (0.0012, 0.0032)]
    collar = ring_lathe([(0.017 + dr, y) for dr, y in band], seg=14).scale(0.8, 1, 1.15).move(x=-0.045, y=0.0022)
    placket = rbox(0.098, 0.0014, 0.010, r=0.0005, seg=1).move(x=0.022, y=0.0018)
    buttons = sum((cyl(0.0024, 0.0024, 0.0012, seg=6, bevel_r=0).move(x=bx, y=0.0028) for bx in (-0.012, 0.02, 0.052)), Shape())
    return fitb(body + collar + placket + buttons, 0.14, 0.010, 0.10)


part('IroningBoards__board', f_box(0.32, 0.020, 0.16), board_main, smooth=50)
part('IroningBoards__nose', f_bounds((0, -0.01, -0.075), (0.075, 0.01, 0.075)), board_nose, smooth=50)
part('IroningBoards__backEnd', f_bounds((-0.08, -0.01, -0.08), (0, 0.01, 0.08)), board_back, smooth=50)
part('IroningBoards__leg', f_cyl(0.006, 0.006, 0.867), lambda: steel_tube(0.006, 0.867))
part('IroningBoards__brace', f_cyl(0.004, 0.004, 0.30), lambda: steel_tube(0.004, 0.30))
part('IroningBoards__foot', f_cyl(0.008, 0.010, 0.006), rubber_foot, smooth=60)
part('IroningBoards__ironBase', f_box(0.09, 0.010, 0.06), iron_base)
part('IroningBoards__ironNose', f_bounds((0, -0.015, -0.032), (0.032, 0.015, 0.032)), iron_nose, smooth=50)
part('IroningBoards__ironUpper', f_box(0.07, 0.020, 0.05), iron_upper, smooth=50)
part('IroningBoards__ironHandle', f_bounds((-0.03, 0, -0.006), (0.03, 0.03, 0.006)), iron_handle, smooth=60)
part('IroningBoards__dial', f_cyl(0.008, 0.008, 0.003), iron_dial)
part('IroningBoards__cordBoot', f_cyl(0.005, 0.005, 0.010), cord_boot, smooth=50)
part('IroningBoards__cord', f_cyl(0.003, 0.003, 0.11), cord_seg, smooth=60)
part('IroningBoards__shirt', f_box(0.14, 0.010, 0.10), folded_shirt, smooth=60)


# --------------------------------------------------------- vacuum cleaner --
def _vac_section(z, hw, yt, yb=-0.0225, rt=0.009):
    c = math.cos(math.pi / 4)
    return [(hw - 0.004, yb, z), (hw, yb + 0.004, z), (hw, yt - rt, z), (hw - rt * (1 - c), yt - rt * (1 - c), z),
            (hw - rt, yt, z), (0.0, yt + 0.0008, z), (-hw + rt, yt, z), (-hw + rt * (1 - c), yt - rt * (1 - c), z),
            (-hw, yt - rt, z), (-hw, yb + 0.004, z), (-hw + 0.004, yb, z)]


def vac_base():
    # floor nozzle: rounded plan, hood sloping down to the front bumper, the
    # brush-roll mouth open across the front underside (the roller shows)
    r = 0.035
    zs = [-0.1, -0.098, -0.092, -0.082, -0.065, -0.03, 0.0, 0.03, 0.055, 0.065, 0.082, 0.092, 0.098, 0.1]
    def hw(z):
        d = abs(z) - (0.1 - r)
        return 0.16 if d <= 0 else 0.16 - r + math.sqrt(max(0.0, r * r - d * d))
    yt = lambda z: 0.0225 if z < -0.02 else 0.0225 - 0.0105 * ((z + 0.02) / 0.12) ** 1.4
    s = loft([_vac_section(z, hw(z), yt(z)) for z in zs])
    s = cut(s, box(0.296, 0.032, 0.075).move(y=-0.0145, z=0.098))
    return fitb(s, 0.32, 0.045, 0.20)


def vac_roller():
    # brush roll: a core with two chevron bristle strips and end bearings
    core = lathe([(0, -0.144), (0.0105, -0.144), (0.0115, -0.139), (0.0135, -0.137), (0.0135, 0.137),
                  (0.0115, 0.139), (0.0105, 0.144), (0, 0.144)], seg=12)
    strips = Shape()
    for k in range(2):
        pts = []
        for i in range(13):
            y = -0.13 + 0.26 * i / 12
            a = k * math.pi + 2.2 * abs(y) / 0.13
            pts.append((0.0163 * math.cos(a), y, 0.0163 * math.sin(a)))
        strips = strips + tube(pts, 0.0037, seg=5)
    return (core + strips).fit((-0.02, -0.144, -0.02), (0.02, 0.144, 0.02))


def vac_wheel():
    return lathe([(0, -0.0032), (0.0085, -0.0032), (0.0095, -0.0045), (0.019, -0.0045), (0.0228, -0.0036),
                  (0.024, 0.0), (0.0228, 0.0036), (0.019, 0.0045), (0.0095, 0.0045), (0.0085, 0.0032),
                  (0, 0.0032)], seg=14).fit((-0.024, -0.005, -0.024), (0.024, 0.005, 0.024))


def vac_canister():
    # upright body: rounded shell, parting seam over the motor housing, a front
    # bay where the dust cup (the placer's clear box) sits, side vents, and
    # the wand socket on top
    o = xz_outl(rounded_rect(0.16, 0.15, 0.024, seg=3))
    s = ystack(o, [(0.004, -0.21), (0, -0.2065), (0, -0.143), (0.0018, -0.1405), (0, -0.138),
                   (0, 0.188), (0.003, 0.197), (0.009, 0.2)])
    bay = box(0.100, 0.146, 0.07).move(x=0.011, y=-0.086, z=0.071)
    vents = [box(0.02, 0.004, 0.07).move(x=sx * 0.08, y=y) for sx in (-1, 1) for y in (-0.192, -0.181, -0.170, -0.159)]
    s = cut(s, bay, *vents)
    boss = cyl(0.021, 0.021, 0.016, seg=16, bevel_r=0.003).move(y=0.202, z=0.045)
    return fitb(s + boss, 0.16, 0.42, 0.15)


def vac_wand():
    return lathe([(0, -0.375), (0.0105, -0.375), (0.012, -0.372), (0.012, -0.013), (0.0135, -0.011), (0.014, -0.007),
                  (0.014, 0.007), (0.0135, 0.011), (0.012, 0.013), (0.012, 0.372), (0.0105, 0.375), (0, 0.375)],
                 seg=12).fit((-0.014, -0.375, -0.014), (0.014, 0.375, 0.014))


def vac_grip():
    return lathe([(0, -0.04), (0.0185, -0.04), (0.02, -0.037), (0.02, -0.024), (0.0178, -0.02), (0.02, -0.016),
                  (0.02, 0.016), (0.0178, 0.02), (0.02, 0.024), (0.02, 0.037), (0.0185, 0.04), (0, 0.04)], seg=10)


def vac_plug():
    pts = [(-0.01, -0.0028), (-0.004, -0.005), (0.01, -0.005), (0.01, 0.005), (-0.004, 0.005), (-0.01, 0.0028)]
    return fitb(prism(pts, 0.014, bevel_r=0.0012), 0.020, 0.010, 0.014)


part('VacuumCleaners__base', f_box(0.32, 0.045, 0.20), vac_base, smooth=45)
part('VacuumCleaners__roller', f_cyl(0.020, 0.020, 0.288), vac_roller, smooth=60)
part('VacuumCleaners__wheel', f_cyl(0.024, 0.024, 0.010), vac_wheel, smooth=50)
part('VacuumCleaners__canister', f_box(0.16, 0.42, 0.15), vac_canister, smooth=45)
part('VacuumCleaners__wand', f_cyl(0.014, 0.014, 0.75), vac_wand)
part('VacuumCleaners__grip', f_cyl(0.020, 0.020, 0.080), vac_grip, smooth=50)
part('VacuumCleaners__button', f_box(0.006, 0.010, 0.008), lambda: rbox(0.006, 0.010, 0.008, r=0.0012, seg=1))
part('VacuumCleaners__plug', f_box(0.020, 0.010, 0.014), vac_plug)
part('VacuumCleaners__prong', f_box(0.010, 0.002, 0.003), lambda: rbox(0.010, 0.002, 0.003, r=0.0006, seg=1))


# ---------------------------------------------------------- broom/dustpan --
def broom_handle():
    # turned dowel: rounded hanging end with a hole (+Y), threaded tip (-Y)
    s = lathe([(0, -0.45), (0.0055, -0.45), (0.0078, -0.446), (0.0078, -0.437), (0.0088, -0.435), (0.0088, -0.428),
               (0.01, -0.424), (0.01, 0.395), (0.0094, 0.398), (0.0094, 0.436), (0.0085, 0.445), (0.006, 0.4492),
               (0, 0.45)], seg=12)
    s = cut(s, cyl(0.0033, 0.0033, 0.03, seg=8, bevel_r=0).rot(x=HPI).move(y=0.427))
    return s.fit((-0.01, -0.45, -0.01), (0.01, 0.45, 0.01))


def broom_head():
    # the bristle block: eased block, a moulded groove round it and the
    # threaded socket boss the handle screws into at -X
    blk = rbox(0.13, 0.016, 0.05, r=0.0035, seg=2).move(x=0.005, y=-0.002)
    blk = cut(blk, *[box(0.14, 0.0014, 0.003).move(y=-0.0035, z=sz * 0.0255) for sz in (-1, 1)])
    boss = lathe([(0.0098, -0.0), (0.0098, 0.018), (0.0082, 0.021), (0.0, 0.021)], seg=12, cap_bottom=True).rot(z=HPI)
    boss.move(x=-0.049)
    return fitb(blk + boss, 0.14, 0.020, 0.05)


def pan_floor():
    plate = rbox(0.16, 0.0032, 0.11, r=0.0012, seg=1).move(y=-0.0014)
    ribs = Shape()
    for i in range(7):
        x = -0.06 + 0.02 * i
        ribs = ribs + prism([(-0.0017, 0.0), (0.0017, 0.0), (0.0008, 0.0032), (-0.0008, 0.0032)], 0.09).move(x=x, y=0.0, z=0.003)
    return fitb(plate + ribs, 0.16, 0.006, 0.11)


def pan_back():
    s = rbox(0.16, 0.030, 0.008, r=0.0028, seg=2)
    boss = rbox(0.03, 0.022, 0.004, r=0.0015, seg=1).move(y=0.002, z=-0.004)
    return fitb(s + boss, 0.16, 0.030, 0.008)


def pan_side():
    # side wall, high at the back (-Z), sweeping down to the front (+Z)
    pts = [(-0.055, -0.012), (0.055, -0.012), (0.055, -0.0075), (0.045, -0.0048), (0.02, 0.0015),
           (-0.02, 0.008), (-0.045, 0.0115), (-0.055, 0.012)]
    s = prism([(-z, y) for z, y in pts], 0.006, bevel_r=0.0012).rot(y=HPI)
    return fitb(s, 0.006, 0.024, 0.11)


def pan_lip():
    # sweeping blade: thick where it meets the floor (-Z), feathered at +Z
    pts = [(-0.015, -0.007), (0.015, -0.007), (0.0155, -0.0062), (0.0145, -0.005), (-0.002, 0.0035), (-0.015, 0.007)]
    s = prism([(-z, y) for z, y in pts], 0.14, bevel_r=0.0008).rot(y=HPI)
    return fitb(s, 0.14, 0.014, 0.03)


def pan_handle():
    return lathe([(0, -0.05), (0.0056, -0.05), (0.006, -0.046), (0.006, 0.018), (0.0052, 0.022), (0.006, 0.026),
                  (0.0052, 0.030), (0.006, 0.034), (0.0058, 0.045), (0.0042, 0.0495), (0, 0.05)], seg=10)


def lumpy_pile(r, seed, sub=2, amt=0.18):
    s = subdivide(box(1, 1, 1), sub)
    s.warp(lambda p: p.normalized() * 0.5)
    s = noisy(s, amt, scale=2.4, seed=seed)
    return s.fit((-r, -r, -r), (r, r, r))


part('BroomsAndDustpans__handle', f_cyl(0.010, 0.010, 0.90), broom_handle)
part('BroomsAndDustpans__head', f_box(0.14, 0.020, 0.05), broom_head)
part('BroomsAndDustpans__band', f_torus(0.012, 0.002),
     lambda: dtorus(0.012, 0.002, seg=12, prof=[(0.0018, 0.002), (-0.002, 0.002), (-0.002, -0.002), (0.0018, -0.002)]))
part('BroomsAndDustpans__panFloor', f_box(0.16, 0.006, 0.11), pan_floor)
part('BroomsAndDustpans__panBack', f_box(0.16, 0.030, 0.008), pan_back)
part('BroomsAndDustpans__panSide', f_box(0.006, 0.024, 0.11), pan_side)
part('BroomsAndDustpans__panLip', f_box(0.14, 0.014, 0.03), pan_lip)
part('BroomsAndDustpans__panHandle', f_cyl(0.006, 0.006, 0.10), pan_handle, smooth=50)
part('BroomsAndDustpans__hookLoop', f_torus(0.006, 0.001),
     lambda: dtorus(0.006, 0.001, seg=12, prof=[(0.001, 0), (0, 0.001), (-0.001, 0), (0, -0.001)]), smooth=60)
part('BroomsAndDustpans__dust', f_sphere(0.020), lambda: lumpy_pile(0.02, 3), smooth=180)
part('BroomsAndDustpans__crumb', f_sphere(0.004), lambda: lumpy_pile(0.004, 8, sub=1, amt=0.22), flat=True)


# --------------------------------------------------------------- suitcase --
def suitcase_body():
    # hardshell: rounded corners, a parting groove round the middle (the gold
    # seam band sits in it), moulded vertical ribs on both big faces
    o = xz_outl(rounded_rect(0.30, 0.14, 0.026, seg=4))
    s = ystack(o, [(0.006, -0.21), (0.0022, -0.2085), (0.0004, -0.205), (0, -0.2), (0, -0.0035), (0.0022, -0.002),
                   (0.0022, 0.002), (0, 0.0035), (0, 0.2), (0.0004, 0.205), (0.0022, 0.2085), (0.006, 0.21)])
    ribs = [rbox(0.009, 0.17, 0.006, r=0.002, seg=1).move(x=x, y=sy * 0.1, z=sz * 0.0715)
            for x in (-0.085, -0.03, 0.03, 0.085) for sy in (-1, 1) for sz in (-1, 1)]
    return fitb(cut(s, *ribs), 0.30, 0.42, 0.14)


def suitcase_seam():
    return ystack(xz_outl(rounded_rect(0.305, 0.145, 0.0285, seg=4)),
                  [(0.0007, -0.0015), (0, -0.0007), (0, 0.0007), (0.0007, 0.0015)])


def suitcase_latch():
    pts = [(-0.007, -0.005), (-0.002, -0.005), (0.007, -0.0032), (0.007, 0.0022), (0.0045, 0.005), (-0.007, 0.005)]
    s = prism([(-z, y) for z, y in pts], 0.024, bevel_r=0.0008).rot(y=HPI)
    return fitb(s, 0.024, 0.010, 0.014)


def small_wheel():
    return lathe([(0, -0.0042), (0.0115, -0.005), (0.014, -0.0026), (0.014, 0.0026), (0.0115, 0.005), (0, 0.0042)],
                 seg=10).fit((-0.014, -0.005, -0.014), (0.014, 0.005, 0.014))


def trolley_shaft():
    return lathe([(0, -0.07), (0.006, -0.07), (0.006, 0.0), (0.0052, 0.0022), (0.0047, 0.003), (0.0047, 0.07),
                  (0, 0.07)], seg=8).fit((-0.006, -0.07, -0.006), (0.006, 0.07, 0.006))


def trolley_bar():
    return lathe([(0, -0.0975), (0.0066, -0.0975), (0.008, -0.094), (0.008, -0.078), (0.0068, -0.075),
                  (0.0072, -0.06), (0.0066, -0.045), (0.0072, -0.03), (0.0072, 0.03), (0.0066, 0.045),
                  (0.0072, 0.06), (0.0068, 0.075), (0.008, 0.078), (0.008, 0.094), (0.0066, 0.0975),
                  (0, 0.0975)], seg=10).fit((-0.008, -0.0975, -0.008), (0.008, 0.0975, 0.008))


def carry_handle():
    pts = [(-0.04, 0.0, 0), (-0.04, 0.02, 0), (-0.036, 0.034, 0), (-0.024, 0.04, 0), (0.024, 0.04, 0),
           (0.036, 0.034, 0), (0.04, 0.02, 0), (0.04, 0.0, 0)]
    grip = tube(pts, 0.008, seg=6, smooth=1)
    mounts = sum((rbox(0.02, 0.008, 0.016, r=0.002, seg=1).move(x=sx * 0.038, y=0.004) for sx in (-1, 1)), Shape())
    return (grip + mounts).fit((-0.048, 0, -0.008), (0.048, 0.048, 0.008))


def side_handle():
    pts = []
    for k in range(11):
        u = math.pi * k / 10
        c, s_ = math.cos(u), math.sin(u)
        pts.append((0.018 * math.copysign(abs(c) ** 0.6, c), 0.018 * abs(s_) ** 0.5, 0))
    return tube(pts, 0.005, seg=6).fit((-0.023, 0, -0.005), (0.023, 0.023, 0.005))


def luggage_tag():
    pts = round_outline([(-0.014, -0.016), (0.014, -0.016), (0.014, 0.009), (0.007, 0.016), (-0.007, 0.016),
                         (-0.014, 0.009)], 0.0018, seg=2)
    s = prism(pts, 0.0016, bevel_r=0.0003)
    s = cut(s, cyl(0.0021, 0.0021, 0.01, seg=8, bevel_r=0).rot(x=HPI).move(y=0.0105))
    return fitb(s, 0.028, 0.032, 0.002)


part('PackedSuitcases__body', f_box(0.30, 0.42, 0.14), suitcase_body)
part('PackedSuitcases__seam', f_box(0.305, 0.003, 0.145), suitcase_seam)
part('PackedSuitcases__latch', f_box(0.024, 0.010, 0.014), suitcase_latch)
part('PackedSuitcases__corner', f_box(0.024, 0.016, 0.024), lambda: rbox(0.024, 0.016, 0.024, r=0.0055, seg=2), smooth=60)
part('PackedSuitcases__wheel', f_cyl(0.014, 0.014, 0.010), small_wheel, smooth=50)
part('PackedSuitcases__shaft', f_cyl(0.006, 0.006, 0.14), trolley_shaft)
part('PackedSuitcases__bar', f_cyl(0.008, 0.008, 0.195), trolley_bar)
part('PackedSuitcases__carryHandle', f_bounds((-0.048, 0, -0.008), (0.048, 0.048, 0.008)), carry_handle, smooth=60)
part('PackedSuitcases__sideHandle', f_bounds((-0.023, 0, -0.005), (0.023, 0.023, 0.005)), side_handle, smooth=60)
part('PackedSuitcases__tag', f_box(0.028, 0.032, 0.002), luggage_tag)
part('PackedSuitcases__eyelet', f_torus(0.003, 0.0008),
     lambda: dtorus(0.003, 0.0008, seg=10, prof=[(0.0008, 0), (0, 0.0008), (-0.0008, 0), (0, -0.0008)]), smooth=60)


# ------------------------------------------------------- travel documents --
def passport():
    # two card covers wrapping a page block, a rounded spine at -X; the top
    # stays flat for the cover texture
    cov = lambda y: prism(rounded_rect(0.0735, 0.1, 0.002, seg=2), 0.0007).rot(x=-HPI).move(x=0.00075, y=y)
    pages = rbox(0.0715, 0.0048, 0.097, r=0.0006, seg=1).move(x=0.0003)
    grooves = [box(0.004, 0.0003, 0.1).move(x=0.036, y=y) for y in (-0.0012, 0.0, 0.0012)]
    spine = cyl(0.003, 0.003, 0.1, seg=12, bevel_r=0.0006).rot(x=HPI).move(x=-0.0345)
    return fitb(cov(0.00265) + cov(-0.00265) + cut(pages, *grooves) + spine, 0.075, 0.006, 0.100)


def banknote(w, d, seed=1):
    s = slab(w, d, 0.0003, 5, 3).rot(x=-HPI)
    s.warp(lambda p: Vector((p.x, p.y + 0.0006 * (1 - abs(p.x) / (w / 2)) + 0.0001 * math.sin(p.z / d * 9 + seed), p.z)))
    return fitb(s, w, 0.001, d)


def pen_barrel():
    return lathe([(0, -0.05), (0.0017, -0.05), (0.002, -0.0488), (0.0026, -0.048), (0.003, -0.046), (0.003, 0.018),
                  (0.0027, 0.021), (0.0028, 0.034), (0.0026, 0.04), (0.0019, 0.0475), (0.0012, 0.05), (0, 0.05)], seg=8)


def pen_tip():
    return lathe([(0, -0.004), (0.002, -0.004), (0.0016, -0.0012), (0.0006, 0.0028), (0.0004, 0.0036), (0, 0.004)], seg=8)


part('TravelDocuments__passport', f_box(0.075, 0.006, 0.100), passport)
part('TravelDocuments__bill', f_box(0.05, 0.001, 0.030), lambda: banknote(0.05, 0.03, 1), smooth=60)
part('TravelDocuments__pen', f_cyl(0.003, 0.003, 0.10), pen_barrel)
part('TravelDocuments__penTip', f_cone(0.002, 0.008), pen_tip, smooth=50)


# ----------------------------------------------------------- glasses case --
CASE_O = rounded_rect(0.15, 0.07, 0.028, seg=4)


def gcase_bottom():
    # lower shell: belly rounding under, a flat rim on top where the lid shuts
    s = scaled_rings(CASE_O, [(0.84, -0.01), (0.93, -0.0085), (0.975, -0.0058), (0.996, -0.0025), (1.0, 0.0065),
                              (0.994, 0.0088), (0.985, 0.01)])
    s = cut(s, *[box(0.2, 0.0008, 0.2).move(y=0.0052)])
    return fitb(s, 0.15, 0.020, 0.07)


def gcase_lining():
    return fitb(scaled_rings(rounded_rect(0.14, 0.06, 0.025, seg=4),
                             [(0.965, -0.0015), (1.0, -0.0007), (1.0, 0.0003), (0.975, 0.0012), (0.92, 0.0015)]),
                0.14, 0.003, 0.06)


def gcase_lid():
    # lid shell: domed outside (+Y), hollow inside (-Y) 4 mm in from the edge
    s = scaled_rings(CASE_O, [(1.0, -0.007), (1.0, 0.0042), (0.992, 0.0058), (0.972, 0.0067), (0.93, 0.007),
                              (0.8, 0.007)])
    hollow = ystack(xz_outl(rounded_rect(0.142, 0.062, 0.024, seg=4)), [(0, -0.012), (0, 0.0048), (0.002, 0.0052)])
    return fitb(cut(s, hollow), 0.15, 0.014, 0.07)


def knuckle_hinge(r, h, n=5, seg=8):
    prof = [(0, -h / 2)]
    for k in range(n):
        y0, y1 = -h / 2 + h * k / n, -h / 2 + h * (k + 1) / n
        g = h / n * 0.08
        prof += [(r * 0.82, y0 + (0 if k == 0 else g)), (r, y0 + g * 1.6), (r, y1 - g * 1.6), (r * 0.82, y1 - (0 if k == n - 1 else g))]
    prof.append((0, h / 2))
    return lathe(prof, seg=seg).fit((-r, -h / 2, -r), (r, h / 2, r))


def lens_cloth():
    s = slab(0.05, 0.05, 0.0005, 6, 6).rot(x=-HPI)
    s.warp(lambda p: Vector((p.x, p.y + 0.0006 * math.sin(p.x * 180 + p.z * 60) + 0.0005 * (p.z / 0.025) ** 2, p.z)))
    return fitb(noisy(s, 0.0002, scale=90, seed=5, axis=(0, 1, 0)), 0.05, 0.002, 0.05)


def cloth_fold():
    s = slab(0.03, 0.03, 0.0006, 4, 4).rot(x=-HPI)
    s.warp(lambda p: Vector((p.x, p.y + 0.0022 * max(0.0, (p.x + p.z) / 0.03) ** 2, p.z)))
    return fitb(s, 0.03, 0.003, 0.03)


def loop_tube(path, r, seg=6, rz=None):
    """A closed tube round a closed planar (x, y) path, section in the plane
    of the path normal and Z (rz: section half-size along Z, default r)."""
    rz = r if rz is None else rz
    n = len(path)
    v, f = [], []
    for i in range(n):
        p0, p1 = Vector(path[i - 1]), Vector(path[(i + 1) % n])
        t = (p1 - p0).normalized()
        nr = Vector((t.y, -t.x))
        for j in range(seg):
            a = TAU * j / seg
            q = Vector(path[i]) + nr * (r * math.cos(a))
            v.append((q.x, q.y, rz * math.sin(a)))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i2 * seg + j, i2 * seg + j2, i * seg + j2))
    return fixw(Shape(v, f))


def superellipse(a, b, n, p=0.8):
    return [(a * math.copysign(abs(math.cos(TAU * k / n)) ** p, math.cos(TAU * k / n)),
             b * math.copysign(abs(math.sin(TAU * k / n)) ** p, math.sin(TAU * k / n))) for k in range(n)]


def eye_rim():
    s = loop_tube(superellipse(0.012, 0.0117, 20), 0.0015, seg=5)
    return s.fit((-0.0135, -0.0135, -0.0015), (0.0135, 0.0135, 0.0015))


def eye_bridge():
    s = tube([(-0.0037, -0.0003, 0), (-0.002, 0.0003, 0), (0.0, 0.0006, 0), (0.002, 0.0003, 0), (0.0037, -0.0003, 0)],
             0.0007, seg=5)
    return fitb(s, 0.008, 0.002, 0.002)


def eye_temple():
    return lathe([(0, -0.014), (0.0008, -0.014), (0.001, -0.0125), (0.001, -0.011), (0.0007, -0.0095), (0.0006, 0.009),
                  (0.0008, 0.0128), (0, 0.014)], seg=5).fit((-0.001, -0.014, -0.001), (0.001, 0.014, 0.001))


part('GlassesCases__bottom', f_box(0.15, 0.020, 0.07), gcase_bottom, smooth=50)
part('GlassesCases__lining', f_box(0.14, 0.003, 0.06), gcase_lining, smooth=60)
part('GlassesCases__lid', f_box(0.15, 0.014, 0.07), gcase_lid, smooth=50)
part('GlassesCases__hinge', f_cyl(0.003, 0.003, 0.1275), lambda: knuckle_hinge(0.003, 0.1275))
part('GlassesCases__clasp', f_box(0.006, 0.004, 0.002), lambda: rbox(0.006, 0.004, 0.002, r=0.0006, seg=1))
part('GlassesCases__cloth', f_box(0.05, 0.002, 0.05), lens_cloth, smooth=70)
part('GlassesCases__clothFold', f_box(0.03, 0.003, 0.03), cloth_fold, smooth=70)
part('GlassesCases__rim', f_torus(0.012, 0.0015), eye_rim, smooth=60)
part('GlassesCases__bridge', f_box(0.008, 0.002, 0.002), eye_bridge, smooth=60)
part('GlassesCases__temple', f_cyl(0.001, 0.001, 0.028), eye_temple, smooth=60)


# ------------------------------------------------------------- wristwatch --
def watch_case():
    # caseback, rounded case band, bezel ring round a flat crystal
    return lathe([(0, -0.005), (0.0165, -0.005), (0.0192, -0.0032), (0.02, -0.0005), (0.0196, 0.003),
                  (0.0184, 0.005), (0.0168, 0.0045), (0, 0.0045)], seg=20).fit((-0.02, -0.005, -0.02), (0.02, 0.005, 0.02))


def watch_hand(w, h, L):
    pts = [(-w / 2, -L / 2), (w / 2, -L / 2), (w * 0.4, L * 0.2), (0.0, L / 2), (-w * 0.4, L * 0.2)]
    return fitb(prism(pts, h).rot(x=HPI), w, h, L)


def watch_crown():
    return lathe([(0, -0.002), (0.0011, -0.002), (0.0011, -0.0013), (0.002, -0.0011), (0.002, 0.0014),
                  (0.0015, 0.002), (0, 0.002)], seg=8)


def strap_piece():
    # leather strap segment: crowned top, a stitch groove along each edge
    half = [(0.006, -0.0015), (0.006, 0.0002), (0.0054, 0.0011), (0.0047, 0.0014), (0.0043, 0.001), (0.0039, 0.0014),
            (0.0, 0.0015)]
    pts = half + [(-x, y) for x, y in reversed(half[:-1])]
    return fitb(prism(pts, 0.015), 0.012, 0.003, 0.015)


def watch_buckle():
    path = [(x, z) for x, z in round_outline([(-0.0055, -0.0065), (0.0055, -0.0065), (0.0055, 0.0065), (-0.0055, 0.0065)], 0.0028, seg=2)]
    s = loop_tube(path, 0.0014, seg=4).rot(x=-HPI)
    return fitb(s, 0.014, 0.005, 0.016)


def buckle_tongue():
    return lathe([(0, -0.007), (0.0009, -0.007), (0.001, -0.006), (0.0007, 0.005), (0.0004, 0.0066), (0, 0.007)], seg=5)


part('Wristwatches__case', f_cyl(0.020, 0.020, 0.010), watch_case, smooth=50)
part('Wristwatches__hourHand', f_box(0.0015, 0.0009, 0.008), lambda: watch_hand(0.0015, 0.0009, 0.008))
part('Wristwatches__minuteHand', f_box(0.001, 0.0009, 0.014), lambda: watch_hand(0.001, 0.0009, 0.014))
part('Wristwatches__crown', f_cyl(0.002, 0.002, 0.004), watch_crown)
part('Wristwatches__strap', f_box(0.012, 0.003, 0.015), strap_piece)
part('Wristwatches__buckle', f_box(0.014, 0.005, 0.016), watch_buckle, smooth=60)
part('Wristwatches__tongue', f_cyl(0.001, 0.001, 0.014), buckle_tongue, smooth=60)


# ---------------------------------------------------------- opened wallet --
def wallet_half(side):
    # one leather leaf: rounded outer corners, square at the fold (side = +1
    # fold at +X), a stitch groove running 3 mm in from the edge
    w, d, r = 0.055, 0.09, 0.007
    x0, x1 = -w / 2, w / 2
    fold_x = x1 if side > 0 else x0
    out_x = x0 if side > 0 else x1
    corners = [(fold_x, -d / 2), (out_x, -d / 2), (out_x, d / 2), (fold_x, d / 2)]
    pts = []
    for i, c in enumerate(corners):
        if abs(c[0] - out_x) < 1e-9:
            pts += propkit._round_corner(c, corners[i - 1], corners[(i + 1) % 4], r, 3)
        else:
            pts.append(c)
    s = ystack(xz_outl(pts), [(0.0007, -0.0025), (0, -0.0017), (0, 0.0015), (0.0007, 0.0025), (0.0027, 0.0025),
                              (0.003, 0.0019), (0.0033, 0.0025), (0.004, 0.0025)])
    return fitb(s, w, 0.005, d)


def wallet_spine():
    pts = [(-0.002, -0.0035), (0.002, -0.0035), (0.002, 0.0008), (0.0015, 0.0025), (0.0006, 0.0033), (0, 0.0035),
           (-0.0006, 0.0033), (-0.0015, 0.0025), (-0.002, 0.0008)]
    return fitb(prism(pts, 0.09), 0.004, 0.007, 0.09)


def pocket_lip():
    s = slab(0.04675, 0.003, 0.0015, 6, 1).rot(x=-HPI)
    s.warp(lambda p: Vector((p.x, p.y, p.z - 0.0009 * (1 - (p.x / 0.0234) ** 2))))
    return fitb(s, 0.04675, 0.0015, 0.003)


def pocket_panel():
    s = ystack(xz_outl(rounded_rect(0.04675, 0.0765, 0.003, seg=2)),
               [(0.0004, -0.00075), (0, -0.0003), (0, 0.0003), (0.0004, 0.00075), (0.0022, 0.00075), (0.0025, 0.0003),
                (0.0028, 0.00075), (0.0034, 0.00075)])
    return fitb(s, 0.04675, 0.0015, 0.0765)


def credit_card():
    return ystack(xz_outl(rounded_rect(0.085, 0.055, 0.0032, seg=2)),
                  [(0.0002, -0.0005), (0, -0.0003), (0, 0.0003), (0.0002, 0.0005)])


def card_chip():
    base = box(0.010, 0.0006, 0.008).move(y=-0.00045)
    pads = sum((box(0.0044, 0.0009, 0.0022).move(x=sx * 0.00255, y=0.0003, z=z) for sx in (-1, 1)
                for z in (-0.0026, 0.0, 0.0026)), Shape())
    return fitb(base + pads, 0.010, 0.0015, 0.008)


def card_emboss():
    digits = Shape()
    for g in range(4):
        for k in range(4):
            x = -0.03 + g * 0.0158 + k * 0.0034 + 0.0014
            digits = digits + box(0.0026, 0.0012, 0.003).move(x=x)
    return fitb(digits, 0.060, 0.0012, 0.003)


part('OpenedWallets__leftHalf', f_box(0.055, 0.005, 0.09), lambda: wallet_half(+1), smooth=50)
part('OpenedWallets__rightHalf', f_box(0.055, 0.005, 0.09), lambda: wallet_half(-1), smooth=50)
part('OpenedWallets__spine', f_box(0.004, 0.007, 0.09), wallet_spine, smooth=60)
part('OpenedWallets__slotLip', f_box(0.04675, 0.0015, 0.003), pocket_lip, smooth=60)
part('OpenedWallets__cashPocket', f_box(0.04675, 0.0015, 0.0765), pocket_panel, smooth=50)
part('OpenedWallets__card', f_box(0.085, 0.001, 0.055), credit_card)
part('OpenedWallets__chip', f_box(0.010, 0.0015, 0.008), card_chip)
part('OpenedWallets__emboss', f_box(0.060, 0.0012, 0.003), card_emboss)
part('OpenedWallets__bill', f_box(0.045, 0.001, 0.025), lambda: banknote(0.045, 0.025, 3), smooth=60)


# ------------------------------------------------------------ jewelry box --
def jbox_base():
    # plinth on four bun feet, eased body, a bead moulding round the top and
    # a shallow tray sunk inside the 6 mm rim (the velvet pad sits in it)
    o = xz_outl(rounded_rect(0.16, 0.11, 0.004, seg=2))
    body = ystack(o, [(0.0012, -0.0215), (0, -0.0205), (0, -0.0165), (0.0012, -0.0155), (0.0026, -0.0145),
                      (0.0026, 0.0205), (0.0014, 0.0215), (0.0014, 0.0235), (0.0022, 0.025), (0.003, 0.025)])
    body = cut(body, ystack(xz_outl(rounded_rect(0.148, 0.098, 0.002, seg=1)), [(0, 0.023), (0, 0.03)]))
    feet = sum((lathe([(0, -0.025), (0.0045, -0.025), (0.0065, -0.0235), (0.0065, -0.022), (0.005, -0.0208), (0, -0.0208)],
                      seg=8).move(x=sx * 0.07, z=sz * 0.045) for sx in (-1, 1) for sz in (-1, 1)), Shape())
    return fitb(body + feet, 0.16, 0.05, 0.11)


def pillow_grid(xs, zs, top_fn, y0):
    """A closed pad: a top surface over the grid xs × zs with heights
    top_fn(x, z), a flat bottom at y0 and side walls."""
    nx, nz = len(xs), len(zs)
    v, f = [], []
    for z in zs:
        for x in xs:
            v.append((x, top_fn(x, z), z))
    T = lambda i, k: k * nx + i
    for k in range(nz - 1):
        for i in range(nx - 1):
            f.append((T(i, k), T(i, k + 1), T(i + 1, k + 1), T(i + 1, k)))
    ring = [T(i, 0) for i in range(nx)] + [T(nx - 1, k) for k in range(1, nz)] + \
           [T(i, nz - 1) for i in range(nx - 2, -1, -1)] + [T(0, k) for k in range(nz - 2, 0, -1)]
    base = len(v)
    for idx in ring:
        x, _, z = v[idx]
        v.append((x, y0, z))
    m = len(ring)
    for j in range(m):
        j2 = (j + 1) % m
        f.append((ring[j], ring[j2], base + j2, base + j))
    f.append(tuple(base + j for j in range(m)))
    return fixw(Shape(v, f))


def jbox_pad():
    # velvet cushion puffed up in each of the six compartments
    xb = [-0.074, -0.04, 0.04, 0.074]
    zb = [-0.049, 0.0, 0.049]
    xs = [-0.074, -0.068, -0.057, -0.046, -0.04, -0.034, -0.018, 0.0, 0.018, 0.034, 0.04, 0.046, 0.057, 0.068, 0.074]
    zs = [-0.049, -0.043, -0.027, -0.009, 0.0, 0.009, 0.027, 0.043, 0.049]
    def bump(t, bs):
        for a, b in zip(bs, bs[1:]):
            if a - 1e-9 <= t <= b + 1e-9:
                return math.sin(math.pi * (t - a) / (b - a)) ** 0.6
        return 0.0
    top = lambda x, z: -0.0005 + 0.0025 * bump(x, xb) * bump(z, zb)
    return fitb(pillow_grid(xs, zs, top, -0.002), 0.148, 0.004, 0.098)


def divider(L, along='z'):
    pts = [(-0.001, -0.003), (0.001, -0.003), (0.001, 0.0018), (0.0006, 0.0027), (0, 0.003), (-0.0006, 0.0027), (-0.001, 0.0018)]
    s = prism(pts, L)
    if along == 'x':
        s.rot(y=HPI)
    return s


def jbox_lid():
    # lid: moulded edge, a raised fielded panel outside (+Y) and hollowed
    # underneath so the lid lining shows inside
    s = ystack(rr(0.16, 0.11, 0.004, 2), [(0.0012, -0.0125), (0, -0.0115), (0, 0.0045), (0.0012, 0.0072), (0.0035, 0.0086),
                                         (0.011, 0.0088), (0.0175, 0.0125), (0.0185, 0.0125)])
    hollow = ystack(xz_outl(rounded_rect(0.148, 0.098, 0.002, seg=1)), [(0, -0.02), (0, 0.0023)])
    return fitb(cut(s, hollow), 0.16, 0.025, 0.11)


def lid_lining():
    return fitb(scaled_rings(rounded_rect(0.148, 0.098, 0.003, seg=2),
                             [(0.96, -0.0015), (0.99, -0.0009), (1.0, 0.0), (0.99, 0.0009), (0.96, 0.0015)]),
                0.148, 0.003, 0.098)


def jbox_clasp():
    plate = prism([(-0.005, 0.003), (0.005, 0.003), (0.005, -0.0008), (0.0022, -0.003), (-0.0022, -0.003), (-0.005, -0.0008)],
                  0.002, bevel_r=0.0004).move(z=-0.0005)
    stud = cyl(0.0012, 0.0012, 0.0012, seg=6, bevel_r=0).rot(x=HPI).move(y=-0.0006, z=0.0009)
    return fitb(plate + stud, 0.010, 0.006, 0.003)


def gem(r, crown=0.45, table=0.55, seg=8):
    """A faceted stone r in every direction: pavilion to a culet, girdle, crown, table."""
    return lathe([(0, -r), (r, -r + 2 * r * (1 - crown) - 0.12 * r), (r, -r + 2 * r * (1 - crown)),
                  (r * (table + 1) / 2, r - 0.35 * r * crown * 2), (r * table, r)], seg=seg).fit((-r, -r, -r), (r, r, r))


def chain_links():
    s = Shape()
    for i in range(3):
        y = -0.0042 + 0.0042 * i
        link = loop_tube([(0.00085 * math.cos(TAU * k / 5 + 0.3), 0.0023 * math.sin(TAU * k / 5 + 0.3)) for k in range(5)],
                         0.00036, seg=3)
        link.move(y=y)
        if i % 2:
            link.rot(y=HPI)
        s = s + link
    return s.fit((-0.0012, -0.006, -0.0012), (0.0012, 0.006, 0.0012))


def band_ring():
    prof = [(0.002, 0.0), (0.0016, 0.0013), (0.0006, 0.002), (-0.0012, 0.0019), (-0.002, 0.0012), (-0.002, -0.0012),
            (-0.0012, -0.0019), (0.0006, -0.002), (0.0016, -0.0013)]
    return dtorus(0.010, 0.002, seg=14, prof=prof)


part('JewelryBoxes__base', f_box(0.16, 0.05, 0.11), jbox_base)
part('JewelryBoxes__pad', f_box(0.148, 0.004, 0.098), jbox_pad, smooth=70)
part('JewelryBoxes__divider', f_box(0.002, 0.006, 0.096), lambda: fitb(divider(0.096), 0.002, 0.006, 0.096))
part('JewelryBoxes__crossDivider', f_box(0.146, 0.006, 0.002), lambda: fitb(divider(0.146, 'x'), 0.146, 0.006, 0.002))
part('JewelryBoxes__lid', f_box(0.16, 0.025, 0.11), jbox_lid)
part('JewelryBoxes__lidLining', f_box(0.148, 0.003, 0.098), lid_lining, smooth=60)
part('JewelryBoxes__hinge', f_cyl(0.003, 0.003, 0.136), lambda: knuckle_hinge(0.003, 0.136))
part('JewelryBoxes__clasp', f_box(0.010, 0.006, 0.003), jbox_clasp)
part('JewelryBoxes__ring', f_torus(0.010, 0.002), band_ring, smooth=60)
part('JewelryBoxes__stone', f_bounds((-0.004, -0.004, -0.004), (0.004, 0.004, 0.004)), lambda: gem(0.004), flat=True)
part('JewelryBoxes__chain', f_cyl(0.0012, 0.0012, 0.012), chain_links, smooth=60)
part('JewelryBoxes__pendant', f_bounds((-0.006, -0.006, -0.006), (0.006, 0.006, 0.006)), lambda: gem(0.006, crown=0.4, table=0.5), flat=True)
part('JewelryBoxes__earring', f_torus(0.005, 0.0008),
     lambda: dtorus(0.005, 0.0008, seg=12, prof=[(0.0008, 0), (0, 0.0008), (-0.0008, 0), (0, -0.0008)]), smooth=60)
part('JewelryBoxes__earStone', f_bounds((-0.003, -0.003, -0.003), (0.003, 0.003, 0.003)), lambda: gem(0.003, seg=6), flat=True)


# ---------------------------------------------------------- umbrella stand --
def stand_body():
    # tall bin: a raised foot band, a slow taper up to a collar band at the top
    return vessel([(0.085, -0.275), (0.088, -0.272), (0.088, -0.256), (0.0858, -0.252), (0.0965, 0.238),
                   (0.0995, 0.242), (0.0995, 0.262), (0.1, 0.273)], 0.004, seg=24)


def furled_umbrella():
    # closed canopy: eight pleats wound in a slow spiral, fullest high up, a
    # strap and snap round the belly, the bare shaft running out at the bottom
    ys = [-0.375, -0.30, -0.27, -0.20, -0.10, 0.0, 0.10, 0.17, 0.2, 0.212, 0.224, 0.25, 0.30, 0.34, 0.365, 0.375]
    R = [(-0.375, 0.0024), (-0.30, 0.0024), (-0.27, 0.0038), (-0.2, 0.0048), (0.0, 0.0062), (0.17, 0.0072),
         (0.25, 0.007), (0.3, 0.0058), (0.34, 0.0042), (0.375, 0.0026)]
    rings = []
    for y in ys:
        r = interp(R, y)
        strap = 0.2 <= y <= 0.224
        pleat = 0.0 if (y < -0.28 or strap) else 0.2 * min(1.0, (y + 0.28) / 0.08)
        if strap:
            r = 0.008
        tw = 2.4 * (y + 0.375) / 0.75
        rings.append([(r * (1 - pleat * (k % 2)) * math.cos(TAU * k / 16 + tw), y,
                       r * (1 - pleat * (k % 2)) * math.sin(TAU * k / 16 + tw)) for k in range(16)])
    s = loft(rings)
    snap = cyl(0.0022, 0.0022, 0.002, seg=6, bevel_r=0).rot(z=HPI).move(x=0.0075, y=0.212)
    return (s + snap).fit((-0.008, -0.375, -0.008), (0.008, 0.375, 0.008))


def ferrule_tip():
    return lathe([(0, -0.01), (0.004, -0.01), (0.0037, -0.006), (0.0021, 0.004), (0.0013, 0.0082), (0.0008, 0.0095),
                  (0, 0.01)], seg=8)


def foot_ring():
    prof = [(-0.008, 0.008), (0.006, 0.008), (0.008, 0.0052), (0.008, 0.0), (0.0065, -0.0045), (0.003, -0.0075),
            (-0.002, -0.008), (-0.0065, -0.006), (-0.008, -0.002)]
    return dtorus(0.11, 0.008, seg=28, prof=prof)


part('UmbrellaStands__stand', f_cyl(0.10, 0.085, 0.55), stand_body)
part('UmbrellaStands__rim', f_torus(0.10, 0.006), lambda: dtorus(0.10, 0.006, seg=28), smooth=60)
part('UmbrellaStands__footRing', f_torus(0.11, 0.008), foot_ring, smooth=60)
part('UmbrellaStands__umbrella', f_cyl(0.008, 0.004, 0.75), furled_umbrella, smooth=50)
part('UmbrellaStands__ferrule', f_cone(0.004, 0.02), ferrule_tip, smooth=50)


# --------------------------------------------------------------- boot tray --
def tray_floor():
    # rubber tray: a thin plate with a raised square grid of ribs
    plate = rbox(0.36, 0.0042, 0.22, r=0.0012, seg=1).move(y=-0.0019)
    rib = [(-0.0016, 0.0), (0.0016, 0.0), (0.0008, 0.0038), (-0.0008, 0.0038)]
    ribs = Shape()
    for k in range(10):
        ribs = ribs + prism(rib, 0.33).rot(y=HPI).move(z=-0.09 + 0.02 * k, y=0.0002)
    for k in range(17):
        ribs = ribs + prism(rib, 0.19).move(x=-0.16 + 0.02 * k, y=0.0002)
    return fitb(plate + ribs, 0.36, 0.008, 0.22)


def tray_lip():
    # a lip strip; one constructor makes all four sides, so the top rounding is
    # a fraction of each horizontal axis (long edges ~2 mm, ends ease off)
    w, h, d, f, dy = 0.36, 0.012, 0.01, 0.2, 0.0035
    rect = lambda a, b, y: [(a, y, b), (-a, y, b), (-a, y, -b), (a, y, -b)]
    rings = [rect(w / 2, d / 2, -h / 2)]
    for k in range(4):
        t = HPI * k / 3
        rings.append(rect(w / 2 - f * w * (1 - math.cos(t)), d / 2 - f * d * (1 - math.cos(t)), h / 2 - dy + dy * math.sin(t)))
    return loft(rings)


def boot_leg():
    return vessel([(0.035, -0.1), (0.0345, -0.084), (0.032, -0.04), (0.0302, 0.03), (0.03, 0.1)], 0.0028, seg=18)


def boot_foot():
    zs = [-0.05, -0.03, -0.01, 0.01, 0.025, 0.036, 0.043, 0.0475, 0.05]
    hw = [0.029, 0.03, 0.03, 0.03, 0.028, 0.024, 0.018, 0.011, 0.004]
    yt = [0.015, 0.015, 0.014, 0.011, 0.008, 0.004, 0.0, -0.004, -0.007]
    s = loft([dring(z, w, y, -0.015, m=12, p=0.5) for z, w, y in zip(zs, hw, yt)])
    return fitb(s, 0.060, 0.030, 0.10)


def boot_sole():
    tab = [(-0.055, 0.021), (-0.045, 0.027), (-0.02, 0.028), (0.0, 0.027), (0.025, 0.031), (0.04, 0.029), (0.05, 0.022),
           (0.055, 0.012)]
    zs = [-0.055 + 0.11 * (1 - math.cos(math.pi * i / 10)) / 2 for i in range(11)]
    env = lambda z: math.sqrt(max(0.0, 1 - (z / 0.055) ** 8))
    right = [(max(0.004, interp(tab, z) * env(z)), z) for z in zs]
    pts = right + [(-x, z) for x, z in reversed(right[1:-1])]
    s = ystack(xz_outl(pts), [(0.001, -0.005), (0, -0.0038), (0, 0.004), (0.0008, 0.005)])
    cutters = [box(0.08, 0.0045, 0.022).move(y=-0.0052, z=-0.012)]
    cutters += [box(0.08, 0.004, 0.0028).move(y=-0.0052, z=z) for z in (0.008, 0.02, 0.032, 0.044, -0.038, -0.048)]
    return fitb(cut(s, *cutters), 0.062, 0.010, 0.11)


part('BootTrays__floor', f_box(0.36, 0.008, 0.22), tray_floor)
part('BootTrays__lip', f_box(0.36, 0.012, 0.01), tray_lip, smooth=50)
part('BootTrays__leg', f_cyl(0.030, 0.035, 0.20), boot_leg, smooth=50)
part('BootTrays__foot', f_box(0.060, 0.030, 0.10), boot_foot, smooth=70)
part('BootTrays__sole', f_box(0.062, 0.010, 0.11), boot_sole)
part('BootTrays__trim', f_torus(0.030, 0.003), lambda: dtorus(0.030, 0.003, seg=20, prof=[(0.003 * math.cos(TAU * k / 6), 0.003 * math.sin(TAU * k / 6)) for k in range(6)]), smooth=60)
