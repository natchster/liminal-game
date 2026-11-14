"""kitchen_and_hearth_left_behind (was r16): the kitchen and the hearth left behind, remodelled at their real
sizes (docs/design/PROPS.md, P2): a wine glass with the last of its wine dried in
it, an enamel stovetop kettle, a cast-iron skillet hung on a nail, a bloomer
gone stale and mouldy on its board, a beer bottle broken on the floor, a
portable sewing machine, a little stack of split firewood. Sources and sizes:
docs/research/props-remodelling/kitchen_and_hearth_left_behind.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
prop's own frame: x across its front, y up from the floor (y = 0 the floor
it rests on), z out of its front; a wall prop's wall is z = 0. Parts named
in CYL_FRAMED take the cylinder's texture coordinates (u round the y axis
through the middle of their bounds, v up), the rest the box's.

wineGlassBowl    a stemmed 8 oz wine glass standing (Libbey Citation 8464
                 class: 171 mm tall, the bowl 73 mm across, the rim 70, the
                 foot 64): its bowl, with the thicker button at its bottom
                 and the rim fire-polished round
wineGlassFoot    its foot, domed under, and the drawn stem
wineGlassLying   the same glass lying on its side, on its foot's rim and
                 its bowl's belly
wineGlassDregs   what is left of the wine in its bowl: a dark lens, dried
                 thick at its edge
bottleBase       a 12 oz longneck's base (231 x 61 mm whole): the push-up,
                 the stippled heel ring, the wall broken off in teeth
bottleNeck       its neck: the crown finish and the shoulder, broken off
                 below the shoulder, lying on its side
bottleShardA..E  pieces of its wall (3 mm glass on its 30.5 mm radius):
                 A, E lying convex up, B concave up on its belly, C standing
                 on edge, D a small one
bottleChips      the glitter of small chips round where it burst
kettleBody       a 2 l enamel stovetop kettle's body and lid (170 mm across
                 the base, 136 mm to the lid's top): the chime at its foot,
                 the shoulder, the lid seat's rolled bead, the domed lid,
                 the handle's two ears; a dent in its side
kettleSpout      its spout, seamed on low and rising, open at its mouth
kettleBail       the wire bail of its handle, hooked through its ears
kettleGrip       the bail's grip and the lid's knob (phenolic, or turned wood)
kettleTrivet     a round cork trivet under it
panBody          a 10 1/4 in cast-iron skillet (Lodge L8SK3 class) hung on
                 the wall by its handle: the bottom against the wall, its
                 two pour lips, the helper handle, the hanging hole
panNail          the 16d common nail it hangs from, driven into the wall
breadBoard       a beech bread board, 320 x 220 x 18 mm, its edges rounded
breadCrust       a 400 g bloomer, 220 x 115 x 88 mm, five slashes across
                 its top opened into ears, a heel cut off; stale: shrunk
                 and cracked
breadCrumb       its cut face, gone dry and caved a little
breadCrumbs      crumbs and a broken corner of crust on the board
sewBed           a portable sewing machine's bed (Singer 221 class: 378 x
                 184 x 248 mm overall, no maker's name of its own): the cast
                 bed, its skirt and rubber feet
sewArm           its pillar, arm and head, and the motor on its back
sewSteel         its bright parts: the face plate, needle and slide plates,
                 the needle bar and needle, the presser bar and foot, the
                 thread guides, tension, take-up lever, bobbin winder,
                 spool pin, stitch lever
sewWheel         the handwheel, centred on its axle (x), so it can turn
sewWheelKnob     its stop-motion knob, on the same axle
sewSpool         a spool of thread on the spool pin
logA*, logB*, logC*, logD*   split firewood, 300 mm stove lengths, each
                 in three parts: Bark (its bark), Split (the faces it was
                 split along) and Ends (the sawn end grain). A a quarter of
                 a 220 mm round, B half of a 130 mm round, C a slab off a
                 340 mm round, D a whole birch round of 80 mm. Lying along
                 x, the pith at z = 0 (so the ends' rings centre on it)
"""
from propkit import *
import propkit

CYL_FRAMED = {'Real__wineGlassBowl', 'Real__wineGlassFoot', 'Real__wineGlassDregs', 'Real__kettleBody', 'Real__kettleTrivet'}


# ------------------------------------------------------------------ util --
def frac(x):
    return x - math.floor(x)


def on_floor(s, y0=0.0):
    lo, hi = s.bounds()
    return s.move(0, y0 - lo.y, 0)


def centre_xz(s):
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2, 0, -(lo.z + hi.z) / 2)


def grid_surface(fn, nu, nv, wrap_u=False):
    """A surface from fn(u, v) -> Vector over u in [0, 1] (nu steps, wrapped
    if wrap_u) and v in [0, 1] (nv steps): an open grid of quads."""
    cols = nu if wrap_u else nu + 1
    v, f = [], []
    for j in range(nv + 1):
        for i in range(cols):
            v.append(Vector(fn(i / nu, j / nv)))
    for j in range(nv):
        for i in range(nu):
            a = j * cols + i
            b = j * cols + (i + 1) % cols if wrap_u else a + 1
            f.append((a, b, b + cols, a + cols))
    return Shape(v, f)


def faces_where(s, keep):
    """The faces of s for which keep(centre, normal) holds, as a shape of
    their own (unused vertices dropped)."""
    out_v, out_f, remap = [], [], {}
    for f in s.f:
        pts = [s.v[i] for i in f]
        c = sum(pts, Vector((0, 0, 0))) / len(pts)
        n = (pts[1] - pts[0]).cross(pts[2] - pts[0])
        if len(pts) > 3:
            n = Vector((0, 0, 0))
            for k in range(len(pts)):
                a, b = pts[k], pts[(k + 1) % len(pts)]
                n += Vector(((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y)))
        n = n.normalized() if n.length > 1e-12 else Vector((0, 1, 0))
        if not keep(c, n):
            continue
        nf = []
        for i in f:
            if i not in remap:
                remap[i] = len(out_v)
                out_v.append(s.v[i].copy())
            nf.append(remap[i])
        out_f.append(tuple(nf))
    return Shape(out_v, out_f)


def ring_cutter(hfun, above=True, seg=72, far=0.6):
    """A solid filling everything above (or below) the height hfun(a) round
    the y axis (a in radians from +z toward +x), out to 6 cm: a bottle cut
    along it is broken there. Its middle is at the mean height, its rings
    from 18 mm out follow hfun, so a wall between 18 and 60 mm is cut on
    the line."""
    hs = [hfun(TAU * i / seg) for i in range(seg)]
    hc = sum(hs) / seg
    yf = far if above else -far
    v = [Vector((0, hc, 0))]
    for r in (0.018, 0.06):
        for i in range(seg):
            a = TAU * i / seg
            v.append(Vector((r * math.sin(a), hs[i], r * math.cos(a))))
    for i in range(seg):
        a = TAU * i / seg
        v.append(Vector((0.06 * math.sin(a), yf, 0.06 * math.cos(a))))
    v.append(Vector((0, yf, 0)))
    f = []
    r1, r2, top, ct = 1, 1 + seg, 1 + 2 * seg, 1 + 3 * seg
    for i in range(seg):
        j = (i + 1) % seg
        f.append((0, r1 + i, r1 + j))
        f.append((r1 + i, r2 + i, r2 + j, r1 + j))
        f.append((r2 + i, top + i, top + j, r2 + j))
        f.append((top + i, ct, top + j))
    return propkit._fix_winding(Shape(v, f))


