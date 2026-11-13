"""t12: what a party room leaves, years on (docs/design/PROPS.md P3, "the party and
the shrine"): cheap paper, card, foil and crepe, faded, crumpled and dusty.
Sources and sizes: t12.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is in its
own frame, given with it: x across, y up, z out of its front; a floor
thing's floor is y = 0, a ceiling thing's ceiling is y = 0 (it hangs below).
Cloth (the streamers, the tassels' strands, the piñata's tails) is draped
by Blender's own solver, headless (`cloth_drape`, as r13b's apron).

partyHat{Cone,Tassel,Elastic}{,Lying}
                      the printed card party hat (175 x 115 mm), its foil
                      pompom and its round elastic, standing or lying
partyHorn{Cone,Mouth,Fringe}{,Trod}
                      the 9 in foil party horn, whole or trodden flat
paperCrown, paperCrownOpen
                      the die-cut party crown, fastened and trodden, or
                      come open in a curl
streamer{A,B,C,D}     crepe streamer lengths hanging from their tape
garlandTassel{A,B,C}, garlandTwine{S,M,L}
                      the tissue tassel garland: its tassels, its twine
pinata{FringeBody,FringeCones,Tails,TailsB,Broken,Inside,CordS,CordM,
       CordL,Hook}    the star piñata, split open, on its cord and hook
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
HAT_LIE = math.pi / 2 + HAT_A                    # a hat lying: turned so about z (built so)


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
    """Laid down as the hat lies (HAT_LIE, HAT_LIFT), what would go under
    the floor pressed flat on it."""
    lift = hat_lift()
    out = s.copy()
    for i, p in enumerate(out.v):
        q = _hat_pose(p, lift)
        if q.y < 0.0004:
            q.y = 0.0002 + 0.0002 * n1(q.x * 300, q.z * 300)
        out.v[i] = q
    return out


def partyhat_cone_lying():
    """The cone lying on its side, as the game would have turned it (built
    so: a mesh turned off its axes is boxed by its corners, and the
    placement pass would lift it off the floor by them)."""
    lift = hat_lift()
    c = partyhat_cone()
    return Shape([_hat_pose(p, lift) for p in c.v], c.f)


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
    k = _knots()
    return s + Shape([_hat_pose(p, lift) for p in k.v], k.f)


auto('Real__partyHatCone', partyhat_cone, 2400, smooth=40.0)
auto('Real__partyHatTassel', partyhat_tassel, 800, smooth=60.0)
auto('Real__partyHatConeLying', partyhat_cone_lying, 2400, smooth=40.0)
auto('Real__partyHatTasselLying', lambda: partyhat_tassel(True), 800, smooth=60.0)
auto('Real__partyHatElastic', partyhat_elastic, 700, smooth=60.0)
auto('Real__partyHatElasticLying', partyhat_elastic_lying, 700, smooth=60.0)


# ============================================================ the party horn ==
# A 9 in (229 mm) party horn: a cone of card covered in printed foil, a
# white plastic mouthpiece with its reed pushed into the narrow end, a
# foil fringe glued round the bell. Built standing on its bell (y up its
# axis, the bell's rim at y = 0), then laid down as it lies (HORN_LIE*: its
# turn about z, and the lift that puts it on the floor, printed as it
# builds): every part of it is built lying (a mesh turned off its axes is
# boxed by its corners, and the placement pass lifts it by them).
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
        out.v[i] = q
    return out


def party_horn_lying(build, trod):
    """A part of the horn as it lies (built lying, not turned by the game,
    so its box is the floor's way up)."""
    beta, lift = horn_lie(trod)
    sh = build()
    return Shape([_pose_z(p, beta, lift) for p in sh.v], sh.f)


auto('Real__partyHornCone', lambda: party_horn_lying(party_horn_cone, False), 1500, smooth=40.0)
auto('Real__partyHornConeTrod', lambda: party_horn_lying(lambda: party_horn_cone(True), True), 1500, smooth=40.0)
auto('Real__partyHornMouth', lambda: party_horn_lying(party_horn_mouth, False), 420, smooth=40.0)
auto('Real__partyHornMouthTrod', lambda: party_horn_lying(party_horn_mouth, True), 420, smooth=40.0)
auto('Real__partyHornFringe', party_horn_fringe, 800, smooth=60.0)
auto('Real__partyHornFringeTrod', lambda: party_horn_fringe(True), 800, smooth=60.0)


# =========================================================== the paper crown ==
# A party crown die-cut from foil card (or printed card): 3.5 in (89 mm)
# tall, a 30 mm band under eight points, 575 mm long with its tab; fastened
# round by its tab through a slot (530 mm round, a head's size). Two kinds:
#   paperCrown       fastened, set down on its band's edge and trodden: the
#                    ring pressed oval, a crease across the band, three
#                    points bent over at their roots (out, in, out)
#   paperCrownOpen   its tab come out of the slot: the card sprung open in a
#                    loose curl standing on its edge, a point bent
# Each is in its own frame: the ring's (or the curl's) middle over the
# origin, y up from the floor. The game lays its print on by where each
# vertex is round the ring (CR_* in the placer).
CR_C, CR_OV = 0.530, 0.045                     # round, the tab's overlap
CR_HB, CR_HP, CR_T = 0.030, 0.059, 0.0006      # band, points, card
CR_NP = 8
CR_PITCH = (CR_C - CR_OV) / CR_NP
CR_VF = CR_HB + 0.004                          # where a bent point is folded
CR_AX, CR_AZ = 0.094, 0.075                    # the ring pressed oval: half axes
CR_OPEN_R = 0.130                              # the open one's curl
CR_FOLDS = {'ring': {1: 0.62, 4: -0.42, 6: 0.30}, 'open': {2: 0.55, 5: -0.25}}


def _cr_top(u, length):
    """The card's top edge at u along it: the band's top, or a point's."""
    if u <= CR_OV + 1e-9 or u >= CR_C - 1e-9:
        return CR_HB if u < length - CR_OV + 1e-9 or length < CR_C + 1e-9 else CR_HB * 0.80
    f = ((u - CR_OV) / CR_PITCH) % 1.0
    t = 1.0 - abs(2 * f - 1)
    # the point's tip rounded a little
    return CR_HB + CR_HP * (t - 0.06 * max(0.0, t - 0.92) / 0.08)


def _cr_columns(length):
    us = [0.0, CR_OV * 0.5, CR_OV]
    for k in range(CR_NP):
        for f in (0.25, 0.46, 0.5, 0.54, 0.75, 1.0):
            us.append(CR_OV + (k + f) * CR_PITCH)
    if length > CR_C + 1e-9:
        us += [CR_C + CR_OV * 0.33, CR_C + CR_OV * 0.66, CR_C + CR_OV]
    return us


def _cr_card(length):
    """The card's front in (u, v): vertices and faces (a band of quads, the
    points fanned from their roots)."""
    us = _cr_columns(length)
    V, F = [], []
    cols = []
    for u in us:
        top = _cr_top(u, length)
        ids = []
        for j in range(4):
            V.append((u, CR_HB * j / 3 if top >= CR_HB - 1e-9 else top * j / 3))
            ids.append(len(V) - 1)
        if top > CR_HB + 1e-6:
            for j in (1, 2, 3):
                V.append((u, CR_HB + (top - CR_HB) * j / 3))
                ids.append(len(V) - 1)
        cols.append(ids)
    for a, b in zip(cols, cols[1:]):
        for j in range(3):
            F.append((a[j], b[j], b[j + 1], a[j + 1]))
        if len(a) > 4 and len(b) > 4:
            for j in range(3, 6):
                F.append((a[j], b[j], b[j + 1], a[j + 1]))
        elif len(b) > 4:
            for j in range(3, 6):
                F.append((a[3], b[j], b[j + 1]))
        elif len(a) > 4:
            for j in range(3, 6):
                F.append((a[j], b[3], a[j + 1]))
    return V, F


def _cr_local(u, v, w, folds, length):
    """Folded points, the card's waviness and the crease: (v, w) after them."""
    if v > CR_VF and CR_OV < u < CR_C:
        k = int((u - CR_OV) / CR_PITCH)
        a = folds.get(k, 0.0)
        if a:
            dv = v - CR_VF
            v, w = CR_VF + dv * math.cos(a) - w * math.sin(a), dv * math.sin(a) + w * math.cos(a)
    w += 0.0012 * n1(u * 22.0, v * 22.0, 0.7) + 0.0008 * n1(u * 61.0, v * 61.0, 2.1)
    return v, w


def _cr_ring_place(u, v, w):
    th = TAU * u / CR_C
    e = Vector((CR_AX * math.sin(th), v, CR_AZ * math.cos(th)))
    nrm = Vector((CR_AZ * math.sin(th), 0.0, CR_AX * math.cos(th))).normalized()
    # the tab lies over the band's start, outside it
    w += CR_T * 1.6 * smoothstep(CR_C - 0.004, CR_C, u)
    # trodden: a crease pressed in across the band
    kink = 0.0055 * max(0.0, 1.0 - abs(u - 0.205) / 0.016) * (1.0 - 0.5 * v / (CR_HB + CR_HP))
    return e + nrm * (w - kink)


def _cr_open_place(u, v, w):
    length = CR_C + CR_OV
    span = length / CR_OPEN_R
    th0 = math.pi + (TAU - span) / 2
    th = th0 + u / CR_OPEN_R
    # it springs open more at its free ends
    r = CR_OPEN_R * (1.0 + 0.10 * (abs(2 * u / length - 1) ** 3))
    nrm = Vector((math.sin(th), 0.0, math.cos(th)))
    return Vector((r * math.sin(th), v, r * math.cos(th))) + nrm * w


def paper_crown(kind):
    length = CR_C + CR_OV
    folds = CR_FOLDS[kind]
    place = _cr_ring_place if kind == 'ring' else _cr_open_place
    V, F = _cr_card(length)
    nv = len(V)
    pts = []
    for layer, w0 in ((0, CR_T / 2), (1, -CR_T / 2)):
        for (u, v) in V:
            vv, ww = _cr_local(u, v, w0, folds, length)
            pts.append(place(u, vv, ww))
    faces = list(F) + [tuple(reversed([i + nv for i in f])) for f in F]
    # the card's cut edge: stitch the outline of front and back
    count = {}
    for f in F:
        for k in range(len(f)):
            a, b = f[k], f[(k + 1) % len(f)]
            count[(a, b)] = count.get((a, b), 0) + 1
    for (a, b), c in count.items():
        if c == 1 and (b, a) not in count:
            faces.append((b, a, a + nv, b + nv))
    s = Shape(pts, faces)
    s = propkit._fix_winding(s)
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


auto('Real__paperCrown', lambda: paper_crown('ring'), 1600, smooth=50.0)
auto('Real__paperCrownOpen', lambda: paper_crown('open'), 1600, smooth=50.0)


# ================================================================ the cloth ==
def cloth_drape(name, pts, faces, pins, colliders=(), frames=140, mass=0.01, tension=15.0, shear=5.0,
                bending=0.03, air=4.0, quality=6, self_col=False, self_dist=0.0015, col_dist=0.0015,
                friction=5.0, gravity=-9.81):
    """Blender's cloth solver, headless (as batch r13b's apron): the sheet
    `pts`/`faces` let fall under gravity (-y) from where it is, `pins` held,
    against `colliders` (Shapes). Returns the settled points."""
    t0 = time.time()
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(p) for p in pts], [], [tuple(f) for f in faces])
    me.update()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    if pins:
        g = ob.vertex_groups.new(name='pin')
        g.add(sorted(set(pins)), 1.0, 'REPLACE')
    cobs = []
    for k, cs in enumerate(colliders):
        cm = bpy.data.meshes.new(f'{name}_col{k}')
        cm.from_pydata([tuple(p) for p in cs.v], [], [tuple(f) for f in cs.f])
        cm.update()
        co = bpy.data.objects.new(cm.name, cm)
        bpy.context.scene.collection.objects.link(co)
        co.modifiers.new('col', 'COLLISION')
        co.collision.thickness_outer = 0.0015
        co.collision.cloth_friction = friction
        cobs.append((co, cm))
    sc = bpy.context.scene
    grav = tuple(sc.gravity)
    sc.gravity = (0.0, gravity, 0.0)
    cl = ob.modifiers.new('cloth', 'CLOTH')
    s = cl.settings
    s.quality = quality
    s.mass = mass
    s.tension_stiffness = tension
    s.compression_stiffness = tension
    s.shear_stiffness = shear
    s.bending_stiffness = bending
    s.air_damping = air
    if pins:
        s.vertex_group_mass = 'pin'
    c = cl.collision_settings
    c.use_collision = bool(colliders)
    c.distance_min = col_dist
    c.use_self_collision = self_col
    c.self_distance_min = self_dist
    c.collision_quality = 3
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
    for co, cm in cobs:
        bpy.data.objects.remove(co)
        bpy.data.meshes.remove(cm)
    print('t12: %s draped in %.1f s' % (name, time.time() - t0))
    return out


