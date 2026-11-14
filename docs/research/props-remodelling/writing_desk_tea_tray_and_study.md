# b20 notes (victorian)

All 11 placers are covered: 108 parts, 26 458 triangles, wrapping 119 primitives. 0 errors, 0 warnings.

Checked part by part (the whole batch, then each placer) and in the game: level 4, seeds 3 and 11, the sheet variants, level 2.

Shared parts:
- `SpecimenDomes__bead`: bird eye and butterfly pin head.
- `SpecimenDomes__pin`: butterfly and beetle pins.
- `TarnishedMirrors__rosette`: the corner bosses and the crest rosette.
- `SheetCoveredFurniture__skirt` and `__fold`: chair and wingback.

The picture-frame bars (maps, botanical, mirror) come from one primitive that makes both the long top/bottom bars and the tall side bars. The game fits parts per axis, so each bar is a moulding that is symmetric in X and Y: the face height is the product of the same edge profile across and along the bar.

- **place_inkwell_letters**:
  - Parts: `sheet` (tri-folded letter with one mountain and one valley crease and lifted edges; also used for the blank sheet), `barrel` (turned barrel with grip section and thread ring), `capband` (metal end cap with a domed end), `clip` (spring clip with a ball foot), `nib` (section collar and curved pointed nib, convex side up), `drop` (teardrop), `neck` (brass collar band), `cap` (knurled screw cap with stepped top and button).
  - Left as primitives: the jar and ink volume (transparent), the ink surface and smudge (transparent circle_geo! decals).
  - Placement: the loose cap (radius 2.5 cm, centre 4.5 cm from the jar axis) cuts about 4 mm into the jar wall. The jar stands 8 mm over the letter's edge, so its base intersects the sheet.
- **place_silver_trays**:
  - Parts: `tray` (flat salver with a border step and foot ring), `rim` (18-lobe gadroon border), `handle` (cast D-section loop with swelling feet), `engraving` (twisted-rope engraved ring), `cup` (hollow footed teacup; the tea disc shows 3 mm under the lip), `saucer` (cup well, marly, foot ring), `cupHandle` (porcelain ear with a thumb rest), `spoon` (flat fiddle stem), `spoonBowl` (egg-shaped bowl, built at true size against the 1.4 × 0.4 × 1 scale), `plate`, `biscuit` (fluted shortbread), `sugar` (bellied bowl with a lid gallery), `lid` (domed lid with a flange), `knob` (turned finial), `napkin` (two folded linen layers).
  - Left as primitives: the tea surface and the steam wisps (transparent / basic material).
  - Placement:
    - The handles (half torus, rotation.z π/2) stand upright, curve inwards over the tray and reach y −0.010, 1 cm below the tray's underside, instead of looping outwards at rim level.
    - Everything on the tray floats 2–2.5 mm above the tray top (y 0.012): saucers, plate and napkin bottoms at 0.0145, sugar bowl at 0.014.
    - Each cup's foot (y 0.014) is 3.5 mm inside its saucer (top 0.0175).
    - Each cup handle ring (rotation.y π/2) lies in the plane tangent to the cup, 2 mm outside the cup wall, instead of looping out radially.
    - The spoon bowl (y 0.0084–0.0116) is buried in the tray (top 0.012).
    - The sugar lid's rim (y 0.046) floats 8 mm above the bowl top (0.038). The lid knob (y 0.042–0.050) sits under the lid in that gap instead of on top of it (0.060).
- **place_antique_maps**:
  - Parts: `bar` (walnut frame bar: rounded edge, bead, cove, crowned face).
  - Left as primitives: the map canvas (plane_geo!, DoubleSide texture).
  - Placement OK.
