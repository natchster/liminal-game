"""r09: the props remodelled at their real sizes (docs/design/PROPS.md, P1): the
wooden crate, the broken window sash, the umbrella, and the things put away
(docs/design/SECRETS.md 5). Sources and sizes: docs/research/props-remodelling/crate_sash_umbrella_and_things_put_away.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is modelled
in its prop's own frame: x across its front, y up from the floor, z out of
its front.

The wooden crate (`place_wooden_crates`): a nailed slatted crate of rough
sawn pine, a cube 0.40 m (and two smaller ones, 0.30 and 0.225, for the
one stacked on it), its boards 12 mm (9-10 in the small ones) with gaps
between them, nailed to square corner posts; a slatted lid on cleats;
packed with wood wool. Its bottom at y = 0, centred on x and z.

crateSideA<S>         the four boards of a long side (the full width), on
                      its +z face; turned a half turn for the back
crateSideB<S>         the four boards of a short side (between the long
                      ones), on +z; turned a quarter turn for x
crateSideA/BBroken1/2/12  a side with its second (1) or third (2) board
                      from the floor (or both) snapped: a stub at each end
                      still nailed on, the one end pushed in, the middle gone
crateFrame<S>         the corner posts, the bottom boards and the lid's
                      cleats
crateLid<S>           the lid's boards
crateNails<S>         every nail head: two at each end of each board
crateWool<S>          the wood wool packed inside, seen through the gaps
crateSplinter         a splinter off a broken board, lying on the floor
crateWoolTuft         a tuft of wood wool pulled out through the hole

(<S> is '' for the 0.40 m crate, 'M' for 0.30, 'S' for 0.225.)
"""
from propkit import *
import propkit
import random
import zlib


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, smooth=35.0):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name (the
    shape is built here, at registration)."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget, smooth=smooth)


def rnd(a, b):
    return a + (b - a) * random.random()


# ================================================================ crate ===
# (W outer cube, n boards a side, t board thickness, gap, post section)
CRATES = {'': (0.40, 4, 0.012, 0.020, 0.030), 'M': (0.30, 3, 0.010, 0.018, 0.026), 'S': (0.225, 3, 0.009, 0.014, 0.022)}


def crate_dims(key):
    W, n, t, gap, post = CRATES[key]
    hs = W - t                          # the sides' height under the lid
    bw = (hs - (n - 1) * gap) / n       # a board's width
    return W, n, t, gap, post, hs, bw


def board(L, bw, t, seed, chip=False):
    """A rough sawn board L long (x), bw wide (y), t thick (z), centred: its
    arrises eased, its ends sawn not quite square; now and then a corner
    split off."""
    s = rbox(L, bw * rnd(0.97, 1.0), t * rnd(0.94, 1.0), r=min(0.0016, t * 0.14), seg=1)
    sq = rnd(-0.004, 0.004)            # the end out of square
    s.warp(lambda p: (p.x + (sq * p.y / bw if abs(p.x) > L / 2 - 0.01 else 0.0), p.y, p.z))
    if chip:
        # a corner split off along the grain at one end
        cx = (L / 2) * (1 if random.random() < 0.5 else -1)
        cy = (bw / 2) * (1 if random.random() < 0.5 else -1)
        c = box(rnd(0.03, 0.06), rnd(0.012, 0.02), t * 3).rot(0, 0, rnd(-0.15, 0.15)).move(cx, cy, 0)
        s = cut(s, c)
    return s


def side(key, long_side, broken=()):
    """A side's boards on the crate's +z face (its outside at z = W/2): the
    long side's run the full width; the short side's fit between them. The
    boards in `broken` (counted from the floor) are snapped."""
    W, n, t, gap, post, hs, bw = crate_dims(key)
    L = W if long_side else W - 2 * t
    out = Shape()
    for k in range(n):
        y = k * (bw + gap) + bw / 2
        z = W / 2 - t / 2
        if k in broken:
            out = out + broken_board(L, bw, t, k).move(0, y, z)
            continue
        b = board(L, bw, t, k, chip=(random.random() < 0.25))
        b.rot(0, 0, rnd(-0.006, 0.006)).move(rnd(-0.002, 0.002), y + rnd(-0.0015, 0.0015), z - rnd(0, 0.0008))
        lo, hi = b.bounds()
        if lo.y < 0.0:                  # the bottom board stands on the floor, not in it
            b.move(0, -lo.y, 0)
        out = out + b
    return out


def jagged_end(x0, sgn, bw, n=6, reach=0.05):
    """The splintered end of a snapped board: points across its width from
    -bw/2 to bw/2, each pushed out along the grain by a splinter's length
    (x0 the break, sgn the side the board lies on: +1 a piece running to +x)."""
    pts = []
    for i in range(n + 1):
        y = -bw / 2 + bw * i / n
        if 0 < i < n:
            y += rnd(-0.3, 0.3) * bw / n
        long_ = rnd(0.0, 1.0) ** 2 * reach
        pts.append((x0 - sgn * long_, y))
        if 0 < i < n and random.random() < 0.6:
            pts.append((x0 - sgn * rnd(0.0, 0.25) * reach, y + bw / n * 0.45))
    return pts


