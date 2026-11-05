"""t20: a study's rare things (Props P3, docs/PROPS.md): a fireside bellows,
a floor globe on its stand, an orrery, a chess table with its board and
pieces, a Chinese abacus, a humidor and its cigars, a hunter-case pocket
watch on its chain and a child's hand-crank music box. Sources and sizes:
t20.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is modelled at
its real size in its prop's own frame: x across its front, y up, z out of
its front; a floor thing's floor is the plane y = 0. Every part is named
`Real__<prop><Part>`:

bellows*     an English oak and leather fireside bellows lying on its back,
             its nozzle to -x, its handles to +x: Boards (the two boards,
             their handles and the nose block between them), Leather (the
             gusset round the boards, pleated, nailed to their edges), Tacks
             (its brass nails), Nozzle (turned brass, along -x)
globe*       a 12 in terrestrial globe on a floor stand (y = 0 the floor; the
             ball's middle at GL_CY): Ball (the sphere, r 152.5 mm, about the
             origin, a `sphere` frame so its map wraps it), Meridian (the
             brass ring round it in the xy plane, about the origin), Pins
             (its axis pins and the hour circle at the north pole, the axis
             along +y, about the origin), Tripod (three turned legs, their
             stretchers, the column and the meridian's socket) and Horizon
             (the wooden horizon ring on them) with HorizonPaper (its printed
             paper), or Pedestal (a turned column on a cross of four feet,
             the meridian's socket)
orrery*      a London planetary orrery of about 1820: Base (the turned drum
             on three feet), Dial (its printed paper), Pillar (the brass
             pillar and its sleeves), Sun (brass), Arm0..4 (Mercury to
             Jupiter's brass arms and posts, along +x from the pillar),
             Planet0..4 (ivory balls about their own middles, `sphere`
             frames), Moon (the Earth's moon on its arm, about the Earth's
             middle), Crank and CrankKnob (its handle)
chess*       a tripod table and a game left on it: TableTop (the moulded
             round top), TableBase (the turned column, the three cabriole
             legs), Board (a folding board, 300 mm), King, Queen, Bishop,
             Knight, Rook, Pawn (Staunton chessmen, a 64 mm king, their foot
             on y = 0), Ashtray (a cast brass cigar ashtray)
abacus*      a small Chinese abacus (suanpan) lying flat, 13 rods: Frame
             (its rails and beam), Rods (bamboo), Brass (its corner straps),
             BeadsA / BeadsB (its 91 beads: cleared, or a sum left on it)
humidor*     a Victorian cigar humidor, 260 x 190 mm: Box (the veneered
             case, its base moulding), Lining (the cedar inside), Lid (its
             veneered top, modelled shut with its hinge line on the x axis),
             LidLining, Brass (the hinges' and the escutcheon's), LidBrass
             (the lid's hinge leaves and its name plate), Cigar (a Corona,
             140 mm, along x about its middle), CigarBand, CigarAsh (a burnt
             foot's ash), Cutter (a steel guillotine cutter), Matchbook
watch*       a 16-size hunter-case pocket watch lying on its back, its
             pendant to +x: Case (the band, the domed back, the bezel, the
             pendant, the crown and the bow), Cover (the front lid, modelled
             shut, its hinge line along z at x = 0), Dial, Crystal, Hour,
             Minute, Second (blued hands, pointing to -z from their arbor at
             the origin), Link (one curb link of its Albert chain, along x),
             TBar, Key
musicBox*    a Swiss child's hand-crank music box, 100 x 70 mm: Case (the
             box), Lid (modelled shut, its hinge line on the x axis), Paper
             (the print inside the lid), Brass (the bedplate, the pinned
             cylinder, its wheel and the crank), Comb (steel, its pins),
             Glass (the pane over the works), Knob (porcelain)
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


def ring_lathe(prof, seg=48):
    """Revolve a closed loop of [(r, y)] (every r > 0) round +Y."""
    n = len(prof)
    v, f = [], []
    for i in range(seg):
        a = TAU * i / seg
        for (r, y) in prof:
            v.append((r * math.sin(a), y, r * math.cos(a)))
    for i in range(seg):
        i2 = (i + 1) % seg
        for k in range(n):
            k2 = (k + 1) % n
            f.append((i * n + k, i * n + k2, i2 * n + k2, i2 * n + k))
    return propkit._fix_winding(Shape(v, f))


def grid_shell(rows, close_rows=False):
    """Join rows of points (each the same count) into a sheet; `close_rows`
    joins each row's last point to its first."""
    n = len(rows[0])
    v, f = [], []
    for r in rows:
        v += [Vector(p) for p in r]
    m = n if close_rows else n - 1
    for i in range(len(rows) - 1):
        for k in range(m):
            k2 = (k + 1) % n
            f.append((i * n + k, i * n + k2, (i + 1) * n + k2, (i + 1) * n + k))
    return Shape(v, f)


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


def sweep(path, radius, seg=8):
    """A tube along `path` [(x, y, z)], its radius `radius(t)` (t 0..1 along
    it), capped."""
    pts = [Vector(p) for p in path]
    n = len(pts)
    tang = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tang.append(t.normalized())
    up = Vector((0, 1, 0)) if abs(tang[0].y) < 0.9 else Vector((1, 0, 0))
    nrm = tang[0].cross(up).normalized()
    secs = []
    for i in range(n):
        if i:
            q = tang[i - 1].rotation_difference(tang[i])
            nrm = (q @ nrm).normalized()
        b = tang[i].cross(nrm).normalized()
        r = radius(i / (n - 1))
        secs.append([pts[i] + (nrm * math.cos(TAU * j / seg) + b * math.sin(TAU * j / seg)) * r for j in range(seg)])
    return loft(secs)


def dome_tack(r, h, seg=6):
    """A round-headed nail's head on y = 0, its dome up +y."""
    return lathe([(0, 0), (r, 0), (r * 0.82, h * 0.6), (r * 0.42, h * 0.95), (0, h)], seg=seg)


def outline_normals(pts):
    """Outward normals (2D) of a closed counter-clockwise outline."""
    out = []
    n = len(pts)
    for i in range(n):
        ax, az = pts[i - 1]
        bx, bz = pts[(i + 1) % n]
        tx, tz = bx - ax, bz - az
        l = math.hypot(tx, tz) or 1.0
        out.append((tz / l, -tx / l))
    return out


def ccw(pts):
    area = sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(pts, pts[1:] + pts[:1]))
    return pts if area > 0 else list(reversed(pts))


def xz_prism(outline, y0, y1, bevel_r=0.0, bevel_seg=1):
    """A slab of the outline [(x, z)] from y0 to y1."""
    s = prism([(x, -z) for x, z in outline], y1 - y0, bevel_r=bevel_r, bevel_seg=bevel_seg)
    # prism extrudes along its z: turn it so its depth is y (its (x, y) -> (x, -z))
    s.rot(x=-math.pi / 2)
    return s.move(0, (y0 + y1) / 2, 0)


# =========================================================== the bellows ===
# An English oak and leather fireside bellows of about 1880, 45 cm overall,
# 19 cm across its boards (t20.notes.md). It lies on its back board, the
# top board resting on the leather a little open at the back.
BW_NOSE, BW_BACK = -0.130, 0.120        # the body's nose and back (x)
BW_HANDLE = 0.236                       # the handles' ends
BW_HALF = 0.095                         # the body's half width
BW_T = 0.012                            # a board's thickness
BW_GAP_NOSE, BW_GAP_BACK = 0.014, 0.030  # between the boards at the nose and the back
BW_HINGE_X = -0.100                     # where the leather's hinge holds the top board


def bw_half(x):
    """The body's half width at x: narrow at its nose block, widest a third
    from its back, rounded off there."""
    u = (x - BW_NOSE) / (BW_BACK - BW_NOSE)
    if u < 0.62:
        k = smoothstep(0.0, 0.62, u)
        return 0.026 + (BW_HALF - 0.026) * math.sin(k * math.pi / 2) ** 1.3
    k = (u - 0.62) / 0.38
    return max(0.0, BW_HALF * math.sqrt(max(0.0, 1.0 - k ** 2.2)))


def bw_outline(with_handle=True, inset=0.0):
    """The board's outline [(x, z)], counter-clockwise: the pear of its body,
    its handle out to +x (narrowing to its grip, rounded at its end)."""
    n = 26
    xs = [BW_NOSE + (BW_BACK - BW_NOSE) * (i / n) ** 0.9 for i in range(n + 1)]
    side = [(x, -max(bw_half(x) - inset, 0.0)) for x in xs]
    pts = side[:]
    if with_handle:
        # the handle leaves the body's back: its half width 17 mm, a swell at its grip
        hx0 = BW_BACK - 0.028
        side = [(x, z) for x, z in side if x < hx0]
        hw = lambda x: 0.017 + 0.004 * smoothstep(0.17, 0.22, x) - 0.0035 * smoothstep(0.13, 0.16, x) * (1 - smoothstep(0.17, 0.21, x))
        hxs = [hx0 + (BW_HANDLE - 0.016 - hx0) * i / 8 for i in range(9)]
        lower = [(x, -max(bw_half(x), hw(x))) for x in hxs]
        end = [(BW_HANDLE - 0.016 + 0.016 * math.sin(a), -hw(BW_HANDLE) * math.cos(a)) for a in [math.pi * k / 8 for k in range(1, 8)]]
        pts = side + lower + end + [(x, -z) for x, z in reversed(lower)] + [(x, -z) for x, z in reversed(side)]
    else:
        pts = side + [(x, -z) for x, z in reversed(side[:-1])]
    # drop doubled points at the nose and the tail
    out = []
    for p in pts:
        if not out or math.hypot(p[0] - out[-1][0], p[1] - out[-1][1]) > 1e-4:
            out.append(p)
    return ccw(out)


