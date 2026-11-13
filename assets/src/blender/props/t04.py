"""t04: the washroom's fittings and what was left in it, and two things on
its walls (Props P3, docs/design/PROPS.md): a soap-smudged washroom mirror, a
frosted window, a wall faucet whose basin is gone, a towel bar with its
towel, a trigger spray bottle, a hair dryer dropped on the floor and a fire
alarm's strobe. Sources and sizes: t04.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is modelled at
its real size in its prop's own frame: x across its front, y up, z out of
its front; a wall thing's wall is the plane z = 0, a floor thing's floor the
plane y = 0. Every part is named `Real__<prop><Part>`:

smudgeMirrorFrame     a 16 x 20 in stainless channel-frame washroom mirror
                      (the Bradley 781 class): the one-piece channel, 13 mm
                      face, mitred, its hem round the glass, two locking
                      screws under it; y = 0 the mirror's middle
frostedWindowFrame    a small four-light casement (650 x 550 mm over its
                      architrave), all its painted softwood: the moulded
                      architrave on three sides, the window board and the
                      apron under it, the lining, the sash with its rails,
                      stiles and glazing bars, the ovolo beads round each
                      pane; y = 0 the window's middle
frostedWindowIron     its cockspur fastener and striker, black iron
sinkFaucetBody        a wall-mounted two-valve faucet on 4 in centres (the
                      Chicago Faucets 445 class): the wall flanges, the
                      supply arms, the valve bodies and bonnets, the cross
                      body, the swivel nut; y = 0 the arms' axis
sinkFaucetSpout       its L-type swing spout (8 in reach) and aerator, as
                      modelled straight out; the game swings it about
                      SPOUT_AXIS
sinkFaucetHandles     its two four-arm handles
sinkFaucetIndexHot    the red index button on the hot (left) handle
sinkFaucetIndexCold   the blue one on the cold (right) handle
sinkFaucetHanger      the steel hanger the basin hung on, left on the wall
                      with its four screws
sinkFaucetWaste       the basin's waste: the chromed tubular wall bend's
                      stub, its slip nut and escutcheon, cut open
towelBarMetal         a 12 in towel bar (chromed): the 19 mm bar, its two
                      posts on their rosettes; y = 0 the bar's axis
towelBarTowel         a cotton terry hand towel (41 x 66 cm) folded in half
                      along its length and hung over the bar: draped by
                      Blender's cloth solver
sprayBottleBody       a 16 oz HDPE cylinder round (62 x 200 mm, 28/400 neck),
                      standing on y = 0 (a cylinder's frame: its print wraps
                      round it)
sprayBottleHead       its trigger sprayer: the ribbed closure, the shroud
                      over the pump, the trigger
sprayBottleNozzle     the sprayer's turning nozzle cap
hairdryer*            an 1875 W pistol-grip hair dryer lying on its side on
                      the floor as it fell: Body (the barrel and the
                      handle, its hang loop), Grille (the intake and the
                      outlet), Switches, Strain (the cord's strain relief);
                      Plug (a moulded two-pin plug lying flat, its cord
                      along -x) and Prongs
strobe*               a fire alarm's wall strobe (the Edwards Genesis
                      G1RF-VM class, 70 x 114 x 21 mm): the red Housing, the
                      clear prismatic Lens, the Reflector and the xenon
                      Tube behind it; y = 0 its middle
"""
from propkit import *
import propkit
import random
import zlib


# ------------------------------------------------------------------ util --
def auto(name, build, budget, smooth=35.0, kind='box', flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth, flat=flat)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def rect_sweep(profile, w, h):
    """A closed profile [(d, z)] swept round a w x h rectangle centred on
    the origin: d in from its outer edge, z out from the wall; the corners
    mitred."""
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


def clean_profile(prof):
    out = []
    for p in prof:
        if not out or abs(p[0] - out[-1][0]) + abs(p[1] - out[-1][1]) > 1e-6:
            out.append(p)
    return out


def loft(sections, cap0=True, cap1=True):
    """Join rings of points (each the same count, in the same turn) into a
    closed tube; the ends capped by a fan to their middle."""
    n = len(sections[0])
    v, f = [], []
    for s in sections:
        v += [Vector(p) for p in s]
    for i in range(len(sections) - 1):
        for k in range(n):
            k2 = (k + 1) % n
            f.append((i * n + k, i * n + k2, (i + 1) * n + k2, (i + 1) * n + k))
    if cap0:
        c = sum((Vector(p) for p in sections[0]), Vector((0, 0, 0))) / n
        v.append(c)
        ci = len(v) - 1
        for k in range(n):
            f.append((ci, (k + 1) % n, k))
    if cap1:
        c = sum((Vector(p) for p in sections[-1]), Vector((0, 0, 0))) / n
        v.append(c)
        ci = len(v) - 1
        base = (len(sections) - 1) * n
        for k in range(n):
            f.append((ci, base + k, base + (k + 1) % n))
    return propkit._fix_winding(Shape(v, f))


def superellipse(a, b, n=16, e=2.6):
    """Points round a superellipse of half-axes a, b (in its own u, v)."""
    pts = []
    for i in range(n):
        t = TAU * i / n
        c, s = math.cos(t), math.sin(t)
        pts.append((a * math.copysign(abs(c) ** (2 / e), c), b * math.copysign(abs(s) ** (2 / e), s)))
    return pts


def along_z(s):
    """A shape built round +y (a lathe, a cylinder) turned to stand out of
    the wall along +z (its +y end out)."""
    return s.rot(math.pi / 2, 0, 0)


def along_x(s):
    """A shape built round +y turned to lie along +x (its +y end to +x)."""
    return s.rot(0, 0, -math.pi / 2)


def hexagon(r, h, seg=6):
    return cyl(r, r, h, seg=seg, bevel_r=min(0.0006, h * 0.2), bevel_seg=1)


# ======================================================= smudged mirror ===
# Bradley 781 channel-frame mirror, 16 x 20 in (406 x 508 mm): a one-piece
# roll-formed stainless channel, mitred, 1/4 in glass; hung on a concealed
# wall hanger, locked by two screws under its bottom rail (t04.notes.md).
SM_W, SM_H = 0.406, 0.508
SM_FACE = 0.013                # the channel's face
SM_PROJ = 0.019                # out from the wall
SM_BACK = 0.006                # the hanger behind the frame
SM_SHEET = 0.0009
SM_GLASS_FRONT = SM_PROJ - 0.0018


