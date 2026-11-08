"""Batch b16 - "toys" and "party": bright, chunky, rounded plastic, rubber,
plush and paper things lying on the floor.

place_childrens_tea_sets  teapot (belly, gallery rim, lid), spout, handle, cup, saucer
place_deflated_balls     slumped ball with a caved-in side and wrinkles
place_rubber_ducks       duck body with a raised tail and wings, head, bill
place_gameboys          handheld body, bezel, LCD, D-pad arms, buttons, pills
place_yoyos             yo-yo halves, string hub
place_rubiks_cubes       grooved core
place_lego_bricks        hollow brick, stud
place_jump_ropes         coiled cord, turned handle
place_teddy_bears        plush body, head with muzzle, ears, arms, legs, eyes, nose
place_crayons           paper wrapper, sharpened tip, wax end
place_action_figures     torso, head, arm, leg
place_baby_rattles       lobed ball, handle, teether ring
place_toy_dinosaurs      body, head, tail, legs, back plates
place_pacifiers         shield, teat, ring
place_sippy_cups         cup, screw lid, spout, handle
place_baby_bottles       milk, screw collar, teat
place_stuffed_rabbits    body, head, ears, arms, feet, tail, nose
place_stuffed_elephants  body, head, trunk, ears, legs, tusks
place_teething_rings     nubbed ring, ribbed handle
place_child_drawings     moulded frame
place_pet_toys           bone shaft and knobs, rubber ball, stripe
place_broken_whole_dolls  torso, dress, ruffles, head, ringlets, limbs, hand, shoe
place_hobby_horses       pole, knob, carved head, muzzle, ears, mane, reins, wheels
place_small_balloons     balloon, knot, deflated skin, string
"""
from propkit import *
import propkit
from mathutils import Vector, noise

fixw = propkit._fix_winding
HPI = math.pi / 2


# ------------------------------------------------------------- helpers ---
def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


def fitb(s, w, h, d):
    return s.fit((-w / 2, -h / 2, -d / 2), (w / 2, h / 2, d / 2))


def rsphere(fn, seg=16, rings=10):
    """UV sphere whose radius follows fn(unit direction) -> radius."""
    return sphere(1.0, seg, rings).warp(lambda p: p.normalized() * fn(p.normalized()) if p.length > 1e-9 else p)


def gauss(d, c, w):
    """1 at direction c, falling off with the angle to it (width w radians)."""
    c = Vector(c).normalized()
    a = math.acos(max(-1.0, min(1.0, d.dot(c))))
    return math.exp(-(a / w) ** 2)


def ring_lathe(profile, seg=24):
    """Revolve a CLOSED profile loop [(r, y), ...] around +Y."""
    v, f, n = [], [], len(profile)
    for r, y in profile:
        for i in range(seg):
            a = TAU * i / seg
            v.append((r * math.sin(a), y, r * math.cos(a)))
    for k in range(n):
        a, b = k * seg, ((k + 1) % n) * seg
        for i in range(seg):
            j = (i + 1) % seg
            f.append((a + i, b + i, b + j, a + j))
    return fixw(Shape(v, f))


def arc_sweep(section, R, a0, a1, n, caps=True, rfn=None):
    """Sweep a closed 2D section [(dr, dz), ...] around the Z axis at radius R
    from angle a0 to a1 (a ring in the XY plane, like torus_geo!).
    rfn(t) optionally scales the section along the sweep (t = 0..1)."""
    m = len(section)
    full = abs(a1 - a0) >= TAU - 1e-6
    steps = n if full else n + 1
    v, f = [], []
    for i in range(steps):
        t = i / n
        a = a0 + (a1 - a0) * t
        k = rfn(t) if rfn else 1.0
        c, s = math.cos(a), math.sin(a)
        v += [((R + dr * k) * c, (R + dr * k) * s, dz * k) for dr, dz in section]
    for i in range(n):
        i2 = (i + 1) % steps
        for j in range(m):
            jj = (j + 1) % m
            f.append((i * m + j, i * m + jj, i2 * m + jj, i2 * m + j))
    if caps and not full:
        f.append(tuple(reversed(range(m))))
        f.append(tuple(n * m + j for j in range(m)))
    return fixw(Shape(v, f))


def circ(r, n=8, sx=1.0, sy=1.0):
    return [(r * sx * math.cos(TAU * k / n), r * sy * math.sin(TAU * k / n)) for k in range(n)]


def half_torus_frame(R, r):
    """Bounds of torus_geo!(R, r, .., .., PI): the upper half ring."""
    return f_bounds((-R - r, 0.0, -r), (R + r, R + r, r))


def loft(rings, caps=True):
    """Bridge equal-length 3D rings; cap both ends."""
    v, f = [], []
    n = len(rings[0])
    for ring in rings:
        v += [tuple(p) for p in ring]
    for r in range(len(rings) - 1):
        a, b = r * n, (r + 1) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    if caps:
        f.append(tuple(reversed(range(n))))
        f.append(tuple((len(rings) - 1) * n + i for i in range(n)))
    return fixw(Shape(v, f))


def rr4(w, d, rads, seg=3):
    """Rounded-rect outline (x, z), CCW, per-corner radii for the corners
    (+x+z, -x+z, -x-z, +x-z); constant point count for any inset."""
    pts = []
    corners = [(w / 2, d / 2, 0.0), (-w / 2, d / 2, HPI), (-w / 2, -d / 2, math.pi), (w / 2, -d / 2, 1.5 * math.pi)]
    for (cx, cz, a0), r in zip(corners, rads):
        ox = cx - math.copysign(r, cx)
        oz = cz - math.copysign(r, cz)
        for k in range(seg + 1):
            a = a0 + HPI * k / seg
            pts.append((ox + r * math.cos(a), oz + r * math.sin(a)))
    return pts


def ystack(outline, prof):
    """Flat moulding: outline(inset) -> [(x, z)] rings at heights y for each
    (inset, y) in prof (bottom to top); capped."""
    return loft([[(x, y, z) for x, z in outline(i)] for i, y in prof])


def rrect_out(w, d, r, seg=2):
    return lambda i: rr4(w - 2 * i, d - 2 * i, [max(r - i, 1e-4)] * 4, seg)


# ----------------------------------------------------------- tea sets --
def tea_pot():
    # round-bellied teapot on a foot ring, shoulder rising to a gallery rim
    # with a recessed seat for the lid; built at its squashed (x0.8) size
    s = lathe([(0, -0.0192), (0.0125, -0.0192), (0.014, -0.016), (0.021, -0.0105), (0.024, -0.002),
               (0.022, 0.008), (0.0155, 0.015), (0.0095, 0.0192), (0.0085, 0.0172), (0, 0.0172)], seg=16)
    return fitf(s, f_sphere(0.024))


def tea_spout():
    # narrow where it leaves the pot, flaring to a hollow lipped mouth
    s = lathe([(0, -0.01), (0.0031, -0.01), (0.0027, 0.0), (0.0036, 0.0065), (0.005, 0.01),
               (0.0036, 0.0088), (0, 0.006)], seg=8)
    return s


def tea_handle():
    sec = circ(0.002, 5, 1.0, 0.8)
    return arc_sweep(sec, 0.008, 0.0, math.pi, 8, rfn=lambda t: 1.0 + 0.25 * math.sin(math.pi * t))


def tea_lid():
    # plug (hidden in the pot), flanged lid, dome and a ball knob
    return lathe([(0, -0.006), (0.0045, -0.006), (0.0062, -0.0015), (0.006, 0.0005), (0.0032, 0.003),
                  (0.0017, 0.0038), (0.0027, 0.0053), (0, 0.006)], seg=8)


def tea_cup():
    # a little footed cup with a rolled rim and a loop handle on one side
    cup = vessel([(0.0062, -0.0075), (0.0072, -0.0068), (0.0086, -0.0045), (0.0101, 0.001),
                  (0.0108, 0.0075)], 0.0009, seg=12)
    h = tube([(0.0092, 0.003, 0), (0.0122, 0.0025, 0), (0.0125, -0.0015, 0), (0.0088, -0.003, 0)],
             0.0011, seg=4, smooth=1)
    return fitf(cup + h, f_cyl(0.011, 0.008, 0.015))


def tea_saucer():
    return lathe([(0, -0.0006), (0.0088, -0.0006), (0.0093, -0.001), (0.0108, -0.001), (0.0175, 0.0006),
                  (0.018, 0.001), (0.0168, 0.001), (0.0105, 0.0), (0.0, -0.0002)], seg=14)


