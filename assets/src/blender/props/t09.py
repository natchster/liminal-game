"""t09: the props remodelled at their real sizes (docs/PROPS.md, P3, the
rare): a shrine's things, what a person brought and set out (docs/LORE.md;
docs/PROPS.md 1, "the party and the shrine"): the resin skulls stacked, the
glass skull, the brass candlestick and its candle burnt down, the carved
owl and the carnival mask. Sources and sizes: t09.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame: x across its front, y up from the floor, z out of
its front (a face's), unless its section says otherwise.
"""
from propkit import *
from propkit import _mesh_obj, _eval_to_shape, _drop
import propkit
import os
import random
import zlib
from mathutils.bvhtree import BVHTree


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, smooth=35.0, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    only = os.environ.get('T09_ONLY')
    if only and not any(name.startswith(o) for o in only.split(',')):
        return
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
    """A blobby surface from metaball elements (r03's, t08's): ('ball',
    centre, radius[, stiffness]), ('ell', centre, (rx, ry, rz)[, stiffness[,
    euler]]), ('cap', a, b, radius[, stiffness]); '-ball' / '-ell' / '-cap'
    subtract. Sizes are the element's own surface."""
    _meta_n[0] += 1
    name = f'_t09mb{_meta_n[0]}x'
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
    ob = _mesh_obj('_t09dec', s)
    m = ob.modifiers.new('d', 'DECIMATE')
    m.decimate_type = 'COLLAPSE'
    m.ratio = tris / n
    m.use_collapse_triangulate = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def smooth_shape(s, factor=0.5, iterations=4):
    """Blender's smooth modifier (relaxes the metaball lumps)."""
    ob = _mesh_obj('_t09sm', s)
    m = ob.modifiers.new('s', 'SMOOTH')
    m.factor = factor
    m.iterations = iterations
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def bvh_of(s):
    return BVHTree.FromPolygons([p.copy() for p in s.v], [list(f) for f in s.f])


def to_floor(s):
    lo, _ = s.bounds()
    return s.move(0, -lo.y, 0)


def split_faces(s, keep):
    """Two shapes: the faces for which keep(centre, normal) is true, and the
    rest (each compacted)."""
    a, b = [], []
    for f in s.f:
        p = [s.v[i] for i in f]
        c = sum(p, Vector((0, 0, 0))) / len(p)
        n = (p[1] - p[0]).cross(p[2] - p[0])
        n = n.normalized() if n.length > 1e-12 else Vector((0, 1, 0))
        (a if keep(c, n) else b).append(f)
    return _compact(s, a), _compact(s, b)


def _compact(s, faces):
    used = sorted({i for f in faces for i in f})
    m = {o: k for k, o in enumerate(used)}
    return Shape([s.v[i].copy() for i in used], [tuple(m[i] for i in f) for f in faces])




def remesh(s, voxel):
    """A voxel remesh: overlapping shells fused into one surface."""
    ob = _mesh_obj('_t09rm', s)
    m = ob.modifiers.new('r', 'REMESH')
    m.mode = 'VOXEL'
    m.voxel_size = voxel
    m.adaptivity = 0.0
    m.use_smooth_shade = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return _fix(out)


def tube_frames(pts, seg=12, cap=True):
    """A tube through `pts` = [(centre, long axis, half long, half thin)]:
    each ring an ellipse across the path, its long axis given (made square
    to the path), its thin axis the third; caps closed by a fan."""
    cs = [Vector(p[0]) for p in pts]
    v, f = [], []
    n = len(pts)
    for i, (c, ax, a, b) in enumerate(pts):
        c = Vector(c)
        t = (cs[min(i + 1, n - 1)] - cs[max(i - 1, 0)]).normalized()
        L = Vector(ax)
        L = (L - t * L.dot(t)).normalized()
        T = t.cross(L).normalized()
        for j in range(seg):
            q = TAU * j / seg
            v.append(c + L * a * math.cos(q) + T * b * math.sin(q))
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    if cap:
        v.append(cs[0]); f += [(len(v) - 1, (j + 1) % seg, j) for j in range(seg)]
        v.append(cs[-1]); k = (n - 1) * seg
        f += [(len(v) - 1, k + j, k + (j + 1) % seg) for j in range(seg)]
    return propkit._fix_winding(Shape(v, f))


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


# ================================================================ skull ===
# A life-size cast resin replica of an adult skull (place_skull_stacks), its
# mandible wired on. Built in millimetres in the Frankfurt horizontal (y up
# from the plane through the ear openings and the orbits' lowest rims, z
# forward from the ear openings, x to the skull's left), then turned up
# 12.5 degrees as it sits on a floor on its jaw's lower border, put on the
# floor and given in metres.
# Its size (t09.notes.md): 188 mm glabella to the back, 140 mm across the
# parietals, 130 mm across the cheekbones, about 205 mm crown to chin.
SKULL_TILT = math.radians(-12.5)
MM = 0.001


def _mirror(elems):
    """The elements given for one side (x > 0), and their mirrors."""
    out = []
    for e in elems:
        out.append(e)
        k = e[0].lstrip('-')
        if k == 'cap':
            m = (e[0], (-e[1][0], e[1][1], e[1][2]), (-e[2][0], e[2][1], e[2][2])) + tuple(e[3:])
        else:
            m = (e[0], (-e[1][0], e[1][1], e[1][2])) + tuple(e[2:4])
            if len(e) > 4:
                rx, ry, rz = e[4]
                m = m + ((rx, -ry, -rz),)
        out.append(m)
    return out


def _mm(elems):
    """Metaball elements in millimetres to metres."""
    out = []
    for e in elems:
        k = e[0].lstrip('-')
        if k == 'cap':
            out.append((e[0], tuple(c * MM for c in e[1]), tuple(c * MM for c in e[2]), e[3] * MM) + tuple(e[4:]))
        elif k == 'ell':
            out.append((e[0], tuple(c * MM for c in e[1]), tuple(c * MM for c in e[2])) + tuple(e[3:]))
        else:
            out.append((e[0], tuple(c * MM for c in e[1]), e[2] * MM) + tuple(e[3:]))
    return out


