# The Backrooms — game canon

This is the game's own lore: what is true *in this game*, written in our own
words, ported from the HTML version's canon (`../docs/LORE.md` in the parent
folder, with the research behind it in `../docs/LORE_RESEARCH.md` and
`../docs/SCP_LOCATIONS.md`; read-only references). When a design question
comes up ("would a Hound do that?"), this document is the answer. It is a
living bible: change it when the game's truth changes, in the same commit.

## Canon at a glance

| Kind | Canon |
|---|---|
| Place | The Building: an endless megastructure of concrete with no outside, and inside its walls the Backrooms, damp, fluorescent, empty rooms; every exit leads down |
| Pillars | Emptiness, monotony, familiar but wrong, the sound that never stops ([`PILLARS.md`](PILLARS.md)) |
| Levels | 1–3 the Building's floors (the Well, the Stair Halls, the Works); from 4, game level *n* ≈ Backrooms Level *n* − 4, one dominant zone naming it |
| The Building | Vast floors at many heights over a void: galleries, flights, bridges, cells with hermetic doors; nothing walks there; the self-assembly comes from far off; now and then a figure stands far across the void and turns away |
| Zone archetypes | Lobby, Pillar Field, Abandoned Office, the Maze, Utility Halls, Electrical Station, Habitable Zone, Poolrooms, blackout zones |
| Zone purposes | What a part of a level was for: office wing, living quarters, play area, parlour, washrooms, overgrown atrium, maintenance, shrine, party room, or nothing at all |
| Creatures | Smilers, Hounds, the Watcher; the Face (Stairwell only); faceless staff (Showroom); shadows (Borrowed House); the seated man (the Loop). Not a creature: the figure across the Building's void, which nothing sends and which harms no one |
| Anomalies | The Stairwell, the Showroom, the Borrowed House, the Loop: at most one per level, once per run |
| Objects | Almond Water, batteries, paper, lore notes |
| People | Elias, M., the field surveys, the Stairwell logs (lore notes; M. also by her trail and her camps); the Building's residents, gone behind their doors, and its cleaners, known only by their log and their marks |

## How canon reaches the player

Lore is never explained; it is noticed. A new piece of canon must use one of
these channels, and the `lore-keeper` reviewer checks it:

- **Architecture and props**: what a place was for, shown by what is left in
  it (zone purposes, set dressing), never labelled beyond the zone caption.
- **Sound**: what is somewhere else, heard before it is seen (signatures,
  steps, the hum, crying).
- **Creature behaviour**: rules that are learnable by playing, never told.
- **Torn notes**: survivors' fragments (`ui/lore.rs`), which hint and never
  explain; an anomaly's notes lie only on its level, and the Building's (its
  residents' notices, its cleaners' log, its surveys) only on its floors.
  A notice says what to do, as notices do; it never says why.
- **Her notebook**: one page at M.'s camp on each floor down to level 13
  (`story/notebook.rs`), which hints as a survivor would and never explains.
  The last line of the last page you took stays at the top of the screen
  (the *thread*): a line she wrote, never an instruction.
- **Arrival lines**: one quiet line under a level title, giving the *kind* of
  place, never a name, a number or a rule.

Not allowed: tutorials about creatures, maps that explain the place, named
SCP numbers in the game, any narrator.

## Premise

You noclipped out of reality, through the wall at the back of a storage room,
and came out in **the Building**: concrete without end, galleries round a
void with no top and no bottom, flights of stairs a hundred metres long,
windows in their thousands and a door behind every one shut from inside.
People lived here once; nobody opens now. Far off, now and then, a siren,
and a fog that comes up out of the dark (*The self-assembly*).

Inside its walls are **the Backrooms**: an endless building of damp,
fluorescent, empty rooms that is almost, but not quite, a real place. The
way down from the Building leads into them, and on down. There is no way
back. There are exits, and every exit leads down. Up in the Building the
danger is far off and the place itself is the challenge; down in the rooms
the danger is near.

The pillars of the place, in order of importance:

