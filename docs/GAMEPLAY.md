# Gameplay: the descent as a story

What a level asks of the player, why, and the plan that moved it from "walk
toward the arrow until something chases you" to a descent with a reason. The
canon it serves is [`LORE.md`](LORE.md); every change here quotes a rule in
[`PILLARS.md`](PILLARS.md).

(Since 2026-10-27 the descent begins in the Building, three floors of vast
concrete above the rooms, where the challenge is the way down and the
danger far off: [`MEGASTRUCTURE.md`](MEGASTRUCTURE.md). Her trail, camps
and pages run through them as below; her camp there is a cell, and her page
sounds the siren.)

## The analysis (2026-10-26)

What a normal level was, read from the code:

1. You arrive at tile (1, 1). A random objective line is set once and never
   changes (`FIND THE EXIT DOOR`, `KEEP MOVING`, `DO NOT ANSWER THE PHONE`
   on levels with no phone; `session/flow.rs`, `TEXT_POOLS["objectives"]`).
2. A HUD compass labelled EXIT points at the door the whole time, at no less
   than 15% opacity however far away (`ui/hud.rs` `update_hud`). The door is
   always the open tile furthest from the spawn (`props_exit_door`).
3. You walk the way the arrow points. Pickups, drawers, a phone, a vacuum and
   2–4 torn notes lie about; the notes are drawn at random from a pool of 36,
   so a story's pages (Elias, Day 47 → Day 219 → Final) arrive in any order,
   on any floor.
4. After 45–80 s of calm the director sends one creature. You run. It backs
   off. Repeat until the door.
5. The door leads to the next level. There is no end; medals reward
   collecting (all 52 notes, level 10).

Measured against the pillars:

| Pillar | Where the game worked against it |
|---|---|
| 1. Emptiness is the threat | You are never lost: the arrow makes the size of the place irrelevant, so the emptiness is scenery, not a threat. |
| 4. You are never shown everything | The compass is a map ("no maps that explain the place"). The objective line is a tutorial ("rules are learned, not told"). |
| 5. Dread, then release | The only rhythm is the director's; nothing the player chooses to do brings a scare on or earns relief. Walking is the whole verb between chases. |
| Canon: lore is noticed | Notes are loot. Arcs complete out of order; the medals count pages, not understanding. Nothing pulls you down except the next door. |
| Coherence | A level is rebuilt differently after you die if you read a note on it: the note placer shuffles only the *unread* ids, so its draw count, and every placer after it, depends on the save (`tail_lore_notes`). `Continue` builds before the save is applied. |

## The design

**One sentence:** you follow someone down, by the trail she left, and her
notebook is the story.

