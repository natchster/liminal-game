"""t10: the rare big things of the parlour and the office (Props P3,
docs/PROPS.md): a hanging bookshelf, a steel locker, a stack of 45s, a
fireplace, a carved cuckoo clock, a giltwood wall mirror and a brandy
snifter on its coaster. Sources and sizes: t10.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is modelled at
its real size in its prop's own frame: x across its front, y up, z out of
its front; a wall thing's wall is the plane z = 0, a floor thing's floor the
plane y = 0. Every part is named `Real__<prop><Part>`:

record*        a 7 in single (45 rpm) lying flat: Sleeve (the paper company
               sleeve, 184 mm square, two sheets with the 95 mm hole, open
               along +x), Vinyl (175 mm, the 38 mm hole, the raised label
               area and rim; its bottom on y = 0), VinylLow (the same, cut
               down, for the records hidden in their sleeves), Label (the
               paper label on its top)
snifter*       a 5.5 oz brandy snifter (the Libbey Embassy 3702 class) on a
               coaster: Bowl, Foot (its foot and stem; its foot on y = 0),
               BrandyLow / Mid / High (the brandy in its bowl, open at its
               surface), CoasterWood (turned, its rim), CoasterCork,
               CoasterLeather (square, stitched), Rocks (a 2 oz heavy-base
               whisky glass) and RocksDregs (what dried in it)
ornateMirror*  a small Italian giltwood mirror, 300 x 400 mm over its moulding:
               Frame (the moulding swept round, its rebate; y = 0 the glass's
               middle) and Crest (the rocaille crest, the corners' leaves,
               the apron: carved ornament, gilt apart)
bookshelf*     a hanging wall bookshelf, 600 x 160 mm, 300 mm tall: Sides
               (scrolled at the foot), Boards (top and bottom), Back
               (hardboard); its books are r02's
locker*        a single-tier steel locker, 12 x 12 x 60 in on 6 in legs:
               Case (sides, back, top, bottom, the frame round the door, the
               legs and feet, the hinges), Door (the pan, its louvres and
               their hoods, the latch's pocket), Latch (the cup's rim, the
               lift handle, the padlock staple), Plate (the number plate and
               its rivets, at its own origin)
cuckoo*        a carved Black Forest cuckoo clock (y = 0 the dial's middle):
               Case (back board, case, mouldings, the cuckoo's door, the
               dial's seat), Carving (oak leaves, acorns, the bird), Dial,
               Hour and Minute (bone hands, up +y from their boss at the
               origin), PendRod (its hook at the origin) and PendBob (a
               carved leaf), Chains (brass, two weighted strands and two free
               ends with rings), Weights (two cast pine cones)
fireplace*     a Victorian bedroom fireplace against the wall: Mantel (painted
               pine), Insert (the cast-iron arched register, its canopy),
               Grate (the basket and the fret), Firebox (the firebrick back
               and cheeks), Hearth (slate), Ash, LogA..LogD (charred ends,
               along x, at their own origin); a lit fire's Embers (the bed
               of coals), LogAGlow..LogDGlow (each end's underside burning)
               and FlameA..C (tongues of flame, tall to short, up +y from
               their root at the origin)
"""
from propkit import *
import propkit
import random
import zlib


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, kind='box', flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth, flat=flat)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def ring_lathe(prof, seg=48):
    """Revolve a closed loop of [(r, y)] (every r > 0) round +Y."""
    n = len(prof)
    v, f = [], []
    for i in range(seg):
        a = TAU * i / seg
        for (r, y) in prof:
            v.append((r * math.sin(a), y, r * math.cos(a)))
    for i in range(seg):
        i2 = (i + 1) % seg
        for k in range(n):
            k2 = (k + 1) % n
            f.append((i * n + k, i * n + k2, i2 * n + k2, i2 * n + k))
    return propkit._fix_winding(Shape(v, f))


def annulus(r0, r1, seg=40, y=0.0):
    """A flat ring facing +Y (single sided)."""
    v, f = [], []
    for i in range(seg):
        a = TAU * i / seg
        v.append((r0 * math.sin(a), y, r0 * math.cos(a)))
        v.append((r1 * math.sin(a), y, r1 * math.cos(a)))
    for i in range(seg):
        j = (i + 1) % seg
        f.append((2 * i, 2 * i + 1, 2 * j + 1, 2 * j))
    s = Shape(v, f)
    # face up
    if (s.v[1] - s.v[0]).cross(s.v[3] - s.v[0]).y < 0:
        s.f = [tuple(reversed(q)) for q in s.f]
    return s


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle centred on
    the origin: d in from its outer edge, z out from the wall; the corners
    mitred."""
    v, f = [], []
    n = len(profile)
    for (d, z) in profile:
        x, y = w / 2 - d, h / 2 - d
        v += [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    for i in range(n):
        j = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, j * 4 + k2, j * 4 + k))
    return propkit._fix_winding(Shape(v, f))


def loft(sections, cap0=True, cap1=True):
    """Join rings of points (each the same count, in the same turn) into a
    closed tube; the ends capped by a fan to their middle."""
    n = len(sections[0])
    v, f = [], []
    for s in sections:
        v += [Vector(p) for p in s]
    for i in range(len(sections) - 1):
        for k in range(n):
            k2 = (k + 1) % n
            f.append((i * n + k, i * n + k2, (i + 1) * n + k2, (i + 1) * n + k))
    if cap0:
        c = sum((Vector(p) for p in sections[0]), Vector((0, 0, 0))) / n
        v.append(c)
        ci = len(v) - 1
        for k in range(n):
            f.append((ci, (k + 1) % n, k))
    if cap1:
        c = sum((Vector(p) for p in sections[-1]), Vector((0, 0, 0))) / n
        v.append(c)
        ci = len(v) - 1
        base = (len(sections) - 1) * n
        for k in range(n):
            f.append((ci, base + k, base + (k + 1) % n))
    return propkit._fix_winding(Shape(v, f))


def warp_noise(s, amount, scale, seed, mask=None, axis=None):
    """Push each vertex by noise, `mask(p)` (0..1) of it."""
    off = Vector((seed * 17.3, seed * 5.1, seed * 9.7))
    ax = Vector(axis).normalized() if axis else None
    for i, p in enumerate(s.v):
        k = mask(p) if mask else 1.0
        if k <= 0:
            continue
        q = p * scale + off
        if ax:
            s.v[i] = p + ax * noise.noise(q) * amount * k
        else:
            s.v[i] = p + noise3(q) * amount * k
    return s


# =============================================================== the 45s ===
# A 7 in single: the disc 175 mm (6 7/8 in), its 38.2 mm (1 1/2 in) centre
# hole, the paper label 3 1/2 in (89 mm) across; the company sleeve 7 1/4 in
# (184 mm) square with a 95 mm hole (RIAA / IEC 60098; see t10.notes.md).
SLEEVE = 0.184
SLEEVE_HOLE = 0.0475
SHEET = 0.0003                 # one sheet of the sleeve (drawn a little thick)
VINYL_R, VINYL_HOLE = 0.0875, 0.0191
LABEL_R = 0.0445
VINYL_T = 0.0019               # at its label area (the grooved band 1.3 mm)
SLEEVE_T = 2 * SHEET + VINYL_T + 0.0001


def sleeve_sheet(y0, th, n=32, m=2):
    """One sheet of the sleeve: a square with the round hole, `th` thick,
    its bottom at y0."""
    half = SLEEVE / 2
    rings = []
    for i in range(n):
        a = TAU * i / n
        c, s = math.cos(a), math.sin(a)
        k = half / max(abs(c), abs(s))
        rings.append(((SLEEVE_HOLE * c, SLEEVE_HOLE * s), (k * c, k * s)))
    v, f = [], []
    for y in (y0, y0 + th):
        for (ix, iz), (ox, oz) in rings:
            for j in range(m + 1):
                t = j / m
                t = t * t * 0.4 + t * 0.6      # finer near the hole
                v.append((ix + (ox - ix) * t, y, iz + (oz - iz) * t))
    per = n * (m + 1)
    idx = lambda layer, i, j: layer * per + (i % n) * (m + 1) + j
    for i in range(n):
        for j in range(m):
            f.append((idx(0, i, j), idx(0, i + 1, j), idx(0, i + 1, j + 1), idx(0, i, j + 1)))
            f.append((idx(1, i, j), idx(1, i, j + 1), idx(1, i + 1, j + 1), idx(1, i + 1, j)))
        # the hole's edge and the outer edge
        f.append((idx(0, i, 0), idx(1, i, 0), idx(1, i + 1, 0), idx(0, i + 1, 0)))
        f.append((idx(0, i, m), idx(0, i + 1, m), idx(1, i + 1, m), idx(1, i, m)))
    return propkit._fix_winding(Shape(v, f))


def record_sleeve():
    bot = sleeve_sheet(0.0, SHEET)
    top = sleeve_sheet(SLEEVE_T - SHEET, SHEET)
    # the top sheet cockled a little, up only (the record is under it)
    warp_noise(top, 0.0007, 26.0, 3, mask=lambda p: smoothstep(SLEEVE_HOLE, SLEEVE_HOLE + 0.03, math.hypot(p.x, p.z)), axis=(0, 1, 0))
    for p in top.v:
        p.y = max(p.y, SLEEVE_T - SHEET - 0.00005) if p.y < SLEEVE_T - SHEET * 0.5 else max(p.y, SLEEVE_T - 0.00005)
    # the folds along three sides (the mouth open along +x)
    half = SLEEVE / 2
    folds = Shape()
    for (x, z, w, d) in [(0, -half, SLEEVE, 0.0), (0, half, SLEEVE, 0.0), (-half, 0, 0.0, SLEEVE)]:
        folds = folds + box(max(w, SHEET * 2), SLEEVE_T - 2 * SHEET, max(d, SHEET * 2)).move(x, SLEEVE_T / 2, z)
    return bot + top + folds


def vinyl_profile(fine=True):
    top = [(VINYL_HOLE + 0.0004, VINYL_T), (0.0440, VINYL_T), (0.0468, 0.0013), (0.0830, 0.0013),
           (0.0848, 0.0017), (0.0866, 0.0016), (VINYL_R, VINYL_T / 2)]
    if not fine:
        top = [(VINYL_HOLE + 0.0004, VINYL_T), (0.0450, VINYL_T), (0.0468, 0.0013), (0.0855, 0.0016), (VINYL_R, VINYL_T / 2)]
    bottom = [(r, VINYL_T - y) for r, y in reversed(top[:-1])]
    hole = [(VINYL_HOLE, 0.0004), (VINYL_HOLE, VINYL_T - 0.0004)]
    return hole + top + bottom


def record_vinyl():
    return ring_lathe(vinyl_profile(True), seg=44)


def record_vinyl_low():
    return ring_lathe(vinyl_profile(False), seg=24)


def record_label():
    return annulus(VINYL_HOLE + 0.0003, LABEL_R, seg=40, y=VINYL_T + 0.00004)


auto('Real__recordSleeve', record_sleeve, 900, smooth=30.0)
auto('Real__recordVinyl', record_vinyl, 1400)
auto('Real__recordVinylLow', record_vinyl_low, 560)
auto('Real__recordLabel', record_label, 100)


# ========================================================== the snifter ===
# Libbey Embassy 3702 (5.5 oz): 4 1/8 in (105 mm) tall, 2 7/8 in (73 mm)
# at its widest, 2 in (51 mm) across the rim, the foot 2 3/8 in (60 mm).
SN_YC, SN_A = 0.058, 0.0365          # the bowl: its widest, half across
SN_BLO, SN_BUP = 0.040, 0.0659       # its half heights below and above
SN_WALL = 0.0012
SN_BLO_IN = 0.034                    # the inside's, below (its thick bottom)
SN_RIM_T = math.radians(45.7)        # where the rim is (r 25.5 mm, y 105 mm)
SN_LEVELS = (0.040, 0.046, 0.052)    # the brandy's surface: 25, 44, 65 ml
SN_FOOT_Y = 0.0                      # (the coaster's top is set in the game)


def sn_outer(t):
    b = SN_BLO if t < 0 else SN_BUP
    return (SN_A * math.cos(t), SN_YC + b * math.sin(t))


def sn_inner(t):
    b = SN_BLO_IN if t < 0 else SN_BUP - SN_WALL
    return ((SN_A - SN_WALL) * math.cos(t), SN_YC + b * math.sin(t))


def sn_inner_r(y):
    """The bowl's inside radius at height y (below its widest)."""
    k = (y - SN_YC) / SN_BLO_IN
    return (SN_A - SN_WALL) * math.sqrt(max(0.0, 1 - k * k))


