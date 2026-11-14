"""coat_hook_broom_apron_and_rugs (was r13b): things hung on a wall and rugs, remodelled at their real sizes
(docs/design/PROPS.md, P2): the wall coat hook (on its oak block), the corn broom
hung by its handle's hole, the bib apron hung by its neck strap, the area
rug rolled up and tied, and the scatter rug on the floor. Sources and sizes:
docs/research/props-remodelling/coat_hook_broom_apron_and_rugs.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part is in its
own frame: x across its front, y up, z out of its front; a wall thing's
wall is the plane z = 0, a floor thing's floor the plane y = 0. The cast
iron hook (`Real__coatHook`) and the overcoat on it (`Real__coatHung`,
`Real__hookCoatButtons`) are family `wall_hole_pipe_run_posters_and_hooks`'s; the screws `Real__hookScrew`.

hookBlock        an oak block (70 x 130 x 19 mm) the single hook is screwed
                 to, its front edges rounded over, two countersunk screws
scarfHung        a hand-knitted wool scarf thrown over the hook's arm, its
                 halves fallen either side and against the wall, fringed
broomHandle      a round hearth broom's handle (24 mm, 0.46 m), a hardwood
                 stick, its top rounded, the hole across it for the cord;
                 built hanging from the nail
broomNail        the nail it hangs on, driven into the wall a little upward
broomCord        the jute loop through the hole and over the nail, knotted
broomCorn        the broomcorn round the handle's foot: the folded shoulder,
                 its ends splayed, worn short on one side and bent over,
                 straws standing out of the sweep
broomTwine       the twine wound round it in two bands
apronCloth       a child's bib apron (38 x 51 cm, cotton twill) hung from a hook by
                 its neck strap: draped by Blender's cloth solver, the bib's
                 top fallen into a cowl between the strap's ends, the waist
                 corners dropped, the skirt against the wall in folds; its
                 patch pocket, the hems, the neck strap over the hook and
                 the two ties hanging from the waist
rugRoll{A,B,C}Pile    an area rug rolled pile out (8 x 10, 6 x 9, 5 x 8 ft):
                 the outer turn and its cut edge; built standing (its axis
                 +y), the game lays it down (its floor side +x)
rugRoll{A,B,C}Ends    at both ends of the spiral, every turn's overcast side,
                 and the hollow core closed a little way in
rugRoll{A,B,C}Back    the foundation's turns at both ends (cotton, pale)
rugRoll{A,B,C}Fringe  the outer end's fringe, falling to the floor
rugRoll{A,B,C}Twine   jute twine tied round it twice, cinching it
rugFace          a scatter rug (2'5" x 4', a Hamadan): its pile, worn down
                 in a path, its edges overcast, its corners curling
rugBack          its back and its overcast sides
rugFringe        the warp's fringe at both ends, matted, lying on the floor
"""
from propkit import *
import propkit
import random
import zlib
import time


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget, smooth=35.0, kind='box'):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one. Its randomness is seeded from its name (the
    shape is built here, at registration)."""
    random.seed(zlib.crc32(name.encode()))
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), kind), lambda: shape, budget=budget, smooth=smooth)


def rnd(a, b):
    return a + (b - a) * random.random()


def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def grid_surface(fn, nu, nv, wrap_u=False):
    """A surface from fn(u, v) -> Vector over u in [0, 1] (nu steps, wrapped
    if wrap_u) and v in [0, 1] (nv steps): an open grid of quads."""
    cols = nu if wrap_u else nu + 1
    v, f = [], []
    for j in range(nv + 1):
        for i in range(cols):
            v.append(fn(i / nu, j / nv))
    for j in range(nv):
        for i in range(nu):
            i2 = (i + 1) % cols if wrap_u else i + 1
            f.append((j * cols + i, j * cols + i2, (j + 1) * cols + i2, (j + 1) * cols + i))
    return Shape(v, f)


def ribbon(pts, sides, width, thick):
    """A band along the points `pts`, `width` across in the direction
    `sides[i]` at each point (unit vectors), `thick` through: a strap, a tie."""
    v, f = [], []
    n = len(pts)
    for i in range(n):
        p = Vector(pts[i])
        t = (Vector(pts[min(i + 1, n - 1)]) - Vector(pts[max(i - 1, 0)])).normalized()
        s = Vector(sides[i])
        s = (s - t * s.dot(t)).normalized()
        nn = t.cross(s).normalized()
        for a, b in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            v.append(p + s * (a * width / 2) + nn * (b * thick / 2))
    for i in range(n - 1):
        for k in range(4):
            k2 = (k + 1) % 4
            f.append((i * 4 + k, i * 4 + k2, (i + 1) * 4 + k2, (i + 1) * 4 + k))
    f.append((3, 2, 1, 0))
    f.append(((n - 1) * 4, (n - 1) * 4 + 1, (n - 1) * 4 + 2, (n - 1) * 4 + 3))
    return propkit._fix_winding(Shape(v, f))


def strip(pts, sides, widths):
    """A flat strip (one sided; the game draws it from both) along `pts`,
    `widths[i]` across in the direction `sides[i]`."""
    v, f = [], []
    for p, sd, w in zip(pts, sides, widths):
        v.append(Vector(p) - Vector(sd) * w / 2)
        v.append(Vector(p) + Vector(sd) * w / 2)
    for i in range(len(pts) - 1):
        f.append((2 * i, 2 * i + 1, 2 * i + 3, 2 * i + 2))
    return Shape(v, f)


# ================================================================ apron ===
# A child's bib apron (Chef's Closet's child apron, 15 x 20 in: 38 cm across
# the skirt, 51 cm long), the bib 20 cm across its top, the armholes cut in a
# quarter ellipse to the waist 17 cm down; cotton twill, its ties 40 cm.
# Hung from a cast iron hook (r06's coatHook, its plate at the origin, on the
# wall) by its neck strap over the hook's cradle; the strap 20 cm down each
# side to the bib's top corners. In the cloth's flat frame (FX across, FV
# down from the bib's top, 0 to -0.51) the grid is 1.5 cm.
AP_W, AP_L, AP_BIB, AP_WAIST = 0.19, 0.51, 0.10, 0.17
AP_STEP = 0.015
AP_CRADLE = Vector((0.0, -0.047, 0.058))      # where the strap bears on the hook
AP_TOP_Y = -0.229                              # the bib's corners, hung
AP_GATHER = 0.30                               # the bib's top drawn in by the strap
AP_TILT = 0.08                                 # how far the strap has slid round the hook
AP_POCKET = (-0.10, 0.10, -0.27, -0.39)        # x0, x1, v top, v bottom


def apron_half(v):
    """Half the apron's width at FV = v (<= 0)."""
    d = -v
    if d <= 0.0:
        return AP_BIB
    if d >= AP_WAIST:
        return AP_W
    return AP_W - (AP_W - AP_BIB) * math.sqrt(max(0.0, 1.0 - (d / AP_WAIST) ** 2))


