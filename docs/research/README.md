# Research: what the design rests on

The outside sources and what the design took from them
([`SOURCES.md`](SOURCES.md)); the research notes below, each answering a
brief; the props' census ([`props-census.md`](props-census.md)) and how
each family of props was remodelled ([`props-remodelling/`](props-remodelling/README.md)); and what
people and the tools found playing it: the playtest session script
([`PLAYTEST.md`](PLAYTEST.md)) and its records
([`playtests/`](playtests/TEMPLATE.md), the visual playtest's in
[`playtests/visual/`](playtests/visual/README.md)).

## The creatures

Five research notes gathered on 2026-10-28 for the creatures' redesign
([`../design/ENTITIES.md`](../design/ENTITIES.md)), each answering [the brief](brief.md).
Every claim links its source; each note says what it could not verify and
gives the licence of every design it describes. They were written by
research agents and are kept whole, as the record of what was found;
[`SOURCES.md`](SOURCES.md) section 11 sums up what the design took
from them and where it shows up. Read a source before relying on one of its
details.

| Note | What it covers |
|---|---|
| [`backrooms-and-samosbor.md`](backrooms-and-samosbor.md) | 22 entities of the Backrooms wikis (look, picture, rule, habitat, licence, what is overdone), how the games implement them; the Samosbor's rules, beings, liquidators and pictures; which fit our canon; eight concepts |
| [`scp.md`](scp.md) | SCP rule-monsters (173, 096, 106, 939, 513, 1499, 2316, ...) and why the rules frighten; *SCP – Containment Breach* read from its source code (the blink meter, the checks, the numbers), *Secret Laboratory*; what players found frightening or annoying; nine concepts and their credits |
| [`folklore-and-imaging.md`](folklore-and-imaging.md) | Creepypasta, analog horror, sleep-paralysis figures, Japanese and Slavic folklore (34 entities); how entities are pictured so they frighten (Mori, MacDorman, Carroll, Fisher; scale, faces, eyeshine, stillness, cloth); ten imaging principles; what not to copy |
| [`creature-ai-and-fear.md`](creature-ai-and-fear.md) | Creature AI in horror games (Alien: Isolation, Left 4 Dead, The Last of Us, Amnesia, Resident Evil 2, Silent Hill, Dead by Daylight, Lethal Company, Phasmophobia, FNAF, stealth AI); principles (tells, roles, senses, memory, visible learning, fairness, testing with bots); the psychology of fear; an architecture and a test plan for us |
| [`modelling.md`](modelling.md) | CC0 base meshes (MakeHuman, Blender's Human Base Meshes) and textures, with their licence lines; emaciation, cloth, hair, eyeshine, wet skin and readability by script; public-domain reference; a pipeline, budgets and risks |

Two more notes were gathered on 2026-10-29 for the vending machine
([`../design/VENDING.md`](../design/VENDING.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 12:

| Note | What it covers |
|---|---|
| [`vending-machines.md`](vending-machines.md) | Real glass-front snack and lit-panel drink machines from their factory manuals (AP Snackshop 113/123/133, Crane National 167, Dixie-Narco 501E): sizes, the glass, trays and spirals (counts, depths, how a product sits and sticks), price rolls and codes, keypads and display messages, money and the delivery bin, feet and the lean, the one tube inside; how they age (photographs and their licences); their sounds; stuck snacks, drop sensors, tipping deaths; CC0 surfaces; what looks fake |
| [`vending-horror.md`](vending-horror.md) | Uncanny vending machines: SCP-261, 294, 8733 and others; the Backrooms wikis' machines and the Backrooms games'; Japanese legends, creepypasta, Ohashi's *Roadside Lights*, the CPSC's tipping deaths; machines in games (BioShock, Half-Life, Lethal Company's counter, Control, Fallout, RE4, Yume Nikki); why a mundane machine turns uncanny, what players will try, design lessons; eight concepts and three picks |

Two more were gathered on 2026-10-30 for the windows
([`../design/WINDOWS.md`](../design/WINDOWS.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 13:

| Note | What it covers |
|---|---|
| [`windows-real.md`](windows-real.md) | Real windows of the buildings the rooms imitate (wood double-hung sash, rolled-steel hoppers, aluminium, office storefront framing): their sizes, sills, reveals and trim, hardware, finishes, how they age (NPS briefs, manufacturers' manuals, a tower survey), blinds and curtains (a contract mini blind's cut sheet); how glass looks (Fresnel, the night mirror, double glazing, tint, old glass, dirt and veiling glare, smears); how games show a view beyond a window (skybox, ground-projected panoramas, interior mapping, parallax-corrected cubemaps, layered planes, portals) and what holds up; seventeen CC0 panoramas; what looks fake; window types, sizes and glass for this game |
| [`window-horror.md`](window-horror.md) | Windows that look back: fifteen SCP articles (SCP-965, 1148, 2885, 508, 3052, L682 and others, with the Japanese and Chinese branches'), the Backrooms' Windows (Entity 2) and Level 188, Kunekune and Yamanoke, creepypasta, Soviet children's horror and Russian window taboos; public-domain literature (*Wuthering Heights*, *Dracula*, M. R. James, *Viy*, Poe, Wells, Lovecraft) and folk custom; games and films; why window horror works; how it must differ from the canon's gaze rules; seven concepts and three picks |

Two more were gathered on 2026-10-30 for the marks and the secrets
([`../design/SECRETS.md`](../design/SECRETS.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 14:

| Note | What it covers |
|---|---|
| [`wayfinding-and-secrets.md`](wayfinding-and-secrets.md) | Real route marks (the Appalachian Trail's blazes and the offset double blaze, cairns and the scouts' stone signs, cavers' tape and soot, divers' line arrows and personal cookies, FEMA's search marks, Berlin's shelter arrows, Hansel's pebbles); how games guide without arrows (wienies, light, the yellow paint debate, runner vision, *Left 4 Dead*'s walls, *Half-Life 2*'s lambdas, the Souls' messages) and how faint a mark can be; secrets in level design (*Doom*, *Quake*, *Rayman Legends*, *Link's Awakening*'s hollow clank; the rules designers state; how many a level); real hiding places and dead drops (Penkovsky's radiator, Ames's chalk line, Hanssen's tape); a vocabulary of her marks, twelve hiding places, a dead-drop sign, counts and tests |
| [`secret-items.md`](secret-items.md) | Alice's DRINK ME, EAT ME and the mushroom, read for design; the other world's food (Persephone, Izanami, Thomas the Rhymer, *Goblin Market*, the Lotus-eaters, fairy ointment, the Samosbor's ration); SCP objects that change the one who takes them (330, the first 1162, 427, 268, 714, 207, 178, 1916, 832, 362); the Backrooms wikis' objects (Almond Water's kinds, Cashew Water, Liquid Pain, Royal Rations, Liquid Silence, Moth Jelly); creepypasta and charms (the Midnight Game's salt, omamori and the ladanka, the red envelope, found coins); games with hidden-effect consumables (Isaac, NetHack, Darkest Dungeon, Pathologic, STALKER, Brogue; Harris's rules); how small an effect can be felt; fourteen candidates |

Three more were gathered on 2026-10-31 for the props
([`../design/PROPS.md`](../design/PROPS.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 15:

| Note | What it covers |
|---|---|
| [`props-census.md`](props-census.md) | Every kind of placed prop framed on 272 floors (`--prop-census`) and graded by seven fresh reviewers against `STYLE.md` (their reviews and every grade in [`props-census/`](props-census/)): 213 of 652 kinds read as toys, why (primitive bodies, icons for pictures and print, toy primaries, cloth as slabs, hard-edged decals, glow, proportions), how often each is placed, what is broken, the placers never placed |
| [`barricades.md`](barricades.md) | How the barricades are chosen, built and collide; how much of an opening they fill (measured: half stand in open floor; a pile fills a third of a corridor's width); how real ones are built (FEMA's board-up procedure, construction hoarding, furniture piles; lumber, plywood, desks, cabinets, pallets, shelving, mattresses, doors at their real sizes); what the canon says; five kinds by purpose, the code (collision, sight, paths, the sweep, the draws kept) and the Blender parts |
| [`props-interactions.md`](props-interactions.md) | Every verb the game has, where and how often; the interactions that break the canon now; the rules a new one must keep, each with its source; the props on every floor with no verb; fifteen proposals ranked, each with its verb, what it gives, its sound, the lines of `LORE.md` it touches and its cost; what not to do |


One more was gathered on 2026-11-08 for the voices
([`../design/VOICES.md`](../design/VOICES.md)):

| Note | What it covers |
|---|---|
| [`voice-tts.md`](voice-tts.md) | Text-to-speech models that run on a 64 GB Apple Silicon Mac, each with its code and weights licence (the gate: what it makes may be sold), size, Mac support and speed, emotion and intensity control, nonverbals, voice design and cloning; the excluded ones and why (non-commercial weights, Llama terms, an audible disclaimer); the independent evidence (and its absence for emotion); English pronunciation; the evaluation stack (speech recognition, emotion recognition, AudioSet's nonverbal labels, speaker similarity, prosody); the Python environments; the risks |

One more was gathered on 2026-11-10 for your asides
([`../design/VOICES.md`](../design/VOICES.md), *Your asides*), summed up in
[`SOURCES.md`](SOURCES.md) section 16:

| Note | What it covers |
|---|---|
| [`inner-voice.md`](inner-voice.md) | How a game chooses a character's line and keeps it from repeating (Valve's dynamic dialog, Ruskin GDC 2012, read in full: facts, rules, the most specific match, memory written back by what was said, nothing as an answer); bark managers in practice (priorities, dropping the stale, cooldowns and per-line fatigue); the little found on voicing the player in horror; what this game's canon lets a voice of yours say |

One more was gathered on 2026-11-11 for the cutscenes
([`../design/CUTSCENES.md`](../design/CUTSCENES.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 17:

| Note | What it covers |
|---|---|
| [`cutscenes.md`](cutscenes.md) | How sequencers store a cutscene (Unreal's tracks and *When Finished*, Unity Timeline's wrap modes and signals, Cinemachine's blend against the live camera, Valve's VCD scenes, Frictional's HPL2 primitives); first-person games that never take the camera (Half-Life 2, SOMA) and title screens that lead straight into play; telling the player their hands are taken (hidden HUD, input locks, letterbox and its complaints, hold to skip, Xbox and Game Accessibility Guidelines); motion comfort (Forsyth, Meta, XAG 117); death in first-person horror (Grip, Red Barrels, The Callisto Protocol, Alien: Isolation); testing by time, golden frames, skip as the end state; twelve design rules |


One more was gathered on 2026-11-11 for the visual playtest
([`../engineering/VISUAL_PLAYTEST.md`](../engineering/VISUAL_PLAYTEST.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 18:

| Note | What it covers |
|---|---|
| [`visual-qa.md`](visual-qa.md) | How studios automate visual QA (Frostbite's cameras per level and strict thresholds, Unreal's screenshot actors, Unity's golden images, flythroughs, bots); z-fighting (its cause, polygon offset, depth precision and reversed-Z, an image-based coplanarity detector, geometric tests); holes and leaks (the hall of mirrors, T-junctions, watertightness, the winding number, thin walls); the near plane and the camera inside geometry (cast its face, not a ray); out-of-place assets (EA SEED's texture classifier and its white-placeholder lesson, texel density, albedo validation); triage tools and what vision models can and cannot find (GlitchBench, VideoGameQA-Bench); ten rules |

One more was gathered on 2026-11-14 for the agents' trust
([`../engineering/TRUST.md`](../engineering/TRUST.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 19:

| Note | What it covers |
|---|---|
| [`agent-trust.md`](agent-trust.md) | Whether any issue-implementing skill or integration acts only on maintainers' words and on others' once approved: Claude Code skills and plugins, claude-code-action (what it reads, who triggers, its edit freeze and sanitizer, agent mode), the GitHub MCP server's lockdown mode, gh-aw's integrity filter, endorsements that go stale on edit and protected files, Copilot, OpenCode, Gemini CLI, OpenHands; GitHub's mechanics a gate needs (`pull_request_target` and its default block from 2026-12-02, commit statuses as required checks, roles, edit times, rulesets) |

One more was gathered on 2026-11-14 for the replays and saves
([`../plan/REPLAYS.md`](../plan/REPLAYS.md)), summed up in
[`SOURCES.md`](SOURCES.md) section 20:

| Note | What it covers |
|---|---|
| [`replays.md`](replays.md) | How games record and replay: input demos over a deterministic simulation (Doom, Quake's network demos, StarCraft, Age of Empires, Supreme Commander, Factorio, Halo, Trackmania, osu!, Celeste's TAS files) against state streams (Quake III, Unreal, Rocket League, Overwatch) and keyframes with inputs (MotoGP, Photon Quantum); starting anywhere (simulating from a checkpoint, snapshots, a save as a replay's start); determinism (the time step, random streams, iteration order, threads, and floating point across builds and systems: libm, FMA, fast-math, `powi`, the `libm` and CORE-MATH libraries); file size (inputs quantized where they enter, varints, deflate; sizes a minute); desync checks and binding a replay to its build; recording, browsing and playback in the menus; what this game does today, read from its code; twelve recommendations and a file layout |
