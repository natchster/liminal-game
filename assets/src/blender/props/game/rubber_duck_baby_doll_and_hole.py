"""r03: the props remodelled at their real sizes (docs/design/PROPS.md), P1: the
rubber duck, the baby doll and the hole in the floor (core props).

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part's frame is
given with it. Sources and sizes: docs/research/props-remodelling/rubber_duck_baby_doll_and_hole.md.

duckBody, duckBill, duckEyeWhite, duckEyePupil
        the bath duck: x across, y up from the floor, z out of its front
        (the bill)
floorHoleMask, Cover, Ledge, Face, Deck, Wire, Shaft, Joist (under the
floor, drawn through it), Rubble, Dust (on it)
        the hole: centred on it, y up from the floor's surface
dollRomper, dollCollar, dollArms, dollLegs, dollHead, dollHair
        the doll lying on its back: head toward +x, face up, its right
        toward -z
dollFace, dollEyeR, dollEyeL, dollLids, dollLashes
        the head's own frame (origin under the neck, y up the head, z out
        of the face), laid down by the game (DOLL_NECK, DOLL_TURN)
"""
from propkit import *
from propkit import _mesh_obj, _eval_to_shape, _drop
import propkit
from mathutils.bvhtree import BVHTree

_meta_n = [0]
MB_K = 1.0 / 0.564


# ------------------------------------------------------------- helpers ---
def meta(elems, res=0.002, threshold=0.6):
    """A blobby surface from metaball elements (the kit's frame, metres):
    ('ball', centre, radius[, stiffness]), ('ell', centre, (rx, ry, rz)[,
    stiffness[, (ax, ay, az) euler]]), ('cap', a, b, radius[, stiffness]),
    and '-ball' / '-ell' subtract."""
    _meta_n[0] += 1
    name = f'_mb{_meta_n[0]}x'            # a family of its own (no '.')
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
        # sizes are given as the element's own surface (alone, at stiffness
        # 2 and threshold 0.6 an element's surface is 0.564 of its radius)
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
    ob = _mesh_obj('_dec', s)
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
    `up` its long axis) laid onto a surface: every vertex is cast back onto
    the surface along -normal and lifted `off` off it. Returns a Shape (a fan
    of rings), facing out of the surface."""
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
    # face out of the surface (the fan winds counter-clockwise about +n)
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


# ================================================================ duck ===
# The classic bath duck (Schylling's "Rubber Duckie", 3.5 x 3 x 3 in; the
# common 8-9 cm PVC duck): blow-moulded vinyl, hollow, a flat base, the tail
# turned up to a point, wings moulded in low relief, a painted bill and
# painted eyes. Its frame: x across, y up from the floor, z out of its front
# (the bill).
HEAD_C = Vector((0, 0.0585, 0.012))        # the head's centre


def duck_body_raw():
    e = [
        ('ell', (0, 0.022, -0.006), (0.033, 0.019, 0.034)),            # the hull
        ('ell', (0, 0.025, 0.014), (0.026, 0.020, 0.021), 1.8),        # the breast
        ('ell', (0, 0.031, -0.024), (0.018, 0.012, 0.012), 1.6),       # the rump, rising to the tail
        ('ell', (0, 0.039, -0.036), (0.0105, 0.017, 0.0075), 2.0, (-0.95, 0, 0)),  # the tail, turned up to a point
        ('ell', (0, 0.043, 0.008), (0.018, 0.013, 0.016), 1.8),        # the neck
        ('ell', tuple(HEAD_C), (0.0225, 0.0215, 0.021)),                # the head
    ]
    for sx in (-1, 1):
        e.append(('ball', (sx * 0.0095, 0.0525, 0.022), 0.009, 1.4))   # the cheeks
        # the wings: a low, long relief on each flank, swept back and up
        e.append(('ell', (sx * 0.026, 0.028, -0.012), (0.007, 0.010, 0.019), 1.6, (-0.30, sx * 0.15, 0)))
    s = meta(e, res=0.0013)
    s = flatten_below(s, 0.008, 0.10)
    return to_floor(s)


_duck_cache = {}


def duck_frame():
    """The body as built, with what the painted parts are laid on."""
    if 'body' not in _duck_cache:
        b = decimate(duck_body_raw(), 540)
        _duck_cache['body'] = b
        _duck_cache['bvh'] = bvh_of(b)
    return _duck_cache


def duck_body():
    return duck_frame()['body'].copy()


def duck_face():
    """Where the bill roots: cast from the head's centre forward and a
    little down."""
    hit = duck_frame()['bvh'].ray_cast(HEAD_C, Vector((0, -0.22, 1)).normalized(), 0.2)
    return hit[0]


def _loft(rings):
    """A closed skin through rings of points (each the same count), capped
    at both ends."""
    n = len(rings[0])
    v = [p.copy() for ring in rings for p in ring]
    f = []
    for k in range(len(rings) - 1):
        for i in range(n):
            j = (i + 1) % n
            f.append((k * n + i, k * n + j, (k + 1) * n + j, (k + 1) * n + i))
    c0 = sum(rings[0], Vector((0, 0, 0))) / n
    c1 = sum(rings[-1], Vector((0, 0, 0))) / n
    v += [c0, c1]
    a, b = len(v) - 2, len(v) - 1
    last = (len(rings) - 1) * n
    for i in range(n):
        j = (i + 1) % n
        f.append((a, j, i))
        f.append((b, last + i, last + j))
    s = Shape(v, f)
    centre = sum(v, Vector((0, 0, 0))) / len(v)
    return _fix_winding_out(s, centre)


def _mandible(f, length, w0, top0, top1, bot0, bot1, up, n_u=10, n_a=18):
    """One half of the bill: lofted along +z from its root at f (sunk 5 mm
    into the face), its planform broad and rounded at the tip, its section
    a flattened oval between its top and bottom lines, the tip turned up by
    `up`."""
    rings = []
    for k in range(n_u + 1):
        u = k / n_u
        z = -0.005 + (length + 0.005) * u
        tip = max(0.0, (z - length * 0.55) / (length * 0.45))
        w = w0 * math.sqrt(max(0.02, 1 - tip ** 2.4)) * (1 - 0.12 * u)
        top = top0 + (top1 - top0) * u + up * u * u
        bot = bot0 + (bot1 - bot0) * u + up * u * u
        cy, hh = (top + bot) / 2, max(0.0004, (top - bot) / 2)
        ring = []
        for i in range(n_a):
            a = TAU * i / n_a
            ca, sa = math.cos(a), math.sin(a)
            x = w * math.copysign(abs(ca) ** 0.6, ca)
            y = cy + hh * math.copysign(abs(sa) ** 0.8, sa)
            ring.append(Vector((x, f.y + y, f.z + z)))
        rings.append(ring)
    return _loft(rings)


def duck_bill():
    """The bill: the upper mandible broad, flat and rounded, overhanging
    and turned up a little at its tip; the lower one narrower and thinner
    under it, the gape a groove between; its root sunk into the face."""
    f = duck_face()
    upper = _mandible(f, 0.0150, 0.0114, 0.0056, 0.0021, -0.0004, -0.0002, 0.0014)
    lower = _mandible(f, 0.0112, 0.0090, -0.0008, -0.0012, -0.0062, -0.0030, 0.0004)
    return decimate(upper + lower, 270)


def duck_eye_part(kind):
    """The painted eyes, laid on the head: the white (an oval, a little
    tilted) and the black pupil inside it, toward the front."""
    bvh = duck_frame()['bvh']
    out = Shape()
    for sx in (-1, 1):
        dirn = Vector((sx * math.sin(math.radians(48)), math.sin(math.radians(22)), math.cos(math.radians(48)))).normalized()
        hit = bvh.ray_cast(HEAD_C, dirn, 0.2)
        p, n = hit[0], hit[1].normalized()
        if kind == 'white':
            s = project_patch(bvh, p, n, (0, 1, 0.25), 0.0098, 0.0124, 0.00025, seg=14, rings=2)
        else:
            q = p + Vector((0, 0.0012, 0.0019))
            s = project_patch(bvh, q, n, (0, 1, 0.2), 0.0066, 0.0084, 0.0005, seg=12, rings=2)
        out = out + s
    return out


# ========================================================== floor hole ===
# A hole broken through an office floor: the covering (carpet, carpet tile or
# vinyl, 10 mm) torn back, the slab under it (lightweight concrete on a 2 in
# composite steel deck, 150 mm in all, 6x6-W1.4xW1.4 welded wire mesh near
# its top) broken through, the deck torn and bent down, the mesh's wires cut
# and bent, rubble and dust at the rim, nothing but dark below. Its frame:
# centred on the hole, y up from the floor's surface (y = 0); x, z across.
# The game draws what is under the floor first, without depth, deepest first,
# then a mask over the opening (depth only) so the floor is not drawn there.
import random as _random

HOLE_N = 96                 # points round the outline
SLAB_TOP = -0.010           # the concrete's top, under the floor covering
DECK_HI, DECK_LO = -0.100, -0.150   # the deck's flats: 2 in ribs
DECK_PITCH = 0.305          # 12 in
WIRE_PITCH = 0.152          # the mesh: 6 in
WIRE_R = 0.0017             # W1.4: 3.4 mm wire
SQUARE = 0.80               # what stands on the floor stays inside |x|, |z| <= this

_hole = {}


def _periodic(rng, n, amp, harmonics):
    """A smooth closed curve's offsets at n angles (a few random harmonics)."""
    terms = [(k, rng.uniform(0, TAU), amp * rng.uniform(0.5, 1.0) / k ** 0.7) for k in range(1, harmonics + 1)]
    return [sum(a * math.sin(k * TAU * i / n + ph) for k, ph, a in terms) for i in range(n)]