part('ChildrensTeaSets__pot', f_sphere(0.024), tea_pot)
part('ChildrensTeaSets__spout', f_cyl(0.005, 0.003, 0.020), tea_spout)
part('ChildrensTeaSets__handle', half_torus_frame(0.008, 0.002), tea_handle)
part('ChildrensTeaSets__lid', f_sphere(0.006), tea_lid)
part('ChildrensTeaSets__cup', f_cyl(0.011, 0.008, 0.015), tea_cup)
part('ChildrensTeaSets__saucer', f_cyl(0.018, 0.018, 0.002), tea_saucer)


# ------------------------------------------------------ deflated balls --
def deflated_ball():
    # a ball gone soft: flat contact patch, sides slumping out low down, a
    # flattish top (where the placer's decals sit) and the side wall buckled
    # in at two places with creases running round
    rx, ry = 0.075, 0.04125
    prof = []
    N = 11
    for k in range(N + 1):
        th = -HPI + math.pi * k / N
        c, s_ = math.cos(th), math.sin(th)
        m = 3.2 if s_ > 0 else 5.0
        r = rx * abs(c) ** (2 / 2.2) * (1.0 + 0.05 * max(0.0, -s_) * (1 - abs(s_)) * 4)
        y = ry * math.copysign(abs(s_) ** (2 / m), s_)
        prof.append((r, y))
    prof[0] = (0, prof[0][1])
    prof[-1] = (0, prof[-1][1])
    s = lathe(prof, seg=24)
    dents = [(Vector((1.0, 0.55, 0.1)).normalized(), 0.034, 0.72), (Vector((-0.55, 0.3, -0.85)).normalized(), 0.018, 0.5)]

    def w(p):
        d = Vector((p.x / rx, p.y / ry, p.z / rx))
        if d.length < 1e-9:
            return p
        d.normalize()
        q = p.copy()
        for c, depth, width in dents:
            g = gauss(d, c, width)
            q -= Vector((d.x, d.y * 0.35, d.z)) * depth * g
        az = math.atan2(p.z, p.x)
        q.y += 0.0014 * math.sin(az * 5 + 0.7) * max(0.0, d.y)
        return q + propkit.noise3(p * 30.0) * 0.001
    return fitf(s.warp(w), f_sphere(0.075))


part('DeflatedBalls__body', f_sphere(0.075), deflated_ball, smooth=180)


# -------------------------------------------------------- rubber ducks --
def duck_body():
    # lofted along X at its scaled size (1.2, 0.9, 1): flat bottom, full
    # chest at +X, the back sweeping up into a pointed tail at -X, and a
    # folded wing raised on each side
    xs = [-0.036, -0.033, -0.027, -0.018, -0.008, 0.004, 0.016, 0.026, 0.032, 0.036]
    top = [0.027, 0.023, 0.017, 0.0145, 0.0145, 0.0155, 0.0155, 0.012, 0.006, 0.0]
    bot = [0.019, 0.008, -0.012, -0.023, -0.027, -0.027, -0.026, -0.021, -0.013, -0.005]
    hws = [0.0012, 0.007, 0.0165, 0.0245, 0.029, 0.030, 0.029, 0.025, 0.018, 0.006]
    rings = []
    for x, t, b_, hw in zip(xs, top, bot, hws):
        yc, hh = (t + b_) / 2, (t - b_) / 2
        ring = []
        for k in range(12):
            ph = TAU * k / 12
            c, s_ = math.cos(ph), math.sin(ph)
            e = 0.45 if s_ < 0 else 0.9
            z = hw * math.copysign(abs(c) ** 0.8, c)
            y = yc + hh * math.copysign(abs(s_) ** e, s_)
            wing = math.exp(-((x + 0.006) / 0.014) ** 2 - ((y - 0.004) / 0.009) ** 2)
            z += math.copysign(0.0028 * wing, z) if abs(z) > 1e-6 else 0.0
            ring.append((x, y, z))
        rings.append(ring)
    return fitf(loft(rings), f_sphere(0.030))


def duck_head():
    # round head, puffed cheeks low at the front, crown slightly domed
    def rad(d):
        r = 0.020
        r += 0.0016 * (gauss(d, (0.5, -0.35, 0.8), 0.55) + gauss(d, (0.5, -0.35, -0.8), 0.55))
        r -= 0.0012 * gauss(d, (1, 0.1, 0), 0.5)
        return r
    return fitf(rsphere(rad, 16, 9), f_sphere(0.020))


def duck_bill():
    # a flat spatula bill, rounded at the tip, with the mouth crease round it
    def out(i):
        hb, hw, cx = 0.0034 - i, 0.005 - i, 0.002
        pts = [(-0.007, -hb)]
        pts += [(cx + hw * math.cos(a), hw * math.sin(a)) for a in [-HPI + math.pi * k / 6 for k in range(7)]]
        pts.append((-0.007, hb))
        return pts
    s = ystack(out, [(0.0007, -0.003), (0.0, -0.0021), (0.0005, -0.0004), (0.0, 0.0012), (0.0009, 0.003)])
    return fitb(s, 0.014, 0.006, 0.010)


part('RubberDucks__body', f_sphere(0.030), duck_body, smooth=180)
part('RubberDucks__head', f_sphere(0.020), duck_head, smooth=180)
part('RubberDucks__bill', f_box(0.014, 0.006, 0.010), duck_bill, smooth=60)


# ------------------------------------------------------------ gameboys --
# lying face up; the screen end is -Z, the speaker corner is +X +Z (the
# big rounded corner of the original handheld)
def gb_body():
    out = lambda i: rr4(0.060 - 2 * i, 0.090 - 2 * i, [0.013 - i, 0.004 - i, 0.004 - i, 0.004 - i], seg=3)
    s = ystack(out, [(0.0012, -0.007), (0.0, -0.0058), (0.0, 0.0055), (0.0015, 0.007)])
    slots = []
    for k in range(6):                         # slanted speaker grille
        t = (k - 2.5) * 0.0036
        c = box(0.0015, 0.004, 0.015).rot(y=math.pi / 4).move(x=0.019 + t * 0.707, y=0.0075, z=0.034 - t * 0.707)
        slots.append(c)
    return cut(s, *slots)


def gb_bezel():
    out = lambda i: rr4(0.048 - 2 * i, 0.038 - 2 * i, [0.008 - i, 0.002 - i, 0.002 - i, 0.002 - i], seg=2)
    return ystack(out, [(0.0, -0.001), (0.0, 0.0003), (0.0006, 0.001)])


def gb_screen():
    return ystack(rrect_out(0.036, 0.028, 0.0015, seg=1), [(0.0, -0.0005), (0.0004, 0.0005)])


def gb_dpad(w, d):
    return ystack(rrect_out(w, d, 0.0008, seg=1), [(0.0, -0.0015), (0.0, 0.0007), (0.0006, 0.0015)])


part('Gameboys__body', f_box(0.060, 0.014, 0.090), gb_body)
part('Gameboys__bezel', f_box(0.048, 0.002, 0.038), gb_bezel)
part('Gameboys__screen', f_box(0.036, 0.001, 0.028), gb_screen)
part('Gameboys__dpadH', f_box(0.014, 0.003, 0.005), lambda: gb_dpad(0.014, 0.005))
part('Gameboys__dpadV', f_box(0.005, 0.003, 0.014), lambda: gb_dpad(0.005, 0.014))
part('Gameboys__button', f_cyl(0.005, 0.005, 0.003),
     lambda: lathe([(0, -0.0015), (0.005, -0.0015), (0.005, 0.0), (0.0042, 0.0011), (0.0022, 0.0015), (0, 0.0015)], seg=10))
part('Gameboys__pill', f_cyl(0.002, 0.002, 0.008), lambda: capsule(0.002, 0.004, seg=8, cap_rings=2), smooth=60)


# --------------------------------------------------------------- yoyos --
# each half is used both ways round, so it is symmetric: rounded rim, a
# shallow face with a recessed centre cap on both sides
def yoyo_half():
    return lathe([(0, -0.0034), (0.013, -0.0034), (0.0145, -0.004), (0.024, -0.0036), (0.028, -0.0012),
                  (0.028, 0.0012), (0.024, 0.0036), (0.0145, 0.004), (0.013, 0.0034), (0, 0.0034)], seg=16)


def yoyo_axle():
    # hub with the string wound round its waist
    return lathe([(0, -0.004), (0.0056, -0.004), (0.006, -0.003), (0.0054, 0.0), (0.006, 0.003),
                  (0.0056, 0.004), (0, 0.004)], seg=10)


