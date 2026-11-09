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
| P0 | `f754918` | The census: `Game::census_shots` frames each kind of placed prop (a core prop's kind tagged by `Game::tag_kind` as `place_props` runs, `kind!`), `--prop-census N`, `--census-only`, and `--shots-at` for the same camera again; 272 floors, 652 kinds, seven reviewers (`research/props-census/`). The research: `research/props-census.md`, `research/barricades.md`, `research/props-interactions.md` |
| P1 (1) | `024cdba` | The wet floor sign, the model for the rest: the common A-frame at its real size (batch `r01`), its print painted, its wet a soft patch; its draws kept (the reference levels moved by its two signs alone) |
| P1 (2) | `98fb138` | Twelve more, made by three workers in worktrees of their own (batches `r02`, `r04`, `r06`) to the brief of "How a prop is remodelled": the book stack (cloth hardbacks and a paperback, each stack its own painted atlas), the suitcase (a 1960s hard shell, open, its lining, straps and the clothes packed in it), the coat stand (a bentwood rack, a wool overcoat hanging from its collar loop), the pile (a mound of grit with brick, concrete, drywall and a ceiling tile's corner, its dust); the torn note (a typed sheet of bond folded and torn, its corner turned down when read; the Building's notes too), the mannequin (a fibreglass egg-head figure on its plate and calf rod, its join lines, chipped), the wall clock (a 34 cm school clock, its printed dial); the hole in the wall (torn drywall into its stud bay, a mask drawn before the wall, chips below), the rusted pipes (painted steel pipe with its fittings, strut clamps and a gate valve, rust weeping), the posters (six flyers and notices in real type, taped, curled, torn, faded), the fire alarm (a pull station, FIRE and PULL DOWN raised, its conduit), the coat hooks (an oak rail with cast double hooks). Each kept its draws (every other placer's objects the same on their evidence floors). A blind review against the old (24 shots from the census cameras, both orders) preferred the new 24 to 0. Integration: a part name two batches shared, the asset lint reading the budgets `auto()` declares, a hole's inside marked as the wall's own (the leaning-wall test), soft blobs squashed whole (a hard-rimmed dust patch, and the wet floor sign's patch); `core_props_tail` compares the notes' place and turn, not their make, and remodelled builders' draws, not their tops. Re-recorded: the core props' fixtures, the reference levels (the remodelled props and the five wall placers' objects alone) and three goldens |
| P5 | `e791bb6` | **Barricades that close the way** (`core_props/barricade.rs`, batch `x01`: plywood, a 2x4, a 1x6, bolt heads and nuts, a steel desk and its top, a stacking chair's frame and shell, a pallet, boltless shelving and a deck, a twin mattress, a moulded door, a dresser). The old choice runs for its draws alone (its pile built and never added), so nothing placed after it moves; its tiles stand in while the themed props are placed; then `barricades_settled`, once the walls are final and before her trail, chooses as many on a stream of its own (`BARRICADE_SALT`) among the real openings (a tile walled across the way and open along it, measured on the solids at a knee, a hand and a head: 0.9–4.1 m face to face; its top the lintel or the ceiling), never near the spawn, in a pool hall, on a floor with rules of its own, beside another or cutting off the door or a pickup, one at most on the Lobby, and builds each by its zone's purpose: an office's desk on end, filing cabinets, chairs upside down and boxes; a store room's shelving, pallets on edge and boxes; a home's mattresses leaning, a door propped across and a dresser; else boards nailed across a doorway onto two uprights, or sheets of plywood (each its own veneer, nailed round) braced and bolted. A slab across the opening stops a body and a look (`circle_hits_wall`, `los`), and closes its tile to the creatures' ways and hearing (`bfs_open`) and the way to the door. `tests/barricades.rs` (91 barricades on 54 floors: each seals, fits under its ceiling, one a tile; the door, every pickup and her walk stay whole; where they stand); the level sweep counts what lies behind one by design, and fails a pickup locked away (the open issue closed). A blind review against the old piles preferred them 12 to 0. Her wall marks keep off a face something stands against (a machine, a barricade, a cell's door: `mark_wall`), found when her trail, going round the barricades, taped one behind a machine. Re-recorded: the reference levels (the old piles gone, the new barricades, her trail and camp and the secrets round them; every themed placer's objects the same) and eight goldens (a board-up ends the service tunnel's corridor; the Poolrooms' pile gone; a mark of hers moved) |
| P1 (3) | `6c37ebb` | Twenty-two more from five workers (batches `r03`, `r05`, `r07`, `r08`, `r09`): the baby doll (a 1970s bent-limb baby on its back in a faded romper, one sleep eye shut, one stuck half open), the rubber duck (a blow-moulded bath duck, its yellow faded, its bill's paint chipped), the hole in the floor (drawn through the floor: the dark under it, a steel joist, the torn deck, the slab's broken face; broken concrete round the rim); the wall mirror (a stainless channel-frame washroom mirror, its silver gone black from the edges, cracked from a blow), the still floors' cobwebs (dusty tangles found in a corner or along a wall's top; the weather extras' other floors are still to do), the office plant (a neglected Dracaena in a dusty planter); the marks (wall and ceiling stains, water streaks, ceiling cracks with the bits that fell, cobwebs, puddles: painted soft, never a hard ellipse); the scrap paper, the chalk marks, the bin, the sock, the bottle caps, the crushed cans; the wooden crate (slatted pine gone grey, stencilled, wood wool), the broken window (a lower sash taken out and leaning, its glass), the umbrella (a furled stick umbrella on its ferrule), and the things put away (a flat cap, a hongbao, a linen pouch, a holed coin). Each kept its draws. Two blind reviews against the old (census cameras, both orders): 30 to 0 for `r03`, `r07`, `r08`; 17 to 0 with 2 ties (shots where the thing was out of frame) for `r05`, `r09`. `core_props_tail` checks a still floor's web hangs at the ceiling within a tile of the old decal's spot. Census and `--shots-at` shots now run as `--fresh`: without it a floor's tint, flicker and dust came from the clock, and a reviewer saw 11/12's walls change colour between the two sides. Re-recorded: the core props' fixtures, the reference levels (the remodelled kinds one for one; a still web is one object, not five; the crate on 4242/8 no longer dropped by the placement pass for its old pellets) and seven goldens (the marks, far props) |
| P6 | `6c37ebb` | **Things a hand can use** (`core_props/usable.rs`): once the props have settled, `usable_settled` gives the top box of a stack, a bin, a book stack, a shut locker whose padlock hangs open (its shackle lifted, the tell), a dead lift and the floor's television their verbs (OPEN THE BOX, LOOK INSIDE, TEAR OUT PAGES, OPEN THE LOCKER, PRESS THE BUTTON, SWITCH IT OFF / ON), from level 5 and never on a floor with rules of its own. What each holds is fixed on a stream of its own (`USE_SALT`): paper from boxes (one in four), bins (three in ten) and books (always, once; from level 9 the damp leaves a page or two); in an opened locker its sides, a jacket on its hook (the coat stand's overcoat on the coat rail's hook) and a jumper folded on its floor, and one in seven an almond water in the jacket's pocket; a car going by behind one dead lift in three; the lit set's hiss goes with its light. A hand reaches 1.2 m. Each sound comes from its prop, only within earshot (`prop_sound`), and creatures hear it as they hear the machines (`prop_noise`): the tape, a locker's door and a set coming on are loud enough to stop one near (above `SUSPECT` after a silent frame's drain), a bin and a page reach only one already listening. The ajar lockers (the builder's) have the same sides and jacket in place of their box coat. `tests/usable.rs`: every verb on its prop, what it gives fixed and given once, the TV off and on, a loud verb stopping a roaming Hound 2 m off and a quiet one not (it fails at the first draft's loudness); `--census-use` uses every verb before the census frames them. Canon: `LORE.md` *Within reach*, after a `lore-keeper` round (it found the first draft's sounds too quiet to stop anything, a torn page left on the floor that the Reader would not stop for, the wrong set named, and her paper running out against books on every floor) |
| P1 (4), P2 (1) | `c2270eb` | The last everyday kind, and seventeen of the common, from four workers (batches `r10`, `r11a`, `r12`, `r17`): the weather extras of the wet floors (the puddles of `r07`, lit; none on a pool's water), the derelict floors (a fallen 610 mm mineral-fibre tile broken in four, a bent cross tee of its grid, its crumbs and dust), the electric floors (a cut length of sheathed cable, stripped to its cores, its spark a small blue-white arc) and the haunted floors (a pillar candle burnt down, its flame a flame, its wax pool); the water puddles, dust bunnies (felted lint, fibres cut out in an alpha test), handprints (grime where a hand's pads touched), claw marks (paper torn to its plies, gypsum gouged) and chalk arrows (a stranger's way-mark; half the old ones had faced the wall and never shown); the office wall phone, the payphone, the emergency phone, the lift's button panel (ADA 407.4), the fire alarm pull (`r06`'s station, flush), the extinguisher cabinet and its extinguisher, the breaker panel; the rubber ball, the die-cast car, the paper pinwheel (lying down; it had stood on its point), the jigsaw piece, the magnetic letters. A painter's own stream started the same on every floor, so the first claw mark of every floor was the same: such painters mix where the thing is into the seed they use. Each kept its draws. Blind reviews against the old (fresh reviewers, both orders): 13–0, 15–0, 9–0, 11–0. `core_props_tail` compares a weather extra's kind, place and turn (a candle turns by where it stands). Re-recorded: the reference levels (these placers' own objects; the weather extras rest on the floor where the placement pass had lifted them) |
| P2 (2) | `c2270eb` | Twenty-three more of the common, from four workers (batches `r11b`, `r13a`, `r14a`, `r16`): the peeled paint (a film cracked and flaking to an older coat or the plaster, curled flakes modelled), the peeling wallpaper (a drop come away at its seam, its pasted back), the rust streaks (from a rusted anchor or a shelf's last bracket), the mould (colonies in soft clusters, the damp's tide line), the pipe leak (`r06`'s pipe, a weeping elbow's crust and drop); the framed wall mirror (wood or gilt, `r05`'s glass, a washroom's wall-hung basin under it), the cracked mirror, the chalkboard (an office's mundane notes in chalk, the last wiped), the hanging plank sign, the biohazard sign (ISO 7010 W009, the trefoil to its construction), the cross-stitch sampler (14 count on linen); the work boot, the derbies, the shoe box, the dropped and the work glove (one glove modelled); the wine glass (the windows' glass), the enamel kettle, the cast-iron skillet on its nail, the stale loaf on its board, the broken bottle, the portable sewing machine (an invented maker), the split firewood. The chalkboard no longer says THEY WATCH in a typewriter's capitals (lore is noticed, never told). Each kept its draws. Blind reviews against the old: 15–0, 16–0, 15–0, 15–0. Found along the way: `r06`'s rusted pipes turned their into-the-wall elbows out of the wall (fixed), and the wall placers take a pool's curb for a wall (open). Re-recorded: the reference levels (these placers' own objects) |
| P2 (3) | `c2270eb` | The last twenty-six of the common, from four workers (batches `r13b`, `r14b`, `r15`, `r18`): the wall hook (`r06`'s cast hook on an oak block, a knitted scarf over it), the hearth broom, the child's apron (draped by Blender's cloth), the rolled rugs (tied, sagging, fringed), the floor rug (a Hamadan, knotted, its path worn); the safety glasses, the wristwatch, the keys on their ring with a tag, the reading glasses on `r02`'s books, the curling ribbon, the twin-bell alarm clock; the tissue box, the tape roll, the playing cards, the receipt (`r08`'s paper), the paperback (open or shut), the newspapers (an invented local daily), the paper dart, the napkin; the washcloth, the nail polish, the floss, the overturned pail, the mop bucket and its wringer, the school desk, the dead geraniums. Where a real one's width would have pinned a pillar's face into a wall, a smaller real one was chosen (the scarf for a coat, a hearth broom for a corn broom, a child's apron): `settle_props` probes a wall prop's width. The reference levels now record the settled walls (`solidBoxes`, `solidsHash`); on 110 floors they are the same before P2 and after it. The asset lint reads a budget declared before `auto()`'s keywords and on names made in a loop. Blind reviews against the old: 14–0, 12–0, 16–0, 13–0. The batches' placers and what only they use live in a module a batch (`placers_p19/src/r12.rs` and the like), so no shared file passes the size cap. Re-recorded: the reference levels |
| P3 | `2b7a82b` … `6611fb2` | **The rare**: the 112 kinds on fewer than 10 of the census's 272 floors, and the 43 placers it never saw (26 rare, found on a probe of 440 more floors; 17 instanced out of the dump), in 22 batches (`t01`–`t22`) by workers in worktrees of their own, each to the brief of "How a prop is remodelled": the pictures on the walls and the maps, plates, posters and triptych (real CC0 pictures in one atlas, `visuals::pictures`, two tarot cards public domain); the washroom's leavings and fittings; bits of the outside indoors and the garden's things; the kitchen, hall and party table; the Halloween decorations, the shrine's and the study's things; furniture (the bookshelf, the locker, the fireplace, the cuckoo clock, the secretary desk, the aquarium); taxidermy; the party's paper; plush, dolls and toys; things on walls; and the small everyday (the cigarette butts and the wet floor sign, common and kept instanced). Each kept its draws call for call (every other placer's objects, `solidsHash` and the marks the same on its floors). Blind reviews against the old (fresh reviewers, both orders): 16–0 for t01, t02, t05, t06, t07, t09, t10, t11, t12, t13, t14, t15, t16, t17, t19, t20, t21, t22; 15–0 for t08 and t18 (30 pairs); 14–0 for t03 and t04. One rule for behaviour across the batches (above): the lit fireplace keeps its fire, light and crackle; the shrine's and the lantern's candles, the aquarium's pump; the glows that were no flame went, and the motion with no cause. Found along the way: the placement pass lifts a tilted part by its box; the asset lint misreads a conditional budget; the usable test's creature guessed from a clock-seeded stream (now seeded from its floor). Re-recorded: the reference levels, one object for one each wave (the doll's head on 4242/8 now stands on the floor; 77/8 one animator fewer, the teething ring's wobble). A first session's four unfinished batches were lost with their worktrees and restored from git's stash objects and the workers' transcripts |
| P4 | (this commit) | **Wear by use** (`crates/engine/src/shaders.metal`'s `WEAR` block, `Material::wear_use`, `visuals::prop_look::handled`). Every settled prop surface, textured ones too (the too-clean props were all printed or painted: the Halloween things, the triptych, the secretary's veneer), now wears by use, in metres and from what the shader already has: no texture read and no new pipeline variant. Dust lies on what faces up: a patchy grey film that greys a dark top and dulls a pale one, a thin film on the sides, none underneath. The floor's grime reaches a thing's foot up to a mop's tide line 8–18 cm up, with splashes and shoe scuffs above it. Convex edges are rubbed (dark paint to a paler, duller ground, a pale surface greyed by hands) and chipped in flakes, more where a thing is knocked low down, and creases hold dirt. Convexity is the change of the normal along the surface over a pixel, so only a bevel wider than a pixel or two shows it, never as an outline. A thin sheet drawn from both sides (cloth, paper) is never chipped. How much a thing is handled comes from its box (`handled`): furniture on the floor 1, a small thing on the floor 0.7, a thing hung out of reach 0.45. Two blind reviews against the old, each with 20 images (the ten too-clean props and eight common ones close up, two at the census distance), fresh reviewers, both orders: 12–0 with 8 ties, then 10–0 with 10 ties on the final shader. The final shader dropped the chips on two-sided sheets after the first reviewer saw flaking paint on the cloth ghost. Preferred both times: the secretary (three shots), the chair, the wheelchair, the suitcases, the bin, the books and the TV cabinet. Ties: the small Halloween things on the floor (dark and 10–15 cm tall, the wear too small to see in a still frame), the bat on the ceiling, and the triptych (a print on a wall, out of the dust's way), which split by order in both reviews. Bench, recorded: first frame 4242 L4 19.1 → 20.4 ms, L5 20.6 → 21.9 ms, 7 L5 16.2 → 17.4 ms (the `WEAR` pipelines are larger). Build 7 L5 540 → 557 ms, within budget; the Building's floors, with almost no props, moved as much (run to run). Accepted: 15 goldens, the change on props only (the window's sill and blind dusted in the glass frames, the Poolrooms curb and balls, a bin, far furniture); no look anchor moved |

## Open after P3

The blind reviews found small faults on the new side too, none broken:

- **No small prop has a contact shadow** (two reviewers): a doll's shoes,
  a ball or a rake on the floor can look a little lifted. A renderer
  matter (contact shading for props), not a prop's.
- **Glass**: a mirror's highlight blooms (the washroom mirror, the gilt
  mirror; `Patch::Glass`); the atomizer's glass almost vanishes on a
  bright floor.
- **Shapes**: the nail polish brush runs outside the neck; the spider web
  reads as a tangle, not a corner web; the opened suitcase's garment pokes
  through its wall; two umbrella crooks touch; the roe head's muzzle reads
  as a hole straight on; the trout's paired fins are flat cards; the
  elephant's ears are flat panels; the doll's limbs cross in one view; the
  conker husk's valves stand like petals; the paper clips' wire aliases;
  the mask's strap is cable-thick; the bookcase's feet hang in the air.
- **Clean** (resolved by P4): the Halloween things, the triptych and the
  pale secretary now wear by use like the rest. The secretary's wear is
  seen (preferred in both reviews). On the small Halloween things on the
  floor and on the triptych, too little shows in a still frame to tell
  (ties).
- **Placement** (older than P3): a botanical frame hangs in open space on
  5008/9; a primitive leaf decal beside the graduation photo on 11/8; a
  bright speck low on a wall on 555/10. Blender makes no metaball cell
  under 5 mm, so `r03`'s and `t08`'s small metaball parts may be coarser
  than meant.
- **Canon** (P7): lit candles and a lit fire in empty rooms are kept, a
  censer is cold; one rule for what still burns belongs in `LORE.md`.

## What only people can tell

- Does a remodelled prop read as the thing at a glance, in the dark, by
  the lamp? Is anything now too clean, too new or too detailed for where
  it is?
- Does a barricade read as something people built to keep something out
  (or in)? Do players try to get through it?
- Do players use the new verbs, and does using them feel like handling a
  real thing, or like a game's loot?
