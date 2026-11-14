"""t15: toys left on the floor, remodelled at their real sizes (docs/design/PROPS.md,
P3, the rare): the action figure, the two toy dinosaurs (a stegosaurus and a
brontosaurus), the small tyrannosaurus, the spinning top, the fairy wand,
the baby rattle, the teething ring, a dog's squeaky bone and rubber ball,
and the toy microphone. Sources and sizes: docs/research/props-remodelling/toys_left_on_the_floor.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame: x across its front, y up from the floor (y = 0 the
floor it rests on), z out of its front, unless its section says otherwise.
What lies on the floor is built lying (the game turns it only about y).
Dimensions in the code are millimetres, scaled to metres as each part is
finished (`mm`).
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


def mm(s):
    """A shape built in millimetres, in metres."""
    return s.scale(0.001)


# ------------------------------------------------------------- helpers ---
_meta_n = [0]
MB_K = 1.0 / 0.564


def meta(elems, res=1.0, threshold=0.6, unit=1.0):
    """A blobby surface from metaball elements (r03's, t08's), in the units
    given: ('ball', centre, radius[, stiffness]), ('ell', centre, (rx, ry,
    rz)[, stiffness[, euler]]), ('cap', a, b, radius[, stiffness]); '-ball' /
    '-ell' / '-cap' subtract. Sizes are the element's own surface. Blender
    holds an element's size under 20 (a capsule longer than 40 is cut
    short), so a long shape is built in `unit`s and scaled back."""
    k = 1.0 / unit
    _meta_n[0] += 1
    name = f'_t15mb{_meta_n[0]}x'
    mb = bpy.data.metaballs.new(name)
    mb.resolution = res * k
    mb.render_resolution = res * k
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
            el.co = Vector(e[1]) * k
            el.radius = e[2] * MB_K * k
            el.stiffness = e[3] if len(e) > 3 else 2.0
        elif kind == 'ell':
            el.type = 'ELLIPSOID'
            el.co = Vector(e[1]) * k
            el.radius = MB_K
            el.size_x, el.size_y, el.size_z = (x * k for x in e[2])
            el.stiffness = e[3] if len(e) > 3 else 2.0
            if len(e) > 4:
                el.rotation = Euler(e[4], 'XYZ').to_quaternion()
        elif kind == 'cap':
            a, b = Vector(e[1]) * k, Vector(e[2]) * k
            el.type = 'CAPSULE'
            el.co = (a + b) / 2
            el.radius = e[3] * MB_K * k
            el.size_x = max(1e-5, (b - a).length / 2)
            el.stiffness = e[4] if len(e) > 4 else 2.0
            el.rotation = Vector((1, 0, 0)).rotation_difference((b - a).normalized())
        else:
            raise ValueError(kind)
    s = _eval_to_shape(ob)
    bpy.data.objects.remove(ob)
    bpy.data.metaballs.remove(mb)
    return _fix(s.scale(unit) if unit != 1.0 else s)


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
    ob = _mesh_obj('_t15dec', s)
    m = ob.modifiers.new('d', 'DECIMATE')
    m.decimate_type = 'COLLAPSE'
    m.ratio = tris / n
    m.use_collapse_triangulate = True
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def smooth_shape(s, factor=0.5, iterations=4):
    """Blender's smooth modifier (relaxes the metaball lumps)."""
    ob = _mesh_obj('_t15sm', s)
    m = ob.modifiers.new('s', 'SMOOTH')
    m.factor = factor
    m.iterations = iterations
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def to_floor(s):
    lo, _ = s.bounds()
    return s.move(0, -lo.y, 0)


def bvh_of(s):
    return BVHTree.FromPolygons([p.copy() for p in s.v], [list(f) for f in s.f])


def superellipse(c, ex, ey, a, b, p=2.0, n=16, phase=0.0):
    """n points round centre c in the plane (ex, ey): |x/a|^p + |y/b|^p = 1"""
    c, ex, ey = Vector(c), Vector(ex), Vector(ey)
    out = []
    for i in range(n):
        t = TAU * i / n + phase
        ct, st = math.cos(t), math.sin(t)
        x = a * math.copysign(abs(ct) ** (2.0 / p), ct)
        y = b * math.copysign(abs(st) ** (2.0 / p), st)
        out.append(c + ex * x + ey * y)
    return out


def loft(rings, cap=True):
    """Close a stack of equal rings (lists of points) into a solid, the
    ends closed by a fan to each end ring's centre."""
    n = len(rings[0])
    v, f = [], []
    for r in rings:
        v += [Vector(p) for p in r]
    for i in range(len(rings) - 1):
        for j in range(n):
            j2 = (j + 1) % n
            f.append((i * n + j, i * n + j2, (i + 1) * n + j2, (i + 1) * n + j))
    if cap:
        c0 = sum((Vector(p) for p in rings[0]), Vector()) / n
        c1 = sum((Vector(p) for p in rings[-1]), Vector()) / n
        v.append(c0)
        f += [(len(v) - 1, (j + 1) % n, j) for j in range(n)]
        v.append(c1)
        k = (len(rings) - 1) * n
        f += [(len(v) - 1, k + j, k + (j + 1) % n) for j in range(n)]
    return propkit._fix_winding(Shape(v, f))


def loft_y(secs, n=16, p=2.4, x0=0.0):
    """A solid along +y: secs = [(y, half across, half deep, z offset[, p])],
    each a superellipse in the xz plane."""
    rings = []
    for sec in secs:
        y, a, b, z = sec[:4]
        pp = sec[4] if len(sec) > 4 else p
        rings.append(superellipse((x0, y, z), (1, 0, 0), (0, 0, 1), a, b, pp, n))
    return loft(rings)


def loft_path(secs, n=14, p=2.2, up=(0, 0, 1)):
    """A limb along a path: secs = [(centre, half across, half deep[, p])],
    'across' square to the path and to `up`, 'deep' along `up` (made square
    to the path)."""
    cs = [Vector(s[0]) for s in secs]
    rings = []
    m = len(secs)
    for i, sec in enumerate(secs):
        t = (cs[min(i + 1, m - 1)] - cs[max(i - 1, 0)]).normalized()
        u = Vector(up)
        u = (u - t * u.dot(t)).normalized()
        x = t.cross(u).normalized()
        pp = sec[3] if len(sec) > 3 else p
        rings.append(superellipse(cs[i], x, u, sec[1], sec[2], pp, n))
    return loft(rings)


def lay_on(bvh, s, origin, direction, off=0.05):
    """Move s (built about the origin, its 'up' +z) to where a ray from
    `origin` along `direction` meets the surface, turned to its normal and
    lifted `off` off it."""
    hit, nrm, _, _ = bvh.ray_cast(Vector(origin), Vector(direction).normalized(), 200.0)
    if hit is None:
        return Shape()
    q = Vector((0, 0, 1)).rotation_difference(nrm.normalized())
    out = s.copy().apply(q.to_matrix().to_4x4())
    return out.move(*(hit + nrm.normalized() * off))


def rest_angle_x(parts, lo_sel, hi_sel, a0, a1, steps=200):
    """The turn about x (between a0 and a1) at which the lowest point of the
    vertices `lo_sel` picks and the lowest of `hi_sel` lie level (a body
    resting on two places at once)."""
    best, ang = 1e9, a0
    verts = [p for s in parts for p in s.v]
    for k in range(steps + 1):
        a = a0 + (a1 - a0) * k / steps
        ca, sa = math.cos(a), math.sin(a)
        y1 = y2 = 1e9
        for p in verts:
            y = p.y * ca - p.z * sa
            z = p.y * sa + p.z * ca
            if lo_sel(p):
                y1 = min(y1, y)
            if hi_sel(p):
                y2 = min(y2, y)
        if abs(y1 - y2) < best:
            best, ang = abs(y1 - y2), a
    return ang


