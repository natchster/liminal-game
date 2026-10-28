"""
Anatomy by displacement, for bodies posed from a person (human.py): where a
point of the skin is relative to the skeleton, and the starved reads that
follow from it (wasting, ribs, spine, shoulder blades, hip bones), and
garments grown off the skin. All in the game's frame, NumPy only, seeded.
"""
import numpy as np


def seg_param(P, a, b):
    """for N points: the parameter along a→b (clamped 0..1), the distance to
    the segment, and the radial vector from it"""
    a, b = np.asarray(a, float), np.asarray(b, float)
    d = b - a
    L2 = max(float(d @ d), 1e-12)
    t = np.clip(((P - a) @ d) / L2, 0.0, 1.0)
    foot = a + t[:, None] * d
    r = P - foot
    return t, np.linalg.norm(r, axis=1), r


def nearest_bones(P, skel, names=None):
    """index (into skel.bones) of the nearest bone segment for every point, and the distance"""
    idx = [i for i, b in enumerate(skel.bones) if names is None or b['name'] in names]
    D = np.stack([seg_param(P, skel.bones[i]['head'], skel.bones[i]['tail'])[1] for i in idx], axis=1)
    k = np.argmin(D, axis=1)
    return np.array(idx)[k], D[np.arange(len(P)), k]


def region(P, skel, prefixes):
    """mask of points whose nearest bone's name starts with one of `prefixes`"""
    nb, _ = nearest_bones(P, skel)
    names = np.array([skel.bones[i]['name'] for i in nb])
    return np.array([any(n.startswith(p) for p in prefixes) for n in names]), names


def waste(P, skel, bones, amount, joint_keep=0.06, owner=None, buried=()):
    """pull points nearest each of `bones` toward its segment by `amount`
    (0..1, or a dict of amounts by bone), less near its ends (the joints
    stay knobby; not the head of a bone in `buried`, a joint inside the
    trunk: the hip, the shoulder). `owner` (an index into skel.bones per
    point, as `carry_values` gives) says whose each point is instead of the
    nearest bone"""
    nb = nearest_bones(P, skel)[0] if owner is None else owner
    out = P.copy()
    for i, b in enumerate(skel.bones):
        if b['name'] not in bones:
            continue
        m = nb == i
        if not m.any():
            continue
        t, dist, r = seg_param(P[m], b['head'], b['tail'])
        L = np.linalg.norm(np.array(b['tail']) - np.array(b['head']))
        e = np.minimum(t * L, (1 - t) * L) if b['name'] not in buried else (1 - t) * L   # from the nearer joint
        a = amount[b['name']] if isinstance(amount, dict) else amount
        k = a * np.clip(e / joint_keep, 0.0, 1.0)
        out[m] = P[m] - r * k[:, None]
    return out


def carry_values(kit, before, values, o):
    """`values` (per vertex of `before`, positions in the game frame: the
    bone each follows, a mask) carried onto the vertices of `o` (a remesh
    or decimation of it): each takes its nearest's"""
    from mathutils import kdtree
    tree = kdtree.KDTree(len(before))
    for i, p in enumerate(before):
        tree.insert(p, i)
    tree.balance()
    return np.array([values[tree.find(p)[1]] for p in kit.to_game_np(kit.verts_np(o))])


def relax_offsets(kit, o, P0, P1, iters, factor=0.5):
    """P0 moved to P1 with the offsets smoothed over `o`'s edges (its
    topology, P0's order): no step where neighbours moved differently"""
    import numpy as np
    E = np.zeros(len(o.data.edges) * 2, dtype=np.int64)
    o.data.edges.foreach_get('vertices', E)
    E = E.reshape(-1, 2)
    deg = np.bincount(E.ravel(), minlength=len(P0)).astype(float)[:, None]
    D = P1 - P0
    for _ in range(iters):
        acc = np.zeros_like(D)
        np.add.at(acc, E[:, 0], D[E[:, 1]])
        np.add.at(acc, E[:, 1], D[E[:, 0]])
        avg = np.where(deg > 0, acc / np.maximum(deg, 1), D)
        D = D + factor * (avg - D)
    return P0 + D


