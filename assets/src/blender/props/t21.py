"""t21: four larger rare things made real (Props P3, docs/design/PROPS.md): a brass
candle lantern hanging on its chain, a small drop-front desk, a ten-gallon
aquarium on its stand left for years, and a leaded stained glass panel in a
frame on a wall. Sources and sizes: t21.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is modelled at
its real size in its prop's own frame: x across its front, y up, z out of
its front. Every part is named `Real__<prop><Part>`:

lantern*      a small Victorian-style brass candle lantern, 150 mm square
              over its base, 390 mm from its drop to the top of its ring;
              y = 0 is where its candle's flame burns (the light):
              Brass (the base tray and its drop, the corner posts, rails,
              the pyramid roof, the vented chimney and its cap, the finial
              and ring, the door's hinges and latch, the candle cup), Glass
              (four panes), Candle (a 7/8 in taper burnt to a stub in the
              cup, its runs over the cup's rim; the wick and flame are t09's
              `Real__brassStickWickB` / `FlameB`, set 180 mm lower), Soot
              (the roof's underside over the panes, black), Canopy
              (the ceiling cup and hook; its top at y = 0), Chain1, Chain4,
              Chain16 (jack chain links, their foot at y = 0, up +y)
secDesk*      a petite drop-front desk on tapered legs, 570 x 355 x 1100 mm,
              its fall front open (lying flat, its free edge 470 mm out from
              the middle); the floor at y = 0, the case centred on z = 0:
              Case (legs, sides, rails, bed, back, the fitted interior:
              small drawers' carcase, pigeonholes and valances), Fronts (the
              two drawer fronts, the four small ones, the fall front, the
              top and its crest: their grain across), Leather (the fall's writing leather), Brass (bail
              pulls, escutcheons, knobs, the fall's hinges and its stays),
              LettersA / LettersB (what stands in a pigeonhole, on its
              floor at the origin), Sheet (a half sheet of note paper,
              lying), Inkwell (a pressed glass inkwell), InkCap (its brass
              cap), Ink (the ink dried in it)
aquarium*     a standard 10 gallon tank (20 1/4 x 10 1/2 x 12 5/8 in) on a
              low laminated particleboard cabinet stand; the floor at y = 0:
              Stand (the cabinet, its doors, its swollen foot), StandTop
              (its top board, apart for its own grain), Knobs,
              Glass (five panes), Trim (the black rims and the corner
              silicone), Gravel, Hood (a black full hood, its feeding flap),
              Tube (its fluorescent tube, under the hood), Airline (the
              vinyl tube from the stone over the back rim into the stand),
              Airstone
stainedGlass* a Victorian leaded panel, 300 x 440 mm in sight, in a pine
              frame 340 x 480 mm hung on the wall; y = 0 its middle, the
              wall at z = -0.010: Frame (the moulding), Panes (the glass,
              one sheet, the pieces painted), Lead (the cames, the edge came
              and the solder at every joint), Bar (the iron saddle bar and
              its ties)
"""
from propkit import *
import propkit
import random
import zlib

SQ2 = math.sqrt(2.0)


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, kind='box', flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth, flat=flat)


def rnd(a, b):
    return a + (b - a) * random.random()


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def tri_count(s):
    return sum(len(f) - 2 for f in s.f)


def sq_lathe(profile, cap_bottom=True, cap_top=True):
    """A profile [(half-width, y)] revolved round +Y as a square (four
    sides, mitred corners): a lathe of four, turned 45 degrees."""
    s = lathe([(r * SQ2, y) for r, y in profile], seg=4, cap_bottom=cap_bottom, cap_top=cap_top)
    return s.rot(0, math.pi / 4, 0)


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle in XY centred
    on the origin: d in from its outer edge, z out from the wall; the
    corners mitred (t10's)."""
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


def loop_sweep(pts, r, seg=6, normal=(0, 0, 1)):
    """A closed loop of points swept by a circle of radius r (a ring of
    wire); the loop lies in the plane whose normal is `normal`."""
    nrm = Vector(normal).normalized()
    n = len(pts)
    pts = [Vector(p) for p in pts]
    v, f = [], []
    for i in range(n):
        t = (pts[(i + 1) % n] - pts[i - 1]).normalized()
        side = t.cross(nrm).normalized()
        for j in range(seg):
            a = TAU * j / seg
            v.append(pts[i] + (side * math.cos(a) + nrm * math.sin(a)) * r)
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, i2 * seg + j2, i2 * seg + j))
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
            s.v[i] = p + ax * (noise.noise(q) * amount * k)
        else:
            s.v[i] = p + noise3(q) * (amount * k)
    return s


def bar_between(a, b, w, d, up=(0, 1, 0)):
    """A flat bar (w across, d thick) from point a to point b."""
    a, b = Vector(a), Vector(b)
    L = (b - a).length
    s = rbox(w, L, d, r=min(w, d) * 0.3, seg=1)
    q = Vector((0, 1, 0)).rotation_difference((b - a).normalized())
    s.apply(q.to_matrix().to_4x4())
    return s.move(*((a + b) / 2))


# ================================================================ lantern ===
# A small brass candle lantern of the Victorian hall kind (t21.notes.md):
# a square tray on a turned drop, four corner posts holding four panes (the
# front one a door on two hinges, its latch on the right post), a pyramid
# roof up to a vented chimney under a domed cap, a finial and a ring; a
# candle cup on the tray. y = 0 is the light: the flame's root 10 mm below.
LT_HALF = 0.065                 # the corner posts' outer half-width
LT_POST = 0.007                 # a post's square
LT_PANE_Y0, LT_PANE_Y1 = -0.065, 0.105
LT_SOCKET_FLOOR = -0.047        # where the candle's foot stands in its cup
LT_RIM = -0.035                 # the cup's rim
LT_RING_C, LT_RING_R, LT_RING_W = 0.2295, 0.0130, 0.0025


