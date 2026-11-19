"""
The Hound (remade 2026-11-19 from docs/research/hound-references.md; its
skin and detail again the same day, E14, from
docs/research/creature-horror-references.md): an old person on all fours,
starved, rump high, the head hanging so low you see only the back of the
skull; one leg drags. Blind: the eyelids have grown shut.
(docs/design/ENTITIES.md, design 1.)

MakeHuman's old, thin man (CC0), posed on hands and feet (the pose is the
rest pose the game binds), the forearms a little too long (the one wrong
thing at a glance: the back is level where a person's would slope),
starved further by displacement (wasted limbs with knobby joints, ribs on
the flanks, the spine's knobs and the shoulder blades on the back), in a
vest and long johns gone the colour of the floor, thin hair matted over the
skull and hanging in locks. Close (E14), the forms a doll lacks: kneecaps,
the shin's edge and the elbow's point it kneels and leans on, worn into
sores; tendons over the backs of the hands and feet, knuckles, the heel's
cord; loose skin bunched at the joints; long cracked nails; and the hair
lying flat over the crown before it falls. Near, about 100k triangles;
from 6 m a far body of under 25k (`kit.far_copy`).
"""
import numpy as np

ORDER = 10


def ik(root, target, pole, a, b):
    """two-bone directions from root toward target, bending toward pole"""
    t = target - root
    d = min(np.linalg.norm(t), (a + b) * 0.999)
    u = t / np.linalg.norm(t)
    p = pole - root
    p = p - u * (p @ u)
    p = p / np.linalg.norm(p)
    x = (a * a - b * b + d * d) / (2 * d)
    h = np.sqrt(max(a * a - x * x, 0.0))
    mid = u * x + p * h
    return mid, u * d - mid


def pose_of(human, skel):
    """the hound's pose (2026-11-19, from docs/research/hound-references.md):
    on hands and feet as a person holds it, the knees bent under a high
    rump, the back humped (the spine rising from the pelvis, then falling
    steeply to the shoulders), the head hanging straight down below them;
    the elbows splayed out and up; the fingers clawed into the floor; the
    left foot trailing, on its toes (the drag). The forearms a little too
    long still: the one wrong thing at a glance."""
    L = lambda n: np.linalg.norm(skel.tail(n) - skel.head(n))
    stretch = {'forearm.L': 1.12, 'forearm.R': 1.12, 'hand.L': 1.06, 'hand.R': 1.06}
    aims = {
        'hips': (0, -0.10, 1), 'spine': (0, 0.26, 1), 'chest': (0, 0.04, 1), 'upperchest': (0, -0.62, 1),
        'neck': (0, -0.92, 0.42), 'head': (0, -0.97, 0.26), 'jaw': (0, -0.9, -0.35),
    }
    root = (0, 0.70, -0.32)
    s1 = human.fk(skel, aims, stretch, root=root)
    for side, sx in (('.L', 1), ('.R', -1)):
        sh = s1.head('upperarm' + side)
        hand_t = np.array([sx * 0.21, 0.045, sh[2] + 0.06])
        a, b = L('upperarm' + side), L('forearm' + side) * stretch['forearm' + side]
        # the elbows out and up, as a thing crouched to spring holds them
        aims['upperarm' + side], aims['forearm' + side] = ik(sh, hand_t, sh + np.array([sx * 0.55, 0.25, -0.35]), a, b)
        aims['hand' + side] = (sx * 0.08, -0.16, 1)
        # the fingers clawed into the floor: knuckles up, the tips dug in
        for f in range(2, 6):
            spread = sx * (f - 3) * 0.13
            aims[f'finger{f}1{side}'] = (spread, -0.35, 1)
            aims[f'finger{f}2{side}'] = (spread * 0.6, -0.92, 0.42)
            aims[f'finger{f}3{side}'] = (spread * 0.3, -1, -0.12)
        aims['finger11' + side] = (-sx * 0.55, -0.35, 0.75)
        aims['finger12' + side] = (-sx * 0.35, -0.8, 0.5)
        aims['finger13' + side] = (-sx * 0.2, -1, 0.1)
        hp = s1.head('thigh' + side)
        # the knees bent under the rump, toward the hands (a person's bear
        # crawl); the left foot trails further back, on its toes (the drag)
        back = 0.34 if side == '.L' else 0.22
        foot_t = np.array([sx * 0.14, 0.085 if side == '.R' else 0.11, hp[2] - back])
        a, b = L('thigh' + side), L('shin' + side)
        aims['thigh' + side], aims['shin' + side] = ik(hp, foot_t, hp + np.array([sx * 0.15, -0.35, 1.0]), a, b)
        aims['foot' + side] = (0, -0.72, 0.7) if side == '.R' else (0, -0.9, 0.42)
    return aims, stretch, root


TORSO = ('hips', 'spine', 'chest', 'upperchest')
LIMBS = ('upperarm', 'forearm', 'thigh', 'shin')
FINGERS = [f'finger{f}3{s}' for s in ('.L', '.R') for f in range(1, 6)]