part('Yoyos__half', f_cyl(0.028, 0.028, 0.008), yoyo_half, smooth=50)
part('Yoyos__axle', f_cyl(0.006, 0.006, 0.008), yoyo_axle, smooth=50)


# --------------------------------------------------------- rubiks cubes --
def rubik_core():
    # rounded black core; the stickers (decals) cover the faces, so the
    # turning grooves between them stay a colour gap, not geometry
    return rbox(0.055, 0.055, 0.055, r=0.0032, seg=2)


part('RubiksCubes__core', f_box(0.055, 0.055, 0.055), rubik_core)


# --------------------------------------------------------- lego bricks --
def lego_body():
    # crisp moulded brick with hairline-eased edges, hollow underneath
    # (typical 2x2 size; the placer stretches it for the longer bricks)
    s = rbox(0.020, 0.010, 0.040, r=0.0004, seg=1)
    return cut(s, box(0.0176, 0.0095, 0.0376).move(y=-0.00625))


part('LegoBricks__body', f_box(0.020, 0.010, 0.040), lego_body)
part('LegoBricks__stud', f_cyl(0.004, 0.004, 0.003),
     lambda: lathe([(0, -0.0015), (0.004, -0.0015), (0.004, 0.0009), (0.0037, 0.0014), (0.0032, 0.0015), (0, 0.0015)], seg=10))


# ---------------------------------------------------------- jump ropes --
def rope_loop():
    # a loop of plastic cord lying in a coil: not quite round, rising and
    # falling a little as it crosses itself
    R, r = 0.046, 0.004
    sec = circ(r, 7)
    n = 36
    v, f = [], []
    for i in range(n):
        u = TAU * i / n
        Ru = R * (1 + 0.035 * math.sin(2 * u + 1.0) + 0.02 * math.sin(3 * u + 0.3))
        lift = 0.0016 * math.sin(2 * u + 0.4)
        c, s_ = math.cos(u), math.sin(u)
        v += [((Ru + dr) * c, (Ru + dr) * s_, dz + lift) for dr, dz in sec]
    m = len(sec)
    for i in range(n):
        i2 = (i + 1) % n
        for j in range(m):
            jj = (j + 1) % m
            f.append((i * m + j, i * m + jj, i2 * m + jj, i2 * m + j))
    return fitf(fixw(Shape(v, f)), f_torus(0.046, 0.004))


def rope_handle():
    # turned wooden grip, the same at both ends (the placer points them
    # opposite ways): rounded ends, a bead ring near each end, fat middle
    return lathe([(0, -0.015), (0.0035, -0.015), (0.0048, -0.0138), (0.0052, -0.012), (0.0045, -0.0105),
                  (0.0058, -0.006), (0.006, 0.0), (0.0058, 0.006), (0.0045, 0.0105), (0.0052, 0.012),
                  (0.0048, 0.0138), (0.0035, 0.015), (0, 0.015)], seg=10)


part('JumpRopes__loop', f_torus(0.046, 0.004), rope_loop, smooth=70)
part('JumpRopes__handle', f_cyl(0.006, 0.006, 0.030), rope_handle, smooth=50)


# --------------------------------------------------------- teddy bears --
# plush: the face is at -Z (the placer puts eyes and nose there)
def teddy_body():
    # sitting pear: flat seat, round belly pushing forward, narrower
    # shoulders; a stitched seam down the front and back
    s = lathe([(0, -0.0405), (0.026, -0.0405), (0.036, -0.037), (0.043, -0.028), (0.045, -0.016),
               (0.0445, -0.002), (0.041, 0.012), (0.034, 0.025), (0.024, 0.0345), (0.012, 0.0395),
               (0, 0.0405)], seg=20)

    def w(p):
        d = p.normalized()
        q = p + d * 0.003 * gauss(d, (0, -0.2, -1), 0.6)
        if abs(p.x) < 1e-5:
            q = q - Vector((0, 0, math.copysign(0.0009, p.z)))
        return q
    return fitf(s.warp(w), f_sphere(0.045))


def teddy_head():
    # round head with the face flattened a little, a muzzle and cheeks
    def rad(d):
        face = max(0.0, -d.z)
        r = 0.035 * (1 - 0.14 * face * face)
        r += 0.0068 * gauss(d, (0, -0.35, -1), 0.36)
        r += 0.0015 * (gauss(d, (0.75, -0.3, -0.55), 0.5) + gauss(d, (-0.75, -0.3, -0.55), 0.5))
        return r
    return fitf(rsphere(rad, 20, 12), f_sphere(0.035))


def teddy_ear():
    # round ear with a cupped front (-Z); its lower half is a root that
    # sits inside the head
    disc = lathe([(0, -0.0015), (0.006, -0.0022), (0.0085, -0.003), (0.0098, -0.0015), (0.01, 0.0005),
                  (0.0092, 0.0025), (0.006, 0.0032), (0, 0.0035)], seg=12).rot(x=HPI)
    root = ellipsoid(0.008, 0.005, 0.01, seg=8).move(y=-0.005)
    return fitf(disc + root, f_sphere(0.010))


def teddy_arm():
    return fitf(capsule(0.0155, 0.028, seg=12, cap_rings=3).rot(z=HPI), f_sphere(0.020))


def teddy_leg():
    # stubby leg sticking out along +Z, ending in a flat round sole
    prof = [(0, -1.0), (0.6, -0.945), (0.85, -0.77), (0.97, -0.42), (1.0, 0.0), (0.98, 0.49), (0.93, 0.77),
            (0.86, 0.9), (0.8, 0.93), (0.7, 0.99), (0, 1.0)]
    s = lathe(prof, seg=12).rot(x=HPI)
    return fitf(s, f_sphere(0.022))


def teddy_nose():
    # stitched nose: a rounded triangle, point down, domed forward (-Z)
    tri = round_outline([(-0.005, 0.0035), (0.005, 0.0035), (0.0, -0.005)], 0.0022, seg=2)
    rings = [[(x * k, y * k, z) for x, y in tri] for k, z in ((0.8, 0.005), (1.0, -0.001), (0.8, -0.0038), (0.35, -0.005))]
    return fitf(loft(rings), f_sphere(0.005))


part('TeddyBears__body', f_sphere(0.045), teddy_body, smooth=180)
part('TeddyBears__head', f_sphere(0.035), teddy_head, smooth=180)
part('TeddyBears__ear', f_sphere(0.010), teddy_ear, smooth=180)
part('TeddyBears__arm', f_sphere(0.020), teddy_arm, smooth=180)
part('TeddyBears__leg', f_sphere(0.022), teddy_leg, smooth=180)
part('TeddyBears__eye', f_sphere(0.003), lambda: sphere(0.003, 10, 6), smooth=180)
part('TeddyBears__nose', f_sphere(0.005), teddy_nose, smooth=70)


# ------------------------------------------------------------- crayons --
def crayon_wrapper():
    # paper sleeve: slightly lifted edges at both ends and the glued
    # overlap running down one side
    s = lathe([(0, -0.0275), (0.0036, -0.0275), (0.004, -0.0271), (0.0039, -0.024), (0.0038, 0.0),
               (0.0039, 0.024), (0.004, 0.0271), (0.0036, 0.0275), (0, 0.0275)], seg=12)

    def w(p):
        a = math.atan2(p.x, p.z)
        if math.hypot(p.x, p.z) > 0.003 and -0.3 < a < 0.3:
            k = 1.04
            return Vector((p.x * k, p.y, p.z * k))
        return p
    return fitf(s.warp(w), f_cyl(0.004, 0.004, 0.055))


def crayon_tip():
    # sharpened cone above a short round collar, the point worn to a slant
    s = lathe([(0, -0.005), (0.004, -0.005), (0.004, -0.0028), (0.0021, 0.0024), (0.001, 0.005), (0, 0.005)], seg=10)

    def w(p):
        if p.y > 0.0:
            p.y -= 0.0011 * (p.y / 0.005) * (0.5 + p.x / 0.004)
        return p
    return fitf(s.warp(w), f_cone(0.004, 0.010))


part('Crayons__wrapper', f_cyl(0.004, 0.004, 0.055), crayon_wrapper)
part('Crayons__tip', f_cone(0.004, 0.010), crayon_tip)
part('Crayons__back', f_cyl(0.004, 0.004, 0.008),
     lambda: lathe([(0, -0.004), (0.004, -0.004), (0.004, 0.0031), (0.0033, 0.004), (0, 0.004)], seg=10))


