"""t17: bits of a garden and a wood indoors, remodelled at their real
sizes (docs/design/PROPS.md P3): acorns of the pedunculate oak, with and without
their cups; horse chestnuts (conkers) and their split husks; an opened
Scots pine cone and the pine's shed needles in their pairs; a fallen frond
of common polypody; a small botanical plate framed under glass; a pair of
cotton gardening gloves dropped one over the other; a bow rake with an ash
handle. Sources and sizes: docs/research/props-remodelling/a_garden_and_a_wood_indoors.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
own frame: x across, y up from the floor it rests on (y = 0), z out of its
front; a thing that lies on the floor lies in its frame as it lies, its
lowest point at y = 0. A wall thing's wall is z = 0, its y from its
middle (the botanical's frame, as `t02`).

acornNut          a bare acorn (24 x 15.6 mm), lying along x, its axis at
                  y = NUT_R, its base (the cup's scar) at x = -NUT_L / 2
acornPairNut      an acorn still in its cup, lying tipped as it rests
acornPairCup      that cup (the game lays the nut's and the cup's maps on
                  by the axis, tipped by PAIR_TILT about the pair's pivot)
acornCup          an empty cup lying mouth up (18 mm across), its stalk
conker            a horse chestnut's seed (31 x 24 x 28 mm), lying on its
                  flatter side, its hilum up and to +z (a sphere frame:
                  the game paints it round its middle)
conkerSeated      the same seed sitting in an opened husk (the husk's frame)
conkerHuskOut     a husk split into its three valves and opened, lying on
                  its base: their leathery outsides and stout spines
conkerHuskIn      their pale insides and the split edges
pineCone          an opened Scots pine cone (56 mm, 38 mm across the open
                  scales), lying along x, its stalk at -x: the axis and the
                  woody scales
pineConeApo       the scales' exposed ends (apophyses), each with its umbo
pineNeedlesA..C   shed Scots pine needles: a pair in its sheath (A, B), a
                  single needle broken from its pair (C), lying flat
pineSheathA, B    the pairs' papery sheaths
fernFrond         a fallen frond of common polypody (Polypodium vulgare):
                  the blade over its outline's envelope (the game cuts the
                  outline out), lying, its segments' tips on the floor
fernStipe         its stalk and the rachis up the blade
botFrame          a 200 x 300 mm frame of a 16 mm flat moulding (the wall
                  z = 0, y from its middle)
botMat            the cream mat in it, its window cut with a bevel
botGlass          the glass
botPaper          the plate, behind the mat
botBack           the backing board
botWire           the screw eyes and the wire
gloveL*, gloveR*  a pair of cotton jersey gardening gloves with knitted
                  wrists: the left lying back up, the right dropped across
                  it palm up, draped over it (Body: the jersey; Cuff: the
                  knitted wrist)
rakeHandle        a bow rake's ash handle (1372 mm, 28.6 mm across), lying
                  as the rake lies tines down: along +x from the head
rakeSteel         the steel: the ferrule, the bow, the head and its 14
                  tines
"""
import math
import random
import zlib

import bmesh
from propkit import *
import propkit
from mathutils.geometry import delaunay_2d_cdt

FRAMED = {}       # name -> frame kind, where not 'box'


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    kind = FRAMED.get(name, 'box')
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, smooth=smooth, flat=flat, budget=budget)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def on_floor(s, centre=True):
    """Lowest point to y = 0, its middle (x, z) over the origin."""
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2 if centre else 0, -lo.y, -(lo.z + hi.z) / 2 if centre else 0)


def n1(x, y=0.0, z=0.0):
    return noise.noise(Vector((x, y, z)))


def sweep(pts, radii, seg=6, cap0=True, cap1=True, rough=None, flat=1.0, up=None, twist=0.0):
    """A tube along `pts` whose radius at each point is `radii[i]`
    (parallel-transport frame); `flat` squashes its section along the
    frame's normal (a needle's flat face), `up` sets that normal's start,
    `twist` turns the section along the length (radians, in all).
    `rough(i, j, p, d) -> dr` pushes a vertex out along d. cap0/cap1:
    'dome' rounds the end, True closes it flat, False leaves it open."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    tang = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tang.append(t.normalized() if t.length > 1e-12 else Vector((1, 0, 0)))
    if up is None:
        up = Vector((0, 1, 0)) if abs(tang[0].y) < 0.9 else Vector((1, 0, 0))
    nrm = (Vector(up) - tang[0] * Vector(up).dot(tang[0])).normalized()
    v, f = [], []
    for i in range(n):
        if i:
            q = tang[i - 1].rotation_difference(tang[i])
            nrm = (q @ nrm).normalized()
        b = tang[i].cross(nrm).normalized()
        tw = twist * i / max(1, n - 1)
        nn = nrm * math.cos(tw) + b * math.sin(tw)
        bb = tang[i].cross(nn).normalized()
        for j in range(seg):
            a = TAU * j / seg
            r = radii[i] + (rough(i, j, pts[i], nn * math.cos(a) + bb * math.sin(a)) if rough else 0.0)
            v.append(pts[i] + nn * (math.cos(a) * r * flat) + bb * (math.sin(a) * r))
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    s = Shape(v, f)
    for end, cap in ((0, cap0), (n - 1, cap1)):
        if not cap:
            continue
        ring = [end * seg + j for j in range(seg)]
        if cap == 'dome':
            out = -tang[0] if end == 0 else tang[-1]
            s.v.append(pts[end] + out * radii[end] * 0.8)
        else:
            s.v.append(pts[end].copy())
        c = len(s.v) - 1
        for a, b in zip(ring, ring[1:] + ring[:1]):
            s.f.append((b, a, c) if end == 0 else (a, b, c))
    return propkit._fix_winding(s)


def chaikin(pts, k=2):
    pts = [Vector(p) for p in pts]
    for _ in range(k):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    return pts


def hull(points):
    bm = bmesh.new()
    for p in points:
        bm.verts.new(p)
    bmesh.ops.convex_hull(bm, input=list(bm.verts))
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context='VERTS')
    s = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return propkit._fix_winding(s)


def grid_surface(fn, nu, nv, wrap_u=False):
    """A surface fn(u, v) -> Vector over [0,1]^2, (nu x nv) quads."""
    v, f = [], []
    cu = nu if wrap_u else nu + 1
    for j in range(nv + 1):
        for i in range(cu):
            v.append(fn(i / nu, j / nv))
    for j in range(nv):
        for i in range(nu):
            i2 = (i + 1) % cu if wrap_u else i + 1
            f.append((j * cu + i, j * cu + i2, (j + 1) * cu + i2, (j + 1) * cu + i))
    return Shape(v, f)


def closed_patch(fn_out, fn_in, nu, nv):
    """A solid between two surfaces over [0,1]^2 (outside and inside, same
    grid), its four edges closed: a shell's valve."""
    so = grid_surface(fn_out, nu, nv)
    si = grid_surface(fn_in, nu, nv)
    n = len(so.v)
    v = so.v + si.v
    f = list(so.f) + [tuple(reversed(tuple(i + n for i in q))) for q in si.f]
    cu = nu + 1
    rim = [i for i in range(cu)] + [j * cu + nu for j in range(1, nv + 1)] + \
          [nv * cu + i for i in range(nu - 1, -1, -1)] + [j * cu for j in range(nv - 1, 0, -1)]
    for a, b in zip(rim, rim[1:] + rim[:1]):
        f.append((b, a, a + n, b + n))
    return propkit._fix_winding(Shape(v, f))


# ================================================================ acorns ===
# Pedunculate oak (Quercus robur): the acorn oblong-ovoid, 19-35 mm long
# (2.7 cm on average), 12-20 mm across, a quarter to a third of it in the
# cup; the cup hemispherical, 15-20 mm across, grey-brown, of small
# appressed felted scales; the nut with a short point (the style) at its
# tip and a pale scar at its base where it sat in the cup. Most fall from
# their cups (the cups stay on the stalks); a few fall in them.
NUT_L, NUT_R = 0.025, 0.0076
# the nut's radius (a fraction of NUT_R) along its length from its base:
# blunt at its base, widest a quarter of the way, tapering to a rounded cone
NUT_PROF = [(0.000, 0.62), (0.012, 0.76), (0.035, 0.87), (0.08, 0.95), (0.15, 0.99), (0.25, 1.0), (0.35, 0.99),
            (0.45, 0.965), (0.55, 0.92), (0.65, 0.85), (0.73, 0.76), (0.80, 0.65), (0.86, 0.53), (0.91, 0.40),
            (0.95, 0.27), (0.98, 0.14), (1.0, 0.0)]
