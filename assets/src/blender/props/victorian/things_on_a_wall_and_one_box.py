"""t19: the props remodelled at their real sizes (docs/design/PROPS.md, P3, the
rare): things on a parlour's or a chapel's wall, and one box: a gilt-brass
reliquary casket, a roe's skull on its shield, a household crucifix, a
carved and painted armorial shield of no real family, a cut-paper
silhouette in its oval frame, the picture hooks left where a picture hung
(and its wire), and a small iron-tyred wheel. Sources and sizes:
docs/research/props-remodelling/things_on_a_wall_and_one_box.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame (x across its front, y up, z out of its front);
each section says where its origin is.
"""
from propkit import *
from propkit import _mesh_obj, _eval_to_shape, _drop
import propkit
import random
import zlib


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, smooth=35.0, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth)


def rnd(a, b):
    return a + (b - a) * random.random()


def lerp(a, b, t):
    return a + (b - a) * t


# ------------------------------------------------------------- helpers ---
_meta_n = [0]
MB_K = 1.0 / 0.564


def meta(elems, res=0.002, threshold=0.6):
    """A blobby surface from metaball elements (t11's): ('ball', centre,
    radius[, stiffness]), ('ell', centre, (rx, ry, rz)[, stiffness[,
    euler]]), ('cap', a, b, radius[, stiffness]); '-ball' / '-ell' / '-cap'
    subtract."""
    _meta_n[0] += 1
    name = f'_t19mb{_meta_n[0]}x'
    mb = bpy.data.metaballs.new(name)
    mb.resolution = res
    mb.render_resolution = res
    mb.threshold = threshold
    ob = bpy.data.objects.new(name, mb)
    bpy.context.scene.collection.objects.link(ob)
    for e in elems:
        kind = e[0]
        el = mb.elements.new()
        el.use_negative = kind.startswith('-')
        kind = kind.lstrip('-')
        if kind == 'ball':
            el.type = 'BALL'
            el.co = Vector(e[1])
            el.radius = e[2] * MB_K
            el.stiffness = e[3] if len(e) > 3 else 2.0
        elif kind == 'ell':
            el.type = 'ELLIPSOID'
            el.co = Vector(e[1])
            el.radius = MB_K
            el.size_x, el.size_y, el.size_z = e[2]
            el.stiffness = e[3] if len(e) > 3 else 2.0
            if len(e) > 4:
                el.rotation = Euler(e[4], 'XYZ').to_quaternion()
        elif kind == 'cap':
            a, b = Vector(e[1]), Vector(e[2])
            el.type = 'CAPSULE'
            el.co = (a + b) / 2
            el.radius = e[3] * MB_K
            el.size_x = max(1e-5, (b - a).length / 2)
            el.stiffness = e[4] if len(e) > 4 else 2.0
            el.rotation = Vector((1, 0, 0)).rotation_difference((b - a).normalized())
        else:
            raise ValueError(kind)
    s = _eval_to_shape(ob)
    bpy.data.objects.remove(ob)
    bpy.data.metaballs.remove(mb)
    return _fix(s)


def _fix(s):
    bm = propkit.to_bmesh(s)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    out = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


def tri_count(s):
    return sum(len(f) - 2 for f in s.f)


def decimate(s, tris):
    """Collapse-decimate to about `tris` triangles."""
    n = tri_count(s)
    if n <= tris:
        return s
    ob = _mesh_obj('_t19dec', s)
    m = ob.modifiers.new('d', 'DECIMATE')
    m.decimate_type = 'COLLAPSE'
    m.ratio = tris / n
    m.use_collapse_triangulate = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def remesh(s, voxel):
    """A voxel remesh: overlapping shells fused into one surface."""
    ob = _mesh_obj('_t19rm', s)
    m = ob.modifiers.new('r', 'REMESH')
    m.mode = 'VOXEL'
    m.voxel_size = voxel
    m.adaptivity = 0.0
    m.use_smooth_shade = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return _fix(out)


def vnormals(s):
    nr = [Vector((0, 0, 0)) for _ in s.v]
    for f in s.f:
        a = s.v[f[0]]
        for i in range(1, len(f) - 1):
            n = (s.v[f[i]] - a).cross(s.v[f[i + 1]] - a)
            for k in (f[0], f[i], f[i + 1]):
                nr[k] += n
    return [n.normalized() if n.length > 1e-12 else Vector((0, 1, 0)) for n in nr]


def smooth_shape(s, factor=0.5, iterations=4):
    """Blender's smooth modifier."""
    ob = _mesh_obj('_t19sm', s)
    m = ob.modifiers.new('s', 'SMOOTH')
    m.factor = factor
    m.iterations = iterations
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def bump_along_normals(s, fn):
    """Push every vertex along its normal by fn(p) metres."""
    ns = vnormals(s)
    out = s.copy()
    for i, p in enumerate(out.v):
        out.v[i] = p + ns[i] * fn(p)
    return out


def loft_rings(rings, cap=True, wrap=False):
    """Close a stack of equal rings (lists of points) into a solid; `wrap`
    joins the last ring to the first (a moulding's closed section)."""
    n, seg = len(rings), len(rings[0])
    v = [Vector(p) for r in rings for p in r]
    f = []
    for i in range(n if wrap else n - 1):
        i2 = (i + 1) % n
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, i2 * seg + j2, i2 * seg + j))
    if cap and not wrap:
        f.append(tuple(reversed(range(seg))))
        f.append(tuple((n - 1) * seg + j for j in range(seg)))
    return propkit._fix_winding(Shape(v, f))


_cache = {}


def cached(key, build):
    if key not in _cache:
        state = random.getstate()
        random.seed(zlib.crc32(key.encode()))
        _cache[key] = build()
        random.setstate(state)
    return _cache[key]


def rot_x(p, a):
    """A point turned about x by a (y toward z)."""
    c, s = math.cos(a), math.sin(a)
    return Vector((p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c))


def resample(outline, n):
    """`n` points evenly spaced along a closed outline."""
    pts = [Vector((x, y)) for x, y in outline]
    seg = [(pts[i], pts[(i + 1) % len(pts)]) for i in range(len(pts))]
    lens = [(b - a).length for a, b in seg]
    total = sum(lens)
    out, k, acc = [], 0, 0.0
    for i in range(n):
        t = total * i / n
        while acc + lens[k] < t:
            acc += lens[k]
            k += 1
        a, b = seg[k]
        u = (t - acc) / lens[k] if lens[k] > 1e-12 else 0.0
        p = a.lerp(b, u)
        out.append((p.x, p.y))
    return out


def offset(outline, d):
    """A closed counter-clockwise outline moved in by `d` (mitred)."""
    pts = [Vector((x, y)) for x, y in outline]
    n = len(pts)
    area = sum(pts[i].x * pts[(i + 1) % n].y - pts[(i + 1) % n].x * pts[i].y for i in range(n))
    sgn = 1.0 if area > 0 else -1.0
    out = []
    for i in range(n):
        a, p, b = pts[i - 1], pts[i], pts[(i + 1) % n]
        e0, e1 = (p - a).normalized(), (b - p).normalized()
        n0 = Vector((-e0.y, e0.x)) * sgn
        n1 = Vector((-e1.y, e1.x)) * sgn
        m = (n0 + n1)
        m = m.normalized() if m.length > 1e-9 else n0
        k = 1.0 / max(0.35, m.dot(n0))
        q = p + m * d * k
        out.append((q.x, q.y))
    return out


