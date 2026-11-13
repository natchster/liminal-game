# The things that live here: the creatures, redesigned

What the creatures of the rooms are, why, how each one behaves and is
beaten, and the plan that rebuilds them from scratch, body, behaviour and
sound, to the quality of the rest of the game. The canon it changes is
[`LORE.md`](LORE.md) (*Things that live here*); the pillars it serves are
[`PILLARS.md`](PILLARS.md) 4 and 5 above all; the research behind it is in
[`research/`](../research/README.md) (five notes, every claim linked) and
summed up in [`SOURCES.md`](../research/SOURCES.md) section 11.

## The analysis (2026-10-28)

What the creatures were, read from the code and the goldens:

1. **Three creatures roam the rooms** (levels 4 on; nothing walks the
   Building): the Smiler, the Hound and the Watcher
   (`entities/{mod,ai,anim}.rs`). One to three a level
   (`spawn_entities(1 + (depth − 1) / 2)`), the kind drawn at random, with
   the Watcher from depth 2 and the Hound from depth 3.
2. **One behaviour for all three**, with a few `if kind == …` exceptions:
   wander, search, hunt (strings), a view cone, hearing as a straight
   distance through walls, a 5 s memory. The Smiler lunges if you turn
   your back or run; the Hound stops for 2.8 s if you stare at it (12 s
   cooldown); the Watcher moves only unseen. **Three of the three turn on
   your gaze**, so the counters blur into one ("look at it").
3. **Touch is death**, at once, within 1.05 m. The Hound runs at 7.2 m/s
   and more, faster than your sprint (6.4): once one hunts you, only the
   stare saves you. Nothing postpones a death, and nothing checks that a
   death was warned.