def jag(rng, n, lo, hi, sharp=0.35):
    """A broken line's heights round a circle: n points between lo and hi,
    between each two a tooth or a notch; returned as hfun(a)."""
    pts = []
    for i in range(n):
        a = TAU * (i + rng.uniform(-0.3, 0.3)) / n
        pts.append((a % TAU, rng.uniform(lo, hi)))
        # a tooth between this point and the next
        a2 = TAU * (i + 0.5 + rng.uniform(-0.15, 0.15)) / n
        mid = rng.uniform(lo, hi)
        pts.append((a2 % TAU, mid + rng.choice((-1, 1)) * sharp * (hi - lo)))
    pts.sort()

    def h(a):
        a %= TAU
        for k in range(len(pts)):
            a0, h0 = pts[k - 1]
            a1, h1 = pts[k]
            if k == 0:
                a0 -= TAU
            if a0 <= a <= a1:
                t = (a - a0) / max(1e-9, a1 - a0)
                return min(hi, max(lo, h0 + (h1 - h0) * t))
        a0, h0 = pts[-1]
        a1, h1 = pts[0][0] + TAU, pts[0][1]
        t = (a - a0) / max(1e-9, a1 - a0)
        return min(hi, max(lo, h0 + (h1 - h0) * t))
    return h


# ============================================================ wine glass ===
# Libbey Citation 8464 (8 oz): 6 3/4 in (171 mm) tall, 2 3/4 in (70 mm)
# across the rim, 2 7/8 in (73 mm) at its widest, the foot 2 1/2 in (64 mm).
GLASS_PROFILE = [
    # the foot: domed up underneath, its edge fire-rounded, rising to the stem
    (0.0, 0.0034), (0.016, 0.0024), (0.0295, 0.0003), (0.0318, 0.0007), (0.0318, 0.0024),
    (0.0285, 0.0036), (0.016, 0.0058), (0.0068, 0.0098),
    # the drawn stem
    (0.0046, 0.0150), (0.0040, 0.0400), (0.0043, 0.0680), (0.0062, 0.0770),
    # the bowl, its thick button at the bottom, widest a little above half
    (0.0110, 0.0835), (0.0200, 0.0915), (0.0280, 0.1030), (0.0335, 0.1180),
    (0.0362, 0.1350), (0.0360, 0.1530), (0.0350, 0.1695),
    # the rim, round
    (0.0346, 0.1710), (0.0339, 0.1700),
    # the inside, 1.1 mm in
    (0.0340, 0.1640), (0.0349, 0.1530), (0.0351, 0.1350), (0.0325, 0.1190),
    (0.0270, 0.1050), (0.0190, 0.0945), (0.0100, 0.0878), (0.0, 0.0858),
]
GLASS_TIDE = (0.100, 0.122)     # where the wine stood (the old fill level's range)
DREGS_TOP = 0.0905


GLASS_JOIN = GLASS_PROFILE.index((0.0062, 0.0770))


def wine_glass():
    return lathe(GLASS_PROFILE, seg=26)


def wine_glass_bowl():
    """The bowl, from where the stem flares into it: its own part, so the
    wine's marks on it map round it (u) and up it (v) alone."""
    return lathe([(0.0, 0.0770)] + GLASS_PROFILE[GLASS_JOIN:], seg=28)


def wine_glass_foot():
    """The foot and the stem, up to the bowl."""
    return lathe(GLASS_PROFILE[:GLASS_JOIN + 1] + [(0.0, 0.0770)], seg=24)


def _glass_inner_r(y):
    inner = [(r, yy) for r, yy in GLASS_PROFILE[GLASS_PROFILE.index((0.0339, 0.1700)):]]
    inner = sorted(inner, key=lambda p: p[1])
    for (r0, y0), (r1, y1) in zip(inner, inner[1:]):
        if y0 <= y <= y1:
            return r0 + (r1 - r0) * (y - y0) / max(1e-9, y1 - y0)
    return inner[-1][0]


def wine_glass_dregs():
    """The wine gone down to a dark lens in the bottom of the bowl: its
    surface flat, its edge drawn up the glass a little where it dried."""
    prof = [(0.0, 0.0861)]
    for y in (0.0870, 0.0880, 0.0892, DREGS_TOP - 0.0004):
        prof.append((_glass_inner_r(y) - 0.0002, y))
    prof.append((_glass_inner_r(DREGS_TOP) - 0.0003, DREGS_TOP + 0.0005))
    prof.append((_glass_inner_r(DREGS_TOP) - 0.0012, DREGS_TOP))
    prof.append((0.0, DREGS_TOP - 0.0001))
    return lathe(prof, seg=26)


def wine_glass_lying():
    """The glass on its side: it rests on the rim of its foot and the belly
    of its bowl, the axis a little down toward the bowl; the stem in the air."""
    s = wine_glass()
    # lay it along +x (the foot at -x, the mouth at +x)
    s.rot(z=-math.pi / 2)
    # the two points it rests on: the foot's rim (x 0, r 0.0318) and the
    # bowl's belly (x 0.135, r 0.0362)
    tilt = math.atan2(0.0362 - 0.0318, 0.135)
    s.rot(z=-tilt)
    s = on_floor(s)
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2, 0, 0)


# ======================================================== broken bottle ===
# A 12 oz (355 ml) longneck: 231 mm tall, 61 mm across, the crown finish
# for a 26 mm cap; its wall about 3 mm, its base thicker.
BOTTLE_OUT = [
    (0.0, 0.0062), (0.010, 0.0055), (0.019, 0.0030), (0.0235, 0.0005), (0.0255, 0.0),
    (0.0285, 0.0018), (0.0302, 0.0060), (0.0305, 0.0130), (0.0305, 0.0500), (0.0305, 0.0900),
    (0.0305, 0.1250), (0.0300, 0.1360), (0.0278, 0.1480), (0.0230, 0.1620), (0.0175, 0.1760),
    (0.0140, 0.1880), (0.0128, 0.2000), (0.0124, 0.2140), (0.0127, 0.2160), (0.0136, 0.2175),
    (0.0137, 0.2230), (0.0133, 0.2250), (0.0136, 0.2268), (0.0134, 0.2300), (0.0122, 0.2312),
]
BOTTLE_IN = [
    (0.0105, 0.2300), (0.0098, 0.2250), (0.0090, 0.2140), (0.0092, 0.2000), (0.0105, 0.1880),
    (0.0145, 0.1760), (0.0200, 0.1620), (0.0248, 0.1480), (0.0270, 0.1360), (0.0275, 0.1250),
    (0.0275, 0.0900), (0.0275, 0.0500), (0.0274, 0.0160), (0.0262, 0.0100), (0.0225, 0.0080),
    (0.0150, 0.0105), (0.0, 0.0128),
]


def bottle_whole(seg=32):
    return lathe(BOTTLE_OUT + BOTTLE_IN, seg=seg)


def bottle_base():
    """The base, stood where it fell: its wall broken off in four or five
    teeth, 22 to 62 mm up."""
    rng = random.Random(51401)
    h = jag(rng, 5, 0.022, 0.062, sharp=0.45)
    s = cut(bottle_whole(36), ring_cutter(h, above=True))
    return s


def bottle_neck():
    """The neck and shoulder in one piece, broken off below the shoulder,
    lying on its side on the shoulder's edge and the finish."""
    rng = random.Random(51402)
    h = jag(rng, 5, 0.120, 0.150, sharp=0.4)
    s = cut(bottle_whole(32), ring_cutter(h, above=False))
    # lay it down: its axis along x, the finish at +x
    s.rot(z=-math.pi / 2)
    s.rot(z=-math.atan2(0.0305 - 0.0137, 0.10))
    s = on_floor(s)
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2, 0, -(lo.z + hi.z) / 2)


