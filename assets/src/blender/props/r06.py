"""r06: five everyday wall props remodelled at their real sizes (docs/PROPS.md,
P1): the hole kicked through a wall, the rusted pipe run, the weathered
posters, the fire alarm pull station and the coat hook rail. Sizes and
sources: r06.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`; the placers in
`crates/placers/placers_p31/src/wall.rs`). Every part is modelled at its real
size in its own frame: x across its front, y up, z out of its front (out of
the wall it hangs on). Where a part sits on a wall, the wall's face is z = 0
unless said otherwise.

The hole (three sizes: A a fist, B an elbow or a shoulder, C a kick), its
centre at the origin on the wall's face:
holeXMask     the outline of the torn face paper, flat, 2 mm proud: drawn
              first and invisibly (it writes depth only), so the wall is not
              drawn inside it and what is behind shows through
holeXGypsum   the gypsum: bared in a ring where the paper tore back, its
              broken edge through the 12.7 mm board, the chunks still hanging
              on their paper into the stud bay
holeXPaper    the face paper: its torn lip lifted round the hole, its flaps
              and the skins of the hanging chunks pushed in (drawn in the
              wall's own surface)
holeStud      a 3-5/8 in steel stud (92 x 41 mm C, its knockouts), upright,
              its face flange at z = -0.0127 (the board's back)
holeChips     what fell out on the floor: gypsum chips and crumbs, scraps of
              paper (y up from the floor, z out from the wall's foot)

The pipe run, 2 in Schedule 40 steel (OD 60.3 mm) with 150 lb malleable iron
fittings (ASME B16.3), each fitting centred on its axes' meeting point:
pipeElbow     90 degree elbow, a leg along +x and a leg along +y
pipeTee       a tee, its run along x, its branch up +y
pipeCoupling  a coupling along x
pipeUnion     a union along x, its octagonal nut
pipeValve     a 2 in bronze gate valve along x, bonnet and stem up +y
pipeWheel     its cast handwheel, flat in xz, the hub at the origin
pipeClamp     a strut clamp: a 41 x 41 mm channel stub bolted to the wall
              (z = -0.080) and the two-piece clamp round the pipe at the origin
pipeCollar    a split-ring escutcheon where the pipe goes into a wall, the
              pipe along z, its back on the wall at z = 0

The pull station, a single-action die-cast station on its surface back box
(the Fire-Lite BG-12 and SB-10 class: 4.13 x 5.5 in), its back on the wall:
pullBox       the back box, 35 mm deep, the conduit's knockout on top
pullBody      the cover: the handle's well, the glass rod's lugs, the lock
pullHandle    the lever, pivoting at its top edge (y = 0 at the pivot)
pullLetters   FIRE raised over the handle's well
pullHandleText PULL DOWN raised on the lever (moves with it)
pullScrews    the cover's four screws
pullRod       the glass rod across the lever; pullRodBroken its two stubs
pullLock      the reset lock's cylinder
emtConnector  a 1/2 in EMT set-screw connector, its axis +y, base at y = 0
emtStrap      a one-hole strap and its clamp-back spacer, conduit along y,
              the wall at z = 0, the conduit's axis at z = 0.0175
rodShard      a piece of the broken glass rod, lying (y up)

The coat hook rail:
hookRail      an oak rail 700 x 95 x 19 mm, its edges chamfered, the back
              on the wall at z = 0, centred
hookScrew     a countersunk slotted screw head, flush in the rail
coatHook      a cast iron double coat-and-hat hook (140 mm tall, 95 mm out),
              its base plate's back at z = 0, the plate's centre at the origin
coatHung      a wool overcoat hung by its loop, the loop's contact on the
              hook at the origin, the wall at z = -0.077

The posters (their frames 'plane': the print is laid flat across x and y):
posterA2      a 420 x 594 mm sheet, its top right corner curled off the wall
posterTabloid a 279 x 432 mm sheet, its bottom left corner curled
posterLetter  a 216 x 279 mm sheet, its bottom right corner curled
posterTape    a 48 mm strip of masking tape, a little creased
posterClip    a clip frame's spring clip, at the edge of its glass
glassShard    a shard of a frame's broken glass, lying (y up)
"""
from propkit import *
import propkit
import random as _random

# ================================================================ the hole ===
BOARD_T = 0.0127                       # 1/2 in gypsum board
HOLE_SIZES = {'A': (0.056, 0.046), 'B': (0.076, 0.062), 'C': (0.105, 0.080)}  # half width, half height
HARM = [(2, 0.07), (3, 0.09), (4, 0.05), (5, 0.06), (7, 0.04), (9, 0.03)]


def _outline_radii(var, n):
    """The torn outline's radius at n angles (1.0 the ellipse), its own
    harmonics and jags; and the gypsum break's, inside it everywhere."""
    rng = _random.Random(4100 + ord(var))
    phase = [rng.uniform(0, TAU) for _ in HARM]
    base = []
    for i in range(n):
        a = TAU * i / n
        base.append(1.0 + sum(A * math.cos(k * a + p) for (k, A), p in zip(HARM, phase)))
    paper = [b * (1.0 + rng.uniform(-0.06, 0.12)) for b in base]
    gyps = [b * (0.85 + rng.uniform(-0.08, 0.07)) for b in base]
    return paper, gyps


def _ring(rx, ry, radii, z=0.0, scale=1.0):
    n = len(radii)
    return [Vector((rx * r * scale * math.cos(TAU * i / n), ry * r * scale * math.sin(TAU * i / n), z)) for i, r in enumerate(radii)]


def _loft(rings, closed=True):
    """Quads between successive rings of equal length."""
    v, f = [], []
    n = len(rings[0])
    for r in rings:
        v += [p.copy() for p in r]
    for j in range(len(rings) - 1):
        for i in range(n if closed else n - 1):
            i2 = (i + 1) % n
            f.append((j * n + i, j * n + i2, (j + 1) * n + i2, (j + 1) * n + i))
    return Shape(v, f)


