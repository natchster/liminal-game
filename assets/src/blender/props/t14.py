"""t14: the props remodelled at their real sizes (docs/design/PROPS.md, P3, the
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
from propkit import _mesh_obj, _eval_to_shape, _drop, _fix_winding
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


def shell(s, d, floor=None):
    """The pile's tips: the surface pushed out `d` along its normals (drawn
    stippled by the game, an alpha-tested layer that softens the outline as
    a pile does); kept off the floor."""
    bm = propkit.to_bmesh(s)
    bm.normal_update()
    bm.verts.index_update()
    out = Shape([v.co + v.normal * d for v in bm.verts], [tuple(v.index for v in f.verts) for f in bm.faces])
    bm.free()
    if floor is not None:
        out.warp(lambda p: Vector((p.x, max(p.y, floor), p.z)))
    return out


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


auto('Real__dollHeadNeckLip', head_neck_lip, 440, smooth=60)
auto('Real__dollHeadNeckHole', head_neck_hole, 100, kind='plane')
auto('Real__dollHeadLidsUp', lambda: r03_lids((0.97, 0.50)), 280, smooth=70)
auto('Real__dollHeadLashesUp', lambda: r03_lashes((0.97, 0.50)), 100)
auto('Real__dollLimbArm', limb_arm, 900, smooth=70)
auto('Real__dollLimbArmHole', limb_arm_hole, 100)
auto('Real__dollLimbLeg', limb_leg, 860, smooth=70)
auto('Real__dollLimbLegHole', limb_leg_hole, 100)


# ======================================================= the mohair teddy ===
# A jointed mohair teddy bear of the 1950s, 18 cm standing (Steiff's and
# its imitators': golden or dark brown mohair pile on a woven cotton
# backing, stuffed hard with wood wool; jointed at the neck, shoulders and
# hips by discs inside; glass eyes; the nose and mouth embroidered in black
# floss; felt pads on the paws and feet; long arms curved at the paws, a
# hump on the back, a long clipped muzzle). Sitting as a jointed bear sits,
# its legs out in front. Its frame: x across, y up from the floor, z out of
# its front. The head is built in its own frame (origin at the neck joint)
# and placed by the game at TEDDY_NECK, where it turns on its joint.
TEDDY_NECK = Vector((0.0, 0.0705, -0.006))
TEDDY_HC = Vector((0.0, 0.026, 0.0))          # the head's centre in its frame
TEDDY_ARMS = [(Vector((sx * 0.026, 0.061, -0.001)), Vector((sx * 0.031, 0.041, 0.012)), Vector((sx * 0.027, 0.025, 0.028)))
              for sx in (-1, 1)]
TEDDY_LEGS = [(Vector((sx * 0.016, 0.015, 0.008)), Vector((sx * 0.020, 0.012, 0.050)), Vector((sx * 0.021, 0.019, 0.057)))
              for sx in (-1, 1)]
_teddy = {}


def teddy_torso():
    el = [('ell', (0, 0.030, 0.003), (0.025, 0.030, 0.022)),
          ('ell', (0, 0.055, -0.002), (0.020, 0.020, 0.018), 1.8),
          ('ell', (0, 0.059, -0.012), (0.015, 0.014, 0.011), 1.6),            # the hump
          ('ell', (0, 0.022, 0.010), (0.021, 0.016, 0.015), 1.6)]            # the round tummy
    s = meta(el, res=0.0012)
    s.rot(-0.10, 0, 0)
    s = flatten_below(to_floor(s), 0.005, 0.15)
    # the seams: down the middle of the front and the back
    s = groove(s, lambda p: abs(p.x), 0.0005, 0.0012, lambda p: Vector((0, p.y, 0.0)))
    return to_floor(s)


def teddy_arm(i):
    S, M, P = TEDDY_ARMS[i]
    sx = -1 if i == 0 else 1
    d = (P - M).normalized()
    el = [('ball', tuple(S + Vector((0, 0.003, -0.001))), 0.0088, 2.0),
          ('cap', tuple(S), tuple(M), 0.0086, 2.0), ('cap', tuple(M), tuple(P), 0.0079, 2.0),
          # the paw: a spoon, curved in toward the body and up at its end
          ('ell', tuple(P + d * 0.004 + Vector((-sx * 0.001, 0, 0))), (0.0080, 0.0068, 0.0102), 2.0,
           (-0.55, sx * 0.25, 0))]
    return meta(el, res=0.0011)


def teddy_leg(i):
    H, A, F = TEDDY_LEGS[i]
    el = [('ball', tuple(H + Vector((0, 0.002, 0.004))), 0.0138, 1.8),         # the thigh, round at its joint
          ('cap', tuple(H + Vector((0, 0, 0.006))), tuple(A), 0.0112, 2.0),
          ('ball', tuple(A - Vector((0, 0, 0.012))), 0.0104, 1.6),
          # the foot: long, oval, its sole forward
          ('ell', tuple(F), (0.0096, 0.0170, 0.0082), 2.0, (-0.20, 0, 0))]
    return meta(el, res=0.0011)


def teddy_body_mesh():
    if 'body' not in _teddy:
        torso = decimate(teddy_torso(), 520)
        arms = [decimate(teddy_arm(i), 230) for i in range(2)]
        legs = [decimate(teddy_leg(i), 260) for i in range(2)]
        legs = [to_floor(flatten_below(l.copy(), l.bounds()[0].y + 0.003, 0.2)) for l in legs]
        _teddy['torso'], _teddy['arms'], _teddy['legs'] = torso, arms, legs
    return _teddy


def teddy_body():
    m = teddy_body_mesh()
    return m['torso'] + m['arms'][0] + m['arms'][1] + m['legs'][0] + m['legs'][1]


def teddy_pads():
    """Felt pads: an oval on the front of each paw, one on each sole."""
    m = teddy_body_mesh()
    out = Shape()
    for i in range(2):
        sx = -1 if i == 0 else 1
        S, M, P = TEDDY_ARMS[i]
        bvh = bvh_of(m['arms'][i])
        c = P + (P - M).normalized() * 0.004
        hit, n = on_surface(bvh, c, Vector((-sx * 0.45, -0.25, 0.85)))
        out = out + project_patch(bvh, hit, n, (P - M), 0.0100, 0.0150, 0.0014, seg=16, rings=3)
        H, A, F = TEDDY_LEGS[i]
        bvh = bvh_of(m['legs'][i])
        hit, n = on_surface(bvh, F + Vector((0, 0.0, -0.002)), Vector((0, 0.18, 1)))
        out = out + project_patch(bvh, hit, n, (0, 1, 0), 0.0135, 0.0245, 0.0014, seg=18, rings=3)
    return out


def teddy_head_raw():
    el = [('ell', tuple(TEDDY_HC), (0.0255, 0.0235, 0.0225)),
          ('ell', (0, 0.034, -0.004), (0.020, 0.016, 0.018), 1.6),                 # the crown, broad
          ('cap', (0, 0.021, 0.012), (0, 0.0175, 0.032), 0.0086, 2.0),             # the muzzle, long
          ('ball', (0, 0.0175, 0.0335), 0.0074, 2.0)]
    for sx in (-1, 1):
        el.append(('ell', (sx * 0.011, 0.019, 0.010), (0.013, 0.012, 0.012), 1.5))   # the cheeks
    s = meta(el, res=0.0010)
    # the gusset's two seams, from the muzzle over the crown to the nape
    for sx in (-1, 1):
        s = groove(s, lambda p, sx=sx: abs(p.x - sx * (0.0045 + 0.0035 * max(0.0, min(1.0, (0.02 - p.z) / 0.02)))) if p.y > 0.012 else 1.0,
                   0.0003, 0.0014, lambda p: TEDDY_HC)
    return s


def teddy_ear(sx):
    """An ear: two layers of mohair sewn into a cup, its open side forward,
    set across the head's seam (its lower third in the head)."""
    e = sphere(1.0, seg=12, rings=8).scale(0.0108, 0.0098, 0.0030)
    e = e.warp(lambda p: Vector((p.x, p.y, p.z + 90.0 * (p.x * p.x + 0.5 * p.y * p.y))))   # cupped
    e.rot(0.15, 0, -sx * 0.30)
    return e.move(sx * 0.0185, 0.0445, -0.007)