def broken_board(L, bw, t, k):
    """A board stove in: the stub at its left end still on its nails, the
    one at its right end snapped and pushed in about its nails, the middle
    gone (on the floor, in splinters)."""
    xl = -L / 2 + L * rnd(0.30, 0.40)       # where the left stub ends
    xr = L / 2 - L * rnd(0.28, 0.36)        # where the right stub starts
    left = [(-L / 2, -bw / 2)] + jagged_end(xl, -1, bw) + [(-L / 2, bw / 2)]
    right = [(L / 2, bw / 2)] + list(reversed(jagged_end(xr, +1, bw))) + [(L / 2, -bw / 2)]
    a = prism(left, t * 0.97)
    b = prism(right, t * 0.97)
    # the right stub bent in about its nails (near its end): its broken end into the crate
    hinge = L / 2 - 0.03
    ang = -rnd(0.16, 0.26)
    b.move(-hinge, 0, 0).rot(0, ang, rnd(-0.04, 0.04)).move(hinge, 0, 0)
    a.rot(0, -rnd(0.0, 0.04), 0)
    return a + b


def crate_frame(key):
    """The corner posts inside the corners, the bottom boards between the
    sides, and the lid's two cleats under it"""
    W, n, t, gap, post, hs, bw = crate_dims(key)
    parts = []
    pc = W / 2 - t - post / 2
    for sx in (-1, 1):
        for sz in (-1, 1):
            p = rbox(post, hs - t - 0.002, post, r=0.002, seg=1)
            parts.append(p.move(sx * pc, t + (hs - t) / 2, sz * pc))
    nb = 4 if key == '' else 3
    inner = W - 2 * t
    bwid = inner / nb - 0.004
    for i in range(nb):
        z = -inner / 2 + (i + 0.5) * inner / nb
        parts.append(rbox(inner - 0.002, t, bwid, r=0.0012, seg=1).move(0, t / 2, z))
    for sx in (-1, 1):
        parts.append(rbox(post * 0.9, post * 0.7, inner - 2 * post - 0.004, r=0.0015, seg=1).move(sx * (inner / 2 - post - 0.03), hs - post * 0.35, 0))
    return sum(parts[1:], parts[0])


def crate_lid(key):
    """The lid: boards across the top, gaps between them, on the cleats"""
    W, n, t, gap, post, hs, bw = crate_dims(key)
    nl = n
    lw = (W - (nl - 1) * gap * 1.1) / nl
    out = Shape()
    for i in range(nl):
        z = -W / 2 + lw / 2 + i * (lw + gap * 1.1)
        b = board(W, lw, t, 10 + i, chip=(random.random() < 0.2)).rot(-math.pi / 2, 0, 0)
        out = out + b.rot(0, rnd(-0.008, 0.008), 0).move(rnd(-0.002, 0.002), hs + t / 2 + rnd(0, 0.0008), z + rnd(-0.002, 0.002))
    return out


def nail_head(r, plain=False):
    """A wire nail's head, driven home (a little proud, a little off true)"""
    h = r * 0.35
    prof = [(r, 0), (0, h)] if plain else [(r, 0), (r * 0.82, h * 0.7), (0, h)]
    s = lathe(prof, seg=5, cap_bottom=False)
    return s.rot(rnd(-0.15, 0.15), rnd(0, 6.3), rnd(-0.15, 0.15)).move(0, -h * rnd(0.1, 0.5), 0)


def on_face(s, x, y, face):
    """s (made facing +y) set on the crate's face `face` (0 +z, 1 +x, 2 -z,
    3 -x, 4 top) at (x along the face, y up), proud of the outside"""
    if face == 4:
        return s.move(x, 0, y)
    s.rot(math.pi / 2, 0, 0)            # +y -> +z
    s.move(x, y, 0)
    return s.rot(0, face * math.pi / 2, 0)


def crate_nails(key):
    W, n, t, gap, post, hs, bw = crate_dims(key)
    r = 0.0032 if key == '' else 0.0027
    out = Shape()
    pc = W / 2 - t - post / 2
    for face in range(4):
        for k in range(n):
            yc = k * (bw + gap) + bw / 2
            for sx in (-1, 1):
                for dy in (-0.27, 0.27):
                    h = nail_head(r, key == 'S')
                    h = on_face(h, sx * pc + rnd(-0.003, 0.003), yc + dy * bw + rnd(-0.003, 0.003), face)
                    out = out + h.move(*((0, 0, W / 2) if face == 0 else (W / 2, 0, 0) if face == 1 else (0, 0, -W / 2) if face == 2 else (-W / 2, 0, 0)))
    # the lid: two at each end of each board, into the short sides' top edges
    nl = n
    lw = (W - (nl - 1) * gap * 1.1) / nl
    for i in range(nl):
        z = -W / 2 + lw / 2 + i * (lw + gap * 1.1)
        for sx in (-1, 1):
            for dz in (-0.25, 0.25):
                h = nail_head(r, key == 'S').move(sx * (W / 2 - t / 2) + rnd(-0.002, 0.002), W, z + dz * lw + rnd(-0.003, 0.003))
                out = out + h
    return out


def heightfield_block(w, d, y0, nx, nz, hfun):
    """A block w (x) by d (z) from y0 up to a top of humps hfun(x, z)"""
    v, f = [], []
    for j in range(nz + 1):
        for i in range(nx + 1):
            x, z = -w / 2 + w * i / nx, -d / 2 + d * j / nz
            v.append((x, hfun(x, z), z))
    for j in range(nz):
        for i in range(nx):
            a = j * (nx + 1) + i
            f.append((a, a + nx + 1, a + nx + 2, a + 1))
    rim = [i for i in range(nx + 1)] + [j * (nx + 1) + nx for j in range(1, nz + 1)] + \
          [nz * (nx + 1) + i for i in range(nx - 1, -1, -1)] + [j * (nx + 1) for j in range(nz - 1, 0, -1)]
    n0 = len(v)
    for k in rim:
        x, _, z = v[k]
        v.append((x, y0, z))
    m = len(rim)
    for k in range(m):
        a, b = rim[k], rim[(k + 1) % m]
        f.append((a, b, n0 + (k + 1) % m, n0 + k))
    f.append(tuple(n0 + k for k in range(m)))
    return propkit._fix_winding(Shape(v, f))


