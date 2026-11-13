# Local TTS for game voice lines on a 64 GB Apple Silicon Mac: research and toolchain

Researched 2026-11-08 from the web pages listed under Sources: model cards, READMEs, licence files, leaderboards and reviews. Nothing was installed or downloaded. Where a fact comes only from a secondary source, or could not be confirmed, the text says so. **Licence facts are as published on 2026-11-08. Licences change: Spark-TTS moved from Apache-2.0 to CC-BY-NC-SA. So snapshot the LICENSE file and the model card revision hash on the day you download.**

Use case: short voice lines for named personas. The first is a frightened man crying for help in an endless concrete building. Each persona needs:
- emotion and intensity control (fear, desperation, exhaustion, whisper, shout);
- nonverbals where possible (gasps, sobs, screams);
- the same designed voice across many lines;
- correct English;
- code AND weights that allow commercial use of the output.

---

## 1. Recommendation (short)

| Role | Pick | Licence (code / weights) | Why |
|---|---|---|---|
| **Voice designer** (makes each persona's reference clip from a text description; no real person) | **Qwen3-TTS-12Hz-1.7B-VoiceDesign** (alternative: VoxCPM2's own design mode) | Apache-2.0 / Apache-2.0 | Best-documented description-to-voice model with a permissive licence. The README documents a "design, then clone" workflow. Lowest English WER of the group (Seed-TTS test-en 1.24). |
| **Primary line synthesis** | **VoxCPM2** (OpenBMB, 2B) | Apache-2.0 / Apache-2.0 ("free for commercial use") | The only permissive model that documents *style or emotion instructions on top of a cloned voice* ("controllable cloning"). Clone the persona reference, then add a per-line prefix such as `(terrified, breathless, whispering)`. That gives a consistent voice with directed emotion. Runs on MPS (official), on MLX (mlx-audio, bf16/8-bit/4-bit) and on llama.cpp Metal. On its own InstructTTSEval-EN table it leads Qwen3-TTS-VD. |
| **Second model: intensity dial and nonverbals** | **Chatterbox** (original 0.5B for the `exaggeration` knob; **Turbo** 350M for paralinguistic tags) | MIT / MIT | Clones the same persona reference. `exaggeration` is a continuous intensity knob, useful for fear rising to desperation to screaming. Turbo has inline tags: `[laugh] [chuckle] [cough]` are official; `[sigh] [gasp] [groan] [sniff] [clear throat] [shush]` come from secondary sources. There is an official Mac example (MPS) and an MLX port. Caveat: retired in August 2026 (archived, MIT, no updates). |
| **Nonverbal fallback** (screams, gasps, sobbing as separate clips) | **Dia 1.6B** with the persona clip as audio prompt (`(screams) (gasps) (inhales) (exhales) (groans)`); or **Maya1** for `<cry> <scream> <gasp> <whisper>` | Apache-2.0 / Apache-2.0 (both) | Dia keeps the voice through an audio prompt; on a Mac it runs only through mlx-audio. Maya1 has the richest permissive tag set, including crying and screaming, but no cloning, so its voice drifts. Use it only for short clips. |
| **Watch and bake-off candidate** | **Ming-3.0-tts-0.5B** (Ant inclusionAI, Oct 2026) | MIT / MIT (as stated by the card and repo) | Explicit `EMOTION` task with a **fear** label on a cloned reference, voice design, and 14 inline tags (breathing, sigh, cough, laughter and others; no gasp, sob or scream). Elo 1187 on the Artificial Analysis open-weights arena (4th). Very new ("quiet release", unfinished tech report), tested only on CUDA. MLX builds exist in mlx-audio. |
| **ASR (are the words right)** | **mlx-whisper** with whisper-large-v3-turbo; second opinion from **parakeet-mlx** with Parakeet-TDT-0.6B-v3; **jiwer** for WER | MIT / MIT; Apache-2.0 / CC-BY-4.0; Apache-2.0 | Native MLX on the GPU, with word timestamps (which also give speech rate). Two different ASR families make hallucinations easier to catch. |
| **Emotion check** | **emotion2vec+ large** (9 classes incl. *fearful*); **3loi Odyssey WavLM** categorical (8 classes incl. *Fear*) and multi-attribute (arousal, valence, dominance) | emotion2vec: MIT code, weights under the custom FunASR Model Licence (flagged); 3loi: MIT | No model scores "desperation" or "exhaustion". Combine the fear and sad probabilities, arousal, and prosody (section 5). |
| **Nonverbal and style check** | **AST AudioSet** classifier (`MIT/ast-finetuned-audioset-10-10-0.4593`) | BSD-3-Clause | Has exact labels: Screaming, Gasp, "Crying, sobbing", Whimper, "Wail, moan", Whispering, Shout, Yell, Sigh, Breathing, Pant, Groan. |
| **Voice consistency check** | **SpeechBrain ECAPA** (`speechbrain/spkrec-ecapa-voxceleb`) cosine similarity to the persona reference centroid | Apache-2.0 | Catches takes that drifted away from the persona voice. |
| **Prosody** | **praat-parselmouth** (F0, jitter, shimmer, HNR, voicing), **librosa** (pyin, RMS), **torchcrepe** (robust F0), **pyloudnorm** (LUFS) | GPL-3.0 (internal tool only), ISC, MIT, MIT | Pitch, rate, loudness and breathiness. pyloudnorm also normalises loudness on the lines you ship. |
| **Environment** | **uv 0.13.7, Python 3.12**, **torch 2.15.0** (MPS), **mlx 0.32.4**, **mlx-audio 0.6.4**; one uv venv per model family | — | Version pins conflict (section 6). Chatterbox wants its own Python 3.11 venv with torch 2.6.0. |

