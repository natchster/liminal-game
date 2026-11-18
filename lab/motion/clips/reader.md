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
