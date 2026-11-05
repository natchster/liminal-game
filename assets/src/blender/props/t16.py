"""t16: the props remodelled at their real sizes (docs/PROPS.md, P3, the
rare): a play area's toys, years on a floor (docs/PROPS.md 1, "toys stay
toys"): a cotton skipping rope with beech handles, a PVC licorice rope
with moulded handles, a deflated size 3 football and a deflated size 3
basketball, a steel toddler tricycle on its side, a plastic cruiser
skateboard, a paper diamond kite torn on its spars, and an eight note toy
metallophone. Sources and sizes: t16.notes.md. (The crayon drawing on a
wall is painted in the game: `placers_p21/src/placers/t16.rs`.)

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
prop's own frame: x across, y up from the floor, z out of its front; a
thing that lies on the floor is built lying, its lowest point on y = 0
(the game turns it only about y).

jumpRope{Cord,Handles}       the cotton rope (8 mm, 2.3 m) wound loosely
                             and dropped, its two turned beech handles
licoriceRope{Cord,Handles}   the PVC rope (4.8 mm, 2.4 m) dropped in a
                             springy loop, its two moulded handles side
                             by side
deflatedFootball{White,Black}  a size 3 football (32 stitched panels, 19
                             cm) gone soft: sagging on the floor, its top
                             caved in, creased at the fold
deflatedBasketball{Skin,Lines}  a size 3 rubber basketball (17.8 cm), its
                             8 panels' black channels, gone soft the same
trike{Frame,Plated,Rubber,Grips,Seat}  a steel tricycle (10 in front
                             wheel, 7 in rears, spoked) lying on its side,
                             its steering swung
cruiser{Deck,Trucks,Bushings,Wheels}{,Flip}  a 22 x 6 in moulded
                             plastic cruiser board, 3 in trucks, 59 mm
                             wheels, on its wheels or upside down
kite{Sail,Spars,String,Bow}  a paper diamond kite (600 x 500 mm) on pine
                             spars, torn, its tail and line lying slack
xylo{Frame,Bars,Pins,Felt,Mallet,MalletHead}  an eight note toy
                             metallophone: enamelled aluminium bars
                             pinned over felt on a pine frame, a mallet
"""
import math
import os
import random
import zlib

import bpy
import bmesh
from propkit import *
import propkit

ONLY = set(filter(None, os.environ.get('T16_ONLY', '').split(',')))


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    if ONLY and name not in ONLY:
        return
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, smooth=smooth, flat=flat, budget=budget)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def n1(x, y=0.0, z=0.0):
    """Scalar Perlin noise (repeats in every Blender process)."""
    return noise.noise(Vector((x, y, z)))


