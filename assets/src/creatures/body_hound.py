"""
The Hound: an old person on all fours, starved, rump high, the head hanging
so low you see only the back of the skull; one leg drags. Blind: the eyelids
have grown shut. (docs/design/ENTITIES.md, design 1.)

MakeHuman's old, thin man (CC0), posed on hands and feet (the pose is the
rest pose the game binds), the forearms a little too long (the one wrong
thing at a glance: the back is level where a person's would slope),
starved further by displacement (wasted limbs with knobby joints, ribs on
the flanks, the spine's knobs and the shoulder blades on the back), in a
vest and long johns gone the colour of the floor, thin hair matted over the
skull.
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
    """the hound's pose: spine and head aimed, then each limb to the floor"""
    L = lambda n: np.linalg.norm(skel.tail(n) - skel.head(n))
    stretch = {'forearm.L': 1.22, 'forearm.R': 1.22, 'hand.L': 1.1, 'hand.R': 1.1}
    aims = {
        'hips': (0, -0.05, 1), 'spine': (0, 0.12, 1), 'chest': (0, -0.18, 1), 'upperchest': (0, -0.45, 1),
        'neck': (0, -0.8, 0.6), 'head': (0, -0.55, 0.85), 'jaw': (0, -0.95, -0.2),
    }
    root = (0, 0.80, -0.30)
    s1 = human.fk(skel, aims, stretch, root=root)
    for side, sx in (('.L', 1), ('.R', -1)):
        sh = s1.head('upperarm' + side)
        hand_t = np.array([sx * 0.20, 0.035, sh[2] + 0.10])
        a, b = L('upperarm' + side), L('forearm' + side) * stretch['forearm' + side]
        aims['upperarm' + side], aims['forearm' + side] = ik(sh, hand_t, sh + np.array([sx * 0.4, 0.0, -0.6]), a, b)
        aims['hand' + side] = (0, -0.12, 1)
        hp = s1.head('thigh' + side)
        # the left foot trails further back, on its toes (the drag)
        back = 0.34 if side == '.L' else 0.22
        foot_t = np.array([sx * 0.15, 0.075 if side == '.R' else 0.10, hp[2] - back])
        a, b = L('thigh' + side), L('shin' + side)
        aims['thigh' + side], aims['shin' + side] = ik(hp, foot_t, hp + np.array([0, 0.2, 1.0]), a, b)
        aims['foot' + side] = (0, -0.55, 0.85) if side == '.R' else (0, -0.9, 0.42)
    return aims, stretch, root


