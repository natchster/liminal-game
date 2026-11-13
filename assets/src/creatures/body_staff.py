"""
The Showroom's staff (docs/design/LORE.md, *The Showroom*): too tall, and no faces.
By day they browse the floor with you and stop to look at you; by night
they hunt.

MakeHuman's man (CC0), between a thin young one and an older, heavier one,
drawn out along his bones to the staff rig's 2.14 m
(`anomalies::rig::HUMAN_RIG_STAFF`: long in the leg and the arm, the head
a little too long for the body), not widened; bald, and where his face
was, skin (smoothed away: no eyes, no nose, no mouth, the brow and the
cheeks running on into each other). A store's uniform worn neat: a mustard
polo shirt buttoned to the collar and tucked in, a blank name badge, dark
work trousers and a belt, black shoes. Skinned to the rig the game walks
him on (`figure.fit`).

(Four bodies before this one lost blind reviews to the old staff: thin,
with short hair, its arms read as sticks with no elbows, its shirt worn
out over the trousers as a smock, its head small under the hair; then its
hands as spikes, its face a flat egg, its collar a mock neck. This one,
with the arms' muscles and the neck's tendons in relief, a faint hollow
for a face, an open collar, fuller hands, a badge with a photograph and
a buckle, was preferred by two reviewers at 2 m and 4 m.)

Inspired by SCP-3008 "A Perfectly Normal, Regular Old IKEA" by Mortos
(CC BY-SA 3.0); see docs/design/LORE.md, Attribution.
"""
import os
import numpy as np

ORDER = 60


def smooth01(x):
    x = np.clip(x, 0.0, 1.0)
    return x * x * (3 - 2 * x)


