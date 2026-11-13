"""t08: the props remodelled at their real sizes (docs/design/PROPS.md, P3, the
rare): the Halloween decorations of a party room or a shrine, as cheap shop
decorations are, years after the party: the plastic crow, the hanging bat,
the resin cat, the besom, the plastic cauldron, the cheesecloth ghost, the
carved pumpkin gone soft and the plastic skeleton hand. Sources and sizes:
t08.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame: x across its front, y up from the floor, z out of
its front (a creature's face), unless its section says otherwise.

The crow (`place_small_crows`): a moulded plastic crow of the 6 in (16 cm)
kind, standing on its feet; the body moulded with its wings and tail in
relief, the long wing and tail feathers stand free.

crowBody       the body, head, wings and tail (black polypropylene)
crowBeak       the beak (moulded with it, its black rubbed grey)
crowLegs       the legs and toes (black plastic over wire)
crowEyes       the two glass beads pressed into the head
"""
from propkit import *
from propkit import _mesh_obj, _eval_to_shape, _drop
import propkit
import random
import zlib
from mathutils.bvhtree import BVHTree


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


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


# ------------------------------------------------------------- helpers ---
_meta_n = [0]
MB_K = 1.0 / 0.564


def meta(elems, res=0.002, threshold=0.6):
    """A blobby surface from metaball elements (r03's): ('ball', centre,
    radius[, stiffness]), ('ell', centre, (rx, ry, rz)[, stiffness[, euler]]),
    ('cap', a, b, radius[, stiffness]); '-ball' / '-ell' subtract. Sizes are
    the element's own surface."""
    _meta_n[0] += 1
    name = f'_t08mb{_meta_n[0]}x'
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
    ob = _mesh_obj('_t08dec', s)
    m = ob.modifiers.new('d', 'DECIMATE')
    m.decimate_type = 'COLLAPSE'
    m.ratio = tris / n
    m.use_collapse_triangulate = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def bvh_of(s):
    return BVHTree.FromPolygons([p.copy() for p in s.v], [list(f) for f in s.f])


def project_patch(bvh, centre, normal, up, w, h, off, seg=12, rings=3, shape=None):
    """A thin painted patch (an oval w x h in the tangent plane at `centre`,
    `up` its long axis) laid onto a surface (r03's): every vertex is cast back
    onto the surface along -normal and lifted `off` off it."""
    n = Vector(normal).normalized()
    u = Vector(up)
    u = (u - n * u.dot(n)).normalized()
    r = n.cross(u).normalized()
    c = Vector(centre)
    pts = [(0.0, 0.0)]
    for k in range(1, rings + 1):
        t = k / rings
        for i in range(seg):
            a = TAU * i / seg
            x, y = math.cos(a) * w / 2 * t, math.sin(a) * h / 2 * t
            if shape:
                x, y = shape(x, y, a, t)
            pts.append((x, y))
    v = []
    for x, y in pts:
        q = c + r * x + u * y
        hit = bvh.ray_cast(q + n * 0.02, -n, 0.05)
        if hit[0] is None:
            v.append(q + n * off)
        else:
            v.append(hit[0] + hit[1].normalized() * off)
    f = []
    for i in range(seg):
        f.append((0, 1 + i, 1 + (i + 1) % seg))
    for k in range(1, rings):
        a0, b0 = 1 + (k - 1) * seg, 1 + k * seg
        for i in range(seg):
            j = (i + 1) % seg
            f.append((a0 + i, b0 + i, b0 + j, a0 + j))
    s = Shape(v, f)
    s0 = s.v[0]
    e1, e2 = s.v[1] - s0, s.v[2] - s0
    if e1.cross(e2).dot(n) < 0:
        s.f = [tuple(reversed(ff)) for ff in s.f]
    return s


def flatten_below(s, y0, k=0.12):
    """Squash everything under y0 toward it (a moulded flat base)."""
    def fn(p):
        if p.y < y0:
            p.y = y0 - (y0 - p.y) * k
        return p
    return s.warp(fn)


def to_floor(s):
    lo, _ = s.bounds()
    return s.move(0, -lo.y, 0)


def fix_out(s, c):
    """Turn every face away from the point c."""
    out = []
    for f in s.f:
        p = [s.v[i] for i in f]
        n = (p[1] - p[0]).cross(p[2] - p[0])
        m = sum(p, Vector((0, 0, 0))) / len(p)
        out.append(tuple(reversed(f)) if n.dot(m - c) < 0 else f)
    s.f = out
    return s


def loft(rings, cap0=True, cap1=True):
    """Join closed rings of points (each the same count) into a surface,
    its ends closed with a fan to their centres."""
    m = len(rings[0])
    v = [Vector(p) for ring in rings for p in ring]
    f = []
    for i in range(len(rings) - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, (i + 1) * m + j2, (i + 1) * m + j))
    if cap0:
        v.append(sum((Vector(p) for p in rings[0]), Vector()) / m)
        c = len(v) - 1
        for j in range(m):
            f.append((c, (j + 1) % m, j))
    if cap1:
        v.append(sum((Vector(p) for p in rings[-1]), Vector()) / m)
        c = len(v) - 1
        n = len(rings) - 1
        for j in range(m):
            f.append((c, n * m + j, n * m + (j + 1) % m))
    return propkit._fix_winding(Shape(v, f))


def frame_along(d):
    """Two unit vectors square to the direction d and to each other."""
    d = Vector(d).normalized()
    up = Vector((0, 1, 0)) if abs(d.y) < 0.9 else Vector((1, 0, 0))
    a = d.cross(up).normalized()
    b = a.cross(d).normalized()
    return a, b


def lathe_along(prof, p0, p1, seg=8, squash=1.0, up=None):
    """A lathe of [(r, t)] (t from 0 at p0 to 1 at p1) turned about the line
    p0 -> p1, its section squashed by `squash` along `up` (a bone flatter
    from back to front)."""
    p0, p1 = Vector(p0), Vector(p1)
    d = p1 - p0
    L = d.length
    dn = d.normalized()
    if up is not None:
        b = (Vector(up) - dn * Vector(up).dot(dn)).normalized()
        a = dn.cross(b).normalized()
    else:
        a, b = frame_along(dn)
    rings = []
    for r, t in prof:
        c = p0 + dn * (L * t)
        rings.append([c + a * (r * math.cos(TAU * i / seg)) + b * (r * squash * math.sin(TAU * i / seg)) for i in range(seg)])
    return loft(rings)


def feather(p0, p1, w0, w1, t, side, camber=0.0, n=7, seg=6, bend=0.0):
    """A long feather (a vane), flattened: from its root p0 to its tip p1,
    `w0` wide at the root narrowing to `w1` and a rounded tip, `t` thick;
    `side` the vane's broad direction (square to its length), cambered
    `camber` (its middle bowed out along the normal) and its length bowed
    `bend` toward `side`'s normal."""
    p0, p1 = Vector(p0), Vector(p1)
    d = (p1 - p0)
    L = d.length
    dn = d.normalized()
    s = Vector(side)
    s = (s - dn * s.dot(dn)).normalized()
    nrm = dn.cross(s).normalized()
    rings = []
    for k in range(n + 1):
        u = k / n
        tip = max(0.0, (u - 0.7) / 0.3)
        w = (w0 + (w1 - w0) * u) * math.sqrt(max(0.0, 1 - tip * tip)) * 0.5 + 0.0004
        c = p0 + dn * (L * u) + nrm * (bend * L * u * u)
        ring = []
        for i in range(seg):
            a = TAU * i / seg
            x = math.cos(a) * w
            y = math.sin(a) * t * 0.5 + camber * (1 - (math.cos(a)) ** 2) * w
            ring.append(c + s * x + nrm * y)
        rings.append(ring)
    return loft(rings)


# ================================================================= crow ===
# A moulded plastic crow, the 6 in (16 cm) shop kind: a hollow black
# polypropylene body with the wings and tail moulded on it, the primaries
# and the tail's feathers free of it, a stout beak, two glass bead eyes, and
# legs of plastic over wire, its feet spread on the floor. Standing: its
# feet at y = 0, the beak toward +z.
CROW_HEAD = Vector((0.0, 0.082, 0.047))


