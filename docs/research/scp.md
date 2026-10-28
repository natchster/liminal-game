# SCP rule-monsters, and how SCP games make them playable

Research for the entity redesign of "The Backrooms" (see [the brief](brief.md)). Part 1 surveys SCP Foundation entities whose horror is a rule. Part 2 shows how SCP games (mainly *SCP – Containment Breach*, whose source is public) turn those rules into mechanics, with numbers from the code. Part 3 lists concepts we could adapt, with attribution, for the Building (Soviet megastructure, samosbor) and the Backrooms.

## Key takeaways

- The SCP monsters people remember are **one-sentence rules**: it moves when unseen (173), seeing its face kills you (096), it speaks in your friend's voice (939), do not recognise the bodies (2316), do not write about it (2521). Each rule turns a normal act (blinking, looking, talking, resting) into a risk.
- *Containment Breach* (CB) is forgiving in its code. 173 freezes if it is anywhere in your view cone, even behind a wall. 096 only triggers if you stand in front of it, its head is on screen, you have line of sight, and you are not mid-blink. 106 always signals with a sound and a spreading floor stain before it rises, and gives up after about 45–50 s out of sight. The horror comes from the rule. The checks themselves are lenient.
- CB players praise 173, 106 and 939. They call 966 and 049 annoying (unclear threat, slow attrition). In SCP: Secret Laboratory (SL), the most-criticised part is the pure-luck exit from the pocket dimension ([Steam summary](https://vaporlens.app/app/2178380/scp_containment_breach.md), [SL Steam thread](https://steamcommunity.com/app/700330/discussions/10/1743352529782907642)).
- Best fits for empty rooms and a Soviet block: 1499 (a **Soviet GP-5 gas mask** that shows another place), 939 (voices behind doors), 106 (lured by a cry, which matches our "cry out" verb, over the block's wired radio), 096 (a resident you must not see face-on), 860-2 (stay on the painted walkway), 513 (a bell that makes something appear at the edges), 049 (a "doctor" hunting the fog-exposed).

## 0. Licence ground rules

- SCP Wiki text is CC BY-SA 3.0. Attribution must give the work's name, the author's handle and a link. Derived works must also be CC BY-SA. The guide does not rule on game mechanics and tells developers to seek their own legal advice ([licensing guide](https://scp-wiki.wikidot.com/licensing-guide)).
- **SCP-173's original image is not free.** It was a photo by Keisuke Yamamoto of Izumi Katō's sculpture *Untitled 2004*, used without permission. In 2014 Katō allowed non-commercial use only. The wiki removed the image in February 2022 because of the licence conflict and merchandise sellers. At Moto42's request no replacement was added ([Wikipedia](https://en.wikipedia.org/wiki/SCP-173), [CGMagazine](https://www.cgmagonline.com/news/creepypasta-scp-173s-soon-be-removed)). Never model anything on the peanut-shaped sculpture.
- Images on SCP pages have their own credits. Some are "used with permission" and are **not** CC (e.g. SCP-3199's art by Ryan Van Dongen, one SCP-2521 image). They are noted below.
- CB's code and assets are CC BY-SA 3.0 ([README](https://github.com/Regalis11/scpcb/blob/master/README.md)), except SCP-173 ([Wikipedia](https://en.wikipedia.org/wiki/SCP_%E2%80%93_Containment_Breach)). SL, *SCP: Containment Breach Multiplayer* and the Unity remakes are separate works. Take only general ideas from them.

---

## Part 1 — The entities

Each entry: look · rule · why it frightens · how it is pictured · author · licence. The text of every page is CC BY-SA 3.0 unless noted.

### A. Rules about looking

**SCP-173 — "The Sculpture"** ([page](https://scp-wiki.wikidot.com/scp-173), author Moto42, 2007)
- *Look:* concrete and rebar with traces of Krylon spray paint. The floor of its cell is stained with blood and faeces.
- *Rule:* it cannot move while it is in someone's direct line of sight. Staff go in threes and announce each blink. It kills by snapping the neck or by strangling. Scraping stone is heard from the cell when nobody is inside.
- *Why it frightens:* a reflex you cannot fully control (blinking) becomes the danger. Every look away is a bet. The scraping heard from an empty room is the "something moved while you weren't there" moment.
- *Pictures:* since 2022 there is no official image. Fan redesigns are collected on a "Peanut Gallery" page and range from humanoid to abstract ([Wikipedia](https://en.wikipedia.org/wiki/SCP-173)).
- *Kin:* [SCP-650](https://scp-wiki.wikidot.com/scp-650) (Dr Talson) is a black, faceless, handless statue. When unobserved it moves silently to stand right behind someone in a threatening pose. It never attacks, so the whole threat is the scare when you turn round.

**SCP-096 — "The Shy Guy"** ([page](https://scp-wiki.wikidot.com/scp-096), author Dr Dan)
- *Look:* about 2.38 m tall, malnourished, with no pigment or hair. Its arms are about 1.5 m long and its jaw opens to four times the human range.
- *Rule:* viewing its face directly, on video or in a photo triggers it. Drawings do not. It covers its face, screams and cries, and one to two minutes later runs at the viewer at 35 km/h or more. It always knows where the viewer is, nothing stops it, and it leaves no remains.
- *Why it frightens:* the mistake is tiny and already in the past. The crying phase is a countdown you can hear.
- *Pictures:* the page carries no image of it. As an in-joke, a photograph of its face would itself be a trigger.

**SCP-372 — the peripheral jumper** ([page](https://scp-wiki.wikidot.com/scp-372), author Sylocat; photo by Sylocat, CC BY-SA)
- *Look:* about 2 m long and thin, with eight pairs of limbs that cling to surfaces.
- *Rule:* it reads nearby brains to predict where people will look. It hides in their blind spots and the gaps during eye movements. Staff are told to report any flicker at the edge of vision.
- *Why it frightens:* it targets something everyone has seen, a shape at the edge of vision that is gone when you turn. One patient was made to believe he was hallucinating it.

**SCP-513 — the cowbell** ([page](https://scp-wiki.wikidot.com/scp-513), author beefwit; photo public domain)
- *Rule:* anyone who hears the bell feels anxious and watched. About an hour later they start to glimpse SCP-513-1, a tall, emaciated figure with very large hands. It shows up at sudden changes of view, such as opening a door or turning the head. Only exposed people can see it. Sightings come every 14 to 237 minutes, and victims who fall asleep are attacked.
- *Why it frightens:* a harmless sound starts a curse. You cannot rest, and tiredness makes your own perception unreliable.

**SCP-1471 — "MalO ver1.0.0"** ([page](https://scp-wiki.wikidot.com/scp-1471), author LurkD; image by Tanagryph, CC BY-SA)
- *Look:* a large black-haired humanoid with a canine skull.
- *Rule:* a phone app sends pictures with the figure in them, and it gets closer by stage. First places you visit often, then places you were recently (48 h), then live pictures of you with it nearby (72 h). From about 90 h you see it in reflections and at the edge of vision, for good, unless you stop looking at the pictures before then. It is not reported to be hostile.
- *Why it frightens:* a fixed schedule, and the creature closing in through your own device.

### B. Rules about sound and words

**SCP-939 — "With Many Voices"** ([page](https://scp-wiki.wikidot.com/scp-939), authors Adam Smascher and EchoFourDelta)
- *Look:* pack predators about 2.2 m tall and 250 kg, with translucent red skin and long eyeless heads. Red, faintly glowing fangs up to 6 cm long. Light-sensing spots run along the spine and heat pits surround the jaws.
- *Rule:* they lure prey by imitating human speech, often the voices of earlier victims. They also breathe out an amnestic, AMN-C227, that causes memory gaps and mild hallucination.
- *Why it frightens:* the cry for help is the bait, so being decent becomes the danger. It also turns the creature's victims into its voice.
- *Note:* the page itself does not say they hunt by sound. That rule comes from the games (Part 2).

**SCP-2521 — ●●|●●●●●|●●|●** ([page](https://scp-wiki.wikidot.com/scp-2521), author LurkD, 2015 Short Works Contest; one image is copyrighted, used with permission)
- *Look:* a tall humanoid of black tentacles, about 2.25 m. Each arm ends in four sticky tendrils. It can teleport and pass through matter ([SCP Explained](https://scpexplained.wikidot.com/scp-2521)).
- *Rule:* it notices word-based information about itself (writing or speech) and takes it, along with the person who produced it. It cannot read pictures, so its whole file is drawn in pictograms ([SCP Explained](https://scpexplained.wikidot.com/scp-2521)).
- *Why it frightens:* the act of warning someone summons it. The page's own form, pictograms only, is the containment.

**SCP-4975** ([page](https://scp-wiki.wikidot.com/scp-4975), author Scented_Shadow) is a beaked, bird-like stalker. Its neck turns one vertebra at a time with a rhythmic click, and anyone who hears the clicking must isolate themselves. The sound comes before it is ever seen.

**SCP-106 — "The Old Man"** ([page](https://scp-wiki.wikidot.com/scp-106), author Dr Gears; images by Klay Abele/MrKlay and Cinemamind, CC BY-SA)
- *Look:* an elderly humanoid in advanced decay. It leaves a black, corrosive mucus that rusts, rots or cracks whatever it touches.
- *Rule:* it passes through solid matter by stepping into a "pocket dimension" and can come out anywhere linked to it. It prefers victims aged 10–25, hunts for pleasure and keeps prey alive in its dimension. **Recontainment:** a lure subject is injured and their cries are played over the site's public-address system until it comes to a prepared cell.
- *Why it frightens:* walls are no protection, it is slow and patient, and the only lure is human suffering broadcast through the building.

### C. Rules about light and dark

**SCP-017** ([page](https://scp-wiki.wikidot.com/scp-017), author CityToast) is a child-sized, featureless "shadowy, smoke-like shroud". When a shadow falls on it, it leaps at whatever cast the shadow and wraps it, leaving nothing. It is kept lit from every side. The danger is your own shadow, which depends on where the light is.

**SCP-280** ([page](https://scp-wiki.wikidot.com/scp-280), author Dr Gears; image by S D Locke, CC BY-SA)
- *Look:* a black human-shaped mass with two large white eyes and long thin hands. It has no legs and can turn to smoke.
- *Rule:* it drifts towards humans and tears at them for one to five minutes. In the dark it is mistaken for a shadow or clothing. It backs away slowly from light, turning its eyes towards it. Full light or a bright flash makes it vanish and reappear elsewhere.
- *Why it frightens:* an ordinary dark shape in the room turns out to be it. Light drives it off but does not get rid of it.

**SCP-966** ([page](https://scp-wiki.wikidot.com/scp-966), author Enma Ai; infrared image by stephlynch, CC BY-SA)
- *Look:* thin, hairless predators 1.4–1.6 m tall that walk on their toes. Long faces, needle teeth and claws up to 20 cm, but weak. They are visible mainly in near-infrared (700–900 nm).
- *Rule:* a "wave" stops victims sleeping, within about 20 m. Lead blocks it. They stalk the victim while it weakens and eat it once it collapses.
- *Why it frightens:* an invisible cause of slow decline. You know something is there only from how you feel.

### D. Rules about places

**SCP-860 / 860-2 — the blue key and the forest** ([page](https://scp-wiki.wikidot.com/scp-860))
- *Rule:* the blue key opens any door it fits onto a misty forest clearing with an 80 cm footpath. The door locks behind you. The path normally leads to a second door in an endless concrete wall, which opens into the room you meant to enter. Other events are in Documents 860-I to IV.
- *Note:* I could not confirm details of the creature 860-2 on the main page. It is known mostly from CB (Part 2).
- *Why it frightens:* there is one safe line through a hostile space.

**SCP-2316 — "Field Trip"** ([page](https://scp-wiki.wikidot.com/scp-2316), author djkaktus; images by djkaktus, CC BY-SA, derived from CC0)
- *Rule:* corpses float in a lake (about 45, perhaps 200 or more) and behave as one entity. Viewers come to believe the bodies are people they knew, usually from childhood. Touching one, or entering the water, brings more and pulls people in. The page repeats to the reader that they do not recognise the bodies.
- *Why it frightens:* familiar faces made wrong, and a defence you must keep repeating to yourself. The page addresses the reader directly, so the reader feels exposed too.

**SCP-1499 — the Soviet gas mask** ([page](https://scp-wiki.wikidot.com/scp-1499), author Trasknari; image CC BY-SA)
- *Rule:* putting on this **Soviet GP-5 gas mask** moves the wearer to a barren place of dark stone, tall black towers, hollow black cubes and large complexes. Taking it off brings them back to the same spot. It is home to SCP-1499-1: tall, dark humanoids covered in a thick coating, with many eyes and mouths. They are violent among themselves, and observers hid from them and followed them unseen.
- *Why it frightens:* protective gear is also a door. The other world matches the Soviet-block mood, with towers, cubes and complexes.

**SCP-610** ([page](https://scp-wiki.wikidot.com/scp-610), author NekoChris) is a flesh infection near **Lake Baikal, Siberia**. The infected approach anyone they can see (about 30 m is safe from blind ones), then root in place and their growth covers the objects around them. It is a model for the "deeper levels rot" idea.

**SCP-3000** ([page](https://scp-wiki.wikidot.com/scp-3000), authors A Random Day, djkaktus, Joreth) is a giant eel. Being near it erases memory and the sense of self: you lose who you are, not your life.

### E. Others worth knowing

**SCP-049 — "Plague Doctor"** ([page](https://scp-wiki.wikidot.com/scp-049), Gabriel Jade; rewrite djkaktus and Gabriel Jade; images by Jadeitor and VolgunStrife, CC BY-SA)
- *Look:* about 1.9 m. Its robe is hide grown from its skin and its plague-doctor mask is chitin.
- *Rule:* it sees people with "the Pestilence" as ill. Its touch stops all bodily functions. It then operates on the body, creating reanimated SCP-049-2. It is formal and polite but gets agitated when criticised. Lavender calms it.
- *Why it frightens:* a polite monster that thinks it is curing you.

The rest, in short:

| Entity | Rule in one line | Author | Fit for us |
|---|---|---|---|
| [SCP-457](https://scp-wiki.wikidot.com/scp-457) | A living fire. It grows smarter with fuel, lays traps, writes by charring, and splits into rivals | agatharights | Factory floors. Fuel management |
| [SCP-4666](https://scp-wiki.wikidot.com/scp-4666) | Very tall, thin, naked old man. For about 12 midwinter nights he watches the house and makes noises in the attic, then kills or takes a child. Cases include Russia and Finland | Hercules Rockefeller | Winter dread. Noises above the ceiling |
| [SCP-3199](https://scp-wiki.wikidot.com/scp-3199) | Featherless egg-layers, 2.9 m, with necks that turn 340°. They kill to lay eggs that fill empty space | bittermixin (image copyrighted, by permission) | Too gory for us |
| [SCP-055](https://scp-wiki.wikidot.com/scp-055) | Anti-meme. You can only remember what it is *not* | qntm, CptBellman | Note-taking mechanic |
| [SCP-5031](https://scp-wiki.wikidot.com/scp-5031) | A "murder monster" seen only by its shadow, rehabilitated through food, music and play | PeppersGhost | A rare kindness beat |
| [SCP-087](https://scp-wiki.wikidot.com/scp-087) | An endless stairwell. Light over 75 W fails, there are cries below, and a face (087-1) | Zaeyde | Already used |

I checked SCP-3812, 1000, 4290, 2852, 6789 and 7179 and left them out: they have no learnable rule for a lone player. One exception: [SCP-6789](https://scp-wiki.wikidot.com/scp-6789) is a calm, pulsing, growing room in a Polish factory, which could be a deep-level motif.

### Why the rules work (psychological patterns)

1. **A normal act becomes a risk:** blinking, looking, talking, resting, helping. The player cannot avoid the act, so the tension lasts.
2. **Off-screen change:** 173 and 650 move only when unwatched. The scare is finding the room has changed.
3. **Familiar things made wrong:** 939's voices and 2316's faces turn trust into bait.
4. **A known countdown:** 096's crying, 1471's schedule, 4666's 12 nights.
5. **The cure is a discipline:** keep eye contact, don't sleep, don't recognise. Survival is a habit the player can learn, which is what makes these good game rules.

---

## Part 2 — The games

### SCP – Containment Breach (Joonas "Regalis" Rikkonen / Undertow Games)

- *Facts:* released 15 April 2012; final version 1.3.11 on 19 August 2018; Blitz3D; procedural map from a seed; three difficulties (Safe, Euclid, Keter, the last with permadeath). Rikkonen made *SCP-087-B* first and focused on atmosphere, sound and jump scares ([Wikipedia](https://en.wikipedia.org/wiki/SCP_%E2%80%93_Containment_Breach)). The source is on [GitHub](https://github.com/Regalis11/scpcb) under CC BY-SA 3.0. A community preservation release came to Steam on 20 February 2026 ([Steam](https://store.steampowered.com/app/2178380/SCP__Containment_Breach/), [Wikipedia](https://en.wikipedia.org/wiki/SCP_%E2%80%93_Containment_Breach)).
- *Units:* the game runs at 70 ticks per second. Player walk speed is 0.018 units/tick (≈1.26 u/s) and sprint is ×2.5 ([Main.bb](https://github.com/Regalis11/scpcb/blob/master/Main.bb)). A room is about 8 units, so 1 unit ≈ 1 m. NPC speeds are in [Data/NPCs.ini](https://github.com/Regalis11/scpcb/blob/master/Data/NPCs.ini).

All mechanics below are read from [NPCs.bb](https://github.com/Regalis11/scpcb/blob/master/NPCs.bb), [Main.bb](https://github.com/Regalis11/scpcb/blob/master/Main.bb) and [UpdateEvents.bb](https://github.com/Regalis11/scpcb/blob/master/UpdateEvents.bb).

**The blink meter.**
- The meter starts at a random 455–665 ticks on Normal (490–700 Easy, 420–630 Hard). It drains at 0.6 per tick, so a forced blink comes every **≈11–16 s**.
- A blink lasts 20 ticks (≈0.29 s), with full black for about 10 ticks.
- The blink key blinks at once, and holding it keeps the eyes shut.
- Eye drops slow the drain. Irritants speed it up. SCP-966 nearby sets the drain rate to at least 1.5×.

The design trick: the player can **choose when to blink**, for example after backing into a doorway. The risk is a planned act, not only a random one.

**SCP-173.**
- *When it freezes:* if its model is **inside the camera view** (`EntityInView`) and the player is not in the black part of a blink. The check uses the view cone only, not line of sight, so a 173 behind a wall but inside your view cone is still frozen. That is lenient.
- *How it moves:* speed 0.38 u/tick, about 21× walking speed, so in practice it appears in a new place.
- *Attack:* it kills only if it has line of sight and is within 0.65 units.
- *Sound:* while moving it loops a **stone-scraping sound whose volume matches the distance it moved**.
- *Feedback when close:* while you hold it in view, the screen blurs and the camera zooms as it gets closer, with horror stings at 3.5 and 1.5 units.
- *When far:* if you are more than ~50 units away it teleports to a waypoint 15–25 units from you. At medium range it wanders. Within about 3 rooms it hunts and can open simple doors, but not keycard or code doors.
- *Teaching:* the death message names the cause ("Fatal cervical fracture…"), so the rule is taught after death.

**SCP-106.**
- *Before an attack:* it waits off-map for a timer of 22,000–27,000 ticks (**≈5.2–6.4 min**). The timer counts twice as fast on aggressive settings and does not count at all in rooms flagged "DisableDecals", which are safe rooms.
- *The tell:* when it attacks, a **corrosion stain grows on the floor under the player**, a decay sound plays, and it rises through the floor.
- *Chase:* it walks slower than the player (0.012 u/tick, about 0.67× walking speed) and passes through walls when it can see you or is far away. Chase music plays.
- *Leaving:* after about 3,000–3,500 ticks (≈45–50 s) without seeing you, it sinks away if it is out of view and more than 5 units off.
- *Ambush:* if you are more than 10 units away and not looking, it can teleport to a spot about 5 units **behind you** (picked by casting backwards from the player), with a cooldown of 10 s ÷ (difficulty + 1).
- *Capture:* its touch sends you to the pocket dimension, where you take slow damage. The first room "breathes" (the geometry scales) while 106 circles you, and after about 65 s it may attack. Later rooms include a "trick room", trenches watched by a flying thing, and a pillar room where 106 jumps between pillars. Exits are partly luck.
- **Femur breaker:** in 106's chamber you turn on the sound-transmission lever and press the button. A lure subject's bone is broken and his screams are broadcast. 106 emerges into the cell, and you must turn the electromagnets on to hold it. This is the article's lure procedure made into a lever puzzle.

**SCP-096.**
- *Trigger:* it is checked only within 8 units. You must be in **its front half-circle** (where its face points), its head must project **onto your screen**, you must have **line of sight**, and you must **not be mid-blink**.
- *Countdown:* it then cries for **18 s** (70×18 ticks) plus a get-up animation, so there is a known window before the chase.
- *Chase:* it sprints at 0.09 u/tick and kills on contact.
- *Counterplay:* approach from behind. Its idle crying music plays near it, so you know it is close.

**SCP-939** (in a dark storage room).
- *Patrol and lure:* the creatures walk waypoints and now and then play one of ten **lure lines** in human voices (three voice sets).
- *Hearing:* footsteps set a noise value of **4.0 sprinting, 2.5 walking, 1.9 crouch-walking**, which then fades. A 939 attacks if noise × 1.2 > distance (sprint ≈ 4.8 u, walk ≈ 3 u, crouch ≈ 2.3 u). It becomes alert and turns towards you if noise × 1.6 > distance. It always attacks if you are within 1.5 u. Line of sight within 4 u counts as one unit closer.
- *Attack:* it lunges at your last known spot and claws, and several hits kill.
- *What the rule teaches:* slow down and crouch. Hearing a voice means one is near.

**SCP-049.** It walks slightly slower than you (0.015 u/tick), path-finds, **opens ordinary doors and closes them behind itself**, talks while it spots, searches and catches you, and kills on touch. You escape through keycard or code doors, which it cannot use.

**SCP-513-1 and SCP-372.**
- *513-1:* ringing the bell item spawns it. It appears, twitching, in rooms 3–9 units away. When seen it **rings the bell** and moves to waypoints further from you, or vanishes. It does no damage.
- *372:* it appears only **during a blink**, 1.5–2 units away in front of you, and darts sideways out of view with a rustle. It is harmless and there to make the player doubt their eyes.

**SCP-966.**
- *Visibility:* it is invisible unless you wear night-vision goggles. Without them you get messages such as "You feel something breathing right next to you."
- *Effect:* within 16 units it drains stamina twice as fast and raises the blink rate.
- *Trigger:* if your **stamina stays below 10 for 5 s**, every 966 turns aggressive.
- *Counterplay:* do not sprint until you are exhausted near the breathing.

**SCP-860-2** (the forest).
- *Appearing:* it spawns between you and an empty, off-path forest cell, **never in view**. It glimpses you from the trees or steps onto the path and walks towards you.
- *Escalation:* running makes it scream. Within 4.5 u, or after a while, it charges at 0.064 u/tick, which is faster than your sprint, and kills.
- *Leaving:* it disappears if you get more than 15–20 u away while it is still at walking pace.
- *Lesson:* do not linger, and do not let it get close.

**SCP-1499-1.** In the mask dimension, half of the citizens notice you within 10 u (with line of sight) and scream. **Within 5 u, the scream turns every citizen hostile.** Taking the mask off is the escape.

**How CB teaches its rules:** about 25 in-world SCP documents ([Items.bb](https://github.com/Regalis11/scpcb/blob/master/Items.bb)), sound tells (scraping, decay, lure voices, the bell, breathing messages), and death messages that name the cause.

Writing on the game explains the core: CB's horror comes from **removing control**. Blinking, restricted looking, few verbs, no weapons. The author argues that extra actions and puzzles would drain the dread ([Fistful of Wits](https://fistfulofwits.com/2013/04/23/design-elements-in-horror-scp-containment-breach/)). Reviewers singled out 173 and the blink meter, which "keeps you from relaxing" ([GameSkinny](https://www.gameskinny.com/reviews/cute-little-things-scp-containment-breach-review/)).

**What players found frightening or annoying.** A summary of Steam reviews ([VaporLens](https://vaporlens.app/app/2178380/scp_containment_breach.md); it is AI-generated, so treat it as a rough signal):
- *Frightening:* 106 and 939. 173's blink-stalking builds tension.
- *Annoying:* 966, 049 (and 969), the confusing blink system, Keter mode as unfair, getting lost or stuck, SCPs spawning inside walls.
- *Reading of this:* players accept sudden death from a rule they understand (173, 096). They resent attrition they cannot read (966 without goggles, 049's slow pursuit through doors) and outcomes decided by luck.

### SCP: Secret Laboratory (Northwood Studios, 2017, multiplayer; copyrighted game, motifs only)

- *SCP-173:* frozen while humans watch it. "Breakneck Speeds" makes it 1.7× faster with 1.8× longer blinks, but **it cannot kill while the ability is active**. Tantrum leaves a slowing filth ([beta wiki](https://betawiki.scpslgame.com/index.php/SCP-173)). A player recalls a developer saying they would not add a blink meter, to keep 173 unpredictable ([search summary, Steam](https://steamcommunity.com/app/700330/discussions/0/3112522283876535332)).
- *SCP-096:* looking at it **or shooting it** makes you a target, and targets hear an audio cue. It moves slowly for 6 s, then rages for 15 s plus 3 s per extra target (at most 30 s). It sees targets through walls and breaks most doors, but not blast or checkpoint doors (official wiki via search snippet, [fandom mirror](https://scp-secret-laboratory-official.fandom.com/wiki/File:096_legacy_forward.png); numbers change between versions).
- *SCP-106:*
  - **Stalk** lets it sink and travel fast, but it can only see nearby humans (or those already "Traumatized").
  - **Hunter's Atlas** moves it to the centre of a chosen room.
  - A rebalance **removed the one-hit capture** (now two hits, 1.5 s apart) and added **constant breathing audible up to 7 m** ([patch notes](https://steamdb.info/patchnotes/11526522), [later patch](https://steamdb.info/patchnotes/14642027)).
  - The pocket dimension has 8 corridors. 6 kill and 2 lead back, and players call the choice a guess with "luck rituals" ([Steam thread](https://steamcommunity.com/app/700330/discussions/10/1743352529782907642)).
- *SCP-939:*
  - **Very limited sight.** Nearby sounds appear to it as blips.
  - Lunge attack.
  - An **amnestic cloud** that hides 939 from humans inside it.
  - **Mimicry** replays set sounds or **the last few seconds of a killed or marked human's microphone**, and can project them from a placed point ([official wiki](https://en.scpslgame.com/index.php?title=939)). Later patches cut the mimicry cooldown and let 939 trim clips ([13.0 notes](https://wiki.biligame.com/scpsl/Updates/13.0.0)).
- *Lesson:* over time SL added **audible tells** (106's breathing, 096's target cue) and removed one-hit kills where players had no way to avoid them.

### Other SCP games

- **SCP-087-B** (Regalis, 2012): Rikkonen's stepping stone to CB ([Wikipedia](https://en.wikipedia.org/wiki/SCP_%E2%80%93_Containment_Breach)). Endless dark corridors and stairs, built from a few room types chosen at random, with occasional monsters that chase ([search summary of TV Tropes](https://tvtropes.org/pmwiki/pmwiki.php/VideoGame/SCP087); [itch re-upload](https://word-9.itch.io/scp-087-b)). The randomness gives every floor the same uncertainty. Sources disagree on which monsters it has.
- **SCPCB: Unity** (Wreckless Interactive): a free Unity remake of CB ([Game*Spark](https://www.gamespark.jp/article/2022/09/08/122139.html)). A separate *SCP: Unity* (Gearwork Games) advertises SCPs that use line of sight, hearing and memory-based pathing ([devlog](https://gearwork-games.itch.io/scp-unity/devlog/930545/massive-update-new-scps-and-bug-fixes)).
- **SCP: Containment Breach Multiplayer** (Fusion Creators Studio, 2021): co-op CB with a mode where players take SCP roles ([Steam](https://store.steampowered.com/app/1782380/)). Multiplayer drops the solitude that makes our game work.

---

## Part 3 — Implications for our game

General lessons from the code and the community:

1. **Lenient checks, strict rule.** Freeze "moves when unseen" creatures whenever they are in the view cone. For "don't see its face", require: in front of it, face on screen, line of sight, eyes open. A player who dies should think "I looked", never "the game cheated".
2. **Every rule has a tell before it bites:** scraping (173), stain and decay sound (106), voices (939), the bell (513), breathing (966, SL 106).
3. **A countdown you can hear** (096's 18 s of crying) makes a mistake survivable.
4. **No luck exits.** SL's 2-of-8 pocket corridors draw the most complaints. Make capture spaces escapable by a learnable sign.
5. **Teach after death.** Like CB's death messages, but one line in the cleaners' log or M.'s hand. It hints, never instructs.
6. **Few verbs, each loaded.** "Cry out", lamp, crouch, paper and hermetic doors should each counter some rule.

Reference numbers to tune from (CB, 1 u ≈ 1 m): blink every 11–16 s; noise radii sprint 4 / walk 2.5 / crouch 1.9 (attack at 1.2× noise, alert at 1.6× noise); 106 rest 5–6 min and gives up after about 45 s unseen; 096 crying 18 s; 966 triggers after 5 s below 10 stamina; 1499 citizens alert at 10 u and the crowd turns at 5 u.

### Adaptable concepts (each needs a credit line: entity name, author, link, CC BY-SA 3.0)

**1. The Watcher (existing; after SCP-173 by Moto42, and CB).**
- *Rule:* moves only when unseen.
- *Changes:*
  - Freeze it whenever it is inside the view cone, even behind a wall.
  - Loop a dust-sheet drag sound that scales with how far it moved.
  - Use the **lamp flicker** as our blink. A low battery makes the lamp stutter, and each stutter is a chance for it to move. A "close eyes / cover lamp" input would give the player a chosen blink.
  - When far away, move it to a hidden spot about 15–25 m from the player, never in view.
- *Counterplay:* back away while watching it, put doors between you, change batteries before you enter its rooms.
- *Don't:* show the Katō sculpture in any form. The dust sheet already avoids that.

**2. The Turned-Away Resident (after SCP-096 by Dr Dan).**
- *Place:* the Building. A tall, thin resident sits in an open cell **facing the wall**, weeping. "Residents gone behind their doors" made visible.
- *Rule:* if you see its face (you are in its front half, its face is on screen, line of sight, lamp on), it covers its face and wails for about 15–20 s. Then it runs to you through the galleries.
- *Tells:* weeping heard from a distance. Wet marks on the cell floor.
- *Counterplay:* pass it from behind. Keep the lamp low (point it down). During the wail, reach a **hermetic cell and spin the wheel shut**. For fairness, unlike in the source, it cannot open a sealed hermetic door. It waits outside, then leaves.
- *Fairness:* never place it facing a doorway you must pass through. Always offer a way round behind it.

**3. The Voices Behind the Doors (after SCP-939 by Adam Smascher and EchoFourDelta; hearing numbers from CB).**
- *Place:* the Building's canon line "sometimes something knocks back", and the Backrooms' blackout zones.
- *Rule:* it is blind and hunts by sound. It lures with voices: a resident asking for help, or **M.'s voice reading a line from her notebook with one word wrong**.
- *Tells:* the voice repeats from a small set and never answers what you do. Voices come from behind doors that the paper trail shows are empty.
- *Counterplay:* crouch-walk (an attack radius of about 2.3 m against about 4.8 m when running). Use **"cry out" as a decoy** thrown towards a far wall or door. Never answer a voice.
- *Fairness:* the alert pose and turn come before the attack (at 1.6× the noise radius), so the player gets a warning.

**4. The Old Tenant (after SCP-106 by Dr Gears, and CB's femur breaker).**
- *Place:* deep Backrooms levels that "rot", or the lower Building.
- *Rule:* after a long rest (5–6 min, set by the pacing director), a **black rust stain spreads on the floor under or near the player** with a wet crackle. A slow, decayed figure rises and walks through walls at about 0.7× walking speed. If it does not see you for about 45 s, it sinks away. Certain rooms (the hermetic cells, M.'s camps) never get the stain.
- *Lure:* in the Building, every flat has a **wired radio point (радиоточка)**, so a cry played over the block's broadcast network draws it to a chosen cell. Seal the door to trap it. This is a Soviet version of the femur breaker, using our "cry out" verb at a microphone. That is an inference for our design, not something from the source.
- *Capture:* it takes you to a "below" with a **learnable exit** (for example, follow the one corridor where the hum continues). Never use random corridors.

**5. The Sanitar (after SCP-049 by Gabriel Jade, djkaktus; plague-doctor costume is historical, public domain).**
- *Place:* after a samosbor. A figure in a rubber protective suit and gas mask makes rounds, sure that the **fog-exposed are sick**.
- *Rule:* if you carry fog residue (from time spent in the violet fog), it pursues you and opens ordinary doors. It ignores you if you have washed (a shower or tap in a cell) or wear a sealed mask. Its formal, calm voice gives it away.
- *Canon:* this can stay true to the canon "Nothing walks there… cleaners known only by their log" if it is **heard only**: boots on the gallery, the knock, the polite questions through the door. Opening the door with residue on you is death. Red marks appear on the doors of those who didn't answer.
- *Fairness:* the residue shows on the player's hands and lamp lens, and washing is always within reach of the shelters.

**6. Other-Building Sight (after SCP-1499 by Trasknari).**
- *Rule:* the **GP-5 gas mask**, which we need during samosbor, sometimes shows the *other* Building when worn outside the fog. It has black towers, hollow cubes and tall many-eyed figures. Half of them are "watchful": they notice you within about 10 m and scream within about 5 m, and then the crowd turns. Taking the mask off brings you back where you stood.
- *Use:* some routes or marks are visible only in the mask, so it becomes a tool that carries risk. The grey figure across the void (our SCP-3333 nod) could be one of them.

**7. The Painted Line (after SCP-860 and CB's 860-2 forest monster).**
- *Place:* dead factory floors in fog, with painted safety walkways (yellow lines).
- *Rule:* step off the line and something among the machines appears, always just outside your view. Running makes it scream and charge faster than you can sprint, so it cannot be outrun. Walking back onto the line makes it lose interest.
- *Tells:* footsteps in the machinery, which get louder the longer you are off the line.
- *Fairness:* the line is always visible with the lamp. Gaps in the paint are short and marked by M.'s folded paper.

**8. The Doorbell (after SCP-513 by beefwit; SCP-372 by Sylocat; CB's harmless versions).**
- *Rule:* on one floor, a flat's doorbell rings by itself. After you hear it, a **tall, thin figure with huge hands** appears at view changes: door openings, quick turns, the lamp coming back on. In CB it is harmless while you are awake. In our game it attacks only if you **rest** (heal, sleep at a camp) before leaving that floor or before it fades (one pacing cycle).
- *Counterplay:* don't rest near a bell you have heard. Leave the floor.
- *Effect:* drains nerve, not health.

**9. The Faces in the Water (after SCP-2316 by djkaktus).**
- *Place:* poolrooms or flooded lower galleries. Bodies float face down. Looking at them for long turns them face up, and the faces start to look like M.
- *Rule:* staring drains nerve and slowly pulls the camera and steps towards the water. Turning away, or repeating M.'s written phrase (one press of "cry out"), breaks the pull.
- *Text:* don't copy the article's sentence. Write our own line in M.'s hand.
- *Fairness:* slow onset, clear audio and visual distortion, and it always breaks if you leave.

Optional or lore-only:
- **2521** (LurkD): M.'s notebook draws one thing only as dots and bars and never names it. The rule (it comes to written words) suits a note-free hidden area where using paper marks is the mistake.
- **4975** (Scented_Shadow): clicking in a stair hall means you go into a cell now.
- **966** (Enma Ai): something unseen that only drains stamina recovery, and attacks if you exhaust yourself (below 10% for 5 s) while you can hear breathing. This could replace the Hounds' "hear footsteps" with a clearer rule in the habitable zone.

### What to credit where we adapt

Put in the credits: SCP-173 "Moto42"; SCP-096 "Dr Dan"; SCP-106 "Dr Gears"; SCP-049 "Gabriel Jade", "djkaktus"; SCP-939 "Adam Smascher", "EchoFourDelta"; SCP-513 "beefwit"; SCP-372 "Sylocat"; SCP-860 (author to confirm on the page's history); SCP-1499 "Trasknari"; SCP-2316 "djkaktus"; SCP-2521 "LurkD"; SCP-966 "Enma Ai"; SCP-4975 "Scented_Shadow". Each needs a link and "CC BY-SA 3.0", as the [licensing guide](https://scp-wiki.wikidot.com/licensing-guide) requires. If we port CB logic or numbers, credit "SCP – Containment Breach by Joonas Rikkonen (Regalis) and contributors, CC BY-SA 3.0". Use no SCP images (some are not CC). Write all text ourselves.
