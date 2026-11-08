# fixes-G: placement bugs logged in b21, b22, b23, b24

Every change is inside the named placer. No primitive or `pg(...)` wrapper was touched. Random draws (count and order), probability gates, materials, animators and tracked positions are unchanged; where a draw's use moved (hat angle, nest tilt, bud turn, leaf turn) it is still drawn at the same point. Checked on real levels with the batches' parts applied, and in narrow-fov close-ups.

`+` marks an extra bug found in the same placer while fixing a logged one.

## b21

- place_victorian_hall_stands (mirror): FIXED. The frame torus and the glass are no longer rotated: both stand upright in the mirror group's XY plane, whose +Z faces radially out, so the glass faces out with its frame. The group moved from r 0.055 to 0.033, so the frame's back touches the post.
- place_victorian_hall_stands (umbrella crooks): FIXED. Each crook is an upright half-ring turned `rotation.y = π − uAngle`. It is placed from the shaft's real top (the shaft's +Y end through its Euler), so the crook's thick stick end sits on the shaft and the crook curls radially out.
- place_victorian_hall_stands (hats): FIXED. The hat angle snaps to the nearest upper hook (same random draw). The brim rests on the peg (arm top, y 0.020 + hookYs[0] + 0.004) with the brass knob inside the crown. The top-hat crown stands on its brim; the bowler dome and brim sit at brim height.
- place_servant_hall_bell_racks (bell reads as a cup): SKIPPED (handled by model). The part is a mouth-down bell in the same bounds.
- place_servant_hall_bell_racks (springs float 1.5 cm over the caps): FIXED. The six 60° arcs now overlap into one 210° curl across the board (YZ plane: `rotation.y −π/2`, `rotation.z −π/3 + s·π/6`, centre y 0.054, z 0.0195). The curl runs from the board face at the top round to the bell cap. + The bells and caps moved from z 0.020 to 0.025; each bell's back was 4 mm inside the board.
- place_servant_hall_bell_racks (trims flush, 9 mm into the wall): FIXED. z −0.005 → +0.004. The backs are on the wall and the fronts stand 9 mm proud of the board.
- place_cast_iron_door_stops (dog legs buried): SKIPPED (handled by model). The body part is modelled lying down with its front paws.
- place_cast_iron_door_stops (boot laces on the +Z side): FIXED. The laces now cross the shaft's toe-side (+X) face: position (0.0095, y, 0), `rotation.x π/2`.
- place_cast_iron_door_stops (toe 4 mm into the base): FIXED. y 0.020 → 0.0245, so the toe rests on the plate.
- place_correspondence_cards / place_empty_painting_outlines / place_heraldic_crests / place_ornate_wall_mirrors / place_small_pocket_watches: no problem logged.
- place_chess_side_stands (board 5 mm into the top): FIXED. `boardTop` is the stand top's surface, so the board sits at boardTop + 0.005 and the squares at + 0.0105.
- place_chess_side_stands (pieces float 4 mm): FIXED. Pieces stand at boardTop + 0.0106, on the squares.
- place_chess_side_stands (ashtray floats 6 mm): FIXED. The tray is on the top (boardTop + 0.002). The cigar lies across it in the rim notches (+0.0075) with the ember at its lit end, and all three moved 7 mm inward.
  - + The snifter was lowered 6.5 mm onto the top.
  - + The top is scaled 1.15 in x/z (r 0.2185): the board's corners (r 0.212), the snifter and the tray all overhung the round top.
  - + The captured pawns lay off the top's edge; they now lie in a row along the board's +X edge (`rotation.x π/2`).
