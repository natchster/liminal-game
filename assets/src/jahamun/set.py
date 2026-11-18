"""The Jahamun set, built in Blender and lit by Cycles (crates/jahamun/README.md).

    blender -b --factory-startup -P assets/src/jahamun/set.py -- [options]

    --out FILE          write the set (default crates/jahamun/assets/set.bin)
    --samples N         samples a baked corner (default 384)
    --only A,B,...      bake only these pieces (the rest is not written)
    --render FILE       instead of baking: a Cycles frame of the set as built
    --view x,y,z,yaw,pitch[,fov]   the frame's camera (the set's frame, below)
    --blend FILE        also save the scene as built (to open in Blender)

The place is built in Blender's frame: x across the bore (the walkway at
-x), y along it (into the tunnel = +y; the portal at y = 0), z up; the set
file is written in the engine's (x, z, -y). Everything is made here from
numbers (no model files): the cross-section, the 6 m module and its
variants, the approach outside the portal, and the light sources. Cycles
bakes each piece's light (direct and indirect, without the surface's own
colour) into its corners' colours; the set multiplies it by the
photographed albedo at run time. A module is baked in the middle of a run of
its neighbours, so its light is the light of an endless tunnel, and its two
ends are made to agree with the plain module's, so any module may follow
any other.
"""
import json
import math
import os
import struct
import sys
import time
import zlib

import bpy
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
SET = os.path.join(ROOT, "crates", "jahamun")
TEX = os.path.join(SET, "assets", "textures")


def args():
    a = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    o = {"out": os.path.join(SET, "assets", "set.bin"), "samples": 384, "only": None, "render": None, "view": None, "blend": None}
    i = 0
    while i < len(a):
        if "=" in a[i]:
            k, val = a[i].lstrip("-").split("=", 1)
            o[k] = val
            i += 1
        else:
            o[a[i].lstrip("-")] = a[i + 1]
            i += 2
    o["samples"] = int(o["samples"])
    if o["only"]:
        o["only"] = set(o["only"].split(","))
    return o


ARGS = args()

# ------------------------------------------------------------- the section ---
# One bore, looking in (+y): the walkway on the left behind the glass.

WALK_Z = 0.20     # the walkway over the road
WALL_X = -1.70    # the tiled wall's face
BASE_X = -0.06    # the barrier's foot, walkway face
GLASS_X = 0.10    # the glass
FOOT_X1 = 0.22    # the foot's road face
SILL_Z = 0.95     # the foot's top on the walkway side
GLASS_Z0 = 1.02   # the glass's foot
HEAD_Z = 3.05     # the glass's head
HEAD_TOP = 3.22
TILE_TOP = 2.25
SPRING = 2.30     # the vault springs from the tiles' top (the video: the lining leans in at once)
RIGHT_X = 9.10    # the far wall
CX = (WALL_X + RIGHT_X) / 2.0
VA = (RIGHT_X - WALL_X) / 2.0     # the vault: half an ellipse
VB = 7.70 - SPRING
ROAD_X1 = 8.60
CURB_Z = 0.25
MOD = 6.0         # a module: three bays
BAY = 2.0
TILE_REPEAT = 4.2  # the tile photograph's repeat on the wall: rows about 13 cm, tiles about 30 cm (the video)
LED_LEN = 1.20
TUNNEL = 486.0    # 81 modules
N_MOD = int(TUNNEL / MOD)
MIRROR = 20.8     # the other bore is this one mirrored about x = MIRROR / 2
UPPER_Z = 7.20    # the street above the stairs
RIGHT_TOP = 8.0   # the ground over the right retaining wall
LEFT_WALL = -5.5  # the left retaining wall's face
RIGHT_WALL = 22.5

# Light (Cycles watts; only their ratios matter: the set's exposure is its own)
LED_W = 9.0       # a walkway bar, 1.2 m
LAMP_W = 90.0     # a road lamp in the vault
STREET_W = 140.0  # a street lamp head outside (its beam 110 degrees: pools, the dark between)
RAIL_W = 1.6      # 3 m of LED strip under the balustrade's rail
RED_W = 2.5
COLD_W = 40.0     # the light deep in the passage behind the open door

# The escape door in the tiled wall (variants F and G, the way in:
# docs/design/WAY_IN.md): its leaf, its frame, where it stands in its module
DOOR_W = 0.95
DOOR_TOP = 2.15
DOOR_Y = 3.0
DOOR_FRAME = 0.06
DOOR_DEPTH = 0.06
DOOR_OPEN = 32.0  # degrees: the changed door stands a hand's width open
PASSAGE = 3.2
SIGNAL_W = 4.0
WINDOW_W = 3.0

COOL = (0.93, 0.98, 1.0)      # the tunnel's LEDs (~5500 K, a little green in the video)
COLD = (0.80, 0.86, 0.93)      # behind the open door (the way in): a cold grey light
LAMP_C = (0.95, 0.98, 1.0)
WARM = (1.0, 0.80, 0.58)      # street lamps (~3300 K)
RED = (1.0, 0.06, 0.03)
GREEN = (0.1, 1.0, 0.35)
WINDOW = (1.0, 0.78, 0.5)


def vault_point(t):
    return (CX + VA * math.cos(t), SPRING + VB * math.sin(t))


def vault_t_at_x(x):
    c = max(-1.0, min(1.0, (x - CX) / VA))
    return math.acos(c)


def resample(pts, step):
    """A polyline resampled every `step` of its length (ends kept)."""
    P = np.array(pts, dtype=np.float64)
    seg = np.linalg.norm(np.diff(P, axis=0), axis=1)
    s = np.concatenate([[0.0], np.cumsum(seg)])
    n = max(1, int(math.ceil(s[-1] / step)))
    t = np.linspace(0.0, s[-1], n + 1)
    out = np.stack([np.interp(t, s, P[:, k]) for k in range(P.shape[1])], axis=1)
    return out, t


def vault_profile(step, t0=math.pi, t1=0.0):
    ts = np.linspace(t0, t1, 400)
    pts = [vault_point(t) for t in ts]
    return resample(pts, step)


CANOPY_X = -1.20
CANOPY_T = vault_t_at_x(CANOPY_X)
CANOPY_TOP = vault_point(CANOPY_T)        # where the lean-to meets the vault
CANOPY_LOW = (GLASS_X, HEAD_TOP)

# ------------------------------------------------------------- the kit ---


class Mesh:
    """A mesh in the making: vertices, faces (with a uv a corner), its
    material and whether it is shaded smooth."""

    def __init__(self, name, mat, smooth=False):
        self.name, self.mat, self.smooth = name, mat, smooth
        self.co, self.faces, self.uvs = [], [], []

    def v(self, p):
        self.co.append((float(p[0]), float(p[1]), float(p[2])))
        return len(self.co) - 1

    def face(self, idx, uv, want=None):
        if want is not None:
            P = [self.co[i] for i in idx]
            n = [0.0, 0.0, 0.0]
            for a in range(len(P)):
                p, q = P[a], P[(a + 1) % len(P)]
                n[0] += (p[1] - q[1]) * (p[2] + q[2])
                n[1] += (p[2] - q[2]) * (p[0] + q[0])
                n[2] += (p[0] - q[0]) * (p[1] + q[1])
            c = [sum(p[k] for p in P) / len(P) for k in range(3)]
            w = want(c) if callable(want) else want
            if n[0] * w[0] + n[1] * w[1] + n[2] * w[2] < 0:
                idx, uv = idx[::-1], uv[::-1]
        self.faces.append(list(idx))
        self.uvs.append([tuple(map(float, u)) for u in uv])

    def grid(self, P, U, ni, nj, want):
        """Vertices P(i, j) on an (ni+1) x (nj+1) grid, uv U(i, j); quads
        facing `want` (a vector or a function of the face's centre)."""
        ids = [[self.v(P(i, j)) for j in range(nj + 1)] for i in range(ni + 1)]
        for i in range(ni):
            for j in range(nj):
                q = [ids[i][j], ids[i + 1][j], ids[i + 1][j + 1], ids[i][j + 1]]
                uv = [U(i, j), U(i + 1, j), U(i + 1, j + 1), U(i, j + 1)]
                self.face(q, uv, want)

    def quad(self, a, b, c, d, step, want, uvf):
        """A planar quad a-b-c-d split into a grid about `step` apart; uv
        from uvf(point)."""
        A, B, C, D = (np.array(x, np.float64) for x in (a, b, c, d))
        ni = max(1, int(math.ceil(np.linalg.norm(B - A) / step)))
        nj = max(1, int(math.ceil(np.linalg.norm(D - A) / step)))

        def P(i, j):
            u, v = i / ni, j / nj
            return (A * (1 - u) + B * u) * (1 - v) + (D * (1 - u) + C * u) * v

        self.grid(P, lambda i, j: uvf(P(i, j)), ni, nj, want)

    def box(self, lo, hi, step, uvs=1.0):
        """An axis-aligned box, its faces outward, uv by world box mapping."""
        lo, hi = np.array(lo, np.float64), np.array(hi, np.float64)
        self.obox(lo, np.eye(3) * (hi - lo), step, uvs)

    def obox(self, o, axes, step, uvs=1.0):
        """A box at corner `o` spanned by three edge vectors (rows of `axes`)."""
        o = np.array(o, np.float64)
        a, b, c = (np.array(x, np.float64) for x in axes)
        centre = o + (a + b + c) / 2
        faces = [
            (o, o + a, o + a + c, o + c),
            (o + b, o + c + b, o + a + c + b, o + a + b),
            (o, o + c, o + c + b, o + b),
            (o + a, o + a + b, o + a + b + c, o + a + c),
            (o, o + b, o + a + b, o + a),
            (o + c, o + a + c, o + a + b + c, o + b + c),
        ]
        for f in faces:
            fc = sum(f) / 4.0
            out = fc - centre
            e1, e2 = f[1] - f[0], f[3] - f[0]
            l1, l2 = np.linalg.norm(e1), np.linalg.norm(e2)
            if l1 < 1e-9 or l2 < 1e-9:
                continue
            d1, d2 = e1 / l1, e2 / l2
            self.quad(*f, step, tuple(out), lambda p, d1=d1, d2=d2: (np.dot(p, d1) / uvs, np.dot(p, d2) / uvs))

    def tube(self, p0, p1, r, sides=8, step=0.5, uvs=1.0):
        """A round pipe from p0 to p1 (open ends), shaded smooth by its own mesh."""
        p0, p1 = np.array(p0, np.float64), np.array(p1, np.float64)
        d = p1 - p0
        L = np.linalg.norm(d)
        d /= L
        up = np.array([0, 0, 1.0]) if abs(d[2]) < 0.9 else np.array([1.0, 0, 0])
        e1 = np.cross(d, up)
        e1 /= np.linalg.norm(e1)
        e2 = np.cross(d, e1)
        nl = max(1, int(math.ceil(L / step)))

        def P(i, j):
            a = 2 * math.pi * i / sides
            return p0 + d * (L * j / nl) + r * (math.cos(a) * e1 + math.sin(a) * e2)

        def want(c):
            c = np.array(c)
            t = np.dot(c - p0, d)
            return tuple(c - (p0 + d * t))

        self.grid(P, lambda i, j: (i / sides * 2 * math.pi * r / uvs, L * j / nl / uvs), sides, nl, want)


def merge(meshes):
    """Meshes of one material and shading joined."""
    out = {}
    for m in meshes:
        k = (m.name, m.mat, m.smooth)
        if k not in out:
            out[k] = Mesh(m.name, m.mat, m.smooth)
        o = out[k]
        base = len(o.co)
        o.co += m.co
        o.faces += [[i + base for i in f] for f in m.faces]
        o.uvs += m.uvs
    return list(out.values())


class Kit:
    """What a piece is made of: meshes by name and the lights in it."""

    def __init__(self):
        self.meshes = {}
        self.lights = []  # dicts: kind, centre, size, dir, colour, watts, and the run-time hint

    def m(self, name, mat, smooth=False):
        k = (name, mat, smooth)
        if k not in self.meshes:
            self.meshes[k] = Mesh(name, mat, smooth)
        return self.meshes[k]

    def light(self, kind, centre, size, direction, colour, watts, axis=(0, 1, 0), spread=180.0):
        self.lights.append(dict(kind=kind, centre=tuple(map(float, centre)), size=tuple(map(float, size)),
                                dir=tuple(map(float, direction)), axis=tuple(map(float, axis)), colour=tuple(colour), watts=float(watts),
                                spread=float(spread)))