def _shard(outline, seed, pose):
    """A piece of the bottle's wall inside `outline` (x across, y up, seen
    from the front), its edges the breaks: laid convex up ('up'), on its
    belly ('belly') or standing on an edge ('edge')."""
    rng = random.Random(seed)
    pts = [(x + rng.uniform(-0.0015, 0.0015), y + rng.uniform(-0.0015, 0.0015)) for x, y in outline]
    cutter = prism(pts, 0.06).move(0, 0, 0.03)
    wall = bottle_whole(48)
    s = boolean(wall, cutter, 'INTERSECT')
    lo, hi = s.bounds()
    s.move(-(lo.x + hi.x) / 2, -(lo.y + hi.y) / 2, 0)
    if pose == 'up':
        s.rot(x=-math.pi / 2)
    elif pose == 'belly':
        s.rot(x=math.pi / 2)
    else:
        s.rot(x=-math.pi / 2)
        s.rot(z=1.2)
    s.rot(y=rng.uniform(0, TAU))
    s = centre_xz(s)
    return on_floor(s)


def bottle_shard_a():
    return _shard([(-0.020, 0.060), (0.004, 0.052), (0.022, 0.071), (0.012, 0.104), (-0.006, 0.112), (-0.018, 0.090)], 51410, 'up')


def bottle_shard_b():
    return _shard([(-0.016, 0.070), (0.018, 0.064), (0.020, 0.088), (0.002, 0.098), (-0.019, 0.086)], 51411, 'belly')


def bottle_shard_c():
    return _shard([(-0.012, 0.075), (0.014, 0.080), (0.009, 0.118), (-0.006, 0.108)], 51412, 'edge')


def bottle_shard_d():
    return _shard([(-0.008, 0.090), (0.010, 0.086), (0.006, 0.104), (-0.006, 0.101)], 51413, 'up')


def bottle_shard_e():
    return _shard([(-0.024, 0.040), (-0.004, 0.034), (0.010, 0.060), (0.004, 0.122), (-0.010, 0.126), (-0.020, 0.096)], 51414, 'up')


def bottle_chips():
    """Small chips and slivers spread where it burst: flat bits of 3 mm
    glass from 3 to 11 mm across, most of them near the middle."""
    rng = random.Random(51420)
    s = Shape()
    for k in range(26):
        n = rng.choice((3, 4, 4, 5))
        size = rng.uniform(0.0015, 0.0055) * (1.0 if k % 4 else 1.6)
        pts = []
        for i in range(n):
            a = TAU * (i + rng.uniform(-0.3, 0.3)) / n
            r = size * rng.uniform(0.6, 1.2)
            pts.append((r * math.cos(a), r * math.sin(a) * rng.uniform(0.4, 1.0)))
        chip = prism(pts, rng.uniform(0.0012, 0.0028)).rot(x=-math.pi / 2)
        chip.rot(x=rng.uniform(-0.12, 0.12), z=rng.uniform(-0.12, 0.12))
        chip.rot(y=rng.uniform(0, TAU))
        rr = 0.17 * math.sqrt(rng.uniform(0.0, 1.0)) ** 1.3
        a = rng.uniform(0, TAU)
        chip = on_floor(chip)
        chip.move(rr * math.cos(a), 0, rr * math.sin(a))
        s = s + chip
    return s


# ================================================================ kettle ===
# A 2 l enamel stovetop kettle (the Kamille / Emalia 2.5 l class is 190 mm
# across and 215 mm to its handle's top): pressed steel, enamelled, a wire
# bail through two ears, a phenolic grip and knob. Its base 170 mm across,
# the lid's top 137 mm, the grip's top 216 mm; the spout to +x.
KETTLE_BODY = [
    (0.0, 0.0045), (0.0720, 0.0045), (0.0790, 0.0034), (0.0830, 0.0009), (0.0848, 0.0),
    (0.0858, 0.0022), (0.0862, 0.0080), (0.0868, 0.0300), (0.0862, 0.0500), (0.0836, 0.0680),
    (0.0782, 0.0850), (0.0698, 0.0990), (0.0602, 0.1100), (0.0528, 0.1162), (0.0492, 0.1192),
    (0.0489, 0.1228), (0.0472, 0.1247), (0.0452, 0.1236), (0.0446, 0.1200), (0.0440, 0.1160),
    (0.0, 0.1150),
]
KETTLE_LID = [
    (0.0, 0.1185), (0.0420, 0.1185), (0.0428, 0.1240), (0.0494, 0.1250), (0.0502, 0.1268),
    (0.0470, 0.1292), (0.0350, 0.1336), (0.0180, 0.1360), (0.0, 0.1367),
]
KETTLE_DENT = (-1.9, 0.052, 0.030, 0.0045)     # angle (from +z toward +x), height, radius, depth
KETTLE_SPOUT = [(0.068, 0.030), (0.082, 0.035), (0.095, 0.048), (0.107, 0.066), (0.117, 0.086),
                (0.124, 0.101), (0.1285, 0.1105)]


def _dent(p, a0, y0, rad, depth):
    a = math.atan2(p.x, p.z)
    da = (a - a0 + math.pi) % TAU - math.pi
    r = math.hypot(p.x, p.z)
    d = math.hypot(da * r, p.y - y0)
    if d > rad or r < 1e-6:
        return p
    w = 0.5 + 0.5 * math.cos(math.pi * d / rad)
    k = (r - depth * w) / r
    return Vector((p.x * k, p.y, p.z * k))


def kettle_body():
    body = lathe(KETTLE_BODY, seg=40)
    body.warp(lambda p: _dent(p, *KETTLE_DENT))
    body.warp(lambda p: _dent(p, 2.6, 0.012, 0.016, 0.0025))
    lid = lathe(KETTLE_LID, seg=36)
    # the bail's ears, riveted to the shoulder at +-x, a hole for the wire
    ears = Shape()
    for sx in (-1, 1):
        ear = rbox(0.0028, 0.024, 0.018, r=0.0011, seg=1)
        ear = cut(ear, cyl(0.0032, h=0.02, seg=10).rot(z=math.pi / 2).move(0, 0.0045, 0))
        ear.rot(z=sx * 0.55)
        ear.move(sx * 0.0655, 0.1175, 0)
        rivet = cyl(0.0026, h=0.004, seg=8).rot(z=math.pi / 2).move(sx * 0.0675, 0.1095, 0)
        ears = ears + ear + rivet
    return body + lid + ears


def kettle_spout():
    pts = [Vector((x, y, 0)) for x, y in KETTLE_SPOUT]
    s = tube(pts, 0.0175, seg=16, smooth=2, r_end=0.0078)
    tip, prev = pts[-1], pts[-2]
    d = (tip - prev).normalized()
    hole = cyl(0.0064, h=0.03, seg=14, bevel_r=0)
    hole.apply(Vector((0, 1, 0)).rotation_difference(d).to_matrix().to_4x4())
    hole.move(*tip)
    return cut(s, hole)


def kettle_bail():
    pts = [(-0.056, 0.1220), (-0.0680, 0.1220), (-0.0720, 0.1300), (-0.0715, 0.1600), (-0.0620, 0.1880),
           (-0.0440, 0.2030), (-0.0300, 0.2060), (0.0300, 0.2060), (0.0440, 0.2030), (0.0620, 0.1880),
           (0.0715, 0.1600), (0.0720, 0.1300), (0.0680, 0.1220), (0.056, 0.1220)]
    return tube([Vector((x, y, 0)) for x, y in pts], 0.00225, seg=8, smooth=1)


