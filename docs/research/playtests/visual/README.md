# Visual playtests

What the visual playtest (`docs/engineering/VISUAL_PLAYTEST.md`; the `visual-playtest`
skill) found, one file a playtest: `YYYY-MM-DD-<scope>.md`, the scope a
few words (`building-1-3`, `stair-halls-area`, `rooms-pool`). The play
sessions with people are a folder up (`docs/research/playtests/`, from their
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
| 2026-11-13 | [the Building, levels 1–3, seeds 4242 and 7](2026-11-13-building-1-3.md) | F1 door frames fight the piers; F2, F3's clip (withdrawn: the tour's body); F3, F4 the metalwork and the exit door flat; F5 the photographs at corners; F6 the rail posts; F7 the lamp posts walked through; F8 a flight's steps through its parapet | F1, F6, F8 (and the gaps detector); F4 tried, not preferred; F7, F4, F5 queued (`MILESTONES.md` 15–17) |
| 2026-11-15 | [the writing on the rooms' walls, levels 5, 7, 9, seed 4242](2026-11-15-rooms-writing.md) | none in the writing (three coplanar patches that do not fight); the rooms' own gaps and flicker found, not from it | queued (`MILESTONES.md` 23) |
| 2026-11-16 | [the assets (almond water, the battery), levels 5, 7, 12, seeds 4242 and 7](2026-11-16-assets.md) | none in the pickups; F1, the eye in a bottle on a filing cabinet, is the tour's body inside the cabinet (the rooms' body finding); the rooms' own gaps, flicker, clip; for a person: the brick, the flashlight at 35 cm, the wallpaper's shapes, edges bowing | queued (`MILESTONES.md` 23) |
| 2026-11-17 | [the doors: six areas of seeds 4242 and 7, one a kind of door](2026-11-17-doors.md) | F1-F3 coplanar faces in the exit door and its sign, F4 the broken lift's panel flickering on its frame, F5 its writing across the gap on the air; the inspector's own: the rooms' and the Building's faults, not the doors' | F1-F5 fixed; the rest kept, why in the record |
| 2026-11-17 | [the lamps of the facility, seed 7 level 6: a hall and a tunnel](2026-11-17-lamps.md) | F1 the jar lamp's guard fighting its glass; F2 its cast fitting plain | F1 fixed; F2 kept, why in the record |
| 2026-11-17 | [the Utility Halls remade, seed 7 level 6: the whole floor against its tour before](2026-11-17-utility-halls.md) | F1 the haunch tangent to its wall flickering; F2 the runs and bolts in flat paint; F3, F4 a haunch's open end and a coupling on a clamp coplanar; F5 the far runs' thin parts shimmering; F6 the haunch's photograph stretched round its arc; F7 haunches crossing at inside corners | F1-F4, F6, F7 fixed; F5 kept, why in the record |
| 2026-11-17 | [the Habitable Zone remade, seed 4242 level 5: the whole floor against its tour before](2026-11-17-habitable-zone.md) | F1 a neighbouring zone's wall fighting the hall's, the floor's own, more visible on paler paint | F1 kept for `MILESTONES.md` 23; nothing of F6's own |
| 2026-11-18 | [drops in the halls, seed 7 levels 6 and 13, 4242 level 13: each floor against its tour before](2026-11-18-drops.md) | F1 the catwalk's stringers fighting its deck; F2 the grating shimmering from afar; F3 a rail run into a pier at a drop's corner | F1-F3 fixed; the railing's wear to F11 |
