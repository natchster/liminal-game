"""The evaluation of a take beyond its words (docs/design/VOICES.md, The evaluation):
its emotion, its tone of voice, its voice. Used by evaluate.py.

Every measure is a screen for a person's ear, not a verdict: no recogniser
knows desperation or exhaustion, all of them were trained on acted or
podcast speech, and they grow unsure on screams. So each check reads two
families where it can, and the thresholds are set on known-good and
known-bad takes (`cargo xtask voice selftest`).
"""

import functools
import math
import pathlib

import numpy as np

from common import load, pin_hub_loads, snapshot

pin_hub_loads()

# ---------------------------------------------------------------- emotion

# What each of our emotions is, in the recognisers' classes (a share of each
# class counts toward it: they hear frightened shouting as angry, surprised
# or disgusted as often as fearful, so those count for the loud ones) and in
# arousal (how intense: 0 calm, 1 frantic, at intensity 0.5; intensity moves
# it). The classes only rank takes; arousal and valence gate them.
EMOTION = {
    "unease": {"fear": 1.0, "surprise": 0.8, "neutral": 0.5, "sad": 0.4, "arousal": 0.45},
    "fear": {"fear": 1.0, "surprise": 0.8, "angry": 0.5, "sad": 0.4, "disgust": 0.3, "arousal": 0.62},
    "terror": {"fear": 1.0, "surprise": 0.8, "angry": 0.7, "disgust": 0.4, "arousal": 0.78},
    "desperation": {"fear": 1.0, "sad": 1.0, "surprise": 0.4, "angry": 0.3, "arousal": 0.62},
    "exhaustion": {"sad": 1.0, "neutral": 0.6, "fear": 0.5, "arousal": 0.35},
    # your asides (personas/you_aside.toml): said under the breath, so most
    # sit near a calm read's arousal (0.33-0.37 measured on his voice, V4)
    # and the gate's two sides hold them there: a shouted read of a hushed
    # line fails (`voice selftest`, aside_shouted). Only a startle or a
    # blow lifts it, never to a shout's 0.75-0.91.
    "wary": {"fear": 1.0, "neutral": 0.8, "surprise": 0.5, "sad": 0.5, "arousal": 0.38},
    "startled": {"surprise": 1.0, "fear": 1.0, "angry": 0.3, "arousal": 0.55},
    "dread": {"fear": 1.0, "sad": 0.6, "neutral": 0.5, "arousal": 0.32},
    "shaken": {"fear": 1.0, "sad": 0.8, "surprise": 0.4, "arousal": 0.45},
    "pain": {"sad": 0.8, "angry": 0.6, "fear": 0.6, "disgust": 0.3, "arousal": 0.55},
    "resolve": {"neutral": 1.0, "sad": 0.6, "fear": 0.5, "arousal": 0.36},
}

# emotion2vec+'s classes and the WavLM head's, in common names
E2V = {"angry": "angry", "disgusted": "disgust", "fearful": "fear", "happy": "happy", "neutral": "neutral", "other": "other", "sad": "sad", "surprised": "surprise"}
WAVLM = {"Angry": "angry", "Sad": "sad", "Happy": "happy", "Surprise": "surprise", "Fear": "fear", "Disgust": "disgust", "Contempt": "contempt", "Neutral": "neutral"}


@functools.cache
def e2v_model():
    from funasr import AutoModel

    return AutoModel(model=snapshot("emotion2vec"), disable_update=True, disable_pbar=True, log_level="ERROR")


def emotion2vec(path):
    """emotion2vec+'s class shares for a take"""
    r = e2v_model().generate(load(path, 16000), granularity="utterance", extract_embedding=False)[0]
    out = {}
    for label, score in zip(r["labels"], r["scores"]):
        name = label.split("/")[-1]
        if name in E2V:
            out[E2V[name]] = out.get(E2V[name], 0.0) + float(score)
    return out


