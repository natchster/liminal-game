# The props: real things, barricades that close, things a hand can use

How the rooms' props are made to look like real, worn things; how a
barricade closes the opening it stands in; and which props can be used.
The canon is [`LORE.md`](LORE.md), the art bible [`STYLE.md`](STYLE.md),
the rules [`PILLARS.md`](PILLARS.md). The research behind it:
[`research/props-census.md`](research/props-census.md) (every kind of prop,
looked at and graded), [`research/barricades.md`](research/barricades.md)
and [`research/props-interactions.md`](research/props-interactions.md).

## The brief (2026-10-31)

> Find all remaining cartoonish-looking non-interactive props in backrooms
> levels and remodel them in more suitable and realistic way. May
> barricades more filling the space, make more props interactive in some
> way without breaking the lore

## The analysis

**A third of the props read as toys.** The census
([`research/props-census.md`](research/props-census.md)) framed every kind
of placed prop on 272 floors of the rooms (`--prop-census`) and had seven
fresh reviewers grade each against `STYLE.md`: of 652 kinds, 213 are
`cartoon`, 412 `plain`, 21 `real`, 6 could not be seen. Thirty-five of
the cartoon kinds are on 40 or more of the 272 floors (the torn notes'
sheets, the mannequin, the wet floor sign, the mirrors, the wall clock,
the book stacks, the coat rack, the stains on every wall and ceiling, the
scrap paper, the chalk, the holes in the walls, the rusted pipes, the fire
alarms, the trash cans, the crushed cans, the crates), 66 on 10–39, 112
on fewer. What makes them toys is a short list: bodies of primitives (an
egg with a sphere for a head and dots for eyes); an icon where a picture
or print should be (a clock without numerals, a portrait that is a
restroom sign, black bars for text); toy primaries; cloth as a slab;
decals as hard-edged polygons; light from what is not a light (a glowing
cauldron, lamps for a cat's eyes); wrong proportions and parts that do not
join. And nothing is worn: even the props graded `real` are clean.

A placer composes its prop of primitives (`box_geo!`, `cyl_geo!`), each
swapped for a Blender part fitted to the primitive's box (`pg`), in one
flat colour: the parts are finer than the boxes, but the composition and
the flat colour are still the toy. The props that read as real (the
vending machine, the windows, the cardboard boxes, the suitcase) were
made the other way: the whole thing modelled at its real size from a real
one, in the materials it is made of, with what is printed on it painted.

**The barricades do not close anything**
([`research/barricades.md`](research/barricades.md)). A barricade is an
oversized filing cabinet, a book stack, two to four boxes and a traffic
cone (1.25 m wide in the median) on a tile whose two opposite sides are
open; the whole 4 m tile is an invisible wall. Of 53 measured, 28 stand
in open floor where a body walks round them; in a corridor one fills 32%
of the width and 20% of the face, and a body stops 1.5–2 m short of it,
on clear floor it cannot enter. The same tile can be chosen twice; a
creature sees, hears and paths through one.

**Almost nothing can be used**
([`research/props-interactions.md`](research/props-interactions.md)). A
floor has about 26 verbs, all on the core props and the systems (drawers,
switches, the payphone, the lifts, the exit, her page, the machines, the
blinds, the secrets); one themed placer of 649 has one (a red and green
game button). The props on every floor (boxes, chairs, books, trash cans,
cabinets, lockers, a TV) do nothing.

## The plan

### 1. A real thing for every toy

Each `cartoon` kind is made again as the real thing it stands for:

- **From a real one.** Its real size and construction from a maker's
  sheet or a measured photograph (cited in the batch's notes); its
  materials (painted steel, moulded polypropylene, chipboard, kraft card,
  cotton, glazed ceramic) at their real albedo and finish.
- **Modelled whole** in a Blender batch of its own (`assets/src/blender/props/r*.py`),
  at its real size, one part a material, with what a real one has: its
  bevels and seams, its screws and hinges, its feet, its moulding marks.
  The game places the parts as modelled (`prop_part_as_modelled`, a scale
  of one), merging what does not move by material.
- **Its print painted.** Labels, faces, type and pictograms are painted
  in the canvas (`canvas_tex`, its own stream) as a real one is laid out,
  in a real typeface, rubbed where hands rub it; never a bar for text.
- **Its picture a picture.** A photograph, portrait, painting, map, chart
  or poster shows an image, from a public-domain or CC0 source listed with
  its licence and checksum like the Building's photographs, never an
  icon.
- **Cloth hangs.** Coats, aprons, towels and sheets are draped in Blender
  (its cloth, headless; `research/modelling.md`), with shoulders, folds
  and weight.
- **A mark is a mark.** Stains, mould, rust, water streaks, chalk, peeled
  paint, cracks and cobwebs are painted soft and uneven, with a source
  and a direction (a tide line, a drip, the grain of chalk), never a
  polygon.
- **Nothing shines that does not.** No emissive on what is not a lamp, a
  screen or a flame; eyes catch the light (`STYLE.md`).
- **Toys stay toys,** as real ones are: moulded, faded, scuffed, in the
  muted colours a toy has after years on a floor.
- **The party and the shrine** leave what real ones leave: paper bats
  taped to a wall, a carved pumpkin gone soft, a plastic skull from a
  shop, candles burnt down; never a cartoon of them.
- **Wear.** Every surface gets the wear its use gives it: paint chipped at
  edges and corners, plastic scuffed where it is held, dust on what faces
  up, grime where it meets the floor (the shader's wear made strong
  enough to read, milestone P4).
- **Nothing else moves.** A remodelled placer keeps its random draws, the
  same calls in the same order, so nothing placed after it moves; where
  that cannot hold, the reference levels are re-recorded and the commit
  says why. It keeps its theme, its places and how many.

What the census found broken is mended with its prop: the floor hole that
stands up out of the carpet, the mannequin's arms, the TV's antenna, a
pinwheel and a spinning top on their points, text running off a poster.

### 2. Barricades that close the way

Following [`research/barricades.md`](research/barricades.md) 5:

- **Only in an opening:** a doorway in a partition or a corridor between
  solid walls, measured on the solids (its width wall face to wall face,
  its top the lintel or the ceiling); never twice on one tile, never
  cutting off a pickup, never near the spawn, never in the Showroom or the
  pool halls; spawn to exit still solvable.
- **Filling it:** wall to wall within 2 cm, floor to within 5–30 cm of the
  lintel or the ceiling, never above it; the gaps a real pile has.
- **Of what the place had,** by its purpose: a boarded doorway (plywood
  and 2x4 braces, bolted) in utility halls and service rooms; desks on
  end, filing cabinets and chairs in an office; shelving, pallets and
  archive boxes in storage; mattresses on edge, a door off its hinges and
  a dresser in living quarters; hoarding to the lintel in a tall hall.
  Who built them is never said.
- **It is a wall:** a body stops at the pile, not 2 m short of it; a
  creature cannot see through it, hear you through it or path through it;
  the place's answer to a cry goes round it.
- **Nothing else moves:** the old draws are kept (replayed without
  meshes), the new choice and dressing come from a stream of their own.

### 3. Things a hand can use

Following [`research/props-interactions.md`](research/props-interactions.md)
4, on the props that are on every floor, each a real thing a hand does,
in the game's voice, with no narrator, no dice and no effect said:

- a cardboard box opens (packing paper, sometimes a few sheets of paper);
- a locker opens where its padlock hangs open;
- a CRT turns off and on;
- the dead lift's button can be pressed;
- a book stack gives pages to write on;
- a chair knocked over can be set upright;
- a trash can can be looked into;
- a phone's receiver can be lifted (a tone, or nothing; never a voice);
- a tap can be turned.

Each is held to the checklist of the research (2): its sound from the
prop, silent until within earshot; noise given to creatures through what
they hear, never by setting them to hunt; registered after settling;
none on the Lobby, in the Showroom or the Borrowed House; written into
`LORE.md` from the code's real gates and reviewed by the `lore-keeper`.

## How a prop is remodelled

1. Read its placer (or builder): what it composes, its sizes, its draws,
   where it goes. Read the reviewer's evidence for it
   (`research/props-census/review*.md`).
2. Find the real one: a maker's sheet, a catalogue or a measured photo.
   Note its size, construction and materials in the batch's notes.
3. Model it in the batch, at its real size, in its own frame (x across its
   front, y up from the floor, z out of its front), one part a material;
   register each with `auto(name, build, budget)`; keep to the budget.
   `cargo xtask assets props --batch rNN` builds and checks it (seconds);
   `--write` writes `prop_meshes.bin`.
4. Rewrite its build: the parts as modelled, its materials real, its
   print painted, its random draws kept call for call. A size the placer
   randomized may scale the whole thing a little; nothing is stretched out
   of its proportions.
5. Shoot it: the census's camera (`census.tsv`) on the old build and
   `--shots-at` on the new give the same shot of both, for a blind review.

## Milestones

| # | What | Evidence |
|---|---|---|
| P0 | The census (`--prop-census`, `--census-only`, `--shots-at`; the core props tagged by kind), the research (three notes), this design | the census, graded by seven reviewers |
| P1 | **The everyday**: the 35 `cartoon` kinds on 40 or more of 272 floors, and what the census found broken | a blind review of each against its old self (census cameras, both orders); reference levels; materials and asset lints; the full check |
| P2 | **The common**: the 66 on 10–39 floors | the same |
| P3 | **The rare**: the 112 on fewer, and the 43 placers never placed, judged from their code | the same |
| P4 | **Wear**: the shader's wear strong enough to read, by use (edges, what faces up, the floor) | a blind review; goldens; bench |
| P5 | **Barricades** that close the way | a test of every barricade (it seals, fits, stays under the ceiling, cuts off no pickup, is never on her walk); the level sweep; a blind review; reference levels |
| P6 | **Things a hand can use** | a test a rule; listen; the placers' smoke tests; `lore-keeper` |
| P7 | **The canon and the reviews**: `LORE.md`, `QUALITY.md`, the ledger; `lore-keeper` rounds; the full check | |

## Progress

| # | Commit | Notes |
|---|---|---|
| P0 | (this commit) | The census: `Game::census_shots` frames each kind of placed prop (a core prop's kind tagged by `Game::tag_kind` as `place_props` runs, `kind!`), `--prop-census N`, `--census-only`, and `--shots-at` for the same camera again; 272 floors, 652 kinds, seven reviewers (`research/props-census/`). The research: `research/props-census.md`, `research/barricades.md`, `research/props-interactions.md` |

## What only people can tell

- Does a remodelled prop read as the thing at a glance, in the dark, by
  the lamp? Is anything now too clean, too new or too detailed for where
  it is?
- Does a barricade read as something people built to keep something out
  (or in)? Do players try to get through it?
- Do players use the new verbs, and does using them feel like handling a
  real thing, or like a game's loot?
