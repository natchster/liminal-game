"""t03: the washroom shelf and the dressing table's leavings (Props P3,
docs/design/PROPS.md): a glass shelf on its clamps with the bottles left on it, a
shower cap dropped on the floor, a powder compact, two kinds of perfume
bottle and a makeup sponge. (The nail polish is batch r18's.) Sources and
sizes: docs/research/props-remodelling/washroom_shelf_and_dressing_table.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is modelled at
its real size in its prop's own frame: x across its front, y up from the
floor (y = 0 what it stands on), z out of its front. Every part is named
`Real__<prop><Part>`:

shampooShelf*    a 500 x 120 mm shelf of 8 mm toughened glass (Glass, its
                 green polished edge GlassEdge) in two chromed clamp
                 Brackets; the wall at z = 0, the glass's top at y = 0
shampooOval*     a 400 ml oval HDPE bottle of shampoo standing (Body), its
                 printed front (Label), its flip-top Cap
shampooRound*    a 250 ml Boston round PET bottle (Body), its wrap-round
                 Label, its disc-top Cap
showerCap*       a PEVA shower cap dropped on the floor, opening down: the
                 crown fallen in on itself in creases, spilled over its
                 elastic on one side, the gathered frill round the elastic
                 flopped out on the floor in soft waves (Film)
makeupCompact*   a pressed powder compact, 76 x 15 mm: the Base (its well,
                 hinge knuckles, catch), the Lid shut (its knuckle and thumb
                 tab; the game turns it open about HINGE), its gilt Trim
                 ring, the Mirror in the lid, the aluminium Pan and the Powder pressed in it, worn
                 into a hollow
perfumeFlacon*   a 30 ml rectangular flacon (42 x 25 x 78 mm): Glass, its
                 thick GlassBase, the Scent left in it (ScentLow, ScentMid,
                 ScentHigh: a fifth, a half, three quarters of the inside),
                 the DipTube, the crimped Pump, the square Cap over it
perfumeAtomizer* a pressed-glass atomizer bottle (48 x 25 mm, fluted):
                 Glass, GlassBase, ScentLow/Mid/High; its head: the gilt Collar, the
                 netted rubber Bulb lying on the floor, the silk-covered Hose
                 to it (Hose on this bottle, HoseFlacon from the flacon's
                 higher neck), the Tassel
makeupSponge*    a teardrop blending sponge (45 x 65 mm) standing on its
                 base (Foam; the game lays it on its side: SPONGE_LAY)
"""
from propkit import *
import propkit

# ------------------------------------------------------------------ util --
# the parts whose texture coordinates go round them (a cylinder's)
CYL_FRAMED = {'Real__shampooRoundLabel', 'Real__makeupSpongeFoam'}


def auto(name, build, budget, smooth=35.0, flat=False):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    kind = 'cyl' if name in CYL_FRAMED else 'box'
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, smooth=smooth, flat=flat, budget=budget)


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def face_toward(s, want):
    """Turn each face of an open surface to face `want(centroid)`."""
    f2 = []
    for f in s.f:
        a, b, c = s.v[f[0]], s.v[f[1]], s.v[f[2]]
        n = (b - a).cross(c - a)
        if len(f) > 3:
            n = n + (c - a).cross(s.v[f[3]] - a)
        cen = sum((s.v[i] for i in f), Vector()) / len(f)
        f2.append(f if n.dot(want(cen)) >= 0 else tuple(reversed(f)))
    s.f = f2
    return s


def grid_surface(fn, nu, nv, wrap_u=False):
    """An open grid of quads from fn(u, v) -> (x, y, z), u and v in [0, 1]."""
    cols = nu if wrap_u else nu + 1
    v, f = [], []
    for j in range(nv + 1):
        for i in range(cols):
            v.append(Vector(fn(i / nu, j / nv)))
    for j in range(nv):
        for i in range(nu):
            a = j * cols + i
            b = j * cols + (i + 1) % cols if wrap_u else a + 1
            f.append((a, b, b + cols, a + cols))
    return Shape(v, f)


def sell(a, b, n=2.0):
    """A superellipse a across (x), b deep (z), its squareness n: t -> (x, z)."""
    def f(t):
        c, s = math.cos(t), math.sin(t)
        return (a * math.copysign(abs(c) ** (2.0 / n), c), b * math.copysign(abs(s) ** (2.0 / n), s))
    return f


def loft(rings, seg=32, close_bottom=None, close_top=None):
    """Rings [(y, f(t) -> (x, z))] from the bottom up, joined in quads; a
    ring closed by a fan to a centre at the given height."""
    v, f = [], []
    for y, fn in rings:
        for j in range(seg):
            x, z = fn(TAU * j / seg)
            v.append(Vector((x, y, z)))
    n = len(rings)
    for i in range(n - 1):
        for j in range(seg):
            a, b = i * seg + j, i * seg + (j + 1) % seg
            f.append((a, b, b + seg, a + seg))
    s = Shape(v, f)
    if close_bottom is not None:
        s.v.append(Vector((0, close_bottom, 0)))
        c = len(s.v) - 1
        for j in range(seg):
            s.f.append((c, (j + 1) % seg, j))
    if close_top is not None:
        s.v.append(Vector((0, close_top, 0)))
        c = len(s.v) - 1
        top = (n - 1) * seg
        for j in range(seg):
            s.f.append((c, top + j, top + (j + 1) % seg))
    return propkit._fix_winding(s)


def plate_xz(outline, t, y0=0.0, bevel_r=0.0, bevel_seg=1):
    """A plate of outline [(x, z)] in plan, t thick, its bottom at y0."""
    s = prism([(x, -z) for x, z in outline], t, bevel_r=bevel_r, bevel_seg=bevel_seg)
    return s.rot(x=-math.pi / 2).move(0, y0 + t / 2, 0)


