"""
The anomalies' figures (docs/design/ENTITIES.md, E9): people's bodies (MakeHuman's,
CC0) fitted to the procedural human rig the game walks the Showroom's staff
and the Borrowed House's shadows on (`crates/game/src/anomalies/rig.rs`),
and faces smoothed away.

The rig is not a body's own skeleton: the game makes its bones itself and
binds the body to them as they stand at rest, so a body must be made in
that rest pose, with the rig's bone names. `rigs.json` (written by
`crates/game/tests/anomaly_bodies.rs`) is that pose; `fit` poses MakeHuman's
body bone by bone along it (each bone turned along its rig bone and
stretched to its length), then moves every joint onto the rig's, blended by
the skin's weights, and `rig_skel` is the skeleton the body is weighted to.
"""
import json, os
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))


def rig_skel(kit, which):
    """the rig `which` ('staff', 'shadow') at rest, as a kit.Skel"""
    bones = json.load(open(os.path.join(HERE, 'rigs.json')))[which]['bones']
    s = kit.Skel()
    for b in bones:
        s.add(b['name'], b['parent'], b['head'], b['tail'])
    return s


def pairs():
    """MakeHuman's bones and the rig's they are fitted to (the rig's chest is
    MakeHuman's chest and upperchest; its fingers count from the index, the
    thumb last; MakeHuman's from the thumb)"""
    out = [('hips', 'pelvis'), ('spine', 'abdomen'), ('neck', 'neck'), ('head', 'head')]
    for s in ('.L', '.R'):
        out += [('clavicle' + s, 'clavicle' + s), ('upperarm' + s, 'upperarm' + s), ('forearm' + s, 'forearm' + s),
                ('hand' + s, 'hand' + s), ('thigh' + s, 'thigh' + s), ('shin' + s, 'shin' + s), ('foot' + s, 'foot' + s),
                ('toes' + s, 'toe' + s)]
        for f, k in ((2, 0), (3, 1), (4, 2), (5, 3), (1, 4)):
            out += [(f'finger{f}{j + 1}{s}', f'finger{k}{j}{s}') for j in range(3)]
    return out


