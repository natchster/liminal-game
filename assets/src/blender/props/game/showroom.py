"""Family `showroom` - the Showroom (store level), plus two small sets for the lead.

Every primitive is wrapped in pg('Store__...' / 'Staff__...') in the game
(`anomalies::store_kit`, the store's placer and `make_staff`).

store_kit      bed, nightstand, table lamp, wardrobe, rug, sofa, armchair, coffee
              table, floor lamp, bookcase, TV bench + TV, kitchen run, fridge,
              dining table + chair, desk, office chair, bunk bed, toy box,
              pallet racking + cartons, shopping trolley, price-tag stand
place_store    department sign board, hours board post and frame
make_staff     the too-tall faceless employee: polo shirt torso, collar, badge,
              neck, blank head, sleeves, forearms, hands, trousers, shoes
make_shadow    (Shade__) the Borrowed House's thin shadow figures: silhouettes
place_house    (Nest__) the heaped brass/iron lamps and the wooden crate
"""
from propkit import *
import propkit

HALF = math.pi / 2


# ------------------------------------------------------------- helpers ---
def loft(rings, cap_bottom=True, cap_top=True):
    """Connect 2D outlines (same point count) stacked along Z into a solid.
    rings: [(outline [(x, y), ...], z), ...] from back to front."""
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


def ystack(rings, cap_bottom=True, cap_top=True):
    """Horizontal outlines [(x, z), ...] stacked up +Y: rings [(outline, y)], bottom first."""
    return loft([([(x, -z) for x, z in pts], y) for pts, y in rings], cap_bottom, cap_top).rot(x=-HALF)


def rr(w, h, r, seg=2, cx=0.0, cy=0.0):
    """Rounded rectangle outline with a constant point count."""
    r = max(min(r, w / 2 - 1e-5, h / 2 - 1e-5), 2e-4)
    return [(x + cx, y + cy) for x, y in rounded_rect(w, h, r, seg)]


def plate(w, h, profile, r=0.004, seg=2):
    """A plate built from the back (first) to the front (last):
    profile = [(inset, z), ...]; each ring is the rounded outline inset."""
    return loft([(rr(w - 2 * i, h - 2 * i, r - i, seg), z) for i, z in profile])


def frame_ring(wo, ho, wi, hi, d, z=0.0):
    """A rectangular ring (outer minus inner), d deep, centred at z (a groove cutter)."""
    return cut(box(wo, ho, d), box(wi, hi, d * 2)).move(z=z)


def sq_post(profile, r=0.004, seg=2):
    """A square-section post: profile [(half_width, y), ...] bottom to top."""
    return ystack([(rr(2 * hw, 2 * hw, min(r, hw * 0.8), seg), y) for hw, y in profile])


def rect_post(profile, r=0.004, seg=2):
    """A rectangular post: profile [(half_x, half_z, y), ...] bottom to top."""
    return ystack([(rr(2 * hx, 2 * hz, min(r, hx * 0.8, hz * 0.8), seg), y) for hx, hz, y in profile])


def edge_coords(h, n, e):
    """Grid coordinates from -h to h: dense within e of both ends, n even steps between."""
    inner = [(-h + e) + (2 * (h - e)) * i / n for i in range(n + 1)]
    return [-h, -h + e * 0.3] + inner + [h - e * 0.3, h]


def gbox(xs, ys, zs, bottom=True):
    """A closed box whose six faces are grids on the given coordinates.
    bottom=False closes the underside with a single polygon (cheap, hidden)."""
    v, f = [], []

    def face(us, ws, mk):
        base = len(v)
        n = len(us)
        for b in ws:
            for a in us:
                v.append(mk(a, b))
        for j in range(len(ws) - 1):
            for i in range(n - 1):
                k = base + j * n + i
                f.append((k, k + 1, k + n + 1, k + n))

    face(xs, zs, lambda a, b: (a, ys[-1], b))
    if bottom:
        face(xs, zs, lambda a, b: (a, ys[0], b))
    else:
        y0 = ys[0]
        loop = [(x, y0, zs[0]) for x in xs] + [(xs[-1], y0, z) for z in zs[1:]] + \
               [(x, y0, zs[-1]) for x in reversed(xs[:-1])] + [(xs[0], y0, z) for z in reversed(zs[1:-1])]
        base = len(v)
        v += loop
        f.append(tuple(range(base, base + len(loop))))
    face(xs, ys, lambda a, b: (a, b, zs[-1]))
    face(xs, ys, lambda a, b: (a, b, zs[0]))
    face(zs, ys, lambda a, b: (xs[-1], b, a))
    face(zs, ys, lambda a, b: (xs[0], b, a))
    return propkit._fix_winding(weld(Shape(v, f), 1e-7))


def round_box_map(p, H, r):
    """Push a point on a box surface (half sizes H) onto the rounded box of radius r."""
    c = Vector([max(-H[k] + r, min(H[k] - r, p[k])) for k in range(3)])
    d = p - c
    L = d.length
    return c + d * (r / L) if L > 1e-9 else p


def softbox(w, h, d, r, n=(4, 1, 4), puff=(0.0, 0.0, 0.0), bottom=True, xs=None, ys=None, zs=None):
    """An upholstered block: rounded edges of radius r and faces that bulge
    out by `puff` (per axis) in the middle. Returns the shape before fitting."""
    H = (w / 2, h / 2, d / 2)
    xs = xs or edge_coords(H[0], n[0], r)
    ys = ys or edge_coords(H[1], n[1], r)
    zs = zs or edge_coords(H[2], n[2], r)
    s = gbox(xs, ys, zs, bottom)

    def fn(p):
        q = round_box_map(p, H, r)
        out = q.copy()
        for a in range(3):
            b, c = [k for k in range(3) if k != a]
            wa = (abs(q[a]) / H[a]) ** 6
            g = max(0.0, 1 - (q[b] / H[b]) ** 2) * max(0.0, 1 - (q[c] / H[c]) ** 2)
            out[a] += math.copysign(puff[a] * wa * g, q[a])
        return out
    return s.warp(fn)


def loop_tube(pts, r, seg=5):
    """A closed tube through 3D points (a ring of piping, a wire loop)."""
    return tube(list(pts) + [pts[0]], r, seg=seg, caps=False)


def pipe_rect(w, d, rc, y, r, seg=5, cseg=2):
    """Piping around a rounded rectangle w x d (in XZ) at height y."""
    return loop_tube([(x, y, z) for x, z in rounded_rect(w, d, rc, cseg)], r, seg)


def on_z(s):
    """A lathe built on +Y turned to face +Z."""
    return s.rot(x=HALF)


def knob(r, h, seg=12):
    """A round furniture knob on +Z, base at z=0: neck and a domed cap."""
    return on_z(lathe([(r * 0.62, 0), (r * 0.5, h * 0.28), (r * 0.62, h * 0.42), (r, h * 0.62),
                       (r * 0.92, h * 0.88), (r * 0.55, h), (0, h)], seg, cap_bottom=True))


def bar_pull(L, r, stand, seg=8, along='x'):
    """A bar handle on two standoffs on +Z (base at z=0), length L along x or y."""
    a = L / 2
    pts = [(-a, 0, -0.001), (-a, 0, stand * 0.75), (-a * 0.96, 0, stand), (a * 0.96, 0, stand), (a, 0, stand * 0.75), (a, 0, -0.001)]
    s = tube(pts, r, seg=seg, smooth=1)
    return s.rot(z=HALF) if along == 'y' else s


def lathe_loop(prof, seg):
    """Revolve a closed (r, y) loop into a ring-shaped solid."""
    s = lathe(list(prof) + [prof[0]], seg, cap_bottom=False, cap_top=False)
    return propkit._fix_winding(weld(s, 1e-6))


# =============================================================== BEDROOM ===
# ------------------------------------------------------------------- bed --
def bed_leg():
    # a tapered square leg: straight block where it meets the frame, a
    # chamfered taper, a small floor glide
    return sq_post([(0.019, -0.17), (0.023, -0.166), (0.025, -0.162), (0.0335, 0.10), (0.035, 0.105),
                    (0.035, 0.166), (0.033, 0.17)], r=0.005, seg=1)


def bed_base(W, H, D, t=0.032, n_slats=16, beam=True):
    """Side and end rails around a slatted base on ledgers: what you see
    from the side, and from underneath when hiding under it."""
    hw, hh, hd = W / 2, H / 2, D / 2
    rail = lambda: rbox(t, H, D, r=0.007, seg=2)
    s = rail().move(x=-hw + t / 2) + rail().move(x=hw - t / 2)
    for sz in (-1, 1):
        s = s + rbox(W - 2 * t + 0.002, H * 0.94, t, r=0.006, seg=2).move(y=-H * 0.03, z=sz * (hd - t / 2))
    # ledgers on the inside of the side rails carry the slats
    st = 0.018
    for sx in (-1, 1):
        s = s + box(0.024, 0.032, D - 2 * t).move(x=sx * (hw - t - 0.012), y=hh - st - 0.018)
    inner = W - 2 * t - 0.004
    pitch = (D - 2 * t - 0.08) / (n_slats - 1)
    for i in range(n_slats):
        z = -hd + t + 0.04 + i * pitch
        s = s + rbox(inner, st, 0.068, r=0.003, seg=1).move(y=hh - st / 2 - 0.004, z=z)
    if beam:
        s = s + box(0.05, 0.07, D - 2 * t).move(y=hh - st - 0.04)
    return s


def mattress(W, H, D, pitch=0.2, r=0.035, groove=0.012):
    """A channel-quilted mattress: rounded edges, puffy channels across the
    top divided by stitched V-grooves, a tape-edge piping round the top."""
    hx, hy, hz = W / 2, H / 2, D / 2
    n = max(3, int(round((D - 2 * r) / pitch)))
    p = (D - 2 * r) / n
    lines = [-hz + r + k * p for k in range(1, n)]
    centres = [-hz + r + (k + 0.5) * p for k in range(n)]
    zs = sorted([-hz, -hz + r * 0.3, -hz + r, hz - r, hz - r * 0.3, hz] + centres + lines +
                [z + d for z in lines for d in (-0.012, 0.012)])
    xs = edge_coords(hx, 4, r)
    ys = [-hy, -hy + r, hy - r, hy - r * 0.3, hy]
    s = gbox(xs, ys, zs, bottom=False)
    Hh = (hx, hy, hz)

    def near(z, pts, tol=2e-5):
        return any(abs(z - q) < tol for q in pts)

    def fn(q0):
        top = q0.y > hy - 1e-5
        q = round_box_map(q0, Hh, r)
        if top and abs(q0.x) < hx - r + 1e-5 and abs(q0.z) < hz - r + 1e-5:
            g = max(0.0, 1 - (q0.x / (hx - r)) ** 8)
            if near(q0.z, lines):
                q.y -= groove * g
            elif near(q0.z, [z + d for z in lines for d in (-0.012, 0.012)]):
                q.y -= groove * 0.25 * g
            elif near(q0.z, centres):
                q.y += 0.005 * g
        if abs(q.y) < hy - r * 0.5:                      # the side walls bulge a little
            f = 1 - (q.y / hy) ** 2
            if abs(q.x) > hx - 1e-4 - r:
                q.x += math.copysign(0.004 * f, q.x) * (abs(q.x) / hx) ** 8
            if abs(q.z) > hz - 1e-4 - r:
                q.z += math.copysign(0.004 * f, q.z) * (abs(q.z) / hz) ** 8
        return q
    s = s.warp(fn)
    a = 0.707
    pw, pd = W - 2 * r * (1 - a), D - 2 * r * (1 - a)
    s = s + pipe_rect(pw, pd, r * a + 0.004, hy - r * (1 - a), 0.0045, seg=5, cseg=2)
    return s.fit((-hx, -hy, -hz), (hx, hy, hz))


def duvet():
    # a duvet over the foot two-thirds of the mattress: puffy, drooping over
    # the sides and the foot, its head end turned down in a thick fold
    W, D = 1.6, 1.35
    s = slab(W, D, 0.05, 22, 18).rot(x=-HALF)          # XZ plane, thickness along Y
    hw, hd = W / 2, D / 2

    def fn(p):
        u, w = p.x / hw, p.z / hd
        top = p.y > 0
        # puffy body: thicker in the middle, pinched to a seam at the rim
        edge = max(0.0, min(1.0, (1 - abs(u)) * 9, (1 - abs(w)) * 9))
        thick = 0.012 + 0.045 * edge ** 0.6
        y = (thick if top else 0.0) - 0.02
        # the turned-down fold at the head end
        if w < -0.72:
            k = (w + 1) / 0.28
            y += (0.016 * max(0.0, math.sin(k * math.pi)) ** 0.7) if top else 0.004
        if -0.74 < w < -0.66 and top:
            y -= 0.008 * (1 - abs(w + 0.70) / 0.04)
        # droop over the sides and the foot
        dx = max(0.0, (abs(u) - 0.9) / 0.1)
        dz = max(0.0, (w - 0.9) / 0.1)
        y -= 0.052 * (dx ** 1.6) + 0.045 * (dz ** 1.6)
        return Vector((p.x * (1 - 0.012 * dx), y, p.z * (1 - 0.01 * dz)))
    s = s.warp(fn)
    s = noisy(s, 0.006, scale=3.2, seed=7, axis=(0, 1, 0))
    s = noisy(s, 0.0025, scale=9.0, seed=2)
    return s.fit((-hw, -0.074, -hd), (hw, 0.04, hd))


def pillow():
    # a stuffed pillow: domed both faces, pinched seam, concave sides that
    # leave the corners as little ears
    W, D, T = 0.62, 0.38, 0.12
    s = slab(W, D, T, 14, 10).rot(x=-HALF)
    hw, hd = W / 2, D / 2

    def fn(p):
        u, w = p.x / hw, p.z / hd
        g = max(0.0, 1 - abs(u) ** 2.4) ** 0.55 * max(0.0, 1 - abs(w) ** 2.4) ** 0.55
        y = p.y * max(0.1, g)
        x = p.x * (1 - 0.07 * (1 - w * w))
        z = p.z * (1 - 0.09 * (1 - u * u))
        return Vector((x, y, z))
    s = s.warp(fn)
    s = noisy(s, 0.004, scale=7, seed=4)
    return s.fit((-hw, -T / 2, -hd), (hw, T / 2, hd))


