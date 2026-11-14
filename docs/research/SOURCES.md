# Sources

The outside sources behind the game's design and the project's structure,
what each one says that mattered, and where it shows up in the code. They
were gathered in October 2026 in two rounds of research:

* **Horror design** (sections 1–7): what makes liminal horror suspenseful,
  for the work on threats, creatures, sound, rooms and props.
* **Rust game development** (section 8): how to lay out and organise a
  Rust game, for the workspace restructuring.
* **UI and controls** (section 9, a third round): layouts, reach,
  accessibility, HUDs and menus in first-person horror, for
  [`UI.md`](../design/UI.md).
* **Megastructure** (section 10, a fourth round): the Samosbor myth,
  megastructure SCPs and Backrooms levels, wayfinding and distant threats,
  for [`MEGASTRUCTURE.md`](../design/MEGASTRUCTURE.md).
* **Creatures** (section 11, a fifth round): rule-monsters, folklore and
  how entities are pictured, creature AI and the psychology of fear, CC0
  bodies, for [`ENTITIES.md`](../design/ENTITIES.md); the notes in full are in
  [`research/`](README.md).
* **The vending machine** (section 12, a sixth round): real machines from
  their manuals, and uncanny ones from SCP, the Backrooms, folklore and
  games, for [`VENDING.md`](../design/VENDING.md).

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
`docs/engineering/DEVELOPMENT.md` for the conventions that came out of it).

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

## 9. UI and controls

Gathered on 2026-10-26 by a research pass for [`UI.md`](../design/UI.md). Most
keybinding pages are fan guides; they are listed for the layouts, which
agree with each other, and marked *(secondary)*.

