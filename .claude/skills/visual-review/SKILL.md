---
name: visual-review
description: Run a blind A/B review of rendered frames (before vs after a change) in The Backrooms with a fresh reviewer that cannot tell which side is new, both orders, ties on disagreement, and the verdict logged for calibration.
---

# Blind visual review

Never grade your own frames. A reviewer that knows which side is new, or
sees one order only, mostly measures its own bias.

1. **Render both sides** headless into two folders with the same file names:
   - the goldens are the *before* of the current shot list
     (`crates/backrooms/tests/golden/`), and `cargo xtask shots` writes the
     *after* to `target/xtask/shots/`;
   - or render your own (`--fresh` so each frame repeats), e.g.
     `--dump-level DIR --views` on the old commit (`git stash` or a
     worktree) and on the new one.
2. **Prepare the pairs:**
   `cargo xtask ab prepare --before DIR1 --after DIR2 [--names a,b] [--question "…"]`.
   It writes each image twice, once in each order, under shuffled names in
   `target/xtask/ab/pairs/`, with `target/xtask/ab/pairs/questions.md`; the key stays in
   `target/xtask/ab/.key.json`. Ask one question per review (realism, dread,
   how frightening a creature is, what is broken). It prints how much of
   each image changed (pixels moved by more than the film grain) and warns
   when an image hardly differs: such a pair can only tie, so choose
   cameras where the change shows, or report its ties as "not seen", not
   as "no preference".
3. **Ask a fresh reviewer**: copy `target/xtask/ab/pairs/` to a neutral
   folder outside the repository (the key, and the folder names you rendered
   into, are readable from inside it), then give the `art-critic` subagent
   only that folder's path and nothing about the change. Ideally also a
   reviewer on a different model family (same-model judges share blind
   spots). Save its JSON answers to a file.
4. **Score:** `cargo xtask ab score ANSWERS.json --reviewer art-critic`.
   Per image, both orders must agree for a preference; otherwise it is a
   tie. The verdicts are appended to
   `crates/backrooms/tests/review/verdicts.jsonl`.
5. **Report** the counts (after preferred / before preferred / tie) and the
   reviewer's evidence for any *before* verdict. A tie on most images means
   the change is not visible, not that it is good.
6. **People.** When the developer looks at the same pairs, score their
   answers too (`--reviewer human`) and add any disagreement with the
   reviewer, with their reason, to `.claude/agent-memory/art-critic/MEMORY.md`.
