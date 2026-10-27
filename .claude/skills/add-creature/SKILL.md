---
name: add-creature
description: Add or change a creature in The Backrooms: brief against canon and pillars, model and rig, silhouette and close-up goldens, senses and behaviour tests, the pacing director, and the ledger.
---

# Add or change a creature

1. **Brief.** Write the creature into `docs/LORE.md` (*Things that live
   here*) first: what it looks like in the dark and up close, its learnable
   rule, its sound. Ask `lore-keeper` to check the brief. Quote the pillar
   rules it serves (`docs/PILLARS.md` 4 and 5).
2. **Body.** Build it in `crates/game/src/entities/mod.rs` (`make_*`,
   textures, `EntAssets`) from the Blender meshes (`assets/creatures.rs`,
   `creature_meshes.bin`); animation in `entities/anim.rs`. Eyes catch light
   as eyeshine, they do not glow (`docs/STYLE.md`, Creatures).
3. **Silhouette and close-up.** Add it to `CREATURES` in
   `xtask/src/shots.rs`, run `cargo xtask shots`, and look at it at 8 m (it
   must read as *something standing there*), 4 m, 2 m and in its death
   scene (`--level 4 --showcase kind:dist --death`: a floor of the rooms). Accept the goldens with a reason.
4. **Senses and behaviour** (`entities/ai.rs`): view cone, hearing, states,
   what it does on notice. Add behaviour tests to
   `crates/game/tests/entities.rs` in the style of
   `creatures_see_ahead_not_behind`; re-record its build and animation
   sections with `REGEN=1` (the `regen-fixtures` skill).
5. **Pacing.** Decide how the director (`entities/director.rs`) sends it, how
   it announces itself and how it backs off. Run
   `cargo test --release -p backrooms --test pacing -- --nocapture` and keep
   the bands (first build < 150 s, calms ≤ 95 s, reliefs ≤ 50 s, peaks, few
   deaths per peak).
6. **Sound.** Its steps and calls come from where it is (`play_from`); check
   with the `add-sound` skill's gates.
7. **Gates.** `cargo xtask check` (full); `tests/structure.rs` requires its
   goldens and behaviour tests.
8. **Ledger and review.** A `creature/<kind>` entry in `docs/content.json`;
   a blind review of its frames against the old ones (`visual-review`);
   whether it frightens is for fresh players (`docs/QUALITY.md`).
