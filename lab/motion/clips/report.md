# The Smiler's clips

Made by `motion.py bake` from `clips.toml` (Kimodo SOMA SEED v1.1 on MLX, kimodo-mlx).

| clip | seed | speed m/s | skate m/s | jitter | floor m | loop | score |
|---|---|---|---|---|---|---|---|
| stand | 1001 | 0.005 | 0.0054 | 3.4 | 0.0 | – | 0.076 |
| idle | 1001 | 0.021 | 0.008 | 4.6 | 0.0 | 79–226 (0.0045) | 0.261 |
| look | 1001 | 0.001 | 0.0049 | 4.0 | 0.0 | – | 0.071 |
| turn | 1003 | 0.133 | 0.0278 | 23.4 | 0.0 | – | 0.405 |
| creep | 1002 | 0.692 | 0.0405 | 36.1 | 0.0 | 31–137 (0.0138) | 1.056 |
| listen | 1003 | 0.097 | 0.0141 | 12.1 | 0.0 | – | 0.207 |
| stalk | 1000 | 1.894 | 0.1029 | 180.3 | 0.0 | 47–135 (0.036) | 2.938 |
| run | 1005 | 2.094 | 0.1002 | 282.7 | 0.0 | 10–145 (0.0795) | 5.408 |
| lunge | 1000 | 0.859 | 0.0701 | 102.3 | 0.0 | – | 1.132 |
| stand_over | 1003 | 0.014 | 0.0099 | 5.5 | 0.0 | – | 0.138 |
| back_away | 1001 | 1.075 | 0.0573 | 55.3 | 0.0 | – | 0.854 |

Candidates (lower is better: speed outside the brief ×3, foot skating / 0.08 m/s, jitter / 400 m/s³, below the floor / 2 cm):

- **stand** (the stance every clip is measured from (the body's rest pose is its stoop)): “A person stands upright and completely still, with arms relaxed at their sides, looking straight ahead.”; candidates 1001: 0.076, 1003: 0.09, 1000: 0.096, 1002: 0.11
- **idle** (Idle: in its dark, facing a corner; it barely moves, and the little it does is weight): “A person stands still with arms hanging limp, slowly shifting their weight from one foot to the other.”; candidates 1001: 0.261, 1003: 0.501, 1000: 1.343, 1002: 1.344
- **look** (Alert: it has seen a light, and its head goes to it before its body does): “A person stands still and turns their head sharply to the right to look at something, then holds it there.”; candidates 1001: 0.071, 1003: 0.082, 1002: 0.088, 1000: 0.103
- **turn** (Alert: the body following the head round): “A person standing still turns around in place to the left to face the other way.”; candidates 1003: 0.405, 1002: 0.406, 1001: 0.414, 1000: 0.435
- **creep** (Search: to where your light went out, slower, unsure where you are): “A person walks forward very slowly and carefully, with their arms hanging down.”; candidates 1002: 1.056, 1000: 1.33, 1003: 1.416, 1001: 1.864
- **listen** (Search, arriving: it stops where the light was and listens): “A person stops walking, tilts their head to one side as if listening, and stays completely still.”; candidates 1003: 0.207, 1000: 0.231, 1001: 0.244, 1002: 0.263
- **stalk** (Hunt, the approach: long strides, leaning in): “A person walks forward quickly with long strides, leaning forward.”; candidates 1000: 2.938, 1003: 3.056, 1002: 3.842, 1001: 5.039
- **run** (Hunt: you are lit and in its sight; it comes at 4.5 m/s, faster than your walk): “A person sprints forward as fast as they can.”; candidates 1005: 5.408, 1003: 6.281, 1001: 6.321, 1002: 7.613, 1000: 8.256, 1004: 8.316, 1007: 8.403, 1006: 9.041
- **lunge** (Strike: a sharp inhale, coiling as it comes, then the lunge (7 m/s over at most 4 m)): “A person bends their knees, then quickly steps forward and grabs at something in front of them with both hands.”; candidates 1000: 1.132, 1001: 1.364, 1003: 11.275, 1002: 15.952
- **stand_over** (Over your body: it stands over you and looks down): “A person stands still and bends forward to look down at the ground in front of their feet.”; candidates 1003: 0.138, 1002: 0.179, 1001: 0.18, 1000: 0.19
- **back_away** (Withdraw: it goes back into the dark): “A person slowly walks backwards for a few steps, then turns around and walks away.”; candidates 1001: 0.854, 1000: 0.858, 1003: 0.871, 1002: 1.078
