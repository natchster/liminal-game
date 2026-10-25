---
paths:
  - "crates/game/src/audio/**"
  - "crates/engine/src/audio.rs"
  - "crates/engine/src/wav.rs"
---

# Audio rules

- **Every sound comes from a place**: a prop within earshot, a level
  signature at a fixed spot, or a weather-bed anchor (`BED_ANCHORS`,
  `bed_at`, `nearest_fixture`). Use `play_from(x, y, z, ref_dist, gain,
  occlude, f)`; never pan a one-shot relative to the player.
- **Carry and gain** of each signature live in `sig_carry`; a new ambient
  sound needs an entry there.
- **The hush** (`amb_hush`, driven by the director) softens and holds
  one-shots, room tone and hum; new ambient events must respect it.
- **Stable per-level choices** use `SplitMix64` with a salt, not the build
  stream (see `SIG_SALT`, `BED_SALT`). Rare cadences with `run: true` count
  down across levels.
- **You cannot hear.** Check with `cargo test --release -p backrooms --test listen -- --nocapture`
  (WAVs and dB-per-50 ms timelines in `target/listen/`), compare levels
  with the bed, and ask a person to listen to any change in timbre.