def fit(kit, human, base, which, name, head=1.12, smooth_weights=6):
    """MakeHuman's body `base` in the rest pose of rig `which`: returns the
    skin object, its posed skeleton (MakeHuman's bone names, its joints on
    the rig's) and the rig's skeleton. The head keeps its own shape, `head`
    times as long as a man's, its crown on the rig's: MakeHuman's head bone
    starts at the eyes, the rig's at the skull's foot, so the neck takes up
    the difference"""
    rig = rig_skel(kit, which)
    o, skel = human.body(kit, base, name)
    length = lambda s, n: float(np.linalg.norm(np.array(s.tail(n)) - np.array(s.head(n))))
    unit = lambda v: np.asarray(v, float) / np.linalg.norm(v)
    aims, stretch = {}, {}
    for mh, rg in pairs():
        aims[mh] = np.array(rig.tail(rg)) - np.array(rig.head(rg))
        stretch[mh] = length(rig, rg) / length(skel, mh)
    trunk = length(skel, 'chest') + length(skel, 'upperchest')
    for mh in ('chest', 'upperchest'):
        aims[mh] = np.array(rig.tail('chest')) - np.array(rig.head('chest'))
    # the trunk stretched evenly, its four bones alike (each to its own rig
    # bone's length, the short spine went twice as far as the chest, and the
    # skin creased in a ridge between them); the joints' move sets it right
    even = (length(rig, 'pelvis') + length(rig, 'abdomen') + length(rig, 'chest')) \
        / sum(length(skel, n) for n in ('hips', 'spine', 'chest', 'upperchest'))
    for mh in ('hips', 'spine', 'chest', 'upperchest'):
        stretch[mh] = even
    span = length(rig, 'neck') + length(rig, 'head')
    stretch['head'] = head
    stretch['neck'] = (span - head * length(skel, 'head')) / length(skel, 'neck')
    posed = human.pose(kit, o, skel, aims, stretch, root=rig.head('pelvis'), smooth_weights=smooth_weights)
    # every joint onto the rig's: a move per bone (its head's), blended by
    # the skin's weights to the posed bones
    want = {mh: np.array(rig.head(rg)) for mh, rg in pairs()}
    want['chest'] = np.array(rig.head('chest'))
    k = length(skel, 'chest') / trunk
    want['upperchest'] = np.array(rig.head('chest')) + k * (np.array(rig.tail('chest')) - np.array(rig.head('chest')))
    want['head'] = np.array(rig.tail('head')) - unit(np.array(rig.tail('head')) - np.array(rig.head('head'))) * head * length(skel, 'head')
    nb = len(posed.bones)
    dh = np.zeros((nb, 3))
    for n, p in want.items():
        dh[posed.ix[n]] = p - np.array(posed.head(n))
    dh[posed.ix['jaw']] = dh[posed.ix['head']]
    # (a bone's tail moves as the child whose head is there, else as its head)
    dt = dh.copy()
    for b in posed.bones:
        p = b['parent']
        if p >= 0 and np.linalg.norm(np.array(b['head']) - np.array(posed.bones[p]['tail'])) < 1e-4:
            dt[p] = dh[posed.ix[b['name']]]
    import anatomy as A
    J, W, _ = kit.auto_weights(o, posed)
    P = kit.to_game_np(kit.verts_np(o))
    move = np.zeros_like(P)
    Wf = W / 255.0
    for i, b in enumerate(posed.bones):
        m = (J == i) & (Wf > 0)
        rows = np.nonzero(m.any(1))[0]
        if not len(rows):
            continue
        t = np.clip(A.seg_param(P[rows], b['head'], b['tail'])[0], 0, 1)[:, None]
        shift = dh[i] + (dt[i] - dh[i]) * t
        move[rows] += (Wf[rows] * m[rows]).sum(1)[:, None] * shift
    kit.set_verts_np(o, kit.to_bl_np(P + move))
    # (the weights were only for the move: the body is weighted to the rig later)
    o.vertex_groups.clear()
    # the posed skeleton moved with it: each head and tail as above
    for i, b in enumerate(posed.bones):
        b['head'] = tuple(np.array(b['head']) + dh[i])
        b['tail'] = tuple(np.array(b['tail']) + dt[i])
    return o, posed, rig


def follow(skel, posed, bone):
    """points at rest (on `skel`) carried as bone `bone` was posed (turned,
    stretched along its length, moved): -> a function of N x 3 points, and
    the bone's turn (3 x 3)"""
    import human
    b0, t0 = np.array(skel.head(bone)), np.array(skel.tail(bone))
    b1, t1 = np.array(posed.head(bone)), np.array(posed.tail(bone))
    R = human._rot_between(t0 - b0, t1 - b1)
    a = (t0 - b0) / np.linalg.norm(t0 - b0)
    s = np.linalg.norm(t1 - b1) / np.linalg.norm(t0 - b0)
    S = np.eye(3) + (s - 1.0) * np.outer(a, a)
    return (lambda X: b1 + (np.asarray(X, float) - b0) @ (R @ S).T), R


def edges(o):
    E = np.zeros(len(o.data.edges) * 2, dtype=np.int64)
    o.data.edges.foreach_get('vertices', E)
    return E.reshape(-1, 2)


