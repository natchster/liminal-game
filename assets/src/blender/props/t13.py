"""t13: the props remodelled at their real sizes (docs/PROPS.md, P3, the
rare): a party table's leavings, years on (docs/PROPS.md 1, "the party and
the shrine"): the printed paper cups, the paper dessert plates, the pressed
glass cake stand and the cake nobody ate on it, the cornbread muffins in
their paper cases. Sources and sizes: t13.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame: x across, y up from the floor, z out of its front;
the round things about the y axis (angles from +z toward +x, as the game's
cylinder texture coordinates run).

paperCupOuter / Inner / Rim / Bottom   a 9 oz paper hot/cold cup: the
                         printed outside of the wall (its side seam a board
                         proud, the foot folded in under the bottom); the
                         white inside of the wall; the rim's curl; the floor
                         and the bottom disc's underside
paperCupSqueezed*        the same cup squeezed in a fist
paperPlateTop / Under    a 7 in coated paper dessert plate, pressed: its
                         well, wall and flange, pleated; the printed top
                         face and the plain board under it
paperPlateBentTop / Under   the same plate trodden on: creased across, a
                         third of it bent up, crumpled at the crease
cakeStandFoot / Plate    an 11 in pressed glass cake stand: the hollow
                         trumpet foot in twelve panels with its knop, and
                         the plate (rayed underneath, its rim scalloped)
cakeStandCake            a two-tier buttercream cake on it, dried, shrunk
                         and slumped: shell borders, six rosettes
cakeStandCandle / Wick   a spiral birthday candle burnt down in the top
                         tier, its wax run onto the frosting; its wick
cornMuffinCase           a standard pleated paper baking cup, loose round
                         the shrunk muffin
cornMuffinA / B          the muffin: the body in the case and its crown,
                         pebbly, cracked across
"""
from propkit import *
import propkit
import random
import zlib


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, smooth=35.0, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth)


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def tri(x):
    """A triangle wave of period 1: 1 at whole x, -1 at the halves."""
    f = x - math.floor(x)
    return 4 * abs(f - 0.5) - 1


def even(n, start=0.0):
    return [start + TAU * i / n for i in range(n)]


def lathe_r(profile, angles, warp=None):
    """Revolve [(r, y[, tag]), ...] round +y at the given angles (from +z
    toward +x). Faces turn to the right of the profile's direction in the
    (r, y) half plane: up the outside faces out, outward along a bottom
    faces down, inward along a top faces up. A point with r = 0 is one
    vertex on the axis. warp(a, r, y, tag) -> (dr, dy) moves a point."""
    v, rings = [], []
    for p in profile:
        r, y = p[0], p[1]
        tag = p[2] if len(p) > 2 else None
        if r <= 1e-7:
            v.append(Vector((0.0, y, 0.0)))
            rings.append([len(v) - 1])
            continue
        ids = []
        for a in angles:
            rr, yy = r, y
            if warp:
                dr, dy = warp(a, r, y, tag)
                rr, yy = rr + dr, yy + dy
            v.append(Vector((rr * math.sin(a), yy, rr * math.cos(a))))
            ids.append(len(v) - 1)
        rings.append(ids)
    n = len(angles)
    f = []
    for A, B in zip(rings, rings[1:]):
        if len(A) == 1 and len(B) == 1:
            continue
        for i in range(n):
            j = (i + 1) % n
            if len(A) == 1:
                f.append((A[0], B[j], B[i]))
            elif len(B) == 1:
                f.append((A[i], A[j], B[0]))
            else:
                f.append((A[i], A[j], B[j], B[i]))
    return Shape(v, f)


def ring_sweep(centre_y, R, n, samples, tube, tseg=5, lift=0.0):
    """A piped border round the y axis: n shells, each a fat head tapering to
    a tail into the next, `samples` sections a shell; tube(phase) is the
    section's radius (phase 0 at the head)."""
    N = n * samples
    v, f = [], []
    for i in range(N):
        a = TAU * i / N
        ph = (i % samples) / samples
        rt = tube(ph)
        out = Vector((math.sin(a), 0.0, math.cos(a)))
        c = out * R + Vector((0.0, centre_y + lift * rt, 0.0))
        for k in range(tseg):
            b = TAU * k / tseg
            v.append(c + out * (rt * math.cos(b)) + Vector((0.0, rt * math.sin(b) * 0.85, 0.0)))
    for i in range(N):
        i2 = (i + 1) % N
        for k in range(tseg):
            k2 = (k + 1) % tseg
            f.append((i * tseg + k, i * tseg + k2, i2 * tseg + k2, i2 * tseg + k))
    return propkit._fix_winding(Shape(v, f))


