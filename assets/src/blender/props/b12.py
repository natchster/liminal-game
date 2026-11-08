"""Batch b12: domestic and bathroom props (hat racks, lamps, hall tables,
paintings, pet bowls, leashes, backpacks, kettles, suitcases, sewing
machines, bowls, ashtrays, hooks, brooms, the R1 wall switch, hand dryers,
towel dispensers, hand mirrors, lipsticks).

Every part is modelled in its primitive's frame at the canonical size
written in the placer (README.md). Local helpers sit at the top.
"""
from propkit import *
import propkit


# ------------------------------------------------------------ local kit --
def _fw(s):
    return propkit._fix_winding(s)


def ring_section(R, sec, seg=32, arc=TAU, a0=0.0):
    """Sweep a closed 2D section [(dr, dz), ...] around the Z axis in the XY
    plane (a torus_geo! with any cross-section). arc < TAU caps the ends."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    m = len(sec)
    v, f = [], []
    for i in range(n):
        a = a0 + arc * i / seg
        ca, sa = math.cos(a), math.sin(a)
        for dr, dz in sec:
            v.append(((R + dr) * ca, (R + dr) * sa, dz))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    if not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(seg * m + j for j in range(m)))
    return _fw(Shape(v, f))


def revolve(sec, seg=24):
    """Revolve a closed (r, y) section around +Y (washers, collars, rims)."""
    return ring_section(0.0, sec, seg).rot(x=-math.pi / 2)


def circ_sec(r, n=8, sx=1.0, sy=1.0, a0=0.0):
    return [(r * sx * math.cos(a0 + TAU * i / n), r * sy * math.sin(a0 + TAU * i / n)) for i in range(n)]


def arc_bounds(R, r, arc, kind='other'):
    """Bounds of torus_geo!(R, r, .., .., arc)."""
    xs, ys = [], []
    for i in range(721):
        u = arc * i / 720
        for j in range(24):
            w = TAU * j / 24
            xs.append((R + r * math.cos(w)) * math.cos(u))
            ys.append((R + r * math.cos(w)) * math.sin(u))
    return f_bounds((min(xs), min(ys), -r), (max(xs), max(ys), r), kind)


def loft(rings, cap0=True, cap1=True):
    """Skin a list of closed rings (same point count) into a solid."""
    n = len(rings[0])
    v, f = [], []
    for r in rings:
        v += [tuple(p) for p in r]
    for i in range(len(rings) - 1):
        for j in range(n):
            j2 = (j + 1) % n
            f.append((i * n + j, i * n + j2, (i + 1) * n + j2, (i + 1) * n + j))
    if cap0:
        f.append(tuple(reversed(range(n))))
    if cap1:
        f.append(tuple((len(rings) - 1) * n + j for j in range(n)))
    return _fw(Shape(v, f))


def rect_ring(hx, hy, z, r=0.0, seg=2, axis='z'):
    """Closed rounded-rectangle ring (hx, hy half sizes) at depth z along
    `axis` ('z': ring in XY; 'y': ring in XZ with hy along Z)."""
    pts = rounded_rect(2 * hx, 2 * hy, r, seg) if r > 0 else [(-hx, -hy), (hx, -hy), (hx, hy), (-hx, hy)]
    if axis == 'z':
        return [(x, y, z) for x, y in pts]
    return [(x, z, y) for x, y in pts]


def xz_prism(outline, h, bevel_r=0.0, seg=1, angle=50):
    """Outline [(x, z), ...] extruded along Y (centred)."""
    s = prism([(x, -z) for x, z in outline], h).rot(x=-math.pi / 2)
    return bevel(s, bevel_r, seg=seg, angle=angle) if bevel_r > 0 else s


def yz_prism(outline, length, bevel_r=0.0, seg=1, angle=50):
    """Outline [(z, y), ...] extruded along X (centred)."""
    s = prism([(-z, y) for z, y in outline], length).rot(y=math.pi / 2)
    return bevel(s, bevel_r, seg=seg, angle=angle) if bevel_r > 0 else s


def arc_pts(cx, cy, r, a0, a1, n):
    return [(cx + r * math.cos(a0 + (a1 - a0) * i / n), cy + r * math.sin(a0 + (a1 - a0) * i / n)) for i in range(n + 1)]


# -------------------------------------------------------------- hat rack --
def hat_rail():
    """Wall rail: flat back, rounded top front edge, a bead groove across the
    face, chamfered underside, eased ends and two wooden screw plugs."""
    sec = [(-0.015, -0.010), (0.011, -0.010), (0.015, -0.006), (0.015, -0.0016), (0.0139, -0.0008),
           (0.0139, 0.0008), (0.015, 0.0016)]
    sec += arc_pts(0.010, 0.005, 0.005, 0.0, math.pi / 2, 3)
    sec += [(-0.015, 0.010)]
    s = yz_prism(sec, 0.50, bevel_r=0.0018, seg=1, angle=60)
    for sx in (-1, 1):
        s += lathe([(0.0028, -0.0006), (0.0024, 0.0003), (0.0012, 0.0008), (0, 0.0009)], seg=8,
                   cap_bottom=False).rot(x=math.pi / 2).move(x=sx * 0.232, y=0.0045, z=0.0144)
    return s


def hat_peg():
    """Turned brass peg: rosette against the rail, waisted shank, swollen tip."""
    return lathe([(0, -0.014), (0.004, -0.014), (0.004, -0.0125), (0.0031, -0.0113), (0.0025, -0.009),
                  (0.0024, 0.006), (0.003, 0.0105), (0.0029, 0.0128), (0.0019, 0.014), (0, 0.014)], seg=10)


def ball_knob(r, neck=0.58, seg=10):
    """Ball knob with a collar, axis along Z, collar toward -Z."""
    return lathe([(0, -r), (r * neck, -r), (r * neck * 1.05, -r * 0.78), (r * 0.9, -r * 0.42), (r, 0.05 * r),
                  (r * 0.87, r * 0.5), (r * 0.52, r * 0.86), (0, r)], seg=seg).rot(x=math.pi / 2)


part('HatRacks__rail', f_box(0.50, 0.020, 0.030), hat_rail)
part('HatRacks__peg', f_cyl(0.004, 0.004, 0.028), hat_peg)
part('HatRacks__knob', f_sphere(0.006), lambda: ball_knob(0.006, seg=8))


def fedora_brim():
    """Felt brim with a bound edge, saddled: sides turned up, front and back
    snapped down (the crown hides the middle)."""
    t = 0.00125
    sec = [(0.042, -t), (0.0705, -t), (0.0738, -t * 1.25), (0.075, 0.0), (0.0738, t * 1.25), (0.0705, t), (0.042, t)]
    s = revolve(sec, seg=32)
    def curl(p):
        r = math.hypot(p.x, p.z)
        a = math.atan2(p.z, p.x)
        k = max(0.0, (r - 0.042) / 0.033)
        return Vector((p.x, p.y + 0.00275 * k * math.cos(2 * a), p.z))
    return s.warp(curl)


def fedora_crown():
    """Hollow felt crown: tapered sides, a teardrop crease down the top along
    Z, pinches at the front; open underneath."""
    prof = [(0, 0.0255), (0.028, 0.0248), (0.0425, 0.017), (0.0452, 0.0), (0.0466, -0.0275), (0.048, -0.0275),
            (0.0471, -0.012), (0.0460, 0.008), (0.0450, 0.019), (0.0428, 0.0248), (0.036, 0.0272), (0.018, 0.0275),
            (0, 0.0275)]
    s = lathe(prof, seg=24)
    def shape(p):
        x, y, z = p.x, p.y, p.z
        h = max(0.0, (y - 0.004) / 0.0235)
        crease = 0.0085 * math.exp(-(x / (0.010 + 0.004 * (1 - z / 0.048))) ** 2) * h ** 1.5
        crease *= 0.75 + 0.25 * (-z / 0.048)
        pinch = 1 - 0.16 * max(0.0, z / 0.048) * h ** 1.2
        return Vector((x * pinch, y - crease, z))
    return s.warp(shape)


def fedora_band():
    """Grosgrain band hugging the crown with a flat bow on the left side."""
    sec = [(-0.001, -0.0048), (0.0004, -0.005), (0.001, -0.0042), (0.001, 0.0042), (0.0004, 0.005), (-0.001, 0.0048)]
    s = ring_section(0.0487, sec, seg=30)
    knot = rbox(0.0034, 0.0055, 0.0085, r=0.0012, seg=1).move(x=-0.0503)
    loops = sum((ellipsoid(0.0016, 0.0045, 0.0036, seg=6).move(x=-0.0502, y=sy * 0.0052, z=0.0006) for sy in (-1, 1)), Shape())
    return s + knot + loops


def cap_dome():
    """Six-panel cap crown: hollow half shell, sunken seams from the rim to
    the button, a stiffened front panel."""
    n = 6
    outer = [(0.05 * math.cos(a), 0.05 * math.sin(a)) for a in (math.pi / 2 * i / n for i in range(n + 1))]
    inner = [(0.0485 * math.cos(a), 0.0485 * math.sin(a)) for a in (math.pi / 2 * i / 4 for i in range(4, -1, -1))]
    prof = [(max(r, 0), y) for r, y in inner] + outer[:-1] + [(0, 0.05)]
    s = lathe(prof, seg=24)
    def seams(p):
        a = math.atan2(p.z, p.x)
        d = abs(((a / (TAU / 6)) + 0.5) % 1 - 0.5) * (TAU / 6)
        r = math.hypot(p.x, p.z)
        k = 1 - 0.035 * math.exp(-(d / 0.05) ** 2) * min(1.0, r / 0.012)
        front = 1 + 0.03 * max(0.0, math.cos(a)) * (p.y / 0.05)
        return Vector((p.x * k * front, p.y, p.z * k))
    return s.warp(seams)


def cap_visor():
    """Curved visor: crescent outline (the inner edge follows the head),
    sides bent down, a rounded front edge."""
    t = 0.0024
    rings = []
    zs = [0.053 * math.sin(math.pi / 2 * (-1 + 2 * i / 16)) for i in range(17)]
    for z in zs:
        xo = math.sqrt(max(0.0, 0.055 ** 2 - z * z))
        xi = 0.013 * (1 - (z / 0.055) ** 2)
        drop = 0.0034 * (z / 0.055) ** 2
        xs = [xi, xi + (xo - xi) * 0.5, xo - 0.0012, xo]
        ring = [(xs[0], -t / 2), (xs[1], -t / 2), (xs[2], -t / 2), (xs[3], 0.0), (xs[2], t / 2), (xs[1], t / 2), (xs[0], t / 2)]
        rings.append([(x, y + 0.0017 - drop, z) for x, y in ring])
    return loft(rings)


part('HatRacks__brim', f_cyl(0.075, 0.075, 0.008), fedora_brim)
part('HatRacks__crown', f_cyl(0.045, 0.048, 0.055), fedora_crown, smooth=60)
part('HatRacks__dent', f_box(0.010, 0.005, 0.045),
     lambda: ellipsoid(0.005, 0.0025, 0.0225, seg=10).warp(lambda p: Vector((p.x * (1 - 0.35 * p.z / 0.0225), p.y, p.z))),
     smooth=180)
part('HatRacks__band', f_torus(0.047, 0.005), fedora_band)
part('HatRacks__capDome', f_bounds((-0.05, 0, -0.05), (0.05, 0.05, 0.05)), cap_dome, smooth=70)
part('HatRacks__capBrim', f_bounds((0, -0.003, -0.055), (0.055, 0.003, 0.055)), cap_visor, smooth=50)
part('HatRacks__button', f_sphere(0.005),
     lambda: lathe([(0, -0.005), (0.0042, -0.005), (0.005, -0.0032), (0.0049, -0.0005), (0.004, 0.0028),
                    (0.0022, 0.0045), (0, 0.005)], seg=10), smooth=180)


# ------------------------------------------------------- tall floor lamp --
def floor_lamp_base():
    """Weighted base: rounded foot edge, an ogee rising to a domed boss."""
    prof = [(0, -0.007), (0.106, -0.007), (0.1098, -0.0058), (0.1096, -0.0032), (0.1045, -0.0012),
            (0.088, 0.0022), (0.062, 0.0046), (0.030, 0.0058), (0.019, 0.0062), (0.0165, 0.007), (0, 0.007)]
    return lathe(prof, seg=30)


def floor_lamp_stand():
    """Three tube sections joined by coupling collars, a hex-ish foot nut at
    the bottom and a switch collar under the harp."""
    y0, y1 = -0.65, 0.65
    rr = lambda y: 0.0103 - 0.0011 * (y - y0) / (y1 - y0)
    prof = [(0, y0), (0.0112, y0), (0.012, y0 + 0.003), (0.012, y0 + 0.020), (0.0112, y0 + 0.023), (rr(y0 + 0.026), y0 + 0.026)]
    for yc in (-0.217, 0.217):
        prof += [(rr(yc - 0.012), yc - 0.012), (0.0114, yc - 0.009), (0.0116, yc - 0.004), (0.0109, yc),
                 (0.0116, yc + 0.004), (0.0114, yc + 0.009), (rr(yc + 0.012), yc + 0.012)]
    prof += [(rr(0.60), 0.60), (0.0112, 0.604), (0.0115, 0.628), (0.0106, 0.634), (0.0080, 0.638),
             (0.0072, 0.646), (0.0060, y1), (0, y1)]
    return lathe(prof, seg=14)


def floor_lamp_band():
    """Decorative collar clamped on the stand: flat inside, rounded outside."""
    sec = [(-0.003, -0.0022), (0.0012, -0.003), (0.0028, -0.0017), (0.003, 0.0), (0.0028, 0.0017),
           (0.0012, 0.003), (-0.003, 0.0022)]
    return ring_section(0.014, sec, seg=20)


def floor_lamp_harp():
    """Harp saddle ring with the two lugs the harp arms clip into."""
    ring = ring_section(0.0180, circ_sec(0.0016, 6), seg=18)
    lugs = sum((box(0.0036, 0.0028, 0.0034).move(x=sx * 0.0182) for sx in (-1, 1)), Shape())
    return ring + lugs


part('TallFloorLamps__base', f_cyl(0.10, 0.11, 0.014), floor_lamp_base)
part('TallFloorLamps__stand', f_cyl(0.010, 0.012, 1.30), floor_lamp_stand)
part('TallFloorLamps__band', f_torus(0.014, 0.003), floor_lamp_band)
part('TallFloorLamps__harp', f_torus(0.018, 0.002), floor_lamp_harp)


# ------------------------------------------------------- hall side table --
def table_top():
    """Tabletop board: rounded corners in plan, a cove moulding run round
    the underside edge (the trim slab covers the top face)."""
    W, D = 0.50, 0.24
    levels = [(0.010, 0.0), (0.0055, 0.0003), (0.0015, 0.0012), (-0.0025, 0.0028), (-0.0065, 0.0048), (-0.010, 0.0056)]
    rings = [rect_ring(W / 2 - ins, D / 2 - ins, y, r=0.008 - ins * 0.6, seg=2, axis='y') for y, ins in reversed(levels)]
    return loft(rings)


def table_trim():
    """Top slab with a bullnose edge all round and rounded plan corners."""
    W, D = 0.508, 0.248
    levels = [(-0.003, 0.0016), (-0.0022, 0.0005), (-0.0008, 0.0), (0.0008, 0.0), (0.0022, 0.0005), (0.003, 0.0016)]
    return loft([rect_ring(W / 2 - ins, D / 2 - ins, y, r=0.010 - ins, seg=2, axis='y') for y, ins in levels])


def table_leg():
    """Turned leg: square pommel under the top, a collar, a long taper and a
    small foot."""
    H = 0.71
    y1, y0 = H / 2, -H / 2
    block = rbox(0.0238, 0.085, 0.0238, r=0.0025, seg=1).move(y=y1 - 0.0425)
    prof = [(0, y0), (0.0082, y0), (0.0096, y0 + 0.004), (0.0098, y0 + 0.012), (0.0082, y0 + 0.018),
            (0.0080, y0 + 0.024), (0.0090, y0 + 0.30), (0.0108, y1 - 0.14), (0.0112, y1 - 0.108),
            (0.0136, y1 - 0.100), (0.0136, y1 - 0.094), (0.0112, y1 - 0.090), (0, y1 - 0.086)]
    return block + lathe(prof, seg=12)


def table_drawer():
    """Drawer front: eased edges, flat border, a raised field with bevels."""
    W, H = 0.48, 0.06
    zs = [(-0.0015, 0.0), (0.0, 0.0), (0.0006, 0.0009), (0.0006, 0.0075), (0.0015, 0.0125)]
    return loft([rect_ring(W / 2 - ins, H / 2 - ins, z) for z, ins in zs])


def mushroom_knob(r=0.008, seg=8):
    """Drawer knob: rosette, neck, mushroom head; axis along +Z."""
    return lathe([(0, -r), (r * 0.66, -r), (r * 0.62, -r * 0.8), (r * 0.32, -r * 0.5),
                  (r * 0.74, -r * 0.12), (r, r * 0.25), (r * 0.72, r * 0.8), (0, r)],
                 seg=seg).rot(x=math.pi / 2)


def key_tray():
    """Shallow coin/key dish: foot ring, low sloped wall, rolled rim."""
    prof = [(0, -0.004), (0.0425, -0.004), (0.045, -0.0032), (0.0472, -0.0005), (0.0492, 0.0025), (0.0500, 0.0036),
            (0.0492, 0.0040), (0.0478, 0.0034), (0.0455, 0.0015), (0.042, 0.0010), (0, 0.0010)]
    return lathe(prof, seg=24)


def key_blade():
    """Flat key blade with cut bitting along one edge and a shouldered tip."""
    out = [(-0.010, -0.0015), (0.0085, -0.0015), (0.010, -0.0004), (0.010, 0.0006), (0.0085, 0.0015)]
    x = 0.0085
    for k in range(4):
        out += [(x - 0.0012, 0.0015), (x - 0.0020, 0.0004), (x - 0.0028, 0.0015)]
        x -= 0.0033
    out += [(-0.0055, 0.0015), (-0.0060, 0.0011), (-0.010, 0.0011)]
    return xz_prism(out, 0.001)


def key_bow():
    """Key head: a round bow with an off-centre ring hole, eased edges."""
    t = 0.0005
    s = revolve([(0.0017, -t), (0.006, -t), (0.006, t), (0.0017, t)], seg=12)
    return s.warp(lambda p: Vector((p.x - 0.0028 * (math.hypot(p.x, p.z) < 0.003), p.y, p.z)))


part('HallSideTables__top', f_box(0.50, 0.020, 0.24), table_top)
part('HallSideTables__trim', f_box(0.508, 0.006, 0.248), table_trim)
part('HallSideTables__leg', f_cyl(0.010, 0.014, 0.71), table_leg)
part('HallSideTables__drawer', f_box(0.48, 0.06, 0.003), table_drawer)
part('HallSideTables__knob', f_sphere(0.008), mushroom_knob)
part('HallSideTables__tray', f_cyl(0.05, 0.045, 0.008), key_tray)
part('HallSideTables__keyBlade', f_box(0.020, 0.001, 0.003), key_blade)
part('HallSideTables__keyBow', f_cyl(0.006, 0.006, 0.001), key_bow)


def envelope():
    """Envelope, address side up: a paper sheet whose corners lift a little,
    the back flap showing as a second layer underneath, a soft fold edge."""
    W, D, t = 0.10, 0.06, 0.0008
    sheet = slab(W, D, t, 8, 4).rot(x=-math.pi / 2)
    flap = xz_prism([(-W / 2 + 0.001, -D / 2 + 0.001), (W / 2 - 0.001, -D / 2 + 0.001), (W / 2 - 0.001, -D / 2 + 0.006),
                     (0.006, 0.008), (-0.006, 0.008), (-W / 2 + 0.001, -D / 2 + 0.006)], 0.0003).move(y=-t / 2 - 0.00014)
    s = sheet + flap
    def lift(p):
        u, v = p.x / (W / 2), p.z / (D / 2)
        return Vector((p.x, p.y + 0.0005 * u * u + 0.00035 * v * v + 0.00018 * u * v, p.z))
    return s.warp(lift)


def postage_stamp():
    """Stamp with perforated (scalloped) edges and a chamfered face."""
    w, d = 0.012, 0.014
    pts = []
    def edge(a, b, n):
        for i in range(n):
            t0 = i / n
            ax, az = a[0] + (b[0] - a[0]) * t0, a[1] + (b[1] - a[1]) * t0
            pts.append((ax, az))
            tm = (i + 0.5) / n
            mx, mz = a[0] + (b[0] - a[0]) * tm, a[1] + (b[1] - a[1]) * tm
            nx, nz = -(b[1] - a[1]), (b[0] - a[0])
            L = math.hypot(nx, nz)
            pts.append((mx - nx / L * 0.0005, mz - nz / L * 0.0005))
    c = [(-w / 2, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (-w / 2, d / 2)]
    for i in range(4):
        a, b = c[i], c[(i + 1) % 4]
        edge(a, b, 3)
    return xz_prism(pts, 0.002)


def small_vase():
    """Trumpet vase: foot ring, slim waist, flaring neck to a rolled lip,
    hollow inside the mouth."""
    prof = [(0.0235, -0.050), (0.0240, -0.047), (0.0225, -0.044), (0.0200, -0.040), (0.0195, -0.030),
            (0.0215, -0.018), (0.0210, -0.004), (0.0178, 0.012), (0.0178, 0.024), (0.0225, 0.040),
            (0.0285, 0.048), (0.0300, 0.050)]
    return vessel(prof, 0.0022, seg=14, floor=0.04)


part('HallSideTables__envelope', f_box(0.10, 0.002, 0.06), envelope)
part('HallSideTables__stamp', f_box(0.012, 0.002, 0.014), postage_stamp)
part('HallSideTables__vase', f_cyl(0.030, 0.024, 0.10), small_vase)


# ------------------------------------------------------- picture frames --
def picture_frame(W, H, D, border, profile):
    """Moulded picture frame as a box-frame part: flat back, the moulding
    [(inset, z), ...] from the outer edge inward, a flat centre under the
    picture plane. Corners mitre naturally."""
    rings = [rect_ring(W / 2, H / 2, -D / 2)]
    for ins, z in profile:
        rings.append(rect_ring(W / 2 - ins, H / 2 - ins, z))
    return loft(rings)


part('AbstractPaintings__frame', f_box(0.28, 0.28, 0.012),
     lambda: picture_frame(0.28, 0.28, 0.012, 0.01,
                           [(0.0, 0.0005), (0.0007, 0.0022), (0.0025, 0.0035), (0.0038, 0.0039), (0.0045, 0.0046),
                            (0.0062, 0.0052), (0.0082, 0.0059), (0.0095, 0.0060)]))
part('ConstellationEmbroidery__frame', f_box(0.22, 0.26, 0.014),
     lambda: picture_frame(0.22, 0.26, 0.014, 0.01,
                           [(0.0, 0.0012), (0.0006, 0.0034), (0.0018, 0.0052), (0.0034, 0.0063), (0.0048, 0.0060),
                            (0.0056, 0.0050), (0.0066, 0.0056), (0.0078, 0.0068), (0.0090, 0.0070), (0.0096, 0.0070)]))


# ------------------------------------------------------------ pet bowls --
def pet_bowl():
    """Moulded pet bowl, open wall part: flared outer skirt, a thick wall
    that turns over under the rim bead and runs down inside to a floor
    (the placer's bottom disc sits hidden under it)."""
    h = 0.0225
    prof = [(0.0765, -h), (0.0790, -h + 0.004), (0.0822, -0.008), (0.0858, 0.008), (0.0885, 0.018), (0.0895, h - 0.0015),
            (0.0875, h), (0.0835, h - 0.001), (0.0790, 0.012), (0.0720, -0.004), (0.0640, -0.0135), (0.0560, -0.0165),
            (0, -0.0165)]
    return lathe(prof, seg=24, cap_bottom=False)


def rim_bead(R, r, seg=32, flat_in=0.75, n=7):
    """Rolled rim bead: round outside, a flatter inside."""
    sec = [(r * math.cos(a) * (1.0 if math.cos(a) > 0 else flat_in), r * math.sin(a)) for a in (TAU * i / n for i in range(n))]
    return ring_section(R, sec, seg)


def kibble():
    """Dry dog food nugget: a lumpy rounded disc with a dimple."""
    s = lathe([(0, -0.0086), (0.0068, -0.0078), (0.0097, -0.0032), (0.0098, 0.0014), (0.0080, 0.0066),
               (0.0035, 0.0080), (0, 0.0062)], seg=7)
    return noisy(s.rot(x=0.35, z=0.2), 0.0011, scale=140, seed=5)


def feeding_mat():
    """Rubber feeding mat: rounded corners, a raised spill lip all round,
    a flat floor inside."""
    W, D = 0.35, 0.16
    lv = [(-0.0025, 0.0), (0.0015, 0.0), (0.0025, 0.0022), (0.0025, 0.0055), (0.0004, 0.0085), (0.0004, 0.0086)]
    rings = [rect_ring(W / 2 - ins, D / 2 - ins, y, r=0.022 - ins, seg=2, axis='y') for y, ins in lv]
    return loft(rings)


def name_plate():
    """Brass name tag: rounded ends, eased edges, two pin heads."""
    s = xz_prism(rounded_rect(0.04, 0.010, 0.0045, seg=3), 0.0006, bevel_r=0.00015, seg=1).move(y=-0.0002)
    for sx in (-1, 1):
        s += lathe([(0.0011, 0.0), (0.0008, 0.00032), (0, 0.0004)], seg=6, cap_bottom=False).move(x=sx * 0.0165, y=0.0001)
    return s


part('PetBowls__bowl', f_cyl(0.09, 0.0765, 0.045), pet_bowl, smooth=40)
part('PetBowls__rim', f_torus(0.09, 0.004), lambda: rim_bead(0.09, 0.004, seg=36))
part('PetBowls__kibble', f_sphere(0.010), kibble, smooth=180)
part('PetBowls__mat', f_box(0.35, 0.005, 0.16), feeding_mat)
part('PetBowls__label', f_box(0.04, 0.001, 0.010), name_plate)


# ------------------------------------------------------------ dog leash ---
def rope_ring(R, r, seg=30, lobes=3, twist=5, sec_n=9):
    """Braided rope bent into a ring: a lobed section that twists along the
    ring so the strands spiral round it."""
    m = sec_n
    v, f = [], []
    for i in range(seg):
        a = TAU * i / seg
        ca, sa = math.cos(a), math.sin(a)
        for j in range(m):
            b = TAU * j / m
            rr = r * (0.82 + 0.18 * math.cos(lobes * (b - twist * a)))
            v.append(((R + rr * math.cos(b)) * ca, (R + rr * math.cos(b)) * sa, rr * math.sin(b)))
    for i in range(seg):
        i2 = (i + 1) % seg
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i2 * m + j, i2 * m + j2, i * m + j2))
    return _fw(Shape(v, f))


