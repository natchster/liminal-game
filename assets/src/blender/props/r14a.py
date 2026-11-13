"""r14a: things worn and dropped, remodelled at their real sizes
(docs/design/PROPS.md): a glove lying limp, a shoe and a boot (one last, one
construction), and a shoe box. Sources and sizes: r14a.notes.md.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Every part is in its own
frame: x across its front, y up from the floor (y = 0 the floor it rests
on), z out of its front (a glove's fingers, a shoe's toe, a box's long
side).

glove           (gloveBack, gloveUnder, gloveCuff, gloveTips) one glove, a
                right one lying back up, empty and limp: the hand a flat
                pillow of two layers sewn round its outline, the fingers
                fanned a little, two curling up off the floor, folds across
                the back and the fingers' joints; the wrist (cuff) its own
                part; the fingertip caps, the knuckle strap and the thumb of
                a leather palm work glove a skin over the back
shoe            (shoe<Part>L, shoe<Part>Side; parts Upper, Lining, Sole,
                Heel, Tread, Laces, Eyelets) a men's derby (UK 9, 300 mm),
                welted: the upper lofted between its topline and its
                featherline, the quarters lapped over the vamp, the backstay,
                creases across the vamp where the foot bent it; the tongue;
                the lining and insole; a leather sole on a stacked heel, the
                welt; five pairs of eyelets, the lace undone. L is the left
                one standing; Side the right one fallen on its outer side, as
                it comes to rest (its tread printed as it lies)
boot            (boot<Part>, boot<Part>Side; Upper, Lining, Sole, Heel,
                Tread, Laces, Hooks) a 6 in lace-up work boot on the same
                construction: the shaft, the padded collar, the pull loop, a
                lug sole, four pairs of eyelets and three of speed hooks, the
                lace undone from the hooks; standing, and fallen on its side
shoeBox         (shoeBoxBase, shoeBoxLid, shoeBoxLabel, shoeBoxLidPrint,
                shoeBoxTissue) a two-piece chipboard box for men's shoes
                (340 x 120 x 200 mm), its walls doubled, a corner of the lid
                crushed in, the base's end label, the lid's print, tissue
                paper crumpled in it
"""
from propkit import *
import propkit
from mathutils.geometry import delaunay_2d_cdt


# ---------------------------------------------------------------- parts ---
# how a part's normals are finished, where not the default
SHADE = {'Real__shoeBoxTissue': dict(flat=True)}


def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget, **SHADE.get(name, {}))


# ------------------------------------------------------------------ util --
def smoothstep(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def ridge(d, w, h):
    return h * math.exp(-(d / w) ** 2)


def chaikin(pts, n=2, closed=True):
    p = [Vector(q) for q in pts]
    for _ in range(n):
        q = []
        m = len(p) if closed else len(p) - 1
        if not closed:
            q.append(p[0])
        for i in range(m):
            a, b = p[i], p[(i + 1) % len(p)]
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        if not closed:
            q.append(p[-1])
        p = q
    return p


def arclen(p, closed):
    s = [0.0]
    m = len(p) if closed else len(p) - 1
    for i in range(m):
        s.append(s[-1] + (p[(i + 1) % len(p)] - p[i]).length)
    return s


def resample(pts, n, closed=True):
    """n points evenly spaced along a polyline (closed: n round the loop,
    starting at its first point)."""
    p = [Vector(q) for q in pts]
    s = arclen(p, closed)
    L = s[-1]
    out = []
    j = 0
    cnt = n if closed else n - 1
    for i in range(n):
        t = L * i / cnt
        while j < len(s) - 2 and s[j + 1] < t:
            j += 1
        a, b = p[j % len(p)], p[(j + 1) % len(p)]
        k = (t - s[j]) / max(1e-12, s[j + 1] - s[j])
        out.append(a.lerp(b, min(1.0, max(0.0, k))))
    return out


def simplify(pts, tol):
    """Douglas-Peucker on a closed loop (2D Vectors): keeps the corners."""
    def dp(seg):
        if len(seg) < 3:
            return seg
        a, b = seg[0], seg[-1]
        ab = b - a
        L = ab.length
        best, bi = -1.0, 0
        for i in range(1, len(seg) - 1):
            if L < 1e-12:
                d = (seg[i] - a).length
            else:
                d = abs(ab.x * (seg[i].y - a.y) - ab.y * (seg[i].x - a.x)) / L
            if d > best:
                best, bi = d, i
        if best > tol:
            return dp(seg[:bi + 1])[:-1] + dp(seg[bi:])
        return [a, b]
    # split the loop at its two farthest points
    i0 = 0
    i1 = max(range(len(pts)), key=lambda i: (pts[i] - pts[0]).length)
    a = dp(pts[i0:i1 + 1])
    b = dp(pts[i1:] + [pts[0]])
    return a[:-1] + b[:-1]


def max_seg(pts, step):
    """Split any edge of the loop longer than step."""
    out = []
    for i in range(len(pts)):
        a, b = pts[i], pts[(i + 1) % len(pts)]
        n = max(1, int(math.ceil((b - a).length / step)))
        for k in range(n):
            out.append(a.lerp(b, k / n))
    return out


def seg_hit(p, r, a, b):
    """Intersection parameters of segment p->r with a->b, or None."""
    d1, d2 = r - p, b - a
    den = d1.x * d2.y - d1.y * d2.x
    if abs(den) < 1e-14:
        return None
    w = a - p
    t = (w.x * d2.y - w.y * d2.x) / den
    u = (w.x * d1.y - w.y * d1.x) / den
    if 0.0 <= t <= 1.0 and 0.0 <= u < 1.0:
        return t, u
    return None


def cut_lines(outline, lines, inside=None):
    """Insert where each line (a polyline of 2D points) crosses the outline
    into the outline, and return (outline, constraint segments inside it).
    `inside(x, z)` tells which pieces of a line lie inside (else they are
    taken in pairs of crossings)."""
    outline = list(outline)
    segs = []
    for line in lines:
        line = [Vector(q) for q in line]
        pts = []
        for p, r in zip(line, line[1:]):
            hits = []
            i = 0
            while i < len(outline):
                a, b = outline[i], outline[(i + 1) % len(outline)]
                h = seg_hit(p, r, a, b)
                if h:
                    t, u = h
                    q = a.lerp(b, u)
                    if u > 1e-6:
                        outline.insert(i + 1, q)
                        i += 1
                    hits.append((t, q))
                i += 1
            hits.sort(key=lambda x: x[0])
            pts += [(p, False)] + [(q, True) for t, q in hits]
        pts.append((line[-1], False))
        if inside is None:
            cr = [q for q, hit in pts if hit]
            for k in range(0, len(cr) - 1, 2):
                segs.append((cr[k], cr[k + 1]))
        else:
            for (a, _), (b, _) in zip(pts, pts[1:]):
                m = (a + b) / 2
                if (b - a).length > 1e-7 and inside(m.x, m.y):
                    segs.append((a, b))
    return outline, segs


def cdt(outline, interior, segs):
    """Triangles filling the outline (2D Vectors, its points kept), with
    interior points and constraint segments. Returns (verts, tris)."""
    verts = [Vector(p) for p in outline]
    face = list(range(len(outline)))
    if sum(a.x * b.y - b.x * a.y for a, b in zip(verts, verts[1:] + verts[:1])) < 0:
        face.reverse()
    edges = []
    for a, b in segs:
        verts += [Vector(a), Vector(b)]
        edges.append((len(verts) - 2, len(verts) - 1))
    verts += [Vector(p) for p in interior]
    r = delaunay_2d_cdt(verts, edges, [face], 1, 1e-7, True)
    return [Vector(v) for v in r[0]], [tuple(f) for f in r[2]]


def lift_tris(v2, tris, yfun, up=True):
    """3D shape from 2D (x, z) triangles, each vertex at y = yfun(x, z),
    faces turned to face up (or down)."""
    v = [Vector((p.x, yfun(p.x, p.y), p.y)) for p in v2]
    f = []
    for a, b, c in tris:
        n = (v[b] - v[a]).cross(v[c] - v[a])
        f.append((a, b, c) if (n.y > 0) == up else (a, c, b))
    return Shape(v, f)


def cut_tris(sh, keep):
    """The faces of sh whose centroid passes keep(centroid), reindexed."""
    idx, v, f = {}, [], []
    for fc in sh.f:
        c = sum((sh.v[i] for i in fc), Vector()) / len(fc)
        if not keep(c):
            continue
        g = []
        for i in fc:
            if i not in idx:
                idx[i] = len(v)
                v.append(sh.v[i].copy())
            g.append(idx[i])
        f.append(tuple(g))
    return Shape(v, f)


def offset_along_normals(sh, d):
    """Push every vertex along its averaged face normal by d."""
    acc = [Vector() for _ in sh.v]
    for fc in sh.f:
        a, b, c = sh.v[fc[0]], sh.v[fc[1]], sh.v[fc[2]]
        n = (b - a).cross(c - a)
        for i in fc:
            acc[i] += n
    out = sh.copy()
    for i, n in enumerate(acc):
        if n.length > 1e-12:
            out.v[i] = sh.v[i] + n.normalized() * d
    return out


def grid_points(lo, hi, step, inside, jitter=0.3, seed=1):
    rng = random.Random(seed)
    pts = []
    nx = int((hi[0] - lo[0]) / step) + 1
    nz = int((hi[1] - lo[1]) / step) + 1
    for j in range(nz):
        for i in range(nx):
            x = lo[0] + (i + 0.5 * (j % 2)) * step + rng.uniform(-jitter, jitter) * step
            z = lo[1] + j * step * 0.87 + rng.uniform(-jitter, jitter) * step
            if inside(x, z):
                pts.append(Vector((x, z)))
    return pts


def marching_contour(sdf, lo, hi, h):
    """The zero contour of a 2D field (one closed loop, the longest), by
    marching squares on an h grid."""
    nx = int((hi[0] - lo[0]) / h) + 2
    nz = int((hi[1] - lo[1]) / h) + 2
    val = [[sdf(lo[0] + i * h, lo[1] + j * h) for i in range(nx)] for j in range(nz)]
    # edge points keyed by the grid edge they lie on
    def ep(i0, j0, i1, j1):
        a, b = val[j0][i0], val[j1][i1]
        t = a / (a - b)
        return Vector((lo[0] + (i0 + (i1 - i0) * t) * h, lo[1] + (j0 + (j1 - j0) * t) * h))
    nb = {}
    pos = {}
    for j in range(nz - 1):
        for i in range(nx - 1):
            # the cell's edges in order round it: bottom, right, top, left
            cs = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)]
            ins = [val[q[1]][q[0]] < 0 for q in cs]
            cross = []
            for e in range(4):
                a, b = cs[e], cs[(e + 1) % 4]
                if ins[e] != ins[(e + 1) % 4]:
                    key = (min(a, b), max(a, b))
                    if key not in pos:
                        pos[key] = ep(key[0][0], key[0][1], key[1][0], key[1][1])
                    cross.append((e, key))
            if len(cross) == 2:
                pairs = [(cross[0][1], cross[1][1])]
            elif len(cross) == 4:
                # a saddle: keep the inside corners apart
                if ins[0]:
                    pairs = [(cross[3][1], cross[0][1]), (cross[1][1], cross[2][1])]
                else:
                    pairs = [(cross[0][1], cross[1][1]), (cross[2][1], cross[3][1])]
            else:
                pairs = []
            for a, b in pairs:
                nb.setdefault(a, []).append(b)
                nb.setdefault(b, []).append(a)
    # walk the loops
    seen = set()
    loops = []
    for k0 in nb:
        if k0 in seen:
            continue
        loop = [k0]
        seen.add(k0)
        prev, cur = None, k0
        while True:
            nxt = [k for k in nb[cur] if k != prev and k not in seen]
            if not nxt:
                break
            prev, cur = cur, nxt[0]
            seen.add(cur)
            loop.append(cur)
        loops.append([pos[k] for k in loop])
    return max(loops, key=len)