def sweep(pts, radii, seg=6, cap0=True, cap1=True, up=None):
    """A tube along `pts` whose radius at each point is `radii[i]`
    (parallel-transport frame); cap0/cap1 close its ends (t12's)."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    tang = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tang.append(t.normalized() if t.length > 1e-12 else Vector((1, 0, 0)))
    upv = Vector(up) if up else (Vector((0, 1, 0)) if abs(tang[0].y) < 0.9 else Vector((1, 0, 0)))
    nrm = tang[0].cross(upv).normalized()
    v, f = [], []
    for i in range(n):
        if i:
            q = tang[i - 1].rotation_difference(tang[i])
            nrm = (q @ nrm).normalized()
        b = tang[i].cross(nrm).normalized()
        for j in range(seg):
            a = TAU * j / seg
            v.append(pts[i] + (nrm * math.cos(a) + b * math.sin(a)) * radii[i])
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    s = Shape(v, f)
    for end, cap in ((0, cap0), (n - 1, cap1)):
        if not cap:
            continue
        ring = [end * seg + j for j in range(seg)]
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


def path_len(pts):
    return sum((b - a).length for a, b in zip(pts, pts[1:]))


def resample(pts, step):
    """Points every `step` along the polyline (its ends kept)."""
    pts = [Vector(p) for p in pts]
    total = path_len(pts)
    n = max(2, int(round(total / step)) + 1)
    out, seg, acc = [pts[0].copy()], 0, 0.0
    for k in range(1, n - 1):
        s = total * k / (n - 1)
        while seg < len(pts) - 2 and acc + (pts[seg + 1] - pts[seg]).length < s:
            acc += (pts[seg + 1] - pts[seg]).length
            seg += 1
        a, b = pts[seg], pts[seg + 1]
        L = (b - a).length
        out.append(a.lerp(b, (s - acc) / L if L > 1e-12 else 0.0))
    out.append(pts[-1].copy())
    return out


def along(shape, d, origin):
    """A shape built along +y turned to lie along direction `d` and moved
    to `origin`."""
    q = Vector((0, 1, 0)).rotation_difference(Vector(d).normalized())
    shape.apply(q.to_matrix().to_4x4())
    return shape.move(*origin)


def lay_cord(pts2, r, rise0=0.0, rise1=0.0, end_len=0.04, step=0.013, relax=4):
    """A cord lying slack along the floor through the plan points `pts2`
    ((x, z)), smoothed: where it crosses itself the later pass rides over
    the earlier (it lifts over a run either side), and it rises from the
    floor to `rise0`/`rise1` (its axis's height) over `end_len` at its ends.
    Returns the points of its axis."""
    p = resample(chaikin([Vector((x, 0.0, z)) for x, z in pts2], 3), step)
    for _ in range(relax):          # a hand's kinks pulled out
        p = [p[0]] + [p[i] * 0.5 + (p[i - 1] + p[i + 1]) * 0.25 for i in range(1, len(p) - 1)] + [p[-1]]
    n = len(p)
    s = [0.0]
    for a, b in zip(p, p[1:]):
        s.append(s[-1] + (b - a).length)
    d = 2 * r
    h = [0.0] * n
    for i in range(n):
        for j in range(i):
            if s[i] - s[j] < 4 * d:
                continue
            q = (Vector((p[i].x, 0, p[i].z)) - Vector((p[j].x, 0, p[j].z))).length
            if q < d * 1.15:
                h[i] = max(h[i], h[j] + d * (1.0 - smoothstep(0.55, 1.15, q / d)) * 0.92)
    # the lift spreads along the cord (it cannot step)
    span = max(2, int(round(3.2 * d / step)))
    for _ in range(3):
        g = h[:]
        for i in range(n):
            for k in range(1, span + 1):
                w = 1.0 - k / (span + 1)
                for jj in (i - k, i + k):
                    if 0 <= jj < n:
                        g[i] = max(g[i], h[jj] * (w * w * (3 - 2 * w)))
        h = g
    out = []
    for i in range(n):
        y = r + h[i]
        if s[i] < end_len:
            y = max(y, r + (rise0 - r) * (1 - smoothstep(0.0, end_len, s[i])))
        if s[-1] - s[i] < end_len:
            y = max(y, r + (rise1 - r) * (1 - smoothstep(0.0, end_len, s[-1] - s[i])))
        out.append(Vector((p[i].x, y, p[i].z)))
    return out


# ======================================================== the jump ropes ===
# A child's skipping rope of braided cotton with turned beech handles (a
# Glueckskaefer or a "Denise": 2.5-3.1 m of rope, handles 120 x 25 mm),
# wound loosely round a hand and dropped: three slack loops spreading apart,
# the two handles out of the coil, near each other.
JR_R = 0.004            # 8 mm rope
JR_HL, JR_HR = 0.120, 0.0125


def hermite(p0, t0, p1, t1, n):
    """n points of the cubic from p0 (leaving along t0) to p1 (arriving
    along t1), ends excluded; plan points (x, z)."""
    out = []
    for k in range(1, n + 1):
        u = k / (n + 1)
        h00, h10, h01, h11 = 2 * u ** 3 - 3 * u ** 2 + 1, u ** 3 - 2 * u ** 2 + u, -2 * u ** 3 + 3 * u ** 2, u ** 3 - u ** 2
        out.append(tuple(h00 * p0[i] + h10 * t0[i] + h01 * p1[i] + h11 * t1[i] for i in range(2)))
    return out


# the handles' rope ends (x, z) and the way each handle points from there
JR_HA, JR_DA = (-0.035, 0.205), (-0.45, 0.89)
JR_HB, JR_DB = (0.030, 0.215), (-0.12, 0.99)


def jr_plan():
    """The rope's plan (x, z): out of handle A, round three loose turns
    whose middles wander as they slackened, so each crosses the others, and
    back to handle B beside A."""
    loops = []
    turns, n = 2.6, 46
    for k in range(n + 1):
        t = k / n
        a = -math.pi * 0.35 + TAU * turns * t
        r = 0.118 + 0.018 * math.sin(3.7 * t + 0.4) + 0.016 * n1(t * 4.1, 1.7)
        sq = 0.84 + 0.12 * math.sin(2.9 * t + 1.2)
        cx = 0.075 * math.cos(TAU * 0.62 * t + 0.4) - 0.01
        cz = -0.035 + 0.060 * math.sin(TAU * 0.48 * t + 0.2)
        loops.append((cx + r * math.cos(a), cz + r * math.sin(a) * sq))
    ta = (loops[1][0] - loops[0][0], loops[1][1] - loops[0][1])
    tb = (loops[-1][0] - loops[-2][0], loops[-1][1] - loops[-2][1])
    k = 0.30 / math.hypot(*ta)
    a0 = (JR_HA[0] - JR_DA[0] * 0.045, JR_HA[1] - JR_DA[1] * 0.045)
    b0 = (JR_HB[0] - JR_DB[0] * 0.045, JR_HB[1] - JR_DB[1] * 0.045)
    pts = [JR_HA, a0]
    pts += hermite(a0, (-JR_DA[0] * 0.25, -JR_DA[1] * 0.25), loops[0], (ta[0] * k, ta[1] * k), 4)
    pts += loops
    k = 0.30 / math.hypot(*tb)
    pts += hermite(loops[-1], (tb[0] * k, tb[1] * k), b0, (JR_DB[0] * 0.25, JR_DB[1] * 0.25), 4)
    pts += [b0, JR_HB]
    return pts


def jr_axis():
    return lay_cord(jr_plan(), JR_R, rise0=JR_HR, rise1=JR_HR, end_len=0.05)


def beech_handle():
    """A turned beech handle along +y from its rope end (bored) to its
    domed end: a neck, the swell of the grip, a bead before the end."""
    prof = [(0.0045, 0.012), (0.0045, 0.0), (0.0085, 0.0), (0.0092, 0.002), (0.0096, 0.008),
            (0.0110, 0.016), (0.0118, 0.026), (0.0124, 0.045), (0.0125, 0.065), (0.0121, 0.088),
            (0.0112, 0.101), (0.0106, 0.104), (0.0114, 0.108), (0.0112, 0.113), (0.0098, 0.117),
            (0.0060, 0.1195), (0.0, 0.120)]
    return lathe(prof, seg=14, cap_bottom=True, cap_top=False)


def jump_rope_cord():
    return sweep(jr_axis(), [JR_R] * len(jr_axis()), seg=6)


def jump_rope_handles():
    out = Shape()
    for (x, z), (dx, dz) in ((JR_HA, JR_DA), (JR_HB, JR_DB)):
        d = Vector((dx, 0, dz)).normalized()
        # the rope's end runs 18 mm up the bore
        out = out + along(beech_handle(), d, Vector((x, JR_HR, z)) - d * 0.018)
    return out


# A PVC "licorice" rope (3/16 in, 8 ft: ExerRopes, Champion), its moulded
# polypropylene handles (4 1/2 in) side by side where it was dropped, the
# rope in one springy loop that crossed itself.
LR_R = 0.0024
LR_HR = 0.0115


# the handles lie side by side, pointing away from the loop (-x), 26 mm
# apart, their capped ends at x = 0; the rope leaves each cap and swings out
LR_HZ = (0.013, -0.013)


def lr_plan():
    c = [(0.000, LR_HZ[0]), (0.03, 0.016), (0.08, 0.030), (0.20, 0.085), (0.34, 0.150), (0.48, 0.150),
         (0.58, 0.090), (0.60, -0.010), (0.53, -0.105), (0.40, -0.140), (0.29, -0.090),
         (0.26, 0.010), (0.31, 0.110), (0.43, 0.215), (0.57, 0.250), (0.69, 0.200),
         (0.75, 0.080), (0.73, -0.070), (0.62, -0.190), (0.44, -0.255), (0.25, -0.220),
         (0.11, -0.100), (0.05, -0.030), (0.02, -0.016), (0.000, LR_HZ[1])]
    return c


def lr_axis():
    return lay_cord(lr_plan(), LR_R, rise0=LR_HR, rise1=LR_HR, end_len=0.045, step=0.014)


def pvc_handle():
    """A moulded polypropylene handle along +y from its capped rope end:
    the cap with the rope's hole, a collar, the ribbed grip, a flared foot,
    open (hollow) at its far end."""
    prof = [(0.0026, 0.0), (0.0068, 0.0), (0.0078, 0.0015), (0.0080, 0.010), (0.0098, 0.012),
            (0.0102, 0.018), (0.0094, 0.021), (0.0096, 0.026)]
    y = 0.026
    for k in range(7):           # the grip's rings
        prof += [(0.0101, y + 0.004), (0.0108, y + 0.0075), (0.0100, y + 0.0115)]
        y += 0.0115
    prof += [(0.0104, 0.1115), (0.0118, 0.1130), (0.0116, 0.1140), (0.0090, 0.1140),
             (0.0084, 0.1090), (0.0026, 0.105)]
    return lathe(prof, seg=10, cap_bottom=False, cap_top=False)


def licorice_cord():
    ax = lr_axis()
    return sweep(ax, [LR_R] * len(ax), seg=6)


def licorice_handles():
    out = Shape()
    for z in LR_HZ:
        out = out + along(pvc_handle(), (-1, 0, 0), (0.004, LR_HR, z))
    return out


auto('Real__jumpRopeCord', jump_rope_cord, 2500)
auto('Real__jumpRopeHandles', jump_rope_handles, 900)
auto('Real__licoriceRopeCord', licorice_cord, 2600)
auto('Real__licoriceRopeHandles', licorice_handles, 1400)


# ===================================================== the deflated balls ===
# A size 3 football (circumference 23-24 in: 19 cm across; 32 machine
# stitched PVC panels, 12 black pentagons and 20 white hexagons, each
# puffed between its seams) and a size 3 rubber basketball (22 in: 17.8
# cm; its eight panels parted by black channels: two great circles and
# two side curves), each gone soft over years on a floor: it sags, its
# bottom spread flat where it rests, its top caved in to a dish (the skin
# folded over on itself) with creases radiating from the fold.
FB_R = 0.095
BB_R = 0.089


def icosa():
    ph = (1 + 5 ** 0.5) / 2
    v = [(-1, ph, 0), (1, ph, 0), (-1, -ph, 0), (1, -ph, 0), (0, -1, ph), (0, 1, ph), (0, -1, -ph),
         (0, 1, -ph), (ph, 0, -1), (ph, 0, 1), (-ph, 0, -1), (-ph, 0, 1)]
    v = [Vector(p).normalized() for p in v]
    f = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2),
         (10, 7, 6), (7, 1, 8), (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5),
         (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1)]
    return v, f


def football_panels():
    """The truncated icosahedron's faces as corner lists on the unit
    sphere's chords: (pentagons, hexagons)."""
    v, f = icosa()
    pents, hexes = [], []
    for i, c in enumerate(v):
        nb = sorted({j for t in f if i in t for j in t if j != i})
        e1 = (v[nb[0]] - c * c.dot(v[nb[0]])).normalized()
        e2 = c.cross(e1)
        pts = [c + (v[j] - c) / 3 for j in nb]
        pts.sort(key=lambda p: math.atan2((p - c).dot(e2), (p - c).dot(e1)))
        pents.append(pts)
    for a, b, c in f:
        A, B, C = v[a], v[b], v[c]
        hexes.append([A + (B - A) / 3, A + (B - A) * 2 / 3, B + (C - B) / 3, B + (C - B) * 2 / 3,
                      C + (A - C) / 3, C + (A - C) * 2 / 3])
    return pents, hexes


def panel_mesh(corners, R, n=4, puff=0.010, seam=0.012):
    """One stitched panel on a ball of radius R: its corners fanned from its
    middle, each fan triangle split n ways, put on the sphere, puffed
    between its seams and sunk at them."""
    c = sum(corners, Vector()) / len(corners)
    v, f, key = [], [], {}
    k = len(corners)

    def vid(m, a, b):
        # (a, b) steps toward corners m and m + 1; shared edges keyed once
        if a == 0 and b == 0:
            kk = ('c',)
        elif b == 0:
            kk = ('e', m % k, a)
        elif a == 0:
            kk = ('e', (m + 1) % k, b)
        else:
            kk = ('i', m, a, b)
        if kk in key:
            return key[kk]
        p = c + (corners[m % k] - c) * (a / n) + (corners[(m + 1) % k] - c) * (b / n)
        t = (a + b) / n
        r = R * (1 + puff * (1 - t ** 2.2) - seam * smoothstep(0.72, 1.0, t))
        v.append(p.normalized() * r)
        key[kk] = len(v) - 1
        return key[kk]
    for m in range(k):
        for a in range(n):
            for b in range(n - a):
                f.append((vid(m, a, b), vid(m, a + 1, b), vid(m, a, b + 1)))
                if a + b < n - 1:
                    f.append((vid(m, a + 1, b), vid(m, a + 1, b + 1), vid(m, a, b + 1)))
    return propkit._fix_winding(Shape(v, f))


def deflate(s, R, hc=0.36, eps=0.075, crease=0.05, folds=5, seed=0.0):
    """The ball gone soft: its top (above hc R along a dent axis tipped a
    little) folded in on itself to a dish, the fold rounded over eps R and
    creased `folds` ways; the whole sagged, its bottom spread on the floor.
    Its parts are put on the floor together (`on_floor`)."""
    ax = Vector((0.13, 1.0, 0.07)).normalized()
    e1 = ax.cross(Vector((0, 0, 1))).normalized()
    e2 = ax.cross(e1)
    out = s.copy()
    for i, p in enumerate(out.v):
        y = p.dot(ax)
        rad = p - ax * y
        rr = rad.length
        th = math.atan2(rad.dot(e2), rad.dot(e1)) if rr > 1e-9 else 0.0
        u = y - hc * R
        y2 = hc * R - math.sqrt(u * u + (eps * R) ** 2) + eps * R
        # creases from the fold: the skin buckles in a few lobes
        w = math.exp(-(u / (0.30 * R)) ** 2)
        lob = math.sin(folds * th + 0.8 * math.sin(2 * th + seed) + seed) + 0.35 * n1(th * 2.0, seed, 3.3)
        rr *= 1 + crease * w * lob
        y2 += crease * R * 0.55 * w * lob * (1 if u > 0 else 0.6)
        # it sags: lower and wider, the bottom spread flat on the floor
        y2 *= 0.93
        yb, soft = -0.80 * R, 0.06 * R
        t = yb + soft - y2
        if t > 0:
            held = soft * (1 - math.exp(-t / soft))
            y2 = yb + soft - held
            rr += (t - held) * 0.55
        rr *= 1 + 0.045 * math.exp(-((y2 + 0.38 * R) / (0.38 * R)) ** 2)
        # unevenness of a soft skin
        q = p * (6.0 / R)
        rr += 0.0012 * n1(q.x + seed, q.y, q.z)
        y2 += 0.0008 * n1(q.x, q.y + seed, q.z + 2.0)
        dirn = (e1 * math.cos(th) + e2 * math.sin(th))
        out.v[i] = ax * y2 + dirn * rr
    return out


def on_floor(*shapes):
    """Move shapes that make one thing together so their lowest point is
    on the floor."""
    lo = min(sh.bounds()[0].y for sh in shapes)
    return [sh.move(0, -lo, 0) for sh in shapes]


_balls = {}


def football(black):
    if 'fb' not in _balls:
        pents, hexes = football_panels()
        w, b = Shape(), Shape()
        for c in hexes:
            w = w + panel_mesh(c, FB_R)
        for c in pents:
            b = b + panel_mesh(c, FB_R)
        _balls['fb'] = on_floor(deflate(w, FB_R, seed=1.3), deflate(b, FB_R, seed=1.3))
    return _balls['fb'][1 if black else 0].copy()


BB_DENT = dict(hc=0.50, eps=0.085, crease=0.06, folds=4, seed=2.1)


def basketball(lines):
    if 'bb' not in _balls:
        _balls['bb'] = on_floor(deflate(sphere(BB_R, seg=44, rings=30), BB_R, **BB_DENT), basketball_lines_raw())
    return _balls['bb'][1 if lines else 0].copy()


def basketball_lines_raw():
    """The channels: thin strips 6 mm across along the two great circles and
    the two side curves, laid on the gone-soft skin (each point found on
    the skin as it sagged, then lifted 0.25 mm off it along the skin's
    outside, which faces up in the dish)."""
    m = 120
    curves = [[Vector((0, math.cos(TAU * k / m), math.sin(TAU * k / m))) for k in range(m)],
              [Vector((math.cos(TAU * k / m), 0, math.sin(TAU * k / m))) for k in range(m)]]
    c = 0.52
    rc = math.sqrt(1 - c * c)
    for sx in (-1, 1):
        curves.append([Vector((sx * c, rc * math.cos(TAU * k / m), rc * math.sin(TAU * k / m))) for k in range(m)])
    hw = 0.0030 / BB_R
    pts = []
    for cv in curves:
        for k in range(m):
            p, q = cv[k], cv[(k + 1) % m]
            t = (q - p).normalized()
            side = p.cross(t).normalized()
            for sd in (1, -1):
                d = (p + side * hw).normalized() if sd > 0 else (p - side * hw).normalized()
                t1 = d.cross(Vector((0.31, 0.83, 0.47))).normalized()
                t2 = d.cross(t1)
                pts += [d * BB_R, (d + t1 * 0.004).normalized() * BB_R, (d + t2 * 0.004).normalized() * BB_R]
    dd = deflate(Shape(pts, []), BB_R, **BB_DENT).v
    s = Shape()
    i = 0
    for cv in curves:
        v, f, ns = [], [], []
        for k in range(m):
            for sd in (1, -1):
                a, b, c2 = dd[i], dd[i + 1], dd[i + 2]
                i += 3
                n = (b - a).cross(c2 - a)
                n = n.normalized() if n.length > 1e-14 else Vector((0, 1, 0))
                v.append(a + n * 0.00025)
                ns.append(n)
        for k in range(m):
            k2 = (k + 1) % m
            q = (2 * k, 2 * k + 1, 2 * k2 + 1, 2 * k2)
            fn = (v[q[1]] - v[q[0]]).cross(v[q[3]] - v[q[0]])
            f.append(q if fn.dot(ns[2 * k]) >= 0 else tuple(reversed(q)))
        s = s + Shape(v, f)
    return s


auto('Real__deflatedFootballWhite', lambda: football(False), 2200)
auto('Real__deflatedFootballBlack', lambda: football(True), 1100)
auto('Real__deflatedBasketballSkin', lambda: basketball(False), 2800)
auto('Real__deflatedBasketballLines', lambda: basketball(True), 1000)


# ========================================================== the tricycle ===
# A toddler's steel tricycle of the classic kind (Radio Flyer's 10 in
# Classic, 34B: 26 x 22 x 21 in; a 10 x 1 1/4 in front wheel, 7 x 1 1/2 in
# rears; seat 13 1/2 in from the floor; an all-steel frame, spoked wheels,
# solid rubber tyres, a rear step), built upright with its steering swung
# as it fell, then laid on its left side as it comes to rest (the turn
# that rests it lowest, found below), its lowest point on the floor.
TF_R, TR_R = 0.127, 0.089           # front and rear wheels' outer radii
TF_Z, TR_Z = 0.245, -0.165          # their axles (z)
TR_X = 0.178                        # the rear wheels' middles (x)
T_AXIS = Vector((0.0, 0.951, -0.309))   # the steering axis, 72 deg, through the front axle
T_STEER = 0.52                      # the steering swung as it fell (rad)


def rot_about(s, pivot, axis, ang):
    m = Matrix.Translation(Vector(pivot)) @ Matrix.Rotation(ang, 4, Vector(axis)) @ Matrix.Translation(-Vector(pivot))
    return s.apply(m)


def x_axis(s):
    """A shape built about +y (a lathe) turned to lie about +x."""
    return s.rot(0, 0, -math.pi / 2)


def wheel(c, r_out, sect, n_spokes, seg, hub_len):
    """A spoked wheel about +x at c: the solid rubber tyre (its section
    `sect` across), the rolled steel rim, the spokes laced from the hub's
    two flanges, the hub. Returns (rubber, plated)."""
    c = Vector(c)
    tr = sect / 2
    tyre = torus(r_out - tr, tr, seg=seg, tseg=8).scale(1, 1, 0.92).rot(0, math.pi / 2, 0)
    # the rim: a channel the tyre sits in
    r0 = r_out - sect + 0.002
    rim = x_axis(lathe([(r0 - 0.006, -sect * 0.42), (r0 + 0.002, -sect * 0.45), (r0 + 0.003, -sect * 0.30),
                        (r0 + 0.001, 0.0), (r0 + 0.003, sect * 0.30), (r0 + 0.002, sect * 0.45),
                        (r0 - 0.006, sect * 0.42), (r0 - 0.007, 0.0)], seg=seg, cap_bottom=False, cap_top=False))
    hub = x_axis(lathe([(0.0045, -hub_len / 2 - 0.012), (0.0045, -hub_len / 2), (0.017, -hub_len / 2),
                        (0.017, -hub_len / 2 + 0.004), (0.011, -hub_len / 2 + 0.006), (0.012, 0.0),
                        (0.011, hub_len / 2 - 0.006), (0.017, hub_len / 2 - 0.004), (0.017, hub_len / 2),
                        (0.0045, hub_len / 2), (0.0045, hub_len / 2 + 0.012)], seg=10))
    nuts = Shape()
    for sx in (-1, 1):
        nuts = nuts + x_axis(cyl(0.0075, h=0.007, seg=6, bevel_r=0.0008, bevel_seg=1)).move(sx * (hub_len / 2 + 0.008), 0, 0)
    spokes = Shape()
    for k in range(n_spokes):
        a = TAU * (k + 0.5 * (k % 2)) / n_spokes
        side = -1 if k % 2 else 1
        h = Vector((side * (hub_len / 2 - 0.004), 0.015 * math.cos(a), 0.015 * math.sin(a)))
        rr = r0 - 0.004
        e = Vector((0.0, rr * math.cos(a + 0.12 * side), rr * math.sin(a + 0.12 * side)))
        spokes = spokes + sweep([h, e], [0.0013, 0.0013], seg=4)
    plated = rim + hub + nuts + spokes
    return tyre.move(*c), plated.move(*c)


def trike_upright():
    """The tricycle standing, its steering swung: {material: shape}."""
    fr, fp = wheel((0, TF_R, TF_Z), TF_R, 0.032, 14, 28, 0.070)
    steer_rub, steer_pl, steer_paint = fr, fp, Shape()
    # the crank axle out of the hub each side, its arms opposite, the pedals
    for sx, up in ((-1, 1), (1, -1)):
        x0 = sx * 0.047
        arm = [Vector((x0, TF_R, TF_Z)), Vector((x0 + sx * 0.010, TF_R, TF_Z)),
               Vector((x0 + sx * 0.014, TF_R + up * 0.072, TF_Z + 0.012)),
               Vector((x0 + sx * 0.022, TF_R + up * 0.075, TF_Z + 0.012))]
        arm = chaikin(arm, 1)
        steer_pl = steer_pl + sweep(arm, [0.0052] * len(arm), seg=6)
        sp = Vector((x0 + sx * 0.022, TF_R + up * 0.075, TF_Z + 0.012))
        steer_pl = steer_pl + x_axis(cyl(0.004, h=0.080, seg=6)).move(sp.x + sx * 0.040, sp.y, sp.z)
        ped = rbox(0.068, 0.022, 0.034, r=0.004, seg=1)
        for k in range(3):        # the tread's grooves
            ped = cut(ped, box(0.070, 0.004, 0.003).move(0, 0.011, -0.010 + k * 0.010))
        steer_rub = steer_rub + ped.move(sp.x + sx * 0.046, sp.y, sp.z)
    # the fork: two blades from the axle's ends up the steering axis to the crown
    axle = Vector((0, TF_R, TF_Z))
    crown = axle + T_AXIS * 0.170
    for sx in (-1, 1):
        a = axle + Vector((sx * 0.040, 0, 0))
        b = crown + Vector((sx * 0.036, 0, 0)) - T_AXIS * 0.012
        steer_paint = steer_paint + sweep([a, a.lerp(b, 0.5) + Vector((sx * 0.003, 0, 0)), b], [0.0085] * 3, seg=8)
        steer_paint = steer_paint + rbox(0.010, 0.030, 0.018, r=0.003).move(a.x, a.y, a.z)   # the dropout
    cr = rbox(0.090, 0.020, 0.030, r=0.006)
    q = Vector((0, 1, 0)).rotation_difference(T_AXIS)
    cr.apply(q.to_matrix().to_4x4())
    steer_paint = steer_paint + cr.move(*crown)
    # the steerer up through the head tube to the stem, the handlebar
    top = axle + T_AXIS * 0.395
    steer_pl = steer_pl + sweep([crown, axle + T_AXIS * 0.30, top], [0.0095] * 3, seg=10)
    stem = top + Vector((0, 0.004, -0.010))
    for sx in (-1, 1):
        bar = [stem, stem + Vector((sx * 0.06, 0.010, -0.004)), stem + Vector((sx * 0.13, 0.024, -0.040)),
               stem + Vector((sx * 0.175, 0.026, -0.085)), stem + Vector((sx * 0.205, 0.022, -0.120))]
        bar = chaikin(bar, 2)
        steer_pl = steer_pl + sweep(bar, [0.0095] * len(bar), seg=10)
    steer_pl = steer_pl + cyl(0.016, h=0.030, seg=12, bevel_r=0.002).move(stem.x, stem.y - 0.012, stem.z)  # the clamp
    grips = Shape()
    for sx in (-1, 1):
        e = stem + Vector((sx * 0.205, 0.022, -0.120))
        d = (e - (stem + Vector((sx * 0.175, 0.026, -0.085)))).normalized()
        g = lathe([(0.0, -0.002), (0.012, -0.002), (0.0125, 0.010), (0.0118, 0.020), (0.0126, 0.030),
                   (0.0118, 0.040), (0.0126, 0.050), (0.0118, 0.060), (0.0126, 0.070), (0.0130, 0.082),
                   (0.0165, 0.086), (0.0165, 0.094), (0.0120, 0.098), (0.0, 0.099)], seg=12)
        grips = grips + along(g, d, e - d * 0.062)
    # steer: swing everything that turns about the steering axis
    for s in (steer_rub, steer_pl, steer_paint, grips):
        rot_about(s, axle, T_AXIS, T_STEER)
    paint = steer_paint
    # the head tube
    ht = cyl(0.0175, h=0.090, seg=14, bevel_r=0.002)
    ht.apply(q.to_matrix().to_4x4())
    paint = paint + ht.move(*(axle + T_AXIS * 0.245))
    # the frame: one bent tube from the head tube down and back to the rear bridge
    h0 = axle + T_AXIS * 0.235
    frame = [h0, h0 + Vector((0, -0.020, -0.060)), Vector((0, 0.250, 0.040)), Vector((0, 0.215, -0.040)),
             Vector((0, 0.185, -0.110)), Vector((0, 0.160, -0.150))]
    frame = chaikin(frame, 2)
    paint = paint + sweep(frame, [0.0125] * len(frame), seg=12)
    # the rear bridge: the axle housing tube across, its two struts, the step
    paint = paint + x_axis(cyl(0.0125, h=0.300, seg=12, bevel_r=0.002)).move(0, TR_R + 0.026, TR_Z)
    for sx in (-1, 1):
        paint = paint + sweep([Vector((sx * 0.145, TR_R + 0.026, TR_Z)), Vector((sx * 0.150, TR_R, TR_Z))],
                              [0.010, 0.010], seg=8)
        paint = paint + sweep([Vector((0, 0.160, -0.150)), Vector((sx * 0.110, TR_R + 0.032, TR_Z + 0.010))],
                              [0.0095, 0.0095], seg=8)
    step = rbox(0.270, 0.004, 0.072, r=0.0018, seg=1)
    for sx in (-1, 1):
        step = step + rbox(0.004, 0.011, 0.072, r=0.0015, seg=1).move(sx * 0.133, -0.0045, 0)
    step = step + rbox(0.270, 0.011, 0.004, r=0.0015, seg=1).move(0, -0.0045, -0.034)
    for k in range(4):
        step = step + rbox(0.220, 0.0025, 0.006, r=0.001, seg=1).move(0, 0.003, -0.024 + k * 0.016)
    paint = paint + step.move(0, TR_R + 0.040, TR_Z - 0.050)
    # the seat post on the frame, the saddle on it
    sp0 = Vector((0, 0.236, -0.030))
    paint = paint + sweep([sp0, sp0 + Vector((0, 0.060, -0.004))], [0.0110, 0.0110], seg=10)
    # the saddle: moulded, wide at the back with a raised rim, its nose
    # narrowing forward, domed, on its post
    out = [(0.0, 0.084), (0.034, 0.078), (0.056, 0.050), (0.070, 0.000), (0.072, -0.040), (0.060, -0.072),
           (0.030, -0.086), (0.0, -0.088), (-0.030, -0.086), (-0.060, -0.072), (-0.072, -0.040),
           (-0.070, 0.000), (-0.056, 0.050), (-0.034, 0.078)]
    seat = prism(round_outline(out, 0.018, seg=3), 0.034, bevel_r=0.010, bevel_seg=3)
    seat.rot(-math.pi / 2, 0, 0)

    def dome(p):
        if p.y > 0:
            p.y += 0.008 * max(0.0, 1 - (p.x / 0.075) ** 2) + 0.012 * smoothstep(-0.03, -0.085, p.z)
        return p
    seat.warp(dome)
    seat = seat.move(0, 0.318, -0.048)
    plated = steer_pl
    for sx in (-1, 1):
        rr, rp = wheel((sx * TR_X, TR_R, TR_Z), TR_R, 0.038, 12, 24, 0.048)
        steer_rub = steer_rub + rr
        plated = plated + rp
    plated = plated + x_axis(cyl(0.0048, h=0.420, seg=8)).move(0, TR_R, TR_Z)   # the rear axle
    bolts = Shape()
    for sx in (-1, 1):
        bolts = bolts + x_axis(cyl(0.006, h=0.006, seg=6, bevel_r=0.0007, bevel_seg=1)).move(sx * 0.012, 0.268, -0.034)
    plated = plated + bolts
    return {'paint': paint, 'plated': plated, 'rubber': steer_rub, 'grips': grips, 'seat': seat}


def rest_pose(shapes, roll_range, pitch_range, steps=24):
    """The turn (a roll about z, then a pitch about x) at which the thing
    lies lowest (its middle of mass, as the mean of its vertices, nearest
    the floor over its lowest point): how it comes to rest on its side."""
    pts = [p for s in shapes for p in s.v]
    com = sum(pts, Vector()) / len(pts)
    best = None
    for i in range(steps + 1):
        a = roll_range[0] + (roll_range[1] - roll_range[0]) * i / steps
        for j in range(steps + 1):
            b = pitch_range[0] + (pitch_range[1] - pitch_range[0]) * j / steps
            m = Matrix.Rotation(b, 3, 'X') @ Matrix.Rotation(a, 3, 'Z')
            lo = min((m @ p).y for p in pts)
            h = (m @ com).y - lo
            if best is None or h < best[0]:
                best = (h, a, b)
    return best[1], best[2]


_trike = {}


def trike(kind):
    if not _trike:
        up = trike_upright()
        a, b = rest_pose(list(up.values()), (math.radians(60), math.radians(100)), (math.radians(-25), math.radians(25)))
        m = (Matrix.Rotation(b, 4, 'X') @ Matrix.Rotation(a, 4, 'Z'))
        allv = []
        for k, s in up.items():
            s.apply(m)
            allv += s.v
        lo = Vector((min(p.x for p in allv), min(p.y for p in allv), min(p.z for p in allv)))
        hi = Vector((max(p.x for p in allv), max(p.y for p in allv), max(p.z for p in allv)))
        c = (lo + hi) / 2
        for k, s in up.items():
            s.move(-c.x, -lo.y, -c.z)
        _trike.update(up)
        print('[t16] trike rests at roll %.1f pitch %.1f deg' % (math.degrees(a), math.degrees(b)))
    return _trike[kind].copy()


auto('Real__trikeFrame', lambda: trike('paint'), 2400)
auto('Real__trikePlated', lambda: trike('plated'), 4200)
auto('Real__trikeRubber', lambda: trike('rubber'), 2400)
auto('Real__trikeGrips', lambda: trike('grips'), 600)
auto('Real__trikeSeat', lambda: trike('seat'), 900)


# ============================================================ the cruiser ===
# A plastic cruiser board (22 x 6 in: 559 x 152 mm; a moulded
# polypropylene deck, its top textured for grip, a kicked tail; 3 in cast
# aluminium trucks; 59 mm urethane wheels on steel bearings), on its wheels
# (the game turns it over, a half turn about its length, for one lying
# upside down). Its length along x, its nose +x.
CR_L, CR_W = 0.559, 0.152
CR_AX = 0.0295                      # the axles' height: the wheels' radius
CR_DECK = CR_AX + 0.047             # the deck's underside over its trucks
CR_TRUCK_X = 0.138


def cruiser_halfwidth(x):
    r = CR_W / 2 * 1.04
    ax = abs(x)
    hw = CR_W / 2
    if ax > CR_L / 2 - r:
        hw = math.sqrt(max(0.0, r * r - (ax - (CR_L / 2 - r)) ** 2))
    return min(CR_W / 2, hw) * (1 - 0.05 * math.exp(-(x / 0.10) ** 2))


def cruiser_kick(x):
    return 0.032 * smoothstep(-0.165, -CR_L / 2, x) ** 1.5 + 0.010 * smoothstep(0.19, CR_L / 2, x) ** 1.5


def cruiser_deck():
    """The deck lofted along its length from its section (a dished top, its
    rim rounded, flat beneath), kicked up at its tail and a little at its
    nose; three ribs moulded along its underside between the trucks."""
    sec = [(-1.0, 0.0085), (-0.97, 0.0118), (-0.85, 0.0125), (-0.5, 0.0118), (0.0, 0.0115), (0.5, 0.0118),
           (0.85, 0.0125), (0.97, 0.0118), (1.0, 0.0085), (0.985, 0.0025), (0.94, 0.0), (0.5, 0.0),
           (0.0, 0.0), (-0.5, 0.0), (-0.94, 0.0), (-0.985, 0.0025)]
    xs = []
    n = 46
    for k in range(n + 1):
        u = -1 + 2 * k / n
        xs.append(math.copysign(abs(u) ** 0.8, u) * (CR_L / 2 - 0.0008))
    rings = []
    for x in xs:
        hw = max(cruiser_halfwidth(x), 0.0015)
        th = 1.0 if hw > 0.012 else 0.5 + 0.5 * hw / 0.012
        y0 = CR_DECK + cruiser_kick(x)
        rings.append([Vector((x, y0 + y * th, zn * hw)) for zn, y in sec])
    v = [p for r in rings for p in r]
    m = len(sec)
    f = []
    for i in range(len(rings) - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, (i + 1) * m + j2, (i + 1) * m + j))
    f.append(tuple(reversed(range(m))))
    f.append(tuple((len(rings) - 1) * m + j for j in range(m)))
    deck = propkit._fix_winding(Shape(v, f))
    for z in (-0.045, 0.0, 0.045):
        deck = deck + rbox(0.200, 0.004, 0.004, r=0.0012, seg=1).move(0, CR_DECK - 0.0018, z)
    for x in (-0.06, 0.0, 0.06):
        deck = deck + rbox(0.004, 0.004, 0.105, r=0.0012, seg=1).move(x, CR_DECK - 0.0018, 0)
    return deck