def moulding(outline, steps, n=160):
    """A moulding round an outline, its face from the outer edge in: rings
    at each (d, z) of `steps` (d in from the edge, z out from the back),
    the last ring capped (a board's field), the back capped."""
    base = resample(outline, n)
    rings = []
    for d, z in steps:
        rings.append([(x, y, z) for x, y in (offset(base, d) if d > 1e-9 else base)])
    return loft_rings(rings)


def ring_sweep(prof, a0=0.0, a1=TAU, n=48):
    """A section `prof` [(r, z), ...] (closed, convex) swept round the z
    axis from angle a0 to a1; a part ring is capped at its ends."""
    full = abs(a1 - a0 - TAU) < 1e-9
    m = n if full else n + 1
    v, f = [], []
    k = len(prof)
    for i in range(m):
        a = a0 + (a1 - a0) * i / n
        c, s = math.cos(a), math.sin(a)
        for r, z in prof:
            v.append((r * c, r * s, z))
    for i in range(n):
        i2 = (i + 1) % m if full else i + 1
        for j in range(k):
            j2 = (j + 1) % k
            f.append((i * k + j, i * k + j2, i2 * k + j2, i2 * k + j))
    if not full:
        f.append(tuple(reversed(range(k))))
        f.append(tuple(n * k + j for j in range(k)))
    return propkit._fix_winding(Shape(v, f))


def rrect_section(r0, r1, z0, z1, rr, seg=2):
    """A rounded rectangle section (r across, z along) for `ring_sweep`."""
    return [(r, z) for r, z in round_outline([(r0, z0), (r1, z0), (r1, z1), (r0, z1)], rr, seg)]


def strip(pts, w, t):
    """A flat strip w wide (along x) and t thick swept along a path in the
    yz plane (a bent brass strip)."""
    P = [Vector(p) for p in pts]
    n = len(P)
    v, f = [], []
    for i in range(n):
        tg = (P[min(i + 1, n - 1)] - P[max(i - 1, 0)]).normalized()
        nr = Vector((0, -tg.z, tg.y))
        for sx, sn in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            v.append(P[i] + Vector((sx * w / 2, 0, 0)) + nr * sn * t / 2)
    for i in range(n - 1):
        for j in range(4):
            j2 = (j + 1) % 4
            f.append((i * 4 + j, i * 4 + j2, (i + 1) * 4 + j2, (i + 1) * 4 + j))
    f.append((3, 2, 1, 0))
    k = (n - 1) * 4
    f.append((k, k + 1, k + 2, k + 3))
    return propkit._fix_winding(Shape(v, f))


def place_on(s, p, normal):
    """`s` (modelled along +y) stood on point p, its +y along `normal`."""
    q = Vector((0, 1, 0)).rotation_difference(Vector(normal).normalized())
    s = s.copy()
    s.v = [q @ v + Vector(p) for v in s.v]
    return propkit._fix_winding(s)


# ============================================================ reliquary ===
# A small reliquary casket of gilt brass, French or Italian, of the later
# 19th century (brass reliquary caskets sold: 14 x 7 x 6 cm, 15 x 6.5 x 10
# cm): four cast bun feet under a stepped plinth, a glazed body between
# four turned corner columns (bevelled glass on all four sides), a cornice,
# a hipped lid with a cast cresting along its ridge and a small cross
# finial; paste cabochons in collets on the lid's slopes. Inside, on a
# cushion of red silk velvet: the relic (a fragment of bone), its paper
# label and the red wax seal of its authentication. Frame: the casket's
# middle on the floor (y = 0), its front toward +z.
RQ_W, RQ_D = 0.140, 0.090          # the plinth's footprint
RQ_FOOT = 0.009                    # the feet's height
RQ_PLINTH = 0.019                  # the plinth's top (the body's floor)
RQ_BODY = 0.064                    # the body's top
RQ_CORNICE = 0.071                 # the cornice's top (the lid's foot)
RQ_RIDGE = 0.089                   # the lid's ridge
BW, BD = 0.124, 0.074              # the glazed body's outside
LID0 = (0.130, 0.080)              # the lid at its foot and at its ridge
LID1 = (0.070, 0.022)
# the cabochons, front slope (z > 0) and back: x, and how far up the slope
RQ_TILT = math.atan2(RQ_RIDGE - RQ_CORNICE, (LID0[1] - LID1[1]) / 2)   # the slope's normal from +y
STONES = [(-0.034, -1), (-0.034, 1), (0.034, -1), (0.034, 1)]           # the old gems' order
STONE_UP = 0.5                     # half way up the slope


def stone_place(x, side, up=STONE_UP):
    """Where a stone sits on the lid's slope, and the slope's normal."""
    y = RQ_CORNICE + (RQ_RIDGE - RQ_CORNICE) * up
    z = side * (LID0[1] / 2 + (LID1[1] / 2 - LID0[1] / 2) * up)
    n = Vector((0, math.cos(RQ_TILT), side * math.sin(RQ_TILT)))
    return Vector((x, y, z)), n


def rq_lid():
    (w0, d0), (w1, d1) = LID0, LID1
    y0, y1 = RQ_CORNICE, RQ_RIDGE
    v = [(-w0 / 2, y0, -d0 / 2), (w0 / 2, y0, -d0 / 2), (w0 / 2, y0, d0 / 2), (-w0 / 2, y0, d0 / 2),
         (-w1 / 2, y1, -d1 / 2), (w1 / 2, y1, -d1 / 2), (w1 / 2, y1, d1 / 2), (-w1 / 2, y1, d1 / 2)]
    f = [(0, 1, 2, 3), (4, 7, 6, 5), (0, 4, 5, 1), (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)]
    return bevel(propkit._fix_winding(Shape(v, f)), 0.0007, seg=1)


def rq_column():
    return lathe([(0, 0), (0.0036, 0), (0.0036, 0.0018), (0.0028, 0.0030), (0.0024, 0.0050), (0.0023, 0.0230),
                  (0.0024, 0.0400), (0.0029, 0.0425), (0.0036, 0.0435), (0.0036, 0.0450), (0, 0.0450)], seg=10)