def on_floor(s, y0=0.0):
    lo, hi = s.bounds()
    return s.move(0, y0 - lo.y, 0)


# ============================================================ paper cup ===
# A 9 oz printed paper hot/cold cup (Creative Converting's poly paper cup:
# 3 in across the top, 3 1/2 in tall, 2 in across the bottom). Cupstock
# board 0.4 mm; the rim curled outward into a 2.9 mm bead; the bottom disc
# set 6 mm up, the wall's foot folded in round its skirt.
CUP_RB = 0.0254          # the base's edge
CUP_RW = 0.0352          # the wall at the top, under the curl
CUP_YW = 0.0870          # where the wall meets the curl
CUP_T = 0.0004           # the board
CUP_RC = 0.00145         # the curl's radius
CUP_DISC = 0.0060        # the bottom disc's underside
CUP_FLOOR = 0.0066       # the floor inside
CUP_FOOT = 0.0016        # the foot fold's width under the disc
CUP_C = (CUP_RW + 0.7071 * CUP_RC, CUP_YW + 0.7071 * CUP_RC)    # the curl's centre
CUP_ANG = sorted(even(36) + [0.012, 0.2000, 0.2015])


def cup_r(y):
    return CUP_RB + (CUP_RW - CUP_RB) * y / CUP_YW


def cup_seam(a, r, y, tag):
    """The side seam: the blank's outer end lies over its inner end, a
    board proud, its cut edge a step."""
    return (CUP_T * 0.9 if 0.012 <= a <= 0.2001 else 0.0, 0.0)


def cup_outer():
    prof = [(CUP_RB - CUP_FOOT, CUP_DISC), (CUP_RB - CUP_FOOT, 0.0012), (CUP_RB - CUP_FOOT + 0.0003, 0.0002),
            (CUP_RB - 0.0006, 0.0), (CUP_RB - 0.0001, 0.0004), (CUP_RB, 0.0012)]
    prof += [(cup_r(y), y) for y in (0.006, 0.016, 0.028, 0.040, 0.052, 0.064, 0.076, CUP_YW)]
    return lathe_r(prof, CUP_ANG, cup_seam)


CUP_CURL = [(CUP_C[0] + CUP_RC * math.cos(math.radians(225 + 52.5 * k)), CUP_C[1] + CUP_RC * math.sin(math.radians(225 + 52.5 * k)))
            for k in range(7)]
CUP_WALL_FOOT = (cup_r(CUP_FLOOR + 0.002) - CUP_T, CUP_FLOOR + 0.002)


def cup_rim():
    """The curl, from the outside's top over to the inside's."""
    return lathe_r(CUP_CURL, even(36))


def cup_inner():
    """The inside of the wall, from the curl down to the floor's crease."""
    wall = [CUP_CURL[-1]] + [(cup_r(y) - CUP_T, y) for y in (0.082, 0.070, 0.058, 0.046, 0.034, 0.022, 0.012)] + [CUP_WALL_FOOT]
    return lathe_r(wall, even(36))


def cup_bottom():
    """The floor inside, its crease into the wall, and the bottom disc's
    underside in the foot."""
    floor = [CUP_WALL_FOOT, (cup_r(CUP_FLOOR) - CUP_T - 0.0006, CUP_FLOOR + 0.0003), (cup_r(CUP_FLOOR) - CUP_T - 0.0016, CUP_FLOOR),
             (0.012, CUP_FLOOR), (0.0, CUP_FLOOR)]
    under = lathe_r([(0.0, CUP_DISC), (0.012, CUP_DISC), (CUP_RB - CUP_FOOT, CUP_DISC)], even(36))
    return lathe_r(floor, even(36)) + under