def apron_flat():
    """The cloth: rows 2 cm apart down the apron, each row's points 2 cm
    apart across it from edge to edge (so the cloth is even everywhere and
    its edge is the outline), rows of unequal length joined by triangles.
    Returns (flat points [(x, v)], faces, rows [[point id]])."""
    nv = int(round(AP_L / AP_STEP))
    pts, rows, faces = [], [], []
    for j in range(nv + 1):
        v = -j * AP_STEP
        h = apron_half(v)
        n = max(2, int(round(2 * h / AP_STEP))) + 1
        ids = []
        for i in range(n):
            ids.append(len(pts))
            pts.append([-h + 2 * h * i / (n - 1), v])
        rows.append(ids)
    for A, B in zip(rows, rows[1:]):
        if len(A) == len(B):
            for i in range(len(A) - 1):
                faces.append((A[i], A[i + 1], B[i + 1], B[i]))
            continue
        i = k = 0
        a, b = len(A) - 1, len(B) - 1
        while i < a or k < b:
            if k == b or (i < a and (i + 1) / a <= (k + 1) / b):
                faces.append((A[i], A[i + 1], B[k]))
                i += 1
            else:
                faces.append((A[i], B[k + 1], B[k]))
                k += 1
    return pts, faces, rows


def apron_drape():
    """Drape the apron in Blender's cloth solver: its bib's top corners held
    where the strap holds them (drawn together), the rest let fall against
    the wall from a little out from it. Returns (settled points, faces,
    flat points, the flat grid's ids)."""
    import bpy
    pts, faces, rows = apron_flat()
    init = []
    for x, v in pts:
        d = -v
        w = 1.0 - smoothstep(0.0, 0.17, d)
        xi = x * (1.0 - AP_GATHER * w)
        yi = AP_TOP_Y + v
        # the slack the strap leaves at the bib's top falls forward: a cowl
        zi = 0.010 + 0.05 * (d / AP_L) + 0.03 * w * max(0.0, 1.0 - (x / AP_BIB) ** 2)
        zi += 0.003 * noise.noise(Vector((x * 9.0, v * 9.0, 0.37)))
        # the strap has slid on the hook: one corner hangs a little lower
        a = AP_TILT
        xr, yr = xi * math.cos(a) - (yi - AP_TOP_Y) * math.sin(a), xi * math.sin(a) + (yi - AP_TOP_Y) * math.cos(a)
        init.append((xr, AP_TOP_Y + yr, zi))
    me = bpy.data.meshes.new('apron_sim')
    me.from_pydata(init, [], faces)
    me.update()
    ob = bpy.data.objects.new('apron_sim', me)
    bpy.context.scene.collection.objects.link(ob)
    # the strap's ends: the bib's top corners and a hand's width in from them
    pins = []
    for j in (0, 1):
        for k in rows[j]:
            if abs(abs(pts[k][0]) - AP_BIB) < (0.021 if j == 0 else 0.001):
                pins.append(k)
    g = ob.vertex_groups.new(name='pin')
    g.add(pins, 1.0, 'REPLACE')
    # the wall it falls against
    wme = bpy.data.meshes.new('apron_wall')
    wme.from_pydata([(-1.5, -2.0, 0.0), (1.5, -2.0, 0.0), (1.5, 1.0, 0.0), (-1.5, 1.0, 0.0)], [], [(0, 1, 2, 3)])
    wme.update()
    wall = bpy.data.objects.new('apron_wall', wme)
    bpy.context.scene.collection.objects.link(wall)
    wall.modifiers.new('col', 'COLLISION')
    wall.collision.thickness_outer = 0.002
    wall.collision.cloth_friction = 8.0
    sc = bpy.context.scene
    grav = tuple(sc.gravity)
    sc.gravity = (0.0, -9.81, 0.0)
    cl = ob.modifiers.new('cloth', 'CLOTH')
    s = cl.settings
    s.quality = 8
    s.mass = 0.04
    s.tension_stiffness = 15.0
    s.compression_stiffness = 15.0
    s.shear_stiffness = 5.0
    s.bending_stiffness = 0.07
    s.air_damping = 3.0
    s.vertex_group_mass = 'pin'
    c = cl.collision_settings
    c.use_collision = True
    c.distance_min = 0.003
    c.use_self_collision = True
    c.self_distance_min = 0.002
    c.collision_quality = 4
    frames = 200
    sc.frame_start, sc.frame_end = 1, frames
    cl.point_cache.frame_start, cl.point_cache.frame_end = 1, frames
    for fr in range(1, frames + 1):
        sc.frame_set(fr)
    dg = bpy.context.evaluated_depsgraph_get()
    em = ob.evaluated_get(dg).to_mesh()
    out = [Vector(vv.co) for vv in em.vertices]
    ob.evaluated_get(dg).to_mesh_clear()
    sc.frame_set(1)
    sc.gravity = grav
    bpy.data.objects.remove(ob)
    bpy.data.meshes.remove(me)
    bpy.data.objects.remove(wall)
    bpy.data.meshes.remove(wme)
    return out, faces, pts, rows


_AP = None


def apron_sim():
    global _AP
    if _AP is None:
        t0 = time.time()
        _AP = apron_drape()
        print('r13b: the apron draped in %.1f s' % (time.time() - t0))
    return _AP


def _normals(P, faces):
    """Area-weighted vertex normals of a surface, turned out from the wall."""
    N = [Vector((0, 0, 0)) for _ in P]
    for f in faces:
        a, b, c = P[f[0]], P[f[1]], P[f[2]]
        n = (b - a).cross(c - a)
        if len(f) == 4:
            n += (c - a).cross(P[f[3]] - a)
        for i in f:
            N[i] += n
    out = []
    for n in N:
        n = n.normalized() if n.length > 1e-12 else Vector((0, 0, 1))
        out.append(n)
    # the cloth's front faces the room on the whole: turn all one way
    if sum(n.z for n in out) < 0:
        out = [-n for n in out]
    return out


def _boundary_loops(faces):
    """The cloth's edge: its loops of boundary vertices, in order."""
    count = {}
    for f in faces:
        for k in range(len(f)):
            a, b = f[k], f[(k + 1) % len(f)]
            key = (min(a, b), max(a, b))
            count[key] = count.get(key, 0) + 1
    nb = {}
    for (a, b), c in count.items():
        if c == 1:
            nb.setdefault(a, []).append(b)
            nb.setdefault(b, []).append(a)
    loops, seen = [], set()
    for start in sorted(nb):
        if start in seen:
            continue
        loop, prev, cur = [start], None, start
        seen.add(start)
        while True:
            nxt = [x for x in nb[cur] if x != prev and x not in seen]
            if not nxt:
                break
            prev, cur = cur, nxt[0]
            loop.append(cur)
            seen.add(cur)
        loops.append(loop)
    return loops