def cruiser_truck(x0):
    """One truck at x0 under the deck: its baseplate (bolted through the
    deck), the kingpin and its nut, the hanger and its axle, the axle nuts
    and the wheels' bearing shields. Returns (metal, bushings)."""
    side = 1 if x0 > 0 else -1
    bp = rbox(0.062, 0.006, 0.042, r=0.0025, seg=1).move(x0, CR_DECK - 0.003, 0)
    # the hanger: a cast body across, tapering to the axle each side
    hz = [(-0.040, 0.008), (-0.020, 0.010), (0.0, 0.0125), (0.020, 0.010), (0.040, 0.008)]
    hang = Shape()
    for (za, ra), (zb, rb) in zip(hz, hz[1:]):
        pass
    hang = sweep([Vector((x0, CR_AX + 0.004, z)) for z, _ in hz], [r for _, r in hz], seg=10)
    hang = hang + rbox(0.020, 0.024, 0.024, r=0.005).move(x0 - side * 0.004, CR_AX + 0.018, 0)  # the pivot cup
    axle = x_axis(cyl(0.004, h=0.180, seg=8)).rot(0, math.pi / 2, 0).move(x0, CR_AX, 0)
    pin = sweep([Vector((x0 + side * 0.012, CR_DECK - 0.006, 0)), Vector((x0 + side * 0.004, CR_AX + 0.012, 0))],
                [0.0042, 0.0042], seg=8)
    nut = cyl(0.0068, h=0.006, seg=6, bevel_r=0.0008, bevel_seg=1).move(x0 + side * 0.003, CR_AX + 0.009, 0)
    metal = bp + hang + axle + pin + nut
    for sz in (-1, 1):
        for zz in (0.0855, 0.0405):         # the axle nut, and the bearing in each face
            c = cyl(0.0055 if zz > 0.06 else 0.011, h=0.004, seg=6 if zz > 0.06 else 12, bevel_r=0.0006, bevel_seg=1)
            metal = metal + c.rot(math.pi / 2, 0, 0).move(x0, CR_AX, sz * zz)
        metal = metal + cyl(0.011, h=0.004, seg=12, bevel_r=0.0006, bevel_seg=1).rot(math.pi / 2, 0, 0).move(x0, CR_AX, sz * 0.0845)
    # the bolts through the deck: their flat heads on its top, nuts below
    for bx in (-0.019, 0.019):
        for bz in (-0.014, 0.014):
            top = CR_DECK + 0.0118 + cruiser_kick(x0 + bx)
            metal = metal + cyl(0.0042, h=0.0012, seg=8, bevel_r=0.0004, bevel_seg=1).move(x0 + bx, top + 0.0003, bz)
            metal = metal + cyl(0.0045, h=0.005, seg=6, bevel_r=0.0006, bevel_seg=1).move(x0 + bx, CR_DECK - 0.0085, bz)
    bush = Shape()
    for t in (0.35, 0.72):
        c = Vector((x0 + side * (0.012 - 0.008 * t), CR_DECK - 0.006 - (CR_DECK - 0.006 - CR_AX - 0.012) * t, 0))
        bush = bush + lathe([(0.0, -0.005), (0.0085, -0.005), (0.0098, -0.003), (0.0098, 0.003), (0.0085, 0.005), (0.0, 0.005)],
                            seg=10).move(*c)
    return metal, bush