def reliquary_brass():
    s = Shape()
    # the feet: cast buns on a collar
    foot = lathe([(0, 0), (0.0040, 0), (0.0062, 0.0015), (0.0070, 0.0040), (0.0062, 0.0066), (0.0044, 0.0080),
                  (0.0040, 0.0090), (0, 0.0090)], seg=12)
    for sx in (-1, 1):
        for sz in (-1, 1):
            s = s + foot.copy().move(sx * (RQ_W / 2 - 0.011), 0, sz * (RQ_D / 2 - 0.011))
    # the plinth: two steps and a fillet
    s = s + rbox(RQ_W, 0.0045, RQ_D, r=0.0012, seg=2).move(0, RQ_FOOT + 0.00225, 0)
    s = s + rbox(RQ_W - 0.007, 0.0035, RQ_D - 0.007, r=0.0010, seg=2).move(0, RQ_FOOT + 0.0045 + 0.00175, 0)
    s = s + rbox(BW + 0.004, 0.0020, BD + 0.004, r=0.0005, seg=1).move(0, RQ_PLINTH - 0.0010, 0)
    # the columns at the corners
    col = rq_column()
    for sx in (-1, 1):
        for sz in (-1, 1):
            s = s + col.copy().move(sx * (BW / 2 - 0.0036), RQ_PLINTH, sz * (BD / 2 - 0.0036))
    # the glazing bars: a rail round the foot and the head of the glass
    for y, h in ((RQ_PLINTH + 0.00175, 0.0035), (RQ_BODY - 0.0020, 0.0040)):
        for sz in (-1, 1):
            s = s + rbox(BW - 0.010, h, 0.0030, r=0.0006, seg=1).move(0, y, sz * (BD / 2 - 0.0015))
        for sx in (-1, 1):
            s = s + rbox(0.0030, h, BD - 0.010, r=0.0006, seg=1).move(sx * (BW / 2 - 0.0015), y, 0)
    # the cornice
    s = s + rbox(BW + 0.008, 0.0030, BD + 0.008, r=0.0008, seg=1).move(0, RQ_BODY + 0.0015, 0)
    s = s + rbox(BW + 0.012, 0.0040, BD + 0.012, r=0.0012, seg=2).move(0, RQ_BODY + 0.0050, 0)
    # the lid, its ridge, the cresting and the finial cross
    s = s + rq_lid()
    s = s + rbox(LID1[0] + 0.004, 0.0030, LID1[1] + 0.003, r=0.0008, seg=1).move(0, RQ_RIDGE + 0.0015, 0)
    for k in range(-3, 4):
        if k == 0:
            continue
        x = k * 0.0095
        s = s + cyl(0.0007, h=0.0030, seg=6, bevel_r=0).move(x, RQ_RIDGE + 0.0045, 0)
        s = s + sphere(0.0019, seg=8, rings=5).move(x, RQ_RIDGE + 0.0070, 0)
        # a leaf either side of each knop
        s = s + ellipsoid(0.0026, 0.0011, 0.0008, seg=8).rot(0, 0, 0.6).move(x - 0.0026, RQ_RIDGE + 0.0042, 0)
        s = s + ellipsoid(0.0026, 0.0011, 0.0008, seg=8).rot(0, 0, -0.6).move(x + 0.0026, RQ_RIDGE + 0.0042, 0)
    s = s + sphere(0.0034, seg=10, rings=6).move(0, RQ_RIDGE + 0.0062, 0)
    s = s + rbox(0.0030, 0.0170, 0.0030, r=0.0006, seg=1).move(0, RQ_RIDGE + 0.0170, 0)
    s = s + rbox(0.0110, 0.0030, 0.0030, r=0.0006, seg=1).move(0, RQ_RIDGE + 0.0205, 0)
    for x, y in ((0, RQ_RIDGE + 0.0258), (-0.0056, RQ_RIDGE + 0.0205), (0.0056, RQ_RIDGE + 0.0205)):
        s = s + sphere(0.0014, seg=8, rings=5).move(x, y, 0)
    # the collets the stones are set in
    collet = lathe([(0, 0), (0.0050, 0), (0.0052, 0.0012), (0.0046, 0.0019), (0.0042, 0.0012), (0, 0.0012)], seg=14)
    big = lathe([(0, 0), (0.0068, 0), (0.0070, 0.0013), (0.0063, 0.0021), (0.0058, 0.0013), (0, 0.0013)], seg=16)
    for x, side in STONES:
        p, n = stone_place(x, side)
        s = s + place_on(collet, p, n)
    p, n = stone_place(0.0, 1)
    s = s + place_on(big, p, n)
    # the keyhole escutcheon on the front's rail, and the hinges behind
    s = s + rbox(0.0070, 0.0034, 0.0006, r=0.0003, seg=1).move(0, RQ_BODY + 0.0050, BD / 2 + 0.0063)
    for sx in (-1, 1):
        s = s + cyl(0.0012, h=0.012, seg=8, bevel_r=0.0003).rot(0, 0, math.pi / 2).move(sx * 0.035, RQ_CORNICE - 0.0005, -LID0[1] / 2 - 0.0008)
    return s


def reliquary_stone():
    """A paste cabochon, its base at y = 0, its dome up +y."""
    return lathe([(0, 0), (0.0042, 0), (0.0042, 0.0005), (0.0037, 0.0016), (0.0026, 0.0025), (0.0012, 0.0030), (0, 0.0031)], seg=16)


def reliquary_stone_big():
    return reliquary_stone().scale(1.35, 1.15, 1.35)


def reliquary_glass():
    gh = RQ_BODY - RQ_PLINTH - 0.0040
    y = RQ_PLINTH + 0.0035 + gh / 2 - 0.0015
    s = Shape()
    for sz in (-1, 1):
        s = s + box(BW - 0.0090, gh, 0.0014).move(0, y, sz * (BD / 2 - 0.0030))
    for sx in (-1, 1):
        s = s + box(0.0014, gh, BD - 0.0090).move(sx * (BW / 2 - 0.0030), y, 0)
    return s


def reliquary_cushion():
    """A cushion of silk velvet over the body's floor, plump in its middle,
    pressed down where the relic lies."""
    a, b = (BW - 0.014) / 2, (BD - 0.014) / 2
    s = subdivide(rbox(2 * a, 0.0080, 2 * b, r=0.0030, seg=2), 1)
    def puff(p):
        if p.y > 0:
            k = max(0.0, 1 - (p.x / a) ** 2) * max(0.0, 1 - (p.z / b) ** 2)
            dent = math.exp(-((p.x - 0.004) / 0.020) ** 2 - ((p.z + 0.002) / 0.008) ** 2)
            return Vector((p.x, p.y + 0.0030 * k - 0.0012 * dent, p.z))
        return p
    s.warp(puff)
    return s.move(0, RQ_PLINTH + 0.0040 + 0.0002, 0)


CUSHION_TOP = RQ_PLINTH + 0.0080 + 0.0002


