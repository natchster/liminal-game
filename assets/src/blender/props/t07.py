"""t07: the props remodelled at their real sizes (docs/design/PROPS.md, P3, the
rare): the smoking ashtray, the hanging colander, the herb sieve, the wall
coat rack, the upright vacuum cleaner, the boot tray and the umbrella stand.
(The opened suitcase is r02's suitcase, the coats r06's, the scarf r13b's,
the umbrellas r09's and the boots r14a's.) Sources and sizes: t07.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame: x across its front, y up from the floor (or, for a
thing on a wall, from the point it hangs at), z out of its front (out of the
wall). The colander is modelled standing (the game turns it to hang), so
its bowl's texture is laid round its axis (a 'sphere' frame).

The ashtray (`place_smoking_ashtrays`): a round pressed-glass ashtray of
the Libbey 5156 kind, 108 mm across and 25 mm tall, four rests across its
rim; ash in it, three butts stubbed out, and a cigarette left burning in a
rest, its ash grown on it. Its bottom at y = 0, centred.

ashtrayGlass       the glass: the foot and the dish under it, the thick
                   bottom, the bowl, the flat rim and its four rests
ashtrayAsh         the ash in the bowl, its lumps and a fallen column
ashtrayButtFilters the three butts' filters (cork tipping)
ashtrayButtPapers  their paper stubs, crushed out
ashtrayButtChar    their charred ends
ashtrayCigFilter   the burning cigarette's filter, out over the rim
ashtrayCigPaper    its paper, 35 mm of tobacco left
ashtrayCigEmber    the burning ring between the paper and the ash
ashtrayCigAsh      the column of ash grown on it, cracked, drooping

The colander (`place_hanging_colanders`): a 24 cm two-handled colander
(enamelled steel, stainless or tin-lined copper), a stamped bowl on a foot
ring, a rolled rim, two ears riveted under it, each with a hole to hang it
by. Standing: its bottom at y = 0 (the foot below it), its axis on y, the
ears along x.

colanderBowl       the perforated bowl, one sheet (the holes are the
                   game's: cut out of its texture, laid round the axis)
colanderRim        the rolled rim, the foot ring, the two ears and rivets
colanderNail       a wire nail driven into the wall a little upward (the
                   wall's frame: the wall at z = 0, the shank's entry at
                   the origin); the sieve hangs on one too

The sieve (`place_herb_sieves`): a 7 in (178 mm) fine-mesh strainer, a
tinned steel band round a woven mesh bowl, a looped wire handle and a pan
hook, hung by its handle's loop on a nail (the wall's frame: the nail at
the origin; the rim against the wall, the bowl out).

sieveMesh          the woven mesh bowl, one sheet (its weave the game's)
sieveFrame         the band, the handle's two rods and loop, the pan hook

The coat rack (`place_coat_racks`): a 24 in (610 mm) board of stained
hardwood, its front rounded and beaded, screwed to the wall, with five
cast brass hooks 120 mm apart (the wall's frame: the wall at z = 0, the
board's middle at the origin). A hook's cradle, where a coat's loop rests,
is 77 mm out from the wall (as r06's hooks', so r06's coat and r13b's
scarf hang on it as modelled).

coatRackBoard      the board, its ends and front edges rounded, a bead
                   routed along its face
coatRackScrews     the two countersunk screws through it
coatRackPegs       the five hooks: a round rose, an arm out and down to
                   its cradle, turned up to a ball

The vacuum (`place_vacuum_cleaners`): an upright of the Hoover
Convertible kind, 45 x 13 x 12 in, its handle locked upright: the hood
over the brush on four wheels, its headlight, a vinyl bumper round it;
a chromed handle on a bail; the cloth outer bag hung from a hook on the
handle and clamped over the fill tube; the cord wound on two hooks, its
loose end the game's. Its frame: the hood's front at +z.

vacuumHood         the moulded hood, the headlight's housing, the bail's
                   sockets
vacuumRubber       the bumper, the wheels
vacuumLens         the headlight's lens
vacuumMetal        the bail, the handle tube, the bag hook, the cord hooks
vacuumGrip         the grip and the switch housing on top of the handle
vacuumBag          the cloth bag, sagging, full low down
vacuumFillTube     the elbow from the fan up into the bag, its clamp
vacuumCord         the cord wound on its hooks, down to where it hangs
                   free (CORD_FREE)
vacuumPlug         the plug on the cord's end (its own frame: the cord
                   along -z into it, the blades along +z)
vacuumPlugBlades   its two blades

The boot tray (`place_boot_trays`): a 32 x 16 in moulded rubber boot tray,
an inch deep, its floor ribbed; its bottom at y = 0, centred, its length
along x.

bootTray           the tray

The umbrella stand (`place_umbrella_stands`): a rolled steel cylinder
200 mm across and 490 mm tall, a rolled bead at its top and a swaged rib
at its foot and shoulder, its lap seam down the back, a pressed bottom
seamed on (a 'cyl' frame: its texture laid round it).

umbrellaStandBody  the stand
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


def rnd(a, b):
    return a + (b - a) * random.random()


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def along_dir(s, d, origin):
    """A shape built along +x, turned so +x runs along `d` and moved to
    `origin`."""
    q = Vector((1, 0, 0)).rotation_difference(Vector(d).normalized())
    return s.apply(q.to_matrix().to_4x4()).move(*origin)


def rod(pts, r, seg=8, smooth=1, r_end=None, caps=True):
    return tube(pts, r, seg=seg, caps=caps, smooth=smooth, r_end=r_end)


def loft(rings, cap0=True, cap1=True):
    """Join closed rings of points (each the same count) into a surface."""
    m = len(rings[0])
    v = [Vector(p) for ring in rings for p in ring]
    f = []
    for i in range(len(rings) - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, (i + 1) * m + j2, (i + 1) * m + j))
    if cap0:
        f.append(tuple(reversed(range(m))))
    if cap1:
        n = len(rings) - 1
        f.append(tuple(n * m + j for j in range(m)))
    return propkit._fix_winding(Shape(v, f))


def band(outer, inner, y0, y1):
    """A ring between two closed plan outlines [(x, z)] of one count, from
    y0 up to y1 (r02's)."""
    n = len(outer)
    v = [Vector((x, y0, z)) for x, z in outer] + [Vector((x, y1, z)) for x, z in outer] + \
        [Vector((x, y0, z)) for x, z in inner] + [Vector((x, y1, z)) for x, z in inner]
    f = []
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
        f.append((2 * n + j, 2 * n + i, 3 * n + i, 3 * n + j))
        f.append((n + i, n + j, 3 * n + j, 3 * n + i))
        f.append((j, i, 2 * n + i, 2 * n + j))
    return propkit._fix_winding(Shape(v, f))


def along_y(outline_xz, depth, bevel_r=0.0, bevel_seg=1):
    """An outline in the (x, z) plane (a plan) extruded up y, centred on y = 0."""
    s = prism([(x, -z) for x, z in outline_xz], depth, bevel_r=bevel_r, bevel_seg=bevel_seg)
    return s.rot(-math.pi / 2, 0, 0)


def rrect_plan(w, d, r, seg=4):
    """A rounded rectangle's plan [(x, z)], counter-clockwise from +x."""
    out = []
    for cx, cz, a0 in ((w / 2 - r, d / 2 - r, 0.0), (-w / 2 + r, d / 2 - r, math.pi / 2),
                       (-w / 2 + r, -d / 2 + r, math.pi), (w / 2 - r, -d / 2 + r, 1.5 * math.pi)):
        for k in range(seg + 1):
            a = a0 + (math.pi / 2) * k / seg
            out.append((cx + r * math.cos(a), cz + r * math.sin(a)))
    return out


# ============================================================== ashtray ===
AT_H = 0.0252                    # rim top
NOTCH_R, NOTCH_Y, NOTCH_MID = 0.0050, 0.0262, 0.0485
ASH_TOP = 0.0126                 # the ash's crown, at the middle
CIG_R = 0.0039                   # 7.8 mm
CIG_AZ = math.pi / 4             # the rest it lies in
CIG_TILT = math.radians(10.6)    # its burning end down on the ash
CIG_OUT = 0.0665                 # the filter's end, out from the middle
CIG_FILTER, CIG_PAPER, CIG_EMBER, CIG_ASH = 0.025, 0.035, 0.0013, 0.0088


def ashtray_glass():
    prof = [(0.0, 0.0028), (0.030, 0.0026), (0.038, 0.0018), (0.0415, 0.0004), (0.0445, 0.0), (0.0482, 0.0),
            (0.0497, 0.0012), (0.0512, 0.0060), (0.0530, 0.0160), (0.0540, 0.0228), (0.0537, 0.0244),
            (0.0526, AT_H), (0.0446, AT_H), (0.0432, 0.0243), (0.0402, 0.0205), (0.0352, 0.0135),
            (0.0310, 0.0098), (0.0255, 0.0086), (0.0, 0.0082)]
    s = lathe(prof, seg=36)
    for k in range(4):
        a = math.pi / 4 + k * math.pi / 2
        c = cyl(NOTCH_R, NOTCH_R, 0.032, seg=12, bevel_r=0).rot(0, 0, math.pi / 2).rot(0, -a, 0)
        s = cut(s, c.move(NOTCH_MID * math.cos(a), NOTCH_Y, NOTCH_MID * math.sin(a)))
    return s


def ashtray_ash():
    prof = [(0.0, 0.0084), (0.0302, 0.0087), (0.0292, 0.0095), (0.024, 0.0104), (0.016, 0.0115),
            (0.008, 0.0123), (0.0, ASH_TOP)]
    s = lathe(prof, seg=24)
    s = noisy(s, 0.0009, scale=260, seed=3, axis=(0, 1, 0))
    # flakes and lumps of ash knocked off, a column that fell whole
    for (x, z, r, k) in ((-0.012, 0.009, 0.0045, 0.45), (0.010, -0.014, 0.0035, 0.5), (-0.004, -0.020, 0.003, 0.55),
                         (-0.020, -0.008, 0.0038, 0.4)):
        lump = sphere(r, seg=8, rings=5).scale(1.0, k, 1.0)
        lump = noisy(lump, r * 0.25, scale=500, seed=int(x * 1000) + 7)
        s = s + lump.move(x, 0.0108 + r * k * 0.4, z)
    col = lathe([(0.0, 0.0), (0.0034, 0.0004), (0.0036, 0.004), (0.0033, 0.0068), (0.0029, 0.008), (0.0, 0.0082)], seg=8)
    col = noisy(col.rot(0, 0, math.pi / 2), 0.0004, scale=600, seed=11)
    s = s + col.rot(0, 0.9, 0).move(-0.006, 0.0128, 0.016)
    return s


# The butts: (filter end (x, y, z), the other end (x, y, z), paper left, crushed)
BUTTS = (((-0.026, 0.0162, 0.004), (-0.014, 0.0160, -0.0225), 0.0076, 0.8),
         ((0.022, 0.0158, -0.012), (0.000, 0.0160, -0.025), 0.0040, 1.0),
         ((-0.010, 0.0165, 0.014), (-0.0235, 0.0216, 0.0295), 0.0011, 0.5))


def _butt(k, which):
    a, b, _, crush = BUTTS[k]
    a, b = Vector(a), Vector(b)
    L = (b - a).length
    filt = 0.021
    if which == 'filter':
        s = cyl(CIG_R, CIG_R, filt, seg=10, bevel_r=0.0006, bevel_seg=1).rot(0, 0, -math.pi / 2).move(filt / 2, 0, 0)
    elif which == 'paper':
        pl = L - filt
        s = cyl(CIG_R * 0.98, CIG_R * 0.98, pl, seg=10, bevel_r=0).rot(0, 0, -math.pi / 2).move(filt + pl / 2, 0, 0)
        # crushed out: flattened toward its end, kinked
        def crush_it(p):
            t = max(0.0, (p.x - filt) / max(pl, 1e-4))
            p.z *= 1.0 - 0.55 * crush * t
            p.y *= 1.0 + 0.25 * crush * t
            p.y += 0.0018 * crush * t * t
            return p
        s.warp(crush_it)
    else:
        s = sphere(CIG_R * 0.95, seg=8, rings=5).scale(0.45, 1.15, 1.0 - 0.5 * crush)
        s = noisy(s, 0.0007, scale=900, seed=k * 13 + 5).move(L, 0.0018 * crush, 0)
    return along_dir(s, b - a, a)


def ashtray_butts(which):
    return lambda: sum((_butt(k, which) for k in range(len(BUTTS))), Shape())


def _cig_frame():
    d = Vector((-math.cos(CIG_AZ) * math.cos(CIG_TILT), -math.sin(CIG_TILT), -math.sin(CIG_AZ) * math.cos(CIG_TILT)))
    y0 = (NOTCH_Y - NOTCH_R) + CIG_R + (CIG_OUT - NOTCH_MID) * math.tan(CIG_TILT)
    o = Vector((CIG_OUT * math.cos(CIG_AZ), y0, CIG_OUT * math.sin(CIG_AZ)))
    return d, o


def _along(x0, x1, r0, r1, seg=10):
    return cyl(r1, r0, x1 - x0, seg=seg, bevel_r=0).rot(0, 0, -math.pi / 2).move((x0 + x1) / 2, 0, 0)


def cig_filter():
    d, o = _cig_frame()
    s = cyl(CIG_R, CIG_R, CIG_FILTER, seg=12, bevel_r=0.0006, bevel_seg=1).rot(0, 0, -math.pi / 2).move(CIG_FILTER / 2, 0, 0)
    return along_dir(s, d, o)


def cig_paper():
    d, o = _cig_frame()
    s = _along(CIG_FILTER, CIG_FILTER + CIG_PAPER, CIG_R * 0.99, CIG_R * 0.99, seg=12)
    return along_dir(s, d, o)


def cig_ember():
    d, o = _cig_frame()
    x0 = CIG_FILTER + CIG_PAPER
    s = _along(x0, x0 + CIG_EMBER, CIG_R * 1.01, CIG_R * 0.97, seg=12)
    return along_dir(s, d, o)


def cig_ash():
    d, o = _cig_frame()
    x0 = CIG_FILTER + CIG_PAPER + CIG_EMBER
    prof = [(0.0, 0.0), (CIG_R * 0.96, 0.0002), (CIG_R * 0.98, 0.0024), (CIG_R * 0.90, 0.0029), (CIG_R * 0.97, 0.0034),
            (CIG_R * 0.95, 0.0056), (CIG_R * 0.86, 0.0061), (CIG_R * 0.93, 0.0066), (CIG_R * 0.88, CIG_ASH - 0.0008),
            (CIG_R * 0.6, CIG_ASH - 0.0001), (0.0, CIG_ASH)]
    s = lathe(prof, seg=10)
    s = noisy(s, 0.00035, scale=1400, seed=21)
    s.rot(0, 0, -math.pi / 2)        # its length along +x
    # drooping a little at its tip
    s.warp(lambda p: Vector((p.x, p.y - 0.012 * (p.x / CIG_ASH) ** 2 * 0.06, p.z)))
    return along_dir(s.move(x0, 0, 0), d, o)


auto('Real__ashtrayGlass', ashtray_glass, 1700)
auto('Real__ashtrayAsh', ashtray_ash, 900)
auto('Real__ashtrayButtFilters', ashtray_butts('filter'), 300)
auto('Real__ashtrayButtPapers', ashtray_butts('paper'), 300)
auto('Real__ashtrayButtChar', ashtray_butts('char'), 300)
auto('Real__ashtrayCigFilter', cig_filter, 120)
auto('Real__ashtrayCigPaper', cig_paper, 80)
auto('Real__ashtrayCigEmber', cig_ember, 80)
auto('Real__ashtrayCigAsh', cig_ash, 300)


# ============================================================= colander ===
# The bowl's profile (radius, height), from the middle of its bottom to the
# rim; the game paints the holes by it (t07 COL_PROF)
COL_PROF = [(0.000, 0.000), (0.030, 0.000), (0.052, 0.002), (0.068, 0.008), (0.082, 0.018), (0.094, 0.032),
            (0.103, 0.048), (0.110, 0.065), (0.1145, 0.082), (0.1170, 0.0955)]
COL_EAR_HOLE = (0.1385, 0.0915)  # an ear's hole: out from the axis, height


def colander_bowl():
    # rings across its bottom too, so the holes punched there are laid true
    # (a fan of long triangles from the middle would bend them)
    prof = [(0.0, 0.0), (0.006, 0.0), (0.012, 0.0), (0.018, 0.0), (0.024, 0.0)] + COL_PROF[1:]
    s = lathe(prof, seg=48, cap_bottom=False, cap_top=False)
    # a knock it took, on its shoulder (off the axes, so its bounds stand)
    c = Vector((0.098 * math.cos(2.2), 0.040, 0.098 * math.sin(2.2)))

    def dent(p):
        d = (p - c).length
        k = math.exp(-(d / 0.018) ** 2) * 0.0035
        if k < 1e-6:
            return p
        rr = math.hypot(p.x, p.z)
        return Vector((p.x * (1 - k / max(rr, 1e-4)), p.y + k * 0.3, p.z * (1 - k / max(rr, 1e-4))))
    return s.warp(dent)


def colander_rim():
    bead = torus(0.1192, 0.0027, seg=48, tseg=8).rot(math.pi / 2, 0, 0).move(0, 0.0972, 0)
    foot = lathe([(0.0505, 0.0008), (0.0548, -0.0150), (0.0568, -0.0172), (0.0574, -0.0181), (0.0556, -0.0186),
                  (0.0536, -0.0170), (0.0490, 0.0008), (0.0505, 0.0008)], seg=40, cap_bottom=False, cap_top=False)
    s = bead + weld(foot)
    for side in (-1, 1):
        hx, hy = COL_EAR_HOLE
        plan = round_outline([(0.1165, -0.017), (0.150, -0.0125), (0.157, 0.0), (0.150, 0.0125), (0.1165, 0.017)], 0.004, seg=2)
        ear = along_y(plan, 0.0018, bevel_r=0.0005).move(0, hy, 0)
        ear = cut(ear, cyl(0.0068, 0.0068, 0.02, seg=14, bevel_r=0).move(hx, hy, 0))
        # its sides pressed up into a rib, its root bent down onto the wall
        ear.warp(lambda p: Vector((p.x, p.y + 0.0024 * smoothstep(0.009, 0.0165, abs(p.z)) * smoothstep(0.112, 0.12, p.x), p.z)))
        flange = rbox(0.0018, 0.020, 0.034, r=0.0006, seg=1).rot(0, 0, -0.26).move(0.1160, 0.0815, 0)
        rivets = sum((screw_head(0.0028, 0.0013, slot=False).rot(0, 0, -math.pi / 2 - 0.26).move(0.1167, 0.080, z)
                      for z in (-0.009, 0.009)), Shape())
        piece = ear + flange + rivets
        if side < 0:
            piece = piece.rot(0, math.pi, 0)
        s = s + piece
    return s


def colander_nail():
    shank = cyl(0.00135, 0.00135, 0.030, seg=8, bevel_r=0).move(0, 0.009, 0)
    head = cyl(0.0033, 0.0033, 0.0009, seg=12, bevel_r=0.0003, bevel_seg=1).move(0, 0.0245, 0)
    s = (shank + head).rot(math.pi / 2, 0, 0)       # along +z, from 6 mm in the wall
    return s.rot(-math.radians(8), 0, 0)            # driven a little upward


auto('Real__colanderBowl', colander_bowl, 1400, smooth=60, kind='sphere')
auto('Real__colanderRim', colander_rim, 2100)
auto('Real__colanderNail', colander_nail, 140)


# ================================================================ sieve ===
SV_LOOP = Vector((0.0, -0.0088, 0.0118))     # the handle's loop's middle (on the nail)
SV_LOOP_R, SV_WIRE = 0.0105, 0.0021
SV_HANDLE = 0.135                            # from the loop to the band
SV_R = 0.0895                                # the band's middle radius
SV_CY = SV_LOOP.y - SV_LOOP_R - SV_HANDLE - SV_R
SV_Z0 = 0.0015                               # the band's back, near the wall
SV_DEPTH = 0.078


def sieve_mesh():
    prof = [(0.0876, 0.0), (0.0872, 0.010), (0.0846, 0.026), (0.0782, 0.043), (0.0672, 0.058), (0.0510, 0.069),
            (0.0300, 0.0760), (0.0120, 0.0778), (0.0, SV_DEPTH)]
    s = lathe(prof, seg=36, cap_bottom=False, cap_top=False)
    s = s.rot(math.pi / 2, 0, 0)                 # its axis out of the wall
    # sagged a little where things were pressed through it
    s.warp(lambda p: Vector((p.x, p.y, p.z - 0.0025 * math.exp(-((p.x + 0.02) ** 2 + (p.y + 0.01) ** 2) / 0.0012))))
    return s.move(0, SV_CY, SV_Z0 + 0.0035)


def sieve_frame():
    band_ = lathe([(SV_R - 0.0018, 0.0), (SV_R + 0.0018, 0.0), (SV_R + 0.0020, 0.0035), (SV_R + 0.0018, 0.0072),
                   (SV_R - 0.0018, 0.0072), (SV_R - 0.0020, 0.0035), (SV_R - 0.0018, 0.0)], seg=40,
                  cap_bottom=False, cap_top=False)
    band_ = weld(band_).rot(math.pi / 2, 0, 0).move(0, SV_CY, SV_Z0)
    loop = torus(SV_LOOP_R, SV_WIRE, seg=16, tseg=6).move(SV_LOOP.x, SV_LOOP.y, SV_LOOP.z)
    rods = Shape()
    top = SV_CY + SV_R
    for sx in (-1, 1):
        pts = [(sx * 0.0175, top - 0.004, SV_Z0 + 0.0036), (sx * 0.0140, top + 0.020, SV_Z0 + 0.0048),
               (sx * 0.0088, top + 0.070, SV_Z0 + 0.0075), (sx * 0.0050, SV_LOOP.y - SV_LOOP_R - 0.010, SV_LOOP.z - 0.001),
               (sx * 0.0030, SV_LOOP.y - SV_LOOP_R + 0.001, SV_LOOP.z)]
        rods = rods + rod(pts, SV_WIRE, seg=6, smooth=1)
    # the pan hook under the band: a wire bent in a U, out and down
    bot = SV_CY - SV_R
    hook = rod([(-0.009, bot + 0.004, SV_Z0 + 0.004), (-0.0095, bot - 0.007, SV_Z0 + 0.007), (-0.006, bot - 0.012, SV_Z0 + 0.009),
                (0.0, bot - 0.0135, SV_Z0 + 0.0095), (0.006, bot - 0.012, SV_Z0 + 0.009), (0.0095, bot - 0.007, SV_Z0 + 0.007),
                (0.009, bot + 0.004, SV_Z0 + 0.004)], 0.0018, seg=6, smooth=1)
    return band_ + loop + rods + hook


auto('Real__sieveMesh', sieve_mesh, 800, smooth=60)
auto('Real__sieveFrame', sieve_frame, 1300)


# ============================================================ coat rack ===
CR_W, CR_H, CR_T = 0.610, 0.090, 0.019
CR_PEGS = [-0.24 + 0.12 * i for i in range(5)]


def coat_rack_board():
    s = rbox(CR_W, CR_H, CR_T, r=0.0045, seg=2).move(0, 0, CR_T / 2)
    for y in (-0.031, 0.031):
        s = cut(s, rbox(CR_W - 0.05, 0.0022, 0.004, r=0.0008, seg=1).move(0, y, CR_T + 0.0005))
    # the board's front a little cupped, its corners knocked
    s.warp(lambda p: Vector((p.x, p.y, p.z - 0.0008 * (p.y / (CR_H / 2)) ** 2 * (p.z / CR_T))))
    return s


def coat_rack_screws():
    out = Shape()
    for x in (-0.285, 0.285):
        h = screw_head(0.0042, 0.0011).rot(math.pi / 2, 0, 0).move(x, 0.0, CR_T - 0.0002)
        out = out + h.rot(0, 0, 0)
    return out


def coat_rack_pegs():
    out = Shape()
    for i, x in enumerate(CR_PEGS):
        rose = lathe([(0.0, 0.0), (0.0110, 0.0), (0.0112, 0.0012), (0.0098, 0.0030), (0.0060, 0.0042), (0.0, 0.0045)], seg=14)
        rose = rose.rot(math.pi / 2, 0, 0).move(0, 0, CR_T)
        arm = rod([(0, 0.0, CR_T + 0.003), (0, -0.0022, 0.045), (0, -0.0046, 0.066), (0, -0.0050, 0.077), (0, -0.0028, 0.086),
                   (0, 0.0045, 0.0925), (0, 0.0150, 0.0950)], 0.0046, seg=8, smooth=1, r_end=0.0040)
        ball = sphere(0.0072, seg=10, rings=6).move(0, 0.0205, 0.0950)
        peg = rose + arm + ball
        out = out + peg.move(x, 0, 0)
    return out


auto('Real__coatRackBoard', coat_rack_board, 600)
auto('Real__coatRackScrews', coat_rack_screws, 200)
auto('Real__coatRackPegs', coat_rack_pegs, 2400)


# =============================================================== vacuum ===
VAC_Z0, VAC_Z1 = -0.115, 0.165          # the hood's back and front
VAC_HW = 0.160                           # its half width
VAC_Y0 = 0.018                           # its lower edge (the bumper's)
VAC_PIVOT_Y = 0.205                      # the handle's foot


def vac_top(z):
    """The hood's height at z: low over the brush, up over the motor"""
    return 0.088 + (0.124 - 0.088) * smoothstep(0.09, -0.07, z)


def vac_halfw(z):
    """The hood's half width at z (its plan's corners rounded)"""
    rf, rb = 0.060, 0.030
    if z > VAC_Z1 - rf:
        d = z - (VAC_Z1 - rf)
        return VAC_HW - rf + math.sqrt(max(rf * rf - d * d, 0.0))
    if z < VAC_Z0 + rb:
        d = (VAC_Z0 + rb) - z
        return VAC_HW - rb + math.sqrt(max(rb * rb - d * d, 0.0))
    return VAC_HW


def vac_section(z, hw, top, n=5):
    """The hood's cross-section at z: a rounded box over its lower edge"""
    rt, rb = 0.026, 0.004
    pts = []
    # (corner centre x, y, start angle)
    for cx, cy, a0, r in ((hw - rb, VAC_Y0 + rb, -math.pi / 2, rb), (hw - rt, top - rt, 0.0, rt),
                          (-hw + rt, top - rt, math.pi / 2, rt), (-hw + rb, VAC_Y0 + rb, math.pi, rb)):
        for k in range(n + 1):
            a = a0 + (math.pi / 2) * k / n
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a), z))
    return pts