def deck_y(z):
    """The deck's top along its ribs' cross-section (the ribs run along x)."""
    t = (z / DECK_PITCH) % 1.0
    if t < 0.32:
        return DECK_LO
    if t < 0.45:
        return DECK_LO + (DECK_HI - DECK_LO) * (t - 0.32) / 0.13
    if t < 0.82:
        return DECK_HI
    if t < 0.95:
        return DECK_HI + (DECK_LO - DECK_HI) * (t - 0.82) / 0.13
    return DECK_LO


def hole_plan():
    """The outlines, shared by every part of the hole."""
    if _hole:
        return _hole
    rng = _random.Random(4111)
    n = HOLE_N
    th = [TAU * i / n for i in range(n)]
    low = _periodic(rng, n, 0.045, 5)
    # broken, not cut: bites where pieces came away, and a jagged edge
    mid = [0.0] * n
    for k in range(6, 15):
        a, ph = rng.uniform(0.004, 0.014), rng.uniform(0, TAU)
        for i in range(n):
            mid[i] += a * math.sin(k * TAU * i / n + ph)
    for _ in range(7):
        c, wdt, dep = rng.uniform(0, n), rng.uniform(1.2, 3.5), rng.uniform(0.025, 0.07)
        for i in range(n):
            d = min(abs(i - c), n - abs(i - c))
            mid[i] += dep * math.exp(-(d / wdt) ** 2)
    r_cover = [min(0.78, max(0.60, 0.69 + low[i] + mid[i] + rng.uniform(-0.016, 0.016))) for i in range(n)]
    # the concrete broke short of the covering's tear in places: a ledge
    led = _periodic(rng, n, 1.0, 3)
    ledge = [max(0.0, 0.04 * led[i] + rng.uniform(-0.012, 0.018)) for i in range(n)]
    r_break = [r_cover[i] - ledge[i] for i in range(n)]
    bot = _periodic(rng, n, 0.02, 4)
    r_bot = [r_break[i] + bot[i] + rng.uniform(-0.015, 0.015) for i in range(n)]
    # the deck's torn lip: short of the concrete, or hanging into the hole
    lip_lf = _periodic(rng, n, 1.0, 4)
    lip = [max(0.0, 0.07 * lip_lf[i] + rng.uniform(-0.01, 0.03)) for i in range(n)]
    bend = [0.25 + 1.1 * min(1.0, lip[i] / 0.07) + rng.uniform(-0.1, 0.2) for i in range(n)]
    _hole.update(rng=rng, n=n, th=th, r_cover=r_cover, r_break=r_break, r_bot=r_bot, lip=lip, bend=bend)
    return _hole