def snifter_profile():
    prof = [(0.0, 0.0030), (0.0150, 0.0022), (0.0285, 0.0003), (0.0300, 0.0009), (0.0300, 0.0021),
            (0.0272, 0.0034), (0.0150, 0.0050), (0.0080, 0.0080),
            # the short stem and its knop
            (0.0050, 0.0110), (0.0046, 0.0135), (0.0060, 0.0150), (0.0050, 0.0165), (0.0072, 0.0185)]
    for d in (-68, -55, -42, -28, -14, 0, 12, 24, 33, 40):
        prof.append(sn_outer(math.radians(d)))
    prof.append(sn_outer(SN_RIM_T))
    r, y = sn_outer(SN_RIM_T)
    prof.append((r - 0.0005, y + 0.0005))         # the rim, fire-rounded
    for d in (45.0, 40, 33, 24, 12, 0, -14, -28, -42, -55, -68, -80):
        prof.append(sn_inner(math.radians(d)))
    prof.append((0.0, SN_YC - SN_BLO_IN))
    return prof


SN_JOIN = (0.0072, 0.0185)           # where the stem flares into the bowl


def snifter_bowl():
    """The bowl, from where the stem flares into it: a part of its own, so
    its foot and stem can be drawn as the thicker glass they are."""
    prof = snifter_profile()
    return lathe([(0.0, SN_JOIN[1])] + prof[prof.index(SN_JOIN):], seg=30)


def snifter_foot():
    prof = snifter_profile()
    return lathe(prof[:prof.index(SN_JOIN) + 1] + [(0.0, SN_JOIN[1])], seg=24)


def snifter_brandy(level):
    def build():
        prof = [(0.0, SN_YC - SN_BLO_IN + 0.0003)]
        y0 = SN_YC - SN_BLO_IN + 0.0003
        for k in range(1, 7):
            y = y0 + (level - y0) * (k / 6) ** 0.7
            prof.append((sn_inner_r(y) - 0.0002, y))
        return lathe(prof, seg=36, cap_bottom=False, cap_top=False)
    return build


def coaster_wood():
    prof = [(0.0, 0.0), (0.0520, 0.0), (0.0540, 0.0018), (0.0540, 0.0062), (0.0530, 0.0080), (0.0478, 0.0080),
            (0.0466, 0.0068), (0.0450, 0.0062), (0.0, 0.0062)]
    return lathe(prof, seg=48)


def coaster_cork():
    return cyl(0.050, 0.050, 0.0042, seg=40, bevel_r=0.0008, bevel_seg=1).move(0, 0.0021, 0)


def coaster_leather():
    out = rounded_rect(0.100, 0.100, 0.009, seg=3)
    s = prism(out, 0.0042, bevel_r=0.0012, bevel_seg=2)
    return s.rot(x=-math.pi / 2).move(0, 0.0021, 0)


# A 2 oz heavy-base whisky glass (the Libbey 48 class): 2 in (51 mm) across
# its rim, 1 1/2 in (38 mm) at its foot, 2 3/8 in (60 mm) tall, its base 14 mm
RK_R, RK_H, RK_BASE = 0.0255, 0.060, 0.014


def _rk_out(y):
    return 0.0192 + (RK_R - 0.0192) * min(1.0, max(0.0, y / RK_H))


def rocks_glass():
    prof = [(0.0, 0.0006), (0.0178, 0.0004), (0.0190, 0.0018), (_rk_out(0.010), 0.0100),
            (RK_R, RK_H - 0.0008), (RK_R - 0.0007, RK_H), (RK_R - 0.0016, RK_H - 0.0006),
            (_rk_out(0.020) - 0.0016, 0.0200), (_rk_out(RK_BASE) - 0.0020, RK_BASE + 0.0012),
            (_rk_out(RK_BASE) - 0.0045, RK_BASE), (0.0, RK_BASE - 0.0004)]
    return lathe(prof, seg=36)


def rocks_dregs():
    r = _rk_out(RK_BASE) - 0.0021
    prof = [(0.0, RK_BASE - 0.0002), (r - 0.0026, RK_BASE + 0.0001), (r - 0.0002, RK_BASE + 0.0014),
            (r, RK_BASE + 0.0026), (r - 0.0026, RK_BASE + 0.0021), (0.0, RK_BASE + 0.0020)]
    return lathe(prof, seg=32)


