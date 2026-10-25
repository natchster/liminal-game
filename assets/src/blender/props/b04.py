"""b04 — office props: fountains, boards, clipboards, cabinets, first aid,
phones, briefcases, laptops, desk and game clutter.

Every part is modelled in its primitive's own frame (see README.md). Parts
that stand for a plane (boards, doors, paper) keep their front at z = 0 and
put their thickness behind it.
"""
import propkit
from propkit import *

PI = math.pi
_rc = propkit._round_corner


# ---------------------------------------------------------------- helpers --
def ribs(s, n, depth, ymin=-1e9, ymax=1e9, rmin=0.0):
    """Vertical ribs (knurl) round the Y axis: pull every vertex between
    ymin..ymax and further out than rmin in by up to `depth`."""
    def f(p):
        r = math.hypot(p.x, p.z)
        if ymin <= p.y <= ymax and r > rmin and r > 1e-7:
            k = (0.5 + 0.5 * math.cos(n * math.atan2(p.z, p.x))) * depth
            p.x *= (r - k) / r
            p.z *= (r - k) / r
        return p
    return s.warp(f)


def side_prism(outline_zy, width):
    """Extrude a side profile [(z, y), ...] along X (centred, `width` wide)."""
    return prism(outline_zy, width).rot(y=-PI / 2)


def plan_prism(outline_xz, height):
    """Extrude a plan outline [(x, z), ...] along Y (centred)."""
    return prism([(x, -z) for x, z in outline_xz], height).rot(x=-PI / 2)


def plan_ring(w, d, r, depth, h, seg=3):
    """A thin ring cutter round a rounded-rect plan (w × d, corner r): cuts a
    groove `depth` deep and `h` tall round the sides only."""
    outer = plan_prism(rounded_rect(w + 0.02, d + 0.02, r + 0.01, seg), h)
    inner = plan_prism(rounded_rect(w - 2 * depth, d - 2 * depth, max(r - depth, 0.0005), seg), h * 3)
    return cut(outer, inner)


def bezier(p0, p1, p2, n=10):
    return [tuple(p0[k] * (1 - t) ** 2 + 2 * p1[k] * (1 - t) * t + p2[k] * t * t for k in range(2))
            for t in (i / n for i in range(n + 1))]


def strip(curve, t):
    """A 2D band of thickness t below a polyline (outline for prism)."""
    inner = []
    for i, (x, y) in enumerate(curve):
        a = curve[max(i - 1, 0)]
        b = curve[min(i + 1, len(curve) - 1)]
        dx, dy = b[0] - a[0], b[1] - a[1]
        l = math.hypot(dx, dy) or 1
        inner.append((x + dy / l * t, y - dx / l * t))
    return curve + inner[::-1]


def sheet(w, h, t, nx, ny, lift):
    """Thin sheet (front at z = 0, thickness behind), warped by lift(u, v)
    with u, v in 0..1 across width / height (z offset in metres)."""
    s = slab(w, h, t, nx, ny).move(z=-t / 2)
    def f(p):
        p.z += lift(p.x / w + 0.5, p.y / h + 0.5)
        return p
    return s.warp(f)


# -------------------------------------------------------- water fountain --
def fountain_body():
    # stainless wall cabinet: rounded front corners, sloped under-apron
    r = 0.03
    out = [(-0.14, 0.10), (0.14, 0.10)] + _rc((0.14, -0.10), (0.14, 0.10), (-0.14, -0.10), r, 4) \
        + _rc((-0.14, -0.10), (0.14, -0.10), (-0.14, 0.10), r, 4)
    s = prism(out, 0.38).rot(x=-PI / 2)         # plan (x, -z) → extrude along y
    apron = side_prism([(0.0, -0.2075), (0.13, -0.094), (0.13, -0.26), (0.0, -0.26)], 0.4)
    # louvre slots across the sloped apron, and a seam under the top cap
    vents = [box(0.17, 0.006, 0.008).rot(x=math.atan2(0.08, 0.07))
             .move(y=-0.19 + 0.07 * t, z=0.02 + 0.08 * t) for t in (0.3, 0.52, 0.74)]
    seam = box(0.3, 0.0016, 0.004).move(y=0.135, z=0.1)
    s = cut(s, apron, seam, *vents)
    return bevel(s, 0.003, seg=2, angle=35)


def fountain_basin():
    s = rbox(0.22, 0.03, 0.14, r=0.006, seg=2)
    bowl = ellipsoid(0.088, 0.024, 0.046, seg=20).move(y=0.017, z=0.012)
    drain = cyl(0.006, 0.006, 0.03, seg=10).move(y=-0.004, z=0.014)
    return cut(s, bowl, drain)


def fountain_spout():
    prof = [(0, -0.03), (0.012, -0.03), (0.012, -0.027), (0.0095, -0.024), (0.0078, -0.018),
            (0.0074, 0.012), (0.008, 0.02), (0.0078, 0.027), (0.0055, 0.03), (0.0022, 0.03),
            (0.0018, 0.026), (0, 0.026)]
    return lathe(prof, seg=12)


def fountain_button():
    prof = [(0, -0.004), (0.02, -0.004), (0.02, -0.0015), (0.0185, -0.0005), (0.0162, -0.0002),
            (0.0158, 0.002), (0.0148, 0.0034), (0.009, 0.0039), (0, 0.004)]
    return lathe(prof, seg=20)


part('WaterFountains__body', f_box(0.28, 0.38, 0.20), fountain_body)
part('WaterFountains__basin', f_box(0.22, 0.03, 0.14), fountain_basin)
part('WaterFountains__spout', f_cyl(0.008, 0.012, 0.06), fountain_spout)
part('WaterFountains__button', f_cyl(0.02, 0.02, 0.008), fountain_button)


# ------------------------------------------------------- bulletin board --
def cork_base():
    # a 12 mm cork sheet on a backing board, front at z = 0
    return rbox(1.0, 0.6, 0.012, r=0.002, seg=1).move(z=-0.006)


def board_note():
    # a pinned note: flat at the pin, the free bottom edge curling off the cork
    def lift(u, v):
        lo = max(0.0, 0.55 - v) / 0.55
        return 0.0028 * lo * lo + 0.0010 * lo * u + 0.0004 * math.sin(u * 5.0 + v * 3.0)
    return sheet(0.17, 0.13, 0.0003, 8, 6, lift)


part('BulletinBoards__cork', f_bounds((-0.5, -0.3, 0), (0.5, 0.3, 0), 'plane'), cork_base)
part('BulletinBoards__framebar', f_bounds((-0.52, -0.015, 0), (0.52, 0.015, 0), 'plane'),
     lambda: box(1.04, 0.03, 0.02).move(z=-0.01))
part('BulletinBoards__note', f_bounds((-0.085, -0.065, 0), (0.085, 0.065, 0), 'plane'), board_note)


# ------------------------------------------------------------ clipboard --
def clipboard_board():
    s = prism(rounded_rect(0.20, 0.28, 0.012, seg=3), 0.008).rot(x=-PI / 2)
    s = cut(s, cyl(0.0055, 0.0055, 0.02, seg=14).move(z=-0.128))
    return bevel(s, 0.0014, seg=1, angle=40)


def clipboard_paper():
    # a few sheets under the clip, the free bottom corner lifting
    def lift(u, v):
        lo = max(0.0, 0.35 - v) / 0.35
        return 0.0035 * lo * lo * (0.35 + 0.65 * u)
    return sheet(0.18, 0.24, 0.0012, 6, 8, lift)