def teddy_head_mesh():
    if 'head' not in _teddy:
        h = decimate(teddy_head_raw(), 760)
        _teddy['head'] = h
        _teddy['head_bvh'] = bvh_of(h)
    return _teddy['head']


def teddy_head():
    return teddy_head_mesh() + teddy_ear(-1) + teddy_ear(1)


def teddy_eyes():
    """Glass eyes on wires, set close by the muzzle's root."""
    teddy_head_mesh()
    bvh = _teddy['head_bvh']
    out = Shape()
    for sx in (-1, 1):
        hit, n = on_surface(bvh, TEDDY_HC, Vector((sx * 0.42, 0.16, 0.89)), sink=0.0010)
        out = out + sphere(0.0029, seg=10, rings=6).move(*hit)
    return out


def teddy_nose():
    """The nose embroidered in black floss, satin stitches across a shield
    on the muzzle's end, and the mouth: a stitch down from it and two out."""
    teddy_head_mesh()
    bvh = _teddy['head_bvh']
    inside = Vector((0, 0.0175, 0.022))
    out = Shape()
    top, bot = 0.0222, 0.0150           # the shield's top and point (y), at the muzzle's end
    for k in range(11):
        t = (k - 5) / 5.0
        x = t * 0.0056
        y0 = bot + (top - bot) * (0.25 + 0.75 * abs(t) ** 1.6) * 0.95
        pts = surface_path(bvh, [(x, top, 0.045), (x * 0.95, (top + y0) / 2, 0.045), (x * 0.9, y0, 0.045)], 0.0004, inside)
        out = out + tube(pts, 0.00072, seg=4, caps=True)
    # the mouth: down from the nose's point, then out each way
    mid = surface_path(bvh, [(0, bot, 0.045), (0, bot - 0.0045, 0.045)], 0.0003, inside)
    out = out + tube(mid, 0.00045, seg=4)
    for sx in (-1, 1):
        arm = surface_path(bvh, [(0, bot - 0.0045, 0.045), (sx * 0.0030, bot - 0.0058, 0.045), (sx * 0.0058, bot - 0.0052, 0.045)], 0.0003, inside)
        out = out + tube(arm, 0.00045, seg=4)
    return out


auto('Real__teddyBody', teddy_body, 1500, smooth=70)
auto('Real__teddyBodyPile', lambda: shell(teddy_body(), 0.0011, floor=0.0004), 1500, smooth=70)
auto('Real__teddyPads', teddy_pads, 360, smooth=60)
auto('Real__teddyHead', teddy_head, 1150, smooth=70)
auto('Real__teddyHeadPile', lambda: shell(teddy_head(), 0.0010), 1150, smooth=70)
auto('Real__teddyEyes', teddy_eyes, 280, smooth=80)
auto('Real__teddyNose', teddy_nose, 560, smooth=60)


# ======================================================== the plush teddy ===
# A machine-made plush teddy bear of the 1980s-90s, 30 cm standing (the
# shop and fairground kind: acrylic plush pile on a knit backing, stuffed
# with polyester fibre, unjointed, sewn to sit; a contrasting velour muzzle,
# inner ears and foot pads; plastic safety eyes and a moulded nose; the
# mouth stitched; a satin ribbon tied in a bow at its neck). Slumped where
# it was left: its head fallen forward and to one side, its arms limp. Its
# frame: x across, y up from the floor, z out of its front.
PB_NECK = Vector((0.0, 0.119, -0.002))
PB_HEAD_TURN = Euler((0.30, 0.10, 0.17), 'XYZ')       # forward, a little round, to its left
PB_HC = Vector((0.0, 0.050, 0.0))                    # the head's centre in its own frame
_pb = {}


def pb_head_to_body(p):
    return PB_NECK + PB_HEAD_TURN.to_matrix() @ Vector(p)


def pb_body_raw():
    el = [('ell', (0, 0.048, 0.004), (0.052, 0.048, 0.044)),
          ('ell', (0, 0.089, -0.004), (0.040, 0.032, 0.034), 1.8),
          ('ell', (0, 0.030, 0.020), (0.044, 0.026, 0.034), 1.6)]          # the belly sagging into the lap
    for sx in (-1, 1):
        el += [('cap', (sx * 0.030, 0.026, 0.020), (sx * 0.041, 0.023, 0.078), 0.0235, 2.0),
               ('ell', (sx * 0.043, 0.032, 0.090), (0.022, 0.030, 0.015), 2.0, (-0.28, 0, 0)),          # the foot
               ('cap', (sx * 0.040, 0.099, 0.004), (sx * 0.060, 0.060, 0.030), 0.0180, 2.0),          # the arm, limp
               ('ball', (sx * 0.061, 0.053, 0.036), 0.0190, 2.0)]                                   # the paw
    s = meta(el, res=0.0016)
    s = flatten_below(to_floor(s), 0.006, 0.15)
    s = lumpy(s, 0.0012, 45.0, 3)
    # the seams: down the front and the back, the legs' insides
    s = groove(s, lambda p: abs(p.x) if p.y > 0.035 else 1.0, 0.0010, 0.0020, lambda p: Vector((0, p.y, -0.004)))
    return to_floor(s)


def pb_head_raw():
    el = [('ell', tuple(PB_HC), (0.056, 0.048, 0.049)),
          ('ell', (0, 0.064, -0.006), (0.046, 0.032, 0.040), 1.6)]
    for sx in (-1, 1):
        el.append(('ell', (sx * 0.024, 0.036, 0.018), (0.030, 0.026, 0.028), 1.5))     # the cheeks
    s = meta(el, res=0.0016)
    s = lumpy(s, 0.0008, 40.0, 5)
    # the head's gusset seams, crown to nape
    for sx in (-1, 1):
        s = groove(s, lambda p, sx=sx: abs(p.x - sx * 0.016) if p.y > 0.06 else 1.0, 0.0008, 0.0018, lambda p: PB_HC)
    return s


