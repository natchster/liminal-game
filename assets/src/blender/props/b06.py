"""Batch b06 — office props.

placeCoffeeMugs        tapered stoneware mug (foot ring, hollow, rim bead, D-section handle)
placeNewspapers        folded newspaper sheet + two crumpled lifted folds
placeLetters           tri-folded letter, crumpled bits, envelope body/flap, perforated stamp
placeOffHookPhones     rotary desk phone: housing, dial, hub, finger-hole rims, cradle plungers,
                       handset cups and grip, curly-cord rings
placeChessBoards       board, chamfered squares, Staunton-style turned pieces, knight head
placeCarKeys           split ring, key bows/blades/teeth, remote fob, buttons, leather tag
placeFormalDiplomas    moulded frame + gold fillet mat
placeWideBookshelves   shelves, planked back, scalloped sides, hardcover books, vase,
                       mini photo frame, trophy
placeKitchenCalendars  nail + wire loop
placeTravelPosters / PropagandaPosters / VintageAds / FramedNewspapers / PeriodicTables /
placeFramedMaps / MissionPatches / AnatomyCharts   picture frames (one moulding each),
                       mats, brass plaque, pull-down chart dowels
placeVintageTypewriters  chassis with sloped hood, stepped keyboard, keycaps, platen, knobs...
placeEaselChalkboards  easel battens, brace, board frame, chalk tray, chalk, eraser + felt
placeSchoolDesks       desk top, lip, steel upright/foot, plywood seat/back, legs, wire rack,
                       open book, apple, pencil
placeSmallTypewriters  compact portable typewriter
"""
from propkit import *
import propkit

_fixw = propkit._fix_winding

# crease angles: low-segment turned/tubular parts need a wider smoothing angle
# than the default 35 deg, or their facets shade flat
SMOOTH = {
    'CoffeeMugs__rim': 180, 'CoffeeMugs__handle': 60, 'OffHookPhones__cord': 180,
    'OffHookPhones__prong': 60, 'OffHookPhones__hub': 50, 'OffHookPhones__fingerhole': 50,
    'OffHookPhones__earcup': 50, 'OffHookPhones__mouthcup': 50,
}
_part = part


def part(name, frame, build, **kw):
    if 'smooth' not in kw:
        if name in SMOOTH:
            kw['smooth'] = SMOOTH[name]
        elif name.startswith('ChessBoards__') and name not in ('ChessBoards__board', 'ChessBoards__square',
                                                                'ChessBoards__cren', 'ChessBoards__knighthead'):
            kw['smooth'] = 50
    if kw.get('smooth', 35) >= 50 and 'weighted' not in kw:
        kw['weighted'] = False          # face-area weighting flattens small lathes
    return _part(name, frame, build, **kw)