def leash_handle():
    """Rope handle loop with the stitched splice sleeve where it doubles back."""
    loop = rope_ring(0.030, 0.0056, seg=18, twist=4, sec_n=6)
    sleeve = ring_section(0.030, circ_sec(0.0062, 6), seg=4, arc=0.55, a0=-math.pi / 2 - 0.275)
    return loop + sleeve


def d_ring(R, r, n=6, arc_n=8):
    """Metal D-ring: round wire bent into a D, the straight side at -Y."""
    c = 0.3 * R
    path = [(-R + c, -R), (R - c, -R)]
    path += [(R - c + c * math.cos(-math.pi / 4), -R + c + c * math.sin(-math.pi / 4))]
    path += [(R, -R + c), (R, 0.0)]
    path += [(R * math.cos(a), R * math.sin(a)) for a in (math.pi * k / arc_n for k in range(1, arc_n))]
    path += [(-R, 0.0), (-R, -R + c)]
    path += [(-R + c + c * math.cos(-3 * math.pi / 4), -R + c + c * math.sin(-3 * math.pi / 4))]
    path += [(-R + c, -R)]
    return tube([(x, y, 0) for x, y in path], r, seg=n)


def snap_hook():
    """Leash snap hook lying flat: cast hook body curling round at the nose,
    a thin spring gate closing it."""
    body = tube([(-0.0058, 0, 0.0), (-0.003, 0, -0.0010), (0.0, 0, -0.0016), (0.0028, 0, -0.0017), (0.0048, 0, -0.0012),
                 (0.0058, 0, 0.0002), (0.0049, 0, 0.0015), (0.0027, 0, 0.0017)], 0.0012, seg=5).scale(1, 1.25, 1)
    gate = tube([(-0.0036, 0, 0.0004), (0.0024, 0, 0.0015)], 0.0006, seg=4)
    return body + gate


