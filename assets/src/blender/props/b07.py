"""b07 — office and industrial props (30 of the batch's 34 placers; the other
four are decals / transparent and stay primitives, see b07.notes.md).

Office: rotary phones (small + desk), ceramic mugs, floor books.
Industrial: rusted / wall / ceiling pipes, junction and fuse boxes, conduit,
cardboard boxes, mops and buckets (janitor, mop, metal, wooden), rolled carpets,
extension cords, tin cans, nails, hammers, hose reel, hard hat, vest, light
bulb socket, safety cone, drain and vent grates, lantern, tool pegboard,
rustic sign, biohazard plate.
"""
import propkit as _pk
from propkit import *


# ================================================================ helpers ==
def rrect_pts(w, d, r, cseg=3):
    """Rounded rectangle (x, z) outline with a fixed point count 4*(cseg+1)."""
    r = max(1e-4, min(r, w / 2 - 1e-4, d / 2 - 1e-4))
    out = []
    for cx, cz, a0 in ((w / 2 - r, d / 2 - r, 0), (-w / 2 + r, d / 2 - r, 90),
                       (-w / 2 + r, -d / 2 + r, 180), (w / 2 - r, -d / 2 + r, 270)):
        for i in range(cseg + 1):
            a = math.radians(a0 + 90 * i / cseg)
            out.append((cx + r * math.cos(a), cz + r * math.sin(a)))
    return out


def loft(rings, cap0=True, cap1=True):
    """Skin closed loops of 3D points (equal counts) into one closed solid."""
    n = len(rings[0])
    v, f = [], []
    for ring in rings:
        v += [Vector(p) for p in ring]
    for k in range(len(rings) - 1):
        a, b = k * n, (k + 1) * n
        for i in range(n):
            j = (i + 1) % n
            f.append((a + i, a + j, b + j, b + i))
    if cap0:
        f.append(tuple(range(n)))
    if cap1:
        f.append(tuple(range((len(rings) - 1) * n, len(rings) * n)))
    return _pk._fix_winding(Shape(v, f))


def yloft(profile, w, d, rc, cseg=3, cx=0.0, cz=0.0):
    """Stack of rounded rectangles along +Y: profile = [(y, inset), ...]."""
    rings = []
    for y, ins in profile:
        pts = rrect_pts(w - 2 * ins, d - 2 * ins, max(0.002, rc - ins * 0.6), cseg)
        rings.append([(cx + x, y, cz + z) for x, z in pts])
    return loft(rings)


def sweep(pts, sec, bnorm=None, caps=True, closed=False):
    """Sweep a varying cross-section along a path. sec(i, t) -> [(u, v), ...]
    offsets along the path normal / binormal. bnorm fixes the binormal (planar
    paths: the plane normal), otherwise a parallel-transport frame is used."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    T = []
    for i in range(n):
        if closed:
            t = pts[(i + 1) % n] - pts[(i - 1) % n]
        else:
            t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        T.append(t.normalized())
    frames = []
    if bnorm is not None:
        B0 = Vector(bnorm).normalized()
        for t in T:
            N = B0.cross(t).normalized()
            frames.append((N, t.cross(N).normalized()))
    else:
        up = Vector((0, 1, 0)) if abs(T[0].y) < 0.9 else Vector((1, 0, 0))
        N = T[0].cross(up).normalized()
        for i, t in enumerate(T):
            if i:
                N = (T[i - 1].rotation_difference(t) @ N).normalized()
            frames.append((N, t.cross(N).normalized()))
    v, f = [], []
    m = 0
    for i, p in enumerate(pts):
        N, B = frames[i]
        ring = sec(i, i / max(1, (n if closed else n - 1)))
        m = len(ring)
        v += [p + N * a + B * b for a, b in ring]
    for i in range(n - 1 + (1 if closed else 0)):
        i2 = (i + 1) % n
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, i2 * m + j2, i2 * m + j))
    if caps and not closed:
        f.append(tuple(reversed(range(m))))
        f.append(tuple((n - 1) * m + j for j in range(m)))
    return _pk._fix_winding(Shape(v, f))


def ellipse(a, b, seg=10, phase=0.0):
    return [(a * math.cos(TAU * j / seg + phase), b * math.sin(TAU * j / seg + phase)) for j in range(seg)]


def ring_lathe(profile, seg=24):
    """Revolve a *closed* (r, y) loop (no point on the axis): rings, washers, bands."""
    s = lathe(list(profile) + [profile[0]], seg, cap_bottom=False, cap_top=False)
    return _pk._fix_winding(weld(s))


def drop_faces(s, pred):
    """Remove faces whose vertices all satisfy pred(Vector) (hidden undersides)."""
    keep = [f for f in s.f if not all(pred(s.v[i]) for i in f)]
    used = sorted({i for f in keep for i in f})
    remap = {o: n for n, o in enumerate(used)}
    return Shape([s.v[i] for i in used], [tuple(remap[i] for i in f) for f in keep])


def fitf(s, frame):
    """Fit a shape exactly onto a frame's bounds."""
    return s.fit(frame[1], frame[2])


def arc_pts(R, a0, a1, n):
    return [(R * math.cos(a0 + (a1 - a0) * i / (n - 1)), R * math.sin(a0 + (a1 - a0) * i / (n - 1)), 0.0) for i in range(n)]


def ribbed(path, r_hi, r_lo, seg=5, bnorm=None):
    """Coiled-cord look: a tube whose radius alternates at every path point."""
    return sweep(path, lambda i, t: ellipse(r_hi if i % 2 == 0 else r_lo, r_hi if i % 2 == 0 else r_lo, seg), bnorm)


# ========================================================== rotary phones ==
def small_phone_base():
    # pressed-steel base plate lip, bakelite body with soft shoulders, flat top
    prof = [(-0.025, 0.002), (-0.0235, 0.0), (-0.017, 0.0), (-0.0155, 0.0035), (-0.013, 0.004),
            (0.010, 0.0062), (0.017, 0.0078), (0.022, 0.0098), (0.0245, 0.0122), (0.025, 0.0142)]
    return yloft(prof, 0.13, 0.10, 0.022, cseg=4)

F_SP_BASE = f_box(0.13, 0.05, 0.10)
part('SmallRotaryPhones__base', F_SP_BASE, small_phone_base)

# finger wheel: number-plate step at the rim, raised wheel, recessed centre card
part('SmallRotaryPhones__dial', f_cyl(0.028, 0.028, 0.006), lambda: lathe(
    [(0, -0.003), (0.0275, -0.003), (0.028, -0.002), (0.028, -0.0003), (0.0258, 0.0005),
     (0.0255, 0.003), (0.0135, 0.003), (0.012, 0.0015), (0, 0.0015)], seg=16))

# finger-hole plugs (dark): a short sleeve with a dished top, reads as a hole
part('SmallRotaryPhones__hole', f_cyl(0.003, 0.003, 0.008), lambda: lathe(
    [(0, -0.004), (0.003, -0.004), (0.003, 0.0035), (0.0026, 0.004), (0.0019, 0.004),
     (0.0016, 0.0022), (0, 0.0022)], seg=8))

# cradle prongs: tapered, rounded tops
part('SmallRotaryPhones__post', f_cyl(0.008, 0.010, 0.014), lambda: lathe(
    [(0, -0.007), (0.010, -0.007), (0.0098, -0.004), (0.0085, 0.003), (0.008, 0.0052),
     (0.0066, 0.0066), (0.004, 0.007), (0, 0.007)], seg=12))


def small_handset():
    # banana grip: oval section, bowed toward local +X (world up once laid down)
    n = 10
    pts = [(0.0028 * (1 - (2 * i / (n - 1) - 1) ** 2), -0.055 + 0.11 * i / (n - 1), 0.0) for i in range(n)]
    def sec(i, t):
        k = 1 + 0.18 * (2 * t - 1) ** 2
        return ellipse(0.0062 * k, 0.0082 * k, 10)
    return fitf(sweep(pts, sec, bnorm=(0, 0, 1)), f_cyl(0.010, 0.010, 0.11))

part('SmallRotaryPhones__handset', f_cyl(0.010, 0.010, 0.11), small_handset, smooth=60)

# ear / mouth cups: rounded puck, wide face with a grille ring
part('SmallRotaryPhones__cup', f_cyl(0.014, 0.012, 0.010), lambda: lathe(
    [(0, -0.005), (0.0105, -0.005), (0.012, -0.0038), (0.0133, -0.0005), (0.014, 0.0025),
     (0.0138, 0.0042), (0.0127, 0.005), (0.009, 0.005), (0.0084, 0.0042), (0, 0.0042)], seg=16))