def reliquary_relic():
    """A fragment of a long bone's shaft, 3.4 cm, broken off at both ends
    (the breaks ragged), lying across the cushion."""
    rings = []
    n = 12
    for i, x in enumerate([-0.0170, -0.0160, -0.0120, -0.0060, 0.0, 0.0060, 0.0120, 0.0160, 0.0172]):
        r = [0.0018, 0.0040, 0.0047, 0.0045, 0.0043, 0.0044, 0.0047, 0.0042, 0.0016][i]
        ring = []
        for j in range(n):
            a = TAU * j / n
            rr = r * (1.0 + 0.10 * math.sin(3 * a + i) + 0.06 * math.cos(5 * a))
            # the breaks: the ends jagged along x
            dx = 0.0012 * math.sin(4 * a + 1.3 * i) if i in (0, 1, 7, 8) else 0.0
            ring.append((x + dx, rr * math.sin(a) * 0.85, rr * math.cos(a)))
        rings.append(ring)
    s = loft_rings(rings)
    s = noisy(s, 0.0003, scale=400.0, seed=3)
    lo, hi = s.bounds()
    s.move(0, -lo.y, 0)
    return s.rot(0, 0.22, 0).move(0.004, CUSHION_TOP - 0.0014, -0.002)


def reliquary_label():
    """Its label (a cedula), a strip of paper curled a little, lying in
    front of it."""
    s = slab(0.030, 0.0085, 0.0003, nx=10, ny=3).rot(-math.pi / 2, 0, 0)
    s = bend(s, 0.5, axis='z', along='x')
    lo, hi = s.bounds()
    s.move(-(lo.x + hi.x) / 2, CUSHION_TOP - lo.y - 0.0004, 0)
    return s.rot(0, -0.08, 0).move(-0.006, 0, 0.016)


def reliquary_seal():
    """The red wax seal of its authentication, pressed with a die."""
    s = cyl(0.0058, 0.0062, 0.0020, seg=16, bevel_r=0.0008)
    s = noisy(s, 0.0003, scale=400.0, seed=5)
    s = cut(s, cyl(0.0038, h=0.0020, seg=16, bevel_r=0).move(0, 0.0016, 0))
    return s.move(0.034, CUSHION_TOP - 0.0002, -0.017)


auto('Real__reliquaryBrass', reliquary_brass, 7000, smooth=40.0)
auto('Real__reliquaryGlass', reliquary_glass, 120)
auto('Real__reliquaryStone', reliquary_stone, 200, smooth=60.0)
auto('Real__reliquaryStoneBig', reliquary_stone_big, 200, smooth=60.0)
auto('Real__reliquaryCushion', reliquary_cushion, 600, smooth=60.0)
auto('Real__reliquaryRelic', reliquary_relic, 700, smooth=50.0)
auto('Real__reliquaryLabel', reliquary_label, 200, smooth=60.0)
auto('Real__reliquarySeal', reliquary_seal, 400, smooth=40.0)


# ============================================================ crucifix ===
# A household wall crucifix of the common kind (sold at 10-12 in): a cross
# of two boards half-lapped, 28 x 16 cm and 16 mm thick, its face's edges
# chamfered, its ends cut square; a cast metal corpus (spelter, once
# gilt), 13 cm from its hands to its feet, nailed on; a small titulus
# above it. Hung from a nail through a hole behind its head. Frame: the
# upright's middle (the old cross's centre), the wall at z = -0.014.
CR_BACK = -0.013
CR_T = 0.016
CR_UP = (0.028, 0.280)
CR_BAR = (0.160, 0.026, 0.050)
CR_FRONT = CR_BACK + CR_T


def crucifix_cross():
    up = prism(rounded_rect(CR_UP[0], CR_UP[1], 0.0012, 1), CR_T, bevel_r=0.0022, bevel_seg=1)
    bar = prism(rounded_rect(CR_BAR[0], CR_BAR[1], 0.0012, 1), CR_T, bevel_r=0.0022, bevel_seg=1).move(0, CR_BAR[2], 0)
    s = boolean(up, bar, 'UNION')
    # knocks and the grain raised by years of dusting
    s = noisy(s, 0.00018, scale=180.0, seed=7)
    return s.move(0, 0, CR_BACK + CR_T / 2)


def crucifix_corpus():
    z = CR_FRONT + 0.0062
    e = []
    # the head, bowed to its right and forward, the hair falling, the beard
    e += [('ell', (-0.0040, 0.0440, z + 0.0018), (0.0066, 0.0082, 0.0068), 2.0, (0.25, 0, 0.30)),
          ('ell', (-0.0058, 0.0392, z + 0.0042), (0.0040, 0.0042, 0.0036), 2.0),
          ('ell', (-0.0005, 0.0418, z - 0.0012), (0.0074, 0.0090, 0.0052), 2.0, (0, 0, 0.30))]
    # the neck, the chest, the ribs' cage, the belly
    e += [('cap', (-0.0010, 0.0345, z), (-0.0032, 0.0395, z + 0.0012), 0.0032),
          ('ell', (0.0, 0.0250, z), (0.0115, 0.0115, 0.0062)),
          ('ell', (0.0, 0.0120, z - 0.0004), (0.0085, 0.0100, 0.0056)),
          ('ell', (0.0, 0.0040, z - 0.0006), (0.0076, 0.0060, 0.0050))]
    # the hands nailed near the crossbar's ends (the arms are swept below)
    for sx in (-1, 1):
        e += [('ell', (sx * 0.0535, 0.0570, z - 0.0015), (0.0048, 0.0026, 0.0020), 2.0, (0, 0, sx * 0.5)),
              ('ell', (sx * 0.0100, 0.0310, z), (0.0050, 0.0042, 0.0045), 2.0)]
    # the loincloth, its knot and fall at one hip
    e += [('ell', (0.0, -0.0040, z + 0.0004), (0.0102, 0.0062, 0.0066)),
          ('ell', (0.0085, -0.0065, z + 0.0010), (0.0032, 0.0070, 0.0036))]
    # the legs, the knees a little bent, the right foot over the left
    for sx in (-1, 1):
        e += [('cap', (sx * 0.0046, -0.0070, z), (sx * 0.0040, -0.0320, z + 0.0018), 0.0042),
              ('cap', (sx * 0.0040, -0.0320, z + 0.0018), (sx * 0.0013, -0.0600, z - 0.0004), 0.0032)]
    e += [('cap', (0.0012, -0.0600, z), (-0.0004, -0.0700, z - 0.0010), 0.0024),
          ('cap', (-0.0012, -0.0598, z - 0.0016), (0.0004, -0.0695, z - 0.0026), 0.0023)]
    s = meta(e, res=0.0006)
    # the arms: smooth tubes from the shoulders up to the hands, the elbows
    # sagging a little, the forearms thinner
    for sx in (-1, 1):
        pts = [(sx * 0.0090, 0.0315, z), (sx * 0.0200, 0.0372, z - 0.0002), (sx * 0.0300, 0.0418, z - 0.0005),
               (sx * 0.0400, 0.0485, z - 0.0009), (sx * 0.0500, 0.0552, z - 0.0013)]
        s = s + tube(pts, 0.0036, seg=10, smooth=2, r_end=0.0024)
    s = remesh(s, 0.0005)
    s = smooth_shape(s, 0.6, 6)
    s = bump_along_normals(s, lambda p: 0.00008 * noise.noise(Vector((p.x * 900, p.y * 900, p.z * 900))))
    s = decimate(s, 3100)
    # the nails' heads in the hands and the feet
    for sx in (-1, 1):
        s = s + cyl(0.0013, h=0.0010, seg=8, bevel_r=0.0003).rot(math.pi / 2, 0, 0).move(sx * 0.0535, 0.0575, z + 0.0012)
    s = s + cyl(0.0013, h=0.0010, seg=8, bevel_r=0.0003).rot(math.pi / 2, 0, 0).move(0.0, -0.0665, z + 0.0008)
    # flattened behind where it was cast against the mould's back
    s = cut(s, box(0.2, 0.2, 0.02).move(0, 0, CR_FRONT - 0.01))
    return s