def cup_squeeze(p):
    """Squeezed in a fist: the wall pressed flat across x toward the rim (the
    base's disc holds it round), the board folding in sharp creases, the rim
    tipped down on one side. A point's move depends on its angle and height
    (and its radius, a little), so the inside follows the outside."""
    a = math.atan2(p.x, p.z)
    r = math.hypot(p.x, p.z)
    y = p.y
    k = r / 0.031
    s = 0.50 * smoothstep(0.010, 0.078, y)
    fold = smoothstep(0.008, 0.060, y)
    # creases: sharp inward folds where the fingers pressed, a softer
    # outward bulge between them, and the board's crumple
    crease = 0.0
    for m, ph, depth in ((5, 0.6, 0.0030), (3, 2.2, 0.0018)):
        c = math.cos(m * a + ph + 9.0 * y)
        crease -= depth * max(0.0, c) ** 6
        crease += depth * 0.25 * max(0.0, -c)
    q = Vector((math.sin(a) * 3.0, y * 40.0, math.cos(a) * 3.0))
    crumple = 0.0012 * noise.noise(q) + 0.0005 * noise.noise(q * 3.1 + Vector((4.0, 1.0, 7.0)))
    rr = r + (crease + crumple) * fold * k
    x = rr * math.sin(a) * (1 - s)
    z = rr * math.cos(a) * (1 + 0.55 * s)
    # leaning a little, the rim down on its -z side
    x += 0.06 * (y ** 2) / 0.087
    y2 = y - 0.005 * smoothstep(0.05, 0.088, y) * (1 - math.cos(a)) / 2
    return Vector((x, y2, z))


# ============================================================ paper plate ==
# A 7 in (178 mm) coated paper dessert plate, pressed from board 0.45 mm
# thick: a flat well 110 mm across, a wall at about 30 degrees, a flange 13
# mm wide, its edge turned down; 32 pleats in the wall and flange; 12 mm
# deep over all.
PL_T = 0.00045
PL_TOP = [(0.0889, 0.0100), (0.0878, 0.0112), (0.0855, 0.0119), (0.0810, 0.0117), (0.0760, 0.0111),
          (0.0730, 0.0100), (0.0690, 0.0076), (0.0640, 0.0046), (0.0590, 0.0018), (0.0555, 0.0007),
          (0.0450, 0.0006), (0.0250, 0.0008), (0.0, 0.0010)]    # the top face, from the edge in
PL_N = 32


def plate_pleat(a, r, y, tag):
    env = smoothstep(0.056, 0.064, r)
    t = tri(a * PL_N / TAU)
    return (0.00030 * t * smoothstep(0.080, 0.089, r), 0.00050 * env * t)


def plate_under_profile():
    """The underside: the top face moved a board's thickness along its
    normal, from the middle out."""
    pts = list(reversed(PL_TOP))      # from the middle out
    out = []
    for i, (r, y) in enumerate(pts):
        a = pts[max(i - 1, 0)]
        b = pts[min(i + 1, len(pts) - 1)]
        dr, dy = b[0] - a[0], b[1] - a[1]
        l = math.hypot(dr, dy) or 1.0
        nr, ny = -dy / l, dr / l           # left of outward: up
        out.append((max(0.0, r - PL_T * nr) if r > 0 else 0.0, y - PL_T * ny))
    return out


def plate_top():
    under = plate_under_profile()
    edge = [under[-1]]                     # the cut edge, from under to over
    return lathe_r(edge + PL_TOP, even(2 * PL_N), plate_pleat)


def plate_under():
    return lathe_r(plate_under_profile(), even(2 * PL_N), plate_pleat)


PL_CREASE = -0.022      # where it was creased across (z)
PL_BEND = math.radians(28)


def plate_tread(p):
    """Trodden on: creased across at z = PL_CREASE, the far side bent up
    about the crease (the bend taken over 8 mm), the wall and flange
    crumpled where the crease crosses them, the near flange pressed flat."""
    x, y, z = p.x, p.y, p.z
    r = math.hypot(x, z)
    q = Vector((x * 70.0, y * 70.0, z * 70.0))
    # crumple, most at the crease and round the rim
    near = math.exp(-((z - PL_CREASE) / 0.012) ** 2)
    rim = smoothstep(0.055, 0.075, r)
    y += (0.0020 * near * rim + 0.0005 * rim) * noise.noise(q)
    x += 0.0012 * near * rim * noise.noise(q + Vector((3.1, 7.7, 1.3)))
    # the near flange pressed down where the heel came
    if z < PL_CREASE:
        y -= 0.0045 * rim * smoothstep(0.0, 0.05, PL_CREASE - z) * max(0.0, y - 0.002) / 0.012
    dz = z - PL_CREASE
    if dz > 0:
        th = PL_BEND * smoothstep(0.0, 0.008, dz)
        z, y = PL_CREASE + dz * math.cos(th) - y * math.sin(th), dz * math.sin(th) + y * math.cos(th)
    return Vector((x, y, z))


