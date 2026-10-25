# fixes-A: placement bugs logged in b01, b02, b04, b05

Every change is in `backrooms.js`, inside the named placer. No `new THREE.XGeometry(...)` text or `PG(...)` wrapper was touched, and the random draws, probability gates, materials, animators and tracked positions are unchanged. Checked with `node --check`, `node build-standalone.mjs backrooms`, `tools/prop-gallery.mjs` (levels 2, 3; seeds 3, 5, 9) and a bounds dump per part. Shots are in `shots/fix/a_*.png`.

`+` marks an extra bug found in the same placer while fixing a logged one.

## b01

- placeWallVents: NOT FIXED. Not a bug: the note says the opening plane showing through the louvres reads right.
- placeDroppedShoes: FIXED. The toe cap moved from x 0.12 to 0.10, so it ends flush with the sole's front (0.14) instead of overhanging it by 2 cm.
- placeWallSockets: FIXED. The hole planes were 2 mm inside the wall. They now sit 4.5 mm behind the plate face, inside the modelled slots, where they show as dark slot bottoms.
- placeCrackedWallHoles: FIXED. The hole plane was 1 mm inside the wall. It now sits 2.5 mm behind the rim face, inside its dish, scaled to 0.66 so it stays within the torn outline. The dark hole now shows.
- placeFireExtinguishers: FIXED. `rotation.y = atan2(dz, -dx)` turns the bracket side (-X) to the wall and the nozzle (+X) to the room. The wall offset changed from 0.13 to 0.165, so the bracket plate sits on the wall.
- placeExitSigns: FIXED. The glow offset had the wrong sign; the glow is now 1 mm in front of the housing and the EXIT text shows. The housing also moved flush to the wall (it was 1 cm off).
- placeThermostats: FIXED. The dial is now 0.5 mm in front of the housing face instead of behind it. The housing moved flush to the wall (it was 5 mm off).
- placeMotionSensors: FIXED. Height changed from WALL_H-0.02 to WALL_H-0.01, so the base top touches the ceiling.
- placeEmergencyLights: FIXED. Added the missing `world.add(light)`. The housing is now flush (offset 0.055 to 0.05). + The 10 % point light used the wrong forward sign and sat 10 cm inside the wall; it now uses `(sin, cos)` of the yaw.
- placeWetFloorSigns: FIXED. Each leaf now leans ±15° about X instead of Z. The tops meet, the feet are 22 cm apart, the rear leaf is turned π so both fronts face out, and the feet rest on the floor.
- placeHandrails: FIXED. Brackets are centred 3.3 cm off the wall and scaled 1.1 along the wall normal, so each runs from the wall to 6 mm into the rail. On X-facing walls they are turned π/2, so the thin side runs along the rail.
- placeCeilingSpeakers: FIXED. The group moved to WALL_H-0.01, so the trim touches the ceiling. The ring decal moved to y -0.0155, 1 mm under the grille face instead of inside it.
- placeDeadVendingMachines: no problem logged.
- placeCeilingCables: FIXED. The shaft top is now 3 mm into the ceiling, corrected for the tilt with cos. The plug sits on the tilted cable end with the cable 4 mm inside it, and the tracked plug y follows.
- placeDeadCeilingLights: FIXED. The co-located cable's plug is offset by sin(tilt)·len/2, so it sits on the tilted cable end, and the cable top is in the ceiling. + The canopy moved up 2.6 mm so its back touches the ceiling; the socket and bulb moved with it.
- placeCeilingCracks: FIXED. Each chunk's height now comes from its rotated bounds, so its lowest corner rests on the floor (settled in by 2.5 mm at most, using the old jitter draw).
- placeOverturnedChairs: FIXED. y changed from 0 to 0.17, so the tipped chair's seat edge rests on the floor.
- placeOverturnedWastebaskets: FIXED. `rotation.z` changed from π/2 to -π/2. The wide mouth (r 0.11) now faces the trash at +X, and the narrow base (r 0.09) meets the base disc.
- placeCrushedSodaCans: FIXED. Removed the `rotation.z = π/2` tip. The crushed can now lies flat on its base at y 0.00525.
- placeWallThermostats (minor): FIXED. The knob moved from z 0.014 to 0.013, onto the body face.
- placeElevatorButtons (minor): NOT FIXED. The note calls the plate back 7 mm into the wall harmless.
- placeFireAlarmPulls (minor): FIXED. The handle moved from z 0.008 to 0.005, onto the body face.
- placeCrushedWaterBottles (minor): FIXED. The cap moved to the narrow neck end (x -0.034, `rotation.z` -π/2 so its closed top faces out). The label is scaled 1.08 across its width so it wraps the outside of the body. + The bottle was lifted 4 mm because the body sank.

