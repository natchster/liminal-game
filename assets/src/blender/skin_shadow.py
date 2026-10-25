"""
The Borrowed House's shadows (inspired by SCP-1983): the human form, gaunt
and a little too long in every bone, made of darkness. A starved body (ribs,
collarbones, the knobs of the spine, knees and elbows like knots, long thin
fingers, bare feet) and a smooth head with less of a face than the staff's;
smoke wisps rise off the head and shoulders, trail from the elbows, heels
and fingertips.

Run by tools/blender/skinned.py (`... --python tools/blender/skinned.py --
shadow`) on the rig makeShadow builds (HUMAN_RIGS.shadow in backrooms.js,
dumped by `node tools/rig-dump.mjs shadow`). The game draws it with
shadowMat / shadowEdgeMat (flat colour, no albedo), so the vertex colour
carries data instead: red is how much of a wisp a vertex is (0 at the root,
1 at the tip), which the smoke shader frays and stirs; green is the crease
(the painter's cavity).
"""
exec(compile(open(os.path.join(HERE, 'skin_human.py')).read(), os.path.join(HERE, 'skin_human.py'), 'exec'), globals())

ST_BODY = [   # world y, half-width, front z, back z: starved, the waist pinched
    (0.99, 0.04, 0.025, -0.035), (1.02, 0.098, 0.05, -0.074), (1.06, 0.126, 0.064, -0.093),
    (1.11, 0.131, 0.068, -0.093), (1.16, 0.122, 0.066, -0.083), (1.22, 0.107, 0.056, -0.073),
    (1.29, 0.103, 0.052, -0.071), (1.36, 0.113, 0.068, -0.079), (1.44, 0.127, 0.087, -0.089),
    (1.52, 0.135, 0.095, -0.097), (1.6, 0.138, 0.097, -0.101), (1.67, 0.138, 0.092, -0.099),
    (1.72, 0.134, 0.078, -0.091), (1.765, 0.118, 0.064, -0.08), (1.805, 0.08, 0.05, -0.064),
    (1.85, 0.048, 0.04, -0.05),
]
WISPS = []    # (piece, path): the smoke's polylines, root first


def wisp(pts, r0, bones, group='body'):
    """a tendril of smoke along pts (root first), tapering to nothing"""
    pts = [V(p) for p in pts]
    n = len(pts)
    def b(bm, pts=pts):
        sweep(bm, pts, [r0 * (1 - i / (n - 1)) ** 0.8 for i in range(n)], sides=8)
    pc = WP(b, bones, 'wisp', group=group, sig=0.03)
    WISPS.append((pc, pts))
    return pc


def curl(root, d, length, up, k=7, twist=0.0):
    """a smoke path from root along d, bending towards `up` as it goes"""
    d, up = V(d).normalized(), V(up).normalized()
    pts, p = [V(root)], V(root)
    for i in range(1, k):
        t = i / (k - 1)
        dd = (d * (1 - t) + up * t + V((math.sin(twist * t * 3) * 0.3, 0, math.cos(twist * t * 3) * 0.3)) * twist).normalized()
        p = p + dd * (length / (k - 1))
        pts.append(p.copy())
    return pts


def torso_pieces():
    pcs = []
    TB = ['pelvis', 'abdomen', 'chest']
    def body(bm):
        torso_loft(bm, ST_BODY)
        for i in range(6):                                                         # ribs, standing out under the skin
            y0 = 1.39 + i * 0.045
            for s in (-1, 1):
                ridge(bm, [tpt(ST_BODY, y0 - 0.05 * (1 - j / 8) ** 1.3, s * lerp(0.35, 2.4, j / 8), 0.0) for j in range(9)], 0.0062)
        ridge(bm, [tpt(ST_BODY, y, 0.0, 0.0) for y in (1.68, 1.6, 1.52, 1.45)], 0.008)   # the breastbone
        for k in range(15):                                                        # the spine's knobs
            y = lerp(1.13, 1.8, k / 14)
            ell(bm, tpt(ST_BODY, y, math.pi, -0.002), (0.011, 0.012, 0.012), seg=10)
        for s in (-1, 1):
            ellf(bm, Matrix(), tpt(ST_BODY, 1.58, s * 2.5, -0.012), (0.045, 0.07, 0.016), rot=(0.2, s * 0.5, s * 0.15))   # shoulder blades
            tube(bm, tpt(ST_BODY, 1.76, s * 0.18, -0.002), Hd('upperarm.' + ('L' if s > 0 else 'R')) + V((-s * 0.02, 0.022, 0.03)), 0.011, 0.01, seg=10)   # collarbones
            ell(bm, tpt(ST_BODY, 1.135, s * 1.05, -0.004), (0.032, 0.02, 0.04))  # hip bones
            tube(bm, Hd('neck') + V((s * 0.03, -0.02, -0.02)), Hd('upperarm.' + ('L' if s > 0 else 'R')) + V((-s * 0.02, 0.02, -0.005)), 0.026, 0.022, seg=12)   # the shoulder's slope
    pcs.append(WP(body, TB, 'skin', sig=0.05))
    for s, n in ((1, 'L'), (-1, 'R')):
        U = fr('upperarm.' + n)
        def cap(bm, U=U, s=s):
            ellf(bm, U, (s * 0.004, -0.03, 0.0), (0.042, 0.06, 0.042), seg=20)          # a bony shoulder
        pcs.append(WP(cap, ['clavicle.' + n, 'upperarm.' + n], 'skin', sig=0.035))
    return pcs