def pb_parts():
    if 'body' not in _pb:
        body = decimate(pb_body_raw(), 1150)
        head = decimate(pb_head_raw(), 760)
        _pb['body'], _pb['head'] = body, head
        _pb['head_bvh'] = bvh_of(head)
        _pb['body_bvh'] = bvh_of(body)
    return _pb


def pb_ear(sx, inner=False):
    """An ear: two layers of plush sewn into a cup, its lower third in the
    head; `inner`: the velour inside it."""
    e = sphere(1.0, seg=14, rings=8).scale(0.0235, 0.0215, 0.0075)
    e = e.warp(lambda p: Vector((p.x, p.y, p.z + 30.0 * (p.x * p.x + 0.6 * p.y * p.y))))
    if inner:
        bvh = bvh_of(e)
        e = project_patch(bvh, Vector((0, 0.002, 0.03)), (0, 0, 1), (0, 1, 0), 0.032, 0.028, 0.0020, seg=14, rings=2)
        e = Shape([p.copy() for p in e.v], e.f)
    e.rot(0.10, 0, -sx * 0.32)
    return e.move(sx * 0.043, 0.088, -0.008)


def pb_muzzle_raw():
    return meta([('ell', (0, 0.031, 0.040), (0.027, 0.020, 0.020)), ('ell', (0, 0.026, 0.046), (0.020, 0.014, 0.014), 1.6)], res=0.0012)


def pb_fur():
    p = pb_parts()
    head = p['head'] + pb_ear(-1) + pb_ear(1)
    return p['body'] + head.copy().warp(pb_head_to_body)


def pb_accent():
    """The velour: the muzzle, the inner ears, the soles."""
    p = pb_parts()
    out = decimate(pb_muzzle_raw(), 300) + pb_ear(-1, True) + pb_ear(1, True)
    out.warp(pb_head_to_body)
    bvh = p['body_bvh']
    for sx in (-1, 1):
        c = Vector((sx * 0.043, 0.032, 0.080))
        hit, n = on_surface(bvh, c, Vector((0, 0.28, 1)))
        out = out + project_patch(bvh, hit, n, (0, 1, 0), 0.034, 0.048, 0.0024, seg=18, rings=3)
    return out


def _pb_muzzle_bvh():
    if 'mz' not in _pb:
        _pb['mz'] = bvh_of(decimate(pb_muzzle_raw(), 300))
    return _pb['mz']


def _pb_eye_spot(sx):
    hit, n = on_surface(pb_parts()['head_bvh'], PB_HC, Vector((sx * 0.37, 0.10, 0.92)))
    return hit, n


def pb_eye(sx):
    """A safety eye: a black plastic dome, 12 mm, its shank through the
    plush."""
    hit, n = _pb_eye_spot(sx)
    dome = lathe([(0.0060, 0.0), (0.0060, 0.0006), (0.0052, 0.0028), (0.0034, 0.0042), (0.0, 0.0047)], seg=16, cap_bottom=True)
    q = Vector((0, 1, 0)).rotation_difference(n).to_matrix().to_4x4()
    dome.apply(q).move(*(hit - n * 0.0012))
    return dome.warp(pb_head_to_body)


def pb_eye_holes():
    """Where each eye goes through: the plush pierced, its pile parted round
    a dark hole (seen where an eye has been pulled out)."""
    bvh = pb_parts()['head_bvh']
    out = Shape()
    for sx in (-1, 1):
        hit, n = _pb_eye_spot(sx)
        rng = random.Random(77 + sx)
        jit = [rng.uniform(0.8, 1.25) for _ in range(12)]
        out = out + project_patch(bvh, hit, n, (0, 1, 0), 0.0072, 0.0072, 0.0004, seg=12, rings=2,
                                  shape=lambda x, y, a, t: (x * jit[int(a / TAU * 12) % 12] ** t, y * jit[int(a / TAU * 12) % 12] ** t))
    return out.warp(pb_head_to_body)


def pb_nose():
    """The moulded nose: a rounded triangle, its point down, on the
    muzzle's top front."""
    bvh = _pb_muzzle_bvh()
    hit, n = on_surface(bvh, Vector((0, 0.028, 0.038)), Vector((0, 0.55, 0.84)), sink=0.0015)
    tri = round_outline([(-0.0085, 0.0035), (0.0085, 0.0035), (0.0, -0.0060)], 0.0030, seg=4)
    nose = prism(tri, 0.0060, bevel_r=0.0018, bevel_seg=2)
    up = Vector((0, 1, 0))
    m = Matrix((n.cross(up).normalized() * -1, (up - n * up.dot(n)).normalized(), n)).transposed().to_4x4()
    nose.apply(m).move(*hit)
    return nose.warp(pb_head_to_body)


def pb_mouth():
    """The mouth stitched in black thread: down from the nose and out."""
    bvh = _pb_muzzle_bvh()
    inside = Vector((0, 0.028, 0.036))
    out = Shape()
    top, low = 0.0285, 0.0215
    out = out + tube(surface_path(bvh, [(0, top, 0.06), (0, low, 0.06)], 0.0003, inside), 0.0006, seg=4)
    for sx in (-1, 1):
        out = out + tube(surface_path(bvh, [(0, low, 0.06), (sx * 0.006, low - 0.0025, 0.06), (sx * 0.011, low - 0.0005, 0.06)], 0.0003, inside), 0.0006, seg=4)
    return out.warp(pb_head_to_body)


def pb_bow():
    """A satin ribbon (25 mm) round the neck, tied at the front in a bow,
    its tails hanging; its loops crushed flat."""
    out = Shape()
    # the band, round the neck where the head's seam meets the body, laid
    # on the plush (cast out from the neck's axis), a little loose
    bvh = bvh_of(pb_fur())
    n, R = 40, 0.044
    pts = []
    for i in range(n + 1):
        a = TAU * i / n
        y = 0.115 + 0.003 * math.cos(a)
        o = Vector((0.0, y, -0.004))
        d = Vector((math.sin(a), 0.0, math.cos(a)))
        hit = bvh.ray_cast(o, d, 0.2)
        r = (hit[0] - o).length + 0.0012 if hit[0] is not None else R
        pts.append(o + d * r)
    band = _ribbon(pts, 0.016, 0.0012, lambda p: Vector((p.x, 0.0, p.z + 0.004)))
    out = out + band
    # the bow, slipped round to its right side of the chest (out from under
    # the fallen head): built facing +z, turned to where it sits
    k = 3
    a_k = TAU * k / n
    out_d = Vector((math.sin(a_k), 0.0, math.cos(a_k)))
    rot = Matrix.Rotation(a_k, 4, 'Y')
    knot = Vector((0.0, -0.004, 0.006))
    bow = Shape()
    for sx in (-1, 1):
        # a loop: out to the side and back, flattened, drooping
        lp = [knot + Vector((sx * 0.004, 0.002, 0.001)), knot + Vector((sx * 0.018, 0.008, 0.004)),
              knot + Vector((sx * 0.030, 0.003, 0.004)), knot + Vector((sx * 0.028, -0.007, 0.003)),
              knot + Vector((sx * 0.012, -0.004, 0.002)), knot + Vector((sx * 0.003, -0.001, 0.0))]
        bow = bow + _ribbon(lp, 0.014, 0.0012, Vector((0, 0, 1)))
        # a tail: down over the chest, its end cut square
        tl = [knot + Vector((sx * 0.003, -0.004, 0.002)), knot + Vector((sx * 0.010, -0.020, 0.002)),
              knot + Vector((sx * 0.015, -0.036, -0.002)), knot + Vector((sx * 0.017, -0.048, -0.008))]
        bow = bow + _ribbon(tl, 0.013, 0.0012, Vector((0, 0, 1)))
    bow = bow + rbox(0.010, 0.012, 0.006, r=0.002).move(*knot)
    bow.apply(rot).move(*(pts[k] - out_d * 0.001))

    # laid on the plush: nothing of it inside the bear
    def lay(q):
        o = q - out_d * 0.06
        last, at = None, o
        for _ in range(8):            # the outermost surface (the shells overlap)
            hit = bvh.ray_cast(at, out_d, 0.12)
            if hit[0] is None:
                break
            last, at = hit[0], hit[0] + out_d * 1e-4
        if last is not None:
            surf = (last - o).dot(out_d) + 0.0015
            if (q - o).dot(out_d) < surf:
                return o + out_d * surf + (q - o - out_d * (q - o).dot(out_d))
        return q
    return out + bow.warp(lay)


