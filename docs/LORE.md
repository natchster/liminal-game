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
| Place | An endless building of damp, fluorescent, empty rooms; every exit leads down |
| Pillars | Emptiness, monotony, familiar but wrong, the sound that never stops ([`PILLARS.md`](PILLARS.md)) |
| Levels | Game level *n* ≈ Backrooms Level *n* − 1; one dominant zone names it |
| Zone archetypes | Lobby, Pillar Field, Abandoned Office, the Maze, Utility Halls, Electrical Station, Habitable Zone, Poolrooms, blackout zones |
| Zone purposes | What a part of a level was for: office wing, living quarters, play area, parlour, washrooms, overgrown atrium, maintenance, shrine, party room, or nothing at all |
| Creatures | Smilers, Hounds, the Watcher; the Face (Stairwell only); faceless staff (Showroom); shadows (Borrowed House); the seated man (the Loop) |
| Anomalies | The Stairwell, the Showroom, the Borrowed House, the Loop: at most one per level, once per run |
| Objects | Almond Water, batteries, paper, lore notes |
| People | Elias, M., the field surveys, the Stairwell logs (lore notes only) |

## How canon reaches the player

Lore is never explained; it is noticed. A new piece of canon must use one of
these channels, and the `lore-keeper` reviewer checks it:

- **Architecture and props**: what a place was for, shown by what is left in
  it (zone purposes, set dressing), never labelled beyond the zone caption.
- **Sound**: what is somewhere else, heard before it is seen (signatures,
  steps, the hum, crying).
- **Creature behaviour**: rules that are learnable by playing, never told.
- **Torn notes**: survivors' fragments (`ui/lore.rs`), which hint and never
  explain; an anomaly's notes lie only on its level.
- **Arrival lines**: one quiet line under a level title, giving the *kind* of
  place, never a name, a number or a rule.

Not allowed: tutorials about creatures, maps that explain the place, named
SCP numbers in the game, any narrator.

## Premise

You noclipped out of reality and landed in the Backrooms: an endless building
of damp, fluorescent, empty rooms that is almost, but not quite, a real place.
There is no way back. There are exits, and every exit leads down.

The pillars of the place, in order of importance:

1. **Emptiness.** Level 0 is rooms and nothing else. Clutter belongs deeper,
   where the place has started collecting things.
2. **Monotony.** The same yellow, the same hum, the same carpet. Variety is
   earned by descending.
3. **Familiar but wrong.** Office architecture with no office in it: doorways
   to nowhere, a ceiling that steps down for no reason, a pillar in the middle
   of a room.
4. **The sound never stops**, except where it does, and that is worse.

## The descent

Game level *n* is roughly "Backrooms Level *n* − 1". Each level mixes zones
(see *Places*), but one kind dominates and names the level:

| Game level | Dominant zone | Name on arrival |
|---|---|---|
| 1 | Lobby | THE LOBBY |
| 2 | Hall | HABITABLE ZONE |
| 3 | Tunnels | UTILITY HALLS |
| 4 | Electrical | ELECTRICAL STATION |
| 5 | Office | ABANDONED OFFICE |
| 6+ | rotates | … |

Every level ends at an exit door, unless it is one of the run's *anomaly*
levels (see *Anomalies*). There, the door leads somewhere else, or the whole
level is something else.

Deeper levels rot: more of the floor is grimy, more regions are dark, the fog
is thicker and the hum wanders off pitch. The Lobby leaks into every level. It
never quite goes away.

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

- A level holds **at most one** anomaly, and **none** on level 1 (the Lobby
  stays empty).
- Each anomaly appears **once per run** and never again in it, somewhere
  between levels 2 and 12, never on two levels in a row. A new game plans a
  new run. Continue and restarts keep the plan, because it is derived from the
  run's seed.
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
  field surveys, the Stairwell logs. Press J to read them.
- **Crying** out loud is desperate. Something answers.

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
