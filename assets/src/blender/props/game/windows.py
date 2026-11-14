"""windows (was w01): the windows of the rooms (docs/design/WINDOWS.md), at their real sizes
(docs/research/windows-real.md), and the streetlamp outside them.

No map file: the game builds each window from these parts by name
(`core_props/window/model.rs`), placed as modelled (a scale of one, but the
stool, stretched across the wall's depth). The window's frame: x across the
opening (to the right seen from the room), y up from the opening's sill,
z out of the wall toward the room, from the back of the frame (z = 0, the
outside) forward.

A wood double-hung sash window (halls, living quarters, parlours, party
rooms), its rough opening 0.91 x 1.37 m (36 x 54 in):

dhFrame     the frame: jambs, head and sloped sill, 116 mm deep, with the
            blind stop outside, the parting bead between the sashes and the
            interior stop
dhUpper1/2  the upper sash (outboard), one light or two (a muntin, 19 mm)
dhLower1/2  the lower sash (inboard), its deep bottom rail
dhLock      the crescent lock on the meeting rails
dhLift      a sash lift on the lower sash's bottom rail
dhCasing    the casing round the opening on the wall face (legs and head,
            70 mm, mitred), z centred on its own thickness
stool       the stool: the board the window sits on, 1 m deep (the game
            stretches it across the wall's depth), its nose at z = 0
apron       the apron under the stool, on the wall face

An office strip window (office wings), fixed lights 1.2 m wide and 1.3 m
tall between aluminium mullions (50 x 114 mm), over a convector:

stMullion   a mullion, its glazing pockets
stRail      a head or sill rail between two mullions (1.15 m)
convector   the steel convector cabinet under one light (1.2 m), its grille
            on top and its louvred kick

A mini blind (1-inch aluminium slats), 1 m wide (the game scales it across):

blindHead   the steel headrail, 38 x 25 mm
blindSlat   one slat, 25 mm, crowned
blindBottom the C-shaped bottom rail and its end caps
blindWand   the clear hexagonal tilt wand, 0.6 m

The streetlamp outside (a steel pole, a cobra head):

lampPole    the tapered pole on its base, 5 m, the arm reaching 1.5 m
lampHead    the cobra head's housing
lampLens    its sag lens (lit)
"""
from propkit import *
import propkit

# ---------------------------------------------------------------- sizes ---
DH_W, DH_H, DH_D = 0.91, 1.37, 0.116      # rough opening, frame depth
JAMB, HEAD, SILL = 0.022, 0.022, 0.040     # frame members' faces
SASH_T = 0.035                             # sash thickness
UP_Z, LO_Z = (0.019, 0.054), (0.061, 0.096)   # the sashes' channels
STILE, TOP_RAIL, MEET, BOT_RAIL, MUNTIN = 0.050, 0.050, 0.035, 0.070, 0.019
CASE_W, CASE_T, CASE_REVEAL = 0.070, 0.019, 0.006

ST_W, ST_H, ST_D, MULL = 1.2, 1.3, 0.114, 0.050

BLIND_SLAT = 0.025


def dh_frame():
    w, h, d = DH_W, DH_H, DH_D
    iw = w - 2 * JAMB
    parts = []
    # jambs between the sill and the head
    for sx in (-1, 1):
        parts.append(rbox(JAMB, h - HEAD - SILL, d, r=0.0015, seg=1).move(sx * (w - JAMB) / 2, SILL + (h - HEAD - SILL) / 2, d / 2))
    parts.append(rbox(w, HEAD, d, r=0.0015, seg=1).move(0, h - HEAD / 2, d / 2))
    # the sill, sloped to shed water outward: thicker at the back
    sill = prism([(0, 0), (d, 0), (d, SILL * 0.75), (0, SILL)], w).rot(0, math.pi / 2, 0)
    lo, hi = sill.bounds()
    parts.append(sill.move(-(lo.x + hi.x) / 2, 0, -lo.z))
    # stops round the inside of the frame: the blind stop outside, the
    # parting bead between the sashes, the interior stop
    ih = h - HEAD - SILL
    for z0, z1, depth in ((0.0, 0.019, 0.016), (0.054, 0.061, 0.012), (0.096, 0.116, 0.012)):
        dz = z1 - z0
        for sx in (-1, 1):
            parts.append(rbox(depth, ih, dz, r=0.002, seg=1).move(sx * (iw / 2 - depth / 2), SILL + ih / 2, (z0 + z1) / 2))
        parts.append(rbox(iw - 2 * depth, depth, dz, r=0.002, seg=1).move(0, h - HEAD - depth / 2, (z0 + z1) / 2))
    return sum(parts[1:], parts[0])


