"""v01: the vending machine (docs/VENDING.md), a glass-front spiral snack
machine of the 1990s-2000s office corridor, at its real size: 1.83 m tall,
0.99 m wide, 0.89 m deep (72 x 39 x 35 in, the AP Snackshop / National 167
class; docs/research/vending-machines.md).

No map file: the game builds the machine from these parts by name
(`core_props/vending/model.rs`); every part is modelled in its own frame at
its real size, so the game places it with a scale of one. The machine's
frame: x across its front (to the viewer's right), y up from the floor, z out
of its front toward the viewer.

cabinet     the painted steel shell: sides, top, back and base, open in front
door        the door over the whole front: the window, the selection
            column's recess and the delivery opening cut through it
glassFrame  the extrusion round the window
liner       the white liner of the product cabinet (back, sides, top, floor)
tray        a product tray: the pan, its side rails, the price channel at
            its front lip and the motor rack at its back
coilChip    a spiral for bags (10 turns), coilCandy one for bars (15),
            coilBottle one for bottles and pastries (7)
motor       a vend motor's housing, coupling forward
panel       the selection column's bezel: the display window, the keypad
            well, the coin entry's and the bill validator's openings
key         one keypad key (21 of them, 3 x 7, A-H J 1-9 * 0 #)
coinEntry   the chrome coin entry with its slot
returnLever the coin return's button
validator   the bill validator's bezel and mouth
coinCup     the coin return cup
bin         the delivery bin behind the flap (its sloped floor)
flap        the delivery door (smoked polycarbonate, hinged at its top)
kick        the kick plate, its louvres
foot        a levelling foot
lock        the T-handle lock's cylinder
tube        the fluorescent tube across the top inside the door that lights
            the products (an F18T8: 24 in, 26 mm)
"""
from propkit import *
import propkit

# ---------------------------------------------------------------- sizes ---
W, H, D = 0.99, 1.83, 0.889      # the cabinet
DOOR_T = 0.075                   # the door's thickness
KICK = 0.10                      # the kick plate under the door
WIN = (-0.455, 0.205, 0.600, 1.735)       # window: x0 x1 y0 y1 (machine frame)
COL = (0.232, 0.470)                      # the selection column: x0 x1
BIN = (-0.430, 0.180, 0.200, 0.470)       # delivery opening: x0 x1 y0 y1
TRAY_W, TRAY_D = 0.672, 0.680


def door_box():
    """The door, centred: its outline, the openings cut through."""
    h = H - KICK
    cy = KICK + h / 2
    s = rbox(W, h, DOOR_T, r=0.007, seg=2)
    x0, x1, y0, y1 = WIN
    win = rbox(x1 - x0, y1 - y0, DOOR_T * 3, r=0.006, seg=1).move((x0 + x1) / 2, (y0 + y1) / 2 - cy, 0)
    bx0, bx1, by0, by1 = BIN
    bn = rbox(bx1 - bx0, by1 - by0, DOOR_T * 3, r=0.010, seg=2).move((bx0 + bx1) / 2, (by0 + by1) / 2 - cy, 0)
    # the selection column's recess: 12 mm deep, the panel sits in it
    c0, c1 = COL
    rec = rbox(c1 - c0, y1 - y0 + 0.02, 0.024, r=0.004, seg=1).move((c0 + c1) / 2, (y0 + y1) / 2 - cy, DOOR_T / 2)
    # a shallow press line framing the window and the column (the door's skin)
    return cut(s, win, bn, rec)


def cabinet():
    """The shell: 1.2 mm steel shown as 18 mm walls, rounded at its front
    vertical edges, open in front (the door covers it). Low on its back,
    two banks of louvres pressed into the sheet: the air of its ballast
    and its fan (seen when it lies on its face)."""
    d = D - DOOR_T
    h = H - 0.02
    s = rbox(W, h, d, r=0.009, seg=2)
    inner = box(W - 0.036, h - 0.036, d).move(0, 0, 0.02)
    s = cut(s, inner)
    # a seam where the top panel meets the sides, 1 mm, near the top
    seam = box(W + 0.01, 0.0015, d + 0.01).move(0, h / 2 - 0.035, 0)
    # the louvres: slots 9 mm tall, 8 mm into the back
    louvres = [box(0.28, 0.009, 0.016).move(sx * 0.19, -h / 2 + 0.14 + k * 0.026, -d / 2) for sx in (-1, 1) for k in range(7)]
    return cut(s, seam, *louvres)


