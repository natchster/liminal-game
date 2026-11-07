"""
The figure across the Building's void (docs/MEGASTRUCTURE.md, design 7 and
G7; docs/LORE.md, *Someone across the void*): someone who lives in the
Building, standing still at a parapet far off, arms down, turned toward you.
It is seen at 22-60 m in weak light and haze, so its silhouette carries it:
a person's proportions, the shoulders, a long coat's hem, a bare head.

MakeHuman's man (CC0), thin, between young and old, a little taller than
anyone (`HEIGHT`, by MakeHuman's own height target: a tall man's legs are
longer for his height and his head smaller), stood as a person stands
still: the arms brought down from MakeHuman's A pose to hang at his sides,
the feet a little apart, the head a little forward. A long grey work coat
of faded cotton, buttoned to its collar and falling to below the knee
(lofted round the body's slices, not grown off the skin: a coat falls
straight from the shoulders and the chest and hangs past the legs; soft
folds down the back and the sides), its sleeves to the wrists; dark
trousers, black shoes, short dark hair greying at the sides. Grey, paler
than the concrete, because it is lit like everything else: a dark coat was
lost against the Building's dark faces (`COAT` in `build`). His face is his
own: nothing of it reads that far off, and nothing is wrong with it.

He is never posed in the game: `world::far_figures` draws him as one piece
and turns him whole, so his skeleton is only his own and every part rides
on his hips.

The figures on the far ridge are SCP-3333's, by Jekeled (CC BY-SA 3.0); see
docs/LORE.md, Attribution.
"""
import numpy as np

ORDER = 64
HEIGHT = 1.93      # the crown over the soles (m): a little taller than anyone
HEM = 0.40         # the coat's hem over the soles (m), a hand under the knee


def keep_faces(kit, o, keep):
    """`o` cut to the faces with every point in `keep` (a mask over its points)"""
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(o.data)
    bm.verts.ensure_lookup_table()
    drop = [f for f in bm.faces if not all(keep[v.index] for v in f.verts)]
    bmesh.ops.delete(bm, geom=drop, context='FACES')
    loose = [v for v in bm.verts if not v.link_faces]
    bmesh.ops.delete(bm, geom=loose, context='VERTS')
    bm.to_mesh(o.data)
    bm.free()
    o.data.update()
    return o


def ray_out(H, c, u):
    """how far from `c` along the unit `u` the ray leaves the convex polygon `H`"""
    best = 0.0
    for a, b in zip(H, np.roll(H, -1, axis=0)):
        e = b - a
        den = u[0] * (-e[1]) - u[1] * (-e[0])
        if abs(den) < 1e-12:
            continue
        w = a - c
        t = (w[0] * (-e[1]) - w[1] * (-e[0])) / den
        s = (u[0] * w[1] - u[1] * w[0]) / den
        if t > 0 and -1e-9 <= s <= 1 + 1e-9:
            best = max(best, t)
    return best


