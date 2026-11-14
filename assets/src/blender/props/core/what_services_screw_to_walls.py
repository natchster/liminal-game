"""what_services_screw_to_walls (was r12): the things screwed to a wall that a building's services leave there
(docs/design/PROPS.md, P2), remodelled at their real sizes: the office wall phone,
the payphone, the emergency phone, the lift's button panel, the fire alarm
pull station's lamp, the fire extinguisher cabinet and its extinguisher, and
the breaker panel. Sizes and sources: docs/research/props-remodelling/what_services_screw_to_walls.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`; the placers in
`crates/placers/placers_core`, `placers_office` and `placers_industrial`; what
they share in `placers_kit/src/wall_plates.rs`). Each prop's parts share one
frame: x across its front, y up, z out of its front, the wall it hangs on at
z = 0. Where a part moves on its own (a handset, a breaker's handle, a door)
its own frame is given with it.

The wall phone (the 2554 class: a moulded housing on its wall plate, the
handset hung down its front), its frame's origin at the housing's middle:
wallPhoneHousing   the housing: the keypad's well, the earpiece's seat, the
                   mouthpiece's rest, the number card's window, the jack
wallPhoneKeys      the twelve keys in the well
wallPhoneHook      the chrome hanger that holds the earpiece
wallPhoneHandset   the handset, in its own frame: its middle at the origin,
                   the cups' faces at z = 0 facing -z, the earpiece up (+y),
                   the cord's strain relief at the mouthpiece end's foot
wallPhoneCordOn    the coiled cord hanging in a loop from the hung handset
                   to the jack under the housing
wallPhoneCordOff   the cord drawn out straight down, the handset hanging
                   from its end (off the hook)
wallPhoneCordCut   what is left of a cord cut off under the jack: a few
                   turns, the cut end frayed
emergencyPhonePlate  the blank plate a ring-down phone has where the keys
                   would be (the same housing, in red)
emergencyPhoneSign   the engraved sign screwed to the wall above it

The payphone (a single-slot one of the Western kind), its frame's origin on
the wall under the middle of its foot (y = 0 the vault's foot):
wallPayPhoneUpper    the stainless upper housing: the card's window, the
                     keypad's recess, the coin return's chute, the lip
wallPayPhoneVault    the black vault and its door
wallPayPhoneKeys     the twelve chrome keys
wallPayPhoneTrim     the chrome: the coin entry and its slot, the coin
                     return's flap, the switchhook, the cord's grommet, the
                     vault's lock, the security screws
wallPayPhoneHandset  the handset, in its own frame (as the wall phone's;
                     hung on the left side, turned -90 degrees about y)
wallPayPhoneCord     the armoured cord from the handset's foot into the side

The lift's button panel (a car operating panel's faceplate), its origin at
the plate's middle:
elevatorPanelPlate   the 3 mm brushed stainless plate
elevatorPanelBezels  the buttons' bezels, the alarm's, the key switch's
elevatorPanelRings   the dark diffuser ring round each button
elevatorPanelHalo    one lit ring (a lamp), at the origin, for a lit button
elevatorPanelButtons the floor buttons and the door buttons
elevatorPanelAlarm   the alarm button
elevatorPanelKey     the fire service key switch's plug
elevatorPanelScrews  the four tamper-proof screws

The extinguisher cabinet (surface mounted, for a 5 lb extinguisher), its
origin at the cabinet's middle; the extinguisher in it in its own frame
(its axis +y, its foot at y = 0, its label and gauge facing -z):
extCabinetTub        the steel tub, open at the front
extCabinetDoor       the door's frame round the pane
extCabinetGlass      the pane
extCabinetHardware   the piano hinge and the pull
extCabinetExBody     the cylinder's painted wall ('cyl': the label wraps it)
extCabinetExEnds     its foot and its shoulder
extCabinetExValve    the collar, the valve, the handle and lever, the gauge's
                     case, the pin and its ring
extCabinetExSeal     the tamper seal
extCabinetExHose     the hose, its nozzle, the band that holds it

The breaker panel (a 6-space load centre), its origin at the enclosure's
middle:
fuseBoxTub           the enclosure, its knockouts
fuseBoxCover         the front cover, its door's opening, the dead front
fuseBoxBreakers      the six breakers' faces in the dead front's slots
fuseBoxHandle        one breaker's toggle, its pivot at the origin
fuseBoxDoor          the door, in its own frame: its hinge along y at the origin
fuseBoxScrews        the cover's four screws

fireAlarmPullLed     an addressable pull station's LED in its bezel, its back
                     at z = 0 (the station's parts are family `wall_hole_pipe_run_posters_and_hooks`'s)
"""
from propkit import *
import propkit

# ================================================================= util ===


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def chaikin(pts, k=2):
    pts = [Vector(p) for p in pts]
    for _ in range(k):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    return pts


def resample(pts, step):
    """The polyline `pts` at equal steps of arc length."""
    pts = [Vector(p) for p in pts]
    seg = [(a - b).length for a, b in zip(pts[1:], pts)]
    total = sum(seg)
    n = max(2, int(round(total / step)) + 1)
    out, i, acc = [], 0, 0.0
    for k in range(n):
        s = total * k / (n - 1)
        while i < len(seg) - 1 and acc + seg[i] < s:
            acc += seg[i]
            i += 1
        t = 0.0 if seg[i] < 1e-12 else (s - acc) / seg[i]
        out.append(pts[i].lerp(pts[i + 1], min(1.0, max(0.0, t))))
    return out, total


