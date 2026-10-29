"""
The Watcher: someone standing under an old dust sheet, a head over you and
stooping a little, the cloth close about him to the floor and darker at the
hem, a face pressed into it (the brow, the nose; the eyes and the open mouth
grimed dark in the cloth). At a glance it
is furniture under a sheet. Each time you look back it stands differently.
(docs/ENTITIES.md, design 1.)

Under the sheet is MakeHuman's old thin man (CC0, a tall one: 1.96 m), never
shown. The sheet is one cloth grid draped over him by Blender's cloth solver
(`cloth.py`): laid on the crown, let fall to the floor, drawn in round him
below the shoulders (`hug`: it hangs as a narrow shroud, not a cone), then
pressed onto the face and settled round it; where his eyes and his open
mouth are under it the cloth is grimed dark. Each pose is the same grid
draped again over him standing another way, kept as a morph target of the
first, read off its drape where its face fell on the cloth (so the face is
the same part of the sheet in every pose); the game shows a pose only after
you have looked away (`entities::watcher`). The sheet is one layer, drawn
from both sides.
"""
import numpy as np

ORDER = 30

# the poses, the first the rest: (name, the bones' aims over the standing stoop)
POSES = ('stand', 'listen', 'bowed', 'reach', 'lean', 'over')
SHEET = (3.4, 3.6, 86, 91)          # width, length (m); vertices across, along (4 cm)
HUG = 0.3                           # what is left of the cloth's flare below the shoulders
REFINE = 4                          # the drape made 4 times finer (1 cm) for the face
TRIS = 14000                        # the sheet in the game


def aims_of(pose):
    # (its jaw hangs open under the cloth; its head bowed a little, so
    # that it looks down at you)
    a = {'spine': (0, 1, 0.05), 'chest': (0, 1, 0.12), 'upperchest': (0, 1, 0.2), 'neck': (0, 1, 0.3), 'head': (0, 1, 0.4),
         'jaw': (0, -1, 0.35)}
    for side, sx in (('.L', 1), ('.R', -1)):
        a['upperarm' + side] = (sx * 0.15, -1, 0.05)
        a['forearm' + side] = (sx * 0.05, -1, 0.15)
    if pose == 'listen':
        # the head over on one shoulder, as if it listened
        a.update({'neck': (0.12, 1, 0.3), 'head': (0.5, 1, 0.05), 'upperarm.L': (0.25, -1, 0.1)})
    elif pose == 'bowed':
        # the head hung, the back rounded
        a.update({'spine': (0, 1, 0.2), 'chest': (0, 1, 0.4), 'upperchest': (0, 1, 0.55), 'neck': (0, 0.8, 1), 'head': (0, 0.2, 1)})
    elif pose == 'reach':
        # one arm up and forward under the cloth, toward you
        a.update({'upperarm.R': (-0.15, -0.15, 1), 'forearm.R': (-0.05, 0.1, 1), 'neck': (0, 1, 0.4), 'head': (0, 1, 0.25)})
    elif pose == 'lean':
        # leaning to one side, the head the other way
        a.update({'spine': (0.18, 1, 0.05), 'chest': (0.15, 1, 0.12), 'neck': (-0.2, 1, 0.3), 'head': (-0.3, 1, 0.1),
                  'upperarm.R': (-0.3, -1, 0.05)})
    elif pose == 'over':
        # bent over you where you fell
        a.update({'spine': (0, 1, 0.45), 'chest': (0, 0.8, 1), 'upperchest': (0, 0.6, 1), 'neck': (0, 0.3, 1), 'head': (0, -0.35, 1)})
        for side, sx in (('.L', 1), ('.R', -1)):
            a['upperarm' + side] = (sx * 0.2, -0.7, 1)
            a['forearm' + side] = (sx * 0.1, -0.9, 0.5)
    return a