- place_chess_side_stands (pedestal 10 cm short): FIXED. Scaled ×1.21 in y, so it runs from y 0.05 up to the top's underside.
- place_chess_side_stands (tripod legs splay the wrong way): FIXED. Each leg is set by quaternion: the foot end sits on its pad at r 0.12 and the top leans 0.6 rad inward to the pedestal's foot.
- place_miniature_portrait_clusters (frames stick out as horizontal hoops): FIXED. `rotation.x` removed: the ovals stand upright in the wall plane around the portraits, z 0.008 → 0.002, backs on the wall. The frame-tilt animator now tilts an oval in its own plane. + The hanging loop, also laid flat, now stands up with its ends in the frame's top.
- place_peacock_feather_vases (middle trim buried in the hip): FIXED. The hip ring is scaled to r 0.0765, onto the 0.075 hip.
- place_peacock_feather_vases (upper ring floated on the old frustums): SKIPPED (handled by model).
- place_bellows_cameras (tripod legs splay the wrong way): FIXED. Legs are set by quaternion from the feet (r 0.20), leaning 0.16 rad inward to meet in the hub. The two braces moved to the height where the legs are 14 cm apart, so each spans leg to leg.
- place_bellows_cameras (ridges stand as hoops across the bellows): FIXED. Rotations removed: the rings go round the bellows' Z axis, scaled with its taper (0.055 → 0.047).
- place_bellows_cameras (ground glass z-fought): SKIPPED (handled by model).
- place_bellows_cameras (cloth top floats 6 mm, flaps 2 cm off): FIXED. The top rests on the rear plate (camY + 0.074, scaled 0.88 wide). The side flaps hang against the plate's sides (x ±0.084), and the back flap hangs from the top's rear edge (z −0.116). + The thumbscrew moved 2 mm down onto the plate.
- place_bellows_cameras (flash rod hangs in mid air): FIXED. The rod is stretched from a clamp on the +X tripod leg (y 0.987) up to the T-bar. The T-bar, pan and powder moved to z 0, above it.
- place_brass_sconces (arm pieces float, parallel to the wall): FIXED. The two arm segments are stretched between points: plate boss → knop → under the drip cup. The knop sits at the joint.
- place_brass_sconces (candle foot 3 mm above the cup): FIXED. The candle, wick, flame and light moved down 5 mm (foot 2 mm into the socket). The rim ring moved 2 mm down onto the cup's lip.
- place_brass_sconces (chain hangs from nothing): FIXED. The three links hang from the knop, upright, each turned 90° to the last.
- place_open_journals (quill runs into the floor, vane off the shaft): FIXED. The shaft lies flat on the pages (`rotation (0, −π/6, −π/2)`, y 0.0065) along the vane's direction, and the vane covers its upper half.
- place_parchment_scrolls (ribbon hovers 3 mm): FIXED. The ribbon is scaled to r 0.021 across its axis.

## b22

