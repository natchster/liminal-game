# r04: the torn note, the mannequin, the wall clock

Batch `r04.py` (Props P1, `docs/design/PROPS.md`). Each prop is modelled at its
real size from a real one; the game places the parts as modelled.

## The torn note (`core_lore_notes`, `build_lore_note_mesh` in `core_props/tail.rs`)

**The real one.** A typed sheet of 20 lb typewriter bond, US letter:
8.5 x 11 in (215.9 x 279.4 mm), about 0.1 mm thick (20 lb bond, 75 g/m²,
4 mil: https://www.bhphotovideo.com/c/product/1619743-REG/canon_3871v281_20_lb_bond_paper.html ;
the letter size: https://myworksupply.com/products/alliance-8-5-x-11-bond-paper-20-lbs-92-brightness-500-ream-30758 ).
Typed in pica Courier: 10 characters to the inch, 6 lines to the inch,
1-inch margins, so 65 characters to a line (`docs/design/STYLE.md`: the torn notes
are typed paper, Courier New).

**What was made.** The sheet folded in three for an envelope and once more
in half for a pocket, then torn: its lower third is gone, the tear running
along the second fold for the left part and then tearing up and across,
ragged, with a tongue left where it turned. It lies with its two creases
sprung up (each quarter a flat facet, sagging onto the floor where it meets
it) and its torn lip lifting. Its top right corner is cut along the line a
dog-ear folds on (45 mm legs) and is its own part: on an unread note it is
curled up off the floor along that fold; on a note already read
(`fit_lore_notes` → `turn_down_note`, which puts the corner's pivot at
(0.08, 0.0115, 0.11), unturned) it lies folded down over the sheet as a
dog-ear, which is how the part is modelled.

- `noteSheet` (209 tris): the top side of the sheet (it lies on the floor);
  its UVs are its plan (the part's box frame), so the paper and the type
  are painted to the page's own layout.
- `noteCorner` (40 tris): the dog-ear, both sides, springing up a little
  toward its tip.

**Print and wear (painted, `canvas_tex`, their own streams).** The paper:
aged bond (#d6cdb5), the fibre's mottle, toned in from its cut edges,
grime toward the torn foot, each crease a pale broken line with dirt beside
it, foxing in one to three clusters of rust-brown specks with halos,
sometimes a dried water mark with its hard edge, sometimes the scuff of a
shoe, grit. The type: Courier at 10 to the inch and 6 (or 3) lines to the
inch, inch margins, paragraphs indented five, ragged right, the ribbon
inking each word differently, a word now and then doubled or struck out
with x's; the first lines short (clear of the corner that turns down); cut
off by the tear. It is drawn white on a transparent decal (the same sheet
geometry, polygon-offset) whose material colour is the ink, so a note in a
story's ink keeps its print (`ink_note` now tints a copy of the decal's
material instead of replacing it with a flat colour). The words are
letters at English frequencies: it looks like type and says nothing; what a
note says is read in the notebook. The corner: the same bond, greyed by the
floor and by hands, its fold a crease.

Draws: none on the level's stream (the painters draw from their own);
`tail_lore_notes` is unchanged.

## The mannequin (`core_mannequin`, `props_mannequin` in `core_props/mod.rs`)

**The real one.** A fibreglass male display mannequin with an egg head, on
a base plate with a calf rod. Sizes from makers' and dealers' sheets:
HT 1880, chest 990, waist 810 mm, fibreglass
(https://shopforshops.com/fibreglass-male-mannequin-egg-head-b9400wh );
73.5 in tall, chest 38, waist 28.5, hip 34 in, a chrome base through the
calf or the foot (https://www-barrdisplay-com-21978.30.mdoq.io/tradeshow-displays/mannequins-and-forms/retro-high-gloss-fiberglass-mannequins/retro-male-white-gloss.html );
190 cm, polyester reinforced with fibreglass, a calf pin and a base plate
(https://www.archiexpo.com/prod/best-mannequins-bv/product-4655965-2562761.html );
a 16 in round base, calf and foot fittings
(https://www.fixturesanddisplays.com/gene-abstract-male-mannequin-with-egg-head-pose-2-of-5.html );
a black lacquered steel plate, 38 cm (https://www.wayfair.co.uk/home-decor/pdp/symple-stuff-altonya-mannequin-statue-eaot1193.html ).
Its arms come off at the shoulders, its hands at the wrists, its torso at
the waist, its head at the neck, its legs at the hips: each a join line.

**What was made.** 1.89 m standing on a 400 mm black steel plate, 10 mm
thick. The torso lofted through 22 measured sections (chest 0.99 m, waist
0.79 m, hips 0.92 m in girth; the shoulders' trapezius slope, the chest's
depth, the buttocks); the egg head with a jaw, a chin and the faint ridge of
a nose and brow; legs with knees, calves and ankles and bare sculpted feet
turned out a little; arms from a domed deltoid that goes into the torso,
the elbow a little bent; hands with the fingers together and curled a
little, the thumb along the index finger, palms to the thighs. The arms and
the legs simply go into the torso, and the join line is traced where they
meet (so nothing floats and the seam covers the crease); the waist, the
neck and the wrists have theirs round them. Paint knocked off its edges
(knees, elbows, shoulders, toes, heels, knuckles, fingertips, hips, the
crown) shows the grey under it. A chrome calf rod comes up from a boss on
the plate behind the right calf and into its socket's plate.

Parts: `manTorso` 1056, `manHead` 480, `manLegL/R` 840, `manArmL/R` 468,
`manHandL/R` 360, `manSeams` 504, `manJoins` 1152, `manChips` 207,
`manPlate` 704, `manRod` 348 tris.

**Materials.** Gloss white paint over fibreglass gone yellow and dull
(Phong 0xd2ccbe, a weak broad highlight; the shader's wear mottles it and
dirties it toward the floor), the grey under the paint (0x8e8574), the join
lines' dirt (0x3e3933), black lacquered steel (0x151514), chrome (dark,
its colour in its highlight).

Draws: the same four, in the same order (`rand2`, `rand2` for where it
stands, `rand1` for its turn, `rand2` for its timer); its rules (its
timer, turning to you unseen, stepping closer, the wall test) are as they
were.

## The wall clock (`core_wall_clock`, `props_wall_clock` in `core_props/mod.rs`)

**The real one.** The institutional 12-inch wall clock of schools and
offices; the model is Lathem's school clock of the 1980s: 13.39 in (34 cm)
across, 1.97 in (5 cm) deep, a black metal frame, a convex glass, a metal
dial (https://www.1stdibs.com/furniture/decorative-objects/clocks/wall-clocks/vintage-black-school-wall-clock-from-lathem-1980s/id-f_35682842 );
Lathem's 12 in dials with the 24-hour numerals in red inside the hours
(https://www.ebay.de/itm/157036771569 ). Other 12 in clocks run 1.5 to
1.75 in deep (https://www.staples.com/infinity-instruments-business-prosaic-office-wall-clock-round-12-diameter/product_949675 ).

**What was made.** A black steel case 339 mm across and 50 mm deep: its
back on the wall, the drum, and the rolled bezel that holds a domed glass
7.5 mm proud; a 300 mm metal dial; stamped steel hands (a tapered hour
hand, the minute hand, the red sweep hand with its counterweight) and the
cap nut over them.

- `clockCase` 840, `clockGlass` 520, `clockHour` / `clockMinute` 96,
  `clockSecond` 124, `clockHub` 96 tris.

**Print (painted).** The dial: aged white paint, yellowed toward its rim
and blotchy, the minute track (its five-minute marks heavier), the hours
1-12 in bold Helvetica Neue, the 24-hour numerals small in red inside them,
QUARTZ under the pivot and MADE IN U.S.A. at its foot, a water mark dried
along its foot, fly specks. The glass: a film of dust, heaviest along its
top, wiped once across, fly specks, a pale band where the ceiling's light
lies on the dome.

**What it does, unchanged.** It hangs where it hung (the same draws: the
home, the wall spot), 2.1 m up; the hour hand stays where it always was
(2.4 rad), now with a minute hand at twenty-five past seven to go with it,
and the red hand sweeps on (`clock_data.sec_pivot`, turned by the HUD) and
ticks too fast within 4 m. The clock still lies.
