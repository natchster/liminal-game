"""t01: the pictures on the walls, remodelled at their real sizes
(docs/design/PROPS.md, P3): the framed photographs (a snapshot kept at a desk, a
baby's photograph, a graduation portrait, a Victorian family group), the
old portraits in oil, the portrait miniatures, the small landscapes and the
still lifes. Sources and sizes: docs/research/props-remodelling/pictures_on_the_walls.md. What each shows is a real
photograph or painting (CC0, `assets/src/textures/sources.json`,
`pictures`), painted onto its print by the game.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in the
frame of the wall it hangs on: x across its face (its right seen from the
front), y up from the picture's middle, z out of the wall (z = 0 the wall's
face, where the frame's back lies). A frame is a profile swept round its
outline, mitred at the corners: `d` in from the outer edge, `z` out from
the wall. Under each frame's lip (at z = ZL) the stack a framer makes:
glass, then a mat or a slip, the print or canvas, the backing board.

framedPhoto      a 5 x 7 in (127 x 178 mm) desk-and-wall photo frame:
                 FrameWood (a 16 mm moulding, its round top easing down to
                 a bevel at the sight), FrameMetal (a 6.5 mm extruded
                 aluminium section), Glass (2 mm float), Back (3 mm
                 hardboard), Sawtooth (the zinc hanger on its back, shared
                 by the small frames)
babyPhoto        a nursery frame, 200 mm square, a 32 mm rounded moulding
                 (painted by the game): Frame, Mat (1.5 mm card, an oval
                 opening cut with a bevel), Glass, Back
gradPhoto        a 172 x 222 mm frame round a cabinet portrait in a mat:
                 Frame (an 18 mm flat cap moulding with an outer bead),
                 Mat (its window cut with a bevel, the bottom border the
                 wider), Glass, Back, and Tassel (a mortarboard's tassel:
                 its cord looped over the frame's corner, the wrapped head,
                 the strands hanging down the right rail)
familyPortrait   a Victorian walnut ogee frame round a landscape cabinet
                 card (165 x 108 mm): Frame (26 mm, 24 mm deep), Slip (the
                 gilt liner inside it, cove and bead), Glass, Back
oldPortrait      a 17th-century Dutch portrait in oil on a French "figure"
                 No. 6 canvas (33 x 41 cm) in an ebonised ogee frame:
                 Frame (45 mm, its stepped ogee and sight bead), Canvas
                 (the painted face on its stretcher, a little slack, one
                 corner sagging), Stretcher (four bars and their keys),
                 Wire (two screw eyes and the twisted wire between them),
                 Hook (a brass picture hook and its pin, on the wall)
miniature        a portrait miniature in a gilt-brass oval case, 90 x 116
                 mm: Frame (the case, its beaded bezel and its back), Ring
                 (the hanging ring at its top), Glass (the domed crystal),
                 Ivory (the painted oval behind it), Pin (a brass pin with
                 a domed head), Ribbon (a faded silk ribbon from the ring
                 up to a pin), Bow (a ribbon bow)
landscapePainting  an oil sketch on a mahogany panel, French "paysage"
                 No. 3 (27 x 19 cm), in a gilt cove frame 320 x 240 mm:
                 Frame, Panel (cupped a little), Wire
stillLife        a still life on a "figure" No. 3 canvas (27 x 22 cm) in a
                 dark walnut frame with a gilt slip, 340 x 290 mm: Frame,
                 Slip, Canvas, Stretcher, Wire
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


# ------------------------------------------------------------------ kit ---
def clean(profile):
    out = []
    for p in profile:
        if not out or abs(p[0] - out[-1][0]) + abs(p[1] - out[-1][1]) > 1e-7:
            out.append(p)
    if abs(out[0][0] - out[-1][0]) + abs(out[0][1] - out[-1][1]) < 1e-7:
        out.pop()
    return out


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle: d in from
    its outer edge, z out from the wall; the corners mitred."""
    prof = clean(profile)
    v, f = [], []
    n = len(prof)
    for (d, z) in prof:
        x, y = w / 2 - d, h / 2 - d
        v += [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    for i in range(n):
        j = (i + 1) % n
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, j * 4 + k2, j * 4 + k))
    return propkit._fix_winding(Shape(v, f))


def oval_sweep(profile, a, b, seg=64):
    """A closed profile [(d, z)] swept round an a x b ellipse (semi-axes),
    d in along the ellipse's normal, z out from the wall."""
    prof = clean(profile)
    v, f = [], []
    n = len(prof)
    for i, (d, z) in enumerate(prof):
        for k in range(seg):
            t = TAU * k / seg
            c, s = math.cos(t), math.sin(t)
            # the ellipse's normal there
            nx, ny = c / a, s / b
            L = math.hypot(nx, ny)
            nx, ny = nx / L, ny / L
            v.append((a * c - nx * d, b * s - ny * d, z))
    for i in range(n):
        j = (i + 1) % n
        for k in range(seg):
            k2 = (k + 1) % seg
            f.append((i * seg + k, i * seg + k2, j * seg + k2, j * seg + k))
    return propkit._fix_winding(Shape(v, f))


