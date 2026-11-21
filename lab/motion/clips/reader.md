# The Reader: the clips

Made by `motion.py bake --creature reader` from `briefs/reader.toml` (Kimodo SOMA SEED v1.1 on MLX, kimodo-mlx).

| clip | seed | speed m/s | skate m/s | jitter | floor m | loop | score |
|---|---|---|---|---|---|---|---|
| reading | 4002 | 0.002 | 0.0075 | 6.6 | 0.0 | – | 0.11 |
| read_walk | 4003 | 0.807 | 0.0472 | 39.5 | 0.0 | 23–129 (0.0314) | 1.736 |
| read_still | 4003 | 0.001 | 0.0044 | 3.6 | 0.0 | 37–112 (0.0093) | 0.372 |
| pick_up | 4001 | 0.052 | 0.0194 | 15.3 | 0.0 | – | 0.28 |
| hurry | 4000 | 1.519 | 0.0676 | 100.1 | 0.0 | 15–112 (0.0239) | 1.993 |
| tear | 4002 | 0.003 | 0.0071 | 15.2 | 0.0 | – | 0.127 |

Candidates (lower is better: speed outside the brief ×3, foot skating / 0.08 m/s, jitter / 400 m/s³, below the floor / 2 cm; on all fours also the head too high and the hands too little on the floor):

- **reading** (the stance every clip is measured from (its rest pose: stooped over a sheet before its face)): “A person stands still, slightly hunched, holding a sheet of paper with both hands in front of their face and reading it.”; candidates 4002: 0.11, 4000: 0.126, 4003: 0.134, 4001: 0.148
- **read_walk** (Roam: it walks reading, a shuffle, the page before its face): “A person walks slowly forward while reading a sheet of paper held with both hands in front of their face.”; candidates 4003: 1.736, 4001: 2.4, 4002: 2.776, 4005: 2.883, 4000: 3.415, 4004: 4.208
- **read_still** (Read: it stands and reads the sheet to the end (3 s)): “A person stands still reading a sheet of paper held with both hands close to their face, moving their head slightly as they read.”; candidates 4003: 0.372, 4000: 0.464, 4001: 0.837, 4002: 0.888
- **pick_up** (Read, its first moment: it stoops for the sheet): “A heavy person slowly bends down, picks up a sheet of paper from the floor with their right hand, and stands back up.”; candidates 4001: 0.28, 4002: 0.282, 4003: 0.292, 4000: 0.308
- **hurry** (Hunt: it comes at 4.8 m/s, quick soft steps, the sheet snapped flat before it): “A heavy person hurries forward with quick, heavy steps, holding a sheet of paper flat in front of them with both hands.”; candidates 4000: 1.993, 4004: 2.387, 4002: 2.954, 4003: 3.134, 4001: 3.224, 4005: 4.137
- **tear** (What it learns: it tears the sheet in half unread (1 s)): “A person holds a sheet of paper in front of them with both hands and tears it in half.”; candidates 4002: 0.127, 4003: 0.145, 4001: 0.158, 4000: 0.186

## Bridges

This bake's: MotionBricks SOMA on MLX (natchster/motionbricks-mlx@d17bd6c7d8e5), nvidia/MotionBricks@3cb362d9b95a.

Made by MotionBricks (SOMA, the NVIDIA Open Model Agreement; `bricks.py`: the MLX port on Apple Silicon, NVIDIA's ONNX graph on CUDA on Linux) from the four frames of a clip that lead to where it leaves to the four of the clip it goes into, placed `ahead` m in front: the motion between, its ends then moved onto the frames it joins (`fix`, m: how far its body's joints missed the poses it was given, and the most any joint was moved onto the clips' frames: the hands, knees and feet a clip plants are held where they touch by the game's locks). One whose body missed by more than 8 cm, or any joint by more than 30 cm, is left to the blend (listed at the end). The game plays one where its creature goes from the one clip to the other, from the one nearest where it is in the first.