def crucifix_title():
    """The titulus: a small scroll of tin, nailed above the head."""
    s = slab(0.026, 0.0115, 0.0008, nx=8, ny=2)
    s.warp(lambda p: Vector((p.x, p.y, p.z + 0.0018 * (abs(p.x) / 0.013) ** 3)))
    return s.move(0, 0.0895, CR_FRONT + 0.0006)


auto('Real__crucifixCross', crucifix_cross, 900, smooth=30.0)
auto("Real__crucifixCorpus", crucifix_corpus, 3600, smooth=60.0)
auto('Real__crucifixTitle', crucifix_title, 120, smooth=50.0)


# ========================================================= skull plaque ===
# A roe deer's skull, boiled and bleached, on a carved oak shield (or an
# oval board): the European mount, the whole skull bar the jaw, its palate
# to the board, its nose down, its frontal to the room. A roe's skull: 190
# mm long (condylobasal), 83.5 mm across the zygomatic arches, 52 mm
# between the orbits, the nasals 54 mm (an adult buck, skullsite.co.uk);
# a buck's carries its pedicles and antlers (batch t11's), a doe's none.
# Frame: the board's middle on the wall at z = SP_BACK (the old plaque's
# centre, the game hangs it 1.70 m up).
SP_BACK = -0.006
SP_SHIELD = (0.180, 0.250, 0.020)
SP_FIELD = 0.004
SP_TURN = math.pi / 2                       # H (the skull) to the plaque: nose down, frontal out
ROE_BETA = math.radians(52.0)               # the antlers' lean back off the frontal (t11's)
ROE_PED_X = 0.020
SP_FRONT = SP_BACK + SP_SHIELD[2] + SP_FIELD - 0.001
SP_PALATE = -0.0705                         # the skull's lowest in H (the cheek teeth)
SP_O = Vector((0.0, 0.072, SP_FRONT - SP_PALATE + 0.0008))   # H's origin (the burrs' bases) on the plaque


def shield_outline(w, h, notch=0.006, r=0.004):
    """A hunting trophy's shield (t11's heater): rounded shoulders, a small
    peak at the middle of its top, straight sides, two arcs of radius w
    meeting at its foot's point (y = -h/2)."""
    hw, top, bot = w / 2, h / 2, -h / 2
    ys = bot + math.sqrt(w * w - hw * hw)
    right = [(hw, top - 0.016)]
    for k in range(0, 9):
        a = -math.radians(60.0) * k / 8
        right.append((-hw + w * math.cos(a), ys + w * math.sin(a)))
    right[-1] = (0.0, bot)
    pts = [(-0.024, top), (0.0, top + notch), (0.024, top), (hw - 0.016, top)] + right
    pts += [(-x, y) for x, y in reversed(right[:-1])] + [(-hw + 0.016, top)]
    out, n = [], len(pts)
    for i in range(n):
        a, p, b = Vector(pts[i - 1]), Vector(pts[i]), Vector(pts[(i + 1) % n])
        turn = (p - a).normalized().angle((b - p).normalized(), 0.0)
        out += propkit._round_corner(pts[i], pts[i - 1], pts[(i + 1) % n], r, 2) if turn > 0.3 else [pts[i]]
    return out


def skull_shield():
    w, h, t = SP_SHIELD
    board = prism(shield_outline(w, h), t, bevel_r=0.0035, bevel_seg=2)
    board.move(0, 0, t / 2)
    inner = [(x * (w - 0.028) / w, y * (h - 0.028) / h) for x, y in shield_outline(w, h)]
    raised = prism(inner, SP_FIELD + 0.002, bevel_r=0.0016, bevel_seg=1)
    raised.move(0, 0, t + SP_FIELD / 2 - 0.001)
    return (board + raised).move(0, 0, SP_BACK)


def skull_oval():
    """An oval board of oak, its back flat, an ogee routed round its face."""
    a, b = SP_SHIELD[0] / 2, 0.120
    t = SP_SHIELD[2] + SP_FIELD
    steps = [(0.0, 0.0), (0.0, t - 0.0085), (0.0010, t - 0.0062), (0.0032, t - 0.0046), (0.0052, t - 0.0034),
             (0.0064, t - 0.0018), (0.0086, t - 0.0004), (0.0110, t)]
    rings = []
    n = 72
    for d, z in steps:
        rings.append([((a - d) * math.cos(TAU * j / n), (b - d) * math.sin(TAU * j / n), z + SP_BACK - 0.001) for j in range(n)])
    return loft_rings(rings)