def cruiser_wheel(c):
    prof = [(0.0115, -0.0185), (0.0115, -0.0215), (0.019, -0.0225), (0.025, -0.0218), (0.0285, -0.0175),
            (0.0295, -0.009), (0.0295, 0.009), (0.0285, 0.0175), (0.025, 0.0218), (0.019, 0.0225),
            (0.0115, 0.0215), (0.0115, 0.0185)]
    return lathe(prof, seg=18, cap_bottom=True, cap_top=True).rot(math.pi / 2, 0, 0).move(*c)


def cruiser(kind):
    if kind == 'deck':
        return cruiser_deck()
    out = Shape()
    for x0 in (-CR_TRUCK_X, CR_TRUCK_X):
        if kind == 'wheels':
            for sz in (-1, 1):
                out = out + cruiser_wheel((x0, CR_AX, sz * 0.0625))
        else:
            m, b = cruiser_truck(x0)
            out = out + (m if kind == 'trucks' else b)
    return out


_flip = {}


def cruiser_flipped(kind):
    """The board upside down, as it rests: turned over about its length,
    then tipped as it settles on its kicked tail and its nose."""
    if not _flip:
        parts = {k: cruiser(k) for k in ('deck', 'trucks', 'bushings', 'wheels')}
        for sh in parts.values():
            sh.rot(math.pi, 0, 0)
        a, _ = rest_pose(list(parts.values()), (math.radians(-10), math.radians(10)), (0.0, 0.0), steps=80)
        allv = []
        for sh in parts.values():
            sh.apply(Matrix.Rotation(a, 4, 'Z'))
            allv += sh.v
        lo = min(p.y for p in allv)
        for k, sh in parts.items():
            _flip[k] = sh.move(0, -lo, 0)
        print('[t16] the board upside down rests tipped %.2f deg' % math.degrees(a))
    return _flip[kind].copy()


