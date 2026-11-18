# Character motion from open models, on this Mac

*Gathered 2026-11-18 for the lab ([`../../lab/README.md`](../../lab/README.md)):
which open models make realistic human motion, run on an M4 Pro with 64 GB,
and may be used for a game that is sold; NVIDIA's Kimodo and its MLX ports
first. What was measured here says so; the rest links its source.*

## The answer

**NVIDIA Kimodo**, its SOMA checkpoint trained only on the public BONES-SEED
mocap (`nvidia/Kimodo-SOMA-SEED-v1.1`), run on the Mac's GPU through
**kimodo-mlx** (MIT), with the text encoder read from LocalAI's Q8_0 GGUF.
It makes 30 fps motion of a person from a sentence in seconds, the feet's
contacts with it, and nothing else on the list is both licensed for a sold
game and runnable here. The motion is retargeted onto the creature's own body
(the lab's viewer: `lab/viewer/src/motion.rs`), and what makes it the
creature's is laid on top (`lab/viewer/src/smiler.rs`).

## Kimodo

A kinematic motion diffusion model from NVIDIA's Spatial Intelligence Lab,
released 2026-03-16 (v1.1 on 2026-04-10): a two-stage transformer (282 M
parameters; a root stage, then a body stage) denoising a clip of motion
features, conditioned on a text embedding and optionally on constraints
(full-body keyframes, 2D root waypoints or paths, end-effector positions and
rotations) ([paper](https://arxiv.org/abs/2603.15546),
[project](https://research.nvidia.com/labs/sil/projects/kimodo),
[code](https://github.com/nv-tlabs/kimodo), Apache-2.0).

- **Its frame of motion** (read from `kimodo/motion_rep/reps/kimodo_motionrep.py`
  at e1c219c): per frame, the smooth root (3), the heading (cos, sin), the 30
  joints' positions relative to the root (30×3), their global rotations as 6D
  (30×6), their velocities (30×3) and four foot contacts (left heel, left
  toe, right heel, right toe): 369 numbers. 30 fps, at most 300 frames.
- **Its skeleton**: SOMA's 30 joints (`somaskel30`: hips, two spine joints,
  chest, two neck joints, head, jaw, the eyes, each arm's shoulder, arm,
  forearm, hand, thumb end and middle-finger end, each leg's hip, knee, ankle
  and toe base), expanded to 77 joints with relaxed hands for export. Every
  training clip is on one body proportion. Measured here: +Y up, facing +Z,
  the left at +X, the game's own body frame (`entities::body`).
- **The text encoder**: LLM2Vec on Meta Llama-3-8B-Instruct (McGill's MNTP
  and supervised adapters, MIT), mean-pooled to 4096 numbers. Meta's base
  model is gated on Hugging Face; LocalAI publishes the merged encoder as
  GGUF ([`LocalAI-io/Llama-3-Kimodo-GGML`](https://huggingface.co/LocalAI-io/Llama-3-Kimodo-GGML):
  BF16 15.2 GB, Q8_0 8.1 GB recommended, smaller ones experimental), with the
  Llama 3 licence and "Built with Meta Llama 3".
- **What it cannot do** (its model card): non-physical or cartoon motion,
  awareness of objects in the scene; prompts are not always followed; feet
  can skate. It is best at locomotion, gestures and everyday actions.

### Its checkpoints and licences

| Checkpoint | Trained on | Weights' licence |
|---|---|---|
| Kimodo-SOMA-RP v1, v1.1, v1.2 (and ArdyRep) | Bones Rigplay 1 (proprietary) | NVIDIA Open Model License |
| **Kimodo-SOMA-SEED v1, v1.1** | BONES-SEED (public, 288 h of mocap) | NVIDIA Open Model License; the card: "ready for commercial use", uses include "Animations for game and media development" ([card](https://huggingface.co/nvidia/Kimodo-SOMA-SEED-v1.1)) |
| Kimodo-G1 RP, SEED | the Unitree G1 robot's skeleton | NVIDIA Open Model License |
| Kimodo-SMPLX-RP v1 | SMPL-X | NVIDIA R&D Model License (no commercial use) |

Not verified here: the BONES-SEED data licence itself is behind a login on
[its dataset page](https://huggingface.co/datasets/bones-studio/seed) (the
studio describes it as open for academic research and qualifying startups,
with commercial licences for more); NVIDIA's derived releases on it are CC BY
4.0. The weights' licence is what governs the model's outputs; before motion
from it ships in the game, read both licences whole.

## The ports to Apple Silicon

| Port | What it is | Licence | Used |
|---|---|---|---|
| [kimodo-mlx](https://github.com/NomaDamas/kimodo-mlx) (NomaDamas, read at bbb592a) | the two-stage denoiser, the DDIM sampler, separated CFG and the LLM2Vec encoder rewritten on MLX; SOMA-30 only; no constraints, no post-processing; decodes rotations and the root only; reads the encoder as per-layer GGUF files nobody publishes | MIT | **yes**, pinned, with what it lacks added in `lab/motion/kimodo_lab.py` |
| [kimodo-turbo](https://github.com/octaviusp/kimodo-turbo) (octaviusp, read at 8a8b3a0) | NVIDIA's own pipeline (constraints, multi-prompt, IK, foot clean-up, BVH/NPZ export) with only the denoising loop on MLX; a native Rust autorig for GLB humanoids; its own re-hosted encoder | **none stated** (no licence file) | no: without a licence nothing of it may be reused |
| [kimodo.cpp](https://github.com/localai-org/kimodo.cpp) (LocalAI) | Kimodo in C++ on GGML, CPU or Vulkan; the GGUF weights both MLX ports' encoders descend from | see its repository | no (kimodo-mlx carries a Metal patch for it) |

What kimodo-mlx lacked, and the lab added:

- **The encoder from one file.** LocalAI's GGUF holds every tensor under the
  same names kimodo-mlx reads per layer (`layer.NN.attn_q_proj_base.weight`,
  its LoRA `_a` and `_b`, the norms), the base weights Q8_0, the norms BF16,
  the adapters F32. `Encoder` reads them by name, dequantizes Q8_0 with
  gguf-py, and runs every prompt through one layer before loading the next,
  so only one layer (about 0.4 GB) is held at once.
- **Two rows of guidance, batched.** With nothing constrained, the
  constraint row of separated guidance equals the unconditioned one;
  dropping it saves a third, and every candidate seed of a brief is denoised
  in one batch.
- **The whole frame decoded**: joint positions and foot contacts as well as
  rotations, which the retarget and the clean-up need.

### Measured here (M4 Pro, 64 GB, MLX 0.32.4)

- The SEED checkpoint loads in 2.2 s; an unconditioned 4 s clip (120 frames)
  at 50 steps takes 11 s.
- The Smiler's ten briefs: ten prompts encoded in under 30 s (the Q8_0
  encoder, a layer at a time), then four candidates each at 100 steps,
  16–33 s a brief, batched; the whole run 251 s at 12.8 GB peak.
- The motion model alone is 1.13 GB; the Q8_0 encoder is 8.1 GB on disk.
- What it made, retargeted onto the Smiler and measured against the game's
  animation on the same path: [`../../lab/README.md`](../../lab/README.md), *Results*.

## Others considered

- **ARDY** (NVIDIA, 2026-07): real-time autoregressive text-to-motion, code
  Apache-2.0, weights under the NVIDIA Open Model License
  ([a summary](https://studio.aifilms.ai/blog/ardy-nvidia-real-time-motion-generation);
  Kimodo's repository already carries its motion representation,
  `ardy_motionrep.py`). The one to watch if motion is ever made in the game
  as it runs; no Apple Silicon port was found.
- **HY-Motion 1.0** (Tencent, 2025-12; a 1 B flow-matching DiT,
  [paper](https://arxiv.org/abs/2512.23464)): under the Hunyuan Community
  License (free commercial use except very large platforms, as reported by
  [CGWORLD](https://cgworld.jp/flashnews/01-202601-HY-Motion.html); its
  terms read differently in mirrors, unverified here); 24–26 GB of CUDA
  memory; no Mac port found.
- **The HumanML3D family** (MDM, MoMask, T2M-GPT and most academic
  text-to-motion): trained on AMASS and SMPL bodies, whose licences are for
  non-commercial research (not re-read in this session); excluded for a game
  that is sold.

## Retargeting and what the design asks

Kimodo knows people. The design's creatures move by two vocabularies only,
stillness and wrong mechanics (`../design/ENTITIES.md` §6), and the model card
says it cannot make non-physical motion. So the mocap gives the body its
weight and contact (what procedural animation cannot fake: the shift onto a
foot, the hips' drop at each step, the arms' lag), and the wrongness is laid
on top by rule: the head first and the body after, held frames and snaps,
the stoop kept, dead arms. How, and what the numbers show against the game's
animation, is in [`../../lab/README.md`](../../lab/README.md).

## Risks and what is open

- **Licences.** The SEED checkpoint's card allows commercial use; the
  BONES-SEED data licence was not readable here; the encoder is Llama 3's
  (an attribution line if it were ever distributed; it is not: only motion
  leaves the pipeline). Read both before anything ships.
- **Reproducibility.** Sampling on the GPU is seeded but not promised
  bit-identical across machines; the baked clips are committed
  (`lab/motion/clips/`), so nothing downstream depends on sampling again.
- **Skating and prompts.** Feet are planted by their contacts in the bake and
  locked in the world by the viewer; prompts that ask for odd motion get a
  person's nearest motion, so a brief asks for what a person does.