def side_profile(outline, w, bevel_r=0.0):
    """A body of side profile [(z, y)] extruded w across x, centred on x = 0."""
    s = prism([(-z, y) for z, y in outline], w, bevel_r=bevel_r)
    return s.rot(y=math.pi / 2)


def drop_caps(s, n_caps=2):
    """A prism's side walls alone (its first two faces are its caps)."""
    return Shape(s.v, s.f[n_caps:])


# ============================================================ the shelf ==
# A bathroom shelf of 8 mm toughened glass, 500 x 120 mm, its front corners
# rounded and its edges polished, held at its back edge by two chrome-plated
# zinc clamp brackets (each a wall plate, the jaw the glass sits in, a grub
# screw under it). The wall at z = 0, the glass's top at y = 0.
SH_W, SH_D, SH_T = 0.500, 0.120, 0.008
SH_Z0 = 0.010           # the glass's back edge, off the wall by the clamps' plates
SH_BRACKET_X = 0.180


def shelf_outline(grow=0.0):
    w = SH_W / 2 + grow
    z0, z1 = SH_Z0 - grow, SH_Z0 + SH_D + grow
    pts = [(-w, z0), (w, z0), (w, z1), (-w, z1)]
    out = []
    for i, p in enumerate(pts):
        r = 0.012 + grow if i >= 2 else 0.002     # the front corners rounded
        out += propkit._round_corner(p, pts[i - 1], pts[(i + 1) % 4], r, 5)
    return out


def shelf_glass():
    return plate_xz(shelf_outline(), SH_T, -SH_T, bevel_r=0.0008)


def shelf_glass_edge():
    # the polished edge: green as float glass's edge is, a skin over its sides
    s = drop_caps(prism([(x, -z) for x, z in shelf_outline(0.00015)], SH_T - 0.0018))
    return s.rot(x=-math.pi / 2).move(0, -SH_T / 2, 0)


def shelf_bracket():
    # side profile (z out of the wall, y up): the wall plate, the jaw under
    # the glass, the lip over it
    prof = [(0.0, -0.030), (0.009, -0.030), (0.017, -0.0170), (0.051, -0.0138), (0.0555, -0.0108),
            (0.052, -0.0081), (SH_Z0 + 0.0004, -0.0081), (SH_Z0 + 0.0004, 0.0001), (0.029, 0.0001),
            (0.0312, 0.0026), (0.029, 0.0052), (0.0125, 0.0062), (0.009, 0.014), (0.0, 0.014)]
    prof = round_outline(prof, 0.0012, seg=2)
    body = side_profile(prof, 0.024, bevel_r=0.0011)
    # the grub screw's socket head under the jaw, a rubber pad's edge in the slot
    grub = cyl(0.0028, h=0.0014, seg=10, bevel_r=0.0003).move(0, -0.0150, 0.036)
    sock = cyl(0.0012, h=0.003, seg=6, bevel_r=0).move(0, -0.0158, 0.036)
    grub = cut(grub, sock)
    # the wall plate's screw cover cap
    cover = cyl(0.0042, h=0.0012, seg=12, bevel_r=0.0004).rot(x=math.pi / 2).move(0, -0.022, 0.0095)
    return body + grub + cover


def shelf_brackets():
    one = shelf_bracket()
    return one.copy().move(-SH_BRACKET_X, 0, 0) + one.copy().move(SH_BRACKET_X, 0, 0)


auto('Real__shampooShelfGlass', shelf_glass, 200)
auto('Real__shampooShelfGlassEdge', shelf_glass_edge, 200)
auto('Real__shampooShelfBrackets', shelf_brackets, 1500)


# ===================================================== the oval bottle ==
# A 400 ml oval HDPE shampoo bottle of the common "orchid oval" kind (76 x
# 45 mm, 201 mm to the top of its 24 mm neck), a flip-top cap on it.
OV_A, OV_B, OV_N = 0.038, 0.0225, 2.4
OV_NECK = (0.0122, 0.183, 0.201)      # the neck's radius, foot and top


def oval_rings(grow=0.0):
    a, b, n = OV_A + grow, OV_B + grow, OV_N
    r, y0, y1 = OV_NECK
    r += grow
    prof = [  # (y, scale of the oval, how far toward the neck's circle)
        (0.0, 0.90, 0.0), (0.0015, 0.965, 0.0), (0.0045, 0.993, 0.0), (0.010, 1.0, 0.0),
        (0.060, 0.985, 0.0), (0.100, 0.975, 0.0), (0.140, 0.99, 0.0), (0.152, 0.985, 0.02),
        (0.162, 0.94, 0.08), (0.170, 0.84, 0.20), (0.176, 0.70, 0.40), (0.1795, 0.55, 0.70),
        (0.182, 0.0, 1.0)]
    rings = []
    for y, k, m in prof:
        aa = (a * k) * (1 - m) + r * m
        bb = (b * k) * (1 - m) + r * m
        nn = n * (1 - m) + 2.0 * m
        rings.append((y, sell(aa, bb, nn)))
    return rings


def oval_body():
    r, y0, y1 = OV_NECK
    body = loft(oval_rings(), seg=40, close_bottom=0.0022)
    neck = lathe([(0.0, y0 - 0.003), (r, y0 - 0.003), (r, y0 + 0.004), (r + 0.0012, y0 + 0.005), (r + 0.0012, y0 + 0.0065),
                  (r, y0 + 0.0075), (r, y1 - 0.0005), (r - 0.0005, y1), (0.0, y1)], seg=20)
    return body + neck


