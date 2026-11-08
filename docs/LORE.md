# The Backrooms — game canon

This is the game's own lore: what is true *in this game*, written in our own
words. When a design question
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
| Creatures | Smilers, Hounds (blind, they hunt by sound), the Watcher, the one in the corner (placed, never sent), the Reader (it must stop for paper), the one behind the glass (placed, never sent; it comes only for a long look out of a window); the Face (Stairwell only); faceless staff (Showroom); shadows (Borrowed House); the seated man (the Loop). Not a creature: the figure across the Building's void, which nothing sends and which harms no one. Redesigned (`ENTITIES.md`) |
| Anomalies | The Stairwell, the Showroom, the Borrowed House, the Loop: at most one per level, once per run |
| Objects | Almond Water, batteries, paper, lore notes; coins (the vending machines' coin returns); and a few things put away off the way, marked now and then by a small scratched circle, whose effect stays with you and is never said (*What is put away*) |
| Things in the rooms | The vending machine: a glass-front snack machine against a wall on most floors of the rooms (none on the first, the Lobby of level 4, nor in the Showroom or the Borrowed House), still lit or long dead, with rules of its own (*The vending machine*, `VENDING.md`). Not a creature: it never comes after you and never kills. The windows: below the Lobby, where an office, a hall, a party room or someone's rooms would have had them (none in the Showroom or the Borrowed House), every one looking out on the same field in fog with a streetlamp close outside, most with a blind you can let down (*The windows*, `WINDOWS.md`). Below the Lobby, boxes, bins, books, lockers left unlocked, a dead lift's button and the lit television can be used (*Within reach*) |
| People | Elias, M., the field surveys, the Stairwell logs (lore notes; M. also by her trail and her camps); the Building's residents, gone behind their doors (a few still answer a knock, never opening: a thing pushed out, a folded note), and its cleaners, known only by their log and their marks |

## How canon reaches the player

Lore is never explained; it is noticed. A new piece of canon must use one of
these channels, and the `lore-keeper` reviewer checks it:

- **Architecture and props**: what a place was for, shown by what is left in
  it (zone purposes, set dressing), never labelled beyond the zone caption.
  A thing the rooms took carries the fixed printing a real one of its kind
  carries about itself, word for word (a vending machine's prices, its PUSH,
  its warning label): that is dressing, not the place speaking, even where
  it says why. Never a word about the place, its creatures or its
  anomalies; and what a display says as the thing acts (MAKE ANOTHER
  SELECTION) is its behaviour, not dressing.
- **Sound**: what is somewhere else, heard before it is seen (signatures,
  steps, the hum, crying).
- **Creature behaviour**: rules that are learnable by playing, never told.
- **Torn notes**: survivors' fragments (`ui/lore.rs`; their texts are
  `LORE_ENTRIES` in `ui/hud.rs`), which hint and never
  explain; an anomaly's notes lie only on its level, and the Building's (its
  residents' notices, its cleaners' log, its surveys) only on its floors.
  A notice says what to do, as notices do; it never says why.
- **A resident's note**: on the Well, a folded note slid under a sealed door
  that answered a knock (`world::knock`), read as it is picked up: a word of
  the way from that door in a resident's hand (the nearest cell by its
  plate, or the way down: which way from the door, how many flights, the
  storey there), and at most a line of what to do; never why, never a map.
- **Her notebook**: one page at M.'s camp on each floor down to level 13
  (`story/notebook.rs`), which hints as a survivor would and never explains.
  The last line of the last page you took stays at the top of the screen
  (the *thread*): a line she wrote, never an instruction.
- **Your own page**: the notebook's page after the torn notes, of what you
  took (*What is put away*), in your hand (a ballpoint's blue on plain,
  whiter paper, never her pencil on ruled): what and where and the floor,
  never what it did.
- **What you take**: one line as you take a thing put away (or reach into
  the hole): what it is and its taste or feel, never what it does, never
  why.
- **The payphone**: a voice on a ringing line, a few words: a whisper, where
  to look, or worse; never why.
- **Arrival lines**: one quiet line under a level title, giving the *kind* of
  place, never a name, a number or a rule.

Not allowed: tutorials about creatures, maps that explain the place, named
SCP numbers in the game, any narrator.

## Premise

You noclipped out of reality, through the wall at the back of a storage room,
and came out in **the Building**: concrete without end, galleries round a
void with no top and no bottom, flights of stairs a hundred metres long,
windows in their thousands and a door behind every one shut from inside.
People lived here once, and a few still do, behind doors they never open. Far off, now and then, a siren,
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
out for help may still be answered by the place, never by a marker put
there for you: the lit
panels over the way to the door stutter one after another away from you
toward it, or wet bare footprints, nobody's, walk ahead of you toward it a
pace apart and dry away (on the Building's floors, where nothing walks,
the door calls at once instead, louder than its own call); or something
else answers. A payphone may say where to look, and the place shows the
rest.

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
  numbered in the thousands, some marked red, some taped round (*Knocking*,
  below). Washing hangs on lines across the well.
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

**Knocking.** *The sealed door and the knock at it are the Samosbor's; see
Attribution.* Knock at a sealed door on the Well's galleries if you like
(`world::knock`; the only residents' doors are there, about ninety to a
floor). Most stay silent. In the quiet, four of a floor's ordinary doors
(numbered, nothing else on them) answer, once each, and nobody opens:
someone comes to the door, a hatch at its foot opens and a bottle of almond
water or a battery is pushed out onto the gallery, or a folded note is slid
under it, a resident's word of the way from that door, in one of three
hands: the nearest cell by its plate, or the way down, which way from the
door, how many flights down or up, and the storey there. What was taken
from a door stays taken for the run. A door the cleaners marked red answers
wrong: it knocks back in your own rhythm, then is still for the rest of
that visit (after a death or a Continue it knocks back again), and the
siren sounds within seconds (three to six after it knocks back). A door
taped round never answers. While the siren sounds or the fog is up nobody
answers at all; a knock just before the siren is not answered either (the
hatch is never opened while it is up), and that door may answer another
time. Nobody says why: the residents' notice says not to knock where the
cleaners have marked and to answer no knock until it is over; the cleaners'
log has one of them knock at a marked door by mistake, and the knock come
back the way he had knocked, and the siren; the surveyor of the Well once
had something left on the step of a door that did not open, and never
heard anyone behind the tape.