def ceiling_plane(size=3.0, y=0.0):
    return Shape([(-size, y, -size), (size, y, -size), (size, y, size), (-size, y, size)], [(0, 1, 2, 3)])


def _vnormals(P, faces):
    N = [Vector((0, 0, 0)) for _ in P]
    for f in faces:
        a, b, c = P[f[0]], P[f[1]], P[f[2]]
        n = (b - a).cross(c - a)
        if len(f) == 4:
            n += (c - a).cross(P[f[3]] - a)
        for i in f:
            N[i] += n
    return [n.normalized() if n.length > 1e-12 else Vector((0, 0, 1)) for n in N]


# ========================================================= the streamers ==
# Crepe paper streamer (1.75 in, 44.5 mm wide, off an 81 ft roll), twisted
# as it was hung and taped to the ceiling; the swag it was part of torn
# long ago, so each length hangs from its tape, the crepe stretched and
# sagged, its end torn. Draped by Blender's cloth: the taped end held flat
# against the ceiling, the rest let fall from where the swag ran.
# Each in its own frame: the ceiling at y = 0, the tape's middle over the
# origin, the length hanging below it. (The game tapes it with r06's
# masking tape, Real__posterTape.)
STR_W = 0.0445
STR_KINDS = {'A': (1.62, 2.6, 0.4, 12.0), 'B': (1.36, 1.6, 1.3, -20.0), 'C': (1.12, 0.6, 2.2, 5.0), 'D': (0.92, 2.0, 3.1, -8.0)}   # length, turns, seed, heading (deg)
STR_STEP = 0.016
STR_TAPE = 0.034                               # the length held under the tape
_STR = {}