def vac_hood():
    zs = [VAC_Z0, VAC_Z0 + 0.004, VAC_Z0 + 0.012, VAC_Z0 + 0.022, VAC_Z0 + 0.030, -0.06, -0.02, 0.02, 0.06, 0.095,
          VAC_Z1 - 0.048, VAC_Z1 - 0.032, VAC_Z1 - 0.018, VAC_Z1 - 0.008, VAC_Z1 - 0.002, VAC_Z1]
    rings = []
    for z in zs:
        hw = max(vac_halfw(z), 0.07)
        top = vac_top(z) - 0.012 * (1 - math.sqrt(max(0.0, 1 - ((z - (VAC_Z1 - 0.02)) / 0.02) ** 2))) * (z > VAC_Z1 - 0.02)
        rings.append(vac_section(z, hw, top))
    s = loft(rings)
    # the headlight's housing on the nose
    lamp = rbox(0.100, 0.034, 0.040, r=0.008, seg=2).move(0, 0.071, VAC_Z1 - 0.012)
    # the bail's sockets each side at the back, and the motor's vent ribs
    socks = sum((cyl(0.017, 0.017, 0.020, seg=14, bevel_r=0.002).rot(0, 0, math.pi / 2).move(sx * (VAC_HW + 0.004), 0.072, VAC_Z0 + 0.030)
                 for sx in (-1, 1)), Shape())
    ribs = sum((rbox(0.004, 0.006, 0.050, r=0.0015, seg=1).move(x, vac_top(-0.07) + 0.001, -0.07) for x in (-0.03, -0.015, 0.0, 0.015, 0.03)), Shape())
    # the fill tube's collar on the motor
    collar = cyl(0.026, 0.028, 0.016, seg=18, bevel_r=0.002).move(0, vac_top(-0.065) + 0.006, -0.065)
    return s + lamp + socks + ribs + collar


