# The vending machine

What the vending machine of the rooms is, why, what it does and how it is
learned, and the plan that rebuilds it from scratch: as a real machine you
could photograph in a dead office corridor, and as a thing with rules of its
own that matter to play. The canon it extends is [`LORE.md`](LORE.md)
(its torn note, *The Vending Machine*); the pillars it serves are
[`PILLARS.md`](PILLARS.md) 2, 3 and 4 above all; the research behind it is
in [`research/vending-machines.md`](research/vending-machines.md) (the real
machines) and [`research/vending-horror.md`](research/vending-horror.md)
(the uncanny ones), summed up in [`SOURCES.md`](SOURCES.md) section 12.

## The brief (2026-10-29)

"Rebuild vending machine to make it more photorealistic and interactive in
creepy and interesting way."

## The analysis

What the machines were, read from the code and from frames
(`--views-at`, seed 4242, level 4):

- **Two machines, neither of them real.** A core prop, `build_vending`
  (`core_props/builders.rs`), stood on every floor of the rooms: a red box
  1.05 × 2.05 × 0.65 m under a flat cream header that glowed like a sign,
  "COLA-Z" in Impact, five rows of jar-shaped cans behind a pane with a
  painted-on diagonal shine, a keypad of round chrome studs. A placer,
  `place_dead_vending_machines` (`placers_p30`, one floor in eight), stood
  a black box beside it: a dark plane for a window, nine square buttons.
  Both read as toys at a glance (the frames in `QUALITY.md`'s evidence).
- **No light and no weight.** The "lit" one lit nothing: its header was an
  unlit white material, the room around it as dark as before. Props do
  not collide: you walked through both.
- **Its use was a dice roll.** One dead machine in five was "still
  powered": E with a coin you never needed rolled 40% almond water, 25% a
  battery, 25% "STUCK. IT KEEPS YOUR COIN." (with a retry 8 s later), 10%
  a "mercury soda" that made you 30% faster for 20 s. Nothing could be
  learned from it (pillar 4: rules are learned), it cost nothing, and its
  outcomes were drawn from the level's build stream while you played (a
  per-frame draw from the build's stream: invariant 1).
- **The canon had a rule the machine did not keep.** Torn note 15, *The
  Vending Machine*: "Insert coin, get nothing. Insert nothing, get coin. Do
  not attempt the reverse. It costs more than you have."

## What the research says

Two notes, in [`research/`](research/README.md). What they changed:

- **A real machine, at its real size.** The glass-front spiral snack
  machine of the 1990s–2000s office corridor (Automatic Products'
  Snackshop 113/123, National's 167), 1.83 × 0.99 × 0.89 m: a painted steel
  cabinet (True Grit Black, Dark Gray, Dove Gray), a full-height door with
  a 1133 × 683 mm tempered window on the left and a 25 cm selection column
  on the right (a red LED display, a 3 × 7 keypad of 15 mm keys, the coin
  entry and its return, a bill validator, the coin return cup), trays of
  spiral coils (five wide for bags, more and narrower for bars), each with
  its price tabs, one cool-white tube across the top inside the door (the
  bottom shelves in gloom), the delivery bin under the window behind a
  push flap; the lock-side front leg set lower, so it leans; 1–4 in off the
  wall. Its words are few and fixed: "Have a nice day", "Use correct
  change", "Make another selection", "Credit". What makes a modelled one
  read as fake: toy proportions, perfect rows, clean glass, even light, a
  keypad at the wrong scale, standing plumb.
- **The light in the dark.** In fiction the lone lit machine is the
  image (Eiji Ohashi's *Roadside Lights*; Backrooms Levels 187 and 334): the
  only living light, which leads you and draws what else is there.
- **Price everything in what the game already has** (noise, light, nerve,
  time): Lethal Company's counter, where noise is both how you use it and
  what comes for you, with a warning before it bites. Players called it
  unfair when they missed the warning: **warn before you punish**.
- **Rules, not dice.** A random table cannot be learned; what varies is
  fixed per machine (Fallout's save-scummed machines).
- **The real danger is the reverse.** The US CPSC counted 37 deaths and
  113 injuries (1978–95) from people rocking machines for free product;
  the industry's label reads "Never rock or tilt. Vending machine will not
  dispense free product."
- **No talking, no jump scares, no gore** (SCP-6593, the RE4 remake: a
  voice becomes a character and wears out); a prop, never a creature
  (Control's machine that runs away is the line not to cross). The uncanny
  is rationed.

## The design

One machine, a glass-front spiral snack machine, on most floors of the
rooms where a zone would have one (the Lobby, offices, halls, the Poolrooms;
the office wings and party rooms). Six in ten are still on. Its rules, each
held by a test (`tests/vending.rs`):

| # | Rule | What you learn |
|---|---|---|
| 1 | **It is still on.** It lights its corner (the light map) and the floor in front of it, and hums. Every 40–60 s (its own period) its ballast drops out: a relay click, 6–9 s dark and silent, a thunk, and it is back. | the rhythm of the one live thing on the floor |
| 2 | **It goes dark when something comes.** While a creature that is after you, or sent toward you, is within 16 m of it, it is dark and silent out of its rhythm, and comes back a few seconds after it has gone. | a dark machine is a warning, unless it is only the click |
| 3 | **It reads your eyes.** Within 2.5 m, looking through its glass, its display shows the code and the price of the coil you look at, before you touch it; otherwise HAVE A NICE DAY runs across it. | it knows what you want |
| 4 | **Insert nothing, get coin.** The coin return, pressed with nothing in it: a moment later a coin rings down into the cup, and you take it. What hears, within 10 m along the way, hears it. Once a machine. | the note was true |
| 5 | **Insert coin, get nothing.** Your coin in, the coil you look at turns once: its front good comes forward and catches against the glass, hanging. Nothing falls. MAKE ANOTHER SELECTION. Then it, and the place, go silent for 40 s: its hum, the room's sound, and your steps: nothing within 10 m of it hears you walk. | a coin buys a silence (and leaves you deaf) |
| 6 | **What hangs can be rocked loose, and that is heard.** Each shove bangs (what hears, within 20 m along the way, comes to look) and leans it further; what hangs falls on its own count of shoves (two to four); reach into the flap and take it. | it costs noise |
| 7 | **Never rock or tilt** (its side's label). Two shoves past the last thing that fell (three when nothing hung) it creaks, then on the next it falls forward: a crash heard all over the floor (whatever hears comes to where it fell), its glass bursts, its goods spill across the floor, and if you stand in front of it you are thrown down (never to your death). It lies where it fell. | do not attempt the reverse; it costs more than you have |
| 8 | **It restocks when no one is looking.** A coil you emptied is full again once you have not looked at the machine for 20 s. | something else uses it |
| 9 | **Something is inside the dead ones.** Once, after you have looked at a dead machine up close and turned your back, something knocks twice on the inside of its glass; look back and there is a hand print on the inside of the dust. | familiar, and wrong |

What it sells: almond water (the bottom tray's last two coils), a card of
batteries (one narrow coil), and snacks (bags, bars, pastries: a snack is
eaten when you take it, a little nerve back). A dead machine takes no coin
and shows nothing, but rocks, falls and holds its own stock, and now and
then something already hangs in it (someone else's coin).

**Fair.** It never moves by itself and never kills: a fall throws you down
and leaves you `KNOCKED_LEAVES` of nerve. Every price is in what the game
already has (noise for what hears, light for what sees, nerve), and every
punishment is warned twice (the label on its side and the lean for a fall;
the click for the dark). Its varying choices (its rhythm, its count of
shoves) are fixed per machine, from the level's seed and where it stands.

**Determinism.** It is built in the level's build (its choices from the
build's stream); what it does as you play draws from the native stream or
from its own seeded stream (`VEND_SALT`), never from the build's.

**Performance.** One mesh per material (about twenty draws a machine,
whatever it holds); its coils and goods are not drawn beyond 9 m.

## Milestones

| Part | What | Gate |
|---|---|---|
| V0 | Research, this design | the two notes, lore-keeper on rule 4–7's words (later) |
| V1 | **The body**: the Blender parts (batch `v01`), its goods and their printed wrappers, its glass, its light (the light map, the tubes inside, the glow on the floor), its weight (it collides); replaces both old machines | blind review against the old ones; goldens, fixtures and the level sweep; bench |
| V2 | **Its uses**: rules 3–6, the coin in your pockets, the flap | `tests/vending.rs`; personas and pacing unchanged |
| V3 | **The rest of its rules**: 1, 2, 7, 8, 9 | `tests/vending.rs`; the level sweep with a fallen machine |
| V4 | **Its sound**: the hum, the click and thunk, the coin, the motor, the bang, the creak, the crash, the knock | `tests/listen.rs` (a listen test fails with the sound at gain 0), `tests/loudness.rs` |
| V5 | **Reviews and the canon**: blind reviews of the look, `lore-keeper` on its words, `LORE.md`, `PLAYTEST.md`; full check, bench | the reviews, the full check |

## Progress

| Part | Commit | What was done |
|---|---|---|
| V0 | (this commit) | The research (two notes) and this design |

## What only people can tell

Whether the machine reads as a real one in a dead corridor or as a game's
prop; whether its rules are found by playing (the note, the label, the
lean, the click) or never; whether the silence it sells is relief or dread.
