"""r18: the washroom and the janitor's things, the classroom and the plant
nobody watered (Props P2, docs/PROPS.md).

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is modelled at
its real size in its prop's own frame: x across its front, y up from the
floor, z out of its front. Sizes, sources and what the game does with each:
`r18.notes.md`. Every part is named `Real__<prop><Part>`:

overturnedPail*  a 10 l polypropylene pail standing (Body: outside, rim,
                 bead, ears, foot; Inside: its wall and litre marks;
                 InsideFloor), its wire bail and grip settled on the floor as
                 the pail lies on its side (BailSide, GripSide) or upside
                 down (BailDown, GripDown), in the standing frame
mopBucket*       a 26 qt mop bucket: Tub, Inside, Floor, Water (its surface
                 at the fill), CastorForks, Wheels, the side-press Wringer,
                 its Plate, Lever and Grip; the wet mop standing in it
                 (MopHandle, MopClamp, MopStrings)
schoolDesk*      a combination chair desk: Top (laminate), Edge (T-moulding),
                 Core (the particleboard where a corner went), Frame,
                 Glides, Rack, Seat, Back, Screws; the open book on it
                 (BookPages, BookCover), the pencil (Pencil, PencilWood,
                 PencilLead, PencilFerrule, PencilEraser), the dried apple
                 (Apple, AppleStem)
deadPlant*       a terracotta azalea pot (Pot), its Soil, three woody
                 pelargonium stems (StemA, StemB, StemC), a dried leaf and a
                 living one (LeafDry, LeafLive), a leaf fallen on the floor
                 (Fallen), the leaves on the soil (Litter)
washcloth*       a terry washcloth dropped and folded over: its pile (Terry)
                 and its flat-woven band and hem (Border)
nailPolish*      a 13.5 ml bottle: the Glass and its thick GlassBase, the
                 Lacquer dried in it (LacquerLying when it fell over wet),
                 the Cap, the Brush's stem, the Drip
floss*           a 50 m floss dispenser lying on its back: Body, Lid, Cutter,
                 Thread (the end left out)
"""
from propkit import *
import propkit

# ------------------------------------------------------------------ util --
# the parts whose texture coordinates go round them (a cylinder's), not as
# a box lays them
CYL_FRAMED = {'Real__overturnedPailBody', 'Real__overturnedPailInside', 'Real__deadPlantPot'}


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    kind = 'cyl' if name in CYL_FRAMED else 'box'
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget)


def face_toward(s, want):
    """Turn each face of an open surface to face `want(centroid)` (the side
    it is seen from)."""
    f2 = []
    for f in s.f:
        a, b, c = s.v[f[0]], s.v[f[1]], s.v[f[2]]
        n = (b - a).cross(c - a)
        if len(f) > 3:
            n = n + (c - a).cross(s.v[f[3]] - a)
        cen = sum((s.v[i] for i in f), Vector()) / len(f)
        f2.append(f if n.dot(want(cen)) >= 0 else tuple(reversed(f)))
    s.f = f2
    return s


def toward_axis(c):
    return Vector((-c.x, 0.0, -c.z))


def up(c):
    return Vector((0.0, 1.0, 0.0))


def rot_about_x(s, ang, y0=0.0, z0=0.0):
    """Turn a shape about the line parallel to x through (y0, z0)."""
    return s.move(0, -y0, -z0).rot(x=ang).move(0, y0, z0)


def lowest_y(pts, ang, y0, z0, rot_after=None):
    lo = 1e9
    c, s = math.cos(ang), math.sin(ang)
    for p in pts:
        y, z = p.y - y0, p.z - z0
        yy, zz = y * c - z * s + y0, y * s + z * c + z0
        if rot_after:
            yy = rot_after(Vector((p.x, yy, zz)))
        lo = min(lo, yy)
    return lo


# ================================================================= pail ==
# A 10 l household pail (Queen 10 l: 280 mm across, 230 mm high; the base
# about 212 mm across): polypropylene 2.2 mm thick, a flanged rim, a
# stacking bead under it, two moulded ears for a 3.5 mm wire bail.
P_H = 0.230             # to the top of the rim
P_RB = 0.106            # the wall's radius at the foot
P_RT = 0.1315           # the wall's radius under the rim's flange
P_RL = 0.1405           # the flange's outside
P_WALL = 0.0022
P_EAR_Y = P_H - 0.030   # the bail's pivots
P_EAR_X = 0.146
P_SEG = 40


def _pail_wall_r(y):
    return P_RB + (P_RT - P_RB) * (y - 0.006) / (P_H - 0.016 - 0.006)


def _oval(s):
    # stood on and kicked about: a little out of round
    return s.warp(lambda p: Vector((p.x * 1.018, p.y, p.z * 0.985)))


def pail_body():
    bead_y = P_H - 0.050
    prof = [(0.0, 0.0045), (0.060, 0.0040), (0.094, 0.0030), (0.099, 0.0), (0.103, 0.0), (P_RB, 0.003),
            (_pail_wall_r(0.03), 0.03), (_pail_wall_r(bead_y - 0.006), bead_y - 0.006),
            (_pail_wall_r(bead_y) + 0.0022, bead_y - 0.002), (_pail_wall_r(bead_y) + 0.0022, bead_y + 0.002),
            (_pail_wall_r(bead_y + 0.006), bead_y + 0.006), (P_RT, P_H - 0.016),
            (P_RL - 0.001, P_H - 0.0145), (P_RL, P_H - 0.0125), (P_RL, P_H - 0.003), (P_RL - 0.0025, P_H),
            (P_RT - 0.0015, P_H), (P_RT - P_WALL - 0.0005, P_H - 0.0012)]
    body = lathe(prof, seg=P_SEG, cap_bottom=False, cap_top=False)
    # the ears: a moulded boss each side under the rim, the bail's hole
    # through it, a rib down from it to the bead
    ears = Shape()
    for sx in (-1, 1):
        boss = rbox(0.013, 0.030, 0.028, r=0.004, seg=2).move(sx * (P_RT + 0.004), P_EAR_Y, 0)
        hole = cyl(0.0024, h=0.04, seg=8, bevel_r=0).rot(z=math.pi / 2).move(sx * (P_RT + 0.005), P_EAR_Y, 0)
        boss = cut(boss, hole)
        rib = rbox(0.008, 0.032, 0.006, r=0.0015, seg=1).move(sx * (_pail_wall_r(P_EAR_Y - 0.03) + 0.002), P_EAR_Y - 0.030, 0)
        ears = ears + boss + rib
    # a chip out of the rim where it was dropped
    s = body + ears
    s = cut(s, rbox(0.022, 0.012, 0.03, r=0.002).rot(y=0.3, z=0.4).move(0.07, P_H + 0.001, -0.125))
    return _oval(s)


def pail_inside():
    r_in = lambda y: _pail_wall_r(y) - P_WALL
    prof = [(P_RT - P_WALL - 0.0005, P_H - 0.0012), (P_RT - P_WALL, P_H - 0.016)]
    for y in (0.17, 0.11, 0.05, 0.016):
        prof.append((r_in(y), y))
    prof += [(r_in(0.009) - 0.004, 0.0075)]
    s = lathe(prof, seg=P_SEG, cap_bottom=False, cap_top=False)
    # the litre marks moulded on the inside of the wall, one side: a tick
    # and a longer tick, 2 l apart
    for k, y in enumerate((0.060, 0.098, 0.133, 0.166, 0.196)):
        L = 0.022 if k % 2 else 0.012
        tick = rbox(L, 0.0014, 0.0022, r=0.0005, seg=1)
        s = s + tick.move(0, y, -(r_in(y) - 0.0004))
    return _oval(face_toward(s, toward_axis))


def pail_inside_floor():
    r = _pail_wall_r(0.009) - P_WALL - 0.004
    return _oval(face_toward(lathe([(0.0, 0.0075), (r, 0.0075)], seg=P_SEG, cap_bottom=False, cap_top=False), up))


# the bail: 3.5 mm wire hooked into the ears, its grip a moulded sleeve
B_APEX = 0.165          # above the ears, standing up


def _bail_path(n=18):
    pts = []
    # each end turned in through its ear
    pts.append(Vector((-P_RT - 0.001, P_EAR_Y, 0)))
    for i in range(n + 1):
        t = math.pi * i / n
        x = -P_EAR_X * math.cos(t)
        y = P_EAR_Y + 0.008 + (B_APEX - 0.008) * math.sin(t) ** 0.85
        pts.append(Vector((x, y, 0)))
    pts.append(Vector((P_RT + 0.001, P_EAR_Y, 0)))
    pts[1].y = P_EAR_Y
    pts[-2].y = P_EAR_Y
    return pts


def _bail():
    return tube(_bail_path(), 0.00175, seg=6, caps=True)


def _grip():
    prof = [(0.0, -0.055), (0.0085, -0.055), (0.0105, -0.045), (0.0095, -0.02), (0.0088, 0.0), (0.0095, 0.02),
            (0.0105, 0.045), (0.0085, 0.055), (0.0, 0.055)]
    g = lathe(prof, seg=12)
    g = cut(g, cyl(0.0025, h=0.13, seg=8, bevel_r=0))           # the wire runs through it
    return g.rot(z=math.pi / 2).move(0, P_EAR_Y + B_APEX + 0.001, 0)


def _settle_bail(floor_fn, hi_a):
    """The angle about the ears at which the bail and its grip come to rest
    on the floor (the lowest point of either at the floor), between 0 and
    hi_a, from the side where it is clear of the floor."""
    pts = _bail().v + _grip().v
    clear = lambda a: lowest_y(pts, a, P_EAR_Y, 0.0, floor_fn) > 0.0
    a0, a1 = 0.0, hi_a
    c0 = clear(a0)
    for _ in range(40):
        mid = (a0 + a1) / 2
        if clear(mid) == c0:
            a0 = mid
        else:
            a1 = mid
    return a0 if c0 else a1


