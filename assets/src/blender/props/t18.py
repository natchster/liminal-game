"""t18: the props remodelled at their real sizes (docs/design/PROPS.md, P3, the
rare): what a person brought and set out, or left behind, where the place
keeps its shrines and its parties (docs/design/LORE.md; docs/design/PROPS.md 1, "the
party and the shrine"): a brass thurible hung from a ceiling hook, a
printed talking board and its wooden planchette, a fan of tarot cards, a
glass ball on its turned stand, a floor shrine (a board, a book, a framed
photograph, chime candles burnt down, dried rosebuds, a cross or a star, a
black ribbon) and a short length of proof coil chain. The chalk pentagram
is painted only (a decal). Sources and sizes: t18.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame: x across its front, y up from the floor, z out of
its front, unless its section says otherwise. Reused, not modelled again:
t01's 5 x 7 in photo frame, its glass and back (the shrine's photograph),
r02's small hardback (the shrine's riser), t09's candle flame (the shrine's
chime candles).
"""
from propkit import *
from propkit import _mesh_obj, _eval_to_shape, _drop
import propkit
import random
import zlib

MM = 0.001


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, smooth=35.0, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth)


def rnd(a, b):
    return a + (b - a) * random.random()


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def mm(s):
    """A shape built in millimetres, in metres"""
    return Shape([p * MM for p in s.v], s.f, s.sharp)


def _fix(s):
    bm = propkit.to_bmesh(s)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-7)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    out = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    return out


def lathe_mm(prof, seg):
    return mm(lathe(prof, seg=seg, cap_bottom=False, cap_top=False))


def basis(d, up_hint):
    """An orthonormal frame (x along d, y as near up_hint as can be, z)"""
    x = Vector(d).normalized()
    y = Vector(up_hint)
    y = (y - x * y.dot(x))
    if y.length < 1e-9:
        y = Vector((0, 1, 0)) if abs(x.y) < 0.9 else Vector((1, 0, 0))
        y = (y - x * y.dot(x))
    y.normalize()
    z = x.cross(y)
    return Matrix((x, y, z)).transposed()


def place(s, rot3, at):
    """s turned by the 3x3 `rot3` and moved to `at`"""
    m = rot3.to_4x4()
    m.translation = Vector(at)
    return s.copy().apply(m)


# -------------------------------------------------------------- a link ---
def link(l_out, w_out, r, n_end=4, tseg=6, weld=0.0):
    """A welded chain link, its centreline a stadium in the xy plane (long
    axis x), centred: `l_out` x `w_out` outside, of wire `r` in radius; its
    weld a swelling at the middle of one side (`weld` of r)"""
    rc = w_out / 2 - r
    a = l_out / 2 - r - rc
    cl = []
    for i in range(n_end + 1):
        q = -math.pi / 2 + math.pi * i / n_end
        cl.append((Vector((a + rc * math.cos(q), rc * math.sin(q), 0)), Vector((math.cos(q), math.sin(q), 0)), 1.0))
    if weld > 0 and a > 1e-9:
        cl.append((Vector((0, rc, 0)), Vector((0, 1, 0)), 1.0))
    for i in range(n_end + 1):
        q = math.pi / 2 + math.pi * i / n_end
        cl.append((Vector((-a + rc * math.cos(q), rc * math.sin(q), 0)), Vector((math.cos(q), math.sin(q), 0)), 1.0))
    if weld > 0 and a > 1e-9:
        cl.append((Vector((0, -rc, 0)), Vector((0, -1, 0)), 1.0 + weld))
    if a < 1e-9:      # a ring: the two arcs share their ends
        cl = cl[:n_end] + cl[n_end + 1:-1]
    v, f = [], []
    z = Vector((0, 0, 1))
    n = len(cl)
    for c, out, k in cl:
        rr = r * k
        for j in range(tseg):
            q = TAU * j / tseg
            v.append(c + out * rr * math.cos(q) + z * rr * math.sin(q))
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(tseg):
            j2 = (j + 1) % tseg
            f.append((i * tseg + j, i * tseg + j2, i2 * tseg + j2, i2 * tseg + j))
    return propkit._fix_winding(Shape(v, f))


def chain_line(a, b, lnk, pitch, twist0=0.0, up=(0, 1, 0)):
    """Links of `lnk` (long axis x) hung from a to b, each `pitch` on from
    the last, every other one turned a quarter about the line"""
    a, b = Vector(a), Vector(b)
    d = b - a
    L = d.length
    n = max(1, int(round(L / pitch)))
    step = d / n
    rot = basis(d, up)
    out = Shape()
    for k in range(n):
        c = a + step * (k + 0.5)
        tw = twist0 + (math.pi / 2 if k % 2 else 0.0)
        m = rot @ Matrix.Rotation(tw, 3, 'X')
        out = out + place(lnk, m, c)
    return out


