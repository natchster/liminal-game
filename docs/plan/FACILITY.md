# The work on the facility

The deeper floors of the rooms given the shape of underground plant: the
cableways, column halls, vaults and pit halls of the mid-century's
bunkers, powerhouses and mines, made from real places
([`../research/facility.md`](../research/facility.md)), as the Lobby is
office architecture with no office in it. The Lobby and the office floors
stay as they are, the Backrooms' way in. What is still open is queued in
[`MILESTONES.md`](MILESTONES.md).

## The brief (2026-11-17)

The developer: "revamp the backrooms visuals more in line with portal 2
aesthetics (the underground facilities, not the lab rooms)", and, asked
where and how far: the deeper floors ("keep the yellow Lobby and office
floors as the Backrooms' entrance; the deeper floors turn into the
facility: the Utility Halls, service tunnels and Habitable Zone become
vaults, shafts and catwalks, plus a new archetype or two"), all of it,
each milestone committed.

Read as: the mood of that game's old underground facility (vast dug
spaces of the mid-century gone to ruin; concrete and steel; catwalks over
drops; pipes; institutional paint), reached only through real places.
Nothing is taken from the game: no names, logos, signage, characters or
level layouts, nor its own inventions (its spheres, panels, test chambers,
an old facility buried under a new one, eras ordered by depth); it is a
tone reference only (`LORE.md`, *Attribution and licensing*). It serves the
canon's premise, "variety is earned by descending" (`LORE.md`, *Premise*),
and pillar 2, *familiar but wrong*: "real places, slightly off"
(`PILLARS.md`).

## The analysis (2026-11-17)

What decides a rooms floor and how it looks (a read of the code; paths in
the milestones).

**Which archetype, by depth.** `level::archgen` builds a rooms floor from
its depth (`rooms_depth`: game level 4 is depth 1). One archetype
dominates each floor (`DOMINANT_BY_LEVEL`, then `DEEP_DOMINANT` round a
cycle of eight), and the rest are mixed in by weight; the Lobby keeps
weight 2 on every deep floor (about one zone in six: the canon's "the
Lobby leaks into every level"). The floors are not strata: the office
floor (8) lies below the industrial ones (5-7).

| Game level | Depth | Dominant (its share of zones) |
|---|---|---|
| 4 | 1 | lobby (87%) |
| 5 | 2 | hall, the Habitable Zone (44%) |
| 6 | 3 | tunnels, the Utility Halls (43%) |
| 7 | 4 | electrical (41%) |
| 8 | 5 | office (43%) |
| 9-16 | 6-13 | pillars, maze, office, tunnels, hall, lobby, pool, electrical; then again |

**What an archetype is.** Its numbers live in `ARCHETYPES` (ceiling, wall
kind and thickness, pillars, materials, a panel every `light_every`
tiles) and its carver; the industrial three are tunnels (2.35-2.65 m,
thick bulging concrete), electrical (2.6-3.1 m, brick) and hall (5.4-7.6
m, concrete, a column grid, fixtures on rods).

**No height below the Building.** A rooms floor is one storey at y = 0:
no void, pit or catwalk (the Building's `Ground`, `BOX_VOID` and railings
exist only for levels 1-3, and `build_solids_leaning` drops the rooms'
walls when a floor has ground). The light map's ceilings stop at 10 m and
the rooms' far plane is 90 m. A floor hole is the one drop: step in and
you fall one level ("You fell").

**One lamp, one colour.** Every rooms zone is lit by one fluorescent
panel (`panel_geo`, warm white), its pooled spots set to one colour each
frame; ambient, hemisphere and fog are one warm yellow; the light map is
one number a tile, no channel free. No sodium or mercury lamp hangs in the
rooms; the one sodium light is the streetlamp's through a window, and its
orange on a floor is the reach of the one behind the glass.

**Surfaces.** Concrete, brick and tile are canvas painters; the
Building's photographed surfaces (formwork, factory slab, ribbed slab)
resolve in a rooms zone already. Nothing is rusted steel, grating or
painted steel (steel is flat paint: `MILESTONES.md` 16).

**The gates are blind below level 7.** `cargo xtask range` measures
levels 1-6; the goldens' only deep plain floor is 4242 L7.

**What the Works lends, and what it keeps.** Its railing, pipe runs and
the industrial placers (`placers_industrial`) can be borrowed; its dead
machinery the size of houses, its terraces, its gantries (the figure's
perches) and its ceiling "so far up its lamps look like stars" are its
own, as the Well's void and the broken lift's black are theirs.

## What the real places say

The research ([`../research/facility.md`](../research/facility.md)), as
the milestones will use it:

- **A scale ladder** for the rooms' scale (the Building keeps "too big for
  you"): shoulder-wide service corridors; cableways 2.9 m across (Titan
  II); a box tunnel 3.0 x 3.4 m (SLAC); galleries 9-11 m wide and 6.4-7 m
  high (Mittelwerk); column halls about 6 m high on a grid (Greenbrier's
  exhibit hall, a cistern, the salt mines' rooms with pillars as wide as
  their rooms).