def kettle_grip():
    prof = [(0.0, -0.035), (0.0068, -0.035), (0.0092, -0.031), (0.0108, -0.020), (0.0113, 0.0),
            (0.0108, 0.020), (0.0092, 0.031), (0.0068, 0.035), (0.0, 0.035)]
    grip = lathe(prof, seg=16).rot(z=math.pi / 2).move(0, 0.2060, 0)
    knob = lathe([(0.0, 0.1360), (0.0085, 0.1360), (0.0094, 0.1385), (0.0074, 0.1430), (0.0102, 0.1470),
                  (0.0121, 0.1515), (0.0112, 0.1560), (0.0065, 0.1584), (0.0, 0.1588)], seg=18)
    return grip + knob


def kettle_trivet():
    """A round cork trivet, 200 mm across and 10 mm thick, its edge eased."""
    return lathe([(0.0, 0.0), (0.0965, 0.0), (0.1000, 0.0025), (0.1000, 0.0075), (0.0965, 0.0100), (0.0, 0.0100)], seg=40)


# ========================================================= cast-iron pan ===
# Lodge L8SK3: 10 1/4 in across the top (260 mm), 2 in deep, 16 1/8 in
# long with both handles; the bottom 8 in. Built in its pan frame (its axis
# +y, the handle +x), then turned to hang: the bottom flat on the wall
# (z = 0), the cooking face out (+z), the handle up (+y).
PAN_PROFILE = [
    (0.0, 0.0), (0.0940, 0.0), (0.0985, 0.0012), (0.1008, 0.0042), (0.1080, 0.0180), (0.1200, 0.0360),
    (0.1300, 0.0490), (0.1318, 0.0508), (0.1308, 0.0526), (0.1284, 0.0529), (0.1266, 0.0510),
    (0.1180, 0.0360), (0.1050, 0.0170), (0.0970, 0.0076), (0.0915, 0.0056), (0.0, 0.0052),
]
PAN_HOLE = (0.2500, 0.0079)     # the hanging hole: along the handle, radius
PAN_RISE = 0.17                 # the handle rises 10 degrees from the rim


def _pan_lips(p):
    """The two pour lips, at right angles to the handle: the rim drawn out
    and down there."""
    if p.y < 0.02:
        return p
    a = math.atan2(p.x, p.z)
    w = 0.0
    for a0 in (0.0, math.pi):
        da = (a - a0 + math.pi) % TAU - math.pi
        w = max(w, math.exp(-(da / 0.20) ** 2))
    w *= min(1.0, max(0.0, (p.y - 0.02) / 0.03)) ** 1.5
    r = math.hypot(p.x, p.z)
    k = (r + 0.0075 * w) / max(r, 1e-6)
    return Vector((p.x * k, p.y - 0.0026 * w, p.z * k))


def _pan_to_wall(p):
    """The pan frame (axis +y, handle +x) to hanging (axis +z, handle +y)."""
    return Vector((p.z, p.x, p.y))


def _handle_outline(x0, x1, w0, w1, rend):
    pts = [(x0, -w0), (x0 + 0.03, -w1 - 0.0015), (x1 - rend, -w1)]
    for i in range(1, 8):
        a = -math.pi / 2 + math.pi * i / 8
        pts.append((x1 - rend + rend * 1.08 * math.cos(a), rend * math.sin(a)))
    pts += [(x1 - rend, w1), (x0 + 0.03, w1 + 0.0015), (x0, w0)]
    return pts


def pan_body():
    s = lathe(PAN_PROFILE, seg=56)
    s.warp(_pan_lips)
    # the handle: tapering, its end round about the hole, 11 mm thick, its
    # edges eased; rising from the rim
    h = prism(_handle_outline(0.122, 0.2635, 0.0175, 0.0125, 0.0135), 0.011, bevel_r=0.0022, bevel_seg=1)
    h.rot(x=math.pi / 2)
    hx, hr = PAN_HOLE
    h = cut(h, cyl(hr, h=0.04, seg=14, bevel_r=0).move(hx, 0, 0))
    h.move(-0.122, 0, 0).rot(z=PAN_RISE).move(0.122, 0.0465, 0)
    # the helper handle: a lug with a slot for the fingers
    g = prism(_handle_outline(0.122, 0.168, 0.026, 0.023, 0.020), 0.008, bevel_r=0.0018, bevel_seg=1)
    g.rot(x=math.pi / 2)
    g = cut(g, prism(rounded_rect(0.024, 0.014, 0.006, seg=3), 0.05).rot(x=math.pi / 2).move(0.150, 0, 0))
    g.scale(-1, 1, 1)
    g.move(0.122, 0, 0).rot(z=-0.09).move(-0.122, 0.0478, 0)
    out = s + h + g
    return out.warp(_pan_to_wall)


def pan_hole_centre():
    hx, _ = PAN_HOLE
    c = Vector((hx - 0.122, 0, 0))
    c = Matrix.Rotation(PAN_RISE, 3, 'Z') @ c + Vector((0.122, 0.0465, 0))
    return _pan_to_wall(c)


def pan_nail():
    """A 16d common nail (3 1/2 in, its shank 4.1 mm, head 8.7 mm) driven
    into the wall a little up from square, 76 mm of it left out; the
    handle's hole rests on it."""
    c = pan_hole_centre()
    ln = 0.080
    shank = cyl(0.00205, h=ln + 0.006, seg=8, bevel_r=0).move(0, (ln + 0.006) / 2 - 0.006, 0)
    head = lathe([(0.0, ln), (0.00435, ln), (0.00435, ln + 0.0012), (0.0030, ln + 0.0020), (0.0, ln + 0.0022)], seg=12)
    n = (shank + head).rot(x=math.pi / 2 - 0.10)          # along +z, its end a little up
    # its top under the top of the hole, where the hole's edge bears on it
    at = c.z
    y_at = at * math.tan(0.10)
    return n.move(c.x, c.y + PAN_HOLE[1] - 0.00205 - y_at, 0.0)


# ================================================================ bread ===
# A 400 g white bloomer (the 800 g one is 265 x 150 x 120 mm; the small loaf
# 220 x 115 x 88), five slashes across its top, stale and shrunk, its end
# cut off; on a beech bread board 320 x 220 x 18 mm.
BOARD = (0.320, 0.018, 0.220)
LOAF_L, LOAF_W, LOAF_H = 0.214, 0.110, 0.084
LOAF_X0 = -0.035                     # the loaf's middle along the board
LOAF_CUT = 0.068                     # where it was cut, from its middle (+x)


def bread_board():
    w, t, d = BOARD
    b = rbox(w, t, d, r=0.0035, seg=2).move(0, t / 2, 0)
    # a hanging hole at one end, as boards have
    b = cut(b, cyl(0.007, h=0.05, seg=14, bevel_r=0).move(-w / 2 + 0.022, 0, 0))
    return b


def _loaf_section(u):
    """The loaf's half width and height at u (0 its round end, 1 the other)."""
    e = max(0.0, 1.0 - abs(2 * u - 1) ** 2.6) ** 0.5
    return LOAF_W / 2 * (0.22 + 0.78 * e), LOAF_H * (0.30 + 0.70 * e)


