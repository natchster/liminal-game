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
| 4 | **Insert nothing, get coin.** A lit one's coin return, pressed with nothing in it: a moment later a coin rings down into the cup, and you take it. What hears, within 10 m along the way, hears it. Once a machine, and only to empty pockets (one coin at a time; its verbs: the coin return, then a coin in for the coil your eyes are on, then shoves). | the note was true |
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

## Milestones

| Part | What | Gate |
|---|---|---|
| V0 | Research, this design | the two notes, lore-keeper on rule 4–7's words (later) |
| V1 | **The body**: the Blender parts (batch `v01`), its goods and their printed wrappers, its glass, its light (the light map, the tubes inside, the glow on the floor), its weight (it collides); replaces both old machines | blind review against the old ones; goldens, fixtures and the level sweep; bench |
| V2 | **Its uses**: rules 3–6, the coin in your pockets, the flap | `tests/vending.rs`; personas and pacing unchanged |
| V3 | **The rest of its rules**: 1, 2, 7, 8, 9 | `tests/vending.rs`; the level sweep with a fallen machine |
| V4 | **Its sound**: the hum, the click and thunk, the coin, the motor, the bang, the creak, the crash, the knock (in V5 the flap's lift) | `tests/listen.rs` (a listen test fails with the sound at gain 0), `tests/loudness.rs` |
| V5 | **Reviews and the canon**: blind reviews of the look, `lore-keeper` on its words, `LORE.md`, `PLAYTEST.md`; full check, bench | the reviews, the full check |

## Progress

| Part | Commit | What was done |
|---|---|---|
| V0 | `49e683a` | The research (two notes) and this design |
| V1 | `96637cf` | The body. A glass-front spiral snack machine at the AP Snackshop's size (1.83 × 0.99 × 0.89 m), replacing both old machines (the COLA-Z core prop and `place_dead_vending_machines`, one floor in eight): 21 Blender parts (batch `v01`: the painted cabinet and door, the window's frame, the white liner, six trays with their price channels, three spirals of 7, 10 and 15 turns and a vend motor behind each, the selection column's bezel, 21 keys in 3 × 7, the coin entry and its return button, the bill validator, the coin cup, the delivery bin and its flap, the kick plate's louvres, the feet, the lock, the tube), placed as modelled (`prop_part_as_modelled`), and the goods made in code with their wrappers from one painted atlas (ten bags, eight bars, four pastries by makers that never were, almond water, a card of batteries): pouches sealed and crimped at their ends, bars standing on end in the narrow coils, each a little askew, some coils empty. The price tabs (A1 … F5), the stickers (USE EXACT CHANGE, PUSH, a service number, the CPSC's NEVER ROCK OR TILT), the keys' legends and a 10-character red dot display are painted too. Everything that does not move is merged by material: a machine is about twenty draws (the COLA-Z alone had been 182). Painted True Grit Black (twice as often), Dark Gray, Dove Gray or dark brown; leaning a hair forward and to its lock side as the makers set it; 5 cm off its wall; its cord along the floor behind. Six in ten are on: one cool-white tube across the top inside the door lights the goods through a soft spot down into the cabinet (so nothing outside it is lit through its walls; the bottom shelves dimmer), its glass glows onto the floor (a point light) and its tile is lit in the light map (`VEND_LIGHT`); a dead one is dim inside, rusted along the door's bottom, and half of them have OUT OF ORDER taped over the glass. It is solid (`hits_vending` in `circle_hits_wall`); its coils and fine detail are not drawn beyond 8 m. Its build takes one draw from the build's stream and the rest from a stream of its own (`with_own_stream`): what it is made of no longer moves which wall it stands against or anything placed after it (before, every change to it had). Blind review (two fresh reviewers, both orders, 23 views of 12 floors, the same poses in both builds): the new machine preferred in all 21 views that showed both, both reviewers alike ("could be standing in a real corridor" against "a toy prop"), two ties (views of a wall; fixed: `vending_views` now needs a line to its face). Caveat from both: the lit interiors are bright and colourful next to the rooms' muted palette. Found on the way: the core props' fixtures (recorded from the HTML version) had no recorder; `core_props_head` `rerecord` re-records them from the current code, and on unchanged code reproduces every value the tests read. `build_pile`'s colours were random integers between two hex values (any colour at all, lime and orange) and a cross-stitch's fabric too bright: both exposed by the shifted draws in the material lint, both fixed; the door's listen test measured the door against a creature's drone where one stood near the spot (creatures now taken off that floor for it; the door unchanged, +15.1 dB at the spot that failed); `tests/props.rs` held the HTML version's 2,546 parts as all there are (now: every recorded part as recorded, the count from the parts table). Re-recorded: the 14 rooms levels of the reference set, `world_build` (one mark fewer, the dead machines' placer), `world_queries` (that placer's theme), the core props' fixtures, 15 goldens and one look anchor (props moved with the shifted draws: a partition no longer in front of `7_L5_overgrown_atrium`). Bench: within budget, no slowdown; draw calls fewer on most floors of the rooms (−10 to −453), triangles up where a machine is near (+51 k at most), GPU p50 within 0.1 ms |

| V2 | `5d0b9c7` | Its uses (`core_props/vending/rules.rs`): rules 3–6, one verb at a time on four interactables at its face (`vending_verb`: COIN RETURN, INSERT COIN, ROCK IT, REACH IN). Rule 3: within 2.5 m, the ray of your eyes through its glass picks a coil (the tray under the point, the nearest coil on it), and the display shows its code and price (`B1    1.25`) or SOLD OUT; otherwise HAVE A NICE DAY runs across it, three characters a second. Rule 4: the coin return pressed with nothing in it rings a coin down into the cup 1.4 s later (`A COIN IN THE CUP`; the coin is a pocket, `Player::coins`, saved with the run) and what hears within 10 m along the way hears it (`vending_noise`: a rough guess near the machine, not where you are; awareness 0.6); once a machine. Rule 5: a coin in for the coil under your eyes (CREDIT .25): the coil is lifted out of the merge and turns once in 2.6 s, its goods move forward a place, its front good comes off the spiral's end and tips until its top meets the glass, hanging; nothing falls; MAKE ANOTHER SELECTION runs across; then 40 s of silence: the place hushes (the director's hush, 0.85) and nothing within 10 m of it hears your steps (`vending_muffles`, in `perceive`). Rule 6: each shove leans it toward you on its front edge and drops it back with a bang heard 20 m along the way (awareness 0.8); on its own count (2 to 4, fixed when it was built) what hangs falls into the bin; reach in (the flap swings) and take it: almond water to your pockets, two batteries, or a snack eaten at once (+6 nerve; `… IT TASTES OF NOTHING.`). Its first sounds are simple one-shots from where it stands (the lever, the coin's fall and ring, the coin going in, the catch, the bang, the thunk in the bin, the flap); V4 makes them. `tests/vending.rs` (5): it reads your eyes; insert nothing, get coin (once, and heard); insert coin, get nothing (hangs, nothing falls, the coin kept, the silence near it and only near it, no draw from the build's stream); what hangs is rocked loose and heard 14 m off, and taken; it is solid and the same machine on every build. Broken on purpose (no bang's noise, no silence), the tests fail. `--vending-demo hang|bin` plays every lit machine to a state for `--views-at` frames |
| V3 | `527bd6c` | The rest of its rules. Rule 1: each lit machine keeps a rhythm of its own (on 40–60 s, dark 6–9 s, from a point in it of its own, fixed when built): a click and it is dark (its tube grey, its spot and its glow out, its inside as dim as a dead one's: the colours the look settled, kept the first time, at 0.37; its display off), out of the light map (`vending_light`, the map rebuilt at each change), a thump and it is back. Rule 2: while a creature after you (`Mode::after_you`) or the director's stalker is within 16 m of it, it is dark out of its rhythm, until 3 s after it has gone. Rule 7: with nothing hanging, the second shove creaks (`CREAK_ON`), the third (`FALL_ON`) passes its balance: it falls on its face about its front edge in 0.9 s, slow off its feet then all at once; the crash is heard all over the floor (`vending_noise` at any reach: whatever hears comes to where it fell); its glass is gone against the floor, it is dark for good, and it lies solid where it fell (`hits_vending`: its face to its top, before its wall; where it stood is open); its almond water and batteries lie by it to be taken (`vending_spill`, two of each at most, as the level's pickups) among bags and bars thrown across the floor (`vending_scatter`, from its own seed); if you stood before it you are thrown down (`knock_down`, 22 nerve at most, never below `KNOCKED_LEAVES`: it never kills) and out from under it. Rule 8: once nobody has looked at it for 20 s, every coil it lost goods from is as full as it was built (`Machine::restock`, the goods made anew from its own stream). Rule 9: a dead one you looked at within 2.5 m knocks twice from inside its glass when you turn your back within 5 m, once, and a hand is pressed to the inside of the glass (`handprint_tex`, hidden till then); half the dead ones have something already hanging in them (someone else's coin). `tests/vending.rs` (10): its rhythm; dark when something comes, and back after; never rock or tilt (the creak, the fall, the crash heard 60 m off, thrown down and alive, solid where it lies and not where it stood, out from under it, its goods spilled, dark for good); it restocks unseen and not while you look; something inside the dead ones. Broken on purpose (no restock, no dark when something comes, no fall), the tests fail. `--vending-demo fall|dark|print`. Found on the way: as a level began, a lit machine whose rhythm put it dark clicked, and the click drew from the frame stream (two goldens' film grain moved): a machine now begins as it is, silently; and a far machine's click every minute moved every later ambient event of its floor (a door slam fell into the loudness test's minute): its sounds are made only within 30 m of you (`HEARD_WITHIN`), its fall anywhere |
| V4 | `50da029` | Its sound. Its hum (`audio::soundscape::MachineHum`, made the first time a lit machine is heard, one voice slot each, stopped with its level): its tube's magnetic ballast at 120 Hz and a thin series above it, the buzz of the arc chopped at 120 Hz, a transformer's 60 Hz under it, and its fan's soft air, from its top behind the glass; it holds while the machine is lit and not silent (rule 5), sinks in the director's hush, and is gone in its dark beats (rule 1), when something comes (rule 2) and when it has fallen. Its one-shots made: the coin let go, ticking down its chute and ringing once in the steel cup (inharmonic partials, the higher gone first) and rattling there; the vend motor's whine as a held note (a geared motor: its fundamental, its gears, the spiral ticking past its stops); a shove's thud with the cabinet's sheet ringing, the glass in its frame and the goods shifting; and the click, the thump, the creak, the crash and the knock. The ambience's old "vending hum" (a 30 Hz throb now and then near a powered machine, from before this rebuild) is a coil turning by itself (`vending_phantom`: the motor's whine from a lit machine within 12 m, and its spiral turning, nothing coming forward), on the same cadence, so the frame stream's draws are as they were. `tests/listen.rs` `the_vending_machine_is_heard`: 1.5 m before a lit machine its hum's band is 17 dB over the room when lit against dark (with the hum's gain at 0 the test fails: +1.9 dB), the coin's ring 34 dB over its band and a shove's bang 17 dB over the room; the whole mix there 11 dB over the dark room (a real machine idles about 15 dB over a quiet room). Loudness holds unchanged (nothing re-recorded). No person has listened to it |
| V5 | (this commit) | Reviews and the canon. Three `lore-keeper` rounds (fresh reviewers, the brief verbatim), what each found fixed. Where it stands: offices, halls, the Poolrooms, office wings and party rooms (`VENDING` home), never a zone left bare (`Home::skip_bare`, pillar 1), not at all on a floor with none of those; none on the Lobby (level 4), the Showroom (it stayed lit when the store's lights went out) or the Borrowed House (its light is the nest's). Rule 2's dark comes and goes without its click and thump: the tell. Rule 5's silence is the machine's: its hush whole within 5 m, gone at 10 m (it had hushed the whole floor), its rhythm silent in it, no coil turning by itself in it. Rule 8 now fills in its dark (it had filled "when nobody is looking", the Watcher's rule again): as its light comes back. Rule 9 no longer knocks (the canon's sixth knocking): stand within 2 m before the glass of a dead one with something in it (half of them), whichever way you face, with nothing between (never through its wall), 1.5 s, and once its flap lifts inward by itself, is held up 1.5 s on the dark of its bin and let fall with a hinge's squeal and a clack (`Sfx::Lift`), while a hand print comes up on the inside of its glass. What is printed: its almond water plain (the floors' bottles have no label), COUNTY FAIR is HARTWELL, the warning label the real one word for word, and `LORE.md` / `PILLARS.md` bound the exception (a thing's own fixed printing, never about the place); a snack's line one of four in the pickups' NOUN — PHRASE (`snackEat`). Its back: louvres pressed into the cabinet (batch v01) and a riveted data plate, for when it lies on its face. `LORE.md`'s section rewritten to match the code (coins one at a time from lit machines, who can shove it, the fall's throw only before it). `tests/vending.rs` (12): rule 8 in its dark, rule 9 (not in passing, whichever way you face, once, not through its wall, not in an empty one), rule 2's silent dark against its beat's click, the hush only round it, no coil turning in the silence, where it stands (two tests); each broken on purpose fails. `--vending-demo inside` for frames |
## What only people can tell

Whether the machine reads as a real one in a dead corridor or as a game's
prop; whether its rules are found by playing (the note, the label, the
lean, the click) or never; whether the silence it sells is relief or dread.
