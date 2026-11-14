"""six_things_hung_on_a_wall (was r13a): the props remodelled at their real sizes (docs/design/PROPS.md), P2: six
things that hang on a wall. Sources and sizes: docs/research/props-remodelling/six_things_hung_on_a_wall.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part here is in
the frame of the wall it hangs on: x across its face (its right seen from
the front), y up from its middle, z out of the wall (z = 0 the wall's face).
Randomness comes from a `random.Random` of the part's own.

The domestic wall mirror (`place_wall_mirrors`): 42 x 60 cm overall, a
34 mm moulding round 4 mm silvered glass held in its rebate on a hardboard
back, hung flat on four brass mirror plates. Two frames on the one size:

wallMirrorFrameWood   a stained hardwood moulding: rounded outer edge, a
                      low ogee crest, a quirk, a cove down to the sight
                      bead; its rebate behind
wallMirrorFrameGilt   a gilt cushion frame: outer bead, a raised cushion,
                      a sight bead and its slip
wallMirrorPlates      the four brass plates (two at the top, two at the
                      foot) and their countersunk wall screws

Under it, when the old placer laid out its washroom: a wall-hung
vitreous china basin, 50 x 40 cm, its rim 0.895 m from the floor (where
the soap and the toothbrush lie). Its frame: x across, y up from the
floor, z out of the wall.

wallMirrorBasin       the basin: its rim rolled, the deck behind the bowl
                      with two tap holes, the plug hole
wallMirrorBasinBowl   the bowl's glazed inside (painted apart: what ran
                      down it)
wallMirrorBasinTrap   the chrome waste in the plug hole, its tail, the
                      bottle trap and the pipe back into the wall
wallMirrorBasinTaps   two chrome pillar taps on the deck, crossheads

The small cracked mirror (`place_cracked_mirrors`): 30 x 40 cm glass in a
24 mm half-round moulding, hung from one nail (a sawtooth hanger, unseen).

crackedMirrorFrame    the moulding and its rebate

The chalkboard (`place_wall_chalkboards`): the common 30 x 40 cm wall
board, a pine frame 22 mm wide and 14 mm deep round a painted board held
in its groove.

chalkboardFrame       the frame, its groove; the board is the game's plane

The rustic sign (`place_rustic_signs`): three pine boards nailed to two
battens behind, hung by a jute cord from two screw eyes over one nail.
Its frame: y up from the nail's height less 0.172 m (the boards' middle).

rusticSignBoards      the three boards, each its own length and its own
                      sawn ends, edges eased, a little cupped
rusticSignBattens     the two battens behind them
rusticSignNails       the nail heads through the boards into the battens
rusticSignEyes        the two screw eyes in the top board's edge
rusticSignCord        the jute cord, three plies twisted, knotted at each
                      eye with its ends hanging
rusticSignNail        the nail in the wall the cord hangs on

The biohazard sign (`place_biohazard_signs`): an A4 (210 x 297 mm) rigid
PVC sign, 1 mm, its corners rounded, four holes, screwed to the wall.

biohazardPlate        the plate, flat on the wall
biohazardPlateLifted  the same with its top right screw gone: that corner
                      has sprung 4 mm off the wall
biohazardScrews       four pan-head screws
biohazardScrews3      three (the top right one gone)

The sampler (`place_cross_stitches`): an alphabet sampler on linen,
stretched and framed: 22 x 28 cm overall, a 25 mm oak scoop moulding.

samplerFrame          the moulding and its rebate; the linen is the
                      game's plane
"""
from propkit import *
import propkit
import random
from mathutils import Vector


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle: d in from
    its outer edge, z out from the wall; the corners mitred."""
    clean = []
    for p in profile:
        if not clean or abs(p[0] - clean[-1][0]) + abs(p[1] - clean[-1][1]) > 1e-7:
            clean.append(p)
    if abs(clean[0][0] - clean[-1][0]) + abs(clean[0][1] - clean[-1][1]) < 1e-7:
        clean.pop()
    v, f = [], []
    n = len(clean)
    for (d, z) in clean:
        x, y = w / 2 - d, h / 2 - d
        v += [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    for i in range(n):
        j = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, j * 4 + k2, j * 4 + k))
    return propkit._fix_winding(Shape(v, f))


def arc(cx, cy, r, a0, a1, n):
    return [(cx + r * math.cos(a0 + (a1 - a0) * i / n), cy + r * math.sin(a0 + (a1 - a0) * i / n)) for i in range(n + 1)]


# ======================================================== the wall mirror ===
WM_W, WM_H = 0.420, 0.600      # overall
WM_MOULD = 0.034               # the moulding's face, outer edge to sight edge
WM_BACK = 0.0009               # the frame's back stands on the plates
WM_GLASS = 0.0090              # the glass's face in its rebate
WM_REBATE = 0.0280             # the rebate's wall (d)


def wall_mirror_wood():
    B = WM_BACK
    prof = [(0.0, B), (0.0, 0.0150), (0.0006, 0.0172), (0.0020, 0.0186), (0.0040, 0.0193),
            # a low ogee: up over the crest, round into the quirk
            (0.0072, 0.0201), (0.0100, 0.0204), (0.0122, 0.0201), (0.0138, 0.0194), (0.0147, 0.0185),
            (0.0152, 0.0178), (0.0160, 0.0179),
            # the cove down toward the glass
            (0.0185, 0.0174), (0.0210, 0.0164), (0.0235, 0.0150), (0.0262, 0.0137), (0.0290, 0.0129),
            # the sight bead
            (0.0308, 0.0130), (0.0324, 0.0127), (0.0334, 0.0119), (0.0340, 0.0106), (0.0340, WM_GLASS),
            # the rebate behind the lip (the glass and its back sit in it)
            (WM_REBATE, WM_GLASS), (WM_REBATE, B)]
    return rect_sweep(prof, WM_W, WM_H)


def wall_mirror_gilt():
    B = WM_BACK
    prof = [(0.0, B), (0.0, 0.0100), (0.0008, 0.0135), (0.0025, 0.0156)]
    # the outer bead
    prof += [(0.0040, 0.0170), (0.0055, 0.0177), (0.0070, 0.0172), (0.0079, 0.0162), (0.0086, 0.0158)]
    # the cushion
    prof += [(0.0105, 0.0182), (0.0135, 0.0207), (0.0168, 0.0224), (0.0200, 0.0229), (0.0230, 0.0222),
             (0.0254, 0.0204), (0.0270, 0.0183), (0.0277, 0.0170)]
    # the sight bead and its slip
    prof += [(0.0289, 0.0176), (0.0303, 0.0180), (0.0317, 0.0174), (0.0328, 0.0160), (0.0335, 0.0142),
             (0.0340, 0.0122), (0.0340, WM_GLASS), (WM_REBATE, WM_GLASS), (WM_REBATE, B)]
    return rect_sweep(prof, WM_W, WM_H)


def mirror_plate(L=0.038, wd=0.016, out=0.013):
    """One brass mirror plate lying on the wall, its foot under the frame's
    back and its head standing `out` past the frame's edge (+y), the wall
    screw countersunk in its head. Built about the frame's edge (y = 0)."""
    t = 0.0009
    r = wd / 2
    pts = arc(0, out - r, r, 0, math.pi, 6) + arc(0, out - L + r, r, math.pi, 2 * math.pi, 6)
    pl = prism(pts, t).move(0, 0, t / 2)
    # the wall screw, countersunk in its raised hole
    sx, sy = 0, out - r
    head = lathe([(0.0, 0.0), (0.0034, 0.0), (0.0033, 0.0005), (0.0024, 0.0010), (0.0, 0.0011)], seg=10)
    head = cut(head, box(0.0080, 0.0012, 0.0007).move(0, 0.0010, 0)).rot(math.pi / 2, 0, 0)
    head.rot(0, 0, 0.6).move(sx, sy, t)
    return pl + head