def skull_cranium_elems(face=1.0):
    mid = [
        ('ell', (0, 48, -10), (68, 76, 92)),          # the vault
        ('ell', (0, 52, 30), (57, 60, 60)),           # the frontal, the forehead up and forward
        ('ell', (0, 26, -78), (44, 44, 28)),          # the occipital
        ('cap', (-38, 30, 81), (38, 30, 81), 7.5),    # the brow ridge
        ('ell', (0, 13, 86), (10, 15, 9)),            # the root of the nose
        ('cap', (0, 22, 91), (0, 5, 98), 4.5),        # the nasal bones
        ('ell', (0, -24, 63), (31 * face, 22, 24)),   # the maxilla
        ('ell', (0, -38, 76), (22 * face, 10, 15)),   # under the nose, to the teeth
        ('ell', (0, -32, 30), (30, 15, 30)),          # the palate
        ('ell', (0, -12, 0), (48, 26, 52)),           # the base
    ]
    side = [
        ('ell', (44, 72, -22), (24, 34, 44)),         # the parietal boss
        ('ell', (45, -3, 64), (15, 15, 13), 2.4),     # the cheekbone
        ('cap', (48, 26, 72), (47, -2, 67), 6.0),     # the orbit's lateral rim
        ('cap', (36, -2, 80), (50, 0, 70), 5.5),      # the orbit's lower rim
        ('cap', (52, -7, 56), (60, -5, 8), 4.0),      # the zygomatic arch
        ('ell', (50, -22, -12), (10, 16, 11)),        # the mastoid
        ('ell', (20, -30, 76), (8, 16, 7)),           # the canine eminence
    ]
    return _mm(mid + _mirror(side))


def _orbit_cutter(sx):
    """The socket: a rounded quadrilateral opening (38 x 33 mm, its lateral
    side lower, as an orbit is), narrowing 48 mm back toward the optic canal;
    started in front of the face"""
    rings = []
    seg = 24
    for k, (dz, sc) in enumerate([(-30, 1.08), (0, 1.0), (12, 0.86), (26, 0.6), (40, 0.28), (48, 0.08)]):
        ring = []
        for j in range(seg):
            q = TAU * j / seg
            # a superellipse, a little squarer than an ellipse
            cx, cy = math.cos(q), math.sin(q)
            ex = math.copysign(abs(cx) ** 0.75, cx) * 19.5
            ey = math.copysign(abs(cy) ** 0.75, cy) * 16.5
            ring.append(Vector((ex * sc, ey * sc, -dz)))
        rings.append(ring)
    s = loft_rings(rings)
    # its lateral side lower (12 degrees), opening forward and out
    s.rot(0, 0, -sx * math.radians(12))
    s.rot(math.radians(4), sx * math.radians(20), 0)
    return s.move(sx * 30, 16, 92)


def _nose_cutter():
    """The nasal aperture: a pear (narrow at the top, 25 mm across at its
    foot, 33 mm tall), cut 34 mm back"""
    rings = []
    seg = 24
    for dz, sc in [(-20, 1.05), (0, 1.0), (16, 0.85), (34, 0.6)]:
        ring = []
        for j in range(seg):
            q = TAU * j / seg
            y = math.sin(q) * 16.5
            w = 12.5 - 7.5 * smoothstep(-16.5, 16.5, y) ** 1.4
            ring.append(Vector((math.cos(q) * w * sc, y * sc, -dz)))
        rings.append(ring)
    return loft_rings(rings).move(0, -11, 101)


def _jaw_path(face=1.0):
    """The mandible's path from the chin round to each condyle:
    (centre, long axis, half long, half thin) in millimetres. The ramus
    stands in front of the ear's opening (z = 0), a broad plate 32 mm
    across, its back border running down to the angle"""
    right = [
        ((0, -86, 83), (0, 1, 0), 16.0, 7.0),
        ((13 * face, -85.5, 78.5), (0, 1, 0), 15.5, 6.5),
        ((24 * face, -84, 66), (0, 1, 0.05), 14.5, 6.5),
        ((31, -82.5, 50), (0, 1, 0.1), 13.0, 6.5),
        ((37, -81.5, 34), (0, 1, 0.1), 12.0, 6.0),
        ((43, -80.5, 24), (0, 1, 0.1), 11.5, 6.0),
        ((46, -78, 18), (0, 0.8, 0.6), 11.0, 5.5),
        ((47, -68, 16), (0, 0.3, 1), 14.0, 4.5),
        ((47.5, -55, 18), (0, 0, 1), 16.5, 4.0),
        ((48.5, -40, 19), (0, 0, 1), 15.5, 4.0),
        ((49.5, -27, 17), (0, 0, 1), 9.5, 4.2),
        ((50, -17, 14), (0, 0, 1), 6.0, 5.0),
    ]
    return right


def skull_jaw_mm(face=1.0):
    right = _jaw_path(face)
    left = [((-c[0], c[1], c[2]), ax, a, b) for c, ax, a, b in reversed(right[1:])]
    s = tube_frames(left + right, seg=14)
    for sx in (1, -1):
        # the condyle, across its neck, in its fossa before the ear; the
        # coronoid, a blade up under the arch
        s = s + ellipsoid(10, 5.5, 6.5, seg=14).move(sx * 50, -13, 13)
        s = s + tube_frames([((sx * 47.5, -46, 28), (0, 0, 1), 6.5, 3.0),
                             ((sx * 46.5, -34, 33), (0, 0, 1), 4.5, 2.5),
                             ((sx * 45.5, -25, 37), (0, 0, 1), 1.8, 2.0)], seg=10)
        # the chin's mental tubercles, and the ridge below the molars
        s = s + ellipsoid(7, 5, 4, seg=10).move(sx * 9, -95, 85)
    return s


def _arch_z(x, front, back, half):
    """The dental arch: a parabola from the incisors (z = front at x = 0) to
    the last molars (z = back at x = +-half)"""
    x = min(abs(x), half * 1.08)
    return front - (front - back) * (x / half) ** 2


