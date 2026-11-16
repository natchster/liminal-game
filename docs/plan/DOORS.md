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
| D1 | `8da9c33` | **The plan; the layout's rules; the exit door and the steel door.** The five briefs read by a fresh `lore-keeper` before anything was made, and corrected by it: the sign has no arrow (nothing points the way), no light shows under a door but the Works' ("the only place they do"), the steel door keeps its sign, lamps and call, no hatch shows on a gallery door until it answers, the lift's indicator shows no floor and the dead lift no notice, and the door to nowhere has its line in `LORE.md` (*Places*). **Things the layout makes** (`docs/design/ASSETS.md`): `by = "layout"`, each rule `at` its place in the layout; `place_by_layout` records each with its rule, and a thing no rule allows is not made; `what_the_layout_places_keeps_its_rules` holds them on 48 floors (one way down on every one). **The exit door** (`assets/src/props/exit_door/`, 20 references, 9 limited): a flush red-oak veneered door 908 x 2111 x 44 mm in a 3'0" x 7'0" opening, its pressed-steel frame (51 mm face, 16 mm stop) standing on the wall's face, its lever on an 87 mm rose at the strike's line (1024 mm), its closer (311 x 89 x 57 mm) and parallel arm, a stainless kick plate; the veneer painted from its palette (three leaves slip-matched, cathedral figure flaring into straight grain, worn where hands push), the frame's enamel chipped at the strike; nothing on it glows. Its sign is a real exit sign's box, 430 x 230 x 55 mm, its face (ISO 7010 E002 and EXIT on safety green, no arrow) 400 x 200 mm, 170 mm over the frame. The Works' door has the rooms' warm under it and spilled on the floor in front, its paper back on the wall. **The steel door** (`steel_door/`, 18 references, 9 limited): hollow metal, 18-gauge, grey enamel chipped round the lock and rusting at its foot, a cart's dent; its lever, closer and kick plate and the STAIRS plaque (2" x 8" x 1/16"), under the same sign. The old parts (`Props__exitDoor_*`, `exitSign_housing`) retired |
| D2 | `96cdce1` | **The doors of the places a way down leads to.** The Stairwell's two doors are the steel door's (`modes::stairwell`): the top one behind you from its side that pulls (its hinges, nothing to take hold of; its enamel painted the other way round, the lock's side on its left), the bottom one from its side that pushes (its push bar, closer and kick plate; the handle box gone). Both stand 1.375 m from their landing's middle (`DOOR_OFF`): the landings are half round, and the frame's corners clear the curve only that near. The Loop's two side doors are the exit door's parts, a mesh a material (`exit_door_instanced_parts`), its first room's under the exit sign's housing with the Loop's own plain EXIT lit in it, 6 cm over the frame under its 2.7 m ceiling. Both places record their doors against their rules on the way in ("stairwell", "loop"); the layout test goes through the way down where it leads there. The Stairwell's recording is the old game's: `rerecord_doors` writes its tree and each enter's bottom from the build, and only the doors moved (the rest the same within the test's tolerance). The Loop's keeps its recording: the doors and the sign's housing are parts folded into the recorded meshes, as its pickups are (`tests/loop_rooms.rs` `PARTS`). `--screen loop-door`, `loop-exit`, `stair-top`, `stair-bottom` |
| D3 | `6dca133` | **The hermetic door** (`assets/src/props/hermetic_door/`, 15 references, 4 limited: the ДУ series' makers' sheets): a Soviet shelter door the size of a ДУ-IV-2, 1240 x 2060 x 90 mm, its parts in its hinge's frame (`HermeticDoor__*`): the leaf (its outer face plain plate, for the hatch, marks and tape), its inner face (the rib grid of channels, the gearbox, the screw rods to the wedges at head and foot), the seal, two heavy hinges on its axis, the 420 mm wheel (four spokes; a lighter one for the storeys across the void), a cell's frame lining its opening (its face band, the stop at latch and head, the threshold), a gallery's face frame (its band 75 mm beyond the leaf); its enamel painted from its palette (grey-green over red-brown primer, coats, worn edges, rust from the hinges and at the foot). Its wheel is worn bare steel, not the red of many real ones: red is the cleaners' mark, and a red wheel on every door would say what only a marked one says. A cell's door swings on its hinge as it did (`door_leaf`), its two wheels on one spindle (the inner one on the gearbox). A gallery door stands on the block's face, its leaf 9.2 cm out with nothing behind it (it was 5 mm out in a frame 22 cm deep); the red cross, the tape and the hatch lie on the leaf's face (they hung 9.5-10.5 cm in front of it). Every cell's door and every knockable gallery door is held to its rule ("cell", "gallery"). The Well's spawn view 511k triangles (471k before; 948k with the near wheel on every door) |
| D4 | (this commit) | **The lift doors** (`assets/src/props/lift_doors/`, 15 references, 7 limited: ISO 4190, KONE, Mitsubishi and TK Elevator planning guides): a landing entrance's stainless frame round a 1100 x 2100 mm opening (a 1000-1600 kg car's; it was 1.44 m), 120 mm at the jambs and 150 at the head, standing 16 cm out from the wall; two hairline stainless panels 572 x 2094 x 32 mm (painted from its palette: fine lines, prints at hand height, scuffs low); the aluminium sill and its grooves; the hall call station (91 x 171 mm, its two 25 mm buttons 1065 mm up, lit where the lift comes). The indicator over it dark, its floor number gone (a number about the place: where a car goes is never said), its draw kept; the dead lift's handwritten OUT OF SERVICE gone (pressing its button is how it is found dead), its draw kept; the dead lift's buttons found by name now. The broken lift: its panels jammed apart at the gap the shaft's look takes (x -0.075 to 0.47, the black at z 0.10: the cutscene unchanged, its contract holds on every stage), the left one forced (`LiftDoors__panelForced`: kicked in low, its foot pushed toward the shaft, hanging from its top track; its five dent blobs' draws kept), the right one mostly in its pocket behind the jamb; the wires hang in the black before it now (they hung inside the black box); soot round the edges drawn round and soft (a blotch's two centres drew shards across the panels). Eight old parts retired |
| D5 | | The interior door: the door to nowhere, in its frame; the barricade's door |
| D6 | | The playtest's question asked of each, the rules changed by it; a blind review against the old; the visual playtest; a fresh code review |
