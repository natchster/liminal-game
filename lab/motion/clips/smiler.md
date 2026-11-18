# The Smiler: the clips

Made by `motion.py bake --creature smiler` from `briefs/smiler.toml` (Kimodo SOMA SEED v1.1 on MLX, kimodo-mlx).

| clip | seed | speed m/s | skate m/s | jitter | floor m | loop | score |
|---|---|---|---|---|---|---|---|
| stand | 1001 | 0.005 | 0.0054 | 3.4 | 0.0 | – | 0.076 |
| idle | 1003 | 0.001 | 0.0048 | 3.6 | 0.0 | 41–161 (0.0117) | 0.46 |
| look | 1001 | 0.002 | 0.005 | 5.2 | 0.0 | – | 0.076 |
| creep | 1002 | 0.888 | 0.0498 | 59.1 | 0.0 | 38–128 (0.0325) | 1.855 |
| listen | 1003 | 0.056 | 0.0115 | 9.6 | 0.0 | – | 0.168 |
| stalk | 1003 | 1.105 | 0.0558 | 89.5 | 0.0 | 22–131 (0.0332) | 2.676 |
| run | 1005 | 2.094 | 0.1002 | 282.7 | 0.0 | 10–145 (0.0795) | 5.408 |
| lunge | 1000 | 0.859 | 0.0701 | 102.3 | 0.0 | – | 1.132 |
| stand_over | 1000 | 0.008 | 0.0081 | 4.9 | 0.0 | – | 0.113 |

Candidates (lower is better: speed outside the brief ×3, foot skating / 0.08 m/s, jitter / 400 m/s³, below the floor / 2 cm; on all fours also the head too high and the hands too little on the floor):

- **stand** (the stance every clip is measured from (the body's rest pose is its stoop)): “A person stands upright and completely still, with arms relaxed at their sides, looking straight ahead.”; candidates 1001: 0.076, 1003: 0.09, 1000: 0.096, 1002: 0.11
- **idle** (Idle: in its dark, facing a corner; it barely moves, and the little it does is weight): “A tall, heavy person stands upright and still, arms hanging at their sides, slowly and heavily shifting their weight from one foot to the other.”; candidates 1003: 0.46, 1000: 0.523, 1002: 0.593, 1001: 0.844
- **look** (Alert: it has seen a light, and its head goes to it before its body does): “A tall person standing upright and still slowly turns their head to the right to look at something, then holds it there.”; candidates 1001: 0.076, 1002: 0.078, 1003: 0.085, 1000: 0.088
- **creep** (Search: to where your light went out, slower, unsure where you are): “A heavy person walks forward slowly and deliberately with an upright posture, placing each foot down heavily, arms hanging at their sides.”; candidates 1002: 1.855, 1001: 3.062, 1000: 3.412, 1003: 4.088
- **listen** (Search, arriving: it stops where the light was and listens): “A heavy person stops walking, stands upright and completely still, and tilts their head slightly to one side as if listening.”; candidates 1003: 0.168, 1000: 0.182, 1002: 0.202, 1001: 0.204
- **stalk** (Hunt, the approach: long strides, leaning in): “A tall, heavy person walks forward steadily with long, heavy strides and an upright posture, arms hanging loosely.”; candidates 1003: 2.676, 1004: 2.867, 1002: 2.93, 1005: 2.945, 1000: 2.995, 1001: 3.911
- **run** (Hunt: you are lit and in its sight; it comes at 4.5 m/s, faster than your walk): “A person sprints forward as fast as they can.”; candidates 1005: 5.408, 1003: 6.281, 1001: 6.321, 1002: 7.613, 1000: 8.256, 1004: 8.316, 1007: 8.403, 1006: 9.041
- **lunge** (Strike: a sharp inhale, coiling as it comes, then the lunge (7 m/s over at most 4 m)): “A person bends their knees, then quickly steps forward and grabs at something in front of them with both hands.”; candidates 1000: 1.132, 1001: 1.364, 1003: 11.275, 1002: 15.952
- **stand_over** (Over your body: it stands over you and looks down): “A tall person stands upright and bends their head down to look at the ground in front of their feet.”; candidates 1000: 0.113, 1002: 0.118, 1003: 0.133, 1001: 0.144
