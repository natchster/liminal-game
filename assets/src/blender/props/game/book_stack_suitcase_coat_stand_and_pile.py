"""book_stack_suitcase_coat_stand_and_pile (was r02): the props remodelled at their real sizes (docs/design/PROPS.md), the
everyday ones of the core props: the book stack, the suitcase, the coat
stand and the pile of debris. Their sources: `docs/research/props-remodelling/book_stack_suitcase_coat_stand_and_pile.md`.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part's frame is
given with it: x across its front, y up, z out of its front.

The books (`build_book_stack`): each lies flat, face up, x along its
height (head at -x), y up through its thickness, z across it with the
spine at +z and the fore-edge at -z. A book is three parts so that each
takes its own cell of the stack's painted atlas (box UVs): its boards (or
covers), its spine, its page block.

bookHb{A,B,C}Boards  a cloth hardback's two boards, 2.6 mm greyboard, their
                     corners bumped round, the upper one bowed a little
bookHb{A,B,C}Spine   its rounded spine, the cloth over the stiffener,
                     tucked into the joints
bookHb{A,B,C}Pages   its page block, 3 mm in from the boards at head, tail
                     and fore-edge, the fore-edge concave (rounded and
                     backed), the spine side round
bookPbCovers         a trade paperback's covers, flush with the pages, the
                     front one's corner curled up
bookPbSpine          its square spine
bookPbPages          its page block, the fore-edge a little swollen
bookSlip             a torn slip of paper kept in the pages as a marker

The suitcase (`build_suitcase`), lying on its back: x along it, y up from
the floor, z out of its front (the handle's side). The lid's parts are
modelled shut over the base; the game turns them on the hinge line (along
x at y = SC_HB, z = SC_HZ) by SC_OPEN. The clothes are modelled lying flat
at the origin, their bottom at y = 0, except the cardigan pulled out
(`suitcaseSpill`), modelled where it lies in the case.

suitcaseBase       the base shell: moulded, rounded, a rib round it, its feet
suitcaseLid        the lid shell, its feet
suitcaseBaseLining the base's lining and the two check straps holding the
                   lid open (modelled at SC_OPEN)
suitcaseLidLining  the lid's lining, its shirred pocket and elastic edge
suitcaseBaseTrim   the base's valance and tongue, the hinges' leaves and
                   outer knuckles, the latches, the handle's lugs
suitcaseLidTrim    the lid's valance, its hinge leaves and middle knuckles,
                   the latches' catches
suitcaseHandle     the moulded grip hanging down the front
suitcaseTrousers   trousers folded in three
suitcaseSweater    a knitted jumper folded
suitcaseShirt      a shirt folded on its card, its collar, buttons
suitcaseSocks      a pair of socks rolled into each other
suitcaseSpill      a cardigan pulled half out, a sleeve over the front rim

The coat stand (`build_coat_rack`): y up from the floor, the pole on the
y axis, the front coat hook along +z. The coat: the loop in its collar at
the origin (on the hook), y up, z out from the stand.

coatStandPole      the turned pole, its collars and finial
coatStandLegs      the four bent legs and the ring tying them
coatStandHooks     six coat hooks (a J, a knob) and six hat hooks (a scroll)
coat               a wool overcoat hung by its loop: body, sleeves, collar
coatButtons        its three horn buttons

The pile (`build_pile`): y up from the floor, the mound centred on the
origin; the game lays the pieces on its height (heap_h).

pileHeap           the mound of grit and dust, its ragged edge, grit round it
pileConcreteBits   bits of concrete heaped over it
pilePlasterBits    bits of plaster and gypsum
pileBrickBits      bits of brick
pileBrick          a brick, its arrises chipped
pileHalfBrick      half a brick, broken across
pileConcrete       a lump of broken concrete, and pileConcreteSmall a small one
pileDrywall        a broken piece of 1/2 in drywall, one factory edge
pileTile           a corner of a 5/8 in mineral fibre ceiling tile
"""
from propkit import *
import propkit
import random

# ------------------------------------------------------------- helpers ---


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


def along_x(outline_zy, length, bevel_r=0.0, bevel_seg=1):
    """An outline in the (z, y) plane extruded along x, centred on x = 0."""
    s = prism([(-z, y) for z, y in outline_zy], length, bevel_r=bevel_r, bevel_seg=bevel_seg)
    return s.rot(0, math.pi / 2, 0)


def along_y(outline_xz, depth, bevel_r=0.0, bevel_seg=1):
    """An outline in the (x, z) plane (a plan) extruded up y, centred on y = 0."""
    s = prism([(x, -z) for x, z in outline_xz], depth, bevel_r=bevel_r, bevel_seg=bevel_seg)
    return s.rot(-math.pi / 2, 0, 0)


def arc(z0, y0, y1, bulge, n):
    """Points of a circular arc from (z0, y0) to (z0, y1) whose middle is
    `bulge` out along +z (negative: along -z)."""
    h = (y1 - y0) / 2
    yc = (y0 + y1) / 2
    b = abs(bulge) if abs(bulge) > 1e-6 else 1e-6
    R = (h * h + b * b) / (2 * b)
    th = math.asin(min(1.0, h / R))
    sg = 1 if bulge >= 0 else -1
    zc = z0 + sg * (b - R)
    return [(zc + sg * R * math.cos(t), yc + R * math.sin(t)) for t in (-th + 2 * th * i / n for i in range(n + 1))]


# --------------------------------------------------------------- books ---
BT = 0.0026          # board
SQ = 0.003           # the square: boards past the pages at head, tail, fore-edge
ST = 0.0016          # the spine's cloth over its stiffener

# (height along x, board width along z, page block thickness)
HARDBACKS = {'A': (0.222, 0.146, 0.030), 'B': (0.240, 0.164, 0.044), 'C': (0.192, 0.128, 0.021)}


def hb_dims(key):
    H, W, T = HARDBACKS[key]
    Tt = T + 2 * BT
    zb = W / 2 - 0.004              # the boards' spine edge (the joint begins)
    bulge = 0.13 * Tt               # the spine's round
    return H, W, T, Tt, zb, bulge


def hb_boards(key):
    H, W, T, Tt, zb, bulge = hb_dims(key)
    plan = [(-H / 2, -W / 2), (H / 2, -W / 2), (H / 2, zb), (-H / 2, zb)]
    # the fore-edge corners bumped round, the spine ones barely
    pts = []
    for i, p in enumerate(plan):
        r = 0.0042 if i < 2 else 0.0012
        pts += propkit._round_corner(p, plan[i - 1], plan[(i + 1) % 4], r, 2 if i < 2 else 1)
    lower = along_y(pts, BT, bevel_r=0.0007).move(0, BT / 2, 0)
    upper = along_y(pts, BT, bevel_r=0.0007).move(0, Tt - BT / 2, 0)

    def bow(p):
        # the upper board bowed up toward its fore-edge (damp, then dry)
        t = max(0.0, (zb - p.z) / W)
        return Vector((p.x, p.y + 0.0016 * t * t * (1 - (2 * p.x / H) ** 2 * 0.3), p.z))
    return lower + upper.warp(bow)