def lantern_brass():
    parts = []
    # the tray: a moulded square rim round the floor the candle stands on,
    # its underside an inverted pyramid down to the drop
    tray = sq_lathe([(0.0, -0.1120), (0.0200, -0.1120), (0.0225, -0.1100), (0.0660, -0.0800), (0.0725, -0.0775),
                     (0.0755, -0.0745), (0.0755, -0.0715), (0.0725, -0.0690), (0.0705, -0.0670), (0.0705, -0.0655),
                     (0.0670, -0.0650), (0.0, -0.0650)])
    parts.append(tray)
    # the drop under it: a turned finial
    drop = lathe([(0.0, -0.1430), (0.0035, -0.1425), (0.0065, -0.1390), (0.0085, -0.1340), (0.0080, -0.1290),
                  (0.0050, -0.1250), (0.0040, -0.1210), (0.0068, -0.1180), (0.0110, -0.1150), (0.0140, -0.1115),
                  (0.0, -0.1110)], seg=14)
    parts.append(drop)
    # four corner posts, a bead down each, a cast knob on each top corner
    for sx in (-1, 1):
        for sz in (-1, 1):
            x, z = sx * (LT_HALF - LT_POST / 2), sz * (LT_HALF - LT_POST / 2)
            post = rbox(LT_POST, LT_PANE_Y1 - LT_PANE_Y0 + 0.006, LT_POST, r=0.0012, seg=1)
            parts.append(post.move(x, (LT_PANE_Y0 + LT_PANE_Y1) / 2 + 0.001, z))
            knob = lathe([(0.0, 0.0), (0.0045, 0.0005), (0.0052, 0.003), (0.0030, 0.0058), (0.0035, 0.0075), (0.0, 0.0095)], seg=8)
            parts.append(knob.move(sx * (LT_HALF + 0.004), 0.1045, sz * (LT_HALF + 0.004)))
    # the rails round the panes' foot and head (flat bands, 7 mm)
    for y0, y1 in ((LT_PANE_Y0, LT_PANE_Y0 + 0.007), (LT_PANE_Y1 - 0.007, LT_PANE_Y1)):
        band = sq_lathe([(LT_HALF - 0.0015, y0), (LT_HALF + 0.0005, y0), (LT_HALF + 0.0005, y1), (LT_HALF - 0.0015, y1)],
                        cap_bottom=False, cap_top=False)
        parts.append(band)
    # the roof: a pyramid of sheet from its eave up to the chimney, its rolled
    # eave edge; then the vented chimney, its cap on four legs, the finial
    roof = sq_lathe([(LT_HALF - 0.003, 0.1040), (0.0760, 0.1040), (0.0790, 0.1065), (0.0780, 0.1090),
                     (0.0300, 0.1500), (0.0260, 0.1510), (0.0, 0.1510)], cap_bottom=False)
    parts.append(roof)
    chim = sq_lathe([(0.0230, 0.1500), (0.0230, 0.1790), (0.0205, 0.1790), (0.0205, 0.1500)], cap_bottom=False, cap_top=False)
    holes = []
    for k in range(4):
        for dx in (-0.009, 0.0, 0.009):
            c = cyl(0.0028, h=0.02, seg=8, bevel_r=0).rot(math.pi / 2, 0, 0).move(dx, 0.1650, 0.022)
            holes.append(c.rot(0, k * math.pi / 2, 0))
    chim = cut(chim, *holes)
    parts.append(chim)
    cap = lathe([(0.0, 0.1835), (0.0330, 0.1835), (0.0345, 0.1850), (0.0300, 0.1905), (0.0200, 0.1970),
                 (0.0090, 0.2005), (0.0, 0.2010)], seg=20)
    parts.append(cap)
    for k in range(4):
        a = math.pi / 4 + k * math.pi / 2
        parts.append(box(0.003, 0.0055, 0.003).move(math.cos(a) * 0.0215 / SQ2 * SQ2 * 0.85, 0.1812, math.sin(a) * 0.0215 * 0.85))
    finial = lathe([(0.0, 0.2000), (0.0050, 0.2000), (0.0058, 0.2030), (0.0038, 0.2060), (0.0045, 0.2100),
                    (0.0032, 0.2140), (0.0035, 0.2165), (0.0, 0.2170)], seg=10)
    parts.append(finial)
    ring = torus(LT_RING_R, LT_RING_W, seg=20, tseg=7).move(0, LT_RING_C, 0)
    parts.append(ring)
    # the door (the front pane): two hinges on the left post, the latch on
    # the right, a small loop to open it by
    for y in (LT_PANE_Y0 + 0.030, LT_PANE_Y1 - 0.030):
        k = cyl(0.0022, h=0.022, seg=8, bevel_r=0.0004, bevel_seg=1).move(-LT_HALF + 0.0005, y, LT_HALF + 0.0012)
        parts.append(k)
        parts.append(box(0.012, 0.020, 0.0008).move(-LT_HALF + 0.007, y, LT_HALF + 0.0006))
    latch = box(0.0035, 0.012, 0.0035).move(LT_HALF - 0.003, 0.020, LT_HALF + 0.0020)
    parts.append(latch)
    loop = torus(0.0045, 0.0009, seg=10, tseg=5).rot(math.pi / 2, 0, 0).move(LT_HALF - 0.011, 0.020, LT_HALF + 0.0012)
    parts.append(loop)
    # the candle cup on the tray: a foot, a waist, the cup and its lip
    cup = lathe([(0.0, -0.0650), (0.0200, -0.0650), (0.0200, -0.0630), (0.0170, -0.0605), (0.0095, -0.0570),
                 (0.0085, -0.0540), (0.0130, -0.0515), (0.0145, -0.0480), (0.0145, -0.0365), (0.0155, -0.0355),
                 (0.0148, LT_RIM), (0.0118, LT_RIM), (0.0115, LT_SOCKET_FLOOR), (0.0, LT_SOCKET_FLOOR)], seg=18,
                cap_bottom=False, cap_top=False)
    parts.append(cup)
    return sum(parts, Shape())


def lantern_soot():
    """The roof's underside over the panes, black with the candle's soot:
    a plate facing down, closing the roof's sheet."""
    h = LT_HALF - 0.0028
    return plane(2 * h, 2 * h).rot(math.pi / 2, 0, 0).move(0, 0.1039, 0)


def lantern_glass():
    """Four panes, set in behind the posts and the rails: each its outer
    face alone (the soot is on their inside, seen through them)."""
    w = 2 * (LT_HALF - LT_POST) + 0.004
    h = LT_PANE_Y1 - LT_PANE_Y0 - 0.004
    y = (LT_PANE_Y0 + LT_PANE_Y1) / 2
    o = LT_HALF - 0.0025
    panes = []
    for k in range(4):
        panes.append(plane(w, h).move(0, y, o).rot(0, k * math.pi / 2, 0))
    return sum(panes, Shape())


