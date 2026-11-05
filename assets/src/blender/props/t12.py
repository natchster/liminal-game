"""t12: what a party room leaves, years on (docs/PROPS.md P3, "the party and
the shrine"): cheap paper, card, foil and crepe, faded, crumpled and dusty.
Sources and sizes: t12.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is in its
own frame, given with it: x across, y up, z out of its front; a floor
thing's floor is y = 0, a ceiling thing's ceiling is y = 0 (it hangs below).

The party hat (a printed card cone, 175 mm tall, 115 mm across its rim)
partyHatCone          the cone of card (0.4 mm), its glued flap's edge, a dent
                      creased into one side, the point bent; inside and out
                      (the game maps the print outside, plain card inside);
                      standing on its rim (y up its axis)
partyHatTassel        the foil fringe pushed into its point, burst out
partyHatElastic       the round elastic, knotted inside through two holes,
                      lying on the floor in a loop (the hat standing)
partyHatElasticLying  the same with the hat lying on its side (the game's
                      turn, HAT_LIE, and lift, HAT_LIFT; modelled in the
                      cone's frame)
partyHatTasselLying   the fringe of a hat lying, the strands under it
                      pressed flat on the floor
"""
import math
import random
import zlib
import os
import time

import bpy
import bmesh
from propkit import *
import propkit

