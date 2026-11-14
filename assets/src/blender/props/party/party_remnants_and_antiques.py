"""Family `party_remnants_and_antiques`: party remnants and Victorian household objects.

Party hats, gift boxes, horns, plates, candles, pinatas, balloons, strings,
instruments, costume bits, party food (pie, cake, muffins, donuts,
sandwiches), then the antique things: barometer, opera glasses, fan, music
box, portrait frame, leather books, rocking chair.

Paper that the game draws DoubleSide (cups, wrappers, napkins, streamers,
crowns, garland cut-outs, fan paper, horn cone) and the transparent confetti
stay primitives; see docs/research/props-remodelling/party_remnants_and_antiques.md.
"""
from propkit import *
import propkit


# ------------------------------------------------------------ local kit --
def _fw(s):
    return propkit._fix_winding(s)


def loft(rings, cap0=True, cap1=True):
    """Skin closed loops of 3D points (equal counts) into one solid."""
    n = len(rings[0])
    v, f = [], []
    for ring in rings:
        v += [Vector(p) for p in ring]
    for k in range(len(rings) - 1):
        a, b = k * n, (k + 1) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    if cap0:
        f.append(tuple(range(n)))
    if cap1:
        f.append(tuple(range((len(rings) - 1) * n, len(rings) * n)))
    return _fw(Shape(v, f))