def skull_upper_ridge_mm(face=1.0):
    """The upper alveolar ridge, holding the upper teeth"""
    arch = [((x * face, -45, _arch_z(abs(x), 86.5, 46.0, 27.0)), (0, 1, 0), 7.0, 6.0) for x in range(-28, 29, 4)]
    return tube_frames(arch, seg=10)


# the teeth from the midline back: (width, crown height, depth), mm
UPPER = [(8.5, 10.5, 7), (6.5, 9.0, 6), (7.5, 10.0, 8), (7.0, 8.0, 9), (6.5, 7.5, 9), (10.0, 7.0, 10.5), (9.0, 6.5, 10)]
LOWER = [(5.4, 9.0, 6), (5.9, 9.0, 6), (6.9, 10.0, 7.5), (7.0, 8.0, 8), (7.0, 7.5, 8), (11.0, 7.0, 10), (10.5, 6.5, 9.5)]


def _teeth(row, y0, down, front, back, missing=()):
    """One row of teeth along its arch: each crown a chamfered block off
    the gum line y0 (the upper row hangs), turned along the arch, a little
    irregular; `missing` (side, i) lost"""
    out = Shape()
    half = 27.0
    for sx in (1, -1):
        s = 0.0
        for i, (w, h, d) in enumerate(row):
            # along the arch: its arc length so far, and half this tooth
            x = s + w / 2 + 0.2
            s += w * 0.93 + 0.3
            if (sx, i) in missing:
                continue
            xx = min(x * 0.93, half)
            z = _arch_z(xx, front, back, half)
            dz = -2 * (front - back) * xx / half ** 2
            ang = math.atan2(dz, 1.0)
            hh = h * rnd(0.86, 1.0)
            t = rbox(w * 0.94, hh, d, r=min(w, d) * 0.25, seg=1)

            def shape(p, hh=hh, i=i):
                k = (p.y + hh / 2) / hh          # 0 at the edge, 1 at the gum
                p.x *= 0.80 + 0.20 * (1 - k) if i < 3 else 0.9 + 0.1 * (1 - k)
                if i < 2:
                    p.z *= 0.40 + 0.60 * k     # incisors to an edge
                return p
            t.warp(shape)
            if not down:
                t.rot(math.pi, 0, 0)
            t.rot(0, -sx * ang, 0)
            t.rot(rnd(-0.06, 0.06), 0, sx * rnd(-0.05, 0.05))
            yy = y0 - hh / 2 if down else y0 + hh / 2
            out = out + t.move(sx * xx, yy + rnd(-0.3, 0.3), z)
    return out


_SKULL = {}


def _orbit_local(p, sx):
    """`p` (mm) in the orbit cutter's own frame (its depth -z)"""
    q = p - Vector((sx * 30, 16, 92))
    q.rotate(Euler((math.radians(4), sx * math.radians(20), 0), 'XYZ').to_matrix().inverted())
    q.rotate(Euler((0, 0, -sx * math.radians(12)), 'XYZ').to_matrix().inverted())
    return q


def _in_orbit(p, sx, grow=1.12):
    q = _orbit_local(p, sx)
    d = -q.z
    prof = [(-30, 1.08), (0, 1.0), (12, 0.86), (26, 0.6), (40, 0.28), (48, 0.08)]
    if d < -30 or d > 50:
        return False, d
    sc = prof[-1][1]
    for (d0, s0), (d1, s1) in zip(prof, prof[1:]):
        if d0 <= d <= d1:
            sc = s0 + (s1 - s0) * (d - d0) / (d1 - d0)
            break
    sc = sc * grow + 1.5 / 19.5
    k = (abs(q.x) / (19.5 * sc)) ** (4 / 3) + (abs(q.y) / (16.5 * sc)) ** (4 / 3)
    return k <= 1.0, d


def _in_nose(p, grow=1.15):
    q = p - Vector((0, -11, 101))
    d = -q.z
    if d < -20 or d > 40:
        return False, d
    sc = (1.0 - 0.4 * max(0.0, d) / 34) * grow + 0.04
    if abs(q.y) > 16.5 * sc:
        return False, d
    w = (12.5 - 7.5 * smoothstep(-16.5, 16.5, q.y / sc) ** 1.4) * sc
    return abs(q.x) <= w, d


def _hollow(c, n):
    """A face of the cranium (mm, before it is turned) inside an orbit, the
    nasal aperture or an ear's opening: where the antiquing wash lies"""
    for sx in (1, -1):
        inside, d = _in_orbit(c, sx, 1.1)
        if inside and d > 4.0:
            return True
        if (c - Vector((sx * 64, -3, 2))).length < 7.0 and n.x * sx < 0.6:
            return True
    inside, d = _in_nose(c, 1.04)
    return inside and d > 4.0


def _orbit_rings(sx, d0=3.0, k=0.9):
    """The socket's rings (as `_orbit_cutter`) from `d0` mm behind its rim to
    its back, drawn in by `k`"""
    prof = [(0, 1.0), (12, 0.86), (26, 0.6), (40, 0.28), (48, 0.08)]
    sc0 = 1.0 + (0.86 - 1.0) * d0 / 12
    rings, seg = [], 24
    for dz, sc in [(d0, sc0)] + [q for q in prof if q[0] > d0]:
        ring = []
        for j in range(seg):
            q = TAU * j / seg
            cx, cy = math.cos(q), math.sin(q)
            ex = math.copysign(abs(cx) ** 0.75, cx) * 19.5
            ey = math.copysign(abs(cy) ** 0.75, cy) * 16.5
            ring.append(Vector((ex * sc * k, ey * sc * k, -dz)))
        rings.append(ring)
    out = []
    for ring in rings:
        r = Shape(ring, [])
        r.rot(0, 0, -sx * math.radians(12))
        r.rot(math.radians(4), sx * math.radians(20), 0)
        out.append([p + Vector((sx * 30, 16, 92)) for p in r.v])
    return out