def vac_rubber():
    outer = rrect_plan(2 * VAC_HW + 0.012, VAC_Z1 - VAC_Z0 + 0.012, 0.050, seg=5)
    inner = rrect_plan(2 * VAC_HW - 0.002, VAC_Z1 - VAC_Z0 - 0.002, 0.044, seg=5)
    zc = (VAC_Z0 + VAC_Z1) / 2
    bump = band([(x, z + zc) for x, z in outer], [(x, z + zc) for x, z in inner], VAC_Y0 - 0.002, VAC_Y0 + 0.022)
    bump = bevel(bump, 0.0025, seg=2)
    wheels = Shape()
    for (x, z, r, w) in ((-0.130, -0.088, 0.030, 0.020), (0.130, -0.088, 0.030, 0.020), (-0.110, 0.125, 0.017, 0.014),
                         (0.110, 0.125, 0.017, 0.014)):
        wh = lathe([(0.0, -w / 2), (r * 0.82, -w / 2), (r, -w * 0.3), (r, w * 0.3), (r * 0.82, w / 2), (0.0, w / 2)], seg=16)
        wheels = wheels + wh.rot(0, 0, math.pi / 2).move(x, r, z)
    return bump + wheels


def vac_lens():
    s = rbox(0.078, 0.022, 0.006, r=0.0025, seg=2)
    s.warp(lambda p: Vector((p.x, p.y, p.z + 0.0025 * max(0.0, 1 - (p.x / 0.039) ** 2) * (p.z > 0))))
    return s.move(0, 0.072, VAC_Z1 + 0.008)


