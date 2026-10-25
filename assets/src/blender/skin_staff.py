"""
The Showroom's staff (inspired by SCP-3008): a store employee a little too
tall, in the store's mustard polo (collar, placket, short sleeves, a blank
name badge), dark trousers with a belt, black work shoes, pale skin on the
forearms, hands and neck, and a smooth, slightly too long head with only the
faintest brow and nose under the skin: no eyes, no mouth.

Run by tools/blender/skinned.py (`... --python tools/blender/skinned.py --
staff`) on the rig makeStaff builds (HUMAN_RIGS.staff in backrooms.js, dumped
by `node tools/rig-dump.mjs staff`). The clothes are part of the one skinned
body: each piece is tagged shirt / trousers / belt / shoe / skin ..., and the
painter colours a vertex by the pieces it lies on. The folds are sculpted:
the shirt gathered over the belt, the trousers bunched behind the knees and
broken over the shoes, creases at the elbows.
"""
exec(compile(open(os.path.join(HERE, 'skin_human.py')).read(), os.path.join(HERE, 'skin_human.py'), 'exec'), globals())

# the torso (world y, half-width, front z, back z): trousers up to the belt,
# the shirt from the belt up, bloused a little over it
ST_TROUSERS = [
    (0.952, 0.045, 0.03, -0.04), (0.975, 0.11, 0.06, -0.085), (1.0, 0.152, 0.08, -0.108),
    (1.035, 0.168, 0.09, -0.117), (1.07, 0.172, 0.094, -0.117), (1.105, 0.168, 0.097, -0.108),
    (1.135, 0.163, 0.1, -0.1), (1.17, 0.161, 0.103, -0.097),
]
ST_SHIRT = [
    (1.15, 0.161, 0.102, -0.097), (1.172, 0.165, 0.108, -0.099), (1.2, 0.167, 0.112, -0.099),
    (1.25, 0.163, 0.111, -0.095), (1.32, 0.161, 0.112, -0.096), (1.4, 0.163, 0.118, -0.104),
    (1.48, 0.166, 0.124, -0.111), (1.56, 0.166, 0.127, -0.117), (1.62, 0.167, 0.122, -0.118),
    (1.67, 0.168, 0.11, -0.112), (1.71, 0.164, 0.096, -0.103), (1.742, 0.15, 0.08, -0.092),
    (1.765, 0.11, 0.066, -0.08), (1.785, 0.072, 0.056, -0.068), (1.8, 0.062, 0.05, -0.062),
]
SKIN = hexc('#CDB8A4')     # pale, a little sallow
PAL = dict(shirt=hexc('#B38A2B'), trousers=hexc('#2F323A'), belt=hexc('#1A1612'), buckle=hexc('#8B8577'),
           shoe=hexc('#131313'), sole=hexc('#0A0A0A'), badge=hexc('#EDEAE2'), button=hexc('#E3D9C2'), skin=SKIN)
BADGE = {}


def oriented_box(bm, p, n, size, up=V((0, 1, 0))):
    """a box at p, its +Z along n, its +Y as near `up` as it can"""
    z = n.normalized()
    x = up.cross(z).normalized()
    y = z.cross(x)
    R = Matrix((x, y, z)).transposed().to_4x4()
    bmesh.ops.create_cube(bm, size=1.0, matrix=Matrix.Translation(p) @ R @ Matrix.Diagonal(V((*size, 1))))
    return x, y, z


def surf_n(ST, y, a):
    return (tpt(ST, y, a, 0.01) - tpt(ST, y, a)).normalized()


