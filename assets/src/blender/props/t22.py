"""t22: small everyday things remodelled at their real sizes (docs/PROPS.md,
P3): cigarette butts stubbed out on the floor and their ash, cotton swabs
spilled from their box, a scatter of paper clips, candy corn, a lightweight
chandelle feather boa dropped on the floor, a child's pair of wooden
maracas, a twisty puzzle cube. (The wet floor sign is r01's, placed as the
core props place it.) Sources and sizes: t22.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame, already lying as it lies (x along it, y up from
the floor it rests on, y = 0 its lowest point): the game turns it only
about y. Where a part is painted along its length, the game lays its
texture by x (each part's notes say from where), so the parts that share a
painted strip start at the same x.

cigButtA..D     king-size filter cigarettes smoked and dropped, one mesh a
                butt (the game paints the tipping, the paper and the char
                along x from the mouth end at x = 0): A stubbed out, its
                end crushed flat and kinked; B smoked to the tipping, the
                filter trodden oval; C dropped half smoked, gone out, its
                end burnt ragged; D trodden flat
cigButtAsh      the ash knocked off beside one: flakes, and a column
                fallen whole and broken
wetSignHingeHalf  r01's sign's hinge, split between its two panels: one
                panel's half (the pin toward -x, two knuckles)
swabA..D        3 in paper-stick cotton swabs, one mesh a swab (stick and
                cotton; the game paints them along x from the -x tip): A
                new; B its stick kinked; C used, one tuft pressed and
                twisted; D one tuft pulled off, the stick's end bare
paperclipGem    a No. 1 Gem clip (35 mm), plated wire
paperclipOpen   the same, pulled open into an S by a fidgeting hand
paperclipJumbo  a jumbo Gem (48 mm), its outer loop bent up off the floor
candyCornA..C   candy corn lying on a broad face, the base at x = 0, the
                tip toward +x (the game paints its bands along x): A whole;
                B its tip snapped off; C gone soft in the heat and slumped
maracaHead      a child's turned beech maraca's painted head (its collar
                where the handle goes in)
maracaHandle    its turned handle and knob, bare beech
                (both lying as a maraca lies: on its head and its knob,
                the axis rising toward the head along +x; MARACA_TILT)
twistyCubies    a 57 mm 3x3x3 twisty puzzle's 26 cubies, black plastic,
                their edges rounded, the gaps between the layers
twistyStickers  its 53 stickers (one gone), top faces only, two peeling at
                a corner (the game paints the colours by face)
boaCoreA..C, boaFeathersA..C   three lengths of a short chandelle boa
                (75 mm of it each, along x): the dense fluff round its
                cord, and its feathers, each a curled vane of barbs; the
                game lays twelve along the curve the boa fell in
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


def on_floor(s):
    """Lift or lower a shape so its lowest point is on y = 0."""
    lo, _ = s.bounds()
    return s.move(0, -lo.y, 0)


def ring_x(x, r_y, r_z, seg, cy=0.0, cz=0.0, a0=0.0):
    """A ring of `seg` points round the x axis at x (radii r_y up, r_z
    across), centred at (cy, cz); angle from +y toward +z."""
    return [Vector((x, cy + r_y * math.cos(a0 + TAU * j / seg), cz + r_z * math.sin(a0 + TAU * j / seg))) for j in range(seg)]


# ===================================================== cigarette butts ===
# A king-size filter cigarette: 84 mm, 7.8 mm across, a 21 mm filter of
# acetate tow under cork-printed tipping paper that laps 4 mm onto the
# tobacco rod. These numbers are the game's too (t22.rs: CIG_*).
CIG_R = 0.0039
CIG_FILTER = 0.021
CIG_TIPPING = 0.025
CIG_SEG = 10


def butt(paper, crush=0.0, kink=0.0, flat=0.0, oval=0.0, ragged=0.4, seed=1):
    """A butt along +x from its mouth end at x = 0: the filter, `paper`
    metres of the rod left, a charred end. `crush` flattens the paper's
    last part (stubbed out), `kink` bends it up by that angle, `flat`
    squashes the whole butt (trodden), `oval` the filter only, `ragged`
    how unevenly the end burnt."""
    rnd_ = random.Random(seed)
    end = CIG_FILTER + paper
    xs = [0.0, 0.0006, 0.004, 0.010, 0.016, CIG_FILTER - 0.0006, CIG_FILTER, CIG_TIPPING]
    k = max(1, int(round((end - CIG_TIPPING) / 0.0055)))
    xs += [CIG_TIPPING + (end - CIG_TIPPING) * (i + 1) / k for i in range(k)] if end > CIG_TIPPING + 0.001 else []
    xs = sorted(set(round(x, 6) for x in xs if x <= end + 1e-6))
    rings = []
    for x in xs:
        r = CIG_R
        if x < 0.0005:
            r = CIG_R - 0.0005                      # the mouth end's rounded edge
        elif x > CIG_FILTER + 1e-6:
            r = CIG_R * 0.985                      # the rod a shade under the tipping
        if CIG_FILTER - 0.0001 < x < CIG_TIPPING + 0.0001 and x > CIG_FILTER:
            r = CIG_R * 1.004
        rings.append((x, r))
    # the burnt end: a ragged ring of paper, the char inside it shrinking
    rings.append((end + 0.0008, CIG_R * 0.93))
    rings.append((end + 0.0022, CIG_R * 0.72))
    rings.append((end + 0.0032, CIG_R * 0.38))
    out = []
    for i, (x, r) in enumerate(rings):
        t_crush = smoothstep(end - 0.007, end, x) * crush if crush else 0.0
        ry = r * (1.0 - 0.55 * t_crush) * (1.0 - flat)
        rz = r * (1.0 + 0.35 * t_crush) * (1.0 + 0.45 * flat)
        if x <= CIG_TIPPING and oval:
            ry *= 1.0 - oval
            rz *= 1.0 + 0.5 * oval
        ring = ring_x(x, ry, rz, CIG_SEG)
        if x > end:                                # burnt raggedly
            for p in ring:
                p.x += rnd_.uniform(-1, 1) * 0.0011 * ragged
        if i == len(rings) - 1:
            for p in ring:
                p.x += rnd_.uniform(0, 1) * 0.0006
        out.append(ring)
    s = loft(out, cap0=True, cap1=True)
    # a little unevenness: paper is not a lathe
    s = noisy(s, 0.00012, scale=700, seed=seed)
    if kink:
        # bent up from 6 mm before the end, as a stub pressed down and twisted
        x0 = end - 0.006

        def bend_up(p):
            if p.x <= x0:
                return p
            d = p.x - x0
            a = kink * min(1.0, d / 0.004)
            return Vector((x0 + d * math.cos(a) - p.y * math.sin(a) * 0.3, p.y + d * math.sin(a), p.z))
        s.warp(bend_up)
    return on_floor(s)


def cig_ash():
    """Ash knocked off beside a butt: a column fallen whole and broken in
    two, its rings cracked; flakes round it."""
    s = Shape()
    for (x0, x1, a) in ((0.0, 0.0058, 0.0), (0.0068, 0.0102, 0.35)):
        prof = [(0.0, 0.0), (CIG_R * 0.88, 0.0004), (CIG_R * 0.93, 0.002), (CIG_R * 0.84, 0.0026),
                (CIG_R * 0.92, (x1 - x0) * 0.75), (CIG_R * 0.80, x1 - x0 - 0.0004), (0.0, x1 - x0)]
        col = lathe(prof, seg=7)
        col = noisy(col, 0.0003, scale=900, seed=int(x0 * 1e4) + 3)
        col.rot(0, 0, -math.pi / 2).scale(1.0, 0.82, 1.0)     # lying, slumped a little
        col.rot(0, a, 0).move(x0, 0, 0.0)
        s = s + on_floor(col)
    for k in range(4):
        r = rnd(0.0012, 0.0028)
        fl = sphere(r, seg=5, rings=3).scale(1.0, 0.35, rnd(0.6, 1.0))
        fl = noisy(fl, r * 0.3, scale=1200, seed=k + 40)
        a = rnd(0, TAU)
        d = rnd(0.006, 0.012)
        s = s + on_floor(fl).move(0.005 + d * math.cos(a), 0, d * math.sin(a))
    return on_floor(s)


# ===================================================== wet floor sign ===
# r01's sign is placed whole; its hinge (r01's `wetSignHinge`: the pin and
# four knuckles) is split here between the two panels' heads as the
# mouldings carry it, so each panel brings its half: the pin's half toward
# -x and the knuckles at -90 and +30 mm (the other panel, turned about y,
# brings the rest).
def wet_sign_hinge_half():
    pin = cyl(0.0035, h=0.096, seg=6, bevel_r=0).rot(0, 0, math.pi / 2).move(-0.048, 0, 0)
    knuckles = [cyl(0.008, h=0.03, seg=8, bevel_r=0.0008, bevel_seg=1).rot(0, 0, math.pi / 2).move(x, 0, 0) for x in (-0.09, 0.03)]
    return pin + sum(knuckles)


# ================================================================ swabs ===
# A 3 in (76.2 mm) double-tipped swab: a rolled paper stick 2.2 mm across,
# cotton wound on each end, 13 mm long and 5.4 mm across.
SWAB_L = 0.0762
SWAB_STICK_R = 0.0011
SWAB_TUFT_L, SWAB_TUFT_R = 0.013, 0.0027
SWAB_SEG = 8


def tuft(sign, press=0.0, seed=1):
    """A cotton tuft on the end of the stick toward `sign` (+1 or -1): a
    teardrop wound on it, lumpy; `press` packs it smaller and twisted."""
    tip = sign * SWAB_L / 2
    base = tip - sign * SWAB_TUFT_L
    prof = [(0.0, 0.0), (0.18, 0.55), (0.42, 0.85), (0.62, 1.0), (0.80, 0.93), (0.92, 0.70), (0.985, 0.36), (1.0, 0.0)]
    rings = []
    for t, k in prof[1:-1]:
        x = base + sign * SWAB_TUFT_L * t
        r = max(SWAB_STICK_R * 1.05, SWAB_TUFT_R * k * (1.0 - 0.35 * press))
        ring = ring_x(x, r * (1.0 - 0.25 * press), r * (1.0 + 0.15 * press), SWAB_SEG, a0=0.7 * press * t)
        rings.append(ring)
    # close both ends on the axis
    v0 = Vector((base - sign * 0.0003, 0, 0))
    v1 = Vector((tip, 0, 0))
    m = SWAB_SEG
    verts = [v0] + [p for ring in rings for p in ring] + [v1]
    f = []
    for j in range(m):
        f.append((0, 1 + (j + 1) % m, 1 + j))
    for i in range(len(rings) - 1):
        for j in range(m):
            j2 = (j + 1) % m
            a, b = 1 + i * m, 1 + (i + 1) * m
            f.append((a + j, a + j2, b + j2, b + j))
    last = 1 + (len(rings) - 1) * m
    for j in range(m):
        f.append((last + j, last + (j + 1) % m, len(verts) - 1))
    s = propkit._fix_winding(Shape(verts, f))
    s = noisy(s, 0.00035 * (1.0 - 0.5 * press), scale=1400, seed=seed)
    return noisy(s, 0.0002, scale=4000, seed=seed + 7)


def swab_stick(x0, x1, frayed=False):
    s = cyl(SWAB_STICK_R, SWAB_STICK_R, x1 - x0, seg=6, bevel_r=0.0002, bevel_seg=1).rot(0, 0, -math.pi / 2)
    s.move((x0 + x1) / 2, 0, 0)
    if frayed:
        s = noisy(s, 0.0002, scale=3000, seed=9)
    return s


def bend_xz(s, x0, angle):
    """Bend everything past x0 by `angle` about y (in the floor's plane)."""
    def f(p):
        if p.x <= x0:
            return p
        d = p.x - x0
        a = angle * min(1.0, d / 0.002)
        return Vector((x0 + d * math.cos(a) - p.z * math.sin(a), p.y, p.z + d * math.sin(a)))
    return s.warp(f)


def swab(variant):
    inner = SWAB_L / 2 - SWAB_TUFT_L * 0.55
    if variant == 'D':
        # the +x tuft pulled off: the stick's end bare and a little frayed, a
        # wisp of cotton left on it
        s = swab_stick(-inner, SWAB_L / 2 - 0.003, frayed=True) + tuft(-1, seed=3)
        wisp = sphere(0.0013, seg=6, rings=4).scale(2.2, 0.8, 0.9)
        s = s + noisy(wisp, 0.0003, scale=2500, seed=4).move(SWAB_L / 2 - 0.009, 0, 0)
    else:
        s = swab_stick(-inner, inner) + tuft(-1, seed=1) + tuft(1, press=0.8 if variant == 'C' else 0.0, seed=2)
    if variant == 'B':
        s = bend_xz(s, 0.006, math.radians(32))
    return on_floor(s.move(0, SWAB_TUFT_R, 0))


# ========================================================== paper clips ===
# The Gem's wire, in its plane: three U bends of one wire, its two ends
# lying inside (a No. 1 is 35 mm long and 9.5 mm across; a jumbo 48 mm).
# The wire a heavy gauge's 1.1 mm (a No. 1's 0.85 mm came out under a pixel
# at a few steps and broke into dashes: M8)
CLIP_WIRE_R = 0.00055


def arc_pts(c, r, a0, a1, n):
    return [(c[0] + r * math.cos(a0 + (a1 - a0) * i / n), c[1] + r * math.sin(a0 + (a1 - a0) * i / n)) for i in range(n + 1)]


def gem_path(L=0.035, W=0.0095):
    """The Gem's centre line [(x, z)] from its inner end to its outer end."""
    k = L / 0.035
    w = W / 2 - CLIP_WIRE_R
    za, zb = 0.0014 * k, -0.0026 * k                 # the inner loop's legs
    xr_in = L - 0.0065 * k                          # the inner right bend's centre
    xl = w + CLIP_WIRE_R                            # the outer left bend's centre
    pts = [(0.0095 * k, za)]
    pts += arc_pts((xr_in, (za + zb) / 2), (za - zb) / 2, math.pi / 2, -math.pi / 2, 8)
    cl = ((w + zb) / 2)
    pts += arc_pts((xl + 0.0012 * k, cl), (w - zb) / 2, -math.pi / 2, -1.5 * math.pi, 10)
    xr_out = L - w - CLIP_WIRE_R
    pts += arc_pts((xr_out, 0.0), w, math.pi / 2, -math.pi / 2, 10)
    pts.append((0.0105 * k, -w))
    return pts


def wire(pts3, seg=6):
    return tube(pts3, CLIP_WIRE_R, seg=seg, caps=True, smooth=0)


def paperclip_gem():
    p = gem_path()
    return on_floor(wire([(x, 0.0, z) for x, z in p]).move(-0.0175, 0, 0))


def paperclip_open():
    """Pulled open by a fidgeting hand: the inner loop as it was, the big
    bend opened out to a hook, the outer loop standing off it; all still on
    the floor (a half-unbent clip's S)."""
    za, zb = 0.0014, -0.0026
    pts = [(0.0095, za)]
    pts += arc_pts((0.0285, (za + zb) / 2), (za - zb) / 2, math.pi / 2, -math.pi / 2, 8)
    c, r = (0.0060, 0.00085), 0.00345
    end_a = -math.pi / 2 - math.radians(100)
    pts += arc_pts(c, r, -math.pi / 2, end_a, 7)
    x0, z0 = pts[-1]
    h = (math.sin(end_a), -math.cos(end_a))            # the way the wire runs on
    x1, z1 = x0 + h[0] * 0.018, z0 + h[1] * 0.018
    pts.append((x1, z1))
    right = (h[1], -h[0])
    r2 = 0.0043
    c2 = (x1 + right[0] * r2, z1 + right[1] * r2)
    a0 = math.atan2(z1 - c2[1], x1 - c2[0])
    pts += arc_pts(c2, r2, a0, a0 - math.pi, 10)[1:]
    xe, ze = pts[-1]
    pts.append((xe - h[0] * 0.014, ze - h[1] * 0.014))
    s = wire([(x, 0.0, z) for x, z in pts])
    lo, hi = s.bounds()
    return on_floor(s.move(-(lo.x + hi.x) / 2, 0, -(lo.z + hi.z) / 2))


def paperclip_jumbo():
    p = gem_path(0.048, 0.0125)
    pts = []
    for i, (x, z) in enumerate(p):
        t = i / (len(p) - 1)
        # the outer loop's free leg lifted, bent up 12 degrees from its bend
        y = 0.0
        if t > 0.80:
            y = (0.048 - 0.0125 - x) * math.tan(math.radians(12)) * smoothstep(0.80, 0.86, t)
            y = max(0.0, y)
        pts.append((x, y, z))
    return on_floor(wire(pts).move(-0.024, 0, 0))


# ============================================================ candy corn ===
# A piece of candy corn: 19 mm from its base to its tip, 13 mm across the
# base, 7.5 mm thick (about 2 g); lying on a broad face, base at x = 0.
CORN_L, CORN_W, CORN_T = 0.019, 0.013, 0.0075
CORN_SEG = 12


def corn_section(t):
    """Half width and half thickness at t (0 the base, 1 the tip)."""
    w = CORN_W / 2 * (1.0 - 0.80 * t ** 1.15)
    th = CORN_T / 2 * (1.0 - 0.45 * t)
    return w, th


def candy_corn(cut=1.0, slump=0.0, seed=1):
    """Lofted from base to tip: the base's edge rounded over, the sides
    tapering to a blunt rounded tip; `cut` < 1 snaps it there (a rough
    break face), `slump` lets it go soft and settle."""
    stations = [0.0, 0.03, 0.09, 0.18, 0.3, 0.42, 0.55, 0.67, 0.78, 0.87, 0.94, 0.985]
    stations = [t for t in stations if t <= cut + 1e-6]
    if cut < 1.0 and stations[-1] < cut:
        stations.append(cut)
    rings = []
    for t in stations:
        w, th = corn_section(t)
        if t < 0.09:                                # the base rounded over
            k = math.sin((0.5 + t / 0.09 * 0.5) * math.pi / 2) if t > 0 else 0.62
            w *= 0.82 + 0.18 * k
            th *= k
        if cut >= 1.0 and t > 0.9:                  # the tip rounded
            k = math.sqrt(max(0.0, 1.0 - ((t - 0.9) / 0.1) ** 2))
            w *= 0.5 + 0.5 * k
            th *= 0.5 + 0.5 * k
        ring = []
        for j in range(CORN_SEG):
            a = TAU * j / CORN_SEG
            c, s_ = math.cos(a), math.sin(a)
            # a rounded diamond-lens section: fuller than an ellipse
            yy = th * math.copysign(abs(c) ** 0.8, c)
            zz = w * math.copysign(abs(s_) ** 0.7, s_)
            ring.append(Vector((CORN_L * t, yy, zz)))
        rings.append(ring)
    s = loft(rings, cap0=True, cap1=True)
    if cut < 1.0:
        s = noisy(s, 0.00025, scale=2500, seed=seed + 11)
    s = noisy(s, 0.00008, scale=1500, seed=seed)
    if slump:
        lo, _ = s.bounds()

        def settle(p):
            h = p.y - lo.y
            return Vector((p.x, lo.y + h * (1.0 - 0.35 * slump), p.z * (1.0 + 0.12 * slump * (1.0 - h / CORN_T))))
        s.warp(settle)
    return on_floor(s)


# ============================================================== maracas ===
# A child's turned beech maraca (Bigjigs' junior: 133 mm, a 45 mm head),
# its head painted, its handle bare. Built along +x (the knob's end at 0),
# then tipped about z so it lies on its head and its knob (MARACA_TILT).
MAR_L = 0.133
MAR_HEAD_R = 0.0225
MAR_KNOB_R = 0.0105
MAR_HEAD_C = MAR_L - MAR_HEAD_R * 1.12           # the head's centre on the axis
MARACA_TILT = math.atan2(MAR_HEAD_R - MAR_KNOB_R, MAR_HEAD_C - 0.0105)


def maraca_head():
    """The head's section (r, x) from the collar's lip, where the handle
    goes in, to its crown: a turned collar, then an egg a little longer
    than it is wide (its centre MAR_HEAD_C, its half length 1.12 radii)."""
    c, a_len = MAR_HEAD_C, MAR_HEAD_R * 1.12
    prof = [(0.0, c - 0.034), (0.0074, c - 0.034), (0.0080, c - 0.0332), (0.0080, c - 0.0290),
            (0.0094, c - 0.0282), (0.0096, c - 0.0262), (0.0090, c - 0.0255)]
    a0 = -math.acos(0.0102 / MAR_HEAD_R)
    n = 16
    for i in range(n + 1):
        a = a0 + (math.pi / 2 - a0) * i / n
        prof.append((MAR_HEAD_R * math.cos(a), c + a_len * math.sin(a)))
    prof[-1] = (0.0, c + a_len)
    s = lathe(prof, seg=28, cap_bottom=False, cap_top=False)
    return s.rot(0, 0, -math.pi / 2)                 # its axis along +x


def maraca_handle():
    """The turned handle: a knob at its end, a bead, the grip, a bead
    under the head; it runs 9 mm up into the collar."""
    prof = [(0.0, 0.0), (0.0062, 0.0), (0.0090, 0.0012), (MAR_KNOB_R, 0.0060), (0.0098, 0.0125),
            (0.0074, 0.0175), (0.0068, 0.0205), (0.0074, 0.0215), (0.0068, 0.0228), (0.0069, 0.050),
            (0.0074, 0.058), (0.0079, 0.0605), (0.0074, 0.0630), (0.0071, MAR_HEAD_C - 0.025),
            (0.0, MAR_HEAD_C - 0.025)]
    s = lathe(prof, seg=16)
    return s.rot(0, 0, -math.pi / 2)


def maraca_lying(which):
    """Head or handle, tipped as the whole maraca lies (both by one lift)."""
    head = maraca_head().rot(0, 0, MARACA_TILT)
    handle = maraca_handle().rot(0, 0, MARACA_TILT)
    lift = -min(head.bounds()[0].y, handle.bounds()[0].y)
    return (head if which == 'head' else handle).move(0, lift, 0)


# ======================================================== twisty puzzle ===
# A 57 mm 3x3x3 (the original's size): cubies 18.6 mm on a 19 mm pitch,
# their edges rounded; stickers 15.6 mm square with rounded corners.
CUBE = 0.057
CUBE_PITCH = CUBE / 3
CUBIE = CUBE_PITCH - 0.0005
STICKER = 0.0156
STICKER_T = 0.00012
# the stickers that peel (face axis, sign, a, b), and the one gone
PEEL = ((0, 1, 1, -1), (2, 1, -1, 1))
GONE = (1, 1, 1, 1)


def twisty_cubies():
    s = Shape()
    for i in (-1, 0, 1):
        for j in (-1, 0, 1):
            for k in (-1, 0, 1):
                if i == j == k == 0:
                    continue
                b = rbox(CUBIE, CUBIE, CUBIE, r=0.0016, seg=2)
                s = s + b.move(i * CUBE_PITCH, j * CUBE_PITCH + CUBE / 2, k * CUBE_PITCH)
    return on_floor(s)


def sticker(axis, sign, a, b, peel=False):
    """One sticker on the face whose normal is `axis` (0 x, 1 y, 2 z) toward
    `sign`, at facelet (a, b) of the face's other two axes in order."""
    n = 6 if peel else 1
    outline = rounded_rect(STICKER, STICKER, 0.0018, seg=2)
    # the top face as a fan from its middle, or a grid when it peels
    if not peel:
        v = [Vector((0.0, 0.0, 0.0))] + [Vector((x, y, 0.0)) for x, y in outline]
        m = len(outline)
        f = [(0, 1 + j, 1 + (j + 1) % m) for j in range(m)]
        sh = Shape(v, f)
    else:
        sh = grid_plane(STICKER, STICKER, n, n)
        # its corner at (+, +) lifted, curling up from a crease across it
        def lift(p):
            d = (p.x + p.y) / math.sqrt(2) - STICKER * 0.18
            if d > 0:
                return Vector((p.x - d * 0.12, p.y - d * 0.12, d * 0.3 + d * d * 12))
            return p
        sh.warp(lift)
    # lay it on the face: its plane's (x, y) onto the face's (a, b) axes
    others = [k for k in range(3) if k != axis]
    out = Shape()
    for p in sh.v:
        q = [0.0, 0.0, 0.0]
        q[others[0]] = a * CUBE_PITCH + p.x
        q[others[1]] = b * CUBE_PITCH + p.y
        q[axis] = sign * (CUBE / 2 - 0.00025 + STICKER_T + p.z)
        out.v.append(Vector(q))
    out.f = list(sh.f)                              # turned outward by the caller
    return out


def twisty_stickers():
    s = Shape()
    for axis in range(3):
        for sign in (1, -1):
            for a in (-1, 0, 1):
                for b in (-1, 0, 1):
                    if (axis, sign, a, b) == GONE:
                        continue
                    st = sticker(axis, sign, a, b, peel=(axis, sign, a, b) in PEEL)
                    s = s + st
    s.move(0, CUBE / 2, 0)
    # outward winding: every face's normal away from the cube's middle
    c = Vector((0, CUBE / 2, 0))
    fixed = []
    for f in s.f:
        a, b, cc = s.v[f[0]], s.v[f[1]], s.v[f[2]]
        nrm = (b - a).cross(cc - a)
        mid = sum((s.v[i] for i in f), Vector()) / len(f)
        fixed.append(f if nrm.dot(mid - c) >= 0 else tuple(reversed(f)))
    s.f = fixed
    return s


# ================================================================== boa ===
# A short chandelle boa, a child's dress-up one: turkey feathers twisted on
# a cord, about 90 mm across, lying it settles to about 65 mm high. Made of
# lengths: the game lays twelve of them along the curve it fell in,
# overlapping (each one 75 mm along its cord, x; its middle at the origin).
BOA_SEG_L = 0.075


def boa_seg_core(seed):
    """The dense fluff round the cord: a lumpy rope 46 mm across, settled
    onto the floor, its ends tapering into the next length's."""
    rnd_ = random.Random(seed)
    rings = []
    seg = 9
    n = 6
    for i in range(n + 1):
        x = -BOA_SEG_L / 2 + BOA_SEG_L * i / n
        k = 1.0 - 0.25 * (abs(x) / (BOA_SEG_L / 2)) ** 3
        ring = []
        for j in range(seg):
            a = TAU * j / seg
            r = 0.016 * k * (1.0 + 0.16 * math.sin(i * 1.7 + j * 2.3 + seed))
            ring.append(Vector((x, 0.028 + math.sin(a) * r * 0.85, math.cos(a) * r)))
        rings.append(ring)
    s = loft(rings, cap0=True, cap1=True)
    s = noisy(s, 0.004, scale=60, seed=seed)
    s.warp(lambda q: Vector((q.x, max(q.y, 0.0012 + 0.0004 * math.sin(q.x * 900 + q.z * 700)), q.z)))
    return s


def plume(base, out, up, length, curl, rnd_):
    """One chandelle feather from `base` toward `out`: its rachis curling
    over, its barbs fine and long, each a thin blade, shorter toward the
    tip, wisping out to either side and lifting."""
    tris_v, tris_f = [], []
    side = out.cross(up).normalized()
    n = 5
    pts = []
    for i in range(n + 1):
        t = i / n
        a = curl * t * t
        d = (out * math.cos(a) + up * math.sin(a)).normalized()
        pts.append(base.copy() if i == 0 else pts[-1] + d * (length / n))
    for i in range(1, n + 1):
        t = i / n
        p, q = pts[i], pts[i - 1]
        along = (p - q).normalized()
        bl = length * 0.55 * (1.0 - 0.45 * t) * rnd_.uniform(0.8, 1.15)
        for sgn in (-1, 1):
            tip = p + side * (sgn * bl * 0.6) + along * bl * rnd_.uniform(0.4, 0.8) + up * bl * rnd_.uniform(-0.3, 0.3)
            w = side * 0.0016 + up * 0.0006
            k = len(tris_v)
            tris_v += [p - w, p + w, tip]
            tris_f.append((k, k + 1, k + 2))
    k = len(tris_v)
    tip = pts[-1] + (pts[-1] - pts[-2]) * 1.2
    tris_v += [pts[-1] + side * 0.001, pts[-1] - side * 0.001, tip]
    tris_f.append((k, k + 1, k + 2))
    return Shape(tris_v, tris_f)


def boa_seg_feathers(seed):
    """The feathers of one length, wound round its cord: standing out all
    round and leaning back along it, those underneath pressed out flat on
    the floor, a few strays longer."""
    rnd_ = random.Random(seed)
    s = Shape()
    count = 80
    t = Vector((1, 0, 0))
    for i in range(count):
        x = -BOA_SEG_L / 2 + BOA_SEG_L * (i + rnd_.random()) / count
        a = TAU * (i * 0.382 + rnd_.uniform(-0.1, 0.1))
        dirv = Vector((0, math.sin(a), math.cos(a)))
        if dirv.y < -0.2:
            dirv = Vector((0, 0.1, math.copysign(1.0, dirv.z)))
        dirv = (dirv + t * rnd_.uniform(-0.7, 0.3) + Vector((0, 0.3, 0))).normalized()
        base = Vector((x, 0.028, 0.0)) + dirv * 0.009
        up = t.cross(dirv).normalized() if abs(t.dot(dirv)) < 0.95 else Vector((0, 1, 0))
        length = rnd_.uniform(0.032, 0.046) * (1.3 if rnd_.random() < 0.08 else 1.0)
        curl = rnd_.uniform(0.15, 0.6) * (1 if rnd_.random() < 0.5 else -1)
        s = s + plume(base, dirv, up, length, curl, rnd_)
    # nothing below the floor: what reaches it lies on it
    s.warp(lambda q: Vector((q.x, max(q.y, 0.0010 + 0.0004 * math.sin(q.x * 900 + q.z * 700)), q.z)))
    return s


# ---------------------------------------------------------------- parts ---
auto('Real__cigButtA', lambda: butt(0.016, crush=1.0, kink=math.radians(24), ragged=0.5, seed=11), 260)
auto('Real__cigButtB', lambda: butt(0.005, oval=0.28, ragged=0.7, seed=12), 220)
auto('Real__cigButtC', lambda: butt(0.028, ragged=1.0, seed=13), 300)
auto('Real__cigButtD', lambda: butt(0.012, crush=0.6, flat=0.30, ragged=0.6, seed=14), 240)
auto('Real__cigButtAsh', cig_ash, 220)
auto('Real__wetSignHingeHalf', wet_sign_hinge_half, 160)
auto('Real__swabA', lambda: swab('A'), 380, smooth=60.0)
auto('Real__swabB', lambda: swab('B'), 380, smooth=60.0)
auto('Real__swabC', lambda: swab('C'), 380, smooth=60.0)
auto('Real__swabD', lambda: swab('D'), 300, smooth=60.0)
auto('Real__paperclipGem', paperclip_gem, 420, smooth=70.0)
auto('Real__paperclipOpen', paperclip_open, 420, smooth=70.0)
auto('Real__paperclipJumbo', paperclip_jumbo, 420, smooth=70.0)
auto('Real__candyCornA', lambda: candy_corn(seed=1), 320, smooth=60.0)
auto('Real__candyCornB', lambda: candy_corn(cut=0.72, seed=2), 320, smooth=60.0)
auto('Real__candyCornC', lambda: candy_corn(slump=1.0, seed=3), 320, smooth=60.0)
auto('Real__maracaHead', lambda: maraca_lying('head'), 1300, smooth=50.0)
auto('Real__maracaHandle', lambda: maraca_lying('handle'), 600, smooth=50.0)
auto('Real__twistyCubies', twisty_cubies, 3200)
auto('Real__twistyStickers', twisty_stickers, 900)
auto('Real__boaCoreA', lambda: boa_seg_core(1), 140, smooth=180.0)
auto('Real__boaCoreB', lambda: boa_seg_core(2), 140, smooth=180.0)
auto('Real__boaCoreC', lambda: boa_seg_core(3), 140, smooth=180.0)
auto('Real__boaFeathersA', lambda: boa_seg_feathers(11), 960, smooth=180.0)
auto('Real__boaFeathersB', lambda: boa_seg_feathers(12), 960, smooth=180.0)
auto('Real__boaFeathersC', lambda: boa_seg_feathers(13), 960, smooth=180.0)
