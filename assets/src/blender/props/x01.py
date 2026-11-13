"""x01: the barricades (docs/design/PROPS.md 2; docs/research/barricades.md), the
pieces they are built of at their real sizes.

No map file: the game builds each barricade from these parts by name
(`core_props/barricade.rs`), placed as modelled (a scale of one), except a
plywood sheet and a stud, which are cut to the opening (scaled along their
length and width, never their thickness). Each part's frame is given with
it; y is up from the floor unless said.

plywood     a sheet of 18 mm (23/32 in) sheathing plywood, 1.22 x 2.44 m:
            x across, y up, z its thickness (z = 0 its back face); the
            plies show at its edges, its corners knocked, a slight bow
stud        a 2x4 (38 x 89 mm dressed), 2.44 m: x along it, y its 89 mm
            face up, z its 38 mm (z = 0 its back); eased arrises, end grain
boltHead    a carriage bolt's dome head (16 mm bolt) on its washer, its
            face at z = 0, standing out along +z
nut         the nut and washer on the bolt's far end, the same frame
"""
from propkit import *
import propkit

SHEET_W, SHEET_H, PLY_T = 1.22, 2.44, 0.018
STUD_L, STUD_W, STUD_T = 2.44, 0.089, 0.038


def plywood():
    s = rbox(SHEET_W, SHEET_H, PLY_T, r=0.0015, seg=1)
    # the plies: shallow grooves round the edge, five of them
    grooves = []
    for k in range(1, 5):
        z = -PLY_T / 2 + PLY_T * k / 5
        grooves.append(box(SHEET_W + 0.01, 0.0006, 0.0004).move(0, SHEET_H / 2, z))
        grooves.append(box(SHEET_W + 0.01, 0.0006, 0.0004).move(0, -SHEET_H / 2, z))
        grooves.append(box(0.0006, SHEET_H + 0.01, 0.0004).move(SHEET_W / 2, 0, z))
        grooves.append(box(0.0006, SHEET_H + 0.01, 0.0004).move(-SHEET_W / 2, 0, z))
    s = cut(s, *grooves)
    # knocked corners: a chip off two of them
    s = cut(s, box(0.03, 0.03, 0.03).rot(0, 0, 0.6).move(SHEET_W / 2, -SHEET_H / 2, 0),
            box(0.02, 0.025, 0.03).rot(0, 0, 0.4).move(-SHEET_W / 2, SHEET_H / 2, 0))
    # a slight bow: 6 mm out at the middle of its length
    s = s.warp(lambda p: Vector((p.x, p.y, p.z + 0.006 * (1 - (2 * p.y / SHEET_H) ** 2))))
    return s.move(0, SHEET_H / 2, PLY_T / 2)


def stud():
    s = rbox(STUD_L, STUD_W, STUD_T, r=0.003, seg=1)
    # a split and a knot on its face
    s = cut(s, box(0.22, 0.0012, 0.004).rot(0, 0, 0.02).move(0.6, 0.012, STUD_T / 2))
    s = cut(s, cyl(0.012, h=0.004, seg=10, bevel_r=0).rot(math.pi / 2, 0, 0).move(-0.4, -0.01, STUD_T / 2 + 0.001))
    return s.move(0, STUD_W / 2, STUD_T / 2)


def bolt_head():
    washer = cyl(0.019, h=0.003, seg=8, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0015)
    dome = sphere(0.016, seg=8, rings=4)
    dome = cut(dome, box(0.05, 0.05, 0.05).move(0, 0, -0.025)).scale(1, 1, 0.45).move(0, 0, 0.003)
    return washer + dome


def nut():
    washer = cyl(0.019, h=0.003, seg=8, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0015)
    hexa = cyl(0.014, h=0.014, seg=6, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.003 + 0.007)
    thread = cyl(0.008, h=0.02, seg=6, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.02)
    return washer + hexa + thread


def board():
    """A 1x6 board (19 x 140 mm), 2.44 m, rough-sawn: x along it, y its face
    up the doorway, z its thickness (z = 0 its back); a split at one end"""
    s = rbox(2.44, 0.14, 0.019, r=0.002, seg=1)
    s = cut(s, box(0.3, 0.002, 0.03).rot(0, 0, 0.03).move(1.05, 0.02, 0),
            box(0.05, 0.03, 0.03).rot(0, 0, 0.5).move(-1.22, 0.07, 0))
    return s.move(0, 0.07, 0.0095)


