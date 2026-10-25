"""Batch b10 - "domestic" theme: kitchen things, bread, shakers, sewing,
framed pictures, photos, a diary, a wallet.

placeSpoonRests          ceramic rest (rim wall, base plate), wooden spoon (handle, bowl)
placeFramedRecipes       rustic grooved frame
placeShakerPair          salt fill, ribbed shaker cap, turned pepper mill, finial nut
placeAbstractLandscapes  gilt frame with beads and a cove
placeBreadLoaves         bread board with juice groove, bâtard loaf, slashes
... (see the section headers below for the rest)
"""
from propkit import *
import propkit

HALF = math.pi / 2


# ------------------------------------------------------------- helpers ---
def loft(rings, cap_bottom=True, cap_top=True):
    """Connect 2D outlines (same point count) stacked along Z into a closed
    solid. rings: [(outline [(x, y), ...], z), ...] from back to front."""
    v, f = [], []
    n = len(rings[0][0])
    for pts, z in rings:
        assert len(pts) == n, 'loft outlines must have the same point count'
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
    """A plate lofted from the back (first) to the front (last):
    profile = [(inset, z), ...]; each ring is the rounded outline inset."""
    return loft([(rr(w - 2 * i, h - 2 * i, r - i, seg), z) for i, z in profile])


def pframe(w, h, d, prof, r=0.0004, seg=1):
    """A picture frame moulding: a solid block w x h x d whose front is
    profiled. prof = [(inset, dz), ...] runs from the outer front edge
    inwards (dz measured from the front face z = d/2); the last ring is
    capped (the rebate floor behind the picture)."""
    rings = [(rr(w, h, r, seg), -d / 2)]
    for inset, dz in prof:
        rings.append((rr(w - 2 * inset, h - 2 * inset, max(r - inset, 2e-4), seg), d / 2 + dz))
    return loft(rings)


def lathe_rows(rows, seg):
    """Revolve rows [(r, y, rib), ...] around +Y; rib > 0 alternates the
    radius between r and r - rib from segment to segment (knurls, flutes,
    pleats). r = 0 closes the end."""
    v, f, rings = [], [], []
    for r, y, rib in rows:
        if r <= 1e-7:
            v.append((0.0, y, 0.0))
            rings.append([len(v) - 1])
            continue
        ids = []
        for i in range(seg):
            a = TAU * i / seg
            ri = r - (rib if i % 2 else 0.0)
            v.append((ri * math.sin(a), y, ri * math.cos(a)))
            ids.append(len(v) - 1)
        rings.append(ids)
    for a, b in zip(rings, rings[1:]):
        if len(a) == 1 and len(b) == 1:
            continue
        if len(a) == 1:
            for i in range(seg):
                f.append((a[0], b[i], b[(i + 1) % seg]))
        elif len(b) == 1:
            for i in range(seg):
                f.append((a[i], b[0], a[(i + 1) % seg]))
        else:
            for i in range(seg):
                j = (i + 1) % seg
                f.append((a[i], b[i], b[j], a[j]))
    if len(rings[0]) > 1:
        f.append(tuple(reversed(rings[0])))
    if len(rings[-1]) > 1:
        f.append(tuple(rings[-1]))
    return propkit._fix_winding(Shape(v, f))


def on_z(s):
    """A lathe built on +Y turned to face +Z."""
    return s.rot(x=HALF)


def along_x(s):
    """A lathe built on +Y laid along +X."""
    return s.rot(z=-HALF)


def hemi(r):
    """Frame of an upper half sphere (SphereGeometry thetaLength PI/2)."""
    return f_bounds((-r, 0, -r), (r, r, r))


# ------------------------------------------------------------ spoon rest --
def rest_dish():
    # the rest's rolled rim wall. The placer flips the upper half sphere
    # (rim at frame y = 0, pole at y = r), so the bowl is built upside down.
    R = 0.045
    prof = [(R * math.sin(a), R * (1 - math.cos(a)))
            for a in [0.25, 0.55, 0.85, 1.1, 1.3, 1.45, HALF]]
    s = vessel(prof, 0.006, seg=20, floor=0.008)
    s = s.rot(x=math.pi)
    return s.fit((-R, 0, -R), (R, R, R))

def rest_plate():
    # glazed foot plate: flat underside, rounded edge, slightly domed top
    return lathe([(0, -0.002), (0.066, -0.002), (0.0690, -0.0013), (0.0700, 0.0), (0.0690, 0.0012),
                  (0.0660, 0.0019), (0.035, 0.0020), (0, 0.0020)], seg=28)

def spoon_handle():
    # wooden spoon handle: slim at the bowl (+Y), swelling to a rounded,
    # flattened paddle end at -Y with a hanging hole
    L = 0.065
    prof = [(0.0, -L), (0.0030, -L + 0.0004), (0.0050, -L + 0.0022), (0.0059, -L + 0.0060),
            (0.0060, -0.040), (0.0053, -0.010), (0.0045, 0.020), (0.0038, 0.048), (0.0036, 0.058),
            (0.0040, L - 0.002), (0.0036, L), (0.0, L)]
    s = lathe(prof, seg=12).scale(0.75, 1, 1)
    hole = cyl(0.0021, 0.0021, 0.02, seg=10, bevel_r=0).rot(z=HALF).move(y=-L + 0.011)
    s = cut(s, hole)
    return s.fit((-0.0045, -L, -0.006), (0.0045, L, 0.006))

def spoon_bowl():
    # the spoon's bowl: a deep cup in the sphere frame that the placer
    # squashes (y 0.4) into a shallow bowl, narrowing towards the handle (+x)
    R = 0.014
    prof = [(R * math.sin(a), -R + 2 * R * (1 - math.cos(a))) for a in [0.35, 0.8, 1.2, HALF]]
    s = vessel(prof, 0.0028, seg=14, floor=0.006)
    s = s.warp(lambda p: Vector((p.x, p.y, p.z * (1 - 0.22 * max(0.0, p.x / R)))))
    return s.fit((-R, -R, -R), (R, R, R))

part('SpoonRests__dish', hemi(0.045), rest_dish)
part('SpoonRests__plate', f_cyl(0.070, 0.070, 0.004), rest_plate)
part('SpoonRests__handle', f_cyl(0.006, 0.006, 0.13), spoon_handle)
part('SpoonRests__bowl', f_sphere(0.014), spoon_bowl)


# ----------------------------------------------------------- recipe frame --
def recipe_frame():
    # rustic moulding, 2 cm border: eased outer edge, a scored line, a flat
    # face and a chamfer down to the sight edge; rebate behind the paper
    w, h, d = 0.20, 0.26, 0.014
    prof = [(0.0, -0.0022), (0.0007, -0.0005), (0.0018, 0.0), (0.0058, 0.0), (0.0064, -0.0011),
            (0.0070, 0.0), (0.0150, 0.0), (0.0192, -0.0026), (0.0200, -0.0026), (0.0204, -0.0060)]
    return pframe(w, h, d, prof, r=0.0015, seg=1)

part('FramedRecipes__frame', f_box(0.20, 0.26, 0.014), recipe_frame)


# ------------------------------------------------------------ shaker pair --
def salt_fill():
    # the salt inside the glass: a settled column with a grainy, mounded top
    prof = [(0.0, -0.015), (0.0192, -0.015), (0.0200, -0.0142), (0.0200, 0.0100), (0.0185, 0.0122),
            (0.0140, 0.0138), (0.0070, 0.0147), (0.0, 0.0150)]
    s = lathe(prof, seg=18)
    s = s.warp(lambda p: p if p.y < 0.009 else Vector((p.x, p.y + 0.0007 * noise.noise(p * 400), p.z)))
    return s.fit((-0.02, -0.015, -0.02), (0.02, 0.015, 0.02))

def shaker_cap():
    # chromed screw cap: flat pierced top, rounded shoulder, knurled skirt
    # flaring a little to the top, rolled lower edge
    rows = [(0.0, -0.0070, 0), (0.0212, -0.0070, 0), (0.0222, -0.0056, 0.0007), (0.0232, 0.0028, 0.0007),
            (0.0240, 0.0042, 0), (0.0228, 0.0064, 0), (0.0192, 0.0070, 0), (0.0, 0.0070, 0)]
    return lathe_rows(rows, 20)

