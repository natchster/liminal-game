"""Batch b05: office props (boards, tapes, papers, lockers, phones, keys, gadgets).

Conventions from README.md: every part is modelled in its primitive's frame
at the canonical size written in the placer. Helpers local to this batch sit
at the top (loft, ring_section, frame_bar, ...).
"""
import math
from propkit import *
from propkit import _fix_winding


# ---------------------------------------------------------------- helpers --
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
    return _fix_winding(Shape(v, f))


def rect(hx, hy, z):
    return [(-hx, -hy, z), (hx, -hy, z), (hx, hy, z), (-hx, hy, z)]


def ring_section(section, seg=32, arc=TAU):
    """Revolve a closed 2D section [(r, y), ...] around +Y: rims, washers, rings."""
    n = len(section)
    full = arc >= TAU - 1e-6
    steps = seg if full else seg + 1
    v, f = [], []
    for i in range(steps):
        a = arc * i / seg
        for (r, y) in section:
            v.append((r * math.sin(a), y, r * math.cos(a)))
    for i in range(seg):
        i2 = (i + 1) % steps if full else i + 1
        for j in range(n):
            j2 = (j + 1) % n
            f.append((i * n + j, i * n + j2, i2 * n + j2, i2 * n + j))
    if not full:
        f.append(tuple(range(n)))
        f.append(tuple(seg * n + j for j in reversed(range(n))))
    return _fix_winding(Shape(v, f))


def xz_prism(outline, h, bevel_r=0.0, seg=1, angle=50):
    """Outline [(x, z), ...] extruded along Y (centred)."""
    s = prism(outline, h).rot(x=math.pi / 2)
    return bevel(s, bevel_r, seg=seg, angle=angle) if bevel_r > 0 else s


def yz_prism(outline, length, bevel_r=0.0, seg=1, angle=50):
    """Outline [(z, y), ...] extruded along X (centred)."""
    s = prism([(-z, y) for z, y in outline], length).rot(y=math.pi / 2)
    return bevel(s, bevel_r, seg=seg, angle=angle) if bevel_r > 0 else s


def stud(r, h, seg=6, y0=0.0):
    """A low dome / screw head standing on y0 (no hidden bottom face)."""
    return lathe([(r, y0), (r * 0.8, y0 + h * 0.75), (0, y0 + h)], seg=seg, cap_bottom=False)


def frame_bar(w, h, d, f=0.075, dz=0.003, seg=3):
    """A frame bar whose front perimeter is rounded with insets that are a
    fraction of each in-plane size. The same constructor makes the horizontal
    and the vertical bars (the game stretches the part per axis), so a fixed
    fraction gives both the same rounded long edges; the ends taper gently."""
    rings = [rect(w / 2, h / 2, -d / 2)]
    for k in range(seg + 1):
        t = (math.pi / 2) * k / seg
        ix, iy = f * w * (1 - math.cos(t)), f * h * (1 - math.cos(t))
        rings.append(rect(w / 2 - ix, h / 2 - iy, d / 2 - dz + dz * math.sin(t)))
    return loft(rings)


# ---------------------------------------------------------- chalkboard ----
# 0.40 x 0.30 board in a 2 cm wooden frame; the frame bars share one ctor.
part('WallChalkboards__bar', f_box(0.44, 0.02, 0.015),
     lambda: frame_bar(0.44, 0.02, 0.015, f=0.08, dz=0.004, seg=3))

# ------------------------------------------------------------- posters ----
# thin black poster frame (1.4 cm bars, 1 cm deep)
part('InspirationalPosters__bar', f_box(0.348, 0.014, 0.010),
     lambda: frame_bar(0.348, 0.014, 0.010, f=0.07, dz=0.0025, seg=3))


# ------------------------------------------------------------ cassettes ---
def cassette_shell(w, h, d, head=1):
    """Compact-cassette shell lying flat: rounded outline, eased edges, the
    head openings in the `head` edge (+Z or -Z) and four corner screws."""
    s = xz_prism(rounded_rect(w, d, 0.004, seg=2), h, bevel_r=min(0.0009, h * 0.12), seg=1)
    zf = head * d / 2
    oh = h * 0.55
    cutters = [box(0.022, oh, 0.008).move(z=zf)]
    for sx in (-1, 1):
        cutters.append(box(0.007, oh, 0.006).move(x=sx * 0.026, z=zf))
    s = cut(s, *cutters)
    for sx in (-1, 1):
        for sz in (-1, 1):
            s = s + stud(0.0017, 0.0005, seg=5, y0=h / 2 - 0.0001).move(x=sx * (w / 2 - 0.0055), z=sz * (d / 2 - 0.0055))
    return s

part('CassetteTapes__body', f_box(0.10, 0.010, 0.06), lambda: cassette_shell(0.10, 0.010, 0.06, head=1))
part('Cassettes__shell', f_box(0.100, 0.006, 0.062), lambda: cassette_shell(0.100, 0.006, 0.062, head=-1))


# ----------------------------------------------------------- newspapers ---
def newspaper():
    """A folded newspaper: rounded fold along the +Z edge, loose sheets
    showing at the open -Z edge, a slight wave through the stack."""
    W, T, D = 0.20, 0.008, 0.14
    hz, ht = D / 2, T / 2
    zc = hz - ht                        # centre of the fold's round
    def section(x, k):
        pts = [(-hz + 0.0005, -ht), (-0.03, -ht), (0.02, -ht), (zc, -ht)]
        for a in (-60, -30, 0, 30, 60):
            r = math.radians(a)
            pts.append((zc + ht * math.cos(r), ht * math.sin(r)))
        pts += [(zc, ht), (0.02, ht), (-0.03, ht), (-hz + 0.0005, ht * 0.95)]
        # open edge: the sheets show as a ragged stack
        edge = [0.75, 0.55, 0.3, 0.08, -0.2, -0.45, -0.75]
        for i, e in enumerate(edge):
            jag = 0.0012 * (0.5 + 0.5 * math.sin(i * 2.3 + k * 1.7))
            pts.append((-hz + jag, e * ht))
        wave = lambda z: 0.0007 * math.sin(x * 21 + 1.3) * math.cos(z * 18)
        return [(x, y + wave(z), z) for z, y in pts]
    xs = [-W / 2 + W * i / 8 for i in range(9)]
    return loft([section(x, k) for k, x in enumerate(xs)])

part('NewspaperStacks__paper', f_box(0.20, 0.008, 0.14), newspaper)


# -------------------------------------------------------------- lockers ---
def locker_body():
    """Steel locker: rounded carcass, door seam, handle pocket, louvre hoods
    over the vent slots, two hinge knuckles, recessed toe kick."""
    W, H, D = 0.30, 1.60, 0.30
    fz = D / 2
    b = rbox(W, H, D, r=0.005, seg=2)
    x0, x1, y0, y1 = -0.136, 0.136, -0.735, 0.772
    g = 0.0025
    cutters = [
        box(g, y1 - y0 + g, 0.008).move(x=x0, y=(y0 + y1) / 2, z=fz),
        box(g, y1 - y0 + g, 0.008).move(x=x1, y=(y0 + y1) / 2, z=fz),
        box(x1 - x0 + g, g, 0.008).move(y=y0, z=fz),
        box(x1 - x0 + g, g, 0.008).move(y=y1, z=fz),
        rbox(0.024, 0.05, 0.014, r=0.004, seg=2).move(x=0.10, y=0.10, z=fz),
        box(W + 0.02, 0.06, 0.03).move(y=-H / 2 + 0.02, z=fz),
    ]
    b = cut(b, *cutters)
    for i in range(4):
        yi = 0.64 + i * 0.010
        b = b + yz_prism([(fz - 0.0005, yi + 0.002), (fz + 0.0028, yi + 0.0024), (fz - 0.0005, yi + 0.0078)], 0.15)
    for yh in (-0.52, 0.55):
        b = b + cyl(0.0042, 0.0042, 0.06, seg=10, bevel_r=0.001).move(x=x0 - 0.001, y=yh, z=fz + 0.0008)
    return b

