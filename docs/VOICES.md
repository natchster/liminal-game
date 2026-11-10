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

## The bake-off (V5)

The first persona's ten lines, four takes each, from each of the four
models, through the evaluation (`cargo xtask voice build|eval --model M`):

| Model | Lines with a passing take | Takes passing | Mean voice similarity | Checks failed |
|---|---|---|---|---|
| **Chatterbox** | **10/10** | **32/40** | 0.59 | emotion 6, tone 2, words 1 |
| VoxCPM2 | 8/10 | 17/40 | 0.44 | emotion 14, tone 7, voice 6, pitch 5, words 3 |
| Ming-3.0 | 7/10 | 20/40 | 0.56 | emotion 14, tone 9, words 4, pitch 3, voice 3 |
| Qwen3-TTS (voice designed each line) | 2/10 | 3/40 | 0.19 | voice 28, tone 19, emotion 13 |

**Chatterbox makes his lines** (`line_model`). What the bake-off found:

- **A cloning model takes its emotion from the clip it clones.** From his
  calm reference, Chatterbox's terrified lines reached arousal 0.36–0.46 of
  the 0.85 asked, at the top of its `exaggeration` dial; VoxCPM2 alone,
  directed in words, reached 0.91. So his terror has a reference of its own
  (`[references.terror]`: VoxCPM2 clones his calm voice saying a few lines
  of terror; `cargo xtask voice reference --persona you --emotion terror`),
  and Chatterbox clones the terrified lines from it: 0.81–0.96. Ming and
  VoxCPM2 clone from it too.
- **VoxCPM2 now and then says nothing** (two silent takes of forty) and its
  shouts go falsetto (590–650 Hz); CLAP hears all its shouts as nearer "a
  cartoon character" than a frightened man.
- **Qwen3-TTS cannot keep a voice** when each line's emotion has to be
  written into its description (similarity 0.19); it designs the persona's
  reference well and nothing more here.
