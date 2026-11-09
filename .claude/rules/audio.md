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
- **Your own voice** (the cry for help, `say_cry`) is the one sound with
  no place but you: centred, into the soundscape's `world` bus, so it is
  heard in the reverb of the place you stand. Every other voice line comes
  from a place like any sound. Voice lines are made by the voice pipeline
  (`docs/VOICES.md`), chosen from a stream of their own.
- **Carry and gain** of each signature live in `sig_carry`; a new ambient
  sound needs an entry there.
- **The hush** (`amb_hush`, driven by the director) softens and holds
  one-shots, room tone and hum; new ambient events must respect it.
- **Stable per-level choices** use `SplitMix64` with a salt, not the build
  stream (see `SIG_SALT`, `BED_SALT`). Rare cadences with `run: true` count
  down across levels.
- **Beside a lamp** the ballast's hum (60 Hz, 120 Hz and its series to
  1.08 kHz) is the bed. A sound that must be heard wherever it happens needs
  a part above 1.2 kHz, and its listen test a place beside a lamp, measured
  in that band (`tests/listen.rs`, `a_knock_is_heard_beside_a_lamp`).
- **Gate:** `tests/loudness.rs` holds each scenario's integrated loudness
  (±1 LU), loudness range (±3 LU), true peak (≤ −1 dBTP) and repetition (no
  one-shot twice within 8 s). Re-record only after listening.
- **You cannot hear.** Check with `cargo test --release -p backrooms --test listen -- --nocapture`
  (WAVs and dB-per-50 ms timelines in `target/listen/`), compare levels
  with the bed, and ask a person to listen to any change in timbre.