def torso_pieces():
    pcs = []
    TB = ['pelvis', 'abdomen', 'chest']
    def trousers(bm):
        torso_loft(bm, ST_TROUSERS)
        band(bm, ST_TROUSERS, 1.104, 1.13, 0.0022)                                     # the waistband
        for s in (-1, 1):
            ridge(bm, surf_path(ST_TROUSERS, 1.128, s * 0.72, 1.035, s * 1.38, 0.0005, 7), 0.0042)   # front pockets
            for (y0, a0, y1, a1) in ((1.09, s * 2.5, 1.09, s * 2.88), (1.09, s * 2.5, 1.0, s * 2.52), (1.09, s * 2.88, 1.0, s * 2.86)):
                ridge(bm, surf_path(ST_TROUSERS, y0, a0, y1, a1, 0.0004, 5), 0.003)          # back pockets
            ridge(bm, surf_path(ST_TROUSERS, 1.125, s * 1.57, 0.99, s * 1.57, 0.0004, 6), 0.0026)   # side seams
        ridge(bm, surf_path(ST_TROUSERS, 1.125, 0.13, 0.985, 0.02, 0.0004, 7), 0.0036)     # the fly
    pcs.append(WP(trousers, TB, 'trousers', sig=0.05))
    def belt(bm):
        band(bm, ST_TROUSERS, 1.129, 1.163, 0.0058)
    pcs.append(WP(belt, TB, 'belt', sig=0.05))
    def loops(bm):
        for a in (-2.7, -2.1, -1.35, -0.55, 0.55, 1.35, 2.1, 2.7, math.pi):
            ridge(bm, surf_path(ST_TROUSERS, 1.12, a, 1.172, a, 0.0075, 4), 0.0055)
    pcs.append(WP(loops, TB, 'trousers', sig=0.05, bias=0.0006))
    def buckle(bm):
        p = tpt(ST_TROUSERS, 1.146, 0.0, 0.006)
        oriented_box(bm, p, surf_n(ST_TROUSERS, 1.146, 0.0), (0.052, 0.036, 0.012))
    pcs.append(WP(buckle, TB, 'buckle', sig=0.05, bias=0.0015))
    def shirt(bm):
        torso_loft(bm, ST_SHIRT)
        # gathered where it is tucked in: a soft roll over the belt and
        # gathers running up from it
        ring = [tpt(ST_SHIRT, 1.176, TAU * i / 48, 0.002) for i in range(48)]
        chain(bm, ring + [ring[0]], 0.0062, seg=8)
        for k, a in enumerate([-2.8, -2.35, -1.9, -1.45, -1.0, -0.62, 0.62, 1.0, 1.45, 1.9, 2.35, 2.8, 3.05]):
            h = 0.035 + 0.025 * ((k * 7) % 5) / 4
            ridge(bm, surf_path(ST_SHIRT, 1.17, a, 1.17 + h, a + 0.07 * ((k % 3) - 1), 0.0, 6), 0.0058)
        for y, a0, a1 in ((1.245, 2.4, 3.9), (1.29, 2.55, 3.75), (1.23, -0.9, -0.3), (1.235, 0.35, 0.95)):
            ridge(bm, surf_path(ST_SHIRT, y, a0, y + 0.012, a1, 0.0, 7), 0.0048)            # folds at the small of the back and the front
        for s in (-1, 1):
            ridge(bm, surf_path(ST_SHIRT, 1.6, s * 1.45, 1.47, s * 1.62, 0.0, 6), 0.005)     # pulled from under the arms
            ridge(bm, surf_path(ST_SHIRT, 1.585, s * 1.7, 1.49, s * 1.95, 0.0, 6), 0.0045)
        # the placket and its two buttons
        sweep(bm, [tpt(ST_SHIRT, y, 0.0, -0.001) for y in (1.772, 1.74, 1.7, 1.66, 1.632)], [0.0165] * 5, sides=10, flat=0.32, ref=(0, 0, 1))
    pcs.append(WP(shirt, TB, 'shirt', sig=0.05))
    def buttons(bm):
        for y in (1.717, 1.668):
            ell(bm, tpt(ST_SHIRT, y, 0.0, 0.004), (0.0055, 0.0055, 0.004), seg=12)
    pcs.append(WP(buttons, TB, 'button', sig=0.05, bias=-0.0012))
    def badge(bm):
        a = 0.47
        p = tpt(ST_SHIRT, 1.556, a, 0.002)
        x, y, z = oriented_box(bm, p, surf_n(ST_SHIRT, 1.556, a), (0.078, 0.03, 0.011))
        BADGE.update(c=p + z * 0.0055, x=x, y=y)
        ell(bm, p + y * 0.017 + z * 0.004, (0.007, 0.004, 0.004), seg=10)                  # the clip
    pcs.append(WP(badge, TB, 'badge', sig=0.05, bias=0.0018))
    def collar(bm):
        # a rolled polo collar standing round the neck, higher at the back,
        # and its two points turned down onto the chest
        c = Hd('neck') + V((0, 0.012, 0.0))
        loop = [c + V((math.sin(a) * 0.071, 0.024 * (1 - math.cos(a)) * 0.5 - 0.006, math.cos(a) * 0.068 + 0.004)) for a in [TAU * i / 40 for i in range(40)]]
        chain(bm, loop + [loop[0]], 0.0115, seg=10)
        for s in (-1, 1):
            for k in range(4):
                t = k / 3
                ell(bm, c + V((s * lerp(0.016, 0.06, t), lerp(-0.012, -0.038, t), lerp(0.071, 0.078, t))),
                    (lerp(0.017, 0.012, t), lerp(0.016, 0.012, t), 0.0075), rot=(-0.55, 0, s * 0.5), seg=14)
    pcs.append(WP(collar, ['chest', 'neck'], 'shirt', sig=0.04, bias=0.0008))
    for s, n in ((1, 'L'), (-1, 'R')):
        def yoke(bm, s=s, n=n):
            # the shoulder's slope, neck to the point of the shoulder
            tube(bm, Hd('neck') + V((s * 0.05, -0.022, -0.02)), Hd('upperarm.' + n) + V((-s * 0.025, 0.018, 0.0)), 0.032, 0.03, seg=14)
        pcs.append(WP(yoke, ['chest', 'clavicle.' + n], 'shirt', sig=0.05))
        U = fr('upperarm.' + n)
        def cap(bm, U=U, s=s):
            ellf(bm, U, (s * 0.0, -0.036, -0.002), (0.056, 0.066, 0.056), seg=24)              # the shoulder in its sleeve
        pcs.append(WP(cap, ['clavicle.' + n, 'upperarm.' + n], 'shirt', sig=0.035))
        def sleeve(bm, U=U, s=s):
            sweep(bm, [U @ V((0, y, 0.002)) for y in (-0.01, -0.05, -0.1, -0.14)], [0.056, 0.057, 0.056, 0.055], sides=24)
            sweep(bm, [U @ V((0, y, 0.002)) for y in (-0.138, -0.148, -0.158)], [0.0575, 0.058, 0.057], sides=24)   # the ribbed hem
            for k in range(2):                                                                  # creases running round under the arm
                ridge(bm, [U @ V((-s * 0.056 * math.cos(a), -0.05 - 0.035 * k - 0.01 * a, 0.056 * math.sin(a))) for a in [lerp(-1.0, 1.0, i / 5) for i in range(6)]], 0.004)
        pcs.append(WP(sleeve, ['upperarm.' + n, 'clavicle.' + n], 'shirt', sig=0.03))   # the clavicle: it meets the chest at the armpit
    return pcs