CUP_FLOOR = 0.0022     # the nut's base above the cup's foot
CUP_RIM = 0.0084       # the rim's height (the cup holds the nut's lowest quarter)
CUP_WALL = 0.0011


def nut_r(s):
    """The nut's radius s along its axis from its base."""
    t = s / NUT_L
    if t <= 0:
        return NUT_PROF[0][1] * NUT_R
    for (t0, r0), (t1, r1) in zip(NUT_PROF, NUT_PROF[1:]):
        if t <= t1:
            k = (t - t0) / (t1 - t0)
            return (r0 + (r1 - r0) * k) * NUT_R
    return 0.0


def nut_standing():
    """The nut along +y from its base (y = 0) to the tip of its style."""
    prof = [(0.0, 0.0)] + [(r * NUT_R, t * NUT_L) for t, r in NUT_PROF[:-1]] + [(0.0, NUT_L)]
    s = lathe(prof, seg=18)
    # not quite round: a little flatter one way, its sides a little uneven
    off = Vector((3.1, 0.7, 1.9))
    s = s.warp(lambda p: Vector((p.x * 1.0, p.y, p.z * 0.95)) + Vector((p.x, 0, p.z)) * 0.03 * n1(p.y * 140.0, 0.3) +
               Vector((p.x, 0, p.z)).normalized() * 0.00012 * noise.noise(p * 900.0 + off) if (p.x * p.x + p.z * p.z) > 1e-10 else p)
    # the style: a short point at its tip, a ring of fine hair round it
    style = cyl(0.0004, 0.0008, 0.0016, seg=8, bevel_r=0.0002, bevel_seg=1).move(0, NUT_L + 0.0006, 0)
    return s + style


def cup_profile_out(y):
    """The cup's outside radius at height y over its foot."""
    if y <= CUP_FLOOR:
        k = y / CUP_FLOOR
        return (0.0016 + (nut_r(0.0) + CUP_WALL + 0.0006 - 0.0016) * math.sin(k * math.pi / 2) ** 0.8)
    return nut_r(y - CUP_FLOOR) + CUP_WALL + 0.0006 * (1 - smoothstep(CUP_RIM - 0.002, CUP_RIM, y))


def cup_standing():
    """The cup mouth up, its foot at y = 0: felted scales in rows round it
    (imbricate, their tips toward the rim), the rim's lip, the inside
    shaped to the nut, the stalk's stub under it."""
    ys = [0.0, 0.0004, 0.001, 0.0016, CUP_FLOOR, 0.003, 0.0038, 0.0046, 0.0054, 0.0062, 0.007, 0.0077, CUP_RIM - 0.0003]
    outer = [(0.0, 0.0)] + [(cup_profile_out(y), y) for y in ys] + [(nut_r(CUP_RIM - CUP_FLOOR) + CUP_WALL * 0.45, CUP_RIM)]
    inner = [(nut_r(y - CUP_FLOOR) + 0.00008, y) for y in (CUP_RIM - 0.0002, 0.0070, 0.0058, 0.0046, 0.0034, CUP_FLOOR + 0.0004)]
    inner.append((nut_r(0.0) * 0.6, CUP_FLOOR + 0.00005))
    inner.append((0.0, CUP_FLOOR))
    s = lathe(outer + inner, seg=32, cap_bottom=False, cap_top=False)
    rows = 6

    def scales(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6:
            return p
        # only the outside (above its foot, below the rim's lip)
        inside = r < nut_r(max(0.0, p.y - CUP_FLOOR)) + 0.0003 and p.y > CUP_FLOOR
        if inside or p.y < 0.0006:
            return p
        a = math.atan2(p.z, p.x)
        h = p.y / CUP_RIM
        k = h * rows
        row = math.floor(k)
        fv = k - row
        fu = (a / TAU * 13 + 0.5 * (row % 2)) % 1.0
        # a shingle: thicker toward its tip (up), rounded across
        bump = 0.00050 * fv ** 1.4 * math.sin(math.pi * fu) ** 0.6 * (1 - smoothstep(0.9, 1.0, h))
        d = Vector((p.x / r, 0, p.z / r))
        return p + d * (bump + 0.00006 * n1(p.x * 900, p.y * 900, p.z * 900))
    s = s.warp(scales)
    # the stalk's stub (the peduncle broken off)
    stub = cyl(0.0011, 0.0013, 0.0016, seg=8, bevel_r=0.0003, bevel_seg=1).move(0, -0.0002, 0)
    return s + stub


def acorn_nut():
    """Bare, lying along +x (its base at -NUT_L/2), its axis at y = NUT_R."""
    s = nut_standing().rot(0, 0, -math.pi / 2)            # y -> +x
    return s.move(-NUT_L / 2, NUT_R, 0)


def _rest_angle(shape, com, lo_ang, hi_ang, steps=80):
    """The turn about z (radians) between lo and hi at which `shape`, laid
    down, holds its centre of mass `com` lowest over its lowest point."""
    best, ba = 1e9, lo_ang
    for i in range(steps + 1):
        a = lo_ang + (hi_ang - lo_ang) * i / steps
        ca, sa = math.cos(a), math.sin(a)
        ys = [p.x * sa + p.y * ca for p in shape.v]
        h = (com.x * sa + com.y * ca) - min(ys)
        if h < best:
            best, ba = h, a
    return ba


_PAIR = {}


def acorn_pair():
    """A nut in its cup, laid down as it rests: tipped so that the cup's rim
    and the nut's side both touch the floor. Returns (nut, cup) in one
    frame and records the tilt and pivot for the game (PAIR_*)."""
    if not _PAIR:
        nut = nut_standing().move(0, CUP_FLOOR, 0)
        cup = cup_standing()
        both = nut + cup
        com = Vector((0, CUP_FLOOR + NUT_L * 0.42, 0))
        # stood up along +y; lying means a turn of about -90 degrees about z
        a = _rest_angle(both, com, -math.pi / 2 - 0.4, -math.pi / 2 + 0.4)
        R = Matrix.Rotation(a, 4, 'Z')
        nut.apply(R)
        cup.apply(R)
        lo = min(p.y for p in (nut.v + cup.v))
        cx = (min(p.x for p in nut.v + cup.v) + max(p.x for p in nut.v + cup.v)) / 2
        nut.move(-cx, -lo, 0)
        cup.move(-cx, -lo, 0)
        _PAIR['nut'], _PAIR['cup'] = nut, cup
        _PAIR['tilt'] = a
        _PAIR['pivot'] = (-cx, -lo)
        print(f't17 acorn pair: tilt {a:.6f} rad, the cup foot at ({-cx:.6f}, {-lo:.6f})')
    return _PAIR


CUP_LOOSE_TILT = 0.22   # an empty cup lies a little on one side (about x)


def acorn_cup_loose():
    """An empty cup lying mouth up, a little on one side; its foot's middle
    over the origin (the game lays its map on by the tipped axis)."""
    return cup_standing().rot(CUP_LOOSE_TILT, 0, 0.0).move(0, 0.0, 0).move(0, -min(p.y for p in cup_standing().rot(CUP_LOOSE_TILT, 0, 0).v), 0)


auto('Real__acornNut', acorn_nut, 700, smooth=60.0)
auto('Real__acornPairNut', lambda: acorn_pair()['nut'], 700, smooth=60.0)
auto('Real__acornPairCup', lambda: acorn_pair()['cup'], 1500, smooth=50.0)
auto('Real__acornCup', acorn_cup_loose, 1500, smooth=50.0)
print(f't17 acorn loose cup: lifted {-min(p.y for p in cup_standing().rot(CUP_LOOSE_TILT, 0, 0).v):.6f}')


# ============================================================== conkers ===
# Horse chestnut (Aesculus hippocastanum): the fruit a leathery capsule
# about 6 cm across, set with stout prickles 4-8 mm long, splitting into
# three valves; one seed (rarely two or three), 2-4 cm, glossy chestnut
# brown, with a large pale scar (the hilum) where it was attached. Old
# conkers dry dull and dark and wrinkle; the husks go brown and hard.
CK = (0.0155, 0.0120, 0.0140)              # the seed's half sizes (x, y, z)
CK_HILUM = Vector((0.30, 0.50, 0.81)).normalized()     # the scar's middle, from the seed's middle


def conker():
    """The seed lying on its flatter side, its scar up and toward +z; its
    middle over the origin, its foot at y = 0."""
    s = ellipsoid(*CK, seg=30)
    off = Vector((1.7, 4.1, 0.3))

    def shape(p):
        d = Vector((p.x / CK[0], p.y / CK[1], p.z / CK[2]))
        # lumpy, as a seed pressed by the capsule's wall
        p = p + Vector((p.x, p.y, p.z)) * 0.07 * noise.noise(d * 1.6 + off)
        # its flatter side under it
        if p.y < -CK[1] * 0.45:
            p.y = -CK[1] * 0.45 + (p.y + CK[1] * 0.45) * 0.45
        # the scar a little flattened, its edge a low ridge
        c = d.normalized().dot(CK_HILUM)
        if c > 0.70:
            p = p - CK_HILUM * 0.0012 * smoothstep(0.70, 0.92, c)
        p = p + d.normalized() * 0.00025 * math.exp(-((c - 0.72) / 0.03) ** 2)
        # the small beak at the scar's edge
        beak = CK_HILUM.cross(Vector((0, 1, 0))).normalized()
        bd = (CK_HILUM * 0.62 + beak * 0.78).normalized()
        k = d.normalized().dot(bd)
        p = p + bd * 0.0016 * smoothstep(0.93, 1.0, k)
        return p
    s = s.warp(shape)
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2, -lo.y, -(lo.z + hi.z) / 2)


