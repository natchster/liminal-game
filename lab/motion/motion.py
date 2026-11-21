"""
The lab's motion pipeline (lab/README.md): briefs → Kimodo on MLX → the clip
library the viewer retargets onto a creature.

    .venv/bin/python motion.py fetch      # the pinned models (and MotionBricks' backend's files)
    .venv/bin/python motion.py doctor     # are they there, at their revisions and hashes; which backend
    .venv/bin/python motion.py gen [--creature K,…] [--only idle,run] [--candidates N]
    .venv/bin/python motion.py bake [--creature K,…] [--no-bridges]   # → clips/<kind>.motion and clips/<kind>.md
    .venv/bin/python motion.py install    # the libraries into the game (crates/game/assets/motion/)

A creature's briefs are `briefs/<kind>.toml`. `gen` samples each brief from
`candidates` seeds (cached under target/lab/motion/raw/ by everything that
decides a sample: the prompt, the seed, the steps, the guidance, the length
and the model's revision; two creatures with the same brief share it).
`bake` decodes every candidate, scores it (speed against the brief, foot
skating, jitter, the floor), keeps the best, plants its feet (on all fours
its knees and its hands) where they touch the ground, cuts a loop where one is
asked for, and writes the library: each frame's 30 joints in the
creature's own frame (+Y up, +Z ahead, its left at +X; the root at the
origin), the root's path and heading, and six contacts (each foot's heel
and toe, each hand; on all fours each knee stands for its foot). The game reads it (`entities::motion`).

Then its bridges (`[[bridge]]` in the briefs; `bricks.py`): NVIDIA
MotionBricks makes the motion from where one clip is into another, from a
few points in the first, its ends joined exactly to the frames it leaves
and lands on; the game plays one when its creature goes from the one clip
to the other, where it blended them before. MotionBricks runs on MLX on
Apple Silicon (natchster/motionbricks-mlx) or on NVIDIA's ONNX graph on
CUDA on Linux (`MOTIONBRICKS_BACKEND`, `bricks.py`); the bake itself needs
no MLX (only `gen`, which samples Kimodo, does).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
import sys
import tomllib
import zlib
from pathlib import Path

import numpy as np

# its own folder, explicitly (run isolated, `python -I`, nothing else is
# loaded from the folder it is started in)
sys.path.insert(0, str(Path(__file__).resolve().parent))
import kimodo_lab as K  # noqa: E402

CLIPS = K.HERE / "clips"
BRIEFS = K.HERE / "briefs"
# where the game reads them from (`entities::motion`)
GAME = K.ROOT / "crates" / "game" / "assets" / "motion"
MAGIC = b"OCMOTIO2"
# a bridge's values are kept in steps of 1/BRIDGE_SCALE m (0.5 mm)
BRIDGE_SCALE = 2048


def creatures() -> list[str]:
    return sorted(p.stem for p in BRIEFS.glob("*.toml"))


def briefs(kind: str, only: set[str] | None = None) -> tuple[dict, list[dict]]:
    doc = tomllib.loads((BRIEFS / f"{kind}.toml").read_text())
    d = doc["defaults"]
    out = []
    for c in doc["clip"]:
        if only and c["name"] not in only:
            continue
        b = {**d, **c}
        b.setdefault("loop", False)
        b["kind"] = kind
        out.append(b)
    return d, out


def raw_path(b: dict, seed: int) -> Path:
    rev = K.models()["kimodo-soma-seed"]["revision"]
    frames = int(round(b["seconds"] * K.FPS))
    key = hashlib.sha1(json.dumps([b["prompt"], seed, b["steps"], b["cfg"], frames, rev]).encode()).hexdigest()[:16]
    return K.OUT / "raw" / b["name"] / f"{seed}-{key}.npz"


def seeds_of(b: dict) -> list[int]:
    return [b["seed"] + k for k in range(b["candidates"])]


# ------------------------------------------------------------------ gen


def gen(kinds: list[str], only: set[str] | None, candidates: int | None) -> None:
    bs = [b for k in kinds for b in briefs(k, only)[1]]
    if candidates:
        for b in bs:
            b["candidates"] = candidates
    todo = [(b, s) for b in bs for s in seeds_of(b) if not raw_path(b, s).exists()]
    if not todo:
        print("every candidate is already sampled")
        return
    embs = K.embeddings(sorted({b["prompt"] for b, _ in todo}))
    motion = K.Motion()
    import time

    for b in bs:
        seeds = [s for s in seeds_of(b) if not raw_path(b, s).exists()]
        if not seeds:
            continue
        frames = int(round(b["seconds"] * K.FPS))
        t0 = time.time()
        x = motion.sample(embs[b["prompt"]], frames, b["steps"], seeds, b["cfg"])
        for s, xs in zip(seeds, x):
            p = raw_path(b, s)
            p.parent.mkdir(parents=True, exist_ok=True)
            np.savez(p, x=xs, prompt=b["prompt"], seed=s, steps=b["steps"], cfg=b["cfg"])
        print(f"{b['kind']}.{b['name']:<12} {len(seeds)} candidate(s), {frames} frames, {time.time() - t0:.1f} s", flush=True)


# ------------------------------------------------------------------ bake


def gauss(x: np.ndarray, sigma: float) -> np.ndarray:
    """Smoothed along axis 0 (edges held)."""
    if sigma <= 0:
        return x
    r = int(3 * sigma)
    k = np.exp(-0.5 * (np.arange(-r, r + 1) / sigma) ** 2)
    k /= k.sum()
    pad = np.concatenate([np.repeat(x[:1], r, 0), x, np.repeat(x[-1:], r, 0)])
    out = np.zeros_like(x)
    for i, w in enumerate(k):
        out += w * pad[i : i + len(x)]
    return out


def facing(pos: np.ndarray) -> np.ndarray:
    """The yaw each frame faces (0 = +Z), from the hips and the shoulders,
    unwrapped and smoothed: the creature's frame turns with its body, not
    with every sway of it."""
    left = (pos[:, K.J["LeftLeg"]] - pos[:, K.J["RightLeg"]]) + (pos[:, K.J["LeftArm"]] - pos[:, K.J["RightArm"]])
    # forward = left × up = (-lz, 0, lx)
    yaw = np.unwrap(np.arctan2(-left[:, 2], left[:, 0]))
    return gauss(yaw[:, None], 3.0)[:, 0]


def to_local(pos: np.ndarray, root: np.ndarray, yaw: np.ndarray) -> np.ndarray:
    """World joints → each frame's own frame (the root at the origin, facing +Z),
    as the game turns a body: (dx c − dz s, y, dx s + dz c)."""
    s, c = np.sin(yaw)[:, None], np.cos(yaw)[:, None]
    dx = pos[:, :, 0] - root[:, None, 0]
    dz = pos[:, :, 2] - root[:, None, 2]
    out = pos.copy()
    out[:, :, 0] = dx * c - dz * s
    out[:, :, 2] = dx * s + dz * c
    return out


FEET = [("LeftFoot", 0), ("LeftToeBase", 1), ("RightFoot", 2), ("RightToeBase", 3)]
HANDS = [("LeftHand", 4), ("RightHand", 5)]
CONTACTS = ["left heel", "left toe", "right heel", "right toe", "left hand", "right hand"]


# On all fours BONES-SEED crawls on its knees: the knee is what meets the
# floor in step, and it is what the Hound's hind feet are put where (the
# game reads the knee, not the ankle, as a hind end: `Retarget::fours`).
KNEES = [("LeftShin", 0), ("RightShin", 2)]


def low_still(pos: np.ndarray, names: list[str]) -> np.ndarray:
    """Each joint on the floor (Kimodo marks only the feet): low, and nearly
    still across the floor. [T, len(names)], 0 or 1."""
    out = np.zeros((len(pos), len(names)), np.float32)
    for k, name in enumerate(names):
        p = gauss(pos[:, K.J[name]], 1.0)
        hv = np.linalg.norm(np.gradient(p[:, [0, 2]], axis=0), axis=1) * K.FPS
        out[:, k] = ((p[:, 1] < 0.16) & (hv < 0.45)).astype(np.float32)
    return out


def hand_contacts(pos: np.ndarray) -> np.ndarray:
    """A hand on the floor. [T, 2], 0 or 1."""
    return low_still(pos, [n for n, _ in HANDS])


def fours_contacts(pos: np.ndarray) -> np.ndarray:
    """On all fours: each knee for its foot (heel and toe alike), each hand. [T, 6]"""
    kn = low_still(pos, [n for n, _ in KNEES])
    return np.concatenate([kn[:, [0, 0, 1, 1]], hand_contacts(pos)], axis=1)


def score(d: dict, b: dict) -> dict:
    pos, con = d["pos"], d["contacts"]
    fours = b.get("fours", False)
    if fours:
        con = fours_contacts(pos)
    root = gauss(d["root"], 2.0)
    v = np.linalg.norm(np.diff(root[:, [0, 2]], axis=0), axis=1) * K.FPS
    speed = float(v.mean())
    lo, hi = b["speed"]
    off = max(lo - speed, speed - hi, 0.0) / max(hi - lo, 0.2)
    skate = []
    for name, ci in (KNEES + HANDS) if fours else FEET:
        p = pos[:, K.J[name]]
        hv = np.linalg.norm(np.diff(p[:, [0, 2]], axis=0), axis=1) * K.FPS
        on = con[1:, ci] > 0.5
        skate.append(float(hv[on].mean()) if on.any() else 0.0)
    jerk = np.diff(pos, n=3, axis=0) * K.FPS**3
    jitter = float(np.linalg.norm(jerk, axis=2).mean())
    floor = float(np.minimum(pos[:, [K.J[n] for n, _ in FEET], 1], 0.0).mean())
    # a brief that must stay on its feet: the hips never below `min_hips`
    low = max(0.0, b.get("min_hips", 0.0) - float(pos[:, K.J["Hips"], 1].min()))
    # on all fours: the head never above `max_head` (a crawl that stood up
    # is no use to the Hound), its hands on the floor half the time at least
    high = max(0.0, float(pos[:, K.J["Head"], 1].max()) - b.get("max_head", 9.0))
    hands = float(con[:, 4:6].mean()) if fours else 1.0
    down = max(0.0, 0.5 - hands)
    total = 3.0 * off + np.mean(skate) / 0.08 + jitter / 400.0 + abs(floor) / 0.02 + low / 0.05 * 3.0 + high / 0.05 * 3.0 + down * 10.0
    return {"speed": round(speed, 3), "speed_off": round(off, 3), "skate": round(float(np.mean(skate)), 4), "jitter": round(jitter, 1), "floor": round(floor, 4), "hips_low": round(low, 3), "head_high": round(high, 3), "hands_down": round(hands, 2), "total": round(float(total), 3)}


def plant_feet(pos: np.ndarray, con: np.ndarray, fours: bool = False) -> tuple[np.ndarray, np.ndarray]:
    """Each foot joint (and on all fours each hand) held still while its
    contact holds (the mean of the run), eased in and out over three frames;
    nothing below the floor. The contacts come back six wide."""
    pos = pos.copy()
    con = fours_contacts(pos) if fours else np.concatenate([con, np.zeros((len(pos), 2), np.float32)], axis=1)
    c = gauss(con, 1.0) > 0.5
    for name, ci in (KNEES + HANDS) if fours else FEET:
        j = K.J[name]
        t = 0
        T = len(pos)
        while t < T:
            if not c[t, ci]:
                t += 1
                continue
            e = t
            while e < T and c[e, ci]:
                e += 1
            target = pos[t:e, j, [0, 2]].mean(0)
            for k in range(max(0, t - 3), min(T, e + 3)):
                w = 1.0 if t <= k < e else 1.0 - min(abs(k - t), abs(k - (e - 1))) / 3.0
                pos[k, j, [0, 2]] = pos[k, j, [0, 2]] * (1 - w) + target * w
            t = e
    floor = {"LeftFoot": 0.06, "RightFoot": 0.06, "LeftToeBase": 0.012, "RightToeBase": 0.012}
    if fours:
        floor = {"LeftHand": 0.04, "RightHand": 0.04}
    for name, lo in floor.items():
        j = K.J[name]
        pos[:, j, 1] = np.maximum(pos[:, j, 1], lo * 0.8)
    return pos, c.astype(np.float32)


def loop_cut(local: np.ndarray, fade: int) -> tuple[int, int, float]:
    """Where the clip best repeats: frames a < b, a at least `fade` in, b at
    least half the clip after a, the pose and its velocity at b closest to
    a's (each in its own frame)."""
    T = len(local)
    vel = np.diff(local, axis=0, prepend=local[:1])
    best = (fade, T - 1, 1e9)
    for a in range(fade, max(fade + 1, T // 3)):
        for b in range(a + T // 2, T):
            d = np.linalg.norm(local[a] - local[b], axis=1).mean() + 0.3 * np.linalg.norm(vel[a] - vel[b], axis=1).mean() * K.FPS / 10
            d -= 0.002 * (b - a) / T
            if d < best[2]:
                best = (a, b, d)
    return best


def bake(kinds: list[str], bridges: bool = True) -> None:
    motion = K.Motion()
    for kind in kinds:
        bake_one(motion, kind, bridges)


def bake_one(motion: "K.Motion", kind: str, bridges: bool = True) -> None:
    d, bs = briefs(kind)
    # what each clip was made from, for its bridges
    src = {}
    title = d.get("title", kind)
    lib = []
    report = [f"# {title}: the clips", "", f"Made by `motion.py bake --creature {kind}` from `briefs/{kind}.toml` (Kimodo SOMA SEED v1.1 on MLX, kimodo-mlx).", ""]
    report += ["| clip | seed | speed m/s | skate m/s | jitter | floor m | loop | score |", "|---|---|---|---|---|---|---|---|"]
    detail = []
    neutral = None
    for b in bs:
        fours = b.get("fours", False)
        cands = []
        for s in seeds_of(b):
            p = raw_path(b, s)
            if not p.exists():
                continue
            dec = motion.decode(np.load(p)["x"])
            sc = score(dec, b)
            if b["loop"]:
                # a loop's seam counts: how far its best cut is from repeating
                pos, _ = plant_feet(dec["pos"], dec["contacts"], fours)
                seam = loop_cut(to_local(pos, gauss(dec["root"], 2.0), facing(pos)), 10)[2]
                sc["seam"] = round(float(seam), 4)
                sc["total"] = round(float(sc["total"] + seam / 0.03), 3)
            cands.append((sc, s, dec))
        if not cands:
            print(f"{kind}.{b['name']}: no candidates (run gen)", file=sys.stderr)
            continue
        cands.sort(key=lambda c: c[0]["total"])
        sc, seed, dec = cands[0]
        pos, con = plant_feet(dec["pos"], dec["contacts"], fours)
        root = gauss(dec["root"], 2.0)
        yaw = facing(pos)
        local = to_local(pos, root, yaw)
        cut = None
        start = 0
        # (the whole take's root and facing, before a loop is cut: a bridge's
        # ends are put in the frame its clips are)
        root_full, yaw_full = root.copy(), yaw.copy()
        if b["loop"]:
            fade = 10
            a, e, dist = loop_cut(local, fade)
            # cross-fade the end into what comes before the start
            for i in range(fade):
                w = (i + 1) / fade
                w = w * w * (3 - 2 * w)
                local[e - fade + i] = local[e - fade + i] * (1 - w) + local[a - fade + i] * w
                con[e - fade + i] = con[e - fade + i] * (1 - w) + con[a - fade + i] * w
            local, con, root, yaw = local[a:e], con[a:e], root[a:e], yaw[a:e]
            cut = (a, e, round(float(dist), 4))
            start = a
        # the root's path from where the clip starts, facing +Z
        r0, y0 = root[0], yaw[0]
        s0, c0 = np.sin(y0), np.cos(y0)
        dx, dz = root[:, 0] - r0[0], root[:, 2] - r0[2]
        path = np.stack([dx * c0 - dz * s0, dx * s0 + dz * c0, yaw - y0], axis=1)
        frames = np.concatenate([local.reshape(len(local), -1), path, (con > 0.5).astype(np.float32)], axis=1).astype(np.float32)
        src[b["name"]] = {"dec": dec, "a": start, "local": local.copy(), "path": path, "loop": b["loop"], "fours": fours, "root": root_full, "yaw": yaw_full}
        lib.append({"name": b["name"], "prompt": b["prompt"], "for": b["for"], "seed": seed, "loop": b["loop"], "speed": sc["speed"], "score": sc, "data": frames})
        if b.get("neutral"):
            # its most typical frame
            m = local.mean(0)
            neutral = local[np.argmin(np.linalg.norm(local - m, axis=2).mean(1))]
        loop = f"{cut[0]}–{cut[1]} ({cut[2]})" if cut else "–"
        report.append(f"| {b['name']} | {seed} | {sc['speed']} | {sc['skate']} | {sc['jitter']} | {sc['floor']} | {loop} | {sc['total']} |")
        detail.append(f"- **{b['name']}** ({b['for']}): “{b['prompt']}”; candidates " + ", ".join(f"{s}: {c['total']}" for c, s, _ in cands))
        print(f"{kind}.{b['name']:<12} seed {seed}  speed {sc['speed']:.2f}  skate {sc['skate']:.3f}  jitter {sc['jitter']:.0f}  loop {loop}  score {sc['total']}")
    if neutral is None:
        neutral = lib[0]["data"][0, :90].reshape(30, 3)
    bridge_lines = []
    made_by = None
    if bridges:
        import bricks

        doc = tomllib.loads((BRIEFS / f"{kind}.toml").read_text())
        made, bridge_lines = bricks.build(kind, doc, {b["name"]: b for b in bs}, src, sys.modules[__name__])
        lib += made
        if made:
            made_by = bricks.model().made_by
            bridge_lines[3:3] = [f"This bake's: {made_by}.", ""]
            print(f"{kind}: {len(made)} bridges ({sum(len(c['data']) for c in made)} frames), {made_by}")
    header = {
        "format": MAGIC.decode(),
        "kind": kind,
        "fps": K.FPS,
        "joints": K.JOINTS,
        "contacts": CONTACTS,
        "neutral": np.round(neutral, 5).tolist(),
        "model": {k: K.models()[k]["repo"] + "@" + K.models()[k]["revision"][:12] for k in K.models() if k != "motionbricks-soma"},
        "frame": "30 joints (x, y, z) in the creature's frame, then the root (x, z, yaw) from the clip's start, then the contacts",
        "clips": [],
    }
    blob = bytearray()
    block = bytearray()
    for c in lib:
        if c.get("bridge"):
            # a bridge: each value's track over its frames, as 16-bit steps
            # of 1/BRIDGE_SCALE m from the frame before, all of them deflated
            # together after the clips (there are hundreds: as float32 the
            # libraries were 10 MB)
            q = np.round(c["data"].astype(np.float64) * BRIDGE_SCALE).astype(np.int64)
            d = np.diff(q, axis=0, prepend=np.zeros((1, q.shape[1]), np.int64))
            assert np.abs(d).max() < 32767, c["name"]
            header["clips"].append({k: v for k, v in c.items() if k != "data"} | {"frames": len(c["data"]), "offset": len(block), "scale": BRIDGE_SCALE})
            block += d.T.astype("<i2").tobytes()
        else:
            header["clips"].append({k: v for k, v in c.items() if k != "data"} | {"frames": len(c["data"]), "offset": len(blob)})
            blob += c["data"].astype("<f4").tobytes()
    if block:
        z = zlib.compress(bytes(block), 9)
        header["bridges"] = {"offset": len(blob), "length": len(z), "size": len(block), "layout": "each value's track, 16-bit steps from the frame before, deflated", "made_by": made_by}
        blob += z
    CLIPS.mkdir(exist_ok=True)
    h = json.dumps(header, ensure_ascii=False).encode()
    with (CLIPS / f"{kind}.motion").open("wb") as f:
        f.write(MAGIC + struct.pack("<I", len(h)) + h + bytes(blob))
    report += ["", "Candidates (lower is better: speed outside the brief ×3, foot skating / 0.08 m/s, jitter / 400 m/s³, below the floor / 2 cm; on all fours also the head too high and the hands too little on the floor):", ""] + detail
    report += bridge_lines
    (CLIPS / f"{kind}.md").write_text("\n".join(report) + "\n")
    print(f"wrote {CLIPS / (kind + '.motion')} ({len(lib)} clips, {(len(h) + len(blob)) / 1e6:.2f} MB)")


def install() -> None:
    """The baked libraries into the game, where `entities::motion` embeds them."""
    GAME.mkdir(parents=True, exist_ok=True)
    for kind in creatures():
        src = CLIPS / f"{kind}.motion"
        if not src.exists():
            print(f"{kind}: not baked (run bake)", file=sys.stderr)
            continue
        dst = GAME / src.name
        dst.write_bytes(src.read_bytes())
        print(f"{src.relative_to(K.ROOT)} -> {dst.relative_to(K.ROOT)}")


def fetch() -> None:
    """The pinned models into the Hugging Face cache, and MotionBricks'
    backend's files (`bricks.py`: the MLX port's weights are a model; the
    CUDA backend's graph and rig are files)."""
    import bricks

    from huggingface_hub import snapshot_download

    for m in K.models().values():
        if m.get("backend") not in (None, bricks.BACKEND):
            continue
        print(f"fetching {m['repo']}@{m['revision'][:10]}")
        snapshot_download(m["repo"], revision=m["revision"], allow_patterns=m["include"])
    bricks.fetch_files()


def doctor() -> None:
    import bricks

    ok = True
    print(f"MotionBricks backend: {bricks.BACKEND} (MOTIONBRICKS_BACKEND=mlx|cuda)")
    for m in K.models().values():
        if m.get("backend") not in (None, bricks.BACKEND):
            continue
        try:
            snap = K.snapshot(m)
        except Exception as e:  # noqa: BLE001
            print(f"MISSING {m['repo']}@{m['revision'][:10]}: {e.__class__.__name__} (run `motion.py fetch`)")
            ok = False
            continue
        for f, want in (("weights", "weights_sha256"), ("tokenizer.gguf", "tokenizer_sha256")):
            if want in m:
                path = snap / (m[f] if f == "weights" else f)
                got = K.sha256(path)
                good = got == m[want]
                ok &= good
                print(f"{'ok' if good else 'BAD'} {path.name} sha256 {got[:16]}")
        print(f"ok {m['id']}: {m['repo']}@{m['revision'][:10]} ({m['licence']})")
    for f in bricks.files().values():
        if f.get("backend") not in (None, bricks.BACKEND):
            continue
        p = bricks.file_path(f)
        good = p.exists() and K.sha256(p) == f["sha256"]
        ok &= good
        print(f"{'ok' if good else 'MISSING'} {f['id']}: {f['source']} (run `motion.py fetch`)" if not good else f"ok {f['id']}: {f['source']}")
    try:
        print(f"ok {bricks.model().made_by}")
    except SystemExit as e:
        print(f"BAD MotionBricks: {e}")
        ok = False
    try:
        import mlx.core as mx

        print(f"ok MLX {mx.__version__} on {mx.default_device()} (Kimodo's sampling, `gen`)")
    except ImportError:
        print("no MLX: `gen` (Kimodo's sampling) cannot run here; `bake` can")
    sys.exit(0 if ok else 1)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("command", choices=["fetch", "doctor", "gen", "bake", "install"])
    ap.add_argument("--creature", help="creatures, comma-separated (default: every briefs/*.toml)")
    ap.add_argument("--only", help="clips, comma-separated")
    ap.add_argument("--candidates", type=int)
    ap.add_argument("--no-bridges", action="store_true", help="bake: the clips alone (no MotionBricks)")
    a = ap.parse_args()
    only = set(a.only.split(",")) if a.only else None
    kinds = a.creature.split(",") if a.creature else creatures()
    for k in kinds:
        if not (BRIEFS / f"{k}.toml").exists():
            ap.error(f"no briefs/{k}.toml (have: {', '.join(creatures())})")
    {"fetch": fetch, "doctor": doctor, "gen": lambda: gen(kinds, only, a.candidates), "bake": lambda: bake(kinds, not a.no_bridges), "install": install}[a.command]()


if __name__ == "__main__":
    main()