def _tube_loop(pts, r, seg=3):
    """A thin closed tube through the points (a hem rolled round an edge)."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    v, f = [], []
    for i in range(n):
        t = (pts[(i + 1) % n] - pts[i - 1]).normalized()
        up = Vector((0, 0, 1)) if abs(t.z) < 0.9 else Vector((1, 0, 0))
        a = t.cross(up).normalized()
        b = t.cross(a).normalized()
        for k in range(seg):
            ang = TAU * k / seg
            v.append(pts[i] + (a * math.cos(ang) + b * math.sin(ang)) * r)
    for i in range(n):
        i2 = (i + 1) % n
        for k in range(seg):
            k2 = (k + 1) % seg
            f.append((i * seg + k, i * seg + k2, i2 * seg + k2, i2 * seg + k))
    return Shape(v, f)


def apron_cloth():
    P, faces, flat, rows = apron_sim()
    P = [p.copy() for p in P]
    # a little of the solver's crinkle relaxed (twill is heavier than it)
    E = set()
    for f_ in faces:
        for k in range(len(f_)):
            a, b = f_[k], f_[(k + 1) % len(f_)]
            E.add((min(a, b), max(a, b)))
    nbr = [[] for _ in P]
    for a, b in E:
        nbr[a].append(b)
        nbr[b].append(a)
    edge = set(i for lp in _boundary_loops(faces) for i in lp)
    for _ in range(2):
        Q = [p.copy() for p in P]
        for i, p in enumerate(P):
            if i in edge or not nbr[i]:
                continue
            m = sum((P[j] for j in nbr[i]), Vector((0, 0, 0))) / len(nbr[i])
            Q[i] = p.lerp(m, 0.35)
        P = Q
    for p in P:
        p.z = max(p.z, 0.004)
    N = _normals(P, faces)
    s = Shape([p.copy() for p in P], faces)
    # the hem rolled round every edge
    for lp in _boundary_loops(faces):
        s = s + _tube_loop([P[i] for i in lp], 0.0016)
    # the patch pocket, two compartments, its mouth gaping a little
    x0, x1, vt, vb = AP_POCKET
    prow = [r for r in rows if vb - 1e-6 <= flat[r[0]][1] <= vt + 1e-6]
    grid = [[k for k in r if x0 - 1e-6 <= flat[k][0] <= x1 + 1e-6] for r in prow]
    nr, nc = len(grid), len(grid[0])
    pv = []
    for j, r in enumerate(grid):
        for i, k in enumerate(r):
            u = flat[k][0] / x1
            gape = 0.004 * (1 - u * u) * (1.0 if j == 0 else 0.4 if j == 1 else 0.0)
            pv.append(P[k] + N[k] * (0.0024 + gape))
    pf = [(j * nc + i, j * nc + i + 1, (j + 1) * nc + i + 1, (j + 1) * nc + i) for j in range(nr - 1) for i in range(nc - 1)]
    pocket = Shape(pv, pf)
    rim = [pv[i] for i in range(nc)] + [pv[j * nc + nc - 1] for j in range(1, nr)] + \
          [pv[(nr - 1) * nc + i] for i in range(nc - 2, -1, -1)] + [pv[j * nc] for j in range(nr - 2, 0, -1)]
    pocket = pocket + _tube_loop(rim, 0.0013)
    mid = nc // 2
    pocket = pocket + tube([pv[j * nc + mid] + N[grid[j][mid]] * 0.0006 for j in range(nr)], 0.0007, seg=3)
    s = s + pocket
    # the neck strap: from each of the bib's top corners up over the hook's
    # cradle, its width turning from across the bib to along the hook's arm
    top = rows[0]
    cl, cr = P[top[0]], P[top[-1]]
    arc_c = AP_CRADLE + Vector((0, -0.0065, 0))
    rr = 0.0078
    pts, sides = [], []
    leg = 9
    for side, corner in ((-1, cl), (1, cr)):
        lo = corner + Vector((0, -0.028, 0.0028))
        tan = arc_c + Vector((side * rr * math.cos(math.radians(25)), rr * math.sin(math.radians(25)), 0))
        seq = [lo.lerp(corner + Vector((0, 0, 0.0028)), t) for t in (0.0, 0.5)]
        seq += [(corner + Vector((0, 0, 0.0028))).lerp(tan, t / leg) for t in range(leg + 1)]
        sd = []
        for k in range(len(seq)):
            t = max(0.0, (k - 2) / leg)
            sd.append(Vector((1, 0, 0)).lerp(Vector((0, 0, 1)), smoothstep(0.0, 1.0, t)).normalized())
        if side == -1:
            pts += seq
            sides += sd
            for a in range(1, 8):
                ang = math.radians(25 + 130 * a / 8)
                pts.append(arc_c + Vector((-rr * math.cos(ang), rr * math.sin(ang), 0)))
                sides.append(Vector((0, 0, 1)))
        else:
            pts += list(reversed(seq))
            sides += list(reversed(sd))
    s = s + ribbon(pts, sides, 0.020, 0.0018)
    # the ties: from the waist's corners, hanging down over the skirt's sides
    waist = next(r for r in rows if flat[r[0]][1] <= -AP_WAIST - 1e-6)
    for k, sgn in ((waist[0], -1), (waist[-1], 1)):
        c0 = P[k]
        tp, ts = [], []
        n = 24
        for i in range(n + 1):
            t = i / n
            y = c0.y - 0.02 - 0.38 * t
            x = c0.x - sgn * (0.012 + 0.008 * math.sin(t * 5.0 + sgn) + 0.012 * t)
            near = [q.z for q in P if abs(q.x - x) < 0.03 and abs(q.y - y) < 0.03]
            z = max([0.004] + near) + 0.004
            tp.append(Vector((x, y, z)))
            ts.append(Vector((math.cos(0.5 * math.sin(t * 7.0)), 0, math.sin(0.5 * math.sin(t * 7.0)))))
        # the first stretch turns down from the corner
        tp.insert(0, c0 + Vector((0, 0, 0.003)))
        ts.insert(0, Vector((1, 0, 0)))
        s = s + ribbon(tp, ts, 0.020, 0.0016)
    return s


auto('Real__apronCloth', apron_cloth, 4000, smooth=60)



# ================================================================ broom ===
# A hand-made round hearth broom (Mother Earth News' "Homemade Broom":
# broomcorn bound round a stick with twine, a cord through a hole in the
# handle to hang it by): a 24 mm hardwood handle 0.46 m long, the broomcorn
# round it from a folded shoulder to its splayed ends 0.30 m below, bound
# with twine in two bands; 0.71 m from the nail to its longest straw, no
# wider along the wall than 0.11 m (the old broom's, so it meets the wall's
# face where the old one did). Hung on a nail by a jute loop through the
# handle's hole: the nail's entry into the wall is the origin, the wall the
# plane z = 0; it hangs plumb, its head's back against the wall.
BR_R = 0.0120                     # the handle's radius
BR_Z = 0.0500                     # the handle's axis off the wall
BR_NAIL_UP = math.radians(8)      # the nail driven a little upward
BR_NAIL_R = 0.00165
BR_HOLE_Y = -0.066                # the hole through the handle (across it)
BR_TOP = BR_HOLE_Y + 0.016        # the handle's top end
BR_LEN = 0.46
BR_FOOT = BR_TOP - BR_LEN         # the handle's foot, inside the broomcorn
BR_SH = BR_FOOT + 0.11            # the shoulder's top
BR_TIP = -0.702                   # the longest straws (0.71 m below the nail's head)
BR_HEAD = BR_SH - BR_TIP
BR_HALF = 0.0545                  # no wider than this either side
BR_WEAR = 0.045                   # worn shorter on one side (+x)
BR_BANDS = ((0.016, 0.034), (0.088, 0.100))   # the twine's bands below the shoulder's top


def broom_nail():
    d = Vector((0, math.sin(BR_NAIL_UP), math.cos(BR_NAIL_UP)))
    shank = cyl(BR_NAIL_R, BR_NAIL_R, 0.042, seg=8, bevel_r=0.0003, bevel_seg=1)
    head = cyl(0.0034, 0.0034, 0.0013, seg=12, bevel_r=0.0004, bevel_seg=1).move(0, 0.021 + 0.0006, 0)
    s = shank + head
    s.move(0, 0.021 - 0.006, 0)            # 6 mm of it in the wall
    s.apply(Vector((0, 1, 0)).rotation_difference(d).to_matrix().to_4x4())
    return s


def broom_handle():
    """The handle: a hardwood stick, its top rounded off, the hole across
    it the cord runs through, its foot inside the broomcorn."""
    h = BR_LEN
    prof = [(0.0, 0.0), (BR_R * 0.92, 0.0), (BR_R, 0.002), (BR_R, h - 0.008), (BR_R * 0.9, h - 0.003),
            (BR_R * 0.6, h - 0.0006), (0.0, h)]
    s = lathe(prof, seg=14)
    s = noisy(s, 0.0004, scale=30.0, seed=11)        # a stick, not a dowel
    hole = cyl(0.0035, 0.0035, 0.05, seg=10, bevel_r=0).rot(z=math.pi / 2).move(0, h - 0.016, 0)
    s = cut(s, hole)
    return s.move(0, BR_FOOT, BR_Z)


def broom_cord():
    """The jute loop through the hole, over the nail, its ends knotted."""
    nail = Vector((0.0, 0.0015 + 0.024 * math.sin(BR_NAIL_UP), 0.024 * math.cos(BR_NAIL_UP)))
    top = nail + Vector((0, BR_NAIL_R + 0.0022, 0))
    pts = []
    # down the left from over the nail, through the hole, up the right
    for side in (-1, 1):
        seq = []
        for t in (0.0, 0.25, 0.5, 0.75, 1.0):
            p = top.lerp(Vector((side * (BR_R + 0.002), BR_HOLE_Y + 0.004, BR_Z)), t)
            p.x += side * 0.003 * math.sin(t * math.pi)
            seq.append(p)
        pts.append(seq)
    left, right = pts
    through = [Vector((x, BR_HOLE_Y, BR_Z)) for x in (-BR_R - 0.002, 0.0, BR_R + 0.002)]
    s = tube(list(reversed(left)) + through[1:-1] + right, 0.0021, seg=5, caps=True, smooth=1)
    # the knot on the right strand, and its two ends
    k = right[1]
    s = s + noisy(sphere(0.0045, seg=8, rings=5).move(k.x + 0.001, k.y, k.z + 0.001), 0.0006, scale=300, seed=3)
    for e in (-1, 1):
        s = s + tube([k, k + Vector((0.004 + 0.002 * e, -0.012, 0.003 * e)), k + Vector((0.006 + 0.003 * e, -0.024, 0.004 * e))],
                     0.0018, seg=4, caps=True, r_end=0.0012)
    return s


def _br_radius(s_, a):
    """The broomcorn's radius at s_ below its shoulder's top, angle a round
    it: the folded shoulder, pulled in at the bands, splayed below."""
    keys = [(0.000, 0.012), (0.006, 0.024), (0.014, 0.033), (0.030, 0.036), (0.060, 0.039), (0.094, 0.037),
            (0.140, 0.042), (0.200, 0.047), (0.260, 0.051), (0.330, 0.054)]
    for (p0, r0), (p1, r1) in zip(keys, keys[1:]):
        if p0 <= s_ <= p1:
            t = (s_ - p0) / (p1 - p0)
            t = t * t * (3 - 2 * t)
            r = r0 + (r1 - r0) * t
            break
    else:
        r = keys[-1][1]
    for b0, b1 in BR_BANDS:
        c, w = (b0 + b1) / 2, (b1 - b0) / 2
        r -= 0.0035 * smoothstep(w + 0.004, w, abs(s_ - c))
    # the straw lies in bundles: shallow ridges running down it
    r *= 1.0 + 0.05 * noise.noise(Vector((math.cos(a) * 3.0, math.sin(a) * 3.0, s_ * 2.0))) * smoothstep(0.01, 0.04, s_)
    return r


def _br_point(s_, a):
    r = _br_radius(s_, a)
    free = smoothstep(0.11, BR_HEAD, s_)
    x = r * math.cos(a)
    z = r * math.sin(a) * (1.0 + 0.25 * free)          # splayed out from the wall more than across it
    # bent over to one side from being stood on
    x += 0.012 * free ** 2
    # no wider along the wall than the old broom, its back on the wall
    x = max(-BR_HALF, min(BR_HALF, x))
    return Vector((x, BR_SH - s_, max(BR_Z + z, 0.0025)))


def broom_corn():
    rnd_ = random.Random(4417)
    cols, rows = 30, 26
    v, f = [], []
    # each column of straws ends at its own length: worn short on the +x side
    ends = []
    for i in range(cols):
        a = TAU * i / cols
        u = math.cos(a)
        ends.append(BR_HEAD - BR_WEAR * (u + 1) / 2 - rnd_.uniform(0.0, 0.016) - (0.014 if i % 3 == 1 else 0.0))
    for j in range(rows + 1):
        t = j / rows
        for i in range(cols):
            a = TAU * i / cols
            v.append(_br_point(ends[i] * t, a))
    for j in range(rows):
        for i in range(cols):
            i2 = (i + 1) % cols
            f.append((j * cols + i, j * cols + i2, (j + 1) * cols + i2, (j + 1) * cols + i))
    # the folded top closed round the handle, the ends closed below
    v.append(Vector((0, BR_SH + 0.002, BR_Z)))
    c0 = len(v) - 1
    for i in range(cols):
        f.append((c0, (i + 1) % cols, i))
    base = rows * cols
    v.append(sum((v[base + i] for i in range(cols)), Vector((0, 0, 0))) / cols + Vector((0, 0.045, 0)))
    c1 = len(v) - 1
    for i in range(cols):
        f.append((c1, base + i, base + (i + 1) % cols))
    body = propkit._fix_winding(Shape(v, f))
    body = noisy(body, 0.0008, scale=60.0, seed=3)
    # straws a few centimetres past the clumps they end in, so the sweep's
    # edge is ragged as straw is, splayed and bent, broken ones sticking out
    straws = Shape()
    for k in range(180):
        i = rnd_.randrange(cols)
        a = TAU * (i + rnd_.uniform(-0.5, 0.5)) / cols
        s0 = ends[i] - rnd_.uniform(0.02, 0.07)
        p0 = _br_point(s0, a)
        p0 = Vector((p0.x * 0.95, p0.y, BR_Z + (p0.z - BR_Z) * 0.9))
        L = rnd_.uniform(0.03, 0.09) if k < 160 else rnd_.uniform(0.06, 0.12)
        d = Vector((math.cos(a) * rnd_.uniform(0.05, 0.25) + 0.05, -1.0, math.sin(a) * rnd_.uniform(0.0, 0.3))).normalized()
        pts = [p0 + d * L * t + Vector((rnd_.uniform(-0.003, 0.003), 0, 0)) * t for t in (0.0, 0.5, 1.0)]
        for q in pts:
            q.x = max(-BR_HALF, min(BR_HALF, q.x))
            q.z = max(q.z, 0.003)
            q.y = max(q.y, BR_TIP + 0.0005)
        straws = straws + tube(pts, rnd_.uniform(0.0006, 0.0009), seg=3, caps=False, r_end=0.0004)
    return body + straws


def broom_twine():
    """The twine wound round the broomcorn: two bands of close turns."""
    s = Shape()
    for b0, b1 in BR_BANDS:
        n = max(2, int(round((b1 - b0) / 0.0032)))
        for k in range(n):
            s_ = b0 + (b1 - b0) * (k + 0.5) / n
            pts = [_br_point(s_, TAU * i / 24) for i in range(24)]
            pts = [Vector((p.x * 1.02, p.y, BR_Z + (p.z - BR_Z) * 1.02)) for p in pts]
            s = s + _tube_loop(pts, 0.0016, seg=4)
    return s


auto('Real__broomNail', broom_nail, 200)
auto('Real__broomHandle', broom_handle, 500, kind='cyl')
auto('Real__broomCord', broom_cord, 600)
auto('Real__broomCorn', broom_corn, 4400, smooth=50)
auto('Real__broomTwine', broom_twine, 2200)


# ============================================================ hook block ===
# The single hook's oak block: 70 x 130 x 19 mm, its front edges rounded
# over, screwed to the wall above and below the hook's plate (r06's
# hookScrew in the two countersinks). Its back on the wall (z = 0).
HB = (0.070, 0.130, 0.019)


def hook_block():
    w, h, d = HB
    s = prism(rounded_rect(w, h, 0.008, seg=3), d, bevel_r=0.0035, bevel_seg=3).move(0, 0, d / 2)
    sinks = [cone(0.0060, 0.0055, seg=12).rot(x=-math.pi / 2).move(0, sy * 0.050, d + 0.0006) for sy in (-1, 1)]
    return cut(s, *sinks)


auto('Real__hookBlock', hook_block, 600)


# ================================================================== scarf ===
# A hand-knitted wool scarf (13 cm wide, ribbed, its ends fringed) left on a
# coat hook: thrown over the hook's arm (r06's coatHook, its plate's back at
# z = 0, the cradle 47 mm below its middle and 58 mm out), bunched where it
# bends over the arm, its two halves falling either side of it and turning
# to lie against the wall, one over the other, one end longer. Its frame is
# the hook's.
SC_ARM = Vector((0.0, -0.0465, 0.058))         # the top of the arm at its cradle
SC_W, SC_T = 0.13, 0.0045                      # width (as it hangs), thickness
SC_HALVES = ((-1, 0.245, 0.0085), (1, 0.172, 0.0))   # (side, length, how far in front)


def _sc_frame(side, L, front, s_):
    """The scarf's centre, its width's direction and its width at s_ m down
    one half from the top of the arm."""
    turn = smoothstep(0.0, 0.085, s_)                        # width along the arm -> along the wall
    y = SC_ARM.y - s_ - 0.004 * math.sin(min(s_ / 0.012, 1.0) * math.pi / 2)
    x = side * (0.0075 + 0.008 * smoothstep(0.0, 0.05, s_)) - side * 0.011 * smoothstep(0.06, 0.18, s_)
    zb = 0.004 + front                                      # down against the block / wall
    z = SC_ARM.z + (zb - SC_ARM.z) * smoothstep(0.0, 0.07, s_)
    # below the block (70 x 130 mm) it falls back to the wall's face
    z -= 0.016 * smoothstep(0.06, 0.12, s_) * (1.0 if front == 0.0 else 0.9)
    w = 0.045 + (SC_W - 0.045) * smoothstep(0.005, 0.11, s_)
    a = turn * math.pi / 2
    dirw = Vector((math.sin(a) * side, 0.0, math.cos(a)))
    return Vector((x, y, z)), dirw, w


def scarf_hung():
    s = Shape()
    rnd_ = random.Random(5150)
    for side, L, front in SC_HALVES:
        nu, nv = 7, 26
        ph = rnd_.uniform(0, 6)
        surf = []
        for j in range(nv + 1):
            s_ = L * (j / nv) ** 1.15
            c, dw, w = _sc_frame(side, L, front, s_)
            nxt, _, _ = _sc_frame(side, L, front, min(L, s_ + 0.005))
            prv, _, _ = _sc_frame(side, L, front, max(0.0, s_ - 0.005))
            tg = (nxt - prv).normalized()
            dw = (dw - tg * dw.dot(tg)).normalized()
            nrm = tg.cross(dw).normalized()
            if nrm.z < 0:
                nrm = -nrm
            row = []
            for i in range(nu + 1):
                u = i / nu - 0.5
                # the knit's lengthwise folds, deeper as it hangs; bunched at the arm
                fold = 0.0035 * math.sin(u * TAU * 1.4 + ph + s_ * 9.0) * smoothstep(0.02, 0.12, s_)
                fold += 0.006 * math.sin(u * TAU * 2.5 + ph) * (1.0 - smoothstep(0.0, 0.05, s_))
                p = c + dw * (u * w) + nrm * fold
                row.append((p, nrm))
            surf.append(row)
        # a slab: front and back faces, its edges closed
        v, f = [], []
        cols = nu + 1
        for row in surf:
            for p, n in row:
                v.append(p + n * SC_T / 2)
        off = len(v)
        for row in surf:
            for p, n in row:
                v.append(p - n * SC_T / 2)
        for j in range(nv):
            for i in range(nu):
                a, b = j * cols + i, j * cols + i + 1
                c_, d = (j + 1) * cols + i + 1, (j + 1) * cols + i
                f.append((a, b, c_, d))
                f.append((off + d, off + c_, off + b, off + a))
        for j in range(nv):
            for i in (0, nu):
                a, d = j * cols + i, (j + 1) * cols + i
                f.append((a, d, off + d, off + a))
        for j in (0, nv):
            for i in range(nu):
                a, b = j * cols + i, j * cols + i + 1
                f.append((a, b, off + b, off + a))
        half = propkit._fix_winding(Shape(v, f))
        # the fringe at its end: tassels of yarn, a few tangled
        end = surf[-1]
        for k in range(8):
            u = (k + 0.5) / 8
            p0, n0 = end[min(nu, int(u * nu))]
            p0 = end[0][0].lerp(end[-1][0], u)
            ln = rnd_.uniform(0.022, 0.034)
            dx = rnd_.uniform(-0.004, 0.004)
            pts = [p0, p0 + Vector((dx * 0.5, -ln * 0.5, 0.0005)), p0 + Vector((dx, -ln, 0.001))]
            sides = [Vector((1, 0, 0))] * 3
            half = half + strip(pts, sides, [0.006, 0.005, 0.003])
        s = s + half
    return noisy(s, 0.0006, scale=80.0, seed=7)


auto('Real__scarfHung', scarf_hung, 2000, smooth=60)


# ============================================================ rolled rug ===
# An area rug rolled up pile out and tied with jute twine, lying on the
# floor: 8 x 10 ft rolled across its 8 ft (a roll 2.44 m long, 24 cm
# through), 6 x 9 ft (1.83 m, 18 cm) and 5 x 8 ft (1.52 m, 14 cm); the rug
# about 12 mm thick with its pile, rolled on nothing (a hollow core). Built
# standing, its axis along +y (centred), its floor side +x with the floor at
# x = 0, the room +z; the game lays it down (a turn of -pi/2 about z).
# Its outer end (the fringed end) stops low on the room side, the fringe
# falling to the floor.
ROLLS = {'A': (2.44, 0.120, 0.0125, 0.022), 'B': (1.83, 0.090, 0.0105, 0.018), 'C': (1.52, 0.070, 0.0085, 0.015)}
RR_EDGE = math.radians(52)          # where the outer end stops (0 the floor, 90 the room side)
RR_OVAL = (0.95, 1.03)              # squashed by its own weight: (down, across)
RR_SEG = 40                         # round it
RR_TIES = (-0.30, 0.30)             # where it is tied (fractions of its length)


def _rr_r(key, psi):
    """The outer surface's radius of the layer psi radians back from the
    outer end (0 the outer end, growing inward)."""
    L, R, t, r0 = ROLLS[key]
    return R - t * psi / TAU


def _rr_pt(key, r, phi, y):
    """A point at radius r and angle phi (0 toward the floor, pi/2 the room)
    at height y up the axis, squashed and set on the floor (x = 0)."""
    L, R, t, r0 = ROLLS[key]
    x = r * math.cos(phi) * RR_OVAL[0]
    z = r * math.sin(phi) * RR_OVAL[1]
    rb = _rr_r(key, TAU - RR_EDGE) * RR_OVAL[0]         # the turn it rests on
    xmax = rb - 0.004                                   # the flat it is pressed into
    if x > xmax:
        x = xmax + (x - xmax) * 0.15
    # nothing rolled by hand is straight: a little bowed along its length
    u = 2 * y / L
    z += 0.012 * (1 - u * u) - 0.004 * math.sin(u * 2.6)
    return Vector((x - (xmax + 0.0006), y, z))


def _rr_cinch(key, y):
    L, R, t, r0 = ROLLS[key]
    return sum(0.0032 * math.exp(-((y - f * L) / 0.007) ** 2) for f in RR_TIES)


def _rr_rows(key):
    L, R, t, r0 = ROLLS[key]
    ys = [(-0.5 + i / 14) * L for i in range(15)]
    for f in RR_TIES:
        ys += [f * L + d for d in (-0.018, -0.009, -0.004, 0.0, 0.004, 0.009, 0.018)]
    ys = sorted(set(round(y, 5) for y in ys))
    return ys


def roll_pile(key):
    """The outer turn (pile out), its cut edge, and the pile's turns at
    both ends of the spiral."""
    L, R, t, r0 = ROLLS[key]
    ys = _rr_rows(key)
    n = RR_SEG
    v, f = [], []
    # the outer turn: psi from 0 (the outer end) to 2 pi (where it lies on
    # the turn below it), the angle phi = edge + psi
    for j, y in enumerate(ys):
        for i in range(n + 1):
            psi = TAU * i / n
            r = _rr_r(key, psi) - _rr_cinch(key, y)
            # the outer end's last centimetres lie down onto the turn below
            r -= t * 0.35 * smoothstep(0.10, 0.0, psi)
            v.append(_rr_pt(key, r, RR_EDGE + psi, y))
    cols = n + 1
    for j in range(len(ys) - 1):
        for i in range(n):
            f.append((j * cols + i, j * cols + i + 1, (j + 1) * cols + i + 1, (j + 1) * cols + i))
    s = Shape(v, f)
    # the cut edge: from the outer end's face down to the turn below
    ev = []
    for y in ys:
        a = RR_EDGE
        ev.append(_rr_pt(key, _rr_r(key, 0) - t * 0.35 - _rr_cinch(key, y), a, y))
        ev.append(_rr_pt(key, _rr_r(key, 0) - t * 1.0 - _rr_cinch(key, y), a, y))
    ef = [(2 * j, 2 * j + 1, 2 * j + 3, 2 * j + 2) for j in range(len(ys) - 1)]
    s = s + Shape(ev, ef)
    return propkit._fix_winding(s)


def roll_ends(key):
    """At both ends of the roll, the side of the rug in every turn: its
    overcast edge (the pile's part of each turn), and the hollow core
    closed a little way in."""
    L, R, t, r0 = ROLLS[key]
    s = _rr_ends(key, 0.0, 0.72)
    turns = (R - t - r0) / t
    rin = _rr_r(key, TAU * int(turns * 22) / 22) - t
    for y in (-L / 2 + 0.003, L / 2 - 0.003):
        d = Shape([_rr_pt(key, 0.0, 0.0, y)] + [_rr_pt(key, rin, RR_EDGE + TAU * i / 20, y) for i in range(20)],
                  [(0, i + 1, (i + 1) % 20 + 1) for i in range(20)])
        s = s + d
    return propkit._fix_winding(s)


def _rr_ends(key, a0, a1):
    """The band from a0 to a1 of the rug's thickness (0 its face, 1 its
    back) round the spiral, at both ends of the roll."""
    L, R, t, r0 = ROLLS[key]
    turns = (R - t - r0) / t
    per = 22
    m = int(turns * per)
    out = Shape()
    for y, sgn in ((-L / 2, -1), (L / 2, 1)):
        v, f = [], []
        for i in range(m + 1):
            psi = TAU * i / per
            r = _rr_r(key, psi)
            for a in (a0, a1):
                v.append(_rr_pt(key, r - t * a, RR_EDGE + psi, y))
        for i in range(m):
            f.append((2 * i, 2 * i + 2, 2 * i + 3, 2 * i + 1))
        out = out + Shape(v, f)
    return out


def roll_back(key):
    """At both ends, the back of every turn: its foundation, paler."""
    return propkit._fix_winding(_rr_ends(key, 0.72, 1.0))


def roll_fringe(key):
    """The outer end's fringe: the warp's ends, knotted in bundles every
    16 mm, falling over the edge down the roll's front toward the floor;
    matted, some bundles splayed in two, some lying crossed, some short."""
    L, R, t, r0 = ROLLS[key]
    rnd_ = random.Random(zlib.crc32(('fringe' + key).encode()))
    s = Shape()
    n = int(L / 0.016)
    for k in range(n):
        y0 = (-0.5 + (k + 0.5) / n) * L + rnd_.uniform(-0.003, 0.003)
        if abs(y0) > L / 2 - 0.006:
            continue
        split = rnd_.random() < 0.35
        for b in ((-1, 1) if split else (0,)):
            length = 0.062 * rnd_.uniform(0.55, 1.15)
            lean = rnd_.gauss(0.0, 0.18) + b * rnd_.uniform(0.15, 0.4)
            curl = rnd_.uniform(-0.4, 0.4)
            pts, sides, ws = [], [], []
            m = 5
            w0 = rnd_.uniform(0.005, 0.008) * (0.6 if split else 1.0)
            for i in range(m + 1):
                u = i / m
                arc = length * u
                phi = RR_EDGE - arc * math.cos(lean) / (R - t * 0.5)
                lift = 0.004 * math.sin(u * math.pi) * rnd_.uniform(0.2, 1.0)
                r = _rr_r(key, 0) - t * 0.8 + 0.0016 + lift
                y = y0 + b * 0.002 + arc * math.sin(lean) + curl * arc * u * 0.5
                pts.append(_rr_pt(key, r, phi, y))
                ws.append(w0 * (1.0 - 0.55 * u))
            for i in range(m + 1):
                tg = (pts[min(i + 1, m)] - pts[max(i - 1, 0)]).normalized()
                c = _rr_pt(key, 0.0, 0.0, pts[i].y)
                nrm = (pts[i] - c)
                nrm.y = 0.0
                nrm.normalize()
                sides.append(tg.cross(nrm).normalized())
            s = s + strip(pts, sides, ws)
    return s


def roll_twine(key):
    """Jute twine round it twice at each tie, pulled into the pile, knotted
    on the room side, its two ends hanging."""
    L, R, t, r0 = ROLLS[key]
    rnd_ = random.Random(zlib.crc32(('twine' + key).encode()))
    s = Shape()
    for fi, f in enumerate(RR_TIES):
        y0 = f * L
        for k, dy in enumerate((-0.0032, 0.0032)):
            pts = []
            n = 44
            for i in range(n):
                phi = TAU * i / n
                psi = (phi - RR_EDGE) % TAU
                r = _rr_r(key, psi) - 0.0032 + 0.0019
                # it bridges the step at the outer end
                if psi > TAU - 0.5:
                    r = max(r, _rr_r(key, 0) - 0.0032 + 0.0019 - t * (TAU - psi) / 0.5)
                pts.append(_rr_pt(key, r, phi, y0 + dy + 0.0012 * math.sin(phi * 3 + k)))
            s = s + _tube_loop(pts, 0.0020, seg=3)
        # the knot and its ends, on the room side a little up
        kp = _rr_pt(key, _rr_r(key, 0.9) + 0.004, 1.65, y0)
        s = s + noisy(sphere(0.0055, seg=8, rings=5).scale(1.2, 0.9, 1.0).move(*kp), 0.0008, scale=200, seed=fi)
        for e in (-1, 1):
            tail = []
            for i in range(5):
                u = i / 4
                phi = 1.65 - u * 0.55
                r = _rr_r(key, 0.9) + 0.003 + 0.006 * u
                tail.append(_rr_pt(key, r, phi, y0 + e * (0.004 + 0.02 * u) + rnd_.uniform(-0.003, 0.003)))
            s = s + tube(tail, 0.0018, seg=3, caps=True, r_end=0.0014)
    return s


for _k in 'ABC':
    auto(f'Real__rugRoll{_k}Pile', lambda k=_k: roll_pile(k), 3600, smooth=50, kind='cyl')
    auto(f'Real__rugRoll{_k}Ends', lambda k=_k: roll_ends(k), 1200, kind='cyl')
    auto(f'Real__rugRoll{_k}Back', lambda k=_k: roll_back(k), 1200, kind='cyl')
    auto(f'Real__rugRoll{_k}Fringe', lambda k=_k: roll_fringe(k), 2600, smooth=80)
    auto(f'Real__rugRoll{_k}Twine', lambda k=_k: roll_twine(k), 1800)


# ============================================================== floor rug ===
# A Hamadan scatter rug, 2'5" x 4' (0.74 x 1.22 m) without its fringe: wool
# pile on a cotton foundation, about 6 mm thick now that the pile is worn
# low, a flat-woven band 2 cm at each end and the warp's fringe past it; the
# long sides overcast. Lying on the floor: x along its length, z across it,
# the floor y = 0. Old and walked on: the pile worn down in a path across
# it, a ripple where it has crept, two corners curling up.
RUG_L, RUG_W, RUG_T = 1.22, 0.74, 0.0060
RUG_KILIM = 0.020
RUG_NX, RUG_NZ = 44, 26


def _rug_height(x, z):
    """The pile's top at (x, z) over the body, before the curls."""
    # the path worn across it, a little off its middle and on the slant
    d = (x * 0.42 + z * 0.91) - 0.05
    worn = math.exp(-(d / 0.16) ** 2) * (0.6 + 0.4 * math.sin(x * 4.0 + 1.0))
    h = RUG_T - 0.0018 * worn
    # the pile's own unevenness, and a ripple where the rug has crept
    h += 0.0004 * noise.noise(Vector((x * 14.0, z * 14.0, 0.5)))
    r = math.exp(-((x - 0.42 + 0.10 * z) / 0.07) ** 2) * smoothstep(-0.37, -0.1, z) * smoothstep(0.37, 0.15, z)
    return h + 0.007 * r