def U_world(scale, a=0, b=1):
    return lambda p: (p[a] / scale, p[b] / scale)


# ----------------------------------------------------------- the module ---

def module(kit, y0, variant, dense=True, mirror=False):
    """One 6 m module of the bore at y0..y0+6: the walkway, the barrier,
    the lean-to, the vault, the road and its lamps; and the variant's
    extras (A plain; B a floor hatch and an exit sign; C a hydrant cabinet,
    an info sign, a down pipe; D one bar dead and a hatch; E jet fans; F
    the escape door, shut; G the escape door changed, open, her note)."""
    f = 1.0 if dense else 3.0
    ny = max(2, int(MOD / (0.10 * f)))
    ny2 = max(2, int(MOD / (0.25 * f)))
    y1 = y0 + MOD
    Y = lambda j, n: y0 + MOD * j / n

    # the floor: u across 1.8 m (0 at the wall), v the module (the texture is a module long)
    xs = np.linspace(WALL_X, BASE_X, max(2, int(round((BASE_X - WALL_X) / (0.08 * f)))) + 1)
    kit.m("floor", "floor").grid(lambda i, j: (xs[i], Y(j, ny), WALK_Z), lambda i, j: ((xs[i] - WALL_X) / 1.8, j / ny),
                                 len(xs) - 1, ny, (0, 0, 1))
    # the tiled wall (cut for the escape door in F and G)
    zs = np.linspace(WALK_Z, TILE_TOP, max(2, int(round((TILE_TOP - WALK_Z) / (0.10 * f)))) + 1)

    def tiles(ya, yb):
        n = max(2, int(round((yb - ya) / (0.10 * f))))
        Yt = lambda j: ya + (yb - ya) * j / n
        kit.m("tile", "tile").grid(lambda i, j: (WALL_X, Yt(j), zs[i]), lambda i, j: (Yt(j) / TILE_REPEAT, (zs[i] - WALK_Z) / TILE_REPEAT),
                                   len(zs) - 1, n, (1, 0, 0))
    if variant in ("F", "G"):
        yd = y0 + DOOR_Y
        tiles(y0, yd - DOOR_W / 2 - DOOR_FRAME)
        tiles(yd + DOOR_W / 2 + DOOR_FRAME, y1)
    else:
        tiles(y0, y1)
    # its top trim, a steel angle
    t = kit.m("trim", "steel_dark")
    t.quad((WALL_X + 0.025, y0, TILE_TOP), (WALL_X + 0.025, y1, TILE_TOP), (WALL_X + 0.025, y1, TILE_TOP + 0.04),
           (WALL_X + 0.025, y0, TILE_TOP + 0.04), 0.5 * f, (1, 0, 0), U_world(1.0, 1, 2))
    t.quad((WALL_X, y0, TILE_TOP + 0.04), (WALL_X, y1, TILE_TOP + 0.04), (WALL_X + 0.025, y1, TILE_TOP + 0.04),
           (WALL_X + 0.025, y0, TILE_TOP + 0.04), 0.5 * f, (0, 0, 1), U_world(1.0, 1, 0))
    t.quad((WALL_X, y0, TILE_TOP), (WALL_X, y1, TILE_TOP), (WALL_X + 0.025, y1, TILE_TOP),
           (WALL_X + 0.025, y0, TILE_TOP), 0.5 * f, (0, 0, -1), U_world(1.0, 1, 0))
    # the lining over the tiles, then the vault over both
    zb = np.linspace(TILE_TOP + 0.04, SPRING, 3)
    kit.m("band", "vault").grid(lambda i, j: (WALL_X, Y(j, ny2), zb[i]), lambda i, j: (Y(j, ny2) / 3.2, zb[i] / 3.2),
                                len(zb) - 1, ny2, (1, 0, 0))
    prof, s = vault_profile(0.22 * f)
    kit.m("vault", "vault", smooth=True).grid(lambda i, j: (prof[i][0], Y(j, ny2), prof[i][1]), lambda i, j: (s[i] / 3.2, Y(j, ny2) / 3.2),
                                              len(prof) - 1, ny2, lambda c: (CX - c[0], 0, SPRING - c[2]) if c[2] > SPRING else (CX - c[0], 0, 0))
    # the far wall: tiles, a green band, tiles
    for (z0, z1, mat) in ((CURB_Z, 1.45, "tile"), (1.45, 1.75, "tile_green"), (1.75, SPRING, "tile")):
        zz = np.linspace(z0, z1, max(2, int(round((z1 - z0) / (0.25 * f)))) + 1)
        kit.m("far_wall", mat).grid(lambda i, j, zz=zz: (RIGHT_X, Y(j, ny2), zz[i]), lambda i, j, zz=zz: (Y(j, ny2) / TILE_REPEAT, zz[i] / TILE_REPEAT),
                                    len(zz) - 1, ny2, (-1, 0, 0))
    # the road, its far curb, its lines
    xr = np.linspace(FOOT_X1, ROAD_X1, max(2, int(round((ROAD_X1 - FOOT_X1) / (0.3 * f)))) + 1)
    kit.m("road", "road").grid(lambda i, j: (xr[i], Y(j, ny2), 0.0), lambda i, j: (xr[i] / 3.0, Y(j, ny2) / 3.0), len(xr) - 1, ny2, (0, 0, 1))
    c = kit.m("curb", "concrete")
    c.quad((ROAD_X1, y0, 0), (ROAD_X1, y1, 0), (ROAD_X1, y1, CURB_Z), (ROAD_X1, y0, CURB_Z), 0.25 * f, (-1, 0, 0), U_world(1.0, 1, 2))
    c.quad((ROAD_X1, y0, CURB_Z), (ROAD_X1, y1, CURB_Z), (RIGHT_X, y1, CURB_Z), (RIGHT_X, y0, CURB_Z), 0.25 * f, (0, 0, 1), U_world(1.0, 1, 0))
    ln = kit.m("lines", "white_paint")
    for (xa, ya, yb) in ((0.50, y0, y1), (8.30, y0, y1), (4.40, y0 + 0.5, y0 + 3.5)):
        ln.quad((xa, ya, 0.004), (xa, yb, 0.004), (xa + 0.15, yb, 0.004), (xa + 0.15, ya, 0.004), 0.5 * f, (0, 0, 1), U_world(1.0, 1, 0))
    # the barrier's foot: its walkway face (the panel texture, a bay a repeat), the sill, its road face
    zp = np.linspace(WALK_Z, SILL_Z, max(2, int(round((SILL_Z - WALK_Z) / (0.08 * f)))) + 1)
    kit.m("panel", "panel").grid(lambda i, j: (BASE_X, Y(j, ny), zp[i]), lambda i, j: ((Y(j, ny) - y0) / BAY, (zp[i] - WALK_Z) / 0.75),
                                 len(zp) - 1, ny, (-1, 0, 0))
    kit.m("sill", "white_paint").quad((BASE_X, y0, SILL_Z), (BASE_X, y1, SILL_Z), (GLASS_X - 0.04, y1, GLASS_Z0), (GLASS_X - 0.04, y0, GLASS_Z0),
                                      0.1 * f, (-0.4, 0, 1), U_world(1.0, 1, 0))
    rf = kit.m("foot_road", "steel")
    rf.quad((GLASS_X + 0.04, y0, GLASS_Z0), (GLASS_X + 0.04, y1, GLASS_Z0), (FOOT_X1, y1, GLASS_Z0 - 0.05), (FOOT_X1, y0, GLASS_Z0 - 0.05),
            0.25 * f, (0.3, 0, 1), U_world(1.0, 1, 0))
    rf.quad((FOOT_X1, y0, 0.0), (FOOT_X1, y1, 0.0), (FOOT_X1, y1, GLASS_Z0 - 0.05), (FOOT_X1, y0, GLASS_Z0 - 0.05), 0.25 * f, (1, 0, 0),
            U_world(1.0, 1, 2))
    # the glass, a pane a bay; mullions; the head rail
    g = kit.m("glass", "glass")
    for b in range(3):
        ya, yb = y0 + b * BAY + 0.05, y0 + (b + 1) * BAY - 0.05
        g.quad((GLASS_X, ya, GLASS_Z0), (GLASS_X, yb, GLASS_Z0), (GLASS_X, yb, HEAD_Z), (GLASS_X, ya, HEAD_Z), 4.0, (-1, 0, 0),
               lambda p, ya=ya: ((p[1] - ya) / BAY, (HEAD_Z - p[2]) / 2.0))
    st = kit.m("steel", "steel")
    for b in range(3):
        ym = y0 + b * BAY
        st.box((GLASS_X - 0.035, ym - 0.05, SILL_Z), (GLASS_X + 0.035, ym + 0.05, HEAD_Z), 0.12 * f)
    st.box((GLASS_X - 0.07, y0, HEAD_Z), (GLASS_X + 0.07, y1, HEAD_TOP), 0.25 * f)
    # the lean-to: rafters a bay, three purlins, glass over them
    lo, hi = np.array([CANOPY_LOW[0], CANOPY_LOW[1]]), np.array([CANOPY_TOP[0], CANOPY_TOP[1]])
    run = hi - lo
    L = np.linalg.norm(run)
    along = run / L
    nrm = np.array([-along[1], along[0]])
    if nrm[1] < 0:
        nrm = -nrm
    for b in range(6):
        ym = y0 + b * BAY / 2
        o = (lo[0], ym - 0.03, lo[1])
        st.obox(o, [(along[0] * (L - 0.02), 0, along[1] * (L - 0.02)), (0, 0.06, 0), (nrm[0] * 0.12, 0, nrm[1] * 0.12)], 0.12 * f)
    for frac in (0.12, 0.5, 0.88):
        p = lo + run * frac + nrm * 0.12
        st.obox((p[0], y0, p[1]), [(along[0] * 0.05, 0, along[1] * 0.05), (0, MOD, 0), (nrm[0] * 0.05, 0, nrm[1] * 0.05)], 0.3 * f)
    p0 = lo + nrm * 0.18
    p1 = hi + nrm * 0.18
    for b in range(3):
        ya, yb = y0 + b * BAY + 0.03, y0 + (b + 1) * BAY - 0.03
        g.quad((p0[0], ya, p0[1]), (p0[0], yb, p0[1]), (p1[0], yb, p1[1]), (p1[0], ya, p1[1]), 4.0, (-nrm[0], 0, -nrm[1]),
               lambda q, ya=ya: ((q[1] - ya) / BAY, 0.0))
    # the pipes along the wall and the vault, on brackets
    pz_a = (WALL_X + 0.05, TILE_TOP + 0.14)
    tA = vault_t_at_x(-1.62)
    va = vault_point(tA)
    pz_b = (va[0] + 0.06, va[1] + 0.6)
    tb = math.pi - math.asin((3.55 - SPRING) / VB)
    vb = vault_point(tb)
    pz_b = (vb[0] + 0.07, vb[1])
    pp = kit.m("pipes", "galv", smooth=True)
    pp.tube((pz_a[0], y0, pz_a[1]), (pz_a[0], y1, pz_a[1]), 0.032, 8, 0.25 * f)
    pp.tube((pz_b[0], y0, pz_b[1]), (pz_b[0], y1, pz_b[1]), 0.026, 8, 0.25 * f)
    br = kit.m("brackets", "galv")
    for yb in (y0 + 0.75, y0 + 2.25, y0 + 3.75, y0 + 5.25):
        br.box((WALL_X, yb - 0.02, pz_a[1] - 0.05), (pz_a[0] + 0.035, yb + 0.02, pz_a[1] + 0.045), 0.1)
        br.box((vb[0] - 0.01, yb - 0.02, pz_b[1] - 0.04), (pz_b[0] + 0.03, yb + 0.02, pz_b[1] + 0.035), 0.1)
    # the walkway's bars: a 1.2 m LED bar a bay, on the sill, turned up to the walkway
    for b in range(3):
        yc = y0 + b * BAY + BAY / 2
        dead = variant == "D" and b == 1
        hs = kit.m("led_housing", "alu")
        hx, hz = BASE_X + 0.045, SILL_Z + 0.03
        dvec = np.array([-0.62, 0.0, 0.78])
        dvec /= np.linalg.norm(dvec)
        hs.obox((hx - 0.03, yc - LED_LEN / 2, hz - 0.035), [(0.06, 0, 0), (0, LED_LEN, 0), (0, 0, 0.05)], 0.1)
        face = kit.m("led_off" if dead else "led", "led_off" if dead else "led")
        fx, fz = hx - 0.032, hz + 0.0
        face.quad((fx, yc - LED_LEN / 2 + 0.01, fz - 0.012), (fx, yc + LED_LEN / 2 - 0.01, fz - 0.012),
                  (fx + 0.02, yc + LED_LEN / 2 - 0.01, fz + 0.012), (fx + 0.02, yc - LED_LEN / 2 + 0.01, fz + 0.012),
                  0.6, tuple(dvec), lambda p: (0.0, 0.0))
        if not dead:
            kit.light("led", (fx - 0.01, yc, fz + 0.01), (0.025, LED_LEN - 0.04), tuple(dvec), COOL, LED_W)
    # the road's lamps: two rows near the crown, staggered
    for (xl, yl) in ((CX - 1.7, y0 + 1.5), (CX + 1.7, y0 + 4.5)):
        tl = vault_t_at_x(xl)
        zl = vault_point(tl)[1] - 0.16
        lb = kit.m("lamp_body", "steel_dark")
        lb.box((xl - 0.18, yl - 0.36, zl), (xl + 0.18, yl + 0.36, zl + 0.13), 0.12)
        kit.m("lamp", "lamp").quad((xl - 0.15, yl - 0.32, zl - 0.002), (xl + 0.15, yl - 0.32, zl - 0.002), (xl + 0.15, yl + 0.32, zl - 0.002),
                                   (xl - 0.15, yl + 0.32, zl - 0.002), 1.0, (0, 0, -1), lambda p: (0.0, 0.0))
        # (a tunnel luminaire cuts off upward: 100 degrees, so the vault beside it is not lit
        # from below where it curves under the lamp's face)
        kit.light("lamp", (xl, yl, zl - 0.02), (0.30, 0.64), (0, 0, -1), LAMP_C, LAMP_W, spread=100.0)

    # ---- the variants' extras
    def hatch(yc):
        h = kit.m("hatch", "hatch")
        h.box((-1.50, yc - 0.32, WALK_Z), (-0.86, yc + 0.32, WALK_Z + 0.006), 0.1)
        kit.m("hatch_frame", "steel_dark").box((-1.53, yc - 0.35, WALK_Z), (-0.83, yc + 0.35, WALK_Z + 0.003), 0.1)

    def exit_sign(yc, z0=1.55):
        kit.m("plate", "steel_dark").box((WALL_X, yc - 0.19, z0 - 0.04), (WALL_X + 0.012, yc + 0.19, z0 + 0.34), 0.1)
        kit.m("sign_exit", "sign_exit").quad((WALL_X + 0.013, yc - 0.17, z0 - 0.02), (WALL_X + 0.013, yc + 0.17, z0 - 0.02),
                                             (WALL_X + 0.013, yc + 0.17, z0 + 0.32), (WALL_X + 0.013, yc - 0.17, z0 + 0.32), 0.1, (1, 0, 0),
                                             lambda p, yc=yc, z0=z0: ((p[1] - (yc - 0.17)) / 0.34, (p[2] - (z0 - 0.02)) / 0.34))

    if variant == "B":
        hatch(y0 + 1.0)
        exit_sign(y0 + 4.0)
    if variant == "D":
        hatch(y0 + 4.6)
    if variant == "C":
        yc = y0 + 2.6
        cab = kit.m("cabinet", "cabinet")
        cab.box((WALL_X, yc - 0.45, 0.55), (WALL_X + 0.07, yc + 0.45, 1.65), 0.08)
        red = kit.m("red_led", "red_led")
        fx = WALL_X + 0.072
        for (a, b2) in (((yc - 0.46, 1.66), (yc + 0.46, 1.69)), ((yc - 0.46, 0.51), (yc + 0.46, 0.54))):
            red.quad((fx, a[0], a[1]), (fx, b2[0], a[1]), (fx, b2[0], b2[1]), (fx, a[0], b2[1]), 0.2, (1, 0, 0), lambda p: (0.0, 0.0))
        for ys in (yc - 0.48, yc + 0.46):
            red.quad((fx, ys, 0.51), (fx, ys + 0.02, 0.51), (fx, ys + 0.02, 1.69), (fx, ys, 1.69), 0.2, (1, 0, 0), lambda p: (0.0, 0.0))
        kit.light("red", (fx + 0.01, yc, 1.10), (0.04, 0.9), (1, 0, 0), RED, RED_W)
        kit.m("red_lamp", "red_led").box((WALL_X, yc - 0.07, 1.86), (WALL_X + 0.09, yc + 0.07, 2.0), 0.1)
        kit.light("red", (WALL_X + 0.1, yc, 1.93), (0.1, 0.12), (1, 0, 0), RED, RED_W * 0.6)
        exit_sign(yc - 1.6, 1.62)
        # the info plate (its words are the run time's)
        yi = y0 + 4.6
        kit.m("plate", "steel_dark").box((WALL_X, yi - 0.18, 1.06), (WALL_X + 0.012, yi + 0.18, 1.74), 0.1)
        kit.m("sign_info", "sign_info").quad((WALL_X + 0.013, yi - 0.16, 1.08), (WALL_X + 0.013, yi + 0.16, 1.08), (WALL_X + 0.013, yi + 0.16, 1.72),
                                             (WALL_X + 0.013, yi - 0.16, 1.72), 0.1, (1, 0, 0),
                                             lambda p, yi=yi: ((p[1] - (yi - 0.16)) / 0.32, (p[2] - 1.08) / 0.64))
        pp.tube((WALL_X + 0.05, yi + 0.6, WALK_Z), (WALL_X + 0.05, yi + 0.6, pz_a[1]), 0.03, 8, 0.2)
    if variant in ("F", "G"):
        escape_door(kit, y0 + DOOR_Y, variant == "G", exit_sign)
    if variant == "E":
        for xf in (CX - 1.55, CX + 1.55):
            zf = 6.35
            fan = kit.m("fan", "galv", smooth=True)
            fan.tube((xf, y0 + 0.9, zf), (xf, y0 + 5.1, zf), 0.56, 20, 0.3)
            kit.m("fan_in", "black").tube((xf, y0 + 0.92, zf), (xf, y0 + 5.08, zf), 0.53, 20, 0.6)
            for yh in (y0 + 1.6, y0 + 4.4):
                ztop = vault_point(vault_t_at_x(xf))[1]
                kit.m("hanger", "steel_dark").box((xf - 0.05, yh - 0.05, zf + 0.5), (xf + 0.05, yh + 0.05, ztop), 0.2)