def wisps():
    pcs = []
    Hh = fr('head')
    up = V((0, 1, 0))
    # off the back of the head, streaming back and up
    for k, (x, y, z, dx, dz, L) in enumerate(((0.0, 0.17, -0.085, 0.0, -1.0, 0.17), (0.045, 0.13, -0.085, 0.45, -1.0, 0.14),
                                              (-0.05, 0.12, -0.08, -0.5, -1.0, 0.15), (0.02, 0.06, -0.09, 0.15, -1.0, 0.12))):
        root = Hh @ V((x, y, z))
        pcs.append(wisp(curl(root, (dx, 0.15, dz), L, (dx * 0.3, 0.8, -0.6), twist=0.15 * (k - 1.5)), 0.012, ['head']))
    # off the shoulders and the upper back
    for s, n in ((1, 'L'), (-1, 'R')):
        for k, (a, y, L) in enumerate(((s * 2.3, 1.72, 0.14), (s * 2.75, 1.62, 0.12))):
            root = tpt(ST_BODY, y, a, -0.004)
            pcs.append(wisp(curl(root, (s * 0.3, 0.7, -0.6), L, up, twist=0.1 * s), 0.014, ['chest']))
        root = Hd('upperarm.' + n) + V((s * 0.02, 0.035, -0.01))
        pcs.append(wisp(curl(root, (s * 0.6, 0.6, -0.3), 0.12, up), 0.013, ['clavicle.' + n, 'upperarm.' + n]))
        # trailing from the elbows and the heels
        Fo = fr('forearm.' + n)
        pcs.append(wisp(curl(Fo @ V((0, 0.0, -0.03)), Fo.to_3x3() @ V((0, -0.3, -1)), 0.13, V((0, 0.2, -1))), 0.012, ['upperarm.' + n, 'forearm.' + n]))
        F = fr('foot.' + n)
        pcs.append(wisp(curl(F @ V((0, -0.03, -0.05)), (s * 0.2, 0.1, -1), 0.16, (0, 0.5, -1)), 0.016, ['foot.' + n]))
        pcs.append(wisp(curl(fr('shin.' + n) @ V((0, -0.3, -0.035)), (s * 0.3, 0.2, -1), 0.12, (0, 0.6, -1)), 0.013, ['shin.' + n]))
        # the fingers going on into threads
        for k in range(4):
            b = f'finger{k}2.{n}'
            a, t = Hd(b), Tl(b)
            d = (t - a).normalized()
            pcs.append(wisp([t - d * 0.004, t + d * 0.025, t + d * 0.05 + V((0, -0.004, -0.006 * k)), t + d * 0.072 + V((0.004 * s, -0.01, -0.012))],
                            0.0056, [b], group='fine'))
    return pcs


def shadow_pieces():
    PIECES.clear(); WISPS.clear()
    pcs = torso_pieces()
    for s, n in ((1, 'L'), (-1, 'R')):
        pcs.append(leg_skin(n, s, dict(thigh=(0.068, 0.046), shin=(0.044, 0.05, 0.026))))
        pcs.append(foot_bare(n, s, dict(w=0.04, heel=0.06, toes=0.0082)))
        pcs.append(arm_pieces(n, s, dict(up=(0.037, 0.03), fore=(0.031, 0.021), bulk=0.15, creases=False)))
        pcs += hand_pieces(n, s, dict(thick=0.0078, palm=(0.0125, 0.037), nails=False))
    pcs.append(neck_piece(dict(r=(0.042, 0.037), sterno=0.85)))
    pcs.append(head_piece(dict(w=0.94, face=0.3, ears=False)))
    pcs += wisps()
    return pcs


def wisp_at(p):
    """how far along its wisp p is (0 on the body, 1 at a wisp's tip)"""
    best, bd = 0.0, 1e9
    for pc in PIECES:
        if not pc.near(p, 0.02): continue
        d = pc.bvh.find_nearest(p)[3]
        if d < bd: bd, best = d, pc
    if best is None or best.tag != 'wisp': return 0.0
    path = next(pts for pc, pts in WISPS if pc is best)
    tot = sum((path[i + 1] - path[i]).length for i in range(len(path) - 1))
    acc, bestd, bestt = 0.0, 1e9, 0.0
    for i in range(len(path) - 1):
        a, b = path[i], path[i + 1]
        ab = b - a
        L = ab.length
        u = 0.0 if L < 1e-9 else max(0.0, min(1.0, (p - a).dot(ab) / (L * L)))
        dd = (p - (a + ab * u)).length
        if dd < bestd: bestd, bestt = dd, (acc + u * L) / tot
        acc += L
    return smooth(0.05, 1.0, bestt)


def shadow_paint(p, n, cav):
    w = wisp_at(p)
    # sRGB bytes: the game decodes them to linear, so encode the values to arrive as they are
    return (w ** (1 / 2.2), cav ** (1 / 2.2), 0.0)


BODY = dict(name='shBody', pieces=shadow_pieces, paint=shadow_paint,
            body=dict(voxel=0.0036, smooth=(3, 0.5), noise=0.0006, tris=8600),
            fine=dict(voxel=0.002, smooth=(2, 0.5), noise=0.0002, tris=2000))