# ------------------------------------------------------ action figures --
def figure_torso():
    # moulded torso: hips, belt-line waist, chest, square shoulders, neck post
    secs = [(-0.016, 0.0105, 0.0060), (-0.012, 0.0110, 0.0063), (-0.0075, 0.0097, 0.0057),
            (-0.004, 0.0102, 0.0064), (0.004, 0.0115, 0.0070), (0.010, 0.0120, 0.0068),
            (0.0142, 0.0108, 0.0056), (0.016, 0.0048, 0.0040)]
    rings = []
    for y, hw, hd in secs:
        r = min(hw, hd) * 0.55
        rings.append([(x, y, z) for x, z in rr4(2 * hw, 2 * hd, [r] * 4, seg=2)])
    return fitb(loft(rings), 0.024, 0.032, 0.014)


def figure_head():
    # head on a short neck peg: jaw, cheekbones, round crown
    s = lathe([(0, -0.011), (0.0035, -0.011), (0.0036, -0.0085), (0.0075, -0.0075), (0.0098, -0.004),
               (0.011, 0.001), (0.0105, 0.005), (0.008, 0.0088), (0.0045, 0.0105), (0, 0.011)], seg=12)
    return fitf(s, f_sphere(0.011))


part('ActionFigures__torso', f_box(0.024, 0.032, 0.014), figure_torso)
part('ActionFigures__head', f_sphere(0.011), figure_head, smooth=60)
part('ActionFigures__arm', f_box(0.006, 0.024, 0.006),
     lambda: fitb(lathe([(0, -0.012), (0.0027, -0.0112), (0.0021, -0.0075), (0.0024, -0.001), (0.0028, 0.007),
                         (0.0024, 0.011), (0, 0.012)], seg=8), 0.006, 0.024, 0.006), smooth=60)
part('ActionFigures__leg', f_box(0.008, 0.024, 0.008),
     lambda: fitb(lathe([(0, -0.012), (0.004, -0.0115), (0.0037, -0.0055), (0.003, -0.0035), (0.0037, 0.007),
                         (0.0034, 0.0115), (0, 0.012)], seg=8), 0.008, 0.024, 0.008), smooth=60)


# -------------------------------------------------------- baby rattles --
def rattle_ball():
    # five soft lobes like a little pumpkin
    def rad(d):
        a = math.atan2(d.x, d.z)
        side = math.sqrt(max(0.0, 1 - d.y * d.y))
        return 0.020 * (1 - 0.13 * (1 - abs(math.cos(2.5 * a))) * side)
    return fitf(rsphere(rad, 20, 8), f_sphere(0.020))


def rattle_handle():
    # flared ends and three grip rings in the middle
    return lathe([(0, -0.0275), (0.0042, -0.0275), (0.005, -0.026), (0.0036, -0.021), (0.0036, -0.008),
                  (0.0044, -0.0065), (0.0036, -0.005), (0.0044, -0.0035), (0.0036, -0.002), (0.0044, -0.0005),
                  (0.0036, 0.001), (0.0036, 0.021), (0.005, 0.026), (0.0042, 0.0275), (0, 0.0275)], seg=8)


def rattle_ring():
    # teether ring with eight soft bumps
    return fitf(arc_sweep(circ(0.004, 5), 0.014, 0.0, TAU, 24,
                          rfn=lambda t: 0.85 + 0.15 * math.cos(8 * TAU * t)), f_torus(0.014, 0.004))


part('BabyRattles__ball', f_sphere(0.020), rattle_ball, smooth=70)
part('BabyRattles__handle', f_cyl(0.005, 0.005, 0.055), rattle_handle, smooth=50)
part('BabyRattles__ring', f_torus(0.014, 0.004), rattle_ring, smooth=80)


# -------------------------------------------------------- toy dinosaurs --
# moulded vinyl: the head is at -X, the tail runs out along +X
def dino_body():
    # at its scaled size (1.8, 0.9, 1): belly, rounded back, rising into
    # the neck at the front, narrowing into the tail root at the back
    xs = [-0.036, -0.032, -0.024, -0.012, 0.0, 0.012, 0.024, 0.032, 0.036]
    top = [0.016, 0.0175, 0.017, 0.0165, 0.018, 0.0175, 0.013, 0.008, 0.004]
    bot = [0.001, -0.006, -0.013, -0.017, -0.018, -0.017, -0.013, -0.007, -0.004]
    hws = [0.005, 0.009, 0.014, 0.018, 0.020, 0.019, 0.015, 0.010, 0.005]
    rings = []
    for x, t, b_, hw in zip(xs, top, bot, hws):
        yc, hh = (t + b_) / 2, (t - b_) / 2
        rings.append([(x, yc + hh * math.sin(TAU * k / 12), hw * math.cos(TAU * k / 12)) for k in range(12)])
    return fitf(loft(rings), f_sphere(0.020))


def dino_head():
    # at its scaled size (1.3, 1, 1): snout at -X, brow ridges, a mouth line
    xs = [0.0156, 0.012, 0.004, -0.004, -0.010, -0.0145, -0.0156]
    top = [0.006, 0.010, 0.012, 0.009, 0.006, 0.003, 0.001]
    bot = [-0.004, -0.010, -0.012, -0.010, -0.008, -0.006, -0.003]
    hws = [0.005, 0.010, 0.012, 0.010, 0.0085, 0.0065, 0.004]
    rings = []
    for i, (x, t, b_, hw) in enumerate(zip(xs, top, bot, hws)):
        yc, hh = (t + b_) / 2, (t - b_) / 2
        ring = []
        for k in range(12):
            a = TAU * k / 12
            y, z = yc + hh * math.sin(a), hw * math.cos(a)
            if k in (0, 6) and 2 <= i <= 5:
                z *= 0.86                              # mouth line
            if k in (2, 4) and i == 2:
                y += 0.0012                            # brow ridges
            ring.append((x, y, z))
        rings.append(ring)
    return fitf(loft(rings), f_sphere(0.012))


def dino_tail():
    # tapering tail curling up a little at the tip (local -X is up)
    rings = []
    n = 8
    for i in range(n):
        t = i / (n - 1)
        y = -0.0225 + 0.045 * t
        r = 0.010 * (1 - t) ** 0.85 + 0.0008
        ox = -0.003 * t * t
        rings.append([(ox + r * math.cos(TAU * k / 10), y, r * 0.92 * math.sin(TAU * k / 10)) for k in range(10)])
    return fitf(loft(rings), f_cone(0.010, 0.045))


def dino_leg():
    s = lathe([(0, -0.007), (0.0056, -0.007), (0.006, -0.0055), (0.0045, -0.002), (0.005, 0.004),
               (0.0045, 0.007), (0, 0.007)], seg=10)

    def w(p):                                   # the foot reaches forward (-X)
        if p.y < -0.004:
            p.x -= 0.0012
        return p
    return fitf(s.warp(w), f_cyl(0.006, 0.006, 0.014))


def dino_plate():
    # a back plate: thick at the root, thinning to a rounded blade edge
    secs = [(-0.005, 0.005, 0.005), (-0.001, 0.0049, 0.0033), (0.002, 0.0037, 0.0019), (0.004, 0.0019, 0.0011)]
    rings = [[(hx * math.cos(TAU * k / 8), y, hz * math.sin(TAU * k / 8)) for k in range(8)] for y, hx, hz in secs]
    rings.append([(0.0, 0.005, 0.0)] * 8)
    return fitf(weld(loft(rings)), f_cone(0.005, 0.010))


part('ToyDinosaurs__body', f_sphere(0.020), dino_body, smooth=180)
part('ToyDinosaurs__head', f_sphere(0.012), dino_head, smooth=70)
part('ToyDinosaurs__tail', f_cone(0.010, 0.045), dino_tail, smooth=180)
part('ToyDinosaurs__leg', f_cyl(0.006, 0.006, 0.014), dino_leg, smooth=60)
part('ToyDinosaurs__plate', f_cone(0.005, 0.010), dino_plate, smooth=60)


# ----------------------------------------------------------- pacifiers --
# the shield stands on edge: its local +Y faces the ring, -Y the teat
def paci_shield():
    # notched round shield, curved toward the teat side, two vent holes
    R, n = 0.018, 16

    def rim(a):
        return R * (1 - 0.1 * math.exp(-((a - HPI) / 0.3) ** 2) - 0.1 * math.exp(-((a + HPI) / 0.3) ** 2)
                    - 0.1 * math.exp(-((a - 1.5 * math.pi) / 0.3) ** 2))
    outline = [(rim(TAU * k / n) * math.cos(TAU * k / n), rim(TAU * k / n) * math.sin(TAU * k / n)) for k in range(n)]
    rings = []
    for k, t in ((0.45, 1), (0.85, 1), (1.0, 0.6), (1.0, -0.6), (0.85, -1), (0.45, -1)):
        rings.append([(x * k, t * 0.0011 - 0.0013 * k * k, z * k) for x, z in outline])
    s = loft(rings)
    vents = [cyl(0.0022, 0.0022, 0.01, seg=8, bevel_r=0).scale(1, 1, 1.4).move(x=sx * 0.0105) for sx in (-1, 1)]
    return fitf(cut(s, *vents), f_cyl(0.018, 0.018, 0.004))