def build(kit):
    import bpy
    import hm08, human, figure as F, anatomy as A
    base = hm08.Base().target('macro/male-young-thin', 0.45).target('macro/male-old', 0.55)
    eye_l = base.helper('l-eye')[0].mean(0)
    eye_r = base.helper('r-eye')[0].mean(0)
    o, posed, rig = F.fit(kit, human, base, 'staff', 'staff', head=1.18)
    rest = human.skeleton(kit, base)
    to_head, Rh = F.follow(rest, posed, 'head')
    cl, cr = to_head(np.array([eye_l, eye_r]))
    face_fwd = Rh @ np.array([0.0, 0.0, 1.0])
    crown = np.array(posed.tail('head')) - np.array(posed.head('head'))
    crown /= np.linalg.norm(crown)
    eyes = (cl + cr) / 2
    stretch = np.linalg.norm(np.array(posed.tail('head')) - np.array(posed.head('head'))) \
        / np.linalg.norm(np.array(rest.tail('head')) - np.array(rest.head('head')))
    skull = (np.array(posed.head('head')) + np.array(posed.tail('head'))) / 2 + face_fwd * 0.012
    # no face, first on MakeHuman's own skin (its eyes are holes there, which
    # the remesh then closes flush; done after it, the closed sockets'
    # walls overhung and stayed as pits), then again after the remesh
    F.faceless(kit, o, eyes - crown * 0.03 * stretch, face_fwd, crown, skull, half_w=0.066, half_h=0.09 * stretch)
    before = kit.to_game_np(kit.verts_np(o))
    # (fine: at 5 mm the hanging fingers ran together into a mitten)
    kit.remesh(o, 0.003)
    kit.smooth(o, 2, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)
    # hands that read as hands at a few metres: the fingers half as thick
    # again as a man's (thin and close together they read as spikes), the
    # palm a little broader; as long as the rig's
    P = kit.to_game_np(kit.verts_np(o))
    P1 = P.copy()
    for i, b in enumerate(posed.bones):
        k = 1.45 if b['name'].startswith('finger') else (1.12 if b['name'].startswith('hand') else None)
        m = owner == i
        if k is None or not m.any():
            continue
        rv = A.seg_param(P[m], b['head'], b['tail'])[2]
        P1[m] = P[m] + rv * (k - 1.0)
    kit.set_verts_np(o, kit.to_bl_np(A.relax_offsets(kit, o, P, P1, 4)))
    # starved at the arms: thin between the joints, the elbows and the wrists
    # standing out (a body with smooth arms read as made of rubber)
    P = kit.to_game_np(kit.verts_np(o))
    waste = {b['name']: 0.16 for b in posed.bones if b['name'].split('.')[0] in ('upperarm', 'forearm')}
    W = A.waste(P, posed, list(waste), waste, joint_keep=0.045, owner=owner,
                buried=[n for n in waste if n.split('.')[0] == 'upperarm'])
    kit.set_verts_np(o, kit.to_bl_np(A.relax_offsets(kit, o, P, W, 12)))
    # no face: the eyes, the nose and the mouth gone under the skin
    # (more passes on the fine mesh: they reach as far as they did at 5 mm)
    F.faceless(kit, o, eyes - crown * 0.03 * stretch, face_fwd, crown, skull, half_w=0.066, half_h=0.09 * stretch, iters=24, flatten=8,
               hollow=0.006)
    bn_o = [posed.bones[i]['name'] if i >= 0 else '' for i in owner]
    # what the old staff had and the first new ones did not (four blind
    # reviews): the arms' muscles (the biceps, the forearm's swell below the
    # elbow) and the neck's two tendons, in relief
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    lift = np.zeros(len(P))
    for b in posed.bones:
        fam = b['name'].split('.')[0]
        if fam not in ('upperarm', 'forearm'):
            continue
        m = owner == posed.ix[b['name']]
        t = A.seg_param(P[m], b['head'], b['tail'])[0]
        if fam == 'upperarm':
            # the biceps, at the front of the upper arm's middle
            lift[m] = 0.0055 * np.sin(np.pi * np.clip((t - 0.25) / 0.6, 0, 1)) ** 2 * np.clip(N[m] @ np.array([0, 0, 1.0]), 0, 1)
        else:
            # the forearm's swell, outside, just below the elbow
            side = np.array([np.sign(b['head'][0]) or 1.0, 0.0, 0.3])
            side /= np.linalg.norm(side)
            lift[m] = 0.004 * np.exp(-((t - 0.22) / 0.12) ** 2) * np.clip(N[m] @ side, 0, 1)
    nk = np.array(posed.head('neck'))
    hd = np.array(posed.head('head'))
    for sx in (1, -1):
        # behind the ear down to the hollow of the throat
        a = hd + np.array([sx * 0.055, 0.0, -0.01])
        c = nk + np.array([sx * 0.012, -0.01, 0.05])
        d = A.seg_param(P, a, c)[1]
        lift += 0.003 * np.exp(-(d / 0.009) ** 2) * (np.array([n.startswith('neck') for n in bn_o])) * (N @ face_fwd > -0.2)
    kit.set_verts_np(o, kit.to_bl_np(P + N * lift[:, None]))

    # ---- what he wears, grown off the skin
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    # (whose each point is by the bone it belonged to, not the nearest: the
    # hanging hands are nearer the hips than the spine is)
    nb = np.where(owner >= 0, owner, A.nearest_bones(P, posed)[0])
    bn = np.array([posed.bones[i]['name'].split('.')[0] for i in nb])
    # (the fingers by family: their names carry their number, 'finger12')
    bn = np.where(np.char.startswith(bn, 'finger'), 'finger', bn)
    along = np.zeros(len(P))
    for i in np.unique(nb):
        m = nb == i
        along[m] = A.seg_param(P[m], posed.bones[i]['head'], posed.bones[i]['tail'])[0]
    hips = np.array(posed.head('hips'))
    neck = np.array(posed.head('neck'))
    fwd = np.array([0.0, 0.0, 1.0])
    # the polo: tucked in (to a little above the hips' joint, under the
    # trousers' waist), the sleeves to the middle of the upper arm, buttoned
    # to the throat
    hem = hips[1] + 0.03
    # (the trunk by where it is: within a hand of the body's axis, the hands
    # and forearms hanging beside it apart; by its bones, the side of the
    # waist went with the arm that hung from it)
    axis_d = np.linalg.norm(P[:, [0, 2]] - hips[[0, 2]], axis=1)
    trunk = ((axis_d < 0.215) & ~np.isin(bn, ['forearm', 'hand', 'finger', 'neck', 'head', 'jaw'])
             & (P[:, 1] < neck[1] + 0.01)) | np.isin(bn, ['clavicle'])
    polo_keep = ((trunk & (P[:, 1] > hem)) | ((bn == 'upperarm') & (along < 0.5)) | ((bn == 'neck') & (along < 0.3)))
    # open at the front, in a V down to the placket (buttoned to the throat
    # it read as a mock neck)
    v_open = ((P[:, 2] - neck[2]) > 0.0) & (np.abs(P[:, 0]) < 0.012 + 0.35 * np.clip(P[:, 1] - (neck[1] - 0.075), 0, 1)) \
        & (P[:, 1] > neck[1] - 0.075)
    polo = A.garment(kit, o, 'polo', polo_keep & ~v_open, 0.007, sag=0.01, tears=0.0)
    # the collar: a band round the neck's foot, standing a little off it
    # (by where it is, a band round the neck's foot: by its bones it left the
    # hollow of the throat bare)
    neck_d = np.linalg.norm(P[:, [0, 2]] - neck[[0, 2]], axis=1)
    collar_keep = (neck_d < 0.085) & (P[:, 1] > neck[1] - 0.03) & (P[:, 1] < neck[1] + 0.04) & (bn != 'head') & ~v_open
    collar = A.garment(kit, o, 'collar', collar_keep, 0.013, sag=0.0, tears=0.0, smooth_edges=1)
    waist = hips[1] + 0.075
    # (to the waist all round: the thigh's own skin reaches up the hip's side)
    trousers_keep = (P[:, 1] < waist) & ~np.isin(bn, ['forearm', 'hand', 'finger', 'foot', 'toes']) \
        & ~((bn == 'shin') & (along >= 0.94))
    trousers = A.garment(kit, o, 'trousers', trousers_keep, 0.012, sag=0.012, tears=0.0)
    shoes_keep = np.isin(bn, ['foot', 'toes']) | ((bn == 'shin') & (along > 0.9))
    shoes = A.garment(kit, o, 'shoes', shoes_keep, 0.007, sag=0.0, tears=0.0, smooth_edges=1)
    A.loosen(kit, polo, posed, {'upperarm': (0.058, 0.05)})
    A.loosen(kit, trousers, posed, {'thigh': (0.074, 0.06), 'shin': (0.054, 0.047)})
    T = kit.to_game_np(kit.verts_np(trousers))
    tb, _ = A.nearest_bones(T, posed)
    tn = np.array([posed.bones[i]['name'].split('.')[0] for i in tb])
    A.hull_slices(kit, trousers, (tn == 'hips') | ((tn == 'thigh') & (T[:, 1] > hips[1] - 0.085)), give=0.2)
    Cp = kit.to_game_np(kit.verts_np(polo))
    cb, _ = A.nearest_bones(Cp, posed)
    on_trunk = np.isin(np.array([posed.bones[i]['name'].split('.')[0] for i in cb]), ['hips', 'spine', 'chest', 'upperchest', 'thigh'])
    A.hull_slices(kit, polo, on_trunk & (Cp[:, 1] < neck[1] - 0.06))
    # the shirt falls straight from the chest to where it is tucked, over
    # the ribs and the hollow of the waist
    A.hang(kit, polo, (hips[0], hips[2]), neck[1] - 0.2, mask=on_trunk, give=0.25)
    kit.smooth(polo, 6, 0.5)
    F.smooth_masked(kit, polo, np.ones(len(polo.data.vertices)), 10)
    kit.smooth(trousers, 3, 0.5)

    # (the hands kept from it, and the head: the decimation took it to facets)
    own = A.carry_values(kit, before, posed.owner, o)
    head_m = np.array([1.0 if i >= 0 and posed.bones[i]['name'] in ('head', 'jaw', 'neck') else 0.0 for i in own])
    F.decimate_keeping(kit, o, 9300, np.maximum(0.7 * F.hand_mask(posed, own), 0.5 * head_m))
    kit.decimate(polo, 3400)
    kit.smooth(collar, 2, 0.5)
    kit.decimate(collar, 900)
    kit.decimate(trousers, 3000)
    kit.decimate(shoes, 900)
    A.fill_hollows(kit, polo, 60, 0.5)
    A.fill_hollows(kit, trousers, 60, 0.5)
    # the shirt over the skin, and the trousers' waist round the shirt
    # tucked into it, the waistband level (last: nothing moves them after)
    owner = A.carry_values(kit, before, posed.owner, o)
    on_body = np.isin(np.array([posed.bones[i]['name'].split('.')[0] if i >= 0 else '' for i in owner]),
                      ['hips', 'spine', 'chest', 'upperchest'])
    F.enclose(kit, polo, o, (hips[0], hips[2]), hem - 0.005, neck[1] - 0.1, gap=0.003, inner_mask=on_body)
    F.level_edge(kit, trousers, waist - 0.04, waist + 0.04, waist, side=1)
    F.enclose(kit, trousers, polo, (hips[0], hips[2]), hem - 0.02, waist + 0.005, gap=0.006)

    # ---- the badge: a blank card on the left breast
    Cp = kit.to_game_np(kit.verts_np(polo))
    Cn = kit.to_game_np(kit.vnormals(polo))
    spot = np.array([0.085, neck[1] - 0.15, 0.0])
    near = Cp[(np.abs(Cp[:, 0] - spot[0]) < 0.025) & (np.abs(Cp[:, 1] - spot[1]) < 0.02) & (Cn @ fwd > 0.3)]
    spot[2] = (near[:, 2].max() if len(near) else hips[2] + 0.12) + 0.0025
    w, h = 0.072, 0.026
    # (a card fine enough to carry its photograph and its lines of type in
    # the points' colours)
    nx, ny = 25, 10
    xs, ys = np.linspace(-w / 2, w / 2, nx), np.linspace(-h / 2, h / 2, ny)
    Vb = [spot + np.array([x, y, 0.0]) for y in ys for x in xs]
    Fb = [(j * nx + i, j * nx + i + 1, (j + 1) * nx + i + 1, (j + 1) * nx + i) for j in range(ny - 1) for i in range(nx - 1)]
    back = len(Vb)
    Vb += [spot + np.array([x, y, -0.002]) for x, y in ((-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2))]
    Fb.append((back, back + 3, back + 2, back + 1))
    me = bpy.data.meshes.new('badge')
    me.from_pydata([tuple(p) for p in kit.to_bl_np(np.array(Vb))], [], Fb)
    me.update()
    badge = kit.link(bpy.data.objects.new('badge', me))
    kit.triangulate(badge)
    badge_uv = np.array([((x + w / 2) / w, (y + h / 2) / h) for y in ys for x in xs] + [(0, 0)] * 4)
    # ---- the buckle: a plain one, dull steel, on the belt in front
    T0 = kit.to_game_np(kit.verts_np(trousers))
    front_w = T0[(np.abs(T0[:, 0]) < 0.03) & (np.abs(T0[:, 1] - (waist - 0.017)) < 0.015)]
    bz = (front_w[:, 2].max() if len(front_w) else hips[2] + 0.12) + 0.002
    bc = np.array([0.0, waist - 0.018, bz])
    Vk = [bc + np.array([sx * 0.024, sy * 0.017, sz * 0.003]) for sz in (-1, 1) for sy in (-1, 1) for sx in (-1, 1)]
    Fk = [(4, 5, 7, 6), (0, 2, 3, 1), (0, 1, 5, 4), (2, 6, 7, 3), (0, 4, 6, 2), (1, 3, 7, 5)]
    me = bpy.data.meshes.new('buckle')
    me.from_pydata([tuple(p) for p in kit.to_bl_np(np.array(Vk))], [], Fk)
    me.update()
    buckle = kit.link(bpy.data.objects.new('buckle', me))
    kit.triangulate(buckle)

    # ---- paint
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    bn_full = np.array([posed.bones[i]['name'] if i >= 0 else '' for i in owner])
    bn = np.array([n.split('.')[0] for n in bn_full])
    cav = kit.cavity(o)
    pale, cool, warm = (np.array(kit.srgb(h)) for h in ('#c4b29c', '#a99c8e', '#c89f8c'))
    col = pale + (cool - pale) * np.clip(0.5 + kit.fbm(P, 6.0, 3, seed=31), 0, 1)[:, None] * 0.4
    # warmth where blood is near the skin: the knuckles, the ears' rims; the
    # face, where it should flush, as pale as the rest
    knuckle = np.isin(bn, ['hand', 'finger']) & (np.clip(kit.fbm(P, 40.0, 2, seed=32), 0, 1) > 0.2)
    col = col + (warm - col) * (knuckle * 0.25)[:, None]
    head = bn == 'head'
    side = np.cross(face_fwd, crown)
    side /= np.linalg.norm(side)
    ear = head & (np.abs((P - eyes) @ side) > 0.065) & ((P - eyes) @ face_fwd < -0.02)
    col = col + (warm - col) * (ear * 0.35)[:, None]
    # a shadow of a beard where the jaw is, shaved that morning
    jaw = np.linalg.norm(P - np.array(posed.tail('jaw')), axis=1)
    col = col * (1 - 0.08 * head * np.exp(-(jaw / 0.05) ** 2))[:, None]
    skin_col = np.clip(col * (1 - 0.5 * cav)[:, None], 0.03, 0.85)

    def paint_polo(ob):
        C = kit.to_game_np(kit.verts_np(ob))
        Cn = kit.to_game_np(kit.vnormals(ob))
        cv = kit.cavity(ob)
        mustard, faded, seam = (np.array(kit.srgb(h)) for h in ('#a9822c', '#b99a52', '#6e5520'))
        col = mustard + (faded - mustard) * np.clip(0.5 + kit.fbm(C, 7.0, 3, seed=41), 0, 1)[:, None] * 0.35
        # the placket: a short band down from the collar
        band = (Cn @ fwd > 0.4) & (np.abs(C[:, 0]) < 0.014) & (C[:, 1] > neck[1] - 0.14) & (C[:, 1] < neck[1] + 0.01)
        col = col + (seam - col) * (band * 0.35)[:, None]
        # the sleeves' seams round the shoulders
        for sx in (1, -1):
            sh = np.array(posed.head('upperarm.' + ('L' if sx > 0 else 'R')))
            d = np.linalg.norm((C - sh)[:, [0, 1]] * np.array([1.0, 1.4]), axis=1)
            seam_m = np.exp(-((d - 0.055) / 0.007) ** 2) * (np.sign(C[:, 0]) == sx)
            col = col + (seam - col) * (seam_m * 0.5)[:, None]
        # (no buttons painted: on cloth this coarse a centimetre of colour
        # spreads over whole triangles, pale blotches at the throat)
        # (lightly in the creases: the skin's relief under the shirt read as
        # stains)
        return np.clip(col * (1 - 0.22 * cv)[:, None], 0.03, 0.85)

    def paint_plain(ob, base_hex, alt_hex, seed, dirt_k=0.3):
        C = kit.to_game_np(kit.verts_np(ob))
        cv = kit.cavity(ob)
        b, st, dirt = np.array(kit.srgb(base_hex)), np.array(kit.srgb(alt_hex)), np.array(kit.srgb('#24221f'))
        col = b + (st - b) * np.clip(kit.fbm(C, 9.0, 3, seed=seed) * 1.2 + 0.3, 0, 1)[:, None]
        low = np.clip(1 - C[:, 1] / 0.25, 0, 1) ** 1.5
        col = col + (dirt - col) * (low * dirt_k)[:, None]
        return np.clip(col * (1 - 0.45 * cv)[:, None], 0.03, 0.85)

    T = kit.to_game_np(kit.verts_np(trousers))
    trousers_col = paint_plain(trousers, '#2b2f37', '#353943', 51)
    # the belt: a dark band down from the waistband (its buckle a piece of
    # its own: painted on cloth this coarse it spread to a pale blotch)
    belt = T[:, 1] > waist - 0.034
    trousers_col = np.where(belt[:, None], np.array(kit.srgb('#1c1a18')), trousers_col)
    Bv = kit.to_game_np(kit.verts_np(badge))
    card, edge_c, photo, ink = (np.array(kit.srgb(h)) for h in ('#d9d4c6', '#9c917c', '#6f6a62', '#4a4741'))
    badge_col = np.tile(card, (len(Bv), 1))
    u, v = badge_uv[:, 0], badge_uv[:, 1]
    # the photograph at the left (a head and shoulders, too dark to make out)
    in_photo = (u > 0.06) & (u < 0.3) & (v > 0.15) & (v < 0.85)
    badge_col[in_photo] = photo
    figure = in_photo & ((((u - 0.18) / 0.06) ** 2 + ((v - 0.6) / 0.16) ** 2 < 1) | ((v < 0.36) & (np.abs(u - 0.18) < 0.09)))
    badge_col[figure] = np.array(kit.srgb('#3e3a35'))
    # two lines of type at the right, a name and a word under it
    for vy, uw in ((0.68, 0.88), (0.38, 0.72)):
        badge_col[(u > 0.38) & (u < uw) & (np.abs(v - vy) < 0.07)] = ink
    badge_col[(u < 0.02) | (u > 0.98) | (v < 0.04) | (v > 0.96)] = edge_c
    badge_col[Bv[:, 2] < spot[2] - 0.001] = np.array(kit.srgb('#8a8170'))

    dbg = os.environ.get('FIGURE_DEBUG')
    if dbg:
        for ob, c in ((o, skin_col), (polo, paint_polo(polo)), (trousers, trousers_col), (collar, paint_polo(collar))):
            kit.paint_preview(ob, c)
        F.preview_band(kit, [o, polo, collar, trousers, badge], hips[1] - 0.12, hips[1] + 0.3, os.path.join(dbg, 'staff_waist'))
        F.preview_band(kit, [o, polo, collar], neck[1] - 0.05, 2.2, os.path.join(dbg, 'staff_head'))
    body = kit.Body('staff', rig)
    body.skinned('skin', 'skin', o, colors=skin_col)
    body.skinned('polo', 'cloth', polo, colors=paint_polo(polo))
    body.skinned('collar', 'cloth', collar, colors=paint_polo(collar))
    body.skinned_by('trousers', 'cloth', trousers, F.waist_on(kit, trousers, rig, 'pelvis', hips[1] - 0.02, waist - 0.02),
                    colors=trousers_col)
    body.skinned('shoes', 'shoe', shoes, colors=paint_plain(shoes, '#1e1c1b', '#2b2826', 53, 0.1))
    chest_i = rig.ix['chest']
    on = lambda i: (lambda Q: (np.tile([i, 0, 0, 0], (len(Q), 1)), np.tile([1.0, 0, 0, 0], (len(Q), 1))))
    body.skinned_by('badge', 'badge', badge, on(chest_i), colors=badge_col)
    body.skinned_by('buckle', 'shoe', buckle, on(rig.ix['pelvis']), colors=np.tile(np.array(kit.srgb('#6a665e')), (8, 1)))
    return body
