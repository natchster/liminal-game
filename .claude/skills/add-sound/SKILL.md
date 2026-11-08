---
name: add-sound
description: Add or change a sound in The Backrooms (an ambient event, a signature, a bed, a creature or prop sound): synthesis, place, carry, the loudness and listening gates, and a person's ears.
---

# Add or change a sound

You cannot hear. The gates catch levels and repetition; only a person can
tell whether it sounds right. Plan for that from the start.

1. **Where it comes from.** Every sound has a place (`docs/LORE.md`, *The
   sound of the place*; `.claude/rules/audio.md`): a prop within earshot, a
   signature's fixed spot, a bed anchor (`BED_ANCHORS`, `bed_at`), a
   fixture, a creature. Play it with `play_from(x, y, z, ref_dist, gain,
   occlude, f)`, never panned relative to the player.
2. **Synthesis** (`crates/game/src/audio/sys.rs`, the node graph in
   `crates/engine/src/audio.rs`): build it from oscillators, noise, filters
   and envelopes; no samples. Describe in a comment what it imitates. A
   spoken line is the one exception: it comes from the voice pipeline
   (`voice-persona`, `voice-lines` skills; `docs/VOICES.md`), never
   synthesized by hand and never a recording of a person.
3. **Carry and rhythm.** A signature or ambient event needs its `sig_carry`
   entry (near / far / reference distance / gain) and its cadence; per-level
   choices that must not shift the build use `SplitMix64` with a salt. The
   director's hush (`amb_hush`) must soften or hold it.
4. **Gates.**
   - `cargo test --release -p game --test ambient --test soundscape`.
   - `cargo test --release -p backrooms --test listen -- --nocapture`: it
     rises over the bed when it should, nothing piles up at a level's start.
   - `cargo test --release -p backrooms --test loudness -- --nocapture`:
     integrated ±1 LU, range ±3 LU, true peak ≤ −1 dBTP, no one-shot twice
     within 8 s. Re-record only after a person has listened
     (`-- --ignored rerecord`).
5. **Ears.** Ask `audio-analyst` for the numbers and the list of WAVs in
   `target/listen/` to listen to; then ask the developer to listen, and
   record what they said in the commit.
6. **Ledger.** The `sound/ambient` or `sound/mix` entry in
   `docs/content.json`; open issues in `docs/QUALITY.md`.