def hb_spine(key):
    H, W, T, Tt, zb, bulge = hb_dims(key)
    z0 = zb - 0.0018
    y0, y1 = BT * 0.35, Tt - BT * 0.35
    outer = arc(z0, y0, y1, bulge, 8)
    h = (y1 - y0) / 2
    R = (h * h + bulge * bulge) / (2 * bulge)
    zc = z0 + bulge - R
    yc = (y0 + y1) / 2
    inner = [(zc + (z - zc) * (R - ST) / R, yc + (y - yc) * (R - ST) / R) for z, y in reversed(outer)]
    return along_x(outer + inner, H, bevel_r=0.0005)


def hb_pages(key):
    H, W, T, Tt, zb, bulge = hb_dims(key)
    zf = -W / 2 + SQ
    y0, y1 = BT + 0.0002, BT + T - 0.0002
    fore = arc(zf, y0, y1, 0.55 * bulge, 6)                 # concave: its middle in toward the spine
    zs = zb - 0.003
    apex = zb - 0.0018 + bulge - ST - 0.0006
    spine = arc(zs, y0, y1, apex - zs, 6)
    outline = fore + list(reversed(spine))
    s = along_x(outline, H - 2 * SQ)
    # the fore-edge a little cockled
    return noisy(s, 0.0004, scale=60, seed=3, axis=(0, 0, 1))


PB = (0.216, 0.140, 0.022)       # a trade paperback: height, width, pages
CT = 0.0007                      # its cover card


def pb_covers():
    H, W, T = PB
    back = slab(H, W, CT, 4, 3).rot(-math.pi / 2, 0, 0).move(0, CT / 2, 0)
    front = slab(H, W, CT, 8, 6).rot(-math.pi / 2, 0, 0).move(0, CT + T + CT / 2, 0)

    def curl(p):
        # the front cover lifts at its fore-edge, its tail corner curled up
        u = max(0.0, (-p.z - (W / 2 - 0.06)) / 0.06)           # 0..1 toward the fore-edge
        v = max(0.0, (p.x - (H / 2 - 0.07)) / 0.07)            # 0..1 toward the tail
        lift = 0.002 * u * u + 0.013 * (u * v) ** 1.6
        return Vector((p.x - 0.002 * (u * v) ** 2, p.y + lift, p.z + 0.0015 * (u * v) ** 2))
    return back + front.warp(curl)


def pb_spine():
    H, W, T = PB
    Tt = T + 2 * CT
    out = [(W / 2 - 0.0008, 0.0), (W / 2 + 0.0006, 0.0006), (W / 2 + 0.0009, Tt / 2), (W / 2 + 0.0006, Tt - 0.0006),
           (W / 2 - 0.0008, Tt), (W / 2 - 0.0008 - CT, Tt - CT), (W / 2 - 0.0008 - CT, CT)]
    return along_x(out, H)


def pb_pages():
    H, W, T = PB
    s = slab(H - 0.0006, W - 0.0012, T, 6, 4).rot(-math.pi / 2, 0, 0).move(0, CT + T / 2, -0.0006)

    def swell(p):
        # thicker toward the fore-edge, where the pages fan
        u = max(0.0, (-p.z) / (W / 2))
        k = 1 + 0.06 * u * u
        return Vector((p.x, CT + T / 2 + (p.y - CT - T / 2) * k, p.z))
    return noisy(s.warp(swell), 0.0003, scale=50, seed=5, axis=(0, 1, 0))


def book_slip():
    # a torn strip of paper: the torn end ragged, the strip bent where the
    # pages hold it
    rng = random.Random(41)
    L, w = 0.085, 0.022
    pts = [(-w / 2, 0.0), (w / 2, 0.0)]
    n = 7
    for i in range(n + 1):
        x = w / 2 - w * i / n
        pts.append((x, L - 0.004 + rng.uniform(-0.003, 0.003)))
    s = along_y([(x, -z) for x, z in pts], 0.0004)
    # lying in x-z: its length along -z (out of the fore-edge), then bend
    s = bend(s, 0.25, axis='x', along='z')
    return s


# ------------------------------------------------------- more helpers ---
def sweep(path, section, up=(0, 1, 0), scale=None, caps=True, smooth=0):
    """Sweep a closed 2D section [(a, b)] along a polyline: a along the
    side, b along `up` (carried along the path by parallel transport).
    scale(t) gives (sa, sb) at t in 0..1 along the path. smooth: Chaikin
    passes over the path first."""
    pts = [Vector(p) for p in path]
    for _ in range(smooth):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    n = len(pts)
    tan = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tan.append(t.normalized())
    u = Vector(up)
    side = tan[0].cross(u)
    if side.length < 1e-6:
        side = tan[0].cross(Vector((1, 0, 0)))
    side.normalize()
    v, f = [], []
    m = len(section)
    for i in range(n):
        if i:
            side = (tan[i - 1].rotation_difference(tan[i]) @ side).normalized()
        b = tan[i].cross(side).normalized() * -1
        sa, sb = scale(i / (n - 1)) if scale else (1.0, 1.0)
        for a_, b_ in section:
            v.append(pts[i] + side * a_ * sa + b * b_ * sb)
    for i in range(n - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, (i + 1) * m + j2, (i + 1) * m + j))
    if caps:
        f.append(tuple(reversed(range(m))))
        f.append(tuple((n - 1) * m + j for j in range(m)))
    return propkit._fix_winding(Shape(v, f))


def ellipse_sec(a, b, n=8):
    return [(a * math.cos(TAU * i / n), b * math.sin(TAU * i / n)) for i in range(n)]


def band(outer, inner, y0, y1):
    """A ring between two closed plan outlines [(x, z)] of one count, from
    y0 up to y1."""
    n = len(outer)
    v = [Vector((x, y0, z)) for x, z in outer] + [Vector((x, y1, z)) for x, z in outer] + \
        [Vector((x, y0, z)) for x, z in inner] + [Vector((x, y1, z)) for x, z in inner]
    f = []
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, n + j, n + i))
        f.append((2 * n + j, 2 * n + i, 3 * n + i, 3 * n + j))
        f.append((n + i, n + j, 3 * n + j, 3 * n + i))
        f.append((j, i, 2 * n + i, 2 * n + j))
    return propkit._fix_winding(Shape(v, f))