def lantern_candle():
    """The taper (7/8 in) burnt down to a stub in the cup: 38 mm above the
    cup's floor, its crater melted through on one side; the runs down it,
    over the cup's rim and down its side; a drop fallen on the tray (t09's
    taper, made for this cup). The wick and flame are t09's B."""
    MMv = 0.001
    H = 38.0
    rim = (LT_RIM - LT_SOCKET_FLOOR) / MMv          # 12 mm
    prof = [(0, 0), (10.9, 0), (11.0, 1.0)]
    for k in range(1, 12):
        y = H * k / 12
        prof.append((11.0 - 0.9 * y / 70.0, y))
    prof += [(10.6 - 0.9 * H / 70 + 0.2, H + 0.6), (9.8, H + 1.2), (8.6, H - 0.4), (6.0, H - 3.0), (2.5, H - 4.4), (0, H - 4.6)]
    s = lathe(prof, seg=24)

    def melt(p):
        a = math.atan2(p.z, p.x)
        near = max(0.0, math.cos(a + 0.6)) ** 3
        if p.y > H - 8:
            k = smoothstep(H - 8, H + 1, p.y)
            p.y -= near * k * 4.0
            r = math.hypot(p.x, p.z)
            if r > 1e-6:
                bulge = 1 + near * k * 0.06 + 0.03 * noise.noise(Vector((p.x * 0.4, p.y * 0.4, p.z * 0.4)))
                p.x *= bulge
                p.z *= bulge
        return p
    s.warp(melt)

    def run(a, y_from, y_to, rr, over=False):
        a = math.radians(a)
        pts, n = [], 10
        for i in range(n + 1):
            t = i / n
            y = y_from + (y_to - y_from) * t
            r = 11.1
            if over and y < rim + 0.6:
                r = 14.8 + rr * 0.6 if y > rim - 1.0 else 14.5 + rr * 0.6
            wob = 0.06 * math.sin(t * 7 + a * 3)
            pts.append(Vector((math.cos(a + wob) * r, y, math.sin(a + wob) * r)))
        tu = tube(pts, rr, seg=6, r_end=rr * 1.25)
        return tu + ellipsoid(rr * 1.5, rr * 2.0, rr * 1.5, seg=8).move(*pts[-1])

    runs = [run(-35, H - 3, -5.0, 1.8, over=True), run(-80, H - 4, 3.0, 1.4, over=True),
            run(40, H - 2, 18, 1.2), run(150, H - 2, -2.0, 1.5, over=True), run(230, H - 3, 22, 1.2)]
    collar = lathe([(11.0, rim - 1.0), (12.6, rim - 0.6), (13.4, rim + 0.2), (12.8, rim + 1.2), (11.4, rim + 2.2), (10.9, rim + 2.6)], seg=24)
    s = s + collar + sum(runs, Shape())
    # the drop that fell from the rim onto the tray
    s = s + ellipsoid(5.0, 1.1, 3.6, seg=10).rot(0, 0.4, 0).move(24.0, -18.0 + 0.6, -9.0)
    s = Shape([p * MMv for p in s.v], s.f)
    return s.move(0, LT_SOCKET_FLOOR, 0)


def lantern_canopy():
    """The ceiling cup the chain hangs in, its top on the ceiling (y = 0):
    a shallow brass dome 70 mm across, the screw through its middle, the
    hook inside."""
    cup = lathe([(0.0, -0.0300), (0.0050, -0.0300), (0.0060, -0.0290), (0.0120, -0.0270), (0.0240, -0.0210),
                 (0.0320, -0.0120), (0.0350, -0.0040), (0.0355, -0.0005), (0.0350, 0.0)], seg=24, cap_top=True)
    hole = cyl(0.0042, h=0.01, seg=10, bevel_r=0).move(0, -0.030, 0)
    cup = cut(cup, hole)
    hook = torus(0.0045, 0.0012, seg=10, tseg=5).move(0, -0.0255, 0)
    return cup + hook


LINK_W, LINK_L, LINK_R = 0.012, 0.022, 0.0012       # a link's outer width, length, wire radius
LINK_P = LINK_L - 2 * LINK_R                       # the chain's pitch


def chain_link():
    """One link of brass jack chain in the XY plane, its foot at y = 0."""
    a = LINK_W / 2 - LINK_R
    s = (LINK_L - 2 * LINK_R) - 2 * a
    c0, c1 = LINK_R + a, LINK_R + a + s
    pts = []
    for i in range(7):
        t = math.pi + math.pi * i / 6
        pts.append((a * math.cos(t), c0 + a * math.sin(t), 0))
    for i in range(7):
        t = math.pi * i / 6
        pts.append((a * math.cos(t), c1 + a * math.sin(t), 0))
    return loop_sweep(pts, LINK_R, seg=5)


def chain(n):
    links = []
    for i in range(n):
        lk = chain_link()
        if i % 2:
            lk.rot(0, math.pi / 2, 0)
        links.append(lk.move(0, i * LINK_P, 0))
    return sum(links, Shape())


auto('Real__lanternBrass', lantern_brass, 4200, smooth=40.0)
auto('Real__lanternSoot', lantern_soot, 20, smooth=10.0)
auto('Real__lanternGlass', lantern_glass, 120, smooth=10.0)
auto('Real__lanternCandle', lantern_candle, 2600, smooth=45.0, kind='cyl')
auto('Real__lanternCanopy', lantern_canopy, 700, smooth=45.0, kind='cyl')
auto('Real__lanternChain1', lambda: chain(1), 160, smooth=60.0)
auto('Real__lanternChain4', lambda: chain(4), 600, smooth=60.0)
auto('Real__lanternChain16', lambda: chain(16), 2400, smooth=60.0)


# =========================================================== secretary desk ===
# A petite drop-front desk (t21.notes.md): four square tapered legs, a case
# of two drawers under a writing bed, the fall front hinged at the bed and
# lying open on two brass stays, the fitted interior behind it (four small
# drawers, six pigeonholes under arched valances), a top with a moulded
# edge and a low crest at its back. Case 550 x 340 mm, top 570 x 355,
# 1100 mm to the crest; the bed (and the fall's face) at 740 mm.
SD_W, SD_D = 0.550, 0.340
SD_SIDE = 0.018
SD_LEG_H = 0.280
SD_BED = 0.740                  # the writing bed's top, the fall's hinge line
SD_TOP0 = 1.040                 # the top board's underside
SD_FRONT = SD_D / 2             # the case's front (z)
SD_BACK = -SD_D / 2
SD_FIT_Z = 0.080                # the fitted interior's front
SD_FALL = (0.512, 0.300, 0.020)  # the fall: width, depth (open), thickness
SD_DRAWERS = ((0.302, 0.487), (0.505, 0.690))   # the two drawer openings (y)
SD_HOLES = 6
SD_INNER = SD_W / 2 - SD_SIDE   # half the inside width
SD_SMALL_Y = (SD_BED, SD_BED + 0.060)
SD_SHELF = SD_BED + 0.066       # the pigeonholes' floor


def sd_leg():
    """A square leg tapered on its two inner faces (as they were cut), from
    35 mm at its head to 22 mm at its foot, the foot's edges eased."""
    s = rbox(0.035, SD_LEG_H, 0.035, r=0.0025, seg=1).move(0, SD_LEG_H / 2, 0)
    for p in s.v:
        t = 1.0 - p.y / SD_LEG_H
        k = 0.013 * smoothstep(0.0, 1.0, t)
        if p.x > 0:
            p.x -= k
        if p.z > 0:
            p.z -= k
    return s


