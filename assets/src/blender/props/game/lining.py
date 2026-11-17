"""lining: the things of the Utility Halls' sprayed lining and the runs on
their crown (docs/plan/FACILITY.md F5; where they go: `world::lining`; the
research: docs/research/facility.md, *Materials and decay*).

Real things at their real sizes, used as modelled (a scale of one) unless
a part says what scales it, each in its own frame.

Lining__rockBolt   a rock bolt's end where it stands out of a sprayed
                   lining, as bolts are set in a pattern: a domed square
                   face plate 150 mm across, a hardened washer, a hex nut
                   (32 mm across its flats) and the bolt's threaded end;
                   the lining's face at y = 0, the bolt along +y out of it
Run__tray          a ladder cable tray a metre long, along +x from x = 0:
                   two side rails (C-channels 92 mm tall, a 3 in tray's),
                   305 mm apart, their bottoms at y = 0, and rungs every
                   305 mm (12 in)
Run__cables        the cables lying in it: eight PVC-sheathed cables 12 to
                   32 mm across on its rungs, a metre long like the tray
Run__strut         the trapeze a run hangs from: a strut channel (41 mm
                   square, 1 m along z, centred) and its two hex nuts;
                   scaled along z to the run's width, its top at y = 0
Run__pipe          a pipe's length: a tube of radius 1 along +x from x = 0 to
                   1, its ends open (scaled by its length along, its outside
                   radius across; its ends are in its elbows or the crown)
Run__clamp         a pipe clamp round a pipe of radius 1 along x (scaled by
                   the pipe's radius): its strap over the pipe, the two
                   feet bolted down at y = -1
Run__coupling      a grooved coupling round a pipe of radius 1 along x
                   (scaled by its radius): the two half housings and their
                   bolt pads (a 4 in coupling 156 mm across, 226 over its
                   pads, 54 along the pipe; a 2 in one a little fuller)
Run__elbow         a long-radius elbow of a pipe of radius 1 (scaled by its
                   radius): from along +x at the origin it turns up, its
                   other end along +y, 2.6 radii above and beyond (a
                   long-radius elbow's centre to end, 1.5 times its size)
"""
from propkit import *
import propkit


