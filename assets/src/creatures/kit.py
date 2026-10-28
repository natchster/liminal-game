"""
The creature kit: what every body script in this folder builds with.

Frames. A body is described in the GAME's frame: metres, +Y up, +Z the way
it faces, its left at +X. Blender works Z-up, so meshes and the armature
live in Blender as (x, -z, y) of the game's (x, y, z), and everything is
turned back on export (`to_game`).

A body is:

- a **skeleton** (`Skel`): bones with a parent, a head and a tail. In the
  game every bone's rest frame is axis-aligned at its head (see
  `crates/game/src/assets/bodies.rs`), so only heads and tails matter;
- **skinned meshes**, weighted to the skeleton by Blender's bone heat
  (`auto_weights`, the standard automatic weights), four bones a vertex;
- **rigid meshes** on one bone each (eyes, teeth, claws);
- **morphs**: per-vertex offsets of a mesh (a breath, a sheet's folds).

Everything is seeded and uses no `mathutils.noise` vectors (they differ
between Blender processes): noise is `noise3`, a hashed value noise in
numpy, so two builds give the same bytes.
"""
import bpy, bmesh, math, json, os, sys
import numpy as np
from mathutils import Vector, Matrix

# ---------------------------------------------------------------- frames

def to_bl(p):
    """game (x, y, z) -> Blender (x, -z, y)"""
    return (p[0], -p[2], p[1])


def to_game_np(a):
    """N x 3 Blender coordinates -> game"""
    a = np.asarray(a, dtype=np.float64).reshape(-1, 3)
    return np.stack([a[:, 0], a[:, 2], -a[:, 1]], axis=1)


def to_bl_np(a):
    a = np.asarray(a, dtype=np.float64).reshape(-1, 3)
    return np.stack([a[:, 0], -a[:, 2], a[:, 1]], axis=1)


# ---------------------------------------------------------------- noise

def _hash(ix, iy, iz, seed):
    h = (ix * 73856093) ^ (iy * 19349663) ^ (iz * 83492791) ^ (seed * 2654435761)
    h = (h ^ (h >> 13)) * 1274126177
    h = h ^ (h >> 16)
    return (h & 0xFFFFFF).astype(np.float64) / float(0xFFFFFF)


def noise3(p, freq=1.0, seed=0):
    """value noise in -1..1 at the N x 3 points `p` (smooth, repeatable)"""
    p = np.asarray(p, dtype=np.float64).reshape(-1, 3) * freq
    i = np.floor(p).astype(np.int64)
    f = p - i
    u = f * f * (3.0 - 2.0 * f)
    out = np.zeros(len(p))
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (u[:, 0] if dx else 1 - u[:, 0]) * (u[:, 1] if dy else 1 - u[:, 1]) * (u[:, 2] if dz else 1 - u[:, 2])
                out += w * _hash(i[:, 0] + dx, i[:, 1] + dy, i[:, 2] + dz, seed)
    return out * 2.0 - 1.0


def fbm(p, freq=1.0, octaves=4, seed=0, gain=0.5):
    out, amp, tot = 0.0, 1.0, 0.0
    for o in range(octaves):
        out = out + amp * noise3(p, freq * (2.0 ** o), seed + o * 101)
        tot += amp
        amp *= gain
    return out / tot


def smoothstep(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def srgb(h):
    """'#rrggbb' -> (r, g, b) in 0..1 (sRGB)"""
    h = h.lstrip('#')
    return tuple(int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4))


# ---------------------------------------------------------------- the skeleton