def _nose_rings(d0=3.0, k=0.88):
    rings, seg = [], 24
    for dz, sc in [(d0, 1.0 - 0.15 * d0 / 16), (16, 0.85), (34, 0.6)]:
        ring = []
        for j in range(seg):
            q = TAU * j / seg
            y = math.sin(q) * 16.5
            w = 12.5 - 7.5 * smoothstep(-16.5, 16.5, y) ** 1.4
            ring.append(Vector((math.cos(q) * w * sc * k, y * sc * k, -dz)) + Vector((0, -11, 101)))
        rings.append(ring)
    return rings


def _liner(rings):
    """A lining facing in, open at its front ring, closed at its back"""
    n, seg = len(rings), len(rings[0])
    v = [p for r in rings for p in r]
    c = sum(rings[0], Vector((0, 0, 0))) / seg
    f = []
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    f.append(tuple((n - 1) * seg + j for j in range(seg)))
    s = Shape(v, f)
    # facing the axis: the face's normal toward the front ring's middle
    out = []
    for fc in s.f:
        p = [s.v[i] for i in fc]
        m = sum(p, Vector((0, 0, 0))) / len(p)
        nrm = (p[1] - p[0]).cross(p[2] - p[0])
        axis_pt = c + (rings[-1][0] * 0 + sum(rings[-1], Vector((0, 0, 0))) / seg - c) * 0.5
        out.append(fc if nrm.dot(axis_pt - m) > 0 else tuple(reversed(fc)))
    return Shape(v, out)


def skull_raw():
    """The resin (cranium with the jaw wired on), the hollows the wash
    darkened (the orbits, the nasal aperture, the ears) and the teeth, in
    metres on the floor (cached)"""
    if 'raw' in _SKULL:
        return _SKULL['raw']
    cran = meta(skull_cranium_elems(), res=0.0022)
    cran = Shape([p / MM for p in cran.v], cran.f)        # to millimetres
    cran = cran + skull_upper_ridge_mm()
    cran = remesh(cran, 1.4)
    cran = cut(cran, _orbit_cutter(1), _orbit_cutter(-1), _nose_cutter())
    for sx in (1, -1):
        # the temple's hollow behind the orbit, over the arch; the ear's opening
        cran = cut(cran, ellipsoid(16, 24, 26, seg=20).move(sx * 76, 14, 36))
        cran = cut(cran, cyl(4.2, h=24, seg=12).rot(0, 0, math.pi / 2).move(sx * 66, -3, 2))
    cran = remesh(cran, 1.1)
    cran = smooth_shape(cran, 0.5, 2)
    cran = decimate(cran, 4300)
    # the wash in the sockets and the nose: a lining just inside each, from
    # a few mm behind its rim to the back
    hollow = _liner(_orbit_rings(1)) + _liner(_orbit_rings(-1)) + _liner(_nose_rings())
    # in the nose's dark, the septum (a thin plate a little to one side) and
    # the conchae
    hollow = hollow + rbox(2.0, 22, 26, r=0.8, seg=1).move(0.6, -15, 82)
    for sx in (1, -1):
        hollow = hollow + ellipsoid(4.0, 3.5, 12, seg=10).move(sx * 6.5, -21, 80)
    jaw = remesh(skull_jaw_mm(), 1.1)
    jaw = smooth_shape(jaw, 0.5, 3)
    jaw = decimate(jaw, 1300)
    random.seed(4091)
    # the bite closed: the lower incisors' edges just behind the upper's
    teeth = _teeth(UPPER, -50.5, True, 87.0, 47.0, missing={(1, 4), (-1, 6)}) + \
        _teeth(LOWER, -69.0, False, 83.0, 46.5, missing={(-1, 1)})
    out = []
    for sh in (cran, jaw, hollow, teeth):
        out.append(sh.copy().scale(MM).rot(SKULL_TILT, 0, 0))
    lo, _ = (out[0] + out[1]).bounds()
    for sh in out:
        sh.move(0, -lo.y, 0)
    _SKULL['raw'] = out
    _SKULL['lift'] = -lo.y
    return out


def skull_placed(sh):
    """A shape built in the skull's millimetres, turned and lifted as the
    skull is"""
    skull_raw()
    return sh.copy().scale(MM).rot(SKULL_TILT, 0, 0).move(0, _SKULL['lift'], 0)


def skull_bone():
    cran, jaw, _, _ = skull_raw()
    return cran + jaw


def skull_hollows():
    return skull_raw()[2]


def skull_teeth():
    return skull_raw()[3]


auto('Real__skullStackBone', skull_bone, 5600, smooth=50.0)
auto('Real__skullStackHollows', skull_hollows, 900, smooth=50.0)
auto('Real__skullStackTeeth', skull_teeth, 1400, smooth=40.0)


# ======================================================== crystal skull ===
# A cast glass skull ornament (place_crystal_skulls), the small kind sold as
# a "crystal skull": lead-free glass, 105 mm deep, 80 mm across, 83 mm high
# (t09.notes.md). Cast from a skull simplified as such ornaments are: the
# resin skull's cranium, jaw and teeth fused into one body (the jaw shut,
# the teeth a band with the gaps between them shallow), its edges softened
# by the fire polish, its base ground flat to stand.
def _scaled(sh, c, k):
    """A cutter `sh` scaled by `k` about the point `c` (both mm)"""
    c = Vector(c)
    return Shape([(p - c) * k + c for p in sh.v], sh.f)