def transport_frames(pts, up=(0, 0, 1)):
    """Tangent, normal and binormal along a polyline (parallel transport)."""
    n = len(pts)
    tans = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tans.append(t.normalized() if t.length > 1e-12 else Vector((0, 1, 0)))
    u = Vector(up)
    if abs(tans[0].dot(u)) > 0.9:
        u = Vector((1, 0, 0))
    nrm = tans[0].cross(u).normalized()
    out = []
    for i in range(n):
        if i:
            q = tans[i - 1].rotation_difference(tans[i])
            nrm = (q @ nrm).normalized()
        out.append((tans[i], nrm, tans[i].cross(nrm).normalized()))
    return out


def loft(rings, cap0=True, cap1=True):
    """Quads between successive closed rings of equal length, capped."""
    n = len(rings[0])
    v, f = [], []
    for r in rings:
        v += [Vector(p) for p in r]
    for j in range(len(rings) - 1):
        for i in range(n):
            i2 = (i + 1) % n
            f.append((j * n + i, j * n + i2, (j + 1) * n + i2, (j + 1) * n + i))
    if cap0:
        f.append(tuple(reversed(range(n))))
    if cap1:
        m = (len(rings) - 1) * n
        f.append(tuple(m + i for i in range(n)))
    return propkit._fix_winding(Shape(v, f))


def coil_cord(path, coil_r=0.0042, wire=0.0016, pitch=0.0045, per_turn=6, tail=0.012, sides=4, smooth=2):
    """A coiled telephone cord wound round the centre line `path`: `pitch`
    metres a turn (4.5 mm close wound, more where it is drawn out), straight
    tails `tail` long at each end."""
    c = chaikin(path, smooth)
    c, total = resample(c, pitch / per_turn)
    fr = transport_frames(c)
    pts = []
    for k, (p, (t, nn, b)) in enumerate(zip(c, fr)):
        s = total * k / (len(c) - 1)
        a = coil_r * smoothstep(0.0, tail, s) * smoothstep(0.0, tail, total - s)
        th = TAU * k / per_turn
        pts.append(p + (nn * math.cos(th) + b * math.sin(th)) * a)
    return tube(pts, wire, seg=sides, caps=True)


def handset_shape(cup, r_ear, r_mouth, width, depth, bow, relief):
    """A handset: the cups (their faces at z = 0, facing -z) `cup` above and
    below its middle, the handle bowed `bow` out between them, and `relief`
    (built along y, centred) at the foot of the mouthpiece end."""
    def cupf(r, d):
        prof = [(0.0, 0.0), (r - 0.004, 0.0), (r - 0.0012, 0.0006), (r, 0.003), (r, 0.010), (r - 0.003, 0.017),
                (r * 0.62, d - 0.003), (r * 0.40, d), (0.0, d)]
        return lathe(prof, seg=28).rot(x=math.pi / 2)
    ear = cupf(r_ear, 0.031).move(0, cup, 0)
    mouth = cupf(r_mouth, 0.030).move(0, -cup, 0)
    rings = []
    n = 17
    for i in range(n):
        u = -1.0 + 2.0 * i / (n - 1)
        y = u * (cup + 0.004)
        zc = 0.025 + bow * math.cos(u * math.pi / 2) ** 1.2
        w = width + 0.012 * u ** 4
        d = depth + 0.004 * u ** 4
        sec = rounded_rect(w, d, min(w, d) * 0.45, seg=3)
        rings.append([Vector((x, y, zc + z)) for x, z in sec])
    return ear + mouth + loft(rings) + relief.move(0, -cup - r_mouth + 0.001, 0.017)


# ============================================================ the wall phone ===
# The 2554 class (sources: docs/research/props-remodelling/what_services_screw_to_walls.md), measured off a photograph against
# its 8.5 in housing: 123 x 232 mm, 58 mm deep; the handset 212 mm.
WP_W, WP_H, WP_D = 0.123, 0.232, 0.058
WP_WELL = (0.058, 0.090, -0.012, 0.010)        # w, h, centre x, y
WP_KEY_X = (-0.0300, -0.0125, 0.0050)
WP_KEY_Y = (0.0400, 0.0200, 0.0000, -0.0200)
WP_EAR = (-0.026, 0.078)                       # the earpiece's seat
WP_MOUTH = (-0.026, -0.076)                    # the mouthpiece's rest
HS_CUP = 0.077                                 # the handset's cups from its middle
HS_AT = (-0.026, 0.001, WP_D - 0.004)          # the hung handset's origin in the phone's frame
JACK = (0.030, -WP_H / 2, 0.026)               # the cord's jack, under the housing


def wall_phone_housing():
    out = prism(rounded_rect(WP_W, WP_H, 0.013, seg=3), WP_D, bevel_r=0.0045, bevel_seg=2)
    s = out.move(0, 0, WP_D / 2)
    # the face falls back a little to its edges (moulded draft): squeeze the
    # front of the housing by 3 per cent
    s.warp(lambda p: Vector((p.x * (1 - 0.03 * p.z / WP_D), p.y * (1 - 0.012 * p.z / WP_D), p.z)))
    ww, wh, wx, wy = WP_WELL
    well = prism(rounded_rect(ww, wh, 0.004, seg=2), 0.012).move(wx, wy, WP_D)
    ear = cyl(0.034, h=0.010, seg=28, bevel_r=0).rot(x=math.pi / 2).move(WP_EAR[0], WP_EAR[1], WP_D + 0.001)
    mouth = cyl(0.031, h=0.010, seg=28, bevel_r=0).rot(x=math.pi / 2).move(WP_MOUTH[0], WP_MOUTH[1], WP_D + 0.001)
    card = box(0.042, 0.012, 0.004).move(-0.006, -0.047, WP_D)
    jack = box(0.012, 0.012, 0.010).move(JACK[0], JACK[1], JACK[2])
    s = cut(s, well, ear, mouth, card, jack)
    # the lip the mouthpiece rests on, moulded under its rest
    lip = rbox(0.040, 0.006, 0.010, r=0.002, seg=1).move(WP_MOUTH[0], WP_MOUTH[1] - 0.034, WP_D - 0.002)
    # the two plate screws' covers at the top and the foot of the face
    return s + lip