# ===================================================== hanging censer ===
# A brass thurible of the common church kind (place_hanging_censers): a
# bowl on a turned foot, its rim rolled, three lugs on it for the chains; a
# domed lid pierced in two rows and a cupola, a ring on its finial; three
# cable chains from the lugs through guides on the lid's rim to a domed
# chain head with a finger ring, and the lid's own chain up through it.
# 108 mm across the bowl, 70 mm across the foot, 75 mm foot to rim, the lid
# 97 mm (t18.notes.md). Hung from a brass swag hook screwed into the
# ceiling by a length of lamp chain. Frame: y = 0 the bowl's rim, centred;
# the hook's frame: y = 0 the ceiling, its J's lowest point at x = z = 0.
CENSER_LUG_A = [math.radians(90 + 120 * k) for k in range(3)]
CENSER_LID_Y = 1.6               # the lid's foot on the rim's bead (mm)
CENSER_HEAD_Y = 330.0            # the chain head's underside above the rim (mm)
CENSER_RING_R = 12.0             # the finger ring (centreline radius, mm)
CENSER_RING_X = -4.0
CENSER_RING_Y = CENSER_HEAD_Y + 8.0 + CENSER_RING_R + 2.2 - 2.0   # its centre
CENSER_LID_CHAIN_X = 7.0


def censer_bowl():
    prof = [(0, -73.0), (26.0, -75.0), (33.0, -74.5), (35.0, -73.0), (35.0, -70.5), (33.0, -69.0), (28.0, -66.5),
            (22.0, -63.5), (15.0, -61.0), (11.0, -58.5), (11.5, -55.5), (15.5, -53.0), (16.0, -50.5), (13.0, -48.0),
            (18.0, -46.0), (28.0, -41.5), (38.0, -34.0), (45.5, -25.5), (50.5, -16.5), (53.0, -8.0), (54.0, -3.0),
            (55.6, -1.6), (56.2, 0.0), (55.4, 1.5), (53.4, 1.6), (52.4, 0.2), (52.2, -3.0), (51.2, -8.0),
            (48.6, -16.0), (43.6, -25.0), (36.0, -33.0), (26.0, -40.0), (14.0, -44.5), (0, -45.5)]
    s = _fix(lathe_mm(prof, 40))
    # the three lugs: a tab on the bowl's side under the rim, pierced, a
    # ring through it for the chain
    for a in CENSER_LUG_A:
        tab = rbox(9.0, 12.0, 3.0, r=1.0, seg=1).move(0, -6.0, 0)
        tab = cut(tab, cyl(2.0, h=8.0, seg=10, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, -4.0, 0))
        ring = torus(2.6, 0.8, seg=10, tseg=5).rot(0, math.pi / 2, 0).move(0, -1.9, 1.2)
        ob = (tab + ring).rot(0, -a + math.pi / 2, 0)
        s = s + mm(ob.move(55.2 * math.cos(a), 0, 55.2 * math.sin(a)))
    return s


def censer_lid():
    out = [(54.2, 0.0), (55.0, 1.2), (54.6, 3.0), (52.0, 5.0), (50.0, 9.0), (46.5, 16.0), (41.0, 24.0), (34.0, 32.0),
           (27.0, 38.0), (22.0, 42.0), (20.5, 44.0), (20.5, 46.0), (17.5, 47.0), (17.0, 62.0), (19.5, 63.5),
           (19.5, 66.0), (15.0, 68.5), (10.5, 73.0), (7.0, 79.0), (5.0, 84.0), (5.6, 86.5), (6.6, 89.0),
           (5.2, 91.5), (3.0, 92.5), (0.0, 92.6)]
    inner = [(52.6, 2.0), (50.6, 4.5), (48.6, 9.0), (45.1, 16.0), (39.6, 24.0), (32.6, 32.0), (25.8, 37.6), (20.8, 41.6),
             (16.0, 45.6), (15.6, 62.0), (14.0, 67.6), (9.5, 72.2), (6.0, 78.0), (0.0, 80.0)]
    sol = lathe(out, seg=40, cap_bottom=False, cap_top=False)
    hol = lathe(inner, seg=40, cap_bottom=False, cap_top=False)
    # the shell: its outside, its inside, and the ring of its foot between
    ring = lathe([(52.6, 2.0), (54.2, 0.0)], seg=40, cap_bottom=False, cap_top=False)
    s = weld(sol + hol + ring, 1e-4)
    s = propkit._fix_winding(s)
    # pierced: ten round holes round the dome, eight above them, six slots
    # round the cupola
    cutters = []
    for k in range(10):
        a = TAU * (k + 0.5) / 10
        c = cyl(3.6, h=16.0, seg=10, bevel_r=0).rot(0, 0, math.pi / 2).rot(0, 0, 0.46).rot(0, -a, 0)
        cutters.append(c.move(47.6 * math.cos(a), 13.5, 47.6 * math.sin(a)))
    for k in range(8):
        a = TAU * k / 8
        c = cyl(3.0, h=14.0, seg=10, bevel_r=0).rot(0, 0, math.pi / 2).rot(0, 0, 0.72).rot(0, -a, 0)
        cutters.append(c.move(36.0 * math.cos(a), 29.5, 36.0 * math.sin(a)))
    for k in range(6):
        a = TAU * (k + 0.5) / 6
        c = rbox(10.0, 9.5, 4.6, r=1.8, seg=2).rot(0, -a + math.pi / 2, 0)
        cutters.append(c.move(16.8 * math.cos(a), 55.0, 16.8 * math.sin(a)))
    s = boolean(s, sum(cutters[1:], cutters[0]), 'DIFFERENCE')
    # the guides on its rim the chains run through, the ring on its finial
    for a in CENSER_LUG_A:
        g = torus(4.4, 0.9, seg=12, tseg=5).rot(math.pi / 2, 0, 0)
        s = s + g.move(57.6 * math.cos(a), 5.0, 57.6 * math.sin(a))
    s = s + torus(4.0, 1.0, seg=12, tseg=5).move(0, 92.6 + 3.6, 0)
    return mm(s.move(0, CENSER_LID_Y, 0))


