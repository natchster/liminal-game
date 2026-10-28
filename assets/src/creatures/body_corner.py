"""
The one in the corner: someone tall standing in the far corner of a quiet
room, forehead to the angle of the walls, as a child is made to stand; in a
long flannel nightshirt, barefoot, its arms straight at its sides. You see
its back. Its face is a person's face with the eyes and the mouth turned
upside down (the Thatcher illusion), and you see it only once, as it turns.
(docs/ENTITIES.md, design 1.)

MakeHuman's old, thin, tall man (CC0), standing straight, its head a little
bowed (the forehead to the corner); each eye with its brow, and the mouth,
turned half round in the face, easing off at their edges; the nightshirt a
tube of cloth gathered at the neck and let fall over him by Blender's cloth
(`cloth.py`) over his shoulders and his arms to the shins.
"""
import numpy as np

ORDER = 40


def pose_of(skel):
    """standing straight, as it is told to: the arms at its sides, the head
    a little forward and down (its forehead to the corner)"""
    a = {'spine': (0, 1, 0.0), 'chest': (0, 1, 0.02), 'upperchest': (0, 1, 0.04), 'neck': (0, 1, 0.18), 'head': (0, 1, 0.12)}
    for side, sx in (('.L', 1), ('.R', -1)):
        a['upperarm' + side] = (sx * 0.07, -1, 0.0)
        a['forearm' + side] = (sx * 0.02, -1, 0.04)
        a['hand' + side] = (0, -1, 0.02)
        a['thigh' + side] = (sx * 0.03, -1, 0.0)
        a['shin' + side] = (0, -1, 0.0)
    return a


def smooth01(x):
    x = np.clip(x, 0.0, 1.0)
    return x * x * (3 - 2 * x)


def flip_upside_down(P, centre, rx, ry, edge):
    """the face's relief within an ellipse rx x ry round `centre` turned
    upside down: each point of the face there (base frame, +z ahead) takes
    the depth the face has at its mirror across the feature's middle line,
    easing back to its own over `edge` beyond; nothing moves across the
    face, so nothing twists (the Thatcher illusion is a flip)"""
    from mathutils import kdtree
    front = P[:, 2] > centre[2] - 0.035
    idx = np.nonzero(front)[0]
    tree = kdtree.KDTree(len(idx))
    for k, i in enumerate(idx):
        tree.insert((P[i, 0], P[i, 1], 0.0), k)
    tree.balance()
    d = P[:, :2] - centre[:2]
    r = np.hypot(d[:, 0] / rx, d[:, 1] / ry)
    w = (1.0 - smooth01((r - 1.0) / edge)) * front
    out = P.copy()
    for i in np.nonzero(w > 0.0)[0]:
        mx, my = P[i, 0], 2 * centre[1] - P[i, 1]
        near = tree.find_n((mx, my, 0.0), 4)
        ws = np.array([1.0 / max(n[2], 1e-4) for n in near])
        z = sum(P[idx[n[1]], 2] * wk for n, wk in zip(near, ws)) / ws.sum()
        out[i, 2] = P[i, 2] + w[i] * (z - P[i, 2])
    return out, w


