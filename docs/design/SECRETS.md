# Marks and secrets

How the way down is hinted, and what is hidden off it: the plan that turns
the level's guides from one loud sign into a hand you learn to read, and
scatters secrets through the floors, small things that change the one who
takes them for the rest of the run, without a word or a look that says how.
The canon it extends is [`LORE.md`](LORE.md) (*M.'s descent*, *Survival*);
the design it changes is [`GAMEPLAY.md`](GAMEPLAY.md) (her trail, the door
found by the place); the pillars it serves are [`PILLARS.md`](PILLARS.md) 1,
2 and 4 above all. The research behind it is
[`../research/wayfinding-and-secrets.md`](../research/wayfinding-and-secrets.md)
(real route marks, guides in games, secrets in level design, real hiding
places and dead-drop signals) and
[`../research/secret-items.md`](../research/secret-items.md) (Alice, the other
world's food, SCP, the Backrooms' objects, creepypasta and charms, games
with hidden effects), summed up in [`SOURCES.md`](../research/SOURCES.md) section 14.

## The work

How this was planned and done (the brief, the analysis, milestones, progress): [`../plan/SECRETS.md`](../plan/SECRETS.md).

## What the research says

Two notes, in [`research/`](../research/README.md). What they changed:

- **Real route marks are spaced by sight and carry little direction.** The
  Appalachian Trail's blazes are rectangles, one in view at a time, a
  doubled blaze before a turn with the upper one offset toward it, and "too
  many blazes" spoil the trail; a scout's stone sign is a stone on a stone
  and a third beside them the way to go; cavers flag with tape and take it
  out; a caver's or diver's personal marker carries an initial or a notch,
  so its owner knows it. Hansel's pebbles show only by moonlight.
- **A mark reads as a person's** by its variety in hand and medium, its
  unevenness, its wear and its owner's quirk (*Left 4 Dead*'s many-handed
  walls; *Half-Life 2*'s lambda by some caches and not others). The yellow
  paint debate: marks the place did not make break the place.
- **Secrets**: notice before you find (a tell seen first), teach the tell
  early and keep it, off the critical path, few a level where searching
  costs (*Doom* about four or five a map, *Half-Life 2* about three a
  chapter). Real hiding places: a ceiling tile, a toilet cistern, behind a
  radiator (Penkovsky's dead drop), taped under a drawer, a vent. A dead
  drop is signalled apart from it: a chalk mark that says "loaded", never
  where; the same mark crossed says "taken".
- **Alice's bottle and cake**: an invitation with no explanation; her
  caution settles nothing; the effect felt in the room, not told; two-sided
  (small fits the door and loses the key); learned in steps (try,
  generalise, predict, the mushroom's half a rule). **The other world's
  food** binds a little (Persephone's seed, a third of the year; Izanami;
  Laura in *Goblin Market*, who after one taste could no longer hear what
  called her). **SCP**: SCP-330's bowl, two free and a third that costs;
  the old SCP-1162, a hole that gives back and takes; SCP-268's cap of
  neglect; SCP-1916's coloured sweets that change the body; SCP-362, whose
  cost comes on removal. **The Backrooms' objects**: Moth Jelly (the blue
  silences your steps), Cashew Water (an Almond Water that is not),
  Liquid Pain (a broken seal is a warning), Royal Rations. **Charms**: the
  ladanka and the omamori (sewn shut, never opened), salt cones by a door,
  the lucky coin worn and not spent, the red envelope on the road.
- **Games**: Isaac's pills (learned once taken), NetHack's naming, Darkest
  Dungeon's quirks (at most five each way), Brogue (good or bad shows
  before what), Harris's "no cyanide" rule (no unknown thing kills), and
  "every bad item gives something". Effects big enough to feel: 6–8 cm of
  eye height, 15–25% of a creature's range or time, 10–15% of a meter.

## The design

### 1. Her marks

Her trail stays the trail of one person going down, but she spares her
paper as she goes (she writes on it), and marks the way with what she has. **Her hand** runs
through every kind of mark: her arrows are drawn open, never filled, the
**left barb longer than the right**, in pencil grey or chalk white, never
black. It is never explained; it is how a player tells hers from the flat
sheets' maker's, and the folded sheet stays her signature.

| Kind | What | Where | Its way | From |
|---|---|---|---|---|
| **Sheet** | A sheet folded twice, a small pencil arrow in her hand, curled and greyer deeper down | The floor of her walk | The arrow | the first floor |
| **Stack** | Three bits of what lies there: a large one, a small one on it, a third beside them the way she went. Concrete chips on the Building's floors, bits of ceiling tile, bottle caps | Open floor (the Building's decks and halls, pillar fields) | The third bit | 1 |
| **Pencil** | Her arrow on the wall, 10 cm, graphite (a dull sheen in the lamp) | A wall beside her way at hip height (0.9–1.1 m), on the side she kept | The arrow | 2 |
| **Turn** | Two short strokes 5 cm apart, the upper one set toward the turn | The wall ahead where her way turns, at hip height | The offset | 3 |
| **Corner** | A torn corner of her paper, 6 cm, its tip on | On the floor against a wall or a jamb | The tip | 5 |
| **Chalk** | A chalk stroke 8 cm long leaning on, the chalk of her tallies | Low on the wall (10–20 cm), the side she kept her hand on | The lean | 5 |
| **Tape** | A strip of reflective tape 8 × 2.5 cm, grey in the room's light, a white glint when the lamp finds it | A wall beside her way, in the dark (unlit tiles) only | None: one after another | 7 |