def pillow(w, d, h, nx=12, nz=10, edge=None, corner=6.0, warp=None):
    """A soft slab (a folded garment, a cushion of cloth): w along x, d along
    z, its bottom at y = 0 and its top at h, the rim a rounded fold (the top
    and bottom grids meet at the rim, upright there), its plan corners
    rounded (a superellipse of power `corner`). warp(x, z, y, top) -> dy
    shapes the top and the bottom (creases, the sag)."""
    edge = edge or h * 0.9
    def ax(n):
        # denser toward the rim
        return [math.sin(math.pi / 2 * (2 * i / n - 1)) * 0.25 + 0.75 * (2 * i / n - 1) for i in range(n + 1)]
    xs, zs = ax(nx), ax(nz)
    v, f = [], []
    idx = {}
    def plan(s, t):
        m = max(abs(s), abs(t))
        if m < 1e-9:
            return 0.0, 0.0
        k = m / (abs(s) ** corner + abs(t) ** corner) ** (1 / corner)
        return s * k * w / 2, t * k * d / 2
    for top in (True, False):
        for j, t in enumerate(zs):
            for i, s in enumerate(xs):
                rim = i in (0, nx) or j in (0, nz)
                if rim and not top:
                    idx[(False, i, j)] = idx[(True, i, j)]
                    continue
                x, z = plan(s, t)
                e = min(w / 2 - abs(x), d / 2 - abs(z))
                e = max(0.0, min(w / 2 * (1 - abs(s)), d / 2 * (1 - abs(t)), e))
                k = 0.0 if rim else min(1.0, e / edge) ** 0.5
                y = h / 2 + (h / 2 * k if top else -h / 2 * k)
                if warp:
                    y += warp(x, z, y, top) * (k if not rim else 0.0) + (warp(x, z, h / 2, None) if rim else 0.0)
                idx[(top, i, j)] = len(v)
                v.append(Vector((x, y, z)))
    for top in (True, False):
        for j in range(nz):
            for i in range(nx):
                q = (idx[(top, i, j)], idx[(top, i + 1, j)], idx[(top, i + 1, j + 1)], idx[(top, i, j + 1)])
                f.append(q if top else tuple(reversed(q)))
    return propkit._fix_winding(Shape(v, f))


# ------------------------------------------------------------ suitcase ---
SC_L, SC_D = 0.584, 0.406            # 23 x 16 in
SC_HB, SC_HL = 0.112, 0.066          # the base's depth, the lid's (7 in closed)
SC_T, SC_LT = 0.0045, 0.0015         # the shell's wall, the lining
SC_R = 0.036                         # the moulding's round
SC_HZ = -SC_D / 2 - 0.006            # the hinge line (along x, at y = SC_HB)
SC_OPEN = math.pi / 2 + 0.21         # how far the lid stands open: held by its check straps


def opened(p):
    """a point of the lid (modelled shut) where it is when the lid stands open"""
    a = -SC_OPEN
    y, z = p[1] - SC_HB, p[2] - SC_HZ
    return Vector((p[0], SC_HB + y * math.cos(a) - z * math.sin(a), SC_HZ + y * math.sin(a) + z * math.cos(a)))


def tray(w, d, h, r, t, seg=3):
    """An open moulded tray, its bottom at y = 0, open at y = h, its walls
    t thick."""
    outer = rbox(w, 2 * h, d, r=r, seg=seg).move(0, h, 0)
    inner = rbox(w - 2 * t, 2 * h, d - 2 * t, r=max(r - t, 0.004), seg=seg).move(0, h + t, 0)
    top = box(w + 0.1, h + 0.1, d + 0.1).move(0, h + (h + 0.1) / 2, 0)
    return cut(outer, top, inner)


def rr(w, d, r, seg=3):
    """a rounded rectangle in plan [(x, z)]"""
    return rounded_rect(w, d, r, seg)


def closed_to_lid(s):
    """a shape modelled as a tray (open at its top) turned over onto the
    base (turned about z, so its z is the case's): the lid as it is when
    the case is shut"""
    return s.rot(0, 0, math.pi).move(0, SC_HB + SC_HL, 0)


def sc_base():
    s = tray(SC_L, SC_D, SC_HB, SC_R, SC_T)
    # moulded feet: two on the back (the case stands on its hinges), two
    # on the right end
    feet = [rbox(0.034, 0.016, 0.010, r=0.0045, seg=2).move(sx * 0.2, 0.052, -SC_D / 2 - 0.002) for sx in (-1, 1)]
    feet += [rbox(0.010, 0.016, 0.034, r=0.0045, seg=2).move(SC_L / 2 + 0.002, 0.052, sz * 0.12) for sz in (-1, 1)]
    # a shallow moulded rib round the shell a third of the way up
    rib = band(rr(SC_L + 0.002, SC_D + 0.002, SC_R + 0.001), rr(SC_L - 0.004, SC_D - 0.004, SC_R - 0.002), 0.036, 0.040)
    return s + sum(feet) + rib


def sc_lid():
    s = tray(SC_L, SC_D, SC_HL, SC_R, SC_T)
    feet = [rbox(0.034, 0.016, 0.010, r=0.0045, seg=2).move(sx * 0.2, 0.040, -SC_D / 2 - 0.002) for sx in (-1, 1)]
    return closed_to_lid(s + sum(feet))


def sc_lining(h):
    s = tray(SC_L - 2 * SC_T, SC_D - 2 * SC_T, h - SC_T, SC_R - SC_T, SC_LT)
    return s.move(0, SC_T, 0)


def sc_base_lining():
    """the lining, and the two check straps (its cloth, 25 mm) from the
    base's ends to the lid's that hold the lid open"""
    s = sc_lining(SC_HB)
    xi = SC_L / 2 - SC_T - SC_LT - 0.0015
    for sx in (-1, 1):
        a = Vector((sx * xi, SC_HB - 0.03, -0.02))
        b = opened(Vector((sx * xi, SC_HB + SC_HL - 0.025, 0.02)))
        mid = a.lerp(b, 0.5) + Vector((0, -0.012, 0.01))
        strap = sweep([tuple(a), tuple(a.lerp(mid, 0.5)), tuple(mid), tuple(mid.lerp(b, 0.5)), tuple(b)],
                      [(-0.0125, -0.0008), (0.0125, -0.0008), (0.0125, 0.0008), (-0.0125, 0.0008)], up=(sx, 0, 0))
        s = s + strap
    return s


def sc_lid_lining():
    s = sc_lining(SC_HL)
    # the shirred pocket across the half by the hinges, its elastic edge
    # gathered; it lies on the lid's floor (y up here, before it is turned)
    w = SC_L - 2 * SC_T - 0.03
    z0, z1 = -SC_D / 2 + SC_T + 0.012, 0.03
    def gather(x, z, y, top):
        if top is False:
            return 0.0
        u = smooth01((z - (z1 - 0.07)) / 0.07)
        return 0.0028 * u * math.sin(x * 2 * math.pi / (w / 12)) + 0.0015 * (1 - u)
    pocket = pillow(w, z1 - z0, 0.006, nx=48, nz=4, edge=0.003, corner=10, warp=gather)
    # its elastic edge
    pocket = pocket + sweep([(x, 0.0068 + 0.0028 * math.sin(x * 2 * math.pi / (w / 12)), (z1 - z0) / 2 - 0.002) for x in
                             [(-w / 2 + 0.008) + (w - 0.016) * i / 48 for i in range(49)]],
                            ellipse_sec(0.003, 0.0024, 5), up=(0, 1, 0))
    pocket.move(0, SC_T + SC_LT, (z0 + z1) / 2)
    return closed_to_lid(s + pocket)


