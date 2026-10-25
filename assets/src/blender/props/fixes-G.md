# fixes-G: placement bugs logged in b21, b22, b23, b24

Every change is in `backrooms.js`, inside the named placer. No `new THREE.XGeometry(...)` text or `PG(...)` wrapper was touched (`prop-codemod --check`: only the known "not built" lines). Random draws (count and order), probability gates, materials, animators and tracked positions are unchanged; where a draw's use moved (hat angle, nest tilt, bud turn, leaf turn) it is still drawn at the same point. Checked with `node --check`, `node build-standalone.mjs backrooms`, `tools/prop-gallery.mjs --batch b21..b24` (`shots/fix/g_b21.png` … `g_b24.png`, private builds with the batch parts) and narrow-fov close-ups with the parts applied (`shots/fix/g_hs1.png`, `g_b21a/b.png`, `g_b22a/b.png`, `g_b23a.png`, `g_b24a/b.png`, `g_tools.png`, `g_misc.png`).

`+` marks an extra bug found in the same placer while fixing a logged one.

## b21

- placeVictorianHallStands (mirror): FIXED. The frame torus and the glass are no longer rotated: both stand upright in the mirror group's XY plane, whose +Z faces radially out, so the glass faces out with its frame. The group moved from r 0.055 to 0.033, so the frame's back touches the post.
- placeVictorianHallStands (umbrella crooks): FIXED. Each crook is an upright half-ring turned `rotation.y = π − uAngle`. It is placed from the shaft's real top (the shaft's +Y end through its Euler), so the crook's thick stick end sits on the shaft and the crook curls radially out.
- placeVictorianHallStands (hats): FIXED. The hat angle snaps to the nearest upper hook (same random draw). The brim rests on the peg (arm top, y 0.020 + hookYs[0] + 0.004) with the brass knob inside the crown. The top-hat crown stands on its brim; the bowler dome and brim sit at brim height.
- placeServantHallBellRacks (bell reads as a cup): SKIPPED (handled by model). The part is a mouth-down bell in the same bounds.
- placeServantHallBellRacks (springs float 1.5 cm over the caps): FIXED. The six 60° arcs now overlap into one 210° curl across the board (YZ plane: `rotation.y −π/2`, `rotation.z −π/3 + s·π/6`, centre y 0.054, z 0.0195). The curl runs from the board face at the top round to the bell cap. + The bells and caps moved from z 0.020 to 0.025; each bell's back was 4 mm inside the board.
- placeServantHallBellRacks (trims flush, 9 mm into the wall): FIXED. z −0.005 → +0.004. The backs are on the wall and the fronts stand 9 mm proud of the board.
- placeCastIronDoorStops (dog legs buried): SKIPPED (handled by model). The body part is modelled lying down with its front paws.
- placeCastIronDoorStops (boot laces on the +Z side): FIXED. The laces now cross the shaft's toe-side (+X) face: position (0.0095, y, 0), `rotation.x π/2`.
- placeCastIronDoorStops (toe 4 mm into the base): FIXED. y 0.020 → 0.0245, so the toe rests on the plate.
- placeCorrespondenceCards / placeEmptyPaintingOutlines / placeHeraldicCrests / placeOrnateWallMirrors / placeSmallPocketWatches: no problem logged.
- placeChessSideStands (board 5 mm into the top): FIXED. `boardTop` is the stand top's surface, so the board sits at boardTop + 0.005 and the squares at + 0.0105.
- placeChessSideStands (pieces float 4 mm): FIXED. Pieces stand at boardTop + 0.0106, on the squares.
- placeChessSideStands (ashtray floats 6 mm): FIXED. The tray is on the top (boardTop + 0.002). The cigar lies across it in the rim notches (+0.0075) with the ember at its lit end, and all three moved 7 mm inward.
  - + The snifter was lowered 6.5 mm onto the top.
  - + The top is scaled 1.15 in x/z (r 0.2185): the board's corners (r 0.212), the snifter and the tray all overhung the round top.
  - + The captured pawns lay off the top's edge; they now lie in a row along the board's +X edge (`rotation.x π/2`).
