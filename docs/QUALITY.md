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
Last sweep: 2026-10-28.

| Gate | Result |
|---|---|
| Check steps | ok    fmt         0.9 s; ok    clippy      4.5 s; FAIL  test     1261.7 s  log: target/xtask/check/test.log; ok    shots      45.0 s; ok    range      39.0 s; ok    bench      74.9 s |
| Tests | 46 of 47 test files pass; placer smoke tests pass |
| Golden frames | 33 of 33 match |
| Expressive range | energy distance 0.0951, p = 0.544; seed archive: 19/36 cells, most crowded 14% |
| Performance | within budget, no step in the history |
| Largest files | `crates/game/src/level/archgen.rs` 2368 (99% of the cap), `crates/game/src/visuals/textures.rs` 1944 (81% of the cap), `crates/engine/src/audio.rs` 1926 (80% of the cap), `crates/engine/src/render.rs` 1873 (78% of the cap), `crates/game/src/modes/loop_rooms.rs` 1844 (77% of the cap) |
| Content failing a check | anomaly/house: test:backrooms/level_sweep, anomaly/loop: test:backrooms/level_sweep, anomaly/stairwell: test:backrooms/level_sweep, anomaly/store: test:backrooms/level_sweep, building/floors: test:backrooms/level_sweep, props/bathroom: test:backrooms/level_sweep, props/core: test:backrooms/level_sweep, props/domestic: test:backrooms/level_sweep, props/industrial: test:backrooms/level_sweep, props/nature: test:backrooms/level_sweep, props/occult: test:backrooms/level_sweep, props/office: test:backrooms/level_sweep, props/party: test:backrooms/level_sweep, props/toys: test:backrooms/level_sweep, props/victorian: test:backrooms/level_sweep, purpose/empty: test:backrooms/level_sweep, purpose/living_quarters: test:backrooms/level_sweep, purpose/maintenance: test:backrooms/level_sweep, purpose/office_wing: test:backrooms/level_sweep, purpose/overgrown_atrium: test:backrooms/level_sweep, purpose/parlour: test:backrooms/level_sweep, purpose/party_room: test:backrooms/level_sweep, purpose/play_area: test:backrooms/level_sweep, purpose/pool_hall: test:backrooms/level_sweep, purpose/service_tunnels: test:backrooms/level_sweep, purpose/showroom: test:backrooms/level_sweep, purpose/shrine: test:backrooms/level_sweep, purpose/stair_halls: test:backrooms/level_sweep, purpose/storage_hall: test:backrooms/level_sweep, purpose/washrooms: test:backrooms/level_sweep, purpose/well: test:backrooms/level_sweep, purpose/works: test:backrooms/level_sweep, story/camp: test:backrooms/level_sweep, story/trail: test:backrooms/level_sweep |
| Regressions people caught that no gate did | 0 (see *Caught by people*) |
<!-- drift:end -->

## Subsystems