def bw_top_y(x):
    """The top board's underside at x (it rests a little open, hinged at the nose)."""
    k = (BW_GAP_BACK - BW_GAP_NOSE) / (BW_BACK - BW_HINGE_X)
    return BW_T + BW_GAP_NOSE + max(0.0, x - BW_HINGE_X) * k


BW_TILT = math.atan((BW_GAP_BACK - BW_GAP_NOSE) / (BW_BACK - BW_HINGE_X))


def bellows_boards():
    o = bw_outline()
    bottom = xz_prism(o, 0.0, BW_T, bevel_r=0.0025, bevel_seg=2)
    top = xz_prism(o, 0.0, BW_T, bevel_r=0.0025, bevel_seg=2)
    # a carved roundel standing proud on the top board
    boss = cyl(0.032, h=0.003, seg=28, bevel_r=0.0012).move(0.035, BW_T + 0.0012, 0)
    ring = ring_lathe([(0.040, BW_T), (0.044, BW_T), (0.0425, BW_T + 0.0018), (0.0415, BW_T + 0.0018)], seg=32).move(0.035, 0, 0)
    top = top + boss + ring
    # the top board turned up about the hinge line
    top.move(-BW_HINGE_X, 0, 0).rot(z=BW_TILT).move(BW_HINGE_X, BW_T + BW_GAP_NOSE, 0)
    # the nose block between them, which the nozzle screws into
    nose = rbox(0.042, BW_GAP_NOSE + 0.004, 0.050, r=0.003, seg=1).move(BW_NOSE + 0.019, BW_T + BW_GAP_NOSE / 2, 0)
    return bottom + top + nose


def bellows_leather():
    """The gusset: a band of leather round the body from one side of the nose
    to the other, nailed along both boards' edges, its pleats sagging out."""
    side = [(x, -bw_half(x)) for x in [BW_HINGE_X - 0.01 + (BW_BACK - 0.004 - BW_HINGE_X + 0.01) * (i / 30) ** 0.8 for i in range(31)]]
    back = [(BW_BACK - 0.004 + 0.0, 0.0)]
    path = side + back + [(x, -z) for x, z in reversed(side)]
    # an even spacing along the band
    acc = [0.0]
    for a, b in zip(path, path[1:]):
        acc.append(acc[-1] + math.hypot(b[0] - a[0], b[1] - a[1]))
    total = acc[-1]
    m = 90
    samp = []
    j = 0
    for i in range(m + 1):
        s = total * i / m
        while j < len(acc) - 2 and acc[j + 1] < s:
            j += 1
        t = (s - acc[j]) / max(acc[j + 1] - acc[j], 1e-9)
        a, b = path[j], path[j + 1]
        samp.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, s / total))
    rows_out, rows_in = [], []
    nrows = 7
    for k in range(nrows + 1):
        t = k / nrows
        ro, ri = [], []
        for i, (x, z, u) in enumerate(samp):
            ax, az = samp[max(i - 1, 0)][:2]
            bx, bz = samp[min(i + 1, m)][:2]
            tx, tz = bx - ax, bz - az
            l = math.hypot(tx, tz) or 1.0
            nx, nz = tz / l, -tx / l
            if (nx * x + nz * z) < 0 and abs(z) > 0.01:
                nx, nz = -nx, -nz
            if abs(z) < 0.01 and x > 0:
                nx, nz = 1.0, 0.0
            y0 = 0.004
            y1 = bw_top_y(x) + 0.008
            y = y0 + (y1 - y0) * t
            # it sags out between its nailed edges; the pleats fold in and out
            gap = y1 - y0
            folds = 0.5 + 0.5 * math.cos(u * TAU * 11)
            near_nose = smoothstep(BW_HINGE_X - 0.01, BW_HINGE_X + 0.05, x)
            bulge = (0.0015 + (0.004 + 0.005 * folds) * math.sin(math.pi * t) * (gap / 0.045)) * near_nose
            o = 0.0012 + bulge
            ro.append((x + nx * o, y, z + nz * o))
            ri.append((x + nx * (o - 0.0016), y, z + nz * (o - 0.0016)))
        rows_out.append(ro)
        rows_in.append(ri)
    outer = grid_shell(rows_out)
    inner = grid_shell(rows_in)
    inner.f = [tuple(reversed(f)) for f in inner.f]
    s = outer + inner
    # close its top and bottom edges and its two ends
    n = m + 1
    off = len(outer.v)
    for k in (0, nrows):
        for i in range(m):
            a, b = k * n + i, k * n + i + 1
            s.f.append((a, b, off + b, off + a) if k == 0 else (b, a, off + a, off + b))
    for i in (0, m):
        for k in range(nrows):
            a, b = k * n + i, (k + 1) * n + i
            s.f.append((a, b, off + b, off + a) if i == m else (b, a, off + a, off + b))
    s = propkit._fix_winding(s)
    # the leather over the nose block, the hinge
    hinge = rbox(0.040, 0.0016, 0.054, r=0.0007, seg=1).move(BW_NOSE + 0.022, BW_T + BW_GAP_NOSE + 0.0008, 0)
    return s + hinge


def bellows_tacks():
    """Brass round-headed nails through the leather into both boards' edges,
    about 18 mm apart, and two along the hinge."""
    out = Shape()
    xs = [BW_HINGE_X + 0.004 + i * 0.0175 for i in range(13)]
    for side in (-1, 1):
        pts = [(x, side * bw_half(x)) for x in xs if x < BW_BACK - 0.012]
        pts.append((BW_BACK - 0.012, side * bw_half(BW_BACK - 0.012)))
        for (x, z) in pts:
            # the outward normal of the edge there
            dx = 0.002
            dz = (bw_half(x + dx) - bw_half(x - dx)) / (2 * dx)
            nx, nz = -dz, 1.0
            l = math.hypot(nx, nz)
            nx, nz = nx / l, nz / l * side
            for y in (BW_T * 0.5, bw_top_y(x) + BW_T * 0.5 * math.cos(BW_TILT)):
                t = dome_tack(0.0026, 0.0016).rot(z=-math.pi / 2).rot(y=-math.atan2(nz, nx))
                out = out + t.move(x + nx * 0.0024, y, z + nz * 0.0024)
    for z in (-0.016, 0.016):
        out = out + dome_tack(0.0026, 0.0016).move(BW_NOSE + 0.022, BW_T + BW_GAP_NOSE + 0.0016, z)
    return out


def bellows_nozzle():
    """Turned brass, screwed into the nose block: a flange, a collar with two
    beads, the long cone, its lip."""
    prof = [(0.0, 0.0), (0.0175, 0.0), (0.0178, 0.003), (0.0128, 0.0045), (0.0122, 0.010), (0.0135, 0.0125),
            (0.0122, 0.015), (0.0105, 0.018), (0.0098, 0.025), (0.0092, 0.028), (0.0058, 0.070), (0.0050, 0.080),
            (0.0058, 0.0825), (0.0052, 0.085), (0.0030, 0.0855), (0.0028, 0.083), (0.0, 0.083)]
    s = lathe(prof, seg=20)
    # along -x from the nose block's face
    s.rot(z=math.pi / 2)
    return s.move(BW_NOSE - 0.002, BW_T + BW_GAP_NOSE / 2, 0)


auto('Real__bellowsBoards', bellows_boards, 2600)
auto('Real__bellowsLeather', bellows_leather, 3400)
auto('Real__bellowsTacks', bellows_tacks, 2400)
auto('Real__bellowsNozzle', bellows_nozzle, 700)


# ============================================================ the abacus ===
# A small Chinese abacus (suanpan) of 13 rods, two beads above the beam and
# five below, 232 x 133 mm, lying flat on its back: hardwood rails mortised
# at the corners and strapped with brass, bamboo rods, wooden beads
# (t20.notes.md). Its rods run along z, the two-bead deck at -z.
AB_RODS, AB_PITCH = 13, 0.016
AB_RAIL = 0.012                          # the rails' width
AB_H = 0.022                             # the frame's height (lying)
AB_UP, AB_BEAM, AB_LOW = 0.030, 0.008, 0.071   # the decks along z and the beam between
AB_W = AB_RODS * AB_PITCH + 2 * AB_RAIL
AB_D = AB_UP + AB_BEAM + AB_LOW + 2 * AB_RAIL
AB_Z0 = -AB_D / 2 + AB_RAIL              # the inside's far end
AB_BEAD_R, AB_BEAD_H = 0.0068, 0.0084
AB_Y = AB_H / 2                          # the rods' height


def ab_rod_x(i):
    return (i - (AB_RODS - 1) / 2) * AB_PITCH


def abacus_frame():
    rails = [rbox(AB_W, AB_H, AB_RAIL, r=0.0022, seg=2).move(0, AB_H / 2, s * (AB_D / 2 - AB_RAIL / 2)) for s in (-1, 1)]
    stiles = [rbox(AB_RAIL, AB_H - 0.001, AB_D - 2 * AB_RAIL + 0.002, r=0.0018, seg=2).move(s * (AB_W / 2 - AB_RAIL / 2), AB_H / 2, 0) for s in (-1, 1)]
    beam_z = AB_Z0 + AB_UP + AB_BEAM / 2
    beam = rbox(AB_W - 2 * AB_RAIL + 0.002, AB_H * 0.62, AB_BEAM, r=0.0012, seg=2).move(0, AB_Y, beam_z)
    return sum(rails) + sum(stiles) + beam


def abacus_rods():
    out = Shape()
    for i in range(AB_RODS):
        r = cyl(0.0015, h=AB_D - AB_RAIL, seg=6, bevel_r=0).rot(x=math.pi / 2)
        # a bamboo rod's nodes are not seen; a slight bow in each
        r = r.warp(lambda p, k=i: Vector((p.x + 0.0003 * math.sin(p.z / AB_D * math.pi + k), p.y, p.z)))
        out = out + r.move(ab_rod_x(i), AB_Y, 0)
    return out