**Default layouts:** [Amnesia: Rebirth](https://www.magicgameworld.com/?p=47035), [Amnesia: The Bunker](https://www.magicgameworld.com/?p=106893), [Outlast](https://www.magicgameworld.com/outlast-pc-keyboard-controls-key-bindings/), [Outlast 2](https://www.magicgameworld.com/outlast-2-pc-controls-key-bindings-guide/), [Phasmophobia](https://www.magicgameworld.com/?p=125251), [Lethal Company](https://www.magicgameworld.com/?p=96749) *(secondary)*; [Alien: Isolation, the official manual](https://www.feralinteractive.com/en/manual/alienisolation/latest/steam/).
- Say: Shift run, Ctrl crouch (C in Alien), F light, E or left click to use, Tab for the inventory or notebook, R for batteries ("reload": Outlast, Alien), Q/E lean where there is lean. Items are used without stopping: a held item on the left button (Lethal Company, the Bunker) or a key beside WASD (Phasmophobia, Alien); Alien's item wheel runs in real time.
- Used for: the default layout in `session/input.rs` (E/left click, F/right click, R, Tab, C toggle beside a held Ctrl).

**[Phasmophobia quality-of-life patch: inventory hotkeys](https://games.gg/news/phasmophobia-qol-patch-inventory-hotkeys/)** *(secondary, summary only)*.
- Says: direct keys for the slots, because "your incense is sitting two scroll-wheel clicks away. You fumble, you die."
- Used for: a key per item (G/1 water, R/2 battery, Q paper), no scroll-to-select.

**[Game Accessibility Guidelines](https://gameaccessibilityguidelines.com/basic/)**, its page on [holds](https://gameaccessibilityguidelines.com/avoid-provide-alternatives-to-requiring-buttons-to-be-held-down/), and the **[Xbox Accessibility Guidelines 107](https://devdocs.xbox.com/gaming/accessibility/xbox-accessibility-guidelines/107)** and **[117](https://devdocs.xbox.com/gaming/accessibility/xbox-accessibility-guidelines/117)**.
- Say: remappable controls and sensitivity; "holding requires much greater motor ability than pressing", so offer toggles; avoid chords; let camera shake, bobbing and motion blur be turned off; a sensible, adjustable field of view; nothing essential by colour or sound alone; start the game without layers of menus.
- Used for: C toggles a crouch, a run toggle and a true off for head bob in Settings, the field-of-view slider, Continue focused on the title.

**[Create better game settings options: a checklist](https://www.gamedeveloper.com/design/create-better-game-settings-options-handy-checklist-)** (Game Developer).
- Says: sensitivity and invert per axis; brightness calibration matters most for dark games; separate volumes; back goes up one level; confirm before leaving with unsaved changes; pause on focus loss.
- Used for: the Settings page and its brightness mark (U4).

**[How to design a pause menu players appreciate](https://bugnet.io/blog/how-to-design-a-pause-menu-that-players-appreciate)** *(secondary)*.
- Says: pause must reliably stop gameplay (enemies, physics, timers); resume should be quick and obvious.
- Used for: Escape both pauses and resumes; the pause freezes the simulation and ducks the sound (U4).

**Input on macOS:** [Dear ImGui's macOS key-up fix](https://skia.googlesource.com/external/github.com/ocornut/imgui/+/6d5388448794f76cc92576a74ad3cb17da2fca84%5E%21/), [Cendio bug 8035](https://bugzilla.cendio.com/show_bug.cgi?id=8035), [Parallels KB 116171](https://kb.parallels.com/116171); [keyboard rollover](https://deskthority.net/wiki/Rollover,_blocking_and_ghosting) (Deskthority wiki).
- Say: macOS sends no key-up for keys pressed while Command is held; Ctrl+arrows is Mission Control's; cheap keyboards block a third or fourth key held at once.
- Used for: letting go of Command forgets the held keys (`on_key_up`); the Controls page tells arrow players to crouch with C; the run's actions sit on separate rows of the keyboard and on the mouse.

**[Beyond the HUD](https://publications.lib.chalmers.se/records/fulltext/111921.pdf)** (Fagerholt and Lorentzon, Chalmers / EA DICE) and **[Study: most players happy with traditional HUD overlays](https://gamedeveloper.com/design/study-most-players-happy-with-traditional-hud-overlays)** (Jørgensen, Game Developer).
- Say: diegetic, non-diegetic, spatial and meta UI; players accept a HUD that eases play, and a feature that no longer serves its purpose becomes annoying.
- Used for: the HUD at rest is empty, and what changes is shown, then fades (U5); not a HUD-less game.

**[Alien: Isolation](https://www.artofthetitle.com/title/alien-isolation/)** (Art of the Title) and **[its lo-fi design diary](https://www.dualshockers.com/latest-developer-diary-for-alien-isolation-details-the-games-lo-fi-sci-fi-design/)** *(secondary)*.
- Say: the in-world software had to look and behave like old software; overlays "easy to use and minimalist to look at"; the UI was recorded to VHS and played back.
- Used for: the screens speak as the building would (U3–U6).

**[The fonts in Control make its world feel way creepier](https://kotaku.com/the-fonts-in-control-make-its-world-feel-way-creepier-1837714580)** (Kotaku) and **[Fonts in Use: Control](https://fontsinuse.com/uses/48689/control-video-game)**.
- Say: "words written in Helvetica are culturally normalized to the point of seeming incidental"; institutional type makes the uncanny routine.
- Used for: Helvetica Neue as the building's voice (U2).

**[Glanceable fonts](https://www.nngroup.com/articles/glanceable-fonts/)** (Nielsen Norman Group).
- Says: no all-caps for longer passages.
- Used for: capitals only for a label of one to three words.

**Templated UI** *(secondary)*: [AI slop patterns](https://cdn.jsdelivr.net/npm/pixelslop@0.3.8/dist/skill/resources/ai-slop-patterns.md), [Impeccable's anti-patterns](https://mintlify.wiki/pbakaus/impeccable/concepts/anti-patterns), [How to detect AI slop in your design](https://github.com/Laith0003/ux-skill/wiki/How-to-detect-AI-slop-in-your-design).
- Say: monospace "for tech vibes", everything centred, a heading with a subtitle and one call to action, everything in a card, glowing accents on dark, subheadings that restate the heading; the test is whether a person made the choice or a default did.
- Used for: the analysis of the old screens in `UI.md`, and what the new ones avoid.

**Death and retries:** [SOMA's developer on death in horror](https://gamingbolt.com/soma-developer-wants-death-to-have-less-emphasis-in-horror-games) (GamingBolt), [There are things worse than death in video games](https://kotaku.com/there-are-things-worse-than-death-in-video-games-1573207459) (Kotaku).
- Say: repeated deaths kill fear; Amnesia played them down and changed the map a little after each.
- Used for: a low-key death card with no stats, and a wake-up on any key once it is up (U6).

## 10. Megastructure: the Building above the rooms

Gathered on 2026-10-27 by three research passes for
[`MEGASTRUCTURE.md`](../design/MEGASTRUCTURE.md). The Samosbor fan wikis refused the
fetcher; claims from them come from search excerpts and are marked
*(summary only)*. Wiki texts (SCP, Backrooms, Fandom) are CC BY-SA 3.0: the
game takes ideas from them, writes its own words, and credits each in
[`LORE.md`](../design/LORE.md), *Attribution*.

### Samosbor

**Origin:** [the archived 2ch /b/ thread](https://2ch.su/b/arch/2019-02-15/res/191363969.html) (the founding six lines, quoted, and "a year old in May"), [the Samosbor shooter's developers, DTF 2019](https://dtf.ru/indie/44492) ("began on 2ch in spring 2018"), [a history of the community, DTF 2020](https://dtf.ru/id136303/597176-samosbor-istoriya-soobshestva-ch-1), [igm.gg](https://igm.gg/media/d/chto-takoe-samosbor-russkii-scp-kotoryi-okazalsia-strashnee-backrooms-809e87af), [ixbt.games](https://ixbt.games/articles/2024/12/05/strax-i-nenavist-v-gigaxrushhevke-pogruzenie-v-klet-i-vselennuyu-samosbora.html) *(secondary)*.
- Say: an anonymous six-line post, spring 2018: "you live on floor 556 of structure P-46 / last night there was a samosbor again / it flooded even your floor / you're afraid to go out to the stairwell because someone is moaning there / the liquidators will come in a couple of hours at best / the smell of raw meat seeps into the room". The community's pillars: "the gigakhrushch is infinite, the samosbor is unknowable, life is mostly shit". No single author, no licence; written by "everyone and no one".
- Used for: the self-assembly as the Building's event; the canon never explains it.

**The setting and the event:** [VK "Modern foundations of the universe", 2023](https://vk.ru/@-166237605-sovremennye-osnovy-vselennoi), [VK "Samosbor. Basics", 2022](https://vk.ru/@proigrivatel-samosbor-osnovy), [VK "Samosbor: aftermath", 2021](https://vk.ru/@camocbop-samosbor-posledstviya), [the KLETKA wiki](https://kletka.wiki.gg/wiki/Samosbor), [Samosbor Archive (EN), Self-Assembly](https://samosborarchive-en.fandom.com/wiki/Self-Assembly) *(summary only)*.
- Say: a "conditionally infinite" panel-housing block with no entrance, no exit and no sky (the windows show other floors); dwelling cells, blocks, hermetic doors between sections, ration dispensers, garbage chutes, dead lift shafts. The event: a siren from sensors on some floors, minutes ahead (2–5 minutes; some floors have none), a fog crimson, lilac or violet, the smell of raw meat, screeching metal and voices of loved ones begging to open the door; shelter only behind a sealed hermetic door, opened only after the siren stops. After: slime, rearranged floors, liquidators who rake, burn, weld doors and pour foam concrete over whole floors, stairs and all, with a code knock answered from a plate on the door.
- Used for: the siren from speakers, the violet fog, the cells and their wheels, the voices at the door, the residents' notices and the liquidators' log (`MEGASTRUCTURE.md`, designs 4 and 6).

**Games:** [Samosbor (rocketship, 2020)](https://store.steampowered.com/app/1034060/Samosbor/), [KLETKA (2024–2026)](https://store.steampowered.com/app/1699480/KLETKA), [GIGAH|RUSH](https://tenevik.itch.io/gigahrush).
- Say: the siren cuts the expedition short and everyone runs for the lift or the door; KLETKA cuts the lore's minutes to about 30 s; floors come back changed.
- Used for: a warning long enough to reach a cell at a walk; the cell as the one shelter.

### Megastructure SCPs and Backrooms levels

All CC BY-SA 3.0; authors as the wiki credits them.

- [Backrooms Level 188 "The Courtyard of Windows"](https://backrooms-wiki.wikidot.com/level-188) by 1000dumplings, rewritten by Kai4C: an atrium of white concrete with windows at exact intervals, cold and silent. Used for: the Well.
- [SCP-2723 "The Prophet's Tower"](https://scp-wiki.wikidot.com/scp-2723) by HotCocoaNerd: open-sided floors round one stair in a fog, built on by something at the top that never stops. Used for: the Stair Halls (the architecture only).
- [SCP-4370 "Post-Industrial"](https://scp-wiki.wikidot.com/scp-4370) by UraniumEmpire: endless works too big to be underground, ceilings lost in the dark, machines running untended. Used for: the Works (no hunter).
- [SCP-5778 "Hidden In Plain Sight"](https://scp-wiki.wikidot.com/scp-5778) by MarcusLCrassus: concrete tunnels 15 × 20 × 180 m to one hub, placards and lights and air that never switch off. Used for: the Works' signs and lamps for nobody.
- [SCP-3333 "Tower"](https://scp-wiki.wikidot.com/scp-3333) by Jekeled: two figures on a far ridge who turn and go when looked at. Used for: the watchers across the void (the figures only; the place is a mountain lookout and is not used).
- [SCP-3074 "Kafka's Parking Garage"](https://scp-wiki.wikidot.com/scp-3074) by The Great Hippo: a car park of a hundred levels down, lights dying with depth. Used for: the Works' ramps.
- Also read, for the motifs: [SCP-1678 "UnLondon"](https://scp-wiki.wikidot.com/scp-1678) (AstronautJoe), [SCP-1730](https://scp-wiki.wikidot.com/scp-1730) (djkaktus), [SCP-4783](https://scp-wiki.wikidot.com/scp-4783) (caspian2), [Backrooms Level 162 "The Concrete Wall"](https://backrooms-wiki.wikidot.com/level-162) (Arkadelaide), [Level 280 "Foggy Staircases"](https://backrooms-wiki.wikidot.com/level-280) (jan Jejasa), [Level 696 "The Infinity-Scraper"](https://backrooms-wiki.wikidot.com/level-696) (LcY712), [Level 13](https://backrooms-wiki.wikidot.com/level-13) (Greggita Mahayfaio), and Piranesi's *Carceri d'invenzione* (1750; public domain).
- Say (together): megalophobia, kenopsia, no outside or sky, repetition, builders who never stop, infrastructure still running, signage for people who are gone, distant lights, a horizon made of architecture.

### Wayfinding and distant threats

**[Stop Getting Lost: Make Cognitive Maps, Not Levels](https://gdcvault.com/play/1027206/Stop-Getting-Lost-Make-Cognitive)** (Oueijan, GDC 2021; [summary](https://80.lv/articles/thatgamecompany-s-designer-on-how-not-to-get-lost-in-game-using-cognitive-maps/)) and **[the Level Design Book on wayfinding](https://book.leveldesignbook.com/process/blockout/wayfinding)**.
- Say: lost is a mismatch between the mental map and what you see; paths, edges, districts, nodes and landmarks; test without a HUD; edges bold, deep wide barriers (pits, chasms) steer most strongly; players don't look up unless something draws the eye.
- Used for: voids as the edges, the exit's light as the goal you see first, districts by material and light (design 2, 7).

**[A wayfinding study in virtual environments](https://research-portal.uea.ac.uk/en/publications/a-wayfinding-research-in-virtual-environments-the-effect-of-spati/)** (Yesiltepe et al., 2019) and **[Smith and Whitehead, Analyzing the expressive range of a level generator](https://www.pcgworkshop.com/archive/smith2010analyzing.pdf)**.
- Say: the number of decision points predicts wayfinding performance, landmark saliency did not; plot a generator's metrics over many seeds.
- Used for: the navigation measures of B2 and their bands, in `cargo xtask range`.

**[Breath of the Wild's world design (CEDEC 2017)](https://zeldauniverse.net/2017/10/09/a-deep-dive-into-nintendos-development-of-breath-of-the-wild/)** *(secondary)*, **[Dark Souls' interconnection](https://www.thegamer.com/dark-souls-1-fromsoftwares-magnum-opus-of-interconnected-level-design/)** *(secondary)*, **[Mirror's Edge: what went wrong](https://pixelpoppers.com/2010/01/mirrors-edge-what-went-wrong-and-why/)** *(secondary)*, **[the Level Design Book on layout typology](https://book.leveldesignbook.com/process/layout/typology)**.
- Say: places you see long before you reach; loops instead of trees; downhill reads as forward; a visual language for "reachable" must hold everywhere, or players get stuck when it thins; dead ends should pay.
- Used for: the door far and low, dead ends that end on a view, a parapet on every walkable edge and none elsewhere.

**[S.T.A.L.K.E.R.: Call of Pripyat](https://en.wikipedia.org/wiki/S.T.A.L.K.E.R.:_Call_of_Pripyat)**, **[Rain World's rain](https://rainworld.miraheze.org/wiki/Rain)** *(secondary)*, **[Lethal Company forum thread on the giants](https://steamcommunity.com/app/1966720/discussions/0/3958162152962363255)** *(secondary)*, **[Left 4 Dead's AI director (Booth, 2009)](https://cdn.fastly.steamstatic.com/apps/valve/2009/ai_systems_of_l4d_mike_booth.pdf)**.
- Say: an emission warns minutes ahead and you must reach a shelter; Rain World's rain is seen and heard before it comes, and shelters are where you rest; a giant you cannot hide from is unfair; a director changes how often, not how hard.
- Used for: the self-assembly (design 4): a siren, then the fog's surface seen rising, a cell always within reach of the warning at a walk, a cell the same everywhere.

### Photographs of Soviet-era places (the Building's surfaces)

Gathered on 2026-10-27 for the Building's surfaces (`visuals::photo`,
built by `assets/src/textures/build.py`). Searched: Wikimedia Commons
(categories *Khrushchev houses*, *Stairwells in Russia*, *Interiors of
factories in Russia*, *Granite floors in Ukraine*, *Terrazzo floors in
Russia*, *Floor tiles in Russia*, *Rustication (architecture)*, *Lenin
Mausoleum*, *Interior of the 16-story building in Pripyat*, and file
searches), with each file's licence read from the Commons API; and CC0
texture packs. Most Commons photographs of Soviet places are CC BY-SA, and
nothing there was taken unless it is CC0 (the manifest allows MIT, CC0 and
CC BY 4.0 only). Every photograph used is CC0, and each is checked by its
SHA-1 in `assets/src/textures/sources.json`.

- **[PanelkaPack](https://kureca.itch.io/panelka-pack)** by Kureca (CC0, "no generative AI"): photographs of post-Soviet panel housing, its precast fences and yards. Used for: the Well's facade (five photographed panels, each with its window, laid as twelve), its lit windows (the same windows lit from inside), its decks (a cracked, patched yard slab), the Works' floors (the same yard's crack-free parts, quilted into a factory slab saw-cut every 4 m), the Well's parapets and cells (a whitewashed precast fence, its lime flaking off the grey), the Works' walls (the bare fields of another precast fence, cast against steel forms, with the forms' seams and tie holes laid over them) and the Works' ribbed ceiling (that fence whole, cut rib to rib).
- **[Tverskaya Street 11, Moscow](https://commons.wikimedia.org/wiki/File:DSC_03930_moscow_july_2016_people.jpg)** by Rakoon (CC0, 2016): the rock-faced granite rustication of a Stalin-era ministry, built 1949. Used for: the Stair Halls' walls, its blocks cut out and laid again in running bond.
- **[Arsenalna station, Kyiv metro](https://commons.wikimedia.org/wiki/File:%D0%90%D1%80%D1%81%D0%B5%D0%BD%D0%B0%D0%BB%D1%8C%D0%BD%D0%B0%D1%8F_05.jpg)** by Antares 610 (CC0, 2013): the station of 1960, the deepest in the world, its floor of dark speckled granite. Used for: the Stair Halls' floor slabs.
- **[The Fabrika factory store, Moscow](https://commons.wikimedia.org/wiki/File:2026-05-27_%D0%A6%D0%A2%D0%98_%D0%A4%D0%B0%D0%B1%D1%80%D0%B8%D0%BA%D0%B0_-_%D1%81%D0%BA%D0%BB%D0%B0%D0%B4_-_%D0%BF%D0%BB%D0%B8%D1%82%D0%BA%D0%B0_%D0%BF%D0%BE%D0%BB%D0%B0_02.jpg)** by Retired electrician (CC0, 2026): cast-iron hexagonal floor plates in a 19th-century paper factory. Tried for the Works' floors and taken out: a blind review preferred the plain concrete on all 8 views (on walks, stairs and gantries the plates read as a repeated decorative tile).
- Looked at and not used (CC BY-SA, or not Soviet): Sofia's rusticated ministries, the Lenin Mausoleum's granite, Berlin's Soviet memorial, Pripyat's interiors (CC BY-SA 2.0 to 4.0), Metlakh-tiled stairwells (none free of share-alike), the CC0 "seamless floor" textures of Sisters.seamless (not from Soviet places).
- Say: the panel block's facade is its precast panels and their joints, a window to a panel, each window different (wood or PVC frames, net curtains, a grille); Stalinist buildings stand on rock-faced granite; the metro's floors are polished granite slabs; old Russian factories have cast-iron floor plates; precast fences and ribbed slabs are the same concrete everywhere.
- Used for: the Building's look as photographs (design 3), each surface graded to the mean colour of the canvas surface it replaced, so the floors keep their light. Judged blind against the canvas surfaces (`cargo xtask ab`, both orders, `tests/review/verdicts.jsonl`): the Well and the Stair Halls preferred on 16 of 16 views; the Works lost all 8 with iron plates, 5 of 8 with a slab under whitewash, then won all 8 with formwork walls and a worn slab (`MEGASTRUCTURE.md`, B9).

**Scale:** [Shadow of the Colossus's designers on fog and distance](https://www.siliconera.com/graphic-designers-discuss-creating-shadow-of-the-colossus-majestic-world/) *(secondary)*, [the Level Design Book on metrics](https://book.leveldesignbook.com/process/blockout/metrics), [BLAME! and videogame architecture](https://www.killscreen.com/influence-blame-videogame-architecture-rising/) *(secondary)*, [compression and expansion](https://en.wikipedia.org/wiki/Compression_and_expansion).
- Say: fog over far geometry, repeated modules at human scale, long empty transits as mild sensory deprivation, landings every 12–16 steps, a narrow way opening onto a large one.
- Used for: the look of the Building (design 3) and the arrival in a narrow place that opens onto the void.

## 11. Creatures: rule-monsters, folklore, creature AI and fear

Gathered on 2026-10-28 by five research passes for
[`ENTITIES.md`](../design/ENTITIES.md); the notes are kept whole in
[`research/`](README.md), every claim linked and every design's
licence stated. What the design took, and where it shows up:

**The rule, and the tell before it** ([`scp.md`](scp.md)).
- Read: SCP-173 (Moto42), 096 (Dr Dan), 106 (Dr Gears), 939 (Adam Smascher,
  EchoFourDelta), 513 (beefwit), 1499 (Trasknari), 2316 (djkaktus), all CC
  BY-SA 3.0; *SCP – Containment Breach*'s source (Regalis and contributors,
  CC BY-SA 3.0): 173 freezes while anywhere in the view cone, 106 shows a
  stain and a sound before it rises, 939 hears a sprint at 4.8 m and a
  crouch at 2.3 m.
- Say: the remembered monsters are one sentence each, and turn a normal act
  (blinking, looking, helping) into a risk; the checks are lenient and the
  rule strict; players accept a sudden death from a rule they understood
  and resent attrition or luck they cannot read.
- Used for: one rule per creature (design 1); the Watcher's lenient sight
  check; the one in the corner (after 096: its face is the trigger, a
  turning before the scream, a cap on how long it knows where you are).

**One sense, one counter, one brain** ([`creature-ai-and-fear.md`](creature-ai-and-fear.md)).
- Read: [Thompson on Alien: Isolation](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation), [Booth's Left 4 Dead slides](https://steamcdn-a.akamaihd.net/apps/valve/2009/ai_systems_of_l4d_mike_booth.pdf), [Botta, Infected AI in The Last of Us](http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter33_Infected_AI_in_The_Last_of_Us.pdf), [Walsh on Splinter Cell Blacklist's perception](http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter28_Modeling_Perception_and_Awareness_in_Tom_Clancy's_Splinter_Cell_Blacklist.pdf), [Welsh on search](http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter27_Looking_for_Trouble_Making_NPCs_Search_Realistically.pdf), Lethal Company's and Phasmophobia's wikis, Grip on SOMA's monsters.
- Say: one brain, kinds as data and skills; hearing as logical sounds along the way, blocked by walls; sight that builds up; an agitated tell before discovery; a creature that "knows" for 2–3 s after losing you, then searches and forgets visibly; light as a repellent fails (Naughty Dog cut it), light as a sense works; behaviours that unlock from habits, never deaths, and are seen when they unlock; no death without a warning, a grace period, a relief, pursuers slower than you, the first catch postponed.
- Used for: the brain (design 2), the fairness invariants (design 3), the habit ledger (design 4), the arenas and hard fails (design 7).

**Fear** (the same note).
- Read: [Andersen, Clasen et al. 2020, *Psychological Science*](https://pmc.ncbi.nlm.nih.gov/articles/PMC7734554), [de Berker et al. 2016](https://pubmed.ncbi.nlm.nih.gov/27020312/), [Schmitz et al. 2011](https://pmc.ncbi.nlm.nih.gov/articles/PMC3110515), [Rankin et al. 2009](https://pmc.ncbi.nlm.nih.gov/articles/PMC2754195/), [Mori 1970](https://spectrum.ieee.org/the-uncanny-valley), Tinwell's uncanny-valley studies.
- Say: enjoyment peaks at moderate fear with small swings; uncertainty stresses more than the threat; repetition habituates and a pause or a new stimulus restores; a face that does not startle reads as cold.
- Used for: builds that sometimes end in nothing, one unlock a level, faces that do not flinch (designs 4, 5, 6).

**How they are pictured** ([`folklore-and-imaging.md`](folklore-and-imaging.md)).
- Read: Henderson's interviews ([Daily Dead](https://dailydead.com/qa-trevor-henderson-discusses-the-found-footage-style-of-his-fascinating-horror-artwork/), [Reactor](https://reactormag.com/artist-trevor-henderson-siren-head-interview/)), [MacDorman et al. 2009](https://scholarworks.indianapolis.iu.edu/items/e8fb6458-ddad-4067-b877-1207d720ba0c), [McAndrew and Koehnke 2016](https://scottbarrykaufman.com/wp-content/uploads/2016/04/McAndrew-Koehnke-2016.pdf), Carroll's *The Philosophy of Horror*, [Long 2023 on analog horror](https://info.umkc.edu/sosland_journal/wp-content/uploads/2023/08/Long_AnalysisOfAnalogHorror.pdf), [Mirvoda 2018 on Soviet creepypasta](https://press.psu.ru/index.php/philology/article/download/1633/1204/3523); the upyr, Likho, the poludnitsa, the domovoi (public domain).
- Say: slightly-off proportions on realistic surfaces disturb more; creepiness is uncertainty; a hidden or almost-right face; eyeshine in a human head is the fusion of person and animal; stillness, or wrong mechanics; a sound first. Fully lit grins and "shadowed face plus realistic eyes" have turned comic; Soviet detail used as costume is a parody genre.
- Used for: each creature's look (design 1), eyeshine as a reflection (design 6), the Smiler's rictus seen only in the beam, the Reader's compulsion (after the upyr's seed and the slit-mouthed woman's candy), the one in the corner's face (the Thatcher illusion: one error in an upright face).

**The Backrooms and the Samosbor** ([`backrooms-and-samosbor.md`](backrooms-and-samosbor.md)).
- Read: the Backrooms Wikidot's Entities 3 (Smilers), 5 (Clumps), 8 (Hounds) and their authors, all CC BY-SA 3.0; the SAMOSBOR wikis (CC BY-SA); the founding 2ch thread (no author).
- Say: Wikidot's Smilers are drawn to light and provoked by panic; Clumps reach from where they lie; the samosbor is heard, never seen; voices beg at the door and must not be answered; what it leaves is fused and assembled, and its monsters are told apart by behaviour.
- Used for: the Smiler and the Hound (rules adapted, design 1), the Watcher and the one in the corner as a pair with opposite gaze rules (as Inside the Backrooms pairs its Hound and Smiler), and keeping "nothing walks the Building" (none of the samosbor's beings is brought into the game: its lesson is to tell creatures apart by behaviour).

**Bodies** ([`modelling.md`](modelling.md)).
- Read: [MakeHuman's licence](https://github.com/makehumancommunity/makehuman/blob/master/LICENSE.md) (section C: the base mesh, targets, proxies, skins, poses CC0 1.0), [Blender's Human Base Meshes](https://www.blender.org/download/demo-files/) (CC0), Poly Haven and ambientCG (CC0).
- Say: a real human body and its starved targets are free to use; relief is lost under a lamp held at the eye, so anatomy must be in the outline, the cavities and the glints; cloth drapes headless; eyeshine is retroreflection, strongest when the light is at the eye.
- Used for: the bodies (design 6: `assets/src/creatures/`, the hm08 body fetched and checked by SHA-1).

## 12. The vending machine: real machines and uncanny ones

Gathered on 2026-10-29 by two research passes for
[`VENDING.md`](../design/VENDING.md); the notes are kept whole in
[`research/`](README.md) (`vending-machines.md`,
`vending-horror.md`), every claim linked.

**The real machine** ([`vending-machines.md`](vending-machines.md)).
- Read: the factory manuals hosted by Vendors Exchange: [AP 113](https://www.vesolutions.co/manuals/AP_(Automatic_Products)manuals/Snack/AP-113-Snackshop-Parts-Manual.pdf), [AP 123](https://www.vesolutions.co/manuals/AP_(Automatic_Products)manuals/Snack/AP-123-Snackshop-Parts-Manual.pdf), [AP 133](https://www.vesolutions.co/manuals/AP_(Automatic_Products)manuals/Snack/AP-133-Parts-Operation-Service-Manual.pdf), the [Crane National 167/168 setup guide](https://www.vesolutions.co/manuals/Crane_National/167%20168%20Series/National-167-168-Setup-Guide.pdf) and [programming guide](https://www.vesolutions.co/manuals/Crane_National/167%20168%20Series/National-167-168-Programming-Guide.pdf), the [Dixie-Narco Series 90 guide](https://www.vesolutions.co/manuals/Dixie-Narco/Series%2090/DN-Series-90-Installation-Setup-Guide.pdf); [the CPSC's 1995 labelling campaign](https://www.cpsc.gov/Newsroom/News-Releases/1996/CPSC-Soda-Vending-Machine-Industry-Labeling-Campaign-Warns-Of-Deaths-And-Injuries); photographs on Wikimedia Commons (each licence on its page).
- Say: 72 × 38⅞ × 35 in; a 1133 × 683 mm tempered window and a ~25 cm selection column; five or six trays of five (bags) or ten (bars) spirals, each with its motor; one cool-white F18T8 tube across the top inside the door (the bottom shelves in gloom); a red LED reading "Have a nice day", "Use correct change", "Make another selection"; a 3 × 7 keypad of 15 mm keys (A–H, J, 1–9, *, 0, #); the lock-side front leg set lower, so a real machine leans; 1–4 in off the wall; rust and scuffs at kick height, sticker stacks, taped signs; a product sticks when one corner is behind the wire; 37 deaths and 113 injuries from rocking (1978–95).
- Used for: the body (V1: its size, window, column, trays and keypad), its light (one tube, the bottom shelves in gloom, the glow on the floor), its words, its lean, and its warning label (rule 7).

**The uncanny machine** ([`vending-horror.md`](vending-horror.md)).
- Read: SCP-261 (Dr Gears), SCP-294 (Arcibi), SCP-8733 (mothmanUXO), SCP-6593 (PlaguePJP), all CC BY-SA 3.0; Backrooms Levels 4, 6.1, 58, 187, 317 and Entity 36 (Wikidot), Levels 334, 986, 479 (Fandom), all CC BY-SA, with their authors in the note; Eiji Ohashi's *Roadside Lights* ([Hyperallergic](https://hyperallergic.com/409582/the-eerie-allure-of-japanese-vending-machines/)); Lethal Company's counter ([wiki](https://lethal-company.fandom.com/wiki/The_Company_Monster)); Control's altered vending machine; BioShock's Circus of Values; Japanese legends (motifs only).
- Say: the lone lit machine is the image (the only living light, which leads you and draws what else is there); price everything in what the game has, noise above all, and warn before you punish; rules, not dice, fixed per machine; no talking, no jump scares, no gore, and a prop never becomes a creature; the real warning label reads like the canon note.
- Used for: the rules (`VENDING.md`, the design): "insert coin, get nothing" as a silence (the note's *Change and Nothing*), the lit one's rhythm (*The Lit One*), rocking it loose and its fall (*Rock It*); nothing is taken from any one story.

## 13. The windows: real windows, and windows that look back

Gathered on 2026-10-30 by two research passes for
[`WINDOWS.md`](../design/WINDOWS.md); the notes are kept whole in
[`research/`](README.md) (`windows-real.md`, `window-horror.md`),
every claim linked.

**The real window** ([`windows-real.md`](windows-real.md)).
- Read: the US National Park Service's preservation briefs on [wood](https://www.nps.gov/orgs/1739/upload/preservation-brief-09-wood-windows.pdf) and [steel](https://www.nps.gov/orgs/1739/upload/preservation-brief-13-steel-windows.pdf) windows and its technical notes on [replacement sash](https://www.nps.gov/orgs/1739/upload/tech-note-windows-21-replacement-wood-sash.pdf) and [aluminium windows](https://www.nps.gov/orgs/1739/upload/tech-note-windows-22-historic-aluminum-windows.pdf); Marvin's and Andersen's manuals, Kawneer's framing; a contract mini blind's cut sheet; ASHRAE's history of post-war offices; the Fresnel equations, Toet and Alferdinck on dirty glass; van Dongen's interior mapping, Lagarde's parallax-corrected cubemaps, three.js's GroundedSkybox; Poly Haven's and ambientCG's CC0 licences and panoramas; photographs on Wikimedia Commons (each licence on its page).
- Say: a double-hung sash's upper sash is outboard, a meeting rail at mid-height, muntins 19 mm, a frame 116 mm deep; office strip windows are fixed lights between 50 × 114 mm mullions over a convector; glass reflects 4% a face square on and all of it at a graze, and at night a lit room's window is a mirror; windows age where water stands; mini blinds hang crooked; a view without parallax, a box with no reveal and clean glass look fake; a ground-projected panorama is exact for flat ground from any eye.
- Used for: the two kinds of window, their sizes and wear, the blinds, the glass, the view and how it is drawn (`WINDOWS.md` 1–2).

**The window that looks back** ([`window-horror.md`](window-horror.md)).
- Read: SCP-965 (ChazzK), SCP-1148 (Timmantha), SCP-675, SCP-419, SCP-2885 (Pythy, rewrite Zyn), SCP-508 (Hamsterhuey), SCP-7640, SCP-987-ARC, SCP-L682 (CRAt26), SCP-3052 (Witryso), SCP-2316 (djkaktus), SCP-397-JP, SCP-955-JP, SCP-CN-1779, SCP-CN-446, all CC BY-SA 3.0; the Backrooms' [Entity 2, "The Windows"](https://backrooms-wiki.wikidot.com/entity-2) (concept u/glassydude101, rewrite DrAkimoto) and Level 188 (1000dumplings, rewrite Kai4C), CC BY-SA 3.0; Kunekune and Yamanoke (2ch, motifs only); *Wuthering Heights*, *Dracula*, M. R. James, Gogol's *Viy*, Poe's "The Raven", Wells's "The Crystal Egg", Lovecraft's "The Music of Erich Zann" (public domain); Irish, Ukrainian and East Slavic customs; Slender, Look Outside and others (mechanics only).
- Say: the look that takes time and sharpens; a reach short and exact; covering as the counter, and a counter that must not be inverted (SCP-397-JP); a figure that faces you from every angle; through the glass without breaking it; what came in known by its shadow on the floor; a line on the floor it cannot cross; the outside that should not exist.
- Used for: the one behind the glass (`WINDOWS.md` 4): the look that adds up, its coming only while seen, its reach the lamp's light, the blind; nothing is taken from any one story.


## 14. Marks and secrets: route marks, hiding places, and things that change you

Gathered on 2026-10-30 by two research passes for
[`SECRETS.md`](../design/SECRETS.md); the notes are kept whole in
[`research/`](README.md) (`wayfinding-and-secrets.md`,
`secret-items.md`), every claim linked.

**Marks and hiding places** ([`wayfinding-and-secrets.md`](wayfinding-and-secrets.md)).
- Read: the Appalachian Trail Conservancy's *Marking the Trail*; the Scouts' tracking signs; cave and cave-diving markers (the BLM's cave worksheet, line arrows and cookies); FEMA's urban search and rescue marks; the Berliner Mieterverein on the shelter arrows; Grimm's *Hansel and Gretel* (public domain); the Level Design Book on wayfinding; Valve's *Left 4 Dead* graffiti, *Half-Life 2*'s lambda, *Rayman Legends*' secrets, *Link's Awakening*, *Outer Wilds*' signalscope; El-Nasr and Rao on attention in 3D scenes; the dead drops of Ames, Hanssen and Penkovsky; where people hide money (games and real reports; mechanics and facts only).
- Say: one mark in view at a time, by sight; few marks carry direction, and a turn is marked by an offset; an owner's quirk makes a mark someone's; marks wear and vanish; too many marks spoil a trail; notice before you find, a tell kept the same; a dead drop's signal is apart from it, says "loaded", and crossed says "taken".
- Used for: her marks and their mix (`SECRETS.md` 1), the flat sheets' even stamp, the hiding places and their tells, the chalk sign (`SECRETS.md` 4).

**Things that change you** ([`secret-items.md`](secret-items.md)).
- Read: Carroll's *Alice's Adventures in Wonderland*, the *Hymn to Demeter*, the *Kojiki*, Child 37, Hartland, Rossetti's *Goblin Market*, the *Odyssey*, Grimm (public domain); SCP-330 (Dr Kondraki), the first SCP-1162 (author unknown, via the Japanese archive), SCP-427 (Dr Ouros), SCP-268 (Pair Of Ducks), SCP-714 (Arcalane), SCP-207 (Aeish), SCP-178 (91stCataclysm), SCP-1916 (Smapti), SCP-832 (Inglonias), SCP-362 (PaulSharke), CC BY-SA 3.0; the Backrooms' Almond Water (Natedagreat563, Poliacci; 1000dumplings), Cashew Water (yoyoflux, Positive123; Taangeerine), Liquid Pain (ReyDay), Royal Rations (Sariastuff, zakkun_), Liquid Silence (Atom_Rutherford), Moth Jelly (KingofGarbage), CC BY-SA; the Samosbor wiki's food concentrate (ProfessorSacred, CC BY-SA); the Creepypasta Wiki's Midnight Game and *Please Take Just One* (Umbrello), CC BY-SA; omamori, the ladanka, morijio, the red envelope, found coins (folk); games' hidden-effect items (mechanics only); John Harris's rules of roguelike design.
- Say: an invitation with no explanation; the effect felt, then learned; two-sided changes in small steps; one taste binds a little, and can take away what calls you; two free and the third costs; a hole that gives back and takes; a charm that works sewn shut; a bad thing shown by a tell; nothing unknown may kill; caps on how far it all goes.
- Used for: what is put away and what stays with you (`SECRETS.md` 5); no SCP number or object name appears in the game.

## 15. The props: real things, barricades, things a hand can use

Gathered on 2026-10-31 for [`PROPS.md`](../design/PROPS.md); the notes are kept
whole in [`research/`](README.md) (`props-census.md`,
`barricades.md`, `props-interactions.md`), every claim linked.

**The census** ([`props-census.md`](props-census.md)).
- Read: the game itself, every kind of prop on 272 floors, graded by seven reviewers against `STYLE.md`; on what makes a model read as real: bevels and the highlight a real edge catches (Radiator's *Bevels in video games*; Polycount threads), wear where handling puts it, albedo ranges of real materials (Lagarde's chart; the Corona albedo list).
- Say: a third of the kinds read as toys; the toy is in the composition of primitives and the flat colour, not in the parts; the props that read as real were modelled whole from a real one, their print painted.
- Used for: what is remodelled and in what order (`PROPS.md` 1, the milestones).

**Barricades** ([`barricades.md`](barricades.md)).
- Read: the U.S. Fire Administration's *Board-up procedures*; the CITB's guide to temporary hoardings; lockdown guidance (UConn, the University of Houston); makers' sheets for plywood, dimensional lumber, steel desks, vertical files, stacking and task chairs, GMA pallets, boltless shelving, hollow-core doors and twin mattresses.
- Say: a board-up is a sheet over the opening, braced across and bolted through; a furniture barricade is the heaviest flat things against the opening, upended and wedged; nothing real is a pile in the middle of a room.
- Used for: where barricades go and what they are made of (`PROPS.md` 2).

**Things a hand can use** ([`props-interactions.md`](props-interactions.md)).
- Read: the game's own canon and rules (`LORE.md`, `PILLARS.md`, `UI.md`, `VENDING.md`, `WINDOWS.md`, `SECRETS.md`); the Backrooms wiki's Level 1 and Level 2 (CC BY-SA), where they fit the canon.
- Say: a verb is a real hand's; its rules are fixed per thing, never dice; its effects are never said; its sound comes from it; it never sets a creature hunting.
- Used for: the verbs (`PROPS.md` 3).


## 16. Your asides: choosing a line, and not saying it twice

Gathered on 2026-11-10 for [`VOICES.md`](../design/VOICES.md) (*Your asides*); the
note is kept whole in [`inner-voice.md`](inner-voice.md),
every claim linked.

**[AI-driven Dynamic Dialog through Fuzzy Pattern Matching](https://cdn.fastly.steamstatic.com/apps/valve/2012/GDC2012_Ruskin_Elan_DynamicDialog.pdf)** (Elan Ruskin, Valve, GDC 2012; slides with speaker notes).
- Says: an event is a query of facts matched against rules; the rule with the most criteria (the most specific) wins and equally specific ones are chosen between at random; what a line says can be written back as memory the next query reads; where no rule matches, nothing plays.
- Used for: the voice director's choice (`audio::asides`): a use and a subject, the subject's lines before the use's general ones, the run's memory of what was said read by every choice, and silence as an answer.

**Bark managers in practice** *(secondary)*: PulseGeek, Bugnet, StraySpark's Godot docs, an Audiokinetic Q&A (links in the note).
- Say: one manager decides every bark; rank by importance; a minimum gap between any two; drop what cannot be said in its moment rather than queue it; cooldowns per kind and fatigue per line, with shuffled variations.
- Used for: the asides' gap, priorities and moment (rule 5), and a line never twice in a run (rule 2).

## 17. Cutscenes: borrowing the camera and giving it back

Gathered on 2026-11-11 for [`CUTSCENES.md`](../design/CUTSCENES.md); the note is
kept whole in [`cutscenes.md`](cutscenes.md), every
claim linked.

**Unreal Engine's Sequencer** ([overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-sequencer-movie-tool-overview), [when a cinematic ends](https://dev.epicgames.com/documentation/unreal-engine/what-happens-when-my-cinematic-ends-in-unreal-engine), [camera cuts](https://dev.epicgames.com/documentation/en-us/unreal-engine/cinematic-camera-cut-track-in-unreal-engine), [blending to gameplay](https://dev.epicgames.com/documentation/en-us/unreal-engine/blend-gameplay-animation-to-cinematic-animation-in-unreal-engine)).
- Says: tracks of keyed sections, each with an end policy (*Restore State* or *Keep State*); input, look and HUD switched off while it plays; a camera cut may blend in from and out to the gameplay camera; set the control rotation and avoid an abrupt change of field of view at the hand-back.
- Used for: a cutscene as tracks of keys with a declared end (`then`), hands taken while it plays, the blend envelope over the live gameplay camera, and never moving your facing.

**Unity Timeline and Cinemachine** ([the director](https://docs.unity3d.com/cn/2018.4/Manual/class-PlayableDirector.html), [signals](https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/wf-signals.html), [Cinemachine and Timeline](https://docs.unity3d.com/Packages/com.unity.cinemachine@3.1/manual/concept-timeline.html), [blending](https://docs.unity3d.com/Packages/com.unity.cinemachine@3.1/manual/CinemachineBlending.html), [its source](https://raw.githubusercontent.com/Unity-Technologies/com.unity.cinemachine/main/com.unity.cinemachine/Runtime/Behaviours/CinemachineBrain.cs)).
- Says: wrap modes say what is left at the end; signals keep a timeline from naming gameplay code; a shot eases in and out against the live camera (2 s *Ease In Out* by default; *Linear* looks mechanical); a manual update evaluates the graph at a time you give.
- Used for: events by name, registered in code (`[[event]]`); every key eased (`in_out` by default); the camera a function of the cutscene's time.

**Valve's choreographed scenes** ([VCD, srctools](https://srctools.readthedocs.io/en/latest/modules/choreo.html)) and **Frictional's HPL2** ([script functions](https://oldwiki.frictionalgames.com/hpl2/amnesia/script_functions)).
- Say: a scene is a text file of timed events with ramp curves (Catmull-Rom among them), compiled to a binary form for the game; HPL2 has no timeline at all, only a capped-speed look-at, a head offset that slows near its end and a slow roll.
- Used for: the source written as TOML and compiled into the bundle the game embeds (`cargo xtask cutscene write`); positions on a centripetal Catmull-Rom spline; turns capped by the contract tests.

**Never taking the camera** ([how designers turn heads](https://gamedeveloper.com/design/how-designers-turn-heads), [Half-Life 2 at 20](https://www.bfi.org.uk/features/half-life-2-20-years-valve-shooter), [SOMA's pillars](https://frictionalgames.blogspot.com/2013/12/the-five-foundational-design-pillars-of.html), [loss of control](https://gamedeveloper.com/design/emotion-frustration-loss-of-control)).
- Says: Valve and Frictional keep the camera with the player and steer the eye with the world; losing control mid-play breaks immersion "like an actor looking at the camera".
- Used for: few, short cutscenes, each where only the camera can show it (a look into a shaft, a fall); everything else stays the place's job (`PILLARS.md`).

**Title screens into play** ([the title screen](https://www.superjumpmagazine.com/the-unappreciated-beauty-of-the-title-screen/), [immersive intros](https://www.gamedeveloper.com/design/how-to-create-immersive-game-intros)).
- Says: God of War's title leads straight into a new game; when play starts at once, leave a safe zone.
- Used for: the way in from the title into your eyes, in the Well's first quiet.

**Telling the player and letting them out** ([XAG 105](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/105), [XAG 104](https://devdocs.xbox.com/gaming/accessibility/xbox-accessibility-guidelines/104), [XAG 116](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/116), [Game Accessibility Guidelines](https://gameaccessibilityguidelines.com/full-list/), [The Evil Within's bars](https://www.kitguru.net/gaming/jon-martindale/bethesda-fixes-up-evil-within-pc-fps-black-bar-issues/), [why players skip](https://fredzed.substack.com/p/why-do-players-skip)).
- Says: hide the HUD while the camera is taken; cinematics with sound pause (under 3 s exempt); hold to skip; letterboxing *gameplay* drew complaints and was patched out.
- Used for: the bars only while your hands are taken, the HUD hidden, Escape pausing a cutscene, hold to skip.

**Comfort** ([Forsyth on VR sickness](https://developers.meta.com/vr/blog/vr-sickness-the-rift-and-how-game-developers-can-help/), [Meta's locomotion comfort](https://developers.meta.com/horizon/design/locomotion-comfort-usability/), [XAG 117](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/117), [GAG on VR triggers](https://gameaccessibilityguidelines.com/avoid-vr-simulation-sickness-triggers/)).
- Says: turning the view without input, changing the field of view and sudden accelerations are uncomfortable; keep the horizon level; walking is about 1.4 m/s; shake and bob must be avoidable. No source gives a flat-screen limit in degrees a second.
- Used for: the comfort limits the contract tests hold (eased moves, turns under 150°/s, roll only in a fall, the field of view moved at most 15°: our own numbers, as no source gives one) and Settings' shake and head bob applied to a cutscene's sway and shake.

**Death in first-person horror** ([Frictional on Alien: Isolation](https://frictionalgames.blogspot.com/2014/05/alien-isolation-and-two-hardest_5.html), [Grip at GDC Europe](https://gamedeveloper.com/business/gdc-europe-i-amnesia-i-s-grip-delivers-terrifying-tales-of-immersion), [Red Barrels](https://www.vice.com/en/article/how-a-horror-game-about-death-expanded-its-audience-by-letting-them-live), [The Callisto Protocol](https://www.nme.com/news/the-callisto-protocol-now-lets-you-skip-its-gruesome-death-animations-3370695), [Alien: Isolation's saves](https://gamedeveloper.com/design/game-design-deep-dive-the-save-system-of-i-alien-isolation-i-)).
- Says: after a death the fear is gone and a retry spoils it; change something after each death; the fifth replay of a sequence is not scary; long death animations were patched to be skippable; never keep the threat where it was on a reload.
- Used for: deaths short (the card at 1.4 s, as before), a short form after the first of its kind in a run (`repeat_from`), and the floor built again behind the card (as before).

**Testing** ([Fix your timestep](https://gafferongames.com/post/fix_your_timestep/), [Unreal's screenshot comparison](https://dev.epicgames.com/documentation/en-us/unreal-engine/screenshot-comparison-tool-in-unreal-engine), [skipping by the final keyframe](https://discussions.unity.com/t/ive-given-up-on-allowing-player-to-skip-certain-cutscenes/739274)).
- Says: sample by time at a fixed step; compare frames with tolerances; skip by setting the end state, never by fast-forwarding the simulation.
- Used for: the contract suite (the same camera at 30, 60 and 144 frames a second; the last frame the gameplay camera; a skip lands where play lands, its events run and its sounds not) and goldens at named times (`--cut-at`).


## 18. Visual QA: looking where nobody looks

Gathered on 2026-11-11 for [`VISUAL_PLAYTEST.md`](../engineering/VISUAL_PLAYTEST.md); the
note is kept whole in [`visual-qa.md`](visual-qa.md),
every claim linked.

**Screenshot tests in studios** ([Frostbite](https://enginearchitecture.org/downloads/Testing_Rendering_Code_at_Frostbite-REAC.pdf), [Unreal's screenshot comparison](https://dev.epicgames.com/documentation/en-us/unreal-engine/screenshot-comparison-tool-in-unreal-engine), [Aras on testing graphics code](https://aras-p.info/blog/2011/06/17/testing-graphics-code-4-years-later/), [Unity's image comparison](https://docs.unity3d.com/Packages/com.unity.testframework.graphics@8.5/api/UnityEngine.TestTools.Graphics.ImageComparisonSettings.html)).
- Says: many fixed cameras per level, a deterministic mode, noisy features (temporal AA, motion blur, bloom) off, strict per-pixel thresholds; a person approves the reference; every regression a person finds becomes a test.
- Used for: the tour's cameras placed by rule on every edge rather than by hand, the renders taken with the world frozen and the bare ones without the HUD (no grain), and each finding reproduced as a `--shots-at` camera before it is fixed.

**Z-fighting** ([OpenGL FAQ on polygon offset](https://opengl.org.ru/docs/faq/polygonoffset.htm), [depth precision visualized](https://developer.nvidia.com/content/depth-precision-visualized), [Vasilakis and Fudos' coplanarity detector](https://www.cs.uoi.gr/~fudos/coplanarity.pdf), [a geometric test](https://discussions.unity.com/t/how-to-check-for-z-fighting-ie-two-planes-directly-on-top-of-each-other/475692)).
- Says: coplanar faces from different meshes fight because of round-off; offset is for decals; a near plane pulled close costs depth precision; detection is by counting coplanar fragments per pixel or by testing faces for plane proximity and facing.
- Used for: the *flicker* detector (the jittered render diffed away from edges: our own, as no source documents it) paired with a geometric one (two faces in reach, of one thing or two, parallel, within millimetres and overlapping), each finding fixed in its geometry, not by offset.

**Holes and leaks** ([the hall of mirrors](https://beyondunrealwiki.github.io/pages/hall-of-mirrors.html), [Unity's undefined-pixel debug mode](https://docs.unity.com/en-us/engine/6000.0/script-reference/unityengine/rendering/loadstoreactiondebugmodesettings), [Lumen's thin walls](https://dev.epicgames.com/documentation/unreal-engine/lumen-technical-details-in-unreal-engine)).
- Says: where nothing is drawn, what was there before shows; uncleared pixels can be highlighted; walls thinner than 10 cm and one-sided meshes seen from behind leak.
- Used for: the *gaps* detector's sentinel background (our inference from these, checked by known answers: a hole put into a level is found, a clean view has none).

**The near plane** ([accurate collision zoom for cameras](https://gamedeveloper.com/programming/accurate-collision-zoom-for-cameras), [the generalized winding number](https://users.cs.utah.edu/~ladislav/jacobson13robust/jacobson13robust.html)).
- Says: you see through a wall when it crosses the near face of the view frustum; test that face, or its four padded corners, not one ray from the eye.
- Used for: the *clip* detector, which tests the near face's corners (and the eye) against every solid and every thing's bounds at each view.

**Out of place** ([EA SEED's texture classifier](https://arxiv.org/pdf/2406.08231), [Unreal's view modes](https://dev.epicgames.com/documentation/en-us/unreal-engine/viewport-modes-in-unreal-engine), [Unity's material validator](https://docs.unity3d.com/2020.1/Documentation/Manual/MaterialValidator.html), [art styles](https://unity.com/blog/game-art-styles-guide)).
- Says: stretched, low-resolution, missing and placeholder textures can be classified, but a white placeholder taught a classifier to flag white things; averaging several views of an object helps; albedo belongs in a range; one highly detailed object stands out in a plain scene and the reverse. Finding unlit or glowing things in a lit scene automatically: not found.
- Used for: the *plain* detector reading the materials themselves (no texture and no vertex colour, unlit, glowing) rather than guessing from pixels, a thing flagged only when several views agree, and the inspector's eyes for the rest.

**Vision models as reviewers** ([GlitchBench](https://arxiv.org/html/2312.05291v2), [VideoGameQA-Bench](https://arxiv.org/html/2505.15952v2), [Sony 2026](https://www.alphaxiv.org/abs/2607.25921), [bug-frame retrieval](https://arxiv.org/html/2508.04895v1), [FLIP](https://research.nvidia.com/publication/flip)).
- Says: the best models find under half the glitches in a frame and hallucinate clipping on near contact; use them as high-recall filters with a person on every flag; answers vary between runs; reviewers need a fast side-by-side view.
- Used for: the `visual-inspector` as a filter, never the gate (every finding it names is reproduced and looked at before it is fixed), and the report's strips (the look, the gaps, the flicker side by side) for a quick read.

## 19. The agents' trust: whose words an agent acts on

Gathered on 2026-11-14 for [`TRUST.md`](../engineering/TRUST.md); the note
is kept whole in [`agent-trust.md`](agent-trust.md), every claim linked.

**Issue-implementing agents** ([claude-code-action](https://github.com/anthropics/claude-code-action/blob/main/docs/security.md), [Copilot cloud agent](https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations), [gh-aw integrity](https://github.github.com/gh-aw/reference/integrity/), [GitHub MCP server](https://github.com/github/github-mcp-server)).
- Says: they trust write access at most; claude-code-action always reads the issue's body, whoever wrote it, and freezes the thread at trigger time; gh-aw filters each item by author association, and its endorsement reactions go stale when the item is edited; nobody ties a maintainer's approval in a comment to the version approved, or refuses agent-configuration changes because of who wrote the issue.
- Used for: our own tool and skill; an approval that covers words only as last edited before it (gh-aw's stale endorsement, applied to `/trust`); the CI agent fed only trusted threads, with unapproved authors excluded and `gh` denied.

**Protected files** ([gh-aw safe outputs](https://github.github.com/gh-aw/reference/safe-outputs-pull-requests/)).
- Says: an agent's pull request touching `.github/`, `CLAUDE.md`, `.claude/` or dot folders can be held for review or blocked.
- Used for: the protected paths, extended to every pull request (a person's too), refused outright from non-maintainers and held for `/trust-protected <sha>` when a maintainer implements others' words.

**GitHub's mechanics** ([`pull_request_target` from the default branch](https://github.blog/changelog/2025-11-07-actions-pull_request_target-and-environment-branch-protections-changes/), [blocked by default from 2026-12-02](https://github.blog/changelog/2026-11-02-pull_request_target-blocked-by-default-december-2), [rules](https://docs.github.com/en/rest/repos/rules), [roles](https://docs.github.com/en/organizations/managing-user-access-to-your-organizations-repositories/managing-repository-roles/repository-roles-for-an-organization)).
- Says: a `pull_request_target` job runs the default branch's workflow on the pull request's head and can write statuses; an `issue_comment` run cannot attach a check to the pull request but can post a commit status that a ruleset requires; public repositories block `pull_request_target` unless an Actions policy allows it; anyone with write access can edit others' comments.
- Used for: the gate's two triggers and its commit status, the Actions policy scoped to the gate's workflow, approvals not counted when someone other than a maintainer edited them.
