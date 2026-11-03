"""r15: paper and small things dropped on the office floors, remodelled at
their real sizes (docs/PROPS.md): a flat box of facial tissues, a part-used
roll of PVC insulating tape, a pack of playing cards, an open paperback, a
stack of folded newspapers, a paper dart, a party napkin. (The crumpled
receipt is r08's balled paper, placed smaller; the closed paperback is
r02's.) Sources and sizes: r15.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its
own frame: x across its front, y up from the floor (y = 0 the floor it
rests on), z out of its front. The game maps a part's texture by its box
(each face direction takes the whole map over the part's bounds), so a
thing whose faces carry different print is cut into parts by face: the
tissue box's top, its long sides and its ends; a card and a deck's edges;
a newspaper's front page, its body and its open edges.

tissueBoxTop     the carton's top panel, its dispensing oval torn out along
                 the perforation, one corner crushed down
tissueBoxSides   its two long sides and bottom, the corners' folds
tissueBoxEnds    its two ends: the end panel, the outer flap glued over it
tissueBoxInside  the board's inside, seen through the oval
tissueBoxStack   the tissues still in it, a little over half full
tissueBoxSheet   the tissue that stood up out of the slot, gone limp:
                 pleated where the oval gathers it, leaning, one corner
                 fallen onto the top, the other curled back
tapeRollFaces    a roll of 19 mm PVC tape half used, lying flat: its two
                 faces (the wound edges), telescoped a little
tapeRollWrap     its outer wrap, the edges rounded
tapeRollCore     the 38 mm (1.5 in) paperboard core, its ends frayed
tapeRollTail     the loose end peeled off and standing out, curled, its
                 end stretched to a point where it was torn
cardSingle       a poker-size playing card (63.5 x 88.9 mm, 0.3 mm), its
                 corners rounded
cardDeck         the rest of the deck under the loose cards: 46 cards,
                 not quite squared
pbOpenCovers     a trade paperback (r02's, 216 x 140 mm) lying open face
                 up: its covers flat, the spine round under the gutter
pbOpenBlock      its two halves of pages: the edges, swelling at the
                 fore-edge, the gutter
pbOpenSpread     the two printed pages on top, dipping into the gutter, the
                 right-hand leaves lifting at the fore-edge
newsBody         a Berliner newspaper (315 x 470 mm) folded once (315 x
                 235 mm): the fold, the back
newsEdges        its open edges: the leaves, not quite squared
newsFront        the top half of its front page, on top
newsFrontCurl    the same, its open corner curling up (the top paper's)
paperPlane       a dart folded from a sheet of A4 (297 mm long, the wings
                 61 mm each, the keel 44 mm at the tail), crashed: the nose
                 crumpled, the wings drooping, a trailing edge curled;
                 resting on its keel and a wing tip
napkinCrumpled   a 33 cm two-ply luncheon napkin folded in four (165 mm),
                 used and dropped: soft folds, a flap half open
napkinWad        the same scrunched into a loose wad
"""
from propkit import *
import propkit
import random

# ------------------------------------------------------------------ util --
TAU = math.pi * 2


def _fix(s):
    return propkit._fix_winding(s)


def smooth01(t):
    t = min(1.0, max(0.0, t))
    return t * t * (3 - 2 * t)


def grid(fn, nu, nv):
    """An open grid of quads from fn(u, v) -> Vector, u and v over [0, 1]."""
    v, f = [], []
    for j in range(nv + 1):
        for i in range(nu + 1):
            v.append(Vector(fn(i / nu, j / nv)))
    for j in range(nv):
        for i in range(nu):
            a = j * (nu + 1) + i
            f.append((a, a + 1, a + nu + 2, a + nu + 1))
    return Shape(v, f)


def facing(s, d):
    """Turn every face of an open surface to face direction d (by its
    average normal), so a single-sided surface shows from where it should."""
    d = Vector(d)
    out = Shape([p.copy() for p in s.v], [], set())
    for fc in s.f:
        a, b, c = s.v[fc[0]], s.v[fc[1]], s.v[fc[2]]
        n = (b - a).cross(c - a)
        out.f.append(fc if n.dot(d) >= 0 else tuple(reversed(fc)))
    return out


def facing_fn(s, fn):
    """Like `facing`, the direction a function of the face's centre."""
    out = Shape([p.copy() for p in s.v], [], set())
    for fc in s.f:
        a, b, c = s.v[fc[0]], s.v[fc[1]], s.v[fc[2]]
        n = (b - a).cross(c - a)
        ctr = sum((s.v[i] for i in fc), Vector()) / len(fc)
        out.f.append(fc if n.dot(Vector(fn(ctr))) >= 0 else tuple(reversed(fc)))
    return out


def on_floor(s, y0=0.0):
    lo, hi = s.bounds()
    return s.move(0, y0 - lo.y, 0)


def chaikin(pts, k=2):
    pts = [Vector(p) for p in pts]
    for _ in range(k):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    return pts


def along(pts, t):
    """The point at fraction t of the polyline's length, and its tangent."""
    seg = [(a - b).length for a, b in zip(pts[1:], pts)]
    L = sum(seg)
    d = t * L
    for i, s in enumerate(seg):
        if d <= s or i == len(seg) - 1:
            k = 0.0 if s < 1e-12 else min(1.0, d / s)
            tang = (pts[i + 1] - pts[i])
            return pts[i].lerp(pts[i + 1], k), (tang.normalized() if tang.length > 1e-12 else Vector((0, 1, 0)))
        d -= s
    return pts[-1], Vector((0, 1, 0))


