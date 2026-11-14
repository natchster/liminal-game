# Saves

What the game keeps of a run between sessions, and where. The work that
documented and replaced the format is [`../plan/REPLAYS.md`](../plan/REPLAYS.md)
(S1–S3); replays, whose container a save is, are [`REPLAYS.md`](REPLAYS.md).

## V1: a save is a replay's start

```
~/Library/Application Support/Backrooms/saves/<name>.ocsave      (name: default)
```

A save is the `OCREPLAY` container (`REPLAYS.md`, *The file*) with one
segment that starts at a floor's anchor and, for a save the game makes,
holds no frames:

```json
HEAD  { "kind": "save", "level": 4, "created_ms": 1763140000000, "build": { … } }
ANCR  { "first_frame": 0, "start": { "kind": "floor", "anchor": { "level": 4, "how": "continue", "ticks": 1092, "state": { … } } } }
FRMS  (none)   HASH  (none)   TELE  {}   END_
```

The anchor's `state` (`Game::anchor_state`, `session/anchor.rs`) is the
run as it stands at that moment: its `save` (V0's fields, below, read by
`SaveData::from_json`) and everything a floor's anchor keeps that V0 did
not (the per-frame stream and its draws, the clocks, the asides' timing,
the rare cadences, the sound's preset and hum, the dust and pooled
effects, the lens).

- **Made** wherever the game saves (`Game::save_game`): on entering a
  floor, at a death, on taking her page or a thing, a medal, a knock
  answered. It is kept in memory (`Game::saved`) and the app writes it off
  the frame, beside its place and then into it. New descent lets it go
  (`clear_save`: the file is removed in its turn after any write before
  it).
- **Read** before the boot, since the title's Continue (its floor, her last
  line) and New descent's question read it. A save made by another
  build is used: it holds no frames that must repeat.
- **Continue** (`continue_game`) puts the save's state back, keeping what
  is the player's now (the settings, the window's view, the keys held), and
  begins its floor again at the start of the next frame's step: a way into
  a floor like a door (`How::Continue`), so a session's log begins a
  segment there and a replay can start from it. The state is put back
  before the floor is built, so the build reads her progress (V0 built
  first). **Waking** after a death is a way in too (`How::Wake`).
- **A save at any frame of a replay** is the recording cut there
  (`ReplayFile::cut`): the segment the frame is in, from its anchor, with
  its frames up to it, the frame its in-point. Opened, it is simulated to
  its frame, unseen, and play goes on from it: `--from-save FILE` (the
  window), Enter in a replay being watched (play goes on from where the
  replay is), `--save-from-replay FILE --at S --save-to OUT` (a file).
  A recording is opened by `--from-save` the same way (from its in-point).
- **More than one.** `--save NAME` reads and writes `saves/NAME.ocsave`;
  the player is offered only the default one. A `--record` playtest, a
  replay and a run `--from-save` (without `--save`) keep their saves in
  memory: the player's is never written by a tool. Headless runs write
  none.

## V0: the storage file (no longer read)

V0's code last stood at commit `4dcdeea` (S1): the save under the key
`opencarole.backrooms.save.v2` in the storage, written by
`Game::save_game` and read by `load_game`, `clear_save` and
`continue_game` in `crates/game/src/session/flow.rs`, through the storage
of `crates/game/src/base/game.rs` (`storage_set`, `storage_get`,
`storage_flush`, `storage_load`). It was removed by S2; nothing converts a
V0 save, and the storage file is left as it was (it still holds the
settings, and an old save under its key, unread).

### The storage file

One file, the **storage**:

```
~/Library/Application Support/Backrooms/storage.txt
```

Lines of `key<TAB>value`, each side escaped (`\` as `\\`, a newline as
`\n`, a tab as `\t`; `Game::storage_flush`, `storage_load`). The whole
file is written again whenever a key changes. A headless run, a
`--record` session and a replay read it at most and never write it
(`storage_path` is `None`). Two keys:

| Key | Value | Written | Read |
|---|---|---|---|
| `settings` | the player's settings as JSON (`Settings::to_json`: look, invertY, fov, fovRev, bob, shake, grain, runToggle, volume, brightness, focusPause, binds) | when a setting changes | at the boot (`load_settings`) |
| `opencarole.backrooms.save.v2` | **the save**, as JSON (below) | on entering a floor (each way in: `begin_level`) and at a death (`on_death`) | by the title (Continue's line and its floor), New descent (to ask first), Continue |

(The key's `v2` is older than this numbering: the descent once began in
the rooms, and a save of that time names floors that no longer exist. It
is not read.)

### The save's JSON (V1 keeps it as an anchor's `save`)

`Game::save_json` writes it, `SaveData::from_json` reads it
(`crates/game/src/session/flow.rs`):

| Field | What | Read back as |
|---|---|---|
| `levelNum` | the floor (a save without one, or below 1, is no save) | `i32` |
| `runSeed` | the run's seed: every floor is built from it | `u32` (0: a new one) |
| `sanity`, `battery` | her nerve and the lamp's charge | clamped to their ranges; absent: full |
| `bottles`, `batteries`, `papers`, `coins` | what she carries | integers, at least 0 |
| `lore` | the notes found, by index | those that exist |
| `archivist`, `survivor`, `storyteller`, `completionist`, `ironwill` | the medals shown | booleans (a number not 0 counts) |
| `deaths` | the run's deaths | integer |
| `arcs` | the lore arcs completed | strings |
| `notebook` | her pages taken (`story::notebook` ids) | those that exist |
| `arcDone` | her story has ended | boolean |
| `creatures` | the run's memory of the creatures: `met`, `habits` (kind → count), `learned`, `learnedOn`, `shown` (`RunMemory`) | what is known; malformed: a new run's |
| `asides` | your asides: `said`, `about`, `weighed`, `owed`, `events` (`AsideMemory`) | the same |
| `took` | what you took: `[what, floor, place]` each (`secrets::keeps`) | |
| `secrets` | the places emptied: key → uses | |
| `savedAt` | when, in ms since 1970 (`save_game` adds it) | not read |

### What V0 left out

- **Where on the floor.** A save is the floor's start: Continue builds
  the floor and puts her at its arrival, the notebook, pockets and memory
  as saved ("this floor starts again when you come back").
- **The per-frame stream, the clocks, what crosses a floor unsaved** (the
  asides' timing, the rare cadences, the sound's preset; everything a
  floor's anchor keeps, `session::anchor`): a Continue is not the run it
  continues, frame for frame, and no replay can start from a V0 save.
- **Order:** Continue builds the floor first and applies the save after
  (`continue_game`: `init_level`, then `apply_saved` and the fitting of
  notes, story, secrets and knocks), so what the build reads of the body
  (the pockets, the nerve) is a new run's.
- **More than one.** There is one save, in the one storage file.
