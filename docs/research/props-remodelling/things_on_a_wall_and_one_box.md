# t19 notes: P3, the rare: things on a wall, and one box

Seven placers, none of them seen by the census (rare, and instanced out of
its dump), made again as the real things a chapel's, a parlour's or a
barn's wall had on it, years after anyone dusted them. The parts are placed
as modelled (`prop_part_as_modelled`, a scale of one). The placers live in
modules of their own: `placers_p23/src/mystic/t19.rs` (the reliquary, the
crucifix), `placers_p18/src/t19.rs` (the shield of arms, the skull plaque),
`placers_p15/src/t19.rs` (the silhouette), `placers_p17/src/t19.rs` (where
a picture hung) and `placers_p06/src/rustic/t19.rs` (the wheel); each keeps
its draws call for call.

What the old ones were (judged from their code and their first frames): a
brown box with five glowing beads and a coloured point light pulsing round
it; two crossed planks; a fox, deer or raven skull of spheres and boxes
with black discs for eyes on a flat plaque (a glowing brass plate); a
framed sheet of parchment with a clip-art shield on it; a rectangle frame
round a canvas texture of a profile with a gold oval drawn on it; a pale
rectangle on the wall with two nails; a torus rim, a torus band and eight
sticks.

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Reliquary (`place_reliquary_boxes`, p23) | A small reliquary casket of gilt brass of the later 19th century: bun feet, a stepped plinth, a glazed body between turned columns, a cornice, a hipped lid with a cresting and a cross; paste cabochons in collets; inside, on a velvet cushion, the relic (a fragment of bone), its paper label (a cedula) and the red wax seal of its authentication | 14 x 9 cm, 11.6 cm to the cross (brass reliquary caskets sold at 14 x 7 x 6 cm and 15 x 6.5 x 10 cm) | https://www.sellingantiques.co.uk/pdf.asp?id=1144250, https://collections.imm.hu/gyujtemeny/reliquary-casket/43308, https://www.thehoarde.com/product/rag-and-bone/antique-french-gilt-brass-and-cut-glass-souvenir-jewellery-casket-19th-century-775544 |
| Crucifix (`place_wall_crucifixes`, p23) | A household wall crucifix of the common kind: two boards half-lapped, the face's edges chamfered, a cast corpus (spelter) nailed on, a tin titulus; hung from a nail through a hole behind its head | 28 x 16 cm (the old cross's), 16 mm thick; the corpus 13 cm hand to foot (sold at 10-12 in, a wood cross and a metal corpus) | https://www.zieglers.com/wall-crucifix-walnut-finish-cross-gold-finish-corpus-10/religious-articles/crucifix-crosses/, https://kingrichards.com/products/crosses-and-crucifixes/crucifixes/vintage-wood-and-metal-crucifix |
| Skull plaque (`place_skull_plaques`, p18) | A roe deer's skull, boiled and bleached, as a European mount (the whole skull bar the jaw, its palate to the board, its nose down, its frontal to the room) on a carved oak shield or an oval board; a buck's with its antlers (batch `t11`'s), a doe's without; a small brass plate under it with the day it was shot | the skull 190 mm long, 84 mm across the zygomatic arches, 52 mm between the orbits, the nasals 54 mm (an adult roe buck); the shield 18 x 25 cm, 20 mm and a 4 mm raised field (the old plaque's 18 cm width kept: the wall's probes; roe skull shields are sold 13 x 22 to 19 x 26 cm) | https://www.skullsite.co.uk/Roe/roe.htm, https://www.bushwear.co.uk/collections/all/products/solid-oak-roe-skull-shield-small |
| Shield of arms (`place_heraldic_crests`, p18) | A carved armorial plaque: a heater shield of oak, its edge moulded with a bead and a cove into the field, the field gessoed and painted, the ordinary carved proud and painted, the bead gilt over red bole, varnished | 33.6 x 41.6 cm (the old frame's), 2.2 cm deep (a carved armorial plaque of 1820-40: 34.3 x 29.9 x 3.8 cm) | https://www.1stdibs.com/furniture/wall-decorations/decorative-art/19th-century-carved-armorial-plaque/id-f_32155202 |
| Silhouette (`place_silhouette_portraits`, p15) | A cut-paper silhouette of the 1830s-60s: a bust in profile cut from black paper, laid on cream card, under glass with a pressed brass slip, in a small oval frame of turned pearwood (stained, ebonised or gilt), a brass ring at its top for its hook | 18 x 22 cm outside (the old frame's), the slip's window 12.7 x 16.7 cm (oval silhouettes framed 14 x 11.5 cm to 8 x 10 in) | https://www.proantic.com/en/print/fiche_objet?id=1165162, https://www.sulisfineart.com/framed-late-19th-century-cut-paper-silhouette-victorian-women-in-profile.html |
| Where a picture hung (`place_empty_painting_outlines`, p17) | The mark a picture leaves taken down after years: the wall behind it cleaner and less faded than round it, a ghost of dust and soot along its edges (thickest along its top, where warm air rising off the wall laid it); the two single-pin picture hooks it hung from (a strip of brass-plated steel, its top rolled round a hardened pin driven down at 45 degrees), now and then its stranded wire's cut end still on one | the hook 26 x 6 mm, its pin 1.5 mm; the mark the old patch's size with its border (20-42 x 20-44 cm) | https://www.lionpic.co.uk/p/8109/Picture-Hooks-1-Pin-26mm-Brass-200-pack-with-Pins, https://www.lionpic.co.uk/p/11614/Picture-Pins-Hardened-24mm-Knurled-Brass-Head-200-pack |
| Wheel (`place_wagon_wheels`, p06) | A small wooden wheel of the wheelwright's kind (a hand cart's or a goat cart's): an elm hub banded with iron and its iron axle box, ten oak spokes, five ash felloes of two spokes each, an iron tyre shrunk on and bolted through each felloe; hung over a wrought nail under its top felloe | 30 cm over the tyre (a 12 in wheel: the old wheel's 30 cm kept, the wall's probes), 10 spokes, a 3/4 in tyre 5 mm thick, the hub 9 cm long | https://hansenwheel.com/hobby-wheel (12 in: 10 spokes, a 3/4 in steel tyre), https://mdihs.digitalarchive.us/items/show/16445 (a 12 in wheel, wood frame and spokes, an iron strip), https://www.antikeo.com/en/catalog/collectibles/folk-art/pair-of-small-carriage-wheels-in-wood-and-iron-19th-century-perfect-condition-52607 |

A wagon's own wheels run 0.9-1.5 m; a wheel that size would pin faces the
old 30 cm one never touched (`settle_props` probes a wall prop's width),
so the real one chosen is a small cart's 12 in wheel. Likewise the skull's
board keeps the old plaque's 18 cm, the shield of arms and the silhouette's
frame the old frames' outsides, and the mark the old patch's size.

## The parts (`t19.py`)

- **Reliquary** (`Real__reliquaryBrass`, `…Glass`, `…Stone`, `…StoneBig`,
  `…Cushion`, `…Relic`, `…Label`, `…Seal`): the brass is one part (feet,
  plinth, columns, glazing bars, cornice, the hipped lid, its cresting of
  knops and leaves, the cross, the collets, an escutcheon, the hinges'
  knuckles); four bevelled panes; a cabochon (the game sets five in the
  collets, the old gems' order); the cushion plump in its middle and
  pressed where the relic lies; the relic a lofted shaft broken at both
  ends; the label a curled strip; the seal a pressed disc of wax.
- **Crucifix** (`Real__crucifixCross`, `…Corpus`, `…Title`): the cross two
  chamfered boards joined (a boolean union), knocked; the corpus of
  metaballs (the head bowed to its right, the chest, the loincloth and its
  knot, the legs and the right foot over the left), the arms swept as
  tubes up to the hands, voxel-remeshed, smoothed, flattened behind as a
  casting is, the nails' heads; the titulus a tin scroll curled at its ends.
- **Skull plaque** (`Real__skullPlaqueShield`, `…Oval`, `…Doe`, `…Buck`,
  `…Teeth`, `…Plate`): the shield batch `t11`'s heater at 18 x 25 cm
  (`shield_outline`, `shield` copied), the oval an ogee-edged board; the
  skull of metaballs in its own frame H (batch `t11`'s: the pedicles' tops
  at the origin, the nose toward +z): the braincase and occiput, the
  frontals, the orbits' bosses with the sockets open to the side, the
  zygomatic arches, the long maxillae and nasals, the premaxillae, the
  ethmoid gaps and the lachrymal pits, the nasal opening; sawn flat at the
  palate, the foramen magnum notched; the sutures grooved (coronal,
  frontonasal, the midline); the buck's with its pedicles. Turned 90
  degrees about x onto the board (nose down) as `t11`'s trophy is, so the
  game sets `t11`'s antlers on its pedicles at the same turn (0.663 rad).
  The cheek teeth six a side; the brass plate with two screws.
- **Shield of arms** (`Real__crestBoard`, `…Field`, `…Chevron`, `…Bend`,
  `…Cross`): the board a loft of the heater's outline offset inward along
  its moulding's section (the edge rounded, the bead, the cove down to the
  field); the field a thin plate laid in it; each ordinary a band clipped to
  the field (a boolean intersection), 3 mm proud, its edges bevelled.
- **Silhouette** (`Real__silhouetteFrame`, `…Slip`, `…Glass`, `…Card`,
  `…Ring`): the frame a loft of its closed section round the oval (the
  moulding's face, the sight's lip, the rebate behind); the slip a pressed
  ring with a bead at its window; the glass and the card ovals; the ring,
  its screw eye and boss.
- **Where a picture hung** (`Real__outlineHook`, `…Pin`, `…Wire`): the hook
  a bent strip (its plate on the wall, the sleeve for the pin, the foot
  bent out and up into the lip); the pin's shaft and knurled head; the
  wire's cut end, a loop over the lip and 6 cm hanging, kinked, splayed.
- **Wheel** (`Real__wagonWheelTyre`, `…Iron`, `…Hub`, `…Spokes`,
  `…Felloes`): sections swept round the axle (`ring_sweep`): the tyre, the
  felloes (five, a hair apart at their joints), the hub bands and axle box;
  the hub turned, bored through; the spokes lofted ovals tapering to the
  felloe, dished 3 mm; the tyre bolts' heads; the nail under the top
  felloe (the top between two spokes).

Budgets: 37.5k triangles over 35 parts; none past its budget.

## The game side

- **Materials and print.** Gilt brass dark in its diffuse and gold in its
  highlight, chased with a diaper of lozenges and quatrefoils, rubbed and
  tarnished; paste stones the old gems' colours as old glass is (dark,
  their colour in the highlight); faded silk velvet; bone; the label in an
  old hand; the windows' glass (`pictures::glass_mat`). The cross's grain
  up the upright and across the bar, its edges rubbed pale, dust on the
  bar's top; the corpus gilt spelter (or a grey one) worn to the metal.
  The skull's bone ivory, yellowed to the occiput and (a buck's) brown
  round the pedicles, its sutures and pits dark; `t11`'s oak, bone and
  antler painting copied for the board and the antlers. The arms painted
  by the game (the field's tincture, its charges with a dark line round
  them), then crazed, chipped to the gesso, the varnish gone amber; the
  bead gilt over red bole, rubbed on its crest. The silhouette's card
  painted (the bust a closed curve through the cutter's points, the name
  pencilled under) and aged as a print (`pictures::sheet_tex`), its frame
  painted by where each pixel lies on its moulding (`pictures::moulding_tex`,
  `Outline::Oval`), hung from `t01`'s brass hook (`pictures::hang`). The
  mark a decal (`lambert`, transparent, no depth write, polygon offset):
  the footprint lighter, its edge sharp, the ghost outside it; the scrape a
  decal of its own. The wheel's oak and ash weathered grey, checked along
  the grain, the cart's old paint (none, a red or a green, by where it
  hangs) kept in the sheltered corners; the iron rusted.
- **Variety without draws.** Each painter mixes where its thing is into
  its seed; the arms (which of three), the silhouette's sitter (which of
  three, facing left or right) and the wheel's paint are chosen by it too.
