"""
The Smiler: someone very tall, starved down to tendon, gone black all over
as if tarred. Its lips have dried back off the teeth, top and bottom, so it
seems to grin round to its cheeks (a rictus, not a smile). It stoops under
the ceilings; its hands hang past its knees. (docs/design/ENTITIES.md, design 1.)

MakeHuman's young, thin man (CC0), every limb and the neck lengthened (the
forearms and hands most), posed in its stoop (the rest pose the game
binds); wasted to the bone; its lips pushed back off MakeHuman's own teeth;
its eyes deep under the brow (they carry the eyeshine the game drives from
your lamp: `entities::smiler`).

Close (E14, docs/research/creature-horror-references.md): its black is a
crust, not a skin: dried, cracked into plates whose cracks hold pale dust
and show something red-brown under them; the cords of the neck, the
collarbones and breastbone, the hip bones, kneecaps and elbows stand out
of it; tendons run over the backs of its hands to knuckles like knots; its
nails are long and black; its teeth are not a denture's: stained ivory to
brown, uneven, some gone, tartar at the gums. Near, about 60k triangles;
from 6 m a far body of under 20k (`kit.far_copy`).
"""
import numpy as np

ORDER = 20

STRETCH = {
    'hips': 1.1, 'spine': 1.28, 'chest': 1.22, 'upperchest': 1.15, 'neck': 1.7,
    'thigh.L': 1.32, 'thigh.R': 1.32, 'shin.L': 1.36, 'shin.R': 1.36,
    'upperarm.L': 1.35, 'upperarm.R': 1.35, 'forearm.L': 1.62, 'forearm.R': 1.62,
    'hand.L': 1.45, 'hand.R': 1.45,
}
for f in range(1, 6):
    for k in range(1, 4):
        for side in ('.L', '.R'):
            STRETCH[f'finger{f}{k}{side}'] = 1.45 if f > 1 else 1.2


def pose_of(skel):
    """the stoop: a hunch, the neck thrust forward, the head level (it looks
    at you), the arms hanging a little ahead, the knees bent"""
    # (its eyes at 1.8-1.95 m: it looks down at you, its head forward of
    # its shoulders, not under them)
    aims = {
        'hips': (0, 1, 0.02), 'spine': (0, 1, 0.12), 'chest': (0, 1, 0.3), 'upperchest': (0, 1, 0.55),
        'neck': (0, 1, 0.75), 'head': (0, 1, 0.2), 'jaw': (0, -0.75, 0.65),
    }
    for side, sx in (('.L', 1), ('.R', -1)):
        # (one shoulder lower than the other)
        aims['clavicle' + side] = (sx, 0.05 if side == '.L' else -0.06, 0.25)
        aims['upperarm' + side] = (sx * 0.12, -1, 0.18)
        aims['forearm' + side] = (sx * 0.04, -1, 0.3)
        aims['hand' + side] = (0, -1, 0.15)
        aims['thigh' + side] = (sx * 0.06, -1, 0.18)
        aims['shin' + side] = (0, -1, -0.14)
        aims['foot' + side] = (0, -0.45, 1)
        aims['toes' + side] = (0, -0.1, 1)
    return aims


def smooth01(x):
    x = np.clip(x, 0.0, 1.0)
    return x * x * (3 - 2 * x)


