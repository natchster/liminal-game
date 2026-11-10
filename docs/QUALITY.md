# Quality scorecard

A grade for each subsystem and content family, the evidence behind it, and
what is open. The drift sweep rewrites the *Evidence* and *Open* columns; a
grade moves only with an entry under *History* saying why. The ledger of
individual content is [`content.json`](content.json); the tooling that
produces the evidence is in [`TOOLING_ROADMAP.md`](TOOLING_ROADMAP.md).

Grades: **A** held by gates and confirmed by people · **B** held by gates,
not yet confirmed by people · **C** partly covered · **D** not covered.

Last updated: 2026-11-09. The *Latest sweep* section is written by
`cargo xtask drift`; the grades and the rest are written by people.

## Latest sweep

<!-- drift:begin -->
Last sweep: 2026-10-29.

| Gate | Result |
|---|---|
| Check steps | ok    fmt         0.9 s; ok    clippy      0.2 s; ok    test     1942.8 s  339 passed, 13 ignored; ok    shots      74.7 s; ok    range      40.4 s; ok    bench      94.3 s |
| Tests | 53 of 53 test files pass; placer smoke tests pass |
| Golden frames | 48 of 48 match |
| Expressive range | energy distance 0.0951, p = 0.544; seed archive: 19/36 cells, most crowded 14% |
| Performance | within budget, no step in the history |
| Largest files | `crates/game/src/level/archgen.rs` 2368 (99% of the cap), `crates/game/src/visuals/textures.rs` 1970 (82% of the cap), `crates/engine/src/audio.rs` 1926 (80% of the cap), `crates/engine/src/render.rs` 1891 (79% of the cap), `crates/game/src/modes/loop_rooms.rs` 1860 (78% of the cap) |
| Content failing a check | none |
| Regressions people caught that no gate did | 0 (see *Caught by people*) |
<!-- drift:end -->

## Subsystems

