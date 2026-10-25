"""Batch b09 - "domestic" theme: household things lying around or hung up.

placeCoinScatters / placeCoinClusters   coin (rim, relief field), shared
placeHospitalCrosses   rounded sign plate and cross bars (unlit)
placeSingleBoots       lugged sole, toe box, heel counter, shaft with collar
placeFoldedUmbrellas   fluted furled canopy, seam stripes, ferrule tip, crook handle
placePizzaBoxes        corrugated box with lid seam and vents, open lid with flaps
placeSingleGloves      puffy palm, soft fingers, ribbed knit cuff
placeCigarettePacks    flip-top pack, lid
placeMatchbooks        folded matchbook, square match stick, dipped head
placeBrokenSunglasses  acetate rim, bridge, temple, snapped temple
placeUnrolledTPRolls   hollow cardboard core with spiral seam
placeBrokenFlashlights knurled barrel with tail cap, flared head with reflector
placeShoeBoxes         box body, hollow lid
placeCoatRacks         moulded rail, brass pegs and knobs, coat, collar, scarf
placeCrumpledTissues   crumpled tissue lump
placeVinylRecords      grooved LP, label, album sleeve
placeGumBallDispensers cast base, gumball, turn knob
placeCrossStitches     double embroidery hoop, thumb nut
placeWatercolors       gold moulded frame, bevelled mat
placeToteBags          open canvas tote, strap handles, tool handles
placeCoatHooksPair     hook board, rosette, arm, hook curve
placeCastIronPans      skillet, cooking surface, handle, hanging ring
placeVintageMaps       wooden frame
placeMasonJars         screw band, lid, preserve fill
placeWallBrooms        handle, spring clip, besom bristles, twine binding
placeRollingPins       roller, turned handle, knob
placeCuttingBoards     board with hanging hole
placeMixingBowls       bowl, rolled rim, dough
placeWhisks            handle, end cap, ferrule, wire loop
placeWallAprons        peg, neck strap
placeTeapots           trivet, feet, body, spout, lid, knob, handle
placeCookieTins        tin body with lid skirt, rolled rim
placeTeacups           saucer, cup, handle, tea
placeCastIronLadles    twisted handle, ring, bowl
"""
from propkit import *
import propkit
from mathutils import noise

HALF = math.pi / 2


# ------------------------------------------------------------- helpers ---
def loft(rings, cap_bottom=True, cap_top=True):
    """Connect 2D outlines (same point count) stacked along Z into a closed
    solid. rings: [(outline [(x, y), ...], z), ...] from back to front."""
    v, f = [], []
    n = len(rings[0][0])
    for pts, z in rings:
        v += [(x, y, z) for x, y in pts]
    for k in range(len(rings) - 1):
        a, b = k * n, (k + 1) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    if cap_bottom:
        f.append(tuple(reversed(range(n))))
    if cap_top:
        f.append(tuple(range((len(rings) - 1) * n, len(rings) * n)))
    return propkit._fix_winding(Shape(v, f))


def rr(w, h, r, seg=2, cx=0.0, cy=0.0):
    """Rounded rectangle outline with a constant point count."""
    r = max(min(r, w / 2 - 1e-5, h / 2 - 1e-5), 2e-4)
    return [(x + cx, y + cy) for x, y in rounded_rect(w, h, r, seg)]


def plate(w, h, profile, r=0.004, seg=2):
    """Plate from back to front: profile = [(inset, z), ...]."""
    return loft([(rr(w - 2 * i, h - 2 * i, max(r - i, 3e-4), seg), z) for i, z in profile])


def frame_ring(wo, ho, wi, hi, d, z=0.0):
    """Rectangular ring cutter (outer minus inner), d deep, centred at z."""
    return cut(box(wo, ho, d), box(wi, hi, d * 2)).move(z=z)


def on_z(s):
    """A lathe built on +Y turned to face +Z."""
    return s.rot(x=HALF)


def arc_sweep(section, R, a0, a1, n, caps=True):
    """Sweep a closed 2D section [(dr, dz), ...] around the Z axis at radius R
    from angle a0 to a1 (a ring segment in the XY plane, like TorusGeometry)."""
    m = len(section)
    v, f = [], []
    for i in range(n + 1):
        a = a0 + (a1 - a0) * i / n
        c, s = math.cos(a), math.sin(a)
        v += [((R + dr) * c, (R + dr) * s, dz) for dr, dz in section]
    for i in range(n):
        for j in range(m):
            k = (j + 1) % m
            f.append((i * m + j, i * m + k, (i + 1) * m + k, (i + 1) * m + j))
    if caps:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(n * m + j for j in range(m)))
    return propkit._fix_winding(Shape(v, f))


def ellipse(rx, ry, n, cx=0.0, cy=0.0):
    return [(cx + rx * math.cos(TAU * i / n), cy + ry * math.sin(TAU * i / n)) for i in range(n)]


def angular(s, fn):
    """Warp each vertex radially by fn(theta, y, r) -> new r (around +Y)."""
    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-7:
            return p
        a = math.atan2(p.x, p.z)
        k = fn(a, p.y, r) / r
        return Vector((p.x * k, p.y, p.z * k))
    return s.warp(w)


# ------------------------------------------------------------------ coins --
def coin():
    # struck coin: rounded milled edge, raised rim, a field sunk inside it
    r, h = 0.008, 0.001
    return lathe([(0, -h / 2), (0.94 * r, -h / 2), (r, 0.0), (0.94 * r, h / 2), (0.80 * r, 0.30 * h),
                  (0, 0.36 * h)], seg=12)

part('CoinScatters__coin', f_cyl(0.008, 0.008, 0.001), coin)


# ------------------------------------------------------- hospital cross --
def cross_plate(w, h, d, r):
    # unlit sign plastic: only the outline shows, so round the corners
    return plate(w, h, [(0.0006, -d / 2), (0, -d / 2 + 0.0006), (0, d / 2 - 0.0006), (0.0006, d / 2)], r=r, seg=3)

part('HospitalCrosses__back', f_box(0.14, 0.14, 0.010), lambda: cross_plate(0.14, 0.14, 0.010, 0.012))
part('HospitalCrosses__vbar', f_box(0.042, 0.126, 0.008), lambda: cross_plate(0.042, 0.126, 0.008, 0.004))
part('HospitalCrosses__hbar', f_box(0.126, 0.042, 0.008), lambda: cross_plate(0.126, 0.042, 0.008, 0.004))


def rod(points, r, seg=8, smooth=0, caps=2, r_end=None):
    """A round rod along a polyline with domed ends (the domes stay inside
    the rod's own length: the path is shortened by r at each end)."""
    pts = [Vector(p) for p in points]
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
    up = Vector((0, 1, 0)) if abs(tan[0].y) < 0.9 else Vector((1, 0, 0))
    nrm = tan[0].cross(up).normalized()
    frames = []
    for i in range(n):
        if i:
            nrm = (tan[i - 1].rotation_difference(tan[i]) @ nrm).normalized()
        frames.append((nrm.copy(), tan[i].cross(nrm).normalized()))
    rad = [r if r_end is None else r + (r_end - r) * i / max(1, n - 1) for i in range(n)]
    # pull the ends in so the domes finish where the path did
    pts[0] = pts[0] + tan[0] * rad[0]
    pts[-1] = pts[-1] - tan[-1] * rad[-1]
    rings = []                                   # (centre, N, B, radius)
    for k in range(caps, 0, -1):
        b = HALF * k / (caps + 1)
        rings.append((pts[0] - tan[0] * rad[0] * math.sin(b), *frames[0], rad[0] * math.cos(b)))
    rings += [(pts[i], *frames[i], rad[i]) for i in range(n)]
    for k in range(1, caps + 1):
        b = HALF * k / (caps + 1)
        rings.append((pts[-1] + tan[-1] * rad[-1] * math.sin(b), *frames[-1], rad[-1] * math.cos(b)))
    v = [tuple(pts[0] - tan[0] * rad[0])]
    for c, N, B, rr_ in rings:
        v += [tuple(c + (N * math.cos(TAU * j / seg) + B * math.sin(TAU * j / seg)) * rr_) for j in range(seg)]
    v.append(tuple(pts[-1] + tan[-1] * rad[-1]))
    f = []
    last = len(v) - 1
    for j in range(seg):
        f.append((0, 1 + (j + 1) % seg, 1 + j))
    for k in range(len(rings) - 1):
        a, b = 1 + k * seg, 1 + (k + 1) * seg
        for j in range(seg):
            j2 = (j + 1) % seg
            f.append((a + j, a + j2, b + j2, b + j))
    a = 1 + (len(rings) - 1) * seg
    for j in range(seg):
        f.append((a + j, a + (j + 1) % seg, last))
    return propkit._fix_winding(Shape(v, f))


def half_arc_rod(R, r, seg=8, n=12, caps=2):
    """A rod along the half ring of TorusGeometry(R, r, .., PI), the domed
    ends landing on y = 0 like the torus's cut ends."""
    t0 = 0.0
    pts = [(R * math.cos(t0 + (math.pi - 2 * t0) * i / n), R * math.sin(t0 + (math.pi - 2 * t0) * i / n), 0)
           for i in range(n + 1)]
    return rod(pts, r, seg=seg, caps=caps)