def crystal_skull():
    cran, jaw, hollow, teeth = skull_raw()
    # the mould fills what glass cannot hold: the mouth behind the teeth,
    # between each ramus and the base, the temples' deep hollows
    fill = ellipsoid(30, 13, 26, seg=16).move(0, -61, 62)
    for sx in (1, -1):
        fill = fill + ellipsoid(12, 30, 24, seg=14).move(sx * 42, -44, 22)
        fill = fill + ellipsoid(12, 18, 22, seg=14).move(sx * 58, 6, 32)
        fill = fill + ellipsoid(10, 12, 20, seg=14).move(sx * 50, -18, 40)
        fill = fill + ellipsoid(13, 15, 26, seg=14).move(sx * 36, -64, 38)
    s = cran + jaw + teeth + skull_placed(fill)
    s = Shape([p / MM for p in s.v], s.f)
    # its own scale: 105 mm deep, and a little broader for the mould
    lo, hi = s.bounds()
    k = 105.0 / (hi.z - lo.z)
    ks = (k * 1.04, k, k)
    s.scale(*ks)
    s = remesh(s, 1.7)
    s = smooth_shape(s, 0.6, 8)

    def orn(c):
        """a cutter in the skull's millimetres, in the ornament's"""
        c = skull_placed(c)
        return Shape([p / MM for p in c.v], c.f).scale(*ks)
    # the sockets and the nose cut again, deeper and rounder than the fused
    # casting keeps them, as the mould has them
    cutters = [orn(_scaled(_orbit_cutter(sx), (sx * 30, 16, 92), 1.06)) for sx in (1, -1)]
    cutters.append(orn(_scaled(_nose_cutter(), (0, -11, 101), 1.05)))
    # the teeth: the line between the rows, and the gaps between the teeth
    grooves = [tube([(x, -60.5, _arch_z(abs(x), 89.5, 52.0, 27.0)) for x in range(-30, 31, 3)], 1.6, seg=6)]
    xs, acc = [], 0.0
    for w, _, _ in UPPER[:6]:
        acc += w * 0.93 + 0.3
        xs.append(acc)
    for x0 in [0.0] + xs:
        for sx in ((1, -1) if x0 > 0 else (1,)):
            xx = min(x0 * 0.93, 27.0) * sx
            z = _arch_z(abs(xx), 89.5, 52.0, 27.0)
            ang = math.atan2(-2 * (89.5 - 52.0) * xx / 27.0 ** 2, 1.0)
            for y0, h in ((-55.0, 11.0), (-65.5, 10.0)):
                grooves.append(rbox(2.0, h, 4.0, r=0.6, seg=1).rot(0, -ang, 0).move(xx, y0, z + 3.5))
    cutters += [orn(gr) for gr in grooves]
    s = cut(s, *cutters)
    s = smooth_shape(s, 0.5, 2)
    lo, hi = s.bounds()
    # the base ground flat
    s.warp(lambda p: Vector((p.x, max(p.y, lo.y + 3.0), p.z)))
    s = Shape([p * MM for p in s.v], s.f)
    s = to_floor(s)
    return decimate(_fix(s), 2400)


auto('Real__crystalSkull', crystal_skull, 2500, smooth=60.0)


# ==================================================== brass candlestick ===
# A turned, cast solid brass candlestick (place_brass_candlesticks) of the
# common American kind: a round stepped foot 104 mm across, its underside
# hollow, a baluster stem with two knops, a drip pan and a socket for a 7/8
# in (22 mm) taper; 155 mm tall (t09.notes.md). Its taper burnt down: the
# wax run over the socket's rim, down its side and onto the pan; the crater
# at its top, one side of it melted through; the wick standing in it.
STICK_SOCKET_Y = 0.155          # the socket's rim
STICK_SOCKET_FLOOR = 0.133      # where the taper's foot stands in it
STICK_PROFILE = [               # (r, y) mm, from the hollow underside up
    (0, 7.0), (30, 5.0), (44, 2.2), (49, 0.6), (51.6, 0.0), (52.2, 1.4), (52.2, 3.2), (51.2, 4.6),
    (48.8, 5.4), (48.2, 7.2), (46.0, 8.4), (41.0, 9.6), (36.0, 12.2), (30.0, 16.6), (24.0, 20.4),
    (18.5, 22.6), (17.5, 24.4), (15.0, 25.6), (11.5, 27.0), (12.6, 29.0), (13.2, 31.6), (12.4, 34.0),
    (9.2, 36.2), (7.8, 40.0), (7.8, 48.0), (9.0, 55.0), (11.6, 64.0), (13.2, 72.0), (13.4, 78.0),
    (12.4, 86.0), (9.8, 94.0), (7.2, 102.0), (6.2, 108.0), (6.4, 112.0), (9.4, 114.4), (11.6, 117.4),
    (11.4, 120.6), (8.4, 123.2), (7.2, 125.0), (8.6, 127.2), (15.0, 129.4), (20.5, 131.6), (22.0, 133.2),
    (21.6, 134.6), (18.4, 135.0), (15.2, 136.2), (14.4, 139.0), (14.0, 147.0), (14.6, 152.0),
    (15.6, 154.0), (15.4, 155.0), (14.2, 155.0), (11.6, 154.2), (11.3, 152.0), (11.2, 133.0), (0, 133.0),
]


def stick_body():
    s = lathe([(r * MM, y * MM) for r, y in STICK_PROFILE], seg=36, cap_bottom=False, cap_top=False)
    return _fix(s)


def _taper_top(stub):
    """The taper's height above the socket's floor (mm) for the stub kind"""
    return {'A': 70.0, 'B': 38.0}[stub]


