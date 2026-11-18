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
