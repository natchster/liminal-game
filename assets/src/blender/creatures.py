"""
Organic creature parts for the Backrooms entities, generated in Blender.

    /Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
        --python tools/blender/creatures.py -- src/creature-meshes.js [partA,partB]

The brief is docs/ENTITY_REFERENCES.md (sizes in metres, colours, anatomy).
With a second argument only the named parts are rebuilt; the others are kept
from the existing output file.

Solid parts are unions of ellipsoids, tapered tubes and swept tubes fused by
a voxel remesh, carved with exact booleans (eye sockets, the Smiler's grin
slot, the jaw split, the Face's creases), smoothed, roughened with a little
noise and decimated to a triangle budget. Hair, teeth and the Watcher's sheet
are built directly (swept tubes, blades, a lofted drape with analytic folds).

Frames: three.js, +Y up, +Z the way the creature faces, metres. Limbs are
modelled in their joint's frame (joint at the origin, the bone down -Y), and
heads in their rig's head frame, so the rigs in backrooms.js place every part
with no fitting. The numbers the rigs need (eye, hinge and ear positions) are
printed at the end of a run and copied into makeSmiler / makeHound /
makeWatcher.

Colours: a part with a painter gets per-vertex sRGB albedo (uint8, base64),
decoded to linear by src/creatures.js; its material is then white.
Output: int16 positions quantized to each part's bounding box, uint16
triangle indices, base64, one line per part.
"""
import bpy, bmesh, sys, math, base64, struct, random, re, os
from mathutils import Matrix, Vector, Euler, noise as mnoise

ARGS = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
OUT = ARGS[0] if ARGS else 'creature-meshes.js'
ONLY = set(ARGS[1].split(',')) if len(ARGS) > 1 and ARGS[1] else None
V = Vector
TAU = math.tau


def clamp(x, a=0.0, b=1.0): return max(a, min(b, x))
def lerp(a, b, t): return a + (b - a) * t
def smooth(e0, e1, x):
    t = clamp((x - e0) / (e1 - e0))
    return t * t * (3 - 2 * t)
def mix3(c0, c1, t): return tuple(lerp(c0[i], c1[i], t) for i in range(3))
def hexc(h):
    h = h.lstrip('#')
    return tuple(int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))
def mul3(c, k): return tuple(clamp(x * k) for x in c)
def interp(tab, x):
    """piecewise-linear lookup in [(x, value...)] sorted by x"""
    if x <= tab[0][0]: return tab[0][1:]
    for i in range(len(tab) - 1):
        a, b = tab[i], tab[i + 1]
        if x <= b[0]:
            t = (x - a[0]) / (b[0] - a[0])
            return tuple(lerp(a[k], b[k], t) for k in range(1, len(a)))
    return tab[-1][1:]


# ------------------------------------------------------------ primitives ---
def M(pos=(0, 0, 0), rot=(0, 0, 0), scale=(1, 1, 1)):
    return Matrix.Translation(V(pos)) @ Euler(rot, 'XYZ').to_matrix().to_4x4() @ Matrix.Diagonal(V((*scale, 1)))

def ell(bm, c, r, rot=(0, 0, 0), seg=20):
    """ellipsoid centre c, radii r"""
    bmesh.ops.create_uvsphere(bm, u_segments=seg, v_segments=max(8, seg // 2), radius=1.0, matrix=M(c, rot, r))

def tube(bm, a, b, ra, rb, seg=16):
    """tapered cylinder from a (radius ra) to b (radius rb), capped with spheres"""
    a, b = V(a), V(b)
    d = b - a
    q = V((0, 0, 1)).rotation_difference(d.normalized())
    mat = Matrix.Translation((a + b) / 2) @ q.to_matrix().to_4x4()
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=seg, radius1=ra, radius2=rb, depth=d.length, matrix=mat)
    ell(bm, a, (ra, ra, ra), seg=seg)
    ell(bm, b, (rb, rb, rb), seg=seg)

def chain(bm, pts, r, seg=10):
    rs = list(r) if isinstance(r, (list, tuple)) else [r] * len(pts)
    for i in range(len(pts) - 1):
        tube(bm, pts[i], pts[i + 1], rs[i], rs[i + 1], seg)

def sweep(bm, pts, radii, sides=6, flat=1.0, ref=None):
    """one closed tube along pts with a radius per point (0 closes to a tip).
    `flat` squashes the section along the second axis; `ref` orients it
    (the wide side lies across `ref`). Returns the new vertices in order."""
    pts = [V(p) for p in pts]
    n = len(pts)
    T = [(pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized() for i in range(n)]
    r0 = V(ref) if ref is not None else (V((0, 1, 0)) if abs(T[0].y) < 0.9 else V((1, 0, 0)))
    if T[0].cross(r0).length < 1e-6: r0 = V((1, 0, 0))
    N = [T[0].cross(r0).normalized()]
    for i in range(1, n):
        N.append((T[i - 1].rotation_difference(T[i]) @ N[-1]).normalized())
    rings, made = [], []
    for i in range(n):
        if radii[i] <= 1e-7:
            v = bm.verts.new(pts[i]); rings.append([v]); made.append(v); continue
        B = T[i].cross(N[i])
        ring = []
        for k in range(sides):
            a = TAU * k / sides
            v = bm.verts.new(pts[i] + N[i] * (math.cos(a) * radii[i]) + B * (math.sin(a) * radii[i] * flat))
            ring.append(v); made.append(v)
        rings.append(ring)
    for i in range(n - 1):
        A, Bn = rings[i], rings[i + 1]
        if len(A) == 1 and len(Bn) == 1: continue
        for k in range(sides):
            if len(Bn) == 1: bm.faces.new((A[k], A[(k + 1) % sides], Bn[0]))
            elif len(A) == 1: bm.faces.new((A[0], Bn[(k + 1) % sides], Bn[k]))
            else: bm.faces.new((A[k], A[(k + 1) % sides], Bn[(k + 1) % sides], Bn[k]))
    if len(rings[0]) > 1: bm.faces.new(list(reversed(rings[0])))
    if len(rings[-1]) > 1: bm.faces.new(rings[-1])
    return made

def loft(bm, rings):
    """closed surface through rings of equal size (bottom to top), capped"""
    vs = [[bm.verts.new(V(p)) for p in ring] for ring in rings]
    n = len(vs[0])
    for j in range(len(vs) - 1):
        for k in range(n):
            bm.faces.new((vs[j][k], vs[j][(k + 1) % n], vs[j + 1][(k + 1) % n], vs[j + 1][k]))
    bm.faces.new(list(reversed(vs[0])))
    bm.faces.new(vs[-1])

def slab(bm, top, x0, x1, z0, z1, nx, nz, ybot):
    """a closed height-field solid: y from ybot up to top(x, z)"""
    grid = [[bm.verts.new((lerp(x0, x1, i / nx), top(lerp(x0, x1, i / nx), lerp(z0, z1, j / nz)), lerp(z0, z1, j / nz)))
             for j in range(nz + 1)] for i in range(nx + 1)]
    for i in range(nx):
        for j in range(nz):
            bm.faces.new((grid[i][j], grid[i][j + 1], grid[i + 1][j + 1], grid[i + 1][j]))
    ring = ([grid[i][0] for i in range(nx + 1)] + [grid[nx][j] for j in range(1, nz + 1)]
            + [grid[i][nz] for i in range(nx - 1, -1, -1)] + [grid[0][j] for j in range(nz - 1, 0, -1)])
    low = [bm.verts.new((v.co.x, ybot, v.co.z)) for v in ring]
    m = len(ring)
    for k in range(m):
        bm.faces.new((ring[k], low[k], low[(k + 1) % m], ring[(k + 1) % m]))
    bm.faces.new(low)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])

def blade(bm, root, tip, w, th, across, out):
    """a flat pointed tooth: a thin rectangle at the root, a point at the tip"""
    root, tip, across, out = V(root), V(tip), V(across).normalized(), V(out).normalized()
    q = [bm.verts.new(root + across * sx * w / 2 + out * sz * th / 2) for sx, sz in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
    t = bm.verts.new(tip)
    for k in range(4): bm.faces.new((q[k], q[(k + 1) % 4], t))
    bm.faces.new(list(reversed(q)))
    return q + [t]

def xform(bm, mat):
    bmesh.ops.transform(bm, matrix=mat, verts=bm.verts[:])

def spow(x, e): return math.copysign(abs(x) ** e, x)

def sec_y(stations, y):
    """interpolated (w, zf, zb) of a vertical loft at height y"""
    return interp(stations, y)

def sec_pt_y(stations, y, a, expo=2.3, zshift=None):
    """surface point of a vertical loft: a = angle round Y from +Z (front)"""
    w, zf, zb = sec_y(stations, y)
    sa, ca = math.sin(a), math.cos(a)
    z = (zf if ca >= 0 else -zb) * spow(ca, 2 / expo)
    return V((w * spow(sa, 2 / expo), y, z + (zshift(y) if zshift else 0.0)))

def loft_y(bm, stations, n=56, expo=2.3, zshift=None):
    """vertical loft through superelliptic sections (y, half-width, front z, back z)"""
    loft(bm, [[sec_pt_y(stations, st[0], TAU * i / n, expo, zshift) for i in range(n)] for st in stations])

def sec_pt_z(stations, z, a, expo=2.4):
    """surface point of a loft along Z: sections (z, half-width, top y, bottom y);
    a = angle round Z from +Y (the back) towards +X"""
    w, top, bot = interp(stations, z)
    sa, ca = math.sin(a), math.cos(a)
    yc, hh = (top + bot) / 2, (top - bot) / 2
    return V((w * spow(sa, 2 / expo), yc + hh * spow(ca, 2 / (expo if ca >= 0 else 2.1)), z))

def loft_z(bm, stations, n=56, expo=2.4):
    loft(bm, [[sec_pt_z(stations, st[0], TAU * i / n, expo) for i in range(n)] for st in stations])


# ------------------------------------------------ analytic shape helpers ---
def sd_ell(p, c, r, rot=(0, 0, 0)):
    """approximate signed distance to an ellipsoid (negative inside)"""
    q = V(p) - V(c)
    if rot != (0, 0, 0): q = Euler(rot, 'XYZ').to_matrix().inverted() @ q
    k0 = math.sqrt((q.x / r[0]) ** 2 + (q.y / r[1]) ** 2 + (q.z / r[2]) ** 2)
    k1 = math.sqrt((q.x / r[0] ** 2) ** 2 + (q.y / r[1] ** 2) ** 2 + (q.z / r[2] ** 2) ** 2)
    return -min(r) if k1 < 1e-9 else k0 * (k0 - 1) / k1

def sd_union(p, solids): return min(sd_ell(p, *s) for s in solids)

def ray_ell(o, d, c, r):
    """far intersection of the ray o + t d with an axis-aligned ellipsoid"""
    ox, oy, oz = (o[0] - c[0]) / r[0], (o[1] - c[1]) / r[1], (o[2] - c[2]) / r[2]
    dx, dy, dz = d[0] / r[0], d[1] / r[1], d[2] / r[2]
    a = dx * dx + dy * dy + dz * dz; b = 2 * (ox * dx + oy * dy + oz * dz); cc = ox * ox + oy * oy + oz * oz - 1
    disc = b * b - 4 * a * cc
    if disc < 0: return None
    t = (-b + math.sqrt(disc)) / (2 * a)
    return t if t > 0 else None

def surf_r(o, d, solids):
    best = 0.0
    for s in solids:
        if len(s) > 2 and s[2] != (0, 0, 0): continue
        t = ray_ell(o, d, s[0], s[1])
        if t and t > best: best = t
    return best

def push_out(p, solids, pad, frame=None):
    """move p out of the (inflated) solids along the numeric gradient"""
    q = frame.inverted() @ p if frame else V(p)
    for _ in range(6):
        d = sd_union(q, solids) - pad
        if d >= 0: break
        e = 0.002
        g = V((sd_union(q + V((e, 0, 0)), solids) - sd_union(q - V((e, 0, 0)), solids),
               sd_union(q + V((0, e, 0)), solids) - sd_union(q - V((0, e, 0)), solids),
               sd_union(q + V((0, 0, e)), solids) - sd_union(q - V((0, 0, e)), solids)))
        if g.length < 1e-9: break
        q = q - g.normalized() * (d - 0.001)
    return frame @ q if frame else q


# ------------------------------------------------------------- pipeline ---
noise_tex = bpy.data.textures.new('skin', type='CLOUDS')
noise_tex.noise_scale = 0.05
noise_tex.noise_depth = 2

def evaluate(obj):
    dg = bpy.context.evaluated_depsgraph_get()
    return bpy.data.meshes.new_from_object(obj.evaluated_get(dg))

def link(name, me):
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    return obj

def bm_mesh(name, fn):
    bm = bmesh.new()
    ret = fn(bm)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me); bm.free()
    return me, ret

def with_mod(name, me, setup):
    obj = link(name, me)
    setup(obj)
    out = evaluate(obj)
    bpy.data.objects.remove(obj, do_unlink=True)
    return out

def tri_count(me): return sum(len(p.vertices) - 2 for p in me.polygons)

def run_ops(name, me, ops):
    for op in ops:
        k = op[0]
        if k == 'remesh':
            def s(o, v=op[1]):
                m = o.modifiers.new('r', 'REMESH'); m.mode = 'VOXEL'; m.voxel_size = v; m.adaptivity = 0.0
            me = with_mod(name, me, s)
        elif k in ('cut', 'isect'):
            cme, _ = bm_mesh(name + '_' + k, op[1])
            cutter = link(name + '_' + k, cme)
            cutter.hide_viewport = True; cutter.hide_render = True
            def s(o, cutter=cutter, k=k):
                b = o.modifiers.new('b', 'BOOLEAN')
                b.operation = 'DIFFERENCE' if k == 'cut' else 'INTERSECT'
                b.object = cutter; b.solver = 'EXACT'; b.use_self = True
            me = with_mod(name, me, s)
            bpy.data.objects.remove(cutter, do_unlink=True)
        elif k == 'add':
            bm = bmesh.new(); bm.from_mesh(me); op[1](bm)
            me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
        elif k == 'smooth':
            def s(o, n=op[1], f=(op[2] if len(op) > 2 else 0.6)):
                m = o.modifiers.new('s', 'SMOOTH'); m.factor = f; m.iterations = n
            me = with_mod(name, me, s)
        elif k == 'noise':
            def s(o, st=op[1]):
                m = o.modifiers.new('n', 'DISPLACE'); m.texture = noise_tex; m.strength = st; m.mid_level = 0.5
            me = with_mod(name, me, s)
    return me

def finish(me, tris, sym, recalc):
    n = tri_count(me)
    if tris and n > tris:
        def s(o):
            d = o.modifiers.new('d', 'DECIMATE'); d.ratio = min(1.0, tris / n)
            if sym: d.use_symmetry = True; d.symmetry_axis = 'X'
        me = with_mod('dec', me, s)
    bm = bmesh.new(); bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    if recalc: bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.verts.index_update(); bm.normal_update()
    verts = [v.co.copy() for v in bm.verts]
    norms = [v.normal.copy() for v in bm.verts]
    idx = [[v.index for v in f.verts] for f in bm.faces]
    bm.free()
    return verts, norms, idx

PARTS = {}
def part(name, fn, ops=None, voxel=0.008, smooth=5, noise=0.0015, tris=1200, cut=None, sym=False, raw=False, paint=None):
    """build one part. raw parts keep their construction mesh (fn may return
    a colour per created vertex); the others are fused and decimated."""
    if ONLY is not None and name not in ONLY: return
    me, ret = bm_mesh(name, fn)
    if not raw:
        if ops is None:
            ops = [('remesh', voxel)]
            if cut: ops += [('cut', cut), ('remesh', voxel)]
            if smooth: ops.append(('smooth', smooth))
            if noise: ops.append(('noise', noise))
        me = run_ops(name, me, ops)
    v, n, i = finish(me, None if raw else tris, sym, recalc=raw)
    cols = None
    if raw and ret: cols = ret
    elif paint: cols = [paint(p, q) for p, q in zip(v, n)]
    PARTS[name] = (v, i, cols)
    print(f'{name:14s} {len(v):5d} verts {len(i):5d} tris')


# ==================================================================== SMILER
# A tall, starved, black thing that stoops: digitigrade legs, a waist pinch,
# hands too big, and a crescent grin wider than its skull. Limb lengths match
# SM_LEG / SM_ARM in backrooms.js.
SM_SKIN, SM_HOLLOW = hexc('#0C0B0E'), hexc('#050406')

def sm_skin(p, n):
    return mul3(SM_SKIN, 1 + 0.06 * mnoise.noise(p * 40))

# the skin: near-black and wet, but not uniform: an oily, faintly violet cast
# in places, dry greyer patches, fine mottling (body and head share it)
SM_DRY, SM_OIL = hexc('#1c1a1e'), hexc('#120d16')
def sm_skin_at(p):
    c = mix3(SM_SKIN, SM_OIL, 0.5 * smooth(0.0, 0.45, mnoise.noise(p * 5.0 + V((4.1, 0.3, 2.2)))))
    dry = mnoise.noise(p * 9.0 + V((0.7, 5.3, 1.9))) + 0.35 * mnoise.noise(p * 31.0)
    c = mix3(c, SM_DRY, 0.75 * smooth(0.25, 0.62, dry))
    return mul3(c, 1 + 0.08 * mnoise.noise(p * 47) + 0.04 * mnoise.noise(p * 160))

def sm_thigh(bm):                                   # hip → knee, 0.56
    tube(bm, (0, 0.03, 0), (0, -0.54, 0.004), 0.046, 0.03)
    ell(bm, (0, -0.2, 0.016), (0.038, 0.17, 0.033))               # a thin quadriceps
    ell(bm, (0, -0.22, -0.02), (0.031, 0.15, 0.028))              # hamstrings
    ell(bm, (0, -0.56, 0.004), (0.035, 0.036, 0.034))             # knee knob
    ell(bm, (0, -0.545, 0.034), (0.022, 0.026, 0.012))            # kneecap
part('smThigh', sm_thigh, voxel=0.0045, tris=620, paint=sm_skin)

def sm_shin(bm):                                    # knee → hock, 0.56
    tube(bm, (0, 0.0, 0), (0, -0.55, 0), 0.026, 0.017)
    ell(bm, (0, -0.15, -0.02), (0.027, 0.11, 0.025))              # starved calf
    ell(bm, (0, -0.28, 0.018), (0.009, 0.24, 0.008))              # shin bone ridge
    tube(bm, (0, -0.3, -0.02), (0, -0.53, -0.026), 0.007, 0.007, seg=8)   # Achilles tendon
    ell(bm, (0, -0.56, 0), (0.025, 0.022, 0.026))                 # ankle knob
part('smShin', sm_shin, voxel=0.004, tris=480, paint=sm_skin)

def sm_meta(bm):                                    # hock → ball of the foot, 0.30
    tube(bm, (0, 0.0, 0), (0, -0.29, 0.0), 0.021, 0.017)
    ell(bm, (0, 0.012, -0.03), (0.021, 0.03, 0.026))              # the heel, sticking back
    for x in (-0.012, 0.0, 0.012):
        tube(bm, (x, -0.03, 0.012), (x * 1.8, -0.28, 0.012), 0.0065, 0.008, seg=8)   # long foot bones
    ell(bm, (0, -0.3, 0.004), (0.032, 0.025, 0.03))               # ball of the foot
part('smMeta', sm_meta, voxel=0.0038, tris=360, paint=sm_skin)

