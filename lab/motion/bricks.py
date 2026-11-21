"""
NVIDIA MotionBricks (SOMA) for the lab: motion filled in between poses.

MotionBricks (SIGGRAPH 2026, `nvidia/MotionBricks`, NVIDIA Open Model
Agreement) is given the four most recent poses of a body (the context) and
up to four poses it should reach (the target), on the same 30-joint SOMA
skeleton as Kimodo's clips, and makes the motion between them (0.6–1.9 s at
30 Hz): a person's weight carried from one to the other, the steps it takes,
where it turns. Kimodo makes each clip from words; MotionBricks makes what
lies between two clips, so a creature going from one mode to the next moves
as a body does, not as two poses averaged (`motion.py bake`: its bridges).

Two backends, one tensor interface (NVIDIA's ONNX graph's: the same inputs,
the same outputs); `MOTIONBRICKS_BACKEND=mlx|cuda` chooses, else the
platform does:

- `mlx`, on Apple Silicon (the default on a Mac): the MLX port
  natchster/motionbricks-mlx (Apache-2.0; pinned in pyproject.toml; checked
  by its authors against NVIDIA's PyTorch to 1e-5), with the SOMA weights
  from `nvidia/MotionBricks` (models.toml, `motion.py fetch`).
  `MOTIONBRICKS_MLX=DIR` uses a clone of the port instead (and its
  `checkpoints/`), read only: loaded without writing its bytecode. A clone
  beside this repository (`../motionbricks-mlx`) is used when the port is
  not installed.
- `cuda`, on Linux with an NVIDIA GPU (the default there): NVIDIA's own
  graph, NVlabs/motionbricks v0.2.0's ONNX (models.toml's `[[file]]`s,
  fetched and checked by `motion.py fetch`), on ONNX Runtime's TensorRT or
  CUDA provider (`uv sync --group cuda`). `MOTIONBRICKS_ONNX=FILE` uses
  another copy of the graph.
"""
from __future__ import annotations

import os
import platform
import sys
import tomllib
import urllib.request
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import kimodo_lab as K  # noqa: E402

BACKEND = os.environ.get("MOTIONBRICKS_BACKEND") or ("mlx" if sys.platform == "darwin" and platform.machine() == "arm64" else "cuda")
SIBLING = K.ROOT.parents[1] / "motionbricks-mlx"
FPS = 30
BOUNDARY = 3  # frames dropped at each end of what it makes (upstream's engine)
LEGS = (K.J["RightLeg"], K.J["LeftLeg"])
OUTPUTS = ("pred_global_motions", "pred_num_tokens")


def files() -> dict[str, dict]:
    """models.toml's `[[file]]`s: what is fetched from outside Hugging Face."""
    return {f["id"]: f for f in tomllib.loads((K.HERE / "models.toml").read_text()).get("file", [])}


def file_path(f: dict) -> Path:
    return K.OUT / "files" / f["sha256"][:16] / Path(f["url"]).name


def fetch_files(backend: str = BACKEND, only: set[str] | None = None) -> None:
    """The backend's `[[file]]`s into target/lab/motion/files/, each checked
    by its size and SHA-256 (`only`: those ids)."""
    for f in files().values():
        if f.get("backend") not in (None, backend) or (only and f["id"] not in only):
            continue
        dst = file_path(f)
        if dst.exists() and K.sha256(dst) == f["sha256"]:
            print(f"have {f['id']} ({dst.relative_to(K.ROOT)})")
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        part = dst.with_suffix(dst.suffix + ".part")
        print(f"fetching {f['id']} ({f['size'] / 1e6:.0f} MB) from {f['url']}")
        urllib.request.urlretrieve(f["url"], part)
        if part.stat().st_size != f["size"] or K.sha256(part) != f["sha256"]:
            part.unlink()
            raise SystemExit(f"{f['id']}: not the file pinned in models.toml (size or SHA-256 differ)")
        part.rename(dst)


