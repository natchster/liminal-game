"""
Skinned creature bodies: one continuous, weighted surface per creature
instead of a rigid part per joint.

    /Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
        --python assets/src/blender/skinned.py -- smiler [OUT]

The rig is a dump of the creature's rest pose (.cache/rigs/<type>.json:
every bone's name, parent, rest matrix, head and tail), so the body is
modelled in exactly the pose the game binds it in. Nothing in this
repository writes those dumps, so the bodies made here are kept as stored
(`creature_meshes.bin`) and these scripts are the record of how.

A body is a list of pieces. A piece is built by a function in some bone's
frame (most are creatures.py's part builders, which already model each limb
in its joint's frame) and placed by that bone's rest matrix; it weights to
one bone, or spreads over several along their segments (a torso over the
spine bones). The pieces are fused by a voxel remesh, in two resolutions
(the body, and the fine extremities whose fingers and toes would otherwise
melt together), smoothed, roughened, decimated and painted. Every vertex is
then weighted to the pieces it lies on: a soft nearest-piece choice, so a
knee blends thigh and shin over a couple of centimetres while an arm hanging
by the thigh never drags the leg's skin.

Output: one part `<prefix>Body` with four bone indices and weights per
vertex and the bone names, which the game binds to the creature's rig
(`entities::skin_body`).
"""
import bpy, bmesh, sys, os, math, json, base64, struct, re
from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
ARGS = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
TYPE = ARGS[0] if ARGS else 'smiler'
OUT = ARGS[1] if len(ARGS) > 1 else os.path.join(ROOT, 'src', 'creature-meshes.js')

# creatures.py as a library: its helpers and part builders, building nothing
_argv = sys.argv
sys.argv = ['blender', '--', os.devnull, '__none__']
C = {'__name__': 'creatures', 'NO_EXPORT': True}
exec(compile(open(os.path.join(HERE, 'creatures.py')).read(), os.path.join(HERE, 'creatures.py'), 'exec'), C)
sys.argv = _argv
V = Vector

RIG = json.load(open(os.path.join(ROOT, '.cache', 'rigs', TYPE + '.json')))
BONES = {b['name']: b for b in RIG['bones']}
BONE_ORDER = [b['name'] for b in RIG['bones']]

def bone_matrix(name):
    e = BONES[name]['matrix']   # column-major
    return Matrix(((e[0], e[4], e[8], e[12]), (e[1], e[5], e[9], e[13]), (e[2], e[6], e[10], e[14]), (0, 0, 0, 1)))

def seg_dist(p, a, b):
    a, b = V(a), V(b)
    d = b - a
    L2 = d.length_squared
    t = 0.0 if L2 < 1e-12 else max(0.0, min(1.0, (p - a).dot(d) / L2))
    return (p - (a + d * t)).length


class Piece:
    def __init__(self, build, frame, bones=None, group='body', local=None, sig=0.04):
        self.build, self.frame = build, frame
        self.bones = bones or [frame]
        self.group, self.local, self.sig = group, local or Matrix(), sig

    def make(self):
        bm = bmesh.new()
        self.build(bm)
        M = bone_matrix(self.frame) @ self.local
        C['xform'](bm, M)
        if M.to_3x3().determinant() < 0:   # a mirrored frame (the right hand): keep the faces outward
            bmesh.ops.reverse_faces(bm, faces=bm.faces[:])
        bm.normal_update()
        self.bm = bm
        self.bvh = BVHTree.FromBMesh(bm)
        cs = [v.co for v in bm.verts]
        self.lo = V((min(c.x for c in cs), min(c.y for c in cs), min(c.z for c in cs)))
        self.hi = V((max(c.x for c in cs), max(c.y for c in cs), max(c.z for c in cs)))

    def near(self, p, pad):
        return all(self.lo[k] - pad <= p[k] <= self.hi[k] + pad for k in range(3))


def fuse(pieces, name, voxel, smooth, noise, tris):
    bm = bmesh.new()
    for pc in pieces:
        me = bpy.data.meshes.new('tmp')
        pc.bm.to_mesh(me)
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me); bm.free()
    ops = [('remesh', voxel)]
    if smooth: ops.append(('smooth', smooth[0], smooth[1]))
    if noise: ops.append(('noise', noise))
    me = C['run_ops'](name, me, ops)
    return C['finish'](me, tris, True, False)


SIG_PIECE = 0.011      # how far a joint's blend reaches across pieces (m)

def joined(a, b):
    """two bones that meet at a joint: the same, parent and child, or two
    roots (a pelvis and a thigh hung from the creature itself)"""
    pa, pb = BONES[a]['parent'], BONES[b]['parent']
    return a == b or pa == b or pb == a or (pa is None and pb is None)