def _band(rows, inward=True):
    """Quads between rings of points (rows[k][i], each a Vector), closed
    round; facing the hole's centre (inward) or up."""
    v, f = [], []
    n = len(rows[0])
    for row in rows:
        v += [p.copy() for p in row]
    for k in range(len(rows) - 1):
        for i in range(n):
            j = (i + 1) % n
            f.append((k * n + i, k * n + j, (k + 1) * n + j, (k + 1) * n + i))
    s = Shape(v, f)
    # orient: the first quad's normal toward the centre (inward) or up
    a, b, c = s.v[f[0][0]], s.v[f[0][1]], s.v[f[0][2]]
    nrm = (b - a).cross(c - a)
    mid = (a + b + c) / 3
    want = Vector((-mid.x, 0, -mid.z)) if inward else Vector((0, 1, 0))
    if nrm.dot(want) < 0:
        s.f = [tuple(reversed(q)) for q in s.f]
    return s


def hole_mask():
    """The opening at the floor's surface (drawn into depth only)."""
    h = hole_plan()
    v = [Vector((0, 0.0015, 0))] + [Vector((math.cos(a) * r, 0.0015, math.sin(a) * r)) for a, r in zip(h['th'], h['r_cover'])]
    n = h['n']
    f = [(0, 1 + (i + 1) % n, 1 + i) for i in range(n)]
    s = Shape(v, f)
    a, b, c = (s.v[x] for x in f[0])
    if (b - a).cross(c - a).y < 0:
        s.f = [tuple(reversed(q)) for q in s.f]
    return s


def hole_cover():
    """The covering's torn edge: 10 mm of it, ragged, from the floor's
    surface down to the concrete."""
    h = hole_plan()
    top, mid, low = [], [], []
    for i, (a, r) in enumerate(zip(h['th'], h['r_cover'])):
        c, s_ = math.cos(a), math.sin(a)
        fr = 0.004 * math.sin(i * 2.7) + 0.003      # frayed: the backing sticks out under the pile
        top.append(Vector((c * r, 0.0, s_ * r)))
        mid.append(Vector((c * (r - fr * 0.4), -0.005, s_ * (r - fr * 0.4))))
        low.append(Vector((c * (r - fr), SLAB_TOP + 0.0005, s_ * (r - fr))))
    return _band([top, mid, low])


def hole_ledge():
    """The concrete's top, bared where the covering tore back further than
    the slab broke (it runs on under the floor)."""
    h = hole_plan()
    rows = []
    for k, t in enumerate((0.0, 0.5, 1.0)):
        row = []
        for i, a in enumerate(h['th']):
            r0, r1 = h['r_break'][i], h['r_cover'][i] + 0.05
            r = r0 + (r1 - r0) * t
            y = SLAB_TOP + (0.0015 * noise.noise(Vector((math.cos(a) * 9, t * 3, math.sin(a) * 9))) if k < 2 else 0)
            row.append(Vector((math.cos(a) * r, y, math.sin(a) * r)))
        rows.append(row)
    return _band(rows, inward=False)


def hole_face():
    """The slab's broken face: rough, from its top down to the deck (whose
    ribs it follows), a little in and out as concrete breaks."""
    h = hole_plan()
    K = 5
    rows = []
    for k in range(K + 1):
        t = k / K
        row = []
        for i, a in enumerate(h['th']):
            c, s_ = math.cos(a), math.sin(a)
            r = h['r_break'][i] + (h['r_bot'][i] - h['r_break'][i]) * t
            yb = deck_y(s_ * h['r_bot'][i])
            y = SLAB_TOP + (yb - SLAB_TOP) * t
            if 0 < k < K:
                q = Vector((c * 7.0, y * 14.0, s_ * 7.0))
                r += 0.022 * noise.noise(q) + 0.010 * noise.noise(q * 3.1)
                y += 0.008 * noise.noise(q + Vector((7.7, 1.3, 2.9)))
            row.append(Vector((c * r, y, s_ * r)))
        rows.append(row)
    return _band(rows)


def hole_deck():
    """The steel deck under the slab, along its ribs: torn at the hole, its
    lip bent down into it where the concrete went (one sheet, both faces
    drawn)."""
    h = hole_plan()
    rows = []
    # from the torn lip out past the concrete's foot (hidden beyond)
    for k, t in enumerate((0.0, 0.35, 0.7, 1.0, 1.6)):
        row = []
        for i, a in enumerate(h['th']):
            c, s_ = math.cos(a), math.sin(a)
            hinge = h['r_bot'][i] + 0.004
            lip = max(0.010, h['lip'][i] * (1.0 + 0.3 * math.sin(i * 1.9)))   # (a short lip at least: no row collapses)
            if t <= 1.0:
                d = lip * (1.0 - t)                      # how far in from the hinge
                b = h['bend'][i] * (1.0 - 0.3 * t)
                r = hinge - d * math.cos(b)
                y = deck_y(s_ * hinge) - d * math.sin(b) - 0.001
                if k == 0:                               # torn: a jagged edge
                    r += 0.008 * math.sin(i * 4.3) * (lip > 0.01)
            else:
                r = hinge + 0.12
                y = deck_y(s_ * r) - 0.001
            row.append(Vector((c * r, y, s_ * r)))
        rows.append(row)
    return _band(rows, inward=False)


def hole_wires():
    """The mesh's wires, cut where the slab broke: stubs out of the face,
    bent down, some short, some long."""
    h = hole_plan()
    rng = _random.Random(4127)
    bvh = bvh_of(hole_face())
    out = Shape()
    rmin = min(h['r_break'])
    for axis, y0 in (('x', -0.030), ('z', -0.0345)):
        k0 = int(rmin / WIRE_PITCH) + 1
        for k in range(-k0, k0 + 1):
            off = (k + 0.37) * WIRE_PITCH
            for sgn in (-1, 1):
                if rng.random() < 0.25:
                    rng.random()
                    continue
                start = Vector((0, y0, off)) if axis == 'x' else Vector((off, y0, 0))
                d = Vector((sgn, 0, 0)) if axis == 'x' else Vector((0, 0, sgn))
                hit = bvh.ray_cast(start, d, 2.0)
                if hit[0] is None:
                    continue
                p0 = hit[0] + d * 0.002
                L = 0.02 + rng.random() ** 1.6 * 0.20
                droop = rng.uniform(0.0, 1.0) * L * 0.8
                side = rng.uniform(-0.3, 0.3) * L
                lat = Vector((-d.z, 0, d.x))
                pts = []
                for j in range(4):
                    u = j / 3
                    pts.append(p0 - d * L * u + Vector((0, -droop * u * u, 0)) + lat * side * u * u)
                out = out + tube(pts, WIRE_R, seg=4, caps=True)
    return out