def abacus_brass():
    """Brass straps folded round each corner: over the top, down the two
    outer faces; a pin at each end."""
    out = Shape()
    for sx in (-1, 1):
        for sz in (-1, 1):
            x0, z0 = sx * AB_W / 2, sz * AB_D / 2
            top_a = box(0.024, 0.0007, 0.007).move(x0 - sx * 0.012, AB_H + 0.00035, z0 - sz * 0.0035)
            top_b = box(0.007, 0.0007, 0.024).move(x0 - sx * 0.0035, AB_H + 0.00035, z0 - sz * 0.012)
            side_a = box(0.024, 0.014, 0.0007).move(x0 - sx * 0.012, AB_H - 0.007, z0 + sz * 0.00035)
            side_b = box(0.0007, 0.014, 0.024).move(x0 + sx * 0.00035, AB_H - 0.007, z0 - sz * 0.012)
            pins = [dome_tack(0.0011, 0.0006).move(x0 - sx * 0.020, AB_H + 0.0007, z0 - sz * 0.0035),
                    dome_tack(0.0011, 0.0006).move(x0 - sx * 0.0035, AB_H + 0.0007, z0 - sz * 0.020)]
            out = out + bevel(top_a + top_b, 0.0002, seg=1) + side_a + side_b + sum(pins)
    return out


def ab_bead():
    s = lathe([(AB_BEAD_R * 0.38, -AB_BEAD_H / 2), (AB_BEAD_R, 0.0), (AB_BEAD_R * 0.38, AB_BEAD_H / 2)], seg=8)
    return s.rot(x=math.pi / 2)


def abacus_beads(digits):
    """The beads, each rod set to its digit: the upper bead down to the beam
    for a five, so many lower beads up to it."""
    bead = ab_bead()
    out = Shape()
    up_a, up_b = AB_Z0, AB_Z0 + AB_UP
    lo_a, lo_b = up_b + AB_BEAM, up_b + AB_BEAM + AB_LOW
    g = 0.0003
    for i, d in enumerate(digits):
        x = ab_rod_x(i)
        five, ones = d // 5, d % 5
        zs = []
        # the upper deck: its two beads; a five brings one down to the beam
        for k in range(2):
            if k < 2 - five:
                zs.append(up_a + AB_BEAD_H * (k + 0.5) + g * k)
            else:
                zs.append(up_b - AB_BEAD_H * (1 - k + 1 - 0.5) - g)
        # the lower deck: the ones pushed up to the beam, the rest down at the rail
        for k in range(5):
            if k < ones:
                zs.append(lo_a + AB_BEAD_H * (k + 0.5) + g * k)
            else:
                zs.append(lo_b - AB_BEAD_H * (4 - k + 0.5) - g * (4 - k))
        for z in zs:
            j = random.uniform(-0.00025, 0.00025)
            b = bead.copy().rot(z=random.uniform(0, TAU))
            out = out + b.move(x, AB_Y, z + j)
    return out


auto('Real__abacusFrame', abacus_frame, 900)
auto('Real__abacusRods', abacus_rods, 400)
auto('Real__abacusBrass', abacus_brass, 900)
auto('Real__abacusBeadsA', lambda: abacus_beads([0] * AB_RODS), 4400)
auto('Real__abacusBeadsB', lambda: abacus_beads([0, 0, 0, 0, 1, 7, 2, 0, 4, 9, 6, 0, 3]), 4400)


# =========================================================== the humidor ===
# A Victorian cigar humidor, 260 x 190 mm and 102 mm high shut: a case of
# pine veneered (burr walnut, rosewood, mahogany or ebonised), on a moulded
# plinth, lined with Spanish cedar standing proud as a lip the lid shuts
# over; two brass butt hinges, a brass escutcheon, a name plate let into the
# lid (t20.notes.md). The cigars are Coronas, 5 1/2 in by a 42 ring.
HU_W, HU_D = 0.260, 0.190
HU_HB, HU_HL = 0.072, 0.030             # the case's height, the lid's
HU_T = 0.009                            # the walls
HU_FLOOR = 0.008
HU_LIN = 0.003                          # the cedar lining
CG_L, CG_R = 0.140, 0.00825             # a Corona


def humidor_box():
    outer = rbox(HU_W, HU_HB, HU_D, r=0.002, seg=2).move(0, HU_HB / 2, 0)
    inner = box(HU_W - 2 * HU_T, HU_HB, HU_D - 2 * HU_T).move(0, HU_FLOOR + HU_HB / 2, 0)
    case = cut(outer, inner)
    # the plinth: a moulding round its foot, a cove over a fillet
    prof = [(0.0, 0.0), (0.006, 0.0), (0.006, 0.004), (0.0045, 0.0055), (0.0025, 0.0085), (0.0, 0.010)]
    plinth = Shape()
    rings = []
    hw, hd = HU_W / 2, HU_D / 2
    for (d, y) in prof:
        rings.append([(-hw - d, y, -hd - d), (hw + d, y, -hd - d), (hw + d, y, hd + d), (-hw - d, y, hd + d)])
    plinth = loft(rings, cap0=True, cap1=True)
    return case + plinth


def humidor_lining():
    h = HU_HB - HU_FLOOR + 0.006
    w, d = HU_W - 2 * HU_T, HU_D - 2 * HU_T
    outer = box(w, h, d).move(0, HU_FLOOR + h / 2, 0)
    inner = box(w - 2 * HU_LIN, h, d - 2 * HU_LIN).move(0, HU_FLOOR + 0.002 + h / 2, 0)
    s = cut(outer, inner)
    return bevel(s, 0.0006, seg=1)


def humidor_lid():
    """Shut, its hinge line on the x axis: it reaches +z to the front and up
    +y; its top cushioned over a moulded edge, hollow below."""
    outer = rbox(HU_W, HU_HL, HU_D, r=0.0045, seg=3).move(0, HU_HL / 2, HU_D / 2)
    inner = box(HU_W - 2 * HU_T, HU_HL - 0.008, HU_D - 2 * HU_T).move(0, (HU_HL - 0.008) / 2 - 0.001, HU_D / 2)
    s = cut(outer, inner)
    # a bead run round the top's edge, a step below it
    step = rbox(HU_W - 0.026, 0.0012, HU_D - 0.026, r=0.0006, seg=1).move(0, HU_HL + 0.0006, HU_D / 2)
    return s + step


def humidor_lid_lining():
    return rbox(HU_W - 2 * HU_T - 0.001, 0.0025, HU_D - 2 * HU_T - 0.001, r=0.0005, seg=1).move(0, HU_HL - 0.008 - 0.00125, HU_D / 2)


def hu_hinge_leaf(w=0.034, h=0.011):
    leaf = rbox(w, h, 0.0009, r=0.0003, seg=1)
    return leaf


def humidor_brass():
    """The hinges' case leaves and knuckles on its back, the escutcheon on its front."""
    out = Shape()
    for sx in (-1, 1):
        x = sx * 0.075
        out = out + hu_hinge_leaf().move(x, HU_HB - 0.0055, -HU_D / 2 - 0.00045)
        # the knuckles along the hinge line
        k = cyl(0.0021, h=0.034, seg=10, bevel_r=0.0003).rot(z=math.pi / 2).move(x, HU_HB, -HU_D / 2 - 0.0012)
        out = out + k
        for dx in (-0.011, 0.0, 0.011):
            out = out + dome_tack(0.0012, 0.0005).rot(x=-math.pi / 2).move(x + dx, HU_HB - 0.006, -HU_D / 2 - 0.0009)
    # the escutcheon: a brass plate cut to its keyhole, a bead round it
    plate = prism(round_outline([(-0.0075, -0.010), (0.0075, -0.010), (0.0075, 0.007), (0.0, 0.0115), (-0.0075, 0.007)], 0.002, seg=2), 0.0008, bevel_r=0.0002)
    hole = prism([(-0.0011, -0.006), (0.0011, -0.006), (0.0007, -0.001), (0.0018, 0.0), (0.0018, 0.0022), (0.0, 0.0034), (-0.0018, 0.0022), (-0.0018, 0.0), (-0.0007, -0.001)], 0.004)
    esc = cut(plate, hole)
    out = out + esc.move(0, HU_HB - 0.020, HU_D / 2 + 0.0004)
    return out


def humidor_lid_brass():
    out = Shape()
    for sx in (-1, 1):
        out = out + hu_hinge_leaf(h=0.010).move(sx * 0.075, 0.0055, -0.00045)
    # the name plate let into the top, its corners cut
    plate = prism(round_outline([(-0.034, -0.012), (-0.030, -0.016), (0.030, -0.016), (0.034, -0.012),
                                 (0.034, 0.012), (0.030, 0.016), (-0.030, 0.016), (-0.034, 0.012)], 0.001, seg=1), 0.0008, bevel_r=0.0002)
    plate.rot(x=-math.pi / 2).move(0, HU_HL + 0.0012 + 0.0002, HU_D / 2)
    return out + plate