def _orient(s, want):
    """Turn every face of an open surface to face `want(centre)` (a vector)."""
    out = []
    for f in s.f:
        a, b, c = s.v[f[0]], s.v[f[1]], s.v[f[2]]
        n = (b - a).cross(c - a)
        if n.length < 1e-12 and len(f) > 3:
            n = (c - a).cross(s.v[f[3]] - a)
        cen = sum((s.v[i] for i in f), Vector()) / len(f)
        out.append(tuple(reversed(f)) if n.dot(want(cen)) < 0 else f)
    s.f = out
    return s


FRONT = lambda c: Vector((0, 0, 1))
BACK = lambda c: Vector((0, 0, -1))
INWARD = lambda c: Vector((-c.x, -c.y, 0))


def hole_mask(var):
    rx, ry = HOLE_SIZES[var]
    paper, _ = _outline_radii(var, 40)     # the same outline the gypsum's torn ring follows
    ring = _ring(rx, ry, paper, 0.002)
    v = [Vector((0, 0, 0.002))] + ring
    f = [(0, i + 1, (i + 1) % len(ring) + 1) for i in range(len(ring))]
    return _orient(Shape(v, f), FRONT)


def _chunk(p0, p1, inward, depth, swing, rng, thick=BOARD_T):
    """A gypsum chunk still hanging on its paper: a slab from the break line
    p0-p1 inward `depth`, swung into the wall by `swing` about that line."""
    t = (p1 - p0).normalized()
    nrm = inward.normalized()
    mid = (p0 + p1) / 2
    w = (p1 - p0).length
    tip = [(-0.5 * w, 0), (0.5 * w, 0), (0.42 * w + rng.uniform(-0.2, 0.2) * w, depth * rng.uniform(0.7, 1.0)),
           (rng.uniform(-0.15, 0.15) * w, depth), (-0.40 * w, depth * rng.uniform(0.55, 0.9))]
    out = Shape()
    pts = []
    for (a, b) in tip:
        pts.append(mid + t * a + nrm * b)
    # the slab: its front face at the paper's plane, its back thick behind it
    n = len(pts)
    v = [p.copy() for p in pts] + [p + Vector((0, 0, -thick)) for p in pts]
    f = [tuple(range(n)), tuple(reversed(range(n, 2 * n)))]
    for i in range(n):
        j = (i + 1) % n
        f.append((i, n + i, n + j, j))
    s = propkit._fix_winding(Shape(v, f))
    # crumble its broken edges a little
    s = noisy(s, 0.0012, scale=180.0, seed=rng.randint(0, 99))
    # swing it in about the break line
    m = Matrix.Translation(mid) @ Matrix.Rotation(-swing, 4, t) @ Matrix.Translation(-mid)
    return s.apply(m), pts, m


def _hole_parts(var):
    rx, ry = HOLE_SIZES[var]
    n = 40
    paper, gyps = _outline_radii(var, n)
    rng = _random.Random(5200 + ord(var))
    P = _ring(rx, ry, paper, -0.0004)
    G0 = _ring(rx, ry, gyps, -0.0004)
    # the broken edge: crumbly, flaring a little toward the back
    G1 = [Vector((p.x * (1.02 + rng.uniform(-0.03, 0.03)), p.y * (1.02 + rng.uniform(-0.03, 0.03)), -0.006)) for p in G0]
    G2 = [Vector((p.x * (1.06 + rng.uniform(-0.04, 0.04)), p.y * (1.06 + rng.uniform(-0.04, 0.04)), -BOARD_T)) for p in G0]
    gyp = _orient(_loft([P, G0]), FRONT)      # the bared ring, facing out
    edge = _orient(_loft([G0, G1, G2]), INWARD)   # the break through the board
    gyp = gyp + edge
    paper_parts = Shape()
    # the chunks: two or three hang on their paper along the bottom and a side
    spans = {'A': [(28, 33, 0.020, 0.75), (8, 12, 0.014, 0.55)],
             'B': [(27, 33, 0.026, 0.85), (6, 11, 0.020, 0.60), (17, 21, 0.016, 0.45)],
             'C': [(26, 34, 0.034, 0.95), (4, 10, 0.026, 0.70), (15, 20, 0.022, 0.50)]}[var]
    for (i0, i1, depth, swing) in spans:
        p0, p1 = G0[i0 % n], G0[i1 % n]
        inward = -((p0 + p1) / 2).copy()
        inward.z = 0
        ch, pts, m = _chunk(p0, p1, inward, depth, swing, rng)
        gyp = gyp + ch
        # its paper skin, a little bigger than the gypsum (the paper tears
        # wider than the board breaks), a hair in front of it
        mid = sum(pts, Vector()) / len(pts)
        skin = [mid + (p - mid) * 1.12 + Vector((0, 0, 0.0004)) for p in pts]
        nn = len(skin)
        sv = [p.copy() for p in skin] + [p + Vector((0, 0, -0.0003)) for p in skin]
        sf = [tuple(range(nn)), tuple(reversed(range(nn, 2 * nn)))]
        for i in range(nn):
            j = (i + 1) % nn
            sf.append((i, nn + i, nn + j, j))
        sk = propkit._fix_winding(Shape(sv, sf)).apply(m)
        paper_parts = paper_parts + sk
    # free flaps of face paper, torn in and curling toward the bay
    flaps = {'A': [(18, 0.016, 1.1), (36, 0.012, 0.9)], 'B': [(15, 0.020, 1.2), (37, 0.016, 1.0), (23, 0.012, 0.8)],
             'C': [(13, 0.026, 1.25), (38, 0.022, 1.05), (22, 0.018, 0.9)]}[var]
    for (i, ln, swing) in flaps:
        p0 = P[i % n]
        p1 = P[(i + 3) % n]
        t = (p1 - p0)
        inward = -((p0 + p1) / 2)
        inward.z = 0
        inward.normalize()
        g_mid = (G0[i % n] + G0[(i + 3) % n]) / 2
        gap = max(0.0, (((p0 + p1) / 2) - g_mid).dot(-inward))
        rows = 6
        v, f = [], []
        for r in range(rows + 1):
            u = r / rows
            d = (gap + ln) * u
            taper = 1.0 - 0.8 * max(0.0, (d - gap) / ln)          # it narrows to a torn tip
            lo_, hi_ = 0.5 * (1 - taper), 1 - 0.5 * (1 - taper)
            # flat over the bared ring, then curling in past the break
            past = max(0.0, d - gap)
            a = swing * (past / ln) ** 0.8
            for s_ in (lo_, hi_):
                base = p0 + t * s_ + inward * min(d, gap)
                p = base + inward * (past * math.cos(a)) + Vector((0, 0, 0.0003 - past * math.sin(a)))
                v.append(p)
        for r in range(rows):
            a0 = r * 2
            f.append((a0, a0 + 1, a0 + 3, a0 + 2))
        sheet = _orient(Shape(v, f), FRONT)
        back = _orient(Shape([p + Vector((0, 0, -0.0003)) for p in v], list(f)), BACK)
        paper_parts = paper_parts + sheet + back
    return gyp, paper_parts


