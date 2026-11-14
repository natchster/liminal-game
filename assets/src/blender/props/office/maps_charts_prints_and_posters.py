"""t02: the maps, charts, prints and posters on the walls, remodelled at
their real sizes (docs/design/PROPS.md, P3): a cabin's framed map, a parlour's
antique map, an office's framed map, an anatomical plate hung as a school
chart, a botanical print, a home-front poster and a devotional triptych.
Sources and sizes: docs/research/props-remodelling/maps_charts_prints_and_posters.md. What each shows is a real map, plate or
painting (CC0, `assets/src/textures/sources.json`, `pictures`), painted onto
its paper by the game; the poster's type is set by the game.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in the
frame of the wall it hangs on: x across its face (its right seen from the
front), y up from the picture's middle, z out of the wall (z = 0 the wall's
face, where the frame's back lies). A frame is a profile swept round its
outline, mitred at the corners: `d` in from the outer edge, `z` out from
the wall. Under each frame's lip (at z = ZL) the stack a framer makes:
glass, then a mat or a fillet, the paper, the backing board.

vintageMap     a cabin's map: an etched map of 1590-1703 (28 x 21 cm) in
               a plain pine frame, 16 mm, its outer edge rounded, stained:
               Frame, Glass, Print, Back, Wire (two screw eyes, the wire)
antiqueMap     a parlour's antique map: a hand-coloured atlas map (40 x 34
               cm) under a gilt fillet in a 25 mm cushion moulding: Frame,
               Fillet, Glass, Print, Back, Wire
officeMap      an office's framed map print (27 x 18 cm): FrameWood (a 17 mm
               flat gallery moulding), FrameMetal (a 10 mm aluminium
               section), Glass, Print, Back
anatomyChart   a quarto anatomical plate mounted on linen between two
               turned rails, hung by its cord from a nail: Sheet (the plate,
               slack, its foot curling), Rail (a turned dowel with its
               finials; top and bottom), Cord, Nail
botanicalPrint a hand-coloured botanical engraving in a cream mat, in a
               16 mm half-round moulding (306 x 386 mm): Frame, Mat (its
               window cut with a bevel, the bottom border wider), Glass,
               Paper, Back, Wire
warPoster      a home-front window bill (184 x 264 mm) behind glass in a
               cheap 12 mm frame: Frame, Glass, Paper (buckled), Back
triptych       a Flemish devotional triptych, open: Centre (its panel in
               an engaged frame: a flat carved band and a gilt cove), Wing
               (a wing's panel and frame, hinged at its inner front edge;
               both wings), Hinge (an iron knuckle and its pin)
"""
from propkit import *
import propkit
import random


def auto(name, build, budget, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. A painted panel in its frame is `plane`: its
    picture laid on from the front, so its edges and sides take the
    picture's edge (with `box` its sides would show the whole picture
    squeezed across its depth)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget)


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


def paper(w, h, z_back, t, name, nx=10, ny=10, cockle=0.0002, waves=(0.0, 0.0)):
    """A sheet of paper w x h, its back at z_back, `t` thick, cockled a
    little (soft bumps of `cockle` m), with long waves (amplitude, across
    and along); seeded by its name."""
    rng = random.Random(name)
    ph = [rng.uniform(0, 6.28) for _ in range(4)]
    s = slab(w, h, t, nx=nx, ny=ny)
    for p in s.v:
        u, v = p.x / w, p.y / h
        bump = cockle * (math.sin(u * 17.0 + ph[0]) * math.sin(v * 13.0 + ph[1]) + 0.5 * math.sin(u * 29.0 + v * 21.0 + ph[2]))
        wave = waves[0] * math.sin(u * 6.28 + ph[3]) + waves[1] * math.sin(v * 6.28 + ph[2])
        p.z += bump + wave
    return s.move(0, 0, z_back + t / 2)


def screw_eye(at):
    """A screw eye in a frame's back rail: its ring standing out behind the
    frame (in the yz plane), its shank into the wood."""
    ring = torus(0.0024, 0.0006, seg=10, tseg=5).rot(0, math.pi / 2, 0).move(0, 0, -0.0020)
    shank = cyl(0.0009, h=0.005, seg=5, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0015)
    return (ring + shank).move(*at)