def crate_wool(key):
    """The wood wool packed in: a springy mass filling the crate to a little
    over half, wall to wall, its top a tangle of humps"""
    W, n, t, gap, post, hs, bw = crate_dims(key)
    inner = W - 2 * t - 0.006
    hgt = hs * rnd(0.55, 0.65)
    off = Vector((rnd(0, 50), 0, rnd(0, 50)))

    def h(x, z):
        q = Vector((x, 0, z)) * (14 / W) + off
        e = min(inner / 2 - abs(x), inner / 2 - abs(z))     # it rides up the walls a little
        return t + hgt + noise.noise(q) * hgt * 0.10 + noise.noise(q * 3.1) * hgt * 0.04 + max(0.0, 0.02 - e) * 0.8
    k = 12 if key == '' else 8
    return heightfield_block(inner, inner, t + 0.001, k, k, h)


def splinter():
    """A splinter off the broken board, lying flat: 8 cm, ragged both ends"""
    L, w, th = 0.085, 0.012, 0.004
    pts = [(-L / 2, -w / 2), (-L / 2 + 0.01, -w * 0.35), (L / 2 - 0.02, -w / 2), (L / 2, -w * 0.1), (L / 2 - 0.012, w * 0.15),
           (L / 2 - 0.03, w / 2), (-L / 2 + 0.025, w * 0.4), (-L / 2 + 0.008, w * 0.5)]
    s = prism(pts, th).rot(-math.pi / 2, 0, 0)
    s = bend(s, 0.18, axis='z', along='x')
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


def wool_tuft():
    """A tuft of wood wool pulled out and dropped: a loose, flattened tangle"""
    s = subdivide(box(0.05, 0.018, 0.035), 2)
    s = noisy(s, 0.006, scale=90, seed=11)
    s = noisy(s, 0.002, scale=260, seed=5)
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


for key in CRATES:
    auto(f'Real__crateSideA{key}', lambda key=key: side(key, True), 700)
    auto(f'Real__crateSideB{key}', lambda key=key: side(key, False), 700)
    auto(f'Real__crateFrame{key}', lambda key=key: crate_frame(key), 500)
    auto(f'Real__crateLid{key}', lambda key=key: crate_lid(key), 600)
    auto(f'Real__crateNails{key}', lambda key=key: crate_nails(key), 1600)
    auto(f'Real__crateWool{key}', lambda key=key: crate_wool(key), 900, smooth=80)
for sname, long_side in (('A', True), ('B', False)):
    for tag, br in (('1', (1,)), ('2', (2,)), ('12', (1, 2))):
        auto(f'Real__crateSide{sname}Broken{tag}', lambda long_side=long_side, br=br: side('', long_side, br), 900)
auto('Real__crateSplinter', splinter, 120)
auto('Real__crateWoolTuft', wool_tuft, 400, smooth=80)


# ================================================================= sash ===
# The broken window (`place_broken_window_frames`): the lower sash of an old
# double-hung timber window, taken out and left leaning on a wall, two over
# two lights, its glass broken out but for shards still in the putty. Its
# frame: x across, y up from its bottom rail's foot, z out of its outside
# (putty) face, which is turned to the room; its inside face, moulded, at
# z = -SASH_T.
#
# sashFrame   stiles, rails and glazing bars, the rebates for the glass cut
#             in from the putty face, the inside edges moulded (chamfered),
#             the cord grooves in the stiles' edges
# sashPutty   the putty fillets round each light, cracked, gone in places
# sashCord    the stub of a sash cord still nailed in its groove, frayed
SASH_W, SASH_H, SASH_T = 0.60, 0.70, 0.044
SASH_STILE, SASH_TOP, SASH_BOT, SASH_BAR = 0.050, 0.050, 0.080, 0.022
SASH_REB_W, SASH_REB_D = 0.009, 0.013        # the rebate: past the sight line, and deep
SASH_PUTTY = 0.011                           # the putty's width on the glass


def sash_lights():
    """The four lights' sight openings: (x0, x1, y0, y1)"""
    x0, x1 = -SASH_W / 2 + SASH_STILE, SASH_W / 2 - SASH_STILE
    y0, y1 = SASH_BOT, SASH_H - SASH_TOP
    xm, ym = (x0 + x1) / 2, (y0 + y1) / 2
    hb = SASH_BAR / 2
    return [(x0, xm - hb, y0, ym - hb), (xm + hb, x1, y0, ym - hb), (x0, xm - hb, ym + hb, y1), (xm + hb, x1, ym + hb, y1)]


def frustum(w0, h0, w1, h1, z0, z1):
    """A box whose section goes from w0 x h0 at z0 to w1 x h1 at z1, centred on x, y"""
    v = [(-w0 / 2, -h0 / 2, z0), (w0 / 2, -h0 / 2, z0), (w0 / 2, h0 / 2, z0), (-w0 / 2, h0 / 2, z0),
         (-w1 / 2, -h1 / 2, z1), (w1 / 2, -h1 / 2, z1), (w1 / 2, h1 / 2, z1), (-w1 / 2, h1 / 2, z1)]
    f = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (2, 3, 7, 6), (1, 2, 6, 5), (0, 4, 7, 3)]
    return propkit._fix_winding(Shape(v, f))