def glass_frame():
    x0, x1, y0, y1 = WIN
    w, h = x1 - x0 + 0.040, y1 - y0 + 0.040
    s = rbox(w, h, 0.024, r=0.005, seg=2)
    return cut(s, rbox(w - 0.040, h - 0.040, 0.1, r=0.003, seg=1))


def liner():
    """The product cabinet's liner: an open-fronted box of thin white sheet."""
    w, h, d = 0.700, 1.200, 0.720
    s = box(w, h, d)
    return cut(s, box(w - 0.012, h - 0.012, d).move(0, 0, 0.008))


def tray():
    """A product tray, its front lip at +z: a pan, rails, a price channel."""
    pan = rbox(TRAY_W, 0.010, TRAY_D, r=0.002, seg=1).move(0, 0.005, 0)
    rails = [rbox(0.012, 0.045, TRAY_D, r=0.002, seg=1).move(s * (TRAY_W / 2 - 0.006), 0.0225, 0) for s in (-1, 1)]
    # the price channel: a lip 26 mm tall, its face tipped back 15 degrees
    chan = rbox(TRAY_W, 0.026, 0.012, r=0.0025, seg=2).rot(x=-0.26).move(0, 0.013, TRAY_D / 2 + 0.004)
    # the motor rack behind the coils
    rack = rbox(TRAY_W - 0.02, 0.070, 0.050, r=0.003, seg=1).move(0, 0.040, -TRAY_D / 2 + 0.025)
    return pan + rails[0] + rails[1] + chan + rack


def coil(radius, turns, wire, length=0.600, seg=16, sides=4):
    """A spiral along +z: the turns from its back (the motor) to its front,
    a short straight leg at the back into the coupling, the front end cut
    off square."""
    pts = [Vector((0.0, 0.0, -length / 2 - 0.012)), Vector((0.0, radius * 0.6, -length / 2 - 0.004))]
    n = turns * seg
    for i in range(n + 1):
        t = i / n
        a = TAU * turns * t
        pts.append(Vector((radius * math.sin(a), radius * math.cos(a), -length / 2 + length * t)))
    return tube(pts, wire, seg=sides, caps=True)


def motor():
    body = rbox(0.046, 0.050, 0.040, r=0.004, seg=2)
    hub = cyl(0.011, 0.011, 0.010, seg=12).rot(x=math.pi / 2).move(0, 0, 0.024)
    return body + hub


def panel():
    """The selection column's bezel (moulded charcoal plastic)."""
    w, h, t = COL[1] - COL[0] - 0.006, WIN[3] - WIN[2] + 0.010, 0.014
    s = rbox(w, h, t, r=0.004, seg=2)
    top = h / 2
    disp = rbox(0.170, 0.052, 0.04, r=0.004, seg=1).move(0, top - 0.070, t / 2)
    keys = rbox(0.092, 0.170, 0.012, r=0.005, seg=1).move(0, top - 0.205, t / 2)
    coin = box(0.072, 0.100, 0.04).move(-0.045, top - 0.445, t / 2)
    bill = box(0.110, 0.080, 0.04).move(0.0, top - 0.585, t / 2)
    cup = rbox(0.100, 0.070, 0.04, r=0.008, seg=1).move(0.0, top - 0.760, t / 2)
    return cut(s, disp, keys, coin, bill, cup)


def key():
    """A keypad key, 17 x 14 mm (the 3 x 7 pad of 15 mm keys)"""
    return rbox(0.017, 0.014, 0.008, r=0.0025, seg=2)


def coin_entry():
    s = rbox(0.068, 0.096, 0.010, r=0.004, seg=2)
    slot = box(0.0035, 0.032, 0.05).move(0, 0.020, 0)
    funnel = rbox(0.022, 0.050, 0.012, r=0.003, seg=1).move(0, 0.020, 0.008)
    return cut(s, slot, funnel)


