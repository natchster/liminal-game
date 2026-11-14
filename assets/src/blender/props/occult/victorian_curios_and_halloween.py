"""Family `victorian_curios_and_halloween`: Victorian curios and occult / Halloween props.

Victorian: pocket compass, brass telescope, desk globe, phonograph, hanging
brass lantern, telegraph key, balance scales, abacus, trophy, handbell.
Occult: mask, candle clusters, crystal ball stand, pumpkin, witch hat,
skeleton hand, cauldron, tombstone, spider, broom, black cat, treat pail,
skulls, potion cork, owl, brass candlestick.

Transparent glass, glows, flames, halos, dark basic material eye/face decals and
the DoubleSide bat and ghost sheet stay primitives; see docs/research/props-remodelling/victorian_curios_and_halloween.md.
"""
from propkit import *
import propkit


# ------------------------------------------------------------ local kit --
def _fw(s):
    return propkit._fix_winding(s)


def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


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
    """Sweep a closed 2D section [(dr, dz), ...] round the Z axis in the XY
    plane (a torus_geo! with any cross-section). arc < TAU caps the ends.
    rmod(angle, t) -> (k_r, dz_add[, dR]) varies the section along the ring."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        mm = rmod(a, i / seg) if rmod else (1.0, 0.0)
        kr, dzz = mm[0], mm[1]
        dR = mm[2] if len(mm) > 2 else 0.0
        kz = mm[3] if len(mm) > 3 else kr
        for (dr, dz) in sec:
            rr_ = R + dR + dr * kr
            v.append((rr_ * ca, rr_ * sa, dz * kz + dzz))
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


def ring_lathe(profile, seg=24):
    """Revolve a *closed* (r, y) loop (no point on the axis): tubes, bands."""
    s = lathe(list(profile) + [profile[0]], seg, cap_bottom=False, cap_top=False)
    return _fw(weld(s))


def x_axis(s):
    """Y-axis shape laid along +X (its +Y end goes to +X)."""
    return s.rot(z=-math.pi / 2)


def rrect_pts(w, d, r, cseg=2):
    """Rounded rectangle (x, z) outline with 4*(cseg+1) points."""
    r = max(1e-5, min(r, w / 2 - 1e-5, d / 2 - 1e-5))
    out = []
    for cx, cz, a0 in ((w / 2 - r, d / 2 - r, 0), (-w / 2 + r, d / 2 - r, 90),
                       (-w / 2 + r, -d / 2 + r, 180), (w / 2 - r, -d / 2 + r, 270)):
        for i in range(cseg + 1):
            a = math.radians(a0 + 90 * i / cseg)
            out.append((cx + r * math.cos(a), cz + r * math.sin(a)))
    return out


def yloft(profile, w, d, rc, cseg=2):
    """Stack of rounded rectangles along +Y: profile = [(y, inset), ...]."""
    rings = []
    for y, ins in profile:
        pts = rrect_pts(w - 2 * ins, d - 2 * ins, max(0.0003, rc - ins * 0.6), cseg)
        rings.append([(x, y, z) for x, z in pts])
    return loft(rings)


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


def twist_cord(r, h, rings=16, turn=0.6, lobes=2, depth=0.22, n=8):
    """A plied cord along Y: a lobed section that turns as it goes."""
    rs = []
    for k in range(rings + 1):
        y = -h / 2 + h * k / rings
        ring = []
        for i in range(n):
            a = TAU * i / n
            rr_ = r * (1 - depth * 0.5 + depth * 0.5 * math.cos(lobes * (a - turn * k)))
            ring.append((rr_ * math.sin(a), y, rr_ * math.cos(a)))
        rs.append(ring)
    return loft(rs)


def lobe_warp(n, amt, rmin=0.0, phase=0.0):
    """Alternate the radius round Y: n lobes (ribs, petals, knurls)."""
    def w(p):
        r = math.hypot(p.x, p.z)
        if r > rmin:
            a = math.atan2(p.x, p.z) + phase
            k = 1 + amt * math.cos(n * a)
            p.x *= k
            p.z *= k
        return p
    return w


# ===================================================== small compasses ==
def compass_case():
    # pocket-compass case: rounded brass bowl, a band, a raised bezel and a
    # shallow well inside it where the printed face sits
    return lathe([(0.0, -0.006), (0.033, -0.006), (0.036, -0.003), (0.036, 0.0025), (0.0354, 0.0032),
                  (0.0356, 0.005), (0.0344, 0.006), (0.031, 0.006), (0.0306, 0.0048), (0.0, 0.0048)], seg=16)

part('SmallCompasses__case', f_cyl(0.036, 0.036, 0.012), compass_case)


def compass_hinge():
    # hinge barrel: three knuckles split by two grooves, domed pin ends
    return lathe([(0.0, -0.007), (0.005, -0.0064), (0.005, -0.0027), (0.0041, -0.0023), (0.005, -0.0019),
                  (0.005, 0.0019), (0.0041, 0.0023), (0.005, 0.0027), (0.005, 0.0064), (0.0, 0.007)], seg=6)

part('SmallCompasses__hinge', f_cyl(0.005, 0.005, 0.014), compass_hinge)


def compass_cover():
    # the opened lid: domed outside (-Y), a rim and a dished inside (+Y)
    # that faces the dial when the lid stands open
    return lathe([(0.0, -0.003), (0.02, -0.0027), (0.030, -0.002), (0.0345, -0.001), (0.036, 0.0005),
                  (0.036, 0.0025), (0.0352, 0.003), (0.0335, 0.003), (0.033, 0.0012), (0.0, 0.0012)], seg=16)

part('SmallCompasses__cover', f_cyl(0.036, 0.036, 0.006), compass_cover)


# ==================================================== small telescopes ==
def scope_tube():
    # one brass draw tube: a ferrule collar with a bead at -Y (where the
    # next tube slides out, or the objective cell), a thin lip at +Y
    return lathe([(0.0, -0.045), (0.0126, -0.045), (0.0146, -0.0443), (0.015, -0.041), (0.015, -0.0375),
                  (0.0141, -0.0362), (0.0144, -0.0335), (0.0135, -0.0322), (0.0134, 0.043), (0.0139, 0.045),
                  (0.0, 0.045)], seg=14)

part('SmallTelescopes__tube', f_cyl(0.015, 0.015, 0.09), scope_tube)


def scope_grip():
    # stitched leather wrap: soft rounded edges and a stitch line near each
    return lathe([(0.0, -0.01), (0.0195, -0.01), (0.0207, -0.0092), (0.021, -0.0075), (0.0205, -0.0068),
                  (0.021, -0.006), (0.021, 0.006), (0.0205, 0.0068), (0.021, 0.0075), (0.0207, 0.0092),
                  (0.0195, 0.01), (0.0, 0.01)], seg=14)

part('SmallTelescopes__grip', f_cyl(0.021, 0.021, 0.020), scope_grip)


# ======================================================== small globes ==
def globe_base():
    # turned wooden plinth: foot band, cove and a rounded top edge
    return lathe([(0.0, -0.005), (0.0535, -0.005), (0.055, -0.0036), (0.055, -0.002), (0.0515, -0.0006),
                  (0.049, 0.0016), (0.0462, 0.0031), (0.045, 0.0046), (0.0435, 0.005), (0.0, 0.005)], seg=28)

part('SmallGlobes__base', f_cyl(0.045, 0.055, 0.010), globe_base)


def globe_column():
    # turned baluster: foot collar, vase belly, a ring and a capital
    return lathe([(0.0, -0.025), (0.008, -0.025), (0.008, -0.022), (0.0062, -0.0205), (0.0066, -0.018),
                  (0.0055, -0.0165), (0.0068, -0.008), (0.0045, 0.006), (0.004, 0.016), (0.0058, 0.018),
                  (0.0048, 0.0205), (0.006, 0.022), (0.006, 0.025), (0.0, 0.025)], seg=10)

part('SmallGlobes__column', f_cyl(0.006, 0.008, 0.050), globe_column)


def globe_meridian():
    # flat brass meridian band with chamfered outer edges; two pivot pins
    # point in to the poles
    sec = [(0.003, -0.0024), (0.003, 0.0024), (0.0024, 0.003), (-0.002, 0.003), (-0.002, -0.003),
           (0.0024, -0.003)]
    ring = ring_section(0.055, sec, seg=36)
    pins = Shape()
    for sy in (-1, 1):
        pin = lathe([(0.0, 0.0), (0.0016, 0.0), (0.0016, 0.0022), (0.0010, 0.0034), (0.0, 0.0036)], seg=6)
        pins = pins + pin.scale(1, -sy, 1).move(y=sy * 0.0535)
    return _fw(ring + pins)

part('SmallGlobes__meridian', f_torus(0.055, 0.003), globe_meridian)
part('SmallGlobes__sphere', f_sphere(0.050), lambda: sphere(0.05, seg=20, rings=14), smooth=180)


# ==================================================== small phonographs ==
def phono_base():
    # cabinet: plinth moulding, body set in 5 mm with a sunk panel on each
    # side, a rounded top moulding and a crank boss on the right side
    plinth = rbox(0.16, 0.009, 0.14, r=0.003, seg=1).move(y=-0.0255)
    body = rbox(0.150, 0.044, 0.130, r=0.0015, seg=1)
    body = cut(body, box(0.12, 0.026, 0.004).move(z=0.066), box(0.12, 0.026, 0.004).move(z=-0.066),
               box(0.004, 0.026, 0.10).move(x=0.076), box(0.004, 0.026, 0.10).move(x=-0.076))
    top = rbox(0.16, 0.009, 0.14, r=0.0035, seg=2).move(y=0.0255)
    boss = x_axis(lathe([(0.0, 0.0), (0.008, 0.0), (0.008, 0.0015), (0.0055, 0.0032), (0.0, 0.0032)], seg=12))
    return plinth + body + top + boss.move(x=0.0735)

part('SmallPhonographs__base', f_box(0.16, 0.06, 0.14), phono_base)


def phono_platter():
    # turntable platter: flat felt top, a stepped, rounded rim
    return lathe([(0.0, -0.003), (0.0525, -0.003), (0.0548, -0.0018), (0.055, 0.0005), (0.0545, 0.0022),
                  (0.053, 0.003), (0.0, 0.003)], seg=28)

part('SmallPhonographs__turntable', f_cyl(0.055, 0.055, 0.006), phono_platter)


def phono_record():
    # 78 rpm disc: beaded edge, three bands of grooves split by dips, a
    # plain run-out ring and a raised label area
    return lathe([(0.0, -0.001), (0.0508, -0.001), (0.052, -0.0002), (0.0514, 0.0009), (0.049, 0.0007),
                  (0.041, 0.0007), (0.040, 0.0004), (0.039, 0.0007), (0.031, 0.0007), (0.030, 0.0004),
                  (0.029, 0.0007), (0.0165, 0.0006), (0.0158, 0.001), (0.0, 0.001)], seg=22)

part('SmallPhonographs__record', f_cyl(0.052, 0.052, 0.002), phono_record)


def phono_arm():
    # tone arm: pivot collar at -Y, a slim tapering tube, a socket at +Y
    return lathe([(0.0, -0.04), (0.0028, -0.04), (0.003, -0.038), (0.003, -0.033), (0.0022, -0.031),
                  (0.0018, 0.036), (0.0024, 0.037), (0.0024, 0.04), (0.0, 0.04)], seg=8)

part('SmallPhonographs__arm', f_cyl(0.003, 0.003, 0.08), phono_arm, smooth=50)


def phono_head():
    # reproducer: a rounded sound box with the needle clamp underneath
    boxy = rbox(0.010, 0.0042, 0.008, r=0.0014, seg=1).move(y=0.0009)
    needle = lathe([(0.0, -0.003), (0.0006, -0.0028), (0.0010, -0.0010), (0.0010, 0.0), (0.0, 0.0)], seg=6)
    return boxy + needle.move(x=0.003)

part('SmallPhonographs__head', f_box(0.010, 0.006, 0.008), phono_head)


def phono_horn():
    # morning-glory horn: eight bulging panels, thin sheet, a rolled rim at
    # the wide mouth (-Y), the throat at the apex (+Y)
    prof = [(0.0075, -0.1), (0.012, -0.065), (0.020, -0.03), (0.033, 0.01), (0.053, 0.058), (0.0795, 0.1)]
    s = vessel(prof, 0.0022, seg=16, floor=0.012).warp(lobe_warp(8, 0.035, 0.004))
    return fitf(s.rot(x=math.pi), f_cone(0.08, 0.20))

part('SmallPhonographs__horn', f_cone(0.08, 0.20), phono_horn)


# =============================================== hanging brass lanterns ==
def link_chain(L, pitch, a, b, j):
    """Chain along Y: links alternating flat in X and flat in Z (half
    widths a x b), pinched to radius j where they pass through each other."""
    n = max(2, int(round(L / pitch)))
    p = L / n
    rings = []

    def ring(y, ra, rb):
        return [(ra * math.cos(TAU * i / 6), y, rb * math.sin(TAU * i / 6)) for i in range(6)]
    rings.append(ring(-L / 2, j, j))
    for k in range(n):
        y0 = -L / 2 + k * p
        ra, rb = (a, b) if k % 2 == 0 else (b, a)
        rings += [ring(y0 + p * 0.28, ra, rb), ring(y0 + p * 0.72, ra, rb), ring(y0 + p, j, j)]
    return loft(rings)


def lantern_chain(L=0.7):
    # hanging chain from the ceiling
    return fitf(link_chain(L, 0.026, 0.003, 0.0011, 0.0012), f_cyl(0.003, 0.003, L))

part('HangingBrassLanterns__chain', f_cyl(0.003, 0.003, 0.7), lantern_chain, smooth=60)


def lantern_cap():
    # square pyramid roof: an eave lip, a stepped slope and a round boss
    # on top where the chain ring is fixed
    roof = yloft([(-0.0075, 0.0), (-0.0056, 0.0), (-0.005, 0.0022), (-0.0042, 0.0026),
                  (0.0032, 0.025), (0.0042, 0.0275)], 0.08, 0.08, 0.004, cseg=2)
    boss = lathe([(0.0, 0.003), (0.0075, 0.003), (0.0075, 0.0055), (0.006, 0.0072), (0.0, 0.0075)], seg=10)
    return roof + boss

part('HangingBrassLanterns__topCap', f_cyl(0.030, 0.040, 0.015), lantern_cap)
part('HangingBrassLanterns__botCap', f_cyl(0.040, 0.030, 0.015), lambda: _fw(lantern_cap().scale(1, -1, 1)))


def lantern_post():
    # a slim turned corner post with ball-and-collar ends
    return lathe([(0.0, -0.07), (0.0026, -0.07), (0.003, -0.0675), (0.003, -0.0645), (0.0017, -0.062),
                  (0.0015, -0.001), (0.0023, 0.0), (0.0015, 0.001), (0.0017, 0.062), (0.003, 0.0645),
                  (0.003, 0.0675), (0.0026, 0.07), (0.0, 0.07)], seg=6)

part('HangingBrassLanterns__post', f_cyl(0.003, 0.003, 0.14), lantern_post, smooth=50)


# ================================================== small telegraph keys ==
def key_base():
    # wooden base: a plain slab with a stepped, rounded top and four
    # domed screws holding it down
    low = rbox(0.08, 0.006, 0.06, r=0.0015, seg=1).move(y=-0.002)
    up = rbox(0.074, 0.0045, 0.054, r=0.0018, seg=2).move(y=0.00275)
    screws = Shape()
    for sx in (-1, 1):
        for sz in (-1, 1):
            sc = lathe([(0.0, 0.0), (0.0022, 0.0), (0.0019, 0.0005), (0.0, 0.0008)], seg=6)
            screws = screws + sc.move(x=sx * 0.031, y=0.0048, z=sz * 0.021)
    return fitf(low + up + screws, f_box(0.08, 0.010, 0.06))

part('SmallTelegraphKeys__base', f_box(0.08, 0.010, 0.06), key_base)


def key_post():
    # trunnion pillar: a flange on the base, a slim shaft, the pivot-screw
    # head at the top
    return lathe([(0.0, -0.01), (0.004, -0.01), (0.004, -0.0085), (0.0027, -0.0074), (0.0027, 0.0058),
                  (0.0035, 0.0064), (0.0035, 0.0088), (0.0, 0.01)], seg=6)

part('SmallTelegraphKeys__post', f_cyl(0.004, 0.004, 0.020), key_post)


def key_lever():
    # key lever seen from above: round pivot boss, a bar narrowing to the
    # knob end and a rounded spring end
    pts = [(-0.030, -0.0022), (-0.013, -0.0026), (-0.0105, -0.004), (-0.0055, -0.004), (-0.003, -0.0026),
           (0.026, -0.0021), (0.030, -0.0012), (0.030, 0.0012), (0.026, 0.0021), (-0.003, 0.0026),
           (-0.0055, 0.004), (-0.0105, 0.004), (-0.013, 0.0026), (-0.030, 0.0022)]
    s = prism(round_outline(pts, 0.0012, seg=1), 0.004, bevel_r=0.0005, bevel_seg=1).rot(x=math.pi / 2)
    return fitf(s, f_box(0.06, 0.004, 0.008))

part('SmallTelegraphKeys__lever', f_box(0.06, 0.004, 0.008), key_lever)


def key_knob():
    # bakelite mushroom knob (the game squashes it to half height): a
    # rounded cap with a flat top over a short neck
    return lathe([(0.0, -0.01), (0.0038, -0.01), (0.0034, -0.0045), (0.0048, -0.0028), (0.0092, -0.0016),
                  (0.010, 0.0006), (0.0094, 0.0042), (0.0068, 0.0078), (0.0032, 0.0096), (0.0, 0.01)], seg=12)

part('SmallTelegraphKeys__knob', f_sphere(0.010), key_knob, smooth=60)


def key_terminal():
    # binding post: a nut on the base, a threaded stem, a small cap
    return lathe([(0.0, -0.004), (0.004, -0.004), (0.004, -0.0012), (0.0026, -0.0006), (0.0026, 0.0022),
                  (0.0033, 0.0028), (0.0028, 0.004), (0.0, 0.004)], seg=8)

part('SmallTelegraphKeys__terminal', f_cyl(0.004, 0.004, 0.008), key_terminal)
part('SmallTelegraphKeys__wire', f_cyl(0.001, 0.001, 0.10),
     lambda: twist_cord(0.001, 0.10, rings=14, turn=0.9, lobes=2, depth=0.35, n=6), smooth=60)


# ========================================================= small scales ==
def scales_base():
    # cast brass base: foot band, a step and a domed top
    return lathe([(0.0, -0.005), (0.054, -0.005), (0.055, -0.0038), (0.055, -0.0024), (0.0505, -0.0018),
                  (0.0495, 0.0006), (0.0465, 0.002), (0.0452, 0.0042), (0.0425, 0.005), (0.0, 0.005)], seg=24)

part('SmallScales__base', f_cyl(0.045, 0.055, 0.010), scales_base)


def scales_pillar():
    # turned pillar: a foot collar, a slim shaft with a middle ring, a
    # capital that carries the beam
    return lathe([(0.0, -0.05), (0.005, -0.05), (0.005, -0.046), (0.0034, -0.0435), (0.0031, -0.002),
                  (0.0042, 0.0), (0.0031, 0.002), (0.0031, 0.040), (0.0046, 0.0425), (0.0046, 0.0455),
                  (0.0036, 0.0475), (0.0, 0.05)], seg=8)

part('SmallScales__pillar', f_cyl(0.005, 0.005, 0.10), scales_pillar)


def scales_beam():
    # balance beam: a pivot boss in the middle, arms tapering outwards and
    # a knob at each end where the chains hang
    return lathe([(0.0, -0.07), (0.0022, -0.07), (0.0026, -0.068), (0.0026, -0.063), (0.0015, -0.061),
                  (0.0021, -0.006), (0.003, -0.004), (0.003, 0.004), (0.0021, 0.006), (0.0015, 0.061),
                  (0.0026, 0.063), (0.0026, 0.068), (0.0022, 0.07), (0.0, 0.07)], seg=8)

part('SmallScales__beam', f_cyl(0.003, 0.003, 0.14), scales_beam, smooth=50)
part('SmallScales__chain', f_cyl(0.001, 0.001, 0.030),
     lambda: fitf(link_chain(0.03, 0.01, 0.001, 0.0004, 0.00045), f_cyl(0.001, 0.001, 0.03)), smooth=60)


def scales_pan():
    # shallow spun-brass pan with a rolled rim
    s = vessel([(0.017, -0.002), (0.022, -0.0016), (0.0248, -0.0002), (0.026, 0.0017)], 0.0008, seg=16, floor=0.0008)
    return fitf(s, f_cyl(0.024, 0.026, 0.004))

part('SmallScales__pan', f_cyl(0.024, 0.026, 0.004), scales_pan)


# ======================================================= small abacuses ==
# One constructor makes both the long rails and the short stiles, so the
# frame bar only gets a small even chamfer that survives either stretch.
part('SmallAbacuses__bar', f_box(0.15, 0.012, 0.008), lambda: rbox(0.15, 0.012, 0.008, r=0.0007, seg=1))
part('SmallAbacuses__rod', f_cyl(0.0015, 0.0015, 0.13),
     lambda: cyl(0.0015, 0.0015, 0.13, seg=8, bevel_r=0.0004, bevel_seg=1))


def abacus_bead():
    # bicone bead with a rounded waist
    return lathe([(0.0, -0.005), (0.0022, -0.005), (0.0062, -0.0013), (0.007, 0.0), (0.0062, 0.0013),
                  (0.0022, 0.005), (0.0, 0.005)], seg=10)

part('SmallAbacuses__bead', f_cyl(0.007, 0.007, 0.010), abacus_bead, smooth=50)


# ======================================================== small trophies ==
def trophy_base():
    # dark wooden plinth: rounded top edges and a groove cut round it
    # above the foot
    s = rbox(0.10, 0.03, 0.10, r=0.0025, seg=2)
    g = 0.0016
    cutters = [box(0.104, g, 0.002).move(y=-0.0085, z=0.05), box(0.104, g, 0.002).move(y=-0.0085, z=-0.05),
               box(0.002, g, 0.104).move(y=-0.0085, x=0.05), box(0.002, g, 0.104).move(y=-0.0085, x=-0.05)]
    return cut(s, *cutters)

part('SmallTrophies__base', f_box(0.10, 0.03, 0.10), trophy_base)


def trophy_plaque():
    # engraved brass plate with clipped corners, 0.9 mm thick behind its face
    c = 0.003
    pts = [(-0.035 + c, -0.01), (0.035 - c, -0.01), (0.035, -0.01 + c), (0.035, 0.01 - c),
           (0.035 - c, 0.01), (-0.035 + c, 0.01), (-0.035, 0.01 - c), (-0.035, -0.01 + c)]
    return prism(pts, 0.0009, bevel_r=0.0003, bevel_seg=1).move(z=-0.00045)

part('SmallTrophies__plaque', f_bounds((-0.035, -0.01, 0), (0.035, 0.01, 0), 'plane'), trophy_plaque)


def trophy_stem():
    # turned stem: a spreading foot, a slim shaft with a knop, a flared
    # cup socket at the top
    return lathe([(0.0, -0.05), (0.0148, -0.05), (0.015, -0.0475), (0.0115, -0.0435), (0.0062, -0.035),
                  (0.0052, -0.018), (0.009, -0.0055), (0.0052, 0.006), (0.0055, 0.036), (0.0102, 0.05),
                  (0.0, 0.05)], seg=12)

part('SmallTrophies__stem', f_cyl(0.015, 0.015, 0.10), trophy_stem)


def trophy_cup():
    # loving-cup bowl: tulip profile with a raised band, thin wall, rolled lip
    prof = [(0.025, -0.03), (0.0335, -0.021), (0.0365, -0.0165), (0.0358, -0.0135), (0.0415, -0.003),
            (0.0465, 0.013), (0.0495, 0.028)]
    return fitf(vessel(prof, 0.0022, seg=20, floor=0.006), f_cyl(0.05, 0.025, 0.06))

part('SmallTrophies__cup', f_cyl(0.05, 0.025, 0.06), trophy_cup)

F_TROPHY_HANDLE = torus_arc_bounds(0.020, 0.003, math.pi)


def trophy_handle():
    # ribbon handle: a flat oval section that swells into pads where it
    # joins the cup
    sec = circ_sec(1.0, 8, sx=0.0019, sy=0.003)

    def rm(a, t):
        e = max(0.0, abs(t - 0.5) * 2 - 0.72) / 0.28
        return (1.0 + 0.7 * e, 0.0, 0.0, 1.0)
    return fitf(ring_section(0.020, sec, seg=16, arc=math.pi, rmod=rm), F_TROPHY_HANDLE)

part('SmallTrophies__handle', F_TROPHY_HANDLE, trophy_handle)


# ======================================================= small handbells ==
F_BELL_DOME = f_bounds((-0.045, 0.0, -0.045), (0.045, 0.045, 0.045))


def bell_dome():
    # handbell: flat crown, shoulder, concave waist and a flared sound bow;
    # the mouth (open, with a thick lip) is at y 0 like the hemisphere's rim
    prof = [(0.017, 0.0), (0.0225, 0.0025), (0.0255, 0.009), (0.028, 0.024), (0.036, 0.036), (0.045, 0.045)]
    s = vessel(prof, 0.0026, seg=18, floor=0.003).rot(x=math.pi)
    return fitf(s, F_BELL_DOME)

part('SmallHandbells__dome', F_BELL_DOME, bell_dome)
part('SmallHandbells__ring', f_torus(0.008, 0.002),
     lambda: ring_section(0.008, [(0.0019, -0.002), (0.0019, 0.002), (-0.0019, 0.002), (-0.0019, -0.002)], seg=12))


def bell_handle():
    # turned wooden handle: a collar at the bell, a waisted grip and a
    # round knob at the top
    return lathe([(0.0, -0.04), (0.0085, -0.04), (0.010, -0.037), (0.0085, -0.034), (0.006, -0.03),
                  (0.0055, -0.02), (0.0066, 0.0), (0.0072, 0.015), (0.0055, 0.027), (0.0072, 0.031),
                  (0.008, 0.035), (0.0066, 0.0392), (0.0, 0.04)], seg=10)

part('SmallHandbells__handle', f_cyl(0.008, 0.010, 0.08), bell_handle, smooth=60)
part('SmallHandbells__clapper', f_sphere(0.006), lambda: sphere(0.006, seg=10, rings=6), smooth=180)


# ========================================================== small masks ==
F_MASK = f_bounds((-0.06, 0.0, -0.06), (0.06, 0.06, 0.06))


def mask_face(nr=7, nt=24):
    # a full-face mask lying face up (the game squashes it to 15 % height):
    # an egg-shaped plate, forehead to -Z, chin to +Z, with brow, eye
    # hollows, a nose ridge, cheeks and a mouth slit pressed into it
    def outline(a):
        c = math.cos(a)
        return 0.06 * math.sin(a) * (1 - 0.2 * max(0.0, c) ** 2), 0.06 * c

    def g(x, z, cx, cz, sx, sz):
        return math.exp(-((x - cx) / sx) ** 2 - ((z - cz) / sz) ** 2)

    def h(x, z):
        d = max(0.0, 1 - (x / 0.06) ** 2 - (z / 0.062) ** 2)
        y = 0.012 + 0.036 * math.sqrt(d)
        y += 0.008 * g(x, z, 0, 0.008, 0.0055, 0.016)                       # nose
        y += 0.003 * (g(x, z, -0.02, -0.021, 0.013, 0.005) + g(x, z, 0.02, -0.021, 0.013, 0.005))
        y -= 0.005 * (g(x, z, -0.02, -0.008, 0.009, 0.007) + g(x, z, 0.02, -0.008, 0.009, 0.007))
        y += 0.003 * (g(x, z, -0.03, 0.016, 0.012, 0.012) + g(x, z, 0.03, 0.016, 0.012, 0.012))
        y -= 0.004 * g(x, z, 0, 0.036, 0.012, 0.0025)                       # mouth
        return y
    v, f = [], []
    v.append(Vector((0, h(0, 0), 0)))
    for i in range(1, nr + 1):
        u = i / nr
        for j in range(nt):
            ox, oz = outline(TAU * j / nt)
            x, z = ox * u, oz * u
            yy = h(x, z) if i < nr else 0.006
            v.append(Vector((x, yy, z)))
    ring = lambda i, j: 1 + (i - 1) * nt + (j % nt)
    for j in range(nt):
        f.append((0, ring(1, j + 1), ring(1, j)))
    for i in range(1, nr):
        for j in range(nt):
            f.append((ring(i, j), ring(i, j + 1), ring(i + 1, j + 1), ring(i + 1, j)))
    # rolled edge down to a flat underside
    base = len(v)
    for j in range(nt):
        ox, oz = outline(TAU * j / nt)
        v.append(Vector((ox * 0.985, 0.0, oz * 0.985)))
    for j in range(nt):
        f.append((ring(nr, j), ring(nr, j + 1), base + (j + 1) % nt, base + j))
    f.append(tuple(base + j for j in reversed(range(nt))))
    return fitf(_fw(Shape(v, f)), F_MASK)

part('SmallMasks__face', F_MASK, mask_face, smooth=60)
part('SmallMasks__strap', f_cyl(0.002, 0.002, 0.14),
     lambda: twist_cord(0.002, 0.14, rings=18, turn=1.1, lobes=3, depth=0.25, n=6), smooth=60)


# ======================================================= candle clusters ==
def candle_body():
    # wax candle: soft shoulder, a melted dish round the wick and two
    # runs of wax down the side
    body = lathe([(0.0, -0.03), (0.0071, -0.03), (0.0074, -0.0293), (0.0074, 0.026), (0.0071, 0.0288),
                  (0.0058, 0.0299), (0.0036, 0.0292), (0.0014, 0.0293), (0.0, 0.0297)], seg=10)
    drips = Shape()
    for a, y0, ln in ((0.6, 0.029, 0.014), (3.4, 0.0295, 0.009)):
        d = lathe([(0.0, -ln / 2), (0.0011, -ln / 2 + 0.0012), (0.0013, -ln / 2 + 0.003), (0.0008, ln / 2 - 0.002),
                   (0.0, ln / 2)], seg=6).scale(1, 1, 0.65)
        d = d.move(z=0.0073).rot(y=a).move(y=y0 - ln / 2)
        drips = drips + d
    return fitf(body + drips, f_cyl(0.008, 0.008, 0.06))

part('CandleClusters__body', f_cyl(0.008, 0.008, 0.06), candle_body, smooth=60)


def candle_wick(h=0.008, r=0.001):
    # a charred wick that curls over at the tip
    s = tube([(0, -h / 2, 0), (0, h * 0.1, 0), (r * 0.5, h * 0.35, 0), (r * 1.2, h / 2 - r * 0.6, 0)],
             r, seg=5, smooth=1, r_end=r * 0.7)
    return fitf(s, f_cyl(0.001, 0.001, h))

part('CandleClusters__wick', f_cyl(0.001, 0.001, 0.008), candle_wick, smooth=60)


# =================================================== small crystal balls ==
def crystal_base():
    # dark stand foot: a gadrooned (lobed) skirt stepping up to a moulded top
    s = lathe([(0.0, -0.004), (0.0435, -0.004), (0.045, -0.0028), (0.045, -0.0012), (0.041, -0.0004),
               (0.0385, 0.0012), (0.036, 0.0028), (0.0345, 0.004), (0.0, 0.004)], seg=24)
    return fitf(s.warp(lobe_warp(12, 0.02, 0.037)), f_cyl(0.035, 0.045, 0.008))

part('SmallCrystalBalls__base', f_cyl(0.035, 0.045, 0.008), crystal_base)


def crystal_prong():
    # a claw holding the ball: knuckle, tapering finger, hooked point
    s = lathe([(0.0, -0.009), (0.0026, -0.009), (0.003, -0.0075), (0.0022, -0.003), (0.0026, 0.001),
               (0.0018, 0.005), (0.0008, 0.0085), (0.0, 0.009)], seg=6)
    s = s.warp(lambda p: Vector((p.x + 0.0012 * max(0.0, (p.y + 0.002) / 0.011) ** 2, p.y, p.z)))
    return fitf(s, f_cyl(0.003, 0.003, 0.018))

part('SmallCrystalBalls__prong', f_cyl(0.003, 0.003, 0.018), crystal_prong, smooth=50)


# ======================================================== small pumpkins ==
def pumpkin_warp(n, amt, rmin=0.0):
    """Pumpkin ribs: round lobes, sharp grooves between them."""
    def w(p):
        r = math.hypot(p.x, p.z)
        if r > rmin:
            a = math.atan2(p.x, p.z)
            k = (1 - amt) + amt * abs(math.cos(n * a / 2)) ** 0.5
            p.x *= k
            p.z *= k
        return p
    return w


def pumpkin_body(R, n=10, dimple=0.003):
    # squat ribbed pumpkin with a dimple top and bottom
    s = lathe([(0.0, -R * 0.78), (R * 0.24, -R * 0.84), (R * 0.64, -R * 0.76), (R * 0.95, -R * 0.38), (R, 0.02 * R),
               (R * 0.94, R * 0.42), (R * 0.64, R * 0.78), (R * 0.26, R * 0.855), (0.0, R * 0.855 - dimple)], seg=n * 4)
    return fitf(s.warp(pumpkin_warp(n, 0.1, R * 0.1)), f_sphere(R))

part('SmallPumpkins__body', f_sphere(0.055), lambda: pumpkin_body(0.055), smooth=50)


def pumpkin_stem():
    # woody stem: five ribs, flared where it joins, leaning, cut top
    s = lathe([(0.0, -0.01), (0.0098, -0.01), (0.0082, -0.0078), (0.0068, -0.0045), (0.0062, 0.004),
               (0.0066, 0.0085), (0.0058, 0.01), (0.0, 0.0094)], seg=15)
    s = s.warp(lobe_warp(5, 0.12, 0.002))
    s = s.warp(lambda p: Vector((p.x + 0.0022 * ((p.y + 0.01) / 0.02) ** 2, p.y, p.z)))
    return fitf(s, f_cyl(0.008, 0.010, 0.020))

part('SmallPumpkins__stem', f_cyl(0.008, 0.010, 0.020), pumpkin_stem)


# ====================================================== small witch hats ==
def hat_brim():
    # felt brim: thin, rounded edge, thicker toward the crown, the outer
    # part rippling in five soft waves and curling up a little
    s = lathe([(0.0, -0.0009), (0.074, -0.0008), (0.0785, -0.0005), (0.08, 0.0), (0.0785, 0.0005),
               (0.074, 0.0008), (0.046, 0.0011), (0.0, 0.0011)], seg=40)

    def w(p):
        r = math.hypot(p.x, p.z)
        if r > 0.046:
            e = ((r - 0.046) / 0.034) ** 2
            p.y += e * (0.0011 * math.sin(5 * math.atan2(p.x, p.z) + 0.7) + 0.0009)
        return p
    return fitf(s.warp(w), f_cyl(0.08, 0.08, 0.006))

part('SmallWitchHats__brim', f_cyl(0.080, 0.080, 0.006), hat_brim, smooth=50)


def spine_loft(radii, H, bend, n=16, dent=None):
    """Rings along a spine that stays vertical, then bends over toward +X
    (total `bend` radians over the top part). radii[k] per ring."""
    m = len(radii) - 1
    ds = H / m
    P = Vector((0.0, -H / 2, 0.0))
    rings = []
    for k in range(m + 1):
        t = k / m
        phi = bend * max(0.0, (t - 0.45) / 0.55) ** 1.6
        T = Vector((math.sin(phi), math.cos(phi), 0.0))
        N = Vector((math.cos(phi), -math.sin(phi), 0.0))
        Z = Vector((0.0, 0.0, 1.0))
        ring = []
        for i in range(n):
            a = TAU * i / n
            rr_ = radii[k] * (dent(k, a) if dent else 1.0)
            ring.append(tuple(P + N * (rr_ * math.cos(a)) + Z * (rr_ * math.sin(a))))
        rings.append(ring)
        P = P + T * ds
    return rings


def hat_cone():
    # felt crown: a tall cone whose top third flops over to one side, with
    # soft dents where the felt creases at the knee
    m = 13
    radii = [0.045 * (1 - k / m) ** 0.9 + (0.0006 if k == m else 0.0) for k in range(m + 1)]

    def dent(k, a):
        if 6 <= k <= 9:
            return 1 - 0.07 * max(0.0, math.cos(a - math.pi)) ** 2 * (1 - abs(k - 7.5) / 2.0)
        return 1.0
    rings = spine_loft(radii, 0.14, 1.1, n=16, dent=dent)
    rings[-1] = [(rings[-1][0][0], rings[-1][0][1], rings[-1][0][2])] * 16
    s = loft(rings, cap0=True, cap1=False)
    return fitf(weld(s), f_cone(0.045, 0.14))

part('SmallWitchHats__cone', f_cone(0.045, 0.14), hat_cone, smooth=60)


def hat_band():
    # ribbon hat band: a tapered sleeve hugging the crown, soft edges
    return fitf(ring_lathe([(0.0433, -0.005), (0.0445, -0.0048), (0.045, -0.0038), (0.0428, 0.0038),
                            (0.0422, 0.0048), (0.041, 0.005), (0.0412, -0.005)], seg=24), f_cyl(0.045, 0.045, 0.010))

part('SmallWitchHats__band', f_cyl(0.045, 0.045, 0.010), hat_band)


def hat_buckle():
    # square buckle frame with its centre bar
    w, h, t, b = 0.014, 0.010, 0.001, 0.0022
    s = box(w, b, t).move(y=h / 2 - b / 2) + box(w, b, t).move(y=-h / 2 + b / 2)
    s = s + box(b, h - 2 * b, t).move(x=w / 2 - b / 2) + box(b, h - 2 * b, t).move(x=-w / 2 + b / 2)
    return s + box(0.0012, h - 2 * b, t * 0.8)

part('SmallWitchHats__buckle', f_box(0.014, 0.010, 0.001), hat_buckle)


# ================================================= small skeleton hands ==
def bone_lathe(L, r_a, r_s, r_b, seg=6):
    """A long bone along Y: base knob r_a at -Y, shaft r_s, head r_b at +Y."""
    return lathe([(0.0, -L / 2), (r_a, -L / 2 + L * 0.1), (r_s, -L * 0.2),
                  (r_s, L * 0.22), (r_b, L / 2 - L * 0.12), (0.0, L / 2)], seg=seg)


def hand_palm():
    # four metacarpals fanning slightly toward the knuckles (+Z) on a
    # knobbly row of carpals across the wrist end (-Z)
    s = Shape()
    for x in (-0.0205, -0.0068, 0.0068, 0.0205):
        b = bone_lathe(0.040, 0.0038, 0.0024, 0.0048, seg=6).rot(x=math.pi / 2)
        s = s + b.rot(y=-x * 2.0).move(x=x, z=0.0015)
    carp = lathe([(0.0, -0.0275), (0.003, -0.026), (0.004, -0.018), (0.0036, 0.0), (0.004, 0.018), (0.003, 0.026),
                  (0.0, 0.0275)], seg=6)
    s = s + x_axis(carp).scale(1, 1.0, 1.0).move(y=-0.0005, z=-0.019)
    return fitf(s, f_box(0.055, 0.010, 0.045))

part('SmallSkeletonHands__palm', f_box(0.055, 0.010, 0.045), hand_palm, smooth=60)


def hand_bone():
    # a phalanx: waisted shaft with flared, rounded joint ends
    return lathe([(0.0, -0.0075), (0.0032, -0.0072), (0.004, -0.0055), (0.0025, -0.002), (0.0024, 0.002),
                  (0.0036, 0.0055), (0.003, 0.0073), (0.0, 0.0075)], seg=6)

part('SmallSkeletonHands__bone', f_cyl(0.004, 0.004, 0.015), hand_bone, smooth=60)


def hand_joint():
    # knuckle: two rounded condyles side by side with a groove between
    s = sphere(0.005, seg=10, rings=6)

    def w(p):
        k = 1 - 0.14 * math.exp(-(p.x / 0.0014) ** 2)
        return Vector((p.x, p.y * k, p.z * k))
    return fitf(s.warp(w), f_sphere(0.005))

part('SmallSkeletonHands__joint', f_sphere(0.005), hand_joint, smooth=70)


def hand_wrist():
    # a row of carpal pebbles of mixed sizes
    s = Shape()
    for x, r, sy in ((-0.0068, 0.0036, 1.0), (0.0, 0.004, 0.9), (0.0068, 0.0034, 1.05)):
        s = s + sphere(1.0, seg=5, rings=4).scale(r * 0.95, r * sy, r).move(x=x)
    return fitf(s, f_box(0.020, 0.008, 0.008))

part('SmallSkeletonHands__wrist', f_box(0.020, 0.008, 0.008), hand_wrist, smooth=70)


# ======================================================= small cauldrons ==
def cauldron_body():
    # upper wall of the pot: thick cast iron flaring out to the rim, with a
    # casting seam bead round the middle; open top and bottom
    return ring_lathe([(0.045, -0.0225), (0.0485, -0.012), (0.0505, -0.0035), (0.0515, -0.0015),
                       (0.0532, 0.012), (0.055, 0.0225), (0.052, 0.0225), (0.0502, 0.012),
                       (0.0482, 0.0), (0.0455, -0.012), (0.042, -0.0225)], seg=24)

part('SmallCauldrons__body', f_cyl(0.055, 0.045, 0.045), cauldron_body)

F_CAULDRON_BOTTOM = f_bounds((-0.045, -0.045, -0.045), (0.045, 0.0, 0.045))


def cauldron_bottom():
    # round-bottomed bowl of the pot: a thick hemispherical shell, open at
    # the top where the wall sits on it
    prof = [(0.045 * math.sin(math.pi / 2 * k / 6), -0.045 * math.cos(math.pi / 2 * k / 6)) for k in range(1, 7)]
    return fitf(vessel(prof, 0.003, seg=20, floor=0.003), F_CAULDRON_BOTTOM)

part('SmallCauldrons__bottom', F_CAULDRON_BOTTOM, cauldron_bottom)
part('SmallCauldrons__rim', f_torus(0.055, 0.005),
     lambda: ring_section(0.055, [(0.005, -0.0012), (0.0042, 0.003), (0.0018, 0.005), (-0.0025, 0.0048),
                                  (-0.005, 0.002), (-0.005, -0.002), (-0.0025, -0.0048), (0.0018, -0.005),
                                  (0.0042, -0.003)], seg=28))


def cauldron_leg():
    # stubby cast foot: broad where it joins the pot, a waist, a pad
    return lathe([(0.0, -0.007), (0.0036, -0.007), (0.004, -0.0062), (0.0031, -0.0038), (0.0034, 0.0),
                  (0.0048, 0.0042), (0.006, 0.007), (0.0, 0.007)], seg=8)

part('SmallCauldrons__leg', f_cyl(0.006, 0.004, 0.014), cauldron_leg)

F_CAULDRON_HANDLE = torus_arc_bounds(0.055, 0.003, math.pi)


def cauldron_handle():
    # iron bail: a wire arc, thicker grip in the middle, swollen eyes at the
    # two ends where it hooks into the lugs
    def rm(a, t):
        d = abs(t - 0.5) * 2
        k = 1.0 if d < 0.28 else (0.7 if d < 0.86 else 0.7 + 0.3 * (d - 0.86) / 0.14)
        if 0.24 <= d < 0.32:
            k = 0.85
        return (k, 0.0)
    return fitf(ring_section(0.055, circ_sec(0.003, 8), seg=26, arc=math.pi, rmod=rm), F_CAULDRON_HANDLE)

part('SmallCauldrons__handle', F_CAULDRON_HANDLE, cauldron_handle, smooth=50)


# ====================================================== small tombstones ==
def weather(s, amt, scale, seed):
    return noisy(s, amt, scale=scale, seed=seed)


def tomb_body():
    # weathered headstone slab: worn rounded edges and an incised border
    # line round the front face
    s = weather(rbox(0.12, 0.16, 0.024, r=0.003, seg=2), 0.0009, 60.0, 3)
    g, d = 0.0016, 0.003
    xb, yb = 0.054, 0.074
    cutters = [box(2 * xb + g, g, d).move(y=yb, z=0.012), box(2 * xb + g, g, d).move(y=-yb, z=0.012),
               box(g, 2 * yb + g, d).move(x=xb, z=0.012), box(g, 2 * yb + g, d).move(x=-xb, z=0.012)]
    return fitf(cut(s, *cutters), f_box(0.12, 0.16, 0.024))

part('SmallTombstones__body', f_box(0.12, 0.16, 0.024), tomb_body)

F_TOMB_TOP = f_bounds((0.0, -0.012, -0.06), (0.06, 0.012, 0.06))


def tomb_top():
    # rounded head of the stone: a half disc slab, worn edges
    pts = [(0.0, -0.06)] + [(0.06 * math.sin(math.pi * k / 12), -0.06 * math.cos(math.pi * k / 12)) for k in range(1, 12)] + [(0.0, 0.06)]
    s = prism(pts, 0.024, bevel_r=0.0028, bevel_seg=2).rot(x=math.pi / 2)
    return fitf(weather(s, 0.0008, 60.0, 5), F_TOMB_TOP)

part('SmallTombstones__top', F_TOMB_TOP, tomb_top)
part('SmallTombstones__base', f_box(0.14, 0.014, 0.040),
     lambda: fitf(weather(rbox(0.14, 0.014, 0.040, r=0.003, seg=1), 0.0008, 50.0, 7), f_box(0.14, 0.014, 0.040)))
part('SmallTombstones__vBar', f_box(0.010, 0.070, 0.003), lambda: rbox(0.010, 0.070, 0.003, r=0.0011, seg=1))
part('SmallTombstones__hBar', f_box(0.035, 0.010, 0.003), lambda: rbox(0.035, 0.010, 0.003, r=0.0011, seg=1))


# ========================================================= small spiders ==
def spider_abdomen():
    # egg-shaped abdomen: narrow waist toward the head (+X), fullest behind
    # the middle, spinnerets at the tail, a flatter belly
    s = x_axis(lathe([(0.0, -0.014), (0.0035, -0.0132), (0.0085, -0.0095), (0.0125, -0.003), (0.0132, 0.003),
                      (0.0112, 0.0085), (0.0065, 0.0125), (0.0, 0.014)], seg=12))
    s = s.warp(lambda p: Vector((p.x, p.y * (0.82 if p.y < 0 else 1.0), p.z)))
    return fitf(s, f_sphere(0.014))

part('SmallSpiders__body', f_sphere(0.014), spider_abdomen, smooth=70)


def spider_head():
    # cephalothorax: a low shield with two fangs hanging at the front (+X)
    s = x_axis(lathe([(0.0, -0.0098), (0.0056, -0.0086), (0.0088, -0.004), (0.0094, 0.001), (0.0078, 0.0062),
                      (0.0042, 0.0092), (0.0, 0.0098)], seg=10))
    s = s.warp(lambda p: Vector((p.x, p.y * (0.7 if p.y < 0 else 0.9), p.z)))
    fangs = Shape()
    for sz in (-1, 1):
        f = lathe([(0.0, -0.0036), (0.0011, 0.0), (0.0017, 0.0024), (0.0, 0.0036)], seg=5)
        fangs = fangs + f.rot(z=0.35).move(x=0.0066, y=-0.0052, z=sz * 0.0024)
    return fitf(s + fangs, f_sphere(0.0098))

part('SmallSpiders__head', f_sphere(0.0098), spider_head, smooth=70)


def spider_leg():
    # a leg segment: thin, tapering, with a joint knob at each end
    return lathe([(0.0, -0.011), (0.00085, -0.0106), (0.001, -0.0094), (0.0006, -0.002), (0.0007, 0.0092),
                  (0.0009, 0.0104), (0.0, 0.011)], seg=5)

part('SmallSpiders__leg', f_cyl(0.001, 0.001, 0.022), spider_leg, smooth=60)


# ========================================================== small brooms ==
def broom_stick(L=0.55, n=8, m=24):
    # a crooked natural branch: wandering spine, a few knots, a worn,
    # rounded free end (+Y) and a cut end in the bristles (-Y)
    knots = ((0.12, 0.0, 1.0), (0.45, 2.1, 0.8), (0.74, 4.0, 1.0))
    rings = []
    for k in range(m + 1):
        t = k / m
        y = -L / 2 + L * t
        cx = 0.0011 * math.sin(t * 7.1) + 0.0006 * math.sin(t * 17.0)
        cz = 0.0010 * math.sin(t * 5.3 + 1.0)
        r = 0.0049 - 0.0005 * t
        if k == m:
            r *= 0.55
        elif k == m - 1:
            r *= 0.92
        ring = []
        for i in range(n):
            a = TAU * i / n
            rr_ = r
            for kt, ka, kw in knots:
                d = (t - kt) * L / 0.012
                da = math.atan2(math.sin(a - ka), math.cos(a - ka))
                rr_ += 0.0012 * kw * math.exp(-d * d) * math.exp(-(da / 0.9) ** 2)
            ring.append((cx + rr_ * math.cos(a), y, cz + rr_ * math.sin(a)))
        rings.append(ring)
    return fitf(loft(rings), f_cyl(0.006, 0.006, L))

part('SmallBrooms__stick', f_cyl(0.006, 0.006, 0.55), broom_stick, smooth=60)


def broom_twig(L=0.125):
    # a birch twig: thick end in the binding (+Y), kinked as it thins to a
    # fine tip (-Y), with a couple of nodes
    pts = [(0.0, L / 2, 0.0), (0.0003, L * 0.3, 0.0002), (-0.0004, L * 0.05, 0.0004), (0.0005, -L * 0.2, -0.0003),
           (-0.0002, -L * 0.38, 0.0005), (0.0006, -L / 2, 0.0)]
    s = tube(pts, 0.0018, seg=5, smooth=1, r_end=0.0005)
    return fitf(s, f_cyl(0.002, 0.001, L))

part('SmallBrooms__twig', f_cyl(0.002, 0.001, 0.125), broom_twig, smooth=60)


def broom_twine():
    # binding: four coils of twine, running on a slight helix
    prof = [(0.0, -0.01), (0.0068, -0.01)]
    for k in range(4):
        y0 = -0.01 + 0.005 * k
        prof += [(0.0074, y0 + 0.0005), (0.008, y0 + 0.0025)]
    prof += [(0.0074, 0.0096), (0.0068, 0.01), (0.0, 0.01)]
    s = lathe(prof, seg=12)
    s = s.warp(lambda p: Vector((p.x, p.y + 0.0007 * math.sin(math.atan2(p.x, p.z)) * min(1.0, math.hypot(p.x, p.z) / 0.007), p.z)))
    return fitf(s, f_cyl(0.008, 0.008, 0.020))

part('SmallBrooms__twine', f_cyl(0.008, 0.008, 0.020), broom_twine, smooth=50)


# ====================================================== small black cats ==
def cat_body():
    # sitting cat seen as one mass (the game stretches it 1.5x up): deep
    # chest in front (+Z), a narrower neck on top, haunches swelling at the
    # lower sides and back
    s = sphere(0.03, seg=14, rings=10)

    def w(p):
        x, y, z = p.x, p.y, p.z
        u = y / 0.03
        k = 1.0 - 0.22 * max(0.0, u - 0.35) / 0.65
        x *= k
        z *= k
        if z > 0:
            z *= 1.0 + 0.12 * max(0.0, 1 - abs(u + 0.1) / 0.8)
        h = max(0.0, -u + 0.1) * (abs(x) / 0.03)
        x *= 1.0 + 0.16 * h
        if z < 0 and u < 0.2:
            z *= 1.0 + 0.1 * (0.2 - u)
        return Vector((x, y, z))
    return fitf(s.warp(w), f_sphere(0.03))

part('SmallBlackCats__body', f_sphere(0.030), cat_body, smooth=180)


def cat_head():
    # cat head: wide cheeks, a flat face, a small muzzle below the eyes
    # (+Z), a flatter crown
    s = sphere(0.024, seg=14, rings=9)

    def w(p):
        x, y, z = p.x * 1.1, p.y * (0.9 if p.y > 0 else 1.0), p.z * 0.86
        m = math.exp(-(x / 0.009) ** 2 - ((y + 0.006) / 0.006) ** 2)
        if z > 0:
            z += 0.004 * m
        return Vector((x, y, z))
    return fitf(s.warp(w), f_sphere(0.024))

part('SmallBlackCats__head', f_sphere(0.024), cat_head, smooth=180)


def cat_ear():
    # cupped ear: a half cone shell open to the front (+Z), thin rim
    rings = []
    for k in range(4):
        t = k / 3.0
        y = -0.007 + 0.0132 * t
        ro = 0.008 * (1 - t) + 0.0004
        ri = max(0.0002, ro - 0.0014)
        outer = [(ro * math.sin(a), y, -ro * math.cos(a) * 1.0) for a in [math.pi * (i / 4 - 0.5) for i in range(5)]]
        inner = [(ri * math.sin(a), y, -ri * math.cos(a) + 0.0006) for a in [math.pi * (0.5 - i / 4) for i in range(5)]]
        rings.append(outer + inner)
    return fitf(loft(rings), f_cone(0.008, 0.014))

part('SmallBlackCats__ear', f_cone(0.008, 0.014), cat_ear, smooth=50)

F_CAT_TAIL = torus_arc_bounds(0.024, 0.006, math.pi * 1.3)


def cat_tail():
    # curling tail: full at the root (end of the arc), thinning to a
    # rounded tip at the start of the arc
    return fitf(ring_section(0.024, circ_sec(0.006, 6), seg=18, arc=math.pi * 1.3,
                             rmod=lambda a, t: (0.5 + 0.5 * t ** 0.6, 0.0)), F_CAT_TAIL)

part('SmallBlackCats__tail', F_CAT_TAIL, cat_tail, smooth=180)


def cat_legs():
    # what shows under a sitting cat: a soft drum of haunches and two
    # front paws side by side
    drum = lathe([(0.0, -0.01), (0.0215, -0.01), (0.023, -0.0075), (0.0215, 0.004), (0.019, 0.01), (0.0, 0.01)], seg=12)
    drum = drum.warp(lambda p: Vector((p.x * (1 + 0.06 * (1 if p.z < 0 else 0)), p.y, p.z)))
    paws = Shape()
    for sx in (-1, 1):
        paws = paws + sphere(1.0, seg=8, rings=4).scale(0.0058, 0.0048, 0.0075).move(x=sx * 0.0072, y=-0.0052, z=0.0165)
    return fitf(drum + paws, f_cyl(0.020, 0.024, 0.020))

part('SmallBlackCats__legs', f_cyl(0.020, 0.024, 0.020), cat_legs, smooth=60)


# ===================================================== small treat pails ==
part('SmallTreatPails__body', f_sphere(0.070), lambda: pumpkin_body(0.07, n=8, dimple=0.0), smooth=60)


def pail_rim():
    # moulded collar round the top: a rolled lip, a shallow well inside it
    # and a boss in the middle that carries the stem
    return lathe([(0.0, -0.0075), (0.044, -0.0075), (0.0455, -0.0045), (0.046, 0.0035), (0.048, 0.0058),
                  (0.0472, 0.0075), (0.0448, 0.0072), (0.0432, 0.0056), (0.013, 0.0056), (0.0112, 0.0072),
                  (0.0, 0.0075)], seg=24)

part('SmallTreatPails__rim', f_cyl(0.048, 0.048, 0.015), pail_rim)

F_PAIL_HANDLE = torus_arc_bounds(0.055, 0.004, math.pi)


def pail_handle():
    # moulded plastic bail: a flat strap with a thicker grip in the middle
    # and round pivot knobs at the ends
    sec = [(0.0024, -0.0028), (0.0024, 0.0028), (0.0014, 0.004), (-0.0014, 0.004), (-0.0024, 0.0028),
           (-0.0024, -0.0028), (-0.0014, -0.004), (0.0014, -0.004)]

    def rm(a, t):
        d = abs(t - 0.5) * 2
        k = 1.0 + 0.25 * max(0.0, 1 - d / 0.3) + 0.45 * max(0.0, (d - 0.9) / 0.1)
        return (k, 0.0, 0.0, 1.0)
    return fitf(ring_section(0.055, sec, seg=24, arc=math.pi, rmod=rm), F_PAIL_HANDLE)

part('SmallTreatPails__handle', F_PAIL_HANDLE, pail_handle)


# =========================================================== small skulls ==
def skull_cranium(R=0.045, seg=20):
    # skull (the game stretches it 10 % front to back, face to +Z), built
    # round the face axis so the face gets the detail: eye sockets and a
    # nasal hole carved in (the dark discs the placer puts there now show
    # at their bottoms), cheekbones, the lower face drawn in behind the jaw
    # and teeth, the braincase fuller at the back
    th = [150, 118, 94, 76, 61, 49, 39, 30, 22.5, 16, 10, 4.5]
    prof = [(0.0, -R)] + [(R * math.sin(math.radians(t)), R * math.cos(math.radians(t))) for t in th] + [(0.0, R)]
    s = lathe(prof, seg=seg).rot(x=math.pi / 2)        # +Y pole -> +Z (face)

    def g(x, y, cx, cy, sx, sy):
        return math.exp(-((x - cx) / sx) ** 2 - ((y - cy) / sy) ** 2)

    def w(p):
        x, y, z = p.x, p.y, p.z
        if z > 0:
            for sx_ in (-1, 1):
                d = math.hypot(x - sx_ * 0.0145, (y - 0.0065) * 1.1)
                if d < 0.0115:
                    z = min(z, 0.0342 + 0.0045 * (d / 0.0115) ** 2)
            e = (x / 0.0062) ** 2 + ((y + 0.0145) / 0.0085) ** 2
            if e < 1.0:
                z = min(z, 0.0384 + 0.0025 * e)
            z += 0.0014 * g(x, y, 0.0, 0.0185, 0.024, 0.0035)            # brow
        if y < -0.012:
            u = min(1.0, (-0.012 - y) / 0.02)
            if z > 0:
                z *= 1 - 0.36 * u
            x *= 1 - 0.22 * u
        cheek = g(abs(x), y, 0.031, -0.011, 0.009, 0.006) * (1 if z > 0.01 else 0)
        x += math.copysign(0.0025 * cheek, x)
        if z < 0 and y > -0.01:
            z *= 1.04
        return Vector((x, y, z))
    return fitf(s.warp(w), f_sphere(R))

part('SmallSkulls__cranium', f_sphere(0.045), skull_cranium, smooth=60)


def skull_jaw():
    # mandible: a horseshoe of bone round the chin (+Z) running back to
    # the angles, a chin knob low at the front
    pts = [(-0.0225, 0.0, -0.018), (-0.021, 0.0, 0.0), (-0.0145, 0.0, 0.0145), (0.0, 0.0, 0.0215),
           (0.0145, 0.0, 0.0145), (0.021, 0.0, 0.0), (0.0225, 0.0, -0.018)]
    u = tube(pts, 0.0045, seg=6, smooth=1).scale(1, 1.45, 1)
    chin = sphere(1.0, seg=8, rings=4).scale(0.008, 0.0045, 0.0045).move(y=-0.0022, z=0.0245)
    return fitf(u + chin, f_box(0.055, 0.014, 0.040))

part('SmallSkulls__jaw', f_box(0.055, 0.014, 0.040), skull_jaw, smooth=60)
part('SmallSkulls__tooth', f_box(0.005, 0.006, 0.002),
     lambda: fitf(taper(rbox(0.005, 0.006, 0.002, r=0.0008, seg=1), top=0.8, along='y', axes='x'), f_box(0.005, 0.006, 0.002)))


# ========================================================== small potions ==
def potion_cork():
    # cork stopper: tapering into the neck, the top edge rounded, pitted
    s = lathe([(0.0, -0.005), (0.0056, -0.005), (0.006, -0.0044), (0.0068, 0.0032), (0.007, 0.0042),
               (0.0064, 0.005), (0.0, 0.005)], seg=10)
    return fitf(noisy(s, 0.00025, scale=700.0, seed=2), f_cyl(0.007, 0.007, 0.010))

part('SmallPotions__cork', f_cyl(0.007, 0.007, 0.010), potion_cork, smooth=50)


# ============================================================ small owls ==
def owl_body():
    # owl body (the game stretches it 1.4x up): fuller at the bottom, a
    # soft breast in front, folded wings standing proud on the sides and a
    # short tail at the back bottom
    s = sphere(0.04, seg=14, rings=10)

    def w(p):
        x, y, z = p.x, p.y, p.z
        u = y / 0.04
        k = 1.0 + 0.1 * max(0.0, -u) - 0.14 * max(0.0, u - 0.3)
        x *= k
        z *= k
        side = abs(x) / 0.04
        if side > 0.55 and u < 0.6:
            x *= 1.0 + 0.07 * min(1.0, (side - 0.55) / 0.3) * min(1.0, (0.6 - u) / 0.4)
        if z < 0 and u < -0.45:
            z *= 1.0 + 0.2 * (-0.45 - u) / 0.55
        return Vector((x, y, z))
    return fitf(s.warp(w), f_sphere(0.04))

part('SmallOwls__body', f_sphere(0.040), owl_body, smooth=70)


def owl_head(R=0.033, seg=16):
    # owl head, built round the face axis (+Z): wide cheeks, a flat facial
    # disc with two eye bowls pressed in (the eye rings sit in them), a
    # ridge down the middle to the beak, a flatter crown
    th = [160, 128, 95, 65, 49, 35, 23, 10.5]          # rings dense over the face
    prof = [(0.0, -R)] + [(R * math.sin(math.radians(t)), R * math.cos(math.radians(t))) for t in th] + [(0.0, R)]
    s = lathe(prof, seg=seg).rot(x=math.pi / 2)        # +Y pole -> +Z (face)

    def w(p):
        x, y, z = p.x * 1.08, p.y * (0.92 if p.y > 0 else 1.0), p.z
        if z > 0:
            for sx_ in (-1, 1):
                d = math.hypot(x - sx_ * 0.0145, y - 0.005)
                if d < 0.013:
                    lim = 0.0262 + 0.004 * (d / 0.013) ** 2
                    z = min(z, lim)
        return Vector((x, y, z))
    return fitf(s.warp(w), f_sphere(R))

part('SmallOwls__head', f_sphere(0.033), owl_head, smooth=70)


def owl_tuft():
    # ear tuft: three stiff feathers fanning up from one root
    s = Shape()
    for ang, h in ((-0.35, 0.010), (0.0, 0.012), (0.35, 0.0105)):
        f = lathe([(0.0, 0.0), (0.0024, 0.0), (0.0016, h * 0.5), (0.0, h)], seg=5)
        s = s + f.rot(z=ang)
    return fitf(s, f_cone(0.006, 0.012))

part('SmallOwls__tuft', f_cone(0.006, 0.012), owl_tuft, smooth=50)


def owl_perch():
    # log-slice perch: flat sawn top and bottom, rough bark round the side
    s = lathe([(0.0, -0.004), (0.0272, -0.004), (0.028, -0.0028), (0.0255, 0.0028), (0.0238, 0.004),
               (0.0215, 0.0037), (0.0, 0.0037)], seg=20)
    rnd = [1.0, 0.96, 0.99, 0.94, 1.0, 0.97, 0.93, 0.99, 0.95, 1.0, 0.96, 0.92, 0.99, 0.97, 0.94, 1.0, 0.95, 0.98, 0.93, 0.99]

    def w(p):
        r = math.hypot(p.x, p.z)
        if r > 0.022:
            i = int(round(math.atan2(p.x, p.z) / (TAU / 20))) % 20
            p.x *= rnd[i]
            p.z *= rnd[i]
        return p
    return fitf(s.warp(w), f_cyl(0.024, 0.028, 0.008))

part('SmallOwls__perch', f_cyl(0.024, 0.028, 0.008), owl_perch)


# ================================================== brass candlesticks ==
def stick_base():
    # candlestick foot: a beaded edge, a cove and a domed top rising to
    # the column
    return lathe([(0.0, -0.005), (0.0445, -0.005), (0.045, -0.0038), (0.0445, -0.0027), (0.041, -0.0021),
                  (0.0395, 0.0004), (0.037, 0.0026), (0.0352, 0.0038), (0.02, 0.0046), (0.0, 0.005)], seg=24)

part('BrassCandlesticks__base', f_cyl(0.035, 0.045, 0.010), stick_base)


def stick_column():
    # turned column: a spreading foot, a slim waisted shaft, rings and a
    # flared capital under the drip pan
    return lathe([(0.0, -0.03), (0.010, -0.03), (0.010, -0.0275), (0.0068, -0.025), (0.0072, -0.023),
                  (0.0056, -0.02), (0.005, 0.016), (0.0064, 0.019), (0.0056, 0.021), (0.008, 0.027),
                  (0.008, 0.03), (0.0, 0.03)], seg=12)

part('BrassCandlesticks__column', f_cyl(0.008, 0.010, 0.060), stick_column)


def stick_knop():
    # knop: two domes meeting at a sharp belt (the game squashes it flat)
    return lathe([(0.0, -0.014), (0.005, -0.0132), (0.0105, -0.0085), (0.0135, -0.0025), (0.014, 0.0),
                  (0.0135, 0.0025), (0.0105, 0.0085), (0.005, 0.0132), (0.0, 0.014)], seg=14)

part('BrassCandlesticks__knop', f_sphere(0.014), stick_knop)


def stick_cup():
    # drip pan: a dish with a rolled rim and a socket boss the candle
    # stands in
    return lathe([(0.0, -0.003), (0.019, -0.003), (0.0225, -0.0008), (0.024, 0.0025), (0.0237, 0.003),
                  (0.0225, 0.0028), (0.021, 0.0012), (0.0125, 0.0006), (0.0095, 0.001), (0.0092, 0.003),
                  (0.0, 0.003)], seg=14)

part('BrassCandlesticks__cup', f_cyl(0.024, 0.020, 0.006), stick_cup)


def wax_drip(h=0.019):
    # a run of wax: a flat blob where it leaves the rim (-Y; the game turns
    # it over), thinning, then a round bead where it stopped (+Y)
    return fitf(lathe([(0.0, -h / 2), (0.0026, -h / 2 + 0.0008), (0.003, -h / 2 + 0.0025), (0.0016, -h * 0.1),
                       (0.0014, h * 0.22), (0.0022, h / 2 - 0.0028), (0.0019, h / 2 - 0.0008), (0.0, h / 2)], seg=6),
                f_cone(0.003, h))

part('BrassCandlesticks__drip', f_cone(0.003, 0.019), wax_drip, smooth=70)