def smooth_masked(kit, o, w, iters, lam=0.5, mu=-0.53):
    """Taubin smoothing (a shrink step and an inflate step, so the surface
    keeps its volume) where `w` (0..1 per vertex) says, over `o`'s edges"""
    E = edges(o)
    P = kit.to_game_np(kit.verts_np(o))
    n = len(P)
    deg = np.maximum(np.bincount(E.ravel(), minlength=n), 1).astype(float)[:, None]
    w = np.asarray(w, float)[:, None]
    for _ in range(iters):
        for f in (lam, mu):
            acc = np.zeros_like(P)
            np.add.at(acc, E[:, 0], P[E[:, 1]])
            np.add.at(acc, E[:, 1], P[E[:, 0]])
            P = P + f * w * (acc / deg - P)
    kit.set_verts_np(o, kit.to_bl_np(P))


def face_mask(kit, o, centre, fwd, up, half_w, half_h, skull=None):
    """0..1 over `o`'s vertices: the front of the head round `centre` (an
    ellipse `half_w` across and `half_h` up and down, facing `fwd`), soft to
    its edge. Facing is judged from `skull` (the head's middle) when given,
    else by the surface's normals (which, in the hollow of an eye on a fine
    mesh, face sideways and left the socket out)"""
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    if skull is not None:
        R = P - np.asarray(skull, float)
        N = R / np.maximum(np.linalg.norm(R, axis=1), 1e-9)[:, None]
    fwd = np.asarray(fwd, float) / np.linalg.norm(fwd)
    up = np.asarray(up, float) - fwd * (np.asarray(up, float) @ fwd)
    up /= np.linalg.norm(up)
    side = np.cross(up, fwd)
    rel = P - np.asarray(centre, float)
    r = np.sqrt(((rel @ side) / half_w) ** 2 + ((rel @ up) / half_h) ** 2)
    front = np.clip((N @ fwd - 0.05) / 0.4, 0, 1) * ((rel @ fwd) > -0.02)
    return kit.smoothstep(1.0, 0.55, r) * front


def faceless(kit, o, centre, fwd, up, skull, half_w=0.062, half_h=0.085, sigma=0.4, iters=12, flatten=8, hollow=0.0):
    """no face: the eyes, the nose and the mouth gone under the skin. Seen
    from the skull's middle `skull`, each point of the face takes the
    distance of the head round it (an average over the directions near its
    own, of the points outside the face), so the brow, the cheeks and the
    chin run on into each other and the head keeps its shape; then a little
    smoothing where it was done (Taubin's, which keeps the volume, then
    `flatten` plain passes for what is left of a nose); `hollow` m sunk at
    its middle, a faint hollow where a face was, not an egg)"""
    w = face_mask(kit, o, centre, fwd, up, half_w, half_h, skull=skull)
    P = kit.to_game_np(kit.verts_np(o))
    rel = P - np.asarray(skull, float)
    r = np.linalg.norm(rel, axis=1)
    u = rel / np.maximum(r, 1e-9)[:, None]
    fw = np.asarray(fwd, float) / np.linalg.norm(fwd)
    reach = np.percentile(r[w > 0.5], 95) * 1.25 if (w > 0.5).any() else 0.15
    samples = (w < 0.02) & (r < reach) & (u @ fw > -0.3)
    face = np.nonzero(w > 0)[0]
    us, rs = u[samples], r[samples]
    for k0 in range(0, len(face), 256):
        f = face[k0:k0 + 256]
        c = np.clip(u[f] @ us.T, -1, 1)
        k = np.exp(-(1.0 - c) / (sigma * sigma / 2.0))
        rs_f = (k * rs[None, :]).sum(1) / np.maximum(k.sum(1), 1e-12)
        P[f] = skull + u[f] * (r[f] + w[f] * (rs_f - r[f]) - hollow * w[f] ** 2)[:, None]
    P0 = kit.to_game_np(kit.verts_np(o))
    print(f'[creatures] faceless: {int((w > 0.5).sum())} points in the face, {int(samples.sum())} round it, '
          f'moved up to {np.abs(np.linalg.norm(P - P0, axis=1)).max() * 1000:.1f} mm; centre {np.round(centre, 3)}, '
          f'skull {np.round(skull, 3)}, fwd {np.round(fw, 2)}')
    kit.set_verts_np(o, kit.to_bl_np(P))
    smooth_masked(kit, o, np.clip(w * 1.5, 0, 1), iters)
    smooth_masked(kit, o, np.clip(w * 1.5, 0, 1), flatten, lam=0.5, mu=0.0)
    return w