def sc_valance(lid):
    """the aluminium valance capping a half's rim; the base's carries the
    tongue the lid's groove takes"""
    y0, y1 = (SC_HL - 0.013, SC_HL + 0.0015) if lid else (SC_HB - 0.013, SC_HB + 0.0015)
    s = band(rr(SC_L + 0.004, SC_D + 0.004, SC_R + 0.002), rr(SC_L - 2 * SC_T - 0.002, SC_D - 2 * SC_T - 0.002, SC_R - SC_T - 0.001), y0, y1)
    if not lid:
        c = SC_T / 2
        s = s + band(rr(SC_L - SC_T + 0.0012, SC_D - SC_T + 0.0012, SC_R - c + 0.0006), rr(SC_L - SC_T - 0.0012, SC_D - SC_T - 0.0012, SC_R - c - 0.0006), y1, y1 + 0.0035)
    return s


def sc_base_trim():
    s = sc_valance(False)
    for sx in (-1, 1):
        x = sx * 0.17
        # the hinge: its leaf on the back, its two outer knuckles
        s = s + rbox(0.052, 0.026, 0.0022, r=0.001, seg=1).move(x, SC_HB - 0.014, -SC_D / 2 - 0.0011)
        for dx in (-0.017, 0.017):
            s = s + cyl(0.0045, h=0.016, seg=8, bevel_r=0.0008, bevel_seg=1).rot(0, 0, math.pi / 2).move(x + dx, SC_HB, SC_HZ)
        # the latch: its body under the rim, the thumb slide on it
        xl = sx * 0.178
        s = s + rbox(0.046, 0.030, 0.010, r=0.0035, seg=2).move(xl, SC_HB - 0.021, SC_D / 2 + 0.004)
        s = s + rbox(0.016, 0.007, 0.006, r=0.002, seg=1).move(xl, SC_HB - 0.012, SC_D / 2 + 0.010)
        s = s + cyl(0.0042, h=0.004, seg=8, bevel_r=0, bevel_seg=1).rot(math.pi / 2, 0, 0).move(xl, SC_HB - 0.027, SC_D / 2 + 0.0095)
        # the handle's lug
        s = s + rbox(0.014, 0.020, 0.012, r=0.003, seg=1).move(sx * 0.068, SC_HB - 0.022, SC_D / 2 + 0.005)
    return s


def sc_lid_trim():
    s = sc_valance(True)
    for sx in (-1, 1):
        x = sx * 0.17
        s = s + rbox(0.052, 0.024, 0.0022, r=0.001, seg=1).move(x, SC_HL - 0.013, -SC_D / 2 - 0.0011)
        s = s + cyl(0.0045, h=0.017, seg=10, bevel_r=0.0008, bevel_seg=1).rot(0, 0, math.pi / 2).move(x, SC_HL - 0.0002, SC_HZ)
        # the latch's catch
        s = s + rbox(0.038, 0.012, 0.007, r=0.002, seg=1).move(sx * 0.178, SC_HL - 0.009, SC_D / 2 + 0.0035)
    return closed_to_lid(s)


def sc_handle():
    """the moulded grip, hanging down the front from its lugs"""
    zf = SC_D / 2 + 0.010
    y0 = SC_HB - 0.024
    path = [(-0.068, y0, zf), (-0.068, y0 - 0.022, zf + 0.002), (-0.058, y0 - 0.046, zf + 0.004),
            (-0.03, y0 - 0.054, zf + 0.005), (0.03, y0 - 0.054, zf + 0.005), (0.058, y0 - 0.046, zf + 0.004),
            (0.068, y0 - 0.022, zf + 0.002), (0.068, y0, zf)]
    s = sweep(path, ellipse_sec(0.0065, 0.0075, 8), up=(0, 0, 1), smooth=1)
    grip = capsule(0.0105, 0.07, seg=10, cap_rings=3).rot(0, 0, math.pi / 2).scale(1, 1, 0.8).move(0, y0 - 0.053, zf + 0.005)
    return s + grip


def ridge(dist, width, height):
    return height * math.exp(-(dist / width) ** 2)


def smooth01(x):
    x = max(0.0, min(1.0, x))
    return x * x * (3 - 2 * x)


# the clothes in it, each folded, lying flat: x across, y up from what it
# lies on, z along
def gar_trousers():
    """trousers folded in three: the waistband across one end, the crease
    along them, sagging between the fold and the band"""
    w, d, h = 0.40, 0.205, 0.050
    def wp(x, z, y, top):
        if not top:
            return 0.0
        dy = ridge(x - (w / 2 - 0.022), 0.012, 0.004)              # the waistband
        dy += ridge(z - 0.025, 0.004, 0.0016)                      # the crease
        dy -= 0.004 * (1 - (2 * x / w) ** 2) * (1 - (2 * z / d) ** 2)
        dy += ridge(x + 0.06, 0.03, 0.0015)                        # a fold under the top layer
        return dy
    s = pillow(w, d, h, nx=16, nz=10, edge=0.016, warp=wp)
    return noisy(s, 0.0012, scale=25, seed=11, axis=(0, 1, 0))


def gar_sweater():
    """a knitted jumper folded: its ribbed hem along one edge, the ribbed
    neck at the other, the sleeves folded under"""
    w, d, h = 0.32, 0.25, 0.075
    def wp(x, z, y, top):
        if not top:
            return 0.0
        dy = 0.0
        u = z - (d / 2 - 0.035)
        if u > 0:
            dy += 0.0011 * math.sin(x * 2 * math.pi / 0.006) - 0.002     # the rib
        r = math.hypot(x, (z + d / 2) * 1.2)
        dy += ridge(r - 0.075, 0.008, 0.003)                          # the neck's rib
        dy -= ridge(r - 0.04, 0.03, 0.003)
        dy += ridge(abs(x) - 0.095, 0.012, -0.0025)                  # where the sleeves fold under
        dy -= 0.005 * (1 - (2 * x / w) ** 2) * (1 - (2 * z / d) ** 2)
        return dy
    s = pillow(w, d, h, nx=20, nz=16, edge=0.025, warp=wp)
    return noisy(noisy(s, 0.0016, scale=18, seed=12), 0.0006, scale=160, seed=13)


