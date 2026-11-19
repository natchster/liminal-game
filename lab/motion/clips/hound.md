# The Hound: the clips

Made by `motion.py bake --creature hound` from `briefs/hound.toml` (Kimodo SOMA SEED v1.1 on MLX, kimodo-mlx).

| clip | seed | speed m/s | skate m/s | jitter | floor m | loop | score |
|---|---|---|---|---|---|---|---|
| fours | 2002 | 0.008 | 0.0137 | 5.3 | 0.0 | – | 0.185 |
| crawl | 2004 | 0.363 | 0.095 | 33.5 | 0.0 | 28–170 (0.0259) | 2.246 |
| listen | 2002 | 0.134 | 0.0478 | 22.5 | 0.0 | – | 0.654 |
| sniff | 2000 | 0.003 | 0.0086 | 4.4 | 0.0 | 43–128 (0.0057) | 0.893 |
| spring | 2000 | 0.106 | 0.0501 | 16.5 | 0.0 | – | 0.932 |
| gallop | 2001 | 0.593 | 0.1351 | 66.9 | 0.0 | 29–144 (0.0224) | 3.01 |
| prowl | 2001 | 0.215 | 0.0707 | 28.7 | 0.0 | 26–166 (0.023) | 1.723 |
| creep | 2001 | 0.471 | 0.1201 | 56.3 | 0.0 | 50–142 (0.0242) | 2.448 |

Candidates (lower is better: speed outside the brief ×3, foot skating / 0.08 m/s, jitter / 400 m/s³, below the floor / 2 cm; on all fours also the head too high and the hands too little on the floor):

- **fours** (the stance every clip is measured from (the body's rest pose: on hands and feet)): “A person is down on their hands and feet on the floor with their hips raised, holding completely still.”; candidates 2002: 0.185, 2001: 0.211, 2000: 0.219, 2003: 0.274
- **crawl** (Roam: its lope along the halls, stopping to listen every 4-8 s): “A person crawls forward steadily on their hands and feet.”; candidates 2004: 2.246, 2001: 2.351, 2005: 2.501, 2003: 2.602, 2000: 2.614, 2002: 2.778
- **listen** (Alert: the listening crouch, head cocked, 1.5 s (the tell)): “A person on their hands and knees stops crawling, raises their head and stays completely still, listening.”; candidates 2002: 0.654, 2000: 0.803, 2001: 1.276, 2003: 1.353
- **sniff** (Search, arriving: it sniffs about the spot): “A person on their hands and knees moves slowly forward, sniffing along the floor.”; candidates 2000: 0.893, 2001: 0.959, 2003: 1.147, 2002: 3.121
- **spring** (Strike: a crouch, 0.5 s, then it springs): “A person on their hands and feet crouches down and then lunges forward.”; candidates 2000: 0.932, 2002: 24.746, 2001: 32.586, 2003: 35.844
- **gallop** (Hunt: the charge at 6 m/s, below your sprint, for at most 6 s (the fastest a body on all fours was found to go)): “A person runs forward fast on all fours like a dog, using their hands and feet.”; candidates 2001: 3.01, 2002: 3.302, 2000: 3.459, 2006: 3.62, 2005: 3.989, 2007: 4.03, 2003: 4.195, 2004: 4.397
- **prowl** (Roam and Search, slow: its head hanging toward the floor, each hand set down with care (2026-11-19, the remodel)): “A person crawls forward slowly on their hands and knees with their head hanging down toward the floor.”; candidates 2001: 1.723, 2004: 2.03, 2000: 2.241, 2003: 2.441, 2005: 2.443, 2002: 2.473
- **creep** (Stalk: closing in on a sound, a hand set down, held, the next): “A person creeps forward very slowly on their hands and knees, placing each hand down carefully and pausing.”; candidates 2001: 2.448, 2000: 2.462, 2005: 2.519, 2004: 2.586, 2002: 2.657, 2003: 2.834