def locker_handle():
    """Lift latch: a back plate with a lever and a finger lip at the bottom."""
    plate = rbox(0.010, 0.030, 0.003, r=0.001, seg=1).move(z=-0.0035)
    lever = rbox(0.0068, 0.024, 0.005, r=0.0016, seg=2).move(y=0.001, z=0.0)
    lip = rbox(0.0092, 0.0055, 0.0045, r=0.0015, seg=2).move(y=-0.0115, z=0.00275)
    return plate + lever + lip

part('MetalLockers__body', f_box(0.30, 1.60, 0.30), locker_body)
part('MetalLockers__handle', f_box(0.010, 0.030, 0.010), locker_handle)


# ------------------------------------------------------------ mail/envelopes
def envelope():
    """Envelope seen from the back: thin sheet, the V seal flap standing a
    hair proud, both ends lifting a little."""
    W, D = 0.12, 0.06
    sheet = slab(W, D, 0.0007, 6, 3).rot(x=-math.pi / 2).move(y=-0.00015)
    flap = xz_prism([(-W / 2, -D / 2), (W / 2, -D / 2), (W / 2, -D / 2 + 0.004), (0.005, 0.007),
                     (-0.005, 0.007), (-W / 2, -D / 2 + 0.004)], 0.0003).move(y=0.00035)
    s = sheet + flap
    return s.warp(lambda p: (p.x, p.y + 0.0014 * (p.x / (W / 2)) ** 2 + 0.0004 * (p.z / (D / 2)) ** 2, p.z))

part('DroppedMailScatters__envelope', f_box(0.12, 0.001, 0.06), envelope)


# ---------------------------------------------------------- bookshelves ---
def shelf_back():
    """Tongue-and-groove back panel: eased edges and V joints every 12 cm."""
    s = rbox(0.60, 0.16, 0.014, r=0.0015, seg=1)
    grooves = [box(0.003, 0.2, 0.003).rot(z=0, y=math.pi / 4).move(x=x, z=0.007) for x in (-0.18, -0.06, 0.06, 0.18)]
    return cut(s, *grooves)

def shelf_board():
    """Shelf board with a bullnose front edge (+Z)."""
    hz, ht = 0.08, 0.007
    sec = [(-hz, -ht), (hz - ht, -ht)]
    for a in (-60, -30, 0, 30, 60):
        r = math.radians(a)
        sec.append((hz - ht + ht * math.cos(r), ht * math.sin(r)))
    sec += [(hz - ht, ht), (-hz, ht)]
    return yz_prism(sec, 0.60, bevel_r=0.0012, seg=1, angle=60)

def shelf_side():
    """Side panel with a cove cut into its front edge."""
    hy, hz = 0.08, 0.08
    sec = [(-hz, -hy), (hz, -hy), (hz, -0.058)]
    for i in range(1, 8):
        t = i / 8
        sec.append((hz - 0.032 * math.sin(math.pi * t), -0.058 + 0.116 * t))
    sec += [(hz, 0.058), (hz, hy), (-hz, hy)]
    s = prism([(-z, y) for z, y in sec], 0.014).rot(y=math.pi / 2)
    return bevel(s, 0.0015, seg=1, angle=50)

def shelf_book():
    """Hardcover standing on the shelf, spine to +Z: two boards wrapping a
    recessed page block, rounded spine, hinge grooves beside it."""
    w, h, d = 0.021, 0.125, 0.08
    hx, hz, t = w / 2, d / 2, 0.0016
    zs = hz - 0.006                                  # where the spine's round starts
    outer = [(-hx, -hz), (-hx, zs - 0.004), (-hx + 0.0005, zs - 0.003), (-hx, zs - 0.002), (-hx, zs)]
    for i in range(1, 8):
        a = math.pi * i / 8
        outer.append((-hx * math.cos(a), zs + 0.006 * math.sin(a)))
    outer += [(hx, zs), (hx, zs - 0.002), (hx - 0.0005, zs - 0.003), (hx, zs - 0.004), (hx, -hz)]
    ix = hx - t
    inner = [(ix, -hz), (ix, zs)]
    for i in range(1, 8):
        a = math.pi * i / 8
        inner.append((ix * math.cos(a), zs + 0.0045 * math.sin(a)))
    inner += [(-ix, zs), (-ix, -hz)]
    cover = xz_prism(outer + inner, h)
    pages = box(2 * ix + 0.0004, h - 0.005, hz - 0.0015 + zs).move(z=(-hz + 0.0015 + zs) / 2)
    return cover + pages

part('Bookshelves__back', f_box(0.60, 0.16, 0.014), shelf_back)
part('Bookshelves__board', f_box(0.60, 0.014, 0.16), shelf_board)
part('Bookshelves__side', f_box(0.014, 0.16, 0.16), shelf_side)
part('Bookshelves__book', f_box(0.021, 0.125, 0.08), shelf_book)


# ------------------------------------------------------------ key chain ---
def split_ring(R, r, turns=1.85, n=26, seg=5):
    """Split key ring: a wire coiled a little under two turns in the XY
    plane, the two turns lying side by side along Z."""
    pts = []
    for i in range(n + 1):
        a = TAU * turns * i / n
        pts.append((R * math.cos(a), R * math.sin(a), -r * 0.5 + r * i / n))
    return tube(pts, r * 0.5, seg=seg)

def key_blade():
    """Key blade along Y (shoulder +Y, tip -Y), 2 mm thick along X, cut
    bitting on the +Z edge and a milled groove down the side."""
    sec = [(-0.002, 0.025), (0.002, 0.025), (0.002, 0.021)]
    for k in range(6):
        y = 0.019 - k * 0.0055
        sec += [(0.0006 + 0.0005 * (k % 3), y), (0.002, y - 0.0027)]
    sec += [(0.002, -0.017), (0.0, -0.025), (-0.002, -0.021)]
    s = yz_prism(sec, 0.002)
    return cut(s, box(0.0008, 0.042, 0.0009).move(x=0.001, y=-0.002, z=-0.0006))

def ring_bow(R, r):
    """A flat ring bow (skeleton-key style) in the XY plane."""
    ri, ro, t = R - r * 0.9, R + r, r * 0.8
    sec = [(ri, -t), (ro - 0.0011, -t), (ro, 0.0), (ro - 0.0011, t), (ri, t)]
    return ring_section(sec, seg=10).rot(x=math.pi / 2)

part('KeychainKeys__ring', f_torus(0.014, 0.002), lambda: split_ring(0.015, 0.002, n=33, seg=4))
part('KeychainKeys__blade', f_cyl(0.002, 0.002, 0.05), key_blade)
part('KeychainKeys__bow', f_torus(0.008, 0.002), lambda: ring_bow(0.008, 0.002))
part('KeychainKeys__tooth', f_box(0.004, 0.005, 0.002), lambda: rbox(0.004, 0.005, 0.002, r=0.0005, seg=1))