class _SER:
    """The two WavLM emotion heads (3loi's SER-Odyssey baselines, MIT),
    built here rather than from their repository's code: that code is for an
    older transformers and downloads a third model as it starts. WavLM-large
    from its config, attentive statistics pooling, a small regression head;
    every weight from the head's own checkpoint."""

    def __init__(self, kind):
        import json

        import torch
        import torch.nn as nn
        from safetensors.torch import load_file
        from transformers import WavLMConfig, WavLMModel

        cfg = json.loads((pathlib.Path(snapshot(kind)) / "config.json").read_text())
        self.mean, self.std, self.rate = cfg["mean"], cfg["std"], cfg["sampling_rate"]
        self.labels = {int(k): v for k, v in cfg["id2label"].items()}
        h, n = cfg["hidden_size"], cfg["num_classes"]

        class Pool(nn.Module):
            def __init__(self):
                super().__init__()
                self.sap_linear = nn.Linear(h, h)
                self.attention = nn.Parameter(torch.zeros(h, 1))

            def forward(self, x):
                w = torch.softmax(torch.matmul(torch.tanh(self.sap_linear(x)), self.attention).squeeze(2), dim=1).unsqueeze(2)
                mu = torch.sum(x * w, dim=1)
                rh = torch.sqrt((torch.sum(x**2 * w, dim=1) - mu**2).clamp(min=1e-5))
                return torch.cat((mu, rh), 1)

        class Head(nn.Module):
            def __init__(self):
                super().__init__()
                self.fc = nn.ModuleList([nn.Sequential(nn.Linear(2 * h, h), nn.LayerNorm(h), nn.ReLU(), nn.Dropout(0.0))])
                for _ in range(cfg["classifier_hidden_layers"] - 1):
                    self.fc.append(nn.Sequential(nn.Linear(h, h), nn.LayerNorm(h), nn.ReLU(), nn.Dropout(0.0)))
                self.out = nn.Sequential(nn.Linear(h, n))

            def forward(self, x):
                for f in self.fc:
                    x = f(x)
                return self.out(x)

        wavlm_cfg = WavLMConfig.from_pretrained(snapshot("wavlm-large-config"))
        self.ssl_model, self.pool_model, self.ser_model = WavLMModel(wavlm_cfg), Pool(), Head()
        state = load_file(str(pathlib.Path(snapshot(kind)) / "model.safetensors"))
        for name, part in [("ssl_model", self.ssl_model), ("pool_model", self.pool_model), ("ser_model", self.ser_model)]:
            sub = {k[len(name) + 1 :]: v for k, v in state.items() if k.startswith(name + ".")}
            missing, unexpected = part.load_state_dict(sub, strict=False)
            # WavLM's masked-time embedding is for pre-training only
            missing = [k for k in missing if k != "masked_spec_embed"]
            if missing or unexpected:
                raise RuntimeError(f"{kind}: {name} weights do not fit: missing {missing[:4]}, unexpected {unexpected[:4]}")
            part.eval()

    def __call__(self, path):
        import torch

        a = (load(path, self.rate) - self.mean) / (self.std + 1e-6)
        with torch.no_grad():
            x = self.ssl_model(torch.tensor(a).unsqueeze(0)).last_hidden_state
            return self.ser_model(self.pool_model(x))[0].numpy(), self.labels


@functools.cache
def wavlm(kind):
    return _SER(kind)


def wavlm_run(kind, path):
    return wavlm(kind)(path)


def wavlm_categories(path):
    logits, labels = wavlm_run("wavlm-categories", path)
    p = np.exp(logits - logits.max())
    p /= p.sum()
    return {WAVLM.get(labels[i], labels[i]): float(p[i]) for i in range(len(p))}


def wavlm_dimensions(path):
    """arousal, dominance, valence, each about 0-1"""
    v, labels = wavlm_run("wavlm-dimensions", path)
    return {labels[i].lower(): float(v[i]) for i in range(len(v))}