def crow_body_raw():
    e = [
        ('ell', (0, 0.050, 0.002), (0.0205, 0.0195, 0.040), 2.0, (-0.30, 0, 0)),  # the body, its breast up
        ('ell', (0, 0.054, 0.024), (0.0185, 0.0195, 0.017), 1.8),                  # the breast
        ('ell', (0, 0.038, 0.004), (0.0150, 0.0110, 0.022), 1.6),                  # the belly
        ('ell', (0, 0.069, 0.035), (0.0130, 0.0150, 0.0135), 1.8, (-0.5, 0, 0)),   # the neck
        ('ell', tuple(CROW_HEAD), (0.0140, 0.0135, 0.0180), 2.0, (-0.10, 0, 0)),   # the head
        ('ell', (0, 0.0895, 0.042), (0.0085, 0.0050, 0.0130), 1.5),                # the crown, flat
        ('ell', (0, 0.043, -0.032), (0.0125, 0.0085, 0.020), 1.6, (0.25, 0, 0)),   # the rump, down to the tail
        ('ell', (0, 0.040, -0.050), (0.0100, 0.0030, 0.016), 2.0, (0.22, 0, 0)),   # the tail's root
    ]
    for sx in (-1, 1):
        # the folded wing in relief on each flank: its shoulder high, swept back
        e.append(('ell', (sx * 0.0165, 0.057, 0.004), (0.0070, 0.0160, 0.030), 1.6, (-0.20, sx * 0.08, 0)))
        e.append(('ell', (sx * 0.0140, 0.054, -0.026), (0.0060, 0.0110, 0.018), 1.6, (0.05, sx * 0.05, 0)))
        # the thighs, feathered down to the legs
        e.append(('ell', (sx * 0.0080, 0.031, 0.006), (0.0065, 0.0095, 0.0075), 1.6))
    s = meta(e, res=0.0015)
    # the moulding: feather edges pressed in low relief across the back and
    # flanks (rows of shallow scallops)
    out = s.copy()
    for i, p in enumerate(out.v):
        if p.y < 0.034:
            continue
        ang = math.atan2(p.x, p.y - 0.048)
        row = (p.z * 1000.0 / 6.5) + 0.8 * math.sin(ang * 6.0)
        wob = 0.5 - 0.5 * math.cos(TAU * row)
        depth = 0.00045 * smoothstep(0.036, 0.052, p.y) * wob
        n = (p - Vector((0, 0.048, p.z))).normalized()
        out.v[i] = p - n * depth
    return out


_crow = {}


def crow_frame():
    if 'body' not in _crow:
        b = decimate(crow_body_raw(), 950)
        _crow['body'] = b
        _crow['bvh'] = bvh_of(b)
    return _crow


def crow_feathers():
    """The primaries, folded, crossed over the rump to near the tail's end,
    and the tail's feathers, fanned a little, each its own vane."""
    out = Shape()
    for sx in (-1, 1):
        for k in range(4):
            r0 = Vector((sx * (0.0150 - 0.0010 * k), 0.0585 - 0.0016 * k, -0.012 - 0.005 * k))
            r1 = Vector((sx * (0.0045 - 0.0006 * k), 0.0475 - 0.0018 * k, -0.080 + 0.006 * k))
            out = out + feather(r0, r1, 0.0100, 0.0055, 0.0011, (sx * 0.35, 1.0, 0), camber=0.12, n=5, seg=4)
    for k in range(7):
        a = (k - 3) / 3.0
        r0 = Vector((a * 0.0040, 0.0405, -0.040))
        r1 = Vector((a * 0.0105, 0.0335 + 0.0010 * abs(a), -0.093 + 0.004 * abs(a)))
        out = out + feather(r0, r1, 0.0080, 0.0092, 0.0010, (1, 0, 0), camber=0.06, n=5, seg=4)
    return out


def crow_body():
    return crow_frame()['body'].copy() + crow_feathers()


def crow_beak():
    """The beak: deep at its root and arched, its culmen curving down to a
    point, its root sunk into the face."""
    bvh = crow_frame()['bvh']
    d = Vector((0, -0.20, 1)).normalized()
    hit = bvh.ray_cast(CROW_HEAD, d, 0.2)
    root = hit[0] - d * 0.0035
    rings = []
    L = 0.0265
    n = 10
    seg = 12
    for k in range(n + 1):
        u = k / n
        z = L * u
        h = 0.0118 * (1 - u) ** 0.70 + 0.0006           # its depth
        w = 0.0088 * (1 - u) ** 0.95 + 0.0005           # its width
        top = 0.5 * h - 0.0030 * u * u                  # the culmen, arched down
        bot = -0.5 * h - 0.0014 * u * u
        cy = (top + bot) / 2
        hh = (top - bot) / 2
        ring = []
        for i in range(seg):
            a = TAU * i / seg
            ca, sa = math.cos(a), math.sin(a)
            x = w * 0.5 * math.copysign(abs(ca) ** 0.8, ca)
            y = cy + hh * (math.copysign(abs(sa) ** 0.9, sa) if sa < 0 else sa ** 0.7)
            ring.append(Vector((x, root.y + y - 0.0012 * u, root.z + z)))
        rings.append(ring)
    return loft(rings)


def crow_eyes():
    bvh = crow_frame()['bvh']
    out = Shape()
    for sx in (-1, 1):
        d = Vector((sx * math.sin(math.radians(64)), 0.20, math.cos(math.radians(64)))).normalized()
        hit = bvh.ray_cast(CROW_HEAD, d, 0.2)
        p, n = hit[0], hit[1].normalized()
        out = out + sphere(0.0026, seg=10, rings=6).move(*(p - n * 0.0008))
    return out


def crow_legs():
    """The tarsi from the thighs down to the feet; three toes forward and
    one back, spread on the floor, their claws turned down into it."""
    out = Shape()
    for sx in (-1, 1):
        hip = Vector((sx * 0.0080, 0.030, 0.007))
        ankle = Vector((sx * 0.0095, 0.0045, 0.008))
        out = out + tube([hip, ankle], 0.0016, seg=6)
        for a, L in ((-0.42, 0.016), (0.0, 0.020), (0.42, 0.015), (math.pi + sx * 0.15, 0.012)):
            a = a + sx * 0.10
            dirn = Vector((math.sin(a), 0, math.cos(a)))
            p1 = ankle + dirn * (L * 0.5) + Vector((0, -0.0024, 0))
            p2 = ankle + dirn * L + Vector((0, -0.0032, 0))
            p2.y = 0.0010
            p1.y = max(p1.y, 0.0012)
            claw = p2 + dirn * 0.004 + Vector((0, -0.0008, 0))
            claw.y = max(claw.y, 0.0003)
            out = out + tube([ankle, p1, p2, claw], 0.0010, seg=5, r_end=0.0004)
    return out


auto('Real__crowBody', crow_body, 1800, smooth=60)
auto('Real__crowBeak', crow_beak, 280, smooth=50)
auto('Real__crowLegs', crow_legs, 560, smooth=50)
auto('Real__crowEyes', crow_eyes, 280, smooth=80)


# ================================================================== bat ===
# A hanging Halloween bat of the Fun World kind (FUN4421: 19 in, 48 cm,
# across its wings): a body of black foam, flocked, on a plastic skeleton,
# the wings plastic over mesh stretched between the arm's and fingers'
# moulded ribs, spread as in flight; hung level by a cord from its back, the
# cord taped to the ceiling. Its frame: the cord's top (the ceiling) at the
# origin, everything below it; x across the wings, z toward its head.
BAT_Y = -0.150                      # the back, under the ceiling
BAT_SCALE = 1.0


