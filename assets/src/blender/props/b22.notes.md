# b22 notes (victorian / occult)

95 parts, 23 084 triangles, wrapping 102 primitives over 27 of the 29 placers. 0 errors, 0 warnings.

Checked part by part and in the game (level 4, seed 7; level 2; zoomed runs).

The glass, flames, halos, glows, basic material eye/face/socket decals and the textured compass face stay primitives, as the contract says. Tiny parts only got a cheap rounded version: scale chains, spider legs, wicks, teeth, buckle, pivot pins.

- **place_small_compasses**: `case` (rounded bowl, band, raised bezel, shallow face well), `hinge` (three-knuckle barrel), `cover` (domed outside, dished inside facing the dial). Left: the face (circle_geo!, DoubleSide texture). Placement: the cover is centred at (0.030, 0.030) and tilted 60°. Its low edge reaches about (0.012, 0.0), so it cuts down through the case and the dial instead of standing on the hinge at x 0.040.
- **place_small_telescopes**: `tube` (draw tube: ferrule collar and bead at −Y, thin lip at +Y; one part for all three sections), `grip` (stitched leather wrap). Left: eyepiece and objective lens circles (transparent basic material). Placement OK. The eyepiece sits at the wide end, back to front, but harmless.
- **place_small_globes**: `base` (turned plinth: foot band, cove, rounded edge), `column` (baluster), `meridian` (flat chamfered brass band with two pivot pins on the inside), `sphere` (smoother 20×14 sphere; the texture still maps through the sphere UVs). Placement: `rotation.z` 0.4 spins the ring in its own plane, so it never tilts. Only the pins show the 23° tilt. Otherwise OK.
- **place_small_phonographs**:
  - `base`: cabinet with plinth, top moulding, sunk side panels and crank boss.
  - `turntable`: platter with stepped rim.
  - `record`: 78 with beaded edge, groove bands and raised label area.
  - `arm`: tapered tone arm with pivot collar.
  - `head`: sound box with needle.
  - `horn`: eight-panel morning-glory horn with rolled mouth.

  Left: the label (flat decal). Placement (visible in game):
  - The horn cone is upside down. Its throat points up and away (apex near (0.026, 0.243)), and its 8 cm mouth dips into the cabinet (rim down to y ≈ 0.03).
  - The tone arm (`rotation.z` 0.3 on a Y cylinder) stands almost upright at x 0.055, with its lower 2.3 cm buried in the cabinet.
  - The arm head box sits on the record at x 0.032, 4 cm from the arm.
- **place_hanging_brass_lanterns**: `chain` (links alternating flat in X and in Z), `topCap` (square stepped pyramid roof with chain boss), `botCap` (the same, turned over), `post` (turned rod with ball-and-collar ends). Left: glass panels (transparent), flame cone and halo (basic material). Placement:
  - The posts stand on the diagonals at r 0.028, but the glass panels stand on the axes at r 0.028. The panel corners stick about 8 mm outside the posts.
  - The chain stops 2 cm below the ceiling.
- **place_small_telegraph_keys**: `base` (stepped wooden base with four screw heads), `post` (trunnion pillar with pivot screw), `lever` (key lever with pivot boss, narrowing to the knob end), `knob` (bakelite mushroom knob, squashed by the placer), `terminal` (binding post), `wire` (twisted cloth pair). Placement: the "trailing" wire stands almost upright (`rotation.z` −0.2), from y −0.041 to 0.057, 4 cm off the end of the base. It is half in the floor and not connected to the terminals.
- **place_small_scales**: `base` (cast base with a step and a domed top), `pillar` (turned pillar), `beam` (pivot boss, tapering arms, end knobs), `chain` (three-link run), `pan` (spun pan with rolled rim). Placement:
  - Each chain run stops at y −0.085 under the beam, 11 mm above its pan (top −0.096). The pans hang free.
  - The pans are only 2 mm above the base rim and overlap it by 1.6 cm, so they clip into it at the ends of the sway.
- **place_small_abacuses**: `bar` (frame bar, small even chamfer), `rod` (8-sided brass rod), `bead` (bicone with rounded waist).
  - `bar`: one constructor (`box_geo!(fw, fh, t)`) makes both the 15 cm rails and the 9.6 cm stiles, so the part only has a 0.7 mm chamfer. It reads right on the rails. On the stiles it squeezes to a near-sharp edge with a faint slope at the joints. Anything finer would distort on one of the two.
  - Placement: the end beads (to x ±0.066) run 3 mm into the stiles (inner face ±0.063).