def smudge_mirror_frame():
    r1, r2 = 0.0016, 0.0008
    lip_back = SM_GLASS_FRONT
    prof = [(0.0, SM_BACK)]
    prof += [(r1 - r1 * math.cos(a), SM_PROJ - r1 + r1 * math.sin(a)) for a in (0.0, math.pi / 4, math.pi / 2)]
    prof += [(SM_FACE - r2, SM_PROJ)]
    prof += [(SM_FACE - r2 + r2 * math.sin(a), SM_PROJ - r2 + r2 * math.cos(a)) for a in (math.pi / 2, math.pi)]
    prof += [(SM_FACE - r2 - 0.0004, lip_back)]
    prof += [(SM_SHEET, lip_back), (SM_SHEET, SM_BACK + SM_SHEET), (0.0095, SM_BACK + SM_SHEET), (0.0095, SM_BACK)]
    s = rect_sweep(clean_profile(prof), SM_W, SM_H)
    for x in (-0.13, 0.13):
        s = s + screw_head(r=0.0024, h=0.0011).rot(math.pi, 0, 0).move(x, -SM_H / 2, SM_BACK + 0.0065)
    return s


auto('Real__smudgeMirrorFrame', smudge_mirror_frame, 420)


# ======================================================= frosted window ===
# A small side-hung casement of four lights, painted softwood, in a room's
# wall (650 x 550 mm over its architrave and window board): the architrave
# 40 x 19 mm on three sides, the window board (22 mm, its nose rounded) and
# an apron under it; inside them the lining, and the sash (stiles and top
# rail 45 mm, bottom rail 55 mm, glazing bars 20 mm) with an ovolo bead
# round each pane of 4 mm acid-etched glass (the game's).
FW_W, FW_H = 0.650, 0.550
FW_ARC = 0.040                       # the architrave's width
FW_OPEN = (-0.285, 0.285, -0.213, 0.235)   # inside the architraves: x0, x1, y0 (the board's top), y1
FW_SASH = (-0.273, 0.273, -0.201, 0.223)
FW_STILE, FW_TOP, FW_BOT, FW_BAR = 0.045, 0.045, 0.055, 0.020
FW_SASH_Z = 0.014                    # the sash's face off the wall
FW_GLASS_Z = 0.0055                  # the glass (the game's panes)


def fw_panes():
    """The four panes' rectangles (x0, x1, y0, y1), bottom left first."""
    x0, x1, y0, y1 = FW_SASH
    pw = (x1 - x0 - 2 * FW_STILE - FW_BAR) / 2
    ph = (y1 - y0 - FW_TOP - FW_BOT - FW_BAR) / 2
    out = []
    for j in range(2):
        for i in range(2):
            px = x0 + FW_STILE + i * (pw + FW_BAR)
            py = y0 + FW_BOT + j * (ph + FW_BAR)
            out.append((px, px + pw, py, py + ph))
    return out


def fw_frame():
    x0, x1, y0, y1 = FW_OPEN
    # the architrave: a 40 x 19 mm moulding (a rounded outer edge, a flat
    # face, a step up and a rounded inner edge) mitred round the opening,
    # its legs standing on the window board
    prof = [(0.0, 0.0), (0.0, 0.010)]
    prof += [(0.004 - 0.004 * math.cos(a), 0.010 + 0.0045 * math.sin(a)) for a in (math.pi / 6, math.pi / 3, math.pi / 2)]
    prof += [(0.026, 0.0160), (0.0285, 0.0165), (0.0290, 0.0190), (0.0350, 0.0190)]
    prof += [(0.0350 + 0.005 * math.sin(a), 0.0140 + 0.005 * math.cos(a)) for a in (math.pi / 4, math.pi / 2)]
    prof += [(FW_ARC, 0.0)]
    bot = y0 - FW_ARC
    arc_w, arc_h = x1 - x0 + 2 * FW_ARC, y1 + FW_ARC - bot
    arch = rect_sweep(clean_profile(prof), arc_w, arc_h).move(0, (y1 + FW_ARC + bot) / 2, 0)
    arch = cut(arch, box(1.0, 0.2, 0.2).move(0, y0 - 0.1, 0))
    # the window board: 22 mm, out 40 mm from the wall, its nose rounded,
    # its horns to the architrave's outer edges
    board = rbox(FW_W, 0.022, 0.040, r=0.006, seg=3).move(0, y0 - 0.011, 0.020)
    # the apron under it, its lower edge rounded
    ah = FW_H / 2 + y0 - 0.022
    apron = rbox(x1 - x0, ah, 0.012, r=0.003, seg=2).move(0, -FW_H / 2 + ah / 2, 0.006)
    # the lining inside the architraves, a step back from them, and the
    # sash in it, its face proud of the lining, a shadow gap round it
    sx0, sx1, sy0, sy1 = FW_SASH
    lining = cut(box(x1 - x0, y1 - y0, 0.008).move(0, (y0 + y1) / 2, 0.004),
                 box(sx1 - sx0 + 0.004, sy1 - sy0 + 0.004, 0.1).move(0, (sy0 + sy1) / 2, 0))
    sash = box(sx1 - sx0, sy1 - sy0, FW_SASH_Z - 0.002).move(0, (sy0 + sy1) / 2, (FW_SASH_Z + 0.002) / 2)
    holes = [box(px1 - px0, py1 - py0, 0.1).move((px0 + px1) / 2, (py0 + py1) / 2, 0) for px0, px1, py0, py1 in fw_panes()]
    sash = cut(sash, *holes)
    sash = bevel(sash, 0.0015, seg=1)
    # the ovolo bead round each pane, on the glass
    beads = Shape()
    bprof = [(0.0, FW_GLASS_Z), (0.0, FW_SASH_Z - 0.0005)]
    bprof += [(0.0075 - 0.0075 * math.cos(a), FW_GLASS_Z + 0.0005 + (FW_SASH_Z - FW_GLASS_Z - 0.001) * math.cos(a)) for a in
              (math.pi / 8, math.pi / 4, 3 * math.pi / 8, math.pi / 2)]
    bprof += [(0.0075, FW_GLASS_Z)]
    for px0, px1, py0, py1 in fw_panes():
        beads = beads + rect_sweep(clean_profile(bprof), px1 - px0, py1 - py0).move((px0 + px1) / 2, (py0 + py1) / 2, 0)
    # the hinges' knuckles on the left stile
    knuckles = Shape()
    for y in (sy0 + 0.06, sy1 - 0.06):
        knuckles = knuckles + cyl(0.0035, h=0.05, seg=8, bevel_r=0.0006, bevel_seg=1).move(sx0 - 0.001, y, FW_SASH_Z - 0.002)
    return arch + board + apron + lining + sash + beads + knuckles