# ============================================================ tissue box ===
# A flat box of facial tissues (Kleenex Trusted Care 160 class: 8.875 x
# 4.75 x 3.4675 in, 225 x 121 x 88 mm), folding-box board 0.6 mm; the
# dispensing oval 112 x 42 mm.
TB_L, TB_H, TB_D = 0.225, 0.088, 0.121
TB_T = 0.0006
TB_C = 0.0012                    # the vertical folds, chamfered
TB_HOLE = (0.056, 0.021)         # the oval's half-axes
TB_FILL = 0.050                  # the tissues' top: a little over half used


def tb_crush(p):
    """A top corner (+x, +z) trodden on: pressed down and in, softly."""
    cx, cz = TB_L / 2, TB_D / 2
    d = math.hypot(p.x - cx, p.z - cz)
    w = max(0.0, 1 - d / 0.060) ** 2
    h = smooth01((p.y - (TB_H - 0.034)) / 0.034)
    k = w * h
    return Vector((p.x - 0.0035 * k, p.y - 0.0085 * k, p.z - 0.0025 * k))


def _wall(w, h, nx, ny):
    """A wall in the XY plane facing +z, its foot at y = 0, centred in x."""
    return grid(lambda u, v: (-w / 2 + w * u, h * v, 0.0), nx, ny)


def tissue_sides():
    w = TB_L - 2 * TB_C
    front = _wall(w, TB_H, 16, 5).move(0, 0, TB_D / 2)
    back = _wall(w, TB_H, 16, 5).rot(0, math.pi, 0).move(0, 0, -TB_D / 2)
    bottom = grid(lambda u, v: (-TB_L / 2 + TB_L * u, 0.0, -TB_D / 2 + TB_D * v), 4, 3)
    bottom = facing(bottom, (0, -1, 0))
    s = facing(front, (0, 0, 1)) + facing(back, (0, 0, -1)) + bottom
    # the four vertical folds: a narrow chamfer at each corner
    for sx in (-1, 1):
        for sz in (-1, 1):
            a = Vector((sx * (TB_L / 2 - TB_C), 0, sz * TB_D / 2))
            b = Vector((sx * TB_L / 2, 0, sz * (TB_D / 2 - TB_C)))
            ch = grid(lambda u, v, a=a, b=b: (a.lerp(b, u).x, TB_H * v, a.lerp(b, u).z), 1, 5)
            s = s + facing(ch, (sx, 0, sz))
    return s.warp(tb_crush)


def tissue_ends():
    """Each end: the end panel, and the outer flap glued over the inner ones
    (0.5 mm proud, ending 14 mm above the foot, where the bottom flap's edge
    shows)."""
    s = Shape()
    d = TB_D - 2 * TB_C
    for sx in (-1, 1):
        panel = grid(lambda u, v: (0.0, TB_H * v, -d / 2 + d * u), 6, 5)
        panel = facing(panel.move(sx * TB_L / 2, 0, 0), (sx, 0, 0))
        fl = grid(lambda u, v: (0.0, 0.0145 + (TB_H - 0.0153) * v, -(d - 0.003) / 2 + (d - 0.003) * u), 6, 5)
        fl = facing(fl.move(sx * (TB_L / 2 + 0.0005), 0, 0), (sx, 0, 0))
        lip = grid(lambda u, v: (sx * (TB_L / 2 + 0.0005 * v), 0.0145, -(d - 0.003) / 2 + (d - 0.003) * u), 6, 1)
        lip = facing(lip, (0, -1, 0))
        s = s + panel + fl + lip
    return s.warp(tb_crush)


def _top_rings():
    """The top panel as rings from the oval out to the box's edge: the
    edge sampled evenly along each side (corners exact), the oval at the
    same angles."""
    a, b = TB_HOLE
    hw, hd = TB_L / 2, TB_D / 2
    edge = []
    for (x0, z0), (x1, z1), n in (((-hw, -hd), (hw, -hd), 12), ((hw, -hd), (hw, hd), 6),
                                  ((hw, hd), (-hw, hd), 12), ((-hw, hd), (-hw, -hd), 6)):
        for i in range(n):
            t = i / n
            edge.append(Vector((x0 + (x1 - x0) * t, 0, z0 + (z1 - z0) * t)))
    rng = random.Random(7)
    oval = []
    for e in edge:
        ph = math.atan2(e.z, e.x)
        r = 1.0 / math.sqrt((math.cos(ph) / a) ** 2 + (math.sin(ph) / b) ** 2)
        r *= 1.0 + rng.uniform(-0.012, 0.012)            # the perforation's tear
        oval.append(Vector((r * math.cos(ph), 0, r * math.sin(ph))))
    return oval, edge


def tissue_top():
    oval, edge = _top_rings()
    fr = [0.0, 0.08, 0.22, 0.45, 0.72, 1.0]
    n = len(edge)
    v, f = [], []
    for t in fr:
        for o, e in zip(oval, edge):
            p = o.lerp(e, t)
            v.append(Vector((p.x, TB_H, p.z)))
    for j in range(len(fr) - 1):
        for i in range(n):
            i2 = (i + 1) % n
            f.append((j * n + i, j * n + i2, (j + 1) * n + i2, (j + 1) * n + i))
    s = facing(Shape(v, f), (0, 1, 0))
    # the oval's torn edge turned down a little into the box, facing in
    lip = Shape([Vector((o.x, TB_H, o.z)) for o in oval] + [Vector((o.x * 0.985, TB_H - 0.0018, o.z * 0.97)) for o in oval],
                [((i + 1) % n, i, n + i, n + (i + 1) % n) for i in range(n)])
    lip = facing_fn(lip, lambda c: (-c.x, 0, -c.z))
    return (s + lip).warp(tb_crush)