def sash_frame():
    T = SASH_T
    s = rbox(SASH_W, SASH_H, T, r=0.0025, seg=1).move(0, SASH_H / 2, -T / 2)
    cutters = []
    for (x0, x1, y0, y1) in sash_lights():
        cx, cy, w, h = (x0 + x1) / 2, (y0 + y1) / 2, x1 - x0, y1 - y0
        # through, at the sight size
        cutters.append(box(w, h, T * 3).move(cx, cy, -T / 2))
        # the rebate from the putty face
        cutters.append(box(w + 2 * SASH_REB_W, h + 2 * SASH_REB_W, SASH_REB_D * 2).move(cx, cy, 0))
        # the moulding on the inside: a chamfer 7 mm on the sight edges
        cutters.append(frustum(w, h, w + 0.014, h + 0.014, -T + 0.007, -T - 0.001).move(cx, cy, 0))
    # the cord grooves down the stiles' edges, 9 mm wide, 12 deep, to 0.5 m
    for sx in (-1, 1):
        cutters.append(box(0.024, 0.50, 0.009).move(sx * SASH_W / 2, 0.05 + 0.25, -T / 2))
    s = cut(s, *cutters)
    # the bottom rail weathered: its outer arris worn round, a dent or two
    s = noisy(s, 0.0004, scale=40, seed=2)
    return s


def sash_putty():
    """Putty fillets in the rebates: a bevel from the face down to the glass,
    hand-tooled, cracked, whole runs fallen out"""
    out = Shape()
    zg = -SASH_REB_D                       # the glass's face
    for li, (x0, x1, y0, y1) in enumerate(sash_lights()):
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        rw, rh = x1 - x0 + 2 * SASH_REB_W, y1 - y0 + 2 * SASH_REB_W
        a = SASH_REB_W + SASH_PUTTY - SASH_REB_W * 0.0
        ring = cut(box(rw - 0.0005, rh - 0.0005, SASH_REB_D - 0.0015).move(0, 0, zg / 2 - 0.0008),
                   frustum(rw - 2 * a, rh - 2 * a, rw - 0.0012, rh - 0.0012, zg - 0.001, 0.0005))
        # runs fallen out: whole lengths of a side, chunks
        gone = []
        for _ in range(random.choice((1, 2, 2, 3))):
            side_ = random.randrange(4)
            L = rnd(0.04, 0.16)
            t = rnd(-0.5, 0.5)
            if side_ < 2:
                gone.append(box(L, a * 2.2, 0.05).move(t * (rw - L), (rh / 2) * (1 if side_ == 0 else -1), 0))
            else:
                gone.append(box(a * 2.2, L, 0.05).move((rw / 2) * (1 if side_ == 2 else -1), t * (rh - L), 0))
        # cracks across it
        for _ in range(6):
            side_ = random.randrange(4)
            t = rnd(-0.45, 0.45)
            if side_ < 2:
                gone.append(box(0.0012, a * 2.2, 0.05).move(t * rw, (rh / 2) * (1 if side_ == 0 else -1), 0))
            else:
                gone.append(box(a * 2.2, 0.0012, 0.05).move((rw / 2) * (1 if side_ == 2 else -1), t * rh, 0))
        ring = cut(ring, *gone)
        ring = noisy(ring, 0.0005, scale=120, seed=li)
        out = out + ring.move(cx, cy, 0)
    return out


def sash_cord():
    """The stub of a sash cord, cut, still nailed in the groove of the
    right stile: down the groove, then hanging free past the rail, frayed"""
    x = SASH_W / 2 - 0.004
    z = -SASH_T / 2
    pts = [(x, 0.46, z), (x, 0.40, z), (x, 0.30, z), (x + 0.004, 0.20, z), (x + 0.010, 0.12, z + 0.004),
           (x + 0.022, 0.06, z + 0.010), (x + 0.040, 0.025, z + 0.020), (x + 0.065, 0.008, z + 0.030)]
    cord = tube(pts, 0.0042, seg=6, smooth=2)
    # its frayed end: the strands come apart
    end = Vector(pts[-1])
    fray = Shape()
    for k in range(5):
        a = k * 1.3
        tip = end + Vector((0.012 + 0.006 * math.cos(a), -0.003 + 0.003 * math.sin(a * 1.7), 0.006 * math.sin(a)))
        fray = fray + tube([end - Vector((0.004, 0, 0)), tip], 0.0013, seg=4, r_end=0.0004)
    return cord + fray


auto('Real__sashFrame', sash_frame, 1600)
auto('Real__sashPutty', sash_putty, 1800)
auto('Real__sashCord', sash_cord, 700)


# ============================================================= umbrella ===
# The umbrella (`place_umbrellas`): a classic stick umbrella, furled and
# strapped, 0.89 m from the ferrule's tip to the top of its crook (Fulton
# Commissioner class: 90 cm closed, 8 ribs of 25 in, a 10 mm shaft, a
# wooden crook). Its frame: y up from the ferrule's tip, its crook curling
# toward +x.
#
# umbrellaCanopy  the furled canopy: its fabric wrapped round the eight ribs
#                 in a spiral, the panels' folds, the hem gathered at the
#                 rib tips; the strap round it and its tab
# umbrellaShaft   the steel shaft, enamelled black, and the runner on it
# umbrellaHandle  the crook, bent wood, varnish worn
# umbrellaMetal   the ferrule and its top cap, the eight tip cups at the
#                 hem, the collar under the crook, the strap's press stud
UMB_TIP, UMB_HEM, UMB_COLLAR = 0.075, 0.700, 0.775   # top cap, hem, crook collar heights
UMB_RIBS = 8