# ------------------------------------------------------------ single boot --
def outline_foot(L, W, n=14, p=2.6):
    """Plan outline of a sole, heel at -x, toe at +x: [(x, z), ...]."""
    base = [(-1.0, 0.84), (-0.7, 0.89), (-0.25, 0.77), (0.3, 1.0), (0.6, 1.0), (1.0, 0.88)]
    def w(u):
        for (a, wa), (b, wb) in zip(base, base[1:]):
            if u <= b:
                t = (u - a) / (b - a)
                return wa + (wb - wa) * t
        return base[-1][1]
    xs = [-math.cos(math.pi * i / n) for i in range(n + 1)]
    right = [(u * L, W * w(u) * (1 - abs(u) ** p) ** (1 / p)) for u in xs]
    left = [(x, -z) for x, z in reversed(right[1:-1])]
    return right + left


def boot_sole():
    L, W = 0.065, 0.0275
    o = outline_foot(L, W)
    ins = lambda d: [(x * (1 - d / L), z * (1 - d / W)) for x, z in o]
    s = loft([(ins(0.0012), -0.007), (o, -0.0058), (o, 0.0025), (ins(0.0006), 0.003), (o, 0.0035),
              (o, 0.0062), (ins(0.0010), 0.007)]).rot(x=-HALF)
    # arch under the waist, lug grooves across forefoot and heel
    s = cut(s, rbox(0.036, 0.008, 0.07, r=0.0025, seg=1).move(x=-0.006, y=-0.0095))
    for gx in (0.018, 0.029, 0.040, 0.051, -0.050, -0.038):
        s = cut(s, box(0.0025, 0.004, 0.07).move(x=gx, y=-0.0078))
    return s.fit((-0.065, -0.007, -0.0275), (0.065, 0.007, 0.0275))

def boot_toe():
    # toe box: flat on the sole, widest over the ball, rounded toe at +x,
    # a toe-cap stitch line; the back end tucks into the heel upper
    rings = []
    us = [0.0, 0.18, 0.35, 0.52, 0.55, 0.58, 0.7, 0.8, 0.88, 0.94, 0.985]
    for u in us:
        tip = max(0.0, (u - 0.35) / 0.65)
        wz = 0.035 * (0.87 + 0.13 * min(1.0, u / 0.35)) * (1 - tip ** 2.4) ** (1 / 2.4)
        th = max(0.0, (u - 0.2) / 0.8)
        H = 0.07 * (1 - 0.3 * u) * (1 - th ** 2.2) ** (1 / 2.2)
        if abs(u - 0.55) < 0.01:
            wz *= 0.965
            H -= 0.002
        yc = -0.035 + H / 2
        pts = []
        for i in range(16):
            a = TAU * i / 16
            c, sn = math.cos(a), math.sin(a)
            e = 2 / 2.4 if sn >= 0 else 2 / 4.0
            pts.append((wz * math.copysign(abs(c) ** e, c), yc + H / 2 * math.copysign(abs(sn) ** e, sn)))
        rings.append((pts, -0.035 + 0.07 * u))
    s = loft(rings).rot(y=HALF)
    return s.fit((-0.035, -0.035, -0.035), (0.035, 0.035, 0.035))