- placeChessSideStands (pedestal 10 cm short): FIXED. Scaled ×1.21 in y, so it runs from y 0.05 up to the top's underside.
- placeChessSideStands (tripod legs splay the wrong way): FIXED. Each leg is set by quaternion: the foot end sits on its pad at r 0.12 and the top leans 0.6 rad inward to the pedestal's foot.
- placeMiniaturePortraitClusters (frames stick out as horizontal hoops): FIXED. `rotation.x` removed: the ovals stand upright in the wall plane around the portraits, z 0.008 → 0.002, backs on the wall. The frame-tilt animator now tilts an oval in its own plane. + The hanging loop, also laid flat, now stands up with its ends in the frame's top.
- placePeacockFeatherVases (middle trim buried in the hip): FIXED. The hip ring is scaled to r 0.0765, onto the 0.075 hip.
- placePeacockFeatherVases (upper ring floated on the old frustums): SKIPPED (handled by model).
- placeBellowsCameras (tripod legs splay the wrong way): FIXED. Legs are set by quaternion from the feet (r 0.20), leaning 0.16 rad inward to meet in the hub. The two braces moved to the height where the legs are 14 cm apart, so each spans leg to leg.
- placeBellowsCameras (ridges stand as hoops across the bellows): FIXED. Rotations removed: the rings go round the bellows' Z axis, scaled with its taper (0.055 → 0.047).
- placeBellowsCameras (ground glass z-fought): SKIPPED (handled by model).
- placeBellowsCameras (cloth top floats 6 mm, flaps 2 cm off): FIXED. The top rests on the rear plate (camY + 0.074, scaled 0.88 wide). The side flaps hang against the plate's sides (x ±0.084), and the back flap hangs from the top's rear edge (z −0.116). + The thumbscrew moved 2 mm down onto the plate.
- placeBellowsCameras (flash rod hangs in mid air): FIXED. The rod is stretched from a clamp on the +X tripod leg (y 0.987) up to the T-bar. The T-bar, pan and powder moved to z 0, above it.
- placeBrassSconces (arm pieces float, parallel to the wall): FIXED. The two arm segments are stretched between points: plate boss → knop → under the drip cup. The knop sits at the joint.
- placeBrassSconces (candle foot 3 mm above the cup): FIXED. The candle, wick, flame and light moved down 5 mm (foot 2 mm into the socket). The rim ring moved 2 mm down onto the cup's lip.
- placeBrassSconces (chain hangs from nothing): FIXED. The three links hang from the knop, upright, each turned 90° to the last.
- placeOpenJournals (quill runs into the floor, vane off the shaft): FIXED. The shaft lies flat on the pages (`rotation (0, −π/6, −π/2)`, y 0.0065) along the vane's direction, and the vane covers its upper half.
- placeParchmentScrolls (ribbon hovers 3 mm): FIXED. The ribbon is scaled to r 0.021 across its axis.

## b22