def censer_chains():
    lnk = link(10.0, 7.0, 0.75, n_end=2, tseg=3)
    s = Shape()
    for k, a in enumerate(CENSER_LUG_A):
        lo = (57.6 * math.cos(a), 1.6, 57.6 * math.sin(a))
        hi = (13.0 * math.cos(a), CENSER_HEAD_Y - 3.5, 13.0 * math.sin(a))
        s = s + chain_line(lo, hi, lnk, 8.5, twist0=0.4 * k)
    # the lid's chain, up through the head to its ring
    top = CENSER_LID_Y + 92.6 + 3.6 + 4.0 - 1.0
    s = s + chain_line((0, top, 0), (CENSER_LID_CHAIN_X, CENSER_HEAD_Y + 26.0, 0), lnk, 8.5, up=(1, 0, 0))
    s = s + torus(6.5, 1.1, seg=14, tseg=5).rot(0, 0.5, 0).move(CENSER_LID_CHAIN_X, CENSER_HEAD_Y + 32.0, 0)
    # the chain head: a domed disc, the lid chain's hole, three eyes under it
    head = lathe([(0, 0), (19.0, 0), (22.0, 1.2), (22.5, 3.0), (20.0, 5.5), (13.0, 7.6), (6.0, 8.4), (0, 8.5)], seg=28)
    head = cut(head, cyl(3.4, h=30.0, seg=10, bevel_r=0).move(CENSER_LID_CHAIN_X, 4, 0))
    s = s + head.move(0, CENSER_HEAD_Y, 0)
    for a in CENSER_LUG_A:
        e = torus(2.8, 0.9, seg=10, tseg=5).rot(0, -a + math.pi / 2, 0)
        s = s + e.move(13.0 * math.cos(a), CENSER_HEAD_Y - 1.0, 13.0 * math.sin(a))
    # the finger ring standing on the head
    s = s + torus(CENSER_RING_R, 2.2, seg=20, tseg=6).move(CENSER_RING_X, CENSER_RING_Y, 0)
    return mm(s)


# the lamp chain it hangs by: two links of 20 x 12 mm brass wire (1.3 mm),
# the lower in the yz plane; the pitch of a pair 29.6 mm; y = 0 the lower
# link's lowest wire centre
DROP_PITCH = 29.6


def censer_drop_links():
    lnk = link(20.0, 12.0, 1.3, n_end=3, tseg=4)
    lo = lnk.copy().rot(0, 0, math.pi / 2).rot(0, math.pi / 2, 0).move(0, 8.7, 0)
    hi = lnk.copy().rot(0, 0, math.pi / 2).move(0, 8.7 + 14.8, 0)
    return mm(lo + hi)


def censer_hook():
    """A brass swag hook: its canopy screwed to the ceiling, a 4 mm wire J
    in the yz plane whose lowest point (y = -35 mm) is at x = z = 0"""
    can = lathe([(0, 0), (15.5, 0), (15.8, -0.8), (15.0, -2.2), (12.0, -3.2), (5.0, -3.8), (3.0, -5.5), (3.0, -6.5), (0, -6.5)], seg=28)
    pts = [Vector((-9.0, -6.0, 0)), Vector((-9.0, -26.0, 0))]
    for i in range(1, 10):
        q = math.pi + math.pi * i / 9
        pts.append(Vector((math.cos(q) * 9.0, -26.0 + math.sin(q) * 9.0, 0)))
    pts.append(Vector((9.0, -18.0, 0)))
    j = tube(pts, 2.0, seg=8)
    tip = sphere(2.6, seg=10).move(9.0, -17.0, 0)
    screw = screw_head(r=2.0, h=0.8).rot(math.pi, 0, 0).move(1.0, -3.4, 9.0)
    hook = (can.move(-9.0, 0, 0) + j + tip).rot(0, math.pi / 2, 0)
    return mm(hook + screw)


auto('Real__censerBowl', censer_bowl, 4000, smooth=40.0, kind='cyl')
auto('Real__censerLid', censer_lid, 6000, smooth=40.0, kind='cyl')
auto('Real__censerChains', censer_chains, 6400, smooth=60.0)
auto('Real__censerDropLinks', censer_drop_links, 160, smooth=60.0)
auto('Real__censerHook', censer_hook, 900, smooth=50.0)


# ======================================================= talking board ===
# A printed talking board of the 1940s-60s kind (place_ouija_boards): a
# lithographed face on 1/8 in hardboard, 18 x 12 in (457 x 305 mm), its
# corners rounded, warped a little from lying in the damp, its face chipped
# at a corner; and a wooden planchette, a heart 105 x 84 mm, 6 mm thick, a
# window cut near its point, on three turned legs with felt pads. Frame: x
# across the board (its letters' way), y up, z toward its near edge (the
# GOOD BYE edge); the planchette's point toward -z.
TB_W, TB_D, TB_T = 457.0, 305.0, 3.4
TB_R = 9.0


def _round_corners(s, w, d, r):
    """Pull every vertex outside the rounded rectangle onto its corner arc"""
    cx, cz = w / 2 - r, d / 2 - r
    for p in s.v:
        ax, az = abs(p.x), abs(p.z)
        if ax > cx and az > cz:
            dx, dz = ax - cx, az - cz
            l = math.hypot(dx, dz)
            k = r / l
            p.x = math.copysign(cx + dx * k, p.x)
            p.z = math.copysign(cz + dz * k, p.z)
    return s