def sd_capsule_path(p, path, r0, r1=None):
    """Distance from p to a polyline swept by a radius running r0 -> r1,
    and where along it (0..1)."""
    r1 = r0 if r1 is None else r1
    s = arclen(path, False)
    L = s[-1]
    best, bt = 1e9, 0.0
    for i in range(len(path) - 1):
        a, b = path[i], path[i + 1]
        ab = b - a
        k = max(0.0, min(1.0, (p - a).dot(ab) / max(1e-12, ab.length_squared)))
        d = (p - (a + ab * k)).length
        t = (s[i] + k * ab.length) / L
        dd = d - (r0 + (r1 - r0) * t)
        if dd < best:
            best, bt = dd, t
    return best, bt


def sd_round_poly(p, poly, r):
    """Signed distance to a convex polygon (ccw) grown by r."""
    inside = True
    dmin = 1e9
    for i in range(len(poly)):
        a, b = poly[i], poly[(i + 1) % len(poly)]
        ab = b - a
        k = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
        dmin = min(dmin, (p - (a + ab * k)).length)
        if ab.x * (p.y - a.y) - ab.y * (p.x - a.x) < 0:
            inside = False
    return (-dmin if inside else dmin) - r


def smin(a, b, k):
    h = max(k - abs(a - b), 0.0) / k
    return min(a, b) - h * h * k * 0.25


# ----------------------------------------------------------------- glove --
# One glove, a right one lying back up (the palm on the floor), its fingers
# toward +z, the thumb toward +x; the wrist seam at z = 0 and the cuff behind
# it. Its size is a men's L work glove's (PIP's Gunn cut: 260 x 133 mm with
# the thumb out) and a knitted or a lined leather one's (21-24 x 10-11 cm):
# 245 mm from the cuff's mouth to the middle fingertip, the hand 102 mm
# across flat (r14a.notes.md).
G_WRIST, G_CUFF = 0.0, -0.062                 # the wrist seam, the cuff's mouth
G_KNUCK = 0.092                               # the knuckles (the fingers' crotches a little past)
# fingers: (base x, base z, length, flat width, fan angle from +z toward +x, curl)
G_FINGERS = {
    'index': (0.0335, G_KNUCK + 0.002, 0.079, 0.0232, 0.13, 0.0),
    'middle': (0.0110, G_KNUCK + 0.006, 0.087, 0.0238, 0.03, 0.010),
    'ring': (-0.0120, G_KNUCK + 0.004, 0.080, 0.0228, -0.08, 0.007),
    'little': (-0.0335, G_KNUCK - 0.004, 0.064, 0.0205, -0.23, 0.0),
}
G_THUMB = ((0.045, 0.030), 0.068, 0.0265, 0.72)   # base, length, width, angle from +z toward +x
G_TIP_T = 0.70                                    # where a leather fingertip cap begins (t along a finger)
G_STRAP = (0.064, 0.086)                          # the knuckle strap across the back (z)
G_EDGE = 0.0045                                   # the rounding of the sewn rim
G_RIM = 0.0022                                    # the rim's height off the floor


def glove_paths():
    """Each digit's axis as a polyline from its base: the fingers bowed a
    little, as an empty glove lies."""
    out = {}
    for k, (x, z, L, w, ang, curl) in G_FINGERS.items():
        pts = []
        bow = {'index': 0.006, 'middle': -0.003, 'ring': -0.007, 'little': -0.010}[k]
        for i in range(7):
            t = i / 6
            a = ang + bow / L * 2 * t
            pts.append(Vector((x + math.sin(a) * L * t, z + math.cos(a) * L * t)))
        # its base starts back in the palm so it joins the hand
        b = pts[0] - (pts[1] - pts[0]).normalized() * 0.012
        out[k] = [b] + pts
    (bx, bz), L, w, ang = G_THUMB
    pts = []
    for i in range(7):
        t = i / 6
        a = ang - 0.25 * t
        pts.append(Vector((bx + math.sin(a) * L * t, bz + math.cos(a) * L * t)))
    out['thumb'] = [Vector((bx - 0.022, bz - 0.012))] + pts
    return out


G_PATHS = glove_paths()
G_PALM = [Vector(p) for p in [(-0.039, G_WRIST - 0.004), (0.037, G_WRIST - 0.004), (0.045, 0.040), (0.041, G_KNUCK - 0.006),
                              (-0.039, G_KNUCK - 0.012), (-0.046, 0.038)]]
G_CUFF_POLY = [Vector(p) for p in [(-0.040, G_CUFF), (0.039, G_CUFF), (0.037, G_WRIST + 0.006), (-0.038, G_WRIST + 0.006)]]
G_THUMB_T = 0.16                                  # where a work glove's leather thumb begins


def glove_width(k, t):
    """A digit's flat half width at t along it: tapering to a round tip."""
    if k == 'thumb':
        w = G_THUMB[2]
    else:
        w = G_FINGERS[k][3]
    return w / 2 * (1.0 - 0.10 * t)


def glove_field(x, z):
    """(signed distance to the glove's outline, which digit is nearest and
    how far along it, the palm's own distance)."""
    p = Vector((x, z))
    d = sd_round_poly(p, G_PALM, 0.010)
    d = smin(d, sd_round_poly(p, G_CUFF_POLY, 0.004), 0.008)
    palm = d
    best = (1e9, None, 0.0)
    for k, path in G_PATHS.items():
        L = arclen(path, False)[-1]
        dd, t = sd_capsule_path(p, path, glove_width(k, 0.0), glove_width(k, 1.0))
        # t from the true base (the first point is inside the palm)
        t0 = (path[1] - path[0]).length / L
        tt = max(0.0, (t - t0) / (1 - t0))
        if dd < best[0]:
            best = (dd, k, tt)
        d = smin(d, dd, 0.004 if k != 'thumb' else 0.012)
    return d, best[1], best[2], palm


def glove_sdf(x, z):
    return glove_field(x, z)[0]


def _folds():
    """The slack of an empty glove gathered in long soft folds across its
    back: (a point, a direction, width, height)."""
    rng = random.Random(14)
    out = [((0.040, 0.050), 0.95, 0.0065, 0.0034),      # from the thumb's crotch toward the little finger
           ((-0.010, 0.020), -0.55, 0.0055, 0.0020),
           ((0.000, 0.004), 0.05, 0.0045, -0.0016)]     # the wrist drawn in
    for _ in range(5):
        out.append(((rng.uniform(-0.035, 0.035), rng.uniform(0.01, 0.08)), rng.uniform(-1.2, 1.2),
                    rng.uniform(0.003, 0.006), rng.uniform(-0.0012, 0.0018)))
    return out


G_FOLDS = _folds()