CYL_FRAMED = {'Real__partyHatCone'}
ONLY = set(filter(None, os.environ.get('T12_ONLY', '').split(',')))


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name (the
    shape is built here, at registration)."""
    if ONLY and name not in ONLY:
        return
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    kind = 'cyl' if name in CYL_FRAMED else 'box'
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, smooth=smooth, flat=flat, budget=budget)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def n1(x, y=0.0, z=0.0):
    """Scalar Perlin noise (repeats in every Blender process)."""
    return noise.noise(Vector((x, y, z)))


def wrap_pi(a):
    return math.atan2(math.sin(a), math.cos(a))


def sweep(pts, radii, seg=6, cap0=True, cap1=True, up=None):
    """A tube along `pts` whose radius at each point is `radii[i]`
    (parallel-transport frame); cap0/cap1 close its ends."""
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


def strip(pts, sides, widths):
    """A flat strip (one sided; the game draws it from both) along `pts`,
    `widths[i]` across in the direction `sides[i]`."""
    v, f = [], []
    for p, sd, w in zip(pts, sides, widths):
        v.append(Vector(p) - Vector(sd) * w / 2)
        v.append(Vector(p) + Vector(sd) * w / 2)
    for i in range(len(pts) - 1):
        f.append((2 * i, 2 * i + 1, 2 * i + 3, 2 * i + 2))
    return Shape(v, f)


def lump(c, r, squash=(1.0, 1.0, 1.0), seg=6):
    return ellipsoid(r * squash[0], r * squash[1], r * squash[2], seg=seg).move(*c)


def rot_about(p, pivot, axis, ang):
    m = Matrix.Rotation(ang, 3, Vector(axis).normalized())
    return pivot + m @ (Vector(p) - pivot)


# ============================================================ the party hat ==
# A printed card party hat: the common cone of 300 g/m2 board, 7 in (178 mm)
# tall over a 4.5 in (115 mm) rim, rolled from a printed sector and glued
# along a 15 mm flap; a 12 in round elastic knotted through two holes; a
# pompom of foil fringe pushed into its point. Stood on its rim, y up its
# axis; the rim's lowest point on y = 0.
HAT_H, HAT_R, HAT_T = 0.175, 0.0575, 0.0004
HAT_A = math.atan2(HAT_R, HAT_H)                 # its half angle
HAT_SLANT = math.hypot(HAT_R, HAT_H)
HAT_SEAM0, HAT_FLAP = 0.35, 0.26                 # the under edge, the overlap (rad)
HAT_SEAM1 = HAT_SEAM0 + HAT_FLAP                 # the flap's edge, seen
HAT_DENT = (2.50, 0.0065)                        # the crease's angle, depth
HAT_HOLE = 1.45                                  # the elastic's holes: here and opposite
HAT_HOLE_S = 0.012 / HAT_SLANT                   # 12 mm up the slant
HAT_TIP_S = 0.972                                # where the point is closed round the fringe
HAT_BEND = (0.95, 0.80, 0.30)                    # the point bent: towards, from s, by (rad)
HAT_LIE = math.pi / 2 + HAT_A                    # the game's turn (about z) of a hat lying


def _hat_bend(p):
    """The point bent over (every point above s = HAT_BEND[1] turned about a
    pivot on the axis, more the higher it is)."""
    s = p.y / HAT_H
    if s <= HAT_BEND[1]:
        return p
    ang = HAT_BEND[2] * smoothstep(HAT_BEND[1], 1.0, s)
    rad = Vector((math.sin(HAT_BEND[0]), 0, math.cos(HAT_BEND[0])))
    axis = Vector((0, 1, 0)).cross(rad)
    return rot_about(p, Vector((0, HAT_H * HAT_BEND[1], 0)), axis, ang)


def hat_point(phi, s, off=0.0):
    """The cone's surface at angle phi (from +z towards +x), s (0 rim, 1
    point) along its slant, `off` out along its normal; dented, bent, the
    rim gone a little wavy."""
    rad = Vector((math.sin(phi), 0.0, math.cos(phi)))
    n = rad * math.cos(HAT_A) + Vector((0, math.sin(HAT_A), 0))
    r = HAT_R * (1 - s)
    p = Vector((rad.x * r, HAT_H * s, rad.z * r))
    # the dent: pushed in along a crease (a V across phi), deepest a third up
    da = wrap_pi(phi - HAT_DENT[0])
    w = max(0.0, 1.0 - abs(da) / 0.42)
    prof = math.sin(math.pi * min(1.0, max(0.0, (s - 0.10) / 0.62)))
    p -= rad * (HAT_DENT[1] * w * prof)
    # the card gone wavy at the rim, a little out of round
    p.y += 0.0015 * math.sin(2 * phi + 0.5) * max(0.0, 1 - s / 0.3) + 0.0006 * n1(phi * 1.3, s * 4, 0.3)
    p.x *= 1.025
    p.z *= 0.975
    p += n * off
    return _hat_bend(p)


def hat_tip():
    """The point (where the fringe goes in) and the axis there."""
    a = _hat_bend(Vector((0, HAT_H * HAT_TIP_S, 0)))
    b = _hat_bend(Vector((0, HAT_H * 0.999, 0)))
    return a, (b - a).normalized()


def _hat_phis():
    special = [HAT_SEAM1, HAT_SEAM1 + 0.004, HAT_DENT[0]]
    out = list(special)
    for k in range(34):
        a = TAU * k / 34
        if all(abs(wrap_pi(a - b)) > 0.05 for b in special):
            out.append(a)
    return sorted(x % TAU for x in out)


def _hat_lift_raw():
    lo = min(hat_point(TAU * k / 90, 0.0, -HAT_T / 2).y for k in range(90))
    return -lo


def partyhat_cone():
    phis = _hat_phis()
    ss = [0.0, 0.015, 0.04, 0.08, 0.14, 0.21, 0.29, 0.38, 0.47, 0.56, 0.65, 0.74, 0.82, 0.89, 0.94, HAT_TIP_S]
    n = len(phis)
    v, f = [], []

    def flap(phi):
        return HAT_SEAM0 <= phi <= HAT_SEAM1 + 1e-9

    for layer in (0, 1):                          # 0 outside, 1 inside
        for s in ss:
            for phi in phis:
                off = (HAT_T / 2 + (HAT_T if flap(phi) else 0.0)) if layer == 0 else -HAT_T / 2
                v.append(hat_point(phi, s, off))
    ns = len(ss)

    def vid(layer, j, i):
        return layer * ns * n + j * n + (i % n)

    for j in range(ns - 1):
        for i in range(n):
            f.append((vid(0, j, i), vid(0, j, i + 1), vid(0, j + 1, i + 1), vid(0, j + 1, i)))
            f.append((vid(1, j, i), vid(1, j + 1, i), vid(1, j + 1, i + 1), vid(1, j, i + 1)))
    for i in range(n):                            # the rim's cut edge
        f.append((vid(1, 0, i), vid(1, 0, i + 1), vid(0, 0, i + 1), vid(0, 0, i)))
    # the point: the two layers pinched shut
    tip, ax = hat_tip()
    v.append(tip + ax * 0.0015)
    c = len(v) - 1
    for i in range(n):
        f.append((vid(0, ns - 1, i), vid(0, ns - 1, i + 1), c))
        f.append((vid(1, ns - 1, i + 1), vid(1, ns - 1, i), c))
    s = Shape(v, f)
    s.move(0, _hat_lift_raw(), 0)
    return s


def _fringe_strand(rng, root, axis, side0, length, width, spread, droop, seg=7, crinkle=0.55):
    """One foil strand from `root` out round `axis`: crinkled (it kinks
    back and forth along its length), curling over and down."""
    pts, sides, ws = [], [], []
    d = axis.copy()
    p = Vector(root)
    sd = Vector(side0)
    curl = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
    for k in range(seg + 1):
        t = k / seg
        pts.append(p.copy())
        sides.append(sd.copy())
        ws.append(width * (1.0 - 0.3 * t))
        # turn out from the axis, down under its own weight, curl, and kink
        kink = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * crinkle
        d = (d + spread * (1.3 - t) / seg + Vector((0, -droop * t, 0)) / seg + curl * (0.9 / seg) + kink / seg * 2.2).normalized()
        sd = (sd + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * 0.6).normalized()
        sd = (sd - d * sd.dot(d)).normalized()
        p = p + d * (length / seg)
    return strip(pts, sides, ws)


def _tassel_strands(rng):
    """The pompom's strands: (root, start dir, side, length, out dir)."""
    tip, ax = hat_tip()
    perp = ax.cross(Vector((1, 0, 0))).normalized()
    perp2 = ax.cross(perp).normalized()
    out = []
    count = 56
    for k in range(count):
        a = TAU * (k + rng.uniform(-0.3, 0.3)) / count
        el = rng.uniform(0.15, 1.15)             # how far out from the axis it starts
        radial = perp * math.cos(a) + perp2 * math.sin(a)
        d0 = (ax * math.cos(el) + radial * math.sin(el)).normalized()
        root = tip + ax * 0.002 + radial * 0.0008
        side = ax.cross(d0)
        side = side.normalized() if side.length > 1e-6 else perp
        side = (side + radial * rng.uniform(-0.6, 0.6)).normalized()
        length = rng.uniform(0.022, 0.040)
        out.append((root, d0, side, length, radial))
    return out