# ============================================================ cake stand ===
# An 11 in pressed glass cake stand (the mid-century Federal, L.E. Smith,
# Indiana kind: 11 in across, 4 1/2 - 4 3/4 in tall): a hollow trumpet foot
# pressed in twelve panels, its rim ground flat; a knop; the plate pressed
# separately and fused on, rayed underneath, a low rim turned up and
# scalloped.
SF_FOOT = [
    (0.0, 0.0880, 'in'),
    (0.0120, 0.0860, 'in'), (0.0170, 0.0780, 'in'), (0.0215, 0.0600, 'in'), (0.0300, 0.0380, 'in'),
    (0.0420, 0.0200, 'in'), (0.0515, 0.0080, 'in'), (0.0545, 0.0025, 'in'),
    # the foot's rim, ground flat where it stands
    (0.0560, 0.0002, 'rim'), (0.0590, 0.0, 'rim'), (0.0615, 0.0010, 'rim'), (0.0624, 0.0045, 'rim'), (0.0605, 0.0085, 'rim'),
    # up the trumpet, in panels
    (0.0540, 0.0150, 'foot'), (0.0440, 0.0260, 'foot'), (0.0350, 0.0400, 'foot'), (0.0285, 0.0560, 'foot'),
    (0.0245, 0.0710, 'foot'), (0.0218, 0.0850, 'foot'),
    # the knop
    (0.0245, 0.0905, 'knop'), (0.0278, 0.0945, 'knop'), (0.0262, 0.0985, 'knop'), (0.0228, 0.1010, 'knop'),
    (0.0, 0.1015, 'knop'),
]
SF_PANELS = 12
SP_PLATE = [
    (0.0, 0.1000, 'under'),
    (0.0240, 0.1012, 'under'), (0.0350, 0.1050, 'under'), (0.0600, 0.1072, 'under'), (0.0900, 0.1090, 'under'),
    (0.1150, 0.1104, 'under'), (0.1320, 0.1116, 'under'),
    # the edge, turned up into a low scalloped rim
    (0.1382, 0.1130, 'rim'), (0.1400, 0.1152, 'rim'), (0.1404, 0.1180, 'rim'), (0.1394, 0.1199, 'rim'),
    (0.1372, 0.1203, 'rim'), (0.1345, 0.1188, 'rim'), (0.1290, 0.1160, 'top'),
    (0.0900, 0.1155, 'top'), (0.0, 0.1152, 'top'),
]
SP_RAYS = 24
STAND_TOP = 0.1152       # the plate's top at its middle


def stand_foot_warp(a, r, y, tag):
    if tag != 'foot':
        return (0.0, 0.0)
    w = TAU / SF_PANELS
    off = (a % w) - w / 2
    return (r * (math.cos(w / 2) / math.cos(off) - 1.0) * 0.9, 0.0)


def stand_plate_warp(a, r, y, tag):
    ph = (a * SP_RAYS / TAU) % 1.0
    cusp = 1.0 if min(ph, 1 - ph) < 0.05 else 0.0
    if tag == 'under' and r > 0.03:
        # the rays: grooves pressed up into the underside
        return (0.0, 0.0016 * cusp * smoothstep(0.03, 0.05, r) * (1 - smoothstep(0.12, 0.133, r)))
    if tag == 'rim':
        # scallops: the rim notched between them
        return (-0.0026 * cusp, -0.0007 * cusp * smoothstep(0.138, 0.140, r))
    return (0.0, 0.0)


def stand_foot():
    return lathe_r(SF_FOOT, even(2 * SF_PANELS), stand_foot_warp)


def stand_plate():
    return lathe_r(SP_PLATE, even(3 * SP_RAYS), stand_plate_warp)


# ----------------------------------------------------------------- cake ---
# A two-tier buttercream cake, 8 in under 5 in (each tier two layers), left
# for years: dried and shrunk (98 and 63 mm in radius, 79 and 64 mm tall),
# the top tier sunk into the bottom one and leaning, the tops caved in a
# little, the bottom tier's side bulged; piped shell borders round each
# tier's foot, six rosettes on the bottom tier's shoulder.
CK_Y0 = STAND_TOP + 0.0003
CK_R1, CK_H1 = 0.098, 0.079
CK_R2, CK_H2 = 0.063, 0.064
CK_Y1 = CK_Y0 + CK_H1 - 0.003           # the top tier's foot, sunk in
CK_TOP2 = (0.004, -0.003)               # the top tier's middle (x, z), off centre
CK_LEAN = math.radians(1.6)
CANDLE_AT = (0.016, 0.011)              # the candle's place on the top tier (x, z from its middle)


