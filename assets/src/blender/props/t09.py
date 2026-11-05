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
    (centre, long axis, half long, half thin) in millimetres"""
    right = [
        ((0, -86, 83), (0, 1, 0), 16.0, 7.0),
        ((13 * face, -85.5, 78.5), (0, 1, 0), 15.5, 6.5),
        ((24 * face, -84, 66), (0, 1, 0.05), 14.5, 6.5),
        ((31, -82.5, 50), (0, 1, 0.1), 13.0, 6.5),
        ((37, -81.5, 32), (0, 1, 0.1), 12.0, 6.0),
        ((43, -80.5, 18), (0, 1, 0.1), 11.5, 6.0),
        ((46, -79, 12), (0, 1, 0.3), 11.0, 5.5),
        ((47, -70, 4), (0, 0.4, 1), 12.0, 5.0),
        ((47.5, -55, 6), (0, 0, 1), 15.0, 4.5),
        ((48.5, -38, 8), (0, 0, 1), 14.0, 4.5),
        ((49.5, -25, 7), (0, 0, 1), 9.0, 4.5),
        ((50, -15, 4), (0, 0, 1), 6.0, 5.5),
    ]
    return right


def skull_jaw_mm(face=1.0):
    right = _jaw_path(face)
    left = [((-c[0], c[1], c[2]), ax, a, b) for c, ax, a, b in reversed(right[1:])]
    s = tube_frames(left + right, seg=14)
    for sx in (1, -1):
        # the condyle, across its neck; the coronoid, a blade up under the arch
        s = s + ellipsoid(10, 5.5, 6.5, seg=14).move(sx * 50, -12, 3)
        s = s + tube_frames([((sx * 47.5, -44, 16), (0, 0, 1), 6.0, 2.5),
                             ((sx * 46.5, -32, 22), (0, 0, 1), 4.0, 2.2),
                             ((sx * 45.5, -24, 26), (0, 0, 1), 1.5, 1.8)], seg=10)
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


def skull_raw():
    """The resin (cranium, nasal septum, the jaw, wired on) and the teeth, in
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
    # the septum, a thin plate down the nasal cavity, and the conchae
    cran = cran + rbox(2.0, 22, 24, r=0.8, seg=1).move(0.6, -15, 76)
    for sx in (1, -1):
        cran = cran + ellipsoid(4.5, 3.5, 12, seg=10).move(sx * 7, -21, 78)
    cran = remesh(cran, 1.1)
    cran = smooth_shape(cran, 0.5, 2)
    jaw = remesh(skull_jaw_mm(), 1.1)
    jaw = smooth_shape(jaw, 0.5, 3)
    random.seed(4091)
    teeth = _teeth(UPPER, -50.5, True, 87.0, 47.0, missing={(1, 4), (-1, 6)}) + \
        _teeth(LOWER, -71.0, False, 83.5, 46.5, missing={(-1, 1)})
    out = []
    for s in (cran, jaw, teeth):
        out.append(s.copy().scale(MM).rot(SKULL_TILT, 0, 0))
    lo, _ = (out[0] + out[1]).bounds()
    for s in out:
        s.move(0, -lo.y, 0)
    _SKULL['raw'] = out
    return out


def skull_bone():
    cran, jaw, _ = skull_raw()
    return decimate(cran, 3600) + decimate(jaw, 1100)


def skull_teeth():
    return skull_raw()[2]


auto('Real__skullStackBone', skull_bone, 4800, smooth=50.0)
auto('Real__skullStackTeeth', skull_teeth, 1400, smooth=40.0)


# ======================================================== crystal skull ===
# A cast glass skull ornament (place_crystal_skulls), the small kind sold as
# a "crystal skull": lead-free glass, 105 mm deep, 80 mm across, 83 mm high
# (t09.notes.md), the skull simplified as such ornaments are: the jaw fused
# to it and short, the teeth a band with the gaps between them cut, polished
# smooth all over, its base ground flat to stand.
def crystal_skull():
    el = _mm([
        ('ell', (0, 50, -9), (36, 33, 45)),           # the vault
        ('ell', (0, 47, 12), (32, 31, 34)),           # the forehead
        ('cap', (-20, 41, 41), (20, 41, 41), 5.0),    # the brow
        ('ell', (0, 27, 35), (26, 17, 17)),           # the face
        ('ell', (24, 25, 38), (9, 9, 9)), ('ell', (-24, 25, 38), (9, 9, 9)),   # the cheekbones
        ('cap', (28, 22, 34), (35, 22, 8), 3.5), ('cap', (-28, 22, 34), (-35, 22, 8), 3.5),  # the arches
        ('ell', (0, 15, 39), (17, 9, 11)),            # the upper jaw, to the teeth
        ('ell', (0, 6, 36), (18, 7, 12)),             # the jaw, short
        ('ell', (0, 16, -14), (31, 17, 33)),          # the base
    ])
    s = meta(el, res=0.0016)
    s = Shape([p / MM for p in s.v], s.f)
    cutters = []
    for sx in (1, -1):
        o = _orbit_cutter(sx)
        o = Shape([(p - Vector((sx * 30, 16, 92))) * 0.58 + Vector((sx * 15.5, 35, 47)) for p in o.v], o.f)
        cutters.append(o)
    nose = _nose_cutter()
    nose = Shape([(p - Vector((0, -11, 101))) * 0.5 + Vector((0, 22, 49)) for p in nose.v], nose.f)
    cutters.append(nose)
    for sx in (1, -1):
        cutters.append(ellipsoid(8, 12, 14, seg=16).move(sx * 43, 33, 22))   # the temple
    s = cut(s, *cutters)
    s = remesh(s, 0.7)
    s = smooth_shape(s, 0.5, 3)
    # the teeth: the line between the rows, and the gaps between the teeth,
    # cut after the polish
    teeth = [rbox(32, 2.0, 30, r=0.6, seg=1).move(0, 11.0, 43)]
    for i in range(-5, 6):
        x = i * 3.3
        z = 49 - 0.035 * x * x
        teeth.append(box(1.4, 9.5, 8).rot(0, math.atan(0.07 * x), 0).move(x, 11.0, z))
    s = cut(s, *teeth)
    lo, hi = s.bounds()
    # the base ground flat
    s.warp(lambda p: Vector((p.x, max(p.y, lo.y + 2.0), p.z)))
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
    s = lathe([(r * MM, y * MM) for r, y in STICK_PROFILE], seg=40, cap_bottom=False, cap_top=False)
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


auto('Real__brassStickBody', stick_body, 2400, smooth=40.0, kind='cyl')
auto('Real__brassStickCandleA', lambda: stick_candle('A'), 2200, smooth=45.0, kind='cyl')
auto('Real__brassStickCandleB', lambda: stick_candle('B'), 2200, smooth=45.0, kind='cyl')
auto('Real__brassStickWickA', lambda: stick_wick('A'), 200, smooth=60.0)
auto('Real__brassStickWickB', lambda: stick_wick('B'), 200, smooth=60.0)