def headboard():
    # posts with capped tops, a top rail with a cap board, a bottom rail and
    # a tongue-and-groove panel between them, all facing the mattress (+Z)
    W, H, T = 1.7, 1.05, 0.08
    hw, hh = W / 2, H / 2
    pw = 0.075
    px = hw - 0.0425
    s = Shape()
    for sx in (-1, 1):
        s = s + rbox(pw, H - 0.03, pw, r=0.008, seg=2).move(x=sx * px, y=-0.015)
        s = s + rbox(0.085, 0.03, T, r=0.008, seg=2).move(x=sx * px, y=hh - 0.015)
    inner = 2 * (px - pw / 2) + 0.004
    s = s + rbox(inner, 0.13, 0.05, r=0.008, seg=2).move(y=hh - 0.03 - 0.065)
    s = s + rbox(inner + 0.02, 0.028, 0.066, r=0.008, seg=2).move(y=hh - 0.014 - 0.004)
    s = s + rbox(inner, 0.10, 0.05, r=0.006, seg=2).move(y=-0.30)
    ptop, pbot = hh - 0.16, -0.25
    panel = rbox(inner - 0.002, ptop - pbot, 0.022, r=0.003, seg=1).move(y=(ptop + pbot) / 2)
    n = 12
    cutters = Shape()
    for i in range(1, n):
        x = -inner / 2 + inner * i / n
        for sz in (-1, 1):
            cutters = cutters + box(0.006, ptop - pbot + 0.02, 0.006).rot(y=math.pi / 4).move(
                x=x, y=(ptop + pbot) / 2, z=sz * 0.011)
    s = s + cut(panel, cutters)
    return s


part('Store__bed_leg', f_box(0.07, 0.34, 0.07), bed_leg)
part('Store__bed_frame', f_box(1.64, 0.16, 2.1), lambda: bed_base(1.64, 0.16, 2.1))
part('Store__bed_mattress', f_box(1.56, 0.2, 2.0), lambda: mattress(1.56, 0.2, 2.0), smooth=50, weighted=False)
part('Store__bed_duvet', f_box(1.6, 0.08, 1.35), duvet, smooth=180)
part('Store__bed_pillow', f_box(0.62, 0.12, 0.38), pillow, smooth=180)
part('Store__bed_headboard', f_box(1.7, 1.05, 0.08), headboard)


# ------------------------------------------------------------ nightstand --
def stand_body():
    # top with an overhang, two sides, a bottom and a divider under the
    # drawer, an open niche below it, a back panel and four short legs
    s = rbox(0.46, 0.03, 0.40, r=0.007, seg=2).move(y=0.235)
    for sx in (-1, 1):
        s = s + rbox(0.022, 0.43, 0.385, r=0.004, seg=2).move(x=sx * 0.214, y=0.005, z=-0.0055)
    iw = 0.406
    s = s + rbox(iw, 0.022, 0.37, r=0.003, seg=1).move(y=-0.199, z=-0.01)
    s = s + rbox(iw, 0.018, 0.37, r=0.003, seg=1).move(y=0.027, z=-0.01)
    s = s + box(iw - 0.004, 0.13, 0.33).move(y=0.11, z=0.012)          # the drawer box
    s = s + box(0.44, 0.40, 0.008).move(y=0.01, z=-0.19)
    for sx in (-1, 1):
        for sz in (-1, 1):
            s = s + sq_post([(0.012, -0.25), (0.015, -0.246), (0.017, -0.21)], r=0.003, seg=1).move(x=sx * 0.19, z=sz * 0.15)
    return s


def stand_drawer():
    # a drawer front with eased edges and a turned knob
    front = rbox(0.4, 0.14, 0.014, r=0.0035, seg=2).move(z=-0.003)
    return front + knob(0.0115, 0.015, seg=12).move(z=0.004)


part('Store__stand_body', f_box(0.46, 0.5, 0.4), stand_body)
part('Store__stand_drawer', f_box(0.4, 0.14, 0.02), stand_drawer)


# ------------------------------------------------------------ table lamp --
def lamp_base_turned():
    # a turned base: chamfered foot, an ogee shoulder, a collar for the stem
    return lathe([(0, -0.025), (0.086, -0.025), (0.09, -0.021), (0.09, -0.016), (0.084, -0.011), (0.074, -0.003),
                  (0.07, 0.006), (0.064, 0.013), (0.045, 0.019), (0.022, 0.021), (0.018, 0.025), (0, 0.025)], seg=24)


def lamp_stem(r=0.012, h=0.3, seg=12):
    # a tube: a lock nut at the foot, a switch collar, the bulb socket on top
    y0, y1 = -h / 2, h / 2
    rs = r * 0.62
    prof = [(0, y0), (r, y0), (r, y0 + 0.012), (rs * 1.15, y0 + 0.016), (rs, y0 + 0.02),
            (rs, y1 - 0.055), (r * 0.95, y1 - 0.05), (r * 0.95, y1 - 0.042), (rs, y1 - 0.038),
            (rs, y1 - 0.032), (r * 1.25, y1 - 0.028), (r * 1.4, y1 - 0.02), (r * 1.4, y1 - 0.004),
            (r * 1.25, y1), (0, y1)]
    return lathe(prof, seg)


def lampshade(rt, rb, h, seg=24, spider=True, bead=0.0035, wall=0.0022):
    """An open lampshade shell: rolled bead rims top and bottom, an inside
    surface, and a wire spider holding the washer (the shade hangs on it)."""
    y0, y1 = -h / 2, h / 2
    lr = lambda y: rb + (rt - rb) * (y - y0) / h - 0.0015
    b = bead
    prof = [(rb - b * 0.9, y0), (rb, y0 + b * 0.55), (rb, y0 + b * 1.45), (lr(y0 + b * 2.3), y0 + b * 2.3),
            (lr(y1 - b * 2.3), y1 - b * 2.3), (rt, y1 - b * 1.45), (rt, y1 - b * 0.55), (rt - b * 0.9, y1),
            (rt - b * 1.7, y1 - b * 0.7), (lr(y1 - b * 2.3) - wall, y1 - b * 2.3),
            (lr(y0 + b * 2.3) - wall, y0 + b * 2.3), (rb - b * 1.7, y0 + b * 0.7)]
    s = lathe_loop(prof, seg)
    # a thin vertical seam strip where the shade material is joined
    seam = tube([(0, y0 + b * 2.3, lr(y0 + b * 2.3) + 0.0004), (0, y1 - b * 2.3, lr(y1 - b * 2.3) + 0.0004)],
                0.0012, seg=4, caps=True)
    s = s + seam
    if spider:
        yw = y1 - min(0.04, h * 0.18)
        ri = rt - b * 1.8 - wall
        for k in range(3):
            a = TAU * k / 3 + 0.3
            s = s + tube([(ri * math.sin(a), y1 - b * 1.2, ri * math.cos(a)),
                          (0.016 * math.sin(a), yw, 0.016 * math.cos(a))], 0.0013, seg=4, caps=False)
        s = s + torus(0.014, 0.0028, seg=12, tseg=5).rot(x=HALF).move(y=yw)
    return s


part('Store__lamp_base', f_cyl(0.07, 0.09, 0.05), lamp_base_turned)
part('Store__lamp_stem', f_cyl(0.012, 0.012, 0.3), lamp_stem, smooth=50)
part('Store__lamp_shade', f_cyl(0.1, 0.16, 0.2), lambda: lampshade(0.1, 0.16, 0.2), smooth=50)


# ------------------------------------------------------------- wardrobe --
def wardrobe_body():
    # a hollow carcass (the player hides inside it): sides, a top with a
    # front valance above the doors, a bottom on a recessed plinth, a back,
    # a hat shelf and a hanging rail
    W, H, D = 1.2, 2.05, 0.6
    hw, hh, hd = W / 2, H / 2, D / 2
    t = 0.02
    iw = W - 2 * t + 0.002
    s = Shape()
    for sx in (-1, 1):
        s = s + rbox(t, H, D, r=0.004, seg=2).move(x=sx * (hw - t / 2))
    s = s + rbox(iw, 0.022, D - 0.01, r=0.003, seg=1).move(y=hh - 0.011, z=-0.005)
    s = s + rbox(iw, 0.03, 0.02, r=0.003, seg=1).move(y=hh - 0.022 - 0.015, z=hd - 0.012)
    s = s + rbox(iw, 0.02, D - 0.03, r=0.003, seg=1).move(y=-hh + 0.05 + 0.01, z=-0.01)
    s = s + box(iw, 0.05, 0.018).move(y=-hh + 0.025, z=hd - 0.045)                 # recessed kick board
    s = s + box(W - 0.02, H - 0.02, 0.008).move(z=-hd + 0.006)
    s = s + rbox(iw, 0.018, 0.5, r=0.003, seg=1).move(y=0.80, z=-0.03)             # hat shelf
    s = s + tube([(-hw + t, 0.70, 0.0), (hw - t, 0.70, 0.0)], 0.012, seg=10, caps=False)
    for sx in (-1, 1):
        s = s + cyl(0.022, 0.022, 0.008, seg=12, bevel_r=0.002).rot(z=HALF).move(x=sx * (hw - t - 0.004), y=0.70)
    return s


def wardrobe_door():
    # a shaker door: stiles and rails around two chamfered, recessed panels
    W, H, T = 0.58, 1.95, 0.025
    s = rbox(W, H, T, r=0.004, seg=2)
    fz = T / 2
    b = 0.075
    cutters = Shape()
    for y0, y1 in ((-H / 2 + b + 0.02, -0.16), (-0.16 + 0.09, H / 2 - b)):
        w, h = W - 2 * b, y1 - y0
        c = plate(w + 0.012, h + 0.012, [(0.006, fz - 0.006), (0.0, fz), (0.0, fz + 0.01)], r=0.004, seg=1)
        cutters = cutters + c.move(y=(y0 + y1) / 2)
    return cut(s, cutters)


def wardrobe_handle():
    # a round bar pull on two posts, with roses against the door
    s = tube([(0, -0.106, -0.016), (0, -0.106, 0.0085), (0, 0.106, 0.0085), (0, 0.106, -0.016)], 0.0055, seg=8, smooth=2)
    for sy in (-1, 1):
        s = s + cyl(0.0095, 0.0095, 0.004, seg=10, bevel_r=0.0012).rot(x=HALF).move(y=sy * 0.106, z=-0.013)
    return s.fit((-0.01, -0.12, -0.015), (0.01, 0.12, 0.015))


part('Store__wardrobe_body', f_box(1.2, 2.05, 0.6), wardrobe_body)
part('Store__wardrobe_door', f_box(0.58, 1.95, 0.025), wardrobe_door)
part('Store__wardrobe_handle', f_box(0.02, 0.24, 0.03), wardrobe_handle, smooth=60)


# ------------------------------------------------------------------ rug --
def rug():
    # a flat-woven rug: a rolled binding round the edge, a carved border line,
    # a pile that ripples a little, a short fringe on both ends
    W, T, D = 2.2, 0.012, 1.6
    hx, hz = W / 2, D / 2
    rim = [0.0, 0.006, 0.014, 0.022, 0.034, 0.142, 0.15, 0.158, 0.26]
    def coords(h, step):
        n = max(1, int(round((2 * (h - 0.26)) / step)))
        mid = [-(h - 0.26) + 2 * (h - 0.26) * i / n for i in range(1, n)]
        return [-h + d for d in rim] + mid + [h - d for d in reversed(rim)]
    xs, zs = coords(hx, 0.2), coords(hz, 0.2)
    s = gbox(xs, [-T / 2, T / 2], zs, bottom=False)
    prof = [(0.0, 0.004), (0.006, 0.0102), (0.014, 0.012), (0.022, 0.0094), (0.034, 0.0104),
            (0.142, 0.0104), (0.15, 0.0074), (0.158, 0.0104)]

    def height(d):
        for (d0, y0), (d1, y1) in zip(prof, prof[1:]):
            if d <= d1:
                return y0 + (y1 - y0) * (d - d0) / (d1 - d0)
        return 0.0104

    def fn(p):
        if p.y > 0:
            d = min(hx - abs(p.x), hz - abs(p.z))
            y = -T / 2 + height(d)
            if d > 0.03:
                y += 0.0012 * noise.noise(Vector((p.x * 2.1, 0.3, p.z * 2.1)))
            return Vector((p.x, y, p.z))
        return p
    s = s.warp(fn)
    rnd = random.Random(5)
    for sx in (-1, 1):
        n = 22
        for i in range(n):
            z = -hz + 0.05 + (D - 0.1) * i / (n - 1) + rnd.uniform(-0.01, 0.01)
            L = 0.035 + rnd.uniform(-0.006, 0.006)
            a = rnd.uniform(-0.25, 0.25)
            x0 = sx * (hx - 0.004)
            x1, z1 = x0 + sx * L * math.cos(a), z + L * math.sin(a)
            wv = 0.004
            v = [(x0, -T / 2 + 0.0015, z - wv), (x1, -T / 2 + 0.001, z1 - wv), (x1, -T / 2 + 0.001, z1 + wv), (x0, -T / 2 + 0.0015, z + wv)]
            q = Shape(v, [(0, 3, 2, 1)] if sx > 0 else [(0, 1, 2, 3)])
            s = s + q
    return s


part('Store__rug', f_box(2.2, 0.012, 1.6), rug, smooth=50)


# ----------------------------------------------------------------- sofa --
def crease_puff(x, width, offset):
    """0 at the crease lines between cushions (every `width`), 1 mid-cushion."""
    return abs(math.sin(math.pi * (x - offset) / width)) ** 0.7


def sofa_base():
    # the upholstered base under the seat cushions: a puffy front rail with
    # piping along its top and bottom edges
    W, H, D, r = 2.1, 0.3, 0.92, 0.03
    s = softbox(W, H, D, r, n=(10, 2, 4), puff=(0.004, 0.004, 0.014), bottom=False)
    a = 0.707
    for y in (H / 2 - r * (1 - a), -H / 2 + r * (1 - a)):
        s = s + pipe_rect(W - 2 * r * (1 - a), D - 2 * r * (1 - a), r * a + 0.003, y, 0.005, seg=5)
    return s.fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))


def upholstered_back(W, H, D, r, cushion_w, offset, puff=0.03, n=(24, 4, 2), pipe=True):
    """A back rest: rounded top, the front split into puffy cushions by creases."""
    hx, hy, hz = W / 2, H / 2, D / 2
    s = softbox(W, H, D, r, n=n, bottom=False)

    def fn(p):
        if p.z > hz - r * 1.2:
            k = max(0.0, (p.z - (hz - r * 1.2)) / (r * 1.2))
            gy = max(0.0, 1 - ((p.y + hy * 0.15) / (hy * 1.05)) ** 2)
            p.z += puff * k * crease_puff(p.x, cushion_w, offset) * gy
        if p.y > hy - r:                                  # the top sags a little between creases
            p.y -= 0.012 * (1 - crease_puff(p.x, cushion_w, offset))
        return p
    s = s.warp(fn)
    if pipe:
        a = 0.707
        pts = [(x, y, -hz + r * (1 - a)) for x, y in rounded_rect(W - 2 * r * (1 - a), H - 2 * r * (1 - a), r * a + 0.004, 2)]
        s = s + loop_tube(pts, 0.005, seg=5)
    return s.fit((-hx, -hy, -hz), (hx, hy, hz + 0.0))