- **Fewer sheets as she goes down**: sheets are all of her marks on the
  Building's floors but for the stacks and a pencil arrow or two; about
  half on levels 4–6, a quarter on 7–10, one in ten on 11–13. The rest are
  the other kinds, each where it can be (a wall beside the way, open floor,
  the dark).
- **Each in sight of the last, at its own distance.** The trail's rule
  holds for every kind (`lay_sheets`: the furthest step in reach that can
  be seen from the last mark), but a small mark must be near enough to be
  seen at all: a sheet as now (up to the spacing), a stack 12 m, a pencil
  arrow or a turn 9 m, chalk 7 m, a corner 6 m, tape 16 m (in the beam).
  Where no small mark fits, she left a sheet. Small marks lie closer
  together; the trail has more marks deep down, each fainter.
- **Taught by pairing.** Her first mark on every floor is a sheet, in sight
  of the arrival. On the floor where a kind first appears, its first one
  lies with a sheet beside it. Turns are marked only at turns.
- **Never lit, never glowing.** The sheets lose their emissive lift; what
  lights a mark is the place's light or your lamp. Tape is the one kind the
  lamp brings out, as tape does.
- **The flat sheets stay one stamp.** The trails that are not hers are laid
  as no tired person walks: a whole flat page every two tiles (nearer only
  where the next could not be seen), mid-floor, the same heavy black arrow
  every time, filled and even (the over-marking the research names). No
  kind but the page.

### 2. The place answers more quietly

A cry for help that is answered (and the payphone's hint) is answered by
the place, never by a marker put there for you, and not always the same
way:

- **The lights.** Where the way to the door runs under lit panels, they
  stutter one after another along it, away from you toward the door, a
  quarter of a second apart, as far as 30 m (each panel's relay ticking as
  it goes). The way is read from the order.
- **Footprints.** Elsewhere (or where the way ahead is dark), wet
  footprints appear on the floor ahead of you, a pace apart, one after
  another, walking toward the door, sixteen of them, each with a wet step
  where it lands; they dry away over 20 s. Bare feet, nobody's.
- **The door's call.** On the Building's floors nothing walks: there, where
  the lights cannot answer, the door calls at once, louder and heard
  further than its own call.
- No chime, no line on the screen, no glow. The rooms still darken a little
  while it happens (`cry_dim`). The payphone's voice says only where to
  look ("...the lights know the way.", "...look at the floor.", "...listen
  for the door."), and the hint follows from it; where there is no way to
  the door it only whispers.

### 3. The torn notes, quiet

A note lies as paper lies: no halo under it, no beam over it, no stencil on
the wall. A note that continues a story already begun is written in that
story's ink (its colour, the journal's), which only a near look tells. A
note you have read (when every note was read) lies with its corner turned
down.

### 4. Secrets: where things are put away

On each floor a few things are put away off the way: never on her walk, a
room or two off it or at the end of an ordinary dead end, never at a false
trail's end. Each **hiding place** has a tell you can see before you look,
the same everywhere in the run, and a verb in the game's own style; looking
takes a second or two and makes a sound where it is.

| Place | Where | Its tell | Verb |
|---|---|---|---|
| **Ceiling tile** | Under a tiled ceiling (the Lobby, offices, halls) | One tile sits askew, a dark gap along an edge, a chair under it | [E] LIFT THE TILE |
| **Vent** | Low on a wall (offices, utility halls, the electrical station, halls) | A grille hanging by two screws, the other two gone, a wiped arc in its dust | [E] PRISE THE GRILLE |
| **Drawer** | A filing cabinet's drawer (the level's own) | The drawer not quite shut | [E] OPEN DRAWER (something taped under it) |
| **Radiator** | The Building's floors | A rag stuffed behind its fins | [E] REACH BEHIND |
| **Cell** | A cell on the Building's floors, off the way | Two small cones of salt by its door (the salt's own cell) | [E] TAKE … from its bench |
| **The hole** | A wall off the way, at hand height: wallpaper and plaster in the rooms, concrete on the Building's | A fist-wide hole, dark inside, crumbs under it | [E] REACH IN |
| **Off the way** | The end of a dead end, a wet corner, a small table in an empty room, a hook | The thing itself, where nobody would leave it | [E] TAKE / EAT / … |

