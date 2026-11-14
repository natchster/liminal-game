"""everyday_litter (was r08): litter of the everyday remodelled at its real sizes (docs/design/PROPS.md):
office paper balled up and thrown away, a round steel wastebasket knocked
over with its liner and a tied bag beside it, a lost sock, crown caps and a
soda can crushed underfoot. (The chalk marks of the same batch are painted,
not modelled.) Sources and sizes: docs/research/props-remodelling/everyday_litter.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). A part that needs more
triangles than its size gives it (`budget_for`) is registered on a
`part('…', budget=N)` line of its own, which the asset lint reads. Every part is in its
own frame: x across its front, y up from the floor (y = 0 the floor it
rests on), z out of its front; a can or a cap is built standing, its axis
+y, and the game lays it down where it lies.

paperBall       a sheet of A4 balled tight in a fist (65 mm): creases in
                straight runs across it, two corners of the sheet lifting
paperWad        a sheet crumpled loosely (80 mm), more lumpy, three flaps
paperSheet      a sheet crumpled and then smoothed out again, left on the
                floor: its creases, a corner curled up
wastebasket     a round steel wastebasket (26 qt: 343 mm across, 368 mm
                tall), standing: the raised bottom in its chime, two pressed
                beads, the locked side seam, the rolled wire rim, a dent
wastebasketLiner  its liner: lining it, turned over the rim and hanging
                down outside, gathered
wastebasketTrash  what is in it, standing: crumpled paper pressed down to a
                little over half its height, balled sheets on top
trashBag        a small bin bag full and tied: slumped on its bottom, the
                film drawn into pleats at the neck, the knot and its ears
sock            a crew sock lying limp: a flattened tube from the open
                cuff down the leg, the heel turned, the foot to the toe
sockHeelToe     its reinforced heel and toe, a skin over the knit
sockBands       the two bands knitted round its cuff (a sports sock's)
capCrown        a crown cap prised off a bottle: the 21 flutes crimped in,
                the skirt, the knuckle (its painted top is capTop)
capTop          the cap's printed top panel
capLiner        the plastic liner inside it
capCrownBent, capTopBent, capLinerBent   the same, bent by the opener
canCrushedBody  a 355 ml can stamped flat on its end: its printed wall
                folded into rings that bulge out, the top slid off centre
canCrushedEnds  its bottom dome and its lid: the double seam, the
                countersink, the drinking hole pushed in, the tab
"""
from propkit import *
import propkit

# ------------------------------------------------------------------ util --
def frac(x):
    return x - math.floor(x)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def grid_surface(fn, nu, nv, wrap_u=False):
    """A surface from fn(u, v) -> Vector over u in [0, 1] (nu steps, wrapped
    if wrap_u) and v in [0, 1] (nv steps): an open grid of quads."""
    cols = nu if wrap_u else nu + 1
    v, f = [], []
    for j in range(nv + 1):
        for i in range(cols):
            v.append(fn(i / nu, j / nv))
    for j in range(nv):
        for i in range(nu):
            a = j * cols + i
            b = j * cols + (i + 1) % cols if wrap_u else a + 1
            f.append((a, b, b + cols, a + cols))
    return Shape(v, f)


def fan_close(s, ring, centre):
    """Close a ring of vertex ids with a fan to a new centre vertex."""
    s.v.append(Vector(centre))
    c = len(s.v) - 1
    for a, b in zip(ring, ring[1:] + ring[:1]):
        s.f.append((a, b, c))
    return s


def on_floor(s, y0=0.0):
    """Move so the lowest point is at y0."""
    lo, hi = s.bounds()
    return s.move(0, y0 - lo.y, 0)


# ---------------------------------------------------------------- paper --
# A4 is 210 x 297 mm (ISO 216), office paper 80 g/m2, 0.1 mm thick.
A4_W, A4_L = 0.210, 0.297


def _creases(rng, n):
    """n crease planes through the ball: (normal, offset, depth, sign)"""
    out = []
    for _ in range(n):
        nn = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
        if nn.length < 0.1:
            nn = Vector((0, 1, 0))
        out.append((nn.normalized(), rng.uniform(-0.6, 0.6), rng.uniform(0.05, 0.12), rng.choice((1, 1, -1))))
    return out


def _flap(rng, ball, d, size):
    """A corner of the sheet coming away from the ball at direction d: a thin
    triangle lying along the surface, lifting toward its tip, its root under
    the surface."""
    up = Vector((0.3, 1, 0.2)) if abs(d.y) < 0.9 else Vector((1, 0, 0))
    t = d.cross(up).normalized()
    b = d.cross(t).normalized()
    ang = rng.uniform(0, TAU)
    t, b = t * math.cos(ang) + b * math.sin(ang), b * math.cos(ang) - t * math.sin(ang)
    L, W = size, size * rng.uniform(0.8, 1.2)
    tip = rng.uniform(-0.3, 0.3) * W
    pts = [(0.0, -W / 2), (L * 0.5, (-W / 2 + tip) / 2), (L, tip), (L * 0.5, (W / 2 + tip) / 2), (0.0, W / 2)]
    lift = rng.uniform(0.15, 0.45)
    v = []
    for a, c in pts:
        q = (d * 0.6 + t * (a - L * 0.35) / ball + b * c / ball).normalized()
        k = 0.93 + lift * (a / L) ** 1.6 * min(0.4, size / ball)
        v.append(q * ball * k)
    th = 0.00035
    vv = [p * (1 + th / ball) for p in v] + [p * (1 - th / ball) for p in v]
    f = [(0, 1, 3, 4), (1, 2, 3), (9, 8, 6, 5), (8, 7, 6)]
    for a in range(5):
        c = (a + 1) % 5
        f.append((a, c, c + 5, a + 5))
    return _fix(Shape(vv, f))


def _fix(s):
    return propkit._fix_winding(s)