def mill_body():
    # turned beech pepper mill: foot bead, cove, swelling belly, waist
    # beads, the head's parting line and a domed crown
    prof = [(0.0, -0.050), (0.0226, -0.050), (0.0240, -0.0485), (0.0240, -0.0435), (0.0222, -0.0415),
            (0.0214, -0.0370), (0.0228, -0.0280), (0.0222, -0.0120), (0.0186, 0.0040), (0.0172, 0.0120),
            (0.0184, 0.0150), (0.0172, 0.0180), (0.0176, 0.0240), (0.0186, 0.0255), (0.0172, 0.0270),
            (0.0184, 0.0330), (0.0200, 0.0400), (0.0198, 0.0450), (0.0170, 0.0485), (0.0100, 0.0500),
            (0.0, 0.0500)]
    return lathe(prof, seg=14)

def mill_knob():
    # finial nut: knurled ring under a domed top
    rows = [(0.0, -0.007, 0), (0.0072, -0.007, 0), (0.0080, -0.0052, 0.0006), (0.0080, 0.0015, 0.0006),
            (0.0062, 0.0058, 0), (0.0, 0.0070, 0)]
    return lathe_rows(rows, 8)

part('ShakerPair__salt', f_cyl(0.020, 0.020, 0.030), salt_fill)
part('ShakerPair__cap', f_cyl(0.024, 0.022, 0.014), shaker_cap)
part('ShakerPair__mill', f_cyl(0.020, 0.024, 0.100), mill_body)
part('ShakerPair__knob', f_cyl(0.008, 0.008, 0.014), mill_knob)


# ------------------------------------------------------ landscape (gilt) --
def gilt_frame():
    # gilt moulding, 2 cm border: outer half-round bead, a cove, a flat
    # frieze, a raised inner bead and a slip down to the sight edge
    w, h, d = 0.32, 0.24, 0.012
    prof = [(0.0, -0.0040), (0.0006, -0.0016), (0.0018, -0.0003), (0.0032, 0.0), (0.0046, -0.0005),
            (0.0056, -0.0020), (0.0068, -0.0034), (0.0090, -0.0038), (0.0120, -0.0038), (0.0130, -0.0026),
            (0.0142, -0.0012), (0.0156, -0.0010), (0.0168, -0.0024), (0.0200, -0.0036), (0.0204, -0.0070)]
    return pframe(w, h, d, prof, r=0.0005, seg=1)

part('AbstractLandscapes__frame', f_box(0.32, 0.24, 0.012), gilt_frame)


# ------------------------------------------------------------ bread loaf --
def bread_board():
    # oiled chopping board: rounded corners, eased edges, a juice groove
    # around the top
    w, d, t = 0.24, 0.14, 0.010
    rings = [(rr(w, d, 0.016, 3), -t / 2), (rr(w, d, 0.016, 3), t / 2 - 0.0022),
             (rr(w - 0.0016, d - 0.0016, 0.0152, 3), t / 2 - 0.0004), (rr(w - 0.0040, d - 0.0040, 0.014, 3), t / 2),
             (rr(w - 0.0140, d - 0.0140, 0.009, 3), t / 2), (rr(w - 0.0160, d - 0.0160, 0.008, 3), t / 2 - 0.0020),
             (rr(w - 0.0200, d - 0.0200, 0.006, 3), t / 2 - 0.0020), (rr(w - 0.0220, d - 0.0220, 0.005, 3), t / 2)]
    return loft(rings).rot(x=-HALF)

def loaf():
    # a baked batard in the sphere frame (the placer stretches it 2x long):
    # flat, spread base with a burst seam, domed crust, lumpy surface
    R = 0.048
    s = sphere(1.0, seg=22, rings=12)
    def shape(p):
        x, y, z = p.x, p.y, p.z
        if y < 0:
            t = -y
            y = -0.62 * (1 - (1 - t) ** 2)                                  # flat, spread base
            k = 1.0 + 0.08 * max(0.0, 1 - abs(t - 0.55) / 0.35)             # oven-spring bulge
            x, z = x * k, z * k
        else:
            y = y * (1.0 - 0.08 * (x * x))
        return Vector((x * R, y * R, z * R))
    s = s.warp(shape)
    s = noisy(s, 0.0011, scale=90, seed=4)
    return s.fit((-R, -R, -R), (R, R, R))

def bread_score():
    # a slash in the crust: a lens-shaped sliver whose ends dip with the loaf
    L, W, T = 0.045, 0.006, 0.003
    pts = []
    for i in range(9):
        t = i / 8
        pts.append((W / 2 * math.sin(math.pi * t) ** 0.7, -L / 2 + L * t))
    pts += [(-x, y) for x, y in reversed(pts[1:-1])]
    s = prism(pts, T).rot(x=HALF)            # outline in XZ, thickness on Y
    s = s.warp(lambda p: Vector((p.x, p.y - 0.0009 * (2 * p.z / L) ** 2, p.z)))
    return s.fit((-W / 2, -T / 2 - 0.0009, -L / 2), (W / 2, T / 2, L / 2))

part('BreadLoaves__board', f_box(0.24, 0.010, 0.14), bread_board)
part('BreadLoaves__loaf', f_sphere(0.048), loaf, smooth=180)
part('BreadLoaves__score', f_box(0.006, 0.003, 0.045), bread_score, smooth=60)


# ------------------------------------------------------------ still life --
def still_outer():
    # dark wood outer frame: 1.5 cm of visible border with a rounded outer
    # edge and a stepped ogee; the middle is covered by the gilt slip
    w, h, d = 0.34, 0.28, 0.016
    prof = [(0.0, -0.0050), (0.0008, -0.0018), (0.0022, -0.0004), (0.0040, 0.0), (0.0062, -0.0006),
            (0.0080, -0.0024), (0.0098, -0.0030), (0.0112, -0.0022), (0.0140, -0.0022), (0.0150, -0.0040),
            (0.0160, -0.0040), (0.0162, -0.0090)]
    return pframe(w, h, d, prof, r=0.0005, seg=1)

def still_slip():
    # gilt slip (liner): rises from the outer frame to a bead, then a cove
    # down to the sight edge; the middle is sunk well behind the canvas so
    # the painting (at the slip's front face) no longer z-fights with it
    w, h, d = 0.31, 0.25, 0.012
    prof = [(0.0, -0.0040), (0.0010, -0.0016), (0.0030, -0.0004), (0.0052, 0.0), (0.0074, -0.0004),
            (0.0090, -0.0016), (0.0120, -0.0022), (0.0160, -0.0014), (0.0190, -0.0012), (0.0198, -0.0020),
            (0.0202, -0.0070)]
    return pframe(w, h, d, prof, r=0.0005, seg=1)

part('StillLifes__outer', f_box(0.34, 0.28, 0.016), still_outer)
part('StillLifes__slip', f_box(0.31, 0.25, 0.012), still_slip)


# ----------------------------------------------------------- wine bottle --
def bottle_body():
    # the bottle's straight wall: a rolled heel into a pushed-up base,
    # meeting the shoulder at full width
    y0, y1 = -0.09, 0.09
    prof = [(0.0, y0 + 0.006), (0.020, y0 + 0.0015), (0.030, y0), (0.0340, y0 + 0.0012), (0.0356, y0 + 0.0042),
            (0.0360, y0 + 0.0090), (0.0360, y1 - 0.002), (0.0358, y1), (0.0, y1)]
    s = lathe(prof, seg=20)
    return s.fit((-0.036, y0, -0.036), (0.036, y1, 0.036))

def bottle_shoulder():
    # rounded bordeaux shoulder from the body up into the neck
    prof = [(0.0, -0.015), (0.0360, -0.015), (0.0342, -0.0062), (0.0296, 0.0000),
            (0.0222, 0.0074), (0.0160, 0.0126), (0.0140, 0.0150), (0.0, 0.0150)]
    return lathe(prof, seg=20)

def bottle_neck():
    # long neck, a touch wider where it leaves the shoulder
    prof = [(0.0, -0.030), (0.0140, -0.030), (0.0134, -0.024), (0.0130, -0.010), (0.0128, 0.028),
            (0.0126, 0.030), (0.0, 0.030)]
    return lathe(prof, seg=16).fit((-0.014, -0.03, -0.014), (0.014, 0.03, 0.014))

