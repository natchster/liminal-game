"""r10: what a floor's weather leaves (`core_props/weather_fx.rs`,
docs/design/PROPS.md P2), remodelled at their real sizes. Sources and sizes:
docs/research/props-remodelling/what_a_floors_weather_leaves.md. (The wet floors' puddles are painted, not modelled.)

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
prop's own frame: x across its front, y up from the floor (y = 0 the floor
it rests on), z out of its front.

Derelict floors: what falls from a suspended ceiling.

derelictTilePieceA..D   the four pieces a 610 x 610 x 15.9 mm mineral-fibre
                        lay-in tile broke into when it fell (Armstrong Fine
                        Fissured 1728): each in the whole tile's frame (the
                        tile centred on the origin, its back on the floor),
                        so the four fit together; the fibre core: the
                        tile's two square-cut factory edges, the breaks
                        ragged and fibrous, the core crumbled back under the
                        painted face, the unpainted back
derelictTileFaceA..D    each piece's painted face, a skin 0.5 mm on top of
                        it (the game paints its fissures, its yellowing and
                        the leak that brought it down)
derelictTeeSteelA/B     a 2 ft cross tee of the grid (Prelude XL 15/16 in:
                        610 mm long, its face 23.8 mm, 35 mm high), bent
                        where it was pulled down, lying on its side on the
                        floor: the double web, the bulb along its top, the
                        flange, the stab tabs at its ends
derelictTeeCapA/B       the white painted cap over its face (the flange's
                        underside, seen as it lies)

Electric floors: a cut length of building cable.

electricCableSheathA/B  a length of NM-B 12/2 with ground (Southwire
                        Romex SIMpull: a flat yellow PVC jacket 10.7 x 4.8
                        mm), lying slack in a loose loop on the floor, its
                        cut end lifted over the loop, the jacket cut back
                        there; the far end cut flush. Both end at the same
                        place, so the end's parts below fit either
electricCableCoreBlack  the black conductor (12 AWG solid copper, PVC and
                        nylon, 3 mm) out of the jacket, splayed up
electricCableCoreWhite  the white one, splayed the other way, its end
                        bent back toward the black's (where the arc jumps)
electricCableCopper     their stripped ends (2.05 mm copper) and the bare
                        ground wire beside them
electricCablePaper      the paper wrapped round the ground, torn off where
                        the jacket was cut

Haunted floors: a pillar candle burnt down (3 x 6 in, 76 mm across).

hauntedCandleBodyA/B    the candle as it has burnt: A to 105 mm, the rim
                        standing 8-12 mm over its pool with a notch where the
                        wax ran over; B to 72 mm, tunnelled, the rim 14-20
                        mm high and slumped on one side; wax run down the
                        side from the notch in runs ending in drops, and
                        spread round its foot on the floor
hauntedCandlePoolA/B    the pool of melted wax in its crater (the top of it)
hauntedCandleWick       the braided wick standing out of the pool, curled
                        over at its tip, the tip a carbon knob (its foot at
                        y = 0: the game stands it on the pool)
"""
import math
import random
import zlib

import bmesh
from propkit import *
import propkit


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name (the
    shape is built here, at registration). The budget is the last argument
    (the asset lint reads it there); a build's own smoothing angle rides on
    it (`smooth_at`)."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget, smooth=getattr(build, 'smooth', 35.0))


def smooth_at(deg, build):
    """`build`, its part's normals smoothed across edges up to `deg` degrees"""
    build.smooth = deg
    return build


def _fix(s):
    return propkit._fix_winding(s)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def jagged(a, b, depth, rough, rng):
    """A jagged path from a to b (2D): midpoint displacement, `depth` times,
    each midpoint pushed sideways by up to `rough` of its span."""
    pts = [a, b]
    for _ in range(depth):
        nxt = [pts[0]]
        for p, q in zip(pts, pts[1:]):
            off = rng.uniform(-rough, rough)
            nxt.append(((p[0] + q[0]) / 2 - (q[1] - p[1]) * off, (p[1] + q[1]) / 2 + (q[0] - p[0]) * off))
            nxt.append(q)
        pts = nxt
    return pts


