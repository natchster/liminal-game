# Placement fixes, batch E (b15, b16, b17 notes)

One line per logged placement bug. `+` marks an extra bug found in the same placer while fixing a logged one. Every change is in `backrooms.js`, inside the named placer, and only changes positions, rotations (including Euler order), pivots (wrapper Groups), scales or loop logic. No `new THREE.XGeometry(...)` text or `PG(...)` wrapper was touched. Random draws, probability gates, materials, animators (their targets were moved to pivot Groups where needed: metronome arm, elephant ears) and tracked position arrays are unchanged.

Several "tipped on its side" variants rotated about Z, which stands a long object on its end. They now roll about the object's own long axis (X), and the Euler order is `'YXZ'` (roll first, then the random yaw). With the default 'XYZ' order, a world-X rotation after the yaw would tilt the object instead.

Checks: `node --check backrooms.js` after every edit. `node tools/prop-codemod.mjs --check` lists only "not built" problems, all from other batches. `node build-standalone.mjs backrooms`, then `tools/prop-gallery.mjs` on level 4 seed 7/9 and level 2 seed 5. Private close-up renders (several azimuths, rare variants forced) and per-part bounds dumps.

Totals: 112 logged-bug lines, of which 93 are FIXED, 1 is PARTLY FIXED and 18 are SKIPPED. The skips are mostly already handled by the new parts or by the central placement pass, or would need a new model. There are also 12 extra `+` bugs: 10 fixed, 2 skipped (both need new geometry).

Shots:
- b15: `shots/fix/e_b15_c1.png` … `e_b15_c6.png`, `e_b15_gallery.png`.
- b16: `e_b16_1.png`, `e_b16_2.png`, `e_b16_close1.png` … `e_b16_close5.png` (plus `close2b`), `e_b16_doll_before.png`, `e_b16_doll_1.png`, `e_b16_hobby_1.png`, `e_b16_hobby_2.png`.
- b17: `e_b17_1.png`, `e_b17_2.png`, `e_b17_c1.png` … `e_b17_c5.png`.
- All three batches: `e_all_l2.png`.

## b15

