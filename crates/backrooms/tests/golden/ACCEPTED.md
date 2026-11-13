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

- 2026-10-29, on top of 6540e76: frames: 48 shots. Reason: E10: every canvas texture paints from a stream of its own (placers::kit::canvas_tex, with_own_stream), not the per-frame stream, so each texture's stains/bricks/grain pattern changed once and the per-frame stream after start-up moved (grain, the death camera's shake); the Smiler (v7: deeper ribs, dust on the bones, a wider paler grin) and the Watcher (v6: a narrow shroud over a 1.96 m man, eye and mouth marks, poses aligned on the cloth) are new bodies a blind review preferred (two reviewers each); all 48 frames looked at side by side, content otherwise unchanged

- 2026-10-29, on top of 6540e76: look anchors: watcher_death. Reason: E10: the Watcher's new body (blind-reviewed) is taller, so the death camera looks up at its face under the ceiling light: the frame is brighter (luminance 0.305 -> 0.365)

- 2026-10-29, on top of 577a8f8: frames: 48 shots. Reason: Rooms R1: the field of view is the vertical view up to 16:9, capped at 100 degrees across on wider screens, and a new player walks at 64 (was 74 at every aspect); every frame's composition is tighter (preferred 9 of 9 by two blind reviewers at 16:10 and 21:9); all 48 frames looked at

- 2026-10-29, on top of 577a8f8: look anchors: 48 shots. Reason: Rooms R1: the tighter field of view changes each frame's composition (less floor and ceiling at the edges), so the shadow and luminance shares move; the look itself was blind-reviewed (9 of 9, two reviewers)

- 2026-10-29, on top of 577a8f8: frames: 48 shots. Reason: Rooms R3: corners darken where walls meet each other, the floor and the ceiling (the light map's new G and B channels, read from the sample the shader already takes 0.8 m out from each surface; preferred blind by two reviewers, 5 and 7 of 9, never the old); and the rooms' surfaces (drywall, concrete, brick, tile) paint from their painters' own streams instead of the per-frame one, so their stains and marks are laid out once anew (the concrete floors and slab ceilings come out more mottled; they are to be photographed, R6). All 48 frames looked at

- 2026-10-29, on top of 577a8f8: frames: 48 shots. Reason: Rooms R5: the rooms' ceilings are acoustic tiles at a real size (6 x 6 a tile, 0.67 m, on a T-bar grid; the fittings 2 x 1 tiles, 1.33 x 0.67 m, in it; a fallen tile is one of them, not a black slab), and photographed details lie over the wallpaper (woodchip), the carpet and the tiles' face (mineral fibre); preferred blind by two reviewers (7 of 9 each, 2 ties, the old never). All 48 frames looked at

- 2026-10-29, on top of 577a8f8: look anchors: smiler_death, corner_death, 4242_L10_showroom. Reason: Rooms R5: the ceiling fittings are 2 x 1 tiles (1.33 x 0.67 m, a 2 x 4 ft troffer) where they were 1.9 x 1.25 m, and the fallen tile is a tile: frames that look up at a fitting are less bright and less contrasted, and no longer hold the black slab's shadow (blind-reviewed: 7 of 9, two reviewers)

- 2026-10-29, on top of 577a8f8: frames: 48 shots. Reason: Rooms R6: photographed details over the other zones' painted surfaces (plaster on drywall, painted walls and plaster ceilings; rough concrete on concrete walls and slab ceilings; a worn concrete floor; the carpet on the office's carpet tiles; quilted so no feature repeats on a grid), preferred blind by two reviewers (8 of 9 each, 1 tie, the old never); and the rooms' painted floors and ceilings in world space laid from the tiles' edges (as their planes' UVs had laid them: from the centres, a floor's saw-cut ran down the middle of every corridor). All 48 frames looked at