def escape_door(kit, yc, changed, exit_sign):
    """The escape door in the tiled wall (the way in): a steel leaf set back
    in a steel frame, the escape point's plate and the way-out sign beside
    it. Changed (G), it is an older door standing a hand's width open on a
    concrete passage with a cold light deep in it, and her note is taped to
    the tiles by it."""
    ya, yb = yc - DOOR_W / 2, yc + DOOR_W / 2
    z0, z1 = WALK_Z, DOOR_TOP
    xb = WALL_X - DOOR_DEPTH - 0.04          # the frame's back, the passage's mouth
    fr = kit.m("door_frame", "door_frame_old" if changed else "steel_dark")
    fr.box((xb, ya - DOOR_FRAME, z0), (WALL_X + 0.015, ya, TILE_TOP), 0.1)
    fr.box((xb, yb, z0), (WALL_X + 0.015, yb + DOOR_FRAME, TILE_TOP), 0.1)
    fr.box((xb, ya, z1), (WALL_X + 0.015, yb, TILE_TOP), 0.1)
    xl = WALL_X - DOOR_DEPTH                  # the leaf's face, shut
    if not changed:
        kit.m("door", "door").box((xl - 0.045, ya + 0.003, z0 + 0.01), (xl, yb - 0.003, z1 - 0.003), 0.08)
        kit.m("door_handle", "galv").box((xl, yb - 0.17, 1.04), (xl + 0.05, yb - 0.06, 1.065), 0.05)
        kit.m("door_handle", "galv").box((xl, yb - 0.10, 1.00), (xl + 0.02, yb - 0.06, 1.10), 0.05)
    else:
        # hinged at yb (the far side as you walk in), swung in: its latch
        # edge a hand's width off the near jamb, the gap facing you as you come
        a = math.radians(DOOR_OPEN)
        along = np.array([-math.sin(a), -math.cos(a), 0.0]) * (DOOR_W - 0.006)
        face = np.array([math.cos(a), -math.sin(a), 0.0])    # the leaf's face, toward the walkway
        o = np.array([xl, yb - 0.003, z0 + 0.01])
        kit.m("door", "door_old").obox(o, [along, -face * 0.045, (0, 0, z1 - z0 - 0.013)], 0.08)
        h = o + along * 0.86 + np.array([0, 0, 1.04 - o[2]])
        kit.m("door_handle", "door_frame_old").obox(h, [along * 0.12, face * 0.05, (0, 0, 0.025)], 0.05)
        # the passage behind: concrete, going in, lit from deep inside
        pa, pb = ya - 0.25, yb + 0.25
        xe = xb - PASSAGE
        zt = z0 + 2.4
        con = kit.m("passage", "concrete")
        uvw = lambda p: (p[0] / 2.0 + p[1] / 2.0, p[2] / 2.0)
        con.quad((xb, pa, z0), (xe, pa, z0), (xe, pb, z0), (xb, pb, z0), 0.15, (0, 0, 1), lambda p: (p[0] / 2.0, p[1] / 2.0))
        con.quad((xb, pa, zt), (xe, pa, zt), (xe, pb, zt), (xb, pb, zt), 0.2, (0, 0, -1), lambda p: (p[0] / 2.0, p[1] / 2.0))
        con.quad((xb, pa, z0), (xe, pa, z0), (xe, pa, zt), (xb, pa, zt), 0.15, (0, 1, 0), uvw)
        con.quad((xb, pb, z0), (xe, pb, z0), (xe, pb, zt), (xb, pb, zt), 0.15, (0, -1, 0), uvw)
        con.quad((xe, pa, z0), (xe, pb, z0), (xe, pb, zt), (xe, pa, zt), 0.2, (1, 0, 0), uvw)
        # the wall round the opening, seen from inside (the frame's back)
        con.quad((xb, pa, z0), (xb, ya - DOOR_FRAME, z0), (xb, ya - DOOR_FRAME, zt), (xb, pa, zt), 0.2, (-1, 0, 0), uvw)
        con.quad((xb, yb + DOOR_FRAME, z0), (xb, pb, z0), (xb, pb, zt), (xb, yb + DOOR_FRAME, zt), 0.2, (-1, 0, 0), uvw)
        con.quad((xb, ya - DOOR_FRAME, TILE_TOP), (xb, yb + DOOR_FRAME, TILE_TOP), (xb, yb + DOOR_FRAME, zt), (xb, ya - DOOR_FRAME, zt), 0.2,
                 (-1, 0, 0), uvw)
        kit.light("cold", (xe + 0.4, yc, zt - 0.05), (0.5, 0.5), (0, 0, -1), COLD, COLD_W)
        # her note, taped to the tiles beside the door: a sheet folded twice
        yn = yc - 0.80
        nw, nh, nz = 0.21, 0.297, 1.32
        kit.m("note", "note").quad((WALL_X + 0.004, yn - nw / 2, nz), (WALL_X + 0.004, yn + nw / 2, nz), (WALL_X + 0.004, yn + nw / 2, nz + nh),
                                   (WALL_X + 0.004, yn - nw / 2, nz + nh), 0.05, (1, 0, 0),
                                   lambda p, yn=yn: ((p[1] - (yn - nw / 2)) / nw, (p[2] - nz) / nh))
        for yt in (yn - nw / 2 + 0.02, yn + nw / 2 - 0.02):
            kit.m("tape", "tape").box((WALL_X + 0.004, yt - 0.025, nz + nh - 0.03), (WALL_X + 0.006, yt + 0.025, nz + nh + 0.02), 0.05)
    # the escape point's plate (its words are the run time's), the way-out sign
    yi = yc - 1.35
    kit.m("plate", "steel_dark").box((WALL_X, yi - 0.18, 1.06), (WALL_X + 0.012, yi + 0.18, 1.74), 0.1)
    kit.m("sign_info", "sign_info").quad((WALL_X + 0.013, yi - 0.16, 1.08), (WALL_X + 0.013, yi + 0.16, 1.08), (WALL_X + 0.013, yi + 0.16, 1.72),
                                         (WALL_X + 0.013, yi - 0.16, 1.72), 0.1, (1, 0, 0),
                                         lambda p, yi=yi: ((p[1] - (yi - 0.16)) / 0.32, (p[2] - 1.08) / 0.64))
    exit_sign(yc + 0.95, 1.55)