auto('Real__cruiserDeck', lambda: cruiser('deck'), 1800)
auto('Real__cruiserTrucks', lambda: cruiser('trucks'), 2800)
auto('Real__cruiserBushings', lambda: cruiser('bushings'), 400)
auto('Real__cruiserWheels', lambda: cruiser('wheels'), 1800)
auto('Real__cruiserDeckFlip', lambda: cruiser_flipped('deck'), 1800)
auto('Real__cruiserTrucksFlip', lambda: cruiser_flipped('trucks'), 2800)
auto('Real__cruiserBushingsFlip', lambda: cruiser_flipped('bushings'), 400)
auto('Real__cruiserWheelsFlip', lambda: cruiser_flipped('wheels'), 1800)


# =============================================================== the kite ===
# A child's paper diamond kite (a spine 600 mm and a cross spar 500 mm of
# 5 mm pine dowel, the spar a quarter down the spine; printed paper folded
# over a perimeter string; a tail of tissue bows tied on a string; a two
# legged bridle and its line), dropped face up on its spars: the paper
# sags to the floor between them, crumpled, torn open at its lower left;
# the bowstring snapped, the line and the tail lying slack. Its spine along
# z, its top +z.
K_TOP, K_BOT, K_HW, K_CROSS = 0.300, -0.300, 0.250, 0.150
K_DOW = 0.0025                       # the dowels' radius
# where the tail's three bows are tied (x, z), and the tail's way there
K_BOWS = ((-0.035, -0.505), (-0.010, -0.690), (0.070, -0.860))


