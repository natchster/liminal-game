# The props: real things, barricades that close, things a hand can use

How the rooms' props are made to look like real, worn things; how a
barricade closes the opening it stands in; and which props can be used.
The canon is [`LORE.md`](LORE.md), the art bible [`STYLE.md`](STYLE.md),
the rules [`PILLARS.md`](PILLARS.md). The research behind it:
[`../research/props-census.md`](../research/props-census.md) (every kind of prop,
looked at and graded), [`../research/barricades.md`](../research/barricades.md)
and [`../research/props-interactions.md`](../research/props-interactions.md).

## The work

How this was planned and done (the brief, the analysis, milestones, progress, open after p3): [`../plan/PROPS.md`](../plan/PROPS.md).

## The plan

### 1. A real thing for every toy

Each `cartoon` kind is made again as the real thing it stands for:

- **From a real one.** Its real size and construction from a maker's
  sheet or a measured photograph (cited in its family's notes); its
  materials (painted steel, moulded polypropylene, chipboard, kraft card,
  cotton, glazed ceramic) at their real albedo and finish.
- **Modelled whole** in a Blender family of its theme (`assets/src/blender/props/<theme>/<family>.py`),
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

Following [`../research/barricades.md`](../research/barricades.md) 5:

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

Following [`../research/props-interactions.md`](../research/props-interactions.md)
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
   Note its size, construction and materials in its family's notes
   (`docs/research/props-remodelling/<family>.md`).
3. Model it in its family, at its real size, in its own frame (x across its
   front, y up from the floor, z out of its front), one part a material;
   register each with `auto(name, build, budget)`; keep to the budget.
   `cargo xtask assets props --batch <family>` builds and checks it (seconds);
   `--write` writes `prop_meshes.bin`.
4. Rewrite its build: the parts as modelled, its materials real, its
   print painted, its random draws kept call for call. A size the placer
   randomized may scale the whole thing a little; nothing is stretched out
   of its proportions.
5. Shoot it: the census's camera (`census.tsv`) on the old build and
   `--shots-at` on the new give the same shot of both, for a blind review.
   Both run as `--fresh` (the per-frame randomness fixed, the save not
   read): without it a floor's tint, flicker and dust differ from run to
   run, and a reviewer judges those instead of the prop.

What the batches learned (each cost a batch time once):

- **Remodel the look, keep what the prop does.** A flame's light and
  flicker, a sound or a registered position that drives one (a lit
  fireplace's `lit_fireplace_positions` drives the household crackle), a
  pump's hum and bubbles: kept, on the old draws. A glow that is neither a
  flame nor a lamp (glowing eyes, a lit-up ball, a relic, a chalk circle's
  pulse) goes, its draws kept.
- **Motion needs a cause in the room**: air stirring a hanging thing, a
  flame. A thing that moves by itself forever (a top that spins, a globe
  that turns, an orrery that runs, a planchette that drifts, a ring or a
  cigar that rocks) goes, its draws kept.
- **A part tipped over by a rotation in the game is measured by its
  tilted box**: the placement pass lifts a floor prop whose rotated bounds
  dip under the floor (a lying party hat floated 53 mm). Build a lying
  part already lying, and turn it in the game only about y.
- **A static prop whose parts all repeat is instanced** after the build
  (`world::instancing::instancify_static`) and leaves `g.world`, so the
  dump and the census no longer see it: add `--flags noinstance` to the
  dumps, census and shots of such a placer, before and after alike (17
  placers the census "never saw" were these). A common instanced prop
  must stay instanced: every part a shared mesh, no per-instance canvas.
- **Write each `auto()` budget as a literal number**: the asset lint reads
  budgets from the calls' text, and a conditional reads as its last
  number.
- **Never run two `cargo xtask assets props --write` at once**: one reads
  the other's half-written output and rebuilds frozen parts with
  different bytes.
- **Commit as you go** when working in a worktree, and **never put personal
  data in a request** (a generic User-Agent is enough for Wikimedia).

## What only people can tell

- Does a remodelled prop read as the thing at a glance, in the dark, by
  the lamp? Is anything now too clean, too new or too detailed for where
  it is?
- Does a barricade read as something people built to keep something out
  (or in)? Do players try to get through it?
- Do players use the new verbs, and does using them feel like handling a
  real thing, or like a game's loot?
- Does a candle still burning in an empty shrine, or a cigarette in an
  ashtray, read as wrong (someone was just here; nobody was), or only as a
  game's light? Does anyone find what the bench's candles do?
