"""
The MakeHuman base body (hm08) and its targets, as a starting point for the
creatures that were once people.

MakeHuman's assets (the base mesh, its targets and modifiers, textures,
poses) were released as CC0 1.0 in September 2020
(https://github.com/makehumancommunity/makehuman, LICENSE.md, section C).
`sources.json` lists every file used with its URL and SHA-1; `fetch`
downloads each once into `target/creature-sources/` and checks it before
anything reads it. The files are data (an OBJ, text targets) and are only
parsed, never run.

The base is in decimetres, +Y up, facing +Z, its left at +X, feet at
y = -8.17: `load` gives it in the game's frame (metres, feet on the floor).
"""
import os, json, hashlib, urllib.request
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
CACHE = os.path.join(ROOT, 'target', 'creature-sources')
SOURCES = json.load(open(os.path.join(HERE, 'sources.json')))
FEET = 0.81676       # metres from the base's origin down to its soles


def fetch(key):
    """the cached, checked file for source `key` of sources.json"""
    s = SOURCES['files'][key]
    os.makedirs(CACHE, exist_ok=True)
    path = os.path.join(CACHE, key.replace('/', '__'))
    if not os.path.exists(path):
        print(f'[creatures] fetching {s["url"]}')
        with urllib.request.urlopen(s['url'], timeout=60) as r:
            data = r.read()
        open(path + '.part', 'wb').write(data)
        os.replace(path + '.part', path)
    h = hashlib.sha1(open(path, 'rb').read()).hexdigest()
    if h != s['sha1']:
        raise RuntimeError(f'{key}: SHA-1 {h}, expected {s["sha1"]} (delete {path} to fetch it again)')
    return path


class Base:
    """the body's skin as quads, every vertex of the full base (helpers too,
    so targets index it), and the joint centres from the joint helpers"""

    def __init__(self):
        V, faces, g, uv, fuv = [], {}, '?', [], {}
        for line in open(fetch('base.obj'), encoding='utf-8'):
            if line.startswith('v '):
                V.append([float(x) for x in line.split()[1:4]])
            elif line.startswith('vt '):
                uv.append([float(x) for x in line.split()[1:3]])
            elif line.startswith('g '):
                g = line.split()[1]
            elif line.startswith('f '):
                toks = line.split()[1:]
                faces.setdefault(g, []).append([int(t.split('/')[0]) - 1 for t in toks])
                fuv.setdefault(g, []).append([int(t.split('/')[1]) - 1 if '/' in t and t.split('/')[1] else -1 for t in toks])
        self.all = np.array(V) * 0.1
        self.all[:, 1] += FEET
        body = faces['body']
        self.used = np.array(sorted(set(i for f in body for i in f)))
        remap = {int(o): i for i, o in enumerate(self.used)}
        self.faces = np.array([[remap[i] for i in f] for f in body])
        # one UV per skin vertex (the first a face gives it; seams are rare on the skin)
        self.uv = np.zeros((len(self.used), 2))
        uv = np.array(uv)
        for f, fu in zip(body, fuv['body']):
            for i, t in zip(f, fu):
                if t >= 0:
                    self.uv[remap[i]] = uv[t]
        self.joint_groups = {k[6:]: np.array(sorted(set(i for f in fs for i in f)))
                             for k, fs in faces.items() if k.startswith('joint-')}
        # the helper meshes (eyes, teeth, ...): faces into the full vertex list
        self.helpers = {k[7:]: fs for k, fs in faces.items() if k.startswith('helper-')}
        self.offsets = np.zeros_like(self.all)

    def target(self, key, weight=1.0):
        """add target `key` (sources.json) at `weight`"""
        for line in open(fetch(key), encoding='utf-8'):
            if not line.strip() or line.startswith('#'):
                continue
            t = line.split()
            self.offsets[int(t[0])] += weight * 0.1 * np.array([float(t[1]), float(t[2]), float(t[3])])
        return self

    def verts(self):
        """the skin's vertices, targets applied (game frame, metres)"""
        return (self.all + self.offsets)[self.used]

    def helper(self, name):
        """helper mesh `name` (e.g. 'upper-teeth', 'l-eye'): its vertices (targets
        applied, game frame) and faces indexed into them"""
        fs = self.helpers[name]
        used = np.array(sorted(set(i for f in fs for i in f)))
        remap = {int(o): i for i, o in enumerate(used)}
        return (self.all + self.offsets)[used], [[remap[i] for i in f] for f in fs]

    def joint(self, name):
        """a joint's centre, targets applied"""
        return (self.all + self.offsets)[self.joint_groups[name]].mean(0)
