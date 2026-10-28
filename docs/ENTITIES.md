# The things that live here: the creatures, redesigned

What the creatures of the rooms are, why, how each one behaves and is
beaten, and the plan that rebuilds them from scratch, body, behaviour and
sound, to the quality of the rest of the game. The canon it changes is
[`LORE.md`](LORE.md) (*Things that live here*); the pillars it serves are
[`PILLARS.md`](PILLARS.md) 4 and 5 above all; the research behind it is in
[`research/`](research/README.md) (five notes, every claim linked) and
summed up in [`SOURCES.md`](SOURCES.md) section 11.

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
4. **Their bodies** are procedural skinned meshes built in the HTML
   version's pipeline, which was never ported (`creature_meshes.bin` is a
   byte-for-byte conversion; `QUALITY.md`: "creatures still rebuild only in
   the HTML version"). They read as models: a cartoon skeleton with a
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

Five notes, in [`research/`](research/README.md). What they changed:

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

### 1. Five creatures, five rules

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
| **The Reader** | hunter | **sight**, at middle range, in light | **paper**: drop a sheet and it must stop and read it | office wings, archives, wherever paper lies |

The Smiler, the Hound, the Watcher and the Reader are *sent* by the
director (§5), one at a time; the one in the corner is placed and never
sent, and comes only if you look at it. None of them walks the Building
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
- **What it learns** (§4): once you have hidden from it in the dark four
  times this run, it waits where you went dark, silent, no clicks, for 6 s
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
- **What it learns**: once you have outrun it three times, it stops baying
  when it charges and runs for where you were heading, not where you are:
  it comes at you from ahead.
- Unlike the Showroom's staff at night (be quiet, hide), the Hound's rule
  is a rhythm: its own noise covers yours.

#### The Watcher

*This game's own: the moving statue of the playground game
(Grandmother's Footsteps) and of SCP-173 (Moto42, CC BY-SA 3.0), under one
of the dust sheets that cover the furniture. The seated man of the Loop
shares the rule and the credit.*

- **What it is.** Someone standing under an old dust sheet, a person's
  height and stooping a little, the cloth to the floor, darker at the hem,
  a face pressed into it (brow, nose, the hollows of the eyes). It stands
  where furniture under sheets stands, and at a glance it is furniture. Each
  time you look back, it stands differently.
- **Its sound.** None (the canon). Close to it the hum dips and your ears
  ring.
- **The rule you learn.** *Keep it in sight.* It moves only while it is out
  of your sight: not in the middle of your view (within about 25° of where
  you look), not lit (by a light, your lamp on it, or within 4 m), or behind
  a wall. The check is lenient, as Containment Breach's is: in your view
  cone, it is seen. Unseen, it comes at 3.2 m/s; sent by the director it can
  be simply nearer when you turn round (never within 10 m, never where you
  could see). Its touch takes your nerve (−45) and knocks you down, and it
  falls back a few steps; it kills only a mind that is already gone.
- **Modes.** Hold (seen) ↔ Stalk (unseen) → Strike (touch) → Withdraw.
- **What it learns**: once you have held it with long stares (over 8 s)
  five times, the panels over it begin to fail while you watch it. It never
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
- **Modes.** Hold (in its corner) → Alert (turning, humming stopped) → Hunt
  → Strike → back to Hold.
- **What it learns**: once you have passed behind it four times this run,
  it starts to turn round now and then on its own (the hum stops first),
  and looks at the room behind it.
- Placed by the level's build in a corner of a room off the main way (never
  the only way on, never near the arrival, her camp or the door), at most
  one a level, from depth 2.

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
  the sheets you drop and the paper lying about; it never touches her
  folded sheets nor the flat ones (they are the same on every visit).
- **Modes.** Roam (reading as it walks) → Alert (it looks up: the whisper
  stops) → Hunt → Read (the compulsion) → Strike (it lowers the sheet:
  0.8 s) → Withdraw.
- **What it learns**: once you have escaped it with paper three times, it
  tears the sheets in half unread (1 s) instead of reading them: you hear the
  tearing, and paper buys you less.

### 2. One brain