def webbing_ring(R, r, t=0.0034, seg=26):
    """Harness strap lying in a loop on its edge: flat nylon webbing with a
    stitch groove along each edge, width = the torus tube diameter."""
    h = 2 * r
    sec = [(-t / 2, -h / 2), (t / 2 - 0.0007, -h / 2), (t / 2, -h / 2 + 0.0007), (t / 2 - 0.0004, -h / 2 + 0.0027),
           (t / 2, -h / 2 + 0.0038), (t / 2, h / 2 - 0.0038), (t / 2 - 0.0004, h / 2 - 0.0027), (t / 2, h / 2 - 0.0007),
           (t / 2 - 0.0007, h / 2), (-t / 2, h / 2)]
    return ring_section(R + r - t / 2, sec, seg)


def harness_buckle():
    """Side-release buckle: the female housing and the male tongue pushed
    into it, chamfered edges."""
    fem = rbox(0.0086, 0.008, 0.010, r=0.0014, seg=1).move(x=-0.0027)
    male = rbox(0.0058, 0.0068, 0.0088, r=0.0014, seg=1).move(x=0.0041)
    return fem + male


def dog_tag():
    """Round dog tag with the ring hole near its edge."""
    t = 0.001
    s = revolve([(0.0015, -t), (0.010, -t), (0.010, t), (0.0015, t)], seg=12)
    return s.warp(lambda p: Vector((p.x + 0.0068 * (math.hypot(p.x, p.z) < 0.003), p.y, p.z)))


