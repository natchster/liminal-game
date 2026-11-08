"""What the voice scripts share: the pinned models (models.toml), where each
is in the Hugging Face cache, and a take read as mono float32. Imported by
synth.py (tools/voice/synth) and evaluate.py, judge.py (tools/voice/eval);
it reads nothing over the network (the models are fetched by fetch.py)."""

import os
import pathlib
import tomllib

os.environ["HF_HUB_OFFLINE"] = "1"
os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")

HERE = pathlib.Path(__file__).resolve().parent
MODELS = {m["id"]: m for m in tomllib.loads((HERE / "models.toml").read_text())["model"]}


def snapshot(model_id):
    """The model's directory in the cache, at its pinned revision"""
    from huggingface_hub import snapshot_download

    m = MODELS[model_id]
    return snapshot_download(m["repo"], revision=m["revision"], allow_patterns=m["include"], local_files_only=True)


def load(path, rate):
    """A take as mono float32 at `rate`"""
    import numpy as np
    import soundfile as sf
    from scipy.signal import resample_poly

    a, r = sf.read(path, dtype="float32", always_2d=True)
    a = a.mean(axis=1)
    if r != rate:
        g = np.gcd(r, rate)
        a = resample_poly(a, rate // g, r // g).astype(np.float32)
    return a


def pin_hub_loads():
    """Make every load from the hub use the pinned revision: a model's code
    that fetches a repository of its own with no revision (Chatterbox loads
    CosyVoice 2's speech tokenizer so) gets the one models.toml pins, and a
    repository models.toml does not pin is refused"""
    import huggingface_hub

    by_repo = {m["repo"]: m for m in MODELS.values()}

    def pinned(fn):
        def call(repo_id, *a, **kw):
            m = by_repo.get(repo_id)
            if m is None and not os.path.isdir(str(repo_id)):
                raise RuntimeError(f"{repo_id} is not pinned in tools/voice/models.toml: pin it (and check its licence) before a model may load it")
            if m is not None and not kw.get("revision"):
                kw["revision"] = m["revision"]
            return fn(repo_id, *a, **kw)

        return call

    if not getattr(huggingface_hub, "_backrooms_pinned", False):
        huggingface_hub.snapshot_download = pinned(huggingface_hub.snapshot_download)
        huggingface_hub.hf_hub_download = pinned(huggingface_hub.hf_hub_download)
        huggingface_hub._backrooms_pinned = True