F_SP_CORD = f_bounds((-0.022, -0.022, -0.002), (0.022, 0.022, 0.002))
part('SmallRotaryPhones__cord', F_SP_CORD, lambda: fitf(
    ribbed(arc_pts(0.020, 0.0, math.pi * 1.5, 27), 0.002, 0.00125, 5, bnorm=(0, 0, 1)), F_SP_CORD), smooth=60)


def phone_base():
    # lowered body with a round dial seat and two cradle horns at the back
    prof = [(-0.025, 0.002), (-0.0235, 0.0), (-0.016, 0.0), (-0.0145, 0.004), (-0.012, 0.0045),
            (0.006, 0.007), (0.013, 0.0098), (0.017, 0.0135), (0.019, 0.0175)]
    body = yloft(prof, 0.18, 0.14, 0.03, cseg=3)
    seat = lathe([(0, 0.012), (0.047, 0.012), (0.047, 0.0232), (0.0452, 0.025), (0, 0.025)], seg=18).move(z=0.02)
    horns = Shape()
    for sx in (-1, 1):
        horns = horns + loft([[(sx * 0.07 + x, y, -0.044 + z) for x, z in rrect_pts(w, d, 0.008, 2)]
                              for y, w, d in ((0.008, 0.036, 0.042), (0.021, 0.03, 0.034), (0.025, 0.024, 0.028))])
    return body + seat + horns

part('RotaryPhones__base', f_box(0.18, 0.05, 0.14), phone_base)


def phone_dial():
    # drilled finger wheel over a number plate; ten holes on a 270 deg sweep
    d = lathe([(0, -0.004), (0.0448, -0.004), (0.045, -0.0015), (0.0436, -0.001), (0.043, 0.0035),
               (0.0422, 0.004), (0.0135, 0.004), (0.0125, 0.0028), (0, 0.0028)], seg=18)
    holes = [cyl(0.0062, 0.0062, 0.02, seg=7, bevel_r=0).move(x=0.0285 * math.sin(math.radians(-45 + 30 * i)),
                                                            z=0.0285 * math.cos(math.radians(-45 + 30 * i)))
             for i in range(10)]
    # the underside sits on the phone body: drop it so the holes show the dark body
    return drop_faces(cut(d, sum(holes, Shape())), lambda p: p.y < -0.0039)

part('RotaryPhones__dial', f_cyl(0.045, 0.045, 0.008), phone_dial)

part('RotaryPhones__dialCenter', f_cyl(0.010, 0.010, 0.010), lambda: lathe(
    [(0, -0.005), (0.0099, -0.005), (0.010, 0.001), (0.0082, 0.0036), (0.0045, 0.0049), (0, 0.005)], seg=12))


def phone_handset():
    # classic handle: arched grip, ends turning down into the cups, rounded-oval section
    n = 16
    pts = []
    for i in range(n):
        u = 2 * i / (n - 1) - 1
        pts.append((0.08 * u, 0.0042 * (1 - u * u) - 0.0035 * u ** 6, 0.0))
    def sec(i, t):
        u = abs(2 * t - 1)
        k = 0.86 + 0.14 * u ** 2
        return ellipse(0.0105 * k, 0.0138 * k, 14)
    s = sweep(pts, sec, bnorm=(0, 0, 1))
    return fitf(s, f_box(0.16, 0.03, 0.03))

part('RotaryPhones__handsetBar', f_box(0.16, 0.03, 0.03), phone_handset, smooth=60)

# ear / mouth piece: bell cup, flat grille face down, neck into the handle up top
part('RotaryPhones__ear', f_sphere(0.025), lambda: lathe(
    [(0, -0.0225), (0.012, -0.0225), (0.0145, -0.025), (0.021, -0.025), (0.0245, -0.0205),
     (0.025, -0.013), (0.022, -0.001), (0.016, 0.012), (0.0125, 0.025), (0, 0.025)], seg=16), smooth=50)

part('RotaryPhones__cord', f_cyl(0.004, 0.004, 0.15), lambda: ribbed(
    [(0, -0.075 + 0.15 * i / 40, 0) for i in range(41)], 0.004, 0.0026, 6), smooth=60)


# =================================================================== mugs ==
def mug_body():
    prof = [(0.031, -0.045), (0.0345, -0.0445), (0.0358, -0.042), (0.036, -0.036), (0.0359, 0.03), (0.0355, 0.044)]
    return fitf(vessel(prof, 0.0033, seg=24, floor=0.007), f_cyl(0.036, 0.036, 0.09))

part('CeramicMugs__body', f_cyl(0.036, 0.036, 0.09), mug_body)

part('CeramicMugs__saucer', f_cyl(0.06, 0.055, 0.008), lambda: lathe(
    [(0, -0.0028), (0.031, -0.0028), (0.033, -0.004), (0.037, -0.004), (0.039, -0.0026),
     (0.055, -0.0004), (0.0598, 0.0028), (0.0592, 0.004), (0.0565, 0.0036), (0.039, 0.0004), (0, -0.0008)], seg=24))


def mug_handle():
    # D-shaped loop, flattened oval section, flaring where it meets the mug
    n = 14
    pts = []
    for i in range(n):
        t = math.pi * i / (n - 1)
        c, s = math.cos(t), math.sin(t)
        pts.append((0.0205 * math.copysign(abs(c) ** 0.8, c), 0.0205 * abs(s) ** 0.75, 0.0))
    def sec(i, t):
        e = abs(2 * t - 1) ** 4
        return ellipse(0.0032 + 0.0016 * e, 0.0047 + 0.0006 * e, 8)
    return fitf(sweep(pts, sec, bnorm=(0, 0, 1)), F_MUG_HANDLE)

F_MUG_HANDLE = f_bounds((-0.025, -0.005, -0.005), (0.025, 0.025, 0.005))
part('CeramicMugs__handle', F_MUG_HANDLE, mug_handle, smooth=60)


# ================================================================= books ==
def grid_box(xs, ys, zs):
    """Closed box whose faces are grids over the given coordinate lists
    (so seams, bulges and dents can be warped in)."""
    idx, v, f = {}, [], []
    def vid(i, j, k):
        key = (i, j, k)
        if key not in idx:
            idx[key] = len(v)
            v.append(Vector((xs[i], ys[j], zs[k])))
        return idx[key]
    nx, ny, nz = len(xs) - 1, len(ys) - 1, len(zs) - 1
    for i in (0, nx):
        for j in range(ny):
            for k in range(nz):
                f.append((vid(i, j, k), vid(i, j + 1, k), vid(i, j + 1, k + 1), vid(i, j, k + 1)))
    for j in (0, ny):
        for i in range(nx):
            for k in range(nz):
                f.append((vid(i, j, k), vid(i + 1, j, k), vid(i + 1, j, k + 1), vid(i, j, k + 1)))
    for k in (0, nz):
        for i in range(nx):
            for j in range(ny):
                f.append((vid(i, j, k), vid(i + 1, j, k), vid(i + 1, j + 1, k), vid(i, j + 1, k)))
    return _pk._fix_winding(Shape(v, f))


def book_cover():
    # hardcover board: rounded fore-edge corners, square spine side, hinge groove
    w, d, t = 0.12, 0.18, 0.005
    out = [(-w / 2, -d / 2)] + [p for c in (((w / 2 - 0.006, -d / 2 + 0.006), -90), ((w / 2 - 0.006, d / 2 - 0.006), 0))
                                for p in [(c[0][0] + 0.006 * math.cos(math.radians(c[1] + 30 * i)),
                                           c[0][1] + 0.006 * math.sin(math.radians(c[1] + 30 * i))) for i in range(4)]] + [(-w / 2, d / 2)]
    b = prism(out, t, bevel_r=0.0009, bevel_seg=1).rot(x=math.pi / 2)
    b = cut(b, box(0.0022, 0.004, 0.3).move(x=-0.047, y=t / 2 + 0.0012))
    return b.warp(lambda p: Vector((p.x, p.y - 0.0006 * (p.z / 0.09) ** 2, p.z)))

part('FloorBooks__cover', f_box(0.12, 0.005, 0.18), book_cover)


def book_pages():
    # text block: concave fore-edge (+X), convex spine side, page-bundle ridges
    w, h, d = 0.115, 0.020, 0.175
    rings = []
    n = 15
    for k in range(n):
        y = -h / 2 + h * k / (n - 1)
        u = 2 * k / (n - 1) - 1
        ins = 0.0 if k in (0, n - 1) else (0.00035 if k % 2 else 0.0)
        xf = w / 2 - 0.0022 * (1 - u * u) - ins
        xs = -w / 2 + 0.0012 * (1 - u * u) + ins * 0.5
        dd = d - 2 * ins - 0.0008 * (1 - u * u)
        pts = rrect_pts(xf - xs, dd, 0.0015, 2)
        rings.append([((xf + xs) / 2 + x, y, z) for x, z in pts])
    return loft(rings)