def clipboard_clip():
    # lever clip seen from the side: base plate, hinge roll, sprung jaw
    base = box(0.06, 0.0014, 0.014).move(y=-0.0053, z=-0.003)
    base = bevel(base, 0.0005, seg=1)
    roll = cyl(0.0026, 0.0026, 0.056, seg=8).rot(z=PI / 2).move(y=-0.0034, z=-0.0072)
    jaw = strip(bezier((-0.0075, -0.0008), (-0.0015, 0.0142), (0.0098, -0.0052), 8), 0.0011)
    jaw = side_prism(jaw, 0.05)
    lip = cyl(0.0008, 0.0008, 0.05, seg=5, bevel_r=0).rot(z=PI / 2).move(y=-0.0052, z=0.0092)
    return base + roll + jaw + lip


def pen():
    prof = [(0, -0.055), (0.0006, -0.0547), (0.0009, -0.053), (0.0017, -0.0508), (0.0033, -0.043),
            (0.0036, -0.040), (0.0036, -0.027), (0.0040, -0.0255), (0.0040, 0.049), (0.0037, 0.0525),
            (0.0029, 0.0545), (0, 0.055)]
    return lathe(prof, seg=10)


part('Clipboards__board', f_box(0.20, 0.008, 0.28), clipboard_board)
part('Clipboards__paper', f_bounds((-0.09, -0.12, 0), (0.09, 0.12, 0), 'plane'), clipboard_paper)
part('Clipboards__clip', f_box(0.06, 0.012, 0.02), clipboard_clip)
part('Clipboards__pen', f_cyl(0.004, 0.004, 0.11), pen)


# ------------------------------------------------------------ first aid --
def firstaid_body():
    # wall cabinet: rounded shell, a door seam round the front, hinge knuckles
    # on the left, a turn lock on the right (the cross and label are decals)
    s = rbox(0.20, 0.18, 0.10, r=0.005, seg=2)
    g, d, ix, iy = 0.0014, 0.004, 0.092, 0.082
    seam = [box(2 * ix + g, g, d).move(y=sy * iy, z=0.05) for sy in (-1, 1)] + \
           [box(g, 2 * iy + g, d).move(x=sx * ix, z=0.05) for sx in (-1, 1)]
    s = cut(s, *seam)
    knuckles = sum((cyl(0.0034, 0.0034, 0.028, seg=8).move(x=-0.0985, y=y, z=0.0465)
                    for y in (-0.05, 0.05)), Shape())
    lock = cyl(0.0065, 0.0065, 0.004, seg=10).rot(x=PI / 2).move(x=0.078, z=0.05)
    lock = cut(lock, box(0.0085, 0.0014, 0.004).move(x=0.078, z=0.0535))
    return s + knuckles + lock


def pill_bottle():
    prof = [(0, -0.03), (0.017, -0.03), (0.0195, -0.0285), (0.02, -0.026), (0.02, 0.026),
            (0.0185, 0.0268), (0.0178, 0.03), (0, 0.03)]
    return lathe(prof, seg=20)


def pill_cap():
    prof = [(0, -0.0075), (0.0215, -0.0075), (0.022, -0.0066), (0.022, 0.0056), (0.0196, 0.0075),
            (0, 0.0075)]
    return ribs(lathe(prof, seg=24), 12, 0.0011, -0.007, 0.006, rmin=0.019)


def pill():
    prof = [(0, -0.002), (0.0035, -0.0017), (0.0055, -0.0009), (0.006, 0.0), (0.0055, 0.0009),
            (0.0035, 0.0017), (0, 0.002)]
    return lathe(prof, seg=10)


def syringe_barrel():
    # needle end at -y (luer tip), finger flange at +y
    prof = [(0, -0.03), (0.0016, -0.03), (0.0016, -0.026), (0.0048, -0.0235), (0.0056, -0.022),
            (0.0056, 0.0265), (0.007, 0.0272), (0.007, 0.0295), (0.0045, 0.03), (0, 0.03)]
    return lathe(prof, seg=12)


def syringe_needle():
    prof = [(0, -0.02), (0.0004, -0.018), (0.0005, 0.012), (0.001, 0.013), (0.001, 0.02), (0, 0.02)]
    return lathe(prof, seg=6)


def syringe_plunger():
    prof = [(0, -0.004), (0.0028, -0.004), (0.0028, 0.0015), (0.008, 0.0016), (0.008, 0.0033),
            (0.0071, 0.004), (0, 0.004)]
    return lathe(prof, seg=8)


def goggle_rim():
    # moulded eyecup rim: flat back, rounded front lip (ring round Z)
    R, r = 0.024, 0.004
    prof = [(R - r * 0.7, -r), (R + r, -r), (R + r, -r * 0.2), (R + r * 0.7, r * 0.7), (R + r * 0.1, r),
            (R - r * 0.6, r * 0.6), (R - r, r * 0.1), (R - r, -r * 0.6)]
    ring = lathe(prof + [prof[0]], seg=16, cap_bottom=False, cap_top=False)
    return weld(ring).rot(x=PI / 2)


def goggle_bridge():
    # a small arched nose bridge
    s = prism(strip(bezier((-0.006, -0.0025), (0.0, 0.0055), (0.006, -0.0025), 6), 0.0022), 0.008)
    return s.fit((-0.006, -0.0025, -0.004), (0.006, 0.0025, 0.004))


part('FirstAidKits__body', f_box(0.20, 0.18, 0.10), firstaid_body)
part('FirstAidKits__bottle', f_cyl(0.02, 0.02, 0.06), pill_bottle)
part('FirstAidKits__cap', f_cyl(0.022, 0.022, 0.015), pill_cap)
part('FirstAidKits__pill', f_cyl(0.006, 0.006, 0.004), pill)
part('FirstAidKits__barrel', f_cyl(0.007, 0.007, 0.06), syringe_barrel)
part('FirstAidKits__needle', f_cyl(0.001, 0.001, 0.04), syringe_needle)
part('FirstAidKits__plunger', f_cyl(0.008, 0.008, 0.008), syringe_plunger)
part('FirstAidKits__rim', f_torus(0.024, 0.004), goggle_rim)
part('FirstAidKits__bridge', f_box(0.012, 0.005, 0.008), goggle_bridge)
part('FirstAidKits__strap', f_cyl(0.002, 0.002, 0.14),
     lambda: cyl(0.002, 0.002, 0.14, seg=8, bevel_r=0.0008))


# -------------------------------------------------------- cabinet doors --
def cabinet_door():
    # two-panel cupboard door: stiles and rails flush at z = 0, the panels
    # sunk 6 mm with a raised, bevelled field in each
    s = box(0.68, 2.0, 0.012).move(z=-0.006)
    panels = [(-0.10, 0.91), (-0.88, -0.18)]
    s = cut(s, *[box(0.53, y1 - y0, 0.012).move(y=(y0 + y1) / 2) for y0, y1 in panels])
    s = bevel(s, 0.004, seg=1, angle=30)
    for y0, y1 in panels:
        f = box(0.53 - 0.03, y1 - y0 - 0.03, 0.0055)
        def lean(p):
            if p.z > 0:
                p.x -= math.copysign(0.016, p.x)
                p.y -= math.copysign(0.016, p.y)
            return p
        s = s + f.warp(lean).move(y=(y0 + y1) / 2, z=-0.0045)
    return s