def d_outline(w, d, r_back, r_front, seg=3):
    """Rectangle outline (x: w, y: d) with round back (-x) corners."""
    c = [(-w / 2, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (-w / 2, d / 2)]
    rs = [r_back, r_front, r_front, r_back]
    out = []
    for i in range(4):
        out += propkit._round_corner(c[i], c[i - 1], c[(i + 1) % 4], rs[i], seg)
    return out

def boot_heel():
    w, h, d = 0.045, 0.050, 0.055
    o = lambda k: d_outline(w - 2 * k, d - 2 * k, 0.02 - k, 0.006 - k, 4)
    s = loft([(o(0.0015), -h / 2), (o(0), -h / 2 + 0.0015), (o(0), h / 2 - 0.002), (o(0.0018), h / 2)]).rot(x=-HALF)
    return cut(s, box(0.002, h * 0.8, 0.0012).move(x=-w / 2, y=0.002))

def boot_shaft():
    # the shaft of the boot: open top with a padded collar, dark inside
    return vessel([(0.030, -0.03), (0.0286, 0.017), (0.0294, 0.0235), (0.0272, 0.0286)],
                  wall=0.0035, seg=16, floor=0.036)

part('SingleBoots__sole', f_box(0.13, 0.014, 0.055), boot_sole)
part('SingleBoots__toe', f_sphere(0.035), boot_toe, smooth=50)
part('SingleBoots__heel', f_box(0.045, 0.050, 0.055), boot_heel, smooth=50)
part('SingleBoots__shaft', f_cyl(0.028, 0.030, 0.06), boot_shaft, smooth=50)


# ------------------------------------------------------- folded umbrella --
def folds(a, y, r, lobes=8, depth=0.05):
    """Furled canopy: `lobes` soft folds; the ridges at 0/90/180/270 degrees
    line up with the placer's four dark seam stripes."""
    return r * (1 - depth + depth * math.cos(lobes * a))

UMB_PROFILE = [(0, -0.16), (0.0120, -0.16), (0.0162, -0.09), (0.0184, -0.03), (0.0198, 0.04), (0.0200, 0.085),
               (0.0192, 0.112), (0.0158, 0.124), (0.0082, 0.130), (0.0046, 0.133), (0.0044, 0.16), (0, 0.16)]

def umbrella_body():
    # canopy furled around the shaft: slim at the tip (-y, toward the
    # ferrule), fullest near the rib ends, then the bare shaft to the crook
    s = lathe(UMB_PROFILE, seg=24)
    s = angular(s, lambda a, y, r: folds(a, y, r) if r > 0.0065 else r)
    return s.fit((-0.02, -0.16, -0.02), (0.02, 0.16, 0.02))

def umbrella_seam():
    # the dark seam stripes: the placer flattens this to a blade through the
    # axis, so its outline follows the canopy's taper, a hair proud of it
    return lathe(UMB_PROFILE, seg=8).fit((-0.021, -0.16, -0.021), (0.021, 0.16, 0.021))

def umbrella_tip():
    # the canopy gathered into its top cap, and the pointed ferrule
    s = lathe([(0, -0.010), (0.0140, -0.010), (0.0126, -0.0065), (0.0084, -0.0030), (0.0058, -0.0012),
               (0.0061, 0.0004), (0.0040, 0.0016), (0.0017, 0.0090), (0, 0.010)], seg=16)
    s = angular(s, lambda a, y, r: folds(a, -0.16, r) if y < -0.002 else r)
    return s.fit((-0.014, -0.010, -0.014), (0.014, 0.010, 0.014))

part('FoldedUmbrellas__body', f_cyl(0.020, 0.020, 0.32), umbrella_body, smooth=60)
part('FoldedUmbrellas__tip', f_cone(0.014, 0.020), umbrella_tip, smooth=60)
part('FoldedUmbrellas__seam', f_cyl(0.021, 0.021, 0.32), umbrella_seam, smooth=60)
part('FoldedUmbrellas__handle', f_bounds((-0.025, 0, -0.005), (0.025, 0.025, 0.005)),
     lambda: half_arc_rod(0.020, 0.005, seg=8, n=12), smooth=60)


# ------------------------------------------------------------- pizza box --
def pizza_body():
    # corrugated box: eased edges, the lid's edge line around the sides,
    # two punched vents in the top
    w, h = 0.28, 0.024
    s = plate(w, w, [(0.0015, -h / 2), (0, -h / 2 + 0.0015), (0, h / 2 - 0.0012), (0.0012, h / 2)],
              r=0.003, seg=2).rot(x=-HALF)
    seam = frame_ring(0.29, 0.29, w - 0.0016, w - 0.0016, 0.0012).rot(x=-HALF).move(y=0.004)
    vents = [rbox(0.030, 0.006, 0.010, r=0.0025, seg=2).move(x=sx * 0.08, y=h / 2, z=0.105) for sx in (-1, 1)]
    return cut(s, seam, *vents)

def pizza_lid():
    # open lid: 3 mm board with the front and side flaps folded down
    w = 0.28
    top = rbox(w, 0.003, w, r=0.001, seg=1).move(y=0.0025)
    front = rbox(w, 0.008, 0.0025, r=0.0008, seg=1).move(z=w / 2 - 0.00125)
    sides = [rbox(0.0025, 0.0065, w - 0.02, r=0.0008, seg=1).move(x=sx * (w / 2 - 0.00125), y=0.00075, z=0.005)
             for sx in (-1, 1)]
    return top + front + sides[0] + sides[1]

part('PizzaBoxes__body', f_box(0.28, 0.024, 0.28), pizza_body)
part('PizzaBoxes__lid', f_box(0.28, 0.008, 0.28), pizza_lid)


# ------------------------------------------------------------ single glove --
def glove_palm():
    # a flattened leather palm: puffy, pinched along its seams, narrower at
    # the wrist (-z), knuckle bumps where the fingers join (+z)
    W, D, T = 0.055, 0.045, 0.014
    s = slab(W, D, T, 7, 6).rot(x=-HALF)
    def shape(p):
        u = (p.z + D / 2) / D
        p.x *= 0.86 + 0.14 * u
        t = max(abs(p.x) / (W / 2 * (0.86 + 0.14 * u)), abs(p.z) / (D / 2))
        g = (1 - min(1.0, t) ** 4) ** 0.25
        p.y *= max(g, 0.22)
        if p.y > 0:
            k = sum(math.exp(-((p.x - fx) / 0.004) ** 2) for fx in (-0.018, -0.006, 0.006, 0.018))
            p.y += 0.0011 * k * math.exp(-((p.z - 0.015) / 0.005) ** 2)
        return p
    s = noisy(s.warp(shape), 0.0005, scale=150, seed=4)
    return s.fit((-W / 2, -T / 2, -D / 2), (W / 2, T / 2, D / 2))

def glove_finger():
    # a stuffed finger: round tip at +z, two knuckle creases, slight curl
    r, L = 0.005, 0.035
    prof = [(0, -L / 2)]
    for k in (0.55, 0.85):
        prof.append((r * math.sin(k * HALF), -L / 2 + r * (1 - math.cos(k * HALF))))
    for y, f in ((-0.010, 1.0), (-0.0005, 1.0), (0.0015, 0.9), (0.0035, 1.0), (0.0085, 0.97), (0.010, 0.88),
                 (0.0118, 0.97)):
        prof.append((r * f, y))
    for k in (0.4, 0.75, 0.95):
        prof.append((r * math.cos(k * HALF), L / 2 - r + r * math.sin(k * HALF)))
    prof.append((0, L / 2))
    s = lathe(prof, seg=10).rot(x=HALF)
    s = s.warp(lambda p: Vector((p.x, p.y - 0.0012 * ((p.z + L / 2) / L) ** 2, p.z)))
    return s.fit((-r, -r, -L / 2), (r, r, L / 2))

def glove_cuff():
    # knitted cuff: ribbed outside, a rolled edge at the opening (-y)
    loop = [(0.0222, -0.010), (0.0240, -0.0078), (0.0232, -0.0045), (0.0232, 0.010), (0.0206, 0.010),
            (0.0206, -0.0085)]
    s = weld(lathe(loop + loop[:1], seg=22, cap_bottom=False, cap_top=False))
    s = angular(s, lambda a, y, r: r * (1 + 0.035 * math.cos(11 * a)) if r > 0.0228 and y > -0.0046 else r)
    return s.fit((-0.024, -0.010, -0.024), (0.024, 0.010, 0.024))

part('SingleGloves__palm', f_box(0.055, 0.014, 0.045), glove_palm, smooth=180)
part('SingleGloves__finger', f_box(0.010, 0.010, 0.035), glove_finger, smooth=180)
part('SingleGloves__cuff', f_cyl(0.024, 0.024, 0.020), glove_cuff, smooth=60)


# ------------------------------------------------------- cigarette pack --
def cig_pack():
    # card pack with eased edges; the flip-top cut runs across the front and
    # slants up both sides to the hinge at the back
    w, h, d = 0.05, 0.08, 0.020
    s = plate(w, h, [(0.0006, -d / 2), (0, -d / 2 + 0.0006), (0, d / 2 - 0.0006), (0.0006, d / 2)], r=0.0015, seg=2)
    blade = box(w + 0.01, 0.0007, 0.022).rot(x=-0.46).move(y=0.024, z=0.002)
    blade = cut(blade, box(w - 0.0012, 0.02, d - 0.0012).move(y=0.024))
    return cut(s, blade)

def cig_lid():
    # the open lid seen from inside: a thin card with folded side and top edges
    w, h, d = 0.05, 0.020, 0.004
    back = rbox(w, h, 0.0008, r=0.0003, seg=1).move(z=-d / 2 + 0.0004)
    top = rbox(w, 0.0008, d, r=0.0003, seg=1).move(y=h / 2 - 0.0004)
    sides = [rbox(0.0008, h - 0.001, d, r=0.0003, seg=1).move(x=sx * (w / 2 - 0.0004), y=0.0005) for sx in (-1, 1)]
    return back + top + sides[0] + sides[1]

part('CigarettePacks__body', f_box(0.05, 0.08, 0.020), cig_pack)
part('CigarettePacks__lid', f_box(0.05, 0.020, 0.004), cig_lid)


# ------------------------------------------------------------- matchbook --
def matchbook():
    # folded card cover: a rounded fold at -z, the tucked flap and striker
    # strip across the +z end
    body = rbox(0.030, 0.0034, 0.037, r=0.0007, seg=1).move(y=-0.0008, z=0.0015)
    fold = cyl(0.0017, 0.0017, 0.030, seg=10, bevel_r=0.0004, bevel_seg=1).rot(z=HALF).move(y=-0.0008, z=-0.0183)
    flap = rbox(0.030, 0.0014, 0.010, r=0.0005, seg=1).move(y=0.0018, z=0.015)
    return (body + fold + flap).fit((-0.015, -0.0025, -0.020), (0.015, 0.0025, 0.020))

def match_head():
    # dipped head: a bulb that tapers onto the stick (-x)
    s = lathe([(0, -0.0015), (0.0009, -0.0011), (0.0014, -0.0002), (0.0015, 0.0004), (0.0012, 0.0011),
               (0.0006, 0.0014), (0, 0.0015)], seg=8).rot(z=-HALF)
    return s.fit((-0.0015, -0.0015, -0.0015), (0.0015, 0.0015, 0.0015))

part('Matchbooks__booklet', f_box(0.030, 0.005, 0.040), matchbook)
part('Matchbooks__stick', f_cyl(0.0008, 0.0008, 0.024), lambda: rbox(0.0016, 0.024, 0.0016, r=0.0002, seg=1))
part('Matchbooks__head', f_sphere(0.0015), match_head, smooth=60)


# ----------------------------------------------------- broken sunglasses --
def glasses_rim():
    # acetate rim: flat front and back, a lens groove on the inside
    sec = [(0.0030, -0.0017), (0.0030, 0.0017), (0.0008, 0.0030), (-0.0030, 0.0012), (-0.0030, -0.0012),
           (0.0008, -0.0030)]
    s = weld(arc_sweep(sec, 0.024, 0.0, TAU, 22, caps=False))
    return s.fit((-0.027, -0.027, -0.003), (0.027, 0.027, 0.003))

def glasses_arm(y0, y1, broken=False):
    """A temple from y0 (ear tip) to y1, laid along +y: flat bar tapering
    toward the ear, the tip curling in x; a snapped end if broken."""
    rings = []
    n = 9
    for i in range(n + 1):
        t = i / n
        y = y0 + (y1 - y0) * t
        g = (y + 0.05) / 0.10                  # 0 at the ear tip, 1 at the hinge
        w = 0.0024 + 0.0008 * g
        th = 0.0016 + 0.0006 * g
        if g < 0.12:
            w, th = w * 1.12, th * 1.12        # rounded tip
        cx = -0.0005 * max(0.0, (0.3 - g) / 0.3) ** 2
        pts = rr(w, th, min(w, th) * 0.45, 1, cx=cx)
        rings.append((pts, y))
    s = loft(rings)
    if broken:
        k = len(rings[0][0])
        base = len(s.v) - k
        jag = [0.0, 0.0012, -0.0004, 0.0016, 0.0003, -0.0008, 0.001, 0.0002]
        for j in range(k):
            s.v[base + j] = s.v[base + j] + Vector((0, 0, jag[j % len(jag)]))
    return s.rot(x=-HALF)

def glasses_temple():
    arm = glasses_arm(-0.05, 0.044)
    hinge = rbox(0.004, 0.006, 0.004, r=0.0006, seg=1).move(y=0.047)
    return (arm + hinge).fit((-0.002, -0.05, -0.002), (0.002, 0.05, 0.002))

def glasses_broken():
    arm = glasses_arm(-0.03, 0.03, broken=True)
    return arm.fit((-0.002, -0.03, -0.002), (0.002, 0.03, 0.002))

def glasses_bridge():
    s = rod([(-0.0003, -0.007, 0), (0.0003, 0, 0), (-0.0003, 0.007, 0)], 0.0017, seg=6, smooth=1, caps=1)
    return s.fit((-0.002, -0.007, -0.002), (0.002, 0.007, 0.002))

part('BrokenSunglasses__rim', f_torus(0.024, 0.003), glasses_rim, smooth=60)
part('BrokenSunglasses__bridge', f_cyl(0.002, 0.002, 0.014), glasses_bridge, smooth=60)
part('BrokenSunglasses__temple', f_cyl(0.002, 0.002, 0.10), glasses_temple, smooth=50)
part('BrokenSunglasses__broken', f_cyl(0.002, 0.002, 0.06), glasses_broken, smooth=50)


# ------------------------------------------------------------ TP roll core --
def tp_core():
    # hollow cardboard core, 1.2 mm wall, the ends slightly frayed and one
    # side pressed in a little where it was stepped on
    y = [-0.05 + 0.1 * i / 7 for i in range(8)]
    loop = [(0.0236, -0.05)] + [(0.024, yy) for yy in y[1:-1]] + [(0.0236, 0.05), (0.0228, 0.05), (0.0228, -0.05)]
    s = weld(lathe(loop + loop[:1], seg=24, cap_bottom=False, cap_top=False))
    def dent(p):
        k = math.exp(-((p.y - 0.01) / 0.03) ** 2) * max(0.0, p.x / 0.024) ** 3
        p.x -= 0.0022 * k
        return p
    s = s.warp(dent)
    return s.fit((-0.024, -0.05, -0.024), (0.024, 0.05, 0.024))

part('UnrolledTPRolls__tube', f_cyl(0.024, 0.024, 0.10), tp_core, smooth=50)


# ----------------------------------------------------- broken flashlight --
def flash_body():
    # aluminium barrel: thread ring at the head end (-y), a knurled grip,
    # a stepped tail cap with a rubber button at +y
    prof = [(0, -0.06), (0.0132, -0.06), (0.0138, -0.0565), (0.0138, -0.053), (0.0131, -0.051)]
    y = -0.026
    for i in range(3):
        prof += [(0.0133, y), (0.0125, y + 0.0009), (0.0125, y + 0.0032), (0.0133, y + 0.0041)]
        y += 0.016
    prof += [(0.0138, 0.032), (0.0140, 0.052), (0.0134, 0.0585), (0.0070, 0.0595), (0, 0.0600)]
    s = lathe(prof, seg=13)
    return s.fit((-0.014, -0.06, -0.014), (0.014, 0.06, 0.014))

def flash_head():
    # flared head: scalloped bezel ring, a deep reflector cone under the lens
    s = lathe([(0, -0.012), (0.0140, -0.012), (0.0182, 0.003), (0.0200, 0.0055), (0.0200, 0.012), (0.0170, 0.012),
               (0.0050, 0.0), (0, 0.0)], seg=16)
    s = angular(s, lambda a, y, r: r * (0.985 + 0.015 * math.cos(8 * a)) if r > 0.0195 else r)
    return s.fit((-0.020, -0.012, -0.020), (0.020, 0.012, 0.020))

part('BrokenFlashlights__body', f_cyl(0.014, 0.014, 0.12), flash_body)
part('BrokenFlashlights__head', f_cyl(0.020, 0.014, 0.024), flash_head)


# --------------------------------------------------------------- shoe box --
def shoe_body():
    # card box body: eased edges, a finger hole pressed into each end
    w, h, d = 0.34, 0.12, 0.20
    s = plate(w, d, [(0.0015, -h / 2), (0, -h / 2 + 0.0015), (0, h / 2 - 0.001), (0.001, h / 2)], r=0.003).rot(x=-HALF)
    holes = [rbox(0.012, 0.018, 0.05, r=0.006, seg=2).move(x=sx * w / 2, y=0.03) for sx in (-1, 1)]
    return cut(s, *holes)

def shoe_lid():
    # lid: a top with a 2 cm skirt, hollow underneath
    w, h, d = 0.36, 0.020, 0.22
    s = plate(w, d, [(0.0008, -h / 2), (0, -h / 2 + 0.0008), (0, h / 2 - 0.0015), (0.0015, h / 2)], r=0.003).rot(x=-HALF)
    return cut(s, box(w - 0.004, h, d - 0.004).move(y=-0.0018))

part('ShoeBoxes__body', f_box(0.34, 0.12, 0.20), shoe_body)
part('ShoeBoxes__lid', f_box(0.36, 0.020, 0.22), shoe_lid)


# -------------------------------------------------------------- coat rack --
def rack_rail():
    # hardwood rail: flat back on the wall, a rounded top front edge and a
    # bead line along the front, chamfered ends
    zy = [(-0.015, -0.010), (0.0125, -0.010), (0.015, -0.0075), (0.015, -0.0022), (0.0138, -0.0012),
          (0.0138, 0.0012), (0.015, 0.0022), (0.015, 0.0045), (0.0135, 0.0082), (0.0105, 0.010), (-0.015, 0.010)]
    s = prism([(-z, y) for z, y in zy], 0.60, bevel_r=0.0015).rot(y=HALF)
    return s.fit((-0.30, -0.010, -0.015), (0.30, 0.010, 0.015))

def rack_peg():
    # brass peg: a flange on the rail, a slim shank tapering out (+y)
    return lathe([(0, -0.0125), (0.0040, -0.0125), (0.0040, -0.0108), (0.0029, -0.0090), (0.0025, 0.0118),
                  (0.0020, 0.0125), (0, 0.0125)], seg=8)

def rack_knob():
    # mushroom knob on the peg end, its dome facing out (+z)
    s = lathe([(0, -0.006), (0.0045, -0.0055), (0.006, -0.002), (0.0055, 0.0025), (0.003, 0.0055), (0, 0.006)], seg=10)
    return on_z(s)

def coat_shoulders():
    # the coat's shoulder line: high at the neck, sloping to both sides,
    # soft front and back
    W, H, D = 0.13, 0.025, 0.06
    s = slab(W, D, H, 8, 4).rot(x=-HALF)
    def shape(p):
        u = abs(p.x) / (W / 2)
        if p.y > 0:
            p.y -= 0.009 * u ** 1.6
        t = min(1.0, abs(p.z) / (D / 2))
        p.y = -H / 2 + (p.y + H / 2) * max(0.35, (1 - t ** 3) ** (1 / 3))
        return p
    return s.warp(shape).fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))