def build(kit):
    import bpy
    import hm08, human, anatomy as A, cloth
    base = hm08.Base().target('macro/male-old-thin-tall')
    o, skel = human.body(kit, base, 'corner')
    eye_l, eye_l_f = base.helper('l-eye')
    eye_r, eye_r_f = base.helper('r-eye')
    teeth_u, teeth_u_f = base.helper('upper-teeth')
    teeth_l, teeth_l_f = base.helper('lower-teeth')
    # ---- the Thatcher face, in the base's own frame (before posing): each
    # eye and its brow, and the mouth, turned half round where they are
    P = kit.to_game_np(kit.verts_np(o))
    cl, cr = eye_l.mean(0), eye_r.mean(0)
    lift = np.zeros(3)                      # (flipped about the eye's own middle: the brow comes out under it)
    mouth = np.array([0.0, (teeth_u[:, 1].min() + teeth_l[:, 1].max()) / 2, teeth_u[:, 2].max() + 0.01])
    src = P.copy()
    # the brows (a little dark) and the lips (a little red), so the flip
    # reads: the brows come out under the eyes, the mouth's curve the wrong way
    front = src[:, 2] > cl[2] - 0.03
    brow = np.zeros(len(src))
    for c in (cl, cr):
        dx, dy = (src[:, 0] - c[0]) / 0.021, (src[:, 1] - (c[1] + 0.017)) / 0.006
        brow = np.maximum(brow, (1.0 - smooth01(np.hypot(dx, dy) - 0.6)) * front)
    dxm, dym = (src[:, 0] - mouth[0]) / 0.026, (src[:, 1] - mouth[1]) / 0.009
    lip = (1.0 - smooth01(np.hypot(dxm, dym) - 0.7)) * front
    def flip_mask(m, c, rx, ry, edge):
        from mathutils import kdtree
        idx = np.nonzero(front)[0]
        t = kdtree.KDTree(len(idx))
        for k, i in enumerate(idx):
            t.insert((src[i, 0], src[i, 1], 0.0), k)
        t.balance()
        r = np.hypot((src[:, 0] - c[0]) / rx, (src[:, 1] - c[1]) / ry)
        w = (1.0 - smooth01((r - 1.0) / edge)) * front
        out = m.copy()
        for i in np.nonzero(w > 0)[0]:
            j = idx[t.find((src[i, 0], 2 * c[1] - src[i, 1], 0.0))[1]]
            out[i] = m[i] + w[i] * (m[j] - m[i])
        return out
    for c in (cl + lift, cr + lift):
        brow = flip_mask(brow, c, 0.026, 0.022, 0.5)
    lip = flip_mask(lip, mouth, 0.034, 0.02, 0.6)
    for c in (cl + lift, cr + lift):
        P, _ = flip_upside_down(P, c, 0.026, 0.022, 0.5)
    # (each feature's flip reads the face as it was, not as it is being flipped)
    Pm, _ = flip_upside_down(src, mouth, 0.034, 0.02, 0.6)
    mouth_w = np.hypot((src[:, 0] - mouth[0]) / 0.034, (src[:, 1] - mouth[1]) / 0.02) < 1.6
    P[mouth_w, 2] = Pm[mouth_w, 2]
    kit.set_verts_np(o, kit.to_bl_np(P))
    # the eyes themselves are round; the teeth turned with the mouth
    def flip_helper(V, c):
        out = V.copy()
        out[:, 1] = 2 * c[1] - V[:, 1]
        return out
    teeth_u, teeth_l = flip_helper(teeth_u, mouth), flip_helper(teeth_l, mouth)

    posed = human.pose(kit, o, skel, pose_of(skel), None, None, smooth_weights=6)

    def frame(bone):
        b0, b1 = np.array(skel.head(bone)), np.array(posed.head(bone))
        R = human._rot_between(np.array(skel.tail(bone)) - b0, np.array(posed.tail(bone)) - b1)
        return lambda X: b1 + (X - b0) @ R.T
    to_head = frame('head')
    eye_l, eye_r, teeth_u = to_head(eye_l), to_head(eye_r), to_head(teeth_u)
    teeth_l = frame('jaw')(teeth_l)
    # on the floor
    Pw = kit.to_game_np(kit.verts_np(o))
    lift_f = -Pw[:, 1].min()
    kit.set_verts_np(o, kit.to_bl_np(Pw + np.array([0, lift_f, 0])))
    for b in posed.bones:
        b['head'] = (b['head'][0], b['head'][1] + lift_f, b['head'][2])
        b['tail'] = (b['tail'][0], b['tail'][1] + lift_f, b['tail'][2])
    up = np.array([0, lift_f, 0])
    eye_l, eye_r, teeth_u, teeth_l = eye_l + up, eye_r + up, teeth_u + up, teeth_l + up
    before = kit.to_game_np(kit.verts_np(o))
    marks = np.stack([brow, lip], axis=1)
    kit.remesh(o, 0.005)
    kit.smooth(o, 2, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)
    bn = np.array([posed.bones[i]['name'].split('.')[0] if i >= 0 else '' for i in owner])
    # thin, not starved: a little off the limbs
    P = kit.to_game_np(kit.verts_np(o))
    waste = {b['name']: 0.12 for b in posed.bones if b['name'].split('.')[0] in ('upperarm', 'forearm', 'thigh', 'shin')}
    W = A.waste(P, posed, list(waste), waste, joint_keep=0.06, owner=owner,
                buried=[n for n in waste if n.split('.')[0] in ('thigh', 'upperarm')])
    P = A.relax_offsets(kit, o, P, W, 15)
    kit.set_verts_np(o, kit.to_bl_np(P))
    kit.decimate(o, 12000)
    # whose each point is, again (the decimation made new points), and its marks
    owner = A.carry_values(kit, before, posed.owner, o)
    marks_now = A.carry_values(kit, before, marks, o)
    bn = np.array([posed.bones[i]['name'].split('.')[0] if i >= 0 else '' for i in owner])

    # ---- pale, waxy skin; a little colour at the joints and the soles
    P = kit.to_game_np(kit.verts_np(o))
    cav = kit.cavity(o)
    pale = np.array(kit.srgb('#cfc4b8'))
    blue = np.array(kit.srgb('#a9a7ac'))
    col = pale + (blue - pale) * np.clip(0.5 + kit.fbm(P, 9.0, 3, seed=81), 0, 1)[:, None] * 0.35
    col = np.clip(col * (1 - 0.4 * cav)[:, None], 0.03, 0.95)
    browc, lipc = np.array(kit.srgb('#3a312b')), np.array(kit.srgb('#8c5d57'))
    col = col + (browc - col) * np.clip(marks_now[:, 0] * 1.3, 0, 1)[:, None]
    col = col + (lipc - col) * np.clip(marks_now[:, 1] * 0.75, 0, 1)[:, None]

    # ---- the nightshirt, long, a tube of flannel gathered at the neck and
    # let fall over him (shoulders, arms at his sides, to the shins) by
    # Blender's cloth: his arms are in it, their shape in the cloth
    cream = np.array(kit.srgb('#bdb6a3'))
    stripe = np.array(kit.srgb('#8f98a3'))
    nk = np.array(posed.head('neck'))
    hp = np.array(posed.head('hips'))
    knee = np.array(posed.tail('thigh.L'))
    top_y, bottom_y = nk[1] - 0.01, knee[1] - 0.26
    # wide enough at the hands not to catch on them
    hands = P[np.isin(bn, ['hand'])]
    reach = max(0.26, (np.abs(hands[:, 0] - hp[0]).max() if len(hands) else 0.24) + 0.07)
    n_round, n_down = 72, 64
    ys = np.linspace(top_y, bottom_y, n_down)
    V, F = [], []
    # it starts outside him at every height (the solver cannot free cloth
    # begun inside the body): his widest there, and a little more
    def girth(y, axis):
        band = P[np.abs(P[:, 1] - y) < 0.03]
        if not len(band):
            return 0.1
        return np.abs(band[:, axis] - (nk[0] if axis == 0 else hp[2])).max() + 0.035
    for y in ys:
        t = (top_y - y) / (top_y - bottom_y)
        rx = max(girth(y, 0), 0.085 + (reach - 0.085) * smooth01(t / 0.16))
        rz = max(girth(y, 2), 0.78 * rx)
        for i in range(n_round):
            a_ = 2 * np.pi * i / n_round
            V.append((nk[0] + rx * np.sin(a_), y, hp[2] + rz * np.cos(a_)))
    for j in range(n_down - 1):
        for i in range(n_round):
            a0, a1 = j * n_round + i, j * n_round + (i + 1) % n_round
            F.append((a0, a1, a1 + n_round, a0 + n_round))
    me = bpy.data.meshes.new('shirt')
    me.from_pydata([tuple(p) for p in kit.to_bl_np(np.array(V))], [], F)
    me.update()
    shirt = kit.link(bpy.data.objects.new('shirt', me))
    cloth.collider(o, 0.012, 6.0)
    fl = cloth.floor(kit)
    cloth.pin(shirt, range(n_round))                          # the neckline
    S = cloth.settle(kit, shirt, 70, mass=0.3, stiffness=15.0, bending=0.8)
    bpy.data.objects.remove(fl)
    o.modifiers.clear()
    tree = cloth.nearest(P)
    S, _ = cloth.off_the_skin(S, tree, kit.to_game_np(kit.vnormals(o)), gap=0.014, reach=0.12)
    kit.set_verts_np(shirt, kit.to_bl_np(S))
    shirt.vertex_groups.clear()
    for p in shirt.data.polygons:
        p.use_smooth = True
    # faded cream flannel, pale blue stripes down it, greyer toward the hem
    around = np.tile(np.arange(n_round) / n_round, n_down)
    down = np.repeat(np.linspace(0, 1, n_down), n_round)
    st = (np.sin(around * 2 * np.pi * 18) > 0.55).astype(float)
    shirt_col = cream + (stripe - cream) * st[:, None] * 0.55
    shirt_col = shirt_col * (1 - 0.22 * smooth01((down - 0.7) / 0.3))[:, None]

    # ---- eyes and teeth on their bones
    def mesh(name, Vv, Ff):
        me = bpy.data.meshes.new(name)
        me.from_pydata([tuple(p) for p in kit.to_bl_np(Vv)], [], Ff)
        me.update()
        for p in me.polygons:
            p.use_smooth = True
        return kit.link(bpy.data.objects.new(name, me))
    el, er = mesh('eye_l', eye_l, eye_l_f), mesh('eye_r', eye_r, eye_r_f)
    tu, tl = mesh('teeth_u', teeth_u, teeth_u_f), mesh('teeth_l', teeth_l, teeth_l_f)
    def eye_col(ob):
        # the white, and a dark iris and pupil on its front, looking ahead
        E = kit.to_game_np(kit.verts_np(ob))
        c = E.mean(0)
        r = np.linalg.norm(E - c, axis=1).max()
        ahead = np.array(posed.tail('head')) - np.array(posed.head('head'))
        fwd = np.cross(ahead, [1.0, 0, 0])
        fwd = -fwd / np.linalg.norm(fwd)
        k = ((E - c) @ fwd) / r
        white, iris, pupil = (np.array(kit.srgb(h)) for h in ('#d8d2c6', '#5b5a50', '#1f1b18'))
        col = np.tile(white, (len(E), 1))
        col[k > 0.72] = iris
        col[k > 0.9] = pupil
        return col
    tooth_col = lambda ob: np.tile(np.array(kit.srgb('#bfb59a')), (len(ob.data.vertices), 1))

    body = kit.Body('corner', posed)
    body.skinned('skin', 'skin', o, colors=col)
    body.skinned('shirt', 'cloth', shirt, colors=shirt_col)
    body.rigid('eye.L', 'eye', 'head', el, colors=eye_col(el))
    body.rigid('eye.R', 'eye', 'head', er, colors=eye_col(er))
    body.rigid('teeth.upper', 'teeth', 'head', tu, colors=tooth_col(tu))
    body.rigid('teeth.lower', 'teeth', 'jaw', tl, colors=tooth_col(tl))
    return body
