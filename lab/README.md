# The lab: creature motion and plants

A workspace of its own for **the creatures' motion** (an open motion model on
this Mac, retargeted onto their bodies) and **generated plants** (the
office's potted plants, alive, dying and dead), each made and looked at here
first. The creatures' motion has gone into the game: the clips the pipeline
makes are copied into `crates/game/assets/motion/` and the game plays them
(`entities::motion`, `docs/design/ENTITIES.md` §6); the lab's viewer calls the
game's own code, so what it shows is what the game does. The plants are
still the lab's alone. The game's build, its check and its fixtures never
read this folder; the lab reads the game's crates and changes nothing in them.

The research behind it: [`../docs/research/character-motion.md`](../docs/research/character-motion.md)
(the motion models, Kimodo and its MLX ports, their licences) and
[`../docs/research/plant-modeling.md`](../docs/research/plant-modeling.md)
(plant generators and 3D models, what neglect does to a plant).

| Path | What |
|---|---|
| `motion/briefs/<kind>.toml` | each creature's moves, one brief a clip, in plain words about a person |
| `motion/` | the pipeline (Python, uv): briefs → NVIDIA Kimodo on MLX → each creature's clip library |
| `motion/clips/` | what it made: `<kind>.motion` (the clips) and `<kind>.md` (each clip's candidates and scores) |
| `viewer/` | `creature-lab` (Rust, the game's Metal renderer): every creature moved by the game's code, the old animation beside it, the plants; a window, or frames offscreen |

## The viewer

```sh
cd lab
cargo run --release                                    # the window
cargo run --release -- --scene ../target/lab/scene     # every creature's scene, the old animation | the new: frames, sheets, strips, a video
cargo run --release -- --scene DIR --creature hound --view eye --lamp   # one, from where you stand, by your lamp
cargo run --release -- --lineup ../target/lab          # all six through their scenes, side by side
cargo run --release -- --clips ../target/lab/clips     # every clip on its body, in place
cargo run --release -- --metrics ../target/lab         # the numbers, the old animation against the new, beat by beat
cargo run --release -- --motion motion/clips …         # a bake not yet installed in the game
cargo run --release -- --scene DIR --motion motion/clips --against DIR2   # beside it the same creature on DIR2's libraries (one bake against another)
cargo run --release -- --plants ../target/lab/plants   # the plant gallery
cargo test --release                                   # the plants repeat and stay in budget
```

The window (keys in `viewer/src/window.rs`): **1–6** pick the Smiler, the
Hound, the Watcher, the one in the corner, the Reader, the one behind the
glass; **0** shows all six side by side. **Tab** goes from its scene (its
modes, as its brain and the director would take it toward you) to driving
it by hand (**[ ]** its mode, **- =** its speed round the stage from still
to 7 m/s, **K** over your body; it watches the camera) to its clips one by
one (**← →**) to the plants. **O** puts the game's animation from before
its clips beside it on the same path (`viewer/src/legacy.rs`, copied from
1718a5b); **W** lays all, half or none of its wrongness on the mocap; **L**
a study light or a dim room and one lamp; Space pauses, `.` steps; drag and
the wheel turn and come nearer.

`LAB_BRIDGES=1` with `--metrics` prints each bridge a scene plays (its
beat, when). `LAB_TRACE=kind:beat` prints each frame of a beat (its clip, pace, stride,
each foot's lock and where it is); `LAB_POSE=kind` the bones its first
second turns furthest from rest; `LAB_REST=1` with `--metrics` the bones
that move while it stands.

## The motion pipeline

```sh
cd lab/motion
uv sync                              # Python 3.12, MLX, kimodo-mlx and natchster/motionbricks-mlx at their pinned commits
.venv/bin/python motion.py fetch     # the models of models.toml into the Hugging Face cache (10 GB)
.venv/bin/python motion.py doctor    # there, at their revisions and hashes; MotionBricks' backend
.venv/bin/python motion.py gen [--creature hound]    # every brief, its candidate seeds (cached)
.venv/bin/python motion.py bake [--creature hound]   # → clips/<kind>.motion, clips/<kind>.md
.venv/bin/python motion.py install   # → crates/game/assets/motion/ (the game embeds them)
```

**Where it runs.** On **Apple Silicon** all of it: Kimodo on MLX
(kimodo-mlx) and MotionBricks on MLX, its port
[natchster/motionbricks-mlx](https://github.com/natchster/motionbricks-mlx)
(checked here against NVIDIA's ONNX graph: the same durations chosen, the
joints within 6 µm). On **Linux with an NVIDIA GPU** the bake and its bridges
run on NVIDIA's own MotionBricks graph
([NVlabs/motionbricks](https://github.com/NVlabs/motionbricks) v0.2.0's
ONNX, on ONNX Runtime's TensorRT or CUDA provider):

```sh
cd lab/motion
uv sync --group cuda                 # ONNX Runtime with CUDA (MOTIONBRICKS_BACKEND=cuda is the default on Linux)
.venv/bin/python motion.py fetch     # Kimodo's checkpoint, and NVIDIA's graph and rig (758 MB, pinned by SHA-256)
.venv/bin/python motion.py doctor
.venv/bin/python motion.py bake [--creature hound]   # given Kimodo's samples (target/lab/motion/raw/)
```

Sampling new clips (`gen`) runs Kimodo on MLX, so it is made on a Mac (or
by NVIDIA's own Kimodo, outside this pipeline); the bake needs only the
samples and Kimodo's checkpoint, no MLX. `MOTIONBRICKS_BACKEND=mlx|cuda`
chooses the backend; `MOTIONBRICKS_MLX=DIR` (a clone of the port) or
`MOTIONBRICKS_ONNX=FILE` (a copy of the graph) use a checkout of your own,
read only. Each library and report names what made its bridges.

1. **Briefs** (`motion/briefs/<kind>.toml`): a clip for each thing a
   creature's modes need (`../docs/design/ENTITIES.md`), each in plain words
   about a person ("A person walks slowly forward while reading a sheet of
   paper held with both hands in front of their face."), how long, whether
   it loops, the speed its rule needs. One brief is its **neutral**: a
   person standing as the body is bound (upright; stooped over a page; on
   hands and feet), since every clip is laid on the body as its turn away
   from it. Two creatures with the same brief share its samples (the
   Smiler's `stand` serves the corner's and the glass one's).
2. **Generation** (`motion/kimodo_lab.py`): NVIDIA's Kimodo, the SOMA
   checkpoint trained on the public BONES-SEED mocap, on the GPU through
   kimodo-mlx (MIT). The prompt's embedding comes from the LLM2Vec Llama-3
   encoder in LocalAI's Q8_0 GGUF, a layer at a time (cached by prompt);
   every candidate seed of a brief is denoised in one batch with two rows of
   guidance. Each sample is cached by everything that decides it.
3. **The bake** (`motion/motion.py`): every candidate decoded and scored
   (its speed against the brief, how far a foot slides while it touches,
   jitter, the floor; `min_hips`: on its feet; `fours`: the head low and
   the hands on the floor), the best kept, its feet planted, a loop cut
   where the pose and its velocity best repeat and cross-faded, every frame
   turned into the creature's own frame (+Y up, +Z ahead, its left at +X).
   Six contacts a frame: each foot's heel and toe, each hand. **On all
   fours** (the Hound) BONES-SEED crawls on its knees: the knee is what meets
   the floor in step, so the knees are planted and stand for the feet.
4. **Bridges** (`motion/bricks.py`, after the clips in the same bake;
   `bake --no-bridges` leaves them out): NVIDIA MotionBricks (SOMA, the
   NVIDIA Open Model Agreement; on MLX on Apple Silicon, on NVIDIA's ONNX
   graph on CUDA on Linux: above) makes the motion from each clip into each its creature
   goes to (`[[bridge]]` in the briefs: `from`, `to`, `at` where in `to` it
   lands): given the four frames of the one that lead to where it leaves
   and the four of the other from where it lands, placed where the game's
   root would be after 0.7 s (it starts and stops in a tenth of a second),
   lined up by the clips' own facing. Bridges leave a travelling clip every
   0.25 s, a standing one every 0.6 s; their ends are moved onto the frames
   they join (what they plant takes one correction while it touches), and
   one whose body missed its poses by more than 8 cm is left to the blend.
   Each kind's report (`clips/<kind>.md`) lists them; in the library they
   are 16-bit steps, deflated (the five libraries 3.7 MB, 1.4 without).
5. **In the game** (`crates/game/src/entities/motion/`): the retarget lays a
   frame on the body by segment deltas from the neutral (no rotation
   convention of the source matters; a body bound in a stoop keeps it), the
   legs (on all fours the arms too) solved onto their ends, scaled by its
   legs over the mocap's. `Anim` plays what the kind's `pose` picks: reached
   by its bridge where there is one (moving on, it waits at most 0.3 s for
   the next to leave, which starts on its very frame; stopped, it leaves at
   once from the nearest, blended in over 0.15 s), else blended;
   played to the speed the creature moves at (a pace and a stride, shared
   by the pick's `cadence`), held and snapped, its feet locked to the floor
   while they bear weight and stepped when they let go. The kind's `pose`
   lays its wrongness on top (`smiler.rs`, `hound.rs`, `corner.rs`,
   `reader.rs`, `glass.rs`); `tests/motion.rs` holds the rest, every mode's
   clip, the straight legs under a garment, planted feet, and a pose as a
   function of its inputs.

## What each creature asks for

| Kind | Neutral | Clips | Its own on top |
|---|---|---|---|
| the Smiler | `stand` | idle, look, creep, listen, stalk, run, lunge, stand_over | tall, a little of its stoop kept; held frames searching; the head first in snaps; dead arms; the jaw, the fingers |
| the Hound | `fours` (on hands and feet) | prowl, creep, crawl, listen, sniff, spring, gallop | hands planted as feet; the knees for feet; quickened into a scrabble; the left hind dragging; stopping dead mid-step at a sound, then its crouch; its hanging head turned to the sound in snaps, swinging as it moves; the fingers digging in; panting after a charge; the bay; hair to the floor |
| the one in the corner | `stand` | hold, run, grab, walk_back | none of a person's sway at rest, straight legs under the nightshirt; its head further than a neck goes (its rule's); dead arms; the scream |
| the Reader | `reading` (stooped over a page) | read_walk, read_still, pick_up, hurry, tear | the page before its face; the stoop and the tear scrubbed by its rule's clock; the eyes along the lines; the look up; the sheet lowered |
| the one behind the glass | `stand` | still, walk, reach_walk, climb | a walk too even (no rise and fall, arms still, quick short steps); its hands flat on the glass; facing you walking back |
| the Watcher | (none) | (none) | never seen to move: its poses are its sheet's morphs (`watcher::POSES`), shown in the lab as you would catch it, a little nearer each time |

## Results (2026-11-19)

**Generation.** Five libraries, 31 clips, 1.4 MB in the game. The first
Smiler run (ten briefs, forty candidates) took 251 s at 12.8 GB peak (M4
Pro, 64 GB); each new creature's briefs 2–4 minutes (15–50 s a brief).

**Measured** (`--metrics`: each creature's scene, the same path, the same
body; the old is the game's animation before clips):

| | foot slide while it touches, old → new | joints turning, old → new |
|---|---|---|
| the Smiler | 0.092 → 0.033 m/s | 0.18 → 0.71 rad/s |
| the Hound | 0.256 → 0.010 m/s | 1.09 → 1.65 rad/s |
| the one in the corner | 0.003 → 0.022 m/s | 0.15 → 0.68 rad/s |
| the Reader | 0.131 → 0.026 m/s | 0.26 → 1.01 rad/s |
| the one behind the glass | 0.195 → 0.032 m/s | 0.25 → 0.90 rad/s |

The bodies move three to six times as much, and their feet slide a sixth to a
twenty-fifth as much; only the one in the corner's old feet slid less, since
they barely moved. Jerk rose for all but the Smiler (more of the body moves,
and faster). At rest the Smiler, the corner one and the glass one turn
0.02–0.04 rad/s against 0.00–0.01: a breath and a shift of weight.

**The blind review** (`cargo xtask ab`, a strip a beat, a frame every 0.2 s,
both orders, a fresh `art-critic` asked which side moves like a real body
with weight while still reading as the wrong, predatory thing it is): the
new preferred on 15 of 20 beats, the old on one (the corner one's walk back
to its corner: its small stiff steps under the nightshirt read as standing
still), four ties (its head turning, which is stillness by design; the
Hound's search and spring; the climb through the window). The Reader's
stoop lost the first round (the clip's end held the page high over its
head) and won the second once its page came back to its face. Logged to
`crates/backrooms/tests/review/verdicts.jsonl`.

**What the work taught** (each now held by the code or a test):

- A clip that never holds must end a hold begun before it, or the next clip
  starts frozen and its feet are dragged (the Smiler's stalk slid 0.58 m/s).
- A body moving off its heading is turning, not walking backward: only a
  kind that faces you as it goes (`Pick::backs`) plays a clip back to front.
- A re-planting step lands where the foot will be wanted, and is quicker
  the faster it goes.
- A pole on a nearly straight limb is noise: the knee's bend toward it is
  scaled by how bent the limb is.
- A body bound with straight legs is never dropped below its own rest for
  the margin a bent one is given; under a long garment its legs take none
  of the mocap standing (a knee bent 18° creased the nightshirt at the
  knees and lifted its hem).
- A foot is planted only when down and still; the test counts it skating
  only on the floor (a foot lifting away is not skating).
- A reach test from the hip at rest fired late in every fast stance and
  cascaded into tiny steps (the Reader's hurry slid 3 m/s): reverted; the
  Hound's trouble was its mocap's knees, not reach.

**Heavy and upright (2026-11-19, the developer's direction: weigh more,
stand more upright, uncanny but not cartoonish).** Each body stands part of
the way out of the stoop it is bound in (`Retarget::upright`, per kind: the
Smiler 0.55, the Reader 0.4 with its head still bowed to its page, the one
in the corner 0.35, the one behind the glass 0.3, the Hound 0.08: its chest
a little higher, never reared), its legs straightening as it stands taller.
Its weight: the briefs ask for a heavy person, upright, placing each foot
down heavily; each footfall sinks the pelvis on its legs, critically damped
(it settles, never springs back: `Anim::mass`); strides lengthen more than
steps quicken (`cadence` 0.42); nothing is played much past twice its pace
(the Hound's crawl 3.3); a held frame's lurch takes 0.22 s (80 ms was a film
skipping), a head goes to its target quickly, never in a jump. Measured
(each scene): feet slide 0.032 (the Smiler), 0.024 (the Hound), 0.024 (the
one in the corner), 0.031 (the Reader), 0.017 m/s (the one behind the
glass). Blind review against the version before (a fresh `art-critic` each
round, asked which side moves like a heavy, upright body, uncanny without
being cartoonish): the first round preferred the old on five beats, and
its evidence named why (the heavy run and lunge kicked the back foot up as
if running, "long steps" flung the corner one's legs out of its
nightshirt, the Hound's crawl capped at twice its pace sprawled flat
between arches, a footfall's dip rebounded like a hop); those briefs went
back to their first words, the dip was damped, the Hound's crawl played
faster; the second round preferred the new on five of eight, two ties, the
old on the Reader walking away (its hunched neutral made an upright walk a
lean back on top of its posture: it takes half the mocap's lean now).
What the heavy briefs did to the Hound: its knees no longer met the floor
in step (0.2 m/s of slide), so it keeps its first crawl and gets its weight
from the game.

**What is left.** The Hound's charge (6 m/s) and spring (7 m/s) are far past
any person's crawl (the fastest found, 0.59 m/s): played as a scrabble,
quicker more than longer, and still a little short of the ground. The
design's lunge is not a person's motion either. Kimodo's constraints (a
root path, end-effector targets) would let a clip be asked for exactly;
kimodo-mlx does not have them yet. The garments are linearly skinned, so a
long stride still pushes a leg through the nightshirt (as the old one did).
The licences: read [`../docs/research/character-motion.md`](../docs/research/character-motion.md)
*Risks* before a release ships with these clips.
