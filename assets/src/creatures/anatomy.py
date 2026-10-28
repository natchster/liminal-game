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


def waste(P, skel, bones, amount, joint_keep=0.06):
    """pull points nearest each of `bones` toward its segment by `amount`
    (0..1), less near its ends (the joints stay knobby)"""
    nb, _ = nearest_bones(P, skel)
    out = P.copy()
    for i, b in enumerate(skel.bones):
        if b['name'] not in bones:
            continue
        m = nb == i
        if not m.any():
            continue
        t, dist, r = seg_param(P[m], b['head'], b['tail'])
        L = np.linalg.norm(np.array(b['tail']) - np.array(b['head']))
        e = np.minimum(t * L, (1 - t) * L)                     # distance from the nearer joint
        k = amount * np.clip(e / joint_keep, 0.0, 1.0)
        out[m] = P[m] - r * k[:, None]
    return out


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