class Skel:
    def __init__(self):
        self.bones = []      # dicts: name, parent (index or -1), head, tail (game frame)
        self.ix = {}

    def add(self, name, parent, head, tail):
        assert name not in self.ix, name
        p = -1 if parent is None else self.ix[parent]
        self.ix[name] = len(self.bones)
        self.bones.append(dict(name=name, parent=p, head=tuple(map(float, head)), tail=tuple(map(float, tail))))
        return name

    def head(self, name):
        return np.array(self.bones[self.ix[name]]['head'])

    def tail(self, name):
        return np.array(self.bones[self.ix[name]]['tail'])

    def mirror(self, suffix_from='.L', suffix_to='.R'):
        """add a mirrored copy (x -> -x) of every bone ending in `suffix_from`"""
        for b in list(self.bones):
            if b['name'].endswith(suffix_from):
                par = self.bones[b['parent']]['name'] if b['parent'] >= 0 else None
                if par and par.endswith(suffix_from):
                    par = par[:-len(suffix_from)] + suffix_to
                m = lambda v: (-v[0], v[1], v[2])
                self.add(b['name'][:-len(suffix_from)] + suffix_to, par, m(b['head']), m(b['tail']))

    def armature(self, name='rig'):
        """a Blender armature object of this skeleton (for bone heat)"""
        arm = bpy.data.armatures.new(name)
        ao = bpy.data.objects.new(name, arm)
        bpy.context.scene.collection.objects.link(ao)
        bpy.context.view_layer.objects.active = ao
        bpy.ops.object.mode_set(mode='EDIT')
        eb = []
        for b in self.bones:
            e = arm.edit_bones.new(b['name'])
            e.head = to_bl(b['head'])
            t = np.array(b['tail'])
            if np.linalg.norm(t - np.array(b['head'])) < 1e-4:
                t = np.array(b['head']) + np.array([0, 0.01, 0])
            e.tail = to_bl(t)
            e.use_deform = b.get('deform', True)
            if b['parent'] >= 0:
                e.parent = eb[b['parent']]
            eb.append(e)
        bpy.ops.object.mode_set(mode='OBJECT')
        return ao


# ---------------------------------------------------------------- building shapes

def link(obj):
    bpy.context.scene.collection.objects.link(obj)
    return obj


def metaballs(name, elems, resolution=0.012, threshold=0.6):
    """a mesh from metaball elements, given in the GAME frame:
    ('ball', centre, radius, stiffness) | ('capsule', a, b, radius, stiffness)
    | ('ellipsoid', centre, (rx, ry, rz), stiffness) | ('-ball', centre, radius, stiffness) (subtracts)"""
    mb = bpy.data.metaballs.new(name)
    mb.resolution = resolution
    mb.render_resolution = resolution
    mb.threshold = threshold
    o = link(bpy.data.objects.new(name + '_mb', mb))
    for e in elems:
        kind = e[0]
        el = mb.elements.new()
        if kind in ('ball', '-ball'):
            _, c, r, s = e
            el.type = 'BALL'
            el.co = to_bl(c)
            el.radius = r
            el.stiffness = s
            el.use_negative = kind == '-ball'
        elif kind == 'ellipsoid':
            _, c, rr, s = e[:4]
            rot = e[4] if len(e) > 4 else None
            el.type = 'ELLIPSOID'
            el.co = to_bl(c)
            el.radius = 1.0
            el.size_x, el.size_y, el.size_z = rr[0], rr[2], rr[1]
            el.stiffness = s
            if rot is not None:
                el.rotation = rot
        elif kind == 'capsule':
            _, a, b, r, s = e
            a, b = np.array(a, float), np.array(b, float)
            el.type = 'CAPSULE'
            c = (a + b) / 2
            el.co = to_bl(c)
            el.radius = r
            el.size_x = max(1e-4, np.linalg.norm(b - a) / 2)
            el.stiffness = s
            # a capsule lies along its local X: turn X onto (b - a)
            d = Vector(to_bl(b - a)).normalized()
            el.rotation = Vector((1, 0, 0)).rotation_difference(d)
        else:
            raise ValueError(kind)
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(o.evaluated_get(dg))
    bpy.data.objects.remove(o)
    bpy.data.metaballs.remove(mb)
    me.name = name
    return link(bpy.data.objects.new(name, me))


def obj_from_bmesh(name, build):
    bm = bmesh.new()
    build(bm)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    return link(bpy.data.objects.new(name, me))