def fw_iron():
    """The cockspur fastener on the sash's right stile, its striker on the
    lining; black japanned iron."""
    sx1 = FW_SASH[1]
    y = 0.016
    x = sx1 - FW_STILE / 2
    z = FW_SASH_Z
    plate = rbox(0.016, 0.044, 0.003, r=0.0012, seg=1).move(x, y, z + 0.0015)
    boss = cyl(0.0055, h=0.008, seg=12, bevel_r=0.001).rot(math.pi / 2, 0, 0).move(x, y + 0.006, z + 0.006)
    # the lever: from the boss across onto the lining, its spur hooked
    pts = [(x, y + 0.006, z + 0.009), (x + 0.012, y + 0.004, z + 0.010), (x + 0.026, y - 0.004, z + 0.009),
           (x + 0.036, y - 0.016, z + 0.008), (x + 0.040, y - 0.034, z + 0.0075)]
    lever = tube(pts, 0.0032, seg=7, smooth=1, r_end=0.0042)
    spur = tube([(x + 0.010, y + 0.005, z + 0.010), (x + 0.017, y + 0.010, z + 0.006)], 0.0022, seg=6)
    screws = Shape()
    for dy in (-0.016, 0.016):
        screws = screws + screw_head(r=0.0022, h=0.0008).rot(math.pi / 2, 0, 0).move(x, y + dy, z + 0.003)
    striker = rbox(0.008, 0.026, 0.004, r=0.001, seg=1).move(FW_OPEN[1] - 0.006, y + 0.002, 0.010)
    return plate + boss + lever + spur + screws + striker


auto('Real__frostedWindowFrame', fw_frame, 1800)
auto('Real__frostedWindowIron', fw_iron, 600)


# ========================================================== sink faucet ===
# A wall-mounted two-valve faucet on 4 in (102 mm) centres, the Chicago
# Faucets 445 class: the supply arms out of the wall through domed flanges
# into the two valve bodies, a cross body between them, the bonnets and
# stems standing up, four-arm handles with their index buttons, and an
# L-type swing spout (8 in reach) on the swivel in the middle. Below it, what
# the basin left when it was taken off the wall: its steel hanger, and the
# waste's wall bend cut off at its slip nut.
SF_C = 0.0508                 # half the centres
SF_Z = 0.050                  # the valve bodies' axis off the wall
SF_SEAT = 0.035               # the stems' shoulders (the handles' seat)
SF_SWIVEL = 0.022             # the swivel's top (the spout's seat)
SF_HANGER_Y = -0.205          # the hanger's middle
SF_WASTE_Y = -0.430           # the waste's axis


def sf_body():
    s = Shape()
    for sx in (-1, 1):
        x = sx * SF_C
        fl = lathe([(0.0, 0.0), (0.024, 0.0), (0.024, 0.0025), (0.021, 0.005), (0.013, 0.0072), (0.0095, 0.0078), (0.0, 0.0078)], seg=28)
        s = s + along_z(fl).move(x, 0, 0)
        arm = cyl(0.0085, h=SF_Z - 0.006, seg=16, bevel_r=0).move(0, (SF_Z - 0.006) / 2 + 0.006, 0)
        s = s + along_z(arm).move(x, 0, 0)
        vb = lathe([(0.0, -0.024), (0.0125, -0.024), (0.0145, -0.021), (0.0145, 0.012), (0.0125, 0.016), (0.0125, 0.019), (0.0, 0.019)], seg=24)
        s = s + vb.move(x, 0, SF_Z)
        s = s + hexagon(0.0135, 0.010).move(x, 0.024, SF_Z)
        s = s + cyl(0.0105, h=0.006, seg=20, bevel_r=0.0008).move(x, 0.032, SF_Z)
        s = s + cyl(0.0048, h=0.010, seg=12, bevel_r=0.0005).move(x, 0.038, SF_Z)
    s = s + along_x(cyl(0.0115, h=2 * SF_C, seg=20, bevel_r=0)).move(0, -0.006, SF_Z)
    s = s + cyl(0.012, h=0.010, seg=20, bevel_r=0.0008).move(0, 0.009, SF_Z)
    s = s + hexagon(0.0125, 0.008).move(0, SF_SWIVEL - 0.004, SF_Z)
    # a drain plug under the cross body
    s = s + hexagon(0.007, 0.006).move(0, -0.020, SF_Z)
    return s


def sf_spout():
    """The L-type swing spout, swung straight out from the wall: up from the
    swivel, a bend, 8 in out, a bend down to the aerator."""
    Z, R = SF_Z, 0.020
    pts = [(0, SF_SWIVEL - 0.002, Z), (0, 0.060, Z)]
    pts += [(0, 0.060 + R * math.sin(t), Z + R - R * math.cos(t)) for t in [math.pi / 2 * k / 5 for k in range(1, 6)]]
    pts += [(0, 0.080, Z + 0.180 - R * 0.5)]
    pts += [(0, 0.060 + R * math.cos(t), Z + 0.180 + R * math.sin(t)) for t in [math.pi / 2 * k / 5 for k in range(0, 6)]]
    pts += [(0, 0.050, Z + 0.200)]
    s = tube(pts, 0.0080, seg=16)
    # the spout's collar on the swivel
    s = s + cyl(0.0115, h=0.012, seg=20, bevel_r=0.001).move(0, SF_SWIVEL + 0.006, Z)
    # the aerator: a knurled sleeve, its screen recessed
    aer = lathe([(0.0, 0.0), (0.0068, 0.0), (0.0068, 0.0015), (0.0098, 0.0015), (0.0104, 0.002), (0.0104, 0.016), (0.0092, 0.019),
                 (0.0085, 0.020), (0.0, 0.020)], seg=24)
    s = s + aer.move(0, 0.050 - 0.020 - 0.0005, Z + 0.200)
    return s


