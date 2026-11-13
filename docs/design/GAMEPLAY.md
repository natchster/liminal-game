# Gameplay: the descent as a story

What a level asks of the player, why, and the plan that moved it from "walk
toward the arrow until something chases you" to a descent with a reason. The
canon it serves is [`LORE.md`](LORE.md); every change here quotes a rule in
[`PILLARS.md`](PILLARS.md).

(Since 2026-10-27 the descent begins in the Building, three floors of vast
concrete above the rooms, where the challenge is the way down and the
danger far off: [`MEGASTRUCTURE.md`](MEGASTRUCTURE.md). Her trail, camps
and pages run through them as below; her camp there is a cell, and her
page sounds the siren. Since the Building's second pass
(`MEGASTRUCTURE.md` G1–G8, 2026-11-07 to 11-09) a floor there is half as
long, a way of some 85 tiles of walk (the playthrough's bot walks it in a
median 70–90 s), her trail 38–39 marks a floor (`tests/building_floors.rs`),
and so is its self-assembly: her page sounds the siren within 2–4.5 s, the
first floor's first quiet while her camp is there is 60–80 s, and on most
floors a siren comes before the door. Off the way there is more to choose:
every walk that goes nowhere ends on something, rooms off the walks hold
water or a battery, and on the Well a sealed door knocked at may answer
with a resident's word of the way, or, marked by the cleaners, bring the
siren within seconds (`world::knock`; pillar 5: the player chooses when
the dread starts); sealed in a cell, what knocks at it can be answered,
and then the wheel must be held (`world::cell_knock`).)

## The work

How this was planned and done (the analysis, milestones, progress): [`../plan/GAMEPLAY.md`](../plan/GAMEPLAY.md).

## The design

**One sentence:** you follow someone down, by the trail she left, and her
notebook is the story.

**M.** (the author of *M.'s Rules*) came into the Backrooms after **E.**
(Elias), who stopped answering. She followed his marks down and left her own:
sheets of paper folded twice, each with a pencil arrow, from where she arrived
to where she camped and on to the way down. On every floor there is **her
camp**: a bedroll, her things, a lamp still burning, and **one page of her
notebook**. The pages are in order of the descent. They tell her search, hint
at the floor's dangers as a survivor would, and never explain. They end at
**the last camp** on level 13, the first floor below every anomaly. After
that the paper is yours.

The verbs this gives a level, in place of "follow the arrow":

1. **Find the trail.** Her first sheet is within sight of where you arrive,
   but the trail is sparse, has gaps, and gets sparser as she did; deeper
   down it is less and less paper (her marks: [`SECRETS.md`](SECRETS.md)
   1). Deeper down, other trails lie (flat sheets that do not go down).
   Reading the trail is a skill; it is never explained.
2. **Choose the detour.** Her camp is off the straight way. It holds supplies
   and the page. Taking the page wakes the place: her lamp gutters out and the
   calm is cut short. You choose when the dread starts.
3. **Find the door by the place.** The exit has a green light that carries
   down corridors and a sound of its own (the sign's buzz, air moving under
   the door). Crying out for help still works and still costs.
4. **Read.** The notebook (Tab) fills in order; a missed page stays torn out.
   The last line you read stays with you on screen, as the thread.
5. **Reach the bottom of her story.** Level 13 is the last camp and an
   ending; the descent goes on after it, without her paper.

Pillar check: emptiness is a threat again because you can be lost (1); the
camp is "what a place was for, shown by what is left" (1); the trail is
familiar (paper markers, already canon: "Sometimes you find someone else's
trail going the other way") and goes wrong deeper down (2); the exit is
*heard* before it is seen and every sound comes from somewhere (3, 4); no
map, no tutorial, notes hint (4); the page is a player-chosen build-up with
the director's release after it (5).

## What only people can tell

Every gate above holds what can be measured: the door is heard, the trail
can be followed, the pages are in order, the page wakes the place. What
the design is *for* is not measured by any of them, and is the next
playtest round's focus (`PLAYTEST.md`):

- Without a compass, does a fresh player find the door, and how long does
  a floor take? (The recordings' telemetry gives the time; the heatmap
  where they wandered.)
- Is her trail a story or just a guide line? Do they stop at her camp, or
  pass it?
- Does anyone notice the flat sheets before following one, and does the
  dead end feel fair?
- Is taking her page a choice they make (do they wait, or scout the door
  first)?
- At the end, can they say who E. and M. were?
