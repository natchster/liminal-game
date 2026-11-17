# Visual playtest: the Utility Halls remade, seed 7 level 6 (2026-11-17)

## The run

`cargo xtask playtest visual --seeds 7 --levels 6 --out target/xtask/playtest/visual/NAME --no-build`
on `bb08df3` with the facility's F5 (`docs/plan/FACILITY.md`), the whole floor (its
Utility Halls' corridors and every place round them), against the same tour of the
floor before F5 (`bb08df3` built as it was):

| Tour | Views (detected of all; eyes) | Background | Gaps | Flicker | Clip | Body | Plain (flagged) | Coplanar (flagged / alike) |
|---|---|---|---|---|---|---|---|---|
| before F5 | 10908 of 10908; 180 | 508 | 368 | 0 | 17 | 2 | 30 | 52 / 103 |
| F5, first | 10908 of 10908; 180 | 508 | 368 | 53 | 17 | 2 | 2466 | 60 / 846 |
| F5, fixed | 10908 of 10908; 180 | 508 | 368 | 2 | 17 | 2 | 30 | 52 / 682 |
| F5, final (the runs at the references' sizes; the haunch mapped and mitred) | 10908 of 10908; 180 | 508 | 368 | 8 | 17 | 2 | 30 | 52 / 568 |

## What it found

| Id | Kind | Thing | View | Severity | Confirmed by | Fixed |
|---|---|---|---|---|---|---|
| F1 | flicker | the haunch's arc, tangent to its wall and its crown, set 3 mm into each: a grazing band where the two faces crossed fought at a distance | 53 views along the corridors' walls (`L6_1599_right`, `L6_1275_right`, …) | 2 | the detector, me | the arc meets wall and crown at a crease of 15° (`world::lining::CREASE`), set 2 mm in: 53 views to 0 |
| F2 | plain | the runs and the bolts in flat paint: the galvanised strut and tray, the cables, the two pipes, the couplings, the bolts' plates (`#55585a`, `#1e1f1f`, `#5d5a52`, `#6b665b`, `#3e3b36`, `#5d5853`) | 2436 things | 2 | the detector, the blind review ("white plastic bars" before) | each takes the props' settled look (`prop_look::settle_material`: albedo, wear, a finish): 2466 to 30, the floor's before F5 (18 of them the jar lamps' canopies, kept one paint: `2026-11-17-lamps.md`, F2) |
| F3 | coplanar | an open end of a haunch, set into its wall, on one plane with the wall's end | 7 pairs (`box 4.62x2.44x2.00 + buffer 0.24x0.24x4.62`, …) | 1 | the detector | an open end stops 4 mm short of its wall's |
| F4 | coplanar | a pipe's coupling where a hanger's clamp is | 1 pair (`#3e3b36 + #55585a`) | 1 | the detector | a coupling keeps clear of the clamps |
| F5 | flicker | the far runs' thin parts shimmering a pixel wide down a long corridor: the hangers' rods (10 mm), and once the tray's rails were a 3 in tray's (92 mm) its rungs and rails at 10 m and more | 2 views, then 8 (`L6_1722_at`, `L6_1838_at`, `L6_1588_right`, …; 12-26 px) | 1 | the detector, me; the `visual-inspector` ("thin far-off edges, not two surfaces fighting") | kept: thin real parts at a distance alias as any wire does |
| F6 | out of place | the haunch's photograph stretched round its arc into a smooth, streaked, lighter band: `Patch::ArchUV` maps a face by its normal, and up the curve the floor's mapping stretched it up to twice | `L6_1437_up`, `L6_1723_up`, `L6_0769c_up`, … | 1 | the `visual-inspector` | the haunches laid as meshes of their own (one a surface and a chunk), their photograph mapped in metres along them and round the arc, carrying on from the wall's (`world::lining::haunch_face`) |
| F7 | clip | two walls' haunches running through each other at an inside corner (an X, a notch at its apex) | `L6_1800c_up`, `L6_1145c_up` | 1 | the `visual-inspector` | an end that meets a wall is mitred where the other wall's haunch meets it: the two meet in one line, as a groin does |

The blind review (two rounds) found one more, not a detector's: a run's pipes went on into
a plywood barricade's boards (a barricade stands from the floor, so a run took it for the
corridor's own wall, and a barricade's slab is not a solid at all); a run now ends short
of anything in the corridor's middle (a lintel, a jamb, a thing hung from the crown) and
of a barricade's tile (`Game::barricades`).

## Not faults

The `visual-inspector` looked at every flagged picture and every sheet of the second tour.
The gaps (368: the ceiling missing over wall sections' tops, holes in far walls), the clips
(17: the eye in a payphone, lockers, a filing cabinet and the bottle on it) and the
bodies (2) are the floor's before F5, view for view, pixel for pixel; they and what else
it found of the floor's own (pillars dark above the tiles, a seam's office wall in a
tunnel, two-tone pillars, her camp's flat props, the flashlight's pool at 35 cm, a panel's
bright cracks) are `MILESTONES.md` 23's; so are the coplanar pairs left (52,
the floor drains, the suitcases, a vending machine's skins: the same pairs, some under
another mesh's number, a wall now drawn in two halves where shotcrete meets another
surface).

## For a person

- The jar lamp's cast canopy stays one grey paint (`2026-11-17-lamps.md`, F2), among
  runs now worn: whether it reads as plastic beside them.

## The tour again

The third and fourth rows of the table, the same command after the fixes: F1-F4 gone,
F6 and F7 gone (looked at in their views), F5 kept; nothing new.

## Not checked

The other seeds' Utility Halls (4242's level 12 and 13 were looked at in their zone
views, not toured), and the runs from below at a run's mouth from close (the risers are
seen from the corridor, not from under them).