def hole_shaft():
    """The dark below: a box open at the top, its faces inward."""
    b = box(2.5, 2.5, 2.5).move(0, -0.12 - 1.25, 0)
    b.f = [tuple(reversed(f)) for f in b.f]
    return b


def _chunk(rng, size, flat=True):
    """A broken lump of concrete: the hull of a few points in a squat
    ellipsoid, all facets (fractures), and one face flat where it broke off
    the slab's top."""
    sx, sy, sz = size, size * rng.uniform(0.35, 0.6), size * rng.uniform(0.55, 0.95)
    pts = []
    for _ in range(rng.randint(8, 11)):
        while True:
            q = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
            if q.length <= 1:
                break
        pts.append(Vector((q.x * sx / 2, q.y * sy / 2, q.z * sz / 2)))
    if flat:
        top = max(p.y for p in pts) * rng.uniform(0.55, 0.8)
        pts = [Vector((p.x, min(p.y, top), p.z)) for p in pts]
    bm = bmesh.new()
    for p in pts:
        bm.verts.new(p)
    bmesh.ops.convex_hull(bm, input=list(bm.verts))
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context='VERTS')
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=size * 0.02)
    s = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    c = sum(s.v, Vector((0, 0, 0))) / len(s.v)
    return _fix_winding_out(s, c)


def hole_rubble():
    """Concrete broken off the rim, lying round it on the floor: grit and
    small pieces, a few big ones, more on the side where the slab gave
    first; each resting on the floor on one of its faces."""
    h = hole_plan()
    rng = _random.Random(4133)
    out = Shape()
    n = h['n']
    placed, tries = 0, 0
    while placed < 64 and tries < 900:
        tries += 1
        a = rng.gauss(2.2, 0.8) if rng.random() < 0.6 else rng.uniform(0, TAU)
        a %= TAU
        i = int(a / TAU * n) % n
        grit = placed >= 44
        size = rng.uniform(0.004, 0.010) if grit else 0.014 + rng.random() ** 2.2 * 0.12
        r = h['r_cover'][i] + 0.006 + size * 0.45 + rng.random() ** 1.8 * (0.30 if grit else 0.20)
        x, z = math.cos(a) * r, math.sin(a) * r
        if abs(x) + size * 0.6 > SQUARE or abs(z) + size * 0.6 > SQUARE:
            continue
        ch = _chunk(rng, size, flat=not grit)
        # lying on any face: a tilt, and turned about y
        ch.rot(rng.uniform(-0.5, 0.5) if size > 0.05 else rng.uniform(-1.2, 1.2), rng.uniform(0, TAU), rng.uniform(-0.4, 0.4))
        lo, _ = ch.bounds()
        ch.move(x, -lo.y, z)
        out = out + ch
        placed += 1
    return out


JOIST_X = 0.47                 # the joist's line under the hole (it runs along z)
JOIST_D = 0.406                # a 16K: 16 in deep


def hole_joist():
    """An open-web steel joist under the deck, its line across one side of
    the hole: the top chord (two angles back to back, the deck bearing on
    them), the bottom chord, the round-bar web zigzagging between, shop
    primer gone dark in the shadow under the floor."""
    out = Shape()
    L = 2.4
    yt, yb = DECK_LO, DECK_LO - JOIST_D

    def angle(leg, t, y, side, up):
        # an L: one leg flat (out to `side`), one standing (up or down)
        flat = box(leg, t, L).move(JOIST_X + side * (0.013 + leg / 2), y + (t / 2 if up else -t / 2), 0)
        stand = box(t, leg, L).move(JOIST_X + side * (0.013 + t / 2), y + (leg / 2 if up else -leg / 2), 0)
        return flat + stand
    for side in (-1, 1):
        out = out + angle(0.038, 0.004, yt, side, False)     # the top chord hangs from the deck
        out = out + angle(0.032, 0.004, yb, side, True)
    pts = []
    for k in range(9):
        z = -L / 2 + 0.05 + (L - 0.1) * k / 8
        pts.append(Vector((JOIST_X, yt - 0.03 if k % 2 == 0 else yb + 0.026, z)))
    out = out + tube(pts, 0.010, seg=6, caps=True)
    return out


def hole_dust():
    """Concrete dust and grit round the rim, on the floor (a decal ring from
    the opening out)."""
    h = hole_plan()
    inner = [Vector((math.cos(a) * r, 0.002, math.sin(a) * r)) for a, r in zip(h['th'], h['r_cover'])]
    outer = [Vector((math.cos(a) * 1.3, 0.002, math.sin(a) * 1.3)) for a in h['th']]
    return _band([inner, outer], inward=False)


# =========================================================== baby doll ===
# A bent-limb play baby doll of the 1970s-80s, 38 cm (15 in), after Palitoy's
# Tiny Tears of 1978 in the V&A (soft vinyl, sleep eyes with lashes, an open
# mouth for its bottle, creases at the wrists) with the moulded, painted hair
# of the cheaper babies: a soft-vinyl head as wide as its shoulders, its
# cheeks full, sleep eyes (balls with lids that close when it lies down: one
# has stuck half open); chubby bent vinyl arms and legs; a cloth body in a
# cotton romper with puffed sleeves, elastic leg bands, a Peter Pan collar
# and two buttons (docs/research/props-remodelling/rubber_duck_baby_doll_and_hole.md).
#
# Two frames. The body's (`dollRomper`, `dollCollar`, `dollArms`,
# `dollLegs`, and `dollHead`, `dollHair` laid into it): the doll lying on its
# back, its head toward +x, its face up (+y), its right side toward -z. The
# head's own (`dollFace`, `dollEye*`, `dollLids`, `dollLashes`, which the
# game lays down by DOLL_NECK and DOLL_TURN): its origin under the neck, y up
# the head, z out of the face. (What is big is laid down here: the placement
# pass rests a prop on the lowest corner of each part's box, and a big box
# turned dips under the floor.)
DOLL_NECK = Vector((0.050, 0.0592, 0.0))  # the head frame's origin in the body's
DOLL_TURN = -0.30                         # the head rolled toward its right (radians about x)
FLOOR_Y = 0.0005                          # what rests on the floor is flattened onto it here

