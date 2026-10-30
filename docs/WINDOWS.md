# The windows

What the windows of the rooms are, what they look out on, what lives
behind some of them, and the plan that rebuilds them from scratch: as real
windows cut into real walls, onto an outside the rooms do not have, and
with a thing behind the glass whose rule is learned by playing. The canon
it extends is [`LORE.md`](LORE.md) (torn note 27, *Windows*); the pillars it
serves are [`PILLARS.md`](PILLARS.md) 2, 4 and 5 above all; the research
behind it is in [`research/windows-real.md`](research/windows-real.md) (real
windows, glass, how games show a view) and
[`research/window-horror.md`](research/window-horror.md) (windows that look
back), summed up in [`SOURCES.md`](SOURCES.md) section 13.

## The brief (2026-10-30)

"Remake windows in Backrooms zones, making them more realistic. Show creepy
landscape beyond the window. For some windows, make looking for too long
attract attention of entity behind the window, that is able to go through
the window to the level, but in limited reach. Find the optimal entity by
looking for window-related scp and creepypasta and integrate entity and its
interactions in lore."

## The analysis

What the windows were, read from the code and from frames (`--views-at`,
seed 4242, levels 5–11):

- **A picture hung on a wall.** `build_window` (`core_props/builders.rs`):
  a box 1.5 × 1.4 × 0.08 m of dark wood with a cross of 40 mm muntins and a
  sill, its centre 1.55 m up, hung 5 cm off the wall (`tail_window_zone`,
  `core_props/tail.rs`). No opening, no reveal, no depth: the wall goes on
  behind it.
- **An outside that was static.** Its pane was a 256 px canvas of dark
  grey, rain-streak noise and condensation smears, half the time with four
  warm dots ("distant lights that shouldn't be there"). It is the same from
  every side: nothing in it moves as you walk past.
- **No glass, and the wrong light.** The pane was an unlit picture; a cool
  point light (0.4, 4 m) before it, nothing in the light map. On wet levels
  a rain overlay scrolls over it (`rain_windows`, and the rain hiss when you
  face one).
- **Everywhere and anywhere.** Three a level on every floor of the rooms,
  on any wall a tile of the take pool offered: the Lobby's, the Showroom's,
  the Borrowed House's (seed 4242: three on each of levels 5–11).
- **The canon promised more.** Torn note 27, *Windows*: "The windows here
  do not open. What they show does not exist. Do not stare too long or it
  will begin to stare back." Nothing stared back.
- Three themed placers hang small decorative windows: frosted glass (one
  floor in thirty), a broken frame onto black (one in seven), stained glass
  (one in twenty-five, the occult theme).

## Three prototypes (2026-10-30)

Before the design, the parts most likely to fail were tried on a throwaway
build (not committed):

- **The view.** An equirectangular photograph (a CC0 Poly Haven panorama)
  traced per pixel onto a ground disc and a dome round a point behind the
  glass (three.js's GroundedSkybox, done in the fragment shader:
  `Patch::WindowView`). The near ground moves with your head, the far land
  stays where it is: seen from the left you see further to the right, as
  through a real window. One texture read, in its own variant only.
- **The opening.** The walls are drawn from the solids after the props
  (`build_walls`); a window can cut its opening out of what is drawn (the
  solids, what you walk into, stay whole: the glass is solid) and leave a
  sliver at the wall's back (the far room sees an unbroken wall). The
  reveal takes the wall's own surface and world-space detail.
- **A thing beyond the glass.** A body standing 12 m out in the field,
  drawn as itself: what lies beyond the window's plane is slid along each
  eye ray onto a thin slab just in front of the view (nearer things a hair
  nearer, so it hides itself rightly), clipped to the opening, lit by the
  view's sky and hazed by its true distance (`Material::portal`). It stands
  where it is on the screen and is hidden by whatever is in front of the
  glass; what has come through into the room is drawn as anything is.
- Found: `settle_props` rebuilds the solids (faces near interactable things
  are pinned to the tile's edge) and moves what hangs on walls, so a window
  must be finished after settling, from the final solids, and must not
  register its verb before then (it would pin its own wall).

## What the research says

Two notes, in [`research/`](research/README.md). What they changed:

- **Real windows, by the place.** A wood double-hung sash (two sashes
  sliding past each other, the upper one outboard, a meeting rail at
  mid-height, muntins 19 mm, a 116 mm frame, a stool, an apron and casing)
  in older rooms; aluminium strip windows in offices (fixed lights about
  1.2 m wide between 50 × 114 mm mullions, a sill on a convector cabinet,
  insulating glass, some units gone milky as their seals failed); steel
  hoppers with wired glass high in utility walls. Mini blinds (25 mm
  aluminium slats, a steel headrail, a clear hexagonal wand), often raised
  crooked, slats kinked. They age where water stands: paint built up and
  flaking at the sill and bottom rail, cracked putty, a cracked light, dead
  flies along the sill, mould in the bottom corners.
- **Glass.** 4% a face square on (about 8% a single pane, 14% a double),
  all of it at a graze; dust and smears lit by the room; at night a lit
  room's window is a mirror (the room about a thousand times brighter than
  a field), at dusk the view outshines the room's ghost in it.