def bumps_along(P, N, a, b, spacing, amp, width, side_dir, side_min=0.2):
    """knobs along a→b every `spacing` m on the side of the surface facing
    `side_dir` (the spine's knobs on a back): offsets along N"""
    t, dist, r = seg_param(P, a, b)
    L = np.linalg.norm(np.asarray(b, float) - np.asarray(a, float))
    s = t * L / spacing
    knob = 0.5 + 0.5 * np.cos(2 * np.pi * s)
    face = np.clip((N @ np.asarray(side_dir, float) - side_min) / (1 - side_min), 0, 1)
    near = np.exp(-(dist / width) ** 2 * 0.0)  # all points along: the radial falloff is the caller's mask
    return (amp * knob ** 3 * face * near)[:, None] * N


def ribs(P, N, top, bottom, centre_axis, count, amp, mask, irregular=0.0, seed=0):
    """a ribcage's ribs as raised bands with grooves between, on the points of
    `mask`: band k runs round the body at the height of rib k between `top`
    and `bottom` (points on the spine), sloping down toward the front"""
    top, bottom = np.asarray(top, float), np.asarray(bottom, float)
    axis = bottom - top
    L = np.linalg.norm(axis)
    u = axis / L
    t = ((P - top) @ u) / L                                   # 0 at the top rib, 1 at the bottom
    fwd = np.asarray(centre_axis, float)
    fwd = fwd - u * (fwd @ u)
    fwd /= np.linalg.norm(fwd)
    front = (P - top) - np.outer((P - top) @ u, u)
    fn = np.linalg.norm(front, axis=1) + 1e-9
    toward_front = (front @ fwd) / fn                          # +1 at the breast, -1 at the spine
    slope = 0.10 * (toward_front + 1)                          # ribs fall toward the front
    phase = (t + slope) * count
    if irregular:
        # no two ribs alike: their spacing and depth wander
        import kit
        phase = phase + irregular * kit.fbm(P, 9.0, 2, seed=seed)
        amp = amp * (0.7 + 0.6 * np.clip(0.5 + kit.fbm(P, 5.0, 2, seed=seed + 1), 0, 1))
    band = np.cos(2 * np.pi * phase)
    inside = (t > -0.02) & (t < 1.02) & mask
    fade = np.clip(np.minimum(t + 0.02, 1.02 - t) / 0.15, 0, 1)
    lateral = 0.35 + 0.65 * (1 - np.abs(toward_front))        # strongest on the flanks
    off = amp * band * fade * lateral * inside
    return off[:, None] * N


def offset_surface(P, N, base, sag_dir=(0, -1, 0), sag=0.0, noise=None):
    """a garment's surface: the skin pushed out along its normal by `base`,
    and hanging toward `sag_dir` where the skin faces it (loose cloth)"""
    g = np.asarray(sag_dir, float)
    hang = np.clip(N @ g, 0, 1)
    o = P + N * base + g[None, :] * (sag * hang)[:, None]
    if noise is not None:
        o = o + N * noise[:, None]
    return o


