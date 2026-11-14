"""
Builds the Backrooms prop parts in Blender.

    cargo xtask assets props [--batch domestic,showroom] [--blend]   # runs this, packs and compares
    /Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
        --python assets/src/blender/props.py -- [--batch FAMILY|THEME,...] [--parts NAME,...] [--out DIR]

Every family in assets/src/blender/props/<theme>/ (`domestic/kitchen_hall_and_floor_things.py`)
registers parts with `part(name, frame, build)` (the batches of real things
through an `auto(name, build, budget)` of their own; see propkit.py for the
conventions). Each built batch is written
to <out>/<batch>.json (by default .cache/props/); `cargo xtask assets props`
packs every part into crates/game/assets/prop_meshes.bin, which the game
decodes in `assets::props`. A report of triangle counts and frame checks is
printed and saved next to the batch output.
"""
import bpy, sys, os, json, math, base64, struct, importlib.util, time, traceback, random, zlib, collections
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
# `--blend DIR`: also a .blend per theme to look at in Blender (write_blends)
BLEND_DIR = opt('--blend')

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

# a family of props modelled together, in the folder of the theme its parts
# serve (props/<theme>/<family>.py): built and written as one batch, named by
# its file; `--batch` takes families or whole themes
batch_dir = os.path.join(HERE, 'props')
files = []
for dp, dns, fns in os.walk(batch_dir):
    dns[:] = sorted(d for d in dns if not d.startswith(('_', '.')))
    files += [os.path.join(dp, f) for f in fns if f.endswith('.py') and not f.startswith('_')]
files.sort(key=lambda p: os.path.relpath(p, batch_dir))
names = [os.path.basename(p)[:-3] for p in files]
dup = sorted({n for n in names if names.count(n) > 1})
if dup:
    print(f'[props] error: two families of one name: {dup} (a family is named by its file)')
    raise SystemExit(f'two families of one name: {dup}')
themes = {os.path.basename(os.path.dirname(p)) for p in files}
unknown = sorted(only_batches - set(names) - themes)
if unknown:
    msg = (f'--batch {",".join(unknown)}: no family or theme of that name (props/<theme>/<family>.py; '
           f'an old batch name, b07 or t13, is in the "Was" column of props/README.md)')
    print(f'[props] error: {msg}')
    raise SystemExit(msg)
THEME_OF = {}
for path in files:
    b = os.path.basename(path)[:-3]
    theme = os.path.basename(os.path.dirname(path))
    THEME_OF[b] = theme if theme != 'props' else 'selftest'
    if only_batches and b not in only_batches and theme not in only_batches:
        continue
    _current[0] = b
    spec = importlib.util.spec_from_file_location('props_' + b, path)
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
# ------------------------------------------------------------- the props in Blender ---
def write_blends(out_dir, parts):
    """A .blend per theme, to open and look at in Blender: a collection per
    family, each part an object named as the game names it, laid out in a
    row per family; every part marked as an asset (tags: its theme, its
    family, its prop), its catalog the theme and the family, so the folder
    is an asset library (Preferences > File Paths > Asset Libraries).
    Built from the finished meshes after every part is packed, in
    collections no scene holds: nothing here touches what the game gets."""
    import uuid
    os.makedirs(out_dir, exist_ok=True)
    cats = {}

    def catalog(path):
        if path not in cats:
            cats[path] = str(uuid.uuid5(uuid.NAMESPACE_URL, 'backrooms-props/' + path))
        return cats[path]

    for theme, items in sorted(parts.items()):
        root = bpy.data.collections.new(theme)
        by_family = collections.OrderedDict()
        for fam, name, pos, nrm, tris in items:
            by_family.setdefault(fam, []).append((name, pos, nrm, tris))
        row_y = 0.0
        for fam, ps in by_family.items():
            col = bpy.data.collections.new(fam)
            root.children.link(col)
            cid = catalog(f'{theme}/{fam}')
            x, depth = 0.0, 0.0
            for name, pos, nrm, tris in ps:
                # the game's +Y up is Blender's +Z up
                verts = [(p[0], -p[2], p[1]) for p in pos]
                me = bpy.data.meshes.new(name)
                me.from_pydata(verts, [], [tuple(t) for t in tris])
                me.update()
                try:
                    me.normals_split_custom_set_from_vertices([(n[0], -n[2], n[1]) for n in nrm])
                except (AttributeError, RuntimeError):
                    pass
                ob = bpy.data.objects.new(name, me)
                xs = [v[0] for v in verts] or [0.0]
                ys = [v[1] for v in verts] or [0.0]
                w, d = max(xs) - min(xs), max(ys) - min(ys)
                ob.location = (x - min(xs), row_y - min(ys), 0.0)
                x += w + 0.15
                depth = max(depth, d)
                col.objects.link(ob)
                ob.asset_mark()
                ob.asset_data.catalog_id = cid
                prop = name.split('__')[0]
                for tag in (theme, fam, prop):
                    ob.asset_data.tags.new(tag, skip_if_exists=True)
            row_y += depth + 0.6
        path = os.path.join(out_dir, theme + '.blend')
        bpy.data.libraries.write(path, {root}, fake_user=True)
        print(f'[props] {path}: {len(by_family)} families, {len(items)} parts')
    with open(os.path.join(out_dir, 'blender_assets.cats.txt'), 'w') as fh:
        fh.write('# The props by theme and family (written by props.py --blend)\nVERSION 1\n\n')
        for path, cid in sorted(cats.items()):
            fh.write(f'{cid}:{path}:{path.replace("/", "-")}\n')


grand = {'parts': 0, 'tris': 0, 'errors': 0, 'warnings': 0}
blend_parts = collections.defaultdict(list)   # theme -> [(family, part, pos, nrm, tris)]
for b, entries in REG.items():
    out, report = {}, []
    t0 = time.time()
    for name, sp in entries:
        if only_parts and name not in only_parts:
            continue
        rec = {'name': name}
        try:
            # every part draws its randomness from a seed of its own name, so a
            # build repeats exactly (`cargo xtask assets props` compares
            # rebuilds with the committed meshes)
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
                if BLEND_DIR and THEME_OF.get(b) != 'selftest':
                    blend_parts[THEME_OF[b]].append((b, name, pos, nrm, tris))
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
if BLEND_DIR:
    write_blends(BLEND_DIR, blend_parts)
