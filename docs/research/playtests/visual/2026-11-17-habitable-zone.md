# Visual playtest: the Habitable Zone remade, seed 4242 level 5 (2026-11-17)

## The run

`cargo xtask playtest visual --seeds 4242 --levels 5 --out target/xtask/playtest/visual/NAME --no-build`
on `a2d5e30` with the facility's F6 (`docs/plan/FACILITY.md`), the whole floor (its
halls, a quarter of its zones, and every place round them), against the same tour of
the floor before F6 (`a2d5e30` built as it was):

| Tour | Views (detected of all; eyes) | Background | Gaps | Flicker | Clip | Body | Plain (flagged) | Coplanar (flagged / alike) |
|---|---|---|---|---|---|---|---|---|
| before F6 | 7740 of 7740; 180 | 156 | 118 | 12 | 10 | 2 | 14 | 52 / 46 |
| F6, first (the band at 1.25 m, standing water) | 7740 of 7740; 180 | 156 | 118 | 13 | 10 | 2 | 14 | 52 / 46 |
| F6, final (the band at 0.8 m, a bare slab, no standing water) | 7740 of 7740; 180 | 156 | 118 | 13 | 10 | 2 | 14 | 52 / 46 |

## What it found

| Id | Kind | Thing | View | Severity | Confirmed by | Fixed |
|---|---|---|---|---|---|---|
| F1 | flicker | a strip of a neighbouring zone's yellow wall fighting the hall's wall where they meet, as before F6 (1045 px); against the pale paint it shows more (10390 px), and in one more view | `L5_0523_up`, `L5_0524_left` | 2 | the detector, me (`--shots-at` before and after) | not F6's: the rooms' own fights where two zones' walls meet, `MILESTONES.md` 23 |

Nothing of F6's own: the capitals, the paint and the slab add no gap, flicker, clip,
plain thing or coplanar pair (the one coplanar pair "new" is a pair of the floor's own
renamed by its grime's tint).

## Not faults

The gaps (118), clips (10) and bodies (2) are the floor's before F6, view for view.

## For a person

- The paint's band: 0.8 m, under her pencil's marks (the `lore-keeper`'s catch: at
  the hip, the research's height, it hid them); whether it reads as a dado or as a
  skirting.
- The halls are a little lighter painted than bare (luminance +0.02-0.03 on their
  frames' anchors, inside the `STYLE.md` bands).

## What will be fixed and what will not

F1 is the rooms' own and stays with `MILESTONES.md` 23. The blind review's own
findings were taken: the slab's cloud of blotches (now the formwork photograph), a
capital running into a pier's top (now none where anything else stands within its
panel's reach), and sheets of standing water reading as decals (dropped).

## The tour again

The third row: as the floor was before F6, but for F1.

## Not checked

The halls of other seeds and deep floors (4242's level 13 and seed 7's level 6 seen in
their goldens), and a column's capital from right under it.
