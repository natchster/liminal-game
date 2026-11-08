"""
The Smiler's skinned body (run by skinned.py, which provides
C = creatures.py's helpers and builders, Piece, V, BONES, math, bpy).

Starved, sinewy and wrong, not a mannequin: a ribcage of uneven ribs that
flare out over a belly sucked in under the costal arch, a deep hollow down
the sternum, clavicle bars over hollows, winged shoulder blades, a crooked
row of spine knobs, iliac crests and hip points; stringy muscle and cords on
long limbs, knobbly knees, elbows and wrists, and long bony hands with
swollen knuckles and long claws. Slightly asymmetric (ribs, shoulder blades,
the spine). Every limb is built in its bone's rest frame (bone at the
origin, hanging -Y, +Z forward; s = +1 left, -1 right, medial is -s * x).
The finger numbers are its rig's.
"""
import random
ell, tube, chain, sweep = C['ell'], C['tube'], C['chain'], C['sweep']
smooth, lerp, clamp, interp, spow = C['smooth'], C['lerp'], C['clamp'], C['interp'], C['spow']


def solid(build, ops):
    """a piece builder whose geometry goes through creatures.py's ops first
    (remesh, then boolean cuts: hollows the voxel fuse cannot make)"""
    def fn(bm):
        me, _ = C['bm_mesh']('smb_tmp', build)
        me = C['run_ops']('smb_tmp', me, ops)
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    return fn


# ------------------------------------------------------------------ torso ---
# pelvis frame: the hip joints at y = 0, the clavicles at about 0.67 (the rig's
# shoulder is on this torso), +Z forward. Stations: y, half-width, front
# z, back z. The belly is sucked in under a flaring ribcage.
TORSO = [
    (-0.10, 0.042, 0.030, -0.040), (-0.06, 0.095, 0.048, -0.072), (0.0, 0.116, 0.054, -0.082),
    (0.05, 0.12, 0.05, -0.082), (0.10, 0.1, 0.03, -0.074), (0.15, 0.076, 0.018, -0.066),
    (0.21, 0.072, 0.02, -0.066), (0.26, 0.098, 0.04, -0.076), (0.30, 0.126, 0.066, -0.088),
    (0.36, 0.138, 0.084, -0.098), (0.44, 0.146, 0.094, -0.106), (0.52, 0.148, 0.098, -0.11),
    (0.585, 0.144, 0.092, -0.108), (0.635, 0.158, 0.076, -0.096), (0.675, 0.138, 0.054, -0.074),
    (0.715, 0.08, 0.04, -0.052), (0.745, 0.04, 0.03, -0.032),
]
def hunch(y): return 0.05 * smooth(0.3, 0.75, y) ** 1.3      # the upper body rolls forward

def tpt(y, a, out=0.0, yref=None):
    """the torso's surface at height y, angle a round it from the front (+X
    at a = +pi/2), pushed out along the section's normal by `out`. yref
    takes the section's size from another height (ribs flaring out)."""
    w, zf, zb = interp(TORSO, y if yref is None else yref)
    sa, ca = math.sin(a), math.cos(a)
    e = 2 / 2.3
    x, z = w * spow(sa, e), (zf if ca >= 0 else -zb) * spow(ca, e)
    nrm = V((x / (w * w), 0, z / ((zf if ca >= 0 else -zb) ** 2)))
    nrm = nrm.normalized() if nrm.length > 1e-9 else V((0, 0, 1))
    return V((x, y, z + hunch(y))) + nrm * out