def bottle_foil():
    # crimped foil capsule: domed top with a rolled edge, the bottle lip
    # showing through as a bead, fine pleats down the skirt
    rows = [(0.0, -0.010, 0), (0.0150, -0.010, 0), (0.0152, -0.0085, 0.0005), (0.0150, 0.0030, 0.0005),
            (0.0160, 0.0048, 0), (0.0160, 0.0074, 0), (0.0150, 0.0090, 0), (0.0120, 0.0098, 0),
            (0.0, 0.0100, 0)]
    return lathe_rows(rows, 20)

part('WineBottles__body', f_cyl(0.036, 0.036, 0.18), bottle_body)
part('WineBottles__shoulder', f_cyl(0.014, 0.036, 0.030), bottle_shoulder)
part('WineBottles__neck', f_cyl(0.014, 0.014, 0.06), bottle_neck)
part('WineBottles__foil', f_cyl(0.016, 0.016, 0.020), bottle_foil)


# ---------------------------------------------------------- cheese wedge --
# The placer's quarter cylinders run theta 0..PI/2: x = r sin(t), z = r cos(t),
# so the wedge fills x, z >= 0 with its point on the Y axis.
def quarter(r_in, r_out, h, seg=10, r_round=0.0):
    pts = [(r_out * math.sin(HALF * i / seg), r_out * math.cos(HALF * i / seg)) for i in range(seg + 1)]
    if r_in > 0:
        pts += [(r_in * math.sin(HALF * i / seg), r_in * math.cos(HALF * i / seg)) for i in range(seg, -1, -1)]
    else:
        pts.append((0.0, 0.0))
    s = prism(pts, h)                    # outline in XY, height along Z
    s = s.rot(x=-HALF)                   # outline -> XZ (y -> -z), height -> Y
    s = s.scale(1, 1, -1)                # back to +z
    return s

def cheese_body():
    # the cut wedge: sharp cut faces with a few eyes scooped out of them
    R, h = 0.055, 0.040
    s = quarter(0, R, h, seg=8)
    eyes = [((0.014, 0.004, 0.0), 0.0045), ((0.0, -0.008, 0.024), 0.004),
            ((0.024, 0.020, 0.016), 0.0042), ((0.030, -0.020, 0.012), 0.0036)]
    cutters = [sphere(r, seg=7, rings=4).move(*c) for c, r in eyes]
    s = cut(s, *cutters)
    return s.fit((0, -h / 2, 0), (R, h / 2, R))

def cheese_rind():
    # waxed rind: a curved band over the wedge's back, rounded lips
    R, h = 0.056, 0.042
    s = quarter(R - 0.0035, R, h, seg=9)
    s = bevel(s, 0.0010, seg=1, angle=30)
    return s.fit((0, -h / 2, 0), (R, h / 2, R))

part('CheeseWedges__body', f_bounds((0, -0.020, 0), (0.055, 0.020, 0.055)), cheese_body, flat=False)
part('CheeseWedges__rind', f_bounds((0, -0.021, 0), (0.056, 0.021, 0.056)), cheese_rind)


# ---------------------------------------------------------- kitchen clock --
def clock_case():
    # round kitchen-clock case (front = +Y): a stepped-in back, full-round
    # side, rounded bezel lip standing proud around the face
    prof = [(0.0, -0.012), (0.0975, -0.012), (0.1000, -0.0105), (0.1080, -0.0075), (0.1100, -0.0045),
            (0.1100, 0.0060), (0.1092, 0.0098), (0.1072, 0.0120), (0.1052, 0.0114), (0.1040, 0.0100),
            (0.0, 0.0100)]
    return lathe(prof, seg=30)

def clock_tick():
    return rbox(0.006, 0.020, 0.003, r=0.0008, seg=1)

def clock_hand(w, L, t, boss, spade):
    # a pressed hand: round boss at the pivot (-Y), slim blade, leaf-shaped
    # spade (hour) or a plain taper (minute) to a point at +Y
    y0, y1 = -L / 2, L / 2
    pts = []
    cy = y0 + boss
    for i in range(9):                              # boss, lower half
        a = math.pi + math.pi * i / 8
        pts.append((boss * math.cos(a), cy + boss * math.sin(a)))
    neck = w * 0.26
    if spade:
        pts += [(neck, cy + boss * 1.4), (neck, y1 - L * 0.36), (w / 2, y1 - L * 0.20),
                (w * 0.28, y1 - L * 0.07), (0.0, y1)]
        pts += [(-x, y) for x, y in reversed(pts[9:-1])]
    else:
        pts += [(neck * 1.1, cy + boss * 1.4), (neck * 0.9, y1 - L * 0.30), (neck * 0.55, y1 - L * 0.06), (0.0, y1)]
        pts += [(-x, y) for x, y in reversed(pts[9:-1])]
    s = prism(pts, t)
    s = bevel(s, t * 0.18, seg=1, angle=40)
    return s.fit((-w / 2, y0, -t / 2), (w / 2, y1, t / 2))

part('KitchenClocks__case', f_cyl(0.11, 0.11, 0.024), clock_case)
part('KitchenClocks__tick', f_box(0.006, 0.020, 0.003), clock_tick)
part('KitchenClocks__hour', f_box(0.006, 0.050, 0.003), lambda: clock_hand(0.006, 0.050, 0.003, 0.003, True))
part('KitchenClocks__minute', f_box(0.004, 0.075, 0.003), lambda: clock_hand(0.004, 0.075, 0.003, 0.002, False))


# ------------------------------------------------------ rolling pin stand --
def pin_base():
    # stand base board: rounded corners, a chamfer-and-round top edge
    s = plate(0.30, 0.10, [(0.0, -0.005), (0.0, 0.0008), (0.0012, 0.0036), (0.0034, 0.005)], r=0.010, seg=2)
    return s.rot(x=-HALF)

def pin_post():
    # dowel peg: bead at the foot, rounded top
    prof = [(0.0, -0.0225), (0.0060, -0.0225), (0.0060, -0.0195), (0.0050, -0.0175), (0.0050, 0.0170),
            (0.0046, 0.0205), (0.0030, 0.0222), (0.0, 0.0225)]
    return lathe(prof, seg=10).fit((-0.006, -0.0225, -0.006), (0.006, 0.0225, 0.006))

def pin_roller():
    # the barrel: rounded shoulders, a pair of turned lines near each end
    L = 0.12
    half = [(0.0120, -L), (0.0168, -L + 0.0008), (0.0192, -L + 0.0032), (0.0200, -L + 0.0070),
            (0.0200, -L + 0.0170), (0.0193, -L + 0.0180), (0.0200, -L + 0.0190)]
    prof = [(0.0, -L)] + half + [(0.0200, 0.0)] + [(r, -y) for r, y in reversed(half)] + [(0.0, L)]
    return lathe(prof, seg=18)

def pin_handle():
    # turned handle, symmetric (the placer points the two ends both ways)
    prof = [(0.0, -0.020), (0.0050, -0.020), (0.0052, -0.0165), (0.0068, -0.0115), (0.0080, -0.0045),
            (0.0080, 0.0045), (0.0068, 0.0115), (0.0052, 0.0165), (0.0050, 0.020), (0.0, 0.020)]
    return lathe(prof, seg=12)

def pin_knob():
    # end ball, turned on the pin's axis (+X), eased flats on both ends
    prof = [(0.0, -0.012), (0.0055, -0.0116), (0.0098, -0.0082), (0.0118, -0.0035), (0.0120, 0.0),
            (0.0118, 0.0035), (0.0098, 0.0082), (0.0055, 0.0116), (0.0, 0.012)]
    return along_x(lathe(prof, seg=12))

part('RollingPinStands__base', f_box(0.30, 0.010, 0.10), pin_base)
part('RollingPinStands__post', f_cyl(0.006, 0.006, 0.045), pin_post)
part('RollingPinStands__roller', f_cyl(0.020, 0.020, 0.24), pin_roller)
part('RollingPinStands__handle', f_cyl(0.008, 0.008, 0.040), pin_handle)
part('RollingPinStands__knob', f_sphere(0.012), pin_knob, smooth=50)


