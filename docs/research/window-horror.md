# Research: windows that look back (SCP, the Backrooms, folklore, literature, games)

Research note, 2026-10-30. Brief: remake the windows of the rooms to look real, show a frightening landscape beyond them, and on some windows let a long stare draw the attention of something behind the glass. It can come through into the level, but its reach is limited. The rule has to fit canon torn note 27, "Windows": *"The windows here do not open. What they show does not exist. Do not stare too long or it will begin to stare back."* (`crates/game/src/ui/hud.rs`, `LORE_ENTRIES`). It also has to differ from the canon's gaze rules (`LORE.md`, *Things that live here*; `ENTITIES.md` design 1).

**What exists now.** `core_props/builders.rs::build_window` builds a 1.5 × 1.4 m frame with a muntin cross and a sill. The pane is a dark grey canvas with rain streaks, and half the time four warm points of light far off. On wet levels a scrolling rain overlay sits in front (`rain_windows`). `core_props/tail.rs::tail_window_zone` hangs up to three of these in a window zone at 1.55 m, each with a cool point light (0x8090a8, 0.4, 4 m) set 0.6 m into the room. `world/tables.rs` also lists `place_frosted_windows`, `place_broken_window_frames`, `place_window_signs` and, under the occult theme, `place_stained_glass_windows`. Nothing is behind any of them.

**Method and limits.** Every claim links a page I opened. I read Fandom wikis through their MediaWiki API, and took SCP and Backrooms Wikidot authors from the Crom API (`api.crom.avn.sh/graphql`) and the pages' own credit boxes. What I could not do or verify:
- Reddit returned 403, so I read no r/nosleep story.
- The Junji Ito wiki returned 402. "The Window Next Door" comes from one review only.
- I did not open the original 2ch threads for Kunekune and Yamanoke. Kunekune comes from Japanese Wikipedia and Yamanoke from yokai.jp.
- Salem's Lot: no page I opened states the vampire's invitation rule. The invitation rule below comes from *Dracula*.
- Mechanics I could not confirm: a window mechanic in That's not my Neighbor or No, I'm Not a Human, a window PSA in The Mandela Catalogue, and Night of the Living Dead's hands through the boarded windows.
- Folk beliefs I could not confirm: "the dead knock at the window to be let in". The only Slavic source I found gives a knock at the window at night as a *death omen*. The Russian taboo against looking out at night comes from a popular article that cites no sources.
- Not checked, so left out: P.T., Mouthwashing, The Mortuary Assistant, Observation, Little Nightmares, Kairo, Lights Out, Bierce's "The Boarded Window" (not read through), Bachelard's lamp in the window (secondary sources only) and Alberti's "open window" (not found on the page I opened).
- Not windows, so left out: SCP-PL-236 "Okno na świat" (a mirror) and SCP-8432 (a staircase).

**Licences.** SCP Wiki pages, their international branches and Backrooms Wikidot pages are CC BY-SA 3.0, as each page states. Fandom wikis (the Backrooms, Creepypasta) are CC BY-SA. The 2ch stories, Junji Ito, Salem's Lot, Rear Window and every game are someone's work, so for them I take motifs only. Everything in section 4 is public domain: it is hosted on Project Gutenberg or Wikisource, or it is folk custom.

---

## 1. SCP Foundation

