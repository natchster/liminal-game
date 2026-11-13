# t09 notes: P3, the rare: a shrine's things

Five placers the census graded `cartoon`, made again as what a person
brought to a shrine and set out (`docs/design/LORE.md`; `docs/design/PROPS.md` 1, "the
party and the shrine"): ornaments and real things, never glowing or working
occult devices. The parts are placed as modelled (`prop_part_as_modelled`,
a scale of one). The placers live in modules of their own:
`placers_p23/src/mystic/t09.rs` (the glass skull, the owl, the candlestick,
the skull stack) and `placers_p26/src/toys/t09.rs` (the mask); each keeps
its draws call for call.

The census's verdicts: the crystal skull "a featureless translucent white
ball glowing pale green ... It also emits light like a lamp" (review7);
the skull stack "three plain beige spheres stacked ... no sockets,
cheekbones, jaws or sutures" (review7); the candlestick "the brass is a
flat lemon-yellow ... the flame's glow is a visible octagonal sprite"
(review1); the owl "a ball head with ear tufts on an egg body ... lamp-like
yellow eyes" (review7); the mask "a flat pale oval disc with two black eye
slits ... a sticker, not a mask" (review7).

## What each is, and where its size comes from

| Prop (placer) | The real one | Size used | Source |
|---|---|---|---|
| Skull stack (`place_skull_stacks`, p23) | A life-size cast resin skull replica, "aged": an ivory resin under a brown antiquing wash that lies dark in the orbits and the nose; the mandible wired on, the teeth cast with it | 188 mm glabella to the back, 151 mm across, 217 mm crown to chin on its jaw (the replicas sold run 19.6 x 13.7 x 15 cm without the jaw shut; an adult is about 170-190 mm long, 140-145 mm wide) | https://www.desertcart.sg/products/367374684-life-size-human-skull-model-1-1-replica-realistic-human, https://www.desertcart.ma/products/367374684 |
| Crystal skull (`place_crystal_skulls`, p23) | A cast glass "crystal skull" ornament: lead-free glass, the skull simplified (the jaw fused shut, the teeth a band), fire-polished, its base ground flat | 105 mm deep, 72 mm across, 96 mm high (NUDE's small skull: 83 x 80 x 105 mm; a Wayfair glass skull 3.27 x 4.13 x 3.15 in) | https://eu.nudeglass.com/products/rock-pop-memento-mori-skull-small-cins3000, https://www.wayfair.com/-/pdp/-nudd1063.html |
| Brass candlestick (`place_brass_candlesticks`, p23) | A turned and cast solid brass candlestick of the common kind: a stepped round foot with a hollow underside, a baluster stem with knops, a drip pan and a socket for a 7/8 in taper; a dinner taper burnt down in it, the wax run over the socket and onto the pan and foot | 155 mm tall, its foot 104 mm across, the socket 22 mm; the taper burnt to 70 mm (A) or a 38 mm stub (B) above the socket's floor | 6 in candlesticks with a 7/8 in socket: https://www.zieglers.com/altar-candlestick-sold-in-pairs-6-1936-ziegler/church-goods/, https://shop.fullycatholic.com/?p=8142; a 6 in solid brass stick: https://mylampparts.com/products/sl6562 |
| Owl (`place_small_owls`, p23) | A carved wooden owl, the souvenir kind carved from one block and stained dark, standing on a rock carved with it, set with glass eyes (amber, the pupil painted behind) | 134 mm to its ear tufts, 104 mm across its rock (a Black Forest owl of c. 1900, glass eyes, on a rock base: 6 3/4 in) | https://inkwellresearch.com/?p=34004 |
| Mask (`place_small_masks`, p26) | A Venetian volto: a full-face carnival mask of papier-mache, gessoed and painted, the eyes cut through, gilt round the eyes and the edge, tied by two satin ribbons from the temples | 150 mm across, 216 mm brow to chin, 76 mm deep to the nose's tip (a volto argento: about 22 x 15 cm; another face 24.5 x 17.5 cm, 11 cm deep) | https://www.outfit4events.com/eur/product/8327-venetian-mask-volto-argento, https://www.desertcart.nz/products/113227975 |

## The parts (`t09.py`)

- **Skull** (`Real__skullStackBone`, `…Hollows`, `…Teeth`): the cranium of
  metaballs in the Frankfurt horizontal (the vault, the frontal, the
  occipital, the brow ridge, the nasal bones, the maxilla, the parietal
  bosses, the cheekbones, the orbits' rims, the zygomatic arches, the
  mastoids), the upper alveolar ridge, voxel-remeshed; the orbits (a
  squared opening narrowing 48 mm back, the lateral side lower), the pear
  of the nasal aperture, the temples' hollows and the ear openings cut;
  the mandible a tube along its path from the chin round the angles up
  the broad rami to the condyles, the coronoids under the arches, wired on
  in front of the ears; the teeth along two parabolic arches (incisors to
  an edge; 25 of them, three lost), the bite closed. Turned up 12.5 degrees as it sits
  on its jaw. The hollows are a lining just inside each orbit and the
  nose (where the wash lies; the septum and conchae in the nose's dark).
- **Crystal skull** (`Real__crystalSkull`): the resin skull's cranium, jaw
  and teeth fused into one cast with fillers where glass cannot hold air
  (the mouth, between the rami and the base, the temples), scaled to 105
  mm, remeshed and polished; the sockets and the nose cut again deeper as
  the mould has them, the bite and the gaps between the teeth grooved,
  the base ground flat.
- **Candlestick** (`Real__brassStickBody`, `…CandleA`/`B`, `…WickA`/`B`,
  `…FlameA`/`B`): the body one lathe of 56 profile points (the foot's
  bead and steps, the domed top, the lower knop, the baluster, the upper
  knops, the pan with its rolled rim, the socket and its lip; the
  underside hollow); the taper 22 mm tapering a little, its top a crater
  melted through on one side, five runs down its side, three over the
  socket's rim and down it, a collar of wax in the socket's mouth, a drop
  on the pan and one on the foot; the wick a braided cotton tube curled
  over at its tip; the flame a teardrop 11 x 27 mm.
- **Owl** (`Real__owlWood`, `…Eyes`, `…Pupils`): one carving: the body,
  breast and broad head of metaballs, the facial disc two dishes cut round
  the eyes, the folded wings shells standing off the sides, their tips
  crossed over the tail, the tail a wedge laid down the back, the hooked
  beak, two ear tufts, three toes a foot over the rock's front; the rock
  four lumps, chiselled; the glass eyes 15 mm spheres, the pupils discs on
  their fronts.
- **Mask** (`Real__maskFace`, `…Ribbons`): the face a height field (the
  face's curve, the forehead, the brow, the eye sockets, the cheekbones,
  the nose's bridge, tip and wings, the lips closed, the chin), solidified
  2.6 mm thick, cut to the volto's outline (a little square at the brow,
  narrower at the chin) and its almond eyes; laid face up, tilted 4
  degrees onto its brow's edge; the two 10 mm ribbons from its temples
  down off its rims and out across the floor, a long one curled back, a
  short one straight, turned over once or twice.

## The game side

- **Skull stack**: the old count (2 or 3) is kept: two, one set on the
  other's crown (0.201 m up, measured on the model by casting down from
  the top skull onto the lower), or three, two side by side 0.154 m apart
  and one across them (0.207 m up). Each skull's old offset (+-2 cm), turn
  and (above the first) tilt draws are kept and used a little: set off by
  a few mm, turned +-0.25 rad, tilted +-0.06 rad. The bone is an aged
  resin painted per stack (`bone_tex`: ivory under a brown wash pooled in
  blotches, the sutures' lines, pores, dust); the lining a dark wash; the
  teeth a yellower resin.
- **Crystal skull**: glass as the wine glass's (`Patch::Glass`): the room
  in it by Fresnel, a cast's grey body (milk 0.34), dust and smears from
  `glass_dust_tex`; drawn after the opaque things. The old glow light and
  its pulse are gone (glass does not shine); the pulse phase's draw is
  kept.
- **Candlestick**: the old drips' first draw chooses the taper (A or the
  stub B); the other two, the wax pool's eight radii, the lit draw and,
  when lit, the flicker's phase and rate are kept. Lit (60%) as the old
  one was: the same point light (0xffb060, 0.35, 0.9, decay 2), now 1 cm
  over the flame's root, and the same flicker of its intensity and the
  flame's height (the flame scaled from its root); the octagonal halo
  sprite is gone. Brass dull and brown (`tarnish_tex`, phong), the wax
  ivory gone grey with dust and soot at the crater (`wax_tex`), the flame
  a basic material (a flame) from a blue root to a yellow-white body to an
  orange tip (`flame_tex`, shared).
- **Owl**: the old colour pick is the stain; the wood (`owl_wood_tex`)
  painted from the front and sides: the stain over a pale grain, the
  feathers' chisel scallops dark with the stain (larger low on the body
  and toward the sides), fine rays on the facial disc, the rock's facets,
  the crown and tufts rubbed lighter, dust. The eyes amber glass and the
  pupils painted black, both phong: they catch the light, never make it.
- **Mask**: the old colour pick is the paint; the face (`mask_tex`, phong
  with a satin highlight) painted from above in the face's own mm: the
  paint over gesso, crazed; gilt in a band round each eye and along the
  edge; rubbed through to the brown paper where the relief stands out
  (the same height field as the model); grime in the hollows; dust. The
  ribbons faded satin, black or a dark red by where the mask lies.

Every painter mixes where its thing lies into its seed (`salt`), so no two
floors' things are painted alike, and adds no level draw.

## Draws and evidence

On each prop's census floor and one more where it appears (the stack 2/7
and 13/14; the glass skull 9/11 and 2/13; the candlestick 4242/14, 9/18,
19/18 and 9/19; the mask 13/7, 14/16 and 15/16; the owl 19/7, the only
floor of about 460 scanned it stands on), `--dump-level` (with `--flags
noinstance`: a stack's parts all repeat and are instanced) gives the same
draws, the same `solidsHash` and every other placer's objects the same
(placer, pos, min, max, rot) before and after.