- **A one-word scream is said twice** ("Help! Help!"); the line says so.
- No model passes the desperate line `i_cant` but Chatterbox, nor
  `help_me` but Chatterbox and Ming: the hardest lines decide.

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
| V5 | done: the bake-off (four models, forty takes each); Chatterbox chosen; his terror cloned from a reference of its own |
| V6, V8 | done: the ten lines in the game (Chatterbox, the evaluation's chosen takes), `say_cry` in place of `sfx_cry` (its fallback), the cry heard 25.9 dB over the room without clipping; the profile here and in `LORE.md`; the playtest's questions |
| V7 | done: the skills `voice-persona`, `voice-lines`, `voice-eval` and `CLAUDE.md`'s commands; followed end to end by a fresh agent, whose findings were fixed in the skills and the code (`voice bakeoff`; references judged; `--take id=K`; only models that speak; the cry keeps to your lines; the report shows Parakeet, valence, the nearest *never*, silence) |
| V9 | done: `QUALITY.md` (Voices, C), `content.json` (`sound/voice`), `ARCHITECTURE.md`; the full check passes but for the bench's `7 L5` build (625–688 ms over 600 with and without these changes: older than them); `dist/Backrooms.app` built with the voice. Waiting on a person's ears (`PLAYTEST.md`) |

## Personas

A persona is two files: `assets/src/voice/personas/<id>.toml` (who, the
voice's description, the tone, the emotions with the words a model is
directed with, the targets) and `assets/src/voice/lines/<id>.toml` (each
line: its text, emotion, intensity 0–1, where the game plays it, on which
levels, how many takes; and, where its use has several, its `subject`:
what it is said about, a place or a kind of thing, whose lines the game
chooses before the use's general ones). A persona speaks with a voice
designed from its description (`voice.description`, `reference_text`,
`design_model`, `design_seed`), or with another persona's
(`voice.same_as`: every line cloned from that one's reference, so one man
can have two registers with their own tone, emotions and loudness), never
both. `cargo xtask check` reads both
(`voice::tests::every_persona_and_line_file_is_complete`): a missing field,
a field of the wrong kind, an unknown field (a misspelling), an emotion the
persona does not have, a model that is not a synthesis model, a use the game
does not play, all fail it.

### You: the one who cries for help

*The player's own voice when you cry out (`H`); besides it only his
asides, the same man under his breath (`you_aside`, *Your asides*). Never
a narrator.*

| | |
|---|---|
| Who | An ordinary man of about thirty-six who went through the wall at the back of a storage room at the end of a night shift and came out in the Building. Not brave, not a talker; he has never shouted for help in his life before. The further down he goes, the less he expects an answer, and the more he cries out anyway. |
| From | Somewhere in the American Midwest: plain speech, no strong accent. |
| Never said | His name, his work, his face, how long he has been here. The game never says who you are; this profile is for casting the voice, not for the player. |
| Voice | An untrained baritone, slightly rough and tired, General American; recorded close and dry. A stockroom worker's voice, not an announcer's or an actor's. Designed from that description (Qwen3-TTS VoiceDesign), never cloned from a person; his lines said by Chatterbox cloning it, his terror cloned from a terrified clip of the same voice (VoxCPM2). |
| Tone | A real man shouting into a huge empty building, not performing. The breath before the shout; the voice cracks when it is high. Never: a narrator, a radio announcer, a cartoon, a calm man talking, a stage actor declaiming. |
| Emotions | unease (calling out, half expecting nothing), fear (breathless, shouting), terror (screaming, voice cracking), desperation (pleading, close to tears), exhaustion (hoarse, worn out from shouting). |
| Lines | Ten cries (`lines/you.toml`): on the Building's floors he calls out ("Hello? Is anybody there?", "Can anyone hear me? Hello!"); in the rooms he shouts for help ("Help! Help me!", "Get me out of here! Please!"); deep down he is worn out ("Please. Somebody. Help.", "Help me. Please. I can't do this."); a scream ("Help! Help!") when he cries again soon after the last. |
| Targets | Median pitch 85–400 Hz (his scream goes to 300–400 Hz; a falsetto shriek is not him); −16 LUFS, true peak under −1 dBTP; words wrong at most 15%; speaker similarity to his reference at least 0.45 on calm lines, 0.1 on a scream. |
| Reference | `personas/you.wav`: Qwen3-TTS VoiceDesign, seed 4 of six (106 Hz, the roughest, AudioSet's speech-synthesizer 0.03). |

Next after the first: the child below you in the Stairwell ("please",
"help", "down here", the sob and the wail).

## Your asides: the voice director

*What you say to yourself.* The same man who cries for help, under his
breath: a few words on arriving somewhere, at a sudden noise, at what
comes for him, at what the place does back. Never a narrator, never a
guide: a person alone reacting, the way people talk to themselves when
they are frightened. Few, and different every run. The research behind
it is [`research/inner-voice.md`](research/inner-voice.md) (SOURCES 16):
Valve's dynamic dialog (an event matched to the most specific line, the
memory of what was said read by the next choice, nothing as an answer)
and bark managers in practice (priorities, dropping the stale, fatigue).

### The persona

`you_aside` (`personas/you_aside.toml`): **his voice** (`voice.same_as =
"you"`: every line cloned from `personas/you.wav`, never a voice of its
own), in another register: close and low, under the breath, at
−20 LUFS where the cry is at −16, and played lower still
(`ASIDE_GAIN`). Its tone is a man muttering to himself in a huge empty
place (never a narrator, an announcer, an actor, a man shouting); its
emotions are its own: *wary* (low, uncertain), *startled* (a sharp breath
and a few quick words), *dread* (hushed, almost a whisper), *shaken*
(breathless after a close call), *pain* (a grunt, then words), *resolve*
(tired, talking himself on). Its lines are said by Chatterbox, which
made his calm lines best in the bake-off (V5).

**Its evaluation** (A2, `voice selftest`): the six emotions in
`judge.py`, their arousal near a calm read's (0.33–0.37 on his voice),
startled and pain higher, so the gate's two sides hold an aside under the
breath. The known answers, judged as `you_aside`:

| Take | Must | Measured |
|---|---|---|
| `aside_shouted`: a hushed line ("Don't. Please, don't.") cloned from his terror, told to scream | fail *emotion* | arousal 1.05 for 0.32 wanted (and pitch 351 Hz, voice 0.29) |
| `aside_other_man`: another man (Qwen3-TTS, a description of an old Scotsman) muttering | fail *voice* | similarity 0.44 under 0.45: by a hair, as V4 found another man calm at 0.33 |
| `aside_good_wary`, `aside_good_dread` (VoxCPM2), `aside_good_line_model` (Chatterbox): his own | pass all six | words 0.00; arousal 0.49, 0.25, 0.35; voice 0.50, 0.58, 0.54; tone +0.18, +0.13, +0.06 |

What the known answers taught: **a cloning model cannot shout from a
calm clip** (told to scream from his calm reference, VoxCPM2 read the
hushed line at arousal 0.37: the shout had to come from his terror, as V5
found for his cries); and **CLAP cannot screen this register as it
screens the cry's**: on clips this short and quiet it hears every take
nearest "a cartoon character", his own good ones included (0.09–0.22,
above any phrase of ours), so that complaint of synthetic screams is not
one of the asides' *never*; the mutter phrases are the ones it answers to
(his good takes 0.09–0.16; "whispering" −0.11 to −0.20). It cannot tell a
narrator's read from a mutter here (no phrase, nor AudioSet's
*Narration*, parts them), and neither can anything else we run: whether
an aside sounds like a man talking to himself or like a narrator is for an
ear (`PLAYTEST.md`).

### The rules

Each rule ends with its test; the code is `crates/game/src/audio/asides.rs`.

1. **React, never explain.** A line is what a person alone says to
   himself: a breath, a word, a question. Never a creature's name or what
   it does, a rule, a place's name or number, the way, an instruction to
   the player (PILLARS 4: rules are learned, not told; LORE: no
   narrator). *Test:* `lore-keeper` on every line;
   `no_aside_names_what_the_canon_keeps_unsaid` (the creatures' names,
   level, exit, digits, the people's names).
2. **Once is a reaction, twice is a habit.** An event is weighed once
   for its *scope*: a subject (a kind of sound, of creature, of thing)
   once a run, a floor's arrival or your nerve once a floor, and what
   happens again and again (a blow, the relief, the lamp) every time but
   not again within its cooldown. Its chance falls with each line already
   said about the same subject (by half; an arrival's by 0.7) and a
   little with each line of its use (0.85), so a new place late in a run
   still has its word. A line is never said twice in a run; a use whose
   lines are spent is silent; an event with no line here is not weighed
   at all (nothing plays, nothing is spent). An answered reaction that
   could not be said (held, late, a better one waiting) is *owed*: the
   next event like it is answered again, without a new roll, three times
   at most; a key is done with only once its line is said or its roll
   fails (review A). (PILLARS 5: the same scare never repeats soon.)
   *Test:* the director's unit tests; `tests/asides.rs`.
3. **Different every run.** Which events get a word, which word, and how
   much you talk at all (the run's *temperament*, 0.65–1.0 of every
   chance) come from the run's seed on streams of their own (`SplitMix64`,
   its salt the use and the event's count mixed through SplitMix's
   finaliser, so runs a few bits apart share no rolls), never the
   per-frame stream (`CLAUDE.md` invariant 1); a subject's own lines and
   its use's general ones are dealt together by the run, so a kind is
   answered differently from run to run. What was said is kept with the
   run and saved as each line is said, so a retry or a Continue, even
   after leaving without a word to the game, never says a line again.
   *Test:* the same seed says the same; seeds differ; a save round-trip;
   a die and retry, and a Continue, in `tests/asides.rs`.
4. **Hold your breath.** The pacing director says when
   (`Game::holding_breath`): within 3 s of your cry; on a floor, while
   something hunts you, searches within 25 m (`ENGAGED_NEAR`), the
   creature sent in a build-up is within 20 m, or the place itself is
   hushed (`amb_hush` 0.3: something coming, a search near, a silent
   machine, the fog up); in the Stairwell or the Loop only your cry. Each
   use says what holds it (`Use::held_by`): a blow's grunt nothing; the
   siren and a knock at your cell, which the hush is about, only your
   cry. A line of yours playing when you begin to hold your breath is
   faded out (`cut_voice_line`). The release speaks: an *aftermath*
   waits for the relief after a close call (PILLARS 3: silence means
   something; 5: dread, then release). *Test:* the director's unit tests;
   the pacing harness (A6).
5. **One at a time, and room between.** Never two at once, nor over the
   cry; at least 20 s between asides (8 s after a lesser one, for a
   threat or what the place does back); a blow's grunt keeps no gap and
   cuts in over a line of yours. A reaction is said in its moment or
   owed: a startle 0.35–0.9 s after the sound and within 2 s; an arrival
   within 15 s of being weighed, which is once you have walked 2 m, its
   card and line have gone (9 s: a floor is never described twice at
   once), nothing else is waiting, and inside the floor's first minute
   played. The first arrival of a run is always answered, on the first
   floor where it can be said. A floor whose door leads to the Stairwell
   or the Loop takes the general lines, not the rooms': its line is
   theirs. A better one waiting replaces a lesser. The director's times
   run on its own clock, time played: a pause or a menu stops it.
   *Test:* the director's unit tests.
6. **Heard by nothing but you.** Under your breath: no creature hears
   it, it changes nothing in play, it draws nothing from the per-frame
   stream, and the screen shows nothing. *Test:* the pacing, arena and
   playthrough tests, the goldens and the reference levels do not move.
7. **Only for a person playing.** Nothing while you have neither moved,
   looked round nor touched a key for 90 s (long enough to sit sealed in
   a cell watching the door): the asides are few, and are not spent on
   an empty chair. Every subject the game posts is one it knows
   (`Use::subjects`, both ways held by a test). *Test:* the director's
   unit tests; the loudness scenarios (standing still for 120 s, no input)
   do not move.

### What it reacts to

| Use | When | Subjects | For example |
|---|---|---|---|
| `arrive` | a few seconds after your first steps on a floor | the Well, the Stair Halls, the Works, the first rooms, the rooms, deep down, each anomaly | "There's no bottom." |
| `startle` | a sudden sound near you, the first of its kind in a run: a level's signature within its carry (a bang, a scream, steps, a laugh, a moan), the stalker's steps far off | its kind | "What was that?" |
| `sighted` | the first sight of a kind of creature in a run, from where it does not have you (`entity_sighted`; never the Watcher: a word would say which sheet) | the kind (never named): hound, smiler, corner, reader, glass | "Something's crawling." |
| `knocked` | a touch throws you down and lets you go (`creature_contact`) | — | "Get up. Come on." |
| `aftermath` | the relief after a close call (`enter_relax` after a peak survived), said once your breath is let go | — | "Is it gone?" |
| `siren` | the self-assembly's siren (`world::assembly`), through the fog's hush | first (until a line for it is said or declined), again | "What's that siren for?" |
| `held` | sealed in a cell while it is up: the first knock of what knocks (`world::cell_knock`), the all-clear (posted sealed in, said as you may step out) | knock, clear | "Please, not the door." |
| `answered` | a resident's door answers your knock (`world::knock`, `knock_answer`), before you pick it up | hatch, note | "Someone's in there." |
| `uncanny` | the place does something back: a marked door knocks back in your rhythm, the figure across the void turns its back on your look (not when you come upon it); later the hole, a dead machine's flap, a lift going by, the candle's notes, the pane's tick, the Stairwell's crying, the Loop's door | knock_back, figure (and A8's, A9's) | "That was my knock." |
| `found` | the first time in a run you come within 3.5 m of her camp, on its storey (`update_camp_come`, once a floor) | camp | "Whose things are these?" |
| `nerve` | your nerve near its end | — | "Keep it together." |
| `dark` | the lamp's battery runs out | — | "No. Come on." |

### Milestones

| # | Milestone | Done when |
|---|---|---|
| A0 | This plan; the research (`research/inner-voice.md`, SOURCES 16); the queue | committed |
| A1 | **The pipeline**: a persona with another's voice (`voice.same_as`); a line's `subject`, a column of the game's index | `cargo test -p xtask voice`; the game's index tests; `you`'s files unchanged |
| A2 | **The persona** `you_aside`: its file, its emotions in the evaluation, a known answer for them (a shouted read of a hushed line fails *emotion*; a calm one passes) | `voice selftest` right on every answer; `cargo test -p xtask voice` |
| A3 | **The voice director** (`audio::asides`): the chooser (rules 2, 3, 5), holding the breath (4), only for a person playing (7), the memory kept with the run; your cry and your asides played by one player | its unit tests; no golden, reference level, loudness scenario or pacing log moves |
| A4 | **Arriving**: the `arrive` lines made, judged and in the game; the first steps on a floor | the evaluation's gate; `tests/listen.rs` hears an arrival (and fails at `ASIDE_GAIN` 0); `lore-keeper` |
| A5 | **A sudden noise**: the `startle` lines; the place's one-shots, the creature's steps far off, a bang, a knock back, each by how near it is | the gate; the director's tests; `lore-keeper` |
| review 1 | Code and design review of A1–A5; what it finds fixed | committed |
| A6 | **What comes**: `sighted`, `knocked`, `aftermath` | the gate; the pacing harness (nothing in a peak, the aftermath in the relax) |
| A7 | **The Building**: `siren`, `held`, `answered`, the figure across the void | the gate; the assembly and knock harnesses log them |
| A8 | **The place answers**: `uncanny` (the hole, a dead machine, a lift, a candle, the pane), `found` | the gate; their harnesses log them |
| A9 | **The anomalies and the mind**: the Stairwell, the Loop, the Showroom, the Borrowed House; `nerve`, `dark` | the gate; a harness each |
| A10 | **Evidence and the ledger**: the run analysis (asides a level, never a line twice, runs that differ, none in a peak); `LORE.md`, `PILLARS.md`, `QUALITY.md`, `content.json`, `ARCHITECTURE.md`, the skills; the full check; the bundle | full check passes |
| review 2 | Code and design review of A6–A10; what it finds fixed | committed |

### Progress

| # | State |
|---|---|
| A0 | done: this plan (`71338c7`) |
| A1 | done (`e7093bd`): `voice.same_as` (a persona with another's voice: its reference, its description; `voice reference` refuses to make one of its own) and a line's `subject`, a column of `lines.tsv` (`persona id use subject first last intensity emotion text`) read by `audio::voice::lines`; `you`'s ten lines rewritten by `voice write`, the same files byte for byte |
| A2 | done (`121c84f`): `personas/you_aside.toml` (his voice, `same_as = "you"`; −20 LUFS; six emotions), the emotions in `judge.py`, five known answers in `voice selftest` (all twelve right); `Persona::load` takes a persona whose lines are not written yet |
| A3 | done (`f9fc0e5`): the voice director (`audio::asides`): `Game::aside(use, subject)` weighs an event once a run (its key) by the run's seed (temperament, roll, the run's deal of the lines), the first floor's arrival always; the reaction waits for its moment and is said then or let go (`drop:…:late`), held while you hold your breath (`holding_breath`: your cry 3 s, a hunt, a search within 25 m, the stalker within 20 m) or away (30 s neither moved nor looked round), 20 s between (8 s for an urgent one after a lesser); the floor's arrival after 2 m of walking; the memory in the save (`asides`); your cry and your asides through `play_voice_line`. Fourteen unit tests; no line of it in the game yet (A4) |
| A4 | done (`a1369c1`): twenty arrival lines (`lines/you_aside.toml`: the Well, the Stair Halls, the Works, the Lobby, the rooms, deep down, the store, the house, and two general ones), rewritten after two `lore-keeper` rounds (no echo of a floor's own line, no premise stated, no advice, no subtitle repeated; no two floors in a row opening alike); Chatterbox, 62 of 80 takes passing (tone 10, words 5, emotion 3 failing), every line with a passing take; the chosen at voice 0.56–0.75, arousal 0.32–0.52, 94–119 Hz. An arrival waits for the card and the floor's line to go (9 s) and is weighed only in a floor's first minute; you keep quiet while the place is hushed (`HUSHED`, `amb_hush` 0.3); the Stairwell's and the Loop's floors take the general lines. `voice write` removes a line's file and manifest entry once the line is gone, and `tests/structure.rs` fails on a voice entry with no file. `an_arrival_is_your_voice_under_your_breath`: seed 4242's level 5 (the Stairwell's floor) says "Okay. Okay." 3.4 s after the first steps, 18.2 dB over the room, 14.9 dB over your steps, 7.4 dB under your cry, peak −5.1 dBFS; at `ASIDE_GAIN` 0 it fails (3.1 dB). `no_aside_names_what_the_canon_keeps_unsaid`; every line of the index one the game plays. The canon: `LORE.md` (*How canon reaches the player*: your asides; *Crying*), `PILLARS.md` 3 |
| A5 | done (`4970b52`): seven startle lines (two general, rooms only; a bang's, the Building's floors too; a scream's, steps', a laugh's, a moan's), rewritten after a `lore-keeper` round (to himself, never to anyone: a call is the cry's, and something answers it; never who made a sound or what it means; "Who's breathing?" cut, a breath is never a level's signature); 22 of 28 takes passing (words 5, voice 1), every line with one; the startled ones said calmer than asked (arousal 0.40–0.45 for 0.56–0.59, inside the gate: a clone of a calm clip, V5 again; an emotional reference for *startled* if an ear finds them flat). Posted by a level's signature heard within its carry (`startle_of`: a slam, a thud or a clank is a bang; a scream, steps, a laugh, a moan) and by the stalker's steps far off (`cue:steps`). An event with no line left for it here is not weighed (nothing plays, nothing is spent: a sound on the Building keeps its kind for the rooms). `tests/asides.rs`: three floors of the rooms, 150 s each, looking round: 4242's level 6 answers its moan 0.5 s after it ("That sound..."), 77's level 8 its steps; each kind once, the gap kept, no line twice, and the same run says the same |
| review A | done (`bf7a1df`): a code review and a design review of A1–A5 (fresh reviewers, the code and the docs only). Found and fixed: a held or dropped reaction used up its once-a-run key (the sight of a creature sent near you, a knock at your cell under the fog's hush, your nerve: the lines that matter most, silenced on purpose), now *owed*; uses with no subject rolled at every post, now a scope each (run, floor, every time past a cooldown); the breath worked out by the voice director from the creatures, now the pacing director's (`holding_breath`), and in the Stairwell and the Loop only your cry; the fog's hush held the uses that are about it; a line playing went on over a hunt, now cut off, and a blow's grunt kept the gap; decay by every line of a use starved a late new place, now by the subject's; the two general startle lines could never be said, now dealt with each kind's own; a Continue after leaving mid-floor said the same line again, now saved as each is said; the first floor's arrival could be lost to a waiting bang, and paused time ran its minute, now its own clock; neighbouring seeds shared rolls; a subject was free text, now a list held both ways; `tests/asides.rs` could not fail for saying too little, now asserts lines said, a retry and a Continue |
| A6 | done (`26a07fb`): ten lines of what comes, rewritten after a `lore-keeper` round (a Smiler's teeth cut: it is sighted almost only where none can be seen; an arm hurt cut: a touch takes nerve, not a limb; "That was close." cut: an all-clear, and a sign is not a promise; the cry's "help" and "I'm here" not echoed; no sight of the Watcher said at all: a word would say which sheet): `sighted` (two general, the Hound's, the one in the corner's, the Reader's), `knocked` (two), `aftermath` (three). 35 of 40 takes passing; "Ugh. Get up." and "My heart..." passed none (Whisper heard the grunt as "Ah", "Nah", "Go"; a trailing ellipsis cost a word), reworded to "Get up. Come on." and "My heart is pounding." (four of four each). Posted by `entity_sighted`, `creature_contact` (a touch that is not the end) and `enter_relax` after a peak survived. `tests/pacing.rs` keeps the asides now: four floors, 300 s each, live creatures, deaths on: arrivals and the first startles in the calm, the stalker's steps in the build-up while it is far, the grunt and the relief in the relief, a Reader's sight said in the relief after it was held by the hush, nothing chosen in a peak (asserted) and the relief only in the relief (asserted) |
| A7 | done (`9f5161d`): nine lines on the Building's floors, rewritten after a `lore-keeper` round ("Something's wrong." and "That's not right." cut: what a sound means; the knock at your cell neither a someone, which invites an answer, nor a something, which decides it; the all-clear never that it is over; no line to anyone): `siren` (first, again), `held` (knock, clear), `answered` (hatch, note), `uncanny` (knock_back, figure). 21 of 40 takes passing (tone 12, words 6, voice 2), every line with one; "What's that siren for?" was heard nearer "a radio announcer" on its first four (margins −0.01 to −0.05), given eight takes, and passes on one (+0.03): thin, for an ear. Posted by the assembly's siren and all-clear (sealed in), the cell's first knock, a door's answer, a marked door's knock back, the figure turned by your look. `tests/pacing.rs`'s crossings keep the asides: twelve floors, an arrival on each, the first siren answered and later ones, the knock answered inside the cell on three, the all-clear's word as the bot steps out (asserted: a siren answered, the knock's word only sealed in); `tests/knock.rs` and `tests/presences.rs` hold the answer, the knock back and the figure to telling the director. The full check's knock listen tests heard the word over the raps it answered (a knock back's rhythm is the tell): a knock's word now waits for its last rap to ring out (`KNOCK_HEARD`, 0.35 s), a door's answer for its thing to be out, held by `tests/knock.rs`; those listen tests measure the knock alone |
| A8 | done: six lines of the place answering, rewritten after a `lore-keeper` round ("That's a hand." would name a hand, said before the print was there: "Is that a handprint?" once it is all there; "That wasn't me." false of the candle, or a who for its notes: "I'm not staying here."; "It still runs?" a hint the dead lift works; "Something touched me." the screen's own words; a fourth "Was that…" avoided; one camp line, never whose): `uncanny` (hole, from the second floor: holes are on the Building's too; flap; lift; candle; pane) and `found` (camp). 19 of 24 takes passing, every line with one. A word about a sound waits for it to sound (A7's lesson): the lift's after its car (2.9 s), the candle's after its notes, the pane's after its tick (0.5 s), the flap's once the print is full (four fifths of its lift). Held by tests: `tests/vending.rs` (the flap), `tests/glass.rs` (the pane), `tests/secrets.rs` (the hole), `tests/asides.rs` (her camp, once a floor and not from afar; a lift's car and a candle's notes, told only after they sounded) |

## What only people can tell

- Does a line sound like a person in that place, or like a narrator reading?
- Is the cry for help frightening, or embarrassing, after the tenth time?
- Do the takes of one persona sound like one person?
