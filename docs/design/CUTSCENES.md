# Cutscenes: borrowing your eyes

How the game takes the camera for a moment and gives it back: the engine
that stores, plays and tests a cutscene, the clue that your hands are
taken, the pipeline that makes one, and the first three uses (the way in
from the title, looking into a broken lift's shaft, and dying). The
pillars it serves are in [`PILLARS.md`](PILLARS.md); the research behind
it is in [`../research/cutscenes.md`](../research/cutscenes.md), summed up in
[`SOURCES.md`](../research/SOURCES.md) section 17.

## The work

How this was planned and done (the analysis, milestones, progress, left open): [`../plan/CUTSCENES.md`](../plan/CUTSCENES.md).

## What the research says

Section 17 of [`SOURCES.md`](../research/SOURCES.md) has the sources; in short:

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

## The cutscenes

| Cutscene | Plays | Length | Stages | Golden | Serves |
|---|---|---|---|---|---|
| `way_in` | New descent from the title (`cutscene::uses`, `new_descent`) | 5.8 s, held, skippable | `title:4242`, `title:7`, `title:9001` | `way_in_mid` | `PILLARS.md` 1: the title is the place, and you come into it with no cut |
| `shaft` | "Peer into the shaft" at a broken lift, once a lift; what looks back once a run (`peer_into_shaft`) | 6.6 s, live, skippable, breaks off | `broken_elevator:4242:4`, `:7:9`, `:9001:7`, `:7:9:away` | `shaft_eyes` | `PILLARS.md` 5: dread, the black and a breath, then release; nothing explained |
| `death_taken` | a creature's kill (`on_death`, "entity") | 1.4 s, the card | `death:hound`, `death:smiler`, `death:reader`, `death:hound:wall` | `death_fall`, `<kind>_death` | `UI.md` 6: low-key death, the retry at once; shorter on a repeat |
| `death_mind` | your nerve gives out (`on_death`, "sanity") | 1.4 s, the card | `death:sanity` | | the same |
| `death_fog` | the fog takes you (`on_death`, "assembly") | 1.4 s, the card | `death:fog` | | the same |

## What only people can tell

- Do the bars read as "your hands are taken" without a word? Does anyone
  press a key during one, and do they find the skip?
- Does the way in feel like one shot from the title into your hands, or
  like a menu then a game?
- At the shaft: is what looks back frightening, or a jump scare? Would
  they look again?
- Does a death read as falling, and does the short form on a repeat keep
  death from wearing thin?