def sofa_arm():
    # a track arm: rounded top edges, gently puffed faces, piping around both
    # side panels
    W, H, D, r = 0.18, 0.62, 0.92, 0.05
    s = softbox(W, H, D, r, n=(2, 5, 6), puff=(0.008, 0.01, 0.008), bottom=False)
    a = 0.707
    for sx in (-1, 1):
        pts = [(sx * (W / 2 - r * (1 - a)), y, z) for z, y in rounded_rect(D - 2 * r * (1 - a), H - 2 * r * (1 - a), r * a + 0.004, 2)]
        s = s + loop_tube(pts, 0.005, seg=5)
    return s.fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))


def seat_cushion(W, H, D, r=0.035, puff=(0.008, 0.02, 0.008), seed=1, n=(6, 1, 6), bottom_pipe=True):
    """A boxed seat cushion: domed top, piping round the top and bottom seams."""
    s = softbox(W, H, D, r, n=n, puff=puff, bottom=False)
    s = noisy(s, 0.0025, scale=6, seed=seed)
    a = 0.707
    for y in (H / 2 + puff[1] * 0.1 - r * (1 - a), -H / 2 + r * (1 - a))[:2 if bottom_pipe else 1]:
        s = s + pipe_rect(W - 2 * r * (1 - a), D - 2 * r * (1 - a), r * a + 0.004, y, 0.005, seg=5)
    return s.fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))


def sofa_foot():
    # a turned, tapered foot on a square mounting plate
    s = lathe([(0, -0.075), (0.0155, -0.075), (0.0168, -0.072), (0.0172, -0.066), (0.0162, -0.063),
               (0.0175, -0.06), (0.0232, 0.058), (0.0232, 0.066), (0, 0.066)], seg=14)
    return s + rbox(0.05, 0.009, 0.05, r=0.002, seg=1).move(y=0.0705)


part('Store__sofa_base', f_box(2.1, 0.3, 0.92), sofa_base, smooth=60)
part('Store__sofa_back', f_box(2.1, 0.5, 0.22), lambda: upholstered_back(2.1, 0.5, 0.22, 0.06, 0.58, -0.29, puff=0.045), smooth=60)
part('Store__sofa_arm', f_box(0.18, 0.62, 0.92), sofa_arm, smooth=60)
part('Store__sofa_cushion', f_box(0.58, 0.14, 0.66), lambda: seat_cushion(0.58, 0.14, 0.66), smooth=60)
part('Store__sofa_foot', f_box(0.05, 0.15, 0.05), sofa_foot)


# ------------------------------------------------------------- armchair --
def chair_seat():
    # a skirted base with front piping, and the loose seat cushion on it
    W, H, D = 0.78, 0.3, 0.82
    base = softbox(W, 0.2, D, 0.03, n=(3, 1, 1), puff=(0.004, 0.0, 0.012), bottom=False)
    a = 0.707
    base = base + pipe_rect(W - 0.06 * (1 - a), D - 0.06 * (1 - a), 0.03 * a + 0.003, 0.1 - 0.03 * (1 - a), 0.005, seg=4)
    base = base.fit((-W / 2, -H / 2, -D / 2), (W / 2, -H / 2 + 0.2, D / 2))
    cush = seat_cushion(0.585, 0.13, 0.63, r=0.035, puff=(0.006, 0.018, 0.01), seed=3, n=(3, 1, 3), bottom_pipe=False)
    cush = cush.move(y=H / 2 - 0.065, z=D / 2 - 0.315 - 0.004)
    # upholstered rails under the arms and the back, so they have something to sit on
    rails = Shape()
    for sx in (-1, 1):
        rails = rails + rbox(0.11, 0.12, D, r=0.018, seg=1).move(x=sx * (W / 2 - 0.055), y=H / 2 - 0.06)
    rails = rails + rbox(W - 0.2, 0.12, 0.17, r=0.018, seg=1).move(y=H / 2 - 0.06, z=-D / 2 + 0.085)
    return base + cush + rails


def chair_arm():
    W, H, D, r = 0.12, 0.3, 0.82, 0.045
    s = softbox(W, H, D, r, n=(2, 3, 6), puff=(0.006, 0.01, 0.006), bottom=False)
    a = 0.707
    for sx in (-1, 1):
        pts = [(sx * (W / 2 - r * (1 - a)), y, z) for z, y in rounded_rect(D - 2 * r * (1 - a), H - 2 * r * (1 - a), r * a + 0.004, 2)]
        s = s + loop_tube(pts, 0.0045, seg=5)
    return s.fit((-W / 2, -H / 2, -D / 2), (W / 2, H / 2, D / 2))


part('Store__chair_seat', f_box(0.78, 0.3, 0.82), chair_seat, smooth=60)
part('Store__chair_back', f_box(0.78, 0.55, 0.18),
     lambda: upholstered_back(0.78, 0.55, 0.18, 0.06, 0.26, -0.13, puff=0.028, n=(10, 4, 2)), smooth=60)
part('Store__chair_arm', f_box(0.12, 0.3, 0.82), chair_arm, smooth=60)


# --------------------------------------------------------- coffee table --
def ctable_top():
    # three glued boards (V-grooves between them), eased edges
    s = rbox(1.1, 0.05, 0.6, r=0.006, seg=2)
    g = Shape()
    for z in (-0.1, 0.1):
        g = g + box(1.2, 0.005, 0.005).rot(x=math.pi / 4).move(y=0.025, z=z)
    return cut(s, g)


def ctable_shelf():
    # a slatted shelf on two battens
    s = Shape()
    for i in range(5):
        s = s + rbox(1.0, 0.018, 0.088, r=0.003, seg=1).move(y=0.006, z=-0.206 + i * 0.103)
    for sx in (-1, 1):
        s = s + box(0.045, 0.012, 0.49).move(x=sx * 0.45, y=-0.009)
    return s


def ctable_leg():
    return sq_post([(0.017, -0.215), (0.0195, -0.211), (0.02, -0.205), (0.025, 0.17), (0.025, 0.211), (0.023, 0.215)],
                   r=0.004, seg=1)


part('Store__ctable_top', f_box(1.1, 0.05, 0.6), ctable_top)
part('Store__ctable_shelf', f_box(1.0, 0.03, 0.5), ctable_shelf)
part('Store__ctable_leg', f_box(0.05, 0.43, 0.05), ctable_leg)


# ----------------------------------------------------------- floor lamp --
def flamp_base(rt=0.16, rb=0.18, h=0.03):
    # a weighted disc: eased rim, a shallow dome, a collar for the pole
    y0, y1 = -h / 2, h / 2
    return lathe([(0, y0), (rb - 0.004, y0), (rb, y0 + 0.003), (rb, y0 + 0.008), (rb - 0.008, y0 + 0.012),
                  (rt - 0.01, y0 + 0.019), (rt * 0.6, y1 - 0.006), (0.032, y1 - 0.003), (0.026, y1 - 0.0025),
                  (0.024, y1), (0, y1)], seg=32)


def flamp_pole(r=0.015, h=1.55):
    # a two-piece pole: spigot into the base, a joint collar, a socket cup
    y0, y1 = -h / 2, h / 2
    rp = r * 0.72
    prof = [(0, y0), (r, y0), (r, y0 + 0.03), (rp * 1.1, y0 + 0.036), (rp, y0 + 0.04),
            (rp, -0.02), (r * 0.95, -0.016), (r * 0.95, 0.016), (rp, 0.02),
            (rp, y1 - 0.06), (r * 1.05, y1 - 0.054), (r * 1.05, y1 - 0.046), (rp, y1 - 0.042),
            (rp, y1 - 0.034), (r * 1.3, y1 - 0.03), (r * 1.35, y1 - 0.004), (r * 1.2, y1), (0, y1)]
    return lathe(prof, seg=12)


part('Store__flamp_base', f_cyl(0.16, 0.18, 0.03), flamp_base)
part('Store__flamp_pole', f_cyl(0.015, 0.015, 1.55), flamp_pole, smooth=50)
part('Store__flamp_shade', f_cyl(0.18, 0.24, 0.3), lambda: lampshade(0.18, 0.24, 0.3, seg=28, bead=0.004), smooth=50)


# ------------------------------------------------------------- bookcase --
def case_side():
    # a side panel: eased front edge, two columns of shelf-pin holes on
    # both faces (the part serves the left and the right side)
    W, H, D = 0.03, 1.95, 0.32
    s = rbox(W, H, D, r=0.003, seg=2)
    holes = Shape()
    for sx in (-1, 1):
        for z in (-0.1, 0.1):
            for k in range(13):
                y = -0.78 + k * 0.13
                holes = holes + cyl(0.0028, 0.0028, 0.016, seg=6, bevel_r=0).rot(z=HALF).move(x=sx * W / 2, y=y, z=z)
    return cut(s, holes)


def case_shelf():
    # a shelf with a rounded front edge and a hint of sag
    W, H, D = 0.78, 0.025, 0.3
    s = softbox(W, H, D, 0.005, n=(8, 1, 1), bottom=True)
    s = s.warp(lambda p: Vector((p.x, p.y - 0.0025 * (1 - (p.x / (W / 2)) ** 2), p.z)))
    return s.fit((-W / 2, -H / 2 - 0.0025, -D / 2), (W / 2, H / 2 - 0.0005, D / 2))


def case_box():
    # a lidded storage box: slightly tapered body, overhanging lid, a finger
    # hole recessed in the front
    body = rect_post([(0.138, 0.118, -0.1), (0.145, 0.125, 0.074)], r=0.012, seg=2)
    lid = rbox(0.3, 0.03, 0.26, r=0.007, seg=2).move(y=0.085)
    hole = prism(rounded_rect(0.085, 0.028, 0.013, 3), 0.05).move(y=0.02, z=0.125)
    body = cut(body, hole)
    return body + lid


part('Store__case_side', f_box(0.03, 1.95, 0.32), case_side)
part('Store__case_back', f_box(0.8, 1.95, 0.01), lambda: rbox(0.8, 1.95, 0.01, r=0.002, seg=1))
part('Store__case_shelf', f_box(0.78, 0.025, 0.3), case_shelf)
part('Store__case_boxes', f_box(0.3, 0.2, 0.26), case_box)


# ------------------------------------------------------------- TV bench --
def tvbench_body():
    # a low cabinet: overhanging top, two drawers either side with bar pulls,
    # an open middle bay with a shelf and a cable hole in the back
    W, H, D = 1.6, 0.42, 0.4
    hw, hh, hd = W / 2, H / 2, D / 2
    fz = hd - 0.02                                    # carcass front (handles reach hd)
    s = rbox(W, 0.025, D, r=0.005, seg=2).move(y=hh - 0.0125)
    ch = H - 0.025
    cy = -hh + ch / 2
    for sx in (-1, 1):
        s = s + rbox(0.02, ch, D - 0.02, r=0.003, seg=1).move(x=sx * (hw - 0.01), y=cy, z=-0.01)
        s = s + rbox(0.018, ch - 0.02, D - 0.03, r=0.002, seg=1).move(x=sx * 0.27, y=cy + 0.01, z=-0.015)
    s = s + rbox(W - 0.04, 0.02, D - 0.03, r=0.003, seg=1).move(y=-hh + 0.01, z=-0.015)
    back = box(W - 0.02, ch, 0.008).move(y=cy, z=-hd + 0.004)
    back = cut(back, prism(rounded_rect(0.12, 0.05, 0.02, 3), 0.05).move(y=0.06, z=-hd + 0.004))
    s = s + back
    s = s + rbox(0.52, 0.018, D - 0.05, r=0.003, seg=1).move(y=-0.02, z=-0.025)
    dh = (ch - 0.02 - 0.006) / 2
    for sx in (-1, 1):
        x = sx * (0.27 + (hw - 0.02 - 0.27) / 2)
        for k in range(2):
            y = -hh + 0.02 + 0.002 + dh / 2 + k * (dh + 0.003)
            s = s + rbox(0.5, dh, 0.018, r=0.0025, seg=2).move(x=x, y=y, z=fz - 0.009)
            s = s + bar_pull(0.14, 0.0045, 0.018, seg=5).move(x=x, y=y + dh * 0.22, z=fz)
    return s


def tvbench_foot():
    # a sled foot: a flat steel bar bent into a rounded loop
    outer = rounded_rect(0.36, 0.05, 0.018, 4)
    inner = rounded_rect(0.336, 0.026, 0.008, 3)
    s = cut(prism(outer, 0.06), prism(inner, 0.1))
    s = bevel(s, 0.0018, seg=1, angle=40)
    return s.rot(y=HALF)


def tv_screen():
    # a slim TV: a thin bezel round a recessed glass, the back swelling into
    # the electronics housing with vent slots, a neck down to the stand
    W, H, T = 1.2, 0.7, 0.05
    front = plate(W, H, [(0, 0.012), (0, 0.022), (0.002, 0.025)], r=0.006, seg=2)
    glass = box(W - 0.024, H - 0.04, 0.006).move(y=0.008, z=0.025 + 0.0015)
    front = cut(front, glass)
    rings = [(rr(W - 0.02, H - 0.02, 0.006, 2), 0.013), (rr(W - 0.1, H - 0.1, 0.03, 2), 0.004),
             (rr(0.9, 0.5, 0.04, 2, cy=-0.03), -0.017), (rr(0.84, 0.44, 0.03, 2, cy=-0.03), -0.025)]
    housing = loft(rings)
    vents = Shape()
    for i in range(10):
        vents = vents + rbox(0.006, 0.12, 0.01, r=0.002, seg=1).move(x=-0.32 + i * 0.014, y=0.08, z=-0.025)
        vents = vents + rbox(0.006, 0.12, 0.01, r=0.002, seg=1).move(x=0.194 + i * 0.014, y=0.08, z=-0.025)
    housing = cut(housing, vents)
    neck = rbox(0.09, 0.05, 0.025, r=0.004, seg=1).move(y=-H / 2 - 0.004, z=-0.012)
    return front + housing + neck


def tv_stand():
    # a pedestal plate with a bevelled top and the socket the neck sits in
    s = ystack([(rr(0.3, 0.2, 0.03, 3), -0.01), (rr(0.3, 0.2, 0.03, 3), 0.002), (rr(0.286, 0.186, 0.024, 3), 0.01)])
    return s + rbox(0.1, 0.01, 0.03, r=0.003, seg=1).move(y=0.011)