# lying on its side: turned about x by PAIL_TILT (its mouth to +z) so the
# rim's flange and the foot both touch the floor, lifted PAIL_LIFT: the line
# under its +z side through the foot's and the flange's outermost points
def _pail_rest():
    vs = pail_body().v
    foot = max((p for p in vs if p.y < 0.01 and abs(p.x) < 0.01), key=lambda p: p.z)
    rim = max((p for p in vs if p.y > P_H - 0.02 and abs(p.x) < 0.01), key=lambda p: p.z)
    tilt = math.atan2(rim.y - foot.y, rim.z - foot.z)
    c, s_ = math.cos(tilt), math.sin(tilt)
    lift = -min(p.y * c - p.z * s_ for p in vs)
    return tilt, lift


PAIL_TILT, PAIL_LIFT = _pail_rest()


def _side_floor(p):
    c, s = math.cos(PAIL_TILT), math.sin(PAIL_TILT)
    return p.y * c - p.z * s + PAIL_LIFT


def _down_floor(p):
    return P_H - p.y


SIDE_SWING = None
DOWN_SWING = None


def pail_bail(which, grip=False):
    global SIDE_SWING, DOWN_SWING
    if which == 'side':
        if SIDE_SWING is None:
            SIDE_SWING = _settle_bail(_side_floor, math.pi / 2)
        a = SIDE_SWING
    else:
        if DOWN_SWING is None:
            DOWN_SWING = _settle_bail(_down_floor, math.pi / 2)
        a = DOWN_SWING
    s = _grip() if grip else _bail()
    return rot_about_x(s, a, P_EAR_Y, 0.0)


# ---------------------------------------------------------------- parts ---
auto('Real__overturnedPailBody', pail_body, 2400)
auto('Real__overturnedPailInside', pail_inside, 1300)
auto('Real__overturnedPailInsideFloor', pail_inside_floor, 120)
auto('Real__overturnedPailBailSide', lambda: pail_bail('side'), 500)
auto('Real__overturnedPailGripSide', lambda: pail_bail('side', True), 300)
auto('Real__overturnedPailBailDown', lambda: pail_bail('down'), 500)
auto('Real__overturnedPailGripDown', lambda: pail_bail('down', True), 300)
print('r18 pail: tilt %.5f lift %.5f side swing %.5f down swing %.5f' % (PAIL_TILT, PAIL_LIFT, SIDE_SWING or 0, DOWN_SWING or 0))


# =========================================================== mop bucket ==
# A 26 qt commercial mop bucket of the Rubbermaid WaveBrake class: a
# moulded polyethylene tub on four 3 in castors, a side-press wringer
# hooked over one end, a 24 oz cotton wet mop on a 54 in wooden handle.
M_Y0 = 0.108            # the tub's underside
M_Y1 = 0.362            # its rim
M_BOT = (0.385, 0.300, 0.060)   # the tub at its foot: length (x), width (z), corner radius
M_TOP = (0.460, 0.372, 0.075)   # at its rim
M_WALL = 0.004
M_NC = 5                # points a corner


def rr_ring(w, d, r, y, n=M_NC):
    """A rounded rectangle's outline at height y, counter-clockwise from
    above, `4 * (n + 1)` points, starting at the +x +z corner's start."""
    pts = []
    cs = [(w / 2 - r, d / 2 - r, 0.0), (-w / 2 + r, d / 2 - r, math.pi / 2), (-w / 2 + r, -d / 2 + r, math.pi), (w / 2 - r, -d / 2 + r, 1.5 * math.pi)]
    for cx, cz, a0 in cs:
        for i in range(n + 1):
            a = a0 + (math.pi / 2) * i / n
            pts.append(Vector((cx + r * math.cos(a), y, cz + r * math.sin(a))))
    return pts


def loft(rings, cap0=None, cap1=None):
    """Quads between consecutive rings (same count); optional fan caps at
    the first/last ring to a centre point."""
    v, f = [], []
    n = len(rings[0])
    for r in rings:
        v += [p.copy() for p in r]
    for k in range(len(rings) - 1):
        for i in range(n):
            j = (i + 1) % n
            f.append((k * n + i, k * n + j, (k + 1) * n + j, (k + 1) * n + i))
    if cap0 is not None:
        c = len(v)
        v.append(Vector(cap0))
        for i in range(n):
            f.append((c, (i + 1) % n, i))
    if cap1 is not None:
        c = len(v)
        v.append(Vector(cap1))
        base = (len(rings) - 1) * n
        for i in range(n):
            f.append((c, base + i, base + (i + 1) % n))
    return Shape(v, f)


def _m_lerp(t):
    return tuple(a + (b - a) * t for a, b in zip(M_BOT, M_TOP))


def _m_ring(y, grow=0.0, n=M_NC):
    t = (y - M_Y0) / (M_Y1 - M_Y0)
    w, d, r = _m_lerp(t)
    return rr_ring(w + 2 * grow, d + 2 * grow, r + grow, y, n)


def mop_tub():
    ys = [M_Y0 + 0.004, M_Y0 + 0.012, M_Y0 + 0.06, 0.20, 0.27, M_Y1 - 0.035, M_Y1 - 0.030, M_Y1 - 0.025, M_Y1 - 0.020]
    rings = [_m_ring(M_Y0, -0.006)]
    for y in ys:
        g = 0.0
        if M_Y1 - 0.035 < y < M_Y1 - 0.02:
            g = 0.003          # a stacking bead
        rings.append(_m_ring(y, g))
    # the rim: rolled over outward and down, its edge, back in at its top
    rings.append(_m_ring(M_Y1 - 0.016, 0.012))
    rings.append(_m_ring(M_Y1 - 0.004, 0.0135))
    rings.append(_m_ring(M_Y1, 0.010))
    rings.append(_m_ring(M_Y1, -M_WALL + 0.0005))
    s = propkit._fix_winding(loft(rings, cap0=(0, M_Y0 + 0.002, 0)))
    # the moulded grip under the rim at each end, and the bosses under the
    # floor that hold the castors' stems
    w0, d0, _ = M_BOT
    for sx in (-1, 1):
        grip = rbox(0.016, 0.022, 0.16, r=0.006, seg=2).move(sx * (M_TOP[0] / 2 + 0.004), M_Y1 - 0.034, 0)
        s = s + grip
    for sx in (-1, 1):
        for sz in (-1, 1):
            boss = cyl(0.021, 0.019, 0.016, seg=14, bevel_r=0.003).move(sx * (w0 / 2 - 0.045), M_Y0 - 0.006, sz * (d0 / 2 - 0.045))
            s = s + boss
    return s


def mop_inside():
    rings = [_m_ring(M_Y1, -M_WALL + 0.0005), _m_ring(M_Y1 - 0.006, -M_WALL)]
    for y in (0.30, 0.24, 0.18, M_Y0 + 0.03, M_Y0 + 0.012):
        rings.append(_m_ring(y, -M_WALL))
    rings.append(_m_ring(M_Y0 + 0.006, -M_WALL - 0.008))
    s = loft(rings)
    # the fill lines moulded on the far end: 3, 4, 5 gal
    for k, y in enumerate((0.20, 0.245, 0.29)):
        w, d, r = _m_lerp((y - M_Y0) / (M_Y1 - M_Y0))
        tick = rbox(0.0025, 0.0016, 0.05 if k != 1 else 0.08, r=0.0006, seg=1).move(-(w / 2 - M_WALL - 0.0008), y, 0)
        s = s + tick
    return face_toward(s, toward_axis)


def mop_floor():
    y = M_Y0 + 0.006
    return face_toward(loft([_m_ring(y, -M_WALL - 0.008)], cap1=(0, y, 0)), up)


def mop_water():
    y = 0.262
    return face_toward(loft([_m_ring(y, -M_WALL - 0.0008)], cap1=(0, y, 0)), up)


CASTORS = [(sx * (M_BOT[0] / 2 - 0.045), sz * (M_BOT[1] / 2 - 0.045)) for sx in (-1, 1) for sz in (-1, 1)]
CASTOR_TURN = [0.4, 2.1, -1.2, 3.0]     # each swivelled its own way
WHEEL_R, WHEEL_W = 0.038, 0.024


def castor_fork(k):
    x, z = CASTORS[k]
    top = M_Y0 - 0.014
    plate = cyl(0.022, 0.024, 0.008, seg=12, bevel_r=0.0015, bevel_seg=1).move(0, top - 0.004, 0)
    race = cyl(0.019, 0.019, 0.006, seg=12, bevel_r=0.001, bevel_seg=1).move(0, top - 0.011, 0)
    # the fork: two legs down to the axle, set back (the swivel's lead)
    ax_y, lead = WHEEL_R + 0.001, 0.022
    legs = Shape()
    for sz in (-1, 1):
        leg = tube([Vector((0, top - 0.013, sz * 0.016)), Vector((lead * 0.5, top - 0.03, sz * 0.0165)), Vector((lead, ax_y, sz * 0.0165))], 0.0045, seg=6)
        legs = legs + leg
    bridge = rbox(0.036, 0.004, 0.038, r=0.0015, seg=1).move(0.002, top - 0.015, 0)
    axle = cyl(0.0045, h=0.044, seg=8, bevel_r=0.001).rot(x=math.pi / 2).move(lead, ax_y, 0)
    s = plate + race + bridge + legs + axle
    return s.rot(y=CASTOR_TURN[k]).move(x, 0, z)


