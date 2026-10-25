"""Batch b03: placeProps — the gameplay core placement.

Exit door + EXIT sign, Almond Water, batteries, chairs, CRT TVs, wall clock,
balloon, payphone, mannequin, filing cabinets, cardboard, wet-floor sign,
vending machine, lockers, fire extinguisher, trash can, rubber duck, traffic
cone, wheelchair, baby doll, suitcase, mirror, books, coat rack, plants,
windows, elevators, paper piles, floor hole, weather extras, candles, light
switches, lore notes, pool, robot vacuum, safe room, broken elevator, corpse.
Every part is named Props__<object>_<role>.
"""
from propkit import *


# ---------------------------------------------------------------- helpers --
def F(s, fr):
    """Fit a shape exactly onto a frame's bounds (organic parts)."""
    return s.fit(fr[1], fr[2])


def rod_y(L, r, seg=16, ease=None):
    """Round rod along +Y, centred, eased ends."""
    return cyl(r, r, L, seg=seg, bevel_r=r * 0.35 if ease is None else ease)


def rod_x(L, r, seg=16, ease=None):
    return rod_y(L, r, seg, ease).rot(z=math.pi / 2)


def rod_z(L, r, seg=16, ease=None):
    return rod_y(L, r, seg, ease).rot(x=math.pi / 2)


def lathe_arc(profile, a0, a1, seg=16):
    """Revolve [(r, y)] from angle a0 to a1 (three.js convention:
    x = r sin a, z = r cos a) and close both ends with the profile."""
    v, f, rings = [], [], []
    for (r, y) in profile:
        if r <= 1e-7:
            v.append(Vector((0, y, 0)))
            rings.append([len(v) - 1] * (seg + 1))
        else:
            ids = []
            for i in range(seg + 1):
                a = a0 + (a1 - a0) * i / seg
                v.append(Vector((r * math.sin(a), y, r * math.cos(a))))
                ids.append(len(v) - 1)
            rings.append(ids)
    for a, b in zip(rings, rings[1:]):
        for i in range(seg):
            q = [a[i], b[i], b[i + 1], a[i + 1]]
            q2 = []
            for k in q:
                if k not in q2:
                    q2.append(k)
            if len(q2) >= 3:
                f.append(tuple(q2))
    cap0 = []
    cap1 = []
    for rg in rings:
        if rg[0] not in cap0:
            cap0.append(rg[0])
        if rg[seg] not in cap1:
            cap1.append(rg[seg])
    f.append(tuple(cap0))
    f.append(tuple(reversed(cap1)))
    return propkit_fix(Shape(v, f))


def propkit_fix(s):
    import propkit
    return propkit._fix_winding(s)


def ring_u(w_out, h_out, w_in, h_in, d, r=0.004, seg=1, open_bottom=True):
    """A rectangular frame (outer w×h, opening w_in×h_in) of depth d along Z,
    centred; with open_bottom the opening runs out through the bottom edge
    (door casings), keeping the opening's top at h_out/2 - (h_out - h_in)."""
    outer = box(w_out, h_out, d)
    if open_bottom:
        top_band = h_out - h_in
        hole = box(w_in, h_in + 0.2, d + 0.1).move(y=h_out / 2 - top_band - (h_in + 0.2) / 2)
    else:
        hole = box(w_in, h_in, d + 0.1)
    s = cut(outer, hole)
    return bevel(s, r, seg=seg, angle=30) if r > 0 else s


# -------------------------------------------------------------- exit door --
def exit_frame():
    # moulded door casing: U-shaped architrave around a 1.14 x 2.22 opening,
    # stepped profile (outer back band + flat face + quarter round at the
    # opening), with a stop panel far behind so the jamb never shows through
    W, H = 1.5, 2.5
    base = ring_u(W, H, 1.14, 2.22, 0.14, r=0.006).move(z=-0.02)
    band = ring_u(1.47, 2.485, 1.30, 2.40, 0.05, r=0.014, seg=2).move(y=-0.0075, z=0.065)
    bead = ring_u(1.21, 2.255, 1.14, 2.22, 0.03, r=0.01, seg=2).move(y=-0.0175, z=0.06)
    stop = box(1.16, 2.23, 0.02).move(y=-H / 2 + 2.23 / 2, z=-0.08)
    return base + band + bead + stop

part('Props__exitDoor_frame', f_box(1.5, 2.5, 0.18), exit_frame)


def exit_leaf():
    # a four-rail panelled door: stiles/rails with two recessed raised-and-
    # fielded panels, lever handle on the latch side (clear of the light gap)
    W, H = 1.1, 2.2
    leaf = rbox(W, H, 0.075, r=0.006, seg=2).move(z=-0.0125)
    front = 0.025
    panels = [(-0.50, 0.70), (0.50, 0.90)]      # (centre y, height)
    for cy, ph in panels:
        leaf = cut(leaf, box(0.82, ph, 0.03).move(y=cy, z=front))
    for cy, ph in panels:
        fld = bevel(box(0.74, ph - 0.08, 0.011), 0.012, seg=1, angle=30)
        leaf = leaf + fld.move(y=cy, z=front - 0.012 + 0.0055)
    hx, hy = 0.36, -0.10
    rose = cyl(0.03, 0.03, 0.008, seg=20).rot(x=math.pi / 2).move(x=hx, y=hy, z=front + 0.004)
    lever = tube([(hx, hy, front + 0.004), (hx, hy, 0.04), (hx - 0.03, hy, 0.041), (hx - 0.12, hy, 0.041)],
                 0.009, seg=10, smooth=1)
    esc = rbox(0.022, 0.06, 0.006, r=0.002).move(x=hx, y=hy - 0.12, z=front + 0.003)
    return leaf + rose + lever + esc

part('Props__exitDoor_leaf', f_box(1.1, 2.2, 0.1), exit_leaf)


def exit_sign_housing():
    # dark sign box: rounded tray whose face is recessed 6 mm inside a 3 cm
    # bezel, so the glowing sign panel sits in it instead of z-fighting
    s = rbox(1.36, 0.54, 0.04, r=0.008, seg=2)
    s = cut(s, box(1.305, 0.485, 0.02).move(z=0.02))
    return s

part('Props__exitSign_housing', f_box(1.36, 0.54, 0.04), exit_sign_housing)


# ----------------------------------------------------------- almond water --
def almond_bottle():
    # PET water bottle: petaloid-ish base, straight body with three grip
    # grooves, rounded shoulder, neck with a support ring
    prof = [(0, -0.15), (0.068, -0.15), (0.084, -0.145), (0.09, -0.13), (0.09, -0.07),
            (0.085, -0.064), (0.09, -0.058), (0.09, -0.03), (0.085, -0.024), (0.09, -0.018),
            (0.09, 0.04), (0.086, 0.07), (0.072, 0.098), (0.054, 0.118), (0.046, 0.128),
            (0.046, 0.136), (0.053, 0.139), (0.046, 0.143), (0.044, 0.15), (0, 0.15)]
    return lathe(prof, seg=15)

part('Props__almond_bottle', f_cyl(0.09, 0.09, 0.3), almond_bottle, smooth=50)


def almond_cap():
    # knurled screw cap with a rounded top edge
    prof = [(0, -0.03), (0.047, -0.03), (0.05, -0.026), (0.05, 0.022), (0.047, 0.029), (0.04, 0.03), (0, 0.03)]
    s = lathe(prof, seg=36)

    def knurl(p):
        r = math.hypot(p.x, p.z)
        if r > 0.049 and -0.027 < p.y < 0.023:
            a = math.atan2(p.x, p.z)
            k = round(a / (TAU / 36))
            if k % 2:
                p.x *= 0.94
                p.z *= 0.94
        return p
    return s.warp(knurl)

part('Props__almond_cap', f_cyl(0.05, 0.05, 0.06), almond_cap)


# --------------------------------------------------------------- battery --
def battery_body():
    # D-cell: rolled wrapper edge at both ends, crimp groove under the top
    prof = [(0, -0.09), (0.046, -0.09), (0.054, -0.088), (0.059, -0.084), (0.06, -0.078),
            (0.06, 0.064), (0.056, 0.068), (0.06, 0.072), (0.06, 0.08), (0.057, 0.086),
            (0.05, 0.09), (0, 0.09)]
    return lathe(prof, seg=22)

part('Props__battery_body', f_cyl(0.06, 0.06, 0.18), battery_body, smooth=50)
part('Props__battery_tip', f_cyl(0.025, 0.025, 0.03),
     lambda: lathe([(0, -0.015), (0.025, -0.015), (0.025, -0.009), (0.019, -0.007), (0.017, -0.004),
                    (0.017, 0.011), (0.014, 0.015), (0, 0.015)], seg=16), smooth=50)


# ------------------------------------------------------------ soft helpers --
def puffy(w, h, d, dome=0.25, n=6, dimple=0.0, seed=0, lump=0.0):
    """Upholstered pad w×h×d (thickness h along Y): subdivided slab, domed
    top and bottom, optional button dimple in the middle of the top."""
    s = slab(w, d, h, n, n).rot(x=-math.pi / 2)
    s = subdivide(s, 1)

    def shape(p):
        u, v = min(1.0, abs(p.x) / (w / 2)), min(1.0, abs(p.z) / (d / 2))
        k = (1 - u ** 2) * (1 - v ** 2)
        p.y *= 1 + dome * k
        if dimple and p.y > 0:
            p.y -= dimple * math.exp(-((p.x / w) ** 2 + (p.z / d) ** 2) / 0.004)
        return p
    s = s.warp(shape)
    if lump:
        s = noisy(s, lump, scale=9, seed=seed)
    return s


def puffy_z(w, h, d, **kw):
    """Same pad with its thickness along Z (back cushions)."""
    return puffy(w, d, h, **kw).rot(x=math.pi / 2)


# ------------------------------------------------------------------ chair --
part('Props__chair_seat', f_box(0.50, 0.05, 0.50), lambda: rbox(0.50, 0.05, 0.50, r=0.014, seg=3))
part('Props__chair_cushion', f_box(0.44, 0.06, 0.44),
     lambda: F(puffy(0.44, 0.06, 0.44, dome=0.35, dimple=0.012), f_box(0.44, 0.06, 0.44)), smooth=180)
part('Props__chair_piping', f_cyl(0.008, 0.008, 0.45), lambda: rod_y(0.45, 0.008, seg=10), smooth=180)
part('Props__chair_tuft', f_sphere(0.02),
     lambda: lathe([(0, -0.02), (0.017, -0.02), (0.02, -0.012), (0.02, 0.0), (0.017, 0.01),
                    (0.01, 0.017), (0, 0.02)], seg=12), smooth=180)


def chair_post():
    # square back post, chamfered, with a turned ball finial on top
    post = rbox(0.045, 0.58, 0.045, r=0.005, seg=1).move(y=-0.02)
    fin = lathe([(0, 0.27), (0.02, 0.27), (0.02, 0.278), (0.013, 0.284), (0.019, 0.296),
                 (0.017, 0.305), (0.008, 0.31), (0, 0.31)], seg=12)
    return post + fin

part('Props__chair_post', f_box(0.045, 0.62, 0.045), chair_post)
part('Props__chair_rail', f_box(0.48, 0.04, 0.03), lambda: rbox(0.48, 0.04, 0.03, r=0.008, seg=2))


def chair_slat():
    # a flat baluster slat with a waist, eased edges
    pts = []
    for i in range(9):
        y = -0.16 + 0.32 * i / 8
        half = 0.0125 - 0.0035 * math.sin(math.pi * i / 8)
        pts.append((half, y))
    outline = [(x, y) for x, y in pts] + [(-x, y) for x, y in reversed(pts)]
    return prism(outline, 0.02, bevel_r=0.003)

part('Props__chair_slat', f_box(0.025, 0.32, 0.02), chair_slat)
part('Props__chair_backPad', f_box(0.42, 0.28, 0.04),
     lambda: F(puffy_z(0.42, 0.28, 0.04, dome=0.3), f_box(0.42, 0.28, 0.04)), smooth=180)


def tapered_leg(w, h, bottom=0.8):
    s = rbox(w, h, w, r=w * 0.1, seg=1)
    s = taper(s, top=1 / bottom, along='y', axes='xz')
    return F(s, f_box(w, h, w))

part('Props__chair_legTop', f_box(0.05, 0.30, 0.05), lambda: tapered_leg(0.05, 0.30, 0.86))
part('Props__chair_legBottom', f_box(0.035, 0.18, 0.035), lambda: tapered_leg(0.035, 0.18, 0.8))
part('Props__chair_footCap', f_cyl(0.022, 0.022, 0.02),
     lambda: lathe([(0, -0.01), (0.018, -0.01), (0.022, -0.006), (0.022, 0.006), (0.02, 0.01), (0, 0.01)], seg=14))


def stretcher():
    # turned round stretcher along Y (rotated by the caller)
    return lathe([(0, -0.21), (0.0075, -0.21), (0.0075, -0.19), (0.01, -0.12), (0.01, 0.12),
                  (0.0075, 0.19), (0.0075, 0.21), (0, 0.21)], seg=10)

part('Props__chair_braceX', f_box(0.42, 0.02, 0.02), lambda: stretcher().rot(z=math.pi / 2), smooth=60)
part('Props__chair_braceZ', f_box(0.02, 0.02, 0.42), lambda: stretcher().rot(x=math.pi / 2), smooth=60)