def sash(width, height, top, bottom, lights, z):
    """A sash `width` x `height`, its rails `top` and `bottom` deep, `lights`
    side by side, centred at z; its outline on the frame's inside."""
    t = SASH_T
    parts = []
    for sx in (-1, 1):
        parts.append(rbox(STILE, height, t, r=0.003, seg=2).move(sx * (width - STILE) / 2, height / 2, 0))
    parts.append(rbox(width - 2 * STILE, top, t, r=0.003, seg=2).move(0, height - top / 2, 0))
    parts.append(rbox(width - 2 * STILE, bottom, t, r=0.003, seg=2).move(0, bottom / 2, 0))
    if lights == 2:
        parts.append(rbox(MUNTIN, height - top - bottom, t * 0.8, r=0.003, seg=2).move(0, bottom + (height - top - bottom) / 2, 0))
    s = sum(parts[1:], parts[0])
    return s.move(0, 0, z)


def sash_sizes():
    iw = DH_W - 2 * JAMB
    ih = DH_H - HEAD - SILL
    half = ih / 2
    lap = MEET / 2
    up = (half + lap, SILL + half - lap)        # height, its foot
    lo = (half + lap, SILL)
    return iw, up, lo


def dh_upper(lights):
    iw, (h, y0), _ = sash_sizes()
    return sash(iw, h, TOP_RAIL, MEET, lights, sum(UP_Z) / 2).move(0, y0, 0)


def dh_lower(lights):
    iw, _, (h, y0) = sash_sizes()
    return sash(iw, h, MEET, BOT_RAIL, lights, sum(LO_Z) / 2).move(0, y0, 0)


def dh_lock():
    """the crescent lock: its base on the lower sash's meeting rail, the cam
    turned over the keeper on the upper one"""
    _, (hu, yu), (hl, yl) = sash_sizes()
    y = yl + hl - MEET / 2
    base = rbox(0.060, 0.012, 0.018, r=0.003, seg=2).move(0, 0, 0)
    cam = cyl(0.022, 0.022, 0.006, seg=12, bevel_r=0.0015, bevel_seg=1).rot(math.pi / 2, 0, 0).move(0, 0.009, -0.004)
    cam = cut(cam, box(0.06, 0.03, 0.06).move(0, 0, 0.03 - 0.004))
    knob = cyl(0.004, 0.003, 0.016, seg=6).rot(0, 0, 0.6).move(0.024, 0.016, -0.004)
    keeper = rbox(0.050, 0.010, 0.014, r=0.002, seg=1).move(0, 0.002, -0.026)
    return (base + cam + knob + keeper).move(0, y + 0.006, LO_Z[1] - 0.008)


def dh_lift():
    """a sash lift: a bent strap screwed to the bottom rail"""
    hook = tube([(-0.035, 0, 0), (-0.03, 0, 0.012), (0.03, 0, 0.012), (0.035, 0, 0)], 0.003, seg=6, smooth=1)
    plates = [rbox(0.012, 0.022, 0.003, r=0.001, seg=1).move(sx * 0.036, -0.004, 0) for sx in (-1, 1)]
    return (hook + plates[0] + plates[1]).move(0, SILL + 0.045, LO_Z[1] + 0.0015)


def dh_casing():
    """legs and head round the opening, mitred, on the wall face"""
    w, h = DH_W, DH_H
    x_in = w / 2 + CASE_REVEAL
    x_out = x_in + CASE_W
    top_in = h + CASE_REVEAL
    top_out = top_in + CASE_W
    parts = []
    for sx in (-1, 1):
        leg = prism([(x_in, 0), (x_out, 0), (x_out, top_out), (x_in, top_in)], CASE_T)
        if sx < 0:
            leg.scale(-1, 1, 1)
        parts.append(bevel(leg, 0.003, seg=2))
    head = prism([(-x_in, top_in), (x_in, top_in), (x_out, top_out), (-x_out, top_out)], CASE_T)
    parts.append(bevel(head, 0.003, seg=2))
    # a bead run down the casing's face: a shallow groove
    s = sum(parts[1:], parts[0])
    return s


def stool(width):
    """1 m deep, its rounded nose at z = 0 (the room), its back at z = -1"""
    t = 0.022
    board = rbox(width, t, 1.0, r=0.004, seg=2).move(0, t / 2, -0.5)
    return board


def apron(width):
    return rbox(width, 0.070, 0.016, r=0.0025, seg=2).move(0, -0.035, 0)


def st_mullion():
    """50 mm face, 114 mm deep, the glazing pockets either side for the
    insulating unit at mid depth"""
    m = rbox(MULL, ST_H, ST_D, r=0.002, seg=1).move(0, ST_H / 2, ST_D / 2)
    pockets = [box(0.012, ST_H + 0.1, 0.027).move(sx * (MULL / 2), ST_H / 2, ST_D / 2) for sx in (-1, 1)]
    # the snap-in glazing bead's line, inside
    beads = [box(0.002, ST_H + 0.1, 0.004).move(sx * (MULL / 2 - 0.008), ST_H / 2, ST_D - 0.002) for sx in (-1, 1)]
    return cut(m, *pockets, *beads)


