# The one behind the glass: the clips

Made by `motion.py bake --creature glass` from `briefs/glass.toml` (Kimodo SOMA SEED v1.1 on MLX, kimodo-mlx).

| clip | seed | speed m/s | skate m/s | jitter | floor m | loop | score |
|---|---|---|---|---|---|---|---|
| stand | 1001 | 0.005 | 0.0054 | 3.4 | 0.0 | – | 0.076 |
| still | 5000 | 0.001 | 0.0032 | 2.8 | 0.0 | 20–217 (0.0008) | 0.074 |
| walk | 5001 | 1.094 | 0.0499 | 77.6 | 0.0 | 21–172 (0.0251) | 1.653 |
| reach_walk | 5003 | 0.78 | 0.0421 | 45.7 | 0.0 | 36–126 (0.0206) | 1.327 |
| climb | 5001 | 0.094 | 0.0279 | 18.6 | 0.0 | – | 0.41 |

Candidates (lower is better: speed outside the brief ×3, foot skating / 0.08 m/s, jitter / 400 m/s³, below the floor / 2 cm; on all fours also the head too high and the hands too little on the floor):

- **stand** (the stance every clip is measured from (its rest pose: upright, arms at its sides)): “A person stands upright and completely still, with arms relaxed at their sides, looking straight ahead.”; candidates 1001: 0.076, 1003: 0.09, 1000: 0.096, 1002: 0.11
- **still** (Hold and Alert: far out on the track, facing you; it does not move): “A person stands upright and completely still facing forward, arms at their sides, staring straight ahead.”; candidates 5000: 0.074, 5002: 0.109, 5003: 0.528, 5001: 0.895
- **walk** (Stalk: it comes up the track at 2.6 m/s while your eyes are on its glass): “A heavy person walks forward at a steady, even pace with an upright posture and arms held still at their sides, placing each foot down heavily.”; candidates 5001: 1.653, 5004: 2.051, 5005: 2.237, 5002: 2.354, 5003: 2.599, 5000: 3.031
- **reach_walk** (Hunt, inside: to the edge of its light nearest you, at 1.8 m/s, its arms out toward you): “A heavy person walks slowly forward with an upright posture and both arms reaching out in front of them.”; candidates 5003: 1.327, 5002: 1.338, 5000: 1.964, 5001: 2.284
- **climb** (Hunt: through the window, bent low over the sill (2.4 s)): “A heavy person crouches down low, steps forward through a low opening, and slowly stands back up.”; candidates 5001: 0.41, 5003: 0.44, 5002: 0.454, 5000: 0.479

## Bridges

This bake's: MotionBricks SOMA on MLX (natchster/motionbricks-mlx@d17bd6c7d8e5), nvidia/MotionBricks@3cb362d9b95a.

Made by MotionBricks (SOMA, the NVIDIA Open Model Agreement; `bricks.py`: the MLX port on Apple Silicon, NVIDIA's ONNX graph on CUDA on Linux) from the four frames of a clip that lead to where it leaves to the four of the clip it goes into, placed `ahead` m in front: the motion between, its ends then moved onto the frames it joins (`fix`, m: how far its body's joints missed the poses it was given, and the most any joint was moved onto the clips' frames: the hands, knees and feet a clip plants are held where they touch by the game's locks). One whose body missed by more than 8 cm, or any joint by more than 30 cm, is left to the blend (listed at the end). The game plays one where its creature goes from the one clip to the other, from the one nearest where it is in the first.