# ----------------------------------------------------------------- CRT TV --
def tv_stand():
    # low TV cabinet: overhanging top, carcass, recessed plinth, two doors
    top = rbox(0.9, 0.03, 0.5, r=0.008, seg=2).move(y=0.235)
    body = rbox(0.86, 0.43, 0.46, r=0.004, seg=1).move(y=0.005)
    plinth = box(0.82, 0.04, 0.42).move(y=-0.23)
    doors = Shape()
    for sx in (-1, 1):
        dr = rbox(0.412, 0.39, 0.014, r=0.004, seg=1).move(x=sx * 0.211, y=0.005, z=0.235)
        knob = cyl(0.011, 0.013, 0.012, seg=12).rot(x=math.pi / 2).move(x=sx * 0.03, y=0.09, z=0.244)
        doors = doors + dr + knob
    return top + body + plinth + doors

part('Props__crtTv_stand', f_box(0.9, 0.5, 0.5), tv_stand)


def crt_body():
    # CRT set: full-size front bezel block, a recessed surround around the
    # screen, and the tapering picture-tube hood behind it
    front = rbox(0.85, 0.62, 0.18, r=0.02, seg=2).move(z=0.185)
    front = cut(front, bevel(box(0.70, 0.50, 0.02), 0.02, seg=2, angle=80).move(z=0.279),
                box(0.10, 0.008, 0.01).move(x=0.3, y=-0.27, z=0.276))
    hood = rbox(0.78, 0.56, 0.4, r=0.05, seg=2).move(z=-0.075)

    def taper_hood(p):
        t = (p.z + 0.275) / 0.4
        p.x *= 0.62 + 0.38 * t
        p.y = p.y * (0.7 + 0.3 * t) + 0.02 * (1 - t)
        return p
    hood = hood.warp(taper_hood)
    return front + hood

part('Props__crtTv_body', f_box(0.85, 0.62, 0.55), crt_body)


# ------------------------------------------------------------ wall clock --
def clock_rim():
    # moulded bezel ring: flat back, rounded front lip, stepped inner edge
    prof = [(0.278, -0.025), (0.32, -0.025), (0.325, -0.018), (0.325, 0.0), (0.318, 0.017),
            (0.305, 0.025), (0.29, 0.024), (0.28, 0.016), (0.276, 0.006), (0.278, -0.025)]
    s = weld(lathe(prof, seg=48, cap_bottom=False, cap_top=False))
    return s.rot(x=math.pi / 2)

part('Props__clock_rim', f_torus(0.3, 0.025), clock_rim)


def circle_pts(cx, cy, r, n=12):
    return [(cx + r * math.cos(TAU * i / n), cy + r * math.sin(TAU * i / n)) for i in range(n)]


def clock_hour_hand():
    blade = [(-0.004, -0.06), (0.004, -0.06), (0.004, 0.02), (0.015, 0.035), (0.011, 0.05), (0, 0.08),
             (-0.011, 0.05), (-0.015, 0.035), (-0.004, 0.02)]
    tail = [(-0.004, -0.06), (0.004, -0.06), (0.006, -0.08), (-0.006, -0.08)]
    return prism(blade, 0.01) + prism(tail, 0.01) + prism(circle_pts(0, -0.06, 0.011), 0.01)

part('Props__clock_hourHand', f_box(0.03, 0.16, 0.01), clock_hour_hand)


def clock_second_hand():
    needle = [(-0.0015, -0.1), (0.0015, -0.1), (0.0008, 0.12), (-0.0008, 0.12)]
    return (prism(needle, 0.008) + prism(circle_pts(0, -0.09, 0.006), 0.008)
            + prism(circle_pts(0, -0.115, 0.005), 0.008))

part('Props__clock_secondHand', f_box(0.012, 0.24, 0.008), clock_second_hand)


# --------------------------------------------------------------- balloon --
def balloon():
    # latex balloon: fuller at the top, drawn in to a neck and a tied knot
    prof = [(0, -0.3), (0.016, -0.297), (0.02, -0.287), (0.011, -0.275), (0.016, -0.266)]
    for i in range(1, 15):
        th = math.pi * i / 15
        base = math.sin(th)
        pinch = 1 - 0.55 * max(0.0, 1 - th / 1.6) ** 2
        prof.append((0.3 * base * pinch, -0.25 + 0.55 * (1 - math.cos(th)) / 2))
    prof.append((0, 0.3))
    return F(lathe(prof, seg=24), f_sphere(0.3))

part('Props__balloon_body', f_sphere(0.3), balloon, smooth=180)


def side_prism(profile_zy, width):
    """Extrude a side profile given as [(z, y)] across X (width), centred."""
    return prism(profile_zy, width).rot(y=-math.pi / 2)


# -------------------------------------------------------------- payphone --
def payphone_housing():
    # steel enclosure: rounded edges, a seam groove where the front shell
    # meets the wall plate, four tamper-proof screws on the front corners
    s = rbox(0.46, 0.70, 0.216, r=0.02, seg=2).move(z=-0.002)
    ringc = cut(box(0.5, 0.8, 0.006), box(0.44, 0.68, 0.01))
    s = cut(s, ringc.move(z=-0.06))
    for sx in (-1, 1):
        for sy in (-1, 1):
            s = s + screw_head(0.008, 0.004, slot=False).rot(x=math.pi / 2).move(x=sx * 0.2, y=sy * 0.32, z=0.106)
    return s

part('Props__payphone_housing', f_box(0.46, 0.70, 0.22), payphone_housing)
part('Props__payphone_hood', f_box(0.48, 0.05, 0.24),
     lambda: bevel(side_prism([(-0.12, -0.025), (-0.12, 0.025), (0.1, 0.025), (0.12, 0.012), (0.12, -0.004),
                               (0.1, -0.025)], 0.48), 0.005, seg=2))


def payphone_face():
    s = rbox(0.40, 0.60, 0.017, r=0.005, seg=2).move(z=-0.0015)
    for sx in (-1, 1):
        for sy in (-1, 1):
            s = s + screw_head(0.006, 0.003, slot=True).rot(x=math.pi / 2).move(x=sx * 0.18, y=sy * 0.28, z=0.007)
    return s

part('Props__payphone_face', f_box(0.40, 0.60, 0.02), payphone_face)
part('Props__payphone_key', f_box(0.058, 0.038, 0.02),
     lambda: F(taper(rbox(0.058, 0.038, 0.02, r=0.004, seg=2), top=0.9, along='z', axes='xy'), f_box(0.058, 0.038, 0.02)))


def coin_plate(w, h, d):
    base = bevel(box(w, h, d * 0.7), d * 0.2, seg=1).move(z=-d * 0.15)
    boss = bevel(box(w * 0.78, h * 0.5, d * 0.4), d * 0.12, seg=1).move(z=d * 0.3)
    return base + boss

part('Props__payphone_coinPlate', f_box(0.10, 0.06, 0.02), lambda: coin_plate(0.10, 0.06, 0.02))
part('Props__payphone_coinSlot', f_box(0.06, 0.008, 0.005), lambda: rbox(0.06, 0.008, 0.005, r=0.0015, seg=1))
part('Props__payphone_coinCup', f_box(0.11, 0.05, 0.03),
     lambda: cut(rbox(0.11, 0.05, 0.03, r=0.006, seg=2), rbox(0.09, 0.03, 0.03, r=0.004, seg=1).move(y=-0.004, z=0.012)))
part('Props__payphone_coinCupSlot', f_box(0.09, 0.025, 0.02), lambda: rbox(0.09, 0.025, 0.02, r=0.004, seg=1))
part('Props__payphone_cradle', f_box(0.10, 0.04, 0.06),
     lambda: bevel(cut(box(0.10, 0.04, 0.06), box(0.056, 0.03, 0.08).move(y=0.017)), 0.004, seg=2))


def payphone_handset():
    # G-style handset seen from the ear side: two oval cups open to the
    # front (the beige caps sit inside them) joined by a grip behind
    s = Shape()
    for sy in (-1, 1):
        cupv = vessel([(0.016, -0.045), (0.035, -0.028), (0.04, 0.0), (0.035, 0.041)],
                      0.006, seg=12, floor=0.032).rot(x=math.pi / 2)

        def squash(p, sy=sy):
            if p.y * sy > 0:
                p.y *= 0.5
            p.y += sy * 0.15
            return p
        s = s + cupv.warp(squash)
    grip = tube([(0, -0.13, -0.018), (0, -0.06, -0.024), (0, 0.06, -0.024), (0, 0.13, -0.018)], 0.02, seg=10, smooth=1)
    return s + grip.scale(1.6, 1, 1)

part('Props__payphone_handset', f_box(0.08, 0.34, 0.09), payphone_handset)
part('Props__payphone_earCap', f_cyl(0.035, 0.035, 0.02),
     lambda: lathe([(0, 0.008), (0.012, 0.008), (0.014, 0.01), (0.028, 0.01), (0.033, 0.008), (0.035, 0.004),
                    (0.035, -0.01), (0, -0.01)], seg=20))


def cord_coil():
    pts = []
    n = 18
    for i in range(n + 1):
        a = TAU * i / n
        pts.append((0.018 * math.cos(a), 0.018 * math.sin(a), -0.0007 + 0.0014 * i / n))
    return tube(pts, 0.005, seg=6, caps=True)

part('Props__payphone_cordCoil', f_torus(0.018, 0.005), cord_coil, smooth=180)


# ------------------------------------------------------------- mannequin --
def mannequin_torso():
    # shop dummy torso: broad square shoulders, chest, waist, hips, short neck
    prof = [(0, -0.52), (0.07, -0.51), (0.13, -0.475), (0.165, -0.41), (0.172, -0.33), (0.165, -0.22),
            (0.15, -0.1), (0.148, -0.02), (0.16, 0.08), (0.178, 0.18), (0.19, 0.27), (0.196, 0.34),
            (0.19, 0.385), (0.16, 0.415), (0.1, 0.44), (0.066, 0.45), (0.058, 0.48), (0.055, 0.52), (0, 0.52)]
    s = lathe(prof, seg=20)

    def sculpt(p):
        y = p.y
        sh = math.exp(-((y - 0.35) / 0.09) ** 2)          # shoulders stand out sideways
        p.x *= 1 + 0.16 * sh
        p.z *= 0.7 - 0.14 * sh
        if p.z > 0:                                        # chest
            p.z += 0.035 * math.exp(-((y - 0.18) / 0.1) ** 2) * math.exp(-(p.x / 0.12) ** 2)
        else:                                              # seat
            p.z -= 0.03 * math.exp(-((y + 0.33) / 0.09) ** 2) * math.exp(-(p.x / 0.13) ** 2)
        return p
    return F(s.warp(sculpt), f_capsule(0.21, 0.62))

part('Props__mannequin_torso', f_capsule(0.21, 0.62), mannequin_torso, smooth=180)


def mannequin_head():
    s = sphere(1, seg=28, rings=16).scale(0.125, 0.155, 0.135)

    def face(p):
        if p.y < 0:
            p.x *= 1 - 0.28 * (-p.y / 0.155) ** 1.6          # jaw narrows to a chin
            p.z *= 1 - 0.12 * (-p.y / 0.155) ** 2
        if p.z > 0:
            ax = abs(p.x)
            p.z += 0.02 * math.exp(-(p.x / 0.016) ** 2 - ((p.y + 0.012) / 0.03) ** 2)       # nose
            p.z += 0.008 * math.exp(-((p.y - 0.03) / 0.014) ** 2) * math.exp(-(p.x / 0.06) ** 2)   # brow
            p.z -= 0.012 * math.exp(-((ax - 0.042) / 0.02) ** 2 - ((p.y - 0.008) / 0.016) ** 2)  # sockets
            p.z += 0.006 * math.exp(-(p.x / 0.03) ** 2 - ((p.y + 0.11) / 0.02) ** 2)           # chin
        return p
    return F(s.warp(face), f_sphere(0.155))

part('Props__mannequin_head', f_sphere(0.155), mannequin_head, smooth=180)


def mannequin_leg():
    prof = [(0, 0.385), (0.055, 0.375), (0.075, 0.34), (0.072, 0.25), (0.062, 0.12), (0.048, 0.02),
            (0.045, -0.02), (0.05, -0.09), (0.046, -0.17), (0.036, -0.26), (0.028, -0.32), (0.03, -0.345),
            (0, -0.35)]
    s = lathe(list(reversed(prof)), seg=16)
    s = s.warp(lambda p: p if p.z > 0 else Vector((p.x, p.y, p.z - 0.012 * math.exp(-((p.y + 0.09) / 0.07) ** 2))))
    foot = ellipsoid(0.033, 0.02, 0.06, seg=14).move(y=-0.365, z=0.015)
    return F(s + foot, f_capsule(0.075, 0.62))

part('Props__mannequin_leg', f_capsule(0.075, 0.62), mannequin_leg, smooth=180)


def mannequin_arm():
    prof = [(0, 0.315), (0.045, 0.3), (0.055, 0.26), (0.05, 0.14), (0.04, 0.03), (0.037, 0.0),
            (0.04, -0.06), (0.034, -0.15), (0.025, -0.205), (0, -0.215)]
    s = lathe(list(reversed(prof)), seg=14)
    hand = ellipsoid(0.014, 0.05, 0.026, seg=12).move(y=-0.262, z=0.004)
    return F(s + hand, f_capsule(0.055, 0.52))

part('Props__mannequin_arm', f_capsule(0.055, 0.52), mannequin_arm, smooth=180)


# --------------------------------------------------------- filing cabinet --
def cabinet_body():
    # steel carcass with three real drawer bays (seen when a drawer is
    # pulled out) and a recessed toe kick
    s = rbox(0.6, 1.4, 0.55, r=0.012, seg=2)
    bays = [box(0.535, 0.325, 0.505).move(y=y, z=0.0375) for y in (-0.42, 0.03, 0.48)]
    kick = box(0.64, 0.045, 0.05).move(y=-0.69, z=0.275)
    return cut(s, *bays, kick)