def castor_wheel(k):
    x, z = CASTORS[k]
    lead = 0.022
    prof = [(0.0, -WHEEL_W / 2), (WHEEL_R - 0.012, -WHEEL_W / 2), (WHEEL_R - 0.004, -WHEEL_W / 2 + 0.001),
            (WHEEL_R, -WHEEL_W / 2 + 0.006), (WHEEL_R, WHEEL_W / 2 - 0.006), (WHEEL_R - 0.004, WHEEL_W / 2 - 0.001),
            (WHEEL_R - 0.012, WHEEL_W / 2), (0.0, WHEEL_W / 2)]
    w = lathe(prof, seg=20)
    # its hub, sunk
    w = cut(w, cyl(0.014, h=0.006, seg=12, bevel_r=0).move(0, WHEEL_W / 2, 0), cyl(0.014, h=0.006, seg=12, bevel_r=0).move(0, -WHEEL_W / 2, 0))
    w = w.rot(x=math.pi / 2).move(lead, WHEEL_R + 0.001, 0)
    return w.rot(y=CASTOR_TURN[k]).move(x, 0, z)


def mop_castor_forks():
    return sum((castor_fork(k) for k in range(4)), Shape())


def mop_wheels():
    return sum((castor_wheel(k) for k in range(4)), Shape())


# the wringer: a moulded hopper hooked over the +x end
W_X0, W_X1 = M_TOP[0] / 2 - 0.205, M_TOP[0] / 2 + 0.006
W_Z = 0.150             # its half width
W_Y0, W_Y1 = M_Y1 - 0.105, M_Y1 + 0.205
W_T = 0.007


def mop_wringer():
    xc, zc = (W_X0 + W_X1) / 2, 0.0
    lx = W_X1 - W_X0
    h = W_Y1 - W_Y0
    outer = rbox(lx, h, 2 * W_Z, r=0.018, seg=3)
    inner = rbox(lx - 2 * W_T, h + 0.05, 2 * W_Z - 2 * W_T, r=0.012, seg=2)
    s = cut(outer, inner)
    # a hopper: narrower toward its foot, down inside the tub
    s = s.warp(lambda p: Vector((p.x * (0.86 + 0.14 * (p.y + h / 2) / h), p.y, p.z * (0.9 + 0.1 * (p.y + h / 2) / h))))
    s = s.move(xc, (W_Y0 + W_Y1) / 2, zc)
    # its top flared out into a lip, the side ribs, the hooks over the rim
    lip = cut(rbox(lx + 0.02, 0.014, 2 * W_Z + 0.02, r=0.006, seg=2), rbox(lx - 2 * W_T, 0.05, 2 * W_Z - 2 * W_T, r=0.012, seg=2))
    s = s + lip.move(xc, W_Y1 - 0.007, zc)
    for k in range(4):
        x = W_X0 + 0.03 + k * (lx - 0.06) / 3
        for sz in (-1, 1):
            s = s + rbox(0.010, h - 0.06, 0.008, r=0.003, seg=1).move(x, (W_Y0 + W_Y1) / 2 + 0.01, sz * (W_Z + 0.003))
    for sz in (-1, 1):
        hook = rbox(0.05, 0.05, 0.012, r=0.004, seg=1).move(W_X0 + 0.04, M_Y1 - 0.01, sz * (W_Z - 0.02))
        s = s + hook
    # the drain grid across its foot
    for k in range(5):
        x = W_X0 + 0.025 + k * (lx - 0.05) / 4
        s = s + rbox(0.008, 0.012, 2 * W_Z - 0.01, r=0.002, seg=1).move(x, W_Y0 + 0.008, zc)
    # the lever's pivot boss on the +z side
    s = s + cyl(0.026, h=0.02, seg=16, bevel_r=0.003).rot(x=math.pi / 2).move(xc, W_Y1 - 0.06, W_Z + 0.01)
    return s


LEVER = (W_X0 + W_X1) / 2, W_Y1 - 0.06, W_Z + 0.03   # the pivot
LEVER_TOP = 0.865


def mop_plate():
    # the press plate inside: a perforated grey moulding
    xc = (W_X0 + W_X1) / 2
    plate = rbox(W_X1 - W_X0 - 0.03, 0.20, 0.010, r=0.003, seg=1)
    for i in range(3):
        for j in range(4):
            plate = cut(plate, rbox(0.022, 0.03, 0.03, r=0.006, seg=1).move(-0.05 + i * 0.05, -0.06 + j * 0.04, 0))
    return plate.move(xc, W_Y1 - 0.14, W_Z - 0.035)


def mop_lever():
    # the steel lever up the side from the pivot, its cam strap and bolt
    px, py, pz = LEVER
    lever = tube([Vector((px, py, pz)), Vector((px + 0.004, py + 0.12, pz + 0.004)), Vector((px + 0.012, LEVER_TOP - 0.06, pz + 0.008))], 0.0095, seg=10)
    strap = rbox(0.05, 0.08, 0.006, r=0.002, seg=1).move(px, py + 0.01, pz - 0.006)
    bolt = cyl(0.008, h=0.012, seg=6, bevel_r=0.001).rot(x=math.pi / 2).move(px, py, pz + 0.004)
    return lever + strap + bolt


def mop_grip():
    px, py, pz = LEVER
    prof = [(0.0, 0.0), (0.013, 0.0), (0.015, 0.012), (0.0135, 0.05), (0.0145, 0.09), (0.016, 0.108), (0.0, 0.112)]
    g = lathe(prof, seg=14)
    return g.move(px + 0.012, LEVER_TOP - 0.11, pz + 0.008)


# the mop: its head on the tub's floor at the -x end, the handle leaning on
# the wringer's near wall
MOP_FOOT = Vector((-0.085, M_Y0 + 0.085, -0.045))
MOP_REST = Vector((W_X0 - 0.012, W_Y1 + 0.012, -0.075))
MOP_LEN = 1.37
MOP_DIR = (MOP_REST - MOP_FOOT).normalized()


def mop_handle():
    prof = [(0.0, 0.0), (0.0105, 0.0), (0.0118, 0.004), (0.0118, MOP_LEN - 0.03), (0.0105, MOP_LEN - 0.006), (0.006, MOP_LEN), (0.0, MOP_LEN)]
    s = lathe(prof, seg=12)
    # the hanging hole at its top
    s = cut(s, cyl(0.004, h=0.04, seg=8, bevel_r=0).rot(x=math.pi / 2).move(0, MOP_LEN - 0.03, 0))
    q = Vector((0, 1, 0)).rotation_difference(MOP_DIR)
    return s.apply(q.to_matrix().to_4x4()).move(*MOP_FOOT)


def _mop_frame():
    up = MOP_DIR
    side = Vector((0, 0, 1)).cross(up).normalized()
    fwd = up.cross(side).normalized()
    return up, side, fwd


def mop_clamp():
    # a quick-change head clamp: a moulded jaw round the headband, its
    # collar on the handle's foot
    up, side, fwd = _mop_frame()
    jaw = rbox(0.050, 0.035, 0.135, r=0.008, seg=2)
    collar = cyl(0.016, 0.018, 0.06, seg=12, bevel_r=0.003).move(0, 0.045, 0)
    wing = cyl(0.008, h=0.03, seg=8, bevel_r=0.002).rot(z=math.pi / 2).move(0.025, 0.03, 0)
    s = jaw + collar + wing
    m = Matrix((side.to_4d(), up.to_4d(), fwd.to_4d(), (0, 0, 0, 1))).transposed() if False else None
    q = Vector((0, 1, 0)).rotation_difference(MOP_DIR)
    return s.apply(q.to_matrix().to_4x4()).move(*(MOP_FOOT + MOP_DIR * -0.02))


def mop_strings():
    """The cotton yarns, 6 mm, from both sides of the headband: up over its
    edge, down and lying heaped on the tub's floor, matted together."""
    rng = random.Random(1826)
    q = Vector((0, 1, 0)).rotation_difference(MOP_DIR)
    band = MOP_FOOT + MOP_DIR * -0.035
    floor_y = M_Y0 + 0.006
    w, d, _ = M_BOT
    xin, zin = w / 2 - 0.022, d / 2 - 0.022
    s = Shape()
    # the matted mass under the loose yarns
    mass = sphere(1.0, seg=14, rings=8).scale(0.10, 0.028, 0.085)
    mass = noisy(mass, 0.012, scale=14.0, seed=3)
    mass = mass.warp(lambda p: Vector((p.x, max(p.y, -0.026) * (1.0 if p.y > 0 else 0.3), p.z)))
    s = s + mass.move(band.x + 0.01, floor_y + 0.010, band.z)
    n = 46
    for i in range(n):
        side = -1 if i % 2 else 1
        zz = rng.uniform(-0.066, 0.066)
        start = band + q @ Vector((side * 0.010, 0.0, zz))
        heading = Vector((side * 1.0, 0.0, zz * 7.0 + rng.uniform(-0.7, 0.7))).normalized()
        L = 0.30 + rng.uniform(-0.07, 0.05)
        pts = []
        nseg = 6
        for k in range(nseg + 1):
            t = k / nseg
            sl = L * t
            out = max(0.0, sl - 0.015)
            p = Vector((start.x + heading.x * out * 0.95, 0.0, start.z + heading.z * out * 0.95))
            # over the band's edge and down
            hump = 0.018 * math.sin(math.pi * min(1.0, sl / 0.07))
            fall = start.y + hump - (start.y - floor_y) * min(1.0, sl / 0.08) ** 1.5
            dist = (p.xz - band.xz).length
            heap = 0.034 * math.exp(-dist / 0.075) * (0.55 + 0.45 * noise.noise(Vector((p.x * 30, p.z * 30, i * 0.37))))
            y = max(fall, floor_y + 0.005 + heap + 0.0025 * (i % 3))
            p.x += 0.008 * noise.noise(Vector((i * 1.3, t * 4.0, 0.5)))
            p.z += 0.008 * noise.noise(Vector((i * 1.3, t * 4.0, 7.5)))
            p.x = max(-xin, min(xin, p.x))
            p.z = max(-zin, min(zin, p.z))
            p.y = y
            pts.append(p)
        pts[0] = start
        s = s + tube(pts, 0.0068, seg=4, caps=False, r_end=0.0052)
    return s