def oval_label():
    # the front's print, a skin over the bottle from 30 to 140 mm, as far
    # round as it faces more forward than sideways
    ys = [0.030 + 0.110 * j / 10 for j in range(11)]
    prof = oval_rings(0.00022)

    def ring_at(y):
        for (ya, fa), (yb, fb) in zip(prof, prof[1:]):
            if ya <= y <= yb:
                return fa, fb, (y - ya) / (yb - ya)
        return prof[-1][1], prof[-1][1], 0.0
    half = 0.78

    def fn(u, v):
        y = ys[0] + (ys[-1] - ys[0]) * v
        fa, fb, k = ring_at(y)
        t = math.pi / 2 + half * (1 - 2 * u)
        xa, za = fa(t)
        xb, zb = fb(t)
        return (xa + (xb - xa) * k, y, za + (zb - za) * k)
    s = grid_surface(fn, 16, 10)
    return face_toward(s, lambda c: Vector((c.x, 0, c.z)))


def flip_cap(r, y0, h):
    """A flip-top cap r across, from y0, h tall: the lid's line, the hinge
    strap at the back, the thumb recess at the front, the top a shade
    domed with the orifice's ring on it."""
    y1 = y0 + h
    split = y0 + h * 0.62
    body = lathe([(0.0, y0), (r - 0.0006, y0), (r, y0 + 0.0006), (r, split - 0.0005), (r - 0.0005, split - 0.0002),
                  (r - 0.0005, split + 0.0002), (r, split + 0.0005), (r, y1 - 0.0016), (r - 0.0012, y1 - 0.0002),
                  (r * 0.5, y1 + 0.0006), (0.0, y1 + 0.0007)], seg=32)
    notch = rbox(0.011, 0.0042, 0.006, r=0.0018, seg=2).move(0, split + 0.0004, r + 0.0012)
    body = cut(body, notch)
    hinge = rbox(0.010, 0.006, 0.0022, r=0.0007, seg=1).move(0, split, -r - 0.0004)
    ring = torus(0.0032, 0.0005, seg=16, tseg=4).rot(x=math.pi / 2).move(0, y1 + 0.0007, r * 0.25)
    return body + hinge + ring


def oval_cap():
    r, y0, y1 = OV_NECK
    return flip_cap(0.0150, y0 - 0.0005, 0.0235)


auto('Real__shampooOvalBody', oval_body, 2600)
auto('Real__shampooOvalLabel', oval_label, 400)
auto('Real__shampooOvalCap', oval_cap, 900)


# ==================================================== the round bottle ==
# A 250 ml Boston round PET bottle (58 mm across, 129 mm to the top of its
# 24/410 neck), a disc-top cap on it, a paper label round it.
BR_R = 0.029
BR_NECK = (0.0122, 0.113, 0.129)


def round_body():
    r, y0, y1 = BR_NECK
    prof = [(0.0, 0.0018), (0.018, 0.0004), (0.0255, 0.0), (0.0280, 0.0012), (0.0289, 0.0040), (BR_R, 0.0080),
            (BR_R, 0.0890), (0.0286, 0.0950), (0.0270, 0.1010), (0.0236, 0.1060), (0.0186, 0.1095), (0.0140, 0.1118),
            (r, y0), (r, y0 + 0.004), (r + 0.0012, y0 + 0.005), (r + 0.0012, y0 + 0.0065), (r, y0 + 0.0075),
            (r, y1 - 0.0005), (r - 0.0005, y1), (0.0, y1)]
    return lathe(prof, seg=40)


def round_label():
    r = BR_R + 0.00022
    return lathe([(r, 0.014), (r, 0.030), (r, 0.050), (r, 0.070), (r, 0.084)], seg=48, cap_bottom=False, cap_top=False)


def disc_cap(r, y0, h):
    """A disc-top cap: its side ribbed, the disc that tips to open set in
    its top, the orifice's nub on the disc."""
    y1 = y0 + h
    body = lathe([(0.0, y0), (r - 0.0006, y0), (r, y0 + 0.0006), (r, y1 - 0.0012), (r - 0.0010, y1), (r * 0.80, y1),
                  (r * 0.78, y1 - 0.0008), (0.0, y1 - 0.0008)], seg=48)
    # the grip ribs round its side
    ribs = Shape()
    for i in range(30):
        a = TAU * i / 30
        rib = box(0.0007, h * 0.70, 0.0006).move(0, y0 + h * 0.45, r + 0.0001).rot(y=a)
        ribs = ribs + rib
    disc = cyl(r * 0.76, h=0.0010, seg=32, bevel_r=0.0003).rot(x=0.02).move(0, y1 - 0.0006, 0)
    nub = rbox(0.0040, 0.0016, 0.0030, r=0.0007, seg=1).move(0, y1 + 0.0004, r * 0.52)
    return body + ribs + disc + nub


def round_cap():
    r, y0, y1 = BR_NECK
    return disc_cap(0.0136, y0 - 0.0006, 0.0185)


auto('Real__shampooRoundBody', round_body, 1800)
auto('Real__shampooRoundLabel', round_label, 400)
auto('Real__shampooRoundCap', round_cap, 1600)


# ======================================================== the shower cap ==
# A reusable PEVA shower cap (26 cm across laid flat, its elastic 41 cm
# round before it is stretched) dropped on the floor, opening down: the
# elastic gathered to a ring 13 cm across under the crown, which has fallen
# in on itself in soft folds and spilled out over it in places; the frill
# round the elastic flopped out on the floor in tight pleats.
SC_RING = 0.064         # the elastic's radius, gathered
SC_FRILL = 0.021        # how far the frill reaches out from it
SC_H = 0.030            # the crown's height at its highest