- placeSmallCompasses (cover cuts through the case): FIXED. The hinge barrel runs along Z at the case's top edge (0.039, 0.011). The cover is the closed lid swung 120° about it: centre (0.062, 0.043), `rotation.z −2π/3`, dished face toward the dial.
- placeSmallTelescopes (eyepiece at the wide end): SKIPPED. The note calls it harmless.
- placeSmallGlobes (meridian spins in its own plane): FIXED. The sphere now sits in a group tilted `rotation.z 0.4` with the meridian, so its poles meet the pins. The spin animator still turns it about its own, now tilted, axis.
- placeSmallPhonographs (horn upside down): FIXED. The horn is set by quaternion: throat (apex) at the arm pivot on the cabinet's back corner, mouth up and back.
- placeSmallPhonographs (tone arm upright, buried): FIXED. The arm is stretched from its pivot on the cabinet top (0.076, 0.064, −0.068), low over the record, to the sound box.
- placeSmallPhonographs (sound box 4 cm from the arm): FIXED. The arm's tip now ends on the box.
- placeHangingBrassLanterns (panel corners outside the posts): FIXED. The panels sit 0.028/√2 from the axis, as the sides of the square whose corners are the posts.
- placeHangingBrassLanterns (chain stops 2 cm below the ceiling): FIXED. The chain is stretched 2.2 cm; its top is 2 mm into the ceiling.
- placeSmallTelegraphKeys (wire upright, half in the floor): FIXED. The wire is stretched from between the terminals, over the base's end, down to the floor.
- placeSmallScales (chains stop 11 mm above the pans): FIXED. Each chain run is one straight line (z 0) from inside the beam end down into its pan. The pans hang 1.1 cm higher (y −0.087).
- placeSmallScales (pans clip the base): FIXED. With the same change the pans clear the base by about 1.3 cm, even at the ±0.05 sway.
- placeSmallAbacuses (end beads 3 mm into the stiles): FIXED. The bead pitch was recomputed, so the end beads stop 0.5 mm short of the stiles. The bar-chamfer remark is a modelling note only.
- placeSmallTrophies (handles float beside the cup): FIXED. Each handle is a C in the XY plane (`rotation.z −s·(π/2 + 0.395)`) with both ends on the cup wall at x ±0.0365. The chord leans with the cup's flare.
- placeSmallHandbells (bell opens upward like a bowl): FIXED. `rotation.x π` removed, because the upper hemisphere already opens downward. The dome sits at y 0.005: mouth down, crown under the collar and handle, clapper inside. The lying variant rests on its rim.
- placeSmallMasks (strap across the mouth): FIXED. The strap moved to (0, 0.008, −0.01), under the face-up plate at eye level; its ends show at the temples.
- placeCandleClusters: no problem logged.
- placeSmallCrystalBalls (prongs twist like a pinwheel): FIXED. `rotation.set(0, π − a, 0.4, 'YXZ')`: each prong leans radially out, its hooked tip curling toward the ball.
- placeSmallPumpkins (face decals inside the body): FIXED. Each decal sits 1 mm outside the body ellipsoid at its (x, y) and faces along the surface normal.
- placeSmallWitchHats (crown pokes out of the band): FIXED. The crown now tilts about its base, not its middle: position (−sin·0.07, 0.006 + cos·0.07).
- placeSmallSkeletonHands (thumb zig-zags): FIXED. The two thumb bones lie flat, end to end along (−sin 0.9, 0, cos 0.9) from the palm's side.
- placeSmallCauldrons (round bottom 3.7 cm below the leg tips): FIXED. The pot (body, bottom, rim, brew, glow) is lifted 4 cm, so it stands on its legs, 3 mm clear of the floor.
- placeSmallCauldrons (bail laid flat): FIXED. Rotations removed: an upright arc over the pot, with its eye ends in the rim at x ±0.055.
- placeSmallTombstones (half-disc stands as a D): FIXED. `rotation.y π/2` comes before the x turn, so the half-disc spans the full width as an arch.
- placeHangingBats / placeSmallGhostSheets / placeSmallPotions / placeSkullStacks: no problem logged.
- placeSmallSpiders (legs mirrored by position only): FIXED. Each leg is built from points: hip under the body, knee 20° up and out, foot on the floor. The direction is mirrored per side.
- placeSmallSpiders (feet 3 mm above the floor): FIXED. With the same change the feet end at y 0.001.
- placeSmallBrooms (twigs fan vertically, 8 cm into the floor): FIXED. The stick lies on the floor at the handle end and is raised to 0.020 at the bristles. The twigs run by quaternion from roots in the binding to tips on the floor, in a flat ±0.45 rad fan (random draws unchanged).
- placeSmallBlackCats (ears buried): FIXED. The ear bases sit 2 mm into the head, 0.45 rad off the crown, leaning 0.3 rad out; the whole ear shows.
- placeSmallTreatPails (face decals inside the body): FIXED. Same as the pumpkins: 1 mm outside the body ellipsoid, facing its normal.
- placeSmallTreatPails (handle flat above the rim): FIXED. Rotations removed: an upright arch, narrowed ×0.945 so its pivot ends meet the rim's sides at mid-height.
- placeSmallTreatPails (stem in the candy opening): SKIPPED (handled by model). The rim part has a centre boss under the stem.
- placeSmallSkulls (teeth inside the jaw front): FIXED. The teeth sit on the jaw's top edge (y 0.0225), following its horseshoe front (z 0.024 at the chin curving back), each turned to face out.
- placeSmallOwls (tufts buried): FIXED. The tuft bases sit 2 mm into the head, 0.45 rad off the crown, leaning 0.35 rad out.
- placeSmallOwls (wing triangles inside the body): SKIPPED. These are single-sided BufferGeometry triangles, one facing +Z and one −Z. The body part already models folded wings, and pulling the triangles out would show flat one-sided shards.
- placeSmallOwls (eye rings overhang the head): SKIPPED. Minor: flat decals on the part's flat facial disc.
- placeBrassCandlesticks (wax pool hovers 4–5 mm): FIXED. The pool sits on the domed top (y 0.0097), scaled 0.81 to stay inside the cove. + It faced down (the fan winds clockwise seen from above), so it never showed from above; it is flipped with `rotation.x π`.