def tissue_inside():
    """The board's inside: walls facing in, 0.6 mm in from the outside."""
    w, d, h = TB_L - 2 * TB_T, TB_D - 2 * TB_T, TB_H - 0.0012
    s = Shape()
    walls = [
        (lambda u, v: (-w / 2 + w * u, h * v, d / 2), (0, 0, -1), 10),
        (lambda u, v: (-w / 2 + w * u, h * v, -d / 2), (0, 0, 1), 10),
        (lambda u, v: (w / 2, h * v, -d / 2 + d * u), (-1, 0, 0), 6),
        (lambda u, v: (-w / 2, h * v, -d / 2 + d * u), (1, 0, 0), 6),
    ]
    for fn, nd, nx in walls:
        s = s + facing(grid(fn, nx, 4), nd)
    return s.warp(tb_crush)


def tissue_stack():
    """The tissues left in it: their top soft and a little domed, pulled up
    toward the slot where the next one was drawn."""
    w, d = TB_L - 2 * TB_T - 0.001, TB_D - 2 * TB_T - 0.001

    def top(u, v):
        x, z = -w / 2 + w * u, -d / 2 + d * v
        y = TB_FILL - 0.004 * (abs(2 * u - 1) ** 3 + abs(2 * v - 1) ** 3)
        y += 0.003 * math.exp(-(x / 0.05) ** 2 - (z / 0.02) ** 2)
        y += 0.0012 * noise.noise(Vector((x * 40, 1.3, z * 40)))
        return (x, y, z)
    return facing(grid(top, 10, 6), (0, 1, 0))


def tissue_sheet():
    """The tissue that stood up out of the slot, long since gone limp: drawn
    up from the stack, gathered where the oval holds it into pleats that run
    up it and ease out, leaning over toward -z; one top corner fallen and
    lying on the carton, the other curled back on itself, the free edge
    uneven."""
    H = TB_H
    spine = chaikin([(0.0, TB_FILL + 0.002), (0.0, H - 0.014), (0.0, H + 0.012), (-0.006, H + 0.034),
                     (-0.016, H + 0.050), (-0.030, H + 0.056), (-0.042, H + 0.050)], 3)
    spine = [Vector((0, y, z)) for z, y in spine]
    rng = random.Random(15)
    ragged = [rng.uniform(-0.05, 0.03) for _ in range(15)]

    def width(t):
        pts = [(0.0, 0.040), (0.26, 0.026), (0.38, 0.026), (0.62, 0.050), (1.0, 0.064)]
        for (t0, w0), (t1, w1) in zip(pts, pts[1:]):
            if t <= t1:
                return w0 + (w1 - w0) * smooth01((t - t0) / (t1 - t0))
        return pts[-1][1]

    def fn(u, v):
        uu = 2 * u - 1
        vv = v * (1.0 + ragged[min(14, int(u * 14.999))] * smooth01((v - 0.8) / 0.2))
        c, tang = along(spine, min(1.0, vv))
        x = uu * width(v)
        nrm = Vector((0, -tang.z, tang.y))
        # pleats up it from the slot, deep there, easing out above
        g = math.exp(-((v - 0.32) / 0.22) ** 2)
        amp = 0.0060 * g + 0.0022 * (1 - g) * (1 - 0.5 * v)
        off = nrm * amp * math.sin(uu * 9.5 + 0.8 + v * 1.5)
        off += nrm * 0.0018 * noise.noise(Vector((x * 30, v * 5, 2.2)))
        top = smooth01((v - 0.5) / 0.5)
        p = c + Vector((x, 0, 0)) + off
        if uu < 0:
            # the left corner fallen: down and out onto the carton's top
            k = top * (-uu) ** 1.5
            p += Vector((-0.020 * k, -0.046 * k, 0.010 * k))
        else:
            # the right corner curled back over itself
            k = top * uu ** 2
            p += Vector((-0.008 * k, -0.010 * k, 0.014 * k))
        if v > 0.45 and abs(p.z) > TB_HOLE[1] - 0.001:
            p.y = max(p.y, H + 0.0011)
        if v > 0.45 and abs(p.x) > TB_HOLE[0] - 0.001:
            p.y = max(p.y, H + 0.0011)
        return (p.x, p.y, p.z)
    return grid(fn, 14, 16)


# =============================================================== the tape ===
# PVC insulating tape, 19 mm (3/4 in), 0.18 mm (7 mil), on a 1.5 in
# (38.1 mm) paperboard core (3M Temflex 1700 class); a 60 ft roll is about
# 76 mm across; this one half used, 60 mm. Modelled lying flat on a face.
TP_W = 0.019
TP_R = 0.030
TP_CI, TP_CO = 0.01905, 0.02065   # the core: inside, outside radius
TP_SEG = 28


def tape_faces():
    """The two wound faces, annuli; the inner turns stand 0.3 mm proud of
    the outer on top (telescoped), the bottom flat on the floor."""
    s = Shape()
    rings = [TP_CO + 0.0002, TP_CO + 0.004, 0.5 * (TP_CO + TP_R), TP_R - 0.0012]
    for top in (True, False):
        v, f = [], []
        for j, r in enumerate(rings):
            t = (r - rings[0]) / (rings[-1] - rings[0])
            y = (TP_W + 0.0003 * (1 - t)) if top else 0.0001
            for i in range(TP_SEG):
                a = TAU * i / TP_SEG
                v.append(Vector((r * math.sin(a), y, r * math.cos(a))))
        for j in range(len(rings) - 1):
            for i in range(TP_SEG):
                i2 = (i + 1) % TP_SEG
                f.append((j * TP_SEG + i, j * TP_SEG + i2, (j + 1) * TP_SEG + i2, (j + 1) * TP_SEG + i))
        s = s + facing(Shape(v, f), (0, 1 if top else -1, 0))
    return s


