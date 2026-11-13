# The work on the marks and secrets

The plan behind [`../design/SECRETS.md`](../design/SECRETS.md), which says what it is and why: the brief, the analysis, milestones, progress. What is still open is queued in [`MILESTONES.md`](MILESTONES.md).

## The brief (2026-10-30)

"Make navigation hints on levels more subtle and varied. Add secrets
scattered around. Secret items should have slightly positive or negative
lasting effect on player. This effect should not be directly obvious from
look or text on item. The best reference of the way it should be is "eat
me"/"drink me" items from Alice in Wonderland. Use creepypasta and SCP lore
to select secret items and their properties."

## The analysis

What guides the player now, read from the code (`story/`, `player.rs`,
`core_props/tail.rs`, `ui/lore.rs`):

- **Her trail is one mark, repeated.** Every sheet is the same pale paper,
  0.20 × 0.15 m, with the same heavy pencil arrow (a 6 px stroke on 128 px,
  about 10 cm long), lifted a hair off the dark by an emissive tint, every
  3–5 tiles of walk (12–20 m), each in sight of the last
  (`trail_sheet_parts`, `lay_sheets`). From the Lobby down one stretch
  after her camp has lost its sheets. It is a line of road signs: nothing
  in it reads as a person's but the fold.
- **The trails that are not hers** are the same thing larger and darker
  (a whole flat page, a heavier arrow), laid by the same rule.
- **The answer to a cry for help is a game's.** Luminous green chevrons
  (additive, 90% opaque, about a metre long) on every tile of the shortest
  way to the door, bobbing and pulsing for 14 s, with a chime and the line
  SOMETHING SHOWS THE WAY (`spawn_guide_arrows`). The payphone's hint says
  "north. the door is north." whichever way the door is, and shows the
  same arrows (a leftover of the compass).
- **Every torn note glows and points at itself.** An additive halo lies
  under each (0.2 m; 0.32 m and brighter, in its story's colour, for a note
  that continues a story begun, with a tall billboard beam over it), and on
  the nearest wall a rust-red stencil 0.35 × 0.7 m of three chevrons under a
  "?" points down at it (a check mark once read).
- **The door** is found by the place: its green sign and lamps light its
  corner, and it calls (air under the door, the sign's relay). That works
  and stays, as do the Showroom's floor arrows (a store's own signage).
- **Nothing is hidden.** What a floor holds lies in the open (almond water,
  batteries, paper, notes) or in a filing cabinet's drawers. Nothing a
  player takes changes them beyond a meter's worth: there is no lasting
  effect of anything in the game except what the creatures learn.

## Milestones

Each ends with its gates (`cargo xtask check --fast`, the full check where
it touches the level build, rendering or audio), evidence and its own
commit.

| # | Milestone | Done when |
|---|---|---|
| S0 | Research (two notes) and this design | the notes; `lore-keeper` on the canon with S4 |
| S1 | **Her marks, and the place answers more quietly**: the seven kinds in her hand, the mix by depth, each kind's distance, pairing, turns; the flat sheets one even stamp; the lights and the footprints for a cry and the payphone; the notes without halo, beam or stencil | `tests/story.rs`: the trail by kind (each in sight of the last at its distance, the first a sheet, pairing, turns only at turns, on a wall's face), the mix by depth over 39 floors, nothing glows, the flat sheets even and stamped; `crates/game/tests/player.rs`: the answer follows the way to the door, one after another, no line; reference levels, the core props' fixture and goldens re-recorded with a reason; mark views and the prints' screen |
| S2 | **Secrets**: the hiding places and their tells, the sign, what is put away (the things, modelled), the run's plan, the verbs, fitted after the build, the notebook's page, saves | `tests/secrets.rs`: off the way and reachable, the same on every build, a sign near its place, taken stays taken across a retry and a Continue, the plan over a run; frames of each place and thing |
| S3 | **What stays with you**: the body's changes, bounded, felt; the hole's exchange | `tests/secrets.rs`: each thing measured on its system; the bounds and E10; personas and the beatable run unchanged without them |
| S4 | **The canon and the reviews**: `LORE.md`, `GAMEPLAY.md`, `PLAYTEST.md`, `QUALITY.md`, the ledger; `lore-keeper` rounds; a blind review of the marks and the things; the full check | the reviews, the full check |

