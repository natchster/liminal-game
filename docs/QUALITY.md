# Quality scorecard

A grade for each subsystem and content family, the evidence behind it, and
what is open. The drift sweep rewrites the *Evidence* and *Open* columns; a
grade moves only with an entry under *History* saying why. The ledger of
individual content is [`content.json`](content.json); the tooling that
produces the evidence is in [`TOOLING_ROADMAP.md`](TOOLING_ROADMAP.md).

Grades: **A** held by gates and confirmed by people · **B** held by gates,
not yet confirmed by people · **C** partly covered · **D** not covered.

Last updated: 2026-10-25. The *Latest sweep* section is written by
`cargo xtask drift`; the grades and the rest are written by people.

## Latest sweep

<!-- drift:begin -->
Last sweep: 2026-10-25.

| Gate | Result |
|---|---|
| Check steps | full check: fmt, clippy, test (215 passed, 8 ignored), shots, range, bench — all ok |
| Tests | 37 of 37 test files pass; placer smoke tests pass |
| Golden frames | 31 of 31 match |
| Expressive range | energy distance 0.0000, p = 1.000; seed archive: 20/36 cells, most crowded 12% |
| Performance | within budget, no step in the history |
| Largest files | `crates/game/src/ui/draw.rs` 2264 (94% of the cap), `crates/game/src/level/archgen.rs` 2128 (89% of the cap), `crates/game/src/visuals/textures.rs` 1944 (81% of the cap), `crates/engine/src/audio.rs` 1926 (80% of the cap), `crates/engine/src/render.rs` 1873 (78% of the cap) |
| Content failing a check | none |
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
| Canon | C | `LORE.md` (ported, with delivery channels); `PILLARS.md` | No `lore-keeper` review exists yet (roadmap Phase 2) |
| Code health | B | fmt and clippy `-D warnings`; structural tests (layering, determinism, sizes, docs, ledger) | `ui/draw.rs` is 2,264 lines (cap 2,400); the placer registry is repeated in eight test files and `main.rs` |
| Determinism | B | Seeded level builds (fixtures, reference levels, the level sweep's rebuild check); bit-identical `--fresh` frames; replays reproduce 2 × 2,400 frames of a bot run (a clock read injected into the director is caught at its first frame) | The director's first timer depends on what was drawn from the per-frame stream while building (fine per process; replays run in fresh threads) |
| Performance | B | `cargo xtask bench`: budgets in `PERF.md` (build ≤ 600 ms, first frame ≤ 100 ms, update p99 ≤ 4 ms, GPU p99 ≤ 6 ms at 1280×800, draw calls ≤ 2,000, triangles ≤ 2.5 M); history with change-point detection; GPU p99 1.4–2.9 ms today | One run on record (change points need eight); the Showroom is 7× the triangles of a maze level |
| Assets | B | Blender sources in `assets/src/`; `cargo xtask assets props` rebuilds 2,423 of 2,546 prop parts exactly (repeatable since noise3 and seeding); both files proven to be byte-for-byte conversions of the HTML version's modules; `MANIFEST.json` (sources, licence, generator, fingerprint) checked; mesh lint over every part and the parts table | 123 frozen prop parts keep shapes from an unrepeatable build; the two extinguisher lever parts are 68% slivers (`DEGENERATE_EXCEPTIONS`); creatures still rebuild only in the HTML version (rig dumps not ported) |
| Reviewers | C | Calibrated on known answers (`cargo xtask calibrate`): strong damage 12/12, kappa 1.00; mild damage 9/12, kappa 0.45 — mild holes, smears and washed-out light caught, identical pairs judged ties | Mild over-saturation missed 0/3 (the gates' colourfulness anchors and `STYLE.md` bands are the authority there); no human verdicts to calibrate against yet |
| Playtesting | D | The kit: `PLAYTEST.md` (script, debrief, tension trace, A/B by ranking), `playtests/TEMPLATE.md`; `--record` / `--replay` with ↑ / ↓ tension marks; `cargo xtask heatmap` | No human has played this build since the tension pass |

## Content families

| Family | Grade | Notes |
|---|---|---|
| Creatures (Smiler, Hound, Watcher) | B | All three have goldens and behaviour tests |
| Anomalies (Stairwell, Showroom, House, Loop) | C | Fixture tests and the sweep; only the Showroom has a golden view |
| Zone purposes (14) | B | One golden view each; style bands measured from 631 views (five purposes with too few views share the pooled band) |
| Prop families (10 themes, 650 placers) | B | Material lint, smoke tests, the look settle pass; no turntables |
| Sound (mix, ambient) | B | Loudness and listening gates |

## Caught by people, missed by gates

The steering loop (roadmap 5.3): every regression a person found that no
gate caught, and the check added so it cannot slip through again. The drift
sweep counts the rows; the count should fall.

| Date | What a person saw | Which gate should have caught it | Check added |
|---|---|---|---|

## History

- 2026-10-25 — first scorecard, written with the tooling milestones M1–M6.