def _bat_plan():
    """The wing's joints in plan (x out along the right wing, z forward)."""
    P = dict(S=(0.017, 0.020), E=(0.078, 0.040), W=(0.140, 0.024), d2=(0.222, 0.012), d3=(0.238, -0.022),
             d4=(0.190, -0.082), d5=(0.112, -0.106), A=(0.024, -0.066))
    return {k: Vector((x, 0.0, z)) for k, (x, z) in P.items()}


def _bat_y(p):
    """The wing's height at plan point p: its root at the body's middle,
    drooping out to the tips, the leading edge a little higher."""
    ax = abs(p.x) / 0.24
    return BAT_Y - 0.010 - 0.030 * ax ** 1.7 + 0.004 * max(0.0, p.z) / 0.04


def _lift(p, sag=0.0):
    q = p.copy()
    q.y = _bat_y(p) - sag
    return q


def bat_wings():
    P = _bat_plan()
    out = Shape()
    for sx in (-1, 1):
        def X(v):
            return Vector((v.x * sx, 0.0, v.z))
        W = X(P['W'])
        tips = [X(P[k]) for k in ('d2', 'd3', 'd4', 'd5')]
        verts, faces = [], []

        def grid(fn, nu, nv):
            base = len(verts)
            for j in range(nv + 1):
                for i in range(nu + 1):
                    verts.append(fn(i / nu, j / nv))
            for j in range(nv):
                for i in range(nu):
                    a = base + j * (nu + 1) + i
                    faces.append((a, a + 1, a + nu + 2, a + nu + 1))
        # the membranes between the fingers, from the wrist out to the
        # trailing edge, scalloped (bowed in) between the tips, sagging
        for a, b, bow in ((tips[0], tips[1], 0.004), (tips[1], tips[2], 0.020), (tips[2], tips[3], 0.024)):
            def fn(s, t, a=a, b=b, bow=bow):
                e = a.lerp(b, t)
                e = e - (e - W).normalized() * bow * math.sin(math.pi * t)
                p = W.lerp(e, s)
                return _lift(p, 0.006 * math.sin(math.pi * t) * s)
            grid(fn, 6, 6)
        # the membrane between the arm, the fifth finger, the leg and the
        # flank (a Coons patch)
        S, E, A, d5 = X(P['S']), X(P['E']), X(P['A']), tips[3]

        def arm(u):
            return S.lerp(E, u / 0.45) if u < 0.45 else E.lerp(W, (u - 0.45) / 0.55)

        def trail(u):
            e = A.lerp(d5, u)
            return e + Vector((0, 0, 1)) * 0.026 * math.sin(math.pi * u)

        def coons(u, v):
            B, T = arm(u), trail(u)
            L, R = S.lerp(A, v), W.lerp(d5, v)
            p = (B * (1 - v) + T * v + L * (1 - u) + R * u
                 - (S * (1 - u) * (1 - v) + W * u * (1 - v) + A * (1 - u) * v + d5 * u * v))
            return _lift(p, 0.010 * math.sin(math.pi * u) * math.sin(math.pi * v))
        grid(coons, 8, 7)
        mem = Shape(verts, faces)
        out = out + mem
        # the moulded ribs: the arm, the four fingers, the thumb's hook
        out = out + tube([_lift(S) + Vector((0, 0.002, 0)), _lift(E) + Vector((0, 0.002, 0)), _lift(W) + Vector((0, 0.002, 0))], 0.0030, seg=6, r_end=0.0022)
        for t_ in tips:
            mid = W.lerp(t_, 0.5)
            out = out + tube([_lift(W) + Vector((0, 0.0015, 0)), _lift(mid) + Vector((0, 0.0012, 0)), _lift(t_) + Vector((0, 0.0006, 0))], 0.0019, seg=5, r_end=0.0008)
        thumb = W + Vector((sx * 0.004, 0, 0.016))
        out = out + tube([_lift(W) + Vector((0, 0.002, 0)), _lift(thumb) + Vector((0, 0.004, 0))], 0.0018, seg=5, r_end=0.0007)
    return out


def bat_body_raw():
    y = BAT_Y - 0.012
    e = [
        ('ell', (0, y, -0.012), (0.021, 0.016, 0.042), 2.0),                 # the torso
        ('ell', (0, y + 0.001, 0.020), (0.019, 0.015, 0.020), 1.8),          # the chest
        ('ell', (0, y + 0.004, 0.044), (0.014, 0.013, 0.012), 1.8),          # the neck
        ('ell', (0, y + 0.006, 0.060), (0.016, 0.014, 0.016), 2.0),          # the head
        ('ell', (0, y + 0.002, 0.077), (0.0085, 0.0075, 0.0095), 1.8),       # the snout
        ('ell', (0, y - 0.003, 0.083), (0.0055, 0.0045, 0.0045), 1.6),       # the nose leaf
    ]
    for sx in (-1, 1):
        e.append(('cap', (sx * 0.011, y - 0.002, -0.044), (sx * 0.020, y - 0.004, -0.074), 0.0045, 1.6))  # the legs
        e.append(('ell', (sx * 0.022, y - 0.004, -0.080), (0.0050, 0.0030, 0.0065), 1.6))              # the feet
        e.append(('ell', (sx * 0.014, y + 0.004, 0.012), (0.009, 0.007, 0.012), 1.6))                  # the shoulders
    s = meta(e, res=0.0022)
    return s


def bat_ear(sx):
    """An ear: a tall cupped triangle from the head's crown, open forward."""
    y = BAT_Y - 0.012
    base = Vector((sx * 0.010, y + 0.014, 0.058))
    tip = Vector((sx * 0.019, y + 0.042, 0.050))
    rings = []
    n, seg = 6, 8
    d = (tip - base)
    for k in range(n + 1):
        u = k / n
        c = base + d * u
        w = 0.0105 * (1 - u) ** 0.9 + 0.0006
        t = 0.0030 * (1 - u) + 0.0006
        ring = []
        for i in range(seg):
            a = TAU * i / seg
            x, z = math.cos(a) * w, math.sin(a) * t
            # the cup: the front's middle pressed back
            if z > 0:
                z -= 0.6 * w * (1 - (x / w) ** 2) * 0.35
            ring.append(c + Vector((x, 0, z)))
        rings.append(ring)
    return loft(rings)


def bat_body():
    b = decimate(bat_body_raw(), 620)
    return b + bat_ear(-1) + bat_ear(1)


def bat_eyes():
    y = BAT_Y - 0.012 + 0.008
    out = Shape()
    for sx in (-1, 1):
        out = out + sphere(0.0022, seg=10, rings=6).move(sx * 0.0085, y, 0.071)
    return out


def bat_cord():
    """The cord: a loop knotted through the back, up to the tape."""
    top = Vector((0, -0.0012, 0.004))
    knot = Vector((0, BAT_Y + 0.004, 0.004))
    s = tube([knot, knot.lerp(top, 0.5) + Vector((0.0008, 0, 0)), top], 0.0009, seg=5)
    s = s + sphere(0.0022, seg=8, rings=5).move(*knot)
    return s


def bat_tape():
    """A strip of masking tape over the cord's end on the ceiling, a corner
    come away."""
    s = slab(0.050, 0.024, 0.0002, nx=8, ny=4).rot(math.pi / 2, 0, 0)

    def lift(p):
        if p.x > 0.012 and p.z > 0.0:
            k = (p.x - 0.012) / 0.013 * (p.z / 0.012)
            p.y -= 0.004 * max(0.0, k) ** 1.5
        return p
    s = s.warp(lift)
    return s.move(0, -0.0002, 0.004)


auto('Real__batWings', bat_wings, 1300, smooth=50)
auto('Real__batBody', bat_body, 900, smooth=60)
auto('Real__batEyes', bat_eyes, 280, smooth=80)
auto('Real__batCord', bat_cord, 280, smooth=60)
auto('Real__batTape', bat_tape, 200, smooth=30)


# ================================================================== cat ===
# A cast resin black cat figurine, the 6 in (15 cm) shop kind: sitting up,
# its tail round its right side to its forepaws, glazed black, its eyes
# painted yellow-green with a black slit. Its frame: its base at y = 0, the
# face toward +z.
CAT_HEAD = Vector((0.0, 0.114, 0.018))