def sculpt_face(Q, N, eyes, arch, bite, gum_u, gum_l):
    """the skull under the skin, in the head's rest frame (+z ahead, +y up;
    Q points, N normals): deep orbits under a brow, cheekbones over hollow
    cheeks, sunken temples, a shrunken nose, and the lips gone from cheek
    to cheek: inside the rictus the skin goes in behind the teeth (`arch`,
    meeting at `bite`), and above and below them it is gum (from the gum
    lines out to the lips' torn edge). Returns the points and the masks of
    the gums and of the dark inside"""
    Q = Q.copy()
    face = N[:, 2] > -0.2
    for E in eyes:
        c = E.mean(0)
        r = np.linalg.norm(E - c, axis=1).max()
        Ro = 1.9 * r
        rho = np.hypot(Q[:, 0] - c[0], (Q[:, 1] - c[1]) * 1.15)
        # the pit: the (sunken) eye's front just shows at its bottom
        w = smooth01(1.15 - rho / Ro) * (Q[:, 2] > c[2] - r)
        bottom = c[2] - 0.5 * r + r * (rho / Ro) ** 2
        Q[:, 2] -= w * np.maximum(Q[:, 2] - bottom, 0)
        # the brow ridge over it
        brow = smooth01(1 - np.abs(rho - 1.1 * Ro) / (0.8 * r)) * smooth01((Q[:, 1] - c[1]) / r) * (N[:, 2] > 0.2)
        Q[:, 2] += 0.004 * brow
    cy = np.mean([E[:, 1].mean() for E in eyes])
    cz = np.mean([E[:, 2].mean() for E in eyes])
    x0 = arch.x0
    push = np.zeros(len(Q))
    def g(x, y, wx, wy):
        return np.exp(-(((Q[:, 0] - x) / wx) ** 2 + ((Q[:, 1] - y) / wy) ** 2))
    near = Q[:, 2] > cz - 0.05
    for sx in (1, -1):
        # hollow under the cheekbone; the cheekbone itself; the temple
        push -= 0.008 * g(x0 + sx * 0.047, bite + 0.018, 0.015, 0.016) * face * near
        push += 0.003 * g(x0 + sx * 0.048, cy - 0.024, 0.016, 0.006) * face * near
        push -= 0.005 * g(x0 + sx * 0.064, cy + 0.012, 0.01, 0.018) * (Q[:, 2] < cz) * (Q[:, 2] > cz - 0.07)
    Q += N * push[:, None]
    # the nose dried down to its bone
    ridge = (np.abs(Q[:, 0] - x0) < 0.012) & (Q[:, 1] > gum_u) & (Q[:, 1] < cy - 0.01)
    if ridge.any():
        tip = Q[ridge][np.argmax(Q[ridge, 2])]
        Q[:, 2] -= 0.006 * np.exp(-(np.linalg.norm(Q - tip, axis=1) / 0.02) ** 2)
    # the rictus: a wide opening, its corners drawn up and back
    Wo = arch.W + 0.007
    x = Q[:, 0] - x0
    e = np.sqrt(np.clip(1 - (x / Wo) ** 2, 0, 1)) ** 0.5
    # (one side drawn higher than the other: dried, not set)
    lift = 0.004 * (x / Wo) ** 2 + 0.0018 * np.clip(x / Wo, 0, 1)
    top = bite + (gum_u + 0.0025 - bite) * e + lift
    bot = bite - (bite - gum_l + 0.0025) * e + lift
    d = np.minimum(np.minimum(Q[:, 1] - bot, top - Q[:, 1]), Wo - np.abs(x))
    front = arch.front(Q[:, 0])
    ahead = Q[:, 2] > front - 0.012
    inside = smooth01(d / 0.0025) * ahead
    gums = inside * ((Q[:, 1] > gum_u) | (Q[:, 1] < gum_l))
    hole = inside - gums
    target = np.where(gums > 0, front + 0.0008, front - 0.009)
    Q[:, 2] -= inside * np.clip(Q[:, 2] - target, 0, 0.03)
    # the lips' torn rim, drawn tight back toward the teeth
    rim = smooth01(1 + d / 0.004) * (d < 0) * ahead
    Q[:, 2] -= 0.6 * rim * np.clip(Q[:, 2] - (front + 0.004), 0, 0.02)
    return Q, gums, hole


TORSO = ('hips', 'spine', 'chest', 'upperchest')
LIMBS = ('upperarm', 'forearm', 'thigh', 'shin', 'hand')