def wall_phone_keys():
    out = Shape()
    z0 = WP_D - 0.005
    for y in WP_KEY_Y:
        for x in WP_KEY_X:
            k = rbox(0.0128, 0.0098, 0.0062, r=0.0014, seg=1)
            out = out + k.move(x, y, z0 + 0.0031)
    return out


def wall_phone_hook():
    """The chrome hanger at the top of the earpiece's seat: a bent tab coming
    out of its slot, its lip over the cup's rim."""
    t = 0.0016
    pts = [Vector((0, 0.000, 0.000)), Vector((0, -0.010, 0.010)), Vector((0, -0.024, 0.019)), Vector((0, -0.029, 0.020))]
    rings = []
    for p in chaikin(pts, 2):
        rings.append([p + Vector((x, 0, z)) for x, z in ((-0.016, -t / 2), (0.016, -t / 2), (0.016, t / 2), (-0.016, t / 2))])
    s = loft(rings)
    s = bevel(s, 0.0005, seg=1)
    return s.move(0.004, 0.103, WP_D - 0.006)


def wall_phone_handset():
    """A K-style handset: two cups and the handle bowed out between them; the
    cups' faces at z = 0 (on the housing when hung), the earpiece up; the
    cord's strain relief at the mouthpiece end's foot."""
    relief = cyl(0.0042, 0.0050, 0.012, seg=10, bevel_r=0.0008, bevel_seg=1)
    return handset_shape(HS_CUP, 0.0300, 0.0285, 0.0255, 0.0185, 0.011, relief)


def hs_point(local, at, turn_z=0.0, turn_y=0.0):
    """Where a point of the handset's frame lands, the handset turned about z
    then y and moved to `at`."""
    p = Vector(local)
    p.rotate(Euler((0, 0, turn_z)))
    p.rotate(Euler((0, turn_y, 0)))
    return p + Vector(at)


HS_CORD = (0.0, -HS_CUP - 0.034, 0.017)        # the cord's end in the handset's frame
OFF_AT = (0.030, -0.565, 0.012)                # the hanging handset (turned upside down)
OFF_TURN_Y = 0.30


def wall_phone_cord_on():
    a = hs_point(HS_CORD, HS_AT)
    j = Vector(JACK) + Vector((0, -0.002, 0))
    path = [a, a + Vector((-0.002, -0.020, -0.004)), Vector((-0.030, -0.190, 0.050)), Vector((-0.020, -0.290, 0.040)),
            Vector((0.004, -0.318, 0.034)), Vector((0.026, -0.282, 0.031)), Vector((0.034, -0.190, 0.028)),
            j + Vector((0, -0.020, 0)), j]
    return coil_cord(path, pitch=0.0058)


def wall_phone_cord_off():
    a = hs_point(HS_CORD, OFF_AT, turn_z=math.pi, turn_y=OFF_TURN_Y)
    j = Vector(JACK) + Vector((0, -0.002, 0))
    path = [j, j + Vector((0, -0.020, 0)), Vector((0.031, -0.200, 0.024)), Vector((0.030, -0.330, 0.022)),
            a + Vector((0, 0.030, 0.002)), a]
    return coil_cord(path, pitch=0.0105, smooth=2)


def wall_phone_cord_cut():
    j = Vector(JACK) + Vector((0, -0.002, 0))
    end = j + Vector((0.003, -0.118, 0.003))
    path = [j, j + Vector((0, -0.020, 0)), j + Vector((0.001, -0.085, -0.001)), end]
    s = coil_cord(path, pitch=0.0050, tail=0.008)
    # the cut end: the jacket split and four conductors fanned out
    for k, (dx, dz, l) in enumerate(((0.002, 0.001, 0.010), (-0.002, 0.002, 0.008), (0.001, -0.002, 0.012), (-0.001, -0.001, 0.007))):
        s = s + tube([end, end + Vector((dx, -l * 0.6, dz)), end + Vector((dx * 2.2, -l, dz * 2))], 0.00045, seg=4)
    return s


def emergency_phone_plate():
    ww, wh, wx, wy = WP_WELL
    s = prism(rounded_rect(ww - 0.002, wh - 0.002, 0.0035, seg=2), 0.0040, bevel_r=0.0006, bevel_seg=1)
    return s.move(wx, wy, WP_D - 0.005 + 0.0020)


ES_W, ES_H = 0.200, 0.060


def emergency_phone_sign():
    s = prism(rounded_rect(ES_W, ES_H, 0.004, seg=2), 0.0025, bevel_r=0.0005, bevel_seg=1).move(0, 0, 0.00125)
    for x in (-ES_W / 2 + 0.010, ES_W / 2 - 0.010):
        s = s + screw_head(0.0028, 0.0013).rot(x=math.pi / 2).move(x, 0, 0.0025)
    return s


# ============================================================== the payphone ===
# A single-slot payphone of the Western kind (docs/research/props-remodelling/what_services_screw_to_walls.md): a stainless upper
# housing (the instruction card, the coin entry, the keypad, the coin return)
# over the black vault, the switchhook on its left side and the handset hung
# there on its armoured cord. Its frame: the wall at z = 0, x across the
# housing's middle, y = 0 the foot of the vault.
PP_W, PP_D = 0.200, 0.135
PP_UP = (0.236, 0.530)                         # the upper housing, foot to top
PP_VAULT = (0.0, 0.230, 0.196, 0.124)          # foot, top, width, depth
PP_KEY_X = (-0.026, 0.0, 0.026)
PP_KEY_Y = (0.396, 0.370, 0.344, 0.318)
PP_CARD = (-0.026, 0.474, 0.112, 0.074)        # x, y, w, h of the card's window
PP_COIN = (0.064, 0.480)
PP_RETURN = (0.060, 0.266)
PP_HS_AT = (-0.1025, 0.358, 0.070)             # the handset, turned -90 deg about y
PP_HS_CUP = 0.082


