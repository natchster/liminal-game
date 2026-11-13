---
name: regen-fixtures
description: The only sanctioned way to re-record The Backrooms' reference data (fixtures, reference levels, goldens, look anchors, loudness and range baselines) after an intended change, with the diff reviewed and the reason recorded.
disable-model-invocation: true
---

# Re-record reference data

Recorded data is how the game notices it changed. Re-recording it without
looking turns every regression into the new truth. Do this only for a
change that was **meant** to change what the data records.

1. **Name the change and its reason** in one sentence. If you cannot, stop.
2. **See what fails first.** Run `cargo xtask check` (full) and list exactly
   which recordings fail. Each must follow from the change; anything else is
   a regression to fix, not to re-record.
3. **Re-record only those**:
   - creatures: `REGEN=1 cargo test --release -p game --test entities`;
   - level generation: `cargo test --release -p game --test archgen --test world_queries --test world_build -- --ignored rerecord`;
   - reference levels: `cargo test --release -p backrooms --test reference_levels -- --ignored rerecord`;
   - what each placer makes (its fingerprints): `REGEN=1 cargo test --release -p placers --test fingerprints`
     (a new placer adds its lines; code moved between the placer crates must change none);
   - core props: `cargo test --release -p game --test core_props_head -- --ignored rerecord`,
     and their second half: `cargo test --release -p game --test core_props_tail -- --ignored rerecord`;
   - frames: look at `target/xtask/shots/<name>.png` and `<name>-diff.png`
     first, then `cargo xtask shots --accept NAME,… --reason "…"`;
   - look anchors (rarely): `cargo xtask shots --accept-look NAME,… --reason "…"`;
   - loudness, after a person has listened: `cargo test --release -p backrooms --test loudness -- --ignored rerecord`;
   - range, after reading the report: `cargo xtask range --accept --reason "…"`.
4. **Review the data diff**: `git diff --stat` and a look at the changed
   values. They must match the change in size and place.
5. **Run `cargo xtask check` again**; it must pass.
6. **Commit** the code and the data together, the reason in the message
   (the goldens and range baselines also log it in their `ACCEPTED.md`).