def emotion(path, want, intensity):
    """How near a take is to the emotion `want` at `intensity`: the share of
    its classes in both recognisers, and its arousal against the arousal the
    emotion asks"""
    spec = EMOTION[want]
    e2v, cat, dims = emotion2vec(path), wavlm_categories(path), wavlm_dimensions(path)

    def share(p):
        return min(1.0, sum(p.get(k, 0.0) * w for k, w in spec.items() if k != "arousal"))

    target = spec["arousal"] + (intensity - 0.5) * 0.25
    arousal = dims.get("arousal", 0.5)
    fit = max(0.0, 1.0 - abs(arousal - target) / 0.35)
    valence = dims.get("valence", 0.5)
    classes = (share(e2v) + share(cat)) / 2
    return {
        "score": round(0.35 * classes + 0.45 * fit + 0.2 * max(0.0, min(1.0, (0.7 - valence) / 0.35)), 3),
        # the gate: as intense as asked (either way: an exhausted line is not
        # shouted), and not glad (every emotion of ours is a negative one)
        "ok": bool(abs(arousal - target) <= AROUSAL_TOL and valence <= VALENCE_MAX),
        "classes": round(classes, 3),
        "arousal": round(dims.get("arousal", float("nan")), 3),
        "arousal_wanted": round(target, 3),
        "valence": round(dims.get("valence", float("nan")), 3),
        "emotion2vec": {k: round(v, 3) for k, v in sorted(e2v.items(), key=lambda kv: -kv[1])[:3]},
        "wavlm": {k: round(v, 3) for k, v in sorted(cat.items(), key=lambda kv: -kv[1])[:3]},
    }


# ---------------------------------------------------------------- tone of voice


def prosody(path, words):
    """Pitch (median, range in semitones), voicing, breathiness (HNR),
    trembling (jitter, shimmer), rate and pauses (from the word times)"""
    import parselmouth
    from parselmouth.praat import call

    snd = parselmouth.Sound(load(path, 16000).astype(np.float64), sampling_frequency=16000)
    pitch = snd.to_pitch_ac(pitch_floor=70.0, pitch_ceiling=700.0)
    f0 = pitch.selected_array["frequency"]
    voiced = f0[f0 > 0]
    out = {"seconds": round(snd.duration, 3), "voiced": round(len(voiced) / max(1, len(f0)), 3)}
    if len(voiced) > 5:
        out["f0_median"] = round(float(np.median(voiced)), 1)
        lo, hi = np.percentile(voiced, [5, 95])
        out["f0_range_st"] = round(12 * math.log2(hi / lo), 2)
        pp = call(snd, "To PointProcess (periodic, cc)", 70.0, 700.0)
        out["jitter"] = round(call(pp, "Get jitter (local)", 0, 0, 0.0001, 0.02, 1.3), 4)
        out["shimmer"] = round(call([snd, pp], "Get shimmer (local)", 0, 0, 0.0001, 0.02, 1.3, 1.6), 4)
        hnr = snd.to_harmonicity_cc()
        out["hnr_db"] = round(call(hnr, "Get mean", 0, 0), 2)
    if words:
        span = words[-1][2] - words[0][1]
        out["rate_wps"] = round(len(words) / span, 2) if span > 0 else None
        gaps = [b[1] - a[2] for a, b in zip(words, words[1:])]
        out["longest_pause"] = round(max(gaps), 2) if gaps else 0.0
    return out


@functools.cache
def clap():
    from transformers import ClapModel, ClapProcessor

    p = snapshot("clap")
    return ClapModel.from_pretrained(p).eval(), ClapProcessor.from_pretrained(p)


@functools.cache
def clap_text(phrases):
    import torch

    m, proc = clap()
    with torch.no_grad():
        t = m.get_text_features(**proc(text=list(phrases), return_tensors="pt", padding=True))
    t = t.pooler_output if hasattr(t, "pooler_output") else t
    return torch.nn.functional.normalize(t, dim=-1)


def tone(path, like, never):
    """How much nearer a take is to the persona's tone (`like`) than to what
    it must never sound like (`never`), by a text-audio model"""
    import torch

    m, proc = clap()
    with torch.no_grad():
        a = m.get_audio_features(**proc(audio=[load(path, 48000)], sampling_rate=48000, return_tensors="pt"))
    a = a.pooler_output if hasattr(a, "pooler_output") else a
    a = torch.nn.functional.normalize(a, dim=-1)
    lk = (a @ clap_text(tuple(like)).T)[0].numpy()
    nv = (a @ clap_text(tuple(never)).T)[0].numpy()
    worst = int(np.argmax(nv))
    return {"margin": round(float(lk.max() - nv.max()), 4), "like": round(float(lk.max()), 4), "never": round(float(nv.max()), 4), "nearest_never": never[worst]}