def paci_teat():
    # at its scaled size (1, 1, 1.6): flange at the shield (+Z), thin
    # neck, rounded bulb pointing away (-Z)
    s = lathe([(0, -0.0096), (0.006, -0.0096), (0.0052, -0.0078), (0.0031, -0.0035), (0.0055, 0.003),
               (0.0042, 0.0078), (0, 0.0096)], seg=8).rot(x=-HPI)
    return fitf(s, f_sphere(0.006))


part('Pacifiers__shield', f_cyl(0.018, 0.018, 0.004), paci_shield)
part('Pacifiers__teat', f_sphere(0.006), paci_teat, smooth=70)
part('Pacifiers__ring', f_torus(0.010, 0.003), lambda: torus(0.010, 0.003, seg=20, tseg=6), smooth=70)


# ---------------------------------------------------------- sippy cups --
def sippy_body():
    # moulded cup: rounded foot, a grip waist, flaring to the thread neck
    return lathe([(0, -0.035), (0.0238, -0.035), (0.026, -0.032), (0.0265, -0.016), (0.0258, -0.004),
                  (0.029, 0.022), (0.030, 0.031), (0.0282, 0.035), (0, 0.035)], seg=16)


def sippy_lid():
    # screw-on lid: ribbed skirt, rounded shoulder, slightly domed top
    s = lathe([(0, -0.005), (0.0305, -0.005), (0.032, -0.0035), (0.032, 0.002), (0.0305, 0.0038),
               (0.022, 0.0049), (0, 0.005)], seg=20)
    k = [0]

    def w(p):
        r = math.hypot(p.x, p.z)
        if r > 0.0318 and -0.0036 < p.y < 0.0022:
            a = math.atan2(p.x, p.z)
            i = round(a / (TAU / 20))
            if i % 2:
                return Vector((p.x * 0.955, p.y, p.z * 0.955))
        return p
    return s.warp(w)


def sippy_spout():
    # soft spout: round where it plugs into the lid, pressed flat to an oval
    # mouth with a slit
    rings = []
    for y, hx, hz in ((-0.010, 0.008, 0.008), (-0.006, 0.0068, 0.0074), (0.0, 0.0048, 0.0066),
                      (0.006, 0.0036, 0.006), (0.0092, 0.0032, 0.0056), (0.010, 0.0015, 0.0036)):
        rings.append([(hx * math.cos(TAU * k / 12), y, hz * math.sin(TAU * k / 12)) for k in range(12)])
    s = loft(rings)
    return fitf(cut(s, box(0.0012, 0.003, 0.006).move(y=0.0105)), f_cyl(0.006, 0.008, 0.020))


def sippy_handle():
    # moulded handle: oval grip swelling out of two mounting pads
    sec = circ(0.005, 6, 0.85, 1.0)
    s = arc_sweep(sec, 0.014, 0.0, math.pi, 12, rfn=lambda t: 1.0 + 0.35 * max(0.0, 1 - 6 * min(t, 1 - t)))
    return fitf(s, half_torus_frame(0.014, 0.005))


part('SippyCups__body', f_cyl(0.030, 0.026, 0.070), sippy_body, smooth=50)
part('SippyCups__lid', f_cyl(0.032, 0.032, 0.010), sippy_lid, smooth=50)
part('SippyCups__spout', f_cyl(0.006, 0.008, 0.020), sippy_spout, smooth=60)
part('SippyCups__handle', half_torus_frame(0.014, 0.005), sippy_handle, smooth=70)


# --------------------------------------------------------- baby bottles --
def bottle_milk():
    # milk column (seen through the bottle): a meniscus climbing the wall
    return lathe([(0, -0.0225), (0.020, -0.0225), (0.021, -0.0212), (0.0172, 0.0212), (0.0168, 0.0225),
                  (0.012, 0.0214), (0, 0.021)], seg=16)


def bottle_collar():
    # screw collar: rounded ring, a grip band, the lip turning in over the
    # teat's flange
    return ring_lathe([(0.0195, -0.005), (0.022, -0.0042), (0.0214, -0.0008), (0.022, 0.0032), (0.0203, 0.0049),
                       (0.0092, 0.0044), (0.0088, 0.0029), (0.0195, 0.002)], seg=16)


def bottle_teat():
    # at its scaled size (1, 1.4, 1): flange, shoulder, neck and teat
    return fitf(lathe([(0, -0.014), (0.010, -0.014), (0.0098, -0.0115), (0.0072, -0.0065), (0.0044, 0.0005),
                       (0.0046, 0.0065), (0.0036, 0.0118), (0.0014, 0.0138), (0, 0.014)], seg=12), f_sphere(0.010))


part('BabyBottles__milk', f_cyl(0.017, 0.021, 0.045), bottle_milk, smooth=60)
part('BabyBottles__collar', f_cyl(0.022, 0.022, 0.010), bottle_collar, smooth=50)
part('BabyBottles__teat', f_sphere(0.010), bottle_teat, smooth=70)


# ------------------------------------------------------ stuffed rabbits --
# plush, face at -Z; the placer puts the feet behind the body next to the
# tail, so they are modelled as crouching hind feet (toes forward, heels
# and haunches sticking out at the back)
def rabbit_body():
    # at its scaled size (0.9, 1, 0.9): upright pear with a round belly
    s = lathe([(0, -0.035), (0.018, -0.035), (0.027, -0.031), (0.0315, -0.02), (0.031, -0.005),
               (0.0275, 0.01), (0.021, 0.024), (0.012, 0.032), (0, 0.035)], seg=20)
    s.warp(lambda p: p + p.normalized() * 0.0025 * gauss(p.normalized(), (0, -0.3, -1), 0.6))
    return fitf(s, f_sphere(0.035))


def rabbit_head():
    # round head with puffed cheeks and a small muzzle under the nose
    def rad(d):
        r = 0.028
        r += 0.0028 * (gauss(d, (0.55, -0.4, -0.72), 0.45) + gauss(d, (-0.55, -0.4, -0.72), 0.45))
        r += 0.0019 * gauss(d, (0, -0.15, -1), 0.3)
        return r
    return fitf(rsphere(rad, 16, 9), f_sphere(0.028))


def rabbit_ear():
    # at its scaled size (0.7, 2.5, 0.5): long leaf-shaped ear, narrow at
    # the root, scooped out in front (-Z)
    w0, d0, H = 0.0056, 0.004, 0.02
    rings = []
    for t in (0.0, 0.3, 0.58, 0.8, 0.93, 0.985):
        y = -H + 2 * H * t
        if t < 0.45:
            w = w0 * (0.62 + 0.38 * math.sin(HPI * t / 0.45))
        else:
            w = w0 * math.sqrt(max(0.0, 1 - ((t - 0.45) / 0.56) ** 2))
        cup = math.sin(math.pi * min(1.0, t * 1.1)) * 0.9
        ring = []
        for k in range(8):
            a = TAU * k / 8
            c, s_ = math.cos(a), math.sin(a)
            z = d0 * s_ if s_ >= 0 else -d0 * (0.55 - cup * (1 - c * c))
            ring.append((w * c, y, z))
        rings.append(ring)
    s = loft(rings, caps=False)
    n = len(s.v)
    s.v.append(Vector((0.0, H, 0.0005)))
    for k in range(8):
        s.f.append((n - 9 + k, n - 9 + (k + 1) % 8, n))
    s.f.append(tuple(reversed(range(8))))
    return fitf(fixw(s), f_sphere(0.008))