def tape_wrap():
    """The outer wrap and the rounded edges where it meets the faces."""
    R, e = TP_R, 0.0012
    prof = [(R - e, 0.0001), (R - e * 0.3, e * 0.3), (R, e), (R, TP_W - e), (R - e * 0.3, TP_W - e * 0.3 + 0.0001),
            (R - e, TP_W + 0.0001)]
    s = lathe(prof, seg=TP_SEG, cap_bottom=False, cap_top=False)
    # a turn's end, the outer layer's edge lying across the wrap
    return s


def tape_core():
    """The paperboard core: a tube, its ends a little frayed and crushed."""
    prof = [(TP_CI, 0.0002), (TP_CO, 0.0002), (TP_CO, TP_W + 0.0001), (TP_CI, TP_W + 0.0001)]
    v, f = [], []
    for (r, y) in prof:
        for i in range(TP_SEG):
            a = TAU * i / TP_SEG
            v.append(Vector((r * math.sin(a), y, r * math.cos(a))))
    n = len(prof)
    for j in range(n):
        j2 = (j + 1) % n
        for i in range(TP_SEG):
            i2 = (i + 1) % TP_SEG
            f.append((j * TP_SEG + i, j * TP_SEG + i2, j2 * TP_SEG + i2, j2 * TP_SEG + i))
    s = Shape(v, f)
    # winding: outward from the tube's wall section
    s = _fix(s)

    def fray(p):
        if p.y > TP_W * 0.5:
            a = math.atan2(p.x, p.z)
            p = Vector((p.x, p.y - 0.0004 * max(0.0, math.sin(a * 3 + 1.0)), p.z))
        return p
    return s.warp(fray)


def tape_tail():
    """The loose end: off the wrap at the front, tangent to it, standing
    (vinyl is stiff), curling out, its last 12 mm stretched thin and to a
    point where it was pulled apart."""
    R = TP_R + 0.0002
    a0 = 0.35                                  # where it leaves the roll (radians round from +z)
    # the path in plan: along the circle a little, then off along the tangent, curling outward
    pts = []
    for k in range(4):
        a = a0 - 0.30 + 0.10 * k
        pts.append(Vector((R * math.sin(a), 0, R * math.cos(a))))
    t = Vector((math.cos(a0), 0, -math.sin(a0)))           # the tangent (direction of travel round +y)
    o = pts[-1]
    nrm = Vector((math.sin(a0), 0, math.cos(a0)))           # outward
    for k in range(1, 8):
        s_ = 0.006 * k
        curl = 0.0011 * k * k * 0.5
        pts.append(o + t * s_ + nrm * curl)
    pts = chaikin(pts, 1)
    n = len(pts)
    L = sum((b - a).length for a, b in zip(pts, pts[1:]))
    v, f = [], []
    acc = 0.0
    for i, p in enumerate(pts):
        if i:
            acc += (p - pts[i - 1]).length
        u = acc / L
        # stretched to a point at its end: narrower and lower toward the tip
        nar = smooth01((u - 0.72) / 0.28)
        y0 = 0.0005 + 0.006 * nar
        y1 = TP_W - 0.0002 - 0.010 * nar
        if i == n - 1:
            y0, y1 = 0.0070, 0.0082
        tng = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
        side = Vector((tng.z, 0, -tng.x)) * 0.00009
        for y in (y0, y1):
            v.append(Vector((p.x, y, p.z)) + side)
            v.append(Vector((p.x, y, p.z)) - side)
    for i in range(n - 1):
        a, b = 4 * i, 4 * (i + 1)
        f.append((a, b, b + 2, a + 2))            # outer face
        f.append((a + 1, a + 3, b + 3, b + 1))    # inner face
        f.append((a, a + 1, b + 1, b))            # bottom edge
        f.append((a + 2, b + 2, b + 3, a + 3))    # top edge
    a = 4 * (n - 1)
    f.append((a, a + 2, a + 3, a + 1))
    f.append((0, 1, 3, 2))
    return _fix(Shape(v, f))


# ============================================================ the cards ===
# A poker-size playing card, 2.5 x 3.5 in (63.5 x 88.9 mm), 0.30 mm; the
# corners rounded 1/8 in. Lying face up: x across, z along its length.
CD_W, CD_L, CD_T, CD_R = 0.0635, 0.0889, 0.0003, 0.0032
CD_DECK = 46                       # the cards under the loose ones


def card_outline(w=CD_W, l=CD_L, r=CD_R):
    return rounded_rect(w, l, r, seg=3)


def card_single():
    s = prism(card_outline(), CD_T)
    return s.rot(-math.pi / 2, 0, 0).move(0, CD_T / 2, 0)


def card_deck():
    """The deck under the loose cards: 46 cards in four bunches, each a
    little off square with the next (a deck put down, not squared)."""
    s = Shape()
    rng = random.Random(4)
    counts = [12, 11, 12, 11]
    y = 0.0
    for k, c in enumerate(counts):
        h = c * CD_T
        b = prism(card_outline(), h).rot(-math.pi / 2, 0, 0)
        b.rot(0, rng.uniform(-0.012, 0.012), 0)
        b.move(rng.uniform(-0.0007, 0.0007), y + h / 2, rng.uniform(-0.0007, 0.0007))
        s = s + b
        y += h
    return s


# ======================================================= the paperback ===
# r02's trade paperback (216 x 140 mm, 22 mm of pages, 0.7 mm cover card)
# lying open, face up: x across the spread (the gutter at x = 0), z along
# the page's height (its head at -z), y up.
PB_H, PB_W, PB_T, PB_CT = 0.216, 0.140, 0.022, 0.0007
PB_HALF = PB_T / 2
PB_GUT = 0.0012                     # the gutter: where the halves meet the spine