# ---------------------------------------------------------- small radio ---
def radio_body():
    """Transistor radio case: rounded outline, eased edges, a raised bezel
    round the front and the seam where the two case halves meet."""
    W, H, D = 0.12, 0.07, 0.04
    outline = rounded_rect(W, H, 0.012, seg=3)
    body = bevel(prism(outline, D - 0.002).move(z=-0.001), 0.0035, seg=2, angle=50)
    rim_o = prism(rounded_rect(W - 0.002, H - 0.002, 0.011, seg=3), 0.002)
    rim_i = prism(rounded_rect(W - 0.010, H - 0.010, 0.007, seg=3), 0.004)
    bezel = cut(rim_o, rim_i).move(z=D / 2 - 0.001)
    seam_o = prism(rounded_rect(W + 0.004, H + 0.004, 0.014, seg=3), 0.0009)
    seam_i = prism(rounded_rect(W - 0.0012, H - 0.0012, 0.0114, seg=3), 0.002)
    body = cut(body, cut(seam_o, seam_i).move(z=-0.008))
    return body + bezel

def radio_knob():
    """Knurled tuning knob: eight grip lobes, chamfered top."""
    r, h = 0.006, 0.005
    def ring(y, k):
        pts = []
        for i in range(16):
            a = TAU * i / 16
            rr = (r if i % 2 == 0 else r * 0.91) * k
            pts.append((rr * math.cos(a), y, rr * math.sin(a)))
        return pts
    return loft([ring(-h / 2, 1.0), ring(h / 2 - 0.0012, 1.0), ring(h / 2, 0.8)])

def radio_antenna():
    """Telescopic antenna: swivel foot, three sections, ball tip."""
    L = 0.12
    prof = [(0, -L / 2), (0.001, -L / 2), (0.001, -L / 2 + 0.004), (0.0009, -L / 2 + 0.0045),
            (0.0009, -0.015), (0.00075, -0.0145), (0.00075, 0.02), (0.0006, 0.0205),
            (0.0006, L / 2 - 0.0026), (0.0009, L / 2 - 0.0022), (0.001, L / 2 - 0.001), (0.0007, L / 2 - 0.0002), (0, L / 2)]
    return lathe(prof, seg=8)

part('SmallRadios__body', f_box(0.12, 0.07, 0.04), radio_body)
part('SmallRadios__knob', f_cyl(0.006, 0.006, 0.005), radio_knob)
part('SmallRadios__antenna', f_cyl(0.001, 0.001, 0.12), radio_antenna)


# ------------------------------------------------------------- pay phone --
def payphone_body():
    """Payphone housing: rounded steel box, a raised keypad plate (the keys
    sit on it), a boss for the hook, coin-return pocket, instruction-card
    recess and four security screws. Front is +Z."""
    W, H, D = 0.18, 0.40, 0.10
    fz = D / 2
    b = bevel(prism(rounded_rect(W, H, 0.008, seg=3), D), 0.004, seg=2, angle=50)
    b = cut(b,
            rbox(0.05, 0.03, 0.02, r=0.003, seg=1).move(x=0.04, y=-0.155, z=fz),        # coin return
            box(0.075, 0.09, 0.004).move(x=0.047, y=0.075, z=fz))                          # card recess
    pad = bevel(prism(rounded_rect(0.098, 0.128, 0.006, seg=2), 0.006), 0.0015, seg=1, angle=50)
    b = b + pad.move(y=-0.055, z=fz + 0.0015)
    boss = rbox(0.05, 0.03, 0.006, r=0.0015, seg=1).move(x=-0.06, y=0.10, z=fz + 0.0005)
    b = b + boss
    for sx in (-1, 1):
        for sy in (-1, 1):
            b = b + stud(0.004, 0.0015, seg=8).rot(x=math.pi / 2).move(x=sx * 0.072, y=sy * 0.182, z=fz - 0.0002)
    return b

def payphone_coin_slot():
    """Coin-slot bezel with the slit cut into its face."""
    s = rbox(0.10, 0.010, 0.014, r=0.002, seg=2)
    return cut(s, box(0.032, 0.0026, 0.012).move(x=0.012, z=0.007))

def payphone_handset():
    """G-style handset along Y: slim handle on the +Z side, ear and mouth
    cups hanging to -Z at both ends."""
    cup_prof = [(0, -0.0095), (0.0115, -0.0095), (0.0135, -0.008), (0.014, -0.005), (0.014, 0.003),
                (0.012, 0.0075), (0.007, 0.0095), (0, 0.0098)]
    cup = lathe(cup_prof, seg=16).rot(x=math.pi / 2).move(z=-0.0042)
    s = cup.copy().move(y=0.0555) + cup.copy().move(y=-0.0555)
    handle = rbox(0.017, 0.118, 0.013, r=0.005, seg=2).move(z=0.0075)
    return s + handle

def payphone_cradle():
    """Hook switch: a block with two horns and a saddle for the handset."""
    out = [(-0.02, -0.007), (0.02, -0.007), (0.02, 0.007), (0.0145, 0.007), (0.011, 0.0015),
           (-0.011, 0.0015), (-0.0145, 0.007), (-0.02, 0.007)]
    return prism(out, 0.014, bevel_r=0.0015, bevel_seg=2)

def ribbed_arc(R, r, arc, n, tseg=4, rib=0.7):
    """Coiled cord along an arc in the XY plane: rings alternate fat/thin."""
    rings = []
    for i in range(n + 1):
        a = arc * i / n
        c, s_ = math.cos(a), math.sin(a)
        rr = r if i % 2 == 0 else r * rib
        ring = []
        for j in range(tseg):
            b = TAU * (j + 0.5) / tseg
            ring.append(((R + rr * math.cos(b)) * c, (R + rr * math.cos(b)) * s_, rr * math.sin(b)))
        rings.append(ring)
    return loft(rings)

part('WallPayPhones__body', f_box(0.18, 0.40, 0.10), payphone_body)
part('WallPayPhones__coinSlot', f_box(0.10, 0.010, 0.014), payphone_coin_slot)
part('WallPayPhones__button', f_cyl(0.008, 0.008, 0.003),
     lambda: rbox(0.0158, 0.003, 0.0158, r=0.0012, seg=1))
part('WallPayPhones__cradle', f_box(0.04, 0.014, 0.014), payphone_cradle)
part('WallPayPhones__handset', f_cyl(0.014, 0.014, 0.14), payphone_handset)
part('WallPayPhones__cord', f_bounds((-0.027, -0.027, -0.002), (0.027, 0.027, 0.002)),
     lambda: ribbed_arc(0.025, 0.002, math.pi * 1.5, 33, tseg=4))


# ---------------------------------------------------------- cell phones ---
def flip_half(lid):
    """One half of a closed flip phone. The -Z end is a quarter round so the
    two halves together make the hinge barrel; knuckle grooves are cut in it."""
    hy, hz = 0.003, 0.0275
    sgn = 1 if lid else -1
    zc, R = -hz + 0.006, 0.006
    # profile (z, y) for the base (the lid is its mirror in y)
    sec = [(hz - 0.001, -hy), (hz, -hy + 0.001), (hz, hy - 0.0004), (hz - 0.0004, hy), (-hz, hy)]
    for a in (195, 210, 225, 240, 255):
        r = math.radians(a)
        sec.append((zc + R * math.cos(r), hy + R * math.sin(r)))
    sec.append((zc, -hy))
    if lid:
        sec = [(z, -y) for z, y in sec]
    s = bevel(yz_prism(sec, 0.030), 0.0012, seg=2, angle=50)
    cuts = [box(0.0007, 0.02, 0.009).move(x=sx * 0.0068, z=-hz + 0.003) for sx in (-1, 1)]
    if lid:
        cuts.append(rbox(0.014, 0.002, 0.012, r=0.001, seg=1).move(y=hy, z=-0.004))   # outer display window
    return cut(s, *cuts)