def build(kit):
    import hm08, human, anatomy as A
    base = hm08.Base().target('macro/male-young-thin')
    o, skel = human.body(kit, base, 'smiler')
    teeth_u, teeth_u_f = base.helper('upper-teeth')
    teeth_l, teeth_l_f = base.helper('lower-teeth')
    eye_l, eye_l_f = base.helper('l-eye')
    eye_r, eye_r_f = base.helper('r-eye')
    # the teeth: single teeth along MakeHuman's arch, widened (they run on
    # round toward the cheeks), long (the gums have drawn back off them)
    xs = np.vstack([teeth_u, teeth_l])
    mid_u = teeth_u[np.abs(teeth_u[:, 0]) < 0.004]
    mid_l = teeth_l[np.abs(teeth_l[:, 0]) < 0.004]
    bite = (mid_u[:, 1].min() + mid_l[:, 1].max()) / 2
    # (wider than a man's: the grin must read from across a room, E10)
    W = np.abs(xs[:, 0]).max() * 1.6
    zf = mid_u[:, 2].max()
    arch_u = A.Arch(0.0, zf, W, zf - teeth_u[:, 2].min())
    arch_l = A.Arch(0.0, zf - 0.006, W - 0.003, zf - teeth_u[:, 2].min() - 0.002)
    gum_u, gum_l = bite + 0.019, bite - 0.017
    tu_v, tu_f, tu_c = A.teeth_row(arch_u, gum_u, bite - 0.0008, True, 61, missing={(1, 4), (-1, 2)},
                                   broken={(-1, 1), (1, 6)}, jitter=1.7)
    tl_v, tl_f, tl_c = A.teeth_row(arch_l, gum_l, bite + 0.0008, False, 62, missing={(-1, 6), (1, 3)},
                                   broken={(1, 0), (-1, 4)}, flare=0.002, jitter=1.7)
    # bared and dry, the one pale thing on it; but no denture: each tooth
    # its own stain (ivory, yellow, brown; one dead and grey), tartar dark
    # at the gum (E14: clean even teeth read as a mask's)
    def stain(C, seed):
        rng = np.random.RandomState(seed)
        C = C.copy()
        per = 61                                           # one tooth's points (`teeth_row`)
        # (dull ivory crowns, sRGB 150-175, browner roots: AAFS, Piombino-Mascali)
        tints = [np.array(kit.srgb(h)) for h in ('#b0a585', '#a8955f', '#8f7647', '#6f5e45', '#55524a')]
        for k in range(len(C) // per):
            t = tints[rng.choice(5, p=[0.3, 0.3, 0.2, 0.12, 0.08])]
            blk = C[k * per:(k + 1) * per]
            lum = blk.mean(axis=1, keepdims=True) / max(blk.mean(), 1e-6)
            g = np.clip((lum - 0.55) / 0.5, 0, 1)            # toward the root (dark) .. the crown
            tartar = np.array(kit.srgb('#4a3a22'))
            C[k * per:(k + 1) * per] = t * (0.75 + 0.25 * g) * (0.9 + 0.1 * rng.rand()) + (tartar - t) * (1 - g) * 0.6
        return C
    tu_c, tl_c = stain(tu_c, 63), stain(tl_c, 64)

    posed = human.pose(kit, o, skel, pose_of(skel), STRETCH, None, smooth_weights=12)
    # the head's and jaw's turn from rest to posed carries the teeth and eyes
    def frame(bone):
        """(rest → posed, posed → rest, the turn) for `bone`"""
        b0, b1 = np.array(skel.head(bone)), np.array(posed.head(bone))
        R = human._rot_between(np.array(skel.tail(bone)) - b0, np.array(posed.tail(bone)) - b1)
        return (lambda X: b1 + (X - b0) @ R.T), (lambda X: (X - b1) @ R + b0), R
    eyes0 = (eye_l.copy(), eye_r.copy())
    tl_rest = tl_v
    tu_v, eye_l, eye_r = frame('head')[0](tu_v), frame('head')[0](eye_l), frame('head')[0](eye_r)
    tl_v = frame('jaw')[0](tl_v)
    # stand it on the floor
    Pw = kit.to_game_np(kit.verts_np(o))
    lift = -Pw[:, 1].min()
    kit.set_verts_np(o, kit.to_bl_np(Pw + np.array([0, lift, 0])))
    for b in posed.bones:
        b['head'] = (b['head'][0], b['head'][1] + lift, b['head'][2])
        b['tail'] = (b['tail'][0], b['tail'][1] + lift, b['tail'][2])
    up = np.array([0, lift, 0])
    tu_v, tl_v, eye_l, eye_r = tu_v + up, tl_v + up, eye_l + up, eye_r + up
    before = kit.to_game_np(kit.verts_np(o))
    kit.remesh(o, 0.0038)
    kit.smooth(o, 2, 0.5)
    # ---- the face: the skull under it (the remesh closed the lids and lips
    # again), in the head's rest frame; the eyes sink back with it
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    to_posed, to_rest, R = frame('head')
    # (the jaw keeps its place under the head: its teeth are where they were)
    print(f'[smiler] the jaw moved its teeth {np.abs(to_rest(tl_v) - tl_rest).max() * 1000:.1f} mm against the head')
    Q, gums, hole = sculpt_face(to_rest(P), N @ R, eyes0, arch_u, bite, gum_u, gum_l)
    kit.set_verts_np(o, kit.to_bl_np(to_posed(Q)))
    def sink(E):
        c = E.mean(0)
        return to_posed(c + (E - c) * 0.88 - np.array([0, 0, 0.013]))
    eye_l, eye_r = sink(eyes0[0]), sink(eyes0[1])

    # ---- starved down to tendon
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    # whose each point is, from the posed skin's weights (not the nearest
    # bone: the chest's side lies nearer the hanging arm than the spine)
    owner = A.carry_values(kit, before, posed.owner, o)
    waste = {b['name']: 0.34 for b in posed.bones if b['name'].split('.')[0] in LIMBS}
    # the waist pinched in under the ribs
    waste.update(spine=0.22)
    buried = [n for n in waste if n.split('.')[0] in ('thigh', 'upperarm')]
    W = A.waste(P, posed, list(waste), waste, joint_keep=0.06, owner=owner, buried=buried)
    # the pelvis's flesh gone to the bone: in toward its middle, level
    hips = owner == posed.ix['hips']
    c = posed.head('hips')
    W[hips, 0] = c[0] + (W[hips, 0] - c[0]) * 0.72
    W[hips, 2] = c[2] + (W[hips, 2] - c[2]) * 0.8
    # no step where one part was wasted more than its neighbour
    P = A.relax_offsets(kit, o, P, W, 25)
    torso, names = A.region(P, posed, TORSO)
    ahead = np.cross([1, 0, 0], posed.tail('upperchest') - posed.head('hips'))
    ahead /= np.linalg.norm(ahead)
    back = -ahead
    spine_line = (posed.head('hips'), posed.tail('upperchest'))
    t, dist, r = A.seg_param(P, *spine_line)
    near_spine = (dist < 0.2) & torso & (np.abs(P[:, 0]) < 0.03)
    knobs = A.bumps_along(P, N, spine_line[0], spine_line[1], 0.028, 0.005, 0.03, back, 0.3)
    P = P + knobs * near_spine[:, None]
    # ribs only on the ribcage; the belly falls away under it, gradually
    ribcage = torso & np.isin(names, ['chest', 'upperchest'])
    P = P + A.ribs(P, N, posed.tail('upperchest'), posed.head('chest'), ahead, 9, 0.0065, ribcage,
                   irregular=0.18, seed=53)
    t_sp, _, _ = A.seg_param(P, posed.head('hips'), posed.head('chest'))
    under = np.clip(1.0 - np.abs(t_sp - 0.55) / 0.45, 0, 1) ** 1.5
    front = np.clip((N @ ahead - 0.1) / 0.6, 0, 1)
    belly = torso * under * front
    P = P - N * (0.026 * belly)[:, None]
    P = P + N * (0.0015 * kit.fbm(P, 45.0, 3, seed=51))[:, None]
    kit.set_verts_np(o, kit.to_bl_np(P))
    kit.smooth(o, 2, 0.35)

    # ---- close (E14): the cords and bones a starved body is hung on
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    H = lambda n: np.array(posed.head(n))
    T = lambda n: np.array(posed.tail(n))
    unit = lambda v: np.asarray(v, float) / (np.linalg.norm(v) + 1e-12)
    upv = np.array([0.0, 1.0, 0.0])
    sternum = T('upperchest') + ahead * 0.035
    for side, sx in (('.L', 1), ('.R', -1)):
        # the neck's cords, from behind each ear down to the breastbone
        ear = H('head') + np.array([sx * 0.045, 0.01, -0.01])
        P = P + A.cord(P, N, ear, sternum + np.array([sx * 0.012, 0, 0]), 0.0055, 0.008,
                       side=np.array([sx * 0.6, 0, 0]) + ahead, side_min=0.0, ends=0.1)
        # the collarbone, out to the shoulder's point
        sh = H('upperarm' + side)
        P = P + A.cord(P, N, sternum + np.array([sx * 0.02, 0, 0]), sh + upv * 0.02, 0.006, 0.0075,
                       side=upv * 0.6 + ahead, side_min=0.0, ends=0.08)
        P = P + A.bump(P, N, sh + upv * 0.025, 0.004, 0.015)
        # the hip's crest
        P = P + A.bump(P, N, H('hips') + np.array([sx * 0.10, 0.06, 0.03]), 0.008, 0.03)
        # the kneecap, the shin's edge; the elbow's point
        hip, knee, ankle = H('thigh' + side), H('shin' + side), T('shin' + side)
        front = unit(knee - (hip + ankle) / 2)
        if front @ ahead < 0:
            front = unit(ahead - unit(ankle - knee) * (ahead @ unit(ankle - knee)))
        P = P + A.bump(P, N, knee + front * 0.035, 0.007, 0.026)
        P = P + A.cord(P, N, knee + front * 0.015, ankle + front * 0.01, 0.003, 0.007, side=front, side_min=0.3)
        P = P + A.folds(P, N, knee, unit(ankle - knee), 0.0014, 0.011, 0.05, seed=181 + sx)
        el, wr = H('forearm' + side), T('forearm' + side)
        point = unit(el - (sh + wr) / 2)
        P = P + A.bump(P, N, el + point * 0.022, 0.006, 0.02)
        P = P + A.folds(P, N, el, unit(wr - el), 0.0015, 0.010, 0.045, seed=191 + sx)
        # the backs of the hands: tendons to knuckles like knots; the wrist's bone
        hand_ax = unit(T('hand' + side) - H('hand' + side))
        lat = H('finger51' + side) - H('finger21' + side)
        back = unit(np.cross(hand_ax, lat) * (1.0 if side == '.L' else -1.0))
        for f in range(2, 6):
            k1, k2, k3 = H(f'finger{f}1{side}'), H(f'finger{f}2{side}'), H(f'finger{f}3{side}')
            wrist = H('hand' + side) + (k1 - H('hand' + side)) * 0.1
            P = P + A.cord(P, N, wrist, k1, 0.0026, 0.0032, side=back, side_min=0.2, ends=0.1)
            P = P + A.bump(P, N, k1 + back * 0.008, 0.004, 0.008)
            P = P + A.bump(P, N, k2 + back * 0.006, 0.003, 0.006)
            P = P + A.bump(P, N, k3 + back * 0.005, 0.002, 0.005)
        P = P + A.cord(P, N, el, wr, 0.0025, 0.006, side=back, side_min=0.3)
        P = P + A.bump(P, N, wr + back * 0.012 + unit(lat) * 0.015, 0.0035, 0.008)
        # the heel's cord and the tops of the feet
        P = P + A.cord(P, N, knee + (ankle - knee) * 0.55 - front * 0.02, ankle - front * 0.03,
                       0.004, 0.008, side=-front, side_min=0.2)
        fa, fb = H('foot' + side), T('foot' + side)
        top = unit(np.cross(unit(fb - fa), [1, 0, 0]))
        for k in (-1, 0, 1):
            P = P + A.cord(P, N, fa, fb + np.array([sx * 0.012 * k, 0, 0]), 0.0018, 0.003, side=top, side_min=0.3)
    # the breastbone
    P = P + A.cord(P, N, sternum, H('chest') + ahead * 0.06, 0.003, 0.009, side=ahead, side_min=0.3)
    # loose skin folded where the stoop folds it: across the belly
    P = P + A.folds(P, N, H('spine') + ahead * 0.06, unit(T('spine') - H('spine')), 0.0022, 0.016, 0.09,
                    seed=211, inner=ahead)
    # ---- the crust: dried black, cracked into plates (two sizes), the
    # plates' edges lifted a little where they part; finer on the face and hands
    d1, d2 = A.worley(P, 0.034, seed=401)
    e1 = d2 - d1
    d1b, d2b = A.worley(P, 0.011, seed=402)
    e2 = d2b - d1b
    crack = np.exp(-(e1 / 0.0022) ** 2)
    fine = np.exp(-(e2 / 0.0011) ** 2) * np.clip(0.3 + kit.fbm(P, 6.0, 2, seed=403) * 1.5, 0, 1)
    curl = np.exp(-(e1 / 0.006) ** 2) - crack
    P = P + N * (-0.0011 * crack - 0.0005 * fine + 0.0004 * curl + 0.00035 * kit.fbm(P, 70.0, 2, seed=404))[:, None]
    crack_k = np.clip(crack + 0.6 * fine, 0, 1)
    kit.set_verts_np(o, kit.to_bl_np(P))
    full = kit.to_game_np(kit.verts_np(o))
    o_far = kit.far_copy(o, 15500)
    kit.decimate(o, 56000)
    gums_far = A.carry_values(kit, full, gums, o_far)
    hole_far = A.carry_values(kit, full, hole, o_far)
    gums = A.carry_values(kit, full, gums, o)
    hole = A.carry_values(kit, full, hole, o)
    crack_far = A.carry_values(kit, full, crack_k, o_far)
    crack_k = A.carry_values(kit, full, crack_k, o)
    nV, nF, nB, nS = A.nails(full, posed, [f'finger{f}3{s}' for s in ('.L', '.R') for f in range(1, 6)],
                             length=0.014, width=0.0095, thick=0.0016, curl=0.35, seed=71, ragged=0.45)

    # ---- tar-black, dry and cracked: never quite black, never one black
    def paint(o, gums, hole, crack_k):
        P = kit.to_game_np(kit.verts_np(o))
        cav = kit.cavity(o)
        Nv = kit.to_game_np(kit.vnormals(o))
        occ = A.occlusion(P, Nv, 0.05)
        # (near-black brown, never black: nothing real is darker than
        # about 0.02 linear; under the lamp a black under that read as a
        # hole cut in the frame, or a suit)
        black = np.array(kit.srgb('#201a16'))
        warm = np.array(kit.srgb('#3a2b21'))
        ash = np.array(kit.srgb('#3c3834'))
        n1 = kit.fbm(P, 7.0, 4, seed=52)
        n2 = kit.fbm(P, 24.0, 3, seed=55)
        col = black + (warm - black) * np.clip(n1 + 0.3, 0, 1)[:, None] * 0.7
        col = col + (ash - col) * np.clip(n2 * 1.4, 0, 1)[:, None] * 0.35
        col = np.clip(col * (1 - 0.55 * cav)[:, None] * (1 - 0.5 * occ)[:, None], 0.01, 0.85)
        # grey-brown dust on what faces up (the shoulders, the crown, the
        # tops of the forearms): it flattens the crust under the lamp
        upf = np.clip((Nv[:, 1] - 0.25) / 0.6, 0, 1)
        col = col + (np.array(kit.srgb('#56504a')) - col) * (upf * 0.32 * np.clip(0.5 + kit.fbm(P, 20.0, 2, seed=58), 0, 1))[:, None]
        # dust dried on the bones where they stand out (the ribs, the spine,
        # the collarbones, the knuckles, the skull): its anatomy reads even out
        # of the light (all black, a blind review read it as a body suit, E10)
        ridge = np.clip((0.3 - cav) / 0.3, 0, 1) ** 1.5
        dust = np.array(kit.srgb('#4f453c'))
        col = col + (dust - col) * (ridge * 0.45 * np.clip(0.6 + kit.fbm(P, 30.0, 2, seed=53), 0, 1))[:, None]
        # the cracks: pale with the dust caught in them, red-brown deep down
        under = np.array(kit.srgb('#4a2016'))
        caught = np.array(kit.srgb('#5d554a'))
        col = col + (caught - col) * (crack_k * 0.4 * np.clip(0.4 + kit.fbm(P, 18.0, 2, seed=56), 0, 1))[:, None]
        col = col + (under - col) * (np.clip(crack_k - 0.5, 0, 1) * 1.2 * np.clip(0.3 + kit.fbm(P, 9.0, 2, seed=57), 0, 1))[:, None]
        # the gums, dried dark red-brown, where the lips are gone; black behind the teeth
        gum = np.array(kit.srgb('#3d1d16'))
        col = col + (gum - col) * np.clip(gums * 1.5, 0, 1)[:, None]
        col = col * (1 - 0.8 * np.clip(hole * 1.5, 0, 1))[:, None]
        return np.clip(col, 0.01, 0.85)
    col = paint(o, gums, hole, crack_k)
    col_far = paint(o_far, gums_far, hole_far, crack_far)

    # ---- teeth and eyes on their bones
    def mesh(name, V, F):
        import bpy
        me = bpy.data.meshes.new(name)
        me.from_pydata([tuple(p) for p in kit.to_bl_np(V)], [], F)
        me.update()
        for p in me.polygons:
            p.use_smooth = True
        return kit.link(bpy.data.objects.new(name, me))
    tu, tl = mesh('teeth_u', tu_v, tu_f), mesh('teeth_l', tl_v, tl_f)
    el, er = mesh('eye_l', eye_l, eye_l_f), mesh('eye_r', eye_r, eye_r_f)
    dark = lambda o: np.tile(np.array(kit.srgb('#1a1712')), (len(o.data.vertices), 1))

    nails = mesh('nails@near', nV, nF)

    def nail_weights(P):
        J = np.zeros((len(P), 4), dtype=np.int64)
        W = np.zeros((len(P), 4))
        J[:, 0] = nB
        W[:, 0] = 1.0
        return J, W
    horn, black_n = np.array(kit.srgb('#3a2e22')), np.array(kit.srgb('#120e0b'))
    nc = horn + (black_n - horn) * np.clip(nS * 1.1 + 0.3 * kit.fbm(nV, 300.0, 1, seed=72), 0, 1)[:, None]

    body = kit.Body('smiler', posed)
    body.skinned('skin', 'skin', o, colors=col)
    body.rigid('teeth.upper', 'teeth', 'head', tu, colors=tu_c)
    body.rigid('teeth.lower', 'teeth', 'jaw', tl, colors=tl_c)
    body.rigid('eye.L', 'eye', 'head', el, colors=dark(el))
    body.rigid('eye.R', 'eye', 'head', er, colors=dark(er))
    body.skinned_by('nails@near', 'nail', nails, nail_weights, colors=np.clip(nc, 0.01, 0.85))
    # the far body: the skin cut to a quarter (drawn from 6 m out)
    body.skinned('skin@far', 'skin', o_far, colors=col_far)
    return body
