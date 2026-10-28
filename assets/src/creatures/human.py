"""
People's bodies, made into the bodies of things: MakeHuman's base (hm08,
CC0) as a mesh, its joints as a standard humanoid skeleton, and a way to
pose and stretch it so that the pose becomes the rest pose the game binds.

    base = hm08.Base().target('macro/male-old-thin')
    o, skel = human.body(kit, base)                       # the standing body and its skeleton
    posed = human.pose(kit, o, skel, {'spine': (0, 0.3, 1), ...}, stretch={'forearm.L': 1.3})

`pose` weights the body to the standing skeleton by bone heat, turns each
named bone to point along a direction (game frame), scales the ones named
in `stretch` along their length, applies the deformation to the mesh, and
returns the posed skeleton: heads and tails where the posed bones are. The
body is then remeshed and weighted again to that skeleton (kit.Body), so the
creature is bound in the pose it stands in.

Bone names (".L" is the body's left, +X; mirrored to ".R"):
hips, spine, chest, upperchest, neck, head, jaw,
clavicle, upperarm, forearm, hand, finger{1..5}{1..3} (1 the thumb),
thigh, shin, foot, toes.
"""
import numpy as np
import bpy
from mathutils import Matrix, Vector

SIDES = (('.L', 'l-'), ('.R', 'r-'))


def skeleton(kit, base):
    """the standard humanoid skeleton at the base's joints (game frame)"""
    j = base.joint
    s = kit.Skel()
    s.add('hips', None, j('pelvis'), j('spine-4'))
    s.add('spine', 'hips', j('spine-4'), j('spine-2'))
    s.add('chest', 'spine', j('spine-2'), j('spine-1'))
    s.add('upperchest', 'chest', j('spine-1'), j('neck'))
    s.add('neck', 'upperchest', j('neck'), j('head'))
    s.add('head', 'neck', j('head'), j('head-2'))
    hinge = j('head') + np.array([0, -0.01, 0.03])
    s.add('jaw', 'head', hinge, j('jaw'))
    for suf, mh in SIDES:
        s.add('clavicle' + suf, 'upperchest', j(mh + 'clavicle'), j(mh + 'shoulder'))
        s.add('upperarm' + suf, 'clavicle' + suf, j(mh + 'shoulder'), j(mh + 'elbow'))
        s.add('forearm' + suf, 'upperarm' + suf, j(mh + 'elbow'), j(mh + 'hand'))
        s.add('hand' + suf, 'forearm' + suf, j(mh + 'hand'), j(mh + 'finger-3-1'))
        for f in range(1, 6):
            par = 'hand' + suf
            for k in range(1, 4):
                name = f'finger{f}{k}{suf}'
                s.add(name, par, j(f'{mh}finger-{f}-{k}'), j(f'{mh}finger-{f}-{k + 1}'))
                par = name
        s.add('thigh' + suf, 'hips', j(mh + 'upper-leg'), j(mh + 'knee'))
        s.add('shin' + suf, 'thigh' + suf, j(mh + 'knee'), j(mh + 'ankle'))
        s.add('foot' + suf, 'shin' + suf, j(mh + 'ankle'), j(mh + 'foot-1'))
        s.add('toes' + suf, 'foot' + suf, j(mh + 'foot-1'), j(mh + 'foot-2'))
    return s


def body(kit, base, name='body'):
    """the base's skin as a Blender object (quads) and its skeleton"""
    import bpy
    P = base.verts()
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(p) for p in kit.to_bl_np(P)], [], base.faces.tolist())
    me.update()
    for p in me.polygons:
        p.use_smooth = True
    o = kit.link(bpy.data.objects.new(name, me))
    return o, skeleton(kit, base)


def _unit(v):
    v = np.asarray(v, float)
    n = np.linalg.norm(v)
    return v / n if n > 1e-12 else v


