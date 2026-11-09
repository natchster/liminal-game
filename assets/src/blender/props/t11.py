"""t11: the props remodelled at their real sizes (docs/PROPS.md, P3, the
rare): a hunting lodge's trophies and a parlour's ornaments, years after
anyone dusted them: the roe buck's antlers on their skull plate and oak
shield, a trout mounted on its oval board, a roe buck's head on its shield,
a glazed floor vase with peacock feathers and a lady's paper folding fan
dropped open on the floor. Sources and sizes: t11.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame (x across its front, y up, z out of its front, a
wall mount's origin on the wall behind it at its hanging height), unless
its section says otherwise.

The antlers (`roeAntlers`, `roeAntlerTips`) are one pair shared by the two
roe trophies, in a frame of their own (the antler frame, A): the burrs'
bases about the origin, the antlers rising along +y, the brow tines toward
+z (the buck's forward). A skull's own frame (H) has the frontal bone's
face level (+y up out of it) and the nose toward +z; the antlers leave the
pedicles leaning back by ROE_BETA, so a point of A is R_x(-ROE_BETA) of H.
The game turns the pair about x by (the skull's turn) - ROE_BETA.
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


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def lerp(a, b, t):
    return a + (b - a) * t


# ------------------------------------------------------------- helpers ---
_meta_n = [0]
MB_K = 1.0 / 0.564


def meta(elems, res=0.002, threshold=0.6):
    """A blobby surface from metaball elements (r03's, t08's, t09's):
    ('ball', centre, radius[, stiffness]), ('ell', centre, (rx, ry, rz)[,
    stiffness[, euler]]), ('cap', a, b, radius[, stiffness]); '-ball' /
    '-ell' / '-cap' subtract. Sizes are the element's own surface."""
    _meta_n[0] += 1
    name = f'_t11mb{_meta_n[0]}x'
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
    ob = _mesh_obj('_t11dec', s)
    m = ob.modifiers.new('d', 'DECIMATE')
    m.decimate_type = 'COLLAPSE'
    m.ratio = tris / n
    m.use_collapse_triangulate = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def smooth_shape(s, factor=0.5, iterations=4):
    """Blender's smooth modifier."""
    ob = _mesh_obj('_t11sm', s)
    m = ob.modifiers.new('s', 'SMOOTH')
    m.factor = factor
    m.iterations = iterations
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def remesh(s, voxel):
    """A voxel remesh: overlapping shells fused into one surface."""
    ob = _mesh_obj('_t11rm', s)
    m = ob.modifiers.new('r', 'REMESH')
    m.mode = 'VOXEL'
    m.voxel_size = voxel
    m.adaptivity = 0.0
    m.use_smooth_shade = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return _fix(out)


def loft_rings(rings, cap=True):
    """Close a stack of equal rings (lists of points) into a solid."""
    n, seg = len(rings), len(rings[0])
    v = [Vector(p) for r in rings for p in r]
    f = []
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    if cap:
        f.append(tuple(reversed(range(seg))))
        f.append(tuple((n - 1) * seg + j for j in range(seg)))
    return propkit._fix_winding(Shape(v, f))


def vnormals(s):
    """Area-weighted vertex normals of a shape."""
    nr = [Vector((0, 0, 0)) for _ in s.v]
    for f in s.f:
        a = s.v[f[0]]
        for i in range(1, len(f) - 1):
            n = (s.v[f[i]] - a).cross(s.v[f[i + 1]] - a)
            for k in (f[0], f[i], f[i + 1]):
                nr[k] += n
    return [n.normalized() if n.length > 1e-12 else Vector((0, 1, 0)) for n in nr]


def split_faces(s, keep):
    """The faces of `s` whose centroid (and normal) `keep(c, n)` accepts, as
    a shape of their own (the rest is `split_faces(s, not keep)`)."""
    v, f, ix = [], [], {}
    for fc in s.f:
        c = sum((s.v[i] for i in fc), Vector((0, 0, 0))) / len(fc)
        a = s.v[fc[0]]
        n = (s.v[fc[1]] - a).cross(s.v[fc[2]] - a)
        n = n.normalized() if n.length > 1e-12 else Vector((0, 1, 0))
        if not keep(c, n):
            continue
        out = []
        for i in fc:
            if i not in ix:
                ix[i] = len(v)
                v.append(s.v[i].copy())
            out.append(ix[i])
        f.append(tuple(out))
    return Shape(v, f)


def bump_along_normals(s, fn):
    """Push every vertex along its normal by fn(p) metres."""
    ns = vnormals(s)
    out = s.copy()
    for i, p in enumerate(out.v):
        out.v[i] = p + ns[i] * fn(p)
    return out


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


def shield_outline(w, h, notch=0.008, r=0.004):
    """A hunting trophy's shield (a heater-shaped Wappen), centred on x:
    rounded shoulders, a small peak at the middle of its top, straight
    sides, then two arcs of radius w meeting at its foot's point
    (y = -h/2)."""
    hw, top, bot = w / 2, h / 2, -h / 2
    ys = bot + math.sqrt(w * w - hw * hw)
    right = [(hw, top - 0.016)]
    for k in range(0, 9):
        a = -math.radians(60.0) * k / 8
        right.append((-hw + w * math.cos(a), ys + w * math.sin(a)))
    right[-1] = (0.0, bot)
    pts = [(-0.022, top), (0.0, top + notch), (0.022, top), (hw - 0.016, top)] + right
    pts += [(-x, y) for x, y in reversed(right[:-1])] + [(-hw + 0.016, top)]
    # round the corners only (not the arcs' gentle turns)
    out, n = [], len(pts)
    for i in range(n):
        a, p, b = Vector(pts[i - 1]), Vector(pts[i]), Vector(pts[(i + 1) % n])
        turn = (p - a).normalized().angle((b - p).normalized(), 0.0)
        out += propkit._round_corner(pts[i], pts[i - 1], pts[(i + 1) % n], r, 2) if turn > 0.3 else [pts[i]]
    return out


def shield(w, h, t, field=0.004):
    """The oak shield: its board (the front's edge bevelled round, the back
    square) and a raised field inset from its edge, its edge moulded."""
    board = prism(shield_outline(w, h), t, bevel_r=0.0035, bevel_seg=2)
    # the back's edge square, the front's rounded
    board.move(0, 0, t / 2)
    inner = [(x * (w - 0.026) / w, y * (h - 0.026) / h) for x, y in shield_outline(w, h)]
    raised = prism(inner, field + 0.002, bevel_r=0.0016, bevel_seg=1)
    raised.move(0, 0, t + field / 2 - 0.001)
    return board + raised


# ============================================================== antlers ===
# A roe buck's antlers (Capreolus capreolus), a six-pointer of the common
# kind: 21 cm on the beam, each with its burr (the pearled coronet), the
# beam pearled and guttered over its lower half, a brow tine forward from
# 55 % of the way up, a back tine at 80 %, the top curving in a little
# (Black Forest trophies: 23 x 14 x 17 cm on their shields).
ROE_BETA = math.radians(52.0)     # the antlers' lean back off the frontal
ROE_PED_X = 0.020                 # the pedicles' half spacing


def _antler_paths(sx):
    """One antler's beam and tines (points in A, metres) and radii."""
    X = lambda x: sx * x
    beam = [(X(0.020), 0.000, 0.000), (X(0.021), 0.020, -0.003), (X(0.026), 0.055, -0.008),
            (X(0.033), 0.095, -0.011), (X(0.038), 0.125, -0.011), (X(0.041), 0.155, -0.008),
            (X(0.040), 0.178, -0.005), (X(0.035), 0.197, 0.001), (X(0.028), 0.210, 0.008)]
    beam_r = [0.0110, 0.0098, 0.0085, 0.0072, 0.0065, 0.0057, 0.0049, 0.0037, 0.0016]
    brow = [(X(0.037), 0.117, -0.009), (X(0.040), 0.128, 0.008), (X(0.043), 0.140, 0.026), (X(0.045), 0.156, 0.041)]
    brow_r = [0.0052, 0.0044, 0.0032, 0.0013]
    back = [(X(0.040), 0.168, -0.007), (X(0.042), 0.176, -0.022), (X(0.044), 0.186, -0.036)]
    back_r = [0.0042, 0.0030, 0.0012]
    return [(beam, beam_r), (brow, brow_r), (back, back_r)]


def _sweep(pts, radii, seg=10, sub=3):
    """A tube through `pts`, its radius from `radii`, subdivided along."""
    P = [Vector(p) for p in pts]
    q, rq = [], []
    for i in range(len(P) - 1):
        for k in range(sub):
            t = k / sub
            # Catmull-Rom through the points
            p0, p1, p2, p3 = P[max(i - 1, 0)], P[i], P[i + 1], P[min(i + 2, len(P) - 1)]
            t2, t3 = t * t, t * t * t
            q.append(0.5 * ((2 * p1) + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t2 + (-p0 + 3 * p1 - 3 * p2 + p3) * t3))
            rq.append(lerp(radii[i], radii[i + 1], t))
    q.append(P[-1])
    rq.append(radii[-1])
    out = Shape()
    # rings
    n = len(q)
    tangents = [(q[min(i + 1, n - 1)] - q[max(i - 1, 0)]).normalized() for i in range(n)]
    up = Vector((1, 0, 0))
    nrm = tangents[0].cross(up).normalized()
    v, f = [], []
    for i in range(n):
        if i:
            nrm = (tangents[i - 1].rotation_difference(tangents[i]) @ nrm).normalized()
        b = tangents[i].cross(nrm).normalized()
        for j in range(seg):
            a = TAU * j / seg
            v.append(q[i] + (nrm * math.cos(a) + b * math.sin(a)) * rq[i])
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    v.append(q[0] - tangents[0] * rq[0] * 0.3)
    f += [(len(v) - 1, (j + 1) % seg, j) for j in range(seg)]
    v.append(q[-1] + tangents[-1] * rq[-1])
    k = (n - 1) * seg
    f += [(len(v) - 1, k + j, k + (j + 1) % seg) for j in range(seg)]
    return propkit._fix_winding(Shape(v, f))


def _antler_tips():
    """Where each tine ends, with its direction (A)."""
    tips = []
    for sx in (-1, 1):
        for pts, _ in _antler_paths(sx):
            a, b = Vector(pts[-2]), Vector(pts[-1])
            tips.append((b, (b - a).normalized()))
    return tips


def antler_pair():
    """Both antlers, remeshed into one surface: the burrs a ring of pearls
    round each beam's foot, the beams pearled where the velvet's vessels
    ran, the tines smooth toward their points."""
    def build():
        s = Shape()
        for sx in (-1, 1):
            for pts, rr in _antler_paths(sx):
                s = s + _sweep(pts, rr)
            # the burr: a thick ring of pearls round the beam's foot
            burr = torus(0.0118, 0.0050, seg=24, tseg=8).rot(math.pi / 2, 0, 0).move(sx * ROE_PED_X, 0.0040, 0)
            s = s + burr
            for k in range(19):
                a = TAU * k / 19 + sx * 0.3
                r = 0.0140 + 0.0012 * math.sin(k * 2.3)
                s = s + sphere(0.0030 + 0.0007 * math.sin(k * 1.7), seg=6, rings=4).move(
                    sx * ROE_PED_X + r * math.cos(a), 0.0038 + 0.0014 * math.sin(k * 3.1), r * math.sin(a))
            # the pearls: knobs up the beam's lower half, thick at the foot
            for k in range(120):
                t = (k + 0.5) / 120
                y = 0.010 + t * 0.112
                a = k * 2.399 + sx
                pts, rr = _antler_paths(sx)[0]
                # the beam's centre and radius at y
                for i in range(len(pts) - 1):
                    if pts[i][1] <= y <= pts[i + 1][1]:
                        u = (y - pts[i][1]) / (pts[i + 1][1] - pts[i][1])
                        c = Vector(pts[i]).lerp(Vector(pts[i + 1]), u)
                        r0 = lerp(rr[i], rr[i + 1], u)
                        break
                pr = 0.0019 * (1.0 - 0.55 * t) + 0.0004 * math.sin(k * 5.1)
                d = Vector((math.cos(a), 0, math.sin(a)))
                s = s + sphere(pr, seg=6, rings=4).move(*(c + d * (r0 - pr * 0.25)))
        s = remesh(s, 0.0008)
        # gutters and grain: a fine noise along the beam
        s = bump_along_normals(s, lambda p: 0.0005 * noise.noise(Vector((p.x * 650, p.y * 70, p.z * 650))))
        return decimate(s, 5200)
    return cached('antlers', build)


def _near_tip(c):
    for tip, d in _antler_tips():
        w = c - tip
        along = -w.dot(d)            # how far back from the point
        if -0.004 < along < 0.017 and (w + d * along).length < 0.008:
            return True
    return False


def roe_antlers():
    return split_faces(antler_pair(), lambda c, n: not _near_tip(c))


def roe_antler_tips():
    return split_faces(antler_pair(), lambda c, n: _near_tip(c))


auto('Real__roeAntlers', roe_antlers, 5600, smooth=50.0)
auto('Real__roeAntlerTips', roe_antler_tips, 700, smooth=50.0)


# ============================================================ the trophy ===
# The antlers on their skull plate (the frontal bones cut through the
# orbits and the nasals, boiled, bleached) screwed to a carved oak shield
# (Wappen) 15 x 22 cm, 22 mm thick; the plate's cut against the shield, the
# brow forward, the antlers rising up and out of it. Frame: the shield's
# centre on the wall at x = y = 0 (the game hangs it 1.80 m up), its back
# at z = TROPHY_BACK.
TROPHY_BACK = -0.0140
TROPHY_SHIELD = (0.150, 0.220, 0.022)
# H (the skull) to the trophy: the frontal faces the room, the nose down
TROPHY_TURN = math.pi / 2
# where the skull's H origin (the burrs' bases) sits in the trophy frame
TROPHY_SKULL = Vector((0.0, 0.052, TROPHY_BACK + TROPHY_SHIELD[2] + 0.004 + 0.034))


def trophy_shield():
    w, h, t = TROPHY_SHIELD
    s = shield(w, h, t)
    return s.move(0, 0, TROPHY_BACK)


def skull_plate_h():
    """The roe's skull plate in H: the frontal bones' dome, the brow ridges
    and the orbits' upper rims, the nasals' roots, the pedicles; cut flat
    beneath (the saw's cut through the orbits) and across the nasals."""
    e = [('ell', (0, -0.020, 0.004), (0.034, 0.016, 0.036), 2.0),
         ('ell', (0, -0.022, 0.042), (0.019, 0.012, 0.030), 2.0),
         ('cap', (0, -0.018, 0.020), (0, -0.022, 0.070), 0.010, 2.0),
         ('ell', (0, -0.024, -0.018), (0.025, 0.012, 0.015), 2.0)]
    for sx in (-1, 1):
        e += [('ell', (sx * 0.030, -0.020, 0.026), (0.010, 0.010, 0.013), 2.0),
              ('cap', (sx * 0.024, -0.014, 0.012), (sx * 0.041, -0.021, 0.024), 0.0055, 2.0),
              ('cap', (sx * 0.041, -0.021, 0.024), (sx * 0.044, -0.030, 0.040), 0.0045, 2.0),
              ('-ball', (sx * 0.046, -0.028, 0.032), 0.012, 2.0),
              ('cap', (sx * ROE_PED_X, -0.017, 0.010), (sx * ROE_PED_X, -0.0015, 0.001), 0.0085, 2.0)]
    # the parting between the frontals: a shallow groove down the middle
    e.append(('-cap', (0, -0.002, -0.020), (0, -0.007, 0.060), 0.0028, 2.0))
    s = meta(e, res=0.0012)
    s = remesh(s, 0.0011)
    # sawn flat under, and across the nasals and the back
    s = cut(s, box(0.2, 0.1, 0.2).move(0, -0.034 - 0.05, 0))
    s = cut(s, box(0.2, 0.2, 0.1).move(0, 0, 0.074 + 0.05))
    # bone's grain: fine pits and the sutures' wander
    s = bump_along_normals(s, lambda p: 0.00025 * noise.noise(Vector((p.x * 500, p.y * 500, p.z * 500))))
    return decimate(s, 2200)


def _h_to_trophy(s):
    s = s.copy()
    s.v = [rot_x(p, TROPHY_TURN) + TROPHY_SKULL for p in s.v]
    return propkit._fix_winding(s)


def trophy_skull():
    return _h_to_trophy(cached('skullplate', skull_plate_h))


auto('Real__roeTrophyShield', trophy_shield, 900, smooth=40.0)
auto('Real__roeTrophySkull', trophy_skull, 2300, smooth=55.0)


# ================================================================ fish ===
# A trout skin-mounted on an oval board, as a taxidermist mounted them for
# fifty years: the skin over a carved body, its show side out and its back
# flat to the board, the tail swept out off it, the fins spread, a glass
# eye; painted, lacquered (t11.notes.md). 30 cm nose to tail (a 12 in fish)
# on a 35 x 15.5 cm walnut-stained oval with an ogee edge. Frame: the
# board's centre on the wall (the game hangs it 1.65 m up), the board's
# back at z = FISH_BACK; the fish's head to -x.
FISH_BACK, FISH_FRONT = -0.009, 0.010
FISH_L = 0.300                   # nose to the tail's tips
FISH_X0 = -0.150                 # the snout
FISH_BODY = 0.255                # snout to the tail's root


def _prof(s, pts):
    """Piecewise-linear profile [(s, value)] at s."""
    for (s0, v0), (s1, v1) in zip(pts, pts[1:]):
        if s0 <= s <= s1:
            t = (s - s0) / (s1 - s0)
            t = t * t * (3 - 2 * t)
            return lerp(v0, v1, t)
    return pts[-1][1]


FISH_DEPTH = [(0.0, 0.08), (0.03, 0.30), (0.08, 0.52), (0.16, 0.74), (0.24, 0.88), (0.36, 1.0), (0.5, 0.96),
              (0.64, 0.80), (0.78, 0.56), (0.88, 0.44), (0.95, 0.42), (1.0, 0.44)]
FISH_THICK = [(0.0, 0.08), (0.03, 0.32), (0.10, 0.62), (0.24, 0.92), (0.36, 1.0), (0.5, 0.94), (0.65, 0.70), (0.82, 0.46),
              (0.95, 0.36), (1.0, 0.34)]
FISH_H, FISH_T = 0.0330, 0.0185     # half its depth, its show side's bulge


def fish_station(s):
    """At s (0 the snout, 1 the tail's root): x, the centre's y, half the
    depth, the bulge out and the centre's z (the tail swept off the board)."""
    x = FISH_X0 + FISH_BODY * s
    h = FISH_H * _prof(s, FISH_DEPTH)
    t = FISH_T * _prof(s, FISH_THICK)
    yc = 0.0025 * math.sin(math.pi * s) - 0.002 * s
    sweep = 0.013 * smoothstep(0.5, 1.0, s) ** 1.5
    zc = FISH_FRONT + 0.32 * t + sweep
    return x, yc, h, t, zc


def fish_point(s, th):
    x, yc, h, t, zc = fish_station(s)
    c, sn = math.cos(th), math.sin(th)
    # the gill cover's edge: a curve over the side, furthest back at the
    # middle, the skin a step lower just behind it (none along the back)
    if sn < 0.6:
        se = 0.205 + 0.032 * max(0.0, c) ** 2 - 0.01 * max(0.0, -sn)
        if se < s < se + 0.016:
            k = 1.0 - 0.07 * math.sin((s - se) / 0.016 * math.pi) * smoothstep(0.6, 0.2, sn)
            h, t = h * k, t * k
    y = yc + h * math.copysign(abs(sn) ** 0.92, sn)
    z = zc + (t * c ** 0.8 if c > 0 else 0.32 * t * c)
    return Vector((x, y, z))


def fish_body_shell():
    n_s, n_th = 64, 24
    rings = []
    for i in range(n_s + 1):
        s = (i / n_s) ** 1.15
        rings.append([fish_point(max(s, 0.004), TAU * j / n_th + math.pi / 2) for j in range(n_th)])
    body = loft_rings(rings)
    # the mouth: a cut along the jaw, a little open at the front
    x0 = FISH_X0
    mouth = prism([(x0 - 0.01, -0.0045), (x0 + 0.026, -0.0042), (x0 + 0.028, -0.0034), (x0 - 0.01, -0.0005)], 0.08)
    body = cut(body, mouth.move(0, 0, FISH_FRONT))
    # the nostril and the gill opening's slit read in the paint
    return body


def _fin(outline, s_at, z_off=0.0, thick=0.0014):
    """A fin: its outline (x, y), its corners rounded, extruded thin, in
    the plane of the body's middle at s_at (or z_off off it)."""
    f = prism(round_outline(outline, 0.0025, seg=2), thick, bevel_r=0.0004, bevel_seg=1)
    _, _, _, _, zc = fish_station(s_at)
    return f.move(0, 0, zc + z_off)


def fish_fins_vertical():
    out = Shape()
    # dorsal: its base 40-56 % along the back, the leading rays longest
    xa, ya, _, _, _ = fish_station(0.40)
    xb, yb, _, _, _ = fish_station(0.56)
    top_a = fish_point(0.40, math.pi / 2).y
    top_b = fish_point(0.56, math.pi / 2).y
    out = out + _fin([(xa - 0.002, top_a - 0.006), (xa + 0.006, top_a + 0.020), (xa + 0.013, top_a + 0.033),
                      (xa + 0.020, top_a + 0.031), (xb + 0.004, top_b + 0.018), (xb + 0.006, top_b + 0.010),
                      (xb + 0.002, top_b - 0.006)], 0.48)
    # adipose: a small fleshy flap
    xa = fish_station(0.79)[0]
    t = fish_point(0.79, math.pi / 2).y
    out = out + _fin([(xa - 0.004, t - 0.004), (xa + 0.002, t + 0.005), (xa + 0.009, t + 0.008), (xa + 0.013, t + 0.004),
                      (xa + 0.010, t - 0.004)], 0.79, thick=0.0024)
    # anal: under the tail's front
    xa = fish_station(0.67)[0]
    xb = fish_station(0.79)[0]
    ba, bb = fish_point(0.67, -math.pi / 2).y, fish_point(0.79, -math.pi / 2).y
    out = out + _fin([(xa - 0.002, ba + 0.005), (xa + 0.004, ba - 0.016), (xa + 0.010, ba - 0.022),
                      (xb + 0.004, bb - 0.012), (xb + 0.004, bb + 0.005)], 0.73)
    # the tail: from its root spread to a shallow fork
    x1 = FISH_X0 + FISH_BODY
    xe = FISH_X0 + FISH_L
    ht = fish_station(1.0)[2]
    out = out + _fin([(x1 - 0.012, ht * 0.9), (x1 + 0.010, ht + 0.006), (xe - 0.006, 0.031), (xe, 0.031),
                      (xe - 0.003, 0.020), (xe - 0.007, 0.009), (xe - 0.008, 0.0), (xe - 0.007, -0.009),
                      (xe - 0.003, -0.020), (xe, -0.031), (xe - 0.006, -0.031), (x1 + 0.010, -ht - 0.006),
                      (x1 - 0.012, -ht * 0.9)], 0.99)
    # the fins' rays: a fine ripple across each
    return bump_along_normals(weld(out, 1e-6), lambda p: 0.00025 * math.sin(p.x * 900 + p.y * 300))


def fish_body():
    return fish_body_shell() + fish_fins_vertical()


def _fan_fin(length, a0, a1, root, pleats, amp, cup, thick, nr=5, na=12):
    """A paired fin as a trout's is: rays fanned from a short root (in its
    plane, x out along the fin, y across it), the leading ray (a1) the
    longest and stoutest, the membrane between pleated along the rays and
    thinning to a frayed trailing edge, the whole a little cupped. (The
    first were flat paddles, read as cards: M8.)"""
    s = slab(1.0, 1.0, thick, nr, na)

    def w(p):
        u, v = p.x + 0.5, p.y + 0.5
        a = a0 + (a1 - a0) * v
        # the rays shorten from the leading one back, the edge a soft curve
        L = length * (0.62 + 0.38 * v ** 1.6) * (1.0 + 0.04 * math.sin(v * 23.0))
        r = 0.0015 + u * L
        x, y = r * math.cos(a), r * math.sin(a) + (v - 0.5) * root * (1.0 - u)
        # thin toward the edge, stout along the leading ray
        z = p.z * (1.0 - 0.55 * u) * (1.0 + 0.3 * max(0.0, v - 0.85) / 0.15)
        z += amp * math.sin(v * pleats * math.pi) * (0.3 + 0.7 * u)
        z += cup * u * u
        return Vector((x, y, z))
    return s.warp(w)


def fish_fins_paired():
    """The pectoral (behind the gill cover, low) and the pelvic (under the
    middle), spread out from the body toward the room."""
    out = Shape()
    # pectoral: a fan 34 mm long, swept back and out
    pec = _fan_fin(0.034, math.radians(-30), math.radians(14), 0.006, 9, 0.0008, 0.0022, 0.0011)
    pec = pec.rot(0, math.radians(-12), math.radians(-14))
    p = fish_point(0.235, -math.pi / 2 + 1.05)
    out = out + pec.move(p.x, p.y, p.z - 0.0035)
    # pelvic: smaller, under the dorsal's middle
    pel = _fan_fin(0.024, math.radians(-26), math.radians(12), 0.004, 7, 0.0006, 0.0016, 0.0010)
    pel = pel.rot(math.radians(25), math.radians(-25), math.radians(-12))
    p = fish_point(0.52, -math.pi / 2 + 0.55)
    out = out + pel.move(p.x, p.y, p.z - 0.001)
    return out


def fish_eye():
    """A glass fish eye, 12 mm, set into the head, a little proud of it."""
    s = 0.105
    p = fish_point(s, math.radians(68))
    return sphere(0.0062, seg=16, rings=10).move(p.x, p.y, p.z - 0.0034)


def fish_plaque():
    """The oval board: its back flat, its ogee edge routed round the front."""
    a, b = 0.175, 0.0775
    steps = [(0.0, FISH_BACK), (0.0, FISH_FRONT - 0.0075), (0.0010, FISH_FRONT - 0.0055), (0.0032, FISH_FRONT - 0.0040),
             (0.0050, FISH_FRONT - 0.0030), (0.0060, FISH_FRONT - 0.0015), (0.0080, FISH_FRONT - 0.0003),
             (0.0100, FISH_FRONT)]
    rings = []
    n = 72
    for d, z in steps:
        rings.append([((a - d) * math.cos(TAU * j / n), (b - d) * math.sin(TAU * j / n), z) for j in range(n)])
    return loft_rings(rings)


auto('Real__trophyFishBody', fish_body, 4000, smooth=50.0, kind='plane')
auto('Real__trophyFishFins', fish_fins_paired, 640, smooth=50.0, kind='plane')
auto('Real__trophyFishEye', fish_eye, 400, smooth=80.0, kind='sphere')
auto('Real__trophyFishPlaque', fish_plaque, 1200, smooth=40.0, kind='plane')



# ================================================================ head ===
# A roe buck's head mount on an oak shield (a Kopfpraeparat): the hide
# tanned and set over a carved form, the neck cut short against the shield,
# the head turned a little down as if looking ahead, the ears up and out,
# glass eyes, the black nose; the antlers (above) on its pedicles. Roe head
# mounts run 43-46 cm high, 22-23 cm wide and 30-32 cm from the wall (two
# sold, t11.notes.md). Frame: the shield's back on the wall at z =
# HEAD_BACK, the game hangs the origin 2.00 m up.
HEAD_BACK = -0.004
HEAD_SHIELD = (0.190, 0.260, 0.024)
HEAD_SHIELD_Y = -0.075
HEAD_TURN = math.radians(25.0)               # the nose down off level
HEAD_O = Vector((0.0, 0.035, 0.150))         # the skull's H origin (the cranium's middle)
HEAD_ANT_H = Vector((0.0, 0.046, -0.009))    # the burrs' bases in H (the pedicles' tops)
HEAD_ANTLERS = HEAD_O + rot_x(HEAD_ANT_H, HEAD_TURN)
HEAD_CUT = HEAD_BACK + HEAD_SHIELD[2] + 0.002   # the neck's cut, into the shield's field


def head_shield():
    w, h, t = HEAD_SHIELD
    return shield(w, h, t).move(0, HEAD_SHIELD_Y, HEAD_BACK)


def _h(p):
    """A point of H in the head's frame."""
    return HEAD_O + rot_x(Vector(p), HEAD_TURN)


def _to_h(p):
    return rot_x(Vector(p) - HEAD_O, -HEAD_TURN)


def head_form():
    """The hide over its form: the neck from the shield, the throat, the
    jaw, the cheeks, the cranium, the long face and the muzzle, the orbits,
    the pedicles; the nostrils and the mouth's line cut."""
    H = lambda kind, c, *a: (kind, tuple(_h(c)), *a)
    e = [
        # the neck: from the shield up and out to the head
        ('ell', (0, -0.095, 0.020), (0.056, 0.068, 0.030), 2.0),
        ('cap', (0, -0.092, 0.016), (0, -0.010, 0.112), 0.049, 2.0),
        ('ell', (0, -0.050, 0.105), (0.034, 0.040, 0.042), 2.0),
        ('ell', (0, -0.025, 0.060), (0.046, 0.060, 0.046), 2.0),
    ]
    heh = [
        ('ell', (0, 0, 0), (0.043, 0.039, 0.048)),
        ('ell', (0, -0.004, 0.058), (0.033, 0.033, 0.054)),
        ('ell', (0, -0.014, 0.108), (0.0235, 0.027, 0.034)),
        ('ball', (0, -0.015, 0.138), 0.0195),
        ('ell', (0, -0.039, 0.072), (0.021, 0.016, 0.054)),
        ('ball', (0, -0.035, 0.124), 0.0110),
        ('ell', (0, 0.020, 0.026), (0.029, 0.018, 0.034)),
    ]
    for sx in (-1, 1):
        heh += [('ell', (sx * 0.011, -0.026, 0.124), (0.014, 0.0145, 0.021)),
                ('ell', (sx * 0.024, -0.023, 0.030), (0.021, 0.029, 0.034)),
                ('ball', (sx * 0.033, 0.010, 0.042), 0.0145)]
    for k in heh:
        if k[0] == 'ell':
            e.append(('ell', tuple(_h(k[1])), k[2], 2.0, (HEAD_TURN, 0, 0)))
        else:
            e.append((k[0], tuple(_h(k[1])), k[2], 2.0))
    # the pedicles under the hide, up to the burrs
    ay = Vector((0, math.cos(ROE_BETA), -math.sin(ROE_BETA)))
    for sx in (-1, 1):
        top = HEAD_ANT_H + Vector((sx * ROE_PED_X, 0, 0)) - ay * 0.002
        e.append(('cap', tuple(_h(top - ay * 0.018)), tuple(_h(top)), 0.0088, 2.0))
    # the nostrils and the eyes' seats
    for sx in (-1, 1):
        e.append(('-ball', tuple(_h((sx * 0.0078, -0.010, 0.1565))), 0.0042, 2.0))
        e.append(('-ball', tuple(_h((sx * 0.0395, 0.010, 0.047))), 0.0090, 2.0))
    s = meta(e, res=0.0022)
    s = remesh(s, 0.0018)
    # the mouth's line, from its corners to the front
    for sx in (-1, 1):
        a, b = _h((sx * 0.019, -0.031, 0.096)), _h((sx * 0.006, -0.031, 0.147))
        d = (b - a)
        L = d.length
        groove = box(0.0016, 0.004, L + 0.004).rot(0, math.atan2(d.x, d.z), 0)
        s = cut(s, groove.move(*((a + b) / 2)))
    # the neck cut flat into the shield
    s = cut(s, box(0.4, 0.5, 0.2).move(0, 0, HEAD_CUT - 0.1))
    # hair lies in tufts: a soft noise over the hide
    s = bump_along_normals(s, lambda p: 0.0004 * noise.noise(Vector((p.x * 260, p.y * 260, p.z * 90))))
    return decimate(s, 5600)


def _leaf_solid(fn, nu, nv, thick):
    """A thin solid from a surface fn(u, v) -> (point, normal), u in -1..1,
    v in 0..1: the front sheet, the back sheet `thick` behind it, the rim."""
    front, back = [], []
    for j in range(nv + 1):
        for i in range(nu + 1):
            p, n = fn(-1 + 2 * i / nu, j / nv)
            front.append(p)
            back.append(p - n * thick)
    k = nu + 1
    v = front + back
    nf = len(front)
    f = []
    for j in range(nv):
        for i in range(nu):
            a = j * k + i
            f.append((a, a + 1, a + k + 1, a + k))
            f.append((nf + a, nf + a + k, nf + a + k + 1, nf + a + 1))
    rim = [i for i in range(nu + 1)] + [j * k + nu for j in range(1, nv + 1)] + \
          [nv * k + i for i in range(nu - 1, -1, -1)] + [j * k for j in range(nv - 1, 0, -1)]
    for a, b in zip(rim, rim[1:] + rim[:1]):
        f.append((b, a, nf + a, nf + b))
    return propkit._fix_winding(Shape(v, f))


def _ear_frame(sx):
    base = Vector((sx * 0.033, 0.024, -0.028))
    e = Vector((sx * 0.72, 0.62, -0.30)).normalized()
    o = Vector((sx * 0.55, 0.05, 1.0))
    o = (o - e * o.dot(e)).normalized()
    x = e.cross(o).normalized()
    return base, x, e, o


def _ear_surface(sx, shrink=1.0, lift=0.0):
    base, X, Y, Z = _ear_frame(sx)

    def fn(u, v):
        v = 0.03 + 0.97 * v * shrink
        w = 0.0285 * math.sin(math.pi * min(v, 0.999)) ** 0.55 * (1.0 - 0.15 * v) * shrink
        cup = 0.011 * (u * u) * (0.35 + 0.65 * math.sin(math.pi * v)) - 0.010 * v * v
        p_l = X * (u * w) + Y * (v * 0.106) + Z * (cup + lift)
        n = (Z - X * (2 * 0.011 * u * w / 0.03)).normalized()
        return _h(base + p_l), rot_x(n, HEAD_TURN)
    return fn


def head_ears():
    out = Shape()
    for sx in (-1, 1):
        out = out + _leaf_solid(_ear_surface(sx), 8, 12, 0.0026)
    return out


def head_ears_inside():
    """The pale hair inside the ears: a sheet over each ear's hollow, short
    of its rim."""
    out = Shape()
    for sx in (-1, 1):
        fn = _ear_surface(sx, shrink=0.86, lift=0.0005)
        nu, nv = 6, 10
        v, f = [], []
        for j in range(nv + 1):
            for i in range(nu + 1):
                v.append(fn(-1 + 2 * i / nu, j / nv)[0])
        k = nu + 1
        for j in range(nv):
            for i in range(nu):
                a = j * k + i
                f.append((a, a + 1, a + k + 1, a + k) if sx > 0 else (a, a + k, a + k + 1, a + 1))
        out = out + Shape(v, f)
    return out


def _head_regions():
    return cached('head', head_form)


def _pale(c, n):
    """Where the roe's coat is pale: the chin and the lower lip, the
    throat's patch."""
    h = _to_h(c)
    if h.y < -0.034 and h.z > 0.090:
        return True
    # the throat's patch: an oval under the jaw, on the neck's front
    if c.z > 0.075 and n.y < 0.2 and (c.x / 0.027) ** 2 + ((c.y + 0.050) / 0.017) ** 2 < 1.0:
        return True
    return False


def _nose(c, n):
    """The bare black nose (the rhinarium): the muzzle's front over the lip,
    the lip under it haired (the first covered the muzzle's whole front
    down to the chin, which straight on read as a hole: M8)"""
    h = _to_h(c)
    return h.z > 0.1465 and h.y > -0.0215


def head_hide():
    return split_faces(_head_regions(), lambda c, n: not _pale(c, n) and not _nose(c, n)) + head_ears()


def head_pale():
    return split_faces(_head_regions(), lambda c, n: _pale(c, n) and not _nose(c, n)) + head_ears_inside()


def head_nose():
    return split_faces(_head_regions(), lambda c, n: _nose(c, n))


def head_eyes():
    out = Shape()
    for sx in (-1, 1):
        out = out + sphere(0.0105, seg=16, rings=10).scale(0.85, 1.0, 1.0).move(*_h((sx * 0.0372, 0.010, 0.047)))
    return out


auto('Real__roeHeadShield', head_shield, 900, smooth=40.0)
auto('Real__roeHeadHide', head_hide, 7000, smooth=60.0)
auto('Real__roeHeadPale', head_pale, 1600, smooth=60.0)
auto('Real__roeHeadNose', head_nose, 500, smooth=60.0)
auto('Real__roeHeadEyes', head_eyes, 700, smooth=80.0, kind='sphere')


# ================================================================ vase ===
# A glazed earthenware floor vase of the late Victorian kind, 45 cm high: a
# baluster on a turned foot ring, a narrow neck and a trumpet mouth, the
# glaze one colour (majolica or art pottery), the foot ring left bare;
# peacock eye feathers stood in it (the game scales the whole vase by its
# old height, 40-50 cm). Frame: standing on the floor, its axis y.
VASE_H = 0.450
VASE_PROFILE = [(0.058, 0.000), (0.0615, 0.003), (0.0612, 0.012), (0.0640, 0.016), (0.0700, 0.022),
                (0.0820, 0.045), (0.0950, 0.090), (0.1035, 0.150), (0.1060, 0.200), (0.1020, 0.248),
                (0.0900, 0.295), (0.0720, 0.333), (0.0540, 0.358), (0.0440, 0.378), (0.0410, 0.398),
                (0.0425, 0.415), (0.0480, 0.432), (0.0560, 0.445), (0.0590, VASE_H - 0.0021)]


def peacock_vase():
    s = vessel(VASE_PROFILE, 0.0065, seg=40, floor=0.012)
    # a turned groove above the foot and a bead at the neck's root
    return s


auto('Real__peacockVase', peacock_vase, 3000, smooth=40.0, kind='cyl')


# A peacock's train feather, an eye feather of the kind sold for vases
# (76-89 cm with its quill): its bare quill stood in the vase; the herl, long
# bronze-green barbs, sparse along its shaft; the eye at its tip, 11 cm
# long. Only what stands out of the vase is modelled: from 5 cm down in
# the neck (the origin) up its 0.47 m, the vane a ribbon the game paints
# (its alpha the barbs, its tip the eye), dipping as a feather does, its
# face to +z. The shaft is the rachis along the ribbon's middle.
FEATHER_L = 0.470
FEATHER_DROOP = 0.065


def _feather_path(t):
    """The rachis at t (0 its foot, 1 its tip): its point, its tangent."""
    y = FEATHER_L * t
    z = FEATHER_DROOP * t ** 2.4
    dz = FEATHER_DROOP * 2.4 * t ** 1.4 / FEATHER_L
    tng = Vector((0, 1, dz)).normalized()
    return Vector((0, y - 0.25 * z * t, z)), tng


def _feather_half_width(t):
    # bare quill, the herl widening out, the eye's round at the tip
    if t < 0.05:
        return 0.004
    herl = 0.006 + 0.075 * smoothstep(0.05, 0.45, t)
    return herl


def peacock_feather():
    nu, nv = 6, 40
    v, f = [], []
    for j in range(nv + 1):
        t = j / nv
        c, tng = _feather_path(t)
        side = Vector((1, 0, 0))
        nrm = side.cross(tng).normalized()
        w = _feather_half_width(t)
        for i in range(nu + 1):
            u = -1 + 2 * i / nu
            # the barbs sweep back and droop to the sides
            p = c + side * (u * w) - nrm * (0.18 * abs(u) * w) - tng * (0.25 * abs(u) * w)
            v.append(p)
    k = nu + 1
    for j in range(nv):
        for i in range(nu):
            a = j * k + i
            f.append((a, a + 1, a + k + 1, a + k))
    return Shape(v, f)


def peacock_feather_shaft():
    pts = []
    rr = []
    for j in range(25):
        t = j / 24
        c, _ = _feather_path(t)
        pts.append(c - Vector((0, 0, 0.0006)))
        rr.append(lerp(0.0019, 0.0006, t))
    return _sweep(pts, rr, seg=6, sub=1)


auto('Real__peacockFeather', peacock_feather, 600, smooth=60.0, kind='plane')
auto('Real__peacockFeatherShaft', peacock_feather_shaft, 400, smooth=60.0)


# ================================================================= fan ===
# A lady's paper folding fan (a sensu), 19.5 cm: 25 bamboo ribs riveted at
# their heads, the two guards (oyabone) wider and lacquered, the leaf of two
# sheets of washi folded in pleats over the ribs' upper 11 cm; dropped open
# on the floor (110 degrees), lying on its lower guard. Frame: the rivet at
# x = z = 0, the fan opened toward +z, lying on y = 0.
FAN_L = 0.195                 # rivet to the leaf's top edge (the rib's length past the rivet)
FAN_R0 = 0.085                # where the leaf starts
FAN_OPEN = math.radians(110.0)
FAN_RIBS = 25


def _fan_angle(i, n):
    return -FAN_OPEN / 2 + FAN_OPEN * i / (n - 1)


def fan_leaf():
    """The pleated leaf: 2 (ribs - 1) panels, folded up and down in turn,
    the folds deepening outward; two sheets thick."""
    n = 2 * (FAN_RIBS - 1)
    radii = [FAN_R0 + (FAN_L - FAN_R0) * k / 5 for k in range(6)]
    top, bot = [], []
    for i in range(n + 1):
        a = -FAN_OPEN / 2 + FAN_OPEN * i / n
        up = 1.0 if i % 2 == 0 else -1.0     # ribs under the even folds
        for r in radii:
            depth = 0.0016 * (r / FAN_L)
            y = 0.0042 + up * depth + 0.0015 * (1 - r / FAN_L)
            p = Vector((r * math.sin(a), y, r * math.cos(a)))
            top.append(p + Vector((0, 0.00025, 0)))
            bot.append(p - Vector((0, 0.00025, 0)))
    k = len(radii)
    v = top + bot
    nt = len(top)
    f = []
    for i in range(n):
        for j in range(k - 1):
            a = i * k + j
            f.append((a, a + 1, a + k + 1, a + k))
            f.append((nt + a, nt + a + k, nt + a + k + 1, nt + a + 1))
    # the rim
    for i in range(n):
        for j in (0, k - 1):
            a, b = i * k + j, (i + 1) * k + j
            f.append((a, b, nt + b, nt + a) if j == 0 else (b, a, nt + a, nt + b))
    for i in (0, n):
        for j in range(k - 1):
            a, b = i * k + j, i * k + j + 1
            f.append((a, b, nt + b, nt + a) if i == n else (b, a, nt + a, nt + b))
    s = propkit._fix_winding(Shape(v, f))
    # the paper's wear: a little cockle
    return bump_along_normals(s, lambda p: 0.0003 * noise.noise(Vector((p.x * 120, p.y * 120, p.z * 120))))


def fan_ribs():
    """The ribs' bare sticks from their heads past the rivet to the leaf
    (thin inside it), stacked at the rivet; the two guards full length."""
    out = Shape()
    for i in range(FAN_RIBS):
        a = _fan_angle(i, FAN_RIBS)
        guard = i in (0, FAN_RIBS - 1)
        y0 = 0.0006 + 0.00065 * i if not guard else (0.0010 if i == 0 else 0.0006 + 0.00065 * FAN_RIBS + 0.0010)
        L = (FAN_L if guard else FAN_R0 + 0.006) + 0.012
        w0, w1 = (0.0065, 0.0050) if guard else (0.0042, 0.0030)
        th = 0.0020 if guard else 0.0008
        # a rib: a slat from its round head (behind the rivet) out along a
        pts = [(-w0 / 2, -0.012), (w0 / 2, -0.012), (w1 / 2, L - 0.012), (-w1 / 2, L - 0.012)]
        head = propkit._round_corner(pts[0], pts[-1], pts[1], 0.0016, 2) + propkit._round_corner(pts[1], pts[0], pts[2], 0.0016, 2)
        rib = prism(head + pts[2:], th, bevel_r=0.0003 if guard else 0.0, bevel_seg=1)
        rib = rib.rot(math.pi / 2, 0, 0)           # lay it flat: y up through it, its length along +z
        # the slat bends down from the stack to the leaf (inner) or the floor (guard)
        y_end = 0.0042 if not guard else (0.0010 if i == 0 else 0.0068)

        def drop(p, y0=y0, y_end=y_end, L=L):
            t = max(0.0, min(1.0, (p.z + 0.012) / (FAN_R0 + 0.012)))
            return Vector((p.x, p.y + lerp(y0, y_end, t * t * (3 - 2 * t)), p.z))
        rib = rib.warp(drop).rot(0, a, 0)
        out = out + rib
    return out


def fan_rivet():
    """The rivet (kaname): a brass pin through the ribs' heads, its two
    domed washers."""
    h = 0.0006 + 0.00065 * FAN_RIBS + 0.0030
    pin = cyl(0.0012, h=h, seg=10, bevel_r=0.0002).move(0, h / 2, 0)
    cap = lathe([(0, 0), (0.0026, 0), (0.0026, 0.0004), (0.0018, 0.0010), (0, 0.0013)], seg=14)
    return pin + cap.copy().move(0, h, 0) + cap.copy().rot(math.pi, 0, 0).move(0, 0.0013, 0)


auto('Real__fanLeaf', fan_leaf, 1400, smooth=20.0)
auto('Real__fanRibs', fan_ribs, 900, smooth=30.0)
auto('Real__fanRivet', fan_rivet, 300, smooth=40.0)