def tube_z(y):
    """Where the handle's tube is at height y (it leans back 5.3 degrees)"""
    return -0.126 - (y - VAC_PIVOT_Y) * 0.0926


TUBE_R = 0.0127
VAC_TOP = 1.035                       # the tube's top, into the grip
CORD_HOOK_HI, CORD_HOOK_LO = 0.950, 0.420
CORD_R = 0.0034


def vac_metal():
    tube_ = rod([(0, VAC_PIVOT_Y, tube_z(VAC_PIVOT_Y)), (0, VAC_TOP, tube_z(VAC_TOP))], TUBE_R, seg=14, smooth=0)
    # the bail: from the sockets each side up and in to the tube's foot
    bail = Shape()
    for sx in (-1, 1):
        pts = [(sx * (VAC_HW + 0.012), 0.072, VAC_Z0 + 0.030), (sx * (VAC_HW + 0.016), 0.105, VAC_Z0 + 0.012),
               (sx * 0.120, 0.160, VAC_Z0 - 0.005), (sx * 0.050, 0.195, tube_z(0.195) + 0.002), (sx * 0.012, VAC_PIVOT_Y + 0.01, tube_z(VAC_PIVOT_Y + 0.01))]
        bail = bail + rod(pts, 0.0078, seg=10, smooth=2)
    ferrule = cyl(TUBE_R + 0.004, TUBE_R + 0.003, 0.040, seg=14, bevel_r=0.0015)
    ferrule = along_dir(ferrule.rot(0, 0, -math.pi / 2), (0, 1, -0.0926), (0, VAC_PIVOT_Y + 0.02, tube_z(VAC_PIVOT_Y + 0.02)))
    # the bag hook on the front, near the top
    yb = 0.985
    zf = tube_z(yb) + TUBE_R
    bag_hook = rod([(0, yb - 0.010, zf - 0.004), (0, yb - 0.008, zf + 0.012), (0, yb + 0.002, zf + 0.020), (0, yb + 0.016, zf + 0.022)],
                   0.0028, seg=6, smooth=1)
    clamp = cyl(TUBE_R + 0.0025, TUBE_R + 0.0025, 0.016, seg=14, bevel_r=0.001).move(0, yb - 0.010, tube_z(yb - 0.010))
    # the cord hooks on the back: the top one up, the bottom one down
    hooks = Shape()
    for yh, up in ((CORD_HOOK_HI, 1), (CORD_HOOK_LO, -1)):
        zb = tube_z(yh) - TUBE_R
        hooks = hooks + rod([(0, yh, zb + 0.004), (0, yh - up * 0.004, zb - 0.022), (0, yh + up * 0.006, zb - 0.040),
                             (0, yh + up * 0.024, zb - 0.044)], 0.0045, seg=8, smooth=1)
        hooks = hooks + cyl(TUBE_R + 0.003, TUBE_R + 0.003, 0.020, seg=14, bevel_r=0.001).move(0, yh, tube_z(yh))
    return tube_ + bail + ferrule + bag_hook + clamp + hooks