| from → to | leaves → lands | frames | ahead m | travel m | fix, body m | fix, any joint m | skate m/s |
|---|---|---|---|---|---|---|---|
| read_still → read_walk | 0.00 → 0.00 s | 22 | 0.558 | 0.592 | 0.079 | 0.029 | 0.0222 |
| read_still → read_walk | 0.63 → 0.00 s | 22 | 0.558 | 0.593 | 0.08 | 0.029 | 0.0222 |
| read_still → read_walk | 1.27 → 0.00 s | 22 | 0.558 | 0.596 | 0.078 | 0.028 | 0.018 |
| read_still → hurry | 0.00 → 0.00 s | 22 | 0.897 | 0.879 | 0.057 | 0.053 | 0.0045 |
| read_still → hurry | 0.63 → 0.00 s | 22 | 0.897 | 0.88 | 0.056 | 0.053 | 0.0041 |
| read_still → hurry | 1.27 → 0.00 s | 22 | 0.897 | 0.884 | 0.057 | 0.052 | 0.0051 |
| read_still → hurry | 1.87 → 0.00 s | 22 | 0.897 | 0.882 | 0.057 | 0.045 | 0.0044 |
| read_walk → read_still | 0.00 → 0.00 s | 22 | 0.066 | 0.035 | 0.052 | 0.18 | 0.0569 |
| read_walk → read_still | 0.27 → 0.00 s | 22 | 0.065 | 0.085 | 0.046 | 0.109 | 0.0059 |
| read_walk → read_still | 0.50 → 0.00 s | 22 | 0.063 | 0.091 | 0.05 | 0.093 | 0.0127 |
| read_walk → read_still | 0.77 → 0.00 s | 22 | 0.057 | 0.085 | 0.041 | 0.185 | 0.08 |
| read_walk → read_still | 1.00 → 0.00 s | 22 | 0.054 | 0.087 | 0.039 | 0.101 | 0.0134 |
| read_walk → read_still | 1.27 → 0.00 s | 22 | 0.052 | 0.074 | 0.046 | 0.084 | 0.0044 |
| read_walk → read_still | 1.50 → 0.00 s | 22 | 0.053 | 0.057 | 0.042 | 0.099 | 0.0538 |
| read_walk → read_still | 1.77 → 0.00 s | 22 | 0.053 | 0.061 | 0.043 | 0.179 | 0.0068 |
| read_walk → read_still | 2.03 → 0.00 s | 22 | 0.048 | 0.087 | 0.049 | 0.107 | 0.0072 |
| read_walk → read_still | 2.27 → 0.00 s | 22 | 0.046 | 0.08 | 0.056 | 0.116 | 0.0199 |
| read_walk → read_still | 2.53 → 0.00 s | 22 | 0.048 | 0.065 | 0.033 | 0.184 | 0.0609 |
| read_walk → read_still | 2.77 → 0.00 s | 22 | 0.048 | 0.064 | 0.034 | 0.119 | 0.0145 |
| read_walk → read_still | 3.03 → 0.00 s | 22 | 0.05 | 0.052 | 0.05 | 0.094 | 0.0052 |
| read_walk → read_still | 3.27 → 0.00 s | 22 | 0.054 | 0.029 | 0.046 | 0.143 | 0.0528 |
| read_walk → hurry | 0.00 → 0.00 s | 22 | 1.065 | 1.013 | 0.041 | 0.01 | 0.0212 |
| read_walk → hurry | 0.27 → 0.00 s | 22 | 1.064 | 1.054 | 0.056 | 0.008 | 0.0067 |
| read_walk → hurry | 0.50 → 0.00 s | 22 | 1.062 | 1.052 | 0.054 | 0.031 | 0.0059 |
| read_walk → hurry | 0.77 → 0.00 s | 22 | 1.057 | 1.059 | 0.04 | 0.012 | 0.0223 |
| read_walk → hurry | 1.00 → 0.00 s | 22 | 1.055 | 1.068 | 0.049 | 0.012 | 0.0242 |
| read_walk → hurry | 1.27 → 0.00 s | 22 | 1.052 | 1.042 | 0.054 | 0.007 | 0.0229 |
| read_walk → hurry | 1.50 → 0.00 s | 22 | 1.053 | 1.02 | 0.035 | 0.009 | 0.023 |
| read_walk → hurry | 1.77 → 0.00 s | 22 | 1.053 | 1.043 | 0.058 | 0.022 | 0.0101 |
| read_walk → hurry | 2.03 → 0.00 s | 22 | 1.048 | 1.051 | 0.068 | 0.01 | 0.0066 |
| read_walk → hurry | 2.27 → 0.00 s | 22 | 1.046 | 1.031 | 0.063 | 0.011 | 0.0064 |
| read_walk → hurry | 2.53 → 0.00 s | 22 | 1.048 | 1.027 | 0.048 | 0.01 | 0.0495 |
| read_walk → hurry | 2.77 → 0.00 s | 22 | 1.049 | 1.045 | 0.045 | 0.015 | 0.0316 |
| read_walk → hurry | 3.03 → 0.00 s | 22 | 1.05 | 1.03 | 0.048 | 0.009 | 0.0191 |
| read_walk → hurry | 3.27 → 0.00 s | 22 | 1.054 | 1.0 | 0.039 | 0.022 | 0.0225 |
| hurry → read_still | 0.00 → 0.00 s | 22 | 0.207 | 0.226 | 0.047 | 0.078 | 0.0061 |
| hurry → read_still | 0.23 → 0.00 s | 22 | 0.221 | 0.225 | 0.043 | 0.053 | 0.0214 |
| hurry → read_still | 0.50 → 0.00 s | 22 | 0.214 | 0.25 | 0.037 | 0.068 | 0.0107 |
| hurry → read_still | 0.73 → 0.00 s | 22 | 0.211 | 0.208 | 0.037 | 0.053 | 0.0411 |
| hurry → read_still | 1.00 → 0.00 s | 22 | 0.217 | 0.235 | 0.048 | 0.092 | 0.0134 |
| hurry → read_still | 1.23 → 0.00 s | 22 | 0.213 | 0.181 | 0.032 | 0.05 | 0.0271 |
| hurry → read_still | 1.50 → 0.00 s | 22 | 0.207 | 0.173 | 0.046 | 0.11 | 0.0047 |
| hurry → read_still | 1.73 → 0.00 s | 22 | 0.2 | 0.141 | 0.035 | 0.049 | 0.0286 |
| hurry → read_still | 2.00 → 0.00 s | 22 | 0.207 | 0.147 | 0.056 | 0.135 | 0.0171 |
| hurry → read_still | 2.23 → 0.00 s | 22 | 0.195 | 0.144 | 0.046 | 0.12 | 0.0172 |
| hurry → read_still | 2.50 → 0.00 s | 22 | 0.193 | 0.133 | 0.05 | 0.153 | 0.0608 |
| hurry → read_still | 2.73 → 0.00 s | 22 | 0.182 | 0.179 | 0.042 | 0.078 | 0.0064 |
| hurry → read_still | 3.00 → 0.00 s | 22 | 0.179 | 0.169 | 0.042 | 0.16 | 0.0746 |
| hurry → read_walk | 0.00 → 0.00 s | 22 | 0.663 | 0.715 | 0.027 | 0.102 | 0.0 |
| hurry → read_walk | 0.23 → 0.00 s | 22 | 0.669 | 0.707 | 0.036 | 0.027 | 0.026 |
| hurry → read_walk | 0.50 → 0.00 s | 22 | 0.666 | 0.736 | 0.031 | 0.031 | 0.0079 |
| hurry → read_walk | 0.73 → 0.00 s | 22 | 0.665 | 0.696 | 0.046 | 0.05 | 0.0082 |
| hurry → read_walk | 1.00 → 0.00 s | 22 | 0.668 | 0.717 | 0.025 | 0.062 | 0.0 |
| hurry → read_walk | 1.23 → 0.00 s | 22 | 0.666 | 0.664 | 0.048 | 0.299 | 0.0248 |
| hurry → read_walk | 1.50 → 0.00 s | 22 | 0.663 | 0.663 | 0.027 | 0.024 | 0.0126 |
| hurry → read_walk | 1.73 → 0.00 s | 22 | 0.66 | 0.634 | 0.045 | 0.034 | 0.0051 |
| hurry → read_walk | 2.00 → 0.00 s | 22 | 0.663 | 0.636 | 0.036 | 0.043 | 0.0028 |
| hurry → read_walk | 2.23 → 0.00 s | 22 | 0.658 | 0.639 | 0.026 | 0.026 | 0.0373 |
| hurry → read_walk | 2.50 → 0.00 s | 22 | 0.657 | 0.631 | 0.036 | 0.033 | 0.0306 |
| hurry → read_walk | 2.73 → 0.00 s | 22 | 0.653 | 0.676 | 0.037 | 0.025 | 0.0044 |
| hurry → read_walk | 3.00 → 0.00 s | 22 | 0.651 | 0.671 | 0.043 | 0.054 | 0.0157 |

Left to the blend (1):

- read_still>read_walk@56: its ends moved 0.028 m (its body 0.081 m)