auto('Real__mopBucketTub', mop_tub, 2400)
auto('Real__mopBucketInside', mop_inside, 1400)
auto('Real__mopBucketFloor', mop_floor, 120)
auto('Real__mopBucketWater', mop_water, 120)
auto('Real__mopBucketCastorForks', mop_castor_forks, 1800)
auto('Real__mopBucketWheels', mop_wheels, 1700)
auto('Real__mopBucketWringer', mop_wringer, 2400)
auto('Real__mopBucketPlate', mop_plate, 1200)
auto('Real__mopBucketLever', mop_lever, 400)
auto('Real__mopBucketGrip', mop_grip, 300)
auto('Real__mopBucketMopHandle', mop_handle, 400)
auto('Real__mopBucketMopClamp', mop_clamp, 600)
auto('Real__mopBucketMopStrings', mop_strings, 2600)


# ========================================================== school desk ==
# A combination chair desk of the Virco 9400BR class: a 24 x 18 in top of
# laminate on 1 1/8 in particleboard in a T-moulded edge, at 30 in; a seat
# and back of moulded plywood; a frame of 7/8 in steel tube on two floor
# runners, a steel-rod book rack under the seat; the student sits at -z.
D_W, D_D, D_T = 0.610, 0.457, 0.029
D_Y = 0.745             # the top's surface
D_TUBE = 0.0111         # the frame's tube radius
D_SX = 0.212            # the side frames
S_Y = 0.440             # the seat's top
S_Z0, S_Z1 = -0.215, -0.575     # the seat's front and back
CHIP = (-D_W / 2, D_D / 2)      # the corner knocked off (front left)


def _chip_cutter():
    return rbox(0.07, 0.05, 0.05, r=0.004, seg=1).rot(y=0.7).move(CHIP[0] + 0.004, D_Y - D_T / 2 + 0.006, CHIP[1] - 0.004)


def desk_top():
    # (the carving in it is painted: `place_school_desks`)
    s = rbox(D_W - 0.006, D_T - 0.002, D_D - 0.006, r=0.0015, seg=1).move(0, D_Y - D_T / 2, 0)
    return cut(s, _chip_cutter())


def desk_edge():
    # the T-moulding: a rounded band 1 mm proud round the top's edge
    outer = rounded_rect(D_W, D_D, 0.012, seg=3)
    inner = rounded_rect(D_W - 0.008, D_D - 0.008, 0.008, seg=3)
    s = cut(prism(outer, D_T, bevel_r=0.002, bevel_seg=2), prism(inner, D_T * 2))
    s = s.rot(x=-math.pi / 2).move(0, D_Y - D_T / 2, 0)
    return cut(s, _chip_cutter())


def desk_core():
    # the particleboard where the corner went: rough, swollen a little
    c = _chip_cutter()
    s = boolean(rbox(D_W - 0.006, D_T - 0.004, D_D - 0.006, r=0.001, seg=1).move(0, D_Y - D_T / 2 - 0.0005, 0), c, 'INTERSECT')
    s = boolean(s, rbox(D_W - 0.012, D_T, D_D - 0.012, r=0.001, seg=1).move(0, D_Y - D_T / 2, 0), 'DIFFERENCE')
    return noisy(s, 0.0012, scale=300.0, seed=5) if s.v else box(0.01, 0.01, 0.01).move(CHIP[0] + 0.01, D_Y - 0.01, CHIP[1] - 0.01)


def bent(pts, r, n=4):
    """A polyline with each inner corner bent round on a radius about r
    (as a tube bender leaves it)."""
    pts = [Vector(p) for p in pts]
    out = [pts[0]]
    for i in range(1, len(pts) - 1):
        a, p, b = pts[i - 1], pts[i], pts[i + 1]
        la, lb = (a - p).length, (b - p).length
        k = min(r, la * 0.45, lb * 0.45)
        p0, p1 = p + (a - p).normalized() * k, p + (b - p).normalized() * k
        for j in range(n + 1):
            t = j / n
            out.append(p0 * (1 - t) ** 2 + p * 2 * t * (1 - t) + p1 * t * t)
    out.append(pts[-1])
    return out


def _ftube(pts, r=0.045):
    return tube(bent(pts, r), D_TUBE, seg=10, caps=True)


def desk_frame():
    s = Shape()
    ty = D_Y - D_T - D_TUBE        # the rail under the top
    fy = 0.012 + D_TUBE            # the runners
    sy = S_Y - 0.0115 - D_TUBE     # the seat rails
    for sx in (-1, 1):
        x = sx * D_SX
        # the runner from the front foot back to the rear post, and up it to
        # carry the back
        s = s + _ftube([(x, fy, 0.165), (x, fy, S_Z1 + 0.02), (x, 0.42, S_Z1 - 0.015), (x, 0.80, S_Z1 - 0.075)], 0.05)
        # the front post up under the top, the rail back, the bridge down to
        # the seat rail and along it to the rear post
        s = s + _ftube([(x, fy, 0.165 - 0.0), (x, ty, 0.150), (x, ty, -0.10), (x, sy, S_Z0 + 0.03), (x, sy, S_Z1 + 0.005)], 0.045)
    # cross tubes: under the top's front, under the seat front and back
    for (y, z) in ((ty, 0.12), (sy, S_Z0 + 0.04), (sy, S_Z1 + 0.06)):
        s = s + cyl(D_TUBE * 0.85, h=2 * D_SX, seg=10, bevel_r=0).rot(z=math.pi / 2).move(0, y, z)
    # the stamped plates the top is screwed to
    for sx in (-1, 1):
        s = s + rbox(0.036, 0.004, 0.25, r=0.0015, seg=1).move(sx * D_SX, ty + D_TUBE + 0.002, 0.02)
    return s


def desk_glides():
    s = Shape()
    for sx in (-1, 1):
        for z in (0.165, S_Z1 + 0.03):
            g = cyl(0.014, 0.0155, 0.012, seg=10, bevel_r=0.002).move(sx * D_SX, 0.006, z)
            s = s + g
    return s


def desk_rack():
    # steel rod (6 mm) book rack under the seat
    s = Shape()
    z0, z1 = S_Z0 - 0.02, S_Z1 + 0.09
    y0 = 0.22
    for k in range(7):
        z = z0 + (z1 - z0) * k / 6
        s = s + tube(bent([(-D_SX + D_TUBE, S_Y - 0.03, z), (-D_SX + 0.03, y0, z), (D_SX - 0.03, y0, z), (D_SX - D_TUBE, S_Y - 0.03, z)], 0.025, 3), 0.003, seg=6)
    for z in (z0, z1):
        s = s + tube([Vector((-D_SX + 0.03, y0 + 0.004, z0 - 0.01)), Vector((-D_SX + 0.03, y0 + 0.004, z1 + 0.01))], 0.003, seg=6)
        s = s + tube([Vector((D_SX - 0.03, y0 + 0.004, z0 - 0.01)), Vector((D_SX - 0.03, y0 + 0.004, z1 + 0.01))], 0.003, seg=6)
        break
    return s


def _ply(w, d, t, nx=10, nz=10):
    return slab(w, d, t, nx, nz).rot(x=-math.pi / 2)


def desk_seat():
    # moulded plywood: dished a little, the front edge rolled down
    w, d = 0.445, abs(S_Z1 - S_Z0) + 0.02
    s = _ply(w, d, 0.011, 12, 10)
    s = s.warp(lambda p: Vector((p.x, p.y - 0.010 * (1 - (2 * p.x / w) ** 2) * (1 - (2 * p.z / d) ** 2) - (0.022 * ((p.z / (d / 2) - 0.7) / 0.3) ** 2 if p.z > d / 2 * 0.7 else 0.0), p.z)))
    s = bevel(s, 0.003, seg=1, angle=50)
    s = s.warp(lambda p: Vector((p.x * (1 - 0.06 * max(0.0, -p.z / (d / 2))), p.y, p.z)))
    return s.move(0, S_Y - 0.0055, (S_Z0 + S_Z1) / 2)


def desk_back():
    w, h = 0.445, 0.19
    s = _ply(w, h, 0.010, 12, 6).rot(x=math.pi / 2)
    R = 0.55
    s = s.warp(lambda p: Vector((R * math.sin(p.x / R), p.y, p.z + R * (1 - math.cos(p.x / R)))))
    s = bevel(s, 0.003, seg=1, angle=50)
    return s.rot(x=-0.155).move(0, 0.705, S_Z1 - 0.050 + 0.016)


def desk_screws():
    s = Shape()
    for sx in (-1, 1):
        for z in (-0.08, 0.12):
            s = s + screw_head(0.0045, 0.002, slot=False).rot(x=math.pi).move(sx * D_SX, D_Y - D_T - 0.0035, z)
        for z in (S_Z0 - 0.04, S_Z1 + 0.09):
            s = s + screw_head(0.0045, 0.002, slot=False).rot(x=math.pi).move(sx * D_SX, S_Y - 0.0115, z)
        s = s + screw_head(0.0045, 0.002, slot=False).rot(x=math.pi / 2 - 0.155).move(sx * (D_SX - 0.001), 0.70, S_Z1 - 0.050 + 0.016 + 0.006)
    return s


# the open book: a hardback textbook, 168 x 240 mm, open at its middle
BK_W, BK_H, BK_T = 0.168, 0.240, 0.011