HU_R, HU_WALL = 0.0255, 0.0050              # the capsule's outer radius, its wall
# each valve opened back (radians): split as a husk dries on the ground,
# its valves fallen open further than they hold on the tree, unevenly,
# cupped round the seed; tipped a little onto one (opened a third less,
# they stood up round the seed like a flower's petals: M8; split along one
# seam and lain on its side, a valve stood on its edge like a card)
HU_OPEN = (0.92, 1.12, 0.80)
HU_TIP = 0.36                              # the husk tipped onto a valve as it lies
_HUSK = {}


def _valve_point(k, u, v, r):
    """Valve k (of three) at u across it (0..1) and v from the stalk's pole
    (0) to the apex (1), on a sphere of radius r about the capsule's
    middle (the stalk's pole at the origin), closed."""
    gap = 0.06
    phi = TAU * (k + gap + (1 - 2 * gap) * u) / 3
    a = math.pi * (0.04 + 0.94 * v)
    # the capsule a little pointed toward its apex, lumpy
    rr = r * (1 + 0.06 * v ** 3)
    p = Vector((math.sin(a) * math.cos(phi) * rr, HU_R - math.cos(a) * rr, math.sin(a) * math.sin(phi) * rr))
    return p


def _open(k, p):
    """A point of valve k, the valve turned back about the stalk's pole."""
    phim = TAU * (k + 0.5) / 3
    h = Vector((-math.sin(phim), 0, math.cos(phim)))
    return Matrix.Rotation(-HU_OPEN[k], 4, h) @ p


def husk():
    if _HUSK:
        return _HUSK
    rng = random.Random(77)
    outs, ins = Shape(), Shape()
    for k in range(3):
        wob = lambda p, s: p + p.normalized() * 0.0007 * n1(p.x * 180 + s, p.y * 180, p.z * 180)
        fo = lambda u, v, k=k: wob(_valve_point(k, u, v, HU_R), 1.3 * k)
        fi = lambda u, v, k=k: _valve_point(k, u, v, HU_R - HU_WALL)
        nu, nv = 7, 9
        solid = closed_patch(fo, fi, nu, nv)
        # the outside's faces come first; the inside's and the split edges
        # (the wall's thickness) after them, faced out of the solid (into
        # the capsule, and out of the edges)
        no = nu * nv
        used_o = sorted({i for q in solid.f[:no] for i in q})
        used_i = sorted({i for q in solid.f[no:] for i in q})
        mo = {o: k for k, o in enumerate(used_o)}
        mi = {o: k for k, o in enumerate(used_i)}
        sout = Shape([solid.v[i].copy() for i in used_o], [tuple(mo[i] for i in q) for q in solid.f[:no]])
        sin_ = Shape([solid.v[i].copy() for i in used_i], [tuple(mi[i] for i in q) for q in solid.f[no:]])
        # the prickles: stout, scattered, a little curved toward the apex
        spines = Shape()
        for _ in range(11):
            u, v = rng.uniform(0.15, 0.85), rng.uniform(0.18, 0.88)
            base = fo(u, v)
            nrm = (base - Vector((0, HU_R, 0))).normalized()
            up = (fo(u, min(1.0, v + 0.05)) - base).normalized()
            L = rng.uniform(0.004, 0.0075)
            pts = [base - nrm * 0.0008, base + nrm * L * 0.5 + up * L * 0.08, base + nrm * L + up * L * 0.22]
            spines = spines + sweep(pts, [0.0013, 0.0007, 0.00025], seg=6, cap0=True, cap1='dome')
        o = sout + spines
        i_ = sin_
        o.v = [_open(k, p) for p in o.v]
        i_.v = [_open(k, p) for p in i_.v]
        outs = outs + o
        ins = ins + i_
    # the stalk: a short woody stub under the valves' meeting
    stalk = sweep([Vector((0, 0.002, 0)), Vector((0.001, -0.004, 0.0005)), Vector((0.003, -0.009, 0.001))],
                  [0.0028, 0.0025, 0.0022], seg=7, cap0=True, cap1=True)
    outs = outs + stalk
    # it lies tipped onto valve 0
    R = Matrix.Rotation(HU_TIP, 4, Vector((-math.sin(TAU / 6), 0, math.cos(TAU / 6))))
    outs.apply(R)
    ins.apply(R)
    lo = min(p.y for p in outs.v + ins.v)
    xs = [p.x for p in outs.v]
    zs = [p.z for p in outs.v]
    c = Vector(((min(xs) + max(xs)) / 2, lo, (min(zs) + max(zs)) / 2))
    outs.move(-c.x, -c.y, -c.z)
    ins.move(-c.x, -c.y, -c.z)
    # the seed in it: under the cavity's middle, on the inside of the valve
    # it lies on (tipped over, the stalk's pole is no longer its floor)
    mid = (R @ Vector((0, HU_R, 0))) - c
    near = [p.y for p in ins.v if (p.x - mid.x) ** 2 + (p.z - mid.z) ** 2 < 0.010 ** 2 and p.y < mid.y]
    seat = Vector((mid.x, min(near) if near else mid.y - HU_R + HU_WALL, mid.z))
    _HUSK['out'], _HUSK['in'], _HUSK['seat'] = outs, ins, seat
    return _HUSK


def conker_seated():
    s = conker()
    h = husk()
    lo = min(p.y for p in s.v)
    return s.move(h['seat'].x, h['seat'].y - lo + 0.0006, h['seat'].z)


FRAMED['Real__conker'] = 'sphere'
FRAMED['Real__conkerSeated'] = 'sphere'
auto('Real__conker', conker, 1300, smooth=60.0)
auto('Real__conkerSeated', conker_seated, 1300, smooth=60.0)
auto('Real__conkerHuskOut', lambda: husk()['out'], 1500, smooth=55.0)
auto('Real__conkerHuskIn', lambda: husk()['in'], 900, smooth=55.0)


