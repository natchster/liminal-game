# Visual playtests

What the visual playtest (`docs/VISUAL_PLAYTEST.md`; the `visual-playtest`
skill) found, one file a playtest: `YYYY-MM-DD-<scope>.md`, the scope a
few words (`building-1-3`, `stair-halls-area`, `rooms-pool`). The play
sessions with people are a folder up (`docs/playtests/`, from their
`TEMPLATE.md`); these are the machine's tours and what was made of them.

A record holds, in this order:

1. **The run**: the command (seeds, levels, area, budget), the commit, and
   the table from the report, its coverage with it (views detected, for the
   eyes, of all; `--quick` is not a record).
2. **The findings**, one row each: an id (F1, F2, …); the kind (gap,
   flicker, clip, body, plain, coplanar, or out of place for what the eye
   found); the thing (as the report names it); the view and its camera
   (`--shots-at` or `--tour-at`); the pictures looked at; the severity (3
   breaks the illusion at a glance, 2 noticed when looking, 1 only when
   hunting, 0 not a fault); who confirmed it (the detector, the
   `visual-inspector`, you, a person) and where you disagreed.
3. **Not faults**: each flag found false and what the detector mistook (a
   detector's false flag repeated is a detector to tune, against a known
   answer).
4. **For a person**: what is a question of taste or canon, not a fault
   (two docs that disagree, a look that may be meant).
5. **What will be fixed and what will not**, and why; where in the code.
6. **The tour again**: the same command after the fixes, the counts before
   and after, each finding gone or not, anything new; the commits.
7. **Not checked**: what the detectors cannot see on this scope.

| Date | Scope | Findings | Fixed |
|---|---|---|---|
| 2026-11-13 | [the Building, levels 1–3, seeds 4242 and 7](2026-11-13-building-1-3.md) | F1 door frames fight the piers; F2 the crouched eye in the parapets; F3 the exit door walked into, flat; F4 the metalwork flat; F5 the photographs at corners; F6 post tops | (in progress) |