- 2026-10-29, on top of 892597c: frames: 4242_L7_spawn, 4242_L7_empty, 4242_L7_office_wing, 4242_L7_storage_hall, 7_L5_pool_hall, 7_L5_play_area. Reason: Rooms R4: partitions that lean now and then (world::angled); the pool hall's and play area's near partitions lean, the four Lobby-office frames differ by a few pixels (a distant lean, the light map's corners from the staircases)

- 2026-10-29, on top of 49e683a: frames: 15 shots. Reason: Vending V1: the vending machine rebuilt (a real glass-front snack machine, lit when powered, solid) in place of the COLA-Z and the dead black box; the machine's builder takes one draw from the build's stream and the dead machines' placer is gone, so every later placer's draws shift: props moved (a partition, the balls in the pool, a doll, a trash can), machines stand in some views

- 2026-10-29, on top of 49e683a: look anchors: 7_L5_overgrown_atrium. Reason: Vending V1: the half-height partition in front of this view is gone (its 25% chance drawn anew after the vending machine's builder took one draw), so more of the lit wall behind shows: less deep shadow (0.916 to 0.864); content moved, not the look

- 2026-10-30, on top of 50da029: frames: 11 shots. Reason: Vending V5: no machine in the Showroom (4242 L10, staff) and none on seed 7 L5 (no office, hall or pool zone with a purpose there: the machine stands only where one would have put it, never in a bare zone); the props placed after it are placed from where the stream now is (a box stack and cone, a pillar wrap, a lit machine's glow gone). Looked at each frame and error map.

- 2026-10-30, on top of 19efe5c: frames: 4242_L7_maintenance, 7_L5_play_area. Reason: Windows W1: a window far off in each frame is now an opening cut into its wall onto the view (the old framed picture gone); nothing else in either frame moved (the windows keep the old window zone's draws)

- 2026-10-30, on top of 1d1697d: frames: glass_32m, glass_8m, glass_0m, glass_in, glass_death. Reason: W2: the one behind the glass, new (docs/WINDOWS.md 4): out of its window on 4242's level 7, far out on the track in the fog (a smudge), 8 m out, its hood's rim and face to the pane with its hands either side, come through into its lamp's light, and its kill; looked at frame by frame

- 2026-10-30, on top of 1d1697d: look anchors: glass_32m, glass_8m, glass_0m, glass_in, glass_death. Reason: W2: the one behind the glass, new: its first look anchors

- 2026-10-30, on top of 1d1697d: frames: glass_0m, glass_in, glass_death. Reason: W2, after the blind review's findings: its face to the pane seen from a metre, its hands wide on the glass with the elbows out (the sleeves had read as planks seen into their cuffs); its face cold and drowned (bluish skin, veins, raw lids, grey lips, clouded eyes, wet), the hood's rim no longer inked; looked at frame by frame

- 2026-10-30, on top of 1d1697d: look anchors: glass_0m, glass_in, glass_death. Reason: W2: the pressed view's new framing and the figure's new face

- 2026-10-30, on top of 1d1697d: frames: glass_0m, glass_in, glass_death. Reason: W2, after the second blind review's findings: its hands flat on the glass at its chin, the upper arms hanging (lifted from the shoulder, skinning had swung the coat's shoulders up into fins beside the hood); the hood moved only by the head, neck and chest; the coat wet (a sheen); looked at frame by frame

- 2026-10-30, on top of 1d1697d: look anchors: glass_0m, glass_in, glass_death. Reason: W2: the figure's final pose and coat

- 2026-10-30, on top of 6452ff3: frames: glass_0m. Reason: W3: within reach of its window's blind, the HUD offers [E] LOWER THE BLIND beside its face at the glass; nothing else changed (error map: only the prompt)

- 2026-10-30, on top of 8b50e49: frames: 33 shots. Reason: Secrets S1: her trail's sheets are her marks now (fewer sheets, smaller pencil arrows, stacks, marks on walls), and the torn notes lie without a glow or a mark on the wall; each diff is a patch of 2-122 px where a mark or a note's halo was

- 2026-11-01, on top of 024cdba: frames: shadow_2m, shadow_4m, shadow_8m, 4242_L7_storage_hall, 4242_L7_service_tunnels, 7_L5_spawn. Reason: Props P5: barricades close real openings (a board-up ends the service tunnel's corridor; the old pile in the Poolrooms is gone: no barricade in a pool hall; her camp and marks move round the new ones)

- 2026-11-01, on top of 024cdba: frames: 4242_L7_office_wing, 4242_L7_storage_hall. Reason: Props P5: her marks keep off a wall face something stands against (a machine, a barricade): one moved

- 2026-11-03, on top of 13a00f9: frames: 4242_L7_storage_hall, 4242_L7_living_quarters, 7_L5_play_area. Reason: Props P1: remodelled props in view (the bentwood coat stand and its coat; a pile, a book stack)

- 2026-11-03, on top of 2a35e3b: frames: 7 shots. Reason: Props P1 (3): r07's wall marks painted soft (the Lobby's water streak in the spawn and staff views); remodelled props far off in the L7 views (looked at each frame and its error map)

- 2026-11-03, on top of 2a35e3b: frames: smiler_4m, smiler_8m, watcher_8m, staff_2m, staff_4m. Reason: P6: the Lobby's lit set now hisses close by (its place in the CRTs' hiss list); the ambience reschedules it from the frame stream while you are out of earshot, so the film grain of these Lobby showcases is drawn from a shifted stream (scene identical; error maps an even speckle)

- 2026-11-03, on top of 2a35e3b: frames: 15 shots. Reason: P6: a lit set now hisses close by (its place in the CRTs' hiss list); the ambience reschedules it from the frame stream while you are out of earshot, so these frames' film grain is drawn from a shifted stream (scenes identical: mean 0.002, no tile over 0.008)

- 2026-11-03, on top of 4993562: frames: 7_L5_play_area. Reason: P2: a small thing on the floor far off remodelled (one tile's pixels; looked at the frame and its error map)

- 2026-11-07, on top of e751c96: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: G1: the Building's floors are 28 tiles a side (5 platforms) instead of 40, so each of the six Building views (the arrival and one zone view per place, seed 4242) looks at a new floor; looked at: the Well's galleries and a sealed door, the Stair Halls' corridor between masses and a balustraded walk with its lantern, the Works' wall and railings under its ceiling; nothing broken

- 2026-11-07, on top of e751c96: look anchors: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: G1: the six Building views look at new places on the smaller floors (the Stair Halls' zone view was a dark flight of steps up close, shadow 0.886; it is now a lit walk between balustrades, 0.615), so their anchors describe views that no longer exist; the places stay inside their STYLE.md bands (range --look: well, stair halls, works 100% of 16 views each) and the whole set's median look is within bands

- 2026-11-08, on top of cc57fe5: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: G2: cells within 8 tiles of every floor (half the warning): the Building's floors have 14-15 cells instead of 8, placed for coverage (cut into the building, or booths over the void off the door's view line); new cells, their lamps and doors in view, a booth gone from beside the Well's arrival, and the Works' housings' heights drawn after the cells (more blocks)

- 2026-11-08, on top of cc57fe5: look anchors: 4242_L3_spawn, 4242_L3_works. Reason: G2: the Works' views moved with its cells (14-15 a floor, placed for coverage) and its machine housings' heights drawn after them: the spawn view's right-hand housing now stands lower (more of the dark hall in view: deep shadow up), the zone view now looks down a walk between a housing and a cell's booth (its door lit: luminance up, deep shadow down); the place stays inside its STYLE.md band

- 2026-11-09, on top of 458ebe3: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls. Reason: MILESTONES.md 1: a booth stands from the void to a storey over its walk, not to the top of the building; where one stood in view there is void now (two blind reviews: the floors without G2's tall booths preferred 8-2, the low booths over the tall 6-0)

- 2026-11-09, on top of 98fd2fb: frames: 4242_L2_stair_halls. Reason: MILESTONES.md 2: the flights that cross the Stair Halls' void have a level landing halfway up (where the figure stands); the edge of one far flight moves in the top-left corner

- 2026-11-09, on top of 4268009: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_stair_halls. Reason: MILESTONES.md 4 (Building G4): rooms off the walks are cut into the building; the faces round them change, so the Well's lit windows and storey numbers fall elsewhere (the windows' and numbers' dressing walks the faces), and a far room's edge in the Stair Halls

- 2026-11-09, on top of 804176e: frames: 4242_L1_spawn, 4242_L1_well. Reason: review 1: the Well's dressing draws as before rooms again (a lit window at the corridor's end and the storey number back where they were on the floor before rooms, as the goldens before G4 show), and a lit window that floated over a booth's top is left out

- 2026-11-09, on top of 86a857f: frames: 15 shots. Reason: Props P4 (MILESTONES.md 7): wear by use on every settled prop surface (dust on what faces up, the floor's grime at the foot, convex edges rubbed and chipped); looked at each frame and its error map: the change is on props only (the window's sill and blind slats dusted in the glass frames, the Poolrooms curb and floating balls, a bin, far room-set furniture and small far props), nothing broken, every look anchor within its band

- 2026-11-09, on top of 6efc5ef: frames: glass_32m, glass_8m, glass_0m, glass_in. Reason: Code review 2: chips follow how much a thing is handled. The window in the glass arena hangs 0.85 m off the floor (handled 0.45, now chips 0): its sill's convex edges and the blind's slats lose their chip flecks (one-pixel lines along the sill's front edge, at most 160 px, p99 0.06-0.09). Nothing else in the frames changes

- 2026-11-09, on top of 2b6e022: frames: 39 shots. Reason: M8: the dust motes a few millimetres (they were 4.5-6.5 cm, glowing specks), the lamp's glints smaller, the wallpaper's torn patches soft and ragged, contact shadows under small props; looked at each frame and error map: scattered mote dots, the patches on the dirty walls

- 2026-11-09, on top of 0931fa3: frames: 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: Milestone 11: the Stair Halls' lamp heads are iron lanterns and a flight's foot lamp no longer stands in a cell's lamp; the Works' pipes run as pipes (joints, elbows into the wall, brackets, painted steel), none over a flight, and its floodlights have a hood

- 2026-11-09, on top of fa45f64: frames: 4242_L3_spawn. Reason: Code review 3: the Works' pipes and elbows eight-sided (14 before) and the pipe open-ended; only the near pipes' silhouettes move by a pixel at their edges, they still read as round pipes (looked at the frame and its error map)

- 2026-11-09, on top of 833db43: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: Code review 3, floor occlusion from the floor under it: the walls' feet darken toward their own storey's floor (the arrival and these walks stand over y = 0, where the old occlusion from y = 0 left them unoccluded); nothing else moved (looked at each frame and its error map: only the wall feet along the walks)

- 2026-11-09, on top of 833db43: frames: 4242_L7_spawn. Reason: Code review 3, floor occlusion from the floor under it: the pool's floor (-0.55 m) is in the light map, so the beach balls' lower halves and the pool's far wall, over it, lose less of their indirect light (they were occluded as if on y = 0); the room around unchanged (frame and error map looked at)

- 2026-11-09, on top of 833db43: frames: hound_death, corner_death, reader_2m, reader_death, staff_2m. Reason: Code review 3, floor occlusion: single scattered pixels on the creatures' bodies only (no area moved, the error maps black but for specks), from the recompiled fragment shader; their floors are at 0 and their light the same

- 2026-11-10, on top of ca6ca87: frames: 4242_L1_spawn, 4242_L1_well, 4242_L2_stair_halls, 4242_L3_spawn, 4242_L3_works. Reason: Code review 4, the floor kept apart from the void: on the Building the light map's B says which places have a floor and the shader reads A over B, so a surface beside a walk's end or a mass reads the walk's own floor (a filtered sample half over the void had read a floor down to the lowest storey); and a thing that takes wear by use stands on its own floor there. The feet of the doors, door jambs, balusters, railing posts and stair risers beside a gap darken as the wall feet do (up to 7-10 levels, within 0.45 m of the floor; the rest of each frame unchanged; frames and error maps looked at). Blind review: new preferred 12, old 0, tie 1

- 2026-11-11, on top of 9e6440d: frames: way_in_mid. Reason: C5: the way in from the title (New descent), at 2.6 s on run 4242's Well: the camera drawn back from the title's view under the gallery into your eyes, the bars in (docs/CUTSCENES.md)

- 2026-11-11, on top of 9e6440d: look anchors: way_in_mid. Reason: C5: a new shot, the way in at 2.6 s; its look (the Well under the gallery, the bars' black included) is its own baseline

- 2026-11-11, on top of db665e9: frames: shaft_eyes. Reason: C6: a broken lift's shaft at 3.1 s on run 4242's level 4: leaning in, the black between the jammed doors filling the view, two points of eyeshine low in it (docs/CUTSCENES.md)

- 2026-11-11, on top of db665e9: look anchors: shaft_eyes. Reason: C6: a new shot, the shaft's black and its eyeshine; its look is its own baseline

- 2026-11-11, on top of db665e9: frames: shaft_eyes. Reason: C6 after its lore review: what looks back out of the shaft gives back a light (the red lamp's here, your lamp off) instead of glowing on its own

- 2026-11-11, on top of 1c92575: frames: 7 shots. Reason: C7: a death is a cutscene now (death_taken): at 0.25 s the same moment, its face in your eyes, framed by the cutscene's camera and its clock's shake instead of the death camera's, the HUD hidden; and death_fall, the Hound's kill at 0.9 s: on the floor, rolled, it standing over you (docs/CUTSCENES.md)

- 2026-11-11, on top of 1c92575: look anchors: death_fall. Reason: C7: a new shot, the fall; its look is its own baseline

- 2026-11-11, on top of 725c600: frames: shaft_eyes. Reason: C8: the shaft retimed (its approach 1.9 s and its hand-back 2.1 s, so a look begun with your back to the lift stays under 150 deg/s; found by the skill's trial: E has no facing cone); the same moment, at 3.7 s

- 2026-11-13, on top of 5bd9206: frames: way_in_mid, 4242_L1_spawn, 4242_L1_well, 4242_L2_spawn, 4242_L3_works. Reason: The cell door's frame stands 5 mm proud of its reveal and lintel (VP8): its jamb no longer fights the pier (the stone's photograph striping through it, the Works' frame); the other four move by the frame's 5 mm

- 2026-11-13, on top of e35cb35: frames: 4242_L3_spawn, 4242_L3_works. Reason: The Works' railing posts 4.5 cm wide and stopped under the top rail (VP10, F6): they shared the rail's faces and stood 2 cm through its top; the frames differ at the posts only

- 2026-11-13, on top of f12046b: frames: 4242_L3_spawn, 4242_L3_works. Reason: review B: the Works' rail posts run into the top rail (1.00-1.05 m: along's lift is its underside) to its centreline; VP10 stopped them at 0.975 m and every handrail floated 2.5 cm over its posts

- 2026-11-13, on top of f12046b: frames: 4242_L2_stair_halls. Reason: review B, F8: a flight's steps stop 5 mm short of a side over the void, inside its railing (their ends lay on the Well parapet's outer face and fought it); the frame differs where a flight meets the balustrade's plinth only

- 2026-11-13, on top of 031cded: frames: staff_2m, staff_4m, 4242_L10_spawn. Reason: Structure O3b: a placer's painted textures are named by the placer, not by where its code is, so the placers can move by theme; each was painted once more from a stream of the new name. The one change over its threshold is the same drip stain down the spawn room's left wall (shorter and fainter, the same painter), seen in all three; 37 frames changed under their thresholds; the set's look within its bands