- **place_pocket_watches**:
  - Parts: `body` (case with a domed back, middle band and bezel seat), `rim` (half-round milled bezel), `hourHand` (spade hand with a pivot boss), `minHand` (tapered whip hand), `pin` (domed cannon pinion), `crown` (fluted onion crown), `cover` (hunter lid, domed outside with a snap lip inside), `link` (twisted curb link), `fob` (T-bar with ball ends and a centre boss), `keyShaft` (watch-key pipe with a collar), `keyHead` (trefoil bow), `keyBit` (chamfered).
  - Left as primitives: the dial (flat circle_geo! face), the 12 ticks (sub-centimetre, 1.2 × 0.6 × 4–6 mm), the two engraving bars (sub-centimetre dark marks).
  - Placement:
    - Body: rotation.x −π/2 turns the case's axis to Z. The case stands on its edge (y −0.025…0.035), cutting vertically through the flat dial and bezel and sinking 2.5 cm below them. Clearly visible in game.
    - Crown: rotation.z π/2 lays its axis along X, tangent to the case, instead of pointing out radially at −Z.
    - Cover: its lowest edge (y 0.015, z 0.041) is 1 cm behind the case edge and 4 mm above the bezel, so the hinged lid floats free of the case.
    - Chain: the eight 1 cm links are 1.4–2.6 cm apart, so they read as a dotted line of loose rings.
- **place_fire_tool_sets**:
  - Parts: `base` (cast plinth with foot bead, cove and ogee), `rod` (twisted square bar with turned collars and an arm hub), `finial` (acorn on a neck), `arm` (rod with a knob end), `hook` (tapered hook), `shaft` (twisted tool shaft with collars; all four tools), `pokerTip` (forged, quarter-twisted square head), `grip` (turned handle with a ball end), `pincer` (flat forged tong arm with a gripping pad), `loop` (twisted square ring), `scoop` (pan with turned-up sides), `scoopBack` (arched back plate), `bristles` (splayed tufted brush head), `lump` (faceted cinder).
  - Left as primitives: the ash pile (flat circle_geo! decal).
  - Placement:
    - The tongs' loop grip lies horizontal (rotation.x π/2) and floats 1.1 cm above the top of the tongs' shaft, not joined to it.
    - The shovel pan lies horizontal under a vertically hanging shaft, and the shaft ends 2.5 mm above the pan.
    - The poker tip cone has its narrow end up at the shaft and is offset 6 mm sideways. It reads as a flared foot, so the part is modelled as a forged head flaring downwards to match.
- **place_botanical_prints**:
  - Parts: `bar` (slim oak frame bar: rounded edge, fine bead, flat face).
  - Left as primitives: the cream mat (an untextured flat plane_geo! quad directly behind the print; a plane-frame part would add nothing visible) and the specimen canvas (DoubleSide texture).
  - Placement OK.