def _rug_curl(p):
    """The corners lift: one well up, one a little, the others flat."""
    out = p.copy()
    for cx, cz, lift, reach in ((RUG_L / 2 + RUG_KILIM, -RUG_W / 2, 0.014, 0.16), (-RUG_L / 2 - RUG_KILIM, RUG_W / 2, 0.006, 0.12)):
        dx, dz = abs(p.x - cx), abs(p.z - cz)
        d = (dx + dz) / reach
        if d < 1.0:
            out.y += lift * (1 - d) ** 2
    return out


def rug_face():
    """The pile's top over the body and the flat-woven bands, its long
    edges rolled down over the overcasting."""
    xs = [-RUG_L / 2 - RUG_KILIM] + [-RUG_L / 2 + RUG_L * i / RUG_NX for i in range(RUG_NX + 1)] + [RUG_L / 2 + RUG_KILIM]
    zs = [-RUG_W / 2 + RUG_W * j / RUG_NZ for j in range(RUG_NZ + 1)]
    v, f = [], []
    for j, z in enumerate(zs):
        for i, x in enumerate(xs):
            if i == 0 or i == len(xs) - 1:
                h = 0.0028
            else:
                h = _rug_height(x, z)
                if i == 1 or i == len(xs) - 2:
                    h = (h + 0.0028) / 2
            # the overcast sides: rounded down to the floor's edge
            e = min(z + RUG_W / 2, RUG_W / 2 - z)
            if e < 1e-6:
                h *= 0.45
            v.append(_rug_curl(Vector((x, h, z))))
    cols = len(xs)
    for j in range(len(zs) - 1):
        for i in range(cols - 1):
            f.append((j * cols + i, (j + 1) * cols + i, (j + 1) * cols + i + 1, j * cols + i + 1))
    return Shape(v, f)


