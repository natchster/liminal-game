---
name: add-creature
description: Add or change a creature in The Backrooms: brief against canon and pillars, its body (Blender, MakeHuman CC0) and its brain, fairness and what it learns, its place in the ecology, rules and arena tests, goldens, pacing, and the ledger.
---

# Add or change a creature

The plan and the rules every creature keeps are in `docs/design/ENTITIES.md`
(design 1–9) and `.claude/rules/entities.md`; read both first.

1. **Brief.** Write the creature into `docs/design/LORE.md` (*Things that live
   here*) and its design into `docs/design/ENTITIES.md` (design 1): one sense it
   finds you by, one counter it is beaten by, its role, the place it lives,
   its sound, what it learns. Ask `lore-keeper` to check the brief. Quote the
   pillar rules it serves (`docs/design/PILLARS.md` 4 and 5).
2. **Body.** `assets/src/creatures/body_<kind>.py` (a person from
   MakeHuman's CC0 base: `hm08.py`, `human.py`, `anatomy.py`; cloth:
   `cloth.py`), posed so the pose is the rest pose; paint by the bone each
   point belonged to (`posed.owner`), not the nearest; extra bones the skin
   must not follow are added after it is skinned. Budgets: 18k triangles a
   body, 25k a creature. `cargo xtask assets creatures --write` builds
   `bodies.bin` (every body: the others must come out `same`); look at the
   previews in `target/xtask/assets/creatures/`. Eyes catch your lamp as
   eyeshine, they do not glow (`docs/design/STYLE.md`).
3. **Brain.** `crates/game/src/entities/<kind>.rs`: its `DATA`
   (`KindData`), `think` (one `Mode` at a time, each change logged with its
   reason), `animate`, `voice`, `call`; register it in
   `creature::on_new_brain` and the dispatch beside it; in `ROAMING_KINDS`
   if it roams (the bench's pack); among the director's candidates in
   `enter_build` if it is sent; in the ecology (`Game::ecology`: the places
   that call for it) or with a placer of its own after the build
   (`corner::place`, `reader::place`). Nothing of it may draw from the
   per-frame stream as a level starts or while it is far from you: make it
   inside `with_seed`, set its first timers from its place, give its own
   choices a stream of their own (`reader::own_stream`).
4. **Fairness** lives in `creature.rs`: strike only if `strike_allowed`,
   give tells with `creature_tell` (and keep giving them while it hunts),
   touch through `creature_contact`, chase through `creature_go` and burst
   through `creature_burst` (capped). Obey the director: sent away, it
   withdraws as soon as it no longer has you.
5. **What it learns** (design 4): an entry in `creature::LEARNS`, a
   `creature_habit` call where its habit happens, its one new behaviour
   behind `creature_may_show` (the first time, where you can see it).
6. **Tests.** `tests/creatures_rules.rs`: one test a line of its design,
   read back from its brain's log, and its learned behaviour;
   `tests/creatures_arenas.rs`: `report("<kind>")` with its counter played
   as a player would (with it ≥ 95% untouched, ≥ 90% round; without ≤ 70%
   untouched, ≤ 60% killed; when a counter fails, read the trials before
   tuning: `ARENA_DEBUG`), and learned (`learned_uses`: the mark its new
   behaviour leaves; the counter as good as before); `tests/ecology.rs`
   (where it lives); `tests/listen.rs` (its signature over the room);
   its counter in `tests/rule_follower/mod.rs`, then `tests/beatable.rs`
   (≥ 90% of levels, no hard fail) and `the_whole_run_live`.
7. **Silhouette and close-up.** Add it to `CREATURES` in
   `xtask/src/shots.rs`, run `cargo xtask shots`, and look at it at 8 m (it
   must read as *something standing there*), 4 m, 2 m and in its death
   scene (`--level 4 --showcase kind:dist --death`). Accept the goldens and
   look anchors with a reason.
8. **Pacing.** `cargo test --release -p backrooms --test pacing --test
   personas -- --nocapture`: the bands (first build < 150 s, calms ≤ 95 s,
   reliefs ≤ 50 s, peaks, no deaths at a first meeting) and the shape of
   the fear.
9. **Sound.** Its steps and calls come from where it is (its voice slot);
   check with the `add-sound` skill's gates.
10. **Gates.** `cargo xtask check` (full); `tests/structure.rs` requires its
    goldens, its rules and its arena. `cargo xtask bench` records the pack.
11. **Its death.** A creature's kill plays `death_taken` (`docs/design/CUTSCENES.md`):
    `on_death` poses it in your face (the Hound reared up: a kind of its own
    may want its own pose there) and its head is the cutscene's `killer`.
    Add `death:<kind>` to `assets/src/cutscenes/death_taken.toml`'s `stages`
    (`add-cutscene` step 7), `cargo xtask cutscene write`, and check its kill
    on it and its `<kind>_death` golden.
12. **Ledger and review.** A `creature/<kind>` entry in `docs/plan/content.json`;
    a blind review of its frames (`visual-review`); whether it frightens is
    for fresh players (`docs/plan/QUALITY.md`).