def streamer_sim(kind):
    if kind in _STR:
        return _STR[kind]
    L, turns, seed, head = STR_KINDS[kind]
    rng = random.Random(int(seed * 1000))
    nu, nv = 3, int(round(L / STR_STEP))
    head = math.radians(head)
    run = Vector((math.cos(head), 0, math.sin(head)))            # the way the swag ran
    side0 = Vector((-run.z, 0, run.x))
    # the torn end: ragged across, a little on the slant
    tear = [rng.uniform(-0.012, 0.012) + (i / nu - 0.5) * rng.uniform(-0.03, 0.03) for i in range(nu + 1)]
    pts, faces, pins = [], [], []
    drop = math.radians(rng.uniform(78, 88))
    bend = 0.05                                    # from the tape's edge it turns down over this
    for j in range(nv + 1):
        s = j * L / nv
        for i in range(nu + 1):
            a = (i / nu - 0.5) * STR_W
            if j == nv:
                s = L + tear[i]
            if s <= STR_TAPE:
                p = run * (s - STR_TAPE / 2) + side0 * a + Vector((0, -0.0004, 0))
                pins.append(len(pts))
            else:
                t = s - STR_TAPE
                # round a quarter bend from the ceiling, then straight down
                # (a little off true, the way the swag pulled it)
                if t < bend:
                    ang = drop * t / bend
                    rr = bend / max(drop, 1e-6)
                    c = run * (STR_TAPE / 2 + rr * math.sin(ang)) + Vector((0, -0.0007 - rr * (1 - math.cos(ang)), 0))
                    d = run * math.cos(ang) + Vector((0, -math.sin(ang), 0))
                else:
                    rr = bend / max(drop, 1e-6)
                    c0 = run * (STR_TAPE / 2 + rr * math.sin(drop)) + Vector((0, -0.0007 - rr * (1 - math.cos(drop)), 0))
                    d = run * math.cos(drop) + Vector((0, -math.sin(drop), 0))
                    c = c0 + d * (t - bend)
                d = d.normalized()
                # twisted about its own line as it was hung (the paper set so)
                ph = TAU * turns * (t / L) + 0.35 * n1(t * 3.0, seed)
                ph *= smoothstep(0.0, 0.10, t)
                sd = (side0 * math.cos(ph) + d.cross(side0).normalized() * math.sin(ph))
                p = c + sd * a
            pts.append(p)
    for j in range(nv):
        for i in range(nu):
            k = j * (nu + 1) + i
            faces.append((k, k + 1, k + nu + 2, k + nu + 1))
    out = cloth_drape('streamer' + kind, pts, faces, pins, colliders=[ceiling_plane()], frames=150, mass=0.004,
                      tension=20.0, shear=4.0, bending=0.015, air=2.5)
    _STR[kind] = (out, faces, nu, nv)
    return _STR[kind]