## b02

- placeEmergencyStrobes: FIXED. The group origin is now 0.5 mm off the wall (the base back was 1 cm off). + The dome's `rotation.x` changed from -π/2 to +π/2; the red dome had pointed into the wall.
- placeEmergencyPhones: FIXED. The handset moved to z 0.016 and the cups to 0.018, so they rest on the box face. The box moved out flush (CELL/2-0.09) and is no longer half inside the wall.
- placeSprinklerHeads: FIXED. `sprY` changed from WALL_H-0.02 to WALL_H; the stub top had been 2 cm below the ceiling plane.
- placeMotionDetectorDomes: FIXED. `domeY` is now WALL_H-0.005 and the base sits at domeY+0.0025. The base top touches the ceiling and the dome top meets the base.
- placeWallOutlets: FIXED. The cord's upper end is now 4 mm inside the plug's bottom centre, placed from the tilt. + The plug moved onto the plate face (it was 2 mm off).
- placeExtinguisherCabinets: FIXED. The cap moved from y 0.076 to 0.070, onto the cylinder. The cylinder and cap moved from z -0.02 to -0.034, behind the glass. The half-buried cabinet was left, since the note says semi-recessed is fine.
- placeCeilingFans: FIXED. The whole fan moved up 2 cm so the downrod top reaches the ceiling (the gap had been 2 cm).
- placeSwitchPlates: FIXED. The wall offset and facing were already fixed centrally and look right in the gallery. Fixed here: screw z changed from 0.008 to 0.0028, so the heads sit on the plate face and the shanks are inside it.
- placeSmokeDetectors: FIXED. `ceilY` is now WALL_H; the puck top had been 2 cm below the ceiling.
- placeExitArrowSigns: NOT FIXED here. The logged wall offset and facing were already fixed centrally and look right in the gallery. + The arrow shaft moved from z 0.010 to 0.008, onto the panel (it was 2 mm off).
- placeFrostedWindows: NOT FIXED here. It was already fixed centrally; the gallery shows it flush and facing the room.
- placeLeaningFrames: FIXED. The lean was about the frame's centre. It now pivots on the bottom-back edge: that edge rests on the floor and the top-back edge touches the wall. + The frame now uses Euler order YXZ; with XYZ, frames on X-facing walls spun in their own plane instead of leaning.
- placePinnedPhotos: FIXED. The photo is now 2 mm off the wall and the pin is in front of it, flange on the photo. The pin had been behind the photo.
- placeWallDoors: no problem logged.

## b04

- placeCabinetDoors (toolbox handle): FIXED. The handle now lies folded on the closed lid (y 0.108). On the open variant it sits on the open lid's outer face instead of floating 10 cm up.
- placeCabinetDoors (wrench jaw): FIXED. The jaws are now side by side in z, in the plane of the lying wrench, instead of stacked in y. The wrench y is 0.010, so it rests on the knurl.
- placeFirstAidKits (trim): FIXED. y changed from 0.10 to 0.08, so the trim is the top strip of the front face.
- placeFirstAidKits (goggle rims): FIXED. The rims now use `rotation.x = -π/2` and lie flat around the lenses. The strap moved behind the rims. + The bottle, syringe, clipboard and goggles were all placed toward the wall (`fx = -sin`); the sign is flipped so they land in the room. + The kit is now flush (it was 5 mm off).
- placeWallPhones (off-hook coil): FIXED. The three coil pieces now form one cord from the jack under the base to the handset: `scale.z` 3, half-turn offsets, spaced by the pitch. The handset hangs from the end of the cord. + The body is now flush.
- placeBriefcases: FIXED. The lid now reaches forward from its back-edge hinge (z 0.14 in the lid group) and swings up behind the case. The handle is re-parented to the lid. The tipped variant's y changed from 0.14 to 0.18. + The handle was lowered onto the rim (it floated 6 mm).
- placeAbandonedLaptops (open): FIXED. The lid is placed as if closed, forward of the hinge, so the -105° swing stands it up behind the keyboard. The screen now lies on the lid's inner face. + The inline briefcase handle was lowered too.
- placeSmallKeys: FIXED. The bitting is laid flat (`rotation.x = -π/2`) at the blade's side near the tip, instead of hanging 4 mm below it.
- placeWalkieTalkies: FIXED. y changed from 0.015 to 0.03, the rotated half-width.
- placeHandbaskets: FIXED. The handles are no longer rotated; each half-ring arches up from the rim of a long wall (z ±0.075). This matches the modelled pivot hubs.
- placeSmallChessPieces: FIXED. Tipped pieces now sit at y 0.024 (was 0.020), the base radius.
- placeSmallDominoes (minor): FIXED. The divider now sits at y 0.0065 and the pips at 0.0062, on the tile top.
- placeTypewriters: NOT FIXED. The key-block tilt is reversed, but the note says it reads fine; not clearly a bug.