# ======================================================= action figure ===
# A jointed 3 3/4 in (95 mm) soldier of the 1980s kind (no maker's): an
# O-ring body moulded in the uniform's coloured plastic (the waist, the
# T-hip, swivel knees and swivel arms bent in a battle grip), the head and
# hands in flesh plastic, the boots, belt and pouches painted black, a
# steel-pot helmet painted over, the eyes dots of paint. Standing on its
# boots, or (the 'Lie' parts) fallen on its back.
_fig = {}


def _fig_head():
    head = loft_y([(79.6, 2.6, 2.8, 0.6), (80.4, 4.0, 4.4, 1.0), (81.6, 4.7, 5.3, 1.2),
                   (83.6, 5.3, 5.8, 0.8), (86.4, 5.6, 6.1, 0.4), (89.4, 5.6, 6.1, 0.2),
                   (91.8, 4.9, 5.4, 0.0), (93.4, 3.2, 3.6, -0.1), (94.0, 1.0, 1.1, -0.1)], n=20, p=2.2)
    nose = ellipsoid(0.75, 1.5, 0.9, seg=10).move(0, 86.1, 6.15)
    brow = ellipsoid(4.0, 0.6, 0.7, seg=12).move(0, 88.3, 5.85)
    chin = ellipsoid(2.4, 1.3, 1.3, seg=10).move(0, 81.0, 4.6)
    ears = sum(ellipsoid(0.6, 1.6, 1.1, seg=8).move(s * 5.5, 86.6, -0.2) for s in (-1, 1))
    neck = cyl(2.8, 3.0, 5.4, seg=14, bevel_r=0).move(0, 78.4, 0.2)
    return head + nose + brow + chin + ears + neck


def _fig_helmet():
    prof = [(6.75, -0.15), (7.35, 0.0), (7.45, 0.45), (7.15, 0.9), (7.0, 2.4), (6.45, 4.4),
            (5.3, 6.0), (3.4, 7.0), (0.0, 7.45)]
    dome = lathe(prof, seg=24, cap_bottom=True, cap_top=False)
    return dome.scale(0.98, 1.0, 1.08).rot(math.radians(-11), 0, 0).move(0, 87.9, -0.3)


def _fig_hand(s, wrist, fwd):
    """A battle-grip fist at the wrist: the fingers curled round a hole
    for a weapon's grip."""
    f = Vector(fwd).normalized()
    fist = rbox(3.4, 4.6, 4.2, r=1.2, seg=2)
    thumb = capsule(0.9, 2.0, seg=8, cap_rings=3).rot(0, 0, math.pi / 2).move(-s * 0.4, 1.4, 1.6)
    h = fist + thumb
    h = cut(h, cyl(1.15, h=8.0, seg=10, bevel_r=0).move(0, 0, 0.6))
    q = Vector((0, -1, 0)).rotation_difference(f)
    h.apply(q.to_matrix().to_4x4())
    return h.move(*(Vector(wrist) + f * 2.6))


def figure_parts():
    """The figure standing, in mm: {'body', 'skin', 'kit', 'helmet', 'eyes'}"""
    if _fig:
        return _fig
    torso = loft_y([(50.8, 6.4, 4.6, 0.0), (51.2, 7.3, 5.2, 0.0), (55.0, 7.8, 5.6, 0.1),
                    (61.0, 9.3, 6.2, 0.3), (67.0, 10.6, 6.5, 0.4), (71.6, 11.0, 6.0, 0.1),
                    (74.6, 9.6, 5.4, -0.1), (76.6, 6.6, 4.5, 0.0), (77.6, 3.6, 3.0, 0.1)], n=20, p=2.6)
    pelvis = loft_y([(41.6, 4.6, 3.6, 0.0), (42.8, 8.0, 5.0, 0.0), (45.0, 9.0, 5.7, 0.0),
                     (48.0, 8.6, 5.8, 0.0), (50.2, 7.4, 5.3, 0.0), (50.6, 6.6, 4.7, 0.0)], n=20, p=2.6)
    collar = loft_y([(75.6, 7.4, 5.4, 0.0), (76.0, 7.6, 5.6, 0.1), (77.3, 5.2, 4.4, 0.4),
                     (77.6, 4.4, 3.8, 0.4)], n=16, p=2.4)
    pockets = Shape()
    for s in (-1, 1):
        pk = rbox(4.6, 4.4, 1.2, r=0.4, seg=1).move(s * 4.8, 66.0, 5.95)
        flap = rbox(5.0, 1.4, 1.0, r=0.35, seg=1).move(s * 4.8, 68.5, 6.4)
        pockets = pockets + pk + flap
    body = torso + pelvis + collar + pockets
    kit = Shape()
    skin = _fig_head()
    for s in (-1, 1):
        # the leg: thigh (the T-hip's tab rounded into the pelvis), a knee cut, the shin
        thigh = loft_path([((s * 5.0, 48.2, 0.0), 2.4, 2.8), ((s * 5.1, 46.8, 0.0), 4.4, 5.2),
                           ((s * 5.2, 43.5, 0.2), 4.7, 5.5), ((s * 5.4, 36.0, 0.3), 4.2, 5.0),
                           ((s * 5.6, 28.6, 0.4), 3.7, 4.3), ((s * 5.6, 27.9, 0.4), 3.2, 3.8)], n=16, p=2.3)
        cargo = rbox(1.4, 5.0, 4.6, r=0.45, seg=1).move(s * 9.3, 37.5, 0.4)
        shin = loft_path([((s * 5.7, 27.5, 0.5), 3.1, 3.7), ((s * 5.75, 26.8, 0.5), 3.55, 4.25),
                          ((s * 5.9, 22.0, 0.0), 3.4, 4.3), ((s * 6.0, 16.0, -0.2), 3.0, 3.6),
                          ((s * 6.1, 10.0, -0.1), 2.85, 3.3)], n=16, p=2.3)
        body = body + thigh + cargo + shin
        # the boot: its shaft and the foot (toe out), a sole
        shaft = loft_y([(5.0, 3.3, 3.7, -0.4), (9.0, 3.25, 3.65, -0.3), (12.6, 3.35, 3.8, -0.2),
                        (13.2, 3.1, 3.5, -0.2)], n=16, p=2.3, x0=s * 6.1)
        foot = loft([superellipse((0, y, z), (1, 0, 0), (0, 1, 0), a, b, 2.4, 16) for (z, y, a, b) in
                     [(-5.0, 3.0, 2.6, 2.6), (-4.2, 3.4, 3.15, 3.35), (-1.0, 3.6, 3.3, 3.6),
                      (2.5, 3.0, 3.35, 3.0), (6.0, 2.4, 3.2, 2.4), (9.0, 1.9, 2.8, 1.9),
                      (10.4, 1.7, 2.0, 1.55), (10.9, 1.6, 1.0, 0.9)]])
        foot = foot.warp(lambda p: Vector((p.x, max(p.y, 0.0), p.z)))
        foot.rot(0, s * math.radians(7), 0).move(s * 6.2, 0, 0.3)
        kit = kit + shaft + foot
        # the arm: a shoulder ball, the upper arm hanging a little out, the
        # forearm swivelled forward in the battle grip, the fist
        sh = Vector((s * 11.6, 72.6, 0.0))
        el = Vector((s * 14.0, 59.6, -0.6))
        wr = Vector((s * 13.3, 53.4, 7.8))
        upper = loft_path([(sh + Vector((s * 0.3, 2.6, 0)), 1.4, 1.5), (sh + Vector((s * 0.4, 2.0, 0)), 3.0, 3.1),
                           (sh + Vector((s * 0.5, 0.6, 0)), 3.55, 3.6), (sh.lerp(el, 0.45), 3.1, 3.3),
                           (el + Vector((0, 1.2, 0)), 2.75, 2.9), (el + Vector((0, 0.6, 0)), 2.3, 2.4)], n=14, p=2.2)
        fore = loft_path([(el + Vector((0, 0.9, -0.2)), 2.2, 2.3), (el + Vector((0, 0.2, 0.2)), 2.75, 2.85),
                          (el.lerp(wr, 0.35), 2.95, 2.75), (wr, 2.25, 2.15), (wr + (wr - el).normalized() * 0.5, 1.7, 1.6)],
                         n=14, p=2.2, up=(s, 0, 0))
        body = body + upper + fore
        skin = skin + _fig_hand(s, wr, wr - el)
    # the belt, its buckle and two pouches
    belt = loft_y([(49.4, 7.7, 5.6, 0.0), (49.6, 8.4, 6.2, 0.0), (52.8, 8.3, 6.15, 0.0), (53.0, 7.6, 5.5, 0.0)], n=20, p=2.6)
    buckle = rbox(3.4, 2.8, 0.9, r=0.3, seg=1).move(0, 51.2, 6.45)
    pouches = sum(rbox(3.0, 3.4, 1.8, r=0.5, seg=1).move(s * 5.2, 50.6, 5.8) for s in (-1, 1))
    kit = kit + belt + buckle + pouches
    helmet = _fig_helmet()
    # the eyes: two dots of paint on the face
    hb = bvh_of(_fig_head())
    eyes = Shape()
    for s in (-1, 1):
        dot = ellipsoid(0.62, 0.42, 0.12, seg=8)
        eyes = eyes + lay_on(hb, dot, (s * 2.05, 87.2, 20.0), (0, 0, -1), off=0.02)
    _fig.update(body=body, skin=skin, kit=kit, helmet=helmet, eyes=eyes)
    return _fig


