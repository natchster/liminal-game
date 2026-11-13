---
name: visual-playtest
description: Run the visual playtest of The Backrooms on an area, a level or a set of levels. It tours the edges of the walkable floor and finds gaps, flicker, clipping, flat or unlit things and coplanar faces. Read the report and the sheets, have the visual-inspector look, reproduce each finding, record it, fix it and tour again. Use it by hand, and after any major change on every level or place the change touched.
---

# The visual playtest

The design, what each detector means and what it misses are in
`docs/VISUAL_PLAYTEST.md`; read its *What it looks for* and *The design*
first. The code:

| Part | File |
|---|---|
| the tour | `crates/game/src/debug/tour.rs` |
| the pixel detectors | `crates/game/src/debug/sight.rs` |
| the geometric detectors | `crates/game/src/debug/things.rs` |
| the renders and sheets | `crates/backrooms/src/tour.rs` |
| the report | `xtask/src/playtest.rs` |

It is never part of `cargo xtask check`. While a check runs, edit only docs.

1. **When.** Run it by hand when asked, and after any major change. A major
   change touches level generation, the architecture, a placer or core
   prop, materials, textures, lighting or the camera. Run it on every level
   or place the change touched (`CLAUDE.md`, *Definition of done*).
   - The Building is levels 1–3, the rooms are 4 and beyond.
   - A change to one zone or prop: that place, with an area. A change to a
     level's generation: that level on three seeds. A change to everything:
     a set.
2. **The scope.**
   - Seeds and levels: `--seeds 4242,7,1999 --levels 2`.
   - An area: `--area x0,z0,x1,z1` in world metres. Take it from a view's
     camera in an earlier report, from `--dump-level`'s JSON, or from
     `--views`.

   An area picks where the tour stands, not what it sees: a thing outside
   it seen from a stand inside is reported. Every stand is rendered for
   the detectors (about 3 minutes a floor of the Building, 19 ms a view).
   The budget (`--budget`, 180 by default) is the views the eyes get: their
   looks written and the sheets, a sample stratified by the kind of place
   (the edge, a corner, a flight), so every kind is seen before any twice.
   The report's *views* column says how many were detected, for the eyes,
   of all. `--quick` renders the eyes' sample only (seconds a floor): for a
   first look, never for the record.

   The first run builds the release game (minutes). A set of three floors
   on two seeds takes about twenty minutes.
3. **Run.** `cargo xtask playtest visual --seeds S,… --levels L,…
   [--area …] [--budget N] [--size 640x400] [--out DIR]`. It writes, under
   `target/xtask/playtest/visual/`:
   - the report (report.md) and every finding (findings.tsv);
   - a folder a floor (`s<seed>/L<level>/`): views.tsv, findings.tsv,
     meta.tsv, each view's look as `<view>.png` at full size, the masks of
     a flagged view (`<view>_gaps.png`, `<view>_flicker.png`), and the
     sheets.

   The run before is moved to `target/xtask/playtest/visual.prev/`, not deleted. To keep a run
   for longer, or when more than one runs (several seeds apart, an
   inspector still reading), give each its own `--out` (a new or empty
   folder; the task refuses any other).

   One floor by hand, the same thing with the game's own flags:
   `cargo run --release -- --tour DIR --seed S --levels L --tour-budget N
   --tour-area x0,z0,x1,z1 --width 640 --height 400`.
4. **Read the report.** Its head says what each column, colour and size
   means. Read the table first, then each floor, flagged first:
   - *gaps*: the background where a ray meets the level within 40 m.
   - *flicker*: pixels changed by a 0.02° turn either way, by more than
     30% of their brightness, in a patch (not a line).
   - *clip*: the eye or its near face inside a thing.
   - *body*: you can stand inside the thing.
   - *plain*, *unlit*, *glow*: within 5 m and in view, flagged when two
     stands agree; unlit and glow only where the place is dark (lit by
     itself where it is lit, it is `selflit`, listed only).
   - *coplanar*: faces of two things on one plane, overlapping.

   A view is flagged at 12 px of gaps or flicker; views.tsv keeps the
   smaller counts too. What is listed and not flagged (`coplanar_alike`, a
   plain thing one view saw) is in findings.tsv.
5. **Look yourself.**
   - The sheets: `sheet_gaps_NN.png` and `sheet_flicker_NN.png` (each
     flagged view's look beside its mask: the gaps red, the flicker cyan,
     the rest the bare render dimmed, the background there magenta),
     `sheet_things_NN.png`, and every `sheet_all_NN.png`. A sheet is
     written only when it has something.
   - The all-views tiles are small. Open the full-size `<view>.png` of any
     tile that looks wrong.
   - The all-views sheets are how *out of place* is found: stretched or
     smeared photographs, a toy among real things, light that does not
     match its neighbours. No detector measures that.
6. **The inspector.** Brief a fresh general-purpose agent with the body of
   `.claude/agents/visual-inspector.md` (below its front matter) as its
   instructions; the project's agents do not load from the session root.
   - Say it is read-only: its `tools` line does not bind a general-purpose
     agent.
   - Give it the repository's root (its docs are named from there) and the
     playtest's folder, `target/xtask/playtest/visual/`. Never tell it
     what you changed or hope it finds.
   - Expect about 20 minutes and 120k tokens for 60 views. For a set, run
     one inspector a floor, in parallel, each given its floor's folder
     (`s<seed>/L<level>/`) and the report; keep to the eye budget (the
     views a floor you give them: `--budget`). Run them in the background,
     and wait for them before step 10, since the tour again moves the
     folder they read.
   - It is a filter, never the gate: vision models miss half of what is
     there and imagine clipping at near contact
     (`docs/research/visual-qa.md`).
   - What it calls a fault of taste or canon (a look two docs disagree on)
     goes to a person: the record's *For a person*.