def auto(name, build, budget, smooth=35.0, flat=False):
    """A part as modelled: its frame is its own bounds, a box's."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, ('box', tuple(lo), tuple(hi)), lambda: shape, smooth=smooth, budget=budget, flat=flat)


# ---------------------------------------------------------------- the bolt --
def rock_bolt():
    plate = box(0.15, 0.006, 0.15).move(y=0.003)
    # the plate is pressed into a dome round the hole
    dome = cyl(0.040, 0.058, 0.010, seg=12, bevel_r=0).move(y=0.006 + 0.005)
    washer = cyl(0.026, 0.026, 0.004, seg=12, bevel_r=0).move(y=0.016 + 0.002)
    # a hex nut: six sides, 32 mm across the flats (37 across its corners)
    nut = cyl(0.0185, 0.0185, 0.024, seg=6, bevel_r=0).move(y=0.020 + 0.012)
    thread = cyl(0.011, 0.011, 0.028, seg=8, bevel_r=0).move(y=0.044 + 0.010)
    return plate + dome + washer + nut + thread


# ---------------------------------------------------------------- the tray --
TRAY_W = 0.305       # between the rails' webs (a 12 in tray)
RAIL = (0.092, 0.022, 0.0025)   # height, flange, thickness (a 3 in tray's)


def tray():
    d, f, t = RAIL
    rails = None
    for s in (-1, 1):
        z = s * TRAY_W / 2
        web = box(1.0, d, t).move(x=0.5, y=d / 2, z=z)
        # the flanges turn inward, top and bottom
        top = box(1.0, t, f).move(x=0.5, y=d - t / 2, z=z - s * f / 2)
        bot = box(1.0, t, f).move(x=0.5, y=t / 2, z=z - s * f / 2)
        r = web + top + bot
        rails = r if rails is None else rails + r
    rungs = None
    for k in range(4):
        x = 0.15 + 0.305 * k
        if x > 1.0:
            break
        # a rung: a small channel across, on the bottom flanges
        g = box(0.025, 0.012, TRAY_W).move(x=x, y=t + 0.006)
        rungs = g if rungs is None else rungs + g
    return rails + rungs


def cables():
    # eight cables side by side on the rungs, the bigger ones at the bottom
    out = None
    sizes = [0.032, 0.026, 0.020, 0.032, 0.016, 0.024, 0.012, 0.028]
    z = -TRAY_W / 2 + 0.02
    y0 = RAIL[2] + 0.012
    for i, dia in enumerate(sizes):
        r = dia / 2
        if z + r > TRAY_W / 2 - 0.01:
            break
        c = cyl(r, r, 1.0, seg=8, bevel_r=0).rot(z=math.pi / 2).move(x=0.5, y=y0 + r + (0.004 if i % 3 == 2 else 0.0), z=z + r)
        out = c if out is None else out + c
        z += dia + 0.004
    return out


# --------------------------------------------------------------- the strut --
def strut():
    # a 41 mm strut channel, its open side up, 1 m long along z
    s, t = 0.041, 0.0025
    base = box(s, t, 1.0).move(y=-s + t / 2)
    sides = box(t, s, 1.0).move(x=-s / 2 + t / 2, y=-s / 2) + box(t, s, 1.0).move(x=s / 2 - t / 2, y=-s / 2)
    lips = box(0.008, t, 1.0).move(x=-s / 2 + 0.004, y=-t / 2) + box(0.008, t, 1.0).move(x=s / 2 - 0.004, y=-t / 2)
    # the nuts under it where the rods come through, 40 mm in from its ends
    nuts = cyl(0.009, 0.009, 0.008, seg=6, bevel_r=0).move(y=-s - 0.004, z=-0.46) + cyl(0.009, 0.009, 0.008, seg=6, bevel_r=0).move(y=-s - 0.004, z=0.46)
    return base + sides + lips + nuts


# ---------------------------------------------------------------- the pipe --
def pipe():
    return cyl(1.0, 1.0, 1.0, seg=14, open_ended=True).rot(z=-math.pi / 2).move(x=0.5)


def clamp():
    # a strap over the pipe (a half ring, 0.12 radii thick) and its two feet
    strap = torus(1.07, 0.06, seg=20, tseg=6, arc=math.pi).rot(y=math.pi / 2)
    feet = box(0.10, 0.04, 0.45).move(y=-0.02, z=-1.25) + box(0.10, 0.04, 0.45).move(y=-0.02, z=1.25)
    # the strap's ends down to its feet
    legs = box(0.10, 1.0, 0.12).move(y=-0.5, z=-1.07) + box(0.10, 1.0, 0.12).move(y=-0.5, z=1.07)
    return strap.move(y=0.0) + feet.move(y=-1.0) + legs


def coupling():
    # two half housings round the pipe, bolted through lugs at either side
    ring = cyl(1.4, 1.4, 0.95, seg=20, bevel_r=0).rot(z=math.pi / 2)
    lugs = box(0.60, 0.35, 0.70).move(z=1.65) + box(0.60, 0.35, 0.70).move(z=-1.65)
    bolts = cyl(0.12, 0.12, 1.1, seg=6, bevel_r=0).move(z=1.70) + cyl(0.12, 0.12, 1.1, seg=6, bevel_r=0).move(z=-1.70)
    return ring + lugs + bolts


def elbow():
    # a quarter of a torus, bend radius 2.6: from (0, 0, 0) heading +x to
    # (2.6, 2.6, 0) heading +y
    # (a torus's arc runs from +x round toward +y about z: turned back a
    # quarter, it runs from straight below its centre to beside it)
    e = torus(2.6, 1.0, seg=12, tseg=16, arc=math.pi / 2)
    return e.rot(z=-math.pi / 2).move(y=2.6)


auto('Lining__rockBolt', rock_bolt, 200)
auto('Run__tray', tray, 300)
auto('Run__cables', cables, 300)
auto('Run__strut', strut, 200)
auto('Run__pipe', pipe, 60)
auto('Run__clamp', clamp, 300)
auto('Run__coupling', coupling, 300)
auto('Run__elbow', elbow, 500)
