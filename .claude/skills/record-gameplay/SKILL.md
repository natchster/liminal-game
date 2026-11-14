---
name: record-gameplay
description: Record gameplay of The Backrooms (one floor or several, by a bot or by a person), verify that the recording repeats on this build, render frames from it and look at them, and keep it with what it was made by. Use for playtest sessions, evidence of a change in play, bug reports, review material and benchmarks of a session.
---

# Record gameplay

A recording (`OCREPLAY`, `docs/engineering/REPLAYS.md`) is a session's
inputs, replayed into the same simulation: it repeats exactly on the build
that made it, and only there. Never edit one, and never hand one on before
it is verified and its frames have been looked at.

1. **Choose who plays.**
   - **The bot** (repeatable, any floor, a minute of play in seconds): the
     wanderer (`game::debug::bot::Wanderer`) walks on, turns before a wall
     it sees ahead and about when it stops getting anywhere, runs and
     glances now and then. It does not look for doors, use things or
     flee: a floor a file, and play that a person would not call play. The
     same run seed and floor give the same walk.
   - **A person** (a playtest, a bug as it happened, several floors in one
     recording, anything the bot cannot do): you cannot play, so ask
     them. In the game F9, or the pause list's Record, begins a recording
     at that moment and ends it; a small red "● REC 1:23" at the top right
     says one is being made, and the file's name shows there when it is
     written. `--record FILE` records a whole session from a fresh install
     (playtests: `docs/research/PLAYTEST.md`). To start them on a given
     run and floor: `cargo run --release -- --flags 'seed=7&level=5'`, then
     New descent.
2. **Record.**
   - Bot: `cargo run --release -- --record-bot DIR --seed S --levels 2,5 [--bot-seconds 60]`.
     `S` is the run's seed (the run every floor is built from). Each floor
     becomes `DIR/bot_<S>_L<floor>.ocreplay`: a session from the title
     (New descent, then the way in, about 6 s, letterboxed) and then
     `--bot-seconds` of the bot's play.
   - Person: F9 files go to
     `~/Library/Application Support/Backrooms/replays/<UTC date and time>_L<floor>.ocreplay`.
     A recording begun mid-floor holds that floor from its start and
     replays to its in-point unseen.
3. **Verify:** `cargo run --release -- --replay-verify FILE`. Every segment
   must end "every kept state as recorded". "Made by another build" first
   means another executable recorded it: record it again on this one. A
   segment that leaves its recording on this build is a determinism bug,
   never a property of the file: say where (the frame and the part:
   player, entities, director, world) and find it before the recording is
   used (`docs/engineering/REPLAYS.md`, *What makes it repeat*; a probe
   that counts draws or calls a frame; `tests/replay.rs`).
4. **Look at it.** `cargo run --release -- --replay-frames FILE --sheet DIR [--every 5]`
   renders it headless every few seconds from its in-point
   (`DIR/t0012.0.png`, and `DIR/sheet.png`, four across); it says again
   whether the replay kept to its recording. A bot's file shows the
   title at 0:00 and the way in until about 0:06. Read the sheet: is it
   the play you were asked for? If the bot stands at a wall most of the
   time, or never reaches what mattered, record another seed, or hand it
   to a person; a longer `--bot-seconds` only walks on. Never capture the
   screen. Also: `--dump-replay FILE` (its JSON: the header, frames and a
   telemetry row a second with her place, the director's phase and
   events); `cargo xtask heatmap FILE…` (it builds the game first and
   writes `target/xtask/heatmap/<seed>_L<floor>.png`: where she went);
   `--replay FILE` to watch it in a window, for a person (it plays aloud:
   Space, ← →, − +, Escape). The headless modes are silent.
5. **Keep it with what made it.** A recording is bound to its build: from
   its header (`--dump-replay FILE`: `header.build`) note the commit and,
   when the commit ends in `+` (the tree had changes, which the commit
   cannot rebuild), the executable's hash `exe`, beside the file wherever
   it is cited (a playtest's record, a bug, a review), with the seed and
   floor. Recordings are never committed: keep them under `target/`
   (ignored by git, e.g. `target/playtests/`), in the replays folder, or
   outside the repository.
6. **Gates.** Every recording used: verified on the build that uses it,
   its frames looked at and found to be the play asked for. A plain
   recording needs no ledger entry; a change to recording or replaying
   itself does: the replay system's entry (`replays/engine` in
   `docs/plan/content.json`) and its checks (`tests/replay.rs`,
   `tests/anchors.rs`, `tests/menus.rs`), and the bench's `replays`
   series (`cargo xtask bench`, `docs/engineering/PERF.md`).
