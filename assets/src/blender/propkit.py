"""
propkit — a small modelling kit for the Backrooms props, run inside Blender.

Every prop in the game is assembled by a placer (`place_*` in
crates/placers/) from the engine's primitives (`box_geo!`, `cyl_geo!`,
...). A *part* made here replaces one of those primitives: it is modelled in
the primitive's own frame and at its canonical size, and the game fits it to
the primitive's bounding box when it is built (`pg("Name__role", primitive)`,
`assets::props::prop_part`), so the placer keeps its layout, colours and
randomness. A batch of real things (through its `auto()`) models at the
thing's real size instead (docs/design/PROPS.md, "How a prop is remodelled").

Conventions (read these before writing a part):

* Coordinates are the game's: +Y up, metres. Build the part exactly
  where the primitive would be:
    box(w, h, d)            centred on the origin
    cyl(rt, rb, h)          axis +Y, centred (y from -h/2 to +h/2)
    cone(r, h)              axis +Y, centred, apex at +h/2
    sphere(r)               centred
    torus(R, r)             ring in the XY plane, around the Z axis
    capsule(r, len)         axis +Y, centred, total height len + 2r
* One part = one material. Colour details that differ belong to another part.
* A part should look like the real thing at arm's length: chamfered or
  rounded edges, rims and lips, seams and panel lines cut as geometry, screw
  heads, shaped handles, soft forms for soft things.
* Stay within the triangle budget (`budget_for(frame)`): small parts are
  cheap, big furniture may spend more.

Shapes are plain vertex/face lists (class Shape). Build them with the
constructors below, transform them, join them with `+`, then hand the result
to `part()`.
"""
import bpy, bmesh, math, random
from mathutils import Vector, Matrix, Euler, noise

TAU = math.pi * 2


# ----------------------------------------------------------------- Shape ---
class Shape:
    """A mesh as python lists: verts [Vector], faces [tuple of vert ids],
    plus a set of edges (vertex-id pairs) that must stay sharp."""
    __slots__ = ('v', 'f', 'sharp')

    def __init__(self, v=None, f=None, sharp=None):
        self.v = [Vector(p) for p in (v or [])]
        self.f = [tuple(x) for x in (f or [])]
        self.sharp = set(sharp or ())

    def copy(self):
        return Shape([p.copy() for p in self.v], list(self.f), set(self.sharp))

    def __add__(self, o):
        n = len(self.v)
        return Shape(self.v + [p.copy() for p in o.v], self.f + [tuple(i + n for i in f) for f in o.f],
                     self.sharp | {(a + n, b + n) for a, b in o.sharp})

    __radd__ = lambda self, o: self if o == 0 else o + self    # sum([...])

    # transforms (return self so calls chain)
    def apply(self, m):
        m = m if isinstance(m, Matrix) else Matrix(m)
        for i, p in enumerate(self.v):
            self.v[i] = m @ p
        if m.determinant() < 0:
            self.f = [tuple(reversed(f)) for f in self.f]
        return self

    def move(self, x=0, y=0, z=0):
        d = Vector((x, y, z))
        for p in self.v:
            p += d
        return self

    def rot(self, x=0, y=0, z=0, order='XYZ'):
        return self.apply(Euler((x, y, z), order).to_matrix().to_4x4())

    def scale(self, x=1, y=None, z=None):
        y = x if y is None else y
        z = x if z is None else z
        return self.apply(Matrix.Diagonal((x, y, z, 1)))

    def warp(self, fn):
        """fn(Vector) -> Vector, applied to every vertex"""
        self.v = [Vector(fn(p.copy())) for p in self.v]
        return self

    def bounds(self):
        lo = Vector((min(p.x for p in self.v), min(p.y for p in self.v), min(p.z for p in self.v)))
        hi = Vector((max(p.x for p in self.v), max(p.y for p in self.v), max(p.z for p in self.v)))
        return lo, hi

    def fit(self, lo, hi):
        """Scale/translate so the bounds become exactly [lo, hi] (per axis)."""
        a, b = self.bounds()
        lo, hi = Vector(lo), Vector(hi)
        for p in self.v:
            for k in range(3):
                s = (b[k] - a[k])
                p[k] = lo[k] + ((p[k] - a[k]) / s if s > 1e-9 else 0.5) * (hi[k] - lo[k])
        return self