def sm_toes(bm):                                    # ball joint at the origin; floor at y = -0.025
    ell(bm, (0, -0.007, 0.008), (0.036, 0.018, 0.03))             # ball pad
    for k in range(4):
        x = -0.027 + k * 0.018
        L = 0.085 - abs(k - 1.5) * 0.012
        p0, p1, p2 = (x, -0.004, 0.02), (x * 1.25, -0.008, 0.02 + L * 0.55), (x * 1.35, -0.014, 0.02 + L)
        chain(bm, [p0, p1, p2], [0.009, 0.0075, 0.006], seg=8)
        ell(bm, p1, (0.0085, 0.0085, 0.0085), seg=8)               # knuckle
        tube(bm, p2, (x * 1.38, -0.024, 0.02 + L + 0.03), 0.0052, 0.0012, seg=6)   # claw
part('smToes', sm_toes, voxel=0.0026, tris=420, paint=sm_skin)

SM_TORSO = [   # y, half-width, front z, back z   (hip joints at y = 0)
    (-0.08, 0.05, 0.035, -0.045), (-0.045, 0.1, 0.058, -0.08), (0.02, 0.125, 0.066, -0.092),
    (0.08, 0.128, 0.062, -0.088), (0.13, 0.096, 0.046, -0.074), (0.2, 0.075, 0.034, -0.07),
    (0.27, 0.088, 0.046, -0.076), (0.32, 0.132, 0.082, -0.094), (0.4, 0.154, 0.102, -0.108),
    (0.5, 0.158, 0.106, -0.114), (0.58, 0.152, 0.1, -0.112), (0.635, 0.168, 0.08, -0.096),
    (0.675, 0.15, 0.058, -0.072), (0.715, 0.085, 0.04, -0.05), (0.745, 0.04, 0.03, -0.03),
]
def sm_hunch(y): return 0.05 * smooth(0.3, 0.75, y) ** 1.3      # the upper body rolls forward
def sm_tpt(y, a, out=0.0):
    p = sec_pt_y(SM_TORSO, y, a, 2.3, sm_hunch)
    q = V((p.x, 0, p.z - sm_hunch(y)))
    return p + (q.normalized() * out if q.length > 1e-6 else V())

def sm_torso(bm):
    """hip-joint height at y = 0; clavicles at 0.67. A lofted body, already
    hunched, with the anatomy laid on it: eight rib pairs standing proud, a
    sternum, sixteen spine knobs, shoulder blades, clavicles, iliac crests."""
    loft_y(bm, SM_TORSO, n=64, zshift=sm_hunch)
    for i in range(8):                                            # rib pairs, sloping down to the front
        y0 = 0.325 + i * 0.036
        for s in (-1, 1):
            pts = [sm_tpt(y0 - 0.06 * (1 - j / 12) ** 1.2 * (1 if i < 7 else 0.6), s * lerp(0.42, 2.75, j / 12), -0.001)
                   for j in range(13)]
            chain(bm, pts, 0.0082, seg=6)
    tube(bm, sm_tpt(0.6, 0.0, -0.004), sm_tpt(0.36, 0.0, -0.004), 0.011, 0.009, seg=8)   # sternum
    for k in range(16):                                           # spine knobs
        y = lerp(0.05, 0.72, k / 15)
        ell(bm, sm_tpt(y, math.pi, -0.004), (0.012, 0.012, 0.016), seg=8)
    for s in (-1, 1):
        ell(bm, sm_tpt(0.54, s * 2.45, -0.012), (0.052, 0.078, 0.016), rot=(0.25, s * 0.45, s * 0.2))   # scapulae
        tube(bm, sm_tpt(0.66, s * 0.12, -0.004), sm_tpt(0.675, s * 1.35, -0.006), 0.011, 0.01, seg=8)   # clavicles
        tube(bm, (s * 0.028, 0.74, -0.018 + sm_hunch(0.74)), (s * 0.15, 0.672, -0.02 + sm_hunch(0.67)), 0.024, 0.02, seg=10)   # hiked trapezius
        ell(bm, (s * 0.165, 0.648, -0.004 + sm_hunch(0.648)), (0.04, 0.05, 0.046))              # shoulder caps
        ell(bm, sm_tpt(0.085, s * 1.1, -0.006), (0.03, 0.022, 0.05), rot=(0, s * 0.9, s * 0.3))  # iliac crests
        ell(bm, (s * 0.108, -0.005, 0.0), (0.03, 0.032, 0.032))  # hip joints
        chain(bm, [sm_tpt(y, s * 1.3, -0.005) for y in (0.31, 0.26, 0.21, 0.16, 0.11)], 0.012, seg=8)   # obliques
    tube(bm, (0, 0.63, sm_hunch(0.63)), (0, 0.76, 0.02 + sm_hunch(0.76)), 0.042, 0.037)           # neck root
def sm_torso_cut(bm):
    ell(bm, (0, 0.225, 0.052), (0.06, 0.055, 0.022))              # the sucked-in belly under the ribs
part('smTorso', sm_torso, cut=sm_torso_cut, voxel=0.005, smooth=4, tris=2400, sym=True, paint=sm_skin)

def sm_neck(bm):                                    # neck base at the origin, up +Y (the rig leans it forward)
    tube(bm, (0, -0.05, 0), (0, 0.2, 0), 0.034, 0.03)
    for s in (-1, 1): tube(bm, (s * 0.03, -0.03, 0.022), (s * 0.013, 0.19, 0.026), 0.009, 0.007, seg=8)   # tendons
    tube(bm, (0, -0.02, 0.03), (0, 0.17, 0.033), 0.011, 0.01, seg=10)                                     # windpipe
    for k in range(4): ell(bm, (0, 0.02 + k * 0.045, 0.039), (0.012, 0.005, 0.007), seg=8)
    for k in range(5): ell(bm, (0, -0.03 + k * 0.05, -0.029), (0.013, 0.015, 0.013), seg=8)             # cervical knobs
part('smNeck', sm_neck, voxel=0.0035, tris=450, sym=True, paint=sm_skin)

# ---- the Smiler's head. Head frame: origin at the top of the neck, the eye
# line at y = SMH_EYE_Y (= the rig's eyeYOffset), +Z forward. A long, gaunt
# skull under stretched black skin: a high sloping cranium swept back into
# the occiput, sunken temples, a heavy brow over deep sockets, sharp
# cheekbones and arches over hollow cheeks, a long jaw and a narrow chin.
# The grin runs round a wide dental arch: its centre line rises from
# SMH_YMID at the front to SMH_YCOR at the last tooth (+/-SMH_AMAX round the
# face), then runs on as a closed slit, down under the cheekbone, to the jaw
# hinge where ears would be (SMH_ASLIT). The lips are drawn back off swollen
# gums; the teeth are long, crowded and irregular. Angles a are measured
# round a vertical axis through (0, y, SMH_AXZ), from +Z.
SMH_EYE_Y, SMH_EYE_X, SMH_EYE_Z, SMH_EYE_R = 0.035, 0.047, 0.062, 0.0155
SMH_HINGE = (0.0, -0.012, -0.052)
SMH_AXZ = -0.03
SMH_YMID, SMH_YCOR, SMH_YSLIT = SMH_EYE_Y - 0.092, SMH_EYE_Y - 0.022, -0.006
SMH_AMAX, SMH_ASLIT = math.radians(74), math.radians(100)
SMH_GMID, SMH_GCOR = 0.068, 0.028          # the lip gap, lip to lip: middle, last tooth
SMH_GUM_IN = 0.0075                          # the gums' centre line, inside the skin
SMH_HULL = [   # the smooth skull (unrotated: the grin is ray-cast onto these)
    ((0, 0.078, -0.048), (0.088, 0.104, 0.135)),     # cranium, long and narrow
    ((0, 0.084, 0.01), (0.074, 0.078, 0.072)),       # the high forehead
    ((0, 0.11, -0.13), (0.062, 0.066, 0.074)),       # the occiput, swept back
    ((0, -0.03, -0.008), (0.105, 0.046, 0.104)),     # the dental arch: the grin wraps round it
    ((0, -0.012, 0.03), (0.048, 0.024, 0.062)),      # the upper jaw under the nose slits
    ((0, -0.084, -0.008), (0.086, 0.04, 0.09)),      # the jaw, receding under the grin
    ((0, -0.11, 0.044), (0.028, 0.022, 0.032)),      # a narrow chin
    ((0, -0.005, -0.065), (0.062, 0.052, 0.056)),    # skull base, over the neck
]
SMH_BONES = [  # bone under thin skin
    ((0, 0.06, 0.08), (0.016, 0.014, 0.014), (0, 0, 0)),               # glabella
    ((0, 0.05, 0.085), (0.008, 0.02, 0.008), (0.3, 0, 0)),             # a thin nasal bridge
]
for _s in (-1, 1):
    SMH_BONES += [
        ((_s * 0.04, 0.066, 0.075), (0.034, 0.013, 0.02), (0.3, _s * 0.35, -_s * 0.12)),   # brow arches
        ((_s * 0.068, 0.014, 0.056), (0.026, 0.012, 0.02), (0, _s * 0.75, _s * 0.2)),     # cheekbones
        ((_s * 0.083, 0.01, 0.006), (0.009, 0.01, 0.036), (0, _s * 0.22, 0)),     # the arches back to the hinge
        ((_s * 0.082, -0.004, -0.038), (0.016, 0.02, 0.016), (0, 0, 0)),          # the hinge knuckle
        ((_s * 0.082, -0.07, -0.045), (0.017, 0.03, 0.024), (0, 0, 0)),           # jaw angles
        ((_s * 0.087, -0.035, -0.05), (0.015, 0.04, 0.02), (0, 0, 0)),            # rami
        ((_s * 0.07, -0.005, -0.095), (0.018, 0.026, 0.02), (0, 0, 0)),           # mastoids
    ]
SMH_BONES += [((0, 0.168, -0.06), (0.008, 0.012, 0.11), (0.12, 0, 0))]            # a low ridge along the crown
SMH_SOLIDS = SMH_HULL + SMH_BONES

def smh_u(a): return min(1.0, abs(a) / SMH_AMAX)
def smh_gy(a):
    """the grin's centre line: up to the last tooth, then down along the slit"""
    y = SMH_YMID + (SMH_YCOR - SMH_YMID) * smh_u(a) ** 1.8
    return lerp(y, SMH_YSLIT, smooth(SMH_AMAX, SMH_ASLIT, abs(a)))
def smh_hb(a):
    """half the lip gap: wide open over the teeth, a hairline past them, closed at the hinge"""
    u = abs(a) / SMH_AMAX
    g = SMH_GMID + (SMH_GCOR - SMH_GMID) * min(1.0, u) ** 1.4
    g = lerp(g, 0.004, smooth(0.86, 1.1, u))
    return g * (1 - smooth(SMH_ASLIT - 0.12, SMH_ASLIT, abs(a))) / 2
def smh_hv(a): return min(0.0066, 0.4 * smh_hb(a))     # the gums' half height
def smh_rs(a, y):
    """distance from the axis to the skin, direction a, height y"""
    return surf_r((0, y, SMH_AXZ), (math.sin(a), 0, math.cos(a)), SMH_HULL)
def smh_pt(a, y, inset=0.0):
    r = smh_rs(a, y) - inset
    return V((r * math.sin(a), y, SMH_AXZ + r * math.cos(a)))

def smh_split(upper):
    """the height field the head and jaw are split along: the lip gap round
    the face, closed along the slit, then a wall down behind the hinge"""
    def top(x, z):
        a = math.atan2(x, z - SMH_AXZ)
        y = smh_gy(a) + (smh_hb(a) if upper else -smh_hb(a))
        return lerp(y, -0.33, smooth(SMH_ASLIT + 0.07, SMH_ASLIT + 0.3, abs(a)))
    return top
def sm_split_cut(upper):
    def fn(bm): slab(bm, smh_split(upper), -0.24, 0.24, -0.14, 0.26, 96, 84, -0.36)
    return fn

def smh_arc():
    """the grin's centre line as (a, arc length) samples, over the teeth"""
    out, prev, s = [], None, 0.0
    for i in range(401):
        a = -SMH_AMAX + 2 * SMH_AMAX * i / 400
        p = smh_pt(a, smh_gy(a))
        if prev is not None: s += (p - prev).length
        out.append((a, s)); prev = p
    return out
def smh_a_at(arc, s):
    for i in range(1, len(arc)):
        if arc[i][1] >= s:
            a0, s0 = arc[i - 1]; a1, s1 = arc[i]
            return lerp(a0, a1, (s - s0) / max(1e-9, s1 - s0))
    return arc[-1][0]

def smh_teeth_layout(upper):
    """crowded, irregular teeth: lower ones between the upper ones, a few
    pushed out of line into a second row at the front, some broken"""
    rng = random.Random(41 if upper else 43)
    arc = smh_arc(); total = arc[-1][1]
    n = 22 if upper else 21
    pitch = total / 22
    out = []
    def tooth(s, row2=False):
        a = smh_a_at(arc, clamp(s, 0.004, total - 0.004))
        u = smh_u(a)
        broken = rng.random() < 0.1
        out.append(dict(
            a=a, row2=row2,
            len=rng.uniform(0.8, 1.04) * (0.5 if broken else 1.0) * (0.72 if row2 else 1.0),
            w=lerp(0.0122, 0.0078, u) * rng.uniform(0.82, 1.2) * (0.8 if row2 else 1.0),
            lean=clamp(rng.gauss(0, 0.4), -0.8, 0.8) * 0.05,
            hook=0.0 if broken else rng.uniform(0.0012, 0.0048),
            twist=clamp(rng.gauss(0, 0.32), -0.7, 0.7),
            inset=SMH_GUM_IN + rng.uniform(-0.0016, 0.0009) + (0.0055 if row2 else 0.0),
            shade=rng.uniform(0.84, 1.0) * (0.86 if row2 else 1.0)))
    for k in range(n):
        tooth((k + (0.5 if upper else 1.0)) * pitch + rng.uniform(-0.24, 0.24) * pitch)
    for k in range(4 if upper else 3):   # the second row, crowded in behind the front teeth
        tooth(total / 2 + (k - (1.5 if upper else 1.0)) * pitch * 1.9 + rng.uniform(-0.2, 0.2) * pitch, True)
    return out
SMH_TEETH = {True: smh_teeth_layout(True), False: smh_teeth_layout(False)}

def smh_gum_pts(upper):
    """the gums' centre line (a, point, half height), just inside the lip"""
    out, A = [], SMH_AMAX + math.radians(6)
    for i in range(73):
        a = -A + 2 * A * i / 72
        hb, hv = smh_hb(a), smh_hv(a)
        y = smh_gy(a) + (hb - 0.5 * hv) * (1 if upper else -1)
        out.append((a, smh_pt(a, y, SMH_GUM_IN), hv))
    return out
SMH_GUMS = {True: smh_gum_pts(True), False: smh_gum_pts(False)}
def sm_gums(upper):
    """swollen gums under the drawn-back lip, bulging over each root"""
    def fn(bm):
        roots = [t['a'] for t in SMH_TEETH[upper] if not t['row2']]
        pts, radii = [], []
        for a, p, hv in SMH_GUMS[upper]:
            bulge = min(1.0, sum(math.exp(-((a - ta) / 0.03) ** 2) for ta in roots))
            pts.append(p); radii.append(hv / 1.6 * (0.5 + 0.65 * bulge))
        radii[0] = radii[-1] = 0.0
        sweep(bm, pts, radii, sides=8, flat=1.6)
    return fn
def smh_gum_d(p, upper):
    """how far p is from the gum tube, in tube radii (< 1 inside)"""
    best = 9.0
    for a, c, hv in SMH_GUMS[upper][::2]:
        dy = (p.y - c.y) / max(1e-4, hv)
        dh = math.hypot(p.x - c.x, p.z - c.z) / max(1e-4, hv / 1.6)
        best = min(best, math.hypot(dy, dh))
    return best

def sm_head_base(bm):
    for s in SMH_SOLIDS: ell(bm, s[0], s[1], s[2] if len(s) > 2 else (0, 0, 0), seg=24)
def sm_head_features(bm):
    """cut into the skull: deep sockets, sunken temples, hollow cheeks, two slits"""
    for s in (-1, 1):
        ell(bm, (s * SMH_EYE_X, SMH_EYE_Y - 0.002, 0.09), (0.026, 0.021, 0.037))                  # eye sockets
        ell(bm, (s * 0.094, 0.072, 0.0), (0.018, 0.04, 0.05))                                      # temples
        ell(bm, (s * 0.106, 0.004, -0.018), (0.02, 0.028, 0.036))                                  # hollow cheeks
        ell(bm, (s * 0.011, 0.003, 0.097), (0.0042, 0.016, 0.014), rot=(0, 0, s * 0.35))          # nose slits
        # the lip is drawn up so hard the skin above it creases: a fold from
        # beside the slits down to each corner, and snarl lines over the gums
        pts = [smh_pt(s * a, lerp(-0.004, smh_gy(s * a) + smh_hb(s * a) + 0.006, (a / 1.2) ** 1.3), 0.0)
               for a in [0.2 + 1.0 * i / 9 for i in range(10)]]
        sweep(bm, pts, [0.0045] * 9 + [0.0], sides=8)
    ell(bm, (0, -0.008, 0.108), (0.06, 0.015, 0.02))       # the upper lip pulled flat and tight
    ell(bm, (0, -0.128, -0.018), (0.05, 0.018, 0.042))     # the floor under the jaw, hollow between the bones
    # the lips themselves are drawn back to the gums: a bevel along each lip
    # edge, so the skin rolls in to the gum instead of overhanging it
    A = SMH_AMAX + 0.05
    for sg in (1, -1):
        pts = [smh_pt(a, smh_gy(a) + sg * smh_hb(a), 0.0) for a in [-A + 2 * A * i / 30 for i in range(31)]]
        sweep(bm, pts, [0.0] + [0.0068 * (1 - 0.5 * smooth(0.7, 1.0, abs(i / 15 - 1))) for i in range(1, 30)] + [0.0], sides=10)
    for sg, dy, A, r in ((1, 0.016, 0.9, 0.0042), (-1, 0.016, 0.7, 0.004)):   # a snarl line over the gums, a crease under the lip
        pts = [smh_pt(a, smh_gy(a) + sg * (smh_hb(a) + dy), 0.0) for a in [-A + 2 * A * i / 16 for i in range(17)]]
        sweep(bm, pts, [0.0] + [r] * 15 + [0.0], sides=8)
def sm_head_lids(bm):
    """heavy lids round each eye, puffy underneath"""
    for s in (-1, 1):
        ell(bm, (s * SMH_EYE_X, SMH_EYE_Y, SMH_EYE_Z), (0.0205, 0.0196, 0.0196), seg=20)
        ell(bm, (s * (SMH_EYE_X + 0.002), SMH_EYE_Y - 0.0145, SMH_EYE_Z + 0.006), (0.017, 0.0065, 0.011),
            rot=(0.35, 0, s * 0.12), seg=14)
def sm_head_eyes(bm):
    """the eyeball's seat and the lid opening: a narrow almond, outer corner up"""
    for s in (-1, 1):
        E = V((s * SMH_EYE_X, SMH_EYE_Y, SMH_EYE_Z))
        ell(bm, E, (SMH_EYE_R + 0.0009,) * 3, seg=20)
        ell(bm, E + V((s * 0.0008, -0.0024, 0.018)), (0.0124, 0.0072, 0.018), rot=(0, 0, s * 0.14), seg=24)