def weigh(verts, pieces):
    """four (bone index, weight 0..255) per vertex. A vertex blends only
    between pieces whose bones meet at a joint: a curled claw that rests
    by the knee, or two fingers side by side, must not borrow each other's
    weight just because they are close."""
    out = []
    for p in verts:
        ds = [(pc.bvh.find_nearest(p)[3], pc) for pc in pieces if pc.near(p, 0.12)]
        if not ds: ds = [(pc.bvh.find_nearest(p)[3], pc) for pc in pieces]
        dmin, own = min(ds, key=lambda t: t[0])
        acc = {}
        for d, pc in ds:
            if d - dmin > 0.05: continue
            if pc is not own and not any(joined(a, b) for a in own.bones for b in pc.bones): continue
            wp = math.exp(-(d - dmin) / SIG_PIECE)
            if len(pc.bones) == 1:
                acc[pc.bones[0]] = acc.get(pc.bones[0], 0.0) + wp
                continue
            sd = [(seg_dist(p, BONES[b]['head'], BONES[b]['tail']), b) for b in pc.bones]
            m = min(x for x, _ in sd)
            ws = [(math.exp(-(x - m) / pc.sig), b) for x, b in sd]
            tot = sum(w for w, _ in ws)
            for w, b in ws: acc[b] = acc.get(b, 0.0) + wp * w / tot
        top = sorted(acc.items(), key=lambda kv: -kv[1])[:4]
        tot = sum(w for _, w in top)
        q = [(BONE_ORDER.index(b), w / tot) for b, w in top]
        # quantize to bytes that sum to exactly 255
        bytes_ = [int(round(w * 255)) for _, w in q]
        bytes_[0] += 255 - sum(bytes_)
        while len(q) < 4: q.append((0, 0.0)); bytes_.append(0)
        out.append([(q[i][0], bytes_[i]) for i in range(4)])
    return out


def concavity(verts, norms, idx):
    """per-vertex 0 (convex) .. 1 (a deep crease), from the neighbours"""
    nb = [set() for _ in verts]
    for a, b, c in idx:
        nb[a].update((b, c)); nb[b].update((a, c)); nb[c].update((a, b))
    out = []
    for i, p in enumerate(verts):
        if not nb[i]: out.append(0.0); continue
        s = 0.0
        for j in nb[i]:
            d = verts[j] - p
            l = d.length
            if l > 1e-9: s += norms[i].dot(d) / l
        out.append(max(0.0, min(1.0, s / len(nb[i]) * 4.0)))
    # blur once so creases read as soft shadow, not speckle
    return [0.5 * out[i] + 0.5 * (sum(out[j] for j in nb[i]) / len(nb[i]) if nb[i] else 0) for i in range(len(verts))]


def export(name, verts, idx, cols, skin):
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
    b64 = lambda b: base64.b64encode(bytes(b)).decode()
    col = b64(round(max(0.0, min(1.0, ch)) * 255) for c in cols for ch in c)
    j = b64(s[0] for vw in skin for s in vw)
    w = b64(s[1] for vw in skin for s in vw)
    bones = '[' + ', '.join("'" + b + "'" for b in BONE_ORDER) + ']'
    line = (f"  {name}: {{ n: {len(verts)}, t: {len(idx)}, min: {[round(x, 5) for x in mn]}, max: {[round(x, 5) for x in mx]}, "
            f"v: '{b64(pos)}', i: '{b64(ind)}', c: '{col}', j: '{j}', w: '{w}', b: {bones} }},")
    def write():
        src = open(OUT).read()
        pat = re.compile(r'^  ' + name + r': \{.*$', re.M)
        if pat.search(src): src = pat.sub(lambda m: line, src)
        else: src = src.replace('\n};\n', '\n' + line + '\n};\n')
        open(OUT, 'w').write(src)
    C['locked'](OUT, write)
    print(f'wrote {name}: {len(verts)} verts, {len(idx)} tris, {len(BONE_ORDER)} bones -> {OUT}')


# ======================================================================
# The bodies live in assets/src/blender/skin_<type>.py, run in this namespace
# (C, Piece, V, BONES, math, ...). Each defines BODY:
#   name     the part name (e.g. 'smBody')
#   pieces() -> [Piece, ...]
#   paint(p, n, cav) -> (r, g, b) sRGB 0..1; cav is 0 (convex) .. 1 (a crease)
#   body / fine: fuse settings per group: voxel, smooth (iterations, factor),
#                noise, tris (fine is for fingers and toes)
# ======================================================================
NS = globals()
exec(compile(open(os.path.join(HERE, f'skin_{TYPE}.py')).read(), os.path.join(HERE, f'skin_{TYPE}.py'), 'exec'), NS)
spec = BODY
pieces = spec['pieces']()
for pc in pieces: pc.make()
allv, alli, alln = [], [], []
for grp in ('body', 'fine'):
    g = [pc for pc in pieces if pc.group == grp]
    if not g: continue
    o = spec[grp]
    v, nrm, i = fuse(g, f"{spec['name']}_{grp}", o['voxel'], o['smooth'], o['noise'], o['tris'])
    base = len(allv)
    allv += v; alln += nrm; alli += [[a + base for a in f] for f in i]
    print(f'{grp}: {len(v)} verts {len(i)} tris')
cav = concavity(allv, alln, alli)
cols = [spec['paint'](p, nn, c) for p, nn, c in zip(allv, alln, cav)]
skin = weigh(allv, pieces)
export(spec['name'], allv, alli, cols, skin)