def coat_body():
    # hanging coat front: a little A-line toward the hem, soft edges, the
    # front opening down the middle and the lapel lines
    W, H, D = 0.15, 0.28, 0.05
    s = slab(W, H, D, 6, 10)
    def shape(p):
        v = (p.y + H / 2) / H                     # 0 at the hem, 1 at the top
        p.x *= 0.88 + 0.12 * (1 - v)
        t = min(1.0, abs(p.x) / (W / 2 * (0.88 + 0.12 * (1 - v))))
        p.z *= max(0.45, (1 - t ** 4) ** 0.25)
        if p.z > 0 and v < 0.6:
            p.z += 0.002 * math.sin(p.x / W * 5 * math.pi) * (0.6 - v)
        return p
    s = s.warp(shape)
    front = D / 2
    lines = [box(0.0016, H, 0.004).move(y=-0.06, z=front)]
    for sx in (-1, 1):
        lines.append(box(0.0016, 0.13, 0.006).rot(z=sx * 0.42).move(x=sx * 0.022, y=0.09, z=front))
    s = cut(s, *lines)
    return s.fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))

def coat_sleeve():
    # a hanging sleeve: soft oval tube, a slight bend at the elbow, a turned
    # cuff at the bottom
    rings = []
    for y, k, dx in ((-0.110, 0.80, 0.0), (-0.108, 0.97, 0.0), (-0.090, 1.0, 0.0), (-0.086, 0.93, 0.0),
                     (-0.03, 0.92, 0.002), (0.01, 0.95, 0.003), (0.06, 0.97, 0.001), (0.095, 0.98, 0.0),
                     (0.106, 0.85, 0.0), (0.110, 0.55, 0.0)):
        rings.append((ellipse(0.0175 * k, 0.0225 * k, 10, cx=dx), y))
    s = loft(rings).rot(x=-HALF)
    return s.fit((-0.0175, -0.11, -0.0225), (0.0175, 0.11, 0.0225))

def coat_collar():
    # a turned-down collar: a U band round the back of the neck, open at the front
    n = 9
    outer = [(0.045 * math.cos(math.pi * i / n), -(0.0225 - 0.045 * math.sin(math.pi * i / n))) for i in range(n + 1)]
    inner = [(0.030 * math.cos(math.pi * i / n), -(0.0225 - 0.030 * math.sin(math.pi * i / n))) for i in range(n, -1, -1)]
    s = prism(outer + inner, 0.010, bevel_r=0.002).rot(x=-HALF)
    return s.fit((-0.045, -0.005, -0.0225), (0.045, 0.005, 0.0225))

def scarf_strip():
    # knitted scarf end: rows of knit ridges, a gentle wave, soft edges
    W, H, D = 0.020, 0.24, 0.008
    s = slab(W, H, D, 3, 16)
    def shape(p):
        t = min(1.0, abs(p.x) / (W / 2))
        p.z *= max(0.4, (1 - t ** 4) ** 0.25)
        p.z += 0.0006 * math.cos(p.y / 0.015 * TAU) * (1 if p.z > 0 else 0.5)
        p.z += 0.0015 * math.sin(p.y / H * 2.2 * math.pi)
        return p
    return s.warp(shape).fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))