def roe_skull_h(buck):
    """A roe's skull in its own frame H (x across, y up from the frontal,
    z toward the nose; the pedicles' tops, where the burrs sit, at the
    origin, as t11's skull plate): the braincase and its crests, the
    frontals over the orbits (their rims standing out), the nasals and the
    long maxillae to the premaxillae (a deer has no upper incisors), the
    ethmoid gap and the lachrymal pit before each orbit, the nasal
    opening; the pedicles (a buck's) on the frontals."""
    e = [('ell', (0, -0.033, -0.022), (0.030, 0.027, 0.036), 2.0),        # the braincase
         ('ell', (0, -0.042, -0.050), (0.024, 0.022, 0.012), 2.0),        # the occiput
         ('ell', (0, -0.019, 0.006), (0.031, 0.013, 0.034), 2.0),         # the frontals
         ('cap', (0, -0.016, -0.040), (0, -0.014, 0.020), 0.006, 2.0),    # the sagittal ridge, low
         ('ell', (0, -0.040, 0.068), (0.024, 0.024, 0.040), 2.0),         # the maxillae
         ('cap', (0, -0.024, 0.036), (0, -0.034, 0.104), 0.0105, 2.0),    # the nasals
         ('cap', (0, -0.040, 0.100), (0, -0.050, 0.128), 0.0090, 2.0),    # the premaxillae
         ('ell', (0, -0.058, 0.060), (0.021, 0.010, 0.044), 2.0)]         # the palate's sides
    for sx in (-1, 1):
        # the orbit's rim: a thick bony boss round the eye, standing out
        # from the frontal, the socket open to the side and a little forward
        e += [('ell', (sx * 0.0330, -0.0320, 0.0250), (0.0110, 0.0125, 0.0150), 2.0),
              ('cap', (sx * 0.0300, -0.0200, 0.0120), (sx * 0.0380, -0.0240, 0.0300), 0.0050, 2.0)]
        e += [('cap', (sx * 0.027, -0.047, 0.030), (sx * 0.028, -0.048, -0.008), 0.0052, 2.0),   # the zygomatic arch
              ('cap', (sx * 0.020, -0.042, 0.044), (sx * 0.024, -0.052, 0.098), 0.0090, 2.0),    # the maxilla's side
              ('cap', (sx * 0.006, -0.056, 0.112), (sx * 0.007, -0.056, 0.132), 0.0048, 2.0),    # the premaxilla's tip
              ('-ell', (sx * 0.0465, -0.0340, 0.0280), (0.0090, 0.0115, 0.0120), 2.0),             # the orbit
              ('-ell', (sx * 0.020, -0.023, 0.055), (0.0055, 0.0060, 0.0085), 2.0),              # the ethmoid gap
              ('-ell', (sx * 0.031, -0.038, 0.052), (0.0050, 0.0055, 0.0070), 2.0)]              # the lachrymal pit
        if buck:
            e += [('cap', (sx * ROE_PED_X, -0.017, 0.010), (sx * ROE_PED_X, -0.0015, 0.001), 0.0085, 2.0)]
    e += [('-ell', (0, -0.048, 0.134), (0.0075, 0.0100, 0.0160), 2.0),      # the nasal opening
          ('-cap', (0, -0.006, -0.010), (0, -0.010, 0.070), 0.0022, 2.0)]    # the parting between the frontals
    s = meta(e, res=0.0011)
    s = remesh(s, 0.0010)
    # the palate flat (the board), the foramen magnum's notch behind
    s = cut(s, box(0.2, 0.1, 0.3).move(0, -0.068 - 0.05, 0))
    s = cut(s, cyl(0.0085, h=0.04, seg=12, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, -0.050, -0.062))
    # the sutures: shallow grooves wandering across the frontals and the nasals
    def sut(p):
        # the coronal suture behind the pedicles, the frontonasal ahead of
        # the orbits, the midline between the frontals and the nasals
        d1 = abs(p.z + 0.010 + 0.004 * math.sin(p.x * 260))
        d2 = abs(p.z - 0.040 - 0.003 * math.sin(p.x * 300))
        d3 = abs(p.x + 0.0012 * math.sin(p.z * 420))
        d = min(d1, d2, d3 if p.z > -0.04 else 1.0)
        g = -0.00035 * max(0.0, 1.0 - d / 0.0010) if p.y > -0.045 else 0.0
        return g + 0.00018 * noise.noise(Vector((p.x * 600, p.y * 600, p.z * 600)))
    s = bump_along_normals(s, sut)
    return decimate(s, 3600)


def _h_to_plaque(s):
    s = s.copy()
    s.v = [rot_x(p, SP_TURN) + SP_O for p in s.v]
    return propkit._fix_winding(s)


def skull_doe():
    return _h_to_plaque(cached('roe_doe', lambda: roe_skull_h(False)))


def skull_buck():
    return _h_to_plaque(cached('roe_buck', lambda: roe_skull_h(True)))


def skull_teeth():
    """The upper cheek teeth, three premolars and three molars a side,
    their crowns worn flat, stained dark at the grinding faces."""
    s = Shape()
    for sx in (-1, 1):
        z = 0.046
        for k, (L, W) in enumerate([(0.0085, 0.0065), (0.0090, 0.0070), (0.0095, 0.0080), (0.0110, 0.0100), (0.0120, 0.0105), (0.0115, 0.0100)]):
            t = rbox(W, 0.0060, L, r=0.0012, seg=1)
            t = noisy(t, 0.0004, scale=500.0, seed=k + (5 if sx > 0 else 0))
            s = s + t.move(sx * (0.017 + 0.0007 * k), SP_PALATE + 0.0030, z + L / 2)
            z += L + 0.0004
    return _h_to_plaque(s)


def skull_plate():
    """A small brass plate screwed to the board's foot."""
    s = rbox(0.042, 0.012, 0.0010, r=0.0004, seg=1)
    for sx in (-1, 1):
        s = s + screw_head(0.0011, 0.0006).rot(math.pi / 2, 0, 0).move(sx * 0.0175, 0, 0.0005)
    return s.move(0, -0.092, SP_BACK + SP_SHIELD[2] + SP_FIELD + 0.0004)


auto('Real__skullPlaqueShield', skull_shield, 900, smooth=40.0)
auto('Real__skullPlaqueOval', skull_oval, 1200, smooth=40.0)
auto('Real__skullPlaqueDoe', skull_doe, 3800, smooth=55.0)
auto('Real__skullPlaqueBuck', skull_buck, 3800, smooth=55.0)
auto('Real__skullPlaqueTeeth', skull_teeth, 700, smooth=40.0)
auto('Real__skullPlaquePlate', skull_plate, 260, smooth=30.0)


# ============================================================ the crest ===
# A carved armorial plaque (a carved and painted shield of arms, as were
# sold carved to order, 34 x 30 cm and 4 cm deep the one of 1820-40):
# oak, a heater shield 33.6 x 41.6 cm (the old frame's size), its edge
# moulded with a bead and a cove into the field, the field gessoed and
# painted, the ordinary (a chevron, a bend or a cross) carved proud of it
# and painted, the bead gilt. Frame: the shield's middle, its back on the
# wall at z = CREST_BACK.
CREST_BACK = -0.0065
CREST_W, CREST_H = 0.336, 0.416
CREST_STEPS = [(0.0, 0.0), (0.0, 0.0150), (0.0012, 0.0180), (0.0035, 0.0200), (0.0065, 0.0212), (0.0095, 0.0208),
               (0.0120, 0.0192), (0.0140, 0.0172), (0.0165, 0.0162), (0.0190, 0.0160)]
CREST_FIELD_IN = 0.0190
CREST_FIELD_Z = 0.0160
CREST_RELIEF = 0.0030


def crest_outline():
    """The heater, counter-clockwise from its point: two arcs up from the
    point to the middle of the sides, the sides straight, the top flat,
    its two corners rounded."""
    hw, top, bot = CREST_W / 2, CREST_H / 2, -CREST_H / 2
    R = (hw * hw + bot * bot) / (2 * hw)
    cx = hw - R
    a_end = math.atan2(bot, -cx)
    rc = 0.022
    right = [(cx + R * math.cos(a_end * (1 - k / 14)), R * math.sin(a_end * (1 - k / 14))) for k in range(15)]
    right[0] = (0.0, bot)
    right.append((hw, top - rc))
    right += [(hw - rc + rc * math.cos(math.pi / 2 * k / 6), top - rc + rc * math.sin(math.pi / 2 * k / 6)) for k in range(1, 7)]
    left = [(-x, y) for x, y in reversed(right)]
    return right + left[:-1]


def crest_board():
    return moulding(crest_outline(), CREST_STEPS, n=200).move(0, 0, CREST_BACK)


def crest_field_outline():
    return offset(resample(crest_outline(), 200), CREST_FIELD_IN)


def crest_field():
    s = prism(crest_field_outline(), 0.0008)
    return s.move(0, 0, CREST_BACK + CREST_FIELD_Z + 0.0002)