def streamer(kind):
    P, faces, nu, nv = streamer_sim(kind)
    seed = STR_KINDS[kind][2]
    P = [p.copy() for p in P]
    N = _vnormals(P, faces)
    for j in range(nv + 1):
        for i in range(nu + 1):
            k = j * (nu + 1) + i
            if P[k].y > -0.002:
                continue
            # the crepe's crinkle, and its edges gone frilled where it stretched
            e = abs(i / nu - 0.5) * 2
            w = 0.0011 * n1(j * 0.9, i * 0.7, seed) + 0.0016 * e * math.sin(j * 1.9 + seed * 3)
            P[k] = P[k] + N[k] * w
    s = Shape(P, faces)
    return s


for _k in 'ABCD':
    auto(f'Real__streamer{_k}', lambda k=_k: streamer(k), 900, smooth=70.0)


# ======================================================= the tassel garland ==
# A tissue paper tassel garland: tassels each a sheet of tissue (20 x 26 in)
# folded, fringed and rolled, its middle twisted into a loop that the jute
# twine (2 mm) runs through; the twine swagged between two pieces of tape
# on the ceiling. Years on the tissue has faded and gone limp, the strands
# hang in clumps, some torn short.
#   garlandTassel{A,B,C}   a tassel: its loop round the twine (the twine along
#                          x through the origin), the twisted neck, the strands
#                          draped by Blender's cloth (A, B whole; C torn)
#   garlandTwine{S,M,L}    the twine for a span of 1.55, 1.90, 2.25 m: a
#                          catenary from tape to tape (the ceiling y = 0, the
#                          span's middle over the origin), its ends laid flat
#                          under the tape and a cut tail beyond
GT_KINDS = {'A': (40, 0.235, 5.1, 0), 'B': (36, 0.255, 6.3, 0.08), 'C': (26, 0.215, 7.7, 0.35)}   # strands, length, seed, torn share
GT_LOOP_R = 0.011
GT_NECK = (-0.024, -0.042)
GT_SPANS = {'S': 1.55, 'M': 1.90, 'L': 2.25}
GT_DROP = 0.004                                # the twine's ends, under the ceiling


def garland_sag(span):
    return 0.12 * span + 0.02


def garland_cat(span):
    """The catenary's parameter for a span and its sag."""
    d = garland_sag(span)
    lo, hi = 0.05, 50.0
    for _ in range(80):
        a = (lo + hi) / 2
        if a * (math.cosh(span / (2 * a)) - 1) > d:
            lo = a
        else:
            hi = a
    return (lo + hi) / 2


def garland_y(span, x):
    a = garland_cat(span)
    return a * (math.cosh(x / a) - math.cosh(span / (2 * a))) - GT_DROP


_GT = {}


