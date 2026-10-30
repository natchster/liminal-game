"""
The one behind the glass (docs/WINDOWS.md 4): someone standing far out on
the track in a pale raincoat, belted, the hood up and drawn, wet; an
ordinary height, the shoulders a little narrow; its face in the hood's
shadow, and close, pressed to the glass, a person's face, wet and pale,
flattened white where it touches.

MakeHuman's young thin man and young thin woman (CC0), half each, standing
straight, its arms hanging a little out from its sides, its head a little
forward. A raincoat of rubberised cotton:
- its body and sleeves grown off the skin (`anatomy.garment`), the trunk
  smoothed hard, spanned across its hollows and let fall straight from the
  chest (`hang`): stiff cloth keeps none of the body's small shapes;
- its skirt a tube of cloth from the belt to below the knees, taut over the
  seat and let fall from there by Blender's cloth (stiff, a little flared);
- a belt with a buckle over the join;
- a hood that is a shape of its own, not grown: a shell a few centimetres
  off the skull, peaked a little at the back, hanging from its widest round
  to the shoulders, an oval cut for the face drawn in round it and standing
  forward of it, its inside dark;
- trousers under it and rubber boots.
Its face cold and drowned, its eyes clouded, on bones of their own. A morph,
`press`, flattens the front of its face as it would be against glass.

The creature it is the body of is after SCP-2885 (Pythy, rewritten by Zyn),
SCP-3052 (Witryso), SCP-L682 (CRAt26) and the Backrooms wiki's Entity 2
(u/glassydude101, rewritten by DrAkimoto), CC BY-SA 3.0 (`docs/LORE.md`,
Attribution); its body is MakeHuman's (CC0 1.0).
"""
import numpy as np

ORDER = 60


def pose_of(skel):
    """standing straight, its arms at its sides, its head a little forward
    and down (it looks at you from under its hood)"""
    a = {'spine': (0, 1, 0.0), 'chest': (0, 1, 0.01), 'upperchest': (0, 1, 0.03), 'neck': (0, 1, 0.22), 'head': (0, 1, 0.18)}
    for side, sx in (('.L', 1), ('.R', -1)):
        a['upperarm' + side] = (sx * 0.16, -1, 0.02)
        a['forearm' + side] = (sx * 0.08, -1, 0.05)
        a['hand' + side] = (0, -1, 0.04)
        a['thigh' + side] = (sx * 0.035, -1, 0.0)
        a['shin' + side] = (0, -1, 0.0)
    return a


def crown_dir(posed):
    c = np.array(posed.tail('head')) - np.array(posed.head('head'))
    return c / np.linalg.norm(c)


def skin_weights(skin, k=6, only=None, skel=None):
    """weights for a garment from the skin's (`skin`: the body's skinned
    mesh as written): each point the blend of its `k` nearest skin points'
    bones, by inverse distance, the four strongest kept; with `only` (bone
    name prefixes, `skel` the posed skeleton), no other bone moves it"""
    import cloth
    SP = np.array(skin['pos'], float).reshape(-1, 3)
    SJ = np.array(skin['jnt'], int).reshape(-1, 4)
    SW = np.array(skin['wgt'], float).reshape(-1, 4) / 255.0
    tree = cloth.nearest(SP)
    allowed = None
    if only is not None:
        allowed = {i for i, b in enumerate(skel.bones) if b['name'].split('.')[0] in only}
        fallback = next(i for i, b in enumerate(skel.bones) if b['name'] == only[0])

    def weights(P):
        J = np.zeros((len(P), 4), int)
        W = np.zeros((len(P), 4))
        for i, p in enumerate(P):
            acc = {}
            for _, j, d in tree.find_n(p, k):
                f = 1.0 / (d + 0.004)
                for b, w in zip(SJ[j], SW[j]):
                    if w > 0 and (allowed is None or int(b) in allowed):
                        acc[int(b)] = acc.get(int(b), 0.0) + w * f
            if not acc:
                acc = {fallback: 1.0}
            top = sorted(acc.items(), key=lambda t: -t[1])[:4]
            tot = sum(w for _, w in top)
            for c, (b, w) in enumerate(top):
                J[i, c], W[i, c] = b, w / tot
        return J, W
    return weights