auto('Real__snifterBowl', snifter_bowl, 1800, kind='cyl')
auto('Real__snifterFoot', snifter_foot, 700, kind='cyl')
auto('Real__snifterBrandyLow', snifter_brandy(SN_LEVELS[0]), 500, kind='cyl')
auto('Real__snifterBrandyMid', snifter_brandy(SN_LEVELS[1]), 500, kind='cyl')
auto('Real__snifterBrandyHigh', snifter_brandy(SN_LEVELS[2]), 500, kind='cyl')
auto('Real__snifterCoasterWood', coaster_wood, 900)
auto('Real__snifterCoasterCork', coaster_cork, 400)
auto('Real__snifterCoasterLeather', coaster_leather, 500)
auto('Real__snifterRocks', rocks_glass, 900, kind='cyl')
auto('Real__snifterRocksDregs', rocks_dregs, 400, kind='cyl')


# ============================================================ carvings ===
def leaf(length, width, lobes=3, lobe=0.35, depth=0.004, fold=0.35, curl=0.0, twist=0.0, n=14, tip=0.9, seed=0):
    """A carved leaf: its stem at the origin, its tip up +y, its face +z.
    `lobes` round lobes down each side (an oak's); `fold` lifts its edges
    from the midrib, `curl` bends its tip out (+z), `twist` turns it."""
    sections = []
    for i in range(n + 1):
        t = i / n
        env = math.sin(math.pi * min(1.0, t ** tip)) * (1 - 0.15 * t)
        lob = 1 - lobe * (0.5 + 0.5 * math.cos(lobes * TAU * t + 0.6 + math.pi)) ** 3 if lobes else 1.0
        w = max(0.0006, width / 2 * env * lob)
        y = t * length
        zc = curl * length * t * t
        mid = (0.0, y, zc + depth * 0.5)
        sec_top = [(-w, y, zc + fold * w), mid, (w, y, zc + fold * w)]
        sec = []
        for (x, yy, z) in sec_top:
            sec.append(Vector((x, yy, z)))
        # turned about the midrib by `twist` toward the tip
        a = twist * t
        rot = Matrix.Rotation(a, 3, 'Y')
        sec = [rot @ (p - Vector((0, y, 0))) + Vector((0, y, 0)) for p in sec]
        sections.append(sec)
    v, f = [], []
    for sec in sections:            # top (3), then bottom (3): a thin slab
        for p in sec:
            v.append(p.copy())
        for p in sec:
            v.append(p - Vector((0, 0, depth)))
    m = 6
    for i in range(n):
        a, b = i * m, (i + 1) * m
        for k in range(2):
            f.append((a + k, a + k + 1, b + k + 1, b + k))               # top
            f.append((a + 3 + k, b + 3 + k, b + 4 + k, a + 4 + k))       # bottom
        f.append((a + 0, b + 0, b + 3, a + 3))                           # left edge
        f.append((a + 2, a + 5, b + 5, b + 2))                           # right edge
    f.append((0, 3, 4, 1))
    f.append((1, 4, 5, 2))
    e = n * m
    f.append((e + 0, e + 1, e + 4, e + 3))
    f.append((e + 1, e + 2, e + 5, e + 4))
    s = propkit._fix_winding(Shape(v, f))
    if seed:
        warp_noise(s, depth * 0.35, 90.0, seed)
    return s


def place(s, x, y, z, rz=0.0, rx=0.0, ry=0.0, k=1.0):
    return s.scale(k).rot(rx, ry, rz, 'YXZ').move(x, y, z)


# ===================================================== the gilt mirror ===
# A small Italian giltwood mirror of the 1950s-60s in the rococo revival
# (gesso over carved wood, water gilt): 11 3/4 x 15 3/4 in over its
# moulding (300 x 400 mm), the glass 200 x 300 mm in sight, a pierced
# rocaille crest, leaf corners and a small apron.
OM_W, OM_H, OM_MOULD = 0.300, 0.400, 0.050
OM_FACE = [(0.0, 0.0120), (0.0015, 0.0155), (0.0040, 0.0170), (0.0065, 0.0165), (0.0085, 0.0140),
           (0.0120, 0.0150), (0.0170, 0.0200), (0.0230, 0.0260), (0.0290, 0.0290), (0.0330, 0.0285),
           (0.0370, 0.0250), (0.0405, 0.0205), (0.0430, 0.0195), (0.0455, 0.0215), (0.0480, 0.0205),
           (0.0500, 0.0170)]
OM_GLASS_Z = 0.0125            # the glass's face, behind the sight's lip


def mirror_frame():
    prof = OM_FACE + [(OM_MOULD, OM_GLASS_Z), (OM_MOULD - 0.006, OM_GLASS_Z), (OM_MOULD - 0.006, 0.0), (0.0, 0.0)]
    return rect_sweep(prof, OM_W, OM_H)


def relief_fan(cx, cy, r_of, z_of, a0, a1, na=24, nr=6, z0=0.002, rho0=0.06):
    """A carved relief over a fan: (x, y) = c + rho R(a) (cos a, sin a),
    its face at z_of(a, rho), its back flat at z0."""
    top, bot = [], []
    for i in range(na + 1):
        a = a0 + (a1 - a0) * i / na
        R = r_of(a)
        for j in range(nr + 1):
            rho = rho0 + (1 - rho0) * j / nr
            x, y = cx + rho * R * math.cos(a), cy + rho * R * math.sin(a)
            top.append((x, y, z_of(a, rho)))
            bot.append((x, y, z0))
    v = top + bot
    off = len(top)
    idx = lambda i, j: i * (nr + 1) + j
    f = []
    for i in range(na):
        for j in range(nr):
            f.append((idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1)))
            f.append((off + idx(i, j), off + idx(i, j + 1), off + idx(i + 1, j + 1), off + idx(i + 1, j)))
        f.append((idx(i, nr), idx(i + 1, nr), off + idx(i + 1, nr), off + idx(i, nr)))
        f.append((idx(i, 0), off + idx(i, 0), off + idx(i + 1, 0), idx(i + 1, 0)))
    for i in (0, na):
        for j in range(nr):
            f.append((idx(i, j), idx(i, j + 1), off + idx(i, j + 1), off + idx(i, j)))
    return propkit._fix_winding(Shape(v, f))


def shell(cx, cy, R, ribs=9, a0=0.25, a1=math.pi - 0.25, lift=0.012, z0=0.002, down=False):
    sgn = -1 if down else 1
    r_of = lambda a: R * (0.86 + 0.14 * abs(math.cos(ribs * (a - a0) / (a1 - a0) * math.pi)))
    z_of = lambda a, rho: z0 + lift * (0.35 + 0.65 * (1 - rho) ** 0.6) + 0.0035 * abs(math.cos(ribs * (a - a0) / (a1 - a0) * math.pi)) * rho
    if down:
        a0, a1 = -a1, -a0
    return relief_fan(cx, cy, r_of, z_of, a0, a1, na=ribs * 4, nr=5, z0=z0)


def c_scroll(cx, cy, R0, turns, side, z=0.012, r=0.0042, start=0.0, n=22):
    """A C-scroll: a rod curling inward from (cx, cy) + R0, tapering."""
    pts = []
    for i in range(n + 1):
        t = i / n
        a = start + side * turns * TAU * t
        rr = R0 * (1 - 0.72 * t)
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a), z + 0.004 * math.sin(math.pi * t)))
    return tube(pts, r, seg=7, r_end=r * 0.45)