SLASHES = [(-0.074, 0.55), (-0.040, 0.58), (-0.006, 0.54), (0.028, 0.57), (0.062, 0.55)]


def _loaf_point(u, v):
    """u along the loaf (x), v round its section from the middle of its
    foot: a dome on a flat foot, the slashes cut into its top."""
    x = LOAF_X0 - LOAF_L / 2 + LOAF_L * u
    hw, hh = _loaf_section(u)
    t = TAU * v
    c, s = math.cos(t), math.sin(t)
    # a superellipse, its foot flattened where it baked on the tray
    z = hw * math.copysign(abs(s) ** 0.80, s)
    y = hh * (0.38 - 0.62 * math.copysign(abs(c) ** 0.90, c))
    e = 0.006
    y = 0.5 * (y + math.sqrt(y * y + e * e))
    # the slashes: each a groove across the top at an angle, with its ear
    top = max(0.0, (y - hh * 0.55) / (hh * 0.45))
    for xs, ang in SLASHES:
        dx, dz = x - (xs + LOAF_X0), z
        along = dx * math.cos(ang) + dz * math.sin(ang)
        across = -dx * math.sin(ang) + dz * math.cos(ang)
        if abs(across) > 0.050:
            continue
        fade = max(0.0, 1.0 - (abs(across) / 0.050) ** 2)
        g = math.exp(-(along / 0.0062) ** 2) * 0.0085
        ear = math.exp(-((along - 0.0090) / 0.0045) ** 2) * 0.0026
        y += (ear - g) * fade * top
    q = Vector((x, y, z))
    # stale: shrunk, lumpy where the crust dried unevenly
    y += 0.0008 * noise.noise(q * 45.0 + Vector((3.1, 7.7, 1.3))) * min(1.0, y / 0.01)
    return Vector((x, y, z))


def bread_crust():
    ucut = (LOAF_L / 2 + LOAF_CUT) / LOAF_L
    nu, nv = 56, 32
    s = grid_surface(lambda u, v: _loaf_point(0.012 + (ucut - 0.012) * u, v), nu, nv, wrap_u=False)
    # the grid runs u along x; rows are v; close the round end with a fan
    cols = nu + 1
    ring = [j * cols for j in range(nv + 1)][:-1]
    end = Vector((LOAF_X0 - LOAF_L / 2 + 0.0005, max(0.02, sum(s.v[i].y for i in ring) / len(ring)), 0.0))
    s.v.append(end)
    c = len(s.v) - 1
    for a, b in zip(ring, ring[1:] + ring[:1]):
        s.f.append((b, a, c))
    s = weld(s, 1e-7)
    s = propkit._fix_winding(s)
    return s.move(0, BOARD[1] - 0.0004, 0)


def bread_crumb():
    """The cut face: the loaf's section where it was cut, its crumb dried
    and caved in a few millimetres inside the crust."""
    ucut = (LOAF_L / 2 + LOAF_CUT) / LOAF_L
    nv = 32
    rim = [_loaf_point(ucut, j / nv) for j in range(nv)]
    cy = sum(p.y for p in rim) / nv
    centre = Vector((rim[0].x, cy, 0.0))
    v, f = [], []
    rings = [1.0, 0.92, 0.7, 0.45, 0.2]
    for k in rings:
        for p in rim:
            q = centre + (p - centre) * k
            q.x -= 0.0035 * (1 - k * k) + (0.0006 if k < 1 else 0.0)
            q.x += 0.0008 * noise.noise(q * 90.0) * (k < 1)
            v.append(q)
    v.append(Vector((centre.x - 0.0036, centre.y, centre.z)))
    for r in range(len(rings) - 1):
        for j in range(nv):
            a, b = r * nv + j, r * nv + (j + 1) % nv
            f.append((a, b, b + nv, a + nv))
    last = (len(rings) - 1) * nv
    for j in range(nv):
        f.append((last + j, last + (j + 1) % nv, len(v) - 1))
    s = Shape(v, f)
    # it faces out of the loaf (+x)
    nsum = Vector((0, 0, 0))
    for fc in s.f:
        a, b, c = (s.v[i] for i in fc[:3])
        nsum += (b - a).cross(c - a)
    if nsum.x < 0:
        s.f = [tuple(reversed(fc)) for fc in s.f]
    return s.move(0, BOARD[1] - 0.0004, 0)


def bread_crumbs():
    """Crumbs on the board by the cut end, and a broken corner of crust."""
    rng = random.Random(16301)
    s = Shape()
    x0 = LOAF_X0 + LOAF_CUT
    for k in range(18):
        r = rng.uniform(0.0012, 0.0035) * (2.2 if k == 0 else 1.0)
        c = sphere(r, seg=6, rings=4)
        c = noisy(c, r * 0.35, scale=900.0, seed=k)
        c.scale(1, rng.uniform(0.5, 0.8), 1)
        c = on_floor(c, BOARD[1] - 0.0002)
        c.move(x0 + rng.uniform(0.004, 0.060), 0, rng.uniform(-0.07, 0.07))
        s = s + c
    return s


# ======================================================= sewing machine ===
# A portable machine of the Singer 221 class: 14 7/8 x 7 1/4 x 9 3/4 in
# (378 x 184 x 248 mm) with its handwheel, 11 lb of cast aluminium,
# black-japanned with gold decals, bright-plated fittings. The needle to
# -x (left), the handwheel to +x (right), its front +z.
SEW_BED = (0.300, 0.030, 0.150)      # w, thickness, depth
SEW_FEET = 0.008
SEW_TOP = SEW_FEET + SEW_BED[1]      # the bed's top, 38 mm up
SEW_ARM_Z = (-0.060, -0.002)         # the pillar and arm's back and front faces
SEW_AXLE = (0.1405, 0.1960, -0.0310)   # the handwheel's axle: its inner face's x, y, z
SEW_NEEDLE_X, SEW_NEEDLE_Z = -0.1230, -0.0140


def sew_bed():
    w, t, d = SEW_BED
    bed = prism(rounded_rect(w, d, 0.022, seg=4), t, bevel_r=0.0045, bevel_seg=2)
    bed.rot(x=math.pi / 2).move(0, SEW_FEET + t / 2, -0.028)
    feet = Shape()
    for sx in (-1, 1):
        for sz in (-1, 1):
            f = cyl(0.0095, 0.0105, SEW_FEET + 0.002, seg=12, bevel_r=0.0015)
            feet = feet + f.move(sx * (w / 2 - 0.028), (SEW_FEET + 0.002) / 2, -0.028 + sz * (d / 2 - 0.026))
    return bed + feet


def sew_arm():
    # the pillar and arm, seen from the front, cast round: its outline
    # extruded 56 mm deep and its edges rounded 10 mm
    out = [(0.046, SEW_TOP - 0.002), (0.140, SEW_TOP - 0.002), (0.141, 0.150), (0.136, 0.198),
           (0.126, 0.222), (0.108, 0.2340), (0.080, 0.2365), (-0.040, 0.2365), (-0.100, 0.2340),
           (-0.100, 0.1800), (0.030, 0.1790), (0.050, 0.1730), (0.062, 0.1580), (0.067, 0.1350),
           (0.066, 0.1000), (0.058, 0.0650)]
    depth = SEW_ARM_Z[1] - SEW_ARM_Z[0]
    arm = prism(out, depth, bevel_r=0.0095, bevel_seg=2).move(0, 0, (SEW_ARM_Z[0] + SEW_ARM_Z[1]) / 2)
    # the head: a block at the arm's end, deeper and rounder
    head = rbox(0.050, 0.090, 0.066, r=0.011, seg=2).move(-0.120, 0.1930, -0.0285)
    # the motor on the back of the pillar, and its brush caps
    motor = cyl(0.029, h=0.074, seg=20, bevel_r=0.006).rot(z=math.pi / 2).move(0.094, 0.122, -0.088)
    caps = Shape()
    for sx in (-1, 1):
        caps = caps + cyl(0.0055, h=0.008, seg=10).move(0.094 + sx * 0.018, 0.153, -0.088)
    # the boss the handwheel turns on
    boss = cyl(0.016, h=0.008, seg=16).rot(z=math.pi / 2).move(SEW_AXLE[0] - 0.003, SEW_AXLE[1], SEW_AXLE[2])
    return arm + head + motor + caps + boss