## b05

- placeMetalLockers: FIXED. The wall offset changed from 0.15 to 0.30, so the 0.30-deep body backs onto the wall instead of half into it.
- placeDroppedMailScatters: FIXED. The base y changed from 0.003 to 0.0005, so the bottom envelope lies on the floor. The address decal's offset now follows the envelope's long axis. Its rotation already followed the yaw: Euler XYZ applies z before the flattening x.
- placeKeychainKeys (bows): FIXED. `rotation.x = π/2` (was `rotation.y`), so the bows lie flat.
- placeKeychainKeys (tooth): FIXED. The tooth is laid flat beside the blade edge near the tip, instead of 4 mm above the blade.
- placeSmallRadios: FIXED. The antenna's lower end is now 2 mm into the case top, placed from the tilt.
- placeWallPayPhones: FIXED. The handset moved from x -0.06 to -0.02, inside the housing width, and from z 0.024 to 0.016, over the cradle. The 270° cord loop now lies in the face plane with its gap up, so both ends meet the handset's underside. The key and cradle gaps were already solved by the model.
- placeDroppedCellPhones (keypad dots): NOT FIXED. A closed flip phone has no visible keypad, so showing one needs an open-flip variant (new composition). The dots are hidden and harmless.
- placeLanyards: FIXED. The strap now runs along X from the loop's open end to the badge. The badge is turned so its slotted end meets the strap, and the ink lines turn with it.
- placeMailSlots (minor): FIXED. The flap moved 1 mm out, so its back clears the cavity plane; the hinge edge is still in the plate.
- placeTapeRolls: FIXED. The tail now trails from the roll's floor contact, across the axis (z 0.02), instead of 3 cm off along the axis.
- placeAnalogClocks: NOT FIXED here. The wall-prop note was fixed centrally and the gallery shows it flush. The hands-translate note is already handled by `PG(..., 1)`, which gives a private copy.
- placeFramedPhotos: NOT FIXED here. Fixed centrally; checked in the gallery.
- placeDiplomas: NOT FIXED here. Fixed centrally; checked in the gallery.
- placeThermometers: NOT FIXED here. Fixed centrally; checked in the gallery.
- placeDiceClusters: FIXED. The dice now rest on a face: x flips 0/π and z takes quarter turns, using the same three random draws. The pip decal is turned with the die's yaw.
- placeCDs (trivial): FIXED. All parts moved up 0.5 mm, so the underside disc sits on the floor.
- placeRemotes (IR circle 1 mm over the nose): NOT FIXED. It is a decal offset against z-fighting and purely cosmetic.
- placePagers: FIXED. The body, screen and buttons moved up 2 mm, so the pager lies on its belt clip instead of the clip being inside the body.
- placeCalculators: FIXED. The solar strip moved from z -0.038 to -0.0395, clear of the LCD, so they no longer z-fight.
- place8TrackTapes: NOT FIXED. Nothing to do; the new body model already removes the z-fighting.
- placeReadingGlassesAndBooks: FIXED. The bookmark is tucked in the gutter at the top edge with its tail drooping to the floor. The glasses moved to y 0.002 and offset 0.15, clear of the book. The temples fold flat from the hinges across the rims.

## Count

71 lines:

- **57 FIXED.** Lines marked `+` also fix an extra bug in the same placer.
- **12 NOT FIXED:**
  - 6 were already fixed centrally (the wall-prop offset): FrostedWindows, ExitArrowSigns, AnalogClocks, FramedPhotos, Diplomas, Thermometers.
  - 5 are not bugs or are harmless: WallVents, ElevatorButtons, Typewriters, Remotes IR decal, 8TrackTapes.
  - 1 would need a new variant: the DroppedCellPhones keypad.
- **2** placers had no problem logged.