def cigar_body():
    """A Corona along +y about its middle: the foot cut square at -y, the
    head's cap rounded at +y, the wrapper's spiral seam a ridge."""
    h = CG_L / 2
    prof = [(0.0, -h), (CG_R * 0.97, -h), (CG_R, -h + 0.0015)]
    for k in range(1, 12):
        y = -h + 0.0015 + (CG_L - 0.022) * k / 11
        prof.append((CG_R * (1.0 + 0.012 * math.sin(k * 1.7)), y))
    prof += [(CG_R * 0.96, h - 0.012), (CG_R * 0.86, h - 0.0065), (CG_R * 0.66, h - 0.0028), (CG_R * 0.36, h - 0.0006), (0.0, h)]
    s = lathe(prof, seg=14)
    # the seam of the wrapper running round it, and a slight bend
    def wr(p):
        a = math.atan2(p.z, p.x)
        ridge = 0.00018 * max(0.0, math.cos(a * 1.0 - p.y * 70.0)) ** 8
        r = math.hypot(p.x, p.z)
        if r > 1e-6:
            k = (r + ridge) / r
            p = Vector((p.x * k, p.y, p.z * k))
        return Vector((p.x + 0.0006 * (1 - (p.y / h) ** 2), p.y, p.z))
    return s.warp(wr)


def cigar_band():
    """The paper band round it near its head, about the cigar's axis (+y)."""
    y0 = CG_L / 2 - 0.034
    return ring_lathe([(CG_R + 0.00005, y0), (CG_R + 0.00045, y0), (CG_R + 0.00045, y0 + 0.017), (CG_R + 0.00005, y0 + 0.017)], seg=18)


def cigar_foot():
    """The cut foot's filler, a disc a hair proud of the foot."""
    return cyl(CG_R * 0.93, h=0.0004, seg=14, bevel_r=0).move(0, -CG_L / 2 - 0.0002, 0)


def cigar_ash():
    """A smoked cigar's ash: a grey stub on its foot, cracked in rings, its end ragged."""
    prof = [(0.0, -CG_L / 2 + 0.0005), (CG_R * 1.01, -CG_L / 2 + 0.0005)]
    for k in range(1, 6):
        prof.append((CG_R * (1.0 - 0.035 * k + 0.03 * (k % 2)), -CG_L / 2 - 0.0028 * k))
    prof += [(CG_R * 0.7, -CG_L / 2 - 0.0155), (0.0, -CG_L / 2 - 0.0162)]
    s = lathe(prof, seg=14)
    return noisy(s, 0.00035, scale=600.0, seed=7)


def humidor_cutter():
    """A steel guillotine cutter lying flat: its body with the hole the cigar
    goes in, the blade slid out on its finger loop."""
    body = prism(round_outline([(-0.022, -0.014), (0.022, -0.014), (0.026, -0.008), (0.026, 0.008), (0.022, 0.014), (-0.022, 0.014), (-0.026, 0.008), (-0.026, -0.008)], 0.004, seg=2), 0.0045, bevel_r=0.0006)
    body = cut(body, cyl(0.0098, h=0.01, seg=20).rot(x=math.pi / 2).move(0.004, 0, 0))
    body.rot(x=-math.pi / 2).move(0, 0.00225, 0)
    # the blade between the cheeks, slid out to its loop
    blade = prism(round_outline([(-0.014, -0.011), (0.016, -0.011), (0.016, 0.011), (-0.014, 0.011)], 0.003, seg=1), 0.0008, bevel_r=0.0002)
    blade.rot(x=-math.pi / 2).move(-0.012, 0.0045 + 0.0004, 0)
    loop = torus(0.0085, 0.0018, seg=18, tseg=6).rot(x=math.pi / 2).move(-0.036, 0.0019, 0)
    return body + blade + loop


def humidor_matchbook():
    """A paper matchbook: its cover folded over the comb of matches, the
    flap tucked behind the striker, the staple."""
    body = rbox(0.038, 0.0055, 0.047, r=0.0012, seg=2).move(0, 0.00275, 0)
    flap = rbox(0.038, 0.0009, 0.011, r=0.0003, seg=1).move(0, 0.0058, 0.0175)
    staple = box(0.012, 0.0004, 0.0012).move(0, 0.0064, 0.016)
    return body + flap + staple


auto('Real__humidorBox', humidor_box, 900)
auto('Real__humidorLining', humidor_lining, 300)
auto('Real__humidorLid', humidor_lid, 1100)
auto('Real__humidorLidLining', humidor_lid_lining, 120)
auto('Real__humidorBrass', humidor_brass, 900)
auto('Real__humidorLidBrass', humidor_lid_brass, 300)
auto('Real__humidorCigar', cigar_body, 600, kind='cyl')
auto('Real__humidorCigarBand', cigar_band, 200, kind='cyl')
auto('Real__humidorCigarFoot', cigar_foot, 60)
auto('Real__humidorCigarAsh', cigar_ash, 300)
auto('Real__humidorCutter', humidor_cutter, 900)
auto('Real__humidorMatchbook', humidor_matchbook, 300)


# ======================================================= the pocket watch ===
# A 16-size hunter-case pocket watch of about 1900, 50 mm across its case,
# lying on its back: the domed back, the band, the bezel and its crystal,
# the front cover open on its hinge at IX, the pendant, crown and bow at III
# (+x); a white enamel dial, blued steel spade hands; its Albert chain of
# curb links to a T-bar or a key (t20.notes.md).
PW_R = 0.0250                 # the band
PW_DIAL_R, PW_DIAL_Y = 0.0216, 0.0090
PW_HINGE = (-0.0256, 0.0106)  # the cover's hinge line (x, y), along z
PW_PEND_Y = 0.0058            # the pendant's axis (y)


def watch_case():
    prof = [(0.0, 0.0), (0.008, 0.0002), (0.015, 0.0008), (0.020, 0.0018), (0.0236, 0.0034), (0.0248, 0.0048),
            (PW_R, 0.0058), (PW_R, 0.0086), (0.0247, 0.0094), (0.0239, 0.0100), (0.0230, 0.0104), (0.0222, 0.0103),
            (0.0219, 0.0097), (0.0219, PW_DIAL_Y - 0.0004), (0.0, PW_DIAL_Y - 0.0004)]
    s = lathe(prof, seg=40)
    # the case's two knuckles of the cover's hinge at IX
    for z in (-0.0062, 0.0062):
        s = s + cyl(0.0016, h=0.0036, seg=8, bevel_r=0.0003).rot(x=math.pi / 2).move(PW_HINGE[0], PW_HINGE[1] - 0.0002, z)
    # the pendant at III, its collar, the knurled crown
    pend = lathe([(0.0, 0.0), (0.0031, 0.0), (0.0031, 0.0012), (0.0026, 0.0018), (0.0026, 0.0052), (0.0030, 0.0056),
                  (0.0030, 0.0064), (0.0, 0.0064)], seg=12).rot(z=-math.pi / 2).move(PW_R - 0.0008, PW_PEND_Y, 0)
    crown = cyl(0.0043, h=0.0052, seg=24, bevel_r=0.0006)
    crown = crown.warp(lambda p: Vector((p.x * (1 + 0.07 * max(0.0, math.cos(12 * math.atan2(p.z, p.x)))), p.y, p.z * (1 + 0.07 * max(0.0, math.cos(12 * math.atan2(p.z, p.x)))))))
    crown.rot(z=-math.pi / 2).move(PW_R + 0.0056 + 0.0026, PW_PEND_Y, 0)
    # the bow: a loop through the pendant's sides, fallen onto the floor
    bow = []
    for i in range(25):
        a = math.pi * 2 * i / 24
        bow.append((0.0098 * math.cos(a), 0.0, 0.0072 * math.sin(a)))
    bow = sweep(bow, lambda t: 0.0011, seg=6)
    piv_x = PW_R + 0.0040
    bow.move(0.0098 - 0.0001, 0, 0)
    drop = math.atan2(PW_PEND_Y - 0.0011, 0.0196)
    bow.rot(z=-drop).move(piv_x, PW_PEND_Y, 0)
    return s + pend + crown + bow


def watch_cover():
    """The front lid, shut, its hinge line on the z axis and itself out to +x."""
    prof = [(0.0, 0.0146), (0.010, 0.0144), (0.017, 0.0138), (0.0222, 0.0127), (0.0245, 0.0116), (PW_R, 0.0108),
            (PW_R, 0.0102), (0.0242, 0.0102), (0.0238, 0.0108), (0.0215, 0.0119), (0.016, 0.0129), (0.0, 0.0134)]
    s = ring_lathe(list(reversed(prof))[:-1] + [(0.0001, 0.0134), (0.0001, 0.0146)], seg=40)
    s = s + cyl(0.0016, h=0.0080, seg=8, bevel_r=0.0003).rot(x=math.pi / 2).move(PW_HINGE[0], PW_HINGE[1] - 0.0002, 0)
    # the lift's spring catch at III
    s = s + rbox(0.0024, 0.0012, 0.0036, r=0.0004, seg=1).move(PW_R + 0.0006, 0.0104, 0)
    return s.move(-PW_HINGE[0], -PW_HINGE[1], 0)


def watch_dial():
    return cyl(PW_DIAL_R, h=0.0005, seg=40, bevel_r=0.0001).move(0, PW_DIAL_Y - 0.00025, 0)


def watch_crystal():
    prof = [(0.0, 0.0116), (0.008, 0.0115), (0.015, 0.0111), (0.0196, 0.0106), (0.0220, 0.0101), (0.0220, 0.0097),
            (0.0214, 0.0097), (0.019, 0.0101), (0.014, 0.0106), (0.007, 0.0109), (0.0, 0.0110)]
    return ring_lathe(list(reversed(prof))[:-1] + [(0.0001, 0.0110), (0.0001, 0.0116)], seg=32)


