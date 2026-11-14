"""r14b: small things a person dropped or left, remodelled at their real
sizes (docs/design/PROPS.md, P2): a pair of clear safety glasses, a wristwatch, a
bunch of keys on a split ring with a tag, reading glasses (on the book stack
of batch r02), curled gift ribbon, a twin-bell wind-up alarm clock. Sources
and sizes: docs/research/props-remodelling/small_things_dropped_or_left.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
prop's own frame: x across its front, y up from the floor (y = 0 the floor
it rests on), z out of its front. Things that lie as they fell (the
glasses, the keys) are modelled whole in that pose, settled onto the floor
here (`settle`: dropped onto their lowest point and tipped until they rest
on three), so the game places them as they are.

safetyGlassesOpenLens / OpenFrame     wraparound safety spectacles (125 mm
        across, the lens 40 mm deep, temples 132 mm), standing open as they
        were put down: on the lens's foot and the temples' tips
safetyGlassesFoldedLens / FoldedFrame the same folded, lying on their
        temples, the lens up
        (Lens: one polycarbonate lens with its side shields and nose notch,
        2.2 mm; Frame: the nylon temples, their hinge blocks and screws, the
        soft nose pads)
readersFrame / readersLenses          plastic reading glasses (50-20-135),
        folded, lying on their temples
watchCase       a 38 mm quartz wristwatch's case lying on its back: the
                caseback, the mid-case and its four lugs, the sloped bezel,
                the crown; with the strap's tang buckle and its tongue
watchCrystal    its domed acrylic crystal
watchStrap      the two-piece strap, unbuckled and lying out from the lugs:
                padded, its tip curled up, seven holes, two keepers
watchHour, watchMinute, watchSecond   the hands, each pointing to 12 (-z)
                about the pivot at x = z = 0, the second hand with its hub
keysKey0..4     five keys lying fanned on one another, their bows bunched
                on the ring: a 5-pin house key (KW1 class), a larger one
                (SC1 class), a padlock key, a second house key cut
                differently, a small cabinet key; each cut brass, the game
                plates it nickel or not
keysRing        the split ring (25 mm, two turns of flat wire) leaning on
                the bows, and the key tag's metal rim and its jump ring
keysTagPaper    the tag's paper face
ribbonKnot      curling ribbon cut off a parcel: the knot and four curled
                tails, trodden flat in places
ribbonLength    a single cut length lying in a loose S, a half twist in it,
                its ends curled in ringlets
alarmCase       a twin-bell wind-up alarm clock's drum (104 mm): the case
                and its domed back cover, painted steel
alarmChrome     its bright parts: the bezel, the two bells on their posts,
                the hammer, the carrying handle, the feet, the wind keys
                and knobs at the back, the hub over the hands
alarmGlass      the domed glass
alarmHour, alarmMinute, alarmSet   the hands, pointing to 12 (+y) about
                the pivot
"""
from propkit import *
import propkit

# ------------------------------------------------------------------ util --
def on_floor(s, y0=0.0):
    lo, hi = s.bounds()
    return s.move(0, y0 - lo.y, 0)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def transform_all(shapes, m):
    for s in shapes:
        s.apply(m)
    return shapes


def _rot_about(axis, angle, pivot):
    return Matrix.Translation(pivot) @ Matrix.Rotation(angle, 4, axis) @ Matrix.Translation(-pivot)


def _first_contact(pts, pivot, axis, exclude):
    """Rotating about the horizontal `axis` through `pivot` (the sense that
    lowers whatever is on the far side), the smallest angle at which another
    point reaches the floor (y = pivot.y)."""
    best = math.pi / 2
    for i, q in enumerate(pts):
        if i in exclude:
            continue
        r = q - pivot
        if r.y < 1e-6:
            continue
        c = axis.cross(r).y
        # r.y cos(t) + c sin(t) = 0, the smallest t > 0
        t = math.atan2(r.y, -c)
        if 0 < t < best:
            best = t
    return best


def settle(shapes):
    """Drop the shapes (moved as one body) onto the floor and tip them over
    their lowest point, then about the line of their two lowest, until they
    rest on three points (their mean vertex standing for the centre of
    mass)."""
    pts = [p for s in shapes for p in s.v]
    com = sum(pts, Vector((0, 0, 0))) / len(pts)
    i0 = min(range(len(pts)), key=lambda i: pts[i].y)
    transform_all(shapes, Matrix.Translation((0, -pts[i0].y, 0)))
    pts = [p for s in shapes for p in s.v]
    com = sum(pts, Vector((0, 0, 0))) / len(pts)
    p0 = pts[i0].copy()
    d = Vector((com.x - p0.x, 0, com.z - p0.z))
    if d.length < 1e-6:
        return shapes
    d.normalize()
    axis = Vector((0, 1, 0)).cross(d).normalized()
    # the sense that lowers the centre
    if axis.cross(com - p0).y > 0:
        axis = -axis
    near = {i for i, q in enumerate(pts) if (q - p0).length < 1e-4}
    t = _first_contact(pts, p0, axis, near)
    transform_all(shapes, _rot_about(axis, t, p0))
    pts = [p for s in shapes for p in s.v]
    com = sum(pts, Vector((0, 0, 0))) / len(pts)
    i1 = min((i for i in range(len(pts)) if (pts[i] - p0).length > 0.004), key=lambda i: pts[i].y)
    p1 = pts[i1].copy()
    line = Vector((p1.x - p0.x, 0, p1.z - p0.z))
    if line.length < 1e-6:
        return shapes
    line.normalize()
    off = com - p0
    side = off - line * off.dot(line)
    side.y = 0
    if side.length < 1e-6:
        return shapes
    axis2 = line.copy()
    if axis2.cross(com - p0).y > 0:
        axis2 = -axis2
    near = {i for i, q in enumerate(pts) if q.y < 1e-4 and abs((q - p0).cross(line).length) < 2e-3}
    t = _first_contact(pts, p0, axis2, near)
    transform_all(shapes, _rot_about(axis2, t, p0))
    lo = min(p.y for s in shapes for p in s.v)
    transform_all(shapes, Matrix.Translation((0, -lo, 0)))
    return shapes


def shaped_slab(outline, w, h, t, nx, ny):
    """A thin slab (t thick along z) cut to the outline [(x, y), ...], its
    faces gridded so it can be bent afterwards."""
    return boolean(slab(w, h, t, nx, ny), prism(outline, t * 4), 'INTERSECT')