def vac_grip():
    y0 = VAC_TOP - 0.035
    # the switch housing at the grip's foot, the cord's way out at its back
    housing = rbox(0.040, 0.060, 0.044, r=0.008, seg=2).move(0, y0 + 0.020, tube_z(y0 + 0.020) - 0.004)
    toggle = rbox(0.012, 0.020, 0.010, r=0.003, seg=1).rot(0.3, 0, 0).move(0, y0 + 0.030, tube_z(y0 + 0.03) + 0.024)
    boot = cyl(0.007, 0.006, 0.020, seg=10, bevel_r=0.001).rot(math.pi / 2, 0, 0).move(0, y0 + 0.006, tube_z(y0) - 0.030)
    # the grip: up from the housing and bent back, ribbed for the fingers
    pts = [(0, y0 + 0.045, tube_z(y0 + 0.045)), (0, y0 + 0.100, tube_z(y0 + 0.100) - 0.004), (0, y0 + 0.150, tube_z(y0 + 0.15) - 0.020),
           (0, y0 + 0.172, tube_z(y0 + 0.17) - 0.040)]
    grip = rod(pts, 0.0165, seg=14, smooth=2)
    ribs = Shape()
    for k in range(4):
        t = 0.2 + 0.18 * k
        p = Vector(pts[0]).lerp(Vector(pts[1]), t * 1.3) if t < 0.7 else Vector(pts[1]).lerp(Vector(pts[2]), (t - 0.7) * 2)
        ribs = ribs + torus(0.0166, 0.0016, seg=14, tseg=4).rot(math.pi / 2, 0, 0).move(p.x, p.y, p.z)
    cap = sphere(0.0166, seg=14, rings=7).scale(1, 1, 1).move(*pts[-1])
    return housing + toggle + boot + grip + ribs + cap