| from → to | leaves → lands | frames | ahead m | travel m | fix, body m | fix, any joint m | skate m/s |
|---|---|---|---|---|---|---|---|
| still → walk | 0.00 → 0.00 s | 22 | 0.621 | 0.629 | 0.052 | 0.008 | 0.0219 |
| still → walk | 0.60 → 0.00 s | 22 | 0.621 | 0.633 | 0.052 | 0.008 | 0.0141 |
| still → walk | 1.20 → 0.00 s | 22 | 0.621 | 0.633 | 0.053 | 0.009 | 0.0134 |
| still → walk | 1.80 → 0.00 s | 22 | 0.621 | 0.633 | 0.054 | 0.008 | 0.0134 |
| still → walk | 2.40 → 0.00 s | 22 | 0.621 | 0.632 | 0.052 | 0.007 | 0.0189 |
| still → walk | 3.00 → 0.00 s | 22 | 0.621 | 0.631 | 0.054 | 0.007 | 0.0135 |
| still → walk | 3.57 → 0.00 s | 22 | 0.621 | 0.63 | 0.051 | 0.007 | 0.0187 |
| still → walk | 4.17 → 0.00 s | 22 | 0.621 | 0.627 | 0.054 | 0.007 | 0.0235 |
| still → walk | 4.77 → 0.00 s | 22 | 0.621 | 0.628 | 0.053 | 0.007 | 0.0124 |
| still → walk | 5.37 → 0.00 s | 22 | 0.621 | 0.628 | 0.054 | 0.008 | 0.0214 |
| still → walk | 5.97 → 0.00 s | 22 | 0.621 | 0.629 | 0.053 | 0.01 | 0.0216 |
| still → reach_walk | 0.00 → 0.00 s | 22 | 0.517 | 0.557 | 0.07 | 0.027 | 0.0048 |
| still → reach_walk | 0.60 → 0.00 s | 22 | 0.517 | 0.56 | 0.075 | 0.025 | 0.0052 |
| still → reach_walk | 1.20 → 0.00 s | 22 | 0.517 | 0.561 | 0.07 | 0.027 | 0.0047 |
| still → reach_walk | 1.80 → 0.00 s | 22 | 0.517 | 0.56 | 0.07 | 0.026 | 0.0047 |
| still → reach_walk | 2.40 → 0.00 s | 22 | 0.517 | 0.56 | 0.069 | 0.026 | 0.0047 |
| still → reach_walk | 3.00 → 0.00 s | 22 | 0.517 | 0.559 | 0.07 | 0.027 | 0.0048 |
| still → reach_walk | 3.57 → 0.00 s | 22 | 0.517 | 0.557 | 0.07 | 0.027 | 0.0048 |
| still → reach_walk | 4.17 → 0.00 s | 22 | 0.517 | 0.555 | 0.07 | 0.027 | 0.0047 |
| still → reach_walk | 4.77 → 0.00 s | 22 | 0.517 | 0.556 | 0.069 | 0.027 | 0.0048 |
| still → reach_walk | 5.37 → 0.00 s | 22 | 0.517 | 0.555 | 0.07 | 0.026 | 0.0047 |
| still → reach_walk | 5.97 → 0.00 s | 22 | 0.517 | 0.556 | 0.069 | 0.026 | 0.0047 |
| walk → still | 0.00 → 0.00 s | 22 | 0.085 | 0.076 | 0.041 | 0.125 | 0.0131 |
| walk → still | 0.27 → 0.00 s | 22 | 0.091 | 0.105 | 0.032 | 0.049 | 0.0067 |
| walk → still | 0.50 → 0.00 s | 22 | 0.098 | 0.092 | 0.048 | 0.087 | 0.0628 |
| walk → still | 0.77 → 0.00 s | 22 | 0.102 | 0.14 | 0.035 | 0.033 | 0.012 |
| walk → still | 1.00 → 0.00 s | 22 | 0.105 | 0.117 | 0.03 | 0.128 | 0.0163 |
| walk → still | 1.27 → 0.00 s | 22 | 0.099 | 0.117 | 0.04 | 0.129 | 0.0081 |
| walk → still | 1.50 → 0.00 s | 22 | 0.101 | 0.119 | 0.026 | 0.031 | 0.006 |
| walk → still | 1.77 → 0.00 s | 22 | 0.098 | 0.062 | 0.034 | 0.088 | 0.0589 |
| walk → still | 2.00 → 0.00 s | 22 | 0.104 | 0.082 | 0.03 | 0.047 | 0.0108 |
| walk → still | 2.27 → 0.00 s | 22 | 0.104 | 0.055 | 0.03 | 0.12 | 0.0275 |
| walk → still | 2.53 → 0.00 s | 22 | 0.109 | 0.058 | 0.039 | 0.138 | 0.0158 |
| walk → still | 2.77 → 0.00 s | 22 | 0.112 | 0.081 | 0.031 | 0.036 | 0.0037 |
| walk → still | 3.03 → 0.00 s | 22 | 0.115 | 0.052 | 0.047 | 0.087 | 0.0562 |
| walk → still | 3.27 → 0.00 s | 22 | 0.115 | 0.09 | 0.037 | 0.036 | 0.0081 |
| walk → still | 3.53 → 0.00 s | 22 | 0.119 | 0.075 | 0.029 | 0.047 | 0.0501 |
| walk → still | 3.77 → 0.00 s | 22 | 0.114 | 0.067 | 0.041 | 0.134 | 0.0134 |
| walk → still | 4.03 → 0.00 s | 22 | 0.113 | 0.073 | 0.029 | 0.048 | 0.0059 |
| walk → still | 4.27 → 0.00 s | 22 | 0.116 | 0.054 | 0.047 | 0.091 | 0.0627 |
| walk → still | 4.53 → 0.00 s | 22 | 0.107 | 0.101 | 0.035 | 0.026 | 0.0086 |
| walk → still | 4.77 → 0.00 s | 22 | 0.111 | 0.085 | 0.029 | 0.126 | 0.0282 |
| walk → reach_walk | 0.00 → 0.00 s | 22 | 0.575 | 0.606 | 0.027 | 0.016 | 0.0152 |
| walk → reach_walk | 0.27 → 0.00 s | 22 | 0.577 | 0.62 | 0.026 | 0.029 | 0.0078 |
| walk → reach_walk | 0.50 → 0.00 s | 22 | 0.578 | 0.609 | 0.032 | 0.028 | 0.0033 |
| walk → reach_walk | 0.77 → 0.00 s | 22 | 0.579 | 0.651 | 0.021 | 0.018 | 0.0392 |
| walk → reach_walk | 1.00 → 0.00 s | 22 | 0.58 | 0.625 | 0.023 | 0.006 | 0.03 |
| walk → reach_walk | 1.27 → 0.00 s | 22 | 0.578 | 0.637 | 0.021 | 0.014 | 0.0049 |
| walk → reach_walk | 1.50 → 0.00 s | 22 | 0.579 | 0.631 | 0.032 | 0.028 | 0.0035 |
| walk → reach_walk | 1.77 → 0.00 s | 22 | 0.578 | 0.578 | 0.023 | 0.029 | 0.0035 |
| walk → reach_walk | 2.00 → 0.00 s | 22 | 0.58 | 0.591 | 0.031 | 0.019 | 0.0221 |
| walk → reach_walk | 2.27 → 0.00 s | 22 | 0.58 | 0.558 | 0.018 | 0.007 | 0.0294 |
| walk → reach_walk | 2.53 → 0.00 s | 22 | 0.581 | 0.57 | 0.014 | 0.013 | 0.0121 |
| walk → reach_walk | 2.77 → 0.00 s | 22 | 0.582 | 0.582 | 0.031 | 0.024 | 0.0093 |
| walk → reach_walk | 3.03 → 0.00 s | 22 | 0.583 | 0.558 | 0.032 | 0.024 | 0.0032 |
| walk → reach_walk | 3.27 → 0.00 s | 22 | 0.583 | 0.587 | 0.018 | 0.015 | 0.0417 |
| walk → reach_walk | 3.53 → 0.00 s | 22 | 0.584 | 0.565 | 0.014 | 0.006 | 0.0352 |
| walk → reach_walk | 3.77 → 0.00 s | 22 | 0.583 | 0.575 | 0.012 | 0.018 | 0.0077 |
| walk → reach_walk | 4.03 → 0.00 s | 22 | 0.582 | 0.577 | 0.022 | 0.037 | 0.0054 |
| walk → reach_walk | 4.27 → 0.00 s | 22 | 0.583 | 0.561 | 0.03 | 0.028 | 0.0035 |
| walk → reach_walk | 4.53 → 0.00 s | 22 | 0.581 | 0.599 | 0.032 | 0.017 | 0.0361 |
| walk → reach_walk | 4.77 → 0.00 s | 22 | 0.582 | 0.576 | 0.014 | 0.022 | 0.0296 |
| reach_walk → still | 0.00 → 0.00 s | 22 | 0.056 | 0.054 | 0.034 | 0.113 | 0.0232 |
| reach_walk → still | 0.27 → 0.00 s | 22 | 0.056 | 0.044 | 0.036 | 0.144 | 0.0068 |
| reach_walk → still | 0.50 → 0.00 s | 22 | 0.057 | 0.073 | 0.044 | 0.038 | 0.0055 |
| reach_walk → still | 0.73 → 0.00 s | 22 | 0.052 | 0.077 | 0.031 | 0.096 | 0.0091 |
| reach_walk → still | 1.00 → 0.00 s | 22 | 0.05 | 0.071 | 0.033 | 0.104 | 0.0156 |
| reach_walk → still | 1.27 → 0.00 s | 22 | 0.051 | 0.077 | 0.034 | 0.042 | 0.0043 |
| reach_walk → still | 1.50 → 0.00 s | 22 | 0.051 | 0.063 | 0.034 | 0.115 | 0.044 |
| reach_walk → still | 1.73 → 0.00 s | 22 | 0.05 | 0.057 | 0.032 | 0.135 | 0.0117 |
| reach_walk → still | 2.00 → 0.00 s | 22 | 0.051 | 0.079 | 0.039 | 0.027 | 0.0073 |
| reach_walk → still | 2.27 → 0.00 s | 22 | 0.05 | 0.067 | 0.026 | 0.098 | 0.0 |
| reach_walk → still | 2.50 → 0.00 s | 22 | 0.053 | 0.035 | 0.036 | 0.124 | 0.0153 |
| reach_walk → still | 2.73 → 0.00 s | 22 | 0.052 | 0.057 | 0.038 | 0.037 | 0.0067 |
| reach_walk → walk | 0.00 → 0.00 s | 22 | 0.702 | 0.675 | 0.038 | 0.033 | 0.0686 |
| reach_walk → walk | 0.27 → 0.00 s | 22 | 0.702 | 0.698 | 0.024 | 0.021 | 0.0135 |
| reach_walk → walk | 0.50 → 0.00 s | 22 | 0.702 | 0.705 | 0.024 | 0.029 | 0.0043 |
| reach_walk → walk | 0.73 → 0.00 s | 22 | 0.701 | 0.703 | 0.013 | 0.021 | 0.0122 |
| reach_walk → walk | 1.00 → 0.00 s | 22 | 0.7 | 0.73 | 0.03 | 0.028 | 0.0455 |
| reach_walk → walk | 1.27 → 0.00 s | 22 | 0.701 | 0.718 | 0.052 | 0.054 | 0.0306 |
| reach_walk → walk | 1.50 → 0.00 s | 22 | 0.701 | 0.7 | 0.038 | 0.025 | 0.0681 |
| reach_walk → walk | 1.73 → 0.00 s | 22 | 0.7 | 0.717 | 0.026 | 0.025 | 0.01 |
| reach_walk → walk | 2.00 → 0.00 s | 22 | 0.701 | 0.718 | 0.024 | 0.03 | 0.0112 |
| reach_walk → walk | 2.27 → 0.00 s | 22 | 0.7 | 0.692 | 0.013 | 0.025 | 0.0132 |
| reach_walk → walk | 2.50 → 0.00 s | 22 | 0.701 | 0.691 | 0.028 | 0.026 | 0.034 |
| reach_walk → walk | 2.73 → 0.00 s | 22 | 0.701 | 0.692 | 0.043 | 0.045 | 0.0301 |

Left to the blend: none.
