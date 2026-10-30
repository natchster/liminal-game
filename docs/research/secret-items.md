# Research: secret items that change you (Alice, folklore, SCP, the Backrooms, creepypasta, games)

Research note, 2026-10-30. Brief: scatter secrets around the levels, each with a slightly positive or negative lasting effect that neither its look nor its text gives away, in the manner of Alice's "DRINK ME" and "EAT ME", with items and properties drawn from SCP and creepypasta lore. (The same request asks for subtler navigation hints; this note covers the items, and section 8 notes where they touch the hints.)

**What exists now.** Almond Water (`DRINK_STAMINA` 45), batteries (`SWAP_BATTERY` 55), paper, coins, a machine's snack, lore notes and her pages (`player.rs`, `LORE.md` *Survival*) each do what they seem to do, and nothing changes the player for the rest of a run. The systems a secret could touch, with today's values:

| System | Today |
|---|---|
| Eye height | 1.62 m, 1.05 m crouched (`base/config.rs`) |
| Walking speed | 3.6 m/s walk, 6.4 sprint, 2.0 crouched |
| Stamina | 100. Running drains 20/s; it refills at 14/s after 0.8 s |
| Nerve | 100. It refills at 3/s. The dark takes it down to `DARK_FLOOR` 40 at 6.5/s. The Watcher's touch takes 45 and the glass's 40; under 25 the nerve is spent (`player.rs`, `creature.rs`) |
| The lamp | Drains 4%/s. A Smiler sees it from 25 m (`smiler.rs`) |
| Hounds | Hear your steps. A still player within smelling distance is found in 3 s (`hound.rs`) |
| The window | A look counts within 8 m. The one behind the glass notices at 3 s and comes at 5 s (`glass.rs`) |
| The exit door | Its call is heard about 30 m off (`GAMEPLAY.md` G2) |
| Crying out | Shows the way at 50% on level 1, falling 8 points a level to a 5% floor; otherwise something comes (`player.rs::cry_for_help`) |

**Method and limits.** Every claim links a page I or a helper opened. Book passages come from Gutenberg's plain-text files; Wikidot authors from the Crom API (`api.crom.avn.sh/graphql`) and the credit boxes; Fandom pages from each wiki's `api.php`, their creators from the first revision. Helpers read the SCP pages, the Backrooms pages and the games; I spot-checked SCP-330, the old SCP-1162 and Harris's rules. Limits:
- Reddit was not tried (the window note found it blocked), so no r/nosleep story is here. Sacred-texts and Poetry Foundation returned 403: the *Kojiki* was read through a mirror, *Goblin Market* through Gutenberg. The USC Folklore Archives failed TLS; I did not work around it.
- Morijio has no Wikipedia page; the ladanka rests on one Russian encyclopedia page.
- The original SCP-1162 "A Hole in the Wall" is deleted; its rules come from the Japanese wiki's archived translation, author unknown.
- SCP: Secret Laboratory numbers come from its Fandom wiki (the official one refuses automated reads), which may lag the game; its SCP-1853 and SCP-244 figures were snippets only and are left out.
- The SCP-330 flavours, SCP-207's speed-for-health trade and SCP-714's jade change are game inventions, not on the wiki pages.
- Not found on either Backrooms wiki: "Throat Peelers", "Bottled Thought", "Diet Almond Water", "Unknown Pills". Crawl's changelog does not support "unidentified potions removed in 0.27".