def torso_build(bm):
    rng = random.Random(5)
    C['loft_y'](bm, TORSO, n=64, zshift=hunch)
    # ribs: ten a side, each its own: uneven spacing, slope and thickness, the
    # two sides not quite alike. They run from beside the spine round the
    # side and drop to the front: the upper ones to the sternum, the lower
    # ones to the costal arch, the last two floating.
    for s in (-1, 1):
        y0 = 0.61 + (0.006 if s < 0 else 0.0)
        for i in range(10):
            y0 -= 0.031 + rng.uniform(-0.007, 0.008)
            a1 = (0.16, 0.17, 0.18, 0.2, 0.24, 0.42, 0.66, 0.92, 1.45, 1.85)[i] + rng.uniform(-0.03, 0.03)
            drop = min(0.055 + 0.011 * i + rng.uniform(-0.01, 0.01), y0 - 0.272)   # the lowest stay on the flank
            r0 = (0.0062, 0.0072, 0.0082, 0.0088, 0.009, 0.009, 0.0088, 0.0084, 0.0078, 0.007)[i] * rng.uniform(0.9, 1.12)
            pts, rs, n = [], [], 16
            wob, ph = rng.uniform(0.001, 0.0035), rng.uniform(0, 6.3)
            for j in range(n + 1):
                t = j / n
                a = lerp(2.62, a1, t)
                y = y0 + 0.012 * math.sin(math.pi * min(1.0, t * 2.4)) - drop * smooth(0.12, 1.0, t) ** 1.15
                if i < 7: y += (0.006 + 0.007 * i) * smooth(0.74, 1.0, t)      # the cartilage turns up to the sternum
                y += wob * math.sin(t * 9.0 + ph)                                # and no rib is quite straight
                out = 0.0025 + 0.004 * math.sin(math.pi * t) - 0.003 * smooth(0.7, 1.0, t) * (i < 5)
                pts.append(tpt(y, s * a, out, yref=max(y, 0.292)))
                rs.append(r0 * lerp(1.0, 0.62, smooth(0.62, 1.0, t)))
            sweep(bm, pts, rs, sides=7)
            if i < 6:   # a knob where the rib turns to cartilage
                ell(bm, pts[13], (r0 * 1.25,) * 3, seg=10)
    # the costal arch: the cartilage edge of the ribcage over the sunken belly
    for s in (-1, 1):
        pts = [tpt(lerp(0.415, 0.3, t) - 0.012 * t * t, s * lerp(0.13, 1.25, t ** 0.8), 0.003, yref=max(0.3, lerp(0.415, 0.3, t)))
               for t in [k / 10 for k in range(11)]]
        sweep(bm, pts, [0.0075] * 10 + [0.005], sides=8)
    # the sternum: two raised edges either side of a deep hollow (the hollow is cut)
    for s in (-1, 1):
        sweep(bm, [tpt(y, s * 0.13, 0.002) for y in (0.655, 0.6, 0.53, 0.46, 0.415)], [0.008, 0.009, 0.0085, 0.0075, 0.006], sides=8)
    ell(bm, tpt(0.4, 0.0, -0.002), (0.012, 0.016, 0.009), seg=10)          # xiphoid point
    # spine: a crooked row of knobs, sunk between the erector cords low down,
    # standing proud over the hunched upper back
    for k in range(19):
        y = lerp(0.01, 0.725, k / 18)
        big = smooth(0.26, 0.45, y)
        dx = 0.004 * math.sin(y * 9.0) + rng.uniform(-0.0015, 0.0015)
        p = tpt(y, math.pi, lerp(-0.004, 0.003, big)) + V((dx, 0, 0))
        r = lerp(0.01, 0.013, big) * rng.uniform(0.9, 1.1) * (1.25 if k == 18 else 1.0)
        ell(bm, p, (r, r * 0.8, r * 1.2), seg=10)
    for s in (-1, 1):   # the erector cords either side of the lower spine
        sweep(bm, [tpt(y, s * (math.pi - 0.32), o) for y, o in ((0.0, -0.004), (0.1, 0.0), (0.22, 0.002), (0.34, 0.0), (0.44, -0.006))],
              [0.01, 0.016, 0.017, 0.013, 0.006], sides=8)
    # shoulder blades: winged plates, the left higher and further out
    for s in (-1, 1):
        dy, wing = (0.012, 0.012) if s > 0 else (0.0, 0.008)
        sup, inf, acr = tpt(0.65 + dy, s * 2.62, 0.006), tpt(0.42 + dy, s * 2.38, 0.008 + wing), V((s * 0.158, 0.682 + dy * 0.5, -0.03 + hunch(0.68)))
        sweep(bm, [sup, tpt(0.53 + dy, s * 2.62, 0.008 + wing), inf], [0.008, 0.009, 0.007], sides=8)        # medial border, lifted
        sweep(bm, [inf, tpt(0.5 + dy, s * 2.05, 0.004 + wing * 0.5), tpt(0.6 + dy, s * 1.85, 0.002)], [0.007, 0.007, 0.005], sides=8)   # lateral border
        sweep(bm, [tpt(0.6 + dy, s * 2.6, 0.01 + wing), tpt(0.625 + dy, s * 2.2, 0.01 + wing * 0.5), acr], [0.007, 0.009, 0.011], sides=8)   # the spine of it
        ell(bm, tpt(0.55 + dy, s * 2.4, 0.002 + wing * 0.5), (0.034, 0.06, 0.009), rot=(0.15, s * 0.95, s * 0.25))   # the plate
        ell(bm, inf, (0.011, 0.011, 0.01), seg=10)
    # clavicles: S-bent bars from the notch to the shoulder point
    for s in (-1, 1):
        pts = [tpt(0.668, s * 0.14, 0.004), tpt(0.674, s * 0.42, 0.012), tpt(0.68, s * 0.78, 0.01),
               tpt(0.685, s * 1.1, 0.006), V((s * 0.158, 0.69, -0.012 + hunch(0.69)))]
        sweep(bm, pts, [0.011, 0.0095, 0.009, 0.0095, 0.012], sides=8)
        ell(bm, pts[0], (0.012, 0.011, 0.011), seg=10)
        ell(bm, V((s * 0.166, 0.683, -0.016 + hunch(0.683))), (0.018, 0.013, 0.02), seg=12)              # acromion
        # the trapezius, hiked up to the neck: a thin sloping web
        sweep(bm, [V((s * 0.024, 0.75, -0.022 + hunch(0.75))), V((s * 0.08, 0.725, -0.03 + hunch(0.725))), V((s * 0.15, 0.69, -0.028 + hunch(0.69)))],
              [0.016, 0.017, 0.012], sides=8, flat=0.55)
        # the pelvis: an iliac crest round to a sharp hip point, a bony knob at the hip joint
        sweep(bm, [tpt(y, s * a, 0.002) for y, a in ((0.075, 2.55), (0.1, 2.05), (0.105, 1.55), (0.085, 1.0), (0.05, 0.72))],
              [0.009, 0.012, 0.012, 0.011, 0.012], sides=8)
        ell(bm, tpt(0.045, s * 0.7, 0.004), (0.016, 0.016, 0.014), seg=12)                                  # hip point (ASIS)
        ell(bm, V((s * 0.12, -0.012, -0.004)), (0.026, 0.032, 0.03), seg=14)                               # greater trochanter
        # serratus fingers on the side of the ribcage, between the ribs
        for k in range(4):
            y = 0.5 - k * 0.042 + (0.006 if s > 0 else 0)
            ell(bm, tpt(y, s * (1.05 + 0.05 * k), 0.002), (0.012, 0.009, 0.02), rot=(0.4, s * 1.1, 0), seg=10)
    tube(bm, (0, 0.63, hunch(0.63)), (0, 0.76, 0.02 + hunch(0.76)), 0.036, 0.032)                         # neck root