def slab_phone():
    """Candy-bar smartphone: rounded outline and edges, earpiece slot, home
    button dimple, side buttons."""
    W, T, L = 0.040, 0.008, 0.075
    def ring(y, inset):
        return [(x, y, z) for x, z in rounded_rect(W - 2 * inset, L - 2 * inset, 0.0065 - inset, seg=2)]
    rings = [ring(-T / 2, 0.0012), ring(-T / 2 + 0.0012, 0.0),
             ring(T / 2 - 0.0016, 0.0), ring(T / 2 - 0.0007, 0.0003), ring(T / 2, 0.0011)]
    s = loft(rings)
    s = cut(s, rbox(0.009, 0.004, 0.0012, r=0.0005, seg=1).move(y=T / 2, z=-0.0325),
            cyl(0.0034, 0.0034, 0.0012, seg=8, bevel_r=0).move(y=T / 2, z=0.0322))
    s = s + rbox(0.001, 0.0022, 0.012, r=0.0004, seg=1).move(x=-W / 2, y=0.0003, z=-0.012)
    s = s + rbox(0.001, 0.0022, 0.007, r=0.0004, seg=1).move(x=W / 2, y=0.0003, z=-0.018)
    return s

part('DroppedCellPhones__flipBase', f_box(0.030, 0.006, 0.055), lambda: flip_half(False))
part('DroppedCellPhones__flipLid', f_box(0.030, 0.006, 0.055), lambda: flip_half(True))
part('DroppedCellPhones__slab', f_box(0.040, 0.008, 0.075), slab_phone)


# ------------------------------------------------------------ paperclip ---
def gem_clip():
    """A gem paperclip (three nested bends) laid along the diagonal of the
    primitive's square, so it fills the frame whatever the clip's turn."""
    def arc(cx, cy, r, a0, a1, n=4):
        return [(cx + r * math.cos(a0 + (a1 - a0) * i / n), cy + r * math.sin(a0 + (a1 - a0) * i / n)) for i in range(n + 1)]
    uv = [(0.64, 0.04)]
    uv += arc(0.20, -0.02, 0.06, math.pi / 2, 3 * math.pi / 2)          # small bend, left
    uv += arc(0.86, 0.025, 0.105, -math.pi / 2, math.pi / 2)            # right bend
    uv += arc(0.14, -0.005, 0.135, math.pi / 2, 3 * math.pi / 2)        # big bend, left
    uv.append((0.74, -0.14))
    c, s_ = math.cos(math.pi / 4), math.sin(math.pi / 4)
    pts = [((u - 0.48) * c - v * s_, (u - 0.48) * s_ + v * c, 0.0) for u, v in uv]
    w = tube(pts, 0.026, seg=3)
    return w.fit((-0.009, -0.009, -0.001), (0.009, 0.009, 0.001))

part('PaperclipClusters__clip', f_bounds((-0.009, -0.009, -0.001), (0.009, 0.009, 0.001)), gem_clip)


# ----------------------------------------------------------------- pens ---
def pen_cap(y0, y1):
    cap = lathe([(0, y0), (0.0049, y0), (0.005, y0 + 0.001), (0.005, y1 - 0.004), (0.0045, y1 - 0.0015),
                 (0.0033, y1 - 0.0002), (0, y1)], seg=12)
    clip = rbox(0.0026, 0.028, 0.0011, r=0.0004, seg=1).move(y=y1 - 0.02, z=0.0056)
    knob = rbox(0.003, 0.004, 0.0022, r=0.0007, seg=1).move(y=y1 - 0.0055, z=0.005)
    return cap + clip + knob

def whole_pen():
    """Stick pen: hexagonal barrel, round cap with a pocket clip, end plug."""
    barrel = lathe([(0, -0.058), (0.0047, -0.058), (0.0047, 0.016), (0, 0.016)], seg=6)
    plug = lathe([(0, -0.06), (0.0026, -0.06), (0.003, -0.0588), (0.003, -0.057), (0, -0.057)], seg=10)
    return barrel + plug + pen_cap(0.012, 0.06)

def broken_pen_half():
    """Capped half of a snapped pen: hex barrel ending in a jagged break
    with the ink tube poking out."""
    def hexring(y_of, r):
        return [(r * math.cos(TAU * i / 6), y_of(i), r * math.sin(TAU * i / 6)) for i in range(6)]
    barrel = loft([hexring(lambda i: -0.0245 + (0.0035 if i % 2 else 0.0) + 0.001 * (i == 2), 0.0047),
                   hexring(lambda i: 0.012, 0.0047)])
    ink = cyl(0.0012, 0.0012, 0.009, seg=8, bevel_r=0).move(y=-0.021)
    return barrel + ink + pen_cap(0.009, 0.025)

part('BrokenPens__pen', f_cyl(0.005, 0.005, 0.12), whole_pen)
part('BrokenPens__half', f_cyl(0.005, 0.005, 0.05), broken_pen_half)


# -------------------------------------------------------------- lanyard ---
def id_badge():
    """PVC ID card: rounded corners, strap slot punched at the -Z end."""
    s = xz_prism(rounded_rect(0.06, 0.09, 0.0035, seg=2), 0.0008)
    s = cut(s, rbox(0.014, 0.003, 0.0028, r=0.0012, seg=2).move(z=-0.0395))
    return s.warp(lambda p: (p.x, p.y + 0.0003 * (p.x / 0.03) ** 2 - 0.00005, p.z))

part('Lanyards__badge', f_box(0.06, 0.001, 0.09), id_badge)


# -------------------------------------------------------------- post-its --
def post_it(curl):
    """Sticky note: glued strip at -Z stays flat, the free end curls up."""
    s = slab(0.06, 0.06, 0.0009, 4, 6).rot(x=-math.pi / 2).move(y=-0.00005)
    def lift(p):
        t = max(0.0, (p.z + 0.005) / 0.035)
        return (p.x, p.y + curl * t * t, p.z)
    return s.warp(lift)

part('PostItStacks__note', f_box(0.06, 0.001, 0.06), lambda: post_it(0.0004))
part('PostItStacks__loose', f_box(0.06, 0.001, 0.06), lambda: post_it(0.0014))


# ------------------------------------------------------- crumpled receipt --
def paper_ball():
    s = sphere(0.02, seg=12, rings=8)
    s = noisy(s, 0.0038, scale=70, seed=3)
    s = noisy(s, 0.0015, scale=190, seed=7)
    return s.fit((-0.02, -0.02, -0.02), (0.02, 0.02, 0.02))

def paper_flap():
    """A crinkled tongue of paper sticking out of the ball."""
    s = slab(0.010, 0.014, 0.0004, 3, 3).rot(x=-math.pi / 2)
    s = s.warp(lambda p: (p.x, p.y + 0.0034 * max(0.0, (p.z + 0.007) / 0.014) ** 1.4 + 0.0006 * math.sin(p.x * 900), p.z))
    s = noisy(s, 0.0005, scale=400, seed=2)
    return s.fit((-0.005, -0.002, -0.007), (0.005, 0.002, 0.007))