def rug_back():
    """Its back (the foundation, the knots seen through it) and the
    overcast sides and the ends' edges, closing it."""
    xs = [-RUG_L / 2 - RUG_KILIM] + [-RUG_L / 2 + RUG_L * i / RUG_NX for i in range(RUG_NX + 1)] + [RUG_L / 2 + RUG_KILIM]
    zs = [-RUG_W / 2 + RUG_W * j / RUG_NZ for j in range(RUG_NZ + 1)]
    cols = len(xs)
    v, f = [], []
    for z in zs:
        for x in xs:
            v.append(_rug_curl(Vector((x, 0.0004, z))))
    for j in range(len(zs) - 1):
        for i in range(cols - 1):
            f.append((j * cols + i, j * cols + i + 1, (j + 1) * cols + i + 1, (j + 1) * cols + i))
    s = Shape(v, f)
    # the edges round it, from the back up to the face's edge
    face = rug_face()
    ring = [(0, i) for i in range(cols)] + [(j, cols - 1) for j in range(1, len(zs))] + \
           [(len(zs) - 1, i) for i in range(cols - 2, -1, -1)] + [(j, 0) for j in range(len(zs) - 2, 0, -1)]
    ev, ef = [], []
    for (j, i) in ring:
        ev.append(v[j * cols + i].copy())
        ev.append(face.v[j * cols + i].copy())
    n = len(ring)
    for k in range(n):
        k2 = (k + 1) % n
        ef.append((2 * k, 2 * k2, 2 * k2 + 1, 2 * k + 1))
    s = s + Shape(ev, ef)
    # the overcasting: wool wrapped round the long sides, a cord along each
    for sz in (-1, 1):
        pts = [_rug_curl(Vector((x, 0.0032, sz * (RUG_W / 2 + 0.0008)))) for x in xs[1:-1:2]]
        s = s + tube(pts, 0.0032, seg=5, caps=True)
    return propkit._fix_winding(s)


