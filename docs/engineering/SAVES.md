# Saves

What the game keeps of a run between sessions, and where. The work that
documents and replaces it is [`../plan/REPLAYS.md`](../plan/REPLAYS.md)
(S1–S3); replays, whose container the next format shares, are
[`REPLAYS.md`](REPLAYS.md).

## V0: the storage file

The format the game writes today. One file, the **storage**:

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

### The save's JSON

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

### What V0 leaves out

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