def pb_top(x):
    """The height of the page's surface at x (|x| from the gutter): diving
    into the gutter, full a hand's width out, the fore-edge swollen; the
    right-hand leaves lifting toward the fore-edge."""
    ax = abs(x)
    base = PB_CT + 0.0012 + (PB_HALF - 0.0012) * smooth01((ax - PB_GUT) / 0.030) ** 0.8
    swell = 0.0008 * smooth01((ax - 0.10) / 0.04)
    lift = 0.0045 * smooth01((x - 0.085) / 0.058) if x > 0 else 0.0
    return base + swell + lift


def pb_wave(x, z):
    """Cockled from damp: a slow wave down the page."""
    return 0.0006 * math.sin(z * 38 + x * 9) * smooth01((abs(x) - 0.01) / 0.03)


def pb_spread():
    """The two pages' printed faces: one surface across the spread, its
    texture coordinates the box's (u across, v down the page)."""
    xs = [-PB_W - PB_GUT, -0.11, -0.07, -0.04, -0.024, -0.014, -0.007, -PB_GUT - 0.0003, PB_GUT + 0.0003, 0.007, 0.014, 0.024, 0.04,
          0.07, 0.10, 0.12, PB_W + PB_GUT]
    nz = 8

    def fn(i, j):
        x = xs[i]
        z = -PB_H / 2 + PB_H * j / nz
        return Vector((x, pb_top(x) + 0.00025 + pb_wave(x, z), z))
    v = [fn(i, j) for j in range(nz + 1) for i in range(len(xs))]
    f = []
    m = len(xs)
    for j in range(nz):
        for i in range(m - 1):
            if i == 7:
                continue                      # the gutter: the two pages part
            a = j * m + i
            f.append((a, a + 1, a + m + 1, a + m))
    return facing(Shape(v, f), (0, 1, 0))


def pb_block():
    """The pages' edges: each half a block from the gutter to its fore-edge
    under the spread, its head, tail and fore-edge faces; the fore-edge a
    little uneven where the leaves fan."""
    s = Shape()
    for sg in (-1, 1):
        xs = [sg * (PB_GUT + 0.0003 + (PB_W - 0.0003) * t) for t in (0.0, 0.03, 0.08, 0.15, 0.3, 0.55, 0.8, 1.0)]
        xs = sorted(xs)
        nz = 6
        # head and tail: the section's outline
        for z, nd in ((-PB_H / 2 + 0.0004, -1), (PB_H / 2 - 0.0004, 1)):
            v = []
            for x in xs:
                v.append(Vector((x, 0.0003, z)))
                v.append(Vector((x, pb_top(x) + pb_wave(x, z), z)))
            f = [(2 * i, 2 * i + 2, 2 * i + 3, 2 * i + 1) for i in range(len(xs) - 1)]
            s = s + facing(Shape(v, f), (0, 0, nd))
        # the fore-edge
        xf = xs[-1] if sg > 0 else xs[0]
        v = []
        for j in range(nz + 1):
            z = -PB_H / 2 + 0.0004 + (PB_H - 0.0008) * j / nz
            bulge = 0.0006 * math.sin(math.pi * j / nz)
            v.append(Vector((xf + sg * bulge, 0.0003, z)))
            v.append(Vector((xf + sg * bulge, pb_top(xf) + pb_wave(xf, z), z)))
        f = [(2 * j, 2 * j + 2, 2 * j + 3, 2 * j + 1) for j in range(nz)]
        s = s + facing(Shape(v, f), (sg, 0, 0))
    return s


def pb_covers():
    """The covers opened out flat under the pages (a paperback's covers are
    flush with them: a sliver shows at head, tail and fore-edge), the spine
    pressed round under the gutter."""
    xs = [-PB_W - PB_GUT - 0.0005, -0.10, -0.05, -0.012, -0.006, 0.0, 0.006, 0.012, 0.05, 0.10, PB_W + PB_GUT + 0.0005]
    nz = 6

    def y_at(x, z):
        ax = abs(x)
        return PB_CT + 0.0004 * (1 - smooth01(ax / 0.012))       # the spine pressed round under
    v = []
    for j in range(nz + 1):
        z = -PB_H / 2 - 0.0004 + (PB_H + 0.0008) * j / nz
        for x in xs:
            v.append(Vector((x, y_at(x, z), z)))
    m = len(xs)
    f = []
    for j in range(nz):
        for i in range(m - 1):
            a = j * m + i
            f.append((a, a + 1, a + m + 1, a + m))
    top = facing(Shape(v, f), (0, 1, 0))
    bot = facing(Shape([p - Vector((0, PB_CT, 0)) for p in v], list(f)), (0, -1, 0))
    s = top + bot
    return s.warp(lambda p: Vector((p.x, max(p.y, 0.0), p.z)))


# ======================================================= the newspaper ===
# A Berliner daily (315 x 470 mm a page) folded once across: 315 x 235 mm,
# the fold at +z, the front page's top half up (its masthead at the open
# edge, -z); 6 mm thick at the fold, the top half a little short of the
# bottom at the open edge.
NW_X, NW_Z, NW_T = 0.315, 0.235, 0.006
NW_XS = [-NW_X / 2 + NW_X * t for t in (0.0, 0.08, 0.22, 0.38, 0.5, 0.62, 0.78, 0.92, 1.0)]