def resample(pts, step):
    """A polyline (2D) resampled every `step` along its length (its ends kept)."""
    out = [pts[0]]
    acc = 0.0
    for p, q in zip(pts, pts[1:]):
        L = math.hypot(q[0] - p[0], q[1] - p[1])
        t = 0.0
        while acc + (L - t) >= step:
            t += step - acc
            acc = 0.0
            out.append((p[0] + (q[0] - p[0]) * t / L, p[1] + (q[1] - p[1]) * t / L))
        acc += L - t
    if math.hypot(out[-1][0] - pts[-1][0], out[-1][1] - pts[-1][1]) < step * 0.4:
        out[-1] = pts[-1]
    else:
        out.append(pts[-1])
    return out


# ======================================================== the ceiling tile ===
TILE = 0.610        # 24 in
TILE_T = 0.0159     # 5/8 in
FACE_T = 0.0005     # the painted face
BODY_T = TILE_T - FACE_T
H = TILE / 2


def tile_breaks(seed):
    """Where the tile broke: four jagged breaks from a point near its middle
    (where it hit the floor first) out to its four edges, and the outline of
    each corner piece: [(x, z, broken)] round it."""
    rng = random.Random(seed)
    c = (rng.uniform(-0.04, 0.04), rng.uniform(-0.04, 0.04))
    # where each break meets an edge: the -z, +x, +z and -x edges, in turn
    ends = [(rng.uniform(-0.12, 0.12), -H), (H, rng.uniform(-0.12, 0.12)), (rng.uniform(-0.12, 0.12), H), (-H, rng.uniform(-0.12, 0.12))]
    breaks = []
    for e in ends:
        # a little wander at the scale of the piece, then the fibre's ragged zigzag
        coarse = jagged(c, e, 3, 0.09, rng)
        fine = []
        for p, q in zip(coarse, coarse[1:]):
            seg = jagged(p, q, 3, 0.16, rng)
            fine += seg[:-1]
        fine.append(e)
        breaks.append(resample(fine, 0.0075))
    # the tile's corners, between the breaks' ends (counter-clockwise in x, z)
    corners = [(H, -H), (H, H), (-H, H), (-H, -H)]
    pieces = []
    for k in range(4):
        b0, b1 = breaks[k], breaks[(k + 1) % 4]
        out = [(x, z, True) for x, z in b0[:-1]]
        out.append((b0[-1][0], b0[-1][1], False))
        out.append((corners[k][0], corners[k][1], False))
        back = list(reversed(b1[1:]))
        out.append((back[0][0], back[0][1], False))
        out += [(x, z, True) for x, z in back[1:]]
        # (the break point c closes the outline: b1[0] == b0[0])
        pieces.append(out)
    return pieces


TILE_PIECES = tile_breaks(1001)


def _outline_normals(pts):
    """Each outline point's outward normal (x, z): the outline runs
    counter-clockwise or clockwise; the sign is fixed by its area."""
    n = len(pts)
    area = sum(pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1] for i in range(n))
    sgn = 1.0 if area > 0 else -1.0
    out = []
    for i in range(n):
        a, b = pts[i - 1], pts[(i + 1) % n]
        tx, tz = b[0] - a[0], b[1] - a[1]
        L = math.hypot(tx, tz) or 1.0
        out.append((tz / L * sgn, -tx / L * sgn))
    return out


def tile_piece(k):
    """Piece k's fibre core: its back (a flat face on the floor) and its
    edges, three rows up them: the factory edges straight and square, a
    hair's chamfer at the top; the breaks ragged, the fibre crumbled back
    from the painted face, which overhangs it a little."""
    pts = TILE_PIECES[k]
    nrm = _outline_normals([(x, z) for x, z, _ in pts])
    rng = random.Random(4000 + k)
    v, f = [], []
    n = len(pts)
    rows = 3
    for (x, z, broken), (nx, nz) in zip(pts, nrm):
        q = Vector((x * 37.0, 0.0, z * 37.0))
        for r in range(rows):
            if broken:
                y = (0.0, BODY_T * 0.55, BODY_T)[r]
                # torn fibre: in and out along the break, the middle eaten back
                push = noise.noise(q + Vector((r * 3.1, r * 1.7, 0.0))) * 0.0016 + rng.uniform(-0.0006, 0.0006)
                push += (0.0, -0.0014, -0.0005)[r]
            else:
                y = (0.0, BODY_T - 0.0007, BODY_T)[r]
                push = (0.0, 0.0, -0.0007)[r]
            v.append(Vector((x + nx * push, y, z + nz * push)))
    for i in range(n):
        j = (i + 1) % n
        for r in range(rows - 1):
            f.append((i * rows + r, j * rows + r, j * rows + r + 1, i * rows + r + 1))
    # the back
    f.append(tuple(i * rows for i in range(n)))
    s = Shape(v, f)
    # the outline's factory corners stay sharp
    return _fix(s)