# the head (its own frame): a radial sculpt about HC, an ellipsoid with soft
# bumps and dents where a baby's face has them
HC = Vector((0.0, 0.062, -0.004))
HEAD_R = (0.0465, 0.0505, 0.0505)
NECK_R, NECK_Z, NECK_Y0 = 0.019, -0.009, -0.012   # the neck: a stub into the body (its axis, its foot)
EYE_R = 0.0108
EYE_X, EYE_Y = 0.0190, 0.0585            # the eyes' centres (x either side), height
EYE_W, EYE_T, EYE_B = 0.0094, 0.0060, 0.0046   # the opening: half width, top, bottom
LID_OPEN = (0.22, 0.52)                  # right shut, left stuck half open
MOUTH = (0.0, 0.0332)                    # the bottle's hole


def _feat(target, amp, su, sv):
    f = (Vector(target) - HC).normalized()
    e1 = Vector((0, 1, 0)).cross(f)
    e1 = e1.normalized() if e1.length > 1e-6 else Vector((1, 0, 0))
    return (f, e1, f.cross(e1).normalized(), amp, su, sv)


HEAD_FEATS = [
    _feat((0, 0.090, 0.040), 0.0035, 0.60, 0.32),          # the forehead, full
    _feat((0, 0.064, -0.056), 0.0030, 0.80, 0.55),         # the back of the skull
    _feat((0, 0.021, 0.034), 0.0018, 0.24, 0.16),          # the chin
    _feat((0, 0.006, 0.020), -0.0020, 0.45, 0.16),         # under the chin, tucked
    _feat((0, 0.0470, 0.047), 0.0066, 0.085, 0.095),       # the nose, a button
    _feat((0, 0.0538, 0.046), 0.0012, 0.07, 0.08),         # its bridge, low
    _feat((0, 0.0372, 0.045), 0.0028, 0.13, 0.060),        # the upper lip, pursed
    _feat((0, 0.0294, 0.043), 0.0026, 0.11, 0.055),        # the lower lip
]
for _sx in (-1, 1):
    HEAD_FEATS += [
        _feat((_sx * 0.025, 0.037, 0.036), 0.0078, 0.32, 0.27),     # the cheeks, fat
        _feat((_sx * 0.030, 0.026, 0.026), 0.0020, 0.34, 0.22),     # the jowls
        _feat((_sx * 0.0045, 0.0455, 0.047), 0.0018, 0.05, 0.05),   # the nostrils' wings
        _feat((_sx * 0.019, 0.059, 0.042), -0.0032, 0.22, 0.13),    # the eye's socket
        _feat((_sx * 0.020, 0.0680, 0.042), 0.0012, 0.26, 0.055),   # the fold over it
        _feat((_sx * 0.0465, 0.050, -0.010), 0.0060, 0.10, 0.19),   # the ears
    ]


def head_r(d):
    """The head's radius from HC along the unit direction d."""
    a, b, c = HEAD_R
    if d.y < 0:                       # a baby's lower face is short
        b -= 0.0075 * min(1.0, -d.y / 0.7)
    r = 1.0 / math.sqrt((d.x / a) ** 2 + (d.y / b) ** 2 + (d.z / c) ** 2)
    for f, e1, e2, amp, su, sv in HEAD_FEATS:
        cf = d.dot(f)
        if cf <= 0:
            continue
        du, dv = math.atan2(d.dot(e1), cf), math.atan2(d.dot(e2), cf)
        r += amp * math.exp(-(du / su) ** 2 - (dv / sv) ** 2)
    # the neck: a stub down into the body, blended in
    if d.y < -0.05:
        t = (HC.y - NECK_Y0) / -d.y
        # its side: a cylinder about a vertical axis behind HC
        qa = d.x * d.x + d.z * d.z
        if qa > 1e-9:
            pz = HC.z - NECK_Z
            qb = 2 * pz * d.z
            qc = pz * pz - NECK_R * NECK_R
            t = min(t, (-qb + math.sqrt(qb * qb - 4 * qa * qc)) / (2 * qa))
        r = (r ** 6 + t ** 6) ** (1 / 6)
    return r


def head_point(d, off=0.0):
    return HC + d * (head_r(d) + off)


def _dir(az, el):
    """A direction from its azimuth (0 = the face, +x its left) and
    elevation."""
    return Vector((math.sin(az) * math.cos(el), math.sin(el), math.cos(az) * math.cos(el)))


def _sphere_grid(seg, rings, fn):
    """A closed grid of the directions round HC (poles up and down), each
    vertex fn(direction)."""
    v = [fn(Vector((0, -1, 0)))]
    for j in range(1, rings):
        el = -math.pi / 2 + math.pi * j / rings
        for i in range(seg):
            v.append(fn(_dir(TAU * i / seg, el)))
    v.append(fn(Vector((0, 1, 0))))
    f = []
    top = len(v) - 1
    for i in range(seg):
        i2 = (i + 1) % seg
        f.append((0, 1 + i2, 1 + i))
        f.append((top, 1 + (rings - 2) * seg + i, 1 + (rings - 2) * seg + i2))
    for j in range(rings - 2):
        for i in range(seg):
            i2 = (i + 1) % seg
            a, b = 1 + j * seg, 1 + (j + 1) * seg
            f.append((a + i, a + i2, b + i2, b + i))
    return _fix_winding_out(Shape(v, f), HC)


def _fix_winding_out(s, c):
    """Every face wound to face away from c."""
    out = []
    for q in s.f:
        p0, p1, p2 = s.v[q[0]], s.v[q[1]], s.v[q[2]]
        n = (p1 - p0).cross(p2 - p0)
        out.append(q if n.dot(p0 - c) >= 0 else tuple(reversed(q)))
    s.f = out
    return s