1. **Emptiness.** The Building is vast and nobody you can reach is in it; the Lobby
   (Level 0) is rooms and nothing else. Clutter belongs deeper,
   where the place has started collecting things.
2. **Monotony.** The same yellow, the same hum, the same carpet. Variety is
   earned by descending. (The Building above is the one exception: the run
   begins in its three places, each unlike the others, and comes down into
   the sameness of the rooms.)
3. **Familiar but wrong.** Office architecture with no office in it: doorways
   to nowhere, a ceiling that steps down for no reason, a pillar in the middle
   of a room.
4. **The sound never stops**, except where it does, and that is worse.

## The descent

The descent begins in **the Building** (levels 1–3, `docs/MEGASTRUCTURE.md`):
vast floors of concrete you find your way down through, at many heights,
over a void. Below them are the rooms: game level *n* from 4 on is roughly
"Backrooms Level *n* − 4", and what rots with depth counts from the Lobby
(`level::building::rooms_depth`). Each rooms level mixes zones (see
*Places*), but one kind dominates and names the level. (The names are kept
in capitals here and in the code; the screen sets them in sentence case,
under "Level *n*" on the level card: `docs/UI.md`.)

| Game level | Dominant zone | Name on arrival |
|---|---|---|
| 1 | The Building | THE WELL |
| 2 | The Building | THE STAIR HALLS |
| 3 | The Building | THE WORKS |
| 4 | Lobby | THE LOBBY |
| 5 | Hall | HABITABLE ZONE |
| 6 | Tunnels | UTILITY HALLS |
| 7 | Electrical | ELECTRICAL STATION |
| 8 | Office | ABANDONED OFFICE |
| 9+ | rotates | … |

Every level ends at an exit door, unless it is one of the run's *anomaly*
levels (see *Anomalies*). There, the door leads somewhere else, or the whole
level is something else.

**Nothing points the way.** There is no compass and no map. The door is the
furthest place from where you arrive, and it is found the way a real door is:
its green EXIT sign and lamps light its corner of the floor whatever the
region (the light carries down a dark hall), and it **calls**, every nine
seconds or so: air drawn under the door and the sign's relay ticking,
heard from a few rooms away and muffled by walls. In a hush it goes quiet
with everything else. The Loop's ordinary office door has neither. Crying
out for help may still answer with arrows on the floor (or with something
else), and so may a payphone.

Deeper levels rot: more of the floor is grimy, more regions are dark, the fog
is thicker and the hum wanders off pitch. The Lobby leaks into every level of
the rooms (not up into the Building's, but round the Works' last door).
It never quite goes away.

## The Building (levels 1–3)

The Building is what the rooms are inside: a megastructure of concrete
with no outside and no sky. Its floors are not rooms but landscapes of
building: walks at many heights, joined by flights of stairs and bridges,
over a void that drops away into fog. Nothing walks there. You find the
way down by the place itself: the door's light far below, the flights
lit where they start, her paper (`docs/MEGASTRUCTURE.md`).

- **The Well** (level 1). The courtyard of an endless panel block: galleries
  round a light well with no top and no bottom, the galleries of every
  storey above and below yours stacked up and down the face of the block,
  a window in every panel, a few lit, and a sealed door every few metres,
  numbered in the thousands, some marked red, some taped round. Knock if
  you like; once in a long while, something knocks back. Washing hangs on
  lines across the well.
- **The Stair Halls** (level 2). A hall of stone with no far wall: piers
  standing up out of the void, flights of stairs at every height, bridges
  between them, stone balustrades. Chains hang out of the dark, and some
  flights cross the void from nowhere to nowhere with no rail; no walk
  reaches one.
- **The Works** (level 3). Floors of dead machinery the size of houses,
  terraces stepping down past shafts, ramps and steel rails painted yellow,
  pipes along the housings, gantries under a ceiling so far up its lamps
  look like stars, most of them out. Its way down is a small wooden door
  in the machine wall at the foot of its tower, the yellow of the rooms
  round it and a warm light over it, and it is warm behind it: the yellow
  rooms, showing through. It is the only place they do.