TORSO = ('hips', 'spine', 'chest', 'upperchest')
LIMBS = ('upperarm', 'forearm', 'thigh', 'shin')


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
    kit.remesh(o, 0.005)
    kit.smooth(o, 2, 0.5)

    # ---- starved: wasting, then the bones under the skin
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    limbs = [b['name'] for b in posed.bones if b['name'].split('.')[0] in LIMBS]
    P = A.waste(P, posed, limbs, 0.22, joint_keep=0.07)
    torso, names = A.region(P, posed, TORSO)
    up = np.array([0, 1, 0])
    # the spine's knobs along the back (the back faces up on all fours)
    spine_line = (posed.head('hips'), posed.tail('upperchest'))
    t, dist, r = A.seg_param(P, *spine_line)
    near_spine = (dist < 0.16) & torso & (np.abs(P[:, 0]) < 0.035)
    knobs = A.bumps_along(P, N, spine_line[0], spine_line[1], 0.026, 0.006, 0.03, up, 0.5)
    P = P + knobs * near_spine[:, None] * np.clip(1 - np.abs(P[:, 0]) / 0.035, 0, 1)[:, None]
    # ribs on the flanks and back of the chest
    ribcage = torso & np.isin(names, ['chest', 'upperchest', 'spine'])
    P = P + A.ribs(P, N, posed.tail('upperchest'), posed.head('chest') * 0.6 + posed.head('spine') * 0.4,
                   (0, -1, 0), 9, 0.0045, ribcage)
    # the shoulder blades
    for sx in (1, -1):
        c = posed.head('upperchest') * 0.5 + posed.tail('upperchest') * 0.5 + np.array([sx * 0.085, 0.02, -0.02])
        d = np.linalg.norm(P - c, axis=1)
        ridge = np.exp(-(d / 0.045) ** 2) * np.clip(N @ up, 0, 1)
        P = P + N * (0.008 * ridge)[:, None]
    # the belly sucked in under the ribs
    belly = torso & np.isin(names, ['spine', 'hips']) & ((N @ up) < -0.2)
    P = P - N * (0.012 * belly)[:, None]
    # loose skin and a little unevenness
    P = P + N * (0.0012 * kit.fbm(P, 40.0, 3, seed=11))[:, None]
    kit.set_verts_np(o, kit.to_bl_np(P))
    kit.smooth(o, 1, 0.3)

    # ---- what it wears: a vest and long johns, grown off the skin
    P = kit.to_game_np(kit.verts_np(o))
    nb, _ = A.nearest_bones(P, posed)
    bn = np.array([posed.bones[i]['name'].split('.')[0] for i in nb])
    along = np.zeros(len(P))
    for i in np.unique(nb):
        m = nb == i
        along[m] = A.seg_param(P[m], posed.bones[i]['head'], posed.bones[i]['tail'])[0]
    vest_keep = np.isin(bn, ['spine', 'chest']) | ((bn == 'upperchest') & (along < 0.78))
    johns_keep = (bn == 'hips') | (bn == 'thigh') | ((bn == 'shin') & (along < 0.86))
    vest = A.garment(kit, o, 'vest', vest_keep, 0.005, sag=0.022, tears=0.16, tear_seed=31, tear_freq=11.0)
    johns = A.garment(kit, o, 'johns', johns_keep, 0.006, sag=0.014, tears=0.10, tear_seed=37, tear_freq=8.0)

    # ---- hair: thin, matted, hanging from the crown and the back of the skull
    crown = posed.tail('head') - posed.head('head')
    crown /= np.linalg.norm(crown)
    chin = posed.tail('jaw') - posed.head('head')
    face = chin - crown * (chin @ crown)
    face /= np.linalg.norm(face)
    N = kit.to_game_np(kit.vnormals(o))
    scalp = np.where((bn == 'head') & ((N @ crown) > 0.05) & ((N @ face) < 0.25))[0]
    rng = np.random.default_rng(5)
    pick = rng.choice(scalp, size=min(95, len(scalp)), replace=False)
    pick.sort()
    # wet and matted, it falls: each strand leaves the scalp already bending down
    down = np.array([0.0, -1.0, 0.0])
    start = N[pick] * 0.35 + down * 0.65
    start /= np.linalg.norm(start, axis=1)[:, None]
    hair = A.strands(kit, 'hair', P[pick] - N[pick] * 0.002, start, rng.uniform(0.10, 0.24, len(pick)), 0.004,
                     segments=5, sides=3, droop=1.6, seed=6)

    kit.decimate(o, 11000)
    kit.decimate(vest, 1800)
    kit.decimate(johns, 2600)

    # ---- paint
    def paint_skin(o):
        P = kit.to_game_np(kit.verts_np(o))
        cav = kit.cavity(o)
        base_col = np.array(kit.srgb('#958a7a'))            # old skin, grey-yellow, unwashed
        sick = np.array(kit.srgb('#7f7f6c'))
        bruise = np.array(kit.srgb('#6b5a6e'))
        dirt = np.array(kit.srgb('#4a4238'))
        n1 = kit.fbm(P, 6.0, 4, seed=21)
        col = base_col + (sick - base_col) * np.clip(0.5 + n1, 0, 1)[:, None] * 0.6
        br = np.clip((kit.fbm(P, 3.0, 3, seed=22) - 0.35) * 3.0, 0, 1)
        col = col + (bruise - col) * br[:, None] * 0.55
        low = np.clip(1 - P[:, 1] / 0.25, 0, 1) ** 1.5      # dirt where it touches the floor
        col = col + (dirt - col) * (low * (0.6 + 0.4 * np.clip(n1 + 0.5, 0, 1)))[:, None]
        return np.clip(col * (1 - 0.55 * cav)[:, None], 0.02, 0.85)

    def paint_cloth(o, base, stain, seed):
        P = kit.to_game_np(kit.verts_np(o))
        cav = kit.cavity(o)
        b, st, dirt = np.array(kit.srgb(base)), np.array(kit.srgb(stain)), np.array(kit.srgb('#3a342c'))
        n1 = kit.fbm(P, 9.0, 4, seed=seed)
        col = b + (st - b) * np.clip(n1 * 1.5 + 0.2, 0, 1)[:, None]
        low = np.clip(1 - P[:, 1] / 0.35, 0, 1) ** 1.3
        col = col + (dirt - col) * (low * 0.85)[:, None]
        return np.clip(col * (1 - 0.45 * cav)[:, None], 0.02, 0.85)

    body = kit.Body('hound', posed)
    body.skinned('skin', 'skin', o, colors=paint_skin(o))
    body.skinned('vest', 'cloth', vest, colors=paint_cloth(vest, '#a39c8a', '#857852', 41))
    body.skinned('johns', 'cloth', johns, colors=paint_cloth(johns, '#77756c', '#504b40', 43))
    Ph = kit.to_game_np(kit.verts_np(hair))
    hair_col = np.tile(np.array(kit.srgb('#5e5a52')), (len(Ph), 1)) * (0.8 + 0.2 * kit.fbm(Ph, 30.0, 2, seed=8))[:, None]
    # the hair hangs from three bones the game keeps pointing at the floor:
    # each point weighted by how far below the scalp it hangs
    crown_pt = posed.tail('head') * 0.6 + posed.head('head') * 0.4
    top = Ph[:, 1].max()
    bot = Ph[:, 1].min()
    L = (top - bot) / 3.0
    names = []
    par = 'head'
    for k in range(3):
        h = (crown_pt[0], top - L * k, crown_pt[2])
        tl = (crown_pt[0], top - L * (k + 1), crown_pt[2])
        posed.add(f'hair.{k}', par, h, tl)
        posed.bones[posed.ix[f'hair.{k}']]['deform'] = True
        names.append(f'hair.{k}')
        par = f'hair.{k}'

    def hair_weights(P):
        s = np.clip((top - P[:, 1]) / (top - bot + 1e-9), 0, 1) * 3.0     # 0 at the scalp .. 3 at the tips
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
    body.skinned_by('hair', 'hair', hair, hair_weights, colors=hair_col)
    return body