def rabbit_foot():
    # at its scaled size (1, 0.5, 1.4): flat sole, toes at -Z, a raised
    # rounded heel and haunch at +Z
    zs = [-0.0252, -0.022, -0.014, -0.004, 0.008, 0.018, 0.0235, 0.0252]
    hws = [0.006, 0.0115, 0.0155, 0.017, 0.018, 0.0175, 0.014, 0.008]
    tops = [-0.004, 0.0005, 0.003, 0.004, 0.006, 0.0085, 0.009, 0.006]
    rings = []
    for z, hw, t in zip(zs, hws, tops):
        b_ = -0.009 if abs(z) < 0.024 else -0.006
        yc, hh = (t + b_) / 2, (t - b_) / 2
        ring = []
        for k in range(10):
            a = TAU * k / 10
            s_ = math.sin(a)
            y = yc + hh * math.copysign(abs(s_) ** (0.6 if s_ > 0 else 0.35), s_)
            ring.append((hw * math.cos(a), y, z))
        rings.append(ring)
    return fitf(loft(rings), f_sphere(0.018))


def rabbit_tail():
    # fluffy pompom
    return fitf(rsphere(lambda d: 0.008 * (1 + 0.16 * noise.noise(d * 2.6 + Vector((3.1, 1.7, 0.4))))
                        , 10, 6), f_sphere(0.008))


part('StuffedRabbits__body', f_sphere(0.035), rabbit_body, smooth=180)
part('StuffedRabbits__head', f_sphere(0.028), rabbit_head, smooth=180)
part('StuffedRabbits__ear', f_sphere(0.008), rabbit_ear, smooth=180)
part('StuffedRabbits__foot', f_sphere(0.018), rabbit_foot, smooth=180)
part('StuffedRabbits__tail', f_sphere(0.008), rabbit_tail, smooth=180)


# ---------------------------------------------------- stuffed elephants --
# plush, the head and trunk at -X
def eleph_body():
    # at its scaled size (1.6, 1, 1): rounded barrel, a fuller rump, and a
    # short rope tail hanging at the back (+X)
    def rad(d):
        r = 1.0 + 0.05 * gauss(d, (1, 0.3, 0), 0.7) - 0.03 * gauss(d, (0, -1, 0), 0.6)
        return r
    s = rsphere(rad, 20, 12).scale(0.072, 0.045, 0.045)
    tail = tube([(0.066, 0.006, 0), (0.071, 0.0, 0), (0.0735, -0.01, 0), (0.0738, -0.017, 0)], 0.0022,
                seg=6, smooth=1, r_end=0.0032)
    return fitf(s + tail, f_sphere(0.045))


def eleph_head():
    # domed brow over the trunk root, round cheeks
    def rad(d):
        r = 0.028
        r += 0.0022 * (gauss(d, (-0.7, 0.5, 0.35), 0.45) + gauss(d, (-0.7, 0.5, -0.35), 0.45))
        r += 0.003 * gauss(d, (-1, -0.55, 0), 0.5)
        r += 0.0015 * (gauss(d, (-0.3, -0.3, 1), 0.6) + gauss(d, (-0.3, -0.3, -1), 0.6))
        return r
    return fitf(rsphere(rad, 16, 9), f_sphere(0.030))


def eleph_trunk():
    # one ring of the trunk: soft rounded ends, two wrinkle creases
    return lathe([(0, -0.007), (0.0098, -0.007), (0.0116, -0.0058), (0.0119, -0.003), (0.0114, -0.0013),
                  (0.0127, 0.0012), (0.0134, 0.0038), (0.014, 0.0056), (0.0128, 0.0068), (0, 0.007)], seg=12)


def eleph_ear():
    # soft felt ear flap, 2.4 mm thick and centred on the disc (both faces
    # show), with a rolled edge and a thinner middle
    out = [(0.024 * math.cos(TAU * k / 20) * (1 + 0.05 * math.sin(TAU * k / 20)),
            0.024 * math.sin(TAU * k / 20)) for k in range(20)]
    rings = []
    for k, z in ((0.45, 0.0006), (0.82, 0.0009), (1.0, 0.0012), (1.0, -0.0012), (0.82, -0.0009), (0.45, -0.0006)):
        rings.append([(x * k, y * k, z) for x, y in out])
    return fitf(loft(rings), f_bounds((-0.024, -0.024, -0.0012), (0.024, 0.024, 0.0012), 'plane'))


def eleph_leg():
    return lathe([(0, -0.0125), (0.0102, -0.0125), (0.0117, -0.0115), (0.0111, -0.0098), (0.012, -0.0085),
                  (0.0116, 0.0), (0.0112, 0.0125), (0, 0.0125)], seg=14)


def eleph_tusk():
    # curving up toward the rounded tip (local +X ends up pointing up)
    rings = []
    for t in (0.0, 0.25, 0.5, 0.72, 0.9):
        y = -0.007 + 0.014 * t
        r = 0.003 * (1 - t) ** 0.8 + 0.0006
        ox = 0.0016 * t * t
        rings.append([(ox + r * math.cos(TAU * k / 8), y, r * math.sin(TAU * k / 8)) for k in range(8)])
    s = loft(rings, caps=False)
    n = len(s.v)
    s.v.append(Vector((0.0016 + 0.0002, 0.007, 0)))
    for k in range(8):
        s.f.append((n - 8 + k, n - 8 + (k + 1) % 8, n))
    s.f.append(tuple(reversed(range(8))))
    return fitf(fixw(s), f_cone(0.003, 0.014))


part('StuffedElephants__body', f_sphere(0.045), eleph_body, smooth=180)
part('StuffedElephants__head', f_sphere(0.030), eleph_head, smooth=180)
part('StuffedElephants__trunk', f_cyl(0.014, 0.0119, 0.014), eleph_trunk, smooth=70)
part('StuffedElephants__ear', f_bounds((-0.024, -0.024, 0), (0.024, 0.024, 0), 'plane'), eleph_ear, smooth=180)
part('StuffedElephants__leg', f_cyl(0.012, 0.012, 0.025), eleph_leg, smooth=60)
part('StuffedElephants__tusk', f_cone(0.003, 0.014), eleph_tusk, smooth=70)


# ------------------------------------------------------ teething rings --
def teether_loop():
    # soft silicone ring with eight gum-massage bumps
    return fitf(arc_sweep(circ(0.005, 5), 0.022, 0.0, TAU, 24,
                          rfn=lambda t: 0.84 + 0.16 * math.cos(8 * TAU * t)), f_torus(0.022, 0.005))


def teether_handle():
    # ribbed stub with a rounded end
    prof = [(0, -0.014), (0.0028, -0.014), (0.004, -0.0125)]
    for k in range(4):
        y0 = -0.009 + k * 0.0055
        prof += [(0.0034, y0 - 0.0015), (0.004, y0), (0.0034, y0 + 0.0015)]
    prof += [(0.0036, 0.011), (0.0032, 0.0134), (0.0018, 0.014), (0, 0.014)]
    return fitf(lathe(prof, seg=8), f_cyl(0.004, 0.004, 0.028))


part('TeethingRings__loop', f_torus(0.022, 0.005), teether_loop, smooth=80)
part('TeethingRings__handle', f_cyl(0.004, 0.004, 0.028), teether_handle, smooth=60)


# ----------------------------------------------------- child drawings --
def drawing_frame():
    # mitred wooden moulding round a 12 mm border: rounded outer edge, a
    # bead, a cove down to a small inner lip; the drawing covers the flat
    # panel in the middle
    W, H = 0.30, 0.24
    out = lambda i: [(W / 2 - i, -H / 2 + i), (W / 2 - i, H / 2 - i), (-W / 2 + i, H / 2 - i), (-W / 2 + i, -H / 2 + i)]
    prof = [(0.0, -0.006), (0.0, 0.0025), (0.0006, 0.0048), (0.0017, 0.006), (0.0033, 0.006), (0.0046, 0.0052),
            (0.0058, 0.0041), (0.0085, 0.0036), (0.0105, 0.0046), (0.0116, 0.0044), (0.012, 0.0028)]
    rings = [[(x, y, z) for x, y in out(i)] for i, z in prof]
    return loft(rings)


part('ChildDrawings__frame', f_box(0.30, 0.24, 0.012), drawing_frame, smooth=40)


# ----------------------------------------------------------- pet toys --
def bone_shaft():
    # rubber bone shaft: slim waist, swelling toward the knobs
    return lathe([(0, -0.07), (0.0125, -0.07), (0.014, -0.066), (0.0135, -0.05), (0.0118, -0.02), (0.0115, 0.0),
                  (0.0118, 0.02), (0.0135, 0.05), (0.014, 0.066), (0.0125, 0.07), (0, 0.07)], seg=16)


def pet_ball():
    # rubber ball: moulded parting groove round a meridian and a squeaker
    # hole at one pole
    R = 0.038
    prof = [(0, 0.0352), (0.0028, 0.0352)]
    for deg in (7, 22, 40, 58, 75, 87, 90, 93, 105, 125, 145, 162):
        ph = math.radians(deg)
        r = R - (0.0009 if deg == 90 else 0.0)
        prof.append((r * math.sin(ph), r * math.cos(ph)))
    prof.append((0, -R))
    return fitf(lathe(prof, seg=20).rot(z=HPI), f_sphere(0.038))