# --------------------------------------------------------- wicker baskets --
def wicker_vessel(rt, rb, h, seg=22, rows=6, amp=0.0030, wall=0.005):
    """Open woven basket on +Y, centred: the outer wall is a checker of
    raised and sunken vertices (over-under weave), a rolled lip, a plain
    inner wall and a floor. seg must be even."""
    y0, y1 = -h / 2, h / 2
    v, f, rings = [], [], []
    def ring(r, y, rib, phase):
        ids = []
        for i in range(seg):
            a = TAU * (i + 0.5 * phase) / seg
            ri = r + (amp if (i + phase) % 2 else -amp * 0.4) * rib
            v.append((ri * math.sin(a), y, ri * math.cos(a)))
            ids.append(len(v) - 1)
        return ids
    v.append((0.0, y0, 0.0)); bottom = len(v) - 1
    rings.append(ring(rb - 0.006, y0, 0, 0))
    rings.append(ring(rb - 0.0005, y0 + 0.002, 0, 0))
    for k in range(rows + 1):
        t = k / rows
        y = y0 + 0.005 + (h - 0.011) * t
        rings.append(ring(rb + (rt - rb) * t, y, 1, k % 2))
    rings.append(ring(rt - wall * 0.35, y1, 0, 0))                 # rolled lip
    rings.append(ring(rt - wall, y1 - 0.004, 0, 0))                # inner wall
    rings.append(ring(rb - wall + (rt - rb) * 0.1, y0 + 0.006 + 0.004, 0, 0))
    v.append((0.0, y0 + 0.006, 0.0)); top = len(v) - 1
    for i in range(seg):
        f.append((bottom, rings[0][(i + 1) % seg], rings[0][i]))
    for a, b in zip(rings, rings[1:]):
        for i in range(seg):
            j = (i + 1) % seg
            f.append((a[i], a[j], b[j], b[i]))
    for i in range(seg):
        f.append((top, rings[-1][i], rings[-1][(i + 1) % seg]))
    return propkit._fix_winding(Shape(v, f))


def twist_ring(R, r, seg, tseg, amp=0.26, arc=TAU):
    """A torus (XY plane) whose tube bulges in a diagonal checker, like
    cane wrapped round a rim; a partial arc gets closed ends."""
    full = arc >= TAU - 1e-6
    n = seg if full else seg + 1
    v, f = [], []
    for i in range(n):
        a = arc * i / seg
        c = Vector((math.cos(a), math.sin(a), 0))
        for j in range(tseg):
            b = TAU * j / tseg
            rj = r * (1 + (amp if (i + j) % 2 else -amp * 0.5))
            if not full and (i == 0 or i == n - 1):
                rj = r * 0.85
            v.append(c * (R + rj * math.cos(b)) + Vector((0, 0, rj * math.sin(b))))
    for i in range(seg):
        i2 = (i + 1) % n if full else i + 1
        for j in range(tseg):
            j2 = (j + 1) % tseg
            f.append((i * tseg + j, i2 * tseg + j, i2 * tseg + j2, i * tseg + j2))
    if not full:
        f.append(tuple(reversed(range(tseg))))
        f.append(tuple(seg * tseg + j for j in range(tseg)))
    return propkit._fix_winding(Shape(v, f))


def folded_cloth():
    # three folded layers of fabric, each pillowed with a soft fold edge,
    # stacked a little out of true
    w, h, d = 0.12, 0.024, 0.09
    rings = []
    ys = [-0.012, -0.0108, -0.0085, -0.0045, -0.0040, -0.0035, 0.0005, 0.0040, 0.0045, 0.0085, 0.0108, 0.012]
    ins = [0.0040, 0.0010, 0.0, 0.0012, 0.0024, 0.0012, 0.0, 0.0012, 0.0024, 0.0012, 0.0012, 0.0045]
    for k, (y, i) in enumerate(zip(ys, ins)):
        layer = min(2, int((y + 0.012) / 0.008))
        dx = (-0.0018, 0.0015, -0.0006)[layer]
        dz = (0.0010, -0.0012, 0.0014)[layer]
        rings.append((rr(w - 0.004 - 2 * i, d - 0.004 - 2 * i, 0.006 - i, 2, cx=dx, cy=dz), y))
    s = loft(rings).rot(x=-HALF)
    s = noisy(s, 0.0007, scale=70, seed=2)
    return s.fit((-w / 2, -h / 2, -d / 2), (w / 2, h / 2, d / 2))


def spool_end():
    # spool flange: rounded rim, a hole through the middle
    prof = [(0.0030, -0.002), (0.0128, -0.002), (0.0138, -0.0014), (0.0140, 0.0), (0.0138, 0.0014),
            (0.0128, 0.002), (0.0030, 0.002), (0.0030, -0.002)]
    s = weld(lathe(prof, seg=16, cap_bottom=False, cap_top=False))
    return propkit._fix_winding(s)

def thread_wound(L):
    # thread wound on the spool: fine ridges of the winding
    y0 = -L / 2
    prof = [(0.0, y0), (0.0110, y0)]
    n = 8
    for k in range(n + 1):
        prof.append((0.0120 if k % 2 == 0 else 0.0114, y0 + 0.0012 + (L - 0.0024) * k / n))
    prof += [(0.0110, -y0), (0.0, -y0)]
    return lathe(prof, seg=12).fit((-0.012, y0, -0.012), (0.012, -y0, 0.012))

def pincushion():
    # tomato pincushion: six plump lobes pinched towards a tufted centre
    R = 0.020
    prof = [(0.0, 0.0), (0.0185, 0.0), (0.0200, 0.0030), (0.0195, 0.0080), (0.0170, 0.0130),
            (0.0120, 0.0172), (0.0055, 0.0190), (0.0, 0.0172)]
    s = lathe(prof, seg=18)
    def lobe(p):
        a = math.atan2(p.x, p.z)
        k = 1 - 0.12 * (0.5 + 0.5 * math.cos(6 * a))
        return Vector((p.x * k, p.y, p.z * k))
    s = s.warp(lobe)
    return s.fit((-R, 0, -R), (R, R, R))

part('SewingBaskets__body', f_cyl(0.10, 0.075, 0.070), lambda: wicker_vessel(0.10, 0.075, 0.070))
part('SewingBaskets__rim', f_torus(0.10, 0.006), lambda: twist_ring(0.10, 0.006, 40, 6))
part('SewingBaskets__handle', f_bounds((-0.10, 0, -0.005), (0.10, 0.10, 0.005)),
     lambda: twist_ring(0.095, 0.005, 26, 6, arc=math.pi))
part('SewingBaskets__fabric', f_box(0.12, 0.024, 0.09), folded_cloth, smooth=180)
part('SewingBaskets__spoolend', f_cyl(0.014, 0.014, 0.004), spool_end)
part('SewingBaskets__thread', f_cyl(0.012, 0.012, 0.020), lambda: thread_wound(0.020))
part('SewingBaskets__cushion', hemi(0.020), pincushion, smooth=180)


# ------------------------------------------------------- knitting basket --
def yarn_ball():
    # a wound ball of yarn: a core with three bands of wraps crossing over
    # it at different angles; each band stands proud with a groove down it
    R = 0.055
    core = sphere(R * 0.90, seg=10, rings=7)
    hw = R * 0.45
    def zone(y, k=1.0):
        return math.sqrt(max((R * k) ** 2 - y * y, 1e-8))
    prof = [(zone(-hw * 1.1, 0.88), -hw * 1.1), (zone(-hw * 0.55), -hw * 0.55), (zone(0.0, 0.975), 0.0),
            (zone(hw * 0.55), hw * 0.55), (zone(hw * 1.1, 0.88), hw * 1.1)]
    band = lathe(prof, seg=14, cap_bottom=False, cap_top=False)
    b1 = band.copy().rot(x=0.35, z=0.2)
    b2 = band.copy().rot(x=1.45, y=0.8)
    b3 = band.copy().rot(z=1.3, x=-0.5)
    return (core + b1 + b2 + b3).fit((-R, -R, -R), (R, R, R))