def fk(skel, aims=None, stretch=None, root=None):
    """the posed skeleton: every bone keeps its length (times `stretch`) and
    either points along `aims[name]` (game frame) or keeps its rest angle to
    its parent; children ride on their parents. `root` moves the first bone's
    head. Returns a new Skel."""
    import kit as K
    aims = aims or {}
    stretch = stretch or {}
    out = K.Skel()
    rot = []          # each bone's rotation from rest (3x3, game frame)
    for i, b in enumerate(skel.bones):
        h0, t0 = np.array(b['head']), np.array(b['tail'])
        d0 = _unit(t0 - h0)
        L = np.linalg.norm(t0 - h0) * stretch.get(b['name'], 1.0)
        p = b['parent']
        if p < 0:
            head = np.array(root) if root is not None else h0
            prot = np.eye(3)
        else:
            pb, pr = skel.bones[p], rot[p]
            ph0 = np.array(pb['head'])
            # the offset from the parent's rest head, turned with it, from its posed head;
            # a child at its parent's tail stays at the (stretched) tail
            off = h0 - ph0
            ptail0 = np.array(pb['tail'])
            if np.linalg.norm(h0 - ptail0) < 1e-4:
                head = np.array(out.bones[p]['tail'])
            else:
                head = np.array(out.bones[p]['head']) + pr @ off
            prot = pr
        if b['name'] in aims:
            d = _unit(aims[b['name']])
            r = _rot_between(prot @ d0, d) @ prot
        else:
            r = prot
            d = prot @ d0
        rot.append(r)
        par = skel.bones[p]['name'] if p >= 0 else None
        out.add(b['name'], par, head, head + d * L)
    return out


def _rot_between(a, b):
    a, b = _unit(a), _unit(b)
    v = np.cross(a, b)
    c = float(a @ b)
    if c < -0.999999:
        # opposite: turn about any axis across a
        ax = _unit(np.cross(a, [1, 0, 0] if abs(a[0]) < 0.9 else [0, 1, 0]))
        return 2 * np.outer(ax, ax) - np.eye(3)
    vx = np.array([[0, -v[2], v[1]], [v[2], 0, -v[0]], [-v[1], v[0], 0]])
    return np.eye(3) + vx + vx @ vx / (1 + c)


def pose(kit, o, skel, aims=None, stretch=None, root=None):
    """deform `o` from `skel` (its rest) into the posed skeleton and apply it;
    returns the posed skeleton (kit.Skel)"""
    posed = fk(skel, aims, stretch, root)
    arm = skel.armature('pose_rig')
    kit.auto_weights(o, skel, arm=arm, keep=True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode='POSE')
    for i, b in enumerate(skel.bones):
        pb = arm.pose.bones[b['name']]
        rest = arm.data.bones[b['name']].matrix_local          # armature space (Blender frame)
        h0, t0 = Vector(kit.to_bl(b['head'])), Vector(kit.to_bl(b['tail']))
        q = posed.bones[i]
        h1, t1 = Vector(kit.to_bl(q['head'])), Vector(kit.to_bl(q['tail']))
        d0, d1 = (t0 - h0).normalized(), (t1 - h1).normalized()
        delta = d0.rotation_difference(d1).to_matrix().to_4x4()
        rot = delta @ rest.to_3x3().to_4x4()
        s = (t1 - h1).length / max((t0 - h0).length, 1e-9)
        m = Matrix.Translation(h1) @ rot @ Matrix.Diagonal((1.0, s, 1.0, 1.0))
        pb.matrix = m
        bpy.context.view_layer.update()
    bpy.ops.object.mode_set(mode='OBJECT')
    # bake the deformation into the mesh
    for ob in bpy.context.view_layer.objects:
        ob.select_set(ob == o)
    bpy.context.view_layer.objects.active = o
    mod = [m for m in o.modifiers if m.type == 'ARMATURE'][0]
    bpy.ops.object.modifier_apply(modifier=mod.name)
    o.parent = None
    o.vertex_groups.clear()
    bpy.data.objects.remove(arm)
    return posed