def sd_case():
    parts = []
    hx, hz = SD_W / 2, SD_D / 2
    # legs: outer faces flush with the case's sides, front and back
    for sx in (-1, 1):
        for sz in (-1, 1):
            leg = sd_leg()
            leg.scale(-sx if sx < 0 else 1, 1, -sz if sz < 0 else 1)
            leg.scale(-1, 1, -1)       # its tapered faces inward
            parts.append(leg.move(sx * (hx - 0.0175), 0, sz * (hz - 0.0175)))
    # the sides, from the legs' heads to the top
    for sx in (-1, 1):
        side = rbox(SD_SIDE, SD_TOP0 - SD_LEG_H, SD_D, r=0.0015, seg=1)
        parts.append(side.move(sx * (hx - SD_SIDE / 2), (SD_TOP0 + SD_LEG_H) / 2, 0))
    # the back (a thin panel in the sides' rebates)
    parts.append(box(SD_W - 2 * SD_SIDE + 0.006, SD_TOP0 - SD_LEG_H, 0.008).move(0, (SD_TOP0 + SD_LEG_H) / 2, SD_BACK + 0.006))
    # the apron rail under the lower drawer, its foot cut in a shallow arch
    apron = []
    n = 16
    for i in range(n + 1):
        x = -SD_INNER + 2 * SD_INNER * i / n
        t = x / SD_INNER
        apron.append((x, SD_LEG_H - 0.010 + 0.016 * (t * t)))
    outline = [(SD_INNER, SD_DRAWERS[0][0])] + [(-SD_INNER, SD_DRAWERS[0][0])] + apron
    rail = prism(outline, 0.020, bevel_r=0.0012)
    parts.append(rail.move(0, 0, SD_FRONT - 0.010))
    # the rails between the drawers and over them, and the bed
    for y0, y1 in ((SD_DRAWERS[0][1], SD_DRAWERS[1][0]), (SD_DRAWERS[1][1], SD_BED - 0.018)):
        parts.append(rbox(2 * SD_INNER, y1 - y0, 0.020, r=0.0012, seg=1).move(0, (y0 + y1) / 2, SD_FRONT - 0.010))
    bed = rbox(2 * SD_INNER, 0.018, SD_D - 0.010, r=0.0015, seg=1)
    parts.append(bed.move(0, SD_BED - 0.009, 0.0))
    # the drawers' runners and the dark behind each opening: a bottom board
    # under each drawer and its sides, set back (seen in the reveal)
    for y0, y1 in SD_DRAWERS:
        parts.append(box(2 * SD_INNER, 0.010, SD_D - 0.040).move(0, y0 - 0.002, -0.012))
    # the fitted interior: the small drawers' carcase, the pigeonholes
    fit_d = SD_FIT_Z - (SD_BACK + 0.010)
    fz = (SD_FIT_Z + SD_BACK + 0.010) / 2
    # shelf over the small drawers, and the dividers between them
    parts.append(box(2 * SD_INNER, 0.006, fit_d).move(0, SD_SHELF - 0.003, fz))
    for i in range(1, 4):
        x = -SD_INNER + 2 * SD_INNER * i / 4
        parts.append(box(0.006, SD_SHELF - SD_BED - 0.006, fit_d).move(x, (SD_BED + SD_SHELF - 0.006) / 2, fz))
    # pigeonhole dividers up to the top, and the valances with their arches
    hole_w = 2 * SD_INNER / SD_HOLES
    for i in range(1, SD_HOLES):
        x = -SD_INNER + hole_w * i
        d = rbox(0.006, SD_TOP0 - SD_SHELF, fit_d, r=0.001, seg=1)
        parts.append(d.move(x, (SD_TOP0 + SD_SHELF) / 2, fz))
    for i in range(SD_HOLES):
        x0 = -SD_INNER + hole_w * i
        pts = []
        m = 10
        for k in range(m + 1):
            t = k / m
            x = x0 + 0.003 + (hole_w - 0.006) * t
            y = SD_TOP0 - 0.040 + 0.024 * (1 - math.sin(math.pi * t) ** 0.8)
            pts.append((x, y))
        outline = [(x0 + hole_w - 0.003, SD_TOP0), (x0 + 0.003, SD_TOP0)] + pts
        val = prism(outline, 0.006, bevel_r=0.0008)
        parts.append(val.move(0, 0, SD_FIT_Z - 0.008))
    return sum(parts, Shape())


def sd_top():
    """The top, a board with its front and side edges rounded over, and the
    crest along its back (their grain across, as the fronts')."""
    parts = []
    # the top: a board, its edges rounded over, overhanging the case
    top_w, top_d = 0.570, 0.355
    tb = rbox(top_w, 0.018, top_d, r=0.0045, seg=2)
    parts.append(tb.move(0, SD_TOP0 + 0.009, SD_BACK + top_d / 2))
    # the crest at its back: a shaped board, rising to the middle in a
    # gentle arch, its ends scrolled down
    crest = []
    n = 24
    cw = 0.52
    for i in range(n + 1):
        x = -cw / 2 + cw * i / n
        t = abs(x) / (cw / 2)
        y = 0.020 + 0.022 * (1 - t * t) ** 0.7
        if t > 0.86:
            y = 0.020 * (1 - (t - 0.86) / 0.14) + 0.006
        crest.append((x, y))
    outline = [(cw / 2, 0.0), (-cw / 2, 0.0)] + crest
    c = prism(outline, 0.016, bevel_r=0.0015)
    parts.append(c.move(0, SD_TOP0 + 0.018, SD_BACK + 0.016))
    return sum(parts, Shape())


def sd_fronts():
    parts = []
    # the two drawers: fronts a little inside their openings (a 2 mm
    # reveal), their edges thumbnail-moulded; the drawer's box behind
    for y0, y1 in SD_DRAWERS:
        f = rbox(2 * SD_INNER - 0.004, y1 - y0 - 0.004, 0.020, r=0.004, seg=2)
        parts.append(f.move(0, (y0 + y1) / 2, SD_FRONT - 0.011))
    # four small drawers in the interior
    w = 2 * SD_INNER / 4
    for i in range(4):
        x = -SD_INNER + w * (i + 0.5)
        f = rbox(w - 0.010, SD_SMALL_Y[1] - SD_SMALL_Y[0] - 0.006, 0.010, r=0.0018, seg=1)
        parts.append(f.move(x, (SD_SMALL_Y[0] + SD_SMALL_Y[1] - 0.006) / 2 + 0.0005, SD_FIT_Z - 0.004))
    # the fall front lying open: its face (now up) flush with the bed, a
    # moulded edge round its free end and sides; the leather is let into it
    fw, fd, ft = SD_FALL
    fall = rbox(fw, ft, fd, r=0.004, seg=2)
    recess = box(0.440, 0.004, 0.236).move(0, ft / 2, 0.012)
    fall = cut(fall, recess)
    parts.append(fall.move(0, SD_BED - ft / 2, SD_FRONT + fd / 2 + 0.001))
    parts.append(sd_top())
    return sum(parts, Shape())


def sd_leather():
    """The fall's leather, let into its face: a skin 1 mm proud of the
    recess's floor, a tooled line round it 12 mm in, a little cockled."""
    s = slab(0.436, 0.232, 0.0012, nx=12, ny=8).rot(-math.pi / 2, 0, 0)
    warp_noise(s, 0.0004, 18.0, 3, axis=(0, 1, 0))
    fw, fd, ft = SD_FALL
    return s.move(0, SD_BED - 0.0022, SD_FRONT + fd / 2 + 0.001 + 0.012)


def sd_bail(x, y, z):
    """A Hepplewhite bail pull: an oval stamped plate, two posts and the
    bail hanging from them."""
    plate = lathe([(0.0, 0.0), (0.0240, 0.0), (0.0245, 0.0006), (0.0220, 0.0013), (0.0120, 0.0018), (0.0, 0.0019)], seg=18)
    plate.rot(math.pi / 2, 0, 0).scale(1.0, 0.64, 1.0)
    parts = [plate]
    for sx in (-1, 1):
        parts.append(cyl(0.0022, h=0.008, seg=8, bevel_r=0.0005, bevel_seg=1).rot(math.pi / 2, 0, 0).move(sx * 0.017, 0.002, 0.005))
    pts = []
    for i in range(11):
        t = i / 10
        a = math.pi * t
        pts.append(Vector((-0.017 * math.cos(a), 0.002 - 0.016 * math.sin(a), 0.0075)))
    parts.append(tube(pts, 0.0016, seg=6))
    return sum(parts, Shape()).move(x, y, z)