- **Arms of no real family**, invented: azure a chevron between three
  acorns or; gules a bend argent between two mullets or; vert a cross
  argent between four annulets or. (A first draft's azure a cross or read
  as a flag; changed.) No motto: the old Latin mottoes included real ones.
  The silhouettes' names pencilled under them are invented.

## Draws kept, call for call

| Placer | The old draws, in order | What each does now |
|---|---|---|
| reliquary | chance; tile; four gem colours; the medallion's colour; the glow's pulse phase; x; z; turn | the same; the gems' colours are the five paste stones'; the pulse phase drawn and dropped (`let _ = rnd()`) |
| crucifix | chance; wall; the wood colour; the slide along the wall (`hang_on_wall`); the tilt | the same; the colour picks the wood's stain; the tilt is about the nail it hangs by |
| skull plaque | chance; wall; plaque colour; bone colour; shield or oval; fox, deer or raven; the slide (`mount_on_wall`); the tilt | the same; the colours pick the oak's stain and how far the bone has yellowed; the shape the board; the old deer is the buck, the fox and the raven the doe |
| shield of arms | chance; wall; frame colour; the slide; the tilt | the same; the colour picks the oak's stain |
| silhouette | chance; wall tile; frame colour; the tilt | the same; the colour picks the frame's finish (stained, gilt, ebonised, gilt gone dull); the tilt is about its hook |
| where a picture hung | chance; wall; landscape or portrait; width; height; the wire (40%); the scratch (30%: its x, y, turn); the slide | the same; the decal is the old patch with its border, the hooks where the old hooks were |
| wheel | chance; wall tile | the same |

