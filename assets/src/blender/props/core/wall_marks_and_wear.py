"""r11b: marks and wear on the rooms' walls (docs/design/PROPS.md, P2), after r07:
peeled paint, peeling wallpaper, rust streaks, mould and a leaking pipe.
The marks themselves are painted in the game (soft, uneven textures on
transparent planes laid on the wall: `docs/research/props-remodelling/wall_marks_and_wear.md`); what is modelled here
is what stands off the wall: the curled flakes of paint, the wallpaper come
away at its seam, the rusted fixing a streak runs from, and the leak's
crust and drop (the pipe itself is batch r06's: its elbows, coupling and
collars). Sources and sizes: docs/research/props-remodelling/wall_marks_and_wear.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). A wall part is in the
wall's frame: x along the wall, y up, z out of the wall (z = 0 the wall's
face).

peeledPaintFlakeA..C  a flake of the paint film (0.35 mm: three or four
                    coats) come away from the wall and curled: its foot still
                    on the wall along y = 0, the rest lifted off toward +y and
                    rolled over toward +z, its ragged free edge; ...Top is the
                    film's painted face, ...Under its back and its broken
                    edge (the game gives the face the wall's own surface)
peelingWallpaperCurlR/L  the corner of a drop of wallpaper come away at its
                    seam (x = 0 the seam; R runs to +x, L to -x): its foot
                    still pasted along y = 0, the rest (120 x 190 mm) lifted,
                    turned over toward +z and hanging, its far side and its
                    end torn; ...Front the printed face, ...Back the paper's
                    back and its torn edges
rustStreakAnchor    a rusted M12 stud anchor left in the wall: its washer
                    (24 mm), a hex nut (18 mm across flats), the threaded
                    stud standing 14 mm proud of the nut, scaled with rust
rustStreakBracket   what is left of a shelf: a pressed steel corner brace
                    (25 mm wide, 50 x 50 mm, 2.5 mm) on two pan-head screws,
                    its arm out from the wall, the shelf's screw gone
pipeLeakCrust       what a slow leak at a threaded joint leaves under it: lime
                    and rust in a lumpy crust, two little stalactites
pipeLeakDrop        a drop of water hanging from it (5.6 mm), its top at y = 0
"""
import math
import random

from propkit import *
import propkit

FILM_T = 0.00035          # a paint film of three or four coats
PAPER_T = 0.0003          # a wallpaper (paper, about 150 g/m2) and its paste


# ------------------------------------------------------------- sheets ---
def sheet_parts(flat, warp, nx, ny, t):
    """A thin sheet over an (nx x ny) grid: `flat(u, v)` -> (x, y) its flat
    outline (u across, v along), `warp(x, y, z)` -> Vector where a point of
    the flat sheet at height z (0 its back, t its face) goes. Returns
    (face, back): the face (+z before warping) alone, and the back with the
    sheet's edge all round it."""
    grid = [[flat(i / nx, j / ny) for i in range(nx + 1)] for j in range(ny + 1)]
    face_v = [warp(x, y, t) for row in grid for (x, y) in row]
    back_v = [warp(x, y, 0.0) for row in grid for (x, y) in row]
    k = nx + 1
    face_f = [(j * k + i, j * k + i + 1, (j + 1) * k + i + 1, (j + 1) * k + i) for j in range(ny) for i in range(nx)]
    back_f = [tuple(reversed(q)) for q in face_f]
    # the edge: the outline anticlockwise (seen from +z), back to face
    ring = [(0, i) for i in range(nx)] + [(j, nx) for j in range(ny)] + \
           [(ny, i) for i in range(nx, 0, -1)] + [(j, 0) for j in range(ny, 0, -1)]
    n = len(back_v)
    edge_v = [face_v[j * k + i] for (j, i) in ring]
    verts = back_v + edge_v
    edge_f = []
    for a in range(len(ring)):
        b = (a + 1) % len(ring)
        pa = ring[a][0] * k + ring[a][1]
        pb = ring[b][0] * k + ring[b][1]
        edge_f.append((pa, pb, n + b, n + a))
    return Shape(face_v, face_f), Shape(verts, back_f + edge_f)