def tile_face(k):
    """Piece k's painted face: a skin FACE_T thick over the core's top row,
    its edge broken where the core's is."""
    core = tile_piece(k)
    rows = 3
    pts = TILE_PIECES[k]
    n = len(pts)
    top = [core.v[i * rows + 2].copy() for i in range(n)]
    v = [Vector((p.x, BODY_T, p.z)) for p in top] + [Vector((p.x, TILE_T, p.z)) for p in top]
    f = [tuple(range(n, 2 * n))]
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
    return _fix(Shape(v, f))


# ============================================================ the cross tee ===
TEE_L = 0.610       # 24 in
TEE_W = 0.0238      # 15/16 in face
TEE_HT = 0.0349     # 1-3/8 in
STEEL = 0.0009      # the web's double thickness (two 0.4 mm walls), drawn a hair thick


def tee_cuts(at):
    """Where the tee is cut across so it can bend: every 7 mm about the kink
    at x = `at`, sparser out along its arms."""
    xs = [at + 0.007 * i for i in range(-7, 8)]
    xs += [at - 0.1, at - 0.2, at + 0.1, at + 0.2]
    return sorted(x for x in xs if abs(x) < TEE_L / 2 - 0.01)


def tee_upright(cap, at):
    """The straight tee standing on its face along x: the face (flange) at
    y = 0 across z, the web up the middle, the bulb along its top, the stab
    tabs past each end. `cap`: the white cap under the face instead."""
    cuts = tee_cuts(at)
    if cap:
        return _cut_x(box(TEE_L, 0.0005, TEE_W + 0.0008).move(0, -0.0007, 0), cuts)
    flange = _cut_x(box(TEE_L, STEEL, TEE_W).move(0, -STEEL / 2 + 0.0002, 0), cuts)
    # the flange's edges are hemmed: folded under, a rounded lip
    lips = [_cut_x(tube([(-TEE_L / 2, -0.0002, sz * (TEE_W / 2 - 0.0006)), (TEE_L / 2, -0.0002, sz * (TEE_W / 2 - 0.0006))], 0.0009, seg=5), cuts)
            for sz in (-1, 1)]
    web_h = TEE_HT - 0.006
    web = _cut_x(box(TEE_L - 0.004, web_h, STEEL).move(0, web_h / 2, 0), cuts)
    bulb = _cut_x(rbox(TEE_L - 0.004, 0.0065, 0.0042, r=0.0015, seg=1).move(0, TEE_HT - 0.00325, 0), cuts)
    # the stab tabs: the web run on 13 mm past each end, a hook at its foot
    tabs = []
    for sx in (-1, 1):
        t = prism([(0, 0.004), (0.013, 0.006), (0.013, 0.020), (0.006, 0.026), (0, 0.026)], 0.0008)
        if sx < 0:
            t.rot(0, math.pi, 0)
        tabs.append(t.move(sx * (TEE_L / 2 - 0.003), 0, 0))
    return flange + sum(lips) + web + bulb + sum(tabs)


def _cut_x(s, xs):
    """The shape cut across x at each of `xs` (its faces split there, so it
    can bend)."""
    bm = propkit.to_bmesh(s)
    for x in xs:
        geom = bm.verts[:] + bm.edges[:] + bm.faces[:]
        bmesh.ops.bisect_plane(bm, geom=geom, plane_co=(x, 0, 0), plane_no=(1, 0, 0))
    out = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


def tee_crumple(s, at, buckle, seed):
    """The upright tee crumpled where it was bent at x = `at`: the web
    buckled sideways (across z), more toward its top, and pushed down."""
    out = s.copy()
    for p in out.v:
        d = p.x - at
        w = math.exp(-(d / 0.02) ** 2)
        p.z += buckle * w * (p.y / TEE_HT) * math.sin(d * 260.0 + seed)
        p.y -= abs(buckle) * 0.4 * w * (p.y / TEE_HT)
    return out


def tee_swing(s, at, angle):
    """The tee as it lies, bent about the vertical at x = `at`: the part past
    it swung round by `angle` over a few centimetres (a smooth hinge, so it
    stays on the floor as it lay)."""
    span = 0.03
    out = s.copy()
    for p in out.v:
        d = p.x - at
        a = angle * smoothstep(-span, span, d)
        x, z = d, p.z
        p.x = at + x * math.cos(a) + z * math.sin(a)
        p.z = -x * math.sin(a) + z * math.cos(a)
    return out