def kite_spine_y(z):
    """The spine's axis height: on the floor at its ends, over the cross
    spar where they cross (lashed)."""
    return K_DOW + 2 * K_DOW * (1 - smoothstep(0.0, 0.07, abs(z - K_CROSS)))


def kite_corner(u, v):
    top, right, bot, left = (Vector((0, 0, K_TOP)), Vector((K_HW, 0, K_CROSS)), Vector((0, 0, K_BOT)),
                             Vector((-K_HW, 0, K_CROSS)))
    return top * (1 - u) * (1 - v) + right * u * (1 - v) + bot * u * v + left * (1 - u) * v


K_TEAR_C = Vector((-0.115, 0, -0.085))


def kite_tear_r(d):
    """The tear's ragged edge: its radius (in its squashed measure) toward d."""
    a = math.atan2(d.z, d.x)
    return 0.062 + 0.018 * math.sin(7.0 * a + 1.0) + 0.009 * math.sin(17.0 * a + 2.0) + 0.006 * n1(a * 3.0, 5.0)


def kite_tear_m(d):
    return math.hypot(d.x * 0.85, d.z * 1.25)


def kite_tear(p):
    """Is plan point p inside the tear (the paper gone)? A ragged bite out
    of the lower left edge."""
    d = p - K_TEAR_C
    return kite_tear_m(d) < kite_tear_r(d)