part('FloorBooks__pages', f_box(0.115, 0.020, 0.175), book_pages)


# ================================================================== pipes ==
def pipe_profile(r, rc, L, bore, couplings=(), end_len=0.06, rings_every=0.2):
    """Steel pipe along Y: couplings at the ends (and at `couplings` y's),
    an open bore at each end, extra rings along the run for pitting."""
    y0, y1 = -L / 2, L / 2
    prof = [(0, y0 + 0.03), (bore, y0 + 0.03), (bore, y0), (rc - 0.001, y0), (rc, y0 + 0.002),
            (rc, y0 + end_len - 0.003), (r + 0.001, y0 + end_len)]
    marks = sorted(couplings)
    y = y0 + end_len + rings_every
    while y < y1 - end_len - 0.05:
        if all(abs(y - c) > 0.06 for c in marks):
            prof.append((r, y))
        for c in list(marks):
            if c < y + rings_every and c > prof[-1][1]:
                prof += [(r, c - 0.04), (rc - 0.0015, c - 0.034), (rc - 0.0015, c + 0.034), (r, c + 0.04)]
                marks.remove(c)
        y += rings_every
    prof += [(r + 0.001, y1 - end_len), (rc, y1 - end_len + 0.003), (rc, y1 - 0.002), (rc - 0.001, y1),
             (bore, y1), (bore, y1 - 0.03), (0, y1 - 0.03)]
    return prof


def rusted_pipe():
    s = lathe(pipe_profile(0.0355, 0.04, 3.1, 0.029, couplings=(0.35,)), seg=16)
    return noisy(s, 0.0011, scale=40, seed=3)

part('RustedPipes__pipe', f_cyl(0.04, 0.04, 3.1), rusted_pipe)


def pipe_support():
    # cast support: bolted foot, tapered pedestal, dished pad under the pipe
    foot = rbox(0.06, 0.012, 0.08, r=0.003, seg=1).move(y=-0.019)
    ped = yloft([(-0.014, 0.004), (0.012, 0.011), (0.018, 0.008), (0.025, 0.008)], 0.06, 0.08, 0.012, cseg=2)
    bolts = sum((lathe([(0, 0), (0.0055, 0), (0.0055, 0.003), (0.004, 0.0045), (0, 0.0045)], seg=6)
                 .move(x=sx * 0.021, y=-0.013, z=sz * 0.03) for sx in (-1, 1) for sz in (-1, 1)), Shape())
    return fitf(foot + ped + bolts, f_box(0.06, 0.05, 0.08))

part('RustedPipes__bracket', f_box(0.06, 0.05, 0.08), pipe_support)


# ======================================================== junction boxes ==
def zloft(profile, w, h, rc, cseg=3):
    """Stack of rounded rectangles in XY along +Z: profile = [(z, inset), ...]."""
    return loft([[(x, y, z) for x, y in rrect_pts(w - 2 * ins, h - 2 * ins, max(0.002, rc - ins * 0.6), cseg)]
                 for z, ins in profile])


def junction_box():
    body = rbox(0.156, 0.196, 0.05, r=0.006, seg=2).move(z=-0.005)
    cover = zloft([(0.017, 0.0), (0.026, 0.0), (0.0285, 0.0018), (0.029, 0.004), (0.0295, 0.013), (0.03, 0.0145)],
                  0.16, 0.20, 0.008)
    # conduit knockouts pressed into the sides
    ko = sum((cyl(0.011, 0.011, 0.004, seg=14, bevel_r=0).rot(z=math.pi / 2).move(x=sx * 0.079, y=0.02, z=-0.004)
              for sx in (-1, 1)), Shape())
    return fitf(cut(body, ko) + cover, f_box(0.16, 0.20, 0.06))

part('JunctionBoxes__box', f_box(0.16, 0.20, 0.06), junction_box)

# hex screw head with a chamfered top
part('JunctionBoxes__screw', f_cyl(0.005, 0.005, 0.005), lambda: lathe(
    [(0, -0.0025), (0.005, -0.0025), (0.005, 0.0012), (0.0041, 0.0025), (0, 0.0025)], seg=6), smooth=20)


def conduit():
    L, r, rc = 0.30, 0.0122, 0.015
    y0 = -L / 2
    end = [(0, y0 + 0.012), (0.0095, y0 + 0.012), (0.0095, y0), (rc - 0.001, y0), (rc, y0 + 0.0015),
           (rc, y0 + 0.017), (rc - 0.0012, y0 + 0.019), (0.0136, y0 + 0.02), (0.0136, y0 + 0.032), (r, y0 + 0.035)]
    prof = end + [(r, 0.0)] + [(a, -b) for a, b in reversed(end)]
    return lathe(prof, seg=12)

part('JunctionBoxes__conduit', f_cyl(0.015, 0.015, 0.30), conduit)


def torn_wire():
    pts = [(0.0011 * math.sin(i * 1.7), -0.145 + 0.29 * i / 11, 0.0011 * math.cos(i * 2.3)) for i in range(12)]
    return fitf(sweep(pts, lambda i, t: ellipse(0.0029, 0.0029, 6)), f_cyl(0.004, 0.004, 0.29))

part('JunctionBoxes__wire', f_cyl(0.004, 0.004, 0.29), torn_wire, smooth=60)


def copper_tip():
    # frayed strands, bundled at the insulation (+Y) and splaying downwards
    s = Shape()
    for i in range(5):
        a = TAU * i / 5 + 0.3
        s = s + tube([(0.0012 * math.cos(a), 0.01, 0.0012 * math.sin(a)),
                      (0.0022 * math.cos(a), 0.0, 0.0022 * math.sin(a)),
                      (0.0041 * math.cos(a + 0.2), -0.0095, 0.0041 * math.sin(a + 0.2))], 0.0009, seg=3)
    return s

part('JunctionBoxes__tip', f_cyl(0.005, 0.005, 0.02), copper_tip)


# ======================================================= cardboard boxes ==
def cardboard_box():
    w, h, d = 0.40, 0.30, 0.40
    fx = [-0.5, -0.47, -0.3, -0.1, 0.1, 0.3, 0.47, 0.5]
    fz = [-0.5, -0.47, -0.3, -0.1, -0.012, 0.0, 0.012, 0.1, 0.3, 0.47, 0.5]
    fy = [-0.5, -0.46, -0.25, 0.0, 0.25, 0.46, 0.5]
    s = grid_box([w * a for a in fx], [h * a for a in fy], [d * a for a in fz])
    def shape(p):
        u, v, q = p.x / (w / 2), p.y / (h / 2), p.z / (d / 2)
        bx = (1 - v * v) * (1 - q * q)          # side panels bow out a little
        bz = (1 - v * v) * (1 - u * u)
        p.x += 0.006 * bx * (1 if u > 0.99 else -1 if u < -0.99 else 0)
        p.z += 0.006 * bz * (1 if q > 0.99 else -1 if q < -0.99 else 0)
        if abs(v) > 0.99 and abs(p.z) < 1e-4:   # flap seam top and bottom
            p.y -= 0.004 * (1 if v > 0 else -1) * (1 - u ** 8)
        if abs(v) > 0.99 and abs(abs(p.z) - 0.0048) < 1e-4:
            p.y -= 0.0012 * (1 if v > 0 else -1) * (1 - u ** 8)
        return p
    s = bevel(s.warp(shape), 0.004, seg=1, angle=45)
    s = noisy(noisy(s, 0.0015, scale=9, seed=5, axis=(1, 0, 0)), 0.0015, scale=9, seed=6, axis=(0, 0, 1))
    return fitf(s, f_box(w, h, d))

part('CardboardBoxes__body', f_box(0.40, 0.30, 0.40), cardboard_box)


# ================================================================== mops ==
def grid_lathe(radius, ys, seg, top=None, bottom=None):
    """Surface of revolution with a per-vertex radius(j, a, y) -> (r, y):
    lets strands, ridges and ragged hems be warped in. seg may be a list of
    angles (radians) for uneven spacing."""
    angles = list(seg) if isinstance(seg, (list, tuple)) else [TAU * i / seg for i in range(seg)]
    seg = len(angles)
    rings = []
    for j, y in enumerate(ys):
        ring = []
        for a in angles:
            r, yy = radius(j, a, y)
            ring.append((r * math.sin(a), yy, r * math.cos(a)))
        rings.append(ring)
    s = loft(rings, cap0=False, cap1=False)
    v, f = list(s.v), list(s.f)
    n = len(v)
    for ring_ids, yc in ((range(0, seg), bottom), (range((len(ys) - 1) * seg, len(ys) * seg), top)):
        if yc is None:
            continue
        ids = list(ring_ids)
        c = len(v)
        v.append(Vector((0, yc, 0)))
        for i in range(seg):
            f.append((ids[i], ids[(i + 1) % seg], c))
    return _pk._fix_winding(Shape(v, f))