def scarf_fringe():
    # the knotted band the tassels hang from, six tassels of uneven length
    s = taper(box(0.020, 0.0022, 0.006), top=0.9, along="y", axes="z").move(y=0.0029)
    for x, L in ((-0.0082, 0.0068), (-0.0049, 0.0078), (-0.0016, 0.0070), (0.0016, 0.0078), (0.0049, 0.0064),
                 (0.0082, 0.0074)):
        s = s + taper(box(0.0017, L, 0.0030), top=1.6, along='y', axes='xz').move(x=x, y=0.004 - L / 2)
    return s.fit((-0.010, -0.004, -0.003), (0.010, 0.004, 0.003))

part('CoatRacks__rail', f_box(0.60, 0.020, 0.030), rack_rail)
part('CoatRacks__peg', f_cyl(0.004, 0.004, 0.025), rack_peg, smooth=50)
part('CoatRacks__knob', f_sphere(0.006), rack_knob, smooth=60)
part('CoatRacks__shoulders', f_box(0.13, 0.025, 0.06), coat_shoulders, smooth=180)
part('CoatRacks__coat', f_box(0.15, 0.28, 0.05), coat_body, smooth=60)
part('CoatRacks__sleeve', f_box(0.035, 0.22, 0.045), coat_sleeve, smooth=180)
part('CoatRacks__collar', f_box(0.09, 0.010, 0.045), coat_collar, smooth=50)
part('CoatRacks__scarf', f_box(0.020, 0.24, 0.008), scarf_strip, smooth=180)
part('CoatRacks__fringe', f_box(0.020, 0.008, 0.006), scarf_fringe)


# -------------------------------------------------------- crumpled tissue --
def tissue_lump():
    # a scrunched tissue: a ball pushed in and out by folds, crisp creases
    s = sphere(0.0125, seg=12, rings=8)
    def crumple(p):
        n = noise.noise(p * 140 + Vector((3.1, 7.7, 1.3)))
        m = noise.noise(p * 420 + Vector((5.0, 2.0, 9.0)))
        k = 1 + 0.32 * n + 0.10 * m
        return p * k
    s = s.warp(crumple)
    return s.fit((-0.0125, -0.0125, -0.0125), (0.0125, 0.0125, 0.0125))

part('CrumpledTissues__chunk', f_box(0.025, 0.025, 0.025), tissue_lump, smooth=28)


# ----------------------------------------------------------- vinyl record --
def vinyl_disc():
    # LP: rounded bead at the edge, lead-in, four band gaps across the
    # grooved area, the run-out and a thicker label seat
    prof = [(0, -0.0015), (0.1485, -0.0015), (0.1500, 0.0), (0.1490, 0.0015), (0.1440, 0.0012), (0.1420, 0.0009)]
    for r in (0.125, 0.108, 0.091, 0.075):
        prof += [(r, 0.0006), (r - 0.0015, 0.0009)]
    prof += [(0.058, 0.0008), (0.042, 0.0012), (0, 0.0012)]
    return lathe(prof, seg=36)

def vinyl_label():
    return lathe([(0, -0.0016), (0.0398, -0.0016), (0.0400, 0.0010), (0.0394, 0.0016), (0, 0.0016)], seg=28)

def vinyl_sleeve():
    # card album jacket: puffed over the record inside, the open edge (+x)
    # parted a little, soft worn corners
    W, T = 0.32, 0.005
    s = slab(W, W, T, 12, 12).rot(x=-HALF)
    def shape(p):
        rho = math.hypot(p.x, p.z) / 0.15
        c = 0.5 - 0.5 * math.tanh((rho - 0.97) * 12)
        p.y *= 0.55 + 0.45 * c
        if p.x > 0.12 and p.y > 0:
            p.y += 0.0010 * (p.x - 0.12) / 0.04
        k = max(abs(p.x), abs(p.z)) / (W / 2)
        if abs(p.x) > 0.13 and abs(p.z) > 0.13:
            p.x *= 0.985
            p.z *= 0.985
        return p
    s = noisy(s.warp(shape), 0.0003, scale=30, seed=2, axis=(0, 1, 0))
    return s.fit((-W / 2, -T / 2, -W / 2), (W / 2, T / 2, W / 2))

part('VinylRecords__disc', f_cyl(0.15, 0.15, 0.003), vinyl_disc, smooth=30)
part('VinylRecords__label', f_cyl(0.04, 0.04, 0.0032), vinyl_label)
part('VinylRecords__sleeve', f_box(0.32, 0.005, 0.32), vinyl_sleeve, smooth=60)


# ------------------------------------------------------ gumball machine --
def gumball_base():
    # cast pedestal: plinth and cap lines, a panelled front for the coin
    # mechanism, the delivery chute, a round seat for the globe
    w, h = 0.14, 0.24
    q = lambda i: rr(w - 2 * i, w - 2 * i, 0.012 - i, 2)
    rings = [(q(0.002), -0.12), (q(0), -0.118), (q(0), -0.101), (q(0.0022), -0.100), (q(0), -0.099),
             (q(0), 0.089), (q(0.0022), 0.090), (q(0), 0.091), (q(0), 0.117), (q(0.0025), 0.12)]
    s = loft(rings).rot(x=-HALF)
    panel = frame_ring(0.094, 0.15, 0.090, 0.146, 0.004, z=w / 2)
    chute = rbox(0.052, 0.022, 0.03, r=0.004, seg=2).move(y=-0.082, z=w / 2)
    seat = cyl(0.052, 0.052, 0.008, seg=24, bevel_r=0).move(y=h / 2)
    return cut(s, panel, chute, seat).fit((-w / 2, -h / 2, -w / 2), (w / 2, h / 2, w / 2))

def gumball_knob():
    # turn handle: a round boss on the machine, a neck, a wing grip (+y out)
    boss = cyl(0.010, 0.010, 0.005, seg=14, bevel_r=0.0012, bevel_seg=1).move(y=-0.005)
    neck = cyl(0.0045, 0.0045, 0.004, seg=10, bevel_r=0).move(y=-0.0005)
    wing = rbox(0.020, 0.0065, 0.006, r=0.0018, seg=1).move(y=0.00425)
    return (boss + neck + wing).fit((-0.010, -0.0075, -0.010), (0.010, 0.0075, 0.010))

part('GumBallDispensers__base', f_box(0.14, 0.24, 0.14), gumball_base)
part('GumBallDispensers__ball', f_sphere(0.012), lambda: sphere(0.012, seg=12, rings=7), smooth=180)
part('GumBallDispensers__knob', f_cyl(0.010, 0.010, 0.015), gumball_knob)


# --------------------------------------------------------- cross stitch --
def stitch_hoop():
    # a pair of wooden hoops, the outer one set a little behind the inner
    sec = [(0.0120, -0.0110), (0.0120, 0.0090), (0.0110, 0.0100), (0.0, 0.0100), (0.0, 0.0120),
           (-0.0110, 0.0120), (-0.0120, 0.0110), (-0.0120, -0.0090), (0.0, -0.0100), (0.0110, -0.0120)]
    s = weld(arc_sweep(sec, 0.10, 0.0, TAU, 27, caps=False))
    return s.fit((-0.112, -0.112, -0.012), (0.112, 0.112, 0.012))

def stitch_nut():
    # brass thumb nut: ridged grip, chamfered faces
    s = lathe([(0, -0.005), (0.0071, -0.005), (0.008, -0.0038), (0.008, 0.0038), (0.0071, 0.005), (0, 0.005)], seg=12)
    s = angular(s, lambda a, y, r: r * (0.93 + 0.07 * math.cos(6 * a)) if r > 0.0075 else r)
    return s.fit((-0.008, -0.005, -0.008), (0.008, 0.005, 0.008))

part('CrossStitches__hoop', f_torus(0.10, 0.012), stitch_hoop)
part('CrossStitches__clamp', f_cyl(0.008, 0.008, 0.010), stitch_nut)


# ------------------------------------------------------------ watercolor --
def moulded_frame(w, h, d, prof, mitre=True):
    """Picture frame moulding lofted from the outer edge in; the centre is
    a flat panel behind the art. prof = [(inset, z), ...]."""
    s = loft([(rr(w - 2 * i, h - 2 * i, 0.0006, 1), z) for i, z in prof])
    if mitre:
        cutters = [box(0.0008, 0.07, 0.006).rot(z=-sx * sy * math.pi / 4).move(x=sx * w / 2, y=sy * h / 2, z=d / 2)
                   for sx in (-1, 1) for sy in (-1, 1)]
        s = cut(s, *cutters)
    return s

def watercolor_frame():
    d = 0.014
    return moulded_frame(0.34, 0.26, d, [(0, -0.007), (0, 0.0015), (0.0015, 0.0045), (0.004, 0.007), (0.0075, 0.0065),
                                          (0.011, 0.004), (0.016, 0.0045), (0.019, 0.0055), (0.022, 0.003),
                                          (0.030, 0.001)])