SM_GUM, SM_LIDC = hexc('#1a1013'), hexc('#100b10')
def smh_paint(p, n):
    d = sd_union(p, SMH_SOLIDS)
    c = sm_skin_at(p)
    if abs(math.atan2(p.x, p.z - SMH_AXZ)) < SMH_AMAX + 0.15:
        gd = min(smh_gum_d(p, True), smh_gum_d(p, False))
        if gd < 1.35:                                    # wet, swollen gums
            return mul3(mix3(SM_GUM, c, smooth(1.0, 1.35, gd)), 1 + 0.1 * mnoise.noise(p * 120))
    for s in (-1, 1):                                    # darker, wetter round the eyes
        e = (p - V((s * SMH_EYE_X, SMH_EYE_Y, SMH_EYE_Z))).length
        c = mix3(c, SM_LIDC, 0.7 * (1 - smooth(0.02, 0.034, e)))
    if d < -0.0035: return mix3(c, SM_HOLLOW, smooth(-0.0035, -0.009, d))   # cut faces, the inside
    return mul3(c, 1 - 0.45 * smooth(-0.0005, -0.0035, d))                    # hollows a little darker

part('smHead', sm_head_base, ops=[('remesh', 0.0036), ('cut', sm_head_features), ('add', sm_head_lids),
                                   ('remesh', 0.003), ('smooth', 2, 0.5), ('cut', sm_split_cut(True)),
                                   ('add', sm_gums(True)), ('remesh', 0.0028), ('smooth', 1, 0.4),
                                   ('cut', sm_head_eyes), ('noise', 0.0005)],
     tris=2700, paint=smh_paint)
part('smJaw', sm_head_base, ops=[('remesh', 0.0036), ('cut', sm_head_features), ('remesh', 0.003),
                                  ('smooth', 2, 0.5), ('isect', sm_split_cut(False)),
                                  ('add', sm_gums(False)), ('remesh', 0.0028), ('smooth', 1, 0.4),
                                  ('noise', 0.0005)],
     tris=950, paint=smh_paint)

SM_TOOTH_RAMP = [(0.0, *hexc('#5f5446')), (0.14, *hexc('#a99f88')), (0.45, *hexc('#e0dacb')), (1.0, *hexc('#f8f5ef'))]
def sm_teeth(upper):
    """long blades, a little curved and hooked inward, crowded and twisted out
    of line. The material is unlit, so the colours carry the shape: stained
    at the root, white at the tip, the edges darker than the faces."""
    def fn(bm):
        cols, sg = [], (1 if upper else -1)
        for T in SMH_TEETH[upper]:
            a = T['a']
            hb, hv, gy = smh_hb(a), smh_hv(a), smh_gy(a)
            span = 2 * hb - 3 * hv
            if span < 0.005: continue
            edge = gy + sg * (hb - 1.5 * hv)                   # where the gum ends
            L = span * T['len']
            yr, yt = edge + sg * 0.006, edge - sg * L
            pts, rad, fs = [], [], (0.0, 0.3, 0.7, 1.0)
            for f, rf in zip(fs, (1.0, 1.06, 0.62, 0.0)):
                pts.append(smh_pt(a + T['lean'] * f ** 1.5, lerp(yr, yt, f), T['inset'] + T['hook'] * f * f))
                rad.append(T['w'] / 2 * rf)
            out, across = V((math.sin(a), 0, math.cos(a))), V((math.cos(a), 0, -math.sin(a)))
            ref = out * math.cos(T['twist']) + across * math.sin(T['twist'])
            sweep(bm, pts, rad, sides=6, flat=0.46, ref=ref)
            for ring, f in enumerate(fs):
                base = interp(SM_TOOTH_RAMP, (f * L + 0.006) / (L + 0.006))
                for k in range(6 if ring < 3 else 1):
                    sh = (0.7 + 0.3 * abs(math.sin(TAU * k / 6))) if ring < 3 else 1.0
                    cols.append(mul3(base, sh * T['shade']))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        return cols
    return fn
part('smTeethUp', sm_teeth(True), raw=True)
part('smTeethLo', sm_teeth(False), raw=True)

def sm_maw(bm):
    """the black inside of the mouth, behind the gums and teeth"""
    A = SMH_AMAX + math.radians(8)
    for i in range(31):
        a = -A + 2 * A * i / 30
        y, hb = smh_gy(a), smh_hb(a)
        ell(bm, smh_pt(a, y, 0.029), (0.02, max(0.008, hb * 0.95), 0.01), rot=(0, a, 0), seg=10)
    ell(bm, (0, -0.048, -0.008), (0.08, 0.04, 0.058))
part('smMaw', sm_maw, voxel=0.005, smooth=2, noise=0, tris=260)

def sm_uparm(bm):                                   # shoulder → elbow, 0.48
    tube(bm, (0, 0.02, 0), (0, -0.46, 0), 0.03, 0.024)
    ell(bm, (0, -0.05, 0), (0.036, 0.07, 0.036))                  # deltoid
    ell(bm, (0, -0.2, 0.012), (0.022, 0.12, 0.02))                # biceps, wasted
    ell(bm, (0, -0.19, -0.012), (0.024, 0.13, 0.022))             # triceps
    ell(bm, (0, -0.48, -0.01), (0.035, 0.03, 0.03))               # elbow knob, 0.07
    ell(bm, (0, -0.49, -0.03), (0.018, 0.022, 0.015))             # olecranon
part('smUpArm', sm_uparm, voxel=0.004, tris=480, paint=sm_skin)

def sm_fore(bm):                                    # elbow → wrist, 0.46
    tube(bm, (0, 0.0, 0), (0, -0.45, 0), 0.022, 0.017)
    ell(bm, (0, -0.09, 0.004), (0.025, 0.09, 0.022))              # forearm muscle
    tube(bm, (0.004, -0.02, -0.018), (0.004, -0.43, -0.013), 0.006, 0.006, seg=8)   # ulna ridge
    ell(bm, (0, -0.455, 0), (0.022, 0.014, 0.018))                # wrist knob
    ell(bm, (-0.017, -0.445, -0.006), (0.008, 0.009, 0.008), seg=8)
part('smFore', sm_fore, voxel=0.0035, tris=420, paint=sm_skin)

def sm_hand(bm):
    """wrist at the origin, the hand hanging down -Y, palm towards +Z, thumb
    on +X. 0.30 long: palm 0.11, four fingers of three long bones, claws."""
    ell(bm, (0, -0.052, 0.0), (0.042, 0.06, 0.013))               # palm
    ell(bm, (0, -0.008, 0), (0.028, 0.018, 0.016))                # wrist bones
    curl = [0.32, 0.5, 0.45]
    for k, x in enumerate((-0.03, -0.01, 0.01, 0.03)):
        tube(bm, (x * 0.5, -0.015, -0.009), (x, -0.108, -0.006), 0.0055, 0.0065, seg=8)   # tendons on the back
        L = [0.075, 0.065, 0.05]
        sc = (0.9, 1.0, 0.97, 0.84)[k]
        p, ang, pts = V((x, -0.112, 0.0)), 0.0, []
        pts.append(p.copy())
        for j in range(3):
            ang += curl[j]
            p = p + V((x * 0.35, -math.cos(ang), math.sin(ang))).normalized() * L[j] * sc
            pts.append(p.copy())
        chain(bm, pts, [0.0098, 0.0086, 0.0074, 0.0062], seg=8)
        for q in pts[:3]: ell(bm, q, (0.0115, 0.0115, 0.0115), seg=8)          # knuckles
        dcl = V((0, -math.cos(ang + 0.55), math.sin(ang + 0.55)))
        tube(bm, pts[-1], pts[-1] + dcl * 0.035, 0.0052, 0.0008, seg=6)        # claw
    ell(bm, (0.03, -0.045, 0.01), (0.02, 0.035, 0.014), rot=(0, 0, -0.3))  # thenar mass
    t0, t1 = V((0.04, -0.055, 0.016)), V((0.05, -0.1, 0.034))
    t2 = t1 + V((0.002, -0.042, 0.02))
    chain(bm, [t0, t1, t2], [0.012, 0.0092, 0.0072], seg=8)
    ell(bm, t1, (0.0105, 0.0105, 0.0105), seg=8)
    tube(bm, t2, t2 + V((0.0, -0.028, 0.02)), 0.0055, 0.0008, seg=6)
part('smHand', sm_hand, voxel=0.0024, tris=900, paint=sm_skin)


# ===================================================================== HOUND
# A starved human on all fours: rump high, back sloping to the shoulders,
# the head hanging under a curtain of wet black hair. Spine frame: origin at
# the rig's spine joint (0.70 m), +Z forward.
HD_SKIN = hexc('#8F8A84'); HD_BRUISE = (hexc('#6B6670'), hexc('#77706E')); HD_MANGE = hexc('#B3A896')
HD_SCAB = hexc('#4A3833'); HD_BONE = hexc('#D8CEBC'); HD_DIRT = hexc('#5E5550'); HD_CLAW = hexc('#2B2522')
HD_TOOTH, HD_TOOTH_ROOT, HD_GUM = hexc('#D6C9A3'), hexc('#8C7A58'), hexc('#5A2A28')

def hd_skin_at(p, dirt=0.0):
    m = mnoise.noise(p * 8.0 + V((1.3, 0.2, 0.7)))
    b = mnoise.noise(p * 19.0 + V((3.1, 1.7, 2.3)))
    c = HD_SKIN
    if m > 0.3: c = mix3(c, HD_MANGE, smooth(0.3, 0.6, m) * 0.6)
    if b < -0.15: c = mix3(c, HD_BRUISE[0] if m < 0 else HD_BRUISE[1], smooth(-0.15, -0.5, b) * 0.75)
    c = mul3(c, 1 + 0.05 * mnoise.noise(p * 70))
    return mix3(c, HD_DIRT, dirt) if dirt else c
def hd_skin(p, n): return hd_skin_at(p)

HD_TRUNK = [   # z, half-width, top y, bottom y   (spine joint at the origin, rump high)
    (-0.44, 0.05, 0.14, 0.08), (-0.41, 0.11, 0.17, 0.035), (-0.35, 0.145, 0.19, 0.005),
    (-0.26, 0.15, 0.188, 0.012), (-0.17, 0.122, 0.165, 0.035), (-0.09, 0.1, 0.132, 0.01),
    (-0.01, 0.118, 0.106, -0.06), (0.08, 0.142, 0.088, -0.13), (0.18, 0.15, 0.074, -0.15),
    (0.27, 0.166, 0.062, -0.13), (0.33, 0.135, 0.048, -0.1), (0.375, 0.075, 0.025, -0.07),
    (0.395, 0.035, 0.012, -0.05),
]
def hd_tpt(z, a, out=0.0):
    p = sec_pt_z(HD_TRUNK, z, a)
    w, top, bot = interp(HD_TRUNK, z)
    q = V((p.x, p.y - (top + bot) / 2, 0))
    return p + (q.normalized() * out if q.length > 1e-6 else V())
def hd_top(z): return hd_tpt(z, 0.0).y
HD_BODY_SDF = [((0, (t + b) / 2, z), (w, (t - b) / 2, 0.05), (0, 0, 0)) for z, w, t, b in HD_TRUNK[1:-1]]

def hd_body(bm):
    """a human torso on its side: wide flat back, scapulae standing up, ribs
    down the flanks, a knobbed spine, sharp hip bones and a sucked-in belly"""
    loft_z(bm, HD_TRUNK, n=64)
    for s in (-1, 1):
        ell(bm, hd_tpt(0.2, s * 0.55, -0.006) + V((0, 0.012, 0)), (0.05, 0.014, 0.085), rot=(0.1, 0, s * 0.35))   # shoulder blades
        ell(bm, (s * 0.15, -0.07, 0.3), (0.042, 0.048, 0.048))                     # shoulder joints
        ell(bm, hd_tpt(-0.19, s * 0.95, -0.004), (0.03, 0.02, 0.055), rot=(0, 0, s * 0.6))   # sharp hip bones
        ell(bm, (s * 0.11, 0.11, -0.26), (0.048, 0.06, 0.068))                     # round the hip joints
        tube(bm, hd_tpt(0.3, s * 0.2, -0.004), hd_tpt(-0.33, s * 0.18, -0.004), 0.018, 0.02, seg=8)   # muscles beside the spine
        for i in range(11):                                                         # rib ridges down each flank
            z0 = -0.02 + i * 0.023
            pts = [hd_tpt(z0 + 0.05 * (j / 10) ** 1.3, s * lerp(0.5, 2.35, j / 10), -0.001) for j in range(11)]
            chain(bm, pts, 0.0078, seg=6)
    for k in range(19):                                                             # vertebrae
        z = lerp(0.34, -0.38, k / 18)
        ell(bm, hd_tpt(z, 0.0, -0.005), (0.013, 0.012, 0.014), seg=8)
    tube(bm, (0, -0.02, 0.3), (0, -0.07, 0.4), 0.05, 0.042)                         # neck root
def hd_body_cut(bm):
    ell(bm, (0, -0.07, -0.05), (0.07, 0.03, 0.09))                 # sucked-in belly
def hd_body_paint(p, n):
    return hd_skin_at(p)
part('hdBody', hd_body, cut=hd_body_cut, voxel=0.005, smooth=4, tris=2200, sym=True, paint=hd_body_paint)

def hd_neck(bm):                                    # neck frame: +Z along the neck
    tube(bm, (0, 0, -0.05), (0, 0, 0.19), 0.036, 0.032)
    for s in (-1, 1): tube(bm, (s * 0.028, -0.02, -0.02), (s * 0.012, -0.025, 0.18), 0.009, 0.008, seg=8)
    tube(bm, (0, -0.03, 0.0), (0, -0.03, 0.17), 0.011, 0.011, seg=8)
part('hdNeck', hd_neck, voxel=0.004, tris=300, paint=hd_skin)

# ---- the head: a human skull with the lower face pulled forward, hanging.
# Built in a face frame, then pitched down HD_PITCH about HD_S and lifted so
# the eyes land at y = 0.055 in the rig's head frame (origin at the atlas).
HD_S, HD_PITCH = V((0, 0.035, 0.075)), 0.35
HD_EYE_F = V((0.03, 0.055, 0.147))
_Rx = Matrix.Rotation(HD_PITCH, 4, 'X')
_e = HD_S + (_Rx.to_3x3() @ (HD_EYE_F - HD_S))
HD_HEAD_M = Matrix.Translation(V((0, 0.055 - _e.y, 0))) @ Matrix.Translation(HD_S) @ _Rx @ Matrix.Translation(-HD_S)
HD_HINGE_F = V((0, 0.0, 0.06))
HDH_SOLIDS = [
    ((0, 0.06, 0.07), (0.075, 0.095, 0.095), (0, 0, 0)),         # cranium 0.15 x 0.19 x 0.19
    ((0, 0.066, 0.156), (0.064, 0.016, 0.02), (0.2, 0, 0)),      # human brow
    ((0, 0.02, 0.148), (0.058, 0.05, 0.05), (0, 0, 0)),          # midface
    ((0, -0.012, 0.2), (0.046, 0.034, 0.072), (0, 0, 0)),        # upper jaw pushed forward
    ((0, -0.03, 0.2), (0.042, 0.013, 0.068), (0, 0, 0)),         # stretched upper lip
    ((0.05, 0.03, 0.14), (0.024, 0.016, 0.03), (0, 0.4, 0)),     # cheekbones
    ((-0.05, 0.03, 0.14), (0.024, 0.016, 0.03), (0, -0.4, 0)),
    ((0.064, 0.02, 0.06), (0.022, 0.04, 0.034), (0, 0, 0)),      # jaw muscles
    ((-0.064, 0.02, 0.06), (0.022, 0.04, 0.034), (0, 0, 0)),
]
HD_WOUNDS = [(V((0.045, 0.022, 0.172)), 0.012), (V((-0.032, 0.08, 0.158)), 0.011), (V((-0.05, 0.0, 0.16)), 0.009)]

def hd_head(bm):
    for c, r, rot in HDH_SOLIDS: ell(bm, c, r, rot, seg=22)
    tube(bm, (0, 0.056, 0.165), (0, 0.02, 0.232), 0.008, 0.012, seg=10)   # nose bridge (human, not a snout)
    ell(bm, (0, 0.012, 0.238), (0.015, 0.012, 0.012))
    for s in (-1, 1): ell(bm, (s * 0.012, 0.008, 0.23), (0.008, 0.008, 0.008), seg=10)
    xform(bm, HD_HEAD_M)
def hd_head_cut(bm):
    for s in (-1, 1):
        ell(bm, (s * 0.03, 0.055, 0.166), (0.017, 0.014, 0.022))   # small sunken eyes
        ell(bm, (s * 0.008, 0.002, 0.244), (0.004, 0.003, 0.006), seg=10)   # nostrils
    for c, r in HD_WOUNDS: ell(bm, c + V((0, 0, 0.006)), (r, r * 0.8, 0.007), seg=12)   # scratched-open patches
    xform(bm, HD_HEAD_M)
def hd_head_paint(p, n):
    q = HD_HEAD_M.inverted() @ p
    c = hd_skin_at(q)
    for w, r in HD_WOUNDS:
        d = (q - w).length
        if d < r * 0.75: c = HD_BONE
        elif d < r * 1.7: c = mix3(HD_SCAB, c, smooth(r * 0.75, r * 1.7, d))
    if sd_union(q, HDH_SOLIDS) < -0.004: c = mul3(c, 0.55)
    if q.y < -0.036 and n.y < -0.5 and q.z > 0.08 and abs(q.x) < 0.04: c = mix3(c, HD_GUM, 0.6)
    return c
part('hdHead', hd_head, cut=hd_head_cut, voxel=0.0035, smooth=3, noise=0.0008, tris=1800, sym=True, paint=hd_head_paint)

def hd_jaw(bm):
    for s in (-1, 1): tube(bm, (s * 0.058, 0.0, 0.06), (s * 0.028, -0.056, 0.222), 0.015, 0.012, seg=10)
    tube(bm, (-0.028, -0.056, 0.222), (0.028, -0.056, 0.222), 0.013, 0.013, seg=10)
    ell(bm, (0, -0.05, 0.14), (0.04, 0.018, 0.085))
    ell(bm, (0, -0.062, 0.212), (0.028, 0.016, 0.025))
    xform(bm, HD_HEAD_M)
def hd_jaw_paint(p, n):
    q = HD_HEAD_M.inverted() @ p
    c = hd_skin_at(q)
    return mix3(c, HD_GUM, 0.6) if (q.y > -0.052 and (HD_HEAD_M.inverted().to_3x3() @ n).y > 0.5 and abs(q.x) < 0.035) else c
part('hdJaw', hd_jaw, voxel=0.0032, smooth=3, tris=560, sym=True, paint=hd_jaw_paint)

def hd_arcade(t, inset=0.0):
    """the tooth row, t in [-1, 1] from one side round the front to the other"""
    return V((0.043 * t * (1 - inset * 6), 0.0, 0.258 - 0.17 * t * t - inset))
def hd_teeth(upper):
    def fn(bm):
        cols, rng = [], random.Random(5 if upper else 6)
        rows = [(8, 0.42, 0.0, 0.014), (6, 0.28, 0.009, 0.012)]    # two crowded rows in front
        spots = []
        for n, span, inset, L in rows:
            for i in range(n): spots.append((-span + 2 * span * i / (n - 1), inset, L))
        for s in (-1, 1): spots.append((s * 0.56, 0.0, 0.025))       # canines
        for t, inset, L in spots:
            p = hd_arcade(t, inset)
            p.y = -0.041 if upper else -0.05
            dz = -0.34 * t
            across = V((1, 0, dz)).normalized(); out = V((across.z, 0, -across.x)) * (1 if True else -1)
            if rng.random() < 0.15: L *= 0.45                        # a few broken
            L *= rng.uniform(0.85, 1.15)
            tip = p + V((0, -L if upper else L, 0)) + V((0, 0, 0.002))
            vs = blade(bm, p, tip, 0.006 if abs(t) < 0.5 else 0.0075, 0.004, across, (0, 0, 1))
            cols += [HD_TOOTH_ROOT] * 4 + [HD_TOOTH]
        xform(bm, HD_HEAD_M)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        return cols
    return fn