def apply_mod(o, kind, **kw):
    m = o.modifiers.new('m', kind)
    for k, v in kw.items():
        setattr(m, k, v)
    bpy.context.view_layer.objects.active = o
    for ob in bpy.context.view_layer.objects:
        ob.select_set(ob == o)
    bpy.ops.object.modifier_apply(modifier=m.name)
    return o


def remesh(o, voxel):
    return apply_mod(o, 'REMESH', mode='VOXEL', voxel_size=voxel, use_smooth_shade=True)


def decimate(o, tris):
    n = sum(len(p.vertices) - 2 for p in o.data.polygons)
    if n > tris:
        apply_mod(o, 'DECIMATE', decimate_type='COLLAPSE', ratio=tris / n, use_collapse_triangulate=True)
    return o


def triangulate(o):
    return apply_mod(o, 'TRIANGULATE')


def smooth(o, iterations=4, factor=0.5):
    return apply_mod(o, 'SMOOTH', factor=factor, iterations=iterations)


def join(objs, name):
    for ob in bpy.context.view_layer.objects:
        ob.select_set(False)
    for ob in objs:
        ob.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    objs[0].name = name
    return objs[0]


def verts_np(o):
    n = len(o.data.vertices)
    a = np.empty(n * 3)
    o.data.vertices.foreach_get('co', a)
    return a.reshape(-1, 3)


def set_verts_np(o, a):
    o.data.vertices.foreach_set('co', np.asarray(a, dtype=np.float64).reshape(-1))
    o.data.update()


def displace(o, fn):
    """move each vertex (game frame) by `fn(P, N) -> offsets` (N x 3, game frame)"""
    o.data.calc_normals_split() if hasattr(o.data, 'calc_normals_split') else None
    P = to_game_np(verts_np(o))
    N = to_game_np(vnormals(o))
    set_verts_np(o, to_bl_np(P + fn(P, N)))


def vnormals(o):
    n = len(o.data.vertices)
    a = np.empty(n * 3)
    o.data.vertices.foreach_get('normal', a)
    return a.reshape(-1, 3)


# ---------------------------------------------------------------- weights

def auto_weights(o, skel, arm=None, keep=False):
    """bone heat weights (Blender's automatic weights) -> (joints, weights) N x 4,
    weights as bytes summing to 255; bones with use_deform False get none.
    With `keep`, the mesh stays parented to `arm` with its armature modifier
    (to pose it) and nothing is returned."""
    ao = arm or skel.armature()
    for ob in bpy.context.view_layer.objects:
        ob.select_set(False)
    o.select_set(True)
    ao.select_set(True)
    bpy.context.view_layer.objects.active = ao
    r = bpy.ops.object.parent_set(type='ARMATURE_AUTO')
    assert 'FINISHED' in r, r
    if keep:
        return None
    gi = {g.index: skel.ix[g.name] for g in o.vertex_groups}
    n = len(o.data.vertices)
    J = np.zeros((n, 4), dtype=np.int64)
    W = np.zeros((n, 4), dtype=np.int64)
    unweighted = 0
    for v in o.data.vertices:
        gs = sorted(((g.weight, gi[g.group]) for g in v.groups if g.weight > 1e-4), reverse=True)[:4]
        if not gs:
            unweighted += 1
            gs = [(1.0, nearest_bone(skel, to_game_np(v.co)[0]))]
        tot = sum(w for w, _ in gs)
        q = [int(round(w / tot * 255)) for w, _ in gs]
        q[0] += 255 - sum(q)
        for k, (w, b) in enumerate(gs):
            J[v.index, k] = b
            W[v.index, k] = q[k]
    # the parenting is only for the weights: take the mesh back out
    o.parent = None
    for m in list(o.modifiers):
        o.modifiers.remove(m)
    if arm is None:
        bpy.data.objects.remove(ao)
    return J, W, unweighted