def sf_handle(x, turn):
    s = lathe([(0.0, 0.0), (0.0095, 0.0), (0.0105, 0.004), (0.0098, 0.011), (0.0080, 0.0135), (0.0, 0.0140)], seg=20)
    for k in range(4):
        a = turn + math.pi / 2 * k
        d = Vector((math.cos(a), 0, math.sin(a)))
        p0 = Vector((0, 0.0075, 0)) + d * 0.006
        p1 = Vector((0, 0.0085, 0)) + d * 0.027
        s = s + tube([p0, p0.lerp(p1, 0.5), p1], 0.0034, seg=10, r_end=0.0044)
        s = s + sphere(0.0058, seg=12).move(*(Vector((0, 0.0088, 0)) + d * 0.030))
    return s.move(x, SF_SEAT, SF_Z)


SF_TURNS = (0.35, -0.6)       # how the hot and the cold handle were left


def sf_handles():
    return sf_handle(-SF_C, SF_TURNS[0]) + sf_handle(SF_C, SF_TURNS[1])


def sf_index(x):
    b = cyl(0.0064, h=0.0018, seg=20, bevel_r=0.0006, bevel_seg=1)
    return b.move(x, SF_SEAT + 0.0140 + 0.0006, SF_Z)


def sf_hanger():
    """The flat steel hanger a wall-hung basin hooks onto (406 x 51 x 4 mm),
    its two lugs bent out and up, four lag screws."""
    plate = rbox(0.406, 0.051, 0.004, r=0.0012, seg=1).move(0, SF_HANGER_Y, 0.002)
    lugs = Shape()
    for x in (-0.150, 0.150):
        lug = box(0.030, 0.004, 0.014).move(0, 0, 0.007) + box(0.030, 0.012, 0.004).move(0, 0.006, 0.0125)
        lugs = lugs + bevel(lug, 0.0008, seg=1).move(x, SF_HANGER_Y + 0.0235, 0.004)
    screws = Shape()
    for x in (-0.180, -0.062, 0.062, 0.180):
        hx = hexagon(0.0055, 0.0045).rot(math.pi / 2, 0, 0).move(x, SF_HANGER_Y - 0.004, 0.0062)
        washer = cyl(0.0085, h=0.0012, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0).move(x, SF_HANGER_Y - 0.004, 0.0046)
        screws = screws + hx + washer
    return plate + lugs + screws


def sf_waste():
    """The waste's wall bend, cut off: its escutcheon on the wall, the 1 1/4 in
    tube out of it (open), the slip nut on it."""
    esc = lathe([(0.0, 0.0), (0.031, 0.0), (0.031, 0.002), (0.026, 0.006), (0.019, 0.0085), (0.0, 0.0085)], seg=32)
    t = vessel([(0.0159, 0.0), (0.0159, 0.068)], 0.0009, seg=24, floor=0.004)
    nut = lathe([(0.0, 0.040), (0.0205, 0.040), (0.0215, 0.042), (0.0215, 0.053), (0.0195, 0.056), (0.0, 0.056)], seg=24)
    # the knurl on the nut: shallow ribs round it
    nut = nut.warp(lambda p: p if abs(math.hypot(p.x, p.z) - 0.0215) > 1e-4 else
                   Vector((p.x, p.y, p.z)) * 1.0 + Vector((p.x, 0, p.z)).normalized() * 0.0006 * (math.cos(math.atan2(p.z, p.x) * 24) > 0))
    return along_z(esc + t + nut).move(0, SF_WASTE_Y, 0)


auto('Real__sinkFaucetBody', sf_body, 2600)
auto('Real__sinkFaucetSpout', sf_spout, 1100)
auto('Real__sinkFaucetHandles', sf_handles, 1800)
auto('Real__sinkFaucetIndexHot', lambda: sf_index(-SF_C), 180)
auto('Real__sinkFaucetIndexCold', lambda: sf_index(SF_C), 180)
auto('Real__sinkFaucetHanger', sf_hanger, 700)
auto('Real__sinkFaucetWaste', sf_waste, 700)


# ============================================================ towel bar ===
# A 12 in towel bar of the hardware store's kind (the Deltana 88TB12 class:
# 14 5/8 in over its posts, 3 1/8 in out from the wall): a 3/4 in chromed
# tube between two posts on round rosettes. Over it a cotton terry hand towel
# (16 x 26 in), folded in half along its length and hung over the bar,
# draped by Blender's cloth solver.
TB_C = 0.1524                 # half the posts' centres
TB_Z = 0.064                  # the bar's axis off the wall
TB_R = 0.0095                 # the bar
TW_W, TW_L = 0.205, 0.660     # the towel folded: across, along
TW_STEP = 0.012
TW_FRONT = 0.318              # how much of it hangs in front of the bar
TW_SHIFT = 0.018              # slid along the bar from its middle
TW_T = 0.0024                 # half the folded terry's thickness


def tb_metal():
    s = along_x(cyl(TB_R, h=2 * TB_C, seg=24, bevel_r=0))
    s.move(0, 0, TB_Z)
    for sx in (-1, 1):
        x = sx * TB_C
        ros = lathe([(0.0, 0.0), (0.028, 0.0), (0.028, 0.003), (0.025, 0.007), (0.015, 0.0092), (0.0115, 0.0096), (0.0, 0.0096)], seg=32)
        s = s + along_z(ros).move(x, 0, 0)
        post = lathe([(0.0, 0.0090), (0.0105, 0.0090), (0.0088, 0.022), (0.0084, 0.036), (0.0094, TB_Z - 0.010), (0.0, TB_Z - 0.010)], seg=20)
        s = s + along_z(post).move(x, 0, 0)
        # the head the bar's end sits in, its outer end domed
        head = lathe([(0.0, -0.012), (0.0125, -0.012), (0.0135, -0.010), (0.0135, 0.006), (0.0120, 0.0095), (0.0085, 0.0118),
                      (0.0040, 0.0127), (0.0, 0.0129)], seg=24)
        s = s + along_x(head.rot(0, 0, 0)).scale(sx, 1, 1).move(x, 0, TB_Z)
    return s


def tw_flat():
    nu = int(round(TW_W / TW_STEP))
    nv = int(round(TW_L / TW_STEP))
    pts, faces = [], []
    for j in range(nv + 1):
        for i in range(nu + 1):
            pts.append((-TW_W / 2 + TW_W * i / nu, TW_L * j / nv))
    for j in range(nv):
        for i in range(nu):
            a = j * (nu + 1) + i
            faces.append((a, a + 1, a + nu + 2, a + nu + 1))
    return pts, faces, nu, nv