def eye_centre(i):
    """The eyeball's centre: its front a little behind the face's surface."""
    sx = -1 if i == 0 else 1
    x, y = sx * EYE_X, EYE_Y
    # the face's surface in front of the eye (a ray along z from inside)
    lo, hi = 0.0, 0.08
    for _ in range(40):
        z = (lo + hi) / 2
        p = Vector((x, y, z))
        d = (p - HC)
        if d.length < head_r(d.normalized()):
            lo = z
        else:
            hi = z
    return Vector((x, y, lo - 0.0006 - EYE_R))


def _almond(n=24, grow=0.0):
    """The eye's opening (x, y about its centre), the outer corner a little
    up."""
    pts = []
    for k in range(n):
        t = k / n * TAU
        x = math.cos(t) * (EYE_W + grow)
        u = max(0.0, 1 - (x / (EYE_W + grow)) ** 2)
        y = ((EYE_T if math.sin(t) > 0 else -EYE_B) + (grow if math.sin(t) > 0 else -grow)) * u ** 0.6
        pts.append((x, y))
    return pts


def doll_head_raw():
    s = _sphere_grid(72, 48, head_point)
    for i in range(2):
        sx = -1 if i == 0 else 1
        c = eye_centre(i)
        pts = [(x, y + sx * x * 0.10) for x, y in _almond()]     # tilted: the outer corner up
        cutter = prism(pts, 0.05).move(c.x, c.y, c.z + 0.026)
        s = boolean(s, cutter, 'DIFFERENCE')
    # the bottle's hole, a round "o" between the lips
    mouth = cyl(0.0026, h=0.03, seg=12, bevel_r=0).rot(math.pi / 2, 0, 0).move(MOUTH[0], MOUTH[1], 0.051)
    return boolean(s, mouth, 'DIFFERENCE')


_doll = {}


def doll_head_mesh():
    """The whole head in its own frame, decimated once; split in two below
    (each a part of its own, one material each)."""
    if 'head' not in _doll:
        _doll['head'] = decimate(doll_head_raw(), 1045)
    return _doll['head']


def _face_side(s, f):
    """A face of the head that belongs to the painted face: in front of the
    ears and under the hairline."""
    c = sum((s.v[i] for i in f), Vector((0, 0, 0))) / len(f)
    return c.z > -0.006 and c.y < 0.093


def _head_half(face):
    s = doll_head_mesh()
    keep = [f for f in s.f if _face_side(s, f) == face]
    used = sorted({i for f in keep for i in f})
    remap = {i: k for k, i in enumerate(used)}
    return Shape([s.v[i].copy() for i in used], [tuple(remap[i] for i in f) for f in keep])


def doll_head():
    """The head but its face (the skull, the ears, the neck), laid into the
    body's frame (resting on the floor)."""
    return flatten_below(_head_half(False).warp(head_to_body), FLOOR_Y, 0.0)


def doll_face():
    """The face (its own frame; its paint laid on straight along z)."""
    return _head_half(True)


def doll_eye(i):
    """A sleep eye's ball: white, its iris painted forward (+z)."""
    return sphere(EYE_R, seg=14, rings=8).move(*eye_centre(i))


def _lid_line(i, x):
    """The lid's edge (y about the eye's centre) at x across the opening."""
    o = LID_OPEN[i]
    y = -EYE_B + (EYE_T + EYE_B) * o
    return y - 0.0009 * (1 - (x / EYE_W) ** 2) + (-1 if i == 0 else 1) * x * 0.10


def _on_ball(c, x, y, r):
    q = r * r - x * x - y * y
    return Vector((c.x + x, c.y + y, c.z + math.sqrt(max(q, 0.0))))


def doll_lids():
    """The lids of the sleep eyes: from their edge up over the top of the
    ball, filling the opening (and tucked under its rim), the right one
    shut, the left stuck half open; the edge rolled."""
    out = Shape()
    r = EYE_R + 0.0005
    nx, ny = 11, 5
    for i in range(2):
        c = eye_centre(i)
        v, f = [], []
        xs = [(-EYE_W - 0.0008) + (2 * EYE_W + 0.0016) * k / nx for k in range(nx + 1)]
        top = EYE_T + 0.0012
        for j in range(ny + 1):
            for x in xs:
                y0 = _lid_line(i, max(-EYE_W, min(EYE_W, x)))
                y = y0 + (top + abs(x) * 0.1 - y0) * (j / ny) ** 1.3
                v.append(_on_ball(c, x, y, r))
        for j in range(ny):
            for k in range(nx):
                a = j * (nx + 1) + k
                f.append((a, a + 1, a + nx + 2, a + nx + 1))
        # the edge, rolled under toward the ball
        base = len(v)
        for x in xs:
            y0 = _lid_line(i, max(-EYE_W, min(EYE_W, x)))
            p = _on_ball(c, x, y0, r)
            v.append(p + (c - p).normalized() * 0.0009 + Vector((0, 0.0002, 0)))
        for k in range(nx):
            f.append((base + k, base + k + 1, k + 1, k))
        s = _fix_winding_out(Shape(v, f), c)
        out = out + s
    return out


def doll_lashes():
    """Each lid's lashes (nylon, moulded in a strip): fine dark blades from
    its edge, lying down on the cheek from the shut eye, standing out from
    the half-open one."""
    out = Shape()
    r = EYE_R + 0.0006
    for i in range(2):
        c = eye_centre(i)
        n = 15
        v, f = [], []
        for k in range(n):
            x = -EYE_W * 0.86 + 2 * EYE_W * 0.86 * k / (n - 1)
            half = 0.00055
            b0 = _on_ball(c, x - half, _lid_line(i, x - half), r)
            b1 = _on_ball(c, x + half, _lid_line(i, x + half), r)
            base = (b0 + b1) / 2
            nrm = (base - c).normalized()
            down = Vector((0, -1, 0))
            d = (nrm * 0.80 + down * 0.60) if i == 0 else (nrm * 1.0 + Vector((0, 0.25, 0)))
            d = (d + Vector((x / EYE_W * 0.35, 0, 0))).normalized()   # fanned out at the corners
            ln = 0.0046 * (0.7 + 0.3 * math.cos(x / EYE_W * 1.3))
            v += [b0, b1, base + d * ln]
            f.append((3 * k, 3 * k + 1, 3 * k + 2))
        out = out + Shape(v, f)
    return out