class Bricks:
    """A SOMA MotionBricks behind the ONNX graph's tensor interface:
    `run(inputs)` → `pred_global_motions` [N,64,366], `pred_num_tokens` [N]."""

    def __init__(self, backend: str, run, rig: Path, made_by: str):
        self.backend, self._run, self.made_by = backend, run, made_by
        with np.load(rig) as d:
            self.offsets = (d["capture_offsets"].astype(np.float64), d["standard_offsets"].astype(np.float64))

    def run(self, inputs: dict) -> dict:
        return self._run(inputs)


_MODEL: Bricks | None = None


def model(backend: str = BACKEND) -> Bricks:
    """The SOMA model on `backend` (once a process)."""
    global _MODEL
    if _MODEL is None:
        _MODEL = _mlx() if backend == "mlx" else _cuda() if backend == "cuda" else None
        if _MODEL is None:
            raise SystemExit(f"MOTIONBRICKS_BACKEND={backend}: mlx (Apple Silicon) or cuda (Linux, NVIDIA)")
    return _MODEL


def _mlx() -> Bricks:
    sys.dont_write_bytecode = True
    home = os.environ.get("MOTIONBRICKS_MLX")
    try:
        if home:
            raise ImportError
        import motionbricks_mlx as mb

        pin = next((d for d in tomllib.loads((K.HERE / "pyproject.toml").read_text())["project"]["dependencies"] if d.startswith("motionbricks-mlx")), "")
        where = "natchster/motionbricks-mlx@" + (pin.split("@")[2][:12] if pin.count("@") >= 2 else "?")
    except ImportError:
        home = Path(home or SIBLING).resolve()
        if not (home / "model" / "motionbricks_mlx").exists():
            raise SystemExit("the MLX port is not installed (uv sync) and no clone was found: MOTIONBRICKS_MLX=DIR, a clone of natchster/motionbricks-mlx")
        sys.path.insert(0, str(home / "model"))
        import motionbricks_mlx as mb

        where = "a clone of natchster/motionbricks-mlx"
    from importlib import resources

    m = K.models()["motionbricks-soma"]
    try:
        ck = K.snapshot(m) / "soma"
    except Exception:  # noqa: BLE001
        ck = None
    if ck is None or not ck.exists():
        local = Path(home or SIBLING) / "checkpoints" / "MotionBricks" / "soma"
        if not local.exists():
            raise SystemExit("no MotionBricks SOMA weights: motion.py fetch")
        ck = local
    net = mb.load("soma", str(ck))
    rig = Path(str(resources.files("motionbricks_mlx").joinpath("assets", "soma_model_rig.npz")))
    return Bricks("mlx", net.run, rig, f"MotionBricks SOMA on MLX ({where}), {m['repo']}@{m['revision'][:12]}")


def _cuda() -> Bricks:
    try:
        import onnxruntime as ort
    except ImportError:
        raise SystemExit("ONNX Runtime is not installed: uv sync --group cuda (Linux, NVIDIA)") from None
    fs = files()
    onnx = Path(os.environ.get("MOTIONBRICKS_ONNX") or file_path(fs["motionbricks-soma-onnx"]))
    rig = file_path(fs["motionbricks-soma-rig"])
    for p in (onnx, rig):
        if not p.exists():
            raise SystemExit(f"{p}: not fetched (MOTIONBRICKS_BACKEND=cuda motion.py fetch)")
    avail = ort.get_available_providers()
    providers = [p for p in ("TensorrtExecutionProvider", "CUDAExecutionProvider", "CPUExecutionProvider") if p in avail]
    sess = ort.InferenceSession(str(onnx), providers=providers)

    def run(inputs: dict) -> dict:
        return dict(zip(OUTPUTS, sess.run(list(OUTPUTS), inputs)))

    src = fs["motionbricks-soma-onnx"]
    graph = "NVIDIA's ONNX graph" if not os.environ.get("MOTIONBRICKS_ONNX") else "a copy of its ONNX graph (MOTIONBRICKS_ONNX)"
    return Bricks("cuda", run, rig, f"MotionBricks SOMA, {graph} ({src['source']}) on ONNX Runtime ({sess.get_providers()[0]})")