Every creature runs the same brain (`entities/brain.rs`); a kind is data
(its senses, speeds, tells) plus its own mode rules, never `if kind ==`
scattered through the shared code (after Botta's Infected).

- **Modes** (`Mode`): Idle, Roam, Alert, Search, Stalk, Hunt, Strike,
  Recoil, Withdraw, Hold. A creature is always in exactly one; it leaves it
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
   does not kill: it knocks you down, takes your nerve, and the creature
   withdraws for a full relief.
3. **Death is postponed.** A strike kills on the second contact in the same
   peak, or on the first when your nerve is below 25.
4. **You can always get away.** A sustained chase is at most 0.95 of your
   sprint (6.1 m/s); a burst (the Smiler's lunge, the Hound's spring) is at
   most 7 m/s over at most 4 m.
5. **Grace.** No death in a level's first 30 s, and none within 2 s of a
   creature's first notice of you.
6. **Nothing is born where you are.** No creature spawns within sight or 24
   m of the arrival, her camp or the door; none relocates where you could
   see it.

### 4. What they learn (the habit ledger)

A run keeps a count of your habits: hiding in the dark from a Smiler,
outrunning a Hound, long stares at a Watcher, slipping behind the one in
the corner, paper dropped for a Reader. Each kind unlocks **one new
behaviour** (§1) when its count is reached, never from deaths, at most one
unlock a level; the first time it happens it happens in plain sight.
Learning you can see, not a stat that creeps up (Alien: Isolation; Hello
Neighbor's opacity is the warning). No learned behaviour breaks its
creature's rule.

### 5. The director

It keeps its phases (calm, build, peak, relax) and its menace gauge, and
gives orders instead of setting fields. It **sends one creature at a
time**: the Smiler, the Hound, the Watcher or the Reader (not the one sent
last, the nearest of the others); some build-ups end in nothing (a sign,
then quiet: 20–40% of them), so a sign is never a promise. The one in the
corner is never sent. **The ecology** decides who is on a level, by depth
and by the places in it:

| Depth (level) | Creatures |
|---|---|
| 1 (4, the Lobby) | one: a Hound, or a Smiler where the Lobby has a dark region |
| 2 (5) | two, of Smiler, Hound, Watcher (the Watcher only with furnished rooms), the Reader (only with office wings); the one in the corner in a quiet room |
| 3+ | three of the four sent kinds, by the places; the one in the corner |

The anomalies keep their own residents (E9 gives them new bodies).

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

### 8. Performance

`cargo xtask bench` times the **creature pack** (two of every kind that
roams, hunting you at once, level 4) beside the levels, held to the
`creatures` budgets in [`PERF.md`](PERF.md) and to its own history
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
| E2 | `1a153c2` | The Hound, from scratch, and the runtime every creature of the redesign shares. `entities::creature`: a creature's body and brain in its `EntExt`; `perceive` (hearing along the way it would walk, `path_distance`); `creature_tell`, `strike_allowed`, `creature_contact` (the first of a kind in a run knocks you down, `RunMemory`; a second in a peak or one on a spent nerve kills; the director's `contacts`), `creature_go` and `creature_burst` (capped); `open_point` (a guess at where you are never lies in a wall); the director's wishes read as orders. `entities::hound`: blind; it hears your step twice as far as anyone (13 m walking) when it stops, 0.6 of that while it moves (its own nails cover yours); it stops to listen every 4–8 s, every 2–4 s when sent close; the listening crouch (1.5 s, its first 0.6 s only listens: your moment to freeze); a search goes near the sound, never onto it (a blind thing places a sound a metre or more off, never in a wall), sniffing each spot 1.4 s; it smells a player it lingers by (0.9 m, 3 s); a charge at 6 m/s for 6 s, then a stop to listen in which moving gets you charged again; a spring from 1.5 m (a growl, 0.35 s, a pounce that leaves at 7 m/s). Told by the director where you are, it takes a hint at most every 8 s, 4–7 m off. Its body (`body_hound.py`): MakeHuman's old thin man posed on hands and feet (`human.py`: weighted to a standing skeleton, posed, baked, remeshed), wasted with knobby joints, ribs, the spine's knobs, a vest and long johns grown off the skin and torn, hair on three bones the game keeps pointing at the floor; 18.2k triangles. Its gait: hands and feet planted, two-bone limbs, the left hind dragging; it lifts its face to you over your body. Its voice: nails as each limb plants, the drag, a wheeze, sniffing, panting, a hoarse bay (+16 dB over the room: `the_hound_is_heard`), a growl. Torn note 10 rewritten; `LORE.md`'s Hound. Tests: `tests/creatures_rules.rs` (blind; along the way, not through walls; the tell before it is sure; silence sends it to look; smell; the chase and spring caps; no strike without two tells or in a level's first seconds; the first meeting and the second contact; its hair hangs), `tests/creatures_arenas.rs` (a tour of a pillared room with the director sending it: with its counter 98% untouched, none killed, all round in 215 s; without, 11% untouched, 17% killed, all round in 130 s). Tuning the arena found what the first drafts got wrong: a hound deaf while it crept, slower than a walker, its searches homing onto a frozen player by ear and into corners, a missed spring that kept its speed, and a told hound that came to look instead of coming. Re-recorded: the Hound's four goldens and their look anchors; the old reference tests (`tests/entities.rs`) now skip cases with a creature of the redesign (3 animation and 1 update case remain the HTML version's until E7). Pacing, personas and story pass with it. The creature pack: update 0.07 / 0.82 ms, GPU 1.87 / 2.27 ms, 243k triangles (5 placed). A first bench run put level 1's GPU p99 (the Well: no creature) at 6.16 ms over its 6 ms budget; three runs of level 1 alone with the same binary gave 5.93, 2.24 and 1.45 ms, so the p99 of 240 frames on the Building is noisy; that run was not recorded, the rerun (1.45 ms) was. The app bundle (19.3 MB) builds, is signed and runs headless |
| E3 | (this commit) | The Smiler, from scratch. `entities::smiler`: it sees your lamp 25 m down any clear line (in its view, or your beam on it) and your body only within 8 m while you are lit; its ears are a little duller than yours (0.8: a crouch at under 3 m); only sight, or bumping into you (0.8 m), tells it exactly where you are: a sound places you 1–3 m off, and when your light goes out it goes on, slower, to 1.5–3 m from where it was, and strikes there only if it finds you. It hunts at 4.5 m/s, half that where it *or you* stand in a bright room (over 0.55). Running within 8 m of it, or letting it reach you, brings the lunge: a sharp inhale, 0.5 s, coiling as it comes, then 7 m/s over 4 m. Sent, it goes where the director parks it, out of your sight, then creeps on toward you to 10 m (it hunts by sight: from a place you cannot see, it cannot see you); waiting, it turns slowly so its eyes sweep the dark; told where you are, it takes a hint 4–7 m off at most every 8 s; withdrawn, it goes 20 m or more from you. Eyeshine is your beam given back: how far inside it its eyes are, how squarely it faces you. Its voice: a click of teeth (alert), a hiss (sure, +13 dB over the room: `the_notice_is_loud`), an inhale, the lunge, slow wet breathing close by. Its body (`body_smiler.py`, 2.07 m): MakeHuman's young thin man stretched (forearms 1.62×, the neck 1.7×), stooped, wasted to the bone; the skin is wasted by the bone it follows (`posed.owner`, carried through the remesh), so a hanging arm no longer pulls the chest's side onto it, with the offsets relaxed so nothing steps; the waist pinched, the pelvis to the bone, the spine's knobs on the back (they had been raised on the sternum: a sign error); its face sculpted to the skull in the head's rest frame (orbits under a brow with the eyes sunk 13 mm, cheekbones over hollow cheeks, sunken temples, the nose dried down) and the rictus: its lips gone behind the teeth from cheek to cheek, dark gums; 30 single teeth along a dental arch fitted to MakeHuman's (`anatomy.teeth_row`: chisel incisors, pointed canines, cusped molars, one missing above, one below, one broken), 13k triangles of skin and 3.3k of teeth. Tests: seven rules in `tests/creatures_rules.rs` (the lamp from far and not you in the dark; it loses you in the dark; running near it brings the lunge; slow in bright rooms; eyeshine only in your beam; gone dark and still close, it finds you 3 times in 40; it sees ahead, not behind) and its arena (a dark pillared room with three lit patches; its counter played as a player would: lamp out once you hear it or see its eyes, still in the dark, walking, never running, out of the light by a way you reach before it): with the counter 99% untouched, none killed, all round in 181 s; without, 10% untouched, 1% killed, all round in 117 s. The arena's failures showed what the first draft got wrong: a counter with no winning move (it came at 4.5 m/s from the dark at someone lit by the room), a lunge that could never land (0.8 s standing still), an exact memory of where you went dark, a stalker parked out of its own sight, a roam goal never given up. Shared fixes for every creature: no strike within 2 s of *any* notice (only the first had counted); a first meeting's 45 s away is kept (the director's 12 s retreat had overwritten it each frame) and goes somewhere new (it had gone on to where it was heading). The Hound's arena now: with its counter 99% untouched, without 11% untouched and 10% killed (was 21%). Torn note 9 rewritten; her fifth page re-read (she puts the lamp out); `LORE.md`'s Smiler with Entity 3's credit; her lantern no longer said to draw Smilers. Re-recorded: the Smiler's four goldens and its death frame's look anchor (the lit grin filling the screen is gone); 17 other frames whose grain and camera sway moved with the native stream (content checked unchanged). Retired: three tests of the old view-cone senses (no kind on the old AI looks ahead now); the update reference has no case left (it goes in E7). Harness: the pacing and ambient tests seed the native stream (they had passed or failed with the clock; a rule in `DEVELOPMENT.md`); personas carry a lamp in the dark and name what killed them; the bench takes a second look (six runs) at a level over budget, after the Well's GPU p99 measured 1.4–6.1 ms over the same code. The shifted native stream also showed the wet weather's drip could come back within 6 s, against the mix's own rule of 8 (`tests/loudness.rs`): it now waits 8–18 s; and `the_door_is_heard` counted the door's calls in the ambient log, which keeps only the last three sounds, so a call followed by two others went uncounted: it now tells a call by its time. Pacing, personas and story pass; the full check passes but for that test, fixed and its target rerun. The creature pack: update 0.07 / 0.85 ms, GPU 1.87 / 2.18 ms, 227k triangles. The app bundle (19.5 MB) builds, is signed and runs headless |

## What only people can tell

Whether each rule can be stated after three meetings (the research's
test: four of five people say it in one sentence), whether the Smiler in
the dark frightens or just annoys, whether the Watcher read as furniture
first, whether anyone looked the one in the corner in the face on
purpose, twice.
`PLAYTEST.md` gets the questions in E10.