# ======================================================= safety glasses ===
# Worn frame: x across, y up, z out of the front; the lens's middle at the
# origin. The lens is one wraparound piece on a cylinder of radius SG_WRAP.
SG_WRAP = 0.074
SG_HALF = 0.064          # half the lens's length round the wrap
SG_T = 0.0022            # the lens's thickness
SG_HINGE_Y = 0.0065


def sg_outline():
    top = [(-0.064, 0.0125), (-0.061, 0.0165), (-0.052, 0.0185), (-0.040, 0.0202), (-0.024, 0.0208),
           (-0.010, 0.0200), (0.0, 0.0196)]
    bot = [(0.0, -0.0036), (0.0045, -0.0042), (0.0085, -0.0085), (0.0125, -0.0145), (0.0175, -0.0188),
           (0.0255, -0.0206), (0.0360, -0.0205), (0.0460, -0.0180), (0.0545, -0.0130), (0.0605, -0.0075),
           (0.064, -0.0020)]
    right_top = [(-x, y) for x, y in reversed(top[:-1])]
    left_bot = [(-x, y) for x, y in bot[1:]]
    loop = top + right_top + list(reversed(bot)) + left_bot
    return round_outline(loop, 0.0012, seg=2)


def sg_wrap(p):
    """(s, t, d) on the flat lens -> the wrapped lens: s round the wrap, t
    up, d through the lens (+ toward the front)."""
    s, t, d = p.x, p.y, p.z
    a = s / SG_WRAP
    r = SG_WRAP + d
    x = r * math.sin(a)
    z = r * math.cos(a) - SG_WRAP
    # a little curve top to bottom, and the lens's foot tilted in
    z -= t * t / (2 * 0.22)
    z += t * 0.10
    return Vector((x, t, z))


def sg_lens_worn():
    s = shaped_slab(sg_outline(), 0.136, 0.046, SG_T, 26, 4)
    return s.warp(sg_wrap)


def sg_end(side):
    """The lens's end (side = -1 left, +1 right) and its tangent backward."""
    p = sg_wrap(Vector((side * SG_HALF, SG_HINGE_Y, -SG_T / 2)))
    q = sg_wrap(Vector((side * (SG_HALF - 0.004), SG_HINGE_Y, -SG_T / 2)))
    back = (p - q).normalized()
    return p, back


def sg_temple_shape():
    """A temple in its own frame: from its hinge at the origin back along
    -z, x its thickness (2.8 mm), y up; the earpiece bent down."""
    out = [(0.0, -0.0070), (0.006, -0.0072), (0.016, -0.0052), (0.040, -0.0038)]
    out += [(0.090 + 0.003 * i, -0.0031) for i in range(8)]
    out += [(0.124, -0.0030), (0.1305, -0.0018), (0.132, 0.0006), (0.1305, 0.0026), (0.124, 0.0034)]
    out += [(0.111 - 0.003 * i, 0.0030) for i in range(8)]
    out += [(0.040, 0.0036), (0.016, 0.0050), (0.006, 0.0062), (0.0, 0.0060)]
    sh = prism(out, 0.0028, bevel_r=0.0007, bevel_seg=1)
    sh = sh.apply(Matrix.Rotation(math.pi / 2, 4, 'Y'))      # x -> -z, z -> x
    bend_at, bend = 0.094, math.radians(24)

    def earpiece(p):
        u = -p.z
        if u <= bend_at:
            return p
        k = min(1.0, (u - bend_at) / 0.012)
        a = bend * k * k * (3 - 2 * k) if k < 1.0 else bend
        # turn about the bend's line (x) at u = bend_at
        du = u - bend_at
        return Vector((p.x, p.y * math.cos(a) - du * math.sin(a), -(bend_at + du * math.cos(a) + p.y * math.sin(a))))
    sh.warp(earpiece)
    # the hinge block's knuckle and its screw head on the outside
    knuckle = cyl(0.0026, h=0.010, seg=10, bevel_r=0.0005, bevel_seg=1).move(0, 0, 0.0006)
    screw = cyl(0.0012, h=0.0008, seg=8, bevel_r=0.0002, bevel_seg=1).rot(0, 0, math.pi / 2).move(0.0018, 0, 0.0006)
    return sh + knuckle + screw


def sg_pads():
    """The soft nose piece moulded into the lens's notch: a rubber saddle
    following the notch's edge behind the lens, its ends flattened into
    pads."""
    notch = [(-0.0130, -0.0150), (-0.0105, -0.0118), (-0.0082, -0.0083), (-0.0058, -0.0055), (-0.0030, -0.0040),
             (0.0, -0.0035)]
    notch = notch + [(-x, y) for x, y in reversed(notch[:-1])]
    path = [sg_wrap(Vector((x, y + 0.0013, -SG_T / 2 - 0.0013))) for x, y in notch]
    saddle = tube(path, 0.0015, seg=8, smooth=1)
    pads = Shape()
    for side in (-1, 1):
        c = sg_wrap(Vector((side * 0.0118, -0.0122, -SG_T / 2 - 0.0022)))
        pad = ellipsoid(0.0030, 0.0052, 0.0017, seg=10).rot(0, side * 0.45, side * 0.55).move(*c)
        pads = pads + pad
    return saddle + pads


def sg_temples(folded):
    out = Shape()
    for side in (-1, 1):
        hinge, back = sg_end(side)
        t = sg_temple_shape()
        if folded:
            b = math.radians(5 if side < 0 else 11)
            a = side * (math.pi / 2 - b)
        else:
            a = -side * math.radians(7)
        t.apply(Matrix.Rotation(a, 4, 'Y'))
        t.move(*(hinge + back * 0.0025))
        out = out + t
    return out


def sg_pose(folded):
    lens = sg_lens_worn()
    frame = sg_temples(folded) + sg_pads()
    shapes = [lens, frame]
    if folded:
        # lying on its back: the front up
        transform_all(shapes, Matrix.Rotation(-math.pi / 2, 4, 'X'))
    settle(shapes)
    # centred on its footprint
    lo = Vector((min(p.x for s in shapes for p in s.v), 0, min(p.z for s in shapes for p in s.v)))
    hi = Vector((max(p.x for s in shapes for p in s.v), 0, max(p.z for s in shapes for p in s.v)))
    c = (lo + hi) / 2
    transform_all(shapes, Matrix.Translation((-c.x, 0, -c.z)))
    return shapes


_SG = {}


def sg(folded, k):
    if folded not in _SG:
        _SG[folded] = sg_pose(folded)
    return _SG[folded][k]


# ======================================================= reading glasses ===
# 50-20-135: a lens 50 mm wide, 30 deep, the bridge 20, the temples 135.
RG_LW, RG_LH, RG_BR = 0.050, 0.030, 0.020
RG_CX = RG_BR / 2 + RG_LW / 2


