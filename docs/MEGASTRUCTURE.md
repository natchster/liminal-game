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
   shape; turn the wheel and you are sealed in while something knocks at
   the door. Caught in the fog, your mind goes fast. The director runs it
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
   notices, the cleaners' log and the surveys of the three places are
   new torn notes that lie only on the Building's floors.
7. **Distant presences and the way down.** The exit's light is the
   weenie; a few landmarks (a lit tower of windows, a lift cage over the
   void); now and then a figure far across the void, standing where no walk
   reaches, who turns away when you look. The last door of the Building, in
   the Works, is an ordinary wooden door set in the machine wall, small in
   it, warm, with the Lobby's yellow round it: the rooms showing through.

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
| B7 | **Distant presences and the way down** (design 7) | tests: the figure stays out of reach and turns away when seen; the door's light seen from the arrival's views; the last door into the Lobby; PNGs |
| B8 | **Evidence and the ledger**: reviews, playtest questions, `content.json`, `QUALITY.md`, the full check and the bench | full check and drift sweep pass |

## Progress

| # | Commit | Notes |
|---|---|---|
| B0 | `a38c482` | the plan |
| B1 | `26452ce` | `archgen::Ground` (a floor, a void, a stair from `h0` to `h1` along `dir`, a mass from its foot to its top) in `Layout::ground` (empty on the rooms) and `Game::ground`; `ground_breaks` names every pair of open neighbours that do not meet at one height (two flights side by side with the same run are one wide flight). `build_ground_solids`: a mass is a box, a void a box of kind `BOX_VOID` that stops a body and no look (`segment_clear`, `ray_axis` skip it), a floor a slab of kind `BOX_FLOOR` that stops a look and no body (`circle_hits` skips it). `segment_clear_3d`; `los` runs it between 1.5 m over each end's own floor when the level has ground. `floor_y_at` / `floor_y_of`; the body stands on the floor under it, the camera, dropped paper and the floor arrows too. On the rooms everything adds 0.0 and writes nothing new: every fixture, golden, look anchor and the range baseline are unchanged (full check). `tests/height.rs` on `Game::test_ground_level`, a 9 × 5 floor by hand: down a flight and back up, stopped by a void, a wall between two heights and a flight's side, seen across a void, not through a slab or a wall |
| B2 | `aa66055` | `level/building.rs` (pure, its own draws only): a lattice of 2 × 2 platforms every 6 tiles (7 a side, a 40-tile floor, 160 m), a spanning tree of walks grown by a random depth-first walk from the arrival (long ways round), a walk across a missing platform now and then (a 10-tile bridge), loops kept only while the way to the door stays at 80% of the tree's and two walks still go nowhere. The door is the far point of the tree on the outer wall; the floor falls from the arrival to it (fastest near the arrival: you arrive high over the rest), a step up now and then; a walk climbs only what its length allows (a storey a tile; the Works' ramps two tiles a storey). Squares between the walks: the Well's ring of building round its well, the Halls' void with stone piers, the Works' terraces where their four platforms stand level (a terrace that shortens the way is undone). The door's light is the lamp over it and its stair tower's windows lit a storey apart up to the arrival's height (`door_lights`); building on the clearest line from near the arrival to one of them is cut away. Cells are cut where the farthest floor is from one; where no building is near, a booth (3 × 2 tiles) stands out over the void beside the walk. `navigation` measures a floor; over 40 seeds a style: detour 3.9–4.2 (at least 2.15), 9–16 flights on the way, 2+ choices and 2+ dead ends, a cell within 16 tiles of every floor, the door seen from near the arrival on 75–88% of floors. First drafts failed on each: the door was never seen (the ring of building round the arrival, then floors that fall evenly along a winding way, grazed by any straight line), loops cut the way to a Manhattan walk (detour 1.41), a terrace could cut the Works short, and wide void left floor 70 tiles from a cell. `ascii` dumps (`target/building/*.txt`, four seeds a style) |
| B3 | `c3ecfaf` | Levels 1–3 are the Building's floors (`gen_building_level`, `world::building`): slabs under the floors, flights of steps, a parapet on every edge over the void and none elsewhere, masses from the fog 100 m down to 150 m up, a lamp over every cell and over the platforms where ways meet, the door set in its wall, water, a battery and a note or two at the dead ends and in the cells, then her story as on the rooms. A first dressing (`world::building_dress`, on a salted stream of its own): every cell has its front wall with a doorway (a solid of the plan's, `cell_front_boxes`), its hermetic door open inside, wheel, number, the lamp's cage, a bench; the Well's galleries have their sealed doors (numbered, some marked red or taped round; E knocks, and now and then something knocks back) and the galleries of nine storeys above and below; windows by the thousand on its panel faces (a new surface, a window in every panel), a few lit, and the door's tower lit up to the arrival's height; storey numbers by the flights; loudspeakers; chains and unreachable flights in the Stair Halls (new ashlar and flagstone surfaces); a ceiling, gantries and pipes over the Works, whose masses are machine housings 4–9 m over their floors. The rooms from level 4, rot and creatures counted by `rooms_depth` (the Lobby is depth 1, its first room the photograph and its first quiet the long one); the anomalies on 5–12; nothing walks the Building; a thin grey haze and a 230 m far plane there; the title looks across the void at the door's light; the save is v2. Found on the way: a lintel at a cell's doorway stopped a body below y = 0 (what is over a body's head now counts from its floor; the level sweep caught it); the door's call came from 1.2 m over y = 0 on every floor; the listen test's signature claim held on three floors by luck (over eight, 57% of signatures rose 6 dB): the thud, clank, moan, laugh, radio, paper and dial tone were raised (most of those in the open rise 6 dB now, 29 of 33) and the claim states the walls; the signatures test read a shared random stream (now seeded per floor). The reference tests that set a level meaning a depth now set the game level with that depth (no fixture changed for it); the archgen plan fixtures are checked against the roster they were recorded with. Re-recorded with reasons: reference levels, loudness (not yet listened to by a person), goldens and look anchors (ten level views no longer exist), the range baseline; STYLE bands for the three places by the bands' rule |
| B4 | `b90f0bc` | The look, place by place: a railing of each place's own on every edge over the void and on no other (the Well's poured parapet, the Stair Halls' stone balustrade with its balusters, the Works' steel posts and rails with a yellow top rail and toe board), sloping with the flights; a lamp at the foot of every flight beside the first steps (what can be walked is lit where it starts), on a post where no wall is near; the lamps as bulkheads (the Well), iron lanterns (the Stair Halls) and floodlights (the Works); the Building lit at half (0.5); a long, sparse reverb for each place (4.4–5.2 s); and the Building's own air instead of the rooms' weathers (no "the air is still", no haunted tint). The three places' bands re-measured by the rule (16 views each, 94–100% inside); only the Building's six golden frames moved. Reference levels re-recorded for the three floors (the lamps draw from the build's stream) |
| B5 | `7ac8934` | `world::assembly`: on the Building's floors the director runs the self-assembly in its own phases (calm: a quiet of 75–115 s, the first 95–130, the first floor's 120–160 while her camp is there; build: the siren, a held wail rising and falling from the two nearest loudspeakers every 9 s, while the fog rises out of the void 30 m below to the lowest walk in 42 s; peak: it rises among the walks at 0.6 m/s to a crest between halfway and over every walk, and stands 15–22 s; relax: it sinks at 1.2 m/s, then the all-clear). Its surface is three drifting layers of violet over the floor, the lamps under it go out, and with your eyes under it the fog is all you see. Out in it the mind goes (11 a second, 3 wading) and you are gone after 10 s; sealed in a cell you are safe while something knocks at the door. Every cell's door is a leaf on its hinge with a wheel: from inside, E turns it shut (it stops a body) or open. Her page sounds the siren within 4–9 s. `tests/assembly.rs`: the stages in order and the fog rising then sinking, the director's log `build`, `peak`, `relax`; a sealed cell keeps it out and outside you die of it; a shut door stops a body; her page sounds the siren on all three floors; a cell within reach of the warning at a walk (16 tiles: 18 s of 42); a twelve-minute pacing log a floor (three assemblies each, first siren at 111–139 s, quiets of 86–115 s, the fog up 41–79 s). First drafts: the fog stood four minutes a time (too long to wait in a cell) and dipped a metre as the siren ended; and its texture's noise and the first quiet drew from the native stream at a floor's start, which moved every golden frame after level 1 (headless runs build level 1 behind the title): the texture paints from a stream of its own now, and the quiet spreads the director's one draw. `the_siren_is_heard` (listen): +21.8 dB over the Well's bed, a held wail (`target/listen/siren_4242_L1.png`); not yet listened to by a person. `--screen fog-below` / `fog-in` |
| B6 | `bd7b043` | Her story on the Building's floors: her camp is a cell (against a side wall, its bench at the back), her trail runs over the heights (each sheet seen from the last in three dimensions, none on a flight, spaced by depth), and her first four pages are the three places and the Lobby, whatever the run (f1 the Well and the wheel, f2 the Stair Halls and the fog, f3 the Works, his camp, and "a small door, and it was warm behind it", f4 the yellow rooms; f5 the tall ones; "not all the paper down here is mine" is f6 now, so a trail that is not hers starts one floor later: levels 6 and 7 of the reference runs re-recorded for it). Seven torn notes of the Building's own (ids 52–58, tag `building`: the residents' notice, a cell plate, the cleaners' log, the surveys of the three places, a ration coupon) lie two a floor on its floors and nowhere else, and no other notes lie there; a seventh arc, THE BUILDING. E.'s first day and the note of the machines read as the Building. A line under the level card on each place's arrival. Cell plates read `NNN-L`; the red mark on a sealed door is the cleaners'. The Works' last door is a small wooden door in the machine wall with the Lobby's yellow round it and a warm light over it (design 7, built here because f3 promises it). The `lore-keeper` review's findings were all taken: the monotony exception for the Building, the Lobby leaking into the rooms and not up, the attribution of the knocking, the self-assembly's naming. `tests/story.rs`: `the_buildings_notes_lie_only_on_its_floors`; `tests/assembly.rs`: the camp in a cell and the first four pages on levels 1–4. Re-recorded: reference levels (levels 1–3; 6–7 for the false trail), five Building goldens (the plates' letter draws from the dressing's stream, which moves the Well's sealed doors after it) |
| B7 | `9a46e9f` | `world::far_figures`: now and then a figure stands far off across the void where no walk reaches (a gallery of another storey at its parapet in the Well, the middle of a flight nobody can reach in the Stair Halls, a machine's top or a gantry in the Works; never within a storey of a walk), 22–60 m off, turned toward you: looked at for 1.2 s it turns its back, and it is gone the moment you look away (or come within 14 m); it stands again 100–220 s later, somewhere else, never while the siren sounds or the fog is up (one standing when the siren starts turns away), and first 70 s into a floor (B8, after the lore review: 45–100 s was several sightings a floor). Its choices draw from a stream of its own (no build or native draw moves). It stands only where it can be seen and stand out: nothing between your eye and its head, chest and knees, not the solids nor what the dressing builds over the walks (a flight's body, a floor's slab, the Well's decks of the other storeys, a lamp's head); not far under your eye nor overhead (your own parapet would hide it); and something behind it, a face or floor within 35 m or the haze, not the void's depths nor the Works' ceiling. First drafts were invisible in every frame: a dark figure is lost in the Building's dark, so it is grey and unlit, a little paler than the concrete; and the Works' machine housings were drawn to 150 m whatever their plan said (the solids stopped at their tops), so a figure on one stood inside a tower: every mass is drawn to its own top now, and the Works opens up under its ceiling. The Well's lift cage hangs in the open shaft on its cables, its bulb lit, its floor 1.5 m over a walk near the arrival that sees it (level with the eye across the void). The Works' last door (B6) is named for its test. `tests/presences.rs`: every perch a storey off every walk round it, measured from the walks' own floors; the figure stands 22–60 m off, waits unlooked at, turns its back when looked at and stays while watched (for up to 12 s), and is gone when you look away, for 100–220 s; walking straight at it, looking at it, it is gone before you are within 8.4 m (a step's slack); it waits out the fog; the door's light seen from the arrival on 11 of 12 floors with every solid of the built floor in the way; a lift cage over open void on 3+ of 4 Well floors; the Works' door lined with the Lobby's yellow, and through it level 4, the Lobby. `--screen figure` (levels 1–3) and `--screen lift` (level 1). Re-recorded: one golden (`4242_L3_spawn`, a housing far down the corridor stands lower) |
| B8 | (this commit) | Reviews, playtest questions, the ledger. The `code-reviewer` (the whole series): the level-start draws, the state carried between floors, borrows and layering hold; it found the fog's own sound placed at the player ± 6 m (now from one of the void's tiles near you, chosen by the same two draws: invariant 2), the in-fog view shown inside a sealed cell (now not), a figure in mid-air over the Well's islands (its Well perches now come from the decks where the dressing lays them) and stale commands and counts in the docs (the creature showcase on a floor of the rooms, 33 goldens, no anomaly on levels 1–4). The `lore-keeper` (the final pass): no channel break; contradictions taken (flights "nobody has stood on" that the figure stands on, "nobody is in it", storeys poured where galleries are drawn, warm machines that blurred the warm door, the self-assembly named on a coupon, the cleaners known by two different things), the figure recorded as built and classed (not a creature, nothing sends it), rarer (first at 70 s, then 100–220 s apart, never the same perch twice running, gone when the siren starts), and the Well's windows and drop and her counting said once each (new arrival lines). The `level-analyst`: no drift against the B3 baseline (energy distance 0.095, p = 0.54); the one moved measure, the Building's light density (×1.8–2.3), from B4's lamps at the foot of every flight; the three places 100% inside their `STYLE.md` bands; the Works after B7 one hall under its ceiling; the figure read as a bottle at 26 × 9 px and stood 0.24 m in the air (now legs from the ground, shoulders, a head over a neck) and a deck's thickness and parapet were missed by the sight rule (now not). `PLAYTEST.md`: the Building's debrief questions and listening to the siren before loudness is re-recorded. `QUALITY.md`: the Building at B. Full check and drift sweep pass |

## What only people can tell

- Do the Building's floors read as vast and empty, or as a maze with a
  drop beside it?
- Is finding the way down a puzzle they enjoy, or a chore? How long does a
  floor take, and where do they stop?
- Do they understand the siren and the cells the first time, without being
  told? Does the fog feel far, then near?
- Is the change at level 4, from the concrete to the yellow rooms, felt?