def _sc_noise(x, z, f, seed):
    return noise.noise(Vector((x * f + seed * 3.1, z * f - seed * 1.7, seed * 0.37)))


def _ridge(x, z, f, seed):
    """Sharp creases: 1 along the noise's zero lines, falling away from them."""
    return (1.0 - abs(_sc_noise(x, z, f, seed))) ** 4


def cap_out(th):
    """How far the crown reaches from the centre at angle th: just over the
    frill's root, and in a lobe spilled over the frill onto the floor."""
    lobe = math.sin(th + 0.7) * 1.1 + 0.45 * noise.noise(Vector((math.cos(th) * 1.6, math.sin(th) * 1.6, 4.2))) - 0.45
    return SC_RING + 0.005 + 0.040 * max(0.0, min(1.0, lobe)) ** 1.3


def cap_crown_point(rho, th):
    ro = cap_out(th)
    r = rho * ro
    x, z = r * math.cos(th), r * math.sin(th)
    # a low soft dome, off centre
    cx, cz = 0.008, -0.006
    d = math.hypot(x - cx, z - cz) / (ro + 0.016)
    h = SC_H * max(0.0, 1 - d * d) ** 0.85
    # where the edge comes down: on the frill or past it on the floor
    rest = 0.0058 * smoothstep(SC_RING + SC_FRILL + 0.006, SC_RING + SC_FRILL - 0.004, ro) + 0.0006
    k = smoothstep(1.0, 0.78, rho)
    h = rest + (h - rest) * k if h > rest else rest
    # folds running in to the elastic, uneven, sharp along their ridges
    fold = (1.0 - abs(noise.noise(Vector((math.cos(th) * 2.2 + rho * 0.9, math.sin(th) * 2.2 - rho * 0.7, 11.3))))) ** 5
    fold2 = (1.0 - abs(noise.noise(Vector((math.cos(th) * 4.1 - rho * 1.3, math.sin(th) * 4.1 + rho * 1.1, 17.9))))) ** 6
    h += (0.0060 * fold + 0.0035 * fold2 - 0.0012) * smoothstep(0.30, 0.85, rho) * (1 - smoothstep(0.92, 1.0, rho))
    # crumpled: creases every way, two scales
    crum = 0.0038 * (_ridge(x, z, 30, 2) - 0.25) + 0.0016 * (_ridge(x, z, 80, 3) - 0.25)
    h += crum * k
    # the dome given way on one side, and folded over across the other
    h -= 0.012 * math.exp(-(((x + 0.026) / 0.024) ** 2 + ((z - 0.018) / 0.020) ** 2)) * k
    fx, fz = 0.6, -0.8
    h += 0.005 * math.exp(-((((x - 0.02) * fz - (z + 0.03) * fx) / 0.006) ** 2)) * smoothstep(0.1, 0.5, rho) * k
    r2 = r + 0.0022 * _sc_noise(x, z, 30, 4) * rho
    return (r2 * math.cos(th), max(h, rest * 0.85), r2 * math.sin(th))


def cap_frill_point(t, th):
    # t across the frill from the elastic (0) to its hem (1): soft waves
    # round it, uneven, as a gathered film's edge lies
    ph = th * 24 + 1.3 * math.sin(th * 3 + 0.4) + 0.9 * _sc_noise(math.cos(th), math.sin(th), 3.0, 6)
    wave = math.sin(ph)
    amp = 0.6 + 0.5 * _sc_noise(math.cos(th), math.sin(th), 2.2, 7)
    reach = 0.88 + 0.12 * math.sin(th * 5 + 0.3) + 0.08 * _sc_noise(math.cos(th), math.sin(th), 4.0, 5)
    r = SC_RING + t * SC_FRILL * reach + 0.0007 * math.cos(ph) * t
    y = 0.0045 * (1 - t) + 0.0008 + t * 0.0055 * max(0.15, amp) * (0.5 + 0.5 * wave)
    return (r * math.cos(th), max(y, 0.0004), r * math.sin(th))


def _disc_grid(u, v):
    a, b = 2 * u - 1, 2 * v - 1
    x, z = a * math.sqrt(max(0.0, 1 - b * b / 2)), b * math.sqrt(max(0.0, 1 - a * a / 2))
    return min(1.0, math.hypot(x, z)), math.atan2(z, x)


def shower_cap_film():
    crown = grid_surface(lambda u, v: cap_crown_point(*_disc_grid(u, v)), 40, 40)
    crown = face_toward(crown, lambda c: Vector((0, 1, 0)))
    frill = grid_surface(lambda u, v: cap_frill_point(v, u * TAU), 184, 3, wrap_u=True)
    frill = face_toward(frill, lambda c: Vector((0, 1, 0)))
    return crown + frill


auto('Real__showerCapFilm', shower_cap_film, 4400, smooth=60.0)


# ======================================================== the compact ==
# A pressed powder compact for a 59 mm pan (3 in across, 0.6 in thick: 76 x
# 15 mm): a moulded case and its lid on a three-knuckle hinge at the back, a
# catch at the front, the mirror set in the lid, the aluminium pan glued in
# the well. Shut, the lid sits on the base; the game turns it open about
# HINGE (the axis along x at y, z).
CP_R = 0.038
CP_BASE_H = 0.0090
CP_LID_H = 0.0062
CP_PAN_R = 0.0295
CP_WELL = 0.0052
CP_HINGE = (CP_BASE_H, -CP_R - 0.0018)