def stick_candle(stub):
    """The taper burnt down to `stub`'s height: 22 mm at its foot, its top
    a crater whose rim stands up where the wax cooled, melted through on
    one side (+x) where it ran; the runs down its side, over the socket's
    rim and down the socket, one onto the pan and the foot's dome"""
    top = _taper_top(stub)
    y0 = STICK_SOCKET_FLOOR / MM
    H = top
    prof = [(0, 0), (10.9, 0), (11.0, 1.0)]
    for k in range(1, 12):
        y = H * k / 12
        prof.append((11.0 - 0.9 * y / 70.0, y))
    prof += [(10.6 - 0.9 * H / 70 + 0.2, H + 0.6), (9.8, H + 1.2), (8.6, H - 0.4), (6.0, H - 3.0), (2.5, H - 4.4), (0, H - 4.6)]
    s = lathe(prof, seg=28)

    def melt(p):
        # the rim melted through toward +x, and slumped a little round
        a = math.atan2(p.z, p.x)
        near = max(0.0, math.cos(a)) ** 3
        if p.y > H - 8:
            k = smoothstep(H - 8, H + 1, p.y)
            p.y -= near * k * 4.5
            r = math.hypot(p.x, p.z)
            if r > 1e-6:
                bulge = 1 + near * k * 0.06 + 0.03 * noise.noise(Vector((p.x * 0.4, p.y * 0.4, p.z * 0.4)))
                p.x *= bulge
                p.z *= bulge
        return p
    s.warp(melt)
    runs = []

    def run(a, y_from, y_to, r0, rr, over=False):
        """A run of wax from the rim at angle a (deg) down to y_to (mm above
        the socket's floor), r0 its radius from the axis, rr its thickness"""
        a = math.radians(a)
        pts, n = [], 10
        for i in range(n + 1):
            t = i / n
            y = y_from + (y_to - y_from) * t
            r = r0
            if over and y < 22.6:          # over the socket's rim and down its side
                r = 15.2 + rr * 0.6 if y > 20 else 14.6 + rr * 0.6
            wob = 0.06 * math.sin(t * 7 + a * 3)
            pts.append(Vector((math.cos(a + wob) * r, y, math.sin(a + wob) * r)))
        tu = tube(pts, rr, seg=7, r_end=rr * 1.25)
        # the drop it ended in
        return tu + ellipsoid(rr * 1.5, rr * 2.1, rr * 1.5, seg=10).move(*pts[-1])

    if stub == 'A':
        runs.append(run(4, H - 2, 30, 11.2, 1.6))
        runs.append(run(-24, H - 5, 24, 11.1, 1.3))
        runs.append(run(30, H - 6, 9, 11.2, 1.7, over=True))
        runs.append(run(140, H - 1, 40, 10.8, 1.1))
        runs.append(run(205, H - 2, 14, 11.2, 1.4, over=True))
    else:
        runs.append(run(0, H - 3, 6, 11.3, 1.9, over=True))
        runs.append(run(-40, H - 4, 14, 11.2, 1.5, over=True))
        runs.append(run(70, H - 2, 26, 11.0, 1.2))
        runs.append(run(170, H - 2, 4, 11.2, 1.6, over=True))
        runs.append(run(250, H - 3, 18, 11.1, 1.3))
    # the wax lying in the socket's mouth round the taper
    collar = lathe([(11.0, 21.0), (13.6, 21.4), (14.6, 22.2), (14.0, 23.4), (11.6, 24.6), (10.9, 25.0)], seg=28)
    s = s + collar + sum(runs, Shape())
    # the drop that fell onto the pan, and one that ran on to the foot
    s = s + ellipsoid(4.5, 1.1, 3.4, seg=12).move(18.0, 2.0, 3.0)
    s = s + ellipsoid(5.0, 1.2, 4.0, seg=12).rot(0, 0.5, 0).move(-6.0, -(y0 - 13.0) + 0.6, 28.0)
    s = Shape([p * MM for p in s.v], s.f)
    return s.move(0, STICK_SOCKET_FLOOR, 0)


def stick_wick(stub):
    """The braided cotton wick standing out of the crater, curled over at
    its tip as a burning wick curls"""
    H = _taper_top(stub)
    pts = [Vector((0, H - 5.0, 0)), Vector((0.1, H + 1.0, 0)), Vector((0.5, H + 5.5, 0)), Vector((1.5, H + 8.5, 0.2)), Vector((2.8, H + 9.6, 0.4))]
    s = tube(pts, 0.75, seg=6, smooth=2, r_end=0.55)
    s = Shape([p * MM for p in s.v], s.f)
    return s.move(0, STICK_SOCKET_FLOOR, 0)


auto('Real__brassStickBody', stick_body, 4000, smooth=40.0, kind='cyl')
auto('Real__brassStickCandleA', lambda: stick_candle('A'), 3000, smooth=45.0, kind='cyl')
auto('Real__brassStickCandleB', lambda: stick_candle('B'), 3000, smooth=45.0, kind='cyl')
auto('Real__brassStickWickA', lambda: stick_wick('A'), 260, smooth=60.0)
auto('Real__brassStickWickB', lambda: stick_wick('B'), 260, smooth=60.0)


def stick_flame(stub):
    """The flame over the wick (lit sticks only): a candle's teardrop, 11 mm
    across at its widest, 27 mm from its blue root to its tip; its root just
    above the crater, round the wick's tip"""
    H = _taper_top(stub)
    prof = [(0, 0.0), (1.8, 0.4), (3.8, 2.0), (5.2, 5.0), (5.5, 8.0), (5.0, 12.0), (3.8, 16.5), (2.3, 21.0), (1.0, 24.6), (0, 27.0)]
    s = lathe([(r * MM, (y + H - 1.0) * MM) for r, y in prof], seg=16)
    return s.move(0, STICK_SOCKET_FLOOR, 0)


auto('Real__brassStickFlameA', lambda: stick_flame('A'), 300, smooth=80.0, kind='cyl')
auto('Real__brassStickFlameB', lambda: stick_flame('B'), 300, smooth=80.0, kind='cyl')


