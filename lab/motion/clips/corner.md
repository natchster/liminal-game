# The one in the corner: the clips

Made by `motion.py bake --creature corner` from `briefs/corner.toml` (Kimodo SOMA SEED v1.1 on MLX, kimodo-mlx).

| clip | seed | speed m/s | skate m/s | jitter | floor m | loop | score |
|---|---|---|---|---|---|---|---|
| stand | 1001 | 0.005 | 0.0054 | 3.4 | 0.0 | – | 0.076 |
| hold | 3001 | 0.001 | 0.0032 | 2.8 | 0.0 | 64–190 (0.0012) | 0.088 |
| run | 3003 | 1.002 | 0.0889 | 227.5 | 0.0 | 49–140 (0.0542) | 5.484 |
| grab | 3000 | 1.734 | 0.0988 | 205.8 | 0.0 | – | 1.749 |
| walk_back | 3001 | 1.033 | 0.0466 | 64.1 | 0.0 | 59–172 (0.0293) | 1.72 |

Candidates (lower is better: speed outside the brief ×3, foot skating / 0.08 m/s, jitter / 400 m/s³, below the floor / 2 cm; on all fours also the head too high and the hands too little on the floor):

- **stand** (the stance every clip is measured from (its rest pose: upright, arms at its sides)): “A person stands upright and completely still, with arms relaxed at their sides, looking straight ahead.”; candidates 1001: 0.076, 1003: 0.09, 1000: 0.096, 1002: 0.11
- **hold** (Hold: in its corner, as still as a person can stand; only its breath): “A person stands upright and completely still facing a wall, head slightly bowed, arms straight down at their sides.”; candidates 3001: 0.088, 3002: 0.229, 3003: 0.24, 3000: 0.479
- **run** (Hunt: it comes at you at 5.6 m/s, screaming, its arms dead at its sides): “A heavy person runs forward with heavy, pounding strides and an upright posture, arms hanging straight down at their sides.”; candidates 3003: 5.484, 3000: 5.809, 3001: 5.861, 3006: 6.097, 3002: 6.161, 3004: 6.225, 3007: 6.861, 3005: 7.174
- **grab** (Strike: it reaches you): “A heavy person runs forward, stops heavily, and grabs at something in front of them with both hands.”; candidates 3000: 1.749, 3002: 2.127, 3003: 2.401, 3001: 2.826
- **walk_back** (Withdraw: back to its corner, stiffly): “A person walks forward slowly and stiffly with arms straight down at their sides.”; candidates 3001: 1.72, 3002: 2.283, 3000: 2.551, 3003: 3.707

## Bridges

This bake's: MotionBricks SOMA on MLX (natchster/motionbricks-mlx@d17bd6c7d8e5), nvidia/MotionBricks@3cb362d9b95a.

Made by MotionBricks (SOMA, the NVIDIA Open Model Agreement; `bricks.py`: the MLX port on Apple Silicon, NVIDIA's ONNX graph on CUDA on Linux) from the four frames of a clip that lead to where it leaves to the four of the clip it goes into, placed `ahead` m in front: the motion between, its ends then moved onto the frames it joins (`fix`, m: how far its body's joints missed the poses it was given, and the most any joint was moved onto the clips' frames: the hands, knees and feet a clip plants are held where they touch by the game's locks). One whose body missed by more than 8 cm, or any joint by more than 30 cm, is left to the blend (listed at the end). The game plays one where its creature goes from the one clip to the other, from the one nearest where it is in the first.