def pay_phone_upper():
    y0, y1 = PP_UP
    s = rbox(PP_W, y1 - y0, PP_D, r=0.007, seg=2).move(0, (y0 + y1) / 2, PP_D / 2)
    cx, cy, cw, ch = PP_CARD
    card = rbox(cw, ch, 0.010, r=0.003, seg=1).move(cx, cy, PP_D + 0.002)
    plate = rbox(0.098, 0.122, 0.010, r=0.004, seg=1).move(0, 0.357, PP_D + 0.0015)
    ret = rbox(0.046, 0.032, 0.060, r=0.004, seg=1).move(PP_RETURN[0], PP_RETURN[1], PP_D)
    grom = cyl(0.0075, h=0.02, seg=12, bevel_r=0).rot(z=math.pi / 2).move(-PP_W / 2, 0.262, 0.058)
    s = cut(s, card, plate, ret, grom)
    # the lip of the upper housing over the vault, and a security screw each side
    lip = rbox(PP_W + 0.004, 0.010, PP_D + 0.003, r=0.003, seg=1).move(0, y0 + 0.005, (PP_D + 0.003) / 2)
    return s + lip


def pay_phone_vault():
    y0, y1, w, d = PP_VAULT
    s = rbox(w, y1 - y0, d, r=0.006, seg=2).move(0, (y0 + y1) / 2, d / 2)
    door = rbox(0.150, 0.168, 0.004, r=0.003, seg=1).move(0, 0.112, d + 0.0015)
    seam = box(0.156, 0.174, 0.0016).move(0, 0.112, d - 0.0004)
    s = cut(s, seam) + door
    return s


def pay_phone_keys():
    out = Shape()
    for y in PP_KEY_Y:
        for x in PP_KEY_X:
            k = rbox(0.0175, 0.0150, 0.0090, r=0.0018, seg=2)
            out = out + k.move(x, y, PP_D - 0.0035 + 0.0045)
    return out


def pay_phone_trim():
    """The chrome: the coin entry's bezel and its slot, the coin return's
    flap, the switchhook on the left side, the cord's grommet, the vault's
    lock and the security screws."""
    cx, cy = PP_COIN
    bez = rbox(0.040, 0.052, 0.012, r=0.004, seg=2).move(cx, cy, PP_D + 0.004)
    slot = box(0.0285, 0.0034, 0.04).rot(x=-0.35).move(cx, cy + 0.004, PP_D + 0.010)
    bez = cut(bez, slot)
    rx, ry = PP_RETURN
    flap = rbox(0.044, 0.030, 0.0025, r=0.001, seg=1).rot(x=0.18).move(rx, ry + 0.001, PP_D - 0.004)
    # the switchhook: an escutcheon on the side, the lever out and up at its tip
    esc = rbox(0.0035, 0.036, 0.046, r=0.0012, seg=1).move(-PP_W / 2 - 0.00175, 0.423, 0.070)
    pts = [Vector((-PP_W / 2, 0.418, 0.070)), Vector((-PP_W / 2 - 0.030, 0.418, 0.070)), Vector((-PP_W / 2 - 0.046, 0.424, 0.070)),
           Vector((-PP_W / 2 - 0.052, 0.434, 0.070))]
    lever = tube(chaikin(pts, 2), 0.0052, seg=10)
    grom = cyl(0.0095, h=0.008, seg=14, bevel_r=0.0012, bevel_seg=1).rot(z=math.pi / 2).move(-PP_W / 2 - 0.004, 0.262, 0.058)
    lock = cyl(0.0125, h=0.006, seg=20, bevel_r=0.0015, bevel_seg=1).rot(x=math.pi / 2).move(0, 0.168, PP_VAULT[3] + 0.0045)
    lock = cut(lock, box(0.0022, 0.012, 0.01).move(0, 0.168, PP_VAULT[3] + 0.008))
    screws = Shape()
    for (x, y) in ((-0.088, 0.520), (0.088, 0.520), (-0.088, 0.246), (0.088, 0.246)):
        screws = screws + screw_head(0.0032, 0.0014, slot=False).rot(x=math.pi / 2).move(x, y, PP_D)
    return bez + flap + esc + lever + grom + lock + screws


def pay_phone_handset():
    # the armoured cord's ferrule at its foot
    ferrule = cyl(0.0068, 0.0078, 0.020, seg=12, bevel_r=0.001, bevel_seg=1).move(0, -0.006, 0)
    return handset_shape(PP_HS_CUP, 0.0325, 0.0310, 0.029, 0.021, 0.012, ferrule)


def ridged_tube(path, r0, r1, step, sides=8, smooth=2):
    """A tube whose radius steps between r0 and r1 every `step` metres: the
    spiral-wound steel of an armoured cord, seen as its ridges."""
    c = chaikin(path, smooth)
    c, total = resample(c, step / 2)
    fr = transport_frames(c)
    rings = []
    for k, (p, (t, nn, b)) in enumerate(zip(c, fr)):
        r = r1 if k % 2 else r0
        rings.append([p + (nn * math.cos(TAU * j / sides) + b * math.sin(TAU * j / sides)) * r for j in range(sides)])
    return loft(rings)


