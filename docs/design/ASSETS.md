# Assets: what a thing is, why it is here, where it lies

How a prop (an *asset*) of the rooms is made or remade: from what it is and
how it came to be here, through its references, its model and its
photographed surfaces, to what a hand does with it and the rules for where
it lies, which the game reads and agents use to make places. The canon is
[`LORE.md`](LORE.md), the rules [`PILLARS.md`](PILLARS.md), the look
[`STYLE.md`](STYLE.md); how a prop is modelled as the real thing is
[`PROPS.md`](PROPS.md) (*How a prop is remodelled*). The skill that walks it
is `make-asset` (`.claude/skills/make-asset/`).

## The work

How this was planned and done (the brief, the census, the 30% removed,
milestones, progress): [`../plan/ASSETS.md`](../plan/ASSETS.md).

## What earns a place

A kind of thing is in the game only if it earns it (the census of
2026-11-15, [`../research/assets-census.md`](../research/assets-census.md),
which took out the 208 that did not):

- **It belongs where it lies.** Someone who worked, lived or was trapped
  here left it, or the place kept it; a player does not ask why it is
  there, or asks, and the answer is in the canon.
- **It says something.** A use, a person, a story, a scare: a packed
  suitcase says someone meant to leave; a speck on the carpet says
  nothing.
- **It can be seen.** At a crouch, in the lamp, it reads as what it is.
  Smaller than that, it is not a kind of its own.
- **It is one kind.** Two names for the same thing are one too many.
- **Not a stage prop.** Nothing from a theme park, a Halloween shop or
  another genre (a cauldron is kept only because a reviewer found it
  odd but telling; a potion or a sheet ghost is not).

## An asset's dossier

Every asset made or remade by the pipeline has a folder,
`assets/src/props/<name>/` (`cargo xtask asset new <name>` starts one from
the templates), and the game's code that makes it:

| File | What |
|---|---|
| `brief.md` | **What it is**: the real thing, its make and size, what it is made of, its era. **How it came here**: who brought it or left it, when and why, as the canon allows (quoted, with its section); which purposes' areas it fits. **How it fits**: the pillar rule it serves, and what it must never say (`LORE.md`, *How canon reaches the player*). **What a hand does with it**: nothing, or one verb, from the canon (*Within reach*, *Survival*) and the pillars; its sound, how often, once or again. **Where it would be found** (the placement story the rules start from) |
| `references.toml` | Every reference: its URL and page, what it shows (the thing, a place with it in, a maker's sheet, a measurement), its licence class, and notes (sizes, colours, wear, where found). `free` (CC0, public domain, CC BY with its attribution) may be sampled into a texture and is fetched by its checksum (or says `sampled = false`: a document only read); `limited` (all rights reserved, used as fair use) is looked at and measured, never sampled, never committed. The palette (sRGB samples, each with its reference) and the measurements (each with its source) are kept here |
| `placement.toml` | **The rules** for where it lies (below): each with its reason. The game reads them (`crates/game/assets/placement.json`, written by `cargo xtask asset write`) |
| `playtest.md` | **"Why could this be here?"**: what people and the agent playtester answered, beside the brief's story; each difference, and the rule it became |

Its model is a Blender family (`assets/src/blender/props/<theme>/`,
`PROPS.md`), at its real size; its surfaces are painted in the canvas from
its palette and the patterns of its references, or built from a free
photograph as the Building's are (`assets/src/textures/`), never from a
limited one.

## The order

1. **Brief.** What it is, how it came here, how it fits, what a hand does
   with it, where it would be found. `lore-keeper` reads it against the
   canon before anything is made.
2. **References.** Photographs and documents of the thing and of places
   with it in, of both licences; its measurements and palette from them.
   `cargo xtask asset refs <name>` fetches them into
   `target/asset-refs/<name>/` to look at (a free one is checked by its
   checksum).
3. **Model and surfaces.** The real thing at its real size (Blender), its
   surfaces from its palette and patterns (the canvas) or a free
   photograph; the look settle and the material lint pass it as they pass
   every prop.
4. **What a hand does.** Most things do nothing: a thing is interactive
   only where the canon has a hand use it (*Within reach*) or it is what
   the player lives on (*Survival*), and a verb serves a pillar (a sound a
   creature hears, a choice). Nothing says what it is for.
5. **Placement rules**, from the brief's story.
6. **The playtest question.** Frames of it where the rules put it
   (`cargo xtask asset sheet <name>`) go to people (`PLAYTEST.md`) and to
   a fresh agent playtester (the `placement-critic`, `.claude/agents/`) asking
   only: *why could this be here?* Their answers are written beside the
   brief's story in `playtest.md`; where they differ (a reason nobody gave,
   one nobody meant, a place where nobody could say why), the rules change:
   a place without a reason is taken out, a reason people found and the
   brief missed becomes a rule.
7. **Evidence.** A blind review against what it replaced, the census's
   cameras (`visual-review`); the ledger.

## Placement rules

Where a thing may lie, why, and how many: data an agent can read and write
and the game obeys. Each rule is one way it comes to be somewhere.

```toml
# assets/src/props/almond_water/placement.toml
count = [3, 4]             # how many a floor (the game's own number decides within it)
levels = [1, 99]           # from which floor down

[[rule]]
on = "surface"             # floor | surface | wall
of = ["core_filing_cabinet", "place_hall_side_tables"]   # what it stands on (surface): kinds
purposes = ["office wing", "living quarters", "storage hall"]
weight = 3                 # how much likelier than the others
why = "set down where someone worked or rested, and not come back for"

[[rule]]
on = "floor"
near = "wall"              # wall | corner | open
purposes = ["*"]
dark = true                # only where the place has no light of its own
weight = 1
why = "dropped or rolled to the wall"

[never]
purposes = ["empty"]
why = "the emptiness is the content (PILLARS 1)"
```

- **Every rule has its why.** A rule without one fails `cargo xtask asset
  check`; a why is the story a player could tell, not the mechanism.
- **What it stands on is a kind the game places** (`core_<kind>` or a
  placer's name), checked to exist; a purpose is one of `world::purpose`'s.
- **The game places by them after the floor has settled**
  (`game::world::placement_rules`): every spot the rules allow (on a
  thing's top; at a wall's foot or in a corner, a lone pillar no wall; only
  in the dark where a rule says `dark`), in a purpose a rule names and none
  the asset never fits, reachable from where you arrive, never behind a
  barricade, in a wall or in another thing; chosen on a stream of its own
  (`SplitMix64`, never the level's: no floor's draws move), the assets in
  turn, no two of a kind within 6 m (two in one view read as a trail laid).
  A thing its rules allow nowhere is not on that floor. Each keeps the rule
  that placed it, and `what_the_rules_place_keeps_its_rules`
  (`tests/usable.rs`) holds every one to it on 36 floors.
- **For agents making places**: the rules of every asset are one table
  (`cargo xtask asset rules` prints them); a place is dressed from the
  things whose rules allow it, in their counts, for their reasons.

## The samples

Almond water and the battery, the two things every player picks up, were
the first through it (`assets/src/props/almond_water/`,
`assets/src/props/battery/`): a plain 500 ml PET bottle, nothing printed on
it, and an alkaline D cell of a nobody's make, at their real sizes
(`assets/src/blender/props/game/almond_water_and_battery.py`), built by one
builder each for every place that makes one (`core_props::pickups`), still
where they lie. Their rules were asked "why could this be here?" twice:
set down on a cabinet or a box it is obvious every time; on the floor a
bottle reads as dropped when it lies by a wall, a battery only in the dark
or in the halls people walked with torches (their `playtest.md`).