- place_small_compasses (cover cuts through the case): FIXED. The hinge barrel runs along Z at the case's top edge (0.039, 0.011). The cover is the closed lid swung 120° about it: centre (0.062, 0.043), `rotation.z −2π/3`, dished face toward the dial.
- place_small_telescopes (eyepiece at the wide end): SKIPPED. The note calls it harmless.
- place_small_globes (meridian spins in its own plane): FIXED. The sphere now sits in a group tilted `rotation.z 0.4` with the meridian, so its poles meet the pins. The spin animator still turns it about its own, now tilted, axis.
- place_small_phonographs (horn upside down): FIXED. The horn is set by quaternion: throat (apex) at the arm pivot on the cabinet's back corner, mouth up and back.
- place_small_phonographs (tone arm upright, buried): FIXED. The arm is stretched from its pivot on the cabinet top (0.076, 0.064, −0.068), low over the record, to the sound box.
- place_small_phonographs (sound box 4 cm from the arm): FIXED. The arm's tip now ends on the box.
- place_hanging_brass_lanterns (panel corners outside the posts): FIXED. The panels sit 0.028/√2 from the axis, as the sides of the square whose corners are the posts.
- place_hanging_brass_lanterns (chain stops 2 cm below the ceiling): FIXED. The chain is stretched 2.2 cm; its top is 2 mm into the ceiling.
- place_small_telegraph_keys (wire upright, half in the floor): FIXED. The wire is stretched from between the terminals, over the base's end, down to the floor.
- place_small_scales (chains stop 11 mm above the pans): FIXED. Each chain run is one straight line (z 0) from inside the beam end down into its pan. The pans hang 1.1 cm higher (y −0.087).
- place_small_scales (pans clip the base): FIXED. With the same change the pans clear the base by about 1.3 cm, even at the ±0.05 sway.
- place_small_abacuses (end beads 3 mm into the stiles): FIXED. The bead pitch was recomputed, so the end beads stop 0.5 mm short of the stiles. The bar-chamfer remark is a modelling note only.
- place_small_trophies (handles float beside the cup): FIXED. Each handle is a C in the XY plane (`rotation.z −s·(π/2 + 0.395)`) with both ends on the cup wall at x ±0.0365. The chord leans with the cup's flare.
- place_small_handbells (bell opens upward like a bowl): FIXED. `rotation.x π` removed, because the upper hemisphere already opens downward. The dome sits at y 0.005: mouth down, crown under the collar and handle, clapper inside. The lying variant rests on its rim.
- place_small_masks (strap across the mouth): FIXED. The strap moved to (0, 0.008, −0.01), under the face-up plate at eye level; its ends show at the temples.
- place_candle_clusters: no problem logged.
- place_small_crystal_balls (prongs twist like a pinwheel): FIXED. `rotation.set(0, π − a, 0.4, 'YXZ')`: each prong leans radially out, its hooked tip curling toward the ball.
- place_small_pumpkins (face decals inside the body): FIXED. Each decal sits 1 mm outside the body ellipsoid at its (x, y) and faces along the surface normal.
- place_small_witch_hats (crown pokes out of the band): FIXED. The crown now tilts about its base, not its middle: position (−sin·0.07, 0.006 + cos·0.07).
- place_small_skeleton_hands (thumb zig-zags): FIXED. The two thumb bones lie flat, end to end along (−sin 0.9, 0, cos 0.9) from the palm's side.
- place_small_cauldrons (round bottom 3.7 cm below the leg tips): FIXED. The pot (body, bottom, rim, brew, glow) is lifted 4 cm, so it stands on its legs, 3 mm clear of the floor.
- place_small_cauldrons (bail laid flat): FIXED. Rotations removed: an upright arc over the pot, with its eye ends in the rim at x ±0.055.
- place_small_tombstones (half-disc stands as a D): FIXED. `rotation.y π/2` comes before the x turn, so the half-disc spans the full width as an arch.
- place_hanging_bats / place_small_ghost_sheets / place_small_potions / place_skull_stacks: no problem logged.
- place_small_spiders (legs mirrored by position only): FIXED. Each leg is built from points: hip under the body, knee 20° up and out, foot on the floor. The direction is mirrored per side.
- place_small_spiders (feet 3 mm above the floor): FIXED. With the same change the feet end at y 0.001.
- place_small_brooms (twigs fan vertically, 8 cm into the floor): FIXED. The stick lies on the floor at the handle end and is raised to 0.020 at the bristles. The twigs run by quaternion from roots in the binding to tips on the floor, in a flat ±0.45 rad fan (random draws unchanged).
- place_small_black_cats (ears buried): FIXED. The ear bases sit 2 mm into the head, 0.45 rad off the crown, leaning 0.3 rad out; the whole ear shows.
- place_small_treat_pails (face decals inside the body): FIXED. Same as the pumpkins: 1 mm outside the body ellipsoid, facing its normal.
- place_small_treat_pails (handle flat above the rim): FIXED. Rotations removed: an upright arch, narrowed ×0.945 so its pivot ends meet the rim's sides at mid-height.
- place_small_treat_pails (stem in the candy opening): SKIPPED (handled by model). The rim part has a centre boss under the stem.
- place_small_skulls (teeth inside the jaw front): FIXED. The teeth sit on the jaw's top edge (y 0.0225), following its horseshoe front (z 0.024 at the chin curving back), each turned to face out.
- place_small_owls (tufts buried): FIXED. The tuft bases sit 2 mm into the head, 0.45 rad off the crown, leaning 0.35 rad out.
- place_small_owls (wing triangles inside the body): SKIPPED. These are single-sided buffer geometry triangles, one facing +Z and one −Z. The body part already models folded wings, and pulling the triangles out would show flat one-sided shards.
- place_small_owls (eye rings overhang the head): SKIPPED. Minor: flat decals on the part's flat facial disc.
- place_brass_candlesticks (wax pool hovers 4–5 mm): FIXED. The pool sits on the domed top (y 0.0097), scaled 0.81 to stay inside the cove. + It faced down (the fan winds clockwise seen from above), so it never showed from above; it is flipped with `rotation.x π`.

## b23