def garment(kit, src, name, keep, offset, sag=0.0, tears=0.0, tear_seed=0, tear_freq=9.0, smooth_edges=2):
    """a garment grown off the skin object `src`: the faces whose vertices
    are all in `keep` (a mask over src's vertices), pushed out along their
    normals by `offset`, hanging by `sag` where the skin faces down, with
    holes where a noise field passes `1 - tears`. Returns the new object."""
    import bpy, bmesh
    me = src.data.copy()
    o = kit.link(bpy.data.objects.new(name, me))
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.verts.ensure_lookup_table()
    hole = np.zeros(len(P), bool)
    if tears > 0:
        hole = kit.fbm(P, tear_freq, 3, seed=tear_seed) > (1.0 - 2.0 * tears)
    drop = [f for f in bm.faces if not all(keep[v.index] for v in f.verts) or any(hole[v.index] for v in f.verts)]
    bmesh.ops.delete(bm, geom=drop, context='FACES')
    # drop the loose crumbs a noise cut leaves
    islands = []
    seen = set()
    for f in bm.faces:
        if f.index in seen:
            continue
        stack, comp = [f], []
        seen.add(f.index)
        while stack:
            g = stack.pop()
            comp.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h.index not in seen:
                        seen.add(h.index)
                        stack.append(h)
        islands.append(comp)
    bm.faces.ensure_lookup_table()
    for comp in islands:
        if len(comp) < 40:
            bmesh.ops.delete(bm, geom=comp, context='FACES')
    loose = [v for v in bm.verts if not v.link_faces]
    bmesh.ops.delete(bm, geom=loose, context='VERTS')
    bm.to_mesh(me)
    bm.free()
    me.update()
    P = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    g = np.array([0.0, -1.0, 0.0])
    hang = np.clip(N @ g, 0, 1)
    wob = kit.fbm(P, 14.0, 2, seed=tear_seed + 7)
    P = P + N * (offset * (1 + 0.35 * wob))[:, None] + g[None, :] * (sag * hang * (0.7 + 0.6 * np.clip(wob + 0.5, 0, 1)))[:, None]
    kit.set_verts_np(o, kit.to_bl_np(P))
    if smooth_edges:
        kit.smooth(o, smooth_edges, 0.4)
    return o


def fill_hollows(kit, o, iters=60, factor=0.5):
    """a garment `o` drawn across the hollows of the body under it (the
    spine's groove, between the buttocks, behind the knees), as cloth spans
    them: each point moves toward its neighbours' mean, but only outward
    along its normal, so hollows fill and nothing shrinks"""
    E = np.zeros(len(o.data.edges) * 2, dtype=np.int64)
    o.data.edges.foreach_get('vertices', E)
    E = E.reshape(-1, 2)
    S = kit.to_game_np(kit.verts_np(o))
    N = kit.to_game_np(kit.vnormals(o))
    deg = np.maximum(np.bincount(E.ravel(), minlength=len(S)), 1).astype(float)[:, None]
    for _ in range(iters):
        acc = np.zeros_like(S)
        np.add.at(acc, E[:, 0], S[E[:, 1]])
        np.add.at(acc, E[:, 1], S[E[:, 0]])
        out = np.einsum('ij,ij->i', acc / deg - S, N)
        S = S + N * (np.maximum(out, 0.0) * factor)[:, None]
    kit.set_verts_np(o, kit.to_bl_np(S))
    return o


def hang(kit, o, axis_xz, top_y, mask=None, bins=64, give=0.15):
    """a garment `o` hanging under gravity from `top_y` down: round a
    vertical axis through `axis_xz`, no point lower down comes back in
    nearer the axis than the cloth above it (it falls straight from the
    widest of the body, over the buttocks, the belly, the shoulder blades);
    `give` lets it come back in that share of the way (it is soft).
    `mask` limits it to the trunk."""
    S = kit.to_game_np(kit.verts_np(o))
    c = np.asarray(axis_xz, float)
    rel = S[:, [0, 2]] - c
    th = np.arctan2(rel[:, 0], rel[:, 1])
    rho = np.linalg.norm(rel, axis=1)
    use = (S[:, 1] < top_y) if mask is None else ((S[:, 1] < top_y) & mask)
    out = rho.copy()
    # each column of the cloth (overlapping bins, so neighbours agree), top down
    for k in range(bins):
        a = -np.pi + 2 * np.pi * k / bins
        d = np.abs((th - a + np.pi) % (2 * np.pi) - np.pi)
        col = np.nonzero(use & (d < 2.0 * np.pi / bins))[0]
        if not len(col):
            continue
        col = col[np.argsort(-S[col, 1])]
        run = np.maximum.accumulate(rho[col])
        want = run - (run - rho[col]) * give
        out[col] = np.maximum(out[col], want)
    S[:, 0] = c[0] + rel[:, 0] * out / np.maximum(rho, 1e-6)
    S[:, 2] = c[1] + rel[:, 1] * out / np.maximum(rho, 1e-6)
    kit.set_verts_np(o, kit.to_bl_np(S))
    return o


