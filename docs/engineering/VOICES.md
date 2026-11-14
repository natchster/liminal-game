# The voices: the pipeline

How the voices are made, judged and put into the game: the toolchain, the
pipeline, the evaluation and the files the game embeds. What the voices are
and why is [`../design/VOICES.md`](../design/VOICES.md); the work on them is
[`../plan/VOICES.md`](../plan/VOICES.md).

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

## In the game (V6)

- **The files**: `crates/game/assets/voice/<persona>/<line>.wav` (16-bit,
  48 kHz, mono; 2.0 MB for the first persona's ten lines) and the index
  `crates/game/assets/voice/lines.tsv` (each line's use, levels,
  intensity, emotion, text), both written by `cargo xtask voice write`.
  `audio::voice::WAVS` embeds them (one `wav!` line each, like the
  photographed textures); a test holds the list and the index together and
  decodes every file, and `voice::tests::the_games_index_of_lines_matches_the_line_files`
  holds the index to the line files.
- **The cry** (`Game::cry_for_help`, `H`): `say_cry` chooses a line
  (`audio::voice::choose_cry`): one for the level (the Building's floors
  call out, the rooms shout for help, deep down he is worn out), a louder
  one from the second cry on a floor, never the one said last, and the
  scream when you cry again within 45 s of the last. The choice comes from
  a stream of its own (the run's seed, the level, the cry's number in the
  run): a cry draws nothing from the per-frame stream, so no frame after it
  changes. The line is played at its own level (`VOICE_GAIN` 1.0: its WAV
  is at −16 LUFS, true peak −1 dBTP) into the soundscape's `world` bus:
  centred, your own voice, but in the reverb of the place you stand. The
  formant cry (`sfx_cry`) stays as the fallback when no line can be said.
- **Heard**: `tests/listen.rs`'s `the_cry_is_your_voice` (a cry on level
  5: 25.9 dB over the room in the voice's band, peaks at −2.1 dBFS; with
  `VOICE_GAIN` at 0 it fails at 5.1 dB). The formant cry it replaced stood
  36 dB over the room and hit 0 dBFS: it clipped.