def _ordinary(poly):
    clip = prism(offset(resample(crest_outline(), 200), CREST_FIELD_IN + 0.0015), 0.02)
    s = boolean(prism(poly, CREST_RELIEF), clip, 'INTERSECT')
    s = bevel(s, 0.0009, seg=1)
    s = noisy(s, 0.00012, scale=200.0, seed=11)
    return s.move(0, 0, CREST_BACK + CREST_FIELD_Z + 0.0006 + CREST_RELIEF / 2)


def crest_chevron():
    w = 0.052
    ax, ay = 0.0, 0.070
    k = 1.0 / math.cos(math.radians(40))
    dy = math.tan(math.radians(40))
    pts = [(ax, ay), (0.3, ay - 0.3 * dy), (0.3, ay - 0.3 * dy - w * k), (ax, ay - w * k), (-0.3, ay - 0.3 * dy - w * k), (-0.3, ay - 0.3 * dy)]
    return _ordinary(pts)


def crest_bend():
    w = 0.060
    d = Vector((1, -1.15)).normalized()
    n = Vector((-d.y, d.x))
    c = Vector((0.0, 0.020))
    pts = [c - d * 0.4 - n * w / 2, c + d * 0.4 - n * w / 2, c + d * 0.4 + n * w / 2, c - d * 0.4 + n * w / 2]
    return _ordinary([(p.x, p.y) for p in pts])


def crest_cross():
    w = 0.050
    cy = 0.030
    pts = [(-w / 2, 0.3), (w / 2, 0.3), (w / 2, cy + w / 2), (0.3, cy + w / 2), (0.3, cy - w / 2), (w / 2, cy - w / 2),
           (w / 2, -0.3), (-w / 2, -0.3), (-w / 2, cy - w / 2), (-0.3, cy - w / 2), (-0.3, cy + w / 2), (-w / 2, cy + w / 2)]
    return _ordinary(pts)


auto('Real__crestBoard', crest_board, 4200, smooth=40.0)
auto('Real__crestField', crest_field, 900, smooth=30.0)
auto('Real__crestChevron', crest_chevron, 400, smooth=30.0)
auto('Real__crestBend', crest_bend, 300, smooth=30.0)
auto('Real__crestCross', crest_cross, 500, smooth=30.0)


# ========================================================= silhouette ===
# A cut-paper silhouette (a bust in profile cut from black paper, laid on
# cream card) in a small oval frame of the 1830s-60s: a turned moulding
# of pearwood (ebonised, stained, or gilt), a pressed gilt-brass slip
# round the sight, glass, a brass ring at its top for its nail (oval
# silhouettes framed 14 x 11.5 to 18 x 22 cm). 18 x 22 cm outside (the old
# frame's), the sight 13.8 x 17.8 cm, the slip's window 12.6 x 16.6 cm.
# Frame: the frame's middle, its back on the wall at z = 0 (as t01's).
SIL_A, SIL_B = 0.090, 0.110
SIL_SECTION = [(0.0, 0.0), (0.0, 0.0100), (0.0015, 0.0135), (0.0040, 0.0152), (0.0070, 0.0156), (0.0100, 0.0148),
               (0.0130, 0.0126), (0.0155, 0.0129), (0.0175, 0.0140), (0.0195, 0.0131), (0.0210, 0.0110),
               (0.0210, 0.0045), (0.0170, 0.0045), (0.0170, 0.0)]
SIL_FACE = [(d, z) for d, z in SIL_SECTION[1:11]]
SIL_SIGHT = 0.021
SIL_SLIP = 0.027
SIL_RING_Y = SIL_B + 0.0060


def oval_at(a, b, d, n):
    return [((a - d) * math.cos(TAU * j / n), (b - d) * math.sin(TAU * j / n)) for j in range(n)]


def silhouette_frame():
    n = 96
    rings = [[(x, y, z) for x, y in oval_at(SIL_A, SIL_B, d, n)] for d, z in SIL_SECTION]
    return loft_rings(rings, cap=False, wrap=True)


def silhouette_slip():
    """The pressed brass slip on the glass, round the sight, a bead at
    its window."""
    n = 96
    sec = [(SIL_SIGHT - 0.0004, 0.0045), (SIL_SIGHT - 0.0004, 0.0052), (0.0235, 0.0060), (0.0255, 0.0057),
           (SIL_SLIP, 0.0050), (SIL_SLIP, 0.0045)]
    rings = [[(x, y, z) for x, y in oval_at(SIL_A, SIL_B, d, n)] for d, z in sec]
    return loft_rings(rings, cap=False, wrap=True)


def silhouette_glass():
    n = 48
    rings = [[(x, y, z) for x, y in oval_at(SIL_A, SIL_B, 0.0175, n)] for z in (0.0031, 0.0044)]
    return loft_rings(rings)


def silhouette_card():
    n = 64
    rings = [[(x, y, z) for x, y in oval_at(SIL_A, SIL_B, 0.0180, n)] for z in (0.0026, 0.0030)]
    return loft_rings(rings)


def silhouette_ring():
    """A brass ring through the screw eye at the top."""
    ring = torus(0.0058, 0.0011, seg=20, tseg=6).move(0, SIL_RING_Y, 0.0090)
    eye = torus(0.0022, 0.0008, seg=12, tseg=5).rot(0, math.pi / 2, 0).move(0, SIL_B + 0.0005, 0.0090)
    # the screw eye's boss, let into the frame's top
    boss = cyl(0.0025, h=0.004, seg=8, bevel_r=0.0005).move(0, SIL_B - 0.0008, 0.0090)
    return ring + eye + boss


auto('Real__silhouetteFrame', silhouette_frame, 2800, smooth=40.0)
auto('Real__silhouetteSlip', silhouette_slip, 1200, smooth=40.0)
auto('Real__silhouetteGlass', silhouette_glass, 300, smooth=40.0)
auto('Real__silhouetteCard', silhouette_card, 300, smooth=40.0)
auto('Real__silhouetteRing', silhouette_ring, 500, smooth=50.0)


# ===================================================== where it hung ===
# A picture hook (the common single-pin kind: a strip of brass-plated
# steel 26 x 6 mm, its top rolled round a hardened pin driven down into
# the wall at 45 degrees, its foot bent out and up into the lip the wire
# hangs on), the pin's knurled head, and the end of a stranded picture
# wire still caught on one of them. Frame: the hook's lip (where a wire
# bears) at the origin, the wall at z = 0.
def outline_hook():
    plate = strip([(0, -0.0035, 0.0003), (0, 0.0060, 0.0003), (0, 0.0175, 0.0003)], 0.0060, 0.0005)
    collar = place_on(cyl(0.0012, h=0.0050, seg=8, bevel_r=0.0002), Vector((0, 0.0190, 0.0018)), Vector((0, -0.7071, -0.7071)))
    # the foot: bent out from the wall and up into the lip
    lip = strip([(0, -0.0035, 0.0003), (0, -0.0055, 0.0010), (0, -0.0060, 0.0028), (0, -0.0040, 0.0045),
                 (0, -0.0008, 0.0055), (0, 0.0010, 0.0057)], 0.0060, 0.0005)
    return plate + collar + lip