def sd_escutcheon(x, y, z):
    """A small oval keyhole plate, the keyhole pierced in it."""
    p = lathe([(0.0, 0.0), (0.0105, 0.0), (0.0110, 0.0005), (0.0095, 0.0012), (0.0, 0.0013)], seg=14)
    p.rot(math.pi / 2, 0, 0).scale(0.62, 1.0, 1.0)
    hole = cyl(0.0018, h=0.01, seg=8, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0.002, 0)
    slot = box(0.0016, 0.006, 0.01).move(0, -0.002, 0)
    return cut(p, hole, slot).move(x, y, z)


def sd_brass():
    parts = []
    for y0, y1 in SD_DRAWERS:
        yc = (y0 + y1) / 2
        for sx in (-1, 1):
            parts.append(sd_bail(sx * 0.130, yc + 0.012, SD_FRONT))
        parts.append(sd_escutcheon(0.0, yc + 0.004, SD_FRONT))
    # the small drawers' turned knobs
    w = 2 * SD_INNER / 4
    for i in range(4):
        x = -SD_INNER + w * (i + 0.5)
        k = lathe([(0.0, 0.0), (0.0040, 0.0), (0.0030, 0.0035), (0.0034, 0.0060), (0.0062, 0.0090), (0.0060, 0.0120),
                   (0.0035, 0.0140), (0.0, 0.0145)], seg=10)
        parts.append(k.rot(math.pi / 2, 0, 0).move(x, (SD_SMALL_Y[0] + SD_SMALL_Y[1]) / 2 - 0.002, SD_FIT_Z + 0.001))
    # the fall's two butt hinges, their knuckles along the hinge line
    for sx in (-1, 1):
        kn = cyl(0.0032, h=0.050, seg=10, bevel_r=0.0006, bevel_seg=1).rot(0, 0, math.pi / 2)
        parts.append(kn.move(sx * 0.165, SD_BED - 0.002, SD_FRONT + 0.001))
    # the fall's escutcheon on its edge, now underneath
    # the stays: a flat bar from a plate on the fall's face up to a pivot on
    # the side's inner face, each with its screw heads
    fw, fd, ft = SD_FALL
    for sx in (-1, 1):
        x = sx * (SD_INNER - 0.006)
        a = (x, SD_BED + 0.004, SD_FRONT + 0.150)
        b = (x, SD_BED + 0.170, SD_FIT_Z + 0.040)
        parts.append(bar_between(a, b, 0.003, 0.012).rot(0, 0, 0))
        plate = rbox(0.008, 0.002, 0.030, r=0.0006, seg=1).move(x, SD_BED + 0.001, SD_FRONT + 0.150)
        parts.append(plate)
        piv = cyl(0.006, h=0.004, seg=10, bevel_r=0.0008, bevel_seg=1).rot(0, 0, math.pi / 2).move(x - sx * 0.000, b[1], b[2])
        parts.append(piv)
    return sum(parts, Shape())


def sd_letters_a():
    """Envelopes standing in a pigeonhole, leaning: three, a little bowed,
    their flaps up; on its floor at the origin, x across the hole."""
    parts = []
    for i, (lean, dx, w, h) in enumerate(((0.00, -0.020, 0.064, 0.150), (0.10, -0.012, 0.064, 0.150), (0.24, -0.004, 0.064, 0.146))):
        s = slab(w * 2.3 / 2.3, h, 0.0012, nx=3, ny=6)          # a long envelope, stood on its end
        s = bend(s, 0.10 + i * 0.03, axis='y', along='x')
        s.rot(0, math.pi / 2, 0)
        lo, hi = s.bounds()
        s.move(0, -lo.y, 0)
        s.rot(0, 0, -lean).move(dx + i * 0.004, 0.0002, 0)
        parts.append(s)
    out = sum(parts, Shape())
    lo, hi = out.bounds()
    return out.move(0, -lo.y, 0)


def sd_letters_b():
    """A bundle of papers rolled and lying across the pigeonhole's floor,
    and a letter folded in three lying under it."""
    roll = lathe([(0.0, -0.034), (0.0060, -0.034), (0.0072, -0.030), (0.0072, 0.030), (0.0060, 0.034), (0.0, 0.034)], seg=12)
    roll.rot(0, 0, math.pi / 2).move(0, 0.0074 + 0.003, -0.010)
    folded = slab(0.070, 0.105, 0.0025, nx=4, ny=4).rot(-math.pi / 2, 0, 0)
    folded = bend(folded, 0.05, axis='x', along='z')
    lo, hi = folded.bounds()
    folded.move(0, -lo.y, 0.010)
    return roll + folded


def sd_sheet():
    """A half sheet of note paper (5 1/2 x 8 1/2 in) lying, its corners
    lifting where it curled."""
    s = grid_plane(0.140, 0.216, nx=6, ny=8).rot(-math.pi / 2, 0, 0)
    for p in s.v:
        r = max(abs(p.x) / 0.070, abs(p.z) / 0.108)
        p.y += 0.0022 * smoothstep(0.65, 1.0, r) + 0.0008 * noise.noise(Vector((p.x * 40, 1.3, p.z * 40)))
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


def sd_inkwell():
    """A pressed glass square inkwell, 45 mm square, 48 mm tall, the well
    a round bore; the brass cap's collar at its mouth."""
    body = rbox(0.045, 0.040, 0.045, r=0.004, seg=2).move(0, 0.020, 0)
    neck = cyl(0.0120, h=0.008, seg=16, bevel_r=0.001).move(0, 0.044, 0)
    s = body + neck
    bore = cyl(0.0095, h=0.040, seg=16, bevel_r=0).move(0, 0.032, 0)
    return cut(s, bore)


def sd_ink_cap():
    """Its hinged brass cap, open and thrown back on its hinge."""
    cap = lathe([(0.0, 0.0), (0.0140, 0.0), (0.0145, 0.0040), (0.0120, 0.0075), (0.0060, 0.0095), (0.0, 0.0098)], seg=16)
    cap.rot(-1.9, 0, 0).move(0, 0.050, -0.016)
    collar = lathe([(0.0128, 0.0), (0.0136, 0.0), (0.0136, 0.0060), (0.0128, 0.0060)], seg=16, cap_bottom=False, cap_top=False)
    hinge = cyl(0.0022, h=0.010, seg=8, bevel_r=0.0004, bevel_seg=1).rot(0, 0, math.pi / 2).move(0, 0.050, -0.0145)
    return cap + collar.move(0, 0.046, 0) + hinge


def sd_ink():
    """The ink dried in the bore: a cracked crust low in the well."""
    s = cyl(0.0094, h=0.004, seg=16, bevel_r=0.0008).move(0, 0.016, 0)
    return warp_noise(s, 0.0006, 300.0, 4, axis=(0, 1, 0))


