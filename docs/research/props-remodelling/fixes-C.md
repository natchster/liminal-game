# fixes-C — part-to-part bugs from b03 / b09 / b10 notes

All edits are small in-place changes inside the named placers.
No primitive and no `pg(...)` wrapper was changed, and
no new primitive was added: where a missing piece was added, it reuses an
existing geometry object. Randomness, the order of random calls, gates,
materials, animators and tracked arrays are all unchanged (animator base
values were moved along with their meshes).

## b03 — place_props

- Exit door / sign plane z-fighting on the housing front — SKIPPED (no code needed: the housing part's 6 mm recess already fixes it)
- Exit door / mounting hooks buried in the housing — FIXED: the hooks are turned along z (rotation.x π/2), stretched to 0.16 m (scale.y 2.67) and centred at z 0.02. They are now standoff rods from the wall face (z −0.06) to the housing back (z 0.10) at the top corners.
- Battery / drawer-content battery has no tip — FIXED: added the + terminal with the shared `batTipGeo`/`batTipMat` (no new constructor), on the body's +Y end (x −0.105, rotation.z π/2).
- Payphone / ear and mouth caps stick 1.5 cm past the handset ends — FIXED: the caps are oval like the part's squashed cups (scale.z 0.66) and centred inward (y 0.121 / −0.161), inside the cup walls. The ear-hole dots moved with them.
- Payphone / cord rings turned about Z (no effect), facing the viewer — FIXED: rotation.x = π/2, so the coil turns stack down the cord.
- Filing cabinet / four feet buried in the body — FIXED (front pair): the front feet moved to z 0.245 so they show in the body part's toe-kick recess. The rear pair stays hidden against the wall. Raising the body onto the feet would move the drawers, whose world Y `registerCabinetDrawers` hard-codes (gameplay).
- Vending / solid body, interior and glass frame hid the cans — SKIPPED (fixed by the parts; no code needed)
- Vending / header top trim floats 6 cm above the body top — FIXED: the header glow is scaled to 0.786 (spans 1.75–1.97), the brand text is re-centred in it, and the top trim is lowered onto the body top (y 1.985).
- Locker / "ajar" spins the whole 0.5 m column about its centre — FIXED: the ajar branch flattens the column into a 4 cm leaf (scale.z 0.08, front face kept at z 0.25). The leaf is hinged in a pivot group at its left front edge (where the part's knuckles are) and swung −0.6. Louvres, handle plate, handle, padlock, tag and rust patch are re-parented onto the leaf. The coat inside now shows.
- Locker / padlock shackle lies flat 1.5 cm above the lock — FIXED: upright ring (XY plane) at y 0.83, its lower half inside the lock body.
- Extinguisher / gauge face disc faces the floor, 1 cm above the gauge — FIXED: no rotation (faces +Z), placed at (0.06, 1.03, 0.0665) in the bezel part's recess.
- Extinguisher / strap is a slab through the tank — FIXED: rotation.z = π/2 turns the ring's axis from X to Y, so it bands the tank (inner 0.222 around a 0.22 tank).
- Trash can / fixed-radius ribs float off the tapered body — FIXED: each rib is scaled in its ring plane to the body radius at its height, +6 mm. The top rib, which was buried 8 mm, is fixed too.
- Traffic cone / band is a 5 cm collar — FIXED: band scaled ×0.71 in XZ to hug the modelled cone body (r 0.089–0.101 at the band).
- Wheelchair / rear tyres, push rims and casters sink by different amounts — FIXED: the rear axle (tyre, hub, spokes, push rim) is raised to y 0.31. The push rim is scaled ×0.87 (it was bigger than the tyre). The casters go to y 0.10 and their forks to 0.16, so all wheels meet the floor together.
- Baby doll / face on the +X pole, looking along the body — FIXED: head rotated (π, 0, −π/2), so the part's face looks up with the forehead toward the crown. The eyes moved into the turned sockets at (0.24, 0.17, ±0.03).
- Baby doll / arm dips under the floor — FIXED: arms raised to y 0.10 (shoulder end on the floor; it reached 4.8 cm under it).
- CRT on floor / 0.72 m bezel overlaps the control panel (z-fighting) and covers the grille — FIXED: bezel ×0.764 and screen ×0.742 in X, both centred at x −0.015, between the grille (x < −0.295) and the panel (x > 0.265).
- Books / page block inside the cover — SKIPPED (fixed by the book_cover part)
- Coat rack / four hook rings centred on the pole, coinciding in pairs — FIXED: four upright loops, one per side, each 0.079 from the axis with rotation.y −a, sticking out radially with the near side in the pole.
- Coat rack / coat hangs centred on the pole, pierced by it — FIXED: coat at (0, 1.18, 0.085), its back against the pole and its top under the +z hook.
- Potted plant / seams at a fixed r 0.155 stand off the tapered pot — FIXED: each seam leans with the wall (−0.142 rad, YXZ order so the lean is radial) at the mid-height radius (0.1415).
- Potted plant / petals not radial — FIXED: rotation.y = π/2 − pa, so the long axis matches the radial offset.
- Floor plant / crack line floats 2.5 cm off the pot — FIXED: at x 0.2245 on the wall, leaning with the slope (z −0.149). Its diagonal is kept as a 0.15 turn about the radial axis, so it stays in the surface.
- Elevator / floor-number plane z-fighting — SKIPPED (fixed by the part's recess)
- Paper pile / clip at a fixed y 0.08 floats over small stacks — FIXED: y = stack × 0.005 + 0.003 (on the top sheet).
- Pool / ladder "arc" is one straight tube pointing out over the water — FIXED: the arc is turned (rotation.z 1.2) to rise from the rail top over the curb. The missing second segment of the "two angled segments" is added by reusing the arc's geometry (scale.y 3.5) as a post down onto the curb. Each handrail is now an inverted U over the curb.
- Robot vacuum / bumper z-fighting the body — SKIPPED (fixed by the body part's recess)
- Tool note (b00 cache rebuilt) — SKIPPED (not a placer bug)

## b09

- SingleBoots / laces on the shaft's axis, hidden — FIXED: on the shaft's front (+x) face, crossing in its plane (rotation (±0.4, π/2, 0), scale.x 0.7, x 0.0004).
- SingleBoots / "tipped" stands the boot on its heel end, sinking 3 cm — FIXED: it now rolls onto its side about its length axis (rotation.order YXZ, x π/2, y 0.032). The yaw is still random.
- FoldedUmbrellas / strap loop inside the canopy — FIXED: a band round the furled canopy (rotation.y π/2, ×2.6, on the axis).
- FoldedUmbrellas / landed under the pool water (L1 s9) — SKIPPED (level placement, not part-to-part)
- PizzaBoxes / open lid turned about its centre (leans over the box, 5 cm under the floor) — FIXED: hinged on the body's top back edge and flopped back 120° (position (0, 0.143, −bs/2 − 0.0735), rotation.x −2π/3).
- SingleGloves / cuff 3.4× thicker than the palm, 1 cm under the floor — FIXED: flattened (scale.z 0.4 → 1.9 cm tall) and resting on the floor (y 0.0096).
- CigarettePacks / lid hinge 5 mm inside the top — FIXED: lid z −0.0187, so its hinge end is on the back edge.
- BrokenSunglasses / broken temple stuck 45° into the floor — FIXED: lies flat (YXZ, x π/2, yaw π/4).
- BrokenFlashlights / head and lens sink 6 mm / 4 mm — FIXED: the flashlight is tipped 0.042 rad about its local z and lifted 2.5 mm, so the head rim and the tail both rest on the floor.
- ShoeBoxes / lid-off lid overlaps the box by 14 cm — FIXED: lid x 0.24 → 0.40.
- GumBallDispensers / globe sinks 1 cm into the base — SKIPPED (intended: the base part has a round globe seat)
- GumBallDispensers / hangs on the wall with no bracket — SKIPPED (needs a new model)
- CrossStitches / hoop turned 90° about Y, edge-on across the fabric — FIXED: turn removed (the torus is already in the fabric's plane).
- ToteBags / strap ends float 1 cm above the rim — FIXED: straps in the XY plane on the front/back walls (z ±0.047), ends 3 mm under the rim (y 0.157). This also clears the tool handles at z 0.
- CoatHooksPair / curve is a C open to the room — FIXED: a U opening upward (rotation (0, π/2, π), centre (hookX, mountY, 0.062)). One end sits on the arm tip; the ball-tip end is free.
- CastIronPans / cooking surface 3.5 mm proud of the rim — FIXED: inner z 0.026 → 0.021 (top 1.5 mm under the rim).
- CastIronPans / hanging ring lies flat, buried in the handle — FIXED: upright ring (XY plane) at x 0.220, hooked through the handle tip.
- WallBrooms / clips are vertical half rings through the handle — FIXED: horizontal half rings (rotation.x π/2) wrapped round the handle's front, centre z −0.005, ends toward the wall.
- CuttingBoards / hole plug 3 mm proud, 1 mm under the floor — FIXED: scale.y 0.8, y 0.0088 (spans 0–0.0176).
- Whisks / wire ball sinks 9 mm — FIXED: whisk tipped 0.078 rad and lifted 7.6 mm, so the ball and the end cap both rest on the floor.
- Whisks / loop ends meet sideways (±z) — FIXED: loops fanned about the handle axis (rotation.x = a), centre x 0.047, so they meet at the poles with the near pole in the ferrule.
- WallAprons / neck strap stands straight out from the wall — FIXED: in the wall plane, draped over the peg (inner top on the peg) and scaled ×3.6 in XY so its ends tuck behind the bib's top edge.
- Teapots / handle in the side plane on the shoulder — FIXED: a C looping out from the +x side (rotation.z −π/2, at (0.044, 0.052)).
- Teapots / spout slopes down and the tip floats — FIXED: spout rotation.z +π/3 (wide end in the body, rising outward). The tip is at +π/4, chained on the spout's end at (−0.0795, 0.074).
- Teapots / lid floats 2 mm — FIXED: lid 0.101, knob 0.111; the lid-bob animator's base values are updated to match.
- Teapots / trivet feet poke through the trivet — FIXED: feet under it (y −0.0035), teapot group raised to y 0.0075.
- Teacups / handle flat against the cup — FIXED: a C out from the wall (rotation.z −π/2, x 0.0245).
- CastIronLadles / handle stops 1.5 cm above the bowl — FIXED: bowl at (0, mountY − 0.172, 0.037), so its back rim meets the handle end. This also takes the bowl out of the wall.
- CastIronLadles / ring floats 1.3 cm above the handle, horizontal — FIXED: upright ring at mountY + 0.176, its lower edge through the handle top.
- "Other things seen" (wall props on walls) — SKIPPED (not placer bugs)

## b10

- SpoonRests / flipped dish half under the floor — FIXED: dish seated on the plate (scale.y 0.35 → 0.22, y 0.0139). The spoon handle rests on the rim (0.016, 0.0195) and the spoon bowl lies in the dish (−0.056, 0.0165).
- FramedRecipes / paper 2 mm in front of the frame face — FIXED: z 0.002, in the frame's rebate.
- ShakerPair / pepper fill hidden inside the opaque mill — SKIPPED (not a bug: an opaque mill hides its contents)
- AbstractLandscapes / art 2 mm in front of the frame face — FIXED: z 0.0, in the rebate.
- BreadLoaves / loaf sunk 6 mm into the board; outer scores float — FIXED: loaf y 0.0508 (flat base on the board); scores raised with it, the outer two 1.5 mm lower.
- StillLifes / art plane z-fighting the slip — SKIPPED (fixed by the slip part)
- WineBottles / flat label 0.11 m wide on a 0.072 m bottle — PARTIAL (counted as fixed): scaled ×0.5 and set tangent to the front (z 0.0365), so it stays inside the bottle's silhouette. A label wrapped round the bottle needs curved geometry (a new constructor), so from 3/4 views it is still a flat card.
- WineBottles / lying variant has its axis on the floor — FIXED: y 0.036 when lying.
- CheeseWedges / stood on its rind arc, point up, half under the floor; hole decals float 2 cm above — FIXED: the turn is removed, so the wedge lies on its triangular side (y 0.0221). The eye decals moved onto its top face inside the quarter disc (y 0.0426), with the same random calls.
- KitchenClocks / hands turned about their own centres, detached from the hub — FIXED: each hand is in a pivot group at the hub (random angle draws in the same order).
  - also (not logged): the four tick batons were tangential — FIXED: rotation.z = a + π/2 (radial).
- RollingPinStands / pins stacked 8 mm into each other, posts hold nothing — FIXED: side by side (z ±0.030) at y 0.057, each pin's handles resting on the tops of its pair of posts.
- SewingBaskets / "arching" handle lies flat in the rim plane — FIXED: the turns are removed, so it is an upright half hoop (XY plane) with its ends on the rim.
- SewingBaskets / pincushion floats 3 mm — FIXED: cushion y 0.067 (fabric top), pin heads 0.087 (the bob animator reads its base from these).
- KnittingBaskets / needle knobs float about 4 cm off the needle ends — FIXED: each knob sits on its needle's upper end, computed from the needle's rotation.
- KnittingBaskets / needle pokes out through the basket wall — FIXED: the second needle moved to (0.030, 0.088, −0.010), so its low end is inside (r 0.070) and it rises out over the rim.
- HangingColanders / bowl pokes 3 cm into the wall — FIXED: bowl, rim, handle and holes moved to z 0.115 (rim back 4 mm clear of the wall).
- HangingColanders / cord ends 6 cm above the rim — FIXED: cord scale.y 1.6, from the peg down to the rim's back point.
- HangingColanders / perforation decals face into the bowl (culled) — FIXED: `rotateY(π)` after `lookAt`, so they face out.
- EmbroideryHoops / hanging string ends in the air — SKIPPED (needs a nail/hook model)
- SmokingAshtrays / paper and filter lie side by side; ember floats — FIXED: rotation (z −π/2, y −π/4) puts both along the layout line, end to end, with the burnt end at the ember and the filter's mouth face free.
- OpenDiaries / pen tip cone points back into the barrel — FIXED: rotation.z +π/2.
- OpenDiaries / ribbon sticks out level, 1.2 cm above the floor — FIXED: rotation.x 0.184, so it drapes from the page top over the edge onto the floor, swallowtail end down.
- SewingMachines / presser foot and needle end 6 cm above the cloth; foot floats 4 mm under the arm — FIXED: foot scale.y 3.91 at y 0.072, so its shank becomes the presser bar from the arm's underside to the ski on the cloth. The needle is scale.y 4.4 at y 0.074 beside it (animator base updated).
- SewingMachines / gold trims buried inside the arm — FIXED: z ±0.0395 on the arm's faces, scale.x 0.9 (short of the rounded ends).
- Wallets / ID photo box 1.5 mm proud of the card — FIXED: scale.y 0.5, y 0.0005 (the text-line boxes likewise).
- Wall placers hang 0.9–3.5 m off the wall — SKIPPED (handled centrally)
- Page errors (entityHoundMat / entityWatcherMat) — SKIPPED (not placer bugs)

## Counts

- b03: 23 fixed (the filing cabinet only for the front pair of feet), 6 skipped (5 already fixed by the parts, 1 tool note)
- b09: 26 fixed, 4 skipped
- b10: 21 fixed, 1 partial (wine label), 5 skipped, plus 1 unlogged extra fixed (clock ticks)
- Total: 70 fixed + 1 partial + 1 extra; 15 skipped (7 already handled by parts or centrally, 2 need a new model, 6 not placer bugs or intended)

Checked on real levels; for place_props, each object framed on real levels, and front views of the vending machine, the cabinet, the payphone and the extinguisher.