part('DogLeashes__coil', f_torus(0.05, 0.006), lambda: rope_ring(0.05, 0.006, seg=30, twist=7))
part('DogLeashes__handle', f_torus(0.030, 0.006), leash_handle)
part('DogLeashes__clip', f_torus(0.010, 0.002), lambda: d_ring(0.010, 0.002))
part('DogLeashes__snap', f_box(0.014, 0.003, 0.006), snap_hook)
part('DogLeashes__chest', f_torus(0.06, 0.008), lambda: webbing_ring(0.06, 0.008))
part('DogLeashes__neck', f_torus(0.038, 0.006), lambda: webbing_ring(0.038, 0.006, t=0.003, seg=22))
part('DogLeashes__buckle', f_box(0.014, 0.008, 0.010), harness_buckle)
part('DogLeashes__dRing', f_torus(0.008, 0.0015), lambda: d_ring(0.008, 0.0015, n=4, arc_n=5))
part('DogLeashes__tag', f_cyl(0.010, 0.010, 0.002), dog_tag)


# ------------------------------------------------------ spilled backpack --
def backpack_body():
    """Soft pack body: rounded-rectangle sections lofted up the height, the
    sides and front bulging, a soft shoulder round the top, a sagging lid."""
    lv = [(-0.090, 0.212, 0.112, 0.020), (-0.086, 0.230, 0.128, 0.028), (-0.074, 0.238, 0.136, 0.034),
          (-0.040, 0.240, 0.138, 0.036), (0.010, 0.240, 0.138, 0.036), (0.050, 0.236, 0.134, 0.034),
          (0.074, 0.230, 0.124, 0.032), (0.084, 0.218, 0.108, 0.028), (0.0885, 0.200, 0.088, 0.022), (0.090, 0.180, 0.066, 0.016)]
    rings = [rect_ring(w / 2, d / 2, y, r=r, seg=3, axis='y') for y, w, d, r in lv]
    s = loft(rings)
    def soft(p):
        u = p.x / 0.12
        front = 0.0035 * max(0.0, 1 - u * u) * (1 if p.z > 0 else 0.4) * max(0.0, 1 - abs(p.y + 0.01) / 0.1)
        z = p.z + math.copysign(front, p.z) - 0.0018
        y = p.y - 0.0022 * max(0.0, 1 - u * u) * max(0.0, (p.y - 0.07) / 0.02)
        return Vector((p.x * (1 + 0.012 * math.cos(p.y * 30)), y, z))
    return s.warp(soft)


def zipper():
    """Zip track: two tapes and a row of interlocking teeth along the top."""
    L = 0.204
    tape = rbox(L, 0.0014, 0.006, r=0.0005, seg=1).move(y=-0.0008)
    n = 34
    teeth = Shape()
    for i in range(n):
        x = -L / 2 + L * (i + 0.5) / n
        teeth += box(L / n * 0.62, 0.0016, 0.0026).move(x=x, y=0.0007, z=(0.0004 if i % 2 else -0.0004))
    return tape + teeth


def zip_pull():
    """Zip pull tab: a flat tab with a hole, hanging from its slider."""
    tab = prism(rounded_rect(0.0052, 0.0060, 0.0018, seg=2), 0.0012).move(y=-0.0008)
    slider = box(0.006, 0.0022, 0.003).move(y=0.0029)
    return cut(tab, cyl(0.0011, 0.0011, 0.004, seg=6, bevel_r=0).rot(x=math.pi / 2).move(y=-0.0018)) + slider


def backpack_pocket():
    """Front pocket: flat back on the pack, a soft domed front with rounded
    corners and a piped edge."""
    W, H = 0.168, 0.09
    zs = [(-0.010, 0.0), (-0.004, 0.0), (-0.0025, -0.0008), (0.0015, 0.0022), (0.0065, 0.0075), (0.0092, 0.016),
          (0.010, 0.030)]
    return loft([rect_ring(W / 2 - ins, H / 2 - ins, z, r=0.02 - ins * 0.4, seg=2) for z, ins in zs])


