# Research: your own voice, said under your breath

Gathered on 2026-11-10 for the asides ([`../VOICES.md`](../VOICES.md),
*Your asides*): first-person reactions spoken by the player's own voice
on arriving, at a sudden noise, at what comes and at what the place does
back. Two questions: how a game decides when a character speaks and what
it says without repeating itself, and what this game's canon allows a
voice of yours to say. Every claim links its source or names the file;
what could not be verified is said.

## 1. Choosing a line: Valve's dynamic dialog

**Elan Ruskin, *AI-driven Dynamic Dialog through Fuzzy Pattern Matching*,
GDC 2012** ([slides with notes](https://cdn.fastly.steamstatic.com/apps/valve/2012/GDC2012_Ruskin_Elan_DynamicDialog.pdf),
[GDC Vault](https://gdcvault.com/play/1015317/AI-driven-Dynamic-Dialog-through)).
The system behind the barks of Left 4 Dead, Team Fortress 2 and Valve's
later games. Read in full (the slides' speaker notes, extracted locally).

- **A query is the world as facts; a rule is criteria; a response is the
  line.** "A 'context' or 'fact' is a piece of world state … A query is a
  pile of those glued together … A rule is a list of criteria, all which
  must be true for the rule to 'match'" (slide 49). The event asks; the
  database answers with a line or nothing.
- **Nothing is a fine answer.** "If there wasn't a line for an object,
  nothing plays" (slide 31). A character silent where no line fits is
  better than one with a line forced in.
- **The more specific rule wins.** Many rules can match; "the scoring
  function that worked best for us was the simplest one imaginable – the
  number of criteria in a rule. The more criteria a rule has, the more
  specific it is" (slide 62). Equally specific matches are chosen between
  at random "for variety" (slide 64).
- **Memory is written back by what was said.** "'Then' rules can also
  write context back to the character or the world. This context is
  appended to the next query performed by the character. Creates memory,
  the ability to pick subsequent lines based on what happened before"
  (slide 37); the first barrel seen sets `seen_barrels`, and the second
  barrel matches a different rule (slides 35–36).
- **Variations live inside a response**: several readings of a line,
  "the engine choose[s] randomly between them" (slide 65), cheaper than
  parallel rules.

What we take: an event is a *use* and a *subject* with its facts (the
level, the director's phase, how near); the lines for the subject beat
the general ones for the use; what was said is written to the run's
memory and read by the next choice; no line is a fine answer.

## 2. Barks in practice: one manager, priorities, fatigue

Practical guides, *(secondary)*: [PulseGeek, *Dialogue barks: timing and
cooldown best practices*](https://pulsegeek.com/articles/dialogue-barks-timing-and-cooldown-best-practices/),
[Bugnet, *How to fix Unity dialogue barks overlapping into noise*](https://bugnet.io/blog/how-to-fix-unity-dialogue-barks-overlapping-into-noise),
[StraySpark's Godot dialog system, *Barks*](https://www.strayspark.studio/docs/godot-dialog-system/barks),
an [Audiokinetic Q&A asking for event cooldowns in Wwise](https://www.audiokinetic.com/qa/8231/event-and-voice-cooldown-feature-request),
[A Sound Effect on listening fatigue](https://www.asoundeffect.com/game-audio-immersion/)
*(summary only)*.

- One manager decides every bark (callers never play lines themselves):
  it limits how many speak, ranks by importance, and keeps a minimum gap
  between any two.
- A bark that cannot be said now is dropped, not queued: a backlog plays
  lines seconds after their moment.
- Cooldowns per kind of bark, plus fatigue per line: cooldowns alone let
  the top line repeat. Shuffled variation blocks so a line is not said
  word for word again.
- Middleware does not do this for you (Wwise users ask for event
  cooldowns): it lives in game code.
- Repetition is graded by how irritating it is; some is wanted (a
  recognisable cue), most is not.

## 3. Voicing the player in horror

No developer-facing source was found on how horror games voice their
protagonist's reactions (searches turned up forum posts and
search-engine pages). What was found is anecdotal *(secondary)*: an
editorial that good tension is "knowing when not to use voice acting"
([Bloody Disgusting, *Horror Declassified: a look at voice acting*](https://bloody-disgusting.com/news/3229631/horror-declassified-a-look-at-voice-acting/));
players calling post-action barks silly when they break the tone, and
repetition as the usual complaint (itch.io comments); players who mute
horror games and need what a sound tells shown too (wayline.io
*(summary only)*). Alien: Isolation's prototype gave the alien a voice to
telegraph its intent ([Vice](https://www.vice.com/read/the-horror-and-violence-of-alien-isolation-one-year-on-840)):
a voice carries information, so a voice of ours must not tell what the
canon keeps for play. Nothing here rests on these alone.

## 4. What the canon allows (this repository)

- `LORE.md`, *How canon reaches the player*: "Not allowed: tutorials
  about creatures, maps that explain the place, named SCP numbers in the
  game, any narrator." A voice is heard from a place, or is your own.
- `LORE.md`, *Survival*: the cry is "the only words you say: a man's,
  plain and tired, never brave"; who you are is never said. The asides
  change the first half and keep the rest.
- `PILLARS.md` 4: rules are learned, not told; the screen at rest is the
  place alone. 5: the same scare never repeats soon. 3: silence means
  something, so your own silence while something comes is part of it.
- `CLAUDE.md` invariant 1: a choice made at an event draws from a stream
  of its own, never the per-frame stream (`audio::voice::choose_cry` is
  the model). Invariant 2: your own voice is the one sound with no place
  but you (`.claude/rules/audio.md`).
- `entities::director`: the phases an aside must read (calm, build, peak,
  relax), `hush`, `tension`; the release after a close call is where the
  voice comes back.

## What the design took

The seven rules of the asides (`VOICES.md`, *Your asides*): react, never
explain; once is a reaction, twice a habit (a subject once a run, a use's
chance halving with each line said, a line never twice, a spent use
silent); different every run (the run's seed, a stream of its own, the
memory saved with the run); hold your breath (nothing while something is
after you; the release speaks); one at a time with room between (a gap,
priorities, a reaction said in its moment or never, the specific line
first); heard by nothing but you; only for a person playing.