def gar_shirt():
    """a laundered shirt folded round its card: the collar standing at one
    end, the placket and its buttons down the middle"""
    w, d, h = 0.29, 0.205, 0.032
    def wp(x, z, y, top):
        if not top:
            return 0.0
        return ridge(z, 0.008, 0.0012) - 0.002 * (1 - (2 * x / w) ** 2) * (1 - (2 * z / d) ** 2)
    s = pillow(w, d, h, nx=14, nz=10, edge=0.010, warp=wp)
    s = noisy(s, 0.0007, scale=30, seed=14, axis=(0, 1, 0))
    # the collar: a band 32 mm tall round an oval, open at the front where its points lie
    cx = -w / 2 + 0.065
    n = 28
    outer = [(cx + 0.052 * math.cos(TAU * i / n), 0.066 * math.sin(TAU * i / n)) for i in range(n)]
    inner = [(cx + 0.049 * math.cos(TAU * i / n), 0.063 * math.sin(TAU * i / n)) for i in range(n)]
    collar = band(outer, inner, h - 0.004, h + 0.019)
    collar = cut(collar, box(0.05, 0.1, 0.03).move(cx + 0.05, h, 0))
    def roll(p):
        # the band rolled: its top leaning out a little
        k = max(0.0, (p.y - h) / 0.019)
        return Vector((cx + (p.x - cx) * (1 + 0.06 * k), p.y - 0.004 * k * k, p.z * (1 + 0.06 * k)))
    collar = collar.warp(roll)
    points = []
    for sz in (-1, 1):
        tri = [(cx + 0.03, sz * 0.012), (cx + 0.09, sz * 0.006), (cx + 0.07, sz * 0.05)]
        points.append(along_y([(x, -z) for x, z in tri], 0.0025, bevel_r=0.0006).move(0, h + 0.004, 0))
    buttons = [cyl(0.0055, h=0.0025, seg=10, bevel_r=0.0008, bevel_seg=1).move(x, h + 0.0012, 0) for x in (-0.01, 0.05, 0.11)]
    return s + collar + sum(points) + sum(buttons)


def gar_socks():
    """a pair of socks rolled into each other, the cuff turned over the roll"""
    s = ellipsoid(0.055, 0.032, 0.040, seg=16).move(0, 0.032, 0)
    s = noisy(s, 0.003, scale=30, seed=15)
    cuff = torus(0.036, 0.009, seg=16, tseg=8).rot(0, math.pi / 2, 0).scale(1, 0.85, 1.05).move(0.018, 0.032, 0)
    return s + noisy(cuff, 0.0015, scale=40, seed=16)


def gar_spill():
    """a cardigan pulled half out: its body lying on the others, a sleeve
    over the front rim and down to the floor (modelled in the case's frame)"""
    def wp(x, z, y, top):
        if not top:
            return 0.0
        return -0.004 * (1 - (2 * x / 0.30) ** 2) + ridge(z - 0.07, 0.01, 0.003)
    body = pillow(0.30, 0.20, 0.034, nx=14, nz=10, edge=0.012, warp=wp)
    body = noisy(body, 0.002, scale=16, seed=17)
    body.rot(0, -0.22, 0).move(0.075, 0.084, 0.07)
    path = [(0.13, 0.104, 0.10), (0.145, 0.118, 0.16), (0.150, 0.124, 0.193), (0.153, 0.126, 0.212), (0.157, 0.112, 0.226),
            (0.160, 0.075, 0.232), (0.163, 0.040, 0.235), (0.166, 0.016, 0.236)]
    def sc(t):
        return (1.0 - 0.22 * t + 0.03 * math.sin(t * 17), 1.0 - 0.1 * t)
    sleeve = sweep(path, ellipse_sec(0.040, 0.011, 10), up=(0, 1, 0), scale=sc, smooth=2)
    # the ribbed cuff
    rib = [(0.034 * (1 + 0.05 * (i % 2)) * math.cos(TAU * i / 20), 0.010 * (1 + 0.08 * (i % 2)) * math.sin(TAU * i / 20)) for i in range(20)]
    cuff = sweep([(0.1665, 0.016, 0.236), (0.1675, 0.012, 0.25), (0.168, 0.010, 0.262)], rib, up=(0, 0, 1))
    sleeve = sleeve + cuff
    return body + noisy(sleeve, 0.0015, scale=30, seed=18)


# ------------------------------------------------------------ coat stand ---
# A bentwood coat stand of the Thonet kind: a turned beech pole, four bent
# legs on an 18 in square, a ring tying them, six hooks for coats and six
# above them for hats (74 in tall). Frame: y up from the floor, the pole on
# the y axis; the lower hook at angle 0 points along +z.
CS_H = 1.88
CS_HOOK_LO, CS_HOOK_HI = 1.52, 1.70
CS_LEGS_R = 0.32


def cs_pole():
    prof = [(0.0, 0.07), (0.012, 0.07), (0.019, 0.085), (0.021, 0.11), (0.0215, 0.50),
            (0.026, 0.51), (0.028, 0.53), (0.028, 0.60), (0.026, 0.62), (0.0205, 0.63),
            (0.019, CS_HOOK_LO - 0.04), (0.024, CS_HOOK_LO - 0.03), (0.026, CS_HOOK_LO - 0.015), (0.026, CS_HOOK_LO + 0.02),
            (0.024, CS_HOOK_LO + 0.03), (0.0185, CS_HOOK_LO + 0.04), (0.018, CS_HOOK_HI - 0.04), (0.024, CS_HOOK_HI - 0.03),
            (0.026, CS_HOOK_HI - 0.015), (0.026, CS_HOOK_HI + 0.02), (0.024, CS_HOOK_HI + 0.03), (0.0175, CS_HOOK_HI + 0.04),
            (0.017, 1.80), (0.022, 1.81), (0.028, 1.83), (0.029, 1.845), (0.025, 1.862), (0.014, 1.874), (0.006, 1.879), (0.0, CS_H)]
    return lathe(prof, seg=14)


def leg_path():
    return [(0.021, 0.60), (0.023, 0.52), (0.045, 0.42), (0.09, 0.33), (0.15, 0.24), (0.21, 0.15),
            (0.26, 0.08), (0.295, 0.035), (0.318, 0.013), (0.34, 0.010), (0.355, 0.017)]


def cs_legs():
    s = Shape()
    for k in range(4):
        a = math.pi / 4 + k * math.pi / 2
        pts = [(r * math.sin(a), y, r * math.cos(a)) for r, y in leg_path()]
        s = s + sweep(pts, ellipse_sec(0.0105, 0.0135, 7), up=(math.cos(a), 0, -math.sin(a)), smooth=1)
    # the ring tying the legs, outside them, screwed to each
    ring = torus(0.150, 0.0085, seg=40, tseg=6).rot(math.pi / 2, 0, 0).move(0, 0.24 + 0.002, 0)
    return s + ring


def cs_hooks():
    s = Shape()
    lo = [(0.016, CS_HOOK_LO + 0.008), (0.05, CS_HOOK_LO + 0.002), (0.09, CS_HOOK_LO - 0.008), (0.122, CS_HOOK_LO - 0.012),
          (0.142, CS_HOOK_LO - 0.006), (0.152, CS_HOOK_LO + 0.012), (0.154, CS_HOOK_LO + 0.036), (0.15, CS_HOOK_LO + 0.05)]
    hi = [(0.016, CS_HOOK_HI + 0.004), (0.06, CS_HOOK_HI + 0.008), (0.11, CS_HOOK_HI + 0.03), (0.15, CS_HOOK_HI + 0.075),
          (0.168, CS_HOOK_HI + 0.12), (0.162, CS_HOOK_HI + 0.155), (0.14, CS_HOOK_HI + 0.168), (0.122, CS_HOOK_HI + 0.158),
          (0.12, CS_HOOK_HI + 0.142), (0.13, CS_HOOK_HI + 0.134)]
    for k in range(6):
        for path, a0, r in ((lo, 0.0, 0.0072), (hi, math.pi / 6, 0.0080)):
            a = a0 + k * math.pi / 3
            pts = [(rr_ * math.sin(a), y, rr_ * math.cos(a)) for rr_, y in path]
            s = s + sweep(pts, ellipse_sec(r * 0.85, r, 6), up=(math.cos(a), 0, -math.sin(a)), smooth=1)
        # the knob at the coat hook's tip
        a = k * math.pi / 3
        s = s + sphere(0.0105, seg=8, rings=5).move(0.150 * math.sin(a), CS_HOOK_LO + 0.052, 0.150 * math.cos(a))
    return s