def nw_warp(x, z, curl=False):
    """How the folded paper lies: fuller over its middle, the open edge
    wavy where the leaves part; with `curl`, the front page's open corner
    at (-x, -z) curling up off the rest."""
    u = (x + NW_X / 2) / NW_X
    w = (z + NW_Z / 2) / NW_Z
    y = 0.0010 * math.sin(math.pi * u) * math.sin(math.pi * min(1.0, w * 1.15))
    y += 0.0009 * math.sin(x * 31 + 0.7) * (1 - smooth01(w / 0.4))
    if curl:
        c = (1 - smooth01(u / 0.24)) * (1 - smooth01(w / 0.32))
        y += 0.0075 * c ** 1.6
    return y


# the section in (z, y), counterclockwise from the bottom's open edge, and
# what each segment from a point to the next is: the bottom and round the
# fold ('body'), the top back to the open edge ('top', under the front
# page), down the open edges of the two halves ('edge')
NW_R = NW_T / 2
NW_ZF = NW_Z / 2 - NW_R
NW_SEC = ([(-NW_Z / 2, 0.0), (-NW_Z / 2 + 0.03, 0.0), (NW_ZF - 0.02, 0.0)]
          + [(NW_ZF + NW_R * math.cos(-math.pi / 2 + math.pi * k / 6), NW_R + NW_R * math.sin(-math.pi / 2 + math.pi * k / 6)) for k in range(7)]
          + [(NW_ZF - 0.02, NW_T), (-NW_Z / 2 + 0.03, NW_T), (-NW_Z / 2 + 0.0015, NW_T), (-NW_Z / 2 + 0.0015, NW_R + 0.0002),
             (-NW_Z / 2, NW_R - 0.0002)])
NW_SEG = ['body'] * 9 + ['top'] * 3 + ['edge'] * 3


def _news_out(c):
    """Which way a face of the folded paper looks out, from its centre."""
    if abs(c.x) > NW_X / 2 - 1e-4:
        return (math.copysign(1, c.x), 0, 0)
    if c.z < -NW_Z / 2 + 0.003:
        return (0, 0, -1)
    if c.z < NW_ZF - 1e-4:
        return (0, -1, 0)
    return (0, c.y - NW_R, c.z - NW_ZF)


def _news(kind):
    """The folded paper's faces of one kind: 'body' (the bottom and the
    fold) or 'edge' (the open edges and the two cut ends)."""
    n = len(NW_SEC)
    rng = random.Random(12)
    jag = [rng.uniform(-0.0006, 0.0006) for _ in NW_XS]
    v = []
    for i, x in enumerate(NW_XS):
        for (z, y) in NW_SEC:
            zz = z + (jag[i] if z < -NW_Z / 2 + 0.002 else 0.0)
            v.append(Vector((x, y + nw_warp(x, zz) * y / NW_T, zz)))
    f = []
    for i in range(len(NW_XS) - 1):
        for k in range(n):
            if NW_SEG[k] == kind:
                k2 = (k + 1) % n
                f.append((i * n + k, (i + 1) * n + k, (i + 1) * n + k2, i * n + k2))
    s = Shape(v, f)
    if kind == 'edge':
        for i in (0, len(NW_XS) - 1):
            s.f.append(tuple(i * n + k for k in range(n)))
    return facing_fn(s, _news_out)


def news_body():
    return _news('body')


def news_edges():
    return _news('edge')


def news_front(curl=False):
    """The top half of the front page, over the paper's top (with `curl`,
    its open corner lifting: the top paper's)."""
    z0, z1 = -NW_Z / 2 + 0.0015, NW_Z / 2 - NW_T / 2

    def fn(u, v):
        x = -NW_X / 2 + NW_X * u
        z = z0 + (z1 - z0) * v
        return (x, NW_T + 0.0002 + nw_warp(x, z, curl), z)
    return facing(grid(fn, 10, 7), (0, 1, 0))


# ===================================================== the paper dart ===
# The classic dart folded from a sheet of A4 (210 x 297 mm): folded in half
# along its length, the top corners folded to the middle twice (its edges
# then at 22.5 degrees to the middle, reaching the sheet's sides 253 mm
# down), folded in half, then each wing folded down along a line from the
# nose to the tail, 42% of the way out: 297 mm long, the keel 44 mm deep at
# the tail and nothing at the nose, the wings 61 mm each. Crashed nose first
# and left where it fell, years ago: resting on its keel's crease and its
# left wing tip, the wings gone soft and drooping.
PL_L, PL_HW, PL_F = 0.297, 0.105, 0.42
PL_TAN = math.tan(math.radians(22.5))
PL_S = [0.003, 0.010, 0.020, 0.035, 0.055, 0.08, 0.11, 0.14, 0.17, 0.20, 0.23, 0.253, 0.275, 0.297]


def pl_hw(s):
    return min(PL_HW, max(0.0012, s * PL_TAN))


def pl_crash(p, s):
    """The nose crumpled back and down, folded over to the right; the left
    wing's trailing edge curled up; the right wing's tip curled down."""
    q = p.copy()
    k = 1 - smooth01(s / 0.045)
    q.z -= 0.009 * k
    q.y -= 0.004 * k * k
    q.x += 0.004 * k * k + 0.0012 * k * noise.noise(Vector((s * 90, p.y * 90, 1.7)))
    q.y += 0.0010 * k * noise.noise(Vector((s * 120, 3.3, p.x * 120)))
    if p.x > 0:
        out = max(0.0, p.x - 0.040)
        q.y -= out * 0.25 * smooth01((s - 0.26) / 0.04)
    else:
        q.y += 0.006 * smooth01((s - 0.275) / 0.022) * smooth01((-p.x - 0.03) / 0.04)
    return q


