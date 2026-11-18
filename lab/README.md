# The lab: creature motion and plants, apart from the game

A workspace of its own for two things the game may take later, built and
looked at here first: **realistic motion for the creatures** (an open motion
model on this Mac, retargeted onto their bodies, the Smiler first) and
**generated plants** (the office's potted plants, alive, dying and dead).
Nothing in the game reads anything here; the game's build, its check
(`cargo xtask check`), its fixtures and its goldens never see it. The lab
reads the game's crates as they are (the engine, the bodies of
`bodies.bin` and their materials) and changes nothing in them.

The research behind it: [`../docs/research/character-motion.md`](../docs/research/character-motion.md)
(the motion models, Kimodo and its MLX ports, their licences) and
[`../docs/research/plant-modeling.md`](../docs/research/plant-modeling.md)
(plant generators and 3D models, what neglect does to a plant).

| Path | What |
|---|---|
| `motion/` | the motion pipeline (Python, uv): briefs → NVIDIA Kimodo on MLX → the clip library |
| `motion/clips/` | what it made: `smiler.motion` (the clips) and `report.md` (each clip's candidates and scores) |
| `viewer/` | `creature-lab`, the viewer (Rust, the game's Metal renderer): the motion on the creature's body, the game's animation beside it, the plants; a window, or frames offscreen |

## The viewer

```sh
cd lab
cargo run --release                                   # the window
cargo run --release -- --plants ../target/lab/plants  # the plant gallery, offscreen
cargo run --release -- --scene ../target/lab/scene    # the Smiler's scene, the game's animation | the new: frames, sheets, a video
cargo run --release -- --scene DIR --view eye --lamp  # the same from where you stand, by your lamp
cargo run --release -- --clips ../target/lab/clips    # every clip on the body, in place
cargo run --release -- --metrics ../target/lab        # the numbers, the game's animation against the new
cargo test --release                                  # the plants repeat and stay in budget; the retarget's rest
```

The window (keys in `viewer/src/window.rs`): Tab goes from the scene to one
clip to the plants; O puts the game's animation beside the new one on the
same path; W lays all, half or none of the Smiler's wrongness on the mocap;
L switches a study light for a dim room and one lamp; arrows change the clip
or the plants' seed; H their health; drag and the wheel turn and come nearer.

## The motion pipeline

```sh
cd lab/motion
uv sync                              # Python 3.12, MLX, kimodo-mlx at its pinned commit
.venv/bin/python motion.py fetch     # the models of models.toml into the Hugging Face cache (9.3 GB)
.venv/bin/python motion.py doctor    # there, at their revisions and hashes
.venv/bin/python motion.py gen       # every brief of clips.toml, its candidate seeds (cached)
.venv/bin/python motion.py bake      # → clips/smiler.motion, clips/report.md
```

1. **Briefs** (`motion/clips.toml`): a clip for each of the Smiler's modes
   (`../docs/design/ENTITIES.md`: Idle, Alert, Search, Hunt, Strike, over
   your body, Withdraw), each in plain words about a person ("A person runs
   forward fast."), how long, whether it loops, and the speed its rule
   needs.
2. **Generation** (`motion/kimodo_lab.py`): NVIDIA's Kimodo, the SOMA
   checkpoint trained on the public BONES-SEED mocap, on the GPU through
   kimodo-mlx (MIT). The prompt's embedding comes from the LLM2Vec Llama-3
   encoder in LocalAI's Q8_0 GGUF, a layer at a time (cached by prompt);
   every candidate seed of a brief is denoised in one batch with two rows of
   guidance. Each sample is cached by everything that decides it.
3. **The bake** (`motion/motion.py`): every candidate decoded (joint
   positions, foot contacts, the root) and scored: its speed against the
   brief, how far a foot slides while it touches the ground, jitter, the
   floor. The best is kept, its feet held still while they touch (eased in
   and out), a loop cut where the pose and its velocity best repeat and
   cross-faded there, and every frame turned into the creature's own frame
   (+Y up, +Z ahead, its left at +X, as the game's bodies are).
4. **The retarget** (`viewer/src/motion.rs`): each segment of the mocap gets
   a frame from joint positions alone (hip to hip, shoulder to shoulder, eye
   to eye, thumb to finger), so no rotation convention of the source matters,
   and its turn away from the mocap's neutral stance is laid on the body's own
   rest pose. A body bound in a stoop keeps its stoop. The legs are then
   solved onto the feet (two bones, a pole), the feet put where the mocap's
   are, scaled by the creature's legs over the mocap's (1.26 for the Smiler).
   The neutral stance poses the body exactly at rest (a test).
5. **The creature's layer** (`viewer/src/smiler.rs`): what makes it the
   Smiler's (the design's two vocabularies, stillness and wrong mechanics),
   laid on the mocap by rule. It keeps a part of the sway at rest and
   breathes slowly. Its head goes to what it watches in a snap and holds
   there, the body coming round after. Searching, it stops dead mid-step,
   holds, and lurches on to where it would have been. Running, its arms hang
   as dead weight. Its jaw opens as it comes. Each foot is locked to the
   world while the clip says it touches. The scene walks it through its modes
   toward a viewer; the game's animation (`viewer/src/legacy.rs`, copied from
   `entities::smiler::animate`) can walk the same path beside it.

## Results (2026-11-18)

**Generation.** Eleven briefs, four candidates each (eight for the sprint),
100 steps. The first run (ten briefs, forty candidates) took 251 s, 12.8 GB
at its peak (M4 Pro, 64 GB), the encoder under 30 s of it; a changed brief
samples only itself again (16–47 s). Every clip's candidates and scores are
in `motion/clips/report.md`; `motion/clips/smiler.motion` is 0.5 MB.

**What the first bake taught** (each now held by the pipeline):

- The neutral stance must be a person standing upright, not the idle: the
  first idle (asked for "a hunched back") had its head 34 cm forward and a
  forearm across the belly, and every clip inherited it (an arm held out
  sideways all through the stalk). It is its own brief now (`stand`), and
  the prompts no longer ask for the stoop the body already has.
- A clip must stay on its feet when its rule needs it: "crouches down low and
  then lunges forward" ended lying on the floor; the brief asks for a step
  and a grab, and the bake refuses hips below 0.6 m (`min_hips`).
- A loop's seam is part of its score, and a "run" came out a jog; a sprint
  was asked for, from eight seeds.
- Feet: a lock let go must step (lift, carry, set down), never ease along
  the floor; a foot the clip holds low and still is down before its contact
  flag says so; a steered beat ends when it arrives (a clip never runs on
  the spot); a clip's speed is its loop's as it plays, not the whole take's;
  a lock further from its hip than the leg comfortably reaches is stepped
  from, and the pelvis dips at most 10 cm for a foot (more, and the body
  knelt mid-sprint, as the blind review caught).

**The scene, measured** (`--metrics`: the same 30 s on the same path and
body; the game's animation is `entities::smiler::animate` as it is today):

| | the game's | the new |
|---|---|---|
| a foot sliding while it touches the floor (ankle or ball within 1.5 cm of its lowest) | 0.079 m/s | 0.022 m/s |
| jerk of the body's joints (lower is smoother) | 506 m/s³ | 332 m/s³ |
| how much the body turns at its joints | 0.21 rad/s | 0.87 rad/s |
| the same while it stands in its dark (stillness) | 0.010 rad/s | 0.018 rad/s |

The new body moves four times as much and is smoother doing it; its feet
slide a quarter as much (searching 0.014 against 0.263 m/s, the sprint 0.074
against 0.593, withdrawing 0.020 against 0.300); at rest it stays almost as
still. The game's stepping in held snaps keeps its feet put in the stalk
(0.024 against 0.054 m/s) by never moving them while they bear weight.

**The blind review** (`cargo xtask ab`: a strip a beat, a frame every 0.2 s,
both orders, a fresh `art-critic` asked which side moves like a real body
with weight while still reading as the wrong, predatory thing it is):
the new preferred on six beats in both orders (creep, listen, stalk, lunge,
over your body, withdraw), the game's on none, four ties (the idle, the
look and the turn, which are stillness by design and nearly the same on
both; the sprint, half a second long). Logged to
`crates/backrooms/tests/review/verdicts.jsonl`. The reviewer flagged a
near-kneeling frame in the sprint and a pop out of the strike; both were
fixed after (above) and are not yet reviewed again.

**What is left.** The sprint averages 2.1 m/s on the mocap's scale and is
played 1.7 times over to reach the design's 4.5 m/s; the design's lunge (7
m/s over up to 4 m) is not a person's motion, and the clip is a step and a
grab at its own pace. Kimodo's constraints (a root path, end-effector
targets) would let a clip be asked for exactly; kimodo-mlx does not have
them yet. Nothing here reaches the game: bringing it there is the game's
own work (a body's clips in `bodies.bin`'s neighbour, the retarget in
`entities::body`, the layer in `entities::smiler`, and the goldens).
