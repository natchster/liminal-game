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
  another genre that the canon does not keep: a carved pumpkin gone soft
  in a shrine, a cauldron of stale water are the place's (`LORE.md`, *What
  still burns*); a potion or a sheet ghost is not.

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
# assets/src/props/almond_water/placement.toml (as it is)
count = [1, 3]
levels = [4, 99]

[[rule]]
on = "surface"
of = ["core_filing_cabinet", "core_cardboard_stack", "place_school_desks", "place_hall_side_tables", "place_wooden_crates", "place_cardboard_boxes"]
purposes = ["office wing", "living quarters", "storage hall", "maintenance", "play area", "parlour", "party room"]
weight = 4
why = "someone working or resting here set it down on what was at hand, meaning to come back for it"

[[rule]]
on = "floor"
near = "wall"
purposes = ["office wing", "living quarters", "storage hall", "maintenance", "service tunnels", "play area", "parlour", "party room", "overgrown atrium", "empty"]
weight = 2
why = "dropped where someone sat with their back to the wall, or rolled to it"

[[rule]]
on = "floor"
near = "corner"
purposes = ["office wing", "living quarters", "storage hall", "maintenance", "service tunnels", "empty"]
weight = 1
why = "pushed into a corner out of the way by someone who slept there"

[never]
purposes = ["washrooms", "pool hall", "shrine", "showroom"]
why = "nobody keeps their drink where they wash, a shrine is not for it, and the Showroom is spotless"
```

- **A rule may hold only in the dark** (`dark = true`): where the place
  has no light of its own (the battery's floor rules: a cell dropped while
  a torch was changed by feel).
- **Every rule has its why.** A rule without one fails `cargo xtask asset
  check`; a why is the story a player could tell, not the mechanism.
- **What it stands on is a kind the game places** (`core_<kind>` or a
  placer's name), checked to exist; a purpose is one of `world::purpose`'s.
- **`on` is `floor` or `surface`.** Hung on a wall is not placed yet: the
  check and the game refuse it.
- **`levels` start at 4 at the least.** The rules place on the rooms'
  floors; the Building's (1-3) place their own things.
- **`count` is how many a floor, made by its maker.** The rules never make
  a thing: whatever makes it (for the pickups, `core_props::pickups`)
  decides how many, and the rules drop any past the most. Where the floor
  allows anywhere, at least the least are there (the rules test holds it).
- **What the game places by them** is any thing made with `user.s["by"]
  = "rules"` and the asset's name as its `kind`: one pass for every asset,
  each on a stream seeded by its name. A thing marked with no rules for its
  kind fails the rules test.
- **The game places by them after the floor has settled**
  (`game::world::placement_rules`). It looks at every spot the rules allow:
  - on a thing's own top: the highest level face of it, flat under the
    whole of what is set down, nearest its middle (not its box's top: a
    vase or a chair's back stands higher);
  - at a wall's foot or in a corner (a lone pillar is no wall);
  - only in the dark where a rule says `dark`.

  The spot must be in a purpose a rule names and in none the asset never
  fits, reachable from where you arrive, and never behind a barricade, in
  a wall or in another thing. It is chosen on a stream of its own
  (`SplitMix64`, never the level's: no floor's draws move). The assets
  take turns. No two of a kind lie within 6 m (two in one view read as a
  trail laid), nor within 6 m of one that something else made (her camp's).
  A thing its rules allow nowhere is not on that floor. Each keeps the
  rule that placed it, and `what_the_rules_place_keeps_its_rules`
  (`tests/usable.rs`) holds every one to it on 36 floors: resting on what
  it stands on (a ray down meets its top there), apart, in its counts.
- **The bundle is its sources.** `the_bundle_is_its_sources` (xtask)
  fails when a `placement.toml` changed without `cargo xtask asset write`.
- **For agents making places**: the rules of every asset are one table
  (`cargo xtask asset rules` prints them); a place is dressed from the
  things whose rules allow it, in their counts, for their reasons.

## Things the layout makes

A door is not put down by the placement pass: the floor's layout makes it
where its plan has one (the far wall's way down, a cell's front, a
gallery, a lift's place against a wall). Its dossier says so, and each
rule names the place in the layout it stands at instead of where the pass
would put it:

```toml
# assets/src/props/exit_door/placement.toml (as it is)
by = "layout"
count = [1, 3]
levels = [1, 999]

[[rule]]
at = "way down"
purposes = ["*"]
why = "an office floor's stair is at its far end, and its door is the last one on the floor, whatever the room in front of it became"
```

- **`at` is the place in the layout**: "way down", "loop", "stairwell",
  "cell", "gallery", "lift", "broken lift", "wall", "barricade", "lamp"
  (a lamp a zone hangs: `high_bay`, `bulkhead`, `World::lay_panels`); a rule
  with `at` has no `on`, `of`, `near` or `dark` (`cargo xtask asset check`
  refuses them), and `levels` may start on the Building's floors (1).
- **The layout asks before it makes one** (`Game::place_by_layout(asset,
  at, x, z)`): the first rule for `at` whose purposes take the zone's,
  none the asset never fits, on a floor in its levels, and fewer than the
  most of its `count` made already. One allowed is recorded with its rule
  (`Game::layout_things`); one refused is not made, and its maker keeps
  its draws.
- **`what_the_layout_places_keeps_its_rules`** (`tests/usable.rs`) holds
  every one, on 48 floors of the rooms and the Building: its rule's place,
  purposes and levels, its counts; one way down on every floor; what was
  made against what was recorded (every cell's door, every gallery door
  that knocks, every lift and broken lift); and every rule one some
  floor's layout makes (a rule nothing makes is a lie).
  `the_doors_are_built_of_their_parts` holds each builder to its real size.
- **The playtest's question is the same**: frames of it where the layout
  put it (`cargo xtask asset sheet NAME --kinds core_exit_door,…`, the
  census kinds that hold it). Where the answers differ from its story, the
  rule changes and the layout with it.

## The doors

Every door in a wall, remade by the pipeline (`../plan/DOORS.md`): five
real things, a dossier each (`assets/src/props/`), their parts in one
family (`assets/src/blender/props/game/doors.py`), put together by
`core_props::doors`, each made by the layout under its rules:

- **`exit_door`**: a flush red-oak veneered office door in a pressed-steel
  frame, its lever, closer and kick plate, the lit exit sign over it at
  its real size (no arrow): the way down of an office floor, the
  Building's, and the Loop's way in (no sign) and its side doors.
- **`steel_door`**: a hollow-metal stair door in the same frame: the way
  down where the floor's far wall is at the back of the house (a
  maintenance run, service tunnels, a storage hall, a pool hall,
  washrooms: the playtest found a wood office door "soft for a raw back
  room"), with its STAIRS plaque on the Stairwell's floor; the Stairwell's
  top and bottom doors.
- **`hermetic_door`**: a Soviet shelter door the size of a ДУ-IV-2, its
  hand wheel and gearbox: every cell's door and the Well's galleries'
  (the door that knocks among them).
- **`lift_doors`**: a landing entrance, its two hairline stainless panels
  worn by hands and wiping, its dark-bronze frame set nearly flush, its
  sill and call station: the working lift, the dead one, the broken one
  jammed apart on the shaft.
- **`interior_door`**: a hollow-core six-panel moulded door in its
  casing: the door to nowhere against a wall, and the door a home's
  barricade nails across a doorway.

What the playtest asked of each, and the rules it changed, is in each
dossier's `playtest.md`.

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