def _offsets() -> tuple[np.ndarray, np.ndarray]:
    """The rig's capture and standard rest offsets [30,3,3] (the model's)."""
    return model().offsets


def to_capture(rot: np.ndarray) -> np.ndarray:
    """Kimodo's global rotations are the model's standard frame; its
    inputs and outputs are the capture skeleton's (checked: with this, the
    bones it makes keep Kimodo's rest directions to 0.999)."""
    C, S = _offsets()
    return rot @ S @ np.swapaxes(C, -1, -2)


def from_capture(rot: np.ndarray) -> np.ndarray:
    C, S = _offsets()
    return rot @ C @ np.swapaxes(S, -1, -2)


# ---------------------------------------------------------------- rotations


def six_to_mat(six: np.ndarray) -> np.ndarray:
    """6D (the first two columns) → 3×3, columns (x, y, z)."""
    x = six[..., 0:3] / np.linalg.norm(six[..., 0:3], axis=-1, keepdims=True)
    z = np.cross(x, six[..., 3:6])
    z /= np.linalg.norm(z, axis=-1, keepdims=True)
    y = np.cross(z, x)
    return np.stack([x, y, z], axis=-1)


def mat_to_six(m: np.ndarray) -> np.ndarray:
    return np.concatenate([m[..., 0], m[..., 1]], axis=-1)


def yaw_mat(a: float) -> np.ndarray:
    """A turn about +Y by `a` (x' = x c + z s, z' = −x s + z c)."""
    c, s = np.cos(a), np.sin(a)
    return np.array([[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]], np.float32)


def heading(pos: np.ndarray) -> np.ndarray:
    """Where the hips face (0 = +Z), as upstream reads it: across from the
    left hip to the right, forward = up × across."""
    a = pos[..., LEGS[0], :] - pos[..., LEGS[1], :]
    return np.arctan2(a[..., 2], -a[..., 0])


# ---------------------------------------------------------------- a request


def request(ctx: dict, tgt: dict, tokens: tuple[int, ...] = tuple(range(8, 15))) -> tuple[dict, dict]:
    """Four context poses and one to four target poses (world space, +Y up:
    `pos` [n,30,3], `rot` [n,30,3,3] global rotations as Kimodo's) → the
    model's inputs, the last context pose's hips put at the origin facing +X
    (upstream's alignment), and that alignment."""
    n_t = len(tgt["pos"])
    pos = np.concatenate([ctx["pos"], tgt["pos"], np.repeat(tgt["pos"][-1:], 4 - n_t, 0)]).astype(np.float64)
    rot = np.concatenate([ctx["rot"], tgt["rot"], np.repeat(tgt["rot"][-1:], 4 - n_t, 0)]).astype(np.float64)
    h = heading(pos[3])
    origin = np.array([pos[3, 0, 0], 0.0, pos[3, 0, 2]])
    # turn so the last context pose faces +X (heading π/2)
    turn = yaw_mat(np.pi / 2 - h)
    p = (pos - origin) @ turn.T
    r = turn[None, None] @ rot
    hd = heading(p)
    hips = p[:, 0]
    nxt = [min(i + 1, 7) if i != 3 else 3 for i in range(8)]
    dh = np.arctan2(np.sin(hd[nxt] - hd), np.cos(hd[nxt] - hd))
    g = np.zeros((1, 8, 5), np.float32)
    g[0, :, :3] = hips
    g[0, :, 3] = np.cos(hd)
    g[0, :, 4] = np.sin(hd)
    loc = np.zeros((1, 8, 4), np.float32)
    loc[0, :, 0] = dh * FPS
    loc[0, :, 1] = (hips[nxt, 0] - hips[:, 0]) * FPS
    loc[0, :, 2] = (hips[nxt, 2] - hips[:, 2]) * FPS
    loc[0, :, 3] = hips[:, 1]
    ground = hips * np.array([1.0, 0.0, 1.0])
    rel = p[:, 1:] - ground[:, None]
    poses = np.zeros((1, 8, 267), np.float32)
    poses[0, :, :87] = rel.reshape(8, 87)
    poses[0, :, 87:] = mat_to_six(to_capture(r)).reshape(8, 180)
    has = np.array([[1 if i < 4 + n_t else 0 for i in range(8)]], np.int32)
    has_loc = has.copy()
    has_loc[0, 3] = 0
    has_loc[0, 3 + n_t] = 0
    allowed = np.zeros((1, 11), np.int32)
    for t in tokens:
        allowed[0, t - 6] = 1
    inputs = {
        "global_root_values": g,
        "has_global_root_values": has,
        "local_root_values": loc,
        "has_local_root_values": has_loc,
        "local_poses": poses,
        "has_local_poses": has.copy(),
        "allowed_pred_num_tokens": allowed,
    }
    return inputs, {"origin": origin, "turn": turn}