def cat_body_raw():
    e = [
        ('ell', (0, 0.034, -0.010), (0.026, 0.030, 0.030), 2.0),                  # the pelvis, low and wide
        ('ell', (0, 0.058, 0.001), (0.022, 0.034, 0.021), 2.0, (0.30, 0, 0)),     # the back, rising forward
        ('ell', (0, 0.070, 0.016), (0.0185, 0.022, 0.016), 1.8),                  # the chest
        ('ell', (0, 0.086, 0.005), (0.0195, 0.016, 0.016), 1.8),                  # the shoulders
        ('ell', (0, 0.097, 0.011), (0.0145, 0.015, 0.014), 1.8),                  # the neck
        ('ell', tuple(CAT_HEAD), (0.0210, 0.0180, 0.0180), 2.0),                  # the head
        ('ell', (0, 0.1045, 0.034), (0.0085, 0.0062, 0.0065), 1.6),               # the muzzle
        ('ell', (0, 0.0985, 0.030), (0.0060, 0.0040, 0.0050), 1.5),               # the chin
        ('ell', (0, 0.1225, 0.026), (0.0120, 0.0060, 0.0080), 1.4),               # the brow
    ]
    for sx in (-1, 1):
        e += [
            ('ell', (sx * 0.0185, 0.029, -0.013), (0.0185, 0.025, 0.027), 2.0),                     # the haunches
            ('cap', (sx * 0.0100, 0.064, 0.019), (sx * 0.0100, 0.009, 0.025), 0.0062, 1.8),         # the forelegs
            ('ell', (sx * 0.0100, 0.0058, 0.031), (0.0072, 0.0052, 0.0100), 1.8),                    # the forepaws
            ('ell', (sx * 0.0245, 0.0052, 0.007), (0.0070, 0.0048, 0.0150), 1.8),                    # the hind feet
            ('ell', (sx * 0.0095, 0.1065, 0.0265), (0.0110, 0.0090, 0.0090), 1.6),                   # the cheeks
        ]
    # the tail, from under the rump round its right side to its forepaws
    pts = [(0.006, 0.0075, -0.040), (0.026, 0.0070, -0.036), (0.0375, 0.0065, -0.016), (0.0380, 0.0060, 0.008),
           (0.030, 0.0056, 0.028), (0.019, 0.0052, 0.040)]
    for k, (a, b) in enumerate(zip(pts, pts[1:])):
        e.append(('cap', a, b, 0.0062 - 0.00025 * k, 1.8))
    s = meta(e, res=0.0016)
    s = flatten_below(s, 0.0035, 0.08)
    return to_floor(s)


_cat = {}


def cat_frame():
    if 'body' not in _cat:
        b = decimate(cat_body_raw(), 1150)
        _cat['body'] = b
        _cat['bvh'] = bvh_of(b)
    return _cat


def cat_ear(sx):
    """An ear: a cupped triangle from the crown, open forward, a little
    out to its side."""
    base = Vector((sx * 0.0112, 0.1240, 0.0135))
    tip = Vector((sx * 0.0170, 0.1425, 0.0115))
    rings = []
    n, seg = 6, 8
    d = tip - base
    for k in range(n + 1):
        u = k / n
        c = base + d * u
        w = 0.0088 * (1 - u) ** 0.80 + 0.0009
        t = 0.0030 * (1 - u) + 0.0008
        ring = []
        for i in range(seg):
            a = TAU * i / seg
            x, z = math.cos(a) * w, math.sin(a) * t
            if z > 0:
                z -= 0.45 * w * (1 - (x / w) ** 2) * 0.4
            # turned a little out to its side
            q = Vector((x, 0, z))
            q.rotate(Euler((0, -sx * 0.35, 0)))
            ring.append(c + q)
        rings.append(ring)
    return loft(rings)


def cat_body():
    return cat_frame()['body'].copy() + cat_ear(-1) + cat_ear(1)


def _cat_eye_at(sx):
    bvh = cat_frame()['bvh']
    d = Vector((sx * math.sin(math.radians(27)), 0.10, math.cos(math.radians(27)))).normalized()
    hit = bvh.ray_cast(CAT_HEAD + Vector((0, 0.002, 0)), d, 0.2)
    return hit[0], hit[1].normalized()


def cat_eyes():
    """The painted irises: almonds, their outer corners up."""
    bvh = cat_frame()['bvh']
    out = Shape()
    for sx in (-1, 1):
        p, n = _cat_eye_at(sx)

        def almond(x, y, a, t):
            k = abs(x) / 0.00475
            return x, y * (1 - 0.45 * k * k)
        out = out + project_patch(bvh, p, n, (sx * 0.30, 1, 0), 0.0095, 0.0066, 0.00025, seg=14, rings=2, shape=almond)
    return out


def cat_pupils():
    bvh = cat_frame()['bvh']
    out = Shape()
    for sx in (-1, 1):
        p, n = _cat_eye_at(sx)
        out = out + project_patch(bvh, p, n, (sx * 0.05, 1, 0), 0.0017, 0.0060, 0.0005, seg=10, rings=1)
    return out


auto('Real__catBody', cat_body, 1500, smooth=70)
auto('Real__catEyes', cat_eyes, 200, smooth=80)
auto('Real__catPupils', cat_pupils, 100, smooth=80)


# ======================================================== skeleton hand ===
# A life-size moulded plastic skeleton hand, the shop kind (7 in, 18 cm from
# the wrist to the longest fingertip, Fun World FUN5608): a right hand in one
# moulding, the carpals' knot, the five metacarpals fanning from it, the
# phalanges with their knuckles, a stub of the radius and ulna cut square.
# Lying palm down, the fingers a little curled, their tips on the floor:
# x across (the thumb toward -x), y up from the floor, z toward the fingers.
BONE = {
    # (r, t): its base, the shaft, the head; a bone's section a little flat
    'meta': [(0.0072, 0.0), (0.0064, 0.08), (0.0044, 0.30), (0.0042, 0.62), (0.0056, 0.82), (0.0064, 0.92), (0.0050, 0.98), (0.0016, 1.0)],
    'prox': [(0.0068, 0.0), (0.0058, 0.10), (0.0040, 0.35), (0.0039, 0.70), (0.0050, 0.90), (0.0042, 0.98), (0.0014, 1.0)],
    'mid': [(0.0058, 0.0), (0.0049, 0.12), (0.0034, 0.40), (0.0034, 0.70), (0.0043, 0.90), (0.0035, 0.98), (0.0013, 1.0)],
    'dist': [(0.0048, 0.0), (0.0040, 0.15), (0.0026, 0.50), (0.0027, 0.75), (0.0034, 0.90), (0.0016, 1.0)],
}
HAND_FINGERS = [
    # metacarpal base, head; phalanx lengths; yaw (rad, + toward +x)
    ((-0.0140, 0.0110, 0.026), (-0.0215, 0.0150, 0.089), (0.040, 0.023, 0.018), -0.10),
    ((-0.0035, 0.0116, 0.027), (-0.0040, 0.0155, 0.091), (0.044, 0.027, 0.019), -0.01),
    ((0.0075, 0.0112, 0.026), (0.0140, 0.0146, 0.084), (0.041, 0.026, 0.019), 0.10),
    ((0.0175, 0.0104, 0.024), (0.0305, 0.0130, 0.074), (0.032, 0.018, 0.017), 0.22),
]


def _bone(kind, a, b, k=1.0, seg=6):
    prof = [(r * k, t) for r, t in BONE[kind]]
    # each runs a little into the next, as the moulding joins them
    d = (Vector(b) - Vector(a)).normalized()
    return lathe_along(prof, Vector(a) - d * 0.0012, Vector(b) + d * 0.0012, seg=seg, squash=0.74, up=(0, 1, 0))


