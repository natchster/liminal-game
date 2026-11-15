# The assets census (2026-11-15)

Which props earn their place, and which 30% go, for GitHub issue #13
("Current assets are of uneven quality and too varied … remove 30% of less
interesting assets"; the work: [`../plan/ASSETS.md`](../plan/ASSETS.md), A1).
The October census ([`props-census.md`](props-census.md)) asked whether each
kind looked real; every `cartoon` kind has been remodelled since. This one
asks what a kind adds to the place.

## How

The same 272 floors as October (`--prop-census 1`, one shot of each kind a
floor: ten seeds over levels 4-14 and eighteen more over 6-14 for the Maze's
themes), from the build of `29a995f`. Each kind's shot with the most detail
(the largest PNG, as before) went into contact sheets of twelve (55 sheets,
the kinds shuffled with a fixed seed so no sheet is one theme:
[`assets-census/sheets.py`](assets-census/sheets.py), the cells in
[`assets-census/sheets.json`](assets-census/sheets.json)). Five fresh
reviewers, each given the `art-critic` brief and this question
([`assets-census/questions.md`](assets-census/questions.md)), eleven sheets
each, answered for every cell, the evidence first:

- `belongs`: `yes` (someone who worked, lived or was trapped here could
  have left it, or the place kept it), `odd` (a player would ask why it is
  here), `no` (it does not belong at all);
- `interest`: `high` (it says something: a use, a person, a story, a
  scare), `some` (dressing that reads), `none` (filler nobody notices);
- `made`: `real`, `plain`, `cartoon`, or `unseen` (it cannot be made out
  where a player stands or crouches).

How often a kind is placed is counted two ways, the larger kept: the floors
the census framed it on, and the floors its placer drew more than three
times from the level's stream (each level dump's `marks`): the census
cannot see what the instancing merged (the cigarette butts are on 122
floors, the wet floor signs on 65, and neither was ever framed).

The answers ([`assets-census/answers.json`](assets-census/answers.json)),
each reviewer's notes ([`review1.md`](assets-census/review1.md) …
[`review5.md`](assets-census/review5.md)), and every kind's grades, score,
floors and decision ([`assets-census/grades.tsv`](assets-census/grades.tsv))
are kept.

## What they found

**695 kinds**: 657 framed, 38 themed placers never framed. Of the 657,
the reviewers found 523 belong, 127 odd and 7 not at all; 63 of high
interest, 267 some, 327 none; 14 real, 482 plain, 14 cartoon and 147 that
could not be made out where a player would see them.

- **Thirty placers never place anything.** On none of the 272 floors did
  they draw more than a few numbers: the occult's showpieces (censers, a
  memorial shrine, a ouija board, chalked pentagrams, reliquaries, skull
  plaques, crystal balls, stained glass, tarot cards, crucifixes), most of
  the Victorian study (an aquarium, bellows, a chess stand, a floor globe,
  a lantern, heraldry, a humidor, an orrery, a pocket watch, a secretary
  desk, silhouettes, a music box, a wagon wheel, empty painting outlines),
  the garden's acorns, botanicals, ferns, rakes and pine cones, and a
  Rubik's cube.
- **A third of the kinds are specks.** Earrings, coins, wristwatches,
  eyeliner, floss, cotton pads, dominoes, keys, dice, darts, tiaras,
  matchbooks, spinning tops, pills, shards: at a crouch a few pixels on
  the carpet. Nobody finds them, and they cost a placer each.
- **The variety is the Maze's themes, almost never seen.** Of the nature
  theme's 45 kinds, 44 are on fewer than ten of the 272 floors; all the
  occult's 36 and the Victorian's 76, and 67 of the toys' 76. What is rare is also what
  the reviewers most often called odd: the outside indoors (pebbles,
  twigs, dandelions, a bird's nest, soil bags, a bird feeder, snail
  shells), Halloween's props (bats pinned flat to the ceiling, a sheet
  ghost, a tombstone, a witch's hat, a cauldron, potions), a country
  house's (servants' bells, a log basket, a door stop far from any door),
  and kitchen things lying on carpet (a loaf, a rolling pin, a teapot, a
  colander hung on a corridor wall).
- **Duplicates.** Forty-six groups of the same thing under two or more names:
  three kinds of CRT television, four of mirror, four of coat hook, three
  of dead plant in a pot, three of newspaper, two of every small thing
  (typewriters, globes, marbles, yo-yos, nail polish, calendars, clocks,
  thermostats, puddles, wastebaskets, suitcases, ashtrays, mugs).
- **What says the most** is what tells of a person or of the place
  itself: the barricades, the bench's candles, the writing, the claw marks,
  a handprint with a date, the payphone, the wheelchair, the robot vacuum
  still cleaning, the spilled backpack, a packed suitcase, a single glove
  like a hand, a teddy bear sitting alone, a typewriter with a sheet in it,
  a clock reading 12:66, a hook where a photograph hung.

They also found things broken or wrong in what stays (the reviews list
each with its cell): a bookshelf floating half way up a wall, an emergency
light on the floor, a coffee spill with an octagon's edge, the wall stains
as overlapping outlined ellipses (on every floor: the most artificial thing
on the sheets), a door to nowhere with no frame or handle, gauges on bare
wallpaper, a pipe leaking from nothing, a crib's end and a pinwheel
standing with nothing holding them, a skateboard's wheels on its top, a
tap with no basin, a chalk arrow's lettering that reads as the game's own
text; and frames where the census camera failed (the wrong thing in view,
a pillar or a creature in the way). They are queued
([`../plan/MILESTONES.md`](../plan/MILESTONES.md) 24).

## The selection

The rule, written before the result was read
([`assets-census/select.py`](assets-census/select.py)):

1. **Never removed:** the game's own props (`core_*`: their rules, verbs
   and canon), the placers of the story, the secrets, the Showroom and the
   Borrowed House, the only themed verb (`place_gameplay_r1_trigger`) and
   the lockers the hand's verbs open (`place_metal_lockers`, read by
   `core_props::usable`).
2. **First, what never places** (30).
3. **Then the lowest score**: `belongs`, `interest` and `made` each 2, 1, 0
   (`yes`/`odd`/`no`, `high`/`some`/`none`, `real`/`plain`/`cartoon`;
   `unseen` 0, but where the census camera failed it counts as `plain`);
   in each group of duplicates the weaker loses a point; ties go by
   `belongs`, then by how rare (the rarer goes first: it is the variety the
   brief means), until 30% of the 695 kinds: **208**.

The last kinds taken scored 2 (for example `yes`/`none`/`unseen`), as did 34
kept, which are commoner. By theme:

| Theme | Removed | Of |
|---|---|---|
| nature | 29 | 45 |
| occult | 21 | 36 |
| bathroom | 29 | 58 |
| toys | 33 | 76 |
| victorian | 33 | 76 |
| domestic | 26 | 122 |
| office | 24 | 100 |
| party | 6 | 27 |
| industrial | 6 | 38 |
| core (themed placers of the core theme) | 1 | 113 |

The 208, each with its grades, are the rows of
[`assets-census/grades.tsv`](assets-census/grades.tsv) marked `remove`.