def wall_mirror_plates():
    s = Shape()
    for x in (-0.12, 0.12):
        top = mirror_plate().move(x, WM_H / 2, 0)
        foot = mirror_plate().rot(0, 0, math.pi).move(x, -WM_H / 2, 0)
        s = s + top + foot
    return s


# ============================================================== the basin ===
BS_TOP = 0.895                 # the rim and deck, from the floor
BS_W, BS_D = 0.500, 0.400      # across, out from the wall
BS_BOWL = (0.0, 0.240)         # the bowl's middle (x, z)
BS_TAP_Z = 0.052


def loft(rings):
    """Rings of equal point counts [(y, [(x, z), ...]), ...] bottom to top,
    joined by quads, capped."""
    v, f = [], []
    n = len(rings[0][1])
    for (y, pts) in rings:
        v += [(x, y, z) for x, z in pts]
    for r in range(len(rings) - 1):
        for i in range(n):
            j = (i + 1) % n
            f.append((r * n + i, r * n + j, (r + 1) * n + j, (r + 1) * n + i))
    f.append(tuple(reversed(range(n))))
    f.append(tuple((len(rings) - 1) * n + i for i in range(n)))
    return propkit._fix_winding(Shape(v, f))


def basin_ring(k, kd):
    """The plan's outline at a scale: the back straight, the front a half
    ellipse (24 points)."""
    hw = BS_W / 2 * k
    zc, rz = 0.17 * kd, (BS_D - 0.17) * kd
    pts = [(-hw, 0.004), (hw, 0.004), (hw, zc * 0.5)]
    for i in range(1, 20):
        a = math.pi * i / 20                       # 0 at the right, pi at the left
        pts.append((hw * math.cos(a), zc + rz * math.sin(a)))
    pts.append((-hw, zc * 0.5))
    return pts