def _hairline(az):
    """The hairline's elevation (radians) round the head (az 0: the face):
    high on the brow with a lock brought forward, over the ears, low at the
    nape; its edge uneven, as moulded locks end."""
    k = (1 - math.cos(az)) / 2                       # 0 at the brow, 1 at the nape
    e = math.radians(31) + (math.radians(-38) - math.radians(31)) * k ** 0.85
    e -= math.radians(12) * math.exp(-((az - 0.16) / 0.20) ** 2)   # the forelock
    e += math.radians(2.5) * math.sin(az * 9 + 1.3) + math.radians(1.5) * math.sin(az * 17)
    # over the ears the hair is swept back above them
    e += math.radians(8) * math.exp(-((abs(az) - math.radians(95)) / 0.35) ** 2)
    return e


def doll_hair():
    """Moulded hair: a shell 1.5-2.5 mm proud of the scalp above the
    hairline, thinning to nothing at its edge, in broad combed waves from the
    crown (laid into the body's frame)."""
    na, nt = 56, 12
    v = [head_point(Vector((0, 1, 0)), 0.0026)]
    rows = []
    for k in range(na):
        az = -math.pi + TAU * k / na
        eh = _hairline(az)
        col = []
        for j in range(1, nt + 1):
            t = j / nt
            el = math.pi / 2 + (eh - math.pi / 2) * t
            d = _dir(az, el)
            edge = min(1.0, (1 - t) * nt / 2.5)
            wave = 0.5 + 0.5 * math.sin(az * 13 + el * 6 + 1.7 * noise.noise(d * 6))
            col.append(head_point(d, 0.0004 + edge * (0.0013 + 0.0009 * wave)))
        rows.append(col)
    for col in rows:
        v += col
    f = []
    for k in range(na):
        k2 = (k + 1) % na
        f.append((0, 1 + k2 * nt, 1 + k * nt))
        for j in range(nt - 1):
            a0, b0 = 1 + k * nt + j, 1 + k2 * nt + j
            f.append((a0, b0, b0 + 1, a0 + 1))
    sh = _fix_winding_out(Shape(v, f), HC)
    sh = decimate(sh, 540)
    return flatten_below(sh.warp(head_to_body), FLOOR_Y, 0.0)


# the body's joints (lying frame): shoulders, elbows, wrists; hips, knees, ankles
ARMS = [  # (shoulder, elbow, wrist, palm normal)
    (Vector((0.044, 0.036, -0.043)), Vector((0.015, 0.017, -0.075)), Vector((-0.013, 0.017, -0.082)), Vector((0.1, -0.5, 0.85))),
    (Vector((0.044, 0.036, 0.043)), Vector((0.068, 0.017, 0.078)), Vector((0.100, 0.016, 0.083)), Vector((0.0, 0.55, -0.85))),
]
LEGS = [  # (hip, knee, ankle, toe direction)
    (Vector((-0.050, 0.030, -0.025)), Vector((-0.096, 0.022, -0.054)), Vector((-0.140, 0.019, -0.058)), Vector((-0.15, 0.55, -0.82))),
    (Vector((-0.050, 0.030, 0.025)), Vector((-0.104, 0.022, 0.038)), Vector((-0.152, 0.019, 0.042)), Vector((-0.2, 0.92, 0.33))),
]


def _frame(d, n):
    """Orthonormal axes: along d, with n (made perpendicular) as 'up'."""
    d = d.normalized()
    n = (n - d * n.dot(d)).normalized()
    return d, n, d.cross(n).normalized()


def _ell_on(c, axes, half, stiff=2.0):
    """An ellipsoid element at c whose half sizes run along the given axes."""
    m = Matrix((axes[0], axes[1], axes[2])).transposed()
    return ('ell', tuple(c), tuple(half), stiff, tuple(m.to_euler('XYZ')))


def doll_arm(i):
    s_, e_, w_, pn = ARMS[i]
    start = s_.lerp(e_, 0.42)                     # it comes out of the sleeve here
    el = [('cap', tuple(start), tuple(e_), 0.0142, 2.0), ('cap', tuple(e_), tuple(w_), 0.0130, 2.0),
          ('ball', tuple(e_), 0.0146, 1.6), ('ball', tuple(e_.lerp(w_, 0.45)), 0.0142, 1.5)]
    d, n, sd = _frame(w_ - e_, pn)
    palm = w_ + d * 0.016
    el.append(('ball', tuple(w_ - d * 0.001), 0.0098, 1.5))                 # the wrist, a crease either side
    el.append(_ell_on(palm, (sd, d, n), (0.0118, 0.0135, 0.0062)))
    for k in range(4):                                                     # the fingers, moulded together, curled
        off = (k - 1.5) * 0.0056
        b = palm + d * 0.009 + sd * off
        t = b + d * (0.010 - abs(k - 1.5) * 0.0016) - n * 0.0045
        el.append(('cap', tuple(b), tuple(t), 0.0031, 2.0))
    side = 1 if i == 0 else -1
    tb = palm - d * 0.004 + sd * (side * 0.0112)
    el.append(('cap', tuple(tb), tuple(tb + d * 0.008 + sd * side * 0.004 - n * 0.003), 0.0035, 2.0))  # the thumb
    return decimate(meta(el, res=0.0013), 270)


def doll_leg(i):
    h_, k_, a_, td = LEGS[i]
    start = h_.lerp(k_, 0.40)                     # out of the romper's leg band
    el = [('cap', tuple(start), tuple(k_), 0.0190, 2.0), ('cap', tuple(k_), tuple(a_), 0.0155, 2.0),
          ('ball', tuple(k_.lerp(a_, 0.38)), 0.0178, 1.4),                      # the calf
          ('ball', tuple(start.lerp(k_, 0.45)), 0.0215, 1.4),                   # the thigh, fat
          ('ball', tuple(k_), 0.0172, 1.5)]                                      # the knee
    d, n, sd = _frame(td, a_ - k_)
    heel = a_ + n * 0.005 - d * 0.004
    el.append(('ball', tuple(heel), 0.0112, 1.8))
    el.append(_ell_on(a_ + d * 0.017 + n * 0.004, (sd, d, n), (0.0125, 0.0215, 0.0092)))
    for k in range(5):                                                     # the toes
        off = (k - 2) * 0.0046 * (1 if i == 0 else -1)
        r = 0.0043 if k == 0 else 0.0034 - k * 0.0002
        el.append(('ball', tuple(a_ + d * (0.036 - abs(k - 0.5) * 0.0018) + sd * off + n * 0.003), r, 1.6))
    return decimate(meta(el, res=0.0014), 270)


