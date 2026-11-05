# Quality scorecard

A grade for each subsystem and content family, the evidence behind it, and
what is open. The drift sweep rewrites the *Evidence* and *Open* columns; a
grade moves only with an entry under *History* saying why. The ledger of
individual content is [`content.json`](content.json); the tooling that
produces the evidence is in [`TOOLING_ROADMAP.md`](TOOLING_ROADMAP.md).

Grades: **A** held by gates and confirmed by people · **B** held by gates,
not yet confirmed by people · **C** partly covered · **D** not covered.

Last updated: 2026-10-27. The *Latest sweep* section is written by
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
| Rendering and look | B | 48 golden frames (exact, then ꟻLIP); look anchors per frame and over the set; `STYLE.md` bands; the material lint (16,661 materials on six levels); blind A/B reviews preferred the current look (8/8 realism, 8/8 dread); the rooms up close (`ROOMS.md`, R1–R6, each preferred blind by two reviewers, the old never): a field of view that does not stretch the walls on wide screens (`tests/settings.rs`), corners darkened where walls meet (the light map's shape, no texture read added), photographed details over the painted surfaces (woodchip, carpet, acoustic tile, plaster, concrete), a drop ceiling at a real tile's size with its fittings in the grid; partitions that lean now and then (R4, `tests/leaning.rs`: inside their tiles, their staircase inside the drawn wall and closed to a look, no prop inside one; blind reviews: ties) | Store department signs read as dark plates (an opaque material on a clear text canvas); no prop turntable sheets yet; the leaning walls went unnoticed by six blind reviewers (still frames read a few degrees as a wall a little nearer; whether they make the rooms less like a grid is for people); found by those reviewers on both sides, so older: hard-edged dark bands in a Lobby alcove's corner and over its floor (seed 7 level 4, region seams), a near-black strip where a Poolrooms back wall meets a higher ceiling (seed 7 level 5), a flat untextured soffit end and a banded beam face in the Poolrooms; the Lobby of seed 4242 level 4 is terracotta, not `STYLE.md`'s mono-yellow (a level palette's draw); level 37's views under a "Poolrooms" caption show concrete corridors, not white tile; the floors' stains are perfect ellipses (a reviewer of the marks, 2026-10-31, saw a hard-edged flat grey-blue octagon on a Lobby corridor's carpet, seed 77 level 7); flat-coloured cardboard boxes; found by that reviewer on both sides, so older: on the Stair Halls a vertical dashed column of flat, bright, pale-mint squares (the door's lit tower windows, read as a guide line), and the lamp heads there flat cream blocks with no fixture detail; dust motes near the camera read as glowing orbs; no human verdict |
| Creatures | B | Goldens at 2/4/8 m and death scenes. Rebuilt from scratch (`ENTITIES.md`, E1–E10): the Hound, the Smiler, the Watcher, the one in the corner and the Reader on the new brain and bodies, their rules one test each (`tests/creatures_rules.rs`) and their counters held to arenas (`tests/creatures_arenas.rs`: the Hound with its counter 99% untouched, without 9%; the Smiler 99% and 15%; the Watcher 100% and 0%; the one in the corner 100% and 67%; the Reader 100% and 4%), and once each has learned, as well as before (the Smiler, the Watcher, the one in the corner and the Reader 99–100%, each seen doing what it learned in 83–100% of trials; the Hound heard first 59% against 60%); fairness shared by every new creature (two tells before a strike, none within 2 s of a notice, a first meeting that knocks you down, never through your nerve, and sends it away); the Watcher a cloth-draped sheet among furniture under the same sheets; the Reader never touches her sheets (`tests/story.rs`); who lives where by the places (`tests/ecology.rs`); what each kind learns, one test each (`tests/creatures_rules.rs`), saved with the run; the director's empty build-ups and its withdrawal obeyed (`tests/pacing.rs`: no peak loops); the old creature code gone; beatable (`tests/beatable.rs`: levels 4–13 of four runs, the rule-follower, the explorer and the rusher each 32 of 32 with no death (E10), no hard rule broken; the whole run live, three runs to level 14, two deaths on the way); what each kind learns now happens within a run (the explorer's Smiler, three runs of four, E10); a dark zone takes your nerve no lower than 40 (`player::DARK_FLOOR`); a touch kills only a mind already spent (`creature::SPENT`); blind reviews (E10, both orders, a fresh reviewer each, `tests/review/verdicts.jsonl`): against the HTML version's creatures the new Smiler is preferred at 2, 4 and 8 m and at its death, the new Hound at 2 and 4 m and at its death; the Smiler's and the Watcher's bodies redone after a first round lost, each new body preferred to the one it replaced by two reviewers (the Watcher at every distance and its death, the Smiler at 2 m, ties elsewhere); a level's frames the same whatever the thread built before (every texture paints from a stream of its own: `tests/story.rs`); a lore and a code review of E1–E9, their findings fixed (the Watcher's kill, the Hound's ears in the canon, the one in the corner at rest nothing to the director, deaf, and never coming in a relief, the torn notes that taught the rules now hint); the one in the corner's hum heard, a held note at 160–210 Hz, 4.4–5.2 dB over the room in its band on eight streams and heard to stop (`tests/listen.rs`; since E5 each note had been a pluck under the lamps' 120 Hz, inaudible, and its test had passed on the director's heartbeat) | The rule-follower reads a creature's mode for the sign it gives (an upper bound on a player); those who know no counter finish as often (the explorer met them 33 times and never died, E10, since a touch kills only a spent mind; at E8 36 contacts cost it one death: whether being caught costs enough is for people); whether a kind learns too soon (2–3 times) is for people; once a Hound is sure of you at 6 m, only 60% get away untouched (its counter is not to be heard); the lamp's battery lasts 25 s lit, and most of the bots' time in the dark on a spent nerve is with a dead battery (whether players run out is for people); the Reader's face at its death is pale under the death scene's light (its staring eyes and inked mouth read, its skin does not); the one in the corner's Thatcher face reads only as wrong, not yet as upside down; against the HTML version's, the Watcher still loses at 2 and 4 m and at its death (its tall robe with a face showing through, against a sheet read as a bedsheet ghost among the sheeted furniture) and the Hound at 8 m (pale against the wall); the one in the corner and the Reader have nothing to be judged against (people must); reviewers split on the Smiler's sheen (wet read as latex, dull as a body suit; it ships dull); in the death frames that look up, two reviewers took the hanging broken ceiling tile (near black, `world/build.rs`) for a hole; the lore review's sameness: three tells are a sound that stops (the Hound's scratching, the corner's hum, the Reader's whisper), the Reader and the Smiler are both beaten by stillness in the dark, the Smiler idles facing a corner like the one in the corner, "tall" is said of the Smiler, the one in the corner and the staff, the Watcher's learned failing lights are also the director's sign, and the Reader's inky, paper-stuffed body nearly answers who lays the flat sheets; whether they frighten is untested with people |
| Pacing | B | `pacing.rs`: first build under 150 s, calms ≤ 95 s, reliefs ≤ 50 s, ≥ 4 peaks; deaths per peak about 0.11 for a cautious bot. Personas (`personas.rs`): bots that ignore the creatures die in 6 of 9 runs; 20% of the stalker's signs lead to a chase within 20 s; peaks ≤ 60 s, reliefs ≥ 8 s | A stalker that stays far off repeats the same sign (distant steps) up to 7 times in a row: the director picks a sign by distance alone; a player who goes straight to her camp and the door (under two and a half minutes a level) mostly leaves before the first peak (the whole run live: a peak on 9 of 29 levels); no human data |
| Sound | B | `listen.rs` (opening, signatures, the notice, the hush); `loudness.rs` (integrated ±1 LU, range ±3 LU, true peak ≤ −1 dBTP, repetition); ambient place tests | Nobody has listened since the place-based sound pass; the agent cannot hear |
| Canon | C | `LORE.md` (ported, with delivery channels; M.'s descent); `PILLARS.md`; one `lore-keeper` review (M.'s notebook, 2026-10-26: a contradiction with M.'s Rule 1, promised marks the world lacked, two threads that were orders, lines lifted from older notes; all rewritten); one of the UI's words (2026-10-27: death and pause lines that gave tips or stated rules, "NEXT" and "story arcs" in the notebook, the debug key on the Controls page, three names for the light; all fixed); two of the Building's text (Megastructure B6, its findings all taken; B8: the cleaners and the self-assembly named in more places than the canon allows, warm machines blurring the warm door, a figure on flights "nobody has stood on", storeys poured where galleries are drawn, the Well's windows and drop said three times; all fixed) | The older notes have never been reviewed; no human has read the notebook; the mannequin (a prop and torn note 17) is not in the canon |
| Story and wayfinding | B | `tests/story.rs`: a level is the same place whatever was read (G1); her trail holds on 13 levels and all four anomaly floors, each of her marks in sight of the last at its own kind's distance, in her hand, fewer sheets deeper down (`her_marks_change_as_she_goes_down`: 87/53/30/14% sheets over 39 floors); the place's answer to a cry along the way to the door, without a word (`crates/game/tests/player.rs`), never footprints on the Building's floors (`tests/secrets.rs`); her notebook follows the descent on eight runs; taking her page (journal, thread, retry, Continue); the page wakes the place (log: page, wake, build); trails that are not hers; the last camp and the ending. `the_door_is_heard` (the door alone, +1 to +18 dB over the bed from 2 to 9 tiles); the playthrough plays through the ending; plan maps in `target/story/`; a blind review of her marks along the walk (36 views, both orders, against the trail of sheets alone): hers preferred 8, the sheets 2, 26 ties (small by design) | No compass: whether a fresh player finds the door, follows her trail, notices the flat sheets and reads the notebook as a story is untested with people (`PLAYTEST.md` asks); whether her small marks are found by those who look or missed by everyone (deep down they are faint by design), and whether the lights' answer reads as an answer, is for people; the place's answers and her marks were judged by one blind reviewer on still frames |
| UI and controls | B | `tests/controls.rs` (reach, a held key fires once, running and using things together, no stutter, crouch, pause, what you face, the early E, Command, reading while walking), `tests/settings.rs`, `tests/menus.rs` (title, pause, settings, controls, leaving, death), `tests/rest.rs`, `tests/faces.rs`; the HUD test's frames; `--screen` renders every list over a level; a code review and a lore review of U1–U7 | No human has played with these controls or read these screens (`PLAYTEST.md` asks); no gamepad; brightness is a lift or dim over the frame, not a gamma |
| Code health | B | fmt and clippy `-D warnings`; structural tests (layering, determinism (now over `story/` too), sizes, docs, ledger) | `ui/draw.rs` is 1,692 lines (cap 2,400); the placer registry is repeated in nine older test files and `main.rs` (newer tests share `tests/common`); `ambient.rs`'s `weather_sounds_come_from_their_places` failed once in a full suite run (unseeded, counts only each step's last log entry; 30 of 30 alone) |
| Determinism | B | Seeded level builds (fixtures, reference levels, the level sweep's rebuild check); bit-identical `--fresh` frames; replays reproduce 2 × 2,400 frames of a bot run (a clock read injected into the director is caught at its first frame) | The director's first timer depends on what was drawn from the per-frame stream while building (fine per process; replays run in fresh threads) |
| Performance | B | `cargo xtask bench`: budgets in `PERF.md` (build ≤ 600 ms, first frame ≤ 100 ms, update p99 ≤ 4 ms, GPU p99 ≤ 6 ms at 1280×800, draw calls ≤ 2,000, triangles ≤ 2.5 M); history with change-point detection; GPU p99 1.4–2.9 ms today; the creature pack (two of every roaming kind hunting you) timed beside the levels, with its own budgets and history (`perf/creatures.csv`) | One run on record (change points need eight); the Showroom is 7× the triangles of a maze level |
| Assets | B | Blender sources in `assets/src/`; `cargo xtask assets props` rebuilds 2,423 of 2,546 prop parts exactly (repeatable since noise3 and seeding); both files proven to be byte-for-byte conversions of the HTML version's modules; the Building's nine photographed textures rebuilt from their CC0 photographs by `cargo xtask assets textures` (each source checked by its SHA-1); `MANIFEST.json` (sources, licence, generator, fingerprint) checked for meshes and textures; mesh lint over every part and the parts table | 123 frozen prop parts keep shapes from an unrepeatable build; the two extinguisher lever parts are 68% slivers (`DEGENERATE_EXCEPTIONS`); the old creatures (`creature_meshes.bin`) still rebuild only in the HTML version (rig dumps not ported); their replacements' bodies (`bodies.bin`) rebuild here byte for byte (`cargo xtask assets creatures`), so far only the test body |
| Reviewers | C | Calibrated on known answers (`cargo xtask calibrate`): strong damage 12/12, kappa 1.00; mild damage 9/12, kappa 0.45 — mild holes, smears and washed-out light caught, identical pairs judged ties | Mild over-saturation missed 0/3 (the gates' colourfulness anchors and `STYLE.md` bands are the authority there); no human verdicts to calibrate against yet |
| The Building (levels 1–3) | B | `tests/building.rs` (the navigation measures over 40 seeds a place: detour, flights, choices, dead ends, a cell within 16 tiles of every floor, the door seen from near the arrival); `tests/height.rs` (heights, voids, slabs on a hand-made floor); `tests/assembly.rs` (the self-assembly's stages, a sealed cell keeps it out, outside it kills, a cell within reach of the warning, a twelve-minute pacing log a floor); `tests/presences.rs` (the figure across the void out of reach, turning away, going at the siren; the door's light from the arrival on 11 of 12 floors; the lift; the Works' last door into the Lobby); `tests/story.rs` (the Building's notes only on its floors; her camp in a cell); six golden frames and the three places' `STYLE.md` bands; code, lore and level reviews (Megastructure B8); its surfaces photographs of Soviet-era places, CC0 and rebuilt by `cargo xtask assets textures`, preferred blind over the canvas surfaces on every floor (B9: three rounds in `tests/review/verdicts.jsonl`) and linted on all three floors (`tests/materials.rs`) | No person has played it: whether the way down is a puzzle or a chore, whether the siren reads as a warning the first time, whether waiting in a cell is dread or a chore, whether the figure is noticed and how often is too often (`PLAYTEST.md` asks); the siren and the re-recorded loudness have not been heard by a person; the figure's visibility is judged by rules (the solids and what the dressing builds over the walks), not by the renderer, and on one Well floor (seed 7) the lift's chosen view is still hidden by the galleries, so the rule misses something there; a multi-level `--dump-level` keeps level 1's zone caption on the later levels' views, and makes no exit view for some Works floors (the door reached along its wall); the Works' wall pipes come as one-tile stubs (a coin flip a tile); the level sweep found a battery no one can reach on seed 2645350805, level 3 (the Works, at (-70.1, 26.7); it fails on `614b20d` too, before the photographs); in a Works zone view (4242, the arrival's flight) two flight tiles side by side have their treads out of step; the photographed surfaces are judged by a model reviewer only, and the nearest Well windows are soft up close (the panel photographs hold 128 px a metre) |
| Playtesting | D | The kit: `PLAYTEST.md` (script, debrief, tension trace, A/B by ranking), `playtests/TEMPLATE.md`; `--record` / `--replay` with ↑ / ↓ tension marks; `cargo xtask heatmap` | No human has played this build since the tension pass |

## Content families

| Family | Grade | Notes |
|---|---|---|
| Creatures (Smiler, Hound, Watcher, the one in the corner, Reader) | B | All five have goldens at 2, 4 and 8 m and a death scene, their rules one test each, their counters held to arenas, learned and not; the Smiler and the Hound (but at 8 m) preferred blind to the HTML version's, the Watcher to its first body but not yet to the HTML version's close up (E10) |
| The one behind the glass (`WINDOWS.md` 4) | C | Its rules one test a line on real floors and its counter held there (`tests/glass.rs`: with it 100% untouched, without 48%); five goldens at its window (32 m, 8 m, at the glass, inside, its kill); its glass sounds each heard over the room (`tests/listen.rs`); its body preferred blind to its first draft by two reviewers (10 of 12 views each); a blind let down at the glass keeps it out on 13 of 13 floors, and learned (its lamp 1 m lower, its light 4.3 m in for 2.6 m) its counter holds as before | Whether a player ever stares long enough, whether letting a blind down feels like their idea, and whether it frightens, is untested with people; its face up close is pale and wrong but smooth; from inside, its lamp coming down is seen only by its light on the floor; the lore review's sameness: a face pressed into something that gives (the Watcher's sheet), a hand at a glass (the dead vending machine's print), and rules about your eyes now many (the Watcher, the one in the corner, the figure, the Face, the vending display, this one); its tells were moved off breath and the Smiler's click (its tick a knock and the pane's ring, a squeak, its coat's rustle; the place's own drips were too near); its name has the one in the corner's shape, both placed and never sent and both beaten through your eyes, and its touch is the shared one the Watcher's is; the streetlamp is a third ballast at 120 Hz |
| Anomalies (Stairwell, Showroom, House, Loop) | C | Fixture tests and the sweep; the Showroom has a golden view; the staff, the shadows and the seated man have goldens of their own (E9), the staff and the seated man on new bodies a blind review preferred (`tests/anomaly_bodies.rs` holds the staff to its rig and walks it untorn); the shadows keep the HTML version's body (a new one lost or tied four reviews) |
| Zone purposes (14) | B | One golden view each; style bands measured from 631 views (five purposes with too few views share the pooled band) |
| Prop families (10 themes, 649 placers) | C | Material lint, smoke tests, the look settle pass; a census of every kind (`--prop-census`, `research/props-census.md`, 2026-10-31): 652 kinds on 272 floors graded by seven reviewers; P1 (`PROPS.md`): the 35 everyday cartoon kinds remodelled as the real things, P2's 66 common ones and P3's rare, the 112 and the 43 the census never saw (22 batches, each 14–0 to 16–0 against the old; blind reviews, P1 and P2: 24–0, 30–0, 17–0 with 2 ties, 13–0, 15–0, 9–0, 11–0, 15–0, 16–0, 15–0, 15–0, 14–0, 12–0, 16–0, 13–0; 12–0 for the barricades); the reference levels hold the settled walls, unchanged by P2 on 110 floors; the barricades seal real openings (`tests/barricades.rs`); six props have verbs (`tests/usable.rs`) | Almost nothing is worn (P4); no small prop has a contact shadow; the small faults of `PROPS.md` *Open after P3*; the wall placers take a pool's curb for a wall; no person has used a verb or walked into a barricade |
| The vending machine (`VENDING.md`) | B | Rebuilt as a real glass-front snack machine (V1: the Blender parts of batch `v01`, its goods and their printed wrappers, its light, its weight), preferred blind to the old COLA-Z in every frame that showed both (two reviewers, both orders); its nine rules built (V2, V3, V5), each held by a test in `tests/vending.rs` that fails with its rule broken, and where it stands (offices, halls, the Poolrooms, never a zone left bare; none on the Lobby, the Showroom or the Borrowed House); its sound (V4: its hum, its coin, its motor, its bang) held by a listen test that fails with the hum at gain 0; its canon through three `lore-keeper` rounds (V5), what each found fixed | No person has played with it or listened to it (`PLAYTEST.md` has its questions); goods spilled by its fall take the floors' pickup lines ("SOMEONE LEFT IT FOR YOU"); its lit interiors are bright and colourful next to the rooms' muted palette (a reviewer's caveat, held by the look gate); a fallen one's back is seen only from above at eye height (its louvres faint in a dark room) |
| The windows (`WINDOWS.md`) | C | Rebuilt as real windows cut into real walls (W1: wood double-hung and office strip windows from the Blender parts of batch `w01`, their glass, blinds and wear; the view of a field in fog from a CC0 panorama traced per pixel, its streetlamp drawn beyond the glass and its light thrown on the floor; the lamp heard through the glass), preferred blind to the old framed picture in all 22 views that showed both (one reviewer, both orders); held by `tests/windows.rs` (where they stand, the cut, the same on every build, the light in its own room, the same view from every window, later deeper) | No person has looked out of one; the view is one photograph for every window (by design: the same place) and soft close up (23 px a degree); what is beyond the glass is drawn as a thin slab in front of the view, and a part of it nearer than 2 cm could cross a bar's back edge at a graze; the one behind the glass has a row of its own below |
| Sound (mix, ambient) | B | Loudness and listening gates |
| The Building's places (the Well, the Stair Halls, the Works) | B | Two golden frames each, `STYLE.md` bands, the navigation bands; their own notes, surveys and arrival lines; their surfaces photographed (the Well's panels and lit windows, the Stair Halls' granite, the Works' formwork and factory slab), preferred blind on 32 of 32 views in the last rounds; no human verdict |
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
  judged blind against the HTML version's: the Smiler and the Hound
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