def rg_lens_outline(cx, grow=0.0, side=1):
    w, h = RG_LW / 2 + grow, RG_LH / 2 + grow
    # a soft rectangle: its outer bottom corner fullest, the top straighter
    corners = [((-w, -h), 0.0085), ((w, -h), 0.0105), ((w, h), 0.0060), ((-w, h), 0.0055)]
    pts = []
    n = len(corners)
    for i, (p, r) in enumerate(corners):
        pts += propkit._round_corner(p, corners[i - 1][0], corners[(i + 1) % n][0], r + grow * 0.8, 5)
    return [(cx + side * x, y) for x, y in pts]


def rg_front():
    rims = Shape()
    for side in (-1, 1):
        cx = side * RG_CX
        outer = rg_lens_outline(cx, 0.0034, side)
        inner = rg_lens_outline(cx, -0.0006, side)
        ring = bevel(cut(prism(outer, 0.0045), prism(inner, 0.02)), 0.0009, seg=2)
        rims = rims + ring
        # the endpiece where the hinge is let in
        end = rbox(0.0065, 0.0105, 0.0052, r=0.0014, seg=2).move(side * (RG_CX + RG_LW / 2 + 0.0030), RG_LH * 0.22, -0.0006)
        rims = rims + end
    # the keyhole bridge: an arch over the nose
    br = []
    for i in range(9):
        u = -1 + 2 * i / 8
        br.append((u * (RG_BR / 2 - 0.001), RG_LH * 0.18 + 0.004 * (1 - u * u), 0.0))
    bridge = tube(br, 0.0022, seg=8, smooth=1)
    pads = Shape()
    for side in (-1, 1):
        p = rbox(0.0022, 0.012, 0.0055, r=0.0009, seg=1).rot(0, side * 0.5, side * 0.35)
        p.move(side * (RG_BR / 2 + 0.0008), -0.002, -0.0035)
        pads = pads + p
    return rims + bridge + pads


def rg_temple_shape():
    out = [(0.0, -0.0026), (0.010, -0.0025), (0.060, -0.0020)]
    out += [(0.096 + 0.003 * i, -0.0019) for i in range(9)]
    out += [(0.128, -0.0018), (0.1345, -0.0010), (0.135, 0.0005), (0.1338, 0.0017), (0.128, 0.0020)]
    out += [(0.120 - 0.003 * i, 0.0021) for i in range(9)]
    out += [(0.060, 0.0022), (0.010, 0.0030), (0.0, 0.0030)]
    sh = prism(out, 0.0034, bevel_r=0.0007, bevel_seg=1)
    sh = sh.apply(Matrix.Rotation(math.pi / 2, 4, 'Y'))
    bend_at, bend = 0.100, math.radians(28)

    def earpiece(p):
        u = -p.z
        if u <= bend_at:
            return p
        k = min(1.0, (u - bend_at) / 0.014)
        a = bend * k * k * (3 - 2 * k) if k < 1.0 else bend
        du = u - bend_at
        return Vector((p.x, p.y * math.cos(a) - du * math.sin(a), -(bend_at + du * math.cos(a) + p.y * math.sin(a))))
    sh.warp(earpiece)
    return sh


def rg_lenses():
    out = Shape()
    for side in (-1, 1):
        cx = side * RG_CX
        ol = [(x - cx, y) for x, y in rg_lens_outline(cx, 0.0004, side)]
        s = shaped_slab(ol, RG_LW + 0.004, RG_LH + 0.004, 0.0018, 6, 4)

        def bulge(p):
            r2 = (p.x / (RG_LW / 2)) ** 2 + (p.y / (RG_LH / 2)) ** 2
            return Vector((p.x, p.y, p.z + (0.0009 * max(0.0, 1 - r2 * 0.6) if p.z > 0 else 0.0)))
        s.warp(bulge)
        out = out + s.move(cx, 0, 0)
    return out


def rg_pose():
    front = rg_front()
    lenses = rg_lenses()
    temples = Shape()
    for side in (-1, 1):
        t = rg_temple_shape()
        b = math.radians(4 if side < 0 else 10)
        t.apply(Matrix.Rotation(side * (math.pi / 2 - b), 4, 'Y'))
        t.move(side * (RG_CX + RG_LW / 2 + 0.0040), RG_LH * 0.22, -0.0040)
        temples = temples + t
    shapes = [front + temples, lenses]
    transform_all(shapes, Matrix.Rotation(-math.pi / 2, 4, 'X'))
    settle(shapes)
    lo = Vector((min(p.x for s in shapes for p in s.v), 0, min(p.z for s in shapes for p in s.v)))
    hi = Vector((max(p.x for s in shapes for p in s.v), 0, max(p.z for s in shapes for p in s.v)))
    c = (lo + hi) / 2
    transform_all(shapes, Matrix.Translation((-c.x, 0, -c.z)))
    return shapes


_RG = []


def rg(k):
    if not _RG:
        _RG.extend(rg_pose())
    return _RG[k]


# ============================================================ wristwatch ===
# Lying on its back, the dial up (y), 12 o'clock toward -z, 3 toward +x.
WW_R = 0.019            # the case, 38 mm
WW_LUG = 0.0215         # the spring bars, either side of the middle (z)
WW_DIAL = 0.0061        # the dial's face


def ww_case():
    prof = [(0.0, 0.0), (0.0138, 0.0), (0.0151, 0.0003), (0.0158, 0.0011), (0.0162, 0.0018), (0.0179, 0.0019),
            (0.0187, 0.0025), (0.0190, 0.0034), (0.0190, 0.0057), (0.0187, 0.0065), (0.0178, 0.0073),
            (0.0167, 0.0079), (0.0161, 0.0080), (0.0157, 0.0077), (0.0157, 0.0061), (0.0, 0.0059)]
    case = lathe(prof, seg=48, cap_bottom=False, cap_top=False)
    lugs = Shape()
    for sx in (-1, 1):
        for sz in (-1, 1):
            side = [(0.0150, 0.0022), (0.0222, 0.0024), (0.0248, 0.0030), (0.0250, 0.0042), (0.0236, 0.0053),
                    (0.0190, 0.0062), (0.0150, 0.0064)]
            lug = prism(side, 0.0026)                        # profile in (z, y), across x
            lug = bevel(lug, 0.0005, seg=1)
            lug.apply(Matrix.Rotation(-math.pi / 2, 4, 'Y'))  # its x (z of the watch) -> z
            lug.move(sx * (0.0100 + 0.0013), 0, 0)
            if sz < 0:
                lug.scale(1, 1, -1)
            lugs = lugs + lug
    bars = Shape()
    for sz in (-1, 1):
        bars = bars + cyl(0.0009, h=0.0202, seg=8, bevel_r=0).rot(0, 0, math.pi / 2).move(0, 0.0036, sz * WW_LUG)
    stem = cyl(0.0011, h=0.0030, seg=8, bevel_r=0).rot(0, 0, math.pi / 2).move(0.0198, 0.0042, 0)
    crown = lathe([(0.0, 0.0), (0.0024, 0.0), (0.0026, 0.0004), (0.0026, 0.0026), (0.0023, 0.0031), (0.0, 0.0032)], seg=18)
    crown = crown.rot(0, 0, -math.pi / 2).move(0.0208, 0.0042, 0)
    return case + lugs + bars + stem + crown