def _board_warp(p):
    u, w = p.x / (TB_W / 2), p.z / (TB_D / 2)
    p.y += 2.6 * (u * u) + 0.9 * u * w + 0.6 * (w * w)
    return p


def talk_board_core():
    s = slab(TB_W, TB_D, TB_T, nx=24, ny=16).rot(-math.pi / 2, 0, 0)     # lying: its faces up and down
    s = _round_corners(s, TB_W, TB_D, TB_R)
    s.move(0, TB_T / 2, 0)
    s.warp(_board_warp)
    # a corner knocked: the face's edge chipped at the near right
    def chip(p):
        dx, dz = p.x - (TB_W / 2 - 4.0), p.z - (TB_D / 2 - 6.0)
        d = math.hypot(dx, dz)
        if d < 14.0 and p.y > TB_T * 0.5:
            p.y -= (1 - d / 14.0) * 1.1
        return p
    s.warp(chip)
    lo, _ = s.bounds()
    return mm(s.move(0, -lo.y, 0))


def talk_board_face():
    s = grid_plane(TB_W - 1.2, TB_D - 1.2, 40, 28).rot(-math.pi / 2, 0, 0)
    s = _round_corners(s, TB_W - 1.2, TB_D - 1.2, TB_R - 0.6)
    s.move(0, TB_T + 0.06, 0)
    s.warp(_board_warp)
    def chip(p):
        dx, dz = p.x - (TB_W / 2 - 4.0), p.z - (TB_D / 2 - 6.0)
        d = math.hypot(dx, dz)
        if d < 14.0:
            p.y -= (1 - d / 14.0) * 1.1
        return p
    s.warp(chip)
    # the core's lowest point (its warp) is the floor; the face rides on it
    core = slab(TB_W, TB_D, TB_T, nx=24, ny=16).rot(-math.pi / 2, 0, 0)
    core = _round_corners(core, TB_W, TB_D, TB_R)
    core.move(0, TB_T / 2, 0)
    core.warp(_board_warp)
    lo, _ = core.bounds()
    return mm(s.move(0, -lo.y, 0))


PL_LEN, PL_W, PL_T = 105.0, 84.0, 6.0
PL_LEG = 9.0


def _drop_outline(n=44):
    """The planchette's outline (x, z): a teardrop, its point at -z, its
    back broad and round"""
    pts = []
    for i in range(n):
        t = TAU * i / n
        pts.append((math.sin(t) * (0.42 + 0.58 * (1 - math.cos(t)) / 2), -math.cos(t)))
    xs = [p[0] for p in pts]
    k = PL_W / (max(xs) - min(xs))
    return [(x * k, z * PL_LEN / 2) for x, z in pts]


def talk_planchette():
    """Swept over its outline: rings of the outline scaled toward its middle,
    the top flat, the edge rounded over, the underside flat; the window cut
    through; three turned legs under it"""
    outl = _drop_outline(48)
    cz = -4.0                  # the rings' centre, a little toward the point
    prof = [(0.0, PL_T), (0.55, PL_T), (0.88, PL_T), (0.955, PL_T - 0.35), (0.985, PL_T - 1.3), (1.0, PL_T / 2),
            (0.985, 1.2), (0.955, 0.3), (0.88, 0.0), (0.55, 0.0), (0.0, 0.0)]
    n = len(outl)
    v, f = [], []
    for k, y in prof:
        if k == 0.0:
            v.append(Vector((0, y, cz)))
        else:
            for x, z in outl:
                v.append(Vector((x * k, y, cz + (z - cz) * k)))
    # rings: index of ring i's first vertex
    starts, at = [], 0
    for k, _ in prof:
        starts.append(at)
        at += 1 if k == 0.0 else n
    for i in range(len(prof) - 1):
        a0, b0 = starts[i], starts[i + 1]
        ka, kb = prof[i][0], prof[i + 1][0]
        for j in range(n):
            j2 = (j + 1) % n
            if ka == 0.0:
                f.append((a0, b0 + j2, b0 + j))
            elif kb == 0.0:
                f.append((a0 + j, a0 + j2, b0))
            else:
                f.append((a0 + j, a0 + j2, b0 + j2, b0 + j))
    body = propkit._fix_winding(Shape(v, f))
    win = cyl(13.0, h=20.0, seg=28, bevel_r=0).move(0, 0, -PL_LEN * 0.5 + 30.0)
    body = cut(body, win)
    body.move(0, PL_LEG, 0)
    legs = Shape()
    for x, z in PL_FEET:
        leg = lathe([(0, 1.5), (3.0, 1.5), (3.4, 3.0), (2.4, 5.5), (2.6, 7.5), (4.0, PL_LEG), (0, PL_LEG + 0.5)], seg=10)
        legs = legs + leg.move(x, 0, z)
    return mm(body + legs)


PL_FEET = [(-24.0, 26.0), (24.0, 26.0), (0.0, -PL_LEN * 0.5 + 14.0)]


def talk_planchette_felt():
    s = Shape()
    for x, z in PL_FEET:
        s = s + cyl(3.6, h=1.5, seg=8, bevel_r=0.5, bevel_seg=1).move(x, 0.75, z)
    return mm(s)


auto('Real__talkBoardCore', talk_board_core, 2400, smooth=30.0)
auto('Real__talkBoardFace', talk_board_face, 2400, smooth=60.0)
auto('Real__talkPlanchette', talk_planchette, 1400, smooth=35.0)
auto('Real__talkPlanchetteFelt', talk_planchette_felt, 200, smooth=40.0)