def hblock(nx, nz, w, d, top, bottom=0.0):
    """A closed block over [-w/2, w/2] x [-d/2, d/2] whose top is the
    heightfield top(x, z) and whose bottom is flat at `bottom`."""
    v, f = [], []
    for j in range(nz + 1):
        for i in range(nx + 1):
            x, z = -w / 2 + w * i / nx, -d / 2 + d * j / nz
            v.append(Vector((x, top(x, z), z)))
    k = nx + 1
    for j in range(nz):
        for i in range(nx):
            a = j * k + i
            f.append((a, a + k, a + k + 1, a + 1))
    rim = [i for i in range(nx + 1)] + [j * k + nx for j in range(1, nz + 1)] + [nz * k + i for i in range(nx - 1, -1, -1)] + [j * k for j in range(nz - 1, 0, -1)]
    base = len(v)
    for idx in rim:
        p = v[idx]
        v.append(Vector((p.x, bottom, p.z)))
    m = len(rim)
    for a in range(m):
        b = (a + 1) % m
        f.append((rim[b], rim[a], base + a, base + b))
    f.append(tuple(base + a for a in range(m)))
    return _fix_winding_shape(Shape(v, f))


def _fix_winding_shape(s):
    return propkit._fix_winding(s)


def _page_top(x, z):
    ax = abs(x)
    # the gutter, the pages rising out of it and easing down to the fore-edge
    rise = 1 - math.exp(-ax / 0.022)
    fall = 0.0018 * (ax / BK_W) ** 3
    return 0.0035 + BK_T * rise - fall + 0.0006 * math.sin(z * 40 + x * 13)


def desk_book_pages():
    s = hblock(28, 6, 2 * BK_W, BK_H, _page_top, 0.0030)
    return s.move(0, D_Y, 0.0)


def desk_book_cover():
    w, d = 2 * BK_W + 0.008, BK_H + 0.006
    s = _ply(w, d, 0.0028, 16, 2)
    s = s.warp(lambda p: Vector((p.x, p.y + 0.0014 + 0.0015 * min(1.0, abs(p.x) / 0.01) - 0.0004, p.z)))
    return s.move(0, D_Y, 0.0)


# the pencil: a yellow HB, sharpened down to 120 mm, lying on a flat
PEN_FLAT = 0.0035       # half across its flats


def _hexprism(L, r_flat):
    R = r_flat / math.cos(math.pi / 6)
    out = [(R * math.cos(math.pi / 6 + i * math.pi / 3), R * math.sin(math.pi / 6 + i * math.pi / 3)) for i in range(6)]
    s = prism(out, L, bevel_r=0.0004, bevel_seg=1)
    return s.rot(y=math.pi / 2)


def desk_pencil():
    return _hexprism(0.088, PEN_FLAT).move(0, PEN_FLAT, 0)


def desk_pencil_wood():
    prof = [(0.0, 0.0), (PEN_FLAT * 1.08, 0.0), (0.0012, 0.0175), (0.0, 0.0177)]
    s = lathe(prof, seg=12, cap_bottom=True, cap_top=False)
    # the knife's facets: the cone cut in six flat scallops at its foot
    s = s.warp(lambda p: Vector((p.x, p.y, p.z)))
    return s.rot(z=-math.pi / 2).move(0.044, PEN_FLAT, 0)


def desk_pencil_lead():
    prof = [(0.0, 0.0), (0.0012, 0.0), (0.0004, 0.004), (0.0, 0.0042)]
    return lathe(prof, seg=8).rot(z=-math.pi / 2).move(0.044 + 0.0172, PEN_FLAT, 0)


def desk_pencil_ferrule():
    prof = [(0.0, 0.0), (0.0037, 0.0), (0.0039, 0.001), (0.0036, 0.0025), (0.0039, 0.004), (0.0036, 0.0055), (0.0039, 0.007), (0.0038, 0.0095), (0.0, 0.0095)]
    s = lathe(prof, seg=12)
    return s.rot(z=math.pi / 2).move(-0.044 + 0.0005, PEN_FLAT, 0)


def desk_pencil_eraser():
    prof = [(0.0, 0.0), (0.0033, 0.0), (0.0033, 0.0035), (0.0026, 0.0047), (0.0, 0.0050)]
    s = lathe(prof, seg=10)
    return s.rot(z=math.pi / 2).move(-0.044 - 0.0085, PEN_FLAT, 0).warp(lambda p: Vector((p.x, p.y, p.z)) if p.x > -0.0535 else Vector((max(p.x, -0.0565 + 0.001 * p.y / PEN_FLAT), p.y, p.z)))


# the apple, left on the desk: dried to a wrinkled brown husk
def desk_apple():
    s = sphere(0.026, seg=18, rings=12).scale(1.0, 0.82, 0.96)
    # the wrinkles: folds pushed in, deeper where it shrank most
    def wr(p):
        n = p.normalized()
        f1 = noise.noise(n * 5.0 + Vector((1.3, 2.1, 0.7)))
        f2 = noise.noise(n * 11.0 + Vector((4.2, 0.3, 5.1)))
        d = -0.0035 * abs(f1) - 0.0018 * abs(f2) - 0.0025 * max(0.0, -n.y) ** 2
        # its stem's hollow, the calyx under it
        d -= 0.006 * max(0.0, n.y - 0.85) / 0.15 + 0.004 * max(0.0, -n.y - 0.85) / 0.15
        return p + n * d
    s = s.warp(wr)
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


def desk_apple_stem():
    lo_top = 0.026 * 0.82 - 0.006
    return tube([Vector((0, lo_top - 0.004, 0)), Vector((0.001, lo_top + 0.006, 0.0005)), Vector((0.003, lo_top + 0.013, 0.001))], 0.0012, seg=5, r_end=0.0009).move(0, 0.026 * 0.82 - 0.004, 0)


auto('Real__schoolDeskTop', desk_top, 600)
auto('Real__schoolDeskEdge', desk_edge, 700)
auto('Real__schoolDeskCore', desk_core, 300)
auto('Real__schoolDeskFrame', desk_frame, 2600)
auto('Real__schoolDeskGlides', desk_glides, 500)
auto('Real__schoolDeskRack', desk_rack, 1200)
auto('Real__schoolDeskSeat', desk_seat, 900)
auto('Real__schoolDeskBack', desk_back, 600)
auto('Real__schoolDeskScrews', desk_screws, 1000)
auto('Real__schoolDeskBookPages', desk_book_pages, 800)
auto('Real__schoolDeskBookCover', desk_book_cover, 300)
auto('Real__schoolDeskPencil', desk_pencil, 120)
auto('Real__schoolDeskPencilWood', desk_pencil_wood, 60)
auto('Real__schoolDeskPencilLead', desk_pencil_lead, 40)
auto('Real__schoolDeskPencilFerrule', desk_pencil_ferrule, 220)
auto('Real__schoolDeskPencilEraser', desk_pencil_eraser, 80)
auto('Real__schoolDeskApple', desk_apple, 500)
auto('Real__schoolDeskAppleStem', desk_apple_stem, 60)


# =========================================================== dead plant ==
# Zonal pelargoniums (geraniums) in a 31 cm terracotta azalea pot (Deroma
# azalea half pot: 315 mm across, 193 mm high), not watered for years: the
# stems gone woody and grey, knuckled at every node, a few leaves dried on
# them, the rest fallen; the soil shrunk from the pot's wall and cracked.
PT_H = 0.193
PT_RT, PT_RB = 0.1575, 0.112
PT_RIM = 0.034          # the rim's collar
PT_WALL = 0.009
SOIL_Y = 0.150          # the soil's surface, shrunk down


def _pt_r(y):
    """The pot's outside radius under the collar."""
    yr = PT_H - PT_RIM
    return PT_RB + (PT_RT - 0.008 - PT_RB) * y / yr


def dead_plant_pot():
    yr = PT_H - PT_RIM
    out = [(0.0, 0.004), (0.020, 0.004), (0.021, 0.0), (PT_RB - 0.004, 0.0), (PT_RB, 0.004), (_pt_r(0.05), 0.05),
           (_pt_r(yr - 0.004), yr - 0.004), (PT_RT - 0.004, yr - 0.002), (PT_RT, yr + 0.003), (PT_RT, PT_H - 0.004),
           (PT_RT - 0.003, PT_H)]
    inner_top = PT_RT - PT_WALL - 0.002
    inn = [(inner_top, PT_H), (inner_top, PT_H - 0.01), (_pt_r(yr - 0.01) - PT_WALL, yr - 0.01),
           (_pt_r(0.05) - PT_WALL, 0.05), (_pt_r(0.02) - PT_WALL - 0.002, 0.012), (0.03, 0.011), (0.020, 0.011)]
    s = lathe(out + inn, seg=40, cap_bottom=False, cap_top=False)
    # chips knocked off the rim's edge
    for (a, k) in ((0.6, 1.0), (2.9, 0.7), (4.4, 0.85)):
        c = rbox(0.022 * k, 0.014 * k, 0.03, r=0.002, seg=1).rot(x=0.5, y=a + 0.4).move(PT_RT * math.cos(a), PT_H + 0.002, PT_RT * math.sin(a))
        s = cut(s, c)
    return s


def dead_plant_soil():
    # a dry crust shrunk from the wall: lumpy, sunk where the water stood
    r_in = _pt_r(SOIL_Y - 0.0) - PT_WALL - 0.008
    rings, seg = 9, 36
    v, f = [Vector((0, SOIL_Y - 0.004, 0))], []
    for i in range(1, rings + 1):
        rr = r_in * i / rings
        for j in range(seg):
            a = TAU * j / seg
            p = Vector((rr * math.cos(a), 0, rr * math.sin(a)))
            y = SOIL_Y - 0.004 * (1 - (rr / r_in) ** 2) + 0.004 * noise.noise(p * 40.0) + 0.002 * noise.noise(p * 120.0)
            if i == rings:
                y = SOIL_Y - 0.003 + 0.002 * noise.noise(p * 40.0)
            v.append(Vector((p.x * (1 + 0.04 * noise.noise(p * 9.0)), y, p.z * (1 + 0.04 * noise.noise(p * 9.0 + Vector((3, 1, 2)))))))
    for j in range(seg):
        f.append((0, 1 + (j + 1) % seg, 1 + j))
    for i in range(rings - 1):
        for j in range(seg):
            a, b = 1 + i * seg + j, 1 + i * seg + (j + 1) % seg
            f.append((a, b, b + seg, a + seg))
    # the crust's edge, turned down into the gap
    base = len(v)
    for j in range(seg):
        p = v[1 + (rings - 1) * seg + j]
        v.append(Vector((p.x * 0.985, SOIL_Y - 0.022, p.z * 0.985)))
    for j in range(seg):
        a, b = 1 + (rings - 1) * seg + j, 1 + (rings - 1) * seg + (j + 1) % seg
        f.append((b, a, base + j, base + (j + 1) % seg))
    return face_toward(Shape(v, f), lambda c: Vector((c.x * 10.0, 1.0, c.z * 10.0)))