def torso_cuts(bm):
    ell(bm, tpt(0.235, 0.0, 0.03), (0.075, 0.1, 0.058))                 # the belly, sucked in under the costal arch
    ell(bm, tpt(0.535, 0.0, 0.006), (0.022, 0.135, 0.016))              # the sternum hollow
    ell(bm, tpt(0.0, 0.0, 0.018), (0.07, 0.05, 0.03))                    # the lower belly between the hip points
    for s in (-1, 1):
        ell(bm, tpt(0.705, s * 0.6, 0.012), (0.035, 0.024, 0.024))       # hollows over the clavicles
        ell(bm, tpt(0.635, s * 0.75, 0.01), (0.034, 0.016, 0.016))       # and under them
        ell(bm, tpt(0.06, s * 0.5, 0.012), (0.03, 0.04, 0.02))           # inside the hip points
        ell(bm, tpt(0.48, s * 2.75, 0.012), (0.018, 0.09, 0.02))         # between shoulder blade and spine
        ell(bm, tpt(0.58, s * 1.55, 0.016), (0.03, 0.05, 0.024))         # the armpit


# ------------------------------------------------------------------- legs ---
# A human leg on tiptoe, starved: thigh (hip -> knee 0.56), shin (knee -> hock
# 0.56), the long foot (hock -> ball 0.30), toes. The kneecap faces +Z.
def thigh(s):
    m = -s
    def fn(bm):
        tube(bm, (0, 0.03, 0), (0, -0.53, 0.004), 0.034, 0.024)                        # the femur's core
        ell(bm, (m * 0.01, -0.04, -0.004), (0.05, 0.07, 0.048))                         # the hip, into the pelvis
        # wasted quadriceps: a band down the front, the outer one, a teardrop over the knee
        sweep(bm, [(0, -0.05, 0.026), (0.002, -0.2, 0.036), (0, -0.38, 0.03), (0, -0.5, 0.026)], [0.013, 0.018, 0.014, 0.009], sides=8)
        sweep(bm, [(-m * 0.028, -0.08, 0.006), (-m * 0.035, -0.24, 0.01), (-m * 0.03, -0.41, 0.012), (-m * 0.02, -0.51, 0.01)],
              [0.012, 0.017, 0.013, 0.007], sides=8)
        ell(bm, (m * 0.024, -0.455, 0.02), (0.015, 0.042, 0.015))
        # hamstrings: two cords at the back, parting into tendons either side of the knee
        for k, side in enumerate((m, -m)):
            sweep(bm, [(0, -0.07, -0.028), (side * 0.01, -0.25, -0.034), (side * 0.024, -0.46, -0.028), (side * 0.03, -0.56, -0.016)],
                  [0.015, 0.016 - 0.002 * k, 0.008, 0.0055], sides=8)
        # the sartorius strap, from the outside of the hip across to the inside of the knee
        sweep(bm, [(-m * 0.04, -0.01, 0.028), (-m * 0.008, -0.18, 0.04), (m * 0.028, -0.38, 0.022), (m * 0.034, -0.52, -0.004)],
              [0.006, 0.0075, 0.0065, 0.005], sides=8)
        ell(bm, (m * 0.028, -0.13, -0.006), (0.02, 0.09, 0.028))                       # adductors, high inside
        # the knee: condyles either side, the cap, the tendon under it
        ell(bm, (m * 0.025, -0.552, -0.006), (0.021, 0.03, 0.028))
        ell(bm, (-m * 0.022, -0.552, -0.006), (0.019, 0.028, 0.026))
        ell(bm, (0, -0.532, 0.03), (0.02, 0.024, 0.012), rot=(0.3, 0, 0))
    return fn