def hanging_wire(xe, eye_y, apex_y, slack=0.008):
    """Two screw eyes in the frame's side rails (at +-xe, eye_y), the
    picture wire between them behind the frame (z < 0: the game leans the
    frame out from the wall at its top), drawn up to the hook (apex_y),
    each end twisted back on itself."""
    eyes = screw_eye((-xe, eye_y, 0.0)) + screw_eye((xe, eye_y, 0.0))
    z = -0.0020
    pts = [(-xe, eye_y, z), (-xe / 2, (eye_y + apex_y) / 2 - slack * 0.3, z - 0.0004), (0.0, apex_y, z - 0.0010),
           (xe / 2, (eye_y + apex_y) / 2 - slack * 0.3, z - 0.0004), (xe, eye_y, z)]
    wire = tube(pts, 0.0007, seg=4, caps=True, smooth=1)
    tails = Shape()
    for sx in (-1, 1):
        tails = tails + tube([(sx * xe, eye_y, z), (sx * (xe - 0.012), eye_y - 0.004, z - 0.0005),
                              (sx * (xe - 0.024), eye_y - 0.002, z - 0.0005)], 0.0005, seg=4, caps=True, smooth=1)
    return eyes + wire + tails


# ====================================================== the cabin's map ===
VM_PRINT = (0.276, 0.208)         # the sheet under the glass: the plate (280 x 210) trimmed to it
VM_M, VM_LIP, VM_ZL = 0.016, 0.004, 0.0075
VM_W, VM_H = VM_PRINT[0] - 2 * VM_LIP + 2 * VM_M, VM_PRINT[1] - 2 * VM_LIP + 2 * VM_M   # 0.300 x 0.232


def vintage_frame():
    """Plain pine, 16 x 19 mm: its outer edge rounded over, its face a
    little crowned, a small bevel down to the sight."""
    m, zl = VM_M, VM_ZL
    reb = m - VM_LIP
    prof = [(0.0, 0.0), (0.0, 0.0140), (0.0007, 0.0161), (0.0020, 0.0177), (0.0040, 0.0186), (0.0075, 0.0189),
            (0.0110, 0.0186), (0.0132, 0.0178), (0.0144, 0.0166),
            # the bevel down to the sight
            (0.0155, 0.0130), (m, 0.0122), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, VM_W, VM_H)


def vintage_glass():
    return board(VM_PRINT[0] - 0.001, VM_PRINT[1] - 0.001, VM_ZL - 0.0020, VM_ZL - 0.0001, r=0.0004)


def vintage_print():
    return paper(VM_PRINT[0] - 0.0012, VM_PRINT[1] - 0.0012, VM_ZL - 0.0029, 0.0004, 'vintageMapPrint', nx=12, ny=9, cockle=0.00010)


def vintage_back():
    return board(VM_PRINT[0] - 0.0012, VM_PRINT[1] - 0.0012, 0.0006, VM_ZL - 0.0036)


def vintage_wire():
    return hanging_wire(VM_W / 2 - VM_M / 2, VM_H / 2 - 0.075, VM_H / 2 - 0.030, slack=0.004)


# ===================================================== the antique map ===
AM_MAP = (0.404, 0.336)           # the sheet; 6 mm of it under the fillet
AM_M, AM_LIP, AM_ZL = 0.025, 0.004, 0.0125
AM_FIL = 0.013                    # the fillet's width, 4 mm of it under the frame's lip
AM_FIL_OUT = (AM_MAP[0] - 2 * 0.006 + 2 * AM_FIL, AM_MAP[1] - 2 * 0.006 + 2 * AM_FIL)   # 0.418 x 0.350
AM_W, AM_H = AM_FIL_OUT[0] - 2 * AM_LIP + 2 * AM_M, AM_FIL_OUT[1] - 2 * AM_LIP + 2 * AM_M   # 0.460 x 0.392
AM_DEPTH = 0.0280


def antique_frame():
    """A 25 mm cushion moulding, 28 deep: a round rising from the outer
    fillet to its crown, a hollow and a small bead before the sight."""
    m, zl, d = AM_M, AM_ZL, AM_DEPTH
    reb = m - AM_LIP
    prof = [(0.0, 0.0), (0.0, 0.0195), (0.0008, 0.0222), (0.0024, 0.0246), (0.0048, 0.0266), (0.0080, 0.0277),
            (0.0110, d), (0.0140, 0.0274), (0.0165, 0.0258),
            # the hollow and the bead
            (0.0180, 0.0240), (0.0192, 0.0232), (0.0204, 0.0233), (0.0216, 0.0238), (0.0228, 0.0232),
            (0.0238, 0.0218), (m, 0.0196), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, AM_W, AM_H)