def ww_crystal():
    prof = [(0.0, 0.0069), (0.0158, 0.0069), (0.0158, 0.0078), (0.0151, 0.0085), (0.0128, 0.0090), (0.0085, 0.0093),
            (0.0, 0.0094)]
    return lathe(prof, seg=40, cap_bottom=False, cap_top=False)


def ww_hand(outline, t=0.00018):
    """A hand drawn in (x, along) with its pivot at the origin, pointing to
    12 (-z), lying flat, its back at y = 0."""
    s = prism(outline, t)
    s.apply(Matrix.Rotation(-math.pi / 2, 4, 'X'))      # along y -> -z, thickness z -> y
    return s.move(0, t / 2, 0)


def ww_hour():
    o = [(-0.0007, -0.0019), (0.0007, -0.0019), (0.0008, 0.0), (0.0008, 0.0068), (0.0, 0.0086), (-0.0008, 0.0068),
         (-0.0008, 0.0)]
    collet = cyl(0.0017, h=0.00022, seg=16, bevel_r=0).move(0, 0.00011, 0)
    return ww_hand(o) + collet


def ww_minute():
    o = [(-0.0006, -0.0024), (0.0006, -0.0024), (0.0006, 0.0), (0.00055, 0.0112), (0.0, 0.0129), (-0.00055, 0.0112),
         (-0.0006, 0.0)]
    collet = cyl(0.0014, h=0.00022, seg=16, bevel_r=0).move(0, 0.00011, 0)
    return ww_hand(o) + collet


def ww_second():
    o = [(-0.00022, -0.0042), (0.00022, -0.0042), (0.00016, 0.0142), (-0.00016, 0.0142)]
    weight = cyl(0.0008, h=0.00014, seg=12, bevel_r=0).move(0, 0.00007, 0.0031)
    hub = cyl(0.0009, h=0.00026, seg=12, bevel_r=0).move(0, 0.00013, 0)
    return ww_hand(o, 0.00012) + weight + hub


def _strap_long(z):
    """The long strap (6 o'clock, +z) at z: (centre x, bottom y, thickness, width)."""
    yb = 0.0026 * (1 - smoothstep(WW_LUG - 0.001, 0.036, z)) + 0.0042 * smoothstep(0.106, 0.1365, z) ** 2
    t = 0.0034 - 0.0010 * smoothstep(0.040, 0.095, z) - 0.0004 * smoothstep(0.110, 0.136, z)
    w = 0.0200 - 0.0020 * smoothstep(0.024, 0.085, z)
    k = (z - 0.1262) / 0.0105
    if k > 0:
        w *= math.sqrt(max(0.0, 1 - k * k)) * 0.85 + 0.15
    xc = 0.0032 * smoothstep(0.045, 0.1365, z) ** 1.5
    return xc, yb, t, w


def _strap_short(z):
    """The buckle's strap (12 o'clock, -z) at -z."""
    u = -z
    yb = 0.0026 * (1 - smoothstep(WW_LUG - 0.001, 0.036, u))
    t = 0.0034 - 0.0008 * smoothstep(0.040, 0.080, u)
    w = 0.0200 - 0.0020 * smoothstep(0.024, 0.085, u)
    xc = -0.0014 * smoothstep(0.045, 0.095, u)
    return xc, yb, t, w


def _strap_ring(xc, yb, t, w, z):
    ring = []
    for fx, fy in [(-0.5, 0.25), (-0.47, 0.0), (-0.22, 0.0), (0.0, 0.0), (0.22, 0.0), (0.47, 0.0), (0.5, 0.25),
                   (0.475, 0.78), (0.25, 0.98), (0.0, 1.0), (-0.25, 0.98), (-0.475, 0.78)]:
        x = fx * w
        top = 0.82 + 0.18 * (1 - (2 * min(abs(fx), 0.5)) ** 2)
        y = yb + (fy if fy < 0.5 else min(fy, top)) * t
        ring.append(Vector((xc + x, y, z)))
    return ring


def _strap_sweep(fn, zs):
    v, f = [], []
    n = 12
    for z in zs:
        v += _strap_ring(*fn(z), z)
    for j in range(len(zs) - 1):
        for i in range(n):
            a, b = j * n + i, j * n + (i + 1) % n
            f.append((a, b, b + n, a + n))
    s = Shape(v, f)
    for ring, z in ((range(n), zs[0]), (range((len(zs) - 1) * n, len(zs) * n), zs[-1])):
        ids = list(ring)
        c = sum((v[i] for i in ids), Vector((0, 0, 0))) / n
        s.v.append(c)
        ci = len(s.v) - 1
        for a, b in zip(ids, ids[1:] + ids[:1]):
            s.f.append((a, b, ci))
    return propkit._fix_winding(s)


def _samples(spans):
    out = []
    for a, b, step in spans:
        n = max(1, round(abs(b - a) / step))
        for i in range(n):
            out.append(a + (b - a) * i / n)
    out.append(spans[-1][1])
    return out


def ww_strap():
    zl = _samples([(WW_LUG - 0.0012, 0.040, 0.0022), (0.040, 0.104, 0.0064), (0.104, 0.1365, 0.0027)])
    long_ = _strap_sweep(_strap_long, zl)
    holes = Shape()
    for i in range(7):
        z = 0.0785 + 0.006 * i
        xc, yb, t, w = _strap_long(z)
        holes = holes + cyl(0.0011, h=0.012, seg=10, bevel_r=0).move(xc, yb, z)
    long_ = cut(long_, holes)
    zs = _samples([(-(WW_LUG - 0.0012), -0.040, 0.0022), (-0.040, -0.0955, 0.0064)])
    short = _strap_sweep(_strap_short, zs)
    keepers = Shape()
    for zk in (-0.0835, -0.0745):
        xc, yb, t, w = _strap_short(zk)
        # the loop round the strap, its underside flat on the floor
        outer = [(x, y + 0.00045) for x, y in rounded_rect(w + 0.0024, t + 0.0009, 0.0010, seg=2)]
        inner = rounded_rect(w + 0.0002, t + 0.0001, 0.0006, seg=2)
        k = cut(prism(outer, 0.0050), prism(inner, 0.02))
        keepers = keepers + k.move(xc, yb + t / 2, zk)
    return long_ + short + keepers