- **The lamps of the era.** Halophosphate fluorescent (2,940-6,430 K,
  rendering 51-76), the era's and the rooms'; phosphor-coated mercury
  (3,600 K, whiter than clear mercury's green) for a tall hall's high-bay;
  caged incandescent (warm, and silent: no ballast). A tunnel's code
  minimum was 54 lux.
- **Paint and colour.** A two-tone wall dark to the hip and pale above (a
  Titan II corridor); the safety colours (ANSI Z53.1) and pipe colours
  (ANSI A13.1, BS 1710), on the things that carry them.
- **Materials and decay.** Poured concrete and shotcrete, rock bolts in a
  pattern holding mesh, bolted corrugated liner plate, steel floor plate
  and 19-W-4 bar grating; efflorescence, rust bloom and spalling, standing
  water.
- **The code's geometry** for what a body walks: railings 42 in (1.07 m)
  with a mid rail and a 4 in (10 cm) toe board, posts at most 8 ft
  apart.

## The plan

The canon's reading (a fresh `lore-keeper`, 2026-11-17, before anything
was built) made these the plan's rules:

1. **Appearance, not origin.** The deep rooms take the shape of
   underground plant. Never "dug", never a date, never what it made or
   sheltered, never what lies behind the lining; decades are never
   ordered by depth.
2. **Fluorescent stays the rooms' light**, with a high-bay only where a
   real plant needed one (a tall hall), and white (phosphor-coated
   mercury). Light keeps off the canon's colours: no green pool (the door,
   the Loop), no orange on a floor (the window, the one behind the glass),
   no red (the broken lift, the Loop); no caged lamp over a door (the
   cell). A colour is its lamp's alone and dies with it: no tint on
   surfaces. Every lamp stutters, fails and dies in its own real way.
3. **Every lit zone hums from a fixed source** (a ballast; under
   incandescent lamps a transformer or a pump). Silence stays the
   blackout's.
4. **Drops follow the canon.** Every walkable edge over a drop is railed
   and stops a body; a step-off gap is only the floor hole (rare, one
   level down, "You fell"); nothing kills by a fall. A drop has a bottom
   you see (a floor, water), never the void or the black. No ladder or
   stair that looks usable and is not, and none that is a way nobody
   recorded.