def leg_pieces(n, s):
    T = fr('thigh.' + n)
    Lt, Ls = leg_len(n)
    prof = [(0.03, 0.084), (-0.03, 0.088), (-0.12, 0.085), (-0.25, 0.079), (-0.38, 0.072), (-0.47, 0.067),
            (-0.56, 0.064), (-0.7, 0.062), (-0.84, 0.061), (-0.92, 0.062), (-0.955, 0.065)]
    def r_at(y):
        for (y0, r0), (y1, r1) in zip(prof, prof[1:]):
            if y1 <= y <= y0: return lerp(r0, r1, (y - y0) / (y1 - y0))
        return prof[-1][1]
    def leg(bm):
        sweep(bm, [T @ V((0, y, 0.004 * math.sin(-y * 3))) for y, _ in prof], [r for _, r in prof], sides=28, flat=0.96)
        K = -Lt
        # bunched behind the knee, folds pulled across the front of it
        for k, y in enumerate((K + 0.05, K + 0.015, K - 0.022, K - 0.058)):
            ridge(bm, [T @ V((math.sin(a) * r_at(y), y - 0.008 * math.cos(a - math.pi), math.cos(a) * r_at(y))) for a in
                       [math.pi + lerp(-1.1, 1.1, i / 6) for i in range(7)]], 0.0062 - 0.0004 * k)
        for y0, a0, y1, a1 in ((K + 0.12, 0.7, K + 0.05, -0.15), (K + 0.07, -0.9, K + 0.02, 0.2), (K - 0.07, -0.6, K - 0.12, 0.45)):
            ridge(bm, [T @ V((math.sin(lerp(a0, a1, i / 5)) * r_at(lerp(y0, y1, i / 5)), lerp(y0, y1, i / 5),
                              math.cos(lerp(a0, a1, i / 5)) * r_at(lerp(y0, y1, i / 5)))) for i in range(6)], 0.0052)
        # broken over the shoe
        for k, y in enumerate((-Lt - Ls + 0.075, -Lt - Ls + 0.052)):
            ridge(bm, [T @ V((math.sin(a) * r_at(y), y + 0.01 * math.cos(a), math.cos(a) * r_at(y))) for a in
                       [lerp(-1.3, 1.3, i / 6) for i in range(7)]], 0.006 - 0.001 * k)
        # the pressed crease down the front, and the seam down the outside
        ridge(bm, [T @ V((0, y, r_at(y) - 0.0005)) for y in [lerp(-0.13, -0.9, i / 9) for i in range(10)]], 0.0028)
        ridge(bm, [T @ V((s * (r_at(y) - 0.0005), y, 0)) for y in [lerp(-0.05, -0.92, i / 9) for i in range(10)]], 0.0022)
    return [WP(leg, ['thigh.' + n, 'shin.' + n], 'trousers', sig=0.035)]


