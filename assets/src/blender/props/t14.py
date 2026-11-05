"""t14: the props remodelled at their real sizes (docs/PROPS.md, P3, the
rare): a play area's soft toys and its dolls, as found: a jointed mohair
teddy bear, a plush teddy slumped where it was left, a plush teddy's arm
torn off at its seam, a bean-bag elephant, a small hard-plastic doll, a
vinyl baby doll's head and its arms and legs come off at their joints.
Sources and sizes: t14.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame (x across its front, y up from the floor, z out of
its front) unless its section says otherwise.

The doll's head, its sleep eyes and their lids are batch r03's (the baby
doll): its sculpt is read from r03.py (the code before its parts), never
built twice; the small doll is a smaller doll of the same mould.
"""
from propkit import *
from propkit import _mesh_obj, _eval_to_shape, _drop
import propkit
import os
import random
import zlib
from mathutils.bvhtree import BVHTree


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, kind='box', smooth=35.0):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth)


def rnd(a, b):
    return a + (b - a) * random.random()


# ------------------------------------------------------------- helpers ---
_meta_n = [0]
MB_K = 1.0 / 0.564


def meta(elems, res=0.002, threshold=0.6):
    """A blobby surface from metaball elements (r03's): ('ball', centre,
    radius[, stiffness]), ('ell', centre, (rx, ry, rz)[, stiffness[,
    euler]]), ('cap', a, b, radius[, stiffness]); '-ball' / '-ell'
    subtract. Sizes are the element's own surface. Blender holds a
    metaball's resolution at 5 mm or more, so the elements are built
    larger by as much and the surface brought back down: `res` is kept."""
    _meta_n[0] += 1
    name = f'_t14mb{_meta_n[0]}x'           # a family of its own (no '.')
    S = max(1.0, 0.005 / res)
    mb = bpy.data.metaballs.new(name)
    mb.resolution = res * S
    mb.render_resolution = res * S
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
            el.co = Vector(e[1]) * S
            el.radius = e[2] * MB_K * S
            el.stiffness = e[3] if len(e) > 3 else 2.0
        elif kind == 'ell':
            el.type = 'ELLIPSOID'
            el.co = Vector(e[1]) * S
            el.radius = MB_K * S
            el.size_x, el.size_y, el.size_z = e[2]
            el.stiffness = e[3] if len(e) > 3 else 2.0
            if len(e) > 4:
                el.rotation = Euler(e[4], 'XYZ').to_quaternion()
        elif kind == 'cap':
            a, b = Vector(e[1]) * S, Vector(e[2]) * S
            el.type = 'CAPSULE'
            el.co = (a + b) / 2
            el.radius = e[3] * MB_K * S
            el.size_x = max(1e-5, (b - a).length / 2)
            el.stiffness = e[4] if len(e) > 4 else 2.0
            el.rotation = Vector((1, 0, 0)).rotation_difference((b - a).normalized())
        else:
            raise ValueError(kind)
    s = _eval_to_shape(ob)
    bpy.data.objects.remove(ob)
    bpy.data.metaballs.remove(mb)
    s.scale(1.0 / S)
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
    ob = _mesh_obj('_t14dec', s)
    m = ob.modifiers.new('d', 'DECIMATE')
    m.decimate_type = 'COLLAPSE'
    m.ratio = tris / n
    m.use_collapse_triangulate = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def smooth_shape(s, factor=0.5, iterations=4):
    ob = _mesh_obj('_t14sm', s)
    m = ob.modifiers.new('s', 'SMOOTH')
    m.factor = factor
    m.iterations = iterations
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def bvh_of(s):
    return BVHTree.FromPolygons([p.copy() for p in s.v], [list(f) for f in s.f])


def project_patch(bvh, centre, normal, up, w, h, off, seg=12, rings=3, shape=None):
    """A thin patch (an oval w x h in the tangent plane at `centre`, `up`
    its long axis) laid onto a surface, every vertex cast back onto it along
    -normal and lifted `off` off it (r03's). Facing out of the surface."""
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


def on_surface(bvh, origin, direction, sink=0.0):
    """Where a ray from inside meets the surface, and the surface's normal
    there; `sink` moves the point back in along the normal."""
    d = Vector(direction).normalized()
    hit = bvh.ray_cast(Vector(origin), d, 1.0)
    if hit[0] is None:
        raise ValueError(f'no surface from {origin} along {direction}')
    n = hit[1].normalized()
    return hit[0] - n * sink, n