def _chain(head, d, lens, pitches, kinds, k=1.0):
    """Phalanges from a knuckle along d (turned down by each pitch)."""
    out = Shape()
    p = Vector(head)
    yaw = math.atan2(d.x, d.z)
    for L, pitch, kind in zip(lens, pitches, kinds):
        q = p + Vector((math.sin(yaw) * math.cos(pitch), -math.sin(pitch), math.cos(yaw) * math.cos(pitch))) * L
        out = out + _bone(kind, p, q, k)
        p = q
    return out, p


def hand_bones():
    out = Shape()
    for base, head, lens, yaw in HAND_FINGERS:
        out = out + _bone('meta', base, head)
        d = Vector(head) - Vector(base)
        d.y = 0
        d = d.normalized()
        d.rotate(Euler((0, yaw * 0.8, 0)))
        ph, _ = _chain(head, d, lens, (math.radians(7), math.radians(18), math.radians(28)), ('prox', 'mid', 'dist'))
        out = out + ph
    # the thumb: its metacarpal out from the wrist, two phalanges
    tb, th = Vector((-0.0215, 0.0102, 0.020)), Vector((-0.0440, 0.0112, 0.054))
    out = out + _bone('meta', tb, th, 1.08)
    d = (th - tb)
    d.y = 0
    d = d.normalized()
    d.rotate(Euler((0, 0.25, 0)))
    ph, _ = _chain(th, d, (0.031, 0.024), (math.radians(6), math.radians(14)), ('prox', 'dist'), 1.08)
    return out + ph


def hand_wrist():
    """The carpals, two rows of knuckled little bones, and the radius and
    ulna cut square behind them."""
    e = []
    row1 = [(-0.0170, 0.0100, 0.0050, 0.0072), (-0.0055, 0.0108, 0.0040, 0.0068), (0.0055, 0.0105, 0.0042, 0.0064), (0.0160, 0.0096, 0.0060, 0.0056)]
    row2 = [(-0.0200, 0.0098, 0.0170, 0.0064), (-0.0110, 0.0104, 0.0180, 0.0058), (-0.0010, 0.0114, 0.0175, 0.0070), (0.0110, 0.0104, 0.0170, 0.0068)]
    for x, y, z, r in row1 + row2:
        e.append(('ell', (x, y, z), (r * 1.1, r * 0.85, r), 1.6))
    e.append(('ell', (0.0200, 0.0105, 0.0030), (0.0036, 0.0036, 0.0036), 1.4))   # the pisiform
    s = meta(e, res=0.0011)
    s = decimate(s, 320)
    rad = lathe_along([(0.0082, 0.0), (0.0086, 0.5), (0.0112, 0.85), (0.0104, 0.97), (0.0046, 1.0)],
                      Vector((-0.0110, 0.0102, -0.026)), Vector((-0.0115, 0.0104, 0.0030)), seg=8, squash=0.75, up=(0, 1, 0))
    uln = lathe_along([(0.0060, 0.0), (0.0063, 0.6), (0.0080, 0.88), (0.0064, 0.97), (0.0030, 1.0)],
                      Vector((0.0130, 0.0094, -0.026)), Vector((0.0145, 0.0098, 0.0)), seg=7, squash=0.85, up=(0, 1, 0))
    return s + rad + uln


def hand_all():
    """The hand laid down: tipped about x until the wrist's stub and the
    fingertips both rest on the floor."""
    s = hand_bones() + hand_wrist()
    back = min((p for p in s.v if p.z < 0.0), key=lambda p: p.y)
    tip = min((p for p in s.v if p.z > 0.12), key=lambda p: p.y)
    a = math.atan2(tip.y - back.y, tip.z - back.z)
    s.rot(a, 0, 0)
    lo, _ = s.bounds()
    return s.move(0, -lo.y, 0)


auto('Real__skelHand', hand_all, 2600, smooth=55)


# ================================================================ besom ===
# A decorative witch's besom, the shop kind (a 24 in twig-look handle and
# 14 in twig bristles): a crooked stick with its bark on, a head of birch
# twigs bound round its end with two wraps of jute twine, lying on the
# floor. Its frame: along +x from the handle's end (x = 0) to the twigs'
# tips, y up from the floor, z across.
BESOM_HANDLE = 0.62                 # the stick, end to end
BESOM_BIND = (0.505, 0.600)         # where the twigs are bound
BESOM_TIPS = 0.900


def _besom_axis(x):
    """The stick's centre at x: resting on the floor at its end, raised by
    the bound head under it, crooked a little."""
    y = 0.0115 + (0.0355 - 0.0115) * smoothstep(0.0, BESOM_BIND[0], x) ** 1.2
    y += 0.0025 * math.sin(x * 9.0 + 0.6) * smoothstep(0.05, 0.25, x) * (1 - smoothstep(0.40, 0.50, x))
    z = 0.006 * math.sin(x * 6.0 + 1.1) + 0.003 * math.sin(x * 17.0)
    return Vector((x, y, z))


def besom_handle():
    """The stick: a hazel rod, 24 mm at its head end tapering to 20 mm,
    knots where twigs were cut off, its end cut square and rubbed round."""
    rings = []
    n, seg = 26, 9
    for k in range(n + 1):
        x = BESOM_HANDLE * k / n
        c = _besom_axis(x)
        r = 0.0100 + 0.0020 * x / BESOM_HANDLE
        if k == 0:
            r *= 0.85
        ring = []
        for i in range(seg):
            a = TAU * i / seg
            rr = r * (1 + 0.05 * noise.noise(Vector((x * 30.0, a * 1.3, 0.4))))
            # the knots
            for kx, ka in ((0.17, 1.2), (0.33, 4.0), (0.44, 2.4)):
                d = math.hypot((x - kx) / 0.010, ((a - ka + math.pi) % TAU - math.pi) / 0.6)
                rr += 0.0028 * max(0.0, 1 - d) ** 1.5
            ring.append(c + Vector((0, math.cos(a) * rr, math.sin(a) * rr)))
        rings.append(ring)
    return loft(rings)


def besom_twigs():
    """The head: sixty-odd birch twigs gathered round the stick's end,
    bound, then splaying out to the tips, those underneath pressed flat on
    the floor, a few forked."""
    out = Shape()
    n = 62
    x0, x1 = BESOM_BIND
    axis_end = _besom_axis(x1)
    for i in range(n):
        a = TAU * (i + rnd(-0.3, 0.3)) / n
        ring = 0.0150 + rnd(0.0, 0.0170)
        root_x = x0 - rnd(0.0, 0.03)
        pts = []
        tip_x = BESOM_TIPS - rnd(0.0, 0.07)
        spread = rnd(0.55, 1.0)
        bow = rnd(-0.01, 0.01)
        for k in range(8):
            t = k / 7
            x = root_x + (tip_x - root_x) * t
            c = _besom_axis(min(x, x1)) if x < x1 else axis_end
            # gathered tight through the bindings, out to the splay
            g = smoothstep(x1 - 0.01, tip_x, x)
            r = ring * (1 - 0.15 * (1 - g)) + spread * 0.085 * g ** 1.25
            y = c.y + math.cos(a) * r * (0.55 + 0.45 * (1 - g))
            z = c.z + math.sin(a) * r * 1.25 + bow * g
            y += 0.004 * noise.noise(Vector((i * 1.7, t * 5.0, 0.2)))
            z += 0.005 * noise.noise(Vector((i * 1.7, t * 5.0, 3.1)))
            pts.append(Vector((x, max(y, 0.0016), z)))
        out = out + tube(pts, 0.0024, seg=3, r_end=0.0007)
        if i % 4 == 0:
            j = 4
            p = pts[j]
            d = (pts[-1] - p).normalized()
            fk = [p, p + d * 0.05 + Vector((0, rnd(-0.01, 0.01), rnd(-0.02, 0.02))),
                  p + d * 0.10 + Vector((0, rnd(-0.015, 0.015), rnd(-0.035, 0.035)))]
            for q in fk:
                q.y = max(q.y, 0.0012)
            out = out + tube(fk, 0.0011, seg=3, r_end=0.0005)
    return out