- placeSmallRecorders, finger-hole discs float 1 mm over the body: **FIXED**. y 0.028 → 0.0255, so the discs sit inside the body's finger-hole recesses (recess floor at y 0.0251).
- placeSmallRecorders, "bell flare" narrows outward: **SKIPPED**. Not a placement bug: the b15 part models it as the foot-joint socket.
- placeSmallTriangles, the rods cross as an asterisk: **FIXED**. `rotation.z = atan2(-dx, dz)` (was `-atan2(dz, dx)`), so with `rotation.x = π/2` each rod runs along its own side. The triangle closes, with the gap near vertex 0.
- placeSmallXylophones, bars 1–6 hang in the air between parallel rails: **FIXED**. The rails converge (z ±0.032 at the centre, `rotation.y = ±atan(0.003/0.035)`), which is the "trapezoidal frame" the comment describes, so both ends of every bar lie on a rail. The bars were lowered 1 mm (y 0.013) to rest on the rail tops.
- placeConductorBatons, taper reversed: **SKIPPED**. The part corrects it inside the same frame (thick at the cork, fine at the tip).
- placeSmallMetronomes, slot plane hangs vertically in front of the sloping face: **FIXED**. It is tilted back 0.214 rad (the face slope) at z 0.0287, so it stays 0.4 mm in front of the face along its whole length.
- placeSmallMetronomes, the arm swings about its own centre in front of the body: **FIXED**. The arm now hangs in an `armPivot` group at its round foot (y 0.018 on the face), tilted with the face. The animator drives `armPivot.rotation.z` (same period, phase and ±0.35 amplitude), so the arm swings about its foot in the face's plane, 4.5 mm proud of it (the weight 0.5 mm off it).
- placeSmallCastanets, shells swapped and interpenetrating: **FIXED**. The bottom shell is now the flipped one (dome on the floor, hollow up, rim at y 0.018), and the top shell is dome-up with its rim 1 mm above. Both have `scale.y` 0.6, so the closed pair keeps the original ~3.7 cm height instead of becoming a 6 cm ball. The cord loop moved from inside the shells to the ear rim (x 0.030, standing in XY).
- placeSmallSpinningTops, wedges z-fighting the cone face: **SKIPPED**. Already fixed by the part's sunk face.
- placeSmallSpinningTops, stem starts above the face: **FIXED**. Stem y 0.056 → 0.0545; its foot is now 0.2 mm into the sunk face.
- placeSmallToyPlanes, propeller inside the nose sphere: **FIXED**. `propPivot` x -0.005 → -0.015, just ahead of the nose (front at x -0.014).
- placeSmallToyPlanes, "tipped on side" stands the plane on its nose: **FIXED**. Euler order 'YXZ' with `rotation.x = π/2` rolls it about the fuselage. y 0.070, so the lower wing tip rests on the floor.
- placeSmallSkateboards, truck axles 8.5 mm above the wheel centres: **SKIPPED**. Minor and hidden inside the wheels. Lining them up would need a taller truck model: the truck box is 1 cm tall, but axle-to-deck is 1.8 cm.
- placeSmallToyBoats, the prow cone reached through the hull: **SKIPPED**. Already fixed by the new bow part.
- placeSmallToyBoats, "tipped on side" stands the boat on its stern: **FIXED**. 'YXZ' with `rotation.x = π/2` rolls it about the keel; y 0.0225 (the hull's half-beam). The mast and sail now lie along the floor.
- placeSmallJacks, jacks interpenetrate: **FIXED**. Each jack re-draws its scatter position (up to 30 draws, the range widening 6 % per retry) until it is at least 3.4 cm from the other jacks and 2.8 cm from the ball.
- placeMagneticLetters, letters interpenetrate: **FIXED**. Same retry scatter, with at least 4 cm between tile centres (the tile half-diagonal is 2 cm).
- placeSmallPiggyBanks, ears float 4 mm over the back: **FIXED**. The ear triangle was built tip-down. It is now turned tip-up (`rotation.z = π`) with its base at y 0.078, sunk into the back.
- placeSmallPiggyBanks, the single-triangle ears vanish from behind: **PARTLY FIXED**. The left ear's front face pointed inward. It is now mirrored (`rotation.y = π - 0.4`), so both ears show from outside and from the front. From behind they are still one-sided; fixing that needs a material change or a new part.
- placeSmallPiggyBanks, the tail ring lies flat against the rump with its centre inside it: **FIXED**. The curl now stands in the pig's side plane (the `rotation.y` was removed) at x -0.0635. Its thick start is sunk 1.5 mm into the rump, and it curls up and back.
- placeSmallToyDinos, the snout box is mostly inside the head: **FIXED**. x 0.068 → 0.076. The muzzle now sticks 8 mm past the front of the head, with its back half inside the head.
- placeSmallToyDinos, eyes buried: **SKIPPED**. Already fixed by the part's sockets.
- placeSmallToyDinos, the legs end 5 mm above the floor: **SKIPPED**. Not a part-to-part bug: the leg tops are inside the torso and the soles are the lowest point, so the central pass sets the whole dino down.
- placeSmallToyPhones, the cups sank into the base: **SKIPPED**. Already fixed by the part.
- placeSmallToyPhones, the handset hangs 1 cm over the back edge: **FIXED**. The handset and cups moved to z 0.023 (cups x ±0.037 → ±0.033), so they rest wholly on the base top. The 3×4 button grid was tightened to z -0.029…0.0085 (pitch 1.25 cm) so it stays clear in front of the handset. The cord loop moved with the handset (z 0.005, 6 mm in front of the earpiece cup), still clear of the buttons.
- placeSmallToyRobots, the mouth bar floats 0.5 mm: **FIXED**. z 0.012 → 0.0112; its back face is just inside the front of the head.
- placeSmallToyMicrophones, the handle is 8 mm below the grille/collar axis: **FIXED**. The handle and collar are now on the grille's axis, which slopes down 0.072 rad from the grille centre so the butt rests on the floor. The handle's end is inside the collar, which sits 1.6 cm along the axis.
- placeSmallPrincessWands, the star's lower points reach 2 cm below the floor: **FIXED**. The upright star is laid flat (`rotation.x = -π/2`) by a pivot group at its centre on the stick's tip, at the height of the stick's axis.
- placeSmallToySwords, the crossguard reaches 8 mm into the floor: **FIXED**. The sword now lies flat instead of on its blade edge: order 'YZX', `rotation.x = π/2` (roll about the blade axis), `rotation.z = -0.02` (the point dips so it rests on the pommel and the blade end), y 0.010.
- placeDollHeads, the neck stub is hidden inside the head: **FIXED**. y 0.008 → -0.003, so the stub pokes 5 mm out under the head (the head rests on it after the placement pass).
- placeDollHeads, the mouth quad floated and the crown poked through the hair: **SKIPPED**. Already fixed by the head and hair parts.
- placeBabyCribFragments, the spindles start 4 mm above the base rail: **SKIPPED**. The new spindle's tenon already reaches into the rail.
- placeDollBodyParts, the hand/foot ball sinks 4–17 mm into the floor: **FIXED**. Each limb's height is now computed from its tilt, so its lower end (the 1.4·limbR hand ball or the socket rim) rests on the floor. The random draws are unchanged.
- placeSingleChildShoes, the lace was buried: **SKIPPED**. Already fixed by the part's open throat.
- placeSingleChildShoes, the tipped variant stands on its heel: **FIXED**. 'YXZ' with `rotation.x = π/2` rolls it about the heel-toe axis; y 0.025 (its half-width).
- placeWornTeddyBears, the ears float up to 12 mm: **FIXED**. Each ear is tilted out 0.6 rad (`rotation.z = -s·0.6`) and placed along that direction with its base centre 3 mm into the head, so the flat base follows the round head.
- placeWornTeddyBears, the missing-eye socket was inside the head: **SKIPPED**. Already fixed by the head part.
- placeOverturnedTricycles, `rotation.z` stands the trike on its rear: **FIXED**. 'YXZ' with `rotation.x = π/2` rolls it about the front-back axis; y 0.082, so a grip end rests on the floor.
- placeOverturnedTricycles, the handlebar floats 5 cm in front of the fork top: **FIXED**. The handlebar and grips moved onto the fork top (0.069, 0.186).
- placeOverturnedTricycles, the fork ends 2.8 cm above the front axle: **FIXED**. The fork was re-centred so its dropout end is on the axle (0.11, 0.055), with the same 0.3 rad rake.
- + placeOverturnedTricycles, the seat floated 4.3 cm above the main bar: **FIXED**. Seat y 0.16 → 0.117, on top of the bar.
- + placeOverturnedTricycles, the rear wheels are not joined to the frame (there is no rear axle): **SKIPPED**. Needs a new part.
- placeTeddyPaws, the torn disc faces down: **FIXED**. `rotation.x` +π/2 → -π/2, so it faces up. It sits at y 0.0445 in the ragged crater, scaled 0.62 to fit the crater's ~1.1 cm opening; the part's ragged tufts poke through it.

## b16

- placeChildrensTeaSets, handle stands in the YZ plane and bulges sideways (-Z) at the back: **FIXED**. Dropped the `rotation.y = -π/2`, so `rotation.z = π/2` alone puts the C in the XY plane, bulging out behind (-X). Moved it to x -0.0215 so both ends (y 0.012 / 0.028) sit in the pot wall.
- placeChildrensTeaSets, spout's wide end at the tip: **SKIPPED** (not a bug). The part models it as a flared mouth.
- placeDeflatedBalls, decals float 3–9 mm over the squashed top: **FIXED**. The soccer patches moved in to r 0.028 and are spread over the arc away from the buckled +X side (a = 1.1 + (i+0.5)/n·4.3). They are tilted 0.16 rad with the crown's slope and sit at y 0.0814; the measured gaps are 0.3–2 mm, none buried. The basketball lines are scaled 0.45 along their length so they span only the crown, at y 0.0831. The old lines were buried at the centre and floated up to 3 cm over the dent.
- placeRubberDucks, eyes buried in the head: **FIXED**. They moved to (0.0357, 0.0539, ±0.0144), half out of the measured head surface (it is 20 mm from the head centre there).
- placeRubberDucks, only 3 mm of the bill pokes out: **FIXED**. Bill x 0.040 → 0.048. Its root is 3 mm in the head and 11 mm shows.
- placeRubberDucks, body sinks 3 mm: **SKIPPED** (whole object; the central placement pass handles it).
- placeGameboys, speaker dots float 1.5 mm, screen 0.5 mm over the bezel, D-pad/buttons 0.5 mm over the body: **FIXED**. Screen y 0.0164, D-pad and A/B y 0.0153, dots y 0.0143. Each now rests on the surface under it.
- placeYoyos, string dashes don't join each other or the yo-yo: **FIXED**. The 6 segments now form one polyline. It starts at the hub, leaves between the halves, drops to the floor and curves away along it. Each segment is oriented and stretched between consecutive points (quaternion + scale.y) with slight overlap.
- placeLegoBricks, studs float 0.5 mm: **FIXED**. Stud y 0.012 → 0.0114 (on the body top at 0.010).
- placeLegoBricks, "stacked" bricks hover at random spots: **FIXED**. A stacked brick now sits on the previous brick at y + 0.010, with its studs in the hollow underside. It is square to that brick (same yaw or +90°) and shifted along it by up to one stud. The same four random draws are used in the same order; the first brick falls back to the floor.
- placeJumpRopes, inner loops raised 5 / 10 mm so they float: **FIXED**. All three loops are at y 0.004 on the floor. They sit side by side 6 mm apart, and the part's own rise and fall gives the coil.
- placeTeddyBears, legs and arms on the back, face at -Z: **FIXED**. Legs moved to z -0.030 with `rotation.y = π` so the flat soles face out front under the face. Arms moved to z -0.008.
- placeTeddyBears, body floats 4.5 mm / legs 6.8 mm: **FIXED** for the legs (y 0.020 → 0.0177, level with the seat). The whole-object float is left to the central pass.
- placeCrayons, 0.5 mm gap between wrapper and back end: **FIXED**. Back x -0.032 → -0.031 (it butts into the wrapper).
- placeActionFigures, lying pose turned head-to-foot (`rotation.x = +π/2`): **FIXED**. Torso, arms and legs use `-π/2`, so shoulders and the neck post point to the head, fists to the head, boots away. + The head is turned `-π/2` too, so its neck peg points into the torso instead of the floor.
- placeActionFigures, lying torso/arms/legs float 3/7/6 mm: **FIXED**. Lying y: torso 0.007, head 0.011, arms 0.003, legs 0.004 (each rests on the floor). The arms moved to z -0.004 so the shoulder ends line up with the torso's shoulders.
- placeBabyRattles, grip ring stands vertical and is sunk 13 mm: **FIXED**. The ring now lies flat on the floor (`rotation.x = π/2`, y 0.004, x 0.0483).
- placeBabyRattles, handle enters the ball 15 mm low: **FIXED**. The handle is aimed at the ball centre (`rotation.z = π/2 - 0.222`, centre (0.0095, 0.0111)). Its end is 7 mm inside the ball, and it slopes down into the near side of the flat ring, with its low end on the floor. + The 3 dots were 3 mm inside the ball. They are now on the crown (r 0.007, y 0.0389), tilted 0.42 rad with its curve.
- placePacifiers, ring detached 9 mm behind the shield and floating 5 mm: **FIXED**. The ring is hinged at the back centre of the shield (its top edge there), swung down and back (`rotation.x = -0.67`, centre (0, 0.0102, 0.0102)) until its far edge rests on the floor.
- placeSippyCups, handles are arches in the tangent plane, half in the wall: **FIXED**. `rotation.z = -side·π/2` (was `rotation.y = ±π/2`) puts each C in the radial plane, bulging out sideways. x ±0.026 puts both ends in the wall (radius 0.026–0.028 measured at their heights).
- placeSippyCups, tipped variant lies with its axis on the floor, half buried (and its handles pointing up/down): **FIXED**. It now tips about X with Euler order YXZ, so the random heading still applies. `rotation.x = π/2 - 0.074` and y 0.026, so it rests on the lid rim and the foot, and the handles stay level. The 40 % draw is unchanged.
- placeBabyBottles, tipped variant half buried (-24 mm): **SKIPPED** (whole object; no part-to-part problem, and the central placement pass lifts it).
- placeStuffedRabbits, legs behind the body next to the tail: **FIXED**. Feet moved to z -0.028, under the face. The foot part has its toes at -Z, so the toes stick out in front and the raised heel sits under the belly.
- placeStuffedRabbits, feet float 7 mm: **FIXED**. Foot y 0.016 → 0.009 (the sole is on the floor, level with the body).
- placeStuffedElephants, ears half inside the head, flapping about their centres: **FIXED**. Each ear is wrapped in a pivot Group hinged at its root, 4.5 mm inside the side of the head at (-0.050, 0.078, ±0.023), with the disc offset 0.024 outward. The whole flap now shows. The flap animator turns the pivot (same `userData` base/side/phase, same draw), so it swings about the root.
- placeStuffedElephants, tusk bases ~1 mm off the head: **FIXED**. x -0.088 → -0.086 (the base is 1 mm into the head). + The eyes were 22 mm from the head centre (4.5 mm inside the 29 mm surface) and never showed. They now sit 2 mm proud at (-0.0716, 0.0811, ±0.020).
- placePetToys, bone tilt (`rotation.z ±0.125`) sinks one end's knobs 8.6 mm and the shaft 2 mm: **FIXED**. The same draw is now a roll about the bone's long axis (`rotation.x`, order YXZ so the heading still applies). The bone is lifted by 0.02(1-cos) + 0.014|sin| so the lower knobs rest on the floor. `position.set` keeps that lift (0 for the ball).
- placeBrokenWholeDolls, dress reversed (wide end at the head, hem ruffle round the neck): **FIXED**. `rotation.z = -π/2` puts the narrow bodice toward the head and the hem toward the legs.
- placeBrokenWholeDolls, dress sunk 25 mm: **FIXED**. `scale.x = 0.71` (world Y after the turn) flattens the lying skirt and y is now 0.0497. The hem rests on the floor, and the dress still covers the torso top. + The hem ruffle bumps moved from x -0.10 to -0.093, onto the hem end (they were 6 mm off it).
- placeBrokenWholeDolls, eyes, mouth and cheeks inside the head: **FIXED**. All were placed from the measured head surface. The eyes sit in the sockets (0.1091, 0.0409, ±0.0122), just proud. The mouth bar is on the lips (0.1123, 0.0264, 0), turned `rotation.y = π/2` so it runs across the face (it pointed into it). The cheek discs are on the cheeks (0.111, 0.0271, ±0.0164), facing out along the surface normal.
- placeBrokenWholeDolls, second hand floats 3.3 cm off the arm, first hand 1.5 cm off: **FIXED**. Both arms were wholly inside the dress. Each now runs from its shoulder inside the dress side (0.035, 0.040, ±0.042), down and out (spread 0.25 / 0.6 rad) to the hand on the floor, oriented by quaternion. The hand sits on the wrist, 0.073 along the arm. + The legs were 26 mm above the floor. They are now at y 0.014, coming out of the hem, and the shoes at y 0.006. + The missing-limb socket disc was hidden inside the dress. It now sits at the hem opening for a leg, or where the arm leaves the dress for an arm. + The stain was 3–15 mm inside the dress. It is now on its top (y 0.0955).
- placeBrokenWholeDolls, detached limb tilted up to 0.8 rad, one end 13 mm under the floor: **FIXED**. The limb lies flat (`rotation.z = π/2`). The tilt draw is kept and now only adds to its heading.
- placeHobbyHorses, head tilt turns the nose down and sinks head 42 mm / muzzle 54 mm: **FIXED**. `rotation.z = +0.25` (nose up, as the comment says). The head group is raised to y 0.0412, so the back corner of the jaw rests on the floor and nothing is below it (measured min y 0.000). The pole end runs into the back of the jaw. + The pole had floated 1 cm. It now rests on the knob (y 0.014), or on the axle (0.025) when wheeled, and the dust sleeve follows it.
- placeHobbyHorses, rings under the floor: **FIXED** (by the tilt fix; rings are now at y ≈ 0.064–0.074).
- placeHobbyHorses, reins hang down to -120 mm: **FIXED**. Each rein runs from its ring back along the cheek (z ±0.027) to the floor, where its end rests (`rotation.z = -1.084`, centre (0.1058, -0.0334); measured y 0.0003–0.070).
- placeHobbyHorses, mane strands stand upright above the head: **FIXED**. Each strand hangs plumb in the world from the crest (top at local y 0.036, x 0.015 ± spread) down one side of the head (z ±0.0265). It keeps a ±0.25 swing. The same four random draws are used in the same order.
- placeHobbyHorses, wheels and axle have their axis along the pole, so the wheels overlap: **FIXED**. Wheels and axle use `rotation.x = π/2` (axis across the pole, Z). The wheels sit either side of the pole at z ±0.020 and the axle runs through both.
- placeToyDinosaurs, body floats 2 mm, plates half inside the body: **SKIPPED** (whole-object float, handled centrally; the note calls the plates fine).
- + placeBrokenWholeDolls, the 1 mm crack boxes sit inside the head (y 0.040) or the dress (y 0.060) and never show: **SKIPPED**. Laying 3–5 cm straight lines on a 3.8 cm head needs a rework.

## b17

- placeBirthdayHats, tipped variant rests on its brim with apex and tassel 6 cm up: **FIXED**. `rotation.z` = π/2 + atan(0.06/0.18), past 90° by the cone's half-angle, and y = 0.068. The hat now lies along its side, with the brim trim and tassel on the floor.
- placeSmallGiftBoxes, side ribbon wraps built but never added: **FIXED**. The loop now also reads the (x, y, z) entries. Each box is scaled to a 1 cm × 2 mm strip down the middle of its face, positioned there and added. The constructor text is unchanged.
- placeSmallPartyHorns, mouthpiece on the cone's wide end: **SKIPPED**. The code comment says the dark disc goes at the wide end on purpose, so this is not clearly a bug. At the narrow tip (radius about 0) the 2.8 cm part would need rescaling.
- placeSmallPaperPlates, rim dips 1 mm into the floor: **FIXED**. Rim y 0.005 → 0.0066. The rim part is 13.2 mm tall and now starts at y 0.
- placeSmallPartyBanners, end strings aim at the mid-span sag and end in mid-air below the outer pennants: **FIXED**. `nearestY` now uses the outer pennant's own sag term, `(0.5/nFlags − 0.5)·2`, so each end string meets that pennant's top.
- placeSmallPartyBanners, anchors hang in mid-air at WALL_H−0.05: **FIXED**. The end anchors are at WALL_H, so the strings tie into the ceiling.
- placeSmallTrumpets, bell back to front: **FIXED**. Bell `rotation.z` changed from −π/2 to +π/2. The narrow end now sits on the tube at x 0.18 and the bell flares out to 0.24.
- placeSmallTrumpets, bell sinks 1.5 cm: **FIXED**. The trumpet is tilted `rotation.z` = atan(0.023/0.252) about the group origin, with y −0.0069. It rests on the bell rim (1.3 mm) and the mouthpiece (0.3 mm) instead of on a level tube with the bell through the floor.
- placeHangingBalloonBundles, strings end at the balloon centre and the tie dangles free: **FIXED**. The balloons hang neck-up from their knots (`bal.rotation.z = π`). The tie sits on top (balloonY + 0.100, not flipped). Each string runs from the anchor to the knot at balloonY + 0.105.
- placePaperGarlands, nothing ties the ends to anything: **FIXED**. The string loop now runs i = −1 … nDots−1. The two extra segments go from the outer cutouts up to the ceiling (WALL_H) at the span ends. The `PG(...)` text is unchanged.
- placeSmallFeatherBoas, puffs spaced 6.4–10 cm apart read as separate balls: **FIXED**. The 12 puffs are placed 4.5 cm apart by arc length along the same random curve, which is scaled down to fit. They overlap into one rope about 0.5 m long. Random draws are unchanged.
- placeSmallTiaras, arc rods point along (sin φ, 0, cos φ), a zig-zag: **FIXED**. `rotation.z` changed to −atan2(dx, dz), so with Euler XYZ each rod lies along its chord and the band is a smooth half-ring.
- placeSmallTiaras, spikes lie flat pointing +Z and float 2 mm over the band: **FIXED**. `rotation.x` removed so the spikes stand up. y 0.020 → 0.022 puts the base 3 mm into the band.
- placeCandyCornScatters, the 8 mm base sinks 2 mm with the axis level at 6 mm: **FIXED**. `rotation.z` = π/2 + atan(0.28), the kernel's side slope, and y = 0.0082. All three bands now touch y = 0 exactly (checked with precise world bounds).
- placePieSlices, cream placed at (cos, sin) while the wedge is at (sin, cos): **FIXED**. The cream is now at (sin, cos)·0.030, on the wedge's centre line and 3 cm out, where the slice is wide enough for it.
- placeDonuts, sprinkles buried 8–10 mm in the glaze: **FIXED**. Each sprinkle's y comes from the glaze torus surface at its radius: 0.0334 + √(0.018² − (rr−0.045)²).
- placeDonuts, fold tilted 0.4 rad (sinks 7 mm, sticks up 1.7 cm) and overhangs the corner by 1 cm: **FIXED**. The tilt is removed, since the modelled corner curls up by itself. The fold lies on the napkin (y 0.0053) with its tip on the napkin's corner, positioned in the napkin's own rotated frame.
- placeBittenDonuts, wrinkle boxes tilted 0.3–0.6 rad, sinking up to 1 cm: **FIXED**. The wrinkles lie flat on the napkin. The same random draw now sets the ridge height (`scale.y` 0.85–1.15), with y = 0.0028 + 0.0025·scale.y.
- placeSandwiches, bite box sticks 2 cm out of the side: **FIXED**. Moved from (0.4, 0.4)·halfW to (0.25, 0.25)·halfW. It now sits inside the sandwich edge with its craggy face 3 mm proud.
- placeSandwiches, bite box goes through the plate: **FIXED**. y 0.020 → 0.0305. It spans the plate top (0.0065) up to the top slice's crust (0.0545).
- placeSandwiches, crumbs float 0.5–2.5 mm: **FIXED**. y = 0.006 + size/2, so they rest on the plate. The radius range was narrowed to 0.05–0.085 (same draw) to keep them off the rim.
- placeWallBarometers, 5 mm gap behind the case: **FIXED**. The wall offset changed from 0.030 to 0.025, so the 5 cm case back is flush with the wall.
- placeSmallOperaGlasses, barrels end to end on one axis: **FIXED**. Both barrels are at x 0.030, z ±0.023, parallel on either side of the hinge bridge. The lenses moved with them.
- placeSmallOperaGlasses, handle on the barrel line and 2.2 cm into a barrel: **FIXED**. The handle leaves the side of the first barrel (socket 2 mm in) along −Z. It slopes 0.16 rad down so the finial rests on the floor, which gives a lorgnette.
- placeSmallFoldingFans, all 8 sticks parallel along +Z: **FIXED**. `rotation.set(π/2, a, 0, 'YXZ')` lays each stick flat first, then swings it to its heading about the rivet, so the sticks fan out under the 75° paper.
- placeSmallMusicBoxes, crank handle vertical on the arm's line: **FIXED**. The handle is turned `rotation.z` = −π/2 and sticks out +X from the arm's upper boss. + The arm moved up 13 mm so the shaft enters its lower boss (it was centred on the shaft).
- placeAncientPortraits, canvas 1 mm in front of the frame face: **SKIPPED**. The note marks the placement OK. The canvas exactly fills the opening, so nothing overlaps; this is cosmetic.
- placeAncientBookStacks, gilt bands run along the spine and stand 0.5 mm off it: **FIXED**. The bands are turned `rotation.x` = π/2 to run across the spine, scaled to 0.6·bh, and placed on the spine crown at 0.2 and 0.8 of its length.
- placeAncientBookStacks, each book 1 mm above the one below: **FIXED**. `y += bh`.
- placeRockingChairs, rockers run side to side: **FIXED**. `rotation.y` = π/2, at x ±0.12 under each pair of legs, so they run front to back.
- placeRockingChairs, straight rocker tilted 0.15 rad (−1.7 / +3.7 cm): **FIXED**. The tilt is removed, since the modelled rocker is curved. y 0.009 puts its low middle on the floor.
- placeRockingChairs, rock animator turns about X, across the rockers: **FIXED**. `chair.rotation.order = 'YXZ'`, so the animator's `rotation.x` is applied in the chair's own frame and rocks along the rockers whatever the yaw (in XYZ order it rocked about world X). The animator code is unchanged.
- placeRockingChairs, armrests run across the chair with no posts and float 10 cm over the seat: **FIXED**. The arms turn `rotation.x` = π/2 to run front to back at x ±0.12. Each runs from the outer back spindle (z −0.105) to the front leg (z 0.095). The front legs are scaled 1.52 in y to rise through the seat as arm posts (y 0.01–0.344).

No part-to-part problem logged (placement noted OK): placeSmallBirthdayCandles, placeSmallPinatas, placeSmallKazoos, placeCakeStands, placeCornbreadMuffins (the note calls the dimple disc fixed by the new crown). Untouched by the modeller: ConfettiScatters, SmallPaperNapkins, SmallPaperCups, SmallCupcakeWrappers, HangingStreamers, SmallPaperCrowns.
