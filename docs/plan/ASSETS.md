# The work on the assets

The plan behind [`../design/ASSETS.md`](../design/ASSETS.md), for GitHub issue
#13: fewer props, and a pipeline that makes each one from what it is and why
it is here. What is still open is queued in
[`MILESTONES.md`](MILESTONES.md).

## The brief (2026-11-14, issue #13)

> Current assets are of uneven quality and too varied.
>
> 1) Research current asset list and remove 30% of less interesting assets
> 2) Create a create/update asset pipeline, that
> - starts from description what is it, how it came to location of the
>   game and how it fits the lore
> - collects photos and references of such assets and locations (at this
>   step, both free and of limited use are ok)
> - create model and photorealistic textures based on the references, using
>   pallete, patterns and reference material to the extent that is free or
>   fair use
> - during playtest make playtester ask "why this could be here" and
>   compare to lore suggested. Use the differences to create rules for
>   asset placement
> - create interaction lore, based on game lore and gameplay pillars, not
>   every asset should be interactive
> - create rules for asset placement for later use by agent to create
>   in-game places
>
> Pack the resulting pipeline into agent skill and document it.
>
> As a sample, rebuild battery and almond water assets.

## The analysis (2026-11-15)

**Too many kinds, most of them rarely seen.** The props census of October
([`../research/props-census.md`](../research/props-census.md)) saw 652 kinds of
prop on 272 floors; the themed placers alone are 677
(`world::tables::THEMED_ORDER`). Since then every `cartoon` kind has been
remodelled (P1-P3), so what is uneven now is less how each is made than
how many there are and what they are: on the census's 272 floors
(A1, [`../research/assets-census.md`](../research/assets-census.md)), 67 of the
toys' 76 kinds are on fewer than ten, and so are 44 of the nature theme's
45, all the occult's 36 and all the Victorian's 76; thirty placers never
place anything. A floor of an abandoned office building turns up a crystal skull, a
cauldron, a stuffed owl and a xylophone, each once in a long while: a
variety no one place would have.

**Each prop was made from its code, not from what it is.** A placer was
written as primitives, then finer parts were fitted to them, then the
cartoons were remodelled from a maker's sheet. Nothing records why a thing
is where it lies, who left it, or whether a player asks why it is there;
where it may stand is the placer's own code (its theme's purposes,
`THEME_COMPAT`), not a rule anyone can read or reuse.

**The two things everyone picks up are the least real.** The almond water
is an 18 cm wide cylinder that glows faintly; the battery is 12 cm across
and 18 cm long, about three times a D cell each way. Both float and spin.

## Milestones

| # | Milestone | Evidence |
|---|---|---|
| A1 | **The census again**: every kind on the October census's 272 floors, framed as the game places it, graded by fresh blind reviewers for whether it belongs here, whether it is of interest, and how it is made; the selection rule and the 30% to remove | the reviews, the grades, the list with its reasons |
| A2 | **The 30% removed**: their placers, registrations, theme entries, Blender parts and notes; the canon written from the code's gates where it named them | the fixtures re-recorded with their reasons; the range baseline; the level sweep; a zone view per purpose |
| A3 | **The pipeline** (`docs/design/ASSETS.md`, the skill `make-asset`): an asset's brief (what it is, how it came here, how it fits the canon, whether a hand uses it and why), its references (free ones kept with their licences and checksums, the rest as links and notes only), its model and photographed textures, its placement rules, the playtest question; `cargo xtask asset` to start one and to check it | the tool's tests; the skill run on the samples |
| A4 | **Placement rules the game reads**: where an asset may lie (purposes, surfaces, what it is near, how many), held by a test of every placed one; the playtester's "why could this be here?" (people's script and an agent's brief) and how its answers become rules | the test; the rules of the samples |
| A5 | **Almond water**, through the pipeline | its brief, references, model, textures, rules; a blind review against the old; the playtest question asked |
| A6 | **The battery**, through the pipeline | the same |
| A7 | **Review**: the code and the docs of A1-A6 | a fresh reviewer's findings, fixed |