def besom_twine():
    """Two wraps of jute twine round the bound head, its ends tucked."""
    out = Shape()
    for xa, xb, turns in ((0.528, 0.548, 5), (0.578, 0.594, 4)):
        pts = []
        steps = turns * 10
        for k in range(steps + 1):
            t = k / steps
            x = xa + (xb - xa) * t
            c = _besom_axis(x)
            a = TAU * turns * t
            r = 0.0335 + 0.0006 * math.sin(a * 3.0)
            pts.append(c + Vector((0, math.cos(a) * r, math.sin(a) * r * 1.02)))
        out = out + tube(pts, 0.0019, seg=5)
    return out


auto('Real__besomHandle', besom_handle, 600, smooth=60)
auto('Real__besomTwigs', besom_twigs, 3000, smooth=60)
auto('Real__besomTwine', besom_twine, 1100, smooth=60)


# ============================================================= cauldron ===
# A black plastic cauldron, the 6 in candy-bowl kind (PMU 6 in): a
# pot-bellied bowl of moulded polypropylene, 15 cm across its belly, a rolled
# lip, a rib round its shoulder, three stubby feet, two lugs at the rim and a
# plastic bail through them. Its frame: the feet on the floor (y = 0), its
# axis on y, the lugs along x.
CAUL_PROFILE = [(0.0, 0.0125), (0.030, 0.0137), (0.050, 0.0215), (0.0640, 0.0345), (0.0720, 0.0520),
                (0.0745, 0.0680), (0.0730, 0.0820), (0.0690, 0.0950), (0.0640, 0.1050), (0.0628, 0.1110)]
CAUL_WALL = 0.0024
CAUL_LUG_Y = 0.1040
CAUL_WATER = 0.0620


def _caul_r(y):
    """The bowl's outside radius at height y."""
    pr = CAUL_PROFILE
    for (r0, y0), (r1, y1) in zip(pr, pr[1:]):
        if y0 <= y <= y1:
            return r0 + (r1 - r0) * (y - y0) / (y1 - y0)
    return pr[-1][0]


def _caul_floor(r):
    """The inside bottom's height at radius r (as `vessel` makes it: the
    outside moved in by the wall, over a flat floor)."""
    y0 = CAUL_PROFILE[0][1] + CAUL_WALL
    pr = [(0.0, y0)] + [(rr - CAUL_WALL, y) for rr, y in CAUL_PROFILE[1:] if y >= y0]
    for (r0, ya), (r1, yb) in zip(pr, pr[1:]):
        if r0 <= r <= r1:
            return ya + (yb - ya) * (r - r0) / (r1 - r0)
    return pr[-1][1]


def _caul_r_bottom(r):
    """The outside bottom's height at radius r."""
    pr = CAUL_PROFILE
    for (r0, y0), (r1, y1) in zip(pr, pr[1:]):
        if r0 <= r <= r1:
            return y0 + (y1 - y0) * (r - r0) / (r1 - r0)
    return pr[-1][1]


def cauldron_pot():
    s = vessel(CAUL_PROFILE, CAUL_WALL, seg=34)
    rim_r, rim_y = CAUL_PROFILE[-1]
    s = s + torus(rim_r + 0.0012, 0.0036, seg=34, tseg=8).rot(math.pi / 2, 0, 0).move(0, rim_y + 0.0006, 0)
    s = s + torus(_caul_r(0.090) - 0.0002, 0.0017, seg=34, tseg=6).rot(math.pi / 2, 0, 0).move(0, 0.090, 0)
    # the feet: short and splayed, their pads flat on the floor
    for k in range(3):
        a = TAU * k / 3 + math.pi / 2
        x, z = 0.041 * math.cos(a), 0.041 * math.sin(a)
        top = _caul_r_bottom(0.041) + 0.002
        foot = lathe([(0.0, 0.0), (0.0095, 0.0), (0.0108, 0.0018), (0.0102, top * 0.5), (0.0090, top)], seg=10, cap_top=True)
        s = s + foot.move(x, 0, z)
    # the lugs at the rim, bored across for the bail
    for sx in (-1, 1):
        lug = rbox(0.008, 0.016, 0.012, r=0.002, seg=1)
        lug = cut(lug, cyl(0.0030, h=0.02, seg=10, bevel_r=0).rot(0, 0, math.pi / 2).move(0, -0.0015, 0))
        s = s + lug.move(sx * (_caul_r(CAUL_LUG_Y) + 0.0025), CAUL_LUG_Y + 0.0015, 0)
    return s


def cauldron_bail():
    """The bail: a plastic rod through the lugs, its ends hooked inside them,
    arched over the pot and fallen back a little toward -z."""
    R = _caul_r(CAUL_LUG_Y) + 0.0025
    pts = []
    n = 24
    for k in range(n + 1):
        a = math.pi * k / n
        pts.append(Vector((-R * math.cos(a), 0.070 * math.sin(a), 0)))
    s = tube(pts, 0.0024, seg=6)
    for sx in (-1, 1):
        s = s + tube([Vector((sx * (R + 0.007), 0, 0)), Vector((sx * (R - 0.004), 0, 0)), Vector((sx * (R - 0.006), -0.004, 0))], 0.0022, seg=6)
    s.rot(-0.42, 0, 0)
    return s.move(0, CAUL_LUG_Y + 0.0001, 0)


def cauldron_water():
    """Standing water, a little meniscus up the wall."""
    y = CAUL_WATER
    r = _caul_r(y) - CAUL_WALL - 0.0002
    s = lathe([(0.0, y), (r * 0.5, y), (r * 0.92, y), (r - 0.0004, y + 0.0012)], seg=40, cap_bottom=False, cap_top=False)
    return s


def cauldron_dust():
    """The inside bottom's dust, laid on it."""
    prof = [(0.0, _caul_floor(0.0) + 0.0006)]
    for r in (0.012, 0.024, 0.036, 0.044, 0.050, 0.056):
        prof.append((r, _caul_floor(r) + 0.0006))
    return lathe(prof, seg=32, cap_bottom=False, cap_top=False)


def cauldron_wrapper():
    """A twist wrapper (a sweet's), emptied and crumpled: its middle flat,
    the two twisted ends fanned out."""
    body = ellipsoid(0.0105, 0.0030, 0.0068, seg=12)
    body = noisy(body, 0.0012, scale=420.0, seed=3)
    out = body
    for sx in (-1, 1):
        rings = []
        for k in range(6):
            t = k / 5
            x = sx * (0.0085 + 0.012 * t)
            w = 0.0035 * (1 - t) + 0.0080 * t ** 1.5 + 0.0010
            h = 0.0012 + 0.0006 * t
            ring = []
            for i in range(8):
                a = TAU * i / 8 + t * 2.2 * sx
                ring.append(Vector((x, math.sin(a) * h, math.cos(a) * w * (1 + 0.25 * math.sin(3 * a)))))
            rings.append(ring)
        out = out + loft(rings)
    out = noisy(out, 0.0007, scale=600.0, seed=5)
    lo, _ = out.bounds()
    return out.move(0, -lo.y, 0)


auto('Real__cauldronPot', cauldron_pot, 2700, smooth=50, kind='cyl')
auto('Real__cauldronBail', cauldron_bail, 560, smooth=50)
auto('Real__cauldronWater', cauldron_water, 400, smooth=80)
auto('Real__cauldronDust', cauldron_dust, 400, smooth=80)
auto('Real__cauldronWrapper', cauldron_wrapper, 400, smooth=40)


# ================================================================ ghost ===
# A cheesecloth ghost, the tabletop kind (10 in, 25 cm): a square of
# cheesecloth soaked in fabric stiffener and draped over a ball on a bottle,
# a string tied round under the ball for its neck; dried stiff, the form
# taken out, two black felt eyes glued on. The drape is Blender's cloth: a
# 44 cm square let fall over the form onto the floor, then drawn in at the
# neck where the string was. Its frame: the hem on the floor (y = 0), its
# face toward +z.
GHOST_HEAD = Vector((0.0, 0.200, 0.0))
GHOST_HEAD_R = 0.046
GH_N = 30                         # the cloth's grid: GH_N x GH_N cells
GH_SIZE = 0.44
GH_NECK = 0.148                   # where the string was tied