def door_trim():
    # ogee moulding strip, thickness behind the face
    prof = [(-0.013, -0.015), (-0.005, -0.015), (-0.0015, -0.0125), (0.0, -0.008), (0.0, -0.002),
            (-0.001, 0.004), (-0.004, 0.009), (-0.008, 0.013), (-0.010, 0.015), (-0.013, 0.015)]
    return side_prism(prof, 0.68)


def bar_pull():
    ros = [cyl(0.0075, 0.0075, 0.002, seg=10, bevel_r=0).rot(x=PI / 2).move(y=y, z=-0.0065) for y in (-0.036, 0.036)]
    posts = [cyl(0.0028, 0.0028, 0.009, seg=8, bevel_r=0).rot(x=PI / 2).move(y=y, z=-0.001) for y in (-0.036, 0.036)]
    bar = capsule(0.0045, 0.091, seg=8, cap_rings=3).move(z=0.003)
    return sum(ros + posts, Shape()) + bar


def tin_can():
    prof = [(0, -0.037), (0.0212, -0.037), (0.024, -0.0395), (0.024, -0.035), (0.0232, -0.033),
            (0.0232, 0.033), (0.024, 0.035), (0.024, 0.0395), (0.0212, 0.037), (0, 0.037)]
    return lathe(prof, seg=14)


def carton():
    s = rbox(0.05, 0.07, 0.04, r=0.0015, seg=1)
    return cut(s, box(0.054, 0.0009, 0.002).move(y=0.027, z=0.02))


def toolbox_body():
    main = bevel(plan_prism(rounded_rect(0.236, 0.116, 0.010, seg=2), 0.088).move(y=-0.006), 0.0015, seg=1, angle=40)
    lip = plan_prism(rounded_rect(0.24, 0.12, 0.012, seg=3), 0.012).move(y=0.044)
    grooves = [plan_ring(0.236, 0.116, 0.010, 0.0012, 0.0016, seg=2).move(y=y) for y in (-0.028, 0.012)]
    return cut(main, *grooves) + bevel(lip, 0.002, seg=1, angle=40)


def toolbox_lid_open():
    s = plan_prism(rounded_rect(0.245, 0.125, 0.012, seg=3), 0.010)
    s = cut(s, plan_prism(rounded_rect(0.237, 0.117, 0.009, seg=3), 0.010).move(y=-0.0015))
    return bevel(s, 0.0012, seg=1, angle=40)


def toolbox_lid_top():
    plate = plan_prism(rounded_rect(0.245, 0.125, 0.012, seg=3), 0.0028).move(y=-0.0006)
    emboss = rbox(0.19, 0.0026, 0.075, r=0.001, seg=1).move(y=0.0007)
    return bevel(plate, 0.0008, seg=1, angle=40) + emboss


def screwdriver():
    prof = [(0, -0.03), (0.0009, -0.0298), (0.0012, -0.027), (0.0012, -0.004), (0.0034, -0.0028),
            (0.0045, -0.0005), (0.005, 0.004), (0.005, 0.026), (0.0042, 0.0292), (0, 0.03)]
    s = ribs(lathe(prof, seg=12), 6, 0.0009, 0.0, 0.027, rmin=0.004)
    def blade(p):
        if p.y < -0.025:
            p.x *= 1.4
            p.z *= 0.35
        return p
    return s.warp(blade)


def spanner():
    ring = cut(cyl(0.007, 0.007, 0.0036, seg=10, bevel_r=0), cyl(0.0036, 0.0036, 0.01, seg=8, bevel_r=0)).move(x=-0.013)
    head = cut(cyl(0.007, 0.007, 0.0036, seg=10, bevel_r=0),
               box(0.012, 0.01, 0.0056).move(x=0.006)).move(x=0.013)
    shaft = box(0.022, 0.0034, 0.0056)
    s = ring + head + shaft
    def crank(p):
        p.y += 0.0044 * min(1.0, max(0.0, (-0.006 - p.x) / 0.006)) - 0.0022
        return p
    return s.warp(crank)


def carry_handle():
    prof = [(0, -0.07), (0.004, -0.07), (0.0045, -0.068), (0.0045, -0.058), (0.006, -0.055)]
    for y in (-0.03, -0.01, 0.01, 0.03):
        prof += [(0.006, y - 0.004), (0.0052, y), (0.006, y + 0.004)]
    prof += [(0.006, 0.055), (0.0045, 0.058), (0.0045, 0.068), (0.004, 0.07), (0, 0.07)]
    return lathe(prof, seg=10)


def toolbox_latch():
    plate = bevel(box(0.02, 0.015, 0.0018), 0.0005, seg=1).move(z=-0.0021)
    hasp = prism([(-0.006, -0.0075), (0.006, -0.0075), (0.0045, 0.006), (-0.0045, 0.006)], 0.0042).move(z=0.0009)
    return plate + hasp


def wrench_handle():
    prof = [(0, -0.06), (0.004, -0.0595), (0.0055, -0.057), (0.006, -0.05), (0.0052, 0.05),
            (0.0056, 0.056), (0.0056, 0.06), (0, 0.06)]
    return lathe(prof, seg=8)


def knurl_ring():
    prof = [(0, -0.004), (0.0088, -0.004), (0.01, -0.0025), (0.0092, 0), (0.01, 0.0025), (0.0088, 0.004), (0, 0.004)]
    return lathe(prof, seg=10)


part('CabinetDoors__door', f_bounds((-0.34, -1.0, 0), (0.34, 1.0, 0), 'plane'), cabinet_door)
part('CabinetDoors__trim', f_bounds((-0.34, -0.015, 0), (0.34, 0.015, 0), 'plane'), door_trim)
part('CabinetDoors__handle', f_box(0.015, 0.10, 0.015), bar_pull)
part('CabinetDoors__shelf', f_box(0.38, 0.006, 0.14), lambda: rbox(0.38, 0.006, 0.14, r=0.0015, seg=1))
part('CabinetDoors__can', f_cyl(0.024, 0.024, 0.08), tin_can)
part('CabinetDoors__carton', f_box(0.05, 0.07, 0.04), carton)
part('CabinetDoors__toolbox', f_box(0.24, 0.10, 0.12), toolbox_body)
part('CabinetDoors__lidopen', f_box(0.245, 0.010, 0.125), toolbox_lid_open)
part('CabinetDoors__lidtop', f_box(0.245, 0.004, 0.125), toolbox_lid_top)
part('CabinetDoors__screwdriver', f_cyl(0.005, 0.005, 0.06), screwdriver)
part('CabinetDoors__spanner', f_box(0.04, 0.008, 0.014), spanner)
part('CabinetDoors__carryhandle', f_cyl(0.006, 0.006, 0.14), carry_handle)
part('CabinetDoors__latch', f_box(0.020, 0.015, 0.006), toolbox_latch)
part('CabinetDoors__wrench', f_cyl(0.006, 0.006, 0.12), wrench_handle)
part('CabinetDoors__jaw', f_box(0.020, 0.005, 0.008), lambda: rbox(0.020, 0.005, 0.008, r=0.0009, seg=1))
part('CabinetDoors__knurl', f_cyl(0.010, 0.010, 0.008), knurl_ring)


