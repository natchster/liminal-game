# Visual playtest: the writing on the rooms' walls, levels 5, 7 and 9 (2026-11-15)

## The run

`cargo xtask playtest visual --seeds 4242 --levels 5,7,9 --budget 60 --out target/xtask/playtest/writing-vp`
on `a000cea` with issue #11's writing (G2-G5, before the hand's spacing was fixed: that fix changes a patch's
size about its middle, not where patches go) and the planks' fix (481 s). The tour was for what the change
touched: writing baked into the walls of the rooms (levels 5-12) as a patch 1-2.6 mm off a drawn face.

| Seed | Level | Views (detected of all; eyes) | Background | Gaps | Flicker | Clip | Body | Plain (flagged) | Coplanar (flagged / alike) |
|---|---|---|---|---|---|---|---|---|---|
| 4242 | 5 | 7,674 of 7,674; 60 | 158 | 119 | 11 | 17 | 2 | 16 | 50 / 43 |
| 4242 | 7 | 19,650 of 19,650; 60 | 546 | 348 | 21 | 20 | 2 | 20 | 50 / 115 |
| 4242 | 9 | 10,941 of 10,941; 60 | 698 | 524 | 20 | 39 | 3 | 11 | 49 / 79 |

## The findings

None in the writing. Of the 855 rows in `findings.tsv`, three name it (`core_graffiti`), all coplanar (below);
no flicker, gap, clip or plain finding is at a patch, in any of the 38,265 views.

## Not faults

- **The writing on its wall, coplanar** (`core_graffiti + level`, three flagged: `box 1.29x0.85x0.00` on
  `box 4.00x3.04x4.42`, `L7_0333_at:84.0000,1.6200,-70.7800,-3.14159,0.00000`; `box 0.65x0.30x0.00`,
  `L7_0115_crouch:52.0000,1.0500,-85.2200,0.00000,-0.17453`; `box 0.57x0.42x0.00` on `box 4.00x2.80x4.40`,
  `L9_0232_crouch:-40.0000,1.0500,-86.7600,-3.14159,-0.17453`): the patch's back is 1 mm off the wall's face
  and faces into it, its front 2.6 mm out (`core_props::graffiti::bake`). The three views' flicker is 0, as
  is every view's at a patch. As the Building's walk slabs on their walls: a pair that does not fight.

## Found, not from this change

The floors' own counts are the rooms' as they were: the writing adds a thin patch on a face and cannot open
a gap, and the planks' fix lowers a board. The worst gap view, `L5_0198_left:-5.6400,1.6200,-38.8000,2.79253,0.00000`
(8,089 px: the ceiling missing over a partition's top, the void showing), is the same in `a000cea`'s build
(rendered with it: the background there too). No visual playtest of the rooms had been run before this one;
their gaps (wall edges, 119-524 views a floor), flicker (11-21), clip and body are queued as a playtest of
their own (`MILESTONES.md` 23).

## For a person

- A scratch is meant to be seen only in light across it (`graffiti.md`, rule 2 and 12): head-on in the lamp it
  all but vanishes. Whether that is too faint to be found is a player's question.

## What will be fixed and what will not

Nothing in the writing. The rooms' findings: `MILESTONES.md` 23.

## The tour again

Not run: no finding in the writing to fix. The hand's later spacing fix (lines one slope apart, letters their
own width) was looked at on the hand's sheet (`a_sheet_of_the_writing`) and in the game from the blind
review's cameras; it resized the patches about their middles (the reference levels, re-recorded) and let
one more fit its wall (seed 4242, level 9, a pillar at 84.0, 43.4).

## Not checked

- No `visual-inspector` looked at the sheets: the change is the writing, which the blind review (art-critic,
  four cameras, both orders, the new preferred 4 of 4) and the prop census looked at, and of which the
  detectors found nothing. The rooms' playtest (23) should have one.
- Legibility and taste: a person reading the walls.