def mirror_crest():
    top = OM_H / 2
    s = shell(0.0, top - 0.004, 0.060, ribs=9, lift=0.014)
    # the C-scrolls either side of the shell, curling up and in
    for side in (-1, 1):
        s = s + c_scroll(side * 0.062, top + 0.022, 0.030, 0.80, -side, start=(math.pi if side > 0 else 0.0) + side * 0.4)
        s = s + c_scroll(side * 0.105, top + 0.006, 0.018, 0.75, side, z=0.011, r=0.0034, start=(math.pi if side < 0 else 0.0))
        # acanthus leaves running from the scrolls out along the top rail
        s = s + place(leaf(0.070, 0.024, lobes=4, lobe=0.45, depth=0.005, fold=0.5, curl=0.45, seed=3 + side),
                      side * 0.030, top + 0.012, 0.010, rz=-side * 1.05)
        s = s + place(leaf(0.050, 0.020, lobes=3, lobe=0.4, depth=0.004, fold=0.5, curl=0.5, seed=7 + side),
                      side * 0.020, top + 0.040, 0.012, rz=-side * 0.35)
    # a bud on the crest's top
    s = s + place(leaf(0.030, 0.018, lobes=2, lobe=0.3, depth=0.005, fold=0.6, curl=0.6, seed=11), 0.0, top + 0.052, 0.014)
    # the corners break out in leaves, each a cluster of three
    for sx in (-1, 1):
        for sy in (-1, 1):
            cx, cy = sx * OM_W / 2, sy * OM_H / 2
            base = math.atan2(sy, sx) - math.pi / 2          # pointing out of the corner
            for k, (da, ln) in enumerate([(-0.75, 0.040), (0.0, 0.048), (0.75, 0.040)]):
                s = s + place(leaf(ln, 0.020, lobes=3, lobe=0.45, depth=0.0045, fold=0.5, curl=0.5, seed=20 + k + 3 * (sx + 2 * sy)),
                              cx - sx * 0.012, cy - sy * 0.012, 0.014, rz=base + da)
            s = s + sphere(0.007, seg=10).scale(1, 1, 0.6).move(cx - sx * 0.012, cy - sy * 0.012, 0.018)
    # the apron: a small shell hanging under the bottom rail
    s = s + shell(0.0, -OM_H / 2 + 0.004, 0.034, ribs=7, lift=0.010, down=True)
    for side in (-1, 1):
        s = s + place(leaf(0.040, 0.016, lobes=3, lobe=0.4, depth=0.004, fold=0.5, curl=0.4, seed=40 + side),
                      side * 0.012, -OM_H / 2 - 0.004, 0.010, rz=math.pi + side * 1.0)
    return s


auto('Real__ornateMirrorFrame', mirror_frame, 400)
auto('Real__ornateMirrorCrest', mirror_crest, 7000)


# ===================================================== the bookshelf ===
# A hanging wall bookshelf of solid pine, stained: 600 mm wide, 160 mm deep,
# 300 mm between its top and its bottom board, the sides carried 45 mm below
# with a scrolled foot; 18 mm boards housed in the sides, a 5 mm hardboard
# back in their rebates. Its wall at z = 0, the bottom board's underside at
# y = 0 (books stand on y = 0.018).
BS_W, BS_D, BS_H, BS_T = 0.600, 0.160, 0.300, 0.018
BS_FOOT = 0.045
BS_BACK = 0.005


def bookshelf_sides():
    out = [(0.0, -BS_FOOT), (0.072, -BS_FOOT), (0.082, -0.038), (0.090, -0.027), (0.104, -0.020),
           (0.124, -0.016), (0.142, -0.012), (0.154, -0.006), (BS_D, 0.0), (BS_D, BS_H - 0.012),
           (BS_D - 0.004, BS_H - 0.003), (BS_D - 0.012, BS_H), (0.0, BS_H)]
    s = Shape()
    for sx in (-1, 1):
        side = prism(out, BS_T, bevel_r=0.0018, bevel_seg=2).rot(y=-math.pi / 2)
        s = s + side.move(sx * (BS_W / 2 - BS_T / 2), 0, 0)
    return s


def bookshelf_boards():
    inner = BS_W - 2 * BS_T + 0.012          # housed 6 mm into each side
    d = BS_D - BS_BACK - 0.002
    s = Shape()
    for y in (BS_T / 2, BS_H - BS_T / 2):
        s = s + rbox(inner, BS_T, d, r=0.0025, seg=2).move(0, y, BS_BACK + 0.001 + d / 2)
    return s


def bookshelf_back():
    return rbox(BS_W - 2 * BS_T + 0.016, BS_H, BS_BACK, r=0.0006, seg=1).move(0, BS_H / 2, BS_BACK / 2)


auto('Real__bookshelfSides', bookshelf_sides, 900)
auto('Real__bookshelfBoards', bookshelf_boards, 300)
auto('Real__bookshelfBack', bookshelf_back, 60)


# ======================================================== the locker ===
# A single-tier steel locker, 12 x 12 x 60 in (305 x 305 x 1524 mm) on its
# 6 in legs (the Penco / Lyon / Tennsco single-tier class, see the notes):
# a welded 16 ga frame round the door, 24 ga sides, top, bottom and back,
# a 16 ga door with six louvres top and bottom, three five-knuckle hinges
# on its left, a recessed lift latch with a padlock staple on its right.
# The back at z = 0, the floor at y = 0.
LK_W, LK_D, LK_BODY, LK_LEG = 0.305, 0.305, 1.524, 0.152
LK_SHEET = 0.0012
LK_STILE, LK_TOP, LK_KICK = 0.022, 0.032, 0.048
LK_GAP = 0.0025
LK_DOOR_W = LK_W - 2 * LK_STILE - 2 * LK_GAP
LK_DOOR_Y0 = LK_LEG + LK_KICK + LK_GAP
LK_DOOR_H = LK_BODY - LK_TOP - LK_KICK - 2 * LK_GAP
LK_DOOR_Z = LK_D - 0.0015          # the door's face, a hair behind the frame's
LK_LATCH = (LK_DOOR_W / 2 - 0.034, LK_DOOR_Y0 + 0.62 * LK_DOOR_H)


def locker_case():
    y0, y1 = LK_LEG, LK_LEG + LK_BODY
    h = LK_BODY
    t = LK_SHEET
    s = Shape()
    # the sides, back, top and bottom: sheet, their edges turned
    for sx in (-1, 1):
        s = s + rbox(t, h, LK_D - 0.002, r=0.0005, seg=1).move(sx * (LK_W / 2 - t / 2), y0 + h / 2, LK_D / 2 - 0.001)
    s = s + box(LK_W - 2 * t, h, t).move(0, y0 + h / 2, t / 2)
    for y in (y0 + t / 2, y1 - t / 2):
        s = s + box(LK_W - 2 * t, t, LK_D - 0.002).move(0, y, LK_D / 2)
    # the frame round the door: channel, its face 20 mm deep, rounded
    fz = LK_D - 0.010
    for sx in (-1, 1):
        s = s + rbox(LK_STILE, h, 0.020, r=0.0015, seg=2).move(sx * (LK_W / 2 - LK_STILE / 2), y0 + h / 2, fz)
    s = s + rbox(LK_W, LK_TOP, 0.020, r=0.0015, seg=2).move(0, y1 - LK_TOP / 2, fz)
    s = s + rbox(LK_W, LK_KICK, 0.020, r=0.0015, seg=2).move(0, y0 + LK_KICK / 2, fz)
    # the legs: formed angles at the corners, on round adjustable feet
    for sx in (-1, 1):
        for zz in (0.020, LK_D - 0.020):
            leg = rbox(0.030, LK_LEG - 0.010, 0.030, r=0.002, seg=1)
            s = s + leg.move(sx * (LK_W / 2 - 0.016), 0.010 + (LK_LEG - 0.010) / 2, zz)
            s = s + cyl(0.012, 0.013, 0.010, seg=14, bevel_r=0.001).move(sx * (LK_W / 2 - 0.016), 0.005, zz)
    # three five-knuckle hinges on the left stile
    hx = -LK_DOOR_W / 2 - LK_GAP - 0.001
    for yy in (0.16, 0.5, 0.84):
        y = LK_DOOR_Y0 + yy * LK_DOOR_H
        for k in range(5):
            kn = cyl(0.0042, 0.0042, 0.0095, seg=10, bevel_r=0.0006, bevel_seg=1)
            s = s + kn.move(hx, y - 0.025 + k * 0.0102 + 0.0048, LK_D + 0.0035)
        s = s + box(0.010, 0.050, 0.0015).move(hx - 0.007, y, LK_D + 0.0008)
    return s