def ring_section(R, sec, seg=32, arc=TAU, a0=0.0, rmod=None):
    """Sweep a closed 2D section [(dr, dz), ...] around the Z axis in the XY
    plane (a torus_geo! with any cross-section). arc < TAU caps the ends.
    rmod(angle, i) -> (k_r, dz_add[, dR]) wobbles the section along the ring."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        mm = rmod(a, i) if rmod else (1.0, 0.0)
        kr, dzz = mm[0], mm[1]
        dR = mm[2] if len(mm) > 2 else 0.0
        for (dr, dz) in sec:
            rr_ = R + dR + dr * kr
            v.append((rr_ * ca, rr_ * sa, dz * kr + dzz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


def circ_sec(r, n=8, sx=1.0, sy=1.0, phase=0.0):
    return [(r * sx * math.cos(TAU * i / n + phase), r * sy * math.sin(TAU * i / n + phase)) for i in range(n)]


def rrect_pts(w, d, r, cseg=3):
    """Rounded rectangle (x, z) outline with 4*(cseg+1) points."""
    r = max(1e-5, min(r, w / 2 - 1e-5, d / 2 - 1e-5))
    out = []
    for cx, cz, a0 in ((w / 2 - r, d / 2 - r, 0), (-w / 2 + r, d / 2 - r, 90),
                       (-w / 2 + r, -d / 2 + r, 180), (w / 2 - r, -d / 2 + r, 270)):
        for i in range(cseg + 1):
            a = math.radians(a0 + 90 * i / cseg)
            out.append((cx + r * math.cos(a), cz + r * math.sin(a)))
    return out


def sellipse_pts(a, b, n=20, p=2.5, phase=0.0):
    """Superellipse (x, z) outline: p=2 ellipse, larger = squarer."""
    out = []
    for i in range(n):
        t = TAU * i / n + phase
        c, s = math.cos(t), math.sin(t)
        out.append((a * math.copysign(abs(c) ** (2 / p), c), b * math.copysign(abs(s) ** (2 / p), s)))
    return out


def yloft(profile, w, d, rc, cseg=3):
    """Stack of rounded rectangles along +Y: profile = [(y, inset), ...]."""
    rings = []
    for y, ins in profile:
        pts = rrect_pts(w - 2 * ins, d - 2 * ins, max(0.0003, rc - ins * 0.6), cseg)
        rings.append([(x, y, z) for x, z in pts])
    return loft(rings)


def oval_loft(profile, a, b, n=20, p=2.5):
    """Stack of superellipses along +Y: profile = [(y, inset), ...]."""
    return loft([[(x, y, z) for x, z in sellipse_pts(a - ins, b - ins, n, p)] for y, ins in profile])


def torus_arc_bounds(R, r, arc, radial=24, tubular=128):
    """Ideal bounds of torus_geo!(R, r, .., .., arc) as the game
    computes them (24 x 128 samples)."""
    xs, ys = [], []
    for i in range(tubular + 1):
        u = arc * i / tubular
        for j in range(radial + 1):
            w = TAU * j / radial
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r))


def x_axis(s):
    """Y-axis shape laid along +X (its +Y end goes to +X)."""
    return s.rot(z=-math.pi / 2)


def twist_cord(r, h, rings=16, turn=0.6, lobes=2, depth=0.22, n=8, taper=0.0):
    """A plied cord along Y: a lobed section (lobes = plies) that turns
    `turn` radians per ring. Cheap way to read as twisted string or twine."""
    rs = []
    for k in range(rings + 1):
        t = k / rings
        y = -h / 2 + h * t
        ph = turn * k
        ring = []
        for i in range(n):
            a = TAU * i / n
            rr_ = r * (1 - depth * 0.5 + depth * 0.5 * math.cos(lobes * (a - ph)))
            rr_ *= 1 - taper * t
            ring.append((rr_ * math.sin(a), y, rr_ * math.cos(a)))
        rs.append(ring)
    return loft(rs)


def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


def lumpy(s, amount, scale, seed=0):
    return noisy(s, amount, scale=scale, seed=seed)


# ====================================================== birthday hats ==
def hat_cone():
    # rolled cardstock cone, open underneath, a lap seam up one side and a
    # rolled bottom edge; built apex-down as a vessel, then turned over
    prof = [(0.0022, -0.0885), (0.030, 0.0), (0.0592, 0.0875), (0.060, 0.090)]
    s = vessel(prof, 0.0013, seg=24, floor=0.012).rot(x=math.pi)
    step = TAU / 24

    def seam(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6:
            return p
        cone_r = 0.06 * (0.09 - p.y) / 0.18
        if r < cone_r - 0.0007:          # inside surface
            return p
        i = int(round(math.atan2(p.x, p.z) / step)) % 24
        if i in (0, 1):
            k = (r + 0.0008 * max(0.0, (0.09 - p.y) / 0.18)) / r
            p.x *= k
            p.z *= k
        return p
    return s.warp(seam)

part('BirthdayHats__cone', f_cone(0.06, 0.18), hat_cone)


def hat_band():
    # crepe-paper trim round the brim: a flat band hugging the cone with a
    # soft wave along its top edge (the game lays the ring flat: -Z is up)
    sec = [(-0.0022, -0.0046), (0.0032, -0.0050), (0.0050, -0.0028), (0.0050, 0.0030),
           (0.0032, 0.0050), (-0.0022, 0.0046)]
    return ring_section(0.06, sec, seg=44,
                        rmod=lambda a, i: (1.0 - 0.06 * (1 + math.sin(7 * a)) * 0.5, 0.0005 * math.sin(7 * a)))

part('BirthdayHats__band', f_torus(0.06, 0.005), hat_band, smooth=60)


def pompom(r, seed=3):
    # fluffy tissue pompom: a ball whose vertices are pushed in and out
    s = sphere(r * 0.8, seg=12, rings=8)
    rnd = [0.86, 1.0, 0.92, 1.04, 0.84, 0.97, 1.02, 0.88, 0.95]

    def w(p):
        d = p.length
        if d < 1e-9:
            return p
        h = int(abs(p.x * 91731 + p.y * 51137 + p.z * 77101 + seed * 13) * 1000) % len(rnd)
        return p * (rnd[h] * 1.0)
    return s.warp(w)

part('BirthdayHats__tassel', f_sphere(0.010), lambda: fitf(pompom(0.010), f_sphere(0.010)), smooth=70)


# ===================================================== small gift boxes ==
def gift_body():
    # lidded gift box: the lid (top quarter) is full size with a soft edge,
    # the base sits 1.5 mm inside it
    bs = 0.12
    lid_h = bs * 0.24
    lid = rbox(bs, lid_h, bs, r=0.0022, seg=2).move(y=bs / 2 - lid_h / 2)
    base = rbox(bs - 0.003, bs - lid_h + 0.004, bs - 0.003, r=0.0016, seg=2).move(y=-lid_h / 2 + 0.002 - 0.0)
    return lid + base

part('SmallGiftBoxes__body', f_box(0.12, 0.12, 0.12), gift_body)


def ribbon_strip(L, w, t=0.003):
    # satin ribbon over the lid: flat where it lies, swelling at the middle
    # where the two ribbons cross under the bow, soft rolled edges
    s = slab(L, w, 0.0011, 14, 2).rot(x=-math.pi / 2)

    def w_(p):
        u = p.x / (L / 2)
        hump = max(0.0, 1 - (u / 0.28) ** 2)
        p.y += -t / 2 + 0.00055 + hump * (t - 0.0011)
        # a little satin ripple along the length
        p.y += 0.00012 * math.sin(u * 9.0)
        return p
    return s.warp(w_)

part('SmallGiftBoxes__ribbonX', f_box(0.122, 0.003, 0.010), lambda: ribbon_strip(0.122, 0.010))
part('SmallGiftBoxes__ribbonZ', f_box(0.010, 0.003, 0.122), lambda: ribbon_strip(0.122, 0.010).rot(y=math.pi / 2))


# ==================================================== small party horns ==
def horn_mouth():
    # plastic mouthpiece: a flange against the paper (-Y) and a short nozzle
    # with a blow hole on the outside (+Y)
    return lathe([(0.0, -0.0025), (0.0134, -0.0025), (0.014, -0.0012), (0.0128, 0.0),
                  (0.0064, 0.0003), (0.0056, 0.0025), (0.0030, 0.0025), (0.0027, 0.0010), (0.0, 0.0010)], seg=16)

part('SmallPartyHorns__mouth', f_cyl(0.014, 0.014, 0.005), horn_mouth)


# ==================================================== small paper plates ==
def paper_plate_rim():
    # the printed rim of the plate: an up-turned lip rolled over at the
    # edge, with a soft wave pressed into it (frame -Z is up in the game)
    up = [(-0.0060, -0.0045), (-0.0012, -0.0024), (0.0036, 0.0028), (0.0060, 0.0060),
          (0.0060, 0.0034), (0.0034, 0.0004), (-0.0018, -0.0045), (-0.0060, -0.0060)]
    sec = [(dr, -u) for dr, u in up]
    return ring_section(0.095, sec, seg=32,
                        rmod=lambda a, i: (1.0, -0.0006 * math.cos(8 * a)))

part('SmallPaperPlates__rim', f_torus(0.095, 0.006), paper_plate_rim)


# ================================================ small birthday candles ==
def spiral_candle(r, h, flutes=6, turns=2.0, rings=10):
    # twisted birthday candle: fluted spiral body, eased foot, a melted
    # dish round the wick at the top
    n = flutes * 2

    def ring(y, t, k=1.0):
        ph = TAU * turns * t / flutes
        out = []
        for i in range(n):
            a = TAU * i / n + ph
            rr_ = r * k * (1.0 if i % 2 == 0 else 0.84)
            out.append((rr_ * math.sin(a), y, rr_ * math.cos(a)))
        return out
    rs = [ring(-h / 2, 0.0, 0.86)]
    for k in range(rings + 1):
        t = k / rings
        y = -h / 2 + 0.0008 + (h - 0.0016) * t
        rs.append(ring(y, t))
    rs.append(ring(h / 2, 1.0, 0.8))
    rs.append(ring(h / 2 - 0.0014, 1.0, 0.45))
    return loft(rs)

part('SmallBirthdayCandles__body', f_cyl(0.008, 0.008, 0.06), lambda: spiral_candle(0.008, 0.06, rings=7))


def wick(h, r=0.001):
    # a burnt wick that curls over at the tip
    return tube([(0, -h / 2, 0), (0, h * 0.1, 0), (r * 0.5, h * 0.35, 0), (r * 1.2, h / 2 - r * 0.6, 0)],
                r, seg=5, smooth=1, r_end=r * 0.7)

part('SmallBirthdayCandles__wick', f_cyl(0.001, 0.001, 0.008), lambda: fitf(wick(0.008), f_cyl(0.001, 0.001, 0.008)))


# ========================================================= small pinatas ==
def fringe_lathe(tiers, rfun, y0, y1, seg=18, flare=1.07, cut=0.0035):
    """Paper-fringe covered body of revolution: `tiers` rows of crepe
    fringe; each row starts tucked in at its top and flares out at its cut
    bottom edge, where every other strip is snipped shorter."""
    prof = [(0.0, y0)]
    dy = (y1 - y0) / tiers
    for k in range(tiers):
        ya, yb = y0 + dy * k, y0 + dy * (k + 1)
        prof.append((max(0.0005, rfun(ya + dy * 0.12) * flare), ya + dy * 0.12))
        prof.append((max(0.0005, rfun(yb) * 0.93), yb - dy * 0.02))
    prof.append((0.0, y1))
    s = lathe(prof, seg)
    edge_ys = [y0 + dy * k + dy * 0.12 for k in range(tiers)]
    step = TAU / seg

    def w(p):
        for ye in edge_ys:
            if abs(p.y - ye) < 1e-7 and math.hypot(p.x, p.z) > 1e-6:
                i = int(round(math.atan2(p.x, p.z) / step)) % seg
                if i % 2:
                    p.y += min(cut, dy * 0.3)
                    p.x *= 0.96
                    p.z *= 0.96
        return p
    return s.warp(w)


def pinata_core():
    R = 0.09
    s = fringe_lathe(7, lambda y: math.sqrt(max(0.0, R * R - y * y)), -R, R, seg=18, cut=0.006)
    return fitf(s, f_sphere(R))

part('SmallPinatas__core', f_sphere(0.09), pinata_core, smooth=50)


def pinata_point():
    # fringed cone point, rows of fringe like shingles from the tip down
    s = fringe_lathe(4, lambda y: 0.03 * (0.04 - y) / 0.08, -0.04, 0.036, seg=12, flare=1.08, cut=0.004)
    s = s + lathe([(0.0, 0.030), (0.0045, 0.031), (0.003, 0.0375), (0.0, 0.040)], seg=12)
    return fitf(s, f_cone(0.03, 0.08))

part('SmallPinatas__point', f_cone(0.03, 0.08), pinata_point, smooth=50)
part('SmallPinatas__string', f_cyl(0.003, 0.003, 0.6),
     lambda: twist_cord(0.003, 0.6, rings=40, turn=0.9, lobes=2, depth=0.3, n=6), smooth=60)


# ================================================ hanging balloon bundles ==
def balloon():
    # latex balloon: fuller at the top, drawn in to a nipple at the neck
    return lathe([(0.0, -0.090), (0.0045, -0.0895), (0.0075, -0.0865), (0.014, -0.082), (0.032, -0.071),
                  (0.056, -0.050), (0.076, -0.024), (0.0875, 0.004), (0.0900, 0.030), (0.0850, 0.054),
                  (0.0700, 0.073), (0.0450, 0.0860), (0.0200, 0.0895), (0.0, 0.090)], seg=20)

part('HangingBalloonBundles__balloon', f_sphere(0.09), balloon, smooth=70)


def balloon_tie():
    # neck and knot under the balloon; -Y joins the balloon, +Y hangs down
    return lathe([(0.0, -0.007), (0.0080, -0.007), (0.0034, -0.0020), (0.0056, 0.0024),
                  (0.0030, 0.0052), (0.0040, 0.0066), (0.0, 0.0070)], seg=8)

part('HangingBalloonBundles__tie', f_cone(0.008, 0.014), balloon_tie, smooth=60)
# curling ribbon: a flat ribbon slowly twisting as it hangs
part('HangingBalloonBundles__string', f_cyl(0.002, 0.002, 0.6),
     lambda: twist_cord(0.002, 0.6, rings=24, turn=0.33, lobes=2, depth=0.75, n=6), smooth=60)


# ================================================== party banner strings ==
part('SmallPartyBanners__string', f_cyl(0.002, 0.002, 0.4),
     lambda: twist_cord(0.002, 0.4, rings=16, turn=0.9, lobes=2, depth=0.3, n=6), smooth=60)
part('PaperGarlands__string', f_cyl(0.0015, 0.0015, 0.15),
     lambda: twist_cord(0.0015, 0.15, rings=8, turn=0.9, lobes=2, depth=0.3, n=6), smooth=60)


# ========================================================= small trumpets ==
def trumpet_tube():
    # brass leadpipe with ferrule bands, a braced middle and slide collars
    prof = [(0.0, -0.09), (0.0098, -0.09), (0.010, -0.086), (0.0098, -0.082), (0.0084, -0.080)]
    for yc in (-0.035, 0.0, 0.035):
        prof += [(0.0084, yc - 0.006), (0.0094, yc - 0.0045), (0.0094, yc + 0.0045), (0.0084, yc + 0.006)]
    prof += [(0.0084, 0.080), (0.0098, 0.082), (0.010, 0.086), (0.0098, 0.09), (0.0, 0.09)]
    return lathe(prof, seg=14)

part('SmallTrumpets__tube', f_cyl(0.010, 0.010, 0.18), trumpet_tube)


def trumpet_bell():
    # flared bell: throat at the apex (+Y), a thin flare out to a rolled
    # rim at the wide end (-Y), open inside; built rim-up, then turned over
    prof = [(0.0092, -0.030), (0.0125, 0.004), (0.0205, 0.021), (0.0345, 0.030)]
    return vessel(prof, 0.0012, seg=14, floor=0.004).rot(x=math.pi)

part('SmallTrumpets__bell', f_cone(0.035, 0.06), trumpet_bell)


def trumpet_mouthpiece():
    # cup rim at +Y, cup inside, shank at -Y where it enters the leadpipe
    return lathe([(0.0, -0.006), (0.0058, -0.006), (0.0062, -0.0015), (0.0080, 0.0015), (0.0106, 0.0042),
                  (0.0120, 0.0054), (0.0112, 0.0060), (0.0086, 0.0052), (0.0050, 0.0025), (0.0, 0.0018)], seg=14)

part('SmallTrumpets__mouth', f_cyl(0.012, 0.010, 0.012), trumpet_mouthpiece)


def trumpet_valve():
    # valve cap and finger button: knurled cap, stem, dished pearl button
    s = lathe([(0.0, -0.010), (0.0072, -0.010), (0.0075, -0.0040), (0.0060, -0.0030),
               (0.0026, -0.0028), (0.0026, 0.0035), (0.0080, 0.0048), (0.0078, 0.0094),
               (0.0060, 0.0100), (0.0, 0.0092)], seg=12)
    return s

part('SmallTrumpets__valve', f_cyl(0.008, 0.008, 0.020), trumpet_valve)


# =========================================================== small kazoos ==
def ring_lathe(profile, seg=24):
    """Revolve a *closed* (r, y) loop (no point on the axis): tubes, bands."""
    s = lathe(list(profile) + [profile[0]], seg, cap_bottom=False, cap_top=False)
    return _fw(weld(s))


def kazoo_body():
    # tapered tin tube, open at both ends with rolled lips (+Y is the wide
    # mouth end)
    return ring_lathe([(0.0094, -0.045), (0.0100, -0.0440), (0.0134, 0.0420), (0.0140, 0.0440),
                       (0.0134, 0.0450), (0.0124, 0.0440), (0.0086, -0.0440), (0.0088, -0.0450)], seg=16)

part('SmallKazoos__body', f_cyl(0.014, 0.010, 0.09), kazoo_body)


def kazoo_turret():
    # resonator turret: a saddle foot, a ribbed screw ring and a lipped top
    # holding the membrane
    return lathe([(0.0, -0.006), (0.0100, -0.006), (0.0082, -0.0036), (0.0082, -0.0010),
                  (0.0096, -0.0006), (0.0096, 0.0016), (0.0088, 0.0024), (0.0096, 0.0032),
                  (0.0100, 0.0060), (0.0087, 0.0060), (0.0, 0.0054)], seg=12)

part('SmallKazoos__resonator', f_cyl(0.010, 0.010, 0.012), kazoo_turret)


# ===================================================== small feather boas ==
def boa_puff(r=0.028, seed=0):
    # a clump of marabou feathers: a ball bristling with soft tips
    s = sphere(r, seg=14, rings=10)
    k = [1.0, 0.74, 0.92, 0.70, 0.97, 0.78, 0.88, 0.72]

    def w(p):
        d = p.length
        if d < 1e-9:
            return p
        a = math.atan2(p.x, p.z)
        e = math.acos(max(-1.0, min(1.0, p.y / d)))
        i = int(round(a / (TAU / 14))) + int(round(e / (math.pi / 10))) * 3 + seed
        return p * k[i % len(k)]
    return fitf(s.warp(w), f_sphere(r))

part('SmallFeatherBoas__puff', f_sphere(0.028), boa_puff, smooth=180)


# =========================================================== small tiaras ==
def tiara_wire(h=0.0215):
    # a length of the beaded gold band: three beads on a wire
    y = h / 2
    return lathe([(0.0, -y), (0.0028, -y), (0.0040, -y * 0.67), (0.0028, -y * 0.33), (0.0040, 0.0),
                  (0.0028, y * 0.33), (0.0040, y * 0.67), (0.0028, y), (0.0, y)], seg=6)

part('SmallTiaras__band', f_cyl(0.004, 0.004, 0.0215), tiara_wire, smooth=60)


def tiara_spike():
    # a turned spire: collar at the band, a teardrop swell, a thin neck and
    # a ball finial at the tip (+Y)
    return lathe([(0.0, -0.015), (0.0072, -0.015), (0.0080, -0.0135), (0.0060, -0.0115), (0.0046, -0.008),
                  (0.0050, -0.0035), (0.0030, 0.003), (0.0014, 0.008), (0.0012, 0.0105),
                  (0.0024, 0.0118), (0.0024, 0.0138), (0.0, 0.015)], seg=8)

part('SmallTiaras__spike', f_cone(0.008, 0.030), tiara_spike, smooth=50)


# ===================================================== small opera glasses ==
def opera_barrel():
    # one barrel: objective ring at -Y (the lens sits on its face), a
    # covered body between two bands, a narrower eyepiece and eyecup at +Y
    return lathe([(0.0, -0.0222), (0.0163, -0.0222), (0.0180, -0.0212), (0.0180, -0.0160),
                  (0.0170, -0.0150), (0.0168, 0.0070), (0.0178, 0.0085), (0.0178, 0.0120),
                  (0.0136, 0.0135), (0.0132, 0.0195), (0.0142, 0.0215), (0.0118, 0.0225),
                  (0.0, 0.0212)], seg=12)

part('SmallOperaGlasses__barrel', f_cyl(0.018, 0.018, 0.045), opera_barrel)


def opera_bridge():
    # hinge bar between the barrels with a knurled focus wheel in the middle
    bar = rbox(0.030, 0.0048, 0.0056, r=0.0014, seg=1)
    wheel = lathe([(0.0, -0.0025), (0.0036, -0.0025), (0.0040, -0.0018), (0.0040, 0.0018),
                   (0.0036, 0.0025), (0.0, 0.0025)], seg=12)
    step = TAU / 12

    def kn(p):
        r = math.hypot(p.x, p.z)
        if r > 0.0037:
            i = int(round(math.atan2(p.x, p.z) / step)) % 12
            if i % 2:
                p.x *= 0.9
                p.z *= 0.9
        return p
    wheel = x_axis(wheel.warp(kn)).scale(1.0, 1.0, 1.25)
    ends = sum((x_axis(lathe([(0.0, -0.002), (0.0028, -0.002), (0.0030, 0.0), (0.0028, 0.002), (0.0, 0.002)],
                             seg=8)).move(x=sx * 0.013) for sx in (-1, 1)), Shape())
    return fitf(bar + wheel + ends, f_box(0.030, 0.008, 0.010))

part('SmallOperaGlasses__bridge', f_box(0.030, 0.008, 0.010), opera_bridge)


def opera_handle():
    # lorgnette handle: socket at -Y by the barrels, turned collars, a slim
    # grip and an acorn finial at the outer end (+Y)
    return lathe([(0.0, -0.05), (0.0034, -0.05), (0.0040, -0.047), (0.0040, -0.043), (0.0026, -0.041),
                  (0.0024, -0.020), (0.0032, -0.018), (0.0024, -0.016), (0.0021, 0.030), (0.0030, 0.033),
                  (0.0022, 0.036), (0.0036, 0.042), (0.0034, 0.047), (0.0016, 0.0495), (0.0, 0.05)], seg=8)

part('SmallOperaGlasses__handle', f_cyl(0.004, 0.004, 0.10), opera_handle, smooth=50)


# ====================================================== small folding fans ==
def fan_stick():
    # a guard stick: broad and rounded at the rivet end (-Y), tapering to a
    # slim tip (+Y); rounded-square section so it reads from any turn
    rings = []
    for y, k in ((-0.060, 0.55), (-0.0585, 0.92), (-0.056, 1.0), (-0.045, 0.96), (-0.010, 0.84),
                 (0.030, 0.70), (0.055, 0.60), (0.0595, 0.45), (0.060, 0.2)):
        rings.append([(x, y, z) for x, z in sellipse_pts(0.002 * k, 0.002 * k, 10, 3.0)])
    return fitf(loft(rings), f_cyl(0.002, 0.002, 0.12))

part('SmallFoldingFans__stick', f_cyl(0.002, 0.002, 0.12), fan_stick)


def fan_rivet():
    # the pivot: domed rivet heads either side of a washer
    return lathe([(0.0, -0.008), (0.0042, -0.0074), (0.0062, -0.0036), (0.0080, -0.0012),
                  (0.0080, 0.0012), (0.0062, 0.0036), (0.0042, 0.0074), (0.0, 0.008)], seg=8)

part('SmallFoldingFans__knob', f_sphere(0.008), fan_rivet, smooth=50)


# ================================================== candy corn scatters ==
def corn_band(rows, n=12):
    return loft([[(x, y, z) for x, z in sellipse_pts(r, r, n, 3.0)] for y, r in rows])

# the kernel's three bands meet at matching sections: wide yellow base
# with a rounded foot, orange waist, white rounded tip
part('CandyCornScatters__yellow', f_cyl(0.006, 0.008, 0.008), lambda: corn_band(
    [(-0.004, 0.0066), (-0.0033, 0.0078), (-0.0022, 0.0080), (0.004, 0.0060)]), smooth=60)
part('CandyCornScatters__orange', f_cyl(0.004, 0.006, 0.008), lambda: corn_band(
    [(-0.004, 0.0060), (0.0, 0.0051), (0.004, 0.0040)]), smooth=60)
part('CandyCornScatters__white', f_cone(0.004, 0.008), lambda: corn_band(
    [(-0.004, 0.0040), (0.0, 0.0029), (0.0026, 0.0016), (0.0037, 0.0006), (0.004, 0.0001)], n=10), smooth=60)


# ============================================================ pie slices ==
def china_plate(rt, rb, h, seg=28):
    # glazed plate: flat top where the food sits, a rolled edge, a foot
    # ring and a hollow under the well
    y = h / 2
    return lathe([(0.0, -y + 0.0009), (rb * 0.60, -y + 0.0009), (rb * 0.625, -y), (rb * 0.70, -y),
                  (rb * 0.73, -y + 0.0011), (rb * 0.96, -y + 0.0019), (rt - 0.0012, y - 0.0016),
                  (rt, y - 0.0007), (rt - 0.0010, y), (0.0, y)], seg=seg)

part('PieSlices__plate', f_cyl(0.090, 0.082, 0.006), lambda: china_plate(0.090, 0.082, 0.006))


def plate_rim(R, r, seg=32):
    # raised rim on the plate: a ramp up from the well, a flat top and a
    # small bead on the outer edge (the game lays it flat: -Z is up)
    up = [(-1.0, -1.0), (-1.0, -0.5), (-0.55, 0.45), (-0.2, 0.85), (0.35, 0.85), (0.72, 1.0),
          (1.0, 0.55), (1.0, -1.0)]
    return ring_section(R, [(a * r, -b * r) for a, b in up], seg=seg)

part('PieSlices__rim', f_torus(0.086, 0.004), lambda: plate_rim(0.086, 0.004))

PIE_R, PIE_A = 0.075, math.pi / 3
PIE_F = f_bounds((0.0, -0.014, 0.0), (PIE_R * math.sin(PIE_A), 0.014, PIE_R))


def wedge_outline(R, arc_pts=9, edge_pts=2, rfun=None):
    """(x, z) outline of a slice wedge (the engine's theta, from +Z towards +X)."""
    out = [(0.0, 0.0)]
    for k in range(1, edge_pts + 1):
        t = k / (edge_pts + 1)
        out.append((0.0, R * t))
    for k in range(arc_pts):
        th = PIE_A * k / (arc_pts - 1)
        rr_ = rfun(th) if rfun else R
        out.append((rr_ * math.sin(th), rr_ * math.cos(th)))
    for k in range(edge_pts, 0, -1):
        t = k / (edge_pts + 1)
        out.append((R * t * math.sin(PIE_A), R * t * math.cos(PIE_A)))
    return out


def pie_filling():
    # the slice's filling: cut faces sag out a little in the middle, lumpy
    # fruit on the cut; the top slopes up to the crust's edge so the crust
    # always covers it
    base = wedge_outline(PIE_R)
    cx, cz = PIE_R * 0.38, PIE_R * 0.62
    rings = []
    for y, k in ((-0.014, 0.0), (-0.004, 0.018), (0.006, 0.012), (0.0125, 0.0)):
        ring = []
        for (x, z) in base:
            ring.append((x + (x - cx) * k, y, z + (z - cz) * k))
        rings.append(ring)
    top = []
    for (x, z) in base:
        u = math.hypot(x, z) / PIE_R
        top.append((x, 0.0124 + 0.0016 * u, z))
    rings.append(top)
    s = loft(rings)
    s = noisy(s, 0.0009, scale=260, seed=4)
    return fitf(s, PIE_F)

part('PieSlices__filling', PIE_F, pie_filling, smooth=45)


def polar_top_solid(outline_fn, nr, nt, top_fn, y_bot):
    """A slab over a wedge: top surface a polar grid (radial fraction u,
    angle t), bottom a flat n-gon, walls round the rim."""
    grid = []
    for i in range(nr + 1):
        u = max(i / nr, 0.02)
        row = []
        for j in range(nt + 1):
            th = PIE_A * j / nt
            R = outline_fn(th)
            row.append(Vector((u * R * math.sin(th), top_fn(u, th), u * R * math.cos(th))))
        grid.append(row)
    v, f = [], []
    idx = {}
    for i in range(nr + 1):
        for j in range(nt + 1):
            idx[(i, j)] = len(v)
            v.append(grid[i][j])
    for i in range(nr):
        for j in range(nt):
            f.append((idx[(i, j)], idx[(i, j + 1)], idx[(i + 1, j + 1)], idx[(i + 1, j)]))
    loop = [(0, j) for j in range(nt, -1, -1)][:1] + [(i, 0) for i in range(0, nr + 1)] + \
           [(nr, j) for j in range(1, nt + 1)] + [(i, nt) for i in range(nr - 1, 0, -1)]
    # the apex row (i = 0) is nearly a point: use its two end vertices
    loop = [(0, nt)] + loop[1:] if loop[0] != (0, nt) else loop
    bot = []
    for key in loop:
        p = v[idx[key]]
        bot.append(len(v))
        v.append(Vector((p.x, y_bot, p.z)))
    n = len(loop)
    for a in range(n):
        b = (a + 1) % n
        f.append((idx[loop[a]], idx[loop[b]], bot[b], bot[a]))
    f.append(tuple(bot))
    # close the tiny apex row on top
    f.append(tuple(idx[(0, j)] for j in range(nt + 1)))
    return _fw(weld(Shape(v, f), 1e-6))


def pie_crust():
    # top crust: a puffed, crimped edge along the arc, a lightly domed
    # surface, the tip thinner where the slice was cut
    crimp = lambda th: PIE_R * (1 - 0.028 * (0.5 + 0.5 * math.cos(th * 10 / PIE_A * TAU / 2 * 2)))

    def top(u, th):
        rim = max(0.0, (u - 0.72) / 0.28)
        return 0.0006 + 0.0009 * math.sin(min(u, 0.75) / 0.75 * math.pi / 2) + 0.0005 * rim ** 0.5 - 0.0003 * rim ** 3
    s = polar_top_solid(crimp, 4, 20, top, -0.002)
    return fitf(s, f_bounds((0.0, -0.002, 0.0), (PIE_R * math.sin(PIE_A), 0.002, PIE_R)))

part('PieSlices__crust', f_bounds((0.0, -0.002, 0.0), (PIE_R * math.sin(PIE_A), 0.002, PIE_R)), pie_crust)


def rosette(r, n_ridge=8, rings=7, turn=0.9):
    # piped whipped-cream rosette: a star-tip section winding up into a
    # soft point
    rs = []
    for k in range(rings + 1):
        t = k / rings
        y = -r + 2 * r * t
        rad = r * (0.86 * math.sin(math.pi * min(0.98, 0.18 + t * 0.82)) + 0.14 * (1 - t))
        if k == 0:
            rad *= 0.85
        ring = []
        for i in range(n_ridge * 2):
            a = TAU * i / (n_ridge * 2) + turn * t
            kk = 1.0 if i % 2 == 0 else 0.8
            ring.append((rad * kk * math.sin(a), y, rad * kk * math.cos(a)))
        rs.append(ring)
    rs[-1] = [(x * 0.2, y, z * 0.2) for (x, y, z) in rs[-1]]
    return fitf(loft(rs), f_sphere(r))

part('PieSlices__cream', f_sphere(0.012), lambda: rosette(0.012), smooth=60)


# =========================================================== cake stands ==
def stand_foot():
    # moulded foot: a stepped rim that sweeps up in a concave curve to a
    # boss under the column
    return lathe([(0.0, -0.005), (0.1085, -0.005), (0.110, -0.0042), (0.110, -0.0030), (0.1060, -0.0022),
                  (0.1030, -0.0006), (0.0880, 0.0010), (0.0600, 0.0026), (0.0400, 0.0038), (0.0330, 0.0046),
                  (0.0330, 0.0050), (0.0, 0.0050)], seg=28)

part('CakeStands__base', f_cyl(0.10, 0.11, 0.010), stand_foot)


def stand_column():
    # turned baluster: collar at the foot, a vase body, a knop and a
    # flared cup under the platter
    return lathe([(0.0, -0.07), (0.0300, -0.07), (0.0300, -0.064), (0.0250, -0.061), (0.0205, -0.056),
                  (0.0230, -0.050), (0.0205, -0.044), (0.0150, -0.030), (0.0122, -0.010), (0.0118, 0.012),
                  (0.0150, 0.020), (0.0180, 0.026), (0.0150, 0.032), (0.0112, 0.040), (0.0112, 0.050),
                  (0.0160, 0.058), (0.0220, 0.064), (0.0240, 0.070), (0.0, 0.070)], seg=14)

part('CakeStands__column', f_cyl(0.024, 0.030, 0.14), stand_column)


def stand_platter():
    # platter: flat top for the cake, a thickened rolled edge, a shallow
    # hollow underneath round a boss where the column meets it
    return lathe([(0.0, -0.004), (0.0280, -0.004), (0.0320, -0.0022), (0.1300, -0.0014), (0.1370, -0.0030),
                  (0.1400, -0.0012), (0.1400, 0.0024), (0.1385, 0.0040), (0.0, 0.0040)], seg=36)

part('CakeStands__platter', f_cyl(0.14, 0.14, 0.008), stand_platter)


def stand_rim():
    # beaded gallery round the platter edge (ring lies flat, -Z is up)
    up = [(-1.0, -1.0), (-1.0, 0.1), (-0.55, 0.9), (0.2, 1.0), (0.85, 0.45), (1.0, -1.0)]
    return ring_section(0.138, [(a * 0.005, -b * 0.005) for a, b in up], seg=72,
                        rmod=lambda a, i: (0.74 + 0.26 * abs(math.cos(9 * a)), 0.0))

part('CakeStands__rim', f_torus(0.138, 0.005), stand_rim)


def frosted_tier(R, h, seg=32, swags=8):
    # iced tier: a piped bead round the foot, softly bulging iced sides with
    # piped swags, a rounded top edge and a flat top for the next tier
    y = h / 2
    s = lathe([(0.0, -y), (R * 0.95, -y), (R, -y + h * 0.07), (R * 0.985, -y + h * 0.16), (R * 0.975, -y + h * 0.3),
               (R * 0.982, y - h * 0.30), (R * 0.978, y - h * 0.14), (R * 0.955, y - h * 0.04),
               (R * 0.92, y), (0.0, y)], seg=seg)

    def w(p):
        r = math.hypot(p.x, p.z)
        if r > R * 0.97 and -y + h * 0.25 < p.y < y - h * 0.1:
            a = math.atan2(p.x, p.z)
            t = (p.y - (-y + h * 0.25)) / (h * 0.65)
            k = 1 + 0.012 * max(0.0, math.cos(swags * a)) * math.sin(math.pi * t)
            p.x *= k
            p.z *= k
        return p
    return fitf(s.warp(w), f_cyl(R, R, h))

part('CakeStands__tierBottom', f_cyl(0.11, 0.11, 0.055), lambda: frosted_tier(0.11, 0.055))
part('CakeStands__tierTop', f_cyl(0.075, 0.075, 0.045), lambda: frosted_tier(0.075, 0.045, seg=28, swags=7))
part('CakeStands__swirl', f_sphere(0.009), lambda: rosette(0.009, n_ridge=7, rings=6), smooth=60)
part('CakeStands__candle', f_cyl(0.005, 0.005, 0.030), lambda: spiral_candle(0.005, 0.030, flutes=5, rings=6))
part('CakeStands__wick', f_cyl(0.001, 0.001, 0.006), lambda: fitf(wick(0.006), f_cyl(0.001, 0.001, 0.006)))


# ===================================================== cornbread muffins ==
def muffin_base():
    # the baked foot: pleated sides pressed by the paper liner, a flat
    # bottom and a slightly crumbly top edge under the crown
    s = lathe([(0.0, -0.012), (0.0282, -0.012), (0.0300, -0.0104), (0.0342, 0.0100), (0.0350, 0.0120),
               (0.0, 0.0120)], seg=32)
    step = TAU / 32

    def w(p):
        r = math.hypot(p.x, p.z)
        if r > 0.02 and p.y < 0.0115:
            i = int(round(math.atan2(p.x, p.z) / step)) % 32
            if i % 2:
                p.x *= 0.955
                p.z *= 0.955
        return p
    return s.warp(w)

part('CornbreadMuffins__bottom', f_cyl(0.035, 0.030, 0.024), muffin_base)

MUFFIN_TOP = f_bounds((-0.038, 0.0, -0.038), (0.038, 0.038, 0.038))


def muffin_crown():
    # the risen crown: overhanging lip, craggy dome, a sunken centre where
    # the placer lays its dark dimple disc (y 0.032 here)
    s = lathe([(0.0, 0.0318), (0.0070, 0.0318), (0.0110, 0.0350), (0.0160, 0.0380), (0.0240, 0.0345),
               (0.0310, 0.0270), (0.0360, 0.0170), (0.0380, 0.0065), (0.0340, 0.0),
               (0.0, 0.0)], seg=17)

    def w(p):
        if p.y > 0.004 and not (math.hypot(p.x, p.z) < 0.0075):
            q = p * 190 + Vector((3.1, 7.7, 1.3))
            d = noise.noise(q) * 0.0022
            n = Vector((p.x, 0.0, p.z))
            n = (n.normalized() * 0.6 + Vector((0, 0.8, 0))) if n.length > 1e-6 else Vector((0, 1, 0))
            return p + n * d
        return p
    return fitf(s.warp(w), MUFFIN_TOP)

part('CornbreadMuffins__top', MUFFIN_TOP, muffin_crown, smooth=60)


# ================================================================ donuts ==
def ring_custom(R, nsec, seg, arc, pt, dip=0.0):
    """Ring round the Z axis (XY plane) with full control: pt(a, i, j) ->
    (dr, dz, da) for section point j at ring step i. Open arcs get end caps;
    dip > 0 sinks each cap's centre into the ring (a bitten, hollow face)."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    v, f = [], []
    for i in range(n):
        a0 = arc * i / seg
        for j in range(nsec):
            dr, dz, da = pt(a0, i, j)
            a = a0 + da
            v.append(((R + dr) * math.cos(a), (R + dr) * math.sin(a), dz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(nsec):
            j2 = (j + 1) % nsec
            f.append((i * nsec + j, i2 * nsec + j, i2 * nsec + j2, i * nsec + j2))
    if not full and dip <= 0:
        f.append(tuple(reversed(range(nsec))))
        f.append(tuple(seg * nsec + j for j in range(nsec)))
    elif not full:
        for i, a in ((0, dip), (seg, arc - dip)):
            ids = [i * nsec + j for j in range(nsec)]
            cx = sum(v[k][0] for k in ids) / nsec
            cy = sum(v[k][1] for k in ids) / nsec
            cz = sum(v[k][2] for k in ids) / nsec
            rr_ = math.hypot(cx, cy)
            c = len(v)
            v.append((rr_ * math.cos(a), rr_ * math.sin(a), cz))
            for j in range(nsec):
                f.append((ids[j], ids[(j + 1) % nsec], c))
    return _fw(Shape(v, f))


# dough section (out, up) for r = 1: puffy top, flatter fried underside
DOUGH_SEC = [(1.0, 0.02), (0.84, 0.62), (0.40, 0.97), (-0.22, 0.98), (-0.74, 0.66), (-1.0, 0.04),
             (-0.86, -0.58), (-0.36, -0.93), (0.30, -0.93), (0.84, -0.60)]


def _drip(a, seed=0.0):
    return 0.5 + 0.3 * math.sin(5 * a + seed) + 0.2 * math.sin(11 * a + 2.1 + seed)


def dough_pt(r, arc, bite=False):
    def pt(a, i, j):
        o, u = DOUGH_SEC[j]
        k = 1.0 + 0.035 * math.sin(3 * a + 0.7) + 0.02 * math.sin(7 * a)
        da = 0.0
        if bite:
            e = min(a, arc - a)
            if e < 1e-6:                    # the bitten faces: tooth scallops
                sgn = 1 if a < arc / 2 else -1
                da = sgn * 0.10 * (0.55 + 0.45 * math.cos(3.2 * math.atan2(u, o)))
        return (o * r * k, -u * r * (0.98 if u > 0 else 1.0), da)
    return pt


def glaze_pt(r, arc, bite=False):
    # icing: full over the top, its lower edge runs down in drips here and
    # recedes into the dough there
    def pt(a, i, j):
        be = TAU * j / 10
        o, u = math.cos(be), math.sin(be)
        k = 1.0
        if u < 0.15:
            d = _drip(a, 0.4 if o > 0 else 2.2)
            k = 0.86 + 0.2 * d * min(1.0, (0.15 - u) / 0.6) if u < -0.2 else 1.0 + 0.04 * d
        da = 0.0
        if bite and min(a, arc - a) < 1e-6:
            sgn = 1 if a < arc / 2 else -1
            da = sgn * 0.09 * (0.55 + 0.45 * math.cos(3.2 * be))
        return (o * r * k, -u * r * k, da)
    return pt


def donut_dough():
    s = ring_custom(0.045, len(DOUGH_SEC), 28, TAU, dough_pt(0.020, TAU))
    return fitf(noisy(s, 0.0008, scale=150, seed=2), f_torus(0.045, 0.020))


def donut_glaze():
    s = ring_custom(0.045, 10, 28, TAU, glaze_pt(0.018, TAU))
    return fitf(s, f_torus(0.045, 0.018))

part('Donuts__dough', f_torus(0.045, 0.020), donut_dough, smooth=70)
part('Donuts__glaze', f_torus(0.045, 0.018), donut_glaze, smooth=70)


def paper_sheet(w, d, t, n, amp, seed, crease=0.0005, border=0.00035, lift=None):
    # a paper napkin lying flat: quarter-fold creases, an embossed border,
    # soft waves in the paper
    s = slab(w, d, t, n, n).rot(x=-math.pi / 2)

    def f(p):
        y = noise.noise(Vector((p.x * 30 + seed, p.z * 30, seed * 0.7))) * amp
        if abs(p.x) < w / n * 0.51 or abs(p.z) < d / n * 0.51:
            y -= crease
        if abs(abs(p.x) - (w / 2 - w / n)) < w / n * 0.2 or abs(abs(p.z) - (d / 2 - d / n)) < d / n * 0.2:
            y += border
        if lift:
            y += lift(p)
        p.y += y
        return p
    return s.warp(f)

part('Donuts__napkin', f_box(0.20, 0.003, 0.20),
     lambda: fitf(paper_sheet(0.20, 0.20, 0.0012, 10, 0.0007, 1), f_box(0.20, 0.003, 0.20)))


def tri_sheet(p0, p1, p2, n, t, lift):
    """A thin sheet over a triangle (XZ points), lifted by lift(x, z)."""
    P0, P1, P2 = (Vector((p[0], 0, p[1])) for p in (p0, p1, p2))
    v, f = [], []
    ids = {}
    for side, off in ((0, t / 2), (1, -t / 2)):
        for i in range(n + 1):
            for j in range(n + 1 - i):
                q = P0 + (P1 - P0) * (i / n) + (P2 - P0) * (j / n)
                q.y = lift(q.x, q.z) + off
                ids[(side, i, j)] = len(v)
                v.append(q)
    for side in (0, 1):
        for i in range(n):
            for j in range(n - i):
                a, b, c = ids[(side, i, j)], ids[(side, i + 1, j)], ids[(side, i, j + 1)]
                f.append((a, b, c) if side == 0 else (a, c, b))
                if i + j + 1 < n:
                    d = ids[(side, i + 1, j + 1)]
                    f.append((b, d, c) if side == 0 else (b, c, d))
    rim = [(i, 0) for i in range(n)] + [(n - j, j) for j in range(n)] + [(0, n - j) for j in range(n)]
    for k in range(len(rim)):
        a, b = rim[k], rim[(k + 1) % len(rim)]
        f.append((ids[(0,) + a], ids[(0,) + b], ids[(1,) + b], ids[(1,) + a]))
    return _fw(Shape(v, f))


def napkin_fold():
    # a dog-eared corner: a triangle of napkin folded back over itself, its
    # tip (the napkin's corner, +X +Z) curling up
    lift = lambda x, z: 0.0034 * max(0.0, (x + z) / 0.06) ** 1.6
    s = tri_sheet((0.03, 0.03), (-0.03, 0.03), (0.03, -0.03), 6, 0.0012, lift)
    return fitf(s, f_box(0.06, 0.005, 0.06))

part('Donuts__fold', f_box(0.06, 0.005, 0.06), napkin_fold)


def sprinkle():
    # a rod sprinkle with round ends, along X
    return x_axis(capsule(0.001, 0.006, seg=6, cap_rings=2))

part('Donuts__sprinkle', f_box(0.008, 0.002, 0.002), sprinkle, smooth=60)


# ======================================================== bitten donuts ==
BITE_ARC = TAU - math.pi / 3.5
F_BDOUGH = torus_arc_bounds(0.045, 0.020, BITE_ARC)
F_BGLAZE = torus_arc_bounds(0.045, 0.018, BITE_ARC)


def bitten_dough():
    s = ring_custom(0.045, len(DOUGH_SEC), 26, BITE_ARC, dough_pt(0.020, BITE_ARC, bite=True), dip=0.16)
    return fitf(noisy(s, 0.0008, scale=150, seed=5), F_BDOUGH)


def bitten_glaze():
    s = ring_custom(0.045, 10, 26, BITE_ARC, glaze_pt(0.018, BITE_ARC, bite=True), dip=0.14)
    return fitf(s, F_BGLAZE)

part('BittenDonuts__dough', F_BDOUGH, bitten_dough, smooth=70)
part('BittenDonuts__glaze', F_BGLAZE, bitten_glaze, smooth=70)

part('BittenDonuts__napkin', f_box(0.20, 0.003, 0.20),
     lambda: fitf(paper_sheet(0.20, 0.20, 0.0010, 10, 0.0011, 7, crease=0.0003), f_box(0.20, 0.003, 0.20)))


def napkin_wrinkle():
    # paper pinched up into a ridge along one diagonal
    s = slab(0.05, 0.05, 0.0010, 6, 6).rot(x=-math.pi / 2)

    def f(p):
        d = abs(p.x - p.z) / 0.05
        p.y += 0.0038 * max(0.0, 1 - d * 1.6) ** 1.5 + noise.noise(Vector((p.x * 60, p.z * 60, 2.0))) * 0.0004
        return p
    return fitf(s.warp(f), f_box(0.05, 0.005, 0.05))

part('BittenDonuts__wrinkle', f_box(0.05, 0.005, 0.05), napkin_wrinkle)


def crumb_ball(r, seed=0, amount=0.3):
    s = sphere(r, seg=10, rings=7)
    return noisy(s, r * amount, scale=3.0 / r, seed=seed)

part('BittenDonuts__bite', f_sphere(0.014),
     lambda: fitf(crumb_ball(0.014, 3, 0.22), f_sphere(0.014)), smooth=180)


def crumb(seed=1):
    # a crumb: a lumpy torn little chunk
    s = sphere(0.003, seg=6, rings=4).scale(1.0, 0.8, 1.1)
    return fitf(noisy(s, 0.0009, scale=500, seed=seed), f_box(0.006, 0.006, 0.006))

part('BittenDonuts__crumb', f_box(0.006, 0.006, 0.006), lambda: crumb(1), flat=True)


# ============================================================ sandwiches ==
part('Sandwiches__plate', f_cyl(0.10, 0.092, 0.006), lambda: china_plate(0.10, 0.092, 0.006))
part('Sandwiches__rim', f_torus(0.095, 0.004), lambda: plate_rim(0.095, 0.004))


def bread_outline(w, n=20):
    # a sandwich-loaf slice seen from above: straight base, slightly waisted
    # sides and the two-lobed crown of the loaf top (at -X)
    pts = []
    for i in range(n):
        t = TAU * i / n
        c, s_ = math.cos(t), math.sin(t)
        x = math.copysign(abs(c) ** 0.45, c)
        z = math.copysign(abs(s_) ** 0.45, s_)
        if x < 0:
            x -= 0.07 * abs(math.sin(2 * t)) * (-c)       # crown lobes
        z *= 1 - 0.03 * (1 - abs(s_))                      # waist
        pts.append((x * w / 2, z * w / 2))
    return pts


def bread_slice():
    # soft slice: crust round the edge, pillowy faces pinched at the rim
    base = bread_outline(0.08)

    def ring(y, k):
        return [(x * k, y, z * k) for x, z in base]
    rings = [ring(-0.0098, 0.955), ring(-0.0085, 0.99), ring(0.0, 1.0), ring(0.0085, 0.99), ring(0.0098, 0.955),
             ring(0.0100, 0.80)]
    s = loft([ring(-0.0100, 0.80)] + rings)
    s = noisy(s, 0.0004, scale=260, seed=6)
    return fitf(s, f_box(0.08, 0.020, 0.08))

part('Sandwiches__bread', f_box(0.08, 0.020, 0.08), bread_slice, smooth=50)


def deli_slices():
    # folded deli slices between the bread: a soft layer whose edge ripples
    w = 0.0784
    base = [(x * w / 0.08, z * w / 0.08) for x, z in sellipse_pts(0.04, 0.04, 32, 5.0, phase=0.05)]

    def ring(y, k):
        out = []
        for i, (x, z) in enumerate(base):
            wave = 0.0018 * math.sin(TAU * i / 32 * 5 + 0.4)
            out.append((x * k, y + wave, z * k))
        return out
    s = loft([ring(-0.0021, 0.97), ring(-0.0008, 1.0), ring(0.0010, 0.99), ring(0.0021, 0.95)])
    return fitf(s, f_box(0.0784, 0.008, 0.0784))

part('Sandwiches__filling', f_box(0.0784, 0.008, 0.0784), deli_slices, smooth=50)


def lettuce_frill():
    # a frilly lettuce edge poking out: thin leaf, ruffled outer edge (+Z)
    s = slab(0.024, 0.010, 0.0007, 6, 2).rot(x=-math.pi / 2)

    def f(p):
        t = (p.z + 0.005) / 0.010
        p.y += 0.0016 * t * math.sin(p.x / 0.024 * TAU * 1.5) + 0.0003 * t
        p.z += 0.0012 * t * math.cos(p.x / 0.024 * TAU * 5)
        return p
    return fitf(s.warp(f), f_box(0.024, 0.004, 0.010))

part('Sandwiches__lettuce', f_box(0.024, 0.004, 0.010), lettuce_frill, smooth=60)


def torn_bread():
    # the exposed crumb where the bite was torn out: a craggy block
    s = slab(0.030, 0.048, 0.030, 3, 5)
    s = bevel(s, 0.003, seg=1, angle=40)
    s = noisy(s, 0.0022, scale=90, seed=9)
    return fitf(s, f_box(0.030, 0.048, 0.030))

part('Sandwiches__bite', f_box(0.030, 0.048, 0.030), torn_bread, smooth=180)
part('Sandwiches__crumb', f_box(0.005, 0.005, 0.005),
     lambda: fitf(crumb(4), f_box(0.005, 0.005, 0.005)), flat=True)


# ======================================================= wall barometers ==
def loft_closed(rings):
    """Skin closed loops into a ring-shaped solid (last ring joins the first)."""
    n = len(rings[0])
    v, f = [], []
    for ring in rings:
        v += [Vector(p) for p in ring]
    m = len(rings)
    for k in range(m):
        a, b = k * n, ((k + 1) % m) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    return _fw(Shape(v, f))


def arch_outline(w, h, rise, inset=0.0, n_arch=10, rc=0.006, n_c=3):
    """Tall panel with an arched top, as (x, y) points, offset inward by
    `inset` (same point count for every inset)."""
    hw, hh = w / 2 - inset, h / 2 - inset
    R = (w * w / 4 + rise * rise) / (2 * rise)
    cy = h / 2 - R
    Ri = R - inset
    ys = cy + math.sqrt(max(0.0, Ri * Ri - hw * hw))
    pts = []
    r = max(0.0005, rc - inset * 0.5)
    for (cx, cyy, a0) in ((-hw + r, -hh + r, 180), (hw - r, -hh + r, 270)):
        for i in range(n_c + 1):
            a = math.radians(a0 + 90 * i / n_c)
            pts.append((cx + r * math.cos(a), cyy + r * math.sin(a)))
    a_s = math.atan2(ys - cy, hw)
    for i in range(n_arch + 1):
        a = a_s + (math.pi - 2 * a_s) * i / n_arch
        pts.append((Ri * math.cos(a), cy + Ri * math.sin(a)))
    return pts


def barometer_case():
    # dark-wood case: arched top, a moulded border standing proud of a
    # sunk panel, and a round boss behind the dial (dial centre y 0.20)
    w, h, rise = 0.18, 0.60, 0.042
    body = prism(arch_outline(w, h, rise), 0.044).move(z=-0.003)          # z -0.025 .. 0.019
    prof = [(0.0, 0.016), (0.0, 0.0215), (0.0018, 0.0245), (0.0045, 0.025), (0.0085, 0.025),
            (0.0105, 0.0236), (0.0125, 0.0232), (0.0150, 0.0212), (0.0165, 0.0206)]
    rings = [[(x, y, z) for x, y in arch_outline(w, h, rise, ins)] for ins, z in prof]
    border = loft(rings)
    boss = lathe([(0.0, -0.003), (0.0780, -0.003), (0.0780, 0.0015), (0.0770, 0.0025), (0.0745, 0.003),
                  (0.0, 0.003)], seg=40).rot(x=math.pi / 2).move(y=0.20, z=0.022)
    return fitf(body + border + boss, f_box(0.18, 0.60, 0.05))

part('WallBarometers__case', f_box(0.18, 0.60, 0.05), barometer_case)


def bezel_ring():
    # turned bezel round the dial: a half-round bead with a step inside
    sec = [(-1.0, -1.0), (1.0, -1.0), (1.0, -0.1), (0.62, 0.62), (0.0, 1.0), (-0.55, 0.72), (-0.78, 0.2),
           (-1.0, 0.1)]
    return ring_section(0.070, [(a * 0.003, b * 0.003) for a, b in sec], seg=34)

part('WallBarometers__ring', f_torus(0.070, 0.003), bezel_ring)
part('WallBarometers__cord', f_cyl(0.002, 0.002, 0.28),
     lambda: twist_cord(0.002, 0.28, rings=24, turn=0.9, lobes=2, depth=0.3, n=6), smooth=60)


def brass_bob():
    # turned brass weight: an eye on top for the cord, a neck, a shouldered
    # body and a drop finial underneath
    body = lathe([(0.0, -0.025), (0.0030, -0.0242), (0.0050, -0.0205), (0.0090, -0.0185), (0.0122, -0.0150),
                  (0.0130, -0.0100), (0.0130, 0.0080), (0.0118, 0.0125), (0.0085, 0.0155), (0.0050, 0.0170),
                  (0.0056, 0.0190), (0.0, 0.0195)], seg=10)
    eye = torus(0.0028, 0.0011, seg=8, tseg=4).move(y=0.0222)
    return fitf(body + eye, f_cyl(0.013, 0.013, 0.05))

part('WallBarometers__weight', f_cyl(0.013, 0.013, 0.05), brass_bob)


# ===================================================== small music boxes ==
def music_box():
    # wooden box: a moulded plinth, the case, a lid seam with a slight lip
    # and a raised field on the lid (the brass trim frames its edge)
    s = rbox(0.090, 0.005, 0.090, r=0.0015, seg=1).move(y=-0.029)
    s = s + rbox(0.087, 0.040, 0.087, r=0.0012, seg=1).move(y=-0.0075)
    s = s + rbox(0.090, 0.018, 0.090, r=0.0018, seg=2).move(y=0.0210)
    s = s + rbox(0.072, 0.004, 0.072, r=0.0015, seg=2).move(y=0.0295)
    return fitf(s, f_box(0.09, 0.063, 0.09))

part('SmallMusicBoxes__body', f_box(0.09, 0.063, 0.09), music_box)


def brass_trim():
    # brass edging round the lid: a hollow band with a rounded top edge
    prof = [(0.0, -0.002), (0.0, 0.0012), (0.0006, 0.0019), (0.0015, 0.002), (0.0052, 0.002),
            (0.0060, 0.0012), (0.0060, -0.002)]
    rings = [[(x, y, z) for x, z in rrect_pts(0.093 - 2 * ins, 0.093 - 2 * ins, max(0.0005, 0.002 - ins * 0.3), 2)]
             for ins, y in prof]
    return loft_closed(rings)

part('SmallMusicBoxes__trim', f_box(0.093, 0.004, 0.093), brass_trim)


def winding_shaft():
    # key shaft: a rosette against the case (+Y), the shaft, a square-ish
    # boss where the crank arm is pinned (-Y)
    return lathe([(0.0, -0.010), (0.0026, -0.010), (0.0026, -0.0065), (0.0017, -0.0055), (0.0017, 0.0065),
                  (0.0030, 0.0085), (0.0030, 0.010), (0.0, 0.010)], seg=6)

part('SmallMusicBoxes__shaft', f_cyl(0.003, 0.003, 0.020), winding_shaft, smooth=50)


def crank_arm():
    # flat brass crank: waisted, round bosses at both ends
    out = [(0.002 * math.cos(a), -0.013 + 0.002 * math.sin(a)) for a in (math.radians(d) for d in range(180, 361, 30))]
    out += [(0.0014, -0.006), (0.0014, 0.006)]
    out += [(0.002 * math.cos(a), 0.013 + 0.002 * math.sin(a)) for a in (math.radians(d) for d in range(0, 181, 30))]
    out += [(-0.0014, 0.006), (-0.0014, -0.006)]
    s = prism(out, 0.004, bevel_r=0.0006).rot(y=math.pi / 2)
    return fitf(s, f_box(0.004, 0.030, 0.004))

part('SmallMusicBoxes__arm', f_box(0.004, 0.030, 0.004), crank_arm)
part('SmallMusicBoxes__handle', f_cyl(0.004, 0.004, 0.012), lambda: lathe(
    [(0.0, -0.006), (0.0030, -0.006), (0.0034, -0.0045), (0.0040, -0.001), (0.0038, 0.003),
     (0.0026, 0.0055), (0.0, 0.006)], seg=8), smooth=60)


# =================================================== ancient portraits ==
def frame_bar(w, h, t, back=0.45):
    """A frame bar whose face rolls off towards all four edges, so it reads
    as a rounded moulding whichever way it is stretched (one constructor
    makes both the long and the tall bars). A sunk bead line runs round
    close to the edge."""
    us = [-1, -0.985, -0.95, -0.9, -0.84, -0.76, -0.6, -0.3, 0.0, 0.3, 0.6, 0.76, 0.84, 0.9, 0.95, 0.985, 1]
    n = len(us)

    def f_(u):
        a = abs(u)
        base = max(0.0, 1 - a ** 6) ** 0.35
        dip = 0.10 * math.exp(-((a - 0.84) / 0.05) ** 2)
        return max(0.0, base - dip)
    v, f = [], []
    for j, vv in enumerate(us):
        for i, uu in enumerate(us):
            z = -t / 2 + back * t + (1 - back) * t * f_(uu) * f_(vv)
            v.append((uu * w / 2, vv * h / 2, z))
    idx = lambda i, j: j * n + i
    for j in range(n - 1):
        for i in range(n - 1):
            f.append((idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1)))
    rim = [idx(i, 0) for i in range(n)] + [idx(n - 1, j) for j in range(1, n)] + \
          [idx(i, n - 1) for i in range(n - 2, -1, -1)] + [idx(0, j) for j in range(n - 2, 0, -1)]
    base_i = len(v)
    for k in rim:
        x, y, _ = v[k]
        v.append((x, y, -t / 2))
    m = len(rim)
    for a in range(m):
        b = (a + 1) % m
        f.append((rim[b], rim[a], base_i + a, base_i + b))
    f.append(tuple(base_i + a for a in reversed(range(m))))
    return _fw(Shape(v, f))

part('AncientPortraits__bar', f_box(0.412, 0.026, 0.020), lambda: frame_bar(0.412, 0.026, 0.020), smooth=40)


# =================================================== ancient book stacks ==
BOOK = f_box(0.11, 0.024, 0.07)


def book_cover(bw=0.11, bh=0.024, bd=0.07):
    # leather binding: two boards with eased edges and a raised panel, a
    # rounded spine at -X; open at the fore-edge and head and tail so the
    # page block (1.5 mm inside) shows between the boards
    bt = 0.0029
    boards = Shape()
    for sy in (-1, 1):
        b = rbox(bw - 0.0012, bt, bd, r=0.0009, seg=2).move(x=0.0006, y=sy * (bh / 2 - bt / 2))
        panel = rbox(bw - 0.020, 0.0008, bd - 0.016, r=0.0004, seg=1).move(x=0.002, y=sy * (bh / 2 - 0.0002))
        boards = boards + b + panel
    # spine: a half-round shell from board to board
    outer, inner = [], []
    for i in range(9):
        a = -math.pi / 2 + math.pi * i / 8
        outer.append((-bw / 2 + 0.0050 - 0.0050 * math.cos(a), (bh / 2 - 0.0004) * math.sin(a)))
        inner.append((-bw / 2 + 0.0062 - 0.0048 * math.cos(a), (bh / 2 - bt) * math.sin(a)))
    spine = prism(outer + list(reversed(inner)), bd - 0.0006)
    return fitf(boards + spine, BOOK)

part('AncientBookStacks__cover', BOOK, book_cover)


def page_block(w=0.107, h=0.018, d=0.067):
    # the text block: fine page-edge grooves round the three open sides and
    # a slightly hollowed fore-edge (+X)
    rows = []
    n = 9
    for k in range(n):
        y = -h / 2 + h * k / (n - 1)
        ins = 0.0 if k in (0, n - 1) else (0.00025 if k % 2 else 0.0)
        rows.append((y, ins))
    rings = []
    for y, ins in rows:
        t = y / (h / 2)
        ring = []
        for x, z in rrect_pts(w - 2 * ins, d - 2 * ins, 0.0012, 2):
            if x > 0:
                x -= 0.0012 * (1 - t * t)
            ring.append((x, y, z))
        rings.append(ring)
    return fitf(loft(rings), f_box(w, h, d))

part('AncientBookStacks__pages', f_box(0.107, 0.018, 0.067), page_block)
part('AncientBookStacks__band', f_box(0.001, 0.003, 0.07),
     lambda: rbox(0.001, 0.003, 0.07, r=0.00045, seg=1))


# ======================================================== rocking chairs ==
def rocker_seat():
    # saddle seat: dished top, rounded front edge (+Z; the back is at -Z)
    s = slab(0.30, 0.24, 0.020, 8, 6).rot(x=-math.pi / 2)

    def dish(p):
        if p.y > 0:
            r2 = (p.x / 0.15) ** 2 + ((p.z + 0.02) / 0.12) ** 2
            p.y -= 0.004 * max(0.0, 1 - r2)
        if p.z > 0.10:
            p.y -= 0.004 * ((p.z - 0.10) / 0.02) ** 2 * (1 if p.y > 0 else 0.3)
        return p
    return fitf(bevel(s.warp(dish), 0.004, seg=2, angle=40), f_box(0.30, 0.020, 0.24))

part('RockingChairs__seat', f_box(0.30, 0.020, 0.24), rocker_seat)

part('RockingChairs__leg', f_cyl(0.010, 0.010, 0.22), lambda: lathe(
    [(0.0, -0.11), (0.0072, -0.11), (0.0080, -0.104), (0.0070, -0.080), (0.0086, -0.040), (0.0092, -0.030),
     (0.0072, -0.024), (0.0092, -0.018), (0.0100, 0.030), (0.0090, 0.066), (0.0100, 0.074), (0.0100, 0.098),
     (0.0092, 0.106), (0.0, 0.11)], seg=10))

part('RockingChairs__spindle', f_cyl(0.006, 0.006, 0.20), lambda: lathe(
    [(0.0, -0.10), (0.0040, -0.10), (0.0042, -0.090), (0.0060, -0.050), (0.0058, -0.030), (0.0036, 0.020),
     (0.0032, 0.080), (0.0040, 0.092), (0.0040, 0.10), (0.0, 0.10)], seg=8), smooth=50)


def crest_rail():
    # top rail of the back: arched top edge with rounded ears, bowed back
    # (concave towards the sitter at +Z)
    top, bot = [], []
    for i in range(9):
        x = -0.15 + 0.30 * i / 8
        u = x / 0.15
        top.append((x, 0.004 + 0.006 * (1 - u * u) ** 0.7))
        bot.append((x, -0.010 + 0.0015 * u * u))
    out = round_outline(bot + list(reversed(top)), 0.003, seg=1)
    s = prism(out, 0.009, bevel_r=0.0015)

    def bow(p):
        u = p.x / 0.15
        p.z -= 0.0025 * (1 - u * u)
        return p
    return fitf(s.warp(bow), f_box(0.30, 0.020, 0.014))

part('RockingChairs__rail', f_box(0.30, 0.020, 0.014), crest_rail)


def rocker_rail():
    # curved rocker: an arced underside rising towards the ends, a flatter
    # top, the ends tapering to rounded tips
    top, bot = [], []
    for i in range(9):
        x = -0.18 + 0.36 * i / 8
        u = x / 0.18
        bot.append((x, -0.009 + 0.008 * u * u))
        top.append((x, 0.009 - 0.004 * u ** 4))
    out = round_outline(bot + list(reversed(top)), 0.004, seg=1)
    return fitf(prism(out, 0.030, bevel_r=0.003), f_box(0.36, 0.018, 0.030))

part('RockingChairs__rocker', f_box(0.36, 0.018, 0.030), rocker_rail)


def arm_rest():
    # arm rail: flattened oval section, a waist in the middle and rounded
    # hand grips at both ends
    rings = []
    for y, k in ((-0.100, 0.35), (-0.098, 0.78), (-0.094, 0.97), (-0.084, 1.0), (-0.066, 0.86), (0.0, 0.80),
                 (0.066, 0.86), (0.084, 1.0), (0.094, 0.97), (0.098, 0.78), (0.100, 0.35)):
        rings.append([(x, y, z) for x, z in sellipse_pts(0.008 * k, 0.008 * k * 0.8, 10, 2.6)])
    return fitf(loft(rings), f_cyl(0.008, 0.008, 0.20))

part('RockingChairs__arm', f_cyl(0.008, 0.008, 0.20), arm_rest, smooth=50)