def umb_radius(y):
    """The furled canopy's radius at height y (its folds aside): from the top
    cap a slim spindle, fullest two thirds of the way to the hem"""
    prof = [(UMB_TIP, 0.0075), (0.10, 0.012), (0.16, 0.017), (0.28, 0.022), (0.42, 0.025), (0.50, 0.0255),
            (0.585, 0.0265), (0.64, 0.0255), (0.675, 0.022), (UMB_HEM, 0.013)]
    for (y0, r0), (y1, r1) in zip(prof, prof[1:]):
        if y <= y1:
            t = (y - y0) / (y1 - y0)
            t = t * t * (3 - 2 * t)
            return r0 + (r1 - r0) * t
    return prof[-1][1]


STRAP_Y = 0.43


def umb_canopy():
    """The fabric: rings up the canopy; round each, eight ribs pressing
    out, the panels between them folded in, the whole wound in a spiral so
    each panel overlaps the next; squeezed under the strap"""
    nseg, rings = 24, 26
    ys = [UMB_TIP + (UMB_HEM - UMB_TIP) * (i / (rings - 1)) ** 0.9 for i in range(rings)]
    v, f = [], []
    for y in ys:
        r = umb_radius(y)
        squeeze = 1.0 - 0.25 * max(0.0, 1 - abs(y - STRAP_Y) / 0.03)
        wind = 2.4 * (y - UMB_TIP)          # the spiral of the wrap, radians up the canopy
        for j in range(nseg):
            a = TAU * j / nseg
            ph = ((a + wind) * UMB_RIBS / TAU) % 1.0       # where across a panel
            # a panel: proud at its leading edge (the overlap), sinking to its fold
            fold = 0.16 * (1 - ph) ** 1.6 - 0.07
            # near the hem the folds open out a little
            open_ = 1.0 + 0.10 * max(0.0, (y - 0.6) / 0.1)
            rr = r * (1 + fold * squeeze) * open_ * squeeze
            v.append((rr * math.cos(a), y, rr * math.sin(a)))
    for i in range(rings - 1):
        for j in range(nseg):
            a, b = i * nseg + j, i * nseg + (j + 1) % nseg
            f.append((a, b, b + nseg, a + nseg))
    # closed at the hem round the shaft, and at the top cap
    v.append((0, UMB_HEM, 0)); c_hem = len(v) - 1
    v.append((0, UMB_TIP, 0)); c_tip = len(v) - 1
    for j in range(nseg):
        top = (rings - 1) * nseg
        f.append((top + j, top + (j + 1) % nseg, c_hem))
        f.append(((j + 1) % nseg, j, c_tip))
    s = propkit._fix_winding(Shape(v, f))
    return noisy(s, 0.0006, scale=90, seed=4)


def umb_strap():
    """The strap round the furled canopy: a band of the same cloth, 25 mm,
    and its tab over it"""
    rs = umb_radius(STRAP_Y) * 0.80 + 0.0018
    strap = cyl(rs, rs, 0.024, seg=16, bevel_r=0.0008, bevel_seg=1).move(0, STRAP_Y, 0)
    tab = rbox(0.022, 0.036, 0.003, r=0.0012, seg=1).rot(0, 0, 0.12).move(0, STRAP_Y - 0.004, rs + 0.0012)
    return strap + tab


def umb_shaft():
    shaft = cyl(0.005, 0.005, UMB_COLLAR - 0.06, seg=10, bevel_r=0.0005, bevel_seg=1).move(0, 0.06 + (UMB_COLLAR - 0.06) / 2, 0)
    # the runner, slid down to the hem when it was closed, and the top spring
    runner = lathe([(0.0055, 0), (0.0095, 0.004), (0.0098, 0.030), (0.0080, 0.040), (0.0055, 0.042)], seg=12)
    runner.move(0, UMB_HEM - 0.010, 0)
    spring = rbox(0.004, 0.012, 0.004, r=0.001, seg=1).move(0, UMB_HEM + 0.050, 0.0055)
    return shaft + runner + spring


def umb_handle():
    """The crook: up from the collar, then bent over in a hook 9 cm across,
    its end turned down; 23 mm wood, a little thinner at its end"""
    R, r0 = 0.040, 0.0115
    y0 = UMB_COLLAR + 0.012
    pts = [(0, y0, 0), (0, y0 + 0.03, 0), (0, y0 + 0.05, 0)]
    cx, cy = R, y0 + 0.05
    for k in range(1, 15):
        a = math.pi - k / 14 * (math.pi * 1.12)
        pts.append((cx + R * math.cos(a), cy + R * math.sin(a), 0))
    s = tube(pts, r0, seg=10, smooth=0, r_end=r0 * 0.85)
    # its end rounded off
    end = Vector(pts[-1])
    cap = sphere(r0 * 0.85, seg=10, rings=6).move(end.x, end.y, end.z)
    return s + cap


def umb_metal():
    """The ferrule (8 cm, its tip rounded) and its top cap; the eight tip
    cups gathered at the hem; the collar under the crook; the strap's stud"""
    ferrule = lathe([(0.0, 0.0), (0.0032, 0.0012), (0.0042, 0.006), (0.0055, UMB_TIP - 0.01), (0.0065, UMB_TIP - 0.004), (0.0, UMB_TIP - 0.002)], seg=10)
    topcap = lathe([(0.0068, UMB_TIP - 0.006), (0.0090, UMB_TIP - 0.002), (0.0082, UMB_TIP + 0.006), (0.0, UMB_TIP + 0.010)], seg=12, cap_bottom=True)
    cups = Shape()
    for k in range(UMB_RIBS):
        a = TAU * k / UMB_RIBS + 2.4 * (UMB_HEM - UMB_TIP)
        r = umb_radius(UMB_HEM - 0.004) * 1.05
        cup = cyl(0.0018, 0.0022, 0.008, seg=6, bevel_r=0.0006, bevel_seg=1)
        cups = cups + cup.rot(0, 0, 0.15).move(r * math.cos(a), UMB_HEM + 0.002, r * math.sin(a))
    collar = cyl(0.0100, 0.0095, 0.014, seg=14, bevel_r=0.001, bevel_seg=1).move(0, UMB_COLLAR + 0.006, 0)
    rs = umb_radius(STRAP_Y) * 0.80 + 0.0018
    stud = cyl(0.0042, 0.0042, 0.0025, seg=10, bevel_r=0.0008, bevel_seg=1).rot(math.pi / 2, 0, 0).move(0, STRAP_Y - 0.012, rs + 0.0035)
    return ferrule + topcap + cups + collar + stud