def surface_path(bvh, pts, off, axis_from):
    """A polyline laid on a surface: each point cast from `axis_from`
    (a point inside) outward through it, lifted `off`."""
    out = []
    for p in pts:
        p = Vector(p)
        d = (p - Vector(axis_from)).normalized()
        hit = bvh.ray_cast(Vector(axis_from), d, 1.0)
        if hit[0] is None:
            out.append(p)
        else:
            out.append(hit[0] + hit[1].normalized() * off)
    return out


def flatten_below(s, y0, k=0.12):
    """Squash everything under y0 toward it (where it rests on the floor)."""
    def fn(p):
        if p.y < y0:
            p.y = y0 - (y0 - p.y) * k
        return p
    return s.warp(fn)


def to_floor(s):
    lo, _ = s.bounds()
    return s.move(0, -lo.y, 0)


def groove(s, dist_fn, depth, width, centre_fn):
    """A sewn seam: vertices near a line (dist_fn(p) its distance) drawn in
    toward the inside (centre_fn(p) a point inside) by `depth`, falling off
    over `width`."""
    out = s.copy()
    for i, p in enumerate(out.v):
        d = dist_fn(p)
        if d < width:
            k = (1 - d / width) ** 2
            c = Vector(centre_fn(p))
            out.v[i] = p + (c - p).normalized() * depth * k
    return out


def lumpy(s, amount, scale, seed):
    """Uneven stuffing: a soft push in and out along the surface."""
    off = Vector((seed * 17.3, seed * 5.1, seed * 9.7))
    out = s.copy()
    c = sum(out.v, Vector((0, 0, 0))) / max(1, len(out.v))
    for i, p in enumerate(out.v):
        q = p * scale + off
        d = (p - c)
        d = d.normalized() if d.length > 1e-9 else Vector((0, 1, 0))
        out.v[i] = p + d * noise.noise(q) * amount
    return out


def mirror_x(s):
    return s.copy().scale(-1, 1, 1)


def fibres(rng, centre, radius, n, length, r=0.00035, up=0.5):
    """Loose fibres or threads: short curled strands about `centre`."""
    out = Shape()
    for _ in range(n):
        a, e = rng.uniform(0, TAU), rng.uniform(-0.2, up)
        d = Vector((math.cos(a) * math.cos(e), math.sin(e), math.sin(a) * math.cos(e)))
        p0 = Vector(centre) + d * radius * rng.uniform(0.6, 1.0)
        ln = length * rng.uniform(0.5, 1.0)
        bendv = Vector((rng.uniform(-1, 1), rng.uniform(-0.3, 0.8), rng.uniform(-1, 1))).normalized()
        pts = [p0 + d * ln * t + bendv * ln * 0.35 * t * t for t in (0, 0.33, 0.66, 1.0)]
        out = out + tube(pts, r, seg=3, caps=False)
    return out


# ================================================ r03's doll, read again ===
# The baby doll's sculpt (its head, sleep eyes, lids and lashes, its limbs'
# joints) is r03's own code, read from r03.py up to its parts (nothing of it
# is registered or built here but what this batch asks for).
def _load_r03():
    path = os.path.join(os.path.dirname(os.path.abspath(propkit.__file__)), 'props', 'r03.py')
    src = open(path).read()
    src = src[:src.index('# ---------------------------------------------------------------- parts ---')]
    ns = {'__name__': '_t14_r03', '__file__': path}
    exec(compile(src, path, 'exec'), ns)
    ns['meta'] = meta              # metaballs of this batch's own family
    return ns


R03 = _load_r03()


def r03_lashes(state):
    """r03's lash strips for lids in `state` (each eye's opening, 0 shut to
    1 open): lying down on the cheek from a shut lid, standing out from an
    open one."""
    out = Shape()
    EYE_R, EYE_W = R03['EYE_R'], R03['EYE_W']
    r = EYE_R + 0.0006
    for i in range(2):
        c = R03['eye_centre'](i)
        n = 15
        v, f = [], []
        for k in range(n):
            x = -EYE_W * 0.86 + 2 * EYE_W * 0.86 * k / (n - 1)
            half = 0.00055
            b0 = R03['_on_ball'](c, x - half, R03['_lid_line'](i, x - half), r)
            b1 = R03['_on_ball'](c, x + half, R03['_lid_line'](i, x + half), r)
            base = (b0 + b1) / 2
            nrm = (base - c).normalized()
            o = state[i]
            d = nrm * (0.8 + 0.2 * o) + Vector((0, -0.6 + 0.85 * o, 0))
            d = (d + Vector((x / EYE_W * 0.35, 0, 0))).normalized()
            ln = 0.0046 * (0.7 + 0.3 * math.cos(x / EYE_W * 1.3))
            v += [b0, b1, base + d * ln]
            f.append((3 * k, 3 * k + 1, 3 * k + 2))
        out = out + Shape(v, f)
    return out


