# b23 notes (occult, nature)

91 parts, 22 266 triangles, 0 errors, 0 warnings. Every part is within its budget. The largest frame deviation is 10 % (the bust, the dress-form wheel and the fallen leaf's thin axis).
The parts wrap 93 primitives in 25 of the 29 placers.
Shared parts: `DressForms__wheel` is used by both brass wheels (pole and neck). `SmallHourglasses__sand` is used by both sand cones. The two identical hourglass plate primitives are told apart by occurrence, `occ` (0 = top plate, 1 = bottom plate).
Checked part by part, in the game on level 4 seed 7 and level 2 seed 3, in narrow-fov close-ups and in assemblies of the placers' layouts.
Left everywhere: transparent or basic material glass, crystal, gems, flames, halos and wisps. Also left: DoubleSide cards, petals and leaves, decals and flat faces (Plane/Circle), and custom buffer geometry triangles. Whole objects that sink or hover are left to the central placement pass. Only part-to-part problems are logged below.

## Per placer

- **place_wall_crucifixes**: `upright`, `crossbar` (carved bars: square sides, a hipped ridge roof, so the two ridges mitre into an X where they cross; eased edges). Left: nothing. Placement OK.
- **place_stained_glass_windows**: `bar` (hammered iron bar, crowned towards all four edges, because one constructor makes both the rails and the stiles). Left: glass (DoubleSide basic-material plane). Placement OK.
- **place_crystal_skulls**: untouched. Every mesh is transparent basic material (cranium, jaw, sockets).
- **place_reliquary_boxes**: `body` (chest: eased edges, a lid seam under the top band, round-arched sunk niches (3 per long side, 2 per end), a sunk lid panel), `trim` (gilt band: a half-round bead between fillets, used for both bands). Left: the 5 gems (transparent). Placement OK. The centre medallion is half buried in the lid, as before.
- **place_hanging_censers**: `chain` (30 long oval links turned 90° one to the next), `bowl` (gadrooned fire bowl on a stem and spreading foot, closed under the lid), `rim` (flat-topped flange ring), `lid` (flange, ribbed first tier, stepped second tier, cupola), `knob` (collared ball finial with a pip). Placement:
  - The chains' top ends stop 2 cm below WALL_H, with no hook or ceiling rose.
  - The chains' bottom ends (r 0.03, 1 cm above the rim) hang about 3–4 mm clear of the lid's side.
- **place_religious_triptychs**: `bar` (gilt cushion moulding with a sunk bead line, symmetric for rails and stiles), `divider` (half-round colonnette with a square base and capital). Left: canvas (textured DoubleSide plane). Placement: the canvas (z = t/2 + 0.001) covers the whole w × h opening 1 mm in front of the dividers, so the dividers are never seen.
- **place_memorial_shrines**: `platform`, `step` (boards with rounded-over top edges), `frame` (bead-and-cove photo frame round a sunk centre), `plate` (chamfered gilt label), `holder` (drip dish with socket), `candle` (taper: softened top, melt crater with a wick mound, one drip run), `vase` (pewter bud vase with a foot ring), `mouth` (hollow lip ring), `stem` (dry kinked stalk with nodes), `bud` (dried five-lobed rosebud), `crossV`/`crossH` (carved ridge bars like the wall crucifix), `ribbon` (black ribbon lying flat, folded back over itself), `tail` (hanging ribbon end, wavy, swallowtail cut).
  - Left: the portrait face (flat plane), the silhouette (circle), wicks (1.6 mm, tiny), flames (basic material), and the Star of David triangles. A 3-sided cylinder's fitted bounds are its full circle, so an equilateral triangle would be stretched and shifted. I kept the crisp primitive.
  - Placement:
    - The brass holders (y 0.020–0.026) are completely buried in the step (z −0.09…0.05, top 0.035). The candles stand at z 0.03, so their lower 9 mm is inside the step too.
    - The portrait face plane floats 2 mm in front of its frame (frame front +0.004, face +0.006).
    - The standing cross floats 2.5 mm above the platform (bottom 0.0225 against 0.020) and has no foot.
    - The dried buds are off their stems. The stem tips are at x ≈ −0.0175·sin(lean), the buds at +0.04·sin(lean), up to 1.4 cm apart.
    - The hanging ribbon tail (rV) sits mostly inside the platform edge (centre 7 mm in) and reaches 4 cm below the floor (y −0.04…0.01).
- **place_dress_forms**: `leg` (turned tripod leg with foot and top rings), `foot` (domed pad), `hub` (turned hub with collars), `pole` (column with end ferrule rings), `wheel` (knurled brass hand wheel, both wheels), `hip` (flat torso base, full hips drawing in), `waist` (cinched band), `bust` (under-bust drawn in, two forward lobes), `shoulder` (sloping yoke, narrower front to back), `collar` (neck with flared base and capped top), `cushion` (8-segment tomato pincushion), `drape` (cloth sheet with loose folds, free edges falling), `flow` (hanging cloth, folds opening towards an uneven hem). The torso pieces are modelled in the spheres' unscaled frames against the placer's (1, s, 0.85) scales.
  - Left: the seam strips (2 × 1 mm), pins, pin beads and pin marks (under 2 mm).
  - Placement:
    - The legs splay the wrong way. rotation.x/z tip each leg's top outwards (top at r ≈ 0.175, bottom at r ≈ 0.045), so the three legs cross under the hub, and the feet (r 0.19) stand apart from them.
    - The pole wheel has rotation.y = π/2, which stands the ring vertical with the pole running through its rim. It should be rotation.x = π/2, horizontal round the pole, as the neck wheel is.
    - The torso has gaps: hip top 0.123 to waist bottom 0.145, waist top 0.215 to bust bottom 0.275, bust top 0.405 to shoulder bottom 0.43. The parts reach past their spheres within tolerance and close these to about 0.9, 4.1 and 1.3 cm. The waist-to-bust gap stays visible.
    - The drape is a flat sheet at one height, standing 2.5–17.5 cm out in front of the bust instead of lying over it.
    - The seam strips are straight at r 0.115. They are buried in the hip and bust, stand outside the waist and cross the gaps.
- **place_skull_plaques**:
  - Parts: `shieldTop` + `shieldPoint` (one shield outline with an ogee edge, cut at the joint, so the two boxes read as one shield with a gentle crest and a curved point), `oval` (round plaque, ogee face towards +Y, i.e. the room), `idPlate` (brass label with two screws), `foxCranium` (sagittal crest, pinched behind deep orbits, cheekbones), `foxSnout`/`deerFace` (bony muzzle: flat palate, sloping nasal line, nasal opening), `teeth` (scalloped incisor row), `canine` (curved fang), `deerCranium` (vault, pedicle bosses, orbit rims round deep sockets), `antler` (burr, pearled beam), `fork` (tine), `ravenCranium` (round braincase, big orbits, drawn to the beak root), `ravenBeak` (long, taller than wide, slight hook).
  - Left: eye-socket circles (flat decals), the beak seam (1 mm bar).
  - Placement:
    - The "oval" plaque is round. scale.y acts on the rotated cylinder's axis, so it thickens the plaque to 14.7 mm instead of making it taller.
    - The raven beak lies sideways along the wall. rotation.z = −π/2 turns its apex to +X, and it is centred 2.5 cm in front of the cranium, detached (the seam bar likewise).
    - The deer antler stubs lean inwards (rotation.z = side·0.4 tips their tops towards the centre, a Λ). The forks sit about 1.7 cm outboard of the stub tips, also leaning in, detached.
    - The fox and deer eye-socket circles are 4–8 mm inside the crania, and all of them face +X, so they barely show.
- **place_small_candelabras**: `base` (stepped domed foot with a bead), `column` (turned stem with collars at each knop and a top cup), `knop` (cushion knop with a sharp waist rib), `arm` (turned arm with a mid knop and end collars), `cup` (bobeche: dished pan on a socket), `candle` (taper with crater, wick mound, drip). Left: wicks (tiny), flames and halos (transparent). Placement: the arms' rotation.z = −atan2(0.01, ±0.06) leaves them nearly vertical, as 6 cm rods standing beside the column at x ±0.03. The side cups and candles at x ±0.06 float with nothing holding them. The arm would need rotation.z = atan2(dy, dx) − π/2 to reach them.
- **place_small_chain_links**: `link` (forged link: slightly oval within the round frame, flattened section, weld swelling, hammered). Placement: the flat links (rotation.x) sit with their undersides 9 mm above the floor. rotation.z = π/2 leaves the other links upright in the XY plane (they do interlock), reaching 1.5 cm below the floor. The chain cannot lie flat as placed.
- **place_small_crows**: `body` (full breast, back narrowing to the tail root, wing bulges on the flanks, flatter belly; built against the 1.4 x-scale), `head` (flat crown, steep forehead, shaggy throat, cheeks drawn in), `beak` (deep, arched, laterally pinched bill), `leg` (tarsus with a knee and foot swelling, 84 tris). Left: wings and tail (buffer geometry triangles), eyes (basic material). Placement:
  - The eyes (centre 12 mm from the head centre, head r 20 mm) are buried in the head and never show.
  - The flat wing triangles are mostly inside the body, with only their top corners out.
- **place_small_hourglasses**: `topPlate` (occ 0, glass seat facing down), `botPlate` (occ 1, facing up; turned discs with a bead and step), `post` (bead-and-reel spindle), `sand` (heap with hollow flanks, soft crest and grainy surface, both sand cones). Left: glass bulbs (transparent), sand stream (1 mm, animated). Placement: the "upside-down" top bulb is not flipped. The two glass cones meet at their wide ends at y 0.07, so the glass is a diamond, not an hourglass. The upper sand (from 0.0895) floats 2 cm above that waist.
- **place_ouija_boards**: `planchette` (heart/teardrop body, pointer on the frame's +Z corner, domed top, round window near the tip, three felt pegs). Left: board (textured DoubleSide plane). Placement OK.
- **place_tarot_card_fans**: untouched. Every card is a textured DoubleSide plane.
- **place_pentagram_chalks**: untouched. Its only mesh is a transparent basic-material decal.
- **place_small_candles**: `body` (burned pillar: crater inside an uneven rim, wick mound, two wax runs; the part also serves the clones), `drip` (flattened run ending in a bead). Left: wick (2 mm), wisp and flame (transparent), wax pool (decal). Placement OK. The drip's centre is 2 mm inside the candle, so only half of it shows.
- **place_dead_plants**: `pot` (terracotta: thick rolled collar, tapered body, bevelled foot, soil level sunk inside the collar, lightly lumpy), `stem` (dry stalk with swollen nodes and kinks), `leaf` (withered leaf: curled edges, sunk midrib, crinkles), `fallenLeaf` (plane part: flat dead leaf with a stalk, front at z = 0, 1.3 mm of curl behind). Left: soil disc (flat circle). Placement:
  - The stems tilt tangentially round the pot, not outwards. The tilt is (−sin, cos)·droop against the radial (cos, sin).
  - The stem bottoms land at y 0.17–0.27, so most dead stems start above the soil (0.202), by up to 6.6 cm.
  - The stem-tip leaves are placed with the opposite x sign and an independent random height, so they float up to about 25 cm from the stem tips (visible in the game as loose brown dots).
- **place_dead_flower_vases**: `body` (belly on a foot ring, shoulder curving in to meet the neck), `neck` (hollow waisted neck with a flared lip), `stem` (thin dry stalk with nodes). Left: petals (DoubleSide circles). Placement: the stems start at y 0.100, about 1 cm above the neck mouth (0.090), so they hover over the vase.
- **place_potted_plants**: `pot` (small rolled-collar pot, soil level sunk). Left: dirt (flat circle), leaves (buffer geometry, DoubleSide). Placement OK.
- **place_flower_stems**: `stem` (cut stem with nodes, curving gently along the floor), `leaf` (plane part: lance leaf with a sunk midrib, slightly lifted edges). Left: petals (DoubleSide), centre dot (flat circle). Placement OK.
- **place_dirt_clods**: `clod` (crumbly, lumpy soil lump replacing the octahedron). Placement OK. The clods scatter with no spacing and can interpenetrate.
- **place_pebbles**: `pebble` (water-worn, oblong, one side flatter, smooth). Placement OK. They can interpenetrate.
- **place_leaf_litter**: untouched. Every leaf is a DoubleSide circle.
- **place_twigs**: `twig` (tapering, bent, bark knob, snapped side stub, both ends broken). Placement OK.
- **place_acorns**: `nut` (full shoulders, drawn to a point with a nipple; built against the 1.3 y-scale), `cap` (cupule: rows of scales, thick rim, hollow underneath, a stalk stub on top). Left: the stalk (1 mm cylinder). Placement: the stalk (y 0.0215–0.0265) is completely inside the cap (top 0.031). The cap part's own stub stands in for it. The nut sinks 2 mm into the floor.
- **place_pine_cones**: `core` (cone body: fat end at the frame's +Y, stalk there, tip closed; lumpy scale pattern), `scale` (woody scale: diamond apophysis with keel, thickening outwards along the scale's local +Y). Placement OK.
- **place_feathers**: `shaft` (bare calamus at the base, rachis tapering to a fine tip, slight curve). Left: vane (DoubleSide circle). Placement OK.
- **place_mushrooms**: `stem` (swollen base, fibrous, a ragged ring under the cap), `cap` (convex cap with an in-rolled margin and radiating gill ridges underneath, 256 tris). Placement OK. The mushrooms in a cluster can interpenetrate (visible on level 2).