def tw_start(x, v):
    """Where the flat towel's point (x across, v along from the front end)
    starts: over the bar, a little gathered at it, a little askew."""
    R0 = TB_R + 0.0035
    arc = math.pi * R0
    w = 1.0 - smoothstep(0.0, 0.16, abs(v - TW_FRONT - arc / 2))
    xg = x * (1.0 - 0.24 * w) + TW_SHIFT
    # soft folds down the legs, from the gathering at the bar
    wave = 0.005 * math.sin(x / TW_W * TAU * 2.2 + 0.8) * (0.4 + 0.6 * w)
    if v < TW_FRONT:
        d = TW_FRONT - v
        p = Vector((xg + 0.05 * d * (x / TW_W), -d, TB_Z + R0 + 0.004 * d + wave))
    elif v < TW_FRONT + arc:
        a = (v - TW_FRONT) / R0
        p = Vector((xg, R0 * math.sin(a), TB_Z + R0 * math.cos(a)))
    else:
        d = v - TW_FRONT - arc
        p = Vector((xg - 0.03 * d * (x / TW_W), -d, TB_Z - R0 - 0.004 * d - wave * 0.6))
    p += Vector((0.0, 0.0, 0.002 * noise.noise(Vector((x * 11.0, v * 7.0, 0.31)))))
    return p


def tw_drape():
    import bpy
    pts, faces, nu, nv = tw_flat()
    init = [tw_start(x, v) for x, v in pts]
    me = bpy.data.meshes.new('towel_sim')
    me.from_pydata([tuple(p) for p in init], [], faces)
    me.update()
    ob = bpy.data.objects.new('towel_sim', me)
    bpy.context.scene.collection.objects.link(ob)
    # held where it lies across the top of the bar
    R0 = TB_R + 0.0035
    vtop = TW_FRONT + math.pi * R0 / 2
    pins = [k for k, (x, v) in enumerate(pts) if abs(v - vtop) < TW_STEP * 0.75]
    g = ob.vertex_groups.new(name='pin')
    g.add(pins, 1.0, 'REPLACE')
    # the bar and the wall it hangs against
    bar = along_x(cyl(TB_R + 0.0005, h=0.5, seg=24, bevel_r=0)).move(0, 0, TB_Z)
    bm = bpy.data.meshes.new('towel_bar')
    bm.from_pydata([tuple(p) for p in bar.v], [], bar.f)
    bm.update()
    bob = bpy.data.objects.new('towel_bar', bm)
    bpy.context.scene.collection.objects.link(bob)
    bob.modifiers.new('col', 'COLLISION')
    bob.collision.thickness_outer = 0.002
    bob.collision.cloth_friction = 10.0
    wme = bpy.data.meshes.new('towel_wall')
    wme.from_pydata([(-1.0, -1.0, 0.0), (1.0, -1.0, 0.0), (1.0, 1.0, 0.0), (-1.0, 1.0, 0.0)], [], [(0, 1, 2, 3)])
    wme.update()
    wall = bpy.data.objects.new('towel_wall', wme)
    bpy.context.scene.collection.objects.link(wall)
    wall.modifiers.new('col', 'COLLISION')
    wall.collision.thickness_outer = 0.002
    sc = bpy.context.scene
    grav = tuple(sc.gravity)
    sc.gravity = (0.0, -9.81, 0.0)
    cl = ob.modifiers.new('cloth', 'CLOTH')
    st = cl.settings
    st.quality = 8
    st.mass = 0.05
    st.tension_stiffness = 20.0
    st.compression_stiffness = 20.0
    st.shear_stiffness = 8.0
    st.bending_stiffness = 0.25
    st.air_damping = 3.0
    st.vertex_group_mass = 'pin'
    c = cl.collision_settings
    c.use_collision = True
    c.distance_min = 0.003
    c.use_self_collision = True
    c.self_distance_min = 0.003
    c.collision_quality = 4
    frames = 160
    sc.frame_start, sc.frame_end = 1, frames
    cl.point_cache.frame_start, cl.point_cache.frame_end = 1, frames
    for fr in range(1, frames + 1):
        sc.frame_set(fr)
    dg = bpy.context.evaluated_depsgraph_get()
    em = ob.evaluated_get(dg).to_mesh()
    out = [Vector(vv.co) for vv in em.vertices]
    ob.evaluated_get(dg).to_mesh_clear()
    sc.frame_set(1)
    sc.gravity = grav
    for o, m in ((ob, me), (bob, bm), (wall, wme)):
        bpy.data.objects.remove(o)
        bpy.data.meshes.remove(m)
    return out, faces, nu, nv


def tw_towel():
    P, faces, nu, nv = tw_drape()
    P = [p.copy() for p in P]
    # the terry is heavier and softer than the solver's cloth: relax its crinkle
    cols = nu + 1
    for _ in range(2):
        Q = [p.copy() for p in P]
        for j in range(1, nv):
            for i in range(1, nu):
                k = j * cols + i
                m = (P[k - 1] + P[k + 1] + P[k - cols] + P[k + cols]) / 4
                Q[k] = P[k].lerp(m, 0.3)
        P = Q
    # normals of the sheet, turned out to the room on the front leg
    N = [Vector((0, 0, 0)) for _ in P]
    for f in faces:
        a, b, c, d = (P[i] for i in f)
        n = (c - a).cross(d - b)
        for i in f:
            N[i] += n
    N = [n.normalized() if n.length > 1e-12 else Vector((0, 0, 1)) for n in N]
    front = [k for k in range(len(P)) if P[k].y < -0.1 and P[k].z > TB_Z]
    if front and sum(N[k].z for k in front) < 0:
        N = [-n for n in N]
    # two layers of terry folded together: a slab round the sheet, its edges
    # rounded (the fold along one side, the hems along the other)
    top = [P[k] + N[k] * TW_T for k in range(len(P))]
    bot = [P[k] - N[k] * TW_T for k in range(len(P))]
    n = len(P)
    v = top + bot + [P[k] for k in range(n)]
    f = [tuple(x for x in fc) for fc in faces] + [tuple(reversed([x + n for x in fc])) for fc in faces]
    # the rim: every boundary edge of the grid, top to the middle to bottom
    def rim(a, b):
        mid_a, mid_b = 2 * n + a, 2 * n + b
        # the middle ring bulges out a little past the sheet's edge
        return [(a, b, mid_b, mid_a), (mid_a, mid_b, b + n, a + n)]
    edges = []
    for i in range(nu):
        edges.append((i, i + 1))
        edges.append((nv * cols + i + 1, nv * cols + i))
    for j in range(nv):
        edges.append(((j + 1) * cols, j * cols))
        edges.append((j * cols + nu, (j + 1) * cols + nu))
    used_mid = set()
    for a, b in edges:
        f += rim(a, b)
        used_mid.update((a, b))
    # push the middle ring out along the sheet's own edge direction
    for k in used_mid:
        j, i = divmod(k, cols)
        out = Vector((0, 0, 0))
        if i == 0:
            out += P[k] - P[k + 1]
        if i == nu:
            out += P[k] - P[k - 1]
        if j == 0:
            out += P[k] - P[k + cols]
        if j == nv:
            out += P[k] - P[k - cols]
        if out.length > 1e-9:
            v[2 * n + k] = P[k] + out.normalized() * TW_T * 0.8
    # drop the middle points no rim face uses
    s = Shape(v, f)
    used = sorted({i for fc in s.f for i in fc})
    remap = {o: m for m, o in enumerate(used)}
    s = Shape([s.v[i] for i in used], [tuple(remap[i] for i in fc) for fc in s.f])
    return propkit._fix_winding(s)