def hug(S, P, N, shoulders):
    """the cloth below the shoulders drawn in round him (points S over a
    body of points P, normals N): at each height, what stands out past his
    widest there keeps only HUG of it. A sheet laid on a man tents from the
    shoulders into a cone; drawn in, it hangs as a narrow shroud, a person's
    shape. (As a cone, a blind review read it as a bedsheet ghost, E10)"""
    import cloth
    BIN = 0.05
    nb = int(P[:, 1].max() / BIN) + 2
    b = np.clip((P[:, 1] / BIN).astype(int), 0, nb - 1)
    centre, reach = np.zeros((nb, 2)), np.zeros(nb)
    for k in range(nb):
        Q = P[b == k][:, [0, 2]]
        if len(Q) == 0:
            centre[k], reach[k] = centre[k - 1], reach[k - 1]
            continue
        centre[k] = Q.mean(0)
        reach[k] = np.linalg.norm(Q - centre[k], axis=1).max() + 0.04
    # cloth falls from the widest of him above, straight down (drawn in to
    # each height's own width it had wrapped his legs like trousers)
    for k in range(nb - 2, -1, -1):
        reach[k] = max(reach[k], reach[k + 1] if P[b == k + 1, 1].size and k + 1 <= int(shoulders / BIN) else 0.0)
    S = S.copy()
    k = np.clip((S[:, 1] / BIN).astype(int), 0, nb - 1)
    off = S[:, [0, 2]] - centre[k]
    d = np.linalg.norm(off, axis=1) + 1e-9
    x = np.clip((shoulders - S[:, 1]) / 0.3, 0.0, 1.0)
    w = x * x * (3 - 2 * x)
    want = np.where(d > reach[k], reach[k] + (d - reach[k]) * HUG, d)
    nd = d + (want - d) * w
    S[:, [0, 2]] = centre[k] + off * (nd / d)[:, None]
    S, _ = cloth.off_the_skin(S, cloth.nearest(P), N, gap=0.006, reach=0.15)
    return S


def drape(kit, base, pose):
    """the sheet draped over him in `pose`, made fine and its face pressed
    in: (points on the fine grid, its size across and along, the face's
    share at each point)"""
    import bpy
    import human, cloth
    kit.clear_scene()
    o, skel = human.body(kit, base, 'under')
    posed = human.pose(kit, o, skel, aims_of(pose), None, None, smooth_weights=8)
    P = kit.to_game_np(kit.verts_np(o))
    # on the floor
    lift = -P[:, 1].min()
    P[:, 1] += lift
    kit.set_verts_np(o, kit.to_bl_np(P))
    N = kit.to_game_np(kit.vnormals(o))
    # the head's own frame (its face looks along `fwd`, `up` its crown)
    hc = np.array(posed.head('head')) + [0, lift, 0]
    R = human._rot_between(np.array(skel.tail('head')) - np.array(skel.head('head')),
                           np.array(posed.tail('head')) - np.array(posed.head('head')))
    fwd, up = R @ np.array([0.0, 0.0, 1.0]), R @ np.array([0.0, 1.0, 0.0])
    cloth.collider(o)
    fl = cloth.floor(kit)
    # laid on the highest point of him (the crown; bent over, the back)
    top = P[np.argmax(P[:, 1])]
    w, l, nx, ny = SHEET
    sh, _ = cloth.grid(kit, 'sheet', w, l, nx, ny, (top[0], top[1] + 0.012, top[2]), dense=1.0)
    G = kit.to_game_np(kit.verts_np(sh))
    cloth.pin(sh, np.nonzero(np.hypot(G[:, 0] - top[0], G[:, 2] - top[2]) < 0.05)[0])
    S = cloth.settle(kit, sh, 90)
    bpy.data.objects.remove(fl)
    S = hug(S, P, N, top[1] - 0.3)
    # made fine (the smooth surface through the solver's points)
    F, fx, fz = cloth.refine(S, nx, ny, REFINE)
    # the face pressed in: a soft shrink-wrap, the share of each point that
    # goes onto the face (3 mm off) falling off smoothly at the face's edge
    def soft(x):
        x = np.clip(x, 0.0, 1.0)
        return x * x * (3 - 2 * x)
    tree = cloth.nearest(P)
    mask = np.zeros(len(F))
    near = np.nonzero(np.linalg.norm(F - (hc + up * 0.07), axis=1) < 0.32)[0]

    def press(F, first):
        for i in near:
            q, j, d = tree.find(F[i])
            if first:
                rel = np.array(q) - hc
                h = rel @ up
                # (from the chin, below the head's root, to the brow)
                mask[i] = soft((h + 0.035) / 0.025) * soft((0.135 - h) / 0.02) * soft((N[j] @ fwd - 0.25) / 0.35) \
                    * soft((0.13 - d) / 0.05) * (rel @ fwd > 0.0)
            if mask[i] > 0.0:
                F[i] = F[i] + mask[i] * (np.array(q) + N[j] * 0.003 - F[i])
        return F
    # pressed on, then spread evenly over the face and pressed again, a few
    # times over: a face is smaller than the cloth laid on it, and pressed
    # once the spare cloth bunches into creases
    def relax(F, k):
        G = F.reshape(fz, fx, 3)
        A = G.copy()
        A[1:-1, 1:-1] = (G[1:-1, :-2] + G[1:-1, 2:] + G[:-2, 1:-1] + G[2:, 1:-1]) / 4
        return F + (A.reshape(-1, 3) - F) * np.clip(mask * 1.5, 0, 1)[:, None] * k
    F = press(F, True)
    for _ in range(8):
        F = press(relax(F, 0.9), False)
    # a last relaxing, not pressed again: the creases soften to folds
    F = relax(relax(F, 0.5), 0.5)
    # and nothing of it under his skin
    sub = F[near]
    sub, moved = cloth.off_the_skin(sub, tree, N, gap=0.003, reach=0.06)
    F[near] = sub
    print(f'[watcher] {pose}: {int((mask > 0.5).sum())} fine points on the face, {moved} put back out of the skin')
    # where its face is in this pose (for the eyes that look at it: the game's
    # camera over your body), just in front of the cloth
    face_at = hc + up * 0.07 + fwd * 0.11
    # where its eyes and its open mouth are under the cloth (MakeHuman's
    # eyes and teeth, carried by the head and the jaw): the cloth there is
    # grimed dark, soft-edged, as a face shows through a sheet worn over it
    def carried(bone, X):
        b0, b1 = np.array(skel.head(bone)), np.array(posed.head(bone)) + [0, lift, 0]
        Rb = human._rot_between(np.array(skel.tail(bone)) - b0, np.array(posed.tail(bone)) - np.array(posed.head(bone)))
        return b1 + (np.asarray(X) - b0) @ Rb.T
    eyes = [carried('head', base.helper(e)[0].mean(0)) for e in ('l-eye', 'r-eye')]
    tu, tl = base.helper('upper-teeth')[0], base.helper('lower-teeth')[0]
    lip_u = carried('head', tu[np.abs(tu[:, 0]) < 0.004].mean(0))
    lip_l = carried('jaw', tl[np.abs(tl[:, 0]) < 0.004].mean(0))
    X = F[near]
    ab = lip_l - lip_u
    t = np.clip(((X - lip_u) @ ab) / max(ab @ ab, 1e-12), 0, 1)
    m = np.exp(-(np.linalg.norm(X - (lip_u + t[:, None] * ab), axis=1) / 0.026) ** 2)
    for e in eyes:
        m = np.maximum(m, np.exp(-(np.linalg.norm(X - e, axis=1) / 0.032) ** 2))
    marks = np.zeros(len(F))
    marks[near] = m * soft(mask[near] / 0.3)
    return F, fx, fz, mask, face_at, marks