auto('Real__umbrellaCanopy', umb_canopy, 1300, smooth=50)
auto('Real__umbrellaStrap', umb_strap, 300)
auto('Real__umbrellaShaft', umb_shaft, 300)
auto('Real__umbrellaHandle', umb_handle, 600)
auto('Real__umbrellaMetal', umb_metal, 800)


# =============================================================== secrets ===
# The things put away (`secrets/things.rs`, docs/design/SECRETS.md 5): each a
# small ordinary thing, its bottom at y = 0, centred, its front toward +z;
# but for the cap, which only ever hangs on its hook (below).
#
# secretCap           a tweed flat cap hanging on a coat hook by its crown
# secretEnvelope      a red envelope (a hongbao, 9 x 17 cm) lying on a floor
# secretEnvelopeHair  a lock of dark hair caught at its flap, out on the floor
# secretBag           a small cloth bag sewn shut, lying on its back
# secretBagCord       its cord wound round the neck and its loop lying by it
# secretCoin          a brass coin with a hole through it
# secretCoinString    the loop of string through its hole

# The hook (`secrets/dress.rs`, PlaceKind::Hook) and how the cap is hung on
# it: a board on the wall, its peg 8 cm long tilted 0.4 rad up from straight
# out (its tip 87 mm out, 26 mm up from the hook's centre); the cap is set
# 7 cm out and 13 cm under the hook and turned PI/2 - 0.25 about x. The cap
# is modelled hanging, in the hook's space (x along the wall, y up, z out
# of it), and carried back into the frame that pose expects.
HOOK_TIP = Vector((0.0, 0.0256, 0.0868))
HOOK_BASE = Vector((0.0, -0.0056, 0.0132))
CAP_POSE_AT = Vector((0.0, -0.13, 0.07))
CAP_POSE_TURN = math.pi / 2 - 0.25


def smoothstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def cap_worn():
    """A flat cap as worn (y up, z toward the peak): the head band, the
    crown pulled forward over the peak and rolled at its edge, the peak
    a stiff crescent curved down at its sides; closed under by the lining.
    (58 cm: the band 19 x 16 cm; overall 26 x 20 x 6.5 cm.)"""
    ax, az, ch = 0.078, 0.096, -0.012          # the head band's ellipse
    bz, ce = 0.121, 0.006                      # the crown's length, its centre
    nseg = 26
    rings = []
    ts = [0.0, 0.12, 0.25, 0.38, 0.5, 0.58, 0.62, 0.68, 0.75, 0.8, 0.88, 1.0]
    for t in ts:
        ring = []
        for j in range(nseg):
            th = TAU * j / nseg
            sx, cz = math.sin(th), math.cos(th)
            bx = 0.093 + 0.010 * cz                         # wider over the peak than at the back
            fr = max(0.0, cz) ** 3                          # (at the front it lies on the peak)
            ye = 0.027 - 0.010 * cz - 0.010 * fr            # the roll's height
            hb = 0.030 - 0.018 * cz - 0.004 * fr            # the band's top
            e = Vector((bx * sx, ye, ce + bz * cz))
            rim = Vector((ax * sx, 0.0, ch + az * cz))
            if t <= 0.62:
                s = t / 0.62
                xz = Vector((0.0, 0.0, -0.03)).lerp(Vector((e.x, 0.0, e.z)), s)
                # the flat top, higher at the back, its front pulled down to the peak
                ytop = 0.061 - 0.085 * max(0.0, xz.z + 0.03) - 0.9 * max(0.0, xz.z - 0.07) ** 2
                y = ye + (ytop - ye) * (max(0.0, 1 - s ** 10)) ** 0.4
                p = Vector((xz.x, y, xz.z))
                if abs(t - 0.5) < 1e-6:                     # the seam round the top panel
                    p.y -= 0.0025
            elif t <= 0.8:
                s = (t - 0.62) / 0.18                       # under the roll, in to the band's top
                tuck = Vector((rim.x * 1.03, hb, rim.z + 0.004 * cz))
                p = e.lerp(tuck, s ** 0.8)
                p.y = ye + (hb - ye) * s - 0.006 * math.sin(math.pi * s) * (1 - max(0.0, cz))
            else:
                s = (t - 0.8) / 0.2                         # down the band to the rim
                p = Vector((rim.x * 1.03, hb, rim.z + 0.004 * cz)).lerp(rim, s)
                p.y = hb * (1 - s)
            ring.append(p)
        rings.append(ring)
    v, f = [], []
    for ring in rings:
        v += [tuple(p) for p in ring]
    for i in range(len(rings) - 1):
        for j in range(nseg):
            a, b = i * nseg + j, i * nseg + (j + 1) % nseg
            f.append((a, a + nseg, b + nseg, b))
    # the lining across the opening, up inside it
    v.append((0.0, 0.032, ch)); c = len(v) - 1
    base = (len(rings) - 1) * nseg
    for j in range(nseg):
        f.append((base + j, c, base + (j + 1) % nseg))
    crown = weld(propkit._fix_winding(Shape(v, f)), 1e-6)
    # its pleats and the slack of old cloth
    crown = noisy(crown, 0.0012, scale=45, seed=8)
    # the peak: a crescent from the band's front out 7 cm, 4 mm board in cloth
    nx, nz = 8, 2
    peak = slab(1.0, 1.0, 0.004, nx, nz).rot(-math.pi / 2, 0, 0)

    def to_peak(p):
        u, w = p.x, -p.z                         # (-0.5..0.5 across, -0.5..0.5 back to front)
        x = u * 0.150
        zb = ch + az * math.sqrt(max(0.0, 1 - (x / ax) ** 2)) - 0.006
        zf = 0.156 - 0.044 * (x / 0.075) ** 2
        z = zb + (zf - zb) * (w + 0.5)
        y = p.y + 0.003 - 0.011 * (x / 0.075) ** 2 - 0.004 * (w + 0.5)
        return (x, y, z)
    peak.warp(to_peak)
    return crown, peak


