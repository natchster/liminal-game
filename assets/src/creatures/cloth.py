"""
Cloth draped by Blender's cloth solver, headless: a sheet laid over a body
(or a piece of furniture), pinned where it rests, let fall and settle, and
read back. Every drape of one sheet starts from the same grid, so two drapes
have the same vertices in the same order: one pose is a morph of another.

The solver runs a fixed number of frames with fixed settings and no
randomness; `drape` returns the settled positions in the game frame.
"""
import numpy as np
import bpy


def spaced(length, fine, step, coarse, grow=0.35):
    """positions across a sheet `length` m long (its middle at 0): `step`
    apart over the span `fine` = (lo, hi), growing to `coarse` apart over
    `grow` m beyond it, and `coarse` apart out to the edges"""
    lo, hi = fine
    xs = list(np.arange(lo, hi + step * 0.5, step))
    for sign, start in ((1, xs[-1]), (-1, xs[0])):
        x, d = start, step
        while True:
            t = min(abs(x - start) / grow, 1.0)
            d = step + (coarse - step) * t
            x = x + sign * d
            if abs(x) > length / 2:
                break
            xs.append(x)
    return np.array(sorted(xs))


def grid(kit, name, width, length, nx, ny, centre, dense=0.35, xs=None, zs=None):
    """a flat sheet `width` x `length` m of nx x ny vertices, level, its
    middle at `centre` (game frame), denser toward its middle (`dense`: the
    share of even spacing; the rest grows toward the edges, where the cloth
    only hangs), or at the positions `xs` / `zs` across and along it (see
    `spaced`). Faces look up. Returns the object and its UVs (metres of
    cloth, for the weave)."""
    def axis(n, L):
        s = np.linspace(-1.0, 1.0, n)
        return L / 2 * (dense * s + (1 - dense) * np.sign(s) * s * s)
    xs = axis(nx, width) if xs is None else xs
    zs = axis(ny, length) if zs is None else zs
    nx, ny = len(xs), len(zs)
    V = np.array([(centre[0] + x, centre[1], centre[2] + z) for z in zs for x in xs])
    F = [(j * nx + i, (j + 1) * nx + i, (j + 1) * nx + i + 1, j * nx + i + 1)
         for j in range(ny - 1) for i in range(nx - 1)]
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(p) for p in kit.to_bl_np(V)], [], F)
    me.update()
    o = kit.link(bpy.data.objects.new(name, me))
    uv = np.array([(x, z) for z in zs for x in xs])
    return o, uv


def collider(o, thickness=0.008, friction=10.0):
    """make `o` something cloth rests on"""
    bpy.ops.object.select_all(action='DESELECT')
    o.select_set(True)
    bpy.context.view_layer.objects.active = o
    bpy.ops.object.modifier_add(type='COLLISION')
    o.collision.thickness_outer = thickness
    o.collision.cloth_friction = friction


def floor(kit, size=8.0):
    """a floor the cloth pools on (remove it after)"""
    bpy.ops.mesh.primitive_plane_add(size=size, location=(0, 0, 0))
    f = bpy.context.active_object
    collider(f, 0.004, 20.0)
    return f


def pin(o, indices, group='pin'):
    """pin these vertices of `o` where they are (replacing earlier pins)"""
    if group in o.vertex_groups:
        o.vertex_groups.remove(o.vertex_groups[group])
    g = o.vertex_groups.new(name=group)
    for i in indices:
        g.add([int(i)], 1.0, 'REPLACE')


def settle(kit, o, frames, mass=0.25, stiffness=12.0, bending=0.4, self_collide=True, group='pin', gap=0.006):
    """run the solver `frames` frames on `o` (pinned by `group`), keeping
    `gap` m off what it rests on, and apply it"""
    cl = o.modifiers.new('cloth', 'CLOTH')
    s = cl.settings
    s.quality = 8
    s.mass = mass
    s.tension_stiffness = stiffness
    s.compression_stiffness = stiffness
    s.bending_stiffness = bending
    s.air_damping = 1.5
    if group in o.vertex_groups:
        s.vertex_group_mass = group
    c = cl.collision_settings
    c.use_self_collision = self_collide
    c.self_distance_min = 0.006
    c.distance_min = gap
    c.collision_quality = 6
    sc = bpy.context.scene
    sc.frame_start, sc.frame_end = 1, frames
    cl.point_cache.frame_end = frames
    for f in range(1, frames + 1):
        sc.frame_set(f)
    bpy.ops.object.select_all(action='DESELECT')
    o.select_set(True)
    bpy.context.view_layer.objects.active = o
    bpy.ops.object.modifier_apply(modifier=cl.name)
    sc.frame_set(1)
    return kit.to_game_np(kit.verts_np(o))


def nearest(P):
    """a KD-tree over points P (game frame): `.find(p)` -> (co, index, dist)"""
    from mathutils import kdtree
    t = kdtree.KDTree(len(P))
    for i, p in enumerate(P):
        t.insert(p, i)
    t.balance()
    return t