def shin(s):
    m = -s
    def fn(bm):
        tube(bm, (0, 0.0, 0), (0, -0.55, 0), 0.02, 0.013)
        tube(bm, (0, -0.03, 0.016), (0, -0.52, 0.011), 0.0085, 0.006, seg=10)           # the shin bone's edge
        ell(bm, (0, -0.06, 0.022), (0.012, 0.016, 0.01))                                # tuberosity under the kneecap tendon
        tube(bm, (0, 0.0, 0.024), (0, -0.06, 0.022), 0.008, 0.008, seg=10)              # the tendon
        ell(bm, (-m * 0.022, -0.05, -0.006), (0.009, 0.01, 0.009))                     # head of the fibula
        for side in (1, -1):                                                            # a wasted calf: two thin heads
            ell(bm, (side * 0.011, -0.13, -0.02), (0.014, 0.075, 0.015))
        sweep(bm, [(0, -0.17, -0.026), (0, -0.3, -0.031), (0, -0.45, -0.032), (0, -0.54, -0.03)],
              [0.009, 0.0068, 0.0058, 0.0062], sides=8)                                  # the Achilles cord, standing off the bone
        for side, r in ((m, 0.0115), (-m, 0.01)):                                       # ankle knobs
            ell(bm, (side * 0.018, -0.548, 0.002), (r, r * 1.05, r))
    return fn

def meta(s):
    def fn(bm):
        tube(bm, (0, 0.0, 0), (0, -0.29, 0.0), 0.018, 0.0135)
        ell(bm, (0, 0.016, -0.03), (0.019, 0.028, 0.024))                               # the heel, sticking back
        for k, x in enumerate((-0.013, -0.004, 0.005, 0.014)):                          # long foot bones on the top
            tube(bm, (x * 0.6, -0.03, 0.01), (x * 1.6, -0.28, 0.012), 0.0055, 0.0068, seg=8)
        tube(bm, (0, -0.02, 0.018), (0, -0.25, 0.017), 0.004, 0.004, seg=8)            # an extensor tendon
        ell(bm, (0, -0.3, 0.004), (0.031, 0.024, 0.028))                               # the ball of the foot
    return fn