def sew_steel():
    parts = []
    # the face plate on the head's end
    parts.append(rbox(0.0022, 0.082, 0.062, r=0.004, seg=2).move(-0.1458, 0.1930, -0.0285))
    # the needle and slide plates on the bed
    parts.append(rbox(0.050, 0.0016, 0.036, r=0.0007, seg=1).move(SEW_NEEDLE_X, SEW_TOP + 0.0008, -0.016))
    parts.append(rbox(0.050, 0.0014, 0.040, r=0.0007, seg=1).move(SEW_NEEDLE_X, SEW_TOP + 0.0007, 0.023))
    # the needle bar and its clamp, the needle
    parts.append(cyl(0.0038, h=0.070, seg=10).move(SEW_NEEDLE_X, 0.1180, SEW_NEEDLE_Z))
    parts.append(rbox(0.009, 0.010, 0.009, r=0.001, seg=1).move(SEW_NEEDLE_X, 0.0800, SEW_NEEDLE_Z))
    parts.append(cyl(0.0009, h=0.026, seg=6, bevel_r=0).move(SEW_NEEDLE_X, 0.0620, SEW_NEEDLE_Z))
    parts.append(cone(0.0009, 0.003, seg=6).rot(x=math.pi).move(SEW_NEEDLE_X, 0.0475, SEW_NEEDLE_Z))
    # the presser bar behind it, and the foot on the plate: shank and sole
    parts.append(cyl(0.0034, h=0.085, seg=10).move(SEW_NEEDLE_X + 0.001, 0.1080, SEW_NEEDLE_Z - 0.012))
    parts.append(rbox(0.008, 0.016, 0.010, r=0.0012, seg=1).move(SEW_NEEDLE_X + 0.001, 0.0560, SEW_NEEDLE_Z - 0.008))
    sole = rbox(0.013, 0.0035, 0.030, r=0.0012, seg=1)
    sole = cut(sole, box(0.0035, 0.01, 0.014).move(0, 0, 0.012))      # the toes, split for the needle
    parts.append(sole.move(SEW_NEEDLE_X, SEW_TOP + 0.0034, SEW_NEEDLE_Z + 0.004))
    # the presser lifter at the head's back
    parts.append(rbox(0.004, 0.006, 0.028, r=0.0012, seg=1).rot(x=0.4).move(-0.112, 0.160, -0.066))
    # the take-up lever out of its slot in the head's front
    parts.append(rbox(0.006, 0.006, 0.020, r=0.0015, seg=1).move(-0.112, 0.2180, 0.012))
    # the tension: discs and a thumb nut on the head's front
    parts.append(cyl(0.0085, h=0.004, seg=14).rot(x=math.pi / 2).move(-0.122, 0.1680, 0.0060))
    parts.append(cyl(0.0085, h=0.004, seg=14).rot(x=math.pi / 2).move(-0.122, 0.1680, 0.0100))
    parts.append(lathe([(0, 0), (0.0062, 0), (0.0066, 0.006), (0.0052, 0.010), (0, 0.0105)], seg=12)
                 .rot(x=math.pi / 2).move(-0.122, 0.1680, 0.0120))
    # thread guides on the arm's top, the spool pin, the bobbin winder
    for gx in (-0.070, 0.020):
        g = torus(0.0035, 0.0009, seg=10, tseg=5).rot(y=math.pi / 2).move(gx, 0.2395, -0.018)
        parts.append(g)
    parts.append(cyl(0.0016, h=0.040, seg=8).move(0.030, 0.2560, -0.030))
    parts.append(cyl(0.0060, h=0.016, seg=12).rot(z=math.pi / 2).move(0.104, 0.2410, -0.040))
    parts.append(cyl(0.0080, h=0.005, seg=14).rot(z=math.pi / 2).move(0.115, 0.2410, -0.040))
    # the stitch-length lever in its slot on the pillar's front
    parts.append(rbox(0.004, 0.022, 0.006, r=0.0012, seg=1).rot(z=-0.5).move(0.112, 0.100, 0.0015))
    return sum(parts)


def sew_wheel():
    """The handwheel, 82 mm across: a dished disc on its hub, its rim
    rounded; centred on the axle (x)."""
    prof = [(0.0, 0.002), (0.0140, 0.002), (0.0160, 0.0062), (0.0335, 0.0062), (0.0360, 0.0015),
            (0.0395, 0.0), (0.0410, 0.0016), (0.0414, 0.0115), (0.0410, 0.0210), (0.0395, 0.0232),
            (0.0358, 0.0236), (0.0336, 0.0196), (0.0160, 0.0158), (0.0140, 0.0200), (0.0, 0.0204)]
    w = lathe(prof, seg=36).rot(z=-math.pi / 2)
    ax, ay, az = SEW_AXLE
    return w.move(ax, ay, az)


def sew_wheel_knob():
    prof = [(0.0, 0.0185), (0.0108, 0.0185), (0.0114, 0.0240), (0.0108, 0.0290), (0.0080, 0.0318), (0.0, 0.0322)]
    k = lathe(prof, seg=20).rot(z=-math.pi / 2)
    ax, ay, az = SEW_AXLE
    return k.move(ax, ay, az)


def sew_spool():
    """A wooden spool of cotton thread on the pin (32 mm tall)."""
    prof = [(0.0016, 0.0), (0.0110, 0.0), (0.0114, 0.0035), (0.0100, 0.0040), (0.0100, 0.0280),
            (0.0114, 0.0285), (0.0110, 0.0320), (0.0016, 0.0320)]
    s = lathe(prof + [(0.0016, 0.0)], seg=18, cap_bottom=False, cap_top=False)
    s = weld(s)
    return s.move(0.030, 0.2368, -0.030)


# ============================================================ firewood ===
# Split stove wood, 12 in (300 mm) lengths: a quarter of a 210 mm round, half
# of a 130 mm round, a slab split off a 340 mm round, and a 84 mm branch
# round; stacked as the old placer stacked its logs (a bottom row, one or two
# laid in its grooves). Each stack is three parts: its bark, the faces it was
# split along, its sawn ends (one part an end, so each end's rings centre on
# its own pith).
LOG_L = 0.300
LOG_KINDS = {
    # outline in (z, y) about the pith, lying on a face; each edge 'b' (bark) or 's' (split)
    'A': dict(R=0.105, a0=0.0, a1=math.pi / 2, r_in=0.0),
    'B': dict(R=0.064, a0=0.0, a1=math.pi, r_in=0.0),
    'C': dict(R=0.170, a0=0.0, a1=0.80, r_in=0.090),
    'D': dict(R=0.042, a0=0.0, a1=TAU, r_in=0.0),
}


