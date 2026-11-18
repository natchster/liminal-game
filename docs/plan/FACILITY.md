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
   level down, "You fell"); nothing kills by a fall. A drop may keep its
   bottom out of sight (the developer's answer, below): a railed shaft or
   pit goes dark at a room's scale, but it is never the void (no fog
   rises from it, nothing stands across it) nor the broken lift's black
   (nothing looks back out of it, nothing breathes there). No ladder or
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

## The developer's answers (2026-11-17)

Asked after F3, before going on:

- **Drops may be dark.** Of the canon's three limits on the brief (no
  coloured pools, a bottom you see, the rooms' scale), only the second is
  lifted: a railed shaft or pit may keep its bottom out of sight (rule 4
  as it stands now). Nothing falls into one; the colours and the scale
  stay the canon's.
- **The steel's wear is dressing.** F4's second attempt is not made as a
  texture: the peeling and rust at a rail's and a pipe's scale come with
  F11, laid where the forms are (joints, feet, tops) and seen from close
  cameras on real pipes.
- **A hall is lit as a hall.** A high-bay hangs in a tall hall whatever
  it is seen from; the view from an office goes to a person (the
  playtest).
- **No windows in the column hall.** The Habitable Zone's halls lose
  their window rows (F6); windows keep to offices and living quarters.
- **Height first as railed drops** (asked before F7, 2026-11-18): F7 cuts
  drops into the floors, railed all round and seen into, never walked down
  into, the floor staying one storey; floors you walk down to come after
  F14 (F15).
- **The vault's wrong thing: a draught with no opening** (asked before
  F8, 2026-11-18): the cords of its lamps and its cobwebs all lean the
  same way, toward one end of the gallery, and nothing there is open.
  Chosen over rails that run under its end walls and an arch that leans.
- **The pit hall's: wet above the water**: every tank's walls are wet a
  hand or more above its water, as if the level had just fallen, and
  nothing drains. Chosen over one level in every tank and rings spreading
  with nothing falling.

## Milestones