def watch_hand(length, kind):
    """A blued steel hand pointing to -z from its arbor at the origin, flat
    (0.25 mm): a spade for the hour, a whip for the minute, a needle for
    the seconds."""
    t = 0.00025
    pts = []
    if kind == 'spade':
        stem = 0.00045
        sp_c, sp_r = length * 0.62, length * 0.16
        pts = [(stem, -0.0016), (stem, -sp_c + sp_r * 0.7), (sp_r * 0.9, -sp_c), (sp_r * 0.55, -sp_c - sp_r * 0.8),
               (0.00035, -length + 0.0006), (0.0, -length), (-0.00035, -length + 0.0006), (-sp_r * 0.55, -sp_c - sp_r * 0.8),
               (-sp_r * 0.9, -sp_c), (-stem, -sp_c + sp_r * 0.7), (-stem, -0.0016)]
    elif kind == 'whip':
        pts = [(0.0004, -0.0016), (0.00055, -length * 0.55), (0.0003, -length * 0.85), (0.0, -length), (-0.0003, -length * 0.85),
               (-0.00055, -length * 0.55), (-0.0004, -0.0016)]
    else:
        pts = [(0.00018, 0.0012), (0.00018, -length * 0.7), (0.0, -length), (-0.00018, -length * 0.7), (-0.00018, 0.0012)]
    # the boss round the arbor
    boss_r = 0.0011 if kind != 'needle' else 0.0006
    boss = [(boss_r * math.sin(a), boss_r * math.cos(a)) for a in [math.pi * 2 * k / 12 for k in range(12)]]
    hand = prism([(x, -z) for x, z in pts], t).rot(x=-math.pi / 2)
    b = prism(boss, t * 1.6).rot(x=-math.pi / 2)
    return hand + b


def watch_link():
    """One curb link along x: an oval of round wire, twisted so it lies flat
    against its neighbours."""
    pts = []
    for i in range(11):
        a = math.pi * 2 * i / 10
        pts.append((0.0029 * math.cos(a), 0.00045 * math.sin(2 * a), 0.0018 * math.sin(a)))
    return sweep(pts, lambda t: 0.00055, seg=5).move(0, 0.0006, 0)


def watch_tbar():
    bar = capsule(0.0013, 0.024, seg=10, cap_rings=2).rot(x=math.pi / 2)
    knobs = [sphere(0.0019, seg=10, rings=6).move(0, 0, s * 0.0130) for s in (-1, 1)]
    ring = torus(0.0028, 0.0007, seg=14, tseg=5).rot(y=math.pi / 2).move(-0.0035, 0, 0)
    return (bar + sum(knobs) + ring).move(0, 0.0019, 0)


def watch_key():
    """A watch key lying flat: its barrel along x (the square socket at its
    end), its flat bow pierced."""
    barrel = cyl(0.0019, h=0.012, seg=10, bevel_r=0.0003).rot(z=math.pi / 2).move(0.006, 0, 0)
    bow = prism(round_outline([(-0.0050, -0.0045), (0.0010, -0.0045), (0.0020, 0.0), (0.0010, 0.0045), (-0.0050, 0.0045), (-0.0065, 0.0)], 0.0015, seg=2), 0.0012, bevel_r=0.0002)
    bow = cut(bow, cyl(0.0022, h=0.004, seg=12).rot(x=math.pi / 2).move(-0.0028, 0, 0))
    ring = torus(0.0024, 0.0006, seg=12, tseg=5).rot(y=math.pi / 2).move(-0.0085, 0, 0)
    return (barrel + bow + ring).move(0, 0.0019, 0)


auto('Real__pocketWatchCase', watch_case, 3000)
auto('Real__pocketWatchCover', watch_cover, 1250)
auto('Real__pocketWatchDial', watch_dial, 500)
auto('Real__pocketWatchCrystal', watch_crystal, 800)
auto('Real__pocketWatchHour', lambda: watch_hand(0.0125, 'spade'), 120)
auto('Real__pocketWatchMinute', lambda: watch_hand(0.0185, 'whip'), 120)
auto('Real__pocketWatchSecond', lambda: watch_hand(0.0052, 'needle'), 100)
auto('Real__pocketWatchLink', watch_link, 140)
auto('Real__pocketWatchTBar', watch_tbar, 500)
auto('Real__pocketWatchKey', watch_key, 500)


# ========================================================= the music box ===
# A Swiss child's hand-crank music box of about 1900, 100 x 70 x 44 mm: a
# box of stained or painted beech, its lid hinged at the back with a print
# pasted inside it; under a glass the works on their brass bedplate: the
# pinned cylinder, the steel comb of 18 teeth, the wheel the crank turns
# (t20.notes.md). Its crank is on its right (+x).
MB_W, MB_D = 0.100, 0.070
MB_HB, MB_HL = 0.032, 0.012
MB_T, MB_BOT = 0.005, 0.004
MB_CYL = (-0.004, 0.0175, -0.009)       # the cylinder's middle
MB_CYL_R, MB_CYL_L = 0.0062, 0.040


def music_case():
    outer = rbox(MB_W, MB_HB, MB_D, r=0.0016, seg=2).move(0, MB_HB / 2, 0)
    inner = box(MB_W - 2 * MB_T, MB_HB, MB_D - 2 * MB_T).move(0, MB_BOT + MB_HB / 2, 0)
    s = cut(outer, inner)
    # the fillets the glass rests on
    for sz in (-1, 1):
        s = s + box(MB_W - 2 * MB_T, 0.003, 0.003).move(0, 0.0255, sz * (MB_D / 2 - MB_T - 0.0015))
    # a moulding round its foot
    prof = [(0.0, 0.0), (0.0018, 0.0), (0.0018, 0.0022), (0.0008, 0.0034), (0.0, 0.0040)]
    rings = [[(-MB_W / 2 - d, y, -MB_D / 2 - d), (MB_W / 2 + d, y, -MB_D / 2 - d), (MB_W / 2 + d, y, MB_D / 2 + d), (-MB_W / 2 - d, y, MB_D / 2 + d)] for d, y in prof]
    return s + loft(rings)


def music_lid():
    outer = rbox(MB_W, MB_HL, MB_D, r=0.0022, seg=2).move(0, MB_HL / 2, MB_D / 2)
    inner = box(MB_W - 2 * MB_T, MB_HL - 0.003, MB_D - 2 * MB_T).move(0, (MB_HL - 0.003) / 2 - 0.0005, MB_D / 2)
    s = cut(outer, inner)
    # a small turned knob of wood on its front edge, to lift it by
    knob = cyl(0.0022, h=0.003, seg=10, bevel_r=0.0008).rot(x=math.pi / 2).move(0, MB_HL * 0.5, MB_D + 0.0014)
    return s + knob


def music_paper():
    """The print pasted inside the lid, on the hollow's ceiling, facing down."""
    return box(MB_W - 2 * MB_T - 0.002, 0.0003, MB_D - 2 * MB_T - 0.002).move(0, MB_HL - 0.003 - 0.00015, MB_D / 2)


def music_brass():
    """The bedplate on its pillars, the cylinder's bearing brackets, the
    pinned cylinder, the great wheel at its end, the crank's shaft through
    the right wall and its arm outside; the hinges' knuckles."""
    cx, cy, cz = MB_CYL
    bed = rbox(0.082, 0.0018, 0.052, r=0.0005, seg=1).move(0, MB_BOT + 0.003, -0.002)
    pillars = [cyl(0.002, h=0.003, seg=8, bevel_r=0).move(sx * 0.036, MB_BOT + 0.0015, sz * 0.020 - 0.002) for sx in (-1, 1) for sz in (-1, 1)]
    brackets = [rbox(0.0025, cy - MB_BOT - 0.003, 0.010, r=0.0006, seg=1).move(cx + s * (MB_CYL_L / 2 + 0.003), (cy + MB_BOT + 0.004) / 2, cz) for s in (-1, 1)]
    cylinder = cyl(MB_CYL_R, h=MB_CYL_L, seg=24, bevel_r=0.0004).rot(z=math.pi / 2).move(cx, cy, cz)
    arbor = cyl(0.0009, h=MB_CYL_L + 0.010, seg=8, bevel_r=0).rot(z=math.pi / 2).move(cx, cy, cz)
    # the great wheel's teeth
    teeth = 36
    outline = []
    for k in range(teeth * 2):
        a = math.pi * k / teeth
        r = 0.0092 if k % 2 == 0 else 0.0084
        outline.append((r * math.cos(a), r * math.sin(a)))
    wheel = prism(outline, 0.0012).rot(y=math.pi / 2).move(cx + MB_CYL_L / 2 + 0.0055, cy, cz)
    shaft = cyl(0.0012, h=MB_W / 2 - (cx + MB_CYL_L / 2) + 0.004, seg=8, bevel_r=0).rot(z=math.pi / 2)
    shaft.move((cx + MB_CYL_L / 2 + MB_W / 2 + 0.004) / 2, cy, cz)
    # the crank outside the right wall: a boss, its arm up, the knob's pin out
    boss = cyl(0.0028, h=0.0025, seg=12, bevel_r=0.0004).rot(z=math.pi / 2).move(MB_W / 2 + 0.0030, cy, cz)
    arm = prism(round_outline([(-0.0025, -0.0025), (0.0025, -0.0025), (0.0020, 0.0175), (-0.0020, 0.0175)], 0.0018, seg=2), 0.0016, bevel_r=0.0003)
    arm.rot(y=math.pi / 2).move(MB_W / 2 + 0.0050, cy, cz)
    pin = cyl(0.0008, h=0.004, seg=6, bevel_r=0).rot(z=math.pi / 2).move(MB_W / 2 + 0.0075, cy + 0.0155, cz)
    hinges = [cyl(0.0013, h=0.012, seg=8, bevel_r=0.0002).rot(z=math.pi / 2).move(sx * 0.030, MB_HB, -MB_D / 2 - 0.0008) for sx in (-1, 1)]
    leaves = [box(0.012, 0.005, 0.0006).move(sx * 0.030, MB_HB - 0.0028, -MB_D / 2 - 0.0003) for sx in (-1, 1)]
    return bed + sum(pillars) + sum(brackets) + cylinder + arbor + wheel + shaft + boss + arm + pin + sum(hinges) + sum(leaves)