def compact_base():
    r = CP_R
    prof = [(0.0, 0.0006), (r - 0.004, 0.0), (r - 0.0012, 0.0004), (r, 0.0022), (r, CP_BASE_H - 0.0016),
            (r - 0.0008, CP_BASE_H - 0.0012), (r - 0.0008, CP_BASE_H - 0.0004), (r - 0.0014, CP_BASE_H),
            (CP_PAN_R + 0.0012, CP_BASE_H), (CP_PAN_R + 0.0005, CP_BASE_H - 0.0004),
            (CP_PAN_R + 0.0005, CP_BASE_H - CP_WELL), (0.0, CP_BASE_H - CP_WELL)]
    body = lathe(prof, seg=48)
    hy, hz = CP_HINGE
    knuckles = Shape()
    for x in (-0.0125, 0.0125):
        k = cyl(0.0024, h=0.0090, seg=12, bevel_r=0.0004).rot(z=math.pi / 2).move(x, hy, hz)
        web = rbox(0.0090, 0.0040, 0.0040, r=0.0008, seg=1).move(x, hy - 0.0030, hz + 0.0022)
        knuckles = knuckles + k + web
    pin_ends = Shape()
    for x in (-0.0172, 0.0172):
        pin_ends = pin_ends + cyl(0.0011, h=0.0006, seg=8, bevel_r=0).rot(z=math.pi / 2).move(x, hy, hz)
    # the catch: a push button in a boss on the front
    boss = rbox(0.0120, 0.0050, 0.0030, r=0.0010, seg=2).move(0, CP_BASE_H - 0.0040, r - 0.0002)
    button = rbox(0.0070, 0.0030, 0.0016, r=0.0007, seg=1).move(0, CP_BASE_H - 0.0040, r + 0.0018)
    return body + knuckles + pin_ends + boss + button


def compact_lid():
    r = CP_R
    y0, h = CP_BASE_H, CP_LID_H
    prof = [(0.0, y0 + 0.0012), (CP_PAN_R + 0.0004, y0 + 0.0012), (CP_PAN_R + 0.0004, y0 + 0.0003),
            (r - 0.0009, y0 + 0.0003), (r - 0.0009, y0 - 0.0003), (r - 0.0002, y0 - 0.0003), (r, y0 + 0.0008),
            (r, y0 + h - 0.0022), (r - 0.0012, y0 + h - 0.0006), (r - 0.0050, y0 + h), (0.0, y0 + h + 0.0006)]
    body = lathe(prof, seg=48)
    hy, hz = CP_HINGE
    k = cyl(0.0024, h=0.0150, seg=12, bevel_r=0.0004).rot(z=math.pi / 2).move(0, hy, hz)
    web = rbox(0.0150, 0.0040, 0.0040, r=0.0008, seg=1).move(0, hy + 0.0026, hz + 0.0022)
    tab = rbox(0.0100, 0.0018, 0.0034, r=0.0007, seg=1).move(0, y0 + 0.0012, r + 0.0010)
    return body + k + web + tab


def compact_trim():
    # the lid's trim: a thin ring of gilt metal pressed round its top
    y = CP_BASE_H + CP_LID_H
    prof = [(CP_R - 0.0062, y - 0.0002), (CP_R - 0.0060, y + 0.00035), (CP_R - 0.0045, y + 0.0004),
            (CP_R - 0.0026, y - 0.0004), (CP_R - 0.0020, y - 0.0012)]
    return lathe(prof, seg=48, cap_bottom=False, cap_top=False)


def compact_mirror():
    # the glass set in the lid's recess, its silvered face down (it faces the
    # pan when shut)
    return cyl(CP_PAN_R - 0.0002, h=0.0009, seg=40, bevel_r=0.0002).move(0, CP_BASE_H + 0.0013 - 0.00045 - 0.0001, 0)


def compact_pan():
    r = CP_PAN_R
    y0 = CP_BASE_H - CP_WELL + 0.0001
    return vessel([(r - 0.0006, y0), (r, y0 + 0.0006), (r, y0 + CP_WELL - 0.0005)], 0.00035, seg=48)


def compact_powder():
    r = CP_PAN_R - 0.00045
    y0 = CP_BASE_H - CP_WELL + 0.0005
    top = CP_BASE_H - 0.0011

    def fn(u, v):
        th = u * TAU
        rr = r * v
        x, z = rr * math.cos(th), rr * math.sin(th)
        # the hollow worn in it by the puff, off centre, its slopes soft
        d = math.hypot(x + 0.004, z - 0.003) / 0.018
        dip = 0.0022 * math.exp(-d * d * 1.6)
        lip = 0.0002 * smoothstep(0.92, 1.0, v)
        return (x, top - dip - lip + 0.00008 * noise.noise(Vector((x * 400, z * 400, 1.0))), z)
    topsurf = grid_surface(fn, 48, 10, wrap_u=True)
    topsurf = face_toward(topsurf, lambda c: Vector((0, 1, 0)))
    wall = lathe([(r, y0), (r, top - 0.0002)], seg=48, cap_bottom=False, cap_top=False)
    return weld(topsurf, 1e-7) + wall


auto('Real__makeupCompactBase', compact_base, 2400)
auto('Real__makeupCompactLid', compact_lid, 1600)
auto('Real__makeupCompactTrim', compact_trim, 400)
auto('Real__makeupCompactMirror', compact_mirror, 500)
auto('Real__makeupCompactPan', compact_pan, 600)
auto('Real__makeupCompactPowder', compact_powder, 1100)
print('t03 compact: hinge y %.5f z %.5f' % CP_HINGE)