def unit(v):
    v = np.asarray(v, float)
    return v / (np.linalg.norm(v) + 1e-12)


def build(kit):
    import hm08, human, anatomy as A
    base = hm08.Base().target('macro/male-old-thin').target('macro/male-old')
    o, skel = human.body(kit, base, 'hound')
    aims, stretch, root = pose_of(human, skel)
    posed = human.pose(kit, o, skel, aims, stretch, root)
    # stand it on the floor
    P = kit.to_game_np(kit.verts_np(o))
    lift = -P[:, 1].min()
    kit.set_verts_np(o, kit.to_bl_np(P + np.array([0, lift, 0])))
    for b in posed.bones:
        b['head'] = (b['head'][0], b['head'][1] + lift, b['head'][2])
        b['tail'] = (b['tail'][0], b['tail'][1] + lift, b['tail'][2])
    before = kit.to_game_np(kit.verts_np(o))
    kit.remesh(o, 0.0038)
    kit.smooth(o, 2, 0.5)
    # whose each point is, from the posed skin's weights (not the nearest
    # bone: the hanging head lies nearer the chest's front than the chest
    # does, and the vest cut there left only its hem, a ring)
    owner = A.carry_values(kit, before, posed.owner, o)
    H = lambda n: np.array(posed.head(n))
    T = lambda n: np.array(posed.tail(n))

    # ---- starved: wasting, then the bones under the skin
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    limbs = [b['name'] for b in posed.bones if b['name'].split('.')[0] in LIMBS]
    P = A.waste(P, posed, limbs, 0.22, joint_keep=0.07)
    # the shoulders starved to the bone: the deltoid's round gone, the
    # point of the shoulder standing out under the skin
    for side in ('.L', '.R'):
        j = H('upperarm' + side)
        d = np.linalg.norm(P - j, axis=1)
        out = P - j
        out /= np.linalg.norm(out, axis=1)[:, None] + 1e-9
        # (never through the joint's middle: pushed past it, the skin folded)
        P = P - out * (0.024 * np.exp(-(d / 0.095) ** 2) * np.clip(d / 0.05, 0, 1))[:, None]
        tip = j + np.array([0.0, 0.03, 0.0])
        P = P + out * (0.006 * np.exp(-(np.linalg.norm(P - tip, axis=1) / 0.02) ** 2))[:, None]
    torso, names = A.region(P, posed, TORSO)
    up = np.array([0, 1, 0])
    # the spine's knobs along the back (the back faces up on all fours), on
    # up the neck to the knob at its root
    spine_line = (H('hips'), T('upperchest'))
    t, dist, r = A.seg_param(P, *spine_line)
    near_spine = (dist < 0.16) & torso & (np.abs(P[:, 0]) < 0.035)
    knobs = A.bumps_along(P, N, spine_line[0], spine_line[1], 0.024, 0.010, 0.026, up, 0.5)
    P = P + knobs * near_spine[:, None] * np.clip(1 - np.abs(P[:, 0]) / 0.035, 0, 1)[:, None]
    neck_back = unit(np.cross(T('neck') - H('neck'), [1, 0, 0]))
    if neck_back @ up < 0:
        neck_back = -neck_back
    tn, dn, _ = A.seg_param(P, H('neck'), T('neck'))
    nk = A.bumps_along(P, N, H('neck'), T('neck'), 0.02, 0.005, 0.02, neck_back, 0.4)
    P = P + nk * ((dn < 0.08) & (np.abs(P[:, 0]) < 0.025))[:, None]
    P = P + A.bump(P, N, H('neck') + neck_back * 0.035, 0.007, 0.014)      # the knob at the neck's root
    # ribs on the flanks and back of the chest
    ribcage = torso & np.isin(names, ['chest', 'upperchest', 'spine'])
    P = P + A.ribs(P, N, T('upperchest'), H('chest') * 0.6 + H('spine') * 0.4,
                   (0, -1, 0), 10, 0.0075, ribcage, irregular=0.25, seed=3)
    # the shoulder blades
    for sx in (1, -1):
        c = H('upperchest') * 0.5 + T('upperchest') * 0.5 + np.array([sx * 0.085, 0.02, -0.02])
        d = np.linalg.norm(P - c, axis=1)
        ridge = np.exp(-(d / 0.05) ** 2) * np.clip(N @ up, 0, 1)
        # winged: the inner edge lifted most, a hollow under it
        edge = np.exp(-(((P[:, 0] - c[0]) * sx + 0.025) / 0.012) ** 2) * ridge
        P = P + N * (0.004 * ridge + 0.006 * edge)[:, None]
    # the pelvis's bones under the skin: each hip's crest and the sacrum
    crests = []
    for sx in (1, -1):
        crest = H('hips') + np.array([sx * 0.11, 0.03, 0.04])
        crests.append(crest)
        d = np.linalg.norm(P - crest, axis=1)
        P = P + N * (0.010 * np.exp(-(d / 0.035) ** 2) * torso)[:, None]
    sacrum = H('hips') + np.array([0, 0.05, -0.06])
    P = P + A.bump(P, N, sacrum, 0.007, 0.03)
    # the belly sucked in under the ribs, its loose skin hanging below
    belly = torso & np.isin(names, ['spine', 'hips']) & ((N @ up) < -0.2)
    P = P - N * (0.012 * belly)[:, None]
    P = P + A.folds(P, N, H('spine'), (0, 0, 1), 0.0022, 0.022, 0.12, seed=71, inner=(0, -1, 0)) * belly[:, None]

    # ---- its face, seen when it rears up (E14: smooth, it read as a doll's
    # egg): the eyes sunk in their sockets with the lids grown shut over
    # them into slits, the cheeks fallen in, the mouth a dark slack gape,
    # its lips dried thin. MakeHuman's eyes and teeth carried by the head
    # and the jaw from where they were to where they are posed
    def carried(bone, X):
        b0, b1 = np.array(skel.head(bone)), np.array(posed.head(bone))
        R = human._rot_between(np.array(skel.tail(bone)) - b0, np.array(posed.tail(bone)) - b1)
        return b1 + (X - b0) @ R.T, R
    eyes = [carried('head', base.helper(n)[0])[0] for n in ('l-eye', 'r-eye')]
    _, Rh = carried('head', np.zeros((1, 3)))
    f_lat, f_up, f_fwd = Rh @ np.array([1.0, 0, 0]), Rh @ np.array([0, 1.0, 0]), Rh @ np.array([0, 0, 1.0])
    Ec = [e.mean(0) for e in eyes]
    er = max(float(np.linalg.norm(eyes[0] - Ec[0], axis=1).max()), 0.01)
    tu = carried('head', base.helper('upper-teeth')[0])[0]
    tl = carried('jaw', base.helper('lower-teeth')[0])[0]
    mouth = (tu[np.argmax(tu @ f_fwd)] + tl[np.argmax(tl @ f_fwd)]) / 2 + f_fwd * 0.006

    def face_marks(Q):
        """the face's marks at points Q: socket, the shut lids' slit, the
        mouth's gape, its lips, the hollow cheeks (each 0..1)"""
        socket = np.zeros(len(Q))
        slit = np.zeros(len(Q))
        for c in Ec:
            rel = Q - c
            a, b, z = rel @ f_lat, rel @ f_up, rel @ f_fwd
            rho = np.hypot(a, b * 1.15)
            front = np.clip((z + er) / er, 0, 1)
            socket = np.maximum(socket, np.clip(1.15 - rho / (1.9 * er), 0, 1) ** 0.8 * front)
            slit = np.maximum(slit, np.exp(-((b + 0.1 * a * a / er) / 0.0017) ** 2) * np.clip(1 - np.abs(a) / (1.2 * er), 0, 1) * front)
        rel = Q - mouth
        a, b, z = rel @ f_lat, rel @ f_up, rel @ f_fwd
        ell = np.sqrt((a / 0.024) ** 2 + ((b + 0.25 * (a / 0.024) ** 2 * 0.006) / 0.0065) ** 2)
        near = np.clip((z + 0.025) / 0.015, 0, 1)
        gape = np.clip(1.25 - ell, 0, 1) * near
        lips = np.exp(-((ell - 1.15) / 0.3) ** 2) * near
        cheek = np.zeros(len(Q))
        for sx in (1, -1):
            cc = mouth + f_lat * (sx * 0.038) + f_up * 0.022 - f_fwd * 0.012
            cheek = np.maximum(cheek, np.exp(-(np.linalg.norm(Q - cc, axis=1) / 0.016) ** 2))
        return socket, slit, gape, lips, cheek
    sk, sl, gp, lp, ch = face_marks(P)
    P = P - N * (0.010 * sk + 0.0016 * sl + 0.007 * gp - 0.0012 * lp + 0.006 * ch)[:, None]

    # ---- close (E14): the bones it kneels and leans on, its tendons, the
    # loose skin at its joints; and where they pressed the floor, sores
    sores = []
    for side, sx in (('.L', 1), ('.R', -1)):
        hip, knee, ankle = H('thigh' + side), H('shin' + side), T('shin' + side)
        front = unit(knee - (hip + ankle) / 2)
        sd = unit(ankle - knee)
        shin_front = unit(front - sd * (front @ sd))
        P = P + A.bump(P, N, knee + front * 0.03, 0.007, 0.024)                     # the kneecap
        P = P + A.cord(P, N, knee + shin_front * 0.01, ankle, 0.0035, 0.007, side=shin_front, side_min=0.3)
        P = P + A.cord(P, N, knee + (ankle - knee) * 0.55 - shin_front * 0.02, ankle - shin_front * 0.03,
                       0.004, 0.008, side=-shin_front, side_min=0.2)               # the heel's cord
        P = P + A.folds(P, N, knee, sd, 0.0016, 0.011, 0.045, seed=81 + sx, inner=-front)
        sores.append((knee + front * 0.045, 0.024 if side == '.L' else 0.016))
        sh, el, wr = H('upperarm' + side), H('forearm' + side), T('forearm' + side)
        point = unit(el - (sh + wr) / 2)
        P = P + A.bump(P, N, el + point * 0.022, 0.006, 0.018)                     # the elbow's point
        P = P + A.folds(P, N, el, unit(wr - el), 0.0018, 0.010, 0.04, seed=91 + sx)
        sores.append((el + point * 0.03, 0.013))
        # the backs of the hands: a tendon to each knuckle, the knuckles
        hand_ax = unit(T('hand' + side) - H('hand' + side))
        lat = H('finger51' + side) - H('finger21' + side)
        back = unit(np.cross(hand_ax, lat) * (1.0 if side == '.L' else -1.0))
        for f in range(2, 6):
            k1, k2 = H(f'finger{f}1{side}'), H(f'finger{f}2{side}')
            wrist = H('hand' + side) + (k1 - H('hand' + side)) * 0.1
            P = P + A.cord(P, N, wrist, k1, 0.0024, 0.0032, side=back, side_min=0.2, ends=0.1)
            P = P + A.bump(P, N, k1 + back * 0.008, 0.0035, 0.008)
            P = P + A.bump(P, N, k2 + back * 0.006, 0.0022, 0.006)
            sores.append((k1 + back * 0.009, 0.004))
        # the wrist's bone and the forearm's edge
        P = P + A.cord(P, N, el, wr, 0.0022, 0.006, side=back, side_min=0.3)
        P = P + A.bump(P, N, wr + back * 0.012 + unit(lat) * 0.015, 0.003, 0.008)
        # the tops of the feet: their tendons
        fa, fb = H('foot' + side), T('foot' + side)
        fd = unit(fb - fa)
        top = unit(np.cross(fd, [1, 0, 0]))
        for k in (-1, 0, 1):
            off = np.array([sx * 0.012 * k, 0, 0])
            P = P + A.cord(P, N, fa + off * 0.5, fb + off, 0.0018, 0.003, side=top, side_min=0.3)
    for c in crests:
        sores.append((c + np.array([0, 0.012, 0]), 0.016))
    sores.append((sacrum + np.array([0, 0.01, 0]), 0.014))
    sores.append((H('chest') + np.array([0, 0.03, 0]), 0.01))
    wound = np.zeros(len(P))
    for k, (c, r) in enumerate(sores):
        off, w = A.sore(P, N, c, r, 0.0028, seed=200 + k)
        P = P + off
        wound = np.maximum(wound, w)
    wound_P = P.copy()
    # loose old skin over all of it, and a little unevenness
    P = P + N * (0.0010 * kit.fbm(P, 40.0, 3, seed=11) + 0.0006 * kit.fbm(P, 140.0, 2, seed=12))[:, None]
    kit.set_verts_np(o, kit.to_bl_np(P))
    kit.smooth(o, 1, 0.25)
    full_P = kit.to_game_np(kit.verts_np(o))

    # ---- what it wears: a vest and long johns, grown off the skin
    P = full_P
    nb = np.where(owner >= 0, owner, A.nearest_bones(P, posed)[0])
    bn = np.array([posed.bones[i]['name'].split('.')[0] for i in nb])
    along = np.zeros(len(P))
    for i in np.unique(nb):
        m = nb == i
        along[m] = A.seg_param(P[m], posed.bones[i]['head'], posed.bones[i]['tail'])[0]
    vest_keep = np.isin(bn, ['spine', 'chest']) | ((bn == 'upperchest') & (along < 0.78))
    johns_keep = (bn == 'hips') | (bn == 'thigh') | ((bn == 'shin') & (along < 0.86))
    # worn through at the knees from crawling: a ragged hole over each
    for side in ('.L', '.R'):
        knee = H('shin' + side)
        ragged = 0.055 + 0.015 * kit.fbm(P, 60.0, 2, seed=51)
        johns_keep &= ~(np.linalg.norm(P - knee, axis=1) < ragged)
    # (torn small: holed widely, its front went and its hem stayed, a ring)
    vest = A.garment(kit, o, 'vest', vest_keep, 0.005, sag=0.035, tears=0.1, tear_seed=31, tear_freq=16.0)
    johns = A.garment(kit, o, 'johns', johns_keep, 0.007, sag=0.024, tears=0.14, tear_seed=37, tear_freq=8.0)
    # bunched and wrinkled where they sag: folds across each leg, more at
    # the back of the knee and the crotch, broken up so they are cloth's
    for cl, amp, seed in ((johns, 0.0042, 53), (vest, 0.0034, 63)):
        Pj = kit.to_game_np(kit.verts_np(cl))
        Nj = kit.to_game_np(kit.vnormals(cl))
        jb, _ = A.nearest_bones(Pj, posed)
        wr = np.zeros(len(Pj))
        for i in np.unique(jb):
            m = jb == i
            t, _, _ = A.seg_param(Pj[m], posed.bones[i]['head'], posed.bones[i]['tail'])
            Lb = np.linalg.norm(np.array(posed.bones[i]['tail']) - np.array(posed.bones[i]['head']))
            bunch = 0.5 + 0.5 * np.clip(np.abs(t - 0.5) * 2, 0, 1)
            # (folds here and there, not rings: a corrugated tube read as a toy's)
            patch = np.clip(kit.fbm(Pj[m], 7.0, 2, seed=seed + 2) * 2.2 + 0.15, 0, 1)
            wr[m] = np.sin(t * Lb / 0.04 * 2 * np.pi + 4.0 * kit.fbm(Pj[m], 11.0, 2, seed=seed)) * bunch * patch
        # and the cloth's own small creases and its frayed thinning; its
        # edges tucked back against the skin (standing off it, the waist's
        # edge caught the light as a ring)
        fine = 0.0012 * kit.fbm(Pj, 55.0, 2, seed=seed + 5)
        import bmesh
        bm = bmesh.new()
        bm.from_mesh(cl.data)
        tuck = np.zeros(len(Pj))
        for v in bm.verts:
            if v.is_boundary:
                tuck[v.index] = 1.0
                for e in v.link_edges:
                    o2 = e.other_vert(v).index
                    tuck[o2] = max(tuck[o2], 0.5)
        bm.free()
        kit.set_verts_np(cl, kit.to_bl_np(Pj + Nj * (amp * wr * (1 - tuck) + fine - 0.006 * tuck)[:, None]))

    # ---- hair: thin, grey, matted flat over the crown, hanging in locks
    crown = T('head') - H('head')
    crown /= np.linalg.norm(crown)
    chin = T('jaw') - H('head')
    face = chin - crown * (chin @ crown)
    face /= np.linalg.norm(face)
    N = kit.to_game_np(kit.vnormals(o))
    # the scalp by where a point lies round the skull's middle, not by its
    # nearest bone (the back of the skull lies nearer the neck's: bald, it
    # was what you saw of it from in front)
    skull_c = H('head') + crown * 0.09 - face * 0.01

    def scalp_w(Q):
        """0..1, softly: how much of the scalp each point is"""
        r = Q - skull_c
        d = np.linalg.norm(r, axis=1)
        u = r / (d[:, None] + 1e-9)
        return (np.clip((0.135 - d) / 0.015, 0, 1) * np.clip((0.3 - u @ face) / 0.12, 0, 1)
                * np.clip((u @ crown + 0.4) / 0.12, 0, 1))

    def scalp_of(Q):
        return scalp_w(Q) > 0.5
    head_m = np.linalg.norm(P - skull_c, axis=1) < 0.17
    scalp = np.where(scalp_of(P) & ((N @ face) < 0.4))[0]
    rng = np.random.default_rng(5)
    # (the crown's whorl: where the head bone's tail meets the skin)
    hp = P[head_m]
    whorl = hp[np.argmax((hp - H('head')) @ crown)]
    # matted locks, flat on the skull and hanging (tubes this thin read as
    # wire, thicker as rope), and loose hairs among them
    locks = np.sort(rng.choice(scalp, size=min(170, len(scalp)), replace=False))
    lock_len = rng.uniform(0.16, 0.36, len(locks)) * (1 + 0.4 * (rng.random(len(locks)) < 0.2))
    # the front half combed forward over the brow, to hang over the face
    # (the canon's hair over the face; Mama, the Clickers: a face half
    # hidden); the rest away from the whorl over the skull
    brow = (Ec[0] + Ec[1]) / 2 + crown * 0.025
    ahead_of = ((P[locks] - skull_c) @ face) > -0.02
    toward = [face if a else None for a in ahead_of]
    lie_each = np.clip(((P[locks] - brow) @ crown) * 1.35, 0.02, 0.16)
    lock_len = np.where(ahead_of, lock_len + lie_each, lock_len)
    hair_a, free_a = A.scalp_hair(kit, 'hair', P[locks], P[head_m], N[head_m], lock_len, 0.0, whorl,
                                  lie=(0.04, 0.11), step=0.022, lift=0.0028, droop=1.2, seed=6, clumps=40,
                                  clump=0.35, width=0.010, toward=toward, lie_each=lie_each)
    loose = np.sort(rng.choice(scalp, size=min(220, len(scalp)), replace=False))
    loose_len = rng.uniform(0.12, 0.34, len(loose))
    hair_b, free_b = A.scalp_hair(kit, 'hair_loose', P[loose], P[head_m], N[head_m], loose_len, 0.0012, whorl,
                                  lie=(0.03, 0.09), step=0.026, lift=0.004, droop=1.0, seed=7, clumps=40, clump=0.5)
    hair = kit.join([hair_a, hair_b], 'hair')
    hair_free = np.concatenate([free_a, free_b])
    far_locks = locks[::2]
    hair_far, hair_far_free = A.scalp_hair(kit, 'hair@far', P[far_locks], P[head_m], N[head_m], lock_len[::2], 0.0,
                                           whorl, lie=(0.04, 0.11), step=0.04, lift=0.003, droop=1.2, seed=6,
                                           clumps=40, clump=0.35, width=0.018, toward=toward[::2],
                                           lie_each=lie_each[::2])

    # ---- nails: long, thick, cracked, grown on past the fingertips
    nV, nF, nB, nS = A.nails(full_P, posed, FINGERS, length=0.011, width=0.0105, thick=0.0018, curl=0.7, seed=17)
    import bpy
    me = bpy.data.meshes.new('nails@near')
    me.from_pydata([tuple(p) for p in kit.to_bl_np(nV)], [], nF)
    me.update()
    nails = kit.link(bpy.data.objects.new('nails@near', me))

    def clean(o):
        """skin pushed hard in places (the starved shoulders) can fold onto
        itself: points merged where they met, faces with no area dissolved,
        so no normal is left without a direction"""
        import bmesh
        bm = bmesh.new()
        bm.from_mesh(o.data)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.0004)
        bmesh.ops.dissolve_degenerate(bm, edges=bm.edges, dist=0.0004)
        bm.normal_update()
        bm.to_mesh(o.data)
        bm.free()

    clean(o)
    o_far = kit.far_copy(o, 14000)
    kit.decimate(o, 56000)
    for x in (o, o_far):
        clean(x)
    vest_far, johns_far = kit.far_copy(vest, 1700), kit.far_copy(johns, 2900)
    kit.decimate(vest, 6000)
    kit.decimate(johns, 10000)

    # ---- paint
    def paint_skin(o):
        P = kit.to_game_np(kit.verts_np(o))
        cav = kit.cavity(o)
        Nv = kit.to_game_np(kit.vnormals(o))
        occ = A.occlusion(P, Nv, 0.05)
        own = np.array([posed.bones[i]['name'].split('.')[0] for i in A.nearest_bones(P, posed)[0]])
        sore_k = A.carry_values(kit, wound_P, wound, o)
        base_col = np.array(kit.srgb('#665b4a'))            # old skin, sallow, unwashed
        sick = np.array(kit.srgb('#555743'))
        grey = np.array(kit.srgb('#5f5a53'))
        bruise = np.array(kit.srgb('#4f3d4c'))
        scab = np.array(kit.srgb('#3a1d17'))
        raw = np.array(kit.srgb('#5a2a22'))
        slough = np.array(kit.srgb('#76693f'))
        spot = np.array(kit.srgb('#5e452c'))                # liver spots
        vein = np.array(kit.srgb('#4f5866'))
        dirt = np.array(kit.srgb('#30291f'))
        n1 = kit.fbm(P, 6.0, 4, seed=21)
        n2 = kit.fbm(P, 19.0, 3, seed=29)
        col = base_col + (sick - base_col) * np.clip(0.5 + n1, 0, 1)[:, None] * 0.7
        col = col + (grey - col) * np.clip(0.5 + n2, 0, 1)[:, None] * 0.35
        # never one hue: yellowed in patches, purpled in others (purpura)
        yel = np.array(kit.srgb('#8f7d4f'))
        col = col + (yel - col) * np.clip(kit.fbm(P, 4.0, 2, seed=31) * 2.0, 0, 1)[:, None] * 0.35
        pur = np.array(kit.srgb('#5a4446'))
        pp = np.clip((kit.fbm(P, 14.0, 2, seed=32) - 0.25) * 4.0, 0, 1)
        pp = pp * np.where(P[:, 0] > 0, 1.0, 0.35)              # (one side worse than the other)
        col = col + (pur - col) * (pp * np.isin(own, ['forearm', 'upperarm', 'shin', 'thigh', 'hand']))[:, None] * 0.3
        # blotched: no two hand-widths of it the same
        col = col * (0.8 + 0.3 * np.clip(0.5 + kit.fbm(P, 45.0, 2, seed=30), 0, 1))[:, None]
        # liver spots on the back, the hands and the scalp
        sp = np.clip((kit.fbm(P, 28.0, 2, seed=24) - 0.4) * 6.0, 0, 1)
        where = np.isin(own, ['spine', 'chest', 'upperchest', 'hand', 'forearm', 'head', 'neck', 'finger1', 'finger2'])
        col = col + (spot - col) * (sp * where)[:, None] * 0.75
        # veins in the forearms, the backs of the hands and feet, the calves
        ridge = 1 - np.abs(kit.fbm(P * np.array([1.0, 1.0, 0.35]), 18.0, 2, seed=26))
        vn = np.clip((ridge - 0.88) * 9.0, 0, 1) * np.isin(own, ['forearm', 'hand', 'upperarm', 'foot', 'shin'])
        col = col + (vein - col) * vn[:, None] * 0.45
        br = np.clip((kit.fbm(P, 3.0, 3, seed=22) - 0.3) * 3.0, 0, 1)
        col = col + (bruise - col) * br[:, None] * 0.55
        # the knees and the knuckles worn raw: scabbed, bruised round them
        for side in ('.L', '.R'):
            for joint, r in (('shin' + side, 0.06), ('forearm' + side, 0.04), ('finger21' + side, 0.03),
                             ('finger31' + side, 0.03), ('finger41' + side, 0.03), ('finger51' + side, 0.025)):
                d = np.linalg.norm(P - H(joint), axis=1)
                k = np.exp(-(d / r) ** 2)
                col = col + (bruise - col) * k[:, None] * 0.55
                rough = np.clip(k * (0.5 + kit.fbm(P, 70.0, 2, seed=27)), 0, 1)
                col = col + (scab - col) * rough[:, None] * 0.7
        # the sores: raw red-brown, crusted black at their middles, yellowed at the edges
        mid = np.clip((sore_k - 0.55) * 2.5, 0, 1)
        edge = np.clip(sore_k * 2.0, 0, 1) * (1 - mid)
        col = col + (slough - col) * (edge * 0.55)[:, None]
        col = col + (raw - col) * (np.clip(sore_k * 1.4, 0, 1) * 0.8)[:, None]
        col = col + (scab - col) * (mid * np.clip(0.4 + kit.fbm(P, 120.0, 1, seed=28), 0, 1))[:, None]
        # the face: the sockets bruised dark, the slits of the shut lids and
        # the mouth's gape black, the lips dried brown
        sk, sl, gp, lp, _ = face_marks(P)
        col = col + (np.array(kit.srgb('#3b2b26')) - col) * (sk * 0.65)[:, None]
        col = col + (np.array(kit.srgb('#4a3430')) - col) * (lp * 0.6)[:, None]
        col = col + (np.array(kit.srgb('#140e0c')) - col) * np.clip(sl * 1.2 + gp * 1.3, 0, 1)[:, None]
        # the scalp dark under its hair, where the hair grew thin
        col = col + (np.array(kit.srgb('#2e2924')) - col) * scalp_w(P)[:, None] * 0.85
        # filth: where it touches the floor (the hands to the wrists, the
        # knees, the feet), grained into its creases everywhere
        low = np.clip(1 - P[:, 1] / 0.22, 0, 1) ** 1.3
        hands = np.isin(own, ['hand', 'finger1', 'finger2', 'finger3', 'finger4', 'finger5', 'foot', 'toes'])
        # darkest on what faces the floor (the palms, the shins, the belly)
        facing = np.clip(-Nv[:, 1], 0, 1) * np.clip(1 - P[:, 1] / 0.6, 0, 1)
        grime = np.clip(low * (0.65 + 0.35 * np.clip(n1 + 0.5, 0, 1)) + 0.45 * hands + 0.35 * facing, 0, 0.95)
        col = col + (dirt - col) * grime[:, None]
        return np.clip(col * (1 - 0.75 * cav)[:, None] * (1 - 0.45 * occ)[:, None], 0.02, 0.85)

    def paint_cloth(o, base, stain, seed):
        P = kit.to_game_np(kit.verts_np(o))
        cav = kit.cavity(o)
        occ = A.occlusion(P, kit.to_game_np(kit.vnormals(o)), 0.05)
        b, st, dirt = np.array(kit.srgb(base)), np.array(kit.srgb(stain)), np.array(kit.srgb('#352f27'))
        sweat = np.array(kit.srgb('#8a7442'))
        blood = np.array(kit.srgb('#4a231b'))
        n1 = kit.fbm(P, 9.0, 4, seed=seed)
        col = b + (st - b) * np.clip(n1 * 1.5 + 0.2, 0, 1)[:, None]
        # tide marks of old sweat, and blood seeped from the sores beneath
        tide = np.clip((kit.fbm(P, 5.0, 3, seed=seed + 3) - 0.15) * 4.0, 0, 1)
        col = col + (sweat - col) * (tide * 0.45)[:, None]
        near_sore = np.zeros(len(P))
        for c, r in sores:
            near_sore = np.maximum(near_sore, np.exp(-(np.linalg.norm(P - c, axis=1) / (r * 2.5)) ** 2))
        col = col + (blood - col) * (near_sore * np.clip(0.6 + kit.fbm(P, 40.0, 2, seed=seed + 4), 0, 1) * 0.8)[:, None]
        low = np.clip(1 - P[:, 1] / 0.35, 0, 1) ** 1.3
        col = col + (dirt - col) * (low * 0.85)[:, None]
        # sweat and grime soaked in round the waist and the crotch (clean
        # there, the long johns showed as a pale ring under the vest)
        waist = np.exp(-(np.linalg.norm(P - H('hips'), axis=1) / 0.14) ** 2)
        col = col + (np.array(kit.srgb('#3f3627')) - col) * (waist * 0.7 * np.clip(0.6 + kit.fbm(P, 12.0, 2, seed=seed + 9), 0, 1))[:, None]
        # its torn and worn edges frayed dark (a clean edge read as a plastic ring)
        import bmesh
        bm = bmesh.new()
        bm.from_mesh(o.data)
        edge = np.zeros(len(P))
        for v in bm.verts:
            if v.is_boundary:
                edge[v.index] = 1.0
                for e in v.link_edges:
                    edge[e.other_vert(v).index] = max(edge[e.other_vert(v).index], 0.6)
        bm.free()
        col = col + (dirt - col) * (edge * 0.7 * np.clip(0.5 + kit.fbm(P, 50.0, 2, seed=seed + 8), 0.3, 1))[:, None]
        col = col * (0.85 + 0.25 * np.clip(0.5 + kit.fbm(P, 60.0, 2, seed=seed + 6), 0, 1))[:, None]
        return np.clip(col * (1 - 0.6 * cav)[:, None] * (1 - 0.35 * occ)[:, None], 0.02, 0.85)

    def paint_hair(o, free):
        Ph = kit.to_game_np(kit.verts_np(o))
        # near-black, greasy, grey threads through it (the canon's hair over
        # the face: grey, it read as a wig; pale, as paper)
        dark, grey, filth = np.array(kit.srgb('#1a1815')), np.array(kit.srgb('#3d3a35')), np.array(kit.srgb('#3a3326'))
        k = np.clip(0.5 + kit.fbm(Ph, 9.0, 2, seed=8), 0, 1)
        col = dark + (grey - dark) * (k * 0.6)[:, None]
        # strands within a lock: a thread of grey here and there
        thread = np.clip((kit.fbm(Ph, 320.0, 1, seed=11) - 0.3) * 3.0, 0, 1)
        col = col + (np.array(kit.srgb('#6a665e')) - col) * (thread * 0.45)[:, None]
        ends = np.clip(free / 0.25, 0, 1)
        col = col + (filth - col) * (ends * 0.4 * np.clip(0.5 + kit.fbm(Ph, 20.0, 2, seed=9), 0, 1))[:, None]
        return np.clip(col * (0.8 + 0.3 * np.clip(0.5 + kit.fbm(Ph, 40.0, 2, seed=10), 0, 1))[:, None], 0.02, 0.85)

    body = kit.Body('hound', posed)
    body.skinned('skin', 'skin', o, colors=paint_skin(o))
    body.skinned('vest', 'cloth', vest, colors=paint_cloth(vest, '#837a60', '#62563a', 41))
    body.skinned('johns', 'cloth', johns, colors=paint_cloth(johns, '#6e644e', '#4c4230', 43))
    # the far body: the same, cut to a quarter (drawn from 6 m out; weighted
    # before the hair's bones are added, which the skin must not follow)
    body.skinned('skin@far', 'skin', o_far, colors=paint_skin(o_far))
    body.skinned('vest@far', 'cloth', vest_far, colors=paint_cloth(vest_far, '#837a60', '#62563a', 41))
    body.skinned('johns@far', 'cloth', johns_far, colors=paint_cloth(johns_far, '#6e644e', '#4c4230', 43))
    # the hair hangs from three bones the game keeps pointing at the floor:
    # what lies on the scalp rides the head; the rest, each point by how far
    # below where it left the scalp it hangs
    Ph = kit.to_game_np(kit.verts_np(hair))
    crown_pt = T('head') * 0.6 + H('head') * 0.4
    top = Ph[:, 1].max()
    bot = Ph[:, 1].min()
    L = (top - bot) / 3.0
    par = 'head'
    for k in range(3):
        h = (crown_pt[0], top - L * k, crown_pt[2])
        tl = (crown_pt[0], top - L * (k + 1), crown_pt[2])
        posed.add(f'hair.{k}', par, h, tl)
        posed.bones[posed.ix[f'hair.{k}']]['deform'] = True
        par = f'hair.{k}'

    def hair_weights(free):
        def w(P):
            assert len(P) == len(free)
            s = np.clip(free / (top - bot + 1e-9), 0, 1) * 3.0     # 0 on the scalp .. 3 at the longest tips
            J = np.zeros((len(P), 4), dtype=np.int64)
            W = np.zeros((len(P), 4))
            head_i = posed.ix['head']
            for v, x in enumerate(s):
                if x < 0.35:
                    J[v, :2] = [head_i, posed.ix['hair.0']]
                    W[v, :2] = [1 - x / 0.35, x / 0.35]
                else:
                    k = min(int(x), 2)
                    f = x - k
                    a = posed.ix[f'hair.{k}']
                    b = posed.ix[f'hair.{min(k + 1, 2)}']
                    J[v, :2] = [a, b]
                    W[v, :2] = [1 - f, f] if a != b else [1, 0]
            return J, W
        return w
    body.skinned_by('hair', 'hair', hair, hair_weights(hair_free), colors=paint_hair(hair, hair_free))

    # the nails, each on its finger's last bone: horn-yellow, browner and
    # cracked toward the free edge, black with dirt under it
    def nail_weights(P):
        J = np.zeros((len(P), 4), dtype=np.int64)
        W = np.zeros((len(P), 4))
        J[:, 0] = nB
        W[:, 0] = 1.0
        return J, W
    horn, brown, under = np.array(kit.srgb('#7a705c')), np.array(kit.srgb('#4b3b27')), np.array(kit.srgb('#1f1912'))
    nc = horn + (brown - horn) * np.clip(nS * 1.2 + 0.3 * kit.fbm(nV, 300.0, 1, seed=19), 0, 1)[:, None]
    nc = nc + (under - nc) * np.clip((nS - 0.75) * 4.0, 0, 1)[:, None] * 0.7
    body.skinned_by('nails@near', 'nail', nails, nail_weights, colors=np.clip(nc, 0.02, 0.85))

    body.skinned_by('hair@far', 'hair', hair_far, hair_weights(hair_far_free), colors=paint_hair(hair_far, hair_far_free))
    return body