5. **One wrong thing a place**, never said: not real decay (stripped is
   real), not another place's rule (the Loop's, the Poolrooms', the
   Works', the Well's); the remade places keep theirs (the Utility Halls'
   bulge, the Habitable Zone's puddles with no source).
6. **Stencils are a thing's own printing**, word for word (a pipe's
   contents, a crane's load, a maker's plate of a made-up maker): no
   labels, levels or numbers, ways, EXIT, rules, dates, radiation or
   shelter signs; no arrows on walls or floors (the flat sheets' mark); no
   retroreflective stripe (her tape is the one thing a lamp brings out).
7. **Blast doors stand as they were left**: an open one is a doorway, a
   shut one keeps every barricade rule; no wheel a hand turns, nothing
   that knocks, none at a Lobby zone's edge.
8. **Nothing of the Building comes down** (the figure, the siren, the
   fog, cells, notices, storey numbers, loudspeakers, the lift cage), and
   the facility joins the deep rotation: the Maze and the Poolrooms keep
   their floors.
9. **Each remake rewrites its torn note and its `LORE.md` row** in its own
   commit (notes 31, 32 and 33: the Habitable Zone, the Utility Halls, the
   Electrical Station), and its `STYLE.md` row.

And the work:

- **Where.** The Lobby and the office floors as they are. The Habitable
  Zone and the Utility Halls take the facility's surfaces, lamps and
  dressing in their canon's shapes; the Electrical Station stays old brick
  and older wiring, its lamps and surfaces from F3 and F4. Two new
  archetypes (a vault, a pit hall) come from depth 6 (level 9) and join
  the deep rotation.
- **Surfaces from free photographs** (CC0: Poly Haven, ambientCG; the
  research lists them) through `cargo xtask assets textures`, as the
  Building's were; any CC BY photograph credited.
- **The gates first**: the range and the goldens see the deep floors
  before anything on them changes, so every change after shows.
- **Each milestone** in the `add-zone` order (canon and style, layout,
  blockout checks, the walk, lighting, dressing), ending in a blind review
  against what it replaced, the visual playtest of the levels it touched,
  the full check and the ledger; pushed, with the production build made.

## Milestones

| # | Commit | What |
|---|---|---|
| F1 | `510096a` | **The research and this plan.** The real places (`../research/facility.md`, a research agent's note, every fact with its source); the code read for what decides a deep floor; the plan read against the canon by a fresh `lore-keeper`, its nine rules adopted (a premise of origin, coloured lamps, bottomless drops, decay as the wrong thing, and the Works' and the Well's things all taken out of it); Portal 2 recorded as a tone reference only. |
| F2 | (this commit) | **The gates see the deep floors.** `cargo xtask range` measures levels 9 and 13 too (96 levels for 72; the shift is the deep rot the canon asks for, dark share 0.13 to 0.25 and grime 0.35 to 0.58, every purpose still inside its band); the goldens keep every purpose's view of two deep floors (`DEEP` in `xtask/src/shots.rs`): 7's level 6, the Utility Halls' floor, and 4242's level 13, a deep floor of the halls (15 views, frames looked at). Their baselines are the game as it stands, so each remake shows against them. |
| F3 | | **The lamps of the era.** A fixture kind for every panel (`Panel`), each pooled spot in its lamp's colour (fluorescent's tubes; a white high-bay in a tall hall; a caged bulkhead in a tunnel, its hum from a fixed source), each failing its own way; the lamps modelled (the asset pipeline); `STYLE.md`'s light rule and bands measured again. |
| F4 | | **The surfaces of the facility**, from free photographs: painted and rusted steel (also `MILESTONES.md` 16), steel floor plate and bar grating, shotcrete with its bolts and mesh, corrugated liner plate; the two-tone wall; efflorescence and rust bloom. |
| F5 | | **The Utility Halls** in their canon's shape (shoulder-wide, bulging, low): shotcrete and liner plate, bulkheads, cable trays and pipe runs; torn note 32 and their rows. |
| F6 | | **The Habitable Zone** as a column hall (its puddles still with no source): two-tone walls, white high-bays, standing water; torn note 31 and its rows; whether its windows stay (`WINDOWS.md`). |
| F7 | | **Height in the rooms**: walls and ground in one floor (`build_solids_leaning`), drops with a floor or water you see, catwalks over them, railed to the code and stopping a body; the walkable and reachable gates over them (`level_sweep`). |
| F8 | | **The vault**, a new archetype: an arched gallery at a gallery's real size, its one wrong thing. |
| F9 | | **The pit hall**, a new archetype: a hall of open tanks and sumps with water in them, crossed by railed catwalks, its one wrong thing. |
| F10 | | **The blast door** (the asset pipeline): standing open between the facility's zones, or shut as a barricade; at a real door's size. |
| F11 | | **The dressing**: pipes in their colours with their own printing, cable trays, ducts, the decay; the `empty` purpose kept sparse. |
| F12 | | **The deep schedule and its names**: the vault and the pit hall in the deep rotation, their names on arrival (the kind of place, never a name, a number or a rule), `LORE.md`'s table. |
| F13 | | **The sound of the facility**: each lamp's hum from its fixture, room sounds for the new places (kept apart from the Building's), pumps and drips from fixed spots, footsteps on plate and grating. |
| F14 | | **The review**: a blind review of each archetype against what it replaced, the visual playtest on three seeds of the deep floors, people's playtest, a fresh code review, the bench, the ledger. |