def hole_gypsum(var):
    return _hole_parts(var)[0]


def hole_paper(var):
    return _hole_parts(var)[1]


def hole_stud():
    """A 3-5/8 in steel stud (92 mm web, 41 mm flanges, 6 mm return lips),
    0.5 m of it, upright: its face flange on the board's back, the web
    running back into the wall, a knockout punched in the web."""
    t = 0.0012
    web, fl, lip = 0.092, 0.041, 0.006
    h = 0.5
    z0 = -BOARD_T
    # in plan (x, z): the C opens to +x; the web at x = 0 from z0 back to z0 - web
    parts = [
        box(t * 2, h, web).move(0, 0, z0 - web / 2),                       # the web
        box(fl, h, t * 2).move(fl / 2, 0, z0 - t),                         # the face flange, on the board
        box(fl, h, t * 2).move(fl / 2, 0, z0 - web + t),                   # the back flange
        box(t * 2, h, lip).move(fl, 0, z0 - lip / 2 - t),                  # the return lips
        box(t * 2, h, lip).move(fl, 0, z0 - web + lip / 2 + t),
    ]
    s = sum(parts)
    ko = rbox(0.06, 0.038, 0.03, r=0.018, seg=3).rot(0, math.pi / 2, 0).move(0, 0.10, z0 - web / 2)
    return cut(s, ko)


def hole_chips():
    """What fell on the floor: gypsum chips (white core, a paper face on
    some), crumbs, a fleck or two of paper. x along the wall, z out of it."""
    rng = _random.Random(7311)
    out = Shape()
    for k in range(26):
        sz = rng.uniform(0.005, 0.022) * (1.6 if k < 4 else 1.0)
        c = rbox(sz, sz * rng.uniform(0.3, 0.7), sz * rng.uniform(0.5, 0.9), r=sz * 0.12, seg=1)
        c = noisy(c, sz * 0.12, scale=120.0, seed=k)
        c.rot(rng.uniform(-0.4, 0.4), rng.uniform(0, TAU), rng.uniform(-0.4, 0.4))
        x = rng.gauss(0, 0.07)
        z = abs(rng.gauss(0.0, 0.06)) + 0.025
        lo, hi = c.bounds()
        out = out + c.move(x, -lo.y, z)
    return out


# ================================================================== pipes ===
PIPE_R = 0.03015          # 2 in Sch 40: 60.3 mm
HUB_R = 0.0365            # a 2 in malleable fitting's body
BAND_R = 0.0392           # the bead at its mouth
BAND_W = 0.008
E_A = 0.057               # centre to end, 90 degree elbow and tee (B16.3: 2.25 in)


def _hub_x(x0, x1, band_at_end=True):
    """A fitting's hub along x from x0 to x1, its bead and mouth at x1."""
    L = x1 - x0
    prof = [(HUB_R, 0.0), (HUB_R, L - BAND_W - 0.002), (BAND_R, L - BAND_W), (BAND_R, L - 0.0012),
            (BAND_R - 0.0012, L), (PIPE_R + 0.0006, L), (PIPE_R + 0.0006, L - 0.003)]
    s = lathe(prof, seg=16, cap_bottom=True, cap_top=False)
    return s.rot(z=-math.pi / 2).move(x0, 0, 0)


def pipe_elbow():
    a = _hub_x(-0.004, E_A)
    b = _hub_x(-0.004, E_A).rot(z=math.pi / 2)            # the leg up +y
    corner = sphere(HUB_R, seg=16, rings=8)
    return a + b + corner


def pipe_tee():
    a = _hub_x(-0.004, E_A)
    b = _hub_x(-0.004, E_A).rot(z=math.pi)                # the run's other end, -x
    c = _hub_x(-0.004, E_A).rot(z=math.pi / 2)            # the branch, +y
    return a + b + c


def pipe_coupling():
    L = 0.032
    a = _hub_x(-0.001, L)
    b = _hub_x(-0.001, L).rot(z=math.pi)
    rib = cyl(HUB_R + 0.0012, h=0.004, seg=16, bevel_r=0.0008, bevel_seg=1).rot(z=math.pi / 2)
    return a + b + rib


def pipe_union():
    L = 0.042
    a = _hub_x(-0.001, L)
    b = _hub_x(-0.001, L).rot(z=math.pi)
    # the nut: octagonal, 92 mm across, its ribs
    oct8 = [(0.046 * math.cos(TAU * i / 8 + TAU / 16), 0.046 * math.sin(TAU * i / 8 + TAU / 16)) for i in range(8)]
    nut = prism(oct8, 0.030, bevel_r=0.0015, bevel_seg=1).rot(y=math.pi / 2).move(0.004, 0, 0)
    shoulder = cyl(0.041, h=0.010, seg=16, bevel_r=0.001, bevel_seg=1).rot(z=math.pi / 2).move(-0.016, 0, 0)
    return a + b + nut + shoulder


