# The Building: the megastructure above the rooms

What the first three floors are, why, and the plan that turns them from
"the yellow rooms, then more yellow rooms" into a vast, empty concrete
megastructure you have to find your way down through, with a threat you
see coming from far off, before the descent closes in on the rooms. The
canon it changes is [`LORE.md`](LORE.md); the pillars it serves are in
[`PILLARS.md`](PILLARS.md); the research behind it is in
[`SOURCES.md`](SOURCES.md) section 10.

## The analysis (2026-10-27)

What the first floors were, read from the code:

1. Every level is one flat plane at y = 0 (`player.rs`: the camera is
   `EYE` over the floor; `world/arch.rs`: floor quads at 0). A level is a
   31–53 tile grid of 4 m tiles (124–212 m) cut into zones, each with its
   own ceiling (2.35–7.6 m). The longest view is a corridor; the far plane
   is clamped to 28–90 m (`sync_far_to_fog`).
2. Level 1 is the Lobby (always, the photograph first), 2 the Habitable
   Zone, 3 the Utility Halls, 4 the Electrical Station, 5 the Office
   (`archgen::DOMINANT_BY_LEVEL`). Each is enclosed: rooms, halls,
   tunnels.
3. The threat is near from the first floor: one to three creatures walk the
   same plane as you (`spawn_entities(1 + (n-1)/2 …)`), sent by the director
   to the tiles round you that you cannot see.
4. The challenge of a level is to find a door in a plan you cannot see
   over, and to get away from what is sent after you.

Measured against the brief:

| What is asked | Where the game stands |
|---|---|
| Large, empty, strange megastructure on the first three floors | No floor has a height; nothing is ever below or above you; no view is longer than a hall |
| Navigation of a complex landscape as the challenge | The plan is a maze on one plane; no stairs, drops, bridges or galleries; no "see it, then find the way to it" |
| A distant threat there | The creatures are on your plane and come to you |
| Enclosed Backrooms horror, threat lurking near, afterwards | That is the whole game now: it stays, from level 4 |
| Lore notes and mechanics that match | The canon begins in the Lobby; her first page arrives in yellow rooms |

## What the research says

Section 10 of [`SOURCES.md`](SOURCES.md) has the sources; in short:

- **Samosbor** (самосбор, "self-assembly") is an anonymous Russian
  imageboard myth, begun on 2ch's /b/ in spring 2018 as a six-line
  "bugurt" ("you live on floor 556 of structure P-46 / last night there was
  a samosbor again"), with no single author. Its world, the
  *Gigakhrushch*, is an endless Soviet panel-housing block with no outside
  and no sky. Its event is a siren, then a fog (crimson, lilac, violet)
  and the smell of raw meat; the only shelter is a sealed hermetic door
  (гермодверь), and you never open it before the siren stops, whoever begs
  at it. Liquidators clean up after, weld doors shut and pour foam concrete
  over whole floors. Its community's rules: "the building is infinite, the
  samosbor is unknowable"; never explain it.
- **Megastructure SCPs and Backrooms levels** give the architecture: the
  Backrooms' Level 188 (a white atrium with windows at exact intervals),
  Piranesi's *Carceri* and SCP-2723's open-sided tower of stairs, SCP-4370's
  works whose ceilings are lost in the dark, SCP-5778's huge tunnels with
  signs and lights for nobody, SCP-3074's descending car park, SCP-3333's
  figures on a far ridge who turn away when looked at. Their motifs:
  megalophobia, kenopsia, no outside, repetition, builders who never stop,
  infrastructure still running, signage for people who are gone.
- **Wayfinding.** Getting lost is frustrating unless the game is about
  navigation; then it has to be designed (Oueijan, GDC 2021). Show the goal
  and hide the route (Disney's weenie, Journey's mountain, Dark Souls'
  "see it, then find the way to it"); edges are deep, wide barriers
  (chasms); topology, the number of decision points, decides difficulty
  more than landmark looks do (Yesiltepe et al.). The language of what can
  be walked must be consistent everywhere (Mirror's Edge's lesson). Dead
  ends should pay (a view, a cache, a shortcut). Downhill reads as forward
  (Breath of the Wild).
- **Distant threats.** Get-to-shelter mechanics (S.T.A.L.K.E.R.'s emission,
  Rain World's rain) are fair when the warning comes in several channels,
  the nearest shelter is always reachable in the warning, shelters look
  the same everywhere, and the approach is visible as a moving front, not
  a timer. A distant giant needs cover or it is unfair (Lethal Company).
  In vast spaces the threat is seen and far (prospect); in enclosed ones it
  is unseen and near.
- **Scale in first person:** fog over far geometry, repeated human-sized
  modules receding into it (balconies, windows, columns), long reverbs,
  compression and release at thresholds.

## The design

**One sentence:** you arrive not in the yellow rooms but in the Building
they are inside: three floors of vast, empty concrete you find your way
down through, while something gathers far off, and then the rooms close
in.

1. **Height.** A floor of the Building has a floor height per tile: walks
   at many heights, stair flights between them, and voids (no floor,
   a drop into fog) that you cannot enter but can see across. Heights
   change only along a stair or across a wall or a void, so every grid
   search (reachability, her walk, the exit) stays right. The camera, the
   body, sight lines, props and sounds stand on the floor of their tile.
   The rooms keep y = 0 and do not move.
2. **The Building's floors.** A generator of its own grows the walks from
   where you arrive: galleries along great masses of building, bridges
   over the void, flights of stairs down (and sometimes up), landings,
   dead ends that end on a view, loops back. The door is far and low;
   you see its light across the void long before you find the way to it.
   Three floors, three places:
   - **Level 1, the Well:** galleries round a light well with no top and
     no bottom, the face of the block opposite all windows and sealed
     doors (the Gigakhrushch, Level 188).
   - **Level 2, the Stair Halls:** a hall with no far wall, piers and
     arches, flights of stairs crossing at every height, bridges with no
     rail (Piranesi's *Carceri*, SCP-2723).
   - **Level 3, the Works:** floors of dead machinery the size of a town,
     ramps and catwalks over shafts, signs and lamps for nobody, a ceiling
     lost in the dark; its door is a small one, and behind it is yellow
     (SCP-4370, SCP-5778).
3. **The look of the Building.** Concrete and paint in the light map; walls
   that rise out of sight and fall into fog; windows in their thousands,
   a few lit; a parapet on every edge you can walk, and none on what you
   cannot reach; lamps over the cells; long reverb; a far plane and fog for
   a hundred metres and more.
4. **The self-assembly.** No creature walks the Building's floors. What
   comes is the self-assembly, from far off: a siren from the speakers on
   the walls, then a violet fog rising out of the void, its surface seen
   from above, the lamps under it going out one by one. Shelter is a
   *cell*: a recess with a hermetic door, a lamp over it, always the same
   shape; turn the wheel and you are sealed in while voices beg at the
   door. Caught in the fog, your mind goes fast. The director runs it
   (calm, the siren, the fog, the all-clear); every point of a floor has a
   cell within reach of the warning at a walk. Taking her page sounds the
   siren.
5. **The descent's new shape.** Levels 1–3 are the Building; level 4 is the
   Lobby (the photograph first) and the rooms go on as before, counted
   from there (rot, size, creatures, the first long quiet). The anomalies
   come between levels 5 and 12; her last camp stays on level 13, and her
   thirteen pages still fit (eight ordinary floors: three of the Building,
   five of the rooms).
6. **Her story and the notes.** Her trail runs over the heights (each sheet
   seen from the last, in three dimensions); her camp is a cell. Her first
   pages are the Building's; her fourth is the yellow rooms. The residents'
   notices, the liquidators' log and the surveys of the three places are
   new torn notes that lie only on the Building's floors.
7. **Distant presences and the way down.** The exit's light is the
   weenie; a few landmarks (a lit tower of windows, a lift cage over the
   void); now and then a figure far across the void, standing where no walk
   reaches, who turns away when you look. The last door of the Building is
   a cell's door that opens on the other side into the Lobby.

## Milestones

Each ends with its gates passing (`cargo xtask check --fast`, and the full
check where it touches generation, rendering or audio), evidence (a test,
a headless PNG, a log), its own commit, and the playable bundle rebuilt
(`cargo xtask bundle`).

| # | Milestone | Done when |
|---|---|---|
| B0 | This plan; `SOURCES.md` section 10 | committed |
| B1 | **Height in the world** (design 1): per-tile floor heights, stair tiles and voids in the layout; solids that block movement but not sight (voids) and sight but not movement (floor slabs); the body, the camera, sight lines and everything placed stand on their tile's floor | tests on a hand-made layout: up and down a flight, stopped at a void and a cliff, seen across a void, not through a floor; every rooms fixture and golden unchanged |
| B2 | **The Building's generator** (design 2): `level/building.rs`, pure, three styles | tests: deterministic, every walk reachable, the height rule holds, the door the lowest far point, and the navigation measures in their bands (detour index, height changes, decision points, dead ends, the door seen from the arrival); map dumps of a dozen seeds |
| B3 | **The descent's new shape** (design 5) and a first Building: levels 1–3 built from the generator with plain architecture; the rooms from level 4; anomalies 5–12; no creatures on 1–3; level cards; the save's version | the playthrough, story, pacing and personas pass on the new numbering; reference levels, goldens and the range baseline re-recorded with reasons; PNGs of the three floors |
| B4 | **The look of the Building** (design 3) | zone and exit views of the three floors; the look and luminance bands; the bench in budget |
| B5 | **The self-assembly** (design 4) | tests: the siren before the fog, the fog rising and falling, a sealed cell is safe, the fog kills outside, her page sounds the siren, every tile within reach of a cell; a pacing log of the Building's floors; a listen PNG of the siren |
| B6 | **Her story and the notes** (design 6) | the story tests on the Building's floors (followable over heights, camp in a cell); the notebook in order; the new notes only on the Building; `LORE.md`, `PILLARS.md`, `GAMEPLAY.md`; a `lore-keeper` review |
| B7 | **Distant presences and the way down** (design 7) | tests: the watcher stays out of reach and turns away when seen; the door's light seen from the arrival's views; the last door into the Lobby; PNGs |
| B8 | **Evidence and the ledger**: reviews, playtest questions, `content.json`, `QUALITY.md`, the full check and the bench | full check and drift sweep pass |

## Progress

| # | Commit | Notes |
|---|---|---|
| B0 | `a38c482` | the plan |
| B1 | (this commit) | `archgen::Ground` (a floor, a void, a stair from `h0` to `h1` along `dir`, a mass from its foot to its top) in `Layout::ground` (empty on the rooms) and `Game::ground`; `ground_breaks` names every pair of open neighbours that do not meet at one height (two flights side by side with the same run are one wide flight). `build_ground_solids`: a mass is a box, a void a box of kind `BOX_VOID` that stops a body and no look (`segment_clear`, `ray_axis` skip it), a floor a slab of kind `BOX_FLOOR` that stops a look and no body (`circle_hits` skips it). `segment_clear_3d`; `los` runs it between 1.5 m over each end's own floor when the level has ground. `floor_y_at` / `floor_y_of`; the body stands on the floor under it, the camera, dropped paper and the floor arrows too. On the rooms everything adds 0.0 and writes nothing new: every fixture, golden, look anchor and the range baseline are unchanged (full check). `tests/height.rs` on `Game::test_ground_level`, a 9 × 5 floor by hand: down a flight and back up, stopped by a void, a wall between two heights and a flight's side, seen across a void, not through a slab or a wall |

## What only people can tell

- Do the Building's floors read as vast and empty, or as a maze with a
  drop beside it?
- Is finding the way down a puzzle they enjoy, or a chore? How long does a
  floor take, and where do they stop?
- Do they understand the siren and the cells the first time, without being
  told? Does the fog feel far, then near?
- Is the change at level 4, from the concrete to the yellow rooms, felt?