part('CrumpledReceipts__ball', f_sphere(0.020), paper_ball, flat=True)
part('CrumpledReceipts__crinkle', f_box(0.010, 0.004, 0.014), paper_flap, flat=True)


# ------------------------------------------------------------ mail slot ---
def mail_plate():
    """Brass letter plate: bevelled border, a raised lip round the slot,
    a slotted screw at each end."""
    s = bevel(prism(rounded_rect(0.20, 0.06, 0.004, seg=2), 0.006), 0.0018, seg=2, angle=50)
    lip = cut(prism(rounded_rect(0.152, 0.032, 0.003, seg=2), 0.003), box(0.141, 0.021, 0.01))
    s = s + bevel(lip, 0.0006, seg=1, angle=50).move(y=0.008, z=0.002)
    for sx in (-1, 1):
        s = s + screw_head(0.0032, 0.0013).rot(x=math.pi / 2).move(x=sx * 0.088, y=0.008, z=0.0029)
    return s

def mail_flap():
    """Hinged flap: plate, rolled hinge along the top, pull lip at the bottom."""
    plate = rbox(0.14, 0.0255, 0.002, r=0.0006, seg=1).move(y=-0.0012, z=-0.0005)
    hinge = cyl(0.0015, 0.0015, 0.14, seg=10, bevel_r=0.0004).rot(z=math.pi / 2).move(y=0.0125)
    pull = rbox(0.05, 0.0025, 0.003, r=0.0008, seg=2).move(y=-0.0125)
    return plate + hinge + pull

part('MailSlots__plate', f_box(0.20, 0.06, 0.006), mail_plate)
part('MailSlots__flap', f_box(0.14, 0.028, 0.003), mail_flap)


# ------------------------------------------------------------ tape roll ---
part('TapeRolls__roll', f_cyl(0.030, 0.030, 0.020),
     lambda: ring_section([(0.0138, -0.01), (0.0286, -0.01), (0.03, -0.0086), (0.03, 0.0086),
                           (0.0286, 0.01), (0.0138, 0.01)], seg=22))
part('TapeRolls__core', f_cyl(0.014, 0.014, 0.022),
     lambda: ring_section([(0.0122, -0.011), (0.0138, -0.011), (0.014, -0.0104), (0.014, 0.0104),
                           (0.0138, 0.011), (0.0122, 0.011)], seg=18))


def moulding(w, h, prof):
    """Sweep a closed profile [(inset from the outer edge, z), ...] round a
    w x h rectangle with mitred corners: picture frames, bezels."""
    n = len(prof)
    v, f = [], []
    for d, z in prof:
        v += rect(w / 2 - d, h / 2 - d, z)
    for k in range(n):
        k2 = (k + 1) % n
        for j in range(4):
            j2 = (j + 1) % 4
            f.append((k * 4 + j, k * 4 + j2, k2 * 4 + j2, k2 * 4 + j))
    return _fix_winding(Shape(v, f))


# ------------------------------------------------------------ paperback ---
def paperback():
    """Paperback lying flat, spine at -X: thin card covers wrapping a page
    block that stops a hair short of the edges, hinge score on the cover."""
    W, T, D = 0.14, 0.008, 0.18
    hx, ht, c = W / 2, T / 2, 0.0004
    sp = -hx + 0.0012
    outer = [(hx, -ht), (sp, -ht)]
    for a in (-60, -30, 0, 30, 60):
        r = math.radians(a)
        outer.append((sp - 0.0012 * math.cos(r) - 0.0, ht * 0.75 * math.sin(r) / math.sin(math.radians(60)) * 1.0))
    outer += [(sp, ht), (-hx + 0.006, ht), (-hx + 0.0065, ht - 0.0003), (-hx + 0.007, ht), (hx, ht)]
    inner = [(hx, ht - c), (sp + 0.0004, ht - c), (sp + 0.0004, -ht + c), (hx, -ht + c)]
    cover = prism(outer + inner, D)
    pages = box(W - 0.0024, T - 2 * c + 0.0002, D - 0.0012).move(x=0.0006)
    pages = cut(pages, box(0.002, 0.0004, D).move(x=hx, y=0.0012), box(0.002, 0.0004, D).move(x=hx, y=-0.0012))
    return cover + pages

part('PaperbackBooks__book', f_box(0.14, 0.008, 0.18), paperback)


# ------------------------------------------------------------ clipboard ---
def clipboard_board():
    return xz_prism(rounded_rect(0.20, 0.28, 0.012, seg=3), 0.006, bevel_r=0.0012, seg=2)

def clipboard_clip():
    """Lever clip: riveted base plate, spring jaw arching to the paper, pivot
    barrel at the back and a flat lever with a hanging hole folded over it."""
    base = rbox(0.10, 0.0009, 0.028, r=0.0004, seg=1).move(y=-0.0035, z=-0.001)
    arc = [(-0.013, -0.0028), (-0.008, 0.0005), (-0.002, 0.0012), (0.005, 0.0), (0.011, -0.0022), (0.0148, -0.0038)]
    top = [(z, y + 0.0007) for z, y in arc]
    jaw = yz_prism(top + list(reversed(arc)), 0.072)
    barrel = cyl(0.0022, 0.0022, 0.084, seg=10, bevel_r=0).rot(z=math.pi / 2).move(y=-0.0012, z=-0.0125)
    lever = cut(prism(rounded_rect(0.05, 0.02, 0.007, seg=2), 0.0009), prism(rounded_rect(0.024, 0.008, 0.0035, seg=1), 0.004))
    lever = lever.rot(x=math.pi / 2 - 0.12).move(y=0.0028, z=-0.003)
    rivets = sum((stud(0.0022, 0.0008, seg=6, y0=-0.0031).move(x=sx * 0.038, z=-0.008) for sx in (-1, 1)), Shape())
    return base + jaw + barrel + lever + rivets

part('DroppedClipboards__board', f_box(0.20, 0.006, 0.28), clipboard_board)
part('DroppedClipboards__clip', f_box(0.10, 0.008, 0.030), clipboard_clip)


# ------------------------------------------------------ keyring with keys --
def flat_key_bow():
    """Round key head lying flat, the ring hole towards the ring (-X)."""
    s = xz_prism([(0.008 * math.cos(TAU * i / 12), 0.008 * math.sin(TAU * i / 12)) for i in range(12)], 0.002)
    return cut(s, cyl(0.0019, 0.0019, 0.004, seg=8, bevel_r=0).move(x=-0.0042))

def flat_key_shaft():
    """Blade along +X: shoulder stop at the head end, angled tip, groove."""
    out = [(-0.019, -0.0025), (-0.015, -0.0025), (-0.0145, -0.002), (0.014, -0.002), (0.019, -0.0004),
           (0.019, 0.0004), (0.0165, 0.002), (-0.0145, 0.002), (-0.015, 0.0025), (-0.019, 0.0025)]
    s = xz_prism(out, 0.002)
    return cut(s, box(0.029, 0.0008, 0.0008).move(x=0.0005, y=0.001, z=-0.0003))

def flat_key_tooth():
    return xz_prism([(-0.003, -0.0015), (0.003, -0.0015), (0.0011, 0.0015), (-0.0011, 0.0015)], 0.002)