# ========================================================== tarot card ===
# A tarot card of the Rider deck's size (place_tarot_card_fans): 70 x 120
# mm on card stock 0.35 mm thick, its corners worn round, lying bowed a
# little; a second with a corner dog-eared. Frame: lying face up, x across,
# z along its length (its picture's top at -z), y up; lowest point y = 0.
TC_W, TC_L, TC_T = 70.0, 120.0, 0.35


def _card(bent):
    s = slab(TC_W, TC_L, TC_T, nx=12, ny=20).rot(-math.pi / 2, 0, 0)
    s = _round_corners(s, TC_W, TC_L, 2.4)

    def bow(p):
        u, w = p.x / (TC_W / 2), p.z / (TC_L / 2)
        p.y += 0.5 * u * u + 0.9 * w * w
        if bent:
            # the far right corner folded up along a line 16 mm in from it
            k = 16.0 - ((TC_W / 2 - p.x) + (p.z + TC_L / 2)) * 0.7071
            if k > 0:
                a = math.radians(38)
                p.y += k * math.sin(a)
                pull = k * (1 - math.cos(a)) * 0.7071
                p.x -= pull
                p.z += pull
        return p
    s.warp(bow)
    lo, _ = s.bounds()
    return mm(s.move(0, -lo.y, 0))


auto('Real__tarotCard', lambda: _card(False), 1200, smooth=60.0)
auto('Real__tarotCardBent', lambda: _card(True), 1200, smooth=60.0)


# ======================================================== crystal ball ===
# A clear glass ball 80 mm across (the common size sold for scrying, with
# its stand), on a turned and stained hardwood ring stand 56 mm across and
# 21 mm tall whose cupped top it sits in (place_small_crystal_balls).
CB_R = 40.0
CB_SEAT_R, CB_SEAT_Y = 19.0, 20.4
CB_Y = CB_SEAT_Y + math.sqrt(CB_R ** 2 - CB_SEAT_R ** 2)   # the ball's centre


def crystal_ball_glass():
    return mm(sphere(CB_R, seg=40, rings=20).move(0, CB_Y, 0))


def crystal_ball_stand():
    prof = [(0, 1.2), (20.0, 1.2), (21.5, 0.0), (27.0, 0.0), (28.0, 0.8), (28.2, 3.2), (27.2, 5.0), (25.0, 6.2),
            (23.2, 8.4), (22.8, 11.0), (23.8, 13.6), (25.2, 16.0), (25.6, 18.2), (24.8, 20.2), (22.6, 21.2),
            (20.2, 21.0), (CB_SEAT_R, CB_SEAT_Y), (17.0, 18.6), (12.0, 16.6), (6.0, 15.6), (0, 15.4)]
    return lathe_mm(prof, 36)


auto('Real__crystalBallGlass', crystal_ball_glass, 1700, smooth=80.0, kind='sphere')
auto('Real__crystalBallStand', crystal_ball_stand, 1600, smooth=40.0, kind='cyl')


# ===================================================== memorial shrine ===
# A floor shrine (place_memorial_shrines): a planed pine board 350 x 190 x
# 19 mm laid on the floor; on it a small hardback (r02's) for a riser, a 5 x
# 7 in framed photograph (t01's frame, glass and back) standing on it on its
# easel strut; two or three 1/2 in chime candles burnt down in brass chime
# holders; a stoneware bud vase of dried rosebuds; a small brass cross or
# Star of David on its base; now and then a black ribbon bow on the frame.
# Frame: the board's middle on the floor, its long side across x, its front
# +z.
SB_W, SB_T, SB_D = 350.0, 19.0, 190.0


def shrine_board():
    s = rbox(SB_W, SB_T, SB_D, r=1.6, seg=1).move(0, SB_T / 2, 0)
    # the ends sawn, a little out of square; a dent in the front edge
    def dent(p):
        if p.z > SB_D / 2 - 2.5 and abs(p.x - 62.0) < 9.0 and p.y > SB_T - 3.0:
            p.y -= 0.8 * (1 - abs(p.x - 62.0) / 9.0)
        if p.x > SB_W / 2 - 1:
            p.x += (p.z / SB_D) * 1.6
        return p
    return mm(s.warp(dent))


def shrine_easel():
    """The easel strut on the back of t01's 5 x 7 in frame (that part's
    frame: its middle at y = 0, its back at z = 0): black card over board,
    hinged 20 mm above the middle, its foot reaching the ground behind when
    the frame leans back 14 degrees about its back's bottom edge"""
    th = math.radians(14.0)
    d = 76.0
    foot_y = -101.0 + d * math.tan(th)
    hinge = Vector((0, 20.0, -1.2))
    foot = Vector((0, foot_y, -d))
    axis = foot - hinge
    L = axis.length
    # a strip tapering from 26 to 18 mm, 2 mm thick, along the axis
    outline = [(-13.0, 0.0), (13.0, 0.0), (9.0, L), (-9.0, L)]
    st = prism(outline, 2.0, bevel_r=0.4)            # in xy, along +y, thickness z
    up = axis.normalized()
    side = Vector((1, 0, 0))
    nrm = side.cross(up)
    m = Matrix((side, up, nrm)).transposed()
    st = place(st, m, hinge + nrm * 1.0)
    # the hinge: a cloth tab over the board's back, and the strut's foot cut
    # level (a strip of card bent flat)
    tab = box(24.0, 18.0, 0.6).move(0, 26.0, -0.3)
    return mm(st + tab)