def off_the_skin(S, tree, N, gap=0.003, reach=0.03):
    """anything of the cloth under the skin (points S over a body whose
    points `tree` indexes, normals N) goes back out to `gap` over it"""
    S = S.copy()
    moved = 0
    for i in range(len(S)):
        q, j, d = tree.find(S[i])
        if d < reach and (S[i] - np.array(q)) @ N[j] < gap * 0.7:
            S[i] = np.array(q) + N[j] * gap
            moved += 1
    return S, moved


def ease_to(o, S, targets, held, iters=80):
    """move points S of `o` so `targets` ({index: position}) are reached
    and `held` stay put, the rest of the cloth following smoothly (each
    point the mean of its neighbours' moves): no edge is torn long at once,
    which the solver would answer by flying apart"""
    E = np.zeros(len(o.data.edges) * 2, dtype=np.int64)
    o.data.edges.foreach_get('vertices', E)
    E = E.reshape(-1, 2)
    deg = np.maximum(np.bincount(E.ravel(), minlength=len(S)), 1).astype(float)[:, None]
    D = np.zeros_like(S)
    fixed = np.zeros(len(S), bool)
    for i, p in targets.items():
        D[i] = np.asarray(p) - S[i]
        fixed[i] = True
    fixed[list(held)] = True
    for _ in range(iters):
        acc = np.zeros_like(D)
        np.add.at(acc, E[:, 0], D[E[:, 1]])
        np.add.at(acc, E[:, 1], D[E[:, 0]])
        D = np.where(fixed[:, None], D, acc / deg)
    return S + D


def smooth(o, S, iters, factor):
    """points S of `o` relaxed toward their neighbours' mean (Laplacian)"""
    E = np.zeros(len(o.data.edges) * 2, dtype=np.int64)
    o.data.edges.foreach_get('vertices', E)
    E = E.reshape(-1, 2)
    deg = np.maximum(np.bincount(E.ravel(), minlength=len(S)), 1).astype(float)[:, None]
    S = S.copy()
    for _ in range(iters):
        acc = np.zeros_like(S)
        np.add.at(acc, E[:, 0], S[E[:, 1]])
        np.add.at(acc, E[:, 1], S[E[:, 0]])
        S = S + factor * (acc / deg - S)
    return S


def refine(S, nx, ny, k):
    """a drape on a regular nx x ny grid (points S, row-major along the
    sheet), made k times finer along each side by a uniform cubic B-spline
    over the points (the smooth surface through them, as Catmull-Clark's
    limit is): ((ny-1)k+1) x ((nx-1)k+1) points, row-major"""
    G = S.reshape(ny, nx, 3)
    # pad by repeating the edge so the spline reaches the sheet's edge
    G = np.concatenate([2 * G[:1] - G[1:2], G, 2 * G[-1:] - G[-2:-1]], axis=0)
    G = np.concatenate([2 * G[:, :1] - G[:, 1:2], G, 2 * G[:, -1:] - G[:, -2:-1]], axis=1)

    def basis(t):
        return np.stack([(1 - t) ** 3, 3 * t ** 3 - 6 * t ** 2 + 4, -3 * t ** 3 + 3 * t ** 2 + 3 * t + 1, t ** 3], axis=-1) / 6.0

    def along(n):
        # fine parameter positions -> (index of the first of 4 controls, weights)
        u = np.linspace(0, n - 1, (n - 1) * k + 1)
        i = np.minimum(np.floor(u).astype(int), n - 2)
        return i, basis(u - i)
    iz, wz = along(ny)
    ix, wx = along(nx)
    # rows first: for each fine z, blend 4 padded rows
    R = np.einsum('zk,zkxc->zxc', wz, np.stack([G[iz + a] for a in range(4)], axis=1))
    F = np.einsum('xk,zxkc->zxc', wx, np.stack([R[:, ix + a] for a in range(4)], axis=2))
    return F.reshape(-1, 3), F.shape[1], F.shape[0]


def sample(F, fx, fz, xs, zs, uv):
    """points of a regular grid F (fz rows of fx, row-major, at flat-sheet
    positions xs across and zs along) at flat positions `uv` (bilinear)"""
    G = F.reshape(fz, fx, 3)
    u = np.interp(uv[:, 0], xs, np.arange(fx))
    v = np.interp(uv[:, 1], zs, np.arange(fz))
    i0 = np.clip(np.floor(u).astype(int), 0, fx - 2)
    j0 = np.clip(np.floor(v).astype(int), 0, fz - 2)
    a, b = (u - i0)[:, None], (v - j0)[:, None]
    return (G[j0, i0] * (1 - a) * (1 - b) + G[j0, i0 + 1] * a * (1 - b)
            + G[j0 + 1, i0] * (1 - a) * b + G[j0 + 1, i0 + 1] * a * b)