- **What looks fake**: a box on a wall with no reveal, a cross of fat
  muntins, a view brighter than the room, no reflection, clean glass, a
  light spilling from a window at night.
- **The view**: a ground-projected panorama for the land (exact for flat
  ground from any eye), the thing that comes as real geometry beyond the
  glass on the same ground. Poly Haven and ambientCG are CC0 and may be
  shipped in a game.
- **Windows that look back.** SCP's windows make the look take time (15 s
  in SCP-2885, which sharpens in stages; 93 s in SCP-508, which then opens
  both ways), make the reach short and exact (2 cm in SCP-1148), and are
  contained by covering. The Backrooms' Windows (Entity 2) reach 1–2 m with
  an arm and are kept covered. Kunekune: harmless far off, the danger is
  in looking closely. SCP-3052's figure faces you from every angle;
  SCP-L682 comes through glass "like a finger through plastic wrap".
  *Wuthering Heights*: a window that will not open, a grip through the
  glass that lets go when you pull, a hole stopped with books. *Viy*: a
  line on the floor it cannot cross, "Don't look!". Poe's raven: what came
  in through the window is known by its shadow on the floor.
- **How it must differ** from the canon's gaze rules: not a glance at a
  face (the one in the corner), not "it moves while unseen" (the Watcher:
  this one moves only while seen, its mirror), not a figure that turns
  away (the figure across the void: this one faces you). Its counter must
  be a place and a cover, not where you look. And from the canon's
  sameness (`QUALITY.md`): no tell that is a sound stopping, no counter that
  is stillness in the dark, not "tall", no turned back.

## The design

### 1. A real window, cut into the wall

A window is an opening in the wall, finished after the props settle
(`windows_settled`, from the final solids): the drawn wall loses a box the
width and height of the opening and as deep as the wall less a sliver at
its back (12 mm; 0.30 m into a wall that fills a whole tile), so the reveal
is the wall's own surface; what you walk into is unchanged. In the reveal
stands a window as it is built, from Blender parts (batch `w01`), at its
real size:

| Kind | Where | What |
|---|---|---|
| **Office strip** | office archetype, office wings | Fixed lights 1.2 × 1.3 m between 50 mm aluminium mullions 114 mm deep, two or three in a row, the sill 0.85 m up on a steel convector cabinet with its grille; an insulating unit in each light (one now and then gone milky); clear or bronze anodised gone dull; a drywall return and a laminate stool; a mini blind in each light |
| **Double-hung** | halls, living quarters, parlours, party rooms | A wood sash window 0.91 × 1.37 m, the sill 0.8 m up: two sashes, the upper outboard, a meeting rail with a crescent lock, one-over-one or two-over-two; painted (off-white, cream, a dark green or brown) in many coats, flaking at the sill and bottom rail; casing, stool and apron; a mini blind, or none |

