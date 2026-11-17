# high_bay: why could this be here?

What the playtesters answered (frames where the layout hangs it, a fresh
`placement-critic`; people's sessions, `docs/research/PLAYTEST.md`), beside
the brief's story, and the rules each difference became.

## The brief's story

A tall concrete hall's lamp: hung on its stem from the slab of the
Habitable Zone's halls, where the tubes hang elsewhere.

## What they answered

**Round 1** (2026-11-17; 5 frames, seeds 7 level 6 and 4242 levels 7 and
13):

| Where | Obvious | Could be | No idea |
|---|---|---|---|
| a tall bare concrete hall or room, a pillared hall | 3 | 0 | 0 |
| a tall hall seen from an office beside it | 0 | 1 | 0 |
| under a low tiled ceiling over carpet (a hall papered by moss) | 0 | 0 | 1 |

"A tall bare concrete room needs a lamp hung down from its high ceiling;
this is the usual industrial lamp for it"; "a pillared basement hall like a
car park or warehouse; the lamps hang over the aisles between the
columns". Seen from an office: "the room is a big open hall like a
workshop, so hanging lamps light it; as an office … industrial lamps
instead of ceiling tubes feel off". Under the papered ceiling: "a warehouse
lamp under a low tiled lobby ceiling, against a partition; it seems there
only to give the dark a light (the game's reason)".

## The differences, and the rules

- **Only under a tall bare slab.** Where moss has papered a hall with
  tiles and carpet, or its ceiling is too low to hang one from (5.2 m and
  under), the hall keeps its tubes (`World::lay_panels`).
- **A hall seen from an office** keeps its high-bays: the hall is a hall,
  and the answer was "could be". The blind review preferred the tubes there
  twice (`docs/plan/FACILITY.md` F3); for a person.