def ell_points(a, b, seg, dx=0.0, dy=0.0):
    return [(dx + a * math.cos(TAU * k / seg), dy + b * math.sin(TAU * k / seg)) for k in range(seg)]


def loft2(lo, hi, z0, z1):
    """A solid between two outlines of as many points: `lo` at z0, `hi` at z1."""
    n = len(lo)
    v = [(x, y, z0) for x, y in lo] + [(x, y, z1) for x, y in hi]
    f = [tuple(reversed(range(n))), tuple(range(n, 2 * n))]
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
    return propkit._fix_winding(Shape(v, f))


def board(w, h, z0, z1, r=0.0008):
    """A board (backing, glass) w x h between z0 and z1, its edges eased."""
    t = z1 - z0
    return rbox(w, h, t, r=min(r, t * 0.45), seg=1).move(0, 0, (z0 + z1) / 2)






def rect_outline(w, h, dy=0.0):
    return [(-w / 2, -h / 2 + dy), (w / 2, -h / 2 + dy), (w / 2, h / 2 + dy), (-w / 2, h / 2 + dy)]




def sawtooth():
    """A zinc sawtooth hanger, 44 x 7 mm, 0.5 mm, its teeth along its foot,
    a tack through each end; lying on the wall (z 0 to 0.0005), its middle
    at the origin."""
    w, h, t = 0.044, 0.0068, 0.0005
    pts = [(-w / 2, h / 2), (w / 2, h / 2), (w / 2, -h / 2 + 0.0015)]
    n = 14
    for i in range(n, -1, -1):
        x = -w / 2 + 0.006 + (w - 0.012) * i / n
        pts.append((x, -h / 2 + (0.0 if i % 2 else 0.0016)))
    pts.append((-w / 2, -h / 2 + 0.0015))
    strip = prism(pts, t).move(0, 0, t / 2)
    tacks = [cyl(0.0012, h=0.0008, seg=6, bevel_r=0).rot(math.pi / 2, 0, 0).move(sx * (w / 2 - 0.003), 0.0, t + 0.0004)
             for sx in (-1, 1)]
    return strip + sum(tacks)


def twisted_wire(points, r=0.0007, seg=4):
    """Picture wire: a strand along the polyline, its lay twisted (the
    strand's section a little flat, turning along it)."""
    return tube(points, r, seg=seg, caps=True, smooth=1)


def screw_eye(at, axis_up=True):
    """A screw eye in a frame's back: its ring standing out of the wood (in
    the yz plane), the shank into it."""
    ring = torus(0.0032, 0.0007, seg=10, tseg=5).rot(0, math.pi / 2, 0)
    shank = cyl(0.0009, h=0.004, seg=5, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, -0.004)
    s = ring.move(0, 0, 0.0040) + shank
    return s.move(*at)


# ======================================================= the framed photo ===
PH_PRINT = (0.127, 0.178)        # 5 x 7 in
PH_ZL = 0.0060                   # the lip's underside: glass beneath it
PH_WOOD = 0.016                  # the wooden moulding's face
PH_METAL = 0.0065                # the aluminium section's face
PH_LIP = 0.004                   # how far the lip lies over the print


def ph_outer(m):
    return (PH_PRINT[0] - 2 * PH_LIP + 2 * m, PH_PRINT[1] - 2 * PH_LIP + 2 * m)


def framed_photo_wood():
    m, zl = PH_WOOD, PH_ZL
    reb = m - PH_LIP
    prof = [(0.0, 0.0), (0.0, 0.0130), (0.0006, 0.0152), (0.0020, 0.0168), (0.0040, 0.0176), (0.0075, 0.0178),
            (0.0105, 0.0172), (0.0122, 0.0160),
            # the bevel down to the sight
            (0.0158, 0.0100), (m, 0.0094), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, *ph_outer(m))


def framed_photo_metal():
    m, zl = PH_METAL, PH_ZL
    prof = [(0.0, 0.0012), (0.0, 0.0170), (0.0004, 0.0178), (0.0012, 0.0181), (0.0054, 0.0181), (0.0061, 0.0177),
            (m, 0.0170), (m, zl), (0.0025, zl), (0.0025, 0.0024), (0.0012, 0.0024), (0.0012, 0.0012)]
    return rect_sweep(prof, *ph_outer(m))


def framed_photo_glass():
    return board(PH_PRINT[0] - 0.001, PH_PRINT[1] - 0.001, PH_ZL - 0.0020, PH_ZL - 0.0001, r=0.0004)


def framed_photo_back():
    return board(PH_PRINT[0] - 0.0012, PH_PRINT[1] - 0.0012, 0.0006, PH_ZL - 0.0024)