# ----------------------------------------------------------- wall phone --
def phone_body():
    # wall set: rounded shell with a cradle channel along the top front edge
    # (the handset rests in it between two horns), a hook-switch plunger
    s = rbox(0.14, 0.20, 0.06, r=0.008, seg=2)
    chan = cyl(0.0215, 0.0215, 0.092, seg=16, bevel_r=0).rot(z=PI / 2).move(y=0.10, z=0.04)
    s = cut(s, chan)
    s = bevel(s, 0.0015, seg=1, angle=35)
    plunger = rbox(0.016, 0.012, 0.012, r=0.002, seg=1).move(y=0.078, z=0.028)
    jack = cyl(0.004, 0.004, 0.004, seg=8, bevel_r=0).move(x=0.02, y=-0.1, z=0.012)
    return s + plunger + jack


def handset(h, r_top, r_bot):
    # G-style handset: earpiece (+y) and mouthpiece (-y) cups flaring towards
    # their faces (-z), joined along the back by an oval grip
    def cup(r):
        prof = [(0, -r * 0.86), (r * 0.62, -r * 0.9), (r * 0.9, -r), (r, -r * 0.8), (r * 0.97, -r * 0.2),
                (r * 0.8, r * 0.35), (r * 0.58, r), (0, r)]
        return lathe(prof, seg=14).rot(x=PI / 2)
    rm = min(r_top, r_bot)
    yt, yb = h / 2 - r_top, -h / 2 + r_bot
    grip = tube([(0, yb, rm * 0.55), (0, yb * 0.4, rm * 0.68), (0, yt * 0.4, rm * 0.68), (0, yt, rm * 0.55)],
                rm * 0.46, seg=10, smooth=1).scale(1.35, 1, 1)
    return cup(r_top).move(y=yt) + cup(r_bot).move(y=yb) + grip


def coil_cord():
    # 1.5 turns of tightly wound cord (ring round Z, advancing along Z)
    pts = []
    n = 20
    for i in range(n + 1):
        a = TAU * 1.5 * i / n
        pts.append((0.015 * math.cos(a), 0.015 * math.sin(a), -0.0017 + 0.0034 * i / n))
    return tube(pts, 0.0013, seg=5)


part('WallPhones__body', f_box(0.14, 0.20, 0.06), phone_body)
part('WallPhones__handset', f_cyl(0.015, 0.015, 0.16), lambda: handset(0.16, 0.015, 0.015))
part('WallPhones__handsetrest', f_cyl(0.02, 0.015, 0.13), lambda: handset(0.13, 0.02, 0.0155))
part('WallPhones__coil', f_torus(0.015, 0.003), coil_cord)


# ------------------------------------------------------------ briefcase --
def briefcase_body():
    # leather case lying flat: soft corners, a stitched groove round the sides
    s = plan_prism(rounded_rect(0.36, 0.28, 0.022, seg=4), 0.10)
    s = cut(s, plan_ring(0.36, 0.28, 0.022, 0.0014, 0.0016, seg=4).move(y=-0.012))
    return bevel(s, 0.006, seg=2, angle=40)


def briefcase_rim():
    # the lid seam: piping round the edge and the top panel inside it
    out = rounded_rect(0.352, 0.272, 0.019, seg=4)
    path = [(x, 0.0, -z) for x, z in out] + [(out[0][0], 0.0, -out[0][1])]
    piping = tube(path, 0.004, seg=8)
    top = plan_prism(rounded_rect(0.35, 0.27, 0.018, seg=4), 0.006).move(y=0.0005)
    return piping + bevel(top, 0.0015, seg=1, angle=40)


def briefcase_clasp():
    plate = rbox(0.03, 0.003, 0.02, r=0.001, seg=1).move(y=-0.0035)
    catch = rbox(0.021, 0.0065, 0.014, r=0.0018, seg=1).move(y=0.0015, z=0.001)
    catch = cut(catch, box(0.009, 0.004, 0.004).move(y=0.005, z=0.006))
    return plate + catch


def briefcase_handle():
    loop = tube([(-0.05, 0, 0), (-0.05, 0.022, 0), (-0.041, 0.046, 0), (-0.02, 0.05, 0), (0.02, 0.05, 0),
                 (0.041, 0.046, 0), (0.05, 0.022, 0), (0.05, 0, 0)], 0.0036, seg=7, smooth=1)
    grip = capsule(0.005, 0.05, seg=8, cap_rings=2).rot(z=PI / 2).move(y=0.05)
    feet = sum((rbox(0.012, 0.006, 0.01, r=0.0015, seg=1).move(x=sx * 0.05, y=0.003) for sx in (-1, 1)), Shape())
    return loop + grip + feet


def briefcase_lid():
    s = plan_prism(rounded_rect(0.36, 0.28, 0.022, seg=4), 0.02)
    s = cut(s, plan_prism(rounded_rect(0.348, 0.268, 0.017, seg=4), 0.02).move(y=-0.004))
    return bevel(s, 0.0035, seg=2, angle=40)


def paper_scrap():
    def lift(u, v):
        n = noise.noise(Vector((u * 2.3, v * 2.3, 1.7)))
        dog = max(0.0, u + v - 1.55) * 0.006
        return 0.0008 * n + dog + 0.0005 * math.sin(u * 3.1)
    return sheet(0.12, 0.155, 0.0003, 7, 8, lift)


part('Briefcases__body', f_box(0.36, 0.10, 0.28), briefcase_body)
part('Briefcases__rim', f_box(0.36, 0.008, 0.28), briefcase_rim)
part('Briefcases__clasp', f_box(0.03, 0.01, 0.02), briefcase_clasp)
part('Briefcases__handle', f_bounds((-0.055, 0, -0.005), (0.055, 0.055, 0.005)), briefcase_handle)
part('Briefcases__lid', f_box(0.36, 0.02, 0.28), briefcase_lid)
part('Briefcases__scrap', f_bounds((-0.06, -0.0775, 0), (0.06, 0.0775, 0), 'plane'), paper_scrap)


# --------------------------------------------------------------- laptop --
def laptop_base():
    # base: soft plan corners, eased bottom, finger scoop at the front, ports
    s = plan_prism(rounded_rect(0.30, 0.22, 0.012, seg=3), 0.02)
    scoop = cyl(0.009, 0.009, 0.05, seg=10, bevel_r=0).rot(z=PI / 2).move(y=0.0115, z=0.11)
    ports = [box(0.006, 0.0045, w).move(x=-0.15, y=-0.001, z=z) for z, w in ((-0.04, 0.012), (-0.02, 0.012), (0.01, 0.007))]
    s = bevel(s, 0.0022, seg=2, angle=40)
    return cut(s, scoop, *ports)


def laptop_lid():
    s = plan_prism(rounded_rect(0.30, 0.22, 0.012, seg=3), 0.015)
    return bevel(s, 0.0028, seg=2, angle=40)


def paper_cup():
    prof = [(0, -0.043), (0.0262, -0.043), (0.0272, -0.045), (0.0281, -0.0448), (0.0355, 0.040),
            (0.0372, 0.0415), (0.038, 0.0433), (0.0372, 0.045), (0.0356, 0.0448), (0.0346, 0.0425),
            (0.03, 0.0412), (0, 0.041)]
    return lathe(prof, seg=20)


part('AbandonedLaptops__base', f_box(0.30, 0.02, 0.22), laptop_base)
part('AbandonedLaptops__lid', f_box(0.30, 0.015, 0.22), laptop_lid)
part('AbandonedLaptops__cup', f_cyl(0.038, 0.028, 0.09), paper_cup)