def tassel_sim(kind):
    if kind in _GT:
        return _GT[kind]
    n, L, seed, torn = GT_KINDS[kind]
    rng = random.Random(int(seed * 1000))
    rows = 11
    pts, faces, pins, strands = [], [], [], []
    for k in range(n):
        a = TAU * (k + rng.uniform(-0.3, 0.3)) / n
        rad = Vector((math.cos(a), 0, math.sin(a)))
        side = Vector((-math.sin(a), 0, math.cos(a)))
        root = Vector((0, GT_NECK[1], 0)) + rad * rng.uniform(0.003, 0.007)
        # torn short, some of them
        Lk = L * (rng.uniform(0.35, 0.8) if rng.random() < torn else rng.uniform(0.9, 1.05))
        lean = math.radians(rng.uniform(6, 30))
        d = (Vector((0, -1, 0)) * math.cos(lean) + rad * math.sin(lean)).normalized()
        wk = rng.uniform(0.0045, 0.0065)
        twist = rng.uniform(-1.2, 1.2)
        base = len(pts)
        for j in range(rows + 1):
            t = j / rows
            c = root + d * (Lk * t) + Vector((rng.uniform(-1, 1), 0, rng.uniform(-1, 1))) * 0.002 * t
            sd = (side * math.cos(twist * t) + rad * math.sin(twist * t)).normalized()
            for e in (-0.5, 0.5):
                pts.append(c + sd * (e * wk * (1.0 - 0.15 * t)))
            if j == 0:
                pins += [len(pts) - 2, len(pts) - 1]
        for j in range(rows):
            q = base + j * 2
            faces.append((q, q + 1, q + 3, q + 2))
        strands.append((base, rows))
    # the neck and loop as something the strands lie against
    neck = cyl(0.008, 0.006, GT_NECK[0] - GT_NECK[1], seg=10).move(0, (GT_NECK[0] + GT_NECK[1]) / 2, 0)
    out = cloth_drape('tassel' + kind, pts, faces, pins, colliders=[neck], frames=110, mass=0.003,
                      tension=15.0, shear=3.0, bending=0.01, air=3.0, self_col=True, self_dist=0.0012)
    _GT[kind] = (out, faces)
    return _GT[kind]


def _gt_loop(rng):
    """The tissue's middle twisted into a rope and looped over the twine,
    its ends twisted together into the neck."""
    s = Shape()
    # the loop: a twisted rope ring in the y-z plane, its top on the twine
    m = 26
    ring = []
    for k in range(m + 1):
        a = -math.pi / 2 + TAU * k / m
        ring.append(Vector((0.0015 * math.sin(a * 3), -GT_LOOP_R + GT_LOOP_R * math.sin(a) + 0.0016, GT_LOOP_R * math.cos(a) * 0.85)))
    rr = [0.0026 + 0.0006 * math.sin(k * 2.3) for k in range(m + 1)]
    s = s + sweep(ring, rr, seg=7, cap0=False, cap1=False)
    # the neck: twisted, a few turns showing as ridges
    prof = []
    for k in range(9):
        t = k / 8
        y = GT_NECK[0] + (GT_NECK[1] - GT_NECK[0]) * t + 0.004
        prof.append((0.0062 - 0.0012 * t + 0.0009 * math.sin(t * 20), y))
    s = s + lathe([(0.0, prof[-1][1] - 0.001)] + list(reversed(prof)) + [(0.0, prof[0][1] + 0.002)], seg=9).warp(
        lambda p: Vector((p.x + 0.0008 * math.sin(p.y * 300), p.y, p.z)))
    return s


def garland_tassel(kind):
    P, faces = tassel_sim(kind)
    rng = random.Random(kind)
    P = [p.copy() for p in P]
    N = _vnormals(P, faces)
    for i, p in enumerate(P):
        # tissue's crinkle
        P[i] = p + N[i] * 0.0008 * n1(p.x * 90, p.y * 90, p.z * 90)
    return Shape(P, faces) + _gt_loop(rng)


def garland_twine(span):
    a = garland_cat(span)
    pts = []
    n = int(span / 0.045)
    # the end laid under the tape, then out and down
    for k in range(4):
        pts.append(Vector((-span / 2 - 0.035 + k * 0.009, -0.0015, 0.0)))
    for k in range(n + 1):
        x = -span / 2 + span * k / n
        y = garland_y(span, x) if 0 < k < n else -0.0015
        pts.append(Vector((x, y, 0.0012 * n1(x * 9, 0.3))))
    for k in range(1, 4):
        pts.append(Vector((span / 2 + k * 0.009, -0.0015, 0.0)))
    # a cut tail beyond the far tape, hanging
    tail = pts[-1]
    for k in range(1, 6):
        pts.append(tail + Vector((0.006 + 0.002 * k, -0.012 * k, 0.003 * math.sin(k))))
    rr = [0.0014] * len(pts)
    return sweep(pts, rr, seg=5)


for _k in 'ABC':
    auto(f'Real__garlandTassel{_k}', lambda k=_k: garland_tassel(k), 1500, smooth=70.0)
for _k, _sp in GT_SPANS.items():
    auto(f'Real__garlandTwine{_k}', lambda sp=_sp: garland_twine(sp), 700, smooth=60.0)
    print('t12: GT span %.2f catenary a %.6f sag %.4f' % (_sp, garland_cat(_sp), garland_sag(_sp)))


# ============================================================== the piñata ==
# A small star piñata (a 13 in star, as sold): a ball of papier-mâché, 170 mm,
# seven cones of card (four up, three down), all of it under rows of
# fringed tissue glued on in shingles, tissue streamers at the cones' tips;
# hung by a cotton cord from a cup hook in the ceiling. Split open on one
# side where it was hit (the shell's torn edge, its dark inside), one cone
# crushed; the tissue faded, the strands lifted and torn.
# The body's frame: the cord's tie on top of the ball at the origin, the
# ball below it; the cord's and hook's frame: the ceiling at y = 0, the hook
# over the origin, the cord down to its tie (PN_CORDS).
#   pinataShell        the shell under the fringe (in the body's tissue), the
#                      cones' card under theirs is pinataCones
#   pinataFringeBody   the ball's fringe, its strands hanging down it
#   pinataFringeCones  the cones' card and fringe (strands towards the base)
#   pinataTails{,B}    the tissue streamers from the tips, draped by cloth (in
#                      the ball's tissue, and in the cones')
#   pinataBroken       the split: the papier-mâché's torn edge (newsprint)
#   pinataInside       its inside, dark
#   pinataCord{S,M,L}  the cord, 0.45, 0.60, 0.75 m, over the hook
#   pinataHook         a brass cup hook screwed into the ceiling
PN_R = 0.085
PN_C = Vector((0.0, -0.010 - PN_R, 0.0))          # the ball's middle, under the tie
PN_CONE_L, PN_CONE_R = 0.118, 0.031
PN_CONES = [(45, 30), (135, 30), (225, 30), (315, 30), (0, -42), (120, -42), (240, -42)]   # azimuth, elevation (deg)
PN_HOLE = (75.0, -12.0, 0.46)                     # the split: azimuth, elevation, angular radius (rad)
PN_CRUSH = 5                                      # the cone crushed
PN_CORDS = {'S': 0.45, 'M': 0.60, 'L': 0.75}
PN_STRAND_W = 0.010