part('Store__tvbench_body', f_box(1.6, 0.42, 0.4), tvbench_body)
part('Store__tvbench_foot', f_box(0.06, 0.05, 0.36), tvbench_foot)
part('Store__tv_screen', f_box(1.2, 0.7, 0.05), tv_screen)
part('Store__tv_stand', f_box(0.3, 0.02, 0.2), tv_stand)


# -------------------------------------------------------------- kitchen --
def kitchen_base():
    # a base unit: carcass on a recessed plinth, a drawer over a door, bar
    # pulls, 2 mm gaps between the fronts
    W, H, D = 0.6, 0.86, 0.58
    hh, hd = H / 2, D / 2
    s = rbox(W - 0.002, 0.76, D - 0.02, r=0.002, seg=1).move(y=hh - 0.38, z=-0.01)
    s = s + box(W - 0.002, 0.1, D - 0.1).move(y=-hh + 0.05, z=-0.05)
    fz = hd - 0.0095 - 0.02
    s = s + rbox(W - 0.004, 0.146, 0.019, r=0.0025, seg=2).move(y=hh - 0.002 - 0.073, z=fz)
    s = s + rbox(W - 0.004, 0.604, 0.019, r=0.0025, seg=2).move(y=hh - 0.152 - 0.302, z=fz)
    s = s + bar_pull(0.18, 0.0048, 0.021, seg=6).move(y=hh - 0.075, z=fz + 0.0095)
    s = s + bar_pull(0.18, 0.0048, 0.021, seg=6).move(y=hh - 0.2, z=fz + 0.0095)
    return s


def kitchen_wall():
    # a wall cupboard: carcass, one door, a bar pull near the bottom edge
    W, H, D = 0.6, 0.7, 0.35
    hh, hd = H / 2, D / 2
    s = rbox(W - 0.002, H, D - 0.04, r=0.002, seg=1).move(z=-0.02)
    fz = hd - 0.02 - 0.0095
    s = s + rbox(W - 0.004, H - 0.004, 0.019, r=0.0025, seg=2).move(z=fz)
    s = s + bar_pull(0.18, 0.0048, 0.019, seg=6).move(y=-hh + 0.05, z=fz + 0.0095)
    return s


def kitchen_top():
    # a laminate worktop: post-formed round front edge with a drip groove
    # underneath, square back edge (profile extruded along the run)
    prof = [(-0.31, -0.02), (-0.31, 0.02), (0.297, 0.02), (0.31, 0.009), (0.31, -0.008), (0.302, -0.02),
            (0.292, -0.02), (0.292, -0.016), (0.286, -0.016), (0.286, -0.02)]
    pts = [(-z, y) for z, y in prof]
    s = prism(pts, 3.62).rot(y=HALF)
    return bevel(s, 0.0012, seg=1, angle=40)


def kitchen_sink():
    # a stainless inset sink: flat rim, a rounded bowl with a drain, a
    # ribbed drainer on the right
    s = rbox(0.5, 0.02, 0.4, r=0.004, seg=2)
    bowl = rbox(0.28, 0.05, 0.32, r=0.022, seg=2).move(x=-0.08, y=0.01 + 0.025 - 0.015)
    drain = cyl(0.02, 0.02, 0.01, seg=16, bevel_r=0).move(x=-0.08, y=-0.005)
    ribs = Shape()
    for i in range(6):
        ribs = ribs + box(0.008, 0.006, 0.26).rot(z=math.pi / 4).move(x=0.12 + i * 0.022, y=0.01)
    return cut(s, bowl, drain, ribs)


def kitchen_tap():
    # a pillar mixer: flared base, a slim column, a J spout over the sink,
    # a small lever at the back
    col = lathe([(0, -0.14), (0.015, -0.14), (0.015, -0.133), (0.0115, -0.128), (0.0092, -0.118), (0.0085, -0.1),
                 (0.0085, 0.07), (0.0075, 0.075), (0, 0.075)], seg=12)
    spout = tube([(0, 0.06, 0), (0, 0.118, 0), (0, 0.133, 0.004), (0, 0.13, 0.011), (0, 0.112, 0.013)], 0.0055,
                 seg=8, smooth=1)
    lever = tube([(0, -0.02, 0), (0, -0.01, -0.012)], 0.0028, seg=6) + sphere(0.0038, 8).move(y=-0.01, z=-0.012)
    return col + spout + lever


def kitchen_hob():
    # a glass-ceramic hob: bevelled edge, four etched cooking rings, a row of
    # touch-control marks at the front
    s = rbox(0.58, 0.01, 0.5, r=0.003, seg=2)
    cutters = Shape()
    for x, z, R in ((-0.14, -0.1, 0.095), (0.14, -0.1, 0.075), (-0.14, 0.11, 0.075), (0.14, 0.11, 0.095)):
        ring = cut(cyl(R + 0.0015, R + 0.0015, 0.004, seg=24, bevel_r=0), cyl(R - 0.0015, R - 0.0015, 0.01, seg=24, bevel_r=0))
        cutters = cutters + ring.move(x=x, y=0.005, z=z)
    for i in range(5):
        cutters = cutters + cyl(0.006, 0.006, 0.003, seg=8, bevel_r=0).move(x=-0.06 + i * 0.03, y=0.005, z=0.225)
    return cut(s, cutters)


part('Store__kitchen_base', f_box(0.6, 0.86, 0.58), kitchen_base)
part('Store__kitchen_wall', f_box(0.6, 0.7, 0.35), kitchen_wall)
part('Store__kitchen_top', f_box(3.62, 0.04, 0.62), kitchen_top)
part('Store__kitchen_sink', f_box(0.5, 0.02, 0.4), kitchen_sink)
part('Store__kitchen_tap', f_cyl(0.012, 0.015, 0.28), kitchen_tap, smooth=50)
part('Store__kitchen_hob', f_box(0.58, 0.01, 0.5), kitchen_hob)


# --------------------------------------------------------------- fridge --
def fridge_body():
    # cabinet, a fridge door over a freezer door, a recessed pull on the
    # freezer, hinge caps on the left, a vent grille in the plinth
    W, H, D = 0.68, 1.85, 0.66
    hw, hh, hd = W / 2, H / 2, D / 2
    s = rbox(W, H, D - 0.06, r=0.012, seg=2).move(z=-0.03)
    dz = hd - 0.029
    upper = rbox(W - 0.004, 1.142, 0.058, r=0.014, seg=2).move(y=0.35, z=dz)
    lower_h = 0.618
    lower = rbox(W - 0.004, lower_h, 0.058, r=0.014, seg=2).move(y=-0.536, z=dz)
    lower = cut(lower, rbox(0.22, 0.03, 0.03, r=0.008, seg=2).move(x=0.17, y=-0.536 + lower_h / 2, z=hd))
    s = s + upper + lower
    grille = rbox(W - 0.06, 0.06, 0.03, r=0.004, seg=1).move(y=-hh + 0.035, z=hd - 0.06)
    slots = sum((rbox(0.04, 0.008, 0.02, r=0.002, seg=1).move(x=-0.24 + i * 0.06, y=-hh + 0.035, z=hd - 0.045)
                 for i in range(9)), Shape())
    s = s + cut(grille, slots)
    for y in (hh - 0.004, -0.224):
        s = s + rbox(0.05, 0.012, 0.05, r=0.003, seg=1).move(x=-hw + 0.03, y=y, z=hd - 0.03)
    return s


def fridge_handle():
    # a long D-bar pull: the grip standing off the door on two feet
    prof = [(-0.02, -0.25), (0.02, -0.25), (0.02, 0.25), (-0.02, 0.25), (-0.02, 0.215), (0.004, 0.215),
            (0.004, -0.215), (-0.02, -0.215)]
    prof = round_outline(prof, 0.005, seg=2)
    s = prism([(-z, y) for z, y in prof], 0.03)
    s = bevel(s, 0.004, seg=2, angle=40)
    return s.rot(y=HALF)


part('Store__fridge_body', f_box(0.68, 1.85, 0.66), fridge_body)
part('Store__fridge_handle', f_box(0.03, 0.5, 0.04), fridge_handle)


# --------------------------------------------------------------- dining --
def dtable_top():
    # five glued boards between breadboard ends, eased edges, and an apron
    # under the top between the legs (4 cm, within the thin-axis allowance)
    W, H, D = 1.8, 0.05, 0.9
    hw, hh, hd = W / 2, H / 2, D / 2
    s = rbox(W, H, D, r=0.006, seg=2)
    g = Shape()
    for z in (-0.27, -0.09, 0.09, 0.27):
        g = g + box(W - 0.24, 0.006, 0.006).rot(x=math.pi / 4).move(y=hh, z=z)
    s = cut(s, g)
    ends = Shape()
    for sx in (-1, 1):
        ends = ends + box(0.006, 0.006, D + 0.1).rot(z=math.pi / 4).move(x=sx * (hw - 0.12), y=hh)
    s = cut(s, ends)
    ah = 0.04
    for sz in (-1, 1):
        s = s + rbox(1.58, ah, 0.022, r=0.003, seg=1).move(y=-hh - ah / 2 + 0.002, z=sz * 0.37)
    for sx in (-1, 1):
        s = s + rbox(0.022, ah, 0.7, r=0.003, seg=1).move(x=sx * 0.82, y=-hh - ah / 2 + 0.002)
    return s


def dtable_leg():
    # a turned leg under a square block where the apron joins it
    block = sq_post([(0.0265, 0.245), (0.03, 0.25), (0.03, 0.357), (0.028, 0.36)], r=0.004, seg=1)
    turned = lathe([(0, -0.36), (0.016, -0.36), (0.0185, -0.356), (0.019, -0.348), (0.0165, -0.338),
                    (0.0158, -0.30), (0.0215, 0.02), (0.0245, 0.062), (0.026, 0.074), (0.0245, 0.086),
                    (0.0205, 0.098), (0.0235, 0.14), (0.026, 0.18), (0.0215, 0.214), (0.0205, 0.222),
                    (0.0275, 0.228), (0.0275, 0.238), (0.022, 0.244), (0.022, 0.25), (0, 0.25)], seg=14)
    return block + turned


def dchair_seat():
    # a slightly dished plank seat with a rounded front edge
    s = slab(0.44, 0.44, 0.04, 8, 8).rot(x=-HALF)

    def dish(p):
        if p.y > 0:
            r2 = (p.x / 0.22) ** 2 + ((p.z + 0.03) / 0.2) ** 2
            p.y -= 0.007 * max(0.0, 1 - r2)
        return p
    return bevel(s.warp(dish), 0.008, seg=2, angle=40)


def dchair_back():
    # two stiles, a curved top rail, a lower rail and three curved slats
    W, H, D = 0.44, 0.44, 0.03
    hw, hh = W / 2, H / 2
    sag = 0.008
    curve = lambda p: Vector((p.x, p.y, p.z - sag * (1 - (p.x / (hw - 0.035)) ** 2)))
    s = Shape()
    for sx in (-1, 1):
        s = s + rect_post([(0.0175, 0.015, -hh), (0.0175, 0.015, hh - 0.01), (0.013, 0.012, hh)], r=0.004, seg=1).move(x=sx * (hw - 0.0175))
    iw = W - 0.07 + 0.004
    top = softbox(iw, 0.08, 0.022, 0.006, n=(4, 1, 1)).move(y=hh - 0.05)
    low = softbox(iw, 0.04, 0.02, 0.005, n=(4, 1, 1)).move(y=-hh + 0.05)
    s = s + top.warp(curve) + low.warp(curve)
    for x in (-0.09, 0.0, 0.09):
        slat = rbox(0.045, hh * 2 - 0.16, 0.014, r=0.003, seg=1).move(x=x, y=0.0)
        s = s + slat.warp(curve)
    return s


def dchair_leg():
    return sq_post([(0.012, -0.225), (0.0135, -0.221), (0.014, -0.215), (0.0175, 0.2), (0.0175, 0.225)], r=0.003, seg=1)


part('Store__dtable_top', f_box(1.8, 0.05, 0.9), dtable_top)
part('Store__dtable_leg', f_box(0.06, 0.72, 0.06), dtable_leg)
part('Store__dchair_seat', f_box(0.44, 0.04, 0.44), dchair_seat)
part('Store__dchair_back', f_box(0.44, 0.44, 0.03), dchair_back)
part('Store__dchair_leg', f_box(0.035, 0.45, 0.035), dchair_leg)


# ---------------------------------------------------------------- desk --
def desk_top():
    # an edge-banded board with two cable grommets at the back corners
    W, H, D = 1.4, 0.03, 0.7
    s = rbox(W, H, D, r=0.004, seg=2)
    holes = Shape()
    grom = Shape()
    for sx in (-1, 1):
        holes = holes + cyl(0.03, 0.03, 0.06, seg=16, bevel_r=0).move(x=sx * 0.55, z=-0.26)
        ring = lathe_loop([(0.0255, -0.012), (0.0255, H / 2 + 0.0015), (0.0325, H / 2 + 0.002), (0.034, H / 2 + 0.001),
                           (0.034, H / 2 - 0.0005), (0.0298, H / 2 - 0.0005), (0.0298, -0.012)], seg=16)
        grom = grom + ring.move(x=sx * 0.55, z=-0.26)
    return cut(s, holes) + grom


def desk_leg():
    # a closed steel loop of rectangular tube with levelling feet
    outer = rounded_rect(0.62, 0.71, 0.03, 4)
    inner = rounded_rect(0.54, 0.63, 0.012, 3)
    s = cut(prism([(-z, y) for z, y in outer], 0.05), prism([(-z, y) for z, y in inner], 0.1))
    s = bevel(s, 0.003, seg=1, angle=40).rot(y=HALF).move(y=0.005)
    for sz in (-1, 1):
        s = s + cyl(0.012, 0.014, 0.01, seg=10, bevel_r=0.002).move(y=-0.355, z=sz * 0.25)
    return s


def desk_drawers():
    # a drawer pedestal: carcass, three drawer fronts with bar pulls
    W, H, D = 0.4, 0.5, 0.6
    hh, hd = H / 2, D / 2
    s = rbox(W, H, D - 0.04, r=0.003, seg=1).move(z=-0.02)
    fz = hd - 0.04 + 0.009
    y = hh - 0.002
    for h in (0.11, 0.17, 0.212):
        cy = y - h / 2
        s = s + rbox(W - 0.004, h, 0.018, r=0.0025, seg=2).move(y=cy, z=fz)
        s = s + bar_pull(0.12, 0.0045, 0.019, seg=5).move(y=cy + h * 0.5 - 0.035, z=fz + 0.009)
        y -= h + 0.003
    return s


part('Store__desk_top', f_box(1.4, 0.03, 0.7), desk_top)
part('Store__desk_leg', f_box(0.05, 0.72, 0.62), desk_leg)
part('Store__desk_drawers', f_box(0.4, 0.5, 0.6), desk_drawers)


