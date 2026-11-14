"""Family `wall_and_ceiling_fixtures` - "core" theme: wall fixtures, ceiling fixtures and litter.

place_emergency_strobes      strobe mounting base (the red dome is transparent)
place_emergency_phones       red phone housing, handset grip, ear/mouth cups
place_sprinkler_heads        threaded stub + hex, glass bulb, yoke arms, toothed deflector
place_wall_intercoms         faceplate, speaker grille, grille bars, talk button
place_bottle_caps            crimped crown cap
place_single_socks           flattened sock foot, ribbed cuff, soft bands
place_motion_detector_domes   ceiling mounting ring (the dome is transparent)
place_wall_doors             door slab with moulding grooves, fielded panels, knob
place_wall_outlets           duplex cover plate, moulded plug, cord
place_card_readers           reader housing with slot hood and tap ring
place_extinguisher_cabinets  recessed cabinet tub, extinguisher cylinder, valve head
place_elevator_panels        steel plate with button bezels, push buttons
place_ceiling_fans           downrod, motor housing, blades
place_switch_plates          wall plate, toggle lever, screws
place_smoke_detectors        detector body with side vents, vented centre cap
place_exit_arrow_signs        sign housing, arrow shaft, arrow head
place_frosted_windows        moulded window frame, muntin bars
place_leaning_frames         hardboard backing
place_wall_handprints        crumpled scrap paper
place_pinned_photos          push pin
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
    """A plate built from the back (first) to the front (last):
    profile = [(inset, z), ...]; each ring is the rounded outline inset."""
    return loft([(rr(w - 2 * i, h - 2 * i, r - i, seg), z) for i, z in profile])


def frame_ring(wo, ho, wi, hi, d, z=0.0):
    """A rectangular ring (outer minus inner), d deep, centred at z: a cutter
    for moulding grooves."""
    return cut(box(wo, ho, d), box(wi, hi, d * 2)).move(z=z)


def on_z(s):
    """A lathe built on +Y turned to face +Z."""
    return s.rot(x=HALF)


def dome_head(r, h, seg=8, simple=False):
    """A small button/tamper dome head on +Y, base at y=0."""
    if simple:
        return lathe([(r, 0), (r * 0.72, h * 0.8), (0, h)], seg, cap_bottom=False)
    return lathe([(r, 0), (r, h * 0.4), (r * 0.72, h * 0.9), (0, h)], seg, cap_bottom=False)


def pillow_z(p, zmax, ymid, pw=4.0):
    """Pinch a soft thing's thickness toward its rim across Z (a sock, a band)."""
    t = min(1.0, abs(p.z) / zmax)
    g = (1 - t ** pw) ** (1 / pw)
    p.y = ymid + (p.y - ymid) * max(g, 0.18)
    return p


# ------------------------------------------------------ emergency strobe --
def strobe_base():
    # round cast base: wall flange, recessed band, bezel ring the dome sits in
    s = lathe([(0, -0.005), (0.030, -0.005), (0.030, -0.0036), (0.0284, -0.0029), (0.0284, 0.0021),
               (0.0300, 0.0029), (0.0295, 0.005), (0, 0.005)], seg=18)
    return s

part('EmergencyStrobes__base', f_cyl(0.030, 0.030, 0.010), strobe_base)


# ------------------------------------------------------- emergency phone --
def phone_box():
    # red steel housing: rounded body, a door seam around the front, speaker
    # slots under the label, a trough the handset rests against, a call button
    w, h, d = 0.16, 0.24, 0.09
    body = plate(w, h, [(0, -d / 2), (0, d / 2 - 0.006), (0.002, d / 2 - 0.0015), (0.006, d / 2)], r=0.010, seg=2)
    fz = d / 2
    seam = frame_ring(0.144, 0.224, 0.140, 0.220, 0.006, z=fz)
    slots = sum((rbox(0.07, 0.004, 0.006, r=0.0015, seg=1).move(y=0.072 - i * 0.009, z=fz) for i in range(4)), Shape())
    trough = rbox(0.13, 0.034, 0.012, r=0.006, seg=2).move(y=-0.02, z=fz + 0.001)
    button = cyl(0.011, 0.011, 0.01, seg=16, bevel_r=0).rot(x=HALF).move(y=-0.078, z=fz)
    return cut(body, seam, slots, trough, button)

def phone_handset():
    # a waisted grip that swells into the cups and bows away from the wall
    L = 0.12
    s = lathe([(0.0, -L / 2), (0.0125, -L / 2), (0.0140, -L / 2 + 0.008), (0.0118, -0.03), (0.0110, 0.0),
               (0.0118, 0.03), (0.0140, L / 2 - 0.008), (0.0125, L / 2), (0.0, L / 2)], seg=12)
    s = s.warp(lambda p: Vector((p.x, p.y, p.z + 0.0022 * (1 - (p.y / (L / 2)) ** 2))))
    return s.fit((-0.014, -L / 2, -0.014), (0.014, L / 2, 0.014))

