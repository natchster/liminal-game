---
paths:
  - "crates/game/src/entities/**"
  - "crates/game/src/assets/creatures.rs"
  - "crates/game/src/assets/bodies.rs"
  - "crates/game/src/session/**"
  - "assets/src/creatures/**"
---

# Creature and pacing rules

- **The plan is `docs/ENTITIES.md`**: five creatures, five rules (one sense
  and one counter each), one brain, fairness as invariants. Every kind
  `add_entity` makes is on the brain (`creature::on_new_brain`); `ai.rs`
  keeps only the shared steering and the update loop (the anomalies' staff
  and shadows run their own rules there until E9).
- **The new brain** (`entities/brain.rs`, `entities/creature.rs`): a creature
  is in one `Mode` and changes it only by a rule of its kind, logged with a
  reason (`Brain::set`); percepts are computed once a frame (`perceive`:
  hearing is along the way it would walk, `path_distance`, never through
  walls); the director's wishes arrive as `Order`s. Kinds are data
  (`KindData`) plus their own `think` / `animate` / `voice` (`hound.rs`,
  `smiler.rs`, `watcher.rs`, `corner.rs`, `reader.rs`). A placed kind (the
  one in the corner; the Reader, beside paper) is placed after the build
  with no draws (`corner::place`, `reader::place`, from `session::flow`),
  and nothing of it may draw from the per-frame stream as the level starts
  or while it is far from you: make it inside `with_seed` (`add_entity`
  draws a heading), set its first timers from its place, and give what it
  chooses on its own a stream of its own (`reader::own_stream`); a draw
  there shifts every frame after it (invariant 1). The one in the corner
  is never a stalker (the director sends only the Smiler, the Hound, the
  Watcher and the Reader) and goes back to where it was put. Only sight (or touch) gives a creature your exact place; a
  sound, or your light going out, leaves it a rough guess (`open_point`), so
  a counter that hides you can work. A creature hunting by sight must still
  find you when sent: the director parks a stalker out of your sight, so
  give it a way to close in (the Smiler creeps on, and scans where it waits).
  Mind the director's reading of modes (`Mode::legacy_state`): only Hunt
  and Strike start a peak, and a peak is short and ends the errand; a kind
  that acts only when sent (the Watcher) must count as sent while it is
  after you in the peak it began, and should enter Hunt only when its
  reach is near (the long approach is the build-up).
- **Fairness lives in `creature.rs`, so no kind can skip it**: strike only
  if `strike_allowed` (two tells you could perceive in 20 s, not in a
  level's first 30 s, not within 2 s of any notice: stamp `noticed_at` at
  each one); every contact goes
  through `creature_contact` (the first of a kind in a run knocks you down,
  never through your nerve: a contact that is not the end leaves
  `KNOCKED_LEAVES`; a second in a peak, or one on a spent nerve, kills; a kind with
  `touch_nerve` takes that much nerve each touch and kills only a mind it
  leaves spent); chases through
  `creature_go` (capped at `CHASE_CAP`), bursts through `creature_burst`
  (`BURST_CAP` over `BURST_LEN`). Give a tell with `creature_tell`; a kind that hunts must keep giving
  tells you could perceive while it comes (the Smiler's breath, the
  Reader's steps), or a look-up you missed leaves it never allowed to
  strike, trailing you forever (the Reader's arena, E6).
- **Per-frame draws come from the native stream** (`with_native`); a
  creature's build draws nothing from the seeded stream (`make_creature`),
  so spawning one does not shift a level.
- **Morphs**: a body mesh with morph targets gets an influence each at
  build (`build_body`), or the renderer draws none; a pose is set by
  writing `morph_influences` (the Watcher's poses).
- **The director owns pressure** (`entities/director.rs`): calm → build →
  peak → relax. One stalker at a time, rotated (`last_sent`); sent from
  further than `FAR_SEND` along the level it is first brought, out of your
  sight, to `SEND_FROM` (`bring_nearer`: sent from 190 m it never arrived);
  some build-ups end in nothing (`EMPTY_SHARE`: one sign, then quiet; never
  one her page or a trail called). It withdraws them after a close call
  (menace ≥ 1 and lost for 2.5 s, or menace ≥ 1.2 with the hunter more than
  2.5 m away), and when a strike lands (`director_relax_now`). Sent away is
  an order every kind obeys as soon as it no longer has you (the shared
  rule in `update_creature`: one that searched on began the next peak at
  once); a chase is its own until its rule loses you (cut short, running 8
  s beat every creature), and one sent away, finishing its chase, starts no
  new peak. The Watcher acts only as the stalker.
- **The ecology** (`Game::ecology`, design 5) decides who lives on a level,
  by its places, with no draws: one on the Lobby, two at depth 2, three
  deeper; the Smiler where there is dark, the Watcher among furnished rooms,
  the Reader beside paper; places a creature can use (reached from the
  arrival, a quarter of the level off). The Showroom and the Borrowed House
  keep their own residents; a level with the Stairwell's or the Loop's door
  is rooms like any.
- **What they learn** (design 4): `creature::LEARNS`; count a habit with
  `creature_habit` where it happens; at most one kind learns a level (the
  rest on a later one: `creature_learn_pending`); the new behaviour goes
  behind `creature_may_show` (the first time, in plain sight); the run's
  `RunMemory` is saved with it.
- **Bodies** are built by `cargo xtask assets creatures` (Blender headless,
  `assets/src/creatures/`) into `bodies.bin`, posed so the pose is the rest
  pose; animate by aiming bones in the body's frame (`BodyRig::aim`,
  `two_bone`). Never hold two borrows of one node in one expression
  (`root.rotation().y.sin()` and `.cos()` together panics): read it once.
- **Tests**: a kind's rules in `tests/creatures_rules.rs` (one test a line of
  its design, read back from its brain's log); its counter in
  `tests/creatures_arenas.rs` (with it at least 95% untouched and 90% round,
  without it at most 70% untouched and 60% killed). When a counter fails,
  read the trials (`debug_trials`, `ARENA_DEBUG`) before tuning: ask whether
  the rule leaves the player a winning move at all (fix the creature) or
  the bot just plays it badly (fix the bot, as a player would play it).
  What it learns is held there too (`what_each_learns_leaves_its_counter_working`:
  the counter works as well as before, and the arena must show it doing
  what it learned). Beatable (design 7): `tests/beatable.rs` (the
  rule-follower of `tests/rule_follower/` finishes ≥ 90% of levels 4–13,
  no death breaks a hard rule; `BEATABLE_TRACE=seed:level … one_level` to
  see a level) and `the_whole_run_live` in `tests/playthrough.rs`; a new
  kind gets its counter in `rule_follower::counter`.
  Where they live: `tests/ecology.rs`. What is left of the old reference
  fixture (the steering, the staff's limb solver): `REGEN=1 cargo test
  --release -p game --test entities` prunes it to what is read.
- **Evidence:** the creature goldens in `cargo xtask shots` (2, 4, 8 m and
  the death scene), or `--fresh --level 4 --showcase kind:dist [--death]` by hand (a floor of
  the rooms: the Building's floors have none);
  `cargo test --release -p backrooms --test pacing -- --nocapture` for the
  phase sequence, close calls and deaths; `cargo xtask bench` for the
  creature pack (`perf/creatures.csv`).
