# Replays and saves: the work

GitHub issues [#4](https://github.com/natchster/liminal-game/issues/4)
(a record/replay engine, separate from saving) and
[#5](https://github.com/natchster/liminal-game/issues/5) (the save format
documented, and aligned with the replay format), queued on 2026-11-14
after the toolchain's move to Rust 1.100 (`MILESTONES.md`, T1–T4). The
engine and its format will be `REPLAYS.md` and `SAVES.md` among the
engineering docs (written by R3 and S1); what a player sees
(the recording control, the Replays screen) goes into
[`../design/UI.md`](../design/UI.md). The research is
[`../research/replays.md`](../research/replays.md).

## The brief (2026-11-14)

From #4, in the owner's words: research how games record and replay
deterministically while keeping files small, and choose the best practices;
prepare the code, removing any randomness the seed does not fix and any
drift floating point allows, without slowing the game or raising its CPU
and memory use much; record and replay from **any point of play to any
point**; document the format; a control to start and end a recording; a
skill that records one level or several; replays chosen and played in the
game's menu.

From #5: document the save format; a save made at any point of a replay,
and a replay's start equal to a save; the replay format changed only if it
must; if the save format changes, the new one (V1) in a new file, the old
(V0) documented, with the last commit and place of its code, which is
removed, and no conversion from V0 to V1; several save files supported but
not offered to the player (the game saves to and loads its default one),
documented for the playtest and review tools.

## The analysis (2026-11-14)

What exists (`crates/game/src/debug/replay.rs`, `--record`/`--replay`):
a recording of every frame's step, clock and input events (JSON, version
1, 276 KB a minute), always from a fresh install, with a four-part state
hash that `tests/replay.rs` checks frame by frame. It cannot start
anywhere but the beginning, is large, binds itself to no build, and has no
control, screen or skill.

What the research found (`replays.md`, section 0 and *What this suggests*):

- **Inputs over a deterministic simulation** is how Doom, StarCraft,
  Factorio, Halo and Trackmania replay, and it fits this game: levels are
  rebuilt from (run seed, level), and one stream draws per-frame chance.
- **A save and a replay are one container** (Quantum, Factorio, StarCraft
  II): a level-start save is a replay with no frames; a save at any frame
  is that save plus the inputs up to it, simulated again on load. A
  game update takes 0.02–0.08 ms, so a minute of play simulates again in
  about 0.1–0.3 s: seeking by simulating from the level's start needs no
  snapshots of the whole game.
- **Hidden state** stands in the way: the wall clock (`now_ms`) drives
  timers; the look settings scale the mouse; 164 hash maps and sets
  iterate in a per-process order; 33 thread-locals; drawing takes the game
  mutably; and every level's per-frame stream continues from the level
  before, so arriving at a level by play and by loading its save are not
  the same state.
- **Floating point:** the ~3,000 `sin`, `cos`, `atan2`, `powi`, `powf`,
  `exp`, `hypot`… calls go to macOS's libm (its results can change with
  the system) or, for `powi`, to whatever LLVM makes of it (documented as
  non-deterministic). Plain `+ − × ÷ √` are exact everywhere: rustc never
  fuses or reorders them.