def music_comb():
    """The steel comb: its base screwed down in front of the cylinder, 18
    teeth reaching back to it, the bass teeth longer; the cylinder's pins."""
    cx, cy, cz = MB_CYL
    n = 18
    tw, gap = 0.0016, 0.00045
    wid = n * tw + (n - 1) * gap
    base_z0 = cz + MB_CYL_R + 0.010
    base = rbox(wid + 0.004, 0.0028, 0.010, r=0.0005, seg=1).move(cx, cy - MB_CYL_R + 0.0004, base_z0 + 0.005)
    out = base
    for k in range(n):
        x = cx - wid / 2 + tw / 2 + k * (tw + gap)
        tip = cz + MB_CYL_R * 0.55
        ln = base_z0 - tip + 0.0003 * (n - k) / n
        th = 0.0007 + 0.0005 * (n - k) / n
        t = box(tw, th, ln).move(x, cy - MB_CYL_R + 0.0012 - th / 2 + 0.0006, base_z0 - ln / 2 + 0.0002)
        out = out + t
    for sx in (-1, 0, 1):
        out = out + dome_tack(0.0012, 0.0008, seg=6).move(cx + sx * (wid / 2 - 0.002), cy - MB_CYL_R + 0.0018, base_z0 + 0.006)
    # the pins: short wires standing out of the cylinder, a tune's worth
    pin = cyl(0.00022, h=0.0007, seg=4, bevel_r=0)
    for k in range(n):
        x = cx - wid / 2 + tw / 2 + k * (tw + gap)
        for j in range(random.randint(3, 6)):
            a = random.uniform(0, TAU)
            p = pin.copy().move(0, MB_CYL_R + 0.00025, 0).rot(x=a).move(x, cy, cz)
            out = out + p
    return out


def music_glass():
    return box(MB_W - 2 * MB_T - 0.001, 0.0015, MB_D - 2 * MB_T - 0.001).move(0, 0.0255 + 0.0015 + 0.00075, 0)


def music_knob():
    """A turned porcelain knob on the crank's pin, out along +x."""
    s = lathe([(0.0, 0.0), (0.0016, 0.0), (0.0021, 0.0012), (0.0030, 0.0045), (0.0033, 0.0075), (0.0028, 0.0098), (0.0016, 0.0108), (0.0, 0.0111)], seg=12)
    cx, cy, cz = MB_CYL
    return s.rot(z=-math.pi / 2).move(MB_W / 2 + 0.0090, cy + 0.0155, cz)


auto('Real__musicBoxCase', music_case, 500)
auto('Real__musicBoxLid', music_lid, 500)
auto('Real__musicBoxPaper', music_paper, 40)
auto('Real__musicBoxBrass', music_brass, 1600)
auto('Real__musicBoxComb', music_comb, 1650)
auto('Real__musicBoxGlass', music_glass, 40)
auto('Real__musicBoxKnob', music_knob, 200)


# ============================================================ the globe ===
# A 12 in (305 mm) terrestrial globe on a floor stand, about 890 mm to the
# top of its meridian: a library stand of three turned legs, stretchers and
# a wooden horizon ring papered with its calendar; or a turned pedestal on
# a cross of four feet. A full brass meridian, the axis pins, an hour circle
# at the north pole (t20.notes.md).
GL_R = 0.1525
GL_CY = 0.720                         # the ball's middle
GL_MER = (0.1555, 0.1590, 0.0050)     # the meridian: inner and outer radius, half its width
GL_HOR = (0.172, 0.218, 0.014)        # the horizon: inner and outer radius, thickness
GL_SOCKET_Y = GL_CY - GL_MER[1] - 0.003


def globe_ball():
    # a sphere of more rings than a lathe's default, so the map's coasts hold
    return sphere(GL_R, seg=48, rings=26)


def globe_meridian():
    r0, r1, hw = GL_MER
    ring = ring_lathe([(r0, -hw), (r1, -hw), (r1, hw), (r0, hw)], seg=72).rot(x=math.pi / 2)
    return bevel(ring, 0.0004, seg=1)


def globe_pins():
    """The axis along +y: a pin through each pole, a nut on each, the hour
    circle on the meridian over the north pole and its pointer."""
    north = cyl(0.0018, h=0.022, seg=8, bevel_r=0).move(0, GL_R + 0.008, 0)
    nut_n = cyl(0.0032, h=0.0025, seg=6, bevel_r=0.0003).move(0, GL_MER[1] + 0.0042, 0)
    south = cyl(0.0018, h=0.018, seg=8, bevel_r=0).move(0, -GL_R - 0.006, 0)
    nut_s = cyl(0.0032, h=0.0025, seg=6, bevel_r=0.0003).move(0, -GL_MER[1] - 0.0013, 0)
    dial = cyl(0.021, h=0.0009, seg=32, bevel_r=0.0002).move(0, GL_MER[1] + 0.0016, 0)
    hand = box(0.0016, 0.0005, 0.016).move(0, GL_MER[1] + 0.0058, 0.006)
    return north + nut_n + south + nut_s + dial + hand


def turned(prof, seg=16):
    """A turned piece: a profile [(r, y)] up its axis, capped."""
    return lathe([(0.0, prof[0][1])] + prof + [(0.0, prof[-1][1])], seg=seg)


def gl_socket():
    """The cup on the stand's top the meridian stands in, slotted for it."""
    cup = turned([(0.011, -0.030), (0.016, -0.026), (0.018, -0.018), (0.014, -0.012), (0.016, -0.004), (0.016, 0.0)], seg=16)
    cup = cut(cup, box(0.06, 0.02, GL_MER[2] * 2 + 0.001).move(0, 0.004, 0))
    return cup.move(0, GL_SOCKET_Y + 0.006, 0)


# a turned leg's radius down its length (0 under the horizon, 1 at the foot)
GL_LEG = [(0.0, 0.014), (0.05, 0.016), (0.08, 0.012), (0.14, 0.015), (0.30, 0.022), (0.46, 0.016), (0.56, 0.012),
          (0.60, 0.017), (0.64, 0.012), (0.90, 0.010), (0.96, 0.013), (1.0, 0.011)]


def interp(table, t):
    for (t0, a), (t1, b) in zip(table, table[1:]):
        if t <= t1:
            return a + (b - a) * (t - t0) / max(t1 - t0, 1e-9)
    return table[-1][1]


def globe_tripod():
    """Three turned legs from under the horizon out to the floor, stretchers
    to a turned hub, the column up to the meridian's socket."""
    out = Shape()
    hub_y = 0.200
    for k in range(3):
        a = math.pi / 2 + TAU * k / 3
        ca, sa = math.cos(a), math.sin(a)
        top = Vector((GL_HOR[0] * 1.12 * ca, GL_CY - GL_HOR[2], GL_HOR[0] * 1.12 * sa))
        foot = Vector((0.265 * ca, 0.020, 0.265 * sa))
        path = []
        for i in range(33):
            t = i / 32
            p = top.lerp(foot, t)
            # a slight outward bow
            p += Vector((ca, 0, sa)) * 0.006 * math.sin(math.pi * t)
            path.append(p)
        out = out + sweep(path, lambda t: interp(GL_LEG, t), seg=10)
        # a bun foot
        out = out + turned([(0.014, 0.0), (0.021, 0.007), (0.022, 0.014), (0.016, 0.022), (0.011, 0.026)], seg=14).move(foot.x, 0, foot.z)
        # the stretcher to the hub
        sp = top.lerp(foot, (top.y - hub_y) / (top.y - foot.y)) + Vector((ca, 0, sa)) * 0.016
        out = out + sweep([Vector((0.02 * ca, hub_y, 0.02 * sa)), Vector((sp.x, hub_y, sp.z))], lambda t: 0.0075, seg=8)
        # a block under the horizon where the leg is tenoned in
        out = out + rbox(0.040, 0.016, 0.028, r=0.003, seg=1).rot(y=-a).move(top.x, GL_CY - GL_HOR[2] - 0.008, top.z)
    # the hub, a finial hanging under it, the column
    out = out + turned([(0.022, hub_y - 0.016), (0.026, hub_y - 0.008), (0.026, hub_y + 0.008), (0.022, hub_y + 0.016)], seg=16)
    out = out + turned([(0.004, hub_y - 0.070), (0.010, hub_y - 0.060), (0.013, hub_y - 0.045), (0.008, hub_y - 0.030), (0.015, hub_y - 0.016)], seg=12)
    col = [(0.016, hub_y + 0.016), (0.019, hub_y + 0.03), (0.013, hub_y + 0.06), (0.010, 0.40), (0.012, 0.44), (0.017, 0.47),
           (0.011, 0.50), (0.010, GL_SOCKET_Y - 0.026)]
    out = out + turned(col, seg=14)
    return out + gl_socket()


def globe_horizon():
    r0, r1, t = GL_HOR
    ring = ring_lathe([(r0, GL_CY - t), (r1 - 0.004, GL_CY - t), (r1, GL_CY - t + 0.004), (r1, GL_CY - 0.002),
                       (r1 - 0.002, GL_CY), (r0, GL_CY)], seg=64)
    # the brass guides at N and S that the meridian runs in
    return bevel(ring, 0.0005, seg=1)


def globe_horizon_paper():
    r0, r1, t = GL_HOR
    return ring_lathe([(r0 + 0.002, GL_CY), (r1 - 0.004, GL_CY), (r1 - 0.004, GL_CY + 0.0004), (r0 + 0.002, GL_CY + 0.0004)], seg=64)