# ============================================================ pine cone ===
# Scots pine (Pinus sylvestris): the ripe cone 3-7.5 cm, ovoid-conic,
# short-stalked, opening when dry; its scales' exposed ends (apophyses)
# rhombic, flat to pyramidal (most at the base), with a small umbo, often a
# tiny prickle; dull grey-brown outside, the scales' hidden faces darker
# red-brown. The needles in pairs, 2.5-7 cm, twisted, in a 5-10 mm sheath.
PC_L = 0.050          # the axis carrying the scales (the cone 56 mm with its tip)
PC_N = 58
GOLDEN = math.radians(137.508)
_CONE = {}


def _pc_scale(k):
    t = k / (PC_N - 1)
    ax = PC_L * (0.04 + 0.92 * t ** 0.92)
    phi = k * GOLDEN
    L = 0.0045 + 0.0120 * math.sin(math.pi * min(1.0, 0.10 + 0.90 * t) ** 0.85) ** 1.2
    w = 0.0050 + 0.0042 * math.sin(math.pi * min(1.0, 0.08 + 0.92 * t) ** 0.8)
    beta = math.radians(68 - 46 * t ** 1.6)       # opened at its base, still closed toward its tip
    rc = 0.0042 * (1 - t) ** 0.6 + 0.0012
    return t, ax, phi, L, w, beta, rc


def pine_cone():
    if _CONE:
        return _CONE
    rng = random.Random(5)
    blades, apos = Shape(), Shape()
    A = Vector((1, 0, 0))
    for k in range(PC_N):
        t, ax, phi, L, w, beta, rc = _pc_scale(k)
        beta += rng.uniform(-0.08, 0.08)
        L *= rng.uniform(0.92, 1.06)
        r = Vector((0, math.cos(phi), math.sin(phi)))
        d = A * math.cos(beta) + r * math.sin(beta)
        nrm = -A * math.sin(beta) + r * math.cos(beta)
        tg = d.cross(nrm).normalized()
        p0 = A * ax + r * rc
        p1 = p0 + d * L
        th = 0.0011
        # the woody scale: a wedge from the axis, wider toward its end
        w0, w1 = w * 0.35, w * 0.85
        pts = []
        for (pp, ww) in ((p0, w0), (p1 - d * 0.0015, w1)):
            for sx in (-1, 1):
                for sy in (-1, 1):
                    pts.append(pp + tg * (sx * ww / 2) + nrm * (sy * th / 2))
        blades = blades + hull(pts)
        # its exposed end: a rhombic knob, keeled across, the umbo in its
        # middle (on the cone's outside face of the scale)
        h = w * 0.62
        c = p1 - d * h * 0.35 + nrm * (th / 2)
        T = 0.0019 + 0.0012 * (1 - t)            # pyramidal at the base
        q = []
        for (a, b) in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            q.append(c + tg * (a * w / 2) + d * (b * h / 2) - nrm * 0.0009)
        for (a, b) in ((0.85, 0.0), (-0.85, 0.0)):
            q.append(c + tg * (a * w / 2) + d * (b * h / 2) + nrm * (T * 0.35))
        for (a, b) in ((0.4, 0.45), (-0.4, 0.45), (0.4, -0.45), (-0.4, -0.45)):
            q.append(c + tg * (a * w / 2) + d * (b * h / 2) + nrm * (T * 0.45))
        q.append(c + nrm * T)
        q.append(c + nrm * (T + 0.0005) + d * 0.0002)       # the umbo's prickle
        apos = apos + hull(q)
    # the axis and the stalk
    core = sweep([Vector((-0.002, 0, 0)), Vector((PC_L * 0.5, 0, 0)), Vector((PC_L * 1.02, 0, 0))],
                 [0.0040, 0.0034, 0.0010], seg=8, cap0=True, cap1='dome')
    stalk = sweep([Vector((0.001, 0, 0)), Vector((-0.004, -0.0006, 0.0004)), Vector((-0.0085, -0.0018, 0.0012))],
                  [0.0019, 0.0017, 0.0015], seg=7, cap0=True, cap1=True)
    blades = blades + core + stalk
    # laid on its side (the axis along x), on the floor
    lo = min(p.y for p in blades.v + apos.v)
    xs = [p.x for p in blades.v + apos.v]
    zs = [p.z for p in blades.v + apos.v]
    c = Vector(((min(xs) + max(xs)) / 2, lo, (min(zs) + max(zs)) / 2))
    blades.move(-c.x, -c.y, -c.z)
    apos.move(-c.x, -c.y, -c.z)
    _CONE['scales'], _CONE['apo'] = blades, apos
    return _CONE


auto('Real__pineCone', lambda: pine_cone()['scales'], 1600, smooth=40.0)
auto('Real__pineConeApo', lambda: pine_cone()['apo'], 1900, smooth=35.0)


# ========================================================= pine needles ===
NEEDLES = {
    # name: (seed, needle lengths, spread (radians), sheath?)
    'A': (21, (0.052, 0.049), 0.30, True),
    'B': (22, (0.044, 0.046), 0.10, True),
    'C': (23, (0.047,), 0.0, False),
}
_NEEDLE = {}


def pine_needles(which):
    if which in _NEEDLE:
        return _NEEDLE[which]
    seed, lens, spread, sheath = NEEDLES[which]
    rng = random.Random(seed)
    out = Shape()
    for i, L in enumerate(lens):
        a = (spread / 2) * (1 if i == 0 else -1) if len(lens) > 1 else 0.0
        bend = rng.uniform(-0.25, 0.25)
        n = 11
        pts = []
        for j in range(n):
            t = j / (n - 1)
            ang = a + bend * t
            # lying: the base end on the sheath, the rest on the floor
            pts.append(Vector((math.cos(ang) * L * t - 0.004, 0.00045 + 0.0007 * max(0.0, 0.15 - t), math.sin(ang) * L * t * 1.0 + (0.0004 if i else -0.0004))))
        # smooth the polyline as a bent needle
        radii = [0.00078 * (1 - 0.15 * t) * (1 - smoothstep(0.82, 1.0, t) * 0.85) for t in (j / (n - 1) for j in range(n))]
        out = out + sweep(pts, radii, seg=5, flat=0.55, up=Vector((0, 1, 0)), twist=rng.uniform(0.4, 0.8) * (1 if i else -1), cap0=True, cap1=True)
    sh = Shape()
    if sheath:
        sp = [Vector((-0.0105, 0.00095, 0)), Vector((-0.0075, 0.00095, 0.0001)), Vector((-0.0035, 0.00092, 0.0))]
        sh = sweep(sp, [0.00085, 0.00095, 0.00098], seg=7, cap0=True, cap1=False,
                   rough=lambda i, j, p, d: 0.00008 * math.sin(j * 2.3 + i * 1.7))
    lo = min(p.y for p in out.v + sh.v)
    out.move(0, -lo, 0)
    if sh.v:
        sh.move(0, -lo, 0)
    _NEEDLE[which] = (out, sh)
    return _NEEDLE[which]


for _w in NEEDLES:
    auto(f'Real__pineNeedles{_w}', lambda _w=_w: pine_needles(_w)[0], 400, smooth=50.0)
for _w in ('A', 'B'):
    auto(f'Real__pineSheath{_w}', lambda _w=_w: pine_needles(_w)[1], 120, smooth=50.0)


# ================================================================= fern ===
# Common polypody (Polypodium vulgare), the fern of walls, rocks and old
# trees: fronds 10-40 cm, the blade oblong, deeply pinnatifid (cut nearly
# to the rachis) into 10-18 pairs of narrow oblong segments, blunt, their
# margins finely notched, shortening toward the tip; on their backs round
# naked sori, yellow to orange-brown, in a row each side of the midvein.
# A frond broken off lies on its back or its face, its segments' tips on
# the floor, the rachis a ridge. The blade lies in x-z with its rachis
# along +x from its base (x = 0); its shape raises it in y only, so its top
# view is the flat outline the game paints (FERN_* in `t17.rs`).
FERN_L = 0.150          # the blade
FERN_STIPE = 0.055
FERN_N = 13             # segment pairs
FERN_ANG = math.radians(64)    # a segment's angle forward from the rachis