def smooth_where(kit, o, mask, iters, factor):
    """Laplacian smoothing of the points of `o` in `mask`, the rest held
    (and never in under the body after: see the push below)"""
    E = np.zeros(len(o.data.edges) * 2, dtype=np.int64)
    o.data.edges.foreach_get('vertices', E)
    E = E.reshape(-1, 2)
    S = kit.to_game_np(kit.verts_np(o))
    deg = np.maximum(np.bincount(E.ravel(), minlength=len(S)), 1).astype(float)[:, None]
    w = mask.astype(float)[:, None] * factor
    for _ in range(iters):
        acc = np.zeros_like(S)
        np.add.at(acc, E[:, 0], S[E[:, 1]])
        np.add.at(acc, E[:, 1], S[E[:, 0]])
        S = S + (acc / deg - S) * w
    kit.set_verts_np(o, kit.to_bl_np(S))


def make_hood(kit, bpy, o, coat, posed, bn, fc, face_fwd, neck):
    import bmesh, cloth
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    hd = P[bn == 'head']
    c = (hd.min(0) + hd.max(0)) / 2
    up = np.array([0.0, 1.0, 0.0])
    f = face_fwd - up * (face_fwd @ up)
    f /= np.linalg.norm(f)
    sd = np.cross(up, f)
    near = P[np.isin(bn, ['head', 'neck'])]
    rel = near - c
    dist = np.linalg.norm(rel, axis=1)
    dirs = rel / dist[:, None]
    y_col = neck[1] - 0.045
    n_a, n_tube, n_dome = 72, 14, 18
    az = 2 * np.pi * np.arange(n_a) / n_a

    def horiz(a):
        return np.cos(a) * f + np.sin(a) * sd

    def star(d):
        """the skin's furthest along d (within a cone), and a little more"""
        cosang = dirs @ d
        m = cosang > np.cos(np.radians(14))
        return (dist[m] * cosang[m]).max() if m.any() else 0.10
    # the dome
    V = []
    R0 = np.array([star(horiz(a)) for a in az])
    R0 = np.maximum(R0, 0.085)
    rows = []
    for j in range(n_dome):
        e = np.radians(88.0) * j / (n_dome - 1)
        ring = []
        for i, a in enumerate(az):
            d = np.cos(e) * horiz(a) + np.sin(e) * up
            r = max(star(d), 0.085) + 0.026 + 0.012 * np.cos(e)
            back = max(0.0, -np.cos(a))
            r += 0.028 * back ** 1.5 * np.sin(2 * e) ** 2   # its peak, on the back of the head
            ring.append(c + d * r)
        rows.append(ring)
    # the hang below, to the collar
    tube = []
    for j in range(1, n_tube + 1):
        t = j / n_tube
        y = c[1] + (y_col - c[1]) * t
        ring = []
        for i, a in enumerate(az):
            front = max(0.0, np.cos(a)) ** 2
            hang = (R0[i] + 0.038) * (1 + 0.32 * t ** 1.6)
            # under the chin it is drawn in toward the throat
            band = P[(np.abs(P[:, 1] - y) < 0.02) & np.isin(bn, ['neck', 'head', 'upperchest', 'clavicle'])]
            hv = band - c
            proj = hv @ horiz(a)
            lat = np.abs(hv @ (np.cross(up, horiz(a))))
            sk = proj[lat < 0.03].max() if (lat < 0.03).any() else 0.05
            r = (1 - front * t) * hang + front * t * max(sk + 0.035, 0.78 * hang)
            ring.append(np.array([c[0], y, c[2]]) + horiz(a) * r)
        tube.append(ring)
    grid = list(reversed(tube)) + rows
    for ring in grid:
        V.extend(ring)
    V.append(c + up * ((rows[-1][0] - c) @ up))
    F = []
    nr = len(grid)
    for j in range(nr - 1):
        for i in range(n_a):
            a0, a1 = j * n_a + i, j * n_a + (i + 1) % n_a
            F.append((a0, a1, a1 + n_a, a0 + n_a))
    pole = len(V) - 1
    for i in range(n_a):
        F.append(((nr - 1) * n_a + i, (nr - 1) * n_a + (i + 1) % n_a, pole))
    V = np.array(V)
    # the face's oval
    def oval(X):
        r_ = X - fc
        return np.hypot((r_ @ sd) / 0.064, (r_ @ up + 0.005) / 0.098), (r_ @ f)
    k_, fw = oval(V)
    cut = (k_ < 1.0) & (fw > -0.05)
    me = bpy.data.meshes.new('hood')
    me.from_pydata([tuple(q) for q in kit.to_bl_np(V)], [], F)
    me.update()
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.verts.ensure_lookup_table()
    bmesh.ops.delete(bm, geom=[fa for fa in bm.faces if sum(cut[v.index] for v in fa.verts) >= 2], context='FACES')
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context='VERTS')
    bm.to_mesh(me)
    bm.free()
    hood = kit.link(bpy.data.objects.new('hood', me))
    H = kit.to_game_np(kit.verts_np(hood))
    # the rim on the oval, a little forward of the face and drawn in
    bm = bmesh.new()
    bm.from_mesh(me)
    rim = np.array([v.is_boundary and H[v.index, 1] > y_col + 0.012 and (H[v.index] - fc) @ f > -0.06 for v in bm.verts])
    bm.free()
    r_ = H - fc
    lat, ver = r_ @ sd, r_ @ up + 0.005
    k_ = np.hypot(lat / 0.064, ver / 0.098)
    snap = rim / np.maximum(k_, 1e-4)
    H = H + np.where(rim[:, None], (lat * (snap - 1))[:, None] * sd + (ver * (snap - 1))[:, None] * up, 0.0)
    near_rim = np.exp(-(np.maximum(k_ - 1, 0) / 0.45) ** 2) * (r_ @ f > -0.08)
    H = H + f[None, :] * (0.03 * near_rim)[:, None]
    kit.set_verts_np(hood, kit.to_bl_np(H))
    kit.smooth(hood, 2, 0.4)
    # never inside the skin
    tree = cloth.nearest(P)
    H, _ = cloth.off_the_skin(kit.to_game_np(kit.verts_np(hood)), tree, N, gap=0.02, reach=0.08)
    # and it lies on the coat's shoulders
    C = kit.to_game_np(kit.verts_np(coat))
    H, _ = cloth.off_the_skin(H, cloth.nearest(C), kit.to_game_np(kit.vnormals(coat)), gap=0.008, reach=0.06)
    kit.set_verts_np(hood, kit.to_bl_np(H))
    kit.smooth(hood, 1, 0.3)
    kit.decimate(hood, 1300)
    hood['n_outer'] = len(hood.data.vertices)
    kit.apply_mod(hood, 'SOLIDIFY', thickness=0.006, offset=-1.0, use_rim=True)
    for q in hood.data.polygons:
        q.use_smooth = True
    return hood