**Someone across the void.** *After the figures of SCP-3333; see
Attribution.* Nothing walks the Building, but now and then, far off across
the void, someone stands where no walk reaches: on a gallery of another
storey, on a flight that crosses the dark, on top of a machine or on a
gantry under the ceiling. A person in a long grey work coat, bare-headed,
thin, a little taller than anyone, standing still with its arms at its
sides, turned toward you. Look at it a moment and it turns its back; look
away and it is gone, and if you watch it a long while, or come nearer, it is
gone anyway. Long after, it stands somewhere else, always across the void,
and never while the siren sounds or the fog is up (a figure standing when
the siren starts turns its back and goes). It never comes near and it does
nothing: it is not one of the things that live here, nothing sends it, and
it does not announce itself, because it never comes. Nobody writes about
it.

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

**Barricades.** Now and then a doorway or a corridor of the rooms is closed
(never on the Building's floors, nor on a floor with rules of its own, nor
in a pool hall; one at most on the Lobby): boarded with sheets of plywood
braced and bolted through, or boards nailed across, or what the place had
shoved into it, an office's desk on end, its filing cabinets and chairs, a
store room's pallets and shelving, a home's mattresses, a door off its
hinges and a dresser. It fills the opening wall to wall and nearly to the
ceiling, and nothing passes it: not a body, not a look, not a creature's
way to you nor what it hears of you, not the place's answer to a cry. None
stands near where a floor begins, shuts the way to the door or shuts away
anything there is to pick up. Somebody built them; who, and against what,
is never said.

## M.'s descent

Someone came this way before you, and you can follow her down.