def glove_shape(x, z):
    """(how far the top layer stands above the rim when fully inside, the
    width of the rounding at the rim)."""
    d, k, t, palm = glove_field(x, z)
    # the back of the hand
    T, E = 0.0050, 0.0095
    for (px, pz), ang, w, h in G_FOLDS:
        a = (x - px) * math.cos(ang) - (z - pz) * math.sin(ang)
        along = (x - px) * math.sin(ang) + (z - pz) * math.cos(ang)
        T += ridge(a, w, h) * math.exp(-(along / 0.05) ** 2)
    T += 0.0011 * noise.noise(Vector((x * 45.0, z * 45.0, 3.7)))
    if z < G_WRIST + 0.004:
        # the knitted wrist: thicker, its mouth a little open
        k_c = smoothstep(G_WRIST + 0.004, G_WRIST - 0.006, z)
        Tc = 0.0055 + 0.0030 * smoothstep(G_CUFF + 0.03, G_CUFF, z) + 0.0007 * math.sin(x * 260.0 + 1.3)
        T, E = T + (Tc - T) * k_c, E + (0.008 - E) * k_c
    if k is not None:
        # a digit: a round flattened tube, its joints creased
        L = G_FINGERS[k][2] if k != 'thumb' else G_THUMB[1]
        hw = glove_width(k, t)
        Tf = 0.0050 - 0.0014 * t
        for tj in ((0.30, 0.62) if k != 'thumb' else (0.45,)):
            Tf -= ridge(t - tj, 0.03, 0.0013)
        Tf += 0.0006 * math.sin(t * L * 380.0 + len(k))
        Tf += 0.0006 * noise.noise(Vector((x * 70.0, z * 70.0, 9.1)))
        Ef = 0.85 * hw
        b = smoothstep(-0.008, 0.001, palm)
        T, E = T + (Tf - T) * b, E + (Ef - E) * b
    return max(0.0010, T), E


def glove_lift(x, z):
    """The middle and ring fingers kept a little of the hand's curl: their
    tips lift off the floor."""
    d, k, t, palm = glove_field(x, z)
    if k in ('middle', 'ring') and palm > 0.0:
        curl = G_FINGERS[k][5]
        return curl * max(0.0, t - 0.45) ** 2 / 0.3025
    return 0.0


def glove_k(x, z, E):
    d = -glove_sdf(x, z)
    return math.sqrt(max(0.0, min(1.0, d / E)))


def glove_ytop(x, z):
    T, E = glove_shape(x, z)
    return G_RIM + T * glove_k(x, z, E) + glove_lift(x, z)


def glove_ybot(x, z):
    T, E = glove_shape(x, z)
    return G_RIM * (1.0 - glove_k(x, z, min(E, 0.006))) + glove_lift(x, z)


def inward(loop, d):
    """Points d inside the outline (along the field's gradient), apart."""
    out = []
    h = 0.0004
    for p in loop:
        gx = glove_sdf(p.x + h, p.y) - glove_sdf(p.x - h, p.y)
        gz = glove_sdf(p.x, p.y + h) - glove_sdf(p.x, p.y - h)
        g = Vector((gx, gz))
        if g.length < 1e-12:
            continue
        q = p - g.normalized() * d
        if abs(glove_sdf(q.x, q.y) + d) < d * 0.35 and all((q - o).length > d * 1.2 for o in out[-6:]):
            out.append(q)
    return out


def digit_line(k, t, grow=0.004):
    path = G_PATHS[k]
    s_all = arclen(path, False)
    b = (path[1] - path[0]).length
    s = b + (s_all[-1] - b) * t
    for i in range(len(path) - 1):
        if s_all[i + 1] >= s:
            q = path[i].lerp(path[i + 1], (s - s_all[i]) / (s_all[i + 1] - s_all[i]))
            dvec = (path[i + 1] - path[i]).normalized()
            break
    n = Vector((-dvec.y, dvec.x))
    w = glove_width(k, t) + grow
    return (q - n * w, q + n * w)


def glove_mesh():
    lo, hi = (-0.08, G_CUFF - 0.01), (0.12, 0.20)
    loop = marching_contour(glove_sdf, lo, hi, 0.0011)
    loop = simplify(loop, 0.00025)
    loop = max_seg(loop, 0.007)
    # the lines the parts meet along: the wrist seam, the fingertip caps,
    # the leather thumb, the knuckle strap
    lines = [((-0.1, G_WRIST), (0.1, G_WRIST))]
    for k in G_FINGERS:
        lines.append(digit_line(k, G_TIP_T))
    lines.append(G_THUMB_SEAM)
    lines.append(((-0.1, G_STRAP[0]), (0.058, G_STRAP[0])))
    lines.append(((-0.1, G_STRAP[1]), (0.058, G_STRAP[1])))
    loop, segs = cut_lines(loop, lines, inside=lambda x, z: glove_sdf(x, z) < 0)
    ring = inward(loop, 0.0018)
    inner = lambda x, z: glove_sdf(x, z) < -0.0042
    top_pts = ring + grid_points(lo, hi, 0.0074, inner, seed=3)
    bot_pts = grid_points(lo, hi, 0.0150, inner, seed=4)
    v2t, tt = cdt(loop, top_pts, segs)
    v2b, tb = cdt(loop, bot_pts, segs)
    top = lift_tris(v2t, tt, glove_ytop, up=True)
    bot = lift_tris(v2b, tb, glove_ybot, up=False)
    return top, bot


# the seam round a leather thumb (a keystone thumb): from the crotch past
# the thumb's base to its side by the wrist
G_THUMB_SEAM = [(0.0565, 0.0835), (0.0525, 0.0700), (0.0500, 0.0520), (0.0560, 0.0330), (0.0700, 0.0150)]


def thumb_side(c):
    """How far out past the thumb's seam a point lies (+: the thumb's side)."""
    p = Vector((c.x, c.z))
    best, sg = 1e9, 1.0
    seam = [Vector(q) for q in G_THUMB_SEAM]
    for a, b in zip(seam, seam[1:]):
        ab = b - a
        k = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
        d = (p - (a + ab * k)).length
        if d < best:
            best = d
            # the seam runs down toward the wrist: the thumb lies to its left
            sg = 1.0 if ab.x * (p.y - a.y) - ab.y * (p.x - a.x) > 0 else -1.0
    return best * sg


def glove_region(c):
    """Which part a point of the top lies in: 'cuff', 'tips', or 'back'."""
    if c.z < G_WRIST:
        return 'cuff'
    d, k, t, palm = glove_field(c.x, c.z)
    if thumb_side(c) > 0.0 and c.x > 0.045:
        return 'tips'
    if k is not None and k != 'thumb' and palm > 0.0 and t > G_TIP_T:
        return 'tips'
    if G_STRAP[0] < c.z < G_STRAP[1]:
        return 'tips'
    return 'back'


GLOVE_TOP, GLOVE_BOT = glove_mesh()


def glove_back():
    return cut_tris(GLOVE_TOP, lambda c: c.z >= G_WRIST)


def glove_tips():
    s = cut_tris(GLOVE_TOP, lambda c: glove_region(c) == 'tips')
    return offset_along_normals(s, 0.0004)


def glove_under():
    return cut_tris(GLOVE_BOT, lambda c: c.z >= G_WRIST)


def glove_cuff():
    return cut_tris(GLOVE_TOP, lambda c: c.z < G_WRIST) + cut_tris(GLOVE_BOT, lambda c: c.z < G_WRIST)


auto('Real__gloveBack', glove_back, 1300)
auto('Real__gloveUnder', glove_under, 500)
auto('Real__gloveCuff', glove_cuff, 500)
auto('Real__gloveTips', glove_tips, 600)


# --------------------------------------------------------------- shoe box --
# A two-piece box for a pair of men's shoes, of chipboard (solid paperboard,
# its sides doubled): the base 340 x 120 x 200 mm (13.4 x 4.7 x 7.9 in, in
# the 13-14 x 7.5-8.5 x 4.5-5.5 in of men's shoe boxes; Uline's two-piece
# chipboard men's box is 14 x 8 x 5 in), the lid over it with a 40 mm skirt.
# Frame: x along its length, y up, z across (its front the long side at +z);
# the end label on the base's +x end.
SB_W, SB_H, SB_D = 0.340, 0.120, 0.200
SB_T, SB_TB = 0.0022, 0.0016          # the doubled walls, the bottom
SB_CLR = 0.0008                       # the lid's clearance over the base
SB_LW = SB_W + 2 * (SB_T + SB_CLR)    # the lid, outside
SB_LD = SB_D + 2 * (SB_T + SB_CLR)
SB_LH = 0.040                         # its skirt
SB_LID_Y = SB_H - (SB_LH - SB_T)      # where the lid's frame sits when it is on
SB_CRUSH = (-1, 1)                    # the corner trodden on: -x, +z