def smooth01(x):
    x = np.clip(x, 0.0, 1.0)
    return x * x * (3 - 2 * x)


def build(kit):
    import bpy
    import hm08, human, anatomy as A
    base = hm08.Base().target('macro/male-young-thin', 0.5).target('macro/female-young-thin', 0.5)
    o, skel = human.body(kit, base, 'glass')
    eye_l, eye_l_f = base.helper('l-eye')
    eye_r, eye_r_f = base.helper('r-eye')
    cl0, cr0 = eye_l.mean(0), eye_r.mean(0)
    posed = human.pose(kit, o, skel, pose_of(skel), None, None, smooth_weights=6)

    def frame(bone):
        b0, b1 = np.array(skel.head(bone)), np.array(posed.head(bone))
        R = human._rot_between(np.array(skel.tail(bone)) - b0, np.array(posed.tail(bone)) - b1)
        return lambda X: b1 + (X - b0) @ R.T, R
    to_head, Rh = frame('head')
    eye_l, eye_r = to_head(eye_l), to_head(eye_r)
    cl, cr = to_head(cl0[None])[0], to_head(cr0[None])[0]
    # on the floor
    Pw = kit.to_game_np(kit.verts_np(o))
    lift = -Pw[:, 1].min()
    kit.set_verts_np(o, kit.to_bl_np(Pw + np.array([0, lift, 0])))
    for b in posed.bones:
        b['head'] = (b['head'][0], b['head'][1] + lift, b['head'][2])
        b['tail'] = (b['tail'][0], b['tail'][1] + lift, b['tail'][2])
    up = np.array([0, lift, 0])
    eye_l, eye_r, cl, cr = eye_l + up, eye_r + up, cl + up, cr + up
    face_fwd = Rh @ np.array([0.0, 0.0, 1.0])
    before = kit.to_game_np(kit.verts_np(o))
    kit.remesh(o, 0.005)
    kit.smooth(o, 2, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)
    # thin: a little off the limbs
    P = kit.to_game_np(kit.verts_np(o))
    waste = {b['name']: 0.08 for b in posed.bones if b['name'].split('.')[0] in ('upperarm', 'forearm', 'thigh', 'shin')}
    W = A.waste(P, posed, list(waste), waste, joint_keep=0.06, owner=owner,
                buried=[n for n in waste if n.split('.')[0] in ('thigh', 'upperarm')])
    P = A.relax_offsets(kit, o, P, W, 15)
    kit.set_verts_np(o, kit.to_bl_np(P))

    # ---- what it wears, grown off the skin
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    nb, nd = A.nearest_bones(P, posed)
    bn = np.array([posed.bones[i]['name'].split('.')[0] for i in nb])
    along = np.zeros(len(P))
    for i in np.unique(nb):
        m = nb == i
        along[m] = A.seg_param(P[m], posed.bones[i]['head'], posed.bones[i]['tail'])[0]
    hips = posed.head('hips')
    neck = posed.head('neck')
    head = posed.head('head')
    knee = np.array(posed.tail('thigh.L'))
    hem = knee[1] - 0.10
    # the face: an oval before the eyes, from the brow to under the chin
    mid = (cl + cr) / 2
    fc = mid + np.array([0.0, -0.035, 0.0])
    trunk = np.isin(bn, ['hips', 'spine', 'chest', 'upperchest', 'clavicle'])
    # the coat's body and sleeves grown off the skin, down to under the belt;
    # its skirt below is cloth (below)
    waist = hips[1] + 0.13
    # (the arm's own skin: the hip beside a hanging forearm is nearest it too)
    arm = np.isin(bn, ['upperarm', 'forearm']) & (nd < 0.065)
    trunk |= np.isin(bn, ['upperarm', 'forearm']) & ~arm
    coat_keep = ((trunk & (P[:, 1] > waist - 0.025)) | (arm & (bn == 'upperarm'))
                 | (arm & (bn == 'forearm') & (along < 0.93)) | ((bn == 'neck') & (P[:, 1] < neck[1] + 0.05)))
    coat = A.garment(kit, o, 'coat', coat_keep, 0.018, sag=0.015, tears=0.0, tear_seed=71)
    legs_keep = (bn == 'shin') & (P[:, 1] < hem + 0.03) & (along < 0.62)
    trousers = A.garment(kit, o, 'trousers', legs_keep, 0.008, sag=0.01, tears=0.0)
    boots_keep = np.isin(bn, ['foot', 'toes']) | ((bn == 'shin') & (along > 0.58))
    boots = A.garment(kit, o, 'boots', boots_keep, 0.009, sag=0.0, tears=0.0, smooth_edges=1)

    # a coat hangs loose round the arms and spans the body's hollows
    A.loosen(kit, coat, posed, {'upperarm': (0.056, 0.054), 'forearm': (0.054, 0.05)})
    C = kit.to_game_np(kit.verts_np(coat))
    sh = np.zeros(len(C), bool)
    for side in ('.L', '.R'):
        sh |= np.linalg.norm(C - np.array(posed.head('upperarm' + side)), axis=1) < 0.09
    smooth_where(kit, coat, sh, 25, 0.5)
    C = kit.to_game_np(kit.verts_np(coat))
    cb, _ = A.nearest_bones(C, posed)
    cbn = np.array([posed.bones[i]['name'].split('.')[0] for i in cb])
    on_trunk = np.isin(cbn, ['hips', 'spine', 'chest', 'upperchest'])
    # a raincoat keeps none of the body's small shapes (ribs, the bust's
    # points): the trunk smoothed hard, then spanned and let fall (below)
    body_w = ~np.isin(cbn, ['upperarm', 'forearm', 'hand', 'neck', 'head', 'jaw']) & ~np.char.startswith(cbn, 'finger')
    wide = body_w & (C[:, 1] < neck[1] - 0.06)
    smooth_where(kit, coat, wide, 40, 0.5)
    A.hull_slices(kit, coat, on_trunk & (C[:, 1] < neck[1] - 0.05))
    # stiff cloth: from the chest it falls straight (a little in) to the
    # belt, never back into the body's hollows (the bust's nearest bone is
    # the clavicle: the chest's width, not its bones, says what is trunk)
    C = kit.to_game_np(kit.verts_np(coat))
    chest_y = neck[1] - 0.09
    falls = body_w & ((np.abs(C[:, 0] - hips[0]) < 0.14) | (C[:, 1] < neck[1] - 0.17)) & (C[:, 1] > waist - 0.05)
    A.hang(kit, coat, (hips[0], hips[2]), chest_y, mask=falls, give=0.25)
    C = kit.to_game_np(kit.verts_np(coat))
    kit.set_verts_np(coat, kit.to_bl_np(C))
    # belted: drawn in at the waist
    C = kit.to_game_np(kit.verts_np(coat))
    pinch = np.exp(-((C[:, 1] - waist) / 0.03) ** 2) * on_trunk
    C[:, 0] = hips[0] + (C[:, 0] - hips[0]) * (1 - 0.06 * pinch)
    C[:, 2] = hips[2] + (C[:, 2] - hips[2]) * (1 - 0.06 * pinch)
    kit.set_verts_np(coat, kit.to_bl_np(C))
    kit.smooth(coat, 3, 0.5)
    import cloth
    C, _ = cloth.off_the_skin(kit.to_game_np(kit.verts_np(coat)), cloth.nearest(P), N, gap=0.014, reach=0.06)
    kit.set_verts_np(coat, kit.to_bl_np(C))

    # ---- its skirt: a tube of the coat's cloth from the waist to below the
    # knees, let fall over the legs by Blender's cloth (a little flared)
    top_y, bottom_y = waist + 0.02, hem
    n_round, n_down = 64, 40
    ys = np.linspace(top_y, bottom_y, n_down)
    Pk = kit.to_game_np(kit.verts_np(o))
    legs = ~(arm | np.isin(bn, ['hand', 'head', 'neck']) | np.char.startswith(bn, 'finger')) & (Pk[:, 1] < waist + 0.1)
    Pl = Pk[legs]

    def reach_at(y, dx, dz):
        """how far the legs and the seat reach out along (dx, dz) at y"""
        band = Pl[np.abs(Pl[:, 1] - y) < 0.025]
        if not len(band):
            return 0.10
        rx, rz = band[:, 0] - hips[0], band[:, 2] - hips[2]
        along = rx * dx + rz * dz
        across = np.abs(-rx * dz + rz * dx)
        m = across < 0.075   # it spans the gaps (between the legs, the seat's cleft)
        return along[m].max() if m.any() else 0.10
    # it is taut from the belt over the seat (those rows pinned), and hangs
    # from the widest of it, never coming back in, a little flared to the hem
    seat_y = hips[1] - 0.06
    V, F = [], []
    prev = np.zeros(n_round)
    for y in ys:
        t = smooth01((seat_y - y) / max(seat_y - bottom_y, 0.1))
        ring = []
        for i in range(n_round):
            a_ = 2 * np.pi * i / n_round
            dx, dz = np.sin(a_), np.cos(a_)
            ring.append(reach_at(y, dx, dz) + 0.028)
        ring = np.array(ring)
        for _ in range(3):
            ring = (np.roll(ring, 1) + 2 * ring + np.roll(ring, -1)) / 4
        prev = np.maximum(prev, ring)
        for i in range(n_round):
            a_ = 2 * np.pi * i / n_round
            r = prev[i] + 0.05 * t * (0.3 + 0.7 * np.cos(a_) ** 2)   # flared more front and back
            V.append((hips[0] + r * np.sin(a_), y, hips[2] + r * np.cos(a_)))
    for j in range(n_down - 1):
        for i in range(n_round):
            a0, a1 = j * n_round + i, j * n_round + (i + 1) % n_round
            F.append((a0, a1, a1 + n_round, a0 + n_round))
    taut = [k for k in range(len(V)) if V[k][1] > seat_y]
    me = bpy.data.meshes.new('skirt')
    me.from_pydata([tuple(q) for q in kit.to_bl_np(np.array(V))], [], F)
    me.update()
    skirt = kit.link(bpy.data.objects.new('skirt', me))
    # it falls on the legs; the arms hang outside it
    import bmesh
    lm = o.data.copy()
    bm = bmesh.new()
    bm.from_mesh(lm)
    bm.verts.ensure_lookup_table()
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not legs[v.index]], context='VERTS')
    bm.to_mesh(lm)
    bm.free()
    lo = kit.link(bpy.data.objects.new('legs', lm))
    cloth.collider(lo, 0.012, 6.0)
    fl = cloth.floor(kit)
    cloth.pin(skirt, taut)
    # rubberised cotton: stiff
    S = cloth.settle(kit, skirt, 40, mass=0.35, stiffness=22.0, bending=4.0, self_collide=False)
    bpy.data.objects.remove(fl)
    bpy.data.objects.remove(lo)
    tree = cloth.nearest(Pl)
    S, _ = cloth.off_the_skin(S, tree, kit.to_game_np(kit.vnormals(o))[legs], gap=0.016, reach=0.12)
    kit.set_verts_np(skirt, kit.to_bl_np(S))
    skirt.vertex_groups.clear()
    for q in skirt.data.polygons:
        q.use_smooth = True

    # ---- the hood: a shell over the skull a few centimetres off it, peaked
    # a little at the back, hanging from its widest round straight to the
    # collar; an oval cut for the face, drawn in round it and standing
    # forward of it (its face in the hood's shadow)
    hood = make_hood(kit, bpy, o, coat, posed, bn, fc, face_fwd, neck)

    # ---- the belt, round the join, tied in front
    C = kit.to_game_np(kit.verts_np(coat))
    Sk = kit.to_game_np(kit.verts_np(skirt))
    cb, _ = A.nearest_bones(C, posed)
    on_body = ~np.isin(np.array([posed.bones[i]['name'].split('.')[0] for i in cb]), ['upperarm', 'forearm', 'hand'])
    ring = np.vstack([C[on_body & (np.abs(C[:, 1] - waist) < 0.035)], Sk[np.abs(Sk[:, 1] - waist) < 0.035]])
    ang = np.arctan2(ring[:, 0] - hips[0], ring[:, 2] - hips[2])
    rad = np.hypot(ring[:, 0] - hips[0], ring[:, 2] - hips[2])
    nb_ = 48
    R = np.zeros(nb_)
    for k in range(nb_):
        a_ = -np.pi + 2 * np.pi * (k + 0.5) / nb_
        d_ = np.abs((ang - a_ + np.pi) % (2 * np.pi) - np.pi)
        R[k] = rad[d_ < 2 * np.pi / nb_].max() if (d_ < 2 * np.pi / nb_).any() else 0.15
    R = (np.roll(R, 1) + 2 * R + np.roll(R, -1)) / 4 + 0.006
    BV, BF = [], []
    for y, dr in ((waist - 0.045, 0.0), (waist - 0.043, 0.004), (waist - 0.007, 0.005), (waist + 0.0, 0.004), (waist + 0.002, 0.0)):
        for k in range(nb_):
            a_ = -np.pi + 2 * np.pi * (k + 0.5) / nb_
            BV.append((hips[0] + (R[k] + dr) * np.sin(a_), y, hips[2] + (R[k] + dr) * np.cos(a_)))
    for j in range(4):
        for k in range(nb_):
            a0, a1 = j * nb_ + k, j * nb_ + (k + 1) % nb_
            BF.append((a0, a0 + nb_, a1 + nb_, a1))
    # its buckle, in front
    kn = nb_ // 2
    a_ = -np.pi + 2 * np.pi * (kn + 0.5) / nb_
    out = np.array([np.sin(a_), 0.0, np.cos(a_)])
    kp = np.array([hips[0], waist - 0.021, hips[2]]) + out * (R[kn] + 0.008)
    sd = np.cross(np.array([0, 1.0, 0]), out)
    b0 = len(BV)
    for w_ in (-0.026, 0.026):
        for h_ in (-0.024, 0.024):
            for t_ in (-0.004, 0.004):
                BV.append(tuple(kp + sd * w_ + np.array([0, h_, 0]) + out * t_))
    for q_ in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
        BF.append(tuple(b0 + i for i in q_))
    bme = bpy.data.meshes.new('belt')
    bme.from_pydata([tuple(q_) for q_ in kit.to_bl_np(np.array(BV))], [], BF)
    bme.update()
    belt_o = kit.link(bpy.data.objects.new('belt', bme))

    kit.decimate(o, 9000)
    kit.decimate(coat, 4200)
    kit.decimate(skirt, 2400)
    kit.decimate(trousers, 700)
    kit.decimate(boots, 800)
    A.fill_hollows(kit, coat, 80, 0.5)
    owner = A.carry_values(kit, before, posed.owner, o)

    # ---- paint
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    bn_full = np.array([posed.bones[i]['name'] if i >= 0 else '' for i in owner])
    bn = np.array([n.split('.')[0] for n in bn_full])
    cav = kit.cavity(o)
    # cold: the bluish white of someone long out in the wet, mottled
    pale = np.array(kit.srgb('#c6c8c3'))
    cold = np.array(kit.srgb('#98a2aa'))
    col = pale + (cold - pale) * np.clip(0.5 + kit.fbm(P, 9.0, 3, seed=101), 0, 1)[:, None] * 0.55
    # veins under the thin skin, where it is thinnest (the face's sides,
    # the temples, the neck, the backs of the hands)
    vein = np.clip(1.0 - np.abs(kit.fbm(P, 26.0, 2, seed=171)) / 0.035, 0, 1) ** 2
    thin = np.isin(bn, ['head', 'neck', 'hand']) | np.char.startswith(bn, 'finger')
    col = col + (np.array(kit.srgb('#6c7888')) - col) * (0.45 * vein * thin)[:, None]
    # its eyes' rims raw and red, the hollows under them dark, its lips grey
    lid = np.array(kit.srgb('#8c5652'))
    crown = np.array(posed.tail('head')) - np.array(posed.head('head'))
    crown /= np.linalg.norm(crown)
    headm = bn == 'head'
    up_ = crown
    for c in (cl, cr):
        e = np.abs(np.linalg.norm(P - c, axis=1) - 0.0135)
        front = (P - c) @ face_fwd > -0.004
        col = col + (lid - col) * (headm * np.exp(-(e / 0.005) ** 2) * front * 0.85)[:, None]
        under = np.exp(-(np.linalg.norm(P - (c - up_ * 0.016), axis=1) / 0.012) ** 2) * headm * front
        col = col * (1 - 0.35 * under)[:, None]
    mouth = mid - up_ * 0.072 + face_fwd * 0.012
    side_ = np.cross(face_fwd, up_)
    side_ /= np.linalg.norm(side_)
    rel = P - mouth
    lips = np.exp(-((rel @ side_) / 0.024) ** 4 - ((rel @ up_) / 0.009) ** 2) * headm * (N @ face_fwd > 0.2)
    col = col + (np.array(kit.srgb('#757880')) - col) * (0.8 * lips)[:, None]
    skin_col = np.clip(col * (1 - 0.55 * cav)[:, None], 0.03, 0.9)

    def paint_coat(ob, base_hex, dark_hex, seed, cavity=0.45):
        C = kit.to_game_np(kit.verts_np(ob))
        cv = kit.cavity(ob)
        b0, d0 = np.array(kit.srgb(base_hex)), np.array(kit.srgb(dark_hex))
        col = b0 + (d0 - b0) * np.clip(0.45 + kit.fbm(C, 6.0, 3, seed=seed), 0, 1)[:, None] * 0.45
        # wet: darker in runs down it, and along its hem
        runs = np.clip(kit.fbm(C * np.array([6.0, 0.6, 6.0]), 3.0, 2, seed=seed + 1) + 0.1, 0, 1)
        col = col * (1 - 0.18 * runs)[:, None]
        col = col * (1 - 0.25 * smooth01((hem + 0.08 - C[:, 1]) / 0.08))[:, None]
        return np.clip(col * (1 - cavity * cv)[:, None], 0.03, 0.85), C
    coat_col, C = paint_coat(coat, '#b4ad98', '#8d8672', 111)
    belt_col, _ = paint_coat(belt_o, '#9d9682', '#7d7663', 151, cavity=0.1)
    # (its shell's edge is sharp: cavity there had inked a ring round the face)
    hood_col, H = paint_coat(hood, '#b4ad98', '#8d8672', 121, cavity=0.12)
    skirt_col, _ = paint_coat(skirt, '#b4ad98', '#8d8672', 141)
    # its inside (the solidified shell's second half) in shadow
    inner = np.arange(len(H)) >= hood.get('n_outer', len(H))
    hood_col = np.where(inner[:, None], hood_col * 0.55, hood_col)
    trousers_col = np.tile(np.array(kit.srgb('#2f2e2c')), (len(trousers.data.vertices), 1))
    Bt = kit.to_game_np(kit.verts_np(boots))
    boots_col = np.array(kit.srgb('#262a24')) * (1 + 0.15 * kit.fbm(Bt, 12.0, 2, seed=131))[:, None]

    # ---- `press`: the front of its face flattened, as on glass
    P = kit.to_game_np(kit.verts_np(o))
    plane = (mid @ face_fwd) + 0.028
    depth = P @ face_fwd
    over = np.clip(depth - plane, 0.0, None) * (bn == 'head')
    press = -over[:, None] * face_fwd[None, :]
    # where it presses it spreads a little
    side = np.cross(face_fwd, crown)
    side /= np.linalg.norm(side)
    spread = (P - mid) @ side
    press += (np.clip(over * 6.0, 0, 1) * spread * 0.06)[:, None] * side[None, :]

    # ---- the eyes, on their own bones
    def mesh(name, Vv, Ff):
        me = bpy.data.meshes.new(name)
        me.from_pydata([tuple(p) for p in kit.to_bl_np(Vv)], [], Ff)
        me.update()
        for p in me.polygons:
            p.use_smooth = True
        return kit.link(bpy.data.objects.new(name, me))
    el, er = mesh('eye_l', eye_l, eye_l_f), mesh('eye_r', eye_r, eye_r_f)

    def eye_col(ob):
        E = kit.to_game_np(kit.verts_np(ob))
        c = E.mean(0)
        r = np.linalg.norm(E - c, axis=1).max()
        k = ((E - c) @ face_fwd) / r
        # clouded, as a drowned man's
        white, iris, pupil = (np.array(kit.srgb(h)) for h in ('#c9cbc4', '#8d9792', '#33373a'))
        out = np.tile(white, (len(E), 1))
        out[k > 0.72] = iris
        out[k > 0.92] = pupil
        return out

    body = kit.Body('glass', posed)
    body.skinned('skin', 'skin', o, colors=skin_col, morphs=[('press', press)])
    # what it wears moves as the skin under it does (bone heat on a sleeve
    # hanging against the body had let the trunk pull it when the arms
    # rise to the glass)
    like_skin = skin_weights(body.meshes[-1])
    body.skinned_by('coat', 'coat', coat, like_skin, colors=coat_col)
    # (the hood only with the head, the neck and the chest: by the shoulders'
    # skin it had risen with the arms into two flat sheets either side of it)
    hood_w = skin_weights(body.meshes[0], only=['head', 'neck', 'upperchest', 'chest', 'clavicle'], skel=posed)
    body.skinned_by('hood', 'coat', hood, hood_w, colors=hood_col)
    body.skinned_by('skirt', 'coat', skirt, like_skin, colors=skirt_col)
    body.skinned_by('belt', 'coat', belt_o, like_skin, colors=belt_col)
    body.skinned_by('trousers', 'cloth', trousers, like_skin, colors=trousers_col)
    body.skinned_by('boots', 'boots', boots, like_skin, colors=boots_col)
    body.rigid('eye.L', 'eye', 'head', el, colors=eye_col(el))
    body.rigid('eye.R', 'eye', 'head', er, colors=eye_col(er))
    return body