def _log_outline(kind, mirror=False):
    """[(z, y, edge kind of the edge from this point to the next)], the pith
    at (0, 0) for the sectors; D's centre."""
    k = LOG_KINDS[kind]
    R, a0, a1, rin = k['R'], k['a0'], k['a1'], k['r_in']
    pts = []
    if kind == 'D':
        n = 32
        for i in range(n):
            a = TAU * i / n
            pts.append((R * math.cos(a), R + R * math.sin(a), 'b'))
        return pts
    narc = max(4, int((a1 - a0) / 0.066))
    if rin > 0:
        nin = 4
        for i in range(nin):
            r = rin + (R - rin) * i / nin
            pts.append((r * math.cos(a0), r * math.sin(a0), 's'))
    else:
        for i in range(4):
            r = R * i / 4
            pts.append((r * math.cos(a0), r * math.sin(a0), 's'))
    for i in range(narc):
        a = a0 + (a1 - a0) * i / narc
        pts.append((R * math.cos(a), R * math.sin(a), 'b'))
    if rin > 0:
        for i in range(4):
            r = R - (R - rin) * i / 4
            pts.append((r * math.cos(a1), r * math.sin(a1), 's'))
        # the inner face: split straight across
        p0 = (rin * math.cos(a1), rin * math.sin(a1))
        p1 = (rin * math.cos(a0), rin * math.sin(a0))
        for i in range(3):
            t = i / 3
            pts.append((p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t, 's'))
    else:
        for i in range(4):
            r = R - R * i / 4
            pts.append((r * math.cos(a1), r * math.sin(a1), 's'))
    if mirror:
        pts = [(-z, y, e) for z, y, e in reversed(pts)]
        # the edge kinds belong to the edge after each point: shift by one
        ek = [e for _, _, e in pts]
        pts = [(z, y, ek[(i + 1) % len(ek)]) for i, (z, y, _) in enumerate(pts)]
    return pts


def _log(kind, seed, mirror=False):
    """One split, along x from -L/2 to L/2: the outline swept with its
    fibre (split faces: fibres along it, torn; bark: ridged along it), a
    little taper and twist, the ends sawn a little off square. Returns
    (bark, split, endL, endR) shapes and its pith (z, y)."""
    rng = random.Random(seed)
    out = _log_outline(kind, mirror)
    n = len(out)
    nx = 10
    o1 = Vector((rng.uniform(-50, 50), rng.uniform(-50, 50), rng.uniform(-50, 50)))
    taper = rng.uniform(-0.05, 0.05)
    twist = rng.uniform(-0.05, 0.05)
    tilt = (rng.uniform(-0.06, 0.06), rng.uniform(-0.06, 0.06), rng.uniform(-0.06, 0.06), rng.uniform(-0.06, 0.06))
    cz = sum(p[0] for p in out) / n
    cy = sum(p[1] for p in out) / n
    # each outline point's outward normal (in z, y), for the displacement
    norms = []
    for i in range(n):
        za, ya, _ = out[i - 1]
        zb, yb, _ = out[(i + 1) % n]
        tz, ty = zb - za, yb - ya
        l = math.hypot(tz, ty) or 1.0
        nz, ny = ty / l, -tz / l
        # outward: away from the centroid
        if (out[i][0] - cz) * nz + (out[i][1] - cy) * ny < 0:
            nz, ny = -nz, -ny
        norms.append((nz, ny))

    def ring(j):
        t = j / nx
        x = -LOG_L / 2 + LOG_L * t
        sc = 1.0 + taper * (t - 0.5)
        tw = twist * (t - 0.5)
        pts = []
        for i, (z, y, e) in enumerate(out):
            # the edge kinds of the two edges meeting here
            e_prev = out[i - 1][2]
            z1, y1 = (z - cz) * sc, (y - cy) * sc
            z2 = z1 * math.cos(tw) - y1 * math.sin(tw) + cz
            y2 = z1 * math.sin(tw) + y1 * math.cos(tw) + cy
            nz, ny = norms[i]
            # the wood's own waviness: a split follows the grain, a round is
            # never round
            d = 0.0045 * noise.noise(Vector((x * 3.0, z * 22.0, y * 22.0)) + o1)
            if e == 'b' and e_prev == 'b':
                # bark: ridges along the log between deep fissures
                n1 = noise.noise(Vector((x * 4.0, z * 160.0, y * 160.0)) + o1)
                d += 0.0042 * (0.45 - abs(n1)) + 0.0012 * noise.noise(Vector((x * 30.0, z * 300.0, y * 300.0)))
            else:
                # a split face: torn along its fibres
                d += 0.0020 * noise.noise(Vector((x * 9.0, z * 260.0, y * 260.0)) + o1) + 0.0012 * noise.noise(Vector((x * 40.0, z * 500.0, y * 500.0)))
            xx = x
            if j == 0:
                xx = x + tilt[0] * z + tilt[1] * y
            elif j == nx:
                xx = x + tilt[2] * z + tilt[3] * y
            pts.append(Vector((xx, y2 + ny * d, z2 + nz * d)))
        return pts
    rings = [ring(j) for j in range(nx + 1)]
    bark, split = Shape(), Shape()
    for i in range(n):
        e = out[i][2]
        tgt = bark if e == 'b' else split
        for j in range(nx):
            a, b = rings[j][i], rings[j][(i + 1) % n]
            c, d = rings[j + 1][(i + 1) % n], rings[j + 1][i]
            base = len(tgt.v)
            tgt.v += [a.copy(), b.copy(), c.copy(), d.copy()]
            tgt.f.append((base, base + 1, base + 2, base + 3))
    bark = weld(bark, 1e-7)
    split = weld(split, 1e-7)

    def cap(r, sign):
        c = sum(r, Vector((0, 0, 0))) / len(r)
        v = [p.copy() for p in r] + [c]
        f = []
        for i in range(len(r)):
            f.append((i, (i + 1) % len(r), len(r)))
        s = Shape(v, f)
        nsum = Vector((0, 0, 0))
        for fc in s.f:
            a, b, cc = (s.v[k] for k in fc)
            nsum += (b - a).cross(cc - a)
        if nsum.x * sign < 0:
            s.f = [tuple(reversed(fc)) for fc in s.f]
        return s
    endL = cap(rings[0], -1)
    endR = cap(rings[-1], 1)
    # sides face out of the log
    bark = _face_out(bark, Vector((0, cy, cz)))
    split = _face_out(split, Vector((0, cy, cz)))
    if kind == 'D':
        pith = (0.0, LOG_KINDS['D']['R'])
    else:
        pith = (0.0, 0.0)
    return bark, split, endL, endR, pith


def _face_out(s, axis_pt):
    """Turn each face of a sheet round the log so it faces away from the
    line through axis_pt along x."""
    nf = []
    for fc in s.f:
        pts = [s.v[i] for i in fc]
        c = sum(pts, Vector((0, 0, 0))) / len(pts)
        n = (pts[1] - pts[0]).cross(pts[2] - pts[0])
        out = Vector((0, c.y - axis_pt.y, c.z - axis_pt.z))
        nf.append(tuple(reversed(fc)) if n.dot(out) < 0 else fc)
    s.f = nf
    return s


def _poly_span(poly, z):
    """The lowest and highest y of a closed polygon [(z, y)] at z, or None."""
    ys = []
    n = len(poly)
    for i in range(n):
        z0, y0 = poly[i]
        z1, y1 = poly[(i + 1) % n]
        if (z0 <= z <= z1) or (z1 <= z <= z0):
            if abs(z1 - z0) < 1e-9:
                ys += [y0, y1]
            else:
                ys.append(y0 + (y1 - y0) * (z - z0) / (z1 - z0))
    return (min(ys), max(ys)) if ys else None