def partyhat_tassel(lying=False):
    rng = random.Random(1207)
    s = Shape()
    for root, d0, side, length, radial in _tassel_strands(rng):
        st = _fringe_strand(rng, root, d0, side, length, 0.0019, radial * 1.1, 0.9)
        s = s + st
    s.move(0, _hat_lift_raw(), 0)
    if lying:
        s = _hat_floor_press(s)
    return s


# --- the hat lying: its turn and lift in the game, and what that presses ---
def _hat_pose(p, lift):
    c, sn = math.cos(HAT_LIE), math.sin(HAT_LIE)
    return Vector((p.x * c - p.y * sn, p.x * sn + p.y * c + lift, p.z))


def _hat_unpose(p, lift):
    c, sn = math.cos(HAT_LIE), math.sin(HAT_LIE)
    x, y = p.x, p.y - lift
    return Vector((x * c + y * sn, -x * sn + y * c, p.z))


_HAT_LIFT = None


def hat_lift():
    """How far the game lifts a hat lying (HAT_LIFT): its lowest point on
    the floor."""
    global _HAT_LIFT
    if _HAT_LIFT is None:
        cone = partyhat_cone()
        _HAT_LIFT = -min(_hat_pose(p, 0.0).y for p in cone.v)
        print('t12: HAT_LIE %.6f HAT_LIFT %.6f' % (HAT_LIE, _HAT_LIFT))
    return _HAT_LIFT