# the bag: rings from its foot (on the fill tube) to its top hem (on the hook)
BAG_Y0, BAG_Y1 = 0.215, 0.990
BAG_TAB = ((0.0, 0.026, 0.026), (0.05, 0.060, 0.044), (0.13, 0.104, 0.068), (0.30, 0.113, 0.068), (0.52, 0.108, 0.056),
           (0.75, 0.098, 0.044), (0.90, 0.080, 0.030), (0.97, 0.060, 0.016), (1.0, 0.045, 0.008))


def _bag_ab(t):
    for (t0, a0, b0), (t1, a1, b1) in zip(BAG_TAB, BAG_TAB[1:]):
        if t <= t1:
            k = smoothstep(t0, t1, t) if t1 > t0 else 1.0
            return a0 + (a1 - a0) * k, b0 + (b1 - b0) * k
    return BAG_TAB[-1][1], BAG_TAB[-1][2]


def vac_bag():
    rings = []
    n = 22
    nr = 34
    for i in range(nr + 1):
        t = i / nr
        y = BAG_Y0 + (BAG_Y1 - BAG_Y0) * t
        a, b = _bag_ab(t)
        back = tube_z(y) + TUBE_R + 0.005
        zc = back + 0.6 * b
        ring = []
        for j in range(n):
            ang = TAU * j / n
            ca, sa = math.cos(ang), math.sin(ang)
            # flat against the handle at its back, a soft sack in front
            zz = sa * b if sa > 0 else sa * b * 0.6
            x = ca * a
            # the cloth's slack: long vertical folds, a crease where it sags
            fold = 0.0035 * math.sin(ang * 5 + t * 3) * smoothstep(0.08, 0.3, t) * smoothstep(1.0, 0.85, t)
            sag = -0.006 * math.exp(-((t - 0.24) / 0.05) ** 2) * max(0.0, sa)
            ring.append((x * (1 + fold * 6), y + 0.004 * math.sin(ang * 3 + 1) * smoothstep(0.1, 0.2, t) * smoothstep(0.95, 0.8, t),
                         zc + zz + sag + fold))
        rings.append(ring)
    s = loft(rings)
    s = noisy(s, 0.0015, scale=18, seed=9)
    # the hem's eyelet hanging on the hook
    zf = tube_z(0.985) + TUBE_R
    eyelet = torus(0.006, 0.0014, seg=12, tseg=4).move(0, 0.996, zf + 0.012)
    return s + eyelet