# ----------------------------------------------------- bmesh round trips ---
def to_bmesh(s):
    bm = bmesh.new()
    vs = [bm.verts.new(p) for p in s.v]
    bm.verts.ensure_lookup_table()
    for f in s.f:
        try:
            bm.faces.new([vs[i] for i in f])
        except ValueError:
            pass            # duplicate face (e.g. shared caps) — skip
    bm.normal_update()
    return bm


def from_bmesh(bm, keep_sharp=True):
    bm.verts.index_update()
    s = Shape([v.co.copy() for v in bm.verts], [tuple(v.index for v in f.verts) for f in bm.faces])
    if keep_sharp:
        s.sharp = {(e.verts[0].index, e.verts[1].index) for e in bm.edges if not e.smooth}
    return s


def _mesh_obj(name, s):
    bm = to_bmesh(s)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def _eval_to_shape(ob):
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
    bm = bmesh.new()
    bm.from_mesh(me)
    s = from_bmesh(bm, keep_sharp=False)
    bm.free()
    bpy.data.meshes.remove(me)
    return s


def _drop(*obs):
    for ob in obs:
        me = ob.data
        bpy.data.objects.remove(ob)
        if me and me.users == 0:
            bpy.data.meshes.remove(me)


# ------------------------------------------------------------ primitives ---
def box(w, h, d):
    """Axis-aligned box centred on the origin (sharp edges)."""
    x, y, z = w / 2, h / 2, d / 2
    v = [(-x, -y, -z), (x, -y, -z), (x, y, -z), (-x, y, -z), (-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    f = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (2, 3, 7, 6), (1, 2, 6, 5), (0, 4, 7, 3)]
    return Shape(v, f)


def rbox(w, h, d, r=None, seg=2, profile=0.5):
    """Box with rounded (seg >= 2) or chamfered (seg = 1) edges.
    r defaults to 12 % of the smallest side, capped at 6 mm."""
    if r is None:
        r = min(min(w, h, d) * 0.12, 0.006)
    return bevel(box(w, h, d), r, seg=seg, profile=profile)


def lathe(profile, seg=32, cap_bottom=True, cap_top=True, sharp_angle=None):
    """Revolve [(radius, y), ...] (bottom to top) around +Y.
    A point with radius 0 closes the surface there (no cap needed)."""
    v, f = [], []
    rings = []
    for (r, y) in profile:
        if r <= 1e-7:
            v.append(Vector((0, y, 0)))
            rings.append([len(v) - 1])
        else:
            ids = []
            for i in range(seg):
                a = TAU * i / seg
                v.append(Vector((r * math.sin(a), y, r * math.cos(a))))
                ids.append(len(v) - 1)
            rings.append(ids)
    for a, b in zip(rings, rings[1:]):
        if len(a) == 1 and len(b) == 1:
            continue
        if len(a) == 1:
            for i in range(seg):
                f.append((a[0], b[i], b[(i + 1) % seg]))
        elif len(b) == 1:
            for i in range(seg):
                f.append((a[i], b[0], a[(i + 1) % seg]))
        else:
            for i in range(seg):
                j = (i + 1) % seg
                f.append((a[i], b[i], b[j], a[j]))
    if cap_bottom and len(rings[0]) > 1:
        f.append(tuple(reversed(rings[0])))
    if cap_top and len(rings[-1]) > 1:
        f.append(tuple(rings[-1]))
    s = Shape(v, f)
    s = _fix_winding(s)
    return s


def cyl(rt, rb=None, h=1.0, seg=32, bevel_r=None, open_ended=False, bevel_seg=2):
    """Cylinder / frustum along +Y, centred. bevel_r rounds the cap rims
    (default 8 % of the smaller of radius and height, capped at 3 mm)."""
    rb = rt if rb is None else rb
    y = h / 2
    if open_ended:
        return lathe([(rb, -y), (rt, y)], seg, cap_bottom=False, cap_top=False)
    if bevel_r is None:
        bevel_r = min(min(max(rt, rb), h) * 0.08, 0.003)
    if bevel_r <= 0 or bevel_seg < 1:
        return lathe([(0, -y), (rb, -y), (rt, y), (0, y)], seg)
    prof = [(0, -y)]
    prof += _round_corner((rb, -y), (0, -y), (rt, y), bevel_r, bevel_seg)
    prof += _round_corner((rt, y), (rb, -y), (0, y), bevel_r, bevel_seg)
    prof.append((0, y))
    return lathe(prof, seg)


def vessel(profile, wall, seg=32, floor=None):
    """An open container (cup, bucket, bin, pot, bottle neck): the outer
    profile [(r, y), ...] from bottom to rim is revolved, rolled over at the
    rim and followed down the inside `wall` thick to a floor `floor` above
    the bottom (default = wall)."""
    floor = wall if floor is None else floor
    y0 = profile[0][1]
    outer = [(0.0, y0)] + [(r, y) for r, y in profile if r > 0]
    inner = [(max(r - wall, wall * 0.5), y) for r, y in reversed(outer[1:]) if y >= y0 + floor]
    rim_r, rim_y = outer[-1]
    lip = [(rim_r - wall * 0.5, rim_y + wall * 0.35)]
    return lathe(outer + lip + inner + [(0.0, y0 + floor)], seg, cap_bottom=False, cap_top=False)


def cone(r, h, seg=32, bevel_r=None):
    return cyl(0.0, r, h, seg, bevel_r=0 if bevel_r is None else bevel_r)


def sphere(r, seg=24, rings=None):
    rings = rings or max(6, seg // 2)
    prof = [(0, -r)] + [(r * math.sin(math.pi * i / rings), -r * math.cos(math.pi * i / rings)) for i in range(1, rings)] + [(0, r)]
    return lathe(prof, seg)


def ellipsoid(rx, ry, rz, seg=24):
    return sphere(1, seg).scale(rx, ry, rz)


def capsule(r, length, seg=24, cap_rings=6):
    prof = [(0, -length / 2 - r)]
    for i in range(1, cap_rings + 1):
        a = math.pi / 2 * i / cap_rings
        prof.append((r * math.sin(a), -length / 2 - r * math.cos(a)))
    for i in range(0, cap_rings):
        a = math.pi / 2 * i / cap_rings
        prof.append((r * math.cos(a), length / 2 + r * math.sin(a)))
    prof.append((0, length / 2 + r))
    return lathe(prof, seg)


def torus(R, r, seg=32, tseg=12, arc=TAU):
    """Ring in the XY plane around the Z axis, like the engine's `torus_geo!`."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    v, f = [], []
    for i in range(n):
        a = arc * i / seg
        c = Vector((math.cos(a), math.sin(a), 0))
        for j in range(tseg):
            b = TAU * j / tseg
            v.append(c * (R + r * math.cos(b)) + Vector((0, 0, r * math.sin(b))))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(tseg):
            j2 = (j + 1) % tseg
            f.append((i * tseg + j, i2 * tseg + j, i2 * tseg + j2, i * tseg + j2))
    s = Shape(v, f)
    if not full:           # close the ends
        s.f.append(tuple(reversed(range(tseg))))
        s.f.append(tuple(seg * tseg + j for j in range(tseg)))
    return _fix_winding(s)


def tube(points, r, seg=12, caps=True, smooth=0, r_end=None):
    """Sweep a circle along a polyline. smooth=k subdivides the path k times
    (Chaikin) first. r_end tapers the radius linearly to the last point."""
    pts = [Vector(p) for p in points]
    for _ in range(smooth):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    n = len(pts)
    tangents = []
    for i in range(n):
        t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)])
        tangents.append(t.normalized() if t.length > 1e-9 else Vector((0, 1, 0)))
    # parallel-transport frame
    up = Vector((0, 1, 0)) if abs(tangents[0].y) < 0.9 else Vector((1, 0, 0))
    nrm = tangents[0].cross(up).normalized()
    v, f = [], []
    for i in range(n):
        if i:
            q = tangents[i - 1].rotation_difference(tangents[i])
            nrm = (q @ nrm).normalized()
        bin_ = tangents[i].cross(nrm).normalized()
        ri = r if r_end is None else r + (r_end - r) * i / max(1, n - 1)
        for j in range(seg):
            a = TAU * j / seg
            v.append(pts[i] + (nrm * math.cos(a) + bin_ * math.sin(a)) * ri)
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    s = Shape(v, f)
    if caps:
        s.f.append(tuple(reversed(range(seg))))
        s.f.append(tuple((n - 1) * seg + j for j in range(seg)))
    return _fix_winding(s)


def prism(outline, depth, bevel_r=0.0, bevel_seg=1):
    """Extrude a 2D outline [(x, y), ...] (either winding) along Z, centred."""
    pts = [Vector((x, y, 0)) for x, y in outline]
    area = sum(a.x * b.y - b.x * a.y for a, b in zip(pts, pts[1:] + pts[:1]))
    if area < 0:
        pts.reverse()
    n = len(pts)
    z = depth / 2
    v = [p + Vector((0, 0, -z)) for p in pts] + [p + Vector((0, 0, z)) for p in pts]
    f = [tuple(reversed(range(n))), tuple(range(n, 2 * n))]
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
    s = Shape(v, f)
    return bevel(s, bevel_r, seg=bevel_seg) if bevel_r > 0 else s


def plane(w, h):
    """Flat quad in XY facing +Z (like the engine's `plane_geo!`)."""
    return Shape([(-w / 2, -h / 2, 0), (w / 2, -h / 2, 0), (w / 2, h / 2, 0), (-w / 2, h / 2, 0)], [(0, 1, 2, 3)])


def disc(r, seg=32):
    """Flat disc in XY facing +Z (like the engine's `circle_geo!`)."""
    v = [(0, 0, 0)] + [(r * math.cos(TAU * i / seg), r * math.sin(TAU * i / seg), 0) for i in range(seg)]
    return Shape(v, [(0, i + 1, (i + 1) % seg + 1) for i in range(seg)])


def grid_plane(w, h, nx=8, ny=8):
    """Subdivided XY quad facing +Z — for paper, cloth, crumpled sheets."""
    v, f = [], []
    for j in range(ny + 1):
        for i in range(nx + 1):
            v.append((-w / 2 + w * i / nx, -h / 2 + h * j / ny, 0))
    for j in range(ny):
        for i in range(nx):
            a = j * (nx + 1) + i
            f.append((a, a + 1, a + nx + 2, a + nx + 1))
    return Shape(v, f)


def slab(w, h, t, nx=8, ny=8):
    """Thin subdivided box (w × h in XY, thickness t along Z) that deforms
    nicely with noise/bend — paper stacks, cloth, cardboard."""
    top = grid_plane(w, h, nx, ny).move(z=t / 2)
    bot = grid_plane(w, h, nx, ny).rot(y=math.pi).move(z=-t / 2)
    s = top + bot
    # stitch the rims
    k = nx + 1
    n = len(top.v)
    rim_top = [i for i in range(nx + 1)] + [i * k + nx for i in range(1, ny + 1)] + \
              [ny * k + i for i in range(nx - 1, -1, -1)] + [i * k for i in range(ny - 1, 0, -1)]
    # bottom was rotated 180° about Y: vertex (x, y) -> (-x, y), same ids
    def mirror(i):
        j, ii = divmod(i, k)
        return n + j * k + (nx - ii)
    for a, b in zip(rim_top, rim_top[1:] + rim_top[:1]):
        s.f.append((b, a, mirror(a), mirror(b)))
    return _fix_winding(s)


# ------------------------------------------------------------- operators ---
def _round_corner(p, prev, nxt, r, seg):
    """Replace corner p of a 2D polyline with an arc of radius r."""
    p, a, b = Vector((p[0], p[1])), Vector((prev[0], prev[1])), Vector((nxt[0], nxt[1]))
    da, db = (a - p), (b - p)
    if da.length < 1e-9 or db.length < 1e-9:
        return [tuple(p)]
    da.normalize(); db.normalize()
    ang = math.acos(max(-1, min(1, da.dot(db))))
    if ang < 1e-3 or ang > math.pi - 1e-3:
        return [tuple(p)]
    t = r / math.tan(ang / 2)
    t = min(t, (Vector(prev) - p).length * 0.49, (Vector(nxt) - p).length * 0.49)
    rr = t * math.tan(ang / 2)
    p0, p1 = p + da * t, p + db * t
    bis = (da + db).normalized()
    c = p + bis * math.hypot(t, rr)
    a0 = math.atan2(p0.y - c.y, p0.x - c.x)
    a1 = math.atan2(p1.y - c.y, p1.x - c.x)
    d = (a1 - a0 + math.pi) % TAU - math.pi
    return [(c.x + rr * math.cos(a0 + d * i / seg), c.y + rr * math.sin(a0 + d * i / seg)) for i in range(seg + 1)]


def round_outline(outline, r, seg=3):
    """Round every corner of a 2D outline with radius r."""
    out = []
    n = len(outline)
    for i in range(n):
        out += _round_corner(outline[i], outline[i - 1], outline[(i + 1) % n], r, seg)
    return out


def rounded_rect(w, h, r, seg=3):
    return round_outline([(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2)], r, seg)


def bevel(s, r, seg=2, profile=0.5, angle=30):
    """Bevel every edge whose faces meet at more than `angle` degrees."""
    if r <= 0:
        return s
    bm = to_bmesh(s)
    lim = math.radians(angle)
    edges = [e for e in bm.edges if len(e.link_faces) == 2 and e.calc_face_angle(0) > lim]
    if edges:
        bmesh.ops.bevel(bm, geom=edges, offset=r, offset_type='OFFSET', segments=seg, profile=profile,
                        affect='EDGES', clamp_overlap=True)
    out = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


def subdivide(s, levels=1):
    """Catmull-Clark (Blender subsurf) — rounds everything; use on soft things."""
    ob = _mesh_obj('_sub', s)
    m = ob.modifiers.new('sub', 'SUBSURF')
    m.levels = levels
    m.render_levels = levels
    out = _eval_to_shape(ob)
    _drop(ob)
    return out


def boolean(a, b, op='DIFFERENCE'):
    """Exact boolean: 'DIFFERENCE' (a − b), 'UNION', 'INTERSECT'."""
    oa, ob = _mesh_obj('_ba', a), _mesh_obj('_bb', b)
    m = oa.modifiers.new('bool', 'BOOLEAN')
    m.operation = op
    m.object = ob
    m.solver = 'EXACT'
    ob.hide_viewport = True
    out = _eval_to_shape(oa)
    _drop(oa, ob)
    return out


def cut(a, *cutters):
    for c in cutters:
        a = boolean(a, c, 'DIFFERENCE')
    return a


def weld(s, dist=1e-5):
    bm = to_bmesh(s)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=dist)
    out = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


def noise3(q):
    """A 3D noise vector from three scalar Perlin samples at fixed offsets.
    `mathutils.noise.noise_vector` gives different values in different
    Blender processes (seen in 5.2), so a build using it never repeats;
    `noise.noise` does repeat."""
    return Vector((noise.noise(q), noise.noise(q + Vector((31.4, 17.9, 5.3))), noise.noise(q + Vector((-12.7, 47.1, 23.9)))))


def noisy(s, amount, scale=20.0, seed=0, axis=None):
    """Displace every vertex by 3D noise (amount in metres). axis=(x,y,z)
    restricts the push to a direction (e.g. (0,1,0) for a lumpy top)."""
    off = Vector((seed * 17.3, seed * 5.1, seed * 9.7))
    ax = Vector(axis).normalized() if axis else None
    out = s.copy()
    for i, p in enumerate(out.v):
        q = p * scale + off
        if ax:
            out.v[i] = p + ax * noise.noise(q) * amount
        else:
            out.v[i] = p + noise3(q) * amount
    return out


def bend(s, angle, axis='x', along='y'):
    """Bend around `axis`, distributing `angle` radians over the extent of
    `along` (e.g. a curled page, a slumped cushion)."""
    lo, hi = s.bounds()
    ai = 'xyz'.index(along)
    L = hi[ai] - lo[ai]
    if L <= 0 or abs(angle) < 1e-6:
        return s
    R = L / angle
    bi = 3 - ai - 'xyz'.index(axis)          # the axis the bend lifts into
    out = s.copy()
    for p in out.v:
        t = (p[ai] - lo[ai]) / L * angle
        u, w = p[ai] - lo[ai], p[bi]
        p[ai] = lo[ai] + (R - w) * math.sin(t)
        p[bi] = R - (R - w) * math.cos(t)
    return out


def taper(s, top=1.0, along='y', axes='xz'):
    lo, hi = s.bounds()
    ai = 'xyz'.index(along)
    out = s.copy()
    for p in out.v:
        t = (p[ai] - lo[ai]) / ((hi[ai] - lo[ai]) or 1)
        k = 1 + (top - 1) * t
        for a in axes:
            p['xyz'.index(a)] *= k
    return out


def repeat(s, n, step=(0, 0, 0), rot=(0, 0, 0)):
    out = Shape()
    for i in range(n):
        out = out + s.copy().rot(*(r * i for r in rot)).move(*(d * i for d in step))
    return out


def ring_of(s, n, radius, axis='y', start=0.0):
    """n copies of s placed around a circle (s is moved out along +X first)."""
    out = Shape()
    for i in range(n):
        a = start + TAU * i / n
        c = s.copy().move(x=radius)
        if axis == 'y':
            c.rot(y=a)
        elif axis == 'z':
            c.rot(z=a)
        else:
            c.rot(x=a)
        out = out + c
    return out


def screw_head(r=0.003, h=0.0012, slot=True):
    """A small domed screw/bolt head lying on +Y (base at y=0)."""
    s = lathe([(0, 0), (r, 0), (r, h * 0.35), (r * 0.7, h * 0.85), (0, h)], seg=10)
    if slot:
        s = cut(s, box(r * 2.4, h, r * 0.35).move(y=h * 0.9))
    return s


def _fix_winding(s):
    """Make faces point outwards (recalc normals) — keeps constructors honest."""
    bm = to_bmesh(s)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    out = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


# ---------------------------------------------------------------- frames ---
# The primitive a part replaces; used to check the part was built in the right
# frame and at the right size, and by the game to derive texture coordinates.
def f_box(w, h, d):
    return ('box', (-w / 2, -h / 2, -d / 2), (w / 2, h / 2, d / 2))


def f_cyl(rt, rb, h):
    r = max(rt, rb)
    return ('cyl', (-r, -h / 2, -r), (r, h / 2, r))


def f_cone(r, h):
    return ('cyl', (-r, -h / 2, -r), (r, h / 2, r))


def f_sphere(r):
    return ('sphere', (-r, -r, -r), (r, r, r))


def f_torus(R, r):
    return ('torus', (-R - r, -R - r, -r), (R + r, R + r, r))


def f_capsule(r, length):
    return ('cyl', (-r, -length / 2 - r, -r), (r, length / 2 + r, r))


def f_bounds(lo, hi, kind='other'):
    """Any other primitive (half spheres, lathes, shapes): give its bounds."""
    return (kind, tuple(lo), tuple(hi))


def budget_for(frame):
    """Triangle budget from the size of the thing."""
    lo, hi = Vector(frame[1]), Vector(frame[2])
    d = (hi - lo).length
    if d < 0.03:
        return 100
    if d < 0.12:
        return 280
    if d < 0.4:
        return 560
    if d < 1.2:
        return 1100
    return 1800


# ------------------------------------------------------------ finish/export
def finish(name, s, smooth_angle=35.0, weighted=True, flat=False):
    """Shape -> evaluated Blender mesh with final normals.
    Returns (positions [Vector], normals [Vector], triangles [(a,b,c)])."""
    ob = _mesh_obj(name, s)
    me = ob.data
    bm = bmesh.new()
    bm.from_mesh(me)
    lim = math.radians(smooth_angle)
    sharp = s.sharp
    for f in bm.faces:
        f.smooth = not flat
    bm.verts.ensure_lookup_table()
    for e in bm.edges:
        a, b = e.verts[0].index, e.verts[1].index
        forced = (a, b) in sharp or (b, a) in sharp
        e.smooth = not (forced or len(e.link_faces) != 2 or e.calc_face_angle(math.pi) > lim)
    bm.to_mesh(me)
    bm.free()
    if weighted and not flat:
        wn = ob.modifiers.new('wn', 'WEIGHTED_NORMAL')
        wn.keep_sharp = True
        wn.mode = 'FACE_AREA'
        wn.weight = 50
    dg = bpy.context.evaluated_depsgraph_get()
    em = bpy.data.meshes.new_from_object(ob.evaluated_get(dg), preserve_all_data_layers=True, depsgraph=dg)
    em.calc_loop_triangles()
    cn = em.corner_normals
    pos, nrm, tris = [], [], []
    key = {}
    for lt in em.loop_triangles:
        tri = []
        for li in lt.loops:
            vi = em.loops[li].vertex_index
            n = cn[li].vector
            k = (vi, round(n.x * 127), round(n.y * 127), round(n.z * 127))
            idx = key.get(k)
            if idx is None:
                idx = key[k] = len(pos)
                pos.append(em.vertices[vi].co.copy())
                nrm.append(n.copy())
            tri.append(idx)
        if len(set(tri)) == 3:
            tris.append(tuple(tri))
    bpy.data.meshes.remove(em)
    _drop(ob)
    return pos, nrm, tris
