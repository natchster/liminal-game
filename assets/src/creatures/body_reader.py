"""
The Reader: a thin man in a brown cardigan buttoned wrong, paper stuffed in
its pockets and down its front, ink on its fingers, the fingers too long; it
walks bent over a sheet held close to its face, its eyes going left to
right. It looks up when it sees you. (docs/ENTITIES.md, design 1.)

MakeHuman's thin man (CC0), halfway between its young and its old, stooped
over the sheet it holds in both hands before its face, its fingers half as
long again as a man's; a cardigan, trousers and shoes grown off the skin
(`anatomy.garment`), the cardigan's left front hanging a button lower than
its right; sheets of paper (rigid) in its hands, in its pockets and down its
front. Its eyes are on bones of their own, so the game can make them read.
"""
import numpy as np

ORDER = 50
FINGERS = 1.5            # the fingers' length, times a man's
THUMB = 1.25


def pose_of(skel):
    """stooped over a sheet held in both hands before its face, a little
    below it, the head bowed to read; the knees soft"""
    a = {'spine': (0, 1, 0.10), 'chest': (0, 1, 0.24), 'upperchest': (0, 1, 0.34), 'neck': (0, 1, 0.62), 'head': (0, 0.78, 0.62)}
    for side, sx in (('.L', 1), ('.R', -1)):
        a['upperarm' + side] = (sx * 0.16, -0.82, 0.42)
        a['forearm' + side] = (-sx * 0.30, 0.48, 0.82)
        a['hand' + side] = (-sx * 0.45, 0.85, 0.08)
        a['thigh' + side] = (sx * 0.04, -1, 0.06)
        a['shin' + side] = (0, -1, -0.05)
    return a


def stretch_of():
    s = {}
    for side in ('.L', '.R'):
        for f in range(1, 6):
            for k in range(1, 4):
                s[f'finger{f}{k}{side}'] = THUMB if f == 1 else FINGERS
    return s


def smooth01(x):
    x = np.clip(x, 0.0, 1.0)
    return x * x * (3 - 2 * x)


def sheet(centre, right, up, w, h, nx, ny, bow=0.0, lean=0.0):
    """a sheet of paper w x h m round `centre` across `right` and along `up`,
    bowed by `bow` m at its sides toward its normal (right x up), its top
    leaning `lean` m along the normal; quads, both sides drawn"""
    right, up = np.asarray(right, float), np.asarray(up, float)
    n = np.cross(right, up)
    n /= np.linalg.norm(n)
    xs, ys = np.linspace(-w / 2, w / 2, nx), np.linspace(-h / 2, h / 2, ny)
    V = [centre + right * x + up * y + n * (bow * (2 * x / w) ** 2 + lean * (y / h + 0.5)) for y in ys for x in xs]
    F = [(j * nx + i, j * nx + i + 1, (j + 1) * nx + i + 1, (j + 1) * nx + i) for j in range(ny - 1) for i in range(nx - 1)]
    uv = np.array([(x / w + 0.5, y / h + 0.5) for y in ys for x in xs])
    return np.array(V), F, uv