def cake_tiers():
    y0 = CK_Y0
    t1 = lathe_r([(0.0, y0), (0.0960, y0), (0.0982, y0 + 0.003), (0.0992, y0 + 0.016), (0.0985, y0 + 0.040),
                  (0.0972, y0 + 0.064), (0.0950, y0 + 0.0745), (0.0905, y0 + 0.0785), (0.0800, y0 + 0.0792),
                  (0.0660, y0 + 0.0780), (0.0300, y0 + 0.0770), (0.0, y0 + 0.0766)], even(48))
    y1 = CK_Y1
    t2 = lathe_r([(0.0, y1), (0.0615, y1), (0.0630, y1 + 0.004), (0.0634, y1 + 0.026), (0.0626, y1 + 0.051),
                  (0.0608, y1 + 0.0595), (0.0568, y1 + 0.0636), (0.0440, y1 + 0.0642), (0.0200, y1 + 0.0626),
                  (0.0, y1 + 0.0620)], even(48))
    b2 = ring_sweep(y1 + 0.004, 0.0640, 18, 3, lambda ph: 0.0058 * (1.0 - 0.62 * ph ** 1.2), tseg=5)
    top = tier_pose(t2 + b2)
    b1 = ring_sweep(y0 + 0.0055, 0.0995, 28, 3, lambda ph: 0.0072 * (1.0 - 0.62 * ph ** 1.2), tseg=5)
    ros = Shape()
    for i in range(6):
        ang = TAU * i / 6 + 0.26
        base = y0 + 0.0775
        star = lathe_r([(0.0, base - 0.003), (0.0112, base - 0.001), (0.0102, base + 0.005), (0.0062, base + 0.0098),
                        (0.0, base + 0.0118)], even(16),
                       lambda a, r, y, t, i=i: (0.0017 * math.cos(8 * a + (y - base) * 420.0 + i) * smoothstep(0.002, 0.008, r), 0.0))
        star.scale(1.0, 0.86, 1.0)
        lo, hi = star.bounds()
        star.move(0, base - 0.003 - lo.y, 0)
        star.rot(0.0, ang, 0.0).move(0.0812 * math.sin(ang), -0.0012, 0.0812 * math.cos(ang))
        ros = ros + star
    cake = t1 + b1 + top + ros
    # the buttercream's lumps and the slump of years (not where it sits on
    # the glass)
    for i, p in enumerate(cake.v):
        k = smoothstep(CK_Y0 + 0.0005, CK_Y0 + 0.004, p.y)
        cake.v[i] = p + noise3(p * 110.0 + Vector((5.0, 2.0, 9.0))) * 0.0006 * k
    return cake


def tier_pose(s):
    """The top tier as it came to rest: leaning about its foot, off the
    middle (what is in it goes with it)."""
    return s.move(0, -CK_Y1, 0).rot(CK_LEAN, 0, 0).move(CK_TOP2[0], CK_Y1, CK_TOP2[1])


def candle_top_y():
    """The top tier's surface under the candle (in the tier's own frame,
    before it leans)."""
    r = math.hypot(*CANDLE_AT)
    return CK_Y1 + 0.0620 + (0.0626 - 0.0620) * min(1.0, r / 0.02)


CANDLE_R = 0.0026
CANDLE_UP = 0.034        # burnt down to 34 mm above the frosting (of 64)