auto('Real__secDeskCase', sd_case, 4200, smooth=30.0)
auto('Real__secDeskFronts', sd_fronts, 1400, smooth=30.0)
auto('Real__secDeskLeather', sd_leather, 500, smooth=40.0)
auto('Real__secDeskBrass', sd_brass, 3600, smooth=40.0)
auto('Real__secDeskLettersA', sd_letters_a, 600, smooth=40.0)
auto('Real__secDeskLettersB', sd_letters_b, 500, smooth=40.0)
auto('Real__secDeskSheet', sd_sheet, 140, smooth=50.0)
auto('Real__secDeskInkwell', sd_inkwell, 900, smooth=35.0)
auto('Real__secDeskInkCap', sd_ink_cap, 600, smooth=40.0)
auto('Real__secDeskInk', sd_ink, 200, smooth=50.0, kind='cyl')


# ================================================================ aquarium ===
# A standard 10 gallon tank (t21.notes.md): 514 x 267 x 321 mm over its
# black rims, 3.5 mm glass, black silicone at its corners; on a low
# particleboard cabinet stand (oak-grain laminate, two doors) 550 x 300 x
# 600 mm whose foot has swollen where water ran down it; gravel; a black
# full hood with its lamp; an air line from a stone over the back rim.
AQ_STAND = (0.550, 0.600, 0.300)
AQ_TANK = (0.514, 0.321, 0.267)
AQ_Y0 = 0.600                   # the tank's foot (the stand's top)
AQ_GLASS = 0.0035
AQ_RIM = 0.018
AQ_FLOOR = AQ_Y0 + 0.006        # the glass bottom's top
AQ_IN = (AQ_TANK[0] / 2 - 0.006, AQ_TANK[2] / 2 - 0.006)   # half the inside


def aq_stand():
    w, h, d = AQ_STAND
    parts = []
    for sx in (-1, 1):
        parts.append(rbox(0.016, h - 0.019, d - 0.004, r=0.0012, seg=1).move(sx * (w / 2 - 0.008), (h - 0.019) / 2, -0.002))
    # the toe kick under the doors (set back 40 mm), the bottom, the back
    parts.append(box(w - 0.032, 0.050, 0.016).move(0, 0.025, d / 2 - 0.048))
    parts.append(box(w - 0.032, 0.016, d - 0.040).move(0, 0.058, -0.016))
    back = box(w - 0.020, h - 0.019, 0.004).move(0, (h - 0.019) / 2, -d / 2 + 0.004)
    notch = rbox(0.060, 0.050, 0.02, r=0.012, seg=3).move(-0.205, h - 0.040, -d / 2 + 0.004)
    parts.append(cut(back, notch))
    # two doors, overlaying the sides, a 3 mm gap between them
    dw = (w - 0.003) / 2
    for sx in (-1, 1):
        door = rbox(dw - 0.002, h - 0.019 - 0.054, 0.016, r=0.002, seg=1)
        parts.append(door.move(sx * (dw / 2 + 0.0015), 0.054 + (h - 0.019 - 0.054) / 2, d / 2 - 0.008))
    s = sum(parts, Shape())
    # the foot swollen where water ran down and stood: the board's edges
    # flared and crumbled in the lowest 50 mm
    def swell(p):
        k = 1.0 - smoothstep(0.0, 0.055, p.y)
        return k * (0.6 + 0.4 * noise.noise(Vector((p.x * 30, 2.0, p.z * 30))))
    for i, p in enumerate(s.v):
        k = swell(p)
        if k <= 0:
            continue
        sx = 1 if p.x > 0 else -1
        sz = 1 if p.z > 0 else -1
        if abs(p.x) > w / 2 - 0.02:
            p.x += sx * 0.0035 * k
        if p.z > d / 2 - 0.06 or p.z < -d / 2 + 0.02:
            p.z += sz * 0.0030 * k
        if p.y > 0.002:
            p.y += 0.0012 * k * noise.noise(Vector((p.x * 80, 5.0, p.z * 80)))
    return s


def aq_stand_top():
    """The stand's top: a laminated board, its front edge rounded where the
    laminate wraps it, its edges swollen a little where water stood."""
    w, h, d = AQ_STAND
    top = rbox(w, 0.019, d, r=0.004, seg=2).move(0, h - 0.0095, 0)
    return top


def aq_knobs():
    k = lathe([(0.0, 0.0), (0.0050, 0.0), (0.0045, 0.0040), (0.0060, 0.0080), (0.0130, 0.0150), (0.0130, 0.0210),
               (0.0100, 0.0240), (0.0, 0.0245)], seg=14)
    k.scale(0.9).rot(math.pi / 2, 0, 0)
    w, h, d = AQ_STAND
    return sum((k.copy().move(sx * 0.030, h - 0.110, d / 2) for sx in (-1, 1)), Shape())


def aq_glass():
    w, h, d = AQ_TANK
    g = AQ_GLASS
    y0, y1 = AQ_Y0 + 0.003, AQ_Y0 + h - 0.003
    hy = y1 - y0
    yc = (y0 + y1) / 2
    panes = [box(w - 0.006, hy, g).move(0, yc, d / 2 - 0.003 - g / 2), box(w - 0.006, hy, g).move(0, yc, -d / 2 + 0.003 + g / 2),
             box(g, hy, d - 0.006 - 2 * g).move(w / 2 - 0.003 - g / 2, yc, 0), box(g, hy, d - 0.006 - 2 * g).move(-w / 2 + 0.003 + g / 2, yc, 0),
             box(w - 0.006 - 2 * g, 0.005, d - 0.006 - 2 * g).move(0, AQ_Y0 + 0.0035, 0)]
    return sum(panes, Shape())


def aq_trim():
    w, h, d = AQ_TANK
    parts = []
    # the top rim: an outer skirt, its top, a lip in over the glass
    top = rect_sweep_y([(0.0, 0.0), (0.0, AQ_RIM), (0.002, AQ_RIM + 0.0015), (0.014, AQ_RIM + 0.0015), (0.014, AQ_RIM - 0.002),
                        (0.0045, AQ_RIM - 0.002), (0.0045, 0.0)], w, d)
    parts.append(top.move(0, AQ_Y0 + h - AQ_RIM - 0.0015, 0))
    # the brace across its middle
    parts.append(rbox(0.024, 0.004, d - 0.024, r=0.001, seg=1).move(0, AQ_Y0 + h - 0.002, 0))
    # the bottom frame: a skirt and a ledge the glass stands in
    bot = rect_sweep_y([(0.0, 0.0), (0.0, AQ_RIM), (0.0045, AQ_RIM), (0.0045, 0.006), (0.010, 0.006), (0.010, 0.0)], w, d)
    parts.append(bot.move(0, AQ_Y0, 0))
    # the corner silicone: a bead up each inner vertical corner, round the
    # bottom's inner edge
    ix, iz = w / 2 - 0.003 - AQ_GLASS, d / 2 - 0.003 - AQ_GLASS
    for sx in (-1, 1):
        for sz in (-1, 1):
            b = cyl(0.0028, h=h - 0.030, seg=6, bevel_r=0).move(sx * (ix - 0.0012), AQ_Y0 + h / 2, sz * (iz - 0.0012))
            parts.append(b)
    for sz in (-1, 1):
        parts.append(cyl(0.0028, h=2 * ix, seg=6, bevel_r=0).rot(0, 0, math.pi / 2).move(0, AQ_FLOOR + 0.0008, sz * (iz - 0.0012)))
    for sx in (-1, 1):
        parts.append(cyl(0.0028, h=2 * iz, seg=6, bevel_r=0).rot(math.pi / 2, 0, 0).move(sx * (ix - 0.0012), AQ_FLOOR + 0.0008, 0))
    return sum(parts, Shape())