- place_wall_crucifixes / place_stained_glass_windows / place_crystal_skulls / place_ouija_boards / place_tarot_card_fans / place_pentagram_chalks / place_potted_plants / place_flower_stems / place_leaf_litter / place_twigs / place_pine_cones / place_feathers: no problem logged.
- place_reliquary_boxes (medallion half buried): SKIPPED. The note says it is unchanged from before; it reads as inset.
- place_small_candles (drip half inside): SKIPPED. The note says placement is OK.
- place_hanging_censers (chains stop 2 cm below the ceiling): FIXED. The top anchors are 2 mm into the ceiling.
- place_hanging_censers (chain ends 3–4 mm off the lid): FIXED. The bottom anchors are on the rim bead (r 0.035, bowlY + 0.002).
- place_religious_triptychs (canvas covers the dividers): FIXED. The canvas is recessed 8 mm behind the frame face (z t/2 − 0.008), so the frame and the two dividers stand proud of the panels.
- place_memorial_shrines (holders buried in the step): FIXED. The holders and candles moved to z 0.068, on the platform in front of the step.
- place_memorial_shrines (portrait 2 mm in front of the frame): FIXED. The frame, portrait, silhouette and plate share one leaning group. The portrait lies in the frame's sunk centre (+0.001), the silhouette on it, and the plate on the bottom moulding.
- place_memorial_shrines (cross floats 2.5 mm, no foot): FIXED. The cross is lowered 2.5 mm onto the platform. A foot would need a new part. + The Star of David is lowered 1.3 cm; it floated.
- place_memorial_shrines (buds off their stems): FIXED. Each stem's foot is inside the vase (platH + 0.030), leaning out of the mouth, and the bud sits on the stem's tip along its axis. The old bud-z random draw now turns the bud.
- place_memorial_shrines (ribbon tail in the platform and the floor): FIXED. The tail continues the lying ribbon over the platform's right edge and slants 26° down to the floor. The sway animator still moves it ±3 mm.
- place_dress_forms (legs splay the wrong way): FIXED. Legs run by quaternion from the feet (r 0.19), leaning 0.548 rad inward into the hub, stretched 10 %.
- place_dress_forms (pole wheel vertical): FIXED. `rotation.x π/2`: horizontal round the pole, like the neck wheel.
- place_dress_forms (torso gaps): FIXED. The waist is scaled y 0.35 → 0.72 (at +0.199) and the bust 0.50 → 0.577 (at +0.35), so hip, waist, bust and shoulder meet.
- place_dress_forms (drape floats in front of the bust): FIXED. The drape stands against the bust's front (`rotation (π/2, 0, 0.15, 'ZXY')`, z 0.113), narrowed ×0.75. The flowing strip hangs from its right edge, and the pin heads sit on its face. The sway animator is unchanged.
- place_dress_forms (seam strips straight, buried/outside): SKIPPED. A straight 2 mm strip can't follow the torso profile; this needs a curved seam part.
- place_skull_plaques (oval plaque round, 14.7 mm thick): FIXED. `scale.y` → `scale.z`, the rotated cylinder's axis that is vertical in the wall plane: now 22 cm tall and 12 mm thick.
- place_skull_plaques (raven beak sideways, detached): FIXED. `rotation.y −π/2` added: the apex points out of the plaque, 0.1 rad down, with the hook down, and the base is in the cranium's front. The seam is stretched along the underside from base to tip.
- place_skull_plaques (antlers lean in, forks detached): FIXED. The beams lean out (a V) from bases 2 mm into the cranium top. The forks are tines leaving the beam three-quarters of the way up, pointing up and forward.
- place_skull_plaques (eye sockets inside the crania, facing +X): FIXED. Each socket sits 0.5 mm outside its cranium ellipsoid on the side, facing the normal. The raven's sockets moved to its braincase (z 0.030).
- place_small_candelabras (arms nearly vertical, side cups float): FIXED. `rotation.z = atan2(dy, dx) − π/2`, so each arm runs from the column to its cup. + The centre cup moved down onto the column top (0.157); it floated 6 mm.
- place_small_chain_links (chain can't lie flat): FIXED. The on-edge links stand on the floor (centre R + r) and the flat links rest across their lower wires (y 0.0155), at a 1.25 R pitch, so every link threads its neighbours and nothing goes into the floor.
- place_small_crows (eyes buried): FIXED. The eyes sit on the head's surface, 18 mm out along (0.45, 0.25, ±0.86).
- place_small_crows (wing triangles inside the body): SKIPPED. These are single-sided buffer geometry triangles; the −Z wing faces inward, so it would not show if pulled out. The body part models the wing bulges.
- place_small_hourglasses (top bulb not flipped, sand floats): FIXED.
  - The top bulb is flipped, and both bulbs are stretched ×1.233 to meet apex to apex at the waist (0.080) and reach the plates.
  - The upper sand is turned point-down into the upper funnel (narrowed ×0.6 to stay inside the glass).
  - The lower sand is a heap on the bottom plate; it was inverted.
  - The stream runs from the waist to the heap's crest (×1.53), and the animator's base height is updated.
- place_dead_plants (stems tilt tangentially): FIXED. Stems are set by quaternion, tilted radially outward.
- place_dead_plants (stems start above the soil): FIXED. Feet are 1–3 cm down in the soil (same random draw).
- place_dead_plants (tip leaves float): FIXED. Each leaf sits on its stem tip with the stem's orientation. The old independent-height draw now turns the leaf about the stem.
- place_dead_flower_vases (stems hover over the neck): FIXED. The stems start at y 0.080, 1 cm down inside the neck; the petals follow.
- place_dirt_clods / place_pebbles / place_mushrooms (items interpenetrate): SKIPPED. This is random scatter, not a part-to-part bug.
- place_acorns (stalk inside the cap, nut sinks 2 mm): SKIPPED. The cap part's stub stands in for the stalk, and sinking is handled centrally.

## b24

- place_dandelions (wisps don't radiate, not attached): FIXED. Wisps are set by quaternion from the puff's centre, achene 1 mm inside the puff, at elevations −0.3 to 0.6 rad.
- place_moss_patches / place_pine_needles / place_gardening_gloves / place_soil_bags / place_garden_aprons / place_fertilizer_spikes / place_dead_potted_plants: no problem logged.
- place_grass_clumps / place_butterfly_wings / place_spider_webs / place_seed_packets: untouched by the modeller.
- place_ferns (stem taper reversed): SKIPPED (handled by model). The part is thick at the frond's base inside the same frame.
- place_snail_shells (lip flat, half buried): FIXED. The lip is an upright C (`rotation (−0.52, −π/2, −π/2)`, scale 0.83) bulging out of the body whorl. Its lower end is in that whorl and its upper end leans against the third whorl.
- place_bird_nests (twigs alternate tangent/radial): FIXED. `rotation.set(0, π/2 − a, π/2 + tilt, 'YZX')` lays every twig tangent to its ring. The old random x-wobble is now an up/down weave along the ring.
- place_bird_nests (eggs hover 1.1 cm): FIXED. y 0.024 → 0.013, resting in the hollow.
- place_chestnuts (spikes don't radiate): FIXED. Spikes run along the nut ellipsoid's normal from 1 mm inside it, at two alternating latitudes.
- place_hanging_plants (chain ends 6 mm above the pot): FIXED. The chain is stretched ×1.17, from the bracket eye into the pot.
- place_seedlings (dirt 1 mm above the rim): SKIPPED (handled by model). The soil part is lowered at its edge.
- place_trowels (scoop on edge, 1 cm into the floor): FIXED. `rotation (π, 0, π/2)`: the hollow faces up, the width runs across, and the belly rests on the floor (y 0.020).
- place_trowels (tip cone points back): SKIPPED (handled by model). + The round tip reached 5 mm under the floor: it is flattened (×0.35) and lies on the floor, continuing the scoop.
- place_watering_cans (spout rotated the wrong way): FIXED. `rotation.z +π/3`: the narrow end points out and up, and the flanged end is in the body.
- place_watering_cans (rose floats): FIXED. The rose sits on the spout's end with its face out along the spout. The hole dots are on that face, facing the same way.
- place_watering_cans (handle across the can): FIXED. The handle is upright in the spout's plane, with both ends set into the can's top.
- place_pruning_shears (tips swapped vertically; blades flat): FIXED. The shears lie flat, sprung open about a vertical pivot bolt (scaled 0.75). The blade and hook open ±0.15 rad in the floor plane from their roots at the bolt. The tips sit on the blade ends, turned with them and flattened ×0.45 to the blade's thickness.
- place_pruning_shears (tip cones point back): SKIPPED (handled by model).
- place_pruning_shears (handles don't join the blades): FIXED. The handles start at the bolt and open ±0.2 rad.
- place_garden_hoes (head 11 mm beyond the collar): FIXED. The head moved to x −0.282 (1 mm into the collar) and lies on the floor (y 0.003).
- place_garden_rakes (bar 7.5 mm beyond the collar): FIXED. x −0.307, against the collar.
- place_garden_rakes (tines in front of the bar): FIXED. The tine roots are set 1 mm into the bar's top.
- place_garden_rakes (tines half up, half in the floor): FIXED. The rake lies on its back: the tines rise from the bar, 0.3 rad forward (`rotation.z 0.3 − π`).
- place_garden_hoses (four separate rings, not a coil): FIXED as far as closed tori allow. All four turns lie flat on the floor, pressed against each other (1.4 cm pitch for a 1.6 cm hose), with their stripes turned, so they read as one flat coil. A true spiral would need a new part.
- place_garden_hoses (nozzle across the outer loop): FIXED. The nozzle runs along +Z, tangent to the outer turn, with its coupling on the hose; the spray tip is on its front.
- place_garden_shovels (blade 1 cm beyond the collar): FIXED. x −0.308, lying on the floor (y 0.004).
- place_garden_shovels (tip pyramid 3 cm under the floor): FIXED. The apex points outward (−X) and the pyramid is squashed (`scale.z 0.12`) into a flat triangular point in the blade plane.
- place_garden_shovels (D-grip upright, detached): FIXED. The D lies in the blade plane (`rotation (−π/2, −π/2, 0, 'YXZ')`). Its chord crosses the handle's end, narrowed ×0.6 so its ends meet the shaft, and the grip bulges past. The two side bars from the comment would need a new part.
- place_kneeling_pads (cutout z-fight): SKIPPED (handled by model).
- place_twine_balls (end sticks straight out, hovers 9 mm): FIXED. The end leaves the ball's lower flank and slopes down to its frayed end on the floor.
- place_seed_catalogs (page 2 mm in front of the frame): FIXED. z 0.008 → 0.0025, on the recessed backing board under the sight edge.
- place_pressed_flowers (paper 1 mm in front of the frame): FIXED. z 0.010 → 0.002, on the mount at the bottom of the well.
- place_pressed_flowers (glass 4 mm in front): FIXED. z 0.013 → 0.0058, inside the box behind the sight lip.
- place_bird_feeders (loop flat, 8 mm over the cap): FIXED. The loop stands upright, its ends 4 mm into the cap's hanger boss.
- place_botanicals (paper 2 mm in front of the frame): FIXED. z 0.009 → 0.0015, on the backing board.
- place_garden_gnomes (nose and beard on opposite sides): FIXED. The beard moved to the face side (−Z), with the nose.
- place_garden_gnomes (beard inside the head and body): FIXED. The beard hangs from the chin, its top under the nose, down over the chest, tipped 0.2 rad forward (z −0.026).
- place_garden_gnomes (boots under the robe): FIXED. The boots moved to z −0.016, x ±0.009, so the toes peek out from under the hem.
- place_herb_bundles (hook 3 cm above the binding): FIXED. The hook moved down to the twine binding, with its rosette on the wall (it was also 9 mm off the wall).

## Count

127 lines for logged bugs, plus 6 grouped "no problem logged / untouched" lines:

- **108 FIXED** (b21: 23, b22: 30, b23: 24, b24: 31). Lines marked `+` also fix an extra bug in the same placer.
- **19 SKIPPED:**
  - 11 are already handled by the new parts or centrally: bell shape, dog legs, peacock upper ring, bellows ground glass, treat-pail stem, fern taper, seedling dirt, trowel tip, shears tips, kneeling-pad cutout, and the acorn stalk (its nut sinking is handled centrally).
  - 5 are not bugs or are harmless: telescope eyepiece, owl eye rings, reliquary medallion, candle drip, and the clods/pebbles/mushrooms scatter.
  - 2 are the one-sided buffer geometry wing triangles (owl, crow). The body parts already model the wings, and pulling the triangles out would show flat one-sided shards.
  - 1 would need a new model: the dress-form seams.
- A foot for the shrine cross, a true hose spiral and the shovel's D-grip side bars would also need new parts; this is noted in their FIXED lines.