def pipe_valve():
    """A 2 in bronze gate valve, threaded: 92 mm end to end, hex ends, the
    body's bulge, the bonnet and its union nut, the packing nut, the stem."""
    hexo = [(0.0435 * math.cos(TAU * i / 6), 0.0435 * math.sin(TAU * i / 6)) for i in range(6)]
    ends = []
    for sx in (-1, 1):
        h = prism(hexo, 0.020, bevel_r=0.0012, bevel_seg=1).rot(y=math.pi / 2).move(sx * 0.036, 0, 0)
        ends.append(h)
    body = ellipsoid(0.040, 0.050, 0.036, seg=16).move(0, 0.006, 0)
    neck = cyl(0.030, 0.034, 0.040, seg=16, bevel_r=0.002).move(0, 0.050, 0)
    hexb = [(0.040 * math.cos(TAU * i / 6), 0.040 * math.sin(TAU * i / 6)) for i in range(6)]
    bonnet_nut = prism(hexb, 0.018, bevel_r=0.0012, bevel_seg=1).rot(x=math.pi / 2).move(0, 0.074, 0)
    bonnet = cyl(0.022, 0.026, 0.030, seg=16, bevel_r=0.002).move(0, 0.096, 0)
    hexp = [(0.017 * math.cos(TAU * i / 6), 0.017 * math.sin(TAU * i / 6)) for i in range(6)]
    pack = prism(hexp, 0.014, bevel_r=0.001, bevel_seg=1).rot(x=math.pi / 2).move(0, 0.118, 0)
    stem = cyl(0.006, h=0.032, seg=10, bevel_r=0.0008, bevel_seg=1).move(0, 0.140, 0)
    return sum(ends) + body + neck + bonnet_nut + bonnet + pack + stem


def pipe_wheel():
    """The handwheel: 115 mm, a round rim, four spokes, its hub and nut."""
    rim = torus(0.050, 0.0065, seg=28, tseg=8).rot(x=math.pi / 2)
    spokes = [cyl(0.0045, 0.0060, 0.044, seg=8, bevel_r=0).rot(z=math.pi / 2).move(0.024, 0, 0).rot(y=TAU * k / 4 + 0.3) for k in range(4)]
    hub = cyl(0.013, 0.014, 0.018, seg=14, bevel_r=0.0015)
    hexn = [(0.008 * math.cos(TAU * i / 6), 0.008 * math.sin(TAU * i / 6)) for i in range(6)]
    nut = prism(hexn, 0.007, bevel_r=0.0006, bevel_seg=1).rot(x=math.pi / 2).move(0, 0.0125, 0)
    return rim + sum(spokes) + hub + nut


def pipe_clamp():
    """A 41 x 41 mm strut channel stub (160 mm, its lips facing out) bolted to
    the wall, and a two-piece strut clamp round the pipe."""
    t = 0.0026
    z_wall = -0.080
    ch_h = 0.16
    zc = z_wall + 0.0205
    back = box(0.041, ch_h, t).move(0, 0, z_wall + t / 2)
    sides = [box(t, ch_h, 0.041).move(sx * (0.0205 - t / 2), 0, zc) for sx in (-1, 1)]
    lips = [box(0.009, ch_h, t).move(sx * (0.0205 - 0.0045), 0, z_wall + 0.041 - t / 2) for sx in (-1, 1)]
    curls = [box(t, ch_h, 0.007).move(sx * (0.0205 - 0.009), 0, z_wall + 0.041 - 0.0035) for sx in (-1, 1)]
    channel = back + sum(sides) + sum(lips) + curls[0] + curls[1]
    channel = cut(channel, *[rbox(0.012, 0.018, 0.01, r=0.005, seg=2).move(0, y, z_wall) for y in (-0.05, 0.05)])
    # the anchors' hex heads in the channel's floor
    hexa = [(0.0095 * math.cos(TAU * i / 6), 0.0095 * math.sin(TAU * i / 6)) for i in range(6)]
    anchors = [prism(hexa, 0.007, bevel_r=0.0006, bevel_seg=1).move(0, y, z_wall + t + 0.0035) for y in (-0.05, 0.05)]
    washers = [cyl(0.013, h=0.0015, seg=12, bevel_r=0).rot(x=math.pi / 2).move(0, y, z_wall + t + 0.0007) for y in (-0.05, 0.05)]
    # the clamp: a 25 mm strap round the pipe, its ears down into the channel
    sw, st = 0.025, 0.0022
    R = PIPE_R + st / 2 + 0.0004
    pts = []
    for i in range(25):
        a = -math.pi / 2 - 0.05 + (math.pi + 0.1) * i / 24      # from below, round the front, to the top
        pts.append(Vector((0, R * math.sin(a), R * math.cos(a))))
    top = pts[-1].copy()
    bot = pts[0].copy()
    ear_z = z_wall + 0.041 + 0.002
    pts = [Vector((0, bot.y, ear_z)), Vector((0, bot.y, ear_z + 0.004))] + pts + [Vector((0, top.y, ear_z + 0.004)), Vector((0, top.y, ear_z))]
    strap = _strip(pts, sw, st)
    # the back half under the pipe (between the pipe and the channel)
    back_half = []
    for i in range(13):
        a = math.pi / 2 + math.pi * i / 12
        back_half.append(Vector((0, R * math.sin(a), R * math.cos(a))))
    strap2 = _strip(back_half, sw, st)
    hexb = [(0.0065 * math.cos(TAU * i / 6), 0.0065 * math.sin(TAU * i / 6)) for i in range(6)]
    bolts = [prism(hexb, 0.006, bevel_r=0.0005, bevel_seg=1).rot(x=math.pi / 2).move(0, sy * (R + 0.012), ear_z + 0.004) for sy in (-1, 1)]
    ears = [box(sw, 0.022, st).move(0, sy * (R + 0.009), ear_z + st / 2) for sy in (-1, 1)]
    return channel + sum(anchors) + sum(washers) + strap + strap2 + sum(bolts) + sum(ears)


def _strip(pts, w, t):
    """A flat strap w wide (along x), t thick, along a polyline in the yz plane."""
    n = len(pts)
    v, f = [], []
    for i, p in enumerate(pts):
        d = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)])
        d.normalize()
        nrm = Vector((0, -d.z, d.y))        # in-plane normal
        for sx in (-0.5, 0.5):
            for sn in (-0.5, 0.5):
                v.append(p + Vector((sx * w, 0, 0)) + nrm * (sn * t))
    for i in range(n - 1):
        a, b = i * 4, (i + 1) * 4
        # four sides: (x-,n-)=0 (x-,n+)=1 (x+,n-)=2 (x+,n+)=3
        for (p, q) in ((0, 1), (1, 3), (3, 2), (2, 0)):
            f.append((a + p, a + q, b + q, b + p))
    f.append((0, 1, 3, 2))
    e = (n - 1) * 4
    f.append((e + 0, e + 2, e + 3, e + 1))
    return propkit._fix_winding(Shape(v, f))