def loft(kit, A, name, P, use, ys, n_ang, ease, hang_from, give=0.2, smooth=4, folds=None):
    """a coat's body: at each height of `ys` (from the top down), a ring at
    the convex hull of the points of `P` (mask `use`) in that slice, round
    the slice's own middle, pushed out by `ease(y)`; below `hang_from` no
    ring comes back in nearer the middle than the cloth over it (`give`: the
    share of the way it may), so it falls straight from the widest of the
    body; `folds(y, th)` adds the cloth's own (metres out, by height and by
    the angle round from the front). Returns the object (rings of quads,
    open at both ends) and its rings' points (len(ys) x n_ang x 3)."""
    import bpy
    th = np.linspace(-np.pi, np.pi, n_ang, endpoint=False)
    U = np.stack([np.sin(th), np.cos(th)], axis=1)
    dy = abs(ys[1] - ys[0])
    hulls, mids = [], []
    for y in ys:
        m = use & (np.abs(P[:, 1] - y) < max(dy, 0.012))
        H = A._hull2(P[m][:, [0, 2]])
        hulls.append(H)
        mids.append(H.mean(0))
    mids = np.array(mids)
    # (the middles smoothed up and down: a ring's middle jumping from one
    # slice to the next would kink the cloth)
    for _ in range(20):
        mids[1:-1] = 0.5 * mids[1:-1] + 0.25 * (mids[:-2] + mids[2:])
    R = np.array([[ray_out(H, c, u) for u in U] for H, c in zip(hulls, mids)])
    R += np.array([ease(y) for y in ys])[:, None]
    run = None
    for k, y in enumerate(ys):
        if y > hang_from:
            continue
        run = R[k].copy() if run is None else np.maximum(run, R[k])
        R[k] = run - (run - R[k]) * give
        run = np.maximum(run, R[k])
    for _ in range(smooth):
        R = 0.5 * R + 0.25 * (np.roll(R, 1, axis=1) + np.roll(R, -1, axis=1))
        R[1:-1] = 0.6 * R[1:-1] + 0.2 * (R[:-2] + R[2:])
    if folds is not None:
        R = R + np.array([folds(y, th) for y in ys])
    V = np.zeros((len(ys), n_ang, 3))
    V[:, :, 0] = mids[:, None, 0] + R * U[None, :, 0]
    V[:, :, 1] = np.array(ys)[:, None]
    V[:, :, 2] = mids[:, None, 1] + R * U[None, :, 1]
    F = []
    for k in range(len(ys) - 1):
        for j in range(n_ang):
            a, b = k * n_ang + j, k * n_ang + (j + 1) % n_ang
            # (facing out: the rings go round counter-clockwise seen from above
            # in the game's frame, the top ring first)
            F.append((a, a + n_ang, b + n_ang, b))
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(p) for p in kit.to_bl_np(V.reshape(-1, 3))], [], F)
    me.update()
    for p in me.polygons:
        p.use_smooth = True
    o = kit.link(bpy.data.objects.new(name, me))
    return o, V