# ----------------------------------------------------------------- coat ---
# A wool overcoat (size 42: 36 in from the collar seam to the hem, 46 in
# round the chest, 26 in sleeves) hung by the loop inside its collar on a
# hook, as a coat hangs on a stand: the collar pulled up to a point at the
# hook, the front of the neck fallen open in a V, the shoulders falling away
# steeply with the drag of the hook running out to them, the sleeves hanging
# at its sides, the body heavy and folded, its front edges overlapping.
# Frame: the loop on the hook at the origin, y up, z out from the stand
# (its front).
CT_LEN = 0.95


def lerp_table(tab, t):
    for (t0, v0), (t1, v1) in zip(tab, tab[1:]):
        if t <= t1:
            u = (t - t0) / (t1 - t0) if t1 > t0 else 0.0
            return v0 + (v1 - v0) * (u * u * (3 - 2 * u))
    return tab[-1][1]


CT_Y = [(0.0, -0.03), (0.08, -0.095), (0.16, -0.175), (0.26, -0.29), (0.45, -0.48), (1.0, -CT_LEN - 0.02)]
CT_W = [(0.0, 0.062), (0.08, 0.105), (0.16, 0.152), (0.26, 0.198), (0.45, 0.224), (0.75, 0.238), (1.0, 0.258)]
CT_D = [(0.0, 0.046), (0.12, 0.062), (0.30, 0.072), (0.7, 0.066), (1.0, 0.064)]
CT_C = [(0.0, 0.046), (0.12, 0.026), (0.30, 0.004), (1.0, 0.0)]
CT_FOLDS = [(5, 0.3, 1.1, 0.9), (7, 1.7, -0.6, 0.6), (11, 2.9, 0.4, 0.3), (3, 0.9, 0.3, 0.55), (16, 0.2, 2.0, 0.15)]   # (waves, phase, drift, weight)


def seg_dist(px, py, ax, ay, bx, by):
    dx, dy = bx - ax, by - ay
    u = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)))
    return math.hypot(px - ax - u * dx, py - ay - u * dy)


def coat_point(t, th, lift=0.0):
    """the body's surface at t (0 the neck, 1 the hem) and th round it
    (pi/2 the front), pushed out by `lift`"""
    y = lerp_table(CT_Y, t)
    w = lerp_table(CT_W, t)
    d = lerp_table(CT_D, t)
    c = lerp_table(CT_C, t)
    ct, st = math.cos(th), math.sin(th)
    # the neck fallen open at the front: a V down to the top button
    if st > 0:
        y -= 0.10 * max(0.0, 1 - t / 0.30) ** 1.6 * st ** 3
    y += 0.02 * math.sin(3 * th + 1.0) * t ** 5 + 0.008 * math.sin(7 * th + 2.0) * t ** 6   # the hem uneven
    x = w * math.copysign(abs(ct) ** 0.62, ct)
    z = c + d * math.copysign(abs(st) ** 0.85, st)
    amp = 0.002 + 0.034 * t ** 1.3
    fold = 0.0
    for m, ph, drift, wt in CT_FOLDS:
        fold += wt * math.sin(m * th + ph + drift * t * 3)
    fold *= amp
    # the drag of the hook: ridges from the loop out to each shoulder
    drag = 0.0
    if t < 0.35:
        for sx in (-1, 1):
            drag += ridge(seg_dist(x, y, 0.0, -0.02, sx * 0.17, -0.27), 0.016, 0.008)
        drag *= 1 - t / 0.35
    # the front: the left front over the right, its edge a little right of
    # the middle; above the top button the lapels turned back
    front = 0.0
    if st > 0:
        xe = 0.03
        front = 0.0055 * smooth01((xe - x) / 0.006 + 0.5) * st
        if t < 0.32:
            k = 1 - t / 0.32
            lap = 0.16 * k
            front += 0.006 * smooth01((lap - abs(x)) / 0.01 + 0.5) * smooth01((abs(x) - 0.02) / 0.01) * st
            front += ridge(abs(x) - lap, 0.008, 0.004) * st
    n = Vector((ct * d, 0.0, st * w)).normalized()
    p = Vector((x, y, z)) + n * (fold + drag + front + lift)
    a = 0.06 * t
    return Vector((p.x * math.cos(a) + p.z * math.sin(a), p.y, -p.x * math.sin(a) + p.z * math.cos(a) + 0.012 * t * t))


def ct_body():
    nt, nth = 30, 40
    ts = [(i / nt) ** 1.12 for i in range(nt + 1)]
    v, f = [], []
    for t in ts:
        for j in range(nth):
            v.append(coat_point(t, TAU * j / nth - math.pi / 2))
    for i in range(nt):
        for j in range(nth):
            j2 = (j + 1) % nth
            f.append((i * nth + j, i * nth + j2, (i + 1) * nth + j2, (i + 1) * nth + j))
    def inset(first, dy, k):
        ring = []
        cx = sum(v[first + j].x for j in range(nth)) / nth
        cz = sum(v[first + j].z for j in range(nth)) / nth
        for j in range(nth):
            p = v[first + j]
            ring.append(len(v))
            v.append(Vector((cx + (p.x - cx) * k, p.y + dy, cz + (p.z - cz) * k)))
        for j in range(nth):
            j2 = (j + 1) % nth
            f.append((first + j2, first + j, ring[j], ring[j2]))
        f.append(tuple(ring))
    inset(nt * nth, 0.04, 0.93)       # the hem turned up inside, the lining over it
    inset(0, -0.05, 0.85)             # the neck: the lining inside the back
    s = propkit._fix_winding(Shape(v, f))
    return noisy(s, 0.0025, scale=7, seed=21)