**Licences.**
- **CC BY-SA 3.0:** SCP Wiki and branch pages, and Backrooms Wikidot pages, as each page states. Using a concept means crediting its author and page. The images are excluded: SCP-005's photo, for one, is "used with permission".
- **CC BY-SA:** the Fandom wikis (the Backrooms, Creepypasta, Samosbor, Spelunky, STALKER, SCP:SL, Containment Breach, Brogue) and Russian Wikipedia.
- **Other wiki licences:** wiki.gg is CC BY-SA 4.0 for Isaac and Don't Starve, and CC BY-NC-SA 4.0 for Darkest Dungeon. Pathologic's Fandom wiki is CC BY-NC-SA. NetHackWiki is CC BY-SA 3.0. Fextralife has no open licence.
- **CC BY-SA 3.0 game:** SCP – Containment Breach, except SCP-173 ([Wikipedia](https://en.wikipedia.org/wiki/SCP_%E2%80%93_Containment_Breach)).
- **Copyrighted, motifs only:** every other game, and the Holders series, whose original site's licence is unclear.
- **Public domain:** Carroll, the Homeric Hymn, the Odyssey, Grimm, Andersen, Rossetti, Child's ballads, Hartland and Chamberlain's *Kojiki*, and folk custom.

---

## 1. Alice's "DRINK ME" and "EAT ME"

Source: Lewis Carroll, *Alice's Adventures in Wonderland* (1865), [Gutenberg #11](https://www.gutenberg.org/ebooks/11), read in [pg11.txt](https://www.gutenberg.org/cache/epub/11/pg11.txt). It is public domain, and every point in this section comes from it.

| Find | How it is presented | Effect, and what Alice learns |
|---|---|---|
| **The bottle** (ch. I) | On a table she has already searched, "which certainly was not here before", with a paper label: DRINK ME. She will not drink "in a hurry"; she looks first for "poison", remembering cautionary tales. It is not marked, so she "ventured to taste it": cherry-tart, custard, toffee, hot buttered toast. | "What a curious feeling!" She shrinks to ten inches: right for the little door, but the key is now out of reach. |
| **The cake** (ch. I) | It sits in a glass box under the table, with EAT ME "marked in currants". She reasons that either size gets her into the garden. | A bite, a hand on her head, "Which way? Which way?", and nothing. Then she opens out "like the largest telescope" to nine feet: the key in reach, the door not. |
| **The fan** (ch. II) | The White Rabbit drops it. It is not food and has no label. | She finds she is shrinking again, and "the cause of this was the fan she was holding". Something carried can work on you too. |
| **The second bottle** (ch. IV) | "There was no label this time." She drinks anyway, because "something interesting is sure to happen" whenever she eats or drinks anything. | She grows until an arm is out of the window. She has learned the *kind* of rule, not this item. |
| **The pebbles** (ch. IV) | They turn into cakes on the floor. She reasons that one "can't possibly make me larger", so it must make her smaller. | She is right. This is her first prediction. |
| **The mushroom** (ch. V) | The Caterpillar: "One side will make you grow taller, and the other side will make you grow shorter." The mushroom is "perfectly round". She breaks off a bit from each side: "And now which is which?" | The right shrinks her until her chin strikes her foot; the left gives her a serpent's neck. Nibbling "first at one and then at the other", she gets back to her height, and later doses herself from a piece kept in her pocket. |

**Why it works:**
- **An invitation with no explanation.** The only words on the item ask for the act and say nothing about the result.
- **The reader's caution is part of the scene.** Alice checks for poison, and checking settles nothing. Our player should be able to look the thing over and come away no wiser.
- **The effect is felt, not told.** Alice learns her size from the room around her (the tall table, the far key), not from a meter.
- **Delay.** The cake does nothing at the first bite, so the cause and the effect come apart.
- **Every change is two-sided.** Small fits the door but loses the key. Tall gets the key but loses the door.
- **A learning curve in four steps.** She tries; she generalises (the unlabelled bottle); she predicts (the pebbles); she is given half a rule and must test which half is which (the mushroom).
- **Dosing.** The mushroom works a nibble at a time. Small effects that add up in steps fit our game better than one big switch.

## 2. Eating the other world's food (public domain)

| Source | Rule | What sticks for us |
|---|---|---|
| **Persephone.** *Hymn to Demeter*, tr. Evelyn-White ([Gutenberg #348](https://www.gutenberg.org/cache/epub/348/pg348.txt), ll. 370–433) | Hades "secretly" gives her a pomegranate seed as she leaves. Demeter's first question is whether she tasted food below. If she did, she must go back for "a third part of the seasons every year". | One seed is enough, and the bond is partial: a share of the year. That is the scale of effect we want. |
| **Izanami.** *Kojiki*, tr. Chamberlain ([sacred-texts](https://sacred-texts.com/shi/kj/kj016.htm), read via a [mirror](https://neonvagabond.xyz/mirrors/sacred-texts/shi/kj/kj016.htm)) | "I have eaten of the furnace of Hades." Chamberlain's note reads the character as "a place for cooking". She "was now one with the land of the dead" ([Wikipedia](https://en.wikipedia.org/wiki/Izanami)). Izanagi lights a torch, sees her rotting body, and she chases him. | Food cooked *in* the place belongs to the place. |
| **Thomas the Rhymer.** Child 37 ([Wikisource](https://en.wikisource.org/wiki/The_English_and_Scottish_Popular_Ballads/Part_2/Chapter_37)) | The Queen of Elfland forbids the garden's cursed fruit and gives him bread and wine; in one version she gives him an apple and a tongue that cannot lie, which he does not want. | Safe food against the place's food; a gift good and unwanted at once. |
| **Hartland**, 1891, ch. III ([Wikisource](https://en.wikisource.org/wiki/The_Science_of_Fairy_Tales/Chapter_3)) | Across cultures, eating in the other world binds you to it: "you will be as I am, and return no more to your family". | Players half-know the rule, so one object can hint it. |
| **Fairy protection; hungry grass.** [Wikipedia, Fairy](https://en.wikipedia.org/wiki/Fairy), [Hungry grass](https://en.wikipedia.org/wiki/Hungry_grass) | Eating fairy food was "deemed dangerous", like the food of Hades. People put "a piece of dry bread in one's pocket" before going into a fairy place. In Ireland a crust in the pocket also guarded against hungry grass, which doomed whoever walked on it to "insatiable hunger". | A *carried* food can be a ward rather than a meal. |
| **Fairy ointment**, ML 5070. [Wikipedia](https://en.wikipedia.org/wiki/Fairy_ointment) | A midwife gets the fairy baby's eye ointment in her own eye and sees the fairies' house as it really is. When she admits she can see a fairy, "The fairy invariably blinds her in the eye that can see him". | The best two-sided gift of sight in folklore: you see more, and letting it be seen that you see is what costs you. |
| **Goblin Market.** Rossetti, 1862 ([Gutenberg #16950](https://www.gutenberg.org/cache/epub/16950/pg16950.txt)) | "We must not look at goblin men, / We must not buy their fruits". Laura has "no copper in my purse", so she pays with "a golden curl". After eating she can no longer hear the goblins' cry, which her sister still hears: "Poor Laura could not hear". Her "hair grew thin and grey". | The cleanest change of perception we found: one taste, and you stop hearing what calls you. And she pays with part of herself. |
| **The Lotus-eaters.** *Odyssey* IX, tr. Butler ([Gutenberg #1727](https://www.gutenberg.org/cache/epub/1727/pg1727.txt)) | Those who ate the lotus "left off caring about home". | Food that weakens the pull of the way out. |
| **Snow White's apple.** Grimm, tr. Taylor/Edwardes ([Gutenberg #2591](https://www.gutenberg.org/cache/epub/2591/pg2591.txt)) | The apple was "so made up that one side was good, though the other side was poisoned". | Alice's mushroom again: one object, two sides, and nothing on the surface says which. |
| **Baba Yaga**, after Propp. [Russian Wikipedia](https://ru.wikipedia.org/wiki/Баба-яга), CC BY-SA; Propp's books are in copyright, so the idea only | The hero asks her to feed him and give him drink, which the article reads as his way to enter the world of the dead: a ritual funeral meal. | In Slavic tales the other world's food is a *pass*, not a trap. |
| **Bread and salt; the domovoy.** [Wikipedia](https://en.wikipedia.org/wiki/Bread_and_salt), [Wikipedia](https://en.wikipedia.org/wiki/Domovoy) | Bread and salt welcome a guest; bread strewn with salt is offered to the house spirit, who, neglected, may leave the house "unprotected". | Salt as hospitality and ward. |
| **Ladanka.** [azbyka.ru](https://azbyka.ru/sueverie/ladanka.html); folk custom | A small cloth bag worn on a neck cord, beside the baptismal cross. It holds incense, earth from home or from ancestors' graves, or a written prayer, against illness and "the unclean force". | A charm worn sewn shut. |
| **Food concentrate.** Samosbor wiki, «Продукты питания» ([samosbor.fandom.com](https://samosbor.fandom.com/ru/wiki/Продукты_питания), CC BY-SA; created by ProfessorSacred in 2019, per the [history API](https://samosbor.fandom.com/ru/api.php?action=query&prop=revisions&titles=Продукты_питания&rvlimit=1&rvdir=newer&rvprop=user%7Ctimestamp&format=json)) | The only official food is a brown 300 g briquette in a paper wrapper, bitter and salty with an unpleasant aftertaste. There are also saccharin tablets with a metal taste, and moss that grew where the samosbor and the cleaners' disinfectant had been. People eat the moss anyway, though it is forbidden. | Food that fits the Building's floors (1–3) exactly. |

## 3. SCP Foundation (CC BY-SA 3.0)

All pages are on scp-wiki.wikidot.com unless marked. Authors come from Crom.

| Object | Author, year | Item and rule | Effect and cost | What a game could take |
|---|---|---|---|---|
| [SCP-330 "Take Only Two"](http://scp-wiki.wikidot.com/scp-330) | Dr Kondraki, 2008 | A bowl of wrapped sweets with a taped note: "take no more than two, please!!" Whoever takes a third "instantly has both hands severed at the wrists". The count resets after 24 h. | The page gives the sweets no flavour or effect. The flavours are in the games only. | A rule written on the thing in ordinary words. Two are free and the third costs. The player learns it by trying. |
| Original SCP-1162 "A Hole in the Wall" ([JP archive](http://scp-jp.wikidot.com/old:scp-1162)) | Unknown; JP archive by gnmaee, 2015 | A 14.5 cm hole in a cinderblock wall, always dark inside. Reach in and you find something you lost or were looking for, and one of your things disappears. It moves to another wall if no one uses it for 168 h. | A naked subject, carrying nothing, lost a kidney. The current [SCP-1162](http://scp-wiki.wikidot.com/scp-1162) is an unrelated 2021 page by Rounderhouse. | An exchange: something comes back, something goes. With empty pockets it takes from you. |
| [SCP-427 "Lovecraftian Locket"](http://scp-wiki.wikidot.com/scp-427) | Dr Ouros, 2008 | A silver locket. It works only while open, and its effects add up over total time open. | At 10 min, +500% resistance to disease. At 15 min, +1000% and more strength. Past 1 h total, the wearer turns into a "shapeless mass of tissue". | A good effect that turns bad over a hidden running total. |
| [SCP-268 "Cap of Neglect"](http://scp-wiki.wikidot.com/scp-268) | Pair Of Ducks, 2008 | A tweed newsboy cap. While you wear it, people take you "for granted". Speaking or touching someone gets you noticed again for a while. | After about 20 h of total wear the effect lingers without the cap, and with long wear it becomes permanent. An agent ended up forgotten. | Being overlooked, and being overlooked by help too. |
| [SCP-714 "The Jaded Ring"](http://scp-wiki.wikidot.com/scp-714) | Arcalane, 2009 | A jade ring. It makes the wearer tired: walking pace at best, slow thinking. Shocking sights barely register. | No jade transformation; that is game-only. The mind recovers minutes after removal, and the tiredness passes in 2–3 h. | Calm bought with slowness. |
| [SCP-207 "Cola Bottles"](http://scp-wiki.wikidot.com/scp-207) | Aeish, 2010 | More than 5 mL of the cola triggers it. | No need to rest. Speed and reactions rise about 50% every 6 h. Everyone is dead by 48 h. | Tirelessness that the body pays for. |
| [SCP-178 "3-D Specs"](http://scp-wiki.wikidot.com/scp-178) | 91stCataclysm, 2011 | Cardboard red and blue glasses. | Wearers see tall figures, some leaning over their shoulder. Trying to touch one wounds you until you die. | Seeing more is dangerous only if you act on it. |
| [SCP-1916 "Moon Rocks"](http://scp-wiki.wikidot.com/scp-1916) | Smapti, 2012 | Six jawbreakers, each a colour. The box says one per four hours. | The eater's weight changes for 90 min: lighter (Mars, the Moon), heavier (Jupiter), weightless (black). One colour is lethal. | Colour-coded changes to the body, learned by trying. |
| [SCP-832 "Accountant's Coin"](http://scp-wiki.wikidot.com/scp-832) | Inglonias, 2011 | A 1966 half dollar. | The holder knows exactly what they own, grows careful, and panics if it is taken. | A coin that changes the one who holds it. |
| [SCP-362 "A Cool T-Shirt"](http://scp-wiki.wikidot.com/scp-362) | PaulSharke, 2010 | Comfort while you wear it. | Take it off and staring figures come. After 31 days of wear, taking it off coincided with a site-wide power failure. | The cost comes at removal and grows with time worn. |

**Also read, weaker fits:** [SCP-500](http://scp-wiki.wikidot.com/scp-500) (snorlison; red pills that cure every disease within 2 h, too strong), [SCP-942](http://scp-wiki.wikidot.com/scp-942) (InnocentBystander; a gumball after which every body fluid looks like blood, for good), [SCP-442](http://scp-wiki.wikidot.com/scp-442) (Robert Sandfield; a watch that must be kept wound or its owner can never be on time again), [SCP-269](http://scp-wiki.wikidot.com/scp-269) (Aelanna; a bracelet that cures, then quietly removes the immune system), [SCP-2107](http://scp-wiki.wikidot.com/scp-2107) (Dillinger PhD; a soda whose small dose is a pleasant haunting and whose large dose kills), [SCP-2022](http://scp-wiki.wikidot.com/scp-2022) (Bucell; sunlight pills whose overdose makes the eater glow), [SCP-1109](http://scp-wiki.wikidot.com/scp-1109) (Rioghail; painlessness that spreads to taste, then touch).

**Dropped:**
- SCP-198, SCP-1079 and SCP-513: cost only.
- SCP-294: too open.
- SCP-458: too thin.
- SCP-005: a tool, not a change.
- SCP-063: it erases walls.
- SCP-1499: it takes you to another world.
- SCP-1853: the current page, tagged `in-rewrite`, describes a green contact serum that makes users aggressive, not the games' "dexterity" item.

## 4. The Backrooms wikis (CC BY-SA)

**Wikidot** ([backrooms-wiki.wikidot.com](https://backrooms-wiki.wikidot.com)). Authors come from Crom and the page credits.

| Object | Author | Looks like | Effect and cost |
|---|---|---|---|
| [Object 1 "Almond Water"](https://backrooms-wiki.wikidot.com/object-1) | Natedagreat563 and Poliacci (rewrite); original by 1000dumplings | Dented thermoses of clear liquid, in several bottle colours | Grey is food. Green is caffeine; too much brings insomnia. Red boosts immunity and leaves you "terribly tired". Blue makes you drowsy and shields the mind; too much brings dependence and weak sight. Never drink a half-full bottle. |
| [Object 44 "Cashew Water"](https://backrooms-wiki.wikidot.com/object-44); [archived 2020](https://backrooms-wiki.wikidot.com/archived:object-44) | Positive123 (rewrite); original by yoyoflux | Looks like Almond Water and smells of cashews | Strips calories. The archived version: "Whenever held, it attracts all kinds of Entities", and Almond Water reverses its harm to the mind. |
| [Object 48 "Liquid Pain"](https://backrooms-wiki.wikidot.com/object-48) | ReyDay | An Almond Water bottle with a broken seal and a faint red tint | Fatal. For us, lore only: a broken seal is a warning. |
| [Object 16 "Royal Rations"](https://backrooms-wiki.wikidot.com/object-16) | Sariastuff (rewrite); original by zakkun_ | A waxy palm-sized slab | Ends hunger, but the craving "can linger for years" and other food stops satisfying. |
| [Object 17 "Liquid Silence"](https://backrooms-wiki.wikidot.com/object-17) | Atom_Rutherford | Black liquid | Soaks up sound and is used to make silent shoe soles. Even a little brings unease. |
| [Object 37 "Star Candy"](https://backrooms-wiki.wikidot.com/object-37) | Pearbranch | Multicoloured candy | Calms panic and gives energy. Too much brings panic attacks with no trigger. Hounds that eat it become docile. |

**Fandom** ([backrooms.fandom.com](https://backrooms.fandom.com)). Authors come from the page credits and the first revision.

| Object | Author | Looks like | Effect and cost |
|---|---|---|---|
| [Object 6 "Moth Jelly"](https://backrooms.fandom.com/wiki/Object_6) | KingofGarbage; rewrite by Big16gsx93 and UnderPm | Violet jelly in jars | Stamina for the day, and addictive. Blue: "completely inaudible footsteps" for about a minute. Orange: you glow and entities notice you. |
| [Extended Almond Water Sub-Item Dossier](https://backrooms.fandom.com/wiki/Extended_Almond_Water_Sub-Item_Dossier_(Collaborative)) | Created by Stormy08, 2024; 38 contributors | Variants | Almonds soaked in the water give long, weak protection from mental hazards. The off-brand "Alom Wotor" makes you dizzy and sometimes trip. |
| [Object 46 "Sensitive Candies"](https://backrooms.fandom.com/wiki/Object_46) | Brightened Star | Mint-smelling sweets | An hour of sharper sight and hearing, at the cost of stamina. They appear "where a person intends to return". |
| [Object 400 "Work Juice"](https://backrooms.fandom.com/wiki/Object_400) | DECOY.MPG | A mug of black coffee that smells of almonds | 8 h without tiredness, then a crash with "heightened unease". |
| [Object 44 "Cashew Water"](https://backrooms.fandom.com/wiki/Object_44) | Taangeerine | A yellowish Almond Water | Fear and anxiety. |

**Also read:** [Object 35](https://backrooms-wiki.wikidot.com/object-35) (NavyEOD_24; black vials whose hidden crystal's colour decides the effect), [Object 28](https://backrooms-wiki.wikidot.com/object-28) (SoyShamoy; two identical yellow milks, one lucky warm and unlucky cold), [Object 365](https://backrooms-wiki.wikidot.com/object-365) (Robert Goerman; a tea whose small gains build with use, paid for with a disgusting moment each cup), trimmed [Object 27](https://backrooms-wiki.wikidot.com/trimmed:object-27) (RezCheddar; a sweet lucky while hard, unlucky once soft).

**Dropped as weapons, tools or too strong:** Firesalt, Lightning in a Bottle, Warpberries (a recall), the 3D Vision Glasses (a third-person camera), Super Almond Water (metafiction), and Jelly Rations (plain food).

## 5. Creepypasta, urban legend and charms (motifs only unless marked)

| Source | Item and rule | What we take |
|---|---|---|
| **Kuchisake-onna.** Japan, spread in 1979 ([Wikipedia](https://en.wikipedia.org/wiki/Kuchisake-onna)) | Thrown hard candy (*bekko ame*) or money: she "will stop to pick them up". | It is the Reader's paper rule already, so we leave it alone. |
| **The Midnight Game.** [Creepypasta Wiki](https://creepypasta.fandom.com/wiki/Midnight_Game), CC BY-SA; author unknown, dated 27 January 2012 | The candle going out means he is near. Relight it within ten seconds or close a circle of salt around you. No flashlight. | Salt as the last ward. |
| **Hitori kakurenbo.** Japan, mid-2000s ([summary](https://www.urbanlegendsmysteryandmyth.com/2025/08/one-man-hide-and-seek-hitori-kakurenbo.html)) | A doll stuffed with rice and your nail clippings. You hide with salt water held in your mouth. | Salt water held, not drunk. |
| **Found pennies.** [Dartmouth Folklore Archive](https://journeys.dartmouth.edu/folklorearchive/?p=8492), [another entry](https://journeys.dartmouth.edu/folklorearchive/?p=15597) | "Picking up pennies brings good luck." One informant keeps a heads-up penny in her shoe; some say tails-up is bad luck. In Kokkuri-san, by contrast, the coin "should be spent" afterwards ([Wikipedia](https://en.wikipedia.org/wiki/Kokkuri)). | A coin worn, not spent. |
| **"A silver sixpence in her shoe".** [Wikipedia](https://en.wikipedia.org/wiki/Something_old) | It stands for prosperity and wards off harm from rejected suitors. | The same. |
| **A red envelope on the road.** Taiwan ([SoraNews24](https://soranews24.com/2020/06/22/find-a-red-envelope-on-the-ground-heres-why-you-should-never-pick-it-up/)) | Money and a dead girl's hair: whoever picks it up may be taken as her groom. Another reading: the finder takes on someone's small misfortune. | Picking up money binds you, and the hair inside is the tell. |
| **Omamori.** [Wikipedia](https://en.wikipedia.org/wiki/Omamori) | Not opened, "in order to avoid losing their protective benefits". Old ones are burned. | A charm that works only while closed. |
| **Morijio.** [Japaholic](https://en.japaholic.com/salt-not-just-for-cooking/) | Cones of salt outside a door, to ward off evil and purify the place. | A wordless sign that someone feared a door. |
| **"Please Take Just One".** By Umbrello ([Creepypasta Wiki](https://creepypasta.fandom.com/wiki/Please_Take_Just_One), CC BY-SA) | A boy empties a bowl of candy under that sign, and the owner comes for him that night. | SCP-330's rule, told as a story. |
| **"The Seven Silver Coins Ritual".** By Noname000001 ([Creepypasta Wiki](https://creepypasta.fandom.com/wiki/The_Seven_Silver_Coins_Ritual), CC BY-SA) | The marked hand becomes strong and fireproof, but sunlight on its silver shows the demon where you are. | A gift that light gives away. |
| **Poisoned Halloween candy.** [Wikipedia](https://en.wikipedia.org/wiki/Poisoned_candy_myths) | A long-lived fear of found sweets. "No cases of strangers killing children this way have been proven." | The right pitch: the fear is real, and the harm is rare and small. |
| **Carrots and night sight.** WWII ([Today I Found Out](https://www.todayifoundout.com/index.php/2021/06/how-wwii-made-everybody-think-carrots-were-good-for-eyes-and-why-it-didnt-really-have-to-do-with-tricking-the-germans/)) | British propaganda credited radar kills to pilots eating carrots. Vitamin A only restores normal night sight. | A folk belief that eating X helps you see in the dark. |

**Dropped:**
- The Hand of Glory ([Wikipedia](https://en.wikipedia.org/wiki/Hand_of_Glory)): grisly, and a tool rather than a change.
- The Elevator Game: it has no item.
- The Holders series ([Holder of Fear](https://creepypasta.fandom.com/wiki/Holder_of_Fear); "Object N of 538"): about questions, not the body.
- Andersen's *Little Match Girl* ([Gutenberg #1597](https://www.gutenberg.org/cache/epub/1597/pg1597.txt)): a match by whose light "the wall became transparent", too strong to last.
- The sin-eater ([Wikipedia](https://en.wikipedia.org/wiki/Sin-eater)): bread and ale over a coffin "absorb the sins" of the dead; a fine motif for M.'s camp, but no item here uses it.

## 6. Games with hidden-effect consumables (copyrighted; mechanics only)

| Game | Rule | How the effect is shown or hidden | Reception |
|---|---|---|---|
| **The Binding of Isaac** ([Pills](https://bindingofisaacrebirth.wiki.gg/wiki/Pills)) | Each run maps pill colours to effects at random. Speed up is +0.15 and speed down −0.12. "Horse" pills, 1.43% of drops, are double strength. | A pill shows "???" until taken. After that, every pill of its colour carries its name for the run. At 1 heart, Bad Trip becomes Full Health instead of hurting. | Split. "I always take unknown pills no matter what" against "I never use pills unless I have the PHD" ([Steam](https://steamcommunity.com/app/250900/discussions/0/620700960822474848)). |
| **NetHack** ([Identification](https://nethackwiki.com/wiki/Identification), [Price ID](https://nethackwiki.com/wiki/Price_identification)) | Appearances are shuffled each game. | You #name a type you have tried, and it joins the discoveries list. Shop prices sort items into tiers. | John Harris: identify-on-use makes "the first item of each type the player finds a waste" ([@Play 81](https://www.gamedeveloper.com/design/-play-81-on-rogue-s-item-identification-system)). |
| **Darkest Dungeon** ([Curios](https://darkestdungeon.wiki.gg/wiki/Curios), [Quirks](https://darkestdungeon.wiki.gg/wiki/Quirks_(Darkest_Dungeon))) | Touch a curio by hand or use an item on it. Quirks last, with at most 5 good and 5 bad. | The odds are never shown. Eldritch Altar by hand: 40% +25 stress, 10% a bad quirk removed. With Holy Water it always removes one. | "Pure trial-and-error" ([QT3](https://forum.quartertothree.com/t/darkest-dungeon/73511?page=23)). Players use cheat sheets. |
| **Dark Souls / Elden Ring** ([charm](https://eldenring.wiki.fextralife.com/Fire+Scorpion+Charm)) | "Raises fire attack power, but lowers damage negation." | The words name the trade; the numbers (+12% fire, +10% damage taken) are hidden. | (none found) |
| **Pathologic 2** ([Tinctures](https://pathologic.fandom.com/wiki/Tinctures), [Drugs](https://pathologic.fandom.com/wiki/Drugs)) | A basic tincture: +20% immunity and +10% hunger. Antibiotics: −30% infection and −40% health, taken from the current value so neither meter empties. | The hover text leaves out the side cost. | The launch difficulty drew complaints, and the developer added 16 difficulty sliders ([PC Gamer](https://www.pcgamer.com/uk/pathologic-2-adds-16-difficulty-sliders-so-you-can-suffer-however-you-want-to)). |
| **Don't Starve** ([Monster Meat](https://dontstarve.wiki.gg/wiki/Monster_Meat)) | Raw: −20 HP and −15 sanity. Cooked: −3 and −10. | Cooking lowers the cost. | (none found) |
| **SCP: Secret Laboratory** ([207](https://scp-secret-laboratory.fandom.com/wiki/SCP-207), [330](https://scp-secret-laboratory.fandom.com/wiki/SCP-330), [268](https://scp-secret-laboratory.fandom.com/wiki/SCP-268)) | 207 stacks to 4: sprint 8.82 m/s at a drain of 1 HP/s, rising to 11.47 m/s at 4 HP/s. Grabbing more than two candies from 330 severs your hands, which then drain about 10 HP/s. 268 hides you for 15 s, then needs 2 min. | A candy shows only its smell. Pink: "The strawberry scent is as gentle as it looks." It explodes. | (none found) |
| **SCP – Containment Breach** ([427](https://containmentbreach.fandom.com/wiki/SCP-427), [1162](https://containmentbreach.fandom.com/wiki/SCP-1162), [714](https://containmentbreach.fandom.com/wiki/SCP-714)) | 427 boosts stamina and speed and kills if worn too long. 1162 swaps an item, and with empty hands takes a lung. 714 protects but shrinks the sprint bar. | 427 warns in stages: "refreshed and energetic", then "gentle muscle spasm", then "Your muscles are swelling…". | (none found) |
| **S.T.A.L.K.E.R.** ([SoC artifacts](https://stalker.fandom.com/wiki/Artifacts_in_Shadow_of_Chernobyl)) | Belt artifacts pair a gain with a loss. Crystal: −30 radiation, −18 endurance. | Shown openly. | (none found) |
| **Bloodborne** ([Insight](https://bloodborne.wiki.fextralife.com/Insight)) | Insight, gained partly by eating items, reveals hidden things at 15, 40 and 60 and quietly raises the risk of Frenzy. | The cost is never stated. | (none found) |
| **Enter the Gungeon** ([Curse](https://enterthegungeon.wiki.gg/wiki/Curse)) | "A hidden stat": each point raises the Jammed chance, from 1% up to 50%. | Shrine text only: "You are touched by darkness." | (none found) |
| **Brogue** ([Identification](https://brogue.fandom.com/wiki/Item_Identification), [Use-ID](https://brogue.fandom.com/wiki/Use-ID)) | Detect Magic marks items good or bad before their full identity is known. Bad potions are thrown. | Good-or-bad is a cheaper layer of knowledge. | Players choose where to test, for example near water for a possible fire. |

## 7. Design lessons

**How small can a lasting effect be?**
- **Perception has a floor.** The just-noticeable difference in amplitude is "around 1 dB". An improvement below it is "wasted effort because the improvement will not be perceived" ([Wikipedia, JND](https://en.wikipedia.org/wiki/Just-noticeable_difference)). A 12% cut in a sound's amplitude is about 1.1 dB, right at the edge, and noticed only side by side.
- **Changes to what creatures do need more.** The player cannot replay the same moment, so a change must be about 15–25% before it shows over a few meetings.
- **Games hide much smaller numbers so that they are not noticed.** The last sliver of health is worth more in *Assassin's Creed* and *Doom*, and the first enemy shots in *BioShock* always miss ([GameRevolution, on Jennifer Scheurle's 2017 thread](https://www.gamerevolution.com/originals/347027-firewatch-bioshock-devs-share-hidden-mechanics-never-noticed-games)). Ours are meant to be found, so they must sit above that line.
- **Rule of thumb:**

  | Change | Size |
  |---|---|
  | Eye height | 6–8 cm, seen against every door frame and table top |
  | A sound | 1.5–3 dB |
  | A creature's range or time | 15–25% |
  | Stamina and battery | 10–15%, because their meters show a change |
  | Nerve rates | 15–25% |

**Discoverable but not stated:**
- **A moment you feel, then a consequence you learn.** Alice's "curious feeling": the eye easing down over three seconds, one heartbeat, the hum dipping for a breath. Then the Hound that did not turn, the stamina bar that ends further along.
- **No words for the trade at all.** Elden Ring names its trade and hides the numbers; ours may not even name it (`LORE.md`, *How canon reaches the player*). Only what a real thing of its kind carries may carry text: a card on a bowl, a sachet's printing.
- **Hint the bad ones by their look**, as Brogue shows good-or-bad before identity: a yellow tint, a lock of hair, a broken seal. Rules learned only by blind trial are resented ([QT3 on Darkest Dungeon](https://forum.quartertothree.com/t/darkest-dungeon/73511?page=23)).

**Mixed effects and traps:**
- **No item can kill.** This is Harris's "No cyanide rule": "No unidentified item should be immediately fatal upon use given reasonable circumstances" ([Eight Rules, 2011](https://www.gamedeveloper.com/pc/analysis-the-eight-rules-of-roguelike-design)). For us, no cost may lower `DARK_FLOOR` or raise a touch's nerve so far that the dark plus one touch kills. That is the E10 relation in `creature.rs`.
- **When to take it should be a choice.** Harris's rule 4: effects "would be good to discover" in some situations and bad in others.
- **Every bad item gives something.** Brogue throws its bad potions ([Identification](https://brogue.fandom.com/wiki/Item_Identification)); Spelunky 2's curse sets you to 1 HP but opens the Ghist shop ([Debuffs](https://spelunky.fandom.com/wiki/Debuffs_(2))).
- **Cap stacking.** Darkest Dungeon allows at most 5 good and 5 bad quirks ([Quirks](https://darkestdungeon.wiki.gg/wiki/Quirks_(Darkest_Dungeon))). Cap each system at two steps.
- **Give cautious players a way out.** Some never take unknown pills ([Steam](https://steamcommunity.com/app/250900/discussions/0/620700960822474848)). Leaving a secret untouched must always be a fine choice.

**Should the notebook record what was eaten?** Yes, the act, never the effect. Isaac labels a pill once it has been taken, and NetHack lets you name a tried type. Both answer the player's real question, "did I already eat one of those?", without explaining anything. Proposal: on the notebook's last page, in the player's own hand, one line per secret taken: the thing and the floor ("salt, in a cell — level 2"). No effect is ever written. The guessing stays the player's, as Alice's did.

## 8. Candidates for this game

**Rules for every candidate.** Secrets are placed from the level's seed: one draw from the build stream, the rest from a stream of their own, as the vending machine does (`with_own_stream`). Whether one was already taken this run is fitted afterwards, as `fit_lore_notes` is. At most one or two lie on a floor, off the straight way. An effect is a run-long multiplier or offset on a value in the table at the top of this note, stored with the run, capped at two steps a system, and never dependent on where the player stands. The pickup line names the thing as the others do ("A SACHET OF SALT") and nothing more. No SCP number or object name appears in the game; they go in `SOURCES.md` credits, with the author and the page.

| # | What it is, and where | Draws on (licence) | Verb | Lasting effect | Kind |
|---|---|---|---|---|---|
| 1 | **A paper sachet of salt.** On the Building's floors, on a shelf in a cell whose door has two small cones of salt beside it; in the rooms, in a canteen drawer. | The Midnight Game's salt circle (Creepypasta Wiki, CC BY-SA); morijio; bread and salt; the fairy-bread pocket ward (folk) | Keep in a pocket | A touch that takes nerve takes 20% less (Watcher 45 → 36, glass 40 → 32). | Good |
| 2 | **A small cloth bag on a cord, sewn shut.** Under her bedroll at one of M.'s camps (the camps are fixed canon: a lore-keeper question). | The ladanka; omamori, never opened (folk); SCP-362, whose cost comes on removal (PaulSharke, CC BY-SA) | Put on; it can be opened | The dark drains nerve 25% slower (6.5 → 4.9/s) and stops at 45, not 40. Open it and you find dry earth, and the effect ends for the run. | Good, until opened |
| 3 | **A small jar of blue jelly with no label.** In a ceiling void, behind a tile that has slipped. | Blue Moth Jelly (Backrooms Fandom Object 6, KingofGarbage, CC BY-SA); Liquid Silence (Atom_Rutherford, CC BY-SA 3.0) | Eat | Hounds hear your steps 15% less far. | Good |
| 4 | **Pale mushrooms in the wet carpet** (a soaked Lobby corner, the Poolrooms' edge), **and brown ones on a rotten skirting board** on another floor. | Alice's mushroom (public domain); SCP-1916's colour-coded body changes (Smapti, CC BY-SA) | Eat | Pale: eye height 7 cm lower (1.62 → 1.55; crouched 1.05 → 1.00), steps reach 12% less far, walk and sprint 4% slower. Brown: the reverse. Both: your own size again. | Two-sided |
| 5 | **A twist of paper with a few red seeds,** on a saucer at the end of a dead end. | Persephone; Izanami; the Lotus-eaters; Goblin Market; fairy food (public domain) | Eat | The place takes you a little for its own. The dark drains nerve 20% slower and Smilers see your lamp from 15% less far (25 → 21 m), but the exit door's call carries 20% less far (about 30 → 24 m). | Two-sided |
| 6 | **A ration briquette in brown paper.** Building floors only, in a cell's cupboard. | Food concentrate (Samosbor wiki, ProfessorSacred, CC BY-SA); Royal Rations (Sariastuff and zakkun_, CC BY-SA 3.0) | Eat | Stamina ceiling 100 → 112, but Almond Water restores 20% less (45 → 36). | Two-sided |
| 7 | **A small brown medicine bottle, its label soaked off.** In a washroom cabinet behind its mirror. | Alice's unlabelled bottle; SCP-714's drowsy calm (Arcalane, CC BY-SA); blue Almond Water (Natedagreat563, Poliacci, 1000dumplings, CC BY-SA 3.0) | Drink | Nerve refills 20% faster (3.0 → 3.6/s), stamina 10% slower (14 → 12.6/s). | Two-sided |
| 8 | **Eye drops in a dropper bottle.** In a first-aid box on a maintenance wall. | Fairy ointment, ML 5070 (folk); SCP-178 (91stCataclysm, CC BY-SA); the carrot myth | Put in your eyes | Creatures' eyeshine reads from 25% further in the dark, but the one behind the glass notices your look 20% sooner (3 → 2.4 s; comes at 4 s). You see more, and are seen seeing. | Two-sided |
| 9 | **A tweed cap on a coat hook** in an office wing's cloakroom. | SCP-268 (Pair Of Ducks, CC BY-SA) | Put on; it stays on | Hounds take 25% longer to find you standing still (3 → 3.75 s), but a cry shows the way half as often: what helps overlooks you too. | Two-sided |
| 10 | **A foreign coin with a hole in it, on a string,** in the dust behind a radiator. It will not go into a slot. | The lucky penny and the sixpence in the shoe, the Kokkuri coin (folk); SCP-832 (Inglonias, CC BY-SA); "The Seven Silver Coins Ritual" (Noname000001, CC BY-SA) | Put on | A cry shows the way 15 points more often (cap 60%), but the machines count it as the coin you carry, so their coin returns never ring for you again. | Two-sided |
| 11 | **The hole in the wall,** at hand height: in concrete on the Building's floors, through the wallpaper in the rooms. Once per hole. | The original SCP-1162 (author unknown, CC BY-SA); E.'s way in "through the wall at the back of a storage room" (`LORE.md`) | Reach in | You get back what you last used up (a full bottle if you drank one, a battery if you changed one), and one random thing you carry is gone. With empty pockets it takes from you: the stamina ceiling drops by 15. | Two-sided |
| 12 | **A bowl of wrapped boiled sweets on a reception desk,** with a card: PLEASE TAKE ONE. | SCP-330 (Dr Kondraki, CC BY-SA); "Please Take Just One" (Umbrello, CC BY-SA) | Take (eaten as taken, like the machine's snack) | The first: nerve refills 10% faster. The second: 10% faster again. The third: your hands shake for the rest of the run; the beam trembles, and Smilers see your lamp from 20% further (25 → 30 m). | Good twice, then bad |
| 13 | **An Almond Water bottle with a faint yellow tint and a broken seal,** among plain ones. | Cashew Water (yoyoflux; Taangeerine; CC BY-SA); Liquid Pain's broken seal (ReyDay, CC BY-SA 3.0) | Drink | Stamina as usual, but Hounds find you standing still 20% sooner (3 → 2.4 s). | Bad, with a tell |
| 14 | **A red envelope in a corridor,** with a few notes and a lock of dark hair inside. Only on floors with windows. | The Taiwanese ghost marriage (folk) | Pick up; it cannot be dropped | The one behind the glass comes for your look 25% sooner. | Bad, with a tell |

**Strongest picks, and why:**
- **The mushrooms (4).** The brief's own model, and felt the moment you eat: door handles and table tops are in the wrong place. Eating the other kind undoes it, as nibbling one side and then the other did for Alice.
- **The bowl (12).** The clearest rule learned by trying. The card is the only text, in the words a real one would use, and two are safe, which the player learns only by daring the second. Greed is punished in a way you can see.
- **The hole (11).** The brief's own example, and a clean exchange. Reaching in with empty pockets teaches itself.
- **The seeds (5).** The oldest rule in folklore, made partial as Persephone's was, and felt as Laura's was: the thing that calls you is fainter.
- **The sewn bag (2).** The purest Alice temptation: the only way to learn what it is ends what it does.
- **The salt (1).** A wordless clue in the architecture (the cones by the door), and a purely good find to set against the costs. It protects without breaking the E10 relation.

**Balance.** Three good, eight two-sided, one that turns bad on the third take, and two bad with tells. None needs a new creature, and each changes one or two numbers the game already has.

**Navigation hints.** Three of the candidates change the hints themselves: the seeds weaken the door's call, and the coin and the cap change how often a cry shows the way. If the hints are redesigned to be subtler and more varied, these effects should be re-pointed at whatever replaces them. They should stay small shifts in how readily the place shows you the way, never a new hint of their own.