def pay_phone_cord():
    hx, hy, hz = PP_HS_AT
    foot = Vector((hx - 0.017, hy - PP_HS_CUP - 0.042, hz))
    path = [foot, foot + Vector((-0.001, -0.025, 0)), Vector((-0.126, 0.190, 0.072)), Vector((-0.124, 0.122, 0.072)),
            Vector((-0.114, 0.104, 0.068)), Vector((-0.105, 0.130, 0.064)), Vector((-0.106, 0.215, 0.060)),
            Vector((-0.108, 0.250, 0.058)), Vector((-0.102, 0.262, 0.058))]
    return ridged_tube(path, 0.0056, 0.0050, 0.0035, sides=8)


# ======================================================== the lift's panel ===
# A car operating panel's faceplate (docs/research/props-remodelling/what_services_screw_to_walls.md): brushed stainless, 3 mm,
# twelve floor buttons in three columns (ADA 407.4: 3/4 in buttons at least,
# their raised numeral and braille on a plate to the left of each), door open
# and close, the alarm under them, the fire service key switch at the top,
# four tamper-proof screws. Its frame: the wall at z = 0, the plate's middle.
EP_W, EP_H, EP_T = 0.165, 0.360, 0.003
EP_COL = (-0.041, 0.009, 0.059)
EP_ROW = (0.080, 0.035, -0.010, -0.055)        # the floor buttons, top row first
EP_DOOR = ((-0.022, -0.105), (0.028, -0.105))
EP_ALARM = (0.003, -0.150)
EP_KEY = (0.040, 0.142)
EP_BTN_R, EP_RING_R, EP_BEZ_R = 0.0096, 0.0112, 0.0135


def ep_buttons():
    for y in EP_ROW:
        for x in EP_COL:
            yield x, y
    for x, y in EP_DOOR:
        yield x, y


def elevator_panel_plate():
    s = rbox(EP_W, EP_H, EP_T, r=0.0012, seg=1).move(0, 0, EP_T / 2)
    return s


def elevator_panel_bezels():
    out = Shape()
    for x, y in ep_buttons():
        b = lathe([(EP_RING_R, 0.0), (EP_BEZ_R, 0.0), (EP_BEZ_R, 0.0006), (EP_BEZ_R - 0.0008, 0.0016), (EP_RING_R + 0.0003, 0.0018),
                   (EP_RING_R, 0.0010)], seg=20, cap_bottom=False, cap_top=False)
        out = out + b.rot(x=math.pi / 2).move(x, y, EP_T)
    ax, ay = EP_ALARM
    b = lathe([(0.0135, 0.0), (0.0165, 0.0), (0.0165, 0.0008), (0.0157, 0.0020), (0.0138, 0.0022), (0.0135, 0.0012)], seg=24,
              cap_bottom=False, cap_top=False)
    out = out + b.rot(x=math.pi / 2).move(ax, ay, EP_T)
    # the key switch's escutcheon
    kx, ky = EP_KEY
    k = cyl(0.0125, 0.0135, 0.003, seg=20, bevel_r=0.0008, bevel_seg=1).rot(x=math.pi / 2).move(kx, ky, EP_T + 0.0015)
    return out + k


def elevator_panel_rings():
    """The diffusers round each button that light when it is pressed."""
    out = Shape()
    for x, y in ep_buttons():
        r = lathe([(EP_BTN_R, 0.0), (EP_RING_R, 0.0), (EP_RING_R, 0.0009), (EP_BTN_R, 0.0009)], seg=20, cap_bottom=False, cap_top=False)
        out = out + r.rot(x=math.pi / 2).move(x, y, EP_T)
    return out


def elevator_panel_halo():
    """One lit ring (a lamp), its middle at the origin, its back at z = 0."""
    return lathe([(EP_BTN_R + 0.0002, 0.0), (EP_RING_R - 0.0002, 0.0), (EP_RING_R - 0.0002, 0.0011), (EP_BTN_R + 0.0002, 0.0011)], seg=20,
                 cap_bottom=False, cap_top=False).rot(x=math.pi / 2)


def elevator_panel_buttons():
    out = Shape()
    for x, y in ep_buttons():
        b = lathe([(0.0, 0.0024), (EP_BTN_R * 0.7, 0.0026), (EP_BTN_R - 0.0007, 0.0030), (EP_BTN_R, 0.0024), (EP_BTN_R, 0.0)], seg=20,
                  cap_bottom=True, cap_top=False)
        out = out + b.rot(x=math.pi / 2).move(x, y, EP_T)
    return out


def elevator_panel_alarm():
    ax, ay = EP_ALARM
    b = lathe([(0.0, 0.0034), (0.0095, 0.0036), (0.0128, 0.0040), (0.0134, 0.0032), (0.0134, 0.0)], seg=24, cap_bottom=True, cap_top=False)
    return b.rot(x=math.pi / 2).move(ax, ay, EP_T)


def elevator_panel_key():
    kx, ky = EP_KEY
    plug = cyl(0.0078, h=0.0040, seg=24, bevel_r=0.0006, bevel_seg=1).rot(x=math.pi / 2).move(kx, ky, EP_T + 0.0030 + 0.0020)
    plug = cut(plug, box(0.0012, 0.0075, 0.004).move(kx, ky + 0.0005, EP_T + 0.0072))
    return plug


def elevator_panel_screws():
    out = Shape()
    for sx in (-1, 1):
        for sy in (-1, 1):
            h = lathe([(0, 0.0012), (0.0028, 0.0011), (0.0040, 0.0006), (0.0042, 0.0)], seg=16, cap_bottom=True, cap_top=False)
            h = cut(h.rot(x=math.pi / 2), cyl(0.0006, h=0.004, seg=6, bevel_r=0).rot(x=math.pi / 2).move(-0.0018, 0, 0.0012),
                    cyl(0.0006, h=0.004, seg=6, bevel_r=0).rot(x=math.pi / 2).move(0.0018, 0, 0.0012))
            out = out + h.move(sx * (EP_W / 2 - 0.009), sy * (EP_H / 2 - 0.010), EP_T)
    return out