- **place_specimen_domes**:
  - Parts: `base` (ebonised plinth with foot bead, cove and ogee), `baseRing` (gilt bead moulding), `domeRim` (gilt collar with a top bead), `plaque` (canted brass plate with two pins), `perch` (knobbly twig with a snapped stub), `birdBody` (plump torso with a low breast and lifted rump, built against its 1 × 0.85 × 1.3 scale), `birdHead` (round crown, face drawn forward to the beak), `beak` (hooked seed-eater's beak), `tail` (pointed feather bundle with quills), `wing` (folded wing with stepped covert rows), `bead` (eye and pin head), `pin` (insect pin), `bfBody` (thorax and ringed abdomen), `bfHead` (bulging compound eyes), `antenna` (clubbed tip), `beetle` (domed elytra with a sunken seam, waist, pronotum; true size against the 1 × 0.4 × 1.6 scale), `beetleHead`, `label` (curled card).
  - Left as primitives: the glass dome (transparent), the plaque's engraving lines (sub-millimetre), the bird's legs (0.8 mm) and the beetles' legs (0.4 mm), the elytra split line (0.4 mm), the butterfly wings and wing marks (circle_geo!, DoubleSide).
  - Placement:
    - The plaque (z 0.072–0.074) and its engraving lines are inside the plinth (plinth surface r ≈ 0.078 at that height), so they are invisible.
    - The perch (y 0.019–0.025) floats 3.5 mm above the plinth top (0.0155) with nothing holding it.
    - The bird body and wings are stretched 1.3× along Z (sideways), while the head and tail run along X. The bird is wider than it is long, and the wings stick out sideways.
    - The tail is flattened with scale.z, so it stands as a vertical fin instead of a flat fan.
    - The beak's base is buried in the head. The head part narrows towards the beak so more of it shows.
- **place_secretary_desks**:
  - Parts: `cabinet` (carcass with a moulded top, chamfered plinth and fielded front and side panels), `leg` (turned baluster leg with a bun foot), `drawer` and `sdrawer` (drawer fronts with a cock bead), `pull` (mushroom knob), `backplate` (oval escutcheon), `dropPanel` (fall front with a sunk writing field), `hinge` (butt-hinge leaf with knuckle and screws), `stay` (folding brass stay), `upper` (upper case with six pigeonholes cut 4 mm into its front), `roll` (rolled document with a spiral end), `knob`, `paper` (curled sheet), `cornice` (cove and ogee cap moulding, flush at the back).
  - Left as primitives: the pigeonhole backs (black 1 mm boxes, a dark decal), the inkwell jar (transparent), the ink top (circle_geo!).
  - Placement:
    - Drop-front panel: the group's rotation.x −π/2+0.05 turns the panel upright, and dropH/2 along its local Y moves it 15 cm back. The panel stands buried inside the cabinet and upper case (z 0.02–0.03) instead of lying open in front. Its sides coincide with the upper case's sides (z-fighting).
    - The papers and inkwell on the panel are buried with it, 5.5 cm below the panel face.
    - The two support stays float 15 cm in front of the cabinet with no panel to hold.
    - The pull backplates (front z 0.1795) are inside the drawer fronts (front face 0.1805), so they are invisible.
    - The pigeonhole backs were 1.3 mm inside the upper case's front. Now that the part has real pigeonholes, the backs show through and the paper rolls sit in the holes.
- **place_tarnished_mirrors**:
  - Parts: `bar` (gilt frame bar: rounded edge, two beads, cove, sunk channel, crowned face), `rosette` (eight-petal carved boss with a centre button; the 4 corners and the crest), `pediment` (scrolled crest with an applied carved panel), `pendant` (acorn drop).
  - Left as primitives: the mirror surface (plane_geo!, DoubleSide texture).
  - Placement OK.
- **place_sheet_covered_furniture**:
  - Parts:
    - `chairBody` (occ 0) and `wingBody` (occ 1): draped blocks with vertical folds that only push outwards, a fuller drape at seat height and a top edge that rolls over.
    - `skirt`: flared hem with floor folds, drawn in under the body.
    - `fold`: two soft creases feathering into the top surface.
    - `arm`: padded roll.
    - `wing`: pillowy padded block.
    - `tableTop`: sheet flat on top, rolling over the edges.
    - `legBump`: soft post.
    - `centerFold`: long crease.
    - `bump`: mound with radial folds over a hidden object.
  - Left as primitives:
    - The chair's back bulge: it lies entirely inside the body (z −0.254…−0.214 within −0.26) and is never visible.
    - The four table skirt panels: one constructor makes both the thin-in-Z front/back panels and the thin-in-X side panels, and fitting is per axis. Folds modelled for one orientation would smear across the other.
    - The dust circle (transparent).
  - Placement:
    - The chair's back bulge is completely buried in the body.
    - The chair's arm boxes are only 1 mm proud of the body's sides and do not read as arms. The drape covers them now.
    - The table's leg bumps are 7 cm inside the skirt panels and only show through the 3 cm gap under the skirt.
    - The table's centre fold floats 2.5 mm above the table top (y 0.8125 vs 0.81).
- **place_servant_bell_pulls**:
  - Parts: `bracket` (3.5 mm wall plate with a scrolled cast arm), `bell`, `bellTop` (domed cap with a hex nut), `clapper` (ball on its stem), `screw` (slotted domed head), `spring` (coiled split ring), `cord` (three-strand laid cord), `braid` (twisted whipping band), `bulb` (thread-fluted tassel mould with neck and collar, built against the 1.3× y scale), `band` (twisted gilt gimp), `strand` (wavy fringe cord with a knotted end).
  - The bell primitive is a funnel: it is wide (r 4.5 cm) at the top, where the 3 cm cap cannot close it, and narrow at the bottom. The part is modelled as a proper bell, mouth down, inside the same bounds: the crown (r 3 cm) at the top under the cap and the lip (r 4.5 cm) at the bottom. It reads as a bell in game.
  - Nothing left as a primitive.
  - Placement:
    - The two screws (z 0.0015–0.0045) were buried inside the solid bracket box. The bracket part's thin wall plate now lets their heads show 1 mm proud.
    - The sway animator moves the cord, tassel and strands in x but never the three braid rings (the "Also the 3 braid rings" comment has no code). The braids stay put while the cord swings, so they separate from it.