def mop_stick(L, r, hole=True):
    y0, y1 = -L / 2, L / 2
    prof = [(0, y0), (r * 0.9, y0), (r, y0 + 0.002), (r, y0 + 0.05), (r * 0.94, y0 + 0.053), (r * 0.94, y0 + 0.06),
            (r * 0.9, y0 + 0.065), (r * 0.9, y1 - 0.012), (r * 0.8, y1 - 0.004), (r * 0.5, y1), (0, y1)]
    s = lathe(prof, seg=12)
    if hole:
        s = cut(s, cyl(r * 0.32, r * 0.32, r * 3, seg=8, bevel_r=0).rot(z=math.pi / 2).move(y=y1 - 0.035))
    return s

part('Mops__stick', f_cyl(0.015, 0.015, 1.4), lambda: mop_stick(1.4, 0.015))


def mop_head():
    # string mop: strands gathered at the clamp, bellied, ragged hem on the floor
    rnd = random.Random(7)
    hem = [rnd.uniform(0.0, 0.014) for _ in range(32)]
    ys = [-0.07, -0.055, -0.03, 0.0, 0.03, 0.05, 0.063, 0.07]
    rs = [0.058, 0.066, 0.074, 0.078, 0.074, 0.058, 0.036, 0.018]
    def rad(j, a, y):
        i = round(a / TAU * 32) % 32
        r = rs[j] * (1 + (0.07 if i % 2 else -0.05) * (1 if j < 6 else 0.4))
        return r, (y + hem[i] if j == 0 else y)
    s = grid_lathe(rad, ys, 32, top=0.07, bottom=-0.064)
    return fitf(noisy(s, 0.004, scale=30, seed=2), f_cyl(0.08, 0.06, 0.14))

part('Mops__head', f_cyl(0.08, 0.06, 0.14), mop_head, smooth=70)


def janitor_bucket():
    prof = [(0.105, -0.11), (0.1095, -0.106), (0.111, -0.095), (0.1275, 0.03), (0.131, 0.034), (0.1315, 0.042),
            (0.1325, 0.07), (0.1375, 0.1), (0.14, 0.11)]
    return fitf(vessel(prof, 0.006, seg=28, floor=0.008), f_cyl(0.14, 0.11, 0.22))

part('Mops__bucket', f_cyl(0.14, 0.11, 0.22), janitor_bucket)
part('Mops__rim', f_torus(0.14, 0.006), lambda: torus(0.14, 0.006, seg=44, tseg=6))
# wire bail: side strands and a top bar with a grip sleeve
part('Mops__wire', f_cyl(0.003, 0.003, 0.10), lambda: capsule(0.0019, 0.10 - 0.0038, seg=8, cap_rings=2))
part('Mops__handleTop', f_cyl(0.003, 0.003, 0.24), lambda: lathe(
    [(0, -0.12), (0.0019, -0.1195), (0.0019, -0.05), (0.0026, -0.048), (0.003, -0.044), (0.003, 0.044),
     (0.0026, 0.048), (0.0019, 0.05), (0.0019, 0.1195), (0, 0.12)], seg=8), smooth=50)


# ======================================================== rolled carpets ==
def carpet_roll():
    # a wound strip: the ends show the spiral, the outer layer ends in a step
    L, R, t, r0, turns = 2.4, 0.12, 0.021, 0.03, 3.5
    p = (R - t / 2 - r0) / turns
    n = int(turns * 18)
    th = [TAU * turns * i / n for i in range(n + 1)]
    c = lambda a: r0 + p * a / TAU
    outer = [((c(a) + t / 2) * math.cos(a), (c(a) + t / 2) * math.sin(a)) for a in th]
    a1 = th[-1]
    tipc = (c(a1) * math.cos(a1), c(a1) * math.sin(a1))
    tn = Vector((-math.sin(a1), math.cos(a1)))
    cap = [(tipc[0] + t / 2 * (math.cos(a1) * math.cos(b) + tn.x * math.sin(b)),
            tipc[1] + t / 2 * (math.sin(a1) * math.cos(b) + tn.y * math.sin(b))) for b in (0.8, 1.6, 2.4)]
    inner = [((c(a) - t / 2) * math.cos(a), (c(a) - t / 2) * math.sin(a)) for a in reversed(th)]
    start = [(r0 * math.cos(math.pi + 0.5), r0 * math.sin(math.pi + 0.5))]
    loop = outer + cap + inner + start
    # spiral position of every outline point: the inner wraps telescope back from the ends
    tpos = th + [a1] * len(cap) + list(reversed(th)) + [0.0]
    rnd = random.Random(4)
    rings = []
    for k, y in enumerate((-L / 2, -L / 2 + 0.25, -0.4, 0.4, L / 2 - 0.25, L / 2)):
        sc = 1.0 - (0.012 if 0 < k < 5 else 0.0) * rnd.random()
        ox, oz = rnd.uniform(-0.003, 0.003), rnd.uniform(-0.003, 0.003)
        rec = (lambda a: 0.0) if 0 < k < 5 else (lambda a: math.copysign(0.035 * (1 - a / a1) ** 1.5, -y))
        rings.append([(x * sc + ox, y + rec(a), z * sc + oz) for (x, z), a in zip(loop, tpos)])
    return fitf(loft(rings), f_cyl(0.12, 0.12, 2.4))

part('RolledCarpets__roll', f_cyl(0.12, 0.12, 2.4), carpet_roll)


# ======================================================= extension cords ==
def cord_segment():
    # round flex with domed ends (the zig-zag joints overlap into a bend), lying slightly wavy
    L, r = 0.6, 0.0105
    ys = [-0.3, -0.2985, -0.2945, -0.2875, -0.2795] + [-0.25 + 0.05 * i for i in range(11)] + [0.2795, 0.2875, 0.2945, 0.2985, 0.3]
    capr = lambda y: r * math.sqrt(max(0.0, 1 - ((abs(y) - (0.3 - r)) / r) ** 2)) if abs(y) > 0.3 - r else r
    prof = [(0, -0.3)] + [(capr(y), y) for y in ys[1:-1]] + [(0, 0.3)]
    s = lathe(prof, seg=10)
    return s.warp(lambda q: Vector((q.x + 0.0013 * math.sin(q.y * 11), q.y, q.z + 0.0011 * math.sin(q.y * 7 + 1))))

part('ExtensionCords__seg', f_cyl(0.012, 0.012, 0.6), cord_segment, smooth=60)


def plug_body():
    body = rbox(0.036, 0.03, 0.035, r=0.006, seg=2).move(x=0.007)
    boot = lathe([(0, -0.0135), (0.0062, -0.0135), (0.0065, -0.005), (0.0085, 0.002), (0.0095, 0.0045), (0, 0.0045)],
                 seg=10).rot(z=math.pi / 2).move(x=-0.0125)
    ridges = sum((box(0.02, 0.0016, 0.0022).move(x=0.006, y=yy, z=sz * 0.0172) for yy in (-0.004, 0.004) for sz in (-1, 1)), Shape())
    return fitf(body + boot + ridges, f_box(0.05, 0.03, 0.035))

part('ExtensionCords__plug', f_box(0.05, 0.03, 0.035), plug_body)
part('ExtensionCords__prong', f_box(0.02, 0.003, 0.003), lambda: rbox(0.02, 0.003, 0.003, r=0.0007, seg=1))


# ========================================================= rusted tin cans ==
def tin_can():
    # double-seam rims, recessed lid and base, one bead each side of the label
    s = lathe([(0, -0.0362), (0.0245, -0.0362), (0.026, -0.0375), (0.028, -0.036), (0.0266, -0.0325),
               (0.0272, -0.02), (0.0272, 0.02), (0.0266, 0.0325), (0.028, 0.036), (0.026, 0.0375),
               (0.0245, 0.0355), (0, 0.0355)], seg=14)
    return noisy(s, 0.0005, scale=70, seed=8)

part('RustedTinCans__body', f_cyl(0.028, 0.028, 0.075), tin_can)


def can_label():
    # paper band, torn away between 20 and 70 degrees, ragged hems, peeling edge
    rnd = random.Random(11)
    nx = 18
    a0, a1 = math.radians(70), math.radians(380)
    s = slab(1.0, 0.035, 0.0005, nx, 2)
    jit = [rnd.uniform(0, 0.0025) for _ in range(nx + 1)]
    def wrap(q):
        u = q.x + 0.5
        i = round(u * nx)
        a = a0 + (a1 - a0) * u
        r = 0.0287 + q.z + (0.0016 * (1 - u / 0.08) if u < 0.08 else 0.0)
        y = q.y - math.copysign(jit[i], q.y) if abs(q.y) > 0.017 else q.y
        return Vector((r * math.cos(a), y, r * math.sin(a)))
    return s.warp(wrap)