# the stacks: (kind, mirrored) for the bottom row left to right, then the row above
STACKS = {
    3: ([('A', False), ('B', False)], [('D', False)]),
    4: ([('A', True), ('C', False), ('B', False)], [('A', False)]),
    5: ([('B', False), ('C', True), ('A', False)], [('D', False), ('B', False)]),
}
STACK_PITHS = {}


def wood_stack(count):
    """The stack of `count` splits: (bark, split, endL, endR)."""
    rng = random.Random(16500 + count)
    bottom, upper = STACKS[count]
    parts = [Shape(), Shape(), Shape(), Shape()]
    piths = []
    placed = []          # (outline poly in z, y after placing) of the bottom row

    def put(kind, mirror, dz, dy, row, idx):
        bark, split, el, er, pith = _log(kind, 16600 + count * 10 + row * 4 + idx, mirror)
        yaw = rng.uniform(-0.05, 0.05)
        dx = rng.uniform(-0.018, 0.018) + 0.010 * row
        m = Matrix.Translation((dx, dy, dz)) @ Matrix.Rotation(yaw, 4, 'Y')
        for k, s in enumerate((bark, split, el, er)):
            parts[k] = parts[k] + s.copy().apply(m)
        # the pith at each end, after the move
        pz = -pith[0] if mirror else pith[0]
        for ex in (-LOG_L / 2, LOG_L / 2):
            p = m @ Vector((ex, pith[1], pz))
            piths.append((row, idx, ex > 0, p.z, p.y, LOG_KINDS[kind]['R']))
    outs = [[(z, y) for z, y, _ in _log_outline(k, mi)] for k, mi in bottom]
    widths = [(min(p[0] for p in o), max(p[0] for p in o)) for o in outs]
    gap = 0.008
    total = sum(w1 - w0 for w0, w1 in widths) + gap * (len(widths) - 1)
    z = -total / 2
    offs = []
    for (k, mi), (w0, w1), o in zip(bottom, widths, outs):
        dz = z - w0
        offs.append(dz)
        placed.append([(pz + dz, py) for pz, py in o])
        z += (w1 - w0) + gap
    for i, ((k, mi), dz) in enumerate(zip(bottom, offs)):
        put(k, mi, dz, 0.0, 0, i)
    # the row above: each over a groove, as low as the logs under it let it lie
    for i, (k, mi) in enumerate(upper):
        o = [(z, y) for z, y, _ in _log_outline(k, mi)]
        w0, w1 = min(p[0] for p in o), max(p[0] for p in o)
        left, right = placed[i], placed[i + 1]
        groove = (max(p[0] for p in left) + min(p[0] for p in right)) / 2
        dz = groove - (w0 + w1) / 2
        need = -1.0
        zz = w0
        while zz <= w1:
            mine = _poly_span(o, zz)
            for poly in placed:
                under = _poly_span(poly, zz + dz)
                if mine and under:
                    need = max(need, under[1] - mine[0])
            zz += 0.001
        put(k, mi, dz, max(0.0, need) + 0.0015, 1, i)
    # the bark's and the splits' roughness reach a few millimetres under
    # the faces they lie on: the stack's lowest point on the floor
    lo_y = min(p.bounds()[0].y for p in parts)
    for p in parts:
        p.move(0, -lo_y, 0)
    STACK_PITHS[count] = [(a, b, c, pz, py - lo_y, R) for a, b, c, pz, py, R in piths]
    return parts


_STACK_CACHE = {}


def wood(count, k):
    if count not in _STACK_CACHE:
        _STACK_CACHE[count] = wood_stack(count)
    return _STACK_CACHE[count][k]


# ================================================================ parts ===
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    kind = 'cyl' if name in CYL_FRAMED else 'box'
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget)


auto('Real__wineGlassBowl', wine_glass_bowl, 1000)
auto('Real__wineGlassFoot', wine_glass_foot, 700)
auto('Real__wineGlassLying', wine_glass_lying, 1500)
auto('Real__wineGlassDregs', wine_glass_dregs, 400)
auto('Real__bottleBase', bottle_base, 1800)
auto('Real__bottleNeck', bottle_neck, 2000)
auto('Real__bottleShardA', bottle_shard_a, 300)
auto('Real__bottleShardB', bottle_shard_b, 300)
auto('Real__bottleShardC', bottle_shard_c, 300)
auto('Real__bottleShardD', bottle_shard_d, 300)
auto('Real__bottleShardE', bottle_shard_e, 300)
auto('Real__bottleChips', bottle_chips, 700)
auto('Real__kettleBody', kettle_body, 3200)
auto('Real__kettleSpout', kettle_spout, 1000)
auto('Real__kettleBail', kettle_bail, 600)
auto('Real__kettleGrip', kettle_grip, 700)
auto('Real__kettleTrivet', kettle_trivet, 500)
auto('Real__panBody', pan_body, 2600)
auto('Real__panNail', pan_nail, 200)
auto('Real__breadBoard', bread_board, 500)
auto('Real__breadCrust', bread_crust, 3800)
auto('Real__breadCrumb', bread_crumb, 300)
auto('Real__breadCrumbs', bread_crumbs, 1000)
auto('Real__sewBed', sew_bed, 1200)
auto('Real__sewArm', sew_arm, 2400)
auto('Real__sewSteel', sew_steel, 2400)
auto('Real__sewWheel', sew_wheel, 1100)
auto('Real__sewWheelKnob', sew_wheel_knob, 300)
auto('Real__sewSpool', sew_spool, 300)
auto('Real__woodStack3Bark', lambda: wood(3, 0), 2400)
auto('Real__woodStack3Split', lambda: wood(3, 1), 1000)
auto('Real__woodStack3EndL', lambda: wood(3, 2), 200)
auto('Real__woodStack3EndR', lambda: wood(3, 3), 200)
auto('Real__woodStack4Bark', lambda: wood(4, 0), 2400)
auto('Real__woodStack4Split', lambda: wood(4, 1), 1200)
auto('Real__woodStack4EndL', lambda: wood(4, 2), 250)
auto('Real__woodStack4EndR', lambda: wood(4, 3), 250)
auto('Real__woodStack5Bark', lambda: wood(5, 0), 3400)
auto('Real__woodStack5Split', lambda: wood(5, 1), 1400)
auto('Real__woodStack5EndL', lambda: wood(5, 2), 300)
auto('Real__woodStack5EndR', lambda: wood(5, 3), 300)


def _print_piths():
    """The piths on each stack's ends, in the end parts' texture coordinates
    (the box's: +x faces u = 1 - fz, -x faces u = fz, v = fy), for the game's
    end-grain painter (docs/research/props-remodelling/kitchen_and_hearth_left_behind.md)."""
    for count in (3, 4, 5):
        for side, k in (('L', 2), ('R', 3)):
            lo, hi = wood(count, k).bounds()
            ez, ey = hi.z - lo.z, hi.y - lo.y
            rows = []
            for row, idx, right, pz, py, R in STACK_PITHS[count]:
                if right != (side == 'R'):
                    continue
                fz = (pz - lo.z) / ez
                u = 1 - fz if side == 'R' else fz
                v = (py - lo.y) / ey
                rows.append(f'({u:.4f}, {v:.4f}, {R / ez:.4f}, {R / ey:.4f})')
            print(f'[r16] woodStack{count}End{side} piths: [' + ', '.join(rows) + ']')


_print_piths()