def mirror_kit(kit):
    """The other bore: this one mirrored about x = MIRROR / 2 (faces turned
    back the right way)."""
    out = Kit()
    for k, m in kit.meshes.items():
        n = out.m(*k)
        base = len(n.co)
        n.co += [(MIRROR - x, y, z) for (x, y, z) in m.co]
        n.faces += [[i + base for i in reversed(f)] for f in m.faces]
        n.uvs += [list(reversed(u)) for u in m.uvs]
    for li in kit.lights:
        d = dict(li)
        d["centre"] = (MIRROR - li["centre"][0], li["centre"][1], li["centre"][2])
        d["dir"] = (-li["dir"][0], li["dir"][1], li["dir"][2])
        out.lights.append(d)
    return out


def add(kit, other):
    for k, m in other.meshes.items():
        n = kit.m(*k)
        base = len(n.co)
        n.co += m.co
        n.faces += [[i + base for i in f] for f in m.faces]
        n.uvs += m.uvs
    kit.lights += other.lights


# ----------------------------------------------------------- outside ---

def street_lamp(kit, x, y, ground, arm_dx, height=9.0):
    """A pole and a head reaching `arm_dx` over the road (one or two heads)."""
    pole = kit.m("poles", "steel", smooth=True)
    pole.tube((x, y, ground), (x, y, ground + height), 0.09, 10, 0.5)
    for dx in arm_dx:
        hx = x + dx
        arm = kit.m("poles", "steel", smooth=True)
        arm.tube((x, y, ground + height - 0.1), (hx, y, ground + height + 0.25), 0.045, 8, 0.4)
        kit.m("lamp_heads", "steel_dark").box((hx - 0.34, y - 0.15, ground + height + 0.12), (hx + 0.34, y + 0.15, ground + height + 0.3), 0.15)
        kit.m("street_lamp", "street_lamp").quad((hx - 0.3, y - 0.12, ground + height + 0.118), (hx + 0.3, y - 0.12, ground + height + 0.118),
                                                 (hx + 0.3, y + 0.12, ground + height + 0.118), (hx - 0.3, y + 0.12, ground + height + 0.118), 1.0,
                                                 (0, 0, -1), lambda p: (0.0, 0.0))
        kit.light("street", (hx, y, ground + height + 0.1), (0.6, 0.24), (0, 0, -1), WARM, STREET_W, axis=(1, 0, 0), spread=110.0)


def railing(kit, pts, height, post_every=1.5, mat="steel_dark"):
    """A steel railing along a polyline of (x, y, z) feet: posts, a top rail
    and a mid rail."""
    P = np.array(pts, np.float64)
    rail = kit.m("railings", mat, smooth=True)
    for a in range(len(P) - 1):
        p, q = P[a], P[a + 1]
        L = np.linalg.norm(q - p)
        n = max(1, int(round(L / post_every)))
        for k in range(n + (1 if a == len(P) - 2 else 0)):
            f = p + (q - p) * k / n
            rail.tube(f, f + np.array([0, 0, height]), 0.022, 6, 0.6)
        rail.tube(p + np.array([0, 0, height]), q + np.array([0, 0, height]), 0.025, 8, 0.6)
        rail.tube(p + np.array([0, 0, height * 0.5]), q + np.array([0, 0, height * 0.5]), 0.015, 6, 0.6)


def portal_wall(kit, y, x0, x1, top, ground, openings, step=0.25):
    """A facade at y (facing -y) from x0 to x1, up to `top`, with arch
    openings: columns between the ground (or an opening's top) and `top`."""
    xs = set(np.arange(x0, x1 + 1e-6, step).round(4))
    for (ox0, ox1, prof) in openings:
        xs |= {round(ox0 - 0.001, 4), round(ox0, 4), round(ox1, 4), round(ox1 + 0.001, 4)}
        for (px, pz) in prof:
            xs.add(round(px, 4))
    xs = sorted(x for x in xs if x0 - 1e-6 <= x <= x1 + 1e-6)

    def bottom(x):
        for (ox0, ox1, prof) in openings:
            if ox0 <= x <= ox1:
                P = np.array(prof)
                return float(np.interp(x, P[:, 0], P[:, 1]))
        return ground(x)

    nj = max(2, int(math.ceil((top - 0.2) / step)))
    m = kit.m("facade", "stone")

    def P(i, j):
        b = bottom(xs[i])
        return (xs[i], y, b + (top - b) * j / nj)

    m.grid(P, lambda i, j: (P(i, j)[0] / 4.0, P(i, j)[2] / 4.0), len(xs) - 1, nj, (0, -1, 0))


def arch_profile(left, right, step=0.15):
    """The opening of a bore at the portal: up the wall from the walkway,
    over the vault, down the far wall (as (x, z) along x for the facade)."""
    t = np.linspace(math.pi, 0.0, 200)
    pts = [(left, WALK_Z)] + [(CX + VA * math.cos(a) + (left - WALL_X), SPRING + VB * math.sin(a)) for a in t] + [(right, CURB_Z)]
    return pts


def ring(kit, prof, y, width=0.6, proud=0.12):
    """The arch's projecting ring: a band round the opening."""
    P = np.array(prof, np.float64)
    P, _ = resample(P, 0.15)
    m = kit.m("ring", "concrete")
    n = len(P)
    # outward normals of the curve (away from the opening's centre)
    c = np.array([P[:, 0].mean(), SPRING])
    outs = []
    for i in range(n):
        a, b = P[max(0, i - 1)], P[min(n - 1, i + 1)]
        tng = b - a
        nn = np.array([-tng[1], tng[0]])
        nn /= max(np.linalg.norm(nn), 1e-9)
        if np.dot(nn, P[i] - c) < 0:
            nn = -nn
        outs.append(nn)
    outs = np.array(outs)
    inner = P
    outer = P + outs * width
    yv = [y - proud, y]
    for i in range(n - 1):
        a0 = m.v((inner[i][0], y - proud, inner[i][1]))
        a1 = m.v((inner[i + 1][0], y - proud, inner[i + 1][1]))
        b1 = m.v((outer[i + 1][0], y - proud, outer[i + 1][1]))
        b0 = m.v((outer[i][0], y - proud, outer[i][1]))
        m.face([a0, a1, b1, b0], [(inner[i][0] / 2, inner[i][1] / 2), (inner[i + 1][0] / 2, inner[i + 1][1] / 2),
                                  (outer[i + 1][0] / 2, outer[i + 1][1] / 2), (outer[i][0] / 2, outer[i][1] / 2)], (0, -1, 0))
        for (E, sgn) in ((inner, -1), (outer, 1)):
            e0 = m.v((E[i][0], y - proud, E[i][1]))
            e1 = m.v((E[i + 1][0], y - proud, E[i + 1][1]))
            f1 = m.v((E[i + 1][0], y, E[i + 1][1]))
            f0 = m.v((E[i][0], y, E[i][1]))
            w = (outs[i][0] * sgn, 0, outs[i][1] * sgn)
            m.face([e0, e1, f1, f0], [(i * 0.15 / 2, 0), ((i + 1) * 0.15 / 2, 0), ((i + 1) * 0.15 / 2, 0.06), (i * 0.15 / 2, 0.06)], w)


STAIR_X0, STAIR_X1 = LEFT_WALL, LEFT_WALL + 1.5
STAIR_TOP_Y = -21.2   # the top landing's far end
STAIRS = [  # (y from, y to, z from, z to, risers): descending toward the portal
    (-19.2, -13.4, UPPER_Z, 3.7, 20),
    (-11.8, -6.0, 3.7, WALK_Z, 20),
]


def stair_steps():
    """Every tread as (y0, y1, z) along the stairs, landings included."""
    out = [(STAIR_TOP_Y, STAIRS[0][0], UPPER_Z)]
    for i, (ya, yb, za, zb, n) in enumerate(STAIRS):
        run = (yb - ya) / n
        rise = (za - zb) / n
        for k in range(n):
            out.append((ya + run * k, ya + run * (k + 1), za - rise * (k + 1)))
        if i == 0:
            out.append((yb, STAIRS[1][0], zb))
    return out