def antique_fillet():
    """The gilt fillet under the glass, round the map: a cove and a bead,
    its back on the map's margin."""
    w = AM_FIL
    zt = AM_ZL - 0.0021
    zb = 0.0046
    prof = [(0.0, zb), (0.0, zt), (0.0045, zt), (0.0060, zt - 0.0006), (0.0076, zt - 0.0020), (0.0092, zt - 0.0030),
            (0.0104, zt - 0.0031), (0.0116, zt - 0.0036), (0.0124, zt - 0.0044), (w, zt - 0.0052), (w, zb)]
    return rect_sweep(prof, *AM_FIL_OUT)


def antique_glass():
    return board(AM_FIL_OUT[0] - 0.001, AM_FIL_OUT[1] - 0.001, AM_ZL - 0.0020, AM_ZL - 0.0001, r=0.0004)


def antique_print():
    return paper(AM_MAP[0], AM_MAP[1], 0.0039, 0.0004, 'antiqueMapPrint', nx=14, ny=12, cockle=0.00010)


def antique_back():
    return board(AM_FIL_OUT[0] - 0.0012, AM_FIL_OUT[1] - 0.0012, 0.0006, 0.0030)


def antique_wire():
    return hanging_wire(AM_W / 2 - AM_M / 2, AM_H / 2 - 0.130, AM_H / 2 - 0.070, slack=0.006)


# ===================================================== the office's map ===
OM_PRINT = (0.274, 0.178)
OM_LIP = 0.004
OM_WOOD, OM_METAL = 0.017, 0.010
OM_ZL = 0.0065


def om_outer(m):
    return (OM_PRINT[0] - 2 * OM_LIP + 2 * m, OM_PRINT[1] - 2 * OM_LIP + 2 * m)


def office_frame_wood():
    """A flat gallery moulding, 17 x 20 mm: its face flat, eased at the
    outer arris, a step down to a narrow sight bevel."""
    m, zl = OM_WOOD, OM_ZL
    reb = m - OM_LIP
    prof = [(0.0, 0.0), (0.0, 0.0182), (0.0005, 0.0196), (0.0016, 0.0200), (0.0122, 0.0200), (0.0130, 0.0196),
            (0.0134, 0.0184), (0.0142, 0.0180), (0.0158, 0.0176), (m, 0.0168), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, *om_outer(m))


def office_frame_metal():
    """An extruded aluminium section, 10 mm face, 20 deep: the face rounded
    at both edges, the channel behind it where the clips go."""
    m, zl = OM_METAL, OM_ZL
    prof = [(0.0, 0.0012), (0.0, 0.0190), (0.0004, 0.0198), (0.0012, 0.0201), (0.0088, 0.0201), (0.0096, 0.0197),
            (m, 0.0190), (m, zl), (0.0030, zl), (0.0030, 0.0024), (0.0012, 0.0024), (0.0012, 0.0012)]
    return rect_sweep(prof, *om_outer(m))


def office_glass():
    return board(OM_PRINT[0] - 0.001, OM_PRINT[1] - 0.001, OM_ZL - 0.0020, OM_ZL - 0.0001, r=0.0004)


def office_print():
    return paper(OM_PRINT[0] - 0.0012, OM_PRINT[1] - 0.0012, OM_ZL - 0.0028, 0.0004, 'officeMapPrint', nx=12, ny=8, cockle=0.00008)


def office_back():
    return board(OM_PRINT[0] - 0.0012, OM_PRINT[1] - 0.0012, 0.0006, OM_ZL - 0.0035)


# ================================================== the anatomy chart ===
AC_SHEET = (0.215, 0.300)         # the plate on its linen, rail to rail
AC_RAIL_R = 0.0065
AC_RAIL_L = 0.232
AC_RAIL_Y = 0.154                 # the rails' axes, above and below the middle
AC_RAIL_Z = 0.0075                # the rails' axes out from the wall
AC_NAIL_Y = 0.305                 # the nail, above the middle