part('Props__cabinet_body', f_box(0.6, 1.4, 0.55), cabinet_body)
part('Props__cabinet_top', f_box(0.62, 0.02, 0.57), lambda: rbox(0.62, 0.02, 0.57, r=0.006, seg=2))
part('Props__cabinet_foot', f_box(0.06, 0.06, 0.06),
     lambda: lathe([(0, -0.03), (0.028, -0.03), (0.03, -0.026), (0.03, -0.02), (0.026, -0.016), (0.01, -0.014),
                    (0.009, 0.0), (0.016, 0.0), (0.016, 0.012), (0.009, 0.014), (0.009, 0.03), (0, 0.03)], seg=12))
part('Props__cabinet_brandPlate', f_box(0.14, 0.005, 0.05), lambda: rbox(0.14, 0.005, 0.05, r=0.0015, seg=1))
part('Props__cabinet_rivet', f_sphere(0.008), lambda: sphere(0.008, seg=8, rings=5), smooth=180)


def cabinet_drawer_front():
    s = rbox(0.55, 0.34, 0.03, r=0.006, seg=2)
    groove = cut(box(0.51, 0.30, 0.004), box(0.504, 0.294, 0.01))
    return cut(s, groove.move(z=0.015))

part('Props__cabinet_drawerFront', f_box(0.55, 0.34, 0.03), cabinet_drawer_front)
part('Props__cabinet_pullPlate', f_box(0.2, 0.08, 0.02),
     lambda: cut(bevel(box(0.2, 0.08, 0.02), 0.004, seg=1), rbox(0.15, 0.036, 0.024, r=0.008, seg=2).move(y=-0.01, z=0.012)))
part('Props__cabinet_pull', f_box(0.14, 0.02, 0.05),
     lambda: F(tube([(-0.062, 0, -0.025), (-0.062, 0, 0.01), (-0.05, 0, 0.017), (0.05, 0, 0.017),
                     (0.062, 0, 0.01), (0.062, 0, -0.025)], 0.0085, seg=10, smooth=2), f_box(0.14, 0.02, 0.05)), smooth=60)
part('Props__cabinet_lock', f_cyl(0.014, 0.014, 0.02),
     lambda: cut(lathe([(0, -0.01), (0.012, -0.01), (0.012, 0.004), (0.014, 0.005), (0.014, 0.008), (0.012, 0.01),
                        (0, 0.01)], seg=14), box(0.0032, 0.01, 0.014).move(y=0.01)))
part('Props__cabinet_label', f_box(0.16, 0.05, 0.01), lambda: rbox(0.16, 0.05, 0.01, r=0.002, seg=1))
part('Props__cabinet_drawerBottom', f_box(0.53, 0.015, 0.48), lambda: rbox(0.53, 0.015, 0.48, r=0.004, seg=1))
part('Props__cabinet_drawerBack', f_box(0.53, 0.32, 0.015),
     lambda: rbox(0.53, 0.305, 0.011, r=0.003, seg=1).move(y=-0.0075) + rod_x(0.53, 0.0075, seg=10).move(y=0.1525))
part('Props__cabinet_drawerSide', f_box(0.015, 0.32, 0.48),
     lambda: rbox(0.011, 0.305, 0.48, r=0.003, seg=1).move(y=-0.0075) + rod_z(0.48, 0.0075, seg=10).move(y=0.1525))
part('Props__cabinet_fileRail', f_box(0.008, 0.02, 0.44),
     lambda: prism([(-0.004, -0.01), (0.004, -0.01), (0.004, 0.01), (0.0005, 0.01), (0.0005, -0.006), (-0.004, -0.006)], 0.44))


def cabinet_folder():
    # hanging file folder: two covers meeting in a V at the bottom, the
    # hanger bar across the top and an index tab standing up
    s = Shape()
    ang = math.atan2(0.0075, 0.26)
    for sz in (-1, 1):
        cov = box(0.43, 0.262, 0.0018).move(y=0.131)
        cov.rot(x=-sz * ang).move(y=-0.14, z=sz * 0.0008)
        s = s + cov
    s = s + rod_x(0.47, 0.0025, seg=6).move(y=0.121, z=0.0)
    s = s + rbox(0.07, 0.024, 0.0016, r=0.002, seg=1).move(x=-0.12, y=0.128, z=0.0085)
    return F(s, f_box(0.47, 0.28, 0.02))

part('Props__cabinet_folder', f_box(0.47, 0.28, 0.02), cabinet_folder)


# -------------------------------------------------------------- cardboard --
def grid_box(w, h, d, n=6):
    """Box whose six faces are n×n grids (welded), for bulging/denting."""
    faces = [grid_plane(w, h, n, n).move(z=d / 2), grid_plane(w, h, n, n).rot(y=math.pi).move(z=-d / 2),
             grid_plane(d, h, n, n).rot(y=math.pi / 2).move(x=w / 2), grid_plane(d, h, n, n).rot(y=-math.pi / 2).move(x=-w / 2),
             grid_plane(w, d, n, n).rot(x=-math.pi / 2).move(y=h / 2), grid_plane(w, d, n, n).rot(x=math.pi / 2).move(y=-h / 2)]
    return propkit_fix(weld(sum(faces, Shape()), 1e-6))


def cardboard_box():
    # corrugated carton: sides bulge a little, edges slightly crushed,
    # the two top flaps sag toward the seam under the tape
    w, h, d = 0.5, 0.32, 0.5
    s = grid_box(w, h, d, 6)

    def shape(p):
        u, v, t = p.x / (w / 2), p.y / (h / 2), p.z / (d / 2)
        bx = (1 - v * v) * (1 - t * t)
        bz = (1 - v * v) * (1 - u * u)
        p.x *= 1 + 0.03 * bx
        p.z *= 1 + 0.03 * bz
        if v > 0.99:
            p.y -= 0.006 * (1 - abs(t)) * (1 - u * u)
        return p
    s = noisy(s.warp(shape), 0.003, scale=12, seed=3)
    return F(bevel(s, 0.004, seg=1, angle=50), f_box(w, h, d))

part('Props__cardboard_box', f_box(0.5, 0.32, 0.5), cardboard_box)
part('Props__cardboard_flap', f_box(0.24, 0.005, 0.24),
     lambda: noisy(slab(0.24, 0.24, 0.005, 4, 4).rot(x=-math.pi / 2), 0.0015, scale=10, seed=5))
part('Props__cardboard_tear', f_box(0.06, 0.06, 0.06),
     lambda: F(noisy(subdivide(box(0.06, 0.06, 0.06), 2), 0.007, scale=40, seed=7), f_box(0.06, 0.06, 0.06)), flat=True)


# -------------------------------------------------------- wet floor sign --
def wet_board():
    s = prism(rounded_rect(0.38, 0.7, 0.03, seg=3), 0.015, bevel_r=0.003)
    for sz in (-1, 1):
        s = cut(s, prism(rounded_rect(0.32, 0.64, 0.02, seg=2), 0.006).move(z=sz * 0.0075))
    return s

part('Props__wetSign_board', f_box(0.38, 0.7, 0.015), wet_board)


def wet_hinge():
    s = rod_y(0.35, 0.006, seg=8)
    for i in range(5):
        y = -0.175 + 0.035 + i * 0.07
        s = s + cyl(0.015, 0.015, 0.062, seg=11, bevel_r=0.003, bevel_seg=1).move(y=y)
    return F(s, f_cyl(0.015, 0.015, 0.35))

part('Props__wetSign_hinge', f_cyl(0.015, 0.015, 0.35), wet_hinge)
part('Props__wetSign_base', f_box(0.4, 0.03, 0.35),
     lambda: prism(rounded_rect(0.4, 0.35, 0.05, seg=3), 0.03, bevel_r=0.008).rot(x=math.pi / 2))


# ---------------------------------------------------------------- vending --
def push_button(r, h, seg=16):
    """Round push button along +Y: straight skirt, eased rim, dished face."""
    return lathe([(0, -h / 2), (r, -h / 2), (r, h * 0.15), (r * 0.9, h * 0.42), (r * 0.75, h / 2),
                  (r * 0.5, h * 0.42), (0, h * 0.36)], seg=seg)


def vending_body():
    # cabinet shell with rounded edges, louvred vents low on both sides and
    # the display bay cut into the front (the placer's glass, shelves and
    # cans sit in that bay; a solid block here hid all of them)
    s = rbox(1.05, 1.89, 0.65, r=0.02, seg=2)
    vents = []
    for sx in (-1, 1):
        for i in range(4):
            vents.append(rbox(0.04, 0.012, 0.16, r=0.004, seg=1).move(x=sx * 0.525, y=-0.78 + i * 0.035, z=-0.18))
    W, H = 1.05, 2.05
    bay = box(W * 0.60, H * 0.55, 0.34).move(x=-W * 0.16, y=H * 0.55 - 1.025, z=0.17)
    return cut(s, bay, *vents)

part('Props__vending_body', f_box(1.05, 1.89, 0.65), vending_body)
part('Props__vending_plinth', f_box(1.055, 0.08, 0.655),
     lambda: cut(rbox(1.055, 0.08, 0.655, r=0.004, seg=1), box(0.98, 0.05, 0.06).move(y=-0.02, z=0.33)))
part('Props__vending_foot', f_box(0.06, 0.04, 0.06),
     lambda: lathe([(0, -0.02), (0.028, -0.02), (0.03, -0.016), (0.03, -0.008), (0.012, -0.004), (0.012, 0.02), (0, 0.02)], seg=12))
part('Props__vending_topTrim', f_box(1.05, 0.03, 0.655), lambda: rbox(1.05, 0.03, 0.655, r=0.012, seg=3))
part('Props__vending_headerTrim', f_box(1.05, 0.015, 0.655), lambda: rbox(1.05, 0.015, 0.655, r=0.005, seg=1))
# the display compartment is a box open to the front: a solid block here
# hid every can and shelf behind it
part('Props__vending_interior', f_box(0.61, 1.1075, 0.30),
     lambda: cut(box(0.61, 1.1075, 0.30), box(0.57, 1.0675, 0.32).move(z=0.03)))
part('Props__vending_shelf', f_box(0.57, 0.015, 0.20),
     lambda: bevel(side_prism([(-0.1, 0.0075), (0.1, 0.0075), (0.1, -0.0075), (0.094, -0.0075), (0.094, 0.0015),
                               (-0.1, 0.0015)], 0.57), 0.0015, seg=1))
part('Props__vending_can', f_cyl(0.048, 0.048, 0.115),
     lambda: lathe([(0, -0.052), (0.03, -0.0555), (0.04, -0.0575), (0.044, -0.056), (0.048, -0.048), (0.048, 0.044),
                    (0.042, 0.053), (0.04, 0.0575), (0.034, 0.0575), (0.033, 0.054), (0, 0.054)], seg=20), smooth=45)
part('Props__vending_canRim', f_cyl(0.05, 0.05, 0.008),
     lambda: lathe([(0, 0.001), (0.042, 0.001), (0.043, 0.004), (0.048, 0.004), (0.05, 0.002), (0.05, -0.002),
                    (0.046, -0.004), (0, -0.004)], seg=20))
part('Props__vending_canLabel', f_cyl(0.049, 0.049, 0.06325), lambda: cyl(0.049, 0.049, 0.06325, seg=24, bevel_r=0.001, bevel_seg=1))
part('Props__vending_pullTab', f_box(0.02, 0.003, 0.008),
     lambda: cut(prism(rounded_rect(0.02, 0.008, 0.0035, seg=2), 0.003), box(0.007, 0.004, 0.01).move(x=0.004)).rot(x=math.pi / 2))
part('Props__vending_glassFrame', f_box(0.67, 1.1675, 0.015),
     lambda: ring_u(0.67, 1.1675, 0.63, 1.1275, 0.015, r=0.004, seg=2, open_bottom=False))
part('Props__vending_panel', f_box(0.252, 1.1275, 0.02), lambda: rbox(0.252, 1.1275, 0.02, r=0.006, seg=2))
part('Props__vending_displayBezel', f_box(0.21, 0.07, 0.015), lambda: rbox(0.21, 0.07, 0.015, r=0.005, seg=2))
part('Props__vending_keyBezel', f_box(0.052, 0.052, 0.008),
     lambda: cut(prism(rounded_rect(0.052, 0.052, 0.008, seg=2), 0.008, bevel_r=0.0015),
                 cyl(0.023, 0.019, 0.006, seg=20, bevel_r=0).rot(x=math.pi / 2).move(z=0.004)))
part('Props__vending_key', f_cyl(0.02, 0.02, 0.012), lambda: push_button(0.02, 0.012))
part('Props__vending_coinPlate', f_box(0.168, 0.05, 0.02), lambda: coin_plate(0.168, 0.05, 0.02))
part('Props__vending_coinSlot', f_box(0.045, 0.008, 0.008), lambda: rbox(0.045, 0.008, 0.008, r=0.002, seg=1))
part('Props__vending_returnButton', f_cyl(0.024, 0.024, 0.014), lambda: push_button(0.024, 0.014, seg=18))
part('Props__vending_bin', f_box(0.735, 0.18, 0.05),
     lambda: cut(rbox(0.735, 0.18, 0.05, r=0.006, seg=2), box(0.69, 0.14, 0.05).move(z=0.012)))
part('Props__vending_flap', f_box(0.63, 0.14, 0.008),
     lambda: cut(rbox(0.63, 0.14, 0.008, r=0.003, seg=1), box(0.4, 0.008, 0.004).move(y=-0.05, z=0.004)))
part('Props__vending_binLip', f_box(0.756, 0.02, 0.05), lambda: rbox(0.756, 0.02, 0.05, r=0.008, seg=3))
part('Props__vending_returnCup', f_box(0.12, 0.08, 0.04),
     lambda: cut(rbox(0.12, 0.08, 0.04, r=0.006, seg=2), rbox(0.1, 0.05, 0.04, r=0.006, seg=1).move(y=-0.005, z=0.012)))