def kite_height(p, edge):
    """The paper's height at plan point p: on the spars, sagging to the floor
    between them, crumpled; lifted and curled where it was torn."""
    sp = max(0.0, abs(p.x) - 0.009)
    cr = max(0.0, abs(p.z - K_CROSS) - 0.009)
    on = max(kite_spine_y(p.z) + K_DOW + 0.0004 - 0.22 * sp, 2 * K_DOW + 0.0004 - 0.22 * cr, 0.0007)
    y = on + 0.0035 * max(0.0, n1(p.x * 9, p.z * 9, 1.0)) + 0.0018 * n1(p.x * 23, p.z * 23, 2.0)
    y += edge * (0.006 + 0.004 * n1(p.x * 30, p.z * 30, 3.0))
    return max(y, 0.0006)


def kite_sail():
    n = 32
    grid = {}
    v, f = [], []
    keep = {}
    for i in range(n):
        for j in range(n):
            keep[(i, j)] = not kite_tear(kite_corner((i + 0.5) / n, (j + 0.5) / n))
    # the vertices of the cells left round the tear are moved onto its
    # ragged edge, and curl up off the floor as torn paper does
    for i in range(n + 1):
        for j in range(n + 1):
            cells = [(i + a, j + b) for a in (-1, 0) for b in (-1, 0) if 0 <= i + a < n and 0 <= j + b < n]
            near = [keep[c] for c in cells]
            p = kite_corner(i / n, j / n)
            edge = 0.0
            if any(near) and not all(near):
                d = p - K_TEAR_C
                m = kite_tear_m(d)
                if m > 1e-6:
                    p = K_TEAR_C + d * (kite_tear_r(d) / m)
                edge = 1.0
            p.y = kite_height(p, edge)
            grid[(i, j)] = len(v)
            v.append(p)
    for i in range(n):
        for j in range(n):
            if keep[(i, j)]:
                f.append((grid[(i, j)], grid[(i + 1, j)], grid[(i + 1, j + 1)], grid[(i, j + 1)]))
    s = Shape(v, f)
    # one sided paper: its top up
    for k, fc in enumerate(s.f):
        a, b, c = s.v[fc[0]], s.v[fc[1]], s.v[fc[2]]
        if (b - a).cross(c - a).y < 0:
            s.f[k] = tuple(reversed(fc))
    return s


def kite_spars():
    spine = []
    for k in range(25):
        z = K_BOT + 0.004 + (K_TOP - K_BOT - 0.008) * k / 24
        spine.append(Vector((0.0, kite_spine_y(z), z)))
    s = sweep(spine, [K_DOW] * len(spine), seg=6)
    cross = [Vector((x, K_DOW, K_CROSS)) for x in (-K_HW + 0.004, -0.1, 0.0, 0.1, K_HW - 0.004)]
    s = s + sweep(cross, [K_DOW] * len(cross), seg=6)
    return s


def kite_strings():
    s = Shape()
    # the perimeter string, bare where the tear took the paper from it
    left, bot = Vector((-K_HW, 0, K_CROSS)), Vector((0, 0, K_BOT))
    ts = [k / 200 for k in range(201) if kite_tear(left.lerp(bot, k / 200))]
    if ts:
        t0, t1 = max(0.0, ts[0] - 0.03), min(1.0, ts[-1] + 0.03)
        pts = []
        for k in range(9):
            t = t0 + (t1 - t0) * k / 8
            q = left.lerp(bot, t)
            pts.append(Vector((q.x, 0.0009 + 0.0012 * math.sin(math.pi * k / 8), q.z)))
        s = s + sweep(pts, [0.0007] * len(pts), seg=4)
    # the lashing where the spars cross
    for k in range(5):
        a = TAU * k / 5
        s = s + torus(0.0062, 0.0008, seg=10, tseg=4).scale(1, 0.62, 1).rot(0, math.pi / 4 * (1 if k % 2 else -1), 0).move(0, 2 * K_DOW, K_CROSS)
    # the bridle on the face, from the spine at 180 and -150 to its towing knot
    knot = Vector((0.085, 0.0, 0.01))
    for z in (0.180, -0.150):
        a = Vector((0.0, 2 * K_DOW + 0.004 if z > 0 else K_DOW * 2 + 0.0008, z))
        mid = a.lerp(knot, 0.5)
        pts = [a, Vector((mid.x, kite_height(mid, 0.0) + 0.0012, mid.z)), Vector((knot.x, kite_height(knot, 0.0) + 0.0012, knot.z))]
        s = s + sweep(chaikin(pts, 2), [0.0007] * len(chaikin(pts, 2)), seg=4)
    # the line from the knot off the kite's right edge, lying slack, its end frayed
    line = [(0.085, 0.01), (0.20, 0.03), (0.30, 0.04), (0.42, 0.00), (0.50, -0.10), (0.47, -0.22),
            (0.38, -0.27), (0.33, -0.36), (0.40, -0.47), (0.52, -0.50)]
    ax = lay_cord(line, 0.0007, step=0.012, relax=3)
    for i, p in enumerate(ax):
        q = Vector((p.x, 0, p.z))
        if abs(q.x) < K_HW * (1 - abs(q.z - K_CROSS) / 0.45) + 0.004:
            p.y = max(p.y, kite_height(q, 0.0) + 0.0008)
    s = s + sweep(ax, [0.0007] * len(ax), seg=4)
    # the tail from the bottom tip, through its bows, slack
    tail = [(0.0, K_BOT + 0.002), (-0.03, -0.38), (-0.045, -0.44), K_BOWS[0], (-0.01, -0.58), (-0.03, -0.63),
            K_BOWS[1], (0.02, -0.76), (0.05, -0.81), K_BOWS[2], (0.10, -0.91), (0.09, -0.96)]
    ax = lay_cord(tail, 0.0007, step=0.012, relax=3)
    s = s + sweep(ax, [0.0007] * len(ax), seg=4)
    # where the bows sit on it, and its way there (the game ties them on)
    for b in K_BOWS:
        i = min(range(1, len(ax) - 1), key=lambda k: (Vector((ax[k].x, 0, ax[k].z)) - Vector((b[0], 0, b[1]))).length)
        d = ax[i + 1] - ax[i - 1]
        print('[t16] kite bow at (%.4f, %.4f), the tail going %.4f rad' % (ax[i].x, ax[i].z, math.atan2(d.x, d.z)))
    # the bowstring, snapped: a loose end from the cross spar's right tip
    bs = [(K_HW - 0.004, K_CROSS), (K_HW - 0.03, K_CROSS - 0.02), (K_HW - 0.07, K_CROSS - 0.015), (K_HW - 0.10, K_CROSS - 0.03)]
    ax = lay_cord(bs, 0.0007, step=0.01, relax=2)
    for p in ax:
        p.y = max(p.y, kite_height(Vector((p.x, 0, p.z)), 0.0) + 0.0008)
    s = s + sweep(ax, [0.0007] * len(ax), seg=4)
    return s