_fig_lie = {}


def figure_lying():
    """The figure fallen on its back: turned a quarter about x (its front
    up, its head toward -z), resting on its helmet and its heels."""
    if _fig_lie:
        return _fig_lie
    parts = figure_parts()
    allp = list(parts.values())
    # about -90 deg: the back of the helmet and the heels lie level
    a = rest_angle_x(allp, lambda p: p.y > 80.0, lambda p: p.y < 6.0, math.radians(-100), math.radians(-80))
    out = {k: s.copy().rot(a, 0, 0) for k, s in parts.items()}
    lo = min(p.y for s in out.values() for p in s.v)
    zc = sum((min(p.z for s in out.values() for p in s.v), max(p.z for s in out.values() for p in s.v))) / 2
    for k in out:
        out[k].move(0, -lo, -zc)
    _fig_lie.update(out)
    return _fig_lie


auto('Real__actionFigureBodyStand', lambda: mm(figure_parts()['body'].copy()), 3000, smooth=50)
auto('Real__actionFigureSkinStand', lambda: mm(figure_parts()['skin'].copy()), 1400, smooth=50)
auto('Real__actionFigureKitStand', lambda: mm(figure_parts()['kit'].copy()), 1400, smooth=45)
auto('Real__actionFigureHelmetStand', lambda: mm(figure_parts()['helmet'].copy()), 600, smooth=50)
auto('Real__actionFigureEyesStand', lambda: mm(figure_parts()['eyes'].copy()), 200, smooth=60)
auto('Real__actionFigureBodyLie', lambda: mm(figure_lying()['body'].copy()), 3000, smooth=50)
auto('Real__actionFigureSkinLie', lambda: mm(figure_lying()['skin'].copy()), 1400, smooth=50)
auto('Real__actionFigureKitLie', lambda: mm(figure_lying()['kit'].copy()), 1400, smooth=45)
auto('Real__actionFigureHelmetLie', lambda: mm(figure_lying()['helmet'].copy()), 600, smooth=50)
auto('Real__actionFigureEyesLie', lambda: mm(figure_lying()['eyes'].copy()), 200, smooth=60)


# ============================================================ dinosaurs ===
# Solid PVC dinosaur figures of the 1980s kind, sold loose in a tub (no
# maker's): moulded in one colour, their skin sculpted in folds, the eyes
# dots of paint. Each stands on its feet, its head toward +z.
def _skin_folds(s, amount, scale, seed):
    """The sculpt's skin: shallow folds and lumps pushed along the normal."""
    bm = propkit.to_bmesh(s)
    bm.normal_update()
    off = Vector((seed * 13.1, seed * 7.7, seed * 3.3))
    for v in bm.verts:
        q = v.co * scale + off
        n = noise.noise(q) * 0.7 + noise.noise(q * 2.7 + off) * 0.3
        v.co += v.normal * n * amount
    out = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


def _flat_feet(s, y0=0.0):
    """What dips under the floor is pressed flat onto it (the feet's soles)."""
    return s.warp(lambda p: Vector((p.x, max(p.y, y0), p.z)))


def _catmull(pts, t):
    """A point on the Catmull-Rom curve through pts (tuples), t in [0, 1]."""
    n = len(pts) - 1
    u = min(max(t, 0.0), 1.0) * n
    i = min(int(u), n - 1)
    f = u - i
    p0, p1, p2, p3 = pts[max(i - 1, 0)], pts[i], pts[i + 1], pts[min(i + 2, n)]
    out = []
    for k in range(len(p1)):
        a, b, c, d = p0[k], p1[k], p2[k], p3[k]
        out.append(0.5 * (2 * b + (-a + c) * f + (2 * a - 5 * b + 4 * c - d) * f * f + (-a + 3 * b - 3 * c + d) * f ** 3))
    return out


def _spine(pts, step=0.3):
    """Ellipsoids close along a curve through pts = [(z, y, rx, ry)] (a
    body tapering smoothly, without the lumps of a few far-apart balls):
    one every `step` of its own radius, each made smaller by what a chain
    that close swells (1.33 at 0.3, measured)."""
    k = {0.3: 1.0 / 1.33, 0.5: 1.0 / 1.245, 0.15: 1.0 / 1.42}[step]
    fine = [_catmull(pts, i / 600) for i in range(601)]
    out, acc = [], 1e9
    for a, b in zip(fine, fine[1:]):
        acc += math.hypot(b[0] - a[0], b[1] - a[1])
        z, y, rx, ry = b
        if acc >= step * min(rx, ry) or b is fine[-1]:
            out.append(('ell', (0, y, z), (rx * k, ry * k, max(rx, ry) * 0.85 * k), 2.0))
            acc = 0.0
    return out


def _dino_finish(elems, tris, seed, res=1.0, folds=0.35):
    s = meta(elems, res=res)
    s = smooth_shape(s, 0.5, 3)
    s = decimate(s, tris * 3)
    s = _skin_folds(s, folds, 0.32, seed)
    s = _flat_feet(s)
    return _flat_feet(decimate(s, tris))


def _eyes_on(body, spots, r=(1.3, 1.1, 0.25)):
    """Painted eyes laid on the body where rays from the side meet it:
    spots = [(origin, direction)]"""
    b = bvh_of(body)
    out = Shape()
    for o, d in spots:
        out = out + lay_on(b, ellipsoid(*r, seg=10), o, d, off=0.03)
    return out


# --- the tyrannosaurus (place_small_toy_dinos): 18 cm, its body level, the
# tail's tip on the floor behind it, the mouth open on its teeth
_trex = {}