Checked with `--dump-level --flags noinstance` on each prop's floors,
before and after (the same `prop_meshes.bin` in both builds): every floor's
marks (each placer's draw count) the same, `counts.solidsHash` the same,
and every other placer's objects (placer, pos, min, max) the same.

## Behaviours kept or dropped

- The reliquary's point light and its pulsing animator are dropped (a gem
  that glows; `docs/design/PROPS.md`: nothing shines that does not); the pulse
  phase's draw is kept. The floor's animator count drops by one where it
  stands.
- The skull plaque's brass plate had an emissive; gone (brass does not
  glow). The old eye-socket discs and their helper (`sock_on`) went with it.
- Nothing else moved, sounded or was registered.

## Floors

None of these had been framed by the census; four stood on none of the
440 probe floors. A scan of seeds 6001-6300, levels 4-14 (`--dump-level
--flags noinstance`, the trace marks' draw counts) found them: the
reliquary on 6156/6, 6167/6, 6246/6, 6250/7, 6104/10; the skull plaque on
6084/9, 6163/10, 6016/12, 6023/10; the crucifix on 6228/12, 6156/12,
6081/10, 6158/10 and more; the wheel on 6005/10, 6081/9, 6084/9, 6020/11,
6177/10. The evidence floors: reliquary 6156/6, 6167/6; skull 6084/9,
6163/10, 6016/12, 6023/10; crucifix 6228/12, 6156/12; shield 5005/8,
6227/6; silhouette 5019/11, 6001/10; mark 5008/12, 6168/11 (with the
scrape); wheel 6084/9, 6005/10.