**The sign.** Some hiding places are signed by a third hand, neither hers
nor the flat sheets' maker's: a small circle with a point in it, 8 cm,
scratched into the paint or the concrete with a key's point (no chalk: her
tallies and her low strokes are chalk), at 1.4–1.6 m on a wall within two
tiles of the place and seen from it (never on the place, never pointing).
It says *something is put away near here*. The same circle scored through
with an X says *taken*: on the run's first floor, where a place there has
a wall to sign, one is found already opened and empty, its circle scored
through. About half the places that are opened are signed; the rest are
found by their tell. Who scratches them is never said. (A circle struck
through had read as "no".)

**How many**: two or three hiding places a floor on the Building's floors,
none on the Lobby but what grows there (pale mushrooms in its wet carpet,
on some runs: its emptiness is the point; no table, no chair under a tile),
two to four on the floors below. Most hold something ordinary (almond
water, a battery, paper; never a coin: one at a time, from the machines)
or nothing; one or two a floor hold one of the things below.

### 5. What is put away

Each thing is a real, small, ordinary object, found where such a thing
could be. What it does lasts the rest of the run: a small change to one or
two numbers the game already has, never told, never shown in a meter's
words. Its look and its words (only what a real one carries: a card on a
dish) say nothing of it. Taking it is felt only as what it is (its taste or
its weight, the line as you take it; a change of size easing in: every
sound or sway of the body is already something else's, the dread's heart,
the Watcher's ringing, the reality shift's swell) and then learned (the
Hound that did not turn, the stamina that ends further on).