def fern_seg(k, side):
    """Segment k (from the base) on side (+1 z, -1 -z): (x of its root on
    the rachis, its length, its half width)."""
    t = (k + (0.0 if side > 0 else 0.45)) / FERN_N
    x = FERN_L * (0.02 + 0.86 * t)
    # the segments much of a length up the lower half (the blade's sides
    # nearly parallel), the lowest pair a little shorter, then shortening
    # to the tip
    k = 0.90 + 0.10 * min(1.0, t * 4) if t < 0.5 else max(0.0, math.cos((t - 0.5) / 0.5 * math.pi / 2)) ** 0.8
    L = 0.032 * k + 0.004
    hw = 0.0036 + 0.0006 * (1 - t)
    return x, L, hw


def fern_env(x):
    """The blade's half width at x (its envelope, a little outside it)."""
    best = 0.0022
    for k in range(FERN_N):
        for side in (1, -1):
            x0, L, hw = fern_seg(k, side)
            # along the segment's axis from its root to its tip
            ca, sa = math.cos(FERN_ANG), math.sin(FERN_ANG)
            for j in range(9):
                s = L * j / 8
                cx, cz = x0 + s * ca, s * sa
                if abs(cx - x) < hw * 1.2:
                    best = max(best, cz + hw * 1.1)
    # the tip
    if x > FERN_L * 0.88:
        best = max(best, 0.006 * (1 - (x - FERN_L * 0.88) / (FERN_L * 0.12)))
    return best + 0.0012


def fern_frond():
    nu, nv = 44, 12
    xs0, xs1 = -0.002, FERN_L + 0.002
    envs = [fern_env(xs0 + (xs1 - xs0) * i / nu) for i in range(nu + 1)]
    # smoothed
    envs = [max(envs[i], (envs[max(0, i - 1)] + envs[min(nu, i + 1)]) / 2) for i in range(nu + 1)]
    off = Vector((0.3, 1.1, 2.7))

    def height(x, z, e):
        a = x / FERN_L
        b = abs(z) / max(e, 1e-6)
        # the rachis a ridge, the segments arching down to their tips
        y = 0.0026 * (1 - b) ** 1.6 * (1 - 0.55 * a)
        # the frond's length arched a little, its tip down
        y += 0.0018 * math.sin(math.pi * min(1.0, a * 1.1))
        # each segment's margins rolled down a little as it dried: waves
        # across the length at the segments' spacing
        y += 0.00045 * math.sin(a * FERN_N * TAU + (0.0 if z > 0 else 1.4)) * b
        # crumple
        y += 0.0005 * noise.noise(Vector((x * 90, z * 90, 0)) + off)
        return y

    v, f = [], []
    for i in range(nu + 1):
        x = xs0 + (xs1 - xs0) * i / nu
        e = envs[i]
        for j in range(nv + 1):
            b = -1 + 2 * j / nv
            z = b * e
            v.append(Vector((x, height(x, z, e), z)))
    for i in range(nu):
        for j in range(nv):
            a0 = i * (nv + 1) + j
            f.append((a0, a0 + 1, a0 + nv + 2, a0 + nv + 1))
    s = Shape(v, f)
    lo = min(p.y for p in s.v)
    return s.move(0, -lo, 0), height, -lo


def fern_blade():
    return fern_frond()[0]


def fern_stipe():
    s, height, lift = fern_frond()
    # the rachis along the blade's middle, on it
    pts = []
    for i in range(16):
        x = FERN_L * 0.97 * i / 15
        pts.append(Vector((x, height(x, 0.0, 0.01) + lift + 0.0004, 0.0)))
    radii = [0.00085 - 0.00055 * (i / 15) for i in range(16)]
    rach = sweep(pts, radii, seg=5, cap0=False, cap1=True)
    # the stipe: from the blade's base back to where it broke, down to the
    # floor, bent a little
    y0 = pts[0].y
    sp = [Vector((0.002, y0, 0.0)), Vector((-0.015, y0 * 0.8, 0.002)), Vector((-0.032, 0.0019, 0.004)), Vector((-FERN_STIPE, 0.0014, 0.003))]
    sp = chaikin(sp, 2)
    stipe = sweep(sp, [0.0010 + 0.0003 * i / (len(sp) - 1) for i in range(len(sp))], seg=6, cap0=False, cap1=True)
    return rach + stipe


auto('Real__fernFrond', fern_blade, 1300, smooth=60.0)
auto('Real__fernStipe', fern_stipe, 500, smooth=60.0)


# =========================================================== botanical ===
# A small hand-coloured botanical plate (one of the CC0 plates the game
# paints, `visuals::pictures`), in a cream mat under glass, in a 20 x 30 cm
# frame of a 16 mm flat moulding (a stained wood, a "20 x 30" frame as
# sold: 16-20 mm faces, 14-20 mm deep), on its wire. The frame of `t02`:
# x across its face, y up from its middle, z out of the wall (z = 0 the
# wall, where its back lies).
BT_W, BT_H = 0.200, 0.300
BT_M, BT_LIP, BT_ZL = 0.016, 0.004, 0.0085
BT_REB = (BT_W - 2 * (BT_M - BT_LIP), BT_H - 2 * (BT_M - BT_LIP))   # 0.176 x 0.276
BT_WIN = (0.118, 0.188, 0.004)     # the mat's window and how far it is raised
BT_PAPER = (0.132, 0.202)
# the moulding's face, outer edge to sight (d, z): a flat top, its outer
# edge rounded, a small bevel to the sight (BT_FACE in `t17.rs`)
BT_FACE = [(0.0, 0.0150), (0.0006, 0.0166), (0.0018, 0.0177), (0.0035, 0.0181), (0.0110, 0.0181), (0.0128, 0.0177),
           (0.0140, 0.0168), (0.0150, 0.0156), (BT_M, 0.0146)]