part('Props__vending_returnCupSlot', f_box(0.10, 0.05, 0.02), lambda: rbox(0.10, 0.05, 0.02, r=0.006, seg=1))


# ------------------------------------------------------------------ coats --
def hanging_coat(w, h, d, seed=0):
    """A coat hanging from a hook: sloped shoulders drawn in to a collar,
    sleeves pressed to the sides (two vertical folds), folds growing toward
    the hem, and the hanger hook on top."""
    hook_h = 0.05
    hb = h - hook_h
    s = slab(w, hb, d, 4, 8)

    def shape(p):
        y = p.y / (hb / 2)                    # -1 .. 1
        if y > 0.55:                           # shoulders slope to the collar
            k = (y - 0.55) / 0.45
            p.x *= 1 - 0.78 * k ** 1.4
            p.z *= 1 - 0.35 * k
        p.z *= 0.75 + 0.25 * math.cos(p.x / (w / 2) * 1.2)
        return p
    s = subdivide(s.warp(shape), 1)

    def folds(p):
        u = p.x / (w / 2)
        drop = max(0.0, -p.y / (hb / 2) + 0.2)
        if p.z > 0:
            p.z -= d * 0.18 * math.exp(-((abs(u) - 0.62) / 0.12) ** 2)        # sleeve seams
        p.z += d * 0.12 * drop * math.sin(u * 7.0 + seed)
        return p
    s = noisy(s.warp(folds), d * 0.05, scale=14, seed=seed).move(y=-hook_h / 2)
    top = hb / 2 - hook_h / 2
    hook = tube([(0, top - 0.02, 0), (0, top + 0.02, 0), (0, top + hook_h - 0.012, 0), (0.012, top + hook_h, 0),
                 (0.024, top + hook_h - 0.012, 0), (0.024, top + hook_h - 0.022, 0)], 0.004, seg=6, smooth=2)
    return F(s + hook, f_box(w, h, d))


# ----------------------------------------------------------------- locker --
part('Props__locker_back', f_box(0.92, 1.72, 0.05), lambda: rbox(0.92, 1.72, 0.05, r=0.006, seg=1))
part('Props__locker_top', f_box(0.92, 0.04, 0.55),
     lambda: cut(rbox(0.92, 0.04, 0.55, r=0.005, seg=1), box(0.88, 0.04, 0.51).move(y=-0.014, z=-0.03)))
part('Props__locker_rail', f_box(0.06, 0.06, 0.5), lambda: rbox(0.06, 0.06, 0.5, r=0.006, seg=1))


def locker_unit():
    # locker column: rounded carcass, door outline groove on the front,
    # three hinge knuckles down the left edge
    s = rbox(0.42, 1.7, 0.5, r=0.006, seg=1)
    groove = cut(box(0.40, 1.66, 0.006), box(0.394, 1.654, 0.01))
    s = cut(s, groove.move(z=0.25))
    for y in (-0.6, 0.0, 0.6):
        s = s + cyl(0.007, 0.007, 0.07, seg=8, bevel_r=0.002).move(x=-0.203, y=y, z=0.24)
    return s

part('Props__locker_unit', f_box(0.42, 1.7, 0.5), locker_unit)
part('Props__locker_louvre', f_box(0.28, 0.02, 0.03),
     lambda: bevel(side_prism([(-0.015, -0.01), (0.015, -0.01), (0.015, -0.005), (-0.004, 0.01), (-0.015, 0.01)], 0.28), 0.0015, seg=1))
part('Props__locker_handlePlate', f_box(0.14, 0.16, 0.015),
     lambda: cut(rbox(0.14, 0.16, 0.015, r=0.008, seg=2), rbox(0.1, 0.12, 0.015, r=0.012, seg=2).move(z=0.0095)))
part('Props__locker_handle', f_box(0.03, 0.12, 0.025),
     lambda: F(tube([(0, -0.052, -0.0125), (0, -0.052, 0.002), (0, -0.04, 0.007), (0, 0.04, 0.007),
                     (0, 0.052, 0.002), (0, 0.052, -0.0125)], 0.0065, seg=10, smooth=2), f_box(0.03, 0.12, 0.025)), smooth=60)


def padlock_body():
    s = rbox(0.05, 0.06, 0.03, r=0.008, seg=2)
    key = cyl(0.004, 0.004, 0.02, seg=10, bevel_r=0).rot(x=math.pi / 2).move(y=-0.008, z=0.015)
    slot = box(0.0025, 0.012, 0.02).move(y=-0.015, z=0.015)
    return cut(s, key, slot)

part('Props__locker_padlock', f_box(0.05, 0.06, 0.03), padlock_body)
part('Props__locker_shackle', f_torus(0.02, 0.005), lambda: torus(0.02, 0.005, seg=16, tseg=8))
part('Props__locker_coat', f_box(0.24, 0.7, 0.08), lambda: hanging_coat(0.24, 0.7, 0.08, seed=1), smooth=180)


# ---------------------------------------------------- fire extinguisher --
def ext_bracket():
    back = rbox(0.24, 0.06, 0.008, r=0.002, seg=1).move(z=-0.016)
    lip = rbox(0.16, 0.012, 0.04, r=0.003, seg=1).move(y=0.024)
    hook = rbox(0.16, 0.024, 0.006, r=0.002, seg=1).move(y=0.012, z=0.017)
    screws = sum((screw_head(0.006, 0.003).rot(x=math.pi / 2).move(x=sx * 0.1, y=-0.01, z=-0.012) for sx in (-1, 1)), Shape())
    return back + lip + hook + screws

part('Props__extinguisher_bracket', f_box(0.24, 0.06, 0.04), ext_bracket)
part('Props__extinguisher_strap', f_box(0.05, 0.24, 0.24),
     lambda: cut(prism(rounded_rect(0.24, 0.24, 0.05, seg=3), 0.05, bevel_r=0.003),
                 prism(rounded_rect(0.222, 0.222, 0.04, seg=3), 0.08)).rot(y=math.pi / 2))
part('Props__extinguisher_tank', f_cyl(0.11, 0.11, 0.5),
     lambda: lathe([(0, -0.25), (0.106, -0.25), (0.106, -0.235), (0.11, -0.228), (0.11, -0.196), (0.107, -0.19),
                    (0.11, -0.184), (0.11, 0.24), (0.107, 0.248), (0.1, 0.25), (0, 0.25)], seg=28), smooth=50)
part('Props__extinguisher_shoulder', f_cyl(0.08, 0.11, 0.07),
     lambda: lathe([(0, -0.035), (0.11, -0.035), (0.109, -0.02), (0.104, -0.005), (0.096, 0.01), (0.087, 0.024),
                    (0.08, 0.035), (0, 0.035)], seg=28), smooth=50)
part('Props__extinguisher_valve', f_cyl(0.045, 0.06, 0.05),
     lambda: lathe([(0, -0.025), (0.06, -0.025), (0.06, -0.012), (0.052, -0.01), (0.052, 0.0), (0.048, 0.002),
                    (0.048, 0.016), (0.045, 0.02), (0.045, 0.025), (0, 0.025)], seg=16))


def ext_lever(curl):
    # stamped steel lever: channel section, rounded tail, nose curled
    # down (carry handle) or up (squeeze lever)
    prof = [(-0.08, -0.002), (-0.074, -0.009), (0.06, -0.01), (0.08, -0.01 + curl), (0.08, 0.0 + curl),
            (0.06, 0.01), (-0.074, 0.01), (-0.08, 0.004)]
    s = side_prism(prof, 0.09)
    s = cut(s, box(0.07, 0.012, 0.13).move(y=-0.009, z=-0.005))
    return F(bevel(s, 0.002, seg=1), f_box(0.09, 0.02, 0.16))

part('Props__extinguisher_carryHandle', f_box(0.09, 0.02, 0.16), lambda: ext_lever(-0.004))
part('Props__extinguisher_lever', f_box(0.09, 0.02, 0.16), lambda: ext_lever(0.006))
part('Props__extinguisher_pin', f_cyl(0.006, 0.006, 0.06),
     lambda: rod_y(0.056, 0.004, seg=8).move(y=-0.002) + cyl(0.006, 0.006, 0.006, seg=8, bevel_r=0.0015).move(y=0.027))
part('Props__extinguisher_pullRing', f_torus(0.015, 0.004), lambda: torus(0.015, 0.004, seg=14, tseg=6))
part('Props__extinguisher_gauge', f_cyl(0.028, 0.028, 0.02),
     lambda: lathe([(0, -0.01), (0.026, -0.01), (0.028, -0.007), (0.028, 0.006), (0.027, 0.01), (0.022, 0.01),
                    (0.021, 0.006), (0, 0.006)], seg=18))
part('Props__extinguisher_hose', f_cyl(0.012, 0.012, 0.3),
     lambda: lathe([(0, -0.15), (0.012, -0.15), (0.012, -0.124), (0.011, -0.12), (0.012, -0.116), (0.0095, -0.11),
                    (0.0095, 0.11), (0.012, 0.116), (0.011, 0.12), (0.012, 0.124), (0.012, 0.15), (0, 0.15)], seg=10), smooth=60)
part('Props__extinguisher_nozzle', f_box(0.05, 0.05, 0.14),
     lambda: F(vessel([(0.011, -0.07), (0.011, -0.04), (0.015, -0.03), (0.02, 0.03), (0.025, 0.07)], 0.003, seg=14)
               .rot(x=math.pi / 2), f_box(0.05, 0.05, 0.14)), smooth=60)


# -------------------------------------------------------------- trash can --
part('Props__trashCan_body', f_cyl(0.22, 0.19, 0.65),
     # tapered steel can: rolled foot bead, top bead, contents 1.5 cm under the rim
     lambda: lathe([(0, -0.325), (0.18, -0.325), (0.19, -0.318), (0.19, -0.3), (0.186, -0.292), (0.19, -0.285),
                    (0.216, 0.29), (0.212, 0.298), (0.22, 0.306), (0.22, 0.322), (0.212, 0.325), (0.205, 0.312),
                    (0.1, 0.308), (0, 0.31)], seg=28), smooth=40)
part('Props__trashCan_rib', f_torus(0.215, 0.008), lambda: torus(0.215, 0.008, seg=40, tseg=6), smooth=60)
part('Props__trashCan_rim', f_torus(0.22, 0.012), lambda: torus(0.22, 0.012, seg=40, tseg=8), smooth=60)


def paper_ball(seed=0):
    s = subdivide(box(0.1, 0.1, 0.1), 2)
    s = noisy(s, 0.018, scale=28, seed=seed)
    return F(s, f_sphere(0.07))

part('Props__trash_paperBall', f_sphere(0.07), lambda: paper_ball(2), flat=True)


def crushed_can():
    s = lathe([(0, -0.045), (0.026, -0.045), (0.03, -0.04), (0.03, 0.036), (0.026, 0.043), (0.022, 0.045), (0, 0.044)], seg=12)
    s = subdivide(s, 1)

    def crush(p):
        k = math.exp(-(p.y / 0.02) ** 2)
        p.x *= 1 - 0.55 * k
        p.y *= 1 - 0.1 * k
        return p
    return F(noisy(s.warp(crush), 0.003, scale=60, seed=4), f_cyl(0.03, 0.03, 0.09))

part('Props__trash_crushedCan', f_cyl(0.03, 0.03, 0.09), crushed_can, smooth=40)


def peel_strip():
    s = slab(0.02, 0.09, 0.003, 2, 8)
    s = taper(s, top=0.45, along='y', axes='x')
    s = bend(s, 1.1, axis='x', along='y')
    return F(s, f_box(0.02, 0.09, 0.02))

part('Props__trash_peel', f_box(0.02, 0.09, 0.02), peel_strip, smooth=70)


def takeout_lid():
    flange = prism(rounded_rect(0.12, 0.08, 0.014, seg=2), 0.003).move(z=-0.0085)
    dome = prism(rounded_rect(0.104, 0.064, 0.012, seg=2), 0.017, bevel_r=0.004, bevel_seg=2).move(z=0.0015)
    s = cut(flange + dome, prism(rounded_rect(0.098, 0.058, 0.01, seg=2), 0.016).move(z=-0.004))
    return s.rot(x=-math.pi / 2)

part('Props__trash_lid', f_box(0.12, 0.02, 0.08), takeout_lid)
part('Props__trash_butt', f_cyl(0.008, 0.008, 0.03),
     lambda: noisy(lathe([(0, -0.015), (0.0075, -0.015), (0.008, -0.012), (0.008, 0.004), (0.0074, 0.006),
                          (0.008, 0.009), (0.0072, 0.015), (0, 0.013)], seg=10), 0.0006, scale=200, seed=1), smooth=60)
part('Props__trash_buttTip', f_cyl(0.008, 0.008, 0.008),
     lambda: lathe([(0, -0.004), (0.008, -0.004), (0.0078, 0.001), (0.006, 0.0035), (0, 0.004)], seg=10), smooth=60)
part('Props__trash_debris', f_box(0.055, 0.02, 0.055),
     lambda: F(noisy(slab(0.055, 0.055, 0.003, 4, 4).rot(x=-math.pi / 2), 0.007, scale=45, seed=8), f_box(0.055, 0.02, 0.055)),
     flat=True)


# ----------------------------------------------------------- rubber duck --
def duck_body():
    s = sphere(1, seg=20, rings=12)

    def shape(p):
        if p.y < -0.45:                                   # sits flat
            p.y = -0.45 + (p.y + 0.45) * 0.35
        if p.z < 0:                                       # tail swept up
            t = max(0.0, -p.z - 0.25)
            p.y += 0.9 * t ** 1.6 * max(0.0, 1 - abs(p.x) * 1.3)
            p.z *= 1 + 0.15 * t
        if p.z > 0.3:                                     # full chest
            p.z += 0.08 * (p.z - 0.3) * max(0.0, 1 - p.y ** 2)
        return p
    return F(s.warp(shape), f_sphere(0.09))

