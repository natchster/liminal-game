"""lamps: the rooms' lamps that are not tubes (docs/plan/FACILITY.md F3;
their kinds and colours: `world::lamps`; the research:
docs/research/facility.md, *Light*).

Real fixtures of the mid-century at their real sizes, used as modelled (a
scale of one), each in its own frame: y up, its origin where it hangs
(the high-bay's hook, the bulkhead's canopy on the ceiling), so the game
sets one matrix for the body and the glass that glows.

Lamp__highBay        an enclosed high-bay for a 400 W mercury lamp, as tall
                     halls were lit: the ballast's housing (250 mm across,
                     200 mm high) under the hook, the socket, and a
                     spun-aluminium reflector 560 mm across at its rim,
                     430 mm deep, its rolled lip
Lamp__highBayBulb    what glows: its phosphor-coated BT37 bulb (117 x 292
                     mm) in the reflector, and the prismatic glass
                     refractor closing it, a glass bowl 165 mm deep under
                     the rim, lit over its whole curve
Lamp__bulkhead       a vapour-tight jar fixture on a tunnel's crown: its
                     cast canopy (130 mm across, 70 mm), the threaded guard
                     ring, and the four bars of its guard over the jar
Lamp__bulkheadGlobe  its glass jar, 100 mm across and 160 mm long, its
                     foot rounded: the glass that glows
"""
from propkit import *
import propkit


def auto(name, build, budget, smooth=35.0, flat=False):
    """A part as modelled, at a scale of one: its frame is its own bounds,
    a box's (its faces mapped as a box's are)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, ('box', tuple(lo), tuple(hi)), lambda: shape, smooth=smooth, budget=budget, flat=flat)


# ------------------------------------------------------------- high-bay --
HB_HOUSING = (0.125, 0.20)      # radius, height
HB_TOP = -0.03                  # the housing's top, under the hook
HB_RIM = (0.28, -0.66)          # the reflector's rim: radius, y
BULB = (0.0585, 0.146, -0.48)   # BT37: radius, half length, its middle
_Y0 = HB_TOP - HB_HOUSING[1]
# the reflector's outside, from its neck under the housing to its rim
REFLECTOR = [(0.070, _Y0), (0.085, _Y0 - 0.02), (0.120, _Y0 - 0.07), (0.175, _Y0 - 0.16), (0.225, _Y0 - 0.26),
             (0.258, _Y0 - 0.34), (0.275, _Y0 - 0.40), HB_RIM]


def high_bay():
    r, h = HB_HOUSING
    housing = cyl(r, r, h, seg=28, bevel_r=0.008).move(y=HB_TOP - h / 2)
    hook = torus(0.022, 0.005, seg=16, tseg=6).rot(x=math.pi / 2).move(y=-0.006)
    stem = cyl(0.012, 0.012, 0.03, seg=12).move(y=HB_TOP + 0.012)
    # the reflector: its outside down from the neck to the rim, a rolled
    # lip, its inside back up 3 mm in from it (spun aluminium)
    y0 = HB_TOP - h
    lip = [(HB_RIM[0] + 0.006, HB_RIM[1] - 0.002), (HB_RIM[0] + 0.006, HB_RIM[1] + 0.004)]
    inner = [(rr - 0.003, yy + 0.003) for (rr, yy) in reversed(REFLECTOR)]
    reflector = lathe(REFLECTOR + lip + inner, seg=40, cap_bottom=False, cap_top=False)
    socket = cyl(0.032, 0.028, 0.07, seg=16).move(y=y0 - 0.035 - 0.04)
    return housing + hook + stem + reflector + socket


def high_bay_bulb():
    # the refractor: a prismatic glass bowl hung in the rim, up its outside
    # from its foot on the axis to the rim and in to the axis (closed, as
    # the bulkhead's jar)
    rr, ry = HB_RIM
    rr -= 0.010
    refractor = lathe([(0.0, ry - 0.165), (0.10, ry - 0.160), (0.18, ry - 0.135), (0.235, ry - 0.09),
                       (0.258, ry - 0.045), (rr, ry), (rr - 0.02, ry + 0.012), (0.0, ry + 0.012)], seg=40)
    r, half, mid = BULB
    # an ellipse along y, its neck into the socket at the top
    prof = [(0.0, mid - half)]
    for k in range(1, 12):
        t = math.pi * k / 12
        prof.append((r * math.sin(t), mid - half * math.cos(t)))
    prof += [(0.020, mid + half + 0.004), (0.020, mid + half + 0.03), (0.0, mid + half + 0.03)]
    return refractor + lathe(prof, seg=24)


# ------------------------------------------------------------- bulkhead --
BK_CANOPY = (0.065, 0.07)       # radius, height
BK_JAR = (0.05, 0.16)           # radius, length


def bulkhead():
    r, h = BK_CANOPY
    canopy = cyl(r, r * 0.92, h, seg=24, bevel_r=0.006).move(y=-h / 2)
    ring = torus(r - 0.004, 0.008, seg=24, tseg=8).move(y=-h - 0.006)
    # the guard: four bars from the ring down round the jar to a small ring
    # under its foot
    jr, jl = BK_JAR
    foot = -h - jl - 0.018
    bars = None
    for k in range(4):
        a = math.pi / 4 + k * math.pi / 2
        # (16 mm clear of the jar: closer, the bars and the lit glass fought
        # at a distance, the visual playtest)
        pts = [(r - 0.004, -h - 0.008), (jr + 0.020, -h - 0.05), (jr + 0.020, -h - jl + 0.02), (0.024, foot)]
        b = tube([(rr * math.cos(a), yy, rr * math.sin(a)) for (rr, yy) in pts], 0.0035, seg=6, smooth=2)
        bars = b if bars is None else bars + b
    low = torus(0.022, 0.004, seg=12, tseg=6).move(y=foot)
    return canopy + ring + bars + low


def bulkhead_globe():
    r, l = BK_JAR
    top = -BK_CANOPY[1] - 0.010
    prof = [(0.0, top - l), (r * 0.55, top - l + 0.004), (r * 0.85, top - l + 0.018), (r, top - l + 0.045),
            (r, top - 0.012), (r * 0.9, top), (0.0, top)]
    return lathe(prof, seg=24)


auto('Lamp__highBay', high_bay, 2400)
auto('Lamp__highBayBulb', high_bay_bulb, 1500)
auto('Lamp__bulkhead', bulkhead, 1700)
auto('Lamp__bulkheadGlobe', bulkhead_globe, 400)