auto('Real__towelBarMetal', tb_metal, 1700)
auto('Real__towelBarTowel', tw_towel, 5200, smooth=70)


# ========================================================= spray bottle ===
# A 16 oz HDPE cylinder round (Silgan 78997: 2.429 x 7.889 in, a 28/400
# neck) with a 28/400 trigger sprayer screwed on: the ribbed closure, the
# shroud over the pump reaching forward to the nozzle, the trigger under it,
# the turning nozzle cap (OFF / SPRAY / STREAM). Standing on y = 0, its
# nozzle toward +z.
SB_R = 0.0309                 # the body's radius
SB_H = 0.2004                 # to the neck's top
SB_NOZZLE = (0.0, 0.2475, 0.054)


def sb_body():
    prof = [(0.0, 0.0035), (0.010, 0.0030), (0.018, 0.0012), (0.022, 0.0), (0.0255, 0.0), (0.0285, 0.0012), (0.0302, 0.0040),
            (SB_R, 0.0085), (SB_R, 0.150), (0.0306, 0.157), (0.0293, 0.1645), (0.0268, 0.1705), (0.0232, 0.1755),
            (0.0190, 0.1792), (0.0158, 0.1815), (0.0150, 0.1830), (0.0165, 0.1832), (0.0165, 0.1848), (0.0146, 0.1852),
            (0.0146, 0.1985), (0.0136, SB_H), (0.0, SB_H)]
    s = lathe(prof, seg=40)
    # two shallow mould seams down its sides (x = 0 plane)
    return s


def sb_head():
    # the closure: 32.5 mm across, ribbed, over the neck
    clo = cyl(0.0168, h=0.0195, seg=72, bevel_r=0.0008).move(0, 0.1835 + 0.0098, 0)

    def rib(p):
        r = math.hypot(p.x, p.z)
        if r > 0.0163 and 0.1850 < p.y < 0.2020:
            a = math.atan2(p.z, p.x)
            k = 0.0006 if math.cos(a * 36) > 0 else 0.0
            p = Vector((p.x * (r + k) / r, p.y, p.z * (r + k) / r))
        return p
    clo = clo.warp(rib)
    # the shroud: up from the closure and forward to the nozzle, its
    # sections rounded squares
    path = [Vector((0, 0.2020, 0.0)), Vector((0, 0.2160, 0.0)), Vector((0, 0.2290, 0.003)), Vector((0, 0.2390, 0.012)),
            Vector((0, 0.2450, 0.025)), Vector((0, 0.2475, 0.038)), Vector((0, 0.2480, 0.049))]
    sizes = [(0.0160, 0.0160), (0.0150, 0.0150), (0.0150, 0.0150), (0.0150, 0.0140), (0.0130, 0.0120), (0.0110, 0.0100),
             (0.0095, 0.0088)]
    secs = []
    for i, (p, (a, b)) in enumerate(zip(path, sizes)):
        t = (path[min(i + 1, len(path) - 1)] - path[max(i - 1, 0)]).normalized()
        side = Vector((1, 0, 0))
        nrm = t.cross(side).normalized()
        e = 2.0 if i == 0 else 3.2
        secs.append([p + side * u + nrm * w for u, w in superellipse(a, b, 20, e)])
    shroud = loft(secs)
    # its back: a rounded hump over the pump's piston
    back = rbox(0.031, 0.030, 0.034, r=0.009, seg=3).move(0, 0.2335, -0.004)
    # the trigger, hung from the shroud under the nozzle, its finger curve
    tpath = [Vector((0, 0.2380, 0.0355)), Vector((0, 0.2290, 0.0445)), Vector((0, 0.2160, 0.0480)),
             Vector((0, 0.2030, 0.0465)), Vector((0, 0.1930, 0.0420)), Vector((0, 0.1880, 0.0360))]
    tsecs = []
    for i, p in enumerate(tpath):
        t = (tpath[min(i + 1, len(tpath) - 1)] - tpath[max(i - 1, 0)]).normalized()
        side = Vector((1, 0, 0))
        nrm = t.cross(side).normalized()
        a = 0.0085 - 0.0012 * (i / (len(tpath) - 1))
        tsecs.append([p + side * u + nrm * w for u, w in superellipse(a, 0.0044, 16, 4.0)])
    trigger = loft(tsecs)
    return clo + shroud + back + trigger


def sb_nozzle():
    n = rbox(0.0150, 0.0150, 0.0110, r=0.0022, seg=2)
    n = cut(n, cyl(0.0012, h=0.01, seg=8, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.006))
    # the grip fins on its sides
    fins = Shape()
    for sx in (-1, 1):
        fins = fins + rbox(0.0016, 0.0110, 0.0060, r=0.0005, seg=1).move(sx * 0.0080, 0, -0.001)
    return (n + fins).move(SB_NOZZLE[0], SB_NOZZLE[1], SB_NOZZLE[2] + 0.0055)


auto('Real__sprayBottleBody', sb_body, 1700, kind='cyl')
auto('Real__sprayBottleHead', sb_head, 1800)
auto('Real__sprayBottleNozzle', sb_nozzle, 400)