def _pn_dir(az, el):
    a, e = math.radians(az), math.radians(el)
    return Vector((math.cos(e) * math.sin(a), math.sin(e), math.cos(e) * math.cos(a)))


def _pn_hole_dir():
    return _pn_dir(PN_HOLE[0], PN_HOLE[1])


def _pn_in_hole(d, margin=0.0):
    """Is the ball's direction d inside the split (its edge ragged)?"""
    h = _pn_hole_dir()
    ang = math.acos(max(-1.0, min(1.0, d.dot(h))))
    # the ragged edge: a function of the angle round the hole's middle
    ref = h.cross(Vector((0, 1, 0))).normalized()
    q = (d - h * d.dot(h))
    phi = math.atan2(q.dot(h.cross(ref)), q.dot(ref)) if q.length > 1e-9 else 0.0
    edge = PN_HOLE[2] * (1.0 + 0.22 * math.sin(3 * phi + 0.5) + 0.12 * math.sin(7 * phi + 1.3) + 0.08 * math.sin(13 * phi))
    return ang < edge + margin


def _pn_cone_frame(k):
    az, el = PN_CONES[k]
    d = _pn_dir(az, el)
    u = d.cross(Vector((0, 1, 0)))
    if u.length < 1e-6:
        u = Vector((1, 0, 0))
    u.normalize()
    w = d.cross(u).normalized()
    base = PN_C + d * (PN_R - 0.014)
    return d, u, w, base


def _pn_cone_point(k, s, phi, off=0.0):
    """Cone k's surface at s (0 base .. 1 tip) and angle phi round it."""
    d, u, w, base = _pn_cone_frame(k)
    r = PN_CONE_R * (1.0 - s)
    rad = u * math.cos(phi) + w * math.sin(phi)
    a = math.atan2(PN_CONE_R, PN_CONE_L)
    n = (rad * math.cos(a) + d * math.sin(a)).normalized()
    p = base + d * (PN_CONE_L * s) + rad * r + n * off
    if k == PN_CRUSH:
        # crushed: its tip bent over and its side pressed in
        if s > 0.45:
            ang = 0.55 * smoothstep(0.45, 1.0, s)
            p = rot_about(p, base + d * (PN_CONE_L * 0.45), w, ang)
        p -= u * (0.006 * max(0.0, math.cos(phi)) * math.sin(math.pi * min(1.0, s / 0.8)))
    return p, n, d


def _pn_ball_ok(d):
    """Is the ball's surface at direction d out from under a cone?"""
    for k in range(len(PN_CONES)):
        cd = _pn_dir(*PN_CONES[k])
        if math.acos(max(-1.0, min(1.0, d.dot(cd)))) < 0.31:
            return False
    return True


def _pn_strand(rng, root, along, normal, length, width, lift):
    side = along.cross(normal).normalized()
    pts, sides, ws = [], [], []
    tw = rng.uniform(-0.5, 0.5)
    for k in range(3):
        t = k / 2
        lf = lift * t * t + 0.0012
        p = root + along * (length * t) + normal * lf
        sd = (side * math.cos(tw * t) + normal * math.sin(tw * t)).normalized()
        pts.append(p)
        sides.append(sd)
        ws.append(width * (1.0 - 0.18 * t))
    return strip(pts, sides, ws)


def pinata_shell():
    """The ball's skin under its fringe (in the tissue's colour), the split
    cut out of it."""
    nu, nv = 32, 16
    v, f = [], []
    idx = {}
    for j in range(nv + 1):
        th = math.pi * j / nv
        for i in range(nu):
            ph = TAU * i / nu
            d = Vector((math.sin(th) * math.sin(ph), -math.cos(th), math.sin(th) * math.cos(ph)))
            idx[(j, i)] = len(v)
            v.append(PN_C + d * (PN_R + 0.0005 * n1(d.x * 9, d.y * 9, d.z * 9)))
    for j in range(nv):
        for i in range(nu):
            q = [idx[(j, i)], idx[(j, (i + 1) % nu)], idx[(j + 1, (i + 1) % nu)], idx[(j + 1, i)]]
            c = sum((v[x] - PN_C for x in q), Vector()) / 4
            if _pn_in_hole(c.normalized(), -0.02):
                continue
            f.append(tuple(q))
    s = weld(Shape(v, f), 1e-6)
    return s