def ghost_drape():
    import time
    t0 = time.time()
    n = GH_N
    pts, faces = [], []
    for j in range(n + 1):
        for i in range(n + 1):
            x = -GH_SIZE / 2 + GH_SIZE * i / n
            z = -GH_SIZE / 2 + GH_SIZE * j / n
            # a little crumple, so it does not fall as a flat sheet would
            y = GHOST_HEAD.y + GHOST_HEAD_R + 0.006 + 0.004 * noise.noise(Vector((x * 11.0, z * 11.0, 0.5)))
            pts.append((x, y, z))
    for j in range(n):
        for i in range(n):
            a = j * (n + 1) + i
            faces.append((a, a + 1, a + n + 2, a + n + 1))
    me = bpy.data.meshes.new('ghost_sim')
    me.from_pydata(pts, [], faces)
    me.update()
    ob = bpy.data.objects.new('ghost_sim', me)
    bpy.context.scene.collection.objects.link(ob)
    # the form: the ball, the bottle, the stick for its arms, the floor
    form = [
        sphere(GHOST_HEAD_R, seg=24, rings=12).move(*GHOST_HEAD),
        lathe([(0.0, 0.0), (0.034, 0.0), (0.034, 0.120), (0.020, 0.150), (0.013, 0.162), (0.0, 0.162)], seg=16),
        Shape([(-1, 0, -1), (1, 0, -1), (1, 0, 1), (-1, 0, 1)], [(0, 3, 2, 1)]),
    ]
    cols = []
    for k, f in enumerate(form):
        co = _mesh_obj(f'ghost_form{k}', f)
        co.modifiers.new('col', 'COLLISION')
        co.collision.thickness_outer = 0.003
        co.collision.cloth_friction = 12.0
        cols.append(co)
    sc = bpy.context.scene
    grav = tuple(sc.gravity)
    sc.gravity = (0.0, -9.81, 0.0)
    cl = ob.modifiers.new('cloth', 'CLOTH')
    s = cl.settings
    s.quality = 8
    s.mass = 0.02
    s.tension_stiffness = 10.0
    s.compression_stiffness = 10.0
    s.shear_stiffness = 4.0
    s.bending_stiffness = 0.03
    s.air_damping = 2.0
    c = cl.collision_settings
    c.use_collision = True
    c.distance_min = 0.003
    c.use_self_collision = True
    c.self_distance_min = 0.002
    c.collision_quality = 4
    frames = 160
    sc.frame_start, sc.frame_end = 1, frames
    cl.point_cache.frame_start, cl.point_cache.frame_end = 1, frames
    for fr in range(1, frames + 1):
        sc.frame_set(fr)
    dg = bpy.context.evaluated_depsgraph_get()
    em = ob.evaluated_get(dg).to_mesh()
    out = [Vector(vv.co) for vv in em.vertices]
    ob.evaluated_get(dg).to_mesh_clear()
    sc.frame_set(1)
    sc.gravity = grav
    bpy.data.objects.remove(ob)
    bpy.data.meshes.remove(me)
    for co in cols:
        _drop(co)
    print('t08: the ghost draped in %.1f s' % (time.time() - t0))
    return out, faces


_ghost = {}


def ghost_frame():
    if 'cloth' not in _ghost:
        P, faces = ghost_drape()
        for p in P:
            # drawn in at the neck by the string (strongest at its height,
            # nearest the form), the folds above and below gathered to it
            r = math.hypot(p.x, p.z)
            w = math.exp(-((p.y - GH_NECK) / 0.022) ** 2) * (1 - smoothstep(0.06, 0.11, r))
            if r > 1e-6 and w > 0:
                k = (r + (0.021 + 0.004 * noise.noise(Vector((p.x * 60, p.z * 60, 0.3))) - r) * w) / r
                p.x *= k
                p.z *= k
            p.y = max(p.y, 0.0008)
        s = Shape(P, faces)
        s = subdivide(s, 1)
        s = decimate(s, 2300)
        # its faces turned out from the form (the felt is laid on the outside)
        out = []
        for f in s.f:
            q = [s.v[i] for i in f]
            nn = (q[1] - q[0]).cross(q[2] - q[0])
            m = sum(q, Vector((0, 0, 0))) / len(q)
            c = Vector((0, min(max(m.y, 0.05), GHOST_HEAD.y), 0))
            out.append(tuple(reversed(f)) if nn.dot(m - c) < 0 else f)
        s.f = out
        _ghost['cloth'] = s
        _ghost['bvh'] = bvh_of(s)
    return _ghost


def ghost_cloth():
    return ghost_frame()['cloth'].copy()


def ghost_eyes():
    """Two ovals of black felt glued on the face, a little uneven."""
    bvh = ghost_frame()['bvh']
    out = Shape()
    for sx, tilt in ((-1, 0.12), (1, -0.05)):
        d = Vector((sx * math.sin(math.radians(19)), 0.06, math.cos(math.radians(19)))).normalized()
        hit = bvh.ray_cast(GHOST_HEAD, d, 0.2)
        p, n = hit[0], hit[1].normalized()
        if n.dot(d) < 0:
            n = -n
        out = out + project_patch(bvh, p, n, (tilt, 1, 0), 0.0135, 0.0190, 0.0012, seg=14, rings=2)
    return out


auto('Real__ghostCloth', ghost_cloth, 2400, smooth=70)
auto('Real__ghostEyes', ghost_eyes, 200, smooth=60)


# ============================================================== pumpkin ===
# A small carving pumpkin (a pie pumpkin, 19 cm across, 14 cm tall) carved
# into a jack-o'-lantern (two triangle eyes, a nose, a mouth with two teeth;
# a lid cut round the stem at a slant) and left: gone soft, it has slumped
# on itself, its top sunk, its lid fallen in askew, the skin wrinkled. Its
# frame: its bottom on the floor (y = 0), the face toward +z. The skin and
# the cut flesh are parts of their own, and the tea light left inside it.
PK_R, PK_H = 0.093, 0.146
PK_WALL = 0.015
PK_LOBES = 10


def _pk_surface(inset=0.0, seg=44, rings=24):
    """The pumpkin's skin (inset: moved in by that, for the inside), lobed,
    dimpled at the stem and the blossom end; a closed surface."""
    v, f = [], []
    rows = []
    for j in range(rings + 1):
        t = j / rings
        ph = math.pi * t
        row = []
        for i in range(seg):
            a = TAU * i / seg
            lobe = 0.5 + 0.5 * math.cos(PK_LOBES * (a + math.pi / PK_LOBES))
            r = PK_R * math.sin(ph) ** 0.85 * (0.915 + 0.085 * lobe ** 0.6)
            y = PK_H / 2 * (1 - math.cos(ph))
            # the dimples: the stem's cavity and the blossom end
            y -= 0.020 * math.exp(-((1 - t) / 0.16) ** 2) - 0.0
            y += 0.006 * math.exp(-(t / 0.12) ** 2)
            p = Vector((r * math.sin(a), y, r * math.cos(a)))
            if inset > 0:
                c = Vector((0, PK_H * 0.48, 0))
                d = p - c
                L = d.length
                p = c + d * max(0.0, (L - inset) / L)
            row.append(len(v))
            v.append(p)
        rows.append(row)
    for j in range(rings):
        for i in range(seg):
            i2 = (i + 1) % seg
            f.append((rows[j][i], rows[j][i2], rows[j + 1][i2], rows[j + 1][i]))
    s = Shape(v, f)
    s = weld(s, 1e-7)
    return propkit._fix_winding(s)