def yarn_wind():
    # a single loose wrap of yarn around the ball: wavy, round strand
    R, r = 0.055, 0.002
    seg, tseg = 36, 4
    v, f = [], []
    for i in range(seg):
        a = TAU * i / seg
        c = Vector((math.cos(a), math.sin(a), 0.0015 * math.sin(5 * a)))
        for j in range(tseg):
            b = TAU * j / tseg + math.pi / 4
            v.append(c * 1.0 + Vector((math.cos(a), math.sin(a), 0)) * (r * math.cos(b)) + Vector((0, 0, r * math.sin(b))))
    for i in range(seg):
        i2 = (i + 1) % seg
        for j in range(tseg):
            j2 = (j + 1) % tseg
            f.append((i * tseg + j, i2 * tseg + j, i2 * tseg + j2, i * tseg + j2))
    s = propkit._fix_winding(Shape(v, f))
    return s.fit((-R - r, -R - r, -r - 0.0012), (R + r, R + r, r + 0.0012))

def twist_cyl(r, L, seg=6, rows=10, amp=0.16):
    """A plied strand along +Y: the radius alternates in a diagonal
    checker so it reads as twisted cord; rounded ends."""
    v, f, rings = [], [], []
    v.append((0.0, -L / 2, 0.0)); b0 = 0
    for k in range(rows + 1):
        y = -L / 2 + L * k / rows
        rk = r * (0.7 if k in (0, rows) else 1.0)
        ids = []
        for i in range(seg):
            a = TAU * (i + 0.5 * (k % 2)) / seg
            ri = rk * (1 + (amp if (i + k) % 2 else -amp * 0.5))
            v.append((ri * math.sin(a), y, ri * math.cos(a)))
            ids.append(len(v) - 1)
        rings.append(ids)
    v.append((0.0, L / 2, 0.0)); b1 = len(v) - 1
    for i in range(seg):
        f.append((b0, rings[0][(i + 1) % seg], rings[0][i]))
        f.append((b1, rings[-1][i], rings[-1][(i + 1) % seg]))
    for a, b in zip(rings, rings[1:]):
        for i in range(seg):
            j = (i + 1) % seg
            f.append((a[i], a[j], b[j], b[i]))
    s = propkit._fix_winding(Shape(v, f))
    return s.fit((-r, -L / 2, -r), (r, L / 2, r))

def knit_needle():
    # bamboo-coloured needle, gently tapered to a rounded point at each end
    L = 0.11
    prof = [(0.0, -L), (0.0007, -L + 0.0012), (0.0015, -L + 0.0060), (0.0020, -L + 0.0140), (0.0020, L - 0.0140),
            (0.0015, L - 0.0060), (0.0007, L - 0.0012), (0.0, L)]
    return lathe(prof, seg=8)

def needle_knob():
    # turned end knob with a collar where it meets the needle
    prof = [(0.0, -0.006), (0.0030, -0.0058), (0.0032, -0.0046), (0.0055, -0.0022), (0.0060, 0.0006),
            (0.0048, 0.0042), (0.0024, 0.0058), (0.0, 0.006)]
    return lathe(prof, seg=8).fit((-0.006, -0.006, -0.006), (0.006, 0.006, 0.006))

part('KnittingBaskets__body', f_cyl(0.11, 0.090, 0.055), lambda: wicker_vessel(0.11, 0.090, 0.055, rows=5))
part('KnittingBaskets__rim', f_torus(0.11, 0.006), lambda: twist_ring(0.11, 0.006, 40, 6))
part('KnittingBaskets__yarn', f_sphere(0.055), yarn_ball, smooth=70)
part('KnittingBaskets__wind', f_torus(0.055, 0.002), yarn_wind, smooth=180)
part('KnittingBaskets__strand', f_cyl(0.0018, 0.0018, 0.028), lambda: twist_cyl(0.0018, 0.028, 6, 6), smooth=180)
part('KnittingBaskets__needle', f_cyl(0.002, 0.002, 0.22), knit_needle)
part('KnittingBaskets__knob', f_sphere(0.006), needle_knob, smooth=60)


# ------------------------------------------------------- hanging colander --
def colander_bowl():
    # pressed colander bowl in the upper-half-sphere frame (the placer flips
    # it, so the pole is the bottom): a foot ring at the pole, the dome, a
    # sheet-thick lip, and the inside back down to the pole
    R = 0.12
    outer = [(0.0, 0.1100), (0.0440, 0.1060), (0.0450, 0.1200), (0.0530, 0.1200), (0.0545, 0.1040)]
    outer += [(R * math.sin(a), R * math.cos(a)) for a in (0.62, 0.9, 1.15, 1.37, HALF)]
    inner = [(R - 0.0026, 0.0), (0.1122, 0.042), (0.0930, 0.0760), (0.0500, 0.1010), (0.0, 0.1060)]
    s = lathe(outer + inner, seg=20, cap_bottom=False, cap_top=False)
    return s.fit((-R, 0, -R), (R, R, R))

def colander_handle():
    # stamped side handle: a tab with a rounded end, a hanging slot and a
    # raised stiffening rib along it
    L, W, T = 0.08, 0.020, 0.010
    x0, x1 = -L / 2, L / 2
    pts = [(x0, -W / 2)] + [(x1 - W / 2 + W / 2 * math.sin(math.pi * i / 8), -W / 2 * math.cos(math.pi * i / 8))
                            for i in range(9)] + [(x0, W / 2)]
    body = prism(pts, 0.004).rot(x=-HALF).move(y=-T / 2 + 0.002)
    rib = rbox(L * 0.62, 0.006, 0.006, r=0.0028, seg=2).move(x=-0.008, y=-T / 2 + 0.004)
    slot = rbox(0.016, 0.03, 0.0075, r=0.0035, seg=2).move(x=x1 - 0.013)
    s = cut(bevel(body, 0.0012, seg=1, angle=40), slot) + rib
    return s.fit((-L / 2, -T / 2, -W / 2), (L / 2, T / 2, W / 2))

def wall_peg():
    # wall peg: shank into the wall (-Y), a rounded mushroom head (+Y)
    prof = [(0.0, -0.005), (0.0030, -0.005), (0.0030, 0.0012), (0.0060, 0.0022), (0.0052, 0.0045),
            (0.0, 0.0050)]
    return lathe(prof, seg=8)

part('HangingColanders__bowl', hemi(0.12), colander_bowl)
part('HangingColanders__rim', f_torus(0.12, 0.006), lambda: torus(0.12, 0.006, 40, 6))
part('HangingColanders__handle', f_box(0.08, 0.010, 0.020), colander_handle)
part('HangingColanders__cord', f_cyl(0.002, 0.002, 0.10), lambda: twist_cyl(0.002, 0.10, 6, 14), smooth=180)
part('HangingColanders__peg', f_cyl(0.006, 0.006, 0.010), wall_peg, smooth=50)


# ------------------------------------------------------------ herb sieve --
def sieve_handle():
    # flat steel handle: flared where it is riveted to the ring, a rounded
    # end with the hanging hole
    L, H, T = 0.15, 0.014, 0.006
    x0, x1 = -L / 2, L / 2
    top = [(x0, H / 2), (x0 + 0.012, H * 0.36), (x1 - H / 2, H * 0.36)]
    arc = [(x1 - H / 2 + H / 2 * math.sin(math.pi * i / 8), H / 2 * math.cos(math.pi * i / 8)) for i in range(1, 8)]
    pts = top + arc + [(x, -y) for x, y in reversed(top)]
    s = prism(pts, T)
    s = bevel(s, 0.0012, seg=1, angle=40)
    hole = cyl(0.0040, 0.0040, 0.02, seg=12, bevel_r=0).rot(x=HALF).move(x=x1 - 0.0100)
    rivet = on_z(lathe([(0.0022, 0.0), (0.0020, 0.0008), (0.0, 0.0012)], seg=8, cap_bottom=True)).move(x=x0 + 0.005, z=T / 2)
    s = cut(s, hole) + rivet
    return s.fit((x0, -H / 2, -T / 2), (x1, H / 2, T / 2))

part('HerbSieves__ring', f_torus(0.09, 0.006), lambda: torus(0.09, 0.006, 36, 6))
part('HerbSieves__handle', f_box(0.15, 0.014, 0.006), sieve_handle)


