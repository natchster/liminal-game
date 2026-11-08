"""Checks a voice take (docs/VOICES.md, The evaluation).

Runs in tools/voice/eval (`uv run --locked --project tools/voice/eval`);
`cargo xtask voice` calls it. Every model is loaded from the Hugging Face
cache at the revision tools/voice/models.toml pins, offline.

    evaluate.py hear --text T TAKE.wav     the words: two recognisers, the word error rate
    evaluate.py clean --lufs L --peak P RAW.wav OUT.wav   trimmed, 48 kHz, at loudness L, true peak under P
    evaluate.py clean-all ITEMS.json       each {"src", "dst", "lufs", "peak"}: a JSON line each
"""

import argparse
import functools
import pathlib
import sys

from common import load, pin_hub_loads, snapshot

pin_hub_loads()

import numpy as np  # noqa: E402
import soundfile as sf  # noqa: E402


# ---------------------------------------------------------------- cleaning

GAME_RATE = 48000


def write_pcm16(path, audio, rate):
    """16-bit mono WAV, byte for byte the same for the same samples (no
    dither: a voice at game loudness is far above its quantization)"""
    import struct

    q = np.clip(np.round(np.asarray(audio, np.float64) * 32767.0), -32768, 32767).astype("<i2").tobytes()
    fmt = struct.pack("<HHIIHH", 1, 1, rate, rate * 2, 2, 16)
    body = b"WAVE" + b"fmt " + struct.pack("<I", len(fmt)) + fmt + b"data" + struct.pack("<I", len(q)) + q
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    pathlib.Path(path).write_bytes(b"RIFF" + struct.pack("<I", len(body)) + body)


def true_peak_db(a, rate):
    """The true peak (4x oversampled, as BS.1770-4), dBTP"""
    from scipy.signal import resample_poly

    up = resample_poly(a, 4, 1)
    return 20 * np.log10(max(np.abs(up).max(), 1e-9))


def lufs(a, rate):
    import pyloudnorm

    # a clip shorter than the meter's 400 ms block is padded with silence
    # (the gate then ignores it)
    if len(a) < int(0.45 * rate):
        a = np.concatenate([a, np.zeros(int(0.45 * rate) - len(a), a.dtype)])
    return float(pyloudnorm.Meter(rate).integrated_loudness(a))


def trim(a, rate, floor_db=-45.0, pre=0.06, post=0.18):
    """Silence off both ends: from 60 ms before the first 10 ms window
    within `floor_db` of the loudest to 180 ms after the last, with short
    fades so the cut never clicks"""
    win = int(0.01 * rate)
    n = len(a) // win
    if n == 0:
        return a
    rms = np.sqrt(np.mean(a[: n * win].reshape(n, win) ** 2, axis=1) + 1e-12)
    db = 20 * np.log10(rms)
    loud = np.where(db > db.max() + floor_db)[0]
    s = max(0, loud[0] * win - int(pre * rate))
    e = min(len(a), (loud[-1] + 1) * win + int(post * rate))
    out = a[s:e].copy()
    fi, fo = int(0.005 * rate), int(0.03 * rate)
    out[:fi] *= np.linspace(0, 1, fi)
    out[-fo:] *= np.linspace(1, 0, fo)
    return out


def clean(src, dst, target_lufs, peak_dbtp):
    """A raw take made ready for the game: trimmed, at 48 kHz, at the
    persona's loudness, its true peak under the ceiling (the gain lowered,
    never limited: a scream keeps its shape and is quieter instead)"""
    a = trim(load(src, GAME_RATE), GAME_RATE)
    l0 = lufs(a, GAME_RATE)
    g = 10 ** ((target_lufs - l0) / 20)
    tp = true_peak_db(a * g, GAME_RATE)
    if tp > peak_dbtp:
        g *= 10 ** ((peak_dbtp - tp - 0.05) / 20)
    out = (a * g).astype(np.float32)
    write_pcm16(dst, out, GAME_RATE)
    back = load(dst, GAME_RATE)
    return {"seconds": round(len(out) / GAME_RATE, 3), "lufs": round(float(lufs(back, GAME_RATE)), 2), "true_peak_db": round(float(true_peak_db(back, GAME_RATE)), 2), "gain_db": round(float(20 * np.log10(g)), 2)}


# ---------------------------------------------------------------- the words


@functools.cache
def normalizer():
    # Whisper's English normalizer (numbers, contractions, punctuation); no
    # British-to-American spelling map: our lines are written in one
    from transformers.models.whisper.english_normalizer import EnglishTextNormalizer

    return EnglishTextNormalizer({})


def normalize(text):
    return " ".join(normalizer()(text).split())


def whisper(path):
    """What Whisper hears, and when each word is said ([(word, start, end)])"""
    import mlx_whisper

    r = mlx_whisper.transcribe(load(path, 16000), path_or_hf_repo=snapshot("whisper"), language="en", word_timestamps=True, temperature=0.0)
    words = [(w["word"].strip(), float(w["start"]), float(w["end"])) for s in r.get("segments", []) for w in s.get("words", [])]
    return r["text"].strip(), words


@functools.cache
def parakeet_model():
    from parakeet_mlx import from_pretrained

    return from_pretrained(snapshot("parakeet"))


def parakeet(path):
    import tempfile

    # it reads a file at its own rate
    with tempfile.NamedTemporaryFile(suffix=".wav") as f:
        sf.write(f.name, load(path, 16000), 16000)
        return parakeet_model().transcribe(f.name).text.strip()


def wer(ref, hyp):
    import jiwer

    r, h = normalize(ref), normalize(hyp)
    if not r:
        return 0.0 if not h else 1.0
    return float(jiwer.wer(r, h))


def hear(path, text):
    """The words of a take: both recognisers, each one's error rate against
    the text, and whether they agree with each other"""
    w_text, words = whisper(path)
    p_text = parakeet(path)
    return {
        "whisper": w_text,
        "parakeet": p_text,
        "wer": wer(text, w_text),
        "wer_parakeet": wer(text, p_text),
        "disagree": wer(w_text, p_text),
        "words": words,
    }


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    h = sub.add_parser("hear")
    h.add_argument("--text", required=True)
    h.add_argument("take")
    c = sub.add_parser("clean")
    c.add_argument("--lufs", type=float, required=True)
    c.add_argument("--peak", type=float, required=True)
    c.add_argument("src")
    c.add_argument("dst")
    ca = sub.add_parser("clean-all")
    ca.add_argument("file")
    a = ap.parse_args()
    if a.cmd == "clean-all":
        import json

        for it in json.loads(pathlib.Path(a.file).read_text())["items"]:
            r = clean(it["src"], it["dst"], float(it["lufs"]), float(it["peak"]))
            r["dst"] = it["dst"]
            print(json.dumps(r), flush=True)
        return
    if a.cmd == "clean":
        import json

        print(json.dumps(clean(a.src, a.dst, a.lufs, a.peak)))
        return
    if a.cmd == "hear":
        r = hear(a.take, a.text)
        print(f"whisper  {r['whisper']}")
        print(f"parakeet {r['parakeet']}")
        print(f"wer {r['wer']:.3f}")
        print(f"wer_parakeet {r['wer_parakeet']:.3f}")
        sys.exit(0 if r["wer"] <= 0.15 else 1)


if __name__ == "__main__":
    main()