def plane_dart():
    s_all = Shape()
    n = len(PL_S)
    for sx in (-1, 1):
        # the keel's wall on this side: the centre crease at its foot, the
        # wing's fold at its head (the two walls opening a little up it)
        v = []
        for s in PL_S:
            k = PL_F * pl_hw(s)
            for j in range(3):
                t = j / 2
                v.append(pl_crash(Vector((sx * (0.0003 + 0.0012 * t), -k * (1 - t), PL_L / 2 - s)), s))
        f = [(i * 3 + j, i * 3 + j + 1, (i + 1) * 3 + j + 1, (i + 1) * 3 + j) for i in range(n - 1) for j in range(2)]
        s_all = s_all + Shape(v, f)
        # the wing, from the fold out to its edge, drooping more toward the
        # tip (the left more: it hit on that side)
        bend = (math.radians(-4.0), math.radians(-24.0)) if sx > 0 else (math.radians(-6.0), math.radians(-32.0))
        nj = 5
        v = []
        for s in PL_S:
            ww = (1 - PL_F) * pl_hw(s)
            x, y = 0.0015, 0.0
            for j in range(nj):
                if j:
                    t = (j - 0.5) / (nj - 1)
                    a = bend[0] + (bend[1] - bend[0]) * smooth01((t - 0.2) / 0.8)
                    x += ww / (nj - 1) * math.cos(a)
                    y += ww / (nj - 1) * math.sin(a)
                v.append(pl_crash(Vector((sx * x, y + 0.0004 * math.sin(math.pi * j / (nj - 1)), PL_L / 2 - s)), s))
        f = [(i * nj + j, i * nj + j + 1, (i + 1) * nj + j + 1, (i + 1) * nj + j) for i in range(n - 1) for j in range(nj - 1)]
        s_all = s_all + Shape(v, f)
    # on the floor: pitched down until the keel's crease lies along it...
    th = math.atan2(PL_F * PL_HW, PL_HW / PL_TAN)
    s_all.rot(th, 0, 0)
    # ...and rolled onto the left wing until its trailing tip touches
    tip = min(s_all.v, key=lambda p: p.x)
    keel = min(s_all.v, key=lambda p: p.y if abs(p.x) < 0.003 else 9.0)
    phi = math.atan2(tip.y - keel.y, keel.x - tip.x)
    s_all.rot(0, 0, phi)
    return on_floor(s_all)


# ========================================================== the napkin ===
# A two-ply luncheon napkin, 33 x 33 cm, sold folded in four (165 mm
# square); used, crumpled in a hand and dropped: its folds soft (tissue,
# not office paper), one flap of it half open and lifting.
NP_S = 0.165


