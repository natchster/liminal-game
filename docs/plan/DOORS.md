# The work on the doors

Every door in a wall remade by the asset pipeline
([`../design/ASSETS.md`](../design/ASSETS.md), the skill `make-asset`), as
almond water and the battery were ([`ASSETS.md`](ASSETS.md) A4-A6). What is
still open is queued in [`MILESTONES.md`](MILESTONES.md).

## The brief (2026-11-16)

The developer: "Remodel all doors (exit, building doors etc.) the same way
with new pipeline."

Read as: every door a player walks up to in a wall (not a cabinet's, a
locker's, a wardrobe's, a fuse box's, a vending machine's or a clock's)
goes through the pipeline: a dossier each (brief, references with their
licences, measurements and palette, placement rules with their whys, the
playtest's "why could this be here?"), modelled as the real thing at its
real size, its surfaces from its palette, what a hand does with it kept
to the canon, judged blind against what it replaced.

## The analysis (2026-11-16)

Seven kinds of door, made five ways, most of them by code: boxes in Rust,
flat lambert, the exit door's leaf glowing faintly.

| Door | Where | Made of | What a hand does |
|---|---|---|---|
| The exit door | every floor's far wall (`core_props::build_exit_door`); the Building's at its tower's foot (`world::building`), the Works' in the machine wall; the Loop's way in (no sign); the Borrowed House's front door | `Props__exitDoor_frame` (1.5 x 2.5 x 0.18 m) and `_leaf` (1.1 x 2.2 x 0.1, two raised-and-fielded panels, emissive 0x1a1408) in `core_placement.py`; a glowing strip down its latch side; an EXIT sign 1.30 x 0.48 m at 2.68 m | ENTER THE DOOR |
| The Stairwell's door | the exit door recoloured steel with a STAIRS plaque (`build_exit_door`, `to_stairs`); in the Stairwell (`modes::stairwell`) two boxes 0.95 x 2.05 x 0.08 | flat grey | OPEN THE STEEL DOOR; OPEN THE DOOR at the bottom |
| The hermetic door | every cell's front and the Well's galleries, about ninety a floor and the stacked storeys' (`world::building_dress`) | unit boxes 1.24 x 2.06 x 0.09 and a frame of three, a torus wheel and two spokes; flat 0x4d5650 | TURN THE WHEEL, OPEN THE DOOR, KNOCK, HOLD THE WHEEL |
| The Loop's doors | its first room's EXIT door and the side door you came in by (`modes::loop_rooms`) | vertex-coloured boxes, a panelled leaf 0.95 x 2.1 | OPEN THE EXIT DOOR |
| The lift doors | a working and a dead lift a floor (`core_props::builders`), a broken one on half the floors (`core_props::tail`) | `Props__elevator_door` 0.7 x 2.05 (an opening 1.4 m wide) and `brokenElevator_doorL/R`; emissive | CALL, PRESS THE BUTTON, PEER INTO THE SHAFT |
| The door to nowhere | `place_wall_doors` (one floor in 25) | `WallDoors__slab` 0.70 x 1.90 x 0.03, flush on the wall, no frame or casing, a knob 28 mm across; red, brown or black | nothing |
| The barricade's door | `core_props::barricade` (`home_pile`) | `Barricade__door`: a six-panel hollow-core leaf 0.81 x 2.03 x 0.035, primer | it stops a body and a look |

The census's reviewers saw the exit door's "bright white strip" down its
latch side (`../research/props-census/review2.md`) and "a door to nowhere
with no frame or handle" (`../research/assets-census.md`, `MILESTONES.md`
24); the Building's cell and exit doors are "flat paint among photographed
concrete" (`MILESTONES.md` 16).

## The plan

**Five real things, five dossiers** (`assets/src/props/<name>/`): one
dossier for each kind of real door, used wherever the game has one.

| Dossier | The real thing | Where the game has it |
|---|---|---|
| `exit_door` | a flush solid-core red-oak door to an office floor's stair, 914 x 2134 x 44 mm, in a pressed-steel frame; its lever, closer and kick plate; the lit exit sign over it | the exit door everywhere; the Loop's two doors |
| `steel_door` | a hollow-metal stair door, 914 x 2134 x 44 mm, grey enamel; lever and closer on the floor's side, push bar on the stair's; the STAIRS plaque | the Stairwell's way in, its top and bottom doors |
| `hermetic_door` | a Soviet shelter's hermetic door, steel, its wheel, bolts, hinges and seal | the cells; the Well's galleries |
| `lift_doors` | a passenger lift's centre-opening stainless landing doors, their frame, sill and call station | the working, the dead and the broken lift |
| `interior_door` | a hollow-core moulded six-panel door in its jamb and casing, with its knob | the door to nowhere; the barricade's door |

**Things the layout places.** A door is not put down by the placement
pass: the floor's layout makes it where its plan has one (the far wall,
a cell's front, a gallery). Its dossier says so (`by = "layout"`), and
each rule names the place in the layout it stands at (`at`) with its
purposes and its why; the game records every door it makes against the
rule that allows it, a door no rule allows is not made, and a test holds
every one to its rules.

## Milestones

| # | Commit | What |
|---|---|---|
| D1 | (this commit) | **The plan; the layout's rules; the exit door and the steel door.** The five briefs read by a fresh `lore-keeper` before anything was made, and corrected by it: the sign has no arrow (nothing points the way), no light shows under a door but the Works' ("the only place they do"), the steel door keeps its sign, lamps and call, no hatch shows on a gallery door until it answers, the lift's indicator shows no floor and the dead lift no notice, and the door to nowhere has its line in `LORE.md` (*Places*). **Things the layout makes** (`docs/design/ASSETS.md`): `by = "layout"`, each rule `at` its place in the layout; `place_by_layout` records each with its rule, and a thing no rule allows is not made; `what_the_layout_places_keeps_its_rules` holds them on 48 floors (one way down on every one). **The exit door** (`assets/src/props/exit_door/`, 20 references, 9 limited): a flush red-oak veneered door 908 x 2111 x 44 mm in a 3'0" x 7'0" opening, its pressed-steel frame (51 mm face, 16 mm stop) standing on the wall's face, its lever on an 87 mm rose at the strike's line (1024 mm), its closer (311 x 89 x 57 mm) and parallel arm, a stainless kick plate; the veneer painted from its palette (three leaves slip-matched, cathedral figure flaring into straight grain, worn where hands push), the frame's enamel chipped at the strike; nothing on it glows. Its sign is a real exit sign's box, 430 x 230 x 55 mm, its face (ISO 7010 E002 and EXIT on safety green, no arrow) 400 x 200 mm, 170 mm over the frame. The Works' door has the rooms' warm under it and spilled on the floor in front, its paper back on the wall. **The steel door** (`steel_door/`, 18 references, 9 limited): hollow metal, 18-gauge, grey enamel chipped round the lock and rusting at its foot, a cart's dent; its lever, closer and kick plate and the STAIRS plaque (2" x 8" x 1/16"), under the same sign. The old parts (`Props__exitDoor_*`, `exitSign_housing`) retired |
| D2 | | The Stairwell's top and bottom doors (the steel door from the stair's side) and the Loop's doors |
| D3 | | The hermetic door: the cells and the Well's galleries |
| D4 | | The lift doors: the working, the dead and the broken lift |
| D5 | | The interior door: the door to nowhere, in its frame; the barricade's door |
| D6 | | The playtest's question asked of each, the rules changed by it; a blind review against the old; the visual playtest; a fresh code review |