def ragged(rnd, n, amp):
    """`n + 1` offsets along a torn edge: a few larger bites and fine teeth."""
    big = [rnd.uniform(-1, 1) for _ in range(5)]
    out = []
    for i in range(n + 1):
        t = i / n * 4
        a, f = int(t), t - int(t)
        f = f * f * (3 - 2 * f)
        out.append(amp * (big[a] * (1 - f) + big[min(a + 1, 4)] * f + rnd.uniform(-0.45, 0.45)))
    return out


# ------------------------------------------------------- peeled paint ---
def paint_flake(seed, w, length, r, cone, cup):
    """A flake `w` wide whose free part (`length` long) has lifted off the
    wall and turned toward +z with radius `r` (growing by `cone` across it,
    so one corner turns tighter), cupped (its sides `cup` higher), its foot
    (3 mm) still on the wall."""
    rnd = random.Random(seed)
    nx, ny = 6, 12
    foot = 0.003
    side_l, side_r = ragged(rnd, ny, w * 0.07), ragged(rnd, ny, w * 0.07)
    tip = ragged(rnd, nx, length * 0.12)
    taper = rnd.uniform(0.15, 0.35)
    wav = [rnd.uniform(0, TAU) for _ in range(3)]

    def flat(u, v):
        y = -foot + v * (foot + length + tip[round(u * nx)] * v)
        half = w / 2 * (1 - taper * max(0.0, y / length) ** 1.5)
        j = round(v * ny)
        x0, x1 = -half + side_l[j], half + side_r[j]
        return (x0 + (x1 - x0) * u, y)

    def warp(x, y, z):
        if y <= 0:
            return Vector((x, y, z))
        rr = r * (1 + cone * x / w)
        # the film is not quite even: a wrinkle along it, and cupped, its
        # sides lifting more than its middle as it dried and shrank
        z = z + 0.00025 * math.sin(x / w * 7 + wav[0]) * min(1.0, y / length * 2)
        z = z + cup * (2 * x / w) ** 2 * (y / length)
        th = y / rr
        return Vector((x, (rr - z) * math.sin(th), rr - (rr - z) * math.cos(th)))

    return sheet_parts(flat, warp, nx, ny, FILM_T)


FLAKES = {
    #     seed  width  free   radius cone   cup
    'A': (1101, 0.030, 0.024, 0.016, 0.35, 0.0015),
    'B': (1102, 0.020, 0.016, 0.0095, -0.3, 0.001),
    'C': (1103, 0.040, 0.030, 0.0135, 0.45, 0.002),
}


# ------------------------------------------------- peeling wallpaper ---
CURL_W, CURL_L, CURL_FOOT = 0.120, 0.190, 0.008