def _hat_floor_press(s):
    lift = hat_lift()
    out = s.copy()
    for i, p in enumerate(out.v):
        q = _hat_pose(p, lift)
        if q.y < 0.0004:
            q.y = 0.0002 + 0.0002 * n1(q.x * 300, q.z * 300)
        out.v[i] = _hat_unpose(q, lift)
    return out


def _elastic_tube(path, r=0.00065):
    path = chaikin(path, 1)
    return sweep(path, [r] * len(path), seg=4, cap0=True, cap1=True)


def _hat_hole(phi):
    p = hat_point(phi, HAT_HOLE_S, 0.0)
    rad = Vector((math.sin(phi), 0.0, math.cos(phi)))
    n = (rad * math.cos(HAT_A) + Vector((0, math.sin(HAT_A), 0))).normalized()
    return p + Vector((0, _hat_lift_raw(), 0)), n


def _knots():
    s = Shape()
    for phi in (HAT_HOLE, HAT_HOLE + math.pi):
        p, n = _hat_hole(phi)
        s = s + lump(p - n * 0.0018, 0.0016, (1.0, 0.8, 1.0))
    return s


def partyhat_elastic():
    """Standing: from each hole the cord falls to the floor and lies there in
    a loose loop round the front of the rim (305 mm between the holes)."""
    rng = random.Random(77)
    h1, n1_ = _hat_hole(HAT_HOLE)
    h2, n2_ = _hat_hole(HAT_HOLE + math.pi)
    r_ = 0.00065
    pts = [h1 - n1_ * 0.0015, h1 + n1_ * 0.002, h1 + n1_ * 0.006 + Vector((0, -0.006, 0))]
    a0, a1 = HAT_HOLE, HAT_HOLE + math.pi
    steps = 22
    for k in range(steps + 1):
        t = k / steps
        a = a0 - (a1 - a0) * t                   # round the other way: the front
        bulge = 0.055 * math.sin(math.pi * t) ** 0.7 + 0.006 * math.sin(t * 9 + 1)
        rr = HAT_R * 1.04 + 0.006 + bulge
        pts.append(Vector((math.sin(a) * rr * 1.025, r_, math.cos(a) * rr * 0.975))
                   + Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * 0.002)
    pts += [h2 + n2_ * 0.006 + Vector((0, -0.006, 0)), h2 + n2_ * 0.002, h2 - n2_ * 0.0015]
    s = _elastic_tube(pts, r_)
    # what of it lies on the floor: down onto it
    s = s.warp(lambda p: Vector((p.x, max(p.y, 0.00005), p.z)))
    return s + _knots()