part('hdTeethUp', hd_teeth(True), raw=True)
part('hdTeethLo', hd_teeth(False), raw=True)

HD_EAR_F = V((0.077, 0.05, 0.062))
def hd_ear(bm):
    """a human ear, slightly pointed; ear frame: root at the origin, facing +X"""
    ell(bm, (0.006, 0.02, -0.004), (0.008, 0.038, 0.022))
    pts = [V((0.01, 0.02 + 0.036 * math.cos(t), -0.004 + 0.024 * math.sin(t))) for t in [(-2.6 + 5.2 * i / 12) for i in range(13)]]
    pts[6] = pts[6] + V((0, 0.018, -0.004))                         # the point
    chain(bm, pts, 0.0045, seg=6)
    ell(bm, (0.004, -0.01, -0.002), (0.007, 0.014, 0.012))          # lobe
part('hdEar', hd_ear, voxel=0.0018, smooth=2, noise=0, tris=180, paint=hd_skin)

def hd_uparm(bm):                                   # 0.31, shoulder → elbow
    tube(bm, (0, 0.02, 0), (0, -0.3, 0), 0.031, 0.024)
    ell(bm, (0, -0.03, 0), (0.037, 0.06, 0.036))
    ell(bm, (0, -0.14, 0.01), (0.022, 0.09, 0.02))
    ell(bm, (0, -0.31, -0.01), (0.024, 0.022, 0.022))              # elbow knob 0.045
    ell(bm, (0, -0.315, -0.027), (0.013, 0.016, 0.012))            # elbow point, turned back
part('hdUpArm', hd_uparm, voxel=0.004, tris=330, paint=hd_skin)

def hd_fore(bm):                                    # 0.27, elbow → wrist
    tube(bm, (0, 0.0, 0), (0, -0.265, 0), 0.024, 0.017)
    ell(bm, (0, -0.07, 0.0), (0.027, 0.07, 0.024))
    tube(bm, (0.004, -0.01, -0.017), (0.004, -0.25, -0.012), 0.006, 0.006, seg=8)
    ell(bm, (0, -0.27, 0), (0.022, 0.014, 0.017))
part('hdFore', hd_fore, voxel=0.0035, tris=300, paint=hd_skin)

def hd_hand(bm):
    """wrist joint at the origin, 0.035 above the floor; the hand lies flat
    along +Z, palm down, thumb on +X (the right hand; the rig mirrors it).
    Five fingers, webbed to the second knuckle, dull black claws."""
    ell(bm, (0, -0.021, 0.047), (0.041, 0.012, 0.048))              # palm
    ell(bm, (0, -0.019, 0.008), (0.032, 0.014, 0.02))               # heel of the hand
    knuck = []
    for k, x in enumerate((-0.031, -0.0105, 0.0105, 0.03)):
        L = (0.056, 0.068, 0.074, 0.066)[k]
        spl = x * 0.35
        p0 = V((x, -0.022, 0.09))
        p1 = p0 + V((spl * 0.5, -0.002, L * 0.44))
        p2 = p1 + V((spl * 0.3, -0.006, L * 0.32))
        p3 = p2 + V((spl * 0.2, -0.008, L * 0.22))
        chain(bm, [p0, p1, p2, p3], [0.0098, 0.0088, 0.0078, 0.0068], seg=8)
        for q in (p0, p1, p2): ell(bm, q, (0.0105, 0.0095, 0.0105), seg=8)
        tube(bm, p3, p3 + V((0, -0.006, 0.02)), 0.0058, 0.002, seg=6)   # dull claw
        knuck.append((p0, p1))
    for k in range(3):                                                    # webbing between the fingers
        a0, a1 = knuck[k]; b0, b1 = knuck[k + 1]
        c = (a0 + a1 + b0 + b1) / 4
        ell(bm, c + V((0, 0.0, -0.006)), ((b0.x - a0.x) / 2 + 0.003, 0.0036, 0.028), seg=12)
    t0 = V((0.036, -0.02, 0.028)); t1 = t0 + V((0.022, -0.004, 0.03)); t2 = t1 + V((0.01, -0.006, 0.026))
    ell(bm, (0.03, -0.02, 0.035), (0.016, 0.012, 0.03))                    # thenar pad
    chain(bm, [t0, t1, t2], [0.0115, 0.009, 0.0075], seg=8)
    tube(bm, t2, t2 + V((0.0, -0.006, 0.018)), 0.0058, 0.002, seg=6)
    ell(bm, (0.042, -0.02, 0.066), (0.01, 0.0035, 0.02), rot=(0, 0.5, 0), seg=12)   # web to the thumb
def hd_hand_paint(p, n):
    c = hd_skin_at(p)
    tip = 0.09 + (0.056, 0.068, 0.074, 0.066)[min(3, max(0, int((p.x + 0.041) / 0.0205)))] * 0.99
    if p.x < 0.042 and p.z > tip - 0.004: c = mix3(c, HD_CLAW, smooth(tip - 0.004, tip + 0.006, p.z))
    if p.x >= 0.042 and p.z > 0.1: c = mix3(c, HD_CLAW, smooth(0.1, 0.11, p.z))
    if n.y < -0.3: c = mix3(c, HD_DIRT, 0.8)                        # dirty palm
    return c
part('hdHand', hd_hand, voxel=0.0024, smooth=3, noise=0.0006, tris=520, paint=hd_hand_paint)

def hd_thigh(bm):                                   # 0.44, hip → knee
    tube(bm, (0, 0.02, 0), (0, -0.43, 0.005), 0.05, 0.033)
    ell(bm, (0, -0.17, 0.02), (0.045, 0.14, 0.038))
    ell(bm, (0, -0.2, -0.026), (0.036, 0.13, 0.03))
    ell(bm, (0, -0.44, 0.008), (0.036, 0.034, 0.035))              # knee knob 0.07
    ell(bm, (0, -0.428, 0.038), (0.022, 0.026, 0.012))             # kneecap (dirty)
def hd_knee_paint(p, n):
    return hd_skin_at(p, dirt=0.75 * smooth(-0.36, -0.44, p.y) * smooth(0.0, 0.03, p.z))
part('hdThigh', hd_thigh, voxel=0.0042, tris=360, paint=hd_knee_paint)

def hd_shin(bm):                                    # 0.42, knee → ankle
    tube(bm, (0, 0.0, 0), (0, -0.41, 0), 0.03, 0.02)
    ell(bm, (0, -0.12, -0.026), (0.028, 0.09, 0.024))              # stringy calf
    ell(bm, (0, -0.2, 0.024), (0.009, 0.18, 0.008))
    tube(bm, (0, -0.22, -0.022), (0, -0.4, -0.026), 0.008, 0.007, seg=8)
    ell(bm, (0, -0.42, 0), (0.025, 0.02, 0.024))
part('hdShin', hd_shin, voxel=0.0036, tris=320, paint=hd_skin)

def hd_foot(bm):
    """ankle at the origin, 0.13 above the floor; heel raised 0.10, weight on
    the ball, toes curled and gripping, long nails. World-aligned in the rig."""
    ell(bm, (0, -0.024, -0.04), (0.025, 0.03, 0.03))                # heel
    ell(bm, (0, -0.05, 0.015), (0.03, 0.028, 0.05), rot=(0.55, 0, 0))   # arch
    ell(bm, (0, -0.09, 0.066), (0.038, 0.022, 0.045), rot=(0.45, 0, 0))
    ell(bm, (0, -0.113, 0.1), (0.045, 0.016, 0.028))                # ball, 0.09 wide
    for k in range(5):
        x = -0.03 + k * 0.015
        L = (0.052, 0.046, 0.041, 0.036, 0.03)[k]
        p0 = V((x, -0.112, 0.118)); p1 = p0 + V((x * 0.08, 0.004, L * 0.55)); p2 = p1 + V((0, -0.013, L * 0.35))
        chain(bm, [p0, p1, p2], [0.0105 if k == 0 else 0.0085, 0.0075, 0.0062], seg=8)
        tube(bm, p2, p2 + V((0, -0.005, 0.017)), 0.0052, 0.0012, seg=6)   # long nails
def hd_foot_paint(p, n):
    c = hd_skin_at(p, dirt=0.8 if n.y < -0.4 else 0.0)
    if p.z > 0.155: c = mix3(c, HD_CLAW, smooth(0.152, 0.165, p.z))
    return c
part('hdFoot', hd_foot, voxel=0.0028, smooth=3, noise=0.0006, tris=440, paint=hd_foot_paint)

# ---- hair: tapering, flattened tube clumps stuck together in wet points
def fib_dirs(n):
    g = math.pi * (3 - math.sqrt(5))
    for i in range(n):
        y = 1 - 2 * (i + 0.5) / n
        r = math.sqrt(1 - y * y)
        yield V((math.cos(g * i) * r, y, math.sin(g * i) * r))

def grow(root, d0, length, seg, collide, rng, grav=1.0, stiff=0.55):
    pts, d = [V(root)], V(d0).normalized()
    step = length / seg
    for i in range(seg):
        d = (d * stiff + V((0, -1, 0)) * grav * (0.35 + 0.08 * i)
             + V((rng.uniform(-0.12, 0.12), 0, rng.uniform(-0.12, 0.12)))).normalized()
        p = collide(pts[-1] + d * step)
        d = (p - pts[-1]).normalized() if (p - pts[-1]).length > 1e-6 else d
        pts.append(p)
    return pts

HD_EAR_H = [HD_HEAD_M @ V((s * HD_EAR_F.x, HD_EAR_F.y, HD_EAR_F.z)) for s in (-1, 1)]
HD_NECK_DIR = V((0, 0.62, -0.78)).normalized()      # from the atlas back up to the chest, head frame
def hd_hair_collide(p):
    p = push_out(p, HDH_SOLIDS + [((0, -0.03, 0.14), (0.045, 0.03, 0.09), (0, 0, 0))], 0.012, HD_HEAD_M)
    for e in HD_EAR_H:
        d = p - e
        if d.length < 0.03: p = e + d.normalized() * 0.03
    t = clamp(p.dot(HD_NECK_DIR), 0, 0.3)                          # the neck, as a capsule
    q = HD_NECK_DIR * t
    if (p - q).length < 0.05: p = q + (p - q).normalized() * 0.05
    return p

def wet_clump(bm, rng, guide, r_main, n_thin, spread, ref, sides=5, flat=0.5):
    """one wet clump along a guide path: a flat ribbon, and thin strands that
    peel away from it in the middle and stick back to it at the point"""
    n = len(guide)
    radii = [r_main * (1 - i / (n - 1)) ** 0.8 for i in range(n)]; radii[-1] = 0.0
    sweep(bm, guide, radii, sides=sides, flat=flat, ref=ref)
    side = (guide[-1] - guide[0]).cross(V(ref)).normalized() if (guide[-1] - guide[0]).cross(V(ref)).length > 1e-6 else V((1, 0, 0))
    for k in range(n_thin):
        off = (side * rng.uniform(-1, 1) + V((rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3)))) * spread
        pts = [guide[i] + off * (math.sin(math.pi * min(1.0, i / (n - 1) * 1.15)) ** 0.7) for i in range(n)]
        r0 = rng.uniform(0.0028, 0.0045)
        rr = [r0 * (1 - i / (n - 1)) ** 0.6 for i in range(n)]; rr[-1] = 0.0
        sweep(bm, pts, rr, sides=3, flat=0.8, ref=ref)

def hd_hair(bm):
    """the curtain: wet clumps from the crown, over the face and down both sides
    to just off the floor, stuck together in points"""
    rng = random.Random(8)
    cx, cr = V(HDH_SOLIDS[0][0]), HDH_SOLIDS[0][1]
    roots = []
    for u in fib_dirs(420):
        if u.y < -0.05: continue                                    # nothing below the ears
        if u.z > 0.35 and u.y < 0.55: continue                      # the forehead and face stay bare at the root
        if abs(u.x) > 0.8 and u.y < 0.3: continue                   # around the ears
        roots.append(u)
    rng.shuffle(roots)
    roots = roots[:72]
    # the scalp is covered: a thin layer of matted hair over the crown, under the clumps
    cap = bmesh.new()
    bmesh.ops.create_uvsphere(cap, u_segments=28, v_segments=16, radius=1.0)
    for v in cap.verts:
        u = v.co.normalized()
        v.co = cx + V((u.x * cr[0], u.y * cr[1], u.z * cr[2])) * 1.045
        v.tag = u.y < -0.02 or (u.z > 0.4 and u.y < 0.5) or (abs(u.x) > 0.84 and u.y < 0.25)
    bmesh.ops.delete(cap, geom=[v for v in cap.verts if v.tag], context='VERTS')
    xform(cap, HD_HEAD_M)
    me = bpy.data.meshes.new('hdcap'); cap.to_mesh(me); cap.free(); bm.from_mesh(me); bpy.data.meshes.remove(me)
    for u in roots:
        pf = cx + V((u.x * cr[0], u.y * cr[1], u.z * cr[2])) * 0.97
        p = HD_HEAD_M @ pf
        nrm = (HD_HEAD_M.to_3x3() @ u).normalized()
        comb = V((u.x * 0.6, 0.0, 0.9 if u.z > -0.2 else -0.3))     # fall forward over the face, down the sides
        tip_y = rng.uniform(-0.39, -0.28)                              # tips 0.07-0.16 m off the floor
        pts = grow(p, nrm * 0.5 + comb, 0.62, 14, hd_hair_collide, rng)
        cut = next((k for k in range(1, len(pts)) if pts[k].y <= tip_y), len(pts) - 1)
        if pts[cut].y < tip_y:                                          # end exactly at the tip height
            a, b = pts[cut - 1], pts[cut]
            pts[cut] = a.lerp(b, (a.y - tip_y) / max(1e-6, a.y - b.y))
        pts = pts[:cut + 1]
        pts = [pts[round(i * (len(pts) - 1) / 6)] for i in range(7)] if len(pts) >= 7 else pts
        if len(pts) < 3: continue
        wet_clump(bm, rng, pts, rng.uniform(0.008, 0.016), rng.choice((1, 1, 2)), rng.uniform(0.008, 0.016), nrm)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
part('hdHair', hd_hair, raw=True)

# The skinned body's trunk (tools/blender/skin_hound.py models it on these):
# sections along Z in the root frame (the spine joint, 0.70 m up): z,
# half-width, top y, bottom y. The mane roots on its back.
HD_SKIN_TRUNK = [
    (-0.458, 0.03, 0.128, 0.085), (-0.445, 0.08, 0.158, 0.04), (-0.41, 0.114, 0.183, 0.014),
    (-0.35, 0.136, 0.196, 0.004), (-0.28, 0.14, 0.19, 0.014), (-0.21, 0.13, 0.177, 0.03),
    (-0.145, 0.106, 0.158, 0.046), (-0.085, 0.102, 0.142, 0.03), (-0.03, 0.114, 0.126, -0.02),
    (0.04, 0.136, 0.108, -0.085), (0.12, 0.15, 0.092, -0.122), (0.2, 0.156, 0.078, -0.134),
    (0.27, 0.15, 0.064, -0.126), (0.33, 0.128, 0.048, -0.108), (0.375, 0.09, 0.03, -0.088),
    (0.405, 0.05, 0.014, -0.07), (0.42, 0.03, 0.006, -0.06),
]
HD_SKIN_SDF = [((0, (t + b) / 2, z), (w, (t - b) / 2, 0.045), (0, 0, 0)) for z, w, t, b in HD_SKIN_TRUNK[1:-1]]
def hd_back(z, x):
    w = interp(HD_SKIN_TRUNK, z)[0]
    a = math.copysign(math.asin(min(1.0, (abs(x) / w) ** 1.2)), x)
    return sec_pt_z(HD_SKIN_TRUNK, z, a)

# The mane: a scraggly band of wet clumps from the nape to the sacrum, lifting
# off the spine and falling over both sides, the skin showing between them.
# Three pieces, one per spine bone (the lumbar bone starts at z -0.08, the
# thorax at 0.08), so the band bends with the back; root frame.
def hd_mane_clumps():
    rng = random.Random(9)
    out = []
    for i in range(100):
        z = lerp(0.37, -0.385, i / 99) + rng.uniform(-0.005, 0.005)
        s = -1 if (i + (rng.random() < 0.2)) % 2 else 1
        x = s * rng.uniform(0.0, 0.024)
        L = rng.uniform(0.07, 0.17) * (1.3 if z < -0.24 or z > 0.28 else 1.0)
        out.append((z, x, s, L, rng.random()))
    return out
HD_MANE = hd_mane_clumps()
def hd_mane(lo, hi):
    def fn(bm):
        rng = random.Random(19)
        def col(p): return push_out(p, HD_SKIN_SDF, 0.008)
        for z, x, s, L, r in HD_MANE:
            rr = random.Random(int(r * 1e6))
            if not (lo <= z < hi): continue
            root = hd_back(z, x) + V((0, -0.005, 0))               # rooted in the skin
            d0 = V((s * rr.uniform(0.15, 0.55), rr.uniform(0.2, 0.5), -rr.uniform(0.75, 1.0)))   # lying back, a little off to a side
            pts = grow(root, d0, L, 6, col, rr, grav=0.4, stiff=0.8)
            wet_clump(bm, rr, pts, rr.uniform(0.0045, 0.008), rr.choice((0, 1, 1)), rr.uniform(0.004, 0.009), V((0, 1, 0)), sides=4, flat=0.6)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return fn
part('hdManeP', hd_mane(-1.0, -0.08), raw=True)
part('hdManeL', hd_mane(-0.08, 0.08), raw=True)
part('hdManeT', hd_mane(0.08, 1.0), raw=True)

def hd_tail(bm):
    """one segment of the trailing clump that stands in for a tail: wet
    strands stuck together, hanging from the end of the mane"""
    rng = random.Random(11)
    for i in range(7):
        a = TAU * i / 7 + rng.uniform(-0.3, 0.3)
        root = V((math.cos(a) * 0.016, 0.012, math.sin(a) * 0.016))
        pts = [root + V((math.cos(a) * 0.004 * k, -0.034 * k, -0.01 * k + rng.uniform(-0.004, 0.004))) for k in range(5)]
        wet_clump(bm, rng, pts, rng.uniform(0.006, 0.0095), 1, 0.006, V((math.cos(a), 0, math.sin(a))), sides=4, flat=0.6)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
part('hdTail', hd_tail, raw=True)


# =================================================================== WATCHER
# A dust sheet over someone standing very still. The sheet is solved, not
# lofted: a hidden figure (head with a real face, neck, shoulders and
# collarbones, chest, arms and hands) is posed for each of the game's
# WT_POSES, and one sheet surface is laid over it as a taut membrane: it
# rests on the high points (crown, brow, nose, cheekbones, chin, shoulders,
# knuckles), bridges the hollows (eye sockets, the gap under the nose and
# chin), hangs from its supports and flares to the floor, where it pools.
# Folds are then grown on the free cloth: tension folds radiating from every
# support, long hanging folds, catenary sags across the chest and back, one
# diagonal pull, and crumples and ripples in the pooled hem.
#
# The sheet is a grid of rays (rings round the column, a cap over the head),
# identical for every pose, so all the poses share one topology: the game
# blends them as morph targets on one mesh (wtBody = the first pose, with its
# triangles; wtPose1..4 = positions and colours, wtBow = positions). The
# fine grid is decimated once against every pose together (wt_qem), so the
# kept vertices are exactly grid points and each pose keeps its own detail.
#
# Frames: the floor under it, +Y up, +Z the way it faces, +X its left.
import numpy as np
import time, heapq