def napkin_crumpled():
    rng = random.Random(21)
    folds = []
    for _ in range(7):
        a = rng.uniform(0, math.pi)
        folds.append((Vector((rng.uniform(-0.05, 0.05), 0, rng.uniform(-0.05, 0.05))), Vector((math.cos(a), 0, math.sin(a))),
                      rng.uniform(0.003, 0.009) * rng.choice((1, 1, -1)), rng.uniform(0.008, 0.02)))

    def fn(u, v):
        x, z = -NP_S / 2 + NP_S * u, -NP_S / 2 + NP_S * v
        q = Vector((x, 0, z))
        h = 0.0
        pull = Vector((0, 0, 0))
        for p, d, amp, wdt in folds:
            rel = q - p
            dist = rel.x * d.z - rel.z * d.x
            g = math.exp(-(dist / wdt) ** 2)
            h += amp * g
            # where it was gathered the cloth is drawn in across the fold
            pull += Vector((d.z, 0, -d.x)) * (-0.004 * g * math.copysign(1, dist) * abs(amp) / 0.009)
        # the flap of the top layer at the open corner (+x, +z) lifting and curling back
        c = smooth01((x / (NP_S / 2) + z / (NP_S / 2) - 0.9) / 1.1)
        h += 0.022 * c ** 1.5
        pull += Vector((-0.010, 0, -0.010)) * c ** 2
        h += 0.0025 * noise.noise(Vector((x * 30, 0.5, z * 30))) + 0.0012 * noise.noise(Vector((x * 70, 2.5, z * 70)))
        return (x + pull.x + 0.002 * noise.noise(Vector((x * 20, 7.0, z * 20))), h, z + pull.z)
    s = grid(fn, 16, 16)
    hs = sorted(p.y for p in s.v)
    base = hs[len(hs) // 3]
    s.warp(lambda p: Vector((p.x, max(0.0, p.y - base) + 0.0004, p.z)))
    return facing(s, (0, 1, 0))


def _hull_sphere(n, rng, jitter=0.45):
    pts = []
    ga = math.pi * (3 - math.sqrt(5))
    step = math.sqrt(4 * math.pi / n)
    for i in range(n):
        y = 1 - 2 * (i + 0.5) / n
        r = math.sqrt(max(0.0, 1 - y * y))
        p = Vector((math.cos(ga * i) * r, y, math.sin(ga * i) * r))
        p += Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * step * jitter
        pts.append(p.normalized())
    bm = bmesh.new()
    for p in pts:
        bm.verts.new(p)
    bmesh.ops.convex_hull(bm, input=bm.verts)
    bm.verts.index_update()
    s = Shape([v.co.copy() for v in bm.verts], [tuple(v.index for v in f.verts) for f in bm.faces])
    bm.free()
    return _fix(s)


def _flap(rng, ball, d, size, lift=(0.15, 0.45)):
    """A corner of the sheet coming away from the ball at direction d: a thin
    triangle along the surface, lifting toward its tip (after r08's)."""
    up = Vector((0.3, 1, 0.2)) if abs(d.y) < 0.9 else Vector((1, 0, 0))
    t = d.cross(up).normalized()
    b = d.cross(t).normalized()
    ang = rng.uniform(0, TAU)
    t, b = t * math.cos(ang) + b * math.sin(ang), b * math.cos(ang) - t * math.sin(ang)
    L, W = size, size * rng.uniform(0.8, 1.2)
    tip = rng.uniform(-0.3, 0.3) * W
    pts = [(0.0, -W / 2), (L * 0.5, (-W / 2 + tip) / 2), (L, tip), (L * 0.5, (W / 2 + tip) / 2), (0.0, W / 2)]
    lf = rng.uniform(*lift)
    v = []
    for a_, c in pts:
        q = (d * 0.6 + t * (a_ - L * 0.35) / ball + b * c / ball).normalized()
        k = 0.93 + lf * (a_ / L) ** 1.6 * min(0.5, size / ball)
        v.append(q * ball * k)
    th = 0.0003
    vv = [p * (1 + th / ball) for p in v] + [p * (1 - th / ball) for p in v]
    f = [(0, 1, 3, 4), (1, 2, 3), (9, 8, 6, 5), (8, 7, 6)]
    for a_ in range(5):
        c = (a_ + 1) % 5
        f.append((a_, c, c + 5, a_ + 5))
    return _fix(Shape(vv, f))


def napkin_wad():
    """Scrunched in a hand and dropped: a loose wad of soft tissue, wider
    than tall, its many small creases (r08's crumple, looser), pockets
    where it folded in, its corners and an edge coming away from it."""
    rng = random.Random(33)
    r = 0.036
    s = _hull_sphere(104, rng, 0.42)
    cr = []
    for _ in range(11):
        nn = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
        cr.append((nn.normalized() if nn.length > 0.1 else Vector((0, 1, 0)), rng.uniform(-0.6, 0.6), rng.uniform(0.04, 0.10), rng.choice((1, 1, -1))))
    o1 = Vector((rng.uniform(-30, 30), rng.uniform(-30, 30), rng.uniform(-30, 30)))
    jit = [rng.uniform(-1, 1) for _ in s.v]
    pk = [Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))).normalized() for _ in range(8)]

    def crumple(i, p):
        d = p.normalized()
        k = 1.0 + 0.16 * noise.noise(d * 1.3 + o1) + 0.07 * jit[i]
        for nn, off, depth, sg in cr:
            x = d.dot(nn) - off
            k -= sg * depth * max(0.0, 0.14 - abs(x)) / 0.14
        for q in pk:
            k -= 0.24 * max(0.0, d.dot(q) - 0.92) / 0.08
        q = d * r * k
        q.y *= 0.62
        q.x *= 1.15
        return q
    s.v = [crumple(i, p) for i, p in enumerate(s.v)]
    lo, hi = s.bounds()
    flat = lo.y + (hi.y - lo.y) * 0.12
    s.warp(lambda p: Vector((p.x, max(p.y, flat), p.z)))
    for k in range(4):
        d = Vector((rng.uniform(-1, 1), rng.uniform(-0.3, 0.6), rng.uniform(-1, 1))).normalized()
        f = _flap(rng, r, d, 0.032 * rng.uniform(0.8, 1.25), (0.25, 0.6))
        f.scale(1.15, 0.62, 1.0)
        s = s + f
    return on_floor(s)


# ================================================================ parts ===
CYL = {'Real__tapeRollFaces', 'Real__tapeRollWrap', 'Real__tapeRollCore'}
LOOK = {
    # (smoothing angle, flat)
    'Real__tissueBoxSheet': (75.0, False), 'Real__tissueBoxStack': (75.0, False),
    'Real__tapeRollWrap': (50.0, False), 'Real__tapeRollCore': (50.0, False), 'Real__tapeRollTail': (60.0, False),
    'Real__cardSingle': (35.0, True), 'Real__cardDeck': (35.0, True),
    'Real__pbOpenSpread': (60.0, False), 'Real__pbOpenBlock': (40.0, False),
    'Real__newsFront': (60.0, False), 'Real__newsFrontCurl': (60.0, False), 'Real__newsBody': (50.0, False),
    'Real__paperPlane': (35.0, True),
    'Real__napkinCrumpled': (70.0, False), 'Real__napkinWad': (42.0, False),
}


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    smooth, flat = LOOK.get(name, (35.0, False))
    kind = 'cyl' if name in CYL else 'box'
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, smooth=smooth, flat=flat, budget=budget)


auto('Real__tissueBoxTop', tissue_top, 560)
auto('Real__tissueBoxSides', tissue_sides, 560)
auto('Real__tissueBoxEnds', tissue_ends, 300)
auto('Real__tissueBoxInside', tissue_inside, 300)
auto('Real__tissueBoxStack', tissue_stack, 200)
auto('Real__tissueBoxSheet', tissue_sheet, 480)
auto('Real__tapeRollFaces', tape_faces, 380)
auto('Real__tapeRollWrap', tape_wrap, 300)
auto('Real__tapeRollCore', tape_core, 240)
auto('Real__tapeRollTail', tape_tail, 200)
auto('Real__cardSingle', card_single, 100)
auto('Real__cardDeck', card_deck, 280)
auto('Real__pbOpenCovers', pb_covers, 300)
auto('Real__pbOpenBlock', pb_block, 200)
auto('Real__pbOpenSpread', pb_spread, 300)
auto('Real__newsBody', news_body, 300)
auto('Real__newsEdges', news_edges, 200)
auto('Real__newsFront', news_front, 160)
auto('Real__newsFrontCurl', lambda: news_front(True), 160)
auto('Real__paperPlane', plane_dart, 400)
auto('Real__napkinCrumpled', napkin_crumpled, 560)
auto('Real__napkinWad', napkin_wad, 330)