def ct_sleeve(sx):
    side = 0.0 if sx > 0 else math.pi
    sh = coat_point(0.17, side)
    path = [sh + Vector((-sx * 0.05, 0.02, 0.0)), sh + Vector((sx * 0.004, -0.03, 0.006)),
            sh + Vector((sx * 0.024, -0.12, 0.022)), sh + Vector((sx * 0.03, -0.27, 0.038)),
            sh + Vector((sx * 0.028, -0.42, 0.052)), sh + Vector((sx * 0.022, -0.54, 0.058)),
            sh + Vector((sx * 0.017, -0.63, 0.054))]
    def sc(t):
        k = 1 - 0.27 * t
        w = 1 + 0.05 * math.sin(t * 23 + sx) * (1 - t) + 0.06 * ridge(t - 0.55, 0.05, 1.0)
        return (k * w * (1 + 0.12 * ridge(t - 0.12, 0.1, 1.0)), (1 - 0.15 * t) * (1 + 0.2 * ridge(t - 0.12, 0.1, 1.0)))
    s = sweep([tuple(p) for p in path], ellipse_sec(0.072, 0.038, 12), up=(sx, 0, 0), scale=sc, smooth=2)
    return noisy(s, 0.003, scale=9, seed=22 if sx > 0 else 23)


def ct_collar():
    """the collar standing round the neck, highest at the back where the
    loop pulls it, open at the front; and the loop"""
    n = 26
    path = []
    for j in range(n + 1):
        th = math.pi / 2 + 0.55 + (TAU - 1.1) * j / n
        p = coat_point(0.0, th, lift=0.004)
        back = ridge(math.sin(th) + 1, 0.6, 1.0)
        path.append((p.x, p.y + 0.012 + 0.018 * back, p.z))
    def sc(t):
        return (1.0, 0.55 + 0.45 * math.sin(math.pi * t))
    s = sweep(path, ellipse_sec(0.006, 0.028, 8), up=(0, 1, 0), scale=sc, caps=True)
    loop = torus(0.011, 0.0025, seg=10, tseg=5).rot(0, math.pi / 2, 0).move(0, 0.006, 0.001)
    return noisy(s, 0.0015, scale=20, seed=24) + loop


def ct_coat():
    return ct_body() + ct_sleeve(1) + ct_sleeve(-1) + ct_collar()


def ct_buttons():
    out = Shape()
    for t in (0.34, 0.49, 0.64):
        p = coat_point(t, math.pi / 2)
        out = out + cyl(0.0115, h=0.005, seg=12, bevel_r=0.0015, bevel_seg=1).rot(math.pi / 2, 0, 0).move(p.x, p.y, p.z + 0.003)
    return out


# ----------------------------------------------------------------- pile ---
# A heap of debris swept into a corner of a store or a tunnel: grit and
# plaster dust in a low mound, bits of brick and concrete through it, and
# on it the bigger pieces the game lays (a brick, half bricks, lumps of
# concrete, broken drywall, a broken ceiling tile). Frame: y up from the
# floor, the mound centred on the origin. The mound's height is
# PL_H * (1 - (x/PL_RX)^2 - (z/PL_RZ)^2)^1.3 (less its noise): the game lays
# the pieces on it.
PL_RX, PL_RZ, PL_H = 0.28, 0.235, 0.14


def heap_h(x, z):
    q = 1 - (x / PL_RX) ** 2 - (z / PL_RZ) ** 2
    return PL_H * q ** 1.3 if q > 0 else 0.0


def chunk(sx, sy, sz, seed, cuts=6, bite=0.32, r=0.002):
    """a broken lump: a block with corners and edges broken off along
    random planes (its fractures flat and sharp), a little uneven"""
    rng = random.Random(seed)
    s = rbox(sx, sy, sz, r=r, seg=1)
    for _ in range(cuts):
        n = Vector((rng.uniform(-1, 1), rng.uniform(-0.6, 1), rng.uniform(-1, 1))).normalized()
        ext = abs(n.x) * sx / 2 + abs(n.y) * sy / 2 + abs(n.z) * sz / 2
        d = ext * (1 - bite * rng.uniform(0.4, 1.0))
        cutter = box(2, 2, 2).move(0, 0, 1)
        q = Vector((0, 0, 1)).rotation_difference(n).to_matrix().to_4x4()
        cutter.apply(q)
        cutter.move(*(n * d))
        s = cut(s, cutter)
    return s.warp(lambda p: p + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * 0.0012)


def jag_outline(pts, step, amp, seed):
    """a broken outline: each edge of `pts` marked (True) as broken gets
    jagged every `step` metres by up to `amp`"""
    rng = random.Random(seed)
    out = []
    n = len(pts)
    for i in range(n):
        (x0, z0, broken), (x1, z1, _) = pts[i], pts[(i + 1) % n]
        L = math.hypot(x1 - x0, z1 - z0)
        k = max(1, int(L / step)) if broken else 1
        nx, nz = (z1 - z0) / L, -(x1 - x0) / L
        for j in range(k):
            u = j / k
            o = rng.uniform(-amp, amp) if (broken and j) else 0.0
            out.append((x0 + (x1 - x0) * u + nx * o, z0 + (z1 - z0) * u + nz * o))
    return out


def pl_brick(half=False):
    """a common brick (194 x 57 x 92 mm), its arrises chipped; or the half
    of one broken across, its break rough"""
    L = 0.104 if half else 0.194
    s = rbox(L, 0.057, 0.092, r=0.003, seg=1)
    rng = random.Random(31 if half else 32)
    bites = []
    for _ in range(5):
        c = Vector((rng.choice((-1, 1)) * L / 2, rng.choice((-1, 1)) * 0.0285, rng.uniform(-0.046, 0.046)))
        bites.append(rbox(0.03, 0.02, 0.025, r=0.004, seg=1).rot(rng.uniform(0, 3), rng.uniform(0, 3), 0).move(*c))
    s = cut(s, *bites)
    if half:
        # the break: a ragged face across the end
        brk = noisy(slab(0.08, 0.12, 0.08, 6, 6).rot(0, math.pi / 2, 0), 0.008, scale=28, seed=33).move(L / 2 + 0.04 - 0.012, 0, 0)
        s = cut(s, brk)
    return s.move(0, 0.0285, 0)


def pl_concrete(size, seed):
    sx, sy, sz = size
    return chunk(sx, sy, sz, seed, cuts=8, bite=0.35).move(0, sy / 2 * 0.8, 0)


def pl_drywall():
    """a broken piece of 1/2 in gypsum board: one factory edge (its long
    taper), the rest snapped and crumbled, a little sagged from the wet"""
    pts = [(-0.17, -0.11, False), (0.15, -0.11, True), (0.18, 0.02, True), (0.06, 0.12, True), (-0.12, 0.09, True), (-0.19, 0.0, True)]
    o = jag_outline(pts, 0.014, 0.006, 41)
    s = along_y(o, 0.0127).move(0, 0.0127 / 2, 0)
    s = bend(s, 0.10, axis='z', along='x')
    return s


def pl_tile():
    """a corner broken off a 2 x 2 ft lay-in mineral fibre ceiling tile,
    5/8 in: two factory edges, the break ragged, sagged"""
    pts = [(-0.13, -0.12, False), (0.16, -0.12, True), (0.10, 0.03, True), (0.02, 0.07, True), (-0.13, 0.15, False)]
    o = jag_outline(pts, 0.012, 0.007, 42)
    s = along_y(o, 0.0159).move(0, 0.0159 / 2, 0)
    s = bend(s, -0.12, axis='z', along='x')
    return noisy(s, 0.0008, scale=60, seed=43, axis=(0, 1, 0))


