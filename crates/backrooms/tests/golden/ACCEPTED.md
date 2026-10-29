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

- 2026-10-27, on top of 614b20d: frames: 33 shots. Reason: The Building's surfaces are photographs of Soviet-era places (visuals::photo, CC0: a panel block's facade and its lit windows, whitewashed and formwork concrete, a yard's and a factory's slab, the granite of a 1949 Moscow ministry and of Kyiv's Arsenalna, ribbed precast slabs), judged blind against the canvas surfaces in three rounds (the Well 8/8, the Stair Halls 16/16, the Works 8/8 after replacing cast-iron plates and whitewash, which lost). The photographs take no native draw, so the canvas painting behind the title (level 1) no longer moves the per-frame stream: film grain, the rooms' canvas-texture noise, the death camera's shake and creature animation phases shift in every frame. Looked at the error maps of all six Building frames, 7_L5_overgrown_atrium, 4242_L10_showroom, 7_L6_spawn, 4242_L7_office_wing, 4242_L7_service_tunnels, hound_2m and the three death scenes: the rooms' frames differ by texture noise, grain and the shake only; the set-wide look shift is within its bands

- 2026-10-27, on top of 614b20d: look anchors: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: The Building's surfaces are photographs (visuals::photo): the deep-shadow share of its frames rises with the photographs' real joints, cracks and relief (4242_L1_spawn 0.41 to 0.55), each surface graded to the mean colour of the canvas one it replaced; judged blind against the canvas surfaces in three rounds (the Well 8/8, the Stair Halls 16/16, the Works 8/8)

- 2026-10-28, on top of 614b20d: frames: 4242_L1_spawn, 4242_L1_well. Reason: A photographed surface is decoded once whatever its grime, the grime being its material's tint (a linear factor) instead of a darker copy of the texture (code review: deck, poured and stone were each decoded and uploaded twice, about 80 MB on the GPU); the Well's grime-2 decks and parapets differ by rounding only (mean 0.004-0.005)

- 2026-10-28, on top of ed4dace: frames: hound_2m, hound_4m, hound_8m, hound_death. Reason: Entities E2: the Hound rebuilt from scratch (docs/ENTITIES.md): MakeHuman's CC0 old man on all fours, starved, blind, hair hanging over the skull; at its death it lifts its face to you. Only the Hound's four frames moved.

- 2026-10-28, on top of ed4dace: look anchors: hound_2m, hound_4m, hound_8m, hound_death. Reason: Entities E2: a new Hound, its look meant to change (a pale starved figure in place of the dark one; at death its face fills the frame)

- 2026-10-28, on top of 1a153c2: frames: smiler_2m, smiler_4m, smiler_8m, smiler_death. Reason: E3: the Smiler rebuilt from scratch (docs/ENTITIES.md): a too-tall black shape under the panels, its rictus of single teeth and deep eyes, eyeshine only in your beam; the death scene is its skull-like head looking down at you, no lit grin

- 2026-10-28, on top of 1a153c2: look anchors: smiler_death. Reason: E3: the old death frame was a lit black-and-white grin filling the screen, which the redesign rejects; the new one is its dark head over you, so less contrast and less deep shadow

- 2026-10-28, on top of 1a153c2: frames: 17 shots. Reason: E3: the Smiler on the new brain draws from the native stream differently each frame, so grain and camera sway shift on levels with a Smiler; frames and error maps checked: no change of content

- 2026-10-28, on top of 6d9a031: frames: watcher_2m, watcher_4m, watcher_8m, watcher_death. Reason: E4: the Watcher rebuilt from scratch (docs/ENTITIES.md): one dust sheet draped by Blender's cloth over a hidden body, its face pressed into the cloth, standing among furniture under the same sheets; its death frame faces you at eye level

- 2026-10-28, on top of 6d9a031: look anchors: watcher_death. Reason: E4: the old death frame looked up at a pale sheet against a lit ceiling panel; the new one faces it at eye level in the room, so less light and contrast

- 2026-10-28, on top of 6d9a031: frames: 8 shots. Reason: E4: the Watcher on the new brain draws from the native stream differently each frame, so grain and camera sway shift on levels with one; frames and error maps checked: no change of content

- 2026-10-28, on top of 94e7d7f: frames: corner_2m, corner_4m, corner_8m, corner_death. Reason: E5: the one in the corner, new (docs/ENTITIES.md): its back as you meet it at 2, 4 and 8 m, a tall figure in a long nightshirt; its death frame, its face turned to you

- 2026-10-28, on top of 94e7d7f: look anchors: corner_2m, corner_4m, corner_8m, corner_death. Reason: E5: a new creature's first look anchors

- 2026-10-28, on top of 94e7d7f: frames: 7_L6_spawn. Reason: E5: run 7's level 6 has the one in the corner now (placed after the build, no draws), so the native stream shifts its grain; frame and error map checked: no change of content

- 2026-10-28, on top of 94e7d7f: frames: 7_L6_spawn. Reason: the one in the corner no longer draws from the per-frame stream as a level starts (its making on a stream of its own, its first look round set by its place): this frame is back to E4's, pixel for pixel

- 2026-10-28, on top of ddf18ba: frames: reader_2m, reader_4m, reader_8m, reader_death. Reason: the Reader (E6): new frames at 2, 4 and 8 m (bent over its page, its long fingers on the back of it) and its death (the sheet lowered: its staring eyes, its open mouth inked)

- 2026-10-28, on top of ddf18ba: look anchors: reader_2m, reader_4m, reader_8m, reader_death. Reason: the Reader (E6): its frames are new, so are their look anchors

- 2026-10-28, on top of ddf18ba: frames: reader_2m, reader_4m, reader_8m, reader_death. Reason: the Reader (E6): new frames at 2, 4 and 8 m (bent over its page, its long fingers on the back of it) and its death (the sheet lowered: its staring eyes, its open mouth inked)

- 2026-10-28, on top of ddf18ba: look anchors: reader_2m, reader_4m, reader_8m, reader_death. Reason: the Reader (E6): its frames are new, so are their look anchors

- 2026-10-29, on top of 8274df8: frames: 41 shots. Reason: E7: the old creatures' textures, shapes and rigs are gone, so the start-up no longer paints them on the per-frame stream: every canvas texture after them (bricks, stains, grain) draws other numbers, and the death camera shakes otherwise; content checked unchanged (the Building's arrival, the office wing, the Smiler's death)

- 2026-10-29, on top of 8c48beb: frames: 7 shots. Reason: E9: the anomalies' figures get goldens of their own (none had one): the Showroom's staff and the Loop's seated man on their new MakeHuman bodies (blind reviews preferred both), the Borrowed House's shadows on the body they keep

- 2026-10-29, on top of 8c48beb: look anchors: 7 shots. Reason: E9: look anchors for the anomalies' new figure shots (first anchors, nothing moved)