def ww_buckle():
    """The tang buckle at the short strap's end, lying flat, and its tongue."""
    z0 = -0.0960
    outline = rounded_rect(0.0215, 0.0160, 0.0050, seg=4)
    path = [(x, 0.0011, z0 - 0.0080 - y) for x, y in outline] + [(outline[0][0], 0.0011, z0 - 0.0080 - outline[0][1])]
    frame = tube(path, 0.0011, seg=8, caps=False)
    tongue = tube([(0.0, 0.0014, z0 + 0.0002), (0.0, 0.0018, z0 - 0.0060), (0.0, 0.0024, z0 - 0.0145),
                   (0.0, 0.0014, z0 - 0.0172)], 0.0008, seg=8, smooth=1)
    return frame + tongue


# ================================================================== keys ===
def _key_profile(x0, x1, top, cuts, flat=0.3, step=0.25):
    """The bitted edge from x0 to x1 (mm): `top` cut by V cuts (x, depth)."""
    pts = []
    n = int((x1 - x0) / step)
    for i in range(n + 1):
        x = x0 + (x1 - x0) * i / n
        y = top
        for cx, d in cuts:
            y = min(y, top - d + max(0.0, abs(x - cx) - flat))
        pts.append((x, y))
    # drop the points that sit on a straight run
    out = [pts[0]]
    for a, b, c in zip(pts, pts[1:], pts[2:]):
        if abs((b[1] - a[1]) * (c[0] - b[0]) - (c[1] - b[1]) * (b[0] - a[0])) > 1e-6:
            out.append(b)
    out.append(pts[-1])
    return out


def _bow(cx, a, b, e, a0, a1, n):
    """A superellipse bow centred at (cx, 0), half-sizes a (along), b
    (across), from angle a0 to a1 (degrees)."""
    out = []
    for i in range(n + 1):
        t = math.radians(a0 + (a1 - a0) * i / n)
        c, s_ = math.cos(t), math.sin(t)
        out.append((cx + a * math.copysign(abs(c) ** (2 / e), c), b * math.copysign(abs(s_) ** (2 / e), s_)))
    return out


KEYS = {
    # kind: bow (cx, a, b, e), hole r, neck x, shoulder x, blade half-width, tip x, cuts [(x, depth)], thickness
    'house': dict(bow=(2.4, 8.0, 12.2, 2.7), hole=2.3, neck=(10.0, 6.4), shoulder=13.6, half=4.25, tip=48.4,
                  cuts=[(20.0, 1.6), (23.8, 3.3), (27.6, 2.2), (31.4, 3.8), (35.2, 1.1)], t=2.0, stop=True),
    'house2': dict(bow=(2.4, 8.0, 12.2, 2.7), hole=2.3, neck=(10.0, 6.4), shoulder=13.6, half=4.25, tip=48.4,
                   cuts=[(20.0, 2.8), (23.8, 1.1), (27.6, 3.9), (31.4, 2.2), (35.2, 2.8)], t=2.0, stop=True),
    'schlage': dict(bow=(3.6, 10.2, 13.5, 2.2), hole=2.0, neck=(12.6, 6.0), shoulder=15.4, half=4.3, tip=51.0,
                    cuts=[(21.3, 2.1), (25.3, 3.4), (29.3, 1.3), (33.2, 2.9), (37.2, 1.9)], t=2.1, stop=True),
    'padlock': dict(bow=(4.2, 9.6, 9.6, 2.0), hole=3.0, neck=(12.2, 4.6), shoulder=13.6, half=3.5, tip=44.0,
                    cuts=[(19.5, 1.4), (23.5, 2.6), (27.5, 1.0), (31.5, 2.2)], t=2.3, stop=False),
    'cabinet': dict(bow=(2.2, 6.4, 8.6, 2.4), hole=1.7, neck=(8.0, 4.4), shoulder=9.6, half=3.0, tip=38.0,
                    cuts=[(14.5, 1.0), (18.5, 1.6), (22.5, 0.8), (26.5, 1.4)], t=1.6, stop=False),
}


def key_flat(kind):
    """A key lying flat: its hole at the origin, its blade along +x, its
    bitted edge toward -z, its faces at y = 0 and y = t."""
    k = KEYS[kind]
    cx, a, b, e = k['bow']
    h = k['half']
    nx, nw = k['neck']
    sh, tip = k['shoulder'], k['tip']
    cuts = k['cuts']
    # bottom edge (y -h) from the shoulder to the tip, the tip, the bitted top
    pts = [(nx, -nw)]
    pts += [(sh - 0.6, -h - 0.2), (sh, -h)]
    pts += [(tip - 2.6, -h), (tip - 0.6, -h * 0.55), (tip, -h * 0.05)]
    last = max(c[0] for c in cuts) + 4.0
    top = _key_profile(sh + 0.5, last, h, cuts)
    pts += [(tip - 1.0, h * 0.35), (last + 1.2, h - 0.6)]
    pts += list(reversed(top))
    if k['stop']:
        pts += [(sh + 0.3, h), (sh, h + 1.5), (sh - 1.4, h + 1.6)]
    else:
        pts += [(sh, h), (sh - 0.6, h + 0.2)]
    pts += [(nx, nw)]
    # the bow, round from the neck's top over its far end to the neck's bottom
    bow = _bow(cx, a, b, e, 50, 310, 26)
    pts += bow
    pts = [(x * 0.001, y * 0.001) for x, y in pts]
    t = k['t'] * 0.001
    s = prism(pts, t)
    s = cut(s, cyl(k['hole'] * 0.001, h=t * 4, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0))
    # the keyway milled along the blade, both faces
    grooves = Shape()
    for side, yy, gw, gd in [(1, 1.1, 1.2, 0.5), (-1, -1.6, 1.0, 0.45), (-1, 2.4, 0.8, 0.35)]:
        if yy + gw / 2 > h - 0.6:
            continue
        L = tip - sh + 2.0
        g = box(L * 0.001, gw * 0.001, gd * 0.002).move((sh + 1.0 + L / 2) * 0.001, yy * 0.001, side * t / 2)
        grooves = grooves + g
    if grooves.v:
        s = cut(s, grooves)
    s = bevel(s, 0.00018, seg=1, angle=40)
    s.apply(Matrix.Rotation(-math.pi / 2, 4, 'X'))    # outline y -> -z, thickness -> y
    return s.move(0, t / 2, 0), tip * 0.001


