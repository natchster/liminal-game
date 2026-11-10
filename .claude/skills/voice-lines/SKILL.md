---
name: voice-lines
description: Write and make a voice persona's lines in The Backrooms (the line file, the takes from the line model, the evaluation and its gate, the chosen takes into the game's assets and code, the listen test), ending in a person's ears and the ledger.
---

# Write and make a persona's lines

The persona exists and has its reference (`voice-persona` skill). You cannot
hear: the evaluation screens every take, a person makes the last call.

1. **The lines** `assets/src/voice/lines/<persona>.toml`: at the top
   `persona = "<id>"` (the file's name), then each `[[line]]`
   with an `id` (lower case, `_`), its `text` (what a person in that place
   would say, short; a one-word scream gets said twice, so write it twice),
   its `emotion` (one of the persona's) and `intensity` (0 calm … 1 a
   scream), its `use` (where the game says it: one of `voice::USES`, which
   the game must play) and `levels` (first, last), and `takes` (4).
   `direction` overrides the emotion's words for one line. `subject`
   (lower case, `_`) says what a line is about within its use (a place, a
   kind of thing): the game chooses a subject's lines before the use's
   general ones.
   `cargo test -p xtask voice` reads them.
2. **The takes.** `cargo xtask voice build --persona <p>` (the persona's
   `line_model`; `--model M` for another; `--only id,…` for some lines):
   raw and cleaned takes in `target/voice/<p>/<model>/`. Every take of the
   chosen lines is made again each run (`--only` for the ones you changed);
   a take is made again the same (seed = take number), so adding takes makes
   new ones and never changes the old.
3. **The evaluation.** `cargo xtask voice eval --persona <p>`: the report
   (`target/voice/<p>/<model>/report.md`), each line's best passing take,
   the gate (`voice-eval` skill to read it). When a line has no passing
   take, in this order: more takes; a better `direction`; an emotional
   reference for its emotion (`voice-persona` step 5); its wording; another
   model through a bake-off. **Never loosen a threshold to pass a line**:
   thresholds change only through `voice selftest`'s known answers.
4. **Into the game.** `cargo xtask voice write --persona <p>`: the chosen
   takes into `crates/game/assets/voice/<p>/`, their `MANIFEST.json`
   entries, the index `lines.tsv`. In `crates/game/src/audio/voice.rs`,
   one `wav!("<p>", "<id>")` line in `WAVS` per new line. A chosen take
   can be overruled: `--take id=K,…` (`--take K` sets every line). A new
   `use`: add it to `voice::USES` (xtask) and give the game the code that
   plays it, choosing from a stream of its own (never the per-frame stream;
   `CLAUDE.md` invariant 1). The one player of voice lines so far is
   `say_cry` (`crates/game/src/audio/sys.rs`): your own voice, centred into
   the soundscape's `world` bus, choosing among the persona `you`'s lines
   only (`voice::CRY_PERSONA`). A voice that is someone else's comes from a
   place, like any sound (`play_from`, `.claude/rules/audio.md`).
5. **Gates.**
   - `cargo test -p xtask voice` (the files and the index),
     `cargo test --release -p game --lib audio::voice` (every line decodes,
     the list and the index agree, the choice),
     `cargo test --release -p backrooms --test structure` (the manifest).
   - `cargo test --release -p backrooms --test listen -- --nocapture`: the
     cry (`the_cry_is_your_voice`) stands over the room and never clips; it
     must fail with `VOICE_GAIN` (`crates/game/src/audio/sys.rs`) at 0 (try
     it once). A new `use` gets a listen test of its own, made the same way.
   - `cargo xtask check` (audio is touched): no golden, reference level or
     loudness scenario moves.
6. **Ears.** The chosen WAVs and `target/listen/` renders to the developer
   (`docs/PLAYTEST.md`); record what they said in the commit. What they
   reject goes back to step 3 and, if it is a pattern, into `voice selftest`
   as a known answer.
7. **Ledger.** The `sound/voice` entry in `docs/content.json`, open issues
   in `docs/QUALITY.md`, the persona's *Lines* row in `docs/VOICES.md`.