def cap_hung(s):
    """The cap (its crown or its peak) hung on the hook by the inside of its
    crown: the peg in it near the back, the crown draped over its tip; the
    cap hangs from there, its opening to the wall, its peak down and its
    foot against the wall, the cloth settling the further it is from the
    peg; then carried into the frame the hook's pose expects"""
    P = Vector((0.0, 0.050, -0.072))             # where the peg's tip bears inside the crown
    H = HOOK_TIP + Vector((0.0, 0.0085, -0.002))
    out = []
    for p in s.v:
        d = p - P
        phi = math.radians(90.0 - 4.0 * smoothstep(-0.06, 0.16, p.z))
        cy, sy = math.cos(phi), math.sin(phi)
        q = Vector((d.x, d.y * cy - d.z * sy, d.y * sy + d.z * cy)) + H
        # the weight: what hangs far below the peg settles in and down
        drop = max(0.0, H.y - q.y)
        q.y -= 0.03 * drop * drop / 0.25
        # what is above the peg folds back over it toward the wall, a crease
        # across the crown where it bends
        r = q - H
        a = -0.55 * smoothstep(-0.004, 0.045, r.y)
        ca, sa = math.cos(a), math.sin(a)
        q = H + Vector((r.x, r.y * ca - r.z * sa, r.y * sa + r.z * ca))
        q.z -= 0.0045 * math.exp(-((q.y - (H.y - 0.004)) / 0.007) ** 2) * smoothstep(0.03, 0.06, q.z)
        # and it hangs a little turned and tipped, as nothing hangs true
        r = q - H
        ct, st = math.cos(0.13), math.sin(0.13)
        r = Vector((r.x * ct + r.z * st, r.y, -r.x * st + r.z * ct))
        cr_, sr_ = math.cos(0.06), math.sin(0.06)
        r = Vector((r.x * cr_ - r.y * sr_, r.x * sr_ + r.y * cr_, r.z))
        out.append(H + r)
    s.v = out
    # draped over the peg: nothing within the peg (its 6 mm and the cloth)
    ax_ = (HOOK_TIP - HOOK_BASE).normalized()
    L = (HOOK_TIP - HOOK_BASE).length
    for i, q in enumerate(s.v):
        t = max(0.0, min(L, (q - HOOK_BASE).dot(ax_)))
        c = HOOK_BASE + ax_ * t
        r = q - c
        if r.length < 0.0095 and t > 0.01:
            s.v[i] = c + (r.normalized() if r.length > 1e-6 else Vector((0, 1, 0))) * 0.0095
        # and nothing through the wall: pressed flat against it
        if s.v[i].z < 0.003:
            s.v[i].z = 0.003
    # into the frame the pose expects
    cr, sr = math.cos(-CAP_POSE_TURN), math.sin(-CAP_POSE_TURN)
    for i, q in enumerate(s.v):
        d = q - CAP_POSE_AT
        s.v[i] = Vector((d.x, d.y * cr - d.z * sr, d.y * sr + d.z * cr))
    return s


def envelope():
    """A red envelope, 17 x 9 cm, two layers of paper and what is in it,
    lying on its face's back: a little bowed, its corners lifting"""
    L, W = 0.170, 0.090
    s = slab(L, W, 0.0016, 10, 5).rot(-math.pi / 2, 0, 0)
    def shape(p):
        u, w = p.x / L * 2, p.z / W * 2
        bulge = 0.0012 * (1 - u * u) * (1 - w * w)         # something inside
        curl = 0.0025 * (abs(u) ** 3) + 0.0018 * (w ** 4) * (1 if p.x > 0 else 0.4)
        return (p.x, p.y + 0.0008 + (bulge if p.y > 0 else 0.0) + curl, p.z)
    s.warp(shape)
    # the flap's edge on the back, felt through the paper as a step near its end
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


def envelope_hair():
    """A lock of dark hair caught at the flap: strands from inside the end
    of it out over its edge and onto the floor, curling"""
    out_v, out_f = [], []
    lock_bend = rnd(0.2, 0.6)
    for k in range(20):
        z0 = rnd(-0.008, 0.010)
        length = rnd(0.06, 0.10)
        bend_ = lock_bend + rnd(-0.25, 0.25)
        pts = []
        n = 8
        for i in range(n + 1):
            t = i / n
            a = bend_ * t * t + rnd(-0.015, 0.015)
            x = 0.072 + t * length * math.cos(a * 0.8)
            z = z0 + t * length * math.sin(a * 0.8) + 0.004 * math.sin(t * 9 + k)
            y = 0.0032 if x < 0.085 else max(0.0004, 0.0032 - (x - 0.085) * 0.25)
            pts.append(Vector((x, y, z)))
        wdt = 0.00028
        base = len(out_v)
        for i, p in enumerate(pts):
            d = (pts[min(i + 1, n)] - pts[max(i - 1, 0)]).normalized()
            side_ = Vector((-d.z, 0, d.x)) * wdt * (1 - 0.5 * i / n)
            out_v += [tuple(p - side_), tuple(p + side_)]
        for i in range(n):
            a = base + 2 * i
            out_f.append((a, a + 2, a + 3, a + 1))
    return Shape(out_v, out_f)