def tube_r(points, radii, seg=8, caps=True):
    """A tube along a polyline with its own radius at each point."""
    pts = [Vector(p) for p in points]
    n = len(pts)
    tangents = []
    for i in range(n):
        t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)])
        tangents.append(t.normalized() if t.length > 1e-9 else Vector((0, 1, 0)))
    up = Vector((0, 1, 0)) if abs(tangents[0].y) < 0.9 else Vector((1, 0, 0))
    nrm = tangents[0].cross(up).normalized()
    v, f = [], []
    for i in range(n):
        if i:
            q = tangents[i - 1].rotation_difference(tangents[i])
            nrm = (q @ nrm).normalized()
        b = tangents[i].cross(nrm).normalized()
        for j in range(seg):
            a = TAU * j / seg
            v.append(pts[i] + (nrm * math.cos(a) + b * math.sin(a)) * radii[i])
    for i in range(n - 1):
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((i * seg + j, i * seg + j2, (i + 1) * seg + j2, (i + 1) * seg + j))
    if caps:
        f.append(tuple(reversed(range(seg))))
        f.append(tuple((n - 1) * seg + j for j in range(seg)))
    return propkit._fix_winding(Shape(v, f))


def _stem(seed, L, lean, branch):
    """A woody pelargonium stem from its foot (origin) up +y: it zigzags a
    little at each node, swollen there round the old leaf's base, a dried
    petiole's stub on the node's outer side, tapering, grey bark."""
    rng = random.Random(seed)
    pts, radii, nodes = [Vector((0, 0, 0))], [0.0068], []
    d = Vector((lean[0], 1.0, lean[1])).normalized()
    y, side = 0.0, 1
    r0, r1 = 0.0068, 0.0034
    while True:
        seg_len = rng.uniform(0.024, 0.042) * (1.0 if y > 0.06 else 0.6)
        if y + seg_len > L:
            seg_len = L - y
        p0 = pts[-1]
        # each internode: a slight bow, then the node
        for k in (0.5, 1.0):
            q = p0 + d * seg_len * k + Vector((0.0015 * side, 0, 0.001 * side)) * (1 if k == 0.5 else 0)
            t = (y + seg_len * k) / L
            pts.append(q)
            radii.append((r0 + (r1 - r0) * t) * (1.0 if k == 0.5 else 1.28))
        y += seg_len
        nodes.append((pts[-1], d.copy(), radii[-1], side))
        if y >= L - 1e-6:
            break
        # the zigzag: the next internode turns away from the last node's leaf
        turn = Vector((rng.uniform(0.10, 0.22) * side, 0, rng.uniform(-0.12, 0.12)))
        d = (d + turn + Vector((lean[2] * 0.04, 0, 0))).normalized()
        side = -side
    s = tube_r(pts, radii, seg=7)
    STEM_NODES.append((seed, nodes[-2][0].copy(), nodes[-2][1].copy()))
    for (c, dd, rr, sd) in nodes[:-1]:
        out = dd.cross(Vector((0, 0, 1))).normalized() * sd
        if out.length < 0.5:
            out = Vector((sd, 0, 0))
        stub_l = rng.uniform(0.006, 0.016)
        tip = c + (out * 0.9 + dd * 0.6).normalized() * stub_l
        s = s + tube([c, tip], rr * 0.42, seg=4, r_end=rr * 0.3)
    if branch:
        c, dd, rr, sd = nodes[len(nodes) // 2]
        out = Vector((-sd * 0.7, 0.8, 0.3)).normalized()
        tip = c + out * 0.10
        s = s + tube_r([c, c.lerp(tip, 0.5) + Vector((0, 0.008, 0)), tip], [rr * 0.75, r1 * 0.85, r1 * 0.6], seg=5)
    # the flower stalk dried at the top
    top = pts[-1]
    s = s + tube([top, top + d * 0.035 + Vector((0.01, 0.01, 0.004)), top + d * 0.06 + Vector((0.024, 0.012, 0.01))], 0.0016, seg=4, r_end=0.0011)
    return s


STEM_L = 0.30
STEM_NODES = []


def dead_plant_stem(k):
    params = [(101, (0.05, 0.02, 0.3), True), (202, (-0.04, 0.05, -0.4), False), (303, (0.02, -0.05, 0.5), True)][k]
    return _stem(params[0], STEM_L, params[1], params[2])


def _leaf_outline(r, n=24, lobes=7, crenate=0.08):
    """A pelargonium leaf's outline: round, shallowly lobed, crenate, its
    sinus where the petiole joins (at -y)."""
    pts = []
    for i in range(n):
        a = -math.pi / 2 + 0.35 + (TAU - 0.7) * i / (n - 1)
        rr = r * (1 + crenate * math.cos(a * lobes) + 0.02 * math.sin(a * 23))
        pts.append((rr * math.cos(a), rr * math.sin(a) + r * 0.85))
    pts.append((0.0, r * 0.62))
    return pts


def _leaf_sheet(r, n=10):
    """The leaf as a fan of triangles from its sinus (centre) to the
    outline, two-sided (a sheet 0.4 mm thick)."""
    out = _leaf_outline(r, n * 2 + 1)
    c = Vector((0, r * 0.85, 0))
    v = [c] + [Vector((x, y, 0)) for x, y in out]
    m = len(out)
    rings = 3
    verts = [c]
    for k in range(1, rings + 1):
        t = k / rings
        for x, y in out:
            verts.append(c.lerp(Vector((x, y, 0)), t))
    f = []
    for i in range(m - 1):
        f.append((0, 1 + i, 1 + i + 1))
    for k in range(rings - 1):
        for i in range(m - 1):
            a = 1 + k * m + i
            f.append((a, a + m, a + m + 1, a + 1))
    return Shape(verts, f)


def dead_plant_leaf_dry():
    # shrivelled: the blade curled in on itself and crumpled, hanging from
    # its petiole off the stem's top
    s = _leaf_sheet(0.026, 9)
    def curl(p):
        q = p.copy()
        d = (q.xy - Vector((0, 0.022))).length
        # the edges rolled under, the whole cupped and crinkled
        q.z += 0.010 * (d / 0.026) ** 2 + 0.004 * noise.noise(q * 120.0) + 0.006 * noise.noise(q * 45.0)
        q.x *= 0.8 - 0.25 * (d / 0.026) ** 2
        return q
    s = s.warp(curl)
    s = s.rot(x=1.9).move(0, 0.0, 0.0)
    petiole = tube([Vector((0, 0, 0)), Vector((0, 0.01, 0.02)), Vector((0, -0.005, 0.035))], 0.0011, seg=4)
    lo, hi = s.bounds()
    return petiole + s.move(0, -0.006, 0.035)


def dead_plant_leaf_live():
    s = _leaf_sheet(0.032, 10)
    s = s.warp(lambda p: Vector((p.x, p.y, 0.006 * ((p.xy - Vector((0, 0.027))).length / 0.032) ** 2 + 0.0015 * noise.noise(p * 80.0))))
    s = s.rot(x=1.2)
    petiole = tube([Vector((0, 0, 0)), Vector((0, 0.02, 0.025)), Vector((0, 0.022, 0.045))], 0.0015, seg=5)
    return petiole + s.move(0, 0.022, 0.045)


def dead_plant_fallen():
    # a dried leaf lying on the floor, curled up at its edges
    s = _leaf_sheet(0.031, 9)
    s = s.warp(lambda p: Vector((p.x * (0.85 - 0.1 * ((p.xy - Vector((0, 0.026))).length / 0.031)), p.y, 0.009 * ((p.xy - Vector((0, 0.026))).length / 0.031) ** 2.2 + 0.003 * noise.noise(p * 90.0) + 0.0012)))
    s = s.rot(x=-math.pi / 2)
    pet = tube([Vector((0, 0.0015, 0.0)), Vector((0.002, 0.003, 0.012)), Vector((0.006, 0.006, 0.02))], 0.0009, seg=4)
    s = s + pet
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2, -lo.y, -(lo.z + hi.z) / 2)


def dead_plant_litter():
    # dry leaves fallen onto the soil
    out = Shape()
    rng = random.Random(77)
    for k in range(5):
        a = rng.uniform(0, TAU)
        r = rng.uniform(0.03, 0.10)
        l = dead_plant_fallen().scale(rng.uniform(0.8, 1.05)).rot(y=rng.uniform(0, TAU), x=rng.uniform(-0.15, 0.15))
        out = out + l.move(r * math.cos(a), SOIL_Y + 0.001 - 0.002 * (r / 0.1), r * math.sin(a))
    return out


auto('Real__deadPlantPot', dead_plant_pot, 2200)
auto('Real__deadPlantSoil', dead_plant_soil, 800)
auto('Real__deadPlantStemA', lambda: dead_plant_stem(0), 900)
auto('Real__deadPlantStemB', lambda: dead_plant_stem(1), 900)
auto('Real__deadPlantStemC', lambda: dead_plant_stem(2), 900)
for seed, c, d in STEM_NODES:
    print('r18 stem %d: top node (%.5f, %.5f, %.5f) dir (%.4f, %.4f, %.4f)' % (seed, c.x, c.y, c.z, d.x, d.y, d.z))