CH_SOCKET_FLOOR = 4.0
CH_TOPS = {'A': 80.0, 'B': 66.0, 'C': 52.0}   # the candle's top above the board (mm)
CH_R = 6.35


def shrine_chime_holder():
    prof = [(0, 0), (16.0, 0), (19.0, 0.6), (20.0, 1.8), (19.8, 3.0), (18.8, 3.6), (17.6, 3.1), (10.5, 2.1),
            (8.6, 2.6), (8.4, 4.0), (8.3, 13.4), (8.8, 14.0), (8.4, 14.8), (7.2, 14.8), (6.9, 14.0),
            (6.9, CH_SOCKET_FLOOR), (0, CH_SOCKET_FLOOR)]
    return lathe_mm(prof, 32)


def shrine_chime_candle(k):
    """A 1/2 in chime candle burnt down to `k`'s height: its crater, a side
    melted through, the runs down it into the socket's mouth and one over
    onto the dish"""
    H = CH_TOPS[k] - CH_SOCKET_FLOOR
    prof = [(0, 0), (CH_R - 0.15, 0), (CH_R, 0.8)]
    for i in range(1, 10):
        y = H * i / 10
        prof.append((CH_R - 0.25 * y / 90.0, y))
    prof += [(CH_R + 0.05, H + 0.5), (5.6, H + 0.9), (4.6, H - 0.5), (2.8, H - 2.2), (0, H - 2.8)]
    s = lathe(prof, seg=20)
    ph = {'A': 0.4, 'B': 2.2, 'C': 4.1}[k]

    def melt(p):
        a = math.atan2(p.z, p.x)
        near = max(0.0, math.cos(a - ph)) ** 3
        if p.y > H - 6:
            t = smoothstep(H - 6, H + 1, p.y)
            p.y -= near * t * 3.2
            r = math.hypot(p.x, p.z)
            if r > 1e-6:
                b = 1 + near * t * 0.08 + 0.03 * noise.noise(Vector((p.x * 0.5, p.y * 0.5, p.z * 0.5)))
                p.x *= b
                p.z *= b
        return p
    s.warp(melt)

    def run(a, y0, y1, rr, over=False):
        pts = []
        for i in range(9):
            t = i / 8
            y = y0 + (y1 - y0) * t
            r = CH_R + 0.1
            if over and y < 11.0:
                r = 8.9 + rr * 0.5 if y > 9.6 else 8.6 + rr * 0.6
            w = 0.07 * math.sin(t * 6 + a)
            pts.append(Vector((math.cos(a + w) * r, y, math.sin(a + w) * r)))
        return tube(pts, rr, seg=6, r_end=rr * 1.3) + ellipsoid(rr * 1.6, rr * 2.0, rr * 1.6, seg=8).move(*pts[-1])

    runs = [run(ph, H - 2, H * 0.45, 0.9), run(ph + 0.7, H - 3, 9.0, 1.1, over=True), run(ph + 2.6, H - 1, H * 0.7, 0.7)]
    collar = lathe([(CH_R, 8.6), (8.0, 9.4), (8.7, 10.4), (8.2, 11.2), (CH_R + 0.2, 11.8)], seg=20)
    s = s + collar + sum(runs, Shape())
    # the drop that ran down the socket onto the dish
    dx, dz = math.cos(ph + 0.7) * 11.5, math.sin(ph + 0.7) * 11.5
    s = s + ellipsoid(3.6, 0.9, 2.8, seg=10).rot(0, -(ph + 0.7), 0).move(dx, 2.4 - CH_SOCKET_FLOOR, dz)
    return mm(s.move(0, CH_SOCKET_FLOOR, 0))


def shrine_chime_wick():
    """The wick standing out of the crater, curled at its tip; y = 0 its root"""
    pts = [Vector((0, -2.0, 0)), Vector((0.05, 2.0, 0)), Vector((0.4, 5.0, 0)), Vector((1.2, 6.8, 0.1)), Vector((2.1, 7.4, 0.3))]
    return mm(tube(pts, 0.5, seg=6, smooth=2, r_end=0.38))


def shrine_vase():
    """A small salt-glazed stoneware bud vase, 62 mm, its neck 13 mm"""
    out = [(0, 0), (12.0, 0), (14.5, 0.8), (16.8, 3.0), (18.0, 8.0), (18.2, 16.0), (17.0, 26.0), (13.0, 36.0),
           (8.5, 44.0), (6.5, 50.0), (6.6, 57.0), (7.8, 60.0), (7.6, 62.0)]
    s = vessel(out, 2.6, seg=28, floor=6.0)
    return mm(_fix(s))


ROSE_STEM = 95.0