**M.** (the author of *M.'s Rules*) came into the Backrooms after **E.**
(Elias), who stopped answering. She followed his marks down and left her own:
sheets of paper folded twice, each with a pencil arrow, from where she arrived
to where she camped and on to the way down. On every floor there is **her
camp**: a bedroll, her things, a lamp still burning, and **one page of her
notebook**. The pages are in order of the descent. They tell her search, hint
at the floor's dangers as a survivor would, and never explain. They end at
**the last camp** on level 13, the first floor below every anomaly. After
that the paper is yours.

The verbs this gives a level, in place of "follow the arrow":

1. **Find the trail.** Her first sheet is within sight of where you arrive,
   but the trail is sparse, has gaps, and gets sparser as she did. Deeper
   down, other trails lie (flat sheets that do not go down). Reading the
   trail is a skill; it is never explained.
2. **Choose the detour.** Her camp is off the straight way. It holds supplies
   and the page. Taking the page wakes the place: her lamp gutters out and the
   calm is cut short. You choose when the dread starts.
3. **Find the door by the place.** The exit has a green light that carries
   down corridors and a sound of its own (the sign's buzz, air moving under
   the door). Crying out for help still works and still costs.
4. **Read.** The notebook (Tab) fills in order; a missed page stays torn out.
   The last line you read stays with you on screen, as the thread.
5. **Reach the bottom of her story.** Level 13 is the last camp and an
   ending; the descent goes on after it, without her paper.

Pillar check: emptiness is a threat again because you can be lost (1); the
camp is "what a place was for, shown by what is left" (1); the trail is
familiar (paper markers, already canon: "Sometimes you find someone else's
trail going the other way") and goes wrong deeper down (2); the exit is
*heard* before it is seen and every sound comes from somewhere (3, 4); no
map, no tutorial, notes hint (4); the page is a player-chosen build-up with
the director's release after it (5).

## Milestones

Each milestone ends with its gates passing (`cargo xtask check --fast`, and
the full check where it touches level generation, rendering or audio),
evidence (a test, a headless PNG, a pacing log) and its own commit.

| # | Milestone | Done when |
|---|---|---|
| G0 | This plan | committed |
| G1 | **The level stays the level.** A level's build never reads the save: notes are placed from the whole pool, then the read ones are swapped for unread ones after the build on their own stream; `Continue` and restarts rebuild the same level | a test builds a level, reads notes, rebuilds: same draws, same objects; fixtures unchanged |
| G2 | **Wayfinding belongs to the place.** The compass goes; the exit is lit into the light map and has its own sound; the payphone's hint becomes the guide arrows; no tutorial objective lines | tests: no compass element, the exit's sound anchor and light; personas and playthrough still reach every exit; a PNG of the exit glow |
| G3 | **Her trail and her camp.** `story` module: after the build, on a salted stream, a camp off the shortest path and a trail of folded sheets from the arrival through the camp to the door, each sheet visible from the last except at gaps | tests: deterministic, reachable, followable, ends at the door, on every level kind; reference levels and goldens re-recorded with a reason |
| G4 | **Her notebook.** Thirteen pages (eight for the floors, four for the anomalies, one last), placed at the camps in order; the journal's NOTEBOOK; the thread line; saves; the canon in `LORE.md` | tests: page order over a run, journal, save/load; a `lore-keeper` review of the text |
| G5 | **The page wakes the place.** Taking the page cuts the calm short and puts her lamp out; level 1's first build waits for the page (or a long calm) | pacing and persona logs: the build follows the page; bands hold |
| G6 | **Trails that lie.** From the floor of her fifth floor page on (her sixth since the Building came above the rooms: `MEGASTRUCTURE.md`) (the page that says *not all the paper down here is mine*), flat sheets branch off her trail into a dead end where the place waits | tests: the false trail branches from the true one, ends in a dead end, never reaches the door |
| G7 | **The last camp.** Level 13: the final page, no trail onward, the ending card, and the descent after it | a test plays the arc's end through the flow; saves keep it |
| G8 | **Evidence and the ledger.** Playtest focus questions for the story, `content.json` entries, `QUALITY.md`, the full check | full check and drift sweep pass |

## Progress

| # | Commit | Notes |
|---|---|---|
| G0 | `b23834e` | the plan |
| G1 | `f16bd4b` | the old build moved even the barricades after a note was read (`tests/story.rs` fails on it); now the build runs with nothing read and `fit_lore_notes` swaps read notes for unread ones afterwards. The rare level built with every note read hides nothing: its notes keep the READ mark |
| G2 | `b4bd003` | the compass and the objective line are gone; the door lights its tile in the light map and calls (`sfx_exit_call`: a draught and the sign's relay, every 8–10 s). `the_door_is_heard` measures the door alone (lockstep runs with and without it): +6 to +18 dB over the bed two tiles off, +8 to +13 at five, +1 to +4 at nine (about 30 m). The payphone's hint (pinned by a fixture) now brings the floor arrows. `--views` adds an exit view |
| G3 | `0fa2e02` | `story/`: `plan_story` (pure: camp, walk, sheets, gaps) and `dress`. Placed after settling (the walls are final then; placed before, a tenth of the sight lines failed in the finished level) and before placement. Her camp needs a wall a body long behind it (a pillar left the tallies in mid-air). Found on the way: every headless view kept the spawn's prop cull, so small props far from the spawn were missing from the zone views and goldens (fixed: each view culls for its own camera); and the level sweep found an almond inside a maze wall on seed 905325066 L8, there before her story (fixed: `free_buried_pickups`) |
| G4 | `1d785e5` | `story/notebook.rs`: 13 pages and their threads, `page_on` (floor pages count the run's ordinary floors). Her page lies on the bedroll ([E] TAKE HER PAGE); the journal lists her pages first in the order of the descent, a torn edge for each floor passed without taking hers, and opens on the page just taken; the thread replaces the objective line; saves keep the notebook. The `lore-keeper` review of the first draft found Rule 1 contradicted, marks of E.'s promised that the world does not have, a sound the Loop does not make, two threads that were orders, and lines lifted from existing notes; all rewritten |
| G5 | `73dedb7`, fixed in `298830b` | taking her page lets her lamp gutter out over 1.6 s (its light leaves the light map) and `director_wake` cuts the quiet to 4–9 s; level 1's first quiet is 110–150 s while her camp is there. `her_page_wakes_the_place`: on three levels the log reads `story:page`, `wake`, `build` within the window, and a retry keeps the lamp out. Pacing and personas bands unchanged (first builds 34–67 s). `73dedb7` was committed with the fast check failing (its exit status was lost in a pipe): the longer first quiet took a second native draw on level 1, where every headless run starts, and moved every golden. Fixed by spreading the one draw over the longer span; CLAUDE.md invariant 1 now says so |
| G6 | `34c615a` | `plan_false_trails`: from the floor of her page `f5` one trail of flat sheets (two from `f7`'s) leaves her walk for a dead end three walls round, 8+ tiles from the door, never back onto her walk; laid like hers (`lay_sheets`, shared, her own trail's draws unchanged: reference levels differ only from the `f5` floor on). At its end a breath close by and `director_lure` (a quiet ends in 1.5–3.5 s; a build-up's stalker learns where you are). The flat sheet is a whole page with a heavy black arrow; hers is folded, in pencil |
| G7 | `8aa398f` | `story/ending.rs`: on level 13 her lamp is cold, her shoes stand by the bedroll and her trail stops at her camp; leaving the floor (door, lift or hole) the first time shows THE LAST CAMP over black (pages, notes, deaths; the overlay's controls hidden), E or a click goes on with her last 12 sheets; `arc_done` is saved, and below 13 there are no camps or trails. The playthrough (13 levels by default) presses E at the card |
| G8 | (this commit) | `content.json` gains `story/trail`, `story/camp`, `story/notebook`, `story/false_trails`, `story/ending`, `wayfinding/exit` (all passing in the drift sweep); `QUALITY.md` grades Story and wayfinding B (gates, no people); `PLAYTEST.md` asks about her paper, her page and the notebook; the `lore-keeper` reviewer reads the notebook too |

## What only people can tell

Every gate above holds what can be measured: the door is heard, the trail
can be followed, the pages are in order, the page wakes the place. What
the design is *for* is not measured by any of them, and is the next
playtest round's focus (`PLAYTEST.md`):

- Without a compass, does a fresh player find the door, and how long does
  a floor take? (The recordings' telemetry gives the time; the heatmap
  where they wandered.)
- Is her trail a story or just a guide line? Do they stop at her camp, or
  pass it?
- Does anyone notice the flat sheets before following one, and does the
  dead end feel fair?
- Is taking her page a choice they make (do they wait, or scout the door
  first)?
- At the end, can they say who E. and M. were?