def exterior(kit):
    """The approach at night: the road out of the twin portals, the median,
    the red walkway and its plaza behind a glass balustrade, the left
    retaining wall with its long stairs to the street above, the right
    retaining wall, the portal's stone face, the street lamps."""
    Y0 = -80.0
    # roads, the median, the far walkway
    for (xa, xb) in ((FOOT_X1, ROAD_X1), (MIRROR - ROAD_X1, MIRROR - FOOT_X1)):
        xr = np.linspace(xa, xb, int((xb - xa) / 0.5) + 1)
        yr = np.linspace(Y0, 0.0, int(-Y0 / 0.5) + 1)
        kit.m("road_out", "road").grid(lambda i, j, xr=xr, yr=yr: (xr[i], yr[j], 0.0), lambda i, j, xr=xr, yr=yr: (xr[i] / 3.0, yr[j] / 3.0),
                                       len(xr) - 1, len(yr) - 1, (0, 0, 1))
        ln = kit.m("lines_out", "white_paint")
        for xa2 in (xa + 0.28, xb - 0.45):
            ln.quad((xa2, Y0, 0.004), (xa2, 0.0, 0.004), (xa2 + 0.15, 0.0, 0.004), (xa2 + 0.15, Y0, 0.004), 0.5, (0, 0, 1), U_world(1.0, 1, 0))
        xc = (xa + xb) / 2 - 0.075
        for yd in np.arange(Y0, 0.0, 6.0):
            ln.quad((xc, yd + 0.5, 0.004), (xc, yd + 3.5, 0.004), (xc + 0.15, yd + 3.5, 0.004), (xc + 0.15, yd + 0.5, 0.004), 0.5, (0, 0, 1),
                    U_world(1.0, 1, 0))
    med = kit.m("median", "concrete")
    med.quad((ROAD_X1, Y0, CURB_Z), (ROAD_X1, 0.0, CURB_Z), (MIRROR - ROAD_X1, 0.0, CURB_Z), (MIRROR - ROAD_X1, Y0, CURB_Z), 0.5, (0, 0, 1),
             U_world(2.0, 0, 1))
    med.quad((ROAD_X1, Y0, 0), (ROAD_X1, 0.0, 0), (ROAD_X1, 0.0, CURB_Z), (ROAD_X1, Y0, CURB_Z), 0.5, (-1, 0, 0), U_world(2.0, 1, 2))
    med.quad((MIRROR - ROAD_X1, Y0, 0), (MIRROR - ROAD_X1, 0.0, 0), (MIRROR - ROAD_X1, 0.0, CURB_Z), (MIRROR - ROAD_X1, Y0, CURB_Z), 0.5,
             (1, 0, 0), U_world(2.0, 1, 2))
    # a concrete barrier down the median (a safety-shape profile)
    bx = MIRROR / 2
    prof = [(-0.30, CURB_Z), (-0.22, CURB_Z + 0.08), (-0.09, CURB_Z + 0.33), (-0.075, CURB_Z + 0.81), (0.075, CURB_Z + 0.81),
            (0.09, CURB_Z + 0.33), (0.22, CURB_Z + 0.08), (0.30, CURB_Z)]
    yb = np.linspace(Y0, 0.0, int(-Y0 / 0.5) + 1)
    kit.m("barrier", "concrete").grid(lambda i, j: (bx + prof[i][0], yb[j], prof[i][1]), lambda i, j: (i * 0.2, yb[j] / 2.0), len(prof) - 1,
                                      len(yb) - 1, lambda c: (c[0] - bx, 0, 0.3))
    # the walkways outside: ours widens into the plaza; the other bore's stays narrow
    pl = kit.m("plaza", "plaza")
    pl.quad((LEFT_WALL, Y0, WALK_Z), (FOOT_X1, Y0, WALK_Z), (FOOT_X1, 0.0, WALK_Z), (LEFT_WALL, 0.0, WALK_Z), 0.25, (0, 0, 1), U_world(2.0, 0, 1))
    pl.quad((MIRROR - FOOT_X1, Y0, WALK_Z), (RIGHT_WALL, Y0, WALK_Z), (RIGHT_WALL, 0.0, WALK_Z), (MIRROR - FOOT_X1, 0.0, WALK_Z), 0.3, (0, 0, 1),
            U_world(2.0, 0, 1))
    cu = kit.m("curbs", "concrete")
    cu.quad((FOOT_X1, Y0, 0), (FOOT_X1, 0.0, 0), (FOOT_X1, 0.0, WALK_Z), (FOOT_X1, Y0, WALK_Z), 0.5, (1, 0, 0), U_world(1.0, 1, 2))
    cu.quad((MIRROR - FOOT_X1, Y0, 0), (MIRROR - FOOT_X1, 0.0, 0), (MIRROR - FOOT_X1, 0.0, WALK_Z), (MIRROR - FOOT_X1, Y0, WALK_Z), 0.5,
            (-1, 0, 0), U_world(1.0, 1, 2))
    # the glass balustrades: posts, panes, an orange top rail, an LED strip under it
    for (xg, side) in ((0.10, 1), (MIRROR - 0.10, -1)):
        ys = np.arange(Y0, 0.01, 1.6)
        for yp in ys:
            kit.m("balustrade_posts", "steel_dark").box((xg - 0.03, yp - 0.025, WALK_Z), (xg + 0.03, yp + 0.025, WALK_Z + 1.12), 0.12)
        for a in range(len(ys) - 1):
            kit.m("glass_out", "glass").quad((xg, ys[a] + 0.04, WALK_Z + 0.12), (xg, ys[a + 1] - 0.04, WALK_Z + 0.12),
                                             (xg, ys[a + 1] - 0.04, WALK_Z + 1.02), (xg, ys[a] + 0.04, WALK_Z + 1.02), 4.0, (-side, 0, 0),
                                             lambda p: (0.0, 0.0))
        kit.m("top_rail", "rail_orange", smooth=True).tube((xg, Y0, WALK_Z + 1.16), (xg, 0.0, WALK_Z + 1.16), 0.045, 10, 0.5)
        strip = kit.m("rail_led", "rail_led")
        strip.quad((xg - 0.012 * side, Y0, WALK_Z + 1.105), (xg - 0.012 * side, 0.0, WALK_Z + 1.105), (xg + 0.012 * side, 0.0, WALK_Z + 1.105),
                   (xg + 0.012 * side, Y0, WALK_Z + 1.105), 1.0, (0, 0, -1), lambda p: (0.0, 0.0))
        for yl in np.arange(Y0 + 1.5, 0.0, 3.0):
            kit.light("rail", (xg, yl, WALK_Z + 1.095), (0.03, 2.9), (-0.25 * side, 0, -1), COOL, RAIL_W)
    # the left retaining wall, its coping and railing (a gap at the stairs' head)
    zs = np.linspace(WALK_Z, UPPER_Z, int((UPPER_Z - WALK_Z) / 0.3) + 1)
    yw = np.linspace(Y0, 0.0, int(-Y0 / 0.3) + 1)
    kit.m("wall_left", "stone").grid(lambda i, j: (LEFT_WALL, yw[j], zs[i]), lambda i, j: (yw[j] / 4.0, zs[i] / 4.0), len(zs) - 1, len(yw) - 1,
                                     (1, 0, 0))
    cop = kit.m("coping", "concrete")
    cop.box((LEFT_WALL - 0.3, Y0, UPPER_Z), (LEFT_WALL + 0.05, STAIR_TOP_Y, UPPER_Z + 0.25), 0.4)
    cop.box((LEFT_WALL - 0.3, STAIRS[0][0], UPPER_Z), (LEFT_WALL + 0.05, 0.0, UPPER_Z + 0.25), 0.4)
    railing(kit, [(LEFT_WALL - 0.12, Y0, UPPER_Z + 0.25), (LEFT_WALL - 0.12, STAIR_TOP_Y, UPPER_Z + 0.25)], 1.0)
    railing(kit, [(LEFT_WALL - 0.12, STAIRS[0][0], UPPER_Z + 0.25), (LEFT_WALL - 0.12, -1.0, UPPER_Z + 0.25)], 1.0)
    # the right retaining wall
    zs2 = np.linspace(WALK_Z, RIGHT_TOP, int((RIGHT_TOP - WALK_Z) / 0.3) + 1)
    kit.m("wall_right", "stone").grid(lambda i, j: (RIGHT_WALL, yw[j], zs2[i]), lambda i, j: (-yw[j] / 4.0, zs2[i] / 4.0), len(zs2) - 1,
                                      len(yw) - 1, (-1, 0, 0))
    kit.m("coping", "concrete").box((RIGHT_WALL - 0.05, Y0, RIGHT_TOP), (RIGHT_WALL + 0.3, 0.0, RIGHT_TOP + 0.25), 0.4)
    railing(kit, [(RIGHT_WALL + 0.12, Y0, RIGHT_TOP + 0.25), (RIGHT_WALL + 0.12, -1.0, RIGHT_TOP + 0.25)], 1.0)
    # the street above the left wall: a pavement, the road, the houses' dark face with a few lit windows
    up = kit.m("upper", "concrete")
    up.quad((-8.2, Y0, UPPER_Z), (LEFT_WALL - 0.3, Y0, UPPER_Z), (LEFT_WALL - 0.3, -1.0, UPPER_Z), (-8.2, -1.0, UPPER_Z), 0.5, (0, 0, 1),
            U_world(2.0, 0, 1))
    kit.m("upper_road", "road").quad((-17.0, Y0, UPPER_Z - 0.12), (-8.2, Y0, UPPER_Z - 0.12), (-8.2, -1.0, UPPER_Z - 0.12),
                                     (-17.0, -1.0, UPPER_Z - 0.12), 0.6, (0, 0, 1), U_world(3.0, 0, 1))
    kit.m("upper", "concrete").quad((-8.2, Y0, UPPER_Z - 0.12), (-8.2, -1.0, UPPER_Z - 0.12), (-8.2, -1.0, UPPER_Z), (-8.2, Y0, UPPER_Z), 0.5,
                                    (-1, 0, 0), U_world(1.0, 1, 2))
    rng = np.random.default_rng(7)
    hx = -17.0
    for (ya, yb2, h) in ((-80.0, -61.0, 10.5), (-61.0, -47.0, 13.0), (-47.0, -30.0, 9.0), (-30.0, -14.0, 12.0), (-14.0, -1.0, 8.5)):
        kit.m("houses", "concrete").quad((hx, ya, UPPER_Z - 0.12), (hx, yb2, UPPER_Z - 0.12), (hx, yb2, UPPER_Z + h), (hx, ya, UPPER_Z + h), 0.6,
                                         (1, 0, 0), U_world(3.0, 1, 2))
        kit.m("roofs", "steel_dark").quad((hx - 6, ya, UPPER_Z + h), (hx, ya, UPPER_Z + h), (hx, yb2, UPPER_Z + h), (hx - 6, yb2, UPPER_Z + h),
                                          1.0, (0, 0, 1), U_world(3.0, 0, 1))
        for fl in range(int(h // 3.0)):
            for yw2 in np.arange(ya + 1.5, yb2 - 1.0, 3.2):
                lit = rng.random() < 0.18
                mat = "window_lit" if lit else "window_dark"
                z0 = UPPER_Z + 1.2 + fl * 3.0
                kit.m("windows", mat).quad((hx + 0.02, yw2, z0), (hx + 0.02, yw2 + 1.6, z0), (hx + 0.02, yw2 + 1.6, z0 + 1.3),
                                           (hx + 0.02, yw2, z0 + 1.3), 2.0, (1, 0, 0), lambda p: (0.0, 0.0))
                if lit:
                    kit.light("window", (hx + 0.05, yw2 + 0.8, z0 + 0.65), (1.3, 1.6), (1, 0, 0), WINDOW, WINDOW_W, axis=(0, 0, 1))
    # the stairs: treads with dark nosings, risers, their stone side, a railing, a handrail on the wall
    tr = kit.m("treads", "concrete")
    no = kit.m("nosings", "steel_dark")
    sd = kit.m("stair_side", "stone")
    steps = stair_steps()
    prev_z = None
    for (ya, yb2, z) in steps:
        tr.quad((STAIR_X0, ya, z), (STAIR_X1, ya, z), (STAIR_X1, yb2, z), (STAIR_X0, yb2, z), 0.25, (0, 0, 1), U_world(1.0, 0, 1))
        if abs(yb2 - ya) < 0.5:
            no.quad((STAIR_X0, yb2 - 0.05, z + 0.002), (STAIR_X1, yb2 - 0.05, z + 0.002), (STAIR_X1, yb2, z + 0.002), (STAIR_X0, yb2, z + 0.002),
                    0.5, (0, 0, 1), U_world(1.0, 0, 1))
        sd.quad((STAIR_X1, ya, WALK_Z), (STAIR_X1, yb2, WALK_Z), (STAIR_X1, yb2, z), (STAIR_X1, ya, z), 0.3, (1, 0, 0),
                lambda p: (-p[1] / 4.0, p[2] / 4.0))
        if prev_z is not None and abs(prev_z - z) > 1e-6:
            tr.quad((STAIR_X0, ya, z), (STAIR_X1, ya, z), (STAIR_X1, ya, prev_z), (STAIR_X0, ya, prev_z), 0.25, (0, -1, 0) if z > prev_z else (0, 1, 0),
                    U_world(1.0, 0, 2))
        prev_z = z
    # the stairs' end face at the bottom and the top landing's end
    last = steps[-1]
    rail_pts = [(STAIR_X1 - 0.06, STAIR_TOP_Y, UPPER_Z), (STAIR_X1 - 0.06, STAIRS[0][0], UPPER_Z), (STAIR_X1 - 0.06, STAIRS[0][1], 3.7),
                (STAIR_X1 - 0.06, STAIRS[1][0], 3.7), (STAIR_X1 - 0.06, STAIRS[1][1], WALK_Z)]
    railing(kit, rail_pts, 1.0, mat="steel")
    hand = kit.m("handrail", "steel", smooth=True)
    for (ya, yb2, za, zb, n) in STAIRS:
        hand.tube((STAIR_X0 + 0.07, ya, za + 0.9), (STAIR_X0 + 0.07, yb2, zb + 0.9), 0.022, 8, 0.5)
    # the portal: the stone face with its two arches, their rings, the
    # name plate, the lane signals, the cornice
    op1 = (WALL_X, RIGHT_X, arch_profile(WALL_X, RIGHT_X))
    left2, right2 = MIRROR - RIGHT_X, MIRROR - WALL_X
    prof2 = [(MIRROR - x, z) for (x, z) in reversed(arch_profile(WALL_X, RIGHT_X))]
    op2 = (left2, right2, prof2)

    def ground(x):
        if x < LEFT_WALL:
            return UPPER_Z
        if x > RIGHT_WALL:
            return RIGHT_TOP
        if x < FOOT_X1 or x > MIRROR - FOOT_X1:
            return WALK_Z
        if ROAD_X1 < x < MIRROR - ROAD_X1:
            return CURB_Z
        return 0.0

    portal_wall(kit, 0.0, -17.0, 26.0, 11.0, ground, [op1, op2])
    ring(kit, op1[2], 0.0)
    ring(kit, op2[2], 0.0)
    kit.m("cornice", "concrete").box((-17.0, -0.35, 10.75), (26.0, 0.0, 11.05), 0.5)
    railing(kit, [(-17.0, -0.2, 11.05), (26.0, -0.2, 11.05)], 1.0)
    kit.m("plaque", "plaque").box((1.1, -0.06, 9.05), (6.3, 0.0, 9.95), 0.2)
    kit.m("sign_name", "sign_name").quad((1.2, -0.061, 9.12), (6.2, -0.061, 9.12), (6.2, -0.061, 9.88), (1.2, -0.061, 9.88), 0.2, (0, -1, 0),
                                         lambda p: ((p[0] - 1.2) / 5.0, (p[2] - 9.12) / 0.76))
    for (xs, mat) in ((2.45, "signal_green"), (6.35, "signal_green"), (MIRROR - 2.45, "signal_back"), (MIRROR - 6.35, "signal_back")):
        kit.m("signals", "steel_dark").box((xs - 0.32, -0.32, 8.35), (xs + 0.32, -0.02, 8.95), 0.15)
        if mat == "signal_green":
            kit.m("signal", "signal_green").quad((xs - 0.25, -0.322, 8.42), (xs + 0.25, -0.322, 8.42), (xs + 0.25, -0.322, 8.88),
                                                 (xs - 0.25, -0.322, 8.88), 0.25, (0, -1, 0),
                                                 lambda p, xs=xs: ((p[0] - (xs - 0.25)) / 0.5, (p[2] - 8.42) / 0.46))
            kit.light("signal", (xs, -0.34, 8.65), (0.4, 0.4), (0, -1, 0), GREEN, SIGNAL_W, axis=(1, 0, 0))
    # street lamps: the median (two heads), above the left wall (over the stairs), above the right wall
    street_things(kit)
    for yl in (-12.0, -38.0, -64.0):
        street_lamp(kit, MIRROR / 2, yl, CURB_Z + 0.81, (-3.6, 3.6))
    for yl in (-17.5, -9.0, -36.0, -60.0):  # the first two over the stairs' flights
        street_lamp(kit, LEFT_WALL - 0.6, yl, UPPER_Z + 0.25, (1.9,), height=8.0)
    for yl in (-24.0, -50.0, -76.0):
        street_lamp(kit, RIGHT_WALL + 0.6, yl, RIGHT_TOP + 0.25, (-1.9,), height=8.0)


def street_things(kit):
    """What stands about outside (the photographs): delineator posts along
    the median's nose, manhole covers, a utility cabinet at the stairs' foot."""
    for x in (ROAD_X1 + 0.35, MIRROR - ROAD_X1 - 0.35):
        for y in np.arange(-1.5, -16.0, -2.0):
            kit.m("posts", "rail_orange", smooth=True).tube((x, y, CURB_Z), (x, y, CURB_Z + 0.78), 0.04, 10, 0.2)
            for zb in (0.52, 0.66):
                kit.m("post_bands", "white_paint", smooth=True).tube((x, y, CURB_Z + zb), (x, y, CURB_Z + zb + 0.06), 0.042, 10, 0.1)
    for (cx, cy, cz) in ((2.4, -22.0, 0.0), (6.1, -47.0, 0.0), (-2.2, -27.0, WALK_Z)):
        n = 20
        cover = kit.m("manholes", "hatch")
        ids = [cover.v((cx + 0.33 * math.cos(2 * math.pi * k / n), cy + 0.33 * math.sin(2 * math.pi * k / n), cz + 0.006)) for k in range(n)]
        cover.face(ids, [(0.5 + 0.5 * math.cos(2 * math.pi * k / n), 0.5 + 0.5 * math.sin(2 * math.pi * k / n)) for k in range(n)], (0, 0, 1))
    kit.m("cabinet_out", "steel").box((LEFT_WALL, -4.6, WALK_Z), (LEFT_WALL + 0.42, -3.7, WALK_Z + 1.25), 0.15)
    kit.m("cabinet_out", "steel_dark").box((LEFT_WALL + 0.42, -4.5, WALK_Z + 1.0), (LEFT_WALL + 0.43, -3.8, WALK_Z + 1.1), 0.1)


def far_end(kit):
    """The far end of the walkway: a locked gate across it (the far portal
    is shut at night), the dark beyond."""
    yg = TUNNEL - 1.2
    g = kit.m("gate", "steel_dark", smooth=True)
    for x in np.arange(WALL_X + 0.08, BASE_X - 0.02, 0.12):
        g.tube((x, yg, WALK_Z), (x, yg, WALK_Z + 2.1), 0.014, 6, 0.4)
    gb = kit.m("gate_frame", "steel_dark")
    for z in (WALK_Z + 0.1, WALK_Z + 1.05, WALK_Z + 2.08):
        gb.box((WALL_X, yg - 0.03, z - 0.03), (BASE_X, yg + 0.03, z + 0.03), 0.1)
    gb.box((WALL_X, yg - 0.05, WALK_Z), (WALL_X + 0.06, yg + 0.05, WALK_Z + 2.15), 0.1)
    gb.box((BASE_X - 0.06, yg - 0.05, WALK_Z), (BASE_X, yg + 0.05, WALK_Z + 2.15), 0.1)
    kit.m("sign_gate", "sign_gate").quad((-1.15, yg - 0.04, 1.30), (-0.55, yg - 0.04, 1.30), (-0.55, yg - 0.04, 1.70), (-1.15, yg - 0.04, 1.70),
                                         0.1, (0, -1, 0), lambda p: ((p[0] + 1.15) / 0.6, (p[2] - 1.30) / 0.40))
    kit.m("padlock", "galv").box((-0.95, yg - 0.07, 1.12), (-0.88, yg - 0.03, 1.2), 0.05)
    # beyond the far portal: the road on into the night, two street lamps
    Y1 = TUNNEL + 45.0
    kit.m("road_out", "road").quad((FOOT_X1, TUNNEL, 0.0), (ROAD_X1, TUNNEL, 0.0), (ROAD_X1, Y1, 0.0), (FOOT_X1, Y1, 0.0), 0.6, (0, 0, 1),
                                   U_world(3.0, 0, 1))
    kit.m("plaza", "plaza").quad((WALL_X - 3.0, TUNNEL, WALK_Z), (FOOT_X1, TUNNEL, WALK_Z), (FOOT_X1, Y1, WALK_Z), (WALL_X - 3.0, Y1, WALK_Z), 0.6,
                                 (0, 0, 1), U_world(2.0, 0, 1))
    kit.m("median", "concrete").quad((ROAD_X1, TUNNEL, CURB_Z), (ROAD_X1 + 3.0, TUNNEL, CURB_Z), (ROAD_X1 + 3.0, Y1, CURB_Z), (ROAD_X1, Y1, CURB_Z),
                                     0.6, (0, 0, 1), U_world(2.0, 0, 1))
    for yl in (TUNNEL + 14.0, TUNNEL + 38.0):
        street_lamp(kit, ROAD_X1 + 1.2, yl, CURB_Z, (-3.4,))
    # the far portal's face (seen only as the dark round the opening)
    portal_wall(kit, TUNNEL, -6.0, 14.0, 11.0, lambda x: WALK_Z, [(WALL_X, RIGHT_X, arch_profile(WALL_X, RIGHT_X))])
    for k, m in list(kit.meshes.items()):
        if k[0] == "facade":
            for f in m.faces:
                f.reverse()
            for u in m.uvs:
                u.reverse()


# --------------------------------------------------------- the variants ---

def variant_of(k):
    """Which module stands at index k (0..80) of the bore."""
    if k <= 1:
        return "entrance"
    if k == N_MOD - 1:
        return "far"
    if k % 8 == 4:
        return "C"   # a hydrant cabinet every 48 m
    if k % 20 == 11:
        return "E"   # jet fans every 120 m
    if k % 4 == 2:
        return "B"   # an exit sign every 24 m
    if (k * 2654435761) % 97 < 9:
        return "D"   # now and then a dead bar
    return "A"


# ------------------------------------------------------------- Blender ---

MATS = {
    # name: (kind, texture, tint (linear), roughness)
    "floor": ("baked", "floor.jpg", (1, 1, 1), 0.5),
    "plaza": ("baked", "plaza.jpg", (1, 1, 1), 0.45),
    "tile": ("baked", "tile.jpg", (1, 1, 1), 0.2),
    "tile_green": ("baked", "tile.jpg", (0.20, 0.58, 0.36), 0.2),
    "vault": ("baked", "vault.jpg", (1, 1, 1), 0.8),
    "concrete": ("baked", "vault.jpg", (1.25, 1.22, 1.16), 0.8),
    "road": ("baked", "road.jpg", (1, 1, 1), 0.85),
    "stone": ("baked", "stone.jpg", (1, 1, 1), 0.8),
    "panel": ("baked", "panel.jpg", (1, 1, 1), 0.4),
    "steel": ("baked", "paint.jpg", (0.42, 0.44, 0.44), 0.5),
    "steel_dark": ("baked", "paint.jpg", (0.16, 0.16, 0.16), 0.5),
    "alu": ("baked", "paint.jpg", (1.2, 1.2, 1.2), 0.4),
    "galv": ("baked", "paint.jpg", (0.92, 0.93, 0.93), 0.45),
    "white_paint": ("baked", "paint.jpg", (1.45, 1.45, 1.42), 0.5),
    "rail_orange": ("baked", "paint.jpg", (1.35, 0.42, 0.10), 0.4),
    "hatch": ("baked", "paint.jpg", (0.46, 0.44, 0.41), 0.5),
    "cabinet": ("baked", "paint.jpg", (1.35, 1.35, 1.33), 0.3),
    "door": ("baked", "paint.jpg", (0.38, 0.46, 0.42), 0.5),
    "door_old": ("baked", "paint.jpg", (0.30, 0.25, 0.21), 0.6),
    "door_frame_old": ("baked", "paint.jpg", (0.20, 0.17, 0.15), 0.6),
    "note": ("baked", None, (0.80, 0.79, 0.74), 0.7),
    "tape": ("baked", None, (0.70, 0.66, 0.52), 0.4),
    "black": ("baked", None, (0.02, 0.02, 0.02), 0.9),
    "plaque": ("baked", None, (0.03, 0.03, 0.03), 0.3),
    "window_dark": ("baked", None, (0.02, 0.022, 0.025), 0.1),
    "sign_exit": ("sign", None, (0.03, 0.30, 0.12), 0.4),
    "sign_info": ("sign", None, (0.03, 0.06, 0.28), 0.4),
    "sign_name": ("sign", None, (0.03, 0.03, 0.03), 0.4),
    "sign_gate": ("sign", None, (0.75, 0.75, 0.73), 0.4),
    "glass": ("glass", None, (0.85, 0.9, 0.88), 0.05),
    "led": ("emissive", None, COOL, 0.0),
    "led_off": ("baked", None, (0.55, 0.55, 0.55), 0.3),
    "lamp": ("emissive", None, LAMP_C, 0.0),
    "street_lamp": ("emissive", None, WARM, 0.0),
    "rail_led": ("emissive", None, COOL, 0.0),
    "red_led": ("emissive", None, RED, 0.0),
    "signal_green": ("emissive", None, GREEN, 0.0),
    "signal_back": ("baked", None, (0.05, 0.05, 0.05), 0.5),
    "window_lit": ("emissive", None, WINDOW, 0.0),
}


def blender_material(name):
    kind, tex, tint, rough = MATS[name]
    m = bpy.data.materials.get(name)
    if m:
        return m
    m = bpy.data.materials.new(name)
    m.use_nodes = True  # (Blender 5 keeps it for now)
    nt = m.node_tree
    for n in list(nt.nodes):
        nt.nodes.remove(n)
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    if kind == "glass":
        sh = nt.nodes.new("ShaderNodeBsdfTransparent")
        sh.inputs["Color"].default_value = (*tint, 1.0)
        nt.links.new(sh.outputs[0], out.inputs["Surface"])
        return m
    if kind == "emissive":
        # dark to rays: the area light beside it gives the light
        sh = nt.nodes.new("ShaderNodeBsdfDiffuse")
        sh.inputs["Color"].default_value = (0.8, 0.8, 0.8, 1.0)
        nt.links.new(sh.outputs[0], out.inputs["Surface"])
        return m
    sh = nt.nodes.new("ShaderNodeBsdfPrincipled")
    sh.inputs["Roughness"].default_value = rough
    if tex:
        img = bpy.data.images.load(os.path.join(TEX, tex), check_existing=True)
        img.colorspace_settings.name = "sRGB"
        ti = nt.nodes.new("ShaderNodeTexImage")
        ti.image = img
        mix = nt.nodes.new("ShaderNodeMix")
        mix.data_type = "RGBA"
        mix.blend_type = "MULTIPLY"
        mix.inputs["Factor"].default_value = 1.0
        nt.links.new(ti.outputs["Color"], mix.inputs[6])
        mix.inputs[7].default_value = (*tint, 1.0)
        nt.links.new(mix.outputs[2], sh.inputs["Base Color"])
    else:
        sh.inputs["Base Color"].default_value = (*tint, 1.0)
    nt.links.new(sh.outputs[0], out.inputs["Surface"])
    return m


def to_blender(mesh, objname):
    me = bpy.data.meshes.new(objname)
    me.from_pydata(mesh.co, [], mesh.faces)
    uv = me.uv_layers.new(name="UV")
    flat = [c for face in mesh.uvs for u in face for c in u]
    uv.data.foreach_set("uv", flat)
    me.polygons.foreach_set("use_smooth", [mesh.smooth] * len(me.polygons))
    me.validate(clean_customdata=False)
    me.update()
    me.materials.append(blender_material(mesh.mat))
    ob = bpy.data.objects.new(objname, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def area_light(li, name):
    ld = bpy.data.lights.new(name, "AREA")
    ld.shape = "RECTANGLE"
    ld.size, ld.size_y = li["size"]
    ld.energy = li["watts"]
    ld.color = li["colour"]
    ld.spread = math.radians(li.get("spread", 180.0))
    ob = bpy.data.objects.new(name, ld)
    # aim: the light shines down its local -Z; its local Y along `axis`
    d = np.array(li["dir"], np.float64)
    d /= np.linalg.norm(d)
    ax = np.array(li["axis"], np.float64)
    ax = ax - d * np.dot(ax, d)
    if np.linalg.norm(ax) < 1e-6:
        ax = np.array([1.0, 0, 0]) - d * d[0]
    ax /= np.linalg.norm(ax)
    zc = -d
    yc = ax
    xc = np.cross(yc, zc)
    import mathutils
    mtx = mathutils.Matrix(((xc[0], yc[0], zc[0]), (xc[1], yc[1], zc[1]), (xc[2], yc[2], zc[2])))
    ob.matrix_world = mathutils.Matrix.Translation(li["centre"]) @ mtx.to_4x4()
    bpy.context.scene.collection.objects.link(ob)
    return ob


def clear_scene():
    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob, do_unlink=True)
    for me in list(bpy.data.meshes):
        bpy.data.meshes.remove(me)
    for ld in list(bpy.data.lights):
        bpy.data.lights.remove(ld)


def setup_cycles(samples):
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    prefs = bpy.context.preferences.addons["cycles"].preferences
    try:
        prefs.compute_device_type = "METAL"
        prefs.get_devices()
        for d in prefs.devices:
            d.use = True
        sc.cycles.device = "GPU"
    except Exception as e:  # no GPU: the CPU
        print("cycles: CPU", e)
        sc.cycles.device = "CPU"
    sc.cycles.samples = samples
    sc.cycles.max_bounces = 8
    sc.cycles.diffuse_bounces = 6
    sc.cycles.glossy_bounces = 2
    sc.cycles.transmission_bounces = 4
    sc.cycles.transparent_max_bounces = 12
    sc.cycles.caustics_reflective = False
    sc.cycles.caustics_refractive = False
    sc.cycles.sample_clamp_indirect = 8.0
    sc.cycles.use_denoising = False
    world = bpy.data.worlds.new("night") if not bpy.data.worlds else bpy.data.worlds[0]
    world.use_nodes = True
    bg = world.node_tree.nodes.get("Background")
    bg.inputs["Color"].default_value = (0.010, 0.012, 0.020, 1.0)
    bg.inputs["Strength"].default_value = 1.0
    sc.world = world


# ------------------------------------------------------------ the bakes ---

def build_scene(targets, context):
    """Put a bake's kits in the scene: targets (to bake, by piece) and
    context (only lighting and occluding). Returns {piece: [objects]}."""
    clear_scene()
    objs = {}
    n = 0
    for piece, kit in targets.items():
        objs[piece] = []
        for (name, mat, smooth), mesh in kit.meshes.items():
            if not mesh.faces:
                continue
            ob = to_blender(mesh, f"{piece}:{name}:{mat}")
            ob["piece"] = piece
            objs[piece].append(ob)
        for li in kit.lights:
            area_light(li, f"L{n}")
            n += 1
    for kit in context:
        for (name, mat, smooth), mesh in kit.meshes.items():
            if mesh.faces:
                to_blender(mesh, f"ctx:{name}:{mat}")
        for li in kit.lights:
            area_light(li, f"L{n}")
            n += 1
    return objs


def bake(objs):
    """Cycles' diffuse light (direct + indirect, no colour) into a float
    corner colour 'light' on every baked object."""
    todo = [ob for ob in objs if MATS[ob.data.materials[0].name][0] in ("baked", "sign")]
    if not todo:
        return
    for ob in todo:
        me = ob.data
        a = me.color_attributes.new(name="light", type="FLOAT_COLOR", domain="CORNER")
        me.color_attributes.active_color = a
        me.color_attributes.render_color_index = me.color_attributes.active_color_index
    bpy.ops.object.select_all(action="DESELECT")
    for ob in todo:
        ob.select_set(True)
    bpy.context.view_layer.objects.active = todo[0]
    sc = bpy.context.scene
    sc.render.bake.target = "VERTEX_COLORS"
    sc.render.bake.use_pass_direct = True
    sc.render.bake.use_pass_indirect = True
    sc.render.bake.use_pass_color = False
    t = time.time()
    bpy.ops.object.bake(type="DIFFUSE", pass_filter={"DIRECT", "INDIRECT"}, target="VERTEX_COLORS", use_clear=True)
    print(f"  baked {len(todo)} objects, {sum(len(o.data.loops) for o in todo)} corners in {time.time() - t:.1f} s")


def corners(ob):
    """An object's corners: position, normal, uv, light (numpy arrays, Blender frame)."""
    me = ob.data
    me.calc_loop_triangles()
    nl = len(me.loops)
    co = np.empty(len(me.vertices) * 3, np.float32)
    me.vertices.foreach_get("co", co)
    co = co.reshape(-1, 3)
    lv = np.empty(nl, np.int64)
    me.loops.foreach_get("vertex_index", lv)
    nrm = np.empty(nl * 3, np.float32)
    me.corner_normals.foreach_get("vector", nrm)
    uv = np.empty(nl * 2, np.float32)
    me.uv_layers["UV"].data.foreach_get("uv", uv)
    col = np.ones(nl * 4, np.float32)
    if "light" in me.color_attributes:
        me.color_attributes["light"].data.foreach_get("color", col)
    tri = np.empty(len(me.loop_triangles) * 3, np.int64)
    me.loop_triangles.foreach_get("loops", tri)
    return dict(pos=co[lv], nrm=nrm.reshape(-1, 3), uv=uv.reshape(-1, 2), light=col.reshape(-1, 4)[:, :3].copy(), tri=tri.reshape(-1, 3))


def smooth_ends(c, y0, y1):
    """A plain module's two ends made to agree: corners at y0 and y1 on the
    same (x, z) take their mean (the module follows itself)."""
    y = c["pos"][:, 1]
    at = np.nonzero((np.abs(y - y0) < 1e-4) | (np.abs(y - y1) < 1e-4))[0]
    groups = {}
    for i in at:
        k = (round(float(c["pos"][i, 0]), 3), round(float(c["pos"][i, 2]), 3), round(float(c["nrm"][i, 0]), 1), round(float(c["nrm"][i, 2]), 1))
        groups.setdefault(k, []).append(i)
    for idx in groups.values():
        if len(idx) > 1:
            c["light"][idx] = c["light"][idx].mean(0)


def ckeys(c, dy=0.0, mirror=False):
    """Each corner's key: its position (mm) in the plain module's frame and
    its normal (to a tenth)."""
    P = c["pos"].astype(np.float64).copy()
    N = c["nrm"].astype(np.float64).copy()
    P[:, 1] -= dy
    if mirror:
        P[:, 0] = MIRROR - P[:, 0]
        N[:, 0] = -N[:, 0]
    return [tuple(r) for r in np.concatenate([np.round(P, 3), np.round(N, 1)], 1).tolist()]


def plain_lookup(meshes):
    d = {}
    for mat, cs in meshes.items():
        for c in cs:
            for k, l in zip(ckeys(c), c["light"]):
                d[(mat, k)] = l
    return d


def ease(meshes, lookup, ends, dy=0.0, mirror=False, fade=1.5):
    """Light near `ends` (y, the piece's frame) eased to the plain module's
    at the same corner, so a piece meets a plain module without a seam."""
    for mat, cs in meshes.items():
        for c in cs:
            y = c["pos"][:, 1]
            d = np.min(np.stack([np.abs(y - e) for e in ends]), axis=0)
            sel = np.nonzero(d < fade)[0]
            if not len(sel):
                continue
            ks = ckeys(c, dy, mirror)
            for i in sel:
                ref = lookup.get((mat, ks[i]))
                if ref is None:
                    continue
                w = d[i] / fade
                w = w * w * (3 - 2 * w)
                c["light"][i] = ref + (c["light"][i] - ref) * w


def to_engine(p):
    """Blender (x, y, z) -> the engine's (x, z, -y)."""
    return np.stack([p[:, 0], p[:, 2], -p[:, 1]], axis=1)


class Writer:
    def __init__(self):
        self.pos, self.nrm, self.uv, self.light, self.idx = [], [], [], [], []
        self.nv = 0
        self.ni = 0
        self.pieces = {}

    def piece(self, name, meshes, lights, origin):
        """meshes: {material: [corner dicts]} in Blender's frame; written
        welded, in the engine's frame, relative to `origin` (Blender)."""
        out = []
        o = np.array(origin, np.float32)
        for mat in sorted(meshes):
            cs = meshes[mat]
            P = np.concatenate([c["pos"] for c in cs]) - o
            N = np.concatenate([c["nrm"] for c in cs])
            UV = np.concatenate([c["uv"] for c in cs])
            L = np.concatenate([c["light"] for c in cs])
            T = []
            base = 0
            for c in cs:
                T.append(c["tri"] + base)
                base += len(c["pos"])
            T = np.concatenate(T)
            # weld corners that agree in every attribute
            rows = np.concatenate([np.round(P, 4), np.round(N, 3), np.round(UV, 4), np.round(L, 4)], axis=1)
            uniq, first, inv = np.unique(rows, axis=0, return_index=True, return_inverse=True)
            inv = inv.reshape(-1)
            P, N, UV, L = P[first], N[first], UV[first], L[first]
            T = inv[T]
            Pe, Ne = to_engine(P), to_engine(N)
            vstart, istart = self.nv, self.ni
            self.pos.append(Pe.astype(np.float32))
            self.nrm.append(Ne.astype(np.float32))
            self.uv.append(np.stack([UV[:, 0], 1.0 - UV[:, 1]], 1).astype(np.float32))
            self.light.append(L.astype(np.float32))
            # (x, y, z) -> (x, z, -y) is a turn, not a mirror: the winding stands
            self.idx.append(T.reshape(-1).astype(np.uint32))
            self.nv += len(Pe)
            self.ni += T.size
            lo, hi = Pe.min(0), Pe.max(0)
            out.append(dict(material=mat, vertices=[vstart, len(Pe)], indices=[istart, int(T.size)], bounds=[*map(float, lo), *map(float, hi)]))
        ls = []
        for li in lights:
            c = np.array(li["centre"], np.float32) - o
            ce = [float(c[0]), float(c[2]), float(-c[1])]
            d = li["dir"]
            ls.append(dict(kind=li["kind"], at=ce, dir=[d[0], d[2], -d[1]], size=list(li["size"]), axis=[li["axis"][0], li["axis"][2], -li["axis"][1]],
                           colour=list(li["colour"]), watts=li["watts"]))
        self.pieces[name] = dict(meshes=out, lights=ls)

    def write(self, path, manifest):
        manifest["pieces"] = self.pieces
        js = json.dumps(manifest, separators=(",", ":")).encode()
        body = bytearray()
        body += struct.pack("<I", len(js)) + js
        while len(body) % 4:
            body += b" "
        cat = lambda xs, dt: np.concatenate(xs).astype(dt).tobytes() if xs else b""
        body += struct.pack("<II", self.nv, self.ni)
        body += cat(self.pos, np.float32) + cat(self.nrm, np.float32) + cat(self.uv, np.float32) + cat(self.light, np.float32)
        body += cat(self.idx, np.uint32)
        z = zlib.compress(bytes(body), 9)
        with open(path, "wb") as f:
            f.write(b"JHMSET\x00\x01" + struct.pack("<I", len(body)) + z)
        print(f"wrote {path}: {self.nv} vertices, {self.ni // 3} triangles, {len(z) / 1e6:.1f} MB")


def gather(objs):
    by = {}
    for ob in objs:
        by.setdefault(ob.data.materials[0].name, []).append(corners(ob))
    return by


def module_kits(variant, k_mid=0):
    """A variant's bake: the module at k_mid and four plain neighbours a side."""
    target = Kit()
    module(target, k_mid * MOD, variant, dense=True)
    ctx = Kit()
    for k in range(k_mid - 4, k_mid + 5):
        if k != k_mid:
            module(ctx, k * MOD, "A", dense=False)
    return target, ctx


def rewrite_manifest(path):
    """The set file's manifest made again (placements, walkable ground, lit
    places, spawns, materials), its baked pieces kept: for a change that
    moves no light."""
    raw = open(path, "rb").read()
    body = zlib.decompress(raw[12:])
    jl = struct.unpack_from("<I", body, 0)[0]
    old = json.loads(body[4:4 + jl])
    at = (4 + jl + 3) // 4 * 4
    rest = body[at:]
    m = make_manifest()
    m["pieces"] = old["pieces"]
    js = json.dumps(m, separators=(",", ":")).encode()
    nb = bytearray(struct.pack("<I", len(js)) + js)
    while len(nb) % 4:
        nb += b" "
    nb += rest
    open(path, "wb").write(b"JHMSET\x00\x01" + struct.pack("<I", len(nb)) + zlib.compress(bytes(nb), 9))
    print(f"rewrote the manifest of {path}")


def main():
    o = ARGS
    if o.get("manifest-only"):
        return rewrite_manifest(o["out"])
    setup_cycles(o["samples"])
    if o["render"]:
        return render_view(o)
    only = o["only"]
    w = Writer()
    lookup = None
    t0 = time.time()
    for v in ("A", "B", "C", "D", "E", "F", "G"):
        if only and v not in only and v != "A":
            continue
        print(f"module {v}")
        target, ctx = module_kits(v)
        objs = build_scene({v: target}, [ctx])[v]
        bake(objs)
        meshes = gather(objs)
        if v == "A":
            for cs in meshes.values():
                for c in cs:
                    smooth_ends(c, 0.0, MOD)
            lookup = plain_lookup(meshes)
        else:
            ease(meshes, lookup, [0.0, MOD])
        if not only or v in only:
            w.piece(f"module_{v}", meshes, target.lights, (0, 0, 0))
    if not only or "entrance" in only:
        print("entrance")
        ent = Kit()
        exterior(ent)
        module(ent, 0.0, "A")
        module(ent, MOD, "B")
        two = Kit()
        module(two, 0.0, "A")
        module(two, MOD, "A")
        two = mirror_kit(two)
        ctx = Kit()
        for k in range(2, 9):
            module(ctx, k * MOD, "A", dense=False)
        ctx2 = mirror_kit(ctx)
        objs = build_scene({"entrance": ent, "entrance2": two}, [ctx, ctx2])
        bake(objs["entrance"] + objs["entrance2"])
        m1, m2 = gather(objs["entrance"]), gather(objs["entrance2"])
        ease(m1, lookup, [2 * MOD], dy=MOD)
        ease(m2, lookup, [2 * MOD], dy=MOD, mirror=True)
        w.piece("entrance", m1, ent.lights, (0, 0, 0))
        w.piece("entrance2", m2, two.lights, (0, 0, 0))
    if not only or "far" in only:
        print("far end")
        far = Kit()
        module(far, TUNNEL - MOD, "A")
        far_end(far)
        ctx = Kit()
        for k in range(N_MOD - 9, N_MOD - 1):
            module(ctx, k * MOD, "A", dense=False)
        objs = build_scene({"far": far}, [ctx])["far"]
        bake(objs)
        mf = gather(objs)
        ease(mf, lookup, [TUNNEL - MOD], dy=TUNNEL - MOD)
        w.piece("far", mf, far.lights, (0, TUNNEL - MOD, 0))
    manifest = make_manifest()
    os.makedirs(os.path.dirname(o["out"]), exist_ok=True)
    w.write(o["out"], manifest)
    if o["blend"]:
        bpy.ops.wm.save_as_mainfile(filepath=os.path.abspath(o["blend"]))
    print(f"done in {time.time() - t0:.0f} s")


def make_manifest():
    """Everything the run time needs besides the meshes: the materials, where
    each piece stands, the walkable ground, the places that are lit (for the
    fog), the spawns."""
    place = []
    place.append(dict(piece="entrance", at=[0, 0, 0]))
    place.append(dict(piece="entrance2", at=[0, 0, 0]))
    for k in range(2, N_MOD - 1):
        v = variant_of(k)
        place.append(dict(piece=f"module_{v}", at=[0, 0, -k * MOD], index=k))
        place.append(dict(piece="module_A", at=[0, 0, -k * MOD], index=k, mirror=MIRROR, bore=2))
    place.append(dict(piece="far", at=[0, 0, -(TUNNEL - MOD)], index=N_MOD - 1))
    # walkable ground, the engine's frame: boxes x0, x1, z0, z1 at height y
    r = 0.0
    walk = []
    walk.append(dict(x=[WALL_X, BASE_X], z=[-(TUNNEL - 1.25), 0.5], y=WALK_Z, name="walkway"))
    walk.append(dict(x=[STAIR_X1, FOOT_X1 - 0.12], z=[0.0, 80.0], y=WALK_Z, name="plaza"))
    walk.append(dict(x=[LEFT_WALL, STAIR_X1], z=[0.0, -STAIRS[1][1]], y=WALK_Z, name="foot of the stairs"))
    walk.append(dict(x=[LEFT_WALL, STAIR_X1], z=[-STAIR_TOP_Y, 80.0], y=WALK_Z, name="beyond the stairs"))
    for (ya, yb, z) in stair_steps():
        walk.append(dict(x=[STAIR_X0, STAIR_X1], z=[-yb, -ya], y=z, name="stairs"))
    walk.append(dict(x=[-8.2, LEFT_WALL - 0.3], z=[1.0, 80.0], y=UPPER_Z, name="street above"))
    walk.append(dict(x=[LEFT_WALL - 0.3, LEFT_WALL], z=[-STAIRS[0][0], -STAIR_TOP_Y], y=UPPER_Z, name="the stairs' head"))
    walk.append(dict(x=[-17.0, -8.2], z=[1.0, 80.0], y=UPPER_Z - 0.12, name="the road above"))
    # lit places (fog): the bores, and a little light out of the portals
    lit = [dict(x=[WALL_X, RIGHT_X], z=[-TUNNEL, 0.0], lit=1.0), dict(x=[MIRROR - RIGHT_X, MIRROR - WALL_X], z=[-TUNNEL, 0.0], lit=1.0)]
    spawns = {
        "top": [-7.0, UPPER_Z, 23.5, 0.0],
        "plaza": [-2.6, WALK_Z, 14.0, 0.35],
        "portal": [-0.9, WALK_Z, 3.0, 0.0],
        "inside": [-0.85, WALK_Z, -40.0, 0.0],
        "deep": [-0.85, WALK_Z, -300.0, 0.0],
        "end": [-0.85, WALK_Z, -470.0, 0.0],
    }
    mats = {k: dict(kind=v[0], texture=v[1], tint=list(v[2])) for k, v in MATS.items()}
    return dict(version=1, frame="metres; the engine's: x across (the walkway at -x), y up, -z into the tunnel; the portal at z = 0",
                module=MOD, tunnel=TUNNEL, mirror=MIRROR, materials=mats, placements=place, walk=walk, lit=lit, spawns=spawns,
                section=dict(walk_y=WALK_Z, wall_x=WALL_X, base_x=BASE_X, glass_x=GLASS_X, head_y=HEAD_Z, right_x=RIGHT_X))


def render_view(o):
    """A Cycles frame of a piece as built (lit by its area lights), to look
    at the geometry before baking."""
    which = (o.get("piece") or "inside")
    kit = Kit()
    if which == "outside":
        exterior(kit)
        module(kit, 0.0, "A", dense=False)
        module(kit, MOD, "B", dense=False)
        for k in range(2, 10):
            module(kit, k * MOD, "A", dense=False)
        add(kit, mirror_kit(kit_range(0, 8)))
    elif which in ("F", "G"):
        # the escape door, in the fourth of seven modules (its middle at y 21)
        for k in range(0, 7):
            module(kit, k * MOD, which if k == 3 else "A", dense=k == 3)
    else:
        for k in range(0, 12):
            module(kit, k * MOD, variant_of(k + 2) if k > 1 else "A", dense=False)
    build_scene({"view": kit}, [])
    # emissive faces visible to the camera
    for name, (kind, tex, tint, rough) in MATS.items():
        if kind == "emissive" and name in bpy.data.materials:
            m = bpy.data.materials[name]
            nt = m.node_tree
            for n in list(nt.nodes):
                nt.nodes.remove(n)
            out = nt.nodes.new("ShaderNodeOutputMaterial")
            em = nt.nodes.new("ShaderNodeEmission")
            em.inputs["Color"].default_value = (*tint, 1.0)
            em.inputs["Strength"].default_value = 30.0
            nt.links.new(em.outputs[0], out.inputs["Surface"])
    v = [float(x) for x in (o["view"] or "-0.85,-2,1.6,0,0").split(",")]
    x, y, z, yaw, pitch = v[:5]
    fov = v[5] if len(v) > 5 else 70.0
    import mathutils
    cam = bpy.data.cameras.new("cam")
    cam.angle = math.radians(fov)
    cob = bpy.data.objects.new("cam", cam)
    bpy.context.scene.collection.objects.link(cob)
    cob.location = (x, y, z)
    # yaw 0 looks along +y (into the tunnel), pitch up
    cob.rotation_euler = mathutils.Euler((math.radians(90) + pitch, 0.0, yaw), "XYZ")
    sc = bpy.context.scene
    sc.camera = cob
    sc.render.resolution_x = int(o.get("w") or 720)
    sc.render.resolution_y = int(o.get("h") or 1280)
    sc.render.film_transparent = False
    sc.view_settings.view_transform = "AgX"
    sc.view_settings.exposure = float(o.get("exposure") or 0.0)
    sc.cycles.samples = min(o["samples"], 64)
    sc.cycles.use_denoising = True
    sc.render.filepath = os.path.abspath(o["render"])
    bpy.ops.render.render(write_still=True)
    print("rendered", o["render"])


def kit_range(k0, k1):
    k = Kit()
    for i in range(k0, k1):
        module(k, i * MOD, "A", dense=False)
    return k


if __name__ == "__main__":
    import builtins
    _print = builtins.print
    builtins.print = lambda *a, **k: _print(*a, **{**k, "flush": True})
    main()
    # Blender can hang tearing down the GPU after a bake: the work is written, go
    sys.stdout.flush()
    os._exit(0)
