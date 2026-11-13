# The Building: the megastructure above the rooms

What the first three floors are, why, and the plan that turns them from
"the yellow rooms, then more yellow rooms" into a vast, empty concrete
megastructure you have to find your way down through, with a threat you
see coming from far off, before the descent closes in on the rooms. The
canon it changes is [`LORE.md`](LORE.md); the pillars it serves are in
[`PILLARS.md`](PILLARS.md); the research behind it is in
[`SOURCES.md`](../research/SOURCES.md) section 10.

## The work

How this was planned and done (the analysis, milestones, progress, the second pass): [`../plan/MEGASTRUCTURE.md`](../plan/MEGASTRUCTURE.md).

## What the research says

Section 10 of [`SOURCES.md`](../research/SOURCES.md) has the sources; in short:

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
3. **The look of the Building.** Concrete and paint in the light map, its
   surfaces photographs of Soviet-era places (`visuals::photo`, all CC0:
   a panel block's facade and its windows, whitewashed and formwork
   concrete, a yard's cracked slab and a factory's, the granite of a 1949
   Moscow ministry and of a Kyiv metro station, ribbed precast slabs); walls
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

## What only people can tell

- Do the Building's floors read as vast and empty, or as a maze with a
  drop beside it?
- Is finding the way down a puzzle they enjoy, or a chore? How long does a
  floor take, and where do they stop?
- Do they understand the siren and the cells the first time, without being
  told? Does the fog feel far, then near?
- Is the change at level 4, from the concrete to the yellow rooms, felt?
