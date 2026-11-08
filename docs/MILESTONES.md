# Open milestones

Every milestone the docs still hold open, gathered on 2026-11-09 from their
milestone and progress tables, in the order they are being done. Each is
done in its own commit, pushed, with the production build made
(`cargo xtask bundle`); after every fifth commit a code review, and what it
finds made better and faster. The design behind each stays in its own doc;
this file is the queue. A milestone done moves to *Done* with its commit.

Complete and not repeated here: the creatures (`ENTITIES.md` E0–E10), her
story (`GAMEPLAY.md` G0–G8), the Building's first pass (`MEGASTRUCTURE.md`
B0–B9), the rooms up close (`ROOMS.md` R1–R6), the secrets (`SECRETS.md`
S0–S4), the screens (`UI.md` U0–U8), the vending machine (`VENDING.md`
V0–V5), the windows (`WINDOWS.md` W0–W4), the voices (`VOICES.md` V0–V9),
the tooling (`TOOLING_ROADMAP.md` M1–M13).

Left out on purpose: the tooling roadmap's optional items (3.3 Blender MCP,
3.4 generated meshes, 3.5 LODs, 3.6 reference sounds), which wait on a
licence policy or a need; the second voice persona (`VOICES.md`, *Next
after the first*), not yet a milestone.

## The queue

| # | From | Milestone | Done when |
|---|---|---|---|
| 1 | `MEGASTRUCTURE.md` G2 (open) | **The booths in the Stair Halls**: whether a third more building standing in the void (the cells' booths, doubled by G2) costs the Stair Halls their emptiness; a booth lower (from the fog to just over its walk) or narrower if it does | a blind review, same cameras before and after (both orders, a fresh reviewer); the change, if any, with its census and re-recordings |
| 2 | `MEGASTRUCTURE.md` G7 (open) | **The far figure, finished**: the blind review against the old figure (both orders); in the Well only its head and shoulders show over the parapet from below; on a flight a foot may sink into the step above | the review's verdict; `--far-figure` frames before and after; the figure's rule tests |
| 3 | `MEGASTRUCTURE.md` G3 | **The way and what is off it**: an audit of every floor for what makes a way interesting (choices, loops, height, landmarks, the door seen) and what is off it (dead ends that hold something, her trail, notes, water, batteries); a test that each floor keeps them once shorter; the generator or dressing fixed where it does not | the test over many seeds; the audit's numbers in `MEGASTRUCTURE.md` |
| 4 | `MEGASTRUCTURE.md` G4 | **Places off the walks**: rooms you can step into from a walk, of each place's own kinds (the Well's, the Stair Halls', the Works'), each holding something; the cells stay one shape | tests (reached, solid, never on the way only); zone PNGs; a blind review against the bare platforms |
| 5 | `MEGASTRUCTURE.md` G6 | **What knocks at your cell**: sealed in while the fog is up, something knocks; answer it (knock back) and it tries the wheel, and the door gives unless you hold it; stay silent and it goes; a door left unsealed lets the fog in | tests (silent is safe, an answer brings the wheel turning, held is safe, let go is the fog); a pacing log; `LORE.md` |
| 6 | `MEGASTRUCTURE.md` G8 | **The second pass's evidence and ledger**: `GAMEPLAY.md`, `LORE.md`, `QUALITY.md`, `content.json`; the full check and the bench | full check passes |
| 7 | `PROPS.md` P4 | **Wear**: the shader's wear strong enough to read, by use (edges, what faces up, the floor); the too-clean props (the Halloween things, the triptych, the pale secretary) | a blind review; goldens; the bench |
| 8 | `PROPS.md` *Open after P3* | **The props' small faults**: the shapes the reviews found (the nail polish brush, the corner web, the suitcase's garment, the umbrella crooks, the roe head's muzzle, the trout's fins, the elephant's ears, the doll's limbs, the conker husk, the paper clips, the mask's strap, the bookcase's feet), the glass (the mirrors' bloom, the atomizer), the placements (the botanical frame in open space, the leaf decal, the bright speck), and a contact shadow under small props | a blind review of each against its old self; reference levels; the asset lint |
| 9 | `PROPS.md` P7 | **The props' canon and reviews**: one rule in `LORE.md` for what still burns (the lit candles and fire, the cold censer), `QUALITY.md`, the ledger; `lore-keeper` rounds; the full check | full check passes; the lore-keeper's findings fixed |
| 10 | `PERF.md` (the bench) | **The level builds back under budget**: seed 7's level 5 builds in 625–688 ms against 600 (older than the voices) | `cargo xtask bench` passes, recorded |

## Done

| # | Commit | Notes |
|---|---|---|