def pinata_fringe_body():
    rng = random.Random(901)
    s = Shape()
    rows = 15
    for j in range(rows):
        th = 0.20 + (math.pi - 0.42) * j / (rows - 1)           # the strands' roots (from the bottom)
        count = max(5, int(round(TAU * PN_R * math.sin(th) / (PN_STRAND_W * 1.05))))
        off = rng.uniform(0, TAU)
        for i in range(count):
            ph = off + TAU * (i + rng.uniform(-0.15, 0.15)) / count
            d = Vector((math.sin(th) * math.sin(ph), -math.cos(th), math.sin(th) * math.cos(ph)))
            if not _pn_ball_ok(d) or _pn_in_hole(d, 0.05):
                continue
            if rng.random() < 0.06:
                continue                                          # torn off
            root = PN_C + d * (PN_R + 0.001)
            # down the ball: along the meridian towards its bottom
            along = Vector((-math.cos(th) * math.sin(ph), -math.sin(th), -math.cos(th) * math.cos(ph))) * -1
            along = (along - d * along.dot(d)).normalized()
            if along.y > 0:
                along = -along
            L = rng.uniform(0.024, 0.033) * (0.5 if rng.random() < 0.08 else 1.0)
            s = s + _pn_strand(rng, root, along, d, L, PN_STRAND_W, rng.uniform(0.002, 0.007))
    return s + pinata_shell()


def pinata_fringe_cones():
    rng = random.Random(902)
    s = Shape()
    for k in range(len(PN_CONES)):
        # the cone's card (under the fringe; the tissue glued round it)
        ns, nphi = 6, 12
        v, f = [], []
        for j in range(ns + 1):
            for i in range(nphi):
                p, n, d = _pn_cone_point(k, 0.97 * j / ns, TAU * i / nphi)
                v.append(p)
        for j in range(ns):
            for i in range(nphi):
                i2 = (i + 1) % nphi
                f.append((j * nphi + i, j * nphi + i2, (j + 1) * nphi + i2, (j + 1) * nphi + i))
        tip, _, d = _pn_cone_point(k, 1.0, 0.0)
        v.append(tip)
        for i in range(nphi):
            f.append((ns * nphi + i, ns * nphi + (i + 1) % nphi, len(v) - 1))
        s = s + Shape(v, f)
        # the fringe, row on row from the tip back to the base, the strands
        # lying towards the base
        for j, sj in enumerate((0.88, 0.76, 0.64, 0.52, 0.40, 0.28, 0.16)):
            r = PN_CONE_R * (1.0 - sj)
            count = max(4, int(round(TAU * r / (PN_STRAND_W * 1.0))))
            off = rng.uniform(0, TAU)
            for i in range(count):
                ph = off + TAU * (i + rng.uniform(-0.15, 0.15)) / count
                if rng.random() < 0.05:
                    continue
                root, n, d = _pn_cone_point(k, sj, ph, 0.001)
                back, _, _ = _pn_cone_point(k, max(0.0, sj - 0.1), ph, 0.001)
                along = (back - root).normalized()
                L = rng.uniform(0.022, 0.03)
                s = s + _pn_strand(rng, root, along, n, L, PN_STRAND_W * 0.95, rng.uniform(0.003, 0.009))
    return s


def _pn_body_collider():
    s = ellipsoid(PN_R + 0.006, PN_R + 0.006, PN_R + 0.006, seg=20).move(*PN_C)
    for k in range(len(PN_CONES)):
        d, u, w, base = _pn_cone_frame(k)
        c = cone(PN_CONE_R + 0.006, PN_CONE_L, seg=10)
        c.move(0, PN_CONE_L / 2, 0)
        m = Matrix((u, d, w)).transposed().to_4x4()
        c.apply(m)
        c.move(*base)
        s = s + c
    return s


_PN_TAILS = None


def pinata_tails_sim():
    global _PN_TAILS
    if _PN_TAILS is not None:
        return _PN_TAILS
    rng = random.Random(903)
    pts, faces, pins = [], [], []
    owner = []
    for k in range(len(PN_CONES)):
        tip, n, d = _pn_cone_point(k, 0.99, 0.0)
        cnt = 4 if k != PN_CRUSH else 2
        for t in range(cnt):
            L = rng.uniform(0.13, 0.26) * (0.55 if rng.random() < 0.2 else 1.0)
            rows = max(4, int(L / 0.013))
            wdt = rng.uniform(0.011, 0.016)
            a = TAU * t / cnt + rng.uniform(-0.4, 0.4)
            _, u, w, _ = _pn_cone_frame(k)
            sd = (u * math.cos(a) + w * math.sin(a)).normalized()
            # out of the tip and straight down (gravity does the rest)
            go = (Vector((0, -1, 0)) + d * 0.25 + sd * 0.15).normalized()
            across = go.cross(sd).normalized()
            if across.length < 1e-6:
                across = u
            base = len(pts)
            for j in range(rows + 1):
                c = tip + go * (L * j / rows)
                for e in (-0.5, 0.5):
                    pts.append(c + across * (e * wdt))
                if j == 0:
                    pins += [len(pts) - 2, len(pts) - 1]
            for j in range(rows):
                q = base + j * 2
                faces.append((q, q + 1, q + 3, q + 2))
                owner.append(t % 2)
    out = cloth_drape('pinataTails', pts, faces, pins, colliders=[_pn_body_collider()], frames=220, mass=0.004,
                      tension=15.0, shear=3.0, bending=0.01, air=1.5, self_col=False)
    _PN_TAILS = (out, faces, owner)
    return _PN_TAILS


