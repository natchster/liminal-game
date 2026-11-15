# The work on the assets

The plan for GitHub issue #13: fewer props, and a pipeline that makes each
one from what it is and why it is here. What is still open is queued in
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
| A3 | **The pipeline** (its design doc and the skill): an asset's brief (what it is, how it came here, how it fits the canon, whether a hand uses it and why), its references (free ones kept with their licences and checksums, the rest as links and notes only), its model and photographed textures, its placement rules, the playtest question; `cargo xtask asset` to start one and to check it | the tool's tests; the skill run on the samples |
| A4 | **Placement rules the game reads**: where an asset may lie (purposes, surfaces, what it is near, how many), held by a test of every placed one; the playtester's "why could this be here?" (people's script and an agent's brief) and how its answers become rules | the test; the rules of the samples |
| A5 | **Almond water**, through the pipeline | its brief, references, model, textures, rules; a blind review against the old; the playtest question asked |
| A6 | **The battery**, through the pipeline | the same |
| A7 | **Review**: the code and the docs of A1-A6 | a fresh reviewer's findings, fixed |

## Progress

| Step | Commit | What |
|---|---|---|
| A1 | (this commit) | The census again (`../research/assets-census.md`): 695 kinds on the October census's 272 floors, 657 framed and judged by five fresh reviewers (belongs, interest, made), how often each is placed by its draws as well; thirty placers that never place, the specks, the rare themes, 46 groups of duplicates; the rule and the 208 to remove (30%); what is broken in what stays, queued (`MILESTONES.md` 24) |
| A2 | (this commit) | The 208 removed (the census's `grades.tsv`, decision `remove`), by a script with every edit asserted: their modules, registrations, `THEMED_ORDER` (677 → 466 passes) and `PLACER_THEME` (649 → 441) entries, the shared helpers only they used, the asides' second cone placer; each theme's smoke count and the ledger's; `LORE.md`'s *What still burns* written again from what places (no hanging lantern, chandelier, chime candles, teapot or censer). Every kept placer's fingerprint the same (the removed' rows gone); the generator's range holds. Their Blender parts stay in the meshes for now: with the parts older remodels replaced, 1,182 are named by no code (17% of the triangles), queued as `MILESTONES.md` 25 |