# where the bunch lies: (kind, angle on the ring (deg), its heading (deg), its lift (mm))
KEY_SLOTS = [('house', -100, -122, 0.0), ('schlage', -86, -92, 2.2), ('padlock', -70, -60, 4.3),
             ('house2', -94, -108, 6.3), ('cabinet', -78, -76, 8.2)]
KEY_RING_R = 0.0113


def _ring_y(theta):
    """The split ring's height (its middle) at angle theta: leaning on the
    bows at the bunch (-88 deg), on the floor across from it."""
    return 0.0042 + 0.0034 * math.cos(math.radians(theta + 88))


def key_posed(i):
    kind, th, head, lift = KEY_SLOTS[i]
    s, L = key_flat(kind)
    t = KEYS[kind]['t'] * 0.001
    hole_y = lift * 0.001
    beta = math.asin(min(0.3, hole_y / L))
    # lift the bow end, the tip on the floor
    s.apply(_rot_about(Vector((0, 0, 1)), -beta, Vector((L, 0, 0))))
    s.apply(Matrix.Rotation(-math.radians(head), 4, 'Y'))
    hx, hz = KEY_RING_R * math.cos(math.radians(th)), KEY_RING_R * math.sin(math.radians(th))
    return s.move(hx, 0, hz)


def key_ring():
    """The split ring (two turns of flat wire, 25 mm), leaning; the tag's
    rim, its eyelet and the jump ring."""
    v, f = [], []
    turns, seg = 2.0, 36
    n = int(turns * seg)
    ww, wt = 0.0024, 0.00075
    for i in range(n + 1):
        u = TAU * turns * i / n
        ax = -0.0008 + 0.0016 * i / n           # the two turns stacked
        taper = min(1.0, i / 4, (n - i) / 4) if i not in (0, n) else 0.35
        r0, r1 = KEY_RING_R - ww / 2 * taper, KEY_RING_R + ww / 2 * taper
        c, s_ = math.cos(u), math.sin(u)
        for rr, dy in ((r0, -wt / 2), (r1, -wt / 2), (r1, wt / 2), (r0, wt / 2)):
            v.append(Vector((rr * c, ax + dy, rr * s_)))
    for i in range(n):
        for j in range(4):
            a, b = i * 4 + j, i * 4 + (j + 1) % 4
            f.append((a, b, b + 4, a + 4))
    f.append((3, 2, 1, 0))
    f.append(tuple(n * 4 + j for j in range(4)))
    ring = propkit._fix_winding(Shape(v, f))
    # lean it: up at the bunch (-88 deg), down across from it

    def lean(p):
        th = math.degrees(math.atan2(p.z, p.x))
        return Vector((p.x, p.y + _ring_y(th), p.z))
    ring.warp(lean)
    # the tag on its jump ring, across from the bunch
    tdir = Vector((math.cos(math.radians(70)), 0, math.sin(math.radians(70))))
    tc = tdir * 0.0330
    rim = lathe([(0.0136, 0.0002), (0.0158, 0.0), (0.0162, 0.0006), (0.0158, 0.0013), (0.0136, 0.0011), (0.0134, 0.0006),
                (0.0136, 0.0002)], seg=40, cap_bottom=False, cap_top=False)
    rim.move(tc.x, 0, tc.z)
    eye_c = tc - tdir * 0.0128
    eyelet = torus(0.0019, 0.0006, seg=12, tseg=6).rot(math.pi / 2, 0, 0).move(eye_c.x, 0.0012, eye_c.z)
    jc = tdir * (KEY_RING_R + 0.0034)
    jump = torus(0.0040, 0.00055, seg=20, tseg=6).rot(math.pi / 2, 0, 0)
    jump.rot(0, 0, 0.35)
    jump.rot(0, -math.radians(70) + math.pi / 2, 0)
    jump.move(jc.x, 0.0013, jc.z)
    return ring + rim + eyelet + jump


def key_tag_paper():
    tdir = Vector((math.cos(math.radians(70)), 0, math.sin(math.radians(70))))
    tc = tdir * 0.0330
    d = cyl(0.0137, h=0.0003, seg=40, bevel_r=0).move(tc.x, 0.00075, tc.z)
    return cut(d, cyl(0.0019, h=0.004, seg=12, bevel_r=0).move((tc - tdir * 0.0128).x, 0, (tc - tdir * 0.0128).z))


_KEYS = {}


def keys(i):
    if not _KEYS:
        parts = [key_posed(j) for j in range(5)] + [key_ring(), key_tag_paper()]
        lo = min(p.y for s in parts for p in s.v)
        for s in parts:
            s.move(0, -lo, 0)
        _KEYS.update(enumerate(parts))
    return _KEYS[i]


# ================================================================ ribbon ===
RB_W = 0.0048           # 3/16 in curling ribbon


def ribbon_strip(pts, wdirs, width=RB_W):
    """A flat strip along pts, `width` across along wdirs, its middle a
    shade proud (the crimp), seen from both sides."""
    v, f = [], []
    n = len(pts)
    for i in range(n):
        t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
        w = wdirs[i] - t * wdirs[i].dot(t)
        w = w.normalized() if w.length > 1e-9 else Vector((1, 0, 0))
        nrm = t.cross(w).normalized()
        if nrm.y < 0:
            nrm = -nrm
        v += [pts[i] - w * width / 2, pts[i] + nrm * 0.00012, pts[i] + w * width / 2]
    for i in range(n - 1):
        a = i * 3
        f.append((a, a + 3, a + 4, a + 1))
        f.append((a + 1, a + 4, a + 5, a + 2))
    return Shape(v, f)