WT_BASE, WT_GRIME, WT_STAIN, WT_AO = hexc('#D8D0C0'), hexc('#857A68'), hexc('#B8AC94'), hexc('#9A9282')
WT_DUST, WT_YELLOW, WT_DAMP, WT_FRAY = hexc('#DAD6CE'), hexc('#D2C6AC'), hexc('#B4AA98'), hexc('#6E6556')

# The game's WT_POSES, in the same order: head roll (tilt), nod (pitch), the
# left arm (down, a hand to the chest, a hand at the mouth), forward lean.
WT_POSE_SPECS = [
    dict(tilt=0.0, pitch=0.21, arm='down', lean=0.0),     # upright, arms down
    dict(tilt=0.35, pitch=0.24, arm='down', lean=0.0),    # the head tilted 20 degrees
    dict(tilt=0.26, pitch=0.27, arm='chest', lean=0.0),   # a hand raised to the chest
    dict(tilt=0.1, pitch=0.2, arm='mouth', lean=0.0),     # a hand at the mouth
    dict(tilt=0.24, pitch=0.3, arm='mouth', lean=0.17),   # leaning forward ten degrees
]
WT_BOW = 0.25                    # the extra nod under a low ceiling (wtBow, from the first pose)
WT_THICK = 0.0035                # how far the cloth sits off the body


# ------------------------------------------------------------ numpy kit ---
def _rx(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])
def _rz(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])
def _nrm(v):
    v = np.asarray(v, float)
    return v / np.linalg.norm(v)
def _ss(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3 - 2 * t)
def _smin(a, b, k):
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0.0, 1.0)
    return b + (a - b) * h - k * h * (1 - h)
def _g2(x, y, x0, y0, sx, sy):
    return np.exp(-((x - x0) / sx) ** 2 - ((y - y0) / sy) ** 2)

class _Ell:
    """ellipsoid: centre c, radii r, rotation R (local -> world columns)"""
    def __init__(self, c, r, R=None):
        self.c, self.r = np.asarray(c, float), np.asarray(r, float)
        self.R = np.eye(3) if R is None else np.asarray(R, float)
        e = np.sqrt(((self.R * self.r) ** 2).sum(1))
        self.lo, self.hi = self.c - e, self.c + e
    def __call__(self, p):
        q = (p - self.c) @ self.R
        k0 = np.sqrt(((q / self.r) ** 2).sum(-1))
        k1 = np.sqrt(((q / self.r ** 2) ** 2).sum(-1))
        return k0 * (k0 - 1) / np.maximum(k1, 1e-9)

class _Cone:
    """round cone from a (radius ra) to b (radius rb)"""
    def __init__(self, a, b, ra, rb):
        self.a, self.b, self.ra, self.rb = np.asarray(a, float), np.asarray(b, float), ra, rb
        m = max(ra, rb)
        self.lo, self.hi = np.minimum(self.a, self.b) - m, np.maximum(self.a, self.b) + m
    def __call__(self, p):
        ba = self.b - self.a
        l2 = ba @ ba
        rr = self.ra - self.rb
        a2 = l2 - rr * rr
        il2 = 1.0 / l2
        pa = p - self.a
        y = pa @ ba
        z = y - l2
        x = pa * l2 - y[:, None] * ba
        x2 = (x * x).sum(-1)
        y2, z2 = y * y * l2, z * z * l2
        k = math.copysign(1, rr) * rr * rr * x2
        d3 = (np.sqrt(np.maximum(x2 * a2 * il2, 0)) + y * rr) * il2 - self.ra
        d1 = np.sqrt(x2 + z2) * il2 - self.rb
        d2 = np.sqrt(x2 + y2) * il2 - self.ra
        return np.where(np.sign(z) * a2 * z2 > k, d1, np.where(np.sign(y) * a2 * y2 < k, d2, d3))

class _Group:
    """smooth union of primitives (k), culled by bounding box"""
    def __init__(self, prims, k):
        self.prims, self.k = prims, k
        self.lo = np.min([p.lo for p in prims], 0) - k
        self.hi = np.max([p.hi for p in prims], 0) + k
    def __call__(self, p, lo=None, hi=None):
        d = None
        for pr in self.prims:
            if lo is not None and (np.any(pr.hi + self.k < lo) or np.any(pr.lo - self.k > hi)): continue
            v = pr(p)
            d = v if d is None else _smin(d, v, self.k)
        return np.full(len(p), 1.0) if d is None else d


# ------------------------------------------------------ the hidden figure ---
# Head frame: the head's centre, +Z out of the face. Sizes from the brief:
# about 0.19 wide x 0.25 tall x 0.22 deep; on its smooth mass, a brow ridge
# +0.01, eye sockets -0.016 (closed lids in them), the nose rising to +0.03
# at its tip, cheekbones, lips a little apart, a chin +0.014.
WTH_SOLIDS = [
    _Ell((0, 0.018, -0.012), (0.09, 0.108, 0.104)),       # cranium
    _Ell((0, -0.025, 0.02), (0.068, 0.088, 0.08)),        # the face
    _Ell((0, -0.075, 0.028), (0.056, 0.042, 0.062)),      # jaw and chin
    _Ell((0.05, -0.06, -0.01), (0.022, 0.035, 0.03)), _Ell((-0.05, -0.06, -0.01), (0.022, 0.035, 0.03)),
    _Ell((0.084, -0.004, -0.014), (0.009, 0.026, 0.016)), _Ell((-0.084, -0.004, -0.014), (0.009, 0.026, 0.016)),   # ears
]
WTH_ATLAS = np.array((0.0, 0.07, 0.03))   # the head's centre from the top of the neck, head frame

def wt_face_relief(x, y):
    """how far the face stands off its smooth mass (head frame, metres)"""
    ax = np.sqrt(x * x + 0.003 ** 2)                            # |x|, without a crease down the middle
    yb = 0.031 - 5.0 * (ax - 0.03) ** 2                          # the brows arch down at their outer ends
    r = 0.0095 * np.exp(-((ax - 0.03) / 0.023) ** 2 - ((y - yb) / 0.0115) ** 2)   # brow ridge, one over each eye
    r += 0.0035 * _g2(x, y, 0, 0.034, 0.011, 0.011)            # glabella
    r -= 0.0035 * _g2(x, y, 0, 0.017, 0.008, 0.006)            # the dip at the top of the nose
    r += 0.003 * _g2(x, y, 0, 0.068, 0.05, 0.03)               # forehead
    r -= 0.012 * _g2(ax, y, 0.032, 0.006, 0.018, 0.014)        # eye sockets: soft hollows,
    r -= 0.006 * _g2(ax, y, 0.027, 0.015, 0.012, 0.008)        # deepest under the brow
    r += 0.0095 * _g2(ax, y, 0.051, -0.016, 0.016, 0.011)      # cheekbones
    r -= 0.004 * _g2(ax, y, 0.047, -0.05, 0.018, 0.022)        # hollow cheeks
    r -= 0.006 * _g2(ax, y, 0.074, 0.026, 0.012, 0.022)        # temples
    t = np.clip((0.024 - y) / 0.06, 0, 1)                      # the nose: a bridge rising to the tip
    wn = 0.0085 + 0.006 * t
    hn = (0.005 + 0.023 * t ** 1.5) * _ss(0.03, 0.012, y) * _ss(-0.05, -0.034, y)
    r += hn * np.exp(-(x / wn) ** 2)
    r += 0.007 * _g2(x, y, 0, -0.037, 0.011, 0.0095)           # its tip, round
    r += 0.009 * _g2(ax, y, 0.0135, -0.041, 0.0068, 0.0068)    # alae
    r += 0.0065 * _g2(x, y, 0, -0.061, 0.02, 0.0085)           # upper lip
    r -= 0.0026 * _g2(x, y, 0, -0.0705, 0.016, 0.003)          # the lips a little apart
    r += 0.0062 * _g2(x, y, 0, -0.079, 0.018, 0.0075)          # lower lip
    r -= 0.004 * _g2(x, y, 0, -0.09, 0.015, 0.0042)            # under the lower lip
    r += 0.014 * _g2(x, y, 0, -0.104, 0.019, 0.012)            # chin
    t = np.clip((-0.042 - y) / 0.033, 0, 1)                    # nasolabial folds, alae to mouth corners
    d = np.abs(ax - (0.018 + 0.013 * t))
    r -= 0.0015 * np.exp(-(d / 0.0035) ** 2) * _ss(-0.036, -0.046, y) * _ss(-0.084, -0.07, y)
    return r

WT_HEAD_GROUP = _Group(WTH_SOLIDS, 0.022)
def wt_head_sdf(q):
    d = WT_HEAD_GROUP(q)
    m = _ss(0.0, 0.06, q[:, 2]) * (1 - _ss(0.075, 0.095, np.abs(q[:, 0])))
    return d - wt_face_relief(q[:, 0], q[:, 1]) * m


def _ik(S, W, a, b, pole):
    """the elbow of a two-bone limb from S to W, bending toward pole"""
    d = W - S
    L = float(np.linalg.norm(d))
    L = min(max(L, abs(a - b) + 1e-3), a + b - 1e-3)
    x = d / np.linalg.norm(d)
    p = pole - S
    y = _nrm(p - (p @ x) * x)
    m = (a * a - b * b + L * L) / (2 * L)
    return S + x * m + y * math.sqrt(max(a * a - m * m, 0.0))

WT_FINGERS = [   # across the knuckles (toward the thumb), segment lengths
    (0.025, (0.036, 0.022, 0.017)), (0.0082, (0.041, 0.025, 0.019)),
    (-0.0082, (0.038, 0.023, 0.018)), (-0.024, (0.03, 0.018, 0.015)),
]
def wt_hand(P, u, n, side, curl, spread=0.06):
    """palm centre P, u along the fingers, n the way the palm faces; side +1
    for the left hand. Returns the primitives and the knuckle points."""
    u = _nrm(u); n = _nrm(n - (n @ u) * u)
    w = _nrm(np.cross(n, u)) * side                  # toward the thumb
    R = np.stack([w, u, n], 1)
    prims = [_Ell(P, (0.036, 0.044, 0.0115), R)]
    knuckles = []
    for k, (o, segs) in enumerate(WT_FINGERS):
        a = (1.5 - k) * spread                       # the fingers fan a little
        d = _nrm(u * math.cos(a) + w * math.sin(a))
        q = P + u * 0.039 + w * o - n * 0.002
        knuckles.append(q - n * 0.006)
        prims.append(_Ell(q - n * 0.004, (0.0085, 0.008, 0.0078), R))   # the knuckle
        rad = 0.008
        for j, L in enumerate(segs):
            d = _nrm(d * math.cos(curl[j]) + n * math.sin(curl[j]))
            q2 = q + d * L
            prims.append(_Cone(q, q2, rad, rad - 0.0008))
            q, rad = q2, rad - 0.0008
    # the thumb, from the heel of the hand
    q = P - u * 0.014 + w * 0.028 + n * 0.005
    d = _nrm(u * 0.55 + w * 0.6 + n * 0.45)
    for j, (L, r0) in enumerate(((0.033, 0.011), (0.025, 0.0088), (0.019, 0.0076))):
        d = _nrm(d * math.cos(curl[j] * 0.6) + n * math.sin(curl[j] * 0.6))
        q2 = q + d * L
        prims.append(_Cone(q, q2, r0, r0 - 0.0012))
        q = q2
    return prims, knuckles


class WtFigure:
    """the figure under the sheet in one pose (world frame, metres)"""
    def __init__(self, spec, bow=0.0):
        self.spec = spec
        lean = spec['lean']
        self.pivot = np.array((0.0, 1.0, 0.0))
        self.Rt = _rx(lean)
        T = self.T
        self.atlas = T((0, 2.105, 0.012))
        self.Rh = self.Rt @ _rx(spec['pitch'] + bow) @ _rz(spec['tilt'])
        self.hc = self.atlas + self.Rh @ WTH_ATLAS          # the head's centre
        self.neck_base = T((0, 1.9, -0.025))
        torso = [
            _Ell((0, 1.0, -0.01), (0.15, 0.12, 0.105)), _Ell((0, 1.24, 0.0), (0.125, 0.17, 0.092)),
            _Ell((0, 1.58, -0.006), (0.14, 0.22, 0.104)), _Ell((0, 1.835, -0.012), (0.15, 0.07, 0.085)),
        ]
        bony = []
        for s in (-1, 1):
            torso += [_Ell((s * 0.058, 1.705, 0.05), (0.06, 0.045, 0.028)),              # pectorals
                      _Ell((s * 0.158, 1.885, -0.008), (0.046, 0.066, 0.053)),            # deltoids
                      _Ell((s * 0.075, 1.74, -0.082), (0.058, 0.085, 0.03))]              # shoulder blades
            bony += [_Cone((s * 0.02, 1.905, 0.062), (s * 0.15, 1.935, 0.008), 0.0105, 0.012),   # collarbones
                     _Cone((s * 0.03, 1.99, -0.032), (s * 0.14, 1.94, -0.02), 0.03, 0.026)]    # trapezius
        self.torso = _Group([self._place(p) for p in torso], 0.03)
        self.bony = _Group([self._place(p) for p in bony], 0.01)
        self.neck = _Cone(self.neck_base, self.atlas + self.Rh @ np.array((0, 0.03, 0.005)), 0.06, 0.052)
        # arms: the right one hangs; the left one follows the pose
        self.arms, self.hands, self.knuckles = [], [], []
        for s in (-1, 1):
            S = T((s * 0.16, 1.875, -0.012))
            mode = spec['arm'] if s > 0 else 'down'
            if mode == 'down':
                E, W = T((s * 0.19, 1.528, -0.035)), T((s * 0.205, 1.232, -0.005))
                u = self.Rt @ _nrm((s * 0.03, -1, 0.1)); n = self.Rt @ np.array((-s, 0.0, 0.0))
                P = W + u * 0.048
                hp, kn = wt_hand(P, u, n, s, (0.32, 0.38, 0.25))
            elif mode == 'chest':   # the hand flat on the breastbone, the elbow down at the side
                E = T((0.205, 1.53, 0.04))
                tgt = T((-0.035, 1.72, 0.128))
                W = E + _nrm(tgt - E) * 0.27
                u = self.Rt @ _nrm((-0.55, 0.83, 0.03)); n = self.Rt @ np.array((0.0, 0.0, -1.0))
                P = W + u * 0.045
                hp, kn = wt_hand(P, u, n, s, (0.08, 0.1, 0.06), spread=0.09)
            else:   # a hand over the mouth, the fingers spread across the far cheek, the elbow forward
                P = self.H((0.018, -0.072, 0.11))
                u = self.Rh @ _nrm((-0.85, 0.5, 0.0)); n = self.Rh @ np.array((0.0, 0.0, -1.0))
                hp, kn = wt_hand(P, u, n, s, (0.25, 0.32, 0.2), spread=0.05)
                W = P - u * 0.045
                E = self.T((0.165, 1.6, 0.14))
            self.arms.append(_Group([_Cone(S, E, 0.043, 0.036), _Cone(E, W + (E - W) * 0.04, 0.036, 0.027)], 0.012))
            self.hands.append(_Group(hp, 0.005))
            self.knuckles += kn
            if s > 0: self.hand_l, self.elbow_l, self.wrist_l = P, E, W
        self.crown = self.H((0, 0.126, -0.012))

    def T(self, p):
        """the torso's rest frame (the lean turns it about the hips) to world"""
        return self.pivot + self.Rt @ (np.asarray(p, float) - self.pivot)
    def H(self, p):
        """head frame to world"""
        return self.hc + self.Rh @ np.asarray(p, float)
    def _place(self, pr):
        if isinstance(pr, _Ell): return _Ell(self.T(pr.c), pr.r, self.Rt @ pr.R)
        return _Cone(self.T(pr.a), self.T(pr.b), pr.ra, pr.rb)

    def sdf(self, p, lo=None, hi=None):
        """signed distance (approximate) to the figure, for points p (M, 3)"""
        big = np.full(len(p), 1.0)
        def near(g, m=0.03):
            return lo is None or not (np.any(g.hi + m < lo) or np.any(g.lo - m > hi))
        d = self.torso(p, lo, hi) if near(self.torso) else big
        if near(self.bony): d = _smin(d, self.bony(p, lo, hi), 0.012)
        if near(self.neck): d = _smin(d, self.neck(p), 0.03)
        for a in self.arms:
            if near(a): d = _smin(d, a(p, lo, hi), 0.015)
        hlo, hhi = self.hc - 0.16, self.hc + 0.16
        if lo is None or not (np.any(hhi < lo) or np.any(hlo > hi)):
            q = (p - self.hc) @ self.Rh
            d = _smin(d, wt_head_sdf(q), 0.02)
        for h in self.hands:
            if near(h): d = np.minimum(d, h(p, lo, hi))
        return d


# ----------------------------------------------------------- the ray grid ---
# Rings of rays from bottom to top: round the column (horizontal rays from
# its axis), through the neck (the centre rising into the head while the
# rays tip downward), then a cap over the head (rays from the head's centre,
# from under the chin up to the crown). The same rings in every pose.
WT_YBASE = 0.16                  # where the column meets the pooled hem
WT_YTOP = 1.88                   # the last column ring, at the shoulders
WT_BMIN, WT_BMAX = -0.6, 1.52    # the head cap's ray elevations (head frame)

def wt_rings(res):
    """[(kind, value)] bottom to top; res scales the density"""
    ys = []
    y = WT_YBASE
    while y < WT_YTOP - 1e-6:
        ys.append(y)
        # finer round the hands, the chest and the shoulders
        h = 0.0085 - 0.0035 * max(math.exp(-((y - 1.12) / 0.14) ** 2), _ss(1.35, 1.6, y))
        y += h / res
    ys.append(WT_YTOP)
    rings = [('col', y) for y in ys]
    nn = int(round(22 * res))
    rings += [('neck', (k + 1) / (nn + 1)) for k in range(nn)]
    nb = int(round(110 * res))
    rings += [('head', WT_BMIN + (WT_BMAX - WT_BMIN) * (k / (nb - 1)) ** 1.08) for k in range(nb)]
    return rings

def _slerp(Ra, Rb, t):
    qa = Matrix(Ra.tolist()).to_quaternion(); qb = Matrix(Rb.tolist()).to_quaternion()
    return np.array(qa.slerp(qb, t).to_matrix())

def wt_frames(fig, rings):
    """ray origins C (J, 3), frames R (J, 3, 3) and elevations B (J,) for one pose"""
    lean = fig.spec['lean']
    C, R, B = [], [], []
    top = None
    for kind, v in rings:
        if kind == 'col':
            a = lean * _ss(1.0, 1.8, v)
            Ra = _rx(a)
            c = fig.pivot + Ra @ np.array((0.0, v - 1.0, 0.0)) if v > 1.0 else np.array((0.0, v, 0.0))
            c = c + np.array((0.0, 0.0, 0.04 * lean / 0.17 * (1 - _ss(0.2, 1.0, v))))   # the pooled hem is dragged forward a little
            C.append(c); R.append(Ra); B.append(0.0)
            top = (c, Ra)
        elif kind == 'neck':
            s = v * v * (3 - 2 * v)       # the centre rises before the rays tip down: no ring folds back
            C.append(top[0] + (fig.hc - top[0]) * v); R.append(_slerp(top[1], fig.Rh, s * s)); B.append(WT_BMIN * s)
        else:
            C.append(fig.hc.copy()); R.append(fig.Rh); B.append(v)
    return np.array(C), np.array(R), np.array(B)