## b23

- placeWallCrucifixes / placeStainedGlassWindows / placeCrystalSkulls / placeOuijaBoards / placeTarotCardFans / placePentagramChalks / placePottedPlants / placeFlowerStems / placeLeafLitter / placeTwigs / placePineCones / placeFeathers: no problem logged.
- placeReliquaryBoxes (medallion half buried): SKIPPED. The note says it is unchanged from before; it reads as inset.
- placeSmallCandles (drip half inside): SKIPPED. The note says placement is OK.
- placeHangingCensers (chains stop 2 cm below the ceiling): FIXED. The top anchors are 2 mm into the ceiling.
- placeHangingCensers (chain ends 3–4 mm off the lid): FIXED. The bottom anchors are on the rim bead (r 0.035, bowlY + 0.002).
- placeReligiousTriptychs (canvas covers the dividers): FIXED. The canvas is recessed 8 mm behind the frame face (z t/2 − 0.008), so the frame and the two dividers stand proud of the panels.
- placeMemorialShrines (holders buried in the step): FIXED. The holders and candles moved to z 0.068, on the platform in front of the step.
- placeMemorialShrines (portrait 2 mm in front of the frame): FIXED. The frame, portrait, silhouette and plate share one leaning group. The portrait lies in the frame's sunk centre (+0.001), the silhouette on it, and the plate on the bottom moulding.
- placeMemorialShrines (cross floats 2.5 mm, no foot): FIXED. The cross is lowered 2.5 mm onto the platform. A foot would need a new part. + The Star of David is lowered 1.3 cm; it floated.
- placeMemorialShrines (buds off their stems): FIXED. Each stem's foot is inside the vase (platH + 0.030), leaning out of the mouth, and the bud sits on the stem's tip along its axis. The old bud-z random draw now turns the bud.
- placeMemorialShrines (ribbon tail in the platform and the floor): FIXED. The tail continues the lying ribbon over the platform's right edge and slants 26° down to the floor. The sway animator still moves it ±3 mm.
- placeDressForms (legs splay the wrong way): FIXED. Legs run by quaternion from the feet (r 0.19), leaning 0.548 rad inward into the hub, stretched 10 %.
- placeDressForms (pole wheel vertical): FIXED. `rotation.x π/2`: horizontal round the pole, like the neck wheel.
- placeDressForms (torso gaps): FIXED. The waist is scaled y 0.35 → 0.72 (at +0.199) and the bust 0.50 → 0.577 (at +0.35), so hip, waist, bust and shoulder meet.
- placeDressForms (drape floats in front of the bust): FIXED. The drape stands against the bust's front (`rotation (π/2, 0, 0.15, 'ZXY')`, z 0.113), narrowed ×0.75. The flowing strip hangs from its right edge, and the pin heads sit on its face. The sway animator is unchanged.
- placeDressForms (seam strips straight, buried/outside): SKIPPED. A straight 2 mm strip can't follow the torso profile; this needs a curved seam part.
- placeSkullPlaques (oval plaque round, 14.7 mm thick): FIXED. `scale.y` → `scale.z`, the rotated cylinder's axis that is vertical in the wall plane: now 22 cm tall and 12 mm thick.
- placeSkullPlaques (raven beak sideways, detached): FIXED. `rotation.y −π/2` added: the apex points out of the plaque, 0.1 rad down, with the hook down, and the base is in the cranium's front. The seam is stretched along the underside from base to tip.
- placeSkullPlaques (antlers lean in, forks detached): FIXED. The beams lean out (a V) from bases 2 mm into the cranium top. The forks are tines leaving the beam three-quarters of the way up, pointing up and forward.
- placeSkullPlaques (eye sockets inside the crania, facing +X): FIXED. Each socket sits 0.5 mm outside its cranium ellipsoid on the side, facing the normal. The raven's sockets moved to its braincase (z 0.030).
- placeSmallCandelabras (arms nearly vertical, side cups float): FIXED. `rotation.z = atan2(dy, dx) − π/2`, so each arm runs from the column to its cup. + The centre cup moved down onto the column top (0.157); it floated 6 mm.
- placeSmallChainLinks (chain can't lie flat): FIXED. The on-edge links stand on the floor (centre R + r) and the flat links rest across their lower wires (y 0.0155), at a 1.25 R pitch, so every link threads its neighbours and nothing goes into the floor.
- placeSmallCrows (eyes buried): FIXED. The eyes sit on the head's surface, 18 mm out along (0.45, 0.25, ±0.86).
- placeSmallCrows (wing triangles inside the body): SKIPPED. These are single-sided BufferGeometry triangles; the −Z wing faces inward, so it would not show if pulled out. The body part models the wing bulges.
- placeSmallHourglasses (top bulb not flipped, sand floats): FIXED.
  - The top bulb is flipped, and both bulbs are stretched ×1.233 to meet apex to apex at the waist (0.080) and reach the plates.
  - The upper sand is turned point-down into the upper funnel (narrowed ×0.6 to stay inside the glass).
  - The lower sand is a heap on the bottom plate; it was inverted.
  - The stream runs from the waist to the heap's crest (×1.53), and the animator's base height is updated.
- placeDeadPlants (stems tilt tangentially): FIXED. Stems are set by quaternion, tilted radially outward.
- placeDeadPlants (stems start above the soil): FIXED. Feet are 1–3 cm down in the soil (same random draw).
- placeDeadPlants (tip leaves float): FIXED. Each leaf sits on its stem tip with the stem's orientation. The old independent-height draw now turns the leaf about the stem.
- placeDeadFlowerVases (stems hover over the neck): FIXED. The stems start at y 0.080, 1 cm down inside the neck; the petals follow.
- placeDirtClods / placePebbles / placeMushrooms (items interpenetrate): SKIPPED. This is random scatter, not a part-to-part bug.
- placeAcorns (stalk inside the cap, nut sinks 2 mm): SKIPPED. The cap part's stub stands in for the stalk, and sinking is handled centrally.

## b24

- placeDandelions (wisps don't radiate, not attached): FIXED. Wisps are set by quaternion from the puff's centre, achene 1 mm inside the puff, at elevations −0.3 to 0.6 rad.
- placeMossPatches / placePineNeedles / placeGardeningGloves / placeSoilBags / placeGardenAprons / placeFertilizerSpikes / placeDeadPottedPlants: no problem logged.
- placeGrassClumps / placeButterflyWings / placeSpiderWebs / placeSeedPackets: untouched by the modeller.
- placeFerns (stem taper reversed): SKIPPED (handled by model). The part is thick at the frond's base inside the same frame.
- placeSnailShells (lip flat, half buried): FIXED. The lip is an upright C (`rotation (−0.52, −π/2, −π/2)`, scale 0.83) bulging out of the body whorl. Its lower end is in that whorl and its upper end leans against the third whorl.
- placeBirdNests (twigs alternate tangent/radial): FIXED. `rotation.set(0, π/2 − a, π/2 + tilt, 'YZX')` lays every twig tangent to its ring. The old random x-wobble is now an up/down weave along the ring.
- placeBirdNests (eggs hover 1.1 cm): FIXED. y 0.024 → 0.013, resting in the hollow.
- placeChestnuts (spikes don't radiate): FIXED. Spikes run along the nut ellipsoid's normal from 1 mm inside it, at two alternating latitudes.
- placeHangingPlants (chain ends 6 mm above the pot): FIXED. The chain is stretched ×1.17, from the bracket eye into the pot.
- placeSeedlings (dirt 1 mm above the rim): SKIPPED (handled by model). The soil part is lowered at its edge.
- placeTrowels (scoop on edge, 1 cm into the floor): FIXED. `rotation (π, 0, π/2)`: the hollow faces up, the width runs across, and the belly rests on the floor (y 0.020).
- placeTrowels (tip cone points back): SKIPPED (handled by model). + The round tip reached 5 mm under the floor: it is flattened (×0.35) and lies on the floor, continuing the scoop.
- placeWateringCans (spout rotated the wrong way): FIXED. `rotation.z +π/3`: the narrow end points out and up, and the flanged end is in the body.
- placeWateringCans (rose floats): FIXED. The rose sits on the spout's end with its face out along the spout. The hole dots are on that face, facing the same way.
- placeWateringCans (handle across the can): FIXED. The handle is upright in the spout's plane, with both ends set into the can's top.
- placePruningShears (tips swapped vertically; blades flat): FIXED. The shears lie flat, sprung open about a vertical pivot bolt (scaled 0.75). The blade and hook open ±0.15 rad in the floor plane from their roots at the bolt. The tips sit on the blade ends, turned with them and flattened ×0.45 to the blade's thickness.
- placePruningShears (tip cones point back): SKIPPED (handled by model).
- placePruningShears (handles don't join the blades): FIXED. The handles start at the bolt and open ±0.2 rad.
- placeGardenHoes (head 11 mm beyond the collar): FIXED. The head moved to x −0.282 (1 mm into the collar) and lies on the floor (y 0.003).
- placeGardenRakes (bar 7.5 mm beyond the collar): FIXED. x −0.307, against the collar.
- placeGardenRakes (tines in front of the bar): FIXED. The tine roots are set 1 mm into the bar's top.
- placeGardenRakes (tines half up, half in the floor): FIXED. The rake lies on its back: the tines rise from the bar, 0.3 rad forward (`rotation.z 0.3 − π`).
- placeGardenHoses (four separate rings, not a coil): FIXED as far as closed tori allow. All four turns lie flat on the floor, pressed against each other (1.4 cm pitch for a 1.6 cm hose), with their stripes turned, so they read as one flat coil. A true spiral would need a new part.
- placeGardenHoses (nozzle across the outer loop): FIXED. The nozzle runs along +Z, tangent to the outer turn, with its coupling on the hose; the spray tip is on its front.
- placeGardenShovels (blade 1 cm beyond the collar): FIXED. x −0.308, lying on the floor (y 0.004).
- placeGardenShovels (tip pyramid 3 cm under the floor): FIXED. The apex points outward (−X) and the pyramid is squashed (`scale.z 0.12`) into a flat triangular point in the blade plane.
- placeGardenShovels (D-grip upright, detached): FIXED. The D lies in the blade plane (`rotation (−π/2, −π/2, 0, 'YXZ')`). Its chord crosses the handle's end, narrowed ×0.6 so its ends meet the shaft, and the grip bulges past. The two side bars from the comment would need a new part.
- placeKneelingPads (cutout z-fight): SKIPPED (handled by model).
- placeTwineBalls (end sticks straight out, hovers 9 mm): FIXED. The end leaves the ball's lower flank and slopes down to its frayed end on the floor.
- placeSeedCatalogs (page 2 mm in front of the frame): FIXED. z 0.008 → 0.0025, on the recessed backing board under the sight edge.
- placePressedFlowers (paper 1 mm in front of the frame): FIXED. z 0.010 → 0.002, on the mount at the bottom of the well.
- placePressedFlowers (glass 4 mm in front): FIXED. z 0.013 → 0.0058, inside the box behind the sight lip.
- placeBirdFeeders (loop flat, 8 mm over the cap): FIXED. The loop stands upright, its ends 4 mm into the cap's hanger boss.
- placeBotanicals (paper 2 mm in front of the frame): FIXED. z 0.009 → 0.0015, on the backing board.
- placeGardenGnomes (nose and beard on opposite sides): FIXED. The beard moved to the face side (−Z), with the nose.
- placeGardenGnomes (beard inside the head and body): FIXED. The beard hangs from the chin, its top under the nose, down over the chest, tipped 0.2 rad forward (z −0.026).
- placeGardenGnomes (boots under the robe): FIXED. The boots moved to z −0.016, x ±0.009, so the toes peek out from under the hem.
- placeHerbBundles (hook 3 cm above the binding): FIXED. The hook moved down to the twine binding, with its rosette on the wall (it was also 9 mm off the wall).

## Count

127 lines for logged bugs, plus 6 grouped "no problem logged / untouched" lines:

- **108 FIXED** (b21: 23, b22: 30, b23: 24, b24: 31). Lines marked `+` also fix an extra bug in the same placer.
- **19 SKIPPED:**
  - 11 are already handled by the new parts or centrally: bell shape, dog legs, peacock upper ring, bellows ground glass, treat-pail stem, fern taper, seedling dirt, trowel tip, shears tips, kneeling-pad cutout, and the acorn stalk (its nut sinking is handled centrally).
  - 5 are not bugs or are harmless: telescope eyepiece, owl eye rings, reliquary medallion, candle drip, and the clods/pebbles/mushrooms scatter.
  - 2 are the one-sided BufferGeometry wing triangles (owl, crow). The body parts already model the wings, and pulling the triangles out would show flat one-sided shards.
  - 1 would need a new model: the dress-form seams.
- A foot for the shrine cross, a true hose spiral and the shovel's D-grip side bars would also need new parts; this is noted in their FIXED lines.