def phone_cup():
    # ear / mouth piece: a rounded pod whose face (toward the wall) is dished
    r = 0.018
    s = lathe([(0.0, -r + 0.0022), (0.007, -r + 0.0014), (0.0115, -r), (0.0158, -r + 0.0016), (0.0180, -r + 0.0065),
               (0.0180, -0.001), (0.0160, 0.008), (0.0115, 0.0145), (0.0, r)], seg=16)
    return on_z(s).fit((-r, -r, -r), (r, r, r))

part('EmergencyPhones__box', f_box(0.16, 0.24, 0.09), phone_box)
part('EmergencyPhones__handset', f_cyl(0.014, 0.014, 0.12), phone_handset)
part('EmergencyPhones__cup', f_sphere(0.018), phone_cup, smooth=50)


# ------------------------------------------------------ sprinkler heads --
def sprinkler_stub():
    # threaded inlet on top, a hex wrench boss below
    hexb = cyl(0.008, 0.008, 0.008, seg=6, bevel_r=0.0008, bevel_seg=1).move(y=-0.006)
    prof = [(0.0, -0.002)]
    y = -0.002
    for i in range(5):
        prof += [(0.0062, y), (0.0070, y + 0.0012)]
        y += 0.0024
    prof += [(0.0062, 0.010), (0.0, 0.010)]
    thread = lathe(prof, seg=10)
    return hexb + thread

def sprinkler_bulb():
    # the glass bulb: a teardrop with a pinched neck at the top
    return lathe([(0, -0.008), (0.0048, -0.0070), (0.0075, -0.0035), (0.0080, 0.0), (0.0068, 0.0040),
                  (0.0036, 0.0066), (0.0030, 0.008), (0, 0.008)], seg=8)

def sprinkler_arm():
    return taper(rbox(0.014, 0.002, 0.004, r=0.0006, seg=1), top=0.8, along='x', axes='z').move(x=0.0).fit(
        (-0.007, -0.001, -0.002), (0.007, 0.001, 0.002))

def sprinkler_deflector():
    # toothed deflector plate, 1 mm thick behind its face
    # twelve square-ended tines around a solid hub; a pressed sheet, so only
    # its two faces are modelled (the 0.4 mm rim is invisible)
    pts = []
    n = 12
    for i in range(n):
        a = TAU * i / n
        da = TAU / n
        pts += [(0.008 * math.cos(a - da * 0.27), 0.008 * math.sin(a - da * 0.27)),
                (0.008 * math.cos(a + da * 0.27), 0.008 * math.sin(a + da * 0.27)),
                (0.0062 * math.cos(a + da * 0.5), 0.0062 * math.sin(a + da * 0.5))]
    k = len(pts)
    v = [(x, y, 0.0) for x, y in pts] + [(x, y, -0.0004) for x, y in pts]
    return Shape(v, [tuple(range(k)), tuple(reversed(range(k, 2 * k)))])

part('SprinklerHeads__stub', f_cyl(0.008, 0.008, 0.020), sprinkler_stub)
part('SprinklerHeads__bulb', f_sphere(0.008), sprinkler_bulb, smooth=60)
part('SprinklerHeads__arm', f_box(0.014, 0.002, 0.004), sprinkler_arm)
part('SprinklerHeads__deflector', f_bounds((-0.008, -0.008, 0), (0.008, 0.008, 0), 'plane'), sprinkler_deflector)


# -------------------------------------------------------- wall intercom --
def intercom_body():
    # brushed faceplate: eased front edge, grooves ringing the speaker and
    # the button, four tamper screws proud of the face
    w, h, d = 0.10, 0.16, 0.020
    fz = d / 2 - 0.001
    body = plate(w, h, [(0, -d / 2), (0, fz - 0.0025), (0.0012, fz - 0.0006), (0.0025, fz)], r=0.008, seg=2)
    g1 = cut(cyl(0.0365, 0.0365, 0.004, seg=20, bevel_r=0), cyl(0.0350, 0.0350, 0.01, seg=20, bevel_r=0))
    g2 = cut(cyl(0.0135, 0.0135, 0.004, seg=16, bevel_r=0), cyl(0.0122, 0.0122, 0.01, seg=16, bevel_r=0))
    body = cut(body, g1.rot(x=HALF).move(y=0.03, z=fz), g2.rot(x=HALF).move(y=-0.05, z=fz))
    screws = sum((on_z(dome_head(0.0032, 0.0014, seg=6, simple=True)).move(x=sx * 0.038, y=sy * 0.068, z=fz - 0.0004)
                  for sx in (-1, 1) for sy in (-1, 1)), Shape())
    return body + screws

