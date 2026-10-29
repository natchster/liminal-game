"""
The seated man of the Loop (docs/LORE.md, *The Loop*): from the fifth lap,
someone sits in the third room, a man in a shirt and tie with nothing where
his face should be. He never moves while you can see him; look away and
look back, and he is nearer, chair and all.

MakeHuman's man (CC0), between young and old, sat upright on the Loop's
office chair (`modes::loop_rooms::add_chair`: the seat's top 0.51 m up,
its back 0.2 m behind), his hands on his knees, his head tipped a little to
one side; where his face was, skin. A white shirt buttoned to the collar,
a dark tie, grey office trousers, black shoes, his hair short and grey at
the sides. He is never posed in the game: the Loop draws him as one piece
with his chair (`body_geometry`), so his skeleton is only his own.

Its rule (it moves only unseen) is SCP-173's, by Moto42 (CC BY-SA 3.0); the
Loop is SCP-970's, by Freudian (CC BY-SA 3.0); see docs/LORE.md, Attribution.
"""
import numpy as np

ORDER = 62
SEAT = 0.515       # the chair's seat, its top (m)
BACK = -0.16       # how far back he sits on it: his back against the chair's (m)
TILT = 0.14        # his head, tipped to his left


def build(kit):
    import bpy
    import hm08, human, figure as F, anatomy as A
    base = hm08.Base().target('macro/male-young-thin', 0.5).target('macro/male-old', 0.5)
    eye_l = base.helper('l-eye')[0].mean(0)
    eye_r = base.helper('r-eye')[0].mean(0)
    o, skel = human.body(kit, base, 'seated')
    # sat: the trunk upright, the thighs forward (tipped down as far as puts
    # his soles on the floor), the shins down, the hands on the knees
    def aims_for(pitch):
        a = {'hips': (0, 1, -0.06), 'spine': (0, 1, -0.02), 'chest': (0, 1, 0.02), 'upperchest': (0, 1, 0.05),
             'neck': (TILT * 0.4, 1, 0.14), 'head': (TILT, 1, 0.1)}
        for s, sx in (('.L', 1), ('.R', -1)):
            a['thigh' + s] = (sx * 0.06, -pitch, 1)
            a['shin' + s] = (sx * 0.02, -1, 0.1)
            a['foot' + s] = (sx * 0.05, -0.4, 1)
            a['toes' + s] = (0, -0.1, 1)
            a['upperarm' + s] = (sx * 0.08, -1, 0.22)
            a['forearm' + s] = (-sx * 0.02, -0.95, 0.75)
            a['hand' + s] = (0.0, -1, 0.35)
        return a
    ankle_h = float(skel.head('foot.L')[1])           # the ankle over the sole, standing
    seat_h = float(skel.head('hips')[1] - min(skel.head('thigh.L')[1], skel.head('thigh.R')[1])) + 0.09

    def sole_over_floor(pitch):
        p = human.fk(skel, aims_for(pitch))
        lift = SEAT + seat_h - p.head('hips')[1]
        return p.head('foot.L')[1] + lift - ankle_h
    lo, hi = -0.3, 0.6
    for _ in range(30):
        mid = (lo + hi) / 2
        if sole_over_floor(mid) > 0:
            lo = mid
        else:
            hi = mid
    pitch = (lo + hi) / 2
    print(f'[creatures] seated: the thighs tipped {pitch:.3f}, the soles {sole_over_floor(pitch) * 1000:.1f} mm off the floor')
    aims = aims_for(pitch)
    p0 = human.fk(skel, aims)
    root = np.array(p0.head('hips')) + np.array([0.0, SEAT + seat_h - p0.head('hips')[1], 0.0])
    posed = human.pose(kit, o, skel, aims, root=root, smooth_weights=6)
    # back in the chair: his back against its back
    P = kit.to_game_np(kit.verts_np(o))
    back = P[(P[:, 1] > SEAT + 0.15) & (P[:, 1] < SEAT + 0.45), 2].min()
    shift = np.array([0.0, 0.0, BACK - back])
    kit.set_verts_np(o, kit.to_bl_np(P + shift))
    for b in posed.bones:
        b['head'] = tuple(np.array(b['head']) + shift)
        b['tail'] = tuple(np.array(b['tail']) + shift)
    to_head, Rh = F.follow(skel, posed, 'head')
    cl, cr = to_head(np.array([eye_l, eye_r])) + shift
    face_fwd = Rh @ np.array([0.0, 0.0, 1.0])
    crown = np.array(posed.tail('head')) - np.array(posed.head('head'))
    crown /= np.linalg.norm(crown)
    before = kit.to_game_np(kit.verts_np(o))
    kit.remesh(o, 0.005)
    kit.smooth(o, 2, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)
    eyes = (cl + cr) / 2
    skull = (np.array(posed.head('head')) + np.array(posed.tail('head'))) / 2 + face_fwd * 0.012 - crown * 0.03
    F.faceless(kit, o, eyes - crown * 0.03, face_fwd, crown, skull, half_w=0.066, half_h=0.09)

    # ---- what he wears
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    nb = np.where(owner >= 0, owner, A.nearest_bones(P, posed)[0])
    bn = np.array([posed.bones[i]['name'].split('.')[0] for i in nb])
    bn = np.where(np.char.startswith(bn, 'finger'), 'finger', bn)
    along = np.zeros(len(P))
    for i in np.unique(nb):
        m = nb == i
        along[m] = A.seg_param(P[m], posed.bones[i]['head'], posed.bones[i]['tail'])[0]
    hips = np.array(posed.head('hips'))
    neck = np.array(posed.head('neck'))
    trunk = np.isin(bn, ['hips', 'spine', 'chest', 'upperchest', 'clavicle'])
    shirt_keep = (trunk & (P[:, 1] > hips[1] - 0.02)) | (bn == 'upperarm') | ((bn == 'forearm') & (along < 0.93)) \
        | ((bn == 'neck') & (along < 0.3))
    shirt = A.garment(kit, o, 'shirt', shirt_keep, 0.007, sag=0.008, tears=0.0)
    # (to the waist all round, over the shirt tucked into it: below it the
    # skin showed between them in his lap)
    waist = hips[1] + 0.06
    trousers_keep = (P[:, 1] < waist) & ~np.isin(bn, ['forearm', 'hand', 'finger', 'foot', 'toes']) \
        & ~((bn == 'shin') & (along >= 0.93))
    trousers = A.garment(kit, o, 'trousers', trousers_keep, 0.009, sag=0.01, tears=0.0)
    shoes_keep = np.isin(bn, ['foot', 'toes']) | ((bn == 'shin') & (along > 0.9))
    shoes = A.garment(kit, o, 'shoes', shoes_keep, 0.007, sag=0.0, tears=0.0, smooth_edges=1)
    up_e = (P - eyes) @ crown
    front_e = (P - eyes) @ face_fwd
    scalp_keep = (bn == 'head') & (N @ crown > -0.6) & (
        ((front_e > -0.035) & (up_e > 0.06)) | ((front_e <= -0.035) & (up_e > 0.004)))
    hair_ob = A.garment(kit, o, 'hair', scalp_keep, 0.005, sag=0.0, tears=0.0, smooth_edges=3)
    A.loosen(kit, shirt, posed, {'upperarm': (0.06, 0.052), 'forearm': (0.05, 0.044)})
    A.loosen(kit, trousers, posed, {'thigh': (0.085, 0.07), 'shin': (0.058, 0.05)})
    kit.smooth(shirt, 3, 0.5)
    kit.smooth(trousers, 3, 0.5)
    kit.decimate(o, 7000)
    kit.decimate(shirt, 3600)
    kit.decimate(trousers, 2600)
    kit.decimate(shoes, 800)
    kit.decimate(hair_ob, 800)
    A.fill_hollows(kit, shirt, 60, 0.5)
    A.fill_hollows(kit, trousers, 60, 0.5)
    # his shoes on the floor, not in it (the soles were put on it bare)
    low = kit.to_game_np(kit.verts_np(shoes))[:, 1].min()
    if low < 0.002:
        lift = np.array([0.0, 0.002 - low, 0.0])
        for ob in (o, shirt, trousers, shoes, hair_ob):
            kit.set_verts_np(ob, kit.to_bl_np(kit.to_game_np(kit.verts_np(ob)) + lift))
        for b in posed.bones:
            b['head'] = tuple(np.array(b['head']) + lift)
            b['tail'] = tuple(np.array(b['tail']) + lift)
        hips = hips + lift
        neck = neck + lift
        waist = waist + lift[1]
    F.level_edge(kit, trousers, waist - 0.04, waist + 0.04, waist, side=1)
    F.enclose(kit, trousers, shirt, (hips[0], hips[2]), hips[1] - 0.03, waist + 0.005, gap=0.006)
    # the tie: a strip down his front from the collar, lying on the shirt (made last: nothing moves the shirt after)
    S = kit.to_game_np(kit.verts_np(shirt))
    Sn = kit.to_game_np(kit.vnormals(shirt))
    ys = np.linspace(neck[1] - 0.015, neck[1] - 0.42, 18)
    Vt, Ft = [], []
    for k, y in enumerate(ys):
        front = S[(np.abs(S[:, 0]) < 0.02) & (np.abs(S[:, 1] - y) < 0.012) & (Sn[:, 2] > 0.3)]
        z = (front[:, 2].max() if len(front) else S[:, 2].max()) + 0.004
        t = k / (len(ys) - 1)
        w = 0.022 + 0.026 * t * t if t < 0.92 else 0.048 * (1 - (t - 0.92) / 0.08)
        for x in (-w, 0.0, w):
            Vt.append((x, y, z + (0.0 if x else 0.003)))
    for k in range(len(ys) - 1):
        for j in range(2):
            a, b = k * 3 + j, (k + 1) * 3 + j
            # (counter-clockwise seen from before him: the game draws one side)
            Ft.append((a, b, b + 1, a + 1))
    me = bpy.data.meshes.new('tie')
    me.from_pydata([tuple(p) for p in kit.to_bl_np(np.array(Vt))], [], Ft)
    me.update()
    tie = kit.link(bpy.data.objects.new('tie', me))
    kit.triangulate(tie)


    # ---- paint
    def plain(ob, base_hex, alt_hex, seed, k=0.35, cav_k=0.45):
        C = kit.to_game_np(kit.verts_np(ob))
        cv = kit.cavity(ob)
        b, a = np.array(kit.srgb(base_hex)), np.array(kit.srgb(alt_hex))
        col = b + (a - b) * np.clip(kit.fbm(C, 8.0, 3, seed=seed) * 1.2 + 0.4, 0, 1)[:, None] * k
        return np.clip(col * (1 - cav_k * cv)[:, None], 0.03, 0.85)
    P = kit.to_game_np(kit.verts_np(o))
    skin_col = plain(o, '#bfae98', '#a39686', 71, 0.5, 0.5)
    Hh = kit.to_game_np(kit.verts_np(hair_ob))
    hair_col = np.array(kit.srgb('#4a443e')) + (np.array(kit.srgb('#8f8a83')) - np.array(kit.srgb('#4a443e'))) \
        * np.clip((np.abs(Hh[:, 0]) - 0.05) / 0.04, 0, 1)[:, None] * 0.7
    tie_col = np.tile(np.array(kit.srgb('#3b1f1d')), (len(tie.data.vertices), 1))

    body = kit.Body('seated', posed)
    body.skinned('skin', 'skin', o, colors=skin_col)
    body.skinned('shirt', 'cloth', shirt, colors=plain(shirt, '#c9c6bb', '#aaa597', 72, 0.4))
    body.skinned('trousers', 'cloth', trousers, colors=plain(trousers, '#45464a', '#38393c', 73))
    body.skinned('shoes', 'shoe', shoes, colors=plain(shoes, '#1d1b19', '#2a2725', 74, 0.3, 0.2))
    head_i = posed.ix['head']
    chest_i = posed.ix['upperchest']
    on = lambda i: (lambda Q: (np.tile([i, 0, 0, 0], (len(Q), 1)), np.tile([1.0, 0, 0, 0], (len(Q), 1))))
    body.skinned_by('hair', 'hair', hair_ob, on(head_i), colors=np.clip(hair_col, 0.03, 0.85))
    body.skinned_by('tie', 'cloth', tie, on(chest_i), colors=tie_col)
    return body