def desk():
    """A steel tanker desk, 1.52 x 0.74 x 0.76 m (60 x 30 in, 29 in high):
    two pedestals of drawers (a box and a file drawer each side, a pencil
    drawer between), the modesty panel, the toe kick; its top separate
    (`deskTop`). x across its front, y up, z out of its front (the user's
    side); its top's underside at y = 0.72"""
    w, d, h = 1.52, 0.76, 0.72
    ped_w = 0.40
    s = None
    parts = []
    for sx in (-1, 1):
        cx = sx * (w / 2 - ped_w / 2)
        ped = rbox(ped_w, h - 0.06, d - 0.02, r=0.008, seg=2).move(cx, (h - 0.06) / 2 + 0.06, 0)
        ped = cut(ped, box(ped_w - 0.06, h, 0.02).move(cx, h / 2, d / 2 - 0.004))
        parts.append(ped)
        # its drawers: a box drawer over a file drawer
        for (y0, y1) in ((0.44, 0.66), (0.08, 0.42)):
            fr = rbox(ped_w - 0.07, y1 - y0 - 0.012, 0.02, r=0.006, seg=2).move(cx, (y0 + y1) / 2, d / 2 - 0.004)
            pull = rbox(0.12, 0.018, 0.02, r=0.006, seg=2).move(cx, y1 - 0.05, d / 2 + 0.01)
            parts += [fr, pull]
        # the toe kick, recessed
        parts.append(box(ped_w - 0.02, 0.06, d - 0.08).move(cx, 0.03, -0.02))
    # the modesty panel at the back, the pencil drawer under the top
    parts.append(box(w - 2 * ped_w, 0.5, 0.015).move(0, 0.43, -d / 2 + 0.03))
    parts.append(rbox(w - 2 * ped_w - 0.02, 0.07, d - 0.06, r=0.004, seg=1).move(0, h - 0.045, 0))
    return sum(parts)


def desk_top():
    """The desk's top: laminate on a core, its T-moulded edge; y from 0.72"""
    t = rbox(1.52, 0.032, 0.76, r=0.008, seg=2).move(0, 0.736, 0)
    edge = cut(rbox(1.54, 0.036, 0.78, r=0.012, seg=2), box(1.5, 0.05, 0.74)).move(0, 0.736, 0)
    return t + edge


def chair_frame():
    """A stacking chair's steel frame (0.53 x 0.80 x 0.55 m, seat at 0.45):
    four legs, the seat rails, the back's uprights; its polypropylene shell
    is `chairShell`. x across, y up, z out of its front"""
    legs = []
    for sx in (-1, 1):
        for sz in (-1, 1):
            legs.append(tube([(sx * 0.21, 0.0, sz * 0.22), (sx * 0.2, 0.43, sz * 0.19)], 0.011, seg=8))
    rails = [tube([(-0.2, 0.43, sz * 0.19), (0.2, 0.43, sz * 0.19)], 0.009, seg=6) for sz in (-1, 1)]
    back = [tube([(sx * 0.2, 0.43, -0.19), (sx * 0.19, 0.78, -0.24)], 0.009, seg=6) for sx in (-1, 1)]
    feet = [cyl(0.014, h=0.012, seg=8, bevel_r=0).move(sx * 0.21, 0.006, sz * 0.22) for sx in (-1, 1) for sz in (-1, 1)]
    return sum(legs) + sum(rails) + sum(back) + sum(feet)


def chair_shell():
    seat = rbox(0.44, 0.012, 0.42, r=0.006, seg=2).move(0, 0.448, 0.0)
    seat = seat.warp(lambda p: Vector((p.x, p.y - 0.02 * (1 - (2 * p.x / 0.44) ** 2) * (1 - (2 * p.z / 0.42) ** 2), p.z)))
    back = rbox(0.42, 0.26, 0.012, r=0.02, seg=2).rot(-0.18, 0, 0).move(0, 0.66, -0.225)
    grip = cut(rbox(0.12, 0.03, 0.03, r=0.012, seg=2), box(0.1, 0.02, 0.05)).rot(-0.18, 0, 0).move(0, 0.76, -0.24)
    return seat + back + grip


def pallet():
    """A 48 x 40 in GMA pallet (1.219 x 1.016 x 0.14 m): three notched
    stringers, seven top deck boards, five bottom; x along the stringers
    (1.219), y up, z across (1.016)"""
    L, W = 1.219, 1.016
    parts = []
    for sz in (-1, 0, 1):
        st = rbox(L, 0.089, 0.038, r=0.003, seg=1).move(0, 0.016 + 0.0445, sz * (W / 2 - 0.019))
        notches = [box(0.23, 0.04, 0.06).move(nx * 0.33, 0.016 + 0.02, sz * (W / 2 - 0.019)) for nx in (-1, 1)]
        parts.append(cut(st, *notches))
    for k in range(7):
        x = -L / 2 + 0.07 + k * (L - 0.14) / 6
        parts.append(rbox(0.14, 0.016, W, r=0.002, seg=1).move(x, 0.016 + 0.089 + 0.008, 0))
    for k in range(5):
        x = -L / 2 + 0.07 + k * (L - 0.14) / 4
        parts.append(rbox(0.14, 0.016, W, r=0.002, seg=1).move(x, 0.008, 0))
    return sum(parts)


