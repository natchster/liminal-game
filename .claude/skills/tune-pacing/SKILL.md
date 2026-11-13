---
name: tune-pacing
description: Tune The Backrooms' pacing director (menace, build-up, peak, relief, withdrawal) from bot runs: measure the phase statistics, change one parameter, compare the distributions, repeat.
---

# Tune the pacing

The director is `crates/game/src/entities/director.rs` (phases calm → build
→ peak → relax; the menace gauge; withdrawal after a close call). Tune it the
way a playtester's metrics drive a design change: one parameter at a time,
judged by distributions, not by a single run.

1. **Measure first.** Run
   `cargo test --release -p backrooms --test pacing -- --nocapture` three
   times (the creatures' decisions draw from the native stream, so runs
   differ) and write down, per run: time to the first build, calm and relief
   lengths, peaks, close calls survived, withdrawals, deaths.
2. **Name the goal** in those numbers and quote the pillar it serves
   (`docs/design/PILLARS.md` 5, *Dread, then release*), e.g. "fewer deaths per peak
   for a careful player, same number of peaks".
3. **Change one thing** (a threshold, a rate, a radius), with a comment
   saying why.
4. **Measure again** (three runs) and compare the distributions, not the
   best run. Keep the change only if the goal moved and nothing else broke
   (`pacing.rs` bands, `tests/entities.rs` behaviour tests).
5. **Re-record** the director's sections of `entities.json` if they changed
   (`REGEN=1`, the `regen-fixtures` skill).
6. **Report** before/after numbers in the commit message. Whether the result
   is scarier is for fresh players (roadmap 4.4); say that it is untested
   with people until it is.