def pinata_tails(which):
    """The streamers in the ball's tissue (0) or the cones' (1)."""
    P, faces, owner = pinata_tails_sim()
    P = [p.copy() for p in P]
    N = _vnormals(P, faces)
    for i, p in enumerate(P):
        P[i] = p + N[i] * 0.0012 * n1(p.x * 70, p.y * 70, p.z * 70)
    mine = [f for f, o in zip(faces, owner) if o == which]
    used = sorted({i for f in mine for i in f})
    remap = {old: new for new, old in enumerate(used)}
    return Shape([P[i] for i in used], [tuple(remap[i] for i in f) for f in mine])


def _pn_hole_ring(r):
    """The split's ragged edge on a sphere of radius r: a closed loop."""
    h = _pn_hole_dir()
    ref = h.cross(Vector((0, 1, 0))).normalized()
    ref2 = h.cross(ref).normalized()
    out = []
    for i in range(48):
        phi = TAU * i / 48
        edge = PN_HOLE[2] * (1.0 + 0.22 * math.sin(3 * phi + 0.5) + 0.12 * math.sin(7 * phi + 1.3) + 0.08 * math.sin(13 * phi))
        edge -= 0.02
        d = h * math.cos(edge) + (ref * math.cos(phi) + ref2 * math.sin(phi)) * math.sin(edge)
        out.append(PN_C + d.normalized() * r)
    return out


def pinata_broken():
    """The torn edge of the papier-mâché round the split (4 mm of layered
    newsprint), a little ragged in and out."""
    outer = _pn_hole_ring(PN_R + 0.0008)
    inner = _pn_hole_ring(PN_R - 0.004)
    n = len(outer)
    v = outer + inner
    f = []
    for i in range(n):
        i2 = (i + 1) % n
        f.append((i, i2, n + i2, n + i))
    s = Shape(v, f)
    return s.warp(lambda p: p + (p - PN_C).normalized() * 0.0012 * n1(p.x * 200, p.y * 200, p.z * 200))


def pinata_inside():
    """Its hollow (only the half the split shows), dark."""
    nu, nv = 20, 10
    v, f = [], []
    for j in range(nv + 1):
        th = math.pi * j / nv
        for i in range(nu):
            ph = TAU * i / nu
            d = Vector((math.sin(th) * math.sin(ph), -math.cos(th), math.sin(th) * math.cos(ph)))
            v.append(PN_C + d * (PN_R - 0.004))
    for j in range(nv):
        for i in range(nu):
            i2 = (i + 1) % nu
            f.append((j * nu + i, (j + 1) * nu + i, (j + 1) * nu + i2, j * nu + i2))     # facing in
    return weld(Shape(v, f), 1e-6)


def pinata_cord(L):
    """The cord: from its loop over the hook down to its tie on the ball,
    twisted cotton 4 mm."""
    pts = []
    hook = Vector((0.0, -0.022, 0.0))
    # the loop over the hook's bend
    for k in range(9):
        a = math.pi * k / 8
        pts.append(hook + Vector((0.0045 * math.cos(a), 0.0045 * math.sin(a) - 0.001, 0.0)))
    pts.reverse()
    knot = hook + Vector((0, -0.030, 0))
    pts.append(knot)
    n = int(L / 0.04)
    for k in range(1, n + 1):
        y = knot.y + (-L - knot.y) * k / n
        pts.append(Vector((0.0015 * math.sin(k * 1.7), y, 0.0015 * math.cos(k * 1.3))))
    s = sweep(pts, [0.002] * len(pts), seg=6)
    s = s + lump(knot, 0.0045, (1.0, 1.3, 1.0), seg=7)
    return s


def pinata_hook():
    """A brass cup hook screwed into the ceiling: its flange against the
    tile, the hook's open bend below."""
    flange = cyl(0.0055, 0.0055, 0.0015, seg=12).move(0, -0.00075, 0)
    pts = [Vector((0, -0.0015, 0))]
    for k in range(12):
        a = math.pi * k / 11
        pts.append(Vector((0.006 * math.sin(a), -0.016 - 0.006 * (1 - math.cos(a)), 0.0)))
    pts = [Vector((0, -0.0015, 0)), Vector((0, -0.010, 0))] + [Vector((-0.006 * math.cos(math.pi * k / 11) + 0.0, -0.016 - 0.006 * math.sin(math.pi * k / 11), 0.0)) for k in range(12)]
    pts.append(pts[-1] + Vector((0.0, 0.006, 0.0)))
    return flange + sweep(pts, [0.0016] * len(pts), seg=6)


auto('Real__pinataFringeBody', pinata_fringe_body, 3200, smooth=60.0)
auto('Real__pinataFringeCones', pinata_fringe_cones, 3600, smooth=60.0)
auto('Real__pinataTails', lambda: pinata_tails(0), 700, smooth=70.0)
auto('Real__pinataTailsB', lambda: pinata_tails(1), 700, smooth=70.0)
auto('Real__pinataBroken', pinata_broken, 300, smooth=40.0)
auto('Real__pinataInside', pinata_inside, 500, smooth=60.0)
for _k, _L in PN_CORDS.items():
    auto(f'Real__pinataCord{_k}', lambda L=_L: pinata_cord(L), 600, smooth=60.0)
auto('Real__pinataHook', pinata_hook, 360, smooth=40.0)
