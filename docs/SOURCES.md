# Sources

The outside sources behind the game's design and the project's structure,
what each one says that mattered, and where it shows up in the code. They
were gathered in October 2026 in two rounds of research:

* **Horror design** (sections 1–7): what makes liminal horror suspenseful,
  for the work on threats, creatures, sound, rooms and props.
* **Rust game development** (section 8): how to lay out and organise a
  Rust game, for the workspace restructuring.

A note on weight: the strongest sources here are developer-facing write-ups
(Game Developer, the Valve wiki, a peer-reviewed paper). Several others are
reviews, essays and blog posts; they are listed for what they contributed,
marked *(secondary)*, and nothing rests on them alone. Some pages were only
seen through search-result summaries, not read in full; those are marked
*(summary only)*.

## 1. Pacing: tension, build-up and release

**[The Balancing Act of Tension in Horror Game Design](https://www.gamedeveloper.com/design/the-balancing-act-of-tension-in-horror-game-design)** (Game Developer). Read in full.
- Says: horror needs both build-up and release; neither works alone. Use fewer but more dangerous enemies, since players get used to weak ones. Hide the threat: the unknown is the strongest source of dread. Ambiguous cues and recurring audio cues (Silent Hill's radio) build it.
- Used for:
  - The pacing director's cycle of calm, build-up, peak and relief (`crates/game/src/entities/director.rs`).
  - Keeping at most three creatures and sending one at a time, instead of adding more.
  - The presence cues during a build-up: footsteps far off, lights stuttering near the creature, then silence. The creature is heard and sensed before it is seen.

**[L4D2 Director Scripts](https://developer.valvesoftware.com/wiki/L4D2_Director_Scripts)** (Valve Developer Wiki).
- Says: Left 4 Dead's director runs in named states (`BUILD_UP`, `SUSTAIN_PEAK`, `RELAX`). Threats only spawn during build-up, and the relax state holds them back.
- Used for:
  - The director's phases and their order (`director::Phase`: Calm, Build, Peak, Relax).
  - The rule that nothing new is sent after you while you recover. In Relax the creatures withdraw, hear less, and keep away for 25–45 s.

**[PaceMaker: A Practical Tool for Pacing Video Games](https://arxiv.org/pdf/2408.15001)** (arXiv paper).
- Says: the Left 4 Dead director estimates each player's intensity and removes major threats when it runs too high, so the pacing curve of peaks and valleys emerges from rules rather than a script.
- Used for:
  - The director's `tension` value: how close the threat feels, from the nearest creature's distance and state. It drives the drone, the heartbeat and the vignette.
  - Letting the cycle emerge from rules (distances, sight, the menace gauge) instead of timed scripted scares.

**[Defining Horror as Tension: Silent Hill f](https://gamedev.net/news/4735-defining-horror-as-tension-silent-hill-f/)** (GameDev.net summary of a GDC talk).
- Says: horror as sustained gameplay pressure rather than visual shock; the work is balancing threat, information and relief.
- Used for:
  - Building the design around pressure and relief (the director) rather than one-off jump scares.
  - The withdrawal after a peak, which gives the player relief.

**[How Evolve learned from Left 4 Dead's AI director](https://shacknews.com/article/82787/how-evolve-assures-action-peaks-and-valleys)** (Shacknews) and **[modl.ai on the AI director](https://modl.ai/?p=1984)** *(secondary, summary only)*.
- Say: the director is a state machine that "takes its foot off the gas", giving a roller-coaster of quiet stretches and intense bursts.
- Used for: confirming the peak-and-valley shape. Nothing specific was taken from them.

## 2. Creature AI: Alien: Isolation

**[Revisiting the AI of Alien: Isolation](https://www.gamedeveloper.com/design/revisiting-the-ai-of-alien-isolation)** (Game Developer). Read in full.
- Says:
  - Two systems. A director knows where the player is but only ever sends the alien toward the player's general area. The alien has to find the player with its own senses.
  - The alien sees through view cones, not all around, and hears noise radii that grow with walking, running and gunfire. Its senses fade, so it forgets.
  - A disturbance starts a search around the noise, with a radius that shrinks pass by pass. A donut-shaped search keeps it from standing right on the player's goal.
  - A menace gauge fills while the alien is close and seen or heard. When it peaks, the alien backs off into the vents.
- Used for:
  - **The director steers, the creature finds.** The stalker is sent to tiles 6–16 m from you that you cannot see, never to your exact position (`steer_stalker`). Only when a build-up runs out is it told where you are, and again every 4 s for its last stretch (`final_push`).
  - **A view cone.** A creature that isn't hunting sees you only inside a wide cone ahead of it, right beside it, or when your flashlight is aimed at it (`entity_faces_player` in `entities/ai.rs`). Before this they saw in all directions.
  - **Heard means searched.** Hearing you starts a search, not a hunt. The search walks to the noise, stops to listen and tries ever-closer spots, and a creature that loses you searches where it last had you (`entity_search`, `search_step`).
  - **The menace gauge and withdrawal.** The gauge fills with closeness and sight. Once it is full and you have broken away, or it is a little past full and the hunter is not on top of you, everything after you withdraws to far, dark goals (`withdraw_entity`).

**[The Perfect Organism: The AI of Alien: Isolation](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation)** (Game Developer).
- Says: the director's influence is advisory, never orders, and a sustained menace turns tension into aggravation.
- Used for:
  - Keeping the director's role to hints: goals near you, the last-stretch hints.
  - Tuning the withdrawal so that a careful player usually escapes a chase: in the pacing test about 1 death in 9 chases (`crates/backrooms/tests/pacing.rs`).

**[Alien Isolation: The Dual-Brain Mastery Behind the Xenomorph](https://feedme.design/alien-isolation-the-dual-brain-mastery-behind-the-xenomorphs-relentless-pursuit)** *(secondary, summary only)*.
- Says: the same director/alien split, and that the alien never cheats with knowledge it hasn't sensed.
- Used for: corroboration only.

## 3. Sound

**[Silence is scary: sound design in horror games](https://www.wayline.io/blog/silence-is-scary-sound-design-horror-games)**, **[The power of silence in game audio](https://www.wayline.io/blog/power-of-silence-gaming-audio)** (Wayline) and **[The case for silence in game audio](https://bugnet.io/blog/the-case-for-silence-in-game-audio)** (bugnet) *(secondary)*.
- Say:
  - Loudness registers relative to what came before, so silence makes the next sound land harder.
  - Constant background sound desensitises the player and telegraphs scares.
  - The strongest horror ambience is diegetic: sounds made by the place.
- Used for:
  - **The hush.** As something comes, the ambience goes quiet: one-shots soften and hold, the room tone and the lights' hum sink away, and glitches and flicker waves wait (`Game::amb_hush`, driven by `director.hush`).
  - **No random filler.** The centred, random ambience loop went, replaced by sounds from places.
  - **Measured against the bed.** The listening test measures every check against the room's bed (`crates/backrooms/tests/listen.rs`).

**[Seeing with your ears: the audio of Alien: Isolation](https://pcgamer.com/the-audio-of-alien-isolation)** (PC Gamer), with **[Listen or Die](https://audiotechnology.com/?p=29542)** (AudioTechnology), **[Sam Cooper on Alien: Isolation's sound](https://mcvuk.com/?p=75051)** (MCV) and **[The Sound Architect on Alien: Isolation](https://www.thesoundarchitect.co.uk/alienisolation/)** *(summary only)*.
- Say: in Alien: Isolation, listening is survival. The creature's footsteps had to sound alive rather than mechanical, and the sound team worked with conditioning, misdirection and dynamic contrast.
- Used for:
  - **Footfalls from the creature.** Creatures' footsteps play from where they walk: spatial, muffled through walls, loud in a chase, soft while searching. Each has its own gait: the Smiler's long, uneven stride, the Hound's scramble (`entity_steps`, `entity_footfalls`). Before, they were one centred thump.
  - **The notice.** A shriek from where the creature stands plus a dissonant sting (`entity_call("shriek")`, `dread_sting`).

**[IT: Welcome to Derry sound design](https://www.asoundeffect.com/it-welcome-to-derry-sound-design/)** (A Sound Effect) *(summary only)*.
- Says: everyday sources such as fluorescent flicker were processed into unsettling material.
- Used for: treating the fluorescent ballast hum as a sound with a role. It sinks away while something approaches.

**[The sound design behind a famous horror game](https://scholarblogs.emory.edu/introtofilm2024/?p=1299)** (Emory student analysis of Five Nights at Freddy's) *(secondary, summary only)*.
- Says: non-diegetic stingers distract rather than help.
- Used for: keeping stings rare and tied to events. One plays when you first see each kind of creature on a level, and one when a creature notices you, with a cooldown.

How the sound principles show up together (`crates/game/src/audio/ambient/`):
- **From a place.** Every prop sound comes from the nearest prop in earshot, panned by direction and muffled through walls.
- **A level's acoustic identity.** Each level has 3–4 signature sounds anchored to fixed spots, on rhythms of their own. Each carries as far as it really would (`sig_carry`).
- **The weather's sources.** The weather's own sounds come from a draught, a duct, a dying fan, an occupied room or the nearest light fitting (`BED_ANCHORS`).
- **No pile-up.** Nothing piles up when a level opens.
- **Varied, not random.** Per-level choices come from a local stream seeded by the level, so they are stable for a run and different between runs.

## 4. Liminal spaces and the Backrooms

**[Remedy's creative director on the Backrooms and liminal spaces](https://feedbagel.com/post/control-resonants-creative-director-knows-why-youre-so-scared-of-backrooms)** *(secondary)*.
- Says: in liminal spaces the emptiness itself becomes the threat, like an uncanny valley for architecture; the spaces break expectations of how a space behaves.
- Used for:
  - Leaving some areas deliberately empty of themed props (`PURPOSE_EMPTY` in `crates/game/src/world/tables.rs`: 45% of lobby areas, 60% of pillar halls).
  - The director's Calm phase, in which nothing is sent after you.

**[Backrooms, liminal spaces and the subliminal menace of loneliness in indie horror](https://www.universityxp.com/news/2024/7/7/backrooms-liminal-spaces-and-the-subliminal-menace-of-loneliness-in-indie-horror-games)** (UniversityXP) *(secondary)*.
- Says: monotony and repetition themselves unsettle, and minimal environments with careful sound design carry the dread.
- Used for:
  - **Deliberate repetition.** One theme per area, and neighbouring zones of an archetype share one ceiling height.
  - **Sparse sound.** The ambience is sparse, with sounds from places.

**[Anemoiapolis Chapter 1 review](https://techraptor.net/gaming/reviews/anemoiapolis-chapter-1-review)** (TechRaptor) *(secondary)*.
- Says: praises a liminal game for avoiding the genre's default of endless corridors with a chasing monster, leaning on emptiness and darkness instead.
- Used for:
  - Pacing chases instead of having creatures always hunt: one creature is sent per cycle, with long quiet stretches between.
  - Darker, contrasting lighting. The flat ambient fill is cut by about 20% and the light pools under the fixtures are stronger (`crates/game/src/visuals/lighting.rs`).

**[What are the Backrooms and why are they so scary](https://www.pajiba.com/film_reviews/what-are-the-backrooms-and-why-are-they-so-scary.php)** (Pajiba) *(secondary)*.
- Says: the dread is in familiar spaces that feel incorrect. The original text stresses sensory banality: old moist carpet, mono-yellow walls, the endless buzz of fluorescent lights.
- Used for:
  - Keeping Level 0's yellow identity and the fluorescent hum as the signature bed.
  - Grime, stains and wear rather than monsters everywhere.
  - Muting props into believable, aged materials (`crates/game/src/visuals/prop_look.rs`).

**[Kane Pixels' Backrooms (2022–2025) review](https://fandomfrontline.bearblog.dev/kane-pixels-backrooms-2022-2025-review-the-youtube-masterpiece-you-need-to-see-before-the-movie/)** (Fandom Frontline) *(secondary)*.
- Says: the series defined the found-footage, worn-camcorder look of the Backrooms.
- Used for: the camera grade in the composite pass, with a little less saturation, a gentle S-curve, a slight black lift and a round lens vignette (`crates/engine/src/composite.metal`).

**[Kotaku on the Backrooms](https://kotaku.com/best-creepypasta-what-is-backrooms-tiktok-steam-reddit-1850298058)**, **[Caught in a Backroom Nightmare](https://eikonproject.substack.com/p/caught-in-a-backroom-nightmare)**, and the **[Escape the Backrooms](https://www.playnews.gg/en/guides/escape-the-backrooms-secrets-easter-eggs-and-hidden-achievements-2026)** guide and **[wiki entry](https://dic.pixiv.net/a/Escape_the_Backrooms)** *(secondary, summary only)*.
- Say:
  - The fear is of the unknown, in spaces that feel "just wrong".
  - Escape the Backrooms uses hand-tuned dead ends and distinct, themed levels rather than one generated template.
- Used for: corroborating designed structure over pure generation (section 5).

## 5. Procedural level design

**[How and Why Spelunky Makes its Own Levels](https://amara.org/videos/p6MHlmreohsY/en-gb/3101319)** (Game Maker's Toolkit, transcript).
- Says: Spelunky's levels are "half generated, and half designed". Authored room templates are placed on a generated structure, so levels don't read as algorithm output.
- Used for:
  - **Zone purposes.** Each area of a level gets one purpose: a prop theme its archetype welcomes, sometimes a compatible second theme, or nothing at all. Themed placers only see the zones whose purpose welcomes their theme (`crates/game/src/world/purpose.rs`, `Game::themed` in `world/build.rs`). Before, each of the ~650 placers re-rolled its own zones, producing grab-bags.
  - **Places from walls.** Regions, and so wallpaper, grime and darkness, follow zones and rooms instead of random Voronoi cells (`build_regions` in `world/mod.rs`).
  - **Everyday props where they belong.** Core props go where they would be found: filing cabinets in offices, cones and tubes in tunnels, wet-floor signs at the pool.
  - **Fixtures in rooms.** Light panels are centred in each room (`world/build.rs`).

**[hake devlog 4: level generation using templates](https://www.moddb.com/games/hake/news/hake-devlog-4)** (ModDB).
- Says: templates keep the designer's intent and avoid "weird combinations which a human designer would avoid".
- Used for: the theme compatibility table (`THEME_COMPAT` in `world/tables.rs`). An office may share a place with maintenance gear, a parlour with occult things; never a random mix.

**[Building is in](https://www.patreon.com/ProblemMachine/posts/building-is-in-46313671)** (Problem Machine) *(secondary, summary only)*.
- Says: an opinion that generated spaces lack the mystery of authored ones, and that a room's job is to host an event.
- Used for: support for giving areas purposes and leaving some deliberately empty, rather than filling every zone.

**[How the eerie atmosphere of the Backrooms can be incorporated into level design](https://playgama.com/blog/general/how-can-the-eerie-and-unpredictable-atmosphere-described-in-the-backrooms-theory-be-incorporated-into-my-horror-games-level-design/)** (playgama) and **[What should the Backrooms as a concept be about](https://opper.ai/ai-roundtable/questions/what-should-the-backrooms-as-a-concept-be-about-create-a-06692517)** (opper.ai roundtable) *(secondary, summary only)*.
- Say: generic advice on looping layouts and randomised lighting and sound; the suspicion that something is there matters more than a catalogue of monsters.
- Used for: nothing specific. The second agreed with keeping creatures few and unexplained.

## 6. Creature design

**[The Boogeyman: horror creature](https://forums.unrealengine.com/t/sayan-paul-the-boogeyman-horror-creature-character/2739253)** (Unreal Engine forums), **[Zomboid character design](https://extlinkstatic.kiwiirc.com/urgent-zomboid-character-design-a-2vvs.html)** and **[a creature-design guide](https://playbooks.com/skills/omer-metin/skills-for-antigravity/creature-design)** *(secondary, summary only)*.
- Say:
  - The uncanny comes from an ordinary human form with subtly wrong proportions and expressions, not from gore.
  - Motion that is too perfect reads as mechanical; purposeful imperfection unsettles.
  - Context does much of the work: lighting that shadows the features, silence broken by breathing.
- Used for:
  - **The Smiler:** a head taller, twitching more the closer it is, and its head cocking hard to one side the moment it notices you (`entities/anim.rs`).
  - **The Hound as a person on all fours:** its eyes glint through its hair, it rears up to look at you close by, and its back is a bare, starved human back with no band of hair down the spine.
  - **The Watcher:** the face pressed through its sheet, always turned to you.
  - **Context:** the lighting and grade work above, and creatures arriving out of the dark.

## 7. Evaluation

- **D. Hasler and S. Süsstrunk, "Measuring colourfulness in natural images", Proc. SPIE 5007, Human Vision and Electronic Imaging VIII (2003).**
  - Used for: the colourfulness metric in the before/after comparison of renders. Prop close-ups measured about 18% less colourful after the material pass.
- **Blind A/B review.** Not an outside source, only the method: an independent reviewer judged pairs of images from the original and the new build in random order, without knowing which was which. The answer key was kept separately.

## 8. Rust game development and project structure

Used for the earlier restructuring of the workspace (see
`docs/DEVELOPMENT.md` for the conventions that came out of it).

**[Large Rust Workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)** (matklad).
- Says: use a virtual manifest at the root, put every crate in a flat `crates/` folder and name each folder after its crate. The same author's cargo-xtask convention adds an `xtask` crate for automation instead of shell scripts.
- Used for: the root `Cargo.toml` as a virtual manifest, `crates/` (with the placer crates grouped in `crates/placers/`), and `cargo xtask bundle` replacing `build.sh` (`xtask/`, `.cargo/config.toml`).

**[Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)**.
- Says: shared `[workspace.package]` metadata, `[workspace.dependencies]` inherited by members, `[workspace.lints]`, and `default-members`.
- Used for:
  - One place for versions, dependencies (internal crates included) and lint settings, which each crate opts into.
  - `default-members = ["crates/backrooms"]`, so that `cargo run` starts the game.

**[Microsoft Rust Guidelines: project structure](https://microsoft.github.io/rust-guidelines/guidelines/project/)** and the **[Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html)**.
- Say: project-level guidance on structure, naming, documentation and lints.
- Used for:
  - `rustfmt.toml`, plus clippy as a gate with every allowed lint documented by its reason.
  - snake_case names throughout.
  - A doc comment heading each module folder's `mod.rs` describing what it owns.

**[Understanding Cargo Workspaces](https://users.rust-lang.org/t/understanding-cargo-workspaces/19235)** and **[Modern Rust project layouts](https://users.rust-lang.org/t/modern-rust-project-layouts/137003)** (Rust forum).
- Say: practical workspace layouts and splitting large codebases into crates.
- Used for: splitting the ~650 placers across 33 crates so they compile in parallel, and the layering in which `engine` never depends on `game`.

**[ZlormaEngine: a lightweight procedural 2D game engine in Rust](https://users.rust-lang.org/t/zlormaengine-a-lightweight-procedural-2d-game-engine-written-in-rust/141353)** (Rust forum).
- Says: keep the engine apart from the game.
- Used for: the split into `engine` (no game knowledge), `game` (no platform code, so it runs headless for every test) and `backrooms` (the executable that glues them).

**[game-loop crate](https://docs.rs/crate/game-loop/latest)** and **[generational-arena](https://docs.rs/generational-arena)** (docs.rs).
- Say: fixed-timestep loops, and generational indices for entity storage.
- Used for: consulted, not adopted.
  - The game keeps its clamped variable step, with the fixed step that headless runs and tests already used (`fixed_step_dt`).
  - Entities stay in a plain `Vec`, which the reference tests rely on for order.