7. **Reproduce** every finding you act on, with its own camera, and look at
   it:
   - **A look** (plain, a gap, out of place): `cargo run --release --
     --dump-level DIR --seed S --levels L --screenshot x --width 640
     --height 400 --shots-at "NAME:x,y,z,yaw,pitch"`. The picture is
     `DIR/census/NAME.png` (the output calls it a census shot;
     `--screenshot x` turns rendering on, and the level's own PNG and JSON
     land beside it). The report gives each view's camera.
   - **A gap or a flicker**: `--tour DIR --seed S --levels L --width 640
     --height 400 --tour-at "NAME:x,y,z,yaw,pitch"`. It runs every
     detector on that camera and writes every mask however small, and
     `NAME_bare.png`, `NAME_plus.png` and `NAME_minus.png`. A fight cannot
     be seen in one still: flip between those three.
   - **What a gap's rays met**: `TOUR_DEBUG=NAME` with `--tour`. It prints
     the tile, its ground and the solid.
   - **A clip or a body**: from inside a thing the thing is not drawn. The
     report gives a second camera stepped back 0.6 m (`NAME_back`); look
     at both.

   The tour and `--shots-at` settle the lamps' pooled light the same way,
   so the light matches. The film grain and the panels' flicker differ per
   run, so do not compare bytes. A finding you cannot see when reproduced
   is not acted on: record it as not reproduced.

   Reproduce from a stand a player can reach. The tour's cameras are: its
   body is `PLAYER_R` (`tour::BODY`), and
   `the_players_own_step_stops_at_the_tours_stands` walks the player's own
   step to them. A camera you place by hand nearer a thing than that (or
   in the rooms inside a wall tile's slab) sees what no player sees: two
   of the first Building record's findings were seen from a body 15 cm too
   thin, and withdrawn.
8. **Record** in `docs/playtests/visual/YYYY-MM-DD-<scope>.md`, laid out as
   that folder's README says. Then add a line to the README's table. The
   human play sessions are a folder up; do not mix them.
9. **Fix, a kind at a time** (one milestone or commit each):
   - **Gap**: build the geometry that should be there (a ceiling's seam, a
     wall over a partition, a window's panorama).
   - **Flicker or coplanar**: move one face off the plane (a few mm, in
     the placer or the Blender part), or give a decal `polygon_offset`.
     Polygon offset is for decals, never for two solids.
   - **Clip**: either the thing's geometry or what stops the body.
     - On the rooms, what stops the body is `circle_hits_wall`.
     - On the Building, it is the solids: `build_ground_solids_with`, and
       the `extra` boxes beside `cell_front_boxes` that
       `Game::build_solids_leaning` passes. If the thing cannot be
       narrowed, widen what stops the body to it.
   - **Body**: decide whether the thing should stop a body (`PROPS.md`).
     Never make every prop solid; move the thing off the edge if it stands
     where a body must pass.
   - **Plain, unlit, glow**: fix the material's look. Give it a picture,
     wear and a plausible albedo (`.claude/rules/visuals.md`, `add-prop`).
     Glowing is right only for a lamp, a lit window or a screen (where the
     place is lit, such a thing is `selflit`, listed only). The flag is
     easy to clear without changing the look (any wear clears it), so a
     plain fix is accepted only by a blind look before and after
     (`visual-review`), never because the flag went.
   - **Coplanar** is fixed only where a fight is seen (`--tour-at` its
     camera, flip the bare and turned renders): faces of one material that
     overlap usually draw alike, and moving them churns the fixtures.
   - **A detector's false flag**, repeated: tune the detector in
     `sight.rs` or `things.rs` against a known answer in
     `crates/backrooms/tests/visual_tour.rs`. Add the false case as a
     known answer that must stay clean, and make the test fail once
     before you trust it.
   - **Determinism.** The levels are a pure function of
     `(run_seed, level)`.
     - A fix that draws from the level's stream moves later placers.
     - Changing the solids moves the tour's stands even with no draw.
     - Either way the recorded fixtures may change: only through
       `regen-fixtures`, the diff read.
     - Goldens are accepted only after looking (`cargo xtask shots`).
10. **Tour again.** Run the same command. The tour is the same byte for
    byte when nothing changed, so `diff` the findings.tsv with
    `target/xtask/playtest/visual.prev/`'s.
    - The finding should be gone, nothing new flagged at the same places,
      and the looks as good or better (`visual-review` for a change of
      look).
    - After a change to the solids the stands move, so views' names no
      longer line up: compare by place and kind.
    - Put the before and after counts in the record.
11. **What it cannot see** (`docs/VISUAL_PLAYTEST.md`). Say in the record
    what you did not check:
    - gaps further than 40 m, and background beyond the far plane; a hole
      in anything thinner than about 17 cm far off (the gap rays take the
      level 3 cm inside its faces, and half a pixel's span more with
      distance: 9 cm at 40 m, `sight::SPREAD`);
    - a fight seen face-on as thin lines (it is found at a grazing angle),
      and any fight thinner than a patch (the flicker is counted only in
      regions 5 px thick, so a thin far rail losing a sample is not);
    - coplanar faces of one material (listed, not flagged);
    - things inside the architecture's merged meshes (judged by faces, not
      boxes);
    - light that does not match, judged only by eye.
12. **Gates.**
    - `cargo xtask check --fast` after a fix.
    - The full check when the fix touches level generation, rendering or
      creatures.
    - The tour again (step 10).
    - The ledger (`docs/content.json`, the touched family's entry), and
      `docs/QUALITY.md` when a fix closes an open issue.
