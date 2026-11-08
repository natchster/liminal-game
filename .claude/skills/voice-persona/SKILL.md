---
name: voice-persona
description: Make or change a voice persona in The Backrooms (who speaks, the voice designed from a description, its tone of voice, emotions and targets, its reference clips): the canon check, the persona file, the reference voice and any emotional references, the profile in the docs, and the gates.
---

# Make or change a voice persona

A persona is who speaks: one voice, kept the same across all its lines
(`docs/VOICES.md`). You cannot hear; the machine screens, a person decides.

1. **Brief against the canon.** Read `docs/LORE.md` (*How canon reaches
   the player*: no narrator, ever; a voice is heard from a place or is your
   own) and `docs/PILLARS.md`. Say who the persona is, where the game plays
   them, and what the game never says about them. **Never a real person**:
   no names, no "sounds like", no recordings of anyone.
2. **The toolchain.** `cargo xtask voice doctor` passes (if not:
   `uv sync --project tools/voice/synth`, `--project tools/voice/eval`,
   `cargo xtask voice fetch`).
3. **The persona file** `assets/src/voice/personas/<id>.toml` (copy
   `you.toml`, then set `design_seed = 1` and drop its `[references]`):
   at the top `id` (the file's name), `name` and `used_for`; `[person]` (summary, age, sex, from, here, never_said);
   `[voice]` (a description a voice-design model can follow: age, timbre,
   accent, texture, how it is recorded; `reference_text`, 10–15 s of plain
   speech; `design_model`: `qwen3-tts-voicedesign` or `voxcpm2`, the two
   that design a voice; `design_seed`; `line_model`: one of the four that
   speak, chosen by a bake-off, `voice-eval` step 4); `[tone]`
   (`direction` for an actor; `like` and `never`: short phrases the
   evaluation's CLAP compares takes with); `[emotions]` (each emotion the
   persona may take, with the words a model is directed with); `[targets]`
   (pitch band, loudness, peak, word error, speaker similarity calm and
   loud). `cargo test -p xtask voice` reads it: fix every field it names.
4. **The reference voice.**
   `cargo xtask voice reference --persona <id> --candidates 6` makes six
   voices from the description (seeds `design_seed` to `design_seed` + 5)
   and judges each, printing a row a seed: how synthetic AudioSet hears it,
   its pitch, its HNR (lower is rougher), its arousal, its words, and any of
   the checks that matter for a reference it fails (words, pitch, not
   synthetic; the full table is in `target/voice/<id>/reference/report.md`).
   Choose: no failed check; *synthetic* low (of the first persona's six,
   seed 1 scored 0.77, the chosen seed 4 0.03); pitch and roughness matching
   the description; and a person's ear when one is available. Set
   `design_seed`, write why beside it, then `--write`: it is kept as
   `assets/src/voice/personas/<id>.wav`, and every line is cloned from it.
5. **Emotional references**, when the line model cannot reach an emotion
   from the calm voice (the evaluation shows the line's arousal far short of
   what it asks, take after take): add `[references.<emotion>]` (model,
   text: a few lines in that emotion, seed), then
   `cargo xtask voice reference --persona <id> --emotion <emotion> --candidates 4`
   (the model must clone: `voxcpm2`, `chatterbox` or `ming-tts`). Each
   candidate is judged as a take of that emotion at intensity 0.9; choose
   the one with no failed check, the arousal asked and the highest voice
   similarity to the calm reference; set its seed, `--write`. It is kept as
   `assets/src/voice/personas/<id>.<emotion>.wav`, and that emotion's lines
   are cloned from it. The first persona's terror is made so (Chatterbox's
   screams went from arousal 0.4 to 0.9).
6. **The profile** in `docs/VOICES.md` (*Personas*: who, from, never said,
   voice, tone, emotions, lines, targets, reference), and in
   `docs/LORE.md` where the persona touches the canon (one line in the
   section the game plays them in, through a channel the canon allows).
7. **Gates and evidence.**
   - `cargo test -p xtask voice` (the files), `cargo xtask voice doctor`.
   - The candidates' table from step 4 in the commit message.
   - Ask the developer to listen to `assets/src/voice/personas/<id>.wav`
     (and each `<id>.<emotion>.wav`) before any line is made from it; record
     what they said.
8. **Then** its lines: the `voice-lines` skill.
