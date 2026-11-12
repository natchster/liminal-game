---
name: add-cutscene
description: Add or change a cutscene in The Backrooms (a moment that borrows your eyes and gives them back): whether it should be one, its source file, its trigger, its stage, the preview, the contract suite, goldens, the canon and the ledger.
---

# Add or change a cutscene

The design, the file format and what each rule is for are in
`docs/CUTSCENES.md`; read its design first, and its table of the
cutscenes. The engine is `crates/game/src/cutscene/` (`play.rs` the player,
`cues.rs` what a cutscene may name, `uses.rs` where the game plays them),
the test engine `crates/game/src/debug/cutscene.rs`. The way in
(`way_in`), the shaft (`shaft`) and the deaths (`death_taken`,
`death_mind`, `death_fog`) are the worked examples: read the one nearest
yours. While any `cargo xtask check` runs, edit only docs: its later
stages rebuild the game.

1. **Should it be a cutscene at all?** First-person games rarely take the
   camera, and losing control breaks a run (`docs/research/cutscenes.md`).
   Take it only for what play cannot show (a look into something, a fall),
   briefly (a look under 7 s; `skip = "never"` only for 3 s at most, a
   death), and never to explain: a cutscene is the canon's *A look you
   take* (`docs/LORE.md`): no caption, no text, no narrator. If play can
   already show it (a set switched off, a door opening), it is not one,
   even when asked: say so and stop. A trigger the player can repeat (a
   verb on a prop) plays once per thing, floor or run, then does what play
   did; say which in the source's header and test it (the shaft: once a
   lift; what looks back only once a run).
2. **The brief.** Quote the pillar it serves (`docs/PILLARS.md`) in the
   source's header comment and in its row of `docs/CUTSCENES.md`. Brief
   the `lore-keeper` before you build it (by brief: a general-purpose agent
   given `.claude/agents/lore-keeper.md`): the idea in three lines, the
   paragraph of `docs/LORE.md` it touches, and the code's gates (where,
   how often, when it will not play). The shaft's first version glowed
   where the canon says eyeshine never does.
3. **What the engine cannot do yet** (`docs/CUTSCENES.md`, *Left open*):
   there is no actor track (a figure posed over time, its brain held): the
   shaft's eyeshine is made and removed by events; you have no body, so a
   camera outside your eyes shows nobody where you stand; there is one
   `thing` anchor (and `killer` only in a death); a moment tied to a
   creature that lets you live (a sighting, a reveal) is refused while it
   hunts or searches near (`cutscene_danger`), and the Stairwell and the
   Loop refuse a cutscene (they write the camera themselves). If yours
   needs one of these, build it in the engine first, with its tests.
4. **Its shape.** Decide its trigger (a verb on a prop, an event, a death,
   the title), its world (`live`: the world runs on; `held`: only the
   place's light and sound), its end (`then = "back"`: your hands back in
   place; `"card"`: the death card), whether it is skippable, what of it
   must hold if it is skipped or broken off (those events are `on_skip`),
   and what must be cleaned up however it ends (`finally`).