def wallpaper_curl(mirror):
    """The corner of a drop come away at its seam (x = 0): pasted still
    along its foot (y < 0), the rest lifted off the wall, over toward +z and
    hanging down in front of where it was, its far side and end torn."""
    rnd = random.Random(1201)
    nx, ny = 8, 24
    far = ragged(rnd, ny, 0.006)
    end = ragged(rnd, nx, 0.010)
    # the arc it turns through, per unit length, along it: a tight turn as
    # it leaves the wall, then a slower one, its end rolled back under
    steps = 120
    reach = CURL_L + 0.02      # the path runs on past the longest tongue of the torn end

    def phi_rate(s, x):
        k = 1.0 / (0.033 + 0.008 * x / CURL_W)       # the corner further out rolls looser
        if s < 0.095:
            return k
        # then it goes on rolling, less and less, its end turned back under
        return k * max(0.0, 0.35 - 0.25 * min(1.0, (s - 0.095) / 0.1))

    # a path (y, z, phi) per column of the grid, integrated along s
    def path(x):
        pts = [(0.0, 0.0, 0.18)]
        ds = reach / steps
        for q in range(steps):
            y, z, ph = pts[-1]
            ph2 = ph + phi_rate(q * ds, x) * ds
            pm = (ph + ph2) / 2
            pts.append((y + math.cos(pm) * ds, z + math.sin(pm) * ds, ph2))
        return pts

    cols = {}

    def flat(u, v):
        j = round(v * ny)
        x = u * (CURL_W + far[j] * u)
        y = -CURL_FOOT + v * (CURL_FOOT + CURL_L + end[round(u * nx)] * v)
        return (x, y)

    def warp(x, y, z):
        if y <= 0:
            p = Vector((x, y, z))
        else:
            key = round(x, 5)
            if key not in cols:
                cols[key] = path(x)
            pts = cols[key]
            f = min(y / reach * steps, steps - 1e-6)
            a = int(f)
            t = f - a
            (y0, z0, p0), (y1, z1, p1) = pts[a], pts[a + 1]
            py, pz, ph = y0 + (y1 - y0) * t, z0 + (z1 - z0) * t, p0 + (p1 - p0) * t
            # the face is on the inside of the turn: off the path along its normal
            p = Vector((x, py - math.sin(ph) * z, pz + math.cos(ph) * z))
            # paper is never quite flat: a soft cockle across it
            p.z += 0.0012 * math.sin(x / CURL_W * 5.5 + 0.7) * min(1.0, y / 0.05)
        if mirror:
            p.x = -p.x
        return p

    face, back = sheet_parts(flat, warp, nx, ny, PAPER_T)
    if mirror:
        face.f = [tuple(reversed(f)) for f in face.f]
        back.f = [tuple(reversed(f)) for f in back.f]
    return face, back


# ------------------------------------------------------- rust fixings ---
def rust_anchor():
    """An M12 stud anchor: washer (ISO 7089: 24 mm, 2.5 mm), hex nut (ISO
    4032: 18 mm across flats, 10.8 mm), the stud 14 mm proud of it, the
    whole scaled and pitted with rust. Axis along z, its back on the wall."""
    washer = cyl(0.012, h=0.0025, seg=20, bevel_r=0.0004).rot(math.pi / 2, 0, 0).move(z=0.00125)
    nut = cyl(0.0104, h=0.0108, seg=6, bevel_r=0.0009, bevel_seg=1).rot(math.pi / 2, 0, 0).rot(0, 0, math.pi / 6).move(z=0.0025 + 0.0054)
    # the thread: a lathe of teeth at a 1.75 mm pitch, its end chamfered
    prof = [(0.0, 0.0133 + 0.0144)]
    prof.append((0.0045, 0.0133 + 0.0144))
    prof.append((0.0058, 0.0133 + 0.0132))
    y = 0.0133 + 0.0132
    tooth = 0
    while y > 0.0128:
        prof.append((0.0060 if tooth % 2 == 0 else 0.0051, y))
        y -= 0.000875
        tooth += 1
    prof.append((0.0055, 0.0128))
    prof.append((0.0, 0.0128))
    prof.reverse()
    stud = lathe(prof, seg=12).rot(math.pi / 2, 0, 0)
    s = washer + nut + stud
    # rust: scaled and pitted all over, a little more at the washer's rim
    return noisy(s, 0.00018, scale=900.0, seed=31)