# --------------------------------------------------------- office chair --
def ochair_base():
    # a five-star base: a hub, five tapering arms, a caster under each tip.
    # The casters hang 4 cm below the frame so the chair meets the floor
    # (the placer floats the base 5.5 cm up); that costs a frame warning.
    s = lathe([(0, -0.02), (0.042, -0.02), (0.046, -0.012), (0.046, 0.016), (0.036, 0.025), (0.03, 0.025), (0, 0.025)], seg=16)
    arm = loft([(rr(0.05, 0.036, 0.008, 1), 0.02), (rr(0.044, 0.032, 0.008, 1, cy=-0.003), 0.18),
                (rr(0.038, 0.026, 0.008, 1, cy=-0.006), 0.262), (rr(0.03, 0.018, 0.006, 1, cy=-0.007), 0.28)])
    wheel = cyl(0.021, 0.021, 0.032, seg=10, bevel_r=0.006, bevel_seg=1).rot(z=HALF)
    hood = rbox(0.04, 0.028, 0.045, r=0.01, seg=1)
    caster = hood.move(y=-0.032) + wheel.move(y=-0.049, z=-0.012)
    for k in range(5):
        a = TAU * k / 5
        unit = arm.copy() + caster.copy().move(z=0.255)
        s = s + unit.rot(y=a)
    return s


def ochair_post():
    # the gas lift: a tapered foot in the hub, a bellows cover, the piston
    y0, y1 = -0.17, 0.17
    prof = [(0, y0), (0.022, y0), (0.025, y0 + 0.05), (0.03, y0 + 0.056)]
    y = y0 + 0.056
    for i in range(3):
        prof += [(0.03, y + 0.008), (0.026, y + 0.016), (0.03, y + 0.024)]
        y += 0.024
    prof += [(0.028, y + 0.006), (0.024, y + 0.012), (0.024, 0.07), (0.018, 0.078), (0.014, 0.08),
             (0.014, y1 - 0.01), (0.012, y1), (0, y1)]
    return lathe(prof, seg=14)


def ochair_seat():
    # a moulded seat: a pad with a dished top and a waterfall front edge,
    # a welt round the top, a plastic shell underneath
    W, D = 0.5, 0.48
    ph = 0.056
    pad = softbox(W, ph, D, 0.025, n=(6, 1, 6), bottom=False)
    hx, hz = W / 2, D / 2

    def fn(p):
        if p.y > 0:
            g = max(0.0, 1 - (p.x / hx) ** 2) * max(0.0, 1 - ((p.z + 0.03) / hz) ** 2)
            p.y -= 0.01 * g
            if p.z > hz - 0.09:
                t = (p.z - (hz - 0.09)) / 0.09
                p.y -= 0.014 * t * t
        return p
    pad = pad.warp(fn)
    a = 0.707
    pad = pad + pipe_rect(W - 0.05 * (1 - a), D - 0.05 * (1 - a), 0.025 * a + 0.003, ph / 2 - 0.025 * (1 - a) - 0.002, 0.004, seg=4)
    pad = pad.move(y=0.04 - ph / 2)
    shell = rbox(W - 0.03, 0.03, D - 0.03, r=0.01, seg=2).move(y=-0.025)
    return (pad + shell).fit((-hx, -0.04, -hz), (hx, 0.04, hz))


def ochair_back():
    # a padded back: rounded outline, a lumbar swell, sides wrapping
    # forward, a welt round the front, and a spine stub down to the seat
    W, H, D = 0.46, 0.55, 0.07
    hx, hy, hz = W / 2, H / 2, D / 2
    s = softbox(W, H, 0.05, 0.022, n=(6, 8, 1), bottom=True)

    def fn(p):
        u, v = p.x / hx, p.y / hy
        k = 0.06
        # round the corners of the outline
        cx, cy = hx - k, hy - k
        if abs(p.x) > cx and abs(p.y) > cy:
            dx, dy = abs(p.x) - cx, abs(p.y) - cy
            L = math.hypot(dx, dy)
            if L > k:
                p.x = math.copysign(cx + dx * k / L, p.x)
                p.y = math.copysign(cy + dy * k / L, p.y)
        p.z += 0.018 * u * u                          # wrap forward at the sides
        if p.z > 0:
            p.z += 0.012 * math.exp(-((v + 0.3) / 0.3) ** 2) * max(0.0, 1 - u * u)
        return p
    s = s.warp(fn)
    spine = rbox(0.07, 0.13, 0.025, r=0.008, seg=2).move(y=-hy - 0.045 + 0.02, z=-0.012)
    return (s + spine).fit((-hx, -hy - 0.045, -hz), (hx, hy, hz))


part('Store__ochair_base', f_cyl(0.3, 0.3, 0.05), ochair_base)
part('Store__ochair_post', f_cyl(0.03, 0.03, 0.34), ochair_post, smooth=50)
part('Store__ochair_seat', f_box(0.5, 0.08, 0.48), ochair_seat, smooth=60)
part('Store__ochair_back', f_box(0.46, 0.55, 0.07), ochair_back, smooth=60)


# ------------------------------------------------------------- bunk bed --
def bunk_post():
    # a square post with a rounded pyramid top and bolt caps where the
    # frames are bolted through
    s = sq_post([(0.027, -0.85), (0.03, -0.846), (0.03, 0.80), (0.028, 0.815), (0.021, 0.836), (0.012, 0.85)], r=0.006, seg=2)
    for y in (-0.55 + 0.03, 0.45 + 0.03):
        for a in range(4):
            s = s + screw_head(0.008, 0.003, slot=False).rot(x=HALF).move(z=0.0295).rot(y=a * HALF).move(y=y)
    return s


def bunk_rail():
    # a guard rail: a top and bottom bar joined at the rounded ends
    outer = rounded_rect(1.4, 0.18, 0.05, 4)
    slot = rounded_rect(1.22, 0.06, 0.03, 4)
    s = cut(prism([(-z, y) for z, y in outer], 0.03), prism([(-z, y + 0.005) for z, y in slot], 0.1))
    return bevel(s, 0.006, seg=2, angle=40).rot(y=HALF)


part('Store__bunk_post', f_box(0.06, 1.7, 0.06), bunk_post)
part('Store__bunk_frame', f_box(1.0, 0.12, 2.06), lambda: bed_base(1.0, 0.12, 2.06, t=0.03, n_slats=14, beam=False))
part('Store__bunk_mattress', f_box(0.9, 0.14, 1.95), lambda: mattress(0.9, 0.14, 1.95, r=0.03), smooth=50, weighted=False)
part('Store__bunk_rail', f_box(0.03, 0.18, 1.4), bunk_rail)


# -------------------------------------------------------------- toy box --
def toybox():
    # a toy chest: rounded body, an overhanging lid on two hinges, a
    # recessed front panel and hand holds in both ends
    W, H, D = 0.55, 0.36, 0.38
    hw, hh, hd = W / 2, H / 2, D / 2
    body = rbox(W - 0.02, 0.3, D - 0.02, r=0.02, seg=2).move(y=-hh + 0.15)
    cuts = [rbox(0.4, 0.17, 0.02, r=0.02, seg=2).move(y=-0.03, z=hd - 0.01 + 0.006)]
    for sx in (-1, 1):
        cuts.append(prism(rounded_rect(0.11, 0.035, 0.016, 3), 0.05).rot(y=HALF).move(x=sx * (hw - 0.01), y=0.04))
    body = cut(body, *cuts)
    lid = rbox(W, 0.055, D, r=0.022, seg=2).move(y=hh - 0.0275)
    hinges = Shape()
    for sx in (-1, 1):
        hinges = hinges + cyl(0.007, 0.007, 0.05, seg=8, bevel_r=0.002).rot(z=HALF).move(x=sx * 0.16, y=hh - 0.06, z=-hd + 0.012)
    return body + lid + hinges


part('Store__toybox', f_box(0.55, 0.36, 0.38), toybox)


# ============================================================== STORAGE ===
def rack_upright():
    # a roll-formed C-channel upright open at the back, two columns of
    # teardrop slots punched through the face, a foot plate with anchors
    L = 3.1
    t = 0.003
    o = [(-0.04, 0.04), (0.04, 0.04), (0.04, -0.04), (0.024, -0.04), (0.024, -0.04 + t), (0.04 - t, -0.04 + t),
         (0.04 - t, 0.04 - t), (-0.04 + t, 0.04 - t), (-0.04 + t, -0.04 + t), (-0.024, -0.04 + t), (-0.024, -0.04),
         (-0.04, -0.04)]
    s = prism(o, L - 0.01).rot(x=HALF).move(y=0.005)
    tear = [(-0.0028, -0.016), (0.0028, -0.016), (0.0068, 0.004), (0.0, 0.013), (-0.0068, 0.004)]
    slots = Shape()
    for sx in (-1, 1):
        for k in range(25):
            y = -L / 2 + 0.2 + k * 0.12
            slots = slots + prism(tear, 0.02).move(x=sx * 0.02, y=y, z=0.04)
    s = cut(s, slots)
    s = s + rbox(0.1, 0.008, 0.1, r=0.002, seg=1).move(y=-L / 2 + 0.004)
    for sx in (-1, 1):
        s = s + cyl(0.007, 0.007, 0.006, seg=6, bevel_r=0.001).move(x=sx * 0.04, y=-L / 2 + 0.011, z=0.035 * sx)
    return s


def rack_beam():
    # a step beam (box section with a ledge for the decking) and welded
    # connector plates with safety pins at both ends
    L = 2.5
    prof = [(-0.025, -0.05), (0.025, -0.05), (0.025, 0.05), (-0.009, 0.05), (-0.009, 0.034), (-0.025, 0.034)]
    prof = round_outline(prof, 0.0025, seg=1)
    s = prism([(-z, y) for z, y in prof], L - 0.1).rot(y=HALF)
    s = bevel(s, 0.001, seg=1, angle=60)
    for sx in (-1, 1):
        s = s + rbox(0.05, 0.13, 0.008, r=0.002, seg=1).move(x=sx * (L / 2 - 0.05), y=-0.005, z=-0.02)
        s = s + rbox(0.006, 0.13, 0.05, r=0.002, seg=1).move(x=sx * (L / 2 - 0.047), y=-0.005)
        s = s + cyl(0.004, 0.004, 0.012, seg=6, bevel_r=0.001).rot(x=HALF).move(x=sx * (L / 2 - 0.035), y=0.04, z=-0.026)
    return s


def rack_carton():
    # a taped shipping carton: two top flaps meeting at a seam, bowing a
    # little; sides bulged from the load; a tape strip over the seam and
    # down both ends
    W, H, D = 0.62, 0.5, 0.78
    hx, hy, hz = W / 2, H / 2, D / 2
    r = 0.008
    xs = sorted(edge_coords(hx, 6, r) + [-0.008, 0.008])
    s = softbox(W, H, D, r, n=(6, 4, 6), puff=(0.012, 0.0, 0.01), bottom=False, xs=xs)

    def fn(p):
        if p.y > hy - 1e-3:
            ax = abs(p.x)
            if ax < 0.009:
                p.y -= 0.004
            else:
                p.y += 0.006 * math.sin(math.pi * min(1.0, (ax - 0.008) / (hx - 0.008)))
        return p
    s = s.warp(fn)
    s = noisy(s, 0.002, scale=5, seed=9)
    tw, tt = 0.05, 0.0015
    tape = box(tw, tt, D + 0.004).move(y=hy + 0.0065)
    for sz in (-1, 1):
        tape = tape + box(tw, 0.08, tt).move(y=hy - 0.04 + 0.006, z=sz * (hz + 0.0045))
    return (s + tape).fit((-hx, -hy, -hz), (hx, hy + 0.007, hz))


part('Store__rack_upright', f_box(0.08, 3.1, 0.08), rack_upright)
part('Store__rack_beam', f_box(2.5, 0.1, 0.05), rack_beam)
part('Store__rack_carton', f_box(0.62, 0.5, 0.78), rack_carton, smooth=50)


# ------------------------------------------------------------- trolley --
def se_loop(hx, hz, n, p=7.0, cz=0.0):
    """n points evenly spaced (by arc length) round a superellipse: a
    rounded rectangle hx x hz (half sizes) centred at z = cz."""
    dense = []
    for i in range(400):
        a = TAU * i / 400
        c, s_ = math.cos(a), math.sin(a)
        dense.append((hx * math.copysign(abs(c) ** (2 / p), c), cz + hz * math.copysign(abs(s_) ** (2 / p), s_)))
    seg = [math.hypot(b[0] - a[0], b[1] - a[1]) for a, b in zip(dense, dense[1:] + dense[:1])]
    total = sum(seg)
    out, acc, j = [], 0.0, 0
    for k in range(n):
        target = total * k / n
        while acc + seg[j] < target:
            acc += seg[j]
            j += 1
        f = (target - acc) / seg[j]
        a, b = dense[j], dense[(j + 1) % 400]
        out.append((a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f))
    return out


def trolley_basket():
    # a wire basket: a heavy top rim, a smaller bottom rim set back from the
    # sloping nose, vertical wires between them, two horizontal wire bands,
    # a bottom grid and the rear gate frame
    top_y, bot_y = 0.2, -0.2
    T = lambda n: [(x, top_y, z) for x, z in se_loop(0.265, 0.415, n)]
    B = lambda n: [(x, bot_y, z) for x, z in se_loop(0.2, 0.325, n, cz=-0.075)]
    s = loop_tube(T(20), 0.008, seg=5) + loop_tube(B(16), 0.005, seg=4)
    nv = 32
    tv, bv = T(nv), B(nv)
    for h in (0.36, 0.7):
        pts = [tuple(b[k] + (t[k] - b[k]) * h for k in range(3)) for t, b in zip(T(16), B(16))]
        s = s + loop_tube(pts, 0.0035, seg=4)
    for t, b in zip(tv, bv):
        s = s + tube([b, t], 0.003, seg=4, caps=False)
    for i in range(5):
        x = -0.16 + i * 0.08
        s = s + tube([(x, bot_y, -0.39), (x, bot_y, 0.24)], 0.003, seg=4, caps=False)
    for i in range(6):
        z = -0.36 + i * 0.12
        s = s + tube([(-0.19, bot_y + 0.002, z), (0.19, bot_y + 0.002, z)], 0.003, seg=4, caps=False)
    gate = [(-0.22, -0.16, -0.428), (0.22, -0.16, -0.428), (0.22, 0.17, -0.428), (-0.22, 0.17, -0.428)]
    s = s + loop_tube(gate, 0.005, seg=4)
    return s.fit((-0.275, -0.21, -0.433), (0.275, 0.21, 0.425))