def anatomy_sheet():
    """The plate mounted on linen, glued round the rails' fronts: hanging
    slack between them, leaning back toward the wall at its middle, waved
    across, its lower corners curling forward a little where the linen has
    pulled; seeded by name."""
    w, h = AC_SHEET
    t = 0.0007
    rng = random.Random('anatomyChartSheet')
    ph = [rng.uniform(0, 6.28) for _ in range(3)]
    s = slab(w, h, t, nx=12, ny=18)
    zf = AC_RAIL_Z + AC_RAIL_R      # the rails' fronts
    for p in s.v:
        u, v = p.x / w, p.y / h     # -0.5..0.5
        # slack: back toward the wall between the rails
        sag = 0.0040 * math.cos(v * math.pi) ** 2
        wave = 0.0007 * math.sin(u * 9.0 + ph[0]) * math.cos(v * math.pi) + 0.0004 * math.sin(v * 15.0 + ph[1]) * math.cos(v * math.pi)
        curl = 0.0035 * max(0.0, -v - 0.30) / 0.20 * (abs(u) * 2) ** 2
        p.z += zf - t / 2 - sag + wave + curl
    return s


def anatomy_rail():
    """A turned beech dowel along x, 13 mm, its two finials: a bead, a
    neck and a small ball; the axis at the origin."""
    L, r = AC_RAIL_L, AC_RAIL_R
    body = L - 2 * 0.012
    prof = [(0.0, -body / 2 - 0.012), (0.0030, -body / 2 - 0.0118), (0.0052, -body / 2 - 0.0108), (0.0062, -body / 2 - 0.0092),
            (0.0058, -body / 2 - 0.0076), (0.0040, -body / 2 - 0.0064), (0.0036, -body / 2 - 0.0050), (0.0058, -body / 2 - 0.0036),
            (0.0074, -body / 2 - 0.0022), (0.0074, -body / 2 - 0.0010), (r, -body / 2),
            (r, body / 2), (0.0074, body / 2 + 0.0010), (0.0074, body / 2 + 0.0022), (0.0058, body / 2 + 0.0036),
            (0.0036, body / 2 + 0.0050), (0.0040, body / 2 + 0.0064), (0.0058, body / 2 + 0.0076), (0.0062, body / 2 + 0.0092),
            (0.0052, body / 2 + 0.0108), (0.0030, body / 2 + 0.0118), (0.0, body / 2 + 0.012)]
    return lathe(prof, seg=12).rot(0, 0, math.pi / 2)


def anatomy_cord():
    """The hanging cord: tied round the top rail inside each finial, drawn
    up to the nail in a V, over it; a knot at each end."""
    y0, z0 = AC_RAIL_Y, AC_RAIL_Z
    xe = AC_RAIL_L / 2 - 0.022
    apex = (0.0, AC_NAIL_Y - 0.0015, 0.0040)
    s = Shape()
    for sx in (-1, 1):
        x = sx * xe
        # a turn round the rail
        loop = torus(AC_RAIL_R + 0.0010, 0.0011, seg=12, tseg=5).rot(0, math.pi / 2, 0).move(x, y0, z0)
        knot = sphere(0.0022, seg=8, rings=5).scale(1.2, 1.0, 1.0).move(x, y0 + AC_RAIL_R + 0.0015, z0 + 0.0020)
        pts = [(x, y0 + AC_RAIL_R + 0.0025, z0 + 0.0020), (x * 0.72, y0 + 0.045, 0.0062), (x * 0.40, y0 + 0.095, 0.0050),
               (x * 0.10, AC_NAIL_Y - 0.008, 0.0044), apex]
        s = s + loop + knot + tube(pts, 0.0011, seg=5, caps=True, smooth=2)
    # over the nail
    s = s + torus(0.0022, 0.0011, seg=10, tseg=5, arc=math.pi).rot(0, 0, 0).move(0.0, AC_NAIL_Y - 0.0018, 0.0040)
    return s


def anatomy_nail():
    """A wire nail in the wall, its head 6 mm, the shank out 7 mm, angled up."""
    shank = cyl(0.0012, h=0.008, seg=6, bevel_r=0).rot(math.pi / 2 - 0.25, 0, 0).move(0, AC_NAIL_Y, 0.0040)
    head = cyl(0.0030, h=0.0008, seg=10, bevel_r=0.0002).rot(math.pi / 2 - 0.25, 0, 0).move(0, AC_NAIL_Y + 0.0010, 0.0080)
    return shank + head


# ================================================= the botanical print ===
BP_W, BP_H = 0.306, 0.386
BP_M, BP_LIP, BP_ZL = 0.016, 0.004, 0.0090
BP_REB = (BP_W - 2 * (BP_M - BP_LIP), BP_H - 2 * (BP_M - BP_LIP))   # 0.282 x 0.362
BP_WIN = (0.196, 0.280, 0.004)    # the mat's window and how far it is raised
BP_PAPER = (0.208, 0.292)


