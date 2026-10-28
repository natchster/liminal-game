# Creature behaviour in horror games: complex, learnable, fair

Research notes for the entity redesign of "The Backrooms" (native). Part 1 covers case studies. Part 2 draws out principles. Part 3 covers the psychology of fear. Part 4 turns all of it into a behaviour architecture and a test plan for our game.

**Licence note.** Every game named in Part 1 is a copyrighted commercial work. We may take only general mechanics and motifs, such as "moves only when unseen" or "a warning that grows louder as it nears". We may not take names, likenesses, sounds or specific creature designs. Creatures in our canon that come from the SCP Wiki (SCP-087, -970, -1983, -3008, -3333) are CC BY-SA 3.0 and need attribution in the credits ([SCP Wiki licensing guide](https://scp-wiki.wikidot.com/licensing-guide)). Papers and talks are cited, not copied.

---

## Part 1 — Case studies

### 1.1 Alien: Isolation (Creative Assembly, 2014)

**Model.** The game runs two AIs. A "director" always knows where the player is. The alien does not, and it has to find the player with its own senses. Every so often the director sends the alien toward the player's general area, never to the exact spot ([Thompson, Game Developer 2017](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation)).

The director keeps a **menace gauge**, like Left 4 Dead's intensity value:
- Menace rises from proximity and visibility, not from attacks: fastest when the alien is a short walk away, in sight, or close on the tracker; slower when it is near but in another room.
- When menace peaks, the director pulls the alien "backstage" into the vents, aiming for vents ahead of where the player seems to be heading ([same](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation)).
- Its tunables include cooldown timers, time to peak, and the maximum number of menaces per front-stage sequence ([Thompson 2020, from the OpenCAGE mod tools](https://www.gamedeveloper.com/design/revisiting-the-ai-of-alien-isolation)).

**Behaviour tree.** The tree has 100+ nodes, with 36 branches at the root. Some branches start locked and unlock as the player shows certain behaviours. For example:
- Searching lockers and crawling into small vents unlock only after the player has escaped repeatedly by those means.
- Repeated flamethrower use moves its tactics from hissing, to vent ambushes, to flanking.

The unlock conditions are never tied to the player's deaths, so players cannot work out an exploit. Some branches also unlock at fixed campaign points as a backstop ([2017](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation), [2020](https://www.gamedeveloper.com/design/revisiting-the-ai-of-alien-isolation)).

**Senses** ([2020](https://www.gamedeveloper.com/design/revisiting-the-ai-of-alien-isolation)):
- **Vision.** Four view cones (normal, focused, peripheral, close). Detection builds up the longer the player stays in a cone, and readings decay over time.
- **Light.** A flashlight triggers detection almost at once.
- **Hearing.** Sounds have strength thresholds, so gunfire alerts it much faster than footsteps.
- **The player's tracker.** The alien hears the motion tracker's beep only within a very short radius.
- **Behind it.** It has no tail sensors, but short ray traces look behind it, so creeping up directly behind it fails ([2017](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation)).
- **Difficulty.** Novice cuts vision and movement-sound response by over a third.

**Search.** The alien visits search points in a deliberately sub-optimal order and backtracks, which reads as doubt. Designers can mark areas it never looks at, to prevent unfair ambushes. It teleports only twice in the whole campaign, both times for cutscenes ([2017](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation)).

**Fear vs frustration.**
- A third-person prototype was less scary, because the alien seemed to hunt the character rather than the player ([Hope, GDC 2015](https://gamedeveloper.com/design/how-creative-assembly-brought-fear-to-i-alien-isolation-i-)).
- Manual save stations split reviewers. One reviewer quit from "retreading" sections 20–30 minutes apart ([PCWorld](https://www.pcworld.com/article/2848244/alien-isolation-review-the-best-most-terrifying-alien-experience-in-nearly-30-years.html)). Another found that the high stakes made the game tenser ([Games Asylum](https://www.gamesasylum.com/?p=25614)).
- Frictional's Thomas Grip argued that a monster stops being frightening once you can stare at it from under a desk a metre away ([DSOGaming](https://www.dsogaming.com/news/frictional-games-dev-on-alien-isolation-horror-game-suffers-from-two-major-horror-design-issues)).

### 1.2 Left 4 Dead (Valve, 2008): the AI Director, the Witch, audio tells

**Director.** Mike Booth's slides set out the algorithm ([Booth, "The AI Systems of Left 4 Dead", 2009](https://steamcdn-a.akamaihd.net/apps/valve/2009/ai_systems_of_l4d_mike_booth.pdf)).

Each survivor has an intensity value. It rises:
- in proportion to damage taken;
- on being incapacitated;
- on being pulled off a ledge;
- when an infected dies nearby, more so the closer it is.

It decays toward zero, but not while infected are actively engaging that survivor.

The director then cycles through four states:

| State | What happens |
|---|---|
| Build Up | Full threat population until intensity crosses the peak threshold |
| Sustain Peak | Full population for 3–5 s more |
| Peak Fade | Minimal population until intensity decays and the fight ends |
| Relax | Minimal population (no mobs, no new specials) for **30–45 s**, or until the team has travelled far enough |

Booth stresses that the algorithm "adjusts pacing, not difficulty". Frequency changes; amplitude does not. Wanderer counts drop to zero in any area a survivor can see, and boss encounters are exempt from the modulation. Valve also measured skin conductance, heart rate and gaze in playtests ([Ambinder, GDC 2011 slides](https://cdn.cloudflare.steamstatic.com/apps/valve/2011/ValveBiofeedback-Ambinder.pdf)).

**The Witch** is a model of a graded warning ([L4D wiki](https://left4dead.fandom.com/wiki/The_Witch)):
1. She sits crying, which is audible long before she is seen.
2. If a survivor comes close, she stares and groans.
3. If they stay, she stands.
4. If they stay longer, she raises her arms.
5. Only then does she attack the one who startled her.

Writer Chet Faliszek said that hearing her does more than seeing her. Her music is so faint that players wonder whether it is playing at all ([Wikipedia, citing Rock Paper Shotgun](https://en.wikipedia.org/wiki/Witch_(Left_4_Dead))). Each special infected also has its own spawn sound, so players know a threat type is near without seeing it ([TechRaptor](https://techraptor.net/content/10-years-of-left-4-dead)).

### 1.3 The Last of Us (Naughty Dog, 2013): the Infected

These points come from [Botta, "Infected AI in The Last of Us", Game AI Pro 2](http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter33_Infected_AI_in_The_Last_of_Us.pdf).

**Structure.** Every Infected type uses one C++ class. Types differ only in their list of skills and their tuning data:
- Each type has a prioritised skill list. The Clicker's is On-fire > Chase > Search > Follow > Sleep > Wander.
- Wander is always valid, so a creature can never fall into an undefined state.
- Difficulty changes only the stimulus thresholds. On easy, a Clicker turns and barks at a nearby noise; on hard, it chases.

**Hearing.**
- The AI does not hear the real audio. It hears designer-placed "logical sounds" with set radii, and the team tied these to audible sounds wherever possible so players could predict reactions.
- The Infected hear about 6× better than human enemies, but less well while unaware of the player.
- Sounds are partly blocked by walls, checked by rays to each listener.
- The radius of the player's movement sound scales with player speed. Moving slowly is safe even up close, which stops crouching from being an all-purpose pass.
- An "agitated" state warns the player that they are being heard, before they are found.
- A silent "breathing" sound with a very short range lets the AI notice a player standing still nearby.

**What the designers cut, and why.** Both cuts matter for our Smilers:
- An echolocation design, where each bark briefly turned vision on, was dropped because it "confused players that were seen by a blind character".
- A light-sensitive creature that the flashlight could hold at bay failed. If it reacted quickly it did not feel dangerous; if it reacted slowly the flashlight felt useless. It became the Stalker, which moves from cover to cover in the dark, ambushes and falls back.

**Search.** Search points are generated from the last known position, branching out to wherever line of sight breaks, so they reveal hiding places. Choices are biased toward where the player is predicted to be. A "canvass" behaviour picks animations that reveal the most unseen grid cells, which looks frantic rather than methodical. The search is not exhaustive and ends in wander.

**Feedback.** On first contact the creature turns and screams before it chases. Mid-chase it pauses briefly to re-orient, which gives the player openings. A sleeping Infected stirs when the player is too fast or too close, then settles again. This teaches the rule without punishing the player.

### 1.4 Frictional Games: Amnesia: The Dark Descent, SOMA, Amnesia: The Bunker

**The Dark Descent (2010).** Sanity drops in the dark, at unsettling events and when **looking directly at monsters**. Low sanity brings hallucinations and draws monsters. A monster that sees the player chases until it loses sight ([Wikipedia](https://en.wikipedia.org/wiki/Amnesia:_The_Dark_Descent)). Penalising the player for looking keeps the monster half-seen. Fans report that after repeated deaths a monster may be moved away ([community account](https://giantbomb.com/users/6243/articles/are-you-so-blind)). Amnesia: Rebirth skips ahead past an encounter after a death, which players criticised as removing risk ([Dread Central](https://dreadcentral.com/editorials/457052/the-changing-face-of-death-in-the-amnesia-series)).

**SOMA (2015).** Grip's tests for a monster ([Game Developer interview](https://gamedeveloper.com/design/the-tricky-design-problem-of-i-soma-i-s-memorable-monsters)):
- Can the player build a mental model of it?
- Its rules should be "strict—yet still fuzzy".
- If it is too mechanical, it is "uninteresting"; if it is too random, it is "a haze".
- Monsters whose hostility was ambiguous confused playtesters, so the team made them clearly docile or clearly aggressive.

On death, Grip argued that a gory death scene is a relief rather than a scare, and that death should be postponed but still happen eventually, or players learn the monster is harmless ([Bloody Disgusting](https://bloody-disgusting.com/news/3292474/frictional-wants-postpone-death-soma/); [GamingBolt](https://gamingbolt.com/frictional-games-dev-talks-about-alien-isolation-game-suffers-from-two-major-horror-design-issues)). SOMA's later Safe Mode keeps the monsters but makes them curious instead of lethal ([PC Gamer](https://www.pcgamer.com/au/frictional-on-designing-somas-new-monster-free-safe-mode)).

**The Bunker (2023).** This is a sandbox with one unscripted stalker:
- The Beast hears movement: running, knocking things over, and the noisy dynamo flashlight, which lasts about a minute per wind ([Superjump](https://www.superjumpmagazine.com/amnesia-the-bunker-takes-two-scary-steps-forward)).
- It hates light. A running generator keeps it in the walls, and fuel is the clock ([Wikipedia](https://en.wikipedia.org/wiki/Amnesia:_The_Bunker)).
- Damaging it sends it into hiding for a few minutes.
- Item placement is randomised for replayability, a choice Olsson describes as avoiding "linearity and predictability".
- One reviewer found the game easy, and no longer frightening, once item locations and the Beast's behaviour were learned ([Superjump](https://www.superjumpmagazine.com/amnesia-the-bunker-takes-two-scary-steps-forward)). Learnability has a cost: mastery ends dread, so the threat must keep changing in small ways.

### 1.5 Resident Evil 2 remake (Capcom, 2019): Mr. X, the stalker

Mr. X tracks the player across rooms, unlike the original game's Tyrant. When he loses the player he searches rooms on random routes. Gunfire draws him most strongly ([PC Gamer interview with director Kadoi](https://www.pcgamer.com/uk/resident-evil-2s-director-talks-mr-xs-ai-scary-footsteps-and-the-dmx-mod/)).

His telegraph is honest: the footsteps are emitted by the body, so hearing them close means he is close ([We Got This Covered, on the same interview](https://wegotthiscovered.com/gaming/xs-footsteps-created-resident-evil-2/)). He walks, and the player can outrun him. Off-screen he reportedly moves at about 4× his walking speed, without teleporting ([Wikipedia, citing Eurogamer](https://en.wikipedia.org/wiki/Mr._X_(Resident_Evil))). He was designed in the same stalker mould as Nemesis and RE7's Baker family (same source).

Reception was split. Critics called him "panic-inducing" but also "the best and worst part" of the game, and a mod that removes him was popular ([same](https://en.wikipedia.org/wiki/Mr._X_(Resident_Evil))).

### 1.6 Silent Hill 2 and P.T.

**Silent Hill 2.** The pocket radio, carried over from the first game, hisses with static when monsters are near. It is an early-warning system that scares in its own right ([Museum of Play](https://www.museumofplay.org/games/silent-hill/); [PC Gamer](https://www.pcgamer.com/reinstall-silent-hill-2/)). Late in the game, in the prison, the static crackles and nothing is there, which turns the player's trust in the tell against them (same PC Gamer piece). Pyramid Head's designer, Masahiro Ito, set out to make "a monster with a hidden face" ([Wikipedia](https://en.wikipedia.org/wiki/Pyramid_Head)).

**P.T. (2014).** Lisa can kill the player at only "a few moments", but the player cannot tell which, so she feels all-knowing. A fan found that for much of the game she walks directly behind the player ([Stanford Daily](https://www.stanforddaily.com/2020/02/05/p-t-a-chilling-gaming-experience/)). The fear comes from a threat that is rarely live but always possible.

### 1.7 Outlast (Red Barrels)

The player cannot fight and must hide in lockers or flee. Audio cues tell the player they have been seen. Night vision drains scarce batteries ([Wikipedia](https://en.wikipedia.org/wiki/Outlast)). Outlast 2 was criticised for "trial-and-error" chases with unclear exits, for enemies that went straight to the player's hiding spot, and for repetition that made scares "banal" ([PCWorld](https://www.pcworld.com/article/3193037/outlast-2-review-fear-turns-to-frustration.html); [GameSpot](https://www.Gamespot.Com/reviews/outlast-2-review/1900-6416662/)). It shows what an unreadable pursuer costs.

### 1.8 Dead by Daylight (Behaviour Interactive): the terror radius

A survivor inside the killer's radius hears a heartbeat ([DbD wiki](https://deadbydaylight.wiki.gg/wiki/Terror_Radius)):
- The radius is **32 m** for killers moving at 4.6 m/s and **24 m** at 4.4 m/s.
- It is split into three equal layers, and the heartbeat grows louder and faster as the killer nears.
- Chase music replaces the closest layer.
- Stealth is an explicit, readable state. Myers' "Stalker" tier has a 0 m radius and 4.2 m/s; "Pursuer" has 16 m and 4.6 m/s; "Evil Incarnate" has 32 m.

The lesson for a gaze creature: let it grow louder as it "charges up".

### 1.9 Lethal Company (Zeekerss, 2023): one rule per monster

Each monster is a separate puzzle with its own counter. The details come from wikis and guides, so treat the timings as approximate.

| Monster | Rule | Tell | Counter |
|---|---|---|---|
| Coil-head | Moves fast only when nobody looks at it | Head wobbles on its spring when seen | One player watches it; it cannot be killed ([GamesRadar](https://gamesradar.com/lethal-company-enemies)) |
| Bracken | Sneaks up from behind and kills | Eyes visible even in darkness ([Keengamer](https://www.keengamer.com/articles/guides/how-to-escape-the-bracken-in-lethal-company/)) | A glance makes it back off; a long stare provokes it ([wiki](https://lethalcompany.wiki.gg/wiki/Bracken)) |
| Jester | Follows, then winds its crank and pops; afterwards it knows where everyone is | Music box, about 20 s ([number13](https://en.number13.de/lethal-company-all-monsters-and-what-you-can-do-against-them/)) | Leave the building during the wind-up ([wiki](https://lethalcompany.wiki.gg/wiki/Jester)) |
| Nutcracker | Sees only movement; 60 m range, 70° field of view | A scan turn every 1–4 s; the first shot takes 1.75 s to fire | Stand still while it patrols; when aggravated it fires at the last known position for 12 s ([wiki](https://lethalcompany.wiki.gg/wiki/Nutcracker)) |
| Thumper | Sees but cannot hear (per guides); charges straight | — | Turn corners ([number13](https://en.number13.de/lethal-company-all-monsters-and-what-you-can-do-against-them/)) |
| Snare flea | Drops from the ceiling onto the head | Visible on the ceiling | Look up (same source) |

For Bracken, the screen centre counts as "looking", and the edge of the screen does not ([wiki](https://lethalcompany.wiki.gg/wiki/Bracken)). That is a precise, learnable definition of "looking".

### 1.10 Phasmophobia (Kinetic Games)

A ghost's type is deduced from evidence and tells. Most ghosts can start a hunt once average sanity is at or below 50%; some types use other thresholds ([Fandom: Hunt](https://phasmophobia.fandom.com/wiki/Hunt)). When a hunt starts:
- the exits lock;
- nearby lights and electronics malfunction;
- a **grace period** follows in which the ghost cannot see the player: 5 s on Amateur, falling to 2 s on Nightmare (same source).

The tells for "a hunt is starting" are environmental: lights and electronics.

### 1.11 Five Nights at Freddy's (Scott Cawthon, 2014)

The behaviour is rule-based. Roughly every 5 s, each animatronic rolls 1–20 and moves if the roll is at or below its AI level, which rises during the night. Foxy advances only while the player is not watching the cameras ([FNAF wiki](https://fivenightsatfreddys.fandom.com/wiki/Foxy); [ModDB technical notes](https://www.moddb.com/members/what-can-we-use/blogs/technical-fnaf-facts)). A simple random rule, made visible through cameras and sound, is enough to create dread.

### 1.12 Hello Neighbor (Dynamic Pixels, 2017): learning done badly

The marketing promised an AI that "learns from your every move" by laying traps and cameras on routes the player reuses ([Epic store page](https://store.epicgames.com/p/hello-neighbor?lang=en-US)). Reviewers found it opaque instead:
- It seemed to know which cupboard the reviewer hid in ([Checkpoint Gaming](https://checkpointgaming.net/reviews/2017/12/hello-neighbor-review-confusing-horror-with-illogical-puzzles/)).
- It had "no routine you can plan around", and PC Gamer scored the game 38/100 for "trial-and-error drudgery" ([PC Gamer](https://pcgamer.com/hello-neighbor-review)).

Adaptation the player cannot read feels like cheating.

### 1.13 Hunt: Showdown and smaller titles

- **Hunt: Showdown.** Every sound has a real source and falls off realistically with distance. Crows, ducks and chained dogs are "sound traps" that give away whoever disturbs them ([PlayStation Blog](https://blog.playstation.com/?p=221483)). Crouching through hanging chains and cans keeps them quiet ([guide](https://kosgames.com/hunt-showdown-beginners-guide-2021-8865/)). For us: the environment can announce both the player and the creature.
- **Iron Lung.** The player sees outside only through grainy photos ([Wikipedia](https://en.wikipedia.org/wiki/Iron_Lung_(video_game))).
- **Dredge.** A panic meter rises at night and drives phantom rocks and pursuing monsters. Lights lower panic but draw some enemies ([StrategyWiki](https://strategywiki.org/wiki/DREDGE/Panic)), the same light trade-off as The Bunker.

**Not covered:** I found no documented creature-behaviour model for Scorn, Inscryption, Mouthwashing, Fears to Fathom, The Mortuary Assistant, Content Warning or Voices of the Void beyond fan guides.

### 1.14 Lessons from stealth AI

- **Splinter Cell Blacklist** sets four goals for perception: fairness, consistency, good feedback, intelligence ([Walsh, Game AI Pro 2](http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter28_Modeling_Perception_and_Awareness_in_Tom_Clancy's_Splinter_Cell_Blacklist.pdf)).
  - Vision shapes start a **detection timer**, scaled by distance, light and the guard's state.
  - Sound travels room to room through doors and windows rather than in straight lines.
  - Guards who are off-screen but plausibly too far away to hear had their hearing **halved**. The game "instantly became more fun".
  - Barks were tiered: specific ("I think I heard footsteps") the first few times, then generic. Players learn the rule, and then the reminders stop repeating.
- **Rich Welsh's search chapter** ([Game AI Pro 2](http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter27_Looking_for_Trouble_Making_NPCs_Search_Realistically.pdf)) describes:
  - a *cautious* search after a mere noise: check the spot, then give up;
  - an *aggressive* search after losing a known target: last known position first, then a sweep of hiding spots with a time limit;
  - letting the NPC "know" the target's position for **2–3 s** after losing sight, a trick used in Halo, Crysis and Crackdown;
  - a weak bias toward the player's true position. A strong bias makes the AI feel "unfairly stacked against the player".

---

## Part 2 — Principles

### 2.1 Warn by sound before sight, in steps

Strong designs announce a threat in steps, each cheaper to answer than the next:

| Stage | Examples |
|---|---|
| Ambient sign | Mr. X's footsteps, the SH radio, a crying Witch, L4D spawn sounds, Phasmophobia's flickering lights |
| Rising signal | DbD's three heartbeat layers, the Jester's crank, the Witch's stand-up, the TLOU agitated state |
| Last chance | The Nutcracker's 1.75 s first shot, the Infected's scream before a chase, the Alien's attack animation |
| The act | |

Players should be able to trust every stage. Silent Hill can subvert the radio only once, late, after the player has learned to trust it. The warning signal should come from a real source, as Mr. X's footsteps and Hunt's sounds do. TLOU gives an exception: a hidden "breathing" sound lets the AI notice a player standing still at very close range.

### 2.2 Give each creature a role

| Role | Behaviour | Examples | What the director does |
|---|---|---|---|
| **Stalker** | Persistent and slow; knows roughly where you are; you can outrun it | Mr. X, The Bunker's Beast | Sends it; pulls it back at peak menace |
| **Hunter** | Driven by its senses, searches, can be lost | Alien, Clickers | Hints at your area, never your spot |
| **Ambusher** | Hides, strikes, falls back | TLOU Stalker, Bracken | Places it in ambush spots, ahead of the player's path |
| **Sentinel / rule monster** | Static rule, graded warning, avoidable | Witch, Nutcracker, Coil-head | Places it, never sends it |
| **Environmental hazard** | A timed event that changes the space | L4D horde, Dredge's panic, Phasmophobia hunts | Schedules it in Build/Peak |

Mixing roles gives variety. Two creatures with the same role and counter blur together.

### 2.3 Layer the senses, and let the player read each one

- **Sight.** Use a volume and a timer, not an instant cone: detection builds while you stay in view. Alien has four cones; Splinter Cell uses coffin-shaped boxes, because a cone sees too far to the sides at long range.
- **Hearing.** Make it logical, not literal: a radius per action, scaled by speed (TLOU) and by the creature's state (TLOU's unaware penalty), and travelling through doorways and blocked by walls (Splinter Cell, TLOU).
- **Light.** Treat light as a sense to keep it fair. Alien detects a flashlight almost at once. The light trade-off works when light costs something (The Bunker's noisy dynamo, Dredge's lamps drawing enemies). It fails as a magic repellent, as TLOU's prototype showed (Part 1.3).
- **Touch, smell, vibration.** Bumps and footfalls work at close range only. Remember that "being seen by a blind creature" confuses players.
- **Fairness.** Halve the hearing of creatures that are off-screen and far away (Splinter Cell). Lower the senses of an unaware creature (TLOU).

### 2.4 Memory and investigation

- **Last known position, plus 2–3 s of "intuition"** (Welsh).
- **Cautious search after a noise** (go to it, look, give up). **Aggressive search after losing you** (last known position, then hiding spots, with a time limit), choosing points that reveal hiding places (TLOU), in a doubtful order (Alien).
- **Forget on a timer and show the forgetting.** The Nutcracker stays aggravated for 12 s; TLOU creatures drift back to wander. Return to normal behaviour gradually, not all at once (Welsh).
- **No-search zones** chosen by designers rule out unfair ambush spots (Alien).

### 2.5 Learning the player can see

Alien's unlockable branches work because:
- each unlock is a *new behaviour*, not a stat change;
- each is tied to a player *habit*, never to deaths;
- each is visible when it happens (the alien now pulls lockers open);
- campaign backstops keep progression even.

Hello Neighbor fails the same test by being opaque. Grip's question is the acceptance test: can the player make a mental model of it?

### 2.6 Fairness rules

1. **No death without a warning** the player could perceive and act on (Witch stages, Jester wind-up, the Nutcracker's first-shot delay).
2. **A grace period whenever a threat begins** (Phasmophobia: 2–5 s by difficulty).
3. **A cooldown after a peak** (L4D Relax 30–45 s; Alien's backstage retreat).
4. **Pursuers are slower than a sprint**, or the pursuit can be broken in some other way (Mr. X walks; the Thumper loses you at corners). A threat that cannot be escaped once it starts must be avoidable before it starts (the Jester).
5. **Postpone death.** The first catch hurts rather than kills, or the creature backs off (Grip; Amnesia's despawning; the SOMA approach). Death must still be possible eventually.
6. **Keep the cost of death low but real.** Long replays turn fear into chores (Alien's spaced saves, Outlast 2's chases). Skipping the encounter removes risk (Rebirth).
7. **First encounters teach.** TLOU's sleeping Infected stir before they wake; Splinter Cell's first barks name the rule. The first meeting with each creature should show its rule without killing.

### 2.7 Testing beatability with bots

Public detail on how horror studios test completability is thin. The general methods are well documented:
- **Personas.** Bots that differ only in the utility weights of their goals, used to judge levels ([Holmgård et al., AIIDE 2015](https://ojs.aaai.org/index.php/AIIDE/article/view/12849); [personas as critics for generated dungeons](https://mail.antoniosliapis.com/papers/procedural_personas_as_critics_for_dungeon_generation.pdf)). A small set of hand-weighted personas captures human decision styles about as well as clones trained on human play ([personas vs clones](https://mail.antoniosliapis.com/papers/evolving_models_of_player_decision_making.pdf)).
- **RL agents** to find exploits, test map difficulty and extend coverage ([EA SEED, Bergdahl et al. 2020](https://arxiv.org/pdf/2103.15819); [curiosity-driven coverage](https://arxiv.org/pdf/2103.13798)).
- **Death heatmaps** from telemetry (Bungie's Halo 3 lab; [NPR](https://www.npr.org/transcripts/14649338)) and biometrics (Valve, above).

---

## Part 3 — The psychology of fear

### 3.1 Recreational fear has a "sweet spot"

Clasen argues that we evolved to enjoy simulated threat because it lets us rehearse danger safely ([*Why Horror Seduces*, OUP 2017, review](https://americanaejournal.hu/index.php/americanaejournal/article/view/45429/44083)). His lab tested this in a 42-room commercial haunted house ([Andersen, Schjoedt, Price, Rosas, Scrivner & Clasen 2020, *Psychological Science*](https://pmc.ncbi.nlm.nih.gov/articles/PMC7734554)):
- 110 visitors took part (92 analysed). They wore heart-rate monitors and were filmed at three jump-scare points.
- **Enjoyment followed an inverted U against self-reported fear.**
- Fear followed *large-scale* heart-rate swings linearly. Enjoyment followed *small-scale* fluctuations along an inverted U.
- The authors read this as enjoyment needing arousal that is "just right" ([APS summary](https://www.psychologicalscience.org/observer/playing-with-fear)).

For design: keep the player near the edge, with frequent small swings in arousal, rather than long stretches at full alarm.

### 3.2 Uncertainty, prediction and learning

In predictive-processing terms, we enjoy situations where prediction error falls *faster than expected*. Horror supplies uncertainty that is threatening but can be resolved, and that sits at the edge of the player's ability ([Miller, White & Scrivner 2024, *Phil. Trans. R. Soc. B*](https://pmc.ncbi.nlm.nih.gov/articles/PMC10725765/)). The same group treats play in general as deliberately creating uncertainty and then reducing it ([Andersen, Kiverstein, Miller & Roepstorff 2023, *Psychological Review*](https://philpapers.org/s/Marc%20Andersen); [summary](https://bigthink.com/?p=595607)).

Uncertainty itself drives stress. Stress ratings, pupil size and skin conductance tracked *irreducible uncertainty* about electric shocks, more than the shocks themselves ([de Berker et al. 2016, *Nature Communications*](https://pubmed.ncbi.nlm.nih.gov/27020312/)).

For design: rules must be learnable (error can fall), but *when* and *where* must stay uncertain. That is P.T.'s Lisa and the menace gauge's back-off.

### 3.3 Dread and startle are different

- **Predictable threat** (a cue, then a shock) gives short *fear* spikes during the cue. **Unpredictable threat** raises startle *between* cues, which is sustained *anxiety* ([Schmitz et al. 2011, NPU-threat](https://pmc.ncbi.nlm.nih.gov/articles/PMC3110515)).
- Waiting itself costs. In fMRI, some people chose a *stronger shock sooner* over a milder one later ([Berns et al. 2006, *Science*](https://pubmed.ncbi.nlm.nih.gov/16675703/)).

For design: dread comes from the time between tells and outcomes. Startles are spikes; spend them sparingly.

### 3.4 Habituation, and how to avoid it

Rankin et al. ([2009](https://pmc.ncbi.nlm.nih.gov/articles/PMC2754195/)) list the features of habituation:
- Responses fall with repetition, faster when the stimulus comes more often.
- They recover after a pause (spontaneous recovery).
- A *different* stimulus restores them (dishabituation), though the dishabituating stimulus itself wears off with repetition.
- Weak stimuli habituate faster; very strong ones barely do.

In film, self-reported suspense and skin conductance fell between first and second viewings, while enjoyment held steady ([Chun, Park & Shi 2020, *Frontiers in Psychology*](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2020.558234/full)).

For design:
- Space out each creature's appearances. The relax phase supports spontaneous recovery.
- Rotate which creature and which tell leads.
- Make the rare event strong.
- Bring in new behaviours (the unlockables) as dishabituators, one at a time.

### 3.5 The uncanny valley

Mori proposed that affinity collapses into eeriness as a figure approaches, but misses, a human appearance. He held that movement deepens the effect ([Mori 1970, authorised translation, IEEE Spectrum 2012](https://spectrum.ieee.org/the-uncanny-valley)). Tinwell's experiments find:
- Characters with **little upper-face movement** (brows, lids, forehead) are rated more uncanny, especially for fear, sadness, disgust and surprise ([Tinwell et al. 2011](https://vbn.aau.dk/en/publications/facial-expression-of-emotion-and-perception-of-the-uncanny-valley/)).
- Characters that **do not startle at a scream** are rated most uncanny, and are read as psychopathic: cold and untrustworthy ([Tinwell, Abdel Nabi & Charlton 2013](https://ub-ir.bolton.ac.uk/esploro/outputs/journalArticle/Perception-of-psychopathy-and-the-Uncanny/9913397408841)).
- These findings are collected in [*The Uncanny Valley in Games and Animation* (CRC Press, 2014)](https://www.taylorfrancis.com/books/9780429102349).

Noël Carroll's theory adds that horror monsters are "interstitial": they break categories by fusing the human with the impure ([summary](https://caitlinduffy.hcommons.org/?p=742); [Routledge](https://www.routledge.com/products/9780203361894)).

For design: creatures that are almost people (Hounds, staff, the seated man) should have a still, unreactive upper face. They must not flinch when the player cries out.

### 3.6 Showing the monster too much

- Spielberg's mechanical shark kept failing, so he had to make *Jaws* "without seeing the shark itself", and the film was scarier for it ([The Take](https://the-take.com/read/ask-the-professor-how-did-problems-with-the-mechanical-shark-in-ajawsa-lead-to-a-brilliant-moment)).
- Grip: a monster examined from a metre away "destroys the player's imagination".
- Faliszek: hearing the Witch builds her up more than seeing her.
- Ito designed a hidden face on purpose.
- Amnesia charges sanity for looking.
- Iron Lung allows only photographs.

All of these make the full sight of the creature rare, expensive or partial.

---

## Part 4 — Implications for our game

### 4.1 Where the code stands (native/crates/game/src/entities)

| Area | Current behaviour |
|---|---|
| Director | Already runs **Calm → Build → Peak → Relax** with a menace gauge, steering toward your area, a final push and a back-off (`director.rs`). This matches Alien and L4D. |
| States | wander / search / hunt (`ai.rs`) |
| Sight | 12 m, or 17 m when your lamp is on (the Watcher excepted); line-of-sight checks up to 26 m |
| Hearing | Straight-line `d < audible_range`: crouch 3.5 m, walk 6.5 m, sprint 13 m, +3 m in pool water. Hounds hear 1.8×; all creatures hear 0.55× during Relax. |
| Losing you | A hunt drops to search after 5 s without sight or sound |
| Notice | On notice: a 0.35–0.6 s freeze, a shriek and lights stuttering; then a 2.2 s charge at 1.2× speed |
| Death | Any contact within `KILL_DIST` 1.05 m kills |
| Speeds vs player | Player: walk 3.6 m/s, sprint 6.4 m/s. **Hound: 7.2 m/s plus up to 0.9 by depth, faster than your sprint.** **Smiler: 5.5 m/s, but 6.6 m/s during the charge and 1.7× (≈9.4 m/s) during a lunge.** |

Gaps against Part 2:
- **Hearing passes through walls.** It should use the flow-field path distance, which the code already computes in `player_field`.
- **Three of the four roamers turn on gaze** (Smiler, Hound, Watcher), so the counters blur together.
- **Hounds cannot be outrun.** The only escape is a stare that holds them for 2.8 s, with a 12 s cooldown.
- **Contact kills at once.** Nothing postpones death.
- **There is no per-run memory**, so nothing for the creatures to "learn".

### 4.2 Shared architecture (data-driven, as in TLOU)

Use one creature brain whose behaviour comes from per-kind data: senses, a prioritised skill list, tells and unlocks. Kind names should not appear in branches; replace `if kind == "smiler"` with data, as Botta recommends.

**Awareness**, per creature, from 0 to 1:

| Stage | Range | Behaviour |
|---|---|---|
| Unaware | < 0.2 | Its senses work at ×0.6 (TLOU's unaware penalty) |
| Suspicious | 0.2–0.6 | It plays its *agitated tell* (stops, tilts its head, snuffles) and goes to look (cautious search) |
| Alerted | ≥ 0.6 | Hunt |

Awareness builds from sight time (a detection timer of 0.6–2.0 s, scaled by distance, the light on the player, and posture) and from sound events. It decays at about 0.1/s.

**Senses**, each a tunable per kind:
- **Sight.** A volume, not a cone: a wide 30–60° near field, a narrow 20° far field to 12–20 m. In it a timer fills, faster if the player is lit.
- **Hearing.** Sound events from the player (steps by speed, sprint, landing, a door, cry-out, the lamp switch, Almond Water, a battery swap). Each has a radius compared against **path distance through the flow field**, minus a penalty per doorway. Off-screen creatures beyond 10 m hear at ×0.5 (Splinter Cell).
- **Light.** The player's lamp is a *sight booster*, not a lure that works through walls. A lamp beam that crosses a creature's volume counts as seeing the player at 1.5× range.
- **Proximity.** Within 1.5 m, a stationary player is sensed after 2 s (TLOU's silent "breathing" sound).

**Memory:**
- **Last known position** plus 2.5 s of "intuition" after losing sight (Welsh). Then a search: last known position, then up to 6 points chosen from hide-spot and tile-visibility data, biased by ≤ 20% toward your true position. Abandon after 15–25 s, and walk back to wander rather than snapping.
- **Per-run habit ledger** (Part 4.4): counts of how often the player hides, sprints, uses the lamp, marks with paper, stares, cries out, and of which hide spots were used.

**Skills, in priority order (TLOU style):**
1. Flee/withdraw (director back-off, or after a catch)
2. Strike
3. Hunt
4. Search
5. Investigate
6. Kind-specific skill (ambush / stalk / sentinel)
7. Wander (always valid)

**Director link.** Keep the current phases and add:
- a **tell budget**: a creature cannot strike unless it has played at least two tell stages that the player could perceive in the last 20 s;
- a **first-meeting flag** per kind per run: the first strike from a kind never kills (Part 4.5);
- **Relax lasting 30–45 s scaled by depth** (L4D), with senses at ×0.55 as now.

Mix the phases so that arousal swings often but in small amounts (Part 3.1). Several Build stages should end in nothing: a sign, then quiet. Those false alarms keep each tell uncertain (Part 3.2).

### 4.3 Each creature as a separate puzzle

Each roamer gets a **different main sense and a different counter verb**, so players build separate mental models.

**Smiler: light-hunter and ambusher**
- **Role.** Ambusher in dark zones, drawn to light.
- **Main sense.** Light: it sees your *lamp* from 25 m along any line of sight, and your body at only 8 m. It hears normally.
- **Tells, in order:**
  1. Eyes and grin catch your lamp at the edge of the beam. The eyeshine is a reflection, not a glow.
  2. A wet click as it shifts from cover to cover; you only ever glimpse it, as with TLOU's Stalkers.
  3. Lit panels near it dim.
  4. When you turn your back, a short inhale 0.8 s before the lunge.
- **Rule.** It approaches while you are lit, and lunges only if you turn away within 6 m. The lamp does *not* drive it off (the TLOU trap). **Darkness hides you**: lamp off, crouched and still, it loses you in about 3 s.
- **Counters.** Lamp off, then sidestep; back away facing it. It slows to 3.0 m/s in lit *places*.
- **Speeds.** Approach ≤ 4.5 m/s; lunge ≤ 7 m/s over ≤ 4 m.
- **Unlock.** After 4 "lamp off to hide" uses, it *waits* 6 s where you went dark. Tell: the clicking stops.
- **Upper face.** A fixed grin that never flinches at a cry-out (Tinwell 2013).

**Hound: blind, hears through the floor**
- **Role.** Hunter by hearing. It replaces the stare rule with sound.
- **Main sense.** Hearing at 2× the radius, through the path graph. It has very little sight; the wet hair over its face supports that.
- **Tells:**
  1. A distant scrabble on the carpet, then a pause.
  2. A sniffing "listen" pose: the agitated state, 1.5 s, in which it holds still. If you stop moving now, it does not lock on.
  3. A huffing breath.
  4. The charge, with a howl.
- **Counters.** Stand still; crouch-walk (its hearing scales with your speed); throw an object. It goes to the sound, not the thrower (TLOU), so props become distraction tools.
- **Stare.** A weaker version stays: in your beam within 5 m, it hesitates 1 s.
- **Speeds.** Search at 3.0 m/s; charge at 6.0 m/s (*below* your 6.4 sprint) for ≤ 6 s, then a 3 s re-acquire pause. You escape by running *and then going quiet*.
- **Unlock.** After 3 sprint escapes, a second hound answers the howl from the opposite side (TLOU's follow skill). Tell: a howl answered from far off.

**Watcher: gaze, mapped to its own place**
- **Role.** Sentinel and stalker. It moves only when unseen, and it is not a killer.
- **Main sense.** None needed. Its rule is your gaze. Define "looking" exactly as Bracken does: a central screen region (dot > 0.9), within 26 m, with line of sight.
- **Tells:**
  1. Furniture is in a slightly different place each time you look back.
  2. Dust sheets settle.
  3. The hum dips as it nears.
  4. Its breath behind you once it is within 6 m.
- **Rule.** It closes in only while unseen. At contact you lose 45 sanity and are knocked down, as now, but **keep the current teleport only for the director's "sent" case**. Never teleport within 10 m of the player.
- **Counter.** Walk backwards, or leave a light source facing it. Paper marks also count as "a watched place" for 10 s; this is a lore hook for M.'s paper trail.
- **Unlock.** After 5 long stares (over 8 s), it stops while watched but turns its head toward the exit you need. A tell that is also a hint.

**The Building: environment, not creature (Levels 1–3)**
- **The samosbor** is an environmental hazard scheduled by the director, never placed at random:
  - **Calm:** fixtures hum.
  - **Build:** a far siren, radio static in cells, warning chalk on doors.
  - **Peak:** violet fog rising from below at a readable rate, for example 0.3 m/s up the stair hall.
  - **Relax:** a cleaners' log entry, and red marks on doors that did not open.
- **Counter.** Reach a hermetic cell and seal it. The first samosbor on a run starts with its siren at least 90 s before the fog reaches the player's floor.
- **The knocker.** Something behind a resident's door knocks back. Answering it raises menace only: a rule to learn, not a threat.
- **The grey figure across the void** (after SCP-3333, CC BY-SA) turns away when looked at and is never hostile.

**Anomalies.** Each is one rule in one place, so the space itself teaches:
- **SCP-087 face.** Its distance shrinks per landing descended, never while climbing. The tell is the light dimming on each landing.
- **SCP-3008 staff by night.** Night is announced by the lights going off overhead. Staff are sentinels on fixed routes and are slow (2.5 m/s). Shelves block their sight.
- **SCP-1983 shadows.** Deadly only while you are lit. The tell is your own shadow appearing doubled.
- **SCP-970 seated man.** He moves only while unseen, so he shares the Watcher's rule. To keep them distinct, he takes one discrete step per closed door behind you.

All four need SCP attribution (CC BY-SA 3.0).

### 4.4 Learning the player can see (unlocks)

- Keep a **per-run habit ledger**. Each kind has 1–2 unlocks with thresholds. At most **one unlock per level**, never from deaths (Alien), and a backstop at a fixed depth.
- **Announce each unlock** once, inside the world. M.'s notebook can carry it: *"it checks under the beds now"*. This is a hint, not an explanation (pillar 4). The first time an unlocked behaviour happens, it plays in plain view.
- Example: after hiding under a bed 3 times, creatures in search check hide spots. Tell: they kneel and sniff at the bed edge for 2 s before reaching in, which gives the player a window to slip out the other side.

### 4.5 Fairness rules as invariants in code

1. **Tell budget.** No strike unless the striking creature's log has ≥ 2 perceivable tells in the past 20 s. "Perceivable" means within the sound radius or in view.
2. **First meeting.** The first strike from each kind per run never kills: a knockdown (3 s), a sanity hit, then the creature withdraws for one full Relax.
3. **Postponed death.** Track hits per creature per Peak. Death comes on the second contact in the same Peak, or on the first contact once sanity is below 25.
4. **Speed cap.** Sustained chase speed is ≤ 0.95 × player sprint. Bursts are ≤ 7 m/s for ≤ 4 m. Stamina must allow at least 8 s of sprint.
5. **Grace periods.** No death in a level's first 30 s, and none within 2 s of a creature's first notice (Phasmophobia).
6. **No-search and no-spawn zones** near the level entrance and hermetic cells (Alien).
7. **Cost of death.** Respawn at the level entrance or the last hermetic cell. Keep paper marks and the map. Lose half the remaining battery and one Almond Water. Re-roll creature start positions with a *death-seeded* random stream, kept separate from the level build to respect the determinism rules.

### 4.6 Test plan: proving the game stays beatable

Extend the existing persona harness (`crates/backrooms/tests/personas.rs`, which already has an Explorer, a Rusher and a Hider, with oracles for "never stuck", "rusher reaches exit" and "no early death").

**New personas:**

| Persona | Behaviour | What it proves |
|---|---|---|
| **Rule-follower** | Knows each kind's counter: lamp off for Smilers, still for Hounds, backwards for the Watcher, cell for the samosbor | The counters work; upper bound on survival |
| **Naive** | Explorer that ignores all rules | Deaths are fair (preceded by tells) |
| **Sprinter** | Always runs | The Hound charge cap; that speed alone is not a solution |
| **Lamp-on** | Never switches the light off | The Smiler is not a forced death |
| **Habit-abuser** | Hides in the same spot every time | Unlocks trigger, are announced, and are survivable |
| **Statue** | Stops and waits whenever a tell plays | Waiting is not a degenerate win: the director's final push and searches still pressure it |

**Arena tests.** For each kind, a fixed 1-v-1 arena, run 200 seeds with the counter and 200 without:
- Counter survival ≥ 95%. Survival without the counter is 30–70%, so the counter matters and the creature is not instant death.
- The first-meeting rule holds in 100% of runs.

**Level runs.** 30 seeds × the creature levels × all personas, 600 s each, with deaths on. Proposed targets:

| Metric | Target |
|---|---|
| Rule-follower completion | ≥ 90% |
| Rusher completion | ≥ 60% |
| Naive completion | ≥ 35% |
| Deaths per completed level (Naive) | ≤ 1.5 |
| Deaths with < 2 prior tells | **0** (hard fail) |
| Deaths during a first meeting | **0** (hard fail) |
| Deaths in the first 30 s | **0** (hard fail) |
| Peak length | 10–40 s, from the director log |
| Relax after each Peak | ≥ 25 s |
| Peaks per 10 minutes | 2–5 |
| Builds ending in nothing | 20–40% |
| Same tell sequence leading to the same outcome | ≤ 70% (otherwise it becomes predictable and habituates) |
| Time to complete (Rule-follower) | Within 1.5× the Rusher's time, so the counters do not cost too much time |
| Death locations | No single 4 m tile holds > 10% of a level's deaths (death heatmap) |

**Readability (with people).** Five-person playtests per creature. After three encounters, ask each player to state the creature's rule in one sentence. The pass mark is ≥ 4 of 5 correct (Grip's mental-model test). Optionally log heart-rate swings to check that arousal moves in small, frequent swings rather than staying pinned high (Part 3.1).

**Regression.** Run the arena tests and the hard-fail oracles in CI on every change to `entities/`. Run the full persona sweep nightly, and diff the report against the last accepted one.
