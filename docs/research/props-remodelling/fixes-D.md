# fixes-D: placement bugs logged in b11, b12, b13, b14

Every change is inside the named placer, made as small edits. No primitive or `pg(...)` wrapper was touched. Random draws, probability gates, materials, animators and tracked positions are unchanged: where a tilt now feeds a height, it is the same draw stored in a variable, and PillBottles keeps its four old per-tab draws. Checked on real levels (level 4 seed 7 and level 2 seed 3), in close-ups of rare variants, and with a bounds dump or raycast per part.

`+` marks an extra bug found in the same placer while fixing a logged one.

## b11

- place_shoe_pairs (laces stand as vertical X's, front rows float 1–2 cm over the toe): FIXED. Each lace now lies flat (`rotation.z = π/2`, yaw ±0.7) as a criss-cross on the vamp, pitched 0.15 with its slope. The rows run from the throat to the toe (z 0.026–0.080) at the probed surface height (y 0.0405 falling 0.15 per metre of z).
- place_shoe_pairs (tipped right shoe dips up to 9 mm into the floor): FIXED. The tilt is drawn once into `rightTilt` (same draw) and y = `0.045·cos(tilt) + max(0, 0.066·sin(tilt))`, so the lowest edge (sole or collar side) rests on the floor at either end of the tilt.
- place_dropped_gloves (random rotation.z dips the palm/cuff edge a few mm): SKIPPED. The whole glove tilts as one object, so this is whole-object floor contact.
- place_dropped_gloves (trim floats 1 mm over the cuff): SKIPPED. A probe shows the trim is set into the cuff at the centre (trim top 0.019, cuff top 0.0176) and at most 0.5 mm off at the cuff's rounded ends. Nothing is visible.
- place_laundry_baskets (handles stand as C's bulging to -X, half below the rim): FIXED. `rotation.y = π/2` replaces `rotation.z = π/2`. Both half-rings now stand as arches on the rim, in the wall's tangent plane, with their feet in the rim.
- place_laundry_baskets (hanging drape inside the wall): FIXED. `drapeV` moved from x 0.94·bR to bR + 0.011, outside the rim. It also got `rotation.y = π/2`: it had been a radial fin through the wall, and now its face is turned out. + `drapeH` moved from x 0.6·bR (inside, 2 cm short of the rim) to bR − 0.01, y bH + 0.015, so it lies across the rim and the hanging part tucks under it.
- place_ironing_boards (nose and back-end half-discs stand up as fins): FIXED. Both rotations were removed. The θ 0..π and π..2π half-discs already lie flat on the ±X sides.
- place_ironing_boards (iron nose half-frustum vertical, through the board): FIXED. It now has only `rotation.z = π/2` and sits at (0.040, 0, 0). It lies flat, cut face down on the board, with its wide end against the housing front and its narrow end at the tip.
- place_ironing_boards (iron handle lies flat, 1.4 cm above the body): FIXED. The rotations were removed, so the arch stands front to back with its feet on the housing top (y 0.03).
- place_ironing_boards (leg tops poke 5 mm through the board): FIXED. The legs were rebuilt (same constructor) as two rails at z ±0.06, each crossing in an X seen from the side (`rotation.z = ±atan2(0.22, 0.84)`, centre (±0.03, 0.42)). The tops end at y 0.841, just into the board's underside (0.84).
- place_ironing_boards (feet at (±0.14, ±0.06) while the legs stood at z 0): FIXED. The leg feet now land exactly on the four rubber feet. The cross brace is now the pivot rod through the X crossing (`rotation.x = π/2`, `scale.y` 0.44, y 0.5345), linking the two rails. It had floated between them.
- place_ironing_boards (shirt hovers 5 mm): FIXED. y changed from boardH + 0.020 to boardH + 0.015, onto the board top. + The cord boot pointed sideways (`rotation.x`); it now points out of the iron's back (`rotation.z = π/2`), and the cord starts at its end (it had started 2.5 cm away).
- place_vacuum_cleaners (dust chamber and roller now visible through the new canister bay and nozzle mouth): SKIPPED. Not a bug: the note says the chamber reads as a dust-cup window.
- place_vacuum_cleaners (power button floats beside the wand): FIXED. The button is on the wand's +X surface (x -0.004, z -0.14, 1 mm into the tube) and tilted -0.20 like the wand.
- place_vacuum_cleaners (rear wheels hidden inside the base): FIXED. The wheels moved from x ±0.42·baseW to ±(baseW/2 + 0.005), just outside the base's side faces.
- place_brooms_and_dustpans (dustpan handle detached, below the floor): FIXED. The handle's foot now starts in the back wall's boss at (0, 0.02, -0.053). Its centre is foot + dir·L/2 along the existing -0.5 tilt.
- place_brooms_and_dustpans (hook loop 2 cm off the handle's top): FIXED. The loop sits on the handle axis just past the top end (foot + dir·(L + 0.005)).
- place_brooms_and_dustpans (lip tilts the wrong way, back edge below the floor): FIXED. The modelled lip is already a wedge, thick at the pan and feathered at the front. The -0.35 tilt was removed, `scale.y` is 0.45 and y is 0.0032: it lies on the floor and its back meets the pan floor top.
- place_brooms_and_dustpans (broom handle sticks 2 mm out of the head's top): FIXED. The handle and grip band moved from y 0.012 to 0.010, centred on the head's height, into its socket boss.
- place_packed_suitcases (non-roller variant floats 1 cm): FIXED. The corner caps moved from y 0.018 to 0.008. They are now feet under the body (bottom at y 0), so the case stands on them.
- place_packed_suitcases (roller variant: front edge floats 1 cm): FIXED. The same corner feet hold the front on the floor, level with the wheels.
- place_packed_suitcases (wheels half behind the back face): SKIPPED. Not a bug: two-wheel cases have their wheels half recessed at the back bottom edge, and that is how they read here.
- place_packed_suitcases (tag string stops 4 mm above the tag): FIXED. `scale.y` 1.125 at y 0.0175, so the string runs from the eyelet (y -0.005) to 0.04. + The tag hung from nothing on both variants, and on the non-roller one it was half buried in the lid. The roller tag now hangs from the pull bar (tagY bodyH + 0.108, so the string top is in the bar). The non-roller tag lies flat on the lid (`rotation.x = -π/2`, y bodyH + 0.011) with its string running back to the folded handle's foot.
- place_travel_documents (pen tip ignores the pen's yaw, apex points back): FIXED. The tip now uses the pen's yaw (`rotation.z = π/2`, `y = -0.4`) and sits on the pen axis 0.0535 past the centre, apex out. The barrel flipped (`rotation.z = -π/2`) so its narrow modelled writing end is the one with the tip. + The pen moved to (-0.088, 0.003, -0.06), onto the floor and clear of the passport (it had cut 3 mm into it).
- place_travel_documents (bill floats 1.5 mm above the passport): FIXED. y changed from 0.008 to 0.0068, on the cover (0.0062). The portrait moved to 0.0074.
- place_travel_documents (boarding pass hovers 7 mm beyond the passport): FIXED. The pass group moved from y 0.007 to 0, onto the floor with its end tucked under the passport's edge.
- place_glasses_cases (lid lining on the lid's outer side, poking through the shell): FIXED. The lining is now in the lid's hollow, against its ceiling (lid-local y +0.0033, centred), so it shows on the inside of the open lid.
- place_glasses_cases (lid's lower edge 1.75 cm behind the base): FIXED. The lid and its lining now hang in a `lidPivot` group on the hinge line (the base's back top edge), still opened -π/3. The hinge cylinder moved onto that line (y caseH + 0.002).
- place_glasses_cases (temples start inside the rims): FIXED. Each temple now starts at its rim's outer top corner (x ±0.026, z -0.004) and folds inward across the rims, lying on them (`rotation.z = side·π/2`, `y = side·0.2`, y caseH + 0.0095).
- place_wristwatches (hands and ticks turned tangent to the dial): FIXED. `rotation.y = π/2 - angle` (was `-angle`) for the ticks, the hour, minute and second hands, and in the ticking animator, so each points out from the pivot.
- place_wristwatches (face disc floats 1 mm above the case): FIXED. The face moved from y 0.011 to 0.0097, onto the dial (the probe found the dial at 0.0095 inside the bezel). The ticks, hands (animator too) and pivot moved down by the same 1.3 mm.
- place_wristwatches (buckle tongue stands up as a spike): FIXED. The tongue now lies flat across the buckle along the strap (`rotation.x = π/2`, `z = -0.3`, matching the buckle's 0.3 yaw), resting on the strap end at y 0.004.
- place_wristwatches (strap floats 2.5–3.5 mm): FIXED. The strap's y is `0.0015 + 0.0035·(1-t)²`, so it leaves the lug at 0.005 and lies on the floor from mid-length. The buckle moved to y 0.0025 so it does not sink.
- place_opened_wallets (pocket lips and panel float 0.25 mm): SKIPPED. A 0.25 mm gap is invisible.
- place_jewelry_boxes (lid detached: 1 cm up, 2.75 cm behind the box): FIXED. The lid and lining hang in a `lidPivot` group on the back top edge, still opened -π/3. The lining sits centred against the hollow's ceiling (lid-local y 0.0008). The hinge moved onto the pivot line.
- place_jewelry_boxes (necklace runs through the +X side wall): FIXED. The chain starts in the +X compartment (x 0.3·boxW, clear of the divider) and drapes over the rim at x = boxW/2. It then falls down the outside to the floor at x 0.85·boxW, with explicit per-point heights. The x wiggle was dropped so no point doubles back into the wall; the z wiggle is kept.
- place_jewelry_boxes (earring stones sink 1 mm): FIXED. The stones moved to y 0.003, their radius. + The hoops hovered up to 2 mm. Their tilt is drawn once (same draw) and y = `0.005·|sin(tilt)| + 0.0008`, so the low edge touches the floor.
- place_umbrella_stands (umbrellas stand tip-up with the handle end hidden): SKIPPED. Flipping them would show a bare shaft with no handle; a handle needs a new model. As placed they are self-consistent: the modelled canopy narrows to the ferrule at the top.
- place_umbrella_stands (fold slivers don't follow the shaft; one pokes through the wall): FIXED. The shaft's z offset had the wrong sign for Euler XYZ, so the lower end sat up to 11 cm off the stand's axis, through the wall. It now sits on the axis, and the folds follow it with their offset square to the tilt plane. + The ferrule was placed at the stand's axis, up to 10 cm from the shaft's actual top. It is now on the tip (base 2 mm into the canopy).
- place_umbrella_stands (foot ring 2.5 cm outside the stand's bottom): FIXED. The ring is scaled 0.84 in its plane (inner radius 0.0857), so it hugs the stand's foot (r 0.085).
- place_boot_trays (soles sunk 8 mm into the tray floor): FIXED. Each boot group moved up 0.008, onto the tray floor's top.
- place_boot_trays (back of the leg overhangs the heel and floats 1 cm): FIXED. The leg and its trim ring moved to z 0.02, so the leg's back (z -0.015) lines up with the foot's back over the sole's heel.

## b12

- place_hat_racks (hats hang below the pegs): FIXED. Each hat now sits on its peg. The hat is at y `mountY - 0.056` for a fedora and `mountY - 0.042` for a cap (was `mountY - 0.075`), so the knob touches the crown's inner top and the peg enters the back of the crown.
- place_hat_racks (fedora brim into the wall): FIXED. The hat moved out from z 0.045 to 0.078. The brim's back edge now clears the wall and the crown clears the rail's front face.
- place_hat_racks (cap visor stands upright in the wall plane): FIXED. The visor is turned with `rotation.y = -π/2` instead of `rotation.x = -π/2`, so the half-disc lies flat with its round edge pointing out from the dome.
- place_hat_racks (dent box floats over the crown): FIXED. The dent moved from y 0.061 to 0.057, into the crown's crease.
- place_hall_side_tables (drawer front is a free panel): FIXED. The drawer now hangs from under the top (y 0.65–0.71, was 0.64–0.70) with its back on the front legs' faces (z 0.094, was 0.12). `scale.x` 0.917 makes it span leg to leg. The knob moved with it.
- place_hall_side_tables (key bow stands on edge): FIXED. The `rotation.x = π/2` is gone, so the bow lies flat. + The bow now sits on the blade's axis, just past its back end; it had been beside the blade and did not touch it.
- place_hall_side_tables (key floats over the dished tray): FIXED. The blade and bow are now at y +0.0055 over the top (were +0.009 and +0.010), so they rest on the tray's floor.
- place_pet_bowls (name label floats outside the rim): FIXED. The tag lies flat on the food bowl's flared outer wall, facing out: y 0.032, r 0.0855, `rotation.x = π/2 + atan(0.3)`.
- place_pet_bowls (bowls stand 3 mm down inside the mat): FIXED. When the mat is drawn, everything added before it (bowls, bottoms, rims, kibble, water) moves up 3 mm onto the mat top. The label follows through `matLift`.
- place_pet_bowls (a few kibble dip into the bowl floor): SKIPPED. Not a bug: the note says it reads as a pile.
- place_spilled_backpacks (carry handle lies flat, 2 cm above the top): FIXED. `rotation.x` is now 0.15 (was π/2 + 0.15), so the arch stands up and leans with the body. It moved from y 0.205 to 0.184, so its feet are on the top.
- place_spilled_backpacks (straps stick out above the top and dip into the floor): FIXED. The straps are placed on the leaning back face, at body-local z -0.074 turned by the lean. `scale.y` 0.9 fits them to the back's height. They had been 6 mm off the back, with an 18 mm overlength.
- place_floor_kettles (lid and knob hidden inside the body): FIXED. The lid is now at y 0.1195 (was 0.097), with its edge on the body's lid seat, and the knob sits on the lid at 0.129. The lid and the 40 % brass knob are visible again.
- place_floor_kettles (handle half-torus lies flat, half sunk): FIXED. The bail now stands upright over the lid in the spout's plane (`rotation.y = -spoutAngle`), with its feet on the shoulder at y 0.101.
- place_floor_kettles (spout points sideways, tip ring detached): FIXED. The spout and its tip use `rotation.y = π - spoutAngle`, so the narrow end aims outward along (cos, sin). The tip ring sits on the spout axis over its mouth.
- place_small_padlock_keys (shackle hangs down inside the body): FIXED. The `rotation.z = π` is gone, so the U arches up from the posts. + The key's bow stood in the YZ plane, 1.1 cm into the floor, and lifting it made the whole group hover. The key now lies flat: shaft at y 0.002, bow at `rotation.x = π/2`, and the bits flat off the shaft's side. + The lock sub-group moved down 4 mm, so the body rests on the floor.
- place_small_sewing_machines (hand wheel on the column's inner face): FIXED. The wheel moved from x -0.028 to -0.093, onto the column's outer end face, and the gold spokes moved with it.
- place_small_sewing_machines (animator yaws the wheel through the column): FIXED. `wheel.rotation.order = 'ZYX'`, so the animator's `rotation.y` spins the wheel on its axle. The animator itself is unchanged.
- place_ashtrays (filter lands beside or on the burnt end): FIXED. The filter offset now follows the butt's real axis after Rz then Ry, `(cos tz·cos ty, sin tz, -cos tz·sin ty)·0.016`. It sits on the -Y end, away from the modelled lit end.
- place_wall_coat_hooks (45° tip bends sideways): FIXED. The tip now uses `rotation.x = π/4` at (0, 0.006, 0.055), so it rises up and out from the shaft's end. It had been `rotation.z = π/4, x = π/2` at x -0.008. Sibling clones inherit it.
- place_wooden_brooms (leans sideways, never touches the wall; head corner off the floor): FIXED. The lean is now `rotation.x = -lean` with Euler order YXZ, toward the wall and after the yaw. The foot stands `1.37·sin(lean) + 0.012·cos(lean)` from the wall, so the pole's top rests on it (probe: 11.8 mm from the pole's axis, i.e. touching). The group drops by the head's lowest edge, so the head's wall-side edge is on the floor (min y 0.0001, was 0.015). The same single random draw is kept.
- place_paper_towel_dispensers (paper hangs over the cabinet front, not from the slot): FIXED. The paper moved from (y -0.19, z 0.052) to (-0.178, 0.053). Its top edge is now inside the modelled slot recess (y -0.148, z 0.0485), and it hangs out over the slot's lower lip from under the slot decal.
- place_small_hand_mirrors (handle and knob sink 6–8 mm): FIXED. The handle axis moved from y 0.004 to 0.010, so its thick end rests on the floor, and the knob moved to y 0.012. The whole mirror is no longer lifted off the floor.
- place_dog_leashes (the coil is 3 stacked rings, not a spiral): SKIPPED. The note calls the placement OK. A spiral would need a new model.

## b13

- place_small_compacts (open lid detached, no hinge): FIXED. The hinge mesh is now built before the open/closed branch. On the open variant it is turned so its pin runs along Z and sits in the 2 mm gap between the two rims (x 1.025 r, y 0.006), overlapping both, so the lid is joined to the pan. The closed variant is unchanged.
- place_small_perfumes (atomizer bulb floats 9 mm beside the cap): FIXED. The bulb moved from x 0.030 to 0.020, so it presses 1 mm into the cap collar. The three tassel cords moved with it and still hang from the bulb onto the bottle shoulder.
- place_small_hairbrushes (handle 4.5 mm short of the pad, sunk 6 mm): FIXED. The handle moved from (-0.052, 0.006) to (-0.046, 0.012). Its flared end is 1.5 mm into the pad and its thick end rests on the floor.
- place_spilled_pill_bottles (cap 2.5 mm off the bottle, sunk 2 mm): FIXED. The cap moved from x -0.070 to -0.0645, so its hollow end is 3 mm over the bottle's neck end. Its y went from 0.014 to 0.016, so it rests on the floor. The bottle is tipped 0.045 rad (y 0.015): its neck end meets the wider cap coaxially and its base end still lies on the floor.
- place_spray_bottles (trigger lever sticks out of the back of the head): FIXED. The lever moved from (-0.018, 0.148) to (0.027, 0.146) with `rotation.z = π/2 + 0.4`. It now hangs from under the snout and nozzle at +X, swept slightly forward, with the finger scallops facing forward. It works in the tipped variant too.
- place_soap_dispensers (body's back half inside the wall): FIXED. The wall offset changed from `CELL*0.5 - 0.040` to `CELL*0.5 - 0.080`, so the 8 cm body backs onto the wall face (raycast: wall at local z -0.080). The placement pass had pushed it out but left a 4 mm gap.
- place_sink_faucets (spout in the wall plane, off the riser): FIXED. The spout is now `rotation.y = -π/2` (was z π/2, y π) at (0, -0.040, 0.008). Its joint-collar end starts horizontally at the riser top (y 0.020 to 0.030 on the riser axis) and bends down to the aerator outlet 7 cm into the room (z 0.078, y -0.040).
- place_sink_faucets (knobs 1 cm in front of the plate): FIXED. Knob z changed from 0.014 to 0.004, so the fluted skirt sits on the plate face (z 0 to 0.008).
- place_sink_faucets (cross arms: one sticks out of the wall, one lies flat): FIXED. Both arms got `rotation.x = π/2` and sit at z 0.009 on the knob's domed face, forming a "+" in the wall plane: cross1 is 20 mm wide and cross2 is 20 mm tall.
- place_hair_dryers (1 cm gap behind the plate): FIXED. The plate moved to z -0.010, so its back is on the wall (raycast: wall at local z -0.020).
- place_hair_dryers (barrel and nozzle 2.5 cm into the wall): FIXED. The body, nozzle, handle and cord moved to z 0.045, so the barrel rests on the plate face instead of cutting into the wall.
- place_hair_dryers (cord loop 1.5 cm below the handle): FIXED. The half-torus is now `rotation.z = π` (was lying flat with x π/2), scaled 0.65 in X, at y mountY-0.156. It hangs as a U from the handle butt, and both sleeved ends go 4 mm up into the butt.
- place_toilet_paper_dispensers (roll and core hidden inside the housing): FIXED. The roll and core moved from (0.075, mountY-0.03) to (0.070, mountY-0.11). The axis is 1 cm below the housing, so the lower 6.5 cm of paper hangs out of the open bottom, flush with the housing's side. + The tear bar now looked like a blade stuck in the roll's end. It is turned `rotation.y = π/2` and placed at (0.05, mountY-0.10, 0.062), along the housing's front bottom edge over the roll.
- place_towel_bars (2 cm gap behind the bracket rosettes): FIXED. The group origin is now on the wall face (`CELL*0.5`, was `CELL*0.5 - 0.020`), so the brackets (local z 0 to 0.06) sit on the wall (raycast confirmed).
- place_hair_brushes (handle 5 mm past the pad, sunk 6 mm): FIXED. The handle moved from (0, 0.007, 0.11) to (0, 0.014, 0.103). The neck is 2 mm into the pad and the butt rests on the floor.
- place_hand_towel_dispensers ("PULL" strip 3.5 mm in front of the body): FIXED. The badge z changed from 0.076 to 0.0725, so the plate's back is on the cover face (z 0.070) and the chevrons stand proud of it.
- place_makeup_compacts (closed lid floats 1 mm): FIXED. The lid y changed from 0.010 to 0.009, so it sits on the base (0.006).
- place_makeup_compacts (open mirror faces into the lid, culled): FIXED. The mirror circle got `rotation.y = π` and z 0.0253. It faces -Z, out of the lid toward the powder, 0.3 mm proud of the lid's mirror seat. It now shows in game.
- place_lipsticks (loose cap sinks 5 mm): FIXED. The cap now lies on its side (`rotation.set(0, 0.4, π/2)`, was tilted 0.4 rad from upright) at y 0.014 = its radius, still turned 0.4 rad and clear of the tube.
- place_eyeshadow_palettes (closed lid floats 1 mm): FIXED. The lid y changed from 0.010 to 0.009, and the logo decal went from 0.014 to 0.0125 so it stays 0.5 mm over the lid top.
- place_perfume_bottles (atomizer tube enters the bottle's side): FIXED. The tube now runs diagonally (`rotation.z = 1.131`, centre (0.0205, 0.056)) from inside the cap's side at y 0.060 down into the bulb. The bulb is unchanged.
- place_nail_polish (tipped variant half below the floor): SKIPPED. Handled centrally: the whole object sinks, and the placement pass lifts it (group y 0.014, min y 0.000 in the dump).
- place_mascaras (wand-out: cap, wand and brush turned about their own centres, not lined up): FIXED. They now hang in a pivot group `wandSet` at the cap centre, turned 0.3 rad and tipped 0.166 rad. Along its -X axis: the cap at 0, the wand at -0.0195 with its collar 3 mm into the cap socket, and the brush at -0.0345, centred on the wand tip.
- place_mascaras (wand-out: wand and brush 5-7 mm above the floor): FIXED. The tip lowers the brush end until the brush rests on the floor (bottom ~0), and the cap still rests on the floor (group y 0.0101).

## b14

- place_bobby_pins (bend half-torus standing in the YZ plane): FIXED. `rotation.x = π/2` (was `rotation.y = -π/2`) with the existing `rotation.z = π/2`, so the bend lies flat in the pin plane, joins the prongs at z -0.002 / +0.002 and bulges towards -X (y 0.0003-0.0017).
- place_makeup_brushes (cone tuft 3 mm clear of the ferrule): FIXED. The cone moved from x 0.040 to 0.036, so its base is 1 mm inside the ferrule mouth. It moved up from y 0.009 to 0.010 so it no longer dips 1 mm.
- place_makeup_brushes (dome dips 2 mm): FIXED. The dome moved up from y 0.009 to 0.011, so its underside rests on the floor.
- place_hair_claws (teeth buried in the bottom jaw): FIXED. Each tooth is turned `rotation.y = π/2` (long axis and taper along +X) and moved to x 0.027. The row now sticks 8 mm out of the jaw's front rim (x 0.022-0.032, y 0.006-0.010, spaced 4.4 mm) instead of lying across each other inside the jaw.
- place_hand_mirrors (knob sinks 3 mm, twice the handle's width): FIXED. The knob is scaled 0.625 (r 5 mm). It rests on the floor at y 0.005 and overlaps the handle end by 2 mm.
- place_pencil_eyeliners (cap sinks 1 mm): FIXED. The cap moved from y 0.004 to 0.005 (its radius). The body stays inside the sleeve.
- place_makeup_sponges (body dips 4 mm): SKIPPED. The sponge is its own lowest point, so this is a whole-object floor sink that the placement pass handles.
- place_perfumes (tassel cord 4 mm below the bulb): FIXED. The three cord segments and the knot moved up 5 mm, so the top segment runs 1 mm into the bulb's underside (y 0.056 against 0.055).
- place_hairbrushes (pad standing as a vertical disc, scale.y thickening it, bristles floating through): FIXED. The pad's `rotation.x = π/2` is removed and `scale.y = 1.4` became `scale.z = 1.4`. The head lies flat, cushion up (y 0.007-0.021), and is oval along Z like the bristle grid and hair strands. The bristles (y 0.020-0.032) now stand in the cushion.
- place_hairbrushes (handle 5 mm short of the pad): FIXED. The handle moved from x -0.10 to -0.093; its near end is 2 mm into the pad rim.
- place_hairbrushes (knob sinks 2 mm): FIXED. The knob is scaled 0.875 (r 14 mm). It sits on the handle axis, rests on the floor and overlaps the handle end by 2 mm.
- place_standing_mirrors (stem pierces the glass and back plate): FIXED. The stem is scaled 0.714 in y and centred at (0, 0.033, 0.006). It runs from inside the base (y 0.008) up into the bottom of the tilted frame rim (y 0.058, rim bottom 0.054 at z 0.008), and stops short of the glass (0.063) and the back plate (0.060).
- place_standing_mirrors (no yoke connects the side pegs to the stand): SKIPPED. A yoke would need a new part, and the pegs read as pivot knobs on a frame that now sits on its post.
- place_hairdryers (handle and switch inside the barrel): FIXED. The dryer now stands on its handle butt, as the placer's comments intend ("protruding down", cord from the handle base). The barrel, nozzle and both grille decals moved up from `barrelR + 0.002` to `barrelR + 0.075`. The handle moved to y 0.040, so its butt is on the floor and its top is 5 mm into the barrel. The rocker switch moved onto the grip's +Z side face at (x -0.019, y 0.060, z 0.0175); before it was buried in the grip's axis.
- place_hairdryers (nozzle reversed, narrow end on the barrel): SKIPPED. The Blender part was built for this orientation: the wide end has the lipped, recessed front face that the grille decal sits in, and the narrow end is open. Reversing it would need a new model, and a front that widens reads as a diffuser.
- + place_hairdryers (cord): FIXED. `cy` was `0.005 + sin(1.5πt)·0.008`, which put the far half of the cord up to 6 mm under the floor, so the placement pass lifted the whole dryer about 7 mm. The cord now humps and settles on the floor (`0.003 + max(0, sin)·0.008`).
- place_pill_bottles (tipped cap sunk 12 mm): FIXED. The cap now sits in a `capG` group that takes the tip (`rotation.z = π/2 + capTilt`, same random range). The group height `0.021·cos(tilt) + 0.005·|sin(tilt)|` puts the tilted rim on the floor (cap y 0.0007-0.042).
- place_pill_bottles (push tabs in a horizontal ring around the tipped cap): FIXED. The four tabs are children of `capG`. They sit as radial ribs on the cap's skirt, in the cap's own frame: r 0.0215, `rotation.y = -angle`, set 45° off so none is underneath. The per-tab random tilts, which only mimicked the cap's tilt, are no longer applied. Their four `Math.random()` draws are still made (and discarded), so the pill count, types and positions come out as before.
- place_children_shoes (toe overhangs the sole by 2 cm and hovers): FIXED. The toe moved from x 0.06 to 0.04. It ends flush with the sole's front (x 0.07) and its underside (y 0.004) is inside the sole.
- place_children_shoes (reads as a blob end-on): SKIPPED. This is the toe model's shape, not placement.
- place_toy_cars (wheel axles along the car's length): FIXED. `wheel.rotation.x = π/2` (was `rotation.z`). The axles run across the car (Z) and the tyres sit in the body's modelled wheel arches.
- place_toy_cars (flipped variant pitches onto its nose or tail): FIXED. The variant now uses Euler order `YXZ` with `rotation.x = ±π/2` (the same random sign draw), so the car rolls onto its side about its own length axis before the yaw. `y = 0.025` is its half-width, so the side rests on the floor.
- place_small_dolls (head floats 2 mm above the torso): FIXED. The head moved from y 0.14 to 0.136 (chin 2 mm into the torso top) and the eyes moved with it (0.145 to 0.141).
- place_small_dolls (lying variant: arm 3.2 cm into the floor, torso hovering): FIXED. The 40 % variant now lies on its back, face up: Euler `YXZ`, `rotation.x = -π/2`, y 0.028. The back of the head rests on the floor, the torso is 8 mm up (the big head holds it) and the arms and legs are above the floor.
- place_small_kites (tail floats 2 mm): SKIPPED. The tail is the kite's lowest part (the plane is at 3 mm), so this is a whole-object hover that the placement pass sets down.
- place_small_pinwheels (hub 5 mm below the sails' centre, stick through the sail plane): FIXED. The sails group is centred on the hub (y 0.27). The hub moved in front of the sails (z 0.004, dome forward) and the stick behind them (z -0.0055, top at 0.28).
- place_small_pinwheels (spins about the stick like a turnstile): FIXED. The sails are no longer stood up one by one (`sail.rotation.x` removed). Instead the group gets `rotation.x = -π/2`, so the sails are still vertical and facing Z with the same handedness, and the existing animator's `sailsGroup.rotation.y` now spins them about the hub axis. The animator code and its random period and direction are untouched.
- place_small_yo_yos (string a stiff horizontal rod at axle height): FIXED. The string leaves the underside of the axle between the halves (y 0.020) and slopes down 0.18 rad to the floor 10 cm out: `rotation.x = π/2 + 0.18`, centre (0, 0.0105, 0.049). The modelled knot is at the free end on the floor.
- place_toy_drums (strap plane through the drum's axis, 4.5 cm above the head and 5.5 cm into the floor): FIXED. The strap now lies face-up on the floor (`rotation.x = -π/2`, y 0.0015). One end is tucked under the rim at +X and it trails out 0.3 rad off the tangent (the existing `rotation.z = 0.3`). It is single-sided and faces up, so it shows.
- place_toy_drums (bottom membrane at y -0.001): SKIPPED. It faces down under the drum's own floor, so it is never visible and harmless.
- place_small_pacifiers (shield dips 6 mm): SKIPPED. The shield rim is the object's lowest point; the placement pass lifts the whole pacifier onto it.
- place_skipping_ropes (handles across the coil, rope through their middles): FIXED. Each handle now continues from its rope end. The coil's ends are at x 0.099 and -0.104 at z 0 and both run along +Z. Each handle is tangent there and splayed 0.6 rad outward so it clears the neighbouring turn (Euler `YXZ`, `rotation (π/2, ±0.6, 0)`). Its end overlaps the rope end by about 4 mm.
- place_skipping_ropes (handles sink 2 mm): FIXED. The handles are at y 0.010 (their radius).
- place_childrens_books (±0.06 rad tilt dips an edge 4 mm): SKIPPED. The whole book tilts, so this is whole-object floor contact that the placement pass handles.
- place_small_maracas (upright: handle 3 cm into the floor, the rest inside the bulb): FIXED. The upright maraca now stands head-down on its bulb, neck up: `bulb.rotation.x = π`, y 0.0238 (resting on the floor). The handle is flipped (`rotation.x = π`) and rises from the neck at y 0.0836: collar 4 mm inside the neck, knob up, top at 0.124.
- place_small_maracas (tipped variant: bulb sinks 8 mm, handle floats 1.3 cm): FIXED. The variant uses `rotation.z = π/2 + 0.222` and y 0.0333. The maraca rests on the bulb (r 0.028) and the handle's knob end and lies tilted like a real one.

No problem was logged for: CottonPads, Tweezers, NailFiles, SmudgeMirrors, BirdBaths, NailPolishes, PlushBears, RubberBalls, SmallCrayons, DrawingPapers, MarbleScatters, BuildingBlocks, SmallTambourines.

## Seen but not logged, left alone

- place_ironing_boards: the cord still runs through the board's back end on its way to the floor. Fixing that needs a new cord path.
- place_lipsticks: the band and twist rings are wider than the tube and dip about 3 mm below the floor.
- place_mascaras (closed variant): the cap sinks 2 mm and sits 0.5 mm off the tube's neck.
- place_small_compacts (closed variant): the hinge sits on the top shoulder, not at the seam.
- place_toilet_paper_dispensers: the housing's back is 1 cm inside the wall, where it can't be seen.
- place_wall_coat_hooks: on level 2 / seed 3, sibling clones seem to float away from any wall, maybe past a wall's end. That is a wall/cluster placement problem, not a part-to-part one.
- Placement pass (`world::placement`): it skips protected objects with `lo > 0` and ignores moves under 6 mm. Small whole-object hovers skipped here as "central" (e.g. SmallKites, 2 mm) will stay.

## Count

126 lines:

- **109 FIXED** (b11 36, b12 22, b13 23, b14 28). Lines marked `+` also fix an extra bug in the same placer.
- **17 SKIPPED:**
  - 6 are whole-object floor contact, left to the placement pass: DroppedGloves tilt, NailPolish tipped, MakeupSponges, SmallKites, SmallPacifiers, ChildrensBooks.
  - 7 are not bugs or are harmless: DroppedGloves trim, VacuumCleaners open bay, PackedSuitcases wheels, OpenedWallets 0.25 mm, PetBowls kibble, ToyDrums bottom membrane, DogLeashes coil.
  - 4 would need a new model: UmbrellaStands handle (tip-up umbrellas), StandingMirrors yoke, Hairdryers nozzle direction, ChildrenShoes toe shape.