def toes(s):
    """ball joint at the origin, floor at y = -0.025; four long knuckled toes
    with hooked claws"""
    def fn(bm):
        ell(bm, (0, -0.007, 0.008), (0.035, 0.017, 0.03))
        for k in range(4):
            x = -0.027 + k * 0.018
            L = 0.09 - abs(k - 1.5) * 0.012
            p0, p1, p2 = V((x, -0.002, 0.02)), V((x * 1.25, -0.002, 0.02 + L * 0.55)), V((x * 1.35, -0.013, 0.02 + L))
            chain(bm, [p0, p1, p2], [0.0085, 0.0068, 0.0058], seg=8)
            ell(bm, p1, (0.0088, 0.0092, 0.0088), seg=8)
            sweep(bm, [p2, p2 + V((0, -0.002, 0.016)), p2 + V((0, -0.008, 0.03)), p2 + V((0, -0.016, 0.038))],
                  [0.005, 0.0042, 0.0025, 0.0], sides=6)                                   # claw
    return fn


# ------------------------------------------------------------------- arms ---
# upper arm (shoulder -> elbow 0.48), forearm (elbow -> wrist 0.46). In the
# forearm the thumb side is +Z, the palm side medial.
def uparm(s):
    m = -s
    def fn(bm):
        tube(bm, (0, 0.015, 0), (0, -0.46, 0), 0.022, 0.017)
        ell(bm, (0, -0.035, 0), (0.033, 0.055, 0.032))                                  # a wasted deltoid
        sweep(bm, [(0, -0.03, 0), (-m * 0.013, -0.11, 0.002), (-m * 0.016, -0.19, 0.0)], [0.026, 0.017, 0.005], sides=8)
        sweep(bm, [(0, -0.08, 0.014), (0, -0.21, 0.023), (0, -0.37, 0.018), (0, -0.465, 0.012)],
              [0.008, 0.015, 0.0095, 0.0055], sides=8)                                   # biceps, stringy
        sweep(bm, [(0, -0.06, -0.02), (-m * 0.004, -0.2, -0.026), (0, -0.35, -0.021), (0, -0.46, -0.022)],
              [0.012, 0.016, 0.0105, 0.0075], sides=8)                                   # triceps
        sweep(bm, [(m * 0.016, -0.12, -0.006), (m * 0.019, -0.3, -0.004), (m * 0.014, -0.44, -0.004)], [0.005, 0.006, 0.0045], sides=8)   # a cord inside
        ell(bm, (m * 0.027, -0.468, -0.006), (0.013, 0.014, 0.012))                    # epicondyles: the inner one bigger
        ell(bm, (-m * 0.022, -0.468, -0.004), (0.0095, 0.011, 0.01))
        ell(bm, (0, -0.485, -0.025), (0.0135, 0.019, 0.0135))                           # the elbow's point
    return fn

def forearm(s):
    m = -s
    def fn(bm):
        tube(bm, (0, 0.0, 0), (0, -0.45, 0), 0.017, 0.0125)
        sweep(bm, [(-m * 0.008, 0.05, 0.012), (0, -0.06, 0.025), (0, -0.2, 0.019), (0, -0.31, 0.011)],
              [0.009, 0.018, 0.011, 0.005], sides=8)                                     # brachioradialis, thumb side
        ell(bm, (m * 0.016, -0.1, -0.004), (0.016, 0.085, 0.017))                      # flexors (palm side) ...
        ell(bm, (-m * 0.015, -0.095, 0.004), (0.014, 0.08, 0.015))                     # ... and extensors, high and thin
        for k, z in enumerate((-0.009, 0.0, 0.009)):                                    # running out into tendons
            tube(bm, (m * 0.013, -0.17, z), (m * 0.012, -0.44, z * 0.8), 0.0045, 0.0038, seg=8)
            tube(bm, (-m * 0.012, -0.17, z * 0.8), (-m * 0.011, -0.44, z * 0.7), 0.004, 0.0035, seg=8)
        tube(bm, (0, -0.02, -0.019), (0, -0.43, -0.014), 0.0062, 0.0055, seg=10)       # the ulna's edge
        ell(bm, (0, -0.448, -0.017), (0.0085, 0.009, 0.0085))                          # wrist knobs
        ell(bm, (0, -0.452, 0.015), (0.009, 0.0085, 0.009))
        ell(bm, (0, -0.455, 0), (0.019, 0.013, 0.018))
    return fn


