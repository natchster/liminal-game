"""
The Hound's skinned body (run by tools/blender/skinned.py, which provides
C = creatures.py's helpers, Piece, V, BONES, bone_matrix, math).

A starved adult on all fours. Seen from above: a knobbed spine between thin
erector cords, the ribs standing out down both flanks, shoulder blades lying
on the ribcage (their own bones, so they slide and stand up under the skin),
sharp hip bones and a sacrum, the flank sucked in between the last rib and
the pelvis. From the side: a deep, narrow-bottomed chest, a hollow belly,
stringy arms with knobbed elbows, long hands splayed flat with broken nails,
thin legs on the balls of the feet with the heels up.

Everything is placed from the rig (.cache/rigs/hound.json, from makeHound):
the trunk and the shoulder blades are modelled in the root frame (the spine
joint, 0.70 m up), the limbs in their bones' frames (the bone hanging -Y),
the hands, feet and every digit segment in their bones' frames, so the
numbers here never repeat the rig's.
"""
ell, tube, chain, sweep, loft = C['ell'], C['tube'], C['chain'], C['sweep'], C['loft']
lerp, smooth, clamp, mix3, mul3, hexc, interp, spow = C['lerp'], C['smooth'], C['clamp'], C['mix3'], C['mul3'], C['hexc'], C['interp'], C['spow']
mnoise = C['mnoise']
from mathutils import Matrix as _Mx
TAU = math.tau

def bmat(b): return bone_matrix(b)
def to_frame(frame, p): return bmat(frame).inverted() @ V(p)
def head_w(b): return V(BONES[b]['head'])
def tail_w(b): return V(BONES[b]['tail'])
def tail_local(b): return to_frame(b, BONES[b]['tail'])
ROOT_W = head_w('root')          # the root frame: the spine joint, axes of the creature
def rootp(p): return V(p) + ROOT_W   # root frame -> world (the root is not rotated at rest)

