"""
Kimodo on MLX for the lab: the pinned models, the prompt's embedding, the
sampler and the decode of what it makes. Built on kimodo-mlx (MIT; its MLX
denoiser and its LLM2Vec layer), with what it lacks for this pipeline:

- the text encoder read from LocalAI's single Q8_0 GGUF (kimodo-mlx reads a
  bundle of per-layer files nobody publishes): every prompt is run through
  one layer at a time, so only one layer's weights are in memory;
- the sampler batched over candidates, with the two rows classifier-free
  guidance needs when nothing is constrained (kimodo-mlx evaluates a third,
  identical one);
- the whole feature vector decoded: joint positions, foot contacts and the
  smooth root, not only rotations (NVIDIA's `KimodoMotionRep` layout:
  smooth root 3, heading 2, joint positions 30x3, global rotations 30x6,
  velocities 30x3, foot contacts 4).
"""
from __future__ import annotations

import hashlib
import tomllib
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT / "target" / "lab" / "motion"

JOINTS = [
    "Hips", "Spine1", "Spine2", "Chest", "Neck1", "Neck2", "Head", "Jaw", "LeftEye", "RightEye",
    "LeftShoulder", "LeftArm", "LeftForeArm", "LeftHand", "LeftHandThumbEnd", "LeftHandMiddleEnd",
    "RightShoulder", "RightArm", "RightForeArm", "RightHand", "RightHandThumbEnd", "RightHandMiddleEnd",
    "LeftLeg", "LeftShin", "LeftFoot", "LeftToeBase", "RightLeg", "RightShin", "RightFoot", "RightToeBase",
]
J = {n: i for i, n in enumerate(JOINTS)}
FPS = 30.0


def models() -> dict[str, dict]:
    return {m["id"]: m for m in tomllib.loads((HERE / "models.toml").read_text())["model"]}


def snapshot(model: dict) -> Path:
    """The model's directory in the Hugging Face cache, at its pinned revision."""
    from huggingface_hub import snapshot_download

    return Path(snapshot_download(model["repo"], revision=model["revision"], allow_patterns=model["include"], local_files_only=True))


def fetch() -> None:
    from huggingface_hub import snapshot_download

    for m in models().values():
        print(f"fetching {m['repo']}@{m['revision'][:10]}")
        snapshot_download(m["repo"], revision=m["revision"], allow_patterns=m["include"])


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1 << 24), b""):
            h.update(block)
    return h.hexdigest()


# ---------------------------------------------------------------- the text


class Encoder:
    """LLM2Vec on Llama-3-8B, from LocalAI's monolithic GGUF, on MLX."""

    def __init__(self, weights: Path, tokenizer: Path):
        from gguf import GGUFReader
        from kimodo_mlx import text_encoder as te

        self.te = te
        self.reader = GGUFReader(str(weights), "r")
        self.tensors = {t.name: t for t in self.reader.tensors}
        self.tokenizer = te._Tokenizer(tokenizer)

    def _array(self, name: str, rows=None):
        import mlx.core as mx
        from gguf import GGMLQuantizationType, quants

        t = self.tensors[name]
        shape = tuple(reversed([int(x) for x in t.shape]))
        kind = int(t.tensor_type)
        if kind == int(GGMLQuantizationType.Q8_0):
            data = t.data if rows is None else t.data[rows]
            return mx.array(quants.dequantize(np.asarray(data), GGMLQuantizationType.Q8_0).astype(np.float32)).astype(mx.bfloat16)
        if kind == int(GGMLQuantizationType.BF16):
            return self.te._bf16(t.data, shape)
        if kind == int(GGMLQuantizationType.F32):
            return mx.array(np.asarray(t.data, dtype=np.float32).reshape(shape))
        raise ValueError(f"{name}: tensor type {kind} is not read here")

    def encode(self, prompts: list[str]) -> np.ndarray:
        """Each prompt's 4096 numbers (the mean of its tokens after BOS)."""
        import mlx.core as mx

        te = self.te
        ids = [self.tokenizer.encode(p) for p in prompts]
        xs = []
        for tokens in ids:
            if len(tokens) < 2 or len(tokens) > 512:
                raise ValueError("a prompt must be 1..511 tokens")
            xs.append(self._array("token_embedding.weight", rows=np.array(tokens)))
        positions = [mx.arange(len(t), dtype=mx.int32) for t in ids]
        stems = ("attn_q_proj", "attn_k_proj", "attn_v_proj", "attn_o_proj", "ffn_gate_proj", "ffn_up_proj", "ffn_down_proj")
        for i in range(32):
            names = ["attn_norm.weight", "ffn_norm.weight"]
            for stem in stems:
                names += [stem + "_base.weight", stem + "_lora_a.weight", stem + "_lora_b.weight"]
            layer = {n: self._array(f"layer.{i:02d}.{n}") for n in names}
            for k in range(len(xs)):
                xs[k] = te._layer(xs[k], layer, positions[k])
                mx.eval(xs[k])
            del layer
            print(f"  text layer {i + 1}/32", end="\r", flush=True)
        print()
        norm = self._array("final_norm.weight")
        out = []
        for x in xs:
            x = te._rms(x, norm, bf16=False)
            x = mx.mean(x[1:], axis=0)
            mx.eval(x)
            out.append(np.asarray(x, dtype=np.float32))
        return np.stack(out)