# ------------------------------------------------------------------- neck ---
def neck(bm):
    """neck frame: base at the origin, up +Y to the head at 0.3 (it runs on
    into the skull). Cords, windpipe, a knot of larynx, the cervical knobs."""
    tube(bm, (0, -0.05, 0), (0, 0.27, -0.002), 0.033, 0.028)
    for s in (-1, 1):
        sweep(bm, [V((s * 0.05, 0.27, -0.022)), V((s * 0.038, 0.16, 0.008)), V((s * 0.022, 0.04, 0.032)), V((s * 0.012, -0.03, 0.042))],
              [0.0095, 0.0105, 0.0105, 0.009], sides=8)                                  # the cords from the skull to the notch
        sweep(bm, [V((s * 0.014, 0.26, -0.026)), V((s * 0.018, 0.12, -0.03)), V((s * 0.03, -0.03, -0.03))],
              [0.008, 0.009, 0.011], sides=8)                                            # nape cords
    tube(bm, (0, -0.03, 0.026), (0, 0.2, 0.03), 0.011, 0.01, seg=10)                   # windpipe
    for k in range(4): ell(bm, (0, 0.0 + k * 0.035, 0.035), (0.012, 0.0045, 0.007), seg=8)
    ell(bm, (0, 0.175, 0.037), (0.013, 0.018, 0.012))                                   # larynx
    for k in range(6): ell(bm, (0, -0.03 + k * 0.05, -0.028), (0.011, 0.013, 0.012), seg=8)   # cervical knobs


# ------------------------------------------------------------------ hands ---
# palm frame: wrist at the origin, hanging -Y, palm +Z, thumb +X (mirrored
# for the right hand by its frame). Four long fingers of three bones, curled
# a little at rest; the bone positions are the rig's.
FING_X, FING_SC = (-0.03, -0.01, 0.01, 0.03), (0.9, 1.0, 0.97, 0.84)
FING_L, FING_CURL = (0.08, 0.07, 0.056), (0.32, 0.5, 0.45)
def finger_pts(k):
    x, sc = FING_X[k], FING_SC[k]
    p, ang, pts, angs = V((x, -0.112, 0.0)), 0.0, [], []
    pts.append(p.copy())
    for j in range(3):
        ang += FING_CURL[j]
        p = p + V((x * 0.35, -math.cos(ang), math.sin(ang))).normalized() * FING_L[j] * sc
        pts.append(p.copy()); angs.append(ang)
    return pts, angs

def claw(bm, base, ang, length, r):
    """a long claw hooking towards the palm"""
    pts, p, a = [base], base.copy(), ang
    for t in (0.36, 0.34, 0.3):
        a += 0.32
        p = p + V((0, -math.cos(a), math.sin(a))) * length * t
        pts.append(p.copy())
    sweep(bm, pts, [r, r * 0.8, r * 0.45, 0.0], sides=7)