def intercom_grille():
    # dark speaker mesh: flat rim, concentric pressed rings, 1.5 mm deep
    prof = [(0.0, -0.0006), (0.0150, -0.0006), (0.0160, -0.0001), (0.0170, -0.0006),
            (0.0285, -0.0006), (0.0295, 0.0), (0.0320, 0.0), (0.0320, -0.0015)]
    # (r, z) from the centre out; the back stays open against the faceplate
    s = lathe([(r, -z) for r, z in reversed(prof)], seg=20, cap_bottom=False, cap_top=False)
    return s.rot(x=-HALF)

def intercom_slot():
    return rbox(0.040, 0.002, 0.001, r=0.0004, seg=1)

def intercom_button():
    # talk button: a chrome-less domed red cap on a short skirt
    return lathe([(0, -0.0025), (0.010, -0.0025), (0.010, -0.0006), (0.0088, 0.0012), (0.0060, 0.0025), (0, 0.0025)], seg=12)

part('WallIntercoms__body', f_box(0.10, 0.16, 0.020), intercom_body)
part('WallIntercoms__grille', f_bounds((-0.032, -0.032, 0), (0.032, 0.032, 0), 'plane'), intercom_grille)
part('WallIntercoms__slot', f_box(0.040, 0.002, 0.001), intercom_slot)
part('WallIntercoms__button', f_cyl(0.010, 0.010, 0.005), intercom_button, smooth=50)


# ---------------------------------------------------------- bottle caps --
def crown_cap():
    # crimped crown cap lying open side down: flat top, rolled shoulder,
    # fluted skirt flaring to the crimp, hollow underside
    seg = 32
    v, f = [], []
    def ring(r_of, y):
        ids = []
        for i in range(seg):
            a = TAU * i / seg
            r = r_of(i)
            v.append((r * math.sin(a), y, r * math.cos(a)))
            ids.append(len(v) - 1)
        return ids
    top_c = len(v); v.append((0, 0.0015, 0))
    r0 = ring(lambda i: 0.0086, 0.0015)
    r1 = ring(lambda i: 0.0100 if i % 2 == 0 else 0.0097, 0.0006)
    r2 = ring(lambda i: 0.0110 if i % 2 == 0 else 0.0099, -0.0015)
    r3 = ring(lambda i: 0.0104 if i % 2 == 0 else 0.0094, -0.0015)
    bot_c = len(v); v.append((0, 0.0009, 0))
    for i in range(seg):
        j = (i + 1) % seg
        f.append((top_c, r0[j], r0[i]))
        for a, b in ((r0, r1), (r1, r2), (r2, r3)):
            f.append((a[i], a[j], b[j], b[i]))
        f.append((bot_c, r3[i], r3[j]))
    s = propkit._fix_winding(Shape(v, f))
    return s

part('BottleCaps__cap', f_cyl(0.010, 0.011, 0.003), crown_cap)


# ----------------------------------------------------------- single sock --
ZS = 0.018          # half the sock's width

def sock_foot():
    # a flattened knit foot: puffy in the middle, pinched at the fold lines,
    # rounded toe at +x, a slight heel bulge on one side
    w, t = 0.10, 0.014
    s = slab(w, 2 * ZS, t, 10, 5).rot(x=-HALF)          # XZ plane, thickness along Y
    def shape(p):
        u = (p.x + w / 2) / w
        if u > 0.72:                                     # toe
            k = min(1.0, (u - 0.72) / 0.28)
            p.z *= (1 - k ** 3.0) ** (1 / 3.0) * 0.85 + 0.15
        if 0.18 < u < 0.42 and p.z > 0:                  # heel bulge
            p.z *= 1 + 0.10 * math.sin((u - 0.18) / 0.24 * math.pi)
        return pillow_z(p, ZS * 1.02, 0.0)
    s = s.warp(shape)
    s = noisy(s, 0.0006, scale=180, seed=3)
    return s.fit((-w / 2, -t / 2, -ZS), (w / 2, t / 2, ZS))

def sock_cuff():
    # ribbed cuff: ribs run along the sock; the mouth at -x stands a little open
    w, t = 0.04, 0.014
    s = slab(w, 2 * ZS, t, 4, 12).rot(x=-HALF)
    def shape(p):
        rib = 0.0007 * math.cos(p.z / (2 * ZS) * 12 * math.pi)
        if p.y > 0:
            p.y += rib
            u = (p.x + w / 2) / w
            p.y += 0.0012 * max(0.0, 1 - u * 3)          # open mouth lifts the top layer
        else:
            p.y -= rib * 0.5
        return pillow_z(p, ZS * 1.02, 0.0)
    s = s.warp(shape)
    return s.fit((-w / 2, -t / 2, -ZS), (w / 2, t / 2, ZS))