def shrine_rose_stem():
    """A dried rose's stem, 95 mm from its cut end (y = 0) to the bud's
    seat, a little crooked, two dried leaves, a few thorns"""
    pts = [Vector((0, 0, 0)), Vector((0.6, 25.0, 0.3)), Vector((-0.4, 50.0, -0.5)), Vector((0.8, 75.0, 0.2)), Vector((0.2, ROSE_STEM, 0))]
    s = tube(pts, 1.3, seg=6, smooth=2, r_end=1.0)
    for y, a in [(34.0, 0.4), (48.0, 2.6), (63.0, 4.4), (82.0, 1.8)]:
        th = cone(0.9, 2.6, seg=5).rot(0, 0, -1.2).rot(0, a, 0)
        s = s + th.move(math.cos(a) * 1.2, y, -math.sin(a) * 1.2)
    for y, a, l in [(44.0, 0.9, 22.0), (70.0, 3.8, 18.0)]:
        # a leaf dried and curled: a narrow blade rolled at its edges
        leaf = slab(l, l * 0.42, 0.25, nx=6, ny=3)
        def curl(p, l=l):
            t = (p.x + l / 2) / l
            w = abs(p.y) / (l * 0.21)
            p.y *= 1 - 0.75 * t * t
            p.z += 2.2 * w * w + 1.5 * t
            return p
        leaf.warp(curl)
        leaf = leaf.rot(0, 0, 0.6).move(l / 2 * 0.8 + 1.0, 0, 0)
        stalk = tube([(0, 0, 0), (2.5, 1.0, 0)], 0.4, seg=4)
        s = s + (leaf + stalk).rot(0, a, 0).move(0, y, 0)
    return mm(s)


def shrine_rose_bud():
    """A rosebud dried on its stem: the petals shrunk tight and creased, the
    sepals curled back; y = 0 where it sits on the stem"""
    bud = lathe([(0, 0), (2.0, 0.5), (4.8, 3.0), (6.8, 7.0), (7.2, 11.0), (6.2, 15.0), (4.0, 18.6), (1.6, 20.6), (0, 21.2)], seg=14)
    def crease(p):
        a = math.atan2(p.z, p.x)
        r = math.hypot(p.x, p.z)
        k = 1 + 0.12 * math.sin(5 * a + p.y * 0.3) + 0.06 * noise.noise(Vector((p.x * 0.6, p.y * 0.6, p.z * 0.6)))
        p.x *= k
        p.z *= k
        p.x += 0.6 * math.sin(p.y * 0.25)
        return p
    bud.warp(crease)
    seps = Shape()
    for i in range(5):
        a = TAU * i / 5 + 0.3
        sep = tube([(0, 2.0, 0), (3.5, 3.0, 0), (6.0, 1.6, 0), (7.2, -1.2, 0)], 0.6, seg=4, r_end=0.2)
        seps = seps + sep.rot(0, a, 0)
    hip = sphere(2.4, seg=8).move(0, 0.8, 0)
    return mm(bud + seps + hip)


def shrine_cross():
    """A small brass standing cross, 88 mm, on a stepped base 38 mm square"""
    base = rbox(38.0, 5.0, 38.0, r=1.2, seg=1).move(0, 2.5, 0)
    base = base + rbox(28.0, 4.0, 28.0, r=1.0, seg=1).move(0, 7.0, 0)
    base = base + rbox(16.0, 4.0, 16.0, r=0.8, seg=1).move(0, 11.0, 0)
    up = rbox(8.0, 76.0, 5.0, r=1.0, seg=1).move(0, 12.5 + 38.0, 0)
    arm = rbox(40.0, 8.0, 5.0, r=1.0, seg=1).move(0, 12.5 + 52.0, 0)
    ends = Shape()
    for x, y in [(0, 12.5 + 76.0), (-20.0, 12.5 + 52.0), (20.0, 12.5 + 52.0)]:
        ends = ends + rbox(10.0, 4.0, 6.0, r=1.2, seg=2).move(x, y, 0).rot(0, 0, 0)
    return mm(base + up + arm + ends)


def shrine_star():
    """A brass Star of David, 64 mm across its points, 4 mm thick, on a
    round turned base 40 mm across"""
    base = lathe([(0, 0), (19.0, 0), (20.0, 1.0), (19.6, 4.0), (14.0, 6.0), (9.0, 8.0), (6.0, 11.0), (0, 11.0)], seg=24)
    R, w = 32.0, 5.0
    tri = []
    for flip in (0, 1):
        o = [(R * math.cos(math.pi / 2 + TAU * k / 3 + flip * math.pi), R * math.sin(math.pi / 2 + TAU * k / 3 + flip * math.pi)) for k in range(3)]
        outer = prism(o, 4.0)
        inner = [(x * (R - w * 2) / R, y * (R - w * 2) / R) for x, y in o]
        ring = cut(outer, prism(inner, 8.0))
        tri.append(ring)
    star = bevel(tri[0] + tri[1], 0.5, seg=1)
    star = star.move(0, 11.0 + R - 1.5, 0)
    return mm(base + star)


def shrine_ribbon_bow():
    """A black satin ribbon (16 mm) tied in a bow on the frame's top left
    corner, its knot over the corner, two loops and a short tail; in t01's
    frame (its middle at y = 0, its front at z = 17.8 mm). y = 0 the knot"""
    w = 16.0

    def band(pts, width, twist=0.0):
        pts = [Vector(p) for p in pts]
        v, f = [], []
        n = len(pts)
        for i, p in enumerate(pts):
            t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
            side = Vector((0, 0, 1)).cross(t)
            if side.length < 1e-6:
                side = Vector((1, 0, 0))
            side.normalize()
            q = twist * i / max(1, n - 1)
            side = side * math.cos(q) + Vector((0, 0, 1)) * math.sin(q)
            v.append(p - side * width / 2)
            v.append(p + side * width / 2)
        for i in range(n - 1):
            f.append((2 * i, 2 * i + 1, 2 * i + 3, 2 * i + 2))
        s = Shape(v, f)
        top = s.copy()
        bot = s.copy().move(0, 0, -0.3)
        bot.f = [tuple(reversed(x)) for x in bot.f]
        return top + bot

    knot = ellipsoid(7.0, 9.0, 4.0, seg=12).move(0, 0, 4.0)
    loops = Shape()
    for sx in (-1, 1):
        pts = []
        for i in range(13):
            t = TAU * i / 12
            pts.append((sx * (14.0 + 13.0 * math.cos(t + math.pi)) + sx * 1.0, 6.0 * math.sin(t) + 3.0 * sx, 3.0 + 2.5 * math.sin(t)))
        loops = loops + band(pts, w * 0.8, twist=0.4 * sx)
    tail = band([(0, -2.0, 3.0), (-3.0, -16.0, 2.2), (-2.0, -30.0, 1.8), (-6.0, -40.0, 1.6)], w * 0.85, twist=0.3)
    return mm(knot + loops + tail)