def seg_dist(P, a, b):
    a, b = np.asarray(a, float), np.asarray(b, float)
    d = b - a
    L2 = max(float(d @ d), 1e-12)
    t = np.clip(((P - a) @ d) / L2, 0.0, 1.0)
    return np.linalg.norm(P - (a + t[:, None] * d), axis=1)


def nearest_bone(skel, p):
    P = np.asarray(p, float).reshape(1, 3)
    ds = [seg_dist(P, b['head'], b['tail'])[0] if b.get('deform', True) else 1e9 for b in skel.bones]
    return int(np.argmin(ds))


# ---------------------------------------------------------------- shading helpers

def cavity(o):
    """per-vertex 0 (convex) .. 1 (a crease), from the neighbours (game frame)"""
    P = verts_np(o)
    N = vnormals(o)
    n = len(P)
    acc = np.zeros(n)
    cnt = np.zeros(n)
    E = np.empty(len(o.data.edges) * 2, dtype=np.int64)
    o.data.edges.foreach_get('vertices', E)
    E = E.reshape(-1, 2)
    for a, b in ((E[:, 0], E[:, 1]), (E[:, 1], E[:, 0])):
        d = P[b] - P[a]
        l = np.linalg.norm(d, axis=1) + 1e-9
        np.add.at(acc, a, np.einsum('ij,ij->i', N[a], d) / l)
        np.add.at(cnt, a, 1)
    c = np.clip(acc / np.maximum(cnt, 1) * 4.0, 0.0, 1.0)
    # blur once over the edges
    s = np.zeros(n)
    np.add.at(s, E[:, 0], c[E[:, 1]])
    np.add.at(s, E[:, 1], c[E[:, 0]])
    k = np.zeros(n)
    np.add.at(k, E[:, 0], 1)
    np.add.at(k, E[:, 1], 1)
    return 0.5 * c + 0.5 * s / np.maximum(k, 1)


# ---------------------------------------------------------------- export

def mesh_arrays(o):
    """triangulated positions, normals (game frame) and indices of `o`"""
    triangulate(o)
    me = o.data
    P = to_game_np(verts_np(o))
    N = to_game_np(vnormals(o))
    I = np.empty(len(me.polygons) * 3, dtype=np.int64)
    me.polygons.foreach_get('vertices', I)
    return P, N, I


def r5(a):
    return [round(float(x), 5) for x in np.asarray(a).reshape(-1)]


def paint_preview(o, colors):
    """write sRGB per-vertex colours onto `o` so the Workbench preview shows them"""
    if colors is None:
        return
    ca = o.data.color_attributes.new('preview', 'BYTE_COLOR', 'POINT')
    c = np.ones((len(o.data.vertices), 4))
    c[:, :3] = np.asarray(colors, float).reshape(-1, 3)
    ca.data.foreach_set('color_srgb', c.reshape(-1))
    o.data.color_attributes.active_color = ca