- **place_small_trophies**: `base` (plinth with rounded top edges and a groove above the foot), `plaque` (clipped-corner brass plate, 0.9 mm thick behind its face), `stem` (spreading foot, knop, flared cup socket), `cup` (tulip bowl with a band and rolled lip), `handle` (ribbon handle swelling into pads at its ends). Left: the three engraving lines (transparent basic-material decals). Placement: the handles are semicircles in the YZ plane at x ±0.06. They face sideways and stand 2 cm clear of the cup wall (r ≈ 0.0375 at y 0.16), so they float beside the cup (visible in game).
- **place_small_handbells**: `dome` (handbell: flat crown, waist, flared sound bow, open mouth at the hemisphere's rim), `ring` (square collar), `handle` (turned handle with a knob), `clapper` (smooth ball). Placement: the dome is the upper hemisphere turned over by `rotation.x` π, so the bell opens upward like a bowl. The handle rises out of the open mouth, and the clapper sits in the bowl (visible in game).
- **place_small_masks**:
  - `face`: an egg-shaped face plate with a flat back and a rolled edge. The front has a brow, eye hollows, a nose ridge, cheeks and a mouth slit. The placer squashes it to 15 %.
  - `strap`: plaited elastic cord.

  Left: the eye circles (basic-material decals). Placement: the strap (y 0.015, z 0.05) runs straight across the front of the mask at the mouth, half buried in the surface, instead of behind it (visible in game).
- **place_candle_clusters**: `body` (candle with a soft shoulder, a melted dish and two wax runs), `wick` (charred, curled). Both are shared with BrassCandlesticks. Left: flame cone and halo (basic material). Placement OK.
- **place_small_crystal_balls**: `base` (dark foot with a gadrooned skirt and moulded top), `prong` (claw with knuckle and hooked point). Left: the ball and the sparkle (transparent basic material). Placement: the prongs lean tangentially. `rotation.x` = cos a·0.4 tilts the prong at angle a sideways, so the three claws twist round like a pinwheel instead of splaying out under the ball.
- **place_small_pumpkins**: `body` (ten sharp-grooved lobes, dimples top and bottom), `stem` (five-ribbed woody stem, flared foot, leaning). Left: eyes and mouth (basic-material decals) and halo. Placement: the face decals at z 0.048 lie inside the body (surface at z 0.049–0.052 there). They only show where a rib groove dips (level 2 shot); on level 4 they are hidden.
- **place_small_witch_hats**: `brim` (thin felt brim, five waves, edge curled up), `cone` (crown whose top third flops over, with creases at the knee), `band` (tapered ribbon sleeve), `buckle` (square frame with centre bar). Placement: the crown turns by `rotation.z` about its middle and is also shifted by `position.x` = 0.05·sin. Its base ends up to 1.2 cm off centre, so it pokes out of the band on one side.
- **place_small_skeleton_hands**: `palm` (four metacarpals on a row of carpals), `bone` (waisted phalanx with flared ends; also the two thumb bones), `joint` (two-condyle knuckle), `wrist` (row of carpal pebbles). Placement: both thumb bones are tilted 60° (`rotation.z` π/3) and rise at 30° off the floor side by side. They are stepped 8 mm apart in height and 8 mm in z, not joined end to end, so the thumb reads as a zig-zag.
- **place_small_cauldrons**: `body` (thick cast wall flaring to the rim, seam bead, open both ends; the old one was see-through from above), `bottom` (thick hemispherical bowl shell), `rim` (heavy rolled lip), `leg` (stubby cast foot), `handle` (wire bail with thicker grip and eye ends). Left: the brew disc (transparent basic material). Placement:
  - The round bottom (lower hemisphere at y 0.008) reaches 3.7 cm below the leg tips, and the three legs are buried inside it.
  - The bail is laid flat (`rotation.x` then `rotation.z` leave it in the XZ plane) at the rim's height and radius. It lies inside the rim torus on one side and never arches over the pot.
- **place_small_tombstones**: `body` (weathered slab with worn edges and an incised border line), `top` (weathered half-disc head), `base` (worn plinth), `vBar` / `hBar` (chamfered cross strokes). Placement: the rounded top (half cylinder, `thetaStart` 0, `rotation.x` π/2) stands as a D on the right half of the stone (x 0…0.06, y 0.104…0.224) instead of an arch across the top (visible in game).
- **place_hanging_bats**: untouched. Body and ears use a DoubleSide material, the wings are buffer geometry, and the eyes are basic material.
- **place_small_spiders**: `body` (egg abdomen, flatter belly), `head` (low cephalothorax with two fangs), `leg` (thin jointed segment, shared by both segment constructors). Left: the eyes (basic material, 1.5 mm). Placement:
  - The legs are mirrored by position but not by rotation. On the −X side, the first segment slopes down going outwards and the second rises, so those four legs form inverted Vs.
  - The second segments end about 3 mm above the floor.
- **place_small_brooms**: `stick` (crooked branch with knots and a worn end), `twig` (kinked tapering birch twig with nodes), `twine` (four coils on a slight helix). Placement: the twigs fan out vertically (±45° in the XY plane) from a stick end 2 cm above the floor. The lower twigs run up to 8 cm into the floor instead of the bundle lying flat.
- **place_small_ghost_sheets**: untouched. Every sheet mesh uses a DoubleSide material, and the eyes are basic material.
- **place_small_black_cats**: `body` (sitting mass: chest, narrow neck, haunches), `head` (wide cheeks, flat face, muzzle), `ear` (cupped half-cone shell open to the front), `tail` (tapering, thickest at the root inside the body), `legs` (haunch drum with two front paws). Left: the eyes (basic material). Placement: the ears are 1.4 cm cones centred at y 0.108, where the head surface is at about 0.109. Only their top 6 mm shows.
- **place_small_treat_pails**: `body` (moulded eight-lobe pumpkin pail), `rim` (collar with rolled lip, shallow well and a centre boss under the stem), `handle` (flat plastic strap with thicker grip and pivot knobs). The stem uses `SmallPumpkins__stem`. Left: face decals (basic material). Placement:
  - The face decals at z 0.060 are inside the body (surface ≈ 0.063–0.067), so they are hidden.
  - The handle is laid flat in the XZ plane 1.2 cm above the rim instead of arching over it (visible in game).
  - The stem stands in the middle of the candy opening.
- **place_small_skulls**:
  - `cranium`: built round the face axis. Eye sockets and a nasal hole are carved to just behind the placer's dark discs, which were buried 6 mm inside the old sphere and now show at the socket bottoms. It also has a brow, cheekbones, the lower face drawn in over the jaw and teeth, and a fuller braincase.
  - `jaw`: horseshoe mandible with a chin knob.
  - `tooth`: rounded, tapered.

  Left: socket circles and the nose triangle (basic-material decals). Placement: the jaw box and teeth were fully inside the old cranium. Even now the teeth (z 0.028–0.030) sit inside the jaw box's front (0.030), and only the outer ones show in front of the curved jaw.
- **place_small_potions**: `cork` (tapered, rounded top edge, pitted). Left: the bottle body, neck and liquid (all transparent basic material). Placement OK.
- **place_skull_stacks**: uses `SmallSkulls__cranium` and `SmallSkulls__jaw`. The sockets now show in game. Left: socket circles. Placement OK: the skulls overlap 1.5 cm where stacked, which reads as resting on each other.
- **place_small_owls**:
  - `body`: fuller base, breast, folded wings standing proud, short tail.
  - `head`: built round the face axis, with wide cheeks and two eye bowls pressed into a flat facial disc. The eye-ring circles, half buried in the old sphere, now show (level 4 seed 21 shot).
  - `tuft`: three stiff feathers.
  - `perch`: log slice with a rough bark edge.

  Left: eyes (basic material), eye rings (flat decals), beak and wing triangles (buffer geometry). Placement:
  - The tufts (y 0.134–0.146 at x ±0.014) are buried in the head; only about 1 mm pokes out.
  - The wing triangles at z 0 are inside the body.
  - The eye rings' outer edges still overhang the sides of the head.
- **place_brass_candlesticks**: `base` (beaded edge, cove, domed top), `column` (turned, with rings and a flared capital), `knop` (two domes meeting at a sharp belt), `cup` (drip pan with rolled rim and a socket boss the candle stands on), `drip` (wax run with a bead at its end). The candle and wick use `CandleClusters__body` and `CandleClusters__wick`. Left: flame and halo (basic material), the wax pool (buffer geometry). Placement: the wax pool is a flat disc at y 0.014, which hovers 4–5 mm above the sloping base top.
