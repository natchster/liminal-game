# The windows

What the windows of the rooms are, what they look out on, what lives
behind some of them, and the plan that rebuilds them from scratch: as real
windows cut into real walls, onto an outside the rooms do not have, and
with a thing behind the glass whose rule is learned by playing. The canon
it extends is [`LORE.md`](LORE.md) (torn note 27, *Windows*); the pillars it
serves are [`PILLARS.md`](PILLARS.md) 2, 4 and 5 above all; the research
behind it is in [`../research/windows-real.md`](../research/windows-real.md) (real
windows, glass, how games show a view) and
[`../research/window-horror.md`](../research/window-horror.md) (windows that look
back), summed up in [`SOURCES.md`](../research/SOURCES.md) section 13.

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
  glass (a ground-projected sky, `docs/research/SOURCES.md` 13, done in the fragment shader:
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

Two notes, in [`research/`](../research/README.md). What they changed:

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
the opening and half a metre past it either side (never a pillar's face,
nor a block standing out of its wall), a clear floor before it, nothing
already hung there; in rows of one to three lights or windows. The rows
stand on the three tiles the old window zone took, with its draws, so
nothing placed after them moves; one is planned only where its tile's zone
is a window's home (most floors below the Lobby have some: 52 of 74
over fourteen runs). None on the Lobby (level 4: its
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
floor it cannot cross: its light's edge) and* Wuthering Heights *(a window
that will not open). (Poe's raven, known by its shadow on the floor, and
Wuthering Heights' grip through the glass were planned and not built.)*

- **What it is.** Someone standing far out on the track in a pale raincoat,
  belted, the hood up and drawn, wet; an ordinary height, the shoulders a
  little too narrow; its face in the hood's shadow. Far off it is a smudge
  you are not sure of. **It always faces you**: walk along the glass and it
  turns with you without moving its feet. Close, pressed to the glass, its
  face is cold and wet and drowned-pale, flattened where it touches, its
  eyes open and clouded.
- **Where.** Behind one window of a floor, on about four in ten of the
  floors with windows from the second of the rooms (level 5) on, never on
  an anomaly's floor: the
  window chosen by the level's seed, off the way from the arrival to the
  door, far from the arrival, her camp and the door. Nothing marks its
  window but what stands out on the track.
- **The rule you learn.** *Do not stare out of a window too long.* What
  counts is your eyes' time on its glass: the line of your view through any
  light of the row its window stands in (it is seen through all of them),
  within 8 m, nothing between, that light not covered. It adds up across glances (a second a second while you look,
  draining a quarter as fast when you do not).
  1. **Three seconds**: it has seen you. The pane ticks, sharp, as glass
     does under strain (a tell, heard near the window).
  2. **Five seconds**: it comes. While your eyes are on its glass it walks
     up the track toward you, evenly, too smoothly, its arms still; look
     away and it stands where it is, facing you. It never goes back.
  3. **In the lamp's light** (within 7 m of the glass) it no longer needs
     your look: it comes on to the glass.
  4. **At the glass**: its face pressed to the pane, its hands flat on it;
     the glass creaks under it; its breath mists the pane, on your side,
     and wet skin squeaks on it (tells you see and hear).
  5. **Through.** Two seconds later, unless the glass is covered or the
     place lies quiet after a close call (then it waits at the glass,
     nothing to the director while it does), the pane
     bows in round its face and hands like film, without breaking (the
     windows do not open), and it leans in to the waist and comes down onto
     the floor inside, into the lamp's light.
- **Its reach is the light.** Inside, it can be only where its lamp's light
  falls through its window's row: the patch on the floor. It can never set
  a foot out of it. In it, within its reach, it takes you: your mind drains
  and you are thrown down out of the light (its touch is the shared one: it
  kills only a mind already gone, spent before it touches, and never at a
  first meeting). Inside, water runs off its coat onto the floor (its tell
  there, every 2 s). Out of its light
  you are out of its reach: it stands at the edge of the light nearest you,
  facing you.
- **It goes back.** Out of its sight a few seconds (or far off), or once
  it has watched you keep out of its light 12 s, or while the place lies
  quiet (as they all back off), it goes back through the glass and stands
  out in the lamp's light, facing the window, still. Going in and out of
  an empty room is no getting away from it (what it learns from). Now it is close: a long look and it comes straight to the
  glass.
- **Covered.** Lower the blind (`[E] LOWER THE BLIND`, a second, its slats
  rattling down: what hears, near, comes to look) or find a window someone
  papered over, and the glass is covered: no look counts, no light comes in,
  and it does not come through. It stands where it was, facing a covered
  window, and is there when the blind goes up (`[E] RAISE THE BLIND`, as
  loud), two seconds at the glass before it comes. If it is in when its
  row's light is all covered, its light is gone and it goes back out.
- **What it learns** (design 4): escape it twice out of its light, and the
  lamps outside the windows hang lower (1 m), every window's of the floor
  (they look out on the same place: its window stays unmarked): their light
  lies further into the rooms (the first time where you can see it: in its
  window's room, near, the light creeping in across the floor over 3 s;
  after that from the first, on every floor with windows).
- **Its sound**: glass, and water. The tick, the creak of the pane under
  its face, wet skin squeaking on the pane, the long low groan of glass
  bowing; inside, its wet raincoat rustling. No voice; its breath is seen
  on the pane, never heard.
- **Placed, never sent**, like the one in the corner: nothing sends it,
  it comes only for a look, and it does not come through while the place
  lies quiet after a close call. Its first touch follows the shared rule
  (`touch_nerve`: it takes your mind, and kills only a mind already gone).

| Thing | What your eyes do | What it does | You are safe when |
|---|---|---|---|
| The Watcher | keep it in sight | moves only when unseen | you look at it |
| The one in the corner | must not meet its lit face | screams and comes at once | you look away when the hum stops |
| The figure across the void | look at it | turns its back, then is gone | always |
| **The one behind the glass** | **linger on a window's glass** | **comes only while you look; never back up the track** | **you are out of its light, or the glass is covered** |

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
| W0 | `19efe5c` | The research (`research/windows-real.md`, `research/window-horror.md`), the three prototypes, this design |
| W1 | `1d1697d` | The window. A row is chosen in the core props' tail on the three tiles the old window zone took, with its draws (a take and a wall side each, on every floor of the rooms: nothing placed after it moves), planned only on floors with windows and where the tile's zone is a window's home (`WINDOW_HOME`); one to three lights or windows a row and its own seed from a stream of its own. It is finished after the props settle on the final walls (`windows_settled`, no draws): the wall's face probed from the solids over the span and half a metre past it at the sill, the middle and the head (never a doorway, a corner, a pillar's face or a partition that leans), nothing already hung there, the wall's thickness measured, and the drawn wall less a box as deep as the wall less 12 mm (0.30 m into a wall that fills its tile; `build_walls`, `box_minus`); the solids stay whole, and a piece of wallpaper under a window keeps its wall's height (its skirting runs on). Two kinds from Blender parts (batch `w01`, 20 parts): a wood double-hung sash window 0.91 x 1.37 m (one-over-one or two-over-two, painted off-white, cream, dark green or brown, its lock and lifts, casing, stool stretched across the reveal, apron) in halls and homes; an office strip window, lights 1.2 m wide between 50 mm mullions over steel convectors, in offices. Each light its glass (`Patch::Glass`: Fresnel 8% to all, the room in it, dirt from three CC0 ambientCG maps lit by the room and by the view, a failed unit's milk); a mini blind raised, let down crooked with its slats open, or pulled down and left; now and then newspaper and sheets taped over it; dead flies on the sill. The view (`Patch::WindowView`): Poly Haven's *misty_farm_road* (Greg Zaal, CC0) turned so a track runs away from the glass, half its turn graded to the last of the light (`assets/src/textures`, `view.jpg`), traced on a ground disc and a dome round a point just outside the glass, the same from every window, later the deeper the floor (night from depth 7). A streetlamp close outside drawn beyond the glass (`Material::portal`), its pool on the ground and its halo in the fog; its light thrown in on the floor (a decal: the window's shape, the muntin's and the blind's shadows, soft-edged, falling off as a lamp's does, never through a wall; its lit cells kept as the window's light, `Window::lit_at`); its ballast heard through the glass within 7 m (`MachineHum::lamp`). The old framed window (`build_window`, `tail_window_zone`) is gone. Found on the way: the broken window frame's shards lay in a circle round it, half inside its wall (a leaning wall showed it: now half a turn about the way out). Blind review (a fresh reviewer, the art-critic's brief, both orders, 22 window views of 11 floors from each build): the new windows preferred in all 22 ("a real opening cut into the wall, with depth… plausible glass and outside" against "a printed panel"); what it found broken on the new side fixed (a papered light read as plaster: now newsprint and sheets; the skirting jogged under windows in tall halls; a window in a pillar and in a block standing out of its wall: a wall round it now; the lamp's pole an unfogged black bar). `tests/windows.rs` (6): where they stand, the cut and the solids, the same on every build, the light on its own room's floor, the same place from every window and later deeper, the lamp heard only near |
| W2 | `6452ff3` | **The one behind the glass** (`entities/glass.rs`, §4). **Its body** (`assets/src/creatures/body_glass.py`, MakeHuman CC0: the young thin man and woman, half each): a raincoat whose body and sleeves are grown off the skin, the trunk smoothed hard, spanned across its hollows and let fall straight from the chest (a garment grown off the skin had kept the ribs and the bust); a skirt of cloth from the belt to below the knees, pinned taut over the seat and let fall by Blender's cloth, stiff, without self-collision (begun narrower than the hips it had crumpled, and folded into the seat's cleft); a belt and buckle over the join; a hood that is a shape of its own (grown off the skull it had been a helmet, then flaps): a shell a few centimetres off the skull, peaked a little at the back, hanging from its widest round to the shoulders, an oval cut for the face, drawn in and forward of it, its inside darker; everything it wears weighted from the skin under it, the hood by the head, neck and chest only (by bone heat a hanging sleeve had followed the trunk; by the shoulders' skin the hood rose with the arms into two flat sheets); its face cold and drowned (bluish, veined, its lids raw, its lips grey, its eyes clouded), wet; a `press` morph flattening its face as on glass. **Its rule**: your eyes on its glass (your view's line through one of its window's lights within 8 m, nothing between, uncovered) add up across glances; at 3 s the pane ticks (it has seen you), at 5 s it comes up the track at 2.6 m/s only while you look and never goes back; in its lamp's light (7 m) it comes to the glass on its own (the creak, its breath misting the pane: `Patch::Glass`'s mist where its face is), and 2 s later, unless the glass is covered or the place lies quiet, the pane bows in round it (its window's panes made a 12 × 16 grid when it is placed) and it comes through over the sill (2.4 s) into its light; inside it is only where that light falls (`lit_at`), at its edge nearest you, and its touch (`touch_nerve` 40) throws you down out of the light; out of its sight 4 s (or 12 m) it goes back out (2 s) to 1.4 m beyond the glass. It always faces you (at the glass within 34° of the way in). **Drawn beyond the glass** (`Material::portal` on every part of it: past the window's plane it is slid onto the slab and seen only through the opening, half in and half out as it comes through). It starts 32 m out (at 55 m the fog had taken it entirely: a smudge, not nothing). **Placed** after the build with no draws (`glass::place`, `with_seed`), on 16 of 38 floors with windows (a coin of 0.65 from the level's seed), its window's first open tile off the main way (`corner::main_way`, shared), 8 steps from the arrival, 5 from the door and her camp (measured 0.8 m before the cut, 61 of 166 lights had been judged from inside their own wall). Never sent; standing out there it is nothing to the director and nothing reaches the room (no pressure through its wall, no sighting through its glass: `ai.rs`); a showcase stages it at its window (`--showcase glass:OUT`). **Its sound** is glass only, heard only where its glass is (within 9 m, in its room): the tick (a click and the pane's ring), the creak, its breath, the groan. Tests: `tests/glass.rs` (9): where it stands, the look adding up, coming only while looked at, to the glass and through, not through covered glass or in a quiet place, held to its light and its touch, going back out, nothing to the director; its counter on real floors (an arena room has no window): with it 100% untouched over 21 floors (it never came in), without it 48% (it came in on every one; the rest stopped at the edge of its light, out of its reach); `listen.rs` (each sound over the room in its band: tick +15, creak +8, breath +14, groan +12 dB; half-second windows had hidden the tick); the rule-follower's counter (eyes off its glass once it has some of your look; out of its light); five goldens. **Blind reviews** (two fresh reviewers, both orders, the art-critic's brief, against the first draft of its body): preferred in 10 of 12 views by each (the far smudge tied); what they found broken on its side fixed (the hood's fins with raised arms, the sleeves seen into their cuffs, an inked ring round the face, a matte habit rather than a wet coat). Not yet: the blind as a verb and what it learns (W3); its grip drawing you to the glass is not built (its touch is the shared one) |
| W3 | `e39bf74` | **Covering, and what it learns.** **The blind as a verb** (`core_props/window/blind.rs`): each blind is its own now (`Hang`: the lights it covers, where it hangs, its slats and cords two meshes of their own; before, a window's blinds were merged with its parts), `[E] LOWER THE BLIND` lets it all the way down and shut over a second, `[E] RAISE THE BLIND` draws it up, drawn afresh fifteen times a second on its way; its clatter (dull aluminium slats, 1.5–2.4 kHz, well under the pane's tick, and the bottom rail's knock) made only where it is heard, and heard 10 m along the way by what hears (`window_noise`, as the vending machine's). A covered light (its blind down and shut, or papered) throws no light (`throw_light`, `lit_at`, `lit_cells`; thrown again when a blind comes to rest) and takes no look; a row whose every light is covered goes quiet (its lamp's ballast). **The one behind the glass, covered**: on its way to a covered glass it stands where it was; at a covered pane, or at the glass while the place lies quiet, it waits (Alert: nothing to the director; as a hunt waiting behind a blind it had held the floor in a peak with no end, which the tests had not seen, running it without the director: the lore review found it), and when the blind goes up or the quiet ends it presses again, two seconds at the glass before it comes; inside when its row's light is all covered, it goes back out. Its glass is its row's: your look counts through any of the row's lights (it is seen through all of them), its light is all of theirs; it comes through its own. Stalk is a search to the director only while it comes. Its tells moved off breath (four creatures breathe): at the glass wet skin squeaking on the pane, inside its wet coat rustling. **What it learns** (`creature::LEARNS`, sixth): got away from twice out of its light, the lamps outside the windows hang 1 m lower (`view::set_lamp_drop`: head and lens down the pole, the pole shorter, the view's pool and halo with them; every window's of the floor, so its own stays unmarked), their light thrown further in (4.3 m into the room for 2.6 m; 1.6 m lower had thrown it 7.3 m); the first time where you can see it (in its window's room, near: the light creeping in over 3 s), after that on every floor with windows from the first (fitted after the build). The lore review (a fresh `lore-keeper`, W1–W3): the endless peak, the cover that did not stop it near the glass, the lesson that marked its window, its glass narrower than its window, its touch misdescribed, sameness (breath, the rattle in its tick's band, "someone stands"), attribution without authors; all fixed or written into the canon (`LORE.md`: covering, its lesson in the list of what they learn, the clatter both ways, the quiet row, party rooms, the sources' authors; the files keep their attribution, the corner's too). `--screen blind-down` and `lamp-low` with `--showcase glass:…` for evidence. Tests: `tests/glass.rs` (14): a blind let down covers its glass and puts out its light (it goes back out; raised, the light is back), its clatter brings what hears, got away from twice its lamps hang lower and its light lies further in (only where seen), covered or quiet it waits and comes only after two more seconds at the glass, nothing to the director while it waits; the arena with a blind let down at the glass (13 of 13 floors kept out) and learned (its counter as before, its lamps low on all); `listen.rs` (its squeak and drips in their bands) |
| W4 | `ac4aebb` | **The canon and the reviews.** Two fresh `lore-keeper` rounds over W1–W3: the first's findings fixed in W3; the second confirmed them and found what was left, fixed here. **Its look after a lapse**: come into its lamp's light on its own it needs your look no longer, a cover or the quiet only holding it (`coming`); covered on the way it had waited for your eyes again. Gone back out, a long look still brings it. **No relief while it was in the room**: inside it was a hunt to the director whatever happened, so standing at the edge of its light held a peak with no end; now it backs off while the place lies quiet (as they all do), and once it has watched you keep out of its light 12 s (`KEPT_OUT`), and either is a getting away from it only if it saw you in there (an empty room had counted). **Sounds through the wall**: the lamp's ballast and its sounds in the room are heard only from its room (a line through it). **Sameness**: its tick was the Smiler's click (now a dull knock and the pane's inharmonic ring, 1.2/2.7 kHz), its drips in the room were the wet floors' own (now its wet coat rustling), the squeak moved up out of the blind's clatter, "figure" is the void's (now a pale raincoat). The canon (`LORE.md`, `ENTITIES.md`, `WINDOWS.md`, the entities rules): six creatures, its lesson in every list, seen through one window only (the track empty through the rest), its breath seen and never heard, "never back up the track", party rooms, the touch's wording everywhere. Tests: `tests/glass.rs` (17): come into its light, a cover only holds it; gone back out, a long look again; inside it backs off when the place is quiet or you keep out of its light, and an empty room is no getting away; `listen.rs` (the tick, the squeak and the rustle in their new bands). The blind reviews of the windows (W1) and of the figure (W2, two reviewers) stand |

## What only people can tell

Whether a window reads as a window or as a picture; whether the view is
unsettling or merely pretty; whether players notice the same view from
every window, and the wall behind; whether the smudge on the track draws a
stare, and whether "do not stare too long" is learned from one meeting;
whether its light on the floor reads as its reach.