class Body:
    """what one body script returns: the skeleton and its meshes"""

    def __init__(self, name, skel):
        self.name, self.skel = name, skel
        self.meshes = []

    def skinned(self, name, material, o, colors=None, morphs=None, uv=None):
        """`o`: the finished mesh object; colors: N x 3 sRGB 0..1 (per vertex);
        morphs: [(name, N x 3 offsets in the game frame)]"""
        J, W, un = auto_weights(o, self.skel)
        paint_preview(o, colors)
        P, N, I = mesh_arrays(o)
        assert len(P) == len(J), 'triangulating changed the vertex count'
        self.meshes.append(dict(name=name, material=material, bone=-1, pos=r5(P), nrm=r5(N),
                                col=r5(colors) if colors is not None else None, uv=r5(uv) if uv is not None else None,
                                jnt=[int(x) for x in J.reshape(-1)], wgt=[int(x) for x in W.reshape(-1)],
                                idx=[int(x) for x in I], morphs=[dict(name=k, pos=r5(d), nrm=None) for k, d in (morphs or [])]))
        print(f'[creatures] {self.name}/{name}: {len(P)} verts, {len(I) // 3} tris, skinned'
              + (f', {un} verts fell back to the nearest bone' if un else ''))
        return P

    def skinned_by(self, name, material, o, weights, colors=None, morphs=None, uv=None):
        """a mesh skinned by weights the script computes: `weights(P) -> (J, W)`
        with J bone indices and W weights summing to 1, N x 4 (game frame);
        morphs: [(name, N x 3 offsets[, N x 3 normal offsets])]"""
        paint_preview(o, colors)
        P, N, I = mesh_arrays(o)
        J, Wf = weights(P)
        W = np.round(np.asarray(Wf) * 255).astype(np.int64)
        W[:, 0] += 255 - W.sum(axis=1)
        ks = [dict(name=k[0], pos=r5(k[1]), nrm=r5(k[2]) if len(k) > 2 else None) for k in (morphs or [])]
        self.meshes.append(dict(name=name, material=material, bone=-1, pos=r5(P), nrm=r5(N),
                                col=r5(colors) if colors is not None else None, uv=r5(uv) if uv is not None else None,
                                jnt=[int(x) for x in np.asarray(J).reshape(-1)], wgt=[int(x) for x in W.reshape(-1)],
                                idx=[int(x) for x in I], morphs=ks))
        print(f'[creatures] {self.name}/{name}: {len(P)} verts, {len(I) // 3} tris, skinned by the script')
        return P

    def rigid(self, name, material, bone, o, colors=None):
        paint_preview(o, colors)
        P, N, I = mesh_arrays(o)
        self.meshes.append(dict(name=name, material=material, bone=self.skel.ix[bone], pos=r5(P), nrm=r5(N),
                                col=r5(colors) if colors is not None else None, uv=None, jnt=None, wgt=None,
                                idx=[int(x) for x in I], morphs=[]))
        print(f'[creatures] {self.name}/{name}: {len(P)} verts, {len(I) // 3} tris, on {bone}')
        return P

    def write(self, out):
        d = dict(name=self.name, bones=[dict(name=b['name'], parent=b['parent'], head=r5(b['head']), tail=r5(b['tail']))
                                        for b in self.skel.bones], meshes=self.meshes)
        with open(os.path.join(out, self.name + '.json'), 'w') as f:
            json.dump(d, f, separators=(',', ':'))


def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def preview(objs, path, height=None, views=((0, 'front'), (90, 'side'), (180, 'back'), (30, 'three-quarter')), size=(360, 440)):
    """Workbench views (vertex colours) framing the given objects, one PNG a view"""
    sc = bpy.context.scene
    sc.render.engine = 'BLENDER_WORKBENCH'
    sc.display.shading.light = 'STUDIO'
    sc.display.shading.color_type = 'VERTEX'
    sc.display.shading.show_cavity = False
    pts = np.concatenate([np.array([tuple(o.matrix_world @ v.co) for v in o.data.vertices]) for o in objs])
    lo, hi = pts.min(0), pts.max(0)
    centre = (lo + hi) / 2
    span = max(hi[2] - lo[2], (hi[0] - lo[0]) * size[1] / size[0], (hi[1] - lo[1]) * size[1] / size[0]) if height is None else height
    cam = link(bpy.data.objects.new('cam', bpy.data.cameras.new('cam')))
    cam.data.type = 'ORTHO'
    cam.data.ortho_scale = span * 1.08 * max(1.0, size[0] / size[1])
    sc.camera = cam
    sc.render.resolution_x, sc.render.resolution_y = size
    files = []
    for deg, tag in views:
        a = math.radians(deg)
        # the body faces -Y in Blender; walk the camera round it
        cam.location = (centre[0] + math.sin(a) * 8.0, centre[1] - math.cos(a) * 8.0, centre[2])
        cam.rotation_euler = (math.pi / 2, 0, a)
        f = f'{path}.{tag}.png'
        sc.render.filepath = f
        bpy.ops.render.render(write_still=True)
        files.append(f)
    bpy.data.objects.remove(cam)
    return files