# ===================================================== the perfumes ==
# The flacon: a 30 ml rectangular bottle of the "Vermeer" kind (a 42 x 25 mm
# base, 78 mm to the top of its 15 mm crimp neck, a heavy bottom), its spray
# pump crimped on, the overcap on it.
FL_W, FL_D, FL_H = 0.042, 0.025, 0.064          # the body to its shoulder
FL_NECK = (0.0075, 0.0705, 0.078)               # radius, foot, top
FL_BASE = 0.008
FL_WALL = 0.0032
# The atomizer: pressed glass of the dressing table's kind (a 48 x 25 mm
# base, fluted front and back), a gilt collar and a rubber bulb in a silk net
# on a covered hose, a tassel.
AT_W, AT_D, AT_H = 0.048, 0.025, 0.060
AT_NECK = (0.0085, 0.0645, 0.072)
AT_BASE = 0.007
AT_WALL = 0.0030


def _sq_body(w, d, y0, y1, r_side, r_end):
    """A rectangular glass body w x d from y0 to y1: its upright edges
    rounded r_side, its top and bottom edges r_end."""
    s = rbox(w, y1 - y0, d, r=r_end, seg=2)
    bm = propkit.to_bmesh(s)
    bmesh.ops.subdivide_edges(bm, edges=[e for e in bm.edges], cuts=2, use_grid_fill=True)
    s = propkit.from_bmesh(bm, keep_sharp=False)
    bm.free()
    lx, lz = w / 2 - r_side, d / 2 - r_side

    def round_edges(p):
        ax, az = abs(p.x), abs(p.z)
        if ax > lx and az > lz:
            dx, dz = ax - lx, az - lz
            dd = math.hypot(dx, dz)
            if dd > r_side:
                k = r_side / dd
                return Vector((math.copysign(lx + dx * k, p.x), p.y, math.copysign(lz + dz * k, p.z)))
        return p
    return s.warp(round_edges).move(0, (y0 + y1) / 2, 0)


def _bottle_glass(w, d, h, neck, base, wall, flutes=0):
    r, y0, y1 = neck
    outer = _sq_body(w, d, 0.0, h, 0.0045, 0.0018)
    if flutes:
        # pressed flutes down its front and back, rounded hollows
        for side in (-1, 1):
            for i in range(flutes):
                x = (i - (flutes - 1) / 2) * (w - 0.010) / (flutes - 1)
                fl = cyl(0.0024, h=h - 0.016, seg=10, bevel_r=0.0012).scale(1.0, 1.0, 0.45).move(x, h / 2 + 0.002, side * (d / 2 + 0.0001))
                outer = cut(outer, fl)
    shoulder = lathe([(0.0, h - 0.002), (min(w, d) / 2 - 0.002, h - 0.002), (min(w, d) / 2 - 0.0035, h + 0.0010),
                      (r + 0.0028, y0 - 0.0008), (r, y0), (0.0, y0)], seg=24)
    neck_s = lathe([(0.0, y0 - 0.001), (r, y0 - 0.001), (r, y1 - 0.0022), (r + 0.0010, y1 - 0.0016), (r + 0.0010, y1 - 0.0004),
                    (r - 0.0004, y1), (0.0, y1)], seg=20)
    s = boolean(outer, shoulder, 'UNION')
    s = boolean(s, neck_s, 'UNION')
    ch = h - base - wall * 0.6
    cav = rbox(w - 2 * wall, ch, d - 2 * wall, r=0.0030, seg=2).move(0, base + ch / 2, 0)
    cav = boolean(cav, cyl(r - 0.0023, h=0.03, seg=12, bevel_r=0).move(0, y1 - 0.004, 0), 'UNION')
    return boolean(s, cav, 'DIFFERENCE')


GLASSES = {}


def _glass(key):
    if key not in GLASSES:
        if key == 'flacon':
            GLASSES[key] = _bottle_glass(FL_W, FL_D, FL_H, FL_NECK, FL_BASE, FL_WALL)
        else:
            GLASSES[key] = _bottle_glass(AT_W, AT_D, AT_H, AT_NECK, AT_BASE, AT_WALL, flutes=7)
    return GLASSES[key].copy()


def glass_walls(key, base):
    return boolean(_glass(key), box(0.08, 0.02, 0.06).move(0, base - 0.01, 0), 'DIFFERENCE')


def glass_base(key, base):
    return boolean(_glass(key), box(0.08, 0.02, 0.06).move(0, base - 0.01, 0), 'INTERSECT')


SCENT_FILLS = (('Low', 0.22), ('Mid', 0.48), ('High', 0.74))


def scent(w, d, h, base, wall, fill):
    # what is left in it, `fill` of the way up the inside: the inside's
    # shape below that, a hair inside the glass
    ch = h - base - wall * 0.6 - 0.0003
    s = rbox(w - 2 * wall - 0.0003, ch, d - 2 * wall - 0.0003, r=0.0028, seg=2).move(0, base + 0.00015 + ch / 2, 0)
    top = base + 0.00015 + ch * fill
    return boolean(s, box(0.08, 0.06, 0.06).move(0, top + 0.03, 0), 'DIFFERENCE')