def sock_band():
    # a soft band that wraps the sock with the same pinched section
    w, t = 0.010, 0.015
    s = slab(w, 2 * ZS, t, 2, 8).rot(x=-HALF)
    s = s.warp(lambda p: pillow_z(p, ZS * 1.02, 0.0))
    return s.fit((-w / 2, -t / 2, -ZS), (w / 2, t / 2, ZS))

part('SingleSocks__foot', f_box(0.10, 0.014, 0.036), sock_foot, smooth=180)
part('SingleSocks__cuff', f_box(0.04, 0.014, 0.036), sock_cuff, smooth=180)
part('SingleSocks__stripe', f_box(0.010, 0.015, 0.036), sock_band, smooth=180)


# ---------------------------------------------------- motion detector --
def motion_base():
    # ceiling ring: flat top against the ceiling, chamfered skirt, a collar
    # on the underside the dome clips into, three screws
    s = lathe([(0, -0.0025), (0.0355, -0.0025), (0.0365, -0.0016), (0.0380, -0.0012), (0.0430, -0.0010),
               (0.0450, 0.0004), (0.0450, 0.0025), (0, 0.0025)], seg=32)
    screws = Shape()
    for k in range(3):
        a = TAU * k / 3 + 0.5
        screws = screws + dome_head(0.0022, 0.0009, seg=8).rot(x=math.pi).move(
            x=0.0405 * math.cos(a), y=-0.0009, z=0.0405 * math.sin(a))
    return s + screws

part('MotionDetectorDomes__base', f_cyl(0.045, 0.045, 0.005), motion_base)


# ------------------------------------------------------------ wall door --
def door_slab():
    # a painted slab with eased edges and a sunk moulding groove framing
    # each raised panel
    w, h, d = 0.70, 1.90, 0.030
    s = rbox(w, h, d, r=0.004, seg=2)
    for py in (-0.35, 0.35):
        s = cut(s, frame_ring(0.545, 0.645, 0.505, 0.605, 0.008, z=d / 2).move(y=py))
    # a kick-rail line near the floor
    s = cut(s, rbox(0.62, 0.006, 0.006, r=0.002, seg=1).move(y=-0.83, z=d / 2))
    return s

def door_panel():
    # raised and fielded panel: a thin tongue, a sloped fielding, a flat field
    w, h = 0.50, 0.60
    return plate(w, h, [(0.0, -0.005), (0.0, -0.0015), (0.004, -0.0005), (0.036, 0.0042), (0.038, 0.005)],
                 r=0.004, seg=1)

def door_knob():
    # a round brass knob on its rose, pointing out of the door (+Z); the
    # rose sits on the door face 10 mm behind the knob's centre line
    prof = [(0.0, -0.014), (0.0145, -0.014), (0.0145, -0.0095), (0.0110, -0.0078), (0.0050, -0.0072),
            (0.0046, -0.0035), (0.0065, -0.0003), (0.0120, 0.0040), (0.0140, 0.0085), (0.0120, 0.0128),
            (0.0, 0.0140)]
    s = on_z(lathe(prof, seg=14))
    return s.fit((-0.014, -0.015, -0.014), (0.014, 0.015, 0.014))

part('WallDoors__slab', f_box(0.70, 1.90, 0.030), door_slab)
part('WallDoors__panel', f_box(0.50, 0.60, 0.010), door_panel)
part('WallDoors__knob', f_cyl(0.014, 0.014, 0.030), door_knob, smooth=45)


# ---------------------------------------------------------- wall outlet --
def outlet_plate():
    # duplex cover: bevelled plate, two raised receptacle faces, centre screw
    w, h, d = 0.070, 0.11, 0.005
    fz = d / 2 - 0.001
    body = plate(w, h, [(0, -d / 2), (0, -d / 2 + 0.0012), (0.0022, fz - 0.0004), (0.0034, fz)], r=0.004, seg=2)
    faces = Shape()
    for oy in (0.025, -0.025):
        o = round_outline([(-0.0175, -0.018), (0.0175, -0.018), (0.0175, 0.010), (0.0105, 0.0135),
                           (-0.0105, 0.0135), (-0.0175, 0.010)], 0.003, seg=2)
        faces = faces + loft([(o, -0.0005), ([(x * 0.97, y * 0.97 - 0.0001) for x, y in o], d / 2)]).move(y=oy - 0.0012)
    screw = on_z(dome_head(0.0035, 0.0011, seg=10)).move(z=fz - 0.0002)
    screw = cut(screw, box(0.006, 0.0007, 0.004).move(z=fz + 0.001))
    return body + faces + screw