# ------------------------------------------------------------------ key --
def key_bow():
    # flat bow with a hanging hole (ring round Z like the torus it replaces)
    out = [(0.017 * math.cos(TAU * i / 18), 0.017 * math.sin(TAU * i / 18)) for i in range(18)]
    s = prism(out, 0.006)
    s = cut(s, cyl(0.0042, 0.0042, 0.02, seg=10, bevel_r=0).rot(x=PI / 2).move(x=-0.007))
    return bevel(s, 0.0013, seg=1, angle=40)


def key_blade():
    # blade along y: shoulder at +y (bow end), keyway grooves, bevelled tip at -y
    s = box(0.0042, 0.024, 0.0056).move(y=-0.002)
    sh = box(0.006, 0.004, 0.006).move(y=0.012)
    s = cut(s, *[box(0.0012, 0.03, 0.0012).move(x=sx * 0.0021, z=0.0006 * sx) for sx in (-1, 1)])
    def tip(p):
        if p.y < -0.009:
            p.z *= 0.45 + 0.55 * (p.y + 0.014) / 0.005 if p.y > -0.014 else 0.45
        return p
    return s.warp(tip) + sh


def key_bitting():
    out = [(-0.004, 0.003), (0.004, 0.003), (0.004, -0.0005), (0.0028, -0.003), (0.0016, -0.0012),
           (0.0004, -0.003), (-0.0009, -0.0008), (-0.0022, -0.003), (-0.004, -0.0012)]
    return prism(out, 0.003)


part('SmallKeys__bow', f_torus(0.014, 0.003), key_bow)
part('SmallKeys__blade', f_cyl(0.003, 0.003, 0.028), key_blade)
part('SmallKeys__bitting', f_box(0.008, 0.006, 0.003), key_bitting)


# --------------------------------------------------------- walkie-talkie --
def walkie_body():
    # radio: rounded shell, battery seam, PTT on the left, channel knob on top,
    # belt clip on the back; the front stays flat for the grille and LED
    core = rbox(0.056, 0.13, 0.026, r=0.006, seg=2).move(y=-0.005, z=0.002)
    core = cut(core, plan_ring(0.056, 0.026, 0.006, 0.0008, 0.0014, seg=2).move(y=-0.035, z=0.002))
    ptt = rbox(0.004, 0.034, 0.012, r=0.0015, seg=1).move(x=-0.028, y=0.02, z=0.002)
    knob = ribs(cyl(0.0065, 0.0065, 0.012, seg=12, bevel_r=0.001), 6, 0.0008, -0.005, 0.005, rmin=0.006).move(x=0.014, y=0.064)
    clip = rbox(0.03, 0.07, 0.003, r=0.001, seg=1).move(y=0.0, z=-0.0135)
    return core + ptt + knob + clip


def rubber_antenna():
    prof = [(0, -0.055), (0.003, -0.055), (0.003, -0.036), (0.0024, -0.032), (0.0021, 0.05), (0.0017, 0.054), (0, 0.055)]
    return lathe(prof, seg=8)


part('WalkieTalkies__body', f_box(0.06, 0.14, 0.03), walkie_body)
part('WalkieTalkies__antenna', f_cyl(0.003, 0.003, 0.11), rubber_antenna)


# ------------------------------------------------- digital clock / radio --
part('DigitalWallClocks__cord', f_cyl(0.004, 0.004, 1.9),
     lambda: cyl(0.004, 0.004, 1.9, seg=8, bevel_r=0.001, bevel_seg=1))


def radio_case():
    # clock radio: soft shell, speaker slots on the top left, a snooze bar on
    # the top right, a plinth groove; the front stays flat for dial and LED
    s = rbox(0.20, 0.074, 0.12, r=0.005, seg=2).move(y=-0.003)
    slots = [box(0.0028, 0.01, 0.07).move(x=-0.085 + 0.009 * i, y=0.034, z=-0.005) for i in range(6)]
    plinth = plan_ring(0.20, 0.12, 0.005, 0.001, 0.0015, seg=2).move(y=-0.033)
    s = cut(s, plinth, *slots)
    snooze = rbox(0.07, 0.008, 0.022, r=0.003, seg=2).move(x=0.05, y=0.035, z=0.02)
    return s + snooze


def radio_knob():
    prof = [(0, -0.003), (0.012, -0.003), (0.012, 0.0014), (0.0106, 0.003), (0.0045, 0.0026), (0, 0.0026)]
    return ribs(lathe(prof, seg=24), 12, 0.0009, -0.0029, 0.0015, rmin=0.011)


part('ClockRadios__case', f_box(0.20, 0.08, 0.12), radio_case)
part('ClockRadios__knob', f_cyl(0.012, 0.012, 0.006), radio_knob)


# ----------------------------------------------------------- typewriter --
def typewriter_body():
    # side profile: sloped front apron under the keys, rounded back; a typebar
    # well between the keys and the platen, rubber feet under the corners
    prof = [(-0.11, -0.045), (0.11, -0.045), (0.11, -0.03), (0.086, 0.05), (-0.098, 0.05),
            (-0.106, 0.047), (-0.11, 0.04)]
    s = side_prism(prof, 0.28)
    well = cyl(0.04, 0.04, 0.2, seg=12, bevel_r=0).rot(z=PI / 2).move(y=0.075, z=-0.015)
    s = bevel(cut(s, well), 0.004, seg=2, angle=30)
    feet = sum((cyl(0.008, 0.009, 0.006, seg=6, bevel_r=0).move(x=sx * 0.12, y=-0.047, z=sz * 0.09)
                for sx in (-1, 1) for sz in (-1, 1)), Shape())
    return s + feet


def typewriter_keys():
    # three stepped rows of round keys on a basket plate, and a space bar
    plate = box(0.22, 0.006, 0.10).move(y=-0.012)
    keys = Shape()
    for row in range(3):
        z = 0.022 - row * 0.024
        top = -0.002 + row * 0.0075
        for i in range(9):
            x = -0.088 + i * 0.022 + (row % 2) * 0.006
            k = lathe([(0.0082, -0.009), (0.0082, top), (0, top)], seg=6, cap_bottom=False)
            keys = keys + k.move(x=x, z=z)
    space = rbox(0.12, 0.006, 0.012, r=0.002, seg=1).move(y=-0.006, z=0.044)
    fill = box(0.22, 0.004, 0.004).move(y=0.013, z=-0.048)       # rear key-lever guard
    return plate + keys + space + fill


def platen():
    prof = [(0, -0.12), (0.018, -0.12), (0.02, -0.117), (0.02, -0.103), (0.018, -0.1), (0.006, -0.1),
            (0.006, -0.093), (0.016, -0.092), (0.016, 0.092), (0.006, 0.093), (0.006, 0.1), (0.018, 0.1),
            (0.02, 0.103), (0.02, 0.117), (0.018, 0.12), (0, 0.12)]
    s = lathe(prof, seg=16)
    s = ribs(s, 8, 0.0015, -0.1175, -0.1025, rmin=0.019)
    return ribs(s, 8, 0.0015, 0.1025, 0.1175, rmin=0.019)


def typed_sheet_standing():
    return sheet(0.18, 0.14, 0.0003, 6, 7, lambda u, v: -0.0034 * v * v + 0.0005 * math.sin(u * 4.0))


def typed_sheet_floor():
    # a sheet that has been round a platen keeps a curl along its length
    return sheet(0.18, 0.24, 0.0003, 6, 10, lambda u, v: 0.0034 * (2 * v - 1) ** 2 + 0.0004 * math.sin(u * 5.0 + v))