def bag():
    """A small cloth bag, 6 x 7.5 cm, stuffed and sewn shut, lying on its
    back: its mouth gathered into a neck and bound, the gathered cloth
    above the binding ruffled; creased where it was handled"""
    prof = [(0.0, -0.036), (0.016, -0.0352), (0.025, -0.031), (0.0295, -0.022), (0.030, 0.008), (0.027, 0.022),
            (0.019, 0.031), (0.0105, 0.038), (0.0080, 0.041), (0.0078, 0.044), (0.012, 0.048), (0.017, 0.053), (0.0, 0.0545)]
    s = lathe([(r, y) for r, y in prof], seg=14).rot(math.pi / 2, 0, 0)     # its axis along +z
    # flattened by lying (and flatter at the bottom), the ruffle at its mouth
    def lie(p):
        x, y, z = p.x, p.y, p.z
        y *= 0.45 if y > 0 else 0.30
        a = math.atan2(y, x)
        if z > 0.045:
            k = 1 + 0.25 * math.sin(a * 7 + z * 300)
            x, y = x * k, y * k
        elif z > 0.016:                  # gathered into the neck: pleats
            k = 1 + 0.10 * math.sin(a * 9) * smoothstep(0.016, 0.034, z)
            x, y = x * k, y * k
        return (x * 1.0, y, z)
    s.warp(lie)
    s = noisy(s, 0.0020, scale=50, seed=6)
    s = noisy(s, 0.0008, scale=140, seed=9)
    lo, hi = s.bounds()
    return s.move(0, -lo.y, 0)


def bag_cord():
    """The cord, 3 mm: three turns round the neck, knotted, its ends tied
    into a loop for the neck, lying loose beside the bag"""
    lo = bag().bounds()[0]
    yc = 0.0135 * 0.45 + 0.004            # the neck's height above the surface (flattened)
    pts = []
    for i in range(3 * 8 + 1):
        a = TAU * i / 8
        pts.append((0.0098 * math.cos(a), yc + 0.0062 * math.sin(a), 0.0415 + 0.0012 * i / 8))
    helix = tube(pts, 0.0016, seg=4)
    knot = sphere(0.0034, seg=8, rings=5).scale(1.0, 0.8, 1.2).move(0.011, yc, 0.045)
    # the loop: out from the knot, round in a loose oval on the surface, back
    loop = [(0.011, yc, 0.045), (0.025, 0.004, 0.058), (0.050, 0.0016, 0.070), (0.085, 0.0016, 0.062),
            (0.105, 0.0016, 0.030), (0.098, 0.0016, -0.010), (0.072, 0.0016, -0.030), (0.048, 0.0016, -0.018),
            (0.040, 0.0016, 0.010), (0.034, 0.003, 0.034), (0.014, yc, 0.046)]
    cord = tube(loop, 0.0016, seg=5, smooth=1)
    return helix + knot + cord


def coin():
    """A brass coin, 22 mm, a hole of 5 mm through it, its rim raised and
    its faces worn; resting a little tipped on its string"""
    prof = [(0.0026, 0.0), (0.0110, 0.0), (0.0110, 0.0015), (0.0096, 0.0015), (0.0093, 0.0012), (0.0026, 0.0012), (0.0026, 0.0)]
    s = weld(lathe(prof, seg=12, cap_bottom=False, cap_top=False), 1e-6)
    return s.rot(0.06, 0, 0.04).move(0, 0.0014, 0)


def coin_string():
    """The string, 1.5 mm: through the hole and round the coin's edge, its
    ends knotted, the loop lying in a loose curve beside it"""
    loop = [(0.0, 0.0018, 0.0), (0.0, 0.0034, 0.006), (0.0, 0.0032, 0.0115), (0.0, 0.0012, 0.015), (0.004, 0.0008, 0.022),
            (0.014, 0.0008, 0.034), (0.034, 0.0008, 0.044), (0.060, 0.0008, 0.040), (0.074, 0.0008, 0.018),
            (0.070, 0.0008, -0.008), (0.050, 0.0008, -0.024), (0.026, 0.0008, -0.026), (0.010, 0.0008, -0.018),
            (0.002, 0.0006, -0.010), (0.0, 0.0006, -0.002), (0.0, 0.0018, 0.0)]
    st = tube(loop, 0.0008, seg=4, smooth=1)
    knot = sphere(0.0019, seg=6, rings=4).scale(1.2, 0.8, 1.0).move(0.018, 0.0012, -0.0235)
    return st + knot


auto('Real__secretCap', lambda: cap_hung(cap_worn()[0]), 700, smooth=60)
auto('Real__secretCapPeak', lambda: cap_hung(bevel(cap_worn()[1], 0.0012, seg=1)), 400, smooth=40)
auto('Real__secretEnvelope', envelope, 300)
auto('Real__secretEnvelopeHair', envelope_hair, 700)
auto('Real__secretBag', bag, 1200, smooth=60)
auto('Real__secretBagCord', bag_cord, 1200)
auto('Real__secretCoin', coin, 300)
auto('Real__secretCoinString', coin_string, 900)