def rect_sweep_y(profile, w, d):
    """A closed profile [(d_in, y)] swept round a w x d rectangle lying in
    XZ: d_in in from its outer edge; the corners mitred."""
    v, f = [], []
    n = len(profile)
    for (di, y) in profile:
        x, z = w / 2 - di, d / 2 - di
        v += [(-x, y, -z), (x, y, -z), (x, y, z), (-x, y, z)]
    for i in range(n):
        j = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, j * 4 + k2, j * 4 + k))
    return propkit._fix_winding(Shape(v, f))


def aq_gravel():
    """Aquarium gravel, banked up to the back as it was laid: a lumpy bed,
    its top pebbled, from 32 mm deep at the front to 52 at the back."""
    ix, iz = AQ_IN[0] - 0.001, AQ_IN[1] - 0.001
    s = grid_plane(2 * ix, 2 * iz, nx=22, ny=12).rot(-math.pi / 2, 0, 0)
    for p in s.v:
        t = (iz - p.z) / (2 * iz)            # 0 at the front, 1 at the back
        edge = min(ix - abs(p.x), iz - abs(p.z))
        p.y = AQ_FLOOR + 0.032 + 0.020 * t + 0.004 * noise.noise(Vector((p.x * 9, 0.5, p.z * 9)))
        p.y += 0.0016 * noise.noise(Vector((p.x * 160, 1.5, p.z * 160)))
        if edge < 0.004:
            p.y -= 0.002
    # its sides down the glass to the floor (seen through the front)
    lo_y = AQ_FLOOR + 0.0005
    border = []
    nx, ny = 22, 12
    k = nx + 1
    ids = [i for i in range(nx + 1)] + [j * k + nx for j in range(1, ny + 1)] + \
          [ny * k + i for i in range(nx - 1, -1, -1)] + [j * k for j in range(ny - 1, 0, -1)]
    n0 = len(s.v)
    for i in ids:
        p = s.v[i]
        s.v.append(Vector((p.x, lo_y, p.z)))
    m = len(ids)
    for a in range(m):
        b = (a + 1) % m
        s.f.append((ids[a], ids[b], n0 + b, n0 + a))
    s.f.append(tuple(n0 + a for a in range(m)))
    return propkit._fix_winding(s)


def aq_hood():
    """A black full hood: a moulded lid 516 x 270 mm, 62 mm tall at the
    lamp's housing along its back half, the feeding flap at its front
    hinged on a moulded hinge line, a cord notch at its back."""
    w, h, d = AQ_TANK
    hw, hd = w / 2 + 0.001, d / 2 + 0.0015
    y0 = AQ_Y0 + h
    parts = []
    # the skirt round the rim and the low front deck (the flap)
    skirt = rect_sweep_y([(0.0, -0.006), (0.0, 0.020), (0.003, 0.020), (0.003, -0.006)], 2 * hw, 2 * hd)
    parts.append(skirt.move(0, y0, 0))
    flap = rbox(2 * hw - 0.010, 0.004, 0.105, r=0.0015, seg=1).move(0, y0 + 0.020, hd - 0.0575)
    parts.append(flap)
    # the flap's finger tab and the hinge line behind it
    parts.append(rbox(0.060, 0.006, 0.012, r=0.002, seg=1).move(0, y0 + 0.019, hd + 0.004))
    parts.append(cyl(0.004, h=2 * hw - 0.04, seg=8, bevel_r=0).rot(0, 0, math.pi / 2).move(0, y0 + 0.022, hd - 0.111))
    # the lamp's housing: a raised box over the back, its top domed a little
    hs = rbox(2 * hw - 0.008, 0.044, 0.150, r=0.008, seg=2)
    for p in hs.v:
        if p.y > 0.0:
            p.y += 0.004 * (1 - (p.z / 0.075) ** 2)
    parts.append(hs.move(0, y0 + 0.020 + 0.022 - 0.004, -hd + 0.004 + 0.075))
    # vent slots along its top (moulded ribs)
    for i in range(7):
        x = -0.12 + 0.04 * i
        parts.append(rbox(0.026, 0.002, 0.006, r=0.0008, seg=1).move(x, y0 + 0.064, -hd + 0.060))
    # the cord's notch at the back
    s = sum(parts, Shape())
    return s


def aq_tube():
    """The lamp: an 18 in T8 fluorescent tube (25 mm) in its two sockets,
    under the hood's housing."""
    tube_ = cyl(0.0127, h=0.430, seg=14, bevel_r=0.004).rot(0, 0, math.pi / 2)
    return tube_.move(0, AQ_Y0 + AQ_TANK[1] + 0.020, -AQ_TANK[2] / 2 + 0.085)


def aq_airline():
    """3/16 in vinyl air line: from the stone at the back left corner up
    the back glass, over the rim and down the back into the stand's notch."""
    w, h, d = AQ_TANK
    x = -0.212
    zin = -AQ_IN[1] + 0.006
    zout = -d / 2 - 0.004
    ytop = AQ_Y0 + h + 0.012
    pts = [(-0.200, AQ_FLOOR + 0.062, -0.090), (-0.204, AQ_FLOOR + 0.072, zin + 0.014), (x + 0.004, AQ_FLOOR + 0.10, zin),
           (x + 0.002, AQ_Y0 + 0.20, zin), (x, ytop - 0.040, zin), (x, ytop, zin + 0.004), (x, ytop + 0.006, (zin + zout) / 2),
           (x, ytop, zout - 0.002), (x, ytop - 0.040, zout - 0.006), (x + 0.004, AQ_Y0 + 0.15, zout - 0.008),
           (x + 0.002, AQ_Y0 + 0.020, zout - 0.010), (x, AQ_Y0 - 0.010, -AQ_STAND[2] / 2 + 0.010)]
    return tube(pts, 0.0032, seg=7, smooth=2)


def aq_airstone():
    """A cylindrical air stone, 25 x 25 mm, its grit rough."""
    s = cyl(0.0125, h=0.025, seg=14, bevel_r=0.002)
    s = warp_noise(s, 0.0006, 400.0, 2)
    return s.rot(math.pi / 2 - 0.25, 0, 0.3).move(-0.200, AQ_FLOOR + 0.058, -0.078)


