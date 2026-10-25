---
paths:
  - "crates/game/src/entities/**"
  - "crates/game/src/assets/creatures.rs"
  - "crates/game/src/session/**"
---

# Creature and pacing rules

- **The director owns pressure** (`entities/director.rs`): calm → build →
  peak → relax. One stalker at a time, rotated (`last_sent`); it withdraws
  after a close call (menace ≥ 1 and lost for 2.5 s, or menace ≥ 1.2 with the
  hunter more than 2.5 m away). The Watcher acts only as the stalker.
- **Senses:** a view cone (dot ≥ 0.42, or within 3.5 m, or the flashlight on
  it); states wander / search / hunt; per-frame decisions draw from the
  native stream.
- **Re-record** after an intended change to builds, animation or behaviour:
  `REGEN=1 cargo test --release -p game --test entities`, then review the
  diff. The behaviour tests in `tests/entities.rs` must keep passing.
- **Evidence:** `--showcase kind:dist` at 2, 4 and 8 m and `--death`;
  `cargo test --release -p backrooms --test pacing -- --nocapture` for the
  phase sequence, close calls and deaths.