part('RustedTinCans__label', f_cyl(0.029, 0.029, 0.035), can_label)


# =========================================================== mop buckets ==
def bail(R, r, grip=None, over=0.07, hook=0.0025, n=26):
    """Wire bucket bail in the half-torus frame (arc 0..pi in XY): the ends run
    a little past the ears and hook inward; optional grip sleeve at the top."""
    pts = [(R, -hook, 0.0)]
    pts += [(R * math.cos(t), R * math.sin(t), 0.0) for t in
            [-over + (math.pi + 2 * over) * i / (n - 1) for i in range(n)]]
    pts += [(-R, -hook, 0.0)]
    pts[0] = (R - 0.004, -hook, 0.0)
    pts[-1] = (-R + 0.004, -hook, 0.0)
    def sec(i, t):
        rr = r
        if grip and abs(t - 0.5) < grip[1]:
            rr = grip[0] if abs(t - 0.5) < grip[1] - 0.025 else (r + grip[0]) / 2
        return ellipse(rr, rr, 6)
    return sweep(pts, sec, bnorm=(0, 0, 1))

part('MopBuckets__body', f_cyl(0.075, 0.055, 0.15), lambda: fitf(vessel(
    [(0.052, -0.075), (0.0552, -0.0715), (0.0556, -0.066), (0.0712, 0.049), (0.0738, 0.053), (0.0738, 0.068), (0.075, 0.075)],
    0.004, seg=20, floor=0.005), f_cyl(0.075, 0.055, 0.15)))
part('MopBuckets__handle', f_bounds((-0.078, -0.003, -0.003), (0.078, 0.078, 0.003)),
     lambda: bail(0.075, 0.0024), smooth=50)
part('MopBuckets__stick', f_cyl(0.008, 0.008, 0.60), lambda: mop_stick(0.60, 0.008, hole=False))


def small_mop_head():
    # dirty cotton strands bundled into a lumpy tassel
    rnd = random.Random(3)
    hem = [rnd.uniform(0.0, 0.008) for _ in range(20)]
    ys = [-0.045, -0.04, -0.02, 0.005, 0.03, 0.045]
    rs = [0.022, 0.034, 0.043, 0.045, 0.036, 0.016]
    def rad(j, a, y):
        i = round(a / TAU * 20) % 20
        r = rs[j] * (1 + (0.08 if i % 2 else -0.06))
        return r, (y + hem[i] if j == 0 else y)
    s = grid_lathe(rad, ys, 20, top=0.045, bottom=-0.042)
    return fitf(noisy(s, 0.0025, scale=45, seed=9), f_sphere(0.045))

part('MopBuckets__mopHead', f_sphere(0.045), small_mop_head, smooth=70)


# ======================================================== biohazard sign ==
def sign_plate():
    # enamelled plate: rounded corners, 3 mm deep into the wall, corner screw holes
    s = prism(rounded_rect(0.2, 0.2, 0.012, 3), 0.003, bevel_r=0.0006, bevel_seg=1).move(z=-0.0015)
    return cut(s, sum((cyl(0.0028, 0.0028, 0.02, seg=8, bevel_r=0).rot(x=math.pi / 2).move(x=sx * 0.086, y=sy * 0.086)
                       for sx in (-1, 1) for sy in (-1, 1)), Shape()))

part('BiohazardSigns__plate', f_bounds((-0.1, -0.1, 0), (0.1, 0.1, 0), 'plane'), sign_plate)


# ========================================================== rusted nails ==
part('RustedNails__shaft', f_cyl(0.001, 0.001, 0.032), lambda: sweep(
    [(0.0004 * math.sin(math.pi * i / 5), -0.016 + 0.032 * i / 5, 0.0) for i in range(6)],
    lambda i, t: ellipse(0.001, 0.001, 6)), smooth=60)
part('RustedNails__head', f_cyl(0.003, 0.003, 0.002), lambda: lathe(
    [(0, -0.001), (0.0029, -0.001), (0.003, -0.0003), (0.0026, 0.0007), (0.0012, 0.001), (0, 0.001)], seg=10))


# ======================================================== rusted hammers ==
def hammer_handle():
    # hickory handle: flared into the eye (-Y), slim neck, swelling grip, knob butt (+Y)
    s = lathe([(0, -0.1), (0.0086, -0.1), (0.0088, -0.085), (0.0072, -0.055), (0.0070, -0.02), (0.0080, 0.03),
               (0.0084, 0.07), (0.0093, 0.088), (0.0092, 0.095), (0.0075, 0.0995), (0, 0.1)], seg=12)
    return s.scale(0.84, 1, 1)          # oval section, flat side down on the floor

part('RustedHammers__handle', f_cyl(0.008, 0.010, 0.20), hammer_handle)


def hammer_head():
    # forged head lying on its cheek: eye block, round striking face on -Z, tapered peen on +Z
    eye = rbox(0.030, 0.02, 0.012, r=0.003, seg=1)
    neck = lathe([(0, 0.005), (0.0082, 0.005), (0.0078, 0.009), (0.0092, 0.011), (0.0092, 0.0128),
                  (0.0082, 0.014), (0, 0.014)], seg=12).rot(x=-math.pi / 2)
    peen = loft([[(x, y, z) for x, y in rrect_pts(w, h, 0.002, 1)]
                 for z, w, h in ((0.004, 0.026, 0.018), (0.014, 0.016, 0.006))])
    return fitf(eye + neck + peen, f_box(0.030, 0.020, 0.028))

part('RustedHammers__head', f_box(0.030, 0.020, 0.028), hammer_head)


def hammer_claw():
    # curved, tapering prong with a flattened tip
    pts = [(-0.0015 + 0.004 * (i / 7) ** 2, -0.01 + 0.02 * i / 7, 0.0) for i in range(8)]
    s = sweep(pts, lambda i, t: ellipse(0.0036 - 0.002 * t, 0.0036 - 0.0012 * t, 6), bnorm=(0, 0, 1))
    return fitf(s, f_box(0.008, 0.020, 0.008))

part('RustedHammers__claw', f_box(0.008, 0.020, 0.008), hammer_claw)


# ======================================================= fire hose reels ==
def hose_backplate():
    plate = zloft([(-0.005, 0.0), (0.0005, 0.0), (0.002, 0.0018)], 0.42, 0.42, 0.02)
    bolts = sum((lathe([(0, 0), (0.0075, 0), (0.0075, 0.002), (0.006, 0.003), (0, 0.003)], seg=6)
                 .rot(x=math.pi / 2).move(x=sx * 0.18, y=sy * 0.18, z=0.002) for sx in (-1, 1) for sy in (-1, 1)), Shape())
    return plate + bolts

part('FireHoseReels__plate', f_box(0.42, 0.42, 0.010), hose_backplate)


def hose_drum():
    # reel wheel: rim, recessed face, five spokes, hub boss (front = +Y)
    s = lathe([(0, -0.04), (0.1, -0.04), (0.1, 0.037), (0.098, 0.04), (0.087, 0.04), (0.085, 0.03),
               (0.026, 0.03), (0.024, 0.038), (0.021, 0.04), (0, 0.04)], seg=28)
    spokes = sum((box(0.011, 0.007, 0.064).move(y=0.0335, z=0.055).rot(y=TAU * k / 5)
                  for k in range(5)), Shape())
    return s + spokes

part('FireHoseReels__drum', f_cyl(0.10, 0.10, 0.08), hose_drum)


def hose_coil():
    # three wraps of flat hose: smooth outer wrap, grooves between wraps on the face
    sec = [(0.019, -0.0135), (0.019, 0.0135), (0.0172, 0.0182), (0.0085, 0.0187), (0.0065, 0.0145), (0.0045, 0.0187),
           (-0.0045, 0.0187), (-0.0065, 0.0145), (-0.0085, 0.0187), (-0.0172, 0.0182), (-0.019, 0.0135),
           (-0.019, -0.0135), (-0.0172, -0.0182), (0.0172, -0.0182)]
    pts = arc_pts(0.14, 0.0, TAU, 33)[:-1]
    return sweep(pts, lambda i, t: [(-u, v) for u, v in sec], bnorm=(0, 0, 1), closed=True)

part('FireHoseReels__hose', f_torus(0.14, 0.020), hose_coil, smooth=50)

# brass branch pipe: coupling (wide, +Y) into a tapered barrel and a tip orifice
part('FireHoseReels__nozzle', f_cyl(0.014, 0.010, 0.030), lambda: lathe(
    [(0, -0.0135), (0.0035, -0.0135), (0.0035, -0.015), (0.0058, -0.013), (0.0085, -0.002), (0.0105, -0.0005),
     (0.0105, 0.004), (0.014, 0.0055), (0.014, 0.0135), (0.0125, 0.015), (0, 0.015)], seg=14))