5. **Write the source**, `assets/src/cutscenes/ID.toml` (its `id` the file's
   name; comment it as the others are). The traps, each found the hard way:
   - Anchors are frames: x right, y up, z out of the anchor's face, so -z is
     forward. A thing's frame is `Frame::thing(foot, yaw)` with its front
     as +z; a thing facing you has your yaw, not your yaw + π.
   - Read a prop's place and facing from what the world has *after
     settling*: a child of its group, `world_position()` and
     `world_direction()` (its +z), not a position or normal recorded at
     build. Settling moved a lift a metre; a set's recorded normal was +z
     whatever way it faced.
   - With `blend_in = 0` the first key is the camera as it is (`t = 0`,
     `anchor = "start"`, nothing else); otherwise it blends from the camera
     as it was. `then = "back"` must leave its veil clear at the end.
   - Speed: turns under 150°/s, moves under 4 m/s, roll only in a death,
     `fov` (an offset from play's) within 15°; mark a deliberate start or
     fall `jolt = true`. An eased turn peaks at one and a half times its
     mean, so 180° wants 1.9 s; a spline's speed peaks above its eased
     average next to a near key. A verb has no facing cone (E takes what is
     under you or alone behind you), so a look may start, and hand back,
     with your back to it: give its first segment and its blend out time
     for that, and give it an `:away` stage (step 7).
   - Sounds and events by name only: a new sound or event is a line in
     `cutscene/cues.rs` and a function on `Game`. A prop's own sound kept
     in a private match is pulled out into a public `sfx_*` both share (it
     is the same sound: `add-sound`'s gates hold as before). A sound draws
     from the native stream; a skip plays none, so a death's sounds stay
     in `on_death`. A cue set at the very end runs at the end.
   - The tracks less used: a key with `cut = true` jumps to it at its
     time (another angle; the contract lets that one frame through, so do
     not hide a fast move in a jolt); `[[say]]` posts one of your asides
     (`use`, `subject` from `audio::asides`), said by their own rules once
     your hands are back, never during; `repeat_from` is where its short
     form starts on a repeat (a death: counted by what killed you), with a
     blend in of at least 0.2 s from the camera as it was.
   - Something a cutscene puts in the world (the shaft's eyeshine) is made
     by an event and removed by another and by `finally`, never left.
   - Delaying an effect behind a cutscene keeps the world's per-frame draws
     going longer before it (a lit set's flicker): the native stream after
     it moves for anything that uses the verb (bots, replays): run the full
     check.
6. **Wire the trigger.** One function in `cutscene/uses.rs` that starts it
   (`self.play_cutscene("ID", Anchors { thing, killer })`) and returns
   whether it played; the call site (a prop's `on_use` closure in
   `core_props/`, `on_death`, the menu) keeps the old behaviour under
   `if !played`. It does not play with one already playing, outside play,
   in the Stairwell or the Loop, without an anchor it names, or (one that
   gives your hands back) in danger. A cutscene never moves you; a new
   run's start pose may be set before it (the way in). The props a hand
   can use are in `core_props/usable.rs` (each `usable_*`, its `interact`
   call, and the gates on where they stand); their records in
   `base/records.rs`. Then grep the tests for the prompt's text and the
   functions you changed: a test that used the verb and checked its effect
   at once now steps frames until your hands are back.
7. **A stage** in `debug/cutscene.rs` (`cutscene_stage`, `cutscene_trigger`,
   `cutscene_signature`): set the game up as play meets it, on several
   seeds or places and facings (three lifts and one turned away, three
   Wells), start it through the real path (`cut_override` with the real
   E, `on_death`, `new_descent_with`), and sign what a skip must leave the
   same (the lift peered, the run's seed, the card). To find a floor with
   a prop, read `crates/backrooms/tests/reference/` (each floor's props)
   with the prop's gates, or probe with a throwaway test. List the stages
   in the source's `stages`.
8. **Compile and look**: `cargo xtask cutscene write` (a new or changed
   source fails `check` until it is written), then `cargo xtask cutscene
   check` and `cargo xtask cutscene preview ID`, and look at every stage's
   `target/xtask/cutscene/ID/STAGE/sheet.png` (frames at
   `target/xtask/cutscene/ID/STAGE/tSS.SS.png`; `--every 0.1` for a fast
   moment) and read `contract.txt` and `timeline.tsv`. Never edit
   `crates/game/assets/cutscenes.json` by hand: the fast check compiles the
   sources and compares.
9. **Tests.** Add its line to `contract_tests!` in
   `crates/backrooms/tests/cutscenes.rs` (a test a cutscene, so the check
   runs them side by side; `every_cutscene_has_a_contract_test` fails
   until you do), and the contract suite plays it on each of its stages
   (`cargo test --release -p backrooms --test cutscenes`: its short form,
   with shake and head bob off, the HUD read, your nerve after a skip as
   after watching; played, skipped
   early and late, paused, broken off if live, at 30, 60 and 120 frames a
   second). Add a test of what it does to the world beside the shaft's and
   the deaths' in `crates/backrooms/tests/cutscenes.rs`, and make it fail
   once on purpose (take the effect out) before you trust it.
10. **Goldens**: a frame at a named time in `xtask/src/shots.rs` (`FIGURES`:
   `--cutscene ID --stage S --cut-at T`); run `cargo xtask shots` (it
   renders into `target/xtask/shots/NAME.png`, with `NAME-diff.png` for a
   change), look, then `cargo xtask shots --accept NAME --reason "…"` and,
   for a new shot, `--accept-look`. A recorded fixture whose session uses
   the trigger changes with it (the shaft's `core_props_tail`): only
   through `regen-fixtures`, the diff read field by field.
11. **Canon**: what it shows written into `docs/LORE.md` from the code
    (where it happens, what you see and hear, what it costs, when it will
    not play), and the `lore-keeper` again on the source and that text.
12. **Ledger and docs**: a `cutscene/ID` entry in `docs/content.json` (copy
    one of the five: `kind` "cutscene", its checks; `passes` as they
    stand, the drift sweep keeps them); its row in the table of the
    cutscenes in `docs/CUTSCENES.md`; questions in `docs/PLAYTEST.md`'s
    *Cutscenes* block (what only a person can tell of it). The structure
    test fails a cutscene with no ledger entry.
13. **Gates**: `cargo xtask check --fast`; the full `cargo xtask check` when
    it touches creatures, deaths, rendering, audio, or delays an effect
    (step 5).