def backpack_strap():
    """Padded shoulder strap: plump padded top, narrowing into webbing at
    the bottom, a gentle S along its length."""
    rings = []
    for i in range(9):
        t = i / 8
        y = -0.099 + 0.198 * t
        pad = min(1.0, max(0.0, (t - 0.12) / 0.2))
        w = 0.021 + 0.009 * pad
        d = 0.0045 + 0.0055 * pad
        z0 = 0.0012 * math.sin(t * math.pi * 1.6)
        rings.append([(x, y, z + z0) for x, z in rounded_rect(w, d, min(d, w) * 0.45, seg=1)])
    return loft(rings)


def webbing_arch(R, r, t=0.0024, seg=14):
    """Carry handle: flat webbing bent over in an arch (width along Z)."""
    sec = [(-t / 2, -r * 0.9), (t / 2, -r * 0.95), (t / 2, r * 0.95), (-t / 2, r * 0.9)]
    return ring_section(R + r - t / 2, sec, seg, arc=math.pi)


def book_cover(W=0.13, H=0.017, D=0.095, t=0.0024):
    """Hardcover case: boards top and bottom, a rounded spine at -Z with
    hinge grooves; open at the fore-edge and the ends, where the page block
    shows."""
    hz, hy = D / 2, H / 2
    sr = hy
    zs = -hz + sr
    out = [(hz, hy), (zs + 0.006, hy), (zs + 0.0045, hy - 0.0006), (zs + 0.003, hy)]
    out += [(zs - sr * math.sin(a), sr * math.cos(a)) for a in (math.pi * k / 6 for k in range(7))]
    out += [(zs + 0.003, -hy), (zs + 0.0045, -hy + 0.0006), (zs + 0.006, -hy), (hz, -hy), (hz, -hy + t)]
    ri = hy - t
    out += [(zs + 0.001 - ri * math.sin(a), -ri * math.cos(a)) for a in (math.pi * k / 4 for k in range(5))]
    out += [(hz, hy - t)]
    return yz_prism(out, W)


def book_pages(W=0.1248, H=0.016, D=0.089):
    """Page block: square spine side, the fore-edge dished in."""
    hz, hy = D / 2, H / 2
    sec = [(-hz, -hy), (hz - 0.0012, -hy)]
    sec += [(hz - 0.0012 * (1 - (2 * k / 6 - 1) ** 2), -hy + H * k / 6) for k in range(1, 6)]
    sec += [(hz - 0.0012, hy), (-hz, hy)]
    return yz_prism(sec, W)


def pencil():
    """Hexagonal pencil: flat back end, sharpened cone, graphite point."""
    body = lathe([(0, -0.050), (0.0027, -0.050), (0.0032, -0.0493), (0.0032, 0.031), (0.002, 0.0325)], seg=6)
    tip = lathe([(0.0028, 0.027), (0.0023, 0.034), (0.0009, 0.0455), (0.0006, 0.0475), (0, 0.050)], seg=10, cap_bottom=False)
    return body + tip


def paper_ball():
    s = sphere(0.019, seg=12, rings=8)
    s = noisy(s, 0.0042, scale=70, seed=11)
    return noisy(s, 0.0015, scale=190, seed=4)


part('SpilledBackpacks__body', f_box(0.24, 0.18, 0.14), backpack_body, smooth=60)
part('SpilledBackpacks__zip', f_box(0.204, 0.003, 0.006), zipper)
part('SpilledBackpacks__pull', f_box(0.006, 0.008, 0.003), zip_pull)
part('SpilledBackpacks__pocket', f_box(0.168, 0.09, 0.020), backpack_pocket, smooth=60)
part('SpilledBackpacks__strap', f_box(0.030, 0.198, 0.010), backpack_strap, smooth=60)
part('SpilledBackpacks__handle', arc_bounds(0.024, 0.005, math.pi), lambda: webbing_arch(0.024, 0.005))
part('SpilledBackpacks__book', f_box(0.13, 0.017, 0.095), book_cover)
part('SpilledBackpacks__pages', f_box(0.1248, 0.016, 0.089), book_pages)
part('SpilledBackpacks__pencil', f_cyl(0.003, 0.003, 0.10), pencil)
part('SpilledBackpacks__paper', f_sphere(0.020), paper_ball, flat=True)


# ------------------------------------------------------------ floor kettle --
def kettle_body():
    """Cast-iron kettle body (the placer squashes it to 0.8 height): foot
    step, full belly with a casting seam, shoulder, a raised collar and a
    lid seat on top."""
    prof = [(0, -0.075), (0.036, -0.0745), (0.042, -0.069), (0.058, -0.056), (0.0705, -0.034), (0.0752, -0.006),
            (0.0735, 0.014), (0.066, 0.036), (0.052, 0.056), (0.0385, 0.0668), (0.0352, 0.0712),
            (0.0334, 0.0719), (0.0318, 0.0711), (0.0282, 0.0731), (0, 0.0750)]
    return lathe(prof, seg=20)


def kettle_foot():
    return lathe([(0, -0.004), (0.0605, -0.004), (0.06375, -0.0022), (0.0634, 0.0008), (0.0590, 0.0035),
                  (0.050, 0.004), (0, 0.004)], seg=24)


def kettle_spout():
    """Tapered spout: flared root on the body, a slight waist, a hollow mouth."""
    return lathe([(0, -0.030), (0.0180, -0.030), (0.0158, -0.024), (0.0132, -0.012), (0.0113, 0.004), (0.0104, 0.020),
                  (0.0100, 0.0285), (0.0092, 0.030), (0.0070, 0.0296), (0.0064, 0.020), (0, 0.017)], seg=12)


def kettle_bail():
    """Cast bail handle: a round bar arched over the lid, ends turned down
    into the lugs, a thicker grip in the middle."""
    R, r = 0.0525, 0.0034
    path = [(R - 0.006, 0.0014, 0), (R - 0.0012, 0.0036, 0)]
    path += [(R * math.cos(a), R * math.sin(a), 0) for a in (0.12 + (math.pi - 0.24) * k / 12 for k in range(13))]
    path += [(-R + 0.0012, 0.0036, 0), (-R + 0.006, 0.0014, 0)]
    bar = tube(path, r, seg=6)
    grip = ring_section(R, circ_sec(0.0042, 8), seg=6, arc=0.8, a0=math.pi / 2 - 0.4)
    return bar + grip


part('FloorKettles__body', f_sphere(0.075), kettle_body)
part('FloorKettles__foot', f_cyl(0.06375, 0.06375, 0.008), kettle_foot)
part('FloorKettles__spout', f_cyl(0.010, 0.018, 0.060), kettle_spout)
part('FloorKettles__spoutTip', f_cyl(0.010, 0.010, 0.006),
     lambda: revolve([(0.0068, -0.003), (0.010, -0.003), (0.010, 0.003), (0.0068, 0.003)], seg=12))
part('FloorKettles__handle', arc_bounds(0.0525, 0.004, math.pi), kettle_bail)
part('FloorKettles__lid', f_cyl(0.0315, 0.0315, 0.008),
     lambda: lathe([(0, -0.004), (0.029, -0.004), (0.0315, -0.0026), (0.0315, -0.0006), (0.0282, 0.0006),
                    (0.020, 0.0026), (0.010, 0.0037), (0, 0.004)], seg=18))
part('FloorKettles__knob', f_sphere(0.006),
     lambda: lathe([(0, -0.006), (0.0036, -0.006), (0.0024, -0.0035), (0.0026, -0.0015), (0.0058, 0.0008),
                    (0.0050, 0.0042), (0.0024, 0.0058), (0, 0.006)], seg=8))
part('FloorKettles__trivet', f_cyl(0.095, 0.095, 0.003),
     lambda: lathe([(0, -0.0015), (0.0925, -0.0015), (0.095, -0.0004), (0.0945, 0.0009), (0.0925, 0.0015),
                    (0.080, 0.0015), (0.0790, 0.0007), (0.0780, 0.0015), (0.056, 0.0015), (0.055, 0.0007),
                    (0.054, 0.0015), (0, 0.0015)], seg=24))


# --------------------------------------------------------- padlock + key --
def padlock_body():
    """Brass padlock body: well-rounded sides, eased faces, two holes on top
    where the shackle legs enter."""
    s = rbox(0.040, 0.048, 0.020, r=0.0048, seg=2)
    holes = [cyl(0.0046, 0.0046, 0.010, seg=8, bevel_r=0).move(x=sx * 0.014, y=0.024) for sx in (-1, 1)]
    return cut(s, *holes)


def shackle_leg():
    """Shackle leg: round bar with the locking notch and a rounded foot."""
    return lathe([(0, -0.010), (0.0032, -0.010), (0.004, -0.0088), (0.004, -0.0066), (0.0030, -0.0058),
                  (0.0030, -0.0040), (0.004, -0.0032), (0.004, 0.010)], seg=7, cap_top=False)