part('Props__duck_body', f_sphere(0.09), duck_body, smooth=180)


def duck_head():
    s = sphere(1, seg=18, rings=10)

    def shape(p):
        p.x *= 1 + 0.08 * math.exp(-((p.y + 0.3) / 0.35) ** 2)     # cheeks
        if p.y > 0.5:
            p.y = 0.5 + (p.y - 0.5) * 0.8
        return p
    return F(s.warp(shape), f_sphere(0.06))

part('Props__duck_head', f_sphere(0.06), duck_head, smooth=180)


def duck_beak():
    # bill: round where it joins the head, flat and wide toward the tip
    s = lathe([(0, -0.0275), (0.025, -0.0275), (0.024, -0.012), (0.021, 0.006), (0.017, 0.019),
               (0.01, 0.026), (0, 0.0275)], seg=16)

    def flat(p):
        t = (p.y + 0.0275) / 0.055
        p.z *= 1 - 0.62 * t
        p.x *= 1 + 0.18 * t
        return p
    return F(s.warp(flat), f_cone(0.025, 0.055))

part('Props__duck_beak', f_cone(0.025, 0.055), duck_beak, smooth=70)
part('Props__duck_eye', f_sphere(0.012), lambda: sphere(0.012, seg=10, rings=6), smooth=180)


# ---------------------------------------------------------- traffic cone --
def cone_base():
    low = prism(rounded_rect(0.34, 0.34, 0.05, seg=3), 0.018, bevel_r=0.004).rot(x=math.pi / 2).move(y=-0.006)
    top = prism(rounded_rect(0.28, 0.28, 0.06, seg=3), 0.012, bevel_r=0.004).rot(x=math.pi / 2).move(y=0.009)
    return low + top

part('Props__cone_base', f_box(0.34, 0.03, 0.34), cone_base)
part('Props__cone_body', f_cone(0.16, 0.55),
     lambda: lathe([(0, -0.275), (0.16, -0.275), (0.158, -0.263), (0.15, -0.252), (0.03, 0.255), (0.024, 0.27),
                    (0.016, 0.275), (0, 0.275)], seg=28), smooth=40)
part('Props__cone_band', f_cyl(0.13, 0.15, 0.05), lambda: cyl(0.13, 0.15, 0.05, seg=28, bevel_r=0.004))