# ======================================================== the baby photo ===
BB_W = 0.200                     # overall, square
BB_M = 0.032
BB_ZL = 0.0070
BB_LIP = 0.004
BB_REB = BB_W - 2 * (BB_M - BB_LIP)          # the rebate: mat, glass, back
BB_OVAL = (0.0430, 0.0560, 0.0030)           # the mat's window: semi-axes, raised above the middle


def baby_frame():
    m, zl = BB_M, BB_ZL
    reb = m - BB_LIP
    prof = [(0.0, 0.0), (0.0, 0.0120), (0.0015, 0.0168), (0.0045, 0.0200), (0.0090, 0.0218), (0.0140, 0.0221),
            (0.0190, 0.0212), (0.0232, 0.0190), (0.0262, 0.0160),
            # a bead at the sight
            (0.0278, 0.0150), (0.0294, 0.0148), (0.0308, 0.0138), (0.0316, 0.0122), (m, 0.0104), (m, zl),
            (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, BB_W, BB_W)


def baby_mat():
    a, b, dy = BB_OVAL
    z0, z1 = BB_ZL - 0.0036, BB_ZL - 0.0021
    seg = 56
    back = ell_points(a, b, seg, 0, dy)
    front = ell_points(a + 0.0021, b + 0.0021, seg, 0, dy)
    card = board(BB_REB - 0.001, BB_REB - 0.001, z0, z1, r=0.0003)
    # the cutter: the bevel's slope carried past both faces
    s = 0.0021 / (z1 - z0)
    e = 0.0008
    lo = ell_points(a - s * e, b - s * e, seg, 0, dy)
    hi = ell_points(a + 0.0021 + s * e, b + 0.0021 + s * e, seg, 0, dy)
    return cut(card, loft2(lo, hi, z0 - e, z1 + e))


def baby_glass():
    return board(BB_REB - 0.001, BB_REB - 0.001, BB_ZL - 0.0020, BB_ZL - 0.0001, r=0.0004)


def baby_back():
    return board(BB_REB - 0.0012, BB_REB - 0.0012, 0.0006, BB_ZL - 0.0038)


# ================================================== the graduation photo ===
GR_W, GR_H = 0.172, 0.222
GR_M = 0.018
GR_ZL = 0.0070
GR_LIP = 0.004
GR_REB = (GR_W - 2 * (GR_M - GR_LIP), GR_H - 2 * (GR_M - GR_LIP))
GR_WIN = (0.088, 0.124, 0.004)               # the mat's window (w, h) and how far it is raised


def grad_frame():
    m, zl = GR_M, GR_ZL
    reb = m - GR_LIP
    prof = [(0.0, 0.0), (0.0, 0.0125), (0.0008, 0.0148), (0.0025, 0.0161), (0.0045, 0.0160), (0.0058, 0.0151),
            (0.0066, 0.0141),
            # the flat cap
            (0.0140, 0.0140),
            # a step and a small bead at the sight
            (0.0146, 0.0146), (0.0156, 0.0147), (0.0166, 0.0140), (0.0174, 0.0126), (m, 0.0110), (m, zl),
            (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, GR_W, GR_H)


def grad_mat():
    w, h, dy = GR_WIN
    z0, z1 = GR_ZL - 0.0036, GR_ZL - 0.0021
    e = 0.0008
    s = 0.0021 / (z1 - z0)
    card = board(GR_REB[0] - 0.001, GR_REB[1] - 0.001, z0, z1, r=0.0003)
    lo = rect_outline(w - 2 * s * e, h - 2 * s * e, dy)
    hi = rect_outline(w + 2 * (0.0021 + s * e), h + 2 * (0.0021 + s * e), dy)
    return cut(card, loft2(lo, hi, z0 - e, z1 + e))


def grad_glass():
    return board(GR_REB[0] - 0.001, GR_REB[1] - 0.001, GR_ZL - 0.0020, GR_ZL - 0.0001, r=0.0004)


def grad_back():
    return board(GR_REB[0] - 0.0012, GR_REB[1] - 0.0012, 0.0006, GR_ZL - 0.0038)


def grad_tassel():
    """A mortarboard's tassel (9 in, a 6 in loop) hung by its loop over the
    frame's top right corner: one half of the cord over the top rail, the
    other down the right side, knotted, the wrapped head, the strands down
    the right rail and past the frame's foot."""
    rng = random.Random('gradTassel')
    cx, top = GR_W / 2, GR_H / 2
    front = 0.0162
    knot = Vector((cx - 0.012, top - 0.074, front + 0.0060))
    a = [(cx - 0.034, top + 0.0002, 0.0030), (cx - 0.030, top + 0.0022, front * 0.6), (cx - 0.026, top + 0.0012, front + 0.0016),
         (cx - 0.022, top - 0.010, front + 0.0022), (cx - 0.017, top - 0.040, front + 0.0034), knot]
    b = [(cx + 0.0016, top - 0.010, 0.0040), (cx + 0.0024, top - 0.022, front * 0.7), (cx - 0.003, top - 0.040, front + 0.0030),
         (cx - 0.009, top - 0.062, front + 0.0048), knot]
    cord = tube(a, 0.0011, seg=6, smooth=2) + tube(b, 0.0011, seg=6, smooth=2)
    # the knot, the wrapped head below it (a waisted spool of turns)
    k = sphere(0.0034, seg=10, rings=6).scale(1.0, 0.8, 0.9).move(*knot)
    hy = knot.y - 0.004
    head = lathe([(0.0, 0.0), (0.0040, 0.0006), (0.0046, 0.0040), (0.0040, 0.0110), (0.0048, 0.0170), (0.0052, 0.0190),
                  (0.0046, 0.0204), (0.0, 0.0210)], seg=12)
    # the turns of the wrap: shallow ridges
    for p in head.v:
        t = p.y / 0.021
        if 0.15 < t < 0.85:
            r = math.hypot(p.x, p.z)
            if r > 1e-6:
                g = 1.0 + 0.05 * math.sin(p.y * 2900.0)
                p.x *= g
                p.z *= g
    head.move(0, -0.021, 0).move(knot.x, hy, knot.z)
    # the strands: a bundle of flat threads from the head's foot, hanging
    # straight, splaying a little at their ends, some shorter
    strands = Shape()
    y0 = hy - 0.021 + 0.002
    L = 0.140
    for i in range(30):
        ang = TAU * i / 30 + rng.uniform(-0.1, 0.1)
        r0 = 0.0036 * math.sqrt(rng.uniform(0.2, 1.0))
        x0, z0 = knot.x + r0 * math.cos(ang), knot.z + r0 * math.sin(ang) * 0.8
        ln = L * rng.uniform(0.93, 1.0)
        spread = rng.uniform(0.004, 0.011)
        pts = []
        for j in range(5):
            t = j / 4
            pts.append((x0 + math.cos(ang) * spread * t * t + rng.uniform(-0.0004, 0.0004) * t,
                        y0 - ln * t,
                        max(z0 + math.sin(ang) * spread * 0.6 * t * t, front + 0.0010)))
        strands = strands + tube(pts, 0.00055, seg=3, caps=False)
    return cord + k + head + strands


# ===================================================== the family portrait ===
FP_W, FP_H = 0.220, 0.166
FP_M = 0.026
FP_REB = 0.022                               # the moulding's rebate, in from its outer edge
FP_ZL = 0.0110
FP_SLIP = (FP_W - 2 * FP_REB - 0.001, FP_H - 2 * FP_REB - 0.001)   # the slip's outside
FP_SLIP_W = 0.012                            # the slip's width (4 mm of it under the lip)


def family_frame():
    m, zl, reb = FP_M, FP_ZL, FP_REB
    prof = [(0.0, 0.0), (0.0, 0.0160), (0.0010, 0.0195), (0.0030, 0.0225), (0.0060, 0.0240), (0.0095, 0.0238),
            (0.0125, 0.0225), (0.0150, 0.0200), (0.0165, 0.0180), (0.0185, 0.0172), (0.0210, 0.0168),
            (0.0235, 0.0155), (0.0250, 0.0138), (m, 0.0122), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, FP_W, FP_H)


def family_slip():
    w = FP_SLIP_W
    prof = [(0.0, 0.0025), (0.0, FP_ZL - 0.0021), (0.0045, FP_ZL - 0.0021), (0.0062, FP_ZL - 0.0028),
            (0.0080, 0.0068), (0.0094, 0.0058), (0.0104, 0.0054), (0.0112, 0.0050), (0.0118, 0.0043), (w, 0.0036),
            (w, 0.0025)]
    return rect_sweep(prof, *FP_SLIP)


def family_glass():
    return board(FP_SLIP[0], FP_SLIP[1], FP_ZL - 0.0020, FP_ZL - 0.0001, r=0.0004)


def family_back():
    return board(FP_SLIP[0] - 0.0002, FP_SLIP[1] - 0.0002, 0.0004, 0.0022)


# =========================================================== the old portrait ===
OP_CANVAS = (0.330, 0.410)                   # French "figure" No. 6, 33 x 41 cm
OP_M = 0.045
OP_LIP = 0.004
OP_W, OP_H = OP_CANVAS[0] - 2 * OP_LIP + 2 * OP_M, OP_CANVAS[1] - 2 * OP_LIP + 2 * OP_M
OP_ZL = 0.0230                               # the lip's underside: the canvas's face under it
OP_DEPTH = 0.0360
OP_STR = 0.019                               # the stretcher's depth


def old_frame():
    m, zl, d = OP_M, OP_ZL, OP_DEPTH
    reb = m - OP_LIP
    prof = [(0.0, 0.0), (0.0, 0.0240), (0.0012, 0.0262),
            # the outer fillet and a step
            (0.0035, 0.0270), (0.0045, 0.0285),
            # the big ogee rising to its crest
            (0.0060, 0.0305), (0.0085, 0.0328), (0.0115, 0.0347), (0.0150, 0.0358), (0.0185, d), (0.0215, 0.0356),
            # and falling into the hollow
            (0.0245, 0.0343), (0.0270, 0.0325), (0.0295, 0.0312), (0.0320, 0.0306),
            # a flat, then the sight bead and the lip
            (0.0360, 0.0305), (0.0370, 0.0312), (0.0385, 0.0316), (0.0400, 0.0312), (0.0412, 0.0300), (0.0425, 0.0286),
            (0.0438, 0.0270), (m, 0.0258), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, OP_W, OP_H)


def slack_canvas(w, h, z_face, t, sag_corner, name, nx=14, ny=18):
    """A canvas's face on its stretcher: a slab whose face is a little
    slack (soft waves, deepest away from the bars), one corner sagging."""
    rng = random.Random(name)
    s = slab(w, h, t, nx=nx, ny=ny)
    cxs, cys = sag_corner
    for p in s.v:
        if p.z <= 0:
            continue
        u, v = p.x / (w / 2), p.y / (h / 2)
        # how far from the stretcher bars (0 at the edge, 1 in the middle)
        edge = min(1.0 - abs(u), 1.0 - abs(v))
        k = min(1.0, max(0.0, edge / 0.25))
        wave = math.sin(p.x * 31.0 + 1.3) * math.sin(p.y * 23.0 + 0.4) * 0.0005
        corner = math.exp(-(((u - cxs) / 0.55) ** 2 + ((v - cys) / 0.55) ** 2)) * 0.0016
        p.z -= (wave + corner) * k
    return s.move(0, 0, z_face - t / 2)


def old_canvas():
    return slack_canvas(OP_CANVAS[0], OP_CANVAS[1], OP_ZL - 0.0004, 0.004, (0.7, -0.75), 'oldPortraitCanvas')


def stretcher(w, h, bar, z0, z1, name):
    """A stretcher: four bars mitred at the corners, bevelled away from the
    canvas inside, a key wedged in each corner's slot."""
    rng = random.Random(name)
    t = z1 - z0
    bars = Shape()
    for (bw, bh, x, y) in [(w, bar, 0, h / 2 - bar / 2), (w, bar, 0, -h / 2 + bar / 2), (bar, h - 2 * bar, -w / 2 + bar / 2, 0),
                           (bar, h - 2 * bar, w / 2 - bar / 2, 0)]:
        bars = bars + rbox(bw - 0.001, bh - 0.001, t, r=0.0015, seg=1).move(x, y, (z0 + z1) / 2)
    keys = Shape()
    for sx in (-1, 1):
        for sy in (-1, 1):
            for k in range(2):
                kx = sx * (w / 2 - bar - 0.006) if k == 0 else sx * (w / 2 - bar * 0.5)
                ky = sy * (h / 2 - bar * 0.5) if k == 0 else sy * (h / 2 - bar - 0.006)
                key = prism([(-0.006, -0.012), (0.006, -0.012), (0.004, 0.012), (-0.004, 0.012)], 0.003)
                key.rot(0, 0, (0 if k == 0 else math.pi / 2) + rng.uniform(-0.1, 0.1))
                keys = keys + key.move(kx, ky, z0 - 0.0016)
    return bars + keys


def old_stretcher():
    return stretcher(OP_CANVAS[0] - 0.002, OP_CANVAS[1] - 0.002, 0.045, OP_ZL - 0.0044 - OP_STR, OP_ZL - 0.0044, 'oldPortraitStretcher')


def hanging_wire(w, eye_y, apex_y, z_back, slack=0.008):
    """Two screw eyes in the back (at +-w/2, eye_y) and the wire between
    them drawn up to the hook (apex_y), each end twisted back on itself."""
    eyes = screw_eye((-w / 2, eye_y, z_back)) + screw_eye((w / 2, eye_y, z_back))
    z = z_back + 0.0040
    pts = [(-w / 2, eye_y, z), (-w / 4, (eye_y + apex_y) / 2 - slack * 0.3, z - 0.0005), (0.0, apex_y, z - 0.0012),
           (w / 4, (eye_y + apex_y) / 2 - slack * 0.3, z - 0.0005), (w / 2, eye_y, z)]
    wire = twisted_wire(pts)
    tails = Shape()
    for sx in (-1, 1):
        tails = tails + twisted_wire([(sx * w / 2, eye_y, z), (sx * (w / 2 - 0.012), eye_y - 0.004, z + 0.0006),
                                      (sx * (w / 2 - 0.024), eye_y - 0.002, z + 0.0006)], r=0.0005)
    return eyes + wire + tails


def old_wire():
    # the screw eyes in the stretcher's back a third down, the wire drawn up
    # to 9 cm below the frame's top
    zb = OP_ZL - 0.0044 - OP_STR
    return hanging_wire(OP_CANVAS[0] - 0.050, OP_CANVAS[1] / 2 - 0.135, OP_H / 2 - 0.090, zb)


def picture_hook():
    """A brass picture hook on the wall, its pin through it at 45 degrees:
    the stamped plate (its top folded out over the pin, its foot turned up
    into the cup the wire sits in), its origin where the wire sits."""
    plate = prism(round_outline([(-0.006, -0.002), (0.006, -0.002), (0.0055, 0.022), (-0.0055, 0.022)], 0.002, seg=2), 0.0007)
    plate.move(0, 0.0, 0.00035)
    cup = cyl(0.0050, h=0.012, seg=10, open_ended=True).rot(0, 0, math.pi / 2)
    cup = cut(cup, box(0.02, 0.02, 0.02).move(0, 0.01, 0))
    cup.scale(1, 1, 0.7).move(0, 0.0022, 0.0040)
    lip = rbox(0.011, 0.0008, 0.0045, r=0.0003, seg=1).rot(-0.6, 0, 0).move(0, 0.019, 0.0025)
    pin = cyl(0.0008, h=0.022, seg=6, bevel_r=0).rot(-math.pi / 4, 0, 0).move(0, 0.017, 0.004)
    head = cyl(0.0018, h=0.0008, seg=8, bevel_r=0.0003).rot(-math.pi / 4, 0, 0).move(0, 0.0245, 0.0115)
    return plate + cup + lip + pin + head


# ============================================================= the miniature ===
MN_A, MN_B = 0.045, 0.058                    # the case's outside, semi-axes
MN_SIGHT = 0.011                             # the bezel's width to the sight
MN_REB = 0.008                               # the rebate, in from the outside
MN_ZG = 0.0050                               # the glass's back


def mini_frame():
    seg = 56
    prof = [(0.0, 0.0010), (0.0, 0.0058), (0.0010, 0.0080), (0.0025, 0.0096), (0.0042, 0.0104), (0.0058, 0.0100),
            (0.0068, 0.0092), (0.0078, 0.0094), (0.0090, 0.0097), (0.0100, 0.0093), (0.0107, 0.0084), (MN_SIGHT, 0.0074),
            (MN_SIGHT, 0.0062), (MN_REB, 0.0062), (MN_REB, 0.0010)]

    case = oval_sweep(prof, MN_A, MN_B, seg=seg)
    back = prism(ell_points(MN_A - 0.0004, MN_B - 0.0004, 48), 0.0010).move(0, 0, 0.0005)
    return case + back


def mini_ring():
    """The hanging ring at the case's top, standing up off it in its plane,
    and the collar it turns in."""
    ring = torus(0.0050, 0.0010, seg=16, tseg=6).move(0, MN_B + 0.0068, 0.0050)
    collar = cyl(0.0022, h=0.0040, seg=10, bevel_r=0.0006).move(0, MN_B + 0.0012, 0.0050)
    return ring + collar


def mini_glass():
    """The domed crystal over the painting: flat behind, domed 1.6 mm."""
    seg, rings = 48, 6
    a, b = MN_A - MN_REB - 0.0003, MN_B - MN_REB - 0.0003
    v, f = [], []
    z0, hd = MN_ZG, 0.0016
    for j in range(rings + 1):
        r = 1.0 - j / rings
        for k in range(seg):
            t = TAU * k / seg
            v.append((a * r * math.cos(t), b * r * math.sin(t), z0 + 0.0008 + hd * (1 - r * r)))
    v.append((0.0, 0.0, z0 + 0.0008 + hd))
    for j in range(rings):
        for k in range(seg):
            k2 = (k + 1) % seg
            if j < rings - 1:
                f.append((j * seg + k, j * seg + k2, (j + 1) * seg + k2, (j + 1) * seg + k))
    c = len(v) - 1
    last = (rings - 1) * seg
    for k in range(seg):
        f.append((last + k, last + (k + 1) % seg, c))
    top = Shape(v, f)
    # the flat back and the edge between
    back = [(a * math.cos(TAU * k / seg), b * math.sin(TAU * k / seg), z0) for k in range(seg)]
    n0 = len(top.v)
    top.v += [Vector(p) for p in back]
    top.f.append(tuple(n0 + k for k in reversed(range(seg))))
    for k in range(seg):
        k2 = (k + 1) % seg
        top.f.append((k, n0 + k, n0 + k2, k2))
    return propkit._fix_winding(top)


def mini_ivory():
    """The painted oval (ivory or copper) behind the crystal."""
    a, b = MN_A - MN_REB - 0.0004, MN_B - MN_REB - 0.0004
    return prism(ell_points(a, b, 48), 0.0012).move(0, 0, MN_ZG - 0.0007)


def mini_pin():
    """A brass pin in the wall through the ring: its domed head out."""
    shank = cyl(0.0007, h=0.008, seg=6, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.004)
    head = lathe([(0.0, 0.0), (0.0024, 0.0), (0.0022, 0.0008), (0.0012, 0.0016), (0.0, 0.0018)], seg=10).rot(math.pi / 2, 0, 0)
    head.move(0, 0, 0.0080)
    return shank + head


def mini_ribbon():
    """A silk ribbon, 10 mm, through the ring and up 60 mm to its pin, its
    two halves lying a little apart, a twist in one."""
    w, L = 0.0100, 0.060
    y0 = MN_B + 0.0068
    s = Shape()
    for side in (-1, 1):
        pts = []
        for j in range(9):
            t = j / 8
            x = side * (0.0022 * (1 - t) + 0.0006)
            y = y0 + L * t
            z = 0.0050 * (1 - t) + 0.0015 * t + 0.0006 * math.sin(t * math.pi)
            pts.append((x, y, z))
        # a strip: the ribbon's width across x, turning a little
        v, f = [], []
        for j, (x, y, z) in enumerate(pts):
            ang = side * 0.25 * math.sin(j / 8 * math.pi)
            dx, dz = math.cos(ang) * w / 2, math.sin(ang) * w / 2
            v += [(x - dx * 0.5, y, z - dz), (x + dx * 0.5, y, z + dz)]
        for j in range(8):
            f.append((2 * j, 2 * j + 1, 2 * j + 3, 2 * j + 2))
        strip = Shape(v, f)
        s = s + strip.move(side * w * 0.22, 0, 0)
    # give it a little thickness: a back face
    back = s.copy().move(0, 0, -0.0003)
    back.f = [tuple(reversed(q)) for q in back.f]
    return s + back


def mini_bow():
    """A ribbon bow: two loops and two tails, flat to the wall."""
    loops = Shape()
    for sx in (-1, 1):
        loop = torus(0.0090, 0.0022, seg=14, tseg=5).scale(1.0, 0.55, 0.45).rot(0, 0, sx * 0.25)
        loops = loops + loop.move(sx * 0.0105, 0.002, 0.0030)
    knot = rbox(0.0070, 0.0080, 0.0045, r=0.0018, seg=2).move(0, 0, 0.0035)
    tails = Shape()
    for sx in (-1, 1):
        tail = prism([(-0.0045, 0.0), (0.0045, 0.0), (0.0050 + 0.002 * sx, -0.040), (0.0, -0.034), (-0.0050 + 0.002 * sx, -0.040)], 0.0006)
        tails = tails + tail.rot(0, 0, sx * 0.30).move(sx * 0.003, -0.002, 0.0020)
    return loops + knot + tails


# ======================================================= the landscape ===
LP_PANEL = (0.270, 0.190)                    # French "paysage" No. 3
LP_M = 0.029
LP_LIP = 0.004
LP_W, LP_H = LP_PANEL[0] - 2 * LP_LIP + 2 * LP_M, LP_PANEL[1] - 2 * LP_LIP + 2 * LP_M
LP_ZL = 0.0110
LP_DEPTH = 0.0300


def land_frame():
    m, zl, d = LP_M, LP_ZL, LP_DEPTH
    reb = m - LP_LIP
    prof = [(0.0, 0.0), (0.0, 0.0200), (0.0008, 0.0225),
            # the outer bead
            (0.0022, 0.0250), (0.0040, 0.0262), (0.0058, 0.0258), (0.0068, 0.0247),
            # a step up onto the top and over it
            (0.0078, 0.0268), (0.0095, 0.0290), (0.0115, d), (0.0135, 0.0296),
            # the deep cove down toward the picture
            (0.0160, 0.0278), (0.0185, 0.0252), (0.0210, 0.0226), (0.0232, 0.0206), (0.0250, 0.0196),
            # the sight bead
            (0.0262, 0.0198), (0.0274, 0.0192), (0.0283, 0.0178), (m, 0.0160), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, LP_W, LP_H)


def land_panel():
    """The panel: 6 mm mahogany, its back bevelled to fit the rebate, cupped
    0.6 mm across its grain."""
    w, h = LP_PANEL
    s = slab(w, h, 0.006, nx=10, ny=6)
    for p in s.v:
        u = p.x / (w / 2)
        p.z += 0.0006 * u * u
    return s.move(0, 0, LP_ZL - 0.0030)


def land_wire():
    return hanging_wire(LP_W - 0.070, LP_H / 2 - 0.075, LP_H / 2 - 0.045, 0.0008, slack=0.004)


# ========================================================== the still life ===
SL_W, SL_H = 0.340, 0.290
SL_M = 0.034
SL_REB = 0.030
SL_ZL = 0.0150
SL_DEPTH = 0.0300
SL_SLIP = (SL_W - 2 * SL_REB - 0.001, SL_H - 2 * SL_REB - 0.001)
SL_SLIP_W = 0.0130
SL_CANVAS = (0.270, 0.220)                   # French "figure" No. 3


def still_frame():
    m, zl, d, reb = SL_M, SL_ZL, SL_DEPTH, SL_REB
    prof = [(0.0, 0.0), (0.0, 0.0210), (0.0012, 0.0238), (0.0030, 0.0262), (0.0055, 0.0282), (0.0085, 0.0295),
            (0.0115, d), (0.0145, 0.0297),
            # a scotia (hollow) then a torus round toward the slip
            (0.0170, 0.0284), (0.0190, 0.0272), (0.0210, 0.0268), (0.0230, 0.0272), (0.0252, 0.0270),
            (0.0275, 0.0258), (0.0298, 0.0238), (0.0318, 0.0212), (0.0332, 0.0186), (m, 0.0168), (m, zl),
            (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, SL_W, SL_H)


def still_slip():
    w = SL_SLIP_W
    zt = SL_ZL - 0.0001
    prof = [(0.0, 0.0040), (0.0, zt), (0.0045, zt), (0.0062, zt - 0.0008), (0.0080, zt - 0.0028), (0.0096, zt - 0.0044),
            (0.0108, zt - 0.0050), (0.0118, zt - 0.0058), (0.0125, zt - 0.0068), (w, zt - 0.0080), (w, 0.0040)]
    return rect_sweep(prof, *SL_SLIP)


def still_canvas():
    return slack_canvas(SL_CANVAS[0], SL_CANVAS[1], SL_ZL - 0.0082, 0.004, (-0.75, 0.7), 'stillLifeCanvas', nx=12, ny=10)


def still_stretcher():
    zf = SL_ZL - 0.0082 - 0.004
    return stretcher(SL_CANVAS[0] - 0.002, SL_CANVAS[1] - 0.002, 0.035, zf - 0.016, zf, 'stillLifeStretcher')


def still_wire():
    zb = SL_ZL - 0.0082 - 0.004 - 0.016
    return hanging_wire(SL_CANVAS[0] - 0.050, SL_CANVAS[1] / 2 - 0.075, SL_H / 2 - 0.060, zb, slack=0.004)


# ---------------------------------------------------------------- parts ---
auto('Real__framedPhotoFrameWood', framed_photo_wood, 260)
auto('Real__framedPhotoFrameMetal', framed_photo_metal, 260)
auto('Real__framedPhotoGlass', framed_photo_glass, 60)
auto('Real__framedPhotoBack', framed_photo_back, 60)
auto('Real__framedPhotoSawtooth', sawtooth, 260)
auto('Real__babyPhotoFrame', baby_frame, 300)
auto('Real__babyPhotoMat', baby_mat, 500)
auto('Real__babyPhotoGlass', baby_glass, 60)
auto('Real__babyPhotoBack', baby_back, 60)
auto('Real__gradPhotoFrame', grad_frame, 300)
auto('Real__gradPhotoMat', grad_mat, 200)
auto('Real__gradPhotoGlass', grad_glass, 60)
auto('Real__gradPhotoBack', grad_back, 60)
auto('Real__gradPhotoTassel', grad_tassel, 1500)
auto('Real__familyPortraitFrame', family_frame, 300)
auto('Real__familyPortraitSlip', family_slip, 220)
auto('Real__familyPortraitGlass', family_glass, 60)
auto('Real__familyPortraitBack', family_back, 60)
auto('Real__oldPortraitFrame', old_frame, 500)
auto('Real__oldPortraitCanvas', old_canvas, 1300)
auto('Real__oldPortraitStretcher', old_stretcher, 400)
auto('Real__oldPortraitWire', old_wire, 600)
auto('Real__oldPortraitHook', picture_hook, 400)
auto('Real__miniatureFrame', mini_frame, 2000)
auto('Real__miniatureRing', mini_ring, 400)
auto('Real__miniatureGlass', mini_glass, 700)
auto('Real__miniatureIvory', mini_ivory, 200)
auto('Real__miniaturePin', mini_pin, 120)
auto('Real__miniatureRibbon', mini_ribbon, 120)
auto('Real__miniatureBow', mini_bow, 600)
auto('Real__landscapePaintingFrame', land_frame, 400)
auto('Real__landscapePaintingPanel', land_panel, 400)
auto('Real__landscapePaintingWire', land_wire, 600)
auto('Real__stillLifeFrame', still_frame, 400)
auto('Real__stillLifeSlip', still_slip, 200)
auto('Real__stillLifeCanvas', still_canvas, 700)
auto('Real__stillLifeStretcher', still_stretcher, 400)
auto('Real__stillLifeWire', still_wire, 600)