def st_rail():
    w = ST_W - MULL
    r = rbox(w, MULL, ST_D, r=0.002, seg=1).move(0, MULL / 2, ST_D / 2)
    pocket = box(w + 0.1, 0.012, 0.027).move(0, MULL, ST_D / 2)
    return cut(r, pocket)


def convector():
    """the steel cabinet under a light: 1.2 m long, 0.80 m high, 0.22 m deep;
    a slotted grille on top, a louvred kick at its foot, its seams"""
    w, h, d = ST_W - 0.004, 0.80, 0.22
    c = rbox(w, h, d, r=0.004, seg=2).move(0, h / 2, d / 2)
    slots = [box(0.010, 0.04, d * 0.62).move(-w / 2 + 0.05 + k * 0.024, h, d * 0.5) for k in range(int((w - 0.1) / 0.024))]
    kick = [box(w * 0.86, 0.008, 0.02).move(0, 0.05 + k * 0.016, d) for k in range(4)]
    seam = box(w + 0.1, 0.003, 0.004).move(0, h - 0.10, d)
    return cut(c, *slots, *kick, seam)


def blind_head():
    h = rbox(1.0, 0.025, 0.038, r=0.003, seg=2).move(0, -0.0125, 0)
    return cut(h, box(1.1, 0.012, 0.03).move(0, -0.025, 0))


def blind_slat():
    """a 25 mm slat, 1 m long, crowned (its middle 1.5 mm proud)"""
    s = slab(1.0, BLIND_SLAT, 0.0006, nx=1, ny=3).rot(-math.pi / 2, 0, 0)
    return s.warp(lambda p: (p.x, p.y + 0.0015 * (1 - (2 * p.z / BLIND_SLAT) ** 2), p.z))


def blind_bottom():
    b = rbox(1.0, 0.010, 0.026, r=0.003, seg=2)
    caps = [rbox(0.006, 0.012, 0.028, r=0.002, seg=1).move(sx * 0.5, 0, 0) for sx in (-1, 1)]
    return b + caps[0] + caps[1]


def blind_wand():
    return cyl(0.0035, 0.0035, 0.6, seg=6, bevel_r=0.0008, bevel_seg=1).move(0, -0.3, 0)


def lamp_pole():
    """a tapered steel pole on its base plate, 5 m, its arm reaching 1.5 m
    out at the top"""
    shaft = lathe([(0.090, 0.0), (0.090, 0.02), (0.075, 0.025), (0.070, 0.30), (0.055, 5.0), (0.0, 5.0)], seg=12)
    plate = rbox(0.32, 0.02, 0.32, r=0.004, seg=1).move(0, 0.01, 0)
    arm = tube([(0, 4.8, 0), (0, 5.02, 0.25), (0, 5.08, 0.8), (0, 5.05, 1.5)], 0.035, seg=8, smooth=2)
    return shaft + plate + arm


def lamp_head():
    """the cobra head at the arm's end, its housing over the lens"""
    hood = ellipsoid(0.17, 0.08, 0.36, seg=16).move(0, 0, 0)
    hood = cut(hood, box(0.5, 0.2, 0.9).move(0, -0.1, 0))
    rim = rbox(0.30, 0.03, 0.62, r=0.01, seg=2).move(0, -0.005, 0)
    return (hood + rim).move(0, 5.0, 1.6)


def lamp_lens():
    lens = ellipsoid(0.13, 0.06, 0.27, seg=14)
    lens = cut(lens, box(0.4, 0.2, 0.7).move(0, 0.1, 0))
    return lens.move(0, 4.99, 1.6)


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one (`window::model`)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


CASE_OUT = DH_W + 2 * (CASE_REVEAL + CASE_W)

auto('Window__dhFrame', dh_frame, 900)
auto('Window__dhUpper1', lambda: dh_upper(1), 500)
auto('Window__dhUpper2', lambda: dh_upper(2), 600)
auto('Window__dhLower1', lambda: dh_lower(1), 500)
auto('Window__dhLower2', lambda: dh_lower(2), 600)
auto('Window__dhLock', dh_lock, 400)
auto('Window__dhLift', dh_lift, 240)
auto('Window__dhCasing', dh_casing, 300)
auto('Window__stool', lambda: stool(CASE_OUT + 0.05), 120)
auto('Window__apron', lambda: apron(CASE_OUT), 120)
auto('Window__stMullion', st_mullion, 300)
auto('Window__stRail', st_rail, 200)
auto('Window__convector', convector, 1300)
auto('Window__blindHead', blind_head, 200)
auto('Window__blindSlat', blind_slat, 40)
auto('Window__blindBottom', blind_bottom, 200)
auto('Window__blindWand', blind_wand, 80)
auto('Window__lampPole', lamp_pole, 700)
auto('Window__lampHead', lamp_head, 600)
auto('Window__lampLens', lamp_lens, 400)