def flacon_pump():
    r, y0, y1 = FL_NECK
    # the ferrule crimped under the neck's bead, the stem, the actuator
    ferrule = lathe([(0.0, y1 + 0.0012), (r + 0.0016, y1 + 0.0012), (r + 0.0016, y1 - 0.0004), (r + 0.0018, y1 - 0.0030),
                     (r + 0.0006, y1 - 0.0038), (r + 0.0002, y1 - 0.0042), (r - 0.0004, y1 - 0.0045), (0.0, y1 - 0.0045)], seg=24)
    collar = lathe([(0.0, y1 + 0.0010), (0.0062, y1 + 0.0010), (0.0062, y1 + 0.0042), (0.0048, y1 + 0.0050), (0.0, y1 + 0.0050)], seg=20)
    act = lathe([(0.0, y1 + 0.0055), (0.0060, y1 + 0.0055), (0.0062, y1 + 0.0062), (0.0062, y1 + 0.0150), (0.0055, y1 + 0.0160),
                 (0.0, y1 + 0.0162)], seg=24)
    nozzle = cyl(0.0006, h=0.004, seg=6, bevel_r=0).rot(x=math.pi / 2).move(0, y1 + 0.0118, 0.0062)
    act = cut(act, nozzle)
    return ferrule + collar + act


def flacon_dip_tube():
    r, y0, y1 = FL_NECK
    pts = [Vector((0.0, y1 - 0.003, 0.0)), Vector((0.0004, 0.045, 0.0002)), Vector((0.0016, 0.025, 0.0006)),
           Vector((0.0034, FL_BASE + 0.0016, 0.0012))]
    return tube(pts, 0.0012, seg=6, smooth=2)


def flacon_cap():
    # an overcap of metallised plastic, square as the bottle is, over the pump
    # and down on the shoulder; hollow
    r, y0, y1 = FL_NECK
    yb = FL_H + 0.0003
    w, h = 0.0250, 0.0320
    outer = rbox(w, h, w, r=0.0022, seg=3).move(0, yb + h / 2, 0)
    inner = cyl(0.0098, h=0.040, seg=24, bevel_r=0).move(0, yb + h - 0.0022 - 0.020, 0)
    return cut(outer, inner)


# the atomizer's head, on the pressed glass's neck
def atom_collar(y1=None):
    r, y0, top = AT_NECK
    y1 = top if y1 is None else y1
    col = lathe([(0.0, y1 - 0.0058), (r + 0.0016, y1 - 0.0058), (r + 0.0020, y1 - 0.0050), (r + 0.0020, y1 - 0.0010),
                 (r + 0.0012, y1 + 0.0004), (0.0044, y1 + 0.0012), (0.0028, y1 + 0.0030), (0.0024, y1 + 0.0105),
                 (0.0030, y1 + 0.0112), (0.0030, y1 + 0.0128), (0.0, y1 + 0.0135)], seg=24)
    # knurled bands on the collar
    knurl = Shape()
    for i in range(30):
        a = TAU * i / 30
        knurl = knurl + box(0.0007, 0.0030, 0.0005).move(0, y1 - 0.0034, r + 0.0021).rot(y=a)
    # the spray head across its top (the nozzle at +z) and the spout for the hose (+x)
    head = cyl(0.0023, h=0.0110, seg=10, bevel_r=0.0004).rot(x=math.pi / 2).move(0, y1 + 0.0118, 0.0010)
    spout = cyl(0.0016, h=0.0080, seg=8, bevel_r=0.0003).rot(z=math.pi / 2).move(0.0055, y1 + 0.0118, 0)
    barb = cyl(0.0020, 0.0016, 0.0020, seg=8, bevel_r=0.0002).rot(z=math.pi / 2).move(0.0090, y1 + 0.0118, 0)
    return col + knurl + head + spout + barb


BULB_C = Vector((0.066, 0.0100, 0.012))     # the bulb's centre, lying on the floor
BULB_AX = Vector((1.0, 0.0, 0.32)).normalized()
BULB_NECK, BULB_END = -0.0215, 0.0200      # along its axis: the hose's end, the tassel's


def atom_bulb():
    # a rubber bulb 20 x 34 mm in a knotted silk net, its neck toward the
    # bottle, a cap where the tassel hangs; lying on the floor
    prof = [(0.0, BULB_NECK), (0.0024, BULB_NECK + 0.0002), (0.0026, BULB_NECK + 0.0040), (0.0034, BULB_NECK + 0.0055)]
    for i in range(1, 13):
        a = math.pi * i / 13
        prof.append((0.0100 * math.sin(a) * (1 + 0.08 * math.cos(a)), -0.0150 * math.cos(a) + 0.0005))
    prof += [(0.0030, BULB_END - 0.0030), (0.0026, BULB_END - 0.0012), (0.0, BULB_END)]
    s = lathe(prof, seg=20)
    # the net's knots: little bumps over it
    s = noisy(s, 0.00022, scale=1000.0, seed=3)
    rot = Vector((0, 1, 0)).rotation_difference(BULB_AX).to_matrix().to_4x4()
    s = s.apply(rot).move(*BULB_C)
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


def atom_tassel():
    # a silk tassel from the bulb's far end: its cord, the bound head, the
    # strands lying in a loose bunch on the floor
    end = BULB_C + BULB_AX * BULB_END
    head_c = end + Vector((0.011, 0.0, 0.006))
    head_c.y = 0.0028
    cord = tube([end, end + Vector((0.004, -0.0035, 0.002)), head_c - Vector((0.0025, 0.0, 0.0012))], 0.0007, seg=5, smooth=1)
    hd = Vector((0.88, 0.0, 0.47)).normalized()
    head = ellipsoid(0.0040, 0.0027, 0.0027).rot(y=-math.atan2(hd.z, hd.x)).move(*head_c)
    strands = Shape()
    n = 13
    for i in range(n):
        f = (i - (n - 1) / 2) / ((n - 1) / 2)
        a = math.atan2(hd.z, hd.x) + 0.20 * f + 0.05 * math.sin(i * 1.7)
        d = Vector((math.cos(a), 0.0, math.sin(a)))
        side = Vector((-d.z, 0.0, d.x))
        L = 0.030 + 0.003 * math.sin(i * 2.3)
        p0 = head_c + d * 0.0035 + side * f * 0.0016
        pts = [p0]
        for k in range(1, 5):
            t = k / 4
            pts.append(p0 + d * L * t + side * (0.0012 * math.sin(i * 1.3 + t * 5.0) + f * 0.004 * t)
                       + Vector((0, -0.0022 * min(1.0, t * 2.5), 0)))
        strands = strands + tube(pts, 0.00045, seg=4, smooth=0)
    s = cord + head + strands
    # nothing under the floor
    return s.warp(lambda p: Vector((p.x, max(p.y, 0.0004), p.z)))