def trex_parts():
    if _trex:
        return _trex
    E = []
    spine = [(-98, 2.3, 2.2, 2.0), (-86, 7.0, 4.0, 3.6), (-70, 16.0, 6.6, 6.0), (-52, 30.0, 9.6, 9.0),
             (-34, 44.0, 13.0, 12.5), (-16, 54.0, 16.0, 17.0), (0, 58.0, 17.0, 19.5), (14, 59.0, 15.0, 18.0),
             (26, 62.0, 11.5, 13.5)]
    E += _spine(spine)
    # the neck curving up to the head
    E += [('cap', (0, 62, 26), (0, 72, 40), 10.0), ('cap', (0, 72, 40), (0, 79, 48), 8.6)]
    # the skull, the snout, the brows; the lower jaw dropped open
    E += [('ell', (0, 83, 55), (9.5, 10.0, 14.0)), ('ell', (0, 80.5, 69), (7.4, 7.6, 11.5)),
          ('ell', (0, 79.5, 78), (5.6, 6.0, 5.0))]
    E += [('ball', (s * 6.2, 89.0, 57.5), 3.4) for s in (-1, 1)]
    E += [('ell', (0, 70.5, 64), (7.6, 4.4, 14.0), 2.0, (math.radians(14), 0, 0)),
          ('ell', (0, 68.0, 74.5), (5.2, 3.4, 5.0), 2.0, (math.radians(14), 0, 0))]
    E += [('-ell', (0, 75.0, 70), (5.4, 2.6, 13.0), 2.0, (math.radians(7), 0, 0))]
    for s in (-1, 1):
        # the thigh, the shin, the long foot bones, three toes and a dewclaw
        E += [('ell', (s * 13.5, 49, -4), (7.5, 17.0, 13.0), 2.0, (math.radians(-22), 0, 0)),
              ('cap', (s * 14.5, 37, 5), (s * 15.0, 15, -6), 6.2),
              ('cap', (s * 15.0, 15, -6), (s * 15.0, 4.5, 0), 4.4)]
        for dx, dz, ln in ((-4.2, 1.0, 10.5), (0.0, 1.0, 14.5), (4.2, 1.0, 10.5)):
            E.append(('cap', (s * 15.0, 3.0, 0), (s * 15.0 + s * dx, 2.3, dz + ln), 2.6))
        E.append(('cap', (s * 15.0, 4.5, -1), (s * 14.0, 2.6, -6.5), 1.7))
        # the little arm: upper, fore, two fingers
        E += [('cap', (s * 9.0, 58, 24), (s * 12.0, 51, 30), 3.0), ('cap', (s * 12.0, 51, 30), (s * 11.5, 49, 37), 2.3),
              ('cap', (s * 11.5, 49, 37), (s * 10.6, 48, 40.5), 1.0), ('cap', (s * 11.5, 49, 37), (s * 12.6, 47.5, 40.0), 1.0)]
    body = _dino_finish(E, 3600, 7)
    # the teeth: cones along both jaws' edges, found by rays from inside the mouth
    b = bvh_of(body)
    teeth = Shape()
    for s in (-1, 1):
        for k in range(7):
            z = 61.0 + k * 3.0
            x = s * (5.0 - 0.18 * k)
            for d, y0 in ((1, 75.0), (-1, 75.0 - 0.06 * k)):
                hit, nrm, _, _ = b.ray_cast(Vector((x, y0, z)), Vector((0, d, 0)), 12.0)
                if hit is None:
                    continue
                t = cone(0.85 - 0.02 * k, 2.6 - 0.08 * k, seg=6, bevel_r=0)
                if d > 0:
                    t.rot(math.pi, 0, 0)
                t.move(hit.x, hit.y - d * 0.9, hit.z)
                teeth = teeth + t
    eyes = _eyes_on(body, [((s * 20, 87.0, 60.5), (-s, 0, 0)) for s in (-1, 1)])
    _trex.update(body=body, teeth=teeth, eyes=eyes)
    return _trex


auto('Real__smallTrexBody', lambda: mm(trex_parts()['body'].copy()), 3600, smooth=60)
auto('Real__smallTrexTeeth', lambda: mm(trex_parts()['teeth'].copy()), 400, smooth=30)
auto('Real__smallTrexEyes', lambda: mm(trex_parts()['eyes'].copy()), 200, smooth=60)


# --- the stegosaurus (place_toy_dinosaurs, its old back plates): 13 cm, the
# hips high, the head low, two rows of plates along its back and the four
# spikes of its tail, all moulded with it
_stego = {}


def _plate(h, w, t):
    """A back plate: a rounded diamond, thick at its root, its edge thin."""
    out = [(0, h), (w * 0.42, h * 0.62), (w * 0.5, h * 0.3), (w * 0.32, 0.0), (-w * 0.32, 0.0),
           (-w * 0.5, h * 0.3), (-w * 0.42, h * 0.62)]
    s = prism(round_outline(out, min(w, h) * 0.06, 2), t)
    # thinner toward its edge and tip
    return s.warp(lambda p: Vector((p.x, p.y, p.z * (1.0 - 0.55 * smoothstep(0.0, h, p.y)))))


def stego_parts():
    if _stego:
        return _stego
    spine = [(-70, 26, 2.0, 2.0), (-58, 32, 4.0, 4.2), (-42, 41, 8.0, 8.5), (-24, 50, 15.5, 16.5),
             (-8, 52, 18.0, 19.0), (8, 47, 16.0, 16.0), (22, 40, 11.0, 10.5), (32, 33, 6.4, 6.4),
             (40, 28, 5.0, 5.0)]
    E = _spine(spine)
    E += [('ell', (0, 26.5, 47), (5.4, 5.4, 8.4)), ('ell', (0, 24.5, 55), (3.6, 3.6, 4.5))]
    for s in (-1, 1):
        # hind legs: a heavy thigh, a straight shin, a broad foot
        E += [('ell', (s * 13.0, 38, -16), (6.5, 14.0, 10.0)), ('cap', (s * 14.0, 30, -15), (s * 14.5, 6, -13), 5.8),
              ('ell', (s * 14.5, 3.5, -10.5), (6.0, 3.6, 8.0))]
        # forelegs: shorter, the elbows out
        E += [('cap', (s * 11.0, 36, 13), (s * 13.5, 20, 15), 5.0), ('cap', (s * 13.5, 20, 15), (s * 13.0, 5, 16), 4.4),
              ('ell', (s * 13.0, 3.2, 18), (5.0, 3.2, 5.6))]
    body = _dino_finish(E, 3000, 11)
    b = bvh_of(body)
    plates = Shape()
    # two staggered rows, the largest over the hips, leaning out a little
    rows = [(-50, 6.5, 7), (-40, 9.5, 10), (-30, 12.5, 12.5), (-20, 14.5, 14), (-10, 15, 14.5), (0, 13.5, 13),
            (10, 11, 11), (19, 8.5, 9), (27, 6.5, 7)]
    for i, (z, h, w) in enumerate(rows):
        for s in (-1, 1):
            zz = z + (2.5 if s > 0 else -2.5)
            hit, nrm, _, _ = b.ray_cast(Vector((s * 2.0, 120.0, zz)), Vector((0, -1, 0)), 200.0)
            if hit is None:
                continue
            p = _plate(h, w, 1.8).rot(0, math.pi / 2, 0).rot(0, 0, -s * math.radians(12))
            plates = plates + p.move(hit.x + s * 0.3, hit.y - 1.8, hit.z)
    # the tail's four spikes, out and back and up
    for s in (-1, 1):
        for k, z in enumerate((-60.0, -67.0)):
            hit, nrm, _, _ = b.ray_cast(Vector((0, 120.0, z)), Vector((0, -1, 0)), 200.0)
            if hit is None:
                continue
            sp = cone(1.9 - 0.2 * k, 15.0 - 2.0 * k, seg=8, bevel_r=0).move(0, 7.5 - k, 0)
            sp.rot(math.radians(-40), 0, -s * math.radians(55))
            plates = plates + sp.move(hit.x, hit.y - 2.0, hit.z)
    eyes = _eyes_on(body, [((s * 20, 28.0, 49.5), (-s, 0, 0)) for s in (-1, 1)], r=(1.1, 0.9, 0.22))
    _stego.update(body=body, plates=plates, eyes=eyes)
    return _stego


