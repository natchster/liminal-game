---
name: add-prop
description: Add or change a prop placer in The Backrooms. Covers theme and purpose, realistic materials, the Blender part, the look settle, the material lint, golden frames and the ledger.
---

# Add or change a prop

1. **Where it belongs.** Pick its theme (`PLACER_THEME` in
   `crates/game/src/world/tables.rs`) from the purposes that would have left
   it behind (`docs/STYLE.md`, Purposes). Quote the pillar rule it serves;
   the Lobby and `empty` zones stay sparse.
2. **Real reference.** Look up what the object is made of and its real
   albedo (physicallybased.info, or the values in `docs/STYLE.md`). Author
   colours as the real material, not as a toy: the look settle
   (`visuals/prop_look.rs`) mutes chroma above 0.3 and keeps albedo in
   0.02–0.8, but start realistic.
3. **Build it.** Follow `docs/DEVELOPMENT.md`, *Adding a placer*: a
   `place_things(g, dist)` in a placer crate, registered in its
   `register()`, added to `THEMED_ORDER` and `PLACER_THEME`. Finer parts come
   from the Blender meshes (`pg("Name__part", …)`, `assets/props.rs`);
   adding a new part means rebuilding `prop_meshes.bin` from its script.
4. **Determinism.** A new placer adds random draws: every later placer
   shifts. Re-record the reference levels and fixtures (`regen-fixtures`)
   and say so in the commit.
5. **Gates.**
   - `cargo test -p placers_pNN` (smoke: 12 seeds × 24 runs).
   - `cargo test --release -p backrooms --test materials -- --nocapture`
     (albedo, saturation, clear maps); never add an `EXEMPT` entry without
     a reason a person would accept.
   - `cargo xtask shots`; look at changed frames and their error maps
     before accepting.
   - `cargo xtask range` (prop density and themes per zone move by design;
     say so when accepting).
6. **Evidence.** A zone view that shows the prop where it belongs
   (`--fresh --dump-level … --views`), and a blind review (`visual-review`)
   when it changes the look of a place.
7. **A prop you look into** (a moment that takes the camera, as a broken
   lift's shaft): `add-cutscene`, its anchor read from the prop after
   settling.
8. **Ledger.** The theme's `props/<theme>` entry in `docs/content.json`
   (its placer count in the name); open issues in `docs/QUALITY.md`.
