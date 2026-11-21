---
name: creature-motion
description: Make or change a creature's motion in The Backrooms - its clips (NVIDIA Kimodo, from plain briefs) and the bridges between them (NVIDIA MotionBricks - MLX on Apple Silicon, CUDA on Linux), baked into its library, played by the game, looked at in the lab, held by the tests and judged blind. Use for new or changed clips, bridges, how a creature starts, stops or goes from one mode to the next, and for running the motion pipeline on either platform.
---

# A creature's motion: clips and the bridges between them

A creature moves by **clips**, a person's motion made from words by NVIDIA
Kimodo, laid on its body with its wrongness on top; and it goes from one clip
into the next by **bridges**, the motion between two clips made by NVIDIA
MotionBricks, never two poses averaged (`docs/design/ENTITIES.md` §6). Read
first: that section, `lab/README.md` (the pipeline),
`docs/research/character-motion.md` (the models, their licences, and what
joining MotionBricks to the game's clips found) and `.claude/rules/entities.md`.

Where each command runs: `motion.py` from `lab/motion` with its own
interpreter (`cd lab/motion && .venv/bin/python motion.py …`); the lab's
viewer from `lab` (`cd lab && cargo run --release -- …`, its paths relative
to `lab`); everything else from the repository's root.

1. **Where it runs.** The pipeline is `lab/motion/`, a uv project of its own.
   - **Apple Silicon (macOS)**: all of it. `uv sync` (kimodo-mlx, and
     MotionBricks' MLX port
     [natchster/motionbricks-mlx](https://github.com/natchster/motionbricks-mlx),
     each at its pinned commit), `motion.py fetch` (Kimodo, its text encoder
     and MotionBricks' SOMA weights, at their revisions in `models.toml`),
     `motion.py doctor` (each there and checked; which backend).
   - **Linux with an NVIDIA GPU**: the bake and its bridges, on NVIDIA's own
     MotionBricks graph ([NVlabs/motionbricks](https://github.com/NVlabs/motionbricks)
     v0.2.0's ONNX) on ONNX Runtime's TensorRT or CUDA provider:
     `uv sync --group cuda`, `motion.py fetch` (the graph and the rig's
     offsets, pinned by SHA-256), `motion.py doctor`;
     `MOTIONBRICKS_BACKEND=cuda` is the default there. Sampling new clips
     (`gen`) runs Kimodo on MLX, so make them on a Mac (or with NVIDIA's own
     Kimodo, outside this pipeline).
   - The downloads go into uv's and Hugging Face's caches, outside the
     repository by design (no model is ever in it): about 10 GB the first
     time; ask the developer before a first `fetch` on their machine.
   - Both backends take the same tensors and every library records which
     made its bridges (`made_by`, and the first lines of its report's
     Bridges); the two agree to a few micrometres, so re-bake a creature
     on one. A checkout of your own is used through `MOTIONBRICKS_MLX=DIR`
     (a clone of the port) or `MOTIONBRICKS_ONNX=FILE` (a copy of the
     graph); nothing is ever written there.
2. **Briefs** (`lab/motion/briefs/<kind>.toml`): one `[[clip]]` for each thing
   its modes need, in plain words about a person ("A person walks slowly
   forward while reading..."; Kimodo knows people, not the wrong): seconds,
   `loop`, the `speed` its rule needs; one `neutral = true`, standing as its
   body is bound (upright, a stoop, all fours); `fours = true` for a body on
   all fours. What makes it the creature's is laid on in its `pose`, never
   asked of the model.
3. **Clips**: `motion.py gen --creature <kind>` (every candidate seed of each
   brief, cached by everything that decides it in `target/lab/motion/raw/`).
   The cache is not in git: the bake reads it on every platform. A clip with
   no samples is skipped ("no candidates (run gen)"), a missing seed changes
   which take is kept, and sampling again on another machine is not
   promised the same: bake where the samples were made (or bring them),
   and after a fresh `gen` look and review again.
4. **Bridges**: in the same briefs, a `[[bridge]]` for each change of clip its
   `pose` makes: `from` (a clip or a list), `to`, and `at`, where in `to` it
   lands (its `Pick`'s `from`; or later where `to` begins with a person
   stopping, as a listen does: the bridge is the stop). Find the changes in
   its `pose` (the `Pick` each mode makes, moving and still) and in the lab
   (from `lab`: `LAB_BRIDGES=1 cargo run --release -- --metrics --creature <kind>`
   names each bridge its scene plays; a change with none is blended). None
   into a clip scrubbed by a rule's clock (`Pick::at`: the Reader's page,
   the climb through glass).
5. **Keep a copy** of the libraries as they are, before baking (step 9
   compares against it): from the root,
   `mkdir -p target/lab/motion/before && cp crates/game/assets/motion/*.motion target/lab/motion/before/`.
6. **Bake**: `motion.py bake --creature <kind>` scores and cuts its clips, then
   makes its bridges (`bricks.py`): 0.7 s each, leaving a travelling clip
   every 0.25 s and a standing one every 0.6 s, the target placed where the
   game's root would be by then (it starts and stops in a tenth of a
   second), lined up by the clips' own facing, its ends joined exactly to
   the frames it leaves and lands on (a planted foot takes one correction
   while it touches). Read `lab/motion/clips/<kind>.md`, its Bridges: each
   bridge's `fix` (how far the model missed the poses it was given, over
   the body; and the most any joint was moved) and its last line, "Left to
   the blend": none, or each bridge dropped and why (its body missed by
   more than 8 cm, or a joint by more than 30 cm; that change keeps the
   blend). `--no-bridges` bakes the clips alone, the same bytes as before
   bridges.
7. **Install**: `motion.py install` copies every kind's library into
   `crates/game/assets/motion/`; a kind you did not change must come out the
   same bytes (`cmp` against the copy of step 5). A new kind also goes into
   `motion::KINDS` and `bytes`.
8. **Play** (`motion::anim::Anim`, `crates/game/src/entities/motion/anim.rs`):
   its `pose` picks a clip a mode; `Anim` reaches it by its bridge (moving
   on, it waits at most 0.3 s for the next to leave and starts on its very
   frame; stopped, it leaves at once, blended in over 0.15 s), plays it to
   its own speed, begins no held frame on one, blends a change straight back
   to the clip a bridge left, and, standing, steps a foot left more than
   10 cm from where its clip wants it. Nothing per frame is drawn from the
   game's streams (invariant 1). A rule changed here changes every kind:
   look at them all in step 9.
9. **Look**, in the lab (from `lab`; it refuses a folder that is not there,
   never quietly showing the game's own clips in its place):
   - `cargo run --release` (its number key; Tab: its scene, by hand, its
     clips and bridges with `← →`);
   - side by side against the copy, left the copy and right the new:
     `cargo run --release -- --scene DIR --creature <kind> --motion motion/clips --against ../target/lab/motion/before`
     (frames, strips a beat, a video);
   - the numbers: `--metrics DIR --creature <kind> --motion motion/clips --against ../target/lab/motion/before`
     (foot slide and jerk, beat by beat; all kinds without `--creature`).
   A scene plays only the bridges its beats reach (`LAB_BRIDGES=1` lists
   them); see the rest with `--clips DIR --creature <kind> --bridges` (each
   clip and bridge on its body, in place) or drive it by hand in the window.
10. **Tests**: `cargo test --release -p game --test motion` (every neutral its
    body's rest, every mode's clip, every bridge joins its clips to half a
    centimetre and the Hound's stay on all fours, a stop goes by its bridge
    and lands with its feet set, a pose a function of its inputs); then
    `cargo xtask check --fast`, and the full check for creatures. Its motion
    moves the creature goldens (`cargo xtask shots`: each kind at 2, 4 and
    8 m and its death): look at the frames and their error maps; re-recording
    them is the developer's (`/regen-fixtures`).
11. **Blind review** (`visual-review`): strips of each change of clip the
    scenes play, frames 0.2 s apart, **cropped close round the creature**
    (at the lab's frame size it was 30-80 px and reviewers could only tie),
    the new against the copy, both orders, two fresh `art-critic`s; read
    each loss's evidence, fix its cause, and ask fresh reviewers again.
12. **Ledger and docs**: the kind's entry in `docs/plan/content.json` (what
    its motion is now, its review), a milestone and its progress in
    `docs/plan/ENTITIES.md`, anything learned about the models in
    `docs/research/character-motion.md`. Before a release: MotionBricks'
    licence (the NVIDIA Open Model Agreement) and Bones RigPlay's terms,
    Kimodo's and BONES-SEED's, read whole.
