# Placement fixes, batch B (b06, b07, b08 notes)

One line per logged placement bug. All edits are position / rotation / pivot /
scale / loop changes inside the named placers; no `new THREE.XGeometry(...)` or
`PG(...)` text was touched. Verified with `node tools/prop-gallery.mjs`
(`shots/fix/fixB_b06.png`, `fixB_b07.png`, `fixB_b08.png`) plus close-up renders
(`shots/fix/b06_close*.png`, `b07_close*.png`, `b08_close*.png`, `b08_mop.png`, `b07_cone.png`).

## b06

- placeCoffeeMugs, handle an arch beside the mug, 7 mm off it: **FIXED**. `rotation.z = -π/2` (was `rotation.y = π/2`) puts the C in the radial plane; ends on the wall (`x = bodyR - 0.002`).
- placeNewspapers, fold1 half under the floor: **FIXED**. y 0.008 → 0.030, so the lowest corner rests on the sheet.
- placeNewspapers, fold2 sinks 8 mm: **FIXED**. y 0.006 → 0.017.
- placeLetters, fold1 sinks 11 mm: **FIXED**. y 0.005 → 0.019.
- placeLetters, fold2 sinks 3 mm: **FIXED**. y 0.006 → 0.011.
- placeLetters, opened flap's hinge dips 6 mm below the floor: **FIXED**. The flap now pivots on its hinge edge (+z) at the envelope's back top edge: position (0, 0.0165, -0.043).
- placeLetters, stamp floats 0.5 mm: **FIXED**. y 0.004 → 0.0035.
- placeOffHookPhones, handset cups dip 1 mm into the floor: **FIXED**. handset y 0.014 → 0.015.
- placeChessBoards, pawn body 1.6 mm / bishop 0.5 mm above the base: **NOT FIXED** (already handled). The b06 parts extend down to bridge the gap; moving the bodies down would push the part into the base.
- placeChessBoards, king's cross buried in the cap: **NOT FIXED** (already handled). The open-crown part lets it show.
- placeChessBoards, captured pieces on their side sink 6 mm: **FIXED**. y = cellSize·0.40, so the piece rests on its base rim.
- placeCarKeys, keys/fob/tag lie loose around the ring: **FIXED**. Each item is placed along its own heading, so its hole or small ring sits on the key ring (key bow hole at r 0.012, fob and tag rings linked at r 0.0145). The tag moved to the free side of the ring (base yaw -1.4, same ±0.15 jitter). The dark hole marker moved into the bow's real hole (x -0.0042).
- placeCarKeys, tag-ring decal floats 1 mm: **FIXED**. Decal y 0.0021; the tag's metal ring is lowered to 0.002 so it passes through the tag.
- placeFormalDiplomas / WideBookshelves / KitchenCalendars / TravelPosters / PropagandaPosters / FramedNewspapers / AnatomyCharts / PeriodicTables / FramedMaps / MissionPatches / VintageAds, the "wall bug" (1.5 m off the wall, facing it): **NOT FIXED** here (skipped; fixed centrally). The `wz(z) + dz * 0.485` terms that the first central pass missed are also gone now.
- placeVintageTypewriters, body box enclosed the keys and the deck buried rows 1–3: **NOT FIXED** (already handled by the new body/deck parts, per the notes).
- placeVintageTypewriters, spacebar cuts through key rows 2–3: **FIXED**. Moved to (0, 0.021, 0.118), in front of and below row 3.
- placeVintageTypewriters, platen sinks 7 mm into the body top: **NOT FIXED**. With the new carriage-housing body the platen reads as seated in the carriage (close-ups). Raising it would leave it on nothing and make the knobs float.
- placeEaselChalkboards, front legs splay into a V: **FIXED**. Swapped the `rotation.z` signs (FL -0.12, FR +0.12) to make an A-frame. The cross-brace now also ends at the legs instead of poking 5 cm past them.
- placeEaselChalkboards, front legs pass through the board: **FIXED**. Legs moved back to z 0.040 so their fronts meet the board's back.
- placeEaselChalkboards, feet 4 mm above the floor: **FIXED**. Front legs y -0.005, back leg y -0.012 (it floated 1 cm). The back leg also moved to z -0.12 so its top meets the front legs ("3 legs meeting at top").
- placeEaselChalkboards, chalk floating 2 mm over the tray: **NOT FIXED** (already handled by the tray part's raised floor).
- placeSchoolDesks, pencil tip cone points into the pencil: **FIXED**. `rotation.z = +π/2`. Tip and eraser are now placed along the yawed pencil axis (they drifted up to 1.3 cm sideways), and the pencil lies on the open pages (y +0.011) instead of buried in them.
- placeSchoolDesks, chair back not joined to the seat (3 mm gap): **FIXED** (gap). The back now overlaps the seat's rear edge (z -0.024 offset). Proper back uprights would need new geometry and were not added.
- placeSchoolDesks, wire-rack bars touch the upright at one end only: **NOT FIXED**. The desk has a single side upright; hanging the free end needs extra hanger geometry (a new model part).
- placeSchoolDesks, text lines float 1 mm above the pages: **FIXED**. y +0.0089; positions are also rotated with the book's yaw so lines stay on the pages.
- placeSmallTypewriters, roll floats 1 cm above the base: **FIXED**. y 0.050 → 0.040 (rests on the base). Paper moved down with it. The return lever (floating 1.7 cm off the roll) now leans out from the roll's left end.
- placeSmallTypewriters, key row 3 hangs behind the deck: **FIXED**. Rows re-laid on the deck's recessed floor (z 0.0567 - 0.0169·row, y 0.0305 + 0.0062·row, keys tilted 0.35 with the deck).
- placeSmallTypewriters, deck slopes up toward the typist: **FIXED**. `deck.rotation.x = +0.35`, low at the front and rising toward the roll.
- placeSmallTypewriters, row 0 keys under the deck top: **FIXED** (same re-layout; keys sit in the tray well).

## b07

- placeRotaryPhones, off-hook cord points up out of the floor: **FIXED**. `rotation.z = -π/2`, `rotation.y = -atan2(hz, hx)`, y 0.005: it lies on the floor from the base toward the handset.
- placeRotaryPhones, on-hook handset floats ~7 mm over the horns: **FIXED**. y = 0.0685 + |tilt|·0.075, so the lower ear cup rests on its horn whatever the random tilt.
- placeCeramicMugs, handle arc inside the mug: **FIXED**. `rotation.z = -π/2`.
- placeRustedPipes, bracket unrotated, hanging in mid-air under the pipe: **FIXED**. Now a wall standoff: foot against the wall (rotation z = dx·π/2 or x = -dz·π/2), centre 2 cm off the wall, pad against the pipe's back (checked: 6 cm wallward of the pipe axis, same height).
- placeJunctionBoxes, up-conduit starts 5 cm above the box: **FIXED**. Start y 0.15 → 0.095 (in the box top).
- placeRolledCarpets, roll2/roll3 yaw only spins the roll in the Z-aligned case: **FIXED**. Z-aligned rolls use `rotation.z = π/2`, `rotation.y = π/2 + tilt`, so the tilt is a real yaw.
- placeRolledCarpets, roll2 sinks 2 cm into roll1: **FIXED**. y 0.12 + 0.21 (radii sum); roll3 likewise 0.16 above roll2 (it sank 1 cm).
- placeExtensionCords, segments oriented against the path, never join: **FIXED**. `seg.rotation.y = -curAng`; the three segments now form one continuous zig-zag.
- placeExtensionCords, plug and prongs not turned to the cord: **FIXED**. Plug `rotation.y = -curAng`, boot on the cord end (+0.025 along heading); prongs rotated too and seated on the plug face (+0.059).
- placeMopBuckets, mop head detached in mid-air: **FIXED**. The head sits in the bucket (0.012, 0.058); the stick rises from it and leans out over the rim, touching it (`rotation.z = -0.5`).
- placeMopBuckets, water disc wider than the bucket: **FIXED**. `water.scale` 0.93 (r 0.065).
- placeRustedNails, heads sunk 2 mm: **FIXED**. Nail tilted -0.055 rad and lifted 1.6 mm, so it rests on its head rim and point.
- placeRustedHammers, claw prongs beyond the head along the handle: **FIXED**. Claws now come off the back of the head (+z, opposite the face): `rotation.x = -π/2`, `rotation.z = π`, split vertically, curling back toward the handle.
- placeFireHoseReels, 2 cm gap between hose coil and drum: **FIXED**. `drum.scale` (1.2, 1, 1.2) → r 0.12 = hose inner radius (checked in bounds).
- placeWallPipeLeaks, pipe sunk 1 cm into the wall: **FIXED**. Wall offset 0.014 → 0.026 (axis 2.6 cm off the wall, r 2.4 cm).
- placeWallPipeLeaks, horizontal stain disc in mid-air: **FIXED**. Now a vertical wet patch on the wall under the drip (no rotation, z -0.024).
- placeFuseBoxes, door rotates about its centre, hinge 3.5 cm in the body: **FIXED**. Same -0.6 rad swing, now about its knuckle edge (-x, as modelled) at the box's front-left corner: position (-0.0145, 0, 0.0428).
- placeSafetyCones, reflective band flares 5–15 mm off the cone: **FIXED**. `stripe.scale` (0.95, 2.96, 0.95), fitted to the measured body taper so its 8 facets sit on the surface.
- placeEmergencyLanterns, lens 4.5 mm in front of the housing: **FIXED**. z 0.135 → 0.129. The two chain links (one lying flat, both floating ~4 mm) now stand at 90° and interlock from bracket to lid.
- placeMetalBuckets, tipped variant half sunk in the floor: **FIXED**. Tipped buckets tilt π/2 - 0.197 so the rim and the base edge both touch (y 0.057). Order 'YXZ' so the random yaw points them any way, and the bail folds down over the upper side. Also the bail was turned 90° away from its ears (`rotation.y = π/2`); now 0, so its ends meet the ears.
- placeToolPegboards, hammer head pokes 2 cm through the board: **FIXED**. Head z 0.0315 (claw rests on the board face); the handle hangs from its eye (also closed its 2.5 mm gap). The screwdriver shaft had a 1.25 cm gap below its handle; now it enters it.
- placeWoodenBuckets, bands and stave lines assume a bottom-wide bucket: **FIXED**. Bands scaled to the body radius at their height (both half-embedded). Stave lines follow the measured taper (r 0.077 mid, tilted 0.0655 rad outward).

## b08

- placeFirewoodStacks, pyramid never forms: **FIXED**. Bottom row = ceil((count+1)/2) (3 → 2+1, 4 → 3+1, 5 → 3+2); upper logs are placed in the grooves.
- placeFirewoodStacks, logs overlap 2 mm: **FIXED**. Rows centred, 0.061 spacing, layer height 0.0517.
- placeCleaningMopBuckets, bail a sideways half-ring into the bucket: **FIXED**. `rotation.z = 0` at y = bucketH·0.9, so it arches over the top with its ends on the nubs. The mop was also broken: the stick leaned opposite to its computed ends, so its foot floated outside the bucket and the grip hung 0.36 m away. Now the stick rises from the head in the bucket, leans out, and the grip ring sits around its top.
- placeTrafficCones, white band stands 1.5–2 cm proud: **FIXED**. Flipped (`rotation.x = π`, narrowing upward) and scaled (0.654, 0.8, 0.654) to the measured body taper.
- placeSmallLadders, paint-pail bail sideways: **FIXED**. `rotation.z = 0`, y 0.12 (ends on the rim).
- placeSmallLadders, lean goes into the wall / over open floor: **FIXED**. Rotation order 'YXZ' so the lean is in the ladder's own frame (the XYZ lean always tipped toward world -z). Feet stand rH·sin(lean) + 0.012 out, so the rail tops just touch the wall.
- placeCoatHookRows, pegs hang in front of/below the rail, never touching it: **FIXED**. Each peg's base is in the rail's front face, knob end up 30° (`rotation.x = π/3`, position (x, 0.0125, 0.0367)). The optional coat is re-hung with its collar over the peg and its back clear of the wall.
- placeUmbrellas, stands handle-down, tip up: **FIXED**. Shaft, canopy and handle flipped end for end (tip on the floor, handle on top). It now leans 15° toward the wall (order 'YXZ'; it used to lean sideways onto nothing), feet 0.232 out so the handle touches the wall, and the random ± is kept as a small sideways sway.
- placeSuitcases, handle a sideways C half sunk in the body: **FIXED**. `rotation.z = 0`, y 0.286 (end blocks on the lid).
- placeSuitcases, nameplate 6 mm proud: **FIXED**. z 0.076 → 0.0715.
- placeSuitcases, suitcase standing in the pool (level 4 seed 7): **NOT FIXED** in the placer. It's a level-placement issue, now handled by the central placement pass (its pool check).
- placeBackpacks, straps face into the body: **FIXED**. `strap.rotation.y = π`, padded face outward and thickness into the pack.
- placeBackpacks, handle a sideways C: **FIXED**. `rotation.z = 0`, y 0.36.
- placeSmallJewelryBoxes, open lid ~5 mm behind the box (logged as OK): **FIXED**. The lid pivots on the box's back top edge: position (0, 0.078, -0.0305).
- placeCoatHooks, tips hang 4 mm below their posts: **FIXED**. Tips rise from the posts' front ends, (hx, 0.0315, 0.0357), ball end up.
- placeShatteredVases, broken-rim tabs stick out as radial fins: **FIXED**. `rotation.y = π/2 - a` (width along the rim), radius 0.028, so the outer face is flush with the wall.
- placeOverflowingAshtrays, filters offset from their papers: **FIXED**. Filter offset `(cos a, -sin a)·0.010`, on the paper's axis.
- placeOpenedSuitcases, lid centred on the hinge, back half behind the case and in the floor: **FIXED**. `lidBox` offset (0, bh/2, bd/2) in the lid group so the group pivots on the hinge edge; the hinge now sits on the base (y bh, was a 5 mm gap).
- placeOpenedSuitcases, brass studs poke 4 mm past the sides: **NOT FIXED**. Not clearly a bug: they read as brass corner caps/feet. Moving them fully inside hides them; making them feet would need restacking the whole case.
- placeBrokenPictureFrames, backing under the floor: **FIXED**. The backing plane is turned to face out of the back and now sits on top.
- placeBrokenPictureFrames, frame face-up, not face-down as commented: **FIXED**. `frame.rotation.x = +π/2`.
- placeForgottenWristwatches, both straps start ~1 cm inside the case: **FIXED**. Each strap starts at a lug on the case edge (6 and 12 o'clock) and curls away. The right strap uses order 'YXZ' so it lies flat and then turns half round.
- placePorcelainCupSaucers, handle gap faces away, loop buried in the cup: **FIXED**. `rotation.z = -2.2` (gap and both ends on the wall), `rotation.y` 0 (radial plane).
- placePorcelainCupSaucers, pattern band floats off the wall: **FIXED**. `band.scale` 0.935, just outside the bellied wall. It clears the cup's facets without floating.
- placeAlarmBells, dome an upturned bowl half in the plate: **FIXED**. `rotation.x = π/2`: the gong faces the room, apex at the nut.
- placeAlarmBells, nut floats 2 cm in front: **FIXED** (by the gong fix; the nut sits on the apex).
- placeAlarmBells, clapper dangles outside / through the shell: **FIXED**. The pivot hangs from the centre nut (z 0.080), clear of the gong.

## Totals

- 69 FIXED lines. Unlogged part-to-part bugs fixed along the way: typewriter lever, key hole marker, easel back leg and brace, desk pencil buried/offset, desk text-line yaw, metal-bucket bail vs ears, pegboard handle/shaft gaps, lantern chain, cleaning-mop stick/grip, carpet roll3, coat re-hang.
- 9 NOT FIXED lines:
  - 4 already handled by the new parts: pawn/bishop gaps, king's cross, typewriter body/deck, easel chalk.
  - 1 skipped: the 11 b06 wall-bug placers, fixed centrally.
  - Typewriter platen: reads as seated in the carriage.
  - Desk wire rack: needs new hanger geometry.
  - Suitcase in the pool: level placement, handled by the central pass.
  - Opened-suitcase studs: not clearly a bug.
- The school-desk chair back counts as FIXED (gap closed), but back uprights would need new geometry.