4. **Their bodies** are procedural skinned meshes kept as stored in
   `creature_meshes.bin`, which no build here can make again (its scripts
   need rig dumps that nothing here writes; `QUALITY.md`: "the old creatures
   are kept as stored"). They read as models: a cartoon skeleton with a
   glowing grin, a hound you cannot make out, a plain sheet
   (`crates/backrooms/tests/golden/{smiler,hound,watcher}_*.png`), next to
   floors that are now photographs of real places.
5. **What proves the game beatable** runs in god mode with the creatures
   held still (`tests/playthrough.rs`). The personas die often and the
   pacing bot rarely, but no test plays the creatures' rules and finishes.

Measured against the brief ("more complex and well-defined behaviour, the
game beatable, the updated style, lore and quality"):

| What is asked | Where the creatures stand |
|---|---|
| Complex, well-defined behaviour | Three states as strings, numbers under string keys (`e.n("lungeT")`), one function of 400 lines; what a creature will do cannot be read back or tested except by replaying it |
| Distinct, learnable rules | Three gaze rules; hearing that passes through walls |
| Beatable, fair | Faster than you; touch kills at once; no proof that a rule-follower finishes |
| The updated style | Models, not things; glowing eyes (the art bible asks for eyeshine); no lore of the Building in them |
| Performance tracked | The bench times levels, not creatures |

## What the research says

Five notes, in [`research/`](../research/README.md). What they changed:

- **Every creature is one rule, learned by playing, with a tell before it
  bites** (SCP's rule-monsters; Containment Breach's code: lenient checks,
  strict rule; Lethal Company's one-puzzle-per-monster). Players accept a
  sudden death from a rule they understood and resent attrition they
  cannot read (`research/scp.md`).
- **Give each creature its own sense and its own counter**, so each is its
  own mental model (Inside the Backrooms gives its Hound and Smiler
  opposite gaze rules; Lethal Company; Botta's Infected in *The Last of
  Us*: one brain, kinds as data). **Hearing goes along the path, not
  through walls**; an "agitated" pose warns before a creature is sure;
  sight builds up over time in a volume; a creature remembers where you
  were and forgets visibly (`research/game_ai.md`).
- **Light as a repellent fails** (Naughty Dog cut such a creature: quick
  to react, it is harmless; slow, the light is useless); light works as a
  *sense* that costs you something. A blind creature that "sees" you
  confuses players.
- **Fairness is a set of invariants**: no death without two tells you
  could perceive; a grace period when a threat begins; a relief after a
  peak; pursuers slower than your sprint; the first catch postponed; the
  first meeting teaches (Witch, Jester, Nutcracker, Phasmophobia, L4D,
  Grip). They can be checked in code and by bots.
- **Fear has a sweet spot**: enjoyment follows an inverted U against fear,
  with small, frequent swings of arousal (Andersen, Clasen et al. 2020);
  uncertainty stresses more than the threat itself (de Berker 2016);
  repetition habituates, a pause and a new stimulus restore (Rankin 2009).
  So: rules learnable, *when* and *where* uncertain; behaviours unlock as
  you settle into habits (Alien: Isolation), never from your deaths.
- **How they are pictured** (`research/creepypasta.md`): an outline you
  could name as a black cut-out; height shown against a lintel, not in the
  frame; faces hidden, covered, turned away or *almost* right with one
  error; **eyeshine is a reflection of your lamp, never a glow** (people
  have none, so eyeshine in a human head is the whole horror); stillness,
  or motion that is wrong (stepped, reversed, head first); a sound before
  the body. The grin and "shadowed face plus realistic eyes" have turned
  comic (Jeff, Smile Dog, analog horror's tired tropes): our Smiler sat
  on both lines.
- **The Building's lore** (`research/backrooms_samosbor.md`): the
  samosbor is never seen, only heard; its monsters are "not worth
  cataloguing" and are told apart by *behaviour*; voices beg at the door
  and must not be answered; what the fog leaves is fused, wet, assembled.
  Soviet detail must stay documentary: "Soviet creepypasta" is a parody
  genre on the Russian internet (Mirvoda 2018), one joke-shaped detail
  and it tips over. The design takes the samosbor's lesson (tell them apart
  by behaviour) and leaves its beings on its floors, unexplained: nothing
  new walks or speaks there.
- **Bodies that were people should be people's bodies.** MakeHuman's base
  mesh, targets and skeletons are CC0 since September 2020
  (`research/modelling.md`; the licence checked in its repository): a real
  anatomy, starved by its own targets, made wrong by one specific error,
  on a surface that is real (MacDorman: proportion errors disturb more on
  a realistic surface).

## The design

### 1. Six creatures, six rules

Each creature finds you by a different sense and is beaten by a different
verb; each has a role (the research's roles: hunter, ambusher, stalker,
sentinel); each belongs to a kind of place, so the place warns you what may
be in it. Two of them are a pair on purpose: one must be watched, the other
must not be (opposite gaze rules read best, `research/backrooms-and-samosbor.md`).

| Creature | Role | It finds you by | You beat it by | Where it lives |
|---|---|---|---|---|
| **The Smiler** | ambusher | your **lamp** (from far off; your body only near, and only lit) | **darkness and calm**: lamp off, keep still, never run near it | dark regions, blackout zones |
| **The Hound** | hunter | **sound**, along the way it would walk | **moving when it moves**: freeze when it stops to listen; it cannot see | halls, mazes, offices |
| **The Watcher** | stalker | (it moves only when **unseen**) | **keeping it in sight**: look at it, light it, back away | furnished rooms, among the sheeted furniture |
| **The one in the corner** | sentinel | **your eyes on its face** | **not looking**: pass behind it, keep your lamp off its face | the far corners of quiet rooms |
| **The Reader** | hunter | **sight**, at middle range, in light | **paper**: drop a sheet and it must stop and read it | office wings, wherever paper lies |
| **The one behind the glass** | sentinel | **your eyes on its window's glass**, added up | **not staring out of windows**, or **covering the glass** (a blind let down); inside, **keeping out of its lamp's light** | far out on the track a window looks out on |

The Smiler, the Hound, the Watcher and the Reader are *sent* by the
director (§5), one at a time; the one in the corner is placed and never
sent, and comes only if you look at it. The sixth came with the windows
(`WINDOWS.md` 4): the one behind the glass, placed and never sent like the
one in the corner, which comes only for a long look out of its window. None of them walks the Building
(levels 1–3): there, what comes is the self-assembly, and what knocks at
the cell door stays unexplained.

#### The Smiler

*Backrooms Entity 3 (Wikidot, CC BY-SA 3.0: drawn to light; panic and
noise provoke it).*

- **What it is.** Someone very tall, starved down to tendon, gone black all
  over as if tarred. It is called the Smiler for its mouth: the lips have
  dried back off the teeth, top and bottom, so it seems to grin round to its
  cheeks (a rictus, not a smile). It stoops under the ceilings; its hands
  hang past its knees.
- **How it is seen.** In the dark, nothing, until your lamp finds it: two
  gold points of eyeshine at 1.9 m, set close, and under them the pale arc
  of bare teeth, only as far as the beam reaches. The rest is a black
  outline at the edge of the beam. Under the fluorescent panels it is a
  shape that is too tall and too still, not a face. Never a lit grin filling
  the frame.
- **Its sound.** A click of teeth when it shifts its weight; slow breathing
  close by; nothing when it waits.
- **The rule you learn.** *Your lamp calls it; the dark hides you; do not
  run near it.* Your lamp is seen from 25 m down any line of sight; your
  body only within 8 m, and only while you are lit. Lamp off and still, or
  crouched, it loses you within about 3 s: only sight (or bumping into you)
  tells it exactly where you are, so it goes on, slower, to *about* where
  your light went out, and strikes only if it finds you there. Its ears are
  a little duller than yours (a crouch at under 3 m, a walk at 5, a sprint
  at 10; a sound places you only roughly). It is slow in bright rooms (half
  its pace where the light map is over 0.55, where it stands or where you
  do: it comes into the light reluctantly), so walking out of a lit room
  away from it works. Running within 8 m of it makes it lunge. It is *not*
  driven off by your lamp: the lamp is what it hunts (the research: light
  as a repellent fails).
- **Modes.** Idle (in its dark, facing a corner) → Alert (it has seen a
  light: it turns to it, clicks) → Search (it goes to where the light was;
  arriving, it listens) → Hunt (you are lit and in its sight: it comes at
  4.5 m/s, faster than your walk) → Strike (you ran within 8 m, or let it
  reach you: a sharp inhale, 0.5 s, coiling as it comes, then a lunge at
  7 m/s over at most 4 m) → Withdraw.
- **What it learns** (§4): once you have hidden from it in the dark twice
  this run, it waits where you went dark, silent, no clicks, for 6 s
  before it searches: switching your lamp back on becomes the danger.
- Unlike the Borrowed House's shadows, whose rule is the *rooms'* light
  (dead rooms are safe), the Smiler's is *your* lamp, and calm.

#### The Hound

*Backrooms Entity 8 (Wikidot, CC BY-SA 3.0: a sickly person on all fours,
hair over the face). The stare-down of the old canon goes: the Hound is
blind.*

- **What it is.** An old person on all fours, starved, rump high, the head
  hanging so low you see only the back of the skull, thin hair matted over
  it; a vest and long johns gone the colour of the floor, torn at the knees;
  knuckles, knees and spine standing out; one leg drags. Close up: the
  eyelids have grown shut. No eyes in the dark.
- **Its sound.** Nails on carpet in a scrabble, then a pause (it listens);
  sniffing; panting when it runs; a hoarse bay when it is sure.
- **The rule you learn.** *Move when it moves; freeze when it listens.* It
  hears you along the way it would walk to you (walking 6.5 m, running 13 m,
  crouching 3.5 m, twice as far as anything else hears), through doorways,
  not through walls; its own scrabble covers your steps while it moves.
  When it hears something it stops in its listening crouch, head cocked,
  for 1.5 s (the tell): be silent then and it goes to the spot and sniffs
  about; move and it is sure. Close (1.5 m) it smells you after 2 s however
  still you are: back away slowly. Your lamp means nothing to it.
- **Modes.** Roam (a lope, stopping to listen every 4–8 s) → Alert (the
  listening crouch) → Search (to the sound at 3 m/s, sniffing) → Hunt (the
  bay, then a charge at 6.0 m/s, below your sprint, for at most 6 s, then it
  stops for 2–3 s to listen: go quiet then and it has lost you) → Strike (a
  crouch, 0.5 s, then it springs).
- **What it learns**: once you have outrun it twice, it stops baying
  when it charges and runs for where you were heading, not where you are:
  it comes at you from ahead.
- Unlike the Showroom's staff at night (be quiet, hide), the Hound's rule
  is a rhythm: its own noise covers yours.

#### The Watcher

*This game's own: the moving statue of the playground game
(Grandmother's Footsteps) and of SCP-173 (Moto42, CC BY-SA 3.0), under one
of the dust sheets that cover the furniture. The seated man of the Loop
shares the rule and the credit.*

- **What it is.** Someone standing under an old dust sheet, a head over
  you and stooping a little, the cloth hanging close about him to the
  floor, darker at the hem, a face pressed into it (brow, nose, and where
  the eyes and the open mouth are, the cloth grimed dark). It stands
  where furniture under sheets stands, and at a glance it is furniture. Each
  time you look back, it stands differently.
- **Its sound.** None (the canon). Close to it the hum dips and your ears
  ring.
- **The rule you learn.** *Keep it in sight.* It moves only while it is out
  of your sight: out of your view (lenient, as Containment Breach's is: in
  your view, about the screen, it is seen), not lit (the light where it
  stands over 0.35, your lamp on it, or within 4 m), or behind a wall; at
  your shoulder, behind your back, it is unseen. Only when the director has
  sent it does it move at all; unseen it comes at 4.4 m/s, faster than you
  walk and slower than you run; and it can be simply nearer when you turn
  round (never within 10 m, never where you could see). Its touch takes
  your nerve (−45) and knocks you down (two tells first: your ears ring
  within 8 m, and seeing it nearer than you left it; and never in the 2 s
  after you look away), and it falls back a few steps; it kills only a mind
  that is already gone.
- **Modes.** Idle (not sent: it stands where it is) → Hold (seen) ↔ Stalk
  (unseen, on its way) / Hunt (unseen, within 6 m: the director's peak) →
  Strike (touch) → Withdraw.
- **Its company.** It stands among furniture under the same old sheets (a
  chair, a table, a wingback: `sheeted.py`, the same cloth and linen), two or
  three round it, so that at a glance it is one of them.
- **What it learns**: once you have held it with long stares (over 8 s)
  three times, the panels over it begin to fail while you watch it. It never
  moves while seen; it takes the light you see it by.

#### The one in the corner

*New. After SCP-096 (Dr Dan, CC BY-SA 3.0: seeing its face is the
trigger), the Rake's figure with its back turned (anonymous, 2005; the
motif only), and the child made to stand in the corner. Its face is the
Thatcher illusion: eyes and mouth upside down in an upright face.*

- **What it is.** Someone tall standing in the far corner of a quiet room,
  forehead to the angle of the walls, as a child is made to stand; in a
  long flannel nightshirt, barefoot, its arms straight at its sides. You see
  its back. If you see its face, it is a person's face with the eyes and the
  mouth turned upside down, and you see it only once, as it turns.
- **Its sound.** It hums, low and tuneless, from a room away; the humming
  stops when it is about to turn.
- **The rule you learn.** *Do not look at its face.* It is harmless while
  you see only its back. Come in front of it (within its front half, its
  face in your view, a line between you, the face lit) and it turns its head
  toward you over 1.5 s (the hum stops: the tell, your chance to look away);
  hold the look and it turns fully, screams, and comes at 5.6 m/s (below
  your sprint), knowing where you are for 6 s; out of its sight for 6 s
  after that, it goes back to its corner. Pass behind it; keep your lamp off
  its face; leave the room by another way.
- **It looks round** now and then on its own while you are in its room
  (within 20 m and its sight, every 25–45 s): the hum stops, its head comes
  round over 1.5 s, and if its face comes into your view, lit, it is the
  same as if you had looked. (Without this its face, to the angle of the
  walls, could be seen only by squeezing into the corner to peek; what it
  learns, below, makes it look round more.)
- **Modes.** Hold (in its corner) → Alert (turning, humming stopped) → Hunt
  → Strike → Withdraw (back to its corner) → Hold.
- **What it learns**: once you have passed behind it twice this run,
  it turns round to look at the room behind it far more often (every
  10–18 s, the hum stopping first, against 25–45 s: since E5 it looks round
  now and then anyway, or its rule could never be met).
- Placed after the level's build (with her camp fitted, no draws) in the
  quietest inside corner off the main way (two tiles or more from the walk
  from the arrival to the door; 8 tiles from the arrival, 5 from the door
  and her camp), at most one a level, from depth 2, on about half the
  levels (chosen by the level's seed).

#### The Reader

*New. After the upyr of East Slavic folklore, which must stop to pick up
scattered seed, and the slit-mouthed woman of Japanese legend, who must
stop for candy thrown down (both public domain): a compulsion you can use.*

- **What it is.** A thin man in a brown cardigan buttoned wrong, paper
  stuffed in its pockets and down its front, ink on its fingers, the fingers
  too long; it walks bent over a sheet held close to its face, its eyes going
  left to right, left to right. It looks up when it sees you.
- **Its sound.** A whisper, fast and continuous, reading under its breath;
  a sheet snapped flat.
- **The rule you learn.** *Give it paper.* It sees you at middle range (12
  m, and only where you are lit or moving) and comes at 4.8 m/s. **Any sheet
  of paper on the floor within 4 m of it stops it**: it must pick it up and
  read it to the end (3 s). Drop a sheet behind you and walk on. It reads
  the sheets you drop (and keeps them: a trail of them through its halls
  does not last) and the piles lying about (the top sheet, once each); it
  never touches her folded sheets nor the flat ones (they are the same on
  every visit; `paper_near` reads only `paper_markers` and `papers`).
- **Modes.** Roam (reading as it walks) → Alert (it looks up: the whisper
  stops) → Hunt (it snaps the sheet flat; its quick soft steps behind you
  are a tell every 2 s within 9 m) → Read (the compulsion: `Mode::Read`,
  blind to you) → Strike (it lowers the sheet, 0.8 s, coming on as it does:
  a walk does not outpace it, a run does) → Withdraw.
- **Placed** (`reader::place`, after the build, no draws) when the ecology
  calls for it (paper lying about, design 5), beside a pile (an office
  wing preferred), 18 tiles or more from the arrival and 5 from her
  camp; sent by the director like the Smiler and the Hound. What it
  chooses while it reads far from you draws on a stream of its own (its
  home and a count), so it moves nothing on the per-frame stream.
- **What it learns**: once you have escaped it with paper twice, it
  tears the sheets in half unread (1 s) instead of reading them: you hear the
  tearing, and paper buys you less.

#### The one behind the glass

*New with the windows (`WINDOWS.md` 4). After SCP-2885 (a view that
sharpens the longer you look), SCP-3052 (a figure that faces you from every
angle), SCP-L682 (through glass like plastic wrap) and the Backrooms'
Windows (Entity 2: a reach from the window, and covering as the counter),
all CC BY-SA 3.0; Kunekune (the danger in looking closely, a motif); and,
public domain, Gogol's* Viy *(a line it cannot cross) and* Wuthering
Heights *(a window that will not open).*

- **What it is.** A figure in a pale raincoat, belted, the hood up and drawn
  round its face, standing far out on the track a window looks out on
  (32 m: in the fog there, a smudge you are not sure of). It always faces
  you. Close, its face pressed to the glass is cold, wet and drowned-pale,
  its eyes clouded, the hood's rim on the pane, its hands flat on it.
- **Its sound.** Glass, and water: the pane's tick (a click and its short
  ring, low and inharmonic: not the Smiler's click), the creak of the pane
  under it, wet skin squeaking on the pane, the long groan of the glass
  bowing in; inside, its wet raincoat rustling (the place's own drips were
  too near). No voice; its breath is seen on the pane, never heard (four
  of the others breathe). Heard only in its room: at its glass within 9 m,
  and inside within 9 m of it with nothing between.
- **The rule you learn.** *Do not stare out of a window too long.* What
  counts is your eyes on its glass: your view's line through any light of
  its window's row within 8 m (it is seen through all of them), nothing
  between, that light uncovered. It adds
  up across glances (a second a second, draining a quarter as fast when you
  look away). At 3 s the pane ticks: it has seen you (a tell). At 5 s it
  comes up the track at 2.6 m/s while your eyes are on its glass and stands
  when they are not; it never goes back. Within 7 m of the glass (its
  lamp's light on its own) it needs your look no longer, a cover or the
  quiet only holding it (`coming`): it comes to the glass (the
  creak, its breath misting the pane, the squeak: tells), and 2 s later,
  unless its glass is covered (it stands where it was) or the place lies
  quiet after a close call (it waits at the glass), the pane bows
  in round it like film and it comes through (2.4 s) and down onto the floor,
  into its lamp's light.
- **Its reach is the light.** Inside it can be only where its lamp's light
  falls through its window's row (`Window::lit_at`): it goes to the edge of
  the light nearest you, at 1.8 m/s, and stands there facing you, its coat
  rustling (a tell every 2 s). It backs off while the place lies quiet
  (as they all do), and when it has seen you keep out of its light 12 s
  (`KEPT_OUT`). In its light, within its reach, it takes you
  (`touch_nerve` 40: it kills only a mind already gone, spent before it
  touches, and never at a first meeting) and you are thrown down out of the light. Out of its sight 4 s
  (or 12 m off), it goes back out through the glass (2 s) and stands in the
  lamp's light 1.4 m out; a long look then brings it straight back.
- **Covered** (`core_props::window::blind`): a blind let all the way down
  (`[E] LOWER THE BLIND`, 1 s, its clatter heard 10 m along the way by
  what hears, low and dull, well under the pane's tick) or paper taped over
  a light covers its glass: no look counts through it, its light throws
  none (`lit_at`, `lit_cells`), and it does not come through it; inside
  when its row's light is all covered, it goes back out ("its light is
  gone"). `[E] RAISE THE BLIND` draws it up, as loud. A row whose every
  light is covered goes quiet (its lamp's ballast not heard).
- **What it learns**: got away from twice out of its light (`creature_habit`
  where it goes back out unseen), the lamps outside the windows hang 1 m
  lower (`view::set_lamp_drop`, every window's of the floor, the light
  thrown again: its window stays unmarked): its light reaches 4.3 m into
  the room for 2.6 m. The first time they come down where you can see it
  (in its window's room, near: the light creeping in over 3 s; its own
  check, as the shared one sees nothing through a wall); after that on
  every floor with windows from the first (fitted after the build).
- **Modes.** Hold (out on the track) → Alert (it has seen you; or waiting:
  at a covered pane, or at the glass while the place lies quiet) → Stalk
  (coming while you look) → Hunt (to the glass, through, in its light) →
  Strike → Withdraw (back out) → Alert. To the director only Hunt and
  Strike are a hunt and Stalk a search while it comes; standing out there,
  or waiting, it is nothing (a hunt waiting behind a blind had held the
  floor in a peak with no end)
  (`glass::legacy_state`), and nothing of it reaches the room while it is
  out there (no pressure through its wall, no sighting through its glass).
- **Placed** (`glass::place`, after the build, no draws) behind one window
  of about four in ten of the floors with windows (a coin from the level's
  seed), never where an anomaly is: its window's lights uncovered and
  lighting the floor, the first open tile before it off the main way (two
  tiles from the walk from the arrival to the door), 8 tiles from the
  arrival, 5 from the door and her camp; the furthest from the arrival. Its
  window's panes are made a fine grid then, so they can bow.

### 2. One brain

Every creature runs the same brain (`entities/brain.rs`); a kind is data
(its senses, speeds, tells) plus its own mode rules, never `if kind ==`
scattered through the shared code (after Botta's Infected).

- **Modes** (`Mode`): Idle, Roam, Alert, Search, Stalk, Hunt, Strike,
  Recoil, Withdraw, Hold, Read (made to stop by something it must do: the
  Reader's paper). A creature is always in exactly one; it leaves it
  only by a rule in its kind's table, and every change is logged with its
  reason (`Brain::log`), so what it did can be read back and tested.
- **Percepts**, computed once a frame from the world, never from the
  creature's wishes: distance; a line between its eyes and yours; whether
  you are in its view; how well it sees you (0–1: distance, the light on
  you, your lamp, crouching); how loud you are where it stands (0–1, along
  the path, through doorways); whether you are looking at it; whether your
  lamp is on it; the light where each of you stands.
- **Awareness** (0–1), filled by the percepts at each kind's own rates and
  drained when nothing feeds it: from 0.3 it stops and turns to what it
  noticed (the *tell*); at 1 it is sure. Unaware, its senses are dulled
  (×0.6); in the director's relief, ×0.55. **Memory**: where and when it
  last saw you, last heard you; for 2.5 s after losing you it still "knows"
  where you went, then it searches: where it last had you first, then the
  places that view hides, in an unhurried order, and gives up after 15–25
  s by walking away, not by snapping back.
- **Orders**: the director does not move creatures; it tells them (stalk
  toward the player's part of the level unseen; seek a point; keep away for
  so long).

### 3. Fairness, as invariants

Checked in code (`entities::brain`, the strike gate) and by the tests:

1. **Two tells before a strike.** A creature may strike only if it has given
   at least two tells you could perceive (heard within its sound's reach,
   or in your view) in the last 20 s (its notice is one).
2. **The first meeting teaches.** The first strike of each kind in a run
   does not kill: it knocks you down, takes your nerve (never the last of
   it: a contact that is not the end leaves you `KNOCKED_LEAVES`, and a
   knock-down kills only when its own blow emptied you), and the creature
   withdraws for a full relief.
3. **Death is postponed.** A strike kills on the second contact in the same
   peak, or on the first when your nerve is below 25.
4. **You can always get away.** A sustained chase is at most 0.95 of your
   sprint (6.1 m/s; of your sprint as it is, when what you took has made
   you smaller and slower: `chase_cap`, `docs/design/SECRETS.md` 5); a burst (the
   Smiler's lunge, the Hound's spring) is at most 7 m/s over at most 4 m.
5. **Grace.** No death in a level's first 30 s, and none within 2 s of a
   creature's first notice of you.
6. **Nothing is born where you are.** No creature spawns within sight or 24
   m of the arrival, her camp or the door; none relocates where you could
   see it.

What you took (`docs/design/SECRETS.md` 5, `LORE.md` *Things that change you*)
moves a few of the numbers below a step for the rest of the run (how far
your steps carry, how far a Smiler sees your lamp, how long a Hound takes
to smell you standing still, how fast your look out of a window adds up,
what a blow takes), and never what these invariants rest on: no thing
lowers what the dark leaves you or raises what a touch takes, and a chase
stays under your sprint.

### 4. What they learn (the habit ledger)

A run keeps a count of your habits: hiding in the dark from a Smiler,
outrunning a Hound, long stares at a Watcher, slipping behind the one in
the corner, paper dropped for a Reader, getting away from the one behind
the glass out of its light. Each kind unlocks **one new
behaviour** (§1) when its count is reached, never from deaths, at most one
unlock a level; the first time it happens it happens in plain sight.
Learning you can see, not a stat that creeps up (Alien: Isolation; Hello
Neighbor's opacity is the warning). No learned behaviour breaks its
creature's rule.

As built (E7, `creature::LEARNS`; the counts as E8 set them): twice hidden
from a Smiler in the dark (it loses you with your lamp out), twice outrun
by a Hound (it loses you 15 m or more off), three long stares at a sent
Watcher (over 8 s), two passes behind the one in the corner (within 3 m of
its back), two sheets dropped for a Reader that was after you; and, with
the windows (`WINDOWS.md` 4), twice got away from the one behind the glass
out of its light (the lamps outside the windows hang lower). (E7's
counts, 4, 3, 5, 4 and 3, were never reached: in twelve bot runs of ten
levels no kind learned, and the most any habit came to was 2. Whether a
kind now learns too soon is for people.) A kind that reaches its
count on a level where another learned learns as the next level starts
(`creature_learn_pending`); the first use waits until you are in its line
within 20 m (`creature_may_show`), and from then on it is simply what it
does. The ledger (`RunMemory`: who you have met, your habits, who learned,
what you have seen) is saved with the run.

### 5. The director

It keeps its phases (calm, build, peak, relax) and its menace gauge, and
gives orders instead of setting fields. It **sends one creature at a
time**: the Smiler, the Hound, the Watcher or the Reader (not the one sent
last, the nearest of the others); some build-ups end in nothing (a sign,
then quiet: 20–40% of them), so a sign is never a promise. The one in the
corner is never sent. As built (E7): 30% of build-ups are empty (one sign,
six seconds of quiet, and it goes its way), never one her page or a false
trail called; a stalker sent from further than 60 m along the level is
first moved, out of your sight, to 30–45 m (it had been sent from 190 m
and never arrived); sent away after a close call, every kind withdraws as
soon as it no longer has you (a chase stays its own until its rule loses
you), and one sent away, finishing its chase, starts no new peak.
**The ecology** (`Game::ecology`) decides who is on a level, by depth
and by the places in it:

| Depth (level) | Creatures |
|---|---|
| 1 (4, the Lobby) | one: a Hound, or a Smiler where the Lobby has a dark region |
| 2 (5) | two, of Smiler, Hound, Watcher (the Watcher only with furnished rooms), the Reader (only with office wings); the one in the corner in a quiet room |
| 3+ | three of the four sent kinds, by the places; the one in the corner |

As built: the places are those a creature can use (reached from the
arrival and a quarter of the level or more from it): a dark region (12
tiles under 0.2 of light) calls the Smiler, furnished rooms (8 tiles of
living quarters or parlour) the Watcher, paper lying about the Reader
(office wings the more), and the Hound goes anywhere; each kind once while
the places allow, then another of what goes anywhere; chosen by the level,
not by chance. Each is put in its own kind of place (the Smiler in the
dark, the Watcher among furniture, the Reader beside the paper). The
anomalies keep their own residents (the Showroom, the Borrowed House; E9
gives them new bodies); a level with the Stairwell's or the Loop's door is
rooms like any.

### 6. Bodies, motion, sound

- **Bodies** are built by `cargo xtask assets creatures` (Blender headless,
  `assets/src/creatures/`) into `crates/game/assets/bodies.bin`: each a
  skeleton and its meshes, skinned by Blender's bone heat. The ones that
  were people start from MakeHuman's CC0 body (fetched and checked by its
  SHA-1, starved by its own targets), posed so the pose is the rest pose,
  made wrong by one specific error each, and given their surface (ribs,
  tendons, the skin's wear) as displacement and vertex colour. Budgets: 18k
  triangles a body, 25k a creature.
- **Motion**: procedural, on the rig (`entities::body`: a bone aimed in the
  body's frame; two-bone limbs). Two vocabularies only (the research):
  **stillness** (the Watcher; the one in the corner; the Smiler waiting) and
  **wrong mechanics** (stepped, held frames and snaps; the head first and
  the body after; the Hound's drag). Upper faces do not move: they do not
  flinch when you cry out.
- **Eyes**: eyeshine is your lamp reflected, not a glow: its brightness is
  how much of your beam falls on the eye and how squarely it faces you; only
  the Smiler has it (gold). The Hound's eyes are shut; the Watcher's are
  under the cloth; the one in the corner's and the Reader's are a person's.
- **Sound**: a signature each, from where it is, heard before it is seen
  (§1), none of them a sound the rooms already make; the stings and the
  drone stay the director's.
- **The anomalies' figures** (as built, E9) keep their own rules. The
  Showroom's staff and the Borrowed House's shadows walk on the procedural
  human rig (`anomalies::rig`), so a body for them is MakeHuman's fitted to
  that rig's rest pose (`assets/src/creatures/figure.py`: each bone turned
  and stretched along the rig's, the trunk evenly, the head its own length
  with its crown on the rig's, then every joint moved onto the rig's,
  blended along the bones) and bound to the rig's bones by name
  (`rig::bind_figure`, a skinned mesh a part); `rigs.json`, written by
  `tests/anomaly_bodies.rs`, is that rest pose, and the body is held to it.
  The faces are taken away by putting each point of the face at the
  distance of the head round it, seen from the skull's middle (on
  MakeHuman's own skin, before the remesh closes its eyes' holes into
  sockets that overhang). The Loop's seated man is never posed: his body
  (`body_seated.py`) sits on the Loop's own chair, drawn with it as one
  instanced piece. Nor is the Building's figure across the void (not a
  creature; `docs/design/MEGASTRUCTURE.md` G7): its body (`body_far_figure.py`, a
  man in a long grey work coat standing still) is drawn as one piece and
  turned whole (`world::far_figures`). Only what a blind review preferred ships (`cargo xtask
  ab`, both orders, a fresh reviewer a round): the seated man won every
  round; the staff lost four (sticks for arms, spikes for hands, a flat
  egg for a face, a smock, a mock neck) and won the fifth with two
  reviewers; the shadow never did better than a tie, and the shadows keep
  their old body (`creature_meshes.bin`).

### 7. Beatable, and shown to be

- **Arenas**: for each kind, a small room and the creature, 100 seeds with
  its counter and 100 without: with it a bot survives at least 95% of the
  time; without, 30–70% (the counter matters; the creature is not a
  death sentence).
- **A rule-follower** (a persona that knows every counter) plays levels
  4–13 of several runs with the creatures live and deaths on: it finishes
  at least 90% of levels; the explorer and the rusher less, but some.
- **Hard fails**: a death with fewer than two perceivable tells before it;
  a death at a first meeting; a death in a level's first 30 s; a chase that
  ran faster than the cap.
- **The whole run** from New Game to past level 13, the rule-follower
  playing with the creatures live, ends: the game is beatable as shipped,
  not only in god mode.
- **The dark zone** (found on the way; the arenas do not model a level's
  zones): in a dark zone your nerve drained at 6.5 a second with your lamp
  off, to nothing, and lamp off is the Smiler's counter; on a spent nerve
  any contact kills. E8 measured it on levels before changing anything:
  the rule-follower spent 1006 s in a dark zone on a spent nerve, 997 of
  them with its battery dead and 8 with its lamp out for a Smiler, and all
  7 of the explorer's deaths came on a nerve the dark had spent. Hiding
  from the Smiler was not the trap; the lamp's battery was (it lasts 25 s
  lit, `BAT_DRAIN`). So the dark now takes your nerve down to 40 and no
  further, and gives it back up to 40 (`player::DARK_FLOOR`): a spent
  nerve is the creatures' doing alone, and it passes, even in the dark.
  (At 30, the first floor tried, the dark and one sign of a stalker, up to
  12, still spent a nerve no creature had touched, and the rule-follower
  died of the Smiler's next touch.)

As built (E8):

- **Arenas** (`tests/creatures_arenas.rs`): the bands above, held as
  untouched and killed (with the counter ≥ 95% untouched and ≥ 90% round;
  without ≤ 70% untouched, ≤ 60% killed). And once it has learned
  (`what_each_learns_leaves_its_counter_working`: met before, learned and
  already shown), the counter must work as well as before (within 5
  points) and the arena must show it doing what it learned in most trials
  (`learned_uses`: the mark each new behaviour leaves). The Smiler waits
  in 83% of trials, the Watcher's lights fail in all, the one in the corner
  looks round in all, the Reader tears in 97%: each 99–100% untouched, as
  before. The Hound learns in its charge, which its counter seldom lets
  happen (it charged in 1 trial of 100), so its arena has you heard first
  (walking on, minding nothing, until it comes): 59% untouched learned,
  60% before. That is the Hound's rule, not what it learned: sure of you
  at 6 m, its charge (6.0 m/s against your 6.4) is outrun only to a place
  you reach well before it; the counter is not to be heard. What the
  learned arenas found in the bots: one that heard the hum stop and walked
  out of its reach turned to look a step later as its head came round (a
  player keeps their eyes away a while: 3 s); one that sidestepped a
  Hound's charge was run down.
- **The rule-follower** (`tests/rule_follower/mod.rs`, shared): what a
  player who knows every counter does about the creatures in reach
  (30 m), from the mode each one is in, which stands for the sound or
  sight that gives it away (an upper bound on a player who reads every
  sign): the Smiler, lamp out once it stirs, crouched still while it is
  after you within 14 m, and crept away from within 7 m (its breath: sent,
  it creeps on, and the first bot walked on into it); the Hound, still while it listens and running
  from a charge within 8 m; the Watcher, faced once its ring is in reach
  or it is seen after you; the one in the corner, eyes away when it turns,
  running if it comes; the Reader, a sheet dropped behind you every 2.5 s
  while it is after you within 12 m, running if out of paper. It keeps its
  lamp lit in the dark and out under the panels, puts a spare battery in
  under 10 and picks one up under 40 when it has no spare, and with a
  Reader on the level fills its pockets from a pile on the way (the
  nearest, kept to until taken or given up after 30 s: taking the first
  in reach, it went to and fro between two). It moves by the keys, as a
  player does, toward the farthest of the next metre's points it can walk
  to straight (aimed past a corner, a body pushed into it moves on neither
  axis: it had stood 400 s so).
- **Beatable** (`tests/beatable.rs`, a test per bot): levels 4–13 of four
  runs (the Showroom and the Borrowed House keep their residents' rules
  until E9; the Building's floors have no creatures), 480 s a level,
  deaths on, the place woken by her page; a run's levels in order, what
  it remembers of the creatures carried on (who you have met: a first
  meeting never kills, a later one may; your habits): the rule-follower
  finished all 32 levels with no death (18 peaks, 11 contacts, each a
  knock-down); the explorer 31 (one death, a Watcher's; 41 peaks, 36
  contacts), the rusher all 32 (15 peaks, 14 contacts); no death broke a
  hard rule. Those who know no counter finish nearly as often: a contact
  mostly knocks you down (the first of a kind in a run never kills, and
  the creature backs off after one), so 36 contacts cost the explorer one
  death; whether being caught costs enough is for people. Time in the dark
  on a spent nerve: 65 s for the rule-follower, all with a dead battery
  (1006 s before the floor). At E7's learning counts no kind learned in
  any run; at E8's the explorer's Smiler learned in two runs of four (the
  rule-follower, which seldom lets a creature hunt it, taught nothing).
  The first sweeps failed on the bot, never on a creature: it aimed past
  corners and stood 400 s, went to and fro between two batteries, and
  walked on into a Smiler's breath (each fixed as a player plays).
  A level's frames had depended on what its thread played before (what is
  made once a thread, the creatures' cloth or any canvas texture, drew on
  the per-frame stream the first time only); since E10 every texture paints
  from a stream of its own (`canvas_tex`) and they do not. `one_level`
  still replays the sweep's order: the run's memory of the creatures, carried
  from level to level, is part of what a level meets.
- **The whole run live** (`the_whole_run_live`, `tests/playthrough.rs`):
  New Game to level 14 with the rule-follower following her story on the
  rooms' own floors (her camp, her page, the door; a death is woken from
  and the level played again), the Building's floors, the Showroom, the
  Borrowed House, the Stairwell and the Loop as before: three runs (101,
  202, 303) end, 42 levels; two deaths on the way (run 101: a Smiler on
  level 8, a Hound on level 14; each level played again and passed); the
  creatures came on 9 of the 29 live levels (11 peaks, 11 contacts). This
  bot walks the playthrough's way (no battery kept, no paper taken), so it
  is weaker than the sweep's. A player who
  goes straight to her camp and the door (under two and a half minutes a
  level) mostly leaves before the first build-up peaks (it may take 150
  s): a question for pacing and for people, not for whether it can be
  beaten.

### 8. Performance

`cargo xtask bench` times the **creature pack** (two of every kind that
roams, hunting you at once, level 4) beside the levels, held to the
`creatures` budgets in [`PERF.md`](../engineering/PERF.md) and to its own history
(`perf/creatures.csv`). The first row is the old creatures (5 placed:
update 0.07 ms p50, GPU 2.2 ms p50, 263k triangles). Every milestone
records a run.

### 9. The canon each creature brings

What `LORE.md` must record as each lands (the E0 `lore-keeper` review):

- **The Hound** (E2): blind; the eyelids grown shut, no eyes in the dark;
  a bay, not a bark; hears along the way it would walk; the listening
  crouch; smells you close by. Torn note 10 ("stare it down") rewritten as
  a hint of the new rule.
- **The Smiler** (E3): eyeshine is your lamp's, gold; the rictus; slow in
  bright rooms; running near it. Torn note 9 ("hold its eyes") rewritten;
  her fifth page re-read against it.
- **The Watcher** (E4): still silent; sheeted furniture; its touch knocks
  you down; SCP-173's credit beside the seated man's.
- **The one in the corner** (E5): its rule, its hum, its face; SCP-096's
  credit; placed, never sent.
- **The Reader** (E6): its compulsion; paper under *Survival*; it never
  takes her sheets nor the flat ones, and who lays those is still never
  said; the upyr.
- **The director** (E7): one is sent at a time; some build-ups end in
  nothing; the first touch is not lethal and death is postponed; what they
  learn.
- **The anomalies' figures** (E9): the seated man's new body carries
  nothing of E. (his face stays unsaid).

## Milestones

| | What | Evidence |
|---|---|---|
| E0 | This plan; the research (`docs/research/`, `SOURCES.md` section 11) | committed |
| E1 | **The ground**: the creature pipeline (MakeHuman CC0, the kit, `cargo xtask assets creatures`, `bodies.bin`), rigs and posing; the brain's core; the creature bench | tests: every body decodes soundly, the posing helpers; the pack bench and its history; no level, golden or fixture moves |
| E2 | **The Hound** on the new brain and body; hearing along paths; the strike gate and the first meeting (design 3); its canon and note (design 9) | arena (with and without the counter); goldens at 2, 4, 8 m and its death; a listen PNG; pacing and personas pass |
| E3 | **The Smiler**; eyeshine | the same |
| E4 | **The Watcher**; the sheet and its poses; sheeted furniture | the same |
| E5 | **The one in the corner**; placed in the build | the same; the level sweep (every level passable) |
| E6 | **The Reader**; paper as a counter | the same; the story tests (her sheets untouched) |
| E7 | **The director and the place**: orders, the ecology, the habit ledger and the unlocks; the old creature code gone | pacing, personas, story; reference levels re-recorded with reasons |
| E8 | **Beatable**: the rule-follower, the arenas, the hard fails, the whole run live | numbers in their bands (design 7) |
| E9 | **The anomalies' figures** (the Showroom's staff, the Borrowed House's shadows, the seated man) on the new bodies | their goldens; a blind review |
| E10 | **Evidence and the ledger**: blind reviews of every creature, `lore-keeper`, `code-reviewer`, `content.json`, `QUALITY.md`, the full check and the drift sweep | full check and drift pass |

Every milestone ends committed, with `cargo xtask check --fast` (the full
check for creatures, rendering and audio), the app built and run
headless, and a bench run recorded.

## Progress

| | Commit | What was done |
|---|---|---|
| E0 | `c21798d` | The plan and five research notes (`docs/research/`, `SOURCES.md` section 11). The `lore-keeper` reviewed the first draft and its findings were all taken: a creature that laid the flat sheets ("who lays the flat sheets is never said") and dressed like the cleaners ("never seen") became the Reader, which never touches her sheets nor the flat ones; a voice at the cell door during the self-assembly (it spoke as the residents, who are gone, and through the siren, which stops when the fog is up) and a fused mass carrying the Building's things down into the rooms (it showed what the fog does) were dropped for the one in the corner, the Watcher's opposite; a Watcher that turned toward the way down (it "only moves when you are not looking"; nothing points the way) and a second Hound answering the first ("only one comes at a time") were given other things to learn; the Watcher stays silent; the Smiler is slowed by bright rooms, not repelled; two torn notes that teach the old counters are rewritten with their creatures (design 9) |
| E1 | `ed4dace` | The ground. `assets::bodies` reads `bodies.bin`: per body a skeleton (bones with a parent, a head and a tail; every rest frame axis-aligned at its head) and its meshes, skinned (four bone slots) or rigid on one bone, with sRGB colours, UVs and morph offsets, in deflated buffers behind a JSON header. `entities::body` builds one into the scene (`build_body`: bones, skins bound, rigid meshes on their bones); `BodyRig::aim` points a bone along a direction in the body's frame whatever its parents are doing, `two_bone` solves a limb toward a pole. `entities::brain`: the modes, percepts, awareness (from 0.3 the tell, at 1 sure), memory, orders, and a log of every change with its reason. `cargo xtask assets creatures` runs `assets/src/creatures/build.py` in Blender headless (`kit.py`: skeletons, metaballs, remesh, decimation, bone-heat weights, hashed noise, previews; `hm08.py`: MakeHuman's CC0 base and its targets, fetched once and checked by SHA-1) and packs the result; two builds give the same bytes. The only body so far is the test body (`body_mannequin.py`). The creature pack (`backrooms --bench-creatures`, `perf/creatures.csv`, the `creatures` budgets in `PERF.md`) times the old creatures first: 5 placed, update 0.07 / 0.89 ms, GPU 2.2 / 3.6 ms. `tests/bodies.rs`: every body decodes soundly, `aim` and `two_bone` do what they say. No golden, fixture or level moved (fast check: 267 tests, 33 of 33 goldens); the app bundle builds, is signed, and runs headless |
| E2 | `1a153c2` | The Hound, from scratch, and the runtime every creature of the redesign shares. `entities::creature`: a creature's body and brain in its `EntExt`; `perceive` (hearing along the way it would walk, `path_distance`); `creature_tell`, `strike_allowed`, `creature_contact` (the first of a kind in a run knocks you down, `RunMemory`; a second in a peak or one on a spent nerve kills; the director's `contacts`), `creature_go` and `creature_burst` (capped); `open_point` (a guess at where you are never lies in a wall); the director's wishes read as orders. `entities::hound`: blind; it hears your step twice as far as anyone (13 m walking) when it stops, 0.6 of that while it moves (its own nails cover yours); it stops to listen every 4–8 s, every 2–4 s when sent close; the listening crouch (1.5 s, its first 0.6 s only listens: your moment to freeze); a search goes near the sound, never onto it (a blind thing places a sound a metre or more off, never in a wall), sniffing each spot 1.4 s; it smells a player it lingers by (0.9 m, 3 s); a charge at 6 m/s for 6 s, then a stop to listen in which moving gets you charged again; a spring from 1.5 m (a growl, 0.35 s, a pounce that leaves at 7 m/s). Told by the director where you are, it takes a hint at most every 8 s, 4–7 m off. Its body (`body_hound.py`): MakeHuman's old thin man posed on hands and feet (`human.py`: weighted to a standing skeleton, posed, baked, remeshed), wasted with knobby joints, ribs, the spine's knobs, a vest and long johns grown off the skin and torn, hair on three bones the game keeps pointing at the floor; 18.2k triangles. Its gait: hands and feet planted, two-bone limbs, the left hind dragging; it lifts its face to you over your body. Its voice: nails as each limb plants, the drag, a wheeze, sniffing, panting, a hoarse bay (+16 dB over the room: `the_hound_is_heard`), a growl. Torn note 10 rewritten; `LORE.md`'s Hound. Tests: `tests/creatures_rules.rs` (blind; along the way, not through walls; the tell before it is sure; silence sends it to look; smell; the chase and spring caps; no strike without two tells or in a level's first seconds; the first meeting and the second contact; its hair hangs), `tests/creatures_arenas.rs` (a tour of a pillared room with the director sending it: with its counter 98% untouched, none killed, all round in 215 s; without, 11% untouched, 17% killed, all round in 130 s). Tuning the arena found what the first drafts got wrong: a hound deaf while it crept, slower than a walker, its searches homing onto a frozen player by ear and into corners, a missed spring that kept its speed, and a told hound that came to look instead of coming. Re-recorded: the Hound's four goldens and their look anchors; the old reference tests (`tests/entities.rs`) now skip cases with a creature of the redesign (3 animation and 1 update case remain the old creatures' until E7). Pacing, personas and story pass with it. The creature pack: update 0.07 / 0.82 ms, GPU 1.87 / 2.27 ms, 243k triangles (5 placed). A first bench run put level 1's GPU p99 (the Well: no creature) at 6.16 ms over its 6 ms budget; three runs of level 1 alone with the same binary gave 5.93, 2.24 and 1.45 ms, so the p99 of 240 frames on the Building is noisy; that run was not recorded, the rerun (1.45 ms) was. The app bundle (19.3 MB) builds, is signed and runs headless |
| E3 | `6d9a031` | The Smiler, from scratch. `entities::smiler`: it sees your lamp 25 m down any clear line (in its view, or your beam on it) and your body only within 8 m while you are lit; its ears are a little duller than yours (0.8: a crouch at under 3 m); only sight, or bumping into you (0.8 m), tells it exactly where you are: a sound places you 1–3 m off, and when your light goes out it goes on, slower, to 1.5–3 m from where it was, and strikes there only if it finds you. It hunts at 4.5 m/s, half that where it *or you* stand in a bright room (over 0.55). Running within 8 m of it, or letting it reach you, brings the lunge: a sharp inhale, 0.5 s, coiling as it comes, then 7 m/s over 4 m. Sent, it goes where the director parks it, out of your sight, then creeps on toward you to 10 m (it hunts by sight: from a place you cannot see, it cannot see you); waiting, it turns slowly so its eyes sweep the dark; told where you are, it takes a hint 4–7 m off at most every 8 s; withdrawn, it goes 20 m or more from you. Eyeshine is your beam given back: how far inside it its eyes are, how squarely it faces you. Its voice: a click of teeth (alert), a hiss (sure, +13 dB over the room: `the_notice_is_loud`), an inhale, the lunge, slow wet breathing close by. Its body (`body_smiler.py`, 2.07 m): MakeHuman's young thin man stretched (forearms 1.62×, the neck 1.7×), stooped, wasted to the bone; the skin is wasted by the bone it follows (`posed.owner`, carried through the remesh), so a hanging arm no longer pulls the chest's side onto it, with the offsets relaxed so nothing steps; the waist pinched, the pelvis to the bone, the spine's knobs on the back (they had been raised on the sternum: a sign error); its face sculpted to the skull in the head's rest frame (orbits under a brow with the eyes sunk 13 mm, cheekbones over hollow cheeks, sunken temples, the nose dried down) and the rictus: its lips gone behind the teeth from cheek to cheek, dark gums; 30 single teeth along a dental arch fitted to MakeHuman's (`anatomy.teeth_row`: chisel incisors, pointed canines, cusped molars, one missing above, one below, one broken), 13k triangles of skin and 3.3k of teeth. Tests: seven rules in `tests/creatures_rules.rs` (the lamp from far and not you in the dark; it loses you in the dark; running near it brings the lunge; slow in bright rooms; eyeshine only in your beam; gone dark and still close, it finds you 3 times in 40; it sees ahead, not behind) and its arena (a dark pillared room with three lit patches; its counter played as a player would: lamp out once you hear it or see its eyes, still in the dark, walking, never running, out of the light by a way you reach before it): with the counter 99% untouched, none killed, all round in 181 s; without, 10% untouched, 1% killed, all round in 117 s. The arena's failures showed what the first draft got wrong: a counter with no winning move (it came at 4.5 m/s from the dark at someone lit by the room), a lunge that could never land (0.8 s standing still), an exact memory of where you went dark, a stalker parked out of its own sight, a roam goal never given up. Shared fixes for every creature: no strike within 2 s of *any* notice (only the first had counted); a first meeting's 45 s away is kept (the director's 12 s retreat had overwritten it each frame) and goes somewhere new (it had gone on to where it was heading). The Hound's arena now: with its counter 99% untouched, without 11% untouched and 10% killed (was 21%). Torn note 9 rewritten; her fifth page re-read (she puts the lamp out); `LORE.md`'s Smiler with Entity 3's credit; her lantern no longer said to draw Smilers. Re-recorded: the Smiler's four goldens and its death frame's look anchor (the lit grin filling the screen is gone); 17 other frames whose grain and camera sway moved with the native stream (content checked unchanged). Retired: three tests of the old view-cone senses (no kind on the old AI looks ahead now); the update reference has no case left (it goes in E7). Harness: the pacing and ambient tests seed the native stream (they had passed or failed with the clock; a rule in `DEVELOPMENT.md`); personas carry a lamp in the dark and name what killed them; the bench takes a second look (six runs) at a level over budget, after the Well's GPU p99 measured 1.4–6.1 ms over the same code. The shifted native stream also showed the wet weather's drip could come back within 6 s, against the mix's own rule of 8 (`tests/loudness.rs`): it now waits 8–18 s; and `the_door_is_heard` counted the door's calls in the ambient log, which keeps only the last three sounds, so a call followed by two others went uncounted: it now tells a call by its time. Pacing, personas and story pass; the full check passes but for that test, fixed and its target rerun. The creature pack: update 0.07 / 0.85 ms, GPU 1.87 / 2.18 ms, 227k triangles. The app bundle (19.5 MB) builds, is signed and runs headless |
| E4 | `94e7d7f` | The Watcher, from scratch. `entities::watcher`: seen is in your view (a lenient cone, about the screen), lit (the light map over 0.35 where it stands, your lamp on it, or within 4 m) and not behind a wall; at your shoulder it is unseen. It moves only when the director has sent it, and only unseen: at 4.4 m/s (faster than you walk; at 3.2, the first draft, it never caught a walker), and after 2.5 s out of sight it may be simply nearer, up to 6 m along the way, never within 10 m of you nor where you could see it. Within 6 m it is after you (Hunt: the director's peak, which is short, so the long approach is the build-up); in the peak it began it stays sent. Its touch goes through `creature_contact` with a new `KindData::touch_nerve`: −45 nerve and a knock-down every time, a death only for a mind it leaves spent; two tells first (your ears ring within 8 m, a faint high beating tone from where it stands; and seeing it nearer than you left it) and never in the 2 s after you look away (its notice is each time it starts to come). It falls back 3 m. Each time you look back it stands differently: its pose changes only unseen. Its body (`body_watcher.py`): one dust sheet, MakeHuman's old thin man under it never shown; the sheet draped by Blender's cloth (`cloth.py`) on a 4 cm grid over him in six stances, each drape made 1 cm fine by a cubic B-spline over the grid, its face pressed in (a soft shrink-wrap, relaxed and pressed again eight times so the spare cloth does not crease), the stand drape thinned once to 14k triangles with the face kept, and every other drape read off at the same places on the flat cloth as a morph with its normals (`cloth.sample`); linen grey with handling, yellowed in blotches, dirt from the hem (`linen.py`). What the drapes taught: the solver gives every vertex the same mass, so a grid denser at the face dragged the sheet forward (simulate even, refine after); a second solver run with cloth pinned on the skin flew apart (press geometrically); Laplacian smoothing drew the cloth in under the skull; the face band must start below the head bone's root, where the mouth is. Furniture under the same sheets (`sheeted.py`: a chair, a table, a wingback, 3.2k triangles each, in `bodies.bin`): the placer that had built them of boxes now drapes them, its draws kept in order so the placers after it are unchanged, and up to three stand round every Watcher (chosen by place, no draws). Its death frame: it stands close, facing you, its face in the cloth at your eyes' height (two marker bones the camera looks at). Shared: a body mesh with morphs gets an influence each at build (the renderer drew none); `Hold` reads to the director as a search; the director gives the stalker's signs only within 40 m (it had played steps far off from a creature 100 m away: a sign must mean it); the Smiler's final push takes a hint every 5 s and goes at its stalking pace, and sent, it creeps to 5 m. Tests: seven rules (moves only unseen; in the dark only your lamp or nearness holds it; a wall hides you from it; it stands differently only unseen; never put nearer than 10 m nor in sight; its touch, and a spent mind; the ring is its tell) and its arena (a lit pillared room; its counter keeps it in sight once it is about, and glances back every 2.5 s; the naive walks the tour, no sign of it but the ring): with the counter 100% untouched, all round in 150 s; without, 0% untouched, none killed. The arenas give your nerve back as the game does outside a dark zone (3 a second; nothing had); the arena bot can watch one thing while walking to another (`step_facing`). `bodies.rs` checks each pose's own extent (the box of relative morphs is wider, for culling). Retired: the animation reference has no case left (E7). Re-recorded: the Watcher's four goldens and its death look anchor; eight frames whose grain shifted with the native stream (content checked unchanged). A tension for E8: in a dark zone your nerve drains at 6.5 a second with your lamp off, and lamp off is the Smiler's counter (design 7). Pacing, personas (2 of 8 signs led to a peak, 25%; with fewer than 10 signs the share rule did not apply) and story pass; the full check passes. The bench: one build put every level's update p99 at 4.7 ms under `cargo xtask bench` and at 0.75 ms by hand with the same binary; a change to code after the timed loop took it away (the bench now lists frames over 2 ms, `update_slow`; none since); recorded run within budget, the creature pack update 0.07 / 0.83 ms, GPU 1.87 / 2.18 ms, 222k triangles; `bodies.bin` 1.0 MB. The app bundle (20.1 MB) builds, is signed and runs headless |
| E5 | `ddf18ba` | The one in the corner, from scratch, and the first placed kind. `entities::corner`: someone tall standing in the far corner of a quiet room, forehead to the angle of the walls, humming. Placed after the build and never sent (`corner::place`, from `session::flow` once the story is fitted): from depth 2, not on the Building's floors nor an anomaly's level, on about half the levels by a hash of the run's seed and the level (no draws); in the quietest inside corner more than two tiles off the walk from the arrival to the door, 8 tiles or more from the arrival and 5 from the door and her camp; it goes back there. Its back is harmless. The rule (`sees_its_face`): you in front of its face, its face in your view, a line between you, and lit (the light map over 0.35, your lamp on it, or within 3 m). Then the hum stops and its head comes round over 1.5 s: look away and it turns back; hold the look and it turns, screams and comes at 5.6 m/s, below your sprint, knowing where you are for 6 s and then only while it sees you; out of its sight 6 s, it walks back to its corner. Its face is to the corner, so the rule alone almost never fired: now and then (every 25–45 s while you are within 20 m and in its sight) the hum stops and it looks round at the room behind it, and then your eyes are the danger. Its touch is a contact (`creature_contact`: the first knocks you down). Its voice: one low hum note at a time (98–140 Hz, wandering; 4 dB over the room's bed: `the_one_in_the_corner_hums`; the first draft queued a phrase and stacked them), the scream. Its body (`body_corner.py`): MakeHuman's old thin tall man, straight, arms at his sides, the head a little bowed; a person's face with the eyes and the mouth upside down: each point of a feature takes the depth of its mirror across the feature's midline (turning the surface half round had swirled the skin about it), the brows and lips painted and flipped with it, the teeth turned over; 12k triangles of pale, waxy skin. A nightshirt to the shins (9.1k triangles), a tube of cream flannel with pale blue stripes gathered at the neck and let fall over him by Blender's cloth: begun outside him at every height (cloth begun inside a body cannot be freed by the solver; a cape, the first draft, let the hands through), its pin group cleared after (the packer read it as a bone and broke the skinning). It turns its head up to 2.6 rad on the neck, drops its jaw to scream, breathes, and walks on planted feet. Tests: six rules (its back is harmless; seeing its face turns it and looking away in time saves you; holding its look brings it, below your sprint; it knows where you are for 6 s, then goes back; now and then it looks round, and then your eyes are the danger; it is never sent); its placement on 40 levels of five runs (`tests/corner_placement.rs`: at most one a level, in an inside corner, off the way, far from the arrival, the door and her camp; it stood on 11 of the 20 levels of the rooms), which is the level sweep: the walk from the arrival to the door never passes within two tiles of it, and it acts only on a look; and its arena (a small lit room, it in a corner, five laps of a tour past it, running once it comes; the counter is to look anywhere but at it from when the hum stops): with the counter 98% untouched, all round; without, 67% untouched. The arena showed what the first drafts got wrong: a rule that never fired, a room too big and a tour too short to meet it, and a trial that ended in the middle of a chase (a trial now goes on up to 30 s past the tour while something hunts). Harness: `cargo xtask assets` runs Blender with `--python-exit-code 1` and fails on a traceback (a script that raised had written nothing and passed). Re-recorded: the corner's four goldens and their look anchors; `seed4242_level09.json` (its one change: the one in the corner). The full check found two things. The loudness test (seed 9001, level 11: a loudness range of 2.4 LU against 6.6, the corner 190 m off) showed the corner drew from the per-frame stream as a level started (its heading in `add_entity`, its first look round), against invariant 1, so every frame after it moved: its making now draws on a stream of its own and its first look round is set by its place, and level 6's arrival frame, which had moved with it, is back to E4's pixel for pixel. And the random level sweep found a Building floor (seed 2408377353, level 3) with a torn note in a wall: a dead end's spot was drawn where its platform runs into the edge of the floor; the spots now keep only walkable tiles, the draws unchanged, so no recorded level moved. Pacing, personas and story pass; the full check passes. The creature pack: update 0.07 / 0.85 ms, GPU 1.87 / 2.20 ms, 222k triangles (meant to include it, hunting; the pack had run out of tiles before it, found and fixed in E6); `bodies.bin` 1.2 MB. The app bundle (20.3 MB) builds, is signed and runs headless |
| E6 | `8274df8` | The Reader, from scratch. `entities::reader`: it sees you over its page, 12 m in its view and only where you are lit or moving (still in the dark you are nothing to it; it feels you within 0.8 m), and has no ears. Its compulsion (`paper_near`): any sheet on the floor within 4 m that it can see stops it whatever it is doing: it goes to it, stoops (0.8 s), takes it and reads it to the end (3 s), blind to you, in a mode of the brain's own, `Read` (the director feels it as a search); the sheets you drop it keeps, a pile lying about it reads the top sheet of, once. It reads only `paper_markers` and `papers`, so her folded sheets and the flat ones are never its. Roam (reading as it walks, whispering) → Alert (it looks up: the whisper stops, a tell) → Hunt (it snaps the sheet flat, a tell, and comes at 4.8 m/s; its quick soft steps behind you are a tell every 2 s within 9 m) → Strike (it lowers the sheet over 0.8 s as it comes on: a walk does not outpace it, a run does) → Withdraw. Sent, it goes where the director would have it and reads its way on toward you to 9 m. Placed (`reader::place`, after the build, no draws): from depth 2, on about seven levels in ten with paper lying about, beside a pile (not on it), an office wing preferred, 18 tiles or more from the arrival and 5 from her camp: on 15 of the 20 room levels of five runs (17 had paper). What it chooses far from you draws on a stream of its own (`own_stream`: its home and a count), its making inside `with_seed`, its rustle fixed: it moves nothing on the per-frame stream, and no recorded level's frames or loudness moved. Its body (`body_reader.py`, 20.6k triangles): MakeHuman's thin man, half young and half old, stooped over a page held before its face, the fingers half as long again (the thumb a quarter), spread on the back of the page, the page square to its eyes on the face side of the finger pads; a cardigan, trousers and shoes grown off the skin and made to hang as cloth does (new in `anatomy.py`: `hull_slices`, each level slice of the trunk out to its convex hull; `hang`, no point lower down nearer the axis than the cloth above; `fill_hollows`, smoothing that only moves outward; `loosen`, limbs to a least radius), the cardigan's left front a button lower; slips of paper at the V and in the pockets; a thin cap of hair on the head bone alone; staring eyes on bones of their own that read the page line by line; ink on the fingertips and at the corners of its mouth (it licks its fingers to turn the pages). What the body taught: painted by the nearest bone, its brow came out inked (the long fingers raised before its bowed face are nearer the brow than the skull's bone): paint by the bone each point belonged to. Its voice: the whisper, runs of 4–8 syllables of shaped noise over a breath, 17 dB over the room at 2.6–4.8 kHz (`the_reader_whispers`; a burst falls away over its whole length, so the first syllables were clicks); the snap; a sheet taken; its steps, and the director's sign of it, a shuffle and a page turned. Tests: eight rules (only lit or moving; 12 m and ahead; its tells and its pace; a sheet stops it and it reads to the end; a pile once, left; never her sheets; it lowers its sheet before it touches you; it is sent); its arena (a lit pillared room, eight sheets in your pockets; the counter drops one behind you when it comes within 9 m and walks away): with it 100% untouched, without 3%, none killed; its placement (`tests/reader_placement.rs`); the story test (`the_reader_leaves_her_sheets_alone`: beside each of 34 of her sheets and the flat ones it never stopped, and they ended as they do without it). The arena found two flaws: lowering its sheet standing still let any walker leave its reach, and a look-up missed beyond the whisper's reach left one tell, so it trailed you at 0.2 m never allowed to strike (a rule now: a hunter must keep giving tells you can perceive). Shared fixes: the personas' Rusher and Hider died at their first meeting with it, the knock-down taking the last of a nerve the dark had spent: a contact that is not the end now leaves `KNOCKED_LEAVES` (5) and a knock-down kills only when its own blow emptied you (design 3). The creature bench's pack had stopped at five creatures (level 4's arrival has five tiles within 14 m): neither the Reader nor the one in the corner had been in it (E5's row said otherwise, corrected); its ring now widens to fit every kind, and its history compares only packs of one size. Re-recorded: the Reader's four goldens and look anchors (new); `seed4242_level06.json`, `seed4242_level11.json`, `seed77_level08.json` (one change each: the Reader). Pacing, personas (2 of 10 signs led to a peak, 20%; no deaths) and story pass; the full check passes. For E8: the Hider persona sits at zero nerve in the dark from 90 s on (the dark zone's drain, design 7). The creature pack (now nine: two of each kind that roams and the one in the corner, all hunting): update 0.09 / 0.86 ms, GPU 1.83 / 2.23 ms, 293k triangles; `bodies.bin` 1.46 MB. The app bundle (20.6 MB) builds, is signed and runs headless |
| E7 | `2068965` | The director and the place. **The old creature code is gone**: `ai.rs` keeps only the shared steering, the update loop and the screen's pressure (1084 lines to 581), `anim.rs` (680 lines) and the old builders, textures, rigs and poses of `entities/mod.rs` are deleted, the director's old search, notice and strike are orders now; the old creatures' reference fixture is pruned through REGEN to what is still read (the steering, the staff's limb solver, two tables: 9.2 MB to 1.3 MB) and its builds, animation, update and spawn references are retired with their creatures; `tests/structure.rs` reads the kinds from `on_new_brain` and asks each for goldens, rules and an arena (it had read the old builders), and the add-creature skill is rewritten for the new pipeline. **The ecology** (`Game::ecology`, by the places, no draws): one on the Lobby (a Hound, or a Smiler where it is dark), two at depth 2, three deeper; a dark region calls the Smiler, furnished rooms the Watcher, paper lying about the Reader (office wings the more), the Hound goes anywhere; each kind once while the places allow, each put in its own kind of place; places a creature can use only (reached from the arrival, a quarter of the level off: a Watcher had been given furniture it could not reach). On 35 levels of five runs: Smilers 33, Hounds 19, Watchers 21, Readers 19, the one in the corner 11 (`tests/ecology.rs`, which takes over the corner's and the Reader's placement tests: each level built once). A level with the Stairwell's or the Loop's door is rooms like any (the first draft gave them nothing). **What they learn** (`creature::LEARNS`, design 4): four hidings from a Smiler in the dark, and it waits silent 6 s where you went dark (your lamp on then, and it has you); three escapes from a Hound, and it charges without a bay (its nails the tell) for where you are heading; five long stares at a sent Watcher, and the panels over it fail while you watch it, its place dark 5 s, and it comes on in the dark it made; four passes behind the one in the corner, and it looks round every 10–18 s; three sheets for a Reader after you, and it tears them unread (1 s, and you hear it). At most one kind a level (the rest as the next level starts), the first use where you can see it, the ledger saved with the run (an old save reads as a new run's). **The director**: three in ten build-ups end in nothing (one sign, six seconds of quiet; never one her page or a trail called); a stalker sent from further than 60 m along the level is first moved, out of your sight, to 30–45 m (sent from 190 m, it had never arrived: pacing's four levels had 3 peaks, now 9–14 over six streams); sent away, every kind withdraws as soon as it no longer has you (a Smiler that searched on in the relief began the next peak at once: 10 peaks in 300 s), but a chase stays its own until its rule loses you (cut short by the director, a naive bot outran every creature for 8 s and the counters stopped mattering), and one sent away, finishing its chase, starts no new peak (each chase had counted twice). **Fairness**: a Watcher's touch kills only a mind it leaves spent and never at a first meeting (the personas' explorer and rusher had died at their first touch). Tests: seven of learning (counts, one a level, the save; the first time in plain sight; each kind's new behaviour), the ecology; the arenas hold their bands (the corner 98% / 67%, the Watcher 100% / 0%, the Reader 100% / 4%, the Smiler 99% / 15%, the Hound 99% / 9%). Re-recorded: all 41 goldens (the start-up no longer paints the old creatures' textures on the per-frame stream, so every canvas texture after them draws other numbers and the death camera shakes otherwise; content checked unchanged: the Building's arrival, the office wing, the Smiler's death); 11 reference levels (one change each: the creatures the ecology gives); the loudness scenarios (the same reshuffle: seed 9001 level 11's range 6.6 to 2.9 LU with integrated loudness within 0.7 LU and the loudest moment unchanged; bringing stalkers nearer was checked not to be the cause). Pacing, personas (9 of 31 signs led to a peak, 29%; no deaths), story and listen pass; the full check passes. The creature pack: update 0.09 / 0.83 ms, GPU 1.87 / 2.36 ms, 299k triangles. The app bundle (20.5 MB) builds, is signed and runs headless |
| E8 | `8c48beb` | Beatable, and shown to be (design 7 as built). **The dark zone, measured first**: the rule-follower spent 1006 s in a dark zone on a spent nerve, 997 of them with its battery dead (a battery lasts 25 s lit) and 8 with its lamp out for a Smiler; all 7 of the explorer's deaths came on a nerve the dark had spent (now 65 s, all with a dead battery, and none). Hiding from the Smiler was not the trap, the battery was; so a dark zone now takes your nerve no lower than 40 and gives it back up to 40 (`player::DARK_FLOOR`): a spent nerve is the creatures' doing alone, and it passes (at 30, the dark and one sign of a stalker, up to 12, still spent a nerve no creature had touched). **The rule-follower** (`tests/rule_follower/mod.rs`): every counter as a player plays it from what each creature shows (its mode standing for the sign), its lamp kept as a player keeps it (a spare in under 10, one picked up under 40), paper taken from a pile when a Reader is about; it moves by the keys. **Beatable** (`tests/beatable.rs`, a test per bot, levels 4–13 of four runs in order, what the run remembers of the creatures carried on, 480 s a level, deaths on, the place woken by her page): the rule-follower finished all 32 levels with no death (18 peaks, 11 contacts), the explorer 31 (one death, a Watcher's), the rusher 32; no hard rule broken. Those who know no counter finish nearly as often (36 contacts cost the explorer one death: a question for people). The first sweeps failed on the bot, never on a creature: aimed past a corner it stood 400 s (it steers now to the farthest point a body walks to straight), it went to and fro between two batteries (one detour, kept), it walked on into a Smiler's breath (it creeps away within 7 m), and a first meeting never killed because each level started a fresh run (the run's memory is carried now). And a level's frames depend on what its thread played before: what is made once a thread (the creatures' cloth, any canvas texture) draws on the per-frame stream the first time only, so `one_level` replays the sweep's order. **What they learn, measured**: at E7's counts (4, 3, 5, 4, 3) no kind learned in twelve bot runs, the most any habit reached being 2; now 2, 2, 3, 2, 2 (`creature::LEARNS`), and the explorer's Smiler learned in two runs of four. **The whole run live** (`the_whole_run_live`): her story followed on the rooms' own floors, three runs to level 14; two deaths on the way, each level then passed; the creatures came on 9 of 29 live levels (11 peaks, 11 contacts). It had failed at the Loop: god mode was restored after the Loop's walk, not before. **The learned arenas** (`what_each_learns_leaves_its_counter_working`): each kind met, learned and shown; the counter as good as before and the new behaviour seen in most trials (`learned_uses`): the Smiler waits in 83%, the Watcher's lights fail in all, the one in the corner looks round in all, the Reader tears in 97%, each 99–100% untouched; the Hound, which its counter keeps from charging (1 trial in 100), heard first: 59% learned, 60% before. They found two bots playing badly, not creatures: eyes turned back to the one in the corner a step out of the hum's reach as its head came round (eyes away 3 s now), and a sidestep from a Hound's charge (now to a place reached well before it). Found on the way: a player who goes straight to her camp and the door mostly leaves before the first peak (a question for pacing and people). Tests: the learned arenas; `beatable.rs` (a test per bot, `one_level`); `the_whole_run_live`; the rules test reads the counts from `LEARNS`. Pacing (first build-ups at 43–63 s, no deaths), personas (7 of 24 signs led to a peak, 29%), story and the rest pass; the full check passes (its bench found two steps E7 had made, now three runs back: level 4's build 135 → 148 ms, the creatures' first-use setup paid there, and 4242 L5's triangles, another creature in view; explained in `PERF.md`, and a second run recorded). The creature pack (nine, all hunting): update 0.09 / 0.83 ms, GPU 1.87 / 2.35–2.38 ms, 299k triangles; within budget. The app bundle (20.5 MB) builds, is signed and runs headless |
| E9 | `6540e76` | The anomalies' figures on new bodies, kept only where a blind review preferred them. **The rig's rest pose** (`assets/src/creatures/rigs.json`, written by `tests/anomaly_bodies.rs` from `anomalies::rig::human_rig`; REGEN=1 after a rig changes) is what the body scripts fit MakeHuman to (`figure.py`: `fit`, the head its own length with its crown on the rig's: MakeHuman's head bone starts at the eyes and the rig's at the skull's foot, so mapped bone to bone the face sat on the neck; the trunk stretched evenly: the short spine stretched twice as far as the chest had left a ridge; the joints moved onto the rig's, blended along each bone); `rig::bind_figure` binds every part to the rig's bones by name, and `sync_entity_skins` keeps each one bound as the figure moves. **Faceless** (`figure::faceless`): each point of the face at the distance of the head round it, from the skull's middle, done on MakeHuman's own skin before the remesh (after it, the closed eye sockets overhung and stayed as pits), the face judged facing by its direction from the skull, not its normals. **The staff** (`body_staff.py`): 2.14 m, bald, a faint hollow for a face, the arms' muscles and the neck's tendons in relief, an open-collared mustard polo tucked into dark trousers and a belt with a buckle of its own, a badge with a photograph; its hands kept from the decimation (`decimate_keeping`) after a 3 mm remesh (at 5 mm its hanging fingers ran together). **The seated man** (`body_seated.py`): on the Loop's chair (the seat and the floor solved for: his thighs tipped until his soles meet it), his hands on his thighs, a white shirt, a dark tie, grey trousers; the Loop draws him with his chair (`Builder::add_painted`, his own colours). **Blind reviews** (`cargo xtask ab`, both orders, a fresh reviewer a round): the seated man preferred in all five; the staff lost four rounds (arms like sticks, hands like spikes, a flat egg, a smock, a mock neck; the old one's muscle, tendons and open collar were what they liked) and won the fifth with two reviewers at 2 m and 4 m; the shadow lost or tied four (the old one's hooked claws), so the shadows keep their old body. What the reviews taught the tools: masks by the bone a point belonged to (by the nearest, the hanging hands took the hips' side), garments cut by height where bones overlap (the thigh's skin reaches up the hip), finger bones grouped by family (`finger12`), colour not painted finer than the cloth's triangles (buttons smeared to pale diamonds), a waistband weighted to the pelvis (by heat to the thighs, it opened 12× at the back in a stride). Tests: `tests/anomaly_bodies.rs` (the rig's pose is what the scripts read; the staff's bones on the rig's, every part bound, in budget; skinned on the CPU through the hunting stride, no edge torn: 99.9% within 1.59×; the seated man's soles on the floor); `anomalies.rs` and `loop_rooms.rs` compare their recorded trees without the bodies, which are no longer the reference's. Goldens: the staff at 2 and 4 m, the shadows at 2, 4 and 8 m, the seated man at 2.5 and 1.6 m (`--screen loop-figure[-near]`, new; `--showcase kind:dist:walk` places a figure once and lets it walk). The Showroom with its staff (4242 L10): build 248 ms, GPU 1.14 / 4.08 ms, 1.0M triangles, within budget; the bench within budget, no slowdown; the creature pack 0.09 / 0.87 ms, GPU 1.87 / 2.35 ms; `bodies.bin` 1.82 MB. The full check passes. The app bundle (20.8 MB) builds, is signed and runs headless (the staff, the seated man) |
| E10 | (this commit) | Evidence and the ledger. **Blind reviews** (`cargo xtask ab`, both orders, a fresh reviewer each round, `tests/review/verdicts.jsonl`): against the old creatures, the first round preferred the new Hound at 2 and 4 m and at its death (not at 8 m: pale against the wall) and the new Smiler and Watcher only at their deaths (the Smiler a smooth black mannequin with a faint grin, the Watcher a cone-shaped bedsheet ghost). **The Smiler redone** (`body_smiler.py`): ribs 6.5 mm deep (3), the belly fallen 26 mm under them (16), dust dried on the bones where they stand out (the vertex colour where the skin is least hollow), the dental arch 1.6 times MakeHuman's (1.35), the teeth longer and paler (bared and dry); a wet sheen was tried and dropped (one reviewer read it as latex, where the dull one had read as a body suit to another). **The Watcher redone** (`body_watcher.py`): a man of 1.96 m under it (MakeHuman's old thin man, 0.4 of the way to the tall one), a sheet of 3.4 × 3.6 m, drawn in round him below the shoulders (`hug`: what stands out past his widest above keeps 30% of it, so it hangs as a narrow shroud; drawn in to each height's own width it had wrapped his legs like trousers), the jaw open and the head bowed to look down at you, and where his eyes and open mouth are (MakeHuman's eyes and teeth, carried by the head and the jaw) the cloth grimed dark. Each pose is read off its own drape where its face fell on the cloth (bent over you, its face was 15 and 27 cm away and shared 8% of the standing face's cloth, now 65%), so the face is kept fine and stained in every pose; stains from the cloth's creases (read as tears) and from every pose's hollows together (blots) were tried and dropped. Each new body preferred to the one it replaced by two reviewers: the Watcher at every distance and its death; the Smiler at 2 m (once at its death too), ties elsewhere, never the old. Against the old ones, the final Smiler is preferred at 2, 4 and 8 m and at its death; the Watcher still loses at 2 and 4 m and at its death (its tall robe with a face showing through), ties at 8 m. **A level's frames whatever came before**: the code review found the creatures' textures painted on the per-frame stream the first time a thread needed them; its fix (paint them at start-up) would not have held, since a sweep seeds the stream before start-up. Every canvas texture now paints from a stream of its own (`base::rng::with_own_stream`, seeded by its painter's place in the code, its size and how many it has made since the level began: `placers::kit::canvas_tex`), so a texture is the same whenever it is made and takes nothing from the frames after it; `tests/story.rs` `a_levels_frames_are_the_same_whatever_came_before` holds it (it fails with the painters on the per-frame stream). **The reviews' other findings, fixed**: a touch after the first meeting kills only a mind it leaves under 25 (`creature::SPENT`; a Watcher's 45 had killed anyone in a dark zone, where the dark leaves 40); the one in the corner at rest is nothing to the director (it had held the tension at 0.2 a level and kept peaks from ending) and is deaf (a cry had stranded it away from its corner), and never comes in a relief; the canon follows the Hound's ears (less well while it moves) and says its face is the last thing you see before it comes, that only one is sent at a time and the corner never, that the dark wears your nerve only so far; torn notes 9 and 10 hint where they had taught the rules; the credits name MakeHuman for every body; the module docs, DEVELOPMENT's frames, REGEN and showcase lists, and the kind modules' exception to `impl Game`. **The one in the corner heard at last**: with its rest no longer a search to the director, `the_one_in_the_corner_hums` failed, and showed it had passed since E5 on the director's heartbeat and drone: the hum itself added nothing (a full-scale tone into the mix added under 2 dB). Each note had been a pluck (`Soundscape::tone` falls away over its whole length, 80 dB in a note) at 98–140 Hz among the lamps' 120 Hz and the room's 47 Hz drone. Now each is held (`Soundscape::note`: up, held, down), at 160–210 Hz (`HUM_GAIN` 0.15), and the test seeds the per-frame stream (`LISTEN_NATIVE` shifts it) and measures the hum's own band, 150–225 Hz in half seconds, its notes against the room's loudest once it stops: 4.4–5.2 dB over on eight streams, 0.5 dB with the hum silent. Open (`QUALITY.md`): the lore review's sameness (three tells a sound that stops; the Reader and the Smiler both beaten by stillness in the dark; "tall" said of three), the Reader's body nearly answering who lays the flat sheets, and the hanging broken ceiling tile that two reviewers took for a hole in the death frames. Tests: the Watcher's touch (40, met: no death; under 25, first meeting: none; under 25, met: death), the corner at rest (nothing to the director, deaf), a level's frames whatever came before, the hum by its band. Re-recorded: all 48 goldens (each texture's pattern once, the per-frame stream after start-up, the two bodies; all looked at side by side) and the Watcher's death look anchor (0.305 → 0.365: its taller face draws the camera up under the panel). Beatable on the new stream: the rule-follower, the explorer and the rusher each finish 32 of 32 levels with no death (27, 41 and 19 peaks; 21, 33 and 15 contacts; the Smiler learned in one run of four for the rule-follower, three for the explorer): that the explorer, who knows no counter, is never killed (a touch now kills only a spent mind) is for people. Pacing, personas, story and listen pass; the full check and the drift sweep pass (53 test files, 48 goldens, the range unchanged, no content failing a check). The creature pack (nine, all hunting): update 0.09 / 0.86 ms, GPU 1.83 / 2.19 ms, 301k triangles; within budget, no slowdown in the history; `bodies.bin` 1.83 MB. The app bundle (22 MB) builds, is signed and runs headless |

## What only people can tell

Whether each rule can be stated after three meetings (the research's
test: four of five people say it in one sentence), whether the Smiler in
the dark frightens or just annoys, whether the Watcher read as furniture
first, whether anyone looked the one in the corner in the face on
purpose, twice.
`PLAYTEST.md` asks them (*The creatures*, E10), with what E8 and E9 left
to people: whether being caught costs enough (a contact mostly knocks you
down), whether a kind learns too soon, whether players run out of light,
and whether a player who crosses a floor quickly should meet anything.