def basin():
    rings = [(BS_TOP - 0.175, basin_ring(0.70, 0.62)), (BS_TOP - 0.13, basin_ring(0.88, 0.86)),
             (BS_TOP - 0.06, basin_ring(0.985, 0.99)), (BS_TOP, basin_ring(1.0, 1.0))]
    s = loft(rings)
    s = bevel(s, 0.010, seg=3, angle=40)
    bx, bz = BS_BOWL
    bowl = sphere(1.0, seg=28).scale(0.190, 0.125, 0.135).move(bx, BS_TOP + 0.005, bz)
    plug = cyl(0.021, 0.021, 0.08, seg=16, bevel_r=0).move(bx, BS_TOP - 0.13, bz + 0.01)
    holes = [cyl(0.0175, 0.0175, 0.06, seg=14, bevel_r=0).move(sx * 0.08, BS_TOP, BS_TAP_Z) for sx in (-1, 1)]
    return cut(s, bowl, plug, *holes)


def in_bowl(c):
    """Is a face (its centre) on the bowl's glazed inside?"""
    bx, bz = BS_BOWL
    e = ((c.x - bx) / 0.190) ** 2 + ((c.y - BS_TOP - 0.005) / 0.125) ** 2 + ((c.z - bz) / 0.135) ** 2
    return abs(e - 1.0) < 0.08 and c.y < BS_TOP - 0.003


def split(s, keep):
    """The faces of s for which keep(centre) holds, unused vertices dropped."""
    faces = [f for f in s.f if keep(sum((s.v[i] for i in f), Vector((0, 0, 0))) / len(f))]
    used = sorted({i for f in faces for i in f})
    remap = {o: n for n, o in enumerate(used)}
    return Shape([s.v[i] for i in used], [tuple(remap[i] for i in f) for f in faces])


BASIN = []


def basin_whole():
    if not BASIN:
        BASIN.append(basin())
    return BASIN[0]


def basin_body():
    return split(basin_whole(), lambda c: not in_bowl(c))


def basin_bowl():
    return split(basin_whole(), in_bowl)