# ================================================ the extinguisher cabinet ===
# A surface-mounted steel cabinet for a 5 lb extinguisher (17-15/16 x 8-1/16
# x 6-1/8 in), its door a full pane in a steel frame on a piano hinge, and the
# extinguisher in it: a 5 lb stored-pressure dry chemical one (4-1/4 in
# across, about 16 in tall). Its frame: the wall at z = 0, the cabinet's middle.
EC_W, EC_H, EC_D = 0.205, 0.456, 0.156
EC_T = 0.0014                                   # 18 ga
EC_DOOR_D = 0.022                               # the door's frame, deep
EC_BORDER = 0.024
EX_R, EX_BODY = 0.054, 0.300                    # the cylinder


def ext_cabinet_tub():
    d = EC_D - EC_DOOR_D
    outer = rbox(EC_W, EC_H, d, r=0.003, seg=1).move(0, 0, d / 2)
    inner = box(EC_W - 2 * EC_T, EC_H - 2 * EC_T, d).move(0, 0, d / 2 + EC_T)
    return cut(outer, inner)


def ext_cabinet_door():
    """The door's frame: a rolled steel channel round the pane, its back open
    to the tub."""
    d0 = EC_D - EC_DOOR_D
    outer = rbox(EC_W, EC_H, EC_DOOR_D, r=0.004, seg=2).move(0, 0, d0 + EC_DOOR_D / 2)
    hole = box(EC_W - 2 * EC_BORDER, EC_H - 2 * EC_BORDER, 0.1).move(0, 0, d0)
    back = box(EC_W - 2 * EC_T, EC_H - 2 * EC_T, EC_DOOR_D - 0.0016).move(0, 0, d0 + (EC_DOOR_D - 0.0016) / 2 - 0.0005)
    return cut(outer, hole, back)


def ext_cabinet_glass():
    w, h = EC_W - 2 * EC_BORDER + 0.012, EC_H - 2 * EC_BORDER + 0.012
    return box(w, h, 0.003).move(0, 0, EC_D - 0.004)


def ext_cabinet_hardware():
    """The piano hinge down the door's left edge, and the pull on its right."""
    x = -EC_W / 2 - 0.0028
    hinge = cyl(0.0028, h=EC_H - 0.010, seg=10, bevel_r=0.0006, bevel_seg=1).move(x, 0, EC_D - EC_DOOR_D + 0.002)
    leaf = box(0.0012, EC_H - 0.010, 0.012).move(x + 0.0016, 0, EC_D - EC_DOOR_D + 0.006)
    knuck = Shape()
    for k in range(9):
        y = -EC_H / 2 + 0.03 + k * (EC_H - 0.06) / 8
        knuck = knuck + box(0.0066, 0.0008, 0.0066).move(x, y, EC_D - EC_DOOR_D + 0.002)
    px = EC_W / 2 - EC_BORDER / 2
    pts = [Vector((px, 0.032, EC_D)), Vector((px, 0.032, EC_D + 0.018)), Vector((px, -0.032, EC_D + 0.018)), Vector((px, -0.032, EC_D))]
    pull = tube(chaikin(pts, 2), 0.0042, seg=10)
    roses = Shape()
    for y in (0.032, -0.032):
        roses = roses + cyl(0.0065, h=0.003, seg=14, bevel_r=0.0008, bevel_seg=1).rot(x=math.pi / 2).move(px, y, EC_D + 0.0015)
    return hinge + leaf + knuck + pull + roses


EX_WALL = (0.012, EX_BODY - 0.050)               # the straight wall, foot to shoulder


def extinguisher_body():
    """The cylinder's straight wall, open at both ends, its axis +y (the
    'cyl' texture wraps it: u = 0.5 faces -z, where its label's middle is)."""
    y0, y1 = EX_WALL
    return cyl(EX_R, h=y1 - y0, seg=40, open_ended=True).move(0, (y0 + y1) / 2, 0)


def extinguisher_ends():
    """Its foot ring and drawn bottom, and the shoulder up to the neck."""
    r = EX_R
    y0, y1 = EX_WALL
    foot = lathe([(0.0, 0.004), (r - 0.010, 0.002), (r - 0.004, 0.0), (r - 0.001, 0.004), (r, y0)], seg=40, cap_bottom=False, cap_top=False)
    top = lathe([(r, y1), (r - 0.004, EX_BODY - 0.030), (r - 0.014, EX_BODY - 0.014), (r - 0.030, EX_BODY - 0.003), (0.019, EX_BODY),
                 (0.0, EX_BODY)], seg=40, cap_bottom=False, cap_top=False)
    return foot + top