auto('Real__aquariumStand', aq_stand, 2200, smooth=30.0)
auto('Real__aquariumStandTop', aq_stand_top, 300, smooth=30.0)
auto('Real__aquariumKnobs', aq_knobs, 400, smooth=45.0)
auto('Real__aquariumGlass', aq_glass, 80, smooth=10.0)
auto('Real__aquariumTrim', aq_trim, 900, smooth=30.0)
auto('Real__aquariumGravel', aq_gravel, 900, smooth=60.0)
auto('Real__aquariumHood', aq_hood, 1100, smooth=35.0)
auto('Real__aquariumTube', aq_tube, 220, smooth=45.0, kind='cyl')
auto('Real__aquariumAirline', aq_airline, 900, smooth=60.0)
auto('Real__aquariumAirstone', aq_airstone, 500, smooth=60.0)


# =========================================================== stained glass ===
# A Victorian leaded panel of the terraced-house kind (t21.notes.md), 300 x
# 440 mm in sight: a border of strips with a square at each corner, the
# field split by a lozenge round a smaller one, each joined to the field's
# edges; in a moulded pine frame (20 mm face) hung on the wall. The cames'
# lines are the same as the game's painter's (`SG_*` in `t21.rs`).
SG_W, SG_H = 0.300, 0.440       # the glass in sight
SG_B = 0.032                    # the border's width
SG_OUT = (0.085, 0.130)         # the outer lozenge's half-width, half-height
SG_INNER = (0.036, 0.055)       # the inner lozenge
SG_FRAME = (0.340, 0.480)       # over the frame
SG_GLASS_Z = 0.002              # the glass's middle (z), the wall at -0.010


def sg_lines():
    """The cames' centre lines, as segments ((x0, y0), (x1, y1))."""
    hw, hh = SG_W / 2, SG_H / 2
    bx, by = hw - SG_B, hh - SG_B
    ox, oy = SG_OUT
    ix, iy = SG_INNER
    L = []
    # the border's inner edge
    L += [((-bx, -by), (bx, -by)), ((bx, -by), (bx, by)), ((bx, by), (-bx, by)), ((-bx, by), (-bx, -by))]
    # the corner squares' outer sides, out to the edge came
    for sx in (-1, 1):
        for sy in (-1, 1):
            L.append(((sx * bx, sy * by), (sx * hw, sy * by)))
            L.append(((sx * bx, sy * by), (sx * bx, sy * hh)))
    # the long sides' strips split at the middle
    for sx in (-1, 1):
        L.append(((sx * bx, 0.0), (sx * hw, 0.0)))
    # the lozenges
    for (ax, ay) in (SG_OUT, SG_INNER):
        L += [((0, ay), (ax, 0)), ((ax, 0), (0, -ay)), ((0, -ay), (-ax, 0)), ((-ax, 0), (0, ay))]
    # the outer lozenge's points to the field's edges, and the inner's to the outer's
    L += [((0, oy), (0, by)), ((0, -oy), (0, -by)), ((ox, 0), (bx, 0)), ((-ox, 0), (-bx, 0))]
    L += [((0, iy), (0, oy)), ((0, -iy), (0, -oy)), ((ix, 0), (ox, 0)), ((-ix, 0), (-ox, 0))]
    return L


def came(a, b, w=0.0060, h=0.0050):
    """A lead came along a segment: an H section seen from each face, its
    leaf rounded, 6 mm across, 5 mm through the glass's plane."""
    a, b = Vector((a[0], a[1], 0)), Vector((b[0], b[1], 0))
    L = (b - a).length
    prof = [(-w / 2, 0.0), (-w / 2 + 0.0012, 0.0018), (-0.0012, 0.0025), (0.0012, 0.0025), (w / 2 - 0.0012, 0.0018), (w / 2, 0.0)]
    # front leaf (z +) and back leaf (z -) closed round
    sec = [(x, z) for x, z in prof] + [(x, -z) for x, z in reversed(prof)][1:-1]
    pts = []
    n = len(sec)
    v, f = [], []
    for end in (0.0, L):
        for (x, z) in sec:
            v.append(Vector((x, end, z)))
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
    f.append(tuple(reversed(range(n))))
    f.append(tuple(range(n, 2 * n)))
    s = propkit._fix_winding(Shape(v, f))
    ang = math.atan2(b.y - a.y, b.x - a.x) - math.pi / 2
    s.rot(0, 0, ang)
    return s.move(a.x, a.y, 0)


def sg_lead():
    parts = [came(a, b) for a, b in sg_lines()]
    # the solder at every joint: a flattened blob over each crossing
    joints = set()
    for a, b in sg_lines():
        joints.add((round(a[0], 5), round(a[1], 5)))
        joints.add((round(b[0], 5), round(b[1], 5)))
    for (x, y) in sorted(joints):
        blob = ellipsoid(0.0058, 0.0058, 0.0017, seg=6).move(x, y, 0.0024)
        parts.append(blob)
    # the edge came round the panel (a U, mostly under the frame's lip)
    edge = rect_sweep([(0.0, -0.0028), (0.0, 0.0028), (0.0045, 0.0028), (0.0055, 0.0018), (0.0055, -0.0018), (0.0045, -0.0028)],
                      SG_W + 0.008, SG_H + 0.008)
    parts.append(edge)
    s = sum(parts, Shape())
    return s.move(0, 0, SG_GLASS_Z)


def sg_panes():
    """The glass: one sheet 3 mm, a little uneven (each piece cut from a
    different sheet, laid at its own slope in the lead)."""
    s = slab(SG_W + 0.006, SG_H + 0.006, 0.003, nx=14, ny=20)
    warp_noise(s, 0.0004, 25.0, 7, axis=(0, 0, 1))
    return s.move(0, 0, SG_GLASS_Z)


def sg_frame():
    """The frame: a pine moulding, 20 mm on its face, 25 mm deep from the
    wall, a bead at its sight edge; the glass in its rebate from behind."""
    W, H = SG_FRAME
    back = -0.009
    prof = [(0.0, back), (0.0, 0.007), (0.002, 0.011), (0.006, 0.0145), (0.011, 0.0160), (0.0155, 0.0150),
            (0.0175, 0.0120), (0.0185, 0.0100), (0.0200, 0.0080), (0.0200, 0.0055), (0.0170, 0.0055), (0.0170, back)]
    return rect_sweep(prof, W, H)


def sg_bar():
    """A saddle bar across the panel's middle on its room side (iron, 10 x
    4 mm), let into the frame at its ends, tied to the lead by two copper
    ties twisted round it."""
    bar = rbox(SG_W + 0.020, 0.010, 0.004, r=0.0012, seg=1).move(0, -0.040, SG_GLASS_Z + 0.0052)
    ties = []
    for x in (-0.085, 0.085):
        t = torus(0.0062, 0.0008, seg=10, tseg=4).rot(0, math.pi / 2, 0).move(x, -0.040, SG_GLASS_Z + 0.0045)
        ties.append(t)
    return bar + sum(ties, Shape())


auto('Real__stainedGlassFrame', sg_frame, 200, smooth=30.0)
auto('Real__stainedGlassPanes', sg_panes, 2000, smooth=30.0)
auto('Real__stainedGlassLead', sg_lead, 4000, smooth=40.0)
auto('Real__stainedGlassBar', sg_bar, 260, smooth=40.0)