def enclose(kit, outer, inner, axis_xz, y0, y1, gap=0.003, bins=32, step=0.01, inner_mask=None):
    """push `outer` out round a vertical axis through `axis_xz`, between the
    heights `y0` and `y1`, to lie at least `gap` outside `inner` (the
    trousers' waist over a tucked shirt): per column and height, the
    inner's furthest point (of those in `inner_mask`: the trunk, not the
    arms hanging beside it)"""
    O = kit.to_game_np(kit.verts_np(outer))
    I = kit.to_game_np(kit.verts_np(inner))
    c = np.asarray(axis_xz, float)

    def polar(Q):
        rel = Q[:, [0, 2]] - c
        return np.arctan2(rel[:, 0], rel[:, 1]), np.linalg.norm(rel, axis=1), rel
    tho, rho, relo = polar(O)
    thi, rhi, _ = polar(I)
    ki = np.floor((thi + np.pi) / (2 * np.pi) * bins).astype(int) % bins
    ko = np.floor((tho + np.pi) / (2 * np.pi) * bins).astype(int) % bins
    hi = np.floor((I[:, 1] - y0) / step).astype(int)
    ho = np.floor((O[:, 1] - y0) / step).astype(int)
    nh = max(int(np.ceil((y1 - y0) / step)) + 1, 1)
    far = np.zeros((bins, nh))
    m = (I[:, 1] >= y0 - step) & (I[:, 1] <= y1 + step)
    if inner_mask is not None:
        m &= np.asarray(inner_mask, bool)
    # (each column and height takes its neighbours' too: a garment's
    # triangles span more than a bin)
    for k, h, r in zip(ki[m], np.clip(hi[m], 0, nh - 1), rhi[m]):
        for dk in (-1, 0, 1):
            for dh in (-1, 0, 1):
                hh = min(max(h + dh, 0), nh - 1)
                far[(k + dk) % bins, hh] = max(far[(k + dk) % bins, hh], r)
    sel = (O[:, 1] >= y0) & (O[:, 1] <= y1)
    want = far[ko[sel], np.clip(ho[sel], 0, nh - 1)] + gap
    new = np.maximum(rho[sel], want)
    out = rho.copy()
    out[sel] = new
    O[:, 0] = c[0] + relo[:, 0] * out / np.maximum(rho, 1e-6)
    O[:, 2] = c[1] + relo[:, 1] * out / np.maximum(rho, 1e-6)
    kit.set_verts_np(outer, kit.to_bl_np(O))


def preview_band(kit, objs, y0, y1, path):
    """(debugging) Workbench views of the given objects cut to the heights
    y0..y1 (copies, removed after): with FIGURE_DEBUG=dir set, a body
    script can look closely at its waist or its face"""
    import bpy, bmesh
    copies = []
    for ob in objs:
        me = ob.data.copy()
        c = kit.link(bpy.data.objects.new(ob.name + '_band', me))
        bm = bmesh.new()
        bm.from_mesh(me)
        P = kit.to_game_np(np.array([v.co for v in bm.verts]))
        drop = [v for v, p in zip(bm.verts, P) if p[1] < y0 or p[1] > y1]
        bmesh.ops.delete(bm, geom=drop, context='VERTS')
        bm.to_mesh(me)
        bm.free()
        copies.append(c)
    for c in [c for c in copies if not len(c.data.vertices)]:
        copies.remove(c)
        bpy.data.objects.remove(c)
    # (nothing else drawn: only the copies)
    others = [ob for ob in bpy.context.scene.objects if ob.type == 'MESH' and ob not in copies and not ob.hide_render]
    for ob in others:
        ob.hide_render = True
    kit.preview(copies, path)
    for ob in others:
        ob.hide_render = False
    for c in copies:
        bpy.data.objects.remove(c)


