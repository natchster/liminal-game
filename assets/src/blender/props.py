"""
Builds the Backrooms prop parts in Blender.

    /Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
        --python tools/blender/props.py -- [--batch b01,b02] [--parts NAME,...]

Every batch module in tools/blender/props/ (bNN.py) registers parts with
`part(name, frame, build)` (see propkit.py for the conventions). Each built
batch is written to .cache/props/<batch>.json; `node tools/prop-assemble.mjs`
then packs every batch into src/prop-meshes.js, which the game decodes in
src/props.js. A report of triangle counts and frame checks is printed and
saved next to the batch output.
"""
import bpy, sys, os, json, math, base64, struct, importlib.util, time, traceback, random, zlib
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import propkit
from propkit import finish, budget_for

argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
def opt(name, default=None):
    return argv[argv.index(name) + 1] if name in argv else default

# `--out DIR` (the native build: `cargo xtask assets props` passes target/xtask/assets/props)
OUT_DIR = opt('--out') or os.path.join(ROOT, '.cache', 'props')
os.makedirs(OUT_DIR, exist_ok=True)
only_batches = set(filter(None, (opt('--batch') or '').split(',')))
only_parts = set(filter(None, (opt('--parts') or '').split(',')))

# ------------------------------------------------------------- registry ---
REG = {}          # batch -> [(name, spec)]
_current = [None]


def part(name, frame, build, smooth=35.0, weighted=True, flat=False, budget=None):
    """Register one part. `build` is a zero-argument function returning a
    propkit.Shape; it only runs when the part's batch is being built."""
    if not isinstance(name, str) or not name.replace('_', '').isalnum():
        raise ValueError(f'bad part name {name!r} (use letters, digits, _)')
    REG.setdefault(_current[0], []).append((name, dict(frame=frame, build=build, smooth=smooth,
                                                         weighted=weighted, flat=flat, budget=budget)))


propkit.part = part          # batch modules do `from propkit import *`

batch_dir = os.path.join(HERE, 'props')
files = sorted(f for f in os.listdir(batch_dir) if f.endswith('.py') and not f.startswith('_'))
for fn in files:
    b = fn[:-3]
    if only_batches and b not in only_batches:
        continue
    _current[0] = b
    spec = importlib.util.spec_from_file_location('props_' + b, os.path.join(batch_dir, fn))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)

# ------------------------------------------------------------- packing ---
def pack(pos, nrm, tris):
    lo = [min(p[k] for p in pos) for k in range(3)]
    hi = [max(p[k] for p in pos) for k in range(3)]
    v = bytearray()
    for p in pos:
        for k in range(3):
            s = hi[k] - lo[k]
            q = 0 if s < 1e-9 else round((p[k] - lo[k]) / s * 65534 - 32767)
            v += struct.pack('<h', max(-32767, min(32767, q)))
    m = bytearray()
    for n in nrm:
        n = n.normalized() if n.length > 1e-9 else Vector((0, 1, 0))
        for k in range(3):
            m += struct.pack('<b', max(-127, min(127, round(n[k] * 127))))
    ind = bytearray()
    for t in tris:
        for i in t:
            ind += struct.pack('<H', i)
    r6 = lambda a: [round(x, 6) for x in a]
    return dict(n=len(pos), t=len(tris), min=r6(lo), max=r6(hi),
                v=base64.b64encode(bytes(v)).decode(), m=base64.b64encode(bytes(m)).decode(),
                i=base64.b64encode(bytes(ind)).decode())


def check_frame(name, frame, lo, hi):
    """How far the part strays outside / falls short of its frame, relative
    to the frame's size on that axis (0 = exact)."""
    kind, flo, fhi = frame
    big = max(fhi[k] - flo[k] for k in range(3))
    worst = 0.0
    for k in range(3):
        # a thin axis (paper, a plate) may crumple or bow by a fraction of
        # the part's overall size without being "wrong"
        ext = max(fhi[k] - flo[k], 0.25 * big)
        if ext < 1e-9:
            continue
        worst = max(worst, abs(lo[k] - flo[k]) / ext, abs(hi[k] - fhi[k]) / ext)
    return worst


# --------------------------------------------------------------- build ---
t_all = time.time()
grand = {'parts': 0, 'tris': 0, 'errors': 0, 'warnings': 0}
for b, entries in REG.items():
    out, report = {}, []
    t0 = time.time()
    for name, sp in entries:
        if only_parts and name not in only_parts:
            continue
        rec = {'name': name}
        try:
            # every part draws its randomness from a seed of its own name, so a
            # build repeats exactly (the native port's `cargo xtask assets props`
            # compares rebuilds with the committed meshes)
            random.seed(zlib.crc32(name.encode()))
            shape = sp['build']()
            if not shape.v or not shape.f:
                raise ValueError('empty shape')
            pos, nrm, tris = finish(name, shape, sp['smooth'], sp['weighted'], sp['flat'])
            if len(pos) > 65535:
                raise ValueError(f'{len(pos)} vertices (max 65535)')
            if not tris:
                raise ValueError('no triangles')
            d = pack(pos, nrm, tris)
            kind, flo, fhi = sp['frame']
            d['f'] = kind
            d['fmin'] = [round(x, 6) for x in flo]
            d['fmax'] = [round(x, 6) for x in fhi]
            budget = sp['budget'] or budget_for(sp['frame'])
            dev = check_frame(name, sp['frame'], d['min'], d['max'])
            rec.update(tris=d['t'], verts=d['n'], budget=budget, frameDev=round(dev, 3))
            if dev > 0.35:
                rec['error'] = f'part is far outside/inside its frame ({dev:.0%}); wrong frame or orientation?'
            elif dev > 0.10:
                rec['warn'] = f'part deviates {dev:.0%} from its frame'
            if d['t'] > budget * 1.25:
                rec['error'] = f"{d['t']} triangles, budget {budget}"
            elif d['t'] > budget:
                rec['warn'] = f"{d['t']} triangles, budget {budget}"
            if 'error' not in rec:
                out[name] = d
        except Exception as e:
            rec['error'] = f'{type(e).__name__}: {e}'
            rec['trace'] = traceback.format_exc(limit=3)
        report.append(rec)
        grand['parts'] += 1
        grand['tris'] += rec.get('tris', 0)
        grand['errors'] += 'error' in rec
        grand['warnings'] += 'warn' in rec
    if only_parts:
        # merge into the existing batch file so a partial rebuild keeps the rest
        path = os.path.join(OUT_DIR, b + '.json')
        if os.path.exists(path):
            old = json.load(open(path))
            old.update(out)
            out = old
    with open(os.path.join(OUT_DIR, b + '.json'), 'w') as fh:
        json.dump(out, fh, separators=(',', ':'))
    with open(os.path.join(OUT_DIR, b + '.report.json'), 'w') as fh:
        json.dump(report, fh, indent=1)
    errs = [r for r in report if 'error' in r]
    warns = [r for r in report if 'warn' in r]
    tris = sum(r.get('tris', 0) for r in report)
    print(f'[props] {b}: {len(report)} parts, {tris} tris, {len(errs)} errors, {len(warns)} warnings, {time.time() - t0:.1f}s')
    for r in errs:
        print(f"  ERROR {r['name']}: {r['error']}")
        if 'trace' in r:
            print('    ' + r['trace'].strip().replace('\n', '\n    '))
    for r in warns:
        print(f"  warn  {r['name']}: {r['warn']}")
print(f"[props] total: {grand['parts']} parts, {grand['tris']} tris, {grand['errors']} errors, "
      f"{grand['warnings']} warnings, {time.time() - t_all:.1f}s")