Each is worn from a stream of its own (salted by where it stands): smears
and dust on every pane (ambientCG's photographed smears, CC0), now and then
a cracked light, dead flies along the sill, mould in a bottom corner, a
blind raised crooked or with kinked slats, and here and there a window
someone covered: sheets of paper taped over the glass from inside, the
blind pulled down and left (pillar 1: what is absent says more).

**Glass** (`Patch::Glass`): Fresnel from 8% square on to all at a graze,
the room seen in it (the light where it stands), dust and smears lit by the
room and from behind by the view, a mist that can bloom on it. A row of
lights shares one outside.

### 2. What they look out on

Every window of the rooms looks out on the same place, from the same place,
whichever way it faces and whatever is on the other side of its wall
(pillar 2: familiar, and wrong once you notice). A ploughed field in fog,
two gravel tracks between the furrows, the one before the window running
straight away from the glass into the fog, a few trees standing in it
(Poly Haven's *misty_farm_road*, CC0, turned and graded), and close outside,
beside the track, a **streetlamp**, just lit, its sodium light orange on the
gravel and in the fog round it. It is the last of the light: the deeper the
floor, the later it is, until below the sixth floor of the rooms it is
night and the glass gives back mostly the room, the lamp and its pool still
showing through. Nothing in it moves. Go round to the other side of its
wall and the wall is a wall.

- The view is a photograph laid on a ground disc and a dome round a point
  just outside the glass (`Patch::WindowView`), the lamp's pool and its
  halo in the fog added where they fall; the lamp itself (a steel pole and
  a cobra-head luminaire, Blender) and anything else out there are drawn
  as themselves beyond the glass (`Material::portal`).
- **The lamp's light comes in.** Through the opening it throws a patch on
  the floor inside: the shape of the window, the shadows of its bars and
  mullions across it, about 0.7 to 2.7 m out from the wall, orange on the
  carpet (a placed decal, the lamp's own; a small warm point light at the
  sill for what stands in it). A lowered blind takes it away. Nothing else
  of the outside lights the room (pillar 4's light from places: the
  research's "no light from a window at night").
- **Its sound**: through the glass, within a few metres of it, the lamp's
  ballast, a faint, steady buzz (pillar 3).
- On wet floors the rain still runs down the glass (`rain_windows`).

### 3. Where windows are

Where a real building would have them: offices and office wings, halls,
living quarters, parlours and party rooms (a `Home`, as the vending
machine's), on a straight wall (never one that leans), its face whole over
the opening, a clear floor before it; in rows of one to three a few metres
apart, two to four windows a floor. None on the Lobby (level 4: its
emptiness is the point), none in the Showroom or the Borrowed House (their
light is their own), none on the Building's floors (whose windows are its
own, the Well's), none on the Loop's or the Stairwell's. The three themed
decorative windows (frosted, broken frame, stained glass) keep their
places for now.

### 4. The one behind the glass

*After SCP-2885 (a view that sharpens the longer you look), SCP-3052 (a
figure that faces you from every angle), SCP-L682 (through glass like
plastic wrap) and the Backrooms' Windows (Entity 2: a reach from the window
and covering as the counter), all CC BY-SA 3.0; Kunekune (the danger in
looking closely, a motif); and, public domain, Gogol's* Viy *(a line on the
floor it cannot cross), Poe's raven (known by its shadow on the floor) and*
Wuthering Heights *(a window that will not open, a grip through the glass
that lets go).*

- **What it is.** Someone standing far out on the track in a pale raincoat,
  belted, the hood up and drawn, wet; an ordinary height, the shoulders a
  little too narrow; its face in the hood's shadow. Far off it is a smudge
  you are not sure of. **It always faces you**: walk along the glass and it
  turns with you without moving its feet. Its eyes give back your lamp.
  Close, pressed to the glass, its face is a person's, wet and pale and
  flattened white where it touches, its eyes open.
- **Where.** Behind one window of a floor, on about half the floors from
  the second of the rooms (level 5) on, never on an anomaly's floor: the
  window chosen by the level's seed, off the way from the arrival to the
  door, far from the arrival, her camp and the door. Nothing marks its
  window but what stands out on the track.
- **The rule you learn.** *Do not stare out of a window too long.* What
  counts is your eyes' time on its glass: the line of your view through one
  of its window's lights, within 8 m, nothing between, the glass not
  covered. It adds up across glances (a second a second while you look,
  draining a quarter as fast when you do not).
  1. **Three seconds**: it has seen you. The pane ticks, sharp, as glass
     does when it cools (a tell, heard near the window).
  2. **Five seconds**: it comes. While your eyes are on its glass it walks
     up the track toward you, evenly, too smoothly, its arms still; look
     away and it stands where it is, facing you. It never goes back.
  3. **In the lamp's light** (within 7 m of the glass) it no longer needs
     your look: it comes on to the glass.
  4. **At the glass**: its face pressed to the pane, its hands beside it;
     the glass creaks under it; its breath mists the pane, on your side
     (tells you see and hear).
  5. **Through.** Two seconds later, unless the glass is covered, the pane
     bows in round its face and hands like film, without breaking (the
     windows do not open), and it leans in to the waist and comes down onto
     the floor inside, into the lamp's light.
- **Its reach is the light.** Inside, it can be only where its lamp's light
  falls through its window: the patch on the floor. It can never set a foot
  out of it. In it, within its reach, it takes you by the wrist and draws
  you to the glass, and your mind drains while it holds; the first time in
  a run it lets go, and you are thrown down out of the light. A mind it
  leaves spent goes through the glass. Out of its light you are out of its
  reach: it stands at the edge of the light nearest you, facing you.
- **It goes back.** Out of its sight a few seconds (or far off), it goes
  back through the glass and stands out in the lamp's light, facing the
  window, still. Now it is close: a long look and it comes straight to the
  glass.
- **Covered.** Lower the blind (`[E] LOWER THE BLIND`, a second, its slats
  rattling down: what hears, near, comes to look) or find a window someone
  papered over, and the glass is covered: no look counts, no light comes in,
  and it does not come through. It stands where it was, facing a covered
  window, and is there when the blind goes up.
- **What it learns** (design 4): escape it twice out of its light, and the
  lamp outside its window hangs lower: its light lies further into the room
  (the first time, where you can see it).
- **Its sound**: glass only. The tick, the creak of the pane under its
  face, the long low groan of glass bowing, wet skin on glass. No voice.
- **Placed, never sent**, like the one in the corner: nothing sends it,
  it comes only for a look, and it does not come through while the place
  lies quiet after a close call. Its first touch follows the shared rule
  (`touch_nerve`: it takes your mind, and kills only a mind it leaves
  spent).

| Thing | What your eyes do | What it does | You are safe when |
|---|---|---|---|
| The Watcher | keep it in sight | moves only when unseen | you look at it |
| The one in the corner | must not meet its lit face | screams and comes at once | you look away when the hum stops |
| The figure across the void | look at it | turns its back, then is gone | always |
| **The one behind the glass** | **linger on a window's glass** | **comes only while you look; never goes back** | **you are out of its light, or the glass is covered** |

### 5. The canon

`LORE.md` gains the windows (*Things in the rooms*: what they show, the
lamp, the light on the floor, the blind) and the one behind the glass
(*Things that live here*, placed and never sent), with its attribution.
The rooms still have no outside: their windows show one anyway, and "what
they show does not exist" (torn note 27, which already hints all three of
its parts and needs no new note). The blind's bottom rail may carry the
real cord warning word for word, as the vending machine carries its label
(fixed printing about itself, `PILLARS.md` 4's exception). The Backrooms'
Level 188 credit gains its first author, 1000dumplings (found by the
research).

### 6. Fair, deterministic, cheap

**Fair.** Two tells you could perceive before any touch (the tick, its
coming up the track in plain view, its face at the glass, the creak, the
mist); never through a covered window; never in a level's first 30 s; its
reach is a lit shape you can see before it comes. It never comes unless you
look.

**Determinism.** Windows are chosen in the level's build (a few draws from
its stream, as before) and finished after settling with no draws; their
wear and their outside from salted streams of their own; the thing behind
the glass is placed after the build with no draws (`with_seed`), and its
rules read only your eyes and its clock.

**Performance.** The view: one pipeline variant and one texture (3072 ×
2304, made once per level that has windows); glass in the transparent pass;
a window's parts merged by material. The thing beyond the glass costs what
a creature costs.

## Milestones

| Part | What | Gate |
|---|---|---|
| W0 | Research (two notes), the prototypes, this design | the notes; `lore-keeper` on the design's canon (with W4) |
| W1 | **The window**: the opening cut in the wall, the two kinds (Blender batch `w01`), glass, the blinds, the wear; the view (its texture from the CC0 photograph, the shader), the lamp outside, its light on the floor and its buzz; where windows are; the old window gone | blind review against the old windows; goldens, fixtures and the level sweep; materials lint; bench |
| W2 | **The one behind the glass**: its body (MakeHuman CC0, a raincoat), drawn beyond the glass, its rule (the look, coming, the glass, through, its reach, going back), its sounds | `tests/creatures_rules.rs`, an arena for its counter, `tests/ecology.rs`, `tests/listen.rs`; its goldens; pacing and the beatable run |
| W3 | **Covering, and what it learns**: the blind as a verb (its rattle heard), papered windows, the lower lamp | tests; the rule follower |
| W4 | **The canon and the reviews**: `LORE.md`, `ENTITIES.md`, `PLAYTEST.md`, `QUALITY.md`, the ledger; `lore-keeper` rounds; blind reviews of the windows and the figure; full check, bench | the reviews, the full check |

## Progress

| Part | Commit | What was done |
|---|---|---|
| W0 | (this commit) | The research (`research/windows-real.md`, `research/window-horror.md`), the three prototypes, this design |

## What only people can tell

Whether a window reads as a window or as a picture; whether the view is
unsettling or merely pretty; whether players notice the same view from
every window, and the wall behind; whether the smudge on the track draws a
stare, and whether "do not stare too long" is learned from one meeting;
whether its light on the floor reads as its reach.