def rock(sx, sy, sz, seed, n=9):
    """a fragment: the hull of a few points in an ellipsoid, its faces flat
    and its edges sharp, as a broken bit is"""
    rng = random.Random(seed)
    bm = bmesh.new()
    for _ in range(n):
        while True:
            q = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
            if q.length <= 1:
                break
        q = q.normalized() * (0.55 + 0.45 * q.length)
        bm.verts.new((q.x * sx / 2, q.y * sy / 2, q.z * sz / 2))
    bmesh.ops.convex_hull(bm, input=bm.verts[:])
    out = from_bmesh(bm, keep_sharp=False)
    bm.free()
    return propkit._fix_winding(out)


def grit_dy(x, y, z):
    """the mound's grit: what it adds to heap_h where the mound is up"""
    p = Vector((x, y, z))
    k = min(1.0, y / 0.02)
    return (noise.noise(p * 8 + Vector((3, 1, 7))) * 0.018 + noise.noise(p * 37) * 0.006) * k


def surface(x, z):
    h = heap_h(x, z)
    return max(0.0, h + grit_dy(x, h, z))


def strew(count, seed, size, lift, spread=1.0):
    """`count` fragments over the mound, more and bigger toward its middle,
    half sunk in it"""
    rng = random.Random(seed)
    out = Shape()
    for k in range(count):
        a = rng.uniform(0, TAU)
        r = rng.uniform(0.0, 1.0) ** 0.55 * spread
        x, z = r * PL_RX * math.cos(a), r * PL_RZ * math.sin(a)
        sz = rng.uniform(size[0], size[1]) * (1.25 - 0.55 * r)
        c = rock(sz, sz * rng.uniform(0.35, 0.7), sz * rng.uniform(0.55, 0.95), seed * 1000 + k)
        c.rot(rng.uniform(-0.4, 0.4), rng.uniform(0, 3.2), rng.uniform(-0.4, 0.4))
        low = min(p.y for p in c.v)
        # half sunk in the mound, never through the floor
        c.move(x, max(surface(x, z) + sz * rng.uniform(0.05, lift + 0.1), 0.0005 - low), z)
        out = out + c
    return out


def pl_heap():
    """the mound: grit and dust, its edge ragged where it was swept, grit
    strewn round it"""
    nr, na = 10, 32
    ragged = [1.0 + 0.18 * noise.noise(Vector((math.cos(TAU * j / na) * 1.7, math.sin(TAU * j / na) * 1.7, 0.3))) for j in range(na)]
    v = [Vector((0.0, heap_h(0, 0), 0.0))]
    f = []
    for i in range(1, nr + 1):
        r = (i / nr) ** 0.85
        for j in range(na):
            a = TAU * j / na
            k = min(1.0, i / (nr * 0.6))
            rr_ = r * (1.0 + (ragged[j] - 1.0) * k) * 1.06
            x, z = rr_ * PL_RX * math.cos(a), rr_ * PL_RZ * math.sin(a)
            v.append(Vector((x, heap_h(x / 1.06, z / 1.06) if i < nr else 0.0, z)))
    for j in range(na):
        f.append((0, 1 + (j + 1) % na, 1 + j))
    for i in range(nr - 1):
        for j in range(na):
            a0, a1 = 1 + i * na + j, 1 + i * na + (j + 1) % na
            f.append((a0, a1, a1 + na, a0 + na))
    s = Shape(v, f)
    s = s.warp(lambda p: Vector((p.x, max(0.0, p.y + grit_dy(p.x, p.y, p.z)), p.z)))
    return s + strew(40, 10, (0.012, 0.04), 0.4, spread=1.05)


def pl_concrete_bits():
    """the grey bits of concrete heaped over it"""
    return strew(150, 7, (0.02, 0.07), 0.3)


def pl_plaster_bits():
    """pale bits of plaster and gypsum among them"""
    return strew(48, 8, (0.02, 0.065), 0.3) + strew(14, 11, (0.012, 0.035), 0.4, spread=1.05)


def pl_brick_bits():
    """bits of brick among them"""
    return strew(30, 9, (0.02, 0.06), 0.3, spread=1.05)


# ---------------------------------------------------------------- parts ---
auto('Real__bookHbABoards', lambda: hb_boards('A'), 420)
auto('Real__bookHbASpine', lambda: hb_spine('A'), 200)
auto('Real__bookHbAPages', lambda: hb_pages('A'), 200)
auto('Real__bookHbBBoards', lambda: hb_boards('B'), 420)
auto('Real__bookHbBSpine', lambda: hb_spine('B'), 200)
auto('Real__bookHbBPages', lambda: hb_pages('B'), 200)
auto('Real__bookHbCBoards', lambda: hb_boards('C'), 420)
auto('Real__bookHbCSpine', lambda: hb_spine('C'), 200)
auto('Real__bookHbCPages', lambda: hb_pages('C'), 200)
auto('Real__bookPbCovers', pb_covers, 340)
auto('Real__bookPbSpine', pb_spine, 100)
auto('Real__bookPbPages', pb_pages, 300)
auto('Real__bookSlip', book_slip, 100)
auto('Real__suitcaseBase', sc_base, 1400)
auto('Real__suitcaseLid', sc_lid, 1300)
auto('Real__suitcaseBaseLining', sc_base_lining, 900)
auto('Real__suitcaseLidLining', sc_lid_lining, 1600)
auto('Real__suitcaseBaseTrim', sc_base_trim, 1200)
auto('Real__suitcaseLidTrim', sc_lid_trim, 800)
auto('Real__suitcaseHandle', sc_handle, 500)
auto('Real__suitcaseTrousers', gar_trousers, 700)
auto('Real__suitcaseSweater', gar_sweater, 1300)
auto('Real__suitcaseShirt', gar_shirt, 1200)
auto('Real__suitcaseSocks', gar_socks, 900)
auto('Real__suitcaseSpill', gar_spill, 1400)
auto('Real__coatStandPole', cs_pole, 900)
auto('Real__coatStandLegs', cs_legs, 1700)
auto('Real__coatStandHooks', cs_hooks, 3000)
auto('Real__coat', ct_coat, 4600)
auto('Real__coatButtons', ct_buttons, 300)
auto('Real__pileHeap', pl_heap, 1400)
auto('Real__pileConcreteBits', pl_concrete_bits, 2200)
auto('Real__pilePlasterBits', pl_plaster_bits, 900)
auto('Real__pileBrickBits', pl_brick_bits, 800)
auto('Real__pileBrick', lambda: pl_brick(False), 300)
auto('Real__pileHalfBrick', lambda: pl_brick(True), 500)
auto('Real__pileConcrete', lambda: pl_concrete((0.17, 0.10, 0.13), 51), 200)
auto('Real__pileConcreteSmall', lambda: pl_concrete((0.10, 0.065, 0.085), 52), 200)
auto('Real__pileDrywall', pl_drywall, 600)
auto('Real__pileTile', pl_tile, 600)