- **Size:** inputs quantized where they enter (Doom's ticcmds), the time
  in integer ticks, varints and deflate: about 15–20 KB a minute.

## Decisions

1. **Any point by simulating from an anchor.** An anchor is a level's
   start, which a save already describes. The game keeps the inputs since
   the current level's start in memory (about 30 KB a minute), so a
   recording can begin at any moment, even after the fact, and a replay
   seeks by simulating, muted and unseen, from the anchor before its
   in-point. A whole-game snapshot serializer is not built (it would be
   the only way to resume mid-level on another build; not asked for).
2. **Live play is its own replay.** Time is integer ticks of 1/65,536 s,
   from which both the step and the game clock come, in play as in replay;
   the mouse is rounded where it enters, with Doom's carry; the settings
   that steer the simulation are in the file's header and their changes
   are events. What the simulation consumed is exactly what is recorded.
3. **The math is ours.** The simulation's and the level generation's
   transcendental functions come from one module backed by a statically
   linked implementation, `powi` becomes products, and clippy bans the
   std methods there; rendering and the sound's synthesis keep the
   system's (they draw nothing the simulation reads). Kept only if the
   bench holds.
4. **One container, `OCREPLAY`:** a small JSON header (the build it was
   made by, the anchor save, the settings, the in- and out-points) and
   chunks (frames, state hashes every 60 frames, the next level's anchor,
   telemetry), deflated, with an index. A save (V1) is the same container
   with no frames, or with frames up to where it resumes.
5. **Verified as it plays:** a mismatch of the state hash pauses and says
   where; a replay made by another build says so before it plays.
6. **V0 is not read any more.** The old save stays where it was, untouched;
   its format and the commit and code that last wrote and read it are in
   `SAVES.md`.

## Milestones

Each in its own commit, pushed, with the production build made; after the
fifth commit and at the end, a code review and what it finds fixed.

| # | Milestone | Gates |
|---|---|---|
| R0 | The plan and the research | `replays.md` whole; this plan; the queue |
| R1 | **The math is ours**: `dmath` for the simulation and level generation (statically linked, correctly rounded where it can be), `powi` as products, clippy's `disallowed-methods` | bench (build, update p50/p99) within its budgets; what moved re-recorded with the reason; the full check |
| R2 (a, b) | **No hidden state**: time in ticks (play too), `now_ms` the game clock; hash iteration out of the simulation (lint); the thread-locals sorted; drawing writes no simulation state; a level's start an anchor (its per-frame stream from (run seed, level); what crosses levels saved or reset) | arriving at a level by play = loading its save (the state hash, extended); a replay from that anchor = the live run, frame by frame; drawn and headless replays hash alike; the full check |
| R3 | **The format**: `OCREPLAY` (header, frames, hashes, level anchors, index), the mouse quantized with its carry, the settings and their changes, the build; `--record`, `--replay`, `--replay-verify`, `--dump-replay` | decoder and round-trip tests; ≤ 25 KB a minute on the bot's runs; `REPLAYS.md` |
| R4 | **Recording any stretch**: the in-memory log from the level's start; start and stop (a key, the pause menu), the REC mark; segments across levels; seeking by simulating, muted, with a progress bar; the bench's replay speed | a recording begun mid-level replays exactly from its in-point; one across two levels too; the RAM and CPU of recording measured |
| R5 | **The Replays screen**: the list (level, date, length, this build or another), play, delete; while playing: pause, speed, ±10 s, a progress line, exit; a divergence said where | `--screen replays` and the playback screens looked at; `UI.md` |
| R6 | **The skill** `record-gameplay`: one level or several, by a bot or a person, verified, frames rendered from a replay; the ledger | followed end to end by a fresh agent |
| S1 | **The save format documented**: V0 as it is | `SAVES.md` |
| S2 | **Save V1 = the container**: a new file, V0's code removed, the default save at startup, named saves, a save from any frame of a replay ("take control here") | a replay's start = its save; a save at frame N then frames N… = the run unbroken |
| S3 | **The tools from saves**: the playtest, the shots and the reviews start from a save file | the docs (`VISUAL_PLAYTEST.md`, `DEVELOPMENT.md`, `CLAUDE.md`) |

## Progress

| # | Commit | What was done |
|---|---|---|
| R1 | `3acf397` | The math is ours: `engine::dmath` (CORE-MATH, correctly rounded) for all 3,277 transcendental calls of the engine, the game, the placers and the tests, `dpowi` as products and `dhypot` as √(x² + y²); clippy refuses std's (the sound's synthesis and xtask excepted, saying why). 220 placers' fingerprints moved in their last bits and were re-recorded; the reference levels, goldens and range held. Builds +6.3%, update p50 +0.01 ms, within budget (`PERF.md`) |
| R2a | `8ebbcbb` | Time in ticks of 1/65 536 s: one integer a frame gives the step and `now_ms`, in play as in a replay (`Game::advance_clock`); recordings keep it (format 2). No hash iteration reaches the game (audited); the lint `iter_over_hash_type` holds it |
| R2b | (this commit) | A floor's start is an anchor: one entry (`Game::enter_level`, `How`), its state kept as JSON (the save, the stream, the clocks, settings, view, keys, what crosses floors unsaved, the sound's preset and hum, the dust and pooled effects), nothing pending carried over. A zone's reverb on a stream of its own; the Stairwell's child by the game's clock; the hum's lamp picked anew on each floor. `tests/anchors.rs`: a fresh game from the anchor equals play, frame by frame, for doors, the lift and a fall. The reverbs off the frame stream moved 11 goldens' grain (looked at, accepted) and the soundscape's reference test was moved on by the reverbs' draws; the reverb's salt is the first of eight tried on which the Well's knock reads (`MILESTONES.md` 22). Left for S2: Continue still builds before it applies the save (the body read before the things that make it), mended when it starts from an anchor |