def extinguisher_valve():
    """The neck collar, the forged valve, the carrying handle and the lever
    over it (out along -x), the gauge's case facing -z, the pin's ring."""
    y0 = EX_BODY
    collar = cyl(0.021, h=0.014, seg=24, bevel_r=0.0015, bevel_seg=1).move(0, y0 + 0.006, 0)
    body = rbox(0.030, 0.042, 0.034, r=0.005, seg=2).move(0, y0 + 0.032, 0)
    outlet = cyl(0.0075, h=0.024, seg=12, bevel_r=0.001, bevel_seg=1).rot(z=math.pi / 2).move(0.022, y0 + 0.030, 0)
    def bar(pts, w, t):
        rings = []
        for p in chaikin(pts, 2):
            rings.append([p + Vector((0, dy, dz)) for dy, dz in ((-t / 2, -w / 2), (-t / 2, w / 2), (t / 2, w / 2), (t / 2, -w / 2))])
        return bevel(loft(rings), min(w, t) * 0.3, seg=1)
    handle = bar([Vector((0.0, y0 + 0.028, 0)), Vector((-0.040, y0 + 0.030, 0)), Vector((-0.100, y0 + 0.016, 0)), Vector((-0.108, y0 + 0.010, 0))],
                 0.020, 0.0045)
    lever = bar([Vector((0.008, y0 + 0.050, 0)), Vector((-0.040, y0 + 0.058, 0)), Vector((-0.095, y0 + 0.046, 0)), Vector((-0.104, y0 + 0.040, 0))],
                0.020, 0.0045)
    gauge = cyl(0.0125, h=0.010, seg=20, bevel_r=0.0015, bevel_seg=1).rot(x=math.pi / 2).move(0, y0 + 0.034, -0.020)
    ring = torus(0.009, 0.0012, seg=16, tseg=6).rot(y=math.pi / 2).move(0.017, y0 + 0.054, 0.0)
    pin = cyl(0.0012, h=0.040, seg=6, bevel_r=0).rot(x=math.pi / 2).move(0.004, y0 + 0.045, 0.0)
    return collar + body + outlet + handle + lever + gauge + ring + pin


def extinguisher_seal():
    """The tamper seal: a plastic tie from the pin's ring round the lever."""
    y0 = EX_BODY
    pts = [Vector((0.017, y0 + 0.046, 0.0)), Vector((0.012, y0 + 0.052, 0.006)), Vector((0.002, y0 + 0.058, 0.008)), Vector((-0.004, y0 + 0.056, 0.0)),
           Vector((0.004, y0 + 0.050, -0.006)), Vector((0.014, y0 + 0.047, -0.004))]
    tie = tube(chaikin(pts, 2), 0.0009, seg=5)
    tag = rbox(0.010, 0.006, 0.0015, r=0.0005, seg=1).move(0.020, y0 + 0.040, 0.0)
    return tie + tag


def extinguisher_hose():
    """The hose from the outlet over the shoulder and down the cylinder's
    side, its nozzle held in the band; the band round the cylinder."""
    y0 = EX_BODY
    r = EX_R
    pts = [Vector((0.034, y0 + 0.030, 0.0)), Vector((0.052, y0 + 0.022, 0.002)), Vector((0.062, y0 - 0.010, 0.006)), Vector((r + 0.008, y0 - 0.080, 0.010)),
           Vector((r + 0.009, 0.120, 0.012))]
    hose = tube(chaikin(pts, 3), 0.0068, seg=10)
    nozzle = cyl(0.0060, 0.0085, 0.050, seg=12, bevel_r=0.001, bevel_seg=1).move(r + 0.009, 0.095, 0.012)
    band = cyl(r + 0.0012, h=0.014, seg=40, bevel_r=0, open_ended=True).move(0, 0.100, 0)
    band = band + cyl(r + 0.0005, h=0.014, seg=40, bevel_r=0, open_ended=True).move(0, 0.100, 0)
    clip = rbox(0.012, 0.016, 0.022, r=0.002, seg=1).move(r + 0.004, 0.100, 0.012)
    return hose + nozzle + band + clip


# ===================================================== the breaker panel ===
# A 6-space main lug load centre on the surface (12-9/16 in tall, 4.2 in
# deep with its cover): the steel enclosure, the front cover with its dead
# front, the door over the breakers, six single-pole breakers in two columns.
# Its frame: the wall at z = 0, the enclosure's middle.
FB_W, FB_H, FB_D = 0.215, 0.320, 0.083
FB_COVER = (0.229, 0.334, 0.0035)
FB_DOOR = (0.150, 0.198)                      # w, h of the door, hinged on its left
FB_DOOR_C = (0.0, -0.032)
FB_COL = (-0.036, 0.036)
FB_ROW = (0.000, -0.026, -0.052)
FB_FRONT = FB_D + FB_COVER[2]


def fuse_box_tub():
    s = rbox(FB_W, FB_H, FB_D, r=0.003, seg=1).move(0, 0, FB_D / 2)
    # knockouts on its top: two rings pressed in, one taken out
    kos = Shape()
    for x, r in ((-0.050, 0.0110), (0.0, 0.0140), (0.050, 0.0110)):
        kos = kos + cyl(r + 0.0015, h=0.0012, seg=20, bevel_r=0).move(x, FB_H / 2 + 0.0003, FB_D * 0.5)
    hole = cyl(0.0110, h=0.01, seg=20, bevel_r=0).move(0.050, FB_H / 2, FB_D * 0.5)
    return cut(s, hole) + kos


def fuse_box_cover():
    cw, ch, ct = FB_COVER
    s = rbox(cw, ch, ct, r=0.0012, seg=1).move(0, 0, FB_D + ct / 2)
    # the turned-back edge of the cover
    rim = cut(rbox(cw, ch, 0.010, r=0.002, seg=1), box(cw - 0.003, ch - 0.003, 0.02)).move(0, 0, FB_D - 0.003)
    # the door's opening and, through it, the dead front's two slots
    dw, dh = FB_DOOR
    dx, dy = FB_DOOR_C
    opening = box(dw + 0.004, dh + 0.004, 0.01).move(dx, dy, FB_D + ct)
    s = cut(s + rim, opening)
    dead = rbox(dw + 0.003, dh + 0.003, 0.0012, r=0.0004, seg=1).move(dx, dy, FB_D + 0.0006)
    slots = [box(0.064, 0.080, 0.01).move(x, -0.026, FB_D) for x in FB_COL]
    dead = cut(dead, *slots)
    return s + dead


def fuse_box_breakers():
    out = Shape()
    for x in FB_COL:
        for y in FB_ROW:
            b = rbox(0.060, 0.0240, 0.020, r=0.0012, seg=1).move(x, y, FB_D - 0.008)
            out = out + b
    return out