def tee_lying(cap, at, angle, buckle, seed):
    """The tee crumpled, laid on its side as it lies on the floor (on the
    hem of its face and the bulb, its face and its white cap toward +z),
    then bent where it crumpled."""
    def lay(s):
        s = tee_crumple(s, at, buckle, seed)
        # on its side: the face stood up facing +z, the web out toward -z,
        # tipped down until the bulb meets the floor
        s.rot(-math.pi / 2, 0, 0)
        s.rot(-math.atan2(TEE_W / 2, TEE_HT), 0, 0)
        return tee_swing(s, at, angle)
    steel = lay(tee_upright(False, at))
    # centred on the steel, on the floor (the cap moved with it)
    lo, hi = steel.bounds()
    s = lay(tee_upright(True, at)) if cap else steel
    return s.move(-(lo.x + hi.x) / 2, -lo.y, -(lo.z + hi.z) / 2)


TEE_A = (0.04, 0.55, 0.004, 1.0)
TEE_B = (-0.12, -0.32, 0.006, 2.3)


# ================================================================ the cable ===
CAB_W, CAB_T = 0.0107, 0.0048       # NM-B 12/2 W/G, Southwire: 0.422 x 0.190 in
COND_R = 0.00145                    # a 12 AWG conductor in its insulation
CU_R = 0.00103                      # 12 AWG solid copper (2.05 mm)


def oval_sweep(points, w, t, seg=10):
    """A flat cable: an oval w wide and t thick swept along `points`, its
    flat sides kept facing up (toward +y) as far as the path allows; the
    ends capped."""
    pts = [Vector(p) for p in points]
    n = len(pts)
    v, f = [], []
    up = Vector((0, 1, 0))
    prev_side = None
    for i in range(n):
        tg = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
        side = tg.cross(up)
        side = side.normalized() if side.length > 1e-6 else prev_side
        prev_side = side
        nup = side.cross(tg).normalized()
        for j in range(seg):
            a = TAU * j / seg
            # a rounded rectangle more than an ellipse (the jacket is flat over the conductors)
            ca, sa = math.cos(a), math.sin(a)
            sx = math.copysign(abs(ca) ** 0.6, ca) * w / 2
            sy = math.copysign(abs(sa) ** 0.8, sa) * t / 2
            v.append(pts[i] + side * sx + nup * sy)
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    f.append(tuple(reversed(range(seg))))
    f.append(tuple((n - 1) * seg + j for j in range(seg)))
    return _fix(Shape(v, f))


# the cut end, the same for every length: the jacket ends at CUT, heading
# along CUT_DIR (its last few centimetres bent up off the floor), and the
# conductors run on from it to SPARK, where their bared ends nearly meet
# (the game's spark is there: weather_fx.rs CABLE_SPARK)
SPARK = Vector((0.0, 0.068, 0.0))
CUT_DIR = Vector((math.cos(math.radians(32)), math.sin(math.radians(32)), -0.10)).normalized()
CUT = Vector((-0.068, 0.030, 0.004))