**Why VoxCPM2 over Qwen3-TTS as the primary.** Qwen3-TTS's README limits instruction control to the VoiceDesign and CustomVoice models. The Base model, which does the cloning, has none. So with Qwen3 you can have either a fixed designed voice (clone, no emotion instruction) or emotion instructions (with the voice re-designed on every line, so it drifts). The other option is Qwen's preset CustomVoice speakers. VoxCPM2 documents both together: a cloned reference plus a style instruction. No independent emotional listening test compares the two (section 7). **Before committing, run a 10-line bake-off of VoxCPM2, Qwen3-TTS, Chatterbox and Ming-3.0 with the evaluation pipeline in section 5.**

**Per-persona workflow:**
1. Write a voice description: age, timbre, accent, texture. Never use a real person's or celebrity's name.
2. Generate 5–10 neutral 10–20 s reference candidates with Qwen3-TTS VoiceDesign or VoxCPM2 design. Pick one and keep it as the persona's "voice bible" clip, with its transcript, seed, model revision and licence snapshot.
3. Optionally create emotional reference variants with VoxCPM2 controllable cloning, e.g. `(terrified, trembling)` or `(exhausted, hoarse)`. Keep only variants that stay within your speaker-similarity band. These become references for Chatterbox, whose emotion category comes from the reference clip.
4. Generate each line in 3–5 takes:
   - VoxCPM2 with a style prefix;
   - Chatterbox with an `exaggeration` sweep (0.5 / 0.7 / 0.9) and `cfg_weight` ≈ 0.3 for dramatic reads;
   - nonverbal stingers separately.
5. Auto-screen and rank the takes (section 5). A human makes the final pick by ear.
6. Normalise loudness (pyloudnorm), export, and write a provenance manifest line per shipped file: model, revision, licence, seed, prompt, reference clip.

---

## 2. Comparison table

Legend for **Output for commercial game?**
- **YES**: permissive code and weights.
- **FLAG**: allowed, but with extra terms or unclear points.
- **NO**: non-commercial or restricted. Excluded.

"n/s" means not stated in the sources read. RTF conventions differ between sources and are quoted as given.

### 2a. Models that pass the licence gate