## Progress

| Step | Commit | What |
|---|---|---|
| A1 | (this commit) | The census again (`../research/assets-census.md`): 695 kinds on the October census's 272 floors, 657 framed and judged by five fresh reviewers (belongs, interest, made), how often each is placed by its draws as well; thirty placers that never place, the specks, the rare themes, 46 groups of duplicates; the rule and the 208 to remove (30%); what is broken in what stays, queued (`MILESTONES.md` 24) |
| A2 | (this commit) | The 208 removed (the census's `grades.tsv`, decision `remove`), by a script with every edit asserted: their modules, registrations, `THEMED_ORDER` (677 → 466 passes) and `PLACER_THEME` (649 → 441) entries, the shared helpers only they used, the asides' second cone placer; each theme's smoke count and the ledger's; `LORE.md`'s *What still burns* written again from what places (no hanging lantern, chandelier, chime candles, teapot or censer). Every kept placer's fingerprint the same (the removed' rows gone); the generator's range holds. Their Blender parts stay in the meshes for now: with the parts older remodels replaced, 1,182 are named by no code (17% of the triangles), queued as `MILESTONES.md` 25 |
| A3 | (this commit) | The pipeline: `docs/design/ASSETS.md` (what earns a place, the dossier, the order, the placement rules' format), the skill `make-asset`, the `placement-critic` (a player's "why could this be here?" from frames alone), `cargo xtask asset new | check | refs | write | rules | sheet` and the dossier's templates (`assets/src/props/TEMPLATE/`), `crates/game/assets/placement.json` (no asset yet); the session script asks people the question too (`PLAYTEST.md`) |
| A4-A6 | (this commit) | Almond water and the battery through the pipeline (their dossiers, `assets/src/props/`): a plain 500 ml PET bottle and an alkaline D cell at their real sizes (`game/almond_water_and_battery.py`, ten parts; the four old fitted ones gone), one builder each for the seven places that made them (`core_props::pickups`: the bottle sealed, half or low, standing or on its side; the cell's printed film, an old one's crystals), still where they lie (no bob or turn), nothing glowing. Placement rules the game reads (`world::placement_rules`, after the floor settles, on a stream of its own, no draw moved), held by `what_the_rules_place_keeps_its_rules`. "Why could this be here?" asked twice (`placement-critic`): surfaces obvious; the floor's no-reason frames 4 → 2 (water) and 11 → 5 (battery) after the rules it gave: at the wall's foot, no lone pillar, 6 m apart, a bottle on the floor lying, a battery on the floor only in the dark or the halls walked with torches. A blind review against the old |
| A7 | (this commit) | The review of A1-A6 by two fresh reviewers. The lore-keeper's: every almond water sealed and full (an opened one broke the yellow one's tell and what is said on picking one up: the half and low states gone; the yellowish one put away built by the same builder), the cell's make HALDEN (HOLLIS is a snack maker), its caution a handbook's and its lot code no date, the machine's painted cells and her camp's caps made to match, the two pickup lines that told the player a story gone, `LORE.md` written from the rules (*Survival*; no teapot). The code-reviewer's: a thing set on a surface rests on the thing's own level top, flat under it (`rest_on`; it floated on a box's top over flaps, vases and chair backs), held by rays in the test; one pass for any asset marked `by = "rules"` (its stream from its name, `count`'s most kept, `on = "wall"` and levels before 4 refused); the Loop's pickups instances of the builders' parts; a pickup's state from both coordinates whole; her camp's pickups kept clear of and apart from; the bundle held to its sources (`the_bundle_is_its_sources`); the water drawn inside its own bottle only (a shared origin, not a render order); the cost measured in turn against A6 and `5946106` (`../engineering/PERF.md`: none to speak of; no run recorded while the machine is not quiet) |