def cable_path(variant):
    """The jacket's centre line, from its far end to CUT: a loose loop on
    the floor (the coil's memory), out of it along the floor, then its end
    bent up to the cut."""
    rise_from = CUT - CUT_DIR * 0.07
    rise_from.y = 0.0
    if variant == 0:
        # a loop on the +z side, left at its -z side
        r0, turns, sense, far = 0.125, 1.3, 1.0, Vector((0.16, 0, 0.30))
    else:
        # a wider loop the other way round, on the -z side, left at its +z side
        r0, turns, sense, far = 0.15, 0.8, -1.0, Vector((-0.42, 0, -0.05))
    leave = rise_from - Vector((0.06, 0, 0))
    K = leave + Vector((0, 0, sense * r0))
    a_end = -sense * math.pi / 2
    a0 = a_end - sense * TAU * turns
    def radius(a, t):
        # the loop not round: it wanders in and out, and closes to r0 where it is left
        wob = 0.16 * noise.noise(Vector((a * 0.7, 3.3 + variant, 0.0))) + 0.07 * math.sin(a * 2.3 + variant)
        return r0 * (1 + wob * (1 - smoothstep(0.75, 1.0, t)))
    pts = []
    first = K + Vector((math.cos(a0), 0, math.sin(a0))) * radius(a0, 0.0)
    # the far end's run in to the loop: a quadratic, tangent to the loop where it joins
    into = Vector((-math.sin(a0), 0, math.cos(a0))) * sense
    ctrl = first - into * 0.22
    for i in range(15):
        t = i / 14
        pts.append((1 - t) ** 2 * far + 2 * (1 - t) * t * ctrl + t * t * first)
    steps = int(turns * 34)
    for i in range(1, steps + 1):
        t = i / steps
        a = a0 + (a_end - a0) * t
        pts.append(K + Vector((math.cos(a), 0, math.sin(a))) * radius(a, t))
    # along the floor out of it, then bent up to the cut (a quadratic, tangent CUT_DIR there)
    pts.append(leave.lerp(rise_from, 0.5))
    pts.append(rise_from.copy())
    mid = CUT - CUT_DIR * 0.035
    mid.y = 0.004
    for i in range(1, 7):
        t = i / 6
        pts.append((1 - t) ** 2 * rise_from + 2 * (1 - t) * t * mid + t * t * CUT)
    # resting on the floor, or on itself where it crosses over
    out = []
    for p in pts:
        q = p.copy()
        q.y = max(q.y, CAB_T / 2)
        out.append(q)
    n = len(out)
    for i in range(n):
        p = out[i]
        if p.y > CAB_T * 0.6:
            continue
        for j in range(0, max(0, i - 6)):
            o = out[j]
            if math.hypot(p.x - o.x, p.z - o.z) < CAB_W * 1.1:
                p.y = max(p.y, CAB_T * 1.45)
    # the climbs over a crossing eased
    ys = [p.y for p in out]
    for _ in range(3):
        ys = [ys[0]] + [max(ys[i], (ys[i - 1] + ys[i + 1]) / 2) for i in range(1, n - 1)] + [ys[-1]]
    for p, y in zip(out, ys):
        p.y = y
    return out


def cable_sheath(variant):
    return oval_sweep(cable_path(variant), CAB_W, CAB_T)


def conductor_paths():
    """The two insulated conductors from inside the cut out to where their
    insulation was stripped, and their bared ends on to the spark: black
    (left of the end, as it rises) and white (right)."""
    side = CUT_DIR.cross(Vector((0, 1, 0))).normalized()
    up = side.cross(CUT_DIR).normalized()
    paths = {}
    for name, s, bend in (('black', -1, 0.9), ('white', 1, -0.6)):
        start = CUT - CUT_DIR * 0.004 + side * s * 0.0027
        # splayed apart, then (white) bent back toward the black's end
        p1 = CUT + CUT_DIR * 0.03 + side * s * 0.012 + up * 0.004 * bend
        p2 = CUT + CUT_DIR * 0.055 + side * s * 0.016 + up * 0.01 * bend
        tip = SPARK + side * s * 0.0035 + Vector((0, -0.002 * s, 0))
        strip = p2.lerp(tip, 0.25)
        ins = [start, CUT + CUT_DIR * 0.012 + side * s * 0.004, p1, p2, strip]
        bare = [strip, strip.lerp(tip, 0.5) + up * 0.002, tip]
        paths[name] = (ins, bare)
    return paths


def cable_core(name):
    ins, _ = conductor_paths()[name]
    return tube(ins, COND_R, seg=7, smooth=2)


def cable_copper():
    out = []
    for name in ('black', 'white'):
        _, bare = conductor_paths()[name]
        out.append(tube(bare, CU_R, seg=6, smooth=1))
    # the bare ground between them, shorter, bent down and away
    side = CUT_DIR.cross(Vector((0, 1, 0))).normalized()
    g = [CUT - CUT_DIR * 0.004, CUT + CUT_DIR * 0.02 + Vector((0, -0.002, 0)), CUT + CUT_DIR * 0.045 - side * 0.004 + Vector((0, -0.012, 0)),
         CUT + CUT_DIR * 0.06 - side * 0.002 + Vector((0, -0.024, 0))]
    out.append(tube(g, CU_R, seg=6, smooth=2))
    return sum(out)