part('Typewriters__body', f_box(0.28, 0.10, 0.22), typewriter_body)
part('Typewriters__keys', f_box(0.22, 0.03, 0.10), typewriter_keys)
part('Typewriters__platen', f_cyl(0.020, 0.020, 0.24), platen)
part('Typewriters__paper', f_bounds((-0.09, -0.07, 0), (0.09, 0.07, 0), 'plane'), typed_sheet_standing)
part('Typewriters__sheet', f_bounds((-0.09, -0.12, 0), (0.09, 0.12, 0), 'plane'), typed_sheet_floor)


# ---------------------------------------------------------- alarm clock --
def alarm_shell():
    # drum case, front (+y) bezel ring round the dial, domed back
    prof = [(0, -0.0245), (0.03, -0.024), (0.047, -0.0225), (0.056, -0.019), (0.0595, -0.013), (0.06, -0.006),
            (0.06, 0.019), (0.0596, 0.0232), (0.0582, 0.025), (0.0535, 0.025), (0.052, 0.0232), (0, 0.0232)]
    return lathe(prof, seg=28)


def alarm_bell():
    prof = [(0, 0.0), (0.0205, 0.0), (0.025, 0.0008), (0.0235, 0.005), (0.019, 0.0125), (0.012, 0.019),
            (0.0045, 0.0212), (0.0045, 0.025), (0, 0.025)]
    return lathe(prof, seg=14)


def alarm_leg():
    # a turned bun foot on a short neck, oval to fill the 15 × 20 mm block
    prof = [(0, -0.01), (0.0055, -0.01), (0.0072, -0.0085), (0.0075, -0.006), (0.0062, -0.0032),
            (0.0036, -0.0005), (0.0034, 0.006), (0.0048, 0.0078), (0.0048, 0.01), (0, 0.01)]
    return lathe(prof, seg=10).scale(1, 1, 1.333)


part('AlarmClocks__shell', f_cyl(0.06, 0.06, 0.05), alarm_shell)
part('AlarmClocks__bell', f_bounds((-0.025, 0, -0.025), (0.025, 0.025, 0.025)), alarm_bell)
part('AlarmClocks__leg', f_box(0.015, 0.02, 0.02), alarm_leg)


# ------------------------------------------------------- vintage camera --
def camera_body():
    # rangefinder: rounded ends, top deck with a speed dial and shutter
    # button, top and bottom plate seams round the leatherette band
    core = bevel(plan_prism(rounded_rect(0.10, 0.05, 0.012, seg=3), 0.052).move(y=-0.004), 0.0018, seg=1, angle=40)
    seams = [plan_ring(0.10, 0.05, 0.012, 0.0008, 0.0012, seg=3).move(y=y) for y in (0.013, -0.022)]
    core = cut(core, *seams)
    deck = plan_prism(rounded_rect(0.094, 0.044, 0.01, seg=2), 0.006).move(y=0.024)
    dial = ribs(cyl(0.0075, 0.0075, 0.005, seg=12, bevel_r=0), 6, 0.0007, -0.0026, 0.0026, rmin=0.007)
    button = cyl(0.0028, 0.0028, 0.004, seg=6, bevel_r=0)
    return core + deck + dial.move(x=0.03, y=0.0295, z=-0.006) + button.move(x=0.017, y=0.028, z=0.008)


def camera_lens():
    prof = [(0, -0.015), (0.0185, -0.015), (0.02, -0.0125), (0.02, -0.002), (0.0174, -0.0015), (0.0174, 0.009),
            (0.019, 0.0095), (0.019, 0.015), (0.0164, 0.015), (0, 0.013)]
    return ribs(lathe(prof, seg=16), 8, 0.0012, -0.0122, -0.0024, rmin=0.019)


def camera_finder():
    s = rbox(0.02, 0.015, 0.015, r=0.0018, seg=1)
    return cut(s, box(0.012, 0.007, 0.004).move(y=0.001, z=0.0075), box(0.008, 0.005, 0.004).move(y=0.001, z=-0.0075))


def film_can():
    prof = [(0, -0.03), (0.017, -0.03), (0.018, -0.029), (0.018, 0.026), (0.0172, 0.0268), (0.0172, 0.03), (0, 0.03)]
    return lathe(prof, seg=18)


def film_cap():
    prof = [(0, -0.004), (0.0182, -0.004), (0.019, -0.003), (0.019, 0.0024), (0.0182, 0.004), (0.0162, 0.004),
            (0.0156, 0.0026), (0, 0.0026)]
    return lathe(prof, seg=18)


part('VintageCameras__body', f_box(0.10, 0.06, 0.05), camera_body)
part('VintageCameras__lens', f_cyl(0.020, 0.020, 0.03), camera_lens)
part('VintageCameras__finder', f_box(0.02, 0.015, 0.015), camera_finder)
part('VintageCameras__can', f_cyl(0.018, 0.018, 0.06), film_can)
part('VintageCameras__cap', f_cyl(0.019, 0.019, 0.008), film_cap)


# ---------------------------------------------------- wall flashlight --
def flash_bracket():
    # back plate screwed to the wall, a saddle cradling the torch body
    plate = rbox(0.05, 0.06, 0.003, r=0.0015, seg=1).move(z=-0.006)
    saddle = cut(rbox(0.036, 0.024, 0.012, r=0.0015, seg=1).move(z=0.0015),
                 cyl(0.0252, 0.0252, 0.06, seg=14, bevel_r=0).rot(z=PI / 2).move(z=0.032))
    wings = [box(0.06, 0.006, 0.004).move(y=sy * 0.027, z=-0.0055) for sy in (-1, 1)]
    screws = [screw_head(0.0028, 0.0014, slot=False).rot(x=PI / 2).move(x=sx * 0.018, y=0.022 * sy, z=-0.0045)
              for sx, sy in ((-1, 1), (1, -1))]
    return plate + saddle + sum(wings + screws, Shape())


def flash_barrel():
    # -y joins the head, +y is the tail cap; knurled grip between two rings
    prof = [(0, -0.08), (0.0245, -0.08), (0.0245, -0.058), (0.025, -0.056), (0.025, -0.03), (0.0232, -0.028),
            (0.025, -0.026), (0.025, 0.05), (0.0232, 0.052), (0.025, 0.054), (0.025, 0.07), (0.0235, 0.075),
            (0.019, 0.079), (0, 0.08)]
    return ribs(lathe(prof, seg=20), 10, 0.0012, -0.024, 0.048, rmin=0.0245)


def flash_head():
    # bezel at the lens end (-y), tapering back to the barrel
    prof = [(0, -0.0135), (0.0238, -0.0135), (0.0245, -0.015), (0.0294, -0.015), (0.03, -0.0142), (0.03, -0.0095),
            (0.0284, -0.0075), (0.0266, 0.008), (0.0255, 0.015), (0, 0.015)]
    return lathe(prof, seg=16)


part('WallFlashlights__bracket', f_box(0.06, 0.06, 0.015), flash_bracket)
part('WallFlashlights__barrel', f_cyl(0.025, 0.025, 0.16), flash_barrel)
part('WallFlashlights__head', f_cyl(0.030, 0.028, 0.03), flash_head)


# ----------------------------------------------------------- tissue box --
def ellipse(a, b, n=24):
    return [(a * math.cos(TAU * i / n), b * math.sin(TAU * i / n)) for i in range(n)]