# ============================================================ hair dryer ===
# An 1875 W pistol-grip hair dryer of the common kind (a hotel dryer:
# 7.3 x 3.5 x 9.5 in; a 5 ft cord): the barrel 183 mm long and 76 mm
# across, the intake's grille at its back, the outlet's in its nozzle, the
# handle under it leaning back with two slide switches on its front and a
# hang loop at its foot, the cord's strain relief out of it. Modelled upright
# (barrel along +x, the handle down), then laid on its side as it fell: on
# the barrel and the handle's side, the handle along -z.
HD_R = 0.038
HD_REAR, HD_FRONT = -0.088, 0.095
HD_HANDLE = [Vector((-0.010, -0.018, 0)), Vector((-0.016, -0.050, 0)), Vector((-0.024, -0.090, 0)),
             Vector((-0.031, -0.125, 0)), Vector((-0.036, -0.148, 0))]
HD_HSIZE = [(0.0175, 0.0140), (0.0170, 0.0138), (0.0160, 0.0132), (0.0158, 0.0128), (0.0168, 0.0132)]


def hd_barrel():
    prof = [(0.0, HD_REAR + 0.004), (0.0300, HD_REAR + 0.004), (0.0300, HD_REAR), (0.0335, HD_REAR), (0.0362, HD_REAR + 0.003),
            (0.0376, HD_REAR + 0.010), (HD_R, -0.050), (HD_R, 0.000), (0.0373, 0.030), (0.0352, 0.055), (0.0318, 0.074),
            (0.0290, 0.086), (0.0278, 0.093), (0.0266, HD_FRONT), (0.0222, HD_FRONT), (0.0222, HD_FRONT - 0.006),
            (0.0, HD_FRONT - 0.006)]
    return along_x(lathe(prof, seg=40))


def hd_frame(i, path):
    t = (path[min(i + 1, len(path) - 1)] - path[max(i - 1, 0)]).normalized()
    side = Vector((0, 0, 1))
    nrm = side.cross(t).normalized()      # toward the front (+x) of the handle
    return t, nrm, side


def hd_handle():
    secs = []
    for i, (p, (a, b)) in enumerate(zip(HD_HANDLE, HD_HSIZE)):
        t, nrm, side = hd_frame(i, HD_HANDLE)
        secs.append([p + nrm * u + side * w for u, w in superellipse(a, b, 24, 2.8)])
    h = loft(secs)
    # the hang loop at its foot
    end = HD_HANDLE[-1]
    t, nrm, side = hd_frame(len(HD_HANDLE) - 1, HD_HANDLE)
    loop = torus(0.0085, 0.0028, seg=20, tseg=8)
    # its ring in the handle's front/back plane, below the foot, behind the cord
    loop = loop.apply(Matrix((nrm, t, side)).transposed().to_4x4())
    loop.move(*(end + t * 0.012 - nrm * 0.008))
    return h + loop


def hd_body():
    return hd_barrel() + hd_handle()


def hd_grille():
    s = Shape()
    # the intake: concentric rings and spokes in the back's recess
    for r in (0.008, 0.014, 0.020, 0.026):
        s = s + torus(r, 0.0011, seg=28, tseg=4).rot(0, math.pi / 2, 0).move(HD_REAR + 0.003, 0, 0)
    for k in range(6):
        a = TAU * k / 6
        s = s + box(0.002, 0.026, 0.0024).move(0, 0.015, 0).rot(a, 0, 0).move(HD_REAR + 0.003, 0, 0)
    s = s + along_x(cyl(0.0045, h=0.003, seg=12, bevel_r=0)).move(HD_REAR + 0.003, 0, 0)
    # the outlet: a disc of rings and bars recessed in the nozzle
    for r in (0.006, 0.012, 0.018):
        s = s + torus(r, 0.0010, seg=24, tseg=4).rot(0, math.pi / 2, 0).move(HD_FRONT - 0.004, 0, 0)
    for k in range(4):
        a = TAU * k / 4 + 0.3
        s = s + box(0.0018, 0.040, 0.0020).rot(a, 0, 0).move(HD_FRONT - 0.004, 0, 0)
    return s


def hd_switches():
    s = Shape()
    for (k, w) in ((0.30, 0.014), (0.52, 0.012)):
        n = len(HD_HANDLE) - 1
        f = k * n
        i = int(f)
        p = HD_HANDLE[i].lerp(HD_HANDLE[i + 1], f - i)
        a = HD_HSIZE[i][0] + (HD_HSIZE[i + 1][0] - HD_HSIZE[i][0]) * (f - i)
        t, nrm, side = hd_frame(i, HD_HANDLE)
        base = rbox(0.004, 0.020, 0.012, r=0.0015, seg=2)
        knob = rbox(0.004, w * 0.55, 0.009, r=0.0012, seg=2).move(0.003, 0.003, 0)
        sw = base + knob
        sw = sw.apply(Matrix((nrm, t, side)).transposed().to_4x4())
        s = s + sw.move(*(p + nrm * (a - 0.0008)))
    return s


def hd_pose_matrix():
    """Laid on its side (the handle along -z), then tilted about the barrel
    till the handle's foot is down on the floor too; then lifted onto it."""
    m0 = Matrix.Rotation(math.pi / 2, 4, 'X')
    body = hd_body()
    best = None
    for k in range(0, 61):
        th = -0.005 * k
        m = Matrix.Rotation(th, 4, 'X') @ m0
        ys = [(m @ p).y for p in body.v]
        barrel_low = -HD_R
        hand_low = min((m @ p).y for p in body.v if (m @ p).z < -0.06)
        d = abs(hand_low - barrel_low)
        if best is None or d < best[0]:
            best = (d, m)
    m = best[1]
    low = min((m @ p).y for p in body.v)
    return Matrix.Translation((0, -low, 0)) @ m


HD_POSE = None


def hd_posed(s):
    global HD_POSE
    if HD_POSE is None:
        HD_POSE = hd_pose_matrix()
    return s.apply(HD_POSE)