# -------------------------------------------------------- embroidery hoop --
def hoop_ring(R_out, radial, depth, seg, r_round, split=0.0, rseg=2):
    """A bent-wood hoop in the XY plane (like the placer's torus): a
    rounded-rectangle section `radial` wide and `depth` deep whose outer
    face sits on R_out; split > 0 cuts the gap for the tightening screw at
    the top."""
    r0, r1, z = R_out - radial, R_out, depth / 2
    sec = [(r1, -z), (r1, z), (r0, z), (r0, -z)]
    loop = round_outline(sec, r_round, seg=rseg)
    v, f = [], []
    n = len(loop)
    for i in range(seg):
        a = TAU * i / seg
        for r, zz in loop:
            v.append((r * math.cos(a), r * math.sin(a), zz))
    for i in range(seg):
        i2 = (i + 1) % seg
        for j in range(n):
            j2 = (j + 1) % n
            f.append((i * n + j, i2 * n + j, i2 * n + j2, i * n + j2))
    s = propkit._fix_winding(Shape(v, f))
    if split > 0:
        s = cut(s, box(split, radial * 4, depth * 3).move(y=R_out - radial / 2))
    return s

def hoop_outer():
    # outer hoop: 12 mm of bent beech, 18 mm deep, split under the screw
    s = hoop_ring(0.145, 0.012, 0.018, 44, 0.004, split=0.004)
    return s.fit((-0.145, -0.145, -0.009), (0.145, 0.145, 0.009))

def hoop_inner():
    # inner hoop: thinner ring the fabric is stretched over
    s = hoop_ring(0.130, 0.008, 0.009, 32, 0.0026, rseg=1)
    return s.fit((-0.130, -0.130, -0.0045), (0.130, 0.130, 0.0045))

def hoop_screw():
    # tightening hardware: two lugs, the screw through them, a knurled
    # thumb nut at +x and a domed head at -x
    lugs = box(0.0036, 0.011, 0.010).move(x=-0.0028, y=-0.0015) + box(0.0036, 0.011, 0.010).move(x=0.0028, y=-0.0015)
    shaft = along_x(lathe([(0.0016, -0.010), (0.0016, 0.007)], seg=6, cap_bottom=False, cap_top=False))
    nut = along_x(lathe_rows([(0.0, 0.0060, 0), (0.0070, 0.0060, 0.0008), (0.0070, 0.0100, 0.0008), (0.0, 0.0100, 0)], 10))
    head = along_x(lathe([(0.0, -0.0100), (0.0035, -0.0090), (0.0035, -0.0068), (0.0, -0.0068)], seg=6))
    s = lugs + shaft + nut + head
    return s.fit((-0.010, -0.007, -0.007), (0.010, 0.007, 0.007))

part('EmbroideryHoops__outer', f_torus(0.135, 0.010), hoop_outer)
part('EmbroideryHoops__inner', f_torus(0.125, 0.005), hoop_inner)
part('EmbroideryHoops__screw', f_box(0.020, 0.014, 0.014), hoop_screw)


# ------------------------------------------------ picture & photo frames --
def sampler_frame():
    # wooden sampler frame, 1 cm border: rounded outer edge, a bead, and a
    # small cove down to the sight edge just under the cloth
    w, h, d = 0.28, 0.22, 0.016
    prof = [(0.0, -0.0050), (0.0008, -0.0018), (0.0020, -0.0004), (0.0036, 0.0), (0.0050, -0.0006),
            (0.0062, -0.0016), (0.0078, -0.0012), (0.0094, -0.0008), (0.0100, -0.0010), (0.0104, -0.0060)]
    return pframe(w, h, d, prof, r=0.0006, seg=1)

def family_frame():
    # classic half-round moulding, 1 cm border
    w, h, d = 0.24, 0.19, 0.014
    prof = [(0.0, -0.0048), (0.0008, -0.0022), (0.0022, -0.0006), (0.0040, 0.0), (0.0058, -0.0005),
            (0.0074, -0.0018), (0.0088, -0.0030), (0.0100, -0.0032), (0.0104, -0.0060)]
    return pframe(w, h, d, prof, r=0.0005, seg=1)

def baby_frame():
    # nursery frame: soft rounded corners and a pillowed face
    w, h, d = 0.20, 0.20, 0.014
    prof = [(0.0, -0.0050), (0.0010, -0.0022), (0.0026, -0.0006), (0.0046, 0.0), (0.0068, -0.0006),
            (0.0084, -0.0018), (0.0090, -0.0022), (0.0094, -0.0060)]
    return pframe(w, h, d, prof, r=0.016, seg=3)

def wedding_frame():
    # ornate moulding: outer bead, cove, a raised torus bead, then a flat
    # slip to the sight edge
    w, h, d = 0.20, 0.24, 0.016
    prof = [(0.0, -0.0060), (0.0006, -0.0034), (0.0016, -0.0024), (0.0026, -0.0030), (0.0034, -0.0042),
            (0.0044, -0.0030), (0.0052, -0.0008), (0.0062, 0.0), (0.0072, -0.0008), (0.0080, -0.0026),
            (0.0098, -0.0030), (0.0102, -0.0060)]
    return pframe(w, h, d, prof, r=0.0005, seg=1)

def graduation_frame():
    # document frame: flat wide face, eased outer edge, a thin raised lip
    # around the sight edge
    w, h, d = 0.18, 0.22, 0.014
    prof = [(0.0, -0.0030), (0.0006, -0.0010), (0.0016, -0.0006), (0.0068, -0.0006), (0.0072, 0.0),
            (0.0084, 0.0), (0.0090, -0.0014), (0.0094, -0.0060)]
    return pframe(w, h, d, prof, r=0.0005, seg=1)

def announcement_frame():
    # soft painted frame with slightly rounded corners and a scotia step
    w, h, d = 0.20, 0.22, 0.014
    prof = [(0.0, -0.0042), (0.0008, -0.0016), (0.0022, -0.0003), (0.0040, 0.0), (0.0052, -0.0008),
            (0.0060, -0.0024), (0.0072, -0.0026), (0.0098, -0.0020), (0.0104, -0.0060)]
    return pframe(w, h, d, prof, r=0.005, seg=2)

def postcard_frame():
    # slim vintage frame, 9 mm border: stepped profile with a fine bead
    w, h, d = 0.24, 0.17, 0.012
    prof = [(0.0, -0.0036), (0.0006, -0.0012), (0.0016, 0.0), (0.0034, 0.0), (0.0040, -0.0014),
            (0.0056, -0.0014), (0.0063, -0.0006), (0.0072, -0.0012), (0.0088, -0.0022), (0.0092, -0.0050)]
    return pframe(w, h, d, prof, r=0.0005, seg=1)

part('CrossStitchSamplers__frame', f_box(0.28, 0.22, 0.016), sampler_frame)
part('FamilyPhotos__frame', f_box(0.24, 0.19, 0.014), family_frame)
part('BabyPhotos__frame', f_box(0.20, 0.20, 0.014), baby_frame)
part('WeddingPhotos__frame', f_box(0.20, 0.24, 0.016), wedding_frame)
part('GraduationPhotos__frame', f_box(0.18, 0.22, 0.014), graduation_frame)
part('BirthAnnouncements__frame', f_box(0.20, 0.22, 0.014), announcement_frame)
part('VintagePostcards__frame', f_box(0.24, 0.17, 0.012), postcard_frame)


# --------------------------------------------------------------- ashtray --
def ash_pile():
    # a low, crumbly heap of ash with a ragged outline
    prof = [(0.0, -0.0015), (0.0196, -0.0015), (0.0188, -0.0007), (0.0150, 0.0002), (0.0090, 0.0010),
            (0.0035, 0.0015), (0.0, 0.0015)]
    s = lathe(prof, seg=18)
    def rag(p):
        a = math.atan2(p.x, p.z)
        k = 1 + 0.10 * math.sin(3 * a + 0.7) * math.sin(5 * a)
        return Vector((p.x * k, p.y + (0.0004 * noise.noise(p * 600) if p.y > -0.0014 else 0.0), p.z * k))
    s = s.warp(rag)
    return s.fit((-0.02, -0.0015, -0.02), (0.02, 0.0015, 0.02))

