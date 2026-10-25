"""
The human body under the Showroom's staff and the Borrowed House's shadows.
Not a body on its own: tools/blender/skin_staff.py and skin_shadow.py exec
it, in skinned.py's namespace (C = creatures.py's helpers, Piece, V, BONES,
bone_matrix, math), and build their bodies from these parts.

Everything is modelled in world space on the rig's rest pose (the rig is
humanRig() in backrooms.js, dumped by tools/rig-dump.mjs), so every shape
sits on its bones exactly: a part is placed with a bone's rest frame
(`fr('forearm.L') @ V(...)`: +Y back up the bone, the bone running down -Y,
+Z forward, +X the figure's left), or straight from the bones' heads and
tails. Mirrored parts take s = +1 (left, +X) or -1 (right).

A piece made with WP() is weighted to its bones (several: split along the
bone segments over `sig` metres) and carries a `tag` (shirt, skin, shoe...)
that the painters read back: a vertex takes the colours of the pieces whose
surfaces it lies on, blended over a couple of millimetres.
"""
import bmesh
from mathutils import Matrix
from mathutils import noise as mnoise

ell, tube, chain, sweep, loft, Mx = C['ell'], C['tube'], C['chain'], C['sweep'], C['loft'], C['M']
lerp, smooth, clamp, spow, hexc, mix3 = C['lerp'], C['smooth'], C['clamp'], C['spow'], C['hexc'], C['mix3']
TAU = math.tau


def Hd(n): return V(BONES[n]['head'])
def Tl(n): return V(BONES[n]['tail'])
def fr(n): return bone_matrix(n)