def tissue_box():
    # card box: folded edges, glued end flaps, a perforated oval round the slot
    s = rbox(0.20, 0.06, 0.10, r=0.0016, seg=1)
    ring = cut(plan_prism(ellipse(0.063, 0.027), 0.004), plan_prism(ellipse(0.0615, 0.0255), 0.006)).move(y=0.03)
    flaps = [box(0.004, 0.0012, 0.104).move(x=sx * 0.1, y=-0.008) for sx in (-1, 1)] + \
            [box(0.004, 0.03, 0.0012).move(x=sx * 0.1, y=-0.016, z=0.02) for sx in (-1, 1)]
    return cut(s, ring, *flaps)


part('TissueBoxes__box', f_box(0.20, 0.06, 0.10), tissue_box)


# ----------------------------------------------------------- hand basket --
def basket_body():
    # plastic shopping basket: walls taper to the base, slots moulded into
    # the walls, a thick rolled rim (the top is covered by the dark inside)
    main = plan_prism(rounded_rect(0.214, 0.154, 0.012, seg=3), 0.108).move(y=-0.006)
    slots = [box(0.022, 0.05, 0.006).move(x=-0.076 + 0.038 * i, y=-0.002, z=sz * 0.077)
             for i in range(5) for sz in (-1, 1)] + \
            [box(0.006, 0.05, 0.024).move(x=sx * 0.107, y=-0.002, z=-0.04 + 0.04 * i)
             for i in range(3) for sx in (-1, 1)]
    main = cut(main, *slots)
    def tp(p):
        k = 0.86 + 0.14 * min(1.0, (p.y + 0.06) / 0.108)
        p.x *= k
        p.z *= k
        return p
    rim = bevel(plan_prism(rounded_rect(0.22, 0.16, 0.013, seg=3), 0.012), 0.003, seg=2, angle=40).move(y=0.054)
    return main.warp(tp) + rim


def basket_handle():
    arc = tube([(0.05 * math.cos(PI * i / 12), 0.05 * math.sin(PI * i / 12), 0) for i in range(13)], 0.0042, seg=8)
    grip = capsule(0.006, 0.034, seg=10, cap_rings=2).rot(z=PI / 2).move(y=0.05)
    hubs = sum((cyl(0.006, 0.006, 0.01, seg=10, bevel_r=0.001).rot(x=PI / 2).move(x=sx * 0.05, y=0.006)
                for sx in (-1, 1)), Shape())
    return arc + grip + hubs


part('Handbaskets__body', f_box(0.22, 0.12, 0.16), basket_body)
part('Handbaskets__handle', f_bounds((-0.056, 0, -0.006), (0.056, 0.056, 0.006)), basket_handle)


# ----------------------------------------------------- dice, cards, chips --
part('SmallDicePairs__die', f_box(0.028, 0.028, 0.028), lambda: rbox(0.028, 0.028, 0.028, r=0.0034, seg=3))
part('SmallCardStacks__card', f_box(0.045, 0.001, 0.065),
     lambda: plan_prism(rounded_rect(0.045, 0.065, 0.0036, seg=3), 0.001))


def poker_chip():
    # raised rim ring on both faces round a recessed inlay, notched edge
    prof = [(0, -0.0017), (0.0138, -0.0017), (0.0145, -0.002), (0.0184, -0.002), (0.019, -0.0014),
            (0.019, 0.0014), (0.0184, 0.002), (0.0145, 0.002), (0.0138, 0.0017), (0, 0.0017)]
    return ribs(lathe(prof, seg=16), 8, 0.0006, -0.0015, 0.0015, rmin=0.0188)


part('SmallPokerChips__chip', f_cyl(0.019, 0.019, 0.004), poker_chip)
part('SmallDominoes__tile', f_box(0.040, 0.006, 0.020), lambda: rbox(0.040, 0.006, 0.020, r=0.0016, seg=2))


# -------------------------------------------------------------- chess --
def chess_base():
    prof = [(0, -0.007), (0.0235, -0.007), (0.024, -0.005), (0.021, -0.003), (0.0212, 0.0005), (0.0195, 0.003),
            (0.0195, 0.0062), (0.0185, 0.007), (0, 0.007)]
    return lathe(prof, seg=16)


def pawn_stem():
    prof = [(0, -0.013), (0.015, -0.013), (0.0132, -0.0085), (0.0098, -0.003), (0.0072, 0.004), (0.0065, 0.0085),
            (0.010, 0.0105), (0.010, 0.0118), (0.0062, 0.013), (0, 0.013)]
    return lathe(prof, seg=12)


def rook_body():
    prof = [(0, -0.02), (0.02, -0.02), (0.0185, -0.015), (0.0138, -0.006), (0.013, 0.006), (0.0165, 0.0115),
            (0.0165, 0.014), (0.0145, 0.015), (0.016, 0.02), (0.0115, 0.02), (0, 0.0175)]
    return lathe(prof, seg=12)


def bishop_body():
    prof = [(0, -0.0275), (0.016, -0.0275), (0.0125, -0.021), (0.0078, -0.01), (0.0068, -0.002), (0.011, -0.0008),
            (0.0078, 0.0015), (0.0095, 0.0105), (0.0068, 0.0195), (0.0028, 0.0255), (0, 0.0275)]
    s = lathe(prof, seg=12)
    return cut(s, box(0.014, 0.0022, 0.03).rot(z=0.6).move(x=0.0065, y=0.0145))


def knight_neck():
    out = [(-0.007, -0.02), (0.007, -0.02), (0.006, -0.009), (0.0042, 0.004), (0.007, 0.02), (-0.004, 0.02),
           (-0.007, 0.012), (-0.0058, -0.004)]
    s = prism(out, 0.02)
    def thin(p):
        p.z *= 1 - 0.25 * (p.y + 0.02) / 0.04
        return p
    return bevel(s.warp(thin), 0.0016, seg=2, angle=30).fit((-0.007, -0.02, -0.01), (0.007, 0.02, 0.01))


def knight_head():
    out = [(-0.013, -0.007), (0.013, -0.007), (0.013, 0.001), (0.006, 0.007), (-0.009, 0.007), (-0.013, 0.003)]
    s = prism(out, 0.02)
    def muzzle(p):
        p.z *= 1 - 0.22 * (p.x + 0.013) / 0.026
        return p
    return bevel(s.warp(muzzle), 0.0018, seg=2, angle=30).fit((-0.013, -0.007, -0.01), (0.013, 0.007, 0.01))


def knight_snout():
    s = rbox(0.014, 0.010, 0.016, r=0.0028, seg=1)
    return cut(s, box(0.006, 0.0008, 0.02).move(x=0.007, y=-0.0015))


part('SmallChessPieces__base', f_cyl(0.020, 0.024, 0.014), chess_base)
part('SmallChessPieces__pawnstem', f_cyl(0.010, 0.015, 0.026), pawn_stem)
part('SmallChessPieces__pawnhead', f_sphere(0.014), lambda: sphere(0.014, seg=16), smooth=180)
part('SmallChessPieces__rook', f_cyl(0.016, 0.020, 0.040), rook_body)
part('SmallChessPieces__merlon', f_box(0.008, 0.008, 0.008), lambda: rbox(0.008, 0.008, 0.008, r=0.0012, seg=1))
part('SmallChessPieces__bishop', f_cone(0.016, 0.055), bishop_body)
part('SmallChessPieces__finial', f_sphere(0.008), lambda: sphere(0.008, seg=10), smooth=180)
part('SmallChessPieces__neck', f_box(0.014, 0.040, 0.020), knight_neck)
part('SmallChessPieces__head', f_box(0.026, 0.014, 0.020), knight_head)
part('SmallChessPieces__snout', f_box(0.014, 0.010, 0.016), knight_snout)
part('SmallChessPieces__ear', f_cone(0.004, 0.010),
     lambda: lathe([(0, -0.005), (0.004, -0.005), (0.0036, -0.001), (0.002, 0.003), (0, 0.005)], seg=6))


