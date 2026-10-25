---
name: add-zone
description: Add or change a zone archetype or a zone purpose in The Backrooms (level generation). Walks the Level Design Book order — style entry, layout, blockout checks, bot walk, lighting, set dressing last — and ends with the gates and the ledger.
---

# Add or change a zone

A zone is an **archetype** (the building: `ARCHETYPES` and the `carve_*`
functions in `crates/game/src/level/archgen.rs`) or a **purpose** (what the
place was for: `PURPOSE_LABEL`, `THEME_ALLOW`, `THEME_COMPAT`,
`PURPOSE_EMPTY` in `crates/game/src/world/tables.rs`, used by
`world/purpose.rs`). Work in this order; do not dress a space before its
layout passes.

1. **Canon and style first.** Find the place in `docs/LORE.md` (Places) or
   add it there; write its row in `docs/STYLE.md` (archetype table, or the
   purpose table: what is left, wayfinding, threat). Quote the pillar rule it
   serves (`docs/PILLARS.md`).
2. **Layout.** Implement the archetype's carving or the purpose's tables.
   Keep determinism: draws only from `rnd()` and its helpers, in a stable
   order (`.claude/rules/world.md`).
3. **Blockout checks** before any props or lights:
   - `cargo test --release -p backrooms --test level_sweep -- --nocapture`
     (walkable, reachable, repeatable); a new pocket kind is a bug;
   - `cargo xtask range` and read `target/xtask/range/report.md`: which
     measures moved (loops, dead ends, depth, sightlines, room size), and do
     the new levels fill new archive cells or crowd one?
4. **Walk it.** `cargo test --release -p backrooms --test playthrough --
   --nocapture` and look at zone views:
   `cargo run --release -- --fresh --dump-level DIR --seed S --levels L --screenshot x --views`.
5. **Lighting**, then **set dressing** (purposes: which themes may place
   there). Then `cargo xtask range --look`: the purpose's views against its
   band. For a new purpose, add its band to `STYLE.md` from the measured
   percentiles (see the note above the bands).
6. **Gates and records.**
   - `cargo xtask check` (full).
   - Re-record what the change meant to change, with the reason in the
     commit: the `regen-fixtures` skill.
   - Accept the range baseline (`cargo xtask range --accept --reason …`) and
     any new goldens only after reading the report and looking at the frames.
   - Add the purpose's golden view if it has none (it gets one when it
     first appears in `LEVELS` of `xtask/src/shots.rs`).
7. **Ledger.** Add or update the entry in `docs/content.json` (for a purpose:
   `purpose/<label>` with its shots, style, sweep and lint checks);
   `cargo test --release -p backrooms --test structure` keeps it honest.
   Note open issues in `docs/QUALITY.md`.
8. **Review.** Ask `level-analyst` to explain the range report, and run the
   `visual-review` skill on before/after zone views.