# ============================================================= hard hats ==
def hat_shell():
    # moulded shell with a crown ridge front-to-back and two side ribs
    ridges = [(math.pi / 2, 0.0065, 0.13), (3 * math.pi / 2, 0.0065, 0.13)]
    base = [TAU * i / 24 for i in range(24)]
    extra = [c + d for c, _, _ in ridges for d in (-0.1, -0.05, 0.0, 0.05, 0.1)]
    angles = sorted(set(round(a % TAU, 5) for a in base + extra))
    ys = [0.0, 0.012, 0.026, 0.04, 0.051, 0.06, 0.066, 0.0695]
    def rad(j, a, y):
        r = 0.0695 * math.sqrt(max(0.0, 1 - (y / 0.0695) ** 2)) + (0.0005 if j == 0 else 0)
        for c, hgt, w in ridges:
            d = abs((a - c + math.pi) % TAU - math.pi)
            if d < w:
                r += hgt * (1 - d / w) ** 1.5 * (0.4 + 0.6 * y / 0.07)
        return r, y
    s = grid_lathe(rad, ys, angles, top=0.07, bottom=0.0)
    return fitf(s, F_HAT_DOME)

F_HAT_DOME = f_bounds((-0.07, 0, -0.07), (0.07, 0.07, 0.07))
part('HardHats__dome', F_HAT_DOME, hat_shell, smooth=50)

# brim ring with an upturned rain gutter
part('HardHats__brim', f_cyl(0.070, 0.080, 0.010), lambda: ring_lathe(
    [(0.062, -0.005), (0.079, -0.005), (0.080, -0.0025), (0.080, 0.0012), (0.0785, 0.0016), (0.0765, -0.0004),
     (0.0725, 0.0015), (0.0700, 0.005), (0.062, 0.005)], seg=28))


def hat_peak():
    out = [(-0.015, -0.015), (-0.005, -0.015), (0.003, -0.015)]
    out += [(0.003 + 0.012 * math.sin(math.pi * i / 8), -0.015 * math.cos(math.pi * i / 8)) for i in range(1, 8)]
    out += [(0.003, 0.015), (-0.005, 0.015), (-0.015, 0.015)]
    s = prism(out, 0.0038, bevel_r=0.0008, bevel_seg=1).rot(x=math.pi / 2)
    s = s.warp(lambda p: Vector((p.x, p.y - 0.0042 * max(0.0, (p.x + 0.005) / 0.02) ** 2, p.z)))
    return fitf(s, f_box(0.030, 0.008, 0.030))

part('HardHats__peak', f_box(0.030, 0.008, 0.030), hat_peak)
part('HardHats__strapDot', f_cyl(0.006, 0.006, 0.004), lambda: lathe(
    [(0, -0.002), (0.006, -0.002), (0.006, -0.0006), (0.0048, 0.0014), (0.0025, 0.002), (0, 0.002)], seg=10))


# =========================================================== safety vests ==
def folded_vest():
    # folded garment: three layers (grooves round the sides), crease between the
    # stripes, a slightly pillowed top, eased edges
    xs = [-0.1, -0.096, 0.0, 0.096, 0.1]
    ys = [-0.01, -0.0035, 0.0035, 0.01]
    zs = [-0.08, -0.076, -0.004, 0.0, 0.004, 0.074, 0.08]
    s = grid_box(xs, ys, zs)
    def w(p):
        if abs(abs(p.y) - 0.0035) < 1e-5:
            if abs(p.x) > 0.0999:
                p.x -= math.copysign(0.0028, p.x)
            if abs(p.z) > 0.0799:
                p.z -= math.copysign(0.0028, p.z)
        if p.y > 0.0099:
            if abs(p.z) < 1e-4:
                p.y -= 0.0011
            elif abs(p.x) < 0.097 and abs(p.z) < 0.075:
                p.y += 0.0008
        return p
    s = bevel(s.warp(w), 0.003, seg=2, angle=40)
    return fitf(noisy(s, 0.001, scale=25, seed=4), f_box(0.20, 0.020, 0.16))

part('SafetyVests__body', f_box(0.20, 0.020, 0.16), folded_vest, smooth=70)


# ======================================================= wall pipe leaks ==
part('WallPipeLeaks__pipe', f_cyl(0.024, 0.024, 0.60), lambda: noisy(lathe(
    pipe_profile(0.0215, 0.024, 0.60, 0.016, end_len=0.035, rings_every=0.1), seg=14), 0.0005, scale=50, seed=12))


def pipe_coupling():
    s = lathe([(0, -0.01), (0.0232, -0.01), (0.0253, -0.0082), (0.0253, -0.002), (0.026, 0.0), (0.0253, 0.002),
               (0.0253, 0.0082), (0.0232, 0.01), (0, 0.01)], seg=16)
    return noisy(s, 0.0006, scale=90, seed=13)

part('WallPipeLeaks__joint', f_cyl(0.026, 0.026, 0.020), pipe_coupling)


# ============================================================ fuse boxes ==
def fuse_body():
    # enclosure with a dead-front: rim lip, recessed panel, breaker slot, top knockout
    shell = rbox(0.14, 0.2, 0.054, r=0.005, seg=2).move(z=-0.003)
    shell = cut(shell,
                box(0.112, 0.176, 0.01).move(z=0.0275 + 0.0025),
                box(0.02, 0.158, 0.02).move(x=-0.03, z=0.024 + 0.005),
                cyl(0.012, 0.012, 0.004, seg=14, bevel_r=0).move(y=0.1, z=-0.004))
    return fitf(shell, f_box(0.14, 0.20, 0.06))

part('FuseBoxes__body', f_box(0.14, 0.20, 0.06), fuse_body)


def fuse_door():
    # pressed door: skin, folded return flange behind, hinge knuckles (-X), quarter-turn latch (+X)
    skin = rbox(0.14, 0.2, 0.0025, r=0.001, seg=1).move(z=0.0018)
    fl = 0.0012
    flange = (box(0.14, fl, 0.0048).move(y=0.1 - fl / 2, z=-0.0017) + box(0.14, fl, 0.0048).move(y=-0.1 + fl / 2, z=-0.0017)
              + box(fl, 0.2, 0.0048).move(x=0.07 - fl / 2, z=-0.0017) + box(fl, 0.2, 0.0048).move(x=-0.07 + fl / 2, z=-0.0017))
    knuckles = sum((cyl(0.0028, 0.0028, 0.03, seg=8, bevel_r=0.0006).move(x=-0.0705, y=yy, z=-0.001) for yy in (-0.065, 0.065)), Shape())
    latch = lathe([(0, 0.0), (0.0065, 0.0), (0.0065, 0.0005), (0.0055, 0.001), (0, 0.001)], seg=12).rot(x=math.pi / 2).move(x=0.056, z=0.003)
    latch = cut(latch, box(0.0012, 0.009, 0.004).move(x=0.056, z=0.004))
    return fitf(skin + flange + knuckles + latch, f_box(0.14, 0.20, 0.008))

part('FuseBoxes__door', f_box(0.14, 0.20, 0.008), fuse_door)


def breaker_toggle():
    paddle = rbox(0.009, 0.02, 0.0042, r=0.0012, seg=1).move(z=-0.0004)
    ridge = rbox(0.009, 0.0022, 0.0012, r=0.0005, seg=1).move(y=0.0065, z=0.0019)
    return fitf(paddle + ridge, f_box(0.010, 0.020, 0.005))

part('FuseBoxes__switch', f_box(0.010, 0.020, 0.005), breaker_toggle)


# ==================================================== hanging light bulbs ==
def twisted_cord(L, r, lobes=2, depth=0.35, turns_per_m=35.0, rings=40, seg=8):
    """Cloth flex / rope along Y: a lobed section twisting down the length."""
    pts = [(0.0, -L / 2 + L * i / (rings - 1), 0.0) for i in range(rings)]
    def sec(i, t):
        tw = TAU * turns_per_m * L * t
        return [((r * (1 - depth) + r * depth * abs(math.cos(lobes * a / 2 + tw))) * math.cos(a),
                 (r * (1 - depth) + r * depth * abs(math.cos(lobes * a / 2 + tw))) * math.sin(a))
                for a in [TAU * j / seg for j in range(seg)]]
    return sweep(pts, sec)

part('HangingLightBulbs__cord', f_cyl(0.002, 0.002, 0.75), lambda: twisted_cord(0.75, 0.002), smooth=70)