def cake_candle():
    base = candle_top_y() - 0.008
    top = candle_top_y() + CANDLE_UP
    rows = [base + (top - 0.002 - base) * i / 13 for i in range(14)]
    prof = [(0.0, base)] + [(CANDLE_R, y) for y in rows] + [(CANDLE_R - 0.0004, top), (0.0012, top - 0.0004),
                                                           (0.0006, top - 0.0016), (0.0, top - 0.0014)]
    # four flutes twisted round it, as they are pressed
    c = lathe_r(prof, even(16), lambda a, r, y, t: (0.00045 * math.cos(4 * a + (y - base) * TAU / 0.011) if r >= CANDLE_R - 1e-6 else 0.0, 0.0))
    # the wax run down it and onto the frosting: two runs and a pool
    runs = Shape()
    for ang, ln in ((0.7, 0.012), (2.9, 0.019)):
        y_end = top - 0.002 - ln
        pts = [Vector((math.sin(ang) * (CANDLE_R + 0.0003), top - 0.0025, math.cos(ang) * (CANDLE_R + 0.0003))),
               Vector((math.sin(ang) * (CANDLE_R + 0.0005), (top + y_end) / 2, math.cos(ang) * (CANDLE_R + 0.0005))),
               Vector((math.sin(ang) * (CANDLE_R + 0.0004), y_end, math.cos(ang) * (CANDLE_R + 0.0004)))]
        runs = runs + tube(pts, 0.0007, seg=5, r_end=0.0011)
    surf = candle_top_y()
    pool = lathe_r([(0.0, surf - 0.0004), (0.0062, surf - 0.0004), (0.0060, surf + 0.0006), (0.0040, surf + 0.0013),
                    (CANDLE_R + 0.0002, surf + 0.0022), (0.0, surf + 0.0022)], even(14),
                   lambda a, r, y, t: (0.0012 * math.sin(3 * a + 1.0) * (r / 0.006), 0.0))
    s = c + runs + pool
    return tier_pose(s.move(CANDLE_AT[0], 0, CANDLE_AT[1]))


def cake_wick():
    top = candle_top_y() + CANDLE_UP
    pts = [(0.0, top - 0.0016, 0.0), (0.0002, top + 0.0020, 0.0), (0.0010, top + 0.0045, 0.0003), (0.0024, top + 0.0056, 0.0006)]
    w = tube([Vector(p) for p in pts], 0.00042, seg=5, r_end=0.00030)
    return tier_pose(w.move(CANDLE_AT[0], 0, CANDLE_AT[1]))


# ============================================================== muffins ===
# A cornbread muffin baked in a standard paper baking cup (2 in base, the
# wall 1 5/16 in, flared to about 2 3/4 in by the batter), years on: the
# muffin dried hard and shrunk away from its case, its crown cracked across.
MC_R0, MC_R1, MC_H = 0.0250, 0.0335, 0.0330
MC_N = 24


def case_r(y):
    return MC_R0 + (MC_R1 - MC_R0) * y / MC_H


def muffin_case():
    prof = [(0.0, 0.0002), (0.0150, 0.0002), (MC_R0 - 0.0007, 0.0002), (MC_R0 - 0.0001, 0.0008)]
    prof += [(case_r(y), y) for y in (0.0040, 0.0120, 0.0200, 0.0270, MC_H)]

    def pleat(a, r, y, tag):
        env = smoothstep(0.0006, 0.0040, y)
        loose = smoothstep(0.018, MC_H, y)
        splay = 0.0016 * loose * max(0.0, noise.noise(Vector((math.sin(a) * 2.2, 0.3, math.cos(a) * 2.2))) + 0.15)
        return (0.0007 * env * tri(a * MC_N / TAU) + splay, -0.0008 * loose * max(0.0, math.sin(3 * a + 0.4)))
    return lathe_r(prof, even(2 * MC_N), pleat)


def muffin(seed, dome, cracks):
    """The muffin, shrunk: its body in the case, a gap opening toward the
    case's edge; its crown just over the edge, domed `dome` m above it, the
    cornmeal's pebbled crust; cracks [(x0, z0, x1, z1, depth)] across it."""
    body = [(0.0, 0.0006), (0.0236, 0.0006), (0.0241, 0.0030)]
    body += [(case_r(y) - 0.0007 - 0.0005 - 0.0016 * y / MC_H, y) for y in (0.010, 0.020, 0.028, 0.0325)]
    top0 = MC_H + 0.0025
    crown = [(0.0329, top0), (0.0322, top0 + 0.0030), (0.0296, top0 + dome * 0.40), (0.0258, top0 + dome * 0.62),
             (0.0215, top0 + dome * 0.78), (0.0165, top0 + dome * 0.89), (0.0110, top0 + dome * 0.96),
             (0.0055, top0 + dome * 0.99), (0.0, top0 + dome)]
    s = lathe_r(body + crown, even(36))
    out = s.copy()
    for i, p in enumerate(s.v):
        if p.y < MC_H - 0.002:
            continue
        up = smoothstep(MC_H - 0.002, MC_H + 0.004, p.y)
        q = p * 180.0 + Vector((seed * 3.1, seed * 1.7, seed * 2.3))
        # the pebbled crust: lumps of the coarse meal
        d = (0.0007 * noise.noise(q) + 0.00035 * noise.noise(q * 2.7)) * up
        n = Vector((p.x, 0.0, p.z))
        n = n.normalized() if n.length > 1e-6 else Vector((0, 0, 0))
        n = (n + Vector((0.0, 1.3, 0.0))).normalized()
        pp = p + n * d
        # the cracks: the crown split open as it dried
        for (x0, z0, x1, z1, depth) in cracks:
            ax, az = x1 - x0, z1 - z0
            L2 = ax * ax + az * az
            t = max(0.0, min(1.0, ((p.x - x0) * ax + (p.z - z0) * az) / L2))
            dist = math.hypot(p.x - (x0 + ax * t), p.z - (z0 + az * t))
            w = 0.0036
            if dist < w:
                pp.y -= depth * (1 - dist / w) ** 2 * up * math.sin(math.pi * min(1.0, 0.15 + t * 0.85)) ** 0.5
        out.v[i] = pp
    return out


