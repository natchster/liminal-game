"""The census's frames into contact sheets: each kind's best shot (the
largest PNG, the most detail, as in October), how many floors had it, its
theme; 12 cells a sheet (4 x 3), each numbered."""
import collections, glob, json, os, subprocess, sys

C = sys.argv[1]
OUT = sys.argv[2]
REPO = sys.argv[3]
os.makedirs(OUT, exist_ok=True)

floors = collections.defaultdict(set)
best = {}
for tsv in glob.glob(f"{C}/r_*/census/census.tsv"):
    d = os.path.dirname(tsv)
    for line in open(tsv):
        f = line.rstrip("\n").split("\t")
        if len(f) < 5:
            continue
        name, kind, seed, level = f[0], f[1], f[2], f[3]
        floors[kind].add((seed, level))
        png = os.path.join(d, name + ".png")
        if not os.path.exists(png):
            continue
        sz = os.path.getsize(png)
        if kind not in best or sz > best[kind][0]:
            best[kind] = (sz, png, f[5] if len(f) > 5 else "")

# every themed placer and its theme, from the game's tables
src = open(os.path.join(REPO, "crates/game/src/world/tables.rs")).read()
import re
order = re.findall(r'"(place_[a-z0-9_]+)"', src.split("THEMED_ORDER")[1].split("];")[0])
theme_of = dict(re.findall(r'\("(place_[a-z0-9_]+)",\s*"([a-z]+)"\)', src))
old = {}
for line in open(os.path.join(REPO, "docs/research/props-census/grades.tsv")):
    f = line.rstrip("\n").split("\t")
    if f[0].startswith("#"):
        continue
    old[f[0]] = f

kinds = sorted(best, key=lambda k: (theme_of.get(k, "core" if k.startswith("core_") else old.get(k, ["", "", "", "", "", "?"])[5]), k))
never = [p for p in order if p not in floors]
import random
random.Random(13).shuffle(kinds)
rows = []
for k in kinds:
    th = theme_of.get(k) or (old[k][5] if k in old else ("core" if k.startswith("core_") else "?"))
    rows.append((k, th, len(floors[k]), best[k][1], best[k][2]))
json.dump({"rows": rows, "never": never, "order": order}, open(f"{OUT}/kinds.json", "w"), indent=1)

font = "/System/Library/Fonts/Helvetica.ttc"
per = 12
for si in range(0, len(rows), per):
    cells = rows[si:si + per]
    n = si // per + 1
    name = f"sheet{n:03d}"
    args = ["ffmpeg", "-y", "-loglevel", "error"]
    for c in cells:
        args += ["-i", c[3]]
    # pad to 12 with black
    fl = []
    for i, c in enumerate(cells):
        fl.append(f"[{i}:v]scale=312:192,pad=320:200:4:4:white[c{i}]")
    for i in range(len(cells), per):
        fl.append(f"color=c=black:s=320x200:d=1[c{i}]")
    layout = "|".join(f"{(i % 4) * 320}_{(i // 4) * 200}" for i in range(per))
    fl.append("".join(f"[c{i}]" for i in range(per)) + f"xstack=inputs={per}:layout={layout}[out]")
    args += ["-filter_complex", ";".join(fl), "-map", "[out]", "-frames:v", "1", f"{OUT}/{name}.png"]
    subprocess.run(args, check=True)
    with open(f"{OUT}/{name}.txt", "w") as t:
        for i, c in enumerate(cells):
            t.write(f"{i+1}\t{c[0]}\n")
print(len(rows), "kinds,", (len(rows) + per - 1) // per, "sheets;", len(never), "themed placers never placed")