| Subsystem | Grade | Evidence | Open |
|---|---|---|---|
| Level generation | B | Determinism fixtures; 20 recorded reference levels; the level sweep through the collision geometry (spawn, exit, open space, pickups, rebuild equality); zone-purpose properties; the expressive-range baseline (72 levels; the seed archive fills 20 of 36 cells, no cell over 12%) | — (a barricade no longer locks a pickup away: `PROPS.md` 2) |
| Rendering and look | B | 48 golden frames (exact, then ꟻLIP); look anchors per frame and over the set; `STYLE.md` bands; the material lint (16,661 materials on six levels); blind A/B reviews preferred the current look (8/8 realism, 8/8 dread); the rooms up close (`ROOMS.md`, R1–R6, each preferred blind by two reviewers, the old never): a field of view that does not stretch the walls on wide screens (`tests/settings.rs`), corners darkened where walls meet (the light map's shape, no texture read added), photographed details over the painted surfaces (woodchip, carpet, acoustic tile, plaster, concrete), a drop ceiling at a real tile's size with its fittings in the grid; partitions that lean now and then (R4, `tests/leaning.rs`: inside their tiles, their staircase inside the drawn wall and closed to a look, no prop inside one; blind reviews: ties) | Store department signs read as dark plates (an opaque material on a clear text canvas); no prop turntable sheets yet; the leaning walls went unnoticed by six blind reviewers (still frames read a few degrees as a wall a little nearer; whether they make the rooms less like a grid is for people); found by those reviewers on both sides, so older: hard-edged dark bands in a Lobby alcove's corner and over its floor (seed 7 level 4, region seams), a near-black strip where a Poolrooms back wall meets a higher ceiling (seed 7 level 5), a flat untextured soffit end and a banded beam face in the Poolrooms; the Lobby of seed 4242 level 4 is terracotta, not `STYLE.md`'s mono-yellow (a level palette's draw); level 37's views under a "Poolrooms" caption show concrete corridors, not white tile; the floors' stains are perfect ellipses (a reviewer of the marks, 2026-10-31, saw a hard-edged flat grey-blue octagon on a Lobby corridor's carpet, seed 77 level 7); flat-coloured cardboard boxes; found by that reviewer on both sides, so older: on the Stair Halls a vertical dashed column of flat, bright, pale-mint squares (the door's lit tower windows, read as a guide line), and the lamp heads there flat cream blocks with no fixture detail (iron lanterns since `MILESTONES.md` 11, preferred blind 3 of 3); no human verdict |
| Creatures | B | Goldens at 2/4/8 m and death scenes. Rebuilt from scratch (`ENTITIES.md`, E1–E10): the Hound, the Smiler, the Watcher, the one in the corner and the Reader on the new brain and bodies, their rules one test each (`tests/creatures_rules.rs`) and their counters held to arenas (`tests/creatures_arenas.rs`: the Hound with its counter 99% untouched, without 9%; the Smiler 99% and 15%; the Watcher 100% and 0%; the one in the corner 100% and 67%; the Reader 100% and 4%), and once each has learned, as well as before (the Smiler, the Watcher, the one in the corner and the Reader 99–100%, each seen doing what it learned in 83–100% of trials; the Hound heard first 59% against 60%); fairness shared by every new creature (two tells before a strike, none within 2 s of a notice, a first meeting that knocks you down, never through your nerve, and sends it away); the Watcher a cloth-draped sheet among furniture under the same sheets; the Reader never touches her sheets (`tests/story.rs`); who lives where by the places (`tests/ecology.rs`); what each kind learns, one test each (`tests/creatures_rules.rs`), saved with the run; the director's empty build-ups and its withdrawal obeyed (`tests/pacing.rs`: no peak loops); the old creature code gone; beatable (`tests/beatable.rs`: levels 4–13 of four runs, the rule-follower, the explorer and the rusher each 32 of 32 with no death (E10), no hard rule broken; the whole run live, three runs to level 14, two deaths on the way); what each kind learns now happens within a run (the explorer's Smiler, three runs of four, E10); a dark zone takes your nerve no lower than 40 (`player::DARK_FLOOR`); a touch kills only a mind already spent (`creature::SPENT`); blind reviews (E10, both orders, a fresh reviewer each, `tests/review/verdicts.jsonl`): against the old creatures the new Smiler is preferred at 2, 4 and 8 m and at its death, the new Hound at 2 and 4 m and at its death; the Smiler's and the Watcher's bodies redone after a first round lost, each new body preferred to the one it replaced by two reviewers (the Watcher at every distance and its death, the Smiler at 2 m, ties elsewhere); a level's frames the same whatever the thread built before (every texture paints from a stream of its own: `tests/story.rs`); a lore and a code review of E1–E9, their findings fixed (the Watcher's kill, the Hound's ears in the canon, the one in the corner at rest nothing to the director, deaf, and never coming in a relief, the torn notes that taught the rules now hint); the one in the corner's hum heard, a held note at 160–210 Hz, 4.4–5.2 dB over the room in its band on eight streams and heard to stop (`tests/listen.rs`; since E5 each note had been a pluck under the lamps' 120 Hz, inaudible, and its test had passed on the director's heartbeat) | The rule-follower reads a creature's mode for the sign it gives (an upper bound on a player); those who know no counter finish as often (the explorer met them 33 times and never died, E10, since a touch kills only a spent mind; at E8 36 contacts cost it one death: whether being caught costs enough is for people); whether a kind learns too soon (2–3 times) is for people; once a Hound is sure of you at 6 m, only 60% get away untouched (its counter is not to be heard); the lamp's battery lasts 25 s lit, and most of the bots' time in the dark on a spent nerve is with a dead battery (whether players run out is for people); the Reader's face at its death is pale under the death scene's light (its staring eyes and inked mouth read, its skin does not); the one in the corner's Thatcher face reads only as wrong, not yet as upside down; against the old one, the Watcher still loses at 2 and 4 m and at its death (its tall robe with a face showing through, against a sheet read as a bedsheet ghost among the sheeted furniture) and the Hound at 8 m (pale against the wall); the one in the corner and the Reader have nothing to be judged against (people must); reviewers split on the Smiler's sheen (wet read as latex, dull as a body suit; it ships dull); in the death frames that look up, two reviewers took the hanging broken ceiling tile (near black, `world/build.rs`) for a hole; the lore review's sameness: three tells are a sound that stops (the Hound's scratching, the corner's hum, the Reader's whisper), the Reader and the Smiler are both beaten by stillness in the dark, the Smiler idles facing a corner like the one in the corner, "tall" is said of the Smiler, the one in the corner and the staff, the Watcher's learned failing lights are also the director's sign, and the Reader's inky, paper-stuffed body nearly answers who lays the flat sheets; whether they frighten is untested with people The first-sight sting (`entity_sighted`) plays the moment you face the Watcher among its sheets, so it says which sheet is the one (found by `lore-keeper` on the asides, A6, which say nothing at its sight) |
| Pacing | B | `pacing.rs`: first build under 150 s, calms ≤ 95 s, reliefs ≤ 50 s, ≥ 4 peaks; deaths per peak about 0.11 for a cautious bot. The Building's floors (G2, G6): the first siren 48–76 s into a floor, a 21 s siren, the fog up 20–40 s, quiets of 38–55 s; a floor crossed in 68–123 s of walking meets a siren on 11 of 12 floors, sealed within 7.9 s of the warning, no death. Personas (`personas.rs`): bots that ignore the creatures die in 6 of 9 runs; 20% of the stalker's signs lead to a chase within 20 s; peaks ≤ 60 s, reliefs ≥ 8 s | A stalker that stays far off repeats the same sign (distant steps) up to 7 times in a row: the director picks a sign by distance alone; a player who goes straight to her camp and the door (under two and a half minutes a level) mostly leaves before the first peak (the whole run live: a peak on 9 of 29 levels); no human data |
| Sound | B | `listen.rs` (opening, signatures, the notice, the hush); `loudness.rs` (integrated ±1 LU, range ±3 LU, true peak ≤ −1 dBTP, repetition); ambient place tests; `audio_smoke.rs` and `ambient.rs` render the game's whole graph offline as they go, and fail on a panic in the render (since code review 3: on the silent clock they had rendered nothing since `2b6e022`) | Nobody has listened since the place-based sound pass; the agent cannot hear. The Building's floors take the "normal" weather's whole ambient pool, so a signature there can be steps, a laugh, a scream or a piano, against "nothing walks there" (`LORE.md`; found by `lore-keeper` on the asides, A5, which say nothing to them there) |
| Voices | C | `docs/VOICES.md`: a pipeline on locally run text-to-speech models licensed for sale (14 models pinned, `voice doctor`); the first persona (you, the cry for help) with a designed voice and a terrified reference; a bake-off of four models (Chatterbox 10/10 lines, VoxCPM2 8, Ming-3.0 7, Qwen3-TTS 2); every take judged on words, emotion (arousal, valence), pitch, tone (CLAP), voice (ECAPA), synthetic (AudioSet), the evaluation held to seven known answers (`voice selftest`); every shipped line in the manifest and the index; the cry heard 25.9 dB over the room, never clipping (`listen.rs`); your asides (`you_aside`, his voice under the breath: 57 lines over twelve uses, chosen by a voice director, `audio::asides`): the evaluation held to five more known answers (all twelve right), every line passing its gate; the director's rules held by its unit tests, `tests/asides.rs` (a floor's arrival, sudden sounds, a retry and a Continue never saying a line again, every hook telling it), `tests/pacing.rs` (nothing chosen said in a peak, the relief in the relief, the Building's siren and cell; eight runs: 2.5 lines a 4-minute floor, every run's words its own, 9 % shared between two), the arrival heard 18 dB over the room and 7 dB under the cry (`listen.rs`); every line through `lore-keeper` before it was made | Nobody has listened to a line; the emotion recognisers hear frightened shouting as anger (arousal gates instead); speaker similarity cannot tell his shout from another man's; CLAP hears synthetic screams as cartoons, and how far that is CLAP and how far the takes is unknown; one persona only (the Stairwell child next); the asides: nobody has listened to one; CLAP cannot tell a mutter from a narrator on lines this short (no phrase, nor AudioSet, parts them); the startled ones are said calmer than asked (a clone of a calm clip; an emotional reference for *startled* if an ear finds them flat); "What's that siren for?" passed its tone on one take of eight |
| Canon | C | `LORE.md` (the canon, with delivery channels; M.'s descent); `PILLARS.md`; one `lore-keeper` review (M.'s notebook, 2026-10-26: a contradiction with M.'s Rule 1, promised marks the world lacked, two threads that were orders, lines lifted from older notes; all rewritten); one of the UI's words (2026-10-27: death and pause lines that gave tips or stated rules, "NEXT" and "story arcs" in the notebook, the debug key on the Controls page, three names for the light; all fixed); two of the Building's text (Megastructure B6, its findings all taken; B8: the cleaners and the self-assembly named in more places than the canon allows, warm machines blurring the warm door, a figure on flights "nobody has stood on", storeys poured where galleries are drawn, the Well's windows and drop said three times; all fixed); one of G5's knock (all taken) and one of the Building's second pass (G8, 2026-11-09: her second page's knocking "the whole time it was up" where it now stops, a door that gives said to let in fog that may stand below it, the Well's open room against "a door behind every one shut from inside", a resident's way down "under the lit windows" that are everywhere, what knocks never said to be unseen; all fixed); three of the props' canon (P7, 2026-11-09, *What still burns*: the bench said to be on every floor and the rug's edge, its hum and screen glow marking a safe place, its chime, a heartbeat it was said to hold off that beat every frame there, a haunted floor's candles blown out on floors where no hand switches a set off, which flames are lit said from part of the draws, her lamp against "nothing burns down", sameness in "awake", "lights its corner", "thread" and "a beat late"; all fixed) | The older notes have never been reviewed; the second pass's round found the cell's prompts giving away what the place does not show (`MILESTONES.md` 13) and sameness: every place off the Building's way holds the same water or battery, "lit" names three of the residents' landmarks, and the cleaners' log tells the red door's whole chain (kept: an account of one night, never why); no human has read the notebook; the mannequin (a prop and torn note 17) is not in the canon |
| Story and wayfinding | B | `tests/story.rs`: a level is the same place whatever was read (G1); her trail holds on 13 levels and all four anomaly floors, each of her marks in sight of the last at its own kind's distance, in her hand, fewer sheets deeper down (`her_marks_change_as_she_goes_down`: 85/53/27/14% sheets over 39 floors, measured at G1); the place's answer to a cry along the way to the door, without a word (`crates/game/tests/player.rs`), never footprints on the Building's floors (`tests/secrets.rs`); her notebook follows the descent on eight runs; taking her page (journal, thread, retry, Continue); the page wakes the place (log: page, wake, build); trails that are not hers; the last camp and the ending. `the_door_is_heard` (the door alone, +1 to +18 dB over the bed from 2 to 9 tiles); the playthrough plays through the ending; plan maps in `target/story/`; a blind review of her marks along the walk (36 views, both orders, against the trail of sheets alone): hers preferred 8, the sheets 2, 26 ties (small by design) | No compass: whether a fresh player finds the door, follows her trail, notices the flat sheets and reads the notebook as a story is untested with people (`PLAYTEST.md` asks); whether her small marks are found by those who look or missed by everyone (deep down they are faint by design), and whether the lights' answer reads as an answer, is for people; the place's answers and her marks were judged by one blind reviewer on still frames |
| UI and controls | B | `tests/controls.rs` (reach, a held key fires once, running and using things together, no stutter, crouch, pause, what you face, the early E, Command, reading while walking), `tests/settings.rs`, `tests/menus.rs` (title, pause, settings, controls, leaving, death), `tests/rest.rs`, `tests/faces.rs`; the HUD test's frames; `--screen` renders every list over a level; a code review and a lore review of U1–U7 | No human has played with these controls or read these screens (`PLAYTEST.md` asks); no gamepad; brightness is a lift or dim over the frame, not a gamma |
| Code health | B | fmt and clippy `-D warnings`; structural tests (layering, determinism (now over `story/` too), sizes, docs, ledger) | `ui/draw.rs` is 1,692 lines (cap 2,400); the placer registry is repeated in nine older test files and `main.rs` (newer tests share `tests/common`); `ambient.rs`'s `weather_sounds_come_from_their_places` failed once in a full suite run (unseeded, counts only each step's last log entry; 30 of 30 alone) |
| Determinism | B | Seeded level builds (fixtures, reference levels, the level sweep's rebuild check); bit-identical `--fresh` frames; replays reproduce 2 × 2,400 frames of a bot run (a clock read injected into the director is caught at its first frame) | The director's first timer depends on what was drawn from the per-frame stream while building (fine per process; replays run in fresh threads) |
| Performance | B | `cargo xtask bench`: budgets in `PERF.md` (build ≤ 600 ms, first frame ≤ 100 ms, update p99 ≤ 4 ms, GPU p99 ≤ 6 ms at 1280×800, draw calls ≤ 2,000, triangles ≤ 2.5 M); history with change-point detection; GPU p99 1.4–2.9 ms today; the creature pack (two of every roaming kind hunting you) timed beside the levels, with its own budgets and history (`perf/creatures.csv`) | One run on record (change points need eight); the Showroom is 7× the triangles of a maze level |
| Assets | B | Blender sources in `assets/src/`; `cargo xtask assets props` rebuilds 2,423 of 2,546 prop parts exactly (repeatable since noise3 and seeding); the Building's nine photographed textures rebuilt from their CC0 photographs by `cargo xtask assets textures` (each source checked by its SHA-1); `MANIFEST.json` (sources, licence, generator, fingerprint) checked for meshes and textures; mesh lint over every part and the parts table | 123 frozen prop parts keep shapes from an unrepeatable build; the two extinguisher lever parts are 68% slivers (`DEGENERATE_EXCEPTIONS`); the old creatures (`creature_meshes.bin`: the shadows' body, the Stairwell's face) are kept as stored, no build here makes them (their scripts need rig dumps nothing here writes); their replacements' bodies (`bodies.bin`) rebuild here byte for byte (`cargo xtask assets creatures`), so far only the test body |
| Reviewers | C | Calibrated on known answers (`cargo xtask calibrate`): strong damage 12/12, kappa 1.00; mild damage 9/12, kappa 0.45 — mild holes, smears and washed-out light caught, identical pairs judged ties | Mild over-saturation missed 0/3 (the gates' colourfulness anchors and `STYLE.md` bands are the authority there); no human verdicts to calibrate against yet |
| The Building (levels 1–3) | B | `tests/building.rs` (the navigation measures over 40 seeds a place: detour, flights, choices, dead ends, a cell within 8 tiles of every floor, the door seen from near the arrival; since G1 the floors are 28 tiles a side and the way a median 83–87 tiles, half what it was, every band held over 300 seeds a place; since G2 a cell within 8 tiles (0.43 of the 21 s warning at a walk) on every one of 2,000 floors a place, 14–15 cells a floor, a booth for one where no building is near standing only a storey over its walk); `tests/building_floors.rs` (G3: the way and what is off it on 72 floors, every dead end holding something, her trail, notes, water, batteries and the figure's places in their bands; G4: the rooms off the walks, 2–3 a floor in the Well and 0–3 elsewhere, reached, closed on three sides a storey high, off the way, each holding water or a battery, their fronts stopping a body beside the doorway); `tests/height.rs` (heights, voids, slabs on a hand-made floor); `tests/assembly.rs` (the self-assembly's stages, a sealed cell keeps it out, outside it kills, a cell within reach of the warning and `CELL_REACH` tied to it, a six-minute pacing log a floor, every time of it half the first pass's since G2; what knocks at your cell since G6: silent is safe, an answer brings the wheel turning, held is safe, let go is the fog, an unsealed cell lets the fog in; a visit that does not give leaves the wheel at rest shut; since `MILESTONES.md` 13 its prompts follow the door: the hold only from the wheel's first pull, held within its play by a press a reaction after it; its knock to answer until it is heard going; and the door to open only once its going, heard, is over); `tests/knock.rs` (G5: four ordinary doors a floor answer once with water, a battery or a resident's note, a red door's siren within seconds, a taped door never, nobody while it is up, taken stays taken, the plan its own); `tests/listen.rs` (the siren; a knock answered and a red door knocking back, away from the lamps and beside them, 0.4–3.9 m from a cell's lamp, where the ballast's hum is the bed and they are heard above it, which fails with the answer's sounds at gain 0 (`MILESTONES.md` 12); the wheel tried and the door giving, which fails with both sounds at gain 0; what knocks at your cell heard beside the cell's own lamp, in the band above 1.2 kHz: its knocking, no knock after an answer, and its going, unanswered and let go of the wheel, which fails with the going's sounds at gain 0, with the knock's click and ring at gain 0, and with an answer hushing nothing; once it has gone the audio graph as it was, no knock's gain held, code review 4; held, each pull against your hold heard over the hum, a creak of steel under load at 1.9 and 2.9 kHz, +16.4 to +16.9 dB in the band, which fails with that creak at gain 0, code review 4b; the hum at the nearest lit lamp of your storey, which by the floor plan alone had been another storey's at a sixth of the walks' tiles, `tests/building_floors.rs`); `tests/pacing.rs` `the_building_paces_its_floors` (a bot crossing twelve floors at the playthrough's pace, sheltering at each siren and never answering: no death, sealed within 7.9 s of a 21 s warning, a siren met on the way down on 11 of 12 floors, 3 of 12 before G2); `tests/presences.rs` (the figure across the void out of reach, turning away, going at the siren; since G7 a person, MakeHuman CC0, standing on its perch's floor, on a level landing on the Stair Halls' flights and in the Well only where seen from the hips up; the door's light from the arrival on 11 of 12 floors; the lift; the Works' last door into the Lobby); `tests/story.rs` (the Building's notes only on its floors; her camp in a cell); six golden frames and the three places' `STYLE.md` bands; blind reviews (`tests/review/verdicts.jsonl`): its photographed surfaces over the canvas ones on every floor (B9, three rounds), the booths a storey over their walk over the tall ones 6 to 0 and 6 ties (`MILESTONES.md` 1, after the tall ones lost 8 to 2 to the floors before G2), the figure as a person over the old one on 15 of 15 images (G7), the rooms off the walks over the bare walls 5 of 5 (G4), the small faults mended over their old selves 20 to 0 and 1 tie (`MILESTONES.md` 11, two rounds: the Well's rooms lost 3 to 0 with their windows lined over and won 8 to 0 with their joints covered), the floor occlusion from each wall's own floor over the old one from y = 0 9 to 0 and 1 tie (code review 3), and at a walk's end from the walk's own floor, the thin streak down a mass's face gone, 12 to 0 and 1 tie (code review 4); code, lore and level reviews (Megastructure B8; the code review after `MILESTONES.md` 1–4; `lore-keeper` rounds at G5 and G8); its surfaces linted on all three floors (`tests/materials.rs`) | No person has played it: whether the way down is a puzzle or a chore, whether the siren reads as a warning the first time and 21 s is long enough to understand it, whether waiting in a cell is dread or a chore, whether the figure is noticed and how often is too often, whether players knock at a door at all and read the crosses and the tape before they learn them, whether they answer what knocks at their cell, hear the wheel as a threat and find the hold, and whether a door that gives is fair (`PLAYTEST.md` asks); the siren, the knock and its answer (since `MILESTONES.md` 12 a rap's click and ring and a heel above the lamps' hum), the wheel, the door giving, and what knocks at your cell, its knock's click and ring, letting go of the wheel and going (`MILESTONES.md` 13: something wet drawn off the door in four breaths, not steps) are synthesized and have not been heard by a person, nor the re-recorded loudness, nor the held wheel's creak of steel (code review 4b); the small faults the blind reviews of `MILESTONES.md` 1 and 4 found are mended (`MILESTONES.md` 11; `tests/building_floors.rs` keeps the lamps apart and the rooms darker than their walks), but a room's darker light was not seen by its reviewer (measured, its floor 0.95 of its walk's brightness before, 0.89 after); found on the way: the cells' benches are flat dark blocks yet; beside a Well room's front, where its poured wall meets the panels, the facade's joint shows as a dark band with ragged pale edges; the hatch at a resident's door is drawn only while it is open; six of 2,000 Works floors keep G1's band misses (a detour of 1.78, two flights, no choice: the lattice's tries run out); the figure's visibility is judged by rules (the solids and what the dressing builds over the walks), not by the renderer, and on one Well floor (seed 7) the lift's chosen view is still hidden by the galleries, so the rule misses something there; a multi-level `--dump-level` keeps level 1's zone caption on the later levels' views, and makes no exit view for some Works floors (the door reached along its wall); the level sweep found a battery no one can reach on seed 2645350805, level 3 (the Works, at (-70.1, 26.7); it fails on `614b20d` too, before the photographs); in a Works zone view (4242, the arrival's flight) two flight tiles side by side have their treads out of step; the photographed surfaces are judged by a model reviewer only, and the nearest Well windows are soft up close (the panel photographs hold 128 px a metre) |
| Playtesting | D | The kit: `PLAYTEST.md` (script, debrief, tension trace, A/B by ranking), `playtests/TEMPLATE.md`; `--record` / `--replay` with ↑ / ↓ tension marks; `cargo xtask heatmap` | No human has played this build since the tension pass |

## Content families

| Family | Grade | Notes |
|---|---|---|
| Creatures (Smiler, Hound, Watcher, the one in the corner, Reader) | B | All five have goldens at 2, 4 and 8 m and a death scene, their rules one test each, their counters held to arenas, learned and not; the Smiler and the Hound (but at 8 m) preferred blind to the old creatures, the Watcher to its first body but not yet to the old Watcher close up (E10) |
| The one behind the glass (`WINDOWS.md` 4) | C | Its rules one test a line on real floors and its counter held there (`tests/glass.rs`: with it 100% untouched, without 48%); five goldens at its window (32 m, 8 m, at the glass, inside, its kill); its glass sounds each heard over the room (`tests/listen.rs`); its body preferred blind to its first draft by two reviewers (10 of 12 views each); a blind let down at the glass keeps it out on 13 of 13 floors, and learned (its lamp 1 m lower, its light 4.3 m in for 2.6 m) its counter holds as before | Whether a player ever stares long enough, whether letting a blind down feels like their idea, and whether it frightens, is untested with people; its face up close is pale and wrong but smooth; from inside, its lamp coming down is seen only by its light on the floor; the lore review's sameness: a face pressed into something that gives (the Watcher's sheet), a hand at a glass (the dead vending machine's print), and rules about your eyes now many (the Watcher, the one in the corner, the figure, the Face, the vending display, this one); its tells were moved off breath and the Smiler's click (its tick a knock and the pane's ring, a squeak, its coat's rustle; the place's own drips were too near); its name has the one in the corner's shape, both placed and never sent and both beaten through your eyes, and its touch is the shared one the Watcher's is; the streetlamp is a third ballast at 120 Hz |
| Anomalies (Stairwell, Showroom, House, Loop) | C | Fixture tests and the sweep; the Showroom has a golden view; the staff, the shadows and the seated man have goldens of their own (E9), the staff and the seated man on new bodies a blind review preferred (`tests/anomaly_bodies.rs` holds the staff to its rig and walks it untorn); the shadows keep their old body (a new one lost or tied four reviews) |
| Zone purposes (14) | B | One golden view each; style bands measured from 631 views (five purposes with too few views share the pooled band) |
| Prop families (10 themes, 649 placers) | C | Material lint, smoke tests, the look settle pass; a census of every kind (`--prop-census`, `research/props-census.md`, 2026-10-31): 652 kinds on 272 floors graded by seven reviewers; P1 (`PROPS.md`): the 35 everyday cartoon kinds remodelled as the real things, P2's 66 common ones and P3's rare, the 112 and the 43 the census never saw (22 batches, each 14–0 to 16–0 against the old; blind reviews, P1 and P2: 24–0, 30–0, 17–0 with 2 ties, 13–0, 15–0, 9–0, 11–0, 15–0, 16–0, 15–0, 15–0, 14–0, 12–0, 16–0, 13–0; 12–0 for the barricades); the reference levels hold the settled walls, unchanged by P2 on 110 floors; the barricades seal real openings (`tests/barricades.rs`); six props have verbs (`tests/usable.rs`); P4: every prop wears by use (dust on what faces up, the floor's grime at its foot, convex edges rubbed and chipped), preferred blind 12–0 and 10–0 over the clean props; since code review 2 the thing's own, measured from its own floor, and the same whatever level was built before (`tests/prop_wear.rs`); M8: the small faults P3's reviewers found mended, each against its old self in blind rounds (as shipped 27 preferred, 0 the old, 14 ties over 41 images), small things on the floor sit in contact shadows (`visuals::contact`, preferred 9 of 9), a mirror shows the lamp as a hot spot, not glare, and the dust motes are motes, not glowing specks; P7: what still burns is canon, written from the placers' own draws (`LORE.md`, *What still burns*: what was left alight burns on and never burns down, what was cold stays cold, the censer among them), the place's own flames (the candles on the bench, a haunted floor's) where the verbs are, below the Lobby and never on a floor with rules of its own, and a candle blown out stays out (`tests/flames.rs`); the bench says nothing, its hum comes from it, and by it the low-nerve heartbeat holds off (`tests/player.rs`) | The wear by use (P4) is seen on furniture and on things handled, not on the smallest things on the floor nor on a print on a wall (ties in its two blind reviews); M8's ties (the nail polish, the umbrella crooks, the trout's fins, the conker husks, the clips, the mask's ribbons, the atomizer's glass): whether the husks still read as petals is for people; the wall placers take a pool's curb for a wall; no person has used a verb or walked into a barricade; the older interactions that break the house style (`research/props-interactions.md` 1.3: the light switch's "(bad wiring)", the vacuum's and the payphone's dice and lines, the shaft's lines, the staff ID's names, the red and green button), the silver tray's faint glow and the nerve's recovery chime and flash (most often heard on the bench's rug) are still there; whether a candle burning in an empty shrine reads as wrong or as a game's light is for people |
| The vending machine (`VENDING.md`) | B | Rebuilt as a real glass-front snack machine (V1: the Blender parts of batch `v01`, its goods and their printed wrappers, its light, its weight), preferred blind to the old COLA-Z in every frame that showed both (two reviewers, both orders); its nine rules built (V2, V3, V5), each held by a test in `tests/vending.rs` that fails with its rule broken, and where it stands (offices, halls, the Poolrooms, never a zone left bare; none on the Lobby, the Showroom or the Borrowed House); its sound (V4: its hum, its coin, its motor, its bang) held by a listen test that fails with the hum at gain 0; its canon through three `lore-keeper` rounds (V5), what each found fixed | No person has played with it or listened to it (`PLAYTEST.md` has its questions); goods spilled by its fall take the floors' pickup lines ("SOMEONE LEFT IT FOR YOU"); its lit interiors are bright and colourful next to the rooms' muted palette (a reviewer's caveat, held by the look gate); a fallen one's back is seen only from above at eye height (its louvres faint in a dark room) |
| The windows (`WINDOWS.md`) | C | Rebuilt as real windows cut into real walls (W1: wood double-hung and office strip windows from the Blender parts of batch `w01`, their glass, blinds and wear; the view of a field in fog from a CC0 panorama traced per pixel, its streetlamp drawn beyond the glass and its light thrown on the floor; the lamp heard through the glass), preferred blind to the old framed picture in all 22 views that showed both (one reviewer, both orders); held by `tests/windows.rs` (where they stand, the cut, the same on every build, the light in its own room, the same view from every window, later deeper) | No person has looked out of one; the view is one photograph for every window (by design: the same place) and soft close up (23 px a degree); what is beyond the glass is drawn as a thin slab in front of the view, and a part of it nearer than 2 cm could cross a bar's back edge at a graze; the one behind the glass has a row of its own below |
| Sound (mix, ambient) | B | Loudness and listening gates |
| The Building's places (the Well, the Stair Halls, the Works) | B | Two golden frames each, `STYLE.md` bands, the navigation bands; their own notes, surveys and arrival lines; their surfaces photographed (the Well's panels and lit windows, the Stair Halls' granite, the Works' formwork and factory slab), preferred blind on 32 of 32 views in the last rounds; each its own rooms off the walks (a resident's table and chair, a stone bench in a pier's niche, an operator's desk), preferred blind 5 of 5 to the bare walls (G4), furnished since `MILESTONES.md` 11 with props' parts worn as props are (a stone bench, a kitchen chair, a steel desk, its monitor and a stool), the Stair Halls' niches roofed with a granite slab, each preferred blind to its old self; no human verdict |
| Story (trail, camps, notebook, false trails, ending) | B | `tests/story.rs`; ledger entries `story/*`; her marks (`SECRETS.md` S1) in seven kinds, held by the trail's checks; no human verdict |
| What is put away (`SECRETS.md`) | C | The run's plan and each floor's (`tests/secrets.rs`: off the way, apart, the Lobby only what grows, the Building never walked on, taken stays taken across a retry and a save, the hole, the dish, the first scratched sign); what each thing does held by a pure test and by the player's own (`crates/game/tests/player.rs`, `tests/vending.rs`), bounded (two steps a system, never below what fairness rests on, a chase under your sprint as it is); two `lore-keeper` rounds; two blind reviews of the places and things (23 views, both orders, against the floors without them): the first preferred them 10 to 3, 4 ties (all three the lifted tile floating under an intact grid: it now hangs from its edge out of a gap), the second 19 to 0, 4 ties, every repeat the same | No person has found, eaten or noticed anything (`PLAYTEST.md` asks); the things are primitives and the props' parts, judged by a model reviewer only; effects small by design and so perhaps never noticed; the lore review's sameness: "hole" now names three things (the torn note's way down, a hole as a way off the last floor, the hole in the wall), wet is many things' image (the prints, the corners, the glass's coat, the Hound's hair), chalk was four hands' (the sign moved to scratching); the second art review's doubts: the hanging tile is smoother than its neighbours and its gap a flat black, the chair one flat colour, and a thing on a cell's bench too small to read from the cell's door (the bench itself an untextured box, older than the secrets) |
| UI (controls, menus, HUD, cards, notebook, type) | B | Ledger entries `ui/*`; no human verdict |

## Caught by people, missed by gates

The steering loop (roadmap 5.3): every regression a person found that no
gate caught, and the check added so it cannot slip through again. The drift
sweep counts the rows; the count should fall.

| Date | What a person saw | Which gate should have caught it | Check added |
|---|---|---|---|

## History

- 2026-11-10 — Code review 4b: two of what milestone 13 left open. Held,
  the wheel's strain had been all under 1.2 kHz (a 700 Hz creak, the
  bolts' thud), so beside the cell's lamp it could be seen and not heard;
  now each pull against your hold is heard over the hum too, the spokes
  slipping and gripping in your hands, a creak of steel under load at 1.9
  and 2.9 kHz (+16.4 to +16.9 dB in the band, +0.9 to +2.2 without it),
  oscillators only. The ballast's hum chose its lamp by the floor plan, so
  on the Building a lamp of the walk over or under you stood nearest at a
  sixth of the walks' tiles (460 of 2,786 on twelve floors) and set the
  hum at full strength at one in twenty; it sits at the nearest lit lamp
  of your own storey now. The rooms (one floor) unchanged, the loudness
  scenarios the same to the tenth.
- 2026-11-10 — Code review 4 (`fa45f64..ca6ca87`): the grades stay. The
  light map's floors were blended with the void by its linear filter:
  within a texel of a walk's end a mass's face over the void darkened in
  a thin streak from the walk's floor down to the lowest storey (a
  hairline through the Stair Halls' granite, seen from a walk 6.7 m off),
  and the feet of doors, jambs, balusters and steps beside a gap read a
  floor below their own. On the Building the map's B now says which
  places have a floor and the floor is A over B, and a thing that takes
  wear by use stands on its own floor: preferred blind 12 to 0, 1 tie.
  What knocked at your cell and went unanswered kept its knocks' gains,
  and with them their voices in the audio graph for the rest of the run;
  let go now, held by a count of the graph. The light map made its floors
  anew at each of a fog pass's 18–32 rebuilds (0.37–0.39 ms each); made
  once a level now (0.15 ms). The rap is one table both knocks read, the
  same samples.
- 2026-11-09 — What the cell's prompts give away (`MILESTONES.md` 13):
  the Building stays at B. Answered, the hold had shown a second before
  the wheel turned, and what knocked went without a sound, so the screen
  told what the place did not. The hold now shows from the wheel's first
  pull, which moves the wheel through its play (0.4 s) before it turns
  the screw, so a press a reaction after it holds the wheel before the
  door begins to open; and however it goes it is heard going (its hold
  slipping from the wheel, then something wet drawn off the door, four
  breaths), the door to open coming back only once it has gone.
  Knocking, it goes only once its last knock has rung out, its knock to
  be answered until then; answered, it knocks no more. Its going carries
  above the cell's own lamp's hum (+22 to +24 dB in the band), from the
  visit's own stream, no per-frame draw added; so does its knocking now
  (+19.4 dB, a knuckle's click and the leaf's ring, where it had been
  +6.4 to +9.6). Two `lore-keeper` looks: the going was first steps going
  away (against the canon, nothing walks the Building, and the same as a
  resident's steps), and its raps rang on after an answer and its knock
  went from the screen before it was heard going; all remade.
- 2026-11-09 — A knock heard by a lamp (`MILESTONES.md` 12): the Building
  stays at B. Within a few metres of a lit lamp the ballast's hum is the
  bed (−35 dB against −43 dB away from the lamps, nearly all of it under
  1.2 kHz), and the knock and someone at the door had been made in its
  band: beside a lamp your raps, the red door's raps back and the steps
  behind a door were not heard, and the thing pushed out, plain above
  1.2 kHz, was measured over the whole mix, where the hum's lines outweigh
  it. A rap now has the knuckle's click and the leaf's ring and a step its
  heel above the hum, with no draw added; the listen test knocks beside a
  lamp on three floors and measures in the band where a knock is heard.
  The second sound lost under the lamps (the one in the corner's hum was
  the first, E10): `add-sound` and the audio rules now say so.
- 2026-11-09 — Code review 3, its second commit: floor occlusion from the
  floor under a surface. It had darkened every wall by its height over
  y = 0, so on the Building's storeys under 0 (they run from about −18 m
  to +21 m) every face that does not face up lost 45% of its indirect
  light, above 0.45 m no wall foot darkened, and a pool's walls were dark
  throughout. The light map's alpha now holds each place's floor (the
  Building's, a flight's along its run, a pool's), read from the sample
  the shader already takes: no texture read. Preferred blind 9 to 0 with
  1 tie (the pool, seen through its water).
- 2026-11-09 — Code review 3 (`a443d89..fa45f64`), its first commit: the
  grades stay. Two audio tests had tested nothing since `2b6e022` (the
  silent clock renders nothing, so no panic in the render could fail them):
  `audio_smoke.rs` and `ambient.rs` render the game's whole graph offline
  now, and fail with a panic put in `Engine::fill`. The windowed game with
  no output device had no clock (its time stood at 0, what it was sent piled
  up): it runs on the silent clock, which queues nothing it is sent and
  whose thread ends with its context. The relight's leftovers are gone (its
  debug rows and always-0 tallies, its tables, milestone flags never set,
  the screen glow no code turned on). The bench's hum walks no sight line
  beyond the 3 m it is heard in; a small thing's contact shadow goes with
  it past its draw distance (it was left on the floor); the Works' pipes
  and elbows are 8-sided (their floors 36% and 31% fewer triangles) and the
  rooms' furniture shares its geometry, so the instancing takes it together.
- 2026-11-09 — The Building's small faults (`MILESTONES.md` 11): it stays
  at B. What the blind reviews of the second pass found on both sides
  alike is mended at its cause: the Works' pipes run as pipes, the Stair
  Halls' lamps are lanterns and no two stand in one place, the flights have
  soffits, and the rooms off the walks have a ceiling, a bench and furniture
  that read as real, no seam down their corners and less light than their
  walks. Preferred blind 20 to 0 with 1 tie; the first lining of the Well's
  rooms lost 3 to 0 (it took their windows) and was reworked. No draw moved.
  Found on the way: the floor occlusion measured from y = 0 on floors that
  are not there.
- 2026-11-09 — The props made real (`PROPS.md` P0–P7, `MILESTONES.md` 7–9):
  they stay at C. Of 652 kinds the census framed, every cartoon one is now
  modelled whole from a real one (P1–P3, batches `r01`–`r18` and
  `t01`–`t22`, each keeping its draws call for call and preferred blind to
  its old self), barricades
  close real openings (P5), six kinds of prop can be used (P6), every prop
  wears by use (P4), small things sit in contact shadows and the small
  faults the reviewers found are mended (M8), and what still burns is canon
  (P7). Held by the reference levels (every placer's objects), the material
  and asset lints, `tests/barricades.rs`, `tests/usable.rs`,
  `tests/prop_wear.rs` and `tests/flames.rs`; their look by blind reviews,
  not gates, and by no person yet. Found on the way: a material shared
  between props and levels carried one prop's wear to the next level (code
  review 2); the candles on the bench burnt on the Lobby, in the spotless
  Showroom and in the Borrowed House, whose rule is light, said what they
  do on the screen, and at a nerve near its end set its heartbeat beating
  every frame; a haunted floor's candles were lit again from the lamp's
  battery, with a narrator's lines (P7).
- 2026-11-09 — The Building's second pass (Megastructure G1–G8,
  `MILESTONES.md` 1–6): it stays at B. Its floors and its self-assembly
  half as long, every navigation band held; every dead end holding
  something; rooms off the walks; a knock at a resident's door that
  answers, or brings the siren; what knocks at your cell a choice; the
  figure across the void a person. Held by `tests/building_floors.rs`,
  `tests/knock.rs`, `tests/assembly.rs`, `tests/pacing.rs` and
  `tests/presences.rs`; blind reviews preferred the low booths, the rooms
  and the person. Found on the way: the doubled booths closing the Stair
  Halls' void (now a storey over their walk), a dead end holding nothing on
  14 of 72 floors, a room's front no body met and rooms that moved every
  later draw of the dressing (review 1), a knock drowned by a cell's lamp
  (`MILESTONES.md` 12), and in the canon her second page's knocking that
  no longer goes on (the G8 `lore-keeper` round). No person yet.
- 2026-10-31 — Her marks and what is put away (`SECRETS.md`, S0–S4). Her
  trail is now in her own hand and in seven kinds (fewer sheets deeper
  down), the guide arrows and the notes' glow and wall clue are gone, and
  a cry is answered by the place (the lights, wet footprints, the door's
  call). Off the way, two to four places a floor hold something put away;
  thirteen things change you a little for the rest of the run and nothing
  says how. Held by `tests/secrets.rs`, `tests/story.rs` and the player's
  tests; two blind reviews of the places, one of the marks; two
  `lore-keeper` rounds. Found on the way: her marks nudged off their walls
  by `settle_placement`; the ceiling's tile grid read along the wrong axis
  (the lifted tile sat across two tiles); a radiator inside the Building's
  mass; a hung cap laid flat by Euler order; pale mushrooms deeper than a
  chase could be outrun (`chase_cap`); a felt cue that was the Watcher's
  own tell; and a full check's playthrough that was never seeded (a
  timeout at run 101, level 14, that no rerun repeated).
- 2026-10-30 — The vending machine (`VENDING.md`, V0–V5): C to B. Its
  rules are built and each held by a test that fails with the rule broken
  (it keeps its rhythm, goes dark when something comes, reads your eyes,
  gives a coin for nothing and nothing for a coin, sells a silence round
  it, is rocked loose and falls, fills in its dark, and something is
  inside the dead ones); its sound by a listen test; where it stands by
  two tests (offices, halls and the Poolrooms, never a zone left bare;
  none on the Lobby or the anomaly floors); its canon through three
  `lore-keeper` rounds, what each found fixed (the silence was floor-wide;
  it stood in the Showroom, the Borrowed House and bare Lobby zones; its
  almond water had a label the floors' bottles do not; a dead one could be
  set off through its wall; the warning label had been edited to fit the
  game). No person yet.
- 2026-10-29 — Leaning walls (`ROOMS.md` R4): now and then a straight
  partition with a free end leans 2–10° about its corner, inside its wall
  tiles, on a salted stream of its own; a staircase of boxes inside the
  drawn wall stands in for it in the solids, so collision, sight, sound and
  the light map are unchanged in kind, and `settle_props` hangs things on
  its face, turned. A blind review found one bug on the way (a probe along
  a leaning run grazed its side, slid a lamp 3 m and lost a corner's pinned
  fill; `tests/leaning.rs` now fails on any prop inside a leaning wall);
  after it, ties on every frame.
- 2026-10-29 — The rooms up close (`ROOMS.md`, R1–R6): a person's field
  of view (wide screens had stretched the walls like a wide-angle lens),
  corners that darken where walls meet, photographed details over every
  painted surface but brick and tile, and a drop ceiling of real 0.6 m
  tiles with its fittings in the grid, each preferred blind by two
  reviewers. Found on the way: the rooms' surface painters still drew on
  the per-frame stream (a hole E10's fix left; held by a test now); a
  floor laid in world space from the tiles' centres ran its saw-cut down
  every corridor; walls that bow a few millimetres show nothing under
  these rooms' mostly ambient light.

- 2026-10-29 — The creatures redesigned from scratch (`ENTITIES.md`,
  E1–E10): five rules on one brain, fairness as invariants, what they
  learn, beatable by a bot that plays the counters, on MakeHuman bodies;
  judged blind against the old ones: the Smiler and the Hound
  preferred, the Watcher only at a distance. Found on the way: a fix the
  code review proposed (paint the creatures' textures at start-up) left a
  level's frames hanging on what the thread built before; every texture
  now paints from a stream of its own, held by a test that fails without
  it. The Watcher's poses each put its face on another part of the cloth,
  so its face was kept fine and stained in one pose only. And the one in
  the corner had never been heard: each note of its hum a pluck under the
  lamps' hum, its listening test passing on the heartbeat the director
  played while its rest counted as a search; held now, and measured in
  its own band.

- 2026-10-28 — The Building's surfaces made from photographs of Soviet-era
  places (Megastructure B9), all CC0, judged blind against the canvas
  surfaces: the Well and the Stair Halls won at once, the Works only after
  two rounds lost (cast-iron plates, then whitewash). Found on the way: two
  tests set the run seed before `start_new_game` drew a new one, so they
  measured random floors (`presences.rs`, `assembly.rs`); the level sweep's
  unreachable battery on a Works floor (open).

- 2026-10-27 — The Building added at B (Megastructure B0–B8,
  `MEGASTRUCTURE.md`): three floors of a megastructure above the rooms,
  held by gates, not yet by people. Found on the way: a lintel that stopped
  a body below y = 0 in every cell's doorway (the level sweep), the door's
  call sounding from y = 0 on every floor, a listening claim that held on
  three floors by luck (57% over eight), the fog's texture drawing from the
  per-frame stream at a floor's start (every golden after level 1 moved),
  and the Works' machine housings drawn as 150 m towers while the solids
  stopped at their tops (found when a figure placed on one could not be
  seen).

- 2026-10-25 — first scorecard, written with the tooling milestones M1–M6.
- 2026-10-26 — Story and wayfinding added at B (gameplay milestones G1–G7,
  `GAMEPLAY.md`): held by `tests/story.rs` and the door's listening test,
  not yet by people. Canon stays C: one review of new text, none of the old.
  Found by the new gates on the way: a level built differently after a
  note was read (G1), small props missing from every headless view but the
  first (G3), an almond buried in a maze wall (G3).
- 2026-10-27 — UI and controls added at B (milestones U1–U7, `UI.md`):
  held by gates, not yet by people. Found on the way: the run stuttered at
  zero stamina (three frames of running every 0.9 s), key repeat strobed
  the flashlight, a focus loss reported the lost pointer twice, the
  title's shade covered every golden for one check, and the always-on
  HUD had lifted every view's measured luminance by about 0.01 (the STYLE
  floors were re-measured).