def decode(out: dict, align: dict, i: int = 0) -> dict:
    """What it made, back in the world: positions [T,30,3], global
    rotations [T,30,3,3] (Kimodo's convention), contacts [T,4]; the
    boundary frames dropped."""
    n = int(out["pred_num_tokens"][i]) * 4
    f = out["pred_global_motions"][i, BOUNDARY : n - BOUNDARY].astype(np.float64)
    T = len(f)
    hips = f[:, 0:3]
    rel = f[:, 5:92].reshape(T, 29, 3)
    ground = hips * np.array([1.0, 0.0, 1.0])
    p = np.concatenate([hips[:, None], rel + ground[:, None]], axis=1)
    r = from_capture(six_to_mat(f[:, 92:272].reshape(T, 30, 6)))
    back = align["turn"].T
    pos = p @ back.T + align["origin"]
    rot = back[None, None] @ r
    return {"pos": pos.astype(np.float32), "rot": rot.astype(np.float32), "contacts": np.clip(f[:, 362:366], 0, 1).astype(np.float32)}


def kimodo_frames(dec: dict, frames) -> dict:
    """Frames of a decoded Kimodo sample as poses: positions and global rotations."""
    idx = np.asarray(frames)
    return {"pos": dec["pos"][idx], "rot": six_to_mat(dec["rot6"][idx])}


# ---------------------------------------------------------------- bridges


def specs(doc: dict) -> list[dict]:
    """A creature's bridges (`[[bridge]]` in its briefs): from each clip of
    `from` into `to`, landing on `to` at `at` s (where its pick starts it),
    the target `ahead` m in front (default: as far as the game's root goes
    in `BRIDGE_S` from the one clip's speed to the other's, `travel`)."""
    out = []
    for b in doc.get("bridge", []):
        froms = b["from"] if isinstance(b["from"], list) else [b["from"]]
        tos = b["to"] if isinstance(b["to"], list) else [b["to"]]
        for f in froms:
            for t in tos:
                if f != t:
                    out.append({"from": f, "to": t, "at": float(b.get("at", 0.0)), "ahead": b.get("ahead")})
    return out


# Every bridge is 7 tokens: 22 frames once its ends are dropped, 0.7 s
TOKENS = (7,)
BRIDGE_S = 21 / FPS
# How fast the game changes a creature's speed, m/s², on the clips' scale
# (`ai::ent_motion`: 9, the Hound 16, on bodies 1.5-2 times the mocap's
# speed): a creature starts and stops in a tenth of a second, so a bridge's
# target is put where the game's root would be by then, not where a person
# easing into a walk would (who fell behind the root: planted feet stretched
# 40 cm and whipped forward, strides split flat to keep up)
ACCEL = 6.0
ACCEL_FOURS = 10.0