def shoe_pieces(n, s):
    Fo = fr('foot.' + n)
    g = -Hd('foot.' + n).y
    def shoe(bm):
        in_frame(bm, Fo, lambda b: C['loft_z'](b, [
            (-0.089, 0.026, g + 0.032, g + 0.004), (-0.077, 0.036, g + 0.033, g), (-0.03, 0.04, g + 0.032, g),
            (0.03, 0.04, g + 0.026, g + 0.002), (0.09, 0.05, g + 0.022, g), (0.15, 0.054, g + 0.022, g),
            (0.2, 0.047, g + 0.024, g + 0.004), (0.225, 0.033, g + 0.026, g + 0.01), (0.237, 0.012, g + 0.026, g + 0.016)], n=36, expo=4.0))
        up = [(-0.083, 0.024, g + 0.075, g + 0.02), (-0.07, 0.034, g + 0.087, g + 0.016), (-0.04, 0.037, g + 0.1, g + 0.016),
              (0.0, 0.036, g + 0.112, g + 0.016), (0.04, 0.038, g + 0.106, g + 0.016), (0.09, 0.044, g + 0.086, g + 0.016),
              (0.14, 0.048, g + 0.067, g + 0.016), (0.18, 0.045, g + 0.059, g + 0.017), (0.21, 0.036, g + 0.053, g + 0.019),
              (0.229, 0.021, g + 0.045, g + 0.021)]
        in_frame(bm, Fo, lambda b: C['loft_z'](b, up, n=40, expo=2.6))
        top = lambda z: C['interp']([(u[0], u[2]) for u in up], z)[0]
        for z in (0.035, 0.058, 0.081, 0.104):                                     # the laces
            tube(bm, Fo @ V((-0.019, top(z) - 0.004, z)), Fo @ V((0.019, top(z) - 0.004, z + 0.006)), 0.0042, 0.0042, seg=8)
        ridge(bm, [Fo @ V((math.sin(a) * 0.046, top(0.166) - 0.004 - 0.02 * (1 - math.cos(a)), 0.166 - 0.004 * abs(math.sin(a)))) for a in
                   [lerp(-1.2, 1.2, i / 6) for i in range(7)]], 0.0026)            # the toe cap's stitching
    return [WP(shoe, ['foot.' + n, 'toe.' + n], 'shoe', sig=0.02)]


def staff_pieces():
    PIECES.clear()
    pcs = torso_pieces()
    for s, n in ((1, 'L'), (-1, 'R')):
        pcs += leg_pieces(n, s)
        pcs += shoe_pieces(n, s)
        pcs.append(arm_pieces(n, s, dict(up=(0.05, 0.04), fore=(0.042, 0.027), bulk=0.6)))
        pcs += hand_pieces(n, s, dict(thick=0.0094, palm=(0.0165, 0.043)))
    pcs.append(neck_piece(dict(r=(0.06, 0.052), apple=True)))
    pcs.append(head_piece(dict(w=1.0, face=1.0, ears=True)))
    return pcs


def staff_colour(tag, p, pc):
    if tag == 'skin':
        c = mul3(SKIN, 1 + 0.03 * mnoise.noise(p * 40.0))
        for n in ('L', 'R'):
            k = nail_at(p, n, dict(thick=0.0094))
            if k: c = mix3(c, hexc('#E2D2C8'), k)
        return c
    if tag == 'shoe':
        return PAL['sole'] if p.y < 0.025 else mul3(PAL['shoe'], 1 + 0.4 * mnoise.noise(p * 90.0) * 0.2)
    if tag == 'badge' and BADGE:
        q = p - BADGE['c']
        u, v = q.dot(BADGE['x']), q.dot(BADGE['y'])
        if v > 0.008: return hexc('#2E4E7A')                         # a blue band across the top
        if abs(v + 0.002) < 0.0024 and abs(u) < 0.026: return hexc('#B9B6AE')   # a line where a name would be
        return PAL['badge']
    base = PAL.get(tag, SKIN)
    if tag in ('shirt', 'trousers', 'belt'):
        return mul3(base, 1 + fabric(p))
    return base


def mul3(c, k): return tuple(clamp(x * k) for x in c)


def staff_paint(p, n, cav):
    c = tag_mix(p, staff_colour)
    k = 1.0 - 0.6 * cav
    return tuple(ch * k for ch in c)


BODY = dict(name='stBody', pieces=staff_pieces, paint=staff_paint,
            body=dict(voxel=0.0035, smooth=(3, 0.5), noise=0.0004, tris=11000),
            fine=dict(voxel=0.0021, smooth=(2, 0.5), noise=0.0002, tris=3000))
