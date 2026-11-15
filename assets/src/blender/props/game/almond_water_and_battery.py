"""almond_water_and_battery: the two things every player picks up, made by
the asset pipeline (issue #13; their dossiers: assets/src/props/almond_water/,
assets/src/props/battery/, with every measurement's source).

Real things at their real sizes, used as modelled (a scale of one), each
part centred in its cylinder frame (axis +Y), so a part's texture wraps
round it (`assets::props::uv_for`). The game stacks them
(`core_props::pickups`).

AlmondWater__bottle       a plain 500 ml still-water PET bottle with nothing
                          printed on it (the canon: VENDING.md): 65 mm
                          across, 203 mm to the top of its PCO-1881 28 mm
                          neck; a five-footed still-water base round its
                          gate, four grip bands low on the body, a plain
                          panel, five hoop ribs, a sloped shoulder, the
                          neck's support ring and thread
AlmondWater__liquidFull   what is in it: up into the shoulder (every bottle
                          is full: LORE.md, Survival)
AlmondWater__cap          the HDPE screw cap, 31.5 mm across, its skirt knurled
AlmondWater__tamperRing   its tamper band: on the cap of a sealed bottle, left
                          on the neck of an opened one
Battery__jacket           an alkaline D cell (LR20): 33.2 mm across, 61 mm
                          long; the printed film over the steel can, rolled
                          over both shoulders (its print painted in the game)
Battery__positive         the nickel-plated positive cover and its nub
                          (9 mm across, standing 1.8 mm)
Battery__negative         the negative end: the flat nickel disc in the
                          crimped rim
Battery__crystals         what an old cell leaks: white crystals round the
                          negative end's seal
"""
from propkit import *
import propkit

# the bottle (metres; y up from the base, then centred)
B_R = 0.0325          # body radius (65 mm)
B_H = 0.203           # base to the top of the neck finish
B_GROOVE = 0.0306     # a grip band's groove
B_RIB = 0.0313        # between two hoop ribs
LIQUID_TOP = 0.165    # the water's top: up into the shoulder
CAP_R, CAP_H, BAND_H = 0.01575, 0.0122, 0.0032


def bottle_profile():
    p = [(0.0, 0.0050), (0.0040, 0.0052), (0.0060, 0.0046), (0.0120, 0.0030), (0.0200, 0.0008),
         (0.0250, 0.0), (0.0300, 0.0012), (0.0318, 0.0040), (B_R, 0.0090)]
    # four grip bands low on the body
    for c in (0.022, 0.036, 0.050, 0.064):
        p += [(B_R, c - 0.0045), (B_GROOVE, c), (B_R, c + 0.0045)]
    p += [(B_R, 0.080), (B_R, 0.118)]
    # five hoop ribs high on it
    for k in range(5):
        y = 0.122 + k * 0.0056
        p += [(B_R, y), (B_RIB, y + 0.0028)]
    p += [(B_R, 0.150), (0.0318, 0.156), (0.0300, 0.162), (0.0272, 0.168), (0.0236, 0.173), (0.0196, 0.177),
          (0.0160, 0.180), (0.0137, 0.183), (0.0131, 0.186),
          # the support ring
          (0.0165, 0.1865), (0.0165, 0.1880), (0.0130, 0.1885), (0.0126, 0.1890)]
    # the thread (PCO-1881: 2.7 mm a turn)
    for k in range(4):
        y = 0.190 + k * 0.0030
        p += [(0.0124, y), (0.0137, y + 0.0015)]
    p += [(0.0124, 0.2010), (0.0122, B_H - 0.0004), (0.0108, B_H), (0.0, B_H)]
    return p


def star_base(p):
    # a still-water bottle's five feet: the base pressed in between them
    if p.y < 0.006:
        a = math.atan2(p.x, p.z)
        k = 0.5 + 0.5 * math.cos(5 * a)
        r = math.hypot(p.x, p.z)
        if r > 0.008:
            f = 0.93 + 0.07 * k
            p.x *= f
            p.z *= f
            p.y += (1.0 - k) * 0.0012 * min(1.0, r / 0.025)
    return p


def bottle():
    return lathe(bottle_profile(), seg=32).warp(star_base).move(y=-B_H / 2)


