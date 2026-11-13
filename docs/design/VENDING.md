# The vending machine

What the vending machine of the rooms is, why, what it does and how it is
learned, and the plan that rebuilds it from scratch: as a real machine you
could photograph in a dead office corridor, and as a thing with rules of its
own that matter to play. The canon it extends is [`LORE.md`](LORE.md)
(its torn note, *The Vending Machine*); the pillars it serves are
[`PILLARS.md`](PILLARS.md) 2, 3 and 4 above all; the research behind it is
in [`../research/vending-machines.md`](../research/vending-machines.md) (the real
machines) and [`../research/vending-horror.md`](../research/vending-horror.md)
(the uncanny ones), summed up in [`SOURCES.md`](../research/SOURCES.md) section 12.

## The work

How this was planned and done (the brief, the analysis, milestones, progress): [`../plan/VENDING.md`](../plan/VENDING.md).

## What the research says

Two notes, in [`research/`](../research/README.md). What they changed:

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
  the labels that campaign put on them read "Never rock or tilt. Vending
  machine will not dispense free product." and "Never rock or tilt.
  Machine can fall over and cause serious injury or death." (the game
  prints the second: here rocking does dispense).
- **No talking, no jump scares, no gore** (SCP-6593, the RE4 remake: a
  voice becomes a character and wears out); a prop, never a creature
  (Control's machine that runs away is the line not to cross). The uncanny
  is rationed.

## The design

One machine, a glass-front spiral snack machine, on most floors of the
rooms where a zone would have one (offices, halls, the Poolrooms; the
office wings and party rooms), never in a zone left bare (the `empty`
purpose: pillar 1), and on a floor with none of those, not at all
(`VENDING` in `core_props/mod.rs`). None on the Lobby (level 4: its
emptiness is the point, as the anomaly plan has it), nor in the Showroom
(it would stay lit when the store's lights go out) or the Borrowed House
(its light is the nest's). Six in ten are still on. Its rules, each held by
a test (`tests/vending.rs`):

| # | Rule | What you learn |
|---|---|---|
| 1 | **It is still on.** It throws its light (the light map) on the floor in front of it, and hums. Every 40–60 s (its own period) its ballast drops out: a relay click, 6–9 s dark and silent, a thunk, and it is back. | its rhythm, and that a lit one is awake |
| 2 | **It goes dark when something comes.** While a creature that is after you, or sent toward you, is within 16 m of it, it is dark and silent out of its rhythm, and comes back a few seconds after it has gone; it goes out and comes back without its click and thump (its own beat has them). | a dark machine is a warning, unless you heard its click |
| 3 | **It reads your eyes.** Within 2.5 m, looking through its glass, its display shows the code and the price of the coil you look at, before you touch it; otherwise HAVE A NICE DAY runs across it. | it knows what you want |
| 4 | **Insert nothing, get coin.** A lit one's coin return, pressed with nothing in it: a moment later a coin rings down into the cup, and you take it (unless you wear the holed coin put away off the way: then nothing comes, `SECRETS.md` 5). What hears, within 10 m along the way, hears it. Once a machine, and only to empty pockets (one coin at a time; its verbs: the coin return, then a coin in for the coil your eyes are on, then shoves). | the note was true |
| 5 | **Insert coin, get nothing.** Your coin in, the coil you look at turns once: its front good comes forward and catches against the glass, hanging. Nothing falls. MAKE ANOTHER SELECTION. Then it goes silent for 40 s: its hum stops, its rhythm's click and thump are not heard and no coil turns by itself, the ambience round it hushes (the director's hush, whole within 5 m of it and gone at 10 m: the machine's place, heard where you stand as any sound's level is; above 0.6 the flicker waves and glitches hold too), and while you are within 10 m of it (straight, through walls) nothing anywhere hears your steps. Creatures sound as ever. | a coin buys a silence, where it stands (the place holds still round it) |
| 6 | **What hangs can be rocked loose, and that is heard.** Each shove bangs (what hears, within 20 m along the way, comes to look) and leans it further; what hangs falls on its own count of shoves (two to four); reach into the flap and take it. | it costs noise |
| 7 | **Never rock or tilt** (its side's label). With nothing hanging, the second shove creaks (`CREAK_ON`) and the third (`FALL_ON`) passes its balance: it falls forward: a crash heard all over the floor (whatever hears comes to where it fell), its glass bursts, its goods spill across the floor, and if you stand in front of it you are thrown down (never to your death). It lies where it fell. | do not attempt the reverse; it costs more than you have |
| 8 | **It fills in its dark.** A lit one's coil you emptied is full again when its light comes back from a dark beat (rule 1 or 2), and not before, however long you look away (and not while something still hangs in it). | something else uses it, in the dark |
| 9 | **Something is inside the dead ones**, half of them (a bit of its seed, `State::inhabited`: not every dead one, so the scare does not come every other floor). Stand within 2 m before a dead machine's glass, with nothing between, whichever way you face, for a moment (1.5 s; standing, not looking: the canon's gaze rules are many) and, once, its flap lifts from inside: slowly, held up on the dark of its bin, let fall. While it is held a hand print comes up slowly on the inside of its glass, and stays. | something in there wanted to see you |

What it sells: almond water (the bottom tray's last two coils), a card of
batteries (one narrow coil), and snacks (bags, bars, pastries: a snack is
eaten when you take it, a little nerve back). A dead machine takes no coin
and shows nothing, but rocks, falls and holds its own stock, and now and
then something already hangs in it (what someone else's coin bought).
Its almond water is plain, nothing printed on its bottles, as on the
bottles of the floors; its makers' names (MIDWAY, HARTWELL, HOLLIS,
BAYSIDE, SUNNYVALE) are nobody's. Its warning label is the real one word
for word (the industry's labelling campaign with the CPSC, `research/vending-machines.md`:
NEVER ROCK OR TILT. MACHINE CAN FALL OVER AND CAUSE SERIOUS INJURY OR
DEATH.), though its fall never kills: what a real thing of its kind
carries word for word is dressing, not the place speaking (`LORE.md`,
*How canon reaches the player*).

**Fair.** It never comes after you and never kills: a fall throws you down
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

## What only people can tell

Whether the machine reads as a real one in a dead corridor or as a game's
prop; whether its rules are found by playing (the note, the label, the
lean, the click) or never; whether the silence it sells is relief or dread.