def rug_fringe():
    """The warp's fringe at both ends, 4-6 cm, knotted in bundles every
    16 mm: matted, splayed, some turned back over the band, some crossed."""
    rnd_ = random.Random(77012)
    s = Shape()
    n = int(RUG_W / 0.016)
    for sx in (-1, 1):
        x0 = sx * (RUG_L / 2 + RUG_KILIM - 0.002)
        for k in range(n):
            z0 = -RUG_W / 2 + (k + 0.5) * RUG_W / n + rnd_.uniform(-0.002, 0.002)
            split = rnd_.random() < 0.4
            back = rnd_.random() < 0.06
            for b in ((-1, 1) if split else (0,)):
                length = rnd_.uniform(0.035, 0.060)
                ang = rnd_.gauss(0.0, 0.16) + b * rnd_.uniform(0.12, 0.35)
                curl = rnd_.uniform(-0.6, 0.6)
                m = 4
                w0 = rnd_.uniform(0.005, 0.008) * (0.65 if split else 1.0)
                pts, ws = [], []
                for i in range(m + 1):
                    u = i / m
                    a = ang + curl * u * u * 0.6
                    if back and u > 0.4:
                        a += math.pi * smoothstep(0.4, 1.0, u)
                    x = x0 + sx * length * u * math.cos(ang) * (1.0 if not back else 1 - u)
                    z = z0 + length * u * math.sin(a) + b * 0.0015
                    y = 0.0010 + 0.0016 * math.sin(u * math.pi) * (1 if i else 0) + (0.003 * u if back else 0.0)
                    pts.append(_rug_curl(Vector((x, y if i else 0.0022, z))))
                    ws.append(w0 * (1 - 0.5 * u))
                sides = []
                for i in range(m + 1):
                    tg = (pts[min(i + 1, m)] - pts[max(i - 1, 0)]).normalized()
                    sides.append(tg.cross(Vector((0, 1, 0))).normalized())
                s = s + strip(pts, sides, ws)
    return s


auto('Real__rugFace', rug_face, 2600, smooth=60)
auto('Real__rugBack', rug_back, 3400, smooth=40)
auto('Real__rugFringe', rug_fringe, 1400, smooth=80)