def locker_door():
    w, h, t = LK_DOOR_W, LK_DOOR_H, 0.0016
    # the pan: its face and the returns turned back 18 mm, the edges rounded
    face = rbox(w, h, t, r=0.0007, seg=1).move(0, 0, -t / 2)
    rets = Shape()
    for sx in (-1, 1):
        rets = rets + box(t, h - 0.004, 0.018).move(sx * (w / 2 - t / 2), 0, -0.009 - t)
    for sy in (-1, 1):
        rets = rets + box(w - 0.004, t, 0.018).move(0, sy * (h / 2 - t / 2), -0.009 - t)
    # six louvres top and bottom: slots punched, each under a pressed hood
    slots, hoods = [], Shape()
    lw = 0.150
    for y0 in (h / 2 - 0.085, -h / 2 + 0.040):
        for k in range(6):
            y = y0 + k * 0.0115
            slots.append(box(lw, 0.0048, 0.02).move(0, y, -0.005))
            hood = cyl(0.0042, 0.0042, lw, seg=10, open_ended=True).rot(z=math.pi / 2)
            hood = cut(hood, box(lw + 0.02, 0.02, 0.02).move(0, 0, -0.010))       # the front half
            hood = cut(hood, box(lw + 0.02, 0.02, 0.02).move(0, -0.0100, 0))      # its top only
            hoods = hoods + hood.scale(1, 1.0, 0.8).move(0, y + 0.0018, 0.0)
            # its two ends pinched shut
            for sx in (-1, 1):
                hoods = hoods + box(0.0012, 0.0040, 0.0034).move(sx * lw / 2, y + 0.0035, 0.0017)
    face = cut(face, *slots)
    # the latch's pocket, pressed in
    lx, ly = LK_LATCH
    ly -= LK_DOOR_Y0 + h / 2
    face = cut(face, rbox(0.044, 0.160, 0.02, r=0.004, seg=2).move(lx, ly, 0))
    pocket = Shape()
    pocket = pocket + box(0.044, 0.160, 0.0012).move(lx, ly, -0.0135)
    for sx in (-1, 1):
        pocket = pocket + box(0.0012, 0.160, 0.013).move(lx + sx * 0.022, ly, -0.007)
    for sy in (-1, 1):
        pocket = pocket + box(0.044, 0.0012, 0.013).move(lx, ly + sy * 0.080, -0.007)
    s = face + rets + hoods + pocket
    # its frame: the door's middle at its own origin, moved to where it hangs
    return s.move(0, LK_DOOR_Y0 + h / 2, LK_DOOR_Z)


def locker_latch():
    lx, ly = LK_LATCH
    z = LK_DOOR_Z
    s = Shape()
    # the stainless cup's rim, pressed round the pocket
    rim = cut(rbox(0.054, 0.170, 0.0018, r=0.005, seg=2), rbox(0.042, 0.158, 0.01, r=0.004, seg=2))
    s = s + rim.move(lx, ly, z + 0.0009)
    # the lift handle in the pocket: a bent strip, its grip up
    s = s + rbox(0.030, 0.056, 0.004, r=0.0012, seg=1).move(lx, ly + 0.018, z - 0.006)
    s = s + rbox(0.030, 0.006, 0.010, r=0.0012, seg=1).move(lx, ly + 0.044, z - 0.002)
    # the padlock staple through the pocket's foot
    eye = cut(rbox(0.004, 0.026, 0.024, r=0.0012, seg=1), box(0.01, 0.010, 0.010).move(0, 0, 0.004))
    s = s + eye.move(lx, ly - 0.050, z - 0.002)
    return s


def locker_plate():
    p = rbox(0.050, 0.024, 0.0008, r=0.0006, seg=1).move(0, 0, 0.0004)
    for sx in (-1, 1):
        p = p + cyl(0.0022, 0.0022, 0.0012, seg=8, bevel_r=0.0005, bevel_seg=1).rot(x=math.pi / 2).move(sx * 0.020, 0, 0.0012)
    return p


auto('Real__lockerCase', locker_case, 2800)
auto('Real__lockerDoor', locker_door, 1000)
auto('Real__lockerLatch', locker_latch, 400)
auto('Real__lockerPlate', locker_plate, 200)


# ================================================== the cuckoo clock ===
# A carved Black Forest cuckoo clock of the traditional kind, one-day,
# weight driven (the leaf-and-bird "Jagdstueck" style): the case 170 x 200
# x 110 mm on its back board, a 100 mm dial, oak leaves carved round it
# and a bird with its wings spread on its top, 220 mm across the leaves;
# a carved leaf on the pendulum, two cast pine-cone weights on brass chains.
# Its wall at z = 0, the dial's middle at y = 0.
CK_CW, CK_CH, CK_CD = 0.170, 0.200, 0.110
CK_CY = -0.020                     # the case's middle (y)
CK_BACK = 0.008
CK_FRONT = CK_BACK + CK_CD         # the case's front (z)
CK_BOTTOM = CK_CY - CK_CH / 2      # its underside (y)
CK_DIAL_R = 0.050
CK_CHAIN_X = (-0.036, 0.036)       # where the weights' chains come out
CK_CHAIN_Z = 0.074
CK_CHAIN_LEN = 0.170
CK_PEND = (0.0, CK_BOTTOM, 0.030)  # the pendulum's hook


def cuckoo_case():
    s = rbox(0.200, 0.250, CK_BACK, r=0.0015, seg=1).move(0, CK_CY + 0.005, CK_BACK / 2)
    s = s + rbox(CK_CW, CK_CH, CK_CD, r=0.003, seg=2).move(0, CK_CY, CK_BACK + CK_CD / 2)
    # a cornice round its top, a plinth moulding round its foot
    for (y, hh, out) in ((CK_CY + CK_CH / 2 - 0.006, 0.012, 0.008), (CK_CY + CK_CH / 2 + 0.003, 0.006, 0.004),
                         (CK_BOTTOM + 0.005, 0.010, 0.006)):
        s = s + rbox(CK_CW + 2 * out, hh, CK_CD + out, r=0.002, seg=2).move(0, y, CK_BACK + (CK_CD + out) / 2)
    # the cuckoo's door above the dial: its frame and its two shut leaves
    dy = 0.058
    fr = cut(rbox(0.046, 0.040, 0.004, r=0.0012, seg=1), box(0.036, 0.030, 0.01))
    s = s + fr.move(0, dy, CK_FRONT + 0.002)
    for sx in (-1, 1):
        lf = rbox(0.0175, 0.029, 0.0025, r=0.0008, seg=1)
        s = s + lf.move(sx * 0.0092, dy, CK_FRONT + 0.0013)
        s = s + sphere(0.0011, seg=6).move(sx * 0.0025, dy, CK_FRONT + 0.003)
    # the dial's seat, a turned ring
    s = s + ring_lathe([(CK_DIAL_R + 0.001, 0.0), (CK_DIAL_R + 0.006, 0.0), (CK_DIAL_R + 0.006, 0.002),
                        (CK_DIAL_R + 0.003, 0.0045), (CK_DIAL_R + 0.001, 0.004)], seg=40).rot(x=math.pi / 2).move(0, 0, CK_FRONT)
    # the chains' holes and the pendulum's slot are under it: unseen
    return s


def cuckoo_dial():
    prof = [(0.0, 0.0), (CK_DIAL_R, 0.0), (CK_DIAL_R, 0.0028), (CK_DIAL_R - 0.002, 0.0040), (0.0, 0.0040)]
    return lathe(prof, seg=40).rot(x=math.pi / 2).rot(z=0).move(0, 0, CK_FRONT)


def _bird():
    b = ellipsoid(0.026, 0.017, 0.016, seg=16).rot(z=0.30).move(0.0, 0.132, 0.098)
    b = b + sphere(0.012, seg=14).move(0.023, 0.148, 0.101)
    b = b + cone(0.0040, 0.015, seg=8).rot(z=-math.pi / 2 + 0.15).move(0.040, 0.147, 0.101)
    for side in (-1, 1):
        # the wings raised, their primaries spread
        w = leaf(0.085, 0.040, lobes=6, lobe=0.22, depth=0.005, fold=0.12, curl=-0.20, tip=1.4, n=20, seed=60 + side)
        b = b + place(w, side * 0.006, 0.138, 0.092 - 0.004 * side, rz=-side * 0.62, ry=side * 0.30)
    t = leaf(0.050, 0.026, lobes=4, lobe=0.25, depth=0.005, fold=0.2, curl=0.15, n=14, seed=63)
    b = b + place(t, -0.020, 0.126, 0.096, rz=2.1)
    return b