def pet_stripe():
    # moulded band round the ball's equator: flat, crowned outside
    sec = [(-0.001, -0.0026), (0.0022, -0.003), (0.003, -0.0012), (0.003, 0.0012), (0.0022, 0.003), (-0.001, 0.0026)]
    return fitf(arc_sweep(sec, 0.0382, 0.0, TAU, 22), f_torus(0.0382, 0.003))


part('PetToys__shaft', f_cyl(0.014, 0.014, 0.14), bone_shaft, smooth=50)
part('PetToys__knob', f_sphere(0.020), lambda: sphere(0.020, 16, 9), smooth=180)
part('PetToys__ball', f_sphere(0.038), pet_ball, smooth=60)
part('PetToys__stripe', f_torus(0.0382, 0.003), pet_stripe, smooth=50)


# ---------------------------------------------------- broken whole dolls --
# porcelain doll lying along X, the head at +X with its face turned to +X
def doll_torso():
    # at its scaled size (0.85, 1.6, 0.85) before the placer lays it down:
    # neck and shoulders at -Y (toward the head), waist, hips at +Y
    s = lathe([(0, -0.08), (0.0105, -0.08), (0.012, -0.068), (0.027, -0.058), (0.039, -0.046), (0.0425, -0.028),
               (0.039, -0.002), (0.034, 0.02), (0.0395, 0.045), (0.038, 0.064), (0.025, 0.077), (0, 0.08)], seg=16)
    return fitf(s, f_sphere(0.05))