def watercolor_mat():
    # mat board with a bevel-cut window round the painting
    w, h, d = 0.28, 0.20, 0.010
    s = plate(w, h, [(0.0005, -d / 2), (0, -d / 2 + 0.0005), (0, d / 2 - 0.0005), (0.0005, d / 2)], r=0.001, seg=1)
    win = loft([(rr(0.221, 0.161, 0.0003, 1), d / 2 - 0.0016), (rr(0.2245, 0.1645, 0.0003, 1), d / 2),
                (rr(0.227, 0.167, 0.0003, 1), d / 2 + 0.002)])
    return cut(s, win)

part('Watercolors__frame', f_box(0.34, 0.26, 0.014), watercolor_frame)
part('Watercolors__mat', f_box(0.28, 0.20, 0.010), watercolor_mat)


# -------------------------------------------------------------- tote bag --
def superellipse(a, b, n, p=6.0):
    return [(a * math.copysign(abs(math.cos(TAU * i / n)) ** (2 / p), math.cos(TAU * i / n)),
             b * math.copysign(abs(math.sin(TAU * i / n)) ** (2 / p), math.sin(TAU * i / n))) for i in range(n)]

def tote_body():
    # soft canvas tote, open at the top: bellied sides, a doubled hem, side
    # gussets pinched in toward the mouth
    W, H, D = 0.16, 0.16, 0.10
    def ring(k, y, pinch=0.0, t=0.0):
        a, b = W / 2 * k - t, D / 2 * k - t
        pts = superellipse(a, b, 20)
        return ([(x * (1 - pinch * (1 - abs(z) / b) ** 2), z) for x, z in pts], y)
    spec = [(0.96, -H / 2, 0.0), (1.0, -H / 2 + 0.004, 0.0), (1.0, -0.02, 0.0), (0.985, 0.05, 0.04),
            (0.985, 0.068, 0.07), (1.0, 0.071, 0.07), (1.0, H / 2, 0.07)]
    outer = loft([ring(k, y, pc) for k, y, pc in spec]).rot(x=-HALF)
    inner = loft([ring(1.0, -H / 2 + 0.004, 0.0, 0.0035)] + [ring(k, y, pc, 0.0035) for k, y, pc in spec[2:]] +
                 [ring(1.0, H / 2 + 0.01, 0.07, 0.0035)]).rot(x=-HALF)
    return cut(outer, inner).fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))

def tote_strap():
    # flat webbing strap bent over into a handle
    sec = [(0.0015, -0.0045), (0.0015, 0.0045), (0.0, 0.0050), (-0.0015, 0.0045), (-0.0015, -0.0045), (0.0, -0.0050)]
    return arc_sweep(sec, 0.0335, 0.0, math.pi, 16).fit((-0.035, 0, -0.005), (0.035, 0.035, 0.005))

def tool_handle():
    # a fluted tool handle (butt up), ferrule and shank sunk in the bag
    s = lathe([(0, -0.055), (0.0022, -0.055), (0.0022, -0.034), (0.0040, -0.033), (0.0046, -0.029), (0.0056, -0.025),
               (0.0060, -0.010), (0.0057, 0.034), (0.0060, 0.047), (0.0050, 0.0540), (0, 0.055)], seg=12)
    s = angular(s, lambda a, y, r: r * (0.93 + 0.07 * math.cos(6 * a)) if -0.026 < y < 0.048 else r)
    return s.fit((-0.006, -0.055, -0.006), (0.006, 0.055, 0.006))

part('ToteBags__body', f_box(0.16, 0.16, 0.10), tote_body, smooth=60)
part('ToteBags__strap', f_bounds((-0.035, 0, -0.005), (0.035, 0.035, 0.005)), tote_strap)
part('ToteBags__tool', f_cyl(0.006, 0.006, 0.11), tool_handle, smooth=50)


# ------------------------------------------------------- coat hook pair --
def hook_board():
    # oak board: ogee front edge, two counterbored screws at the ends
    w, h, d = 0.16, 0.05, 0.015
    s = plate(w, h, [(0, -d / 2), (0, 0.002), (0.0012, 0.0045), (0.0035, 0.0065), (0.0045, d / 2)], r=0.004)
    bores = [cyl(0.0042, 0.0042, 0.004, seg=12, bevel_r=0).rot(x=HALF).move(x=sx * 0.068, z=d / 2) for sx in (-1, 1)]
    s = cut(s, *bores)
    for sx in (-1, 1):
        s = s + screw_head(0.0032, 0.0012).rot(x=HALF).move(x=sx * 0.068, z=d / 2 - 0.0017)
    return s

def hook_rosette():
    return lathe([(0, -0.003), (0.008, -0.003), (0.0078, -0.0012), (0.0055, 0.0018), (0.0045, 0.003), (0, 0.003)], seg=12)

def hook_arm():
    return lathe([(0, -0.02), (0.005, -0.02), (0.0049, -0.017), (0.0043, -0.012), (0.0037, 0.016), (0.0035, 0.0195),
                  (0.0025, 0.020), (0, 0.020)], seg=12)

def hook_curve():
    # the bent end of the hook, finished with a ball tip (+x end is free)
    s = half_arc_rod(0.012, 0.0036, seg=7, n=8)
    s = s + sphere(0.0042, 8, 4).move(0.0118, 0.0042, 0)
    return s.fit((-0.016, 0, -0.004), (0.016, 0.016, 0.004))

part('CoatHooksPair__board', f_box(0.16, 0.05, 0.015), hook_board)
part('CoatHooksPair__rosette', f_cyl(0.008, 0.008, 0.006), hook_rosette, smooth=50)
part('CoatHooksPair__arm', f_cyl(0.005, 0.005, 0.040), hook_arm, smooth=50)
part('CoatHooksPair__curve', f_bounds((-0.016, 0, -0.004), (0.016, 0.016, 0.004)), hook_curve, smooth=60)


# -------------------------------------------------------- cast iron pan --
def skillet_body():
    # the pan: flat base on the wall side (-y), flared wall, rolled rim;
    # the floor is raised to meet the cooking-surface disc
    return vessel([(0.076, -0.015), (0.080, -0.0135), (0.0835, -0.006), (0.0875, 0.006), (0.0893, 0.0132)],
                  wall=0.005, seg=32, floor=0.0185)

def skillet_inside():
    # the seasoned cooking surface: a shallow dish with a soft edge
    return lathe([(0, -0.0075), (0.0755, -0.0075), (0.0760, 0.0050), (0.0735, 0.0075), (0.0690, 0.0062),
                  (0.0620, 0.0045), (0, 0.0040)], seg=32)

def skillet_handle():
    # tapering handle with a channel along its face (+z) and a teardrop
    # end pierced for hanging
    rings = []
    for x, h, d in ((-0.100, 0.014, 0.014), (-0.080, 0.0135, 0.0135), (-0.020, 0.0115, 0.013), (0.055, 0.0100, 0.012),
                    (0.080, 0.0130, 0.012), (0.092, 0.0120, 0.0115), (0.098, 0.0075, 0.009), (0.100, 0.003, 0.005)):
        rings.append((rr(d, h, min(h, d) * 0.3, 2), x))
    s = loft(rings).rot(y=HALF)
    s = cut(s, rbox(0.15, 0.0045, 0.006, r=0.002, seg=1).move(x=-0.005, z=0.0075),
            cyl(0.0034, 0.0034, 0.03, seg=12, bevel_r=0).rot(x=HALF).move(x=0.086))
    return s.fit((-0.10, -0.007, -0.007), (0.10, 0.007, 0.007))

part('CastIronPans__body', f_cyl(0.09, 0.09, 0.030), skillet_body)
part('CastIronPans__inner', f_cyl(0.076, 0.076, 0.015), skillet_inside)
part('CastIronPans__handle', f_box(0.20, 0.014, 0.014), skillet_handle)
part('CastIronPans__loop', f_torus(0.006, 0.002), lambda: torus(0.006, 0.002, 12, 4), smooth=60)


# ------------------------------------------------------------ vintage map --
part('VintageMaps__frame', f_box(0.30, 0.22, 0.014),
     lambda: moulded_frame(0.30, 0.22, 0.014, [(0, -0.007), (0, 0.002), (0.001, 0.0045), (0.003, 0.007), (0.009, 0.0068),
                                               (0.012, 0.0055), (0.014, 0.0047), (0.016, 0.0042), (0.020, 0.0025),
                                               (0.024, 0.001)]))


# ------------------------------------------------------------- mason jar --
def jar_band():
    # screw band: knurled skirt, rolled bottom edge, a shoulder turning in
    s = lathe([(0, -0.005), (0.0285, -0.005), (0.0300, -0.0038), (0.0300, 0.0028), (0.0290, 0.0045), (0.0265, 0.005),
               (0, 0.005)], seg=24)
    s = angular(s, lambda a, y, r: r * (0.985 + 0.015 * math.cos(12 * a)) if r > 0.0296 and -0.0036 < y < 0.0026 else r)
    return s.fit((-0.030, -0.005, -0.030), (0.030, 0.005, 0.030))