def outline_pin():
    d = Vector((0, -0.7071, -0.7071))
    top = Vector((0, 0.0215, 0.0040))
    shaft = tube([top, top + d * 0.0065], 0.00075, seg=6)
    head = cyl(0.0015, h=0.0012, seg=10, bevel_r=0.0003)
    head = place_on(head, top - d * 0.0004, -d)
    return shaft + head


def outline_wire():
    """Seven-strand picture wire: a loop still over the lip, the rest hanging
    down the wall, kinked, its cut end splayed."""
    pts = [(0.0012, 0.0010, 0.0030), (0.0004, 0.0018, 0.0058), (-0.0006, 0.0006, 0.0072), (-0.0010, -0.0030, 0.0060),
           (-0.0008, -0.0080, 0.0042), (0.0010, -0.0200, 0.0030), (-0.0020, -0.0350, 0.0026), (0.0015, -0.0480, 0.0022),
           (0.0040, -0.0600, 0.0020)]
    s = tube(pts, 0.00065, seg=6, smooth=2)
    end = Vector(pts[-1])
    for k, dv in enumerate([(0.0020, -0.0060, 0.0004), (0.0045, -0.0040, 0.0002), (0.0005, -0.0065, 0.0008)]):
        s = s + tube([end, end + Vector(dv)], 0.00028, seg=4)
    return s


auto('Real__outlineHook', outline_hook, 300, smooth=40.0)
auto('Real__outlinePin', outline_pin, 160, smooth=40.0)
auto('Real__outlineWire', outline_wire, 600, smooth=50.0)


# ========================================================= the wheel ===
# A small wooden wheel of the wheelwright's kind (a 12 in wheel for a hand
# cart, a goat cart or a barrow; Hansen Wheel's 12 in wheel: ten spokes, a
# 3/4 in steel tyre): an elm hub with an iron band at each end and an iron
# axle box, ten oak spokes, five ash felloes of two spokes each, an iron
# tyre shrunk on and bolted through each felloe; 30 cm over the tyre (the
# old wheel's). Hung on a wrought nail under its top felloe. Frame: the
# wheel's axis along z, its centre at the origin; the hub's back on the
# wall at z = -WH_HUB.
WH_R = 0.150
WH_TYRE = 0.0050
WH_FEL = (0.117, 0.145)
WH_W = 0.019
WH_DISH = 0.003
WH_HUB = 0.045
WH_SPOKES = 10
WH_SPOKE0 = math.pi / 2 + math.pi / WH_SPOKES    # the first spoke: the top between two spokes
WH_JOINT0 = WH_SPOKE0 + math.pi / WH_SPOKES      # the first felloe joint, half way to the next spoke


def wheel_tyre():
    z0, z1 = WH_DISH - WH_W / 2, WH_DISH + WH_W / 2
    s = ring_sweep(rrect_section(WH_FEL[1], WH_R, z0, z1, 0.0008, 1), n=60)
    return s


def wheel_iron():
    """The iron that is not the tyre: the hub's bands, the axle box, the
    tyre bolts' heads and the nail it hangs from."""
    s = Shape()
    for zc in (-WH_HUB + 0.0045, WH_HUB - 0.0045):
        s = s + ring_sweep(rrect_section(0.0268, 0.0300, zc - 0.0040, zc + 0.0040, 0.0008, 1), n=28)
    s = s + ring_sweep(rrect_section(0.0100, 0.0150, WH_HUB - 0.006, WH_HUB + 0.0035, 0.0010, 1), n=28)
    for k in range(5):
        a = WH_JOINT0 + TAU * k / 5 + TAU / 10
        p = Vector((math.cos(a), math.sin(a), 0))
        head = ellipsoid(0.0042, 0.0016, 0.0042, seg=8)
        s = s + place_on(head, p * WH_R + Vector((0, 0, WH_DISH)), p)
    # the nail: a wrought square shank out of the wall, its head
    y = WH_FEL[0] - 0.0032
    s = s + rbox(0.0050, 0.0050, WH_HUB + 0.019, r=0.0007, seg=1).move(0, y, -WH_HUB + (WH_HUB + 0.019) / 2)
    s = s + rbox(0.0110, 0.0100, 0.0030, r=0.0012, seg=1).move(0, y, 0.0195)
    return noisy(s, 0.00015, scale=300.0, seed=13)


def wheel_hub():
    prof = [(0, -WH_HUB), (0.0262, -WH_HUB), (0.0270, -WH_HUB + 0.010), (0.0330, -0.018), (0.0355, -0.006),
            (0.0360, 0.004), (0.0345, 0.016), (0.0285, WH_HUB - 0.010), (0.0262, WH_HUB), (0, WH_HUB)]
    s = lathe([(r, z) for r, z in prof], seg=28).rot(math.pi / 2, 0, 0)
    s = cut(s, cyl(0.0100, h=0.12, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0))
    return noisy(s, 0.00025, scale=150.0, seed=17)


def wheel_spokes():
    s = Shape()
    rs = [0.026, 0.040, 0.062, 0.085, 0.104, WH_FEL[0] + 0.004]
    for k in range(WH_SPOKES):
        a = WH_SPOKE0 + TAU * k / WH_SPOKES
        d = Vector((math.cos(a), math.sin(a), 0))
        t = Vector((-d.y, d.x, 0))
        rings = []
        for r in rs:
            u = (r - rs[0]) / (rs[-1] - rs[0])
            wa, wb = lerp(0.0075, 0.0062, u), lerp(0.0058, 0.0050, u)
            if r > 0.103:
                wa, wb = wa * 0.9, wb * 0.9
            c = d * r + Vector((0, 0, WH_DISH * u))
            rings.append([c + t * (wa * math.cos(TAU * j / 10)) + Vector((0, 0, wb * math.sin(TAU * j / 10))) for j in range(10)])
        sp = loft_rings(rings)
        s = s + noisy(sp, 0.00020, scale=180.0, seed=19 + k)
    return s


def wheel_felloes():
    s = Shape()
    z0, z1 = WH_DISH - WH_W / 2 + 0.0004, WH_DISH + WH_W / 2 - 0.0004
    sec = rrect_section(WH_FEL[0], WH_FEL[1], z0, z1, 0.0020, 2)
    gap = 0.0007 / WH_FEL[1]
    for k in range(5):
        a0 = WH_JOINT0 + TAU * k / 5 + gap / 2
        a1 = a0 + TAU / 5 - gap
        s = s + noisy(ring_sweep(sec, a0, a1, n=14), 0.00022, scale=160.0, seed=23 + k)
    return s


auto('Real__wagonWheelTyre', wheel_tyre, 1000, smooth=40.0)
auto('Real__wagonWheelIron', wheel_iron, 2000, smooth=40.0)
auto('Real__wagonWheelHub', wheel_hub, 1200, smooth=45.0)
auto('Real__wagonWheelSpokes', wheel_spokes, 1600, smooth=50.0)
auto('Real__wagonWheelFelloes', wheel_felloes, 1800, smooth=40.0)