## Progress

| # | Commit | Notes |
|---|---|---|
| S0 | `8b50e49` | The research (`research/wayfinding-and-secrets.md`, `research/secret-items.md`) and this design |
| S1 | `3a6a139` | **Her marks** (`story/marks.rs`): seven kinds in her hand (the left barb longer), each where it fits at the furthest step in sight of the last at its own distance (a sheet as before; a stack 12 m, a pencil arrow or a turn 9, chalk 7, a corner 6, tape 16), a sheet where no other fits; the first a sheet; where a kind first appears, a sheet beside it; a turn only where the way turns; marks on walls only on a flat face of a wall that stands where it was drawn (never a partition that leans, never the void's railing). Over 39 floors of three runs sheets are 87% of her marks on levels 1–3, 53% on 4–6, 30% on 7–10, 14% on 11–13 (meant: 75, 50, 25, 10%; where nothing else fits she left a sheet); in the rooms a stack is the last resort before a sheet (bits on a floor read as litter). None glows (the sheets lost their emissive lift); tape glints only in the lamp (one animator a level for all of it; one each had added nine to a level). Settling the props nudged a corner touching a wall out of it: the marks are put back where she left them after (`story_marks_settled`). **The flat sheets**: one stamp, a filled black arrow, mid-floor every two tiles. **The place's answer** (`show_the_way`): the lit panels over the first 30 m of the way stutter in turn toward the door (each relay ticking where it hangs), or sixteen wet footprints a pace apart, a wet step each, drying over 20 s; not the lights twice running; no chime, no line; the payphone says only where to look. **The notes**: no halo, beam or wall stencil (its draws kept); a story's ink, a corner turned down when read. Evidence: `--views` (a view of each kind, by the lamp where dark), `--screen way-prints`. Tests: `tests/story.rs` (the trail by kind; `her_marks_change_as_she_goes_down`; the flat sheets even and stamped), `crates/game/tests/player.rs` (the prints; the way shown along the way, in order, without a word) |
| S2 | `3e4b4ba` | **Secrets** (`secrets/`: `mod.rs` the run's plan and the floor's, `places.rs` where each kind fits, `dress.rs` the places made and their verbs, `things.rs` the things, `keeps.rs` the run's list). **The run's plan** (`run_things`, from the run's seed alone): each of the thirteen things on its floors of the first descent (the mushrooms of both kinds and the dish on two each, the rest on one), at most two a floor and one on the Lobby, none in the Showroom or the Borrowed House; the hole on three floors (not the Lobby); below level 13 a floor has one now and then (60%) and its hole a quarter of the time. **The floor's plan** (`plan_secrets`, in the build after her trail, on `SECRET_SALT`, never `rnd()`): two or three places on the Building's floors, none or one on the Lobby, two to four below; off the way (two tiles from her walk and the flat sheets' walks, off the shortest way, three from the arrival and the door, two from her camp, clear of the corner the one in the corner will stand in: `corner::corner_spot`, its choice split out of `place`), four tiles apart; each thing at the first of its kinds of place that fits, what any floor holds (almond water 30%, a battery 25%, a coin 15%, paper 10%, nothing 20%) in a kind chosen by the stream. **The places**: a ceiling tile askew in a cell of the grid clear of the panel (the ceiling's texture runs u along z: a first try sat across the bars), a chair under it; a vent grille hanging by two screws; a cabinet's drawer ajar (the drawer's own verb finds it); a radiator under its rag (the Building's floors; its floor is the tile's: read at the wall's face it was the mass's, and the radiator stood inside it); a cell's bench (two cones of salt by the salt's door); the hole (its exchange: what you last used up back, one thing you carry gone, with nothing on you something touches your wrist and it takes from you); and things lying where nobody would leave them (a dead end, a wet corner, a corridor, a small table, a coat hook). **The sign**: a chalk circle struck through, at 1.4–1.6 m on a wall within two tiles and seen from the place, on about half the places that are opened; on the first floor the first place with a wall to sign is found open and empty, the circle crossed out (3 of 5 runs tried). **The things** (`things.rs`, primitives and the props' Blender parts): mushrooms half again a field mushroom's size (brown ones on brown carpet had vanished), a dish of sweets in cellophane with its card, seeds on a saucer, a cloth bag on a cord, a salt sachet, a jar of blue jelly, a ration briquette, an amber bottle, a tweed cap hung by its brim (turned in a frame of its own: Euler order had laid it flat), a holed coin on a string, a yellowish almond water with its cap off true, a red envelope with hair at its flap. Taking one: a line of its taste (YOU EAT THEM — COLD, AND EARTHY), a sound where it was, the run's list (`took`) and the place's state (`secrets_taken`) saved and fitted after a rebuild (`fit_secrets`); the notebook's page *What I took* in your hand. What each does is S3. Evidence: `--views` adds a view of each place. Tests: `tests/secrets.rs` (6): the run's plan over eight runs, places off the way over eleven floors, taken stays taken across a retry and a save, the hole, the dish's third sweet, the first sign |
| S3 | `b909578` | **What stays with you** (`secrets/keeps.rs`: `body_of`, a pure function of the run's list; `Game::body`, refreshed when a thing is taken and when a save is loaded). Hooked where the numbers already live: your size into the walk and run (`water_mul`) and the eye (eased in at 2.5 cm a second, crouched a little less); your steps' reach as the creatures hear it (`steps_heard`, read by the shared percept, the Hound, the Smiler and the Showroom's staff; the run's own `audible_range` is untouched, so "running" still reads as running); nerve's refill (and in a safe room), the dark's drain and what it leaves; stamina's refill, its ceiling (the meter's bar never past full; it shows when under your own ceiling), what almond water gives (the yellowish one is almond water too); a blow's nerve (`creature_contact`); a Smiler's sight of your lamp; a Hound's smell of you standing still; your look out of a window (the glass's stare); the door's call (its reach and its distance); a cry's chance of the way (capped at 60%); a machine's coin return (no coin, no ring); a third sweet's shaking hands (the lamp's target sways on the game's clock, no draws). Felt: your heart twice in your ears a moment after. Tests: `tests/secrets.rs` (each thing does what it says; whatever is taken, in any number, stays in its bounds: size within two steps, the dark leaves at least 40, a touch at most what it took, stamina 70–112), `crates/game/tests/player.rs` (a pale mushroom: the eye eased down, the walk 4% slower, steps 12% less far; the ration and the medicine: stamina to 112, its refill and nerve's; the bag: the dark's drain and its 45), `tests/vending.rs` (the coin you wear keeps the cup empty). A chase stays under your sprint whatever you took (`chase_cap`, `ENTITIES.md`); the playthrough seeded (`seed_native`: a timeout at run 101, level 14, came from an unseeded stream, not from this change) |
| S4 | `9df0d3b` | **The canon and the reviews.** `LORE.md` (her trail, the cry's answer, the notes without glow, *What is put away*, *Things that change you*, your own page and the line as you take a thing among the channels), `PILLARS.md` (never a game's signpost), `VENDING.md` (the coin you wear), `PLAYTEST.md`, `QUALITY.md`. Two `lore-keeper` rounds, what each found fixed: the hole's touch takes once; the caps per system; the ration printed as what it is; no wet footprints on the Building's floors (the door's call there); the payphone's lines; your page after the torn notes, whiter, in ballpoint; the Lobby only what grows there; the sign scratched with a point (chalk was four hands'); the tastes; no coin among what any floor holds (one at a time, from the machines: almond water 35%, a battery 30%, paper 15%, nothing 20%); no felt cue on taking (the ringing was the Watcher's tell, the heartbeat the dread's). Blind reviews: her marks along the walk preferred 8 to 2, 26 ties; the places and things 10 to 3 (all three the lifted tile floating under an intact grid: it now hangs from one edge out of a gap, `TILE_HANG`), then 19 to 0, 4 ties. The reference levels re-recorded: the run's plan (the Lobby only what grows), what any floor holds, the hanging tile, the sign 8 cm |