def shelving():
    """Boltless steel shelving, 0.91 x 1.83 x 0.46 m: four angle posts,
    five levels of beams (its particleboard decks are `shelfDeck`); x
    across, y up, z out of its front"""
    W, H, D = 0.91, 1.83, 0.46
    parts = []
    for sx in (-1, 1):
        for sz in (-1, 1):
            post = cut(box(0.04, H, 0.04), box(0.034, H + 0.01, 0.034).move(-sx * 0.003, 0, -sz * 0.003))
            parts.append(post.move(sx * (W / 2 - 0.02), H / 2, sz * (D / 2 - 0.02)))
    for k in range(5):
        y = 0.06 + k * (H - 0.12) / 4
        for sz in (-1, 1):
            parts.append(box(W - 0.04, 0.035, 0.02).move(0, y, sz * (D / 2 - 0.02)))
        for sx in (-1, 1):
            parts.append(box(0.02, 0.035, D - 0.04).move(sx * (W / 2 - 0.02), y, 0))
    return sum(parts)


def shelf_deck():
    """One particleboard deck, laid on a level's beams; y its top at 0"""
    return rbox(0.87, 0.016, 0.42, r=0.002, seg=1).move(0, -0.008, 0)


def mattress():
    """A twin mattress, 0.97 x 0.2 x 1.91 m: its quilted panels, the
    tape edge round both faces; x across, y its thickness, z its length"""
    W, T, L = 0.97, 0.20, 1.91
    s = rbox(W, T, L, r=0.04, seg=3)
    # quilting: shallow dimples on both faces
    s = subdivide(s, 1)
    s = s.warp(lambda p: Vector((p.x, p.y - math.copysign(0.006, p.y) * (0.5 + 0.5 * math.cos(p.x * 28) * math.cos(p.z * 18)) if abs(p.y) > T / 2 - 0.02 else p.y, p.z)))
    tape = []
    for sy in (-1, 1):
        ring = cut(rbox(W + 0.008, 0.016, L + 0.008, r=0.042, seg=3), rbox(W - 0.03, 0.03, L - 0.03, r=0.03, seg=3))
        tape.append(ring.move(0, sy * (T / 2 - 0.006), 0))
    return (s + sum(tape)).move(0, T / 2, 0)


def door_leaf():
    """A hollow-core interior door, 0.81 x 2.03 x 0.035 m, moulded with six
    panels, the bore for its knob, its hinge mortises; x across, y up, z
    out of its face"""
    W, H, T = 0.81, 2.03, 0.035
    s = rbox(W, H, T, r=0.002, seg=1).move(0, H / 2, 0)
    panels = []
    for (y0, y1) in ((0.12, 0.78), (0.9, 1.5), (1.6, 1.92)):
        for sx in (-1, 1):
            r = rbox(0.27, y1 - y0, 0.01, r=0.01, seg=2).move(sx * 0.17, (y0 + y1) / 2, T / 2)
            panels.append(r)
    s = cut(s, *panels)
    bore = cyl(0.027, h=0.05, seg=12, bevel_r=0).rot(math.pi / 2, 0, 0).move(W / 2 - 0.07, 0.91, 0)
    mortises = [box(0.006, 0.09, 0.03).move(-W / 2, y, 0) for y in (0.25, 1.8)]
    return cut(s, bore, *mortises)


def dresser():
    """A chest of drawers, 0.76 x 0.82 x 0.45 m, veneered chipboard: its
    carcass, four drawers with their pulls; x across, y up, z out of its front"""
    W, H, D = 0.76, 0.82, 0.45
    s = rbox(W, H - 0.06, D, r=0.004, seg=1).move(0, (H - 0.06) / 2 + 0.06, 0)
    plinth = box(W - 0.04, 0.06, D - 0.04).move(0, 0.03, -0.01)
    top = rbox(W + 0.02, 0.022, D + 0.015, r=0.006, seg=2).move(0, H + 0.011 - 0.022, 0.005)
    fronts = []
    for k in range(4):
        y0 = 0.08 + k * (H - 0.12) / 4
        y1 = y0 + (H - 0.12) / 4 - 0.008
        fronts.append(rbox(W - 0.04, y1 - y0, 0.018, r=0.003, seg=1).move(0, (y0 + y1) / 2, D / 2 + 0.009))
        fronts.append(cyl(0.012, h=0.025, seg=8, bevel_r=0.002).rot(math.pi / 2, 0, 0).move(-0.18, (y0 + y1) / 2, D / 2 + 0.03))
        fronts.append(cyl(0.012, h=0.025, seg=8, bevel_r=0.002).rot(math.pi / 2, 0, 0).move(0.18, (y0 + y1) / 2, D / 2 + 0.03))
    return s + plinth + top + sum(fronts)


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one (a sheet and a stud may be cut: scaled along
    their length and width)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


auto('Barricade__plywood', plywood, 900)
auto('Barricade__stud', stud, 300)
auto('Barricade__boltHead', bolt_head, 160)
auto('Barricade__nut', nut, 160)
auto('Barricade__board', board, 200)
auto('Barricade__desk', desk, 1600)
auto('Barricade__deskTop', desk_top, 400)
auto('Barricade__chairFrame', chair_frame, 900)
auto('Barricade__chairShell', chair_shell, 600)
auto('Barricade__pallet', pallet, 1200)
auto('Barricade__shelving', shelving, 1600)
auto('Barricade__shelfDeck', shelf_deck, 60)
auto('Barricade__mattress', mattress, 1800)
auto('Barricade__door', door_leaf, 1200)
auto('Barricade__dresser', dresser, 1400)