def pipe_collar():
    """A split-ring escutcheon: 115 mm, domed, round the pipe, on the wall."""
    prof = [(PIPE_R + 0.0008, 0.0), (0.0575, 0.0), (0.0575, 0.0025), (0.052, 0.006), (0.040, 0.010),
            (PIPE_R + 0.004, 0.012), (PIPE_R + 0.0008, 0.012)]
    s = lathe(prof, seg=28, cap_bottom=False, cap_top=False)
    s = propkit._fix_winding(s)
    split = box(0.0015, 0.03, 0.03).move(0, 0.045, 0.006)
    s = cut(s.rot(x=math.pi / 2), split)
    return s


# ========================================================== the pull station ===
PB_W, PB_H, PB_D = 0.105, 0.140, 0.035     # SB-10 back box: 4.13 x 5.5 x 1.37 in
PC_D = 0.030                               # the cover's depth in front of it
WELL = (0.080, 0.040, 0.004)               # the handle's well: w, h, centre y


def pull_box():
    s = rbox(PB_W, PB_H, PB_D, r=0.006, seg=2).move(0, 0, PB_D / 2)
    # the knockout on top (the conduit's), and a seam where the cover meets it
    ko = cyl(0.011, h=0.01, seg=16, bevel_r=0).move(0, PB_H / 2, PB_D / 2)
    seam = box(PB_W + 0.01, PB_H + 0.01, 0.0012).move(0, 0, PB_D - 0.0006)
    return cut(s, ko, seam)


def _text(body, size, depth, offset=0.0, align='CENTER'):
    """Raised letters (Blender's own face), lying in xy facing +z, their
    back at z = 0."""
    cu = bpy.data.curves.new('txt', 'FONT')
    cu.body = body
    cu.size = size
    cu.extrude = depth / 2
    cu.offset = offset
    cu.resolution_u = 2
    cu.align_x = align
    cu.align_y = 'CENTER'
    cu.fill_mode = 'BOTH'
    ob = bpy.data.objects.new('txt', cu)
    bpy.context.scene.collection.objects.link(ob)
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
    bm = bmesh.new()
    bm.from_mesh(me)
    s = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    bpy.data.meshes.remove(me)
    bpy.data.objects.remove(ob)
    bpy.data.curves.remove(cu)
    return s.move(0, 0, depth / 2)


def pull_body():
    """The die-cast cover: its front domed a little at the edges, the well
    the lever sits in, the two lugs that hold the glass rod, the lock's boss."""
    s = rbox(PB_W, PB_H, PC_D, r=0.008, seg=3).move(0, 0, PB_D + PC_D / 2)
    ww, wh, wy = WELL
    well = rbox(ww, wh, 0.03, r=0.004, seg=2).move(0, wy, PB_D + PC_D)
    s = cut(s, well)
    front = PB_D + PC_D
    lugs = [rbox(0.009, 0.012, 0.010, r=0.002, seg=1).move(sx * (ww / 2 + 0.002), wy - 0.002, front + 0.004) for sx in (-1, 1)]
    boss = cyl(0.0085, h=0.004, seg=16, bevel_r=0.001, bevel_seg=1).rot(x=math.pi / 2).move(0.030, -0.050, front + 0.0015)
    return s + lugs[0] + lugs[1] + boss


def pull_letters():
    front = PB_D + PC_D
    t = _text('FIRE', 0.024, 0.0012, offset=0.00045)
    return t.move(0, 0.046, front - 0.0002)


def pull_handle():
    """The lever: 70 x 30 mm, its grip edge thick at the bottom, pivoting on
    its top edge (the origin); its front flush with the cover's."""
    ww, wh, _ = WELL
    s = rbox(ww - 0.008, wh - 0.008, 0.012, r=0.003, seg=2).move(0, -(wh - 0.008) / 2, -0.006)
    grip = rbox(ww - 0.008, 0.008, 0.016, r=0.003, seg=2).move(0, -(wh - 0.008) + 0.004, -0.004)
    pin = cyl(0.002, h=ww - 0.004, seg=8, bevel_r=0).rot(z=math.pi / 2).move(0, -0.001, -0.008)
    return s + grip + pin


def pull_handle_text():
    t = _text('PULL DOWN', 0.0085, 0.0008, offset=0.00022)
    return t.move(0, -0.013, 0.0)


def pull_screws():
    out = Shape()
    front = PB_D + PC_D
    for (x, y) in ((-0.040, 0.060), (0.040, 0.060), (-0.040, -0.060), (0.040, -0.060)):
        h = screw_head(0.0032, 0.0016).rot(x=math.pi / 2).move(x, y, front - 0.0012)
        out = out + h
    return out


def pull_rod():
    return cyl(0.0025, h=0.090, seg=10, bevel_r=0.0008, bevel_seg=1).rot(z=math.pi / 2)


def pull_rod_broken():
    """What is left of the rod in its lugs: two stubs, their ends snapped."""
    out = Shape()
    for sx in (-1, 1):
        s = cyl(0.0025, h=0.017, seg=10, bevel_r=0.0005, bevel_seg=1).rot(z=math.pi / 2)
        s = noisy(s, 0.0006, scale=600, seed=3 + sx)
        out = out + s.move(sx * 0.0365, 0, 0)
    return out


def pull_lock():
    barrel = cyl(0.0065, h=0.006, seg=16, bevel_r=0.0012, bevel_seg=1).rot(x=math.pi / 2)
    key = box(0.0012, 0.006, 0.01).move(0, 0, 0.003)
    return cut(barrel, key)