def partyhat_elastic_lying():
    """The hat lying on its side: the cord from the upper hole falls over the
    cone to the floor, and both lie in a loop past the rim."""
    rng = random.Random(78)
    lift = hat_lift()
    h1, n1_ = _hat_hole(HAT_HOLE)
    h2, n2_ = _hat_hole(HAT_HOLE + math.pi)
    P1, P2 = _hat_pose(h1, lift), _hat_pose(h2, lift)
    N1 = _hat_pose(h1 + n1_, lift) - P1
    N2 = _hat_pose(h2 + n2_, lift) - P2
    r_ = 0.00065
    # the rim's centre, lying, and the way out of it
    rim_c = _hat_pose(Vector((0, _hat_lift_raw(), 0)), lift)
    out_dir = (rim_c - _hat_pose(Vector((0, _hat_lift_raw() + HAT_H, 0)), lift))
    out_dir.y = 0
    out_dir.normalize()
    side = Vector((0, 1, 0)).cross(out_dir).normalized()
    # over the cone's flank, beside the rim, and down to the floor
    pts = [P1 - N1 * 0.0015, P1 + N1 * 0.003, P1 + N1 * 0.006 + side * 0.010]
    pts.append(rim_c + side * (HAT_R + 0.006) + Vector((0, 0.012, 0)) - out_dir * 0.006)
    f1 = Vector((rim_c.x, r_, rim_c.z)) + side * (HAT_R + 0.014) + out_dir * 0.006
    pts.append(f1)
    f2 = Vector((P2.x, r_, P2.z)) + out_dir * 0.012 - side * 0.006
    # a loose loop on the floor in front of the open rim
    steps = 18
    for k in range(1, steps):
        t = k / steps
        base = f1.lerp(f2, t)
        bulge = out_dir * (0.085 * math.sin(math.pi * t) ** 0.7) + side * (0.035 * math.sin(math.pi * t) * (1 - t))
        pts.append(base + bulge + Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * 0.002)
    pts.append(f2)
    pts += [P2 + N2 * 0.004 + out_dir * 0.003, P2 + N2 * 0.0015, P2 - N2 * 0.0015]
    for p in pts[2:-2]:
        if p.y < r_ * 1.5:
            p.y = r_
    s = _elastic_tube(pts, r_)
    s = s.warp(lambda p: Vector((p.x, max(p.y, 0.00005), p.z)))
    s = Shape([_hat_unpose(p, lift) for p in s.v], s.f)
    return s + _knots()


auto('Real__partyHatCone', partyhat_cone, 2400, smooth=40.0)
auto('Real__partyHatTassel', partyhat_tassel, 800, smooth=60.0)
auto('Real__partyHatTasselLying', lambda: partyhat_tassel(True), 800, smooth=60.0)
auto('Real__partyHatElastic', partyhat_elastic, 700, smooth=60.0)
auto('Real__partyHatElasticLying', partyhat_elastic_lying, 700, smooth=60.0)


# ============================================================ the party horn ==
# A 9 in (229 mm) party horn: a cone of card covered in printed foil, a
# white plastic mouthpiece with its reed pushed into the narrow end, a
# foil fringe glued round the bell. Built standing on its bell (y up its
# axis, the bell's rim at y = 0); the game lays it down (HORN_LIE*: its turn
# about z, and the lift that puts it on the floor, printed as it builds).
HORN_L, HORN_RB, HORN_RT, HORN_T = 0.200, 0.024, 0.0048, 0.0004
HORN_A = math.atan2(HORN_RB - HORN_RT, HORN_L)
HORN_SEAM0, HORN_FLAP = 2.2, 0.30
HORN_SEAM1 = HORN_SEAM0 + HORN_FLAP
HORN_MP = (0.197, 0.229)                     # the mouthpiece: from (inside the cone) to its lip


def _horn_tread(p, trod):
    """The trodden horn: its bell half pressed flat (across x), out along z."""
    if not trod:
        return p
    t = max(0.0, 1.0 - p.y / 0.13) ** 1.6
    q = p.copy()
    q.x *= 1.0 - 0.62 * t
    q.z *= 1.0 + 0.30 * t
    # the folds where it was pressed: sharper at the two edges
    q.x += 0.0025 * t * math.sin(p.z * 260.0) * (abs(p.z) / max(1e-6, HORN_RB))
    return q