| Object | Author (Crom) | Rule | What sticks |
|---|---|---|---|
| [SCP-965 "The Face In The Window"](https://scp-wiki.wikidot.com/scp-965) | ChazzK, 2011 | A pale male face, shadowed, at many ages from about 10 to 55. It shows in a framed window only while the light *outside* is under 5 candelas, whatever the light inside. It moves to another pane only when its pane is destroyed. People who sleep in view of it dream of being chased. After 3–10 dream cycles it smiles more, and they bleed inside. | It is "looking away, into the room." When its window was smashed it appeared in the next chamber's observation window. It looks disappointed in an empty room. *The outside must be dark* is a real optical condition (section 6). |
| [SCP-1148](https://scp-wiki.wikidot.com/scp-1148) "Imperfect Image" | Timmantha, 2012 | It is a *reflection* held on a window, seen only from the lit side. After about 15 min of looking the viewer feels watched and cannot stop looking. About 5 min later figures show in the reflection (a hooded man, a winged biped, a pair of glowing eyes), and they vanish when looked at directly. About 5 min after that comes an urge to go nearer and close a blind that is not there. Within about 2 cm of the glass the viewer is pulled in. No one has been recovered. | Containment is a light difference of at least 75 W between the two sides. Lighting both sides equally should end it, but would push it into another surface. The figures live at the edge of your view, and the reach is exact and tiny. |
| [SCP-675 "Shadows Through the Window"](https://scp-wiki.wikidot.com/scp-675) | Dr Talson; rewrite weizhong | Shadowy figures behind an ordinary living-room window *tap on the glass* and do nothing hostile. After about 45 days, observers are sure the figures are malevolent and plan to "cleanse" this world. Observers have fortified the house and set lethal traps. | The harm comes from the watching, not from the watched. |
| [SCP-419 "Window to the World"](https://scp-wiki.wikidot.com/scp-419) | Kulzn; rewrite Rejekyll | A 1.5 × 3.6 m pane shows a non-Euclidean city, but only to a human looking at it directly. Cameras and sensors see a mirror. About 800 lux comes from the glass, though no sun or moon is in the view. Red figures march in processions, and after an incident the city burns. | A landscape that gives off light it has no source for. It is a view and not a threat, and it ages. |
| [SCP-2885 "Window to Another You"](https://scp-wiki.wikidot.com/scp-2885) | Pythy; rewrite Zyn | Look through a 21 × 14 cm pane for 15 s and a vision starts that *sharpens in stages*. For 0–1 h it is a faint landscape moving at walking pace. At 1–3 h it is sharper, with everyday sounds. At 3–6 h come footsteps and familiar places. At 6–10 h the viewer is functionally blind. Past 10 h the hallucination is total. | A view that resolves the longer you look. That is a staring meter you can *see*. A stretcher is carved "you are not you". |
| [SCP-508 "Random-dot Stereogram"](https://scp-wiki.wikidot.com/scp-508) | Hamsterhuey, 2010 | Stare at it from about 1 m and it becomes a window, after about 93 s on average and never under 60 s even for repeat viewers. It works both ways: the viewer is exposed to the gaze of whatever is viewed. After the same time again it becomes permeable. One subject was pulled through and dismembered. SCP-507 climbed out of the frame. Covering most of the image stops it. | The clearest "it stares back": the same time that lets you see lets it come. |
| [SCP-7640 "A Window to our Souls"](https://scp-wiki.wikidot.com/scp-7640) | TroutMaskReplica and Anonymous, 2023 | Windows appear in O5 quarters and show the viewer the life they gave up. Every viewer tries to resign, and every one has been terminated. Others in the room feel "a heightened curiosity". | A landscape made of what you lost. |
| [SCP-987-ARC "A Window"](https://scp-wiki.wikidot.com/scp-987-arc) | Unknown author (archived) | A 20 × 25 cm sash window shows a ransacked 1987 living room. Naked, bloodstained people pass through it and copy ordinary behaviour without understanding it. The farmhouse family it was found with were sitting in front of it, armed and starved. | The watchers died watching. |
| [SCP-L682](https://scp-wiki.wikidot.com/scp-l682) | CRAt26, 2026 | A figure outside frosted glass, its face covered by black hair and its hand charred. It scrapes its nails on the glass, then taps, drags and presses. It comes in "like a finger through plastic wrap" and then lives the subject's routine. | Through glass without breaking it. That fits "the windows here do not open". |
| [SCP-3052](https://scp-wiki.wikidot.com/scp-3052) | Witryso | A flat stick-figure humanoid that "any subject viewing … will perceive … as facing them directly", from any angle. | A cheap, precise wrongness for a figure behind glass: it always faces you. |
| [SCP-2316 "Field Trip"](https://scp-wiki.wikidot.com/scp-2316) | djkaktus, 2016 | Bodies float in a lake, and viewers come to believe they know them. People who enter the lake are never recovered. The page repeats "You do not recognize the bodies in the water." | A *landscape* as the cognitohazard: familiarity is the lure. |

**International branches:**
- [SCP-397-JP](http://scp-jp.wikidot.com/scp-397-jp) "カーテンの向こう側で" (Beyond the curtain; tokage-otoko, 2014). A curtain on a bare concrete wall. Only while it is *closed* do you see grassland and a forest beyond it. A clawed figure over 6 m tall, dressed in patched, bloodstained clothes, comes out of the forest toward you, and it stops coming while the curtain is open. *A warning for us: a "cover it" counter can be inverted, and then nobody can learn it.*
- [SCP-955-JP](http://scp-jp.wikidot.com/scp-955-jp) "窓の裏に" (Behind the window; boatOB, 2015). A man who looks at one dormitory window from outside, alone, between 19:37 and 26:11, sees a woman reading in an armchair. Months later he sees her at the edges of his vision and in mirrors: 「彼女はそこにいるんだ。」("She's right there."). Shutters block it from both sides.
- [SCP-CN-1779](http://scp-wiki-cn.wikidot.com/scp-cn-1779) "窗外" (Outside the window; Suloob, 2021). Someone alone in a high-floor room sees a woman in white outside the window. She asks to come in, makes to climb over the frame, then falls. Her line: 「我迷路了，您能带我回家吗」("I'm lost, can you take me home?").
- [SCP-CN-446](http://scp-wiki-cn.wikidot.com/scp-cn-446) "窗外的脸" (Face outside the window; W Asriel, 2018). A spider whose abdomen looks like a pale face with no irises. It climbs to windows where people are home and rubs the glass to draw their eyes, and it leaves when the light comes on. It is the "face at the window" explained as an animal. The page calls the likeness "a coincidence".

**What makes them work:**
1. The pane is ordinary. The anomaly is what it shows.
2. **The look takes time**: 15 s (SCP-2885), about 93 s (SCP-508), 15 min (SCP-1148), 45 days (SCP-675). A game needs seconds, but the shape is the same: a stare that adds up.
3. **The reach is short and exact**: 2 cm (SCP-1148), "plastic wrap" (SCP-L682).
4. **Light decides it**: the outside must be dark (SCP-965), or the lit side must be brighter (SCP-1148).
5. Containment is covering: shutters, curtains, an envelope (SCP-955-JP, SCP-987-ARC).

## 2. The Backrooms

**Wikidot** ([backrooms-wiki.wikidot.com](https://backrooms-wiki.wikidot.com), CC BY-SA 3.0). This deepens `backrooms-and-samosbor.md`, which summarised an older version.
- [Entity 2 "The Windows"](https://backrooms-wiki.wikidot.com/entity-2). Rewrite by DrAkimoto, 2025. The concept is by u/glassydude101 and the original entry by an anonymous Fandom user.
  - What it is: a "predatory thoughtspace" that appears on walls, often in place of a real window and styled to suit its level.
  - The view: each viewer sees a different, blurred landscape that feels familiar.
  - Occupied windows hold a silhouette that looks like someone you know. It fades over time and grows denser after it feeds.
  - The reach: "Once within a range of one to two meters, the Window will manifest a physical representation of an arm".
  - Vacant windows: a "newborn" window is born of a longing to escape. A "dying" window threatens anyone within 5–8 m.
  - Its protocols: "Resist taking compulsive actions when observing a Window"; bases cover their windows. To walk away from a vacant window, repeat in your mind "There is no way out, and I accept that."
  - The page gives no time limit on looking.
- [Archived Entity 2](https://backrooms-wiki.wikidot.com/archived:entity-2) (anonymous Fandom user; u/glassydude101). The figure inside is "always pointing at the target". It whispers, and grabs "even if the said Window is closed". A group calls them "The Men Behind The Glass". "Do not trust any Windows in any Level if a Figure is behind it."
- [Level 188 "The Courtyard of Windows"](https://backrooms-wiki.wikidot.com/level-188). The page credits "The original author of this page was 1000dumplings. Rewritten by Kai4C". **`LORE.md` credits Kai4C alone, so 1000dumplings should be added.**
  - A white atrium about 45 m across and 30 m high with 100–200 evenly spaced windows, "most with blinds covering the insides".
  - From a quarter to almost all of the windows are Entity 2.
  - Of the shadowy silhouettes in its corridors: "Do not acknowledge their existence." "Do not look directly at them."
  - Hearing music means leave now. The windows "are not exits."
- [Level 301 "The Window of Hope"](https://backrooms-wiki.wikidot.com/level-301) (I_am_a_dragon). A 2 × 2 m window shows the Eiffel Tower: an exit to the Frontrooms that nobody can reach through the corrosive air. It is the view of home you cannot reach.
- [Level 5.2 "Scenic Views"](https://backrooms-wiki.wikidot.com/level-5-2) (jan Jejasa): hotel walkways face each other across a 50 m gap, and you can see the other side over the railing. It is the Well's arrangement.
- No window rule on [Level 9](https://backrooms-wiki.wikidot.com/level-9) (u/Bart0nius; rewrite Stretchsterz). [Level 700](https://backrooms-wiki.wikidot.com/level-700) (CamaradeAlbabar) is a short suicide piece with no rule, so it is left out.

**Fandom** ([backrooms.fandom.com](https://backrooms.fandom.com), CC BY-SA):
- [Entity 2](https://backrooms.fandom.com/wiki/Entity_2): first edit by MoonlightSerenity in 2022, 43 contributors.
  - It splits windows into *Predatory* and *Innocuous*.
  - Behind the curtains are "Window Shadows", silhouettes with mutated proportions ("extremely long hands, small torsos").
  - Predatory windows go passive near more than four people. They drag a victim into a room behind the glass and keep the blinds shut until the victim is "exterminated".
  - The shadows in innocuous windows vanish when you approach.
- [Level 9/Entities](https://backrooms.fandom.com/wiki/Level_9/Entities): windows "disguise themselves as commonplace features of any building".

**Lesson.** The wikis make windows into lures and grabbers with a 1–2 m reach. Their counters are "avoid" and "cover". None makes *how long you look* the rule, and none shows the player where the reach ends.

## 3. Net folklore and creepypasta

- **Kunekune** ([Japanese Wikipedia](https://ja.wikipedia.org/wiki/くねくね); motif only).
  - Origin: a story on a ghost-story site around 2000, changed and posted to 2ch's occult board in 2003 *labelled as fiction*. The label was lost as it spread.
  - The story: the brother sees what it is and will not say, only that it is better not to know (the page's gist, in my translation). In the 2ch version he looks through binoculars, turns pale, drops them and goes mad.
  - The rule: seen far off, it is harmless. Seeing it in detail, and understanding it, breaks the mind.
  - It is white, moves in no human way, and is seen over summer rice paddies.
  - This verifies and deepens `folklore-and-imaging.md` §25. Hachishakusama tapping at the window is in §24 and not repeated here.
- **Yamanoke** ([yokai.jp](https://yokai.jp/en/yokai/yamanoke); 2ch occult board, 5 February 2007; motif only).
  - A father and daughter spend the night in a stalled car on a mountain road.
  - A white, headless, one-legged thing with a grinning face on its chest hops to the car, muttering "Ten… sō… metsu". It comes to the passenger window where the daughter sleeps.
  - She sits up and murmurs "I got in." A priest gives it 49 days to become permanent.
  - It got in without the glass opening.
- **Creepypasta Wiki** (CC BY-SA). Many short amateur pieces; I list those with a usable mechanic.
  - [Window Man](https://creepypasta.fandom.com/wiki/Window_Man) (first edit Chris-Wreckless, 2014): a figure at a second-floor window taps and points at the lock. The boy shakes his head, and the lock is open anyway.
  - [A Knock on the Window](https://creepypasta.fandom.com/wiki/A_Knock_on_the_Window) (unknown author, 2012): "It was standing inside the whole time, knocking on my window." It is the reflection twist.
  - [Black Windows](https://creepypasta.fandom.com/wiki/Black_Windows) (anonymous 4chan /x/ post): a library break room's blinds were never opened. Behind them the windows are black at 3 pm in June. The staff take their breaks together because alone they "felt creeped out … Like we were being watched."
- **Soviet children's horror** ([New East Digital Archive, Raspopina 2017](https://www.new-east-archive.org/articles/show/9177/soviet-scares-creep-yourself-out-halloween-with-these-ussr-urban-legends)). In *Yellow Curtains* the curtains call at night: "Hey boy, come to the window and see how your grandma's hanging up the laundry outside". They throw him out of the window, then his sister. It is a lure that is *a view*, and it is laundry, as in the Well.
- **Russian window taboos** ([Rambler / Russkaya semyorka, 2023](https://weekend.rambler.ru/read/50229321-okonnye-tabu-samye-strannye-zaprety-u-russkih/); popular, no sources cited):
  - Children were kept from gazing out at night instead of sleeping, since a child at the window may draw in devils.
  - Do not look out at the full moon. Do not look out at a funeral or a coffin.
  - 「Нельзя смотреть в разбитое окно」: do not look through a broken window; cover it with a curtain.
- **Knocking at a high-storey window**: I found no known story, only individual authored pieces on proza.ru and author.today (all rights reserved). Not used.

## 4. Public-domain literature and folklore

| Work | Where I read it | What happens at the window | What we can take |
|---|---|---|---|
| Emily Brontë, *Wuthering Heights* (1847), ch. III | [Gutenberg #768](https://www.gutenberg.org/files/768/768-h/768-h.htm) | The casement will not open: "The hook was soldered into the staple". Lockwood knocks "my knuckles through the glass" and grips "a little, ice-cold hand!" It says "Let me in—let me in!", and "I discerned, obscurely, a child's face looking through the window." He rubs its wrist on the broken pane until it bleeds. "Let *me* go, if you want me to let you in!", and "The fingers relaxed". He "piled the books up in a pyramid against it". | A window that does not open; a reach exactly as long as an arm through a hole; a grip that lets go when you pull away; a hole stopped with paper. |
| Bram Stoker, *Dracula* (1897) | [Gutenberg #345](https://www.gutenberg.org/ebooks/345) | The Count leaves his window and crawls down the castle wall "*face down* … just as a lizard moves along a wall". Van Helsing: "He may not enter anywhere at the first, unless there be some one of the household who bid him to come". Lucy hears "flapping at the window", and later "the head of a great, gaunt grey wolf" fills the broken panes. | Wrong motion on a facade (head first). The invitation rule. |
| M. R. James, "The Mezzotint" (*Ghost Stories of an Antiquary*) | [Gutenberg #8486](https://www.gutenberg.org/ebooks/8486) | An engraving of a manor house. First, at its edge, a head "the back turned to the spectator, and looking towards the house". Then a figure "crawling on all-fours towards the house". Then a ground-floor window stands open: "he must have got in". At last it strides "towards the front of the picture" carrying a child. **Check against the brief:** it does *not* come nearer while looked at: "From five to seven the three companions sat and watched the picture by turns. But it never changed." It changes between viewings, which is the Watcher's rule. | Its path: far off, crossing the lawn, in at a window, coming toward the viewer. Not its timing. |
| M. R. James, "Oh, Whistle, and I'll Come to You, My Lad" (same book) | [Gutenberg #8486](https://www.gutenberg.org/ebooks/8486) | The room has "neither blinds nor curtains". The thing made of bedclothes is blind and gropes. Parkins nearly goes out of the window. The Colonel judges that "its one power was that of frightening". | A thing whose harm is fear alone: a nerve cost, not a body cost. |
| N. V. Gogol, "Viy" (Wikisource text of 1842) | [ru.wikisource](https://ru.wikisource.org/wiki/Вий_(Гоголь)) | The dead witch walks the church blind, "as if wishing to catch someone". Khoma draws a chalk circle, and "she evidently had no power to cross it". Viy asks for his eyelids to be lifted, and an inner voice whispers 「Не гляди!」 ("Don't look!"). Khoma looks, and Viy cries 「Вот он!」 ("There he is!"). At cockcrow the spirits rush for the windows and doors and stay there 「завязнувши в дверях и окнах」 ("stuck in the doors and windows"). | A reach line drawn on the floor that it cannot cross. Things stuck halfway through a window. Looking is what lets it find you. |
| E. A. Poe, "The Raven" (1845) | [en.wikisource](https://en.wikisource.org/wiki/The_Raven_and_Other_Poems/The_Raven) | A tapping "at my window lattice". "Open here I flung the shutter … In there stepped a stately raven". It never leaves, and "the lamp-light o'er him streaming throws his shadow on the floor". | What came through the window is known by its shadow on the floor. |
| H. G. Wells, "The Crystal Egg" (1897) | [en.wikisource](https://en.wikisource.org/wiki/Tales_of_Space_and_Time/The_Crystal_Egg) | The crystal shows a plain "as if from a tower or a mast". Then "a face … with very large eyes, came as it were close to his own and as if on the other side of the crystal". Some people cannot see its light at all, and it glows only in a ray of light under a millimetre wide. | A view from a height that cannot be. The other side looks back through its own window. |
| H. P. Lovecraft, "The Music of Erich Zann" (1922) | [en.wikisource](https://en.wikisource.org/wiki/Famous_Fantastic_Mysteries/Volume_12/Number_3/The_Music_of_Erich_Zann) | The garret's lone curtained window is "the only point on the steep street … from which one could see over the wall at the summit", and Zann drags the narrator away from it in "a frightened rage". When the narrator does look: "I saw no city spread below … but only the blackness of space illimitable". | A window that shows what should not be there. |
| F. Marion Crawford, "The Upper Berth" (Wikisource header: 1885) | [en.wikisource](https://en.wikisource.org/wiki/The_Upper_Berth) | However hard it is screwed down, the porthole is found "wide open and fastened back". The cabin smells "horribly of stagnant sea-water", and there is an arm "smooth, and wet, and icy cold". | A window that will not stay shut. Cold and wet as tells. |
| Irish wake | [Wikipedia, Wake](https://en.wikipedia.org/wiki/Wake_(ceremony)) | "windows of a wake house are left open to let the soul leave the room"; "mirrors are covered or turned around"; "clocks are stopped". | The window as the way between the living and the dead; covering as protection. |
| Ukrainian death beliefs | [Kukharenko, *Folklorica* XVI (2011)](https://journals.ku.edu/index.php/folklorica/article/download/4209/3966) | Warning signs of a death include "a knock on the window at night or birdcalls [Hnatiuk 1904: 115-16]". The dead man's eyes were covered with coins, because "his stare … would cause another death in the family". Poppy seed was sown round graves for the unquiet dead to count. | The knock as an omen. The dead's stare as the danger. |
| East Slavic and Polish funeral rites | [Andryunina, *Traditional Culture* 19(3), 2018](https://www.trad-culture.ru/en/article/anthropomorphic-symbolism-house-funeral-and-remembrance-rites-body-soul-and-sacrifice) | "Universally doors and windows were opened" for the dying, and coffins were carried out through such openings. That survives now only for the "unholy dead". | The window as a door for what should not use the door. |
| Mokumokuren (目目連) | [Wikipedia](https://en.wikipedia.org/wiki/Mokumokuren); [Japanese Wikipedia](https://ja.wikipedia.org/wiki/目目連) | Eyes fill the torn shoji (paper screens) of a ruined house, in Toriyama Sekien's *Konjaku Hyakki Shūi*. One tale has a traveller wrap his head to ignore them and wake without his eyes. "The only way to remove the spirit from the wall is to patch up the holes in it." Japanese Wikipedia: probably Sekien's own invention (Murakami Kenji); it links the eyes to the Bergen illusion and to a moonlit screen taken for eyes. | The counter: patch the holes. The eyes are pareidolia, so a frosted pane can make them. |
| The "don't look" taboo (見るなのタブー) | [Japanese Wikipedia](https://ja.wikipedia.org/wiki/見るなのタブー); [Izanami](https://en.wikipedia.org/wiki/Izanami); [Tsuru no Ongaeshi](https://en.wikipedia.org/wiki/Tsuru_no_Ongaeshi) | A worldwide motif: Lot's wife, Orpheus, Izanagi, the crane wife. Izanagi lights his comb, sees Izanami's decay, flees, and seals Yomotsuhirasaka with a boulder. From behind it she swears to kill 1,000 a day. In the crane tale the husband "peeks in" and she flies away for good. | Looking breaks the boundary, and a boundary (the boulder) holds the pursuer back. |

## 5. Games and films

- **Slender: The Eight Pages** (Parsec Productions, 2012; [Wikipedia](https://en.wikipedia.org/wiki/Slender:_The_Eight_Pages)). "Looking at him for too long causes a game over." He is announced by "audio cues and visual distortions, similar to TV noise." *Lesson:* a stare timer is fair only with a tell that grows.
- **Five Nights at Freddy's** (2014; [Wikipedia](https://en.wikipedia.org/wiki/Five_Nights_at_Freddy%27s_(video_game))). Lights check the hallway on each side, at a cost in power. A screenshot shows an animatronic "standing at the right hallway window". Critics found the dread in what *might* be there, and some found the jump scares repetitive.
- **Look Outside** (Francis Coulombe / Devolver, 2025; [Wikipedia](https://en.wikipedia.org/wiki/Look_Outside)). Something outside transforms "those who observe it", "directly or indirectly". Sam feels an urge to look out, and you choose whether to answer knocks at the door. Metacritic 83. Critics called its difficulty "both too hard and too easy". *Lesson:* the urge to look is the game.
- **That's not my Neighbor** (Nacho Sama, 2024; [wiki](https://thats-not-my-neighbor.fandom.com/wiki/That%27s_Not_My_Neighbor)). A doorman keeps doppelgangers out of an apartment block. I did not verify a window mechanic.
- **No, I'm Not a Human** (Trioskaz, 2025; [Wikipedia](https://en.wikipedia.org/wiki/No,_I%27m_Not_a_Human)). Visitors knock at night, and a new tell is announced on TV each day. Visitors do not show every trait, and some humans show a few: tells with noise in them keep the doubt alive.
- **Fears to Fathom – Home Alone** (Rayll, 2021; [wiki](https://fearstofathom.fandom.com/wiki/Fears_to_Fathom_-_Home_Alone)). A man watches the house. A window shatters upstairs, and his shadow waits in the parents' doorway.
- **Iron Lung** (David Szymanski, 2022; [Wikipedia](https://en.wikipedia.org/wiki/Iron_Lung_(video_game))). "The forward viewport has been encased in metal". You see out only through a poor camera's photos. The *missing* window is the horror.
- **Films and comics** (owned, motif only):
  - *Salem's Lot* (1979 miniseries; [Wikipedia](https://en.wikipedia.org/wiki/Salem%27s_Lot_(1979_miniseries))): "the scenes of vampire boys floating outside windows" inspired *The Lost Boys*. Danny Glick "flies up to Mark Petrie's bedroom window" ([King wiki](https://stephenking.fandom.com/wiki/Danny_Glick)).
  - *Rear Window* (1954; [Wikipedia](https://en.wikipedia.org/wiki/Rear_Window)): Thorwald realises he is watched, crosses the courtyard and drops Jeff from his own window. Ebert: "we looked too, and so we share the guilt."
  - *Skinamarink* (2022; [Wikipedia](https://en.wikipedia.org/wiki/Skinamarink)): "the windows, doors, and other objects in their house are gradually vanishing". K. Austin Collins: "It's scary because it already knows that our imagination will do half of the work". Audiences were polarised.
  - *Bird Box* (Malerman, 2014; [Wikipedia](https://en.wikipedia.org/wiki/Bird_Box_(novel))): people begin "hiding in their homes and covering the windows."
  - Junji Ito, "The Window Next Door" ([review](https://thegeekiary.com/?p=50147)): "the neighbor's house only has one window", facing the boy's. Her arms reach toward it.
  - *Night of the Living Dead* (1968) is public domain because it was "mistakenly created without a copyright notice" ([Wikipedia](https://en.wikipedia.org/wiki/Night_of_the_Living_Dead)). Ben boards up the windows.
- Local 58's "Weather Service" ("viewers should not look at the night sky"; [Wikipedia](https://en.wikipedia.org/wiki/Local_58)) is already in `folklore-and-imaging.md`. [The Mandela Catalogue](https://en.wikipedia.org/wiki/The_Mandela_Catalogue)'s Wikipedia page describes no window PSA.

**Lessons:**
1. Give a stare limit a growing tell (Slender).
2. Checking a window costs something (FNaF's power); frequent jump scares wear out.
3. Tells may be noisy, but they must be learnable (No, I'm Not a Human).
4. The urge to look must come from the player, not the script (Look Outside).

## 6. Why window horror works

- **The window is a shelter and a lookout at once.** Appleton's prospect–refuge theory (*The Experience of Landscape*, 1975) explains taste by two innate desires, for a wide view and for a protected place ([Wikipedia](https://en.wikipedia.org/wiki/Jay_Appleton)). A window gives both, until the view looks back.
- **Being seen when you think you are unseen is real optics.** In the [Naked Scientists' experiment](https://thenakedscientists.com/get-naked/experiments/ghostly-windows), roughly 95% of light passes through glass and about 5% reflects (their rough figures).
  - With your torch on and theirs off you see your own reflection; with theirs on and yours off you see them.
  - At night a lit room is a stage and its window is a mirror to the people inside. A [one-way mirror](https://en.wikipedia.org/wiki/One-way_mirror) is the same effect with a metal coat: the lit side sees itself.
  - This is SCP-965's dark outside and SCP-1148's light difference.
- **A blur reads as a stare.** When unsure where someone is looking, "we are more likely to think that the person is staring at us, even when they aren't" ([MRC CBU on Mareschal, Calder & Clifford, *Current Biology* 2013](https://www.mrc-cbu.cam.ac.uk/blog/tag/calder/)). A figure far off and half-resolved needs no animated eyes to feel like it is looking at you.
- **The feeling of being stared at is common, and false.** It was reported by "over two thirds of the students questioned" (1913). Titchener (1898) and Coover (1913) found no real sense, about 50% in Coover's trials, and traced the "unpleasant tingling" to attention on the neck ([Wikipedia](https://en.wikipedia.org/wiki/Psychic_staring_effect)). The game can cue the feeling with sound; it does not have to justify it.
- **The safe distance collapses.** A courtyard (*Rear Window*), a lawn ("The Mezzotint") or a gap between houses (Junji Ito) is crossed.
- **The outside should not exist.** Erich Zann's window, SCP-419's sourceless light, Level 301's view of home. In our canon the Building has "no outside and no sky" (`LORE.md`), so any landscape is Fisher's "something present where there should be nothing" (quoted in `vending-horror.md` §5).
- **The frame cuts the view down.** "The information is narrowed down by the window" ([Gigazine on danwitz's *Dark Doings*, 2009](https://gigazine.net/gsc_news/en/20090815_dark_doings)). This is `folklore-and-imaging.md` P7, small degraded views. Totaro's audio-visual essay asks the same question, "Why has the image of a face at the window been so prevalent in horror?" ([Offscreen, 2018](https://offscreen.com/view/the-face-at-the-window-a-glimpse-into-an-iconic-image)); the page holds no text of the argument.
- **Everyone's answer is to cover it.** Irish wakes cover mirrors. Russians curtain the night window and cover broken glass. Mokumokuren is ended by patching the holes. Lockwood piles books against the hole. Entity 2's bases cover their windows. SCP-955-JP is shuttered. Players will try this unprompted, so it should work (SCP-397-JP shows what an inverted cover does to learnability).

## 7. Making the window itself real (short)

- **Glass is a mirror whenever the room is brighter than the view.** Under the rooms' fluorescents a night landscape would show mostly the room. Either make the view brighter than the room (an overcast day, a floodlit yard, a sodium lamp just outside), or keep a faint reflection layer and let it be part of the fear (concept 4). Double glazing gives two offset reflections.
- **Parallax sells depth.** Interior mapping (Joost van Dongen, CGI 2008) traces the view ray against virtual room planes on a flat pane. It "eats little performance", SimCity (2013) used it, and Unreal ships it as InteriorCubeMap. Its limits are flat furniture and faults that show close up ([80.lv](https://80.lv/articles/interior-mapping-rendering-real-rooms-without-geometry)). For a landscape, two or three depth layers do the same job: a fixed horizon, mid-distance poles with parallax, and grass near.
- **Keep it on the window's own material.** Project memory: a texture read added to a global shader patch costs about 2.7× compile time on Metal. Also take the view from a CC0 photograph or a `canvas_tex` on its own stream (invariant 1), never from per-frame random.
- **Real dressing.** A reveal 10–20 cm deep, a sill, condensation in the lower corners, dirt, the existing rain overlay, and an office Venetian blind. Real blinds carry a cord warning on the bottom rail: "Cords and bead chains can loop around a child's neck and STRANGLE." ([CPSC recall, 2009](https://www.cpsc.gov/Recalls/2009/risk-of-strangulation-prompts-recall-to-repair-faux-wood-blinds-by-american-vintage)). It is fixed printing of the kind `LORE.md` allows, like the vending machine's label, and it echoes the *Yellow Curtains*.
- **The window's light on the floor is a placed light, in the light map** (invariant 2). Today's spill is a point light, which cannot draw the muntins' cross. A light-map patch, a cookie or a decal could.

## 8. How it must differ from the canon's gaze rules

| Thing | What your eyes do | What it does | You are safe when |
|---|---|---|---|
| The Watcher | keep it in sight | moves only when unseen | you look at it |
| The one in the corner | must not meet its lit face | screams and comes at one glance | you look away when the hum stops |
| The figure across the void | look at it | turns its back, then is gone | always (it never comes) |
| The Reader | (it sees *you*, lit or moving) | comes when sure | you drop paper |
| The Face (Stairwell) | it waits at the edge of your light | jerks nearer, never chases | you walk toward it |
| The vending machine | look at a coil | shows that coil's code | always |
| **The window (proposed)** | **linger on its glass: time, not a glance, and on the view, not a face** | **resolves, comes only while you look, and does not go back when you look away** | **you are out of its light on the floor, or its glass is covered** |

Constraints from the open issues in `QUALITY.md` ("the lore review's sameness"):
- No tell may be a sound that stops; the Hound, the corner and the Reader already use one.
- No counter may be stillness in the dark (the Reader and the Smiler).
- Avoid "tall", which is said of three already.
- Avoid a turned back, which the corner, the far figure and the Smiler's idle already use. The Mezzotint's back-turned start is therefore *not* taken.

## 9. Seven concepts

All of them follow these rules:
- Most windows are only windows. An *occupied* one is unmarked.
- At most one occupied window a floor, from depth 2 (level 5 on), off the main way, never on the Building's floors or an anomaly's.
- It is placed, never sent (as the corner is). At rest it is nothing to the director, and it never comes during a relief.
- Its touch follows the shared fairness: two tells or more, a first touch that throws you down and lets you go, death only for a spent mind.
- Choose the occupied window with a salted `SplitMix64` so that no build draw shifts (`.claude/rules/world.md`).
- Its sounds are held notes (`note()`, not the plucked `tone()`), start silent, and play only within earshot. Its listen test must fail at gain 0 (project memory).

**What counts as staring** (concepts 1, 2, 5, 7):
- The pane's centre is within about 8° of the view's centre (tighter than the Watcher's "lenient cone, about the screen"), within 8 m, and nothing stands between you and it.
- The meter fills at 1 a second and empties at 0.25 a second otherwise, so glances add up. It is full at 10.
- No randomness; it reads the eyes only.

**1. The one behind the glass** (a field and its lamp)
- **Landscape:** a flat field at night under low cloud, lit orange from below by a town past the horizon, with a line of wooden poles. One sodium lamp on a pole stands just outside. It throws an orange patch, crossed by the muntins' shadow, onto the floor inside. Nothing moves but the grass.
- **Look:**
  - Far off (60–120 m) it is a pale smudge: a belted raincoat with the hood up, the face in shadow.
  - Its error is that it is *flat*: it always faces you (after SCP-3052). At the glass, step sideways and it turns with you without moving its feet.
  - Shoulders about 15% too narrow. Never "tall".
  - Eyeshine: two points in your lamp's beam, behind your lamp's own reflection on the pane, never a glow (`folklore-and-imaging.md` P3).
- **Tells:**
  - At 3: it is sharp (SCP-2885's sharpening), and its eyes catch your lamp. The note's "begin to stare back" is literal.
  - At 6: it is nearer. It walks only while you look, in a stepped gait, and stands when you look away, never going back. A glass tick.
  - At 8.5: its face is pressed white on the pane, and condensation blooms on *your* side. One fingertip tap. Its shadow falls across the orange patch on the floor, so you can see it without looking at the window (Poe's shadow).
- **Through:** the pane bows in like cling film round its head and hands (SCP-L682's "plastic wrap"; the glass never breaks, because the windows do not open). It leans through to the waist, then steps down into the patch.
- **Reach:** the lit patch and nothing else. With a sill at 0.85 m and a lamp 40° up, the patch runs from about 1 m to 2.7 m out from the wall (my geometry), the width of the window. It cannot set a foot outside the light (Viy's circle). The edge is crisp, so you see the limit.
- **Harm:** in the patch it takes your wrist and draws you to the glass, and your nerve drains while it holds. The first time it lets go ("The fingers relaxed") and you fall outside the light. A spent mind goes through. The death frame is the yellow room seen from the field, through the glass, your lamp lying on the floor.
- **Counter:**
  - Do not linger.
  - Once it is through, stay out of its light. It stands in the light facing you until you are out of its sight, then is a smudge in the field again.
  - Press a sheet of paper on the pane, or lower a blind, and that window is done for the floor.
- **Sound:** glass only: thermal ticks, a tap, skin squeaking on wet glass, the low flex of the bowing pane. No voice.
- **Learns:** step out of its light just in time three times, and next time the lamp outside hangs lower, so the patch is visibly longer.
- **Distinct:** your eyes' *time* on a view. It is the Watcher's mirror (it moves only while seen), but looking away does not undo it. It faces you where the far figure turns away. Its reach is a lit shape you can see. Its counter is a place and a cover, not a gaze.
- **Sources:** SCP-2885, SCP-3052, SCP-L682 and Entity 2 (CC BY-SA 3.0); Kunekune (motif); Viy, "The Raven" and *Wuthering Heights* (public domain).
- **Build:**
  - Art medium: a coat on a thin body, a layered field backplate, the bowing pane.
  - Code medium: the stare meter, a patch in the light map with a moving shadow, a creature held inside a polygon.
  - Sound low.

**2. The window opposite** (offices: the light well)
- **Landscape:** an office window onto a light well 6–9 m across. The far wall is all windows, done with interior mapping: blinds down, a few lit. One window opposite has its blind up, with a dark office behind it.
- **Look:** someone stands close to that glass with both palms flat on it, lit from below by a monitor.
- **Trigger:** staring at *that* window.
- **Tells:**
  - At 3: its hands come off the glass, and the blind there drops.
  - At 6: blinds opposite go up one by one, a storey nearer each time. It is coming through the rooms over there.
  - At 8.5: it comes across the well along the wall, head first, "as a lizard moves along a wall" (Dracula). Then a tap on your glass.
- **Through and reach:** as in concept 1. The light is the well's grey daylight, crossed by your blind's slats.
- **Counter:** lower your own blind. The cord rattles the slats down, which is a noise event Hounds hear, and you lose that view. The bottom rail carries the real cord warning (section 7).
- **Distinct:** a rule about one window you can name, "the one with the blind up". Its counter is priced in noise, the currency of the Hounds and of the vending machine.
- **Sources:** Level 188 (1000dumplings, Kai4C; CC BY-SA 3.0: blinds; "Do not look directly at them"); Level 5.2 (jan Jejasa; CC BY-SA 3.0); *Rear Window* and Junji Ito (motifs); *Dracula* and *Yellow Curtains* (public domain); CPSC (a public notice).
- **Build:** art medium-high (facade, a blind with its cord); code medium; sound low.

**3. Let me in** (the broken pane)
- **Landscape:** snow driven across a moor at night, a dry-stone wall, a fir branch at the glass.
- **Look:** one pane is broken, with a fist-sized hole and frost round it. Only a small white hand and forearm are ever seen, and frost forms where it touches.
- **Trigger:** looking at the hole from within 1.5 m for about 5 s. Looking *closely* is what wakes it.
- **Tells:** frost creeps from the hole across the pane. The branch knocks, then a knuckle (the Ukrainian omen: a knock at the window at night). Fingers curl over the hole's edge.
- **Through and reach:** an arm through the hole, reaching 0.7 m. Within that it grips your wrist and draws the view to the hole. Back away hard and it lets go.
- **Counter:** wedge a sheet of paper or a book into the hole (Lockwood's books, Mokumokuren's patched holes).
- **Distinct:** no figure and no view to stare at. It is a close-range reach trap, the cheapest way to teach "do not linger at the glass".
- **Sources:** *Wuthering Heights*, Mokumokuren and Ukrainian belief (public domain); Entity 2's arm (CC BY-SA 3.0).
- **Build:** low (a hand and arm, a frost decal, a holed pane).

**4. The night mirror** (lit rooms)
- **Landscape:** the glass shows mostly the room behind you, as real glass does at night (section 6), with a dark field faint beyond.
- **Look and tells:** stare at the reflection, and someone stands in the *reflected* room behind you, never in the real one. It grows sharper, and it comes up behind your reflection only while you watch the glass. Its reflected hand reaches your reflected shoulder.
- **Through and reach:** it steps out of the glass, but only into the part of the room the glass shows you.
- **Counter:** stand where you cannot see yourself in it, or cover it.
- **Distinct:** it lives in the reflection (SCP-1148). It overlaps `vending-horror.md` concept 7 (the dark glass that reflects what is behind you).
- **Sources:** SCP-1148 (Timmantha), SCP-965 (ChazzK) (CC BY-SA 3.0); "A Knock on the Window" (Creepypasta Wiki, CC BY-SA).
- **Build:** high (a render-target reflection and visibility through it).

**5. The lamp in the field** (not hers)
- **Landscape:** a dark field. Far off a small warm light swings at walking height, the colour of M.'s candle lantern.
- **Tells:** the light comes nearer only while you look, and you see that it is carried. At the glass the lantern is lifted to the pane, and behind it a face gives back your lamp twice.
- **Through and reach:** it sets the lantern on the inside sill and can move only within that lantern's light.
- **Counter:** cover the glass or leave the light. The lantern goes out when it goes.
- **Lore:**
  - It plays on her camps ("her lamp gutters out as you lift the page") and on pillar 2's "not every trail is hers".
  - It must never say what it is, and it must not answer who lays the flat sheets.
  - Lore-keeper risk: medium to high.
- **Sources:** Entity 2's familiar silhouettes, SCP-CN-1779's "I'm lost, can you take me home?" and SCP-7640 (CC BY-SA 3.0); *Wuthering Heights* ("I'd lost my way on the moor!", public domain).
- **Build:** medium.

**6. Eyes in the frosting** (frosted and papered windows)
- **Landscape:** none. A frosted pane, light behind it.
- **Tells:** staring makes the dark flecks in the frosting become eyes, first one and then several, and they follow you (the pareidolia of Mokumokuren). The frosting thins to a hole.
- **Through and reach:** fingers push through the hole, reaching as far as an arm.
- **Counter:** paper over the hole.
- **Distinct:** the window itself is the creature, as in Entity 2's first version. It suits `place_frosted_windows`.
- **Sources:** Mokumokuren (public domain); Entity 2 (CC BY-SA 3.0).
- **Build:** low (decals that open, a hole).

**7. The facade** (a window onto the Well)
- **Landscape:** the Well seen from a window in the rooms: the panel facade across the void, its washing lines, a lit window or two. It joins the two halves of the world (`backrooms-and-samosbor.md` concept 7).
- **Tells:** something leaves a lit window over there and comes down the facade head first, only while you watch. It crosses the void hand over hand along a washing line. It reaches your glass.
- **Lore risk: high.** "Nothing walks the Building". Even though "what they show does not exist", players will read a walker in the Building.
- **Recommendation:** use the Well only as *landscape*, with no figure in it.
- **Sources:** *Dracula* (public domain); Level 188 (CC BY-SA 3.0); `LORE.md` (the Well).

| # | Landscape | Trigger | Reach shown as | Counter | Build | Lore risk |
|---|---|---|---|---|---|---|
| 1 | field, lamp outside | stare 10 s | lit patch on the floor | leave the light; paper or blind | medium | low |
| 2 | light well, the window opposite | stare at that window | lit patch | lower the blind (noise) | medium-high | low |
| 3 | moor, broken pane | look closely, about 5 s | arm's length from the hole | stop the hole | low | low |
| 4 | the room's reflection | stare at the reflection | what the glass shows you | stand out of the mirror; cover | high | low |
| 5 | field, a lantern | stare | its lantern's light | cover; leave the light | medium | medium-high |
| 6 | frosting | stare | arm's length | paper over the hole | low | low |
| 7 | the Well | stare | lit patch | as 1 | medium | high |

## 10. My three picks

1. **The one behind the glass (1).**
   - One sense (your time on its glass), one counter (stay out of its light, or cover it), at least three tells, and a reach anyone can see.
   - The reach is a placed light in the light map, which obeys invariant 2 instead of working round it.
   - It differs from every gaze rule in the canon (section 8) and from the sameness issues in `QUALITY.md`.
   - It reads note 27 word for word: the windows do not open (it comes *through* the glass), the view does not exist, and it stares back.
   - Its strongest motifs are public domain: Viy's line, Poe's shadow on the floor, Lockwood's grip that lets go.
2. **The window opposite (2), as the same creature in offices.**
   - It keeps the same rule. It adds the most real architecture, a light well and a wall of blinds, which the brief's "more realistic" asks for.
   - Its cover verb costs noise, which ties the window into the Hound economy.
   - It also gives the remake its best dressing: blinds, a cord warning, interior-mapped windows opposite.
3. **Let me in (3), as the teacher.**
   - The first occupied window a run meets should be a broken pane. It is cheap and close, and it teaches "do not linger at the glass" at low cost before the full creature.
   - It is wholly public domain apart from Entity 2's arm.

Not first: 4 (costly, and it overlaps the vending Glass); 5 (only after a `lore-keeper` review); 6 (a cheap variant for frosted windows later). For 7, take the landscape and never a figure in it.

**Lore.**
- Note 27 already hints all three parts, so no new note is needed.
- Let what is absent speak: here and there a window covered with taped sheets, or a blind pulled down and left, says someone was afraid, without a word (pillar 1).
- An optional line on one of M.'s pages must pass `lore-keeper`, and must hint, never explain.
- In `LORE.md` it would join *Things that live here* as placed and never sent, like the one in the corner, with an attribution entry.

**Attribution:**
- **CC BY-SA 3.0, if built** (concept only, our own words):
  - Entity 2 (concept u/glassydude101; original by an anonymous Fandom user; rewrite DrAkimoto): the reach and the covering.
  - SCP-2885 (Pythy; rewrite Zyn): sharpening with time.
  - SCP-3052 (Witryso): always facing you.
  - SCP-L682 (CRAt26): through the glass like plastic wrap.
  - SCP-965 (ChazzK): the dark outside.
  - SCP-1148 (Timmantha): only if concept 4 is built.
  - Level 188 (1000dumplings; rewrite Kai4C): for concept 2. Fix the existing Level 188 credit in `LORE.md` either way.
- **Motif only, credited as inspiration:** Kunekune, Yamanoke, Hachishakusama (2ch), Junji Ito, *Salem's Lot*, *Rear Window*, *Look Outside*.
- **Public domain:** Brontë, Stoker, M. R. James, Gogol, Poe, Wells, Lovecraft's 1922 story, Crawford, Sekien's Mokumokuren, the Kojiki's Izanagi, and the Irish, Ukrainian and Slavic customs.
