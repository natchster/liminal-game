---
name: add-anomaly
description: Add or change an anomaly level in The Backrooms (the Stairwell, Showroom, Borrowed House, Loop or a new one): canon and attribution, the run rules, its own notes, tests, goldens and the ledger.
---

# Add or change an anomaly

1. **Canon.** Write it into `docs/LORE.md` (*Anomalies*, its own section,
   *Attribution and licensing* when it is inspired by an SCP article: CC
   BY-SA 3.0, author and link). Ask `lore-keeper` to check it. It reaches
   the player only through the allowed channels: one arrival line giving the
   kind of place, its own torn notes, the place itself.
2. **The run rules** (`crates/game/src/level/scp.rs`): at most one anomaly
   per level, none on levels 1–4 (the Building's floors and the Lobby), each once per run, never on two levels in a
   row, planned from the run seed. Add its key to `SCP_KEYS` and its entry.
3. **Build it**: a mode (`crates/game/src/modes/`) or an anomaly in
   `crates/game/src/anomalies/`, deterministic like any level build.
4. **Tests.** A fixture or behaviour test in `crates/game/tests/` (see
   `stairwell.rs`, `store.rs`, `loop_rooms.rs`, `anomalies.rs`); the level
   sweep must stay green on its levels (add a fixed case to `ALWAYS` in
   `tests/level_sweep.rs` if it is special).
5. **Goldens.** If one of the `LEVELS` in `xtask/src/shots.rs` does not show
   it, add a `(seed, level)` that does (find one with `--dump-level`: the
   JSON names the `scp`). Look, then accept with a reason.
6. **Gates.** `cargo xtask check` (full), `cargo xtask range`.
7. **Ledger.** An `anomaly/<key>` entry in `docs/content.json` (the
   structure test requires one per `SCP_KEYS` entry); open issues in
   `docs/QUALITY.md`.