def outlet_plug():
    # moulded plug body: tapered sides, grip grooves, rounded front
    w, h, d = 0.035, 0.036, 0.020
    body = plate(w, h, [(0, -d / 2), (0, -d / 2 + 0.002), (0.0018, d / 2 - 0.004), (0.0034, d / 2 - 0.0008),
                        (0.0048, d / 2)], r=0.005, seg=2)
    # grip grooves follow the tapered sides (inset 1.8 mm over 14 mm of depth)
    slope = math.atan2(0.0018, 0.014)
    face_x = w / 2 - 0.0018 * (0.009 / 0.014)
    grooves = Shape()
    for sx in (-1, 1):
        for k in range(3):
            grooves = grooves + box(0.004, 0.0015, 0.011).rot(y=-sx * slope).move(
                x=sx * (face_x + 0.0012), y=0.006 - k * 0.0048, z=0.001)
    return cut(body, grooves)

def outlet_cord():
    # round cord with a slight lazy curve
    L = 0.20
    pts = [(0.003 * math.sin(math.pi * i / 10) ** 1.5, -L / 2 + L * i / 10, 0.0) for i in range(11)]
    s = tube(pts, 0.002, seg=6)
    return s

part('WallOutlets__plate', f_box(0.070, 0.11, 0.005), outlet_plate)
part('WallOutlets__plug', f_box(0.035, 0.036, 0.020), outlet_plug)
part('WallOutlets__cord', f_cyl(0.002, 0.002, 0.20), outlet_cord, smooth=60)


# ---------------------------------------------------------- card reader --
def card_reader():
    # moulded reader housing: soft front edge, a hood around the card slot,
    # a grooved tap ring on the lower face
    w, h, d = 0.08, 0.12, 0.020
    fz = d / 2 - 0.002
    body = plate(w, h, [(0, -d / 2), (0, fz - 0.005), (0.0025, fz - 0.0012), (0.0045, fz)], r=0.010, seg=2)
    hood = plate(0.062, 0.016, [(0, fz - 0.003), (0, d / 2 - 0.0012), (0.0014, d / 2)], r=0.004, seg=2).move(y=0.02)
    ring = cut(cyl(0.0175, 0.0175, 0.004, seg=24, bevel_r=0), cyl(0.0162, 0.0162, 0.01, seg=24, bevel_r=0))
    body = cut(body, ring.rot(x=HALF).move(x=-0.006, y=-0.030, z=fz))
    return body + hood

part('CardReaders__body', f_box(0.08, 0.12, 0.020), card_reader)


# ------------------------------------------------ extinguisher cabinet --
def ext_cabinet():
    # a recessed steel tub: rolled outer edge, a glass rebate, deep interior
    w, h, d = 0.16, 0.44, 0.08
    shell = plate(w, h, [(0, -d / 2), (0, d / 2 - 0.004), (0.0015, d / 2 - 0.001), (0.004, d / 2)], r=0.008, seg=2)
    rebate = box(0.142, 0.422, 0.006).move(z=d / 2)
    pocket = rbox(0.130, 0.410, d, r=0.006, seg=2).move(z=d / 2 - d / 2 + 0.004 + 0.002)
    return cut(shell, rebate, pocket)

def extinguisher():
    # steel cylinder: rolled foot, straight wall, domed shoulder to the neck
    R, H = 0.032, 0.24
    y0, y1 = -H / 2, H / 2
    prof = [(0.0, y0), (0.0275, y0), (0.0305, y0 + 0.002), (0.0320, y0 + 0.007), (0.0306, y0 + 0.011),
            (0.0320, y0 + 0.015), (0.0320, y1 - 0.024), (0.0306, y1 - 0.013), (0.0270, y1 - 0.006),
            (0.0200, y1 - 0.0015), (0.0130, y1), (0.0, y1)]
    return lathe(prof, seg=24)

def ext_valve():
    # black valve head: knurled collar nut, valve body, lever across the top
    seg = 20
    v, f = [], []
    def ring(r_of, y):
        ids = []
        for i in range(seg):
            a = TAU * i / seg
            r = r_of(i)
            v.append((r * math.sin(a), y, r * math.cos(a)))
            ids.append(len(v) - 1)
        return ids
    knurl = lambda i: 0.0320 if i % 2 == 0 else 0.0303
    rs = [ring(lambda i: 0.0300, -0.010), ring(knurl, -0.0088), ring(knurl, -0.0040),
          ring(lambda i: 0.0200, -0.0026)]
    c1 = len(v); v.append((0, -0.0026, 0))
    for i in range(seg):
        j = (i + 1) % seg
        for a, b in zip(rs, rs[1:]):
            f.append((a[i], a[j], b[j], b[i]))
        f.append((c1, rs[-1][j], rs[-1][i]))
    collar = propkit._fix_winding(Shape(v, f))
    body = lathe([(0.0165, -0.0030), (0.0170, 0.0055), (0.0152, 0.0064), (0.0, 0.0064)], seg=10, cap_bottom=False)
    # carry handle on top, squeeze lever under it, both hinged at -x
    lever = rbox(0.048, 0.0032, 0.011, r=0.0012, seg=1).rot(z=0.05).move(y=0.0081)
    grip = rbox(0.028, 0.0028, 0.009, r=0.0010, seg=1).rot(z=0.10).move(x=0.004, y=0.0050)
    return collar + body + lever + grip