def ribbon_tail(start, heading, flat_len, turns, r0, r1, pitch, crush, wander=0.0):
    """A tail: lying flat from `start` along `heading` (deg) for flat_len,
    then rolled up in a ringlet round the ribbon's width, `turns` turns from
    radius r0 to r1, advancing `pitch` a turn, squashed to `crush`."""
    A = Vector((math.cos(math.radians(heading)), 0, math.sin(math.radians(heading))))
    B = Vector((-A.z, 0, A.x))
    up = Vector((0, 1, 0))
    pts, ws = [], []
    nflat = max(2, int(flat_len / 0.004))
    for i in range(nflat + 1):
        u = i / nflat
        p = start + A * flat_len * u + B * wander * math.sin(u * math.pi)
        p.y = max(0.0003, start.y * (1 - smoothstep(0.0, 0.5, u)) + 0.0003)
        pts.append(p)
        ws.append(B.copy())
    end = pts[-1].copy()
    end.y = 0.0
    nh = max(1, int(turns * 18))
    for i in range(1, nh + 1):
        u = TAU * turns * i / nh
        r = r0 + (r1 - r0) * i / nh
        c = end + B * (pitch * u / TAU) + up * r * crush
        p = c + A * (r * math.sin(u)) - up * (r * crush * math.cos(u))
        p.y = max(p.y, 0.0002)
        pts.append(p)
        ws.append(B.copy())
    return ribbon_strip(pts, ws)


def ribbon_knot():
    """Ribbon cut from round a parcel: the knot (two loops pulled tight,
    crushed), and four tails, curled in ringlets or trodden flat."""
    o = Vector((0, 0.0012, 0))
    knot = ribbon_tail(Vector((-0.004, 0.0015, -0.002)), 20, 0.004, 1.3, 0.0026, 0.0022, 0.0030, 0.55)
    knot = knot + ribbon_tail(Vector((0.002, 0.0015, 0.003)), 120, 0.003, 1.1, 0.0024, 0.0022, 0.0022, 0.5)
    tails = [
        ribbon_tail(o.copy(), 15, 0.030, 2.6, 0.0055, 0.0034, 0.0075, 0.85, 0.004),
        ribbon_tail(o.copy(), 95, 0.012, 3.4, 0.0048, 0.0030, 0.0052, 0.95, -0.002),
        ribbon_tail(o.copy(), 205, 0.046, 1.4, 0.0060, 0.0050, 0.0120, 0.45, 0.006),
        ribbon_tail(o.copy(), 290, 0.022, 2.1, 0.0050, 0.0036, 0.0095, 0.7, -0.003),
    ]
    return on_floor(sum(tails[1:], tails[0]) + knot)


def ribbon_length():
    """A single cut length lying in a loose S, trodden flat in the middle,
    a twist where it turned over, each end rolled in a ringlet."""
    pts, ws = [], []
    n = 30
    for i in range(n + 1):
        u = i / n
        x = -0.055 + 0.110 * u
        z = 0.016 * math.sin(u * math.pi * 1.7) - 0.006 * u
        pts.append(Vector((x, 0.0003, z)))
    for i in range(n + 1):
        t = (pts[min(i + 1, n)] - pts[max(i - 1, 0)]).normalized()
        side = Vector((-t.z, 0, t.x))
        # a half twist two thirds along: the ribbon stood on its edge there
        k = smoothstep(0.58, 0.70, i / n) - smoothstep(0.70, 0.80, i / n)
        w = side * math.cos(k * math.pi / 2) + Vector((0, 1, 0)) * math.sin(k * math.pi / 2)
        if k > 0.05:
            pts[i].y = 0.0003 + RB_W / 2 * math.sin(k * math.pi / 2)
        ws.append(w)
    mid = ribbon_strip(pts, ws)
    end = pts[-1] - pts[-2]
    a = ribbon_tail(pts[-1].copy(), math.degrees(math.atan2(end.z, end.x)), 0.002, 3.2, 0.0050, 0.0030, 0.0060, 0.9)
    beg = pts[0] - pts[1]
    b = ribbon_tail(pts[0].copy(), math.degrees(math.atan2(beg.z, beg.x)), 0.004, 2.6, 0.0056, 0.0036, 0.0080, 0.7)
    return on_floor(mid + a + b)


# =========================================================== alarm clock ===
# Standing, its face to +z; the case's middle at (0, AC_Y, 0).
AC_R = 0.0522
AC_Y = 0.0600
AC_DIAL = 0.0145


def _drum(prof, seg=48):
    s = lathe(prof, seg=seg, cap_bottom=False, cap_top=False)
    s.apply(Matrix.Rotation(math.pi / 2, 4, 'X'))     # lathe y -> z
    return s


def ac_case():
    prof = [(0.0, -0.0236), (0.0200, -0.0233), (0.0300, -0.0228), (0.0345, -0.0226), (0.0355, -0.0232), (0.0365, -0.0226),
            (0.0440, -0.0218), (0.0490, -0.0205), (0.0512, -0.0188), (0.0520, -0.0165), (AC_R, -0.0100),
            (AC_R, 0.0120), (0.0519, 0.0150), (0.0505, 0.0160), (0.0480, 0.0155), (0.0470, 0.0142), (0.0, 0.0140)]
    return _drum(prof).move(0, AC_Y, 0)


def ac_glass():
    R, sag, z0 = 0.0466, 0.0045, 0.0190
    rc = (R * R + sag * sag) / (2 * sag)
    prof = []
    for i in range(8):
        r = R * (1 - i / 8)
        prof.append((r, z0 + math.sqrt(rc * rc - r * r) - (rc - sag)))
    prof.append((0.0, z0 + sag))
    return _drum(prof, 40).move(0, AC_Y, 0)


def _radial(deg, r):
    a = math.radians(deg)
    return Vector((math.sin(a) * r, AC_Y + math.cos(a) * r, 0))