def rust_bracket():
    """What is left of a shelf: a pressed steel corner brace (25 mm wide,
    50 x 50 mm, 2.5 mm thick) on two pan-head screws, its corner up, the
    shelf's screw gone from its arm, rusted through its paint. Its corner on
    the wall at the origin: the wall arm down (-y), the shelf arm out (+z)."""
    t, w, a = 0.0025, 0.025, 0.05
    wall = rbox(w, a, t, r=0.0008, seg=1).move(0, -a / 2, t / 2)
    arm = rbox(w, t, a, r=0.0008, seg=1).move(0, -t / 2, a / 2)
    # the bend: a quarter round inside the corner
    bend = cyl(0.004, h=w, seg=8).rot(0, 0, math.pi / 2).move(0, -t - 0.001, t + 0.001)
    s = wall + arm + bend
    holes = [cyl(0.0024, h=0.02, seg=10).rot(math.pi / 2, 0, 0).move(0, y, 0) for y in (-0.016, -0.038)]
    holes.append(cyl(0.0024, h=0.02, seg=10).move(0, 0, 0.032))
    s = cut(s, *holes)
    screws = [screw_head(r=0.0042, h=0.0018).rot(math.pi / 2, 0, 0).move(0, y, t) for y in (-0.016, -0.038)]
    s = s + sum(screws)
    return noisy(s, 0.00016, scale=800.0, seed=47)


# ---------------------------------------------------------- the leak ---
def leak_crust():
    """Lime and rust left by a slow leak, in a lumpy crust 32 mm along the
    joint's underside (its top, y = 0, against it), two stalactites."""
    body = ellipsoid(0.016, 0.004, 0.011, seg=16)
    body.move(y=-0.002)
    lumps = [ellipsoid(0.005, 0.003, 0.004, seg=8).move(x, -0.003 - 0.001 * k, z) for k, (x, z) in
             enumerate([(-0.009, 0.002), (0.004, -0.004), (0.010, 0.003)])]
    drips = [cone(0.0022, 0.009, seg=8).rot(math.pi, 0, 0).move(x, -0.0095, z) for (x, z) in [(-0.003, 0.001), (0.007, -0.002)]]
    s = body + sum(lumps) + sum(drips)
    s = noisy(s, 0.0006, scale=320.0, seed=53)
    # its top flat against the joint
    s.v = [Vector((p.x, min(p.y, 0.0), p.z)) for p in s.v]
    return s


def leak_drop():
    """A drop hanging (5.6 mm across, 8 mm long), its neck at y = 0."""
    prof = [(0.0, -0.0080), (0.0016, -0.0076), (0.0025, -0.0066), (0.0028, -0.0053), (0.0026, -0.0040),
            (0.0019, -0.0026), (0.0011, -0.0013), (0.0008, -0.0004), (0.0, 0.0)]
    return lathe(prof, seg=12)


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


FLAKE = {k: paint_flake(*v) for k, v in FLAKES.items()}
CURL = {'R': wallpaper_curl(False), 'L': wallpaper_curl(True)}
# (each on a line of its own, its budget last: the asset lint reads them)
auto('Real__peeledPaintFlakeATop', lambda: FLAKE['A'][0], 200)
auto('Real__peeledPaintFlakeAUnder', lambda: FLAKE['A'][1], 280)
auto('Real__peeledPaintFlakeBTop', lambda: FLAKE['B'][0], 200)
auto('Real__peeledPaintFlakeBUnder', lambda: FLAKE['B'][1], 280)
auto('Real__peeledPaintFlakeCTop', lambda: FLAKE['C'][0], 200)
auto('Real__peeledPaintFlakeCUnder', lambda: FLAKE['C'][1], 280)
auto('Real__peelingWallpaperCurlRFront', lambda: CURL['R'][0], 420)
auto('Real__peelingWallpaperCurlRBack', lambda: CURL['R'][1], 520)
auto('Real__peelingWallpaperCurlLFront', lambda: CURL['L'][0], 420)
auto('Real__peelingWallpaperCurlLBack', lambda: CURL['L'][1], 520)
auto('Real__rustStreakAnchor', rust_anchor, 800)
auto('Real__rustStreakBracket', rust_bracket, 700)
auto('Real__pipeLeakCrust', leak_crust, 500)
auto('Real__pipeLeakDrop', leak_drop, 180)