part('ExtinguisherCabinets__cabinet', f_box(0.16, 0.44, 0.08), ext_cabinet)
part('ExtinguisherCabinets__extinguisher', f_cyl(0.032, 0.032, 0.24), extinguisher)
part('ExtinguisherCabinets__valve', f_cyl(0.024, 0.032, 0.020), ext_valve)


# ------------------------------------------------------- elevator panel --
EL_BUTTONS = [(-0.025 + bx * 0.025, 0.08 - by * 0.025, 0.0105) for by in range(4) for bx in range(3)] + [(0.0, -0.09, 0.0125)]

def elevator_plate():
    # brushed steel plate: chamfered edge, raised bezels under every button,
    # an indicator window recess at the top, tamper screws in the corners
    w, h, d = 0.10, 0.24, 0.014
    fz = d / 2 - 0.0014
    body = plate(w, h, [(0, -d / 2), (0, fz - 0.0015), (0.0015, fz)], r=0.006, seg=1)
    body = cut(body, rbox(0.064, 0.018, 0.004, r=0.002, seg=1).move(y=0.104, z=fz))
    bezels = Shape()
    for x, y, r in EL_BUTTONS:
        b = lathe([(r, -0.0006), (r - 0.0007, 0.0014), (0.0, 0.0014)], seg=10, cap_bottom=False)
        bezels = bezels + on_z(b).move(x=x, y=y, z=fz)
    screws = sum((on_z(dome_head(0.0026, 0.0010, seg=6, simple=True)).move(x=sx * 0.040, y=sy * 0.108, z=fz - 0.0002)
                  for sx in (-1, 1) for sy in (-1, 1)), Shape())
    return body + bezels + screws

def elevator_button():
    # push button: straight skirt, rounded rim, slightly dished face
    return lathe([(0, -0.002), (0.008, -0.002), (0.008, 0.0008), (0.0075, 0.0016), (0.0066, 0.0020), (0, 0.0016)], seg=12)

part('ElevatorPanels__plate', f_box(0.10, 0.24, 0.014), elevator_plate)
part('ElevatorPanels__button', f_cyl(0.008, 0.008, 0.004), elevator_button, smooth=50)


# ---------------------------------------------------------- ceiling fan --
def fan_downrod():
    # steel downrod with a coupling collar and cross-pin at the motor end
    y0, y1 = -0.04, 0.04
    rod = lathe([(0.0, y0), (0.0100, y0), (0.0100, y0 + 0.014), (0.0088, y0 + 0.016), (0.0076, y0 + 0.017),
                 (0.0076, y1 - 0.004), (0.0090, y1 - 0.002), (0.0090, y1), (0.0, y1)], seg=12)
    pin = cyl(0.0022, 0.0022, 0.0205, seg=6, bevel_r=0.0005).rot(z=HALF).move(y=y0 + 0.007)
    return rod + pin

def fan_motor():
    # motor housing: domed bottom bowl, a seam band, shoulder up to the rod
    y0, y1 = -0.015, 0.015
    prof = [(0.0, y0), (0.018, y0), (0.032, y0 + 0.0012), (0.041, y0 + 0.0040), (0.0450, y0 + 0.0085),
            (0.0450, 0.0010), (0.0438, 0.0018), (0.0438, 0.0034), (0.0450, 0.0042), (0.0450, y1 - 0.0055),
            (0.0405, y1 - 0.0022), (0.0200, y1 - 0.0004), (0.0110, y1), (0.0, y1)]
    return lathe(prof, seg=22)

def fan_blade():
    # a flat blade, narrow at the root and full width with rounded tip
    L, t, W = 0.30, 0.008, 0.06
    x0, x1 = -L / 2, L / 2
    top = [(x0, 0.020), (x0 + 0.06, 0.026), (x1 - 0.022, W / 2), (x1 - 0.006, W / 2 - 0.005), (x1, W / 2 - 0.02)]
    pts = top + [(x, -y) for x, y in reversed(top)]
    pts = round_outline(pts, 0.006, seg=2)
    s = prism(pts, t).rot(x=HALF)
    s = bevel(s, 0.0022, seg=2, angle=40)
    return s.fit((-L / 2, -t / 2, -W / 2), (L / 2, t / 2, W / 2))