# --- the brontosaurus (place_toy_dinosaurs, the old smooth back): 16 cm, its
# neck raised, the tail's tip dragging on the floor, legs like pillars
_bronto = {}


def bronto_parts():
    if _bronto:
        return _bronto
    spine = [(-84, 3.0, 2.0, 2.0), (-70, 9.0, 4.5, 4.5), (-52, 22.0, 9.0, 9.0), (-32, 38.0, 15.0, 15.5),
             (-12, 46.0, 20.0, 21.0), (6, 46.0, 20.5, 21.0), (20, 47.0, 15.0, 15.0), (30, 56.0, 9.5, 9.5),
             (38, 68.0, 7.4, 7.4), (44, 80.0, 6.0, 6.0), (48, 88.0, 5.2, 5.2)]
    E = _spine(spine)
    E += [('ell', (0, 92.0, 52.0), (5.6, 5.4, 8.0), 2.0, (math.radians(-12), 0, 0)),
          ('ell', (0, 90.0, 59.5), (4.2, 3.8, 4.6))]
    for s in (-1, 1):
        E += [('ell', (s * 15.0, 36, -16), (8.0, 15.0, 11.0)), ('cap', (s * 15.5, 26, -15), (s * 15.5, 5, -14), 7.0),
              ('ell', (s * 15.5, 3.6, -12.5), (8.0, 3.6, 8.6))]
        E += [('ell', (s * 14.0, 36, 16), (6.8, 12.0, 8.0)), ('cap', (s * 14.5, 28, 17), (s * 14.5, 5, 18.5), 6.3),
              ('ell', (s * 14.5, 3.4, 20.0), (7.0, 3.4, 7.2))]
    body = _dino_finish(E, 3000, 13)
    eyes = _eyes_on(body, [((s * 20, 93.0, 54.0), (-s, 0, 0)) for s in (-1, 1)], r=(1.1, 0.9, 0.22))
    _bronto.update(body=body, eyes=eyes)
    return _bronto


auto('Real__toyStegoBody', lambda: mm(stego_parts()['body'].copy()), 3000, smooth=60)
auto('Real__toyStegoPlates', lambda: mm(stego_parts()['plates'].copy()), 1600, smooth=40)
auto('Real__toyStegoEyes', lambda: mm(stego_parts()['eyes'].copy()), 200, smooth=60)
auto('Real__toyBrontoBody', lambda: mm(bronto_parts()['body'].copy()), 3000, smooth=60)
auto('Real__toyBrontoEyes', lambda: mm(bronto_parts()['eyes'].copy()), 200, smooth=60)


# ========================================================= spinning top ===
# A turned beech twirl top (spun between finger and thumb by its stem): 51
# mm across, 48 mm tall, painted in bands on the lathe (three colours and
# thin black lines), the stem and tip left bare and lacquered. It has
# fallen over as it stopped: lying on its shoulder and its tip, the stem up.
TOP_PROFILE = [(0.0, 0.0), (0.9, 0.35), (1.5, 1.3), (2.4, 2.4), (4.2, 4.0), (9.0, 8.0), (15.0, 12.5),
               (20.0, 16.8), (23.4, 20.3), (24.9, 22.4), (25.4, 24.0), (25.1, 25.5), (24.0, 26.4),
               (20.0, 27.1), (12.0, 28.0), (6.0, 28.7), (4.3, 29.3), (3.5, 30.4), (3.3, 32.0),
               (3.3, 44.0), (3.1, 46.4), (2.3, 47.6), (0.0, 48.0)]


def _top_band(r, y):
    """Which paint a point of the profile (radius, height) carries."""
    if y >= 29.2 or y < 4.8:
        return 'wood'
    if y < 9.6:
        return 'a'
    if y < 10.6:
        return 'line'
    if y < 15.0:
        return 'b'
    if y < 16.0:
        return 'line'
    if y < 21.2:
        return 'c'
    if y < 26.6 or r > 19.0:
        return 'a'           # the shoulder and rim
    if r > 18.0:
        return 'line'
    if r > 11.0:
        return 'b'
    if r > 10.0:
        return 'line'
    return 'c'


TOP_Y_CUTS = (4.8, 9.6, 10.6, 15.0, 16.0, 21.2, 26.6, 29.2)
TOP_R_CUTS = (19.0, 18.0, 11.0, 10.0)


def _resample(prof, step):
    """The profile with a point wherever a band starts (so each band's edge
    is where it is painted) and none farther apart than `step`."""
    out = [prof[0]]
    for a, b in zip(prof, prof[1:]):
        ts = {1.0}
        for yc in TOP_Y_CUTS:
            if (a[1] - yc) * (b[1] - yc) < 0:
                ts.add((yc - a[1]) / (b[1] - a[1]))
        for rc in TOP_R_CUTS:
            if (a[0] - rc) * (b[0] - rc) < 0 and a[1] > 26.0:
                ts.add((rc - a[0]) / (b[0] - a[0]))
        ts = sorted(ts)
        t0 = 0.0
        for t1 in ts:
            L = math.hypot(b[0] - a[0], b[1] - a[1]) * (t1 - t0)
            n = max(1, int(math.ceil(L / step)))
            for k in range(1, n + 1):
                t = t0 + (t1 - t0) * k / n
                out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t))
            t0 = t1
    return out


_top = {}


def top_parts():
    """{'wood', 'a', 'b', 'c', 'line'}: the top's bands, each an open strip
    of the lathe, lying as it fell"""
    if _top:
        return _top
    prof = _resample(TOP_PROFILE, 2.2)
    bands = {}
    run, cur = [prof[0]], None
    for p, q in zip(prof, prof[1:]):
        k = _top_band((p[0] + q[0]) / 2, (p[1] + q[1]) / 2)
        if cur is None:
            cur = k
        if k != cur:
            bands.setdefault(cur, []).append(run)
            run, cur = [p], k
        run.append(q)
    bands.setdefault(cur, []).append(run)
    parts = {}
    for k, runs in bands.items():
        s = Shape()
        for r in runs:
            s = s + lathe(r, seg=30, cap_bottom=False, cap_top=False)
        parts[k] = s
    # lie it on its shoulder and tip: turned about z until both touch
    allv = [p for s in parts.values() for p in s.v]
    best, ang = 1e9, 0.0
    for i in range(361):
        a = math.radians(30 + 40 * i / 360)
        ca, sa = math.cos(a), math.sin(a)
        ytip = min(p.x * sa + p.y * ca for p in allv if p.y < 2.0)
        yrim = min(p.x * sa + p.y * ca for p in allv if 20.0 < p.y < 27.0)
        if abs(ytip - yrim) < best:
            best, ang = abs(ytip - yrim), a
    for k in parts:
        parts[k].rot(0, 0, -ang)
    lo = min(p.y for s in parts.values() for p in s.v)
    xs = [p.x for s in parts.values() for p in s.v]
    for k in parts:
        parts[k].move(-(min(xs) + max(xs)) / 2, -lo, 0)
    _top.update(parts)
    return _top