# ================================================================== owl ===
# A carved wooden owl (place_small_owls), the souvenir kind carved from one
# block of lime or walnut and stained dark: a horned owl standing on a rock
# carved with it, its feet gripping the rock's front, 15 cm to the tips of
# its ear tufts (the Black Forest owl of t09.notes.md is 17 cm), set with
# glass eyes (amber, the pupil painted on the back). The feathers' chisel
# cuts are painted in the game (the stain lying in them).
def owl_wood():
    el = _mm([
        ('ell', (0, 58, -2), (34, 40, 30)),             # the body
        ('ell', (0, 70, 8), (30, 30, 26)),              # the breast
        ('ell', (0, 104, 3), (33, 25, 27)),             # the head, broad
        ('ell', (0, 93, 0), (31, 14, 25)),              # no neck: the head sits on the body
        ('ell', (0, 40, -24), (14, 14, 8), 2.0, (-0.6, 0, 0)),     # the tail's root
        ('cap', (20, 112, 6), (23, 129, 2), 5.0),      # the ear tufts
        ('cap', (-20, 112, 6), (-23, 129, 2), 5.0),
        ('ell', (0, 98, 26), (5, 8, 6)),                # the beak's root
    ])
    s = meta(el, res=0.0018)
    s = Shape([p / MM for p in s.v], s.f)
    # the facial disc: two shallow dishes round the eyes, the ruff between
    dishes = [ellipsoid(15, 15, 9, seg=20).move(sx * 12.5, 104, 33) for sx in (1, -1)]
    s = cut(s, *dishes)
    # the folded wings: each a carved shell over the side, its edge standing
    # off the body; their tips crossed over the tail
    wings = Shape()
    for sx in (1, -1):
        w = ellipsoid(9, 38, 27, seg=20)
        w.warp(lambda p: Vector((p.x, p.y, p.z + 0.004 * p.y * p.y)))
        w.rot(0.18, 0, -sx * 0.10).move(sx * 29, 64, -5)
        wings = wings + w
        tip = ellipsoid(6, 24, 3.5, seg=12).rot(-0.45, sx * 0.25, 0).move(sx * 6, 38, -31)
        wings = wings + tip
    # the tail, a flat wedge of feathers down onto the rock
    tail = rbox(26, 30, 6, r=2.5, seg=2)
    tail.warp(lambda p: Vector((p.x * (1.0 - 0.006 * p.y), p.y, p.z)))
    tail.rot(-0.95, 0, 0).move(0, 26, -36)
    # the beak: a hooked point down out of the disc
    beak = tube([(0, 103, 25), (0, 100.5, 32), (0, 96.5, 35.5), (0, 92.5, 35.0), (0, 90.5, 33.2)], 4.2, seg=10, smooth=2, r_end=0.7)
    beak.warp(lambda p: Vector((p.x * 0.8, p.y, p.z)))
    # the feet: three toes forward over the rock's edge, talons into it
    feet = Shape()
    for sx in (1, -1):
        for k in (-1, 0, 1):
            a = k * 0.32
            x0 = sx * 13 + math.sin(a) * 2
            pts = [(x0, 27, 10), (x0 + math.sin(a) * 6, 24.5, 15 + math.cos(a) * 3), (x0 + math.sin(a) * 9, 21.5, 19 + math.cos(a) * 4),
                   (x0 + math.sin(a) * 10, 17.5, 21 + math.cos(a) * 4)]
            feet = feet + tube(pts, 2.6, seg=8, smooth=1, r_end=1.0)
        feet = feet + ellipsoid(8, 6, 8, seg=12).move(sx * 13, 28, 8)
    # the rock: a block carved rough, its faces chiselled flat
    rock = rbox(96, 22, 74, r=6, seg=2).move(0, 11, -2)
    rock = remesh(rock, 2.0)
    rock = noisy(rock, 3.4, scale=0.05, seed=3)
    rock.warp(lambda p: Vector((p.x, max(0.0, min(p.y, 21 + 2.5 * noise.noise(Vector((p.x * 0.05, 3.0, p.z * 0.05))))), p.z)))
    s = remesh(s + wings + tail + beak + feet + rock, 1.0)
    s = smooth_shape(s, 0.5, 2)
    # the chisel's facets on the rock and the body: a little flat-planed
    s = Shape([p * MM for p in s.v], s.f)
    s = to_floor(s)
    return decimate(_fix(s), 5600)


OWL_EYE = (12.5, 104.5, 25.5, 7.5)       # x, y, z of the glass's centre, its radius (mm)


def owl_eyes():
    x, y, z, r = OWL_EYE
    out = Shape()
    for sx in (1, -1):
        out = out + sphere(r, seg=16).move(sx * x, y, z)
    return to_floor_like_owl(out)


def owl_pupils():
    x, y, z, r = OWL_EYE
    out = Shape()
    for sx in (1, -1):
        # painted on the glass's back, seen through it: a disc just under the
        # front of the glass, looking a little in toward the beak
        d = cyl(3.4, h=0.8, seg=12, bevel_r=0.25).rot(math.pi / 2, 0, 0)
        out = out + d.move(sx * (x - 0.6), y, z + r - 0.35)
    return to_floor_like_owl(out)


def to_floor_like_owl(s):
    """mm to metres, lifted as the wood was (its rock's foot at y = 0)"""
    return Shape([p * MM for p in s.v], s.f)


auto('Real__owlWood', owl_wood, 5800, smooth=50.0)
auto('Real__owlEyes', owl_eyes, 700, smooth=70.0)
auto('Real__owlPupils', owl_pupils, 300, smooth=40.0)


# ================================================================= mask ===
# A Venetian volto (place_small_masks): a full-face carnival mask of
# papier-mache, gessoed and painted, the eyes cut through, gilt round the
# eyes and the edge, tied on by two satin ribbons from the temples; 15 cm
# across, 21.5 cm brow to chin (t09.notes.md). Lying on its back, face up,
# on its side rims and its brow's edge, the ribbons out across the floor.
# Modelled as a face (X across, Y up the face, Z out of it, mm), then laid
# down: Z up (y), Y toward -z (its brow at -z).
MASK_W, MASK_TOP, MASK_CHIN = 75.0, 106.0, -110.0
MASK_EYES = ((32.0, 22.0), 16.0, 7.4)       # centre (+-x, y), half width, half height
MASK_TILT = math.radians(4.0)


def _g(x, y, cx, cy, sx, sy):
    return math.exp(-(((x - cx) / sx) ** 2 + ((y - cy) / sy) ** 2))


