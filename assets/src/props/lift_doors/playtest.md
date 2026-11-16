# lift_doors: why could this be here?

What the playtesters answered (`cargo xtask asset sheet lift_doors --kinds
core_elevators,core_broken_elevator`, a fresh `placement-critic` a round;
people's sessions, `docs/research/PLAYTEST.md`), beside the brief's story,
and the rules each difference became.

## The brief's story

An office floor's lifts against a wall of its core, where the floor's
rooms are: a working one and a dead one, and on half the floors a broken
one, jammed open on the black of its shaft.

## What they answered

**Round 1** (2026-11-16; 16 frames, seeds 4242, 77, 202, levels 5-9):

| Where | Obvious | Could be | No idea |
|---|---|---|---|
| in a core wall or pillar of an office floor, a lift lobby | 7 | 0 | 0 |
| a back corridor, the frame standing out of the wall | 0 | 3 | 0 |
| a broken one in a homely room, by a dormitory's beds, at a camera's edge | 0 | 3 | 0 |
| in front of a pillar narrower than itself | 0 | 0 | 1 |
| out of the camera's view | 0 | 0 | 2 |

"The lift core in the middle of an open-plan office floor"; "the pillar is
the lift shaft rising through the open floor". In back corridors: "the
doors look stuck on the wall like a cupboard rather than opening into a
shaft". In front of a thin pillar: "standing on their own, with no wall or
shaft around them: it looks like the game's way out".

## The differences, and the rules

- **A lift's entrance is set nearly flush in its wall.** Its frame stood
  23.5 cm out (16 cm, and its origin 6 cm further off the wall than its
  code said: the code review); it stands 7.5 cm out now, the panels just
  behind its face, the shaft's look moved with them.
- **Not against a wall narrower than its frame** (the thin pillar): queued
  (`docs/plan/MILESTONES.md` 26). A check by the floor's solids refused
  many spots the floors have always used (their walls are thin partitions
  and openings, not their tiles' blocks), the shaft's stage among them;
  it needs the walls' drawn faces.
- Two frames that missed the lift are the census's camera, not its place.