def horn_point(phi, s, off=0.0, trod=False):
    """The cone at angle phi, s (0 bell .. 1 narrow end), `off` along its
    normal."""
    rad = Vector((math.sin(phi), 0.0, math.cos(phi)))
    n = rad * math.cos(HORN_A) + Vector((0, math.sin(HORN_A), 0))
    r = HORN_RB + (HORN_RT - HORN_RB) * s
    p = Vector((rad.x * r, HORN_L * s, rad.z * r)) + n * off
    # a slight wave in the bell's card, and a soft dent
    p.y += 0.0008 * math.sin(3 * phi + 1.0) * max(0.0, 1 - s / 0.15)
    p -= rad * (0.0012 * max(0.0, 1 - abs(wrap_pi(phi - 4.1)) / 0.5) * math.sin(math.pi * min(1, max(0, (s - 0.3) / 0.4))))
    return _horn_tread(p, trod)


def _horn_phis(n=28):
    special = [HORN_SEAM1, HORN_SEAM1 + 0.004]
    out = list(special)
    for k in range(n):
        a = TAU * k / n
        if all(abs(wrap_pi(a - b)) > 0.06 for b in special):
            out.append(a)
    return sorted(x % TAU for x in out)


def party_horn_cone(trod=False):
    phis = _horn_phis()
    ss = [0.0, 0.02, 0.06, 0.12, 0.2, 0.3, 0.42, 0.55, 0.68, 0.8, 0.9, 1.0]
    n, ns = len(phis), len(ss)
    v, f = [], []

    def flap(phi):
        return HORN_SEAM0 <= phi <= HORN_SEAM1 + 1e-9

    for layer in (0, 1):
        for s in ss:
            for phi in phis:
                off = (HORN_T / 2 + (HORN_T if flap(phi) else 0.0)) if layer == 0 else -HORN_T / 2
                v.append(horn_point(phi, s, off, trod))

    def vid(layer, j, i):
        return layer * ns * n + j * n + (i % n)

    for j in range(ns - 1):
        for i in range(n):
            f.append((vid(0, j, i), vid(0, j, i + 1), vid(0, j + 1, i + 1), vid(0, j + 1, i)))
            f.append((vid(1, j, i), vid(1, j + 1, i), vid(1, j + 1, i + 1), vid(1, j, i + 1)))
    for j in (0, ns - 1):                        # the bell's and the narrow end's cut edges
        for i in range(n):
            if j == 0:
                f.append((vid(1, j, i), vid(1, j, i + 1), vid(0, j, i + 1), vid(0, j, i)))
            else:
                f.append((vid(0, j, i), vid(0, j, i + 1), vid(1, j, i + 1), vid(1, j, i)))
    return Shape(v, f)


def party_horn_mouth():
    """The plastic mouthpiece: its spigot in the cone, a flange against the
    card's end, the mouth tube flattened to a lip, the reed's slot."""
    y0, y1 = HORN_MP
    prof = [(0.0, y0), (HORN_RT - 0.0006, y0), (HORN_RT - 0.0004, HORN_L - 0.001), (0.0064, HORN_L - 0.0005),
            (0.0066, HORN_L + 0.0025), (0.0050, HORN_L + 0.0045), (0.0046, HORN_L + 0.012),
            (0.0050, y1 - 0.004), (0.0052, y1 - 0.0012), (0.0040, y1), (0.0016, y1 - 0.0004), (0.0016, y1 - 0.006), (0.0, y1 - 0.006)]
    s = lathe(prof, seg=14, cap_bottom=False, cap_top=False)
    # the lip pressed flat across x
    def flat(p):
        t = smoothstep(HORN_L + 0.010, y1 - 0.002, p.y)
        return Vector((p.x * (1 + 0.35 * t), p.y, p.z * (1 - 0.40 * t)))
    return s.warp(flat)