auto('Real__deadPlantLeafDry', dead_plant_leaf_dry, 260)
auto('Real__deadPlantLeafLive', dead_plant_leaf_live, 260)
auto('Real__deadPlantFallen', dead_plant_fallen, 200)
auto('Real__deadPlantLitter', dead_plant_litter, 900)


# ============================================================ washcloth ==
# A 12 x 12 in (305 mm) cotton terry washcloth, about 500 g/m2: the looped
# pile 2.5 mm thick, a flat-woven (dobby) band 14 mm wide set 8 mm in from
# each edge, a folded hem 6 mm round it. Dropped: folded over on itself
# about a third of the way across, the fold's corner turned back, lying
# limp in the floor's dips and rucked up where it fell.
WC = 0.305              # its side
WC_T = 0.0032           # the pile's thickness
WC_FOLD_V = 0.035       # the fold line across the flat cloth (v), its radius
WC_FOLD_R = 0.0045


def _fold(p, h, s_line, n_line, r):
    """Fold the flat cloth point p (x, z) at mid-surface height h over the
    line {q : (q - s_line) . n_line = 0}: the side n_line points to goes up
    over a half-round of radius r and lies back flat on top."""
    d = (p - s_line).dot(n_line)
    if d <= 0:
        return p, h
    foot = p - n_line * d
    if d < math.pi * r:
        th = d / r
        return foot + n_line * (r * math.sin(th)), h + r - r * math.cos(th)
    return foot - n_line * (d - math.pi * r), h + 2 * r


def _wc_map(u, v):
    """The flat cloth's point (u, v) in [-WC/2, WC/2]^2 to its place on the
    floor (x, z) and the height of its mid-surface."""
    p = Vector((u, v))
    h = WC_T / 2
    p, h = _fold(p, h, Vector((0.0, WC_FOLD_V)), Vector((0.0, 1.0)), WC_FOLD_R)
    if h > WC_T / 2 + WC_FOLD_R * 1.9:
        # the flap's far corner curled up off it
        c = Vector((-WC / 2, WC_FOLD_V - (WC / 2 - WC_FOLD_V)))
        dc = (p - c).length
        h += 0.038 * max(0.0, 1.0 - dc / 0.12) ** 2.0
        # and the flap askew as it fell, sagging into soft ridges
        q = p - Vector((0.0, WC_FOLD_V))
        a = -0.22 * max(0.0, -q.y) / 0.12
        p = Vector((0.0, WC_FOLD_V)) + Vector((q.x * math.cos(a) - q.y * math.sin(a), q.x * math.sin(a) + q.y * math.cos(a)))
    # its weave never quite square, pulled out of true: edges wavy
    p.x += 0.004 * noise.noise(Vector((u * 7, v * 7, 1.0)))
    p.y += 0.004 * noise.noise(Vector((u * 7, v * 7, 4.0)))
    return p, h