def embeddings(prompts: list[str]) -> dict[str, np.ndarray]:
    """Each prompt's embedding, from the cache or made (all at once)."""
    cache = OUT / "embed"
    cache.mkdir(parents=True, exist_ok=True)
    m = models()["llm2vec-llama3-q8"]
    key = lambda p: hashlib.sha1((m["revision"] + "\n" + p).encode()).hexdigest()
    have = {p: np.load(cache / f"{key(p)}.npy") for p in prompts if (cache / f"{key(p)}.npy").exists()}
    todo = [p for p in prompts if p not in have]
    if todo:
        snap = snapshot(m)
        print(f"encoding {len(todo)} prompt(s) with {m['repo']}")
        enc = Encoder(snap / m["weights"], snap / "tokenizer.gguf")
        for p, e in zip(todo, enc.encode(todo)):
            np.save(cache / f"{key(p)}.npy", e)
            have[p] = e
    return have


# ---------------------------------------------------------------- the motion


class Motion:
    """The motion model and its statistics."""

    def __init__(self):
        from kimodo_mlx.motion import MotionModel

        self.dir = snapshot(models()["kimodo-soma-seed"])
        self.model = MotionModel.load(self.dir)
        t = self.model.numpy_tensors
        self.stats = {k: t[f"stats.{k}"] for k in ("global_root.mean", "global_root.std", "body.mean", "body.std")}

    def sample(self, emb: np.ndarray, frames: int, steps: int, seeds: list[int], cfg: float) -> np.ndarray:
        """Normalised features [len(seeds), frames, 369]: one sample a seed,
        all denoised together; guidance from the text row and the empty one."""
        from kimodo_mlx.diffusion import ddim_step, make_cosine_schedule

        n = len(seeds)
        schedule = make_cosine_schedule(1000, steps)
        x = np.stack([np.random.default_rng(s).standard_normal((frames, self.model.motion_dim), dtype=np.float32) for s in seeds])
        texts = np.concatenate([np.broadcast_to(emb, (n, 4096)), np.zeros((n, 4096), np.float32)]).astype(np.float32)
        for index in range(steps - 1, -1, -1):
            t = int(schedule.use_timesteps[index])
            preds = self.model.denoise(np.concatenate([x, x]), texts, t)
            pred = preds[n:] + cfg * (preds[:n] - preds[n:])
            for k in range(n):
                flat = x[k].reshape(-1)
                ddim_step(schedule, index, flat, pred[k].reshape(-1), flat)
        return x

    def decode(self, x: np.ndarray) -> dict[str, np.ndarray]:
        """One sample's features → the world: positions [T,30,3], foot
        contacts [T,4] (0..1: left heel, left toe, right heel, right toe),
        the smooth root [T,3], the heading [T,2], global 6D rotations."""
        s = self.stats
        scale = lambda std: np.sqrt(std * std + 1e-5).astype(np.float32)
        f = x.copy()
        f[:, :5] = f[:, :5] * scale(s["global_root.std"]) + s["global_root.mean"]
        f[:, 5:] = f[:, 5:] * scale(s["body.std"]) + s["body.mean"]
        T = f.shape[0]
        root = f[:, 0:3]
        pos = f[:, 5:95].reshape(T, 30, 3).copy()
        pos[:, :, 0] += root[:, None, 0]
        pos[:, :, 2] += root[:, None, 2]
        return {
            "pos": pos,
            "root": root,
            "heading": f[:, 3:5],
            "rot6": f[:, 95:275].reshape(T, 30, 6),
            "contacts": np.clip(f[:, 365:369], 0.0, 1.0),
        }