def emt_connector():
    """1/2 in EMT set-screw connector: the locknut in the box, the body, the
    set screw (its axis +y, base at y = 0)."""
    oct8 = [(0.0125 * math.cos(TAU * i / 8), 0.0125 * math.sin(TAU * i / 8)) for i in range(8)]
    nut = prism(oct8, 0.004, bevel_r=0.0006, bevel_seg=1).rot(x=-math.pi / 2).move(0, 0.002, 0)
    hexs = [(0.0125 * math.cos(TAU * i / 6), 0.0125 * math.sin(TAU * i / 6)) for i in range(6)]
    hexn = prism(hexs, 0.006, bevel_r=0.0006, bevel_seg=1).rot(x=-math.pi / 2).move(0, 0.007, 0)
    body = cyl(0.0108, 0.0112, 0.022, seg=16, bevel_r=0.001).move(0, 0.021, 0)
    boss = cyl(0.004, h=0.007, seg=8, bevel_r=0.0006, bevel_seg=1).rot(x=math.pi / 2).move(0, 0.024, 0.0125)
    screw = screw_head(0.0028, 0.0018).rot(x=math.pi / 2).move(0, 0.024, 0.0155)
    return nut + hexn + body + boss + screw


def emt_strap():
    """A one-hole strap over 1/2 in EMT (OD 17.9 mm) on a clamp-back spacer."""
    r = 0.00895 + 0.0012
    zc = 0.0175
    pts = []
    for i in range(17):
        a = math.pi * i / 16
        pts.append(Vector((r * math.cos(a), 0, zc + r * math.sin(a))))
    # the strap runs across x here: build in xz then lay as a band 13 mm tall (y)
    w, t = 0.013, 0.0015
    v, f = [], []
    path = [Vector((-r, 0, zc - 0.0085))] + pts[::-1] + [Vector((r, 0, zc - 0.0085)), Vector((r + 0.022, 0, zc - 0.0085))]
    path = path[::-1]
    n = len(path)
    for i, p in enumerate(path):
        d = (path[min(i + 1, n - 1)] - path[max(i - 1, 0)]).normalized()
        nrm = Vector((-d.z, 0, d.x))
        for sy in (-0.5, 0.5):
            for sn in (-0.5, 0.5):
                v.append(p + Vector((0, sy * w, 0)) + nrm * (sn * t))
    for i in range(n - 1):
        a, b = i * 4, (i + 1) * 4
        for (p, q) in ((0, 1), (1, 3), (3, 2), (2, 0)):
            f.append((a + p, a + q, b + q, b + p))
    f.append((0, 1, 3, 2))
    e = (n - 1) * 4
    f.append((e, e + 2, e + 3, e + 1))
    strap = propkit._fix_winding(Shape(v, f))
    spacer = rbox(0.050, 0.016, 0.0085, r=0.0015, seg=1).move(0.006, 0, 0.00425)
    screw = screw_head(0.0034, 0.0018).rot(x=math.pi / 2).move(r + 0.012, 0, zc - 0.0085 + t)
    return strap + spacer + screw


def rod_shard():
    s = cyl(0.0025, h=0.020, seg=8, bevel_r=0).rot(z=math.pi / 2)
    s = noisy(s, 0.0007, scale=500, seed=11)
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


# ========================================================== the coat hooks ===
RAIL = (0.700, 0.095, 0.019)


def hook_rail():
    w, h, d = RAIL
    s = box(w, h, d).move(0, 0, d / 2)
    # chamfer the front edges (a router's 45, 5 mm), round the rest a little
    s = bevel(s, 0.0012, seg=1)
    cutters = []
    for sy in (-1, 1):
        c = box(w + 0.02, 0.010, 0.010).rot(x=math.pi / 4).move(0, sy * h / 2, d)
        cutters.append(c)
    for sx in (-1, 1):
        c = box(0.010, h + 0.02, 0.010).rot(y=math.pi / 4).move(sx * w / 2, 0, d)
        cutters.append(c)
    s = cut(s, *cutters)
    # the screws' countersinks
    sinks = [cone(0.0062, 0.006, seg=12).rot(x=-math.pi / 2).move(sx * 0.335, 0.0, d + 0.0008) for sx in (-1, 1)]
    return cut(s, *sinks)


def hook_screw():
    s = cyl(0.0050, 0.0050, 0.0016, seg=10, bevel_r=0.0004, bevel_seg=1)
    s = cut(s, box(0.012, 0.0012, 0.0009).move(0, 0.0008, 0))
    return s.rot(x=math.pi / 2)


def coat_hook():
    """A cast iron double hook: the base plate (26 x 56 mm) with its two
    screws; the coat hook, out and down and curling up to a knob; the hat
    prong above it, longer, up at 35 degrees to a ball."""
    plate = rbox(0.026, 0.056, 0.006, r=0.0022, seg=2).move(0, 0, 0.003)
    screws = [screw_head(0.0034, 0.0014).rot(x=math.pi / 2).move(0, y, 0.006) for y in (-0.021, 0.021)]
    # the coat hook: a tapering arm out of the plate's lower half
    coat = [Vector((0, -0.012, 0.004)), Vector((0, -0.020, 0.020)), Vector((0, -0.032, 0.034)),
            Vector((0, -0.045, 0.046)), Vector((0, -0.052, 0.058)), Vector((0, -0.050, 0.068)),
            Vector((0, -0.040, 0.074)), Vector((0, -0.030, 0.074))]
    arm1 = tube(coat, 0.0062, seg=8, caps=True, smooth=1, r_end=0.0048)
    knob1 = sphere(0.0058, seg=10, rings=5).move(0, -0.029, 0.073)
    # the hat prong
    hat = [Vector((0, 0.004, 0.004)), Vector((0, 0.012, 0.022)), Vector((0, 0.026, 0.046)),
           Vector((0, 0.042, 0.070)), Vector((0, 0.054, 0.088))]
    arm2 = tube(hat, 0.0060, seg=8, caps=True, smooth=1, r_end=0.0046)
    knob2 = sphere(0.0068, seg=10, rings=5).move(0, 0.056, 0.091)
    s = plate + sum(screws) + arm1 + knob1 + arm2 + knob2
    return s


COAT_Z_WALL = -0.077
COAT_L = 0.98
# the profile down the coat: (y, half width, depth, fold amplitude)
COAT_PROF = [(-0.020, 0.030, 0.030, 0.0), (-0.050, 0.075, 0.058, 0.002), (-0.090, 0.155, 0.092, 0.006),
             (-0.140, 0.208, 0.118, 0.012), (-0.220, 0.218, 0.128, 0.016), (-0.450, 0.228, 0.142, 0.022),
             (-0.750, 0.248, 0.156, 0.027), (-COAT_L, 0.268, 0.166, 0.031)]