def key_shaft():
    """Barrel key shank: a collar near the bow (+Y), rounded tip (-Y)."""
    return lathe([(0, -0.040), (0.0011, -0.0398), (0.0016, -0.0385), (0.0016, 0.031), (0.0020, 0.0325),
                  (0.0020, 0.036), (0.0015, 0.0375), (0.0015, 0.040), (0, 0.040)], seg=6)


def key_bow_ring():
    """Ornate key bow: a flat ring with four swelling lobes."""
    s = ring_section(0.0115, [(-0.0026, -0.0026), (0.0026, -0.0026), (0.003, 0.0), (0.0026, 0.0026), (-0.0026, 0.0026)], seg=24)
    def lobes(p):
        a = math.atan2(p.y, p.x)
        k = 1 + 0.055 * math.cos(4 * a)
        return Vector((p.x * k, p.y * k, p.z))
    return s.warp(lobes)


def key_bit():
    """Key bit tooth: a flat ward with a stepped notch cut in."""
    out = [(-0.003, -0.004), (0.003, -0.004), (0.003, -0.0012), (0.0012, -0.0012), (0.0012, 0.0012),
           (0.003, 0.0012), (0.003, 0.0034), (0.0024, 0.004), (-0.003, 0.004)]
    return prism(out, 0.002)


part('SmallPadlockKeys__body', f_box(0.040, 0.048, 0.020), padlock_body)
part('SmallPadlockKeys__shackle', arc_bounds(0.014, 0.004, math.pi),
     lambda: ring_section(0.014, circ_sec(0.004, 8), seg=12, arc=math.pi))
part('SmallPadlockKeys__post', f_cyl(0.004, 0.004, 0.020), shackle_leg)
part('SmallPadlockKeys__shaft', f_cyl(0.002, 0.002, 0.080), key_shaft)
part('SmallPadlockKeys__bow', f_torus(0.012, 0.003), key_bow_ring)
part('SmallPadlockKeys__tooth', f_box(0.006, 0.008, 0.002), key_bit)


# --------------------------------------------------------- small suitcase --
def small_case_body():
    """Leather case: rounded plan corners, piping beads round the top and
    bottom edges, the faces a hair inside the piping."""
    lv = [(-0.050, 0.0065), (-0.0494, 0.0035), (-0.0482, 0.0012), (-0.0466, 0.0), (-0.0446, 0.0), (-0.0432, 0.0012),
          (0.0432, 0.0012), (0.0446, 0.0), (0.0466, 0.0), (0.0482, 0.0012), (0.0494, 0.0035), (0.050, 0.0065)]
    return loft([rect_ring(0.11 - ins, 0.07 - ins, y, r=0.014 - ins, seg=3, axis='y') for y, ins in lv])


def case_band():
    """Brass seam band round the middle: a hollow rounded-rectangle strip."""
    outer = xz_prism(rounded_rect(0.222, 0.142, 0.015, seg=3), 0.003)
    inner = xz_prism(rounded_rect(0.214, 0.134, 0.011, seg=3), 0.006)
    return cut(outer, inner)


part('SmallSuitcases__body', f_box(0.22, 0.10, 0.14), small_case_body)
part('SmallSuitcases__seam', f_box(0.222, 0.003, 0.142), case_band)
part('SmallSuitcases__handle', arc_bounds(0.024, 0.005, math.pi),
     lambda: ring_section(0.02525, circ_sec(0.005, 8, sx=0.75), seg=14, arc=math.pi))
part('SmallSuitcases__buckle', f_box(0.014, 0.014, 0.004),
     lambda: prism(rounded_rect(0.014, 0.014, 0.003, seg=2), 0.002).move(z=-0.001)
     + rbox(0.008, 0.009, 0.0022, r=0.0007, seg=1).move(z=0.0009))
part('SmallSuitcases__latch', f_box(0.008, 0.006, 0.002), lambda: rbox(0.008, 0.006, 0.002, r=0.0007, seg=1))
part('SmallSuitcases__foot', f_cyl(0.008, 0.008, 0.008),
     lambda: lathe([(0, -0.004), (0.008, -0.004), (0.0078, -0.0022), (0.0068, 0.0008), (0.0046, 0.0031), (0, 0.004)], seg=10))


# --------------------------------------------------------- sewing machine --
def sewing_bed():
    """Cast bed plate: rounded plan corners, a chamfered top edge and the
    slide-plate recess in front of the needle."""
    lv = [(-0.007, 0.001), (-0.006, 0.0), (0.004, 0.0), (0.0062, 0.0014), (0.007, 0.0032)]
    s = loft([rect_ring(0.10 - ins, 0.04 - ins, y, r=0.012 - ins, seg=2, axis='y') for y, ins in lv])
    return cut(s, box(0.036, 0.003, 0.030).move(x=0.076, y=0.0075))


def sewing_pillar():
    """Column: flared foot on the bed, tapering rounded body, a soft top."""
    lv = [(-0.050, 1.0, 1.0), (-0.046, 0.94, 0.95), (-0.036, 0.87, 0.9), (0.030, 0.83, 0.87), (0.044, 0.85, 0.89), (0.050, 0.86, 0.9)]
    return loft([[(x * sx, y, z * sz) for x, z in rounded_rect(0.06, 0.06, 0.013, seg=2)] for y, sx, sz in lv])


def sewing_arm():
    """Arm: rounded section, a slight taper toward the head end (+X)."""
    rings = []
    for x, k in ((-0.08, 1.0), (-0.07, 0.99), (0.05, 0.93), (0.08, 0.9)):
        rings.append([(x, y * k, z * k) for z, y in rounded_rect(0.05, 0.03, 0.011, seg=3)])
    return loft(rings)


def sewing_head():
    """Needle head: rounded face-plate block, the needle-bar bushing below."""
    return rbox(0.020, 0.0205, 0.035, r=0.0045, seg=2).move(y=0.00225) + cyl(0.0035, 0.0035, 0.006, seg=8, bevel_r=0.0006).move(y=-0.0095)


part('SmallSewingMachines__bed', f_box(0.20, 0.014, 0.08), sewing_bed)
part('SmallSewingMachines__pillar', f_box(0.06, 0.10, 0.06), sewing_pillar)
part('SmallSewingMachines__arm', f_box(0.16, 0.03, 0.05), sewing_arm)
part('SmallSewingMachines__head', f_box(0.020, 0.025, 0.035), sewing_head)
part('SmallSewingMachines__needle', f_cyl(0.001, 0.001, 0.020),
     lambda: lathe([(0, -0.010), (0.0003, -0.0088), (0.0006, -0.006), (0.0006, 0.005), (0.001, 0.0058), (0.001, 0.010), (0, 0.010)], seg=5))
part('SmallSewingMachines__wheel', f_cyl(0.030, 0.030, 0.006),
     lambda: lathe([(0, -0.003), (0.0065, -0.003), (0.0085, -0.0014), (0.0235, -0.0014), (0.0252, -0.003), (0.0296, -0.0028),
                    (0.030, 0.0), (0.0296, 0.0028), (0.0252, 0.003), (0.0235, 0.0014), (0.0085, 0.0014),
                    (0.0065, 0.003), (0, 0.003)], seg=12))


# ------------------------------------------------------------ ceramic bowl --
part('CeramicBowls__bowl', f_cyl(0.06, 0.045, 0.04),
     lambda: lathe([(0, -0.020), (0.034, -0.020), (0.037, -0.0172), (0.046, -0.012), (0.054, 0.000), (0.0588, 0.012),
                    (0.060, 0.019), (0.0584, 0.0200), (0.0563, 0.0182), (0.0515, 0.0045), (0.041, -0.0075), (0, -0.0118)], seg=24),
     smooth=45)


# -------------------------------------------------------------- table lamp --
def table_lamp_base():
    """Round lamp base: stepped foot, an ogee sweep up to the stem boss."""
    prof = [(0, -0.015), (0.097, -0.015), (0.100, -0.0128), (0.100, -0.0072), (0.0945, -0.0042), (0.082, 0.0018),
            (0.062, 0.0068), (0.036, 0.0108), (0.021, 0.0128), (0.0165, 0.015), (0, 0.015)]
    return lathe(prof, seg=26)


def table_lamp_stem():
    """Turned stem: collar at the foot, a knop a third of the way up, a
    collar and the threaded nipple at the top."""
    prof = [(0, -0.160), (0.0100, -0.160), (0.0100, -0.152), (0.0082, -0.148), (0.0072, -0.135), (0.0072, -0.065),
            (0.0088, -0.058), (0.0098, -0.052), (0.0088, -0.046), (0.0072, -0.039), (0.0068, 0.138), (0.0090, 0.142),
            (0.0092, 0.150), (0.0060, 0.152), (0.0056, 0.160), (0, 0.160)]
    return lathe(prof, seg=12)