def jar_lid():
    # flat lid: rounded edge, the sealing ring, a pressed centre button
    return lathe([(0, -0.0015), (0.0280, -0.0015), (0.0280, 0.0005), (0.0265, 0.0015), (0.0236, 0.0010), (0.0220, 0.0015),
                  (0, 0.0011)], seg=24)

def jar_fill():
    # preserves: rounded to the jar's inside at the bottom, a lumpy top
    s = lathe([(0, -0.035), (0.028, -0.035), (0.0315, -0.032), (0.032, -0.027), (0.032, 0.031), (0.0303, 0.0338),
               (0.022, 0.0348), (0.011, 0.0350), (0, 0.0345)], seg=16)
    def lumpy(p):
        if p.y > 0.032:
            p.y += 0.0012 * noise.noise(p * 120 + Vector((1.0, 2.0, 3.0)))
        return p
    return s.warp(lumpy).fit((-0.032, -0.035, -0.032), (0.032, 0.035, 0.032))

part('MasonJars__band', f_cyl(0.030, 0.030, 0.010), jar_band)
part('MasonJars__lid', f_cyl(0.028, 0.028, 0.003), jar_lid)
part('MasonJars__fill', f_cyl(0.032, 0.032, 0.07), jar_fill, smooth=60)


# ----------------------------------------------------------------- broom --
def broom_handle():
    # turned handle: domed top pierced for a hanging cord, a tenon at the
    # bottom where it enters the bristles
    s = lathe([(0, -0.30), (0.0075, -0.30), (0.0082, -0.296), (0.0100, -0.292), (0.0100, 0.285), (0.0088, 0.2955),
               (0.0052, 0.2995), (0, 0.30)], seg=12)
    return cut(s, cyl(0.0028, 0.0028, 0.03, seg=10, bevel_r=0).rot(x=HALF).move(y=0.283))

def broom_clip():
    # spring-steel clip: a flat strip bent into a half ring
    sec = [(0.0008, -0.003), (0.0008, 0.003), (-0.0008, 0.003), (-0.0008, -0.003)]
    return arc_sweep(sec, 0.0162, 0.0, math.pi, 14).fit((-0.017, 0, -0.003), (0.017, 0.017, 0.003))