def hd_strain():
    """The cord's strain relief: out of the handle's foot along it, bending
    down to the floor; its ribs. Built posed."""
    global HD_CORD
    end = HD_HANDLE[-1]
    t, nrm, side = hd_frame(len(HD_HANDLE) - 1, HD_HANDLE)
    hd_posed(Shape())
    p0 = HD_POSE @ (end - t * 0.004)
    d = (HD_POSE.to_3x3() @ t).normalized()
    pts = [p0]
    p = p0.copy()
    for k in range(1, 9):
        d = (d + Vector((0, -0.10, 0))).normalized()
        p = p + d * 0.0065
        p.y = max(p.y, 0.0058 + (0.0038 - 0.0058) * k / 8 + 0.0003)
        pts.append(p.copy())
    s = tube(pts, 0.0058, seg=14, r_end=0.0038)
    s = s.warp(lambda q: Vector((q.x, max(q.y, 0.0), q.z)))      # flattened where it lies on the floor
    for k in (2, 3, 4, 5, 6):
        r = 0.0058 + (0.0038 - 0.0058) * k / 8 + 0.0006
        rr = torus(r, 0.0007, seg=14, tseg=4)
        tt = (pts[k + 1] - pts[k - 1]).normalized()
        rr = rr.apply(tt.to_track_quat('Z', 'Y').to_matrix().to_4x4())
        s = s + rr.move(*pts[k])
    HD_CORD = (pts[-1], (pts[-1] - pts[-2]).normalized())
    print('t04: the hair dryer\'s cord leaves its strain relief at %.4f %.4f %.4f going %.3f %.3f %.3f' % (tuple(pts[-1]) + tuple(HD_CORD[1])))
    return s


HD_CORD = None


def hd_plug():
    """A moulded two-pin plug lying flat, its cord into it from -x: the body
    (26 x 12 x 22 mm), its grip ribs, the cord's stub."""
    body = rbox(0.026, 0.012, 0.022, r=0.003, seg=2).move(0.013, 0.006, 0)
    ribs = Shape()
    for k in range(4):
        ribs = ribs + rbox(0.0016, 0.0120, 0.0225, r=0.0006, seg=1).move(0.004 + 0.0035 * k, 0.006, 0)
    stub = tube([(-0.010, 0.0036, 0), (0.0, 0.0042, 0), (0.004, 0.0052, 0)], 0.0034, seg=10, r_end=0.0046)
    return body + ribs + stub


def hd_prongs():
    s = Shape()
    for z, w in ((-0.00635, 0.0063), (0.00635, 0.0079)):
        b = rbox(0.0165, 0.0015, w, r=0.0004, seg=1)
        b = cut(b, cyl(0.0016, h=0.01, seg=8, bevel_r=0).move(0.0055, 0, 0))
        s = s + b.move(0.026 + 0.0080, 0.006, z)
    return s


auto('Real__hairdryerBody', lambda: hd_posed(hd_body()), 2600)
auto('Real__hairdryerGrille', lambda: hd_posed(hd_grille()), 2000)
auto('Real__hairdryerSwitches', lambda: hd_posed(hd_switches()), 460)
auto('Real__hairdryerStrain', hd_strain, 900)
auto('Real__hairdryerPlug', hd_plug, 700)
auto('Real__hairdryerProngs', hd_prongs, 200)


# =============================================================== strobe ===
# A fire alarm's wall strobe of the compact kind (the Edwards Genesis
# G1RF-VM: 2.75 x 4.5 in, about 0.8 in out from the wall): a red moulded
# housing on its mounting plate, a clear prismatic lens over the xenon tube
# and its reflector in its upper part; FIRE below it (the game's print).
ST_W, ST_H, ST_D = 0.070, 0.114, 0.021
ST_LENS = (0.0, 0.026, 0.050, 0.034)      # x, y, w, h
ST_LENS_D = 0.007


def st_housing():
    plate = rbox(ST_W + 0.002, ST_H + 0.002, 0.004, r=0.0012, seg=1).move(0, 0, 0.002)
    shell = prism(rounded_rect(ST_W, ST_H, 0.006, seg=3), ST_D - 0.0045)
    shell = bevel(shell, 0.0035, seg=3)
    shell.move(0, 0, 0.0045 + (ST_D - 0.0045) / 2)
    # the lens's seat: a shallow frame round it
    lx, ly, lw, lh = ST_LENS
    seat = prism(rounded_rect(lw + 0.006, lh + 0.006, 0.004, seg=2), 0.0016).move(lx, ly, ST_D + 0.0008)
    seat = bevel(seat, 0.0006, seg=1)
    # the window the lens covers, open to the reflector behind it
    win = rbox(lw - 0.003, lh - 0.003, 0.02, r=0.002, seg=1).move(lx, ly, ST_D + 0.006 - 0.01 + 0.0)
    return plate + cut(shell, win) + cut(seat, win)


def st_lens():
    lx, ly, lw, lh = ST_LENS
    # its side profile (y, z): the back on the housing, the front ridged
    # across in eight prisms
    out = [(-lh / 2, 0.0), (lh / 2, 0.0)]
    n = 8
    for k in range(n + 1):
        y = lh / 2 - lh * k / n
        out.append((y, ST_LENS_D - (0.0012 if 0 < k < n else 0.0022)))
        if k < n:
            out.append((y - lh / n / 2, ST_LENS_D))
    s = prism(out, lw)          # the profile in its x, y; extruded along its z
    s = bevel(s, 0.0004, seg=1)
    # its x is our y, its y our z, its z (the extrusion) our x
    s = s.apply(Matrix(((0, 0, 1, 0), (1, 0, 0, 0), (0, 1, 0, 0), (0, 0, 0, 1))))
    return s.move(lx, ly, ST_D + 0.0016)


def st_reflector():
    lx, ly, lw, lh = ST_LENS
    v, f = [], []
    nu, nv = 6, 10
    r = 0.012
    z0 = ST_D + 0.0004
    for i in range(nu + 1):
        x = lx - lw / 2 + 0.003 + (lw - 0.006) * i / nu
        for j in range(nv + 1):
            a = -math.pi / 2 + math.pi * j / nv
            v.append((x, ly + (lh / 2 - 0.002) * math.sin(a), z0 - 0.010 * math.cos(a) ** 0.7))
    for i in range(nu):
        for j in range(nv):
            a = i * (nv + 1) + j
            f.append((a, a + nv + 1, a + nv + 2, a + 1))
    return Shape(v, f)


def st_tube():
    lx, ly, lw, lh = ST_LENS
    t = along_x(cyl(0.0024, h=lw - 0.010, seg=12, bevel_r=0.0015))
    ends = Shape()
    for sx in (-1, 1):
        ends = ends + along_x(cyl(0.0032, h=0.004, seg=12, bevel_r=0.0005)).move(sx * (lw / 2 - 0.006), 0, 0)
    return (t + ends).move(lx, ly - 0.001, ST_D - 0.0015)


auto('Real__strobeHousing', st_housing, 1100)
auto('Real__strobeLens', st_lens, 400)
auto('Real__strobeReflector', st_reflector, 200)
auto('Real__strobeTube', st_tube, 460)