def botanical_frame():
    """A 16 mm half-round with a bead at the sight, 20 deep (maple or
    rosewood, as such prints were framed)."""
    m, zl = BP_M, BP_ZL
    reb = m - BP_LIP
    prof = [(0.0, 0.0), (0.0, 0.0140), (0.0008, 0.0168), (0.0026, 0.0189), (0.0055, 0.0200), (0.0085, 0.0197),
            (0.0110, 0.0183), (0.0126, 0.0166), (0.0136, 0.0161), (0.0146, 0.0156), (0.0154, 0.0145), (m, 0.0130),
            (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, BP_W, BP_H)


def botanical_mat():
    w, h, dy = BP_WIN
    z0, z1 = BP_ZL - 0.0036, BP_ZL - 0.0021
    e = 0.0008
    s = 0.0021 / (z1 - z0)
    card = board(BP_REB[0] - 0.001, BP_REB[1] - 0.001, z0, z1, r=0.0003)
    lo = rect_outline(w - 2 * s * e, h - 2 * s * e, dy)
    hi = rect_outline(w + 2 * (0.0021 + s * e), h + 2 * (0.0021 + s * e), dy)
    return cut(card, loft2(lo, hi, z0 - e, z1 + e))


def botanical_glass():
    return board(BP_REB[0] - 0.001, BP_REB[1] - 0.001, BP_ZL - 0.0020, BP_ZL - 0.0001, r=0.0004)


def botanical_paper():
    z0 = BP_ZL - 0.0036
    return paper(BP_PAPER[0], BP_PAPER[1], z0 - 0.0006, 0.0004, 'botanicalPrintPaper', nx=10, ny=12, cockle=0.00005).move(0, BP_WIN[2], 0)


def botanical_back():
    return board(BP_REB[0] - 0.0012, BP_REB[1] - 0.0012, 0.0006, BP_ZL - 0.0048)


def botanical_wire():
    return hanging_wire(BP_W / 2 - BP_M / 2, BP_H / 2 - 0.120, BP_H / 2 - 0.065, slack=0.005)


# ===================================================== the war poster ===
WP_W, WP_H = 0.200, 0.280
WP_M, WP_LIP, WP_ZL = 0.012, 0.004, 0.0060
WP_PAPER = (WP_W - 2 * (WP_M - WP_LIP), WP_H - 2 * (WP_M - WP_LIP))   # 0.184 x 0.264


def poster_frame():
    """A cheap 12 x 14 mm moulding, its face flat, both edges rounded."""
    m, zl = WP_M, WP_ZL
    reb = m - WP_LIP
    prof = [(0.0, 0.0), (0.0, 0.0118), (0.0008, 0.0134), (0.0024, 0.0140), (0.0092, 0.0140), (0.0106, 0.0134),
            (0.0115, 0.0124), (m, 0.0112), (m, zl), (reb, zl), (reb, 0.0)]
    return rect_sweep(prof, WP_W, WP_H)


def poster_glass():
    return board(WP_PAPER[0] - 0.001, WP_PAPER[1] - 0.001, WP_ZL - 0.0020, WP_ZL - 0.0001, r=0.0004)


def poster_paper():
    """Thin poster paper, buckled where the damp got under the glass."""
    return paper(WP_PAPER[0] - 0.0012, WP_PAPER[1] - 0.0012, WP_ZL - 0.0032, 0.0003, 'warPosterPaper', nx=10, ny=14,
                 cockle=0.0002, waves=(0.00012, 0.00018))


def poster_back():
    return board(WP_PAPER[0] - 0.0012, WP_PAPER[1] - 0.0012, 0.0006, WP_ZL - 0.0042)


# ======================================================== the triptych ===
TR_C = (0.216, 0.280)             # the centre, frame and panel
TR_WING = (0.109, 0.280)          # a wing
TR_CD, TR_WD = 0.028, 0.018       # their depths


def triptych_centre():
    """The centre panel in its engaged frame: a flat band (carved, painted
    and silvered), eased at its outer arris, then a gilt cove falling to
    the painted panel; the panel's face at z = 0.016."""
    w, h = TR_C
    d = TR_CD
    prof = [(0.0, 0.0), (0.0, d - 0.0018), (0.0012, d), (0.0186, d), (0.0198, d - 0.0008),
            # the gilt moulding: a bead, then the cove down to the panel
            (0.0204, d - 0.0020), (0.0214, d - 0.0030), (0.0228, d - 0.0036), (0.0250, d - 0.0052), (0.0275, d - 0.0074),
            (0.0300, d - 0.0094), (0.0322, d - 0.0106), (0.0338, d - 0.0112), (0.0340, 0.0040), (0.0300, 0.0040), (0.0300, 0.0)]
    frame = rect_sweep(prof, w, h)
    panel = board(w - 2 * 0.0330, h - 2 * 0.0330, 0.0040, 0.0160, r=0.0003)
    back = board(w - 0.002, h - 0.002, 0.0, 0.0040, r=0.0006)
    return frame + panel + back


def triptych_wing():
    """A wing: its panel in a narrow frame (a flat band, a gilt cove), 18
    deep; hinged along one long side's front edge."""
    w, h = TR_WING
    d = TR_WD
    prof = [(0.0, 0.0), (0.0, d - 0.0012), (0.0010, d), (0.0072, d), (0.0080, d - 0.0006),
            (0.0086, d - 0.0018), (0.0100, d - 0.0026), (0.0122, d - 0.0042), (0.0146, d - 0.0060), (0.0166, d - 0.0072),
            (0.0174, d - 0.0076), (0.0176, 0.0040), (0.0140, 0.0040), (0.0140, 0.0)]
    frame = rect_sweep(prof, w, h)
    panel = board(w - 2 * 0.0170, h - 2 * 0.0170, 0.0040, d - 0.0080, r=0.0003)
    back = board(w - 0.002, h - 0.002, 0.0, 0.0040, r=0.0006)
    return frame + panel + back


def triptych_hinge():
    """A wrought-iron hinge's knuckle (three of them on a pin), standing
    vertical, its pin's domed heads: the origin on its axis."""
    s = Shape()
    for k in range(3):
        s = s + cyl(0.0024, h=0.0068, seg=8, bevel_r=0.0004).move(0, -0.0072 + k * 0.0072, 0)
    for sy in (-1, 1):
        s = s + sphere(0.0016, seg=8, rings=4).scale(1, 0.6, 1).move(0, sy * 0.0110, 0)
    return s


# ---------------------------------------------------------------- parts ---
auto('Real__vintageMapFrame', vintage_frame, 200)
auto('Real__vintageMapGlass', vintage_glass, 60)
auto('Real__vintageMapPrint', vintage_print, 600)
auto('Real__vintageMapBack', vintage_back, 60)
auto('Real__vintageMapWire', vintage_wire, 600)
auto('Real__antiqueMapFrame', antique_frame, 300)
auto('Real__antiqueMapFillet', antique_fillet, 200)
auto('Real__antiqueMapGlass', antique_glass, 60)
auto('Real__antiqueMapPrint', antique_print, 800)
auto('Real__antiqueMapBack', antique_back, 60)
auto('Real__antiqueMapWire', antique_wire, 600)
auto('Real__officeMapFrameWood', office_frame_wood, 200)
auto('Real__officeMapFrameMetal', office_frame_metal, 200)
auto('Real__officeMapGlass', office_glass, 60)
auto('Real__officeMapPrint', office_print, 500)
auto('Real__officeMapBack', office_back, 60)
auto('Real__anatomyChartSheet', anatomy_sheet, 1100)
auto('Real__anatomyChartRail', anatomy_rail, 600)
auto('Real__anatomyChartCord', anatomy_cord, 900)
auto('Real__anatomyChartNail', anatomy_nail, 160)
auto('Real__botanicalPrintFrame', botanical_frame, 250)
auto('Real__botanicalPrintMat', botanical_mat, 200)
auto('Real__botanicalPrintGlass', botanical_glass, 60)
auto('Real__botanicalPrintPaper', botanical_paper, 600)
auto('Real__botanicalPrintBack', botanical_back, 60)
auto('Real__botanicalPrintWire', botanical_wire, 600)
auto('Real__warPosterFrame', poster_frame, 200)
auto('Real__warPosterGlass', poster_glass, 60)
auto('Real__warPosterPaper', poster_paper, 700)
auto('Real__warPosterBack', poster_back, 60)
auto('Real__triptychCentre', triptych_centre, 300, 'plane')
auto('Real__triptychWing', triptych_wing, 300, 'plane')
auto('Real__triptychHinge', triptych_hinge, 400)
