# The voices: the bake-off

The first persona's lines from four text-to-speech models, through the
evaluation (`docs/engineering/VOICES.md`), and the model chosen; a record of
that comparison, kept as found. The design it served is
[`../design/VOICES.md`](../design/VOICES.md).

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