def cig_paper():
    # paper tube, burnt down to a ragged, charred end at +Y
    L = 0.049
    y0, y1 = -L / 2, L / 2
    prof = [(0.0, y0), (0.0027, y0), (0.0030, y0 + 0.0005), (0.0030, y1 - 0.0040), (0.0028, y1 - 0.0016),
            (0.0023, y1 - 0.0004), (0.0010, y1), (0.0, y1 - 0.0003)]
    s = lathe(prof, seg=10)
    s = s.warp(lambda p: Vector((p.x, p.y + (0.0006 * noise.noise(p * 2500) if p.y > y1 - 0.0020 else 0.0), p.z)))
    return s.fit((-0.003, y0, -0.003), (0.003, y1, 0.003))

def cig_filter():
    # cork filter: eased ends, the fibre face dished slightly at -Y
    L = 0.021
    prof = [(0.0, -L / 2 + 0.0004), (0.0024, -L / 2), (0.0030, -L / 2 + 0.0006), (0.0030, L / 2),
            (0.0, L / 2)]
    return lathe(prof, seg=8).fit((-0.003, -L / 2, -0.003), (0.003, L / 2, 0.003))

part('SmokingAshtrays__ash', f_cyl(0.020, 0.020, 0.003), ash_pile, smooth=70)
part('SmokingAshtrays__paper', f_cyl(0.003, 0.003, 0.049), cig_paper, smooth=60)
part('SmokingAshtrays__filter', f_cyl(0.003, 0.003, 0.021), cig_filter, smooth=60)


# ----------------------------------------------------------- open diary --
def diary_pages():
    # the open page block: both leaves rise from a sunken gutter into a
    # soft hump and roll down to the fore-edges
    W, T, D = 0.22, 0.006, 0.15
    y0 = -T / 2
    def top(u):
        g = 0.52 + 0.48 * math.sin(min(1.0, u / 0.28) * HALF) ** 0.8
        return y0 + T * (g - 0.20 * max(0.0, (u - 0.35) / 0.65) ** 1.4)
    us = [0.0, 0.03, 0.07, 0.12, 0.18, 0.26, 0.36, 0.5, 0.65, 0.8, 0.9, 0.96, 1.0]
    right = [(u * W / 2, top(u)) for u in us]
    right[-1] = (W / 2, top(1.0) - 0.0012)
    pts = [(-W / 2, y0), (W / 2, y0)] + list(reversed(right)) + [(-x, y) for x, y in right[1:]]
    s = prism(pts, D)
    return s.fit((-W / 2, y0, -D / 2), (W / 2, T / 2, D / 2))

def diary_spine():
    # the spine roll showing in the gutter, rounded at head and tail
    L = 0.075
    prof = [(0.0, -L), (0.0022, -L + 0.0003), (0.0030, -L + 0.0015), (0.0030, L - 0.0015), (0.0022, L - 0.0003),
            (0.0, L)]
    return lathe(prof, seg=8)

def diary_cover():
    # hardcover board: rounded corners, eased edges
    w, d, t = 0.1155, 0.1575, 0.003
    return plate(w, d, [(0.0, -t / 2), (0.0, t / 2 - 0.0008), (0.0007, t / 2)], r=0.006, seg=2).rot(x=-HALF)

def diary_ribbon():
    # satin ribbon marker: gently rippled, swallowtail cut at the free end (+Z)
    W, T, L = 0.008, 0.0008, 0.06
    n = 12
    side = [(W / 2, -L / 2 + L * i / n) for i in range(n + 1)]
    pts = side + [(0.0, L / 2 - 0.006)] + [(-x, y) for x, y in reversed(side)]
    s = prism(pts, T).rot(x=-HALF)                       # outline -> XZ (y -> -z), thickness -> Y
    s = s.warp(lambda p: Vector((p.x, p.y + 0.0005 * math.sin(p.z / L * 9.0), p.z)))
    return s.fit((-W / 2, -0.001, -L / 2), (W / 2, 0.001, L / 2))

def pen_barrel():
    # ballpoint barrel: plain end at -Y, ribbed grip tapering to the tip (+Y)
    prof = [(0.0, -0.050), (0.0034, -0.050), (0.0040, -0.0490), (0.0040, 0.0280), (0.0036, 0.0300),
            (0.0038, 0.0330), (0.0035, 0.0355), (0.0038, 0.0380), (0.0035, 0.0405), (0.0036, 0.0450),
            (0.0030, 0.0500), (0.0, 0.0500)]
    return lathe(prof, seg=10)

def pen_tip():
    # writing cone with the ball point at the apex
    prof = [(0.0, -0.005), (0.0030, -0.005), (0.0025, -0.0015), (0.0012, 0.0033), (0.0006, 0.0044), (0.0, 0.005)]
    return lathe(prof, seg=8)

def pen_button():
    # click button: collar against the barrel (+Y), plunger outwards (-Y)
    prof = [(0.0, -0.003), (0.0026, -0.003), (0.0028, -0.0016), (0.0044, -0.0012), (0.0050, -0.0002),
            (0.0050, 0.003), (0.0, 0.003)]
    return lathe(prof, seg=8).fit((-0.005, -0.003, -0.005), (0.005, 0.003, 0.005))

def pen_clip():
    # pocket clip: tapered strip with a rounded free end at -X
    L, W, T = 0.020, 0.004, 0.001
    pts = [(L / 2, -W / 2), (L / 2, W / 2), (-L / 2 + W * 0.5, W * 0.38)]
    pts += [(-L / 2 + W * 0.5 - W * 0.4 * math.sin(math.pi * i / 6), W * 0.38 * math.cos(math.pi * i / 6)) for i in range(1, 6)]
    pts += [(-L / 2 + W * 0.5, -W * 0.38)]
    s = prism(pts, T).rot(x=-HALF)
    s = bevel(s, 0.0003, seg=1, angle=40)
    return s.fit((-L / 2, -T / 2, -W / 2), (L / 2, T / 2, W / 2))

part('OpenDiaries__pages', f_box(0.22, 0.006, 0.15), diary_pages)
part('OpenDiaries__spine', f_cyl(0.003, 0.003, 0.15), diary_spine)
part('OpenDiaries__cover', f_box(0.1155, 0.003, 0.1575), diary_cover)
part('OpenDiaries__ribbon', f_box(0.008, 0.002, 0.06), diary_ribbon, smooth=60)
part('OpenDiaries__pen', f_cyl(0.004, 0.004, 0.10), pen_barrel)
part('OpenDiaries__tip', f_cone(0.003, 0.010), pen_tip)
part('OpenDiaries__button', f_cyl(0.005, 0.005, 0.006), pen_button)
part('OpenDiaries__clip', f_box(0.020, 0.001, 0.004), pen_clip)


# -------------------------------------------------------- sewing machine --
def section(w, h, rb, rt, seg=2):
    """A w x h cross-section outline with bottom corners rounded rb and
    top corners rounded rt (constant point count)."""
    c = [(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2)]
    out = []
    for i, r in enumerate((rb, rb, rt, rt)):
        out += propkit._round_corner(c[i], c[i - 1], c[(i + 1) % 4], r, seg)
    return out

def machine_base():
    # wooden base: rounded corners, an ogee-ish routed top edge
    w, d, t = 0.24, 0.14, 0.014
    s = plate(w, d, [(0.0, -t / 2), (0.0, 0.0), (0.0010, 0.0022), (0.0030, 0.0030), (0.0036, 0.0052),
                     (0.0048, t / 2)], r=0.008, seg=2)
    return s.rot(x=-HALF)

def machine_bed():
    # cast bed plate: rounded corners, eased edge, a slide-plate panel line
    # under the needle
    w, d, t = 0.204, 0.091, 0.010
    s = plate(w, d, [(0.0, -t / 2), (0.0, t / 2 - 0.0020), (0.0012, t / 2 - 0.0003), (0.0024, t / 2)],
              r=0.010, seg=2).rot(x=-HALF)
    groove = cut(box(0.046, 0.004, 0.044), box(0.043, 0.01, 0.041)).move(x=0.070, y=t / 2)
    return cut(s, groove)

def machine_column():
    # the pillar: rounded upright, narrowing as it rises, flaring into the bed
    w, h, d = 0.025, 0.10, 0.077
    rings = [(rr(w, d, 0.006, 2), -h / 2), (rr(w, d, 0.006, 2), -h / 2 + 0.004),
             (rr(w * 0.86, d * 0.90, 0.006, 2), -h / 2 + 0.014), (rr(w * 0.80, d * 0.80, 0.0055, 2), h / 2 - 0.006),
             (rr(w * 0.82, d * 0.82, 0.0055, 2), h / 2)]
    s = loft(rings).rot(x=-HALF)
    return s.fit((-w / 2, -h / 2, -d / 2), (w / 2, h / 2, d / 2))