What every floor of the Building has:

- **A railing on every edge you can walk**, and none on what you cannot
  reach; a lamp at the foot of every flight.
- **Cells**: rooms cut into the building, each with a hermetic door, its
  wheel, its number, a bench, and a lamp in a cage over it. There is always
  one nearer than the fog. Her camp is in one.
- **The door down**, at the foot of its stair tower, whose windows are lit a
  storey apart all the way up: you see its light across the void from where
  you arrive, long before you find the way to it.
- **Storey numbers** painted by the flights (in the thousands on the first
  floor, counting down as you go down), **loudspeakers** on the walls, and
  the residents' notices.
- Its own air: none of the rooms' weathers reach up here.

**Someone across the void.** *After the figures of SCP-3333; see
Attribution.* Nothing walks the Building, but now and then, far off across
the void, someone stands where no walk reaches: on a gallery of another
storey, on a flight that crosses the dark, on top of a machine or on a
gantry under the ceiling. Grey, thin, a little taller than anyone, turned
toward you. Look at it a moment and it turns its back; look away and it is
gone, and if you watch it a long while, or come nearer, it is gone anyway.
Long after, it stands somewhere else, always across the void, and never
while the siren sounds or the fog is up (a figure standing when the siren
starts turns its back and goes). It never comes near and it does nothing:
it is not one of the things that live here, nothing sends it, and it does
not announce itself, because it never comes. Nobody writes about it.

**The lift.** In the Well, a lift cage hangs in the open shaft on its
cables, stopped between storeys, its bulb still lit. Nothing reaches it.

## Places (zone archetypes)

Each level is cut into rectangular zones. Every zone has its own ceiling height,
wall thickness, doorways, materials and props.

- **The Lobby** (Level 0). Mono-yellow wallpaper and wet carpet. The fluorescent
  tubes sit over every other ceiling tile and the ceiling is low (2.7–3.05 m).
  Thin partitions stand in open floor, full of gaps. About half the gaps are
  doorways with jambs and a lintel. There are lone pillars, and almost nothing
  else. The emptiness is the content.