def _hull_sphere(n, rng, jitter=0.45):
    """A unit sphere of irregular triangles: n points spread evenly
    (a Fibonacci spiral), each pushed aside at random, and their hull."""
    pts = []
    ga = math.pi * (3 - math.sqrt(5))
    step = math.sqrt(4 * math.pi / n)
    for i in range(n):
        y = 1 - 2 * (i + 0.5) / n
        r = math.sqrt(max(0.0, 1 - y * y))
        p = Vector((math.cos(ga * i) * r, y, math.sin(ga * i) * r))
        p += Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * step * jitter
        pts.append(p.normalized())
    bm = bmesh.new()
    for p in pts:
        bm.verts.new(p)
    bmesh.ops.convex_hull(bm, input=bm.verts)
    bm.verts.index_update()
    s = Shape([v.co.copy() for v in bm.verts], [tuple(v.index for v in f.verts) for f in bm.faces])
    bm.free()
    return _fix(s)


def paper_ball(r, seed, n=110, folds=9, lump=0.12, jag=0.15, pockets=7, squash=0.86, flaps=3, flap_size=0.02):
    """A sheet balled in a fist: a few dozen flat facets meeting at sharp
    creases (every corner of them a little in or out of the next), pockets
    where the sheet was folded in on itself, crease lines where it was
    folded over, lumpy, settled on its flattest side, corners of the sheet
    coming away from it. Shaded flat: paper has no round in it."""
    rng = random.Random(seed)
    cr = _creases(rng, folds)
    o1 = Vector((rng.uniform(-30, 30), rng.uniform(-30, 30), rng.uniform(-30, 30)))
    s = _hull_sphere(n, rng)
    jit = [rng.uniform(-1, 1) for _ in s.v]
    # pockets: where the sheet was pushed in, a few corners sink deep
    pk = [Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))).normalized() for _ in range(pockets)]

    def crumple(i, p):
        d = p.normalized()
        k = 1.0 + lump * noise.noise(d * 1.2 + o1) + jag * jit[i]
        for nn, off, depth, sg in cr:
            x = d.dot(nn) - off
            k -= sg * depth * max(0.0, 0.13 - abs(x)) / 0.13
        for q in pk:
            k -= 0.30 * max(0.0, d.dot(q) - 0.93) / 0.07
        q = d * r * k
        q.y *= squash
        return q
    s.v = [crumple(i, p) for i, p in enumerate(s.v)]
    lo, hi = s.bounds()
    flat = lo.y + (hi.y - lo.y) * 0.07
    s.warp(lambda p: Vector((p.x, max(p.y, flat), p.z)))
    for _ in range(flaps):
        d = Vector((rng.uniform(-1, 1), rng.uniform(-0.1, 1), rng.uniform(-1, 1))).normalized()
        s = s + _flap(rng, r, d, flap_size * rng.uniform(0.8, 1.2))
    return on_floor(s)


