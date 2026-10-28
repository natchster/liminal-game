"""
Furniture under dust sheets: a chair, a table, a wingback armchair, each
built plainly (it is never seen), a sheet of the Watcher's linen thrown
over it by Blender's cloth (`cloth.py`) and let settle. They stand where
the Watcher stands (`entities::watcher`); at a glance one is the other.
The sheets are one layer, drawn from both sides, like the Watcher's.
"""
import numpy as np

# name: (sheet width, length (m); what is under it: boxes (centre x, y, z; size x, y, z))
PIECES = {
    'chair': (2.0, 2.0, [
        (0, 0.43, 0, 0.46, 0.05, 0.46),                     # the seat
        (0, 0.72, -0.21, 0.44, 0.56, 0.04),                 # its back
    ] + [(sx * 0.2, 0.21, sz * 0.2, 0.04, 0.42, 0.04) for sx in (-1, 1) for sz in (-1, 1)]),
    'table': (2.4, 1.9, [
        (0, 0.73, 0, 1.10, 0.04, 0.70),                     # the top
    ] + [(sx * 0.5, 0.36, sz * 0.3, 0.05, 0.72, 0.05) for sx in (-1, 1) for sz in (-1, 1)]),
    'wingback': (2.4, 2.4, [
        (0, 0.36, 0.02, 0.62, 0.20, 0.56),                  # the seat
        (0, 0.78, -0.24, 0.62, 0.72, 0.12),                 # its back
        (0.27, 0.56, 0.02, 0.10, 0.22, 0.54),               # the arms
        (-0.27, 0.56, 0.02, 0.10, 0.22, 0.54),
        (0.26, 0.98, -0.12, 0.12, 0.34, 0.26),              # the wings
        (-0.26, 0.98, -0.12, 0.12, 0.34, 0.26),
    ]),
}
CELL = 0.04          # the solver's cloth, m
TRIS = 3200          # each piece in the game


def build(kit, name):
    import bpy
    import cloth, linen
    w, l, parts = PIECES[name]
    kit.clear_scene()
    # what is under the sheet, as one solid
    objs = []
    for (x, y, z, sx, sy, sz) in parts:
        bpy.ops.mesh.primitive_cube_add(size=1.0, location=kit.to_bl((x, y, z)))
        o = bpy.context.active_object
        o.scale = kit.to_bl((sx, sy, sz))
        o.scale = (abs(o.scale[0]), abs(o.scale[1]), abs(o.scale[2]))
        bpy.ops.object.transform_apply(scale=True)
        objs.append(o)
    bpy.ops.object.select_all(action='DESELECT')
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    under = bpy.context.active_object
    # rounded a little (bevelled edges catch the cloth like upholstery)
    bev = under.modifiers.new('bev', 'BEVEL')
    bev.width = 0.02
    bev.segments = 2
    bpy.ops.object.modifier_apply(modifier='bev')
    cloth.collider(under, 0.006, 12.0)
    fl = cloth.floor(kit)
    P = kit.to_game_np(kit.verts_np(under))
    top = P[np.argmax(P[:, 1])]
    nx, ny = int(round(w / CELL)) + 1, int(round(l / CELL)) + 1
    sh, _ = cloth.grid(kit, 'sheet', w, l, nx, ny, (0.0, P[:, 1].max() + 0.02, 0.0), dense=1.0)
    # thrown over and lying where it fell: its middle held over the top
    G = kit.to_game_np(kit.verts_np(sh))
    cloth.pin(sh, np.nonzero(np.hypot(G[:, 0], G[:, 2]) < 0.06)[0])
    S = cloth.settle(kit, sh, 80)
    bpy.data.objects.remove(fl)
    F, fx, fz = cloth.refine(S, nx, ny, 2)
    # the game's mesh: the fine drape thinned
    kit.clear_scene()
    fxs, fzs = np.linspace(-w / 2, w / 2, fx), np.linspace(-l / 2, l / 2, fz)
    Fq = [(j * fx + i, (j + 1) * fx + i, (j + 1) * fx + i + 1, j * fx + i + 1) for j in range(fz - 1) for i in range(fx - 1)]
    me = bpy.data.meshes.new('sheet_' + name)
    me.from_pydata([tuple(p) for p in kit.to_bl_np(F)], [], Fq)
    me.update()
    o = kit.link(bpy.data.objects.new('sheet_' + name, me))
    uvf = np.array([(x, z) for z in fzs for x in fxs])
    lay = me.uv_layers.new(name='cloth')
    loops = np.empty(len(me.loops), dtype=np.int64)
    me.loops.foreach_get('vertex_index', loops)
    lay.data.foreach_set('uv', uvf[loops].reshape(-1))
    bpy.ops.object.select_all(action='DESELECT')
    o.select_set(True)
    bpy.context.view_layer.objects.active = o
    dec = o.modifiers.new('thin', 'DECIMATE')
    dec.ratio = TRIS / (2.0 * len(Fq))
    bpy.ops.object.modifier_apply(modifier='thin')
    kit.triangulate(o)
    me = o.data
    lu = np.empty(len(me.loops) * 2)
    me.uv_layers['cloth'].data.foreach_get('uv', lu)
    lv = np.empty(len(me.loops), dtype=np.int64)
    me.loops.foreach_get('vertex_index', lv)
    uv = np.zeros((len(me.vertices), 2))
    uv[lv] = lu.reshape(-1, 2)
    for p in me.polygons:
        p.use_smooth = True
    Pf = kit.to_game_np(kit.verts_np(o))
    col = linen.colours(kit, o, uv, Pf[:, 1])
    skel = kit.Skel()
    skel.add('root', None, (0, 0, 0), (0, 1.0, 0))
    body = kit.Body('sheet_' + name, skel)
    one = lambda P: (np.zeros((len(P), 4), dtype=np.int64), np.tile([1.0, 0, 0, 0], (len(P), 1)))
    body.skinned_by('sheet', 'sheet', o, one, colors=col, uv=uv)
    return body