def vac_fill_tube():
    zt = tube_z(BAG_Y0) + TUBE_R + 0.031
    pts = [(0, vac_top(-0.065) + 0.008, -0.065), (0, vac_top(-0.065) + 0.040, -0.068), (0, BAG_Y0 - 0.045, zt + 0.004),
           (0, BAG_Y0 + 0.012, zt)]
    elbow = rod(pts, 0.022, seg=14, smooth=2)
    clamp = torus(0.0275, 0.0035, seg=18, tseg=5).rot(math.pi / 2, 0, 0).move(0, BAG_Y0 + 0.010, zt)
    return elbow + clamp


# the cord wound on the hooks: up over the top one, down under the bottom
# one, four times round, the last time on down to hang free
CORD_FREE = (-0.010, 0.330, None)


def _cord_path():
    pts = []
    yt, yb = CORD_HOOK_HI + 0.012, CORD_HOOK_LO - 0.012
    zt, zb = tube_z(CORD_HOOK_HI) - TUBE_R - 0.034, tube_z(CORD_HOOK_LO) - TUBE_R - 0.034
    # from the switch housing's boot to the top hook
    y0 = VAC_TOP - 0.035
    pts += [(0, y0 + 0.006, tube_z(y0) - 0.042), (0.002, y0 - 0.010, tube_z(y0) - 0.050), (0.004, yt + 0.006, zt - 0.004)]
    loops = 4
    for k in range(loops):
        x = -0.010 + 0.0065 * k
        dz = 0.0035 * ((k % 2) * 2 - 1)
        # down (bowing out a little), round under the bottom hook
        for t in (0.25, 0.5, 0.75):
            y = yt + (yb - yt) * t
            pts.append((x + 0.002 * math.sin(k + t * 3), y, zt + (zb - zt) * t - 0.012 * math.sin(math.pi * t) + dz))
        pts += [(x + 0.001, yb, zb + dz), (x + 0.0015, yb - 0.008, zb + 0.006 + dz), (x + 0.002, yb - 0.004, zb + 0.014 + dz)]
        if k == loops - 1:
            break
        # back up, round over the top hook
        for t in (0.25, 0.5, 0.75):
            y = yb + (yt - yb) * t
            pts.append((x + 0.004 + 0.002 * math.cos(k + t * 2), y, zb + (zt - zb) * t - 0.010 * math.sin(math.pi * t) + 0.004 - dz))
        pts += [(x + 0.0045, yt, zt + 0.004 - dz), (x + 0.005, yt + 0.008, zt + 0.010 - dz), (x + 0.0055, yt + 0.004, zt + 0.016 - dz)]
    # the last time: off the bottom hook and down
    x = pts[-1][0]
    pts += [(x - 0.002, yb - 0.035, zb + 0.020), (CORD_FREE[0], CORD_FREE[1], zb + 0.016)]
    return pts