| from → to | leaves → lands | frames | ahead m | travel m | fix, body m | fix, any joint m | skate m/s |
|---|---|---|---|---|---|---|---|
| hold → run | 0.00 → 0.00 s | 22 | 0.661 | 0.689 | 0.044 | 0.038 | 0.006 |
| hold → run | 0.60 → 0.00 s | 22 | 0.661 | 0.689 | 0.047 | 0.039 | 0.0063 |
| hold → run | 1.20 → 0.00 s | 22 | 0.661 | 0.686 | 0.045 | 0.034 | 0.0072 |
| hold → run | 1.80 → 0.00 s | 22 | 0.661 | 0.684 | 0.045 | 0.036 | 0.0072 |
| hold → run | 2.40 → 0.00 s | 22 | 0.661 | 0.687 | 0.048 | 0.04 | 0.0073 |
| hold → run | 3.00 → 0.00 s | 22 | 0.661 | 0.686 | 0.052 | 0.04 | 0.0071 |
| hold → run | 3.60 → 0.00 s | 22 | 0.662 | 0.689 | 0.05 | 0.04 | 0.0081 |
| hold → walk_back | 0.00 → 0.00 s | 22 | 0.707 | 0.686 | 0.064 | 0.038 | 0.009 |
| hold → walk_back | 0.60 → 0.00 s | 22 | 0.707 | 0.686 | 0.065 | 0.037 | 0.0091 |
| hold → walk_back | 1.20 → 0.00 s | 22 | 0.707 | 0.683 | 0.063 | 0.033 | 0.0091 |
| hold → walk_back | 1.80 → 0.00 s | 22 | 0.707 | 0.681 | 0.066 | 0.031 | 0.0088 |
| hold → walk_back | 2.40 → 0.00 s | 22 | 0.707 | 0.684 | 0.067 | 0.033 | 0.0088 |
| hold → walk_back | 3.00 → 0.00 s | 22 | 0.707 | 0.683 | 0.067 | 0.039 | 0.0088 |
| hold → walk_back | 3.60 → 0.00 s | 22 | 0.707 | 0.686 | 0.067 | 0.038 | 0.0088 |
| run → hold | 0.00 → 0.00 s | 22 | 0.098 | 0.076 | 0.043 | 0.022 | 0.0195 |
| run → hold | 0.27 → 0.00 s | 22 | 0.108 | 0.091 | 0.038 | 0.028 | 0.0228 |
| run → hold | 0.50 → 0.00 s | 22 | 0.115 | 0.103 | 0.032 | 0.077 | 0.004 |
| run → hold | 0.77 → 0.00 s | 22 | 0.118 | 0.104 | 0.032 | 0.036 | 0.0214 |
| run → hold | 1.00 → 0.00 s | 22 | 0.123 | 0.095 | 0.049 | 0.03 | 0.0258 |
| run → hold | 1.27 → 0.00 s | 22 | 0.12 | 0.091 | 0.037 | 0.04 | 0.0295 |
| run → hold | 1.53 → 0.00 s | 22 | 0.13 | 0.109 | 0.038 | 0.104 | 0.0044 |
| run → hold | 1.77 → 0.00 s | 22 | 0.118 | 0.119 | 0.036 | 0.033 | 0.0312 |
| run → hold | 2.03 → 0.00 s | 22 | 0.132 | 0.142 | 0.038 | 0.041 | 0.0288 |
| run → hold | 2.27 → 0.00 s | 22 | 0.144 | 0.163 | 0.045 | 0.122 | 0.0061 |
| run → hold | 2.53 → 0.00 s | 22 | 0.134 | 0.145 | 0.031 | 0.098 | 0.02 |
| run → hold | 2.77 → 0.00 s | 22 | 0.136 | 0.141 | 0.037 | 0.037 | 0.0332 |
| run → walk_back | 0.00 → 0.00 s | 22 | 0.821 | 0.774 | 0.022 | 0.012 | 0.0378 |
| run → walk_back | 0.27 → 0.00 s | 22 | 0.822 | 0.781 | 0.015 | 0.009 | 0.0583 |
| run → walk_back | 0.50 → 0.00 s | 22 | 0.822 | 0.787 | 0.021 | 0.009 | 0.0273 |
| run → walk_back | 0.77 → 0.00 s | 22 | 0.822 | 0.786 | 0.028 | 0.057 | 0.0441 |
| run → walk_back | 1.00 → 0.00 s | 22 | 0.822 | 0.771 | 0.038 | 0.009 | 0.062 |
| run → walk_back | 1.27 → 0.00 s | 22 | 0.822 | 0.768 | 0.028 | 0.01 | 0.0478 |
| run → walk_back | 1.53 → 0.00 s | 22 | 0.822 | 0.774 | 0.026 | 0.011 | 0.0579 |
| run → walk_back | 1.77 → 0.00 s | 22 | 0.822 | 0.799 | 0.02 | 0.011 | 0.051 |
| run → walk_back | 2.03 → 0.00 s | 22 | 0.822 | 0.811 | 0.016 | 0.009 | 0.0469 |
| run → walk_back | 2.27 → 0.00 s | 22 | 0.823 | 0.817 | 0.023 | 0.009 | 0.0352 |
| run → walk_back | 2.53 → 0.00 s | 22 | 0.822 | 0.811 | 0.042 | 0.061 | 0.0333 |
| run → walk_back | 2.77 → 0.00 s | 22 | 0.823 | 0.807 | 0.026 | 0.037 | 0.0461 |
| walk_back → hold | 0.00 → 0.00 s | 22 | 0.115 | 0.136 | 0.05 | 0.129 | 0.0073 |
| walk_back → hold | 0.27 → 0.00 s | 22 | 0.119 | 0.147 | 0.03 | 0.103 | 0.0048 |
| walk_back → hold | 0.50 → 0.00 s | 22 | 0.128 | 0.139 | 0.042 | 0.093 | 0.0582 |
| walk_back → hold | 0.77 → 0.00 s | 22 | 0.134 | 0.161 | 0.026 | 0.051 | 0.009 |
| walk_back → hold | 1.00 → 0.00 s | 22 | 0.142 | 0.138 | 0.035 | 0.12 | 0.0425 |
| walk_back → hold | 1.27 → 0.00 s | 22 | 0.14 | 0.157 | 0.049 | 0.102 | 0.0089 |
| walk_back → hold | 1.50 → 0.00 s | 22 | 0.141 | 0.135 | 0.029 | 0.109 | 0.0036 |
| walk_back → hold | 1.77 → 0.00 s | 22 | 0.137 | 0.107 | 0.044 | 0.152 | 0.065 |
| walk_back → hold | 2.00 → 0.00 s | 22 | 0.131 | 0.121 | 0.028 | 0.045 | 0.007 |
| walk_back → hold | 2.27 → 0.00 s | 22 | 0.128 | 0.088 | 0.035 | 0.18 | 0.0777 |
| walk_back → hold | 2.50 → 0.00 s | 22 | 0.128 | 0.114 | 0.051 | 0.119 | 0.0092 |
| walk_back → hold | 2.77 → 0.00 s | 22 | 0.127 | 0.096 | 0.031 | 0.106 | 0.0048 |
| walk_back → hold | 3.00 → 0.00 s | 22 | 0.126 | 0.08 | 0.046 | 0.098 | 0.0604 |
| walk_back → hold | 3.27 → 0.00 s | 22 | 0.127 | 0.115 | 0.029 | 0.059 | 0.0046 |
| walk_back → hold | 3.50 → 0.00 s | 22 | 0.13 | 0.1 | 0.031 | 0.141 | 0.0629 |
| walk_back → run | 0.00 → 0.00 s | 22 | 0.76 | 0.808 | 0.038 | 0.043 | 0.0 |
| walk_back → run | 0.27 → 0.00 s | 22 | 0.76 | 0.814 | 0.02 | 0.007 | 0.0127 |
| walk_back → run | 0.50 → 0.00 s | 22 | 0.761 | 0.799 | 0.025 | 0.011 | 0.0377 |
| walk_back → run | 0.77 → 0.00 s | 22 | 0.762 | 0.814 | 0.013 | 0.008 | 0.0062 |
| walk_back → run | 1.00 → 0.00 s | 22 | 0.763 | 0.785 | 0.018 | 0.006 | 0.0135 |
| walk_back → run | 1.27 → 0.00 s | 22 | 0.763 | 0.808 | 0.022 | 0.008 | 0.0 |
| walk_back → run | 1.50 → 0.00 s | 22 | 0.763 | 0.784 | 0.026 | 0.003 | 0.016 |
| walk_back → run | 1.77 → 0.00 s | 22 | 0.763 | 0.759 | 0.02 | 0.012 | 0.0276 |
| walk_back → run | 2.00 → 0.00 s | 22 | 0.762 | 0.772 | 0.014 | 0.006 | 0.0048 |
| walk_back → run | 2.27 → 0.00 s | 22 | 0.761 | 0.742 | 0.019 | 0.003 | 0.0134 |
| walk_back → run | 2.50 → 0.00 s | 22 | 0.761 | 0.775 | 0.021 | 0.013 | 0.0 |
| walk_back → run | 2.77 → 0.00 s | 22 | 0.761 | 0.755 | 0.025 | 0.004 | 0.0145 |
| walk_back → run | 3.00 → 0.00 s | 22 | 0.761 | 0.74 | 0.024 | 0.011 | 0.0379 |
| walk_back → run | 3.27 → 0.00 s | 22 | 0.761 | 0.774 | 0.018 | 0.009 | 0.0039 |
| walk_back → run | 3.50 → 0.00 s | 22 | 0.762 | 0.757 | 0.017 | 0.013 | 0.0105 |
| hold → grab | 0.00 → 0.50 s | 22 | 1.368 | 1.399 | 0.065 | 0.062 | 0.006 |
| hold → grab | 0.60 → 0.50 s | 22 | 1.368 | 1.399 | 0.068 | 0.065 | 0.0041 |
| hold → grab | 1.20 → 0.50 s | 22 | 1.368 | 1.397 | 0.068 | 0.067 | 0.0025 |
| hold → grab | 1.80 → 0.50 s | 22 | 1.368 | 1.395 | 0.067 | 0.068 | 0.0023 |
| hold → grab | 2.40 → 0.50 s | 22 | 1.368 | 1.398 | 0.066 | 0.064 | 0.0049 |
| hold → grab | 3.00 → 0.50 s | 22 | 1.368 | 1.396 | 0.068 | 0.065 | 0.0028 |
| hold → grab | 3.60 → 0.50 s | 22 | 1.368 | 1.399 | 0.066 | 0.065 | 0.0019 |
| run → grab | 0.50 → 0.50 s | 22 | 1.858 | 1.875 | 0.079 | 0.017 | 0.0304 |
| run → grab | 1.27 → 0.50 s | 22 | 1.866 | 1.864 | 0.078 | 0.012 | 0.0232 |
| run → grab | 1.53 → 0.50 s | 22 | 1.881 | 1.883 | 0.074 | 0.014 | 0.0091 |
| grab → hold | 1.63 → 0.00 s | 22 | 0.245 | 0.24 | 0.072 | 0.076 | 0.0423 |
| grab → hold | 2.30 → 0.00 s | 22 | 0.006 | 0.05 | 0.023 | 0.051 | 0.0 |
| grab → hold | 2.97 → 0.00 s | 22 | 0.001 | 0.047 | 0.009 | 0.043 | 0.0 |
| grab → run | 0.30 → 0.00 s | 22 | 1.052 | 1.055 | 0.063 | 0.02 | 0.0579 |
| grab → run | 0.97 → 0.00 s | 22 | 1.08 | 1.045 | 0.06 | 0.021 | 0.0374 |
| grab → run | 1.63 → 0.00 s | 22 | 0.792 | 0.814 | 0.043 | 0.008 | 0.0128 |
| grab → run | 2.30 → 0.00 s | 22 | 0.704 | 0.681 | 0.045 | 0.026 | 0.0276 |
| grab → run | 2.97 → 0.00 s | 22 | 0.67 | 0.651 | 0.044 | 0.059 | 0.0121 |
| grab → walk_back | 0.30 → 0.00 s | 22 | 1.087 | 1.042 | 0.061 | 0.103 | 0.0902 |
| grab → walk_back | 0.97 → 0.00 s | 22 | 1.114 | 1.03 | 0.048 | 0.171 | 0.0886 |
| grab → walk_back | 1.63 → 0.00 s | 22 | 0.846 | 0.819 | 0.049 | 0.011 | 0.0383 |
| grab → walk_back | 2.30 → 0.00 s | 22 | 0.753 | 0.681 | 0.051 | 0.016 | 0.0135 |
| grab → walk_back | 2.97 → 0.00 s | 22 | 0.717 | 0.649 | 0.054 | 0.036 | 0.0095 |

