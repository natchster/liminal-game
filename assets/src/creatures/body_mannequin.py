"""
The mannequin: the pipeline's test body (a plain figure of metaballs on a
humanoid skeleton). The game uses it for the rig's tests and the creature
bench, never in a level.
"""
import numpy as np

ORDER = 0


def skeleton(kit):
    s = kit.Skel()
    s.add('pelvis', None, (0, 0.95, 0), (0, 1.12, 0))
    s.add('spine', 'pelvis', (0, 1.12, 0), (0, 1.32, 0.01))
    s.add('chest', 'spine', (0, 1.32, 0.01), (0, 1.48, 0.0))
    s.add('neck', 'chest', (0, 1.48, 0.0), (0, 1.58, 0.02))
    s.add('head', 'neck', (0, 1.58, 0.02), (0, 1.78, 0.03))
    s.add('upperarm.L', 'chest', (0.19, 1.44, 0), (0.24, 1.17, 0))
    s.add('forearm.L', 'upperarm.L', (0.24, 1.17, 0), (0.27, 0.92, 0.03))
    s.add('hand.L', 'forearm.L', (0.27, 0.92, 0.03), (0.28, 0.75, 0.04))
    s.add('thigh.L', 'pelvis', (0.1, 0.92, 0), (0.11, 0.5, 0.01))
    s.add('shin.L', 'thigh.L', (0.11, 0.5, 0.01), (0.11, 0.08, -0.02))
    s.add('foot.L', 'shin.L', (0.11, 0.08, -0.02), (0.11, 0.02, 0.16))
    s.mirror()
    return s


def build(kit):
    s = skeleton(kit)
    el = []
    for b in s.bones:
        h, t = np.array(b['head']), np.array(b['tail'])
        r = {'pelvis': 0.13, 'spine': 0.12, 'chest': 0.15, 'neck': 0.05, 'head': 0.1}.get(b['name'].split('.')[0], 0.05)
        if b['name'].startswith(('thigh', 'shin')):
            r = 0.065
        el.append(('capsule', h, t, r, 2.0))
    o = kit.metaballs('mannequin', el, resolution=0.02)
    kit.remesh(o, 0.012)
    kit.smooth(o, 3, 0.5)
    kit.decimate(o, 6000)
    P = kit.to_game_np(kit.verts_np(o))
    grey = 0.45 + 0.08 * kit.fbm(P, 6.0, 3, seed=7)
    cols = np.stack([grey, grey * 0.97, grey * 0.93], axis=1)
    body = kit.Body('mannequin', s)
    body.skinned('skin', 'skin', o, colors=cols)
    return body
