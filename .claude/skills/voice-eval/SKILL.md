---
name: voice-eval
description: Evaluate voice takes in The Backrooms (words, emotion, tone of voice, voice): read the report, choose the takes, run a bake-off between models, and change the evaluation only through known answers (voice selftest).
---

# Evaluate voice takes

The evaluation (`tools/voice/judge.py`, `docs/VOICES.md` *The evaluation*)
is a screen for a person's ear, not a verdict. Know what each check can and
cannot tell before trusting it.

1. **Run it.** `cargo xtask voice eval --persona <p> [--model M]` judges
   every take in `target/voice/<p>/<M>/`; `target/voice/<p>/<M>/report.md`
   has a row a take (★ the chosen one), `target/voice/<p>/<M>/report.json`
   every measure. The gate fails when a line has no take that passes all
   six checks. The log is long (the models' INFO lines and warnings,
   speechbrain saying it fetches from the hub "if not cached": it runs
   offline); the lines that matter are the last ones.
2. **Read a failure by its check.**
   - **words**: Whisper's word error over `wer_max`. Parakeet's error is
     beside it in brackets; a take the two hear differently is unclear
     speech. A take marked *(silent)* is one the model left (nearly)
     silent.
   - **emotion**: WavLM's arousal outside ±0.25 of what the emotion asks at
     its intensity (the column shows arousal, the wanted arousal, then
     valence), or valence over 0.55. Too low: the model read it calmly
     (more takes, a direction, an emotional reference). Too high on a calm
     line: it shouted. The score after the semicolon only ranks takes (it
     mixes in the classes of emotion2vec+ and WavLM, which both hear
     frightened shouting as *angry*, *disgusted* or *surprised*): a take can
     score low and pass, or higher and fail.
   - **pitch**: the median outside the persona's band. Over it, usually a
     falsetto shriek (VoxCPM2's shouts at 590–650 Hz).
   - **tone**: CLAP hears the take nearer one of the persona's `never`
     phrases (named in brackets) than its `like` ones. Synthetic screams
     land near "a cartoon character".
   - **voice**: ECAPA similarity to the reference under the floor for the
     line's intensity. On calm lines it tells him from another man; on
     shouts it cannot (another man's shout reads like his), so a loud
     line's voice is for an ear.
   - **not synthetic**: AudioSet's *speech synthesizer* at 0.5 or more.
3. **Choose.** The best passing take is chosen by score (emotion first,
   then voice, tone, words); `voice write` takes it. Overrule it only with
   `--take id=K` (that line) and a reason in the commit (a person preferred
   another); `--take K` sets every line.
4. **A bake-off** (choosing a persona's `line_model`): `voice build` and
   `voice eval` with each `--model` of the four that speak (`voxcpm2`,
   `qwen3-tts-voicedesign`, `chatterbox`, `ming-tts`), then
   `cargo xtask voice bakeoff --persona <p>` prints the table: lines with a
   passing take, takes passing, mean voice similarity, failures by check.
   The hardest lines decide; record the table and the choice
   (`docs/VOICES.md`, *The bake-off*).
5. **Changing the evaluation** (a threshold, a check, a model): first make
   the known answer that shows the need (a take that must fail the check,
   or one that must pass) and add it to `pipeline::selftest`
   (`xtask/src/voice/pipeline.rs`: a job in `jobs` that makes it, a row in
   `judged` saying what it was meant to be, a `Known` naming the check it
   must fail, or none); then change `judge.py`; `cargo xtask voice selftest`
   must give every answer right (the bad fail their own check, the good
   pass all; its own gate prints FAILS first, by design, since the bad ones
   fail); then re-run `voice eval` for every persona and say what moved.
   Its good takes are VoxCPM2's whatever the line model. Write the numbers
   behind a threshold beside it (`judge.py`) and in `docs/VOICES.md`.
6. **Never grade by ear: you cannot hear.** Say what the measures say and
   ask the developer to listen (`docs/PLAYTEST.md`).