def globe_pedestal():
    """A turned column on a cross of four feet, its socket on top."""
    out = Shape()
    for k in range(4):
        a = TAU * k / 4 + math.pi / 4
        arm = prism(round_outline([(0.0, -0.022), (0.215, -0.014), (0.225, 0.0), (0.215, 0.014), (0.0, 0.022)], 0.006, seg=2), 0.030, bevel_r=0.003, bevel_seg=2)
        arm.rot(x=-math.pi / 2).rot(y=-a).move(0, 0.033, 0)
        out = out + arm
        out = out + turned([(0.014, 0.0), (0.020, 0.006), (0.020, 0.013), (0.015, 0.018)], seg=12).move(0.205 * math.cos(a), 0, 0.205 * math.sin(a))
    col = [(0.045, 0.048), (0.050, 0.060), (0.042, 0.075), (0.034, 0.09), (0.040, 0.13), (0.046, 0.18), (0.040, 0.24),
           (0.026, 0.30), (0.020, 0.36), (0.026, 0.38), (0.018, 0.40), (0.015, 0.46), (0.017, 0.50), (0.013, GL_SOCKET_Y - 0.026)]
    out = out + turned(col, seg=18)
    return out + gl_socket()


auto('Real__globeBall', globe_ball, 2600, kind='sphere')
auto('Real__globeMeridian', globe_meridian, 1200)
auto('Real__globePins', globe_pins, 650)
auto('Real__globeTripod', globe_tripod, 3500)
auto('Real__globeHorizon', globe_horizon, 1600)
auto('Real__globeHorizonPaper', globe_horizon_paper, 560)
auto('Real__globePedestal', globe_pedestal, 2400)


# =========================================================== the orrery ===
# A London planetary orrery of about 1820, 300 mm across its base (Newton's
# of 326 mm, Whipple Museum): a turned mahogany drum on three feet, its
# printed paper dial, a brass pillar carrying the brass sun and, on their
# own sleeves, five brass arms with their posts and the ivory planets,
# Mercury to Jupiter, the Earth's moon on its little arm; a crank handle in
# its side (t20.notes.md).
OR_R = 0.150
OR_TOP = 0.088                        # the dial's face
OR_ORBIT = (0.050, 0.075, 0.104, 0.135, 0.178)
OR_SIZE = (0.0042, 0.0072, 0.0082, 0.0058, 0.0145)
OR_ARM_Y = (0.176, 0.158, 0.140, 0.122, 0.104)
OR_PLANET_Y = 0.236
OR_SUN = (0.264, 0.030)


def orrery_base():
    drum = turned([(OR_R - 0.004, 0.050), (OR_R, 0.054), (OR_R, 0.064), (OR_R - 0.004, 0.068), (OR_R - 0.004, 0.078),
                   (OR_R + 0.002, 0.083), (OR_R + 0.002, OR_TOP - 0.001), (OR_R, OR_TOP)], seg=56)
    feet = Shape()
    for k in range(3):
        a = math.pi / 2 + TAU * k / 3
        f = turned([(0.010, 0.0), (0.014, 0.008), (0.012, 0.016), (0.008, 0.026), (0.010, 0.036), (0.016, 0.044), (0.014, 0.050)], seg=12)
        feet = feet + f.move(0.112 * math.cos(a), 0, 0.112 * math.sin(a))
    return drum + feet


def orrery_dial():
    return cyl(OR_R - 0.008, h=0.0005, seg=56, bevel_r=0).move(0, OR_TOP + 0.00025, 0)


def orrery_pillar():
    bezel = ring_lathe([(OR_R - 0.008, OR_TOP), (OR_R - 0.002, OR_TOP), (OR_R - 0.002, OR_TOP + 0.0022), (OR_R - 0.0065, OR_TOP + 0.0022)], seg=56)
    foot = turned([(0.020, OR_TOP), (0.020, OR_TOP + 0.003), (0.014, OR_TOP + 0.006), (0.0095, OR_TOP + 0.010)], seg=18)
    col = turned([(0.0085, OR_TOP + 0.005), (0.0085, OR_SUN[0] - OR_SUN[1] - 0.004), (0.011, OR_SUN[0] - OR_SUN[1] - 0.002),
                  (0.006, OR_SUN[0] - OR_SUN[1] + 0.002)], seg=14)
    return bezel + foot + col


def orrery_sun():
    s = sphere(OR_SUN[1], seg=28, rings=16)
    fin = turned([(0.003, OR_SUN[1] - 0.001), (0.0045, OR_SUN[1] + 0.002), (0.002, OR_SUN[1] + 0.005), (0.0028, OR_SUN[1] + 0.0075)], seg=10)
    return (s + fin).move(0, OR_SUN[0], 0)


def orrery_arm(i):
    """Arm i along +x from its sleeve on the pillar to its post, the post up
    to the planet's cup; the Earth's carries the moon's little arm."""
    y, r = OR_ARM_Y[i], OR_ORBIT[i]
    sleeve = ring_lathe([(0.0090, y - 0.004), (0.0125 + 0.0006 * i, y - 0.004), (0.0125 + 0.0006 * i, y + 0.004), (0.0090, y + 0.004)], seg=18)
    bar = prism(round_outline([(0.008, -0.0032), (r, -0.0026), (r + 0.0045, 0.0), (r, 0.0026), (0.008, 0.0032)], 0.0015, seg=2), 0.0026, bevel_r=0.0004)
    bar.rot(x=-math.pi / 2).move(0, y, 0)
    boss = cyl(0.0040, h=0.006, seg=12, bevel_r=0.0006).move(r, y + 0.001, 0)
    top = OR_PLANET_Y - OR_SIZE[i] - 0.0015
    post = cyl(0.0013, h=top - y, seg=8, bevel_r=0).move(r, (top + y) / 2, 0)
    cup = turned([(0.0013, top - 0.001), (0.0026, top + 0.0005), (0.0022, top + 0.0016)], seg=10).move(r, 0, 0)
    s = sleeve + bar + boss + post + cup
    if i == 2:
        # the moon's arm from the Earth's post, its own small post
        my = OR_PLANET_Y - 0.020
        s = s + cyl(0.0009, h=0.024, seg=6, bevel_r=0).rot(z=math.pi / 2).move(r + 0.012, my, 0)
        s = s + cyl(0.0008, h=OR_PLANET_Y - my - 0.003, seg=6, bevel_r=0).move(r + 0.024, (OR_PLANET_Y - 0.003 + my) / 2, 0)
    return s


def orrery_planet(i):
    return sphere(OR_SIZE[i], seg=18 if i < 4 else 24, rings=10 if i < 4 else 14)


def orrery_moon():
    return sphere(0.0026, seg=10, rings=6).move(0.024, 0, 0)


def orrery_crank():
    """The crank in the drum's side at +x: its boss, the arm, the pin."""
    y = 0.066
    boss = cyl(0.0055, h=0.008, seg=14, bevel_r=0.0008).rot(z=math.pi / 2).move(OR_R + 0.003, y, 0)
    arm = prism(round_outline([(-0.004, -0.004), (0.004, -0.004), (0.003, 0.028), (-0.003, 0.028)], 0.0028, seg=2), 0.003, bevel_r=0.0005)
    arm.rot(y=math.pi / 2).rot(x=math.pi * 0.62).move(OR_R + 0.0085, y, 0)
    return boss + arm


def orrery_crank_knob():
    a = math.pi * 0.62
    y = 0.066 + 0.0255 * math.cos(a)
    z = 0.0255 * math.sin(a)
    k = lathe([(0.0, 0.0), (0.0028, 0.0), (0.0040, 0.006), (0.0042, 0.013), (0.0030, 0.019), (0.0, 0.0205)], seg=12)
    return k.rot(z=-math.pi / 2).move(OR_R + 0.010, y, z)


auto('Real__orreryBase', orrery_base, 1800)
auto('Real__orreryDial', orrery_dial, 240)
auto('Real__orreryPillar', orrery_pillar, 900)
auto('Real__orrerySun', orrery_sun, 1100)
for _i in range(5):
    auto(f'Real__orreryArm{_i}', lambda i=_i: orrery_arm(i), 700)
    auto(f'Real__orreryPlanet{_i}', lambda i=_i: orrery_planet(i), 700, kind='sphere')
auto('Real__orreryMoon', orrery_moon, 120)
auto('Real__orreryCrank', orrery_crank, 500)
auto('Real__orreryCrankKnob', orrery_crank_knob, 200)


# ============================================================ the chess ===
# A Victorian tripod table, its round top 520 mm across and 700 mm high, a
# folding board of 35 mm squares on it and a game of Staunton chessmen
# (boxwood and ebony, a 64 mm king) left on it; a small cast brass ashtray
# with a cigar gone out on its rest (t20.notes.md).
CH_TOP_R, CH_TOP_Y = 0.260, 0.700       # the top's radius, its face
CH_BOARD = 0.300                        # the board: 8 x 35 mm and a 10 mm border
CH_BOARD_T = 0.014
CH_KING = 0.064


def chess_table_top():
    t = 0.020
    y0 = CH_TOP_Y - t
    top = turned([(CH_TOP_R - 0.012, y0 - 0.002), (CH_TOP_R - 0.004, y0), (CH_TOP_R, y0 + 0.004), (CH_TOP_R + 0.001, y0 + 0.012),
                  (CH_TOP_R - 0.001, y0 + 0.017), (CH_TOP_R - 0.004, CH_TOP_Y), (CH_TOP_R - 0.010, CH_TOP_Y + 0.0006)], seg=64)
    # under it: the two bearers and the block the column's tenon goes into
    bearers = [rbox(0.38, 0.022, 0.040, r=0.004, seg=1).move(0, y0 - 0.011, s * 0.085) for s in (-1, 1)]
    block = rbox(0.13, 0.030, 0.13, r=0.004, seg=1).move(0, y0 - 0.026 - 0.001, 0)
    return top + sum(bearers) + block