def travel(va: float, vb: float, accel: float, dur: float = BRIDGE_S) -> float:
    """How far a root goes in `dur` s from `va` m/s toward `vb` at `accel`."""
    t1 = min(abs(vb - va) / accel, dur)
    v1 = va + np.sign(vb - va) * accel * t1
    return (va + v1) / 2 * t1 + v1 * (dur - t1)


def phases(n_frames: int, looped: bool, speed: float) -> list[int]:
    """Where in a clip bridges leave from (frames). The game waits for the
    next of them to come round before it leaves (`anim::Anim`), so a body
    in step leaves on the very frame its bridge was made from: a clip that
    travels every 0.25 s (a step's time: a longer wait shows), one that stands
    every 0.6 s (blended from where it is: it hardly moves); a clip that plays
    once at a few points and at its end, where it is held."""
    dur = n_frames / FPS
    if looped:
        every = 0.25 if speed > 0.2 else 0.6
        n = int(np.clip(round(dur / every), 3, 20))
        return sorted({int(round(i * n_frames / n)) % n_frames for i in range(n)})
    n = int(np.clip(round(dur / 0.6), 2, 5))
    first = min(9, n_frames - 1)
    return sorted({int(round(first + (n_frames - 1 - first) * i / (n - 1))) for i in range(n)})


def speed_at(path: np.ndarray, k: int) -> float:
    """The clip's speed about frame k, m/s (its root's path over ±5 frames)."""
    a, b = max(0, k - 5), min(len(path) - 1, k + 5)
    if b <= a:
        return 0.0
    return float(np.linalg.norm(path[b, :2] - path[a, :2]) * FPS / (b - a))