# AudioSet labels that tell the style of a take
STYLE = ["Male speech, man speaking", "Female speech, woman speaking", "Narration, monologue", "Speech synthesizer", "Shout", "Yell", "Screaming", "Whispering", "Crying, sobbing", "Whimper", "Gasp", "Sigh", "Breathing", "Pant", "Groan"]


@functools.cache
def ast():
    from transformers import ASTFeatureExtractor, ASTForAudioClassification

    p = snapshot("ast-audioset")
    return ASTForAudioClassification.from_pretrained(p).eval(), ASTFeatureExtractor.from_pretrained(p)


def style(path):
    """AudioSet's view of a take: each style label's probability (sigmoid:
    labels are not exclusive)"""
    import torch

    m, fx = ast()
    with torch.no_grad():
        logits = m(**fx(load(path, 16000), sampling_rate=16000, return_tensors="pt")).logits[0]
    p = torch.sigmoid(logits).numpy()
    ids = m.config.label2id
    return {k: round(float(p[ids[k]]), 3) for k in STYLE if k in ids}


# ---------------------------------------------------------------- the voice


@functools.cache
def ecapa():
    from speechbrain.inference.speaker import EncoderClassifier

    return EncoderClassifier.from_hparams(source=snapshot("ecapa"), run_opts={"device": "cpu"}, savedir=None)


@functools.cache
def embed(path):
    import torch

    e = ecapa().encode_batch(torch.tensor(load(path, 16000)).unsqueeze(0))[0, 0]
    return torch.nn.functional.normalize(e, dim=-1).numpy()


def similarity(path, reference):
    """Speaker similarity (cosine of ECAPA embeddings) to the persona's reference"""
    return round(float(np.dot(embed(path), embed(reference))), 4)


# ---------------------------------------------------------------- the report


def judge_take(t, persona, reference):
    """Every check of one take: its words, emotion, tone of voice and voice,
    and whether it passes each (the persona's targets)"""
    from evaluate import hear

    tg = persona["targets"]
    reference = t.get("reference") or reference
    r = {"line": t["line"], "take": t["take"], "path": t["path"]}
    # a take the model left (nearly) silent
    a = load(t["path"], 16000)
    r["silent"] = bool(len(a) == 0 or float(np.abs(a).max()) < 0.02)
    words = hear(t["path"], t["text"])
    r["words"] = {k: words[k] for k in ("whisper", "parakeet", "wer", "wer_parakeet", "disagree")}
    if t.get("emotion"):
        r["emotion"] = emotion(t["path"], t["emotion"], float(t["intensity"]))
    else:
        # a persona's calm reference: no emotion asked of it
        dims = wavlm_dimensions(t["path"])
        r["emotion"] = {"score": 0.0, "ok": True, "classes": 0.0, "arousal": round(dims.get("arousal", float("nan")), 3), "arousal_wanted": None, "valence": round(dims.get("valence", float("nan")), 3), "emotion2vec": {}, "wavlm": {}}
    r["prosody"] = prosody(t["path"], words["words"])
    r["tone"] = tone(t["path"], persona["tone"]["like"], persona["tone"]["never"])
    r["style"] = style(t["path"])
    r["similarity"] = similarity(t["path"], reference)
    f0 = r["prosody"].get("f0_median")
    lo, hi = tg["f0_median_hz"]
    r["checks"] = {
        "words": words["wer"] <= tg["wer_max"],
        "emotion": r["emotion"]["ok"],
        "pitch": f0 is not None and lo <= f0 <= hi,
        "tone": r["tone"]["margin"] >= TONE_MIN,
        "voice": r["similarity"] >= similarity_floor(tg, float(t["intensity"])),
        "not_synthetic": r["style"].get("Speech synthesizer", 0.0) < SYNTH_MAX,
    }
    r["passes"] = all(r["checks"].values())
    # the best take of a line: the emotion most, then the voice, the tone, the words
    r["score"] = round(0.4 * r["emotion"]["score"] + 0.25 * r["similarity"] + 0.2 * (0.5 + r["tone"]["margin"] * 5) + 0.15 * (1 - min(1.0, words["wer"])), 4)
    return r


