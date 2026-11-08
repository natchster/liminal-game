# Voices: the voice-over pipeline

The game's voices are made by formant synthesis: the cry you give when you
call for help (`sfx_cry`, the `H` key: "a male voice shouting HEEEELP", a
sawtooth through two band filters) and the child below you in the Stairwell
(a table of phonemes, `audio::soundscape::PHRASES`). Both read as machines.
This file is the plan for real voice-over: spoken lines made by
text-to-speech models that run on this machine, licensed for a game that may
be sold, each voice a **persona** kept the same across all its lines, and
every line checked by machine for its words, its tone of voice and its
emotion before a person listens.

## The design

1. **Personas** (`assets/src/voice/personas/<id>.toml`): who speaks. The
   person (age, build, where they are from, what they are doing here), the
   voice (a description a voice-design model turns into a reference clip;
   made by us, never a real person's, never "sounds like" anyone), the tone
   of voice (a few words a director would give an actor, and what the voice
   must never sound like), the emotions it may take and their intensity, and
   the measurable targets the evaluation holds it to (pitch, its range, rate,
   loudness, how near its own reference it stays).
2. **Lines** (`assets/src/voice/lines/<persona>.toml`): what is said. Each
   line has an id, its text (nonverbals as tags: a gasp, a sob), its emotion
   and intensity, where it is used in the game, and how many takes to make.
3. **The models** (`docs/research/voice-tts.md`): a persona's voice is
   *designed* once from its description (Qwen3-TTS VoiceDesign, or VoxCPM2's
   own design mode) and kept as its reference clip; each line is then
   *cloned* from that clip with the line's emotion as an instruction
   (VoxCPM2's controllable cloning), or with an intensity dial (Chatterbox's
   `exaggeration`), or as a fear-labelled clone (Ming-3.0). Which of them
   makes a persona's lines is decided by a bake-off through the evaluation
   (V5), not by the vendors' tables: no independent test of their emotion
   exists. Only models whose code **and** weights allow selling what they
   make are used (Apache-2.0 or MIT; not the non-commercial ones, not those
   bound to Llama's terms, not one that speaks a disclaimer into every file).
4. **The pipeline** (`cargo xtask voice build`): for each line, its takes
   from the persona's model (a pinned model revision, a fixed seed a take, so
   a take is made again the same), cleaned (trimmed, its loudness set to the
   persona's level, its peak under −1 dBTP), and the chosen take a line
   written as WAV into `crates/game/assets/voice/` and listed in
   `MANIFEST.json` (the model and its licence, the persona, the text, the
   seed, the fingerprint). The models run in pinned Python environments
   (`tools/voice/`, `uv`, MLX on the GPU), their weights in the user's cache,
   never in the repository; each model's licence and card are kept at the
   revision used.
5. **The evaluation** (`cargo xtask voice eval`): every take checked for
   - its **words**: speech recognition (Whisper large-v3-turbo on MLX, and
     Parakeet as a second opinion) against the text, the word error rate
     after both are normalized; a take the two hear differently is flagged;
     a line that is only a nonverbal is not transcribed;
   - its **emotion**: speech emotion recognition (emotion2vec+, the WavLM
     categories and its arousal, valence and dominance) against the line's
     emotion and intensity; no model knows desperation or exhaustion, so
     those are read from fear and sadness together, arousal, pitch and
     loudness;
   - its **tone of voice**: pitch, its range, rate, loudness, breathiness
     and pauses against the persona's targets; a text-audio model (CLAP)
     asked whether the take is nearer the persona's tone than what it must
     never sound like; the AudioSet classifier for the nonverbals and the
     style (a scream, a gasp, a sob, a whisper, a shout);
   - its **voice**: speaker similarity to the persona's reference (one
     person, not several).

   A report per persona (`target/voice/<persona>/report.md`), the best take
   a line chosen by it, and a gate that fails a persona whose chosen takes
   miss. The machine screens; a person makes the last choice by ear.
6. **In the game**: the engine plays a sample (a WAV decoded into a
   buffer); a voice line is chosen by its context (the cry by how far down
   you are and how often you have cried), from a stream of its own, never
   the per-frame stream (`CLAUDE.md`, invariant 1). The formant voices stay
   in the code as a fallback until each persona replaces them.
7. **Skills** (`.claude/skills/`): `voice-persona` (make a persona and its
   reference), `voice-lines` (write and make its lines), `voice-eval` (check
   them and choose the takes), each ending in its gates and evidence, as the
   content skills do.

## The toolchain

```sh
cargo xtask voice doctor [--smoke]        # uv, both environments, every model at its revision, the licences
cargo xtask voice fetch [--only ID,…]     # the models into the Hugging Face cache (once; ~25 GB)
uv sync --project tools/voice/synth       # the environments (once; uv: `brew install uv`)
uv sync --project tools/voice/eval
```

- **`tools/voice/models.toml`** pins every model to a full revision and says
  which files it needs. A synthesis model (`role = "synth"`) makes what the
  game ships: its code and weights must be Apache-2.0 or MIT, its upstream
  card and licence are kept at the revision used in
  `assets/src/voice/models/<id>/`, and its upstream is in `MANIFEST.json`'s
  `allowed_models` (the doctor checks all three). An evaluation model
  (`role = "eval"`) never ships; its licence is recorded, not gated
  (emotion2vec+'s says nothing of commercial use). The MLX conversions'
  tags can differ from the original's (`chatterbox-fp16` says Apache-2.0;
  Resemble AI's weights are MIT): the upstream governs.
- **Two environments**, each a `uv` project locked in `uv.lock`:
  `tools/voice/synth` (Python 3.12, MLX and mlx-audio: VoxCPM2, Qwen3-TTS,
  Chatterbox, Ming-3.0 on the GPU) and `tools/voice/eval` (Python 3.13,
  torch: the recognisers, the emotion models, CLAP, AudioSet, ECAPA, Praat,
  the loudness meter). They never share a process, so their pins never
  meet. `common.py` is what both read: the pins, where a model is in the
  cache, a take as samples.
- **Nothing reaches the network once fetched**: the scripts set
  `HF_HUB_OFFLINE`, and a model that loads a second repository as it starts
  (Chatterbox loads CosyVoice 2's speech tokenizer) has that one pinned too.
- **A download that stops** at the same byte every time (it happened to
  `chatterbox-fp16` at 292,905,779 bytes, twice) is Hugging Face's chunked
  transfer (xet) failing on that file: fetch it again with
  `HF_HUB_DISABLE_XET=1` (plain HTTP, slower, resumes).
- **A take is made again the same**: the same seed gives the same samples
  on the same machine and versions, and the WAVs are written with headers of
  our own (libsndfile stamps the time into a float WAV).

## The pipeline

```sh
cargo xtask voice reference --persona you --candidates 6   # voices from the description, each heard back
cargo xtask voice reference --persona you --write          # keep voice.design_seed's as personas/you.wav
cargo xtask voice build --persona you [--model M] [--only LINE,…]   # every take, raw then cleaned
cargo xtask voice write --persona you [--model M] [--take K]        # the chosen takes into the game
```

1. **The reference**: candidates designed from the persona's description
   (its design model, seeds `design_seed` on), each heard back by both
   recognisers. One is chosen (`voice.design_seed`) and kept, cleaned to
   −20 LUFS, as `assets/src/voice/personas/<id>.wav`: every line is cloned
   from it, so it is kept with the sources, not made again. AudioSet's
   *Speech synthesizer* label tells a natural candidate from a machine-like
   one (of the first persona's six, seed 1 scored 0.77, seed 4 0.03).
2. **The takes**: each line's `takes`, seed = the take's number, from the
   line model (or `--model`): VoxCPM2 clones the reference and takes the
   emotion's words as its instruction; Qwen3-TTS designs the voice again
   with the words added; Chatterbox clones it with exaggeration from the
   intensity; Ming-3.0 clones it with its emotion label. Raw takes go to
   `target/voice/<persona>/<model>/raw/<line>/take<k>.wav` with a JSON of how
   each was made; the cleaned ones beside them.
3. **The cleaning**: trimmed (60 ms before the first sound, 180 ms after the
   last, short fades), 48 kHz, set to the persona's loudness with its true
   peak under the ceiling by lowering the gain (never a limiter: a scream
   keeps its shape and ends up quieter; the first persona's takes sit at a
   median −17.9 LUFS for a −16 target), 16-bit. A take the model left
   nearly silent is never raised more than 24 dB and is marked silent (one
   of the first persona's forty).
4. **Into the game** (`voice write`): each line's chosen take as
   `crates/game/assets/voice/<persona>/<line>.wav`, its `MANIFEST.json`
   entry written (the models that said it and designed its voice, both in
   `allowed_models`; its text, emotion and seed; its sources; the
   fingerprint). `tests/structure.rs` fails on a voice file whose
   fingerprint, licence, generator or models are not right.

Built twice, the first persona's forty takes are the same files, byte for
byte, raw and cleaned.

## The evaluation

```sh
cargo xtask voice eval --persona you [--model M]   # every take judged; target/voice/you/M/report.md; the gate
cargo xtask voice selftest                         # the evaluation against known answers
```

Each take (cleaned, as it would ship) is judged on six checks
(`tools/voice/judge.py`); a line's best take is the best-scoring one that
passes all six, and the gate fails a persona with a line that has none.

| Check | How | Passes when |
|---|---|---|
| words | Whisper large-v3-turbo and Parakeet TDT 0.6B, both normalized (Whisper's English normalizer), word error rate | Whisper's error ≤ `wer_max` (Parakeet's beside it, for a take the two hear differently) |
| emotion | WavLM's arousal and valence (3loi's MSP-Podcast baseline, rebuilt here without its remote code); emotion2vec+ and WavLM's classes | arousal within ±0.25 of what the emotion asks at its intensity (two-sided: an exhausted line is not shouted) and valence ≤ 0.55; the classes only rank takes |
| pitch | Praat's median F0 | within `f0_median_hz` (a man's 600 Hz "scream" is a falsetto shriek) |
| tone | CLAP: the take against the persona's `like` and `never` phrases | nearer the nearest `like` than the nearest `never` |
| voice | ECAPA speaker similarity to the reference | ≥ `similarity_min` up to intensity 0.4, falling to `similarity_min_loud` at 1 |
| not synthetic | AudioSet's *Speech synthesizer* | below 0.5 |

What the known answers taught (`voice selftest`, all seven right):

- **The emotion recognisers hear frightened shouting as anger**, disgust or
  surprise far more often than fear (emotion2vec+ called VoxCPM2's shouts
  *angry* at 0.88–0.96), and the two families disagree (WavLM heard fear
  where emotion2vec+ heard neutral). The classes cannot gate; **arousal
  can**: a calm read of a terrified line is 0.33–0.37 where 0.80–0.86 is
  wanted, the good loud takes 0.75–0.91.
- **Speaker similarity works on calm lines and not on shouts**: another man
  reads 0.33 and a woman 0.04 against his own calm takes' 0.54–0.75, but
  his shouts read 0.13–0.34, where another man's shout (0.31) also falls. On
  a loud line the check only catches a voice wholly unlike his; whether a
  scream is still the same man is for an ear.
- **CLAP hears every one of VoxCPM2's shouts as nearer "a cartoon
  character"** than "a frightened man shouting for help"; the calm lines it
  places the right way. Kept as a screen: it is the complaint people make of
  synthetic screams.

The first persona on VoxCPM2: seven of ten lines have a take that passes;
the two loudest shouts (`help_me`, `help_scream`) and the most desperate
line (`i_cant`, read too calmly) have none. That is where the bake-off
(V5) starts.

## The research

[`docs/research/voice-tts.md`](research/voice-tts.md) (2026-11-08): the
models that run here and their licences (code and weights), how each
controls emotion and keeps a voice, the evaluation tools, the Python
environments and the risks (licences drift: keep the licence at the revision
used; a designed voice may resemble someone: keep descriptions generic;
emotion control is promised more than it is measured).

## Milestones

| # | Milestone | Done when |
|---|---|---|
| V0 | This plan; the research (`docs/research/voice-tts.md`) | committed |
| V1 | **The toolchain**: `tools/voice/` (a `uv` project for synthesis on MLX, one for the evaluation, each locked); the models fetched to the cache at pinned revisions, their licences and cards kept; the synthesis models in `MANIFEST.json`'s `allowed_models`; `cargo xtask voice doctor` | `voice doctor` passes; one sentence synthesized and transcribed back |
| V2 | **Personas and lines**: the formats, a test that reads every persona and line file and fails on a missing or unknown field; the first persona's profile and its lines drafted | the test; the profile below |
| V3 | **The pipeline**: `cargo xtask voice build [--persona …] [--write]`: the reference clip, the takes, the cleaning, the chosen takes into the assets and the manifest | a persona's lines built twice give the same files; `MANIFEST.json` lists them; the structure test covers them |
| V4 | **The evaluation**: `cargo xtask voice eval` (words, emotion, tone of voice, voice; the report; the best takes; the gate) | a known-bad take (wrong words; a calm read of a terrified line; another voice) fails it, a good one passes |
| V5 | **The bake-off**: the first persona's lines from VoxCPM2, Qwen3-TTS, Chatterbox and Ming-3.0 through the evaluation; the model chosen and why, here and in the research | the scores per model; the choice |
| V6 | **In the game**: a sample played by the engine; the cry chosen by context, its draws its own | `tests/listen.rs` hears it (and fails with the line at gain 0); the loudness gate; no golden or reference level moves |
| V7 | **Skills and the guide**: `voice-persona`, `voice-lines`, `voice-eval`; `CLAUDE.md` | the skills followed end to end by a fresh agent |
| V8 | **The first persona**: the one who cries for help (you), profiled here and in `LORE.md`, their lines made, checked and in the game in place of `sfx_cry` | the evaluation passes; a listen render; a person listens (`PLAYTEST.md`) |
| V9 | **Evidence and the ledger**: `QUALITY.md`, `content.json`, `ARCHITECTURE.md`; the full check; the bundle | full check passes |

### Progress

| # | State |
|---|---|
| V0 | done: `4810c24` |
| V1–V2 | done: `voice doctor --smoke` passes (13 models at their revisions, the five synthesis licences kept and allowed, a sentence said by Qwen3-TTS and heard back by both recognisers, word error 0); the persona and line formats and their test; the first persona drafted |
| V3 | done: the reference (six candidates, seed 4 kept), forty takes from VoxCPM2 made twice the same, the cleaning, `voice write` and the manifest test on voice files |
| V4 | done: the six checks, `voice eval` (report and gate) and `voice selftest` (seven known answers, all right) |

## Personas

A persona is two files: `assets/src/voice/personas/<id>.toml` (who, the
voice's description, the tone, the emotions with the words a model is
directed with, the targets) and `assets/src/voice/lines/<id>.toml` (each
line: its text, emotion, intensity 0–1, where the game plays it, on which
levels, how many takes). `cargo xtask check` reads both
(`voice::tests::every_persona_and_line_file_is_complete`): a missing field,
a field of the wrong kind, an unknown field (a misspelling), an emotion the
persona does not have, a model that is not a synthesis model, a use the game
does not play, all fail it.

### You: the one who cries for help

*The player's own voice, heard only when you cry out (`H`). Never a word
besides, never a narrator.*

| | |
|---|---|
| Who | An ordinary man of about thirty-six who went through the wall at the back of a storage room at the end of a night shift and came out in the Building. Not brave, not a talker; he has never shouted for help in his life before. The further down he goes, the less he expects an answer, and the more he cries out anyway. |
| From | Somewhere in the American Midwest: plain speech, no strong accent. |
| Never said | His name, his work, his face, how long he has been here. The game never says who you are; this profile is for casting the voice, not for the player. |
| Voice | An untrained baritone, slightly rough and tired, General American; recorded close and dry. A stockroom worker's voice, not an announcer's or an actor's. Designed from that description (Qwen3-TTS VoiceDesign), never cloned from a person. |
| Tone | A real man shouting into a huge empty building, not performing. The breath before the shout; the voice cracks when it is high. Never: a narrator, a radio announcer, a cartoon, a calm man talking, a stage actor declaiming. |
| Emotions | unease (calling out, half expecting nothing), fear (breathless, shouting), terror (screaming, voice cracking), desperation (pleading, close to tears), exhaustion (hoarse, worn out from shouting). |
| Lines | Ten cries (`lines/you.toml`): on the Building's floors he calls out ("Hello? Is anybody there?", "Can anyone hear me? Hello!"); in the rooms he shouts for help ("Help! Help me!", "Get me out of here! Please!"); deep down he is worn out ("Please. Somebody. Help.", "Help me. Please. I can't do this."); a scream ("Help!") when he cries again soon after the last. |
| Targets | Median pitch 85–330 Hz (a scream goes high); −16 LUFS, true peak under −1 dBTP; words wrong at most 15%; speaker similarity to his reference at least 0.45 on calm lines, 0.1 on a scream. |
| Reference | `personas/you.wav`: Qwen3-TTS VoiceDesign, seed 4 of six (106 Hz, the roughest, AudioSet's speech-synthesizer 0.03). |

Next after the first: the child below you in the Stairwell ("please",
"help", "down here", the sob and the wail).

## What only people can tell

- Does a line sound like a person in that place, or like a narrator reading?
- Is the cry for help frightening, or embarrassing, after the tenth time?
- Do the takes of one persona sound like one person?