def ellm(bm, c, ax, ay, az, r, seg=16):
    """an ellipsoid at c with its radii r along the (orthonormal) axes ax, ay, az"""
    ax, ay, az = V(ax).normalized(), V(ay).normalized(), V(az).normalized()
    m = _Mx(((ax.x * r[0], ay.x * r[1], az.x * r[2], c[0]), (ax.y * r[0], ay.y * r[1], az.y * r[2], c[1]),
             (ax.z * r[0], ay.z * r[1], az.z * r[2], c[2]), (0, 0, 0, 1)))
    bmesh.ops.create_uvsphere(bm, u_segments=seg, v_segments=max(8, seg // 2), radius=1.0, matrix=m)


# ------------------------------------------------------------------ trunk ---
# Sections along Z in the root frame (z, half-width, top y, bottom y), kept in
# creatures.py's HOUND section because the mane roots on them too. The hips
# (x 0.12, y 0.12, z -0.25) sit in the pelvis, the shoulder joints (x 0.15,
# y -0.075, z 0.30) just outside the ribcage at mid-depth. The anatomy is a
# displacement of the plain sections (disp), so a loft angle a runs 0..2pi.
HDS = C['HD_SKIN_TRUNK']
def sec(z):
    w, top, bot = interp(HDS, z)
    return w, (top + bot) / 2, (top - bot) / 2

def base_pt(z, a):
    """the plain section, a = angle round Z from the back (+Y) towards +X"""
    w, yc, hh = sec(z)
    sa, ca = math.sin(a), math.cos(a)
    return V((w * spow(sa, 2 / 2.4), yc + hh * spow(ca, 2 / (2.4 if ca >= 0 else 2.1)), z))

def out_dir(z, a):
    """outward normal of the section at a (in the section's plane)"""
    p0, p1 = base_pt(z, a - 0.01), base_pt(z, a + 0.01)
    t = p1 - p0
    n = V((t.y, -t.x, 0.0))
    if n.length < 1e-9: return V((0, 1, 0))
    n.normalize()
    q = base_pt(z, a); w, yc, hh = sec(z)
    if n.dot(V((q.x, q.y - yc, 0))) < 0: n = -n
    return n

def gauss(x, s): return math.exp(-(x / s) ** 2)

def frame_at(z, a):
    """(round the section, out of the skin, along the body) at a trunk point"""
    n = out_dir(z, a)
    t = n.cross(V((0, 0, 1))).normalized()
    return t, n, t.cross(n).normalized()

# rib i leaves the spine at z0 and runs down the flank towards the tail (on all
# fours "down the body" is back), the lower ones more steeply
RIBS = [(0.335 - i * 0.0265, 0.04 + 0.0045 * i, i) for i in range(12)]
A0, A1 = 0.42, 2.55
def from_back(a):
    """the angle from the dorsal midline, 0..pi, whichever side"""
    return abs((a + math.pi) % TAU - math.pi)

def ribs_at(z, a):
    t = from_back(a)
    if t < A0 or t > A1 or z < -0.12 or z > 0.37: return 0.0
    u = (t - A0) / (A1 - A0)
    s = 0.0
    for z0, dz, i in RIBS:
        zc = z0 - dz * u ** 1.15
        if abs(z - zc) > 0.03: continue
        env = smooth(0.0, 0.22, u) * (1 - smooth(0.62 - 0.02 * i, 0.98 - 0.025 * i, u))   # the lower ribs end higher (floating ribs)
        env *= lerp(0.4, 1.0, smooth(1.0, 6.0, i))     # the upper ribs sit under the shoulder muscles
        env *= 0.75 + 0.25 * math.sin(i * 2.3 + u * 5.0)                    # uneven, one standing out more than the next
        s += gauss(z - zc, 0.0078) * env
    return s

def costal_z(t):
    """the lower edge of the ribcage: from the breastbone's tip back up the flank"""
    return lerp(0.035, -0.07, smooth(math.pi, 1.45, t))

def disp(z, a, ribs=True):
    """how far the skin stands out (+) or sinks (-) from the plain section"""
    t, d = from_back(a), 0.0
    # ribs: ridges with grooves between them; the ribcage stands proud of the hollow belly
    cage = smooth(-0.1, -0.05, z) * (1 - smooth(0.34, 0.39, z)) * smooth(A0, 0.7, t)
    if ribs: d += 0.0088 * ribs_at(z, a) - 0.0034 * cage
    # the spine: a groove the vertebrae stand in, thin erector cords beside it
    sp = smooth(-0.43, -0.36, z) * (1 - smooth(0.36, 0.41, z))
    d -= 0.0072 * gauss(t, 0.11) * sp
    d += 0.0066 * gauss(t - 0.3, 0.12) * sp * lerp(0.6, 1.0, smooth(0.05, -0.2, z))
    # the sunken flank between the last rib and the pelvis
    fl = smooth(-0.2, -0.15, z) * (1 - smooth(-0.04, 0.02, z)) * smooth(0.9, 1.3, t) * (1 - smooth(2.5, 2.9, t))
    d -= 0.011 * fl
    # the costal margin: a ridge with the belly falling away behind it
    if t > 1.4:
        zm = costal_z(t)
        d += 0.004 * gauss(z - zm, 0.008)
        d -= 0.008 * smooth(zm, zm - 0.05, z) * (1 - smooth(-0.2, -0.13, z)) * smooth(1.7, 2.4, t)
    # the breastbone and thin pectorals
    d += 0.0035 * gauss(t - math.pi, 0.13) * smooth(0.06, 0.12, z) * (1 - smooth(0.33, 0.38, z))
    d += 0.005 * gauss(t - 2.25, 0.35) * gauss(z - 0.29, 0.045)
    # the pelvis: the iliac crest from the sacrum round to the front hip point,
    # dimples either side of the sacrum, the sacrum itself a flat raised plate
    if -0.4 < z < -0.12:
        u = clamp((z + 0.34) / 0.17)
        ac = lerp(0.3, 1.5, u ** 0.8)
        d += 0.0105 * gauss(t - ac, 0.06) * smooth(-0.37, -0.33, z) * (1 - smooth(-0.17, -0.14, z))
        d -= 0.004 * gauss(t - 0.26, 0.07) * gauss(z + 0.335, 0.014)
        d += 0.0028 * gauss(t, 0.2) * smooth(-0.44, -0.4, z) * (1 - smooth(-0.33, -0.3, z))
    # the rump parts into two thin lobes behind, the cleft between them
    pinch = smooth(-0.385, -0.452, z)
    d -= pinch * (0.016 * gauss(t, 0.3) + 0.03 * gauss(t - math.pi, 0.5))
    # the hollow on the side of the hip, under the crest and behind the trochanter
    d -= 0.006 * gauss(t - 1.55, 0.3) * gauss(z + 0.29, 0.04)
    # the trapezius: a thin sheet from the nape out over the shoulders
    d += 0.006 * gauss(t - 0.75, 0.42) * smooth(0.24, 0.33, z) * (1 - smooth(0.385, 0.415, z))
    # a hint of a navel
    d -= 0.003 * gauss(t - math.pi, 0.06) * gauss(z + 0.075, 0.008)
    return d

def surf(z, a, extra=0.0, ribs=True):
    """a point on the finished trunk surface (root frame); ribs=False: as if
    the ribs were smoothed over (what a shoulder blade lies on)"""
    return base_pt(z, a) + out_dir(z, a) * (disp(z, a, ribs) + extra)

def hd_trunk(bm):
    zs, z = [], HDS[0][0]
    while z < HDS[-1][0]:
        zs.append(z)
        z += 0.0032 if -0.15 < z < 0.38 else 0.005
    zs.append(HDS[-1][0])
    n = 144
    rings = [[surf(z, TAU * i / n) for i in range(n)] for z in zs]
    loft(bm, rings)
    # the vertebrae: spinous processes from the neck (C7, the big one) to the
    # sacrum, standing out of the groove; the lumbar ones longer and blunter
    for k in range(22):
        z = lerp(0.385, -0.39, k / 21)
        big = 1.3 if k == 0 else (1.12 if -0.3 < z < -0.08 else 1.0)
        p = surf(z, 0.0, -0.0012)
        ellm(bm, p, *frame_at(z, 0.0), (0.0075 * big, 0.0068 * big, 0.0175), seg=12)
    for z in (-0.405, -0.425, -0.44):                       # the sacrum's ridge
        ell(bm, surf(z, 0.0, 0.0), (0.006, 0.005, 0.007), seg=8)
    for s in (-1, 1):
        # the front hip points and the crests' backs, sharp under the skin
        ellm(bm, surf(-0.17, s * 1.45, -0.006), *frame_at(-0.17, s * 1.45), (0.014, 0.009, 0.024), seg=12)
        ellm(bm, surf(-0.33, s * 0.33, -0.005), *frame_at(-0.33, s * 0.33), (0.011, 0.0075, 0.018), seg=12)
        # thin, sagging buttocks; the cleft forms between them
        ellm(bm, (s * 0.046, 0.062, -0.418), (1, 0, 0), (0, 0.94, -0.34), (0, 0.34, 0.94), (0.044, 0.058, 0.036), seg=18)
        # collarbones from the base of the throat out to the shoulders
        tube(bm, (s * 0.022, -0.078, 0.395), (s * 0.12, -0.04, 0.335), 0.0075, 0.0085, seg=10)
        ell(bm, (s * 0.13, -0.035, 0.33), (0.016, 0.012, 0.015), seg=10)      # the shoulder's point


# ---------------------------------------------------------- shoulder blades ---
def a_at(z, x):
    """the section angle where the trunk's surface is x out from the midline"""
    w = sec(z)[0]
    return math.asin(min(1.0, (abs(x) / w) ** 1.2))

def d2seg(p, a, b):
    """distance from p to the segment ab, in the (z, x) plane"""
    dz, dx = b[0] - a[0], b[1] - a[1]
    L2 = dz * dz + dx * dx
    t = clamp(((p[0] - a[0]) * dz + (p[1] - a[1]) * dx) / L2) if L2 > 0 else 0.0
    return math.hypot(p[0] - a[0] - dz * t, p[1] - a[1] - dx * t)

def tri_sd(p, A, B, Cc):
    """signed distance to a triangle in the (z, x) plane, negative inside"""
    d = min(d2seg(p, A, B), d2seg(p, B, Cc), d2seg(p, Cc, A))
    def side(a, b): return (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
    s1, s2, s3 = side(A, B), side(B, Cc), side(Cc, A)
    inside = (s1 >= 0 and s2 >= 0 and s3 >= 0) or (s1 <= 0 and s2 <= 0 and s3 <= 0)
    return -d if inside else d

# the blade in (z, x) on the back: inferior, superior and lateral angles, and its spine
SC_I, SC_S, SC_L = (0.122, 0.07), (0.272, 0.058), (0.288, 0.122)
SC_SP = ((0.238, 0.06), (0.3, 0.132))
def scap_bump(z, x):
    """how far the blade lifts the skin: a low plate, its inner border and lower
    point standing out, the spine of the blade a ridge out to the shoulder, a
    hollow under it; it sinks under the skin all round its edge"""
    p = (z, x)
    sd = tri_sd(p, SC_I, SC_S, SC_L)
    h = 0.0128 * smooth(0.018, -0.008, sd)
    h += 0.0042 * gauss(d2seg(p, SC_I, SC_S), 0.0065) * smooth(0.02, 0.0, sd)
    h += 0.0042 * gauss(math.hypot(z - SC_I[0], x - SC_I[1]), 0.011)
    h += 0.0058 * gauss(d2seg(p, *SC_SP), 0.0068) * smooth(0.03, 0.0, sd)
    below = smooth(SC_SP[0][0] - 0.005, SC_SP[0][0] - 0.06, z)
    h -= 0.0014 * smooth(0.0, -0.02, sd) * below
    return h - 0.0068      # under the ribs' grooves all round its edge, by more than a voxel: no seam

def hd_scapula(s):
    """a shoulder blade lying on the ribcage (root frame; s = +1 left, -1 right)"""
    def build(bm):
        nz, nx = 30, 16
        z0, z1 = 0.092, 0.322
        top, bot = {}, {}
        for i in range(nz + 1):
            z = lerp(z0, z1, i / nz)
            xm = min(0.152, 0.94 * sec(z)[0])
            for j in range(nx + 1):
                x = lerp(0.034, xm, j / nx)
                a = s * a_at(z, x)
                top[i, j] = bm.verts.new(surf(z, a, scap_bump(z, x), ribs=False))
                bot[i, j] = bm.verts.new(surf(z, a, -0.018, ribs=False))
        for i in range(nz):
            for j in range(nx):
                bm.faces.new((top[i, j], top[i + 1, j], top[i + 1, j + 1], top[i, j + 1]))
                bm.faces.new((bot[i, j + 1], bot[i + 1, j + 1], bot[i + 1, j], bot[i, j]))
        ring = ([(i, 0) for i in range(nz + 1)] + [(nz, j) for j in range(1, nx + 1)]
                + [(i, nx) for i in range(nz - 1, -1, -1)] + [(0, j) for j in range(nx - 1, 0, -1)])
        for k in range(len(ring)):
            p, q = ring[k], ring[(k + 1) % len(ring)]
            bm.faces.new((top[p], bot[p], bot[q], top[q]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        # the acromion, the end of the blade's spine over the joint, meeting the collarbone
        tube(bm, surf(0.3, s * a_at(0.3, 0.128), 0.0), (s * 0.135, -0.032, 0.322), 0.008, 0.011, seg=10)
        # the top of the shoulder rides on the blade: a cap over the joint
        ell(bm, (s * 0.132, -0.048, 0.302), (0.021, 0.029, 0.029), seg=14)
    return build


# -------------------------------------------------------------------- neck ---
def hd_neck(bm):
    """neck frame: +Z along the neck from the chest to the skull (0.21), +Y the nape"""
    tube(bm, (0, 0.006, -0.07), (0, 0.0, 0.2), 0.039, 0.031)
    for s in (-1, 1):
        tube(bm, (s * 0.031, 0.004, 0.175), (s * 0.013, -0.027, -0.035), 0.008, 0.0095, seg=10)   # the cords to the breastbone
        tube(bm, (s * 0.022, 0.022, 0.15), (s * 0.04, 0.02, -0.05), 0.009, 0.013, seg=10)        # the nape's muscles into the trapezius
    tube(bm, (0, -0.031, -0.02), (0, -0.028, 0.15), 0.011, 0.0095, seg=10)                         # the windpipe
    ell(bm, (0, -0.037, 0.095), (0.011, 0.011, 0.014), seg=10)                                     # Adam's apple
    for k in range(4): ell(bm, (0, 0.03, 0.025 + k * 0.042), (0.0065, 0.006, 0.008), seg=8)       # the knobs of the nape


# ------------------------------------------------------------------ limbs ---
# Starved but human: the bones show at every joint (an elbow's point, the
# knee's flat front, the ankle's knobs), the muscle is stringy and separate,
# and nothing is round like a ball joint.
UP, FORE, THIGH, SHIN = (tail_local(b).length for b in ('upperarm.L', 'forearm.L', 'thigh.L', 'shin.L'))

def hd_uparm(s):
    """shoulder at the origin, the elbow at -UP; +X out on the left arm"""
    L = UP
    def build(bm):
        ell(bm, (-s * 0.004, -0.004, -0.002), (0.019, 0.02, 0.02), seg=14)            # the head of the humerus
        tube(bm, (0.0, -0.012, -0.002), (s * 0.012, -0.135, 0.004), 0.022, 0.007)      # the deltoid, a thin cap to a point
        tube(bm, (0, -0.01, 0), (0, -L + 0.03, -0.002), 0.0175, 0.0155)               # the arm
        ell(bm, (0, -0.175, 0.012), (0.0155, 0.088, 0.0145))                          # biceps, a thin cord
        ell(bm, (-s * 0.002, -0.14, -0.012), (0.019, 0.105, 0.0165))                  # triceps
        ell(bm, (-s * 0.008, -0.2, -0.016), (0.011, 0.075, 0.011))                    # its long head, stringy
        tube(bm, (-s * 0.013, -0.08, 0.005), (-s * 0.017, -L + 0.035, 0.003), 0.004, 0.0035, seg=8)   # a vein on the inside
        ell(bm, (0, -L + 0.012, -0.004), (0.021, 0.017, 0.016), seg=14)              # the elbow, wider than deep
        ell(bm, (-s * 0.024, -L + 0.013, -0.005), (0.0085, 0.011, 0.0095), seg=10)   # the inner knob pokes out
    return build

def hd_fore(s):
    """elbow at the origin, the wrist at -FORE; the elbow's point is -Z"""
    L = FORE
    def build(bm):
        ell(bm, (0, 0.002, -0.017), (0.0115, 0.016, 0.0105), seg=12)                   # the olecranon
        tube(bm, (0, -0.004, -0.013), (s * 0.012, -L + 0.016, -0.008), 0.0095, 0.0072)  # the ulna's ridge down the back
        tube(bm, (0, -0.01, 0.002), (0, -L + 0.014, 0.001), 0.0185, 0.0115)             # the forearm
        ell(bm, (s * 0.013, -0.065, 0.009), (0.018, 0.068, 0.0165))                     # extensors, lateral
        ell(bm, (-s * 0.011, -0.08, -0.004), (0.018, 0.078, 0.0155))                    # flexors, inside
        for x in (-0.006, 0.0, 0.006):                                                  # tendons to the hand
            tube(bm, (x, -0.15, 0.011), (x * 1.6, -L + 0.024, 0.009), 0.0026, 0.0024, seg=6)
        ell(bm, (s * 0.016, -L + 0.012, -0.006), (0.0068, 0.0075, 0.007), seg=10)      # the wrist's knob
        ell(bm, (0, -L + 0.004, 0), (0.021, 0.0115, 0.0145))                            # the wrist, flat
    return build

def hd_thigh(s):
    """hip at the origin, the knee at -THIGH; the knee points +Z"""
    L = THIGH
    def build(bm):
        tube(bm, (-s * 0.008, -0.025, -0.012), (0, -L + 0.045, 0.006), 0.037, 0.031)    # the thigh, from inside the rump
        ell(bm, (s * 0.03, -0.05, -0.004), (0.013, 0.03, 0.022), seg=12)               # the trochanter, a bony knob
        ell(bm, (0, -0.21, 0.02), (0.033, 0.16, 0.03))                                 # quadriceps, wasted
        ell(bm, (s * 0.02, -0.22, 0.003), (0.021, 0.15, 0.025))                        # the outer thigh
        ell(bm, (0, -0.17, -0.025), (0.031, 0.14, 0.025))                              # hamstrings
        tube(bm, (-s * 0.018, -0.26, -0.021), (-s * 0.021, -L + 0.03, -0.022), 0.0085, 0.0068)   # their tendons, either side
        tube(bm, (s * 0.02, -0.26, -0.021), (s * 0.024, -L + 0.03, -0.02), 0.0078, 0.0066)
        ell(bm, (0, -L + 0.01, -0.002), (0.032, 0.026, 0.025), seg=16)                # the knee: a squat block
        ell(bm, (0, -L + 0.02, 0.026), (0.019, 0.022, 0.011), seg=14)                  # the kneecap
    return build

def hd_shin(s):
    """knee at the origin, the ankle at -SHIN; the shinbone's edge is +Z"""
    L = SHIN
    def build(bm):
        ell(bm, (0, -0.012, 0.006), (0.031, 0.022, 0.024))                             # the tibia's head
        ell(bm, (0, -0.048, 0.023), (0.009, 0.012, 0.008), seg=10)                     # the bump under the kneecap
        tube(bm, (0, -0.02, 0.0), (0, -L + 0.012, -0.002), 0.022, 0.0155)              # the shin, its end over the ankle
        tube(bm, (0, -0.045, 0.018), (0, -L + 0.07, 0.0145), 0.0068, 0.0055, seg=8)    # the shinbone's sharp edge
        ell(bm, (-s * 0.01, -0.125, -0.021), (0.0185, 0.085, 0.02))                    # the calf, two thin heads
        ell(bm, (s * 0.012, -0.11, -0.019), (0.0155, 0.075, 0.018))
        tube(bm, (0, -0.19, -0.02), (0, -L + 0.004, -0.024), 0.0085, 0.0072, seg=10)   # the Achilles tendon
        ell(bm, (s * 0.018, -L + 0.004, 0.0), (0.0065, 0.009, 0.0075), seg=10)         # the ankle bones
        ell(bm, (-s * 0.016, -L + 0.01, 0.002), (0.0075, 0.0095, 0.008), seg=10)
    return build

def hd_armpit(s, n):
    """the folds either side of the armpit: the chest's edge to the front of the
    arm, the back's edge to the back of it (root frame, split thorax / arm)"""
    M = bmat('upperarm.' + n)
    def arm(p): return M @ V(p) - ROOT_W
    def build(bm):
        tube(bm, (s * 0.06, -0.118, 0.255), arm((0, -0.085, 0.012)), 0.011, 0.0085, seg=10)   # the chest's fold
        tube(bm, (s * 0.115, -0.045, 0.215), arm((0, -0.08, -0.014)), 0.012, 0.009, seg=10)   # the back's fold
    return build


# ----------------------------------------------------------- hands and feet ---
# Hand frame: wrist at the origin, the hand flat along +Z, palm down, thumb +X
# (the right hand; the left one's frame is mirrored). Foot frame: ankle at the
# origin, heel up, big toe -X (the left foot). Digit frames: each bone's head
# at the origin, unrotated against the hand or foot at rest, so +Y is the
# back of the finger (the nail side).
NAILS = []        # (world centre, radius): painted as dirty claw

def digit_piece(b, r0, r1, knuckle, nail):
    """one bone of a finger or toe: a tapered segment, the joint knobbed at its
    head, and on the last one the pad and the nail: 'stub' (broken off short,
    a square end) or 'claw' (grown out and hooked, worn dull)"""
    t = tail_local(b)
    d = t.normalized()
    up = V((0, 1, 0))
    side = d.cross(up).normalized()
    up = side.cross(d).normalized()
    def build(bm):
        tube(bm, (0, 0, 0), t, r0, r1, seg=10)
        if knuckle: ell(bm, up * (r0 * 0.18), (r0 * 1.2, r0 * 1.12, r0 * 1.16), seg=10)
        if nail:
            ellm(bm, t - d * (r1 * 0.2) - up * (r1 * 0.25), side, up, d, (r1 * 1.08, r1 * 0.85, r1 * 1.25), seg=10)   # the pad
            base = t - d * (r1 * 1.5) + up * (r1 * 0.8)
            ellm(bm, base, side, up, d, (r1 * 0.88, r1 * 0.32, r1 * 1.45), seg=10)                                   # the nail bed
            if nail == 'claw':
                pts = [base + d * (r1 * 0.6), t + d * (r1 * 0.8) + up * (r1 * 0.55),
                       t + d * (r1 * 2.0) + up * (r1 * 0.15), t + d * (r1 * 2.9) - up * (r1 * 0.55)]
                sweep(bm, pts, [r1 * 0.8, r1 * 0.72, r1 * 0.55, 0.0], sides=6, flat=0.4, ref=up)
            else:
                pts = [base + d * (r1 * 0.6), t + d * (r1 * 0.9) + up * (r1 * 0.45)]
                sweep(bm, pts, [r1 * 0.8, r1 * 0.78], sides=6, flat=0.42, ref=up)    # snapped off square
            NAILS.append((bmat(b) @ (t + up * (r1 * 0.5)), r1 * 2.2))
    return build

def hd_palm(n):
    fr = 'hand.' + n
    mcp = [to_frame(fr, head_w(f'finger{k}0.{n}')) for k in range(4)]
    def build(bm):
        ell(bm, (0, -0.006, 0.008), (0.021, 0.0125, 0.017))                            # the wrist's bones, inside the forearm's end
        ell(bm, (0.0, -0.021, 0.05), (0.04, 0.0105, 0.046))                            # the palm, flat
        ell(bm, (-0.022, -0.024, 0.042), (0.016, 0.0095, 0.036), seg=14)               # the pad under the little finger
        ell(bm, (0.026, -0.023, 0.034), (0.0175, 0.011, 0.03), rot=(0, 0.35, 0), seg=14)   # the ball of the thumb
        for k in range(4):                                                             # tendons over the back of the hand
            tube(bm, (mcp[k].x * 0.35, -0.007, 0.012), (mcp[k].x, mcp[k].y + 0.006, mcp[k].z - 0.006), 0.0052, 0.0068, seg=8)
            ell(bm, mcp[k] + V((0, 0.0025, 0)), (0.0102, 0.0094, 0.0108), seg=12)       # the knuckles
    return build

def hd_web(n, a, b, reach):
    """the skin between two fingers, from the knuckles to `reach` of the way to
    the next joint, its free edge curved back"""
    fr = 'hand.' + n
    A0, A1 = to_frame(fr, head_w(a[0])), to_frame(fr, tail_w(a[0]))
    B0, B1 = to_frame(fr, head_w(b[0])), to_frame(fr, tail_w(b[0]))
    def build(bm):
        nu, nt = 6, 5
        top, bot = {}, {}
        for i in range(nu + 1):
            u = i / nu
            tm = reach * (1 - 0.32 * math.sin(math.pi * u))
            for j in range(nt + 1):
                t = tm * j / nt
                p = (A0 + (A1 - A0) * t).lerp(B0 + (B1 - B0) * t, u) + V((0, -0.0025, 0))
                top[i, j] = bm.verts.new(p + V((0, 0.0028, 0)))
                bot[i, j] = bm.verts.new(p - V((0, 0.0028, 0)))
        for i in range(nu):
            for j in range(nt):
                bm.faces.new((top[i, j], top[i + 1, j], top[i + 1, j + 1], top[i, j + 1]))
                bm.faces.new((bot[i, j + 1], bot[i + 1, j + 1], bot[i + 1, j], bot[i, j]))
        ring = ([(i, 0) for i in range(nu + 1)] + [(nu, j) for j in range(1, nt + 1)]
                + [(i, nt) for i in range(nu - 1, -1, -1)] + [(0, j) for j in range(nt - 1, 0, -1)])
        for k in range(len(ring)):
            p, q = ring[k], ring[(k + 1) % len(ring)]
            bm.faces.new((top[p], bot[p], bot[q], top[q]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return build

def hd_foot(n):
    fr = 'foot.' + n
    mtp = [to_frame(fr, head_w(f'toe{k}0.{n}')) for k in range(5)]
    def build(bm):
        ell(bm, (0, -0.014, -0.006), (0.0165, 0.016, 0.0165))                         # the ankle, inside the shin's end
        ell(bm, (0, -0.03, -0.034), (0.0155, 0.019, 0.026), rot=(-0.5, 0, 0))         # the heel bone, up off the floor
        tube(bm, (0, 0.03, -0.035), (0, -0.016, -0.05), 0.0085, 0.011)                # the Achilles into it
        ell(bm, (0, -0.062, 0.035), (0.026, 0.022, 0.058), rot=(0.62, 0, 0))          # the arch
        for k in range(5):                                                             # the long bones of the foot
            a = V((lerp(-0.011, 0.011, k / 4), -0.024, 0.0))
            tube(bm, a, mtp[k] + V((0, 0.006, -0.006)), 0.0062, 0.0105 if k == 0 else 0.0078, seg=8)
        ell(bm, (0.002, -0.112, 0.1), (0.044, 0.016, 0.024))                          # the ball, 0.09 wide
        ell(bm, (-0.027, -0.11, 0.1), (0.013, 0.0145, 0.016), seg=12)                 # the big toe's joint
        ell(bm, (0.03, -0.116, 0.078), (0.01, 0.012, 0.03), seg=12)                   # the outer edge
    return build

# per finger (little, ring, middle, index, thumb): base radius and nails (one hand broken where the other is grown)
FINGER_R = [0.0076, 0.0086, 0.009, 0.0088, 0.0118]
NAIL_L = ['stub', 'claw', 'claw', 'stub', 'claw']
NAIL_R = ['claw', 'stub', 'claw', 'claw', 'stub']
TOE_R = [0.0105, 0.0078, 0.0072, 0.0066, 0.006]

def hand_pieces(n):
    pcs = [Piece(hd_palm(n), 'hand.' + n, group='fine')]
    nails = NAIL_L if n == 'L' else NAIL_R
    for k in range(5):
        r = FINGER_R[k]
        rad = [(r * 1.06, r * 0.95), (r * 0.95, r * 0.84), (r * 0.84, r * 0.68)] if k < 4 else \
              [(r * 1.08, r * 0.92), (r * 0.92, r * 0.8), (r * 0.8, r * 0.64)]
        for j in range(3):
            b = f'finger{k}{j}.{n}'
            pcs.append(Piece(digit_piece(b, rad[j][0], rad[j][1], j > 0, nails[k] if j == 2 else None), b, group='fine'))
    for k in range(3):                                  # webbed to the second knuckle
        a, b = (f'finger{k}0.{n}',), (f'finger{k + 1}0.{n}',)
        pcs.append(Piece(hd_web(n, a, b, 0.62), 'hand.' + n, [a[0], b[0]], group='fine', sig=0.006))
    a, b = (f'finger41.{n}',), (f'finger30.{n}',)
    pcs.append(Piece(hd_web(n, a, b, 0.45), 'hand.' + n, [f'finger40.{n}', a[0], b[0]], group='fine', sig=0.006))
    return pcs

def foot_pieces(n):
    pcs = [Piece(hd_foot(n), 'foot.' + n, group='fine')]
    for k in range(5):
        r = TOE_R[k]
        pcs.append(Piece(digit_piece(f'toe{k}0.{n}', r, r * 0.88, False, None), f'toe{k}0.{n}', group='fine'))
        pcs.append(Piece(digit_piece(f'toe{k}1.{n}', r * 0.9, r * 0.74, True, 'stub' if (k + (n == 'R')) % 3 == 1 else 'claw'),
                         f'toe{k}1.{n}', group='fine'))
    return pcs


# ------------------------------------------------------------------ pieces ---
def hound_pieces():
    pcs = [Piece(hd_trunk, 'root', ['pelvis', 'lumbar', 'thorax', 'ribs'], sig=0.045),
           Piece(hd_neck, 'neck', ['neck', 'head'], sig=0.03)]
    for n, s in (('L', 1), ('R', -1)):
        pcs.append(Piece(hd_scapula(s), 'root', ['scapula.' + n]))
        pcs.append(Piece(hd_uparm(s), 'upperarm.' + n, ['upperarm.' + n, 'scapula.' + n], sig=0.03))   # the deltoid rides on the blade too
        pcs.append(Piece(hd_armpit(s, n), 'root', ['thorax', 'upperarm.' + n], sig=0.03))
        pcs.append(Piece(hd_fore(s), 'forearm.' + n))
        pcs.append(Piece(hd_thigh(s), 'thigh.' + n))
        pcs.append(Piece(hd_shin(s), 'shin.' + n))
        pcs += hand_pieces(n)
        pcs += foot_pieces(n)
    return pcs


# ------------------------------------------------------------------- paint ---
SKIN, BELLY, BACK = hexc('#7C7771'), hexc('#88827A'), hexc('#6B6764')
BRUISE, BRUISE2, ROT = hexc('#5A5462'), hexc('#6A625F'), hexc('#4E3F4E')
MANGE, SCAB, BLOOD = hexc('#A39882'), hexc('#43302B'), hexc('#4E2420')
DIRT, GRIME, CLAW, NAILY = hexc('#5E5550'), hexc('#4A423C'), hexc('#2B2522'), hexc('#6E6048')
VEIN = hexc('#6C7488')

import random as _random
_r = _random.Random(23)
# scratches: short dragged strokes (world, rest pose), the worst on the flanks and forearms
SCRATCHES = []
def _add_scratches(centre, spread, count, length):
    for _ in range(count):
        c = V(centre) + V((_r.uniform(-1, 1) * spread[0], _r.uniform(-1, 1) * spread[1], _r.uniform(-1, 1) * spread[2]))
        dvec = V((_r.uniform(-0.3, 0.3), _r.uniform(-1, 1), _r.uniform(-1, 1))).normalized()
        L = length * _r.uniform(0.6, 1.3)
        for k in range(_r.choice((1, 3, 4))):                       # claw rakes come in threes and fours
            off = dvec.cross(V((1, 0, 0))).normalized() * (0.006 * k)
            SCRATCHES.append((c + off, c + off + dvec * L))
for s in (-1, 1):
    _add_scratches(rootp((s * 0.14, 0.0, 0.06)), (0.02, 0.05, 0.1), 4, 0.05)      # flanks
    _add_scratches(rootp((s * 0.1, 0.12, -0.3)), (0.04, 0.04, 0.06), 2, 0.04)     # haunches
    m = (head_w('forearm.' + ('L' if s > 0 else 'R')) + tail_w('forearm.' + ('L' if s > 0 else 'R'))) / 2
    _add_scratches(m, (0.02, 0.08, 0.03), 3, 0.035)                                # forearms

def _bone_d(p, b): return seg_dist(p, BONES[b]['head'], BONES[b]['tail'])

def hd_paint(p, n, cav):
    q = p - ROOT_W                                                      # root frame
    up = clamp(n.y * 0.5 + 0.5)
    c = mix3(BELLY, BACK, smooth(0.25, 0.85, up))                       # paler underneath, greyer on the back
    c = mix3(c, SKIN, 0.5)
    # bruising: large cool blotches, purple rot in a few of them
    b = mnoise.noise(p * 9.0 + V((3.1, 1.7, 2.3))) + 0.35 * mnoise.noise(p * 31.0)
    if b < -0.1: c = mix3(c, BRUISE if mnoise.noise(p * 3.0) < 0 else BRUISE2, smooth(-0.1, -0.5, b) * 0.85)
    rot = mnoise.noise(p * 14.0 + V((7.7, 0.4, 5.5)))
    if rot > 0.42: c = mix3(c, ROT, smooth(0.42, 0.62, rot) * 0.75)
    # mange: pale scaly patches, dark scabbed pits in them
    m = mnoise.noise(p * 7.0 + V((1.3, 0.2, 0.7)))
    if m > 0.28:
        k = smooth(0.28, 0.55, m)
        c = mix3(c, MANGE, k * 0.6)
        if mnoise.noise(p * 140.0) > 0.35: c = mix3(c, SCAB, k * 0.7)
    # blue-grey veins on the forearms and shins
    for b_ in ('forearm.L', 'forearm.R', 'shin.L', 'shin.R', 'upperarm.L', 'upperarm.R'):
        if _bone_d(p, b_) < 0.035:
            v = abs(mnoise.noise(V((p.x * 30, p.y * 6, p.z * 30)) + V((2.0, 0.0, 9.0))))
            if v < 0.05: c = mix3(c, VEIN, 0.45 * (1 - v / 0.05))
            break
    # scratches and scabs
    for a, e in SCRATCHES:
        d = seg_dist(p, a, e)
        if d < 0.004:
            c = mix3(c, BLOOD if d < 0.0016 else SCAB, 0.85 * (1 - d / 0.004))
            break
    # grime: whatever touches the floor, worst on the palms and soles; dirty knees and elbows
    dirt = 0.6 * smooth(0.1, 0.02, p.y) + 0.35 * smooth(0.06, 0.0, p.y) * clamp(-n.y * 2)
    for kb in ('shin.L', 'shin.R'):
        dirt = max(dirt, 0.55 * smooth(0.045, 0.015, (p - head_w(kb)).length))
    for kb in ('forearm.L', 'forearm.R'):
        dirt = max(dirt, 0.3 * smooth(0.04, 0.015, (p - head_w(kb)).length))
    dirt *= 0.75 + 0.5 * clamp(mnoise.noise(p * 40.0) + 0.5)
    c = mix3(c, DIRT, clamp(dirt))
    # nails: dark, crusted, yellowed at the broken edges
    for cn, r in NAILS:
        d = (p - cn).length
        if d < r and n.y > -0.35:
            k = smooth(r, r * 0.55, d)
            c = mix3(c, mix3(CLAW, NAILY, clamp(mnoise.noise(p * 300.0) + 0.3)), k)
            break
    # the creases are the only shadow there is; grime gathers in them
    c = mix3(c, GRIME, 0.45 * cav)
    c = mul3(c, (1.0 - 0.62 * cav) * (1 + 0.08 * mnoise.noise(p * 60) + 0.04 * mnoise.noise(p * 230)))
    return c

BODY = dict(name='hdSkin', pieces=hound_pieces, paint=hd_paint,
            body=dict(voxel=0.0028, smooth=(3, 0.5), noise=0.0007, tris=13500),
            fine=dict(voxel=0.0017, smooth=(2, 0.5), noise=0.0003, tris=6500))