def cuckoo_carving():
    s = Shape()
    z = CK_FRONT + 0.004
    # oak leaves round the dial, two rings overlapping, their stems under its
    # turned ring, reaching past the case's sides
    for k in range(12):
        a = -math.pi / 2 + (k + 0.5) * TAU / 12
        if math.sin(a) > 0.8:                   # the cuckoo's door stays clear
            continue
        rr = CK_DIAL_R + 0.004 + 0.002 * (k % 2)
        ln = 0.044 + 0.006 * ((k * 7) % 3) / 2
        s = s + place(leaf(ln, 0.036, lobes=3, lobe=0.38, depth=0.006, fold=0.45, curl=0.30, twist=0.25 * (1 if k % 2 else -1), n=18, seed=70 + k),
                      rr * math.cos(a), rr * math.sin(a), z + 0.003 * (k % 2), rz=a - math.pi / 2)
    for k in range(8):
        a = -math.pi / 2 + k * TAU / 8 + 0.2
        if math.sin(a) > 0.7:
            continue
        rr = CK_DIAL_R + 0.012
        s = s + place(leaf(0.036, 0.028, lobes=3, lobe=0.38, depth=0.005, fold=0.4, curl=0.4, n=16, seed=200 + k),
                      rr * math.cos(a), rr * math.sin(a), z + 0.008, rz=a - math.pi / 2 + 0.5)
    # acorns among them
    for (x, y) in ((-0.060, -0.072), (0.058, -0.074), (0.0, -0.088)):
        s = s + ellipsoid(0.0055, 0.0075, 0.0055, seg=10).move(x, y, z + 0.010)
        s = s + sphere(0.0062, seg=10).scale(1, 0.7, 1).move(x, y + 0.0055, z + 0.010)
    # the leaves on its top the bird stands on
    for k, (x, a, ln) in enumerate(((-0.06, 0.95, 0.060), (-0.025, 0.35, 0.055), (0.025, -0.35, 0.055), (0.06, -0.95, 0.060))):
        s = s + place(leaf(ln, 0.030, lobes=3, lobe=0.42, depth=0.006, fold=0.45, curl=-0.2, seed=90 + k),
                      x, CK_CY + CK_CH / 2 - 0.004, CK_FRONT - 0.012 + 0.006 * (k % 2), rz=a)
    s = s + _bird()
    # three leaves hanging under the case
    for k, (x, a) in enumerate(((-0.030, math.pi - 0.45), (0.0, math.pi), (0.030, math.pi + 0.45))):
        s = s + place(leaf(0.052, 0.030, lobes=3, lobe=0.42, depth=0.006, fold=0.45, curl=0.3, seed=100 + k),
                      x, CK_BOTTOM + 0.008, CK_FRONT - 0.020, rz=a)
    return s


def _hand(length, width, tail, loop_at):
    """A cuckoo clock's hand: up +y from its boss at the origin, flat on
    z = 0 (1.2 mm thick), a pierced loop along it, a spade at its tip."""
    w = width
    pts = [(-0.0016, -tail), (0.0016, -tail), (w * 0.25, -0.004), (w * 0.25, loop_at - 0.006)]
    pts += [(w * 0.25, length * 0.80), (w * 0.95, length * 0.86), (w * 0.15, length), (0.0, length * 1.02)]
    pts = pts + [(-x, y) for (x, y) in reversed(pts)]
    pts = [p for i, p in enumerate(pts) if i == 0 or (abs(p[0] - pts[i - 1][0]) + abs(p[1] - pts[i - 1][1])) > 1e-6]
    s = prism(pts, 0.0012).move(0, 0, 0.0006)
    s = s + torus(width * 0.62, 0.0009, seg=16, tseg=5).move(0, loop_at, 0.0006)
    s = s + cyl(0.0048, 0.0048, 0.0018, seg=14, bevel_r=0.0005, bevel_seg=1).rot(x=math.pi / 2).move(0, 0, 0.0009)
    return s


def cuckoo_hour():
    return _hand(0.031, 0.0085, 0.006, 0.013)


def cuckoo_minute():
    return _hand(0.044, 0.0075, 0.008, 0.019)


def cuckoo_pend_rod():
    hook = tube([(0, 0.004, 0), (0.002, 0.0055, 0), (0.0035, 0.0035, 0), (0.002, 0.0, 0), (0, 0.0, 0)], 0.0007, seg=5)
    return hook + cyl(0.0008, 0.0008, 0.150, seg=6, bevel_r=0).move(0, -0.075, 0)


def cuckoo_pend_bob():
    lf = leaf(0.058, 0.040, lobes=3, lobe=0.40, depth=0.006, fold=0.35, curl=0.1, n=16, seed=120)
    return lf.rot(z=math.pi).move(0, -0.128, 0.002)


def _link(vertical):
    l = torus(0.0028, 0.0006, seg=8, tseg=3).scale(0.75, 1.35, 1.0)
    return l.rot(y=math.pi / 2) if vertical else l


def cuckoo_chains():
    s = Shape()
    pitch = 0.0062
    for k, x in enumerate(CK_CHAIN_X):
        # the weight's strand, and beside it the free end with its ring
        for (dx, ln, ring) in ((0.0, CK_CHAIN_LEN, False), (0.012 if x > 0 else -0.012, 0.055, True)):
            n = int(ln / pitch)
            for i in range(n):
                s = s + _link(i % 2).move(x + dx, CK_BOTTOM - 0.003 - i * pitch, CK_CHAIN_Z)
            if ring:
                s = s + torus(0.0075, 0.0011, seg=14, tseg=4).move(x + dx, CK_BOTTOM - 0.003 - n * pitch - 0.007, CK_CHAIN_Z)
    return s


def _pinecone():
    prof = [(0.0, 0.0)]
    for i in range(1, 16):
        t = i / 15
        r = 0.0145 * math.sin(math.pi * min(1.0, 0.18 + 0.86 * t) ** 0.85) ** 0.8
        prof.append((max(0.002, r), -0.092 * (1 - t)))
    prof[0] = (0.0, -0.093)
    prof.append((0.006, 0.0005))
    prof.append((0.0, 0.0012))
    cone_ = lathe(prof, seg=20)
    # its scales: lifted in a spiral, each one's lip
    for p in cone_.v:
        r = math.hypot(p.x, p.z)
        if r < 0.003:
            continue
        a = math.atan2(p.x, p.z)
        u = a * 8 / TAU + p.y / 0.011
        w = a * 5 / TAU - p.y / 0.011
        bump = (0.5 + 0.5 * math.cos(TAU * u)) * (0.5 + 0.5 * math.cos(TAU * w))
        k = 1 + 0.16 * bump
        p.x *= k
        p.z *= k
    hook = tube([(0, 0.000, 0), (0, 0.006, 0), (0.0035, 0.010, 0), (0, 0.0135, 0), (-0.0032, 0.0105, 0)], 0.0011, seg=6)
    return cone_ + hook


def cuckoo_weights():
    s = Shape()
    for x in CK_CHAIN_X:
        s = s + _pinecone().move(x, CK_BOTTOM - 0.003 - CK_CHAIN_LEN - 0.014, CK_CHAIN_Z)
    return s


auto('Real__cuckooCase', cuckoo_case, 1400)
auto('Real__cuckooDial', cuckoo_dial, 260)
auto('Real__cuckooCarving', cuckoo_carving, 9500)
auto('Real__cuckooHour', cuckoo_hour, 360)
auto('Real__cuckooMinute', cuckoo_minute, 360)
auto('Real__cuckooPendRod', cuckoo_pend_rod, 120)
auto('Real__cuckooPendBob', cuckoo_pend_bob, 300)
auto('Real__cuckooChains', cuckoo_chains, 3700)
auto('Real__cuckooWeights', cuckoo_weights, 1800)


# ===================================================== the fireplace ===
# A Victorian bedroom fireplace: a painted pine mantelpiece, 960 mm wide and
# 1100 mm to its shelf, its pilasters on plinths, corbels under the shelf, a
# frieze with a centre tablet; in its opening a cast-iron arched register
# insert (660 x 850 mm, the arch 420 mm wide) with its canopy, fire basket
# and fret; a firebrick back sooted black; a slate hearth. Ash and charred
# ends in the basket; when a fire was lit (the game's draw), embers under
# them and tongues of flame. Its wall at z = 0, the floor at y = 0.
FP_W, FP_SHELF_Y, FP_SHELF_T, FP_SHELF_D = 0.960, 1.070, 0.030, 0.210
FP_PIL_W, FP_PIL_D = 0.140, 0.140
FP_OPEN_W = FP_W - 2 * FP_PIL_W            # 0.68
FP_FRIEZE = (0.880, 1.040)                 # its bottom and top (y)
FP_INS_Z = 0.105                           # the insert plate's back (z)
FP_INS_T = 0.012
FP_ARCH_W, FP_ARCH_SPRING, FP_HEARTH_Y = 0.420, 0.520, 0.030
FP_BASKET = (0.360, 0.130, 0.250)          # its bars' width, bottom and top (y)