def mask_height(x, y):
    """The face's relief: Z (mm) at (X, Y)"""
    ax = abs(x)
    z = -(x * x) / (2 * 64.0) - (y * y) / (2 * 260.0)
    z += 4.5 * _g(x, y, 0, 72, 42, 30)                                  # the forehead
    z += 5.0 * math.exp(-((y - 37) / 9.0) ** 2) * math.exp(-(x / 48.0) ** 4)   # the brow
    z -= 7.0 * _g(ax, y, 32, 21, 19, 11)                                # the eye sockets
    z += 7.5 * _g(ax, y, 38, -10, 19, 20)                               # the cheekbones
    # the nose: its bridge from the brow, its tip, its wings
    t = smoothstep(32, -16, y) * (1 - smoothstep(-16, -27, y))
    w = 5.5 + 7.0 * smoothstep(32, -16, y)
    z += (3.0 + 21.0 * t ** 1.3) * math.exp(-(x / w) ** 2) * (1 - smoothstep(-22, -30, y) * 0.6) * (1 - smoothstep(30, 44, y))
    z += 6.5 * _g(ax, y, 11, -21, 6, 6)
    z += 5.5 * _g(x, y, 0, -44, 20, 5.5)                                # the upper lip
    z += 5.0 * _g(x, y, 0, -55.5, 17, 5.0)                              # the lower lip
    z -= 2.4 * math.exp(-((y + 49.6) / 1.3) ** 2) * math.exp(-(x / 19.0) ** 4)  # the mouth, closed
    z += 8.0 * _g(x, y, 0, -82, 19, 14)                                 # the chin
    return z


def _mask_outline(k=1.0):
    """The volto's outline (X, Y), a little square at the brow, narrower at the
    chin"""
    pts = []
    for j in range(72):
        q = TAU * j / 72
        c, sn = math.cos(q), math.sin(q)
        if sn >= 0:
            e = 2.6
            y = MASK_TOP * abs(sn) ** (2 / e)
        else:
            e = 1.8
            y = MASK_CHIN * abs(sn) ** (2 / e)
        x = MASK_W * math.copysign(abs(c) ** (2 / (2.6 if sn >= 0 else 2.3)), c)
        if sn < 0:
            x *= 1.0 - 0.25 * abs(sn) ** 2
        pts.append((x * k, y * k))
    return pts


def _eye_outline(sx):
    (cx, cy), hw, hh = MASK_EYES
    pts = []
    for j in range(28):
        q = TAU * j / 28
        x = math.cos(q) * hw
        # an almond: its inner corner lower and rounder, the outer drawn up
        y = math.sin(q) * hh * (1 - 0.35 * (x / hw) ** 2) + 2.2 * (x / hw) * sx * 0.5 * (1 + math.cos(q))
        pts.append((sx * x + sx * cx, y + cy))
    return pts


def _solidify(s, t, offset=-1.0):
    ob = _mesh_obj('_t09so', s)
    m = ob.modifiers.new('so', 'SOLIDIFY')
    m.thickness = t
    m.offset = offset
    m.use_even_offset = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


_MASK = {}


def mask_face_mm():
    """The shell in the face's own frame (mm)"""
    if 'face' in _MASK:
        return _MASK['face']
    st = 1.6
    nx, ny = int(170 / st), int(240 / st)
    v, f = [], []
    for j in range(ny + 1):
        y = -120 + j * st
        for i in range(nx + 1):
            x = -85 + i * st
            v.append(Vector((x, y, mask_height(x, y))))
    for j in range(ny):
        for i in range(nx):
            a = j * (nx + 1) + i
            f.append((a, a + 1, a + nx + 2, a + nx + 1))
    sheet = Shape(v, f)
    shell = _solidify(sheet, 2.6, -1.0)
    shell = _fix(shell)
    keep = prism(_mask_outline(), 200)
    shell = boolean(shell, keep, 'INTERSECT')
    shell = cut(shell, *[prism(_eye_outline(sx), 200) for sx in (1, -1)])
    # the rim rolled a little: the edge is the paper's turned edge
    _MASK['face'] = shell
    return shell


def _mask_lay(s):
    """mm face frame -> metres, lying face up, tilted onto its brow's edge"""
    s = s.copy().rot(-math.pi / 2, 0, 0).rot(-MASK_TILT, 0, 0)
    return Shape([p * MM for p in s.v], s.f)


def _mask_lift():
    lo, _ = _mask_lay(mask_face_mm()).bounds()
    return -lo.y


def mask_face():
    return decimate(_mask_lay(mask_face_mm()).move(0, _mask_lift(), 0), 4200)


def mask_ribbons():
    """Two satin ribbons (10 mm, a hole at each temple) from the mask's side
    rims out across the floor, lying loose: one long and curled back, one
    short"""
    lift = _mask_lift()
    out = Shape()
    for sx, path in ((1, [(0, 0), (30, -10), (70, 5), (110, 40), (135, 90), (128, 150), (100, 175)]),
                     (-1, [(0, 0), (35, 8), (80, 2), (120, -30), (160, -36), (205, -20)])):
        # where it is tied: the temple's edge, X = +-74, Y = 22
        a = _mask_lay(Shape([Vector((sx * 72.5, 22.0, mask_height(sx * 72.5, 22.0) - 1.3))], [])).move(0, lift, 0).v[0] / MM
        pts = []
        # down off the rim to the floor
        pts.append(((a.x, a.y, a.z), (0, 0, 1), 5.0, 0.35))
        pts.append(((a.x + sx * 6, a.y * 0.45, a.z + 2), (0, 0, 1), 5.0, 0.35))
        floor = [Vector((a.x + sx * (dx + 12), 0.45, a.z + 4 + dz)) for dx, dz in path]
        for _ in range(2):           # the turns soft (Chaikin)
            q = [floor[0]]
            for p0, p1 in zip(floor, floor[1:]):
                q += [p0.lerp(p1, 0.25), p0.lerp(p1, 0.75)]
            floor = q + [floor[-1]]
        for k, p in enumerate(floor):
            nxt = floor[min(k + 1, len(floor) - 1)] - floor[max(k - 1, 0)]
            side = Vector((-nxt.z, 0, nxt.x)).normalized()
            # a ribbon lying flat, a twist or two where it turned over
            tw = 0.25 * math.sin(k * 0.21 + sx)
            ax = side * math.cos(tw) + Vector((0, 1, 0)) * math.sin(tw)
            pts.append(((p.x, p.y + 0.6 * abs(math.sin(tw)) * 5, p.z), tuple(ax), 5.0, 0.35))
        r = tube_frames(pts, seg=6)
        out = out + Shape([p * MM for p in r.v], r.f)
    return out


auto('Real__maskFace', mask_face, 4400, smooth=40.0)
auto('Real__maskRibbons', mask_ribbons, 800, smooth=60.0)