def level_edge(kit, o, y_lo, y_hi, y, side=-1):
    """the open edge of `o` between the heights `y_lo` and `y_hi` (a hem, a
    waistband) brought to the height `y` exactly, and the points within 2 cm
    of it on the garment's side (`side` -1: above it, a hem; 1: below it, a
    waistband) eased toward the same line: a clean edge, not the remesh's
    stair"""
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(o.data)
    rim = set()
    for e in bm.edges:
        if len(e.link_faces) == 1:
            for v in e.verts:
                rim.add(v.index)
    bm.free()
    P = kit.to_game_np(kit.verts_np(o))
    idx = np.array(sorted(i for i in rim if y_lo < P[i, 1] < y_hi), dtype=np.int64)
    if len(idx):
        P[idx, 1] = y
    d = (P[:, 1] - y) * -side
    near = (d > 0) & (d < 0.02) & ~np.isin(np.arange(len(P)), idx)
    P[near, 1] = y + (P[near, 1] - y) * 0.85
    kit.set_verts_np(o, kit.to_bl_np(P))


def decimate_keeping(kit, o, tris, keep, passes=8):
    """`o` decimated to about `tris` triangles, collapsing least where `keep`
    (0..1 per vertex) is high: the hands keep their fingers apart. (A group
    holds its points back from one pass, so passes repeat until the count
    is reached.)"""
    w = 1.0 - np.clip(np.asarray(keep, float), 0, 1)
    tri_count = lambda: sum(len(p.vertices) - 2 for p in o.data.polygons)
    for _ in range(passes):
        n = tri_count()
        if n <= tris * 1.02:
            break
        g = o.vertex_groups.new(name='collapse')
        for v in np.unique(np.round(w, 2)):
            g.add([int(i) for i in np.nonzero(np.round(w, 2) == v)[0]], float(v), 'REPLACE')
        kit.apply_mod(o, 'DECIMATE', decimate_type='COLLAPSE', ratio=max(tris / n, 0.05), use_collapse_triangulate=True,
                      vertex_group='collapse', vertex_group_factor=1.0)
        # (the weights go with the points that are left)
        vg = o.vertex_groups['collapse']
        w = np.array([next((x.weight for x in v.groups if x.group == vg.index), 1.0) for v in o.data.vertices])
        o.vertex_groups.remove(vg)
    if tri_count() > tris * 1.02:
        kit.decimate(o, tris)
    return o


def hand_mask(posed, owner):
    """1 for the points of the hands (by the bone each belonged to), else 0"""
    names = [posed.bones[i]['name'] if i >= 0 else '' for i in owner]
    return np.array([1.0 if n.startswith(('hand', 'finger')) else 0.0 for n in names])


def waist_on(kit, o, skel, bone, y0, y1):
    """weights for garment `o` (`kit.Body.skinned_by`): Blender's bone heat,
    and from the height `y0` up to `y1` more and more all on `bone` (a
    waistband rides the pelvis: weighted to the thighs by heat, the seat of
    the trousers opened at the back in a long stride)"""
    k = skel.ix[bone]

    def weights(Q):
        J, W, _ = kit.auto_weights(o, skel)
        Wf = W / 255.0
        b = kit.smoothstep(y0, y1, np.asarray(Q)[:, 1])
        Wf = Wf * (1.0 - b)[:, None]
        for v in np.nonzero(b > 0)[0]:
            slots = list(J[v])
            s = slots.index(k) if k in slots else int(np.argmin(Wf[v]))
            if k not in slots:
                J[v, s], Wf[v, s] = k, 0.0
            Wf[v, s] += b[v]
        Wf /= np.maximum(Wf.sum(1, keepdims=True), 1e-9)
        return J, Wf
    return weights

