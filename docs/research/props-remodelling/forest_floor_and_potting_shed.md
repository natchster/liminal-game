# b24 notes (nature: forest-floor litter, potting-shed plants and tools, framed botanicals)

Build: 81 parts, 17 606 triangles, 0 errors, 0 warnings. Every part is within its own budget.
The parts wrap 86 primitives in 29 of the 33 placers. One part serves both shears tips and one both shears handles. The two shears blades are told apart by occurrence (`occ`).
Shared parts: `GardenHoes__handle` also serves the rake handle, and `GardenHoes__collar` serves the rake and shovel ferrules.
Checked part by part, in the game on level 4 and on level 2 with seed 3, and in narrow-fov close-ups.

Whole objects sinking or hovering are left to the central placement pass. Below are part-to-part problems and wrong orientations.

## Per placer

- **place_dandelions**: `stem` (slightly bowed scape, swollen under the head), `wisp` (a seed of the clock: achene, beak, pappus tuft). Left: the puff sphere (transparent). Placement: `rotation.z = -a` only tilts each wisp in the XY plane. Wisps at a ≈ 0 or π stand vertical beside the puff, and those at a ≈ ±π/2 lie horizontal and tangential, so none radiate. They are centred at r 0.024, outside the 0.020 puff, and are not attached to it.
- **place_moss_patches**: `mat` (plane frame: low lumpy moss cushion with a scalloped edge, underside down to the floor), `tuft` (moss cushion with a flat underside and a lumpy top). Left nothing. No placement problem.
- **place_ferns**: `stem` (rachis grooved on top, drawn to a fine tip). Left: the leaflets (DoubleSide circles). Placement: the placer's taper is reversed. Its thick end (rb) lies at the frond tip (+x, the small pinnae) and its thin end at the base (the big pinnae). The part is thick at the base inside the same frame.
- **place_grass_clumps**: untouched. Its blades are DoubleSide planes.
- **place_butterfly_wings**: untouched. The wing is a transparent DoubleSide custom buffer geometry and the eyespots are flat decals.
- **place_snail_shells**: `ring` (a whorl: outer face sloping out towards the bottom, tucked sutures, uneven growth), `lip` (flared rolled aperture lip tapering into the whorl). Left nothing. Placement: the lip half-ring lies flat (rotation.x = π/2) at the height of the bottom whorl (y 0.006), 1.5 cm off the spire axis. It is half buried in that whorl and half sticks out as a flat arc, so it never stands up as an aperture.
- **place_pine_needles**: `needle` (half-round needle, flat face down, sharp tip, sheath sliver at the base, faint curve). Left nothing. No placement problem.
- **place_spider_webs**: untouched. The web is a transparent DoubleSide custom buffer geometry.
- **place_bird_nests**: `twig` and `twigThin` (crooked twigs: kinks, bark lumps, nodes, a snapped side stub, broken ends), `egg` (blunt end down, pointed end up). Left nothing. Placement:
  - `rotation.z = PI/2` followed by `rotation.y = a + PI/2` turns each twig's axis to (sin a, 0, cos a). That is tangent only at a = 0, π/2, π and 3π/2, and radial on the diagonals, so the "woven ring" reads as a star of sticks (visible in every shot). `rotation.y = PI/2 - a` would lay them tangent.
  - The eggs (y 0.024, bottom at 0.011) hover 1.1 cm above the floor, because the nest has no floor under them.
- **place_chestnuts**: `nut` (plump nut: broad scar base, one face flattened), `tip` (dried style tuft), `spike` (a little burr of three spines). Left nothing. Placement: `rotation.z = -a` gives each spike the axis (sin a, cos a, 0). The spike at a = 0 points straight up, the one at a = π points down into the nut, and those at a = ±π/2 lie tangentially along x, so the husk spines never radiate.
- **place_hanging_plants**: `arm` (forged bracket: wall plate, twisted square bar, an eye at the end), `chain` (five alternating links), `pot` (planter bowl: foot recess, rolled rim, soil face just under the dirt disc). Left:
  - the dirt disc: a flat opaque disc at 1.83 m, only ever seen from below
  - the vine leaves (DoubleSide)

  Placement: the single chain hangs over the pot's centre and ends 6 mm above the rim (chain bottom at mountY − 0.060, pot top at mountY − 0.066), so the pot hangs from nothing.