auto('Real__spinTopWood', lambda: mm(top_parts()['wood'].copy()), 1200, smooth=50)
auto('Real__spinTopPaintA', lambda: mm(top_parts()['a'].copy()), 900, smooth=50)
auto('Real__spinTopPaintB', lambda: mm(top_parts()['b'].copy()), 700, smooth=50)
auto('Real__spinTopPaintC', lambda: mm(top_parts()['c'].copy()), 700, smooth=50)
auto('Real__spinTopLines', lambda: mm(top_parts()['line'].copy()), 500, smooth=50)


# ============================================================ fairy wand ===
# A party-shop fairy wand (a dressing-up toy): a moulded plastic star, 56 mm
# point to point, faceted like a cut star and glittered, pushed onto a 5 mm
# plastic rod 18 cm long; three satin ribbons tied under the star. Lying on
# the floor, the rod along z (the star toward +z), the ribbons trailing
# back along it.
WAND_R, WAND_r = 28.0, 12.5
WAND_ROD_R = 2.5
WAND_STAR_Z = 96.5          # the star's centre: the rod 6 mm into its foot


def wand_star():
    """The star lying face up: its back flat on the floor, each arm's ridge
    rising to the raised centre (6 mm), its edge 1.4 mm thick; one point
    toward +z, the rod's socket between the two at -z."""
    pts = []
    for i in range(10):
        a = math.pi / 2 + i * math.pi / 5
        r = WAND_R if i % 2 == 0 else WAND_r
        pts.append((r * math.cos(a), r * math.sin(a)))
    n = len(pts)
    v = [Vector((x, 0.0, -y)) for x, y in pts] + [Vector((x, 1.4, -y)) for x, y in pts]
    v.append(Vector((0, 6.0, 0)))
    v.append(Vector((0, 0.0, 0)))
    top, bot = 2 * n, 2 * n + 1
    f = []
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))         # the edge
        f.append((n + i, n + j, top))          # the facets
        f.append((j, i, bot))                  # the back
    s = propkit._fix_winding(Shape(v, f))
    s = bevel(s, 0.35, seg=1, angle=25)
    # the arms' inner vertices are -z of the point at +z in the canvas' sense: turn so a point is at +z
    return s.rot(0, math.pi, 0).move(0, 0, WAND_STAR_Z)


def wand_rod():
    rod = cyl(WAND_ROD_R, h=180.0, seg=12, bevel_r=0.9).rot(math.pi / 2, 0, 0)
    return rod.move(0, WAND_ROD_R, WAND_STAR_Z - WAND_r + 6.0 - 90.0)


def wand_knot():
    """The ribbons' knot round the rod under the star: a wrapped band and
    the knot's pinched bow."""
    z0 = WAND_STAR_Z - WAND_r - 4.0
    band = torus(WAND_ROD_R + 0.4, 0.9, seg=14, tseg=6).move(0, WAND_ROD_R, z0)
    band2 = torus(WAND_ROD_R + 0.4, 0.9, seg=14, tseg=6).move(0, WAND_ROD_R, z0 - 1.6)
    knot = ellipsoid(2.6, 1.8, 2.4, seg=10).move(1.2, WAND_ROD_R + 2.4, z0 - 0.8)
    return (band + band2 + knot).warp(lambda p: Vector((p.x, max(p.y, 0.05), p.z)))


RIB_W, RIB_T = 6.0, 0.25
RIB_LEN = {'S': 80.0, 'M': 110.0, 'L': 140.0}
RIB_FAN = (-24.0, 8.0, 28.0)            # degrees off straight back, ribbon 0, 1, 2
RIB_SIDE = (-1.0, 1.0, 1.0)             # the side of the rod each comes down


def wand_ribbon(i, size):
    """Ribbon i (0..2) of length `size`: from the knot, down off the rod to
    the floor, then lying in a loose curve fanned off the rod, its cut end
    curled up a little; a thin slab (both faces)."""
    L = RIB_LEN[size]
    random.seed(zlib.crc32(f'rib{i}{size}'.encode()))
    z0 = WAND_STAR_Z - WAND_r - 4.8
    fan = math.radians(RIB_FAN[i])
    wav, ph = rnd(5.0, 9.0), rnd(0, TAU)
    pts, ups = [], []
    n = max(12, int(L / 3.5))
    for k in range(n + 1):
        t = k / n
        d = L * t
        # its heading swings from the rod's line out to its fan angle
        h = fan * smoothstep(0.0, 0.35, t) + 0.25 * math.sin(d / 38.0 + ph) * smoothstep(0.1, 0.4, t)
        if k == 0:
            p = Vector((0.6 * (i - 1), WAND_ROD_R + 1.2, z0))
        else:
            q = pts[-1]
            step = L / n
            p = q + Vector((math.sin(h) * step, 0, -math.cos(h) * step))
            p.x += math.sin(d / wav + ph) * 0.06 * step
        # height: off the rod and down to the floor in the first 14 mm, the end curled up
        y = (WAND_ROD_R + 1.2) * (1.0 - smoothstep(0.0, 14.0, d)) + 0.18 + 2.2 * smoothstep(0.86, 1.0, t) ** 2
        pts.append(Vector((p.x, y, p.z)))
        ups.append(Vector((0, 1, 0)))
    # each comes down off the rod to its own side
    for k, p in enumerate(pts):
        p.x += RIB_SIDE[i] * (WAND_ROD_R + RIB_W / 2 + 0.6) * smoothstep(0.0, 14.0, L * k / n)
    v, f = [], []
    for k, p in enumerate(pts):
        t = (pts[min(k + 1, n)] - pts[max(k - 1, 0)]).normalized()
        across = t.cross(ups[k]).normalized()
        # a slight twist where it turned over in falling
        tw = 0.35 * math.sin(k / n * math.pi * 1.5 + ph)
        nrm = ups[k] * math.cos(tw) + across * math.sin(tw)
        across = t.cross(nrm).normalized()
        for sx, sy in ((-1, 1), (1, 1), (1, -1), (-1, -1)):
            v.append(p + across * sx * RIB_W / 2 + nrm * (sy * RIB_T / 2 + RIB_T / 2))
    for k in range(n):
        a, b = 4 * k, 4 * (k + 1)
        for j in range(4):
            j2 = (j + 1) % 4
            f.append((a + j, a + j2, b + j2, b + j))
    f.append((0, 1, 2, 3))
    f.append((4 * n + 3, 4 * n + 2, 4 * n + 1, 4 * n))
    s = propkit._fix_winding(Shape(v, f))
    return s.warp(lambda p: Vector((p.x, max(p.y, 0.0), p.z)))


auto('Real__fairyWandStar', lambda: mm(wand_star()), 300, smooth=20)
auto('Real__fairyWandRod', lambda: mm(wand_rod()), 300, smooth=40)
auto('Real__fairyWandKnot', lambda: mm(wand_knot()), 500, smooth=50)
for _i in range(3):
    for _sz in 'SML':
        auto(f'Real__fairyWandRibbon{_i}{_sz}', lambda i=_i, sz=_sz: mm(wand_ribbon(i, sz)), 400, smooth=50)


# ========================================================== baby rattle ===
# A hard plastic baby rattle of the 1960s: a hollow ball (40 mm) moulded in
# two halves, its sides fluted, beads loose inside; a cream handle fluted
# to its waist, ending in a ring a small hand holds, a cap over the mould's
# gate at the ball's end. 12 cm long, lying on the ball and the ring.
def rattle_ball():
    R = 20.0
    s = sphere(R, seg=48, rings=16)
    # eight shallow flutes about its axis (x), and the seam where the halves meet
    def fl(p):
        d = p.normalized()
        th = math.atan2(d.z, d.y)
        rho = math.hypot(d.y, d.z)
        c = abs(math.cos(4 * th))
        r = R - 0.9 * math.exp(-(c / 0.16) ** 2) * rho ** 1.5
        r += 0.3 * math.exp(-(d.x / 0.025) ** 2) * rho
        return d * r
    return s.warp(fl)


