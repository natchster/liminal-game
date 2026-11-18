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