def trolley_base():
    # the lower tray: a tube frame and a wire grid
    s = loop_tube([(x, 0.0, z) for x, z in se_loop(0.242, 0.367, 16, p=9)], 0.008, seg=5)
    for i in range(6):
        x = -0.2 + i * 0.08
        s = s + tube([(x, 0.0, -0.36), (x, 0.0, 0.36)], 0.0035, seg=4, caps=False)
    for z in (-0.2, 0.0, 0.2):
        s = s + tube([(-0.24, -0.004, z), (0.24, -0.004, z)], 0.0035, seg=4, caps=False)
    return s.fit((-0.25, -0.015, -0.375), (0.25, 0.015, 0.375))


def trolley_handle():
    # a moulded grip bar with swollen end caps
    prof = [(0, -0.28), (0.012, -0.2795), (0.018, -0.2745), (0.02, -0.265), (0.02, -0.242), (0.0165, -0.236),
            (0.0165, 0.236), (0.02, 0.242), (0.02, 0.265), (0.018, 0.2745), (0.012, 0.2795), (0, 0.28)]
    return lathe(prof, seg=12).rot(z=HALF)


def trolley_wheel():
    # a castor wheel: a rounded tyre on a dished hub with a bearing boss
    prof = [(0, -0.012), (0.009, -0.012), (0.011, -0.0135), (0.03, -0.0132), (0.035, -0.015), (0.046, -0.0142),
            (0.05, -0.008), (0.05, 0.008), (0.046, 0.0142), (0.035, 0.015), (0.03, 0.0132), (0.011, 0.0135),
            (0.009, 0.012), (0, 0.012)]
    return lathe(prof, seg=18)


part('Store__trolley_basket', f_box(0.55, 0.42, 0.85), trolley_basket, smooth=80)
part('Store__trolley_base', f_box(0.5, 0.03, 0.75), trolley_base, smooth=80)
part('Store__trolley_handle', f_box(0.56, 0.04, 0.04), trolley_handle, smooth=50)
part('Store__trolley_wheel', f_cyl(0.05, 0.05, 0.03), trolley_wheel, smooth=50)


# ------------------------------------------------------ price-tag stand --
def tag_pole():
    # a thin rod on a threaded collar, topped by a clip that holds the card
    s = lathe([(0, -0.475), (0.009, -0.475), (0.009, -0.463), (0.0065, -0.459), (0.0045, -0.455),
               (0.0045, 0.445), (0.006, 0.448), (0, 0.448)], seg=8)
    clip = rbox(0.03, 0.03, 0.016, r=0.003, seg=1).move(y=0.46, z=0.004)
    clip = cut(clip, box(0.04, 0.02, 0.0055).move(y=0.475, z=0.0075))
    return s + clip


def tag_foot():
    # a square base plate with a bevelled top and a boss for the rod
    s = ystack([(rr(0.14, 0.14, 0.02, 3), -0.006), (rr(0.14, 0.14, 0.02, 3), -0.002),
                (rr(0.132, 0.132, 0.016, 3), 0.002)])
    return s + cyl(0.016, 0.018, 0.004, seg=16, bevel_r=0.001).move(y=0.004)


part('Store__tag_pole', f_cyl(0.006, 0.006, 0.95), tag_pole, smooth=50)
part('Store__tag_foot', f_box(0.14, 0.012, 0.14), tag_foot)


# ------------------------------------------------ department sign, board --
def sign_board():
    # the sign's back panel: a lipped tray the printed face (a plane at
    # z = 0.021) sits in, a routed line on the back, two hanger eyes on top
    W, H, T = 1.86, 0.5, 0.04
    s = plate(W, H, [(0, -T / 2), (0, T / 2 + 0.002), (0.003, T / 2 + 0.005)], r=0.02, seg=2)
    s = cut(s, box(1.806, 0.456, 0.02).move(z=0.018 + 0.01),
            frame_ring(W - 0.05, H - 0.05, W - 0.058, H - 0.058, 0.006, z=-T / 2))
    for sx in (-1, 1):
        s = s + torus(0.011, 0.003, seg=10, tseg=5).move(x=sx * 0.8, y=H / 2 + 0.009)
    return s


def board_post():
    # a steel post on a round weighted foot, a clamp collar under the board
    return lathe([(0, -1.15), (0.076, -1.15), (0.08, -1.146), (0.08, -1.14), (0.07, -1.136), (0.045, -1.13),
                  (0.04, -1.118), (0.026, -1.112), (0.025, -1.1), (0.025, 1.02), (0.034, 1.022), (0.035, 1.03),
                  (0.035, 1.056), (0.034, 1.06), (0.025, 1.062), (0.025, 1.145), (0.022, 1.15), (0, 1.15)], seg=16)


def board_frame():
    # the hours board's case: a rounded moulding standing 5 mm proud of the
    # printed face (a plane at z = 0.031), a socket under it for the post
    W, H, T = 0.84, 0.46, 0.06
    s = plate(W, H, [(0, -T / 2), (0, T / 2 - 0.004), (0.004, T / 2 + 0.004), (0.012, T / 2 + 0.006)], r=0.012, seg=2)
    s = cut(s, rbox(0.768, 0.388, 0.04, r=0.004, seg=1).move(z=0.028 + 0.02))
    s = s + cyl(0.036, 0.036, 0.016, seg=16, bevel_r=0.002).move(y=-H / 2 - 0.006)
    return s


part('Store__sign_board', f_box(1.86, 0.5, 0.04), sign_board)
part('Store__board_post', f_cyl(0.03, 0.03, 2.3), board_post, smooth=50)
part('Store__board_frame', f_box(0.84, 0.46, 0.06), board_frame)


# ================================================================ STAFF ===
# The staff rig (make_staff): hips at 1.06, a 2.27 m tall figure facing +Z.
# Everything is modelled at its true in-game shape; the torso and the head
# are scaled by the rig afterwards (torso z * 0.62, head (0.92, 1.18, 1)),
# so they are built at real size and stretched by the inverse here.
def section(rx, zf, zb, n, p=2.0, y=0.0, disp=None):
    """A horizontal cross-section: a superellipse rx wide, zf deep to the
    front (+Z) and zb to the back; disp(a, y) pushes it out radially.
    a = 0 is the front, a = pi/2 is +X."""
    pts = []
    e = 2.0 / p
    for i in range(n):
        a = TAU * i / n
        s, c = math.sin(a), math.cos(a)
        x = rx * math.copysign(abs(s) ** e, s)
        z = (zf if c >= 0 else zb) * math.copysign(abs(c) ** e, c)
        if disp:
            d = disp(a if a <= math.pi else a - TAU, y)
            L = math.hypot(x, z) or 1.0
            x += x / L * d
            z += z / L * d
        pts.append((x, z))
    return pts


def body(stations, n, p=2.0, disp=None):
    """A limb or trunk: stations [(y, rx, zf, zb[, p]), ...] bottom to top."""
    rings = []
    for st in stations:
        y, rx, zf, zb = st[:4]
        rings.append((section(rx, zf, zb, n, st[4] if len(st) > 4 else p, y, disp), y))
    return ystack(rings)


def ridge(a, y, a0, y0, a1, y1, amp, width, arc=0.15):
    """A soft fold along the segment (a0, y0)-(a1, y1) on a body surface
    (a measured as arc length at radius `arc`)."""
    px, py = a * arc, y
    ax, ay, bx, by = a0 * arc, y0, a1 * arc, y1
    dx, dy = bx - ax, by - ay
    t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)))
    d = math.hypot(px - (ax + dx * t), py - (ay + dy * t))
    taper = math.sin(math.pi * (0.15 + 0.7 * t))
    return amp * taper * math.exp(-(d / width) ** 2)


def interp(stations, y, k):
    for a, b in zip(stations, stations[1:]):
        if a[0] <= y <= b[0]:
            t = (y - a[0]) / (b[0] - a[0])
            return a[k] + (b[k] - a[k]) * t
    return stations[-1][k] if y > stations[-1][0] else stations[0][k]


TORSO = [(-0.44, 0.150, 0.092, 0.088), (-0.438, 0.172, 0.107, 0.103), (-0.43, 0.178, 0.112, 0.108),
         (-0.418, 0.1755, 0.1095, 0.1055), (-0.405, 0.178, 0.112, 0.107), (-0.35, 0.175, 0.109, 0.104),
         (-0.3, 0.171, 0.107, 0.102), (-0.24, 0.167, 0.104, 0.100), (-0.15, 0.166, 0.104, 0.099),
         (-0.05, 0.172, 0.109, 0.101), (0.05, 0.181, 0.116, 0.106), (0.14, 0.188, 0.118, 0.108),
         (0.215, 0.193, 0.115, 0.107, 2.4), (0.27, 0.201, 0.106, 0.103, 2.8), (0.31, 0.2, 0.098, 0.097, 2.8),
         (0.345, 0.182, 0.092, 0.092, 2.5), (0.375, 0.15, 0.087, 0.087), (0.402, 0.118, 0.080, 0.080),
         (0.424, 0.088, 0.070, 0.070), (0.44, 0.062, 0.060, 0.060)]


def torso_disp(a, y):
    d = 0.0
    for sg in (-1, 1):
        d += ridge(a, y, sg * 1.05, 0.17, sg * 0.35, -0.2, 0.006, 0.03)        # drag folds from the armpits
        d -= ridge(a, y, sg * 1.25, 0.12, sg * 0.6, -0.26, 0.0035, 0.028)
        d += ridge(a, y, sg * (math.pi - 1.0), 0.2, sg * (math.pi - 0.35), -0.24, 0.0055, 0.03)
    d += 0.0045 * math.exp(-((y + 0.3) / 0.035) ** 2) * (0.6 + 0.4 * math.cos(2 * a))   # blousing over the hem
    d -= 0.0016 * math.exp(-((abs(a) - math.pi / 2) / 0.1) ** 2) * (1 if y < 0.2 else 0)  # side seams
    return d


def staff_torso():
    # a mustard polo: shoulders sloping to the collar, chest, waist, the
    # shirt blousing over a stitched hem, drag folds from the armpits and on
    # the back, side seams, a buttoned placket under the collar
    s = body(TORSO, 24, p=2.2, disp=torso_disp)
    zf = lambda y: interp(TORSO, y, 2) + torso_disp(0.0, y)
    pl = []
    for y in (0.27, 0.31, 0.35, 0.39, 0.43):
        z = zf(y)
        pl.append(([(-0.017, z - 0.006), (0.017, z - 0.006), (0.017, z + 0.0028), (-0.017, z + 0.0028)], y))
    s = s + ystack(pl)
    for y in (0.315, 0.375):
        slope = (zf(y + 0.01) - zf(y - 0.01)) / 0.02
        btn = on_z(lathe([(0.0058, 0.0), (0.0058, 0.0012), (0.004, 0.0024), (0, 0.0026)], seg=8, cap_bottom=False))
        s = s + btn.rot(x=math.atan(slope)).move(y=y, z=zf(y) + 0.0022)
    return s.scale(1, 1, 1 / 0.62)


def staff_collar():
    # a polo collar: a stand round the neck, the leaf folded over it and
    # down onto the shoulders, lower at the front where the two leaves open
    # in a V over the placket and end in points
    prof = [(0.058, -0.03), (0.056, 0.022), (0.059, 0.032), (0.064, 0.035), (0.069, 0.032), (0.079, 0.012),
            (0.09, -0.012), (0.1, -0.027), (0.098, -0.031), (0.088, -0.022), (0.068, -0.022), (0.062, -0.03)]
    start = [0.1, 0.1, 0.11, 0.11, 0.12, 0.17, 0.25, 0.33, 0.33, 0.3, 0.12, 0.1]
    k = len(prof)
    ncol = 21
    v, f = [], []
    for j in range(ncol):
        for i, (r, y) in enumerate(prof):
            a0 = start[i]
            a = a0 + (TAU - 2 * a0) * j / (ncol - 1)
            front = max(0.0, math.cos(a))
            if 1 <= i <= 4:
                y -= 0.008 * front ** 2
            if 5 <= i <= 9:
                y -= 0.004 * front ** 2
                if i in (7, 8) and (j <= 1 or j >= ncol - 2):
                    y -= 0.005 if j in (0, ncol - 1) else 0.002
            zs = 1 - 0.13 * max(0.0, min(1.0, (r - 0.06) / 0.04))
            v.append((r * math.sin(a), y, r * math.cos(a) * zs))
    for j in range(ncol - 1):
        for i in range(k):
            i2 = (i + 1) % k
            f.append((j * k + i, j * k + i2, (j + 1) * k + i2, (j + 1) * k + i))
    f.append(tuple(range(k)))
    f.append(tuple((ncol - 1) * k + i for i in reversed(range(k))))
    return propkit._fix_winding(Shape(v, f))


def staff_badge():
    # a blank name badge: a rounded plate, a raised rim round an empty window
    s = plate(0.08, 0.04, [(0, -0.006), (0, 0.0035), (0.0015, 0.006)], r=0.005, seg=2)
    return cut(s, rbox(0.066, 0.026, 0.006, r=0.002, seg=1).move(z=0.0065))


def neck_disp(a, y):
    t = (y + 0.07) / 0.14                       # 0 at the collar, 1 under the head
    phi = 0.3 + (1.9 - 0.3) * t                 # the neck muscles run from behind the ears to the sternum
    d = 0.0042 * math.exp(-((abs(a) - phi) / 0.3) ** 2)
    d -= 0.0015 * math.exp(-((abs(a) - phi + 0.55) / 0.2) ** 2)
    d += 0.0022 * math.exp(-(a / 0.28) ** 2) * math.exp(-((y - 0.005) / 0.022) ** 2)
    d -= 0.0022 * math.exp(-((abs(a) - math.pi) / 0.5) ** 2) * t
    return d


def staff_neck():
    # a long neck leaning a little forward: the muscles running down from
    # behind the ears to the collarbones, a faint throat, the trapezius
    # swelling into the back; it runs 1.3 cm on up into the skull
    st = [(-0.07, 0.049, 0.045, 0.051), (-0.05, 0.047, 0.044, 0.049), (-0.03, 0.045, 0.043, 0.046),
          (-0.01, 0.044, 0.042, 0.044), (0.01, 0.043, 0.042, 0.043), (0.03, 0.042, 0.042, 0.042),
          (0.05, 0.042, 0.042, 0.042), (0.07, 0.041, 0.041, 0.041), (0.083, 0.038, 0.038, 0.038)]
    s = body(st, 16, disp=neck_disp)
    return s.warp(lambda p: Vector((p.x, p.y, p.z + 0.007 * (p.y + 0.07) / 0.14)))


