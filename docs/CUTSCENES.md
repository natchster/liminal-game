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
4. **Never into harm.** Nothing strikes or touches you while your hands
   are taken (a creature's strike and contact, the Showroom's staff, the
   House's shadows), and nothing it tells you counts toward a strike. A
   cutscene that gives your hands back does not start in danger
   (something hunting you, something searching near, the fog up on the
   Building, your nerve near its end: `cutscene_danger`), and one in a
   live world breaks off at it (back in 0.35 s), what must hold of it
   holding as on a skip. A death ends any cutscene and plays its own.
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
blend_in = 0.6        # from the camera as it was into the keys (default 0.5)
blend_out = 1.0       # from the keys back into the camera play draws (default 1.0)
repeat_from = 0.0     # the short form starts here (deaths)
stages = ["broken_elevator"]   # where tests and previews play it
finally = []          # events run once when your hands come back, however they do

[[camera]]            # x right, y up, z out of the anchor's face: -z is forward
t = 1.2
anchor = "thing"      # start | eyes | thing | killer
pos = [0.2, 1.5, 0.42]
look = [0.2, 0.6, -1.0]   # a point in the same frame (or look_at = "killer")
ease = "in_out"       # linear | in | out | in_out
fov = -6              # degrees from play's field of view (as the player set it)
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
| C0 | `670a112` | the plan and the research |
| C1 | `dafaea8` | `game::cutscene`: `model` (the types; a strict parse of the compiled form: an unknown, missing or ill-kinded field is an error naming the cutscene and the field; `shape_problems`, the rules of a cutscene's shape: length, blend envelope, keys forward in time, the first key the camera as it is when nothing blends in, a stage to play it in, bars in order) and `curve` (cubic eases flat at their ends, a centripetal Catmull-Rom spline that passes its keys and never loops, angles the short way round, a sway made of sines). `cargo xtask cutscene check|write|list` (`xtask/src/cutscene.rs`): the format (fields, kinds, defaults written out) is xtask's, the rules are the game's, run on the compiled bundle by `backrooms --cutscene-check`, so there is one copy of them; `write` runs them on a copy first and never writes a bundle that breaks them. One bundle (`crates/game/assets/cutscenes.json`, embedded once: no list of files in code to keep in step), its `MANIFEST.json` entry (a new generator, "written by hand, compiled by scripts") checked by `tests/structure.rs`. A sample with every field (`crates/game/tests/data/cutscene/sample.toml`) is compiled by xtask's test into the fixture the game's tests parse, so the format and the parse cannot drift. Tests: xtask 3 (defaults, every fault named, the sample), game 9 (the bundle, the sample, refusals, every broken rule named, curves). The library is empty until C5 |
| C2 | `19b98eb` | `cutscene::play`: `play_cutscene(id, anchors)` (or `play_cutscene_data` for a cutscene made in code), refused while one plays, outside play, in the Stairwell or the Loop, or without a thing it names. Its clock is time played: advanced only in `cutscene_post`, which the pause does not run. The anchors are fixed at the start (the camera, your eyes, the thing as a frame whose +z is its front) but the killer (its head, live). The camera is the keys on the centripetal spline (a cut ends a run of keys; two keys looking at one live point follow it from where the camera is), eased per segment, with a sway of sines scaled by Settings' shake, mixed over the gameplay camera by the envelope (blend in from 0, out to the end; a repeat that starts part-way blends in over at least 0.2 s); at the end the weight is 0 and the camera is play's, bitwise. `cutscene_pre` puts the gameplay camera of the frame before back before play's update, because the player's update eases roll and field of view from the camera as it was: without it a twin run diverges at the first frame a cutscene rolls (the known answer: the test fails with it taken out). Hands taken: keys go to `cutscene_key` (Escape pauses, Mute mutes, the use key, Space or Enter held 0.6 s skips, any key shows the skip line), the mouse turns nothing, your body keeps its place, facing and crouch (`update_player` with `still`), the early E and the notebook are put away; `strike_allowed` and `creature_contact` refuse; your asides wait (`aside_held`: "cutscene"). A live world breaks off (0.35 s) when something hunts you or searches near; a held one runs only your still body, the cull, lights, HUD, dust, grain and sound. A skip runs every event left whose effect must hold (`on_skip`), not its sounds or asides, then blends back in 0.3 s if the camera is within 1 m and 35° of your eyes, or dips through black (0.2 s closing, 0.25 s opening on your eyes) if not. A death or a new floor ends one where it is. Its fades are a veil of its own (`veil_at`: a function of time); `then = "back"` must leave it clear (a new shape rule). `cues`: the sounds (played from their place by `play_from`) and events by name, and `lint` (part of the game's rules: names it lacks, a say the asides' director does not take, a killer outside a death). `rng::native_draws` counts the native stream. Tests: `tests/cutscenes.rs` (9: a twin run without the cutscene draws the same gameplay camera and the same random numbers every frame; body, mouse, cues once across a 1.6 s frame, pause, skip near and far and held too briefly, no strike, a hunt breaks it off, a held world holds the creatures, one at a time, a death ends it); game unit tests 12 |
| C3 | `8ab86b0` | `ui/cinema.rs`: the cutscene's veil (its fades, kept on the game: a death's black behind its card, a far skip opening on your eyes), then two black bars, each 11% of the view's height (a 16:10 view about 2.05:1, a 16:9 one near the cinema's 2.35:1), sliding in from outside the view over 0.4 s and out over 0.5 s; each bar is one canvas painted once for the view's size and slid by its rect, so a slide repaints nothing. The HUD (`paint_hud`) and the level card are not painted while your hands are taken. A key pressed shows "Hold E to skip" (the use key as Settings bind it) small at the right of the lower bar in the building's voice, 2.5 s; held, it brightens and a hairline under it fills (in thirtieths, a repaint a step). A first key that starts on the camera must have no sway (a new shape rule: a sway there would jolt the first frame). `--screen cutscene` and `cutscene-skip` (a glance at a thing two metres ahead, made in code) are the evidence, looked at: the bars, the HUD gone, the camera moving, and the way out half held. The thing anchor's frame was found facing away from you in the tests' `ahead` (its +z is its front, so a thing facing you has your yaw, not your yaw + π): fixed in the tests and the screen. `tests/cutscenes.rs` gains `the_bars_come_in_the_hud_goes_and_a_key_shows_the_way_out` (the bars' layers up, every HUD layer that was up gone, no words until a key, all of it back after) |
| C4 | `0639f6f` | `debug::cutscene`: a stage sets a game up as a cutscene meets it in play and starts it as play does (`cutscene_stage`, `cutscene_trigger`; for now `ahead`, a floor of the rooms with a thing two metres ahead, for cutscenes under construction and the known answers; each cutscene's own stages come with it); a run records it frame by frame (`run_cutscene`: the camera shown and play's, the blend, the bars, the veil, your hands and body, and the native draws the engine itself made, its cues' sounds and events counted apart, `cut_engine_draws`), skipping or pausing where asked, a callback seeing each frame (the preview renders there); the contract (`contract`: hands taken until the end and given back, the bars all the way in, your body never moved, the last frame play's camera bitwise, the bars and veil gone with the hands, the card if it ends on one, no draw, turns under 150°/s and moves under 4 m/s outside a declared jolt, the quick way back after a skip or a break-off and a cut under the veil, roll under 5° but in a death, the field of view within 15° of play's; `contract_skip`: a skip leaves what watching leaves; `contract_rates`: at 30, 60 and 120 frames a second, whose steps all land every 1/30 s, the same cues, the same end, the same camera at each shared time). The suite (`tests/cutscenes.rs`): every cutscene of the bundle on each of its stages, played, skipped at 20% and 60%, paused at half, and at the three rates (the bundle is empty until C5); and the known answers: a good glance keeps all of it; one that turns 170° in a second, rolls 20°, narrows the view to 30° or darts 4 m in half a second is named for it; a declared jolt lets the turn through; tampered runs (your body moved, the last frame off play's, a draw, the hands kept or given back early, a skip landing elsewhere, a camera drifting between rates) are each named. Its first run found the contract's own fault: a far skip's cut under black was flagged as a turn (the row after the end was stamped at the cutscene's length, not when it ended): a cut under the veil is not checked. `backrooms --cutscene ID [--stage S] [--bundle FILE] --sheet DIR [--every s]` writes its frames, `sheet.png` (four across, a quarter of the size), `timeline.tsv`, `contract.txt` and `log.txt`; `--screenshot F --cut-at T` its frame at T (for goldens). A preview never reads the save or the player's settings (it is headless to `make_game`). `cargo xtask cutscene preview ID [--stage S] [--every s]` compiles the sources to a copy, runs the game's rules, and previews each stage into `target/xtask/cutscene/ID/STAGE/`, failing on a broken contract. Looked at: a glance's sheet (the bars in, the camera to the thing and back, the bars out, the HUD back) |
| review A | `9e6440d` | A code review (this project's `code-reviewer`, by brief) and a design review of C0–C4, each by a fresh agent given the code; what they found, fixed and held by a test. **Cues ran with the cutscene taken out**: an event that ended it (a death, a new floor) or started another found nothing playing, and the old one was put back over it; now cues run with it in place, each cutscene numbered (`gen`), and a step ends when its number changes. **No end hook**: `finally`, events run once however your hands come back (played, skipped, broken off; the way in's level card), and a break-off now applies what must hold, as a skip does; a repeat runs what must hold of the part it skips, and its bars slide in from its own start. **The blend in mixed play's camera, not the camera as it was** (a title or a death would cut on its first frame): it blends from its start pose, whole (its roll and field of view too). **The veil stayed down** after a near skip or a break-off in a fade: it lifts with the camera now, and whatever is left opens after the hands come back. **A focus lost mid-hold skipped it**: the skip key is read from the keys held each frame. **Danger**: a predicate of its own (`cutscene_danger`: any hunt, a search within 25 m, the fog up, a nerve under 20; `holding_breath` gave only its first reason), refusing a start and breaking one off; the Showroom's staff and the House's shadows kill only when your hands are yours; tells told during one do not count. **The ear was split** (the listener at the camera facing your body; distance, pan and muffling from your body): while one plays, sounds are heard from the camera, facing as it faces. **A death through the engine**: the dead state runs `cutscene_pre`/`cutscene_post`, a cutscene that ends on the card opens it at its end (`open_death_card`, the 1.4 s schedule's body) and holds its camera and veil behind it, and a skip of one goes to its end. **Ending one early** (`cutscene_end_now`) gives play's camera back. **A key's `fov` is now an offset** from play's (the player's own setting kept); `skip = "never"` only for 3 s at most. **The contract could not fail on a real run** in three places (the frame it ended in recorded play's camera as the shown one, the bars as 0): that frame keeps what play drew (`cut_end`), and the contract gains a first frame that does not jump from the camera before it, a frame after the end that does not jump, a played run lasting its length, each `finally` once, and the camera never within 6 cm of a wall; the skip's signature leaves out the nerve (it moves with time); a live cutscene's suite run now sets something hunting at 40% and must break off. **The bundle could drift from its sources** past every gate: an xtask test compiles the sources and compares (`the_bundle_is_what_the_sources_compile_to`, in the fast check). `--cutscene` runs as `--fresh` (never the save). Tests: six new (`tests/cutscenes.rs`, 18 in all). Left open, on purpose: an actor track (a figure posed over time, its brain held; the shaft and the deaths are made with what is there, below), cue sounds' draws from the native stream (a skip plays none, so the stream after differs: a death's sounds stay in `on_death`), a held world's clock still moving the level's graces, a line shown during one timing out unseen, and how often a short form has played not kept in the save |
| C5 | `db665e9` | **The way in.** The title stands a few metres out along the way the floor runs furthest from where you begin (level, walkable, clear of walls and the void; on the Building a gallery with the void beside it), looking on down it (`Game::title_view`; `TITLE_BACK` 3.5 m, at least 2, else at your place as before). New descent (`cutscene::uses`, `new_descent`) keeps the title's place when it shows level 1 of a run nobody has played (`title_place`, set at boot): the run takes the title's seed, the floor built again is the one on the screen, under the camera as it was; it begins facing the way the title looks (its start pose, set before the cutscene: the cutscene never moves you), and `way_in` draws the camera back into your eyes (a held world: the place's light and sound go on). Over another run's floor (after leaving one), a new run is made behind black and the way in starts from its own title view. The level card waits for your hands (`finally = ["arrive"]`: on a skip too). Found on the way: the first title, standing behind where you begin and looking at it, looked into a corner on every seed tried (the arrival is the closed end of its floor, the way out ahead of it), and the run then began facing that corner (the spawn's facing is a rule of the rooms' grid); measured as a walk (clear floor), not a sight line (cell doors and parapets are not in the sight solids), the title now looks out, and you begin facing out. A cue set at a cutscene's very end could be missed (its clock, summed a step at a time, stops a hair short): its last step runs everything up to its length, and the contract now counts each cue once. The glide first peaked at 2.7 m/s (a spline's speed over an eased segment beside a near key): slowed to about a metre a second (3.5 s; at most 2.1 m/s). `walk` (a key's step bob by distance, Settings' head bob, faded at both ends) joins the format for the shaft. Stages `title:SEED`. Evidence: the contract on runs 4242, 7 and 9001's Wells (played, skipped twice, paused, 30/60/120 frames a second); `new_descent_draws_the_title_into_your_eyes_with_no_cut` (three boots: the title's seed kept, no frame stepping over 4 cm or 0.03 rad from the title to your hands and after, the card only after, your first arrival still to be said) and `new_descent_over_another_runs_floor_goes_through_black`; sheets looked at; a golden, `way_in_mid` (run 4242 at 2.6 s). The menus' death test, which chose New descent and looked for the level card half a second later, now waits for your hands (the card comes after the way in) |
| C6 | `1c92575` | **The shaft.** "Peer into the shaft" at a broken lift (`tail_broken_elevator`, about half the floors of the rooms) now plays `shaft` (`cutscene::uses`, `peer_into_shaft`): you step up to the gap between its jammed doors (`walk`) and lean in until the black fills your eyes; a cable scrapes high up, a breath comes from far below (sounds from places in the lift's frame, muffled through its wall); two points of eyeshine open low in the black (`shaft_eyes`: two small unlit points a hair in front of the black, 9 mm apart: seen from a hand's breadth, with nothing else in the black to measure by, they read as far down), and with a clank they are gone; you start back (a declared jolt), and your eyes are yours again. What it did holds on a skip or a break-off (`peered`: the lift's prompt gone, 12 of your nerve, the edges of your sight red); `finally` leaves nothing in the black and the red as your nerve has it. In danger it does not start, and the lift waits. The four toasts in capitals that told you what you had seen ("IT LOOKED BACK.", "YOU SHOULD NOT HAVE LOOKED.", …) and their random draw are gone. Found on the way: settling moves the lift's group onto the wall's real face after its `world` is recorded (a metre out on run 9001's level 7 and 4242's level 7), so its prompt and that position sit in its doors there: the shaft's frame is read from its lamp, a child of the group (`shaft_frame`), and the stage stands you a metre before the real doors (the record itself is left as built: no level fixture moves). The eyeshine first sat inside the black (a box 6 cm deep), and was moved out in front of it. `cut_override`: a test's or a preview's own version of a cutscene, played once in place of the library's, so the stage starts it through the real E. Stages `broken_elevator:SEED:LEVEL` (runs 4242's level 4, 7's level 9, 9001's level 7: three facings). The frame-rate contract now compares the blend at every shared time and the camera only where the cutscene's own is all there is (play's camera smooths by the frame: 0.05 mm apart at 30 and 60 frames a second at the hand-back). Evidence: the contract on the three lifts; `the_shaft_is_looked_into_once_and_leaves_nothing_behind` (its prompt, the look, something looking back and gone after, 12 of your nerve, none of the old lines, not offered again; in danger nothing and the lift waits); sheets looked at, the eyeshine at full size; a golden, `shaft_eyes` (run 4242's lift at 3.1 s); `LORE.md`: the broken lift, and a cutscene as a channel. The `lore-keeper` (by brief, a fresh agent) found: the points glowed on their own (canon: eyeshine never does; and "two points of eyeshine" is the Smiler's image), so they give back a light now, your lamp's or, with it off, the red lamp's (a faint red pair), and `LORE.md` lists what looks back as not a creature; the same look on half the floors broke "the same scare never repeats soon", so only the first lift a run looks back (later, the black and the breath); the lift's older linger scare told you in capitals what fell ("SOMETHING FELL. FAR DOWN.") and ran under the look: its line is gone and it waits while your hands are taken; the canon now says where the lift really stands (no level gate: the Lobby and the floors with rules of their own too), when it will not let you look, that it is built again after a death, and that the channel allows your asides. Left open (older than this work, recorded in `QUALITY.md`): a lift with blood and remains on the Lobby against its emptiness, and its scrape not played from a place. The core props' recorded session uses the lift's E: re-recorded (`regen-fixtures`): in the ten cases with a lift, its look no longer sets it peered at once (the cutscene does, 3.5 s in, and that harness plays none), its toast and the four native draws of the old look are gone, and the animators' wobble after it moved with the stream; nothing else. The linger scare's test now holds that no line says what fell. The full check passed (shots, range, bench) but for that test, then fixed |
| C7 | (this commit) | **Deaths.** `on_death` plays the cause's own, after posing what killed you as before: `death_taken` (a creature: its face in your eyes at once, a declared jolt; you start back; the floor comes up and the view rolls, still on it, standing over you; black from 0.75 s), `death_mind` (your nerve: your knees go, the view sinks and tips to the floor, black from 0.35 s), `death_fog` (the fog: you sink, lying back, into a violet near black). Each 1.4 s, and `then = "card"` opens the card at its end (`open_death_card`, its three draws as before) and holds the camera and the black behind it; on a repeat of its kind in a run each starts at its fall (`repeat_from`, the card sooner). None to play (a creature's death with no creature) falls back to the clock's fade and card. Nothing breaks off a death (`breaks_off = false`, and a new shape rule). The death camera's per-frame work splits off (`death_world`: the killer still moving, its three draws a frame kept, so nothing after a death moves); the shake is now a jitter of the cutscene's clock (`curve::jitter`) scaled as before and by Settings' shake, so a death looks the same at any frame rate (the contract found it random by the frame); the card's draws count as its own, not the engine's; a first frame that is a declared jolt is not a jump. A cutscene naming the killer is refused without one. Stages `death:KIND`, `death:sanity`, `death:fog`. Evidence: the contract on a Hound's, a Smiler's and a Reader's kill, your nerve, the fog; `each_death_plays_its_own_and_the_card_comes_as_always`, `a_second_death_of_a_kind_is_shorter`, `a_death_draws_what_a_death_always_drew` (the same native draws, frame by frame, as a death with no cutscene); the earlier tests that a death ends a cutscene and that a card cutscene holds its camera, now through `on_death`; sheets looked at; the six death goldens re-accepted (the same moment, the camera's framing and no HUD) and `death_fall` (the Hound's kill at 0.9 s, on the floor looking up at it) |

## What only people can tell

- Do the bars read as "your hands are taken" without a word? Does anyone
  press a key during one, and do they find the skip?
- Does the way in feel like one shot from the title into your hands, or
  like a menu then a game?
- At the shaft: is what looks back frightening, or a jump scare? Would
  they look again?
- Does a death read as falling, and does the short form on a repeat keep
  death from wearing thin?