def fireplace_mantel():
    s = Shape()
    for sx in (-1, 1):
        x = sx * (FP_W / 2 - FP_PIL_W / 2)
        # the plinth, the pilaster with a sunk panel, the corbel
        s = s + rbox(FP_PIL_W + 0.012, 0.150, FP_PIL_D + 0.012, r=0.004, seg=2).move(x, 0.075, (FP_PIL_D + 0.012) / 2)
        s = s + rbox(FP_PIL_W + 0.016, 0.012, FP_PIL_D + 0.016, r=0.004, seg=2).move(x, 0.150, (FP_PIL_D + 0.016) / 2)
        pil = rbox(FP_PIL_W, FP_FRIEZE[0] - 0.156, FP_PIL_D, r=0.003, seg=2).move(x, (FP_FRIEZE[0] + 0.156) / 2, FP_PIL_D / 2)
        pil = cut(pil, rbox(FP_PIL_W - 0.050, FP_FRIEZE[0] - 0.300, 0.02, r=0.004, seg=2).move(x, (FP_FRIEZE[0] + 0.156) / 2 + 0.010, FP_PIL_D + 0.003))
        s = s + pil
        # the corbel: a block and a scroll under it
        s = s + rbox(FP_PIL_W + 0.010, FP_FRIEZE[1] - FP_FRIEZE[0], FP_PIL_D + 0.020, r=0.004, seg=2).move(
            x, (FP_FRIEZE[0] + FP_FRIEZE[1]) / 2, (FP_PIL_D + 0.020) / 2)
        sc = prism([(0, 0), (0.05, 0), (0.05, 0.012), (0.035, 0.030), (0.020, 0.060), (0.0, 0.075)], FP_PIL_W - 0.040, bevel_r=0.003)
        sc = sc.rot(y=-math.pi / 2).move(x, FP_FRIEZE[0] - 0.075, FP_PIL_D)
        s = s + sc
    # the frieze, its tablet
    s = s + rbox(FP_OPEN_W + 0.004, FP_FRIEZE[1] - FP_FRIEZE[0], 0.110, r=0.003, seg=2).move(0, (FP_FRIEZE[0] + FP_FRIEZE[1]) / 2, 0.055)
    s = s + rbox(0.200, 0.110, 0.012, r=0.003, seg=2).move(0, (FP_FRIEZE[0] + FP_FRIEZE[1]) / 2, 0.110 + 0.006)
    # a bead under the frieze along the opening's head
    s = s + cyl(0.006, 0.006, FP_OPEN_W, seg=10, bevel_r=0).rot(z=math.pi / 2).move(0, FP_FRIEZE[0] + 0.004, 0.110)
    # the bed moulding (a cove) and the shelf with its bullnose edge
    bed = prism([(0, 0), (0.030, 0), (0.030, 0.006), (0.018, 0.010), (0.010, 0.018), (0.006, 0.030), (0, 0.030)], FP_W - 0.010, bevel_r=0.0)
    s = s + bed.rot(y=-math.pi / 2).move(0, FP_FRIEZE[1], FP_PIL_D + 0.020)
    s = s + rbox(FP_W, FP_SHELF_T, FP_SHELF_D, r=0.010, seg=3).move(0, FP_SHELF_Y + FP_SHELF_T / 2, FP_SHELF_D / 2)
    return s


def _arch_outline(w, spring, y0, n=14, inset=0.0):
    r = w / 2 - inset
    pts = [(-r, y0), (r, y0)]
    for i in range(n + 1):
        a = math.pi * i / n
        pts.append((r * math.cos(a), spring + r * math.sin(a)))
    return pts


def fireplace_insert():
    ih = FP_FRIEZE[0] - 0.004
    plate = rbox(FP_OPEN_W - 0.004, ih, FP_INS_T, r=0.002, seg=1).move(0, ih / 2, FP_INS_Z + FP_INS_T / 2)
    hole = prism(_arch_outline(FP_ARCH_W, FP_ARCH_SPRING, -0.05), 0.08).move(0, 0, FP_INS_Z)
    plate = cut(plate, hole)
    s = plate
    zf = FP_INS_Z + FP_INS_T
    # the beads cast round the arch and round the plate
    path = [(x, y, zf + 0.002) for (x, y) in _arch_outline(FP_ARCH_W + 0.026, FP_ARCH_SPRING, FP_HEARTH_Y, n=16)[2:]]
    path = [(-(FP_ARCH_W + 0.026) / 2, FP_HEARTH_Y + 0.07, zf + 0.002)] + list(reversed(path)) + [((FP_ARCH_W + 0.026) / 2, FP_HEARTH_Y + 0.07, zf + 0.002)]
    s = s + tube(path, 0.0075, seg=8)
    path2 = [(x, y, zf + 0.001) for (x, y) in _arch_outline(FP_ARCH_W + 0.070, FP_ARCH_SPRING, FP_HEARTH_Y, n=16)[2:]]
    path2 = [(-(FP_ARCH_W + 0.070) / 2, FP_HEARTH_Y + 0.07, zf + 0.001)] + list(reversed(path2)) + [((FP_ARCH_W + 0.070) / 2, FP_HEARTH_Y + 0.07, zf + 0.001)]
    s = s + tube(path2, 0.0040, seg=6)
    # the cast spandrels: a leaf spray in each upper corner, a shell at the top
    top = FP_ARCH_SPRING + FP_ARCH_W / 2
    for sx in (-1, 1):
        for k, (a, ln) in enumerate(((0.35, 0.090), (1.15, 0.080), (-0.45, 0.070))):
            s = s + place(leaf(ln, 0.034, lobes=3, lobe=0.45, depth=0.005, fold=0.3, curl=0.1, seed=140 + k + 4 * sx),
                          sx * 0.255, top + 0.020, zf, rz=sx * (a + 0.4) + (math.pi / 2 if sx > 0 else -math.pi / 2) * 0.0)
    s = s + shell(0.0, top + 0.040, 0.045, ribs=9, lift=0.010, z0=zf - 0.001)
    # panels down the sides: a sunk-looking raised border, and a cast rosette
    for sx in (-1, 1):
        x = sx * (FP_OPEN_W / 2 - 0.058)
        fr = cut(rbox(0.070, 0.380, 0.006, r=0.002, seg=1), box(0.054, 0.364, 0.02))
        s = s + fr.move(x, 0.300, zf + 0.003)
        s = s + shell(x, 0.300, 0.020, ribs=8, a0=0.0, a1=TAU - 0.0001, lift=0.006, z0=zf)
    # the canopy: a hood curving out under the arch's crown
    can = []
    for i in range(9):
        t = i / 8
        a = math.pi * (0.12 + 0.76 * t)
        r = FP_ARCH_W / 2 - 0.004
        x = r * math.cos(a)
        can.append([(x, FP_ARCH_SPRING + r * math.sin(a) - d * 0.55, zf - d) for d in (0.0, 0.035, 0.070)])
    hood = Shape()
    for i in range(8):
        for j in range(2):
            a, b, c, d = can[i][j], can[i + 1][j], can[i + 1][j + 1], can[i][j + 1]
            q = Shape([a, b, c, d], [(0, 1, 2, 3)])
            hood = hood + q
    hood = propkit._fix_winding(hood)
    s = s + hood
    return s


def fireplace_grate():
    bw, b0, b1 = FP_BASKET
    s = Shape()
    zf = FP_INS_Z + FP_INS_T
    # the basket: front bars, bowed out a little, between two cheeks
    for k in range(5):
        y = b0 + 0.012 + k * (b1 - b0 - 0.02) / 4
        pts = [(-bw / 2, y, zf + 0.020), (-bw / 4, y, zf + 0.058), (bw / 4, y, zf + 0.058), (bw / 2, y, zf + 0.020)]
        s = s + tube(pts, 0.0055, seg=7, smooth=2)
    for sx in (-1, 1):
        s = s + rbox(0.012, b1 - b0 + 0.03, 0.090, r=0.003, seg=1).move(sx * bw / 2, (b0 + b1) / 2, zf - 0.010)
        s = s + sphere(0.012, seg=10).move(sx * bw / 2, b1 + 0.026, zf + 0.012)
    # its bottom grid, to the firebrick
    for k in range(7):
        x = -bw / 2 + 0.03 + k * (bw - 0.06) / 6
        s = s + box(0.010, 0.010, 0.13).move(x, b0, zf - 0.035)
    # the fret over the ash pan, pierced
    fret = rbox(FP_ARCH_W + 0.020, b0 - FP_HEARTH_Y, 0.006, r=0.002, seg=1).move(0, (b0 + FP_HEARTH_Y) / 2, zf + 0.030)
    holes = [rbox(0.016, 0.050, 0.04, r=0.006, seg=2).move(-0.18 + k * 0.040, (b0 + FP_HEARTH_Y) / 2, zf + 0.030) for k in range(10)]
    s = s + cut(fret, *holes)
    return s