def build(kit):
    import bpy
    import hm08, cloth
    # (a tall man: under the cloth he stands a head over you)
    base = hm08.Base().target('macro/male-old-thin', 0.6).target('macro/male-old-thin-tall', 0.4)
    fine, masks, faces, depths = {}, [], {}, {}
    for pose in POSES:
        F, fx, fz, m, face_at, mk = drape(kit, base, pose)
        fine[pose] = F
        masks.append(m)
        faces[pose] = face_at
        depths[pose] = mk
    w, l, nx, ny = SHEET
    fxs, fzs = np.linspace(-w / 2, w / 2, fx), np.linspace(-l / 2, l / 2, fz)
    # each pose's face falls on its own place of the cloth (bent over you,
    # it is 20 cm from where it is standing): each pose is read off its
    # drape that far along, so its face is where the sheet's is kept fine
    # and stained (read in the same place, the face bent over you had been
    # coarse folds, E10). What is read past the edge is the hem's last row.
    def cell(m):
        idx = np.nonzero(m > 0.5)[0]
        return np.array([np.median(idx % fx), np.median(idx // fx)])
    c0 = cell(masks[0])
    shift, aligned, deep_at = {}, [], []
    for pose, m in zip(POSES, masks):
        dx, dz = np.round(cell(m) - c0).astype(int)
        shift[pose] = np.array([dx * w / (fx - 1), dz * l / (fz - 1)])
        back = lambda a: np.roll(a.reshape(fz, fx), (-dz, -dx), axis=(0, 1)).reshape(-1)
        aligned.append(back(m))
        deep_at.append(back(depths[pose]))
        print(f'[creatures] watcher {pose}: its face read {dx},{dz} cm along; its marks up to {depths[pose].max():.2f}; '
              f'then shares {np.minimum(aligned[-1], masks[0]).sum() / max(m.sum(), 1e-9):.2f} of the standing face')
    masks = aligned
    # ---- the sheet in the game: the fine drape of the first pose, thinned
    # (its face kept fine), every pose then read off its own fine drape at
    # the same places of the cloth
    kit.clear_scene()
    V = fine[POSES[0]]
    uvf = np.array([(x, z) for z in fzs for x in fxs])
    Fq = [(j * fx + i, (j + 1) * fx + i, (j + 1) * fx + i + 1, j * fx + i + 1) for j in range(fz - 1) for i in range(fx - 1)]
    me = bpy.data.meshes.new('watcher_sheet')
    me.from_pydata([tuple(p) for p in kit.to_bl_np(V)], [], Fq)
    me.update()
    sh = kit.link(bpy.data.objects.new('watcher_sheet', me))
    lay = me.uv_layers.new(name='cloth')
    loops = np.empty(len(me.loops), dtype=np.int64)
    me.loops.foreach_get('vertex_index', loops)
    lay.data.foreach_set('uv', uvf[loops].reshape(-1))
    keep = sh.vertex_groups.new(name='face')
    face = np.max(masks, axis=0)
    for i in np.nonzero(face > 0.05)[0]:
        keep.add([int(i)], float(min(1.0, face[i] * 1.5)), 'REPLACE')
    bpy.ops.object.select_all(action='DESELECT')
    sh.select_set(True)
    bpy.context.view_layer.objects.active = sh
    dec = sh.modifiers.new('thin', 'DECIMATE')
    dec.ratio = TRIS / (2.0 * len(Fq))
    dec.vertex_group = 'face'
    dec.invert_vertex_group = True
    dec.vertex_group_factor = 8.0
    bpy.ops.object.modifier_apply(modifier='thin')
    kit.triangulate(sh)
    me = sh.data
    # each vertex's place on the cloth (from its loops)
    lu = np.empty(len(me.loops) * 2)
    me.uv_layers['cloth'].data.foreach_get('uv', lu)
    lv = np.empty(len(me.loops), dtype=np.int64)
    me.loops.foreach_get('vertex_index', lv)
    uv = np.zeros((len(me.vertices), 2))
    uv[lv] = lu.reshape(-1, 2)

    def normals_of(S):
        kit.set_verts_np(sh, kit.to_bl_np(S))
        sh.data.update()
        return kit.to_game_np(kit.vnormals(sh))
    shapes = {p: cloth.sample(fine[p], fx, fz, fxs, fzs, uv + shift[p]) for p in POSES}
    S0 = shapes[POSES[0]]
    N0 = normals_of(S0)
    morphs = []
    for pose in POSES[1:]:
        Nk = normals_of(shapes[pose])
        morphs.append((pose, shapes[pose] - S0, Nk - N0))
    kit.set_verts_np(sh, kit.to_bl_np(S0))
    sh.data.update()
    for p in sh.data.polygons:
        p.use_smooth = True
    # ---- an old linen dust sheet (`linen.py`, the furniture's too)
    import linen
    col = linen.colours(kit, sh, uv, S0[:, 1])
    # the cloth darker where it is pressed deep into the face (the hollows
    # of the eyes, the open mouth): grime and breath in the weave. (Pale
    # all over, its face did not show: a blind review read it as a bedsheet
    # ghost, E10)
    # (the standing face's marks: every pose's together were blots)
    mk = cloth.sample(np.repeat(deep_at[0][:, None], 3, axis=1), fx, fz, fxs, fzs, uv)[:, 0]
    col = col * (1 - 0.8 * np.clip(mk * 1.6, 0, 1))[:, None]

    # one bone: it never moves a limb where you can see; and two marks that
    # move nothing, where its face is standing and bent over you (the
    # game's camera looks there)
    skel = kit.Skel()
    skel.add('root', None, (0, 0, 0), (0, 1.0, 0))
    for name, pose in (('head', 'stand'), ('face.over', 'over')):
        f = faces[pose]
        skel.add(name, 'root', tuple(f), tuple(f + np.array([0, 0, 0.1])))
    body = kit.Body('watcher', skel)
    one = lambda P: (np.zeros((len(P), 4), dtype=np.int64), np.tile([1.0, 0, 0, 0], (len(P), 1)))
    body.skinned_by('sheet', 'sheet', sh, one, colors=col, morphs=morphs, uv=uv)
    return body