def _pk_face_cutters():
    """The face, carved straight in from the front: two eyes, a nose, the
    mouth with its two teeth left standing."""
    z0, z1 = 0.0, 0.2
    d = z1 - z0
    out = []
    for sx in (-1, 1):
        out.append([(sx * 0.016, 0.078), (sx * 0.046, 0.080), (sx * 0.032, 0.104)])          # the eyes
    out.append([(-0.009, 0.062), (0.009, 0.062), (0.0, 0.077)])                                # the nose
    mouth = []
    n = 14
    for k in range(n + 1):
        x = -0.050 + 0.100 * k / n
        mouth.append((x, 0.050 - 0.016 * (1 - (x / 0.050) ** 2) - 0.0))
    low = []
    for k in range(n + 1):
        x = 0.050 - 0.100 * k / n
        y = 0.050 - 0.034 * (1 - (x / 0.050) ** 2) ** 0.8
        # the two teeth: notches left standing, one up from the bottom lip,
        # one down from the top
        if abs(x - 0.016) < 0.006:
            y += 0.010
        low.append((x, y))
    top = [(x, y) for x, y in mouth]
    for k, (x, y) in enumerate(top):
        if abs(x + 0.014) < 0.006:
            top[k] = (x, y - 0.009)
    out.append(top + low)
    cutters = []
    for poly in out:
        pr = prism(poly, d)
        cutters.append(pr.move(0, 0, z0 + d / 2))
    return cutters


def _pk_rot(p, lid=False):
    """Gone soft: slumped (the top lower, the sides out), the top sunk round
    the lid, the skin wrinkled and sagging."""
    q = p.copy()
    h = max(0.0, q.y) / PK_H
    r = math.hypot(q.x, q.z)
    # slump: the top settles, the waist bulges
    q.y -= 0.016 * h ** 1.6
    bulge = 1 + 0.05 * math.sin(math.pi * min(1.0, h * 1.1)) ** 1.5
    q.x *= bulge
    q.z *= bulge
    # the top sinks in round the lid
    q.y -= 0.008 * math.exp(-(r / 0.055) ** 2) * h
    # sags and wrinkles
    n1 = noise.noise(Vector((q.x * 22.0, q.y * 22.0, q.z * 22.0)))
    n2 = noise.noise(Vector((q.x * 70.0 + 3.1, q.y * 55.0, q.z * 70.0)))
    if r > 1e-6:
        k = 1 + (0.035 * n1 + 0.012 * n2) * min(1.0, h * 2.0)
        q.x *= k
        q.z *= k
    q.y += 0.004 * n1 * h
    # the face's side has sagged forward and down a little more
    if q.z > 0:
        q.y -= 0.006 * (q.z / PK_R) * h
    return q


_pk = {}


def _classify(s):
    """Split a carved shell into its skin (faces turned out from the middle)
    and its flesh (the inside and the cut faces)."""
    c = Vector((0, PK_H * 0.48, 0))
    skin, flesh = [], []
    for f in s.f:
        q = [s.v[i] for i in f]
        n = (q[1] - q[0]).cross(q[2] - q[0])
        if len(q) > 3:
            n += (q[2] - q[0]).cross(q[3] - q[0])
        m = sum(q, Vector((0, 0, 0))) / len(q)
        d = (m - c)
        d.y *= 1.4
        k = n.normalized().dot(d.normalized()) if n.length > 1e-12 else 0
        (skin if k > 0.45 else flesh).append(f)
    return Shape([p.copy() for p in s.v], skin), Shape([p.copy() for p in s.v], flesh)


def _compact(s):
    used = sorted({i for f in s.f for i in f})
    m = {o: n for n, o in enumerate(used)}
    return Shape([s.v[i] for i in used], [tuple(m[i] for i in f) for f in s.f])


def pumpkin_build():
    if _pk:
        return _pk
    outer = _pk_surface()
    inner = _pk_surface(PK_WALL)
    inner.f = [tuple(reversed(f)) for f in inner.f]
    shell = outer + inner
    for c in _pk_face_cutters():
        shell = boolean(shell, c, 'DIFFERENCE')
    # the lid: cut round the stem with the knife slanted in
    knife = lathe([(0.0, PK_H * 0.55), (0.024, PK_H * 0.55), (0.044, PK_H + 0.02), (0.0, PK_H + 0.02)], seg=24)
    lid = boolean(shell, knife, 'INTERSECT')
    shell = boolean(shell, knife, 'DIFFERENCE')
    shell = decimate(shell, 3000)
    lid = decimate(lid, 500)
    # the lid fallen in: down into the hole, tipped
    lc = Vector((0, PK_H - 0.008, 0))

    def lid_place(p):
        q = p - lc
        q.rotate(Euler((0.22, 0.4, -0.16), 'XYZ'))
        return q + lc + Vector((0.004, -0.017, -0.003))
    lid.warp(lid_place)
    stem_base = lid_place(Vector((0, PK_H - 0.016, 0)))
    stem_dir = (lid_place(Vector((0, PK_H + 0.02, 0))) - stem_base).normalized()
    sk1, fl1 = _classify(shell)
    sk2, fl2 = _classify(lid)
    skin = _compact(sk1) + _compact(sk2)
    flesh = _compact(fl1) + _compact(fl2)
    skin.warp(_pk_rot)
    flesh.warp(_pk_rot)
    sb = _pk_rot(stem_base)
    lo = min(min(p.y for p in skin.v), min(p.y for p in flesh.v))
    skin.move(0, -lo, 0)
    flesh.move(0, -lo, 0)
    _pk['skin'] = skin
    _pk['flesh'] = flesh
    _pk['stem'] = (sb - Vector((0, lo, 0)), stem_dir)
    # the inside floor under the middle, where the tea light stands
    bvh = bvh_of(flesh)
    hit = bvh.ray_cast(Vector((0.002, 0.045, 0.004)), Vector((0, -1, 0)), 0.2)
    _pk['floor'] = hit[0].y if hit[0] is not None else PK_WALL
    print('t08: the pumpkin\'s inside floor at y = %.4f' % _pk['floor'])
    return _pk


def pumpkin_skin():
    return pumpkin_build()['skin'].copy()


def pumpkin_flesh():
    return pumpkin_build()['flesh'].copy()


def pumpkin_stem():
    """The stem, dried hard: five ridges, twisted, its cut end splintered."""
    base, d = pumpkin_build()['stem']
    rings = []
    n, seg = 6, 10
    a, b = frame_along(d)
    for k in range(n + 1):
        t = k / n
        c = base + d * (0.034 * t) + a * (0.006 * t * t)
        r = 0.0105 * (1 - 0.35 * t) + (0.006 if k == 0 else 0.0)
        ring = []
        for i in range(seg):
            ang = TAU * i / seg + t * 0.6
            rr = r * (0.82 + 0.18 * (0.5 + 0.5 * math.cos(5 * ang)))
            ring.append(c + a * (math.cos(ang) * rr) + b * (math.sin(ang) * rr))
        rings.append(ring)
    return loft(rings)


def pumpkin_tealight():
    """A tea light in its aluminium cup, burnt down, on the inside floor:
    the cup (38 mm, 16 mm tall)."""
    y0 = pumpkin_build()['floor']
    cup = vessel([(0.0, 0.0), (0.0185, 0.0), (0.0190, 0.0010), (0.0192, 0.0160)], 0.0004, seg=24)
    return cup.move(0.002, y0, 0.004)


def pumpkin_wax():
    """The wax left in the cup (4 mm), its wick."""
    y0 = pumpkin_build()['floor']
    wax = cyl(0.0186, h=0.0045, seg=24, bevel_r=0.0004).move(0.002, y0 + 0.0004 + 0.00225, 0.004)
    wick = tube([(0.002, y0 + 0.004, 0.004), (0.0024, y0 + 0.0085, 0.004), (0.0032, y0 + 0.0105, 0.004)], 0.0006, seg=5)
    return wax + wick


auto('Real__pumpkinSkin', pumpkin_skin, 2600, smooth=60, kind='sphere')
auto('Real__pumpkinFlesh', pumpkin_flesh, 2400, smooth=50, kind='sphere')
auto('Real__pumpkinStem', pumpkin_stem, 280, smooth=40)
auto('Real__pumpkinTealight', pumpkin_tealight, 340, smooth=40, kind='cyl')
auto('Real__pumpkinWax', pumpkin_wax, 340, smooth=50)