# the thresholds, set on known-good and known-bad takes (voice selftest):
# a calm read of a terrified line is 0.33-0.37 in arousal where 0.80-0.86 is
# wanted; the good loud takes are 0.75-0.91
AROUSAL_TOL = 0.25
VALENCE_MAX = 0.55
TONE_MIN = 0.0
SYNTH_MAX = 0.5


def similarity_floor(targets, intensity):
    """The least speaker similarity a take may have: the persona's
    `similarity_min` up to intensity 0.4, falling to `similarity_min_loud`
    at 1 (a shout or a scream is far from a calm reference; ECAPA cannot tell
    one man's shout from another's, so on loud lines this only catches a
    voice wholly unlike his, and an ear must do the rest)"""
    t = min(1.0, max(0.0, (intensity - 0.4) / 0.6))
    return targets["similarity_min"] + (targets["similarity_min_loud"] - targets["similarity_min"]) * t


def report(job):
    """All takes of a persona's lines judged; each line's best passing take
    chosen; the gate (every line has one)"""
    persona, reference = job["persona"], job["reference"]
    takes = [judge_take(t, persona, reference) for t in job["takes"]]
    lines = {}
    for r in takes:
        lines.setdefault(r["line"], []).append(r)
    out = {"persona": persona["id"], "model": job.get("model"), "reference": reference, "lines": {}, "takes": takes}
    for line, rs in lines.items():
        ok = [r for r in rs if r["passes"]]
        best = max(ok, key=lambda r: r["score"]) if ok else None
        out["lines"][line] = {"chosen": best["take"] if best else None, "passing": [r["take"] for r in ok], "takes": len(rs)}
    out["passes"] = all(v["chosen"] is not None for v in out["lines"].values())
    return out


def markdown(rep):
    """The report for people: a line a take, the chosen ones marked"""
    rows = [f"# Voice report: {rep['persona']} ({rep.get('model')})", "", f"Reference: `{rep['reference']}`. Gate: **{'passes' if rep['passes'] else 'FAILS'}**.", ""]
    rows.append("| line | take | words: WER (Parakeet's) and what Whisper heard | emotion: arousal (wanted), valence; score | pitch Hz | tone margin (nearest never) | voice | synth? | passes |")
    rows.append("|---|---|---|---|---|---|---|---|---|")
    num = lambda v, f: "-" if v is None else format(v, f)  # noqa: E731
    for r in rep["takes"]:
        chosen = rep["lines"][r["line"]]["chosen"] == r["take"]
        c, e = r["checks"], r["emotion"]
        mark = lambda ok: "" if ok else " ✗"  # noqa: E731
        rows.append(
            f"| {r['line']}{' ★' if chosen else ''} | {r['take']}{' (silent)' if r.get('silent') else ''} "
            f"| {r['words']['wer']:.2f} ({r['words']['wer_parakeet']:.2f}){mark(c['words'])} \"{r['words']['whisper']}\" "
            f"| {num(e['arousal'], '.2f')} ({num(e['arousal_wanted'], '.2f')}), {num(e['valence'], '.2f')}{mark(c['emotion'])}; {e['score']:.2f} "
            f"| {r['prosody'].get('f0_median', '-')}{mark(c['pitch'])} | {r['tone']['margin']:+.3f}{mark(c['tone'])} ({r['tone']['nearest_never']}) "
            f"| {r['similarity']:.2f}{mark(c['voice'])} | {r['style'].get('Speech synthesizer', 0):.2f}{mark(c['not_synthetic'])} | {'yes' if r['passes'] else 'no'} |"
        )
    rows += [
        "",
        "★ chosen (the best score among the takes that pass). ✗ marks the check that failed: the emotion's is arousal and valence, never the score, which only ranks.",
        "The machine screens; a person makes the last choice by ear (docs/research/PLAYTEST.md).",
    ]
    return "\n".join(rows) + "\n"