def build(kit):
    import bpy
    import hm08, human, figure as F, anatomy as A
    # (tall by MakeHuman's own height target, not scaled up: a tall man's
    # legs are longer for his height, his head smaller)
    base = hm08.Base().target('macro/male-old-thin', 0.7).target('macro/male-young-thin', 0.3) \
        .target('macro/male-old-thin-tall', 0.36).target('macro/male-old', 0.3)
    eye_l = base.helper('l-eye')[0].mean(0)
    eye_r = base.helper('r-eye')[0].mean(0)
    o, skel = human.body(kit, base, 'far_figure')
    # stood still: the arms down at his sides from the A pose, a little out
    # from the coat, the elbows soft; the feet a little apart; the head a
    # little forward
    aims = {'neck': (0, 1, 0.16), 'head': (0, 1, 0.1)}
    for s, sx in (('.L', 1), ('.R', -1)):
        aims['upperarm' + s] = (sx * 0.13, -1, -0.02)
        aims['forearm' + s] = (sx * 0.06, -1, 0.08)
        aims['hand' + s] = (sx * 0.02, -1, 0.05)
        aims['thigh' + s] = (sx * 0.03, -1, 0.0)
        aims['shin' + s] = (sx * 0.005, -1, -0.01)
    posed = human.pose(kit, o, skel, aims, smooth_weights=6)
    to_head, Rh = F.follow(skel, posed, 'head')
    eyes = to_head(np.array([eye_l, eye_r])).mean(0)
    face_fwd = Rh @ np.array([0.0, 0.0, 1.0])
    crown = np.array(posed.tail('head')) - np.array(posed.head('head'))
    crown /= np.linalg.norm(crown)

    # a little taller than anyone, his feet under him: the soles on the
    # floor and the middle between his ankles over the origin
    P = kit.to_game_np(kit.verts_np(o))
    ank = (np.array(posed.head('foot.L')) + np.array(posed.head('foot.R'))) / 2
    k = HEIGHT / (P[:, 1].max() - P[:, 1].min())
    shift = np.array([-ank[0], -P[:, 1].min(), -ank[2]])

    def move(X):
        return (np.asarray(X, float) + shift) * k
    kit.set_verts_np(o, kit.to_bl_np(move(P)))
    for b in posed.bones:
        b['head'] = tuple(move(b['head']))
        b['tail'] = tuple(move(b['tail']))
    eyes = move(eyes)
    print(f'[creatures] far_figure: scaled {k:.3f} to {HEIGHT} m')

    before = kit.to_game_np(kit.verts_np(o))
    kit.remesh(o, 0.005)
    kit.smooth(o, 2, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)

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
    sh_y = min(posed.head('upperarm.L')[1], posed.head('upperarm.R')[1])

    # ---- the coat: its body lofted round the trunk and the legs, from the
    # foot of the neck to under the knees, falling straight from the chest
    trunk = np.isin(bn, ['hips', 'spine', 'chest', 'upperchest', 'clavicle', 'thigh', 'shin']) \
        | ((bn == 'upperarm') & (along < 0.3)) | ((bn == 'neck') & (along < 0.25))
    top = neck[1] + 0.01
    ys = np.arange(top, HEM - 1e-6, -0.012)

    def ease(y):
        # wool and the air under it at the chest; more toward the hem (it
        # flares a little); close round the neck (the collar)
        e = 0.016 + 0.022 * kit.smoothstep(hips[1], HEM, y)
        return e + 0.006 * kit.smoothstep(sh_y, top, y)
    def folds(y, th):
        # long soft folds down the skirt from the seat, at the sides and the
        # back (the front hangs flat where it is buttoned)
        k = 0.0065 * kit.smoothstep(hips[1] - 0.05, HEM + 0.1, y) * (0.25 + 0.75 * (1 - np.cos(th)) / 2)
        return k * (0.6 * np.sin(6 * th + 0.7) + 0.4 * np.sin(10 * th + 2.3))
    # (falling from the chest, a little in at the waist: a coat cut for a
    # man, neither a robe nor a dress)
    coat, rings = loft(kit, A, 'coat', P, trunk, ys, 48, ease, hang_from=sh_y - 0.14, give=0.3, folds=folds)
    # the collar: a band standing round the neck from the coat's top
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(coat.data)
    bm.verts.ensure_lookup_table()
    n_ang = rings.shape[1]
    top_ring = rings[0]
    mid = top_ring[:, [0, 2]].mean(0)
    new = []
    for h, out in ((0.022, 0.006), (0.036, 0.024)):
        ring = []
        for p in top_ring:
            d = p[[0, 2]] - mid
            q = np.array([p[0] + d[0] / np.linalg.norm(d) * out, p[1] + h, p[2] + d[1] / np.linalg.norm(d) * out])
            ring.append(bm.verts.new(kit.to_bl(q)))
        new.append(ring)
    bm.verts.ensure_lookup_table()
    prev = [bm.verts[j] for j in range(n_ang)]
    for ring in new:
        for j in range(n_ang):
            bm.faces.new((ring[j], prev[j], prev[(j + 1) % n_ang], ring[(j + 1) % n_ang]))
        prev = ring
    # the hem closed underneath: the lining's dark, not the sky through it
    # (one side is drawn: looked up at, the coat's inside would be a hole)
    last = [bm.verts[(len(ys) - 1) * n_ang + j] for j in range(n_ang)]
    hm = rings[-1].mean(0)
    hub = bm.verts.new(kit.to_bl(np.array([hm[0], HEM + 0.004, hm[2]])))
    for j in range(n_ang):
        bm.faces.new((last[(j + 1) % n_ang], last[j], hub))
    bm.to_mesh(coat.data)
    bm.free()
    coat.data.update()
    # the sleeves, grown off the arms to the wrists
    sleeve_keep = ((bn == 'upperarm') & (along > 0.1)) | ((bn == 'forearm') & (along < 0.93))
    sleeves = A.garment(kit, o, 'sleeves', sleeve_keep, 0.008, sag=0.006, tears=0.0)
    A.loosen(kit, sleeves, posed, {'upperarm': (0.054, 0.05), 'forearm': (0.05, 0.043)})
    kit.smooth(sleeves, 3, 0.5)
    # the sleeve's head tucked in under the coat's shoulder (the loft takes
    # the shoulder to a third down the arm): no ring where it begins
    Sv = kit.to_game_np(kit.verts_np(sleeves))
    for side in ('L', 'R'):
        b = posed.bones[posed.ix['upperarm.' + side]]
        m = np.sign(Sv[:, 0]) == (1 if side == 'L' else -1)
        t, d, r = A.seg_param(Sv[m], b['head'], b['tail'])
        sink = 0.022 * kit.smoothstep(0.34, 0.12, t)
        Sv[m] = Sv[m] - r / np.maximum(d, 1e-6)[:, None] * np.minimum(sink, d - 0.01)[:, None]
    kit.set_verts_np(sleeves, kit.to_bl_np(Sv))
    # trousers (only under the hem is seen) and shoes
    # (no higher than a hand over the hem: higher, a knee came through the coat)
    trousers_keep = np.isin(bn, ['thigh', 'shin']) & ~((bn == 'shin') & (along >= 0.93)) & (P[:, 1] < HEM + 0.07)
    trousers = A.garment(kit, o, 'trousers', trousers_keep, 0.01, sag=0.012, tears=0.0)
    A.loosen(kit, trousers, posed, {'thigh': (0.08, 0.065), 'shin': (0.058, 0.05)})
    kit.smooth(trousers, 3, 0.5)
    shoes_keep = np.isin(bn, ['foot', 'toes']) | ((bn == 'shin') & (along > 0.9))
    shoes = A.garment(kit, o, 'shoes', shoes_keep, 0.008, sag=0.0, tears=0.0, smooth_edges=1)
    # short hair
    up_e = (P - eyes) @ crown
    front_e = (P - eyes) @ face_fwd
    scalp_keep = (bn == 'head') & (N @ crown > -0.6) & (
        ((front_e > -0.035) & (up_e > 0.055)) | ((front_e <= -0.035) & (up_e > 0.0)))
    hair = A.garment(kit, o, 'hair', scalp_keep, 0.004, sag=0.0, tears=0.0, smooth_edges=3)
    # fuller on top than at the sides and the nape, as cut hair lies
    Hp = kit.to_game_np(kit.verts_np(hair))
    Hn = kit.to_game_np(kit.vnormals(hair))
    full = 0.009 * kit.smoothstep(0.03, 0.11, (Hp - eyes) @ crown)
    kit.set_verts_np(hair, kit.to_bl_np(Hp + Hn * full[:, None]))
    kit.smooth(hair, 2, 0.5)
    # the skin seen: the head, the neck over the collar, the hands
    seen = np.isin(bn, ['head', 'jaw', 'neck', 'hand', 'finger']) | ((bn == 'forearm') & (along > 0.8))
    keep_faces(kit, o, seen)

    kit.decimate(o, 3200)
    kit.decimate(coat, 5200)
    kit.decimate(sleeves, 1800)
    kit.decimate(trousers, 1200)
    kit.decimate(shoes, 700)
    kit.decimate(hair, 700)
    # his shoes on the floor, not in it
    low = kit.to_game_np(kit.verts_np(shoes))[:, 1].min()
    lift = np.array([0.0, 0.002 - low, 0.0])
    for ob in (o, coat, sleeves, trousers, shoes, hair):
        kit.set_verts_np(ob, kit.to_bl_np(kit.to_game_np(kit.verts_np(ob)) + lift))
    for b in posed.bones:
        b['head'] = tuple(np.array(b['head']) + lift)
        b['tail'] = tuple(np.array(b['tail']) + lift)

    # ---- paint
    def plain(ob, base_hex, alt_hex, seed, k=0.35, cav_k=0.45, freq=8.0):
        C = kit.to_game_np(kit.verts_np(ob))
        cv = kit.cavity(ob)
        b, a = np.array(kit.srgb(base_hex)), np.array(kit.srgb(alt_hex))
        col = b + (a - b) * np.clip(kit.fbm(C, freq, 3, seed=seed) * 1.2 + 0.4, 0, 1)[:, None] * k
        return np.clip(col * (1 - cav_k * cv)[:, None], 0.03, 0.85)
    skin_col = plain(o, '#bba895', '#a2917f', 81, 0.5, 0.5)
    Hh = kit.to_game_np(kit.verts_np(hair))
    hair_col = np.array(kit.srgb('#2f2b28')) + (np.array(kit.srgb('#77726b')) - np.array(kit.srgb('#2f2b28'))) \
        * np.clip((np.abs(Hh[:, 0]) - 0.055) / 0.035, 0, 1)[:, None] * 0.6
    # the coat a grey work coat's faded cotton, a little cool (the place's
    # light is warm), paler than the concrete: lit by the place's light like
    # everything else, a dark coat was lost against its dark faces (the
    # figure's lightness against what is behind it, CIELAB L*, from 40 m to
    # 12 m: a coat of #3a3733 differed by 2-6, the old unlit grey toy by
    # 19-32; see docs/MEGASTRUCTURE.md, G7)
    COAT, COAT_ALT = '#979a9c', '#898c8e'
    Cc = kit.to_game_np(kit.verts_np(coat))
    Cn = kit.to_game_np(kit.vnormals(coat))
    coat_col = plain(coat, COAT, COAT_ALT, 82, 0.5, 0.35, freq=5.0)
    # the shoulders a shade paler (worn, and the dust settles there), the
    # hem a shade darker (the floors)
    coat_col = coat_col * (1 + 0.08 * kit.smoothstep(sh_y - 0.2, sh_y + 0.02, Cc[:, 1])
                           - 0.14 * kit.smoothstep(HEM + 0.25, HEM, Cc[:, 1]))[:, None]
    # underneath, the lining in its own shadow
    coat_col = coat_col * (1 - 0.75 * np.clip(-Cn[:, 1], 0, 1) * (Cc[:, 1] < HEM + 0.02))[:, None]
    sleeve_col = plain(sleeves, COAT, COAT_ALT, 83, 0.5, 0.35, freq=5.0)
    Sc = kit.to_game_np(kit.verts_np(sleeves))
    sleeve_col = sleeve_col * (1 + 0.08 * kit.smoothstep(sh_y - 0.2, sh_y + 0.02, Sc[:, 1]))[:, None]
    body = kit.Body('far_figure', posed)
    hips_i = posed.ix['hips']
    on = lambda i: (lambda Q: (np.tile([i, 0, 0, 0], (len(Q), 1)), np.tile([1.0, 0, 0, 0], (len(Q), 1))))
    body.skinned_by('skin', 'skin', o, on(hips_i), colors=skin_col)
    body.skinned_by('coat', 'cloth', coat, on(hips_i), colors=np.clip(coat_col, 0.03, 0.85))
    body.skinned_by('sleeves', 'cloth', sleeves, on(hips_i), colors=sleeve_col)
    body.skinned_by('trousers', 'cloth', trousers, on(hips_i), colors=plain(trousers, '#2c2d30', '#36373a', 84))
    body.skinned_by('shoes', 'shoe', shoes, on(hips_i), colors=plain(shoes, '#1d1b19', '#2a2725', 85, 0.3, 0.2))
    body.skinned_by('hair', 'hair', hair, on(hips_i), colors=np.clip(hair_col, 0.03, 0.85))
    return body