def atom_hose(neck_top):
    # a hose in a woven silk cover from the collar's spout, arched down to the
    # bulb's neck on the floor
    a = Vector((0.0105, neck_top + 0.0118, 0.0))
    b = BULB_C - BULB_AX * (-BULB_NECK - 0.0025)
    mid1 = a + Vector((0.010, 0.002, 0.001))
    mid2 = Vector(((a.x + b.x) / 2 + 0.004, (a.y + b.y) / 2 + 0.004, (a.z + b.z) / 2))
    mid3 = b - BULB_AX * 0.010 + Vector((0, 0.004, 0))
    return tube([a, mid1, mid2, mid3, b], 0.0019, seg=8, smooth=2)


auto('Real__perfumeFlaconGlass', lambda: glass_walls('flacon', FL_BASE), 2200)
auto('Real__perfumeFlaconGlassBase', lambda: glass_base('flacon', FL_BASE), 700)
for lvl, fill in SCENT_FILLS:
    auto('Real__perfumeFlaconScent' + lvl, lambda fill=fill: scent(FL_W, FL_D, FL_H, FL_BASE, FL_WALL, fill), 300)
auto('Real__perfumeFlaconPump', flacon_pump, 700)
auto('Real__perfumeFlaconDipTube', flacon_dip_tube, 200)
auto('Real__perfumeFlaconCap', flacon_cap, 500)
auto('Real__perfumeAtomizerGlass', lambda: glass_walls('atomizer', AT_BASE), 4000)
auto('Real__perfumeAtomizerGlassBase', lambda: glass_base('atomizer', AT_BASE), 900)
for lvl, fill in SCENT_FILLS:
    auto('Real__perfumeAtomizerScent' + lvl, lambda fill=fill: scent(AT_W, AT_D, AT_H, AT_BASE, AT_WALL, fill), 300)
auto('Real__perfumeAtomizerCollar', atom_collar, 1200)
auto('Real__perfumeAtomizerBulb', atom_bulb, 900)
auto('Real__perfumeAtomizerHose', lambda: atom_hose(AT_NECK[2]), 600)
auto('Real__perfumeAtomizerHoseFlacon', lambda: atom_hose(FL_NECK[2]), 600)
auto('Real__perfumeAtomizerTassel', atom_tassel, 1100)
print('t03 perfume: flacon neck top %.4f, atomizer neck top %.4f; the scent levels (flacon, atomizer): %s'
      % (FL_NECK[2], AT_NECK[2], ', '.join('%s %.4f %.4f' % (l, FL_BASE + (FL_H - FL_BASE - FL_WALL * 0.6) * f, AT_BASE + (AT_H - AT_BASE - AT_WALL * 0.6) * f) for l, f in SCENT_FILLS)))


# ========================================================= the sponge ==
# A teardrop blending sponge (45 x 45 x 65 mm dry): hydrophilic
# polyurethane foam, a rounded base, a point. Modelled standing on its base;
# the game lays it on its side as it rests (SPONGE_LAY printed below).
def sponge_profile():
    R, H = 0.0225, 0.065
    pts = [(0.0, 0.0)]
    for i in range(1, 9):
        a = math.pi / 2 * i / 8
        pts.append((R * math.sin(a), R - R * math.cos(a)))
    for j in range(1, 15):
        k = j / 15
        pts.append((R * (1 - k ** 1.45) ** 0.78, R + k * (H - R)))
    pts.append((0.0, H))
    return pts


def sponge_foam():
    s = lathe(sponge_profile(), seg=28)
    # foam: never quite true, a dent where it was squeezed
    s = noisy(s, 0.0006, scale=60.0, seed=7)
    s = noisy(s, 0.00018, scale=420.0, seed=8)
    s = s.warp(lambda p: p - Vector((p.x, 0, p.z)).normalized() * 0.0022 * math.exp(-(((p.y - 0.030) / 0.010) ** 2)) * max(0.0, -p.x / 0.0225) ** 3
               if p.x < 0 else p)
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


auto('Real__makeupSpongeFoam', sponge_foam, 1400, smooth=70.0)


def sponge_rest():
    """The turn about z (from standing) that lays it on its side at rest:
    the centre of its volume lowest over the floor; and the lift then."""
    s = sponge_foam()
    # the centre of its volume, from the profile's discs
    prof = sponge_profile()
    vol, my = 0.0, 0.0
    for (ra, ya), (rb, yb) in zip(prof, prof[1:]):
        dv = math.pi * ((ra + rb) / 2) ** 2 * (yb - ya)
        vol += dv
        my += dv * (ya + yb) / 2
    cy = my / vol
    best = None
    for i in range(0, 241):
        a = -math.radians(60 + i * 0.25)
        c, sn = math.cos(a), math.sin(a)
        low = min(p.x * sn + p.y * c for p in s.v)
        h = cy * c - low
        if best is None or h < best[1]:
            best = (a, h, -low)
    return best


SPONGE_LAY = sponge_rest()
print('t03 sponge: lay %.5f rad, centre %.5f m up, lift %.5f' % SPONGE_LAY)