def return_lever():
    return cyl(0.012, 0.012, 0.010, seg=12, bevel_r=0.0015, bevel_seg=1).rot(x=math.pi / 2)


def validator():
    s = rbox(0.104, 0.074, 0.040, r=0.006, seg=2)
    mouth = rbox(0.078, 0.007, 0.06, r=0.002, seg=1).move(0, 0.006, 0.02)
    lip = rbox(0.090, 0.012, 0.010, r=0.003, seg=1).move(0, -0.008, 0.024)
    return cut(s, mouth) + lip


def coin_cup():
    s = rbox(0.094, 0.064, 0.050, r=0.008, seg=2)
    return cut(s, rbox(0.080, 0.050, 0.05, r=0.006, seg=1).move(0, 0.004, 0.012))


def delivery_bin():
    """The hopper behind the flap: walls and a floor sloping down to the front."""
    w, h, d = BIN[1] - BIN[0] - 0.01, BIN[3] - BIN[2], 0.220
    s = box(w, h, d)
    s = cut(s, box(w - 0.010, h - 0.010, d).move(0, 0.004, 0.006))
    floor = box(w - 0.010, 0.006, d * 1.05).rot(x=-0.32).move(0, -h / 2 + 0.035, 0)
    return s + floor


def flap():
    w, h = BIN[1] - BIN[0] - 0.012, BIN[3] - BIN[2] - 0.012
    s = rbox(w, h, 0.005, r=0.002, seg=1)
    pin = cyl(0.004, 0.004, w + 0.006, seg=8).rot(z=math.pi / 2).move(0, h / 2, -0.002)
    return s + pin


def kick():
    s = rbox(W - 0.03, KICK - 0.02, 0.016, r=0.002, seg=1)
    slots = [rbox(0.100, 0.010, 0.05, r=0.004, seg=1).move(-0.39 + i * 0.13, 0, 0.008) for i in range(7)]
    return cut(s, *slots)


def foot():
    pad = cyl(0.024, 0.026, 0.008, seg=12, bevel_seg=1).move(0, -0.006, 0)
    nut = cyl(0.012, 0.012, 0.010, seg=6, bevel_r=0.001, bevel_seg=1).move(0, 0.003, 0)
    stem = cyl(0.007, 0.007, 0.010, seg=8, bevel_r=0).move(0, 0.010, 0)
    return pad + nut + stem


def lock():
    barrel = cyl(0.016, 0.016, 0.018, seg=16, bevel_r=0.002, bevel_seg=1).rot(x=math.pi / 2)
    slot = box(0.0025, 0.012, 0.03).move(0, 0, 0.01)
    return cut(barrel, slot)


def fl_tube():
    glass = cyl(0.013, 0.013, 0.580, seg=10, bevel_r=0.003, bevel_seg=1)
    caps = [cyl(0.0135, 0.0135, 0.018, seg=10, bevel_r=0.001, bevel_seg=1).move(0, s * 0.29, 0) for s in (-1, 1)]
    return glass + caps[0] + caps[1]


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one (`vending::model`)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


auto('Vending__cabinet', cabinet, 900)
auto('Vending__door', door_box, 900)
auto('Vending__glassFrame', glass_frame, 500)
auto('Vending__liner', liner, 200)
auto('Vending__tray', tray, 700)
auto('Vending__coilChip', lambda: coil(0.050, 10, 0.0025), 1400)
auto('Vending__coilCandy', lambda: coil(0.032, 15, 0.0022, seg=10), 1300)
auto('Vending__coilBottle', lambda: coil(0.050, 7, 0.0026), 900)
auto('Vending__motor', motor, 300)
auto('Vending__panel', panel, 700)
auto('Vending__key', key, 120)
auto('Vending__coinEntry', coin_entry, 300)
auto('Vending__returnLever', return_lever, 160)
auto('Vending__validator', validator, 500)
auto('Vending__coinCup', coin_cup, 400)
auto('Vending__bin', delivery_bin, 300)
auto('Vending__flap', flap, 200)
auto('Vending__kick', kick, 400)
auto('Vending__foot', foot, 260)
auto('Vending__lock', lock, 300)
auto('Vending__tube', fl_tube, 260)