def staff_head():
    # a smooth head with no face on it: cranium, brow ridge, shallow eye
    # hollows, cheekbones, a nose-shaped swelling, jaw and chin, all under
    # unbroken skin. Built at real size (about 0.156 x 0.236 x 0.2), then
    # scaled by the inverse of the rig's (0.92, 1.18, 1).
    az = [math.radians(d) for d in (-40, -30, -20, -10, 0, 10, 20, 30, 40, 62, 84, 106, 128, 150, 172, 194, 216, 238,
                                     260, 282, 304)]
    pol = [math.radians(d) for d in (16, 34, 50, 64, 75, 84, 93, 102, 111, 121, 133, 147, 163)]
    g = lambda d, c, s: math.exp(-(1 - d.dot(Vector(c).normalized())) / (s * s))
    clamp = lambda t: max(0.0, min(1.0, t))

    def point(th, ph):
        d = Vector((math.sin(th) * math.sin(ph), math.cos(th), math.sin(th) * math.cos(ph)))
        x, y, z = d.x * 0.078, d.y * 0.112, d.z * 0.092
        k = 0.07 * g(d, (0, 0.55, -0.6), 0.5)                         # fuller back of the skull
        k -= 0.06 * (g(d, (0.8, 0.3, 0.45), 0.25) + g(d, (-0.8, 0.3, 0.45), 0.25))   # temples
        for sx in (-1, 1):
            k += 0.035 * g(d, (sx * 0.72, -0.55, 0.25), 0.22)             # angle of the jaw
        if d.z > 0:
            fz = d.z ** 2
            k += 0.06 * math.exp(-((d.y - 0.27) / 0.1) ** 2) * math.exp(-(d.x / 0.5) ** 2) * fz   # brow ridge
            for sx in (-1, 1):
                k -= 0.065 * g(d, (sx * 0.33, 0.12, 0.94), 0.14)           # eye hollows
                k += 0.05 * g(d, (sx * 0.55, -0.1, 0.83), 0.2)             # cheekbones
            ny = d.y
            nose = (0.035 + 0.11 * clamp((0.1 - ny) / 0.36)) * (1 - clamp((-0.3 - ny) / 0.1))
            k += nose * math.exp(-(d.x / 0.15) ** 2) * (1 if ny < 0.2 else 0.0) * fz
            k -= 0.025 * g(d, (0, -0.47, 0.88), 0.12)                      # where a mouth would be
            k += 0.06 * g(d, (0, -0.72, 0.7), 0.2)                         # chin
        p = Vector((x, y, z)) * (1 + k)
        if p.y < 0:                                                  # the jaw narrows toward the chin
            t = min(1.0, -p.y / 0.1)
            p.x *= 1 - 0.28 * t ** 1.6 * max(0.3, d.z + 0.6)
        if p.y < -0.07:                                              # a flatter underside over the neck
            p.y = -0.07 + (p.y + 0.07) * (0.7 if d.z < 0.35 else 0.85)
        return p
    v = [Vector((0, 0.112 * 1.03, -0.004))]
    for th in pol:
        for ph in az:
            v.append(point(th, ph))
    v.append(point(math.radians(179.5), 0.0))
    n = len(az)
    f = []
    for i in range(n):
        f.append((0, 1 + i, 1 + (i + 1) % n))
    for r in range(len(pol) - 1):
        for i in range(n):
            a, b = 1 + r * n + i, 1 + r * n + (i + 1) % n
            f.append((a, a + n, b + n, b))
    last = len(v) - 1
    base = 1 + (len(pol) - 1) * n
    for i in range(n):
        f.append((last, base + (i + 1) % n, base + i))
    s = propkit._fix_winding(Shape(v, f))
    lo, hi = s.bounds()
    s = s.move(y=-(lo.y + hi.y) / 2, z=-(lo.z + hi.z) / 2).scale(1 / 0.92, 1 / 1.18, 1)
    lo, hi = s.bounds()
    return s.fit((lo.x, -0.1, -0.1), (hi.x, 0.1, 0.1))


def sleeve_disp(a, y):
    d = 0.0
    c = math.cos(a)
    if -0.18 < y < -0.03 and c > 0:                      # folds at the inside of the elbow
        for y0, amp in ((-0.07, 0.003), (-0.1, -0.0022), (-0.13, 0.0032), (-0.16, -0.002)):
            d += amp * math.exp(-((y - y0 - 0.012 * math.sin(a)) / 0.012) ** 2) * c ** 2
    if y > 0.145 and y < 0.16:
        d -= 0.0015                                       # the shoulder seam
    return d


def staff_upperarm():
    # the short sleeve pulled down to the elbow: shoulder cap, a seam, folds
    # in the crook of the arm, a ribbed band and an open hem the forearm
    # comes out of
    st = [(-0.205, 0.044, 0.042, 0.042), (-0.232, 0.045, 0.043, 0.043), (-0.235, 0.05, 0.048, 0.048),
          (-0.232, 0.054, 0.051, 0.051), (-0.19, 0.054, 0.051, 0.051), (-0.184, 0.0505, 0.048, 0.048),
          (-0.16, 0.051, 0.049, 0.049), (-0.13, 0.051, 0.049, 0.049), (-0.1, 0.051, 0.049, 0.049),
          (-0.07, 0.051, 0.049, 0.049), (-0.03, 0.052, 0.05, 0.05), (0.03, 0.053, 0.051, 0.051),
          (0.09, 0.054, 0.052, 0.052), (0.14, 0.055, 0.053, 0.053), (0.152, 0.055, 0.053, 0.053),
          (0.165, 0.055, 0.053, 0.053), (0.192, 0.052, 0.05, 0.05), (0.212, 0.045, 0.044, 0.044),
          (0.226, 0.034, 0.033, 0.033), (0.233, 0.018, 0.018, 0.018)]
    return body(st, 18, disp=sleeve_disp)


def staff_forearm():
    # a bare forearm: round below the elbow, full at the top, flattening and
    # thinning to the wrist, which runs on into the hand
    st = [(-0.235, 0.022, 0.009, 0.009), (-0.232, 0.026, 0.0115, 0.0115), (-0.222, 0.0285, 0.0135, 0.0135),
          (-0.205, 0.0285, 0.016, 0.0165), (-0.18, 0.029, 0.0185, 0.019), (-0.14, 0.031, 0.021, 0.022),
          (-0.08, 0.034, 0.024, 0.026), (0.0, 0.039, 0.029, 0.032), (0.07, 0.043, 0.034, 0.037),
          (0.13, 0.045, 0.037, 0.04), (0.18, 0.043, 0.037, 0.039), (0.208, 0.036, 0.032, 0.033),
          (0.225, 0.024, 0.022, 0.022), (0.234, 0.01, 0.01, 0.01)]
    return body(st, 14, p=2.3)


def staff_hand():
    # a hand hanging with the fingers pressed together (palm toward -Z): a
    # palm growing out of the wrist, four fingers with knuckle swellings
    # curling in a little, the thumb lying along the palm on its fleshy heel.
    # The part serves both hands, so the fingers are symmetric and one thumb
    # is on the wrong side - which suits a mannequin.
    palm = body([(-0.004, 0.026, 0.0055, 0.0055, 3.0), (0.006, 0.032, 0.0075, 0.009, 3.2), (0.02, 0.0335, 0.0095, 0.012, 3.2),
                 (0.04, 0.0315, 0.0125, 0.013, 2.8), (0.058, 0.0285, 0.0128, 0.0128, 2.4), (0.075, 0.025, 0.0125, 0.0125, 2.3),
                 (0.085, 0.022, 0.011, 0.011, 2.3)], 10)
    s = palm
    for x, L, r in ((-0.0232, 0.068, 0.0083), (-0.0078, 0.08, 0.0089), (0.0078, 0.08, 0.0089), (0.0232, 0.068, 0.0083)):
        prof = [(0, 0.0), (r * 0.7, 0.0025), (r * 0.9, 0.01), (r * 0.86, L * 0.3), (r * 0.96, L * 0.55),
                (r * 0.9, L * 0.72), (r * 1.02, L * 0.93), (0, L)]
        fing = bend(lathe(prof, seg=6).scale(1, 1, 0.86).move(y=-L), 0.4, axis='x', along='y')
        lo, hi = fing.bounds()
        top = [p for p in fing.v if p.y > hi.y - 0.004]
        cz = sum(p.z for p in top) / len(top)
        s = s + fing.move(x=x, y=0.016 - hi.y, z=0.0022 - cz)
    thumb = tube([(0.02, 0.042, -0.006), (0.025, 0.022, -0.01), (0.021, 0.0, -0.011)], 0.0088, seg=6, r_end=0.0072)
    s = s + thumb + sphere(0.0072, 6, 4).move(x=0.021, y=0.0, z=-0.011)
    s = s + sphere(1.0, 6, 4).scale(0.013, 0.02, 0.006).move(x=0.014, y=0.034, z=-0.008)   # the heel of the thumb
    return s.fit((-0.035, -0.085, -0.0175), (0.035, 0.085, 0.0175))


def trouser_disp(creases, extra):
    def d(a, y):
        out = 0.0035 * math.exp(-(a / 0.12) ** 2)
        if creases > 1:
            out += 0.0025 * math.exp(-((abs(a) - math.pi) / 0.12) ** 2)
        return out + extra(a, y)
    return d


def thigh_folds(a, y):
    d = 0.0
    for sg in (-1, 1):
        d += ridge(a, y, sg * 1.35, 0.23, sg * 0.45, 0.08, 0.0028, 0.022, arc=0.075)
    if y < -0.15:
        for y0, amp in ((-0.18, 0.0025), (-0.22, -0.002)):
            d += amp * math.exp(-((y - y0) / 0.015) ** 2) * max(0.0, -math.cos(a)) ** 2
    return d


def staff_thigh():
    # the trouser thigh: pressed creases front and back, pull folds from the
    # crotch, wrinkles behind the knee
    st = [(-0.285, 0.012, 0.012, 0.012), (-0.28, 0.03, 0.03, 0.03), (-0.268, 0.045, 0.045, 0.043),
          (-0.25, 0.055, 0.055, 0.052), (-0.22, 0.059, 0.059, 0.056), (-0.18, 0.06, 0.06, 0.056),
          (-0.13, 0.062, 0.061, 0.057), (-0.07, 0.065, 0.063, 0.059), (0.0, 0.068, 0.066, 0.062),
          (0.07, 0.071, 0.068, 0.066), (0.14, 0.074, 0.07, 0.07), (0.2, 0.075, 0.071, 0.073),
          (0.24, 0.07, 0.067, 0.07), (0.262, 0.06, 0.058, 0.06), (0.276, 0.045, 0.044, 0.045),
          (0.284, 0.022, 0.022, 0.022)]
    return body(st, 20, disp=trouser_disp(2, thigh_folds))


def shin_folds(a, y):
    d = 0.0
    c = math.cos(a)
    if y < -0.17:                                         # the break over the shoe
        for y0, amp in ((-0.2, 0.0035), (-0.232, -0.0025), (-0.25, 0.003)):
            d += amp * math.exp(-((y - y0 - 0.01 * math.sin(a)) / 0.01) ** 2) * (0.4 + 0.6 * max(0.0, c))
    return d


def staff_shin():
    # the trouser leg below the knee: creases, the calf behind, a slight
    # break over the shoe and a turned hem
    st = [(-0.27, 0.047, 0.047, 0.045), (-0.27, 0.056, 0.056, 0.054), (-0.262, 0.06, 0.06, 0.058),
          (-0.245, 0.057, 0.058, 0.055), (-0.225, 0.052, 0.051, 0.052), (-0.205, 0.054, 0.054, 0.052),
          (-0.18, 0.051, 0.05, 0.05), (-0.13, 0.05, 0.049, 0.049), (-0.07, 0.051, 0.049, 0.051),
          (0.0, 0.053, 0.05, 0.055), (0.07, 0.055, 0.052, 0.058), (0.14, 0.057, 0.055, 0.058),
          (0.2, 0.059, 0.058, 0.058), (0.23, 0.056, 0.055, 0.055), (0.25, 0.048, 0.048, 0.048),
          (0.262, 0.035, 0.035, 0.035), (0.268, 0.015, 0.015, 0.015)]
    return body(st, 20, disp=trouser_disp(2, shin_folds))


def staff_shoe():
    # a plain black derby: welted sole with a heel, a closed upper rising to
    # the ankle, toe-cap and heel-counter seams, laces over the instep
    y0 = -0.029
    # (z, half width, top height) from the heel to the toe
    st = [(-0.14, 0.026, 0.012), (-0.136, 0.036, 0.026), (-0.124, 0.042, 0.034), (-0.1, 0.045, 0.039),
          (-0.098, 0.0458, 0.0395), (-0.06, 0.047, 0.04), (-0.02, 0.05, 0.034), (0.02, 0.053, 0.024),
          (0.06, 0.054, 0.013), (0.078, 0.0535, 0.009), (0.081, 0.0542, 0.0085), (0.1, 0.051, 0.003),
          (0.118, 0.044, -0.002), (0.131, 0.034, -0.006), (0.139, 0.02, -0.01)]

    def outline(w, top):
        pts = [(w * 0.96, y0), (-w * 0.96, y0)]
        mid = y0 + 0.004
        for i in range(8):
            th = math.pi - math.pi * i / 7
            c, s_ = math.cos(th), math.sin(th)
            pts.append((w * math.copysign(abs(c) ** 0.7, c), mid + (top - mid) * abs(s_) ** 0.8))
        return pts
    upper = loft([(outline(w, top), z) for z, w, top in st])
    foot = [(w + 0.003, z) for z, w, _ in st] + [(-w - 0.003, z) for z, w, _ in reversed(st)]
    sole = ystack([(foot, -0.04), (foot, -0.031), ([(x * 0.985, z * 0.99) for x, z in foot], -0.028)])
    sole = cut(sole, box(0.14, 0.012, 0.07).move(y=-0.04, z=-0.03))
    laces = Shape()
    for z in (-0.045, -0.03, -0.015, 0.0):
        top = interp([(q[0], q[2]) for q in st], z, 1)
        laces = laces + tube([(-0.013, top - 0.001, z), (0.013, top - 0.001, z)], 0.0022, seg=4)
    return upper + sole + laces


part('Staff__torso', f_capsule(0.19, 0.5), staff_torso, smooth=70)
part('Staff__collar', f_cyl(0.075, 0.1, 0.07), staff_collar, smooth=60)
part('Staff__badge', f_box(0.08, 0.04, 0.012), staff_badge)
part('Staff__neck', f_cyl(0.045, 0.05, 0.14), staff_neck, smooth=180)
part('Staff__head', f_sphere(0.1), staff_head, smooth=180)
part('Staff__upperarm', f_capsule(0.055, 0.36), staff_upperarm, smooth=70)
part('Staff__forearm', f_capsule(0.045, 0.38), staff_forearm, smooth=180)
part('Staff__hand', f_box(0.07, 0.17, 0.035), staff_hand, smooth=180)
part('Staff__thigh', f_capsule(0.075, 0.42), staff_thigh, smooth=60)
part('Staff__shin', f_capsule(0.06, 0.42), staff_shin, smooth=60)
part('Staff__shoe', f_box(0.11, 0.08, 0.28), staff_shoe, smooth=50)