# bakelite lamp holder: cord grip on top, shoulder, ribbed body, shade-ring collar at the mouth
part('HangingLightBulbs__socket', f_cyl(0.014, 0.014, 0.020), lambda: lathe(
    [(0, -0.0085), (0.0095, -0.0085), (0.0108, -0.01), (0.0128, -0.01), (0.0135, -0.0082), (0.0135, -0.0035),
     (0.0127, -0.0025), (0.014, 0.0035), (0.0117, 0.0085), (0.005, 0.0093), (0.0042, 0.01), (0, 0.01)], seg=12))


# ========================================================= ceiling pipes ==
part('CeilingPipes__body', f_cyl(0.024, 0.024, 2.0), lambda: noisy(lathe(
    pipe_profile(0.0215, 0.024, 2.0, 0.016, end_len=0.035, rings_every=0.25), seg=14), 0.0005, scale=50, seed=14))


def pipe_strap():
    # perforated hanger strap: rounded ends, row of holes, slight twist
    s = prism(rounded_rect(0.014, 0.08, 0.004, 2), 0.0022, bevel_r=0.0004, bevel_seg=1)
    s = cut(s, sum((cyl(0.0026, 0.0026, 0.02, seg=8, bevel_r=0).rot(x=math.pi / 2).move(y=yy) for yy in (0.03, 0.012)), Shape()))
    s = s.warp(lambda p: Vector((p.x, p.y, p.z + 0.0009 * math.sin(p.y * 40))))
    return fitf(s, f_box(0.014, 0.08, 0.004))

part('CeilingPipes__strap', f_box(0.014, 0.08, 0.004), pipe_strap)


# ========================================================== safety cones ==
# moulded cone: foot collar, straight taper, rounded top with the throat opening
part('SafetyCones__body', f_cone(0.06, 0.28), lambda: lathe(
    [(0, -0.14), (0.0595, -0.14), (0.06, -0.1385), (0.0585, -0.131), (0.056, -0.127), (0.0555, -0.123),
     (0.0098, 0.133), (0.0088, 0.138), (0.0075, 0.14), (0.0052, 0.14), (0.0047, 0.134), (0, 0.134)], seg=24))
# square base with rounded corners and a chamfered top edge
part('SafetyCones__base', f_box(0.12, 0.010, 0.12), lambda: yloft(
    [(-0.005, 0.0012), (-0.0038, 0.0), (0.001, 0.0), (0.0035, 0.0022), (0.005, 0.0055)], 0.12, 0.12, 0.016, cseg=3))


# ===================================================== wall drain grates ==
def drain_slat():
    # angled louvre slat with an eased front edge (outline in (-z, y), run along X)
    out = [(0.0025, -0.002), (0.0005, -0.002), (-0.0025, 0.0008), (-0.0025, 0.0016), (-0.0018, 0.002), (0.0012, 0.002),
           (0.0025, 0.0005)]
    s = prism(out, 0.08, bevel_r=0.0003, bevel_seg=1).rot(y=math.pi / 2)
    return fitf(s, f_box(0.08, 0.004, 0.005))

part('WallDrainGrates__slot', f_box(0.08, 0.004, 0.005), drain_slat)


# =========================================================== vent grates ==
def vent_frame():
    # stamped register frame: bevelled border rising to the opening, two screws
    s = zloft([(-0.005, 0.0), (-0.0018, 0.0), (0.0008, 0.0035), (0.0035, 0.012), (0.005, 0.019)], 0.30, 0.20, 0.01)
    s = cut(s, box(0.25, 0.15, 0.03))
    screws = sum((lathe([(0, 0), (0.0042, 0), (0.0042, 0.0006), (0.003, 0.0016), (0, 0.0019)], seg=10)
                  .rot(x=math.pi / 2).move(x=sx * 0.1385, z=0.0012) for sx in (-1, 1)), Shape())
    return fitf(s + screws, f_box(0.30, 0.20, 0.010))

part('VentGrates__frame', f_box(0.30, 0.20, 0.010), vent_frame)


def vent_louver():
    # curved sheet blade (bowed toward +Z), rolled front edge
    n = 7
    top = [(0.0035 * math.sin(math.pi * i / (n - 1)) + 0.0005, 0.007 - 0.014 * i / (n - 1)) for i in range(n)]
    bot = [(z - 0.0018, y) for z, y in reversed(top)]
    out = [(y, z) for z, y in top + bot]
    s = prism(out, 0.26, bevel_r=0.0004, bevel_seg=1).rot(y=math.pi / 2)
    return fitf(s, f_box(0.26, 0.014, 0.008))

part('VentGrates__louver', f_box(0.26, 0.014, 0.008), vent_louver)


# ==================================================== emergency lanterns ==
def lantern_bracket():
    # wall plate (-Z), tapered arm, hook curling down at the front for the chain
    plate = rbox(0.010, 0.02, 0.004, r=0.0012, seg=1).move(z=-0.038)
    arm = loft([[(x, y + yc, z) for x, y in rrect_pts(0.007, h, 0.0015, 1)]
                for z, h, yc in ((-0.036, 0.011, 0.002), (0.0, 0.008, 0.004), (0.026, 0.006, 0.005))])
    hook = tube([(0, 0.005, 0.024), (0, 0.0055, 0.033), (0, 0.001, 0.037), (0, -0.005, 0.033), (0, -0.0065, 0.027)],
                0.0022, seg=6, smooth=1)
    return fitf(plate + arm + hook, f_box(0.010, 0.02, 0.08))

part('EmergencyLanterns__bracket', f_box(0.010, 0.02, 0.08), lantern_bracket)


def chain_link():
    pts = []
    for i in range(10):
        a = TAU * i / 10
        pts.append((0.005 * math.cos(a), 0.0048 * math.sin(a) + 0.0012 * math.copysign(1, math.sin(a)) * (abs(math.sin(a)) > 0.3), 0.0))
    return sweep(pts, lambda i, t: ellipse(0.001, 0.001, 5), bnorm=(0, 0, 1), closed=True)

part('EmergencyLanterns__link', f_torus(0.006, 0.001), chain_link, smooth=60)


def lantern_body():
    # housing on a base plate, domed lid, raised bezel round the lens, side fins
    body = rbox(0.060, 0.074, 0.050, r=0.004, seg=1).move(y=-0.004, z=-0.0015)
    base = rbox(0.065, 0.006, 0.055, r=0.0015, seg=1).move(y=-0.042)
    lid = yloft([(0.033, 0.0), (0.038, 0.0015), (0.043, 0.006), (0.045, 0.012)], 0.063, 0.053, 0.006, cseg=2)
    bezel = sum((box(w, h, 0.0035).move(x=x, y=y - 0.001, z=0.0258) for w, h, x, y in
                 ((0.058, 0.006, 0, 0.0305), (0.058, 0.006, 0, -0.0325), (0.006, 0.069, 0.026, -0.001), (0.006, 0.069, -0.026, -0.001))),
                Shape())
    fins = sum((box(0.004, 0.003, 0.044).move(x=sx * 0.0305, y=yy, z=-0.0015) for sx in (-1, 1) for yy in (-0.018, -0.006, 0.006)), Shape())
    return fitf(body + base + lid + bezel + fins, f_box(0.065, 0.09, 0.055))

part('EmergencyLanterns__body', f_box(0.065, 0.09, 0.055), lantern_body)


# ========================================================= metal buckets ==
def metal_pail():
    # galvanised pail: bottom chime, pressed rib, flared wall, rolled rim, floor inside
    prof = [(0, -0.055), (0.0555, -0.055), (0.0584, -0.0515), (0.0655, -0.0125), (0.0673, -0.0095), (0.0673, -0.0045),
            (0.0668, -0.0012), (0.0736, 0.047), (0.075, 0.055), (0.0735, 0.0568), (0.0716, 0.05), (0.0635, -0.008),
            (0.0556, -0.049), (0, -0.049)]
    return fitf(lathe(prof, seg=20, cap_bottom=False, cap_top=False), f_cyl(0.075, 0.058, 0.11))

part('MetalBuckets__body', f_cyl(0.075, 0.058, 0.11), metal_pail)
part('MetalBuckets__rim', f_torus(0.075, 0.005), lambda: torus(0.075, 0.005, seg=36, tseg=6))
part('MetalBuckets__handle', f_bounds((-0.085, -0.003, -0.003), (0.085, 0.085, 0.003)),
     lambda: bail(0.082, 0.0018, grip=(0.003, 0.14)), smooth=50)


def pail_ear():
    # riveted lug plate (rounded top) with a rivet head on both faces
    out = [(-0.003, -0.005), (0.003, -0.005)] + [(0.003 * math.cos(math.pi * i / 5), 0.002 + 0.003 * math.sin(math.pi * i / 5)) for i in range(6)]
    plate = prism(out, 0.004).rot(y=math.pi / 2)
    rivets = sum((lathe([(0, 0), (0.0018, 0), (0.0014, 0.0014), (0, 0.002)], seg=6).rot(z=-sx * math.pi / 2).move(x=sx * 0.002, y=-0.001)
                  for sx in (-1, 1)), Shape())
    return fitf(plate + rivets, f_box(0.008, 0.010, 0.006))