def basin_trap():
    bx, bz = BS_BOWL
    zb = bz + 0.01
    yb = BS_TOP - 0.118                            # the bowl's floor at the plug
    flange = lathe([(0.0, 0.0), (0.0235, 0.0), (0.0235, 0.0012), (0.019, 0.0024), (0.0, 0.0024)], seg=20).move(bx, yb - 0.001, zb)
    grate = sum(box(0.034, 0.002, 0.003).move(bx, yb + 0.0005, zb + d) for d in (-0.008, 0.0, 0.008))
    tail = cyl(0.016, 0.016, 0.12, seg=14, bevel_r=0).move(bx, yb - 0.065, zb)
    nut = cyl(0.022, 0.022, 0.018, seg=14, bevel_r=0.002).move(bx, yb - 0.115, zb)
    body = cyl(0.030, 0.030, 0.13, seg=18, bevel_r=0.004).move(bx, yb - 0.19, zb)
    cap = cyl(0.026, 0.026, 0.02, seg=18, bevel_r=0.003).move(bx, yb - 0.262, zb)
    out_y = yb - 0.15
    pipe = tube([(bx, out_y, zb - 0.02), (bx, out_y, zb * 0.45), (bx, out_y, 0.004)], 0.016, seg=12)
    rose = cyl(0.032, 0.032, 0.006, seg=18, bevel_r=0.001).rot(math.pi / 2, 0, 0).move(bx, out_y, 0.003)
    return flange + grate + tail + nut + body + cap + pipe + rose


def pillar_tap(sx):
    x, z = sx * 0.08, BS_TAP_Z
    y = BS_TOP
    base = cyl(0.023, 0.025, 0.006, seg=18, bevel_r=0.002).move(x, y + 0.003, z)
    body = lathe([(0.0, 0.0), (0.0175, 0.0), (0.0165, 0.02), (0.014, 0.045), (0.0145, 0.055), (0.0175, 0.058),
                  (0.0175, 0.075), (0.013, 0.079), (0.0, 0.079)], seg=14).move(x, y + 0.006, z)
    spout = tube([(x, y + 0.040, z), (x, y + 0.046, z + 0.030), (x, y + 0.040, z + 0.055), (x, y + 0.028, z + 0.062)],
                 0.0075, seg=8, smooth=2)
    hub = cyl(0.007, 0.007, 0.012, seg=10, bevel_r=0.001).move(x, y + 0.091, z)
    arms = Shape()
    for a in (math.pi / 4, 3 * math.pi / 4):
        arm = rbox(0.064, 0.008, 0.009, r=0.0025, seg=1).rot(0, a + sx * 0.2, 0).move(x, y + 0.092, z)
        arms = arms + arm
    button = cyl(0.0075, 0.0075, 0.004, seg=12, bevel_r=0.0012).move(x, y + 0.099, z)
    return base + body + spout + hub + arms + button


def basin_taps():
    return pillar_tap(-1) + pillar_tap(1)


# ====================================================== the cracked mirror ===
CM_GW, CM_GH = 0.300, 0.400    # the glass seen
CM_MOULD = 0.024
CM_GLASS = 0.0080


def cracked_mirror_frame():
    prof = [(0.0, 0.0), (0.0, 0.0100), (0.0012, 0.0140), (0.0038, 0.0167), (0.0075, 0.0179), (0.0115, 0.0178),
            (0.0152, 0.0170), (0.0185, 0.0154), (0.0208, 0.0139), (0.0226, 0.0126), (0.0240, 0.0114),
            (0.0240, CM_GLASS), (0.0190, CM_GLASS), (0.0190, 0.0)]
    return rect_sweep(prof, CM_GW + 2 * CM_MOULD, CM_GH + 2 * CM_MOULD)


# =========================================================== the chalkboard ===
CB_W, CB_H = 0.400, 0.300
CB_MOULD = 0.022
CB_FACE = 0.0070               # the board's painted face


def chalkboard_frame():
    prof = [(0.0, 0.0), (0.0, 0.0115), (0.0007, 0.0131), (0.0024, 0.0140), (0.0196, 0.0140), (0.0213, 0.0131),
            (0.0220, 0.0115), (0.0220, CB_FACE),
            # the groove the board sits in
            (0.0160, CB_FACE), (0.0160, 0.0030), (0.0220, 0.0030), (0.0220, 0.0)]
    return rect_sweep(prof, CB_W, CB_H)