def shrine_ribbon_tail():
    """The bow's long tail, hanging from the knot (y = 0) down the frame's
    side, its end cut in a swallowtail"""
    w = 15.0
    pts = [(2.0, -3.0, 3.0), (5.0, -25.0, 2.0), (6.5, -50.0, 1.4), (6.0, -75.0, 1.6), (7.5, -96.0, 1.2)]
    v, f = [], []
    n = len(pts)
    for i, p in enumerate(pts):
        p = Vector(p)
        side = Vector((1, 0, 0))
        q = 0.25 * math.sin(i * 1.3)
        side = side * math.cos(q) + Vector((0, 0, 1)) * math.sin(q)
        v.append(p - side * w / 2)
        v.append(p + side * w / 2)
    # the swallowtail
    end = Vector(pts[-1])
    v.append(end + Vector((0, 7.0, 0)))
    for i in range(n - 1):
        f.append((2 * i, 2 * i + 1, 2 * i + 3, 2 * i + 2))
    k = 2 * (n - 1)
    f.append((k, k + 1, len(v) - 1))
    s = Shape(v, f)
    bot = s.copy().move(0, 0, -0.3)
    bot.f = [tuple(reversed(x)) for x in bot.f]
    return mm(s + bot)


auto('Real__shrineBoard', shrine_board, 300, smooth=30.0)
auto('Real__shrineEasel', shrine_easel, 120, smooth=30.0)
auto('Real__shrineChimeHolder', shrine_chime_holder, 1200, smooth=40.0, kind='cyl')
for _k in 'ABC':
    auto(f'Real__shrineChimeCandle{_k}', lambda _k=_k: shrine_chime_candle(_k), 1800, smooth=45.0, kind='cyl')
auto('Real__shrineChimeWick', shrine_chime_wick, 260, smooth=60.0)
auto('Real__shrineVase', shrine_vase, 1600, smooth=40.0, kind='cyl')
auto('Real__shrineRoseStem', shrine_rose_stem, 700, smooth=50.0)
auto('Real__shrineRoseBud', shrine_rose_bud, 700, smooth=50.0)
auto('Real__shrineCross', shrine_cross, 640, smooth=30.0)
auto('Real__shrineStar', shrine_star, 900, smooth=30.0)
auto('Real__shrineRibbonBow', shrine_ribbon_bow, 900, smooth=60.0)
auto('Real__shrineRibbonTail', shrine_ribbon_tail, 120, smooth=60.0)


# ========================================================= chain links ===
# A short length of 3/8 in Grade 30 proof coil chain (place_small_chain_
# links): welded links of 9.9 mm wire, 34.9 mm inside length, 14 mm inside
# width (54.7 x 33.8 mm outside), five to eight of them lying on the floor,
# every other link flat, the ones between standing on edge, leaning; the
# length bending a little as a dropped chain lies. Frame: its middle on the
# floor, the chain along x.
PC_WIRE = 4.95
PC_L, PC_W = 54.7, 33.8
PC_PITCH = 34.9


def chain_length(n, seed):
    lnk = link(PC_L, PC_W, PC_WIRE, n_end=6, tseg=6, weld=0.12)
    rc = PC_W / 2 - PC_WIRE
    random.seed(seed)
    heading = rnd(-0.15, 0.15)
    pos = Vector((0, 0, 0))
    centres = []
    for k in range(n):
        if k:
            heading += rnd(-0.22, 0.22)
            pos = pos + Vector((math.cos(heading), 0, math.sin(heading))) * PC_PITCH
        centres.append((pos.copy(), heading))
    mid = (centres[0][0] + centres[-1][0]) / 2
    s = Shape()
    for k, (c, h) in enumerate(centres):
        d = Vector((math.cos(h), 0, math.sin(h)))
        if k % 2 == 0:
            # flat: its plane the floor's
            m = basis(d, (0, 0, 1))
            m = basis(d, d.cross(Vector((0, 1, 0))))
            y = PC_WIRE
            s = s + place(lnk, m, (c.x - mid.x, y, c.z - mid.z))
        else:
            lean = rnd(0.25, 0.5) * (1 if rnd(0, 1) < 0.5 else -1)
            up = Vector((0, 1, 0))
            side = d.cross(up).normalized()
            u = up * math.cos(lean) + side * math.sin(lean)
            m = basis(d, u)
            y = rc * math.cos(lean) + PC_WIRE
            s = s + place(lnk, m, (c.x - mid.x, y, c.z - mid.z))
    return s.move(0, 0, 0)


for _n in (5, 6, 7, 8):
    auto(f'Real__chainLinks{_n}', lambda _n=_n: mm(chain_length(_n, 1800 + _n)), 1700, smooth=50.0)
