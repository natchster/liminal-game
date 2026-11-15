# almond_water: why could this be here?

What the playtesters answered (`cargo xtask asset sheet almond_water`, a
fresh `placement-critic` a round; people's sessions, `docs/research/PLAYTEST.md`),
beside the brief's story, and the rules each difference became.

## The brief's story

Set down by someone who passed through, on what was at hand while they
worked or rested (a filing cabinet, a stack of boxes, a desk), meaning to
come back for it; dropped where they sat against a wall; pushed into a
corner where they slept. Never where people wash, in a shrine, or in the
Showroom.

## What they answered

**Round 1** (2026-11-15; 27 frames, seeds 4242, 77, 202, levels 5, 7, 9):

| Rule | Obvious | Could be | No idea |
|---|---|---|---|
| on a surface | 12 | 1 | 0 |
| on the floor by a wall | 0 | 6 | 3 |
| in a corner | 0 | 4 | 1 |

On furniture or boxes "someone put their drink down there", every time
("a pupil's water, left on their desk with their book and apple"; "someone
packing a parent's things put their drink on the box"). On the floor the
best reason was "someone sat against the wall", and the two lying on their
side "read as dropped, which suits a bottle on the floor better than
standing up". The four with no reason stood out from the wall in the open
("a couple of metres from the wall", "between pillars"), and one beside
another far off "looks like a trail someone laid: it points the way".
Often hard to see: "only the ring of the cap reads".

**Round 2** (after the rules below; a fresh playtester, the same floors):

| Rule | Obvious | Could be | No idea |
|---|---|---|---|
| on a surface | 12 | 0 | 0 |
| on the floor by a wall | 0 | 10 | 0 |
| in a corner | 0 | 3 | 2 |

Every one by a wall now "dropped and rolled to the wall" or "someone sat
with their back to this wall". The two with no reason: a corner of a bare
concrete ledge ("maybe someone huddled here, but nothing else says so"),
and one the playtester could not find (clear PET on a dark floor).

## The differences, and the rules

- A thing "by a wall" that stands out from it reads as set out in the
  open: floor spots now slide to the wall's foot (a hand's breadth at most),
  and a lone pillar is not a wall (`world::placement_rules`, `slide`, `run`).
- A bottle on the floor reads as dropped when it lies on its side: on the
  floor three in four lie; set down on something, it stands (`lay`).
- Two in one view read as a trail, the game's reason: no two of a kind
  nearer than 6 m (`APART`).
- Open: clear PET is hard to see on a dark floor. That is what a real one
  is; whether it is too hard to find is for people (`PLAYTEST.md`).