def _clean(profile):
    out = []
    for p in profile:
        if not out or abs(p[0] - out[-1][0]) + abs(p[1] - out[-1][1]) > 1e-7:
            out.append(p)
    if abs(out[0][0] - out[-1][0]) + abs(out[0][1] - out[-1][1]) < 1e-7:
        out.pop()
    return out


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle, mitred."""
    prof = _clean(profile)
    v, f = [], []
    n = len(prof)
    for (d, z) in prof:
        x, y = w / 2 - d, h / 2 - d
        v += [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    for i in range(n):
        j = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, j * 4 + k2, j * 4 + k))
    return propkit._fix_winding(Shape(v, f))


def loft2(lo, hi, z0, z1):
    n = len(lo)
    v = [(x, y, z0) for x, y in lo] + [(x, y, z1) for x, y in hi]
    f = [tuple(reversed(range(n))), tuple(range(n, 2 * n))]
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
    return propkit._fix_winding(Shape(v, f))


def board(w, h, z0, z1, r=0.0008):
    t = z1 - z0
    return rbox(w, h, t, r=min(r, t * 0.45), seg=1).move(0, 0, (z0 + z1) / 2)


def rect_outline(w, h, dy=0.0):
    return [(-w / 2, -h / 2 + dy), (w / 2, -h / 2 + dy), (w / 2, h / 2 + dy), (-w / 2, h / 2 + dy)]


def bot_frame():
    reb = BT_M - BT_LIP
    prof = [(0.0, 0.0)] + BT_FACE + [(BT_M, BT_ZL), (reb, BT_ZL), (reb, 0.0)]
    return rect_sweep(prof, BT_W, BT_H)


def bot_mat():
    w, h, dy = BT_WIN
    z0, z1 = BT_ZL - 0.0036, BT_ZL - 0.0021
    e = 0.0008
    s = 0.0021 / (z1 - z0)
    card = board(BT_REB[0] - 0.001, BT_REB[1] - 0.001, z0, z1, r=0.0003)
    lo = rect_outline(w - 2 * s * e, h - 2 * s * e, dy)
    hi = rect_outline(w + 2 * (0.0021 + s * e), h + 2 * (0.0021 + s * e), dy)
    return cut(card, loft2(lo, hi, z0 - e, z1 + e))


def bot_glass():
    return board(BT_REB[0] - 0.001, BT_REB[1] - 0.001, BT_ZL - 0.0020, BT_ZL - 0.0001, r=0.0004)


def bot_paper():
    z0 = BT_ZL - 0.0036 - 0.0006
    s = slab(BT_PAPER[0], BT_PAPER[1], 0.0004, nx=8, ny=10)
    for p in s.v:
        p.z += 0.00005 * math.sin(p.x * 120.0) * math.sin(p.y * 90.0)
    return s.move(0, BT_WIN[2], z0 + 0.0002)


def bot_back():
    return board(BT_REB[0] - 0.0012, BT_REB[1] - 0.0012, 0.0006, BT_ZL - 0.0048)


def screw_eye(at):
    ring = torus(0.0022, 0.0006, seg=10, tseg=5).rot(0, math.pi / 2, 0).move(0, 0, -0.0020)
    shank = cyl(0.0009, h=0.005, seg=5, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0015)
    return (ring + shank).move(*at)


BT_EYE_Y = BT_H / 2 - 0.085
BT_APEX = BT_H / 2 - 0.048        # the wire drawn up to the hook, above the middle


def bot_wire():
    xe = BT_W / 2 - BT_M / 2
    eyes = screw_eye((-xe, BT_EYE_Y, 0.0)) + screw_eye((xe, BT_EYE_Y, 0.0))
    z = -0.0020
    pts = [(-xe, BT_EYE_Y, z), (-xe / 2, (BT_EYE_Y + BT_APEX) / 2 - 0.0015, z - 0.0004), (0.0, BT_APEX, z - 0.0010),
           (xe / 2, (BT_EYE_Y + BT_APEX) / 2 - 0.0015, z - 0.0004), (xe, BT_EYE_Y, z)]
    wire = tube(pts, 0.0007, seg=4, caps=True, smooth=1)
    tails = Shape()
    for sx in (-1, 1):
        tails = tails + tube([(sx * xe, BT_EYE_Y, z), (sx * (xe - 0.011), BT_EYE_Y - 0.004, z - 0.0005),
                              (sx * (xe - 0.022), BT_EYE_Y - 0.002, z - 0.0005)], 0.0005, seg=4, caps=True, smooth=1)
    return eyes + wire + tails


auto('Real__botFrame', bot_frame, 300)
auto('Real__botMat', bot_mat, 200)
auto('Real__botGlass', bot_glass, 100)
auto('Real__botPaper', bot_paper, 500)
auto('Real__botBack', bot_back, 100)
auto('Real__botWire', bot_wire, 500)


# ================================================================= rake ===
# A bow rake (Ames/UnionTools 63107 and its like): a 54 in (1372 mm) white
# ash handle 1 1/8 in (28.6 mm) across, a steel ferrule, a tempered steel
# head 13 3/4 in (349 mm) wide on two bows, 14 tines about 2 3/4 in (70 mm)
# long, curved. The tool frame: the handle's axis along +x from the
# ferrule's mouth (x = 0), the head at -x, the tines down (-y). It lies as
# a dropped one does, tines down: on the tines' points and the handle's
# end, tipped down toward its end (RK_TIP); the game turns it about y only.
RK_HANDLE = 1.372
RK_D = 0.0286
RK_IN = 0.085            # the handle's end inside the ferrule
RK_FER = 0.115           # the ferrule's length
RK_HEAD_X = -0.205       # the head bar's middle
RK_W = 0.349
RK_TINES = 14
RK_TINE_L = 0.070
_RAKE = {}


def _rake_parts():
    if _RAKE:
        return _RAKE
    # ---- the handle (ash): from inside the ferrule to its rounded end
    x0, x1 = -RK_IN, RK_HANDLE - RK_IN
    prof = []
    r = RK_D / 2
    # a lathe along y, then laid along x
    ys = [0.0, 0.004, 0.010, 0.03]
    prof.append((0.0, x0))
    prof.append((r * 0.82, x0))
    prof.append((r * 0.9, x0 + 0.03))
    prof += [(r * (0.9 + 0.1 * smoothstep(0, 1, k / 3)), x0 + 0.03 + k * 0.03) for k in range(1, 4)]
    for k in range(1, 12):
        prof.append((r, x0 + 0.12 + (x1 - x0 - 0.16) * k / 11))
    prof += [(r * 0.97, x1 - 0.010), (r * 0.85, x1 - 0.004), (r * 0.55, x1 - 0.0008), (0.0, x1)]
    handle = lathe(prof, seg=14)
    handle = handle.warp(lambda p: Vector((p.y, -p.x, p.z)))     # its axis to +x... (y -> x)
    handle = handle.warp(lambda p: Vector((-p.x if False else p.x, p.y, p.z)))
    # ---- the ferrule: a steel socket, crimped round the handle, its end
    # drawn down onto the bow's stem, a rivet through it
    fer = lathe([(0.0, -RK_FER - 0.020), (0.0055, -RK_FER - 0.020), (0.0068, -RK_FER - 0.010), (0.0135, -RK_FER + 0.018),
                 (0.0158, -RK_FER + 0.040), (0.0160, -0.004), (0.0154, 0.0), (0.0146, 0.0)], seg=14, cap_bottom=True, cap_top=True)
    fer = fer.warp(lambda p: Vector((p.y, -p.x, p.z)))
    rivet = sphere(0.0028, seg=8, rings=4).scale(1, 0.45, 1).move(-0.060, 0.0158, 0.0)
    rivet2 = sphere(0.0028, seg=8, rings=4).scale(1, 0.45, 1).move(-0.060, -0.0158, 0.0)
    steel = fer + rivet + rivet2
    # ---- the bow: two flat straps from the stem out and up to the head's ends
    stem_x = -RK_FER - 0.018
    for sz in (-1, 1):
        pts = []
        for j in range(13):
            t = j / 12
            x = stem_x + (RK_HEAD_X - stem_x) * t
            z = sz * (0.004 + (RK_W / 2 - 0.012 - 0.004) * math.sin(t * math.pi / 2) ** 1.4)
            y = 0.050 * math.sin(math.pi * t) ** 0.8 - 0.004 * t
            pts.append(Vector((x, y, z)))
        steel = steel + sweep(pts, [0.0050] * len(pts), seg=4, flat=0.35, up=Vector((0, 1, 0)), cap0=True, cap1=True)
    # ---- the head bar and the tines
    bar = rbox(0.009, 0.007, RK_W, r=0.0015, seg=1).move(RK_HEAD_X, -0.004, 0)
    steel = steel + bar
    for i in range(RK_TINES):
        z = -RK_W / 2 + 0.0125 + (RK_W - 0.025) * i / (RK_TINES - 1)
        pts = []
        for j in range(6):
            t = j / 5
            # down from the bar, curving back toward the handle at the point
            pts.append(Vector((RK_HEAD_X + 0.010 * t ** 2.2, -0.006 - RK_TINE_L * t, z)))
        radii = [0.0030 * (1 - 0.55 * t) for t in (j / 5 for j in range(6))]
        steel = steel + sweep(pts, radii, seg=4, flat=0.75, up=Vector((1, 0, 0)), cap0=True, cap1=True)
    # ---- laid on the floor: on the tines' points and the handle's end
    tips = Vector((RK_HEAD_X + 0.010, -0.006 - RK_TINE_L - 0.0012))
    end = Vector((x1 - 0.004, -r))
    tip = math.atan2(end.y - tips.y, end.x - tips.x)
    R = Matrix.Rotation(-tip, 4, 'Z')
    handle.apply(R)
    steel.apply(R)
    lo = min(p.y for p in handle.v + steel.v)
    xs = [p.x for p in handle.v + steel.v]
    c = Vector(((min(xs) + max(xs)) / 2, lo, 0.0))
    handle.move(-c.x, -c.y, 0)
    steel.move(-c.x, -c.y, 0)
    _RAKE['handle'], _RAKE['steel'] = handle, steel
    print(f't17 rake: tipped {-tip:.6f} rad, moved by ({-c.x:.6f}, {-c.y:.6f})')
    return _RAKE


auto('Real__rakeHandle', lambda: _rake_parts()['handle'], 700, smooth=50.0)
auto('Real__rakeSteel', lambda: _rake_parts()['steel'], 1800, smooth=40.0)


# ================================================================ gloves ===
# A pair of cotton jersey gardening gloves with knitted wrists (as Town &
# Country's and the like: washable cotton, a knitted wrist; a medium 9 in,
# 230 mm, long): the right one dropped back up, the left one dropped across
# it palm up, its palm's PVC dots up. Each glove is a closed shell (its
# palm and back sewn at the rim), its rest shape as a hand would fill it;
# Blender's cloth solver (headless) drops it from a hand's height, fingers
# first, onto the floor (the left onto the floor and the right), and it
# settles limp, its slack in soft folds. The frame: the floor y = 0, the
# pair's middle over the origin.
GL_WRIST, GL_CUFF = 0.0, -0.062
GL_KN = 0.086
GL_FING = {   # base x, base z, length, flat width, angle from +z toward +x
    'index': (0.029, GL_KN + 0.002, 0.071, 0.0210, 0.08),
    'middle': (0.009, GL_KN + 0.006, 0.080, 0.0218, 0.01),
    'ring': (-0.011, GL_KN + 0.004, 0.074, 0.0208, -0.06),
    'little': (-0.030, GL_KN - 0.004, 0.058, 0.0192, -0.16),
}
GL_THUMB = ((0.039, 0.028), 0.062, 0.0240, 0.62)
GL_PALM = [Vector(p) for p in [(-0.036, GL_WRIST - 0.004), (0.034, GL_WRIST - 0.004), (0.042, 0.038), (0.038, GL_KN - 0.006),
                               (-0.036, GL_KN - 0.012), (-0.043, 0.036)]]
GL_CUFFP = [Vector(p) for p in [(-0.038, GL_CUFF), (0.037, GL_CUFF), (0.035, GL_WRIST + 0.006), (-0.036, GL_WRIST + 0.006)]]
GL_STEP = 0.0062


def _arclen(p):
    s = [0.0]
    for a, b in zip(p, p[1:]):
        s.append(s[-1] + (b - a).length)
    return s


def _sd_capsule_path(p, path, r0, r1):
    s = _arclen(path)
    L = s[-1]
    best, bt = 1e9, 0.0
    for i in range(len(path) - 1):
        a, b = path[i], path[i + 1]
        ab = b - a
        k = max(0.0, min(1.0, (p - a).dot(ab) / max(1e-12, ab.length_squared)))
        d = (p - (a + ab * k)).length
        t = (s[i] + k * ab.length) / L
        dd = d - (r0 + (r1 - r0) * t)
        if dd < best:
            best, bt = dd, t
    return best, bt


def _sd_round_poly(p, poly, r):
    inside = True
    dmin = 1e9
    for i in range(len(poly)):
        a, b = poly[i], poly[(i + 1) % len(poly)]
        ab = b - a
        k = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
        dmin = min(dmin, (p - (a + ab * k)).length)
        if ab.x * (p.y - a.y) - ab.y * (p.x - a.x) < 0:
            inside = False
    return (-dmin if inside else dmin) - r


def _smin(a, b, k):
    h = max(k - abs(a - b), 0.0) / k
    return min(a, b) - h * h * k * 0.25


def _gl_paths():
    out = {}
    for k, (x, z, L, w, ang) in GL_FING.items():
        pts = [Vector((x + math.sin(ang) * L * i / 6, z + math.cos(ang) * L * i / 6)) for i in range(7)]
        out[k] = [pts[0] - (pts[1] - pts[0]).normalized() * 0.012] + pts
    (bx, bz), L, w, ang = GL_THUMB
    pts = []
    for i in range(7):
        a = ang - 0.25 * i / 6
        pts.append(Vector((bx + math.sin(a) * L * i / 6, bz + math.cos(a) * L * i / 6)))
    out['thumb'] = [Vector((bx - 0.022, bz - 0.012))] + pts
    return out


GL_PATHS = _gl_paths()


def _gl_width(k, t):
    w = GL_THUMB[2] if k == 'thumb' else GL_FING[k][3]
    return w / 2 * (1.0 - 0.10 * t)


def _gl_field(x, z):
    p = Vector((x, z))
    d = _sd_round_poly(p, GL_PALM, 0.010)
    d = _smin(d, _sd_round_poly(p, GL_CUFFP, 0.004), 0.008)
    palm = d
    best = (1e9, None, 0.0)
    for k, path in GL_PATHS.items():
        L = _arclen(path)[-1]
        dd, t = _sd_capsule_path(p, path, _gl_width(k, 0.0), _gl_width(k, 1.0))
        t0 = (path[1] - path[0]).length / L
        tt = max(0.0, (t - t0) / (1 - t0))
        if dd < best[0]:
            best = (dd, k, tt)
        d = _smin(d, dd, 0.004 if k != 'thumb' else 0.012)
    return d, best[1], best[2], palm


def _marching(sdf, lo, hi, h):
    """The zero contour of a 2D field (its longest loop), by marching squares."""
    nx = int((hi[0] - lo[0]) / h) + 2
    nz = int((hi[1] - lo[1]) / h) + 2
    val = [[sdf(lo[0] + i * h, lo[1] + j * h) for i in range(nx)] for j in range(nz)]

    def ep(i0, j0, i1, j1):
        a, b = val[j0][i0], val[j1][i1]
        t = a / (a - b)
        return Vector((lo[0] + (i0 + (i1 - i0) * t) * h, lo[1] + (j0 + (j1 - j0) * t) * h))
    nb, pos = {}, {}
    for j in range(nz - 1):
        for i in range(nx - 1):
            cs = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)]
            ins = [val[q[1]][q[0]] < 0 for q in cs]
            cross = []
            for e in range(4):
                a, b = cs[e], cs[(e + 1) % 4]
                if ins[e] != ins[(e + 1) % 4]:
                    key = (min(a, b), max(a, b))
                    if key not in pos:
                        pos[key] = ep(key[0][0], key[0][1], key[1][0], key[1][1])
                    cross.append((e, key))
            if len(cross) == 2:
                pairs = [(cross[0][1], cross[1][1])]
            elif len(cross) == 4:
                pairs = [(cross[3][1], cross[0][1]), (cross[1][1], cross[2][1])] if ins[0] else \
                        [(cross[0][1], cross[1][1]), (cross[2][1], cross[3][1])]
            else:
                pairs = []
            for a, b in pairs:
                nb.setdefault(a, []).append(b)
                nb.setdefault(b, []).append(a)
    seen, loops = set(), []
    for k0 in sorted(nb):
        if k0 in seen:
            continue
        loop = [k0]
        seen.add(k0)
        prev, cur = None, k0
        while True:
            nxt = [k for k in nb[cur] if k != prev and k not in seen]
            if not nxt:
                break
            prev, cur = cur, nxt[0]
            seen.add(cur)
            loop.append(cur)
        loops.append([pos[k] for k in loop])
    return max(loops, key=len)


def _resample_loop(pts, step):
    p = pts + [pts[0]]
    s = _arclen(p)
    L = s[-1]
    n = max(8, int(L / step))
    out, j = [], 0
    for i in range(n):
        t = L * i / n
        while j < len(s) - 2 and s[j + 1] < t:
            j += 1
        k = (t - s[j]) / max(1e-12, s[j + 1] - s[j])
        out.append(p[j].lerp(p[j + 1], min(1.0, max(0.0, k))))
    return out


def glove_shell():
    """The glove's rest shape: a closed shell over its outline (x across, z
    from the wrist toward the fingers, y up), as a hand would fill it, the
    rim (its seam) at y = 0. Returns (verts, faces, the faces' regions:
    'cuff' or 'body')."""
    sdf = lambda x, z: _gl_field(x, z)[0]
    lo, hi = (-0.07, GL_CUFF - 0.01), (0.12, 0.215)
    loop = _resample_loop(_marching(sdf, lo, hi, 0.001), GL_STEP * 0.8)
    rng = random.Random(3)
    pts = []
    nx = int((hi[0] - lo[0]) / GL_STEP) + 1
    nz = int((hi[1] - lo[1]) / GL_STEP) + 1
    for j in range(nz):
        for i in range(nx):
            x = lo[0] + (i + 0.5 * (j % 2)) * GL_STEP + rng.uniform(-0.15, 0.15) * GL_STEP
            z = lo[1] + j * GL_STEP * 0.87 + rng.uniform(-0.15, 0.15) * GL_STEP
            if sdf(x, z) < -GL_STEP * 0.55:
                pts.append(Vector((x, z)))
    verts = [Vector(p) for p in loop] + pts
    face = list(range(len(loop)))
    if sum(a.x * b.y - b.x * a.y for a, b in zip(loop, loop[1:] + loop[:1])) < 0:
        face.reverse()
    r = delaunay_2d_cdt(verts, [], [face], 1, 1e-7, True)
    v2 = [Vector(v) for v in r[0]]
    tris = [tuple(f) for f in r[2]]
    nrim = len(loop)

    def thick(x, z):
        d, k, t, palm = _gl_field(x, z)
        dd = max(0.0, -d)
        if k is not None and palm > 0:
            hw = _gl_width(k, t)
            return 0.85 * hw * math.sqrt(max(0.0, 1 - (1 - min(1.0, dd / hw)) ** 2))
        return min(0.011, 0.9 * dd ** 0.75 * 0.05 ** 0.25)
    V = [Vector((p.x, thick(p.x, p.y), p.y)) for p in v2]
    ib = {}
    for i, p in enumerate(v2):
        if i < nrim:
            ib[i] = i
        else:
            ib[i] = len(V)
            V.append(Vector((p.x, -thick(p.x, p.y), p.y)))
    F, R = [], []
    for a, b, c in tris:
        zc = (v2[a].y + v2[b].y + v2[c].y) / 3
        reg = 'cuff' if zc < GL_WRIST - 0.002 else 'body'
        # the top's faces turned up (out of the shell), the bottom's down
        if (V[b] - V[a]).cross(V[c] - V[a]).y < 0:
            b, c = c, b
        F.append((a, b, c))
        F.append((ib[a], ib[c], ib[b]))
        R += [reg, reg]
    return V, F, R


def _cloth_drop(V, F, place, colliders, frames=150, flat=0.25, seed=0.3):
    """Drop a shell (rest shape V) by the cloth solver: it starts flattened
    to `flat` of its rest thickness, a little lumpy, turned and moved by
    `place` (a function of a point), and falls onto the floor and
    `colliders` (meshes, as (verts, faces))."""
    import bpy
    sc = bpy.context.scene
    me = bpy.data.meshes.new('glove')
    me.from_pydata([tuple(v) for v in V], [], F)
    me.update()
    ob = bpy.data.objects.new('glove', me)
    sc.collection.objects.link(ob)
    basis = ob.shape_key_add(name='Basis')
    start = ob.shape_key_add(name='start')
    for i, v in enumerate(V):
        q = Vector((v.x, v.y * flat, v.z))
        if v.y > 0:
            q.y += 0.0025 * noise.noise(Vector((v.x * 35, v.z * 35, seed))) + 0.001 * noise.noise(Vector((v.x * 110, v.z * 110, seed + 1.0)))
        start.data[i].co = place(q)
    start.value = 1.0
    made = []
    fl = bpy.data.meshes.new('floor')
    fl.from_pydata([(-2, 0, -2), (2, 0, -2), (2, 0, 2), (-2, 0, 2)], [], [(3, 2, 1, 0)])
    fl.update()
    floor = bpy.data.objects.new('floor', fl)
    sc.collection.objects.link(floor)
    floor.modifiers.new('col', 'COLLISION')
    floor.collision.thickness_outer = 0.001
    floor.collision.cloth_friction = 15.0
    made.append(floor)
    for k, (cv, cf) in enumerate(colliders):
        cm = bpy.data.meshes.new(f'col{k}')
        cm.from_pydata([tuple(v) for v in cv], [], cf)
        cm.update()
        co = bpy.data.objects.new(f'col{k}', cm)
        sc.collection.objects.link(co)
        co.modifiers.new('col', 'COLLISION')
        co.collision.thickness_outer = 0.0012
        co.collision.cloth_friction = 12.0
        made.append(co)
    cl = ob.modifiers.new('cloth', 'CLOTH')
    s = cl.settings
    s.quality = 10
    s.mass = 0.05
    s.tension_stiffness = 25.0
    s.compression_stiffness = 25.0
    s.shear_stiffness = 5.0
    s.bending_stiffness = 0.02
    s.air_damping = 3.0
    s.rest_shape_key = basis
    c = cl.collision_settings
    c.use_collision = True
    c.distance_min = 0.0015
    c.use_self_collision = True
    c.self_distance_min = 0.0012
    c.self_friction = 10.0
    c.collision_quality = 4
    grav = tuple(sc.gravity)
    sc.gravity = (0.0, -9.81, 0.0)
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
    for o in made:
        m = o.data
        bpy.data.objects.remove(o)
        bpy.data.meshes.remove(m)
    return out


_GLOVES = {}


def gloves():
    if _GLOVES:
        return _GLOVES
    V, F, R = glove_shell()
    # the right glove (its thumb toward +x, its back up), dropped fingers
    # first from a little over the floor, rolled a little
    def place_r(q):
        p = Matrix.Rotation(-0.9, 4, 'X') @ Matrix.Rotation(0.3, 4, 'Z') @ q
        return p + Vector((0, 0.075, 0))
    PR = _cloth_drop(V, F, place_r, [], seed=0.3)
    # the left glove: the right one mirrored, turned over (palm up) and
    # dropped across the right one's fingers, at an angle to it
    VL = [Vector((-v.x, v.y, v.z)) for v in V]
    FL = [(a, c, b) for (a, b, c) in F]
    lo_r = min(p.y for p in PR)
    # where the right glove's knuckles and wrist went (its rest points nearest)
    def near(x, z):
        return min(range(len(V)), key=lambda i: (V[i].x - x) ** 2 + (V[i].z - z) ** 2 + (0.0 if V[i].y >= 0 else 1.0))
    kn, wr = PR[near(0.0, GL_KN)], PR[near(0.0, GL_WRIST)]
    along = Vector((kn.x - wr.x, 0, kn.z - wr.z)).normalized()
    yaw_r = math.atan2(along.x, along.z)
    top = max(p.y for p in PR)

    def place_l(q):
        p = Matrix.Rotation(math.pi, 4, 'Z') @ q            # palm up
        p = Matrix.Rotation(-0.30, 4, 'X') @ p               # fingers first, a little
        p = Matrix.Rotation(yaw_r + 2.2, 4, 'Y') @ p         # across the right one
        # its palm's middle over the right one's knuckles
        return p + Vector((kn.x, top + 0.030, kn.z)) - Matrix.Rotation(yaw_r + 2.2, 4, 'Y') @ Vector((0, 0, 0.045))
    # the right one is met by its top (its back): its faces made first of each pair
    PL = _cloth_drop(VL, FL, place_l, [([p + Vector((0, 0.0008, 0)) for p in PR], F[0::2])], frames=200, seed=2.1)
    allp = PR + PL
    lo = min(p.y for p in allp)
    xs = [p.x for p in allp]
    zs = [p.z for p in allp]
    c = Vector(((min(xs) + max(xs)) / 2, lo, (min(zs) + max(zs)) / 2))

    def sub(P, faces, reg, which):
        used = sorted({i for f, r in zip(faces, reg) if r == which for i in f})
        m = {o: k for k, o in enumerate(used)}
        return Shape([P[i] - c for i in used], [tuple(m[i] for i in f) for f, r in zip(faces, reg) if r == which])
    _GLOVES['RBody'] = sub(PR, F, R, 'body')
    _GLOVES['RCuff'] = sub(PR, F, R, 'cuff')
    _GLOVES['LBody'] = sub(PL, FL, R, 'body')
    _GLOVES['LCuff'] = sub(PL, FL, R, 'cuff')
    print(f't17 gloves: right {lo_r:.4f}..{max(p.y for p in PR):.4f}, left ..{max(p.y for p in PL):.4f}; moved by {-c.x:.4f}, {-c.z:.4f}')
    return _GLOVES


# each budget a literal of its own: the asset lint reads them from the calls
for _g in ('RBody', 'LBody'):
    auto(f'Real__glove{_g}', lambda _g=_g: gloves()[_g], 2600, smooth=70.0)
for _g in ('RCuff', 'LCuff'):
    auto(f'Real__glove{_g}', lambda _g=_g: gloves()[_g], 700, smooth=70.0)