part('KeyringWithKeys__ring', f_torus(0.020, 0.003), lambda: split_ring(0.0215, 0.003, n=33, seg=4))
part('KeyringWithKeys__bow', f_cyl(0.008, 0.008, 0.002), flat_key_bow)
part('KeyringWithKeys__shaft', f_box(0.038, 0.002, 0.005), flat_key_shaft)
part('KeyringWithKeys__tooth', f_box(0.006, 0.002, 0.003), flat_key_tooth)


# ---------------------------------------------------------- wall clocks ---
def clock_case():
    """Wall clock case along +Y (the front): tapered back, rolled bezel
    whose lip laps over the edge of the dial."""
    prof = [(0, -0.015), (0.128, -0.015), (0.136, -0.0138), (0.144, -0.007), (0.1488, 0.002), (0.15, 0.009),
            (0.1497, 0.0135), (0.1485, 0.0165), (0.1462, 0.018), (0.1438, 0.0176), (0.1428, 0.0158),
            (0.1422, 0.012), (0.139, 0.012), (0, 0.012)]
    return lathe(prof, seg=36)

def clock_hand(w, L, hub, spade):
    """Clock hand pivoting at -Y: round hub boss, thin blade (a spade for
    the hour hand, a slim taper for the minute hand)."""
    hx = w / 2
    if spade:
        out = [(-0.0012, -L / 2), (0.0012, -L / 2), (0.0012, 0.008), (hx, 0.016), (hx * 0.85, 0.023), (0, L / 2),
               (-hx * 0.85, 0.023), (-hx, 0.016), (-0.0012, 0.008)]
    else:
        out = [(-0.001, -L / 2), (0.001, -L / 2), (hx, L / 2 - 0.02), (0, L / 2), (-hx, L / 2 - 0.02)]
    blade = prism(out, 0.0012)
    boss = cyl(hub, hub, 0.004, seg=12, bevel_r=0.0006).rot(x=math.pi / 2).move(y=-L / 2)
    return blade + boss

part('AnalogClocks__rim', f_cyl(0.15, 0.15, 0.030), clock_case)
part('AnalogClocks__hourHand', f_box(0.008, 0.070, 0.004), lambda: clock_hand(0.008, 0.070, 0.004, True))
part('AnalogClocks__minuteHand', f_box(0.005, 0.100, 0.004), lambda: clock_hand(0.005, 0.100, 0.0026, False))


# ---------------------------------------------------------- photo frame ---
def photo_frame():
    """Moulded picture frame: rounded outer edge, crest, sloping sight edge
    lapping 0.5 mm over the photo, thin backing board."""
    prof = [(0.0, -0.007), (0.0, 0.004), (0.0006, 0.0062), (0.0018, 0.0079), (0.0038, 0.0094), (0.0065, 0.0102),
            (0.0088, 0.0098), (0.0105, 0.0091), (0.0122, 0.0093), (0.0138, 0.0099), (0.0155, 0.0099),
            (0.0158, 0.0092), (0.0158, -0.004), (0.0145, -0.007)]
    back = box(0.13, 0.17, 0.002).move(z=-0.004)
    return moulding(0.16, 0.20, prof) + back

part('FramedPhotos__frame', f_box(0.16, 0.20, 0.014), photo_frame)


# ------------------------------------------------------------- diplomas ---
def diploma_frame():
    """Ornate gold frame: stepped outer edge, bead, cove, ogee crest and a
    rosette at each corner. The inner moulding sits in front of its opening."""
    prof = [(0.0, -0.01), (0.0, 0.002), (0.0008, 0.0048), (0.0022, 0.006), (0.0032, 0.0082), (0.0045, 0.0089),
            (0.0058, 0.0079), (0.0066, 0.0064), (0.0078, 0.0068), (0.0092, 0.0092), (0.0108, 0.0116),
            (0.0124, 0.0124), (0.0138, 0.0112), (0.0148, 0.0088), (0.0158, 0.0078), (0.0158, -0.01)]
    s = moulding(0.26, 0.33, prof)
    for sx in (-1, 1):
        for sy in (-1, 1):
            ros = lathe([(0, 0), (0.0062, 0), (0.0056, 0.0014), (0.0034, 0.0026), (0.0014, 0.0029), (0, 0.0038)], seg=10)
            s = s + ros.rot(x=math.pi / 2).move(x=sx * (0.13 - 0.0098), y=sy * (0.165 - 0.0098), z=0.0104)
    return s

def diploma_inner():
    """Inner (darker) moulding: rounded slip whose lip laps over the paper."""
    prof = [(0.0, -0.005), (0.0, 0.0012), (0.0012, 0.003), (0.0028, 0.0042), (0.0048, 0.0042), (0.0068, 0.0036),
            (0.0088, 0.0052), (0.0102, 0.0066), (0.0107, 0.0062), (0.0107, -0.005)]
    return moulding(0.23, 0.30, prof)

part('Diplomas__frame', f_box(0.26, 0.33, 0.020), diploma_frame)
part('Diplomas__moulding', f_box(0.23, 0.30, 0.010), diploma_inner)


# ------------------------------------------------------------------ dice --
# Pips are colour on real dice (a second material); the placer's decal keeps them.
part('DiceClusters__die', f_box(0.020, 0.020, 0.020), lambda: rbox(0.020, 0.020, 0.020, r=0.0026, seg=3))


# ----------------------------------------------------------- poker chips --
def poker_chip():
    """Clay chip: recessed inlay on both faces, raised rim, rounded edge."""
    return lathe([(0, -0.0012), (0.0098, -0.0012), (0.0104, -0.0015), (0.014, -0.0015), (0.015, -0.0006),
                  (0.015, 0.0006), (0.014, 0.0015), (0.0104, 0.0015), (0.0098, 0.0012), (0, 0.0012)], seg=16)

def chip_edge_spots():
    """The contrast ring as eight edge-spot inserts round the rim (XY plane)."""
    sec = [(0.0147, -0.0006), (0.0156, -0.0006), (0.0156, 0.0006), (0.0147, 0.0006)]
    out = Shape()
    arc = math.radians(22)
    for k in range(8):
        out = out + ring_section(sec, seg=2, arc=arc).rot(y=-arc / 2 + k * math.pi / 4)
    return out.rot(x=math.pi / 2)

part('PokerChips__chip', f_cyl(0.015, 0.015, 0.003), poker_chip)
part('PokerChips__stripe', f_torus(0.015, 0.0006), chip_edge_spots)


# -------------------------------------------------------- cassette reel ---
part('Cassettes__reel', f_cyl(0.017, 0.017, 0.001),
     lambda: ring_section([(0.0035, -0.0005), (0.0168, -0.0005), (0.017, 0.0), (0.0168, 0.0005), (0.0082, 0.0005),
                           (0.0078, 0.0001), (0.0072, 0.0001), (0.0068, 0.0005), (0.0035, 0.0005)], seg=14))


# ------------------------------------------------------------- floppies ---
def floppy_shell():
    """3.5-inch floppy: square shell with the notched corner, eased edges
    and the shallow channel the metal shutter slides in (shutter at -Z)."""
    out = round_outline([(-0.0425, -0.044), (0.0372, -0.044), (0.0425, -0.0387), (0.0425, 0.044), (-0.0425, 0.044)],
                        0.0016, seg=2)
    s = xz_prism(out, 0.005, bevel_r=0.0005, seg=1)
    return cut(s, box(0.058, 0.001, 0.0325).move(x=-0.0055, y=0.0025, z=-0.029))

