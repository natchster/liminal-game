---
name: drift-sweep
description: The weekly drift sweep for The Backrooms — run every gate, read the results together, record them in the content ledger and the quality scorecard, and turn what people caught into new checks. Run it in a fresh session.
---

# Drift sweep

Once a week, and before a release, in a **fresh session** (long sessions
drift toward tunnel vision; start from the ledger and the scorecard, not from
memory).

1. **Run it.** `cargo xtask drift` runs the full check, writes each content
   entry's `passes` into `docs/content.json` and rewrites the *Latest sweep*
   section of `docs/QUALITY.md`; the full report is
   `target/xtask/drift/report.md`. On the reference Mac, `cargo xtask nightly`
   also runs a 200-case level sweep and records a bench run in
   `perf/history.csv`.
2. **Read what moved**, not only what failed:
   - golden frames that changed even slightly (`target/xtask/shots/*-diff.png`);
   - the range report (`target/xtask/range/report.md`): measures whose
     spread or mean shifted, archive cells that emptied; ask `level-analyst`
     to explain anything that moved;
   - the performance history: a measure creeping up across runs, even
     inside its budget;
   - the largest files (split one before it reaches the cap);
   - the ledger entries whose `last_human_review` is old or missing.
3. **Docs against code.** `tests/structure.rs` checks that named paths
   exist; also read `docs/ARCHITECTURE.md` and `CLAUDE.md` against what the
   week's commits changed, and fix what is no longer true.
4. **Open issues.** Every failure or worrying trend goes into the *Open*
   column of `docs/QUALITY.md`, with its evidence. Do not change a grade
   without an entry under *History* saying why; grades rise only with
   people's confirmation (playtests, listening, look reviews).
5. **The steering loop.** For each regression a person reported that no
   gate caught, add a row to *Caught by people, missed by gates* and add the
   check (a test, a lint, a band, a rubric line) in the same commit. The
   sweep counts these rows; the count should fall over time.
6. **Commit** the ledger, the scorecard and the history with a summary of
   the sweep in the message.