- **place_seedlings**: `pot` (terracotta pot with a collar rim), `dirt` (plane frame: crumbly soil sinking under the rim), `stem` (hypocotyl with a slight crook, swelling under the seed leaves). Left: the cotyledons (DoubleSide). Placement: the dirt disc sits 1 mm above the rim. The new soil is lowered at its edge, so the rim now shows round it.
- **place_trowels**: `blade`, `tip`, `shaft` (shank with a ferrule collar), `handle` (grip: ferrule ring, waist, palm swell, flared butt), `cap` (end knob with a hanging hole).
  - The blade is a thin dished scoop sheet with its hollow on the half-cylinder's flat side, running from a narrow heel to a full belly.
  - The tip is an ogive point pointing outward.

  Left nothing. Placement:
  - The blade's `rotation.x/z = PI/2` stands the scoop on its edge: the hollow faces sideways (−z), not up, and the blade reaches 1 cm below the floor (y −0.010).
  - The tip cone (`rotation.z = -PI/2`) points its apex back into the blade with its wide base outward. The part is modelled pointing outward inside the same frame.
- **place_seed_packets**: untouched. The packet is a textured DoubleSide plane.
- **place_watering_cans**: `body` (galvanised can: foot bead, two swaged ribs, rolled top seam, low dome with a fill opening), `spout` (tapered pipe: soldered flange at the wide end, rolled lip at the narrow end), `rose` (conical back, slightly domed face in a beaded rim), `handle` (folded strap with rolled edges). Left: the five hole dots (decals). Placement:
  - `rotation.z = -PI/3` tilts the spout the wrong way. Its narrow end pokes out of the can top near the centre (x −0.015, y 0.094), and its wide end hangs down and out at (−0.097, 0.046), 5 cm below the rose.
  - The rose at (−0.098, 0.098) floats unattached. `+PI/3` would bring the spout's end to the rose.
  - The handle's `rotation.y = PI/2` puts the half ring across the can (the YZ plane), tilted 0.6 rad. One end goes into the body (z +0.031, y 0.065) and the other hangs in the air (z −0.031, y 0.108).
- **place_gardening_gloves**: `palm` (stuffed fabric palm: flat underside, puffed back, welted edge, rounded outline narrowing to the wrist), `thumb` (knuckle crease, tip curling towards the palm), `finger` (rounded tip, knuckle crease), `cuff` (knit cuff with rib ridges). Left nothing. No placement problem.
- **place_pruning_shears**: `blade` (occ 0: sharp bypass blade, spine to edge on +z), `hook` (occ 1: broad blunt anvil hook, edge on −z), `tip` (hooked point), `pivot` (domed bolt and nut), `handle` (moulded grip: slim neck, grip rings, rounded butt). Left nothing. Placement:
  - The blades lie flat: 5 mm thick, 10 mm wide, overlapping by 2 mm in z.
  - With `rotation.z = ±0.15`, the blades' far ends sit at y 0.0046 (upper) and 0.0114 (lower), but the tips sit at y 0.010 and 0.006. The tips are swapped vertically by about 5 mm.
  - Both tip cones point their apex back towards the pivot (the part points outward inside its frame).
  - The handles start at x 0.0125, 3.5 mm past the pivot bolt, and do not join the blades (which end at x 0.0025).
- **place_garden_hoes**: `handle` (ash handle: faint swell, out-of-round grain, rounded butt, shoulder into the ferrule), `collar` (crimped ferrule with a rivet), `head` (draw-hoe blade: rounded back corners, bevelled convex front edge, socket boss), `cap` (turned end knob). Left nothing. Placement: the head (x −0.302 to −0.286) floats 11 mm beyond the collar (which ends at −0.275), with no neck joining them.
- **place_garden_rakes**: `bar` (flat bar on edge with eased edges and rounded ends), `tine` (thick root on top, slight curve, point), `cap` (end knob). The handle and collar use `GardenHoes__handle` and `GardenHoes__collar`. Left nothing. Placement:
  - The bar is 7.5 mm beyond the collar.
  - The tines (x −0.325) stand 6 mm in front of the bar (x −0.3175 to −0.3125) and never touch it.
  - The tines are centred on the bar height, so half of each tine sticks up above the bar and the lower half reaches about 6 mm into the floor.
- **place_garden_hoses**: `loop` (a turn of the coil: not quite round, rising and falling a little, a moulded stripe on top), `nozzle` (brass: knurled coupling, grip ring, flared front), `tip` (spray tip with an orifice). Left nothing. Placement:
  - The loops are four separate concentric rings 1 cm apart, not a continuous coil.
  - The nozzle (x 0.07 to 0.13, z 0.010) lies across the outer loop (the ring passes through it at x ≈ 0.09), and no hose end runs into it.