def rrect_loop(w, d, r, nx, nz, cseg=3):
    """A rounded rectangle (x, z) loop, ccw seen from above, starting at the
    middle of its +z side: nx points along each x side, nz along each z
    side, cseg+1 round each corner. Two loops made with the same counts
    correspond point for point."""
    hx, hz = w / 2 - r, d / 2 - r
    pts = []
    corners = [(hx, hz, 0.0), (hx, -hz, -math.pi / 2), (-hx, -hz, math.pi), (-hx, hz, math.pi / 2)]
    # +z side from the middle toward +x, then the +x+z corner, the +x side...
    sides = [((0.0, d / 2), (hx, d / 2), nx // 2), ((w / 2, hz), (w / 2, -hz), nz), ((hx, -d / 2), (-hx, -d / 2), nx),
             ((-w / 2, -hz), (-w / 2, hz), nz), ((-hx, d / 2), (0.0, d / 2), nx - nx // 2)]
    for si, (a, b, n) in enumerate(sides):
        for i in range(n):
            t = i / n
            pts.append(Vector((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)))
        if si < 4:
            cx, cz, a0 = corners[si]
            for j in range(cseg + 1):
                ang = a0 + math.pi / 2 - (math.pi / 2) * j / cseg
                pts.append(Vector((cx + r * math.cos(ang), cz + r * math.sin(ang))))
    return pts


def loft_rows(rows):
    """Quads between consecutive rows of points (each a closed loop of one
    count): the outside when the rows go up a ccw loop."""
    v, f = [], []
    n = len(rows[0])
    for row in rows:
        v += [Vector(p) for p in row]
    for k in range(len(rows) - 1):
        for i in range(n):
            j = (i + 1) % n
            a, b, c, d = k * n + i, k * n + j, (k + 1) * n + j, (k + 1) * n + i
            f.append((a, b, c, d))
    return Shape(v, f)


def cap(loop, y, step, up, seed):
    inside_pts = []
    lo = (min(p.x for p in loop), min(p.y for p in loop))
    hi = (max(p.x for p in loop), max(p.y for p in loop))
    for p in grid_points(lo, hi, step, lambda x, z: True, jitter=0.0, seed=seed):
        if lo[0] + step * 0.5 < p.x < hi[0] - step * 0.5 and lo[1] + step * 0.5 < p.y < hi[1] - step * 0.5:
            inside_pts.append(p)
    v2, tris = cdt(loop, inside_pts, [])
    return lift_tris(v2, tris, lambda x, z: y, up=up)


def tray(w, d, h, t, tb, step=0.030, rows=(0.0, 0.0025, 0.035, 0.07)):
    """An open box of board t thick (its bottom tb), standing, its rim at h:
    the outside, the rolled rim of the doubled wall, the inside, the floor
    and the bottom."""
    r = 0.0026
    nx = max(2, int(round((w - 2 * r) / step)))
    nz = max(2, int(round((d - 2 * r) / step)))
    out = rrect_loop(w, d, r, nx, nz)
    inn = rrect_loop(w - 2 * t, d - 2 * t, max(0.0006, r - t), nx, nz)
    ys = [y for y in rows if y < h - 0.004] + [h - 0.0012]
    R = [[Vector((p.x, y, p.y)) for p in out] for y in ys]
    # the rim: over the top of the doubled wall
    R.append([Vector(((p.x + q.x) / 2, h, (p.y + q.y) / 2)) for p, q in zip(out, inn)])
    yi = [h - 0.0012] + [y for y in reversed(rows) if tb + 0.004 < y < h - 0.004] + [tb]
    R += [[Vector((q.x, y, q.y)) for q in inn] for y in yi]
    walls = loft_rows(R)
    bottom = cap(out, 0.0, step * 1.2, False, 21)
    floor = cap(inn, tb, step * 1.2, True, 22)
    s = weld(walls + bottom + floor, 1e-6)
    return propkit._fix_winding(s)


def sb_crush(p, corner, top_y, amp, R):
    """A corner trodden in: pushed down and in toward the middle, most at
    the corner, the board folded along a crease across it."""
    cx, cz = corner
    sx, sz = math.copysign(1, cx), math.copysign(1, cz)
    q = Vector((p.x - cx, (p.y - top_y) * 1.6, p.z - cz))
    dist = q.length
    if dist >= R:
        return p
    f = (1.0 - dist / R) ** 2
    # the crease: where the corner folded down, a line across it
    a = (cx - p.x) * sx + (cz - p.z) * sz
    fold = ridge(a - R * 0.38, R * 0.08, 1.0)
    dvec = Vector((-sx * 0.55, -1.0, -sz * 0.55)).normalized()
    out = Vector((sx, 0.0, sz)).normalized()
    return p + dvec * amp * f + out * amp * 0.35 * fold * f * smoothstep(top_y - 0.03, top_y - 0.005, p.y)


def sb_dent(p, at, n, amp, R):
    """A knock in a wall: pressed in along -n round the point at."""
    d = (p - Vector(at)).length
    if d >= R:
        return p
    return p - Vector(n) * amp * (0.5 + 0.5 * math.cos(math.pi * d / R))


def sb_base_shape(p):
    # the long walls bowed out a little at the middle, warped by the damp
    if abs(p.z) > SB_D / 2 - 0.03:
        k = smoothstep(SB_D / 2 - 0.03, SB_D / 2 - 0.002, abs(p.z))
        b = 0.0022 * (1 - (2 * p.x / SB_W) ** 2) * math.sin(math.pi * min(1.0, p.y / SB_H)) * k
        p = p + Vector((0, 0, math.copysign(b, p.z)))
    p = sb_crush(p, (SB_CRUSH[0] * SB_W / 2, SB_CRUSH[1] * SB_D / 2), SB_H, 0.008, 0.06)
    p = sb_dent(p, (0.06, 0.05, -SB_D / 2), (0, 0, -1), 0.004, 0.035)
    nz = noise3(p * 18.0 + Vector((3.1, 0.7, 5.5))) * 0.0006
    nz.y *= smoothstep(0.0, 0.01, p.y)
    return p + nz


def sb_lid_shape(p):
    """The lid in its own frame (y = 0 its skirt's foot): its top panel
    sagged in the middle, its skirt splayed a little, a corner trodden in."""
    if p.y > SB_LH - 0.006:
        s = 0.0050 * max(0.0, 1 - (2 * p.x / SB_LW) ** 2) * max(0.0, 1 - (2 * p.z / SB_LD) ** 2)
        p = p - Vector((0, s, 0))
    sp = 0.0016 * (1 - min(1.0, p.y / (SB_LH - 0.005)))
    p = p + Vector((math.copysign(sp, p.x) * smoothstep(SB_LW / 2 - 0.01, SB_LW / 2, abs(p.x)), 0,
                    math.copysign(sp, p.z) * smoothstep(SB_LD / 2 - 0.01, SB_LD / 2, abs(p.z))))
    y0 = p.y
    p = sb_crush(p, (SB_CRUSH[0] * SB_LW / 2, SB_CRUSH[1] * SB_LD / 2), SB_LH, 0.024, 0.085)
    nz = noise3(p * 18.0 + Vector((7.1, 2.7, 1.5))) * 0.0006
    nz.y *= smoothstep(0.0, 0.01, p.y)
    p = p + nz
    # the skirt's foot stays where it stood: the corner buckles above it
    p.y = max(p.y, min(y0, 0.0005))
    return p


def sb_base():
    return tray(SB_W, SB_D, SB_H, SB_T, SB_TB, rows=(0.0, 0.0025, 0.03, 0.06, 0.09)).warp(sb_base_shape)


def sb_lid():
    # a tray turned over: its floor the top panel
    s = tray(SB_LW, SB_LD, SB_LH, SB_T, SB_T, rows=(0.0, 0.0025, 0.02))
    s.rot(math.pi, 0, 0).move(0, SB_LH, 0)
    return s.warp(sb_lid_shape)


def flat_grid(w, h, nx, ny, fn):
    """A grid over w x h (u across, v up) mapped through fn(u, v) -> Vector."""
    v, f = [], []
    for j in range(ny + 1):
        for i in range(nx + 1):
            v.append(fn(-w / 2 + w * i / nx, -h / 2 + h * j / ny))
    for j in range(ny):
        for i in range(nx):
            a = j * (nx + 1) + i
            f.append((a, a + 1, a + nx + 2, a + nx + 1))
    return Shape(v, f)


def sb_label():
    """The end label on the base's +x end: 90 x 50 mm, below the lid's
    skirt, facing +x (u across it toward -z, v up)."""
    x = SB_W / 2 + 0.00035
    return flat_grid(0.090, 0.050, 3, 2, lambda u, v: sb_base_shape(Vector((x, 0.042 + v, -u))) + Vector((0.00035, 0, 0)))


def sb_lid_print():
    """The print on the lid: 220 x 130 mm in the middle of its top."""
    return flat_grid(0.220, 0.130, 8, 5, lambda u, v: sb_lid_shape(Vector((u, SB_LH, -v))) + Vector((0, 0.0003, 0)))


def sb_tissue():
    """Tissue paper left in the box, crumpled, lying on its floor and up
    against one end."""
    def fn(u, v):
        x, z = u, v
        y = 0.004 + 0.016 * (0.5 + 0.5 * noise.noise(Vector((x * 9.0, z * 9.0, 1.3))))
        y += 0.020 * smoothstep(0.06, 0.15, x)                 # bunched toward the +x end
        y += ridge(x - 0.02 - 0.3 * z, 0.012, 0.012)            # a fold across it
        y += ridge(z + 0.03 - 0.2 * x, 0.010, 0.008)
        y += 0.006 * noise.noise(Vector((x * 40.0, z * 40.0, 7.7)))
        y += 0.0025 * noise.noise(Vector((x * 110.0, z * 110.0, 2.2)))
        x += 0.004 * noise.noise(Vector((x * 30.0, z * 30.0, 4.4)))
        return Vector((x, max(SB_TB + 0.001, y), z))
    top = flat_grid(0.300, 0.170, 20, 12, lambda u, v: fn(u, -v))
    bot = top.copy()
    bot.v = [p - Vector((0, 0.0008, 0)) for p in bot.v]
    bot.f = [tuple(reversed(f)) for f in bot.f]
    s = top + bot
    # stitch the two layers round the edge
    nx, ny = 20, 12
    n = len(top.v)
    rim = [i for i in range(nx + 1)] + [j * (nx + 1) + nx for j in range(1, ny + 1)] + \
          [ny * (nx + 1) + i for i in range(nx - 1, -1, -1)] + [j * (nx + 1) for j in range(ny - 1, 0, -1)]
    for a, b in zip(rim, rim[1:] + rim[:1]):
        s.f.append((b, a, a + n, b + n))
    return propkit._fix_winding(s)


auto('Real__shoeBoxBase', sb_base, 1500)
auto('Real__shoeBoxLid', sb_lid, 1300)
auto('Real__shoeBoxLabel', sb_label, 100)
auto('Real__shoeBoxLidPrint', sb_lid_print, 200)
auto('Real__shoeBoxTissue', sb_tissue, 1100)


# --------------------------------------------------------------- footwear --
# A shoe and a boot made on one construction (r14a.notes.md): the sole's
# outline (the outsole's edge, the welt round it) and above it the upper,
# lofted between its topline (the edge round the foot's opening and down
# the lacing) and its featherline (where it is lasted onto the insole, the
# welt's inside edge). The loft's columns run from the one to the other,
# and the panels are runs of them: the backstay down the back of the heel,
# the quarters lapped over the vamp, the vamp fanning from the throat to the
# toe. Both are right ones: x across (+x the big toe's side, -x the outer),
# y up, z toward the toe; the left ones are their mirror images.

def catmull(table, x):
    """A smooth curve through [(x, y), ...] (sorted by x), held at its ends."""
    if x <= table[0][0]:
        return table[0][1]
    if x >= table[-1][0]:
        return table[-1][1]
    i = 0
    while table[i + 1][0] < x:
        i += 1
    p0 = table[max(i - 1, 0)][1]
    p1, p2 = table[i][1], table[i + 1][1]
    p3 = table[min(i + 2, len(table) - 1)][1]
    t = (x - table[i][0]) / (table[i + 1][0] - table[i][0])
    return 0.5 * (2 * p1 + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t + (-p0 + 3 * p1 - 3 * p2 + p3) * t ** 3)


# A men's derby, UK 9 (300 mm outsole: a 10.5 measured 12 3/8 in heel to
# toe, 4 3/4 in across the bottom), welted, a 27 mm stacked heel; the topline
# 72 mm high at the back, five pairs of eyelets.
DERBY = dict(
    plan=[(0.000, -0.150), (-0.020, -0.146), (-0.032, -0.135), (-0.037, -0.115), (-0.038, -0.090), (-0.037, -0.060),
          (-0.040, -0.030), (-0.045, 0.000), (-0.049, 0.030), (-0.050, 0.055), (-0.047, 0.085), (-0.040, 0.110),
          (-0.028, 0.132), (-0.012, 0.146), (0.004, 0.151), (0.020, 0.146), (0.034, 0.134), (0.044, 0.112),
          (0.051, 0.085), (0.053, 0.055), (0.050, 0.025), (0.039, -0.005), (0.029, -0.035), (0.029, -0.065),
          (0.034, -0.095), (0.034, -0.118), (0.029, -0.135), (0.017, -0.146)],
    welt=(0.0045, 0.0015),            # the welt's width at the forepart, at the heel
    ytop=[(-0.150, 0.027), (-0.085, 0.027), (-0.045, 0.023), (-0.005, 0.0155), (0.025, 0.0105), (0.090, 0.0100),
          (0.120, 0.0125), (0.152, 0.0215)],
    thick=[(-0.150, 0.006), (-0.080, 0.006), (0.000, 0.0085), (0.030, 0.0095), (0.152, 0.0095)],
    breast=-0.083,
    # the topline from the back up the outer (lateral) side to the lacing's
    # foot; the inner side is the same a little higher
    top_lat=[(0.0, 0.072, -0.143), (-0.017, 0.069, -0.137), (-0.029, 0.064, -0.120), (-0.034, 0.060, -0.097),
             (-0.034, 0.062, -0.074), (-0.030, 0.068, -0.056), (-0.022, 0.081, -0.042), (-0.017, 0.084, -0.022),
             (-0.0135, 0.0815, -0.002), (-0.0105, 0.077, 0.016), (-0.0075, 0.0725, 0.031)],
    med_dy=0.004,
    facing=(-0.044, 0.031),           # the lacing runs between these z
    throat=(0.0, 0.0705, 0.0385),
    seam=(-0.006, -0.016),            # where the quarters' front edges meet the featherline (z), outer and inner
    back_w=0.009,                     # the backstay's half width
    cols=(2, 13, 20),                 # columns: backstay, each quarter, the vamp
    rows=13,
    bulge=(0.05, 0.10, 0.17),
    eyelets=5,
    hooks=0,
    rim=0.0028,
    pad=0.0,
    tongue=dict(w=0.046, th=0.0026, ext=0.016, up=0.003),
)

# A 6 in lace-up work boot (the shaft 6 in from the sole; 310 mm outsole, a
# 34 mm lug sole and heel); four pairs of eyelets and three of speed hooks,
# a padded collar.
BOOT = dict(
    plan=[(x * 1.07, z * 1.035) for x, z in DERBY['plan']],
    welt=(0.0060, 0.0030),
    ytop=[(-0.156, 0.036), (-0.088, 0.036), (-0.047, 0.032), (-0.005, 0.025), (0.026, 0.0205), (0.093, 0.0200),
          (0.124, 0.0225), (0.157, 0.0300)],
    thick=[(-0.156, 0.012), (-0.085, 0.012), (0.000, 0.019), (0.030, 0.020), (0.157, 0.020)],
    breast=-0.084,
    top_lat=[(0.0, 0.180, -0.138), (-0.026, 0.177, -0.128), (-0.041, 0.173, -0.106), (-0.045, 0.172, -0.082),
             (-0.040, 0.175, -0.058), (-0.026, 0.179, -0.042), (-0.017, 0.180, -0.036), (-0.0165, 0.160, -0.030),
             (-0.016, 0.138, -0.022), (-0.0155, 0.118, -0.010), (-0.0140, 0.101, 0.006), (-0.0115, 0.088, 0.024),
             (-0.0085, 0.080, 0.037)],
    med_dy=0.002,
    facing=(-0.038, 0.037),
    throat=(0.0, 0.0775, 0.0445),
    seam=(0.004, -0.004),
    back_w=0.010,
    cols=(2, 15, 18),
    rows=15,
    bulge=(0.03, 0.08, 0.15),
    eyelets=4,
    hooks=3,
    rim=0.0035,
    pad=0.0080,
    tongue=dict(w=0.056, th=0.0060, ext=0.020, up=0.006),
)


def fw_outline(spec, n):
    """The sole's outline (the outsole's edge), n points, from the heel's
    back round the outer side to the toe and back up the inner side."""
    pts = chaikin([Vector(p) for p in spec['plan']], 3)
    i0 = min(range(len(pts)), key=lambda i: pts[i].y)
    pts = pts[i0:] + pts[:i0]
    return resample(pts, n)


def fw_inward(loop, i):
    a, b = loop[i - 1], loop[(i + 1) % len(loop)]
    t = (b - a).normalized()
    return Vector((t.y, -t.x))         # the loop runs clockwise: the inside on its right


def fw_feather(spec):
    """The featherline: the outline inset by the welt, at the sole's top."""
    O = fw_outline(spec, 480)
    wf, wh = spec['welt']
    out = []
    for i, p in enumerate(O):
        w = wh + (wf - wh) * smoothstep(-0.11, -0.06, p.y)
        q = p + fw_inward(O, i) * w
        out.append(Vector((q.x, catmull(spec['ytop'], q.y) + 0.0004, q.y)))
    return out


def piece(pts, a, b):
    """The run of a closed loop from index a to index b (going forward)."""
    n = len(pts)
    out = []
    i = a
    while True:
        out.append(pts[i % n])
        if i % n == b % n:
            break
        i += 1
    return out


def arc_split(pts, d):
    """Split an open polyline at arc length d."""
    s = arclen(pts, False)
    for i in range(len(pts) - 1):
        if s[i + 1] >= d:
            q = pts[i].lerp(pts[i + 1], (d - s[i]) / max(1e-12, s[i + 1] - s[i]))
            return pts[:i + 1] + [q], [q] + pts[i + 1:]
    return pts, [pts[-1]]


class Shoe:
    """One shoe or boot built from its spec: its upper (the outer surface, its
    panels, the rim and the lining), tongue, sole, heel, laces, eyelets."""

    def __init__(self, spec):
        self.sp = sp = spec
        F = fw_feather(sp)
        n = len(F)
        # the featherline's runs: heel to the outer quarter's front seam, the
        # vamp's run round the toe, the inner quarter's run back to the heel
        itoe = max(range(n), key=lambda i: F[i].z)
        i_sl = next(i for i in range(n) if F[i].z >= sp['seam'][0])
        i_sm = next(i for i in range(itoe, n) if F[i].z <= sp['seam'][1])
        f_lat = piece(F, 0, i_sl)
        f_vamp = piece(F, i_sl, i_sm)
        f_med = piece(F, i_sm, n)                  # back to index 0
        # the topline: up the outer side, round the throat, back the inner side
        tl = [Vector(p) for p in sp['top_lat']]
        tm = [Vector((-p.x, p.y + sp['med_dy'] * smoothstep(-0.14, -0.10, p.z) * (1 - smoothstep(-0.05, -0.03, p.z)), p.z)) for p in tl]
        tl = chaikin(tl, 2, closed=False)
        tm = chaikin(tm, 2, closed=False)
        th = Vector(sp['throat'])
        r = abs(tl[-1].x) + 0.0005
        u_pts = []
        for k in range(9):
            a = math.pi * k / 8
            u_pts.append(Vector((-r * math.cos(a), th.y + (tl[-1].y - th.y) * (1 - math.sin(a)), th.z + (tl[-1].z - th.z) * (1 - math.sin(a)))))
        u_pts[0], u_pts[-1] = tl[-1].copy(), tm[-1].copy()
        nb, nq, nv = sp['cols']
        bw = sp['back_w']
        t_bl, t_ql = arc_split(tl, bw)
        f_bl, f_ql = arc_split(f_lat, bw)
        t_bm, t_qm = arc_split(tm, bw)
        f_bm, f_qm = arc_split(list(reversed(f_med)), bw)
        R = lambda p, k: resample(p, k + 1, closed=False)
        # sections: (name, topline points, featherline points, panel offset)
        # every run goes the loop's way: from the back up the outer side,
        # round the toe, back down the inner side
        self.sections = [
            ('backL', R(t_bl, nb), R(f_bl, nb), 0.0016),
            ('quarterL', R(t_ql, nq), R(f_ql, nq), 0.0009),
            ('vamp', R(u_pts, nv), R(f_vamp, nv), 0.0),
            ('quarterM', list(reversed(R(t_qm, nq))), list(reversed(R(f_qm, nq))), 0.0009),
            ('backM', list(reversed(R(t_bm, nb))), list(reversed(R(f_bm, nb))), 0.0016),
        ]
        nt = sp['rows']
        self.ts = [(1 - math.cos(math.pi * k / nt)) / 2 for k in range(nt + 1)]
        self.build_grid()

    # ---------------------------------------------------------- the loft
    def axis_point(self, p):
        """The nearest point of a line through the inside of the shoe (and up
        the boot's shaft), to tell outside from in."""
        pts = [Vector((0, 0.030, 0.10)), Vector((0, 0.040, -0.08))]
        if self.sp['hooks']:
            pts.append(Vector((0, 0.22, -0.085)))
        best, bq = 1e9, pts[0]
        for a, b in zip(pts, pts[1:]):
            ab = b - a
            k = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
            q = a + ab * k
            if (p - q).length < best:
                best, bq = (p - q).length, q
        return bq

    def build_grid(self):
        sp = self.sp
        cols = []           # (section index, topline point, featherline point)
        for si, (name, T, F, off) in enumerate(self.sections):
            for a, b in zip(T, F):
                cols.append((si, a, b))
        self.cols = cols
        nc, nt = len(cols), len(self.ts)
        base = [[a.lerp(b, t) for t in self.ts] for si, a, b in cols]
        # the outward normal of the ruled surface: across, from the nearest
        # columns that are not copies of this one (the loop closes at the back)
        def step(grid, c, k, d):
            i = c
            for _ in range(nc):
                i = i + d
                if i < 0:
                    i = nc - 2          # column nc-1 is column 0 again
                elif i > nc - 1:
                    i = 1
                if (grid[i][k] - grid[c][k]).length > 1e-7:
                    return i
            return c

        def nrm(grid, c, k):
            ds = grid[step(grid, c, k, 1)][k] - grid[step(grid, c, k, -1)][k]
            dt = grid[c][min(k + 1, nt - 1)] - grid[c][max(k - 1, 0)]
            m = ds.cross(dt)
            if m.length < 1e-12:
                m = grid[c][k] - self.axis_point(grid[c][k])
            m.normalize()
            if m.dot(grid[c][k] - self.axis_point(grid[c][k])) < 0:
                m = -m
            return m
        kb, kq, kv = sp['bulge']
        zs = sp['seam']
        bul = []
        for c, (si, a, b) in enumerate(cols):
            L = (a - b).length
            # how much it bulges: by where its foot is (so the copies of a
            # column at a seam bulge alike): the vamp's dome, the quarters,
            # the back's straighter counter
            k = kq + (kv - kq) * smoothstep(max(zs) - 0.02, max(zs) + 0.05, b.z) + (kb - kq) * smoothstep(-0.11, -0.145, b.z)
            row = []
            for j, t in enumerate(self.ts):
                n = nrm(base, c, j)
                row.append(base[c][j] + n * (k * L * math.sin(math.pi * t) ** 0.9))
            bul.append(row)
        bul = [[self.shape(p) for p in row] for row in bul]
        self.B = bul
        self.N = [[nrm(bul, c, j) for j in range(nt)] for c in range(nc)]
        # do the grid's quads, (c, j) (c+1, j) (c+1, j+1) (c, j+1), face out?
        vote = 0.0
        for c in range(nc - 1):
            for j in range(nt - 1):
                a, b, d = bul[c][j], bul[c + 1][j], bul[c][j + 1]
                n = (b - a).cross(d - a)
                ctr = (a + b + d) / 3
                vote += 1 if n.dot(ctr - self.axis_point(ctr)) > 0 else -1
        self.flip = vote < 0

    def shape(self, p):
        """What wear and the foot left in the leather."""
        sp = self.sp
        ft = catmull(sp['ytop'], p.z)
        h = p.y - ft
        # creases across the vamp where the foot bent at the ball
        z0 = 0.050 if not sp['hooks'] else 0.056
        if p.z > z0 - 0.01 and h > 0.012:
            k = smoothstep(0.012, 0.035, h)
            zz = p.z - 0.10 * p.x * p.x / 0.05
            for i, (dz, dep) in enumerate([(0.0, 0.0012), (0.011, 0.0017), (0.023, 0.0013), (0.034, 0.0007)]):
                c = z0 + dz + 0.002 * math.sin(p.x * 90.0 + i)
                p = p - Vector((0, dep * k * math.exp(-((zz - c) / 0.0028) ** 2), 0))
        # the toe turned up a little more than it was made (the toe spring of wear)
        if p.z > 0.09:
            p = p + Vector((0, 0.004 * ((p.z - 0.09) / 0.06) ** 2, 0))
        if sp['hooks']:
            # the shaft: in at the back above the heel (the Achilles), the
            # leather slumped forward a little on the outer side
            if p.z < -0.09 and p.y > 0.07:
                p = p + Vector((0, 0, 0.007 * math.sin(math.pi * min(1.0, (p.y - 0.07) / 0.11)) * smoothstep(-0.11, -0.13, p.z)))
            if p.y > 0.10:
                p = p + Vector((-0.004 * smoothstep(0.10, 0.18, p.y) * smoothstep(0.0, -0.04, p.x), 0, 0))
        # the heel counter trodden down a little on the outer side
        if p.z < -0.10 and p.x < 0 and h > 0.02:
            p = p - Vector((0.002 * smoothstep(0.02, 0.045, h), 0.0015 * smoothstep(0.03, 0.045, h), 0))
        # lumps in old leather
        q = p * 55.0
        return p + noise3(q + Vector((1.7, 2.9, 0.3))) * 0.0006

    def lining_rows(self):
        return [0.0, 0.08, 0.3, 0.6, 1.0] if not self.sp['hooks'] else [0.0, 0.05, 0.2, 0.45, 0.7, 1.0]

    def is_collar(self, c):
        si, a, b = self.cols[c]
        name = self.sections[si][0]
        return name.startswith('back') or (name.startswith('quarter') and a.z < self.sp['facing'][0] - 0.003)

    def lining_point(self, c, t):
        """A point of the lining: the outer surface less the leather's (and
        the collar's padding's) thickness; at its foot the insole's edge."""
        sp = self.sp
        si, a, b = self.cols[c]
        # nearest grid row
        j = min(range(len(self.ts)), key=lambda j: abs(self.ts[j] - t))
        p, n = self.B[c][j], self.N[c][j]
        th = sp['rim']
        if self.is_collar(c):
            th += sp['pad'] * (1 - smoothstep(0.0, 0.3, t))
        if t >= 1.0:
            # the insole's edge, in from the featherline
            q = Vector((b.x, 0, b.z))
            ctr = self.axis_point(b)
            d = Vector((ctr.x - b.x, 0, ctr.z - b.z))
            d = d.normalized() if d.length > 1e-9 else Vector((0, 0, 0))
            q = q + d * 0.0040
            return Vector((q.x, catmull(sp['ytop'], q.z) + 0.0045, q.z))
        return p - n * th

    def upper(self):
        """The outer surface, panel by panel (a step where one laps over the
        next), and its rim rolled over the topline into the lining."""
        nt, nc = len(self.ts), len(self.cols)
        v, f = [], []
        idx = {}
        for c, (si, a, b) in enumerate(self.cols):
            off = self.sections[si][3]
            for j in range(nt):
                idx[(c, j)] = len(v)
                v.append(self.B[c][j] + self.N[c][j] * off)
        for c in range(nc - 1):
            for j in range(nt - 1):
                f.append((idx[(c, j)], idx[(c + 1, j)], idx[(c + 1, j + 1)], idx[(c, j + 1)]))
        t0 = self.lining_rows()[0]
        lin = [self.lining_point(c, t0) for c in range(nc)]
        crest = []
        for c in range(nc):
            si, a, b = self.cols[c]
            o = v[idx[(c, 0)]]
            up = (a - b).normalized()
            w = (o - lin[c]).length
            crest.append((o + lin[c]) / 2 + up * w * (0.42 if not self.is_collar(c) else 0.30))
        base_c = len(v)
        v += crest
        base_l = len(v)
        v += lin
        for c in range(nc - 1):
            f.append((idx[(c + 1, 0)], idx[(c, 0)], base_c + c, base_c + c + 1))
            f.append((base_c + c + 1, base_c + c, base_l + c, base_l + c + 1))
        if self.flip:
            f = [tuple(reversed(q)) for q in f]
        return weld(Shape(v, f), 1e-7)

    def lining(self):
        """The lining (the leather's inside, its collar padded) and the insole."""
        rows = self.lining_rows()
        nc, nl = len(self.cols), len(rows)
        v, f = [], []
        for c in range(nc):
            for t in rows:
                v.append(self.lining_point(c, t))
        for c in range(nc - 1):
            for j in range(nl - 1):
                f.append(((c + 1) * nl + j, c * nl + j, c * nl + j + 1, (c + 1) * nl + j + 1))
        if self.flip:
            f = [tuple(reversed(q)) for q in f]
        s = weld(Shape(v, f), 1e-7)
        loop = []
        for c in range(nc):
            p = v[c * nl + nl - 1]
            q = Vector((p.x, p.z))
            if not loop or (q - loop[-1]).length > 1e-6:
                loop.append(q)
        if (loop[0] - loop[-1]).length < 1e-6:
            loop.pop()
        ins = cap_fn(loop, 0.012, lambda x, z: catmull(self.sp['ytop'], z) + 0.0045, up=True, seed=31)
        return s + ins

    # ---------------------------------------------------------- the rest
    def facing_points(self, side, fr):
        """The point a fraction fr up the lacing (0 its foot by the throat,
        1 its top), on the outer (side -1) or inner (+1) quarter's edge:
        (column, its topline point)."""
        name = 'quarterL' if side < 0 else 'quarterM'
        cs = [c for c, (si, a, b) in enumerate(self.cols) if self.sections[si][0] == name and a.z > self.sp['facing'][0] - 0.004]
        cs.sort(key=lambda c: self.cols[c][1].z)
        # arc length along these columns' topline points, from the throat end
        pts = [self.cols[c][1] for c in reversed(cs)]
        s = arclen(pts, False)
        tgt = s[-1] * fr
        k = min(range(len(s)), key=lambda i: abs(s[i] - tgt))
        return list(reversed(cs))[k]

    def down_col(self, c, d):
        """The grid row d below the topline along column c."""
        acc = 0.0
        for j in range(1, len(self.ts)):
            acc += (self.B[c][j] - self.B[c][j - 1]).length
            if acc >= d:
                return j
        return len(self.ts) - 1

    def eyelet_pts(self):
        """The eyelets (and hooks) up each side: (position, outward normal,
        along the edge)."""
        sp = self.sp
        ne, nh = sp['eyelets'], sp['hooks']
        out = {-1: [], 1: []}
        for side in (-1, 1):
            off = self.sections[1][3]
            for i in range(ne + nh):
                if nh:
                    fr = 0.06 + 0.40 * i / (ne - 1) if i < ne else 0.58 + 0.36 * (i - ne) / max(1, nh - 1)
                else:
                    fr = 0.08 + 0.84 * i / (ne - 1)
                c = self.facing_points(side, fr)
                j = self.down_col(c, 0.0065 if i < ne else 0.0075)
                p = self.B[c][j] + self.N[c][j] * (off + 0.0004)
                out[side].append((p, self.N[c][j], c))
        return out

    def onto(self, q, lift):
        """q moved onto the outer surface (its nearest point's tangent plane)
        and lift out from it."""
        best, bp, bn = 1e9, None, None
        for c in range(len(self.cols)):
            for j in range(len(self.ts)):
                d = (self.B[c][j] - q).length
                if d < best:
                    best, bp, bn = d, self.B[c][j], self.N[c][j]
        return q + bn * ((bp - q).dot(bn) + lift)

    def laces(self):
        sp = self.sp
        E = self.eyelet_pts()
        ne = sp['eyelets']
        r = 0.0015
        segs = []

        def lace(a, b, sag):
            m = (a + b) / 2
            m = m - (m - self.axis_point(m)).normalized() * sag
            return tube([a, a.lerp(m, 0.5), m, m.lerp(b, 0.5), b], r, seg=5, caps=True)

        L, M = E[-1], E[1]
        sink = lambda e: e[0] - e[1] * 0.0012
        segs.append(lace(sink(L[0]), sink(M[0]), 0.0004))
        for i in range(ne - 1):
            segs.append(lace(sink(L[i]), sink(M[i + 1]), 0.0006))
            segs.append(lace(sink(M[i]), sink(L[i + 1]), -0.0004))
        top_l, top_m = L[ne - 1][0], M[ne - 1][0]
        if not sp['hooks']:
            # untied, the shoe kicked off: the two ends out of the top pair,
            # slack, lying down over the quarters
            for side, e, steps in ((-1, L[ne - 1], [(-0.004, 0.004, 0.006), (-0.010, -0.004, 0.008), (-0.010, -0.012, -0.002), (-0.006, -0.016, -0.010)]),
                                   (1, M[ne - 1], [(0.004, 0.005, -0.004), (0.011, -0.002, -0.006), (0.012, -0.012, 0.000), (0.008, -0.014, 0.008)])):
                p0 = e[0] + e[1] * 0.002
                pts = [p0]
                q = p0
                for dx, dy, dz in steps:
                    q = self.onto(q + Vector((dx, dy, dz)), r + 0.0006)
                    pts.append(q)
                segs.append(tube(pts, r, seg=5, caps=True, smooth=2))
                d = (pts[-1] - pts[-2]).normalized()
                segs.append(tube([pts[-1], pts[-1] + d * 0.012], r * 1.2, seg=6, caps=True))
        else:
            # undone: the two ends out of the top eyelets, down the shaft's
            # sides to the floor
            for side, e, steps in ((-1, L[ne - 1], [(-0.004, 0.006, 0.010), (-0.010, -0.008, 0.016), (-0.006, -0.020, 0.012), (0.004, -0.018, 0.018),
                                                    (0.010, -0.012, 0.022)]),
                                   (1, M[ne - 1], [(0.005, 0.008, 0.006), (0.012, -0.004, 0.004), (0.014, -0.016, -0.004), (0.008, -0.020, -0.010)])):
                p0 = e[0] + e[1] * 0.002
                pts = [p0]
                q = p0
                for dx, dy, dz in steps:
                    q = self.onto(q + Vector((dx, dy, dz)), r + 0.0006)
                    pts.append(q)
                segs.append(tube(pts, r, seg=5, caps=True, smooth=2))
                # its aglet
                d = (pts[-1] - pts[-2]).normalized()
                segs.append(tube([pts[-1], pts[-1] + d * 0.014], r * 1.25, seg=6, caps=True))
        return sum(segs[1:], segs[0])

    def eyelets(self):
        """The eyelets (blind rings in the facing) and, up a boot's shin, the
        speed hooks, each curling over toward the lacing."""
        sp = self.sp
        E = self.eyelet_pts()
        ne = sp['eyelets']
        parts = []
        ring = weld(lathe([(0.0016, 0.0), (0.0030, 0.0), (0.0030, 0.0009), (0.0016, 0.0009), (0.0016, 0.0)], seg=8,
                          cap_bottom=False, cap_top=False), 1e-7)
        ring = propkit._fix_winding(ring)
        for side in (-1, 1):
            other = E[-side]
            for i, (p, n, c) in enumerate(E[side]):
                d = other[i][0] - p
                xa = (d - n * d.dot(n)).normalized()
                za = xa.cross(n)
                m = Matrix(((xa.x, n.x, za.x, 0), (xa.y, n.y, za.y, 0), (xa.z, n.z, za.z, 0), (0, 0, 0, 1)))
                if i < ne:
                    g = ring.copy().move(0, -0.0003, 0)
                else:
                    foot = cyl(0.0030, h=0.0012, seg=8, bevel_r=0.0004, bevel_seg=1).move(0, 0.0004, 0)
                    hk = tube([(0, 0.0008, 0), (0, 0.0036, 0), (0.0020, 0.0054, 0), (0.0042, 0.0050, 0), (0.0052, 0.0031, 0)],
                              0.0011, seg=5, caps=True)
                    g = foot + hk
                g.apply(m)
                parts.append(g.move(*p))
        return sum(parts[1:], parts[0])

    def tongue(self):
        """The tongue down the throat under the laces, its top out above the
        lacing, flopped forward a little."""
        sp = self.sp
        tg = sp['tongue']
        # under the laces: below the line between each pair of eyelets
        E = self.eyelet_pts()
        ne = sp['eyelets'] + sp['hooks']
        pts = []
        for i in range(ne):
            m = (E[-1][i][0] + E[1][i][0]) / 2
            m = m - (m - self.axis_point(m)).normalized() * (tg['up'] + 0.0025)
            pts.append(m)
        # down to the throat
        th = Vector(sp['throat'])
        pts.insert(0, th - (th - self.axis_point(th)).normalized() * (tg['up'] + 0.002))
        d = (pts[-1] - pts[-2]).normalized()
        top = pts[-1] + d * tg['ext'] + Vector((0, 0, 0.004))
        pts.append(top)
        # its foot goes in under the vamp's throat
        pts.insert(0, pts[0] + (pts[0] - pts[1]).normalized() * 0.006 - Vector((0, 0.002, 0)))
        w, th = tg['w'], tg['th']
        sec = []
        n = 14
        for i in range(n):
            a = TAU * i / n
            x = w / 2 * math.copysign(abs(math.cos(a)) ** 0.35, math.cos(a))
            y = th / 2 * math.sin(a)
            y += -0.10 * (x * x) / (w / 2) * (1.0)          # bowed over the instep
            sec.append((x, y))
        return sweep(pts, sec, up=(0, 1, 0), smooth=1)

    def pull_loop(self):
        """The boot's pull loop at the back of the collar."""
        c = 0
        a = self.B[0][0]
        n = self.N[0][0]
        # webbing sewn down the back, looped over the collar's top
        pts = [a - Vector((0, 0.024, 0)) + n * 0.0016, a - Vector((0, 0.004, 0)) + n * 0.0020, a + Vector((0, 0.010, 0)) + n * 0.0028,
               a + Vector((0, 0.017, 0)) + n * 0.0002, a + Vector((0, 0.013, 0)) - n * 0.0030, a + Vector((0, 0.003, 0)) - n * 0.0050]
        return sweep(pts, [(0.0060 * math.copysign(abs(math.cos(TAU * i / 10)) ** 0.4, math.cos(TAU * i / 10)), 0.0009 * math.sin(TAU * i / 10))
                           for i in range(10)], up=(0, 0, 1), smooth=2)

    # ---------------------------------------------------------- the sole
    def sole_y(self, z):
        """(the sole's top, its underside) at z."""
        top = catmull(self.sp['ytop'], z)
        return top, top - catmull(self.sp['thick'], z)

    def sole(self):
        sp = self.sp
        O = fw_outline(sp, 72)
        lug = bool(sp['hooks'])

        # (a lug sole's tread is painted on its underside; its lugs' ends
        # are cut into the edge)
        return sole_slab(O, lambda x, z: self.sole_y(z)[1], lambda x, z: self.sole_y(z)[0], step=0.011, notch=lug)

    def heel(self):
        sp = self.sp
        O = fw_outline(sp, 120)
        zb = sp['breast']
        # the rear run of the (clockwise) outline, from the inner side's
        # breast round the back to the outer side's
        run = []
        n = len(O)
        start = next(i for i in range(n) if O[i].y < zb and O[i - 1].y >= zb)
        i = start
        while O[i % n].y < zb:
            run.append(O[i % n])
            i += 1
        # the breast, curved in a little
        a, b = run[-1], run[0]
        br = [a.lerp(b, k / 6) + Vector((0, 0.003 * math.sin(math.pi * k / 6))) for k in range(1, 6)]
        loop = run + br
        top = lambda x, z: self.sole_y(z)[0] - catmull(sp['thick'], z) + 0.0008
        # the top piece worn off at the back edge, the outer side most
        worn = lambda x, z: 0.0016 * smoothstep(-0.128, -0.150 - 0.004 * (x < 0), z) * (1.2 if x < 0 else 0.8)
        return sole_slab(loop, worn, top, step=0.012, notch=bool(sp['hooks']))


def cap_fn(loop, step, yfun, up, seed):
    lo = (min(p.x for p in loop), min(p.y for p in loop))
    hi = (max(p.x for p in loop), max(p.y for p in loop))
    poly = list(loop)

    def inside(x, z):
        c = False
        j = len(poly) - 1
        for i in range(len(poly)):
            a, b = poly[i], poly[j]
            if (a.y > z) != (b.y > z) and x < (b.x - a.x) * (z - a.y) / (b.y - a.y) + a.x:
                c = not c
            j = i
        return c

    def far(x, z):
        if not inside(x, z):
            return False
        p = Vector((x, z))
        return all((p - q).length > step * 0.5 for q in poly[::2])
    pts = grid_points(lo, hi, step, far, jitter=0.0, seed=seed)
    v2, tris = cdt(loop, pts, [])
    return lift_tris(v2, tris, yfun, up=up)


def sole_slab(loop, ybot, ytop, step, notch=False):
    """A slab on a plan outline: its underside ybot(x, z), its top ytop(x, z),
    the edge between rounded off; notch cuts the lugs' ends into the edge."""
    n = len(loop)
    cen = sum(loop, Vector((0.0, 0.0))) / n

    def ins(i, d):
        p = loop[i]
        a, b = loop[i - 1], loop[(i + 1) % n]
        t = (b - a).normalized()
        nn = Vector((t.y, -t.x))
        if nn.dot(cen - p) < 0:
            nn = -nn
        return p + nn * d
    rows = []
    # the edge from the underside up: rounded at the foot, the lugs' ends
    for k, (fy, d) in enumerate([(0.0, 0.0012), (0.12, 0.0), (0.88, 0.0), (1.0, 0.0006)]):
        row = []
        for i in range(n):
            q = ins(i, d + (0.0022 if notch and k <= 1 and i % 2 else 0.0))
            yb, yt = ybot(q.x, q.y), ytop(q.x, q.y)
            row.append(Vector((q.x, yb + (yt - yb) * fy, q.y)))
        rows.append(row)
    walls = loft_rows(rows)
    bot = cap_fn([Vector((p.x, p.z)) for p in rows[0]], step, ybot, up=False, seed=41)
    top = cap_fn([Vector((p.x, p.z)) for p in rows[-1]], step * 1.6, ytop, up=True, seed=42)
    # the caps' rims take the edge rows' heights
    s = walls + bot + top
    s = weld(s, 2e-4)
    return propkit._fix_winding(s)


def sweep(path, section, up=(0, 1, 0), scale=None, caps=True, smooth=0):
    """Sweep a closed 2D section [(a, b)] along a polyline: a along the
    side, b along `up` (carried along the path by parallel transport).
    (After batch r02's.)"""
    pts = [Vector(p) for p in path]
    for _ in range(smooth):
        q = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            q += [a.lerp(b, 0.25), a.lerp(b, 0.75)]
        q.append(pts[-1])
        pts = q
    n = len(pts)
    tan = []
    for i in range(n):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        tan.append(t.normalized())
    u = Vector(up)
    side = tan[0].cross(u)
    if side.length < 1e-6:
        side = tan[0].cross(Vector((1, 0, 0)))
    side.normalize()
    v, f = [], []
    m = len(section)
    for i in range(n):
        if i:
            side = (tan[i - 1].rotation_difference(tan[i]) @ side).normalized()
        b = tan[i].cross(side).normalized() * -1
        sa, sb = scale(i / (n - 1)) if scale else (1.0, 1.0)
        for a_, b_ in section:
            v.append(pts[i] + side * a_ * sa + b * b_ * sb)
    for i in range(n - 1):
        for j in range(m):
            j2 = (j + 1) % m
            f.append((i * m + j, i * m + j2, (i + 1) * m + j2, (i + 1) * m + j))
    if caps:
        f.append(tuple(reversed(range(m))))
        f.append(tuple((n - 1) * m + j for j in range(m)))
    return propkit._fix_winding(Shape(v, f))


def mirror_x(s):
    out = s.copy()
    out.v = [Vector((-p.x, p.y, p.z)) for p in out.v]
    out.f = [tuple(reversed(f)) for f in out.f]
    return out


SHOE = Shoe(DERBY)
BOOT_ = Shoe(BOOT)


def tread_split(s, tread):
    """The faces of a sole that look down (its tread, printed in plan) or the
    rest (its edge and the welt)."""
    def down(fc):
        a, b, c = s.v[fc[0]], s.v[fc[1]], s.v[fc[2]]
        return (b - a).cross(c - a).normalized().y < -0.6
    out = Shape(s.v, [fc for fc in s.f if down(fc) == tread])
    return cut_tris(out, lambda c: True)


def shoe_parts(sh, extra=None):
    sole, heel = sh.sole(), sh.heel()
    up = sh.upper() + sh.tongue()
    if extra:
        up = up + extra
    return {
        'Upper': up,
        'Lining': sh.lining(),
        'Sole': tread_split(sole, False),
        'Heel': tread_split(heel, False),
        'Tread': tread_split(sole, True) + tread_split(heel, True),
        'Laces': sh.laces(),
        'Eyelets': sh.eyelets(),
    }


def lying(parts, com):
    """The parts turned onto the outer side (about the length, z) as far as
    they fall: where the centre of mass, com, sits lowest over the point
    they rest on; then set down on the floor. Returns (parts, the roll)."""
    allv = [p for s in parts.values() for p in s.v]
    best = None
    for k in range(121):
        th = math.radians(60 + k * 0.5)
        c, sn = math.cos(th), math.sin(th)
        lo = min(p.x * sn + p.y * c for p in allv)
        e = com[0] * sn + com[1] * c - lo
        if best is None or e < best[0]:
            best = (e, th, lo)
    e, th, lo = best
    m = Matrix.Rotation(th, 4, 'Z')
    return {k: v.copy().apply(m).move(0, -lo, 0) for k, v in parts.items()}, th


SHOE_P = shoe_parts(SHOE)
SHOE_SIDE, SHOE_ROLL = lying(SHOE_P, (0.0, 0.032))
BOOT_P = shoe_parts(BOOT_, BOOT_.pull_loop())
BOOT_SIDE, BOOT_ROLL = lying(BOOT_P, (0.0, 0.055))
auto('Real__shoeUpperL', lambda: mirror_x(SHOE_P['Upper']), 2200)
auto('Real__shoeUpperSide', lambda: SHOE_SIDE['Upper'], 2200)
auto('Real__shoeLiningL', lambda: mirror_x(SHOE_P['Lining']), 900)
auto('Real__shoeLiningSide', lambda: SHOE_SIDE['Lining'], 900)
auto('Real__shoeSoleL', lambda: mirror_x(SHOE_P['Sole']), 800)
auto('Real__shoeSoleSide', lambda: SHOE_SIDE['Sole'], 800)
auto('Real__shoeHeelL', lambda: mirror_x(SHOE_P['Heel']), 300)
auto('Real__shoeHeelSide', lambda: SHOE_SIDE['Heel'], 300)
auto('Real__shoeTreadL', lambda: mirror_x(SHOE_P['Tread']), 700)
auto('Real__shoeTreadSide', lambda: SHOE_SIDE['Tread'], 700)
auto('Real__shoeLacesL', lambda: mirror_x(SHOE_P['Laces']), 1100)
auto('Real__shoeLacesSide', lambda: SHOE_SIDE['Laces'], 1100)
auto('Real__shoeEyeletsL', lambda: mirror_x(SHOE_P['Eyelets']), 700)
auto('Real__shoeEyeletsSide', lambda: SHOE_SIDE['Eyelets'], 700)
auto('Real__bootUpper', lambda: BOOT_P['Upper'], 3000)
auto('Real__bootUpperSide', lambda: BOOT_SIDE['Upper'], 3000)
auto('Real__bootLining', lambda: BOOT_P['Lining'], 1100)
auto('Real__bootLiningSide', lambda: BOOT_SIDE['Lining'], 1100)
auto('Real__bootSole', lambda: BOOT_P['Sole'], 900)
auto('Real__bootSoleSide', lambda: BOOT_SIDE['Sole'], 900)
auto('Real__bootHeel', lambda: BOOT_P['Heel'], 350)
auto('Real__bootHeelSide', lambda: BOOT_SIDE['Heel'], 350)
auto('Real__bootTread', lambda: BOOT_P['Tread'], 700)
auto('Real__bootTreadSide', lambda: BOOT_SIDE['Tread'], 700)
auto('Real__bootLaces', lambda: BOOT_P['Laces'], 1100)
auto('Real__bootLacesSide', lambda: BOOT_SIDE['Laces'], 1100)
auto('Real__bootHooks', lambda: BOOT_P['Eyelets'], 1200)
auto('Real__bootHooksSide', lambda: BOOT_SIDE['Eyelets'], 1200)