def table_lamp_shade():
    """Fabric shade (wide at the top): thin wall, rolled wire rims top and
    bottom, a spider fitter with its socket cup inside the top."""
    sec = [(0.0770, -0.0690), (0.0800, -0.0700), (0.0813, -0.0680), (0.1088, 0.0668), (0.1102, 0.0690), (0.1076, 0.0700),
           (0.1063, 0.0672), (0.0784, -0.0666)]
    shell = revolve(sec, seg=26)
    spokes = Shape()
    for k in range(3):
        a = TAU * k / 3
        c, s_ = math.cos(a), math.sin(a)
        spokes += tube([(0.012 * c, 0.058, 0.012 * s_), (0.1045 * c, 0.064, 0.1045 * s_)], 0.0012, seg=4)
    hub = ring_section(0.011, circ_sec(0.0016, 4), seg=8).rot(x=-math.pi / 2).move(y=0.058)
    cup = lathe([(0.0090, 0.056), (0.0105, 0.030), (0.0080, 0.024), (0, 0.023)], seg=8, cap_bottom=False)
    return shell + spokes + hub + cup


part('TableLamps__base', f_cyl(0.10, 0.10, 0.03), table_lamp_base)
part('TableLamps__stem', f_cyl(0.010, 0.010, 0.32), table_lamp_stem)
part('TableLamps__shade', f_cyl(0.11, 0.08, 0.14), table_lamp_shade, smooth=50)


# ---------------------------------------------------------------- ashtray --
def ashtray_rim():
    """Rim bead with three cigarette rests pressed into its top (the top is
    local -Z: the placer turns the ring flat with rotation.x = PI/2)."""
    s = ring_section(0.056, [(0.004 * math.cos(a), 0.004 * math.sin(a)) for a in (TAU * i / 6 + TAU / 12 for i in range(6))], seg=40)
    def rests(p):
        a = math.atan2(p.y, p.x)
        d = min(abs(((a - k * TAU / 3) + math.pi) % TAU - math.pi) for k in range(3))
        if p.z < 0:
            p = Vector((p.x, p.y, p.z + 0.0032 * math.exp(-(d / 0.09) ** 2) * (-p.z / 0.004)))
        return p
    return s.warp(rests)


def cig_butt():
    """Smoked cigarette paper: the lit end (+Y) crushed flat and ragged,
    a slight kink along its length."""
    s = lathe([(0, -0.0125), (0.0046, -0.0125), (0.0047, -0.004), (0.0046, 0.004), (0.0044, 0.009), (0.0038, 0.0118),
               (0, 0.0125)], seg=7)
    def crush(p):
        t = max(0.0, (p.y - 0.002) / 0.0105)
        x = p.x * (1 + 0.14 * t) + 0.0003 * math.sin(p.y * 300)
        z = p.z * (1 - 0.5 * t)
        y = p.y - 0.0012 * t * (0.5 + 0.5 * math.sin(p.x * 900 + p.z * 700))
        return Vector((x + 0.0004 * (p.y / 0.0125) ** 2, y, z))
    return s.warp(crush)


part('Ashtrays__dish', f_cyl(0.06, 0.055, 0.02),
     lambda: lathe([(0, -0.009), (0.050, -0.009), (0.053, -0.010), (0.0556, -0.009), (0.0598, 0.0060), (0.060, 0.0092),
                    (0.0570, 0.0098), (0.050, 0.0092), (0.036, 0.0082), (0, 0.0078)], seg=24))
part('Ashtrays__rim', f_torus(0.056, 0.004), ashtray_rim)
part('Ashtrays__butt', f_cyl(0.005, 0.005, 0.025), cig_butt, smooth=60)
part('Ashtrays__filter', f_cyl(0.005, 0.005, 0.008),
     lambda: lathe([(0, -0.0034), (0.0040, -0.004), (0.0048, -0.0036), (0.005, -0.002), (0.005, 0.004), (0, 0.004)], seg=8))


# ------------------------------------------------------------ coat hooks --
def hook_plate():
    """Domed escutcheon plate with two slotted screws."""
    lv = [(-0.004, 0.0), (0.0015, 0.0), (0.003, 0.0014), (0.004, 0.0042)]
    s = loft([rect_ring(0.015 - ins, 0.020 - ins, z, r=0.006 - ins * 0.8, seg=2) for z, ins in lv])
    for sy in (-1, 1):
        s += screw_head(0.0021, 0.0008).rot(x=math.pi / 2).move(y=sy * 0.0125, z=0.0036)
    return s


part('WallCoatHooks__plate', f_box(0.03, 0.04, 0.008), hook_plate)
part('WallCoatHooks__shaft', f_cyl(0.005, 0.005, 0.05),
     lambda: lathe([(0, -0.025), (0.005, -0.025), (0.005, -0.0232), (0.0039, -0.0214), (0.0036, -0.017),
                    (0.0034, 0.025), (0, 0.025)], seg=10))
part('WallCoatHooks__tip', f_cyl(0.005, 0.005, 0.02),
     lambda: lathe([(0, -0.010), (0.0035, -0.010), (0.0035, 0.004), (0.0046, 0.0058), (0.005, 0.0076), (0.0043, 0.0094),
                    (0, 0.010)], seg=8))


# ---------------------------------------------------------------- broom ---
def broom_pole():
    """Wooden broom handle: rounded top end with a hanging hole, a tapered
    foot where it enters the head block."""
    prof = [(0, -0.65), (0.0088, -0.65), (0.0098, -0.646), (0.0110, -0.625), (0.0120, -0.605), (0.0120, 0.622),
            (0.0112, 0.638), (0.0088, 0.6465), (0.0048, 0.6496), (0, 0.65)]
    s = lathe(prof, seg=12)
    return cut(s, cyl(0.0034, 0.0034, 0.04, seg=8, bevel_r=0).rot(z=math.pi / 2).move(y=0.618))


def broom_bristles():
    """Bristle head: tufted ridges down the long faces, splaying a little
    toward a ragged, uneven tip line."""
    W, D = 0.20, 0.05
    rnd = random.Random(12)
    def outline(k, ridge):
        pts = []
        n = 16
        for i in range(n + 1):
            x = -W / 2 + W * i / n
            for j in (0, 1):
                xx = x + (W / n) * 0.5 * j
                if xx > W / 2:
                    continue
                pts.append((xx, D / 2 * k - (ridge if j else 0.0)))
        back = [(x, -z) for x, z in reversed(pts)]
        return pts + back
    rings = []
    for y, k, ridge in ((0.040, 0.84, 0.0012), (0.000, 0.92, 0.0022), (-0.040, 1.0, 0.0030)):
        ring = [(x * (0.97 + 0.03 * (k - 0.84) / 0.16), y, z) for x, z in outline(k, ridge)]
        if y < 0:
            ring = [(x, y + rnd.uniform(0.0, 0.006), z) for x, y, z in ring]
        rings.append(ring)
    return loft(list(reversed(rings)))


part('WoodenBrooms__pole', f_cyl(0.012, 0.012, 1.3), broom_pole)
part('WoodenBrooms__bristles', f_box(0.20, 0.08, 0.05), broom_bristles, flat=True)
part('WoodenBrooms__block', f_box(0.05, 0.03, 0.05),
     lambda: rbox(0.05, 0.022, 0.05, r=0.003, seg=1).move(y=-0.004)
     + lathe([(0.0175, 0.007), (0.0170, 0.0125), (0.0150, 0.015), (0, 0.015)], seg=14, cap_bottom=False))
part('WoodenBrooms__dust', f_sphere(0.065),
     lambda: noisy(noisy(sphere(0.060, seg=14, rings=8), 0.007, scale=30, seed=2), 0.0025, scale=90, seed=9), smooth=180)


# ------------------------------------------------------ R1 wall switch -----
def switch_plate():
    """Steel backplate: chamfered front, four corner screws and a raised
    guard collar round the push button."""
    lv = [(-0.005, 0.0), (0.0025, 0.0), (0.004, 0.0015), (0.005, 0.0038)]
    s = loft([rect_ring(0.06 - ins, 0.08 - ins, z, r=0.008 - ins, seg=2) for z, ins in lv])
    for sx in (-1, 1):
        for sy in (-1, 1):
            s += screw_head(0.0032, 0.0011, slot=False).rot(x=math.pi / 2).move(x=sx * 0.047, y=sy * 0.066, z=0.0046)
    s += ring_section(0.0355, [(-0.0025, -0.0005), (0.0025, -0.0005), (0.0022, 0.0018), (-0.0022, 0.0018)], seg=24).move(z=0.0052)
    return s


part('GameplayR1Trigger__plate', f_box(0.12, 0.16, 0.010), switch_plate)
part('GameplayR1Trigger__button', f_cyl(0.030, 0.030, 0.020),
     lambda: lathe([(0, -0.010), (0.022, -0.010), (0.020, -0.0065), (0.020, -0.002), (0.0288, -0.0015), (0.030, 0.0008),
                    (0.0292, 0.0045), (0.0255, 0.0075), (0.0150, 0.0094), (0, 0.010)], seg=16))