def chess_table_base():
    out = Shape()
    col = [(0.030, 0.150), (0.044, 0.165), (0.044, 0.200), (0.034, 0.215), (0.040, 0.240), (0.052, 0.300), (0.050, 0.360),
           (0.036, 0.420), (0.026, 0.480), (0.022, 0.540), (0.030, 0.560), (0.022, 0.580), (0.024, 0.620), (0.032, 0.635), (0.030, 0.650)]
    out = out + turned(col, seg=20)
    for k in range(3):
        a = math.pi / 2 + TAU * k / 3
        ca, sa = math.cos(a), math.sin(a)
        # a cabriole leg: out of the column's foot, its knee, its ankle, a pad foot
        pts2 = [(0.028, 0.205), (0.060, 0.190), (0.105, 0.158), (0.150, 0.110), (0.190, 0.062), (0.222, 0.030), (0.245, 0.016), (0.262, 0.012)]
        path = []
        for i in range(25):
            t = i / 24
            u = t * (len(pts2) - 1)
            j = min(int(u), len(pts2) - 2)
            f = u - j
            r = pts2[j][0] + (pts2[j + 1][0] - pts2[j][0]) * f
            y = pts2[j][1] + (pts2[j + 1][1] - pts2[j][1]) * f
            path.append((r * ca, y, r * sa))
        rad = lambda t: 0.026 - 0.013 * smoothstep(0.0, 0.85, t) + 0.004 * math.sin(math.pi * min(1.0, t * 2.5))
        leg = sweep(path, rad, seg=10)
        # flattened top to bottom, as a leg is sawn from a plank
        leg = leg.warp(lambda p: Vector((p.x, p.y, p.z)))
        out = out + leg
        out = out + turned([(0.014, 0.0), (0.020, 0.004), (0.021, 0.008), (0.016, 0.013), (0.012, 0.016)], seg=14).move(0.262 * ca, 0, 0.262 * sa)
    return out


def chess_board():
    s = rbox(CH_BOARD, CH_BOARD_T, CH_BOARD, r=0.0025, seg=2).move(0, CH_TOP_Y + CH_BOARD_T / 2, 0)
    # the fold down its middle
    s = cut(s, box(0.0012, 0.0012, CH_BOARD + 0.01).move(0, CH_TOP_Y + CH_BOARD_T, 0))
    # its two hinges let into the edge at the fold
    return s


def chess_board_hinges():
    out = Shape()
    for z in (-0.100, 0.100):
        out = out + box(0.016, 0.0055, 0.0008).move(0, CH_TOP_Y + CH_BOARD_T * 0.5, z + 0.0)
    out = Shape()
    for sz in (-1, 1):
        leaf = rbox(0.022, 0.008, 0.0009, r=0.0003, seg=1).move(0, CH_TOP_Y + CH_BOARD_T * 0.5, sz * (CH_BOARD / 2 + 0.00045))
        knuckle = cyl(0.0012, h=0.008, seg=8, bevel_r=0.0002).move(0, CH_TOP_Y + CH_BOARD_T * 0.5, sz * (CH_BOARD / 2 + 0.0012))
        out = out + leaf + knuckle
    return out


def staunton(prof, seg=16):
    """A chessman turned from its profile [(r, y)] from its foot up, its
    foot on y = 0, a felt-thin base."""
    return turned(prof, seg=seg)


def st_base(r):
    return [(r * 0.96, 0.0), (r, 0.0012), (r, 0.0035), (r * 0.92, 0.0045), (r * 0.94, 0.0058), (r * 0.80, 0.0078)]


def chess_king():
    H = CH_KING
    prof = st_base(0.0130) + [(0.0090, 0.012), (0.0068, 0.020), (0.0058, 0.030), (0.0060, 0.036), (0.0088, 0.0385), (0.0090, 0.0400),
                              (0.0060, 0.0415), (0.0068, 0.0460), (0.0090, 0.0505), (0.0094, 0.0520), (0.0072, 0.0535), (0.0040, 0.0550)]
    body = staunton(prof)
    v = rbox(0.0030, 0.0110, 0.0030, r=0.0007, seg=1).move(0, 0.0605, 0)
    h = rbox(0.0080, 0.0030, 0.0030, r=0.0007, seg=1).move(0, 0.0615, 0)
    neck = cyl(0.0020, h=0.0025, seg=8, bevel_r=0).move(0, 0.0555, 0)
    return body + v + h + neck


def chess_queen():
    prof = st_base(0.0125) + [(0.0085, 0.011), (0.0064, 0.018), (0.0052, 0.027), (0.0056, 0.032), (0.0084, 0.0345), (0.0086, 0.0358),
                              (0.0056, 0.0372), (0.0062, 0.0410), (0.0084, 0.0455), (0.0088, 0.0470), (0.0060, 0.0482), (0.0001, 0.0490)]
    body = staunton(prof)
    # the coronet: eight points round its crown, the ball on top
    pts = [cone(0.0016, 0.0040, seg=6).move(0.0076 * math.cos(TAU * k / 8), 0.0490, 0.0076 * math.sin(TAU * k / 8)) for k in range(8)]
    ball = sphere(0.0034, seg=10, rings=6).move(0, 0.0530, 0)
    return body + sum(pts) + ball


def chess_bishop():
    prof = st_base(0.0120) + [(0.0080, 0.010), (0.0060, 0.016), (0.0046, 0.024), (0.0050, 0.028), (0.0076, 0.0305), (0.0078, 0.0318),
                              (0.0050, 0.0330), (0.0060, 0.0360), (0.0066, 0.0395), (0.0056, 0.0430), (0.0030, 0.0458), (0.0018, 0.0466),
                              (0.0024, 0.0476), (0.0026, 0.0490), (0.0016, 0.0502), (0.0001, 0.0505)]
    body = staunton(prof, seg=16)
    # the mitre's cut
    slot = box(0.016, 0.0016, 0.020).rot(z=0.75).move(0.0030, 0.0405, 0)
    return cut(body, slot)


def chess_rook():
    prof = st_base(0.0125) + [(0.0088, 0.010), (0.0076, 0.016), (0.0074, 0.026), (0.0088, 0.0280), (0.0094, 0.0292), (0.0094, 0.0370),
                              (0.0001, 0.0370)]
    body = staunton(prof, seg=20)
    for k in range(4):
        body = cut(body, box(0.0035, 0.004, 0.024).rot(y=TAU * k / 8).move(0, 0.0370, 0))
    body = cut(body, cyl(0.0066, h=0.006, seg=16, bevel_r=0).move(0, 0.0372, 0))
    return body


def chess_pawn():
    prof = st_base(0.0110) + [(0.0075, 0.009), (0.0052, 0.015), (0.0040, 0.020), (0.0064, 0.0215), (0.0066, 0.0225), (0.0036, 0.0238),
                              (0.0034, 0.0248)]
    body = staunton(prof, seg=14)
    ball = sphere(0.0056, seg=12, rings=7).move(0, 0.0290, 0)
    return body + ball


def chess_knight():
    """The base turned, the horse's head carved: its profile cut from a slab,
    rounded hard, narrower at the muzzle; the mane's ridge."""
    base = staunton(st_base(0.0125) + [(0.0090, 0.0100), (0.0070, 0.0130), (0.0072, 0.0150), (0.0001, 0.0152)], seg=20)
    head = [(-0.0060, 0.0145), (0.0055, 0.0145), (0.0060, 0.0200), (0.0068, 0.0260), (0.0090, 0.0300), (0.0112, 0.0340),
            (0.0108, 0.0372), (0.0080, 0.0388), (0.0040, 0.0400), (0.0018, 0.0425), (0.0000, 0.0445), (-0.0012, 0.0428),
            (-0.0040, 0.0420), (-0.0062, 0.0395), (-0.0072, 0.0350), (-0.0075, 0.0290), (-0.0072, 0.0220)]
    s = prism(head, 0.0085, bevel_r=0.0022, bevel_seg=2)
    # the muzzle narrower than the jowl
    s = s.warp(lambda p: Vector((p.x, p.y, p.z * (1.0 - 0.35 * smoothstep(0.004, 0.011, p.x)))))
    mane = rbox(0.0030, 0.020, 0.0034, r=0.0012, seg=1).rot(z=0.25).move(-0.0062, 0.0330, 0)
    ears = [cone(0.0013, 0.0040, seg=5).rot(z=0.2).move(-0.0006, 0.0448, s2 * 0.0018) for s2 in (-1, 1)]
    return base + s + mane + sum(ears)


def chess_ashtray():
    """A cast brass cigar ashtray, 90 mm: its dish, a rim with two rests."""
    dish = lathe([(0.0, 0.0), (0.040, 0.0), (0.044, 0.003), (0.045, 0.011), (0.0435, 0.0142), (0.0395, 0.0142),
                  (0.0370, 0.0080), (0.0330, 0.0050), (0.0, 0.0046)], seg=40, cap_bottom=False, cap_top=False)
    dish = weld(dish).rot(y=math.pi / 40)
    for a in (0.0, math.pi):
        rest = cyl(0.0075, h=0.02, seg=12).rot(z=math.pi / 2).move(0.042 * math.cos(a), 0.0168, 0.042 * math.sin(a))
        dish = cut(dish, rest)
    return dish


auto('Real__chessTableTop', chess_table_top, 1200)
auto('Real__chessTableBase', chess_table_base, 2600)
auto('Real__chessBoard', chess_board, 400)
auto('Real__chessBoardHinges', chess_board_hinges, 300)
auto('Real__chessKing', chess_king, 720)
auto('Real__chessQueen', chess_queen, 800)
auto('Real__chessBishop', chess_bishop, 820)
auto('Real__chessKnight', chess_knight, 860)
auto('Real__chessRook', chess_rook, 820)
auto('Real__chessPawn', chess_pawn, 560)
auto('Real__chessAshtray', chess_ashtray, 900)
