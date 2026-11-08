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

## Personas

(Each persona's profile, as it is made: V2 drafts the first.)

Next after the first: the child below you in the Stairwell ("please",
"help", "down here", the sob and the wail).

## What only people can tell

- Does a line sound like a person in that place, or like a narrator reading?
- Is the cry for help frightening, or embarrassing, after the tenth time?
- Do the takes of one persona sound like one person?