def rattle_handle():
    # along +x from inside the ball (its centre at x = 0) to the ring
    prof = [(0.0, 14.0), (7.6, 14.0), (8.4, 15.4), (7.2, 17.0), (5.2, 19.0), (4.6, 24.0), (4.0, 33.0),
            (4.1, 42.0), (4.8, 50.0), (5.2, 53.0), (4.6, 55.0), (0.0, 55.4)]
    h = lathe(prof, seg=18).rot(0, 0, -math.pi / 2)
    ring = torus(11.5, 3.6, seg=28, tseg=10).rot(math.pi / 2, 0, 0).move(55.0 + 10.2, 0, 0)
    cap = lathe([(0.0, 0.0), (5.6, 0.0), (5.4, 1.4), (3.6, 2.6), (0.0, 3.0)], seg=16)
    cap = cap.rot(0, 0, math.pi / 2).move(-19.2, 0, 0)
    return h + ring + cap


_rattle = {}


def rattle_parts():
    if _rattle:
        return _rattle
    parts = {'ball': rattle_ball(), 'handle': rattle_handle()}
    # lying: tipped about z until the ball and the ring's far edge both touch
    allv = [p for s in parts.values() for p in s.v]
    best, ang = 1e9, 0.0
    for i in range(401):
        a = math.radians(-30 + 32 * i / 400)
        ca, sa = math.cos(a), math.sin(a)
        yb = min(p.y * ca + p.x * sa for p in parts['ball'].v)
        yr = min(p.y * ca + p.x * sa for p in parts['handle'].v if p.x > 52.0)
        if abs(yb - yr) < best:
            best, ang = abs(yb - yr), a
    lo = 1e9
    for k in parts:
        parts[k].rot(0, 0, ang)
        lo = min(lo, min(p.y for p in parts[k].v))
    xs = [p.x for s in parts.values() for p in s.v]
    for k in parts:
        parts[k].move(-(min(xs) + max(xs)) / 2, -lo, 0)
    _rattle.update(parts)
    return _rattle


auto('Real__babyRattleBall', lambda: mm(rattle_parts()['ball'].copy()), 1600, smooth=40)
auto('Real__babyRattleHandle', lambda: mm(rattle_parts()['handle'].copy()), 1100, smooth=45)


# ========================================================= teething ring ===
# A water-filled teething ring: two sheets of soft PVC welded round their
# inner and outer edges (the welds a thin flange), filled; 83 mm across and
# 15 mm thick, its top moulded in sectors of ribs and of nubs for gums.
# Lying flat.
def teether_ring():
    Rc, a, b = 31.0, 10.5, 7.3
    n, m = 48, 14
    v, f = [], []
    for i in range(n):
        th = TAU * i / n
        for j in range(m):
            ph = TAU * j / m
            cx, sy = math.cos(ph), math.sin(ph)
            v.append(Vector(((Rc + a * cx) * math.cos(th), b * sy + b, (Rc + a * cx) * math.sin(th))))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    s = propkit._fix_winding(Shape(v, f))
    # its weight presses the bottom flat a little where it lies
    s = s.warp(lambda p: Vector((p.x, 0.6 + (p.y - 0.6) * 0.5 if p.y < 0.6 else p.y, p.z)))
    # the weld flanges at the outer and inner edges
    flange = Shape()
    for R in (Rc + a + 0.3, Rc - a - 0.3):
        flange = flange + torus(R, 0.7, seg=48, tseg=4).rot(math.pi / 2, 0, 0).scale(1, 0.55, 1).move(0, b, 0)
    # two sectors ribbed round the tube, two sectors of nubs on its top
    ribs = Shape()
    nubs = Shape()
    for q in range(4):
        th0 = TAU * q / 4 + TAU / 16
        if q % 2 == 0:
            for k in range(6):
                th = th0 + (k - 2.5) * math.radians(4.8)
                ring = [Vector(((Rc + a * 1.03 * math.cos(ph)) * math.cos(th), b * 1.04 * math.sin(ph) + b,
                                (Rc + a * 1.03 * math.cos(ph)) * math.sin(th))) for ph in
                        [math.radians(-30 + 240 * t / 10) for t in range(11)]]
                ribs = ribs + tube(ring, 0.55, seg=4, caps=True)
        else:
            for k in range(5):
                th = th0 + (k - 2) * math.radians(6.5)
                for rr, ph in ((Rc - 4.5, 2.0), (Rc + 4.5, 1.1), (Rc, 1.57)):
                    pt = Vector(((Rc + a * math.cos(ph)) * math.cos(th), b * math.sin(ph) + b, (Rc + a * math.cos(ph)) * math.sin(th)))
                    nb = sphere(1.5, seg=6, rings=3).scale(1, 0.8, 1)
                    nubs = nubs + nb.move(*pt)
    out = s + flange + ribs + nubs
    return out.move(0, -min(p.y for p in out.v), 0)


auto('Real__teethingRing', lambda: mm(teether_ring()), 4200, smooth=50)


# ============================================================ pet toys ===
# A dog's toys. The squeaky bone: hollow moulded vinyl, 152 mm (6 in) long,
# two knobs at each end, the squeaker's round pip in its middle, its mould
# seam round its edge, chewed: tooth dents and a few holes through the
# vinyl. Lying on its side. The ball: a solid natural rubber ball 76 mm (3
# in) across, its mould seam round it, pocked with tooth marks and a gouge.
def pet_bone():
    E = [('cap', (0, 11, -52), (0, 11, 52), 8.6)]
    for sz in (-1, 1):
        for sx in (-1, 1):
            E.append(('ell', (sx * 10.5, 11.5, sz * 62.0), (12.0, 10.5, 12.0)))
    # the tooth marks: dents pressed in, a few through
    random.seed(zlib.crc32(b'petbone'))
    for k in range(9):
        z = rnd(-60, 60)
        side = random.choice((-1, 1))
        y = rnd(14, 21)
        E.append(('-ball', (side * rnd(2, 9), y + 2.0, z), rnd(1.4, 2.4), 2.0))
    s = meta(E, res=1.0, unit=4.0)
    s = smooth_shape(s, 0.5, 2)
    s = s.warp(lambda p: Vector((p.x, 11.0 + (p.y - 11.0) * 0.82, p.z)))
    s = decimate(s, 1600)
    # the squeaker's pip on top, the seam round its middle
    b = bvh_of(s)
    pip = lathe([(0.0, 0.0), (3.6, 0.0), (3.5, 0.6), (2.6, 1.1), (0.0, 1.2)], seg=16)
    pip = lay_on(b, pip.rot(math.pi / 2, 0, 0), (0, 60, 4.0), (0, -1, 0), off=-0.3)
    return to_floor(_flat_feet(s.move(0, -min(p.y for p in s.v) - 0.4, 0) + pip.move(0, -min(p.y for p in s.v) - 0.4, 0)))


_bone = []