part('CeilingFans__downrod', f_cyl(0.010, 0.010, 0.08), fan_downrod)
part('CeilingFans__motor', f_cyl(0.045, 0.045, 0.030), fan_motor)
part('CeilingFans__blade', f_box(0.30, 0.008, 0.06), fan_blade)


# --------------------------------------------------------- switch plate --
def switch_plate():
    # bevelled wall plate with the toggle opening and two countersunk holes
    w, h, d = 0.075, 0.115, 0.008
    body = plate(w, h, [(0, -d / 2), (0, -d / 2 + 0.0015), (0.0028, d / 2 - 0.0008), (0.0040, d / 2)], r=0.004, seg=2)
    opening = rbox(0.0115, 0.0255, 0.006, r=0.0012, seg=1).move(z=d / 2)
    holes = [cyl(0.0034, 0.0016, 0.0024, seg=12, bevel_r=0).rot(x=HALF).move(y=sy * 0.048, z=d / 2 - 0.0008)
             for sy in (-1, 1)]
    return cut(body, opening, *holes)

def switch_toggle():
    # a toggle lever: square boss at the plate, tapering bat with a round tip
    w, h, d = 0.010, 0.024, 0.014
    z0 = -d / 2
    rings = [(rr(w, h, 0.0015, seg=1), z0), (rr(w, h, 0.0015, seg=1), z0 + 0.003),
             (rr(0.0072, 0.0105, 0.0015, seg=1), z0 + 0.0036), (rr(0.0078, 0.0088, 0.0020, seg=1), d / 2 - 0.0022),
             (rr(0.0060, 0.0068, 0.0020, seg=1), d / 2)]
    return loft(rings)

def switch_screw():
    # slotted dome head on +Y with a short shank behind it
    head = lathe([(0.003, 0.0), (0.003, 0.00056), (0.0021, 0.00136), (0.0, 0.0016)], seg=8)
    head = cut(head, box(0.0072, 0.0016, 0.0006).move(y=0.0016)).move(y=0.0014)
    shank = lathe([(0.0012, -0.003), (0.0012, 0.0016)], seg=6, cap_bottom=False, cap_top=False)
    return head + shank

part('SwitchPlates__plate', f_box(0.075, 0.115, 0.008), switch_plate)
part('SwitchPlates__toggle', f_box(0.010, 0.024, 0.014), switch_toggle)
part('SwitchPlates__screw', f_cyl(0.003, 0.003, 0.006), switch_screw)


# -------------------------------------------------------- smoke detector --
def smoke_body():
    # ceiling puck: flat back, vented side band, soft rounded underside
    seg = 40
    v, f = [], []
    def ring(r_of, y):
        ids = []
        for i in range(seg):
            a = TAU * i / seg
            v.append((r_of(i) * math.sin(a), y, r_of(i) * math.cos(a)))
            ids.append(len(v) - 1)
        return ids
    const = lambda r: (lambda i: r)
    vent = lambda i: 0.0715 if (i % 4) in (1, 2) else 0.0750
    c0 = len(v); v.append((0, -0.014, 0))
    rs = [ring(const(0.050), -0.014), ring(const(0.066), -0.0115), ring(const(0.0735), -0.0065),
          ring(vent, -0.0035), ring(vent, 0.0080), ring(const(0.0750), 0.0120), ring(const(0.0735), 0.014)]
    c1 = len(v); v.append((0, 0.014, 0))
    for i in range(seg):
        j = (i + 1) % seg
        f.append((c0, rs[0][i], rs[0][j]))
        for a, b in zip(rs, rs[1:]):
            f.append((a[i], a[j], b[j], b[i]))
        f.append((c1, rs[-1][j], rs[-1][i]))
    return propkit._fix_winding(Shape(v, f))

def smoke_grill():
    # vented centre cap: stepped rings of vent grooves around a closed boss
    y0, y1 = -0.005, 0.005
    prof = [(0.0, y0), (0.012, y0), (0.0128, y0 + 0.0024), (0.0165, y0 + 0.0024), (0.0173, y0 + 0.0008),
            (0.0240, y0 + 0.0008), (0.0248, y0 + 0.0030), (0.0280, y0 + 0.0030), (0.0288, y0 + 0.0016),
            (0.0370, y0 + 0.0016), (0.0450, y0 + 0.0045), (0.0450, y1), (0.0, y1)]
    return lathe(prof, seg=22)

part('SmokeDetectors__body', f_cyl(0.075, 0.075, 0.028), smoke_body)
part('SmokeDetectors__grill', f_cyl(0.045, 0.045, 0.010), smoke_grill)


# ------------------------------------------------------- exit arrow sign --
def exit_panel():
    # sign housing: rounded case with a raised rim around a recessed face
    w, h, d = 0.30, 0.14, 0.010
    body = plate(w, h, [(0, -d / 2), (0, d / 2 - 0.0016), (0.0016, d / 2)], r=0.010, seg=2)
    face = plate(0.276, 0.116, [(0, 0), (0, 0.01)], r=0.006, seg=2).move(z=d / 2 - 0.0018)
    return cut(body, face)