def cable_paper():
    """The ground's paper wrap, torn off a little way out of the jacket: a
    twisted, crumpled strip."""
    side = CUT_DIR.cross(Vector((0, 1, 0))).normalized()
    pts = [CUT - CUT_DIR * 0.003, CUT + CUT_DIR * 0.008 + Vector((0, -0.001, 0)) + side * 0.001,
           CUT + CUT_DIR * 0.018 + Vector((0, -0.003, 0)) - side * 0.002]
    s = tube(pts, 0.0016, seg=5, smooth=1, r_end=0.0009)
    return noisy(s, 0.0004, scale=600.0, seed=3)


# =============================================================== the candle ===
CAND_R = 0.038          # a 3 in pillar


def candle_dims(variant):
    """(height of its pool, rim height over the pool, the rim's wall, the
    notch's angle, how much the rim slumps on the notch's side)"""
    if variant == 0:
        return (0.096, 0.010, 0.0045, 0.6, 0.85)
    return (0.058, 0.018, 0.0035, 2.4, 0.55)


def candle_body(variant):
    rng = random.Random(5200 + variant)
    pool_y, rim_h, wall, notch_a, slump = candle_dims(variant)
    R = CAND_R
    ri = R - wall
    top = pool_y + rim_h
    # the profile, bottom to the crater's floor (the pool hides it)
    prof = [(0.0, 0.0), (R - 0.0012, 0.0), (R, 0.0012), (R, top * 0.35), (R, top * 0.7), (R, top - 0.004), (R - 0.0006, top - 0.0009),
            (R - wall * 0.5, top), (ri + 0.0006, top - 0.0008), (ri, top - 0.004), (ri, pool_y + rim_h * 0.4), (ri - 0.002, pool_y + 0.0005),
            (0.0, pool_y + 0.0005)]
    seg = 20
    s = lathe(prof, seg=seg)
    # the rim, uneven: lower toward the notch, where the wax ran over; the
    # body not quite round, the wax a little lumpy
    def warp(p):
        a = math.atan2(p.z, p.x)
        r = math.hypot(p.x, p.z)
        da = math.atan2(math.sin(a - notch_a), math.cos(a - notch_a))
        notch = math.exp(-(da / 0.38) ** 2)
        lumpy = 1 + 0.03 * noise.noise(Vector((math.cos(a) * 2.0, p.y * 18.0, math.sin(a) * 2.0 + variant)))
        if p.y > pool_y + 0.0008:
            k = (p.y - pool_y) / rim_h if rim_h > 0 else 0
            k = max(0.0, min(1.2, k))
            h = rim_h * (1 - slump * notch) * (0.85 + 0.25 * noise.noise(Vector((math.cos(a) * 1.6, 7.7 + variant, math.sin(a) * 1.6))))
            y = pool_y + k * max(h, 0.0012)
            # the wall leans in a little at its top, and out where it slumped
            lean = (k ** 2) * (-0.0012 + 0.0025 * notch)
            if r > 1e-6:
                r2 = r + lean
                p = Vector((p.x / r * r2, y, p.z / r * r2))
            else:
                p = Vector((p.x, y, p.z))
        if r > 1e-6 and p.y < pool_y + rim_h * 2:
            r2 = math.hypot(p.x, p.z)
            p = Vector((p.x * lumpy, p.y, p.z * lumpy)) if r2 > ri * 0.8 else p
        return p
    s.warp(warp)
    parts = [s]
    # runs of wax down the side from the notch, each ending in its drop
    nruns = 3 if variant == 0 else 2
    for k in range(nruns):
        a = notch_a + rng.uniform(-0.35, 0.35)
        y0 = pool_y + rim_h * (1 - slump) * 0.6
        length = rng.uniform(0.35, 0.95) * y0 if k else y0 - 0.002
        pts = []
        steps = 8
        for i in range(steps + 1):
            t = i / steps
            y = y0 - length * t
            aa = a + 0.06 * math.sin(t * 5.0 + k) + rng.uniform(-0.01, 0.01)
            rr = R + 0.0006
            pts.append((math.cos(aa) * rr, y, math.sin(aa) * rr))
        run = tube(pts, 0.0034 if k == 0 else 0.0026, seg=6, smooth=0, r_end=0.0018)
        # pressed flat against the side (half in it)
        def flatten(p, a=a):
            r = math.hypot(p.x, p.z)
            if r > R:
                r2 = R + (r - R) * 0.55
                return Vector((p.x / r * r2, p.y, p.z / r * r2))
            return p
        run.warp(flatten)
        parts.append(run)
        # the drop it ended in (or, reaching the floor, the wax spread there)
        end = Vector(pts[-1])
        if end.y > 0.006:
            # flat against the side: thin across it (x, turned to face out), wide along it
            ea = math.atan2(end.z, end.x)
            drop = ellipsoid(0.0018, 0.0042, 0.0034, seg=8).rot(0, -ea, 0)
            drop.move(math.cos(ea) * (R + 0.0004), end.y - 0.0015, math.sin(ea) * (R + 0.0004))
            parts.append(drop)
    # the wax that ran to the floor, spread round its foot, more on the notch's side
    foot = []
    nf = 24
    for i in range(nf):
        a = TAU * i / nf
        da = math.atan2(math.sin(a - notch_a), math.cos(a - notch_a))
        r = R + 0.0025 + 0.010 * math.exp(-(da / 0.6) ** 2) * (1.0 if variant else 0.7) + 0.002 * noise.noise(Vector((math.cos(a) * 3, 1.1, math.sin(a) * 3)))
        foot.append((math.cos(a) * r, math.sin(a) * r))
    fs = prism(foot, 0.0016, bevel_r=0.0006, bevel_seg=1).rot(math.pi / 2, 0, 0).move(0, 0.0008, 0)
    parts.append(fs)
    return sum(parts)