def machine_arm():
    # the arm: bullnose top, flat underside, rounded ends
    L, h, d = 0.18, 0.026, 0.077
    sec = lambda k: [(x * k, y * k + (1 - k) * 0.0) for x, y in section(d, h, 0.003, 0.010, 2)]
    rings = [(sec(0.88), -L / 2), (sec(0.97), -L / 2 + 0.0025), (sec(1.0), -L / 2 + 0.006),
             (sec(1.0), L / 2 - 0.006), (sec(0.97), L / 2 - 0.0025), (sec(0.88), L / 2)]
    s = loft(rings).rot(y=HALF)                     # length along X, section in YZ
    return s.fit((-L / 2, -h / 2, -d / 2), (L / 2, h / 2, d / 2))

def machine_foot():
    # presser foot: a shank with a thumb screw, and the ski-shaped sole
    shank = rbox(0.0065, 0.016, 0.009, r=0.0012, seg=1).move(y=0.003)
    ski = [(-0.015, -0.011), (0.010, -0.011), (0.0135, -0.0098), (0.015, -0.0070), (0.0125, -0.0068),
           (0.0095, -0.0080), (-0.015, -0.0080)]
    sole = prism([(z, y) for z, y in ski], 0.014).rot(y=-HALF)       # outline (z, y) in XY -> ZY
    sole = sole.warp(lambda p: Vector((p.x, p.y, p.z)))
    screw = along_x(lathe([(0.0, 0.0030), (0.0030, 0.0030), (0.0030, 0.0070), (0.0, 0.0070)], seg=8)).move(y=0.006)
    s = shank + sole + screw
    return s.fit((-0.007, -0.011, -0.015), (0.007, 0.011, 0.015))

def hand_wheel():
    # balance wheel (axis +Y): rolled rim, four spokes, a hub with a boss
    rim = lathe([(0.0215, -0.004), (0.0266, -0.004), (0.0280, -0.0016), (0.0280, 0.0016), (0.0266, 0.004),
                 (0.0215, 0.004), (0.0215, -0.004)], seg=14, cap_bottom=False, cap_top=False)
    hub = lathe([(0.0, -0.004), (0.0070, -0.004), (0.0070, 0.0030), (0.0040, 0.0040), (0.0, 0.0040)], seg=8)
    spokes = sum((box(0.017, 0.0045, 0.0045).move(x=0.0145).rot(y=HALF * k) for k in range(4)), Shape())
    return (weld(rim) + hub + spokes).fit((-0.028, -0.004, -0.028), (0.028, 0.004, 0.028))

def crank_knob():
    # wooden crank knob turned on +X: a stem at the wheel, a round grip
    prof = [(0.0, -0.005), (0.0022, -0.005), (0.0022, -0.0018), (0.0045, -0.0002), (0.0050, 0.0020),
            (0.0036, 0.0044), (0.0, 0.005)]
    return along_x(lathe(prof, seg=8)).fit((-0.005, -0.005, -0.005), (0.005, 0.005, 0.005))

def cloth_piece():
    # a scrap of cloth under the foot: soft ripples and a lifted corner
    s = slab(0.06, 0.05, 0.0008, 6, 5).rot(x=-HALF)
    s = s.warp(lambda p: Vector((p.x, p.y + 0.0005 * math.sin(p.x * 140 + p.z * 60)
                                 + 0.0007 * max(0.0, (p.x + p.z * 0.8) / 0.055) ** 2, p.z)))
    return s.fit((-0.03, -0.001, -0.025), (0.03, 0.001, 0.025))

part('SewingMachines__base', f_box(0.24, 0.014, 0.14), machine_base)
part('SewingMachines__bed', f_box(0.204, 0.010, 0.091), machine_bed)
part('SewingMachines__column', f_box(0.025, 0.10, 0.077), machine_column)
part('SewingMachines__arm', f_box(0.18, 0.026, 0.077), machine_arm)
part('SewingMachines__foot', f_box(0.014, 0.022, 0.030), machine_foot)
part('SewingMachines__thread', f_cyl(0.012, 0.012, 0.028), lambda: thread_wound(0.028))
part('SewingMachines__wheel', f_cyl(0.028, 0.028, 0.008), hand_wheel)
part('SewingMachines__knob', f_sphere(0.005), crank_knob, smooth=60)
part('SewingMachines__cloth', f_box(0.06, 0.002, 0.05), cloth_piece, smooth=60)


# ---------------------------------------------------------------- wallet --
def wallet_half():
    # one leaf of the open bi-fold: rounded corners, eased edge, a stitch
    # groove round the edge, the leather slightly pillowed inside it
    w, d, t = 0.045, 0.075, 0.006
    prof = [(0.0, -t / 2), (0.0, t / 2 - 0.0016), (0.0007, t / 2 - 0.0004), (0.0018, t / 2 - 0.0002),
            (0.0030, t / 2 - 0.0002), (0.0034, t / 2 - 0.0010), (0.0038, t / 2 - 0.0003), (0.0100, t / 2)]
    s = plate(w, d, prof, r=0.005, seg=2).rot(x=-HALF)
    return s.fit((-w / 2, -t / 2, -d / 2), (w / 2, t / 2, d / 2))

def wallet_fold():
    # the fold: a soft roll of leather, rounded at both ends
    pts = [(0.002 * math.cos(TAU * i / 10), 0.004 * math.sin(TAU * i / 10)) for i in range(10)]
    s = prism(pts, 0.075)
    s = bevel(s, 0.0012, seg=1, angle=40)
    return s.fit((-0.002, -0.004, -0.0375), (0.002, 0.004, 0.0375))

def wallet_pocket():
    # card-pocket edge: a turned leather strip with its stitch line
    w, d, t = 0.03825, 0.006, 0.002
    s = plate(w, d, [(0.0, -t / 2), (0.0, t / 2 - 0.0006), (0.0006, t / 2)], r=0.0015, seg=1).rot(x=-HALF)
    return cut(s, box(w - 0.004, 0.0012, 0.0006).move(y=t / 2, z=-0.0008))

def id_card():
    # plastic card: rounded corners, eased edges
    w, d, t = 0.03825, 0.04125, 0.002
    return plate(w, d, [(0.0, -t / 2), (0.0, t / 2 - 0.0005), (0.0005, t / 2)], r=0.0032, seg=2).rot(x=-HALF)

def creased(w, d, t, h, nx=10, nz=3):
    """A thin sheet (w x d, t thick) folded in half across X and opened
    again: both halves rise h towards the ends from the crease at x = 0."""
    s = slab(w, d, t, nx, nz).rot(x=-HALF)
    s = s.warp(lambda p: Vector((p.x, p.y + h * (abs(p.x) / (w / 2)) ** 1.3, p.z)))
    return s

def bill():
    s = creased(0.06, 0.030, 0.0004, 0.0006, 10, 3)
    return s.fit((-0.03, -0.0005, -0.015), (0.03, 0.0005, 0.015))

def bill_band():
    # the dark band follows the same crease as its bill
    s = creased(0.06, 0.008, 0.0006, 0.0006, 10, 1)
    return s.fit((-0.03, -0.0006, -0.004), (0.03, 0.0006, 0.004))

def coin():
    # coin: raised rim on both faces, flat fields
    prof = [(0.0, -0.0007), (0.0058, -0.0007), (0.0070, -0.0010), (0.0070, 0.0010), (0.0058, 0.0007), (0.0, 0.0007)]
    return lathe(prof, seg=10)

part('Wallets__half', f_box(0.045, 0.006, 0.075), wallet_half)
part('Wallets__fold', f_box(0.004, 0.008, 0.075), wallet_fold)
part('Wallets__pocket', f_box(0.03825, 0.002, 0.006), wallet_pocket)
part('Wallets__card', f_box(0.03825, 0.002, 0.04125), id_card)
part('Wallets__bill', f_box(0.06, 0.001, 0.030), bill)
part('Wallets__band', f_box(0.06, 0.0012, 0.008), bill_band)
part('Wallets__coin', f_cyl(0.007, 0.007, 0.002), coin)