def doll_dress():
    # gathered bodice under a waistband at the narrow top, eight soft box
    # pleats deepening toward the wavy hem at the wide bottom
    s = lathe([(0, 0.07), (0.050, 0.07), (0.055, 0.0655), (0.0535, 0.0565), (0.056, 0.035),
               (0.062, 0.0), (0.068, -0.04), (0.070, -0.062), (0.066, -0.07), (0, -0.0685)], seg=32)

    def w(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6 or p.y > 0.055:
            return p
        a = math.atan2(p.x, p.z)
        amp = 0.075 * ((0.055 - p.y) / 0.125) ** 0.8
        k = 1 + amp * (math.cos(8 * a) - 0.6)
        q = Vector((p.x * k, p.y, p.z * k))
        if p.y < -0.06:
            q.y += 0.0025 * math.cos(8 * a)
        return q
    return fitf(s.warp(w), f_cyl(0.055, 0.070, 0.14))


def doll_ruffle():
    s = ellipsoid(0.004, 0.0075, 0.0075, seg=8)
    return fitb(s.warp(lambda p: p + Vector((0.0009 * math.sin(p.z * 900), 0, 0))), 0.008, 0.015, 0.015)


def doll_head():
    # bisque head with a modelled face toward +X: brow, eye sockets, a
    # small nose, rounded cheeks, lips and chin (the sphere's pole is the
    # nose so the face gets the dense rings)
    feats = [((1, 0.3, 0), 0.0018, 0.5), ((1, 0.1, 0.42), -0.0045, 0.19), ((1, 0.1, -0.42), -0.0045, 0.19),
             ((1, -0.05, 0), 0.005, 0.14), ((0.85, -0.3, 0.45), 0.0032, 0.3), ((0.85, -0.3, -0.45), 0.0032, 0.3),
             ((1, -0.36, 0), -0.002, 0.1), ((0.8, -0.6, 0), 0.0028, 0.24), ((-0.3, 0.2, 0), 0.0015, 1.0)]

    def rad(d):
        r = 0.036
        for c, a, wd in feats:
            r += a * gauss(d, c, wd)
        return r
    s = sphere(1.0, 20, 13).rot(z=-HPI)
    s.warp(lambda p: p.normalized() * rad(p.normalized()))
    return fitf(s, f_sphere(0.038))


def doll_curl():
    # a ringlet: a strand of hair wound into a round curl
    pts = []
    n = 22
    for i in range(n):
        t = i / (n - 1)
        y = -0.0068 + 0.0136 * t
        rr_ = 0.0066 * math.sqrt(max(0.05, 1 - (y / 0.0085) ** 2))
        a = t * 2.6 * TAU
        pts.append((rr_ * math.cos(a), y, rr_ * math.sin(a)))
    return fitf(tube(pts, 0.0033, seg=5) + ellipsoid(0.0045, 0.006, 0.0045, seg=6), f_sphere(0.010))


def doll_leg():
    # porcelain leg: thigh (-Y, in the dress), knee, calf, slim ankle (+Y)
    return lathe([(0, -0.0375), (0.012, -0.0375), (0.014, -0.034), (0.0138, -0.02), (0.0118, -0.004),
                  (0.0125, 0.006), (0.0112, 0.022), (0.0085, 0.033), (0.0078, 0.0375), (0, 0.0375)], seg=14)


def doll_arm():
    # porcelain arm: rounded shoulder (-Y), elbow, forearm to the wrist (+Y)
    return lathe([(0, -0.0325), (0.008, -0.0325), (0.0098, -0.029), (0.01, -0.022), (0.0085, -0.006),
                  (0.0078, 0.002), (0.0082, 0.01), (0.0066, 0.028), (0.0056, 0.0325), (0, 0.0325)], seg=12)


def doll_hand():
    # a small closed hand: rounded fist, four knuckles, the thumb over them
    def rad(d):
        r = 0.0092
        for k in range(4):
            r += 0.0009 * gauss(d, (0.8, 0.45 - 0.3 * k, 0.4), 0.28)
        r += 0.0014 * gauss(d, (0.55, 0.05, -0.8), 0.35)
        return r
    return fitf(rsphere(rad, 12, 8), f_sphere(0.010))


def doll_shoe():
    # at its scaled size (1, 0.6, 1): a strap shoe lying flat, toe at -X,
    # heel and ankle opening at +X
    xs = [-0.010, -0.0085, -0.005, 0.0, 0.004, 0.008, 0.010]
    hws = [0.003, 0.0068, 0.0086, 0.009, 0.0088, 0.0075, 0.004]
    tops = [-0.002, 0.0015, 0.0035, 0.0048, 0.006, 0.006, 0.004]
    rings = []
    for i, (x, hw, t) in enumerate(zip(xs, hws, tops)):
        b_ = -0.006
        yc, hh = (t + b_) / 2, (t - b_) / 2
        ring = []
        for k in range(10):
            a = TAU * k / 10
            s_ = math.sin(a)
            y = yc + hh * math.copysign(abs(s_) ** (0.7 if s_ > 0 else 0.3), s_)
            if i == 3 and s_ > 0.3:
                y += 0.0007                                  # the strap
            ring.append((x, y, hw * math.cos(a)))
        rings.append(ring)
    s = loft(rings)
    return fitf(cut(s, ellipsoid(0.0045, 0.006, 0.0055, seg=10).move(x=0.0055, y=0.0072)), f_sphere(0.010))


part('BrokenWholeDolls__torso', f_sphere(0.05), doll_torso, smooth=50)
part('BrokenWholeDolls__dress', f_cyl(0.055, 0.070, 0.14), doll_dress, smooth=70)
part('BrokenWholeDolls__ruffle', f_box(0.008, 0.015, 0.015), doll_ruffle, smooth=180)
part('BrokenWholeDolls__head', f_sphere(0.038), doll_head, smooth=180)
part('BrokenWholeDolls__curl', f_sphere(0.010), doll_curl, smooth=180)
part('BrokenWholeDolls__leg', f_cyl(0.014, 0.014, 0.075), doll_leg, smooth=50)
part('BrokenWholeDolls__arm', f_cyl(0.010, 0.010, 0.065), doll_arm, smooth=50)
part('BrokenWholeDolls__hand', f_sphere(0.010), doll_hand, smooth=180)
part('BrokenWholeDolls__shoe', f_sphere(0.010), doll_shoe, smooth=60)


# -------------------------------------------------------- hobby horses --
# a stick horse lying on the floor: the head faces +X at the pole's end
def horse_pole():
    return lathe([(0, -0.4), (0.0085, -0.4), (0.01, -0.3985), (0.01, 0.3985), (0.0085, 0.4), (0, 0.4)], seg=16)


def horse_knob():
    # turned end knob: a squat ball, a bead collar and the neck toward the
    # pole (+X)
    s = lathe([(0, -0.014), (0.006, -0.0133), (0.0105, -0.0105), (0.0135, -0.005), (0.014, 0.0), (0.0128, 0.005),
               (0.0095, 0.0085), (0.0078, 0.0105), (0.0092, 0.0125), (0.0088, 0.014), (0, 0.014)], seg=14)
    return fitf(s.rot(z=-HPI), f_sphere(0.014))


def horse_head():
    # the head cut from a board in profile (nose toward +X): crest and poll
    # at the back, forehead sloping to the nose bridge, round cheek and jaw,
    # throat; edges well rounded
    out = [(-0.07, -0.04), (-0.07, 0.008), (-0.064, 0.026), (-0.05, 0.037), (-0.03, 0.04), (-0.005, 0.038),
           (0.025, 0.032), (0.05, 0.024), (0.07, 0.016), (0.07, -0.02), (0.058, -0.031), (0.035, -0.04),
           (0.012, -0.039), (-0.008, -0.033), (-0.024, -0.024), (-0.04, -0.027), (-0.055, -0.036)]
    s = prism(round_outline(out, 0.006, seg=1), 0.05, bevel_r=0.007, bevel_seg=2)
    return fitb(s, 0.14, 0.08, 0.05)


def horse_muzzle():
    # muzzle narrowing to a rounded nose, a mouth line round the front and
    # two flared nostrils
    rings = []
    for x, h, w in ((-0.025, 0.055, 0.040), (-0.002, 0.055, 0.040), (0.015, 0.054, 0.039), (0.022, 0.050, 0.036),
                    (0.025, 0.041, 0.030)):
        rings.append([(x, y, z) for y, z in rr4(h, w, [min(h, w) * 0.38] * 4, seg=2)])
    s = loft(rings)
    mouth = box(0.016, 0.0016, 0.05).move(x=0.022, y=-0.014)
    nost = [sphere(0.005, 8, 4).move(x=0.026, y=-0.005, z=sz * 0.01) for sz in (-1, 1)]
    return fitb(cut(s, mouth, *nost), 0.05, 0.055, 0.04)


def horse_ear():
    # pointed ear, cupped toward the front (+X)
    rings = []
    for t in (0.0, 0.3, 0.6, 0.85):
        y = -0.015 + 0.03 * t
        w = 0.008 * (1 - t * 0.85)
        cup = 0.7 * math.sin(math.pi * min(1.0, t + 0.15))
        ring = []
        for k in range(8):
            a = TAU * k / 8
            c, s_ = math.cos(a), math.sin(a)
            x = w * c if c <= 0 else w * (c - cup * (1 - s_ * s_))
            ring.append((x, y, w * s_))
        rings.append(ring)
    s = loft(rings, caps=False)
    n = len(s.v)
    s.v.append(Vector((0.0, 0.015, 0.0)))
    for k in range(8):
        s.f.append((n - 9 + k, n - 9 + (k + 1) % 8, n))
    s.f.append(tuple(reversed(range(8))))
    return fitf(fixw(s), f_cone(0.008, 0.030))


def horse_strand():
    # a wavy strand of yarn, slightly frayed at its free end
    pts = [(0.0006 * math.sin(i * 2.1), -0.02125 + 0.0425 * i / 3, 0.0005 * math.cos(i * 2.3)) for i in range(4)]
    return fitf(tube(pts, 0.0014, seg=5, smooth=1, r_end=0.0017), f_cyl(0.0015, 0.0015, 0.0425))


def horse_rein():
    # two-ply twisted cord
    rings = []
    for i in range(12):
        y = -0.05 + 0.1 * i / 11
        rings.append([(0.0015 * (1 + 0.16 * math.cos(2 * a - y * 380)) * math.cos(a), y,
                       0.0015 * (1 + 0.16 * math.cos(2 * a - y * 380)) * math.sin(a)) for a in [TAU * k / 6 for k in range(6)]])
    return fitf(loft(rings), f_cyl(0.0015, 0.0015, 0.10))


def horse_wheel():
    # turned wooden wheel, the same both sides: rounded tyre edge, a dished
    # web and a raised hub
    return lathe([(0, -0.0045), (0.009, -0.005), (0.011, -0.0036), (0.0195, -0.0036), (0.0235, -0.005),
                  (0.025, -0.0032), (0.025, 0.0032), (0.0235, 0.005), (0.0195, 0.0036), (0.011, 0.0036),
                  (0.009, 0.005), (0, 0.0045)], seg=14)


part('HobbyHorses__pole', f_cyl(0.010, 0.010, 0.80), horse_pole)
part('HobbyHorses__knob', f_sphere(0.014), horse_knob, smooth=50)
part('HobbyHorses__head', f_box(0.14, 0.08, 0.05), horse_head)
part('HobbyHorses__muzzle', f_box(0.05, 0.055, 0.04), horse_muzzle, smooth=50)
part('HobbyHorses__ear', f_cone(0.008, 0.030), horse_ear, smooth=60)
part('HobbyHorses__strand', f_cyl(0.0015, 0.0015, 0.0425), horse_strand, smooth=180)
part('HobbyHorses__ring', f_torus(0.004, 0.001), lambda: torus(0.004, 0.001, seg=12, tseg=4), smooth=70)
part('HobbyHorses__rein', f_cyl(0.0015, 0.0015, 0.10), horse_rein, smooth=70)
part('HobbyHorses__wheel', f_cyl(0.025, 0.025, 0.010), horse_wheel, smooth=45)
part('HobbyHorses__axle', f_cyl(0.002, 0.002, 0.050),
     lambda: lathe([(0, -0.025), (0.0017, -0.025), (0.002, -0.0243), (0.002, 0.0243), (0.0017, 0.025), (0, 0.025)], seg=8))


# ------------------------------------------------------- small balloons --
def balloon_body():
    # latex balloon: round crown, tapering to the neck at the bottom
    return lathe([(0, -0.1), (0.009, -0.0985), (0.024, -0.09), (0.048, -0.07), (0.072, -0.043), (0.091, -0.014),
                  (0.0995, 0.015), (0.098, 0.042), (0.088, 0.066), (0.068, 0.086), (0.04, 0.097), (0.016, 0.0995),
                  (0, 0.1)], seg=22)


def balloon_knot():
    # (turned apex-down by the placer) the neck gathering out of the
    # balloon at -Y, the tied knot, and the rolled lip at the free end (+Y)
    return lathe([(0.0, -0.015), (0.0148, -0.015), (0.009, -0.009), (0.005, -0.004), (0.0038, 0.0),
                  (0.0058, 0.0045), (0.0062, 0.008), (0.0045, 0.0105), (0.0068, 0.0125), (0.006, 0.0148),
                  (0, 0.015)], seg=12)


def balloon_skin():
    # a burst/deflated balloon lying flat: wrinkled rubber skin, the neck
    # and knot at one side (-Y), top face up (+Z after the placer turns it)
    def rim(a):
        base = 0.086 + 0.006 * math.sin(3 * a + 0.4) + 0.004 * math.sin(5 * a + 1.3)
        neck = 0.03 * math.exp(-((a + HPI) / 0.28) ** 2)
        return base + neck
    n = 24
    out = [(rim(TAU * k / n) * math.cos(TAU * k / n), rim(TAU * k / n) * math.sin(TAU * k / n) + 0.012)
           for k in range(n)]

    def zt(x, y, k):
        return 0.0012 * noise.noise(Vector((x * 45, y * 45, 0.3))) + 0.0012 * (1 - k) * math.sin(x * 90 + y * 40)
    rings = []
    for k in (0.2, 0.45, 0.7, 0.9, 1.0):
        rings.append([(x * k, y * k, zt(x * k, y * k, k)) for x, y in out])
    rings.append([(x, y, -0.0012) for x, y in out])
    s = loft(rings)
    knot = ellipsoid(0.007, 0.009, 0.0045, seg=8).move(y=out[18][1] * 0.97, z=0.001)
    return fitf(s + knot, f_bounds((-0.1, -0.1, -0.0012), (0.1, 0.1, 0.0045), 'plane'))


part('SmallBalloons__body', f_sphere(0.10), balloon_body, smooth=180)
part('SmallBalloons__knot', f_cone(0.015, 0.03), balloon_knot, smooth=70)
part('SmallBalloons__skin', f_bounds((-0.1, -0.1, 0), (0.1, 0.1, 0), 'plane'), balloon_skin, smooth=180)
part('SmallBalloons__string', f_cyl(0.002, 0.002, 0.7),
     lambda: cyl(0.002, 0.002, 0.7, seg=8, bevel_r=0))