MUFFIN_CRACKS = {
    'A': [(-0.024, 0.006, 0.022, -0.004, 0.0034), (0.004, 0.0, 0.010, 0.020, 0.0024)],
    'B': [(-0.018, -0.016, 0.0, 0.002, 0.0030), (0.0, 0.002, 0.021, -0.006, 0.0028), (0.0, 0.002, -0.004, 0.022, 0.0026)],
}


def muffin_a():
    return muffin(3, 0.0185, MUFFIN_CRACKS['A'])


def muffin_b():
    return muffin(8, 0.0160, MUFFIN_CRACKS['B'])


# ------------------------------------------------------------ the parts ---
SQUEEZED = {}


def squeezed(build):
    return lambda: build().warp(cup_squeeze)


auto('Real__paperCupOuter', cup_outer, 1100, kind='cyl')
auto('Real__paperCupInner', cup_inner, 700, kind='cyl')
auto('Real__paperCupRim', cup_rim, 500, kind='cyl')
auto('Real__paperCupBottom', cup_bottom, 400, kind='cyl')
auto('Real__paperCupSqueezedOuter', squeezed(cup_outer), 1100, smooth=50.0, kind='cyl')
auto('Real__paperCupSqueezedInner', squeezed(cup_inner), 700, smooth=50.0, kind='cyl')
auto('Real__paperCupSqueezedRim', squeezed(cup_rim), 500, smooth=50.0, kind='cyl')
auto('Real__paperCupSqueezedBottom', squeezed(cup_bottom), 400, smooth=50.0, kind='cyl')
auto('Real__paperPlateTop', plate_top, 1700)
auto('Real__paperPlateUnder', plate_under, 1600)
auto('Real__paperPlateBentTop', lambda: plate_top().warp(plate_tread), 1700, smooth=50.0)
auto('Real__paperPlateBentUnder', lambda: plate_under().warp(plate_tread), 1600, smooth=50.0)
auto('Real__cakeStandFoot', stand_foot, 1300, smooth=25.0, kind='cyl')
auto('Real__cakeStandPlate', stand_plate, 2400, smooth=25.0, kind='cyl')
auto('Real__cakeStandCake', cake_tiers, 4600, smooth=60.0, kind='cyl')
auto('Real__cakeStandCandle', cake_candle, 900, smooth=50.0, kind='cyl')
auto('Real__cakeStandWick', cake_wick, 120, smooth=60.0)
auto('Real__cornMuffinCase', muffin_case, 760, kind='cyl')
auto('Real__cornMuffinA', muffin_a, 1300, smooth=60.0, kind='cyl')
auto('Real__cornMuffinB', muffin_b, 1300, smooth=60.0, kind='cyl')

# what the game needs to know of them (copied into the placers' constants)
_root = tier_pose(Shape([(CANDLE_AT[0] + 0.0008, candle_top_y() + CANDLE_UP + 0.0004, CANDLE_AT[1])], [])).v[0]
print(f'[props] t13: cup curl outer r {CUP_C[0] + CUP_RC:.5f} at y {CUP_C[1]:.5f}; base r {CUP_RB:.5f}')
print(f'[props] t13: the flame root ({_root.x:.4f}, {_root.y:.4f}, {_root.z:.4f}); the plate top {STAND_TOP:.4f}')
