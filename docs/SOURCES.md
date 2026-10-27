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
  [`UI.md`](UI.md).
* **Megastructure** (section 10, a fourth round): the Samosbor myth,
  megastructure SCPs and Backrooms levels, wayfinding and distant threats,
  for [`MEGASTRUCTURE.md`](MEGASTRUCTURE.md).

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

## 9. UI and controls

Gathered on 2026-10-26 by a research pass for [`UI.md`](UI.md). Most
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
[`MEGASTRUCTURE.md`](MEGASTRUCTURE.md). The Samosbor fan wikis refused the
fetcher; claims from them come from search excerpts and are marked
*(summary only)*. Wiki texts (SCP, Backrooms, Fandom) are CC BY-SA 3.0: the
game takes ideas from them, writes its own words, and credits each in
[`LORE.md`](LORE.md), *Attribution*.

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

- [Backrooms Level 188 "The Courtyard of Windows"](https://backrooms-wiki.wikidot.com/level-188) by Kai4C: an atrium of white concrete with windows at exact intervals, cold and silent. Used for: the Well.
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

**Scale:** [Shadow of the Colossus's designers on fog and distance](https://www.siliconera.com/graphic-designers-discuss-creating-shadow-of-the-colossus-majestic-world/) *(secondary)*, [the Level Design Book on metrics](https://book.leveldesignbook.com/process/blockout/metrics), [BLAME! and videogame architecture](https://www.killscreen.com/influence-blame-videogame-architecture-rising/) *(secondary)*, [compression and expansion](https://en.wikipedia.org/wiki/Compression_and_expansion).
- Say: fog over far geometry, repeated modules at human scale, long empty transits as mild sensory deprivation, landings every 12–16 steps, a narrow way opening onto a large one.
- Used for: the look of the Building (design 3) and the arrival in a narrow place that opens onto the void.