def paper_sheet(seed=5):
    """A sheet crumpled and smoothed out again: a field of straight creases
    (ridges and valleys a few millimetres high) where the ball's folds were,
    lying on the floor where they let it, one corner curled up."""
    rng = random.Random(seed)
    nx, nz = 14, 20
    s = grid_plane(A4_W, A4_L, nx, nz).rot(x=-math.pi / 2)      # in XZ, facing +y
    lines = []
    for _ in range(26):
        p = Vector((rng.uniform(-A4_W / 2, A4_W / 2), 0, rng.uniform(-A4_L / 2, A4_L / 2)))
        a = rng.uniform(0, math.pi)
        lines.append((p, Vector((math.cos(a), 0, math.sin(a))), rng.uniform(0.002, 0.0055), rng.uniform(0.008, 0.02), rng.choice((1, -1)),
                      rng.uniform(0.04, 0.16)))
    curl = [rng.uniform(0, 1) ** 2 for _ in range(4)]
    curl[rng.randrange(4)] = 1.0
    corners = [(-A4_W / 2, -A4_L / 2), (A4_W / 2, -A4_L / 2), (A4_W / 2, A4_L / 2), (-A4_W / 2, A4_L / 2)]

    def lay(q):
        h = 0.0
        for p, dirn, amp, hw, sg, half in lines:
            rel = q - p
            along = rel.dot(dirn)
            if abs(along) > half:
                continue                     # a crease runs only so far
            dist = abs(rel.x * dirn.z - rel.z * dirn.x)
            fade = 1.0 - (abs(along) / half) ** 2
            h += sg * amp * max(0.0, 1.0 - dist / hw) * fade
        # the old middle fold, not quite flattened
        h += 0.004 * max(0.0, 1.0 - abs(q.x) / 0.03)
        for (cx, cz), k in zip(corners, curl):
            dd = math.hypot(q.x - cx, q.z - cz)
            if dd < 0.075:
                h += k * 0.028 * (1 - dd / 0.075) ** 2.2
        h += 0.0015 * noise.noise(Vector((q.x * 22, 3.1, q.z * 22)))
        return Vector((q.x, h, q.z))
    s.warp(lay)
    # every crossing of the old creases a little up or down: the facets
    for p in s.v:
        p.y += rng.uniform(-0.0011, 0.0011)
        p.x += rng.uniform(-0.0035, 0.0035) * (abs(p.x) < A4_W / 2 - 1e-4)
        p.z += rng.uniform(-0.0035, 0.0035) * (abs(p.z) < A4_L / 2 - 1e-4)
    # where it is held down it lies on the floor
    hs = sorted(p.y for p in s.v)
    base = hs[len(hs) // 3]
    s.warp(lambda p: Vector((p.x, max(0.0, p.y - base) + 0.0003, p.z)))
    return s


# ----------------------------------------------------------- wastebasket --
# Rubbermaid Commercial WB26 class: a round steel wastebasket, 26 qt, 13.5 in
# (343 mm) across, 14.5 in (368 mm) tall, a locked side seam, a rolled wire
# rim, the bottom raised in its chime (docs/research/props-remodelling/everyday_litter.md).
WB_R, WB_H = 0.1715, 0.368
WB_SEAM = -2.4                     # the seam's angle round the can (radians from +z toward +x)
WB_DENT = (-1.0, 0.150, 0.020)     # a dent: angle, height, depth
WB_FLOOR_X = 0.1765                # the game lays it on its +x side: nothing may reach past its rim there
WB_FILL = 0.262                    # how full of paper it stands


def _dent(p, depth=None):
    """The dent a kick gave it: a push in, mid-wall, a crease across it,
    gone 45 mm above and below (the wall's rows on either side stay true,
    so nothing between them bends out past the liner)."""
    a0, y0, d0 = WB_DENT
    depth = d0 if depth is None else depth
    a = math.atan2(p.x, p.z)
    da = (a - a0 + math.pi) % TAU - math.pi
    w = math.exp(-(da / 0.34) ** 2) * max(0.0, 1.0 - ((p.y - y0) / 0.045) ** 2) ** 2
    rr = math.hypot(p.x, p.z)
    if rr < 1e-6 or w < 1e-4:
        return p
    k = (rr - depth * w) / rr
    return Vector((p.x * k, p.y, p.z * k))


def wastebasket():
    """Pressed from one sheet of steel: the bottom raised in its chime, two
    beads pressed round it, the wall rolled over a wire at the rim, the
    side seam locked and standing proud; dented once, mid-wall (the wall
    has rows there, inside and out, so the dent bends it and nothing
    pokes through)."""
    R, H = WB_R, WB_H
    outer = [
        (R - 0.014, 0.010), (R - 0.0065, 0.0015), (R - 0.002, 0.0), (R + 0.0022, 0.004), (R + 0.002, 0.013), (R, 0.020),
        (R, 0.044), (R + 0.005, 0.054), (R, 0.064),
        (R, 0.110), (R, 0.150), (R, 0.190),
        (R, 0.254), (R + 0.005, 0.264), (R, 0.274),
        (R, H - 0.011), (R + 0.0035, H - 0.007), (R + 0.005, H - 0.002), (R + 0.003, H + 0.0018),
        (R - 0.001, H + 0.002), (R - 0.004, H - 0.003), (R - 0.0025, H - 0.009),
    ]
    inner = [(R - 0.0012, H - 0.014), (R - 0.0012, 0.190), (R - 0.0012, 0.150), (R - 0.0012, 0.110),
             (R - 0.0012, 0.030), (R - 0.010, 0.016)]
    s = lathe([(0.0, 0.010)] + outer + inner + [(0.0, 0.016)], seg=30, cap_bottom=False, cap_top=False)
    # the locked seam: a strip folded over itself, standing proud of the wall
    seam = rbox(0.011, H - 0.050, 0.0030, r=0.001, seg=1).move(0, H / 2 + 0.006, R + 0.0006)
    seam.rot(y=WB_SEAM)
    s = s + seam
    return s.warp(_dent)


def wastebasket_trash(seed=12):
    """What is in it, standing: crumpled office paper to a little over half
    its height, a heap of it pressed down, the tops of balled sheets
    standing out of it."""
    rng = random.Random(seed)
    R = WB_R - 0.0095
    nr, na = 4, 24

    def heap(u, v):
        a = u * TAU
        r = R * (v ** 0.8)
        q = Vector((r * math.sin(a), 0, r * math.cos(a)))
        q.y = WB_FILL + 0.022 * (1 - v * v) + 0.012 * noise.noise(q * 18 + Vector((3.3, 1.1, 7.7)))
        q += Vector((0, rng.uniform(-0.003, 0.003), 0))
        return q
    s = _fix(grid_surface(heap, na, nr, wrap_u=True))
    for k in range(6):
        a = k / 6 * TAU + rng.uniform(-0.3, 0.3)
        rr = rng.uniform(0.03, R - 0.045)
        b = paper_ball(rng.uniform(0.024, 0.034), 40 + k, n=38, folds=4, lump=0.12, jag=0.13, pockets=3, flaps=1, flap_size=0.016)
        b.move(rr * math.sin(a), WB_FILL - 0.006 + rng.uniform(0.0, 0.012), rr * math.cos(a))
        s = s + b
    # it follows the dent, and never leaves the liner
    return s.warp(lambda p: _dent(p, WB_DENT[2] * 1.9))


def wastebasket_liner(seed=8):
    """The liner: a black film lining the can, turned out over the rim and
    hanging down outside it, longer here and shorter there, its slack drawn
    into folds."""
    R, H = WB_R, WB_H
    rng = random.Random(seed)
    ph = [rng.uniform(0, TAU) for _ in range(4)]
    n_in, n_rim, n_out = 6, 5, 4
    rows = n_in + n_rim + n_out

    def pt(u, v):
        a = u * TAU
        # hem: how far down the outside it hangs here
        drop = 0.050 + 0.018 * math.sin(2 * a + ph[0]) + 0.008 * math.sin(5 * a + ph[1])
        fold = math.sin(9 * a + ph[2] + 2.0 * math.sin(3 * a)) * 0.5 + 0.5
        j = v * rows
        if j <= n_in:
            t = j / n_in
            y = 0.024 + (H - 0.012 - 0.024) * t
            r = R - 0.0042 - 0.0035 * fold * (0.3 + 0.7 * t)
        elif j <= n_in + n_rim:
            t = (j - n_in) / n_rim
            ang = math.pi * t                          # round the rim bead
            cy, cr = H - 0.003, 0.0105
            r = R + 0.0005 - cr * math.cos(ang)
            y = cy + cr * math.sin(ang) + 0.0015
        else:
            t = (j - n_in - n_rim) / n_out
            y = H - 0.003 - drop * t
            r = R + 0.0074 + (0.002 + 0.014 * fold * fold) * t
        q = Vector((r * math.sin(a), y, r * math.cos(a)))
        q += noise3(q * 30 + Vector(ph[:3])) * 0.0012
        return q
    s = grid_surface(pt, 44, rows, wrap_u=True)
    # the bag's bottom on the can's raised bottom
    ring = list(range(44))
    s = fan_close(s, ring, (0.0, 0.022, 0.0))
    # dented deeper than the wall: its rows are coarser than the wall's, and
    # between them it must still stay inside it
    s = _fix(s).warp(lambda p: _dent(p, WB_DENT[2] * 1.9))
    # where the can lies on the floor the turned-over film is pressed flat
    return s.warp(lambda p: Vector((min(p.x, WB_FLOOR_X), p.y, p.z)))


def trash_bag(seed=3):
    """A small bin bag full of paper and tied (a 7-10 gal liner, 61 cm flat,
    stuffed): slumped on its bottom, the film drawn into pleats toward the
    neck, lumps of what is in it pushing out, the knot, its two ears."""
    rng = random.Random(seed)
    prof = [(0.0, 0.0), (0.105, 0.0), (0.140, 0.012), (0.158, 0.045), (0.160, 0.090), (0.152, 0.135),
            (0.133, 0.175), (0.104, 0.210), (0.070, 0.237), (0.040, 0.255), (0.022, 0.266), (0.015, 0.274),
            (0.016, 0.282), (0.024, 0.292), (0.026, 0.304), (0.017, 0.314), (0.0, 0.317)]
    s = lathe(prof, seg=34)
    o = Vector((rng.uniform(-20, 20), rng.uniform(-20, 20), rng.uniform(-20, 20)))

    def shape(p):
        a = math.atan2(p.x, p.z)
        y = p.y
        top = smoothstep(0.10, 0.262, y) * (1 - smoothstep(0.268, 0.28, y))
        # pleats: sharp folds drawn up into the neck
        pleat = 1.0 - abs(math.sin(4.5 * a + 0.7 + 0.6 * math.sin(2 * a) + 3 * y))
        k = 1.0 + 0.26 * top * (pleat - 0.5)
        body = 1 - smoothstep(0.23, 0.27, y)
        q = Vector((p.x * k * 1.10, y, p.z * k * 0.92))
        # what is inside pushes it out in lumps, the film crinkled over it;
        # it slumps forward a little
        q += noise3(q * 7 + o) * 0.018 * body
        q += noise3(q * 30 + o * 1.7) * 0.0065 * body
        q.x += 0.018 * smoothstep(0.08, 0.30, y)
        return q
    s.warp(shape)
    s.warp(lambda p: Vector((p.x, max(p.y, 0.0), p.z)))
    # the ears: the two ends of the tie, flat, crinkled, standing out of the knot
    for side, (yaw, lift, ln) in enumerate(((0.5, 0.9, 0.065), (-2.4, 0.6, 0.055))):
        c = Vector((0.018 + 0.006, 0.300, 0.0))
        pts = []
        for i in range(6):
            t = i / 5
            d = Vector((math.cos(yaw), 0, math.sin(yaw)))
            p = c + d * (ln * t) + Vector((0, ln * t * lift * (1 - 0.4 * t), 0))
            pts.append(p)
        ear = Shape()
        w0, w1 = 0.016, 0.026
        vv = []
        for i, p in enumerate(pts):
            t = i / 5
            w = w0 + (w1 - w0) * math.sin(t * math.pi * 0.8)
            side_v = Vector((-math.sin(yaw), 0, math.cos(yaw)))
            wob = noise3(p * 60 + o) * 0.004
            vv += [p + side_v * w / 2 + wob + Vector((0, 0.0007, 0)), p - side_v * w / 2 + wob + Vector((0, 0.0007, 0)),
                   p + side_v * w / 2 + wob - Vector((0, 0.0007, 0)), p - side_v * w / 2 + wob - Vector((0, 0.0007, 0))]
        f = []
        for i in range(5):
            a, b = i * 4, (i + 1) * 4
            f += [(a, a + 1, b + 1, b), (a + 2, b + 2, b + 3, a + 3), (a, b, b + 2, a + 2), (a + 1, a + 3, b + 3, b + 1)]
        f.append((20, 21, 23, 22))
        ear = _fix(Shape(vv, f))
        s = s + ear
    return on_floor(s)


# ------------------------------------------------------------------ sock --
# A men's crew sock laid flat: foot (heel to toe) 23 cm, leg (cuff to heel)
# 20 cm, 8.6 cm across flat, the ribbed cuff 5 cm (docs/research/props-remodelling/everyday_litter.md). It lies in
# profile, as a flat sock does: the heel the outer corner of its L.
SOCK_PATH = [(-0.070, -0.190), (-0.058, -0.120), (-0.040, -0.050), (-0.012, -0.004),
             (0.040, 0.020), (0.110, 0.034), (0.175, 0.040), (0.212, 0.036)]
SOCK_HEEL_S = 0.205              # the heel's arc length from the cuff


def _polyline(pts, smooth=3):
    p = [Vector((x, 0, z)) for x, z in pts]
    for _ in range(smooth):
        q = [p[0]]
        for a, b in zip(p, p[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(p[-1])
        p = q
    s = [0.0]
    for a, b in zip(p, p[1:]):
        s.append(s[-1] + (b - a).length)
    return p, s


def _sample(p, s, t):
    """Point and tangent at arc length t."""
    t = min(max(t, 0.0), s[-1])
    for i in range(len(s) - 1):
        if s[i + 1] >= t:
            k = (t - s[i]) / max(1e-9, s[i + 1] - s[i])
            tan = (p[i + 1] - p[i]).normalized()
            return p[i].lerp(p[i + 1], k), tan
    return p[-1], (p[-1] - p[-2]).normalized()


class Sock:
    """The sock's surface as a function of (s, phi): s the arc length along
    the path from the cuff (0) to the toe, phi round the flattened tube.
    Patches (heel, toe, bands) are read off the same function a little
    proud of it."""

    def __init__(self, seed=6):
        self.p, self.s = _polyline(SOCK_PATH)
        self.L = self.s[-1]
        rng = random.Random(seed)
        self.o = Vector((rng.uniform(-20, 20), rng.uniform(-20, 20), rng.uniform(-20, 20)))
        tin = _sample(self.p, self.s, SOCK_HEEL_S - 0.05)[1]
        tout = _sample(self.p, self.s, SOCK_HEEL_S + 0.05)[1]
        self.outer = (tin - tout).normalized()
        self.ph = [rng.uniform(0, TAU) for _ in range(6)]

    def width(self, s):
        a = 0.043
        a += 0.003 * math.exp(-((s - SOCK_HEEL_S) / 0.04) ** 2)
        if s < 0.05:
            a += 0.002 * (1 - s / 0.05)                  # the cuff, a shade wider
        a *= 1 + 0.05 * noise.noise(Vector((s * 11, 4.2, 0.3)) + self.o)
        toe0 = self.L - 0.045
        if s > toe0:
            u = (s - toe0) / (self.L - toe0)
            a *= math.sqrt(max(0.0, 1 - u ** 3))         # a blunt toe
        return a

    def height(self, s):
        b = 0.0033
        b += 0.0020 * math.exp(-((s - SOCK_HEEL_S) / 0.035) ** 2)   # the heel pocket stands up
        if s < 0.05:
            b += 0.0010
        toe0 = self.L - 0.045
        if s > toe0:
            u = (s - toe0) / (self.L - toe0)
            b *= math.sqrt(max(0.0, 1 - u ** 3)) * 0.85 + 0.15 * (1 - u)
        return b

    def point(self, s, phi, off=0.0):
        c, t = _sample(self.p, self.s, s)
        n = Vector((-t.z, 0, t.x))
        a = self.width(s) + off
        b = self.height(s) + off * 0.8
        cs, sn = math.cos(phi), math.sin(phi)
        # its edges are rounded, its faces flat-ish (a lens, squared off)
        ex = math.copysign(abs(cs) ** 0.7, cs)
        ey = math.copysign(abs(sn) ** 0.8, sn)
        q = c + n * (a * ex)
        # the heel's pocket bulges out of the outer corner
        side = n.dot(self.outer) * ex
        q += self.outer * 0.024 * math.exp(-((s - SOCK_HEEL_S) / 0.032) ** 2) * max(0.0, side) ** 1.5
        # limp: the top layer sags and wrinkles across, the edges wander
        y = b + b * ey
        if ey > 0:
            # folds running slant across it, where it was dropped
            wr = noise.noise(Vector((s * 26 + cs * 0.9, 7.7, 1.1)) + self.o) * 0.0040
            wr += noise.noise(Vector((s * 60 - cs * 1.4, 2.3, 5.5)) + self.o) * 0.0015
            # a ridge where the leg was crumpled over
            wr += 0.004 * math.exp(-((s - 0.085 - 0.012 * cs) / 0.010) ** 2)
            y += (wr + 0.0010) * ey
        q += n * noise.noise(Vector((s * 18, 1.3, 2.9)) + self.o) * 0.0035
        # the leg lies rolled a little onto one edge
        y += 0.0035 * cs * (1 - smoothstep(0.10, SOCK_HEEL_S, s)) * (0.5 + 0.5 * ey)
        # the cuff turned up a little off the floor, its mouth open
        y += 0.008 * max(0.0, 1 - s / 0.05) ** 2 * (0.6 + 0.4 * cs) * (1 if ey > -0.2 else 0.3)
        q.y = max(y + off * 0.3, 0.0)
        return q

    def body(self, nt=32, nphi=16):
        L = self.L

        def fn(u, v):
            s = 0.0015 + (L - 0.0015) * (v ** 0.95)
            return self.point(s, u * TAU)
        sh = grid_surface(fn, nphi, nt, wrap_u=True)
        # the toe closes at its tip
        top = [nt * nphi + i for i in range(nphi)]
        sh = fan_close(sh, top, self.point(L, 0.0))
        # the cuff's opening: the knit turns in and goes back inside a little
        inner = grid_surface(lambda u, v: self.point(0.0015 + 0.03 * v, -u * TAU, off=-0.0018 - 0.001 * v), nphi, 3, wrap_u=True)
        n0 = len(sh.v)
        sh = sh + inner
        # join the rim of the outer (row 0) to the rim of the inner (row 0)
        for i in range(nphi):
            j = (i + 1) % nphi
            # inner ring runs the other way round: index of angle u is (nphi - i) % nphi
            ii, jj = (nphi - i) % nphi, (nphi - j) % nphi
            sh.f.append((j, i, n0 + ii, n0 + jj))
        sh = fan_close(sh, [n0 + 3 * nphi + i for i in range(nphi)], self.point(0.032, 0.0, off=-0.004))
        return _fix(sh)

    def patch(self, s0, s1, phi0, phi1, off, nt, nphi, wrap=False):
        """A skin over the sock between s0(phi) and s1(phi) (numbers or
        functions of phi), phi from phi0 to phi1 (or all round)."""
        f0 = s0 if callable(s0) else (lambda ph: s0)
        f1 = s1 if callable(s1) else (lambda ph: s1)

        def fn(u, v):
            ph = u * TAU if wrap else phi0 + (phi1 - phi0) * u
            a, b = f0(ph), f1(ph)
            return self.point(a + (b - a) * v, ph, off)
        return _fix(grid_surface(fn, nphi, nt, wrap_u=wrap))

    def heel_toe(self):
        # the heel: over the outer corner, both layers; which way is out
        c, t = _sample(self.p, self.s, SOCK_HEEL_S)
        n = Vector((-t.z, 0, t.x))
        mid = 0.0 if n.dot(self.outer) > 0 else math.pi
        # a rounded patch: longest over the heel's point, narrowing to its sides
        half = lambda ph: 0.012 + 0.040 * math.sqrt(max(0.0, 1 - ((ph - mid) / 1.3) ** 2))
        heel = self.patch(lambda ph: SOCK_HEEL_S - half(ph), lambda ph: SOCK_HEEL_S + 0.005 + half(ph), mid - 1.3, mid + 1.3, 0.0006, 8, 12)
        # the toe: its seam arched back toward the sole
        toe = self.patch(lambda ph: self.L - 0.044 - 0.012 * max(0.0, math.cos(ph - mid)), self.L - 0.0005, 0, 0, 0.0006, 6, 20, wrap=True)
        return heel + toe

    def bands(self):
        return self.patch(0.014, 0.024, 0, 0, 0.0006, 1, 20, wrap=True) + self.patch(0.032, 0.042, 0, 0, 0.0006, 1, 20, wrap=True)


SOCK = Sock()


# ------------------------------------------------------------ crown caps --
# A crown cork: tinplate 0.2 mm, 32.1 mm across and 6 mm tall as made, 21
# flutes; crimped onto the bottle and prised off it is about 29 mm across,
# the flutes pressed in (docs/research/props-remodelling/everyday_litter.md). Built standing on its skirt, its top
# up; a plastic liner inside the top.
CAP_N = 21
CAP_H = 0.0060
CAP_TOP_R = 0.0122


# the crown's outer surface, bottom to the top panel's edge: (r, y, flute depth)
CAP_ROWS = [(0.01455, 0.0, 0.0012), (0.01425, 0.0030, 0.0007), (0.0137, 0.0053, 0.0002), (CAP_TOP_R, CAP_H, 0.0)]


def _zigzag(a):
    """The flutes round the skirt: -1..1, 21 times round."""
    return abs(((a * CAP_N / TAU) % 1.0) * 2 - 1) * 2 - 1


def cap_crown(bend=None):
    """The crown: its fluted skirt from the cut edge up over the knuckle to
    the top panel's edge; the edge's thickness; the skirt's inside up to
    the liner, and the underside of the top."""
    nseg = CAP_N * 2
    v = []

    def ring(r, y, depth, inset=0.0):
        ids = []
        for i in range(nseg):
            a = TAU * i / nseg
            rr = r + depth * _zigzag(a) - inset
            v.append(Vector((rr * math.sin(a), y, rr * math.cos(a))))
            ids.append(len(v) - 1)
        return ids
    outer = [ring(r, y, d) for r, y, d in CAP_ROWS]
    inner = [ring(CAP_ROWS[0][0], 0.0002, CAP_ROWS[0][2], 0.00035), ring(0.0131, 0.0053, 0.0002)]
    s = Shape(v, [])

    def band(A, B, flip=False):
        for i in range(nseg):
            j = (i + 1) % nseg
            s.f.append((A[j], A[i], B[i], B[j]) if flip else (A[i], A[j], B[j], B[i]))
    for A, B in zip(outer, outer[1:]):
        band(A, B)
    band(outer[0], inner[0], flip=True)
    for A, B in zip(inner, inner[1:]):
        band(A, B, flip=True)
    s = fan_close(s, list(reversed(inner[-1])), (0, 0.0054, 0))
    s = _fix(s)
    return bend(s) if bend else s


def cap_top(bend=None):
    s = lathe([(0.0, CAP_H + 0.0003), (0.006, CAP_H + 0.00025), (0.0105, CAP_H + 0.0001), (CAP_TOP_R + 0.0001, CAP_H - 0.00005)],
              seg=CAP_N * 2, cap_bottom=False, cap_top=False)
    return bend(s) if bend else s


def cap_liner(bend=None):
    s = lathe([(0.0, 0.0052), (0.0088, 0.0052), (0.0101, 0.0046), (0.0118, 0.0050), (0.0124, 0.0054)],
              seg=CAP_N * 2, cap_bottom=False, cap_top=False)
    s = fan_close(s, [len(s.v) - CAP_N * 2 + i for i in range(CAP_N * 2)], (0, 0.0054, 0))
    s = _fix(s)
    return bend(s) if bend else s


def cap_bend(s):
    """The opener's work: its lip under the skirt at +z, its fulcrum on the
    top, the far half of the cap levered up about a crease across it, and
    the flute it caught opened out."""
    z0, yh, beta, w = -0.0015, CAP_H, math.radians(32), 0.0025
    out = s.copy()
    for i, p in enumerate(out.v):
        t = smoothstep(z0 - w, z0 + w, p.z)
        a = beta * t
        dy, dz = p.y - yh, p.z - z0
        if t > 0:
            p = Vector((p.x, yh + dy * math.cos(a) + dz * math.sin(a), z0 + dz * math.cos(a) - dy * math.sin(a)))
        # the caught flute, opened out
        ang = math.atan2(p.x, p.z)
        k = math.exp(-(ang / 0.35) ** 2) * (1 - smoothstep(0.0, 0.004, s.v[i].y))
        rr = math.hypot(p.x, p.z)
        if rr > 1e-6:
            p = Vector((p.x * (1 + 0.12 * k), p.y - 0.0012 * k, p.z * (1 + 0.12 * k)))
        out.v[i] = p
    return out


def _bent_floor():
    """How far the bent cap must come up to rest on the floor (all its parts
    together)."""
    lo = min(p.y for p in cap_crown(cap_bend).v)
    return -lo


def _lift(s, dy):
    return s.move(0, dy, 0)


# ------------------------------------------------------------ soda cans --
# A 355 ml (12 fl oz) two-piece aluminium can: a 211 body (66 mm), 122 mm
# tall, necked to a 202 end (54 mm across the double seam) (docs/research/props-remodelling/everyday_litter.md).
CAN_R = 0.0330
CAN_END_R = 0.0270
# the crush's phases (the game paints the label's cracks along its folds
# from the same numbers: placers_core `crushed_soda_cans.rs` `crushed_can_label`)
CAN_PH = (1.3, 4.1, 0.6, 2.9, 5.2)


class CrushedCan:
    """The can stood on its end and stamped on: the wall's 93 mm folded into
    rings that bulge out (deepest on the side the heel came down), the top
    slid off centre and tipped, the bottom dome pushed up."""

    def __init__(self, seed=4):
        rng = random.Random(seed)
        self.ph = CAN_PH
        self.o = Vector((rng.uniform(-20, 20), rng.uniform(-20, 20), rng.uniform(-20, 20)))
        self.wall0 = 0.012          # where the wall starts above the floor (the stand ring's top)
        self.wall1 = 0.019          # where it ends crushed (was 0.105)
        self.tip = math.radians(17)  # the top's tilt (about x): the heel came down on one side
        self.slide = Vector((0.008, 0, -0.006))

    def wall_point(self, a, t):
        """t from 0 (the bottom's shoulder) to 1 (the neck)."""
        folds = 3.0
        ph = self.ph
        # folds: rings that bulge out and in, out of step round the can
        lag = 0.18 * math.sin(a + ph[0]) + 0.08 * math.sin(2 * a + ph[1])
        x = (t * folds + lag) % 1.0
        tri = abs(x * 2 - 1)                                      # 1 at a fold's crest
        side = 0.40 + 0.60 * math.cos(a - ph[2])                   # the heel's side bulges most
        lobes = 0.75 + 0.25 * math.cos(3 * a + ph[4] + 2 * t)     # and it buckles in lobes round it
        env = math.sin(math.pi * t) ** 0.5
        # sharp creases down the folds too, where the wall buckled in diamonds
        crease = abs(math.sin(3.5 * a + ph[4] + 3 * t)) * 0.004
        bulge = (tri ** 1.4 - 0.3) * 0.024 * side * lobes * env + crease * env
        r = CAN_R + bulge + 0.0030 * env * noise.noise(Vector((math.cos(a) * 3, t * 9, math.sin(a) * 3)) + self.o)
        y = self.wall0 + (self.wall1 - self.wall0) * t + 0.0012 * math.sin(3 * a + ph[3]) * math.sin(math.pi * t)
        # the upper wall turns with the top as it tips
        th = self.tip * smoothstep(0.3, 1.0, t)
        x0, z0 = r * math.sin(a), r * math.cos(a)
        q = Vector((x0, y - z0 * math.sin(th), z0 * math.cos(th)))
        q += self.slide * smoothstep(0.2, 1.0, t)
        return q

    def body(self, nu=32, nv=10):
        def fn(u, v):
            return self.wall_point(u * TAU, v)
        return _fix(grid_surface(fn, nu, nv, wrap_u=True))

    def top_frame(self, p):
        """Carry a point of the lid (modelled at y from 0, its seam at the
        top) onto the crushed can's top: tipped and slid."""
        q = Vector((p.x, p.y, p.z))
        c, s = math.cos(self.tip), math.sin(self.tip)
        q = Vector((q.x, q.y * c - q.z * s, q.y * s + q.z * c))
        return q + Vector((0, self.wall1, 0)) + self.slide

    def ends(self):
        lid = can_lid()
        lid.warp(self.top_frame)
        return can_bottom(self.wall0) + lid


def can_bottom(top=0.012):
    """The bottom: the stand ring and the dome, up to the wall at `top`
    (axis +y, the stand ring on y = 0)."""
    return lathe([(0.0, 0.0085), (0.019, 0.0070), (0.0245, 0.0018), (0.0262, 0.0),
                  (0.0290, 0.0016), (0.0318, 0.0068), (CAN_R + 0.0002, top + 0.0005)],
                 seg=24, cap_bottom=False, cap_top=False)


def can_lid():
    """The top (axis +y, from the wall at y = 0): the neck in to the double
    seam, the countersink, the panel; the drinking hole pushed in, its
    flap hanging inside; the tab riveted at the centre, lying back."""
    neck = [(CAN_R + 0.0001, -0.0006), (CAN_R - 0.0015, 0.0030), (CAN_END_R + 0.0004, 0.0080),
            (CAN_END_R + 0.0002, 0.0100), (CAN_END_R - 0.0010, 0.0106), (CAN_END_R - 0.0020, 0.0060),
            (CAN_END_R - 0.0030, 0.0043), (CAN_END_R - 0.0040, 0.0051), (0.0, 0.0055)]
    top = lathe(neck, seg=24, cap_bottom=False, cap_top=False)
    hole = rbox(0.013, 0.010, 0.019, r=0.004, seg=2).move(0, 0.0053, 0.0118)
    top = cut(top, hole)
    flap = prism(round_outline([(-0.0058, -0.008), (0.0058, -0.008), (0.0048, 0.008), (-0.0048, 0.008)], 0.0035, seg=2), 0.0004)
    flap.rot(x=math.pi / 2 - 0.9).move(0, 0.0010, 0.0105)
    tab = prism(rounded_rect(0.0125, 0.0215, 0.0045, seg=3), 0.0007)
    tab = cut(tab, prism(rounded_rect(0.0075, 0.0085, 0.003, seg=2), 0.002).move(0, -0.0045, 0))
    tab.rot(x=math.pi / 2).rot(x=-0.06).move(0, 0.0064, -0.0035)
    rivet = cyl(0.0022, h=0.0010, seg=10, bevel_r=0.0004, bevel_seg=1).move(0, 0.0059, 0.0)
    return top + flap + tab + rivet


class FlatCan:
    """The can lying on its side and stepped on: its wall pressed flat in the
    middle (a sheet folded at its two edges, 11 mm thick, as wide as half
    its round), its stiff ends only half squashed, the top layer sloping
    down from them, creased where it went flat. Built lying: its axis along
    x, the lid at +x."""

    def __init__(self):
        self.len = 0.118
        self.h = 0.011            # the flat middle's thickness
        self.fe = 0.5             # how far its ends were squashed (0 round, 1 flat)
        self.x0 = -self.len / 2 + 0.012
        self.x1 = self.len / 2 - 0.0106
        self.o = Vector((3.1, -7.7, 12.2))

    def squash(self, r, a, f):
        """A point of a ring of radius r at angle a (round: y = r cos a, z =
        r sin a) pressed flat by f onto the floor at y = -CAN_R."""
        aa = (a + math.pi) % TAU - math.pi
        top = abs(aa) <= math.pi / 2
        fz = r * aa if top else r * (math.copysign(math.pi, aa) - aa)
        k = abs(math.cos(aa)) ** 0.35
        hh = self.h * r / CAN_R
        fy = -CAN_R + hh * (0.5 + 0.5 * k) if top else -CAN_R + hh * 0.5 * (1 - k)
        ry, rz = r * math.cos(a), r * math.sin(a)
        return ry + (fy - ry) * f, rz + (fz - rz) * f

    def flatness(self, t):
        return self.fe + (0.96 - self.fe) * smoothstep(0.0, 0.32, t) * smoothstep(1.0, 0.68, t)

    def body(self, nu=32, nv=12):
        def fn(u, v):
            a = u * TAU
            f = self.flatness(v)
            y, z = self.squash(CAN_R, a, f)
            x = self.x0 + (self.x1 - self.x0) * v
            mid = smoothstep(0.0, 0.3, v) * smoothstep(1.0, 0.7, v)
            if math.cos(a) > 0:
                # creased where it went flat: folds slanting in from its ends
                y += 0.0022 * noise.noise(Vector((x * 70, z * 70, 0.5)) + self.o) * mid
                y -= 0.0025 * abs(math.sin((x - 0.6 * z) * 90 + 1.0)) * (1 - mid) * f
            z += 0.0015 * noise.noise(Vector((x * 50, 1.7, a)) + self.o)
            return Vector((x, y, z))
        return _fix(grid_surface(fn, nu, nv, wrap_u=True))

    def ends(self):
        def end(shape, x_at, sign):
            def m(p):
                # (x, y, z) of the standing part -> (along, x, -z): turned, not mirrored
                r = math.hypot(p.x, p.z)
                a = math.atan2(-p.z, p.x)
                y, z = self.squash(r, a, self.fe)
                return Vector((x_at + sign * p.y, y, z))
            return shape.warp(m)
        return end(can_bottom(0.012), self.x0 - 0.012, 1.0) + end(can_lid(), self.x1, 1.0)


CAN = CrushedCan()
FLAT = FlatCan()


# ---------------------------------------------------------------- parts ---
def built(build, kind='box'):
    """A part's shape and its frame: its own bounds (the game places it as
    built, at a scale of one)."""
    shape = build()
    lo, hi = shape.bounds()
    return shape, f_bounds(tuple(lo), tuple(hi), kind)


def auto(name, build, smooth=35.0, flat=False, kind='box'):
    """Register a part within the budget of its size (`budget_for`)."""
    shape, frame = built(build, kind)
    part(name, frame, lambda: shape, smooth=smooth, flat=flat)


# the paper, within the budgets of their sizes
auto('Real__paperBall', lambda: paper_ball(0.033, 11, n=104, folds=9, lump=0.10, jag=0.13, pockets=6, flaps=3, flap_size=0.020), flat=True)
auto('Real__paperWad', lambda: paper_ball(0.040, 23, n=118, folds=7, lump=0.17, jag=0.10, pockets=8, squash=0.8, flaps=4, flap_size=0.028), flat=True)
auto('Real__paperSheet', paper_sheet, flat=True)
# the wastebasket: a budget of its own for the rolled rim, the chime, the
# beads and a dent the wall has rows for, inside and out
_s, _f = built(wastebasket, 'cyl')
part('Real__wastebasket', _f, lambda s=_s: s, budget=1800)
_s, _f = built(wastebasket_liner)
part('Real__wastebasketLiner', _f, lambda s=_s: s, smooth=60.0, budget=1400)
auto('Real__wastebasketTrash', wastebasket_trash, flat=True)
_s, _f = built(trash_bag)
part('Real__trashBag', _f, lambda s=_s: s, smooth=40.0, budget=1400)
# the sock: a flattened tube with its heel, toe and cuff
_s, _f = built(SOCK.body)
part('Real__sock', _f, lambda s=_s: s, smooth=70.0, budget=1200)
auto('Real__sockHeelToe', SOCK.heel_toe, smooth=70.0)
auto('Real__sockBands', SOCK.bands, smooth=70.0)
# a crown cap: 21 flutes want 42 sides, its skirt and its inside a few rows
_s, _f = built(cap_crown, 'cyl')
part('Real__capCrown', _f, lambda s=_s: s, smooth=25.0, budget=480)
auto('Real__capTop', cap_top)
_s, _f = built(cap_liner)
part('Real__capLiner', _f, lambda s=_s: s, budget=360)
BENT_LIFT = _bent_floor()
_s, _f = built(lambda: _lift(cap_crown(cap_bend), BENT_LIFT), 'cyl')
part('Real__capCrownBent', _f, lambda s=_s: s, smooth=25.0, budget=480)
auto('Real__capTopBent', lambda: _lift(cap_top(cap_bend), BENT_LIFT))
_s, _f = built(lambda: _lift(cap_liner(cap_bend), BENT_LIFT))
part('Real__capLinerBent', _f, lambda s=_s: s, budget=360)
# the crushed can: its folds, and its ends (the double seam, the
# countersink, the opening and the tab) are what make it a can
_s, _f = built(CAN.body, 'cyl')
part('Real__canCrushedBody', _f, lambda s=_s: s, smooth=40.0, budget=700)
_s, _f = built(CAN.ends)
part('Real__canCrushedEnds', _f, lambda s=_s: s, smooth=40.0, budget=1000)
_FLAT_B, _FLAT_E = FLAT.body(), FLAT.ends()
FLAT_LIFT = -min(_FLAT_B.bounds()[0].y, _FLAT_E.bounds()[0].y)
_s, _f = built(lambda: _FLAT_B.copy().move(0, FLAT_LIFT, 0))
part('Real__canFlatBody', _f, lambda s=_s: s, smooth=40.0, budget=800)
_s, _f = built(lambda: _FLAT_E.copy().move(0, FLAT_LIFT, 0))
part('Real__canFlatEnds', _f, lambda s=_s: s, smooth=40.0, budget=1000)