COAT_PHASE = [1.7, 4.1, 0.6]


def _coat_at(y):
    for (y0, w0, d0, a0), (y1, w1, d1, a1) in zip(COAT_PROF, COAT_PROF[1:]):
        if y1 <= y <= y0:
            t = (y0 - y) / (y0 - y1)
            t = t * t * (3 - 2 * t)
            return w0 + (w1 - w0) * t, d0 + (d1 - d0) * t, a0 + (a1 - a0) * t
    return COAT_PROF[-1][1:]


def _coat_surf(y, a):
    """The coat's surface at height y, angle a round it (pi/2 its front)."""
    hw, d, amp = _coat_at(y)
    back = COAT_Z_WALL + 0.004                      # the back lies against the wall
    top_pull = max(0.0, 1.0 - (-y - 0.02) / 0.22)   # 1 at the collar: drawn out to the hook
    zc = (back + d / 2) * (1 - top_pull)
    ca, sa = math.cos(a), math.sin(a)
    k = 1.0 if sa >= 0 else 0.55 + 0.45 * (1 - top_pull)    # flatter at the back
    x = hw * ca
    z = zc + d / 2 * sa * k
    # long folds down the front and sides, crossing a little, deeper to the hem
    ph = COAT_PHASE
    fold = amp * (math.sin(a * 7 + ph[0] + y * 3.0) * 0.65 + math.sin(a * 11 + ph[1] - y * 5.0) * 0.35)
    if sa > -0.2:
        x += fold * ca * 0.6
        z += fold * max(sa, 0.0) * 1.2
    # the front: the right panel lapped over the left; the lapels rolled back
    # in a V from the collar to the top button
    if 1.30 < a < 1.62:
        z += 0.005 * (1 - top_pull)
    if -0.36 < y < -0.06:
        t = (-y - 0.06) / 0.30                      # 0 at the collar, 1 at the button
        for c in (1.57 - 0.55 * (1 - t), 1.57 + 0.55 * (1 - t)):
            g = math.exp(-((a - c) / 0.07) ** 2)
            z += 0.007 * g * (1 - t * 0.6)
    return Vector((x, y, z))


def coat_hung():
    """A wool overcoat hung by the loop in its collar: the loop over the hook,
    the collar rolled under it, the shoulders fallen, the lapels, the body
    hanging against the wall in long folds, its pocket flaps, the sleeves
    down its sides with their cuffs. Its loop's contact at the origin; the
    wall at z = -0.077; 0.98 m from the loop to the hem."""
    cols, rows = 28, 26
    v, f = [], []
    for j in range(rows + 1):
        u = j / rows
        y = COAT_PROF[0][0] + (COAT_PROF[-1][0] - COAT_PROF[0][0]) * (u ** 1.15)
        for i in range(cols):
            v.append(_coat_surf(y, TAU * i / cols))
    for j in range(rows):
        for i in range(cols):
            i2 = (i + 1) % cols
            f.append((j * cols + i, j * cols + i2, (j + 1) * cols + i2, (j + 1) * cols + i))
    body = Shape(v, f)
    # the hem turned up inside: a facing so the hem has a thickness
    last = rows * cols
    hem = [Vector((p.x * 0.96, p.y + 0.05, p.z * 0.92 + COAT_Z_WALL * 0.08)) for p in v[last:last + cols]]
    nb = len(body.v)
    body.v += hem
    for i in range(cols):
        i2 = (i + 1) % cols
        body.f.append((last + i, last + i2, nb + i2, nb + i))
    # the top closed over the loop
    body.v.append(Vector((0, -0.012, 0.0)))
    tip = len(body.v) - 1
    for i in range(cols):
        body.f.append((tip, (i + 1) % cols, i))
    body = propkit._fix_winding(body)
    # the collar, rolled and gathered under the loop
    collar = torus(0.045, 0.013, seg=16, tseg=8).rot(x=math.pi / 2).scale(1.0, 1.0, 0.75).move(0, -0.040, 0.0)
    # the pocket flaps, at the hips either side of the front
    flaps = Shape()
    for a in (1.57 - 0.72, 1.57 + 0.72):
        y = -0.66
        p = _coat_surf(y, a)
        q = _coat_surf(y, a + 0.05)
        t = (q - p).normalized()
        nrm = t.cross(Vector((0, 1, 0))).normalized()
        if nrm.z < 0:
            nrm = -nrm
        flap = rbox(0.165, 0.055, 0.005, r=0.002, seg=1)
        m = Matrix((t, Vector((0, 1, 0)), nrm)).transposed().to_4x4()
        flap.apply(m).move(p.x + nrm.x * 0.003, p.y - 0.010, p.z + nrm.z * 0.003)
        flaps = flaps + flap
    # the sleeves, fallen down the sides, a little forward, their cuffs
    sleeves = Shape()
    for sx in (-1, 1):
        z0 = COAT_Z_WALL
        pts = [Vector((sx * 0.150, -0.175, z0 + 0.075)), Vector((sx * 0.205, -0.240, z0 + 0.100)),
               Vector((sx * 0.222, -0.420, z0 + 0.112)), Vector((sx * 0.212, -0.600, z0 + 0.118)),
               Vector((sx * 0.205, -0.670, z0 + 0.120))]
        sl = tube(pts, 0.050, seg=12, caps=True, smooth=2, r_end=0.045)
        sl.warp(lambda p, sx=sx: Vector((sx * 0.170 + (p.x - sx * 0.170) * 0.6 if p.y < -0.2 else p.x, p.y, p.z)))
        cuff = cyl(0.047, 0.047, 0.07, seg=12, bevel_r=0.004, bevel_seg=1).scale(0.62, 1, 1).move(sx * 0.203, -0.640, z0 + 0.120)
        sleeves = sleeves + sl + cuff
    loop = torus(0.010, 0.0022, seg=12, tseg=6).move(0, -0.002, 0)
    s = body + collar + flaps + sleeves + loop
    return noisy(s, 0.0022, scale=9.0, seed=5)