| # | Commit | What |
|---|---|---|
| F1 | `510096a` | **The research and this plan.** The real places (`../research/facility.md`, a research agent's note, every fact with its source); the code read for what decides a deep floor; the plan read against the canon by a fresh `lore-keeper`, its nine rules adopted (a premise of origin, coloured lamps, bottomless drops, decay as the wrong thing, and the Works' and the Well's things all taken out of it); Portal 2 recorded as a tone reference only. |
| F2 | `ed29ff8` | **The gates see the deep floors.** `cargo xtask range` measures levels 9 and 13 too (96 levels for 72; the shift is the deep rot the canon asks for, dark share 0.13 to 0.25 and grime 0.35 to 0.58, every purpose still inside its band); the goldens keep every purpose's view of two deep floors (`DEEP` in `xtask/src/shots.rs`): 7's level 6, the Utility Halls' floor, and 4242's level 13, a deep floor of the halls (15 views, frames looked at). Their baselines are the game as it stands, so each remake shows against them. |
| F3 | `217e7e2` | **The lamps of the era.** A panel has a kind (`Lamp`; `world::lamps`): the rooms' fluorescent tubes; in the Habitable Zone's tall bare halls an enclosed 400 W mercury high-bay (a spun-aluminium reflector 560 mm across, its prismatic glass bowl lit, a cool white), on a conduit stem; on the Utility Halls' crown a vapour-tight jar lamp (a cast canopy, a 100 mm jar, its guard: a filament's warm white), never within 1.6 m of a door the layout made (a cell's lamp). Two dossiers (`assets/src/props/high_bay/`, `bulkhead/`; 20 references, 12 free; the Day-Brite reflector of 1970 572 mm, the jar fixture's guard 106 mm), their parts (`lamps.py`), each held to its rules where the layout hangs it ("lamp"). Every kind fails on the tubes' own draws, so no floor's frame stream moved: a tube stutters, a high-bay's arc drops out and restrikes, warming up from dim, a filament dims and comes back; each pooled spot lit its lamp's colour. The playtest's question: the jar lamp obvious in its corridors; the high-bay obvious in tall concrete halls and "a warehouse lamp under a low tiled lobby ceiling" where moss had papered a hall, so it hangs only under a tall bare slab. Blind (two rounds): the jar lamp and the close high-bay preferred every time, the tubes where a hall is seen from an office (twice). The visual playtest of a hall and a tunnel: the jar's guard fought its glass (fixed). `LORE.md` and `STYLE.md` (the lamps in their places' rows, *Each lamp is its kind*). The hum stays every lamp's ballast until F13. |
| F4 | | **Folded into F11** (the developer's answer): painted and rusted steel worn at a rail's and a pipe's scale (also `MILESTONES.md` 16). Each other surface comes with the milestone that first uses it (shotcrete, its bolts and mesh, and liner plate with F5; the two-tone wall with F6; floor plate and bar grating with F7; efflorescence and rust bloom with F11). Tried once as a texture (2026-11-17): a 1 m photographed coat (Poly Haven `green_metal_rust`) on the Works' metalwork changed its frames by 0.0-0.2%, near-identical, and was not kept (`MILESTONES.md` 16 says what it taught). |
| F5 | `a2d5e30` | **The Utility Halls as a plant's service tunnels.** Sprayed concrete on their walls and crown (a photograph: ambientCG's Concrete037, CC0, quilted into 2 m and graded to the rooms' concrete, its grain a photographed detail; `world::lining::wall_mat_of`: the generator's table keeps the old game's concrete, the world draws it sprayed); a haunch rounding every corner into the crown, meeting wall and crown at a 15° crease (tangent, it flickered), springing above every door's head (so a low corridor's is small) and stopping short of a door or a window, a row of bolts along it; on the crown of each straight corridor three tiles long or more a run (`service_run`, a dossier: 20 references, 13 free; a ladder tray of cables, a pair of 4 in and 2 in pipes on trapeze hangers 2.4 m apart with their clamps and dull orange grooved couplings, or both; the pipes turning up into the crown on long-radius elbows and the cables with them), out of the jar lamps' way, a person's headroom under it, ending at a turn, a hall, a lintel, a barricade or a thing hung from the crown; none where moss has papered a corridor, no run in an empty one (`world::lining`; the parts in `lining.py`, each worn as a prop is). Torn note 32 and the `LORE.md` and `STYLE.md` rows rewritten, read by a fresh `lore-keeper` (taken: "rock bolts" said what is behind the lining, the pipes' routing made a second wrong thing, the note too long). Blind (two rounds, both orders): the remade halls preferred at all 7 cameras each time. The placement playtest: the runs "obvious" in 4 of 4 frames. The visual playtest (`2026-11-17-utility-halls.md`): the haunch's flicker, the flat paint of the runs, two coplanar faces, and (the `visual-inspector`'s) the haunch's photograph stretched round its arc and its crossing at inside corners, fixed (the haunches are meshes of their own, mapped in metres and mitred); the tour as it was before but for the far runs' thin parts shimmering. Liner plate waits for an arch (F8). |
| F6 | `e2949cc` | **The Habitable Zone as a column hall.** Its walls and columns painted two-tone (a photograph: Poly Haven's painted_plaster_wall, CC0, quilted into 4 m by 8 m and graded twice, a dark band 0.8 m high under a brushed stripe and a pale grey above; the band below her pencil's 0.9-1.1 m, the `lore-keeper`'s catch: at the hip it hid her marks), its slab bare, cast against steel forms (the painted slab's blotches "read more like sky than slab", the blind review), each free-standing column flared at its head into a drop panel (`world::halls`; laid with its own UVs, by height, as the haunches are; none where anything else stands within its panel's reach); no window rows in its halls (`NO_WINDOWS`, the developer's answer: the one behind the glass stands at another window). Torn note 31 and the `LORE.md`, `STYLE.md` and `WINDOWS.md` rows rewritten, read by a fresh `lore-keeper` (taken: the windows' canon still listing halls, an origin in "under the ground", the band's height, the note's echo of note 32's "into the ceiling"). Sheets of standing water were tried: twice the blind review read them as "a flat, opaque decal… not water" (no reflection to give them light), so not kept; its puddles stay the wet floors'. Blind (two rounds, both orders): the hall preferred 7 of 7, then 6 of 7 (the one, a close view of a sheet of water). The visual playtest (`2026-11-17-habitable-zone.md`): the tour of 4242's level 5 as it was before. |
| F7 | `882ffb7` | **Height in the rooms**, as railed drops seen into (the developer's answer; walkable lower floors are F15): pits cut into some bare halls from level 5, a floor or standing water 3.2 m below or dark (rule 4 as eased), catwalks over the lit ones that creatures cross too, railed to the code and stopping a body, the walls built as before the cut; nothing placed on one or hung at its edge; the walkable, reachable and placement gates over them (`level_sweep`). |
| F8 | | **The vault**, a new archetype: an arched gallery at a gallery's real size (corrugated liner plate where it is lined: F5 left it for an arch), its one wrong thing. |
| F9 | | **The pit hall**, a new archetype: a hall of open tanks and sumps with water in them, crossed by railed catwalks, its one wrong thing. |
| F10 | | **The blast door** (the asset pipeline): standing open between the facility's zones, or shut as a barricade; at a real door's size. |
| F11 | | **The dressing**: pipes in their colours with their own printing, cable trays, ducts, the decay; the steel's wear at a rail's and a pipe's scale (the drops' railing too, read as pristine flat paint in F7's blind review; F4, `MILESTONES.md` 16: peeling and rust blooms of 0.3-1 m at joints and feet, dust on tops; seen from close cameras); the `empty` purpose kept sparse. |
| F12 | | **The deep schedule and its names**: the vault and the pit hall in the deep rotation, their names on arrival (the kind of place, never a name, a number or a rule), `LORE.md`'s table. |
| F13 | | **The sound of the facility**: each lamp's hum from its fixture, room sounds for the new places (kept apart from the Building's), pumps and drips from fixed spots, footsteps on plate and grating. |
| F14 | | **The review**: a blind review of each archetype against what it replaced, the visual playtest on three seeds of the deep floors, people's playtest, a fresh code review, the bench, the ledger. |
| F15 | | **Walkable lower floors** (the developer's answer before F7): sunken floors reached by a stair, so a rooms floor has height of its own (`Game::ground` beside the rooms' walls: the solids, the creatures' paths, placement, the light map and the sweep over two heights). |