def exit_shaft():
    return plate(0.16, 0.028, [(0, -0.003), (0, 0.0015), (0.0014, 0.003)], r=0.0015, seg=1)

def exit_head():
    # the 3-sided cone the sign uses, kept as the same wedge (its bounds are
    # the primitive's own) with softened edges
    r, h = 0.036, 0.060
    apex = Vector((0, h / 2, 0))
    base = [Vector((r * math.sin(TAU * i / 3), -h / 2, r * math.cos(TAU * i / 3))) for i in range(3)]
    s = Shape([apex] + base, [(0, 1, 2), (0, 2, 3), (0, 3, 1), (1, 3, 2)])
    s = propkit._fix_winding(s)
    return bevel(s, 0.0025, seg=2, angle=20)

part('ExitArrowSigns__panel', f_box(0.30, 0.14, 0.010), exit_panel)
part('ExitArrowSigns__shaft', f_box(0.16, 0.028, 0.006), exit_shaft)
part('ExitArrowSigns__head', f_cone(0.036, 0.060), exit_head)


# ------------------------------------------------------ frosted window --
def window_frame():
    # moulded frame: rounded outer edge, a bead line, a sunk glazing pocket
    w, h, d = 0.65, 0.55, 0.012
    body = plate(w, h, [(0, -d / 2), (0, d / 2 - 0.004), (0.003, d / 2 - 0.0006), (0.0055, d / 2)], r=0.008, seg=2)
    bead = frame_ring(0.632, 0.532, 0.626, 0.526, 0.003, z=d / 2)
    pocket = box(0.600, 0.500, d).move(z=d / 2 - 0.0045 + d / 2)
    return cut(body, bead, pocket)

def muntin(length, along):
    # glazing bar: chamfered top, flat back
    prof = [(-0.0075, -0.003), (0.0075, -0.003), (0.0075, 0.0), (0.0035, 0.003), (-0.0035, 0.003), (-0.0075, 0.0)]
    s = prism(prof, length)                   # section in XY, length along Z
    if along == 'y':
        s = s.rot(x=HALF)                     # length -> Y, section Y -> Z
    else:
        s = s.rot(x=HALF).rot(z=HALF)         # length -> X, section X -> Y
    return bevel(s, 0.0006, seg=1, angle=40)

part('FrostedWindows__frame', f_box(0.65, 0.55, 0.012), window_frame)
part('FrostedWindows__vbar', f_box(0.015, 0.50, 0.006), lambda: muntin(0.50, 'y'))
part('FrostedWindows__hbar', f_box(0.60, 0.015, 0.006), lambda: muntin(0.60, 'x'))


# -------------------------------------------------------- leaning frame --
def frame_backing():
    # hardboard back: 3 mm thick behind its face, bowed and a little wavy
    w, h, t = 0.24, 0.36, 0.003
    s = slab(w, h, t, 6, 8).move(z=-t / 2)
    s = s.warp(lambda p: Vector((p.x, p.y, p.z - 0.0025 * (1 - (2 * p.x / w) ** 2) * (1 - (2 * p.y / h) ** 2))))
    s = noisy(s, 0.0006, scale=18, seed=5, axis=(0, 0, 1))
    return bevel(s, 0.0008, seg=1, angle=50)

part('LeaningFrames__backing', f_bounds((-0.12, -0.18, 0), (0.12, 0.18, 0), 'plane'), frame_backing)


# ------------------------------------------------------ scrap of paper --
def paper_wad():
    # a crumpled sheet: a lumpy ball, faceted creases
    s = sphere(0.04, seg=14, rings=8).scale(1.0, 0.55, 1.0)
    s = noisy(s, 0.009, scale=40, seed=11)
    s = noisy(s, 0.003, scale=120, seed=4)
    return s.fit((-0.04, -0.02, -0.04), (0.04, 0.02, 0.04))

part('WallHandprints__scrap', f_box(0.08, 0.04, 0.08), paper_wad, flat=True)


# ------------------------------------------------------------- push pin --
def push_pin():
    # thumb tack seen from the front: flange on the wall, waisted grip, domed cap
    prof = [(0.0, -0.003), (0.0040, -0.003), (0.0021, -0.0017), (0.0021, 0.0009), (0.0033, 0.0014),
            (0.0029, 0.0028), (0.0, 0.0030)]
    return on_z(lathe(prof, seg=8)).fit((-0.004, -0.004, -0.003), (0.004, 0.004, 0.003))

part('PinnedPhotos__pin', f_box(0.008, 0.008, 0.006), push_pin, smooth=50)