def floppy_shutter():
    """Sliding metal shutter: thin plate, pressed rim round the head window,
    folded lip over the disk's edge."""
    plate = rbox(0.045, 0.0008, 0.0292, r=0.0003, seg=1).move(y=0.0004, z=0.0004)
    rim = moulding(0.019, 0.012, [(0.0, 0.0), (0.0, 0.0004), (0.0012, 0.0004), (0.0012, 0.0)]).rot(x=-math.pi / 2).move(x=0.015, y=0.0008)
    lip = rbox(0.045, 0.002, 0.0008, r=0.0003, seg=1).move(z=-0.0146)
    return plate + rim + lip

part('FloppyDisks__shell', f_box(0.085, 0.005, 0.088), floppy_shell)
part('FloppyDisks__shutter', f_box(0.045, 0.002, 0.030), floppy_shutter)


# ------------------------------------------------------------------ CDs ---
part('CDs__disc', f_cyl(0.060, 0.060, 0.001),
     lambda: ring_section([(0.0075, -0.0004), (0.0175, -0.0005), (0.0592, -0.0005), (0.06, 0.0), (0.0592, 0.0005),
                           (0.0175, 0.0005), (0.0075, 0.0004)], seg=38))
part('CDs__hubRing', f_torus(0.018, 0.001),
     lambda: ring_section([(0.0171, -0.0009), (0.0189, -0.0009), (0.0189, 0.0009), (0.0171, 0.0009)], seg=32).rot(x=math.pi / 2))


# ------------------------------------------------------------ gadgets -----
def rounded_slab(W, T, L, r, rings):
    """Loft of rounded-rect outlines (XZ) stacked in Y: rings = [(y, inset)]."""
    return loft([[(x, y, z) for x, z in rounded_rect(W - 2 * i, L - 2 * i, max(r - i, 0.0005), seg=2)] for y, i in rings])

def pillow_key(w, h, d, top=0.8):
    """Rubber key: straight sides, softly rounded top."""
    def ring(y, k):
        return [(x, y, z) for x, z in rounded_rect(w * k, d * k, min(w, d) * k * 0.2, seg=1)]
    return loft([ring(-h / 2, 1.0), ring(h / 2 - h * 0.35, 1.0), ring(h / 2, top)])

def bezel(w, d, wall, h):
    """A thin raised frame round a screen, lying in XZ, base at y = 0."""
    return moulding(w, d, [(0.0, 0.0), (0.0, h * 0.6), (wall * 0.4, h), (wall, h), (wall, 0.0)]).rot(x=-math.pi / 2)

def brick_phone():
    """90s mobile: rounded body, raised bezel round the LCD, earpiece slot, mic hole."""
    W, T, L = 0.045, 0.014, 0.100
    s = rounded_slab(W, T, L, 0.008, [(-T / 2, 0.0022), (-T / 2 + 0.0022, 0.0), (T / 2 - 0.0025, 0.0),
                                      (T / 2 - 0.0008, 0.0005), (T / 2, 0.0016)])
    s = cut(s, rbox(0.012, 0.004, 0.0014, r=0.0006, seg=1).move(y=T / 2, z=-0.0455),
            cyl(0.0009, 0.0009, 0.004, seg=6, bevel_r=0).move(y=T / 2, z=0.046))
    return s + bezel(0.0365, 0.0285, 0.0022, 0.0013).move(y=T / 2 - 0.0002, z=-0.028)

def stub_antenna():
    return lathe([(0, -0.01), (0.002, -0.01), (0.002, -0.0065), (0.0015, -0.006), (0.0013, 0.006),
                  (0.0017, 0.0082), (0.001, 0.0098), (0, 0.01)], seg=8)

part('CellPhones__body', f_box(0.045, 0.014, 0.100), brick_phone)
part('CellPhones__key', f_box(0.010, 0.002, 0.010), lambda: pillow_key(0.010, 0.002, 0.010))
part('CellPhones__antenna', f_cyl(0.002, 0.002, 0.020), stub_antenna)


def remote_body():
    """TV remote: flat underside, top that rounds off at both ends, rounded
    sides, and the seam between the two shell halves."""
    W, T, L = 0.045, 0.014, 0.18
    secs = [(-0.09, 0.034, 0.0075), (-0.0875, 0.041, 0.011), (-0.084, 0.0445, 0.0135), (-0.078, 0.045, 0.014),
            (0.078, 0.045, 0.014), (0.084, 0.0445, 0.0132), (0.0875, 0.041, 0.0105), (0.09, 0.033, 0.007)]
    rings = []
    for z, w, t in secs:
        y0 = -T / 2
        pts = [(-w / 2 + 0.002, y0), (w / 2 - 0.002, y0), (w / 2, y0 + 0.0022), (w / 2, y0 + t * 0.62),
               (w / 2 - 0.004, y0 + t * 0.94), (w / 2 - 0.009, y0 + t), (-w / 2 + 0.009, y0 + t),
               (-w / 2 + 0.004, y0 + t * 0.94), (-w / 2, y0 + t * 0.62), (-w / 2, y0 + 0.0022)]
        rings.append([(x, y, z) for x, y in pts])
    return loft(rings)

def rocker():
    """Volume / channel rocker: a pill with a bump at each end."""
    rings = []
    for z, w, h in [(-0.01, 0.0025, 0.0022), (-0.0085, 0.0062, 0.0028), (-0.006, 0.0078, 0.003), (-0.003, 0.008, 0.0028),
                    (0.0, 0.008, 0.0024), (0.003, 0.008, 0.0028), (0.006, 0.0078, 0.003), (0.0085, 0.0062, 0.0028),
                    (0.01, 0.0025, 0.0022)]:
        pts = [(-w / 2, 0), (w / 2, 0), (w / 2, h * 0.75), (0, h), (-w / 2, h * 0.75)]
        rings.append([(x, y - 0.0015, z) for x, y in pts])
    return loft(rings)

part('Remotes__body', f_box(0.045, 0.014, 0.18), remote_body)
part('Remotes__power', f_cyl(0.006, 0.006, 0.003),
     lambda: lathe([(0, -0.0015), (0.006, -0.0015), (0.006, 0.0003), (0.0052, 0.0012), (0.0032, 0.0015), (0, 0.0015)], seg=10))
part('Remotes__key', f_box(0.008, 0.002, 0.008), lambda: pillow_key(0.008, 0.002, 0.008))
part('Remotes__rocker', f_box(0.008, 0.003, 0.020), rocker)
part('Remotes__menu', f_box(0.014, 0.002, 0.008),
     lambda: bevel(xz_prism(rounded_rect(0.014, 0.008, 0.0038, seg=2), 0.002), 0.0005, seg=1, angle=50))


def pager_body():
    """Chunky pager: rounded body, raised bezel round the LCD, grip grooves
    along both sides."""
    W, T, L = 0.048, 0.020, 0.070
    s = rounded_slab(W, T, L, 0.007, [(-T / 2, 0.0025), (-T / 2 + 0.0025, 0.0), (T / 2 - 0.004, 0.0),
                                      (T / 2 - 0.0012, 0.0008), (T / 2, 0.0026)])
    cuts = [box(0.002, 0.0009, 0.034).move(x=sx * W / 2, y=-0.004 + k * 0.003, z=0.008) for sx in (-1, 1) for k in range(3)]
    s = cut(s, *cuts)
    return s + bezel(0.0395, 0.0235, 0.0021, 0.0013).move(y=T / 2 - 0.0002, z=-0.020)