def wt_dirs(R, B, th):
    """ray directions (J, N, 3)"""
    cb, sb = np.cos(B)[:, None], np.sin(B)[:, None]
    loc = np.stack([cb * np.sin(th)[None, :], np.broadcast_to(sb, (len(B), len(th))), cb * np.cos(th)[None, :]], -1)
    return np.einsum('jab,jnb->jna', R, loc)

def wt_obstacle(fig, C, D, tmax, step, tlim=None):
    """outermost t along each ray where the figure is (plus the cloth's
    thickness); 0 where a ray meets nothing. C (J, 3), D (J, N, 3); tlim
    (J, N) shortens some rays (the head cap's, where they point down at a
    shoulder the cloth never reaches from there)."""
    J, N, _ = D.shape
    out = np.zeros((J, N))
    ts = np.arange(0.0, tmax + 1e-9, step)
    nb = max(1, int(60000 // (N * len(ts))))
    for j0 in range(0, J, nb):
        j1 = min(J, j0 + nb)
        c = C[j0:j1, None, None, :]
        d = D[j0:j1, :, None, :]
        P = (c + d * ts[None, None, :, None]).reshape(-1, 3)
        lo, hi = P.min(0), P.max(0)
        f = fig.sdf(P, lo, hi).reshape(j1 - j0, N, len(ts))
        inside = f < WT_THICK
        if tlim is not None:
            # only surfaces the ray leaves before its limit: a body it is still
            # inside at the limit does not count (no flat cut through an arm)
            ext = np.concatenate([inside[..., :-1] & ~inside[..., 1:], inside[..., -1:]], -1)
            ext &= ts[None, None, :] + step <= tlim[j0:j1, :, None]
            inside = ext
        hit = inside.any(-1)
        last = len(ts) - 1 - np.argmax(inside[..., ::-1], -1)
        a = ts[last]
        b = np.minimum(a + step, tmax)
        cc, dd = C[j0:j1, None, :], D[j0:j1]
        for _ in range(7):   # bisect to a tenth of a millimetre
            m = 0.5 * (a + b)
            q = (cc + dd * m[..., None]).reshape(-1, 3)
            ins = (fig.sdf(q, lo, hi) < WT_THICK).reshape(m.shape)
            a = np.where(ins, m, a); b = np.where(ins, b, m)
        out[j0:j1] = np.where(hit, a, 0.0)
    return out


# ------------------------------------------------------------ the membrane ---
def _roll(a, k): return np.roll(a, k, axis=1)

def _vblur(a, n):
    """box blur along the rings (axis 0), n rings each way, edges clamped"""
    if n < 1: return a
    pad = np.concatenate([np.repeat(a[:1], n, 0), a, np.repeat(a[-1:], n, 0)], 0)
    c = np.cumsum(np.concatenate([np.zeros((1,) + a.shape[1:]), pad], 0), 0)
    return (c[2 * n + 1:] - c[:-2 * n - 1]) / (2 * n + 1)

def wt_close(o, rad, width):
    """morphological closing round one ring (periodic): gaps narrower than
    width (metres of arc at radius rad) are bridged"""
    N = len(o)
    k = int(np.clip(np.round(width / (np.median(rad) * TAU / N) / 2), 1, N // 4))
    m = o.copy()
    for s in range(1, k + 1): m = np.maximum.reduce([m, np.roll(o, s), np.roll(o, -s)])
    c = m.copy()
    for s in range(1, k + 1): c = np.minimum.reduce([c, np.roll(m, s), np.roll(m, -s)])
    return np.maximum(c, o)

def wt_slope(o, C, D, k_side, k_up, k_down, n=160):
    """lift the obstacle so the cloth never falls off a protrusion like a
    cliff: sideways and upward it tents back at about k (metres per metre),
    downward it hangs, closing in only gently"""
    J, N = o.shape
    ref = np.maximum(o, 0.1)
    X = C[:, None, :] + D * ref[..., None]
    dt = np.linalg.norm(np.roll(X, -1, axis=1) - X, axis=-1)
    du = np.linalg.norm(X[1:] - X[:-1], axis=-1)
    e = o.copy()
    for _ in range(n):
        e2 = np.maximum.reduce([e, np.roll(e, 1, axis=1) - k_side * np.roll(dt, 1, axis=1), np.roll(e, -1, axis=1) - k_side * dt])
        e2[1:] = np.maximum(e2[1:], e2[:-1] - k_up * du)        # above a protrusion
        e2[:-1] = np.maximum(e2[:-1], e2[1:] - k_down * du)     # below it
        if np.array_equal(e2, e): break
        e = e2
    return e

def wt_membrane(C, D, o, r0, aniso, band, press, iters=900, omega=1.85, rounds=2):
    """the taut sheet: a radius r >= o along each ray. Each sweep moves a
    point along its ray to the weighted mean of its four neighbours (the 3D
    Laplacian, projected on the ray), so a ring held round the shoulders
    pulls in like a band while the hanging cloth falls straight between
    its supports. aniso weighs the hoop tension against the vertical; band
    (0..1 per ring) is how far the hoop acts like a band (1) rather than only
    evening out the radius (0: hanging cloth has no hoop tension); press
    (1/m) is the curvature the cloth may take into a hollow. Ring 0 is fixed;
    the metric comes from a smoothed copy of the last solution."""
    J, N = o.shape
    jj, ii = np.meshgrid(np.arange(J), np.arange(N), indexing='ij')
    colour = ((jj + ii) % 2).astype(bool)
    free = jj > 0
    r = np.maximum(r0, o).copy()
    dp = D[-1].mean(0); dp /= np.linalg.norm(dp)
    # neighbour terms: (X_n - C_j) . d_ji = a + b r_n
    Cu = np.concatenate([C[1:], C[-1:]], 0); Du = np.concatenate([D[1:], np.broadcast_to(dp, (1, N, 3))], 0)
    Cd = np.concatenate([C[:1], C[:-1]], 0); Dd = np.concatenate([D[:1], D[:-1]], 0)
    aU = ((Cu - C)[:, None, :] * D).sum(-1); bU = (Du * D).sum(-1)
    aD = ((Cd - C)[:, None, :] * D).sum(-1); bD = (Dd * D).sum(-1)
    bE = (_roll(D, -1) * D).sum(-1); bW = (_roll(D, 1) * D).sum(-1)
    bE = 1 + (bE - 1) * band[:, None]; bW = 1 + (bW - 1) * band[:, None]
    for rnd in range(rounds):
        ref = _vblur(r, 12)
        X = C[:, None, :] + D * ref[..., None]
        pole = C[-1] + dp * ref[-1].mean()
        up = np.concatenate([X[1:], pole[None, None, :].repeat(N, 1)], 0)
        du = np.maximum(np.linalg.norm(up - X, axis=-1), 1e-3)
        dt = np.maximum(np.linalg.norm(_roll(X, -1) - X, axis=-1), 1e-3)
        wU = 1.0 / du ** 2
        wT = aniso / dt ** 2
        wD = np.concatenate([np.zeros((1, N)), wU[:-1]], 0)
        wW = _roll(wT, 1)
        den = wU + wD + wT + wW
        for it in range(iters):
            for par in (False, True):
                m = (colour == par) & free
                rU = np.concatenate([r[1:], np.full((1, N), r[-1].mean())], 0)
                rD = np.concatenate([r[:1], r[:-1]], 0)
                new = (wU * (aU + bU * rU) + wD * (aD + bD * rD) + wT * bE * _roll(r, -1) + wW * bW * _roll(r, 1) - press) / den
                r = np.where(m, np.maximum(r + omega * (new - r), o), r)
    return r


# ----------------------------------------------------------------- folds ---
def wt_metric(X):
    """vertical (ring j to j+1) and sideways (ray i to i+1) spacing, metres"""
    du = np.linalg.norm(np.concatenate([X[1:] - X[:-1], np.zeros((1,) + X.shape[1:])], 0), axis=-1)
    du[-1] = du[-2]
    dt = np.linalg.norm(_roll(X, -1) - X, axis=-1)
    return np.maximum(du, 1e-4), np.maximum(dt, 1e-4)

def wt_dist(seed, du, dt, n=400):
    """distance over the sheet from the seed cells (chamfer relaxation)"""
    D = np.where(seed, 0.0, 1e3)
    dd = np.sqrt(du ** 2 + dt ** 2)
    for _ in range(n):
        up = np.concatenate([D[1:], np.full((1, D.shape[1]), 1e3)], 0) + du
        dn = np.concatenate([np.full((1, D.shape[1]), 1e3), D[:-1]], 0) + np.concatenate([du[:1], du[:-1]], 0)
        e = _roll(D, -1) + dt
        w = _roll(D, 1) + _roll(dt, 1)
        ue = np.concatenate([_roll(D, -1)[1:], np.full((1, D.shape[1]), 1e3)], 0) + dd
        uw = np.concatenate([_roll(D, 1)[1:], np.full((1, D.shape[1]), 1e3)], 0) + dd
        de = np.concatenate([np.full((1, D.shape[1]), 1e3), _roll(D, -1)[:-1]], 0) + dd
        dw = np.concatenate([np.full((1, D.shape[1]), 1e3), _roll(D, 1)[:-1]], 0) + dd
        D2 = np.minimum.reduce([D, up, dn, e, w, ue, uw, de, dw])
        if np.allclose(D2, D): break
        D = D2
    return D

def _wrap(a): return (a + math.pi) % TAU - math.pi

def wt_hang(touch, du, dt, spread=1.2):
    """how far each cell hangs below what holds it up: straight down from a
    support, or across and down from one nearby (a cone under each support)"""
    J, N = touch.shape
    H = np.zeros((J, N))
    for j in range(J - 2, -1, -1):
        h = np.where(touch[j], 0.0, H[j + 1] + du[j])
        for _ in range(6):
            h = np.minimum.reduce([h, np.roll(h, 1) + spread * np.roll(dt[j], 1), np.roll(h, -1) + spread * dt[j]])
        H[j] = h
    return H

def _lblur(a, n):
    """blur round the rings (axis 1), periodic"""
    if n < 1: return a
    out = a.copy()
    for _ in range(3):   # three box passes ~ a gaussian
        acc = out.copy()
        for k in range(1, n + 1): acc += _roll(out, k) + _roll(out, -k)
        out = acc / (2 * n + 1)
    return out

def wt_streams(H, du, dt, th, fan=0.9):
    """for every cell, the angle at the bottom ring of the fold line through
    it. Fold lines hang straight down, and fan out below a support: they
    move sideways away from where the cloth is held lowest."""
    J, N = H.shape
    gx = (_roll(H, -1) - _roll(H, 1)) / (dt + _roll(dt, 1))                       # d(hang)/d(sideways)
    slope = np.clip(fan * gx * np.exp(-H / 0.5), -1.2, 1.2)                       # sideways metres per metre down
    rho = dt / (TAU / N)                                                          # metres per radian round the ring
    dev = np.zeros((J, N))                                                        # bottom angle - own angle
    di = TAU / N
    for j in range(1, J):
        shift = slope[j] * du[j - 1] / np.maximum(rho[j], 1e-3)                   # radians, one ring down
        shift = np.clip(shift, -3 * di, 3 * di)
        x = (np.arange(N) + shift / di) % N
        i0 = np.floor(x).astype(int); f = x - i0; i1 = (i0 + 1) % N
        d0, d1 = dev[j - 1, i0], dev[j - 1, i1]
        d1 = d0 + _wrap(d1 - d0)
        dev[j] = _wrap(d0 + (d1 - d0) * f + shift)
    # where the lines part under a support the angle jumps: spread the jump
    # over a few centimetres so the folds there crowd (and fade) instead of
    # tearing into a crease
    return th[None, :] + _lblur(dev, 7)

WT_FOLD_WARP = [(2, 1.3, 0.55), (3, 0.2, 0.35), (5, 2.2, 0.25), (7, 4.0, 0.12)]
def wt_fold_phase(phi, n):
    """the folds' phase round the bottom ring: n of them, irregularly spaced"""
    w = n * phi
    for k, ph, a in WT_FOLD_WARP: w = w + a * np.sin(k * phi + ph + n * 0.37)
    return w

def wt_fold_shape(w, sharp):
    """one fold across: -1 in the valley, +1 on the crest; sharp=0 rounds the
    crest (hanging cloth), 1 sharpens it (a fold under tension)"""
    s = np.sqrt(np.sin(w / 2) ** 2 + 0.02) - 0.07
    c = np.sqrt(np.cos(w / 2) ** 2 + 0.006) - 0.04
    return (1 - sharp) * (2 * s - 1) + sharp * (1 - 2 * c)

def wt_alias(w, dt):
    """0 where the folds crowd closer than a few centimetres (where a fan of
    them meets its support), 1 where they are well apart"""
    dw = np.abs(_wrap(_roll(w, -1) - w)) / dt
    lam = TAU / np.maximum(_lblur(dw, 1), 1e-3)
    return _ss(0.018, 0.045, lam)


WT_SAGS = [   # catenary sags across the chest: centre angle, top (m), half-width (rad), depth, amplitude, width, tilt
    (0.08, 1.92, 1.05, 0.09, 0.009, 0.02, 0.03), (-0.1, 1.93, 1.25, 0.2, 0.016, 0.028, -0.05),
    (0.15, 1.9, 1.3, 0.36, 0.011, 0.036, 0.08),
]
WT_PULL = [(-1.05, 1.86), (-0.55, 1.55), (0.1, 1.2), (0.55, 0.85)]   # (angle, height) along the diagonal pull
def wt_pull(th, y, rho):
    """one long diagonal fold dragged from the right shoulder across the
    front toward the left hip: it breaks the drape's symmetry"""
    F = np.zeros(np.broadcast(th, y).shape)
    best = np.full(F.shape, 9.0); along = np.zeros(F.shape)
    for k in range(len(WT_PULL) - 1):
        (t0, y0), (t1, y1) = WT_PULL[k], WT_PULL[k + 1]
        ax, ay = _wrap(th - t0) * rho, y - y0
        bx, by = (t1 - t0) * rho, y1 - y0
        L2 = bx * bx + by * by
        t = np.clip((ax * bx + ay * by) / L2, 0, 1)
        d = np.hypot(ax - t * bx, ay - t * by) * np.sign(ax * by - ay * bx)
        closer = np.abs(d) < np.abs(best)
        best = np.where(closer, d, best); along = np.where(closer, (k + t) / (len(WT_PULL) - 1), along)
    v = best / 0.028
    env = np.sin(np.pi * np.clip(along, 0, 1)) ** 0.6
    return 0.014 * env * (np.exp(-v * v) - 0.6 * np.exp(-((v - 1.4) / 0.8) ** 2))

def wt_sags(th, y):
    """U-shaped cowl folds hanging between the shoulders: a
    rounded ridge along each curve with a shadowed groove under it"""
    F = np.zeros(np.broadcast(th, y).shape)
    for k, (tc, y0, tw, dep, amp, w, tilt) in enumerate(WT_SAGS):
        u = _wrap(th - tc) / tw
        inside = np.clip(1 - u * u, 0, 1)
        v = (y - (y0 + tilt * u - dep * inside)) / w            # not quite level, not quite even
        along = 0.55 + 0.45 * np.sin(2.3 * u + 1.7 * k) ** 2
        F += amp * along * inside ** 0.7 * (np.exp(-v * v) - 0.85 * np.exp(-((v + 1.3) / 0.9) ** 2))
    return F

WT_FACE_PULL = [   # where the face holds the cloth up: (x, y) head frame, reach, strength
    ((0.0, -0.038), 0.03, 1.0), ((0.0, -0.106), 0.035, 0.9), ((0.051, -0.017), 0.025, 0.6), ((-0.051, -0.017), 0.025, 0.6),
]
def wt_face_creases(hl):
    """fine tension creases radiating from the face's high points, mostly
    downward, where the cloth is pulled over them (head frame points)"""
    x, y, z = hl[..., 0], hl[..., 1], hl[..., 2]
    F = np.zeros(x.shape)
    for k, ((fx, fy), reach, st) in enumerate(WT_FACE_PULL):
        dx, dy = x - fx, y - fy
        d = np.hypot(dx, dy)
        a = np.arctan2(dy, dx)
        down = 0.35 + 0.65 * np.clip(-np.sin(a), 0, 1)
        F += st * 0.0013 * np.sin(9 * a + 1.7 * k + 4 * np.sin(2 * a + k)) * np.exp(-d / reach) * _ss(0.006, 0.014, d) * down
    return F * _ss(0.03, 0.07, z)


# --------------------------------------------------------- one pose's sheet ---
WT_N_HI = 288
def wt_theta(N): return np.arange(N) * (TAU / N) - math.pi    # from the back, round through the front

def wt_base_r(th):
    """the column's radius where it reaches the pool"""
    a, b = 0.262, 0.2
    r = 1 / np.sqrt((np.sin(th) / a) ** 2 + (np.cos(th) / b) ** 2)
    return r * (1 + 0.02 * np.sin(2 * th + 0.7) + 0.015 * np.sin(3 * th + 2.1))

def wt_pose_grid(spec, res, bow=0.0):
    fig = WtFigure(spec, bow)
    rings = wt_rings(res)
    N = int(round(WT_N_HI * res / 2)) * 2
    th = wt_theta(N)
    C, R, B = wt_frames(fig, rings)
    D = wt_dirs(R, B, th)
    kinds = np.array([k for k, _ in rings])
    o = np.zeros((len(rings), N))
    for kind, tm, st in (('col', 0.45, 0.004), ('neck', 0.32, 0.0025), ('head', 0.24, 0.0022)):
        sel = np.where(kinds == kind)[0]
        tl = None
        side = _ss(0.75, 1.25, np.abs(th))[None, :]     # rays toward a shoulder, not over the chest
        if kind == 'head': tl = 0.24 - 0.09 * _ss(-0.25, -0.65, D[sel][..., 1]) * side
        if kind == 'neck': tl = 0.32 - 0.16 * _ss(-0.15, -0.55, D[sel][..., 1]) * side
        o[sel] = wt_obstacle(fig, C[sel], D[sel], tm, st, tl)
    rb = wt_base_r(th)
    col = kinds == 'col'
    # cloth cannot drop into a gap narrower than about a hand's width (between
    # a raised forearm and the shoulder, an arm and the ribs): close the
    # obstacle round each ring of the column and neck
    for j in np.where(kinds != 'head')[0]:
        o[j] = wt_close(o[j], np.maximum(o[j], 0.08), 0.11 if kinds[j] == 'col' else 0.06)
    # nor can it drop off a protrusion like a cliff: from anything that stands
    # out (an elbow, a hand) it tents back at about forty degrees
    nh = np.where(kinds != 'head')[0]
    o[nh] = wt_slope(o[nh], C[nh], D[nh], 1.1, 1.0, 0.3)
    # a first guess: hanging straight down from whatever holds it up
    h = o.copy()
    ci = np.where(col)[0]
    h[ci] = np.maximum.accumulate(o[ci][::-1], 0)[::-1]
    r0 = np.maximum(h, np.where(col[:, None], 0.6 * rb[None, :], 0.06))
    r0[0] = rb
    o[0] = rb
    hv = np.array([v for _, v in rings])
    aniso = np.where(col, 0.35 + 0.65 * _ss(1.6, 1.85, hv), 1.0)[:, None] * np.ones((1, N))
    band = np.where(col, _ss(1.74, 1.86, hv), 1.0)
    press = np.zeros_like(o)
    face = (kinds == 'head')[:, None] * _ss(-0.72, -0.5, hv)[:, None] * (1 - _ss(0.6, 0.85, hv))[:, None] \
        * (1 - _ss(0.95, 1.3, np.abs(th)))[None, :]
    press += 18.0 * face
    Xo = C[:, None, :] + D * o[..., None]
    for k, K in enumerate(fig.knuckles):
        hard = 40.0 if (k >= 4 and spec['arm'] != 'down') else 14.0     # a hand pressing, or one hanging
        press += hard * np.exp(-(np.linalg.norm(Xo - K, axis=-1) / 0.045) ** 2) * (o > 0)
    r = wt_membrane(C, D, o, r0, aniso, band, press, iters=int(700 + 500 * res), omega=1.9)
    # heavy cloth will not take a knife-edge: round the column's ridges a little
    nh = kinds != 'head'
    r[nh] = np.maximum(_lblur(r[nh], 2), o[nh])
    r[0] = rb
    X = C[:, None, :] + D * r[..., None]
    g = dict(X=X, r=r, o=o, C=C, D=D, th=th, rings=rings, kinds=kinds, fig=fig, hv=hv)
    wt_folds(g)
    return g

def _mod(phi, seed):
    """a slow random 0..1 strength along the bottom ring"""
    return np.clip(0.55 + 0.22 * np.sin(3 * phi + seed) + 0.18 * np.sin(7 * phi + 2.9 * seed) + 0.12 * np.sin(11 * phi + 5.1 + seed), 0, 1)

def wt_folds(g):
    """grow folds on the free cloth: g['F'] (radial, metres) and the sheet g['Xf']"""
    X, r, o, th, kinds = g['X'], g['r'], g['o'], g['th'], g['kinds']
    J, N = r.shape
    du, dt = wt_metric(X)
    touch = (r - o < 0.0012) & (o > 0)
    touch[0] = False
    Dc = wt_dist(touch, du, dt)
    y = X[..., 1]
    # only what bears the sheet's weight seeds the hanging folds: the head
    # and neck, and contacts whose surface faces up (shoulder tops, the top of
    # a raised forearm), not the cloth sliding past an arm or a hanging hand
    tu = np.concatenate([X[1:2] - X[0:1], X[2:] - X[:-2], X[-1:] - X[-2:-1]], 0)
    tv = _roll(X, -1) - _roll(X, 1)
    nm = np.cross(tv, tu)
    nm *= np.sign((nm * g['D']).sum(-1, keepdims=True) + 1e-12)
    ny = nm[..., 1] / np.maximum(np.linalg.norm(nm, axis=-1), 1e-9)
    holds = touch & ((kinds != 'col')[:, None] | ((ny > 0.45) & (y > 1.35)))
    H = _lblur(wt_hang(holds, du, dt, 1.6), 2)
    phi = wt_streams(_lblur(H, 7), du, dt, th)   # a smooth field: the lines part gently under a support
    head = (kinds == 'head')[:, None]
    slack = (1 - np.exp(-H / 0.17)) * (1 - np.exp(-Dc / 0.025))
    hang = 0.7 + 0.55 * np.clip(1 - y / 2.0, 0, 1)
    w1 = wt_fold_phase(phi, 14)
    A1 = 0.034 * slack * hang * (0.25 + 0.75 * _mod(phi, 1.3)) * wt_alias(w1, dt)
    F = A1 * wt_fold_shape(w1, np.exp(-H / 0.15))
    w2 = wt_fold_phase(phi + 0.11, 31)
    A2 = 0.0085 * (1 - np.exp(-Dc / 0.02)) * np.exp(-H / 0.5) * _mod(phi, 4.2) * wt_alias(w2, dt)
    F += A2 * wt_fold_shape(w2, 0.75)
    free = 1 - np.exp(-Dc / 0.02)
    F += wt_sags(th[None, :], y) * free * (kinds == 'col')[:, None]
    rho = np.hypot(X[..., 0] - g['C'][:, None, 0], X[..., 2] - g['C'][:, None, 2])
    F += wt_pull(th[None, :], y, rho) * free * (kinds == 'col')[:, None]
    hl = (X - g['fig'].hc) @ g['fig'].Rh
    F += wt_face_creases(hl) * head
    # where it rests on the head the cloth is taut: only faint creases from the crown
    F += touch * head * 0.0012 * _mod(phi, 2.2) * wt_fold_shape(wt_fold_phase(phi, 23), 0.9) * _ss(1.3, 2.0, np.abs(th))[None, :]
    rf = r + F
    rf = np.where(o > 0, np.maximum(rf, o + 0.0008), np.maximum(rf, 0.03))
    rf[0] = r[0] + F[0]
    g.update(F=F, Dc=Dc, H=H, phi=phi, touch=touch, holds=holds, rf=rf, w1=w1, A1=A1)
    g['Xf'] = g['C'][:, None, :] + g['D'] * rf[..., None]


# ------------------------------------------------------- the pooled hem ---
WT_TEARS = [(2.25, 0.05, 0.055), (-0.95, 0.035, 0.04)]     # angle, half-width (rad), depth (m)

def wt_pool(g, nb, nf):
    """rings from the hem's edge on the floor up to the column's foot: the
    cloth bends onto the floor and runs out 6-20 cm, crumpled where the
    folds land, its edge scalloped, rolled and torn in two places"""
    Xf, th, C, F = g['Xf'], g['th'], g['C'], g['F']
    c0 = C[0]
    P0 = Xf[0]
    rel = P0 - c0
    rho0 = np.hypot(rel[:, 0], rel[:, 2])
    ux, uz = rel[:, 0] / rho0, rel[:, 2] / rho0
    t = Xf[0] - Xf[1]                                           # down the cloth at its foot
    tr = t[:, 0] * ux + t[:, 2] * uz
    ty = t[:, 1]
    tl = np.hypot(tr, ty)
    tr, ty = tr / tl, ty / tl
    F0 = F[0]
    crest = _lblur(np.clip(F0 / 0.02, -1, 1)[None, :], 4)[0]
    out = 0.055 + 0.012 * np.sin(3 * th + 0.4) + 0.012 * np.clip(crest, 0, 1)
    E = (0.105 + 0.03 * np.sin(2 * th + 0.5) + 0.022 * np.sin(3 * th + 2.1) + 0.016 * np.sin(4 * th + 4.4)
         + 0.012 * np.sin(5 * th + 1.0) + 0.008 * np.sin(7 * th + 2.6) + 0.006 * crest
         + 0.004 * np.sin(23 * th + 0.3) + 0.0025 * np.sin(41 * th + 1.7))
    for ta, tw, dep in WT_TEARS:
        E -= dep * np.exp(-(_wrap(th - ta) / tw) ** 2)
    E = np.maximum(E, 0.03)
    rho1 = rho0 + out
    rho1s = _lblur(rho1[None, :], 7)[0]                          # out on the floor the folds' lobes relax
    rho2 = rho1s + E
    y0 = P0[:, 1]
    rings, qs = [], []
    # the bend onto the floor: a cubic from the column's foot to the floor
    for k in range(1, nb + 1):
        u = k / nb
        q0r, q0y = rho0 + tr * 0.06, y0 + ty * 0.06
        q1r, q1y = rho1 - 0.04, np.full_like(rho0, 0.012)
        b = [(1 - u) ** 3, 3 * (1 - u) ** 2 * u, 3 * (1 - u) * u * u, u ** 3]
        rr = b[0] * rho0 + b[1] * q0r + b[2] * q1r + b[3] * rho1
        yy = b[0] * y0 + b[1] * q0y + b[2] * q1y + b[3] * 0.012
        rings.append((rr, yy)); qs.append(0.4 * u)
    # the run over the floor to the edge, crumpled
    for k in range(1, nf + 1):
        u = k / nf
        rr = rho1 + (rho1s - rho1) * _ss(0.0, 0.8, u) + (rho2 - rho1s) * u
        h = (0.009 * (1 - u) + 0.016 * np.clip(crest, 0, None) * (1 - u) ** 0.7
             + 0.006 * np.clip(np.sin(9 * th + 6 * u + 1.3) * np.sin(4 * th - 3 * u), 0, None))
        lip = 0.0045 * _ss(0.72, 0.9, u) * (1 - _ss(0.93, 1.0, u))           # the rolled hem
        yy = np.maximum(h * (1 - _ss(0.85, 1.0, u)) + lip, 0.0) + 0.0015 * (1 - u)
        if k == nf: yy = np.zeros_like(rr)
        rings.append((rr, yy)); qs.append(0.4 + 0.6 * u)
    pts = []
    for rr, yy in rings:
        pts.append(np.stack([c0[0] + rr * ux, yy, c0[2] + rr * uz], -1))
    return np.array(pts[::-1]), np.array(qs[::-1])            # from the edge inward

def wt_sheet(spec, res, bow=0.0):
    """the whole sheet for one pose: (rings, N, 3) from the hem's edge to the
    crown's last ring, the crown point, and what the paint needs"""
    g = wt_pose_grid(spec, res, bow)
    nb, nf = max(4, int(round(8 * res))), max(4, int(round(8 * res)))
    pool, q = wt_pool(g, nb, nf)
    G = np.concatenate([pool, g['Xf']], 0)
    dp = g['fig'].Rh @ np.array((0.0, 1.0, 0.0))
    pole = g['C'][-1] + dp * float(g['rf'][-1].mean())
    kinds = np.array(['pool'] * len(pool) + list(g['kinds']))
    hv = np.concatenate([q, g['hv']])
    zeros = np.zeros((len(pool), G.shape[1]))
    aux = dict(kinds=kinds, hv=hv,
               touch=np.concatenate([zeros, g['touch'] * 1.0], 0),
               F=np.concatenate([zeros + g['F'][0], g['F']], 0),
               o=np.concatenate([zeros, g['o']], 0), fig=g['fig'])
    return G, pole, aux


# ------------------------------------------- one mesh, every pose on it ---
def _grid_tris(R, N):
    """triangles of a (R rings x N rays) grid plus a crown point, outward"""
    r = np.arange(R - 1)[:, None]; i = np.arange(N)[None, :]
    a = r * N + i; b = r * N + (i + 1) % N; c = (r + 1) * N + (i + 1) % N; d = (r + 1) * N + i
    t = np.concatenate([np.stack([a, b, c], -1).reshape(-1, 3), np.stack([a, c, d], -1).reshape(-1, 3)], 0)
    top = (R - 1) * N
    fan = np.stack([top + np.arange(N), top + (np.arange(N) + 1) % N, np.full(N, R * N)], -1)
    return np.concatenate([t, fan], 0)

def _vnormals(P, T):
    fn = np.cross(P[T[:, 1]] - P[T[:, 0]], P[T[:, 2]] - P[T[:, 0]])
    n = np.zeros_like(P)
    for k in range(3): np.add.at(n, T[:, k], fn)
    return n / np.maximum(np.linalg.norm(n, axis=-1, keepdims=True), 1e-12)

def _bilinear(Gx, rf, fi):
    """sample a (R+1, N, 3) grid (last ring = the crown point) at fractional
    ring rf and ray fi (periodic)"""
    R1, N = Gx.shape[:2]
    r0 = np.clip(np.floor(rf).astype(int), 0, R1 - 2); fr = np.clip(rf - r0, 0, 1)[:, None]
    i0 = np.floor(fi).astype(int); ff = (fi - i0)[:, None]; i0 %= N; i1 = (i0 + 1) % N
    return ((1 - fr) * ((1 - ff) * Gx[r0, i0] + ff * Gx[r0, i1]) + fr * ((1 - ff) * Gx[r0 + 1, i0] + ff * Gx[r0 + 1, i1]))

def wt_smooth_vals(v, T, n=2):
    """average a per-vertex value with its neighbours, n times"""
    nb = np.zeros(len(v)); cnt = np.zeros(len(v))
    for _ in range(n):
        nb[:] = 0; cnt[:] = 0
        for a, b in ((0, 1), (1, 2), (2, 0)):
            np.add.at(nb, T[:, a], v[T[:, b]]); np.add.at(cnt, T[:, a], 1)
            np.add.at(nb, T[:, b], v[T[:, a]]); np.add.at(cnt, T[:, b], 1)
        v = 0.5 * v + 0.5 * nb / np.maximum(cnt, 1)
    return v

def wt_ao(P, T, n_rays=26, reach=0.26):
    """baked occlusion per vertex, 0 open .. 1 shut in (ray-cast)"""
    from mathutils.bvhtree import BVHTree
    bvh = BVHTree.FromPolygons(P.tolist(), T.tolist())
    nrm = _vnormals(P, T)
    rng = np.random.default_rng(7)
    u1, u2 = (np.arange(n_rays) + 0.5) / n_rays, rng.random(n_rays)
    loc = np.stack([np.sqrt(u1) * np.cos(TAU * u2), np.sqrt(u1) * np.sin(TAU * u2), np.sqrt(1 - u1)], -1)
    occ = np.zeros(len(P))
    for k in range(len(P)):
        n = nrm[k]
        a = np.array((1.0, 0, 0)) if abs(n[0]) < 0.9 else np.array((0, 1.0, 0))
        t1 = np.cross(n, a); t1 /= np.linalg.norm(t1); t2 = np.cross(n, t1)
        o = Vector(P[k] + n * 0.002)
        s = 0.0
        for l in loc:
            d = t1 * l[0] + t2 * l[1] + n * l[2]
            hit = bvh.ray_cast(o, Vector(d), reach)
            if hit[0] is not None: s += 1.0 - 0.6 * hit[3] / reach
        occ[k] = s / n_rays
    return wt_smooth_vals(occ, T, 3)

def wt_qem(Ps, T, target, fixed, boost=None):
    """decimate one mesh against several shapes of it at once: half-edge
    collapses ranked by the summed quadric error over every shape (each pose),
    refused where any shape would fold a face over. Ps (K, n, 3); T (m, 3);
    fixed (n,) vertices never removed; boost (n,) multiplies the cost of
    removing a vertex (detail worth more triangles). Returns the kept vertex
    indices and the triangles over them (old indices)."""
    K, n, _ = Ps.shape
    T = T.copy()
    m = len(T)
    # plane quadrics, area weighted, per shape
    Q = np.zeros((K, n, 4, 4))
    for k in range(K):
        A, B, Cc = Ps[k][T[:, 0]], Ps[k][T[:, 1]], Ps[k][T[:, 2]]
        nf = np.cross(B - A, Cc - A)
        ar = np.linalg.norm(nf, axis=-1)
        nn = nf / np.maximum(ar, 1e-15)[:, None]
        pl = np.concatenate([nn, -(nn * A).sum(-1, keepdims=True)], -1)
        q = pl[:, :, None] * pl[:, None, :] * (0.5 * ar)[:, None, None]
        for c in range(3): np.add.at(Q[k], T[:, c], q)
    boost = np.ones(n) if boost is None else boost
    vf = [set() for _ in range(n)]
    for f, (a, b, c) in enumerate(T):
        vf[a].add(f); vf[b].add(f); vf[c].add(f)
    alive_f = np.ones(m, bool)
    alive_v = np.ones(n, bool)
    ver = np.zeros(n, int)
    Ph = np.concatenate([Ps, np.ones((K, n, 1))], -1)          # homogeneous
    def cost(u, v):
        p = Ph[:, v]
        return float(np.einsum('ki,kij,kj->', p, Q[:, u] + Q[:, v], p)) * boost[u]
    # every directed edge, its first cost in bulk
    E = np.concatenate([T[:, [0, 1]], T[:, [1, 2]], T[:, [2, 0]]], 0)
    E = np.unique(np.concatenate([E, E[:, ::-1]], 0), axis=0)
    E = E[~fixed[E[:, 0]]]
    heap = []
    for s0 in range(0, len(E), 40000):
        e = E[s0:s0 + 40000]
        p = Ph[:, e[:, 1]]
        c = np.einsum('kei,keij,kej->e', p, Q[:, e[:, 0]] + Q[:, e[:, 1]], p) * boost[e[:, 0]]
        heap += list(zip(c.tolist(), e[:, 0].tolist(), e[:, 1].tolist(), [0] * len(e), [0] * len(e)))
    heapq.heapify(heap)
    faces = m
    def nbrs(x):
        s = set()
        for f in vf[x]: s.update(T[f])
        s.discard(x)
        return s
    while faces > target and heap:
        c, u, v, vu, vv = heapq.heappop(heap)
        if not (alive_v[u] and alive_v[v]) or ver[u] != vu or ver[v] != vv: continue
        fu = vf[u]
        shared = [f for f in fu if v in T[f]]
        if len(shared) != 2: continue                              # u on the edge of the sheet, or broken
        # link condition: u and v share exactly the two opposite corners
        opp = set()
        for f in shared: opp.update(T[f])
        opp -= {u, v}
        if nbrs(u) & nbrs(v) != opp: continue
        moved = [f for f in fu if f not in shared]
        if moved:
            tri = T[moved]
            ok = True
            for k in range(K):
                P = Ps[k]
                A, B, Cc = P[tri[:, 0]], P[tri[:, 1]], P[tri[:, 2]]
                n0 = np.cross(B - A, Cc - A)
                tri2 = np.where(tri == u, v, tri)
                A2, B2, C2 = P[tri2[:, 0]], P[tri2[:, 1]], P[tri2[:, 2]]
                n1 = np.cross(B2 - A2, C2 - A2)
                l0, l1 = np.linalg.norm(n0, axis=-1), np.linalg.norm(n1, axis=-1)
                if np.any(l1 < 1e-12) or np.any((n0 * n1).sum(-1) < 0.35 * l0 * l1): ok = False; break
            if not ok: continue
        # collapse u into v
        for f in shared:
            alive_f[f] = False
            for x in T[f]: vf[x].discard(f)
        faces -= len(shared)
        for f in moved:
            T[f][T[f] == u] = v
            vf[v].add(f)
        vf[u] = set()
        alive_v[u] = False
        Q[:, v] += Q[:, u]
        ver[v] += 1                     # only v's quadric changed: re-rank the edges at v
        for w in nbrs(v):
            if not fixed[v]: heapq.heappush(heap, (cost(v, w), v, w, ver[v], ver[w]))
            if not fixed[w]: heapq.heappush(heap, (cost(w, v), w, v, ver[w], ver[v]))
    keep = np.where(alive_v)[0]
    return keep, T[alive_f]


def wt_albedo(V, nrm, rf, fi, aux, N, fig):
    """the sheet's own colour (sRGB): old cotton, yellowed in patches, dust
    on what faces up, grime rising from the floor, water-stain rings, damp
    where the face presses, a frayed and torn edge"""
    R = len(aux['kinds'])
    ring = np.clip(np.round(rf).astype(int), 0, R - 1)
    kind = aux['kinds'][ring]
    q = np.where(kind == 'pool', aux['hv'][ring], 0.0)
    th = fi * TAU / N - math.pi
    ext = lambda G: np.concatenate([G, G[-1:]], 0)[..., None]
    touch = _bilinear(ext(aux['touch']), rf, fi)[:, 0]
    hl = (V - fig.hc) @ fig.Rh
    relief = wt_face_relief(hl[:, 0], hl[:, 1])
    rho = np.hypot(V[:, 0], V[:, 2])
    cols = []
    base, yel, dust, grime, stain, damp, fray = (np.array(c) for c in (WT_BASE, WT_YELLOW, WT_DUST, WT_GRIME, WT_STAIN, WT_DAMP, WT_FRAY))
    for k in range(len(V)):
        p = Vector(V[k]); y = V[k, 1]
        c = base * (1 + 0.035 * mnoise.noise(p * 7) + 0.02 * mnoise.noise(p * 23))
        t = 0.5 + 0.5 * mnoise.noise(p * 2.3 + Vector((3.1, 0.7, 5.2)))
        c = c + (yel - c) * 0.45 * smooth(0.5, 0.85, t)
        if y > 0.3: c = c + (dust - c) * 0.45 * smooth(0.45, 0.95, nrm[k, 1])
        streak = 0.75 + 0.25 * mnoise.noise(Vector((p.x * 18, y * 2.5, p.z * 18)))
        gr = smooth(0.4, 0.0, y) ** 1.3 * 0.8 * streak
        if kind[k] == 'pool': gr = max(gr, 0.55 + 0.35 * q[k])
        c = c + (grime - c) * min(gr, 0.92)
        for ts, ys, rs in WT_STAINS:   # faint tide-line rings
            d = math.hypot(_wrap(th[k] - ts) * max(rho[k], 0.1), y - ys)
            c = c + (stain - c) * (0.6 * math.exp(-((d - rs) / 0.006) ** 2) + 0.14 * (d < rs))
        if kind[k] == 'head' and hl[k, 2] > 0.04:   # the cloth goes damp where the face presses it
            c = c + (damp - c) * 0.35 * touch[k] * smooth(0.002, 0.012, relief[k])
        if q[k] > 0.86:   # the frayed edge, worse at the tears
            tear = max(math.exp(-(_wrap(th[k] - ta) / (tw * 1.6)) ** 2) for ta, tw, dep in WT_TEARS)
            f = smooth(0.86, 1.0, q[k]) * (0.45 + 0.4 * tear) * (0.6 + 0.4 * mnoise.noise(p * 140))
            c = c + (fray - c) * f
        cols.append(np.clip(c, 0, 1))
    return np.array(cols)

WT_STAINS = [(0.75, 0.66, 0.075), (-2.35, 0.98, 0.06), (0.2, 1.42, 0.045), (2.9, 0.36, 0.09)]

def wt_cavity(P, T):
    """0 on a ridge .. 1 down in a crease, from each vertex's neighbours"""
    nrm = _vnormals(P, T)
    acc = np.zeros(len(P)); cnt = np.zeros(len(P))
    for a, b in ((0, 1), (1, 2), (2, 0), (1, 0), (2, 1), (0, 2)):
        d = P[T[:, b]] - P[T[:, a]]
        l = np.maximum(np.linalg.norm(d, axis=-1), 1e-9)
        np.add.at(acc, T[:, a], (nrm[T[:, a]] * d).sum(-1) / l); np.add.at(cnt, T[:, a], 1)
    c = np.clip(acc / np.maximum(cnt, 1) * 3.5, 0, 1)
    return wt_smooth_vals(c, T, 1)

def wt_shade(alb, occ, cav):
    """darken by the baked occlusion and the creases, toward a cool grey"""
    a = np.clip(occ * 1.45 + cav * 0.45, 0, 1)[:, None] ** 0.85
    deep = alb * np.array(WT_AO) / np.array(WT_BASE) * 0.56
    return alb + (deep - alb) * a

def wt_build(res=1.0, target=19000):
    t0 = time.time()
    specs = [(sp, 0.0) for sp in WT_POSE_SPECS] + [(WT_POSE_SPECS[0], WT_BOW)]
    sheets = [wt_sheet(sp, res, bow) for sp, bow in specs]
    print('watcher: sheets', round(time.time() - t0, 1), 's')
    R, N = sheets[0][0].shape[:2]
    grids = [np.concatenate([G, np.repeat(pole[None, None, :], N, 1)], 0) for G, pole, aux in sheets]
    T = _grid_tris(R, N)
    P0 = np.concatenate([sheets[0][0].reshape(-1, 3), sheets[0][1][None, :]], 0)
    # spend triangles where they matter: the face, and wherever a pose changes
    # the sheet's shape (the hand at the chest and the mouth); elsewhere the
    # summed error over the poses decides
    th = wt_theta(N)
    aux0 = sheets[0][2]
    kap = []
    for G in grids:
        G = G[:R]
        lap = G - 0.25 * (np.concatenate([G[:1], G[:-1]], 0) + np.concatenate([G[1:], G[-1:]], 0) + _roll(G, 1) + _roll(G, -1))
        h2 = np.maximum(np.linalg.norm(_roll(G, -1) - G, axis=-1) * np.linalg.norm(np.concatenate([G[1:], G[-1:]], 0) - G, axis=-1), 1e-8)
        kap.append(_lblur(np.linalg.norm(lap, axis=-1) / h2, 2))
    curv = np.max(kap, 0)
    change = np.max([np.abs(k - kap[0]) for k in kap[1:]], 0)      # where a pose changes the sheet's shape (not where it only moves it)
    hb = aux0['hv']
    face = ((aux0['kinds'] == 'head')[:, None] * (1 - _ss(-0.65, -0.45, -hb))[:, None] * (1 - _ss(0.85, 1.25, np.abs(th)))[None, :])
    keep = np.clip(curv / 400.0, 0, 0.45)
    keep = np.maximum(keep, 0.92 * face)
    keep = np.maximum(keep, 0.95 * _ss(20.0, 120.0, _vblur(change, 3)) * (aux0['kinds'] != 'pool')[:, None])
    keep = np.concatenate([keep.ravel(), [1.0]])
    Pall = np.array([np.concatenate([G[:R].reshape(-1, 3), G[R:R + 1, 0]], 0) for G in grids])
    fixed = np.zeros(R * N + 1, bool); fixed[:N] = True; fixed[-1] = True      # the hem's edge, the crown point
    kv, F = wt_qem(Pall, T, target, fixed, 1.0 + 60.0 * keep)
    remap = np.full(R * N + 1, -1); remap[kv] = np.arange(len(kv))
    F = remap[F]
    Ps = [P[kv] for P in Pall]
    prm = np.stack([np.minimum(kv // N, R).astype(float), (kv % N).astype(float)], -1)
    print('watcher: decimated to', len(F), 'tris', len(kv), 'verts', round(time.time() - t0, 1), 's')
    alb = wt_albedo(Ps[0], _vnormals(Ps[0], F), prm[:, 0], prm[:, 1], aux0, N, aux0['fig'])
    cols = [wt_shade(alb, wt_ao(P, F), wt_cavity(P, F)) for P in Ps]
    print('watcher: painted', round(time.time() - t0, 1), 's')
    heads = [(aux['fig'].hc, aux['fig'].Rh) for G, pole, aux in sheets]
    return dict(F=F, P=Ps, C=cols, heads=heads)

WT_TRIS = 19000                  # the sheet's triangle budget (one mesh, every pose)
_WT = {}
def wt_built():
    """build once, only when a Watcher part is asked for"""
    if not _WT: _WT.update(wt_build(1.0, WT_TRIS))
    return _WT

def wt_emit(k, faces, colours=True):
    """pose k's vertices in the shared order (the first pose with the
    triangles too), and its colours: the morph targets' baked shading"""
    def fn(bm):
        W = wt_built()
        vs = [bm.verts.new(V(p)) for p in W['P'][k]]
        if faces:
            for t in W['F']:
                try: bm.faces.new((vs[t[0]], vs[t[1]], vs[t[2]]))
                except ValueError: pass
        return [tuple(c) for c in W['C'][k]] if colours else None
    return fn

part('wtBody', wt_emit(0, True), raw=True)
for _k in range(1, 5): part(f'wtPose{_k}', wt_emit(_k, False), raw=True)
part('wtBow', wt_emit(5, False, colours=False), raw=True)   # the bowed head shades as the first pose does
if _WT:   # each pose's head centre from the rig's neck joint (0, 2.02, 0.03): WT_POSES[k].head, WT_BOW_HEAD
    for _k, (_hc, _Rh) in enumerate(_WT['heads']):
        print('RIG watcher', 'bow' if _k == 5 else f'pose {_k}', 'head', [round(float(v), 3) for v in _hc - np.array((0.0, 2.02, 0.03))])


# ================================================================== THE FACE
# A face shell, not a head. It is a height field z(x, y) over the front of a
# face (the Face always turns to look at you, so nothing behind it exists):
# a rounded base with anatomy blended in as smooth bumps and grooves, blank
# eyeballs under heavy open lids, a real nose with a sealed underside, a lip
# mound with no seam, forehead creases and nasolabial folds. Everything past
# the temples and jaw is painted down to black. Origin at the eye line.
FC_E, FC_ER, FC_EZ = 0.032, 0.012, 0.035          # eye centres, eyeball radius, eyeball centre depth
def g2(x, y, x0, y0, sx, sy): return math.exp(-((x - x0) / sx) ** 2 - ((y - y0) / sy) ** 2)
def smax(a, b, k):
    h = clamp(0.5 + 0.5 * (a - b) / k)
    return lerp(b, a, h) + k * h * (1 - h)
def poly_d(x, y, pts):
    best = 1e9
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        dx, dy = bx - ax, by - ay
        t = clamp(((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy))
        best = min(best, math.hypot(x - ax - t * dx, y - ay - t * dy))
    return best
FC_NASOLABIAL = [[(s * 0.019, -0.038), (s * 0.025, -0.055), (s * 0.03, -0.072)] for s in (-1, 1)]
FC_CREASES = [(0.058, 0.03, 0.4), (0.071, 0.036, -0.7)]
def fc_eye(x, y):
    """which eye (sign), u across the opening (-1..1), dy from its axis, upper/lower half-heights"""
    s = 1 if x > 0 else -1
    u = (x - s * FC_E) / 0.0152
    yc = 0.0008 * u * s
    return s, u, y - yc, 0.0058 * max(0.0, 1 - u * u) ** 0.75, 0.0052 * max(0.0, 1 - u * u) ** 0.8
def fc_open(x, y):
    s, u, dy, hu, hl = fc_eye(x, y)
    return abs(u) < 1 and -hl < dy < hu
def fc_skin(x, y):
    rr = (x / 0.086) ** 2 + ((y + 0.006) / 0.132) ** 2
    z = 0.062 * max(0.0, 1 - rr) ** 0.45 - 0.004
    z += 0.003 * g2(x, y, 0, 0.07, 0.04, 0.03)                                   # forehead
    for s in (-1, 1):
        z += 0.0065 * g2(x, y, s * 0.029, 0.021, 0.021, 0.0065)                  # brow ridge, 0.008 proud
        z -= 0.011 * g2(x, y, s * FC_E, 0.001, 0.02, 0.013)                      # the orbit
        z += 0.005 * g2(x, y, s * 0.046, -0.011, 0.016, 0.009)                   # cheekbones
        z -= 0.004 * g2(x, y, s * 0.043, -0.045, 0.013, 0.014)                   # hollow cheeks
        z -= 0.004 * g2(x, y, s * 0.066, 0.03, 0.012, 0.02)                      # temples
        z += 0.003 * g2(x, y, s * FC_E, -0.0145, 0.012, 0.0038)                  # puffy bags under the eyes
        z -= 0.0012 * math.exp(-((y + 0.0205 + 0.003 * max(0.0, 1 - ((x - s * FC_E) / 0.014) ** 2)) / 0.0012) ** 2) \
             * max(0.0, 1 - ((x - s * FC_E) / 0.016) ** 2)                      # the crease under each bag
        d = poly_d(x, y, FC_NASOLABIAL[0 if s < 0 else 1])                         # nasolabial folds
        z -= 0.0013 * math.exp(-(d / 0.0032) ** 2)
    z += 0.003 * g2(x, y, 0, 0.018, 0.009, 0.008)                                # glabella
    # the nose: a bridge rising to a tip, alae, and a sealed underside
    t = clamp((0.012 - y) / 0.052)
    wn = lerp(0.0065, 0.011, t)
    hn = lerp(0.006, 0.034, t ** 1.3) * smooth(0.02, 0.008, y) * smooth(-0.055, -0.037, y) ** 0.8
    z += hn * math.exp(-(x / wn) ** 2)
    z += 0.004 * g2(x, y, 0, -0.035, 0.009, 0.008)
    for s in (-1, 1): z += 0.01 * g2(x, y, s * 0.0125, -0.041, 0.006, 0.0065)
    z -= 0.001 * g2(x, y, 0, -0.051, 0.0022, 0.005)                              # philtrum
    z += 0.0045 * g2(x, y, 0, -0.0625, 0.019, 0.0068)                            # the lip mound, no seam
    z -= 0.002 * g2(x, y, 0, -0.076, 0.013, 0.004)
    z += 0.006 * g2(x, y, 0, -0.093, 0.016, 0.012)                               # chin
    for yc, w, ph in FC_CREASES:                                                  # forehead creases
        if abs(x) < w:
            k = (1 - (x / w) ** 2) ** 0.5
            z -= 0.0018 * k * math.exp(-((y - yc - 0.0025 * math.cos(x / w * 1.4) - 0.0012 * math.sin(x * 90 + ph)) / 0.0015) ** 2)
    return z
def fc_z(x, y):
    z = fc_skin(x, y)
    if abs(abs(x) - FC_E) < 0.03 and abs(y) < 0.03:
        s, u, dy, hu, hl = fc_eye(x, y)
        dx = x - s * FC_E
        t = lerp(0.0016, 0.0034, smooth(-0.004, 0.004, dy))                         # the heavy upper lid
        lid = FC_EZ + math.sqrt(max(0.0, (FC_ER + t) ** 2 - dx * dx - dy * dy))
        z = smax(z, lid, 0.004)
        z -= 0.001 * math.exp(-((dy - 0.0105) / 0.0012) ** 2) * max(0.0, 1 - (dx / 0.015) ** 2)   # the fold over the lid
        if fc_open(x, y):                                                          # a blank eyeball, flush in the lids
            z = FC_EZ + math.sqrt(max(0.0, FC_ER ** 2 - dx * dx - dy * dy))
    return z
def fc_face(bm):
    h = 0.0011
    nx, ny = int(0.176 / h), int(0.272 / h)
    grid = {}
    for i in range(nx + 1):
        for j in range(ny + 1):
            x, y = -0.088 + i * h, -0.142 + j * h
            if (x / 0.088) ** 2 + ((y + 0.006) / 0.136) ** 2 <= 1.0:
                grid[i, j] = bm.verts.new((x, y, fc_z(x, y)))
    for i in range(nx):
        for j in range(ny):
            q = [grid.get(k) for k in ((i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1))]
            if all(q): bm.faces.new(q)
FC_SKIN, FC_SHADOW = hexc('#D6D0C6'), hexc('#8E8A88')
FC_EYEBALL, FC_LID, FC_BAGC = hexc('#EFECE6'), hexc('#B4A8A0'), hexc('#8C8078')
def fc_paint(p, n):
    x, y = p.x, p.y
    c = FC_SKIN
    s, u, dy, hu, hl = fc_eye(x, y)
    if abs(u) < 1.25 and -0.008 < dy < 0.011: c = mix3(FC_SKIN, FC_LID, 0.8 * math.exp(-(u / 1.1) ** 4))
    c = mix3(c, FC_BAGC, 0.55 * g2(x, y, s * FC_E, -0.016, 0.012, 0.004))
    if fc_open(x, y) and p.z > FC_EZ + 0.004: c = FC_EYEBALL
    cav = fc_skin(x, y) - (0.062 * max(0.0, 1 - (x / 0.086) ** 2 - ((y + 0.006) / 0.132) ** 2) ** 0.45 - 0.004)
    if c is FC_SKIN: c = mix3(c, FC_SHADOW, clamp(-cav / 0.012) * 0.5)          # hollows and creases go cool
    c = mix3(c, FC_SHADOW, 0.15 * clamp(-n.y - 0.5))                                # undersides
    c = mul3(c, 1 + 0.04 * mnoise.noise(p * 55) + 0.012 * mnoise.noise(p * 400))    # faint blotching, pores
    rho = math.sqrt((x / 0.08) ** 2 + ((y + 0.012) / 0.124) ** 2)
    return mul3(c, (1 - smooth(0.52, 0.99, rho)) ** 1.3)                           # the rim runs to black
part('stairFace', fc_face, ops=[], tris=4400, sym=True, paint=fc_paint)


# ------------------------------------------------------------------ export ---
def pack(verts, idx, cols):
    mn = [min(v[k] for v in verts) for k in range(3)]
    mx = [max(v[k] for v in verts) for k in range(3)]
    pos = bytearray()
    for v in verts:
        for k in range(3):
            span = (mx[k] - mn[k]) or 1.0
            pos += struct.pack('<h', round((v[k] - mn[k]) / span * 65534) - 32767)
    ind = bytearray()
    for f in idx:
        for i in f: ind += struct.pack('<H', i)
    out = {'min': [round(x, 5) for x in mn], 'max': [round(x, 5) for x in mx],
           'v': base64.b64encode(bytes(pos)).decode(), 'i': base64.b64encode(bytes(ind)).decode(),
           'n': len(verts), 't': len(idx)}
    if cols:
        out['c'] = base64.b64encode(bytes(round(clamp(ch) * 255) for c in cols for ch in c)).decode()
    return out

def locked(path, fn):
    """run fn while holding path + '.lock': several builds (skinned bodies,
    rigid parts) rewrite the same output file"""
    import time
    lock = path + '.lock'
    for _ in range(900):
        try:
            fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY); break
        except FileExistsError:
            time.sleep(0.1)
    else:   # a stale lock from a killed run
        os.remove(lock); fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    try: fn()
    finally:
        os.close(fd); os.remove(lock)

def export():
    locked(OUT, _export)

def _export():
    lines = {}
    if ONLY is not None and os.path.exists(OUT):
        for ln in open(OUT).read().split('\n'):
            m = re.match(r'^  (\w+): \{', ln)
            if m: lines[m.group(1)] = ln
    for name, (v, i, cols) in PARTS.items():
        p = pack(v, i, cols)
        c = f", c: '{p['c']}'" if 'c' in p else ''
        lines[name] = f"  {name}: {{ n: {p['n']}, t: {p['t']}, min: {p['min']}, max: {p['max']}, v: '{p['v']}', i: '{p['i']}'{c} }},"
    with open(OUT, 'w') as f:
        f.write('/*\n * Generated by tools/blender/creatures.py — do not edit by hand.\n'
                ' * int16 positions (quantized to each part\'s bounding box), uint16\n'
                ' * triangle indices and optional sRGB vertex colours, base64.\n'
                ' * Decoded by src/creatures.js.\n */\n')
        f.write('export const CREATURE_MESHES = {\n')
        for name, ln in lines.items(): f.write(ln + '\n')
        f.write('};\n')
    print('wrote', OUT, 'parts', len(lines))

if not globals().get('NO_EXPORT'): export()

# numbers the rigs copy (head frames)
def r3(v): return '(' + ', '.join(f'{x:.4f}' for x in v) + ')'
print('RIG smiler eye', r3((SMH_EYE_X, SMH_EYE_Y, SMH_EYE_Z)), 'hinge', r3(SMH_HINGE))
print('RIG hound eye', r3(HD_HEAD_M @ HD_EYE_F), 'hinge', r3(HD_HEAD_M @ HD_HINGE_F),
      'ear', r3(HD_HEAD_M @ HD_EAR_F), 'pitch', HD_PITCH)
