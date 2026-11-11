# Cutscenes: borrowing your eyes

How the game takes the camera for a moment and gives it back: the engine
that stores, plays and tests a cutscene, the clue that your hands are
taken, the pipeline that makes one, and the first three uses (the way in
from the title, looking into a broken lift's shaft, and dying). The
pillars it serves are in [`PILLARS.md`](PILLARS.md); the research behind
it is in [`research/cutscenes.md`](research/cutscenes.md), summed up in
[`SOURCES.md`](SOURCES.md) section 17.

## The analysis (2026-11-11)

Read from the code. The game has no cutscenes; it has moments scripted
one by one, each with its own clock:

| Moment | How it is done | What is wrong |
|---|---|---|
| New descent | `start_new_game` draws a new run seed and builds level 1 again behind the title | the title shows one Well and the run starts in another: a cut, from a place to a different place |
| A hole (`begin_fall_down`) | a callback every 30 ms rolls the camera 0.02 rad and lowers it | frame-rate dependent (a step per callback, not per second); the roll is never eased |
| A door, a lift (`next_level`, `next_level_via_elevator`) | a fade and a `schedule`d rebuild | fine as a transition; nothing says your hands are taken during the 0.65–1.5 s, the keys simply do nothing |
| The broken lift's shaft (`tail_broken_elevator`, "Peer into the shaft") | a chime, a breath, a shake, an aberration flash and a toast in capitals chosen at random: "SOMETHING WAS DOWN THERE.", "IT LOOKED BACK.", "YOU SHOULD NOT HAVE LOOKED.", "THE DARK BREATHED." | you are *told* you looked; the camera never moves; one toast is advice after the fact (`UI.md` U8 took the same out of the death lines) |
| Death (`on_death`) | the killer posed in your face, `update_death_cam` aims the camera at its head with a shake, `schedule`d flash, fade and card | the camera holds at standing height and cuts to black; the same 1.4 s every time, whatever killed you; the timing lives in five callbacks |
| The title (`update_title_view`) | the camera at the spawn, turning slowly | already the place, live: the right start for a way in |

`schedule` runs on the app's real clock and keeps running while the
world is paused; none of these moments can be paused, skipped, tested
for where the camera ends, or reused.

## What the research says

Section 17 of [`SOURCES.md`](SOURCES.md) has the sources; in short:

- **Store tracks of keys, and say what is left at the end.** Unreal's
  Sequencer and Unity's Timeline keep keyed curves and events, and each
  declares its end (*Restore State* or *Keep State*, *Hold* or *None*);
  Valve's choreographed scenes are text compiled for the game; signals
  keep a timeline from naming gameplay code.
- **Hand back to the live camera.** Cinemachine eases a shot in and out
  against the gameplay camera as it is (2 s by default); Epic sets the
  control rotation and avoids a jump in the field of view at the end.
- **First person rarely takes the camera.** Half-Life 2 and SOMA never
  do; they steer the eye with the world. Losing control mid-play breaks
  immersion. Take it seldom, briefly, and only for what play cannot show.
- **Say the hands are taken; let them out.** Hide the HUD, lock input,
  hold to skip; pause a cinematic with sound (under 3 s exempt). Bars
  over *gameplay* were patched out of The Evil Within: bars belong only
  to a takeover.
- **Comfort.** Turning the view for the player, changing the field of
  view and sudden accelerations are uncomfortable; keep the horizon
  level, ease every move, honour the shake and bob settings. No source
  gives a turn limit for a flat screen: ours are our own.
- **Death wears thin.** After the first death the fear is gone; the
  fifth replay is not scary; long death scenes were patched to skip.
  Short, shorter on a repeat, and never the threat where it was.
- **Test by time.** Sample at a fixed step; skip by evaluating the end,
  never by fast-forwarding; compare the hand-back pose and the state
  after a skip, as well as frames.

## The design

**One sentence:** a cutscene borrows your eyes, never your place: it
takes the camera from where you are looking, shows you something in the
world as it is, says plainly that your hands are taken, and gives the
camera back exactly where it found it.

1. **In the world, from your eyes.** A cutscene is drawn by the game's
   own renderer in the live level, in the place's own light and sound;
   there is no video and no separate scene. Its first and last pose are
   the gameplay camera's: it blends in from the camera as it is and
   blends out into the camera as play would draw it, so there is no cut
   unless a cutscene asks for one (a fade, or a key marked `cut`).
2. **Your hands are taken, and you are told.** While a cutscene holds
   the camera, black bars slide in from the top and bottom, the HUD at
   rest hides (the ring and its verb, the meters, the pockets), and keys
   and the mouse do nothing to your body. A key pressed shows the way
   out in the lower bar ("Hold E to skip"), the only words. Escape still
   pauses and Mute still mutes.
3. **Given back in place.** A cutscene never moves you: your position,
   facing, pitch, crouch and eye height are what they were, and its last
   frame is the gameplay camera exactly. A cutscene that ends anywhere
   else says so (`then = "card"`: the death card), and the hand-off is
   tested.
4. **Never into harm.** Nothing strikes you while your hands are taken.
   A cutscene in a live world breaks off (blends back in 0.35 s) when
   something starts hunting you or searching near you. A death ends any
   cutscene and plays its own.
5. **Time, not frames.** A cutscene has its own clock (time played: it
   stops with the pause), every key is at a time, the camera is a
   function of that time (the same at 30, 60 or 144 frames a second),
   each event runs once at its time even across a long frame, and the
   engine draws nothing from the random streams.
6. **Short, skippable, shorter on repeat.** Holding the skip key for
   0.6 s lands in the same state as watching to the end (every event's
   effect applied, its sounds not: the end evaluated, never the world
   fast-forwarded): a quick blend back if the camera is
   near your eyes, a dip through black if it is far. A death plays its
   short form after the first of its kind in a run.
7. **Comfortable.** Moves are eased, turns stay under 150°/s and roll
   under 5° unless a key is a declared jolt or a fall; the field of view
   moves at most 15°. The contract tests hold every cutscene to it.
   Settings' camera shake and head bob apply to a cutscene's sway, bob
   and shake as they do in play.
8. **Data, not code.** A cutscene is a file (`assets/src/cutscenes/ID.toml`)
   of tracks: camera keys in the frames of named anchors, fades, bars,
   sounds from places, events by name and asides; `cargo xtask cutscene`
   checks, compiles and previews it. Code only starts it
   (`play_cutscene(id, anchors)`) and registers the events it names.

### A cutscene file

```toml
id = "shaft"
about = "Looking into a broken lift's shaft"
world = "live"        # live: the world runs on; held: only its light and sound
then = "back"         # back: hands back in place; card: the death card
skip = "hold"         # hold | never
breaks_off = true     # a hunt breaks it off (live only)
length = 5.2
blend_in = 0.6        # from the camera as it is into the keys (default 0.5)
blend_out = 1.0       # from the keys back into the camera play draws (default 1.0)
repeat_from = 0.0     # the short form starts here (deaths)
stages = ["broken_elevator"]   # where tests and previews play it

[[camera]]            # x right, y up, z out of the anchor's face: -z is forward
t = 1.2
anchor = "thing"      # start | eyes | thing | killer
pos = [0.2, 1.5, 0.42]
look = [0.2, 0.6, -1.0]   # a point in the same frame (or look_at = "killer")
ease = "in_out"       # linear | in | out | in_out
fov = 64
sway = 0.3            # a held camera's sway (deterministic)

[[fade]]
t = 3.9
to = 0.0
over = 0.3

[[sound]]
t = 1.8
name = "breath"       # a registered sound, played from a place
anchor = "thing"
pos = [0.2, -4.0, -0.4]

[[event]]
t = 2.7
name = "shaft_eyes"   # a registered hook; run on a skip too

[[say]]
t = 5.2
use = "uncanny"       # an aside posted to your asides' director
subject = "shaft"
```

The anchors are fixed when the cutscene starts (`start`: the camera then;
`eyes`: your eyes with your facing; `thing`: the object it was started
from, its front as +z), except `killer`, a live point (the head of what
killed you). The camera is the keys through a centripetal Catmull-Rom
curve, its angles eased on the shortest arc, mixed with the gameplay
camera by the blend envelope.

## Milestones

Each is tested, committed and pushed on its own, with the production
build made; after the fifth and at the end, a code and design review
whose findings are fixed.

| # | Milestone | Done when |
|---|---|---|
| C0 | This plan; the research (`research/cutscenes.md`, `SOURCES.md` section 17); the queue in `MILESTONES.md` | committed |
| C1 | **Storage and pipeline**: the file's schema; `cargo xtask cutscene check` and `write` (TOML into `crates/game/assets/cutscenes/*.json` and `MANIFEST.json`); `game::cutscene`, its strict parse and the embedded library; the curves (easing, centripetal Catmull-Rom, shortest-arc angles) | xtask tests (one good file, one bad file a rule); game unit tests (round trip, an unknown key refused, a curve passes its keys, an ease ends flat); the structure test (compiled matches source) |
| C2 | **The player**: `play_cutscene`, its clock, anchors, the blend envelope over the gameplay camera (restored before the player's update, so its smoothing never reads a cutscene's pose), cues (sounds from places, events once, asides), hands taken, the live and held worlds, break-off, no strike, death ends it, pause, skip | tests with a cutscene made in code: the last frame is the gameplay camera bitwise; your pose unchanged; events once across a long frame; pause freezes it; skip lands where play lands; no random draw (a counter of native draws); nothing strikes |
| C3 | **The clue**: the bars (in 0.4 s, out 0.5 s), the HUD hidden, the skip line on a key press, its hold | HUD test frames (bars coming in, up, the skip line held), looked at; the goldens unchanged |
| C4 | **The test engine**: stages (`debug::stage`: the title, a broken lift, a death by each cause); the contract suite (`tests/cutscenes.rs`) over every cutscene and stage: hands, back in place, skip, pause, frame rates, comfort, draws, bars; `backrooms --cutscene ID --stage S --sheet DIR` (frames, a contact sheet, `timeline.tsv`), `cargo xtask cutscene preview ID`, `--cut-at T` for goldens | the suite passes a good test cutscene, and fails each broken one (one that does not come back, one that turns too fast, one that moves you): known answers |
| review A | Code and design review of C0–C4, fixed | committed with what it found |
| C5 | **The way in**: the title stands back on the walk behind where you begin; New descent keeps the title's place (its seed); the camera walks from the title into your eyes; the level card after | the suite; no jump between frames from the title to your hands; the run's seed is the title's; the title's golden re-accepted (why), the way in's goldens; your first arrival still said |
| C6 | **The shaft**: lean in at the gap, the black, a breath from far below, what looks back, the start back, your eyes; the toasts gone; its canon | the suite on a floor with one; once a lift; `lore-keeper`; goldens |
| C7 | **Deaths**: taken (in your face, you fall, it stands over you), the mind (your knees, the floor), the fog (under it); the short form on a repeat; the card at the end | the suite per cause; the death goldens re-accepted (why); menus, story and playthrough pass; the random draws after a death unchanged |
| C8 | **Skills and ledger**: `.claude/skills/add-cutscene` (followed end to end by a fresh agent), `.claude/rules/cutscenes.md`, pointers from `add-prop`, `add-creature`, `add-anomaly`; `AGENTS.md`, `CLAUDE.md`, `DEVELOPMENT.md`; `content.json`, `QUALITY.md`, `PLAYTEST.md` | the structure test; the fresh agent's run; the full check |
| review B | Code and design review of C5–C8, fixed; the full check; the bundle | committed with what it found |

## Progress

| # | Commit | Notes |
|---|---|---|
| C0 | (this commit) | the plan and the research |

## What only people can tell

- Do the bars read as "your hands are taken" without a word? Does anyone
  press a key during one, and do they find the skip?
- Does the way in feel like one shot from the title into your hands, or
  like a menu then a game?
- At the shaft: is what looks back frightening, or a jump scare? Would
  they look again?
- Does a death read as falling, and does the short form on a repeat keep
  death from wearing thin?