part('Pagers__body', f_box(0.048, 0.020, 0.070), pager_body)
part('Pagers__button', f_cyl(0.004, 0.004, 0.002),
     lambda: lathe([(0, -0.001), (0.004, -0.001), (0.004, 0.0002), (0.0032, 0.0009), (0, 0.001)], seg=8))


def calculator_body():
    """Pocket calculator: rounded slab, raised bezel round display and
    solar cell, a shallow keypad well."""
    W, T, L = 0.055, 0.010, 0.085
    s = rounded_slab(W, T, L, 0.006, [(-T / 2, 0.0018), (-T / 2 + 0.0018, 0.0), (T / 2 - 0.0018, 0.0),
                                      (T / 2 - 0.0006, 0.0005), (T / 2, 0.0014)])
    s = cut(s, rbox(0.047, 0.001, 0.0485, r=0.002, seg=1).move(y=T / 2, z=0.011))
    return s + bezel(0.0465, 0.0265, 0.0022, 0.0012).move(y=T / 2 - 0.0002, z=-0.0305)

part('Calculators__body', f_box(0.055, 0.010, 0.085), calculator_body)
part('Calculators__key', f_box(0.009, 0.002, 0.009), lambda: pillow_key(0.009, 0.002, 0.009))


# ---------------------------------------------------------- thermometer ---
def therm_board():
    """Wooden backboard: arched top with a hanging hole, rounded foot,
    routed (rounded) front edges."""
    hw, hh = 0.015, 0.16
    out = [(-hw, -hh + 0.004), (-hw + 0.004, -hh), (hw - 0.004, -hh), (hw, -hh + 0.004), (hw, hh - hw)]
    for i in range(1, 8):
        a = math.pi * i / 8
        out.append((hw * math.cos(a), hh - hw + hw * math.sin(a)))
    out.append((-hw, hh - hw))
    s = bevel(prism(out, 0.008), 0.0018, seg=2, angle=50)
    return cut(s, cyl(0.0022, 0.0022, 0.02, seg=8, bevel_r=0).rot(x=math.pi / 2).move(y=0.147))

part('Thermometers__board', f_box(0.030, 0.32, 0.008), therm_board)
part('Thermometers__bulb', f_sphere(0.010),
     lambda: lathe([(0, -0.01), (0.005, -0.0087), (0.0087, -0.005), (0.01, 0.0), (0.0088, 0.0042), (0.0055, 0.0074),
                    (0.0026, 0.0091), (0.0022, 0.01), (0, 0.01)], seg=12))


# --------------------------------------------------------------- 8-track --
def eight_track():
    """8-track cartridge: square front with the tape opening (the dark
    insert sits in it), big round back corners, recesses the two hubs sit in."""
    W, T, L = 0.108, 0.024, 0.140
    hx, hz = W / 2, L / 2
    back = []
    for sx in (-1, 1):
        c = (sx * (hx - 0.02), -hz + 0.02)
        back.append([(c[0] + sx * 0.02 * math.sin(math.pi / 2 * i / 4), c[1] - 0.02 * math.cos(math.pi / 2 * i / 4)) for i in range(5)])
    out = list(reversed(back[0])) + back[1] + [(hx, hz - 0.004), (hx - 0.004, hz), (-hx + 0.004, hz), (-hx, hz - 0.004)]
    s = xz_prism(out, T, bevel_r=0.002, seg=2)
    s = cut(s, box(0.072, 0.021, 0.018).move(z=hz),
            *[cyl(0.0086, 0.0086, 0.005, seg=14, bevel_r=0).move(x=sx * 0.024, y=T / 2, z=0.058) for sx in (-1, 1)])
    return s

part('EightTrackTapes__body', f_box(0.108, 0.024, 0.140), eight_track)
part('EightTrackTapes__hub', f_cyl(0.008, 0.008, 0.002),
     lambda: ring_section([(0.003, -0.001), (0.008, -0.001), (0.008, 0.001), (0.003, 0.001)], seg=12))


# ------------------------------------------------- reading glasses + book --
def page_stack():
    """Half of an open book's page block: gently crowned, page layers
    grooved round the edges (symmetric: it serves both halves)."""
    s = slab(0.10, 0.14, 0.006, 4, 4).rot(x=-math.pi / 2)
    s = s.warp(lambda p: (p.x, p.y + (0.0006 * (1 - (p.x / 0.05) ** 2) * (1 - (p.z / 0.07) ** 4) if p.y > 0 else 0.0), p.z))
    ring = lambda y: cut(xz_prism(rounded_rect(0.104, 0.144, 0.001, seg=1), 0.0006), xz_prism(rounded_rect(0.0994, 0.1394, 0.001, seg=1), 0.002)).move(y=y)
    return cut(s, ring(-0.0012), ring(0.0012))

def temple_arm():
    """Folded spectacle arm along Y: hinge block at +Y, slim shaft, ear tip."""
    rings = []
    for y, w, d in [(0.024, 0.0019, 0.0019), (0.0205, 0.0019, 0.0019), (0.019, 0.0011, 0.0016), (-0.016, 0.0009, 0.0013),
                    (-0.0205, 0.0013, 0.0017), (-0.0235, 0.0009, 0.0012), (-0.024, 0.0005, 0.0007)]:
        rings.append([(x, y, z) for x, z in rounded_rect(w, d, min(w, d) * 0.45, seg=1)])
    return loft(rings)

def bookmark():
    """Card bookmark with a V-notched tail, arched a little."""
    s = slab(0.012, 0.05, 0.0005, 2, 8).rot(x=-math.pi / 2)
    s = s.warp(lambda p: (p.x, p.y + 0.0026 * math.sin(math.pi * (p.z + 0.025) / 0.05),
                          p.z + (0.004 if abs(p.x) < 1e-5 and p.z < -0.0249 else 0.0)))
    return s.fit((-0.006, -0.0015, -0.025), (0.006, 0.0015, 0.025))

part('ReadingGlassesAndBooks__pages', f_box(0.10, 0.006, 0.14), page_stack)
part('ReadingGlassesAndBooks__spine', f_cyl(0.003, 0.003, 0.14), lambda: cyl(0.003, 0.003, 0.14, seg=10, bevel_r=0.0012))
part('ReadingGlassesAndBooks__cover', f_box(0.003, 0.008, 0.14), lambda: rbox(0.003, 0.008, 0.14, r=0.0009, seg=2))
part('ReadingGlassesAndBooks__rim', f_torus(0.020, 0.002),
     lambda: ring_section([(0.0183, -0.0013), (0.021, -0.0015), (0.022, 0.0), (0.021, 0.0015), (0.0183, 0.0013)], seg=26).rot(x=math.pi / 2))
part('ReadingGlassesAndBooks__bridge', f_box(0.014, 0.002, 0.003),
     lambda: tube([(-0.007, 0, 0.0006), (-0.0035, 0, -0.0006), (0, 0, -0.0009), (0.0035, 0, -0.0006), (0.007, 0, 0.0006)],
                  0.0009, seg=6).fit((-0.007, -0.001, -0.0015), (0.007, 0.001, 0.0015)))
part('ReadingGlassesAndBooks__temple', f_cyl(0.001, 0.001, 0.048), temple_arm)
part('ReadingGlassesAndBooks__bookmark', f_box(0.012, 0.003, 0.05), bookmark)