def kite_bow():
    """A tissue bow tied on the tail: a strip 140 x 40 mm pinched at its
    middle by the knot, its two wings puffed and crumpled; across x, its
    knot at the origin, lying flat."""
    v, f = [], []
    nu, nw = 14, 4
    for i in range(nu + 1):
        u = -1 + 2 * i / nu
        pinch = 0.25 + 0.75 * smoothstep(0.0, 0.55, abs(u))
        for j in range(nw + 1):
            w = -1 + 2 * j / nw
            x = u * 0.060
            z = w * 0.020 * pinch
            y = 0.0012 + 0.006 * math.sin(math.pi * min(1.0, abs(u) * 1.15)) * (1 - 0.6 * w * w) + 0.0015 * n1(x * 60, z * 60, 7.0)
            v.append(Vector((x, max(y, 0.0006), z)))
    for i in range(nu):
        for j in range(nw):
            a = i * (nw + 1) + j
            f.append((a, a + 1, a + nw + 2, a + nw + 1))
    s = Shape(v, f)
    for k, fc in enumerate(s.f):
        a, b, c = s.v[fc[0]], s.v[fc[1]], s.v[fc[2]]
        if (b - a).cross(c - a).y < 0:
            s.f[k] = tuple(reversed(fc))
    knot = sphere(0.0035, seg=8, rings=5).scale(1.0, 0.8, 1.4).move(0, 0.0035, 0)
    return s + knot


auto('Real__kiteSail', kite_sail, 2000)
auto('Real__kiteSpars', kite_spars, 500)
auto('Real__kiteString', kite_strings, 2400)
auto('Real__kiteBow', kite_bow, 200)


# ======================================================== the metallophone ===
# A child's eight note "xylophone" (a metallophone: 26.5 x 10.5 cm, eight
# 3 mm aluminium plates enamelled in colours, C to C, on a pine frame, a
# wooden mallet): the bars stepping down from 100 to 68 mm, each pinned
# through its two nodes (22.4 % in from its ends) over a felt strip on the
# two rails, which converge under the nodes; its mallet lying in front.
# Its length along x (low notes -x), its front +z.
XY_N = 8
XY_PITCH = 0.028
XY_BW, XY_BT = 0.022, 0.003
XY_RAIL_H, XY_FELT = 0.018, 0.0015


def xylo_bar_len(i):
    return 0.100 - i * (0.032 / (XY_N - 1))


def xylo_bar_x(i):
    return -XY_PITCH * (XY_N - 1) / 2 + i * XY_PITCH


def xylo_node(i):
    return xylo_bar_len(i) * (0.5 - 0.224)


def xylo_rail_z(x):
    """The rail's middle (z > 0 side) at x: under the bars' nodes."""
    x0, x1 = xylo_bar_x(0), xylo_bar_x(XY_N - 1)
    t = (x - x0) / (x1 - x0)
    return xylo_node(0) + (xylo_node(XY_N - 1) - xylo_node(0)) * t


XY_X0, XY_X1 = xylo_bar_x(0) - 0.026, xylo_bar_x(XY_N - 1) + 0.026


def xylo_frame():
    """Two pine rails (16 x 18 mm), splayed, joined by an end block at each
    end; every edge eased."""
    s = Shape()
    for sz in (-1, 1):
        a = Vector((XY_X0 + 0.006, XY_RAIL_H / 2, sz * xylo_rail_z(XY_X0 + 0.006)))
        b = Vector((XY_X1 - 0.006, XY_RAIL_H / 2, sz * xylo_rail_z(XY_X1 - 0.006)))
        L = (b - a).length
        r = rbox(L, XY_RAIL_H, 0.016, r=0.0018, seg=2)
        ang = math.atan2(-(b.z - a.z), b.x - a.x)
        r.rot(0, ang, 0)
        s = s + r.move(*((a + b) / 2))
    for x in (XY_X0 + 0.006, XY_X1 - 0.006):
        zz = xylo_rail_z(x) + 0.008
        s = s + rbox(0.014, XY_RAIL_H - 0.002, 2 * zz, r=0.0018, seg=2).move(x, (XY_RAIL_H - 0.002) / 2, 0)
    return s


def xylo_bars():
    s = Shape()
    for i in range(XY_N):
        L = xylo_bar_len(i)
        b = prism(rounded_rect(XY_BW, L, 0.003, seg=2), XY_BT, bevel_r=0.0006, bevel_seg=1)
        b.rot(-math.pi / 2, 0, 0)
        s = s + b.move(xylo_bar_x(i), XY_RAIL_H + XY_FELT + XY_BT / 2, 0)
    return s


def xylo_felt():
    s = Shape()
    for sz in (-1, 1):
        a = Vector((xylo_bar_x(0) - 0.014, 0, sz * xylo_rail_z(xylo_bar_x(0) - 0.014)))
        b = Vector((xylo_bar_x(XY_N - 1) + 0.014, 0, sz * xylo_rail_z(xylo_bar_x(XY_N - 1) + 0.014)))
        L = (b - a).length
        r = box(L, XY_FELT, 0.010)
        r.rot(0, math.atan2(-(b.z - a.z), b.x - a.x), 0)
        s = s + r.move((a.x + b.x) / 2, XY_RAIL_H + XY_FELT / 2, (a.z + b.z) / 2)
    return s


def xylo_pins():
    s = Shape()
    head = lathe([(0.0, 0.0), (0.0026, 0.0), (0.0026, 0.0004), (0.0018, 0.0010), (0.0, 0.0012)], seg=6)
    for i in range(XY_N):
        for sz in (-1, 1):
            s = s + head.copy().move(xylo_bar_x(i), XY_RAIL_H + XY_FELT + XY_BT, sz * xylo_node(i))
    return s


XY_MALLET_HEAD = Vector((0.050, 0.011, 0.090))
XY_MALLET_DIR = Vector((-0.985, 0.0, 0.17)).normalized()


def xylo_mallet():
    """The mallet's handle: a 7 mm beech dowel, its end in the head's bore,
    its other end on the floor."""
    a = XY_MALLET_HEAD + XY_MALLET_DIR * 0.004
    b = XY_MALLET_HEAD + XY_MALLET_DIR * 0.175
    b.y = 0.0035
    return sweep([a, a.lerp(b, 0.5), b], [0.0035, 0.0035, 0.0034], seg=8)


def xylo_mallet_head():
    h = sphere(0.011, seg=14, rings=9)
    return h.move(*XY_MALLET_HEAD)


auto('Real__xyloFrame', xylo_frame, 600)
auto('Real__xyloBars', xylo_bars, 900)
auto('Real__xyloFelt', xylo_felt, 60)
auto('Real__xyloPins', xylo_pins, 600)
auto('Real__xyloMallet', xylo_mallet, 120)
auto('Real__xyloMalletHead', xylo_mallet_head, 300)