def fireplace_firebox():
    s = Shape()
    w = FP_ARCH_W / 2 + 0.010
    top = FP_ARCH_SPRING + FP_ARCH_W / 2
    # the fireback, against the wall, and its splayed cheeks
    s = s + box(0.30, top + 0.02, 0.012).move(0, (top + 0.02) / 2, 0.006)
    for sx in (-1, 1):
        a = Vector((sx * w, FP_HEARTH_Y, FP_INS_Z + 0.002))
        b = Vector((sx * 0.150, FP_HEARTH_Y, 0.012))
        mid = (a + b) / 2
        d = b - a
        ang = math.atan2(d.x, d.z)
        ch = box(0.012, top, d.length).rot(y=ang).move(mid.x, top / 2, mid.z)
        s = s + ch
    # the throat's lintel over it, and the back hearth's firebrick
    s = s + box(2 * w, 0.02, FP_INS_Z).move(0, top + 0.01, FP_INS_Z / 2)
    s = s + box(2 * w, 0.008, FP_INS_Z).move(0, FP_HEARTH_Y + 0.004, FP_INS_Z / 2)
    return s


def fireplace_hearth():
    front = rbox(FP_W, FP_HEARTH_Y, 0.440 - 0.150, r=0.003, seg=2).move(0, FP_HEARTH_Y / 2, 0.150 + (0.440 - 0.150) / 2)
    back = rbox(FP_OPEN_W - 0.010, FP_HEARTH_Y, 0.152, r=0.001, seg=1).move(0, FP_HEARTH_Y / 2, 0.076)
    return front + back


def fireplace_ash():
    bw, b0, _ = FP_BASKET
    s = ellipsoid(bw / 2 - 0.02, 0.030, 0.060, seg=16).move(0, b0 + 0.006, FP_INS_Z - 0.020)
    s = cut(s, box(1, 0.1, 1).move(0, b0 - 0.05 + 0.004, 0))
    warp_noise(s, 0.006, 40.0, 7)
    # what fell through: a drift on the back hearth and over the fret's foot
    d = ellipsoid(0.17, 0.010, 0.07, seg=14).move(0, FP_HEARTH_Y, 0.080)
    d = cut(d, box(1, 0.1, 1).move(0, FP_HEARTH_Y - 0.05, 0))
    warp_noise(d, 0.003, 50.0, 9)
    e = ellipsoid(0.12, 0.006, 0.05, seg=12).move(0.03, FP_HEARTH_Y, FP_INS_Z + 0.075)
    e = cut(e, box(1, 0.1, 1).move(0, FP_HEARTH_Y - 0.05, 0))
    warp_noise(e, 0.002, 60.0, 10)
    return s + d + e


def _charred(length, r, seed):
    """A log burnt to a charred end: its middle eaten away, its bark crazed."""
    prof = []
    for i in range(11):
        t = i / 10
        k = 1 - 0.55 * math.sin(math.pi * t) ** 2 * 0.0 - 0.45 * (1 - abs(2 * t - 1)) ** 1.5
        prof.append((r * max(0.35, k), -length / 2 + length * t))
    prof = [(0.0, -length / 2 - 0.004)] + prof + [(0.0, length / 2 + 0.004)]
    s = lathe(prof, seg=12)
    warp_noise(s, r * 0.28, 60.0, seed)
    # alligatored: the char cracked in blocks
    for p in s.v:
        a = math.atan2(p.x, p.z)
        g = abs(math.sin(a * 6 + seed)) * abs(math.sin(p.y * 90 + seed))
        k = 1 - 0.06 * (g > 0.82)
        p.x *= k
        p.z *= k
    return s.rot(z=math.pi / 2)


# ---- the fire, when one was lit (the old draw): a low wood fire in the basket
FP_LOG_Z = 0.080                   # where the logs lie (z), as the game lays them


def _faces_down(sh, below=-0.15):
    """Keep the faces of `sh` whose normal points down (y < below)."""
    keep = []
    for f in sh.f:
        a, b, c = sh.v[f[0]], sh.v[f[1]], sh.v[f[2]]
        n = (b - a).cross(c - a)
        if n.length > 1e-12 and n.normalized().y < below:
            keep.append(f)
    used = sorted({i for f in keep for i in f})
    remap = {o: k for k, o in enumerate(used)}
    return Shape([sh.v[i] for i in used], [tuple(remap[i] for i in f) for f in keep])


def _charred_glow(length, r, seed):
    """A charred end's underside burning: its shell a hair outside the char,
    the faces that look down only (the game draws its cracks glowing)."""
    s = _charred(length, r, seed)
    for p in s.v:
        p.y *= 1.04
        p.z *= 1.04
    return _faces_down(s)


def fireplace_embers():
    """The bed of embers on the ash under and among the logs: coals of
    every size, the mound glowing under them."""
    bw, b0, _ = FP_BASKET
    rng = random.Random(77)
    s = ellipsoid(bw / 2 - 0.035, 0.012, 0.050, seg=14).move(0, b0 + 0.016, FP_LOG_Z - 0.004)
    s = cut(s, box(1, 0.1, 1).move(0, b0 + 0.016 - 0.05, 0))
    warp_noise(s, 0.003, 50.0, 12)
    for k in range(34):
        r = rng.uniform(0.006, 0.014)
        x = rng.uniform(-bw / 2 + 0.03, bw / 2 - 0.03)
        z = FP_LOG_Z + rng.uniform(-0.050, 0.045)
        c = sphere(r, seg=8).scale(1.0, rng.uniform(0.55, 0.8), rng.uniform(0.8, 1.1)).rot(y=rng.uniform(0, TAU))
        warp_noise(c, r * 0.3, 140.0, 20 + k)
        s = s + c.move(x, b0 + 0.020 + r * 0.4, z)
    return s


def _tongue(h, r, lean, seed, n=11, seg=12):
    """A flame's tongue: a teardrop drawn up from its root at the origin,
    widest a quarter of the way up, flattened across, leaning and wavering
    as it rises."""
    prof = [(0.0, 0.0)]
    for i in range(1, n):
        t = i / n
        prof.append((r * math.sin(math.pi * min(1.0, t ** 0.55)) ** 1.2 * (1 - 0.6 * t), h * t))
    prof.append((0.0, h))
    s = lathe(prof, seg=seg)
    for p in s.v:
        t = max(0.0, p.y / h)
        p.z *= 0.55
        p.x += lean * h * t * t + 0.12 * r * math.sin(t * 5.0 + seed) * t
    return s


def fireplace_flame(kind):
    """One tongue of a fire's flame, its root at the origin."""
    def build():
        h, r, lean = {'A': (0.200, 0.026, 0.10), 'B': (0.150, 0.021, -0.16), 'C': (0.105, 0.017, 0.24)}[kind]
        return _tongue(h, r, lean, ord(kind))
    return build


auto('Real__fireplaceMantel', fireplace_mantel, 2200)
auto('Real__fireplaceInsert', fireplace_insert, 5200)
auto('Real__fireplaceGrate', fireplace_grate, 2200)
auto('Real__fireplaceFirebox', fireplace_firebox, 120)
auto('Real__fireplaceHearth', fireplace_hearth, 200)
auto('Real__fireplaceAsh', fireplace_ash, 1200)
auto('Real__fireplaceLogA', lambda: _charred(0.240, 0.040, 1), 400)
auto('Real__fireplaceLogB', lambda: _charred(0.210, 0.034, 2), 400)
auto('Real__fireplaceLogC', lambda: _charred(0.180, 0.030, 3), 400)
auto('Real__fireplaceLogD', lambda: _charred(0.150, 0.026, 4), 400)
auto('Real__fireplaceLogAGlow', lambda: _charred_glow(0.240, 0.040, 1), 200)
auto('Real__fireplaceLogBGlow', lambda: _charred_glow(0.210, 0.034, 2), 200)
auto('Real__fireplaceLogCGlow', lambda: _charred_glow(0.180, 0.030, 3), 200)
auto('Real__fireplaceLogDGlow', lambda: _charred_glow(0.150, 0.026, 4), 200)
auto('Real__fireplaceEmbers', fireplace_embers, 4600)
auto('Real__fireplaceFlameA', fireplace_flame('A'), 300, smooth=80.0)
auto('Real__fireplaceFlameB', fireplace_flame('B'), 300, smooth=80.0)
auto('Real__fireplaceFlameC', fireplace_flame('C'), 300, smooth=80.0)