def build(kit):
    import bpy
    import hm08, human, anatomy as A
    base = hm08.Base().target('macro/male-young-thin', 0.5).target('macro/male-old-thin', 0.5)
    o, skel = human.body(kit, base, 'reader')
    eye_l, eye_l_f = base.helper('l-eye')
    eye_r, eye_r_f = base.helper('r-eye')
    teeth_u, teeth_u_f = base.helper('upper-teeth')
    teeth_l, teeth_l_f = base.helper('lower-teeth')
    cl0, cr0 = eye_l.mean(0), eye_r.mean(0)
    posed = human.pose(kit, o, skel, pose_of(skel), stretch_of(), None, smooth_weights=6)

    def frame(bone):
        b0, b1 = np.array(skel.head(bone)), np.array(posed.head(bone))
        R = human._rot_between(np.array(skel.tail(bone)) - b0, np.array(posed.tail(bone)) - b1)
        return lambda X: b1 + (X - b0) @ R.T, R
    to_head, Rh = frame('head')
    eye_l, eye_r, teeth_u = to_head(eye_l), to_head(eye_r), to_head(teeth_u)
    cl, cr = to_head(cl0[None])[0], to_head(cr0[None])[0]
    teeth_l = frame('jaw')[0](teeth_l)
    # on the floor
    Pw = kit.to_game_np(kit.verts_np(o))
    lift = -Pw[:, 1].min()
    kit.set_verts_np(o, kit.to_bl_np(Pw + np.array([0, lift, 0])))
    for b in posed.bones:
        b['head'] = (b['head'][0], b['head'][1] + lift, b['head'][2])
        b['tail'] = (b['tail'][0], b['tail'][1] + lift, b['tail'][2])
    up = np.array([0, lift, 0])
    eye_l, eye_r, teeth_u, teeth_l, cl, cr = eye_l + up, eye_r + up, teeth_u + up, teeth_l + up, cl + up, cr + up
    face_fwd = Rh @ np.array([0.0, 0.0, 1.0])
    before = kit.to_game_np(kit.verts_np(o))
    kit.remesh(o, 0.005)
    kit.smooth(o, 2, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)
    # thin: a little off the limbs
    P = kit.to_game_np(kit.verts_np(o))
    waste = {b['name']: 0.10 for b in posed.bones if b['name'].split('.')[0] in ('upperarm', 'forearm', 'thigh', 'shin')}
    W = A.waste(P, posed, list(waste), waste, joint_keep=0.06, owner=owner,
                buried=[n for n in waste if n.split('.')[0] in ('thigh', 'upperarm')])
    P = A.relax_offsets(kit, o, P, W, 15)
    kit.set_verts_np(o, kit.to_bl_np(P))

    # ---- what it wears, grown off the skin
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    nb, _ = A.nearest_bones(P, posed)
    bn = np.array([posed.bones[i]['name'].split('.')[0] for i in nb])
    along = np.zeros(len(P))
    for i in np.unique(nb):
        m = nb == i
        along[m] = A.seg_param(P[m], posed.bones[i]['head'], posed.bones[i]['tail'])[0]
    hips = posed.head('hips')
    neck = posed.head('neck')
    # the body's forward (the chest's front), level
    fwd = np.array([0.0, 0.0, 1.0])
    rel = P - np.array([0.0, 0.0, hips[2]])
    front = (N @ fwd > 0.25) & (rel[:, 2] > 0.0)
    hem = hips[1] - 0.035
    # the cardigan: the trunk to its hem, the arms to the wrists, open in a
    # V at the front from the neck to the breastbone
    v_low = neck[1] - 0.19
    v_open = front & (P[:, 1] > v_low) & (np.abs(P[:, 0]) < 0.075 * smooth01((P[:, 1] - v_low) / (neck[1] - v_low)) + 0.004)
    trunk = np.isin(bn, ['hips', 'spine', 'chest', 'upperchest', 'clavicle']) | ((bn == 'thigh') & (P[:, 1] > hem))
    cardi_keep = ((trunk & (P[:, 1] > hem)) | np.isin(bn, ['upperarm']) | ((bn == 'forearm') & (along < 0.9))) \
        & ~v_open & ~((bn == 'neck') | (bn == 'head'))
    cardi_keep &= ~((bn == 'upperchest') & (P[:, 1] > neck[1] - 0.012))
    cardigan = A.garment(kit, o, 'cardigan', cardi_keep, 0.011, sag=0.014, tears=0.0, tear_seed=51)
    trousers_keep = (np.isin(bn, ['hips']) & (P[:, 1] < hips[1] + 0.03)) | (bn == 'thigh') | ((bn == 'shin') & (along < 0.93))
    trousers = A.garment(kit, o, 'trousers', trousers_keep, 0.008, sag=0.016, tears=0.0)
    shoes_keep = np.isin(bn, ['foot', 'toes']) | ((bn == 'shin') & (along > 0.9))
    crown_d = posed.tail('head') - posed.head('head')
    crown_d /= np.linalg.norm(crown_d)
    above = (P - posed.head('head')) @ crown_d
    scalp_keep = (bn == 'head') & (N @ crown_d > -0.3) & (N @ face_fwd < 0.15) & (above > 0.035)
    bare = (N @ crown_d > 0.78) & (kit.fbm(P, 9.0, 2, seed=97) > -0.15)
    hair_ob = A.garment(kit, o, 'hair', scalp_keep & ~bare, 0.0035, sag=0.0, tears=0.12, tear_seed=98, tear_freq=24.0, smooth_edges=2)
    shoes = A.garment(kit, o, 'shoes', shoes_keep, 0.006, sag=0.0, tears=0.0, smooth_edges=1)

    # knit and wool hang loose round thin limbs, and span the body's hollows
    A.loosen(kit, cardigan, posed, {'upperarm': (0.06, 0.052), 'forearm': (0.052, 0.044)})
    A.loosen(kit, trousers, posed, {'thigh': (0.09, 0.07), 'shin': (0.068, 0.058)})
    C = kit.to_game_np(kit.verts_np(cardigan))
    cb, _ = A.nearest_bones(C, posed)
    on_trunk = np.isin(np.array([posed.bones[i]['name'].split('.')[0] for i in cb]), ['hips', 'spine', 'chest', 'upperchest', 'thigh'])
    A.hull_slices(kit, cardigan, on_trunk & (C[:, 1] < neck[1] - 0.05))
    A.hang(kit, cardigan, (hips[0], hips[2]), v_low, mask=on_trunk)
    # its left front, buttoned a hole low, hangs a button lower: pulled down
    # the more the nearer the hem, the fold at the button band
    C = kit.to_game_np(kit.verts_np(cardigan))
    Cn = kit.to_game_np(kit.vnormals(cardigan))
    lf = on_trunk & (Cn @ fwd > 0.1) & (C[:, 2] > hips[2]) & (C[:, 1] < v_low)
    C[:, 1] -= 0.04 * smooth01((v_low - C[:, 1]) / (v_low - hem)) * smooth01((C[:, 0] + 0.004) / 0.02) * lf
    kit.set_verts_np(cardigan, kit.to_bl_np(C))
    T = kit.to_game_np(kit.verts_np(trousers))
    tb, _ = A.nearest_bones(T, posed)
    tn = np.array([posed.bones[i]['name'].split('.')[0] for i in tb])
    # the seat, down to the crotch seam a hand below the hips' joint
    seat = (tn == 'hips') | ((tn == 'thigh') & (T[:, 1] > hips[1] - 0.085))
    A.hull_slices(kit, trousers, seat, give=0.2)
    kit.smooth(cardigan, 3, 0.5)
    kit.smooth(trousers, 3, 0.5)

    kit.decimate(o, 10000)
    kit.decimate(cardigan, 5200)
    kit.decimate(trousers, 2400)
    kit.decimate(shoes, 900)
    kit.decimate(hair_ob, 1400)
    A.fill_hollows(kit, cardigan, 80, 0.5)
    A.fill_hollows(kit, trousers, 60, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)

    # ---- paint
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    # (by the bone each point belonged to, not the nearest: the long fingers
    # raised before its bowed face are nearer its brow than its skull's bone)
    bn_full = np.array([posed.bones[i]['name'] if i >= 0 else '' for i in owner])
    bn = np.array([n.split('.')[0] for n in bn_full])
    cav = kit.cavity(o)
    sallow = np.array(kit.srgb('#a6937a'))
    flush = np.array(kit.srgb('#c08a78'))
    grey = np.array(kit.srgb('#988f82'))
    ink = np.array(kit.srgb('#1f2330'))
    hair = np.array(kit.srgb('#4f4840'))
    col = sallow + (grey - sallow) * np.clip(0.5 + kit.fbm(P, 7.0, 3, seed=91), 0, 1)[:, None] * 0.35
    # the face flushed (the cheeks, the nose, the ears' rims)
    head = bn == 'head'
    for c in (cl, cr):
        d = np.linalg.norm(P - (c + np.array([0, -0.03, 0.005])), axis=1)
        col = col + (flush - col) * (np.exp(-(d / 0.03) ** 2) * head * 0.55)[:, None]
    nose = (cl + cr) / 2 + np.array([0, -0.03, 0.025])
    col = col + (flush - col) * (np.exp(-(np.linalg.norm(P - nose, axis=1) / 0.02) ** 2) * 0.5)[:, None]
    crown = posed.tail('head') - posed.head('head')
    crown /= np.linalg.norm(crown)
    # dark rings under its eyes (it reads all night), the lids' edges dark
    ring, lid = np.array(kit.srgb('#6a5a54')), np.array(kit.srgb('#3b302b'))
    for c in (cl, cr):
        d = np.linalg.norm(P - (c - crown * 0.013 + face_fwd * 0.004), axis=1)
        col = col + (ring - col) * (head * np.exp(-(d / 0.014) ** 2) * 0.6)[:, None]
        e = np.abs(np.linalg.norm(P - c, axis=1) - 0.014)
        col = col + (lid - col) * (head * np.exp(-(e / 0.003) ** 2) * ((P - c) @ face_fwd > 0.0) * 0.7)[:, None]
    # thin hair at the back and sides, the crown bare
    scalp = head & (N @ face_fwd < 0.1) & ((P - posed.head('head')) @ crown > 0.02)
    thin = np.clip((0.75 - N @ crown) * 2.0, 0, 1) * np.clip(0.6 + kit.fbm(P, 40.0, 2, seed=92), 0, 1)
    col = col + (hair - col) * (scalp * thin * 0.3)[:, None]
    # stubble on the jaw
    jaw_d = np.linalg.norm(P - posed.tail('jaw'), axis=1)
    col = col + (grey * 0.7 - col) * (head * np.exp(-(jaw_d / 0.05) ** 2) * (N @ face_fwd > -0.2) * 0.35)[:, None]
    # ink at the corners of its mouth and on its lip (it licks its fingers
    # to turn the pages)
    mouth = (teeth_u.mean(0) + teeth_l.mean(0)) / 2 + face_fwd * 0.012
    side = np.cross(face_fwd, crown)
    side /= np.linalg.norm(side)
    for k, (dx, r_) in enumerate(((0.0, 0.02), (0.021, 0.009), (-0.021, 0.009))):
        d = np.linalg.norm(P - (mouth + side * dx), axis=1)
        col = col + (ink - col) * (head * np.exp(-(d / r_) ** 2) * (0.6 if k == 0 else 0.8))[:, None]
    # ink on the fingers, most at the tips, and a smear on the right palm
    tip = np.array([n.startswith('finger') and n[7] == '3' for n in bn_full])
    mid = np.array([n.startswith('finger') and n[7] == '2' for n in bn_full])
    blot = np.clip(0.55 + kit.fbm(P, 60.0, 2, seed=93), 0, 1)
    col = col + (ink - col) * ((tip * 0.85 + mid * 0.45) * blot)[:, None]
    palm = (bn_full == 'hand.R') & (np.clip(kit.fbm(P, 30.0, 2, seed=94), 0, 1) > 0.15)
    col = col + (ink - col) * (palm * 0.4)[:, None]
    # a shirt in the V (grey-white, buttoned to the throat)
    shirt = np.array(kit.srgb('#8d887b'))
    in_v = (N @ fwd > 0.0) & (P[:, 2] > hips[2]) & (P[:, 1] > v_low - 0.02) & (P[:, 1] < neck[1] + 0.035) \
        & np.isin(bn, ['upperchest', 'chest', 'neck', 'clavicle'])
    col = np.where(in_v[:, None], shirt * (0.92 + 0.08 * np.clip(kit.fbm(P, 20.0, 2, seed=95) + 0.5, 0, 1))[:, None], col)
    skin_col = np.clip(col * (1 - 0.6 * cav)[:, None], 0.03, 0.85)

    def paint_knit(ob):
        C = kit.to_game_np(kit.verts_np(ob))
        Cn = kit.to_game_np(kit.vnormals(ob))
        cv = kit.cavity(ob)
        brown, worn, dark = (np.array(kit.srgb(h)) for h in ('#5d4836', '#7a6450', '#3a2c22'))
        col = brown + (worn - brown) * np.clip(0.5 + kit.fbm(C, 8.0, 3, seed=61), 0, 1)[:, None] * 0.5
        # the knit's ribs at the cuffs and the hem, and pilling
        rib = (np.sin(np.arctan2(C[:, 0] - hips[0], C[:, 2] - hips[2]) * 90.0) > 0.0)
        near_hem = C[:, 1] < hem + 0.045
        nb2, _ = A.nearest_bones(C, posed)
        bn2 = np.array([posed.bones[i]['name'].split('.')[0] for i in nb2])
        al2 = np.zeros(len(C))
        for i in np.unique(nb2):
            m = nb2 == i
            al2[m] = A.seg_param(C[m], posed.bones[i]['head'], posed.bones[i]['tail'])[0]
        cuff = (bn2 == 'forearm') & (al2 > 0.8)
        col = col * (1 - 0.12 * (rib & (near_hem | cuff)))[:, None]
        # the button band down the front, darker, and the pockets
        band = (Cn @ fwd > 0.3) & (np.abs(C[:, 0] - 0.004) < 0.013) & (C[:, 1] < v_low + 0.01)
        col = col + (dark - col) * (band * 0.45)[:, None]
        for sx in (1, -1):
            px = sx * 0.085
            pk = (Cn @ fwd > 0.3) & (np.abs(C[:, 0] - px) < 0.06) & (C[:, 1] > hem + 0.005) & (C[:, 1] < hem + 0.13)
            edge = pk & ((np.abs(C[:, 0] - px) > 0.05) | (C[:, 1] > hem + 0.12))
            col = col + (dark - col) * (edge * 0.5 + pk * 0.12)[:, None]
        # ink on the right cuff, grime at the elbows
        smear = cuff & (C[:, 0] < 0) & (kit.fbm(C, 25.0, 2, seed=62) > 0.1)
        col = col + (ink - col) * (smear * 0.5)[:, None]
        elbow = np.isin(bn2, ['upperarm', 'forearm']) & (np.minimum(np.abs(al2 - 1.0), np.abs(al2)) < 0.12)
        col = col * (1 - 0.15 * elbow)[:, None]
        return np.clip(col * (1 - 0.4 * cv)[:, None], 0.03, 0.85)

    def paint_plain(ob, base_hex, stain_hex, seed, dirt_k=0.6):
        C = kit.to_game_np(kit.verts_np(ob))
        cv = kit.cavity(ob)
        b, st, dirt = np.array(kit.srgb(base_hex)), np.array(kit.srgb(stain_hex)), np.array(kit.srgb('#2e2924'))
        col = b + (st - b) * np.clip(kit.fbm(C, 9.0, 3, seed=seed) * 1.4 + 0.3, 0, 1)[:, None]
        low = np.clip(1 - C[:, 1] / 0.3, 0, 1) ** 1.5
        col = col + (dirt - col) * (low * dirt_k)[:, None]
        return np.clip(col * (1 - 0.4 * cv)[:, None], 0.03, 0.85)

    # ---- paper: the sheet in its hands, sheets down its front and in its pockets
    def mesh(name, V, F):
        me = bpy.data.meshes.new(name)
        me.from_pydata([tuple(p) for p in kit.to_bl_np(np.asarray(V))], [], F)
        me.update()
        for p in me.polygons:
            p.use_smooth = True
        ob = kit.link(bpy.data.objects.new(name, me))
        kit.triangulate(ob)
        return ob
    paper_white, paper_old = np.array(kit.srgb('#d8d2c0')), np.array(kit.srgb('#bdb39a'))
    type_grey = np.array(kit.srgb('#77736a'))

    def paper_col(V, uv, seed, printed=True, age=0.4):
        col = paper_white + (paper_old - paper_white) * np.clip(age + 0.5 * kit.fbm(V, 18.0, 2, seed=seed), 0, 1)[:, None]
        if printed:
            # a page of type: grey lines inside the margins
            inside = (uv[:, 0] > 0.1) & (uv[:, 0] < 0.9) & (uv[:, 1] > 0.08) & (uv[:, 1] < 0.9)
            line = (np.sin(uv[:, 1] * np.pi * 2 * 22) > 0.2) & inside
            col = col + (type_grey - col) * (line * 0.35)[:, None]
        return np.clip(col, 0.03, 0.85)
    # the page against the long fingers' pads, on the side of its eyes: the
    # fingers behind it, the thumbs before it, as one holds a page to read
    Ps = kit.to_game_np(kit.verts_np(o))
    nbs, _ = A.nearest_bones(Ps, posed)
    fname = [posed.bones[i]['name'] for i in nbs]

    def fingers(side):
        return Ps[np.array([n.startswith('finger') and n[6] in '2345' and n.endswith(side) for n in fname])]
    FL, FR = fingers('.L'), fingers('.R')
    Fg = np.concatenate([FL, FR])
    c0 = Fg.mean(0)
    # (square to its eyes)
    eyes = (cl + cr) / 2
    to_eyes = eyes - c0
    to_eyes /= np.linalg.norm(to_eyes)
    across = FL.mean(0) - FR.mean(0)
    across -= to_eyes * (across @ to_eyes)
    across /= np.linalg.norm(across)
    page_up = np.cross(to_eyes, across)
    if page_up[1] < 0:
        page_up = -page_up
        across = -across
    # (held by its lower half; `sheet` faces right x up: the print to its eyes)
    held_c = c0 + to_eyes * (np.percentile((Fg - c0) @ to_eyes, 80) + 0.004) + page_up * 0.04
    Vh, Fh, uvh = sheet(held_c, across, page_up, 0.21, 0.297, 9, 12, bow=-0.012)
    held = mesh('sheet', Vh, Fh)
    held_col = paper_col(kit.to_game_np(kit.verts_np(held)), uvh, 71, True, 0.25)
    # down its front, in the V, under the cardigan
    chest_c = np.array([0.0, v_low + 0.03, 0.0])
    Pc = kit.to_game_np(kit.verts_np(o))
    near = Pc[(np.abs(Pc[:, 0]) < 0.04) & (np.abs(Pc[:, 1] - chest_c[1]) < 0.03) & (Pc[:, 2] > hips[2])]
    chest_c[2] = near[:, 2].max() + 0.004 if len(near) else hips[2] + 0.12
    stuffed = []
    # (slips of it, under the cardigan's edges: only their tops show in the V)
    for k, (dx, tilt, h) in enumerate(((-0.018, 0.28, 0.07), (0.02, -0.22, 0.065))):
        r = np.array([np.cos(tilt), np.sin(tilt), 0.0])
        u = np.array([-np.sin(tilt), np.cos(tilt), -0.25])
        u /= np.linalg.norm(u)
        V, F, uv = sheet(chest_c + np.array([dx, -h * 0.3, 0.002 * k]), r, u, 0.075, h, 3, 4, bow=0.003)
        stuffed.append((V, F, uv))
    # in the pockets, folded, standing out of them
    Pk = kit.to_game_np(kit.verts_np(cardigan))
    for sx in (1, -1):
        pc = np.array([sx * 0.085, hem + 0.15, 0.0])
        near = Pk[(np.abs(Pk[:, 0] - pc[0]) < 0.03) & (np.abs(Pk[:, 1] - pc[1]) < 0.03) & (Pk[:, 2] > hips[2])]
        pc[2] = (near[:, 2].max() if len(near) else hips[2] + 0.1) + 0.004
        for k, tilt in enumerate((0.15 * sx, -0.1 * sx)):
            r = np.array([np.cos(tilt), np.sin(tilt), 0.0])
            u = np.array([-np.sin(tilt), np.cos(tilt), 0.1])
            u /= np.linalg.norm(u)
            V, F, uv = sheet(pc + np.array([0.01 * k * sx, 0.0, 0.004 * k]), r, u, 0.09, 0.05, 3, 3, bow=0.006, lean=0.012)
            stuffed.append((V, F, uv))
    Vs = np.concatenate([s[0] for s in stuffed])
    Fs, uvs, off = [], [], 0
    for V, F, uv in stuffed:
        Fs += [tuple(i + off for i in f) for f in F]
        uvs.append(uv)
        off += len(V)
    pockets = mesh('papers', Vs, Fs)
    pockets_col = paper_col(kit.to_game_np(kit.verts_np(pockets)), np.concatenate(uvs), 72, True, 0.6)

    # ---- the eyes, on bones of their own (they read); the teeth
    # (a little large, a little forward: it stares)
    eye_l = cl + (eye_l - cl) * 1.2 + face_fwd * 0.0035
    eye_r = cr + (eye_r - cr) * 1.2 + face_fwd * 0.0035
    el, er = mesh('eye_l', eye_l, eye_l_f), mesh('eye_r', eye_r, eye_r_f)
    tu, tl = mesh('teeth_u', teeth_u, teeth_u_f), mesh('teeth_l', teeth_l, teeth_l_f)
    # looking down at the page
    look = held_c - eyes
    look /= np.linalg.norm(look)

    def eye_col(ob, c):
        E = kit.to_game_np(kit.verts_np(ob))
        r = np.linalg.norm(E - c, axis=1).max()
        k = ((E - c) @ look) / r
        white, iris, pupil = (np.array(kit.srgb(h)) for h in ('#cfc6b4', '#5a4a3a', '#1c1814'))
        col = np.tile(white, (len(E), 1))
        col[k > 0.62] = iris
        col[k > 0.86] = pupil
        return col
    tooth_col = lambda ob: np.tile(np.array(kit.srgb('#b5a986')), (len(ob.data.vertices), 1))

    body = kit.Body('reader', posed)
    body.skinned('skin', 'skin', o, colors=skin_col)
    body.skinned('cardigan', 'cloth', cardigan, colors=paint_knit(cardigan))
    body.skinned('trousers', 'cloth', trousers, colors=paint_plain(trousers, '#5b574e', '#47443d', 63))
    body.skinned('shoes', 'shoe', shoes, colors=paint_plain(shoes, '#2c2621', '#3d342b', 64, 0.2))
    body.skinned('papers', 'paper', pockets, colors=pockets_col)
    Hh = kit.to_game_np(kit.verts_np(hair_ob))
    hair_col = np.array(kit.srgb('#463f37')) + (np.array(kit.srgb('#8a8378')) - np.array(kit.srgb('#463f37'))) \
        * np.clip(kit.fbm(Hh, 50.0, 2, seed=99) + 0.2, 0, 1)[:, None] * 0.6
    # (on the head alone: weighted by heat, the hair at the nape stayed with
    # the neck when it lifts its head)
    head_i = posed.ix['head']
    on_head = lambda Q: (np.tile([head_i, 0, 0, 0], (len(Q), 1)), np.tile([1.0, 0, 0, 0], (len(Q), 1)))
    body.skinned_by('hair', 'hair', hair_ob, on_head, colors=np.clip(hair_col * (1 - 0.3 * kit.cavity(hair_ob))[:, None], 0.03, 0.85))
    # bones the skin is not weighted to (added after it): the eyes, at their
    # middles, looking at the page; the sheet, held in the right hand
    for name, c in (('eye.L', cl), ('eye.R', cr)):
        posed.add(name, 'head', tuple(c), tuple(c + look * 0.02))
    posed.add('sheet', 'hand.R', tuple(held_c), tuple(held_c + page_up * 0.1))
    body.rigid('eye.L', 'eye', 'eye.L', el, colors=eye_col(el, cl))
    body.rigid('eye.R', 'eye', 'eye.R', er, colors=eye_col(er, cr))
    body.rigid('teeth.upper', 'teeth', 'head', tu, colors=tooth_col(tu))
    body.rigid('teeth.lower', 'teeth', 'jaw', tl, colors=tooth_col(tl))
    body.rigid('sheet', 'paper', 'sheet', held, colors=held_col)
    return body