# ------------------------------------------------------------- wheelchair --
def tube_loop(w, d, r_corner, r_tube, plane='xz', seg=10):
    """Closed rounded-rectangle loop of tube (w×d centreline)."""
    pts2 = round_outline([(-w / 2, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (-w / 2, d / 2)], r_corner, seg=3)
    pts2 = pts2 + [pts2[0]]
    pts = [(x, 0, z) for x, z in pts2] if plane == 'xz' else [(x, z, 0) for x, z in pts2]
    return tube(pts, r_tube, seg=seg, caps=False)

part('Props__wheelchair_seatFrame', f_box(0.5, 0.04, 0.48),
     lambda: F(tube_loop(0.46, 0.44, 0.05, 0.02) + rod_x(0.46, 0.014, seg=10).move(z=0.0), f_box(0.5, 0.04, 0.48)))
part('Props__wheelchair_cushion', f_box(0.42, 0.08, 0.42),
     lambda: F(puffy(0.42, 0.08, 0.42, dome=0.25, lump=0.002, seed=3), f_box(0.42, 0.08, 0.42)), smooth=180)
part('Props__wheelchair_back', f_box(0.42, 0.55, 0.05),
     lambda: F(bend(puffy_z(0.42, 0.55, 0.04, dome=0.25), 0.25, 'y', 'x'), f_box(0.42, 0.55, 0.05)), smooth=180)
part('Props__wheelchair_backFrame', f_box(0.5, 0.62, 0.03),
     lambda: F(tube_loop(0.47, 0.59, 0.06, 0.015, plane='xy') + rod_x(0.47, 0.011, seg=10).move(y=-0.05),
               f_box(0.5, 0.62, 0.03)))
part('Props__wheelchair_armPad', f_box(0.06, 0.04, 0.32),
     lambda: F(puffy(0.06, 0.04, 0.32, dome=0.2, n=3), f_box(0.06, 0.04, 0.32)), smooth=180)
part('Props__wheelchair_armPost', f_box(0.02, 0.22, 0.02), lambda: rod_y(0.22, 0.01, seg=12))


def wc_grip():
    prof = [(0, -0.04), (0.017, -0.04), (0.02, -0.036), (0.02, -0.031)]
    for i in range(4):
        y = -0.024 + i * 0.015
        prof += [(0.0182, y), (0.0198, y + 0.0075)]
    prof += [(0.02, 0.034), (0.017, 0.04), (0, 0.04)]
    return lathe(prof, seg=10)

part('Props__wheelchair_grip', f_cyl(0.02, 0.02, 0.08), wc_grip, smooth=60)
part('Props__wheelchair_handlePost', f_cyl(0.018, 0.018, 0.22), lambda: rod_y(0.22, 0.018, seg=14))


def wc_tire():
    # pneumatic tyre with tread blocks, ring in the XY plane
    prof = [(0.25, -0.022), (0.26, -0.03), (0.297, -0.03), (0.31, -0.018), (0.31, 0.018), (0.297, 0.03),
            (0.26, 0.03), (0.25, 0.022), (0.25, -0.022)]
    s = weld(lathe(prof, seg=48, cap_bottom=False, cap_top=False))

    def tread(p):
        r = math.hypot(p.x, p.z)
        if r > 0.305:
            k = round(math.atan2(p.x, p.z) / (TAU / 48))
            if k % 2:
                p.x *= 0.985
                p.z *= 0.985
        return p
    return s.warp(tread).rot(x=math.pi / 2)

part('Props__wheelchair_tire', f_torus(0.28, 0.03), wc_tire, smooth=50)
part('Props__wheelchair_pushRim', f_torus(0.31, 0.012), lambda: torus(0.31, 0.012, seg=48, tseg=6), smooth=60)
part('Props__wheelchair_spoke', f_box(0.005, 0.55, 0.005), lambda: cyl(0.0025, 0.0025, 0.55, seg=6, bevel_r=0))
part('Props__wheelchair_hub', f_cyl(0.04, 0.04, 0.06),
     lambda: lathe([(0, -0.03), (0.022, -0.03), (0.024, -0.027), (0.04, -0.024), (0.04, -0.018), (0.02, -0.015),
                    (0.018, 0.015), (0.04, 0.018), (0.04, 0.024), (0.024, 0.027), (0.022, 0.03), (0, 0.03)], seg=16))
part('Props__wheelchair_caster', f_torus(0.08, 0.02),
     lambda: lathe([(0, -0.012), (0.02, -0.012), (0.022, -0.008), (0.058, -0.008), (0.064, -0.016), (0.09, -0.02),
                    (0.1, -0.012), (0.1, 0.012), (0.09, 0.02), (0.064, 0.016), (0.058, 0.008), (0.022, 0.008),
                    (0.02, 0.012), (0, 0.012)], seg=22).rot(x=math.pi / 2), smooth=50)
part('Props__wheelchair_fork', f_box(0.02, 0.12, 0.02),
     lambda: F(rod_y(0.08, 0.008, seg=10).move(y=0.02) + cyl(0.01, 0.01, 0.014, seg=10).move(y=0.053)
               + rbox(0.02, 0.05, 0.008, r=0.003, seg=1).move(y=-0.035), f_box(0.02, 0.12, 0.02)))


def wc_footrest():
    plate = rbox(0.14, 0.012, 0.09, r=0.004, seg=1).move(y=-0.009)
    ribs = sum((rbox(0.12, 0.005, 0.007, r=0.002, seg=1).move(y=-0.001, z=-0.024 + i * 0.016) for i in range(4)), Shape())
    hinge = rod_x(0.14, 0.009, seg=10).move(y=0.006, z=-0.036)
    return F(plate + ribs + hinge, f_box(0.14, 0.03, 0.09))

part('Props__wheelchair_footrest', f_box(0.14, 0.03, 0.09), wc_footrest)
part('Props__wheelchair_sideBar', f_box(0.03, 0.7, 0.03), lambda: rod_y(0.7, 0.015, seg=14))
part('Props__wheelchair_crossBrace', f_box(0.48, 0.03, 0.03), lambda: rod_x(0.48, 0.015, seg=14))


# ------------------------------------------------------------- baby doll --
def doll_body():
    # chubby toddler torso lying along Y: neck at -Y (toward the head), belly on +X
    prof = [(0, -0.19), (0.03, -0.19), (0.035, -0.175), (0.06, -0.16), (0.075, -0.13), (0.082, -0.08),
            (0.088, 0.0), (0.09, 0.07), (0.085, 0.14), (0.06, 0.18), (0, 0.19)]
    s = lathe(prof, seg=18)
    s = s.warp(lambda p: Vector((p.x + (0.014 * math.exp(-(p.y / 0.07) ** 2) if p.x > 0 else 0), p.y, p.z)))
    return F(s, f_capsule(0.09, 0.2))

part('Props__doll_body', f_capsule(0.09, 0.2), doll_body, smooth=180)


def doll_head():
    # oversized baby cranium; the face looks along +X (where the eye
    # sockets are placed): cheeks, button nose, shallow sockets
    s = sphere(1, seg=22, rings=13)

    def face(p):
        if p.x > 0:
            ay = p.y
            p.x += 0.07 * math.exp(-((ay + 0.25) / 0.25) ** 2) * math.exp(-((abs(p.z) - 0.4) / 0.25) ** 2)   # cheeks
            p.x += 0.08 * math.exp(-(p.z / 0.12) ** 2 - ((ay + 0.02) / 0.12) ** 2)                          # nose
            p.x -= 0.1 * math.exp(-((abs(p.z) - 0.33) / 0.14) ** 2 - ((ay - 0.22) / 0.14) ** 2)            # sockets
        return p
    return F(s.warp(face), f_sphere(0.09))

part('Props__doll_head', f_sphere(0.09), doll_head, smooth=180)
part('Props__doll_eye', f_sphere(0.02), lambda: sphere(0.02, seg=10, rings=6), smooth=180)
part('Props__doll_arm', f_capsule(0.028, 0.16),
     lambda: F(lathe([(0, -0.108), (0.022, -0.1), (0.028, -0.07), (0.026, -0.02), (0.022, 0.012), (0.018, 0.04),
                      (0.022, 0.06), (0.024, 0.08), (0.018, 0.1), (0, 0.108)], seg=12), f_capsule(0.028, 0.16)), smooth=180)
part('Props__doll_leg', f_capsule(0.032, 0.14),
     lambda: F(lathe([(0, -0.102), (0.026, -0.095), (0.032, -0.06), (0.03, -0.01), (0.024, 0.03), (0.02, 0.05),
                      (0, 0.056)], seg=12) + ellipsoid(0.022, 0.03, 0.028, seg=12).move(y=0.072, z=0.004),
               f_capsule(0.032, 0.14)), smooth=180)


# -------------------------------------------------------------- suitcase --
def suitcase_shell():
    """Open suitcase half: rounded shell, 1.5 cm walls, opening toward +Y."""
    s = rbox(0.55, 0.14, 0.4, r=0.02, seg=2)
    s = cut(s, rbox(0.52, 0.14, 0.37, r=0.012, seg=1).move(y=0.06))
    groove = cut(box(0.6, 0.006, 0.45), box(0.54, 0.01, 0.39))
    return cut(s, groove.move(y=-0.02))

part('Props__suitcase_base', f_box(0.55, 0.14, 0.4), suitcase_shell)
part('Props__suitcase_lid', f_box(0.55, 0.14, 0.4), lambda: suitcase_shell().rot(x=math.pi))


def cloth_wad(seed=0):
    s = subdivide(box(0.1, 0.06, 0.09), 2)
    s = s.warp(lambda p: Vector((p.x, p.y + 0.012 * math.sin(p.x * 70 + seed) * math.cos(p.z * 50), p.z)))
    return F(noisy(s, 0.006, scale=25, seed=seed), f_sphere(0.06))

part('Props__suitcase_clothes', f_sphere(0.06), lambda: cloth_wad(4), smooth=180)


# ---------------------------------------------------------- broken mirror --
def mirror_frame():
    # carved frame: rounded outer moulding, a lower inner lip, backing board
    mould = ring_u(0.9, 1.4, 0.8, 1.3, 0.06, r=0.01, seg=2, open_bottom=False)
    lip = ring_u(0.81, 1.31, 0.78, 1.28, 0.04, r=0.004, seg=1, open_bottom=False).move(z=-0.01)
    back = box(0.8, 1.3, 0.02).move(z=0.012)
    return mould + lip + back

part('Props__mirror_frame', f_box(0.9, 1.4, 0.06), mirror_frame)


# ---------------------------------------------------------- CRT on floor --
part('Props__crtTv_bezel', f_box(0.72, 0.52, 0.02),
     lambda: cut(rbox(0.72, 0.52, 0.02, r=0.008, seg=2), bevel(box(0.68, 0.48, 0.02), 0.012, seg=1, angle=80).move(z=0.012)))
part('Props__crtTv_panel', f_box(0.15, 0.5, 0.02), lambda: rbox(0.15, 0.5, 0.02, r=0.006, seg=2))


def tv_knob():
    # fluted rotary knob, face on +Y
    s = lathe([(0, -0.015), (0.032, -0.015), (0.032, -0.01), (0.029, -0.007), (0.028, 0.011), (0.025, 0.015), (0, 0.015)], seg=24)

    def flutes(p):
        r = math.hypot(p.x, p.z)
        if r > 0.0275 and -0.008 < p.y < 0.0125:
            k = round(math.atan2(p.x, p.z) / (TAU / 24))
            if k % 2:
                p.x *= 0.92
                p.z *= 0.92
        return p
    return s.warp(flutes)

part('Props__crtTv_knob', f_cyl(0.028, 0.032, 0.03), tv_knob)
part('Props__crtTv_channelWindow', f_box(0.06, 0.03, 0.02),
     lambda: cut(rbox(0.06, 0.03, 0.02, r=0.004, seg=1), box(0.046, 0.018, 0.01).move(z=0.01)))
part('Props__crtTv_grilleSlot', f_box(0.06, 0.008, 0.01), lambda: rbox(0.06, 0.008, 0.01, r=0.0035, seg=2))


def tv_antenna():
    # telescoping rod: three sections, collars, ball tip
    s = cyl(0.0045, 0.0045, 0.14, seg=8, bevel_r=0.0005).move(y=-0.13)
    s = s + cyl(0.0035, 0.0035, 0.14, seg=8, bevel_r=0.0005).move(y=0.0)
    s = s + cyl(0.0025, 0.0025, 0.12, seg=8, bevel_r=0.0005).move(y=0.12)
    s = s + cyl(0.006, 0.006, 0.012, seg=8, bevel_r=0.001).move(y=-0.194)
    s = s + sphere(0.006, seg=8, rings=5).move(y=0.194)
    return F(s, f_cyl(0.006, 0.006, 0.4))

part('Props__crtTv_antenna', f_cyl(0.006, 0.006, 0.4), tv_antenna, smooth=60)


# ------------------------------------------------------------------ books --
def book_cover():
    # hardcover: two boards and a rounded spine (at +Z); the page block is
    # a separate paper-coloured mesh that now shows on the three open edges
    w, h, d = 0.22, 0.045, 0.17
    bt = h * 0.15
    boards = Shape()
    for sy in (-1, 1):
        boards = boards + rbox(w, bt, d - 0.004, r=0.0012, seg=1).move(y=sy * (h / 2 - bt / 2), z=-0.002)
    spine = cut(cyl(h / 2, h / 2, w, seg=14, bevel_r=0.001).rot(z=math.pi / 2).scale(1, 1, 0.2),
                box(w + 0.01, h, h).move(z=-h / 2)).move(z=d / 2 - h * 0.1)
    return F(boards + spine, f_box(w, h, d))

part('Props__book_cover', f_box(0.22, 0.045, 0.17), book_cover)


def book_pages():
    w, h, d = 0.21, 0.0315, 0.165
    s = grid_box(w, h, d, 4)

    def edge(p):
        u = p.y / (h / 2)
        if p.z < -d / 2 + 1e-4:                 # fore-edge a little concave
            p.z += 0.0025 * (1 - u * u)
        return p
    s = s.warp(edge)
    grooves = [box(w + 0.01, 0.0008, 0.004).move(y=-h / 2 + h * k / 5, z=-d / 2) for k in range(1, 5)]
    return F(cut(s, *grooves), f_box(w, h, d))

part('Props__book_pages', f_box(0.21, 0.0315, 0.165), book_pages)
part('Props__book_ribbon', f_box(0.015, 0.105, 0.005),
     lambda: prism([(-0.0075, 0.0525), (0.0075, 0.0525), (0.0075, -0.0525), (0.0, -0.045), (-0.0075, -0.0525)], 0.005))


def book_standing():
    # closed hardcover standing up: boards overhang a recessed page block,
    # rounded spine along the -X edge
    w, h, d = 0.16, 0.22, 0.035
    s = rbox(w, h, d, r=0.0015, seg=1)
    s = cut(s, box(w, h - 0.008, d * 0.72).move(x=0.004 + 0.0, y=0))
    s = s + rbox(w - 0.008, h - 0.008, d * 0.72, r=0.001, seg=1).move(x=-0.004 + 0.0)
    spine = cyl(d / 2, d / 2, h, seg=12, bevel_r=0.001).scale(0.25, 1, 1).move(x=-w / 2 + 0.002)
    return F(s + spine, f_box(w, h, d))

part('Props__book_standing', f_box(0.16, 0.22, 0.035), book_standing)


# ------------------------------------------------------------- coat rack --
part('Props__coatRack_pole', f_cyl(0.025, 0.025, 1.7),
     lambda: lathe([(0, -0.85), (0.022, -0.85), (0.025, -0.83), (0.02, -0.8), (0.018, 0.7), (0.022, 0.72),
                    (0.018, 0.74), (0.018, 0.78), (0.025, 0.81), (0.021, 0.84), (0.01, 0.85), (0, 0.85)], seg=14), smooth=50)
part('Props__coatRack_base', f_cyl(0.22, 0.22, 0.04),
     lambda: lathe([(0, -0.02), (0.22, -0.02), (0.22, -0.008), (0.205, 0.0), (0.19, 0.004), (0.15, 0.008),
                    (0.06, 0.014), (0.045, 0.02), (0, 0.02)], seg=28), smooth=40)
part('Props__coatRack_hook', f_torus(0.06, 0.012), lambda: torus(0.06, 0.012, seg=24, tseg=8))
part('Props__coatRack_coat', f_box(0.4, 0.7, 0.1), lambda: hanging_coat(0.4, 0.7, 0.1, seed=3), smooth=180)


# ------------------------------------------------------- fluorescent tube --
part('Props__tube_endCap', f_cyl(0.05, 0.05, 0.06),
     lambda: lathe([(0, -0.03), (0.042, -0.03), (0.048, -0.027), (0.05, -0.02), (0.05, -0.01), (0.047, -0.005),
                    (0.047, 0.005), (0.05, 0.01), (0.05, 0.02), (0.048, 0.027), (0.042, 0.03), (0, 0.03)], seg=20))


# ------------------------------------------------------------------- pile --
part('Props__pile_block', f_box(0.2, 0.125, 0.2),
     lambda: F(noisy(bevel(grid_box(0.2, 0.125, 0.2, 4), 0.006, seg=1, angle=50), 0.006, scale=10, seed=11),
               f_box(0.2, 0.125, 0.2)), smooth=50)
part('Props__pile_lump', f_sphere(0.09),
     lambda: F(noisy(subdivide(box(0.18, 0.14, 0.16), 2), 0.02, scale=12, seed=12), f_sphere(0.09)), smooth=70)


# ------------------------------------------------------------------ plants --
def leaf_outline(L, W, n=8):
    """Leaf outline along +X (tip at +L/2), widest a third of the way."""
    top = []
    for i in range(n + 1):
        t = i / n
        x = -L / 2 + L * t
        wdt = W / 2 * math.sin(math.pi * t ** 0.8) ** 0.9
        top.append((x, wdt))
    bot = [(x, -y) for x, y in reversed(top[1:-1])]
    return top + bot


def flat_leaf(L, W, t, curl, seed):
    s = prism(leaf_outline(L, W), t).rot(x=math.pi / 2)          # outline in XZ, thickness on Y
    s = s.warp(lambda p: Vector((p.x, p.y + curl * ((p.x / (L / 2)) ** 2 + (p.z / (W / 2)) ** 2), p.z)))
    return F(s, f_box(L, t, W))

part('Props__plant_deadLeaf', f_box(0.05, 0.002, 0.04), lambda: flat_leaf(0.05, 0.04, 0.002, 0.0012, 1), smooth=70)
part('Props__plant_fallenLeaf', f_box(0.11, 0.003, 0.065), lambda: flat_leaf(0.11, 0.065, 0.003, 0.002, 2), smooth=70)
part('Props__pottedPlant_saucer', f_cyl(0.19, 0.17, 0.02),
     lambda: vessel([(0.17, -0.01), (0.185, -0.004), (0.19, 0.008)], 0.008, seg=28, floor=0.004), smooth=45)


def terracotta(rt, rb, h, rings=3, seg=24):
    prof = [(0, -h / 2), (rb - 0.006, -h / 2), (rb, -h / 2 + 0.005), (rb, -h / 2 + 0.014), (rb - 0.004, -h / 2 + 0.02)]
    for i in range(1, rings * 2 + 1):
        t = i / (rings * 2 + 1)
        r = rb + (rt - rb) * t - (0.0025 if i % 2 else 0.0)
        prof.append((r, -h / 2 + h * t))
    prof += [(rt, h / 2 - 0.004), (rt - 0.004, h / 2), (0, h / 2)]
    return lathe(prof, seg=seg)

part('Props__pottedPlant_pot', f_cyl(0.16, 0.12, 0.28), lambda: terracotta(0.16, 0.12, 0.28), smooth=45)
part('Props__pottedPlant_rim', f_cyl(0.17, 0.17, 0.03),
     lambda: lathe([(0, -0.015), (0.155, -0.015), (0.165, -0.013), (0.17, -0.006), (0.17, 0.006), (0.165, 0.013),
                    (0.155, 0.015), (0.14, 0.015), (0.13, 0.01), (0, 0.01)], seg=24), smooth=50)


def soil_disc(r, h, seed):
    s = lathe([(0, h / 2), (r * 0.3, h / 2), (r * 0.6, h * 0.4), (r * 0.85, h * 0.2), (r, 0.0), (r, -h / 2), (0, -h / 2)], seg=20)
    s = s.warp(lambda p: Vector((p.x, p.y + (0.35 * h * noise.noise(Vector((p.x * 30 + seed, p.z * 30, 0)))
                                            if p.y > 0 else 0), p.z)))
    return F(s, f_cyl(r, r, h))

part('Props__pottedPlant_soil', f_cyl(0.16, 0.16, 0.02), lambda: soil_disc(0.16, 0.02, 1), smooth=70)
part('Props__pottedPlant_pebble', f_sphere(0.011),
     lambda: F(noisy(sphere(0.011, seg=8, rings=5).scale(1, 0.7, 0.9), 0.0015, scale=200, seed=3), f_sphere(0.011)), smooth=80)


def grass_blade():
    # long leaf blade: widest low down, pointed tip, folded along the midrib,
    # arching back a little
    L, W = 0.4, 0.022
    s = slab(W, L, 0.0015, 2, 10)

    def shape(p):
        t = (p.y + L / 2) / L
        wdt = math.sin(math.pi * min(1.0, 0.15 + t * 0.85)) ** 0.7 * (1 - 0.25 * t)
        p.x *= max(0.04, wdt)
        p.z += 0.0018 * (1 - abs(p.x) / (W / 2))
        return p
    s = bend(s.warp(shape), 0.18, axis='x', along='y')
    return F(s, f_box(W, L, 0.005))

part('Props__pottedPlant_blade', f_box(0.022, 0.4, 0.005), grass_blade, smooth=70)
part('Props__pottedPlant_flowerCenter', f_sphere(0.015),
     lambda: F(noisy(sphere(0.015, seg=10, rings=6).scale(1, 0.6, 1), 0.0012, scale=300, seed=2), f_sphere(0.015)), smooth=80)


def petal():
    s = prism(leaf_outline(0.03, 0.02, 6), 0.003).rot(z=math.pi / 2).rot(x=math.pi / 2)
    s = s.warp(lambda p: Vector((p.x, p.y + 0.004 * (p.x / 0.01) ** 2, p.z)))
    return F(s, f_box(0.02, 0.008, 0.03))

part('Props__pottedPlant_petal', f_box(0.02, 0.008, 0.03), petal, smooth=70)

part('Props__floorPlant_pot', f_cyl(0.25, 0.19, 0.4), lambda: terracotta(0.25, 0.19, 0.4, rings=2, seg=28), smooth=45)
part('Props__floorPlant_rim', f_cyl(0.27, 0.26, 0.03),
     lambda: lathe([(0, -0.015), (0.255, -0.015), (0.262, -0.012), (0.268, -0.004), (0.27, 0.006), (0.265, 0.013),
                    (0.255, 0.015), (0.24, 0.015), (0.23, 0.01), (0, 0.01)], seg=28), smooth=50)
part('Props__floorPlant_soil', f_cyl(0.23, 0.23, 0.02), lambda: soil_disc(0.23, 0.02, 4), smooth=70)


def gnarled(rt, rb, h, seed, knots=3):
    # wobbling tapered wood with bark noise and a few knots
    n = 14
    prof = [(0, -h / 2)] + [(rb + (rt - rb) * i / n, -h / 2 + h * i / n) for i in range(n + 1)] + [(0, h / 2)]
    s = lathe(prof, seg=9)
    rng = random.Random(seed)
    ks = [(rng.uniform(-0.4, 0.4) * h, rng.uniform(0, TAU)) for _ in range(knots)]

    def wob(p):
        t = p.y / h
        off = Vector((math.sin(t * 7 + seed) * 0.12 * rb, 0, math.cos(t * 5 + seed * 2) * 0.12 * rb))
        r = math.hypot(p.x, p.z)
        if r > 1e-6:
            a = math.atan2(p.x, p.z)
            for ky, ka in ks:
                k = math.exp(-((p.y - ky) / (0.04 * h + 0.01)) ** 2) * max(0.0, math.cos(a - ka)) ** 4
                p.x += p.x / r * 0.35 * rb * k
                p.z += p.z / r * 0.35 * rb * k
        return p + off
    s = noisy(s.warp(wob), 0.08 * rb, scale=80 / h, seed=seed)
    return F(s, f_cyl(rt, rb, h))

part('Props__floorPlant_trunk', f_cyl(0.025, 0.04, 1.4), lambda: gnarled(0.025, 0.04, 1.4, 1, 4), smooth=70)
part('Props__floorPlant_branchA', f_cyl(0.014, 0.022, 0.5), lambda: gnarled(0.014, 0.022, 0.5, 2, 2), smooth=70)
part('Props__floorPlant_branchB', f_cyl(0.014, 0.02, 0.4), lambda: gnarled(0.014, 0.02, 0.4, 3, 2), smooth=70)


# ----------------------------------------------------------------- window --
def window_frame():
    # stepped casing around the pane opening (the pane quad covers the middle)
    outer = ring_u(1.5, 1.4, 1.44, 1.34, 0.08, r=0.01, seg=2, open_bottom=False)
    inner = ring_u(1.445, 1.345, 1.4, 1.3, 0.07, r=0.004, seg=1, open_bottom=False).move(z=-0.005)
    back = box(1.42, 1.32, 0.02).move(z=0.02)
    return outer + inner + back

part('Props__window_frame', f_box(1.5, 1.4, 0.08), window_frame)
part('Props__window_sill', f_box(1.6, 0.06, 0.14),
     lambda: cut(bevel(side_prism([(-0.07, -0.03), (-0.07, 0.03), (0.055, 0.03), (0.066, 0.024), (0.07, 0.008),
                                   (0.067, -0.012), (0.06, -0.02), (0.058, -0.03)], 1.6), 0.004, seg=1, angle=50),
                 box(1.62, 0.01, 0.006).move(y=-0.03, z=0.045)))
MUNTIN = [(-0.02, -0.015), (0.02, -0.015), (0.02, 0.0), (0.012, 0.01), (0.006, 0.015), (-0.006, 0.015), (-0.012, 0.01), (-0.02, 0.0)]
part('Props__window_muntinV', f_box(0.04, 1.3, 0.03), lambda: prism(MUNTIN, 1.3).rot(x=math.pi / 2))
part('Props__window_muntinH', f_box(1.4, 0.04, 0.03), lambda: side_prism([(z, y) for y, z in MUNTIN], 1.4))


# --------------------------------------------------------------- elevator --
def casing(W, H, ow, oh, d, band=0.02, dent=0.0, seed=0):
    """Steel surround: U-frame around an ow×oh opening from the floor, a
    raised face band, a far back panel; `dent` adds battering."""
    base = ring_u(W, H, ow, oh, d - 0.02, r=0.006, seg=1).move(z=-0.01)
    face = ring_u(W - 0.02, H - 0.01, ow + 2 * band, oh + band, 0.03, r=0.008, seg=2).move(y=-0.005, z=d / 2 - 0.015)
    back = box(ow + 0.02, oh + 0.01, 0.01).move(y=-H / 2 + (oh + 0.01) / 2, z=-d / 2 + 0.005)
    s = base + face + back
    if dent:
        s = noisy(s, dent, scale=3, seed=seed)
    return F(s, f_box(W, H, d))

part('Props__elevator_frame', f_box(1.6, 2.4, 0.15), lambda: casing(1.6, 2.4, 1.44, 2.1, 0.15))


def elevator_door(w, h, dent=0.0, seed=0):
    s = rbox(w, h, 0.05, r=0.004, seg=1)
    s = cut(s, box(w + 0.01, 0.004, 0.006).move(y=-h / 2 + 0.3, z=0.025),
            box(w + 0.01, 0.004, 0.006).move(y=h / 2 - 0.12, z=0.025))
    if dent:
        s = noisy(subdivide(grid_box(w, h, 0.05, 6), 0) if False else s, dent, scale=4, seed=seed)
    return F(s, f_box(w, h, 0.05))

part('Props__elevator_door', f_box(0.7, 2.05, 0.05), lambda: elevator_door(0.7, 2.05))
part('Props__elevator_display', f_box(0.5, 0.12, 0.02),
     lambda: cut(rbox(0.5, 0.12, 0.02, r=0.008, seg=2), box(0.46, 0.09, 0.01).move(z=0.01)))


def call_panel():
    s = bevel(box(0.15, 0.28, 0.027), 0.005, seg=1).move(z=-0.0015)
    for sy in (-1, 1):
        s = s + screw_head(0.005, 0.0025).rot(x=math.pi / 2).move(y=sy * 0.12, z=0.012)
    return s

part('Props__elevator_callPanel', f_box(0.15, 0.28, 0.03), call_panel)
part('Props__elevator_button', f_cyl(0.03, 0.03, 0.01),
     lambda: lathe([(0, -0.005), (0.03, -0.005), (0.03, -0.002), (0.028, 0.0), (0.023, 0.0), (0.022, 0.003),
                    (0.02, 0.0045), (0.012, 0.005), (0, 0.005)], seg=20))
part('Props__elevator_strip', f_box(1.5, 0.04, 0.03),
     lambda: side_prism([(-0.015, -0.02), (-0.015, 0.02), (0.005, 0.02), (0.013, 0.014), (0.015, 0.0),
                         (0.013, -0.014), (0.005, -0.02)], 1.5))


# ------------------------------------------------------------ paper pile --
def paper_sheet():
    s = slab(0.21, 0.27, 0.0008, 6, 8).rot(x=-math.pi / 2)

    def wave(p):
        u, v = p.x / 0.105, p.z / 0.135
        lift = 0.0012 * math.sin(u * 2.5 + v * 1.7) + 0.005 * max(0.0, u + v - 1.1) ** 2
        return Vector((p.x, p.y - 0.0016 + lift, p.z))
    return s.warp(wave)

part('Props__paper_sheet', f_box(0.21, 0.004, 0.27), paper_sheet, smooth=70)


def paper_curled():
    s = bend(slab(0.19, 0.24, 0.0008, 8, 6), 0.15, axis='y', along='x')
    lo, hi = s.bounds()
    return s.move(x=-(lo.x + hi.x) / 2, z=-(lo.z + hi.z) / 2)

part('Props__paper_curledSheet', f_box(0.19, 0.24, 0.005), paper_curled, smooth=70)


def paper_clip():
    pts = [(0.006, -0.01, 0), (0.006, 0.012, 0), (0.0, 0.018, 0), (-0.006, 0.012, 0), (-0.006, -0.016, 0),
           (0.003, -0.023, 0), (0.012, -0.016, 0), (0.012, 0.016, 0)]
    return F(tube(pts, 0.003, seg=6, smooth=1), f_torus(0.02, 0.003))

part('Props__paper_clip', f_torus(0.02, 0.003), paper_clip, smooth=60)


# ------------------------------------------------------------- floor hole --
part('Props__floorHole_chunk', f_box(0.14, 0.06, 0.14),
     lambda: F(noisy(grid_box(0.14, 0.06, 0.14, 3), 0.014, scale=18, seed=6), f_box(0.14, 0.06, 0.14)), flat=True)


# ------------------------------------------------------- weather extras --
def tile_shard():
    rng = random.Random(9)
    pts = []
    for i in range(9):
        a = TAU * i / 9 + rng.uniform(-0.2, 0.2)
        r = rng.uniform(0.7, 1.0)
        pts.append((math.cos(a) * r, math.sin(a) * r))
    s = prism(pts, 0.02, bevel_r=0.002).rot(x=math.pi / 2)
    s = s.warp(lambda p: Vector((p.x, p.y - (0.004 if p.y > 0 and noise.noise(Vector((p.x * 9, p.z * 9, 1))) > 0.3 else 0), p.z)))
    return F(s, f_box(0.375, 0.02, 0.3))

part('Props__derelict_shard', f_box(0.375, 0.02, 0.3), tile_shard, flat=True)


def rebar():
    prof = [(0, -0.225)]
    n = 22
    for i in range(n + 1):
        y = -0.222 + 0.444 * i / n
        prof += [(0.0072, y), (0.008, y + 0.006)] if i < n else [(0.0072, y)]
    prof.append((0, 0.225))
    s = bend(lathe(prof, seg=6), 0.08, axis='z', along='y')
    lo, hi = s.bounds()
    return s.move(x=-(lo.x + hi.x) / 2)

part('Props__derelict_rebar', f_cyl(0.008, 0.008, 0.45), rebar, smooth=60)

ARC_R, ARC_T, ARC_A = 0.12, 0.014, math.pi * 1.3
ARC_FRAME = f_bounds((-(ARC_R + ARC_T), math.sin(ARC_A) * (ARC_R + ARC_T), -ARC_T), (ARC_R + ARC_T, ARC_R + ARC_T, ARC_T))


def slack_cable():
    pts = []
    for i in range(25):
        a = ARC_A * i / 24
        rr = ARC_R * (1 + 0.04 * math.sin(i * 1.3))
        pts.append((rr * math.cos(a), rr * math.sin(a), 0.004 * math.sin(i * 0.9)))
    return F(tube(pts, ARC_T, seg=8), ARC_FRAME)

part('Props__electric_cable', ARC_FRAME, slack_cable, smooth=60)


def wire_tip():
    # stripped copper strands, twisted, fanning out at the end
    s = Shape()
    for k in range(5):
        a0 = TAU * k / 5
        pts = []
        for i in range(5):
            t = i / 4
            a = a0 + t * 1.6
            r = 0.005 + 0.004 * t ** 3
            pts.append((r * math.cos(a), -0.05 + 0.1 * t, r * math.sin(a)))
        s = s + tube(pts, 0.003, seg=4)
    return F(s, f_cyl(0.012, 0.012, 0.10))

part('Props__electric_wireTip', f_cyl(0.012, 0.012, 0.10), wire_tip, smooth=60)


# ---------------------------------------------------------------- candles --
def candle(rt, rb, h, seed=0):
    # pillar candle: melt pool sunk inside a soft rim, a few wax drips
    s = lathe([(0, -h / 2), (rb - 0.002, -h / 2), (rb, -h / 2 + 0.003), (rt + 0.0005, h / 2 - 0.006), (rt, h / 2),
               (rt - 0.004, h / 2 - 0.001), (rt - 0.01, h / 2 - 0.007), (0, h / 2 - 0.009)], seg=16)
    rng = random.Random(seed)
    for _ in range(3):
        a = rng.uniform(0, TAU)
        ln = rng.uniform(0.02, 0.045)
        d = ellipsoid(0.0045, ln / 2, 0.003, seg=8).move(y=h / 2 - ln / 2 - 0.002)
        s = s + d.move(x=math.sin(a) * (rt - 0.0005), z=math.cos(a) * (rt - 0.0005))
    return F(s, f_cyl(rt, rb, h))

part('Props__candle_pillar', f_cyl(0.035, 0.04, 0.16), lambda: candle(0.035, 0.04, 0.16, 1), smooth=60)
part('Props__safeRoom_candle', f_cyl(0.028, 0.032, 0.14), lambda: candle(0.028, 0.032, 0.14, 2), smooth=60)


def wick(w, h):
    s = tube([(0, -h / 2, 0), (0, h * 0.1, 0), (w * 0.15, h / 2 - w * 0.4, 0)], w * 0.25, seg=5, smooth=1)
    s = s + sphere(w * 0.4, seg=6, rings=4).move(x=w * 0.15, y=h / 2 - w * 0.4)
    return F(s, f_box(w, h, w))

part('Props__candle_wick', f_box(0.006, 0.022, 0.006), lambda: wick(0.006, 0.022), smooth=70)
part('Props__safeRoom_wick', f_box(0.005, 0.02, 0.005), lambda: wick(0.005, 0.02), smooth=70)


# ----------------------------------------------------------- light switch --
def switch_plate():
    # cream wall plate with a sloped bevel face and the rocker opening
    s = prism(rounded_rect(0.10, 0.16, 0.006, seg=2), 0.012)
    s = taper(s, top=0.93, along='z', axes='xy')
    s = F(s, f_box(0.10, 0.16, 0.012))
    return cut(s, prism(rounded_rect(0.062, 0.102, 0.004, seg=2), 0.03))

part('Props__switch_plate', f_box(0.10, 0.16, 0.012), switch_plate)
part('Props__switch_housing', f_box(0.06, 0.10, 0.006),
     lambda: cut(rbox(0.06, 0.10, 0.006, r=0.0015, seg=1), rbox(0.052, 0.086, 0.006, r=0.002, seg=1).move(z=0.004)))


def switch_rocker():
    prof = [(-0.01, -0.04), (-0.01, 0.04)]
    for i in range(9):
        y = 0.04 - 0.08 * i / 8
        prof.append((0.01 - 0.005 * (y / 0.04) ** 2, y))
    s = bevel(side_prism(prof, 0.05), 0.002, seg=1)
    return F(s, f_box(0.05, 0.08, 0.020))

part('Props__switch_rocker', f_box(0.05, 0.08, 0.020), switch_rocker, smooth=40)


# -------------------------------------------------------------- lore note --
def lore_sheet():
    # a page lying on the floor, humped a little in the middle so the ink
    # still sits just above it, edges settling toward the floor; the top
    # edge is torn
    s = slab(0.26, 0.32, 0.0015, 8, 10)
    rng = random.Random(21)

    def torn(p):
        if p.y > 0.159:
            p.y -= rng.uniform(0.0, 0.012)
        return p
    s = s.warp(torn).rot(x=-math.pi / 2)

    def hump(p):
        r2 = (p.x / 0.13) ** 2 + (p.z / 0.16) ** 2
        return Vector((p.x, p.y + 0.00225 - 0.0048 * min(1.0, r2 / 2), p.z))
    return s.warp(hump)

part('Props__loreNote_sheet', f_box(0.26, 0.006, 0.32), lore_sheet, smooth=70)


def lore_curl():
    s = bend(slab(0.1, 0.1, 0.0012, 4, 4).rot(x=-math.pi / 2), 0.1, axis='z', along='x')
    lo, hi = s.bounds()
    return s.move(x=-(lo.x + hi.x) / 2, y=-(lo.y + hi.y) / 2)

part('Props__loreNote_curl', f_box(0.10, 0.005, 0.10), lore_curl, smooth=70)


# ------------------------------------------------------------------- pool --
part('Props__pool_curb', f_box(3.0, 0.06, 0.2),
     lambda: side_prism([(-0.1, -0.03), (0.1, -0.03), (0.1, 0.01), (0.094, 0.024), (0.08, 0.03), (-0.08, 0.03),
                         (-0.094, 0.024), (-0.1, 0.01)], 3.0))


def beach_ball():
    # six pillowed gores pinched in along their seams, small valve caps
    s = sphere(1, seg=30, rings=15)

    def gores(p):
        r = math.hypot(p.x, p.z)
        if r > 1e-6:
            phi = math.atan2(p.z, p.x)
            k = 1 - 0.05 * (1 - abs(math.sin(3 * phi))) ** 3 * r
            p.x *= k
            p.z *= k
        return p
    s = s.warp(gores)
    s = s + cyl(0.1, 0.1, 0.03, seg=10, bevel_r=0.01, bevel_seg=1).move(y=0.99) + cyl(0.1, 0.1, 0.03, seg=10, bevel_r=0.01, bevel_seg=1).move(y=-0.99)
    return F(s, f_sphere(0.34))

part('Props__pool_ball', f_sphere(0.34), beach_ball, smooth=70)
part('Props__pool_ballStripe', f_torus(0.341, 0.0204),
     lambda: weld(lathe([(0.33, -0.0204), (0.34, -0.0204), (0.343, -0.016), (0.343, 0.016), (0.34, 0.0204),
                         (0.33, 0.0204), (0.33, -0.0204)], seg=40, cap_bottom=False, cap_top=False)).rot(x=math.pi / 2), smooth=50)


def ladder_tube(L, r):
    return cyl(r, r, L, seg=16, bevel_r=r * 0.45, bevel_seg=2)

part('Props__poolLadder_rail', f_cyl(0.022, 0.022, 0.55), lambda: ladder_tube(0.55, 0.022), smooth=50)
part('Props__poolLadder_bend', f_cyl(0.022, 0.022, 0.18), lambda: ladder_tube(0.18, 0.022), smooth=50)
part('Props__poolLadder_railLow', f_cyl(0.022, 0.022, 0.65), lambda: ladder_tube(0.65, 0.022), smooth=50)
part('Props__poolLadder_grip', f_cyl(0.022, 0.022, 0.32), lambda: ladder_tube(0.32, 0.022), smooth=50)


def ladder_rung():
    prof = [(0, -0.15), (0.015, -0.15), (0.018, -0.146), (0.018, -0.1)]
    for i in range(6):
        y = -0.09 + i * 0.036
        prof += [(0.0168, y), (0.018, y + 0.006)]
    prof += [(0.018, 0.146), (0.015, 0.15), (0, 0.15)]
    return lathe(prof, seg=12)

part('Props__poolLadder_rung', f_cyl(0.018, 0.018, 0.30), ladder_rung, smooth=50)


# ----------------------------------------------------------- robot vacuum --
def vacuum_body():
    # disc robot: rounded top edge, a lid seam on top, a recess in the
    # front quarter where the separate bumper sits (it used to z-fight)
    s = lathe([(0, -0.04), (0.175, -0.04), (0.186, -0.036), (0.19, -0.028), (0.185, 0.03), (0.178, 0.038),
               (0.17, 0.04), (0.126, 0.04), (0.124, 0.037), (0.119, 0.037), (0.117, 0.04), (0, 0.04)], seg=40)
    pocket = lathe_arc([(0.176, -0.036), (0.22, -0.036), (0.22, 0.026), (0.176, 0.026)], -0.83, 0.83, seg=12)
    return cut(s, pocket)

part('Props__vacuum_body', f_cyl(0.18, 0.19, 0.08), vacuum_body, smooth=40)
part('Props__vacuum_ring', f_torus(0.16, 0.008),
     lambda: weld(lathe([(0.152, 0.008), (0.168, 0.008), (0.168, -0.004), (0.164, -0.008), (0.156, -0.008),
                         (0.152, -0.004), (0.152, 0.008)], seg=40, cap_bottom=False, cap_top=False)).rot(x=math.pi / 2))
part('Props__vacuum_panel', f_cyl(0.08, 0.08, 0.005), lambda: cyl(0.08, 0.08, 0.005, seg=28, bevel_r=0.0015, bevel_seg=1))
BUMPER_FRAME = f_bounds((-0.185 * math.sin(math.pi / 4), -0.03, 0.0), (0.185 * math.sin(math.pi / 4), 0.03, 0.185))
part('Props__vacuum_bumper', BUMPER_FRAME,
     lambda: lathe_arc([(0, -0.03), (0.176, -0.03), (0.182, -0.028), (0.185, -0.022), (0.185, 0.022), (0.182, 0.028),
                        (0.176, 0.03), (0, 0.03)], -math.pi / 4, math.pi / 4, seg=16), smooth=40)
part('Props__vacuum_brushHub', f_cyl(0.02, 0.02, 0.02),
     lambda: lathe([(0, -0.01), (0.02, -0.01), (0.02, 0.004), (0.016, 0.01), (0.006, 0.01), (0.005, 0.012), (0, 0.012)], seg=12))


def vacuum_roller():
    s = cyl(0.015, 0.015, 0.2, seg=10, bevel_r=0.002)
    for k in range(2):
        pts = [(0.0155 * math.cos(k * math.pi + t * TAU * 1.2), -0.098 + 0.196 * t, 0.0155 * math.sin(k * math.pi + t * TAU * 1.2))
               for t in (i / 15 for i in range(16))]
        s = s + tube(pts, 0.0045, seg=5)
    return F(s, f_cyl(0.02, 0.02, 0.20))

part('Props__vacuum_roller', f_cyl(0.02, 0.02, 0.20), vacuum_roller, smooth=60)


# -------------------------------------------------------------- safe room --
part('Props__safeRoom_benchTop', f_box(0.9, 0.05, 0.28),
     lambda: sum((rbox(0.9, 0.05, 0.088, r=0.007, seg=2).move(z=z) for z in (-0.096, 0.0, 0.096)), Shape()))


def bench_leg():
    # slab trestle leg with an arched cut-out between its two feet
    prof = [(-0.12, -0.21), (-0.07, -0.21)] + [(-0.07 * math.cos(t), -0.21 + 0.075 * math.sin(t)) for t in
                                               (math.pi * i / 8 for i in range(1, 8))] + [(0.07, -0.21), (0.12, -0.21), (0.12, 0.21), (-0.12, 0.21)]
    return bevel(side_prism(prof, 0.06), 0.005, seg=1, angle=40)

part('Props__safeRoom_benchLeg', f_box(0.06, 0.42, 0.24), bench_leg)


# ------------------------------------------------------- broken elevator --
part('Props__brokenElevator_frame', f_box(1.7, 2.5, 0.15), lambda: casing(1.7, 2.5, 1.35, 2.1, 0.15, dent=0.004, seed=5))


def battered_door(w, h, seed):
    s = grid_box(w, h, 0.05, 7)

    def beat(p):
        u, v = p.x / (w / 2), p.y / (h / 2)
        if p.z > 0:
            p.z -= 0.012 * math.exp(-((u - 0.2) / 0.5) ** 2 - ((v + 0.1) / 0.3) ** 2)    # kicked-in dent
        p.z += 0.03 * max(0.0, -v - 0.7) ** 2 * max(0.0, u)                        # bottom corner peeled out
        return p
    s = noisy(s.warp(beat), 0.004, scale=6, seed=seed)
    return F(s, f_box(w, h, 0.05))

part('Props__brokenElevator_doorL', f_box(0.65, 2.05, 0.05), lambda: battered_door(0.65, 2.05, 1), smooth=50)
part('Props__brokenElevator_doorR', f_box(0.30, 2.05, 0.05), lambda: battered_door(0.30, 2.05, 2), smooth=50)
part('Props__brokenElevator_wire', f_cyl(0.006, 0.006, 0.65),
     lambda: tube([(0.004 * math.sin(i * 0.8), 0.325 - 0.65 * i / 9, 0.003 * math.cos(i * 1.1)) for i in range(10)],
                  0.006, seg=6, smooth=1, r_end=0.0035), smooth=60)


# ----------------------------------------------------------------- corpse --
def skull():
    # cranium, brow, deep sockets (the black eye discs sit in them), nasal
    # hole, cheekbones and the upper teeth row; the jaw is separate
    s = sphere(1, seg=22, rings=13)

    def face(p):
        x, y, z = p.x, p.y, p.z
        if y < 0:
            p.x *= 1 - 0.22 * (-y) ** 1.5
        if z > 0:
            ax = abs(x)
            p.z -= 0.22 * math.exp(-((ax - 0.36) / 0.16) ** 2 - ((y - 0.2) / 0.16) ** 2)       # sockets
            p.z -= 0.15 * math.exp(-(x / 0.08) ** 2 - ((y + 0.12) / 0.1) ** 2)                # nasal hole
            p.z += 0.06 * math.exp(-((y - 0.42) / 0.1) ** 2) * math.exp(-(x / 0.5) ** 2)     # brow
            p.x += 0.08 * math.copysign(1, x) * math.exp(-((ax - 0.6) / 0.15) ** 2 - ((y + 0.1) / 0.15) ** 2)  # cheekbones
            p.z += 0.08 * math.exp(-((y + 0.5) / 0.14) ** 2) * math.exp(-(x / 0.35) ** 2)    # teeth row
        else:
            p.y = y if y > -0.4 else -0.4 + (y + 0.4) * 0.5                                    # skull base
        return p
    return F(s.warp(face), f_sphere(0.10))

part('Props__corpse_skull', f_sphere(0.10), skull, smooth=60)


def jaw():
    pts = [(-0.05, 0.004, -0.025), (-0.049, 0.0, 0.0), (-0.035, 0.0, 0.018), (0, 0.0, 0.025), (0.035, 0.0, 0.018),
           (0.049, 0.0, 0.0), (0.05, 0.004, -0.025)]
    s = tube(pts, 0.007, seg=7, smooth=1).scale(1, 1.5, 1)
    for sx in (-1, 1):
        s = s + rbox(0.008, 0.02, 0.012, r=0.003, seg=1).move(x=sx * 0.05, y=0.006, z=-0.022)
    return F(s, f_box(0.11, 0.03, 0.05))

part('Props__corpse_jaw', f_box(0.11, 0.03, 0.05), jaw, smooth=60)
RIB_FRAME = f_bounds((-0.098, 0.0, -0.008), (0.098, 0.098, 0.008))


def rib():
    pts = [(0.09 * math.cos(a), 0.09 * math.sin(a) * (1 + 0.05 * math.sin(a * 3)), 0) for a in (math.pi * i / 12 for i in range(13))]
    s = tube(pts, 0.008, seg=6, r_end=0.005).scale(1, 1, 0.6)
    return F(s, RIB_FRAME)

part('Props__corpse_rib', RIB_FRAME, rib, smooth=60)
part('Props__corpse_vertebra', f_cyl(0.016, 0.014, 0.025),
     lambda: lathe([(0, -0.0125), (0.013, -0.0125), (0.014, -0.011), (0.012, -0.004), (0.0115, 0.0), (0.013, 0.006),
                    (0.016, 0.011), (0.015, 0.0125), (0, 0.0125)], seg=10), smooth=60)


def pelvis():
    # pelvic girdle: a flared bony bowl opening toward the legs (+X), the
    # rim pinched in front and back so the two iliac wings stand out
    s = vessel([(0.018, -0.03), (0.034, -0.012), (0.042, 0.012), (0.046, 0.03)], 0.004, seg=12)
    s = s.warp(lambda p: Vector((p.x * (1 - 0.3 * math.exp(-(p.x / 0.02) ** 2) * max(0.0, p.y + 0.01) / 0.04), p.y, p.z)))
    return F(s.rot(z=-math.pi / 2), f_box(0.06, 0.05, 0.09))

part('Props__corpse_pelvis', f_box(0.06, 0.05, 0.09), pelvis, smooth=60)


def long_bone(R, L):
    # shaft with a ball head at +Y and a broad knuckled end at -Y
    s = lathe([(0, -L / 2), (0.75 * R, -L / 2), (R, -L / 2 + 0.01), (0.85 * R, -L / 2 + 0.024), (0.55 * R, -L / 2 + 0.05),
               (0.5 * R, 0), (0.55 * R, L / 2 - 0.04), (0.8 * R, L / 2 - 0.022), (0.9 * R, L / 2 - 0.01), (0.6 * R, L / 2),
               (0, L / 2)], seg=10)
    s = s.warp(lambda p: Vector((p.x * (1.15 if p.y < -L / 2 + 0.03 else 1.0), p.y, p.z)))
    return F(s, f_cyl(R, R, L))

part('Props__corpse_femurA', f_cyl(0.014, 0.014, 0.24), lambda: long_bone(0.014, 0.24), smooth=60)
part('Props__corpse_femurB', f_cyl(0.014, 0.014, 0.22), lambda: long_bone(0.014, 0.22), smooth=60)
part('Props__corpse_armBone', f_cyl(0.011, 0.011, 0.22), lambda: long_bone(0.011, 0.22), smooth=60)


def rag():
    s = slab(0.2, 0.08, 0.002, 8, 4).rot(x=-math.pi / 2)
    rng = random.Random(3)

    def torn(p):
        if abs(p.x) > 0.099:
            p.x -= math.copysign(rng.uniform(0, 0.012), p.x)
        return p
    return noisy(s.warp(torn), 0.0025, scale=40, seed=6, axis=(0, 1, 0))

part('Props__corpse_rag', f_box(0.20, 0.005, 0.08), rag, smooth=80)
part('Props__corpse_idCard', f_box(0.16, 0.003, 0.10),
     lambda: cut(prism(rounded_rect(0.16, 0.10, 0.008, seg=2), 0.003), rbox(0.014, 0.004, 0.01, r=0.0015, seg=1).move(x=-0.068, y=0.035))
     .rot(x=math.pi / 2))