| Subsystem | Grade | Evidence | Open |
|---|---|---|---|
| Level generation | B | Determinism fixtures; 20 recorded reference levels; the level sweep through the collision geometry (spawn, exit, open space, pickups, rebuild equality); zone-purpose properties; the expressive-range baseline (72 levels; the seed archive fills 20 of 36 cells, no cell over 12%) | A barricade can lock a pickup away (about 1 level in 20; `PICKUPS_BEHIND_BARRICADES`) |
| Rendering and look | B | 31 golden frames (exact, then ꟻLIP); look anchors per frame and over the set; `STYLE.md` bands; the material lint (16,661 materials on six levels); blind A/B reviews preferred the current look (8/8 realism, 8/8 dread) | Store department signs read as dark plates (an opaque material on a clear text canvas); no prop turntable sheets yet; no human verdict |
| Creatures | B | Goldens at 2/4/8 m and death scenes; build and animation fixtures; senses and behaviour tests; blind reviews preferred the current creatures (3/3, 5/5) | The Hound at 2 m sits low in its golden; whether they frighten is untested with people |
| Pacing | B | `pacing.rs`: first build under 150 s, calms ≤ 95 s, reliefs ≤ 50 s, ≥ 4 peaks; deaths per peak about 0.11 for a cautious bot. Personas (`personas.rs`): bots that ignore the creatures die in 6 of 9 runs; 20% of the stalker's signs lead to a chase within 20 s; peaks ≤ 60 s, reliefs ≥ 8 s | A stalker that stays far off repeats the same sign (distant steps) up to 7 times in a row: the director picks a sign by distance alone; no human data |
| Sound | B | `listen.rs` (opening, signatures, the notice, the hush); `loudness.rs` (integrated ±1 LU, range ±3 LU, true peak ≤ −1 dBTP, repetition); ambient place tests | Nobody has listened since the place-based sound pass; the agent cannot hear |
| Canon | C | `LORE.md` (ported, with delivery channels; M.'s descent); `PILLARS.md`; one `lore-keeper` review (M.'s notebook, 2026-10-26: a contradiction with M.'s Rule 1, promised marks the world lacked, two threads that were orders, lines lifted from older notes; all rewritten); one of the UI's words (2026-10-27: death and pause lines that gave tips or stated rules, "NEXT" and "story arcs" in the notebook, the debug key on the Controls page, three names for the light; all fixed); two of the Building's text (Megastructure B6, its findings all taken; B8: the cleaners and the self-assembly named in more places than the canon allows, warm machines blurring the warm door, a figure on flights "nobody has stood on", storeys poured where galleries are drawn, the Well's windows and drop said three times; all fixed) | The older notes have never been reviewed; no human has read the notebook; the mannequin (a prop and torn note 17) is not in the canon |
| Story and wayfinding | B | `tests/story.rs`: a level is the same place whatever was read (G1); her trail holds on 13 levels and all four anomaly floors; her notebook follows the descent on eight runs; taking her page (journal, thread, retry, Continue); the page wakes the place (log: page, wake, build); trails that are not hers; the last camp and the ending. `the_door_is_heard` (the door alone, +1 to +18 dB over the bed from 2 to 9 tiles); the playthrough plays through the ending; plan maps in `target/story/` | No compass: whether a fresh player finds the door, follows her trail, notices the flat sheets and reads the notebook as a story is untested with people (`PLAYTEST.md` asks); the payphone still says the door is "north" (its line is pinned by the core-props fixture, which has no recorder) |
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
| Creatures (Smiler, Hound, Watcher) | B | All three have goldens and behaviour tests |
| Anomalies (Stairwell, Showroom, House, Loop) | C | Fixture tests and the sweep; only the Showroom has a golden view |
| Zone purposes (14) | B | One golden view each; style bands measured from 631 views (five purposes with too few views share the pooled band) |
| Prop families (10 themes, 650 placers) | B | Material lint, smoke tests, the look settle pass; no turntables |
| Sound (mix, ambient) | B | Loudness and listening gates |
| The Building's places (the Well, the Stair Halls, the Works) | B | Two golden frames each, `STYLE.md` bands, the navigation bands; their own notes, surveys and arrival lines; their surfaces photographed (the Well's panels and lit windows, the Stair Halls' granite, the Works' formwork and factory slab), preferred blind on 32 of 32 views in the last rounds; no human verdict |
| Story (trail, camps, notebook, false trails, ending) | B | `tests/story.rs`; ledger entries `story/*`; no human verdict |
| UI (controls, menus, HUD, cards, notebook, type) | B | Ledger entries `ui/*`; no human verdict |

## Caught by people, missed by gates

The steering loop (roadmap 5.3): every regression a person found that no
gate caught, and the check added so it cannot slip through again. The drift
sweep counts the rows; the count should fall.

| Date | What a person saw | Which gate should have caught it | Check added |
|---|---|---|---|

## History

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