# ========================================================= the rustic sign ===
RS_L, RS_BH, RS_T = 0.300, 0.047, 0.014   # a board: length, width, thickness
RS_GAP = 0.0025
RS_BAT = 0.010                             # the battens' thickness (the boards stand on them)
RS_PITCH = RS_BH + RS_GAP
RS_EYE_X = 0.10
RS_NAIL_Y = 0.172                          # the wall nail over the boards' middle


def rustic_board(k, rng):
    """Board k (0 top): its own length, its ends sawn a little off square,
    its edges eased, cupped a millimetre, the sawn face lumpy."""
    L = RS_L + rng.uniform(-0.004, 0.004)
    s = slab(L, RS_BH, RS_T, nx=14, ny=3)
    # ends off square: shear the x of each vertex by its y
    skew = [rng.uniform(-0.05, 0.05), rng.uniform(-0.05, 0.05)]
    shift = rng.uniform(-0.003, 0.003)

    def warp(p):
        side = 0 if p.x < 0 else 1
        if abs(p.x) > L / 2 - 1e-6:
            p.x += skew[side] * p.y
        p.x += shift
        # cupped across its width (the face hollow), a twist along it
        p.z += -0.0012 * (1 - (2 * p.y / RS_BH) ** 2) + 0.0008 * (p.x / RS_L) * (p.y / RS_BH) * 2
        return p
    s = s.warp(warp)
    s = noisy(s, 0.0005, scale=60, seed=11 + k, axis=(0, 0, 1))
    s = bevel(s, 0.0016, seg=1, angle=50)
    return s.move(0, (1 - k) * RS_PITCH, RS_BAT + RS_T / 2)


def rustic_boards():
    rng = random.Random('rusticSignBoards')
    return sum(rustic_board(k, rng) for k in range(3))


def rustic_battens():
    h = 3 * RS_BH + 2 * RS_GAP - 0.012
    return sum(rbox(0.020, h, RS_BAT, r=0.0012, seg=1).move(x, -0.002, RS_BAT / 2) for x in (-0.105, 0.105))


def rustic_nails():
    rng = random.Random('rusticSignNails')
    s = Shape()
    for k in range(3):
        for x in (-0.105, 0.105):
            head = lathe([(0.0, 0.0), (0.0024, 0.0), (0.0023, 0.0005), (0.0015, 0.0009), (0.0, 0.0010)], seg=8)
            head = head.rot(math.pi / 2 + rng.uniform(-0.15, 0.15), 0, 0)
            # sunk a little proud or a little set, as a hammer leaves them
            head.move(x + rng.uniform(-0.004, 0.004), (1 - k) * RS_PITCH + rng.uniform(-0.006, 0.006), RS_BAT + RS_T - 0.0012 + rng.uniform(-0.0003, 0.0004))
            s = s + head
    return s


def eye_at(x):
    """A screw eye in the top board's top edge: its ring standing up, facing out."""
    ring = torus(0.0045, 0.0011, seg=14, tseg=6).move(0, 0.0068, 0)
    shank = cyl(0.0012, 0.0010, 0.008, seg=6, bevel_r=0).move(0, -0.0010, 0)
    top = RS_PITCH + RS_BH / 2
    return (ring + shank).move(x, top, RS_BAT + RS_T / 2)


def rustic_eyes():
    return eye_at(-RS_EYE_X) + eye_at(RS_EYE_X)