def broom_bristles():
    # besom head: a bundle flaring from the bound neck to a ragged sweep,
    # strands showing as fine ridges, one stitched band
    s = lathe([(0, -0.064), (0.055, -0.070), (0.0545, -0.058), (0.047, -0.030), (0.036, 0.0), (0.026, 0.022),
               (0.0215, 0.034), (0.0205, 0.050), (0.0165, 0.060), (0, 0.070)], seg=32)
    def strands(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6:
            return p
        a = math.atan2(p.x, p.z)
        loose = min(1.0, max(0.0, (0.034 - p.y) / 0.05))
        k = 1 + loose * (0.07 * noise.noise(Vector((math.cos(a) * 9, math.sin(a) * 9, p.y * 6)))
                         + 0.05 * math.cos(16 * a + p.y * 20))
        p.x *= k
        p.z *= k
        if p.y < -0.06:
            p.y += 0.004 * noise.noise(Vector((p.x * 90, 1.7, p.z * 90)))
        return p
    return s.warp(strands).fit((-0.055, -0.07, -0.055), (0.055, 0.07, 0.055))

def broom_twine():
    # twine wound round the neck: a slightly lumpy cord
    sec = [(0.003 * math.cos(TAU * j / 6), 0.003 * math.sin(TAU * j / 6)) for j in range(6)]
    s = weld(arc_sweep(sec, 0.014, 0.0, TAU, 22, caps=False))
    return noisy(s, 0.0004, scale=500, seed=9).fit((-0.017, -0.017, -0.003), (0.017, 0.017, 0.003))

part('WallBrooms__handle', f_cyl(0.010, 0.010, 0.60), broom_handle)
part('WallBrooms__clip', f_bounds((-0.017, 0, -0.003), (0.017, 0.017, 0.003)), broom_clip)
part('WallBrooms__bristles', f_cone(0.055, 0.14), broom_bristles, smooth=50)
part('WallBrooms__twine', f_torus(0.014, 0.003), broom_twine, smooth=70)


# ----------------------------------------------------------- rolling pin --
def pin_roller():
    # maple roller: eased ends with a turned ring round the axle
    return lathe([(0, -0.14), (0.0160, -0.14), (0.0165, -0.1392), (0.0200, -0.1392), (0.0228, -0.1385), (0.0240, -0.136),
                  (0.0240, 0.136), (0.0228, 0.1385), (0.0200, 0.1392), (0.0165, 0.1392), (0.0160, 0.14), (0, 0.14)], seg=24)

def pin_stem():
    # turned handle, swelling in the middle (the same from both ends)
    return lathe([(0, -0.025), (0.0072, -0.025), (0.0069, -0.020), (0.0085, -0.012), (0.0100, -0.003), (0.0100, 0.003),
                  (0.0085, 0.012), (0.0069, 0.020), (0.0072, 0.025), (0, 0.025)], seg=14)

def pin_knob():
    # end knob turned on the pin's axis (x), two scribed rings
    prof = []
    for i in range(11):
        t = math.pi * i / 10
        a, r = -0.014 * math.cos(t), 0.014 * math.sin(t)
        if i in (3, 7):
            r *= 0.94
        prof.append((r, a))
    return lathe(prof, seg=14).rot(z=-HALF)

part('RollingPins__roller', f_cyl(0.024, 0.024, 0.28), pin_roller)
part('RollingPins__stem', f_cyl(0.010, 0.010, 0.050), pin_stem, smooth=50)
part('RollingPins__knob', f_sphere(0.014), pin_knob, smooth=60)


# --------------------------------------------------------- cutting board --
def cutting_board():
    # rounded-corner board, eased edges, the hanging hole through at -x
    w, h, d = 0.30, 0.018, 0.20
    q = lambda i: rr(w - 2 * i, d - 2 * i, 0.02 - i, 3)
    s = loft([(q(0.002), -h / 2), (q(0.0005), -h / 2 + 0.0015), (q(0), -h / 2 + 0.0035), (q(0), h / 2 - 0.0035),
              (q(0.0005), h / 2 - 0.0015), (q(0.002), h / 2)]).rot(x=-HALF)
    return cut(s, cyl(0.0110, 0.0110, 0.04, seg=16, bevel_r=0).move(x=-0.130))

part('CuttingBoards__plank', f_box(0.30, 0.018, 0.20), cutting_board)


# ----------------------------------------------------------- mixing bowl --
def bowl_body():
    # stoneware bowl: a foot ring, a rounded belly flaring to the rim
    return vessel([(0.046, -0.0275), (0.048, -0.0245), (0.0465, -0.022), (0.060, -0.010), (0.070, 0.006), (0.0745, 0.0261)],
                  wall=0.004, seg=24, floor=0.005)

def dough_ball():
    # a proved dough ball: soft, lumpy, sagging onto a flat bottom, a fold
    # seam across the top
    s = sphere(0.055, seg=16, rings=10)
    def shape(p):
        if p.y < -0.03:
            p.y = -0.03 + (p.y + 0.03) * 0.5
        if p.y > 0:
            p.y -= 0.006 * math.exp(-(p.x / 0.008) ** 2) * (p.y / 0.055)
        return p
    s = noisy(s.warp(shape), 0.0035, scale=40, seed=7)
    return s.fit((-0.055, -0.055, -0.055), (0.055, 0.055, 0.055))

part('MixingBowls__body', f_cyl(0.075, 0.048, 0.055), bowl_body)
part('MixingBowls__rim', f_torus(0.075, 0.005), lambda: torus(0.075, 0.005, 40, 6), smooth=60)
part('MixingBowls__dough', f_sphere(0.055), dough_ball, smooth=180)


# ----------------------------------------------------------------- whisk --
def whisk_handle():
    # grip swelling toward the cap end (+y), a waist where it meets the ferrule
    return lathe([(0, -0.05), (0.0085, -0.05), (0.0082, -0.035), (0.0095, -0.010), (0.0100, 0.020), (0.0095, 0.040),
                  (0.0092, 0.050), (0, 0.050)], seg=12)

def whisk_cap():
    # end cap: domed outward (-x), flat against the handle, a hanging hole
    prof = [(0, -0.011), (0.006, -0.0095), (0.0095, -0.005), (0.011, 0.0), (0.0105, 0.008), (0.009, 0.011), (0, 0.011)]
    s = lathe(prof, seg=14).rot(z=HALF)
    return cut(s, cyl(0.003, 0.003, 0.03, seg=10, bevel_r=0).move(x=-0.004)).fit((-0.011, -0.011, -0.011), (0.011, 0.011, 0.011))

def whisk_ferrule():
    # pressed metal ferrule: narrow toward the wires (-y), rolled lip on the handle
    return lathe([(0, -0.005), (0.0082, -0.005), (0.0088, -0.0042), (0.0105, 0.002), (0.0110, 0.0035), (0.0108, 0.005),
                  (0, 0.005)], seg=16)

part('Whisks__handle', f_cyl(0.010, 0.010, 0.10), whisk_handle, smooth=50)
part('Whisks__cap', f_sphere(0.011), whisk_cap, smooth=50)
part('Whisks__ferrule', f_cyl(0.011, 0.011, 0.010), whisk_ferrule)
part('Whisks__wire', f_bounds((-0.0212, 0, -0.0012), (0.0212, 0.0212, 0.0012)),
     lambda: half_arc_rod(0.020, 0.0012, seg=6, n=14), smooth=60)


# ------------------------------------------------------------ wall apron --
def apron_peg():
    return lathe([(0, -0.0075), (0.0032, -0.0075), (0.0022, -0.005), (0.0022, 0.004), (0.004, 0.0055), (0.0038, 0.007),
                  (0.0025, 0.0075), (0, 0.0075)], seg=8)

def apron_strap():
    sec = [(0.0012, -0.0027), (0.0012, 0.0027), (0.0, 0.0030), (-0.0012, 0.0027), (-0.0012, -0.0027), (0.0, -0.0030)]
    return arc_sweep(sec, 0.0218, 0.0, math.pi, 14).fit((-0.023, 0, -0.003), (0.023, 0.023, 0.003))

part('WallAprons__peg', f_cyl(0.004, 0.004, 0.015), apron_peg, smooth=50)
part('WallAprons__strap', f_bounds((-0.023, 0, -0.003), (0.023, 0.023, 0.003)), apron_strap)


# ---------------------------------------------------------------- teapot --
def trivet():
    # cast openwork trivet: a rim ring, six spokes, a round hub
    sec = [(0.006, -0.003), (0.006, 0.002), (0.005, 0.003), (-0.006, 0.003), (-0.006, -0.003)]
    s = weld(arc_sweep(sec, 0.069, 0.0, TAU, 32, caps=False)).rot(x=-HALF)
    for k in range(6):
        s = s + box(0.054, 0.004, 0.006).move(x=0.038, y=-0.001).rot(y=TAU * k / 6 + TAU / 12)
    s = s + cyl(0.013, 0.013, 0.006, seg=12, bevel_r=0.001, bevel_seg=1)
    return s.fit((-0.075, -0.003, -0.075), (0.075, 0.003, 0.075))

def trivet_foot():
    return lathe([(0, -0.004), (0.0048, -0.004), (0.006, -0.0022), (0.0058, 0.004), (0, 0.004)], seg=8)

def teapot_body():
    # round-bellied pot on a foot ring; the shoulder rises to a gallery the
    # lid sits in
    return lathe([(0, -0.06), (0.034, -0.06), (0.036, -0.056), (0.045, -0.046), (0.056, -0.028), (0.060, -0.006),
                  (0.057, 0.016), (0.047, 0.036), (0.034, 0.051), (0.029, 0.056), (0.030, 0.060), (0.026, 0.058),
                  (0, 0.058)], seg=24)

def spout_piece(rt, rb, h):
    """A spout section: closed toward the pot (+y), open mouth with a thin
    lip at the wide end (-y)."""
    y = h / 2
    return lathe([(0, -y + 0.009), (rb - 0.004, -y + 0.006), (rb - 0.0025, -y), (rb - 0.0004, -y), (rb, -y + 0.0015),
                  (rt, y), (0, y)], seg=12)

def teapot_handle():
    return arc_sweep(ellipse(0.0045, 0.006, 8), 0.0315, 0.0, math.pi, 14).fit((-0.036, 0, -0.006), (0.036, 0.036, 0.006))

def teapot_lid():
    return lathe([(0, -0.005), (0.0255, -0.005), (0.0260, -0.0035), (0.0250, -0.0020), (0.0220, -0.0015), (0.0210, 0.001),
                  (0.0160, 0.0035), (0.0080, 0.0048), (0, 0.005)], seg=18)

def teapot_knob():
    return lathe([(0, -0.008), (0.004, -0.008), (0.003, -0.004), (0.0065, -0.0005), (0.008, 0.002), (0.0065, 0.0055),
                  (0.0035, 0.0075), (0, 0.008)], seg=8)

part('Teapots__trivet', f_cyl(0.075, 0.075, 0.006), trivet)
part('Teapots__foot', f_cyl(0.006, 0.006, 0.008), trivet_foot, smooth=60)
part('Teapots__body', f_sphere(0.060), teapot_body, smooth=50)
part('Teapots__spout', f_cyl(0.012, 0.014, 0.030), lambda: spout_piece(0.012, 0.014, 0.030), smooth=50)
part('Teapots__spoutTip', f_cyl(0.009, 0.012, 0.024), lambda: spout_piece(0.009, 0.012, 0.024), smooth=50)
part('Teapots__handle', f_bounds((-0.036, 0, -0.006), (0.036, 0.036, 0.006)), teapot_handle, smooth=60)
part('Teapots__lid', f_cyl(0.022, 0.026, 0.010), teapot_lid, smooth=50)
part('Teapots__knob', f_sphere(0.008), teapot_knob, smooth=60)


# ------------------------------------------------------------- cookie tin --
def tin_body():
    # tin: rolled bottom seam, a pressed bead round the body, the lid skirt
    # stepping out over the top
    return lathe([(0, -0.0275), (0.0705, -0.0275), (0.0725, -0.0255), (0.0725, -0.0135), (0.0716, -0.012),
                  (0.0725, -0.0105), (0.0725, 0.004), (0.0750, 0.006), (0.0750, 0.026), (0.0742, 0.0275), (0, 0.0275)],
                 seg=28)

part('CookieTins__body', f_cyl(0.075, 0.075, 0.055), tin_body)
part('CookieTins__rim', f_torus(0.075, 0.003), lambda: torus(0.075, 0.003, 40, 6), smooth=60)


# ---------------------------------------------------------------- teacup --
def saucer():
    # foot ring underneath, a raised plateau for the cup, a shallow well,
    # a rounded rim
    return lathe([(0, -0.0022), (0.028, -0.0022), (0.030, -0.0030), (0.033, -0.0030), (0.035, -0.0020), (0.050, -0.0004),
                  (0.0545, 0.0016), (0.0550, 0.0028), (0.0535, 0.0030), (0.046, 0.0019), (0.036, 0.0017),
                  (0.029, 0.0030), (0, 0.0030)], seg=24)

def teacup():
    # porcelain cup: foot ring, a flaring bowl with a thin rim, hollow
    return vessel([(0.0195, -0.016), (0.0215, -0.0135), (0.0255, -0.002), (0.0278, 0.0149)], wall=0.0022, seg=16,
                  floor=0.004)

def cup_handle():
    return arc_sweep(ellipse(0.0024, 0.003, 6), 0.0126, 0.0, math.pi, 12).fit((-0.015, 0, -0.003), (0.015, 0.015, 0.003))

def tea():
    # the tea's surface with a meniscus climbing the cup wall
    return lathe([(0, -0.0015), (0.024, -0.0015), (0.024, 0.0015), (0.0225, 0.0007), (0.018, 0.0005), (0, 0.0005)], seg=20)

part('Teacups__saucer', f_cyl(0.055, 0.050, 0.006), saucer)
part('Teacups__cup', f_cyl(0.028, 0.022, 0.032), teacup, smooth=50)
part('Teacups__handle', f_bounds((-0.015, 0, -0.003), (0.015, 0.015, 0.003)), cup_handle, smooth=60)
part('Teacups__tea', f_cyl(0.024, 0.024, 0.003), tea)


# ------------------------------------------------------- cast iron ladle --
def ladle_handle():
    # forged square bar: plain near the ends, a twisted run in the middle
    rings = []
    ys = [-0.170, -0.166, -0.10, -0.06] + [-0.06 + 0.16 * i / 22 for i in range(1, 23)] + [0.14, 0.166, 0.170]
    for y in ys:
        t = min(1.0, max(0.0, (y + 0.06) / 0.16))
        ang = t * 2 * math.pi
        k = 0.8 if abs(y) > 0.168 else 1.0
        sq = rr(0.0112 * k, 0.0112 * k, 0.0016, 1)
        c, sn = math.cos(ang), math.sin(ang)
        rings.append(([(x * c - z * sn, x * sn + z * c) for x, z in sq], y))
    s = loft(rings).rot(x=-HALF)
    return s.fit((-0.008, -0.17, -0.008), (0.008, 0.17, 0.008))

def ladle_bowl():
    # the ladle's cup: a hemisphere with a wall and a rounded rim, open on top
    return lathe([(0, -0.035), (0.0175, -0.0303), (0.0303, -0.0175), (0.035, -0.002), (0.0335, 0.0), (0.032, -0.002),
                  (0.0285, -0.016), (0.016, -0.028), (0, -0.032)], seg=16)

part('CastIronLadles__handle', f_cyl(0.008, 0.008, 0.34), ladle_handle)
part('CastIronLadles__ring', f_torus(0.010, 0.002), lambda: torus(0.010, 0.002, 20, 6), smooth=60)
part('CastIronLadles__bowl', f_bounds((-0.035, -0.035, -0.035), (0.035, 0, 0.035)), ladle_bowl, smooth=50)