# ========================================================= SHADOW FIGURES ===
# make_shadow: flat black bodies with a pale back-face outline, so only the
# silhouette counts: narrow sloping shoulders, a ribcage over a pinched waist,
# long thin arms ending in long-fingered hands, legs with knees. The torso
# (z * 0.6) and head (0.9, 1.15, 1) are scaled by the rig: built at real size
# and stretched by the inverse here.
def shade_torso_disp(a, y):
    d = 0.0
    for sg in (-1, 1):
        d += ridge(a, y, 0.0, 0.12, sg * 1.2, -0.02, 0.004, 0.02, arc=0.12)   # the edge of the ribcage
    return d


def shade_torso():
    # a thin trunk: pelvis, a pinched waist, the ribcage swelling above it,
    # shoulders sloping down from a thin neck that runs up to the head
    st = [(-0.43, 0.09, 0.06, 0.06), (-0.41, 0.135, 0.085, 0.085), (-0.37, 0.152, 0.092, 0.095),
          (-0.3, 0.148, 0.09, 0.092), (-0.2, 0.124, 0.08, 0.075), (-0.1, 0.12, 0.078, 0.072),
          (0.0, 0.14, 0.09, 0.08), (0.08, 0.158, 0.1, 0.088), (0.16, 0.165, 0.102, 0.09),
          (0.24, 0.163, 0.095, 0.088), (0.29, 0.172, 0.082, 0.082, 2.6), (0.335, 0.152, 0.07, 0.072),
          (0.37, 0.118, 0.06, 0.062), (0.395, 0.08, 0.05, 0.05), (0.415, 0.047, 0.04, 0.04),
          (0.435, 0.033, 0.032, 0.032), (0.47, 0.029, 0.029, 0.029), (0.49, 0.026, 0.026, 0.026)]
    return body(st, 16, disp=shade_torso_disp).scale(1, 1, 1 / 0.6)


def shade_head():
    # an egg: a full skull tapering to a narrow jaw and a small chin
    rows = [math.radians(d) for d in (20, 40, 60, 78, 95, 112, 128, 144, 160)]
    n = 14
    v = [Vector((0, 0.135, 0))]
    for th in rows:
        for i in range(n):
            ph = TAU * i / n
            d = Vector((math.sin(th) * math.sin(ph), math.cos(th), math.sin(th) * math.cos(ph)))
            p = Vector((d.x * 0.095, d.y * 0.135, d.z * 0.105))
            if p.y < 0.02:
                t = min(1.0, (0.02 - p.y) / 0.14)
                p.x *= 1 - 0.38 * t ** 1.4
                p.z *= 1 - 0.2 * t ** 1.4 * (1 if d.z < 0 else 0.4)
            if d.z < 0:
                p.z *= 1 + 0.08 * max(0.0, d.y + 0.2)                   # the back of the skull
            if abs(d.x) > 0.5 and -0.55 < d.y < -0.2:
                p.x *= 1.05                                             # the angle of the jaw
            v.append(p)
    v.append(Vector((0, -0.13, 0.02)))
    f = [(0, 1 + i, 1 + (i + 1) % n) for i in range(n)]
    for r in range(len(rows) - 1):
        for i in range(n):
            a, b = 1 + r * n + i, 1 + r * n + (i + 1) % n
            f.append((a, a + n, b + n, b))
    last = len(v) - 1
    base = 1 + (len(rows) - 1) * n
    f += [(last, base + (i + 1) % n, base + i) for i in range(n)]
    s = propkit._fix_winding(Shape(v, f)).scale(1 / 0.9, 1 / 1.15, 1)
    lo, hi = s.bounds()
    return s.fit((lo.x, -0.12, -0.12), (hi.x, 0.12, 0.12))


def shade_arm():
    # a long thin arm: shoulder, a knob of an elbow, a forearm thinning to
    # the wrist, then a narrow hand (edge-on from the front) with four long
    # fingers fanned a little and the thumb pointing forward
    st = [(-0.25, 0.011, 0.012, 0.012), (-0.235, 0.013, 0.016, 0.016), (-0.215, 0.015, 0.017, 0.017),
          (-0.17, 0.021, 0.022, 0.022), (-0.1, 0.027, 0.027, 0.027), (-0.04, 0.029, 0.028, 0.029),
          (-0.01, 0.031, 0.028, 0.032), (0.02, 0.028, 0.027, 0.029), (0.08, 0.03, 0.03, 0.03),
          (0.18, 0.034, 0.033, 0.033), (0.28, 0.038, 0.037, 0.037), (0.34, 0.043, 0.041, 0.041),
          (0.375, 0.042, 0.04, 0.04), (0.397, 0.032, 0.031, 0.031), (0.408, 0.016, 0.016, 0.016)]
    s = body(st, 10)
    s = s + sphere(1.0, 8, 6).scale(0.011, 0.042, 0.03).move(y=-0.283)
    for z0 in (-0.021, -0.007, 0.007, 0.021):
        pts = [(0, -0.3, z0 * 0.9), (0, -0.33, z0 * 1.1), (0.002, -0.37, z0 * 1.3), (0.004, -0.405, z0 * 1.45)]
        s = s + tube(pts, 0.0062, seg=5, r_end=0.0038)
    s = s + tube([(0, -0.265, 0.022), (0, -0.295, 0.04), (0.002, -0.325, 0.048)], 0.0068, seg=5, r_end=0.0045)
    return s


def shade_leg():
    # a thin leg: a full thigh narrowing to a bony knee, a calf, a thin
    # ankle over a narrow, pointed foot
    st = [(-0.45, 0.019, 0.02, 0.02), (-0.42, 0.018, 0.019, 0.019), (-0.36, 0.022, 0.022, 0.024),
          (-0.28, 0.03, 0.029, 0.035), (-0.18, 0.038, 0.035, 0.047), (-0.1, 0.041, 0.037, 0.05),
          (-0.04, 0.038, 0.036, 0.04), (-0.01, 0.043, 0.047, 0.04), (0.02, 0.04, 0.041, 0.039),
          (0.08, 0.044, 0.043, 0.043), (0.18, 0.051, 0.05, 0.05), (0.28, 0.058, 0.056, 0.057),
          (0.38, 0.065, 0.062, 0.063), (0.43, 0.062, 0.06, 0.06), (0.455, 0.052, 0.05, 0.05),
          (0.47, 0.03, 0.03, 0.03), (0.475, 0.01, 0.01, 0.01)]
    s = body(st, 12)
    foot = sphere(1.0, 10, 6).scale(0.026, 0.02, 0.068)
    foot = taper(foot, top=0.35, along='z', axes='xy')
    return s + foot.move(y=-0.49, z=0.02)


part('Shade__torso', f_capsule(0.17, 0.52), shade_torso, smooth=180)
part('Shade__head', f_sphere(0.12), shade_head, smooth=180)
part('Shade__arm', f_capsule(0.05, 0.72), shade_arm, smooth=180)
part('Shade__leg', f_capsule(0.065, 0.82), shade_leg, smooth=180)


# ================================================================ NEST ===
# place_house: the heap of old lamps the shadows guard (brass table lamps,
# iron floor lamps, open fabric shades) and the crates they are piled on.
# Names chosen here; the placer still needs PG wrappers for them.
def nest_lamp_base():
    # a turned brass foot: chamfered edge, beads, a waist and a collar
    return lathe([(0, -0.025), (0.087, -0.025), (0.09, -0.022), (0.09, -0.017), (0.084, -0.014), (0.086, -0.011),
                  (0.074, -0.006), (0.058, 0.001), (0.062, 0.006), (0.048, 0.011), (0.03, 0.016), (0.027, 0.021),
                  (0.028, 0.025), (0, 0.025)], seg=18)


def nest_lamp_stem():
    # a brass baluster stem: a vase-shaped knop low down, ring turnings
    y0, y1 = -0.15, 0.15
    return lathe([(0, y0), (0.011, y0), (0.012, y0 + 0.008), (0.008, y0 + 0.014), (0.012, y0 + 0.045),
                  (0.0125, y0 + 0.065), (0.0085, y0 + 0.09), (0.0065, y0 + 0.1), (0.0075, y0 + 0.104),
                  (0.0065, y0 + 0.108), (0.006, y1 - 0.05), (0.009, y1 - 0.046), (0.006, y1 - 0.042),
                  (0.006, y1 - 0.03), (0.0145, y1 - 0.026), (0.0165, y1 - 0.004), (0.014, y1), (0, y1)], seg=10)


def pleated_shade(rt, rb, h, pleats=24, depth=0.0035):
    """An open pleated shade: the fabric folded into pleats all round,
    bound top and bottom with a plain rolled rim; no caps."""
    y0, y1 = -h / 2, h / 2
    b = 0.004
    lr = lambda y: rb + (rt - rb) * (y - y0) / h - depth
    prof = [(rb - b * 0.9, y0, 0), (rb, y0 + b * 0.6, 0), (rb - 0.001, y0 + b * 1.6, 0.3), (lr(y0 + b * 2.4), y0 + b * 2.4, 1),
            (lr(y1 - b * 2.4), y1 - b * 2.4, 1), (rt - 0.001, y1 - b * 1.6, 0.3), (rt, y1 - b * 0.6, 0), (rt - b * 0.9, y1, 0),
            (rt - b * 1.7, y1 - b * 0.8, 0), (lr(y1 - b * 2.4) - 0.002, y1 - b * 2.4, 1),
            (lr(y0 + b * 2.4) - 0.002, y0 + b * 2.4, 1), (rb - b * 1.7, y0 + b * 0.8, 0)]
    k = len(prof)
    ncol = pleats * 2
    v, f = [], []
    for j in range(ncol):
        a = TAU * j / ncol
        m = depth if j % 2 == 0 else -depth * 0.2
        for r, y, w in prof:
            rr_ = r + m * w
            v.append((rr_ * math.sin(a), y, rr_ * math.cos(a)))
    for j in range(ncol):
        j2 = (j + 1) % ncol
        for i in range(k):
            i2 = (i + 1) % k
            f.append((j * k + i, j * k + i2, j2 * k + i2, j2 * k + i))
    return propkit._fix_winding(Shape(v, f))


def nest_flamp_base():
    # a cast-iron foot: a stepped, moulded disc with a boss for the pole
    return lathe([(0, -0.015), (0.166, -0.015), (0.17, -0.012), (0.17, -0.007), (0.16, -0.005), (0.158, -0.001),
                  (0.13, 0.002), (0.128, 0.006), (0.09, 0.008), (0.05, 0.011), (0.03, 0.012), (0.028, 0.015),
                  (0, 0.015)], seg=28)


def nest_flamp_pole():
    # a twisted square iron rod between turned collars
    y0, y1 = -0.7, 0.7
    rings = []
    for i in range(41):
        y = -0.62 + 1.24 * i / 40
        th = y * 9.0
        c, s_ = math.cos(th), math.sin(th)
        rings.append(([(x * c - z * s_, x * s_ + z * c) for x, z in rr(0.019, 0.019, 0.003, 1)], y))
    rod = ystack(rings)
    col = lambda y: lathe([(0, y - 0.03), (0.012, y - 0.03), (0.014, y - 0.024), (0.014, y - 0.012), (0.012, y - 0.008),
                           (0.0135, y - 0.004), (0.0135, y + 0.004), (0.012, y + 0.008), (0, y + 0.008)], seg=10)
    foot = lathe([(0, y0), (0.014, y0), (0.014, y0 + 0.07), (0.012, y0 + 0.076), (0, y0 + 0.076)], seg=10)
    top = lathe([(0, y1 - 0.08), (0.012, y1 - 0.08), (0.014, y1 - 0.06), (0.018, y1 - 0.03), (0.019, y1 - 0.004),
                 (0.016, y1), (0, y1)], seg=10)
    return rod + col(-0.62 + 0.012) + col(0.62 - 0.004) + foot + top


def nest_crate():
    # a slatted crate: corner posts, three slats a side with gaps between,
    # four lid boards; a little crooked, as if nailed by hand
    W, H, D = 0.5, 0.35, 0.5
    hw, hh, hd = W / 2, H / 2, D / 2
    rnd = random.Random(11)
    t = 0.014
    j = lambda: rnd.uniform(-0.0015, 0.0015)
    board = lambda w, h, d: rbox(w, h, d, r=0.0035, seg=1)
    s = Shape()
    for sx in (-1, 1):
        for sz in (-1, 1):
            s = s + board(0.04, H - t - 0.002, 0.04).move(x=sx * (hw - t - 0.02), y=-t / 2, z=sz * (hd - t - 0.02))
    sh = (H - t - 0.048) / 3
    for k in range(3):
        y = -hh + sh / 2 + k * (sh + 0.024)
        for sz in (-1, 1):
            s = s + board(W, sh, t).rot(z=rnd.uniform(-0.008, 0.008)).move(x=j(), y=y + j(), z=sz * (hd - t / 2))
        for sx in (-1, 1):
            s = s + board(t, sh, D - 2 * t - 0.002).rot(x=rnd.uniform(-0.008, 0.008)).move(x=sx * (hw - t / 2), y=y + j(), z=j())
    lw = (W - 3 * 0.02) / 4
    for k in range(4):
        x = -hw + lw / 2 + k * (lw + 0.02)
        s = s + board(lw, t, D).rot(y=rnd.uniform(-0.01, 0.01)).move(x=x + j(), y=hh - t / 2, z=j())
    return s


part('Nest__lamp_base', f_cyl(0.07, 0.09, 0.05), nest_lamp_base)
part('Nest__lamp_stem', f_cyl(0.012, 0.012, 0.3), nest_lamp_stem, smooth=50)
part('Nest__lamp_shade', f_cyl(0.1, 0.16, 0.2), lambda: pleated_shade(0.1, 0.16, 0.2, pleats=22), smooth=40)
part('Nest__flamp_base', f_cyl(0.15, 0.17, 0.03), nest_flamp_base)
part('Nest__flamp_pole', f_cyl(0.014, 0.014, 1.4), nest_flamp_pole, smooth=40)
part('Nest__flamp_shade', f_cyl(0.16, 0.22, 0.28), lambda: lampshade(0.16, 0.22, 0.28, seg=28, bead=0.004), smooth=50)
part('Nest__crate', f_box(0.5, 0.35, 0.5), nest_crate)