def doll_arms():
    return flatten_below(doll_arm(0) + doll_arm(1), FLOOR_Y, 0.0)


def doll_legs():
    return flatten_below(doll_leg(0) + doll_leg(1), FLOOR_Y, 0.0)


def doll_romper_raw():
    """The romper on the cloth body: chest, belly, seat; puffed sleeves to
    mid upper arm; short legs gathered by elastic at mid thigh; the neck
    opening round the head's neck."""
    e = [
        ('ell', (0.030, 0.037, 0), (0.036, 0.033, 0.046)),          # chest
        ('ell', (-0.014, 0.039, 0), (0.042, 0.037, 0.045), 1.8),    # belly
        ('ell', (-0.050, 0.030, 0), (0.028, 0.027, 0.045), 1.8),    # seat
        ('ell', (0.052, 0.045, 0), (0.014, 0.027, 0.031), 1.6),     # round the neck
    ]
    for s_, e_, w_, pn in ARMS:
        d = (e_ - s_).normalized()
        c = s_.lerp(e_, 0.18)
        e.append(_ell_on(c, _frame(d, Vector((0, 1, 0))), (0.022, 0.019, 0.019), 1.8))
    for h_, k_, a_, td in LEGS:
        d = (k_ - h_).normalized()
        c = h_.lerp(k_, 0.24)
        e.append(_ell_on(c, _frame(d, Vector((0, 1, 0))), (0.028, 0.0235, 0.0245), 1.8))
    s = meta(e, res=0.0022)
    s = flatten_below(s, 0.010, 0.15)
    s = to_floor(s)

    # the cloth's folds: soft creases, stronger where it gathers
    def fold(p):
        q = Vector((p.x * 45, p.y * 25, p.z * 30))
        return p + Vector((0, 1, 0)) * 0.0020 * noise.noise(q) + Vector((0.0012 * noise.noise(q * 2.3), 0, 0.0012 * noise.noise(q * 2.1 + Vector((3, 1, 7)))))
    s.warp(fold)
    return flatten_below(s, FLOOR_Y, 0.0)


def doll_romper_mesh():
    if 'romper' not in _doll:
        r = decimate(doll_romper_raw(), 545)
        _doll['romper'] = r
        _doll['romper_bvh'] = bvh_of(r)
    return _doll['romper']


def doll_romper():
    return doll_romper_mesh().copy()


def doll_collar():
    """The Peter Pan collar's two rounded flaps on the chest, and two
    buttons down the front (white cotton, white plastic)."""
    doll_romper_mesh()
    bvh = _doll['romper_bvh']
    out = Shape()
    for sz in (-1, 1):
        c0 = Vector((0.050, 0.2, sz * 0.018))
        hit = bvh.ray_cast(c0, Vector((0, -1, 0)), 0.5)

        def flap(x, y, a, t):
            # a rounded flap: its straight edge along the neck (+x side)
            if math.cos(a) > 0.2:
                x = min(x, 0.004 * t)
            return x, y
        out = out + project_patch(bvh, hit[0], hit[1], (0, 0, sz), 0.032, 0.030, 0.0016, seg=16, rings=3, shape=flap)
    for x in (0.024, 0.000):
        hit = bvh.ray_cast(Vector((x, 0.2, 0)), Vector((0, -1, 0)), 0.5)
        out = out + project_patch(bvh, hit[0], hit[1], (1, 0, 0), 0.0080, 0.0080, 0.0018, seg=8, rings=1)
    return out


def head_to_body(p):
    """A point of the head's frame in the body's (as the game lays it)."""
    m = Matrix.Translation(DOLL_NECK) @ Matrix.Rotation(DOLL_TURN - math.pi / 2, 4, 'X') @ Matrix.Rotation(-math.pi / 2, 4, 'Z')
    return m @ Vector(p)


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, kind='box', smooth=35.0):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. `kind` is the frame the game derives its
    texture coordinates from ('box': each face from its own side, 'plane':
    straight along z)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth)


auto('Real__duckBody', duck_body, 560, smooth=70)
auto('Real__duckBill', duck_bill, 280, smooth=60)
auto('Real__duckEyeWhite', lambda: duck_eye_part('white'), 280)
auto('Real__duckEyePupil', lambda: duck_eye_part('pupil'), 280)
auto('Real__floorHoleMask', hole_mask, 100)
auto('Real__floorHoleCover', hole_cover, 400)
auto('Real__floorHoleLedge', hole_ledge, 400)
auto('Real__floorHoleFace', hole_face, 1100)
auto('Real__floorHoleDeck', hole_deck, 1100)
auto('Real__floorHoleWire', hole_wires, 1800)
auto('Real__floorHoleShaft', hole_shaft, 100)
auto('Real__floorHoleJoist', hole_joist, 1800)
auto('Real__floorHoleRubble', hole_rubble, 1800)
auto('Real__floorHoleDust', hole_dust, 400)
auto('Real__dollHead', doll_head, 560, smooth=70)
auto('Real__dollFace', doll_face, 560, kind='plane', smooth=70)
auto('Real__dollEyeR', lambda: doll_eye(0), 280, kind='sphere', smooth=80)
auto('Real__dollEyeL', lambda: doll_eye(1), 280, kind='sphere', smooth=80)
auto('Real__dollLids', doll_lids, 280, smooth=70)
auto('Real__dollLashes', doll_lashes, 100)
auto('Real__dollHair', doll_hair, 560, smooth=60)
auto('Real__dollRomper', doll_romper, 560, smooth=70)
auto('Real__dollCollar', doll_collar, 280, smooth=60)
auto('Real__dollArms', doll_arms, 560, smooth=70)
auto('Real__dollLegs', doll_legs, 560, smooth=70)