part('MetalBuckets__ear', f_box(0.008, 0.010, 0.006), pail_ear)


# ========================================================= tool pegboards ==
# hardboard panel, eased edges (the holes stay in the texture)
part('ToolPegboards__board', f_box(0.55, 0.42, 0.008), lambda: zloft(
    [(-0.004, 0.0008), (-0.0032, 0.0), (0.0032, 0.0), (0.004, 0.0008)], 0.55, 0.42, 0.006))


def peg_hammer_head():
    # claw-hammer head in side profile (face toward +Z, split claw toward the board)
    out = [(0.0275, -0.0055), (0.0275, 0.0055), (0.024, 0.0062), (0.016, 0.0045), (0.007, 0.0075), (0.007, 0.0125),
           (-0.007, 0.0125), (-0.009, 0.0095), (-0.018, 0.005), (-0.0265, -0.002), (-0.0275, -0.005), (-0.024, -0.0055),
           (-0.016, -0.0045), (-0.007, -0.0095), (-0.007, -0.0125), (0.007, -0.0125), (0.007, -0.0075), (0.016, -0.0045),
           (0.024, -0.0062)]
    s = prism(out, 0.008, bevel_r=0.0008, bevel_seg=1).rot(y=-math.pi / 2)
    return fitf(s, f_box(0.008, 0.025, 0.055))

part('ToolPegboards__hammerHead', f_box(0.008, 0.025, 0.055), peg_hammer_head)
part('ToolPegboards__hammerHandle', f_cyl(0.005, 0.005, 0.11), lambda: lathe(
    [(0, -0.055), (0.0042, -0.055), (0.005, -0.052), (0.005, -0.046), (0.0044, -0.043), (0.0047, -0.012),
     (0.0038, 0.02), (0.0034, 0.055), (0, 0.055)], seg=10))


def peg_wrench():
    # shank: thin bar, waisted in the middle
    rings = []
    for y, k in ((-0.05, 1.0), (-0.03, 0.86), (0.0, 0.8), (0.03, 0.86), (0.05, 1.0)):
        rings.append([(x, y, z) for x, z in rrect_pts(0.006 * (0.7 + 0.3 * k), 0.010 * k, 0.0015, 1)])
    return loft(rings)

part('ToolPegboards__wrench', f_box(0.006, 0.10, 0.010), peg_wrench)


def peg_wrench_head():
    # open-end jaw, opening angled 15 degrees
    head = prism([(0.014 * math.cos(TAU * i / 18), 0.010 * math.sin(TAU * i / 18)) for i in range(18)], 0.008,
                 bevel_r=0.0007, bevel_seg=1)
    jaw = box(0.0115, 0.02, 0.03).rot(z=0.26).move(x=0.0015, y=0.0085)
    return fitf(cut(head, jaw), f_box(0.028, 0.020, 0.010))

part('ToolPegboards__wrenchHead', f_box(0.028, 0.020, 0.010), peg_wrench_head)


def sd_handle():
    # fluted screwdriver grip: domed butt (+Y), six flutes, ferrule taper to the shaft
    ys = [-0.0275, -0.023, -0.012, 0.008, 0.02, 0.025, 0.0275]
    rs = [0.0042, 0.0074, 0.0084, 0.0093, 0.0099, 0.0088, 0.0055]
    angles = [TAU * i / 18 for i in range(18)]
    def rad(j, a, y):
        fl = 1 - (0.13 if round(a / TAU * 18) % 3 == 0 else 0.0) * (1 if 1 <= j <= 4 else 0)
        return rs[j] * fl, y
    return grid_lathe(rad, ys, angles, top=0.0275, bottom=-0.0275)

part('ToolPegboards__sdHandle', f_cyl(0.010, 0.008, 0.055), sd_handle)


def sd_shaft():
    s = lathe([(0, -0.03), (0.0024, -0.03), (0.003, -0.027), (0.003, -0.021), (0.003, 0.03), (0, 0.03)], seg=8)
    def blade(p):
        if p.y < -0.021:
            k = (-0.021 - p.y) / 0.009
            p.z *= 1 - 0.7 * k
            p.x *= 1 + 0.15 * k
        return p
    return fitf(s.warp(blade), f_cyl(0.003, 0.003, 0.060))

part('ToolPegboards__sdShaft', f_cyl(0.003, 0.003, 0.060), sd_shaft)


def peg_pliers():
    # two handles diverging from the joint, dipped grips thickening toward the ends
    s = Shape()
    for sx in (-1, 1):
        pts = [(sx * (0.0014 + 0.0024 * (i / 6) ** 1.3), 0.05 - 0.1 * i / 6, 0.0) for i in range(7)]
        s = s + sweep(pts, lambda i, t: ellipse(0.0018 + 0.0006 * t, 0.0036 + 0.0008 * t, 6))
    return fitf(s, f_box(0.010, 0.10, 0.010))

part('ToolPegboards__pliers', f_box(0.010, 0.10, 0.010), peg_pliers)


def peg_pliers_head():
    # tapering jaws with the split line, pivot rivet through the joint
    out = [(-0.01, -0.015), (0.01, -0.015), (0.009, -0.006), (0.0045, 0.007), (0.002, 0.015), (-0.002, 0.015),
           (-0.0045, 0.007), (-0.009, -0.006)]
    jaws = prism(out, 0.007, bevel_r=0.0007, bevel_seg=1)
    jaws = cut(jaws, box(0.0008, 0.022, 0.02).move(y=0.006))
    rivet = cyl(0.0042, 0.0042, 0.01, seg=10, bevel_r=0.001).rot(x=math.pi / 2).move(y=-0.008)
    return fitf(jaws + rivet, f_box(0.020, 0.030, 0.010))

part('ToolPegboards__pliersHead', f_box(0.020, 0.030, 0.010), peg_pliers_head)


# ============================================================ rustic signs ==
def sign_planks():
    # three worn planks (gaps on the texture's plank lines), nail heads at the ends
    rnd = random.Random(21)
    s = Shape()
    for k, yc in enumerate((-0.05, 0.0, 0.05)):
        pl = rbox(0.296 - rnd.uniform(0, 0.006), 0.0478, 0.0125, r=0.0022, seg=2)
        s = s + pl.rot(z=rnd.uniform(-0.012, 0.012)).move(x=rnd.uniform(-0.002, 0.002), y=yc, z=-0.001 + rnd.uniform(-0.0008, 0.0))
        for sx in (-1, 1):
            s = s + lathe([(0, 0), (0.0026, 0), (0.0022, 0.0009), (0, 0.0013)], seg=7).rot(x=math.pi / 2).move(
                x=sx * 0.128, y=yc + rnd.uniform(-0.004, 0.004), z=0.0052)
    return fitf(s, f_box(0.30, 0.15, 0.015))

part('RusticSigns__panel', f_box(0.30, 0.15, 0.015), sign_planks)
part('RusticSigns__rope', f_cyl(0.002, 0.002, 0.10), lambda: twisted_cord(
    0.10, 0.002, lobes=3, depth=0.3, turns_per_m=60.0, rings=15, seg=9), smooth=70)


# =========================================================== wooden buckets ==
def stave_bucket():
    # twelve staves (joints on the placer's stave lines), end grain at the rim, floor inside
    prof = [(0.0685, -0.07), (0.0712, -0.0665), (0.0752, 0.0), (0.08, 0.07)]
    s = vessel(prof, 0.0055, seg=36, floor=0.012)
    def staves(p):
        r = math.hypot(p.x, p.z)
        if r < 1e-6:
            return p
        a = math.atan2(p.x, p.z) % TAU
        ph = (a / (TAU / 12)) % 1.0
        d = min(ph, 1 - ph)
        k = 1 - 0.022 * math.exp(-(d / 0.07) ** 2) + 0.006 * math.sin(math.pi * ph)
        return Vector((p.x * k, p.y, p.z * k))
    return fitf(s.warp(staves), f_cyl(0.080, 0.070, 0.14))

part('WoodenBuckets__body', f_cyl(0.080, 0.070, 0.14), stave_bucket, smooth=40)


def iron_hoop():
    sec = [(u, v) for u, v in rrect_pts(0.008, 0.0072, 0.0016, 1)]
    return sweep(arc_pts(0.079, 0.0, TAU, 33)[:-1], lambda i, t: sec, bnorm=(0, 0, 1), closed=True)

part('WoodenBuckets__band', f_torus(0.079, 0.004), iron_hoop, smooth=50)
part('WoodenBuckets__handle', f_bounds((-0.088, -0.003, -0.003), (0.088, 0.088, 0.003)),
     lambda: bail(0.085, 0.0022), smooth=50)