def pet_bone_seam():
    """The mould's parting line: a thin ridge round the bone at its middle
    height, where the two halves were welded (the bone cut level there,
    its outline followed a shade outside it)."""
    if not _bone:
        _bone.append(pet_bone())
    s = _bone[0]
    lo, hi = s.bounds()
    yc = lo.y + (hi.y - lo.y) * 0.47
    bm = propkit.to_bmesh(s)
    geom = list(bm.verts) + list(bm.edges) + list(bm.faces)
    res = bmesh.ops.bisect_plane(bm, geom=geom, plane_co=(0, yc, 0), plane_no=(0, 1, 0), dist=1e-6)
    edges = [e for e in res['geom_cut'] if isinstance(e, bmesh.types.BMEdge)]
    nbr = {}
    for e in edges:
        a, b = e.verts
        nbr.setdefault(a, []).append(b)
        nbr.setdefault(b, []).append(a)
    # the longest loop (the outline; the pip and dents give none at this height)
    best = []
    seen = set()
    for start in nbr:
        if start in seen:
            continue
        loop, prev, cur = [start], None, start
        seen.add(start)
        while True:
            nxt = [v for v in nbr[cur] if v is not prev]
            if not nxt or nxt[0] is start:
                break
            prev, cur = cur, nxt[0]
            if cur in seen:
                break
            seen.add(cur)
            loop.append(cur)
        if len(loop) > len(best):
            best = loop
    pts = [v.co.copy() for v in best]
    bm.free()
    # thin it to about 1.2 mm apart and push it out a shade
    thin = [pts[0]]
    for p in pts[1:]:
        if (p - thin[-1]).length > 1.2:
            thin.append(p)
    area = sum(a.x * b.z - b.x * a.z for a, b in zip(thin, thin[1:] + thin[:1]))
    sgn = 1.0 if area > 0 else -1.0
    out = []
    m = len(thin)
    for i, p in enumerate(thin):
        t = (thin[(i + 1) % m] - thin[i - 1]).normalized()
        nrm = Vector((t.z, 0, -t.x)) * -sgn
        out.append(p + nrm * 0.1)
    out.append(out[0])
    return tube(out, 0.45, seg=4, caps=False)


BALL_R = 38.0


def pet_ball():
    """The solid rubber ball, resting, its seam tilted; tooth pocks and a
    gouge pushed into it."""
    random.seed(zlib.crc32(b'petball'))
    n = Vector((0.0, math.cos(math.radians(28)), math.sin(math.radians(28))))
    dents = [(Vector((rnd(-1, 1), rnd(-0.2, 1), rnd(-1, 1))).normalized(), rnd(0.5, 1.1), rnd(0.035, 0.06)) for _ in range(14)]
    gouge = (Vector((0.6, 0.55, -0.58)).normalized(), Vector((0.7, -0.1, 0.7)).normalized())

    def f(p):
        d = p.normalized()
        r = BALL_R
        # the seam: a ridge on the great circle square to n
        a = d.dot(n) * BALL_R
        r += 0.35 * math.exp(-(a / 0.6) ** 2)
        for c, depth, w in dents:
            ang = math.acos(max(-1.0, min(1.0, d.dot(c))))
            r -= depth * math.exp(-(ang / w) ** 2)
        gc, gd = gouge
        ang = math.acos(max(-1.0, min(1.0, d.dot(gc))))
        along = d.dot(gd)
        r -= 1.6 * math.exp(-(ang / 0.12) ** 2 * (1.0 - 0.8 * min(1.0, abs(along) * 4))) * (1 if ang < 0.25 else 0)
        return d * r
    s = sphere(BALL_R, seg=40, rings=22).warp(f)
    return s.move(0, BALL_R, 0)


def pet_ball_band():
    """The contrasting band moulded round some balls: a stripe 8 mm wide on
    a great circle, a shade proud."""
    nb = Vector((math.sin(math.radians(-35)), math.cos(math.radians(-35)) * 0.4, 0.9)).normalized()
    s = Shape()
    rings = []
    seg = 64
    for h in (-4.0, -2.0, 0.0, 2.0, 4.0):
        e1 = nb.orthogonal().normalized()
        e2 = nb.cross(e1).normalized()
        rr = math.sqrt(BALL_R * BALL_R - h * h) + 0.18
        rings.append([nb * h + (e1 * math.cos(TAU * i / seg) + e2 * math.sin(TAU * i / seg)) * rr for i in range(seg)])
    v, f = [], []
    for r in rings:
        v += r
    for k in range(len(rings) - 1):
        for i in range(seg):
            j = (i + 1) % seg
            f.append((k * seg + i, k * seg + j, (k + 1) * seg + j, (k + 1) * seg + i))
    s = propkit._fix_winding(Shape(v, f))
    return s.move(0, BALL_R, 0)


auto('Real__petBone', lambda: mm(_bone[0].copy() if _bone else pet_bone()), 1700, smooth=60)
auto('Real__petBoneSeam', lambda: mm(pet_bone_seam()), 1000, smooth=60)
auto('Real__petBall', lambda: mm(pet_ball()), 1800, smooth=50)
auto('Real__petBallBand', lambda: mm(pet_ball_band()), 520, smooth=50)


# ======================================================== toy microphone ===
# A child's toy microphone (no maker's): 140 mm long, a round grille 42 mm
# across moulded in grey plastic as a mesh and painted silver, a collar,
# a coloured handle tapering to a rounded butt with grip rings, a slide
# switch on its top. Lying on its grille and its butt, rolled a little.
def mic_parts_raw():
    # along z: the grille's centre at z = +49, the butt at z = -70
    grille = sphere(21.0, seg=32, rings=16).move(0, 0, 49.0)
    band = torus(20.5, 1.2, seg=32, tseg=8).move(0, 0, 49.0)
    prof = [(0.0, -70.0), (7.5, -70.0), (9.6, -69.0), (10.6, -67.2), (10.8, -64.0), (10.2, -62.8),
            (10.8, -61.6), (10.2, -60.4), (10.8, -59.2), (10.2, -58.0), (10.9, -56.8), (11.4, -40.0),
            (12.4, -10.0), (13.2, 18.0), (13.4, 22.0), (16.0, 23.0), (17.4, 25.0), (17.6, 29.0),
            (16.6, 31.5), (14.0, 33.0), (0.0, 33.5)]
    handle = lathe(prof, seg=28).rot(math.pi / 2, 0, 0)
    # the switch: a raised plate on the top, its slider in a slot
    boss = rbox(8.0, 2.4, 22.0, r=0.8, seg=2).move(0, 12.4, -4.0)
    slot = box(3.2, 3.0, 13.0).move(0, 14.0, -4.0)
    boss = cut(boss, slot)
    knob = rbox(2.8, 3.0, 5.0, r=0.6, seg=1)
    knob = knob + rbox(2.8, 0.7, 5.0, r=0.2, seg=1).move(0, 1.0, -1.4) + rbox(2.8, 0.7, 5.0, r=0.2, seg=1).move(0, 1.0, 1.4)
    knob = knob.move(0, 13.6, -7.5)
    return {'grille': grille, 'band': band, 'handle': handle + boss, 'knob': knob}


_mic = {}


def mic_parts():
    if _mic:
        return _mic
    parts = mic_parts_raw()
    # rolled 35 degrees about its length, the switch a little to one side
    for k in parts:
        parts[k].rot(0, 0, math.radians(35))
    allp = list(parts.values())
    a = rest_angle_x(allp, lambda p: p.z > 40.0, lambda p: p.z < -60.0, math.radians(-15), math.radians(15))
    for k in parts:
        parts[k].rot(a, 0, 0)
    lo = min(p.y for s in parts.values() for p in s.v)
    zs = [p.z for s in parts.values() for p in s.v]
    for k in parts:
        parts[k].move(0, -lo, -(min(zs) + max(zs)) / 2)
    _mic.update(parts)
    return _mic


auto('Real__toyMicGrille', lambda: mm(mic_parts()['grille'].copy()), 1000, smooth=50)
auto('Real__toyMicBand', lambda: mm(mic_parts()['band'].copy()), 520, smooth=50)
auto('Real__toyMicHandle', lambda: mm(mic_parts()['handle'].copy()), 1600, smooth=40)
auto('Real__toyMicKnob', lambda: mm(mic_parts()['knob'].copy()), 200, smooth=30)