def candle_pool(variant):
    """The pool's top: a thin disc filling the crater at its rim's foot."""
    pool_y, rim_h, wall, _, _ = candle_dims(variant)
    r = CAND_R - wall - 0.0016
    s = cyl(r, h=0.0012, seg=20, bevel_r=0.0004, bevel_seg=1).move(0, pool_y + 0.0004, 0)
    return s


def candle_wick():
    """The wick: a braided cotton wick 2 mm thick standing 9 mm out of the
    pool, curled over at its tip into the flame, a knob of carbon there;
    its foot at y = 0."""
    pts = [(0, -0.002, 0), (0, 0.003, 0), (0.0003, 0.0065, 0), (0.0016, 0.0092, 0), (0.0036, 0.0104, 0)]
    w = tube(pts, 0.00095, seg=6, smooth=1, r_end=0.0007)
    knob = ellipsoid(0.0013, 0.0011, 0.0012, seg=8).move(0.0040, 0.0104, 0)
    return noisy(w, 0.00012, scale=900.0, seed=9) + knob


# ------------------------------------------------------------ registered ---
auto('Real__derelictTilePieceA', lambda: tile_piece(0), 700)
auto('Real__derelictTilePieceB', lambda: tile_piece(1), 700)
auto('Real__derelictTilePieceC', lambda: tile_piece(2), 700)
auto('Real__derelictTilePieceD', lambda: tile_piece(3), 700)
auto('Real__derelictTileFaceA', lambda: tile_face(0), 400)
auto('Real__derelictTileFaceB', lambda: tile_face(1), 400)
auto('Real__derelictTileFaceC', lambda: tile_face(2), 400)
auto('Real__derelictTileFaceD', lambda: tile_face(3), 400)
auto('Real__derelictTeeSteelA', lambda: tee_lying(False, *TEE_A), 1200)
auto('Real__derelictTeeSteelB', lambda: tee_lying(False, *TEE_B), 1200)
auto('Real__derelictTeeCapA', lambda: tee_lying(True, *TEE_A), 260)
auto('Real__derelictTeeCapB', lambda: tee_lying(True, *TEE_B), 260)
auto('Real__electricCableSheathA', smooth_at(70, lambda: cable_sheath(0)), 1400)
auto('Real__electricCableSheathB', smooth_at(70, lambda: cable_sheath(1)), 1400)
auto('Real__electricCableCoreBlack', smooth_at(70, lambda: cable_core('black')), 400)
auto('Real__electricCableCoreWhite', smooth_at(70, lambda: cable_core('white')), 400)
auto('Real__electricCableCopper', smooth_at(70, cable_copper), 400)
auto('Real__electricCablePaper', smooth_at(70, cable_paper), 120)
auto('Real__hauntedCandleBodyA', smooth_at(50, lambda: candle_body(0)), 1200)
auto('Real__hauntedCandleBodyB', smooth_at(50, lambda: candle_body(1)), 1200)
auto('Real__hauntedCandlePoolA', lambda: candle_pool(0), 200)
auto('Real__hauntedCandlePoolB', lambda: candle_pool(1), 200)
auto('Real__hauntedCandleWick', smooth_at(60, candle_wick), 240)