def _wc_surface(u, v):
    p, h = _wc_map(u, v)
    x, z = p.x, p.y
    # limp: rucked where it fell, the ruck lifting every layer with it
    ruck = 0.011 * math.exp(-((x + 0.04) ** 2 / 0.006 + (z + 0.01) ** 2 / 0.0015))
    ruck += 0.005 * max(0.0, noise.noise(Vector((x * 7, z * 7, 2.0))))
    # soft ridges running across it where it bunched as it fell
    for (ax, az, bx, bz, hh, ww) in WC_RUCKS:
        dx, dz = bx - ax, bz - az
        t = max(0.0, min(1.0, ((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz)))
        d = math.hypot(x - ax - t * dx, z - az - t * dz)
        ruck += hh * math.sin(math.pi * t) ** 0.7 * math.exp(-(d / ww) ** 2) * (0.75 + 0.25 * noise.noise(Vector((x * 20, z * 20, 3.0))))
    return Vector((x, h + ruck, z))


# the ridges: from (ax, az) to (bx, bz) on the floor, their height and width
WC_RUCKS = [(-0.14, -0.10, 0.02, 0.02, 0.017, 0.012), (0.03, -0.13, 0.13, 0.01, 0.012, 0.010), (-0.06, 0.035, 0.12, 0.03, 0.008, 0.011)]


def _grid_slab(us, vs):
    """A thin box over the grid lines us x vs (the side k = -0.5 .. 0.5 in z)."""
    nx, ny = len(us) - 1, len(vs) - 1
    v, f = [], []
    for k in (0.5, -0.5):
        for j in range(ny + 1):
            for i in range(nx + 1):
                v.append(Vector((us[i], vs[j], k)))
    K = nx + 1
    n = len(v) // 2
    for j in range(ny):
        for i in range(nx):
            a = j * K + i
            f.append((a, a + 1, a + K + 1, a + K))
            f.append((n + a, n + a + K, n + a + K + 1, n + a + 1))
    rim = [i for i in range(nx + 1)] + [j * K + nx for j in range(1, ny + 1)] + [ny * K + i for i in range(nx - 1, -1, -1)] + [j * K for j in range(ny - 1, 0, -1)]
    for a, b in zip(rim, rim[1:] + rim[:1]):
        f.append((b, a, n + a, n + b))
    return propkit._fix_winding(Shape(v, f))


def _lines(a, b, step, dense=()):
    """Grid lines from a to b about `step` apart, closer within each
    (centre, half width, step) of `dense`."""
    out = [a]
    x = a
    while x < b - 1e-9:
        st = step
        for c, hw, s2 in dense:
            if abs(x - c) < hw + step:
                st = min(st, s2)
        x = min(b, x + st)
        out.append(x)
    return out


def _wc_region(u0, u1, v0, v1, t):
    """A slab of the cloth over [u0, u1] x [v0, v1] of the flat cloth, t
    thick, laid as the cloth lies."""
    us = _lines(u0, u1, 0.013)
    vs = _lines(v0, v1, 0.013, [(WC_FOLD_V + WC_FOLD_R * 1.6, WC_FOLD_R * 2.2, 0.0022)])
    s = _grid_slab(us, vs)
    out = []
    eps = 0.0008
    for p in s.v:
        u, v, k = p.x, p.y, p.z
        c = _wc_surface(u, v)
        du = _wc_surface(u + eps, v) - _wc_surface(u - eps, v)
        dv = _wc_surface(u, v + eps) - _wc_surface(u, v - eps)
        n = du.cross(dv)
        n = n.normalized() if n.length > 1e-12 else Vector((0, 1, 0))
        out.append(c + n * (k * t))
    s.v = out
    return s


WC_HEM, WC_GAP, WC_BAND = 0.006, 0.008, 0.014


def _wc_strips():
    """The flat cloth's [u0, u1, v0, v1] of the hem and dobby band, and of
    the terry field and the terry between them."""
    a = WC / 2
    hem = a - WC_HEM
    gap = hem - WC_GAP
    band = gap - WC_BAND
    def frame(r_out, r_in):
        return [(-r_out, r_out, r_in, r_out), (-r_out, r_out, -r_out, -r_in), (-r_out, -r_in, -r_in, r_in), (r_in, r_out, -r_in, r_in)]
    return frame(a, hem) + frame(gap, band), frame(hem, gap) + [(-band, band, -band, band)]


def washcloth_terry():
    flat, pile = _wc_strips()
    s = Shape()
    for (u0, u1, v0, v1) in pile:
        s = s + _wc_region(u0, u1, v0, v1, WC_T)
    return s


def washcloth_border():
    flat, pile = _wc_strips()
    s = Shape()
    for (u0, u1, v0, v1) in flat:
        s = s + _wc_region(u0, u1, v0, v1, WC_T * 0.55)
    return s


auto('Real__washclothTerry', washcloth_terry, 4000)
auto('Real__washclothBorder', washcloth_border, 2400)


# ========================================================== nail polish ==
# A small bottle of nail polish (the 13.5 ml square "retro" bottle: glass
# 25 x 25 mm, 52 mm high to the top of its neck; a moulded screw cap 30 mm
# long carrying the brush): heavy-bottomed glass, the lacquer dried down in
# it, a drip run down its side and dried, a crust round the neck.
NP_W = 0.025            # the glass's side
NP_H = 0.040            # to its shoulder
NP_NECK = (0.0060, 0.0435, 0.052)    # the neck's radius, foot and top
NP_BASE = 0.007         # the glass's thick bottom
NP_WALL = 0.0024
NP_FILL = 0.029         # the dried lacquer's level (upright)
NP_CAP = (0.0086, 0.0080, 0.0415, 0.0685)   # the cap's radii (foot, top), foot and top


def _np_body(w, y0, y1, r, rr):
    """A square glass body: w across, from y0 to y1, its upright edges
    rounded r, its top and bottom edges rr."""
    s = rbox(w, y1 - y0, w, r=rr, seg=2)
    # the upright edges rounder than the rest
    def round_edges(p):
        ax, az = abs(p.x), abs(p.z)
        lim = w / 2 - r
        if ax > lim and az > lim:
            dx, dz = ax - lim, az - lim
            d = math.hypot(dx, dz)
            if d > r:
                k = r / d
                p = Vector((math.copysign(lim + dx * k, p.x), p.y, math.copysign(lim + dz * k, p.z)))
        return p
    s = subdivide_edges(s)
    return s.warp(round_edges).move(0, (y0 + y1) / 2, 0)


def subdivide_edges(s):
    # more vertices round the corners so the rounding has something to bend
    bm = to_bmesh(s)
    bmesh.ops.subdivide_edges(bm, edges=[e for e in bm.edges], cuts=2, use_grid_fill=True)
    out = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


def polish_glass():
    outer = _np_body(NP_W, 0.0, NP_H, 0.0045, 0.0018)
    r, y0, y1 = NP_NECK
    shoulder = lathe([(0.0, NP_H - 0.002), (NP_W / 2 - 0.002, NP_H - 0.002), (NP_W / 2 - 0.004, NP_H + 0.0008), (r + 0.003, y0 - 0.0005), (r, y0), (0.0, y0)], seg=20)
    neck = lathe([(0.0, y0 - 0.001), (r, y0 - 0.001), (r, y0 + 0.001), (r + 0.0006, y0 + 0.0018), (r, y0 + 0.0028), (r + 0.0006, y0 + 0.0040), (r, y0 + 0.0052), (r, y1 - 0.0006), (r - 0.0006, y1), (0.0, y1)], seg=16)
    s = boolean(outer, shoulder, 'UNION')
    s = boolean(s, neck, 'UNION')
    # the inside: a cavity up into the neck
    cav = rbox(NP_W - 2 * NP_WALL, NP_H - NP_BASE - NP_WALL * 0.5, NP_W - 2 * NP_WALL, r=0.003, seg=2).move(0, NP_BASE + (NP_H - NP_BASE - NP_WALL * 0.5) / 2, 0)
    cav = boolean(cav, cyl(r - 0.0022, h=0.02, seg=12, bevel_r=0).move(0, y1 - 0.004, 0), 'UNION')
    return boolean(s, cav, 'DIFFERENCE')


def _np_cavity_below(level, axis='y'):
    cav = rbox(NP_W - 2 * NP_WALL - 0.0003, NP_H - NP_BASE - NP_WALL * 0.5 - 0.0003, NP_W - 2 * NP_WALL - 0.0003, r=0.0028, seg=2)
    cav = cav.move(0, NP_BASE + (NP_H - NP_BASE - NP_WALL * 0.5) / 2, 0)
    if axis == 'y':
        cut_box = box(0.05, 0.08, 0.05).move(0, level + 0.04, 0)
    else:
        cut_box = box(0.08, 0.08, 0.05).move(level + 0.04, 0.02, 0)
    return boolean(cav, cut_box, 'DIFFERENCE')


def polish_lacquer():
    s = _np_cavity_below(NP_FILL)
    # its surface dried in a skin with a meniscus up the walls
    return s.warp(lambda p: Vector((p.x, p.y + (0.0008 * (max(abs(p.x), abs(p.z)) / (NP_W / 2 - NP_WALL)) ** 6 if p.y > NP_FILL - 0.0005 else 0.0), p.z)))


def polish_lacquer_lying():
    # tipped onto its -x face while still wet: pooled along that face
    return _np_cavity_below(-NP_W / 2 + NP_WALL + 0.0065, 'x')


def polish_cap():
    r0, r1, y0, y1 = NP_CAP
    prof = [(0.0, y0), (r0 - 0.0004, y0), (r0, y0 + 0.0005), (r0, y0 + 0.003), (r0 - 0.0003, y0 + 0.0036),
            (r1, y1 - 0.0016), (r1 - 0.0012, y1), (0.0, y1)]
    return lathe(prof, seg=20)


def polish_brush():
    # the brush's white stem from the cap down through the neck nearly to
    # the bottom, in the lacquer
    r, y0, y1 = NP_NECK
    stem = cyl(0.0016, 0.0019, (y1 - 0.003) - (NP_BASE + 0.004), seg=8, bevel_r=0.0003)
    return stem.move(0, (y1 - 0.003 + NP_BASE + 0.004) / 2, 0)


def polish_drip():
    # dried lacquer: a smear on the shoulder and a drip run from the
    # shoulder's edge down the +z face. Dried, it
    # lies on the glass as a flat run a fraction of a millimetre proud,
    # wider than it is thick, ending in a flattened bead (round, it stood
    # off the glass like the brush's stem bent out of the neck: M8)
    r, y0, y1 = NP_NECK
    zf = NP_W / 2
    path = [Vector((0.0029, NP_H + 0.0003, NP_W / 2 - 0.0030)), Vector((0.003, NP_H - 0.0008, zf - 0.0001)), Vector((0.0033, NP_H - 0.006, zf)),
            Vector((0.0034, NP_H - 0.0105, zf))]
    # the shoulder's surface along +z: its height at a distance out from the axis
    sh0, sh1 = (r + 0.003, y0 - 0.0005), (NP_W / 2 - 0.004, NP_H + 0.0008)

    def shoulder_y(d):
        t = min(max((d - sh0[0]) / (sh1[0] - sh0[0]), 0.0), 1.0)
        return sh0[1] + (sh1[1] - sh0[1]) * t

    def hug(p):
        if p.y < NP_H - 0.0008:
            # on the face: pressed flat against it
            return Vector((p.x, p.y, zf + max(p.z - zf, 0.0) * 0.28))
        if p.z < sh1[0]:
            # on the shoulder: pressed down onto its slope
            ys = shoulder_y(p.z)
            return Vector((p.x, ys + max(p.y - ys, 0.0) * 0.3, p.z))
        return p
    drip = tube(path, 0.0016, seg=6, r_end=0.0012).warp(hug)
    bead = sphere(0.0019, seg=8, rings=5).scale(1.0, 1.2, 0.28).move(0.0034, NP_H - 0.0115, zf)
    # a smear of it on the shoulder where the brush was wiped (no ring under
    # the cap: with the run from it, it read as the stem bent out of the neck)
    smear = sphere(0.0022, seg=8, rings=5).scale(1.4, 0.25, 0.9).rot(x=0.5).move(-0.0030, NP_H - 0.0003, 0.0090)
    return drip + bead + smear


GLASS = []


def _glass():
    if not GLASS:
        GLASS.append(polish_glass())
    return GLASS[0].copy()


def polish_glass_walls():
    return boolean(_glass(), box(0.05, 0.02, 0.05).move(0, NP_BASE - 0.01, 0), 'DIFFERENCE')


def polish_glass_base():
    # the thick bottom: a block of glass, greener and less clear than the walls
    return boolean(_glass(), box(0.05, 0.02, 0.05).move(0, NP_BASE - 0.01, 0), 'INTERSECT')


auto('Real__nailPolishGlass', polish_glass_walls, 1700)
auto('Real__nailPolishGlassBase', polish_glass_base, 600)
auto('Real__nailPolishLacquer', polish_lacquer, 600)
auto('Real__nailPolishLacquerLying', polish_lacquer_lying, 600)
auto('Real__nailPolishCap', polish_cap, 300)
auto('Real__nailPolishBrush', polish_brush, 100)
auto('Real__nailPolishDrip', polish_drip, 300)


# ================================================================ floss ==
# A 50 m dental floss dispenser (the common flip-top kind: about 40 x 55 x
# 20 mm, moulded polypropylene, a stainless cutter in the lid), lying on
# its back: its length along z (the lid at +z), its face up.
FL_W, FL_T, FL_L = 0.040, 0.019, 0.055
FL_SPLIT = 0.011        # the lid's parting line (z)
FL_R = 0.0065


def _floss_shell():
    s = rbox(FL_W, FL_T, FL_L, r=FL_R, seg=4)
    # its faces a little domed
    s = s.warp(lambda p: Vector((p.x, p.y * (1 + 0.06 * (1 - (2 * p.x / FL_W) ** 2) * (1 - (2 * p.z / FL_L) ** 2)), p.z)))
    return s.move(0, FL_T / 2, 0)


def floss_body():
    s = boolean(_floss_shell(), box(0.06, 0.04, 0.06).move(0, 0.01, FL_SPLIT + 0.0002 + 0.03), 'DIFFERENCE')
    # the thumb notch under the lid's front edge
    s = cut(s, cyl(0.006, h=0.01, seg=12, bevel_r=0.001).rot(z=0).move(0, FL_T + 0.003, FL_SPLIT - 0.001))
    return s


def floss_lid():
    s = boolean(_floss_shell(), box(0.06, 0.04, 0.06).move(0, 0.01, FL_SPLIT + 0.0006 - 0.03), 'DIFFERENCE')
    # the floss's exit: a groove across the lid's top end
    s = cut(s, rbox(0.012, 0.004, 0.006, r=0.0012, seg=1).move(0, FL_T - 0.0015, FL_L / 2 - 0.004))
    return s


def floss_cutter():
    # the stainless blade set into the top end, its serrated edge
    plate = rbox(0.011, 0.0005, 0.0045, r=0.0002, seg=1)
    teeth = Shape()
    for i in range(6):
        t = prism([(-0.0007, 0), (0.0007, 0), (0.0, 0.0011)], 0.0005).rot(x=-math.pi / 2).move(-0.0045 + i * 0.0018, 0, 0.0022 + 0.0005)
        teeth = teeth + t
    return (plate + teeth).rot(x=0.35).move(0, FL_T - 0.002, FL_L / 2 - 0.0036)


def floss_thread():
    # the end left out of it, lying on the floor in a loose curl
    pts = [Vector((0.0, FL_T - 0.002, FL_L / 2 - 0.004)), Vector((0.002, FL_T - 0.0012, FL_L / 2 + 0.001)),
           Vector((0.004, FL_T * 0.55, FL_L / 2 + 0.006)), Vector((0.008, 0.0012, FL_L / 2 + 0.012))]
    rng = random.Random(9)
    a, p = 0.4, pts[-1].copy()
    for i in range(9):
        a += 0.42 + 0.22 * math.sin(i * 1.7)
        p = p + Vector((math.cos(a) * 0.008, 0.0, math.sin(a) * 0.008 + 0.003))
        pts.append(Vector((p.x, 0.0007 + 0.0006 * (i % 2), p.z)))
    return tube(pts, 0.00045, seg=4, caps=True)


auto('Real__flossBody', floss_body, 1400)
auto('Real__flossLid', floss_lid, 1200)
auto('Real__flossCutter', floss_cutter, 160)
auto('Real__flossThread', floss_thread, 200)