def twisted(points, r, seg=6, pitch=0.009):
    """A cord along `points` (already fine): three plies twisted round it
    (the radius swells three times round, the swell turning along it)."""
    s = tube(points, r, seg=seg, caps=True)
    # each ring's vertices: modulate their distance from the ring's centre
    n = len(points)
    L = 0.0
    lens = [0.0]
    for a, b in zip(points, points[1:]):
        L += (Vector(b) - Vector(a)).length
        lens.append(L)
    v = list(s.v)
    # tube() keeps the ring vertices in order (ring i: seg vertices)
    for i in range(min(n, len(v) // seg)):
        c = Vector((0, 0, 0))
        for j in range(seg):
            c += v[i * seg + j]
        c /= seg
        for j in range(seg):
            p = v[i * seg + j]
            a = 2 * math.pi * j / seg
            k = 1.0 + 0.16 * math.cos(3 * a - lens[i] / pitch * 2 * math.pi)
            v[i * seg + j] = c + (p - c) * k
    s.v = v
    return s


def rustic_cord():
    top = RS_PITCH + RS_BH / 2
    eye_y = top + 0.0100
    zc = RS_BAT + RS_T / 2
    nail = Vector((0.0, RS_NAIL_Y + 0.0045, 0.009))
    path = []
    # left eye up to the nail and down to the right eye, a slight sag either side
    for side, start in ((-1, Vector((-RS_EYE_X + 0.002, eye_y + 0.002, zc))), (1, Vector((RS_EYE_X - 0.002, eye_y + 0.002, zc)))):
        seg = []
        N = 30
        for i in range(N + 1):
            t = i / N
            p = start.lerp(nail + Vector((side * 0.004, -0.002, 0)), t)
            p.y -= 0.004 * math.sin(math.pi * t)       # it sags a little under its own weight
            seg.append(p)
        path.append(seg)
    left = path[0]
    right = list(reversed(path[1]))
    # over the nail: a short bend round its shank's top
    over = [nail + Vector((-0.004 + 0.008 * i / 4, 0.0015 * math.sin(math.pi * i / 4), 0)) for i in range(5)]
    line = left[:-1] + over + right[1:]
    s = twisted([tuple(p) for p in line], 0.0024, seg=6)
    # the knots at the eyes and their loose ends
    for sx in (-1, 1):
        k = ellipsoid(0.0042, 0.0050, 0.0040, seg=8).move(sx * (RS_EYE_X - 0.001), eye_y + 0.001, zc + 0.001)
        tail = twisted([(sx * (RS_EYE_X + 0.002), eye_y - 0.001, zc + 0.003), (sx * (RS_EYE_X + 0.006), eye_y - 0.010, zc + 0.006),
                        (sx * (RS_EYE_X + 0.008), eye_y - 0.020, zc + 0.006), (sx * (RS_EYE_X + 0.009), eye_y - 0.028, zc + 0.005)],
                       0.0021, seg=6)
        s = s + k + tail
    return s


def rustic_nail():
    head = cyl(0.0035, 0.0035, 0.0012, seg=10, bevel_r=0.0003).rot(math.pi / 2, 0, 0).move(0, 0, 0.0155)
    shank = cyl(0.0014, 0.0014, 0.015, seg=6, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0075)
    return (head + shank).rot(-0.12, 0, 0).move(0, RS_NAIL_Y, 0)


# ========================================================= the biohazard sign ===
BH_W, BH_H, BH_T = 0.210, 0.297, 0.0010
BH_HOLE = (0.090, 0.1335)          # the holes' centres (x, y): 15 mm in from each side


def biohazard_plate(lifted=False):
    # a grid of the outline's bounds (so a corner can spring), its outline
    # rounded by cutting the corners away, the four holes bored through
    s = slab(BH_W, BH_H, BH_T, nx=10, ny=14)
    cutters = []
    for sx in (-1, 1):
        for sy in (-1, 1):
            hx, hy = sx * BH_HOLE[0], sy * BH_HOLE[1]
            cutters.append(cyl(0.0026, 0.0026, 0.01, seg=10, bevel_r=0).rot(math.pi / 2, 0, 0).move(hx, hy, 0))
            # the rounded corner: what lies outside a 6 mm radius at the corner
            cx, cy = sx * (BH_W / 2 - 0.006), sy * (BH_H / 2 - 0.006)
            corner = cut(box(0.012, 0.012, 0.01).move(cx + sx * 0.006, cy + sy * 0.006, 0),
                         cyl(0.006, 0.006, 0.012, seg=12, bevel_r=0).rot(math.pi / 2, 0, 0).move(cx, cy, 0))
            cutters.append(corner)
    s = cut(s, *cutters)
    s = s.move(0, 0, BH_T / 2)
    if lifted:
        # the top right corner, its screw gone, sprung off the wall
        cx, cy = BH_W / 2, BH_H / 2

        def spring(p):
            u = max(0.0, (p.x + 0.02) / (BH_W / 2 + 0.02))
            v = max(0.0, (p.y + 0.05) / (BH_H / 2 + 0.05))
            p.z += 0.004 * (u * v) ** 2
            return p
        s = s.warp(spring)
    return s


def pan_screw():
    head = lathe([(0.0, 0.0), (0.0040, 0.0), (0.0040, 0.0008), (0.0034, 0.0020), (0.0020, 0.0026), (0.0, 0.0028)], seg=14)
    # the cross recess, a slot at a time
    return cut(head, box(0.0046, 0.004, 0.0009).move(0, 0.0034, 0), box(0.0009, 0.004, 0.0046).move(0, 0.0034, 0)).rot(math.pi / 2, 0, 0)


def biohazard_screws(skip=()):
    rng = random.Random('biohazardScrews')
    s = Shape()
    for sx in (-1, 1):
        for sy in (-1, 1):
            turn = rng.uniform(0, math.pi)
            if (sx, sy) in skip:
                continue
            s = s + pan_screw().rot(0, 0, turn).move(sx * BH_HOLE[0], sy * BH_HOLE[1], BH_T)
    return s


# ================================================================ the sampler ===
SP_GW, SP_GH = 0.170, 0.230     # the linen seen
SP_MOULD = 0.025
SP_LINEN = 0.0080


def sampler_frame():
    k = SP_MOULD / 0.030            # the scoop drawn for 30 mm, at 25
    prof = [(0.0, 0.0), (0.0, 0.0120), (0.0010, 0.0153), (0.0030, 0.0174), (0.0060, 0.0185), (0.0100, 0.0187),
            (0.0140, 0.0178), (0.0180, 0.0163), (0.0220, 0.0146), (0.0255, 0.0134), (0.0280, 0.0130),
            (0.0293, 0.0128), (0.0300, 0.0119), (0.0300, SP_LINEN)]
    prof = [(d * k, z) for d, z in prof] + [(SP_MOULD - 0.005, SP_LINEN), (SP_MOULD - 0.005, 0.0)]
    return rect_sweep(prof, SP_GW + 2 * SP_MOULD, SP_GH + 2 * SP_MOULD)


# ---------------------------------------------------------------- parts ---
auto('Real__wallMirrorFrameWood', wall_mirror_wood, 400)
auto('Real__wallMirrorFrameGilt', wall_mirror_gilt, 400)
auto('Real__wallMirrorPlates', wall_mirror_plates, 800)
auto('Real__wallMirrorBasin', basin_body, 2000)
auto('Real__wallMirrorBasinBowl', basin_bowl, 1200)
auto('Real__wallMirrorBasinTrap', basin_trap, 1500)
auto('Real__wallMirrorBasinTaps', basin_taps, 2400)
auto('Real__crackedMirrorFrame', cracked_mirror_frame, 260)
auto('Real__chalkboardFrame', chalkboard_frame, 200)
auto('Real__rusticSignBoards', rustic_boards, 1600)
auto('Real__rusticSignBattens', rustic_battens, 120)
auto('Real__rusticSignNails', rustic_nails, 300)
auto('Real__rusticSignEyes', rustic_eyes, 400)
auto('Real__rusticSignCord', rustic_cord, 1300)
auto('Real__rusticSignNail', rustic_nail, 160)
auto('Real__biohazardPlate', biohazard_plate, 900)
auto('Real__biohazardPlateLifted', lambda: biohazard_plate(True), 900)
auto('Real__biohazardScrews', biohazard_screws, 800)
auto('Real__biohazardScrews3', lambda: biohazard_screws(skip=((1, 1),)), 600)
auto('Real__samplerFrame', sampler_frame, 200)