def r03_lids(state):
    saved = R03['LID_OPEN']
    R03['LID_OPEN'] = state
    try:
        return R03['doll_lids']()
    finally:
        R03['LID_OPEN'] = saved


# ====================================================== the doll's head ===
# The baby's head come off its body (r03's Real__dollHead, Face, Eye*, Lids,
# Lashes and Hair, placed by the game in r03's head frame): what a head
# alone shows besides is its neck, open: the vinyl rolled in at its rim (it
# held the head on the body's neck) and the dark of the hollow head. Its
# frame: r03's head frame (origin under the neck, y up the head, z out of
# the face). A head standing on its neck has its eyes open: the right open,
# the left stuck half (the lids and lashes `Up`).
NECK_Y0, NECK_Z, NECK_R = R03['NECK_Y0'], R03['NECK_Z'], R03['NECK_R']


def head_neck_lip():
    """The neck's end: the vinyl flaring a little to a rim and rolled in
    there (it held the head on the body's neck), 2.2 mm round, proud of
    the stub's end (the head stands on it); a collar over r03's stub."""
    top = NECK_Y0 + 0.017
    prof = [(NECK_R - 0.0040, NECK_Y0 - 0.0002), (NECK_R - 0.0022, NECK_Y0 - 0.0019),
            (NECK_R + 0.0006, NECK_Y0 - 0.0016), (NECK_R + 0.0021, NECK_Y0 + 0.0004),
            (NECK_R + 0.0018, NECK_Y0 + 0.0030), (NECK_R + 0.0009, NECK_Y0 + 0.0085),
            (NECK_R + 0.0002, top - 0.002), (NECK_R - 0.0010, top)]
    s = lathe(prof, seg=30, cap_bottom=False, cap_top=False)
    return s.move(0, 0, NECK_Z)


def head_neck_hole():
    """The dark inside the hollow head, seen through the neck."""
    d = disc(NECK_R - 0.0042, seg=28).rot(math.pi / 2, 0, 0)       # facing down
    return d.move(0, NECK_Y0 - 0.0006, NECK_Z)


# ====================================================== the doll's limbs ===
# A baby's arm and leg (r03's doll: soft vinyl, hollow) come off its cloth
# body: the top of each a neck the cloth was tied round and a lip at its
# open end (the cloth body's sleeve was gathered and tied in the groove),
# the inside dark. Each in its own frame, lying on the floor: the open end
# at the origin facing -x, the limb along +x.
def _limb_top(r_neck, r_lip, length):
    """The groove and the lip at a limb's open end: a lathe about +x from
    the end face (x = 0) to `length`, the hole's wall 1.6 mm thick turned
    in 4 mm (the rest of the hollow is dark: the hole part)."""
    ri = r_neck - 0.0016
    prof = [(ri, 0.004), (ri + 0.0002, 0.0002), (r_lip - 0.0007, 0.0), (r_lip, 0.0008),
            (r_lip - 0.0004, 0.0022), (r_neck, 0.0032), (r_neck - 0.0004, 0.0052),
            (r_neck, length * 0.7), (r_neck + 0.0016, length)]
    s = lathe(prof, seg=18, cap_bottom=False, cap_top=False)
    return s.rot(0, 0, -math.pi / 2)            # its axis from +y to +x


def _limb_hole(r_neck):
    d = disc(r_neck - 0.0016, seg=20).rot(0, -math.pi / 2, 0)    # facing -x
    return d.move(0.004, 0, 0)


ARM_F = Vector((0.0, 0.0, 0.0))
ARM_E = Vector((0.046, 0.0, 0.006))
ARM_W = Vector((0.062, 0.0, 0.033))


