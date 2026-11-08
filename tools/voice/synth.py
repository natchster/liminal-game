"""Text-to-speech for the game's voices, on MLX (docs/VOICES.md).

Runs in tools/voice/synth (`uv run --locked --project tools/voice/synth`);
`cargo xtask voice` calls it. Every model is loaded from the Hugging Face
cache at the revision tools/voice/models.toml pins, offline.

    synth.py say --model ID --text T --out F.wav [--seed N] [--instruct S]
                 [--ref F.wav --ref-text T] [--emotion E --intensity X]
    synth.py jobs JOBS.json

A job: {"model", "text", "out", "seed", and as the model needs "instruct"
(the voice described, or the line directed), "ref_audio" and "ref_text" (the
persona's reference clip), "emotion" and "intensity" (0-1)}. Each take is
written as 32-bit float WAV at the model's rate with a JSON beside it saying
how it was made; cleaning (trim, loudness) is the evaluation's.
"""

import argparse
import json
import pathlib
import random
import struct
import sys
import time

# never a request: the models are fetched by fetch.py, at their revisions
from common import MODELS, pin_hub_loads, snapshot

pin_hub_loads()

import mlx.core as mx  # noqa: E402
import numpy as np  # noqa: E402
from mlx_audio.tts.utils import load_model  # noqa: E402
from mlx_audio.utils import load_audio  # noqa: E402

# Ming-3.0's emotion caption takes the labels it was trained on (Chinese);
# ours map to the nearest
MING_EMOTION = {
    "unease": "恐惧",
    "fear": "恐惧",
    "terror": "恐惧",
    "desperation": "悲伤",
    "exhaustion": "悲伤",
}


def write_wav(path, audio, rate):
    """32-bit float mono WAV, byte for byte the same for the same samples
    (libsndfile stamps the time into a float file's PEAK chunk)"""
    data = np.asarray(audio, dtype="<f4").tobytes()
    fmt = struct.pack("<HHIIHH", 3, 1, rate, rate * 4, 4, 32)
    body = b"WAVE" + b"fmt " + struct.pack("<I", len(fmt)) + fmt + b"data" + struct.pack("<I", len(data)) + data
    pathlib.Path(path).write_bytes(b"RIFF" + struct.pack("<I", len(body)) + body)


def seed_all(seed):
    mx.random.seed(seed)
    np.random.seed(seed)
    random.seed(seed)


def ref_array(model, job):
    if not job.get("ref_audio"):
        return None
    return load_audio(job["ref_audio"], sample_rate=model.sample_rate)


def say_voxcpm2(model, job):
    """A cloned voice (the reference) directed by a style instruction:
    VoxCPM2's controllable cloning, `(direction)text`"""
    return model.generate(
        text=job["text"],
        ref_audio=ref_array(model, job),
        instruct=job.get("instruct") or None,
        cfg_value=float(job.get("cfg", 2.0)),
        inference_timesteps=int(job.get("steps", 10)),
    )


def say_qwen3_design(model, job):
    """A voice designed from its description; the line's direction is added
    to the description (Qwen3-TTS takes no instruction on a cloned voice)"""
    return model.generate(text=job["text"], instruct=job["instruct"], lang_code="english")


def say_chatterbox(model, job):
    """A cloned voice; exaggeration from the line's intensity (0.3 calm to
    0.9 screaming), slower guidance for the dramatic ones"""
    x = float(job.get("intensity", 0.5))
    return model.generate(
        text=job["text"],
        ref_audio=ref_array(model, job),
        exaggeration=float(job.get("exaggeration", 0.3 + 0.6 * x)),
        cfg_weight=float(job.get("cfg", 0.3 if x >= 0.6 else 0.5)),
        temperature=float(job.get("temperature", 0.8)),
        verbose=False,
    )


def say_ming(model, job):
    """A cloned voice with an emotion: its trained caption (the label), or a
    sentence of direction when `instruct` is given"""
    from mlx_audio.tts.models.bailingmm.bailingmm import ming3_instruction

    instruct = job.get("instruct") or ming3_instruction(emotion=MING_EMOTION.get(job.get("emotion", ""), None))
    return model.generate(
        text=job["text"],
        ref_audio=ref_array(model, job),
        ref_text=job.get("ref_text"),
        instruct=instruct,
        cfg_scale=float(job.get("cfg", 2.0)),
        temperature=float(job.get("temperature", 0.0)),
        lang_code="en",
    )


SAY = {
    "voxcpm2": say_voxcpm2,
    "qwen3-tts-voicedesign": say_qwen3_design,
    "chatterbox": say_chatterbox,
    "ming-tts": say_ming,
}


def run(jobs):
    by_model = {}
    for j in jobs:
        by_model.setdefault(j["model"], []).append(j)
    from importlib.metadata import version

    for model_id, js in by_model.items():
        if model_id not in SAY:
            sys.exit(f"{model_id}: not a synthesis model this script can run ({', '.join(SAY)})")
        t0 = time.perf_counter()
        model = load_model(snapshot(model_id))
        print(f"{model_id}: loaded in {time.perf_counter() - t0:.1f} s", flush=True)
        for j in js:
            out = pathlib.Path(j["out"])
            out.parent.mkdir(parents=True, exist_ok=True)
            seed_all(int(j.get("seed", 0)))
            t1 = time.perf_counter()
            parts = [np.array(r.audio, dtype=np.float32) for r in SAY[model_id](model, j)]
            audio = np.concatenate(parts) if parts else np.zeros(0, np.float32)
            secs = time.perf_counter() - t1
            write_wav(out, audio, model.sample_rate)
            m = MODELS[model_id]
            meta = {k: j[k] for k in j if k != "out"}
            meta.update(
                repo=m["repo"],
                revision=m["revision"],
                upstream=m.get("upstream"),
                licence=m["licence"],
                backend=f"mlx {version('mlx')}, mlx-audio {version('mlx-audio')}",
                sample_rate=model.sample_rate,
                seconds=round(len(audio) / model.sample_rate, 3),
                made_in_s=round(secs, 2),
            )
            out.with_suffix(".json").write_text(json.dumps(meta, indent=1, ensure_ascii=False) + "\n")
            print(f"  {out.name}: {meta['seconds']:.2f} s of audio in {secs:.1f} s", flush=True)
        del model
        mx.clear_cache()


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("say")
    s.add_argument("--model", required=True)
    s.add_argument("--text", required=True)
    s.add_argument("--out", required=True)
    s.add_argument("--seed", type=int, default=0)
    s.add_argument("--instruct")
    s.add_argument("--ref")
    s.add_argument("--ref-text")
    s.add_argument("--emotion")
    s.add_argument("--intensity", type=float)
    j = sub.add_parser("jobs")
    j.add_argument("file")
    a = ap.parse_args()
    if a.cmd == "say":
        job = {"model": a.model, "text": a.text, "out": a.out, "seed": a.seed}
        for k, v in [("instruct", a.instruct), ("ref_audio", a.ref), ("ref_text", a.ref_text), ("emotion", a.emotion), ("intensity", a.intensity)]:
            if v is not None:
                job[k] = v
        run([job])
    else:
        run(json.loads(pathlib.Path(a.file).read_text())["jobs"])


if __name__ == "__main__":
    main()