part('GameplayR1Trigger__label', f_box(0.080, 0.014, 0.003),
     lambda: rbox(0.080, 0.014, 0.003, r=0.0009, seg=1)
     + sum((lathe([(0.0014, 0.0), (0.0010, 0.0005), (0, 0.0007)], seg=6, cap_bottom=False).rot(x=math.pi / 2).move(x=sx * 0.035, z=0.0014)
            for sx in (-1, 1)), Shape()))


def troffer():
    """Ceiling light fitting seen from below: a flat flange border, a bevel
    stepping up into the recessed diffuser, a cross louvre."""
    lv = [(0.010, 0.0), (-0.008, 0.0), (-0.010, 0.002), (-0.010, 0.028), (-0.0072, 0.0315), (-0.0055, 0.0335)]
    s = loft([rect_ring(0.30 - ins, 0.30 - ins, y, axis='y') for y, ins in lv])
    s += box(0.534, 0.004, 0.006).move(y=-0.0070)
    s += box(0.006, 0.004, 0.534).move(y=-0.0071)
    return s


part('GameplayR1Trigger__panel', f_box(0.60, 0.020, 0.60), troffer)


# ------------------------------------------------------------ hand dryer --
def dryer_body():
    """Wall hand dryer: back plate against the wall, the cover stepping out
    of it with a seam, flat front with well-rounded edges, an intake grille
    of slots low on the front."""
    lv = [(-0.070, 0.0), (-0.064, 0.0), (-0.0625, 0.004), (-0.061, 0.0028), (0.040, 0.0028), (0.052, 0.0052),
          (0.061, 0.0095), (0.067, 0.016), (0.070, 0.024)]
    s = loft([rect_ring(0.16 - ins, 0.15 - ins, z, r=0.032 - ins * 0.5, seg=3) for z, ins in lv])
    slots = [rbox(0.17, 0.0045, 0.012, r=0.0015, seg=1).move(y=-0.085 - 0.011 * k, z=0.070) for k in range(4)]
    return cut(s, *slots)


def dryer_nozzle():
    """Outlet duct under the dryer: rounded, narrowing to a mouth at -Y with
    three guide vanes inside."""
    rings = []
    for y, k in ((0.020, 1.0), (-0.008, 0.99), (-0.016, 0.95), (-0.020, 0.88)):
        rings.append([(x * k, y, z * k) for x, z in rounded_rect(0.12, 0.06, 0.016, seg=2)])
    s = cut(loft(rings), rbox(0.092, 0.020, 0.034, r=0.004, seg=1).move(y=-0.020))
    for dx in (-0.024, 0.0, 0.024):
        s += box(0.0022, 0.009, 0.033).move(x=dx, y=-0.0145)
    return s


part('HandDryers__body', f_box(0.32, 0.30, 0.14), dryer_body)
part('HandDryers__nozzle', f_box(0.12, 0.04, 0.06), dryer_nozzle)
part('HandDryers__button', f_cyl(0.015, 0.015, 0.005),
     lambda: lathe([(0, -0.0025), (0.015, -0.0025), (0.015, -0.0008), (0.0135, 0.0006), (0.0118, 0.0006), (0.0112, 0.0016),
                    (0.0062, 0.0025), (0, 0.0025)], seg=16))


# ---------------------------------------------------- paper towel dispenser --
def towel_cabinet():
    """Dispenser cabinet: back plate, a cover with a seam and rounded front
    edges, a sight window, the lock at the top and the dispensing slot."""
    lv = [(-0.050, 0.0), (-0.043, 0.0), (-0.0415, 0.0028), (-0.040, 0.0012), (0.034, 0.0012), (0.043, 0.0055), (0.048, 0.010),
          (0.050, 0.016)]
    s = loft([rect_ring(0.12 - ins, 0.17 - ins, z, r=0.014 - ins * 0.4, seg=3) for z, ins in lv])
    cutters = [rbox(0.014, 0.13, 0.008, r=0.003, seg=1).move(y=0.025, z=0.050),
               cyl(0.0075, 0.0075, 0.012, seg=10, bevel_r=0).rot(x=math.pi / 2).move(y=0.135, z=0.050),
               rbox(0.186, 0.014, 0.012, r=0.003, seg=1).move(y=-0.150, z=0.050)]
    return cut(s, *cutters)


def towel_tongue():
    """Paper towel hanging out of the slot: thin, softly waved, the bottom
    edge torn along its zig-zag perforation."""
    W, H, t = 0.10, 0.06, 0.0006
    s = slab(W, H, t, 10, 4).move(z=-t / 2)
    def hang(p):
        u = (p.x + W / 2) / W
        v = (H / 2 - p.y) / H
        y = p.y
        if p.y < -H / 2 + 1e-5:
            y += 0.0035 * (1 if round(u * 10) % 2 else 0)
        z = p.z + 0.0011 * math.sin(u * 7.5 + 0.6) * v - 0.0008 * v * v
        return Vector((p.x, y, z))
    return s.warp(hang)


part('PaperTowelDispensers__body', f_box(0.24, 0.34, 0.10), towel_cabinet)
part('PaperTowelDispensers__towel', f_bounds((-0.05, -0.03, 0), (0.05, 0.03, 0), 'plane'), towel_tongue, smooth=60)


# ------------------------------------------------------------ hand mirror --
part('SmallHandMirrors__frame', f_cyl(0.055, 0.055, 0.008),
     lambda: lathe([(0, -0.004), (0.050, -0.004), (0.0538, -0.0031), (0.055, -0.0012), (0.0548, 0.0014), (0.0532, 0.0035),
                    (0.0512, 0.0040), (0.0494, 0.0034), (0.0482, 0.0026), (0, 0.0026)], seg=28))
part('SmallHandMirrors__handle', f_cyl(0.008, 0.010, 0.12),
     lambda: lathe([(0, -0.060), (0.0085, -0.060), (0.0100, -0.0572), (0.0094, -0.0525), (0.0080, -0.0490),
                    (0.0071, -0.030), (0.0066, 0.0), (0.0066, 0.034), (0.0075, 0.0445), (0.0082, 0.0490), (0.0071, 0.0525),
                    (0.0064, 0.0555), (0.0074, 0.0582), (0.0080, 0.060), (0, 0.060)], seg=12))
part('SmallHandMirrors__knob', f_sphere(0.012),
     lambda: lathe([(0, -0.012), (0.0062, -0.0112), (0.0105, -0.0070), (0.0120, -0.0015), (0.0112, 0.0032), (0.0086, 0.0066),
                    (0.0062, 0.0086), (0.0076, 0.0104), (0.0072, 0.012), (0, 0.012)], seg=12).rot(z=math.pi / 2), smooth=50)


# --------------------------------------------------------------- lipstick --
def lipstick_tip(r=0.008, h=0.010, seg=12):
    """The bullet's slanted tip: a round stick cut at a steep angle, the cut
    edge softened."""
    top = lambda x: -h / 2 + 0.001 + (h - 0.001) * (x + r) / (2 * r)
    rings = []
    for t, k in ((0.0, 1.0), (0.7, 0.99), (1.0, 0.93)):
        ring = []
        for i in range(seg):
            a = TAU * i / seg
            x, z = r * k * math.cos(a), r * k * math.sin(a)
            ring.append((x, -h / 2 + t * (top(x) + h / 2), z))
        rings.append(ring)
    return loft(rings)


part('SmallLipsticks__case', f_cyl(0.010, 0.010, 0.050),
     lambda: lathe([(0, -0.020), (0.0081, -0.020), (0.0081, -0.025), (0.0090, -0.025), (0.0092, -0.0238), (0.0092, -0.0205),
                    (0.0100, -0.0198), (0.0100, 0.0232), (0.0093, 0.0250), (0, 0.0250)], seg=14))
part('SmallLipsticks__cap', f_cyl(0.012, 0.012, 0.035),
     lambda: lathe([(0, -0.0175), (0.0110, -0.0175), (0.0120, -0.0163), (0.0120, 0.0168), (0.0115, 0.0175), (0.0104, 0.0175),
                    (0.0101, 0.0160), (0.0101, -0.0138), (0, -0.0143)], seg=14))
part('SmallLipsticks__bullet', f_cyl(0.008, 0.008, 0.020),
     lambda: lathe([(0, -0.010), (0.0080, -0.010), (0.0080, 0.0093), (0.0074, 0.010), (0, 0.010)], seg=14), smooth=60)
part('SmallLipsticks__tip', f_cone(0.008, 0.010), lipstick_tip, smooth=60)
