# Research: uncanny vending machines (SCP, the Backrooms, folklore, games)

Research note, 2026-10-29. Brief: rebuild the game's vending machine as a photorealistic prop with its own learnable rules. It is not a creature and never kills by itself. It has to stay compatible with canon lore entry 15, "The Vending Machine": *"Insert coin, get nothing. Insert nothing, get coin. Do not attempt the reverse. It costs more than you have."* (`crates/game/src/ui/hud.rs`, `LORE_ENTRIES`).

**Method and limits.** Every claim links to a page I opened. I read Fandom wikis through their MediaWiki API (plain fetches are blocked) and got SCP authors from the Crom metadata API (`api.crom.avn.sh/graphql`). Not done:
- Reddit was unreachable, so I have no r/nosleep or r/creepypasta story.
- I found no analog-horror series about a vending machine.
- Persona, Inscryption, Pony Island, Platform 8, Noclipped, The Complex: Found Footage, Backrooms: Survival, The Backrooms 1998: nothing found, left out. Prey (2017): not verified.

**Licences.** SCP Wiki and Backrooms Wikidot pages are CC BY-SA 3.0, as stated on each page. Fandom wikis report "CC-BY-SA" through their API ([licensing](https://www.fandom.com/licensing)) without naming a version. The games themselves are proprietary. Japanese kaidan sites are all rights reserved, so for those I describe motifs only.

---

## 1. SCP Foundation

| Object | Author | Rule | What sticks |
|---|---|---|---|
| [SCP-261 "Pan-dimensional Vending"](https://scp-wiki.wikidot.com/scp-261) | Dr Gears | A black machine found in a Yokohama alley. Insert yen and enter any 3-digit number, and it vends a random snack (the number does nothing). Heavy or rapid use makes it "unstable" and it vends bizarre items. Containment caps use at 10 per day and ¥500 per transaction. | The test log: "Darkside Cola" turns black in air; "Lemon Clams" vent to steam and burn a researcher; a box of unknown animals makes distress sounds while cooking; a can detonates and kills two researchers, after which the room "smelled of citrus for days". |
| [SCP-294 "The Coffee Machine"](https://scp-wiki.wikidot.com/scp-294) | Arcibi (cover image: StaticFactory) | Put in 50¢, type the name of any liquid, and it fills a 12-oz cup. It reads the user's mind on vague requests. Unreachable requests return "OUT OF RANGE". | "A cup of Joe" is blood and tissue from the agent who asked, who collapses. "The perfect drink" is followed by suicide; the note reads "everything's just one big letdown". "Surprise me" gets boiling water. |
| SCP-1162 (original, "hole in a wall") | **Not verified.** The original was deleted. The [current SCP-1162](https://scp-wiki.wikidot.com/scp-1162) is an unrelated Mars-house article by Rounderhouse, created 2021-05-29 (Crom). A [Steam thread](https://steamcommunity.com/app/700330/discussions/10/3117034127822156257) says the original author deleted their own work after a ban (hearsay). | Known only through the games. In SCP – Containment Breach ([wikiwiki.jp](https://wikiwiki.jp/scpcbjp/SCP-1162)) you reach into a hole in cinderblock and pull out an item tied to your past, and it removes one item you carry. With an empty inventory it takes your right lung and you bleed out. SCP: Secret Laboratory copies this (same Steam thread). | The exchange is symmetric, and it punishes you for having nothing. |
| [SCP-8733](https://scp-wiki.wikidot.com/scp-8733) | mothmanUXO (Crom) | A real model (Automatic Products 113) that dispenses one human body a week. Every identified victim was killed by a vending machine and comes out holding a vending product. | The ending: "It's your fault I'm in here, after all. It's all your fault." |
| [SCP-6593 "Only Cans"](https://scp-wiki.wikidot.com/scp-6593) | PlaguePJP | A self-restocking machine that is sapient and talks when you stand close (tagged adult and sexual). | A counter-example: once the machine talks, it becomes a comedy character. |
| [SCP-3353](https://scp-wiki.wikidot.com/scp-3353) | Zyn | Not a machine: tell an apple a secret and it may become a cake. Some statements, such as a lie, produce no change. | It is "not a fair trade", and not every input pays out. |

**What makes them work.** The transaction stays ordinary (coin, code, cup) and the horror comes only through the output. The logs let the reader infer the rule from evidence, as our pillar 4 does. SCP-261's "push harder and it destabilises" is the clearest risk lever. SCP-294's best entries answer the user, not the request.

## 2. The Backrooms

**Wikidot** ([backrooms-wiki.wikidot.com](https://backrooms-wiki.wikidot.com), CC BY-SA 3.0):
- [Level 4](https://backrooms-wiki.wikidot.com/level-4) (u/M654zy): coolers, vending machines and fountains hold Almond Water. No rules given.
- [Level 6.1 "The Snackrooms"](https://backrooms-wiki.wikidot.com/level-6-1) (anonymous Fandom user, rewritten by Stretchsterz): machines vend "without any payment". Some are "piled on top of each other" to block rooms.
- [Level 58 "Water Wonder"](https://backrooms-wiki.wikidot.com/level-58) (thelastpizzaroll; rewritten by BlockHead42): cracked touchscreen with a working side keypad. Each holds 10 of each item and restocks irregularly, "a few hours to one week". One machine that was opened turned out to be "an empty metal husk" and never restocked again.
- [Level 187 "Nobody's Summer"](https://backrooms-wiki.wikidot.com/level-187) (PinDead): one machine in a dirt clearing with one Oi Ocha inside. "The light of the drink machine was the only thing saving me from total darkness."
- [Level 317 "Neon Shuffle"](https://backrooms-wiki.wikidot.com/level-317) (GhostlyGlyphs): a one-button machine that vends any liquid you picture. A hazardous request gets "a uniquely flavored soda instead". This is SCP-294 with a safety.
- [Entity 36 "Cannibal Cuisine"](https://backrooms-wiki.wikidot.com/entity-36) (Gasaludo): a fake machine that vends free food made from human parts, pushed out by a skeletal hand, with addictive stock and voices of loved ones. Its tells: real machines charge and use coils, and the back carries a tag. "Check the back of any vending machines you find."

**Fandom** ([backrooms.fandom.com](https://backrooms.fandom.com), CC BY-SA):
- [Level 334 "Neverending Vending"](https://backrooms.fandom.com/wiki/Level_334) (MikeScpFan229): accepts any coin-shaped object. Hitting a machine reflects the pain back onto you. Machines refill when unseen and run unplugged. Their glow is the only light on the level, and "most entities are attracted to the light".
- [Level 986](https://backrooms.fandom.com/wiki/Level_986) (Nightfoxy25): floating machines that you pay with "a flat surface, like cloth or bland paper".
- [Level 479](https://backrooms.fandom.com/wiki/Level_479) (Solus): broken glass you can reach through. Inside are fluid, drywall chips and frozen entity meat. The stench can knock you out, and the exposed mechanism electrocutes.
- [Level 4 (Fandom)](https://backrooms.fandom.com/wiki/Level_4) (Jamie/Gamma980): "attempting to enter a vending machine" can send you to Level 197.
- [Level 16](https://backrooms.fandom.com/wiki/Level_16) (Nerdykiddo4844; rewrites by NightmareFreddybear and WindXD) and [Level 98](https://backrooms.fandom.com/wiki/Level_98) (PandaShark71004): pressing "9" then "8" on the break-room machine takes you to Level 98.
- Also: barter machines on [Level 82](https://backrooms.fandom.com/wiki/Level_82) (Moth Jelly in, dead deathmoths out; credits not checked), a spotlit machine selling transport drinks ([Object 53](https://backrooms.fandom.com/wiki/Object_53), Mr. MapMaker), and Almond Water as the coin on [Level 399](https://backrooms.fandom.com/wiki/Level_399) (Lazynickname, CatRBLX, Pexy).

**Backrooms games:**
- *Escape the Backrooms*, Level 4: four machines, each printed with a room object. You count those objects and key in the number to open the door ([wiki](https://escapethebackrooms.fandom.com/wiki/Vending_Machine); [guide](https://itemlevel.net/escape-the-backrooms-level-four-guide/)). The machine is a keypad, not a prop with behaviour.
- *Inside the Backrooms*, Train Station: machines give free supplies by code: 12 for Almond Water, 55 for sanity pills, 30 for moth jelly. Stock is limited ([wiki](https://inside-the-backrooms.fandom.com/wiki/Train_Station)).
- Roblox *The Backstage of Reality*: one press gives a bandage, Almond Water, or a teleport to Level 6.1 ([wiki](https://backrooms-the-backstage-of-reality-roblox.fandom.com/wiki/Vending_Machine)). It is a pure loot roll.

**Lesson.** The fiction makes machines wells (free Almond Water, restocked when unseen, light as lure); the games make them code locks or loot rolls. None makes the machine's behaviour the thing you learn.

## 3. Folklore, creepypasta, real events

- **The restocker** ([AM2時](https://am2ji-shorthorror.com/2018/05/05/%E8%87%AA%E5%8B%95%E8%B2%A9%E5%A3%B2%E6%A9%9F%E3%81%AE%E6%80%96%E3%81%84%E8%A9%B1/); site admin なつまる; © all rights reserved; motif only). A mountain-pass machine sells only one unpopular soda; at dusk a silent line buys it and walks into the forest. Each attempt to remove it was followed by a crash on that road; the boss likens it to a jizo statue holding something back. Beware machines always sold out or never removed.
- **The figure beside the machine** ([オカルティックコネクト](https://umakado.com/archives/1075970620.html); © site; motif only). The fluorescent light hides a grinning old man in its shadow. Locals say the spot holds only a junked machine that has not worked since the Showa era.
- **2ch riddle story** ([kowaihanasi](https://kowaihanasi.ghostmap.jp/hanasi.php?cd=6912); copyright attributed to 2ch). A poisoned can rim with the antidote in the drink: whoever sips slowly dies. A rule you could only learn from someone else's death.
- **Capitol Hill mystery soda machine**, Seattle ([Wikipedia](https://en.wikipedia.org/wiki/Capitol_Hill_mystery_soda_machine)). Nobody knew who stocked it. A "Mystery" button that later took over all six buttons. It vanished in 2018, leaving a note: "Went for a walk."
- **Eiji Ohashi, *Roadside Lights*.** Nine winters photographing lone machines at night, "devoid of people" ([Hyperallergic, Claire Voon, 2017](https://hyperallergic.com/409582/the-eerie-allure-of-japanese-vending-machines/)). On dark commutes, "the light from the machines have led him safely home" ([Feature Shoot, Ellyn Kail, 2017](https://www.featureshoot.com/2017/12/photos-vending-machines-illuminate-japan-night)). This is our reference image: the machine as the only lit, living thing.
- **Creepypasta Wiki** (CC BY-SA). [The Forgotten Vending Machine](https://creepypasta.fandom.com/wiki/The_Forgotten_Vending_Machine) (author unknown, 2011): a powered machine in an abandoned village; pay in 10p coins only for slot "E", and check the colour before you drink. [The All-New Improved Universal Beverage and Snack Machine](https://creepypasta.fandom.com/wiki/The_All-New_Improved_Universal_Beverage_and_Snack_Machine) (Sudobot, 2013): country codes as products ("KNYA, $5"), the phone dies, a cup holds an eye. It overexplains and turns to gore.
- **Real danger.** The US CPSC counted at least 37 deaths and 113 injuries from 1978 to 1995 from people rocking machines for free product. The industry label reads "Never rock or tilt… Vending machine will not dispense free product." ([CPSC, 1995](https://www.cpsc.gov/newsroom/news-releases/1996/cpsc-soda-vending-machine-industry-labeling-campaign-warns-of-deaths-and-injuries)). That is a real-world "do not attempt", and it reads like our canon note.

## 4. Machines in games

- **BioShock** ([Circus of Values](https://bioshock.fandom.com/wiki/Circus_of_Values), [El Ammo Bandito](https://bioshock.fandom.com/wiki/El_Ammo_Bandito), [Rapture machines](https://bioshock.fandom.com/wiki/Vending_Machines_(Rapture)), [Hacking](https://bioshock.fandom.com/wiki/Hacking)). A laughing clown voice (Ken Levine): "Come back when you get some money, buddy!" and "Hey! I've got a family to feed!" One Fort Frolic machine charges 40 times the norm. Hacking lowers prices, but an alarm tile summons security bots. One machine rigged by a Splicer drops a live grenade. *Why it lands:* a cheerful voice in a ruined city, and a hack whose cost you can see. *Fell flat:* BioShock 2 shipped with the voices missing by mistake.
- **Half-Life / HL2** ([vending machine](https://combineoverwiki.net/wiki/Vending_machine), [Breen's Private Reserve](https://combineoverwiki.net/wiki/Dr._Breen%27s_Private_Reserve); CC BY-SA). In HL1 each can gives 1 HP, a machine holds 10, and breaking it spills cans. Coca-Cola declined to license its brand because breakable machines suggested vandalism pays. In HL2 one machine opens with a code and hides the entrance to Kleiner's lab, and a Metrocop's "Pick up that can" teaches the Use key. *Lesson:* players will hit the machine first.
- **Lethal Company: the Company counter** ([wiki](https://lethal-company.fandom.com/wiki/The_Company_Monster)). The best model of a noise-priced exchange. Any noise (the bell, voices) opens the door and drains patience, which depends on a mood rolled each visit (Agitated, Silent Calm, Snoring Giant). Below patience 1 it rumbles a warning; below 0 each noise has a 50% chance of a tentacle attack. Selling junk can also trigger one. *Players:* a solo player died after one ring. Replies say "it's sound in general"; the player wants a nerf ([Steam, Dec 2023](https://steamcommunity.com/app/1966720/discussions/1/4032473436327678421)). It felt unfair because the vent had opened unnoticed. Jonathan Wojcik would "honestly rather never know" what is behind it ([Bogleech](https://bogleech.com/halloween/hall24-lethal)).
- **Control** ([Altered vending machine](https://control.fandom.com/wiki/Vending_Machine_(Altered_Item)), [case file](https://control.fandom.com/wiki/Vending_Machine_Procedures)). Found in an Alberta nursing home, it "will produce whatever they subconsciously desire". The case file's best line: "Attempts to cause the item to create a living [REDACTED] have consistently failed." In play it hides among ordinary FBC machines and flees when struck ("Vending Spree" achievement). *Control Resonant* (2026) repeats this with 35 disguised machines that have a blinking red light and bolt when hit ([AllThings.How](https://allthings.how/control-resonant-every-vending-machine-location-in-manhattan/)). The file chills; the gameplay makes the machine a creature and a joke, which is our line not to cross.
- **Fallout 3** ([wiki](https://fallout.fandom.com/wiki/Nuka-Cola_vending_machine_(Fallout_3))). Machines are containers rolled on first entry, so players save-scum for Quantums. *Lesson:* seed outcomes per machine so reloading cannot reroll them.
- **Resident Evil 4 Merchant** ([Wikipedia](https://en.wikipedia.org/wiki/Merchant_(Resident_Evil))). "What're ya buyin'?" made him iconic. In the remake, his impatience lines when you browse too long divided fans, and a "Patient Merchant" mod removes them. *Lesson:* repeated voice lines wear out.
- **Yume Nikki** ([wiki](https://yumenikki.fandom.com/wiki/Vending_Machine)). The wiki lists no vending-machine "effect". Three machines sell health drinks, but no hazard can hurt Madotsuki, so they have "no practical benefit", and yen comes only from killing NPCs with the Knife. A reward that is useless is unsettling as dream logic but flat as play.
- **Set dressing only:** Mouthwashing's lounge machines (a crew card sits in the slot of the broken one; [Tulpar](https://mouthwashing.fandom.com/wiki/Tulpar), [achievements](https://mouthwashing.fandom.com/wiki/Achievement_Walkthrough)), a candy-bar sidequest in Silent Hill: Downpour ([wiki](https://silenthill.fandom.com/wiki/Candy_Bar)), Fears to Fathom's gas station ([wiki](https://fearstofathom.fandom.com/wiki/Buck%27s_Gas_n_Go)), and RE2's Chocha-Cola ([wiki](https://residentevil.fandom.com/wiki/Chocha-Cola)). None of The Exit 8's 31 anomalies involves a machine ([list](https://gamesfuze.com/guides/the-exit-8-all-anomalies-guide/)).
- **Interdimensional Vending Machine** (Neuroticfly Games and HoruBrain, on Steam since June 2026). It is based on SCP-261: you beg for coins and every purchase is a gamble that may transform you. Rated 4.1/5 from 186 ratings on itch; comments include "a bit repetitive" ([itch.io](https://neuroticfly.itch.io/interdimensional-vending-machine)).

## 5. Principles

**Why an everyday machine turns creepy:**
- **The uncanny.** Freud's *unheimlich* is the antonym of the "homely". Jentsch's version is "intellectual uncertainty" over whether a figure is a person or an automaton ([Wikipedia](https://en.wikipedia.org/wiki/Uncanny)). A machine that seems to choose sits exactly in that doubt.
- **The eerie.** Mark Fisher: "something present where there should be nothing, or… nothing present when there should be something" ([quoted here](https://www.emergentmag.com/exhibitions/something-where-there-should-be-nothing-at-soft-opening)). One powered, humming machine in a dead building is the first case. "Get nothing" is the second.
- **Light as a lure** (Level 334, Level 187, Ohashi). It is the only warm light, so it is both where you want to go and where things gather.
- **It knows you** (SCP-294's cup of Joe, Control's subconscious desire). The fear comes from being answered, not from what comes out.
- **The wrong price, the stuck item, the sold-out slot.** Small economic failures that players read at a glance. The restocker story turns "always sold out" into evidence that something else is buying.

**What players will try:** insert a coin; press buttons with no coin; check the coin return; shine the lamp through the glass; reach into the flap; hit or shake it (HL2, CPSC); read the label and the back (Entity 36). Each needs a response, or the player concludes it is set dressing.

**Design lessons:**
1. **Price everything in currencies the game already has:** noise (Hounds), light (Smilers), paper, nerve, time. The Company counter works because noise is both how you use it and the danger.
2. **Warn before you punish.** The Company rumbles first; players who missed the cue called it unfair. Every penalty needs an audible tell and a beat to react.
3. **Rules, not dice.** An early idea for the machine was a slot table: 40% Almond Water, 25% battery, 25% stuck, 10% speed soda. Random outcomes cannot be learned (pillar 4). Seed any variation per machine (see Fallout's save-scum).
4. **No talking, no jump scares, no gore.** SCP-6593 and the RE4 remake show talk turning into comedy and wearing out. SCP-8733 and the creepypasta eye work on the page; in first person they are one cheap shock.
5. **Habituation.** Thomas Grip: after a first death, later encounters relax players, and gory cutscenes "make you relax" ([DSOGaming, 2014](https://www.dsogaming.com/news/frictional-games-dev-on-alien-isolation-horror-game-suffers-from-two-major-horror-design-issues)). Keep uncanny reveals rare and director-gated, and the mechanical rule stable.
6. **A prop, never a creature.** It never moves or attacks (unlike Control's); danger comes only from what it makes you do.

## 6. Eight concepts

All eight fit the canon note. Two engine notes: the hum is a held sound, so it needs `note()`, not the plucked `tone()`, and its listen test must fail at gain 0 (project memory). Any light the machine gives is a placed light, which fits "lighting is place-based".

**1. Change and Nothing** (the note made literal)
- **Rules:**
  - Press a selection with no coin in: after 2 s one coin rings into the steel return cup. That is a noise event Hounds route toward along walkable paths. One coin per machine per seeded interval.
  - Insert a coin: it is swallowed and nothing is vended. The hum and the room's ambient bed stop for about 45 s, and your footsteps in that room make no noise.
  - The hum returns with a compressor thunk.
- **Learns:** coins are loud to get and buy silence; silence hides you and deafens you.
- **Risk/reward:** noise now, invisibility to Hounds later, at the cost of every audio cue.
- **Build:** art low (coin, cup, slot); code medium (coin item, noise event, a room-anchored mute-and-stealth zone); sound medium.
- **Pillars:** 3, 5, 1.

**2. The Reverse** (reach in)
- **Rules:**
  - One product always hangs at the glass, label out: Almond Water or a battery.
  - Reaching into the flap gets it, and the flap keeps one of each consumable you carry; you hear them drop inside.
  - If you carry nothing, it takes a large drop in nerve. "It costs more than you have."
- **Learns:** empty your pockets first, then pay in nerve.
- **Risk/reward:** a sure item against an inventory toll or a nerve hit.
- **Build:** art medium (flap, arm animation); code low; sound low.
- **Pillars:** 4, 5. Echoes SCP-1162.

**3. Rock It** (the stuck item)
- **Rules:**
  - A product hangs half-dispensed. The side label reads "NEVER ROCK OR TILT. MACHINE WILL NOT DISPENSE FREE PRODUCT."
  - Each shove is a bang (noise event) and tilts the machine further. The item drops on shove *k* (seeded, 3–6).
  - Two shoves past *k* the machine falls. The glass bursts, the whole stock spills, the crash is heard level-wide, and the fallen machine blocks that corridor for good.
- **Learns:** count your shoves; the label and the tilt are the gauge.
- **Risk/reward:** a survival item against noise. Toppling it is a jackpot that rewrites the map, cutting a Hound route or your own.
- **Build:** art medium (tilt pivot, fallen pose, glass); code medium (tilt state, navgrid edit); sound medium.
- **Pillars:** 5, 2.

**4. The Lit One** (the light that leads you home)
- **Rules:**
  - Among the existing dead machines (`place_dead_vending_machines`), one is lit, and its glow counts as a lamp to Smilers.
  - The compressor cycles: about 50 s of hum, a relay click, about 8 s dark and silent, then a thunk and light (seeded period).
  - With your lamp off in its glow, Smilers drift to the machine, not you. When it goes dark, they turn to the next light.
- **Learns:** listen for the click; it is a decoy on a timer.
- **Risk/reward:** a safe spot to change batteries against standing where Smilers gather.
- **Build:** low throughout (emissive panel, a Smiler target, a timer, three sounds).
- **Pillars:** 1, 3, 4.

**5. Receipts**
- **Rules:**
  - The bill slot takes a sheet of your path paper and prints a receipt that falls to the floor.
  - The receipt holds one director-chosen line in the survivor-note voice. It hints and never explains.
  - A receipt on the floor is paper, so the Reader must stop for it.
- **Learns:** machines answer in paper, and receipts stop the Reader.
- **Risk/reward:** a path marker for a hint plus a Reader brake.
- **Build:** low (receipt decal, reuse of the floor-paper item and note pool, printer chatter).
- **Pillars:** 4, 5.

**6. The Price**
- **Rules:**
  - The price display always shows one more than the coins you carry; empty pockets show "1".
  - Behind the glass, rarely and only in the build phase, sit things you lost this run: a spent battery, your bottle, your own marked sheet.
  - It never comments.
- **Learns:** nothing mechanical. It is "costs more than you have" made visible.
- **Risk/reward:** none; pure dread.
- **Build:** low; it needs inventory history.
- **Pillars:** 2, 4.

**7. The Glass**
- **Rules:**
  - The dark glass reflects what is behind you when your lamp hits it.
  - Whatever you see in the glass, you have seen: the Watcher freezes in it, and the one in the corner wakes if its face shows there.
- **Learns:** the glass is an eye that helps against one creature and betrays you to another.
- **Risk/reward:** backing away while watching the glass, against waking the corner-thing.
- **Build:** high (render-target reflection, visibility tests through it).
- **Pillars:** 4, 2.

**8. Sold Out** (the restocker)
- **Rules:**
  - One unpopular slot has a SOLD OUT lamp.
  - Slots change only while the machine is unseen, the Watcher's rule.
  - If the unpopular slot has been restocked since you last looked, a creature's route really did pass the machine.
- **Learns:** the machine is a log of who walked by.
- **Risk/reward:** information only.
- **Build:** low (slot lamps, a hook into entity path history, one relay click).
- **Pillars:** 1, 4.

**My three picks:**
- **1, Change and Nothing.** It turns the canon note into a rule players can test. It is the strongest use of pillar 3: the sound stops, and that is worse. It is also a real two-way trade, since each coin is noise now and silence later.
- **4, The Lit One.** It is cheap and place-based, reuses the dead machines already placed, and extends the Smiler rule without new AI. It is the Ohashi image: warm light in the dark as refuge and lure.
- **3, Rock It.** Everyone will shove the machine, and this makes that verb pay in our existing currency, noise. The real CPSC label is a hint that never explains. The tip-over leaves a lasting mark on the map ("dread, then release").

Leave 2 (The Reverse) unbuilt at first: an unanswered "do not attempt" serves pillar 4. 6 and 8 are cheap layers to add later.