def held_weights(pos: np.ndarray, contacts: np.ndarray, fours: bool, w: np.ndarray, M) -> np.ndarray:
    """How much of the end's correction (against the start's) each joint
    takes each frame [T,30]: `w` (eased from 0 to 1), except that a joint
    the bridge plants takes one value while it touches (none if it touches
    from the start, all if to the end, else where its touch is half done;
    one touching from start to end is eased as the rest)
    and changes only while it swings: eased across a planted foot, the
    correction slid it, and the game's lock held it there, a stance 26 cm
    wrong."""
    T = len(pos)
    W = np.repeat(w[:, None], 30, axis=1)
    if fours:
        on = np.concatenate([M.low_still(pos, [n for n, _ in M.KNEES]), M.low_still(pos, [n for n, _ in M.HANDS])], axis=1) > 0.5
        joints = [[K.J["LeftShin"], K.J["LeftFoot"], K.J["LeftToeBase"]], [K.J["RightShin"], K.J["RightFoot"], K.J["RightToeBase"]],
                  [K.J["LeftHand"], K.J["LeftHandThumbEnd"], K.J["LeftHandMiddleEnd"]], [K.J["RightHand"], K.J["RightHandThumbEnd"], K.J["RightHandMiddleEnd"]]]
    else:
        c = M.gauss(contacts, 1.0) > 0.5
        on = np.stack([c[:, 0] | c[:, 1], c[:, 2] | c[:, 3]], axis=1)
        joints = [[K.J["LeftFoot"], K.J["LeftToeBase"]], [K.J["RightFoot"], K.J["RightToeBase"]]]
    for col, js in enumerate(joints):
        v = w.copy()
        keys = []  # (frame, value) at each touch's ends
        t = 0
        while t < T:
            if not on[t, col]:
                t += 1
                continue
            e = t
            while e < T and on[e, col]:
                e += 1
            # (one touching all through it must still go from the one end to
            # the other: eased, as the rest)
            if not (t == 0 and e == T):
                val = 0.0 if t == 0 else 1.0 if e == T else float(w[(t + e) // 2])
                v[t:e] = val
                keys += [(t, val), (e - 1, val)]
            t = e
        # between touches: from the one's value to the next's
        keys = [(0, 0.0)] + keys + [(T - 1, 1.0)]
        for (a, va), (b, vb) in zip(keys, keys[1:]):
            if b - a > 1:
                x = np.linspace(0.0, 1.0, b - a + 1)[1:-1]
                v[a + 1 : b] = va + (vb - va) * (x * x * (3 - 2 * x))
        for j in js:
            W[:, j] = v
    return W


def make(mb, kind: str, spec: dict, A: dict, B: dict, k: int, M) -> dict | None:
    """One bridge from clip A at its frame k into clip B at `spec['at']`.
    A and B are the bake's sources (`dec`: the Kimodo sample decoded; `a`:
    the raw frame its baked frame 0 is; `local`, `path`: as baked; `fours`).
    → the baked frames (as a clip's) and its report, or None if it cannot."""
    kb = int(round(spec["at"] * FPS))
    ia, ib = A["a"] + k, B["a"] + kb
    if ia < 3 or ib + 4 > len(B["dec"]["pos"]) or ia >= len(A["dec"]["pos"]) or kb >= len(B["local"]):
        return None
    ctx = kimodo_frames(A["dec"], range(ia - 3, ia + 1))
    tgt = kimodo_frames(B["dec"], range(ib, ib + 4))
    va, vb = speed_at(A["path"], k), speed_at(B["path"], kb)
    ahead = spec["ahead"] if spec["ahead"] is not None else travel(va, vb, ACCEL_FOURS if A["fours"] else ACCEL)
    # the target turned to face as the context does, by the clips' own
    # facing (the hips' and the shoulders' together: the frame the game
    # turns a body by), not the hips' alone: on all fours the two differ by
    # 20°, and a target lined up by its hips made every bridge twist 30°
    hc, ht = A["yaw"][ia], B["yaw"][ib]
    oc = ctx["pos"][3, 0] * np.array([1.0, 0.0, 1.0])
    ot = tgt["pos"][0, 0] * np.array([1.0, 0.0, 1.0])
    turn = yaw_mat(hc - ht)
    fwd = np.array([np.sin(hc), 0.0, np.cos(hc)])
    tgt = {"pos": (tgt["pos"] - ot) @ turn.T + oc + fwd * ahead, "rot": turn[None, None] @ tgt["rot"]}
    inputs, align = request(ctx, tgt, TOKENS)
    d = decode(mb.run(inputs), align)
    T = len(d["pos"])
    w = np.linspace(0.0, 1.0, T)
    w = (w * w * (3 - 2 * w))[:, None]
    W = held_weights(d["pos"], d["contacts"], A["fours"], w[:, 0], M)
    # its ends onto the poses it was given, in the world (as upstream's
    # engine does), before its feet are planted; what it plants takes one
    # correction while it touches (`held_weights`): a foot on the floor is
    # never slid by it
    e0, e1 = ctx["pos"][3] - d["pos"][0], tgt["pos"][0] - d["pos"][-1]
    raw = d["pos"] + e0 * (1 - W[:, :, None]) + e1 * W[:, :, None]
    pos, con = M.plant_feet(raw, d["contacts"], A["fours"])
    # its root and facing: its hips', smoothed, eased at each end onto the
    # clip's own there (the frame each clip's joints are kept in), so the
    # joints are measured as the clips' are
    hips = pos[:, 0] * np.array([1.0, 0.0, 1.0])
    root = M.gauss(hips, 4.0)
    yaw = M.facing(pos)
    ra = A["root"][ia] * np.array([1.0, 0.0, 1.0])
    rb = ((B["root"][ib] * np.array([1.0, 0.0, 1.0]) - ot) @ turn.T) + oc + fwd * ahead
    root = root + (ra - root[0]) * (1 - w) + (rb - root[-1]) * w
    ya, yb = A["yaw"][ia], B["yaw"][ib] + (hc - ht)
    wrap = lambda x: (x + np.pi) % (2 * np.pi) - np.pi  # noqa: E731
    yaw = yaw + wrap(ya - yaw[0]) * (1 - w[:, 0]) + wrap(yb - yaw[-1]) * w[:, 0]
    local = M.to_local(pos, root, yaw)
    # its ends made exactly the frames it joins (eased: none of the
    # correction at either end changes how fast it moves there)
    d0 = A["local"][k] - local[0]
    d1 = B["local"][kb] - local[-1]
    local = local + d0 * (1 - W[:, :, None]) + d1 * W[:, :, None]
    if os.environ.get("BRICKS_DEBUG"):
        e_end = np.linalg.norm(local[-1] - B["local"][kb], axis=1)
        e_beg = np.linalg.norm(local[0] - A["local"][k], axis=1)
        if e_end.max() > 1e-4 or e_beg.max() > 1e-4:
            j = int(e_end.argmax())
            print(f"  JOIN {spec['from']}>{spec['to']}@{k}: start {e_beg.max():.4f} end {e_end.max():.4f} {K.JOINTS[j]} W {np.round(W[:, j], 2)}")
    r0, y0 = root[0], yaw[0]
    s0, c0 = np.sin(y0), np.cos(y0)
    dx, dz = root[:, 0] - r0[0], root[:, 2] - r0[2]
    path = np.stack([dx * c0 - dz * s0, dx * s0 + dz * c0, yaw - y0], axis=1)
    frames = np.concatenate([local.reshape(T, -1), path, (con > 0.5).astype(np.float32)], axis=1).astype(np.float32)
    # how it went: the most its ends were moved, its hips' range, a foot's slide
    fix = float(max(np.linalg.norm(d0, axis=1).max(), np.linalg.norm(d1, axis=1).max()))
    # (and of its body alone: the ends a clip plants, its hands, knees and
    # feet, are held where they touch by the game's locks, wherever the
    # clip and the bridge each held them)
    # how far the model missed the poses it was given, its body's joints
    body = [K.J[n] for n in ("Hips", "Spine1", "Spine2", "Chest", "Neck1", "Neck2", "Head", "LeftArm", "RightArm", "LeftLeg", "RightLeg")]
    fix_body = float(max(np.linalg.norm(e0[body], axis=1).max(), np.linalg.norm(e1[body], axis=1).max()))
    if os.environ.get("BRICKS_DEBUG"):
        n0, n1 = np.linalg.norm(d0, axis=1), np.linalg.norm(d1, axis=1)
        print(f"  {spec['from']}>{spec['to']}@{k}: start {n0.max():.3f} {K.JOINTS[n0.argmax()]} (hips {n0[0]:.3f})  end {n1.max():.3f} {K.JOINTS[n1.argmax()]} (hips {n1[0]:.3f})")
    skate = []
    for name, ci in (M.KNEES + M.HANDS) if A["fours"] else M.FEET:
        p = pos[:, K.J[name]]
        hv = np.linalg.norm(np.diff(p[:, [0, 2]], axis=0), axis=1) * FPS
        on = con[1:, ci] > 0.5
        skate.append(float(hv[on].mean()) if on.any() else 0.0)
    rep = {
        "frames": T,
        "ahead": round(float(ahead), 3),
        "travel": round(float(np.linalg.norm(root[-1, [0, 2]] - root[0, [0, 2]])), 3),
        "fix": round(fix, 3),
        "fix_body": round(fix_body, 3),
        "skate": round(float(np.mean(skate)), 4),
        "hips": [round(float(local[:, 0, 1].min()), 3), round(float(local[:, 0, 1].max()), 3)],
        "head_max": round(float(local[:, K.J["Head"], 1].max()), 3),
    }
    return {"frames": frames, "from_t": k / FPS, "to_t": kb / FPS, "report": rep}


# a bridge whose ends had to be moved further than this (m; its body's, and
# any joint's) is not one: the model went somewhere else, and its pair
# keeps the game's blend
MAX_FIX_BODY = 0.08
MAX_FIX = 0.3


def build(kind: str, doc: dict, briefs: dict, src: dict, M) -> tuple[list[dict], list[str]]:
    """Every bridge of a creature: the library's entries and the report's lines."""
    sp = specs(doc)
    if not sp:
        return [], []
    mb = model()
    lib, lines, dropped = [], [], []
    for s in sp:
        A, B = src.get(s["from"]), src.get(s["to"])
        if A is None or B is None:
            dropped.append(f"{s['from']}>{s['to']}: no such clip")
            continue
        speed = float(np.linalg.norm(np.diff(A["path"][:, :2], axis=0), axis=1).sum() * FPS / max(len(A["path"]) - 1, 1))
        for k in phases(len(A["local"]), A["loop"], speed):
            out = make(mb, kind, s, A, B, k, M)
            name = f"{s['from']}>{s['to']}@{k}"
            if out is None:
                dropped.append(f"{name}: its frames run past a clip")
                continue
            r = out["report"]
            bad = []
            if r["fix_body"] > MAX_FIX_BODY or r["fix"] > MAX_FIX:
                bad.append(f"its ends moved {r['fix']} m (its body {r['fix_body']} m)")
            mh = briefs[s["to"]].get("max_head")
            if mh is not None and r["head_max"] > mh:
                bad.append(f"its head rose to {r['head_max']} m")
            lo = briefs[s["to"]].get("min_hips", 0.0)
            if r["hips"][0] < lo - 0.05:
                bad.append(f"its hips fell to {r['hips'][0]} m")
            if bad:
                dropped.append(f"{name}: " + ", ".join(bad))
                continue
            data = out["frames"]
            n = len(data)
            path = data[:, 90:92]
            speed = float(np.linalg.norm(np.diff(path, axis=0), axis=1).sum() * FPS / max(n - 1, 1))
            # (its numbers are in the report, not the library: there are hundreds)
            lib.append({
                "name": name,
                "loop": False,
                "speed": round(speed, 3),
                "bridge": {"from": s["from"], "to": s["to"], "from_t": round(out["from_t"], 4), "to_t": round(out["to_t"], 4)},
                "data": data,
            })
            lines.append(f"| {s['from']} → {s['to']} | {out['from_t']:.2f} → {out['to_t']:.2f} s | {r['frames']} | {r['ahead']} | {r['travel']} | {r['fix_body']} | {r['fix']} | {r['skate']} |")
    head = [
        "",
        "## Bridges",
        "",
        "Made by MotionBricks (SOMA, the NVIDIA Open Model Agreement; `bricks.py`: the MLX port on Apple Silicon, NVIDIA's ONNX graph on CUDA on Linux) from the four frames of a clip that lead to where it leaves to the four of the clip it goes into, placed `ahead` m in front: the motion between, its ends then moved onto the frames it joins (`fix`, m: how far its body's joints missed the poses it was given, and the most any joint was moved onto the clips' frames: the hands, knees and feet a clip plants are held where they touch by the game's locks). One whose body missed by more than 8 cm, or any joint by more than 30 cm, is left to the blend (listed at the end). The game plays one where its creature goes from the one clip to the other, from the one nearest where it is in the first.",
        "",
        "| from → to | leaves → lands | frames | ahead m | travel m | fix, body m | fix, any joint m | skate m/s |",
        "|---|---|---|---|---|---|---|---|",
    ]
    tail = (["", f"Left to the blend ({len(dropped)}):", ""] + [f"- {x}" for x in dropped]) if dropped else ["", "Left to the blend: none."]
    return lib, head + lines + tail