def liquid(top):
    # the bottle's inside (0.5 mm in from its wall), flat at the line it is
    # filled to
    p = [(r - 0.0005, y) for r, y in bottle_profile() if 0.006 <= y <= top and r > 0]
    r_top = p[-1][0]
    return lathe([(0.0, 0.006)] + p + [(r_top, top), (0.0, top)], seg=32).warp(star_base).move(y=-top / 2 - 0.003)


def cap():
    prof = [(0.0, -CAP_H / 2), (0.0150, -CAP_H / 2), (CAP_R, -CAP_H / 2 + 0.0008), (CAP_R, CAP_H / 2 - 0.0014),
            (0.0153, CAP_H / 2 - 0.0003), (0.0138, CAP_H / 2), (0.0, CAP_H / 2)]
    s = lathe(prof, seg=96)

    def knurl(p):
        r = math.hypot(p.x, p.z)
        if r > CAP_R - 0.0004 and -CAP_H / 2 + 0.001 < p.y < CAP_H / 2 - 0.0016:
            a = math.atan2(p.x, p.z)
            if round(a / (TAU / 96)) % 2:
                p.x *= 0.972
                p.z *= 0.972
        return p
    return s.warp(knurl)


def tamper_ring():
    prof = [(0.0131, -BAND_H / 2), (0.0158, -BAND_H / 2), (0.0158, BAND_H / 2), (0.0131, BAND_H / 2)]
    return lathe(prof + [prof[0]], seg=48, cap_bottom=False, cap_top=False)


# the D cell (metres; centred along y, the negative end down)
D_R, D_JACKET = 0.0166, 0.0585


def jacket():
    y0, y1 = -D_JACKET / 2, D_JACKET / 2
    prof = [(0.0, y0 + 0.0006), (0.0140, y0 + 0.0006), (0.0148, y0 + 0.0002), (0.0158, y0 + 0.0003),
            (0.0165, y0 + 0.0012), (D_R, y0 + 0.0020), (D_R, y1 - 0.0020), (0.0165, y1 - 0.0012),
            (0.0156, y1 - 0.0002), (0.0146, y1), (0.0140, y1 - 0.0004), (0.0, y1 - 0.0004)]
    return lathe(prof, seg=40)


def positive():
    prof = [(0.0, 0.0), (0.0142, 0.0), (0.0140, 0.0002), (0.0060, 0.0004), (0.0047, 0.0006),
            (0.0045, 0.0019), (0.0042, 0.0022), (0.0, 0.0022)]
    return lathe(prof, seg=32).move(y=-0.0011)


def negative():
    prof = [(0.0, 0.0004), (0.0095, 0.0004), (0.0100, 0.0002), (0.0105, 0.0), (0.0135, 0.0002),
            (0.0150, 0.0006), (0.0155, 0.0010), (0.0150, 0.0013), (0.0, 0.0013)]
    return lathe(prof, seg=32).move(y=-0.00065)


def crystals():
    # a crust of potassium carbonate round the seal, lumpy and uneven
    s = torus(0.0128, 0.0016, seg=28, tseg=6).rot(x=math.pi / 2)
    s = noisy(s, 0.0012, scale=420.0, seed=3)
    return noisy(s, 0.0005, scale=1300.0, seed=8)


def auto(name, build, budget, smooth=40.0):
    """A part as modelled: its frame its own bounds (a cylinder's, so its
    texture wraps), used at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    r = max(abs(lo[0]), abs(hi[0]), abs(lo[2]), abs(hi[2]))
    part(name, ('cyl', (-r, lo[1], -r), (r, hi[1], r)), lambda: shape, smooth=smooth, budget=budget)


auto('AlmondWater__bottle', bottle, 4400, smooth=50.0)
auto('AlmondWater__liquidFull', lambda: liquid(LIQUID_TOP), 4000, smooth=50.0)
auto('AlmondWater__cap', cap, 1400, smooth=30.0)
auto('AlmondWater__tamperRing', tamper_ring, 400)
auto('Battery__jacket', jacket, 1200, smooth=50.0)
auto('Battery__positive', positive, 600)
auto('Battery__negative', negative, 600)
auto('Battery__crystals', crystals, 1200, smooth=60.0)