# ================================================================ helpers ==
def _rr_pts(w, d, r, cs):
    if r <= 0:
        return [(-w / 2, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (-w / 2, d / 2)]
    r = min(r, w / 2 - 1e-5, d / 2 - 1e-5)
    return rounded_rect(w, d, r, cs)


def rr_loft(w, d, rc, prof, cs=2, caps=True):
    """Stack of rounded rectangles in the XZ plane: prof = [(inset, y), ...]
    bottom to top. Each ring is (w - 2 inset) x (d - 2 inset) with corner
    radius rc - inset (kept > 0 so every ring has the same point count)."""
    v, f, rings = [], [], []
    for inset, y in prof:
        r = max(rc - inset, 0.0002) if rc > 0 else 0
        ids = []
        for (x, z) in _rr_pts(w - 2 * inset, d - 2 * inset, r, cs):
            v.append((x, y, z))
            ids.append(len(v) - 1)
        rings.append(ids)
    n = len(rings[0])
    for a, b in zip(rings, rings[1:]):
        for i in range(n):
            j = (i + 1) % n
            f.append((a[i], a[j], b[j], b[i]))
    if caps:
        f.append(tuple(reversed(rings[0])))
        f.append(tuple(rings[-1]))
    return _fixw(Shape(v, f))


def frame_ring(w, h, prof):
    """Picture-frame sweep: prof = [(u, z), ...] where u is the inset from the
    outer edge (XY rectangle w x h) and z the depth. The first and last loops
    are closed with flat panels (back board / picture recess)."""
    v, f, loops = [], [], []
    for u, z in prof:
        a, b = w / 2 - u, h / 2 - u
        loops.append([len(v) + k for k in range(4)])
        v += [(-a, -b, z), (a, -b, z), (a, b, z), (-a, b, z)]
    for A, B in zip(loops, loops[1:]):
        for i in range(4):
            j = (i + 1) % 4
            f.append((A[i], A[j], B[j], B[i]))
    f.append(tuple(reversed(loops[0])))
    f.append(tuple(loops[-1]))
    return _fixw(Shape(v, f))


def yz_prism(pts, w, bevel_r=0.0, bevel_seg=1):
    """Side profile [(z, y), ...] extruded along X (width w, centred)."""
    return prism(pts, w, bevel_r, bevel_seg).rot(y=-math.pi / 2)


def xz_prism(pts, h, bevel_r=0.0, bevel_seg=1):
    """Plan outline [(x, z), ...] extruded along Y (height h, centred)."""
    return prism(pts, h, bevel_r, bevel_seg).rot(x=math.pi / 2)


def flat_slab(w, d, t, nx, nz):
    """slab() laid flat: w along X, d along Z, thickness t along Y."""
    return slab(w, d, t, nx, nz).rot(x=-math.pi / 2)


def paper(w, d, t, nx, nz, lift=None, top_only=None, grain=0.0, seed=0):
    """A sheet of paper/card lying in XZ. lift(x, z) raises both faces (curl,
    folds); top_only(x, z) raises only the top face (pillowing)."""
    s = flat_slab(w, d, t, nx, nz)
    def f(p):
        if lift:
            p.y += lift(p.x, p.z)
        if top_only and p.y > 0:
            p.y += top_only(p.x, p.z)
        return p
    s = s.warp(f)
    if grain:
        s = noisy(s, grain, scale=60.0, seed=seed, axis=(0, 1, 0))
    return s


def crumple(w, d, t, bow, amount, seed, nx=7, nz=7):
    """A crumpled scrap of paper: bowed along its length, creased by noise."""
    s = flat_slab(w, d, t, nx, nz)
    def f(p):
        k = (p.z / (d / 2)) ** 2
        p.y += bow * (1 - k) + 0.25 * bow * (p.x / (w / 2)) ** 2
        return p
    s = s.warp(f)
    return noisy(s, amount, scale=38.0 / max(w, d) * 0.1, seed=seed, axis=(0, 1, 0))


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


# ============================================================ coffee mug ==
def mug_body():
    # tapered stoneware mug: recessed base inside a foot ring, straight
    # flaring wall, hollow with a floor well below the coffee disc
    h = 0.085 / 2
    prof = [(0.0, -h + 0.0018), (0.0268, -h + 0.0018), (0.0300, -h),
            (0.0330, -h), (0.0342, -h + 0.0024), (0.0366, 0.012),
            (0.0378, h - 0.004), (0.0380, h),
            (0.0350, h), (0.0346, h - 0.006), (0.0318, -h + 0.009),
            (0.0, -h + 0.0062)]
    return lathe(prof, seg=24, cap_bottom=False, cap_top=False)

part('CoffeeMugs__body', f_cyl(0.038, 0.0342, 0.085), mug_body)

# the rim bead: a smooth rolled lip (placer: TorusGeometry(bodyR, 0.003, 4, 14))
part('CoffeeMugs__rim', f_torus(0.038, 0.003), lambda: torus(0.038, 0.003, seg=28, tseg=5))


def mug_handle():
    # half ring, D-section: flatter on the inside, thicker at the roots
    s = torus(0.020, 0.005, seg=14, tseg=8, arc=math.pi)
    def f(p):
        a = math.atan2(p.y, p.x)
        c = Vector((math.cos(a), math.sin(a), 0)) * 0.020
        d = p - c
        radial = d.dot(Vector((math.cos(a), math.sin(a), 0)))
        root = 1.0 + 0.12 * (abs(math.cos(a)) ** 4)
        if radial < 0:                       # flatten the inner face
            d -= Vector((math.cos(a), math.sin(a), 0)) * radial * 0.35
        return c + d * root
    return s.warp(f)

part('CoffeeMugs__handle', f_bounds((-0.025, 0.0, -0.005), (0.025, 0.025, 0.005)), mug_handle)


# ============================================================= newspaper ==
def news_sheet():
    # an opened page: soft ridge where it was folded across, two corners lifting
    def lift(x, z):
        ridge = 0.0022 * math.exp(-(z / 0.018) ** 2)
        c1 = smoothstep(0.05, 0.11, x) * smoothstep(0.07, 0.14, z)
        c2 = smoothstep(0.06, 0.11, -x) * smoothstep(0.09, 0.14, -z)
        spine = 0.0008 * math.exp(-(x / 0.012) ** 2)
        return ridge + 0.0045 * c1 ** 1.6 + 0.0028 * c2 ** 1.6 + spine
    return paper(0.22, 0.28, 0.003, 9, 12, lift=lift, grain=0.00035, seed=3)

part('Newspapers__sheet', f_box(0.22, 0.003, 0.28), news_sheet)
part('Newspapers__fold', f_box(0.088, 0.004, 0.112),
     lambda: crumple(0.088, 0.112, 0.003, 0.0022, 0.0014, seed=5))
part('Newspapers__scrap', f_box(0.055, 0.004, 0.056),
     lambda: crumple(0.055, 0.056, 0.003, 0.0011, 0.0008, seed=9, nx=6, nz=6))


# =============================================================== letters ==
def letter_sheet():
    # a letter folded in three and opened again: ridge and valley creases
    zs = [(-0.090, 0.0026), (-0.030, 0.0), (0.030, 0.0021), (0.090, 0.0004)]
    def lift(x, z):
        for (z0, y0), (z1, y1) in zip(zs, zs[1:]):
            if z <= z1 + 1e-9:
                t = (z - z0) / (z1 - z0)
                return y0 + (y1 - y0) * t
        return zs[-1][1]
    return paper(0.14, 0.18, 0.002, 6, 12, lift=lift, grain=0.00025, seed=11)

part('Letters__sheet', f_box(0.14, 0.002, 0.18), letter_sheet)
part('Letters__fold', f_box(0.07, 0.003, 0.063),
     lambda: crumple(0.07, 0.063, 0.002, 0.0016, 0.0010, seed=13, nx=6, nz=6))
part('Letters__scrap', f_box(0.049, 0.003, 0.054),
     lambda: crumple(0.049, 0.054, 0.002, 0.0010, 0.0007, seed=17, nx=5, nz=5))


def envelope_body():
    # back of an envelope: slightly pillowed by what is inside, with the
    # bottom flap lying over the side flaps as a raised V layer
    pillow = lambda x, z: 0.0006 * max(0.0, 1 - (x / 0.05) ** 2) * max(0.0, 1 - (z / 0.03) ** 2)
    s = paper(0.10, 0.06, 0.0016, 6, 4, top_only=pillow)
    s = s.move(y=-0.0002)
    flap = xz_prism([(-0.0495, 0.0295), (0.0495, 0.0295), (0.006, -0.002), (-0.006, -0.002)], 0.0005)
    return s + flap.move(y=0.0010)

part('Letters__envelope', f_box(0.10, 0.002, 0.06), envelope_body)


def envelope_flap():
    # the opened flap: wide at the hinge (+z), tapering to a rounded point
    s = flat_slab(0.10, 0.04, 0.0014, 6, 6)
    def f(p):
        t = (p.z + 0.02) / 0.04               # 0 at the tip, 1 at the hinge
        k = 0.14 + 0.86 * t ** 0.85
        p.x *= k
        p.y += 0.0012 * (1 - t) ** 2          # tip curls up a little
        return p
    return s.warp(f).fit((-0.05, -0.001, -0.02), (0.05, 0.0012, 0.02))

part('Letters__flap', f_box(0.10, 0.002, 0.04), envelope_flap)


def stamp():
    # postage stamp with perforated edges
    w, d = 0.012, 0.015
    out = []
    def edge(a, b, n):
        for i in range(n):
            t0 = i / n
            out.append((a[0] + (b[0] - a[0]) * t0, a[1] + (b[1] - a[1]) * t0))
            tm = (i + 0.5) / n
            m = (a[0] + (b[0] - a[0]) * tm, a[1] + (b[1] - a[1]) * tm)
            nx, nz = (b[1] - a[1]), -(b[0] - a[0])
            L = math.hypot(nx, nz)
            out.append((m[0] - nx / L * 0.0006, m[1] - nz / L * 0.0006))
    c = [(-w / 2, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (-w / 2, d / 2)]
    edge(c[0], c[1], 3); edge(c[1], c[2], 3); edge(c[2], c[3], 3); edge(c[3], c[0], 3)
    return xz_prism(out, 0.003)

part('Letters__stamp', f_box(0.012, 0.003, 0.015), stamp)


# ========================================================= rotary phone ==
def phone_base():
    # desk-set housing: small plinth with a parting groove, tapered walls,
    # rounded top edge, and a raised cradle saddle along the back (+x)
    body = rr_loft(0.14, 0.10, 0.022, [
        (0.0030, -0.014), (0.0008, -0.0133), (0.0, -0.0118), (0.0, -0.0098),
        (0.0010, -0.0092), (0.0006, -0.0086), (0.0022, 0.0040), (0.0046, 0.0105),
        (0.0080, 0.0133), (0.0120, 0.0140)], cs=3)
    saddle = rr_loft(0.030, 0.100, 0.012, [
        (0.0, 0.006), (0.0, 0.0135), (0.0035, 0.0160), (0.0080, 0.0168)], cs=2)
    return body + saddle.move(x=0.050)

part('OffHookPhones__base', f_box(0.14, 0.028, 0.10), phone_base)


def phone_dial():
    # finger wheel: rounded rim, raised hole ring, recessed number card
    h = 0.003
    prof = [(0.0, -h), (0.0380, -h), (0.0380, h - 0.0012),
            (0.0370, h), (0.0336, h), (0.0328, h - 0.0008), (0.0220, h - 0.0008),
            (0.0210, h - 0.0018), (0.0, h - 0.0018)]
    return lathe(prof, seg=18)

part('OffHookPhones__dial', f_cyl(0.038, 0.038, 0.006), phone_dial)
part('OffHookPhones__hub', f_cyl(0.006, 0.006, 0.007),
     lambda: lathe([(0.0, -0.0035), (0.0060, -0.0035), (0.0060, 0.0012), (0.0048, 0.0030),
                    (0.0020, 0.0035), (0.0, 0.0035)], seg=12))
# a finger hole's rim: a short tube with a dished well
part('OffHookPhones__fingerhole', f_cyl(0.0038, 0.0038, 0.007),
     lambda: lathe([(0.0, -0.0035), (0.0038, -0.0035), (0.0038, 0.0028), (0.0030, 0.0035),
                    (0.0024, 0.0022), (0.0, 0.0010)], seg=10))
# cradle plunger
part('OffHookPhones__prong', f_sphere(0.005),
     lambda: lathe([(0.0, -0.005), (0.0040, -0.005), (0.0048, -0.001), (0.0042, 0.0032),
                    (0.0024, 0.0048), (0.0, 0.005)], seg=10))


def handset_cup(face_up=True):
    # ear/mouth cup: flat speaker face sunk inside a rolled rim, back tapering
    # into the grip
    prof = [(0.0, -0.010), (0.0082, -0.010), (0.0105, -0.0075), (0.0132, -0.0020),
            (0.0148, 0.0040), (0.0150, 0.0078), (0.0138, 0.0100), (0.0118, 0.0092),
            (0.0112, 0.0080), (0.0, 0.0080)]
    s = lathe(prof, seg=16)
    return s if face_up else s.scale(1, -1, 1)

part('OffHookPhones__earcup', f_cyl(0.015, 0.015, 0.020), lambda: handset_cup(True))
part('OffHookPhones__mouthcup', f_cyl(0.015, 0.015, 0.020), lambda: handset_cup(False))


def handset_grip():
    # oval grip, slimmer in the middle, a little arched
    pts = [(x, 0.0012 * (1 - (x / 0.04) ** 2), 0.0) for x in (-0.040, -0.026, -0.013, 0.0, 0.013, 0.026, 0.040)]
    s = tube(pts, 0.007, seg=12)
    def f(p):
        k = 1.0 - 0.14 * math.exp(-(p.x / 0.022) ** 2)
        p.z *= k * 10.0 / 7.0
        return p
    return s.warp(f)

part('OffHookPhones__grip', f_box(0.08, 0.014, 0.020), handset_grip)
part('OffHookPhones__cord', f_torus(0.005, 0.001), lambda: torus(0.005, 0.001, seg=12, tseg=4))


# ================================================================= chess ==
def chess_board():
    # board with a rounded top edge and an inlaid groove round the playing field
    b = rr_loft(0.26, 0.26, 0.004, [(0.0015, -0.005), (0.0, -0.0038), (0.0, 0.0022),
                                     (0.0010, 0.0043), (0.0026, 0.005)], cs=2)
    g = 0.1232
    grooves = [box(2 * g + 0.0016, 0.002, 0.0012).move(y=0.005, z=sz * g) for sz in (-1, 1)] + \
              [box(0.0012, 0.002, 2 * g + 0.0016).move(y=0.005, x=sx * g) for sx in (-1, 1)]
    return cut(b, *grooves)

part('ChessBoards__board', f_box(0.26, 0.010, 0.26), chess_board)
part('ChessBoards__square', f_box(0.0294, 0.002, 0.0294),
     lambda: rr_loft(0.0294, 0.0294, 0.0, [(0.0, -0.001), (0.0, 0.0004), (0.0006, 0.001)]))

# Staunton-style turned pieces (cellSize 0.03, pieceH 0.021)
def chess_base():
    h = 0.00315 / 2
    return lathe([(0.0, -h), (0.0116, -h), (0.0120, -h + 0.0006), (0.0118, -0.0002),
                  (0.0108, 0.0004), (0.0105, h - 0.0003), (0.0094, h), (0.0068, h),
                  (0.0066, h + 0.0005), (0.0, h + 0.0005)], seg=16)

part('ChessBoards__base', f_cyl(0.0105, 0.012, 0.00315), chess_base)


def pawn_body():
    h = 0.01155 / 2
    # extends ~1 mm below its frame to meet the base (the placer leaves a gap)
    return lathe([(0.0, -h - 0.0011), (0.0072, -h - 0.0011), (0.0040, -0.0012), (0.0033, 0.0003),
                  (0.0054, 0.0010), (0.0026, 0.0020), (0.0024, h), (0.0, h)], seg=8)

part('ChessBoards__pawnbody', f_cyl(0.0054, 0.0075, 0.01155), pawn_body)
part('ChessBoards__ball', f_sphere(0.006), lambda: sphere(0.006, seg=10, rings=6))


def knight_body():
    h = 0.01155 / 2
    return lathe([(0.0, -h), (0.0075, -h), (0.0060, -0.0020), (0.0050, 0.0015),
                  (0.0054, 0.0035), (0.0048, h), (0.0, h)], seg=10)

part('ChessBoards__knightbody', f_cyl(0.0054, 0.0075, 0.01155), knight_body)


def knight_head():
    # horse head in profile (muzzle to +z, mane to -z), extruded across x
    pts = [(-0.0045, -0.003675), (0.0012, -0.003675), (0.0014, -0.0014),
           (0.0045, -0.0007), (0.0045, 0.0009), (0.0016, 0.0027),
           (-0.0006, 0.003675), (-0.0022, 0.0030), (-0.0045, 0.0004)]
    return yz_prism(pts, 0.0054, bevel_r=0.0005)

part('ChessBoards__knighthead', f_box(0.0054, 0.00735, 0.009), knight_head)


def rook_body():
    h = 0.01575 / 2
    return lathe([(0.0, -h), (0.0084, -h), (0.0054, -0.0025), (0.0050, 0.0042),
                  (0.0066, 0.0060), (0.0066, h), (0.0046, h), (0.0, h - 0.0010)], seg=8)

part('ChessBoards__rookbody', f_cyl(0.0066, 0.0084, 0.01575), rook_body)
part('ChessBoards__cren', f_box(0.003, 0.00315, 0.003),
     lambda: rbox(0.003, 0.00315, 0.003, r=0.0004, seg=1))


def bishop_body():
    h = 0.01365 / 2
    return lathe([(0.0, -h - 0.0005), (0.0075, -h - 0.0005), (0.0046, -0.0020), (0.0031, 0.0040),
                  (0.0045, 0.0055), (0.0040, h), (0.0, h)], seg=10)

part('ChessBoards__bishopbody', f_cyl(0.0045, 0.0075, 0.01365), bishop_body)


def bishop_mitre():
    h = 0.00735 / 2
    return lathe([(0.0, -h), (0.0042, -h), (0.0060, -0.0012), (0.0050, 0.0012),
                  (0.0024, 0.0026), (0.0013, h - 0.0004), (0.0, h)], seg=10)

part('ChessBoards__mitre', f_cone(0.006, 0.00735), bishop_mitre)


def tall_body(h):
    h = h / 2
    return lathe([(0.0, -h), (0.0084, -h), (0.0056, -h + 0.0035), (0.0034, h - 0.0045),
                  (0.0054, h - 0.0030), (0.0030, h), (0.0, h)], seg=10)

part('ChessBoards__queenbody', f_cyl(0.0054, 0.0084, 0.0147), lambda: tall_body(0.0147))
part('ChessBoards__kingbody', f_cyl(0.0054, 0.0084, 0.01575), lambda: tall_body(0.01575))


def queen_crown():
    # coronet: a cup that flares to a rim, finial ball in the middle
    r = 0.0066
    return lathe([(0.0, -r), (0.0034, -0.0060), (0.0058, -0.0015), (0.0066, 0.0022),
                  (0.0040, 0.0016), (0.0016, 0.0040), (0.0, r)], seg=10)

part('ChessBoards__crown', f_sphere(0.0066), queen_crown)
part('ChessBoards__spike', f_cone(0.0009, 0.0036),
     lambda: lathe([(0.0, -0.0018), (0.0009, -0.0018), (0.0005, 0.0006), (0.0008, 0.0011),
                    (0.0005, 0.0016), (0.0, 0.0018)], seg=6))


def king_cap():
    # flared crown cup, open at the top so the cross stands clear of it;
    # a thin post (hidden in the cross) keeps the frame's top
    r = 0.006
    cup = lathe([(0.0, -r), (0.0030, -0.0055), (0.0060, 0.0005), (0.0056, 0.0025),
                 (0.0035, 0.0017), (0.0, 0.0012)], seg=10)
    return cup + box(0.0006, 0.0052, 0.0006).move(y=r - 0.0026)

part('ChessBoards__kingcap', f_sphere(0.006), king_cap)


# ============================================================== car keys ==
def split_ring():
    # two flat coils of spring steel (a split key ring)
    R, n = 0.012, 30
    pts = []
    for i in range(n + 1):
        a = 2 * TAU * i / n + 0.35
        pts.append((R * math.cos(a), R * math.sin(a), -0.0006 + 0.0012 * i / n))
    s = tube(pts, 0.0006, seg=4)
    def f(p):
        rho = math.hypot(p.x, p.y) or 1e-9
        k = (R + (rho - R) * 1.8) / rho
        p.x *= k
        p.y *= k
        return p
    return s.warp(f)

part('CarKeys__ring', f_torus(0.012, 0.0012), split_ring, smooth=70)


def key_bow():
    # round key head with a hanging hole towards its end (-x)
    ring = [(0.008 * math.cos(TAU * i / 16), 0.008 * math.sin(TAU * i / 16)) for i in range(16)]
    s = xz_prism(ring, 0.0015)
    return cut(s, cyl(0.0018, 0.0018, 0.004, seg=8, bevel_r=0).move(x=-0.0042))

part('CarKeys__bow', f_cyl(0.008, 0.008, 0.0015), key_bow)


def key_blade():
    # blade with a pointed tip and a milled keyway groove along the face
    s = xz_prism([(-0.014, -0.002), (0.0125, -0.002), (0.014, -0.0005), (0.014, 0.0003),
                  (0.0122, 0.002), (-0.014, 0.002)], 0.0015)
    return cut(s, box(0.022, 0.001, 0.0007).move(x=-0.0005, y=0.00085, z=-0.0003))

part('CarKeys__blade', f_box(0.028, 0.0015, 0.004), key_blade)
part('CarKeys__tooth', f_box(0.003, 0.001, 0.003),
     lambda: xz_prism([(-0.0015, -0.0015), (0.0015, -0.0015), (0.0004, 0.0015), (-0.0004, 0.0015)], 0.001))


def key_blade_short():
    # mailbox key: short blade with its bitting cut into the edge
    return xz_prism([(-0.008, -0.0015), (0.0065, -0.0015), (0.008, -0.0004), (0.008, 0.0006),
                     (0.0068, 0.0015), (0.0055, 0.0007), (0.0042, 0.0015), (0.0030, 0.0005),
                     (0.0016, 0.0015), (0.0004, 0.0009), (-0.0008, 0.0015), (-0.008, 0.0015)], 0.0015)

part('CarKeys__blade2', f_box(0.016, 0.0015, 0.003), key_blade_short)
part('CarKeys__fob', f_box(0.024, 0.006, 0.014),
     lambda: rr_loft(0.024, 0.014, 0.005, [(0.0012, -0.003), (0.0, -0.0017), (0.0, 0.0011),
                                           (0.0013, 0.003)], cs=2))
part('CarKeys__button', f_cyl(0.0025, 0.0025, 0.001),
     lambda: lathe([(0.0, -0.0005), (0.0025, -0.0005), (0.0025, 0.0), (0.0017, 0.0004), (0.0, 0.0005)], seg=10),
     smooth=50)
part('CarKeys__smallring', f_torus(0.003, 0.0007), lambda: torus(0.003, 0.0007, seg=12, tseg=4), smooth=180)


def leather_tag():
    # leather fob: clipped corners at the ring end (-z), rounded far end
    pts = [(-0.005, -0.012), (0.005, -0.012), (0.008, -0.009), (0.008, 0.0095), (0.0062, 0.0118),
           (-0.0062, 0.0118), (-0.008, 0.0095), (-0.008, -0.009)]
    return xz_prism(pts, 0.002, bevel_r=0.0005).fit((-0.008, -0.001, -0.012), (0.008, 0.001, 0.012))

part('CarKeys__tag', f_box(0.016, 0.002, 0.024), leather_tag)
part('CarKeys__tagplate', f_box(0.006, 0.002, 0.010),
     lambda: xz_prism(rounded_rect(0.006, 0.010, 0.0012, 1), 0.002, bevel_r=0.0004))


# ====================================================== kitchen calendar ==
part('KitchenCalendars__nail', f_cyl(0.003, 0.003, 0.004),
     lambda: lathe([(0.0, -0.002), (0.0009, -0.002), (0.0009, 0.0009), (0.0030, 0.0010),
                    (0.0030, 0.0015), (0.0020, 0.0020), (0.0, 0.0020)], seg=10), smooth=50)
part('KitchenCalendars__wire', f_torus(0.005, 0.0008), lambda: torus(0.005, 0.0008, seg=12, tseg=4), smooth=180)


# ======================================================== picture frames ==
def pframe(w, h, d, border, front, recess_z):
    """Moulded picture frame, w x h, depth d (centred). `front` is the face
    profile [(u, z), ...] from the outer edge to the inner lip; the picture
    sits in the recess (recess_z) inside the lip. The back has a rabbet with a
    backing board, since these frames are often seen from behind."""
    h2 = d / 2
    prof = [(border + 0.004, -h2 + 0.0014), (0.0055, -h2 + 0.0014), (0.0055, -h2), (0.0006, -h2),
            (0.0, -h2 + 0.0006)] + front + [(border, recess_z), (border + 0.0008, recess_z)]
    return frame_ring(w, h, prof)

h6, h7 = 0.006, 0.007
# travel poster: warm wood with a rounded bead
part('TravelPosters__frame', f_box(0.20, 0.28, 0.012), lambda: pframe(0.20, 0.28, 0.012, 0.007, [
    (0.0, h6 - 0.0012), (0.0006, h6 + 0.0002), (0.0018, h6 + 0.0022), (0.0034, h6 + 0.0031),
    (0.0050, h6 + 0.0025), (0.0062, h6 + 0.0017), (0.0068, 0.0078)], 0.0060))
# propaganda poster: plain flat-faced black frame with small chamfers
part('PropagandaPosters__frame', f_box(0.20, 0.28, 0.012), lambda: pframe(0.20, 0.28, 0.012, 0.007, [
    (0.0, h6 - 0.0004), (0.0007, h6 + 0.0015), (0.0062, h6 + 0.0015), (0.0068, 0.0076)], 0.0060))
# vintage ad: ornate gilt moulding, outer bead, cove, inner bead
part('VintageAds__frame', f_box(0.20, 0.28, 0.012), lambda: pframe(0.20, 0.28, 0.012, 0.007, [
    (0.0, h6 - 0.0012), (0.0004, h6 + 0.0008), (0.0012, h6 + 0.0028), (0.0022, h6 + 0.0026),
    (0.0030, h6 + 0.0012), (0.0040, h6 + 0.0016), (0.0048, h6 + 0.0034), (0.0058, h6 + 0.0030),
    (0.0064, h6 + 0.0019), (0.0068, 0.0078)], 0.0060))
# diploma: formal moulding whose lip stands over the gold mat
part('FormalDiplomas__frame', f_box(0.22, 0.28, 0.014), lambda: pframe(0.22, 0.28, 0.014, 0.0062, [
    (0.0, h7 - 0.0012), (0.0005, h7 + 0.0006), (0.0018, h7 + 0.0024), (0.0034, h7 + 0.0030),
    (0.0048, h7 + 0.0024), (0.0058, 0.0088)], 0.0055))
part('FormalDiplomas__mat', f_box(0.208, 0.268, 0.002), lambda: frame_ring(0.208, 0.268, [
    (0.0, -0.001), (0.0, 0.0010), (0.0052, 0.0010), (0.0062, 0.0016), (0.0078, 0.0017),
    (0.0088, 0.0011), (0.0096, 0.0010)]))
# framed newspaper: slim rounded moulding over a cream bevel-cut mat
part('FramedNewspapers__frame', f_box(0.22, 0.28, 0.014), lambda: pframe(0.22, 0.28, 0.014, 0.0052, [
    (0.0, h7 - 0.0012), (0.0005, h7 + 0.0010), (0.0018, h7 + 0.0022), (0.0034, h7 + 0.0021),
    (0.0048, 0.0087)], 0.0055))
part('FramedNewspapers__mat', f_box(0.21, 0.27, 0.002), lambda: frame_ring(0.21, 0.27, [
    (0.0, -0.001), (0.0, 0.0010), (0.0086, 0.0010), (0.0101, -0.0008), (0.0106, -0.0008)]))


def brass_plaque():
    p = xz_prism(rounded_rect(0.132, 0.004, 0.0015, 2), 0.014, bevel_r=0.0007).rot(x=-math.pi / 2)
    p = p.fit((-0.066, -0.007, -0.002), (0.066, 0.007, 0.002))
    heads = [screw_head(0.0017, 0.0008).rot(x=math.pi / 2).move(x=sx * 0.0595, z=0.0019) for sx in (-1, 1)]
    return p + sum(heads, Shape())

part('FramedNewspapers__plaque', f_box(0.132, 0.014, 0.004), brass_plaque)
# periodic table: flat classroom frame
part('PeriodicTables__frame', f_box(0.36, 0.26, 0.014), lambda: pframe(0.36, 0.26, 0.014, 0.008, [
    (0.0, h7 - 0.0008), (0.0008, h7 + 0.0020), (0.0068, h7 + 0.0020), (0.0078, 0.0086)], 0.0065))
# framed map: wood with a cove and a stepped inner lip
part('FramedMaps__frame', f_box(0.30, 0.22, 0.014), lambda: pframe(0.30, 0.22, 0.014, 0.008, [
    (0.0, h7 - 0.0012), (0.0006, h7 + 0.0012), (0.0016, h7 + 0.0026), (0.0028, h7 + 0.0020),
    (0.0042, h7 + 0.0012), (0.0056, h7 + 0.0022), (0.0068, h7 + 0.0022), (0.0076, 0.0087)], 0.0065))
# mission patch: shadow box, flat face and a deep bevel down to the patch
part('MissionPatches__frame', f_box(0.22, 0.22, 0.014), lambda: pframe(0.22, 0.22, 0.014, 0.009, [
    (0.0, h7 - 0.0008), (0.0006, h7 + 0.0018), (0.0040, h7 + 0.0020), (0.0086, 0.0086)], 0.0065))
# anatomy chart: plain board with a rounded edge
part('AnatomyCharts__frame', f_box(0.22, 0.32, 0.014), lambda: pframe(0.22, 0.32, 0.014, 0.010, [
    (0.0, h7 - 0.0014), (0.0008, h7 + 0.0008), (0.0024, h7 + 0.0016), (0.0092, h7 + 0.0016),
    (0.0098, 0.0086)], 0.0065))
part('AnatomyCharts__dowel', f_cyl(0.004, 0.004, 0.23),
     lambda: lathe([(0.0, -0.115), (0.0024, -0.115), (0.0040, -0.1125), (0.0040, -0.1075),
                    (0.0027, -0.1045), (0.0030, -0.1020), (0.0030, 0.1020), (0.0027, 0.1045),
                    (0.0040, 0.1075), (0.0040, 0.1125), (0.0024, 0.115), (0.0, 0.115)], seg=12), smooth=50)


# ========================================================= wide bookshelf ==
def shelf_board():
    # shelf with a rounded nosing along the front (+z)
    pts = [(-0.08, -0.007), (0.0765, -0.007), (0.0792, -0.0058), (0.0800, -0.0035), (0.0800, 0.0035),
           (0.0792, 0.0058), (0.0765, 0.007), (-0.08, 0.007)]
    return yz_prism(pts, 0.60, bevel_r=0.0012)

part('WideBookshelves__shelf', f_box(0.60, 0.014, 0.16), shelf_board)


def shelf_back():
    # tongue-and-groove planked back: V-grooves on the face towards the books
    b = rbox(0.60, 0.654, 0.008, r=0.0012, seg=1)
    grooves = [box(0.0026, 0.66, 0.0026).rot(y=math.pi / 4).move(x=-0.30 + 0.10 * i, z=0.004) for i in range(1, 6)]
    return cut(b, *grooves)

part('WideBookshelves__back', f_box(0.60, 0.654, 0.008), shelf_back)


def shelf_side():
    # side panel, front edge scalloped between the shelves, rounded top corner
    pts = [(-0.08, -0.327), (0.08, -0.327), (0.08, -0.300)]
    for k in range(1, 6):
        t = k / 6
        pts.append((0.08 - 0.022 * math.sin(math.pi * t), -0.300 + 0.280 * t))
    pts += [(0.08, -0.020), (0.08, 0.020)]
    for k in range(1, 6):
        t = k / 6
        pts.append((0.08 - 0.022 * math.sin(math.pi * t), 0.020 + 0.280 * t))
    pts += [(0.08, 0.300), (0.0785, 0.3165), (0.074, 0.3235), (0.066, 0.327), (-0.08, 0.327)]
    return yz_prism(pts, 0.010, bevel_r=0.0018)

part('WideBookshelves__side', f_box(0.010, 0.654, 0.16), shelf_side)


def hardcover(w=0.027, h=0.19, d=0.12):
    # boards overhanging a recessed page block, rounded spine on +z
    t = 0.0022
    pages = box(w - 2 * t + 0.0004, h - 0.006, d - 0.008).move(z=-0.001)
    covers = [rbox(t, h, d - 0.005, r=0.0007, seg=1).move(x=sx * (w / 2 - t / 2), z=-0.0025) for sx in (-1, 1)]
    spine = cyl(1.0, 1.0, h, seg=12, bevel_r=0).scale(w / 2, 1, 0.0075).move(z=d / 2 - 0.0075)
    return pages + sum(covers, Shape()) + spine

part('WideBookshelves__book', f_box(0.027, 0.19, 0.12), hardcover)
part('WideBookshelves__vase', f_cyl(0.030, 0.024, 0.070),
     lambda: lathe([(0.0, -0.035), (0.0215, -0.035), (0.0240, -0.032), (0.0280, -0.012), (0.0262, 0.008),
                    (0.0190, 0.024), (0.0220, 0.031), (0.0300, 0.035), (0.0272, 0.0352), (0.0178, 0.028),
                    (0.0160, 0.015), (0.0, 0.012)], seg=14, cap_bottom=False, cap_top=False), smooth=60)
part('WideBookshelves__photoframe', f_box(0.05, 0.06, 0.004), lambda: frame_ring(0.05, 0.06, [
    (0.0, -0.002), (0.0, 0.0018), (0.0012, 0.0033), (0.0030, 0.0035), (0.0046, 0.0028),
    (0.0050, 0.0010), (0.0056, 0.0010)]))
part('WideBookshelves__photo', f_box(0.04, 0.05, 0.005),
     lambda: rbox(0.04, 0.05, 0.0039, r=0.0006, seg=1).move(z=-0.00055))
part('WideBookshelves__trophybase', f_box(0.030, 0.010, 0.030),
     lambda: rr_loft(0.030, 0.030, 0.0, [(0.0, -0.005), (0.0, -0.0016), (0.0008, -0.0008), (0.0030, -0.0008),
                                         (0.0030, 0.0038), (0.0040, 0.005)]))
part('WideBookshelves__trophystem', f_cyl(0.004, 0.004, 0.020),
     lambda: lathe([(0.0, -0.01), (0.0040, -0.01), (0.0019, -0.006), (0.0038, 0.0), (0.0019, 0.004),
                    (0.0024, 0.008), (0.0040, 0.01), (0.0, 0.01)], seg=8), smooth=60)
part('WideBookshelves__trophycup', f_cyl(0.014, 0.010, 0.020),
     lambda: lathe([(0.0, -0.01), (0.0095, -0.01), (0.0108, -0.006), (0.0128, 0.004), (0.0140, 0.01),
                    (0.0128, 0.0102), (0.0117, 0.004), (0.0, -0.004)], seg=16, cap_bottom=False, cap_top=False),
     smooth=60)


# ===================================================== vintage typewriter ==
def typewriter_body():
    # side profile: tall carriage housing at the back (-z), hood sloping
    # down to the keyboard, low plinth under the stepped key deck (+z)
    pts = [(-0.11, -0.05), (0.11, -0.05), (0.11, -0.0425), (0.012, -0.0065), (0.0165, 0.020),
           (0.0120, 0.0330), (0.0020, 0.0440), (-0.010, 0.0490), (-0.018, 0.050), (-0.104, 0.050),
           (-0.1085, 0.0485), (-0.11, 0.044)]
    s = yz_prism(pts, 0.30, bevel_r=0.003, bevel_seg=1)
    spools = [lathe([(0.0, 0.049), (0.017, 0.049), (0.018, 0.0515), (0.0155, 0.054), (0.0, 0.0543)], seg=12)
              .move(x=sx * 0.105, z=-0.086) for sx in (-1, 1)]
    feet = [cyl(0.008, 0.009, 0.004, seg=8, bevel_r=0).move(x=sx * 0.13, y=-0.051, z=sz * 0.09)
            for sx in (-1, 1) for sz in (-1, 1)]
    return s + sum(spools, Shape()) + sum(feet, Shape())

part('VintageTypewriters__body', f_box(0.30, 0.10, 0.22), typewriter_body)


def typewriter_deck():
    # four stepped key rows (each key row sits on its own step), side cheeks
    pts = [(-0.055, -0.012), (0.055, -0.012), (0.055, -0.0005), (0.052, 0.0011), (0.028, 0.0011),
           (0.028, 0.00367), (0.0058, 0.00367), (0.0058, 0.0062), (-0.0164, 0.0062), (-0.0164, 0.00875),
           (-0.0345, 0.00875), (-0.040, 0.012), (-0.055, 0.012)]
    s = yz_prism(pts, 0.260, bevel_r=0.0008)
    cheek = [(-0.055, -0.012), (0.055, -0.012), (0.055, 0.0035), (0.050, 0.0055), (-0.040, 0.012), (-0.055, 0.012)]
    cheeks = [yz_prism(cheek, 0.005, bevel_r=0.0012).move(x=sx * 0.1325) for sx in (-1, 1)]
    return s + sum(cheeks, Shape())

part('VintageTypewriters__deck', f_box(0.27, 0.024, 0.11), typewriter_deck)
# round key: chrome-ringed cap with a shallow dish (shared with the small typewriter)
part('VintageTypewriters__key', f_cyl(0.007, 0.007, 0.005),
     lambda: lathe([(0.0, -0.0025), (0.0058, -0.0025), (0.0070, -0.0010), (0.0070, 0.0017),
                    (0.0062, 0.0025), (0.0, 0.0021)], seg=10), smooth=50)
part('VintageTypewriters__spacebar', f_box(0.18, 0.005, 0.018),
     lambda: yz_prism(rounded_rect(0.018, 0.005, 0.0018, 2), 0.18, bevel_r=0.0012))


def platen(r, L):
    e = L / 2
    return lathe([(0.0, -e), (r * 0.76, -e), (r * 0.80, -e + 0.003), (r * 0.92, -e + 0.004), (r, -e + 0.007),
                  (r, e - 0.007), (r * 0.92, e - 0.004), (r * 0.80, e - 0.003), (r * 0.76, e), (0.0, e)], seg=24)

part('VintageTypewriters__platen', f_cyl(0.025, 0.025, 0.21), lambda: platen(0.025, 0.21))


def fluted_knob():
    s = lathe([(0.0, -0.006), (0.011, -0.006), (0.014, -0.0038), (0.014, 0.0038), (0.011, 0.006), (0.0, 0.006)], seg=32)
    def f(p):
        rho = math.hypot(p.x, p.z)
        if rho > 0.0105:
            a = math.atan2(p.z, p.x)
            k = 1.0 - 0.07 * (0.5 + 0.5 * math.cos(16 * a))
            p.x *= k
            p.z *= k
        return p
    return s.warp(f)

part('VintageTypewriters__knob', f_cyl(0.014, 0.014, 0.012), fluted_knob, weighted=False)
part('VintageTypewriters__typebar', f_box(0.003, 0.030, 0.003),
     lambda: rbox(0.0012, 0.026, 0.0024, r=0.0003, seg=1).move(y=-0.002)
     + rbox(0.003, 0.004, 0.003, r=0.0005, seg=1).move(y=0.013))
part('VintageTypewriters__lever', f_cyl(0.002, 0.002, 0.030),
     lambda: lathe([(0.0, -0.015), (0.0020, -0.015), (0.0020, -0.012), (0.0012, -0.011), (0.0012, 0.010),
                    (0.0019, 0.0115), (0.0020, 0.013), (0.0012, 0.0147), (0.0, 0.015)], seg=8), smooth=60)


# ======================================================= small typewriter ==
part('SmallTypewriters__base', f_box(0.16, 0.02, 0.14),
     lambda: rr_loft(0.16, 0.14, 0.012, [(0.004, -0.01), (0.001, -0.0092), (0.0, -0.007), (0.0, 0.006),
                                         (0.0022, 0.0092), (0.005, 0.01)], cs=2))
# key deck as a shallow tray: a rim around a recessed floor the keys sit in
part('SmallTypewriters__deck', f_box(0.16, 0.010, 0.08), lambda: frame_ring(0.16, 0.08, [
    (0.0, -0.005), (0.0, 0.0038), (0.0012, 0.005), (0.0040, 0.005), (0.0052, 0.0020), (0.0060, 0.0020)])
    .rot(x=-math.pi / 2))
part('SmallTypewriters__roll', f_cyl(0.020, 0.020, 0.13), lambda: platen(0.020, 0.13))


# ======================================================== easel + board ==
def easel_leg():
    # planed batten, tapering upwards, chamfered, with peg holes above the board
    s = rr_loft(0.028, 0.022, 0.003, [(0.001, -0.65), (0.0, -0.6485), (0.002, 0.6485), (0.003, 0.65)], cs=1)
    holes = [cyl(0.0035, 0.0035, 0.012, seg=8, bevel_r=0).rot(x=math.pi / 2).move(y=0.40 + 0.06 * i, z=0.011)
             for i in range(4)]
    return cut(s, *holes)

part('EaselChalkboards__leg', f_cyl(0.012, 0.014, 1.30), easel_leg)
part('EaselChalkboards__brace', f_cyl(0.005, 0.005, 0.30),
     lambda: rr_loft(0.010, 0.009, 0.0015, [(0.0006, -0.15), (0.0, -0.1494), (0.0, 0.1494), (0.0006, 0.15)], cs=1))
# chalkboard frame: wooden rails standing proud of the slate face
part('EaselChalkboards__board', f_box(0.44, 0.32, 0.014), lambda: pframe(0.44, 0.32, 0.014, 0.010, [
    (0.0, h7 - 0.0012), (0.0008, h7 + 0.0012), (0.0030, h7 + 0.0028), (0.0075, h7 + 0.0026),
    (0.0094, 0.0088)], 0.0065))


def chalk_tray():
    # trough with a front lip; floor raised to where the chalk and eraser rest
    pts = [(-0.015, -0.007), (0.0130, -0.007), (0.0150, -0.0045), (0.0150, 0.0118), (0.0138, 0.0130),
           (0.0127, 0.0118), (0.0125, 0.0090), (-0.0130, 0.0090), (-0.0150, 0.0105)]
    return yz_prism(pts, 0.44, bevel_r=0.0008)

part('EaselChalkboards__tray', f_box(0.44, 0.014, 0.030), chalk_tray)
part('EaselChalkboards__chalk', f_cyl(0.005, 0.005, 0.055),
     lambda: lathe([(0.0, -0.0275), (0.0042, -0.0275), (0.0050, -0.0265), (0.0048, 0.0215), (0.0040, 0.0258),
                    (0.0020, 0.0275), (0.0, 0.0275)], seg=10), smooth=60)


def board_eraser():
    s = rr_loft(0.05, 0.02, 0.002, [(0.0, -0.005), (0.0, 0.0008), (0.0012, 0.0036), (0.0036, 0.005)], cs=2)
    grooves = [cyl(0.0016, 0.0016, 0.06, seg=8, bevel_r=0).rot(z=math.pi / 2).move(y=-0.0005, z=sz * 0.0106)
               for sz in (-1, 1)]
    return cut(s, *grooves)

part('EaselChalkboards__eraser', f_box(0.05, 0.010, 0.020), board_eraser)
part('EaselChalkboards__felt', f_box(0.048, 0.003, 0.018),
     lambda: noisy(paper(0.048, 0.018, 0.0026, 10, 4,
                         top_only=lambda x, z: 0.0003 * (1 - (x / 0.024) ** 2) * (1 - (z / 0.009) ** 2)),
                   0.00018, scale=900.0, seed=4), smooth=180)


# ============================================================ school desk ==
def desk_top():
    # rounded-corner top, eased edges, pencil groove along the far edge
    s = rr_loft(0.38, 0.28, 0.02, [(0.002, -0.01), (0.0, -0.008), (0.0, 0.0065), (0.0014, 0.0092),
                                   (0.0036, 0.01)], cs=3)
    groove = cyl(0.0042, 0.0042, 0.30, seg=10, bevel_r=0).rot(z=math.pi / 2).move(y=0.0118, z=0.112)
    return cut(s, groove)

part('SchoolDesks__top', f_box(0.38, 0.020, 0.28), desk_top)
part('SchoolDesks__lip', f_box(0.38, 0.010, 0.010),
     lambda: yz_prism([(-0.005, -0.005), (0.005, -0.005), (0.005, 0.001), (0.0035, 0.004), (0.0, 0.005),
                       (-0.0035, 0.004), (-0.005, 0.001)], 0.38, bevel_r=0.001))
part('SchoolDesks__upright', f_box(0.014, 0.72, 0.020),
     lambda: rr_loft(0.014, 0.020, 0.003, [(0.0008, -0.36), (0.0, -0.3592), (0.0, 0.3592), (0.0008, 0.36)], cs=2))
part('SchoolDesks__foot', f_box(0.014, 0.014, 0.16),
     lambda: rr_loft(0.014, 0.014, 0.003, [(0.0012, -0.08), (0.0, -0.0785), (0.0, 0.0785), (0.0012, 0.08)], cs=2)
     .rot(x=math.pi / 2))


def rounded_corners(s, hw, hd, rc):
    """Pull the corner vertices of a flat grid onto rounded corners."""
    def f(p):
        cx, cz = hw - rc, hd - rc
        if abs(p.x) > cx and abs(p.z) > cz:
            dx, dz = abs(p.x) - cx, abs(p.z) - cz
            L = math.hypot(dx, dz)
            m = max(dx, dz) / L if L > 1e-9 else 1.0
            k = rc / max(rc, 1e-9) * m          # square corner -> circle
            p.x = math.copysign(cx + dx * k, p.x)
            p.z = math.copysign(cz + dz * k, p.z)
        return p
    return s.warp(f)


def chair_seat():
    # moulded plywood: dished, with a waterfall front edge (+z, towards the desk)
    s = flat_slab(0.28, 0.24, 0.014, 8, 10)
    def f(p):
        p.y += 0.003
        p.y -= 0.003 * max(0.0, 1 - (p.x / 0.12) ** 2) * max(0.0, 1 - ((p.z + 0.01) / 0.10) ** 2)
        if p.z > 0.072:
            p.y -= 0.006 * ((p.z - 0.072) / 0.048) ** 2
        return p
    outline = xz_prism(rounded_rect(0.28, 0.24, 0.03, 4), 0.06)
    return boolean(s.warp(f), outline, 'INTERSECT')

part('SchoolDesks__seat', f_box(0.28, 0.020, 0.24), chair_seat)


def chair_back():
    # moulded plywood back, curved to the sitter (+z), rounded corners
    s = slab(0.28, 0.28, 0.008, 10, 4)
    s = s.warp(lambda p: Vector((p.x, p.y, p.z - 0.003 + 0.006 * (p.x / 0.14) ** 2)))
    outline = prism(rounded_rect(0.28, 0.28, 0.035, 4), 0.06)
    return boolean(s, outline, 'INTERSECT')

part('SchoolDesks__back', f_box(0.28, 0.28, 0.014), chair_back)
part('SchoolDesks__leg', f_cyl(0.006, 0.006, 0.44),
     lambda: lathe([(0.0, -0.22), (0.0046, -0.22), (0.0060, -0.2175), (0.0060, -0.2125), (0.0048, -0.2105),
                    (0.0048, 0.22), (0.0, 0.22)], seg=10), smooth=50)


def wire_rack_bar():
    # steel rod with the stubs of the cross wires welded on
    rod = lathe([(0.0, -0.171), (0.0018, -0.171), (0.0025, -0.1695), (0.0025, 0.1695), (0.0018, 0.171),
                 (0.0, 0.171)], seg=8).rot(z=math.pi / 2)
    stubs = [cyl(0.0012, 0.0012, 0.012, seg=6, bevel_r=0).rot(x=math.pi / 2).move(x=-0.15 + 0.06 * i)
             for i in range(6)]
    return rod + sum(stubs, Shape())

part('SchoolDesks__wire', f_box(0.342, 0.005, 0.006), wire_rack_bar, smooth=50)


def open_pages():
    # two page blocks rising from the fore-edges towards the spine, dipping
    # into the gutter
    s = flat_slab(0.24, 0.16, 0.006, 24, 2)
    prof = [(0.0, 0.0006), (0.010, 0.0030), (0.030, 0.0036), (0.100, 0.0030), (0.120, 0.0012)]
    def top(ax):
        for (x0, y0), (x1, y1) in zip(prof, prof[1:]):
            if ax <= x1 + 1e-9:
                t = (ax - x0) / (x1 - x0)
                t = t * t * (3 - 2 * t)
                return y0 + (y1 - y0) * t
        return prof[-1][1]
    def f(p):
        if p.y > 0:
            p.y = top(abs(p.x))
        return p
    return s.warp(f)

part('SchoolDesks__pages', f_box(0.24, 0.006, 0.16), open_pages, smooth=40)
part('SchoolDesks__cover', f_box(0.246, 0.003, 0.166),
     lambda: rr_loft(0.246, 0.166, 0.004, [(0.0, -0.0015), (0.0, 0.0008), (0.0008, 0.0015)], cs=2))
part('SchoolDesks__spine', f_cyl(0.002, 0.002, 0.16),
     lambda: lathe([(0.0, -0.08), (0.0014, -0.08), (0.0020, -0.0788), (0.0020, 0.0788), (0.0014, 0.08),
                    (0.0, 0.08)], seg=8), smooth=50)


def apple():
    # apple: shoulders, stem cavity, calyx dimple and five soft lobes at the base
    prof = [(0.0, -0.0245), (0.0055, -0.0262), (0.0120, -0.0280), (0.0205, -0.0235), (0.0265, -0.0120),
            (0.0280, 0.0010), (0.0262, 0.0140), (0.0195, 0.0245), (0.0105, 0.0280), (0.0040, 0.0268),
            (0.0, 0.0250)]
    s = lathe(prof, seg=14)
    def f(p):
        a = math.atan2(p.z, p.x)
        k = 1.0 + 0.035 * math.cos(5 * a) * max(0.0, -p.y / 0.028) + 0.012 * math.cos(2 * a + 0.7)
        p.x *= k
        p.z *= k
        return p
    return s.warp(f)

part('SchoolDesks__apple', f_sphere(0.028), apple, smooth=180, weighted=False)
part('SchoolDesks__stem', f_cyl(0.001, 0.001, 0.008),
     lambda: lathe([(0.0, -0.004), (0.0008, -0.004), (0.0006, 0.003), (0.0010, 0.004), (0.0, 0.004)], seg=6),
     smooth=60)


def leaf():
    # pointed leaf folded along its midrib
    pts = []
    for i in range(6):
        t = i / 5
        pts.append((-0.005 + 0.010 * t, 0.002 * math.sin(math.pi * t) ** 0.8))
    pts += [(x, -z) for x, z in reversed(pts[1:-1])]
    s = xz_prism(pts, 0.0004)
    return s.warp(lambda p: Vector((p.x, p.y + 0.0003 * abs(p.z) / 0.002 - 0.0001, p.z))).fit(
        (-0.005, -0.0005, -0.002), (0.005, 0.0005, 0.002))

part('SchoolDesks__leaf', f_box(0.010, 0.001, 0.004), leaf, smooth=60)


def pencil():
    # hexagonal pencil (corners on +-z like the 6-sided cylinder), eased edges
    hexa = [(0.003 * math.sin(TAU * i / 6), 0.003 * math.cos(TAU * i / 6)) for i in range(6)]
    return xz_prism(round_outline(hexa, 0.0006, seg=2), 0.10)

part('SchoolDesks__pencil', f_cyl(0.003, 0.003, 0.10), pencil)
part('SchoolDesks__tip', f_cone(0.003, 0.008),
     lambda: lathe([(0.0, -0.004), (0.0030, -0.004), (0.0019, -0.0010), (0.0009, 0.0020), (0.0005, 0.0036),
                    (0.0, 0.004)], seg=10), smooth=50)
# pink eraser: flat where it meets the pencil (+y), worn dome at the end (-y)
part('SchoolDesks__eraser', f_cyl(0.0035, 0.0035, 0.008),
     lambda: lathe([(0.0, -0.004), (0.0022, -0.0038), (0.0031, -0.0030), (0.0035, -0.0015), (0.0035, 0.004),
                    (0.0, 0.004)], seg=10), smooth=50)