| Model (org, release) | Code licence | Weights licence | Output for commercial game? | Size / memory | Apple Silicon support, rough speed | Emotion / intensity control | Nonverbals | Voice consistency / cloning / design | Quality notes |
|---|---|---|---|---|---|---|---|---|---|
| **VoxCPM2** (OpenBMB, Apr 2026) | Apache-2.0 | Apache-2.0 ("free for commercial use") | **YES** | 2B. bf16 ≈5 GB; int8 ≈3.0 GB, int4 ≈1.9 GB (MLX). ~8 GB VRAM on CUDA | **MPS** official (`--device auto`/`mps` in web demo). **MLX** in mlx-audio (`mlx-community/VoxCPM2-bf16/8bit/4bit`). llama.cpp-omni GGUF on Metal: RTF ≈1.76 (Q8_0, M4 Pro). The mlx-community card gives 0.48× (bf16) to 0.90× (4-bit) speed, where >1 means faster than real time. Roughly 1–2× slower than real time. Needs Python <3.13 | Text prefix `(style…)`, either on voice design or on a **cloned voice** ("controllable cloning": emotion, pace, expression). Whisper, shout and fear are **not explicitly documented** | Not documented | Voice design from a description; controllable cloning; "ultimate cloning" (reference + transcript); 48 kHz output | Seed-TTS test-en WER 1.84, SIM 75.3. InstructTTSEval-EN 84.2/83.2/71.4 (vendor table, ahead of Qwen3-TTS-VD). Results vary between runs (card advises 1–3 takes) |
| **Qwen3-TTS** 0.6B/1.7B: Base, CustomVoice, VoiceDesign (Alibaba, Jan 2026) | Apache-2.0 | Apache-2.0 | **YES** | 1.7B ≈3.4 GB bf16 weights. ~4.5–6 GB peak on MLX (community) | **MLX** in mlx-audio (`mlx-community/Qwen3-TTS-12Hz-1.7B-VoiceDesign-bf16` and others). **MPS** works per community reports (one says the 0.6B Base needs fp32). M3 Max 64 GB, MPS, 1.7B CustomVoice: RTF ≈2.5–3.0 (≈3 s of compute per 1 s of audio) | Natural-language `instruct` (timbre, emotion, prosody). **Only in VoiceDesign and CustomVoice, not in Base (the cloning model)** | Not documented | VoiceDesign from a description; Base 3-second clone; preset CustomVoice speakers; documented design-then-clone workflow (`create_voice_clone_prompt`) | Seed-TTS test-en WER 1.24 (lowest here). InstructTTSEval-EN (VD) 82.9/82.4/68.4 |
| **Chatterbox** (0.5B), **Turbo** (350M), Multilingual V3 (0.5B), Nano (110M) (Resemble AI, 2025; **retired Aug 2026**) | MIT | MIT | **YES**. Every file carries an imperceptible **PerTh watermark**; no off switch is documented | 0.5B "Llama-architecture" backbone (an architecture, not Meta weights) | **MPS**: official `example_for_mac.py` (device `mps`, patched `torch.load`). **MLX**: mlx-audio (`mlx-community/chatterbox-fp16`, `chatterbox-multilingual-v3`) and the `chatterbox-mlx` fork (claims 0.51× on MPS vs 0.71–0.74× on MLX, M4 32 GB; direction of the metric unclear). Nano ≈3× real time on 8 CPU cores | `exaggeration` (default 0.5; ≥0.7 for drama) plus `cfg_weight` (≈0.3 to slow pacing). The emotion category comes from the reference clip. Turbo's card shows no exaggeration control | Turbo: `[laugh] [chuckle] [cough]` "and more" (official). `[sigh] [gasp] [clear throat] [sniff] [groan] [shush]` per fal.ai (not confirmed officially) | Zero-shot clone from a ~10 s reference | AA arena Elo 1023. Vendor (Podonos) preference test vs ElevenLabs. Archived; pins torch 2.6.0 |
| **Ming-3.0-tts-0.5B / Tiny** (Ant inclusionAI, Sep–Oct 2026) | MIT (LICENSE file in repo) | MIT (card: "Released under the MIT license"; weights not separately addressed) | **YES**, but verify at download (new release) | 0.5B (Qwen2.5-0.5B + DiT head). 2.96 GB safetensors. Tiny repo ≈18 GB | **MLX** in mlx-audio (`mlx-community/Ming-3.0-tts-0.5B-bf16`, tiny 8/4-bit). Official code tested only on NVIDIA H800/H20. No Mac speed data | `task="EMOTION"` with a label (happy, sad, angry, **fear**, disgust, surprise, neutral) on a reference voice. Separate speed, pitch and volume tasks (they can't be combined) | 14 tags: `[Breathing] [Cough] [Sigh] [Laughter] [Surprise-ah] [Surprise-oh] [Dissatisfaction-hnn] [Uhm] [Confirmation-en] [Question-ei/ah/en/oh/huh]`. No gasp, sob or scream | Voice design (style text); clone with transcript | **AA arena Elo 1187 (#4 open-weights)**. Seed EN WER 1.58, SIM 0.691. Tech report unfinished |
| **Dia 1.6B** (Nari Labs, Apr 2025); **Dia2** 1B/2B (2025) | Apache-2.0 | Apache-2.0 (Dia2 card explicit; Dia per yaps.ai). The Mimi codec keeps its own licence | **YES** | Dia ≈4.4 GB bf16 / 7.9 GB fp32 | Official: GPU only ("CPU support soon"). **MLX** in mlx-audio (Dia 1.6B). Dia2 needs CUDA 12.8+. No Mac speed data | None explicit (from text and audio prompt) | 21 tags: `(laughs) (clears throat) (sighs) (gasps) (coughs) (singing) (sings) (mumbles) (beep) (groans) (sniffs) (claps) (screams) (inhales) (exhales) (applause) (burps) (humming) (sneezes) (chuckle) (whistles)` | The voice changes every run unless you use an audio prompt (5–10 s plus transcript) or a fixed seed | English only. Clips under ~5 s sound unnatural; over ~20 s speech speeds up |
| **Maya1** (Maya Research, Nov 2025) | Apache-2.0 (MIT for the streaming example) | Apache-2.0 | **YES**. Llama-style architecture; whether it was initialised from Meta Llama weights is **unconfirmed** | 3.3B. 16 GB+ VRAM recommended on CUDA | No official Mac path. Community GGUF (llama.cpp). Unverified on MLX | `<description="…">` voice and style plus inline emotion tags | 20 tags: angry, appalled, chuckle, **cry**, curious, disappointed, excited, exhale, **gasp**, giggle, gulp, laugh, laugh_harder, mischievous, sarcastic, **scream**, sigh, sing, snort, **whisper** | Description only. **No cloning**, so the voice drifts between generations | AA arena Elo 1047 |
| **Kokoro-82M** (hexgrad, Jan 2025) | Apache-2.0 (GitHub; not re-checked) | Apache-2.0 | **YES** (training data includes synthetic audio from closed TTS; see risks) | 82M | **MLX** (mlx-audio, `mlx-community/Kokoro-82M-*`). MPS with `PYTORCH_ENABLE_MPS_FALLBACK=1` plus espeak-ng. Very fast | None | None | 54 preset voices (8 languages); no cloning | AA Elo 1064. Good for scratch or placeholder lines; misaki G2P allows phoneme control |
| **Zonos v0.1** transformer/hybrid (Zyphra, Feb 2025) | Apache-2.0 | Apache-2.0 | **YES** | ~2B. 6 GB+ VRAM | GitHub lists macOS (espeak-ng via brew); the HF card says "Linux only". Hybrid needs an NVIDIA 3000-series or newer. **MPS unconfirmed** | Emotion conditioning (happiness, anger, sadness, fear…), pitch variation, speaking rate | Whisper only through an audio prefix | Clone from 10–30 s | AA Elo 1000 (anchor) |
| **Parler-TTS Large v1** (HF) | Apache-2.0 | Apache-2.0 | **YES** | 2.2B | CUDA or CPU in the docs; MPS not mentioned | Text description; expressiveness only indirect ("slightly expressive") | None | 34 named speakers for consistency | No formal benchmarks |
| **CosyVoice 2 / Fun-CosyVoice3-0.5B-2512** (Alibaba FunAudioLLM, Dec 2025) | Apache-2.0 | Apache-2.0 (HF tag) | **YES** (the README disclaimer "for academic purposes only" refers to demo content) | 0.5B | Linux/NVIDIA (Docker, TensorRT-LLM). No Mac mention | Instruct (emotion, speed, volume, dialect) | `[breath]` (others unconfirmed) | Zero-shot clone | EN WER 2.24 / SIM 71.8 (RL: 1.68 / 69.5). **English CMU-phoneme pronunciation control** |
| **Kyutai TTS 1.6B** en/fr (2025) | MIT (Python/MLX), Apache (Rust) | CC-BY-4.0 | **FLAG**. Commercial use with attribution, *but* the emotional voices (Expresso, EARS) are **CC-BY-NC**. Only the CC0/CC-BY voices (voice-donations, vctk, alba-mackenna, voice-zero) are safe | 1.8B | **MLX official** (`moshi-mlx`, `scripts/tts_mlx.py`, `--quantize 8/4`) | Only through the choice of voice embedding (the emotional ones are NC) | None | Pre-computed embeddings only; no custom cloning | No watermark |
| **Pocket TTS** (Kyutai, 2026) | n/s (not checked) | CC-BY-4.0 plus prohibited-use terms | **FLAG** (attribution; cloning requires consent) | 100M | CPU ≈6× real time on M4 Air (2 cores) | None | None | 26 voices; cloning (gated repo) | — |
| **NeuTTS Air** (Neuphonic) | n/s | Apache-2.0 (built on Qwen 0.5B) | **YES** (PerTh watermark) | ~0.7B | GGUF; real-time on CPU; MPS through llama-cpp-python | None | None | 3-second clone | — |
| **Bark** (Suno, 2023) | MIT | MIT | **YES** | ~12 GB full; 8 GB small; ~2 GB offloaded | CPU or GPU; MPS not documented on the README | None explicit | `[laughs] [laughter] [sighs] [gasps] [clears throat] [music]`, `♪`, `…` | 100+ presets; no custom cloning | Dated; ~13–14 s limit; quality varies ("1980s phone call") |
| **MeloTTS** (MyShell) | MIT | MIT | **YES** | Small | CPU real time | None | None | 5 English accents | — |
| **Magpie-Multilingual 357M** (NVIDIA, Feb 2026) | NeMo | NVIDIA Open Model License ("ready for commercial use") | **YES** (NVIDIA OML terms) | 357M | Community MLX and CoreML ports | None | None | Fixed voices; "not intended for zero-shot voice cloning" | AA Elo 1064 |
| **Phonon 31M en** (Gradium, Oct 2026) | n/s | Apache-2.0 per secondary sources (the **40M variants are CC-BY-NC**) | **YES**, verify on the official card | 31M | On-device / browser; listed in mlx-audio | None | None | Preset voices; cloning only in the hosted studio | AA Elo 1114 |
| **MOSS-TTS v1.5** (OpenMOSS, May 2026) | n/s | Apache-2.0 (ThursdAI and aggregators; official card not read) | Probably **YES**, verify | 8B | Listed in mlx-audio | Pause control only reported | — | Clone | — |
| **Step-Audio-EditX** (StepFun, Nov 2025 / Jan 2026) | Apache-2.0 | apache-2.0 (HF tag) | **YES**, but **not runnable locally**: CUDA required | 3B. 12–16 GB (AWQ 4-bit 8–10 GB) | **CUDA required; no Mac** | Emotion edit (14 incl. **fear**, sad, depressed); styles incl. **whisper, shout, roar, murmur, loudly**; iterative editing | sigh, inhale, exhale, laugh, chuckle, giggle, snort, clears throat, cough, breath, uhm, surprise and question sounds | Clone, then edit | AA Elo 1101. It would be the ideal emotion editor if it ran on Mac |

### 2b. Flagged (extra terms) or excluded

| Model | Code | Weights | Verdict | Why |
|---|---|---|---|---|
| **Orpheus-TTS 3B** (Canopy) | Apache-2.0 | apache-2.0 tag, but the HF model tree's base is **meta-llama/Llama-3.2-3B-Instruct** | **FLAG** | Llama 3.2 Community Licence terms likely apply (attribution, naming, AUP). Gated repo. Tags `<laugh> <chuckle> <sigh> <cough> <sniffle> <groan> <yawn> <gasp>`; 8 preset voices; cloning "needs fixing". Mac via GGUF (LM Studio or llama.cpp). Best open model in EmergentTTS-Eval (2025) |
| **Sesame CSM-1B** | Apache-2.0 | apache-2.0 tag; gated; the repo **requires access to Llama-3.2-1B** | **FLAG** | Llama licence attaches through the tokenizer or weights dependency. No emotion control. EmergentTTS-Eval notes "flat intonation". In mlx-audio |
| **Higgs Audio v2** (Boson AI) | n/s | **Boson Higgs Audio 2 Community License**, based on Meta Llama 3 | **FLAG** | Over 100k annual active users needs a licence. Required notice "Built with Higgs Materials…". Derived models must be named "Higgs Audio 2…". Llama AUP incorporated. EmergentTTS Emotions win rate 75.7% vs gpt-4o-mini-tts |
| **StyleTTS2** | MIT | Pre-trained models: **must tell listeners speech is synthesised** unless you have the voice's permission | **FLAG** | The disclosure condition applies when reference speakers are not from open datasets. Inference needs a GPL phonemizer. AA Elo 890 |
| **Fish Audio Drama 3** (Oct 2026, 35B MoE / 3B active) | — | **Fish Audio Community License v1.0** (the HF tag says "fish-audio-research-license", a mismatch) | **FLAG / exclude** | Free commercial use only under **US$500k annual revenue** (including affiliates). Must display "**Built with Fish Audio**". Outputs may not train speech models. AA #1 (Elo 1275). H200-class hardware |
| **Tencent AuK** (Sep 2026) | MIT | MIT, **but needs Qwen2.5-Omni-3B at runtime**, which is under the **Qwen Research Licence (non-commercial only)** | **FLAG / exclude** | The inference pipeline depends on non-commercial weights |
| **VibeVoice 1.5B** (Microsoft) | MIT | MIT | **Unsuitable** | "Research and development purposes only". **An audible AI disclaimer is embedded in every file.** Large weights disabled |
| **Supertonic 3** (Supertone) | MIT sample code | OpenRAIL-M (use restrictions) | **FLAG** | Repo archived Sep 2026. Tags `<laugh> <breath> <sigh>`; no cloning |
| **Higgs TTS 3 4B** (Boson, Jun 2026) | — | Research & Non-Commercial (the Creator Use Grant excludes embedding in products) | **NO** | Otherwise excellent: 21 emotion tags incl. fear, whisper, shout, crying, screaming |
| **IndexTTS2 / 2.5** (bilibili) | bilibili licence | bilibili Model Use Licence Agreement | **NO** (unless you get an agreement) | Commercial use: "contact indexspeech@bilibili.com". 8-dim emotion vector incl. *afraid* |
| **F5-TTS** | MIT | **CC-BY-NC-4.0** (Emilia training data) | **NO** | — |
| **XTTS-v2** (Coqui) | — | **Coqui Public Model License** (non-commercial). Coqui shut down, so no commercial licence can be bought | **NO** | — |
| **Spark-TTS 0.5B** | Apache-2.0 | **CC-BY-NC-SA-4.0** (changed from Apache-2.0) | **NO** | — |
| **Voxtral TTS 4B** (Mistral, Mar 2026) | — | **CC-BY-NC-4.0** (inherited from its reference voices) | **NO** | — |
| **OmniVoice** (k2-fsa) | Apache-2.0 | **CC-BY-NC** (Emilia) | **NO** | Has whisper voice design and `[laughter] [sigh]` tags |
| **Fish Audio S2 Pro / OpenAudio S1-mini** | — | Fish Audio Research License / CC-BY-NC-SA | **NO** | — |
| **Breeze TTS 2** (Aug 2026) | — | Non-commercial (per ThursdAI) | **NO** | — |

Not assessed in depth (listed in mlx-audio or on leaderboards): Audio8 TTS, rumik-oss 1, KugelAudio, LongCat-AudioDiT, MisoTTS, MOSS-TTS-Nano, KittenTTS, Soprano 80M, Echo-TTS. Check their licences before use.

---

## 3. Independent quality evidence

- **Artificial Analysis open-weights arena** (blind preference on neutral reads, not on emotion): Fish Drama 3 1275 (FLAG/NO); Breeze TTS 2 1228 (NO); **Ming-3.0-tts 1187**; Phonon 31M 1114; Fish S2 Pro 1108 (NO); **Step-Audio-EditX 1101** (CUDA only); Voxtral 1092 (NO); Magpie 1064; **Kokoro 1064**; **Maya1 1047**; Higgs v3 1039 (NO); **Chatterbox 1023**; Zonos 1000; VibeVoice 953; XTTS-v2 915; StyleTTS2 890. **Qwen3-TTS and VoxCPM2 do not appear on that board**, so there is no arena number for them.
- **EmergentTTS-Eval (2025)**: among the open models tested (Bark, Sesame, Zonos, Tortoise, MiniCPM, Qwen2.5-Omni, Orpheus), Orpheus ranked best. It predates the 2026 models. The live community leaderboard (GitHub) was not readable.
- **No independent emotion-specific blind test** of VoxCPM2 vs Qwen3-TTS vs Chatterbox was found. A personal "JP TTS" page sets up a Qwen3-vs-VoxCPM2 emotion sweep but publishes no scores. Instruction-following numbers (InstructTTSEval) are vendor-reported. **Do your own bake-off.**
- **TTS Arena V2** (HF Space) did not render through the fetcher, so it could not be read.

---

## 4. English pronunciation

- The LLM-based models (Qwen3-TTS, VoxCPM2, Chatterbox, Ming) read text directly. There is no phoneme input documented for VoxCPM2, Chatterbox or Qwen3. Check every line with ASR (section 5). For names that are misread, respell them phonetically in the prompt.
- **CosyVoice3** documents English **CMU-phoneme** control, but it is Linux/NVIDIA-oriented. **Kokoro** (misaki G2P) accepts phonemes, but it has no emotion.
- Lowest reported English WER: Qwen3-TTS 1.24, Ming-0.5B 1.58, VoxCPM2 1.84, CosyVoice3 1.68–2.24 (Seed-TTS test-en; vendor numbers).

---

## 5. Evaluation stack (local, permissive)

The evaluation tools do not ship with the game, so their licences matter less than the synthesis licences. They are still listed, and non-permissive ones are flagged.

### 5a. Speech recognition (are the words right)

| Tool | Licence | Apple Silicon | Notes |
|---|---|---|---|
| **mlx-whisper** 0.4.3 (Aug 2025) | MIT; Whisper weights MIT | MLX on the GPU | Word-level timestamps. Use `mlx-community/whisper-large-v3-turbo`. Python ≥3.8 |
| **whisper.cpp** | MIT | Metal (whole model on GPU); encoder on Core ML / ANE (>3× vs CPU); ANEForge | Word timestamps experimental (`-ml 1`). Good CLI fallback |
| **faster-whisper** | MIT | **CPU only on Mac** (CTranslate2 GPU path is CUDA) | Fine but slower. Not recommended on this machine |
| **parakeet-mlx** with `parakeet-tdt-0.6b-v3` | Apache-2.0; weights **CC-BY-4.0** ("ready for commercial use") | MLX | Second opinion (different architecture). Word timestamps. Open ASR average WER ≈6.3% |
| mlx-audio STT | MIT | MLX | Also wraps Whisper, Parakeet, Qwen3-ASR and **Qwen3-ForcedAligner** (word alignment) |
| **jiwer** | Apache-2.0 | — | WER, CER, MER |

Practice:
- Strip nonverbal tags from the reference text.
- Normalise both texts (Whisper's English normaliser) before computing WER.
- Flag a take when the two ASR systems disagree.
- Whisper can hallucinate on clips that are only nonverbal (screams, sobs). Skip WER for those and use the AudioSet check instead.

### 5b. Speech emotion recognition (does it sound like the target emotion)

| Model | Labels | Licence | Notes |
|---|---|---|---|
| **emotion2vec+ large** (`emotion2vec/emotion2vec_plus_large`, ~300M; run via FunASR) | angry, disgusted, **fearful**, happy, neutral, other, sad, surprised, unknown | Repo badge MIT. Weights tagged "model-license" = **FunASR Model Open Source License v1.1**: requires attribution and **is silent on commercial use** (FLAG, but eval only) | Best general-purpose option. Also gives frame-level output at 50 Hz and embeddings |
| **3loi/SER-Odyssey-Baseline-WavLM-Categorical-Attributes** | Angry, Sad, Happy, Surprise, **Fear**, Disgust, Contempt, Neutral | MIT (training data MSP-Podcast has its own academic terms) | Weak: F1-macro ≈0.31 on Test 3. Use for trends only |
| **3loi/SER-Odyssey-Baseline-WavLM-Multi-Attributes** | arousal, valence, dominance (~0–1) | MIT (per card README in search results) | **Arousal is the intensity proxy**: shout and desperation high, exhaustion low |
| firdhokk/speech-emotion-recognition-with-openai-whisper-large-v3 | Angry, Disgust, **Fearful**, Happy, Neutral, Sad, Surprised | Apache-2.0. Trained on acted corpora (RAVDESS, SAVEE, TESS) that carry NC or research terms (dataset caveat) | 0.92 accuracy on acted test sets. Acted speech resembles game VO |
| superb/wav2vec2-base-superb-er | 4 IEMOCAP classes (no fear) | Apache-2.0 | Not useful here |
| audeering/wav2vec2-large-robust-12-ft-emotion-msp-dim | arousal, dominance, valence | **CC-BY-NC-SA-4.0 / research only** | **Excluded** |

No recogniser has "desperation" or "exhaustion" classes. Proxies:
- **fear**: P(fearful) and high arousal.
- **desperation**: fear and sad together, high arousal, raised F0, louder.
- **exhaustion**: low arousal, low F0 and energy, slow rate, low HNR (breathy).
- **whisper**: high unvoiced fraction (Praat voicing) plus AudioSet "Whispering".
- **shout**: high LUFS and F0 plus AudioSet "Shout"/"Yell"/"Screaming".

### 5c. Nonverbal and style detection

- **AST AudioSet** `MIT/ast-finetuned-audioset-10-10-0.4593`, **BSD-3-Clause**. Confirmed label ids: Shout 8, Yell 11, Screaming 14, Whispering 15, Laughter 16, Crying/sobbing 22, Whimper 24, Wail/moan 25, Sigh 26, Groan 38, Breathing 41, Gasp 44, Pant 45.
- **CLAP** `laion/larger_clap_general`, **Apache-2.0**. Zero-shot with free-text labels, e.g. "a man sobbing", "a terrified whisper".
- Optional audio-LLM judge: **Qwen2-Audio-7B-Instruct** (Apache-2.0; listed in mlx-audio) or **Qwen3-Omni-30B-A3B-Captioner** (Apache-2.0, 32B; mlx-vlm v0.6.0 supports Qwen3-Omni audio, but an MLX build of the captioner itself was not confirmed).

### 5d. Voice consistency and naturalness

- **SpeechBrain ECAPA** `speechbrain/spkrec-ecapa-voxceleb`, **Apache-2.0** (EER 0.80% on VoxCeleb1). Compute the cosine similarity of each take to the persona's reference centroid. Calibrate the threshold on the persona's own accepted takes. Expect lower similarity for screams and whispers.
- **UTMOSv2** (naturalness MOS predictor): MIT code; weights licence not stated.

### 5e. Prosody

| Tool | Licence | Measures |
|---|---|---|
| **praat-parselmouth** | **GPL-3.0** (Praat GPL-2.0+/3.0). Fine as an internal tool; don't distribute it inside the game | F0 mean and range, jitter, shimmer (trembling), HNR (breathiness), voiced/unvoiced fraction (whisper), intensity |
| **librosa 1.0.0** (Aug 2026) | ISC | pyin F0, RMS energy, onsets. **Requires Python ≥3.12** |
| **torchcrepe** | MIT | Robust neural F0 and periodicity (MPS not documented; CPU works) |
| **pyloudnorm** | MIT | ITU-R BS.1770-4 integrated loudness (LUFS). Also use it to normalise shipped lines |
| ASR word timestamps | — | Speaking rate (words/s or syllables/s) and pause lengths |

openSMILE is not recommended: its licence is research/non-commercial, and this was not re-checked here.

---

## 6. Python environment on Apple Silicon (as of 2026-11-08)

**Current versions:**
- **uv 0.13.7** (Nov 3 2026; MIT/Apache-2.0).
- **torch 2.15.0** (GA Oct 28 2026): macOS 14+ arm64 wheels for CPython **3.11–3.15**; no Intel wheels.
- **mlx 0.32.4** (Oct 14 2026; Python ≥3.10; macOS 14+ wheels; MIT).
- **mlx-audio 0.6.4** (Nov 2 2026; Python ≥3.10; MIT).
- **librosa 1.0.0** requires Python ≥3.12.

**Version pins that conflict:**
- **Chatterbox** pins `torch==2.6.0`, `torchaudio==2.6.0`, `transformers==5.2.0`, `librosa==0.11.0`, `numpy<2` (Python <3.13). It was tested on Python 3.11.
- **VoxCPM2** needs Python ≥3.10, **<3.13**, and torch ≥2.5.
- Step-Audio-EditX needs Python ≥3.12 and torch ≥2.9.1 (CUDA only anyway).

**Recommended layout** (separate uv projects, each locked with `uv lock`):

| venv | Python | Contents |
|---|---|---|
| `tts-mlx` | 3.12 | mlx 0.32.4 + mlx-audio 0.6.4. Runs VoxCPM2, Qwen3-TTS, Chatterbox (fp16 / multilingual-v3), Dia, Kokoro, Ming-3.0 on MLX. The fastest path to try everything |
| `tts-voxcpm` | 3.12 | `voxcpm` + torch 2.15.0 on MPS. The reference implementation, for checking parity with the MLX port |
| `tts-qwen` | 3.12 | `qwen-tts` + torch 2.15.0 on MPS (some reports need fp32 for 0.6B Base) |
| `tts-chatterbox` | 3.11 | `chatterbox-tts` with its own pins (torch 2.6.0). Use `example_for_mac.py`-style device handling |
| `eval` | 3.12 | torch 2.15.0, transformers, mlx-whisper, parakeet-mlx, jiwer, funasr (emotion2vec), speechbrain, praat-parselmouth, librosa 1.0, torchcrepe, pyloudnorm, soundfile |

**Other setup:**
- Set `PYTORCH_ENABLE_MPS_FALLBACK=1` for ops MPS lacks.
- Homebrew: `ffmpeg`, plus `espeak-ng` only if you use Kokoro, Zonos or StyleTTS2. espeak-ng is a GPL tool; using it does not change the licence of the generated audio.
- Record the HF model revision hash, sampling seed and backend (MLX or MPS) for every take. Outputs differ between backends.
- Memory: every recommended model fits easily in 64 GB together with the eval stack. The largest are Maya1 (≈7 GB bf16) and VoxCPM2 bf16 (≈5 GB).

---

## 7. Risks

1. **Licences drift.** Spark-TTS went from Apache-2.0 to NC. The Fish Drama 3 HF tag contradicts its LICENSE file. **Mitigation:** archive the LICENSE file and model card at the exact revision you use, and keep a per-file provenance manifest.
2. **Training-data provenance.** Permissive weights don't guarantee clean training data. Kokoro says it used synthetic audio from closed commercial TTS. Most 2026 models don't fully disclose their data. There is residual legal uncertainty that no open model removes.
3. **Accidental resemblance.** A designed voice could resemble a real person. Keep descriptions generic (no names, no "sounds like…"), keep the design prompts, and never clone recordings of real people. The Fish, Dia, CSM and Pocket TTS terms explicitly prohibit cloning without consent.
4. **Hidden Llama terms.** Orpheus (base Llama-3.2-3B-Instruct), CSM (requires Llama-3.2-1B) and Higgs v2 (Llama-3-based licence) carry Llama obligations despite their apache-2.0 tags. Maya1's lineage is unconfirmed.
5. **Watermarks.** Chatterbox and NeuTTS embed an inaudible PerTh watermark in every file. That is harmless for a game; leave it in. Fish's licence forbids removing watermarks. VibeVoice embeds an *audible* disclaimer, so it is unusable.
6. **Maturity and maintenance.** Chatterbox is archived (Aug 2026). Ming-3.0 is weeks old with an unfinished report. MLX ports are community-maintained, and quality parity with the reference PyTorch code is not guaranteed. Spot-check against the reference implementation.
7. **Emotion control is promised more than it is measured.**
   - VoxCPM2 does not explicitly document whisper, shout or fear, and its own card says style results vary between runs.
   - Qwen3-TTS has no instruct control on cloned voices.
   - Ming's emotion task can't be combined with other controls.
   - Nonverbals (sobs and screams especially) are the weakest point among permissive models. Plan to generate them separately (Dia, Maya1, Chatterbox Turbo) or to record or design them as SFX.
8. **Evaluation limits.** SER models are trained on acted or podcast speech. They can't tell fear from desperation, and they degrade on screams and whispers. ASR hallucinates on nonverbal-only clips. Treat the metrics as a screen; a human makes the final call.
9. **Determinism.** Sampling-based models give different takes. Fix and record seeds; MLX and MPS outputs differ.
10. **Store and regulatory disclosure** (not researched here). Check storefront AI-content disclosure rules and any applicable synthetic-media transparency rules before release.

---

## 8. Sources

### Leaderboards, comparisons, roundups
- https://artificialanalysis.ai/text-to-speech/leaderboard/provider-voice/open-weights
- https://www.yaps.ai/blog/best-local-tts-models
- https://thursdai.news/topics/open-source
- https://arxiv.org/pdf/2505.23009 (EmergentTTS-Eval)
- https://github.com/boson-ai/EmergentTTS-Eval-public (referenced; not read)
- https://huggingface.co/spaces/TTS-AGI/TTS-Arena-V2 (did not render)
- https://jp-tts-50ed7e.pages.dev/ (emotion sweep setup, no results)
- https://tinycomputers.io/posts/the-real-cost-of-running-qwen-tts-locally-three-machines-compared.html

### mlx-audio and Apple Silicon ports
- https://github.com/Blaizzy/mlx-audio
- https://pypi.org/project/mlx-audio/
- https://huggingface.co/mlx-community/VoxCPM2-8bit/blob/main/README.md (RTF figures via search result)
- https://soniqo.audio/guides/voxcpm2
- https://pypi.org/project/chatterbox-mlx/
- https://clawhub.ai/qwen3-tts-mlx
- https://dev.to/tumf/qwen3-tts-surprised-by-the-quality-of-japanese-on-apple-silicon-m3-creating-rights-free-voices-k1d
- https://andreisuslov.github.io/qwen-tts/platforms.html
- https://codersera.com/blog/install-and-run-orpheus-3b-tts-on-macos-a-complete-guide/
- https://arahim3.github.io/mlx-tune/audio.html
- https://pypi.org/project/mlx-vlm/

### Synthesis models
- Kokoro: https://huggingface.co/hexgrad/Kokoro-82M
- Chatterbox:
  - https://github.com/resemble-ai/chatterbox
  - https://huggingface.co/ResembleAI/chatterbox
  - https://huggingface.co/ResembleAI/chatterbox-turbo
  - https://raw.githubusercontent.com/resemble-ai/chatterbox/master/pyproject.toml
  - https://raw.githubusercontent.com/resemble-ai/chatterbox/master/example_for_mac.py
  - https://blog.fal.ai/chatterbox-turbo-is-now-available-on-fal/
  - https://github.com/resemble-ai/chatterbox/issues/492
- Orpheus: https://huggingface.co/canopylabs/orpheus-3b-0.1-ft, https://github.com/canopyai/Orpheus-TTS
- Dia: https://github.com/nari-labs/dia, https://github.com/nari-labs/dia2, https://huggingface.co/nari-labs/Dia2-2B
- Zonos: https://github.com/Zyphra/Zonos, https://huggingface.co/Zyphra/Zonos-v0.1-transformer
- Sesame CSM: https://huggingface.co/sesame/csm-1b, https://github.com/SesameAILabs/csm
- Parler-TTS: https://huggingface.co/parler-tts/parler-tts-large-v1
- StyleTTS2: https://github.com/yl4579/StyleTTS2
- CosyVoice: https://github.com/FunAudioLLM/CosyVoice, https://huggingface.co/FunAudioLLM/Fun-CosyVoice3-0.5B-2512
- Kyutai:
  - https://huggingface.co/kyutai/tts-1.6b-en_fr
  - https://huggingface.co/kyutai/tts-voices
  - https://github.com/kyutai-labs/delayed-streams-modeling
  - https://huggingface.co/kyutai/pocket-tts
- Higgs Audio:
  - https://huggingface.co/bosonai/higgs-audio-v2-generation-3B-base
  - https://huggingface.co/bosonai/higgs-audio-v2-generation-3B-base/blob/main/LICENSE
  - https://huggingface.co/bosonai/higgs-tts-3-4b
- IndexTTS: https://github.com/index-tts/index-tts
- F5-TTS: https://huggingface.co/SWivid/F5-TTS
- XTTS-v2: https://huggingface.co/coqui/XTTS-v2
- Bark: https://github.com/suno-ai/bark
- Spark-TTS: https://github.com/SparkAudio/Spark-TTS, https://huggingface.co/SparkAudio/Spark-TTS-0.5B
- MeloTTS: https://github.com/myshell-ai/MeloTTS, https://huggingface.co/myshell-ai/MeloTTS-English
- Qwen3-TTS: https://github.com/QwenLM/Qwen3-TTS, https://huggingface.co/Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign, https://arxiv.org/abs/2601.15621
- VoxCPM2: https://huggingface.co/openbmb/VoxCPM2, https://github.com/OpenBMB/VoxCPM
- Ming-3.0:
  - https://huggingface.co/inclusionAI/Ming-3.0-tts-0.5B
  - https://github.com/inclusionAI/Ming-3.0-tts
  - https://aireiter.com/blog/ming-3-0-tts-review-local-deployment
- Maya1: https://huggingface.co/maya-research/maya1, https://www.marktechpost.com/2025/11/11/maya1-a-new-open-source-3b-voice-model-for-expressive-text-to-speech-on-a-single-gpu/
- Step-Audio-EditX: https://github.com/stepfun-ai/Step-Audio-EditX, https://huggingface.co/stepfun-ai/Step-Audio-EditX
- Fish Drama 3:
  - https://huggingface.co/fishaudio/drama-3
  - https://huggingface.co/fishaudio/drama-3/blob/main/LICENSE
  - https://www.marktechpost.com/2026/10/14/fish-audio-drama-3-open-weight-tts-model/
- Voxtral TTS: https://huggingface.co/mistralai/Voxtral-4B-TTS-2603
- OmniVoice: https://huggingface.co/k2-fsa/OmniVoice
- AuK:
  - https://huggingface.co/tencent/AuK
  - https://comfyui-wiki.com/en/models/auk/auk
  - https://huggingface.co/Qwen/Qwen2.5-Omni-3B
  - https://huggingface.co/Qwen/Qwen2.5-Omni-3B/blob/main/LICENSE
- VibeVoice: https://huggingface.co/microsoft/VibeVoice-1.5B
- NeuTTS Air: https://huggingface.co/neuphonic/neutts-air
- Magpie: https://huggingface.co/nvidia/magpie_tts_multilingual_357m
- Phonon: https://www.marktechpost.com/2026/10/22/gradium-phonon-31m-on-device-tts/, https://gradium.ai/blog/gradium-phonon
- MOSS-TTS (secondary): https://www.llmreference.com/model/moss-tts-1-5, https://huggingface.co/OpenMOSS-Team/MOSS-TTS-Local-Transformer

### Evaluation tools
- https://pypi.org/project/mlx-whisper/
- https://github.com/ggml-org/whisper.cpp
- https://github.com/SYSTRAN/faster-whisper
- https://github.com/senstella/parakeet-mlx
- https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3
- https://github.com/jitsi/jiwer
- https://huggingface.co/emotion2vec/emotion2vec_plus_large
- https://github.com/ddlBoJack/emotion2vec
- https://github.com/modelscope/FunASR/blob/main/MODEL_LICENSE
- https://huggingface.co/3loi/SER-Odyssey-Baseline-WavLM-Categorical-Attributes
- https://huggingface.co/3loi/SER-Odyssey-Baseline-WavLM-Multi-Attributes
- https://huggingface.co/firdhokk/speech-emotion-recognition-with-openai-whisper-large-v3
- https://huggingface.co/superb/wav2vec2-base-superb-er
- https://huggingface.co/audeering/wav2vec2-large-robust-12-ft-emotion-msp-dim
- https://huggingface.co/MIT/ast-finetuned-audioset-10-10-0.4593 (and `/raw/main/config.json`)
- https://huggingface.co/laion/larger_clap_general
- https://huggingface.co/speechbrain/spkrec-ecapa-voxceleb
- https://github.com/sarulab-speech/UTMOSv2
- https://huggingface.co/Qwen/Qwen2-Audio-7B-Instruct
- https://huggingface.co/Qwen/Qwen3-Omni-30B-A3B-Captioner
- https://github.com/YannickJadoul/Parselmouth
- https://pypi.org/project/librosa/
- https://github.com/csteinmetz1/pyloudnorm
- https://github.com/maxrmorrison/torchcrepe

### Environment
- https://pypi.org/project/torch/
- https://dev-discuss.pytorch.org/t/pytorch-2-15-0-general-availability/3470
- https://pypi.org/project/mlx/
- https://pypi.org/project/uv/

---

## Addendum: the bake-off (2026-11-09)

Run in this repository after the note above was written (`docs/design/VOICES.md`,
*The bake-off*): the first persona's ten lines, four takes each, from
VoxCPM2, Qwen3-TTS, Chatterbox and Ming-3.0 on MLX (mlx-audio 0.6.4),
judged by the evaluation stack of section 5. Chatterbox passed all ten
lines (32 of 40 takes), VoxCPM2 eight, Ming-3.0 seven, Qwen3-TTS two. The
recommendation of section 1 (VoxCPM2 first) did not hold for these lines:
Chatterbox's emotion comes from its reference clip, and with a terrified
reference made by VoxCPM2's controllable cloning (section 1's step 3) it
delivered the screams VoxCPM2 itself could not keep in his voice. Two
other findings: the speech emotion recognisers of section 5b hear
frightened shouting as anger, so arousal, not the classes, tells a calm
read from a terrified one; and the WavLM heads' own code needs an older
transformers and downloads a third model, so they are rebuilt from their
checkpoints.