def _ribbon(pts, w, t, face):
    """A flat band along a polyline, `w` wide, `t` thick, its flat side
    toward `face` (a vector, or a function of the point)."""
    v, f = [], []
    pts = [Vector(p) for p in pts]
    for i, p in enumerate(pts):
        d = (pts[min(i + 1, len(pts) - 1)] - pts[max(i - 1, 0)]).normalized()
        fc = Vector(face(p) if callable(face) else face)
        fc = (fc - d * fc.dot(d)).normalized()
        across = fc.cross(d).normalized()
        for sw, st in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            v.append(p + across * (sw * w / 2) + fc * (st * t / 2))
    k = len(pts)
    for i in range(k - 1):
        for j in range(4):
            a, b = i * 4 + j, i * 4 + (j + 1) % 4
            f.append((a, b, b + 4, a + 4))
    f.append((3, 2, 1, 0))
    f.append(tuple((k - 1) * 4 + j for j in range(4)))
    return _fix_winding(Shape(v, f))


auto('Real__plushBearFur', pb_fur, 2400, smooth=70)
auto('Real__plushBearPile', lambda: shell(pb_fur(), 0.0016, floor=0.0004), 2400, smooth=70)
auto('Real__plushBearAccent', pb_accent, 700, smooth=70)
auto('Real__plushBearEyeL', lambda: pb_eye(1), 200, smooth=60)
auto('Real__plushBearEyeR', lambda: pb_eye(-1), 200, smooth=60)
auto('Real__plushBearEyeHoles', pb_eye_holes, 100)
auto('Real__plushBearNose', pb_nose, 400, smooth=50)
auto('Real__plushBearMouth', pb_mouth, 200)
auto('Real__plushBearBow', pb_bow, 700, smooth=40)


# ===================================================== the teddy's arm ===
# A plush teddy's arm (a 40 cm bear of the plush teddy's kind) torn off at
# its shoulder seam: the arm sewn of two pieces round its edge, a velour pad
# on its paw, the seam's threads broken at the torn end and the polyester
# fibre stuffing pulled out of it; tufts of it lying near. Lying on its
# side. Its frame: x along the arm (the paw toward +x, the torn end at
# -x), y up from the floor, z across.
TA_Y = 0.0150                  # the arm's middle (its edge seam) above the floor
TA_END = -0.050                # the torn end


def ta_arm_raw():
    el = [('ell', (-0.034, TA_Y, 0.000), (0.023, 0.0152, 0.0225)),
          ('ell', (0.000, TA_Y - 0.0006, 0.001), (0.026, 0.0136, 0.0172)),
          ('ell', (0.034, TA_Y - 0.0008, 0.002), (0.020, 0.0132, 0.0210), 1.8),
          ('ell', (0.048, TA_Y - 0.0004, 0.003), (0.011, 0.0118, 0.0172), 1.6)]   # the paw: broad, flatter, its end round
    s = meta(el, res=0.0011)
    # curved as a teddy's arm is, toward its paw's inside
    s = bend(s, 0.80, axis='y', along='x')
    lo, hi = s.bounds()
    s.move(0, 0, -(lo.z + hi.z) / 2)
    s = lumpy(s, 0.0007, 70.0, 11)
    # torn off: cut a little aslant
    cutter = box(0.06, 0.08, 0.08).rot(0, 0.18, 0.10).move(TA_END - 0.030, TA_Y, 0)
    s = boolean(s, cutter, 'DIFFERENCE')
    # the edge seam round its outline, at the middle of its thickness
    s = groove(s, lambda p: abs(p.y - TA_Y) if abs(p.z) > 0.008 or p.x > 0.04 else 1.0, 0.0010, 0.0022,
               lambda p: Vector((p.x, TA_Y, 0.0)))
    return s


def ta_arm_mesh():
    if 'ta' not in _pb:
        a = decimate(ta_arm_raw(), 820)
        lo, _ = a.bounds()
        a = flatten_below(a.move(0, -lo.y, 0), 0.0012, 0.25)
        _pb['ta'] = a
        _pb['ta_bvh'] = bvh_of(a)
    return _pb['ta']


def ta_rim(n=14):
    """Points round the torn end's rim (its cut face's edge)."""
    arm = ta_arm_mesh()
    bvh = _pb['ta_bvh']
    pts = []
    for i in range(n):
        a = TAU * i / n
        d = Vector((0.0, math.sin(a), math.cos(a)))
        o = Vector((TA_END + 0.004, TA_Y, 0.0))
        hit = bvh.ray_cast(o, d, 0.05)
        pts.append(hit[0] if hit[0] is not None else o + d * 0.015)
    return pts


def ta_fur():
    """The arm, and at its torn end the plush's ragged edge (flaps of it
    turned out) and the seam's broken threads."""
    out = ta_arm_mesh().copy()
    rng = random.Random(4511)
    for i, p in enumerate(ta_rim(14)):
        r = Vector((0, p.y - TA_Y, p.z)).normalized()
        if i % 3 != 1:
            # a flap of the torn plush, turned a little out
            w, l = rng.uniform(0.004, 0.008), rng.uniform(0.004, 0.009)
            flap = Shape([(-w / 2, 0, 0), (w / 2, 0, 0), (w * 0.2, l, 0.0006), (-w * 0.3, l * 0.8, 0.0006)], [(0, 1, 2, 3)])
            flap = flap + Shape([(-w / 2, 0, -0.0007), (-w * 0.3, l * 0.8, -0.0001), (w * 0.2, l, -0.0001), (w / 2, 0, -0.0007)], [(0, 1, 2, 3)])
            ax = Vector((-1, 0, 0)) * 0.6 + r * 0.8
            m = Matrix((r.cross(ax).normalized(), ax.normalized(), r)).transposed().to_4x4()
            out = out + flap.apply(m).move(*(p - r * 0.0006))
    out = out + fibres(rng, Vector((TA_END - 0.002, TA_Y, 0.0)), 0.016, 7, 0.010, r=0.00035, up=0.2)
    lo, _ = out.bounds()
    return out.warp(lambda q: Vector((q.x, max(q.y, 0.0003), q.z)))