def vac_cord():
    return rod(_cord_path(), CORD_R, seg=6, smooth=1)


def vac_plug():
    body = rbox(0.026, 0.012, 0.030, r=0.004, seg=2).move(0, 0.006, 0.0)
    relief = cyl(0.0058, CORD_R + 0.0004, 0.024, seg=10, bevel_r=0.001).rot(math.pi / 2, 0, 0).move(0, 0.006, -0.026)
    ribs = sum((rbox(0.027, 0.0125, 0.0015, r=0.0005, seg=1).move(0, 0.006, z) for z in (-0.009, -0.005)), Shape())
    return body + relief + ribs


def vac_plug_blades():
    out = Shape()
    for x in (-0.0063, 0.0063):
        b = rbox(0.0015, 0.0063, 0.016, r=0.0004, seg=1).move(x, 0.006, 0.015 + 0.008)
        b = cut(b, cyl(0.0016, 0.0016, 0.004, seg=8, bevel_r=0).rot(0, 0, math.pi / 2).move(x, 0.006, 0.027))
        out = out + b
    return out


auto('Real__vacuumHood', vac_hood, 1700)
auto('Real__vacuumRubber', vac_rubber, 1300)
auto('Real__vacuumLens', vac_lens, 160)
auto('Real__vacuumMetal', vac_metal, 2400)
auto('Real__vacuumGrip', vac_grip, 1400)
auto('Real__vacuumBag', vac_bag, 1700, smooth=60)
auto('Real__vacuumFillTube', vac_fill_tube, 700)
auto('Real__vacuumCord', vac_cord, 2200, smooth=60)
auto('Real__vacuumPlug', vac_plug, 500)
auto('Real__vacuumPlugBlades', vac_plug_blades, 200)


# ============================================================ boot tray ===
BT_W, BT_D, BT_H = 0.810, 0.410, 0.024
BT_FLOOR, BT_RIB = 0.0055, 0.0040


def boot_tray():
    outer = rrect_plan(BT_W, BT_D, 0.030, seg=4)
    inner = rrect_plan(BT_W - 0.026, BT_D - 0.026, 0.018, seg=4)
    lip = band(outer, inner, 0.0, BT_H)
    lip = bevel(lip, 0.003, seg=2)
    floor_ = along_y(rrect_plan(BT_W - 0.020, BT_D - 0.020, 0.022, seg=4), BT_FLOOR).move(0, BT_FLOOR / 2, 0)
    ribs = Shape()
    n = 19
    for i in range(n):
        x = -BT_W / 2 + 0.050 + (BT_W - 0.100) * i / (n - 1)
        rib = rbox(0.0075, BT_RIB + 0.001, BT_D - 0.080, r=0.0028, seg=1).move(x, BT_FLOOR + (BT_RIB - 0.001) / 2, 0)
        ribs = ribs + rib
    # the moulding's drain channel along each long side
    s = lip + floor_ + ribs
    # trodden: its floor a little dished, a corner kinked up
    s.warp(lambda p: Vector((p.x, p.y - 0.0015 * max(0.0, 1 - (p.x / 0.4) ** 2) * max(0.0, 1 - (p.z / 0.2) ** 2) * (p.y > 0.003)
                             + 0.004 * smoothstep(0.30, 0.40, p.x) * smoothstep(0.12, 0.2, p.z), p.z)))
    return s


auto('Real__bootTray', boot_tray, 1600)


# ======================================================= umbrella stand ===
US_R, US_H = 0.100, 0.490


def umbrella_stand():
    t = 0.0008
    out_ = [(0.0, 0.0035), (0.090, 0.0035), (0.0945, 0.0018), (0.0985, 0.0), (0.1015, 0.0022), (0.1025, 0.0065),
            (0.1018, 0.0105), (US_R, 0.014), (US_R, 0.060), (US_R + 0.0022, 0.064), (US_R + 0.0026, 0.067),
            (US_R + 0.0022, 0.070), (US_R, 0.074), (US_R, 0.420), (US_R + 0.0022, 0.424), (US_R + 0.0026, 0.427),
            (US_R + 0.0022, 0.430), (US_R, 0.434), (US_R, US_H - 0.012), (US_R + 0.0020, US_H - 0.008),
            (US_R + 0.0034, US_H - 0.0035), (US_R + 0.0028, US_H - 0.0005), (US_R + 0.0008, US_H), (US_R - 0.0012, US_H - 0.0012),
            (US_R - t, US_H - 0.004), (US_R - t, 0.434), (US_R - t - 0.0022, 0.427), (US_R - t, 0.420), (US_R - t, 0.074),
            (US_R - t - 0.0022, 0.067), (US_R - t, 0.060), (US_R - t, 0.016), (0.096, 0.0135), (0.090, 0.0125), (0.0, 0.0125)]
    s = lathe(out_, seg=32)
    # the lap seam down its back, and its bottom seamed on
    seam = rbox(0.014, US_H - 0.040, 0.0012, r=0.0005, seg=1).move(0, (US_H - 0.040) / 2 + 0.018, -(US_R + 0.0004))
    s = s + seam
    # knocks it took
    for (a, y, d) in ((0.7, 0.22, 0.003), (3.6, 0.36, 0.0022), (5.1, 0.11, 0.0018)):
        c = Vector((US_R * math.cos(a), y, US_R * math.sin(a)))
        s.warp(lambda p, c=c, d=d: p - Vector((p.x, 0, p.z)).normalized() * d * math.exp(-((p - c).length / 0.025) ** 2)
               if math.hypot(p.x, p.z) > 0.08 else p)
    return s


auto('Real__umbrellaStandBody', umbrella_stand, 2300, kind='cyl')