def smiler_hand_pieces(s):
    n = 'L' if s > 0 else 'R'
    frame = 'palm.' + n
    pcs = []
    def palm(bm):
        ell(bm, (0, -0.056, 0.002), (0.04, 0.06, 0.0115))                              # a flat, bony palm
        ell(bm, (0, -0.008, 0), (0.027, 0.018, 0.016))                                  # wrist bones
        ell(bm, (0.028, -0.042, 0.011), (0.019, 0.034, 0.013), rot=(0, 0, -0.3))        # thumb pad
        ell(bm, (-0.033, -0.065, 0.006), (0.011, 0.034, 0.01))                          # the outer edge
        for x in FING_X:                                                                # bones and tendons on the back
            tube(bm, (x * 0.5, -0.015, -0.009), (x, -0.106, -0.0075), 0.0052, 0.0064, seg=8)
    pcs.append(Piece(palm, frame, ['palm.' + n], group='fine'))
    for k in range(4):
        pts, angs = finger_pts(k)
        radii = (0.0088, 0.0077, 0.0067, 0.0057)
        for j in range(3):
            def ph(bm, j=j, pts=pts, angs=angs, radii=radii):
                tube(bm, pts[j], pts[j + 1], radii[j], radii[j + 1], seg=10)
                a = angs[j] - FING_CURL[j] * 0.5
                back = V((0, -math.sin(a), -math.cos(a)))                               # knuckles swell on the back
                kr = (0.0128, 0.0118, 0.0102)[j]
                ell(bm, pts[j] + back * 0.0025, (kr * 1.05, kr, kr), seg=10)
                if j == 2: claw(bm, pts[3], angs[2], 0.05, 0.0058)
            pcs.append(Piece(ph, frame, [f'finger{k}{j}.{n}'], group='fine'))
    t0, t1 = V((0.04, -0.055, 0.016)), V((0.05, -0.1, 0.034))
    t2 = t1 + V((0.002, -0.042, 0.02))
    def th0(bm):
        tube(bm, t0, t1, 0.0115, 0.009, seg=10); ell(bm, t1, (0.0112, 0.0112, 0.0112), seg=10)
    def th1(bm):
        tube(bm, t1, t2, 0.009, 0.007, seg=10); claw(bm, t2, 0.45, 0.04, 0.0055)
    pcs.append(Piece(th0, frame, [f'finger40.{n}'], group='fine'))
    pcs.append(Piece(th1, frame, [f'finger41.{n}'], group='fine'))
    return pcs


def smiler_pieces():
    pcs = []
    for n, s in (('L', 1), ('R', -1)):
        pcs.append(Piece(thigh(s), 'thigh.' + n))
        pcs.append(Piece(shin(s), 'shin.' + n))
        pcs.append(Piece(meta(s), 'meta.' + n))
        pcs.append(Piece(toes(s), 'toes.' + n, group='fine'))
        pcs.append(Piece(uparm(s), 'upperarm.' + n))
        pcs.append(Piece(forearm(s), 'forearm.' + n))
        pcs += smiler_hand_pieces(s)
    pcs.append(Piece(solid(torso_build, [('remesh', 0.003), ('cut', torso_cuts)]), 'pelvis', ['pelvis', 'abdomen', 'chest'], sig=0.05))
    pcs.append(Piece(neck, 'neck', ['neck', 'head'], sig=0.03))
    return pcs


# ------------------------------------------------------------------ paint ---
SM_CLAW = C['hexc']('#2b2621')
def _bone(name): return V(BONES[name]['head']), V(BONES[name]['tail'])
TIPS = []      # (fingertip, direction): beyond a tip is claw
for _n in ('L', 'R'):
    for _k in range(4):
        _h, _t = _bone(f'finger{_k}2.{_n}'); TIPS.append((_t, (_t - _h).normalized()))
    _h, _t = _bone(f'finger41.{_n}'); TIPS.append((_t, (_t - _h).normalized()))
WORN = [V(BONES[b]['head']) for b in BONES if b.split('.')[0] in ('shin', 'forearm', 'meta')
        or (b.startswith('finger') and b[7] in '12')]
TOES = [V(BONES['toes.' + n]['head']) for n in ('L', 'R')]

def smiler_paint(p, n, cav):
    c = C['sm_skin_at'](p)
    for tip, d in TIPS:                                     # claws: dark horn
        if (p - tip).length < 0.055 and (p - tip).dot(d) > -0.003:
            return C['mul3'](SM_CLAW, 1 - 0.3 * cav)
    for t in TOES:
        if (p - t).length < 0.16 and p.z - t.z > 0.1:
            return C['mul3'](SM_CLAW, 1 - 0.3 * cav)
    worn = max([1 - smooth(0.012, 0.034, (p - w).length) for w in WORN] + [0.0])
    c = C['mix3'](c, C['SM_DRY'], 0.55 * worn)              # knees, elbows, knuckles, heels: dry, grey, worn
    k = 1.0 - 0.62 * cav                                     # creases dark (the only occlusion there is)
    return tuple(ch * k for ch in c)

BODY = dict(name='smBody', pieces=smiler_pieces, paint=smiler_paint,
            body=dict(voxel=0.003, smooth=(3, 0.5), noise=0.0009, tris=16000),
            fine=dict(voxel=0.0021, smooth=(2, 0.5), noise=0.0004, tris=5800))