| Thing | Where | Draws on | Verb | What stays with you |
|---|---|---|---|---|
| **Pale mushrooms** in the wet carpet | A wet corner (the Lobby's soaked patches, the Poolrooms' edge, an overgrown atrium) | Alice's mushroom; SCP-1916 (Smapti) | Eat | Smaller: the eye 7 cm lower (crouched 5), your steps heard 12% less far, 4% slower |
| **Brown mushrooms** | The same wet corners, from the sixth floor | the same | Eat | Larger: the eye 7 cm higher, steps heard 12% further, 4% faster. One of each: your own size again; at most two steps either way |
| **A white dish of wrapped sweets**, a card *please take one* | A small table in an empty office, lobby or hall (from the fifth floor) | SCP-330 (Dr Kondraki); "Please Take Just One" (Umbrello) | Take (eaten as taken) | The first and the second: nerve refills 10% faster each. The third: your hands shake for the rest of the run (the beam trembles) and a Smiler sees your lamp from 20% further |
| **The hole in the wall** | Off the way | The first SCP-1162 (author unknown) | Reach in | It gives back what you last used up (the bottle you drank, the battery you changed) and takes one thing you carry; with empty pockets it takes from you: the stamina ceiling 15 lower |
| **Red seeds in a twist of paper**, on a saucer | The end of a dead end | Persephone; Izanami; *Goblin Market*; the Lotus-eaters | Eat | The place takes you a little for its own: the dark takes your nerve 20% slower and a Smiler sees your lamp 15% less far, but the door's call carries 20% less far |
| **A cloth bag on a cord, sewn shut** | Behind a radiator, in a cell, behind a vent, under a drawer | The ladanka; the omamori | Put on | The dark takes your nerve 25% slower and leaves you 45, not 40 |
| **A paper sachet of salt** | On the bench of a cell with salt cones by its door; in the rooms, a drawer | Morijio; the Midnight Game's salt circle | Keep | A touch takes 20% less nerve |
| **A jar of blue jelly**, no label | Above a lifted ceiling tile | Moth Jelly (KingofGarbage); Liquid Silence (Atom_Rutherford) | Eat | Your steps heard 15% less far |
| **A ration of grey concentrate**, in a wrapper printed as the Building's coupon is | A cell, behind a radiator: the Building's floors | The Samosbor's food concentrate (ProfessorSacred); Royal Rations (Sariastuff, zakkun_) | Eat | The stamina ceiling 112, but almond water gives 20% less |
| **A small amber medicine bottle**, its label soaked off | Behind a vent grille, in a drawer, above a tile | Alice's bottle with no label; SCP-714 (Arcalane) | Drink | Nerve refills 20% faster, stamina 10% slower |
| **A tweed cap** | On a coat hook in an office wing | SCP-268 (Pair Of Ducks) | Put on | A Hound takes 25% longer to find you standing still; a cry is answered with the way half as often |
| **A coin with a hole, on a string** | Behind a radiator or a vent, under a drawer | The lucky coin worn and not spent; SCP-832 (Inglonias) | Put on | A cry is answered with the way 15 points more often (60% at most), but no machine's coin return rings for you again |
| **An almond water, faintly yellow, its seal broken** | Under a drawer, on a table, in a corner | Cashew Water (yoyoflux; Taangeerine); Liquid Pain's broken seal (ReyDay) | Drink | Stamina now, as almond water; a Hound smells you standing still sooner (2.5 s, not 3) |
| **A red envelope**, a lock of dark hair in it | On the floor of a corridor | The red envelope on the road (Taiwan) | Pick up | The one behind the glass notices your look 25% sooner |

- **Bounded.** Each system moves at most two steps either way, a step
  being the smallest change a thing makes to it (several things together
  stop there: steps 0.76–1.24, nerve's refill at most 1.2, the dark's
  drain at least 0.6, the stamina ceiling 85–112, a Smiler's sight of your
  lamp 0.7–1.2), and nothing may break the fairness the creatures keep: no
  change lowers what the dark leaves you or raises what a touch takes so
  far that the dark and one touch kill (`creature.rs`, E10), a chase stays
  under your sprint as it is (`chase_cap`), and no thing kills. Leaving a
  thing where it is is always a fine choice.
- **The run decides where.** Which things lie on which floors is a
  function of the run's seed (each on one or two floors, the mushrooms of
  both kinds, the dish and the hole on two or three), so each floor stays a
  function of `(run_seed, level)`; where a thing finds no place on its
  floor, it is not there.
- **What the notebook keeps.** The act, never the effect: a page at the
  back, *What I took*, one line a thing in the order taken, what and where
  ("Pale mushrooms, in a wet corner. Level 5."). It answers "did I eat one
  of those?" and nothing else.
- **Felt, then learned.** On taking, one line in the toast's words, naming
  the thing and its taste or feel, never its effect (YOU EAT THEM — SOFT,
  LIKE BREAD). A change of size comes over a few seconds.

### 6. What it must not break

- **Levels are a pure function of `(run_seed, level)`.** The marks are
  planned on the story's stream (her trail's plan is re-recorded once, with
  a reason); the hiding places and what is in them on a stream of their own
  (`SECRET_SALT`), placed inside the build after her trail (gameplay
  objects: the props round them make room), never from `rnd()`. What was
  taken (a retry, a Continue) is fitted afterwards, as her page is.
- **Light and sound belong to places.** No mark glows; the lights' answer
  is the panels' own flicker; every sound of a hiding place, a print or a
  thing comes from where it is, silent until you are within earshot (they
  draw the frame stream).
- **The screen at rest is the place alone.** No meter, icon or line says
  what a thing did; the effects change what is already there.
- **The creatures' fairness** holds whatever is taken (above).

## What only people can tell

- Do players read her marks as hers, and notice the first one that is not
  a sheet? Do they tell the flat sheets from hers, and by what?
- Is the trail too faint deep down, or right: found by those who look?
- Do they find a hiding place by its tell, by the sign, or not at all? Do
  they start lifting every tile (the tell too weak, or the reward too
  strong)?
- Do they notice a change in themselves, and put it down to what they ate?
  Do they eat the next thing, or leave it?