Left to the blend (26):

- run>grab@0: its ends moved 0.0 m (its body 0.114 m)
- run>grab@8: its ends moved 0.001 m (its body 0.082 m)
- run>grab@23: its ends moved 0.005 m (its body 0.101 m)
- run>grab@30: its ends moved 0.0 m (its body 0.111 m)
- run>grab@53: its ends moved 0.0 m (its body 0.105 m)
- run>grab@61: its ends moved 0.002 m (its body 0.094 m)
- run>grab@68: its ends moved 0.008 m (its body 0.081 m)
- run>grab@76: its ends moved 0.01 m (its body 0.098 m)
- run>grab@83: its ends moved 0.037 m (its body 0.12 m)
- walk_back>grab@0: its ends moved 0.051 m (its body 0.091 m)
- walk_back>grab@8: its ends moved 0.023 m (its body 0.125 m)
- walk_back>grab@15: its ends moved 0.006 m (its body 0.093 m)
- walk_back>grab@23: its ends moved 0.009 m (its body 0.111 m)
- walk_back>grab@30: its ends moved 0.005 m (its body 0.101 m)
- walk_back>grab@38: its ends moved 0.049 m (its body 0.105 m)
- walk_back>grab@45: its ends moved 0.025 m (its body 0.127 m)
- walk_back>grab@53: its ends moved 0.056 m (its body 0.104 m)
- walk_back>grab@60: its ends moved 0.011 m (its body 0.114 m)
- walk_back>grab@68: its ends moved 0.006 m (its body 0.093 m)
- walk_back>grab@75: its ends moved 0.047 m (its body 0.093 m)
- walk_back>grab@83: its ends moved 0.027 m (its body 0.123 m)
- walk_back>grab@90: its ends moved 0.013 m (its body 0.091 m)
- walk_back>grab@98: its ends moved 0.013 m (its body 0.114 m)
- walk_back>grab@105: its ends moved 0.011 m (its body 0.101 m)
- grab>hold@9: its ends moved 0.355 m (its body 0.052 m)
- grab>hold@29: its ends moved 0.328 m (its body 0.065 m)