def _horn_fringe_raw(trod=False):
    """Foil fringe round the bell: its uncut band glued over the card's edge
    (12 mm), its strands (2.2 mm, 30 mm) splayed out past the rim."""
    rng = random.Random(4411)
    s = Shape()
    band_lo, band_hi = -0.002, 0.012
    nb = 40
    v, f = [], []
    for j, y in enumerate((band_lo, band_hi)):
        for i in range(nb):
            a = TAU * i / nb
            p = horn_point(a, max(0.0, y) / HORN_L, HORN_T * 2.2, trod)
            p.y = y + 0.0005 * n1(a * 2, j)
            v.append(p)
    for i in range(nb):
        i2 = (i + 1) % nb
        f.append((i, i2, nb + i2, nb + i))
    s = Shape(v, f)
    count = 64
    for k in range(count):
        a = TAU * (k + rng.uniform(-0.25, 0.25)) / count
        if rng.random() < 0.08:
            continue                                  # torn off
        root = horn_point(a, 0.0, HORN_T * 2.4, trod)
        root.y = band_lo + 0.001
        rad = Vector((math.sin(a), 0, math.cos(a)))
        down = Vector((0, -1, 0))
        flare = rng.uniform(0.35, 1.05)
        d0 = (down * math.cos(flare) + rad * math.sin(flare)).normalized()
        side = Vector((math.cos(a), 0, -math.sin(a)))
        length = rng.uniform(0.020, 0.031)
        s = s + _fringe_strand(rng, root, d0, side, length, 0.0021, rad * 0.6, -0.4, seg=5, crinkle=0.45)
    return s


# --- the horn lying: the turn that lays it on the floor, and the lift ---
def _pose_z(p, beta, lift):
    c, sn = math.cos(beta), math.sin(beta)
    return Vector((p.x * c - p.y * sn, p.x * sn + p.y * c + lift, p.z))


def _unpose_z(p, beta, lift):
    c, sn = math.cos(beta), math.sin(beta)
    x, y = p.x, p.y - lift
    return Vector((x * c + y * sn, -x * sn + y * c, p.z))


_HORN_LIE = {}


def horn_lie(trod):
    """(beta, lift): turned about z till the bell's rim and the mouthpiece's
    lip touch the floor together (it rests on both), lifted onto it."""
    if trod in _HORN_LIE:
        return _HORN_LIE[trod]
    cone = party_horn_cone(trod)
    mouth = party_horn_mouth()
    bell = [p for p in cone.v if p.y < 0.01]
    lip = [p for p in mouth.v]

    def gap(beta):
        b = min(_pose_z(p, beta, 0).y for p in bell)
        m = min(_pose_z(p, beta, 0).y for p in lip)
        return b - m

    lo, hi = math.pi / 2 - 0.2, math.pi / 2 + 0.4
    for _ in range(60):
        mid = (lo + hi) / 2
        if gap(mid) > 0:
            hi = mid
        else:
            lo = mid
    beta = (lo + hi) / 2
    lift = -min(_pose_z(p, beta, 0).y for p in cone.v + mouth.v)
    _HORN_LIE[trod] = (beta, lift)
    print('t12: HORN_LIE%s %.6f HORN_LIFT%s %.6f' % ('_TROD' if trod else '', beta, '_TROD' if trod else '', lift))
    return beta, lift


def party_horn_fringe(trod=False):
    """The fringe as it lies: the strands under the bell pressed flat."""
    beta, lift = horn_lie(trod)
    s = _horn_fringe_raw(trod)
    out = s.copy()
    for i, p in enumerate(out.v):
        q = _pose_z(p, beta, lift)
        if q.y < 0.0004:
            q.y = 0.0002 + 0.00015 * n1(q.x * 400, q.z * 400)
        out.v[i] = _unpose_z(q, beta, lift)
    return out


CYL_FRAMED |= {'Real__partyHornCone', 'Real__partyHornConeTrod'}
auto('Real__partyHornCone', party_horn_cone, 1500, smooth=40.0)
auto('Real__partyHornConeTrod', lambda: party_horn_cone(True), 1500, smooth=40.0)
auto('Real__partyHornMouth', party_horn_mouth, 420, smooth=40.0)
auto('Real__partyHornFringe', party_horn_fringe, 800, smooth=60.0)
auto('Real__partyHornFringeTrod', lambda: party_horn_fringe(True), 800, smooth=60.0)