def fuse_box_handle():
    """A breaker's toggle: its pivot at the origin, standing out along +z,
    tipped about y for on and off."""
    s = rbox(0.0105, 0.0175, 0.013, r=0.0022, seg=2).move(0, 0, 0.0065)
    ribs = Shape()
    for k in (-1, 0, 1):
        ribs = ribs + box(0.0115, 0.0012, 0.0016).move(0, k * 0.004, 0.0128)
    return s + ribs


def fuse_box_door():
    """The door, in its own frame: its hinge along y at x = 0 and z = 0, the
    door across +x, its front at z = 0; the edges folded back, the latch."""
    dw, dh = FB_DOOR
    t = 0.0012
    s = box(dw, dh, t).move(dw / 2, 0, -t / 2)
    for (w, h, x, y) in ((dw, t, dw / 2, dh / 2 - t / 2), (dw, t, dw / 2, -dh / 2 + t / 2), (t, dh, dw - t / 2, 0)):
        s = s + box(w, h, 0.006).move(x, y, -0.003)
    latch = rbox(0.010, 0.030, 0.0035, r=0.0012, seg=1).move(dw - 0.012, 0, 0.0015)
    knuckles = Shape()
    for k in range(4):
        y = -dh / 2 + 0.02 + k * (dh - 0.04) / 3
        knuckles = knuckles + cyl(0.0022, h=0.018, seg=8, bevel_r=0.0004, bevel_seg=1).move(0, y, -0.0012)
    return s + latch + knuckles


def fuse_box_screws():
    out = Shape()
    cw, ch, ct = FB_COVER
    for (x, y) in ((-cw / 2 + 0.010, ch / 2 - 0.012), (cw / 2 - 0.010, ch / 2 - 0.012), (-cw / 2 + 0.010, -ch / 2 + 0.012), (cw / 2 - 0.010, -ch / 2 + 0.012)):
        out = out + screw_head(0.0040, 0.0018).rot(x=math.pi / 2).move(x, y, FB_D + ct)
    return out


# ==================================================== the pull station's lamp ===
def alarm_pull_led():
    """The LED of an addressable pull station: a 3 mm lens in its bezel, the
    back at z = 0."""
    bez = lathe([(0.0022, 0.0), (0.0036, 0.0), (0.0036, 0.0008), (0.0026, 0.0013)], seg=16, cap_bottom=False, cap_top=False)
    lens = lathe([(0.0, 0.0024), (0.0012, 0.0021), (0.0021, 0.0013), (0.0023, 0.0)], seg=16, cap_bottom=False, cap_top=False)
    return (bez + lens).rot(x=math.pi / 2)


# ================================================================== parts ===
CYL_FRAMED = {'Real__extCabinetExBody'}         # textured round their axis


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    kind = 'cyl' if name in CYL_FRAMED else 'box'
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget)


auto('Real__wallPhoneHousing', wall_phone_housing, 1800)
auto('Real__wallPhoneKeys', wall_phone_keys, 900)
auto('Real__wallPhoneHook', wall_phone_hook, 300)
auto('Real__wallPhoneHandset', wall_phone_handset, 1500)
auto('Real__wallPhoneCordOn', wall_phone_cord_on, 3600)
auto('Real__wallPhoneCordOff', wall_phone_cord_off, 2000)
auto('Real__wallPhoneCordCut', wall_phone_cord_cut, 1400)
auto('Real__emergencyPhonePlate', emergency_phone_plate, 200)
auto('Real__emergencyPhoneSign', emergency_phone_sign, 400)
auto('Real__wallPayPhoneUpper', pay_phone_upper, 900)
auto('Real__wallPayPhoneVault', pay_phone_vault, 500)
auto('Real__wallPayPhoneKeys', pay_phone_keys, 1400)
auto('Real__wallPayPhoneTrim', pay_phone_trim, 2400)
auto('Real__wallPayPhoneHandset', pay_phone_handset, 1500)
auto('Real__wallPayPhoneCord', pay_phone_cord, 4000)
auto('Real__elevatorPanelPlate', elevator_panel_plate, 200)
auto('Real__elevatorPanelBezels', elevator_panel_bezels, 3600)
auto('Real__elevatorPanelRings', elevator_panel_rings, 1800)
auto('Real__elevatorPanelHalo', elevator_panel_halo, 200)
auto('Real__elevatorPanelButtons', elevator_panel_buttons, 2200)
auto('Real__elevatorPanelAlarm', elevator_panel_alarm, 260)
auto('Real__elevatorPanelKey', elevator_panel_key, 300)
auto('Real__elevatorPanelScrews', elevator_panel_screws, 600)
auto('Real__extCabinetTub', ext_cabinet_tub, 300)
auto('Real__extCabinetDoor', ext_cabinet_door, 500)
auto('Real__extCabinetGlass', ext_cabinet_glass, 40)
auto('Real__extCabinetHardware', ext_cabinet_hardware, 1200)
auto('Real__extCabinetExBody', extinguisher_body, 200)
auto('Real__extCabinetExEnds', extinguisher_ends, 900)
auto('Real__extCabinetExValve', extinguisher_valve, 1600)
auto('Real__extCabinetExSeal', extinguisher_seal, 300)
auto('Real__extCabinetExHose', extinguisher_hose, 1600)
auto('Real__fuseBoxTub', fuse_box_tub, 600)
auto('Real__fuseBoxCover', fuse_box_cover, 600)
auto('Real__fuseBoxBreakers', fuse_box_breakers, 600)
auto('Real__fuseBoxHandle', fuse_box_handle, 200)
auto('Real__fuseBoxDoor', fuse_box_door, 500)
auto('Real__fuseBoxScrews', fuse_box_screws, 500)
auto('Real__fireAlarmPullLed', alarm_pull_led, 200)
