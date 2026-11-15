---
name: make-asset
description: Make or remake a prop (an asset) of The Backrooms from what it is and why it is here - its brief, references, model and photographed surfaces, what a hand does with it, the rules for where it lies, and the playtest question "why could this be here?" - ending in its rules read by the game and a blind review.
---

# Make or remake an asset

The pipeline is `docs/design/ASSETS.md`; this is its order, with the
commands and gates. One asset at a time: its dossier is
`assets/src/props/<name>/`.

1. **Does it earn a place?** (`ASSETS.md`, *What earns a place*): it
   belongs where it would lie, says something, can be seen at a crouch, is
   not a second name for a kind there is (`docs/research/assets-census/grades.tsv`),
   not a stage prop. If not, stop and say so.
2. **Start its dossier**: `cargo xtask asset new <name>`.
3. **Brief** (`brief.md`): what it is (the real thing, its make, size,
   materials, era); how it came here (quote the canon, with its section;
   never what the canon keeps unsaid: `LORE.md`, *How canon reaches the
   player*); the pillar rule it serves; what a hand does with it (most
   things: nothing; a verb only where the canon has a hand use it, *Within
   reach*, *Survival*); where it would be found. Have `lore-keeper` read it
   (general-purpose agent with `.claude/agents/lore-keeper.md` as its brief
   when the agents cannot be loaded) and fix what it finds.
4. **References** (`references.toml`): ask a research agent for
   photographs and documents of the thing and of places with it in, both
   licences: `free` (CC0, public domain, CC BY with attribution: may be
   sampled; give its SHA-1) and `limited` (fair use: looked at and
   measured, never sampled, never committed). Measurements and the palette,
   each with its reference. `cargo xtask asset refs <name>` fetches them to
   `target/asset-refs/<name>/`; look at them.
5. **Model** in its theme's Blender family (`assets/src/blender/props/<theme>/`,
   `game/` for the game's own), the real thing at its real size
   (`docs/design/PROPS.md`, *How a prop is remodelled*); `cargo xtask
   assets props --write`; `REGEN=1 cargo test --release -p game --test asset_lint`.
6. **Surfaces**: painted in the canvas from its palette and its
   references' patterns (`canvas_tex`, its own stream), or built from a
   `free` photograph (`assets/src/textures/`); never from a `limited` one.
   No emissive on what is not a light. The material lint
   (`tests/materials.rs`) and the look settle pass it.
7. **Placement rules** (`placement.toml`), from the brief's story: each
   rule a way it comes to be somewhere, with its `why`; what it stands on
   is a kind the game places. `cargo xtask asset check <name>`, then
   `cargo xtask asset write` (into `crates/game/assets/placement.json`).
   The game places it by them after a floor of the rooms settles
   (`game::world::placement_rules`): its maker makes it with
   `user.s["by"] = "rules"` and the asset's name as its `kind`, like
   `core_props::pickups`, and decides how many (the rules drop any past
   `count`'s most); the rules decide where. `on` is `floor` or `surface`
   (on a wall: not yet), `levels` from 4. Anything else that holds the
   thing (a list of pickups) must let go of the ones the pass removes:
   `place_by_rules` does it for the pickups' lists. The rules test
   (`what_the_rules_place_keeps_its_rules`, `tests/usable.rs`) holds every
   placed one to its rules, whatever its asset.
8. **The playtest question.** `cargo xtask asset sheet <name>`: frames of
   it where the rules put it. Give them to a fresh agent playtester
   (general-purpose, `.claude/agents/placement-critic.md` as its brief: it
   sees only the frames) and, when there are people, to the session script
   (`docs/research/PLAYTEST.md`): *why could this be here?* Write their
   answers beside the brief's story in `playtest.md`. Where they differ,
   change the rules: a place nobody could give a reason for goes; a reason
   they found and the brief missed becomes a rule; a reason nobody meant
   (a joke, a signpost) is a placement to take out. Ask again after.
9. **Gates and evidence.** `cargo xtask check --fast`, then the full check
   (the reference levels and fixtures re-recorded with `regen-fixtures`,
   the reason said); a blind review against what it replaced, from the
   census's cameras (`visual-review`, both orders, a fresh reviewer); the
   visual playtest where it lies (`visual-playtest --area`).
10. **Ledger.** `docs/plan/content.json` (`props/assets`: the asset in its
    notes), the plan's progress, `LORE.md` where the canon names it.

What an agent making a place uses: `cargo xtask asset rules` prints every
asset's rules as one table: dress a place from the things whose rules
allow it, in their counts, for their reasons.