def ta_pad():
    arm = ta_arm_mesh()
    bvh = _pb['ta_bvh']
    hit, n = on_surface(bvh, Vector((0.036, TA_Y, 0.003)), Vector((0.15, 1, 0.1)))
    return project_patch(bvh, hit, n, (1, 0, 0), 0.032, 0.027, 0.0024, seg=18, rings=3)


def fluff(rng, centre, radius, n, rmin, rmax, squash=1.0, res=0.0009):
    """Polyester fibre stuffing: a soft cloud of small tufts run together,
    and fibres standing out of it."""
    el = []
    pts = []
    for _ in range(n):
        while True:
            q = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
            if q.length <= 1:
                break
        c = Vector(centre) + Vector((q.x, q.y * squash, q.z)) * radius
        r = rng.uniform(rmin, rmax)
        el.append(('ball', tuple(c), r, 1.2))
        pts.append((c, r))
    s = meta(el, res=res)
    s = noisy(s, 0.0004, scale=400.0, seed=rng.randint(0, 99))
    # loose fibres out of its surface, curled
    for c, r in pts[: n // 2]:
        s = s + fibres(rng, c, r * 1.05, 2, r * 0.9, r=0.00022, up=0.9)
    return s


def ta_stuffing():
    """The stuffing pulled out of the torn end: a bulge of it, wisps."""
    rng = random.Random(4523)
    c = Vector((TA_END - 0.006, TA_Y + 0.001, 0.0))
    s = fluff(rng, c, 0.0130, 40, 0.0035, 0.0070, squash=0.85)
    s = s + fluff(rng, c + Vector((-0.013, -0.004, 0.006)), 0.008, 16, 0.0025, 0.0050, squash=0.7)
    s = decimate(s, 1000)
    return s.warp(lambda q: Vector((q.x, max(q.y, 0.0003), q.z)))


def ta_tuft():
    """A tuft of the stuffing lying loose: its origin on the floor under it."""
    rng = random.Random(4537)
    s = fluff(rng, Vector((0, 0.006, 0)), 0.011, 18, 0.0025, 0.0055, squash=0.55)
    s = decimate(s, 420)
    lo, _ = s.bounds()
    s.move(0, -lo.y, 0)
    return flatten_below(s, 0.0015, 0.3)


auto('Real__teddyArmFur', ta_fur, 1100, smooth=70)
auto('Real__teddyArmPile', lambda: shell(ta_arm_mesh(), 0.0016, floor=0.0004), 900, smooth=70)
auto('Real__teddyArmPad', ta_pad, 400, smooth=60)
auto('Real__teddyArmStuffing', ta_stuffing, 1100, smooth=80)
auto('Real__teddyArmStuffingWisps', lambda: shell(ta_stuffing(), 0.0012, floor=0.0004), 1100, smooth=80)
auto('Real__teddyArmTuft', ta_tuft, 460, smooth=80)
auto('Real__teddyArmTuftWisps', lambda: shell(ta_tuft(), 0.0010, floor=0.0003), 460, smooth=80)


# ======================================================== the elephant ===
# A bean-bag plush elephant of the 1990s, 19 cm from tail to trunk (the
# pellet-filled kind sold by the million: short plush on a knit backing, its
# body loosely filled with PVC pellets so it lies limp, its head stuffed
# with fibre; big floppy ears of two layers sewn round, plastic eyes, a
# thin plush tail; a fabric law tag sewn into the seam at its rear).
# Lying on its belly, its legs splayed. Its frame: x across, y up from the
# floor, z out of its front (the trunk). The ears are built in their hinge
# frames (origin at the root on the head's side, EL_EAR), where the game
# hangs them.
EL_EAR = [Vector((sx * 0.0290, 0.058, 0.052)) for sx in (-1, 1)]
EL_HC = Vector((0.0, 0.054, 0.054))


def el_body_raw():
    el = [('ell', (0, 0.026, -0.010), (0.040, 0.026, 0.054)),
          ('ell', (0, 0.020, -0.040), (0.036, 0.020, 0.024), 1.6),             # the haunches, slumped
          ('ell', tuple(EL_HC), (0.031, 0.029, 0.027)),                          # the head
          ('ell', (0, 0.066, 0.046), (0.024, 0.016, 0.020), 1.6),               # the brow, domed
          ('cap', (0, 0.048, 0.074), (0, 0.034, 0.092), 0.0105, 2.0),           # the trunk, down to the floor
          ('cap', (0, 0.034, 0.092), (0, 0.016, 0.101), 0.0086, 2.0),
          ('cap', (0, 0.016, 0.101), (0, 0.009, 0.113), 0.0070, 2.0),
          ('ball', (0, 0.011, 0.118), 0.0064, 2.0)]                              # its end, turned up
    for sx in (-1, 1):
        for sz, z0, z1 in ((1, 0.030, 0.046), (-1, -0.040, -0.056)):
            el.append(('cap', (sx * 0.026, 0.018, z0), (sx * 0.046, 0.010, z1), 0.0125, 2.0))   # the legs, splayed
            el.append(('ell', (sx * 0.049, 0.010, z1 + sz * 0.002), (0.0045, 0.0110, 0.0110), 2.0, (0, 0, sx * 0.4)))  # the sole
    s = meta(el, res=0.0013)
    s = flatten_below(to_floor(s), 0.008, 0.2)
    s = lumpy(s, 0.0009, 55.0, 17)
    # the seams: along the back and over the head to the trunk's root
    s = groove(s, lambda p: abs(p.x) if p.y > 0.035 or p.z < -0.02 else 1.0, 0.0008, 0.0018,
               lambda p: Vector((0, max(0.012, p.y - 0.02), p.z)))
    return to_floor(s)


def el_body_mesh():
    if 'el' not in _pb:
        b = decimate(el_body_raw(), 1150)
        _pb['el'] = b
        _pb['el_bvh'] = bvh_of(b)
    return _pb['el']


def el_tail():
    """The tail: a thin twisted plush tube with a tuft, off its rump."""
    pts = [(0.0, 0.030, -0.060), (0.002, 0.024, -0.071), (0.006, 0.014, -0.077), (0.009, 0.006, -0.080)]
    return tube(pts, 0.0022, seg=6, smooth=1, r_end=0.0018) + sphere(0.0032, seg=8, rings=5).move(0.0095, 0.0045, -0.081)


def el_body():
    return el_body_mesh() + el_tail()


def el_ear(sx):
    """An ear: two layers of plush sewn round a D, unstuffed, floppy, hanging
    from its root on the head's side, back and down; its hinge frame. Its
    two layers lie apart in the middle and meet at the seam round its edge
    (a pillow, thin at the rim), it hangs from its root under its own
    weight and its lower half droops onto the shoulder in soft folds. (The
    first was a flat plate of even thickness bent out at its corner, read
    as a panel standing stiff: M8.)"""
    out = [(0.0, 0.010), (-0.016, 0.016), (-0.036, 0.012), (-0.052, -0.006), (-0.050, -0.031),
           (-0.034, -0.050), (-0.013, -0.052), (0.0, -0.032)]
    e = prism(round_outline(out, 0.007, seg=3), 0.0034, bevel_r=0.0015, bevel_seg=2)
    e = subdivide(e, 2)
    # the D lies in the y-z plane, its flat side facing out (+x)
    e.rot(0, -math.pi / 2, 0)
    e = Shape([Vector((p.x, p.y, p.z)) for p in e.v], e.f)
    # how far a point (y, z) is inside the outline (0 at the seam)
    poly = [(z, y) for (z, y) in out]

    def inside(y, z):
        d = 1.0
        for (z0, y0), (z1, y1) in zip(poly, poly[1:] + poly[:1]):
            ez, ey = z1 - z0, y1 - y0
            t = max(0.0, min(1.0, ((z - z0) * ez + (y - y0) * ey) / (ez * ez + ey * ey)))
            d = min(d, math.hypot(z - (z0 + ez * t), y - (y0 + ey * t)))
        return d

    def flop(p):
        z, y = p.z, p.y
        k = min(1.0, inside(y, z) / 0.012)
        # the pillow: thin at the seam, full between
        x = p.x * (0.35 + 0.65 * k)
        x += 0.0012 * k
        # it hangs: the lower half drops out over the shoulder and curls
        # back in under its weight, the back edge a little away from the head
        low = max(0.0, -y - 0.006)
        x += 9.0 * low * low - 70.0 * low ** 3 + 3.2 * z * z
        # soft folds down it, more toward its free edge
        x += (0.0016 * math.sin(z * 150.0 + 0.6) + 0.0009 * math.sin(z * 290.0 - y * 60.0)) * min(1.0, low / 0.02 + 0.2)
        # its weight pulls the lower edge down and back a little
        return Vector((x, y - 0.06 * low * low, z - 0.10 * low * low))
    e = e.warp(flop)
    e = decimate(e, 440)
    if sx < 0:
        e = e.scale(-1, 1, 1)
    return e


def el_eyes():
    el_body_mesh()
    bvh = _pb['el_bvh']
    out = Shape()
    for sx in (-1, 1):
        hit, n = on_surface(bvh, EL_HC, Vector((sx * 0.55, 0.22, 0.80)))
        dome = lathe([(0.0034, 0.0), (0.0034, 0.0005), (0.0028, 0.0018), (0.0016, 0.0027), (0.0, 0.0030)], seg=14, cap_bottom=True)
        q = Vector((0, 1, 0)).rotation_difference(n).to_matrix().to_4x4()
        out = out + dome.apply(q).move(*(hit - n * 0.0006))
    return out


def el_tag():
    """The law tag: a satin label folded in two, its ends sewn into the seam
    at the rump, lying on the flank (printed by the game)."""
    el_body_mesh()
    bvh = _pb['el_bvh']
    root = Vector((0.0, 0.034, -0.058))
    out = Shape()
    # a strip folded over itself: two layers 0.6 mm apart, laid on the body
    w, l = 0.012, 0.024
    for k, off in enumerate((0.0007, 0.0013)):
        g = grid_plane(w, l, 2, 6)
        v = []
        for p in g.v:
            t = (p.y + l / 2) / l
            q = root + Vector((0.004 + t * 0.016 + p.x * 0.3, -t * 0.014, -t * 0.006 + p.x))
            d = (q - Vector((0, 0.022, -0.030))).normalized()
            hit = bvh.ray_cast(Vector((0, 0.022, -0.030)), d, 0.2)
            v.append(hit[0] + hit[1].normalized() * off if hit[0] is not None else q)
        sh = Shape(v, g.f)
        if k == 1:
            sh.f = [tuple(reversed(f)) for f in sh.f]
        out = out + sh
    return out


auto('Real__elephantBody', el_body, 1360, smooth=70)
auto('Real__elephantPile', lambda: shell(el_body_mesh(), 0.0011, floor=0.0004), 1200, smooth=70)
auto('Real__elephantEarL', lambda: el_ear(1), 460, smooth=85)
auto('Real__elephantEarR', lambda: el_ear(-1), 460, smooth=85)
auto('Real__elephantEyes', el_eyes, 240, smooth=60)
auto('Real__elephantTag', el_tag, 100, smooth=40)


# ======================================================== the small doll ===
# An 8 in (20 cm) toddler doll of the 1950s-60s, the small "walker" kind
# (Vogue's Ginny and its many imitators): hard plastic, jointed at the
# neck, shoulders and hips; sleep eyes with painted lashes under them; a
# synthetic wig glued on, cut in a bob with a fringe; a cotton dress with
# puffed sleeves and a gathered skirt, white socks, oilcloth shoes. Its
# head is batch r03's baby's sculpt (the same mould, a smaller size; its
# mouth closed), its body a toddler's. Found as it was dropped: on its
# back (A), or on its front with its head turned onto its cheek (B).
#
# Built standing (x across, y up from its soles, z out of its front), then
# posed at its joints and laid on the floor. The face, eyes, lids and
# lashes are in r03's head frame scaled by SD_K (origin under the neck):
# the game places them by SD_HEAD[pose] (a position and an XYZ euler),
# which this file prints; the skull and the wig are laid in the pose here
# (turned whole, their boxes would reach under the floor and the placement
# pass would lift the doll off it).
SD_K = 0.47
SD_NECK = Vector((0.0, 0.1365, -0.002))
SD_SHOULDER = [Vector((sx * 0.0185, 0.1240, 0.0)) for sx in (-1, 1)]
SD_HIP = [Vector((sx * 0.0095, 0.0860, 0.0)) for sx in (-1, 1)]
# each pose: the root's turn about x, the arms' and legs' swing at their
# joints (about x: + takes a limb back), the head's turn on its neck
SD_POSES = {
    'A': dict(root=-math.pi / 2, arms=(-0.12, 2.94), legs=(0.0, -0.08), neck=0.35, lids=(0.0, 0.10)),
    'B': dict(root=math.pi / 2, arms=(-0.05, 3.30), legs=(0.0, 0.0), neck=-1.35, lids=(0.55, 0.40)),
}
_sd = {}


def sd_torso():
    el = [('ell', (0, 0.1215, 0.0), (0.0165, 0.0120, 0.0115)),
          ('ell', (0, 0.1040, 0.002), (0.0175, 0.0160, 0.0140)),         # the toddler's belly
          ('ell', (0, 0.0890, -0.001), (0.0185, 0.0110, 0.0125)),
          ('cap', (0, 0.1300, -0.001), (0, 0.1390, -0.002), 0.0058, 2.0)]   # the neck, into the head
    return meta(el, res=0.0008)


def sd_arm(i):
    sx = -1 if i == 0 else 1
    S = SD_SHOULDER[i]
    E = S + Vector((sx * 0.003, -0.024, 0.003))
    W = E + Vector((0.0, -0.020, 0.005))
    el = [('ball', tuple(S), 0.0068, 2.0), ('cap', tuple(S), tuple(E), 0.0060, 2.0), ('cap', tuple(E), tuple(W), 0.0054, 2.0),
          ('ell', tuple(W + Vector((-sx * 0.0005, -0.0075, 0.002))), (0.0034, 0.0082, 0.0058), 2.0, (0.15, 0, 0)),   # the hand
          ('ball', tuple(W + Vector((-sx * 0.0035, -0.0040, 0.0055))), 0.0022, 2.0)]                                   # the thumb
    return meta(el, res=0.0006)


def sd_leg(i):
    sx = -1 if i == 0 else 1
    H = SD_HIP[i]
    K_ = Vector((sx * 0.0100, 0.0500, 0.001))
    A = Vector((sx * 0.0100, 0.0120, 0.0))
    el = [('ball', (sx * 0.0095, 0.0830, 0.001), 0.0102, 2.0), ('cap', tuple(H), tuple(K_), 0.0092, 2.0),
          ('cap', tuple(K_), tuple(A), 0.0076, 2.0), ('ball', (sx * 0.0100, 0.0320, -0.0015), 0.0083, 1.6),
          ('ball', tuple(K_ + Vector((0, 0.002, 0.003))), 0.0080, 1.6)]
    return meta(el, res=0.0007)


def sd_sock(i):
    """An ankle sock, its cuff turned down."""
    sx = -1 if i == 0 else 1
    s = lathe([(0.0080, 0.004), (0.0083, 0.012), (0.0082, 0.018), (0.0089, 0.019), (0.0090, 0.024),
               (0.0080, 0.0245)], seg=16, cap_bottom=False, cap_top=False)
    return s.move(sx * 0.0100, 0.0, 0.0)


def sd_shoe(i):
    """A one-strap shoe of oilcloth: its sole, toe, the strap over the
    instep, a button."""
    sx = -1 if i == 0 else 1
    el = [('ell', (sx * 0.0100, 0.0068, 0.0060), (0.0074, 0.0068, 0.0138)),
          ('ell', (sx * 0.0100, 0.0050, 0.0140), (0.0068, 0.0050, 0.0068), 1.8)]
    s = meta(el, res=0.0006)
    s = s.warp(lambda p: Vector((p.x, max(p.y, 0.0012) if p.y < 0.002 else p.y, p.z)))
    strap = torus(0.0088, 0.0009, seg=16, tseg=4).rot(0, math.pi / 2, 0).rot(0.35, 0, 0).move(sx * 0.0100, 0.0095, 0.0045)
    return s + strap


def sd_head_raw():
    """r03's head sculpt, its eyes' openings cut, its mouth closed (no bottle
    hole): the head frame (r03's size; scaled after)."""
    s = R03['_sphere_grid'](56, 36, R03['head_point'])
    for i in range(2):
        sx = -1 if i == 0 else 1
        c = R03['eye_centre'](i)
        pts = [(x, y + sx * x * 0.10) for x, y in R03['_almond']()]
        cutter = prism(pts, 0.05).move(c.x, c.y, c.z + 0.026)
        s = boolean(s, cutter, 'DIFFERENCE')
    return s


def sd_head_mesh():
    if 'head' not in _sd:
        h = decimate(sd_head_raw(), 760)
        _sd['head'] = h
    return _sd['head']


def sd_head_half(face):
    s = sd_head_mesh()
    keep = [f for f in s.f if R03['_face_side'](s, f) == face]
    used = sorted({i for f in keep for i in f})
    remap = {i: k for k, i in enumerate(used)}
    return Shape([s.v[i] * SD_K for i in used], [tuple(remap[i] for i in f) for f in keep])


def _hairline(az):
    """Where the wig ends round the head (az 0: the face): a fringe straight
    across the brow, over the ears, the bob to the jaw at the sides and
    back (elevations, radians)."""
    k = min(1.0, abs(az) / 1.05)
    return math.radians(24) * (1 - k) + math.radians(-36) * k


def sd_wig_raw():
    """The wig: a bob of synthetic hair glued over the scalp, its fringe
    cut straight, hanging to the jaw at the sides and the nape, thick at its
    ends (r03's head frame, r03's size)."""
    na, nt = 48, 14
    rows = []
    for k in range(na):
        az = -math.pi + TAU * k / na
        e_end = _hairline(az)
        col = []
        for j in range(nt + 1):
            t = j / nt
            el = math.pi / 2 + (e_end - math.pi / 2) * t
            if el >= 0:
                d = R03['_dir'](az, el)
                off = 0.0035 + 0.0045 * t
                p = R03['head_point'](d, off)
            else:
                # below the head's widest it hangs: straight down, flaring a little
                d0 = R03['_dir'](az, 0.0)
                p0 = R03['head_point'](d0, 0.0080)
                drop = (0 - el) / (math.pi / 2) * 0.090
                p = p0 + Vector((d0.x, 0, d0.z)) * (drop * 0.12) + Vector((0, -drop, 0))
            # combed waves, cut ends uneven
            p = p + (p - R03['HC']).normalized() * 0.0008 * t * math.sin(az * 18 + t * 5)
            col.append(p)
        rows.append(col)
    v = [R03['head_point'](Vector((0, 1, 0)), 0.0036)]
    for col in rows:
        v += col[1:]
    f = []
    n = nt
    for k in range(na):
        k2 = (k + 1) % na
        f.append((0, 1 + k2 * n, 1 + k * n))
        for j in range(n - 1):
            a0, b0 = 1 + k * n + j, 1 + k2 * n + j
            f.append((a0, b0, b0 + 1, a0 + 1))
    return R03['_fix_winding_out'](Shape(v, f), R03['HC'])


def sd_wig():
    w = decimate(sd_wig_raw(), 900)
    return w.scale(SD_K)


# ---------------------------------------------------- posing the doll ---
def _sd_mats(pose, tilt):
    P = SD_POSES[pose]
    root = Matrix.Rotation(P['root'] + tilt, 4, 'X')
    arms = [root @ Matrix.Translation(S) @ Matrix.Rotation(a, 4, 'X') @ Matrix.Translation(-S) for S, a in zip(SD_SHOULDER, P['arms'])]
    legs = [root @ Matrix.Translation(H) @ Matrix.Rotation(a, 4, 'X') @ Matrix.Translation(-H) for H, a in zip(SD_HIP, P['legs'])]
    head = root @ Matrix.Translation(SD_NECK) @ Matrix.Rotation(P['neck'], 4, 'Y')
    return root, arms, legs, head


def _sd_pieces():
    if 'pieces' not in _sd:
        _sd['pieces'] = dict(
            torso=decimate(sd_torso(), 360),
            arms=[decimate(sd_arm(i), 200) for i in range(2)],
            legs=[decimate(sd_leg(i), 220) for i in range(2)],
            socks=[sd_sock(i) for i in range(2)],
            shoes=[decimate(sd_shoe(i), 200) for i in range(2)],
            head=sd_head_half(False) + sd_head_half(True),
            wig=sd_wig())
    return _sd['pieces']


def _apply(s, m):
    return s.copy().apply(m)


def sd_pose(pose):
    """The pose laid on the floor: the root turned (and tilted so the doll
    rests on what touches), lifted onto the floor, centred: the matrices."""
    if pose in _sd:
        return _sd[pose]
    pc = _sd_pieces()

    def lowest(tilt, region):
        root, arms, legs, head = _sd_mats(pose, tilt)
        ys = []
        if region == 'head':
            for s in (pc['head'], pc['wig']):
                ys += [(head @ p).y for p in s.v]
        else:
            for i in range(2):
                ys += [(legs[i] @ p).y for p in pc['shoes'][i].v]
        return min(ys)
    # tilt about x until the head and the feet rest at the same height
    lo, hi = -0.35, 0.35
    for _ in range(40):
        mid = (lo + hi) / 2
        d = lowest(mid, 'head') - lowest(mid, 'feet')
        dl = lowest(lo, 'head') - lowest(lo, 'feet')
        if (d > 0) == (dl > 0):
            lo = mid
        else:
            hi = mid
    tilt = (lo + hi) / 2
    root, arms, legs, head = _sd_mats(pose, tilt)
    allv = [root @ p for p in pc['torso'].v]
    for i in range(2):
        allv += [arms[i] @ p for p in pc['arms'][i].v] + [legs[i] @ p for p in pc['legs'][i].v + pc['shoes'][i].v]
    allv += [head @ p for p in pc['head'].v + pc['wig'].v]
    ymin = min(p.y for p in allv)
    cx = (min(p.x for p in allv) + max(p.x for p in allv)) / 2
    cz = (min(p.z for p in allv) + max(p.z for p in allv)) / 2
    lift = Matrix.Translation(Vector((-cx, -ymin + 0.0004, -cz)))
    out = dict(root=lift @ root, arms=[lift @ a for a in arms], legs=[lift @ l for l in legs], head=lift @ head)
    hp = out['head'].to_translation()
    he = out['head'].to_euler('ZYX')       # the game's XYZ Euler order
    print(f"SMALLDOLL {pose} head pos ({hp.x:.5f}, {hp.y:.5f}, {hp.z:.5f}) rot ({he.x:.5f}, {he.y:.5f}, {he.z:.5f})")
    _sd[pose] = out
    return out


def _floor(s):
    return s.warp(lambda p: Vector((p.x, max(p.y, 0.0004), p.z)))


def sd_body(pose):
    m = sd_pose(pose)
    pc = _sd_pieces()
    s = _apply(pc['torso'], m['root'])
    for i in range(2):
        s = s + _apply(pc['arms'][i], m['arms'][i]) + _apply(pc['legs'][i], m['legs'][i])
    return _floor(s)


def sd_socks(pose):
    m = sd_pose(pose)
    pc = _sd_pieces()
    return _floor(sum((_apply(pc['socks'][i], m['legs'][i]) for i in range(2)), Shape()))


def sd_shoes(pose):
    m = sd_pose(pose)
    pc = _sd_pieces()
    return _floor(sum((_apply(pc['shoes'][i], m['legs'][i]) for i in range(2)), Shape()))


def sd_dress(pose):
    """The dress, as it lies with the doll: a bodice and puffed sleeves on
    the posed torso and arms, the skirt gathered at the waist and fallen to
    the floor round the hips and thighs, in soft folds (metaballs)."""
    m = sd_pose(pose)
    root = m['root']
    el = []

    def ell_at(mat, c, half, stiff=2.0):
        q = mat @ Vector(c)
        e = mat.to_3x3().to_euler('XYZ')
        el.append(('ell', tuple(q), half, stiff, (e.x, e.y, e.z)))
    ell_at(root, (0, 0.1190, 0.0003), (0.0182, 0.0150, 0.0130))
    ell_at(root, (0, 0.1060, 0.0020), (0.0188, 0.0080, 0.0150), 1.8)
    for i in range(2):
        sx = -1 if i == 0 else 1
        ell_at(m['arms'][i], (sx * 0.0200, 0.1195, 0.0005), (0.0085, 0.0090, 0.0088), 1.8)
    # the skirt: from the waist over the hips and down the thighs ...
    w = root @ Vector((0, 0.104, 0.001))
    hem = root @ Vector((0, 0.061, 0.001))
    el.append(('cap', tuple(w), tuple(w.lerp(hem, 0.55)), 0.0215, 2.0))
    el.append(('cap', tuple(w.lerp(hem, 0.55)), tuple(hem), 0.0240, 2.0))      # full at its hem
    # ... its sides fallen out onto the floor either side of the hips and
    # thighs, in folds
    axis = (hem - w).normalized()
    side = Vector((axis.z, 0.0, -axis.x)).normalized()
    yaw = math.atan2(axis.x, axis.z)
    rng = random.Random(4561 + ord(pose))
    for sgn in (-1, 1):
        for k in range(3):
            t = 0.15 + 0.35 * k
            c = w.lerp(hem, t) + side * sgn * rng.uniform(0.019, 0.022)
            c.y = rng.uniform(0.0035, 0.005)
            el.append(("ell", tuple(c), (0.0095, 0.0032, 0.012), 1.6, (0, yaw + rng.uniform(-0.3, 0.3), 0)))
    s = meta(el, res=0.0008)
    # the cotton's folds and gathers
    def fold(p):
        q = Vector((p.x * 160, p.y * 90, p.z * 160))
        return p + Vector((0, 0.0008 * noise.noise(q), 0)) + Vector((0.0006 * noise.noise(q * 1.7), 0, 0.0006 * noise.noise(q * 1.9 + Vector((3, 1, 7)))))
    s = s.warp(fold)
    s = decimate(s, 1000)
    return flatten_below(s, 0.0008, 0.0)


def sd_skull(pose):
    """The head but its face, laid as the pose lays it (its box, turned,
    would reach under the floor and lift the doll)."""
    return _floor(_apply(sd_head_half(False), sd_pose(pose)['head']))


def sd_wig_posed(pose):
    return _floor(_apply(sd_wig(), sd_pose(pose)['head']))


def sd_lids(pose):
    return r03_lids(SD_POSES[pose]['lids']).scale(SD_K)


def sd_lashes(pose):
    return r03_lashes(SD_POSES[pose]['lids']).scale(SD_K)


for _p in ('A', 'B'):
    auto(f'Real__smallDoll{_p}Body', lambda p=_p: sd_body(p), 1300, smooth=70)
    auto(f'Real__smallDoll{_p}Dress', lambda p=_p: sd_dress(p), 1100, smooth=70)
    auto(f'Real__smallDoll{_p}Socks', lambda p=_p: sd_socks(p), 340, smooth=60)
    auto(f'Real__smallDoll{_p}Shoes', lambda p=_p: sd_shoes(p), 560, smooth=50)
    auto(f'Real__smallDoll{_p}Lids', lambda p=_p: sd_lids(p), 280, smooth=70)
    auto(f'Real__smallDoll{_p}Lashes', lambda p=_p: sd_lashes(p), 100)
    auto(f'Real__smallDoll{_p}Skull', lambda p=_p: sd_skull(p), 560, smooth=70)
    auto(f'Real__smallDoll{_p}Wig', lambda p=_p: sd_wig_posed(p), 1000, smooth=50)
auto('Real__smallDollFace', lambda: sd_head_half(True), 560, kind='plane', smooth=70)
auto('Real__smallDollEyeR', lambda: R03['doll_eye'](0).scale(SD_K), 280, kind='sphere', smooth=80)
auto('Real__smallDollEyeL', lambda: R03['doll_eye'](1).scale(SD_K), 280, kind='sphere', smooth=80)
