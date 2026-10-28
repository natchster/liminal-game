"""
Build the creatures' bodies (run by `cargo xtask assets creatures`):

    blender --background --factory-startup --python assets/src/creatures/build.py -- \
        --out target/xtask/assets/creatures [--only hound,smiler]

Every `body_<name>.py` in this folder defines `ORDER` (its place in the
file) and `build(kit) -> kit.Body`; each is built in an empty scene and
written as `<out>/<name>.json`, with Workbench previews beside it, and the
order of the bodies as `<out>/bodies.json`. See `kit.py` for the frames and
what a body is.
"""
import bpy, sys, os, json, importlib.util, traceback

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kit  # noqa: E402

args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
out = args[args.index('--out') + 1] if '--out' in args else os.path.join(HERE, 'out')
only = set(args[args.index('--only') + 1].split(',')) if '--only' in args else None
os.makedirs(out, exist_ok=True)

scripts = []
for f in sorted(os.listdir(HERE)):
    if f.startswith('body_') and f.endswith('.py'):
        name = f[5:-3]
        spec = importlib.util.spec_from_file_location('body_' + name, os.path.join(HERE, f))
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        scripts.append((getattr(mod, 'ORDER', 99), name, mod))
scripts.sort(key=lambda s: (s[0], s[1]))

built = []
for _, name, mod in scripts:
    if only and name not in only:
        continue
    kit.clear_scene()
    body = mod.build(kit)
    assert body.name == name, f'body_{name}.py built {body.name}'
    body.write(out)
    shown = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    if shown:
        kit.preview(shown, os.path.join(out, name))
    built.append(name)
    print(f'[creatures] built {name}')

with open(os.path.join(out, 'bodies.json'), 'w') as f:
    json.dump(built, f)