def limb_arm_raw():
    E, W = ARM_E, ARM_W
    pn = Vector((0.15, 0.9, 0.3))               # the palm's normal: up, a little in
    el = [('cap', (0.010, 0, 0), tuple(E), 0.0134, 2.0), ('cap', tuple(E), tuple(W), 0.0128, 2.0),
          ('ball', tuple(E), 0.0150, 1.6), ('ball', tuple(E.lerp(W, 0.45)), 0.0142, 1.5),
          ('ball', (0.026, 0, 0.001), 0.0146, 1.5)]
    d, n, sd = R03['_frame'](W - E, pn)
    palm = W + d * 0.016
    el.append(('ball', tuple(W - d * 0.001), 0.0094, 1.5))        # the wrist, a crease either side
    el.append(R03['_ell_on'](palm, (sd, d, n), (0.0114, 0.0132, 0.0060)))
    for k in range(4):                                            # the fingers, moulded together, curled
        off = (k - 1.5) * 0.0054
        b = palm + d * 0.009 + sd * off
        t = b + d * (0.010 - abs(k - 1.5) * 0.0016) - n * 0.0046
        el.append(('cap', tuple(b), tuple(t), 0.0030, 2.0))
    tb = palm - d * 0.004 + sd * 0.0110
    el.append(('cap', tuple(tb), tuple(tb + d * 0.008 + sd * 0.004 - n * 0.003), 0.0034, 2.0))   # the thumb
    s = meta(el, res=0.0011)
    # what lies inside the top's lathe is cut off: the limb starts at its neck
    s = boolean(s, box(0.02, 0.06, 0.06).move(-0.004, 0, 0), 'DIFFERENCE')
    return s


_limbs = {}


def _limb(kind):
    """The limb as built (cached: its part and its hole's share it), lying
    with its lowest point on the floor: (body, the lift)."""
    if kind not in _limbs:
        if kind == 'arm':
            body = decimate(limb_arm_raw(), 560) + _limb_top(0.0102, 0.0128, 0.012)
        else:
            body = decimate(limb_leg_raw(), 520) + _limb_top(0.0142, 0.0170, 0.014)
        lo, _ = body.bounds()
        _limbs[kind] = (body.move(0, -lo.y, 0), -lo.y)
    return _limbs[kind]


def limb_arm():
    return flatten_below(_limb('arm')[0].copy(), 0.0006, 0.2)


def limb_arm_hole():
    return _limb_hole(0.0102).move(0, _limb('arm')[1], 0)


LEG_K = Vector((0.054, 0.0, 0.008))
LEG_A = Vector((0.094, 0.0, -0.010))


def limb_leg_raw():
    K, A = LEG_K, LEG_A
    td = Vector((0.41, 0.10, 0.91))             # the toes: forward of the bent knee, lying on its outer edge
    el = [('cap', (0.012, 0, 0), tuple(K), 0.0172, 2.0), ('cap', tuple(K), tuple(A), 0.0148, 2.0),
          ('ball', tuple(K.lerp(A, 0.38)), 0.0170, 1.4),                 # the calf
          ('ball', (0.030, 0, 0.003), 0.0200, 1.4),                      # the thigh, fat
          ('ball', tuple(K), 0.0164, 1.5)]                               # the knee
    d, n, sd = R03['_frame'](td, A - K)
    heel = A + n * 0.005 - d * 0.004
    el.append(('ball', tuple(heel), 0.0108, 1.8))
    el.append(R03['_ell_on'](A + d * 0.017 + n * 0.004, (sd, d, n), (0.0120, 0.0205, 0.0088)))
    for k in range(5):                                                   # the toes
        off = (k - 2) * 0.0044
        r = 0.0041 if k == 0 else 0.0033 - k * 0.0002
        el.append(('ball', tuple(A + d * (0.035 - abs(k - 0.5) * 0.0018) + sd * off + n * 0.003), r, 1.6))
    s = meta(el, res=0.0012)
    s = boolean(s, box(0.02, 0.08, 0.08).move(-0.004, 0, 0), 'DIFFERENCE')
    return s


def limb_leg():
    return flatten_below(_limb('leg')[0].copy(), 0.0006, 0.2)


def limb_leg_hole():
    return _limb_hole(0.0142).move(0, _limb('leg')[1], 0)


auto('Real__dollHeadNeckLip', head_neck_lip, 360, smooth=60)
auto('Real__dollHeadNeckHole', head_neck_hole, 100, kind='plane')
auto('Real__dollHeadLidsUp', lambda: r03_lids((0.97, 0.50)), 280, smooth=70)
auto('Real__dollHeadLashesUp', lambda: r03_lashes((0.97, 0.50)), 100)
auto('Real__dollLimbArm', limb_arm, 900, smooth=70)
auto('Real__dollLimbArmHole', limb_arm_hole, 100)
auto('Real__dollLimbLeg', limb_leg, 860, smooth=70)
auto('Real__dollLimbLegHole', limb_leg_hole, 100)