# ---------------------------------------------------------- pool balls --
def ball_stripe():
    # a band of the ball's own sphere instead of a flat-sided tube
    R, h = 0.028028, 0.0084
    prof = [(math.sqrt(R * R - y * y), y) for y in (-h, -h / 2, 0.0, h / 2, h)]
    s = lathe(prof, seg=20, cap_bottom=False, cap_top=False)
    # an open band: make sure the faces look outwards
    c = s.v[s.f[0][0]] + s.v[s.f[0][1]] + s.v[s.f[0][2]]
    a, b, d = (s.v[i] for i in s.f[0][:3])
    if (b - a).cross(d - a).dot(c) < 0:
        s.f = [tuple(reversed(f)) for f in s.f]
    return s


part('SmallPoolBalls__ball', f_sphere(0.028), lambda: sphere(0.028, seg=16), smooth=180)
part('SmallPoolBalls__stripe', f_cyl(0.028028, 0.028028, 0.0168), ball_stripe, smooth=180)


# --------------------------------------------------------------- darts --
def dart_point():
    prof = [(0, -0.0075), (0.003, -0.0075), (0.0027, -0.0058), (0.0012, -0.0048), (0.001, 0.004), (0.0003, 0.0072), (0, 0.0075)]
    return lathe(prof, seg=8)


def dart_barrel():
    # tungsten torpedo: -y towards the point; ringed grip in the middle
    prof = [(0, -0.02), (0.0022, -0.02), (0.0038, -0.017), (0.005, -0.01)]
    for y in (-0.006, 0.0):
        prof += [(0.005, y - 0.0012), (0.0043, y), (0.005, y + 0.0012)]
    prof += [(0.005, 0.009), (0.004, 0.016), (0.0022, 0.02), (0, 0.02)]
    return lathe(prof, seg=10)


def dart_shaft():
    prof = [(0, -0.015), (0.002, -0.015), (0.002, -0.012), (0.0014, -0.011), (0.0013, 0.012), (0.0018, 0.013),
            (0.0018, 0.015), (0, 0.015)]
    return lathe(prof, seg=6)


part('SmallDarts__point', f_cone(0.003, 0.015), dart_point)
part('SmallDarts__barrel', f_cyl(0.005, 0.005, 0.040), dart_barrel)
part('SmallDarts__shaft', f_cyl(0.002, 0.002, 0.030), dart_shaft)


# ------------------------------------------------------------ pool cue --
def cue_shaft():
    # butt (-y, thick) to tip end (+y): butt cap, joint collar, long taper, ferrule
    prof = [(0, -0.45), (0.0125, -0.45), (0.014, -0.448), (0.014, -0.39), (0.0132, -0.33), (0.0122, -0.06),
            (0.0126, -0.058), (0.0126, -0.044), (0.0117, -0.042), (0.0104, 0.1), (0.0082, 0.3), (0.0066, 0.425),
            (0.0063, 0.428), (0.0062, 0.45), (0, 0.45)]
    return lathe(prof, seg=12)


def cue_tip():
    prof = [(0, -0.004), (0.006, -0.004), (0.006, 0.0), (0.0052, 0.0025), (0.003, 0.0038), (0, 0.004)]
    return lathe(prof, seg=12)


def cue_grip():
    # linen wrap between two trim rings
    prof = [(0, -0.06), (0.014, -0.06), (0.015, -0.0585), (0.015, -0.054), (0.0142, -0.053)]
    y = -0.05
    while y < 0.049:
        prof += [(0.0149, y + 0.006), (0.0142, y + 0.0125)]
        y += 0.0125
    prof += [(0.015, 0.054), (0.015, 0.0585), (0.014, 0.06), (0, 0.06)]
    return lathe(prof, seg=10)


part('SmallPoolCues__shaft', f_cyl(0.006, 0.014, 0.90), cue_shaft)
part('SmallPoolCues__tip', f_cyl(0.006, 0.006, 0.008), cue_tip)
part('SmallPoolCues__grip', f_cyl(0.015, 0.015, 0.12), cue_grip)


# ------------------------------------------------------ remote control --
def remote_body():
    # waisted plan, soft underside, the IR end (-z) sloping down; the key
    # field on top stays flat for the buttons
    side = [(0.0225 - 0.0014 * math.exp(-(z / 0.035) ** 2), z) for z in (-0.063, -0.04, -0.015, 0.015, 0.04, 0.063)]
    out = round_outline([(-0.0225, 0.075)] + [(-x, z) for x, z in reversed(side)] + [(-0.0225, -0.075), (0.0225, -0.075)]
                        + [(x, z) for x, z in side] + [(0.0225, 0.075)], 0.011, seg=2)
    s = plan_prism(out, 0.010)
    def slope(p):
        if p.y > 0 and p.z < -0.055:
            p.y -= 0.0026 * (-0.055 - p.z) / 0.02
        return p
    return bevel(s.warp(slope), 0.0022, seg=1, angle=35)


def remote_button():
    return lathe([(0, -0.001), (0.004, -0.001), (0.004, 0.0), (0.0033, 0.0008), (0, 0.001)], seg=10)


def remote_bigbutton():
    s = plan_prism(rounded_rect(0.024, 0.014, 0.004, seg=2), 0.002)
    def ch(p):
        if p.y > 0:
            p.x *= 0.95
            p.z *= 0.92
        return p
    return s.warp(ch)


def remote_antenna():
    return lathe([(0, -0.03), (0.002, -0.0295), (0.002, -0.0275), (0.0014, -0.027), (0.0014, 0.03), (0, 0.03)], seg=6)


part('SmallRemoteControls__body', f_box(0.045, 0.010, 0.15), remote_body)
part('SmallRemoteControls__button', f_cyl(0.004, 0.004, 0.002), remote_button)
part('SmallRemoteControls__bigbutton', f_box(0.024, 0.002, 0.014), remote_bigbutton)
part('SmallRemoteControls__antenna', f_cyl(0.002, 0.002, 0.06), remote_antenna)


# ---------------------------------------------------- broken wall clock --
def wall_clock_case():
    prof = [(0, -0.01), (0.128, -0.01), (0.136, -0.008), (0.14, -0.0035), (0.14, 0.01), (0, 0.01)]
    return lathe(prof, seg=40)


def wall_clock_bezel():
    # moulded bezel ring: flat back on the case, rounded front, inner lip
    prof = [(0.133, -0.008), (0.147, -0.008), (0.148, -0.004), (0.1465, 0.003), (0.142, 0.0072), (0.1375, 0.008),
            (0.1335, 0.0058), (0.132, 0.002), (0.132, -0.006)]
    ring = lathe(prof + [prof[0]], seg=48, cap_bottom=False, cap_top=False)
    return weld(ring).rot(x=PI / 2)


part('BrokenWallClocks__case', f_cyl(0.14, 0.14, 0.02), wall_clock_case)
part('BrokenWallClocks__bezel', f_torus(0.14, 0.008), wall_clock_bezel)
