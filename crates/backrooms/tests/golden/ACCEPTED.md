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