def coat_buttons():
    """Its four horn buttons down the front, 22 mm, and their thread."""
    out = Shape()
    for y in (-0.37, -0.49, -0.61, -0.73):
        a = 1.50
        p = _coat_surf(y, a)
        b = lathe([(0, 0), (0.011, 0), (0.0112, 0.0018), (0.0095, 0.0036), (0.0075, 0.0026), (0, 0.0029)], seg=12).rot(x=math.pi / 2)
        out = out + b.move(p.x, p.y, p.z + 0.003)
    return out


# ============================================================== the posters ===
def _sheet(w, h, nx, ny, curl_corner, curl_r, curl_len, seed, back_only=False):
    """A sheet of paper, cockled, one corner rolled off the wall. Its back
    at z = 0; corners 0 bottom left, 1 bottom right, 2 top right, 3 top left."""
    s = grid_plane(w, h, nx, ny)
    cx = (-w / 2, w / 2, w / 2, -w / 2)[curl_corner]
    cy = (-h / 2, -h / 2, h / 2, h / 2)[curl_corner]
    corner = Vector((cx, cy, 0))
    diag = Vector((-math.copysign(1, cx), -math.copysign(1, cy), 0)).normalized()   # from the corner inward
    rng = _random.Random(seed)
    ph = [rng.uniform(0, 50) for _ in range(3)]
    out = []
    for p in s.v:
        q = p.copy()
        # cockle: the paper's waves from the damp
        q.z = 0.0006 + 0.0012 * (noise.noise(Vector((p.x * 7 + ph[0], p.y * 7 + ph[1], ph[2]))) + 0.5)
        # the roll: past the fold line, wound round a cylinder of radius curl_r
        d = (p - corner).dot(diag)            # distance from the corner along the diagonal
        s_ = curl_len - d                     # how far past the fold line toward the corner
        if s_ > 0:
            a = s_ / curl_r
            a = min(a, 2.2)
            along = curl_r * math.sin(a)
            lift = curl_r * (1 - math.cos(a))
            base = p + diag * s_              # the point on the fold line
            q = base - diag * along
            q.z = lift + 0.0015
        out.append(q)
    s.v = out
    if back_only:
        # the sheet's blank back, a hair behind its face (seen where it curls)
        return Shape([p + Vector((0, 0, -0.0002)) for p in out], [tuple(reversed(x)) for x in s.f])
    return s


def poster_tape():
    s = grid_plane(0.048, 0.019, 6, 2)
    s.warp(lambda p: Vector((p.x, p.y, 0.0003 + 0.00025 * math.sin(p.x * 260.0 + p.y * 90.0))))
    back = Shape([p + Vector((0, 0, -0.0002)) for p in s.v], [tuple(reversed(x)) for x in s.f])
    return s + back


def poster_clip():
    """A clip frame's spring clip: a steel tongue over the glass edge."""
    t = 0.0008
    pts = [Vector((0, 0.0, -0.006)), Vector((0, 0.0, 0.006)), Vector((0, -0.012, 0.0072)), Vector((0, -0.020, 0.0066))]
    s = _strip(pts, 0.016, t)
    return s


def glass_shard():
    rng = _random.Random(9021)
    pts = [(0.0, 0.0), (0.024, 0.004), (0.030, 0.014), (0.012, 0.022), (-0.004, 0.012)]
    pts = [(x + rng.uniform(-0.002, 0.002), y + rng.uniform(-0.002, 0.002)) for (x, y) in pts]
    s = prism(pts, 0.003).rot(x=-math.pi / 2)
    lo, hi = s.bounds()
    return s.move(-(lo.x + hi.x) / 2, -lo.y, -(lo.z + hi.z) / 2)


# ================================================================== parts ===
def auto(name, build, budget, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget)


for _v in 'ABC':
    auto(f'Real__hole{_v}Mask', lambda v=_v: hole_mask(v), 120)
    auto(f'Real__hole{_v}Gypsum', lambda v=_v: hole_gypsum(v), 900)
    auto(f'Real__hole{_v}Paper', lambda v=_v: hole_paper(v), 700)
auto('Real__holeStud', hole_stud, 240)
auto('Real__holeChips', hole_chips, 1200)

auto('Real__pipeElbow', pipe_elbow, 700)
auto('Real__pipeTee', pipe_tee, 700)
auto('Real__pipeCoupling', pipe_coupling, 560)
auto('Real__pipeUnion', pipe_union, 660)
auto('Real__pipeValve', pipe_valve, 1000)
auto('Real__pipeWheel', pipe_wheel, 900)
auto('Real__pipeClamp', pipe_clamp, 1000)
auto('Real__pipeCollar', pipe_collar, 400)

auto('Real__pullBox', pull_box, 500)
auto('Real__pullBody', pull_body, 900)
auto('Real__pullLetters', pull_letters, 500)
auto('Real__pullHandle', pull_handle, 400)
auto('Real__pullHandleText', pull_handle_text, 1400)
auto('Real__pullScrews', pull_screws, 400)
auto('Real__pullRod', pull_rod, 120)
auto('Real__pullRodBroken', pull_rod_broken, 200)
auto('Real__pullLock', pull_lock, 200)
auto('Real__emtConnector', emt_connector, 520)
auto('Real__emtStrap', emt_strap, 400)
auto('Real__rodShard', rod_shard, 100)

auto('Real__hookRail', hook_rail, 500)
auto('Real__hookScrew', hook_screw, 130)
auto('Real__coatHook', coat_hook, 900)
auto('Real__coatHung', coat_hung, 3200)
auto('Real__hookCoatButtons', coat_buttons, 400)

for _nm, _args in (('A2', (0.420, 0.594, 14, 20, 2, 0.022, 0.085, 1)), ('Tabloid', (0.279, 0.432, 10, 16, 0, 0.016, 0.060, 2)),
                   ('Letter', (0.216, 0.279, 8, 12, 1, 0.012, 0.045, 3))):
    auto(f'Real__poster{_nm}', lambda a=_args: _sheet(*a), 700, 'plane')
    auto(f'Real__poster{_nm}Back', lambda a=_args: _sheet(*a, back_only=True), 700, 'plane')
auto('Real__posterTape', poster_tape, 60)
auto('Real__posterClip', poster_clip, 60)
auto('Real__glassShard', glass_shard, 60)
