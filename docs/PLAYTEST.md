# Playtesting

Whether the game frightens is decided here, by people who have not played
this build before. The developer and the agent are both used to the scares
(Until Dawn's team brought in new testers for this reason), and agents
cannot judge fear or sound. Everything else in the tooling is a proxy for
what this page measures.

## When

At each milestone that changes how the game feels (creatures, pacing,
sound, the look), and before any release. **Five or six fresh players** per
round find most problems (Bromley); fix them, then test again rather than
adding players to the same round.

## Before the session

1. Build the version under test: `cargo xtask bundle` (→ `dist/Backrooms.app`),
   and start it with `dist/backrooms --record playtests/<date>-<alias>.rec.json`
   (with the player's consent): the session's inputs, a telemetry row a
   second and the director's events, written on this machine only. A
   recorded session starts fresh (no saved progress), like a new install.
   For an A/B round, build both versions and label them neutrally
   (`build-1`, `build-2`); the player never learns which is new.
2. Decide the **focus** (one or two questions, e.g. "does the Hound's
   approach feel announced?", "is the Lobby too empty or not empty enough?")
   and what you will record. Write both into the session file before the
   player arrives.
3. Copy `playtests/TEMPLATE.md` to `playtests/YYYY-MM-DD-<player>.md`
   (a first name or an alias; no other personal data).
4. Headphones, a quiet room, the lights low. Sound is half the game.
5. **Listen first, once per build that changed sound** (no player needed):
   `cargo test --release -p backrooms --test listen -- --nocapture`, then
   play `target/listen/*.wav` on the same headphones. The Building's siren
   (`siren_*`) and the loudness reference have not yet been heard by a
   person (`docs/MEGASTRUCTURE.md`, B3 and B5): write down whether the
   siren reads as a warning and whether anything is too loud, and only then
   re-record loudness (`cargo test --release -p backrooms --test loudness --
   --ignored rerecord`).

## The session (about 30 minutes)

1. **Brief** (one minute, read it out): "This is a horror game about being
   lost in an endless building. There is no wrong way to play. I won't help
   unless you are completely stuck. Think out loud if you like." Nothing
   about creatures, rules or controls beyond the controls card (the
   README's Controls table, printed; the screens no longer list the keys).
   Watch for: whether they let go of WASD or the mouse to use something,
   whether they find the notebook (Tab) and the glance (X) on their own,
   and whether they open Settings.
2. **Observe silently** for 20 minutes. Write down, with the time: where they
   hesitate, what they say, when they flinch, laugh, stop or turn round,
   deaths, and when they look bored. Step in only if they are truly stuck
   for over two minutes.
3. **Debrief** with the questions below. Ask, don't lead; write down their
   words.
4. **Tension trace** (RankTrace): play the session back with
   `dist/backrooms --replay playtests/<file>.rec.json` and have the player
   press **↑** when tension rises and **↓** when it falls, as it plays (Escape
   stops). The marks go to `<file>.rec.json.trace.json`. Only the *direction*
   of change matters; it predicted skin-conductance responses better than
   ratings.

### Debrief questions

Comprehension (did the design communicate?):

- What did you think was happening when the lights failed / the sound
  stopped?
- What were the creatures you met? What did each one want? How did you
  try to deal with it?
- What did you think the notes were for?
- Where did you think the exit was, and why?
- Who left the paper on the floor? Did you follow it? Did you trust all of
  it? (There are two kinds; never ask about them by name.)
- What happened when you took the page at the camp? Did you choose when?
- What was the notebook about? Who were E. and M. to each other?

Hands and screen (did the controls and the screens get out of the way?):

- Could you run, look round and use things at the same time? When did
  your hands get in the way?
- What did the screen tell you while you played? Was there something you
  wanted to know that it did not show? (Never name the glance key.)
- One word each for the title, the pause and the death screen.
- Did you change a setting? Which, and why?

Experience (how did it feel?):

- When were you most afraid? What made it stop?
- Was there a moment you were bored? Where?
- Did anything feel unfair? Did anything look fake or broken?
- One word for the place.

The Building (levels 1–3; ask until a round has answered them, and never
name the siren's meaning, the cells, the fog or the figure first):

- What did you think the place was, on the first floor? Where did you think
  the way down was, and when did you first see it?
- Did finding the way down feel like a puzzle or a chore? Where were you
  lost, and did you mind?
- What did you do when the siren went, the first time? What did you think
  it meant? What made you think so?
- What was it like to wait in the little room while something knocked?
  Too long, too short? Did you open the door?
- Did you see anyone up there? (Only if they say yes: where, what did it
  do, what did you do?)
- When you came through the small warm door into the yellow rooms, what
  changed?

The creatures (`docs/ENTITIES.md`; ask until a round has answered them,
and never name a creature, its rule or its counter first):

- For each thing you met more than once: what does it notice you by, and
  what keeps you safe from it? (Say it in one sentence. The design holds if
  four players in five can, after three meetings.)
- The grinning one in the dark: when you put your light out, were you
  afraid, or only annoyed? Did the dark itself wear you down?
- The covered shape among the covered furniture: when did you first know
  it was not furniture?
- Someone standing in a corner with its back to you: what did you hear
  near it, and from how far? Did you look at its face? On purpose? Twice?
- The one reading as it walked: what did you do when it looked up? Did
  dropping paper feel like your idea?
- Out of a window: did you see anyone out there? When did you stop
  looking out, and why? If it came to the glass, what did you do?
- Did anything you had beaten before come at you differently later? What
  changed, and when did you notice?
- When something caught you, did it matter? Did you feel the game was
  fair about when it could kill you?
- Did your light run out? When, and what did you do?
- On a floor you crossed quickly, did you meet anything? Did you want to?
- The tall shop staff, the shadows in the borrowed rooms, the man in the
  chair: one word for each.

The vending machine (`docs/VENDING.md`; ask until a round has answered
them. Ask the first two before any other, and never name its coin, its
silence, its rhythm or its fall before the player has: the questions
after the first two are for once they have, and skip any they have not
met):

- Did you notice a vending machine? Was it lit? Did it look real, or like
  a game's? (Anything fake about it?)
- What did you try with it? What happened? What did you think it wanted?
- Did it ever go dark? What did you think it meant, the first time, and
  later?
- If it went quiet after you used it: what was that like? Did you use it
  for anything?
- Did you shove it? How many times? Why did you stop (or not)?
- Did you see one that was dead? Did you go close? What happened?
- Did anything about it make you uneasy? When?

One questionnaire item only, the miniPXI's **Enjoyment** ("I had a good
time playing this game", 1–7). Full questionnaires (GEQ, PXI subscales) do
not hold up well enough for a handful of players.

### A/B rounds

Play both builds in alternating order across players (half A first, half B
first) and ask the player to **rank** them ("which was scarier / more real /
fairer?"), not to rate each one. Score the rankings with
`cargo xtask ab score` style counting: a preference counts only when it
does not flip with the order.

## After the session

0. **Maps:** `cargo xtask heatmap playtests/*.rec.json` draws, per level,
   where the players spent their time (yellow) and where the chases were
   (red), with the deaths and peaks per level; a round's sessions add up.


1. Fill in the session file the same day: observations, answers, the
   tension trace (times of up/down), deaths and quits.
2. **Feed the tools**:
   - each look judgment the player made about a frame or a place is a human
     verdict for calibration: add it to
     `crates/backrooms/tests/review/verdicts.jsonl` (`"reviewer": "human"`)
     and, where it disagrees with the `art-critic`, to
     `.claude/agent-memory/art-critic/MEMORY.md` with the reason;
   - problems become entries under *Open* in `QUALITY.md` and, where a
     gate could have caught them, a new test, lint or band (the steering
     loop);
   - the ledger entries the session covered get `last_human_review`.
3. After the round: a short summary in `playtests/` (what most players did,
   said and felt; what changes), and the grade changes it justifies in
   `QUALITY.md` (*History*).
