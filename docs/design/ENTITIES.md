# The things that live here: the creatures, redesigned

What the creatures of the rooms are, why, how each one behaves and is
beaten, and the plan that rebuilds them from scratch, body, behaviour and
sound, to the quality of the rest of the game. The canon it changes is
[`LORE.md`](LORE.md) (*Things that live here*); the pillars it serves are
[`PILLARS.md`](PILLARS.md) 4 and 5 above all; the research behind it is in
[`research/`](../research/README.md) (five notes, every claim linked) and
summed up in [`SOURCES.md`](../research/SOURCES.md) section 11.

## The work

How this was planned and done (the analysis, milestones, progress): [`../plan/ENTITIES.md`](../plan/ENTITIES.md).

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
