# Accepted goldens

Every change to a golden frame or a look anchor, newest last. Written by
`cargo xtask shots --accept / --accept-look`; the reason is the author's.

- 2026-10-25, on top of 338b254: frames: 31 shots. look anchors: 31 shots. Reason: First goldens: the look after the realism, creature and lighting pass (0590585), reviewed on contact sheets

- 2026-10-26, on top of f16bd4b: frames: 31 shots. Reason: Gameplay G2: the HUD compass and the objective line are gone (PILLARS 4, the way down is found, not shown); dropping the objective's random pick shifts the native stream, which moves film grain, canvas-texture noise and creature animation phases in every frame. Looked at the error maps of 7_L6_spawn and hound_2m: the compass, the objective line, grain and texture noise; the set-wide look shift is within its bands.

- 2026-10-26, on top of b4bd003: frames: 31 shots. Reason: Gameplay G3: M.'s first sheets lie near the arrival (spawn and creature frames), and each headless view now runs the props' distance cull for its own camera; before, every view kept the spawn's cull and small props far from the spawn (store signs, boards, trinkets) were missing from the zone frames. Looked at 4242_L4_showroom and L1_office_wing error maps: distant signs and props now drawn. Set-wide look shift within bands.

- 2026-10-27, on top of 5f08292: frames: 31 shots. Reason: UI U5, the HUD at rest: the always-on LEVEL label, seed, SANITY/STAMINA/FLASHLIGHT bars and labels are gone until they have something to say, and toasts are sentence-case Helvetica Neue; the diff maps are HUD-only (the scene is unchanged), and the whole set moved the same way (colourfulness -2.0, contrast -0.023) because the HUD's capitals and bars no longer count in the look

- 2026-10-27, on top of 5f08292: look anchors: 31 shots. Reason: UI U5, the HUD at rest: the always-on LEVEL label, seed, SANITY/STAMINA/FLASHLIGHT bars and labels are gone until they have something to say, and toasts are sentence-case Helvetica Neue; the diff maps are HUD-only (the scene is unchanged), and the whole set moved the same way (colourfulness -2.0, contrast -0.023) because the HUD's capitals and bars no longer count in the look

- 2026-10-27, on top of aa66055: frames: 33 shots. Reason: Megastructure B3: the descent begins in the Building (levels 1-3: the Well, the Stair Halls, the Works); the rooms from level 4, so every level view is a new floor (the golden levels add the Building's three and 4242's Showroom, now on L10) and the creature showcases moved to level 4, the Lobby; frames looked at

- 2026-10-27, on top of aa66055: look anchors: 33 shots. Reason: Megastructure B3: the descent begins in the Building (levels 1-3: the Well, the Stair Halls, the Works); the rooms from level 4, so every level view is a new floor (the golden levels add the Building's three and 4242's Showroom, now on L10) and the creature showcases moved to level 4, the Lobby; frames looked at
- 2026-10-27, Megastructure B3: removed the ten goldens and look anchors whose level views no longer exist (4242_L1_empty, 4242_L1_office_wing, 4242_L4_showroom, 4242_L7_overgrown_atrium / party_room / play_area / pool_hall / shrine / washrooms, 7_L5_parlour): their floors are other places now that the rooms begin on level 4.

- 2026-10-27, on top of c3ecfaf: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: Megastructure B4: the Building's look (a railing of each place's own, lamps at the foot of every flight and their posts, a brighter light, the Building's own air instead of the rooms' weathers); frames looked at, only the Building's six moved

- 2026-10-27, on top of c3ecfaf: look anchors: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: Megastructure B4: the Building's look (a railing of each place's own, lamps at the foot of every flight and their posts, a brighter light, the Building's own air instead of the rooms' weathers); frames looked at, only the Building's six moved

- 2026-10-27, on top of 7ac8934: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L3_spawn, 4242_L3_works. Reason: B6: cell plates read NNN-L (a draw on the dressing's salted stream moves the Well's sealed doors after it), her camp in a cell, the Building's own notes, the Works' last door lined with the Lobby's yellow

- 2026-10-27, on top of bd7b043: frames: 4242_L3_spawn. Reason: B7: the Works' machine housings drawn to their own tops (they were drawn to 150 m whatever their plan said; the solids already stopped at the tops), so a housing seen far down the arrival's corridor stands lower