def ellf(bm, F, c, r, rot=(0, 0, 0), seg=20):
    """an ellipsoid modelled in frame F (a rigid 4x4)"""
    bmesh.ops.create_uvsphere(bm, u_segments=seg, v_segments=max(8, seg // 2), radius=1.0, matrix=F @ Mx(c, rot, r))


def tubef(bm, F, a, b, ra, rb, seg=16):
    tube(bm, F @ V(a), F @ V(b), ra, rb, seg)


def boxf(bm, F, c, size, rot=(0, 0, 0)):
    bmesh.ops.create_cube(bm, size=1.0, matrix=F @ Mx(c, rot, size))


def in_frame(bm, F, build):
    """run build(bm2) in frame F's local coordinates and add the result to bm"""
    b2 = bmesh.new()
    build(b2)
    C['xform'](b2, F)
    me = bpy.data.meshes.new('tmp')
    b2.to_mesh(me); b2.free()
    bm.from_mesh(me)
    bpy.data.meshes.remove(me)


def ridge(bm, pts, r, seg=8):
    """a fold or seam: a thin tube through pts, tapering to nothing at both ends"""
    n = len(pts)
    rs = [r * math.sin(math.pi * (0.15 + 0.7 * i / (n - 1))) for i in range(n)]
    sweep(bm, pts, rs, sides=seg)


PIECES = []
def WP(build, bones, tag, group='body', sig=0.04, bias=0.0):
    """a piece modelled in world space, weighted to `bones`, painted as `tag`"""
    pc = Piece(build, bones[0], list(bones), group=group, local=fr(bones[0]).inverted(), sig=sig)
    pc.tag, pc.bias = tag, bias
    PIECES.append(pc)
    return pc


# ------------------------------------------------------------- the torso ---
# A vertical loft through superelliptic sections (world y, half-width, front
# z, back z (negative)), and points on it: tpt(ST, y, a, out) is the surface
# at height y, angle a round Y from the front (+Z) towards +X, pushed out
# along the surface normal by `out`.
def tpt(ST, y, a, out=0.0, expo=2.3):
    sp = C['sec_pt_y']
    p = sp(ST, y, a, expo)
    if out:
        t = sp(ST, y, a + 1e-3, expo) - sp(ST, y, a - 1e-3, expo)
        n = V((-t.z, 0, t.x))
        if n.length > 1e-9: p = p + n.normalized() * out
    return p


def torso_loft(bm, ST, n=72, expo=2.3):
    C['loft_y'](bm, ST, n=n, expo=expo)


def band(bm, ST, y0, y1, out, n=72, rows=3, expo=2.3):
    """a closed band round the torso between y0 and y1, `out` proud of it"""
    rings = [[tpt(ST, lerp(y0, y1, j / (rows - 1)), TAU * i / n, out, expo) for i in range(n)] for j in range(rows)]
    loft(bm, rings)


def surf_path(ST, ya, aa, yb, ab, out, k=8, expo=2.3):
    return [tpt(ST, lerp(ya, yb, i / (k - 1)), lerp(aa, ab, i / (k - 1)), out, expo) for i in range(k)]


# ------------------------------------------------------------- the hands ---
def hand_pieces(n, s, K):
    """the hand: a palm on the hand bone and a piece per finger bone (fine
    group). K: thick (finger radius at the knuckle), palm (half thickness,
    half width), nails (True), tag."""
    pcs = []
    H = fr('hand.' + n)
    th = K['thick']
    pw, pt = K['palm']
    tag = K.get('tag', 'skin')
    side = H.to_3x3() @ V((s, 0, 0))       # the back of the hand (world)
    L = abs(Hd('finger10.' + n).y - Hd('hand.' + n).y)   # roughly the palm's length
    def palm(bm):
        ellf(bm, H, (s * 0.002, -L * 0.5, 0.0), (pt, L * 0.56, pw))                    # the palm
        ellf(bm, H, (0, -0.006, 0), (pt * 0.95, 0.022, pw * 0.72))                      # the wrist
        ellf(bm, H, (-s * pt * 0.45, -L * 0.42, pw * 0.62), (pt * 0.85, L * 0.33, pw * 0.42), rot=(0.25, 0, 0))   # thumb ball
        ellf(bm, H, (-s * pt * 0.4, -L * 0.62, -pw * 0.62), (pt * 0.7, L * 0.34, pw * 0.3))                   # heel of the hand
        for k in range(4):                                                              # knuckles and the tendons to them
            m = Hd(f'finger{k}0.{n}')
            ell(bm, m + side * th * 0.25, (th * 1.15, th * 1.15, th * 1.15), seg=10)
            tube(bm, H @ V((s * pt * 0.6, -0.012, pw * 0.25 * (1.5 - k))), m + side * th * 0.55, th * 0.32, th * 0.38, seg=8)
    pcs.append(WP(palm, ['hand.' + n], tag, group='fine'))
    for k in range(5):
        bones = [f'finger{k}{j}.{n}' for j in range(3)]
        pts = [Hd(b) for b in bones] + [Tl(bones[2])]
        sc = (1.0, 1.04, 0.98, 0.86, 1.14)[k]
        rad = [th * sc * f for f in (1.0, 0.9, 0.82, 0.68)]
        for j in range(3):
            def ph(bm, j=j, pts=pts, rad=rad, k=k):
                a, b = pts[j], pts[j + 1]
                tube(bm, a, b, rad[j], rad[j + 1], seg=10)
                ell(bm, a, (rad[j] * 1.08,) * 3, seg=10)                              # the joint
                if j == 2:
                    d = (b - a).normalized()
                    ell(bm, b - d * rad[3] * 0.2 - side * rad[3] * 0.15, (rad[3] * 1.05,) * 3, seg=10)   # the pad
                    if K.get('nails', True):                                         # the nail, on the back
                        c = b - d * rad[3] * 1.3 + side * rad[3] * 0.62
                        ell(bm, c, (rad[3] * 0.8, rad[3] * 0.8, rad[3] * 0.8), seg=8)
            pcs.append(WP(ph, [bones[j]], tag, group='fine'))
    return pcs


def nail_at(p, n, K):
    """0..1: how much p is a fingernail of the hand n (for the painters)"""
    s = 1 if n == 'L' else -1
    H = fr('hand.' + n)
    side = H.to_3x3() @ V((s, 0, 0))
    best = 0.0
    for k in range(5):
        a, b = Hd(f'finger{k}2.{n}'), Tl(f'finger{k}2.{n}')
        d = (b - a).normalized()
        r = K['thick'] * (1.0, 1.04, 0.98, 0.86, 1.14)[k] * 0.68
        c = b - d * r * 1.3 + side * r * 0.62
        q = p - c
        if q.length > r * 2.2: continue
        if q.dot(side) < r * 0.15: continue
        best = max(best, 1.0 - smooth(r * 0.75, r * 1.5, q.length))
    return best


# -------------------------------------------------------------- the arms ---
def arm_len(n):
    return (Hd('forearm.' + n) - Hd('upperarm.' + n)).length, (Hd('hand.' + n) - Hd('forearm.' + n)).length


def arm_pieces(n, s, K):
    """the bare arm, upper arm and forearm as one piece over both bones (the
    elbow bends smoothly). K: up = (r at the shoulder, r at the elbow),
    fore = (r below the elbow, r at the wrist), bulk (muscle, 0..1),
    creases (wrinkles at the elbow), tag."""
    U, Fo = fr('upperarm.' + n), fr('forearm.' + n)
    A, Fl = arm_len(n)
    r0, r1 = K['up']
    f0, f1 = K['fore']
    bulk = K.get('bulk', 1.0)
    def b(bm):
        tubef(bm, U, (0, -0.045, 0), (0, -A, 0), r0, r1, seg=20)
        ellf(bm, U, (0, -0.03, 0), (r0 * 0.9, 0.05, r0 * 0.9))                                                 # into the shoulder
        ellf(bm, U, (s * 0.003, -A * 0.5, r0 * 0.28), (r0 * 0.82, A * 0.3, r0 * (0.6 + 0.25 * bulk)))      # biceps
        ellf(bm, U, (0, -A * 0.42, -r0 * 0.3), (r0 * 0.88, A * 0.36, r0 * (0.62 + 0.25 * bulk)))           # triceps
        ellf(bm, U, (0, -A, -r1 * 0.12), (r1 * 1.08, r1 * 0.95, r1 * 1.0))                                  # the elbow
        ellf(bm, U, (0, -A - 0.004, -r1 * 0.78), (r1 * 0.45, r1 * 0.6, r1 * 0.36))                          # its point
        tubef(bm, Fo, (0, 0.0, 0), (0, -Fl, 0), f0, f1, seg=20)
        ellf(bm, Fo, (s * f0 * 0.3, -Fl * 0.24, f0 * 0.2), (f0 * (0.85 + 0.2 * bulk), Fl * 0.3, f0 * 0.95))   # the forearm's muscle
        ellf(bm, Fo, (-s * f0 * 0.3, -Fl * 0.3, -f0 * 0.1), (f0 * 0.8, Fl * 0.3, f0 * 0.85))
        tubef(bm, Fo, (0, -0.02, -f0 * 0.75), (0, -Fl, -f1 * 0.6), f0 * 0.18, f1 * 0.2, seg=8)              # the ulna's ridge
        ellf(bm, Fo, (0, -Fl, 0), (f1 * 0.8, 0.018, f1 * 1.15))                                              # the wrist
        ellf(bm, Fo, (s * f1 * 0.85, -Fl + 0.004, -f1 * 0.4), (f1 * 0.28, f1 * 0.3, f1 * 0.28), seg=10)       # its knob
        if K.get('creases', True):
            # the inside of the elbow: two soft creases across it, and a few
            # fine ones on its point
            for k, y in enumerate((0.012, -0.006)):
                pts = [Fo @ V((math.sin(a) * f0 * 1.02, y - 0.004 * math.cos(a) ** 2, math.cos(a) * f0 * 1.02))
                       for a in [lerp(-0.95, 0.95, i / 6) for i in range(7)]]
                ridge(bm, pts, 0.0034 - 0.0006 * k)
            for k, y in enumerate((-A + 0.012, -A - 0.006, -A - 0.022)):
                pts = [U @ V((math.sin(a) * r1 * 0.85, y, -math.cos(a) * r1 * 1.0)) for a in [lerp(-0.7, 0.7, i / 5) for i in range(6)]]
                ridge(bm, pts, 0.0026)
    return WP(b, ['upperarm.' + n, 'forearm.' + n], K.get('tag', 'skin'), sig=0.03)


# ------------------------------------------------------- the neck and head ---
def neck_piece(K):
    """K: r = (at the base, under the skull), tag; sterno (the tendons)"""
    Hh = fr('head')
    r0, r1 = K['r']
    def b(bm):
        base, top = Hd('neck') + V((0, -0.04, 0.008)), Hh @ V((0, 0.045, -0.012))
        tube(bm, base, top, r0, r1, seg=24)
        for s in (-1, 1):
            tube(bm, Hh @ V((s * 0.046, 0.035, -0.028)), Hd('neck') + V((s * 0.014, -0.008, 0.05)), 0.0125 * K.get('sterno', 1.0), 0.011, seg=10)
        if K.get('apple'): ell(bm, Hh @ V((0, -0.06, 0.04)), (0.012, 0.016, 0.012))
    return WP(b, ['neck', 'head'], K.get('tag', 'skin'), sig=0.035)


def head_piece(K):
    """a head with nothing on it: the shape where a face should be. K: w
    (width scale), face (how much brow and nose shows under the skin, 0..1),
    ears, tag"""
    Hh = fr('head')
    w, f = K.get('w', 1.0), K.get('face', 1.0)
    crown = (Tl('head') - Hd('head')).length
    def b(bm):
        ellf(bm, Hh, (0, crown - 0.125, -0.012), (0.082 * w, 0.125, 0.102), seg=36)              # the skull, a little too tall
        ellf(bm, Hh, (0, 0.032, 0.014), (0.066 * w, 0.082, 0.075), seg=32)                       # the face and the jaw
        ellf(bm, Hh, (0, -0.03, 0.045), (0.027 * w, 0.024, 0.024), seg=20)                       # the chin
        for s in (-1, 1):
            ellf(bm, Hh, (s * 0.048 * w, -0.006, -0.004), (0.022, 0.03, 0.034))                  # the angles of the jaw, soft
            if K.get('ears'):
                ellf(bm, Hh, (s * 0.08 * w, 0.068, -0.018), (0.01, 0.027, 0.017), rot=(0.1, s * 0.35, 0))
        # the face: only the brow and the bridge of a nose, under the skin
        ellf(bm, Hh, (0, 0.118, 0.066), (0.056 * w, 0.011, 0.021 + 0.002 * f))
        tubef(bm, Hh, (0, 0.104, 0.079), (0, 0.062, 0.078 + 0.005 * f), 0.006, 0.0072 + 0.0012 * f, seg=14)
    return WP(b, ['head'], K.get('tag', 'skin'))


# -------------------------------------------------------- the legs (bare) ---
def leg_len(n):
    return (Hd('shin.' + n) - Hd('thigh.' + n)).length, (Hd('foot.' + n) - Hd('shin.' + n)).length


def leg_skin(n, s, K):
    """a bare leg, thigh and shin as one piece. K: thigh = (r top, r above
    the knee), shin = (r below the knee, calf, r at the ankle), tag"""
    T = fr('thigh.' + n)
    Lt, Ls = leg_len(n)
    t0, t1 = K['thigh']
    c0, cm, c1 = K['shin']
    def b(bm):
        tubef(bm, T, (0, 0.03, 0), (0, -Lt, 0), t0, t1, seg=20)
        ellf(bm, T, (s * 0.004, -Lt * 0.42, t0 * 0.22), (t0 * 0.86, Lt * 0.36, t0 * 0.78))                 # the front of the thigh
        ellf(bm, T, (0, -Lt * 0.4, -t0 * 0.25), (t0 * 0.84, Lt * 0.36, t0 * 0.76))                         # the back
        ellf(bm, T, (0, -Lt, 0.002), (t1 * 1.08, t1 * 1.0, t1 * 1.02))                                      # the knee
        ellf(bm, T, (0, -Lt + 0.012, t1 * 0.8), (t1 * 0.62, t1 * 0.7, t1 * 0.36))                           # its cap
        tubef(bm, T, (0, -Lt, 0), (0, -Lt - Ls + 0.01, 0), c0, c1, seg=20)
        ellf(bm, T, (0, -Lt - Ls * 0.3, -cm * 0.4), (cm * 0.95, Ls * 0.24, cm * 0.85))                     # the calf
        tubef(bm, T, (0, -Lt - 0.02, c0 * 0.75), (0, -Lt - Ls + 0.04, c1 * 0.7), c0 * 0.22, c1 * 0.25, seg=8)   # the shin bone
        tubef(bm, T, (0, -Lt - Ls * 0.55, -c1 * 0.9), (0, -Lt - Ls + 0.01, -c1 * 1.05), c1 * 0.35, c1 * 0.35, seg=8)   # the heel cord
        for k in (-1, 1):
            ellf(bm, T, (k * c1 * 0.85, -Lt - Ls + 0.004, -0.002 * k * s), (c1 * 0.38, c1 * 0.42, c1 * 0.42), seg=10)   # the ankle bones
    return WP(b, ['thigh.' + n, 'shin.' + n], K.get('tag', 'skin'), sig=0.035)


def foot_bare(n, s, K):
    """a bare foot on the ankle and the ball of the foot. K: w (half width at
    the ball), toes (thickness), tag"""
    Fo = fr('foot.' + n)
    ankY = Hd('foot.' + n).y
    bz = (Hd('toe.' + n) - Hd('foot.' + n)).z
    heel = K['heel']
    w = K['w']
    tl = (Tl('toe.' + n) - Hd('toe.' + n)).length
    def b(bm):
        g = -ankY        # the floor, in the foot's frame
        ellf(bm, Fo, (0, g + 0.032, -heel + 0.026), (w * 0.62, 0.032, 0.034))                          # the heel
        ellf(bm, Fo, (0, 0.004, -0.004), (w * 0.62, 0.034, 0.036))                                     # the ankle
        loft(bm, [[Fo @ V((x * w * f + (-s * 0.006 * (1 - f)), g + y, z)) for x, y in
                   [(math.cos(a), 0.018 + 0.017 * math.sin(a) + (0.02 + 0.022 * (1 - z / bz)) * max(0.0, math.sin(a)))
                    for a in [TAU * i / 20 for i in range(20)]]]
                  for z, f in ((-heel + 0.03, 0.62), (0.0, 0.66), (bz * 0.45, 0.78), (bz * 0.8, 0.95), (bz + 0.008, 1.0))])
        for k in range(5):                                                                            # the toes, the big one inside
            x = -s * w * (0.62 - k * 0.32)
            L = tl * (1.0, 0.86, 0.78, 0.7, 0.62)[k]
            r = K['toes'] * (1.35, 0.95, 0.88, 0.82, 0.76)[k]
            a = Fo @ V((x, g + r + 0.002, bz - 0.006))
            c = Fo @ V((x * 1.05, g + r * 0.85, bz + L))
            tube(bm, a, c, r, r * 0.85, seg=10)
            ell(bm, c, (r * 0.95, r * 0.85, r * 1.0), seg=10)
    return WP(b, ['foot.' + n, 'toe.' + n], K.get('tag', 'skin'), sig=0.02)


# ------------------------------------------------------------ the painter ---
def tag_mix(p, colour, rad=0.0022):
    """colour(tag, p, piece) of the pieces whose surfaces p lies on, blended
    over `rad` metres (a piece's `bias` lets small details win near edges)"""
    ds = []
    for pc in PIECES:
        if not pc.near(p, 0.025): continue
        hit = pc.bvh.find_nearest(p)
        if hit[0] is None: continue
        ds.append((hit[3] - pc.bias, pc))
    if not ds: return colour('skin', p, None)
    dmin = min(d for d, _ in ds)
    acc, tot = V((0, 0, 0)), 0.0
    for d, pc in ds:
        w = math.exp(-(d - dmin) / rad)
        if w < 0.02: continue
        acc += V(colour(pc.tag, p, pc)) * w
        tot += w
    return tuple(acc / tot)


def fabric(p, k=1.0):
    """a fine weave and slow mottling, about 0"""
    return 0.035 * mnoise.noise(p * 260.0) + 0.03 * mnoise.noise(p * 23.0) * k