def _hull2(Q):
    """the convex hull of 2D points Q (monotone chain), counter-clockwise"""
    Q = Q[np.lexsort((Q[:, 1], Q[:, 0]))]

    def cross(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    lo, hi = [], []
    for p in Q:
        while len(lo) >= 2 and cross(lo[-2], lo[-1], p) <= 0:
            lo.pop()
        lo.append(p)
    for p in Q[::-1]:
        while len(hi) >= 2 and cross(hi[-2], hi[-1], p) <= 0:
            hi.pop()
        hi.append(p)
    return np.array(lo[:-1] + hi[:-1])


def hull_slices(kit, o, mask, step=0.012, give=0.12):
    """a garment `o` across the hollows of the trunk, as knit lies: in every
    level slice `step` m deep, each point of `mask` pushed out from the
    slice's middle to the slice's convex hull (`give`: the share of the way
    back in it keeps); the arms are left out by the mask"""
    S = kit.to_game_np(kit.verts_np(o))
    idx = np.nonzero(mask)[0]
    if not len(idx):
        return o
    y0, y1 = S[idx, 1].min(), S[idx, 1].max()
    out = S.copy()
    for lo in np.arange(y0, y1 + step, step):
        sl = idx[(S[idx, 1] >= lo - step * 0.5) & (S[idx, 1] < lo + step * 1.5)]
        here = idx[(S[idx, 1] >= lo) & (S[idx, 1] < lo + step)]
        if len(sl) < 8 or not len(here):
            continue
        H = _hull2(S[sl][:, [0, 2]])
        if len(H) < 3:
            continue
        c = H.mean(0)
        for i in here:
            p = S[i, [0, 2]]
            d = p - c
            r = np.linalg.norm(d)
            if r < 1e-6:
                continue
            u = d / r
            # where the ray from the middle through p leaves the hull
            best = r
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
                    break
            want = best - (best - r) * give
            out[i, 0], out[i, 2] = c[0] + u[0] * want, c[1] + u[1] * want
    kit.set_verts_np(o, kit.to_bl_np(out))
    return o


def loosen(kit, o, skel, radii):
    """a garment `o` hanging loose round the limbs: every point nearest a
    bone named in `radii` ({name prefix: (radius at its head, at its tail)})
    pushed out from the bone to at least that radius"""
    S = kit.to_game_np(kit.verts_np(o))
    nb, _ = nearest_bones(S, skel)
    for i in np.unique(nb):
        name = skel.bones[i]['name']
        key = next((k for k in radii if name.split('.')[0] == k), None)
        if key is None:
            continue
        m = nb == i
        t, d, r = seg_param(S[m], skel.bones[i]['head'], skel.bones[i]['tail'])
        r0, r1 = radii[key]
        want = r0 + (r1 - r0) * t
        k = np.where(d < want, want / np.maximum(d, 1e-6), 1.0)
        S[m] = S[m] - r + r * k[:, None]
    kit.set_verts_np(o, kit.to_bl_np(S))
    return o


def strands(kit, name, roots, dirs, lengths, radius, gravity=(0, -1, 0), segments=5, sides=3, droop=0.8, seed=0, clump=0.0):
    """hair as thin tapered tubes: from each root along its direction,
    bending toward gravity; `clump` pulls neighbouring tips together"""
    import bmesh
    rng = np.random.default_rng(seed)
    gvec = np.asarray(gravity, float)

    def build(bm):
        for r0, d0, L in zip(roots, dirs, lengths):
            pts = [np.asarray(r0, float)]
            d = np.asarray(d0, float) / np.linalg.norm(d0)
            step = L / segments
            jitter = rng.normal(0, 0.25, 3)
            for k in range(segments):
                d = d + gvec * droop * (k + 1) / segments + jitter * 0.15
                d /= np.linalg.norm(d)
                pts.append(pts[-1] + d * step)
            rings = []
            for k, p in enumerate(pts):
                a = (pts[min(k + 1, len(pts) - 1)] - pts[max(k - 1, 0)])
                a /= np.linalg.norm(a) + 1e-9
                u = np.cross(a, [0.3, 1, 0.1]); u /= np.linalg.norm(u) + 1e-9
                w = np.cross(a, u)
                rr = radius * (1 - 0.85 * k / segments)
                rings.append([bm.verts.new(tuple(kit.to_bl(p + rr * (np.cos(t) * u + np.sin(t) * w))))
                              for t in np.linspace(0, 2 * np.pi, sides, endpoint=False)])
            for k in range(len(rings) - 1):
                for s in range(sides):
                    bm.faces.new((rings[k][s], rings[k][(s + 1) % sides], rings[k + 1][(s + 1) % sides], rings[k + 1][s]))
    return kit.obj_from_bmesh(name, build)


# ---- teeth: a row of single teeth along a dental arch

class Arch:
    """a dental arch in a head's rest frame (+z ahead, +y up): the front
    of the middle teeth at (x0, ·, zf), half as wide as `W`, its back teeth
    `D` behind the front: x = x0 + W·u, z = zf - D·u², u in [-1, 1]"""

    def __init__(self, x0, zf, W, D):
        self.x0, self.zf, self.W, self.D = x0, zf, W, D

    def at(self, u):
        return self.x0 + self.W * u, self.zf - self.D * u * u

    def frame(self, u):
        """along the arch (toward +u) and out of it (ahead at the middle)"""
        t = np.array([self.W, 0.0, -2 * self.D * u])
        t /= np.linalg.norm(t)
        return t, np.array([-t[2], 0.0, t[0]]) * np.sign(t[0])

    def front(self, x):
        """the arch's z at x (its last tooth's beyond its ends)"""
        u = np.clip((np.asarray(x) - self.x0) / self.W, -1, 1)
        return self.zf - self.D * u * u

    def u_at(self, s):
        """u at arc length s from the middle (s ≥ 0)"""
        u = np.linspace(0, 1.3, 400)
        seg = np.hypot(self.W * np.diff(u), self.D * np.diff(u * u))
        return np.interp(s, np.concatenate([[0], np.cumsum(seg)]), u)

    def half_length(self):
        u = np.linspace(0, 1, 400)
        return float(np.sum(np.hypot(self.W * np.diff(u), self.D * np.diff(u * u))))


# mesiodistal widths (m) from the middle back, and each tooth's shape
UPPER = ((0.0085, 'incisor'), (0.0065, 'incisor'), (0.0075, 'canine'), (0.007, 'premolar'),
         (0.0065, 'premolar'), (0.010, 'molar'), (0.009, 'molar'), (0.0085, 'molar'))
LOWER = ((0.0050, 'incisor'), (0.0055, 'incisor'), (0.007, 'canine'), (0.007, 'premolar'),
         (0.007, 'premolar'), (0.011, 'molar'), (0.0105, 'molar'), (0.010, 'molar'))


def teeth_row(arch, gum_y, tip_y, upper, seed, missing=(), broken=(), flare=0.003, jitter=1.0):
    """single teeth along `arch`, rooted 4 mm inside the gum at `gum_y`,
    their tips at `tip_y` (the bite), each its own shape (chisel incisors,
    pointed canines, cusped molars) and none quite like the next: lengths,
    tilts and turns wander (`seed`); `missing` / `broken` name teeth by
    (side, index from the middle). Returns vertices, faces and colours
    (ivory, stained toward the root)"""
    rng = np.random.RandomState(seed)
    row = UPPER if upper else LOWER
    total = sum(w for w, _ in row)
    scale = arch.half_length() / total
    down = -1.0 if upper else 1.0                      # from the root to the tip
    M = 10
    ang = np.linspace(0, 2 * np.pi, M, endpoint=False)
    ce, se = np.cos(ang), np.sin(ang)
    sx_, sz_ = np.sign(ce) * np.abs(ce) ** 0.7, np.sign(se) * np.abs(se) ** 0.7   # a rounded box
    V, F, C = [], [], []
    ivory, stain = np.array([0.79, 0.75, 0.62]), np.array([0.42, 0.33, 0.20])
    for side in (1, -1):
        s0 = 0.0
        for k, (w, kind) in enumerate(row):
            w = w * scale
            s_mid = s0 + w / 2
            s0 += w
            r = rng.uniform(-1, 1, 6) * jitter                  # drawn for every tooth, kept or not
            if (side, k) in missing:
                continue
            u = side * arch.u_at(s_mid)
            x, z = arch.at(u)
            t, n = arch.frame(u)
            t = t * side
            length = abs(tip_y - gum_y) * (1 + 0.08 * r[0]) + (0.002 if kind == 'canine' else 0.0)
            if (side, k) in broken:
                length *= 0.55
            root = np.array([x, gum_y - down * 0.004, z])
            tip = np.array([x, gum_y + down * length, z]) + n * (flare * (1.0 if kind != 'molar' else 0.3)) \
                + t * 0.0012 * r[1] + n * 0.0008 * r[2]
            axis = tip - root
            twist = 0.15 * r[3]
            tt = t * np.cos(twist) + n * np.sin(twist)
            nn = n * np.cos(twist) - t * np.sin(twist)
            a = w * 0.46                                         # half across the arch (gaps between)
            b = {'incisor': 0.0034, 'canine': 0.0042, 'premolar': 0.0045, 'molar': 0.0052}[kind]
            if not upper:
                b *= 0.9
            rings = (0.0, 0.3, 0.55, 0.8, 0.93, 1.0)
            base = len(V)
            for j, f in enumerate(rings):
                neck = 0.82 + 0.18 * np.clip(f / 0.3, 0, 1)        # narrow at the gum, full in the crown
                if kind == 'incisor':
                    ka, kb = 1.0, 1.0 - 0.65 * max(f - 0.5, 0) / 0.5   # a chisel edge
                elif kind == 'canine':
                    ka = kb = 1.0 - 0.85 * max(f - 0.45, 0) / 0.55     # a point
                elif kind == 'premolar':
                    ka = kb = 1.0 - 0.35 * max(f - 0.7, 0) / 0.3
                else:
                    ka = kb = 1.0 - 0.2 * max(f - 0.8, 0) / 0.2
                c = root + axis * f
                for i in range(M):
                    V.append(c + tt * (a * neck * ka * sx_[i]) + nn * (b * neck * kb * sz_[i]))
                    g = np.clip((f - 0.15) / 0.5, 0, 1)
                    C.append(stain + (ivory - stain) * g * (0.92 + 0.08 * r[4]))
            # the tip: a point, an edge, or cusps (a molar's is a little hollow)
            dip = {'incisor': 0.0, 'canine': 0.001, 'premolar': -0.0005, 'molar': -0.001}[kind]
            V.append(root + axis * (1.0 + dip / max(np.linalg.norm(axis), 1e-6)))
            C.append(ivory * (0.95 + 0.05 * r[5]))
            for j in range(len(rings) - 1):
                for i in range(M):
                    a0, a1 = base + j * M + i, base + j * M + (i + 1) % M
                    b0, b1 = a0 + M, a1 + M
                    F += [[a0, a1, b1], [a0, b1, b0]]
            tipi = len(V) - 1
            last = base + (len(rings) - 1) * M
            for i in range(M):
                F.append([last + i, last + (i + 1) % M, tipi])
            # outward faces: flip this tooth's if its first side faces in
            Vt = np.array(V)
            f0 = F[-(len(rings) - 1) * M * 2 - M]
            nrm = np.cross(Vt[f0[1]] - Vt[f0[0]], Vt[f0[2]] - Vt[f0[0]])
            mid = root + axis * 0.15
            if nrm @ (Vt[f0[0]] - mid) < 0:
                nf = (len(rings) - 1) * M * 2 + M
                F[-nf:] = [[f[0], f[2], f[1]] for f in F[-nf:]]
    return np.array(V), F, np.array(C)