- **place_garden_shovels**: `handle` (the tool handle at the shovel's size), `blade` (dished spade blade thinning to the front edge, rounded front corners, socket strap), `grip` (D-grip rod thickening into the hand grip). The collar uses `GardenHoes__collar`. Left: the blade tip cone, `cone_geo!(0.045, 0.030, 3)`. Its `rotation.z = -PI/2, rotation.x = PI/2` makes it a 9 cm tall, 3-sided pyramid standing across the blade's end, apex back into the blade, reaching 3 cm under the floor. A faithful flat spade point would be 8 mm thick in a 90 mm-deep frame, a frame error, and a refined pyramid would not read better. From above the primitive already reads as a spade point. Placement:
  - The blade is 1 cm beyond the collar.
  - The tip pyramid is the lowest point of the shovel (y −0.031), so the whole shovel gets lifted.
  - The D-grip (`rotation.y = -PI/2`) stands as an upright arch across the handle axis, 1.5 cm past the handle end (x 0.290 against 0.275). Its feet (z ±0.024) are not connected to the handle, and the "two side bars" in the comment are never made.
- **place_soil_bags**: `body` (filled sack: bulging sides and top, slumped, creased plastic), `cap` (bag end drawing in to a flat heat-sealed seam at both ends of its frame; the inner seam hides in the body). Left nothing. The textured material maps through the box and sphere UV conventions as before. No placement problem.
- **place_kneeling_pads**: `body` (closed-cell foam: well-rounded edges, a shallow recessed kneeling panel, the carry slot cut through), `cutout` (stadium plug with a dished top that reads as the slot's shadow). Left nothing. Placement: the old cutout box's top was coplanar with the pad top (z-fight). With the slot cut, the plug now shows as the hole.
- **place_twine_balls**: `ball` (wound ball, crossing bands standing proud, a little lumpy), `wrap` (a turn of 2-ply twine), `end` (twisted, wandering, drooping loose end, frayed at the free end). Left nothing. Placement: the trailing end (y 0.010) sticks straight out of the ball's side and hovers 9 mm above the floor. The part droops 1.3 mm, all its frame allows.
- **place_seed_catalogs**: `frame` (dark oak ogee moulding round a recessed backing board; the rails are deeper at top and bottom, following the page). Left: the page (a textured plane). Placement: the page (z 0.008) floats 2 mm in front of the frame's face (0.006).
- **place_pressed_flowers**: `frame` (painted shadow box: rounded face, deep inner well down to the mount). Left:
  - the paper (a textured plane)
  - the glass (transparent basic material)

  Placement: the paper sits 1 mm in front of the frame face (0.010 against 0.009), and the glass floats 4 mm in front.
- **place_garden_aprons**: `pocket` (canvas patch pocket: hemmed opening towards the waist, stitched divider, puckered). Left: both folded layers and the straps. They use the DoubleSide apron material. No placement problem.
- **place_bird_feeders**: `seeds` (seed column packed against the tube, heaped uneven top), `cap` (overhanging skirt, low dome, hanger boss), `base` (seed tray with a rolled rim), `perch` (rod with a collar at the tube and a ball end), `loop` (wire ring with its twisted ends). Left: the tube (transparent). Placement: `rotation.x = PI/2` lays the hanging loop flat (horizontal), and it floats 8 mm above the cap (y 0.200 against a cap top of 0.192). A hanging loop should stand vertically on the cap boss.
- **place_botanicals**: `frame` (flat reeded frame: raised outer bead, two reeds, a step to the sight edge, backing board). Left: the paper (a textured plane). Placement: the paper (z 0.009) floats 2 mm in front of the frame face (0.007).
- **place_garden_gnomes**: `boot` (toe cap lower than the shaft, toe at −z), `body` (pot-bellied robe: flared hem, belted waist, round shoulders, sleeves meeting in front), `head` (brow, sunk eyes, apple cheeks, a hollow for the nose ball, ears), `beard` (wavy locks drawn to a curling point), `hat` (felt cap with concave flanks, the point flopping back), `brim` (rolled felt band), `nose` (bulb). Left nothing. The face is modelled on −z, where the placer puts the nose. Placement:
  - The nose (z −0.020) and the beard (z +0.010) are on opposite sides of the head.
  - The beard cone (apex down, y 0.070 to 0.100) is almost entirely inside the head and body, and shows only as a white sliver at the back of the head (visible in the game).
  - The boots (x ±0.015) are fully under the robe's 0.024 radius and show only from low angles.
- **place_fertilizer_spikes**: `body` (fluted shank, collar under a flat head), `tip` (fluted point). Left nothing. No placement problem.
- **place_herb_bundles**: `hook` (cup hook: wall rosette, wire turned up), `twine` (two turns of twine with a knot). Left: the stems and leaves (DoubleSide). Placement: the hook (mountY + 0.08) is 3 cm above the twine binding (mountY + 0.05), with nothing between them, so the bundle hangs free of its hook.
- **place_dead_potted_plants**: `pot` (old terracotta pot: foot ring, tapered body, thick collar, weathered), `soil` (plane frame: dry, cloddy, mounded soil sinking under the rim), `stem` (withered stalk: crooked, nodes, two snapped side shoots, the tip flopping over). Left nothing. No placement problem: the tilted stems stay inside the pot, and their feet stay under the soil.