def ac_chrome():
    bezel = _drum([(0.0461, 0.0136), (0.0466, 0.0170), (0.0472, 0.0198), (0.0486, 0.0212), (0.0505, 0.0214),
                   (0.0524, 0.0203), (0.0533, 0.0181), (0.0531, 0.0158), (0.0522, 0.0146), (0.0505, 0.0140)]).move(0, AC_Y, 0)
    parts = [bezel]
    bz = -0.0030
    crowns = []
    for side in (-1, 1):
        deg = side * 40
        # the bell: a dome of spun steel, its mouth to the case
        prof = [(0.0, 0.0), (0.0075, 0.0004), (0.0150, 0.0023), (0.0215, 0.0065), (0.0256, 0.0125), (0.0278, 0.0185),
                (0.0285, 0.0225)]
        bell = vessel(prof, 0.0011, seg=40)
        bell.rot(math.pi, 0, 0)                              # mouth down
        bell.move(0, 0.0225, 0)                              # mouth at y = 0, crown at 0.0225
        nut = cyl(0.0034, h=0.0030, seg=6, bevel_r=0.0004, bevel_seg=1).move(0, 0.0240, 0)
        post = cyl(0.0021, h=0.034, seg=10, bevel_r=0).move(0, 0.0090, 0)
        b = bell + nut + post
        b.move(0, AC_R + 0.0036, 0)
        b.rot(0, 0, -math.radians(deg))
        b.move(0, AC_Y, bz)
        parts.append(b)
        crowns.append(_radial(deg, AC_R + 0.0036 + 0.0255) + Vector((0, 0, bz)))
    # the hammer between them, on its stem out of the case's top
    stem = cyl(0.0012, h=0.014, seg=8, bevel_r=0).move(0, AC_Y + AC_R + 0.006, bz)
    head = capsule(0.0036, 0.011, seg=12, cap_rings=3).rot(0, 0, math.pi / 2).move(0, AC_Y + AC_R + 0.0135, bz)
    parts += [stem, head]
    # the carrying handle from crown to crown
    l, r = crowns
    mid = Vector((0, AC_Y + AC_R + 0.0440, bz))
    path = [l, l.lerp(mid, 0.45) + Vector((0, 0.010, 0)), mid, r.lerp(mid, 0.45) + Vector((0, 0.010, 0)), r]
    parts.append(tube(path, 0.0017, seg=10, smooth=2))
    # the feet: splayed legs, a pad under each
    for side in (-1, 1):
        deg = 180 - side * 32
        a = _radial(deg, AC_R - 0.003)
        b_ = _radial(deg, AC_R + 0.0085)
        leg = tube([a, b_], 0.0025, seg=10)
        pad = cyl(0.0060, 0.0066, h=0.0030, seg=16, bevel_r=0.0008, bevel_seg=1).move(b_.x, 0.0015, b_.z)
        neck = tube([b_, Vector((b_.x, 0.003, b_.z))], 0.0025, seg=10)
        parts += [leg, pad, neck]
    # at the back: the two wind keys, the time-set knob
    for kx, ky in ((-0.017, 0.011), (0.017, 0.011)):
        stem2 = cyl(0.0022, h=0.0060, seg=10, bevel_r=0).rot(math.pi / 2, 0, 0).move(kx, AC_Y + ky, -0.0266)
        wing = [(-0.0105, 0.0), (-0.0098, 0.0058), (-0.0055, 0.0080), (-0.0012, 0.0050), (0.0, 0.0018), (0.0012, 0.0050),
                (0.0055, 0.0080), (0.0098, 0.0058), (0.0105, 0.0), (0.0, -0.0010)]
        wing = prism(wing, 0.0012, bevel_r=0.0003, bevel_seg=1)
        # in the plane through its stem: its span up and down (y), out
        # from the stem's end (-z), its thickness across (x)
        wing.apply(Matrix(((0, 0, 1, 0), (1, 0, 0, 0), (0, -1, 0, 0), (0, 0, 0, 1))))
        wing.move(kx, AC_Y + ky, -0.0290)
        parts += [stem2, wing]
    knob = cyl(0.0058, h=0.0042, seg=20, bevel_r=0.0006, bevel_seg=1).rot(math.pi / 2, 0, 0).move(0, AC_Y - 0.024, -0.0256)
    parts.append(knob)
    # the hub over the hands
    hub = lathe([(0.0, 0.0), (0.0026, 0.0), (0.0026, 0.0010), (0.0016, 0.0022), (0.0, 0.0024)], seg=16)
    hub.rot(math.pi / 2, 0, 0)
    hub.move(0, AC_Y, AC_DIAL + 0.0028)
    parts.append(hub)
    return sum(parts[1:], parts[0])


def ac_hand(outline, t=0.0004):
    """A hand in the dial's plane, pointing to 12 (+y), its back at z = 0."""
    return prism(outline, t).move(0, 0, t / 2)


def ac_hour():
    o = [(-0.0016, -0.0060), (0.0016, -0.0060), (0.0012, 0.0), (0.0012, 0.0150), (0.0036, 0.0175), (0.0030, 0.0205),
         (0.0, 0.0255), (-0.0030, 0.0205), (-0.0036, 0.0175), (-0.0012, 0.0150), (-0.0012, 0.0)]
    collet = cyl(0.0030, h=0.0004, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0002)
    return ac_hand(o) + collet


def ac_minute():
    o = [(-0.0013, -0.0075), (0.0013, -0.0075), (0.0010, 0.0), (0.0010, 0.0250), (0.0028, 0.0275), (0.0022, 0.0305),
         (0.0, 0.0370), (-0.0022, 0.0305), (-0.0028, 0.0275), (-0.0010, 0.0250), (-0.0010, 0.0)]
    collet = cyl(0.0026, h=0.0004, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.0002)
    return ac_hand(o) + collet


def ac_set():
    o = [(-0.0004, -0.0030), (0.0004, -0.0030), (0.0004, 0.0175), (0.0020, 0.0175), (0.0, 0.0215), (-0.0020, 0.0175),
         (-0.0004, 0.0175)]
    collet = cyl(0.0034, h=0.0003, seg=16, bevel_r=0).rot(math.pi / 2, 0, 0).move(0, 0, 0.00015)
    return ac_hand(o, 0.0003) + collet


# ================================================================ parts ===
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


auto('Real__safetyGlassesOpenLens', lambda: sg(False, 0), 1000)
auto('Real__safetyGlassesOpenFrame', lambda: sg(False, 1), 1600)
auto('Real__safetyGlassesFoldedLens', lambda: sg(True, 0), 1000)
auto('Real__safetyGlassesFoldedFrame', lambda: sg(True, 1), 1600)
auto('Real__readersFrame', lambda: rg(0), 2300)
auto('Real__readersLenses', lambda: rg(1), 500)
auto('Real__watchCase', lambda: ww_case() + ww_buckle(), 2400)
auto('Real__watchCrystal', ww_crystal, 420)
auto('Real__watchStrap', ww_strap, 1900)
auto('Real__watchHour', ww_hour, 120)
auto('Real__watchMinute', ww_minute, 120)
auto('Real__watchSecond', ww_second, 120)
for _i in range(5):
    auto(f'Real__keysKey{_i}', lambda i=_i: keys(i), 1200)
auto('Real__keysRing', lambda: keys(5), 1600)
auto('Real__keysTagPaper', lambda: keys(6), 300)
auto('Real__ribbonKnot', ribbon_knot, 1000)
auto('Real__ribbonLength', ribbon_length, 700)
auto('Real__alarmCase', ac_case, 1500)
auto('Real__alarmChrome', ac_chrome, 4600)
auto('Real__alarmGlass', ac_glass, 640)
auto('Real__alarmHour', ac_hour, 120)
auto('Real__alarmMinute', ac_minute, 120)
auto('Real__alarmSet', ac_set, 120)
