# Quality scorecard

A grade for each subsystem and content family, the evidence behind it, and
what is open. The drift sweep rewrites the *Evidence* and *Open* columns; a
grade moves only with an entry under *History* saying why. The ledger of
individual content is [`content.json`](content.json); the tooling that
produces the evidence is in [`TOOLING_ROADMAP.md`](TOOLING_ROADMAP.md).

Grades: **A** held by gates and confirmed by people · **B** held by gates,
not yet confirmed by people · **C** partly covered · **D** not covered.

Last updated: 2026-10-25 (by hand; the drift sweep is not built yet).

## Subsystems

| Subsystem | Grade | Evidence | Open |
|---|---|---|---|
| Level generation | B | Determinism fixtures; 20 recorded reference levels; the level sweep through the collision geometry (spawn, exit, open space, pickups, rebuild equality); zone-purpose properties | A barricade can lock a pickup away (about 1 level in 20; `PICKUPS_BEHIND_BARRICADES`) |
| Rendering and look | B | 31 golden frames (exact, then ꟻLIP); look anchors per frame and over the set; `STYLE.md` bands; the material lint (16,661 materials on six levels); blind A/B reviews preferred the current look (8/8 realism, 8/8 dread) | Store department signs read as dark plates (an opaque material on a clear text canvas); no prop turntable sheets yet; no human verdict |
| Creatures | B | Goldens at 2/4/8 m and death scenes; build and animation fixtures; senses and behaviour tests; blind reviews preferred the current creatures (3/3, 5/5) | The Hound at 2 m sits low in its golden; whether they frighten is untested with people |
| Pacing | B | `pacing.rs`: first build under 150 s, calms ≤ 95 s, reliefs ≤ 50 s, ≥ 4 peaks; deaths per peak about 0.11 for a cautious bot | Scare repetition and cue variety are not measured yet (roadmap 4.2); no human data |
| Sound | B | `listen.rs` (opening, signatures, the notice, the hush); `loudness.rs` (integrated ±1 LU, range ±3 LU, true peak ≤ −1 dBTP, repetition); ambient place tests | Nobody has listened since the place-based sound pass; the agent cannot hear |
| Canon | C | `LORE.md` (ported, with delivery channels); `PILLARS.md` | No `lore-keeper` review exists yet (roadmap Phase 2) |
| Code health | B | fmt and clippy `-D warnings`; structural tests (layering, determinism, sizes, docs, ledger) | `ui/draw.rs` is 2,264 lines (cap 2,400); the placer registry is repeated in eight test files and `main.rs` |
| Performance | D | Draw-call and triangle counts printed by `--screenshot` | No budgets, no history (roadmap 1.8) |
| Assets | D | Magic-number checks on the embedded meshes | Their Blender sources live in the HTML version's folder; no rebuild, lint or licence manifest (roadmap 3.1, 3.2, 1.4) |
| Playtesting | D | — | No human has played this build since the tension pass |

## Content families

| Family | Grade | Notes |
|---|---|---|
| Creatures (Smiler, Hound, Watcher) | B | All three have goldens and behaviour tests |
| Anomalies (Stairwell, Showroom, House, Loop) | C | Fixture tests and the sweep; only the Showroom has a golden view |
| Zone purposes (14) | B | One golden view and a style band each; bands set from one view, to be tightened by the range report |
| Prop families (10 themes, 650 placers) | B | Material lint, smoke tests, the look settle pass; no turntables |
| Sound (mix, ambient) | B | Loudness and listening gates |

## History

- 2026-10-25 — first scorecard, written with the tooling milestones M1–M6.