- **E.** (Elias, whose loose pages are among the torn notes) went into the
  Backrooms first, and stopped answering. **M.** (who wrote *M.'s Rules*)
  went in after him, followed his marks down, and left her own. Neither of
  them is ever named in full, and nothing says how they are related; the
  pages let you work it out.
- **Her trail.** From where you arrive, her marks lie along the way she
  walked: to her camp, then on to the door. At first they are sheets of
  paper folded twice, each with a small pencil arrow; she spares her paper
  as she goes down (she writes on it), and marks the way with what she
  has: three bits of whatever lies there, stacked, the third set beside
  them the way she went (concrete chips on the Building's open decks,
  where a sheet would blow off into the void; bits of ceiling tile, bottle
  caps below); her arrow in pencil on a wall at hip height (from the
  second floor); two short strokes on the wall where her way turns, the
  upper set toward the turn (from the third); a corner torn off a sheet,
  its tip on, and a chalk stroke low on a wall, leaning on (from the
  fifth); a strip of reflective tape on a wall in the dark, which only a
  lamp brings out (from the seventh). Most of her marks are sheets on the
  Building's floors, about half on levels 4 to 6, under a third on 7 to
  10, few at the bottom. One
  hand runs through them all: her arrows are drawn open, never filled, the
  left barb longer than the right. Nothing says so. None of them glows.
  Each can be seen from the last, but only just (a small mark lies nearer
  the last than a sheet does), and they get sparser as you go down (every
  three tiles of walk at first, every five deep down); from level 4 a
  stretch after her camp has lost its marks (two from level 7), and you
  have to find where she went (`docs/SECRETS.md` 1).
- **Trails that are not hers.** From the floor of her page that says *not
  all the paper down here is mine* (the sixth of her floor pages), a trail
  of whole, flat sheets leaves hers and goes into a dead end; two from the
  floor of her seventh. They are laid as no tired person walks: one in the
  middle of the floor every two tiles, the same heavy black arrow stamped
  on every one, filled and even. Something is waiting at the end: you hear
  it breathe close by in the dark, and something is sent for you, or the
  one already coming learns where you are. Hers are folded twice, and
  drawn by a hand; nothing says so but her first page. Who lays the flat
  sheets is never said.
- **Her camp.** On every floor, somewhere off the straight way, against a
  wall (on the Building's floors, in a cell): a bedroll with a blanket
  thrown over it, her pack, a candle lantern
  still burning (it lights its corner), empty
  bottles, a count of days in chalk on the wall, and what she could spare:
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

- **Smilers.** Very tall, starved down to tendon, black all over as if
  tarred, stooping under the ceiling, the hands hanging past the knees. Its
  lips have dried back off its teeth, top and bottom, so it seems to grin
  round to its cheeks: a rictus, not a smile. In the dark you see nothing
  until your lamp finds it: two gold points of eyeshine set close (your lamp
  given back; they never glow on their own) and under them the pale arc of
  bare teeth, only as far as the beam reaches. Under the panels it is a
  shape too tall and too still, not a face. **Your lamp is what it sees**,
  from far off down any clear line; your body only close, and only lit.
  **Put the lamp out and keep still (or crouch), and it loses you** within a
  few seconds: it goes on, slower, to about where your light was. **Never
  run near it**: running within a few metres makes it lunge, thrown forward,
  reaching. It is slow in bright rooms, and coming at you into one, but the
  light does not drive it off. You hear a click of teeth when it shifts, its
  hiss when it is sure of you, slow wet breathing close by, and nothing when
  it waits. (After Backrooms Entity 3, Wikidot, CC BY-SA 3.0.)
- **Hounds.** Old people on all fours, starved, rump high, the head hanging
  so low you see only the back of the skull under thin, wet, matted hair; a
  vest and long johns gone the colour of the floor. One leg drags. They are
  **blind**: close up, the eyelids have grown shut, and in the dark there are
  no eyes at all. They **hear your steps along the way they would walk to
  you**, through doorways and not through walls, twice as far as anything
  else, and less well while they are moving themselves: their own nails on
  the carpet cover yours. **When the scratching stops, it is listening**: it
  stops in a crouch with its head cocked, and if you are still it comes to
  look near where it heard you; if you move, it is sure. Standing right
  beside one for long, it smells you. Sure, it bays and runs at you, not as
  fast as you can run, and stops after a while to listen again: quiet then,
  and it has lost you. Close, it growls and springs. You hear its nails, a
  slow wheeze, sniffing when it searches, panting when it runs. The first
  time one reaches you, it throws you down and lets you go; after that, the
  second time in one chase is the end. (*How a creature's first touch is
  survived is the same for every creature of the redesign; `ENTITIES.md`,
  design 3.*)
- **The Watcher.** Someone standing under an old dust sheet, among the
  furniture under the same sheets, the shape of a face pressed into the
  cloth: at a glance it is one of them. It **only moves when you are not
  looking** (out of your sight, or in the dark beyond your lamp, or behind
  a wall), and after long enough unseen it is simply nearer; each time, it
  is standing differently. Its touch knocks you down and takes your mind;
  it kills only a mind already gone. It makes no sound. Close to it, your
  ears ring. (After the moving statue of Grandmother's Footsteps and SCP-173,
  by Moto42, CC BY-SA 3.0, as the seated man of the Loop is.)
- **The one in the corner.** Someone tall standing in the far corner of a
  quiet room, forehead to the angle of the walls, as a child is made to
  stand, in a long striped nightshirt, barefoot, its arms at its sides
  under it. You see its back. It hums, low and tuneless; you hear it from a
  room away. It is harmless while you see only its back. **Do not look at
  its face**: when the hum stops it is about to turn its head (now and then
  it looks round at the room behind it), and if its face comes round into
  your eyes, lit, it screams and comes, faster than you walk, slower than
  you run, and knows where you are for a while. Look away when the hum
  stops; keep your lamp off its face; pass behind it. Its face is a
  person's face with the eyes and the mouth upside down, and it is the last
  thing you see before it comes. Nothing sends it: it stands where it was
  put, off the way (only below the Lobby, never on the Building's floors
  nor where an anomaly is, at most one to a floor), and goes back there.
- **The Reader.** A thin man in a brown cardigan buttoned a hole wrong,
  paper stuffed in its pockets and down its front, ink on its fingers and
  at the corners of its mouth, the fingers too long. It walks bent over a
  sheet held close to its face, its long fingers spread on the back of the
  page, its eyes going along the lines, whispering what it reads, fast and
  without a voice. It lives where paper lies: by the piles left in the
  office wings and wherever else they lie. It finds you by sight, over its
  page, down a room's length, and only where you are lit or moving: still
  in the dark, you are nothing to it. When it looks up over the page the
  whisper stops; sure of you, it snaps the sheet flat and comes, faster
  than you walk, slower than you run, its soft quick steps close behind you.
  **Give it paper**: any sheet on the floor near it, it must go to, pick up
  and read to the end. Drop one behind you and walk on. The sheets you drop
  it takes (a trail of them through its halls does not last); a pile it
  reads the top sheet of, once, and leaves. It never touches her folded
  sheets nor the flat ones. At arm's length it lowers its sheet, and you
  see its face. (After the upyr of East Slavic folklore, which must stop to
  gather scattered seed, and the slit-mouthed woman of Japanese legend, who
  must stop for sweets thrown down.)
- **The one behind the glass.** Far out on the track a window looks out
  on, in the fog, a pale raincoat waits, belted, its hood up and drawn
  round a face: a smudge you are not sure of, seen through that window's
  row only (through every other window the track is empty). It always
  faces you. **Do not stare out of a window too long.** It counts your
  eyes on the glass of its window (any light of that row), across glances:
  after a few seconds the pane ticks, a dull knock and a ring, as glass
  does under strain, and it has seen you; look longer and it comes up the
  track toward you, evenly, while your eyes are on its glass, and stands
  where it is when they are not. It never goes back up the track. Once it
  has come into the streetlamp's light it needs your look no longer: it
  comes to the glass, the rim of its hood and its face to the pane and its
  hands flat on it; the glass creaks under it, its breath mists the pane
  on your side, wet skin squeaks on it. Then, unless the place lies quiet
  (it waits at the glass), the pane bows in round it like film and does
  not break, and it comes through and down onto the floor, into the lamp's
  light. **Its reach is that light**: inside it can be only where the
  lamp's light through its window lies on the floor, and it stands at the
  edge of it nearest you, its wet coat rustling. Stand in the light by it
  and it takes you: your mind drains and you are thrown down, out of the
  light (it kills only a mind already gone, and never the first time),
  and it goes back out. Out of its sight a few seconds, or watching you
  keep out of its light a while, or when the place lies quiet, it goes
  back out through the glass and stands close in the lamp's light; a long
  look then brings it straight back. **A covered window keeps it out**:
  let a blind all the way down (its slats clatter, and what hears comes to
  see; drawn up again, they clatter as loud) or find one someone papered
  over, and no look counts through that glass, no light comes in by it,
  and it does not come through it; it stands where it was, and is there
  when the blind goes up, going on as it was (at the glass two seconds
  before it comes). If it is in when all its window's light is covered,
  its light is gone, and it goes back out. Get away from it twice out of
  its light and the streetlamps outside the windows hang lower on their
  poles from then on: their light lies further into the rooms. It is heard
  only as glass and its coat: the tick, the creak, the squeak of wet skin
  on the pane, the long groan of the glass bowing, and in the room the
  rustle of its wet coat; its breath is seen on the pane, never heard.
  Nothing sends it: it stands behind one window of a floor, off the way,
  never behind a covered one (only below the Lobby, never where an anomaly
  is).
- **The Face** (the Stairwell only). See below.

**Pressure has a shape.** A pacing director (`entities/director.rs`) decides
when one of them comes for you: a calm, a build-up where the place announces
it (its steps somewhere near, lights failing, then silence), a peak, and then
relief: after a close call it **backs off** and the level goes quiet for a
while. Only one is sent at a time. The Watcher only moves on you when it is
the one sent; otherwise it keeps its distance. The one in the corner is never
sent: it comes only when its face meets your eyes, and it does not look round
while the place lies quiet after a close call. Nor is the one behind the glass:
it comes only for your look, and it does not come through while the place
lies quiet. **A sign is not a promise**: some
build-ups end in nothing, a sound somewhere near and then quiet. **The first
touch of each is not the end**: it throws you down and lets you go (never
taking the last of your nerve); death comes later, from a second touch while
it is still after you, or from one when your mind is already gone. The dark
wears at your nerve, but only so far: what spends it is them. **They
learn your habits**: hide from a Smiler in the dark often enough and it
waits, silent, where you went dark; outrun a Hound and it stops baying and
runs for where you are heading; keep a Watcher in your stare too long and
the lights over it fail while you watch; slip behind the one in the corner and it looks round
more often; feed a Reader paper and it tears the sheets in half unread; get
away from the one behind the glass out of its light and the streetlamps
outside the windows hang lower, their light further in. Each
learns one thing, never from your deaths, no two on the same floor, and you
see it the first time.
Who lives on a floor depends on the floor: Smilers where it is dark,
the Watcher among furniture, the Reader where paper lies, Hounds anywhere.

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
  seconds, as it wakes the place below; so does knocking at a door the
  cleaners marked (*Knocking*). The first quiet on the first floor
  is long, so her camp can be found first, unless a door the cleaners
  marked is knocked at.

## Anomalies

The Backrooms have swallowed other places too, the kind the SCP Foundation
files as buildings and rooms (a survey of 315 of them, Euclid and Keter,
went into choosing). Four have found their way in:

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
  faces (bald, a hollow where a face should be, in the store's mustard polo
  with a badge, its photograph too dark to make out), and when you come
  close they stop and look at you. That is all they do.
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
     nothing where his face should be (nothing on him is E.'s; who he is
     stays unsaid). He never moves while you can see him.
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
- **Paper** marks your path; boxes, bins and books below the Lobby give
  more (*Within reach*). Sometimes you find someone else's trail going the
  other way. Whatever reads in the office wings must stop for a sheet on the
  floor, and keeps the ones you drop.
- **Lore notes** are torn pages left by those who came before: Elias, M., the
  field surveys, the Stairwell logs. They lie as paper lies, lit only as
  the floor is; one that goes on with a story you began is written in that
  story's ink, and one you have read lies with its corner turned down. The
  notebook (Tab) keeps them, typed, after hers.
- **Her pages** are at her camps, one a floor; the notebook keeps them first,
  in the order she wrote them.
- **Crying** out loud is desperate. Something answers. The voice is your
  own, the only words you say: a man's, plain and tired, never brave
  (`docs/VOICES.md`, *You*). On the Building's floors you call out to
  anyone; in the rooms you shout for help; deep down you are worn out; and
  a cry soon after the last is a scream. Who you are is never said.
- **Cells**, on the Building's floors: shut yourself in when the siren goes,
  and wait for the all-clear (*The self-assembly*).
- **Coins** come from the lit vending machines' coin returns, one a
  machine and one at a time. A coin goes down with you and is kept with
  the run.
- **A snack** shaken loose from a machine is eaten as you take it: stale,
  and it steadies your nerve a little.

## What is put away

On every floor a few places off the way (two or three on the Building's,
two to four below; on the Lobby nothing but what grows there, its
emptiness is the point; none in the Showroom or the Borrowed House, nor in
the Stairwell or the Loop themselves) hold something somebody put there (`docs/SECRETS.md` 4–5). They are never on
her way, nor where the flat sheets go, nor in the one in the corner's
corner; they are found by looking where a real building hides things,
and each has the same tell all run:

- **A ceiling tile** that sits askew in its grid, a dark gap along an
  edge, a chair under it (someone stood there): LIFT THE TILE.
- **A vent grille** low on a wall, hanging by the two screws left in it:
  PRISE THE GRILLE.
- **A filing cabinet's drawer** that will not quite shut: something is
  taped under it.
- **A radiator** on the Building's floors with a rag stuffed down behind
  its fins: REACH BEHIND.
- **A cell's bench**, on the Building's floors, in a cell off the way;
  the cell where salt was left has two small cones of salt by its door.
- **The hole in the wall**, at hand height, fist-wide, dark inside, a
  little plaster under it on the floor: REACH IN. It gives back what you
  last used up (another bottle like the one you drank, another battery
  like the one you changed) and takes one thing you carry. With nothing
  on you, something touches your wrist, and it takes from you instead
  (once: after that it has nothing more to take, but you feel it all the
  same). Once a hole; three on the first descent (never on its first floor
  nor the Lobby).
- **Lying where nobody would leave it**: the end of a dead end, a wet
  corner, the floor of a corridor, a small table against a wall in an
  empty room, a coat hook in an office wing.

Most hold what any floor holds (almond water, a battery, a few sheets of
paper) or nothing but dust (never a coin: you carry one at a time, from
the machines). One or two a floor hold one of
the things below, which floors is the run's (on the first descent; below
the last camp one on about three floors in five, and a hole one floor in
four, but never the Building's ration or its salt). Some are signed, by a
hand that is neither hers nor the flat sheets' maker's: a small circle
with a point in it, scratched into the paint or the concrete with a key's
point, at shoulder height on a wall near the place, never on it and never
pointing: *something is put away near here*. The same circle scored
through with an X means *taken*. On the first floor, where a place
there has a wall to sign, one is found already opened and empty, its
sign crossed out. Who draws them is never said.

The things (each a real, small, ordinary thing; only what a real one
carries is printed on it):

| Thing | Where | Taken |
|---|---|---|
| Pale mushrooms, half again a field mushroom's size, in the wet carpet | A wet corner (the Lobby's floors, the Poolrooms, an overgrown atrium, washrooms) | Eaten: soft, like bread |
| Brown mushrooms | The same wet corners, from the sixth floor | Eaten: dry, and bitter |
| A white dish of boiled sweets in twists of cellophane, a tent card in a hand: *please take one* | A small table in an empty office, lobby or hall | One at a time; the dish gives three |
| A twist of paper with a few red seeds, on a saucer | The end of a dead end | Eaten: sharp, and sweet |
| A small cloth bag on a cord, sewn shut | Behind a radiator, in a cell, behind a vent, under a drawer | Put on: it smells of someone else |
| A paper sachet of salt (SALT on it) | A cell with salt by its door; a drawer | Pocketed |
| A small jar of blue jelly, no label | Above a ceiling tile | Eaten: it tastes of nothing at all |
| A ration of grey concentrate in a paper wrapper printed as the Building's coupon is (ONE PORTION. CONCENTRATE, GREY) | A cell, behind a radiator: the Building's floors | Eaten: floury, and it crumbles |
| A small amber medicine bottle, its label soaked off | Behind a vent, under a drawer, above a tile | Drunk: syrup, then warm |
| A tweed cap | On a coat hook in an office wing | Put on: it fits |
| A foreign coin with a hole through it, on a loop of string | Behind a radiator or a vent, under a drawer | Put on: it taps against your chest |
| An almond water, faintly yellow, its seal broken | Under a drawer, on a table, in a corner | Drunk: it tastes of cashews (and it is almond water too) |
| A red envelope with a lock of dark hair caught at its flap | The floor of a corridor | Picked up: there is hair in it |

What each does stays with you for the rest of the run, and is never
said (*Things that change you*). The notebook keeps a page of its own,
in your hand, after hers: *What I took*, a line a thing, what and where
and the floor; never what it did.

## Things that change you

What a thing put away does stays with you for the rest of the run
(`secrets/keeps.rs`), and nothing says it: not the thing, not a meter's
words, not the notebook. Taking one is felt only as what it is (its taste
or its weight, the line as you take it; a change of size comes over you
in about three seconds), and then learned by what follows. Each moves one or two numbers the place already
has, a step at a time:

- **Pale mushrooms**: you are smaller. Your eye is 7 cm lower (5 cm
  crouched), your steps carry 12% less far, you walk and run 4% slower.
  **Brown mushrooms**: larger, the other way. One of each and you are your
  own size again; two steps at most either way.
- **A sweet from the dish**: nerve comes back 10% faster (two count, and
  with the medicine a fifth faster at most). **A
  third from one dish**: your hands shake for the rest of the run (the
  lamp's beam trembles), and a Smiler sees your lamp from a fifth further.
- **The red seeds**: the place takes you a little for its own. The dark
  takes your nerve a fifth slower and a Smiler sees your lamp from 15% less
  far, but the door's call carries a fifth less far.
- **The sewn bag**: the dark takes your nerve a quarter slower and leaves
  you 45, not 40.
- **The salt**: a creature's blow takes a fifth less of your nerve.
- **The blue jelly**: your steps carry 15% less far.
- **The ration**: you can run longer (stamina to 112, not 100), but
  almond water gives a fifth less.
- **The medicine**: nerve comes back a fifth faster, stamina a tenth
  slower.
- **The tweed cap**: a Hound takes a quarter longer to smell you standing
  still; a cry is answered with the way half as often (what helps
  overlooks you too).
- **The holed coin**: a cry is answered with the way more often (15
  points, 60% at most), and no machine's coin return gives you a coin
  again.
- **The yellowish almond water**: it is almond water; and a Hound smells
  you standing still sooner (2.5 s, not 3).
- **The red envelope**: your look out of a window adds up a quarter
  faster, so the one behind the glass ticks and comes sooner.
- **The hole in the wall**, reached into with nothing on you: your stamina
  stops 15 short of where it did (once; a second hole has nothing more to
  take).

Each system moves two steps at most, a step being the smallest change a
thing makes to it (what several things do together stops there: your
steps a quarter either way, nerve a fifth faster, the dark's drain at
most two fifths slower, the stamina ceiling between 85 and 112, a
Smiler's sight of your lamp between 0.7 and 1.2 of itself); nothing
taken lowers what the dark leaves you or raises what a touch takes (the
creatures' fairness: `ENTITIES.md`), a chase stays under your sprint as
it is, and no thing kills. Leaving a thing where it is is always a
fine choice.

## The vending machine

On most floors of the rooms, against a wall where an office, a hall, the
Poolrooms or a party room would have put one, stands a glass-front snack
machine: painted steel, a tall window of spirals and bags and bars, a red
display, a keypad, a coin slot, a flap at the bottom that says PUSH. None
stands on the first floor of the rooms (the Lobby, level 4), whose
emptiness is the point, nor in a place left for nothing, nor in the
Showroom or the Borrowed House, whose light is their own. Most are still
lit; some of the dead ones carry a handwritten OUT OF ORDER on their glass.
What is taken from a lit one comes back, as the bottles on the floors
"just... appear" (torn note 1). The torn note about it (*The Vending
Machine*: "Insert coin, get nothing. Insert nothing, get coin. Do not
attempt the reverse. It costs more than you have.") hints at three of its
rules. What is true of it:

- **A lit one is awake.** It throws its light on the floor before it and
  hums from behind its glass, and now and then its tube clicks out for a
  few seconds and strikes again, on a rhythm of its own. **It goes dark
  when something comes**: while a thing that is after you, or one sent for
  you, is near it, its light is out, and it comes back a little after that
  has gone, without a sound: its own beat clicks out and thumps back, and
  this does not. A dark machine is a warning, unless you heard its click;
  in the quiet a coin bought it makes no sound at all, and there is no
  telling.
- **It knows what you are looking at.** Close to its glass, its display shows
  the code of the coil your eyes are on, before you touch it. Otherwise it
  says HAVE A NICE DAY.
- **Insert nothing, get coin.** (Not for one who wears the holed coin on
  a string: *Things that change you*.) A lit one's coin return, pressed with
  nothing in it, rings a coin down into the cup: once a machine, and only
  into empty pockets (you carry one coin at a time). The ring carries:
  what hears, near, hears it.
- **Insert coin, get nothing.** The coin in, the coil you were looking at
  turns, and what it holds comes forward and catches against the glass,
  hanging. MAKE ANOTHER SELECTION. And then it goes quiet for a while: its
  hum stops, and the place round it holds still (its own sounds, its
  flickers); while you are within ten metres of it, walls or none, nothing
  anywhere hears your steps. What comes still sounds as it does. Walk out
  of its quiet and you are heard again.
- **What hangs can be shaken loose, and that is heard.** A lit one can be
  shoved while something hangs in it, or once it has given its coin and
  your pockets are empty; a dead one, any time. Each shove bangs, and
  what hears comes to look where the bang was. It takes a few. Almond water,
  batteries, or a snack, eaten there and then.
- **Never rock or tilt.** Its label says so, as the real ones do, and says
  it can kill; it never does. With nothing hanging, the second shove
  creaks, and the third it does not come back: it falls on its face, and
  the whole floor hears it: whatever hears, anywhere on it, comes to where
  it fell. Standing before it, you are thrown down. Its almond water and
  batteries lie by it after.
- **It fills in its dark.** Whatever you took from a coil is back when its
  light comes back, whether the dark was its own beat or a thing passing
  near it (not while something still hangs in it). Now and then, near a
  lit one, a coil turns by itself, and nothing comes forward.
- **Some of the dead ones are not empty.** Stand close before one's glass
  a moment, and its flap lifts inward by itself: slowly, held up on the
  dark of its bin, then let fall. While it is held, a hand print comes up
  slowly on the inside of its glass, and stays. Only once a machine. Some
  of the dead ones still have something hanging in them, bought with a
  coin that was not yours.

What fills them and what is in the dead ones stay unexplained: no note,
page or sign may answer it. Their makers are nobody's: MIDWAY, HARTWELL,
HOLLIS, BAYSIDE, SUNNYVALE, printed as real ones are. Their
almond water is plain, nothing printed on its bottles, as on the bottles
found on the floors.

## The windows

The rooms have no outside. Their windows look out on one anyway. Below the
first floor of the rooms (the Lobby has none), where an office, a hall, a
party room or someone's rooms would have had windows, a few are cut into
the walls, one
to three side by side: in the offices, strip windows of aluminium over the
heaters under them; elsewhere, wood sash windows painted over many times.
Most have a blind, raised, or let down crooked, its slats tilted; you
can let one all the way down, or draw it up.

Every one of them looks out on the same place, from the same place,
whichever way it faces and whatever is on the other side of its wall: a
ploughed field in fog, a gravel track running straight away from the glass
into it, a few trees standing out there, and close outside, beside the
track, a streetlamp, its sodium light orange on the gravel. It is the last
of the light, and the deeper the floor, the later it is, until it is night.
Nothing out there moves, except, on some floors, a pale raincoat far out
on the track, through one window only (*Things that live here*: the one
behind the glass). The lamp's light comes in through the glass and
lies on the floor in the shape of the window, the shadows of its bars and
its blind across it, and standing near the glass you hear the lamp's
ballast through it (not through a row whose every light is covered). Here and there someone has covered one: paper taped
over the glass from inside, or the blind pulled all the way down and left.
Go round to the other side of its wall, and it is a wall.

The torn note *Windows* ("The windows here do not open. What they show
does not exist. Do not stare too long or it will begin to stare back.") is
about them.

## Within reach

Below the Lobby (from level 5), on every floor but one with rules of its
own, some of what was left behind still works as it did (`PROPS.md` 3).
Nothing says what a thing does or what it is for; what is found is said as
a pickup is said. What each one holds was settled when the floor was made,
not when it is used, and each is used once (a lift's button and the
television excepted).

- **A box** on top of a stack opens: its tape slit, its flaps out, packing
  paper in it. About one in four holds a few sheets of paper as well (two
  to four). What is written on a box (BOOKS, DO NOT OPEN, a name) never
  says what is in it.
- **A bin** can be looked into. Mostly litter; about one in three has a
  sheet or two worth keeping.
- **A stack of books** gives three to five of its pages to write on, once,
  torn out of the top book; deeper down (from level 9), only a page or
  two. Nothing torn out is left on the floor.
- **A locker** whose padlock hangs open (its shackle lifted out of the
  lock and turned; about two in five of the shut ones) swings open on
  someone's things: a jacket on its hook, a jumper folded on its floor; now
  and then (about one in seven) a bottle of almond water in the jacket's
  pocket. Whose they were is never said; a locker's tag carries a number or
  a word, never a name.
- **A dead lift's button** (a lift that does not come; a floor's first lift
  still does, and is a way down) can be pressed. It lights, weakly, for
  less than a second, and goes out. Behind about one in three of them, a
  few seconds later, a car is heard going by behind its doors. Pressed
  again, it does the same. Where the car goes is never said,
  and it is never a way.
- **The television** on its wooden stand by a wall (in someone's rooms or
  an office; one a floor), snow on its screen, its cold light flickering on
  what is near, hissing to one close by, can be switched off: its picture
  falls to a dot and goes, and its light and its hiss with it. And on
  again. It stands lit on the Lobby and the floors with rules of their own
  too, where no hand can switch it off; so does the small portable set left
  on, now and then, anywhere. The set lying dead on the floor is only that.

(On any floor of the rooms, a filing cabinet's drawers slide out: a
battery, almond water, a stack of paper, or nothing, EMPTY.)

Only a Hound or a Smiler hears them. The tape tearing, a locker's door and
a set coming on are loud: one near stops to listen, and a Hound may come to
look near where the sound was, never told where you are. A bin's rustle and
a page torn out are quiet: they stop nothing, but a Hound already crouched
to listen goes to look near them. A set switched off, a lift's button and
the car behind its doors are heard by nothing but you.

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
- A lit vending machine is a second ballast at the lamps' 120 Hz, from
  behind its glass, with its fan's soft air under it. Now and then, near
  one, its motor whines and a coil turns by itself. A dead one's flap,
  lifting by itself, squeals thinly on its hinge before it falls.
- Near a window you hear the streetlamp outside through the glass: its
  ballast, dulled, steady. Nothing else of out there is heard; now and
  then, the glass itself (*the one behind the glass*). A blind let down or
  drawn up clatters, dull aluminium, to the bottom rail's knock.
- What a hand does is heard from where it is done, and only near: a box's
  tape, a bin's rustle, a page torn out, a locker's door, a lift's button
  and the car going by behind it, a set's whine as it goes off or comes on,
  and a lit set's hiss close by (*Within reach*).

## Attribution and licensing

- **The Stairwell and the Face** are inspired by **SCP-087** by **Zaeyde**
  (<https://scp-wiki.wikidot.com/scp-087>), licensed **CC BY-SA 3.0**. We use
  our own names and text, but the SCP Wiki treats even the concept as a
  derivative work. The Stairwell (`crates/game/src/modes/stairwell.rs`, the Stairwell parts of
  this document and the Stairwell lore notes) should be treated as
  **CC BY-SA 3.0** and keep this attribution.
- **The Showroom** is inspired by **SCP-3008** "A Perfectly Normal, Regular Old
  IKEA" by **Mortos** (<https://scp-wiki.wikidot.com/scp-3008>), **CC BY-SA
  3.0**. `crates/game/src/anomalies/store_kit.rs`, the Showroom parts of `anomalies/mod.rs`,
  `assets/src/creatures/body_staff.py`, its notes and its sections here are
  CC BY-SA 3.0 and keep this attribution. The game uses no
  brand name: it is a generic flat-pack store.
- **The Borrowed House** is inspired by **SCP-1983** "Doorway to Nowhere" by
  **DrEverettMann** (<https://scp-wiki.wikidot.com/scp-1983>), **CC BY-SA 3.0**.
  `crates/game/src/anomalies/house.rs`, its notes and its sections here are
  CC BY-SA 3.0 and keep this attribution.
- **The one in the corner** takes its rule (seeing its face is what wakes
  it) from **SCP-096** "The Shy Guy" by **Dr Dan**
  (<https://scp-wiki.wikidot.com/scp-096>), **CC BY-SA 3.0**; the turned back
  is a motif of the Rake (anonymous, 2005; the motif only), the child made to
  stand in the corner is everyone's, and its face is the Thatcher illusion
  (Peter Thompson, 1980). `crates/game/src/entities/corner.rs` and
  `assets/src/creatures/body_corner.py` keep this attribution.
- **The one behind the glass** takes its sharpening look from **SCP-2885**
  "Window to Another You" by **Pythy**, rewritten by **Zyn**
  (<https://scp-wiki.wikidot.com/scp-2885>), its face that turns with you
  from **SCP-3052** by **Witryso** (<https://scp-wiki.wikidot.com/scp-3052>)
  and its way through glass "like a finger through plastic wrap" from
  **SCP-L682** by **CRAt26** (<https://scp-wiki.wikidot.com/scp-l682>), all
  **CC BY-SA 3.0**; its reach from a window, and covering windows against
  it, from the Backrooms wiki's Entity 2 "The Windows" (the concept by
  u/glassydude101, rewritten by **DrAkimoto**;
  <https://backrooms-wiki.wikidot.com/entity-2>, **CC BY-SA 3.0**); the
  danger in looking closely from the Japanese internet legend Kunekune
  (the motif only); and, public domain, the window that will not open from
  Emily Brontë's *Wuthering Heights* (1847) and, from Gogol's *Viy* (1835),
  "Don't look!" and a line it cannot cross. The raincoat, the track and
  its light are this game's own. `crates/game/src/entities/glass.rs` and
  `assets/src/creatures/body_glass.py` keep this attribution.
- **What is put away** (`docs/SECRETS.md`) takes its rules from: the dish
  of sweets, two free and the third costing, from **SCP-330** "Take Only
  Two" by **Dr Kondraki** (<https://scp-wiki.wikidot.com/scp-330>) and the
  Creepypasta Wiki's "Please Take Just One" by **Umbrello**
  (<https://creepypasta.fandom.com/wiki/Please_Take_Just_One>); the hole in
  the wall that gives back and takes from the first **SCP-1162** "A Hole in
  the Wall" (author unknown; the Japanese branch's archive,
  <http://scp-jp.wikidot.com/old:scp-1162>); the cap that has you overlooked
  from **SCP-268** "Cap of Neglect" by **Pair Of Ducks**
  (<https://scp-wiki.wikidot.com/scp-268>); a body changed by what it eats
  from **SCP-1916** "Moon Rocks" by **Smapti**
  (<https://scp-wiki.wikidot.com/scp-1916>); calm bought with slowness from
  **SCP-714** "The Jaded Ring" by **Arcalane**
  (<https://scp-wiki.wikidot.com/scp-714>); a coin that changes who keeps it
  from **SCP-832** "Accountant's Coin" by **Inglonias**
  (<https://scp-wiki.wikidot.com/scp-832>), all **CC BY-SA 3.0**; the blue
  jelly that silences your steps from the Backrooms Fandom wiki's Moth
  Jelly (Object 6, **KingofGarbage**, **CC BY-SA**) and Wikidot's Liquid
  Silence (Object 17, **Atom_Rutherford**, **CC BY-SA 3.0**); the almond
  water that is not, from Cashew Water (Wikidot Object 44, **yoyoflux**,
  rewritten by **Positive123**; Fandom, **Taangeerine**) and Liquid Pain's
  broken seal (Wikidot Object 48, **ReyDay**), **CC BY-SA**; the ration
  from the Samosbor wiki's food concentrate (**ProfessorSacred**, **CC
  BY-SA**) and Royal Rations (Wikidot Object 16, **Sariastuff**,
  **zakkun_**, **CC BY-SA 3.0**); the salt from the Creepypasta Wiki's "The
  Midnight Game" (author unknown, **CC BY-SA**) and the Japanese custom of
  morijio; and, public domain, Lewis Carroll's *Alice's Adventures in
  Wonderland* (1865: the bottle, the cake, the mushroom), Persephone's seeds
  (the *Homeric Hymn to Demeter*), Izanami in the *Kojiki*, Christina
  Rossetti's *Goblin Market* (1862), and folk custom (the omamori and the
  ladanka sewn shut, the red envelope on the road, the coin worn and not
  spent). The things, their places and what each does are this game's own.
  `crates/game/src/secrets/` keeps this attribution.
- **The Reader** takes its compulsion from folklore, public domain: the
  upyr (upiór) of East Slavic belief, kept busy by poppy seed scattered for
  it to gather, and Kuchisake-onna, the slit-mouthed woman of Japanese urban
  legend, who stops for sweets thrown down. The cardigan, the ink and the
  reading are this game's own. `crates/game/src/entities/reader.rs` and
  `assets/src/creatures/body_reader.py`.
- **The Watcher**, and the seated man of the Loop, take their rule (it moves
  only when unseen) from **SCP-173** by **Moto42**
  (<https://scp-wiki.wikidot.com/scp-173>), **CC BY-SA 3.0**, and from the
  playground game Grandmother's Footsteps; the dust sheet and the furniture
  under it are this game's own. `crates/game/src/entities/watcher.rs`,
  `assets/src/creatures/body_watcher.py`, `sheeted.py` and
  `body_seated.py` keep this attribution.
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
  **Backrooms Level 188** "The Courtyard of Windows" by **1000dumplings**,
  rewritten by **Kai4C**
  (<https://backrooms-wiki.wikidot.com/level-188>), **SCP-2723** "The
  Prophet's Tower" by **HotCocoaNerd** (<https://scp-wiki.wikidot.com/scp-2723>),
  **SCP-4370** "Post-Industrial" by **UraniumEmpire**
  (<https://scp-wiki.wikidot.com/scp-4370>), **SCP-5778** "Hidden In Plain
  Sight" by **MarcusLCrassus** (<https://scp-wiki.wikidot.com/scp-5778>) and
  **SCP-3333** "Tower" by **Jekeled** (<https://scp-wiki.wikidot.com/scp-3333>),
  all **CC BY-SA 3.0**: `crates/game/src/level/building.rs`,
  `crates/game/src/world/building.rs`, `crates/game/src/world/building_dress.rs`,
  `crates/game/src/world/assembly.rs`, `crates/game/src/world/far_figures.rs`,
  `assets/src/creatures/body_far_figure.py`, `docs/MEGASTRUCTURE.md` and the
  Building's notes are CC BY-SA 3.0 and keep this attribution. Piranesi's
  *Carceri* (1750) is in the public domain.
- The anomalies were chosen from a research survey (not in this repository)
  that summarized 315 SCP Wiki articles in our own words, each with its
  author and link (CC BY-SA 3.0).
- **Backrooms** lore comes from the Backrooms wikis (Wikidot and Fandom),
  **CC BY-SA 3.0**. The zone archetypes, Smiler and Hound rules and Almond
  Water are adapted, not copied: the Hound after Entity 8 of the Wikidot
  wiki (<http://backrooms-wiki.wikidot.com/entity-8>; Palico22, rewritten by
  1000dumplings), made blind here.
- **The creatures' bodies** (the Hound's, the Smiler's, the Watcher's, the
  one in the corner's, the Reader's, the one behind the glass's, the
  Showroom's staff's, the seated man's of the Loop and the Building's figure
  across the void) start from MakeHuman's base mesh and targets
  (<https://github.com/makehumancommunity/makehuman>), **CC0 1.0** since
  September 2020 (`assets/src/creatures/sources.json`).
- The original 2019 yellow-room photograph was released **CC0** by the
  building's owner. We draw our own textures and use none of it.
- **Kane Pixels'** *The Backrooms* series is copyrighted and was used only as a
  tone reference. Nothing is taken from it.