- **Pillar Field** (Level 0's pillar grid). Huge rooms with a lattice of
  columns, a higher ceiling (3.2–3.7 m) and the same wallpaper.
- **Abandoned Office** (Level 4). Low ceilings (2.5–2.75 m), drywall and
  carpet tiles. Small rooms with one doorway each, and chest-high cubicle
  partitions you can see over, which means anything else can too. Office
  debris.
- **The Maze** (deep rot). The old thick-walled maze: 4 m walls and braided
  corridors. This is where the Backrooms keeps what it has collected: toys,
  party leftovers, portraits, candles, things from people's lives.
- **Utility Halls** (Level 2). Shoulder-wide corridors (1.6–2.8 m) in poured
  concrete, with walls that bulge into the passage and a low ceiling
  (2.35–2.65 m). Industrial debris. The echo answers a beat late.
- **Electrical Station** (Level 3). The same bones in old brick, tighter and
  more tangled, with wiring behind every wall.
- **Habitable Zone** (Level 1). Poured-concrete halls 5.4–7.6 m tall, columns
  on a wide grid, fluorescent fixtures hanging on rods, puddles and a long
  reverb.
- **The Poolrooms** (Level 37, rare and deep). White tile everywhere, too many
  pillars. Acoustics that cut off too early.
- **Blackout zones** (any level). The lights are dead and there is no hum and
  almost no reverb. Only your breathing.

Where two zones meet, the ceiling steps between their heights with a bulkhead.
Unlike zones are usually separated by a wall with a few openings.

**Every area has a purpose** (`world/purpose.rs`): what it was for before the
place took it — an office wing, living quarters, a play area, a parlour,
washrooms, an overgrown atrium, a maintenance run, a shrine, a party room, or
nothing at all. The purpose decides what is left lying there, and its
wallpaper, grime and darkness change at its walls. Most of the Lobby is for
nothing.

## M.'s descent

Someone came this way before you, and you can follow her down.

- **E.** (Elias, whose loose pages are among the torn notes) went into the
  Backrooms first, and stopped answering. **M.** (who wrote *M.'s Rules*)
  went in after him, followed his marks down, and left her own. Neither of
  them is ever named in full, and nothing says how they are related; the
  pages let you work it out.
- **Her trail.** From where you arrive, sheets of paper folded twice, each
  with a pencil arrow, lie along the way she walked: to her camp, then on to
  the door. Each one can be seen from the last, but only just, and they get
  sparser as you go down (every three tiles of walk at first, every five
  deep down); from level 4 a stretch after her camp has lost its sheets
  (two from level 7), and you have to find where she went.
- **Trails that are not hers.** From the floor of her page that says *not
  all the paper down here is mine* (the sixth of her floor pages), a trail
  of whole, flat sheets with heavy black arrows leaves hers and goes into a
  dead end; two from the floor of her seventh. Something is waiting at the
  end: you hear it breathe close by in the dark, and something is sent for
  you, or the one already coming learns where you are. Hers are folded
  twice; nothing says so but her first page. Who lays the flat sheets is
  never said.
- **Her camp.** On every floor, somewhere off the straight way, against a
  wall (on the Building's floors, in a cell): a bedroll with a blanket
  thrown over it, her pack, a candle lantern
  still burning (it lights its corner, and light draws Smilers), empty
  bottles, a count of days scratched on the wall, and what she could spare:
  almond water, and from level 3 a battery.
- **Her notebook.** At each camp lies one page torn from it, for that floor:
  thirteen in all. Eight follow her down the ordinary floors in turn: the
  first three on the Building's floors and the fourth on the Lobby (always
  levels 1–4), the rest wherever the run's anomalies leave the ordinary
  floors (so no page names a zone or a level); each anomaly's floor has its own page (the
  Stairwell's and the Loop's are written on the floor whose door leads in,
  before she went through); the last lies at **the last camp, on level 13**,
  the first floor below every anomaly. Take it (E) and it is in the notebook
  (Tab), in the order of the descent, a torn edge for each floor whose page you
  left; its last line stays with you on screen.
- **The last camp** is on level 13. Her lamp there went out long ago; her
  shoes stand side by side at the foot of the bedroll, and her trail goes no
  further than her camp: from there you find the door alone. Leaving that
  floor (by its door, a lift, a hole) ends her story: a card over black says
  how many of her pages you carry, and you take what is left of her paper
  (twelve sheets). The descent goes on, without her: below the last camp
  there are no camps and no trails but the ones you drop.
- What the pages let you work out, and never say: E. went in through the
  wall at the back of a storage room (the loose note *Day 1* is his), into
  the Building, and stayed a long time on its third floor (her page there
  finds where; his own notes say "level three"); she
  counts sleeps and calls them days (the tallies at her camps); the
  kitchen in the Borrowed House was theirs; he went into the Loop and she
  thinks he is still going round. Whether the man with no face in the Loop
  is E. is never said, and must stay unsaid.
- Her trail and her camp are part of the place, the same on every visit;
  they are where they are because of where she went (`crates/game/src/story/`).

## Things that live here

The rules are learnable, and learning them is the game.

- **Smilers.** Tall, starved and black, stooping under the ceiling, with arms
  that hang past its knees, hands too big, and legs that bend the wrong way.
  In the dark all you see is the face: two small white eyes over a crescent of
  white teeth that runs round to its cheeks. They lurk where it is dark and are
  **drawn to light**: a lit lamp is seen from much further off. **Hold its gaze
  and back away slowly and it barely creeps. Turn your back on it, or run, and
  it lunges**, thrown forward, reaching. It is slowed in bright rooms, where
  it shies from the light. You hear slow wet breathing, and teeth when it is
  coming.
- **Hounds.** Starved people on all fours, rump high, the head hanging under a
  curtain of wet black hair, a jaw too long for the face. In the dark: two
  small white points, knee high, behind the hair. One of them limps. They
  **hear footsteps from rooms away**, at almost twice anyone else's range.
  **Stare one down and it stops for a breath or two**, crouched and growling,
  though not often. You hear a bark when it starts, then panting.
- **The Watcher.** Someone standing under an old dust sheet, the same sheets
  that cover the furniture, the shape of a face pressed into the cloth. It
  **only moves when you are not looking**, and after long enough unseen it is
  simply nearer; each time, it is standing differently. Touching it doesn't
  kill you, but it takes your mind. It makes no sound. Close to it, your ears
  ring.
- **The Face** (the Stairwell only). See below.

**Pressure has a shape.** A pacing director (`entities/director.rs`) decides
when one of them comes for you: a calm, a build-up where the place announces
it (its steps somewhere near, lights failing, then silence), a peak, and then
relief: after a close call it **backs off** and the level goes quiet for a
while. Only one comes at a time. The Watcher only moves on you when it is the
one sent; otherwise it keeps its distance.

**Taking her page wakes the place.** Her lamp gutters out as you lift the
page, and whatever quiet was left ends within a few seconds: something is
sent. You choose when it starts, by when you go to her camp. On the first
floor of the rooms (the Lobby, level 4) the first quiet is long (two
minutes and more), so there is time to find her before anything comes.
Nothing that hunts lives on the Building's floors above (levels 1–3): what
comes there comes from far off (`docs/MEGASTRUCTURE.md`).

## The self-assembly

*Inspired by the Samosbor (самосбор, "self-assembly"), an anonymous
Russian imageboard myth begun in 2018; see Attribution.*

Nothing walks the Building's floors. What comes there comes from far off,
now and then, and every floor of the Building knows it (`world::assembly`):

- **The siren.** It wails from the loudspeakers on the walls, rising, held,
  falling, every few seconds; you hear the nearest ones.
- **The fog.** Out of the void, far below, a violet fog rises, its surface
  seen from above, the lamps under it going out one by one. It comes up
  among the walks, stands a while at its height (some times over every
  walk, some times not), and sinks away; the all-clear is one long steady
  tone.
- **The cells.** Shelter is a cell: a room cut into the building with a
  hermetic door, a lamp in a cage over it, always the same shape. Inside,
  turn the wheel and the door is shut; it does not come in, though
  something knocks at the door while it is up. Turn the wheel again to go
  out. There is always a cell nearer than the fog.
- **In it**, your mind goes fast, and then you are gone. With your eyes
  under its surface it is all you see.
- **It is never explained.** Who built the cells, who put up the notices
  and what the fog is, nobody says; the residents' notices tell you what to
  do, as notices do, and call it a *self-assembly* (the only place its name
  reaches you). Somebody administers the Building (its ration coupons, its
  notices: "The Building cares for you"), and somebody cleans after the fog:
  the cleaners, known by their log and the red marks on doors that did not
  answer, and never seen. The storeys they poured are high over the first
  floor, above every gallery you can see.
- Taking her page on a floor of the Building sounds the siren within
  seconds, as it wakes the place below. The first quiet on the first floor
  is long, so her camp can be found first.

## Anomalies

The Backrooms have swallowed other places too, the kind the SCP Foundation
files as buildings and rooms (researched in the HTML version's `SCP_LOCATIONS.md`:
315 of them, Euclid and Keter). Four have found their way in:

| Anomaly | What it is | Inspired by |
|---|---|---|
| **The Stairwell** | The exit door opens onto stairs going down for ever | SCP-087 |
| **The Showroom** | The whole level is a furniture store that never closes | SCP-3008 |
| **The Borrowed House** | Rooms taken from other buildings; shadows live in them | SCP-1983 |
| **The Loop** | The exit door opens onto a line of rooms that comes back round | SCP-970 |

The rules of a run (`crates/game/src/level/scp.rs`):

- A level holds **at most one** anomaly, and **none** on the Building's
  floors or the Lobby (levels 1–4: the Lobby stays empty).
- Each anomaly appears **once per run** and never again in it, somewhere
  between levels 5 and 12, never on two levels in a row. A new game plans a
  new run. Continue and restarts keep the plan, because it is derived from the
  run's seed.
- **Arriving** on one of the Building's floors shows one quiet line of its
  kind too: *Washing hung across the drop. Nobody came for it.* (the Well),
  *Stairs that climb past the last of the light.* (the Stair Halls),
  *Everything here was switched off, a long time ago.* (the Works).
- **Arriving** on an anomaly's level shows one quiet line under the level
  title and plays one sound. The line gives the *kind* of place (a chime and
  the smell of sawdust, rooms that do not remember the same house, a door
  clicking shut again and again, someone crying far below). It never gives a
  name, a number or the rules.
- The level's own **torn notes** lie on it, and only on it. They hint, as
  survivors would, and never explain.

## The Showroom

*Inspired by SCP-3008; see Attribution.*

The level is a flat-pack furniture store that has become a building with no
outside: room-sets behind shop partitions under a high warehouse roof, bed
after sofa after kitchen, price tags with names nobody can pronounce, racking
aisles, trolleys left where someone gave up. It is spotless and brightly lit.
A little music plays from somewhere.

- **By day** the staff browse the floor with you. They are too tall and have no
  faces, and when you come close they stop and look at you. That is all they
  do.
- **Closing time.** Boards on posts show OPEN and the store's clock. When the
  clock nears nine, a chime sounds, then a voice through a tinny horn, then a
  relay thunks and every light in the building goes out. The boards read
  CLOSED.
- **By night** the staff hunt. They hear footsteps from a room away, and a lit
  lamp gives you away from across the floor. In the dark, standing still, they
  have to be right on top of you. **Crouch under a display bed or inside a
  wardrobe** and they cannot find you at all.
- The **arrows** on the floor only ever point towards the way out, and only
  while the lights are on. The night lasts under a minute; the lights come
  back with a chime.

## The Borrowed House

*Inspired by SCP-1983; see Attribution.*

No two rooms belong to the same building: a lobby opens onto a tiled bath, then
a concrete hall, then someone's office. The zone captions still name each
room's kind of place, which makes it worse.

- **Shadows** live here: thin figures with no faces. **While you are lit** (your
  lamp on, or standing in a lit room) they have hard edges, they are fast, and
  their touch is the end. **In the dark** they are smoke: slow, blind after a
  few seconds, and their touch only leaves you cold (a large sanity loss).
  About half the rooms are dead, and those are where you are safe.
- Somewhere near the middle is **the nest**: every lamp in the house heaped in
  one room and burning, with a heartbeat that grows as you come near. Put the
  lamps out one by one. Each one that goes out tells the shadows where you
  are.
- **The front door** (the level's exit) will not latch while the nest is lit.
  When the last lamp goes out, the shadows come apart and, somewhere, a door
  settles into its frame.

## The Loop

*Inspired by SCP-970; see Attribution.*

On its level the exit is not an exit. It is an **ordinary office door**, no
sign over it, and it lets you into the third of five small rooms in a row:
yellowed paper, carpet tiles, one fluorescent panel, a steel desk with an old
monitor, two chairs, a filing cabinet, a snake plant in a pot, a clock that
says ten past three. Each room has a doorway front and back, all on one line,
and through them the rooms go on, and on. The side door you came in by is not
there when you turn round. The message on arrival says *straight ahead*.

- **It comes back round.** Walk out of the fifth room and you are in the first
  one again. A door latch clicks, softly, every time. **Count the clicks.**
- **Every lap forward is a little wronger**, and it adds up:
  1. the paper and the light turn a shade greener;
  2. the chairs have turned to watch the doorways, and the plant is on the
     other side of the room;
  3. the lights stutter, and the hum drops;
  4. the light goes **red**, the paper rots, and there are dark stains on the
     carpet and running down the walls;
  5. **someone is sitting in the third room**: a man in a shirt and tie with
     nothing where his face should be. He never moves while you can see him.
     Look away and look back, and he is **nearer**, chair and all. Do not let
     him reach you: his touch takes your mind very fast;
  6. and on: the fog thickens, the furniture is gone, the lamps die one by
     one, and **your mind drains**, faster every lap.
- **Walking back undoes it.** Every lap back is a little more right, and it
  never gets *more* right than normal. Only where nothing has drifted, **in the
  first room, there is a second door** in the side wall with a small green EXIT
  sign. It leads on to the next level.
- **Greed pulls the wrong way.** Almond Water and batteries only lie in the
  rooms from the third lap on, on the desks and the filing cabinets, and on the
  floor once the rooms are bare.
- The calm laps settle you. The room sound grows by one step every lap, as if
  the rooms were getting bigger, and past the sixth the hum is almost gone.

## The Stairwell

*Inspired by SCP-087; see Attribution.* It is one of the run's anomalies: its
steel door is the exit of a single planned level per run.

Some exits are a steel door made up to look like a janitor's closet, with a
plaque that says STAIRS. Behind it:

- Unlit concrete stairs, **thirteen steps to a flight at 38°**, then a
  **half-round landing** and a half turn, over and over. A handrail runs along
  the outer wall. The door **locks behind you**.
- **Your lamp shows nine steps and no more**, however strong it is. The dark
  swallows the rest, and you can see about a flight and a half. The lamp
  drains more slowly here, but it still drains.
- **Somebody is crying far below.** A child, pleading *please*, *help*, *down
  here*. **It never gets closer**: it is the same distance below you on every
  landing. It only gets louder while the Face is out.
- **The Face.** Pale and head-sized, a real human face with no edges: it fades
  into the dark past the temples and jaw. Its eyes are open under heavy lids,
  blank and smooth, nothing in them; the nose has no nostrils, and where the
  mouth should be the skin runs on unbroken. It never blinks. It waits on the
  landing below, at the edge of the light, and stares. It
  **jerks nearer** by half a metre, then by nearly a metre. **It never chases.**
  Walk towards it and it is gone.
- **Do not turn back.** Once you are deep, climbing back up puts it *behind
  you*. When you turn round, it is thirty centimetres from your eyes.
- Earlier explorers left **blue markers every ten landings**, and most of them
  are dead. Deeper down there is a puddle nobody explains, a glassy gouge in a
  step, and a hole in the wall with a light far inside that blinks every two
  seconds. **Knocking** starts on the door far above once you are deep enough.
- There is a door at the bottom, and it leads to the next level. The descent
  gets longer each time (8 flights, then 13, 18… up to 40).

## Survival

- **Almond Water** restores stamina when you drink it. Carry it for when things
  get bad.
- **Batteries** recharge the lamp. The lamp draws Smilers.
- **Paper** marks your path. Sometimes you find someone else's trail going the
  other way.
- **Lore notes** are torn pages left by those who came before: Elias, M., the
  field surveys, the Stairwell logs. The notebook (Tab) keeps them, typed,
  after hers.
- **Her pages** are at her camps, one a floor; the notebook keeps them first,
  in the order she wrote them.
- **Crying** out loud is desperate. Something answers.
- **Cells**, on the Building's floors: shut yourself in when the siren goes,
  and wait for the all-clear (*The self-assembly*).

## The sound of the place

- The lamps hum at **120 Hz, twice the mains frequency**, with a full harmonic
  series and a buzzy crackle chopped at the same rate. The hum comes from the
  nearest live fixture. It drifts, and now and then **swells until your teeth
  ache**.
- Each kind of place has its own room sound: dead over wet carpet, long and
  bright in concrete halls, flutter echo in tunnels and the Stairwell, and in
  the Poolrooms a reverb that stops too soon.
- Footsteps follow the floor: squelching wet carpet, dull carpet tiles, gritty
  concrete, hard tile, hollow stair treads.
- The Building sounds as big as it is: a long, sparse tail over the void in
  each of its places, the hum of the lamps over the cells, and the siren
  from its loudspeakers when it comes.

## Attribution and licensing

- **The Stairwell and the Face** are inspired by **SCP-087** by **Zaeyde**
  (<https://scp-wiki.wikidot.com/scp-087>), licensed **CC BY-SA 3.0**. We use
  our own names and text, but the SCP Wiki treats even the concept as a
  derivative work. The Stairwell (`crates/game/src/modes/stairwell.rs`, the Stairwell parts of
  this document and the Stairwell lore notes) should be treated as
  **CC BY-SA 3.0** and keep this attribution.
- **The Showroom** is inspired by **SCP-3008** "A Perfectly Normal, Regular Old
  IKEA" by **Mortos** (<https://scp-wiki.wikidot.com/scp-3008>), **CC BY-SA
  3.0**. `crates/game/src/anomalies/store_kit.rs`, the Showroom parts of `anomalies/mod.rs`, its notes and its
  sections here are CC BY-SA 3.0 and keep this attribution. The game uses no
  brand name: it is a generic flat-pack store.
- **The Borrowed House** is inspired by **SCP-1983** "Doorway to Nowhere" by
  **DrEverettMann** (<https://scp-wiki.wikidot.com/scp-1983>), **CC BY-SA 3.0**.
  `crates/game/src/anomalies/house.rs`, its notes and its sections here are
  CC BY-SA 3.0 and keep this attribution.
- **The Loop** is inspired by **SCP-970** "The Recursive Room" by **Freudian**
  (<https://scp-wiki.wikidot.com/scp-970>), **CC BY-SA 3.0**. We use our own
  names and text. `crates/game/src/modes/loop_rooms.rs`, its notes
  and its section here are CC BY-SA 3.0 and keep this attribution.
- **The self-assembly and the Building** take their motifs from the
  **Samosbor** myth (самосбор; begun anonymously on 2ch's /b/ in spring 2018
  and written since by "everyone and no one", with no single author or
  licence; `SOURCES.md` section 10): the endless panel block with no
  outside, the siren, the fog, the hermetic door, the knocking at it, the
  cleaners after. We use the motifs and our own words, and no named
  character or story from any author. The Building's places also take from
  **Backrooms Level 188** "The Courtyard of Windows" by **Kai4C**
  (<https://backrooms-wiki.wikidot.com/level-188>), **SCP-2723** "The
  Prophet's Tower" by **HotCocoaNerd** (<https://scp-wiki.wikidot.com/scp-2723>),
  **SCP-4370** "Post-Industrial" by **UraniumEmpire**
  (<https://scp-wiki.wikidot.com/scp-4370>), **SCP-5778** "Hidden In Plain
  Sight" by **MarcusLCrassus** (<https://scp-wiki.wikidot.com/scp-5778>) and
  **SCP-3333** "Tower" by **Jekeled** (<https://scp-wiki.wikidot.com/scp-3333>),
  all **CC BY-SA 3.0**: `crates/game/src/level/building.rs`,
  `crates/game/src/world/building.rs`, `crates/game/src/world/building_dress.rs`,
  `crates/game/src/world/assembly.rs`, `crates/game/src/world/far_figures.rs`,
  `docs/MEGASTRUCTURE.md` and the
  Building's notes are CC BY-SA 3.0 and keep this attribution. Piranesi's
  *Carceri* (1750) is in the public domain.
- The research catalogue `SCP_LOCATIONS.md` (HTML version's docs) summarizes 315
  SCP Wiki articles in our own words, each with its author and link
  (CC BY-SA 3.0).
- **Backrooms** lore comes from the Backrooms wikis (Wikidot and Fandom),
  **CC BY-SA 3.0**. The zone archetypes, Smiler and Hound rules and Almond
  Water are adapted, not copied.
- The original 2019 yellow-room photograph was released **CC0** by the
  building's owner. We draw our own textures and use none of it.
- **Kane Pixels'** *The Backrooms* series is copyrighted and was used only as a
  tone reference. Nothing is taken from it.
