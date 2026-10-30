# Research: route marks, hiding places, and the signs between them

Research note, 2026-10-30. Brief, in the user's words: "Make navigation hints on levels more subtle and varied. Add secrets scattered around. Secret items should have slightly positive or negative lasting effect on player..." This note covers:
- the marks that show a way (M.'s, and the ones that are not hers);
- the places where things are hidden;
- how a hidden thing is hinted.

The secret items themselves, and their lore, are researched separately.

**What exists now** (read from the code on 2026-10-30):
- **Her trail** (`story/mod.rs`, `story/dress.rs`):
  - Each sheet is a 0.20 × 0.15 m cream plane 14 mm above the floor, with two creases and a pencil arrow along her walk.
  - Sheets lie at most 3 tiles of walk apart on levels 1–2, 4 on levels 3–5, and 5 from level 6 (`spacing`). A tile is 4 m (`CELL`), so that is 12, 16 and 20 m.
  - Each sheet lies where the last can be seen from, jittered up to 0.35 m.
  - From level 4 one stretch after her camp has no sheets; from level 7, two.
- **The false trails** (`false_sheet_parts`): a flat 0.21 × 0.28 m page with a heavier, longer black arrow. One trail from her page f6's floor, two from f7's.
- **Her camp** has chalk tallies. Her first page says: "Folded twice, so I know it's mine."
- **The exit** lights its tile and calls every 8–10 s.
- **Crying out and the payphone** (`spawn_guide_arrows`) lay a glowing arrow on every tile to the door, show SOMETHING SHOWS THE WAY, and darken the rooms for 8 s. That is the place speaking, not a person.
- **Verbs** include [E] OPEN DRAWER, REACH IN, PEER INTO THE SHAFT, KNOCK and TURN THE WHEEL.
- **Hide spots** (bed, wardrobe) exist only in the Showroom kit.
- **The cleaners' red marks** are on the Building's sealed doors.

**Method and limits.** Every claim links a page I opened, except those marked **(search)**, which rest on a search result's text because the page refused me. **[est]** marks my own estimate or arithmetic. **[unverified]** marks something I looked for and could not confirm.
- **Refused or timed out:** doomwiki.org (the secret counts come from classicdoom.com), Zelda Wiki, the FBI's Hanssen page, the NSS *Guide to Responsible Caving*, the Ohio and West Virginia mining codes, GDC Vault, Atlas Obscura. I found no NSS marking rule; the caving advice is a BLM cave-trip worksheet's.
- **Not confirmed:** a black guide line on the Paris catacombs' ceiling (one travel page), Penkovsky's lamppost signal, any "shoe pointing the way" convention, any developer statement on Hollow Knight's breakable walls (forums only), the 1998 *Thief*'s readables, and air vents in the *Deus Ex* postmortem (it has none).
- **Doubted by their own sources:** hobo signs and Native American trail trees.
- **Licences:**
  - Real marking conventions are practices, not works: use them freely.
  - FEMA's manual is a US government work, and Grimm and the myths are public domain.
  - Wikipedia (CC BY‑SA 4.0) was used for facts, not text.
  - Every game named is copyrighted, so take general motifs only. Do not copy Half-Life's lambda, the Souls message templates, or Mirror's Edge's red.

---

## 1. Real route marks

### 1.1 Marks people leave

| Mark | Looks like, size | Where | Direction | Subtlety, notes | Source |
|---|---|---|---|---|---|
| **Paint blaze** (Appalachian Trail) | White rectangle 2 × 6 in (51 × 152 mm), "a dollar" in size | About eye height, generally right of the path, on trees 6 in or more across | None: "Blazes are rectangles, not arrows" | Every 5 minutes of walk (244–305 m) where the path is obvious. **No more than one visible at a time**, except at junctions. "Too many blazes blight the primitive nature of the Trail" | [ATC, Marking the Trail](https://appalachiantrail.org/wp-content/uploads/2022/11/Marking-the-Trail-Student-w.pdf) |
| **Double blaze** | Two blazes 2 in apart | Before a turn or confusing junction | **Upper blaze offset toward the turn**; stacked in line means "pay attention" | "When used too frequently they become meaningless". A reassurance blaze goes 50–100 ft past a junction | same |
| **Cairn** | A stack of stones | Bare rock, fog, snow: where there are no trees | The line of cairns | The ATC dismantles visitors' cairns; the US NPS asks people not to build or add to them | [ATC](https://appalachiantrail.org/wp-content/uploads/2022/11/Marking-the-Trail-Student-w.pdf), [Wikipedia](https://en.wikipedia.org/wiki/Cairn) |
| **Scout signs** | Sticks, leaves, stones | On the ground | "This way": a large stone, a small one on top, **another small stone in the direction of travel**; or knotted grass bent that way | A cross: not this way, or danger. A split arrow with a tally of how many went each way. A box with an arrow and a number: "**message N paces this way**". A circle with a centre mark: "gone home". The last follower removes the signs | [Scouts Malta, B01](https://scouts.mt/wp-content/uploads/SK_Downloads/Training/Cubs/Bronze_Arrow/B01_Tracking_signs.pdf) |
| **Bent "trail trees"** | A sapling bent horizontal against a yoke stick, tied with hide | By paths, water, crossings | The bend | Contested; one survey group rejects most trees reported to it | [UNC Charlotte](https://ui.charlotte.edu/story/mystery-trail-trees/) |
| **Soot marks** | A carbide flame's sooty burn | Cave walls at survey stations; a mine tunnel full of carbide graffiti | Writing, points | Cataphiles in Paris still write on walls with the flame | [Historic Graffiti](https://www.historicgraffiti.org/underground), [Loupiote](https://www.loupiote.com/photos/urban-caver-olrik-writing-on-concrete-wall-of-utility-tunnel-with-carbide-lamp-paris-2102082584.shtml) |
| **Cave-trip tape** | Reflective or engineer's tape | "At frequent intervals" | Sequence | "Always remove your trail markers on the way out"; "Continually look back and establish landmarks" | [BLM worksheet](https://www.ntc.blm.gov/krc/system/files/legacy/uploads/24777/Risk%20Managemen%20Worksheet.pdf) |
| **Line arrows** (cave diving) | Notched acute triangle on the guideline | The permanent line | **Toward the nearest exit**, read by touch in a silt-out; two pointing apart mark the halfway point | Personal "cookies" carry no direction but initials, drawings or notches felt by hand. A REM points the way its owner came; others must not follow it | [Wikipedia](https://en.wikipedia.org/wiki/Line_marker), [TDI](https://www.tdisdi.com/tdi-diver-news/cave-diving-directional-and-non-directional-markers-101/?lang=pt-br) |
| **Fire boss's chalk** (coal mines) | Initials and the day of the month | At the face of each place examined | None: "checked, this day" | Required by state law | [Ohio R.C. 1563.28](https://codes.ohio.gov/ohio-revised-code/section-1563.28) (search) |
| **Paris quarry marks** | Worksite number, inspector's initials and year cut in the stone; street-name plaques | Quarry walls | None | — | [quarry plan](https://www.gancarczyk.com/wp-content/uploads/2022/12/katakumby-plan.pdf) (search) |
| **FEMA X-code** | One slash while searching; an X when done | Outside the building | Top: time and date. Left: team. Bottom: results and victims. Right: notes. X in a square: do not enter | Victim spots get International Orange paint or orange tape; a live victim, red tape | [Wikipedia](https://en.wikipedia.org/wiki/Urban_search_and_rescue), [FEMA App. A](https://fema.gov/pdf/emergency/usr/appen_a.pdf) |
| **Shelter arrows** (Berlin, WWII) | White arrows, some phosphorescent for the blackout, some lettered LSR; white strips **about 30 cm high at shoulder height** on corners | House walls | A 1944 decree: from about first-floor height, pointing straight down to the cellar | For rescuers after a raid; survive on clinker and stone | [Berliner Mieterverein](https://www.berliner-mieterverein.de/magazin/online/mm0515/051522.htm) |
| **Hobo signs** | Chalk: a cat for "a kind lady lives here", a zigzag for a barking dog | "Prominent or relevant places" | — | Doubted; hobos of the day called them "a fabrication". Monikers on water tanks (name, date, direction) were real | [Wikipedia](https://en.wikipedia.org/wiki/Hobo_signs) |
| **Kilroy** | A face peeking over a wall | Walls, ships, sealed spaces | — | Troops "always found notices chalked up ahead of them" | [Wikipedia](https://en.wikipedia.org/wiki/Kilroy_was_here) |
| **Hansel's pebbles** | White pebbles, later crumbs | Dropped one by one while he pretends to look back at "my little white cat" | Sequence home | Work only by moonlight, when they "glittered like real silver pennies"; birds eat the crumbs | [Grimm, tr. Taylor/Edwardes](https://etc.usf.edu/lit2go/175/grimms-fairy-tales/3077/hansel-and-gretel/) |
| **Ariadne's clew** | A ball of thread unwound going in | Continuous | Back out | Gave English "clue" | [Word Histories](https://wordhistories.net/2017/05/15/clue/) |

### 1.2 What the real marks agree on

- **Space them by sight.** The next mark is visible from the last, never more than one at a time. Add marks at junctions and a reassurance mark just past a choice ([ATC](https://appalachiantrail.org/wp-content/uploads/2022/11/Marking-the-Trail-Student-w.pdf)). Our "each sheet seen from the last" is this rule.
- **Few marks carry direction.** A blaze takes its direction from the path and from doubling at turns. Where direction is drawn it is an arrow, or a piece set off to one side (the third stone, bent grass).
- **Direction is the maker's convention.** Line arrows point back to air; LSR arrows point down for rescuers; a REM points where its owner came from. Her first page says she leaves paper "so I can find my way back", yet her arrows point on; a caver would point them back. The lore-keeper can keep this as her habit or use it.
- **People mark so they know their own:** initials and notches on a diver's cookie ([Wikipedia](https://en.wikipedia.org/wiki/Line_marker)), "folded twice, so I know it's mine".
- **The material follows surface and light:** paint on bark, stones above the trees, soot and tape underground, chalk on walls, phosphorescent paint in a blackout, pebbles by moonlight.
- **Marks disappear.** The last follower removes them, cavers take their tape out, renovators sandblast, birds eat crumbs. A trail with gaps reads as real.
- **Too many marks is a fault.** Over-marking empties a sign of meaning ([ATC](https://appalachiantrail.org/wp-content/uploads/2022/11/Marking-the-Trail-Student-w.pdf)), and stray cairns are removed because they mislead ([Wikipedia](https://en.wikipedia.org/wiki/Cairn)). That is a model for the flat sheets.

---

## 2. How games guide without arrows

### 2.1 Landmarks, light, paint

| Game / idea | Device | For us | Source |
|---|---|---|---|
| Disney's "wienie" | A landmark seen from afar that people walk to unasked; named for the hot dogs Walt's dog followed | The exit's light and call are ours | [Touring Plans](https://touringplans.com/blog/disney-in-a-minute-what-is-a-wienie/) |
| *Journey* | A mountain seen from the first dune; no words outside title and credits | Show the goal, leave the route | [Wikipedia](https://en.wikipedia.org/wiki/Journey_(2012_video_game)) |
| *Half-Life 2* | A door frame points at the Citadel; birds fly up toward it | Framing and motion draw the eye | [Level Design Book](https://book.leveldesignbook.com/process/blockout/wayfinding) |
| Level Design Book's scale | Guessed reliability: pickup breadcrumbs and distant "weenies" 80%, environmental trails 50%, light and in-world signs 40% ("easily mistaken for irrelevant set dressing"), typology 20%, sound 15%, unusual details 10%. **Start subtle; add aids where playtests show people lost** | The author's estimates, not measurements. Our trail sits between "environmental trail" and "breadcrumbs" | same |
| Yellow paint | Paint on ladders and crates (*RE4* remake, 2023; *FF7 Rebirth*), called immersion-breaking and defended because players get stuck; "an earlier failure in the art design" | A mark the fiction did not make breaks the place; ours must be someone's, in what she carried | [Wikipedia](https://en.wikipedia.org/wiki/Yellow_paint_debate) |
| *Mirror's Edge* | Usable objects turn red: Faith "sees objects in the world as aids" | A guide with a reason in the fiction is accepted | [Game Developer](https://www.gamedeveloper.com/design/the-philosophy-of-faith-a-mirror-s-edge-interview) |
| *Shadow of the Colossus* | The sword's beam points, only in sunlight, and you must stop to aim it; the game "wants you to get a little lost" | A guide that works in some light, at a cost | [Game Developer](https://www.gamedeveloper.com/design/navigation-in-shadow-of-the-colossus) |
| *Outer Wilds* | Hunting a sound "works no matter where you are"; playtesters aimed at nearby rocks and blamed them, so a distance readout was added | Sound leads, and is misattributed: a risk for our door's call | [Mobius Digital](https://www.mobiusdigitalgames.com/news/separating-the-signal-from-the-noise) |
| *Inside* | No text; a rope lit through a hole in the roof; a bird flies off to catch the eye; a mechanic met calm before tense | A calm first meeting teaches | [Game Developer](https://www.gamedeveloper.com/design/inside-teaching-through-level-design) |
| *Outlast* | The camcorder's night vision is the only sight in the dark and eats scarce batteries | Seeing costs; our lamp already does | [Wikipedia](https://en.wikipedia.org/wiki/Outlast) |
| Harvey Smith, *Deus Ex* | "Whenever possible, show players a goal or destination before they can get there" | Also the first rule of secrets (§3.2) | [Game Developer](https://www.gamedeveloper.com/design/postmortem-ion-storm-s-i-deus-ex-i-) |

### 2.2 Hints from someone who came before

| Game | What the previous person left | Reads as a person because | Source |
|---|---|---|---|
| *Left 4 Dead* | Safe-room graffiti written by Valve staff in markers; a script book of lines people rewrote "in their own style"; "lightly supervised chaos" | Many hands and tones, arguments; almost no gameplay tips | [Aftermath](https://aftermath.site/left-4-dead-graffiti-valve-interview/) |
| *Demon's Souls* | Pre-written floor messages; bloodstains replaying a death. Inspired by cars pushing each other up a snowy hill: "a sense of silent cooperation" | Fixed wording, but a person's placement; some messages are traps ([Push Square](https://www.pushsquare.com/guides/demons-souls-how-does-multiplayer-and-online-play-work), search) | [Wikipedia](https://en.wikipedia.org/wiki/Demon%27s_Souls) |
| *Death Stranding* | Others' structures and messages appear and decay | Wear shows time passed | [Wikipedia](https://en.wikipedia.org/wiki/Death_Stranding) |
| *Half-Life 2* | The Resistance's lambda near caches: walls, **above vents, under pipes, beside ladders**; some caches unmarked | A faction's sign, inconsistently used | [Combine OverWiki](https://combineoverwiki.net/wiki/Lambda_Logo) |
| *Tunic* | Manual pages in the world, with notes by "the mysterious, previous owner" that give secrets away | A second hand on a printed page | [Superjump](https://superjumpmagazine.com/tunic-hides-its-secrets-in-plain-sight) |
| *Amnesia* | A note Daniel wrote to himself sets the goal; notes and diaries carry the way on | The guide is someone's intention | [Wikipedia](https://en.wikipedia.org/wiki/Amnesia:_The_Dark_Descent) |

From these, a mark reads as a person's when it has:
1. **Varied hand and medium.** Not one asset everywhere.
2. **Inconsistency.** Some things unmarked, uneven spacing.
3. **Wear.** Smudged, curled, taken away.
4. **Fallibility.** A person can be wrong.
5. **An owner's quirk** repeated in every kind of mark.
6. **A reason in the fiction.** Runner vision has one, yellow paint none.

### 2.3 How faint a mark can be, and how it was tested

- **Eyes pass things without seeing them.** In an eye-tracking test in *Unreal Tournament*, players' eyes crossed an enemy in the open with "a significant time lapse" before they recognised it. Better lighting cut that scan time ([El-Nasr & Rao, 2004](https://history.siggraph.org/wp-content/uploads/2023/01/2004-Poster-082-El-Nasr_Visually-Directing-Users-Attention-in-Interactive-3D-Environments.pdf)). Being on screen is not enough; a mark needs contrast, a lit place, or motion near it.
- **Most players miss optional marks.** *Half-Life 2*'s lambdas are painted on walls, yet 2.6% of its Steam players found all 45 caches ([Guinness, 2018](http://www.guinnessworldrecords.com/world-records/548923-rarest-achievement-on-half-life-2)). The base counts players who never finished **[est]**.
- **Plain sight hides.** *The Witness* hides paths in its scenery that join only from one viewpoint ([Wikipedia](https://en.wikipedia.org/wiki/The_Witness_(2016_video_game))).
- **How it was tested:**
  - *Outer Wilds* watched unhelped players and changed the tool when they misread it ([Mobius](https://www.mobiusdigitalgames.com/news/separating-the-signal-from-the-noise)).
  - El-Nasr timed first fixation to recognition.
  - The Level Design Book adds stronger aids only where playtests show people lost.
- **For us:** a mark is faint enough when a player looking for marks finds it and a player not looking takes it for litter. Deep down her marks may drop below that line for the unwary, since by then players have learned to look.

---

## 3. Secrets in level design

### 3.1 How games hide and hint them

| Game | Where | The tell | Source |
|---|---|---|---|
| *Doom* | Doors wearing wall textures | A texture unlike its surroundings ("a very old Doom trope"). Romero's rules ask for "several secret areas on every level" | [eev.ee](https://eev.ee/blog/2015/12/30/you-should-make-a-doom-level-part-2/) |
| *Quake* E1M1 | Six: a striped panel to shoot (a flashing light, "you can jump up here"), planet screens, a crosshair button seen from a bridge, a tunnel under slime | Things that look usable and are set apart | [Quake Wiki](https://quakewiki.org/wiki/E1M1) |
| *Rayman Legends* | Behind foreground objects, past the camera's edge, breakable walls | "just enough hints to show that something's there": animated enemies, light, captives crying for help. Early levels teach the rules; a level that broke them is criticised | [Game Developer](https://www.gamedeveloper.com/design/secrets-design-in-rayman-legends) |
| *Link's Awakening* | Bombable walls | **Sound**: "If you hear a hollow clank, you can break the wall with a bomb!" | [LP Archive](https://lparchive.org/Legend-of-Zelda-Links-Awakening-DX/Update%20XX/) |
| *Dishonored* | Runes, bone charms | A held Heart that points at them; the team weighed "how obscure or overt" to make it | [GameBanshee](https://gamebanshee.com/xxf) (search) |
| *Tunic* | Everywhere, in plain view | A previous owner's handwriting | [Superjump](https://superjumpmagazine.com/tunic-hides-its-secrets-in-plain-sight) |
| *Animal Well*, *Fez* | Layered puzzles; one needs fifty players' partial answers | Players working together solved *Animal Well*'s within a month and *Fez*'s monolith within days | [Wikipedia](https://en.wikipedia.org/wiki/Animal_Well), [Engadget](https://www.engadget.com/2012-04-20-how-the-community-solved-fezs-hardest-puzzle.html) |

### 3.2 Rules designers state

1. **Notice before you find:**
   - "show players a goal or destination before they can get there" ([Smith](https://www.gamedeveloper.com/design/postmortem-ion-storm-s-i-deus-ex-i-));
   - "just enough hints to show that something's there" ([Rayman](https://www.gamedeveloper.com/design/secrets-design-in-rayman-legends)).
2. **Teach the tell early and keep it.** Breaking the taught rules is a fault ([Rayman](https://www.gamedeveloper.com/design/secrets-design-in-rayman-legends)).
3. **Keep secrets off the critical path, but not off the map.** Nudge players toward optional areas ([eev.ee](https://eev.ee/blog/2015/12/30/you-should-make-a-doom-level-part-2/)). Dead ends should pay ([MEGASTRUCTURE.md](../MEGASTRUCTURE.md)).
4. **Contrast required, optional and secret space** ([Romero via eev.ee](https://eev.ee/blog/2015/12/30/you-should-make-a-doom-level-part-2/)).
5. **Reward in proportion to effort.** *Quake*'s secrets hold shells, armour, health or Quad Damage ([Quake Wiki](https://quakewiki.org/wiki/E1M1)). No page I opened states the rule outright **[unverified]**.
6. **If players start pressing every wall, adjust.** Either the tell is too weak or the reward too strong **[est]**.

**Fair hiding places in a realistic building** are ones that exist in real buildings and show a tell (§4):
- overhead and in the walls: a loose ceiling tile, a vent, a drywall hole, a false back;
- in fittings: a cistern, a fuse box, a locker;
- below and behind: under a bed, behind a vending machine, a crawl hatch;
- in furniture: in a drawer, or taped under one.

### 3.3 Sound as the tell, and secrets that travel

- **Sound tells:**
  - the hollow clank (above);
  - Rayman's captives crying;
  - Outer Wilds' signals.
- **The real version is rescue hailing:** searchers ask buried people to "knock five times", then listen in silence ([FEMA App. A](https://fema.gov/pdf/emergency/usr/appen_a.pdf)). We already have [E] KNOCK on sealed doors.
- **Word of mouth:** *Zelda*'s complexity "encouraged gamers to speak with one another to share tips and tricks", which led to the Nintendo Fun Club newsletter ([Museum of Play](https://www.museumofplay.org/?p=2319)).
- **What travels in a seeded game is a rule, not a place.** A position does not travel between seeds; "a chair under a ceiling tile means look up" does **[est]**.

### 3.4 How many per level

| Game | Secrets per level | Source |
|---|---|---|
| *Doom* (1993) E1–E3, maps with a tally | 1–12; **mean about 5, median 4** **[est]** | [classicdoom.com](https://classicdoom.com/maxsec.htm) |
| *Doom II*, 32 maps | 0–18; **mean about 4.4**; four maps with none **[est]** | same |
| *Quake* E1M1 | 6 | [Quake Wiki](https://quakewiki.org/wiki/E1M1) |
| *Half-Life 2* | 45 caches, **about 3 a chapter** over 14 chapters **[est]** | [Guinness](http://www.guinnessworldrecords.com/world-records/548923-rarest-achievement-on-half-life-2) |
| *Rayman Legends* | 10 or 3 collectibles a level, about half in secret areas | [Game Developer](https://www.gamedeveloper.com/design/secrets-design-in-rayman-legends) |

---

## 4. Real hiding places, and the signals that point at them

### 4.1 What people hide, and where

| Place | Evidence | Source |
|---|---|---|
| Ceiling tile | A restoration crew found over $1,000 in a basement ceiling tile. Drop-in tiles lift out (600 × 600 mm in Europe, 2 × 2 or 2 × 4 ft in the US) onto pipes, wiring and ducts | [Yahoo](https://www.yahoo.com/news/articles/kentucky-mold-clean-team-finds-121724333.html), [Wikipedia](https://en.wikipedia.org/wiki/Dropped_ceiling) |
| In the wall behind a loose toilet | A plumber at Lakewood Church found about 500 envelopes of cash and cheques (2021) | [CBS](https://www.cbsnews.com/amp/texas/news/plumber-money-behind-toilet-joel-osteen-lakewood-mega-church) |
| **Toilet cistern**, books, under carpets, pickle jars, socks, upholstery | Where Soviet households kept cash: "в сливные бачки унитазов", "засовывали их в книги" | [Mentoday](https://www.mentoday.ru/life/experience/gde-sovetskie-lyudi-pryatali-dengi-i-v-chem-hranili-sberejeni) |
| Mattress, dresser and office drawers, vases, freezer | Where burglars look first | [AAA](https://cluballiance.aaa.com/the-extra-mile/advice/home/hiding-spots-where-burglars-always-look-first) |
| Hollow books, fake cans and outlets, picture frames | "Diversion safes"; hollow books among the commonest | [Wikipedia](https://en.wikipedia.org/wiki/Diversion_safe) |
| Antresol and storeroom | Soviet flats had cupboards "под потолком" for things used once a season | [T—Ж](https://t-j.ru/7-sovetskikh-reshenii-v-interere/) |
| Refuse-chute hatch | Loading hatches belong on stair landings by the norms | [Normacs](https://www.normacs.info/answers/1044) (search) |

### 4.2 Dead drops and their signals

- **How a drop works.** Two people pass things through a place agreed in advance. **A separate signal says it is loaded**: a chalk mark on a wall, gum on a lamppost, a towel on a balcony ([Wikipedia](https://en.wikipedia.org/wiki/Dead_drop)).
- **Ames** drew a short horizontal chalk line above the USPS logo on a mailbox. The mailbox was the signal, not the drop (same).
- **Hanssen** stuck a vertical strip of white medical tape on a park's wooden sign: the drop across the road, "a dark, damp place under a footbridge", was loaded ([Wikipedia](https://en.wikipedia.org/wiki/Foxstone_Park)). The answer was a horizontal strip, the same mark turned (search).
- **Penkovsky**'s Moscow drop was **behind a radiator in the foyer of a block of flats** on Pushkinskaya Street ([Daily Beast](https://thedailybeast.com/the-spy-who-saved-the-worldthen-tried-to-destroy-it), search).
- **The pattern:**
  - The signal is small, public and **not on the hiding place**.
  - It says "loaded", not "where".
  - A changed form of the same mark (turned, crossed out) says "taken".
  - FEMA does the same: a slash for "searching", an X for "done", and marks crossed out when facts change ([Wikipedia](https://en.wikipedia.org/wiki/Urban_search_and_rescue)).

Three strengths of hint follow:
- **near here**: the drop signal, the hobo cat's "this house";
- **how far, which way**: the scout box, "message N paces this way";
- **already taken**: crossed out or turned.

---

## 5. For this game

Every proposal is placed in the level build from its own salted stream (never `rnd()`), never reads the save, and fits what was taken afterwards, like the notes (invariant 1). Light stays place-based; only the flashlight follows the player, which STYLE.md allows.

### 5.1 Her marks: a vocabulary

**Her hand.** One habit runs through every kind of mark: an open arrowhead with **the left barb longer** **[est: our invention]**, in pencil grey or chalk white, never black, never filled. It is never explained.

The folded sheet stays her signature and the first thing met. The other kinds come in as her paper runs short, which tells the same story the depth already tells.

| # | Mark | Looks like | Where | Direction | From | Taught by | Model |
|---|---|---|---|---|---|---|---|
| 1 | **Folded sheet** (exists) | 0.20 × 0.15 m, creased, pencil arrow | Floor | Arrow | L1 | Her first page; first sheet in sight of arrival | Pebbles; personal cookie |
| 2 | **Jamb arrow** | 8–12 cm pencil arrow; faint sheen in the flashlight | Jamb or column corner on the side she took, **0.9–1.1 m up** **[est]** | Arrow | L2 | First one on the jamb above a sheet | Blaze on the right of the path |
| 3 | **Doubled mark** | Two 5 cm strokes, **upper offset toward the turn**; at most one mark in four | Only where her walk turns at a junction of 3+ ways | Offset | L3 | Always and only at turns | ATC double blaze |
| 4 | **Torn corner** | 5–7 cm triangle of her paper, creased | Under a door, in a gap, under a chair leg | The tip | L5 | Same paper and creases | Crumbs |
| 5 | **Skirting tick** | 6–10 cm chalk stroke leaning forward, 10–20 cm off the floor | The wall she kept her hand on (left, in mazes) | Lean | L5 | Her camp tallies are the same chalk | Fire boss; Kilroy |
| 6 | **Small stack** | A large piece, a small one on top, **a third beside it, onward**: concrete chips (Building), ceiling-tile shards (Lobby), bottle caps (offices) | Open floors far from walls: Pillar Field, Habitable Zone, Building decks | Third piece | L2 | First one beside a sheet | Scout stones; cairn |
| 7 | **Thread** | 30–50 cm of grey wool from her blanket, knotted | Door handle, pipe or chair leg at knee height | None: "this door" | L6 | First one on a handle above a sheet | Ariadne's clew; flagging |
| 8 | **Tape strip** | 2 × 5 cm; dull grey in room light, **a white glint when the flashlight finds it** | Jambs and pillars in **blackout zones only** | Sequence | L7 | Appears where a sheet would be lost in the dark | Cave tape; moonlit pebbles; phosphorescent arrows |
| 9 | **Counted mark** | Chalk "III" beside her arrow: "the third door" | Where a gap ends | Arrow and count | L8 | The count is right | Scout split arrow |

**With depth** **[est]**:

| Levels | Spacing (exists) | Mix | Wear |
|---|---|---|---|
| 1–3 | 3–4 tiles | Mostly sheets; stacks on decks; a jamb arrow or two | Fresh |
| 4–6 | 4–5 | Half sheets; jamb arrows, doubles, first torn corners and thread | Some curled, some smudged |
| 7–10 | 5 | A quarter sheets; chalk, corners, thread; tape in blackouts; counted marks at gap ends | Fainter, lower, shakier |
| 11–13 | 5, longer gaps | Few sheets; mostly chalk and corners; short frayed thread | The long left barb, unchanged |

**Rules that keep them hers:**
- One mark visible at a time, except at a turn. The existing sight check applies to every kind at that kind's own sight distance (§5.4).
- Never two kinds at one spot, except when a new kind first appears beside an old one. That pairing is the whole tutorial.
- From level 6, now and then a two-mark spur goes into a room and a crossed-out mark comes back. She went wrong and corrected herself **[est]**. That is fallibility, and unlike a false trail it returns.
- Her marks never glow or pulse. Glowing per-tile arrows belong to the cry for help.

**How the false trails stay different:**

| | Hers | Not hers |
|---|---|---|
| Medium | Pencil, chalk, thread, torn paper, tape | Heavy black, whole flat pages |
| Arrowhead | Open, left barb longer | Filled, **identical every time** |
| Spacing | Uneven, by sight | **Even**, as no tired person walks |
| Turns | Doubled mark first | No warning |
| Variety | Several kinds | One kind |
| Visibility | Faint, low, at the edges | Mid-floor and obvious: over-marking, like stray cairns |
| End | The door, or a gap that resumes | A dead end where something breathes |

A player who has learned her hand reads the false trail as wrong by its regularity, before reading any rule. Over-blazing does the same to a real trail, and yellow paint to a level.

**Teaching without a tutorial:**
1. The first sheet is in sight of the arrival (exists).
2. Each new kind of mark first appears beside a sheet.
3. Her notebook may mention one habit in passing, as *Tunic*'s previous owner does, never as an instruction ("Paper won't last. Chalk does." is the register; the lore-keeper writes it).
4. The flat sheets start only after f6, as now.

### 5.2 Hiding places

| # | Place | Zones | Tell, seen or heard first | Verb | Model |
|---|---|---|---|---|---|
| 1 | **Ceiling tile** | Lobby, offices | One tile askew with a dark edge; **a chair or desk under it** | Stand on it: [E] LIFT THE TILE | Kentucky tile; drop-in grid |
| 2 | **Vent grille** | Offices, utility halls | Two of four screws gone, scraped paint, dust wiped in an arc | Crouch: [E] PRISE THE GRILLE | Lambdas above vents (motif) |
| 3 | **Toilet cistern** | Washrooms | Lid off square; **the one cistern that does not hiss** | [E] LIFT THE LID | Soviet cash in cisterns |
| 4 | **Under the vending machine** | Where one stands | A glint under the kick plate when crouched in the flashlight | Crouch: [E] REACH UNDER | Must fit VENDING.md (lore-keeper) |
| 5 | **Under a drawer** | Offices | The drawer will not shut all the way | After OPEN DRAWER: [E] FEEL UNDERNEATH | Burglars' office drawers |
| 6 | **Locker** | Utility, electrical, maintenance | One door wired shut with a twist of wire | [E] UNTWIST THE WIRE (forcing is loud: Hounds) | — |
| 7 | **Fuse box** | Electrical station | Door on one hinge; **hums at another pitch** | [E] OPEN THE PANEL | — |
| 8 | **Wall patch or skirting** | Offices, living quarters | A square of fresher paint, or a board standing proud with bright nail holes: the real form of *Doom*'s odd texture | [E] PULL THE BOARD | *Doom* |
| 9 | **Radiator** | Building stair halls, cell fronts | A rag behind the fins; chalk on the wall beside it | [E] REACH BEHIND | **Penkovsky's radiator** |
| 10 | **Chute hatch** | Building landings | String tied to the handle, running inside | [E] OPEN THE HATCH: a bundle on the string | Hatches on landings |
| 11 | **Antresol** | Building cells, living quarters | A small door under the ceiling ajar | Stand on the bench: [E] REACH UP | Soviet antresol |
| 12 | **Wall tile** | Poolrooms, washrooms | Darker grout round one tile; **a hollow knock** | [E] KNOCK, then [E] PRISE THE TILE | Hollow clank; FEMA hailing |

Spares, if a zone needs one:
- a wardrobe shallower inside than out (a false back);
- the mattress of a bed;
- a hollow book on an office shelf;
- a crawl hatch under a Building stair, with a draught through it.

**Rules:**
- **Tells are consistent across the run.** A chair under a tile always means "up". A tell never appears without a cache, except at the taught empty ones (§5.3). The same props without the tell are ordinary dressing.
- **Placement.** Never on her walk and never at a false trail's end. Put them one or two rooms off her walk, or at ordinary dead ends, so that dead ends pay.
- **Searching costs exposure, not tedium.** Each search is 1–2 s of crouching or standing on furniture **[est]**, and forcing a cache is loud.

### 5.3 The sign that something is put away

One small chalk sign in a third hand. Who makes it is never said; the lore-keeper decides whether it may be E.'s. It works like the dead-drop signal.

- **Look:**
  - a chalk circle about 6 cm across with one stroke through it;
  - 1.4–1.6 m up on a jamb or a pillar corner, facing into the room **[est]**;
  - round, not an arrow, with none of her habits.
- **Meaning:** "something is put away **in this room**". It never sits on the hiding place and never points; the tell (§5.2) says where.
- **"Taken" form:** the circle crossed out. The first one a player meets is crossed, beside an opened, empty hiding place: a lifted tile with nothing above it. That teaches the sign safely, the way *Inside* meets a hazard calm before tense.
- **With depth:** up to level 6 the sign is at the room's door and the tell is plain. Deeper, it may stand in a corridor outside two or three rooms ("near here") **[est]**.
- **About half the caches carry no sign** and are found by their tell alone, as with *Half-Life 2*'s unmarked caches.
- **The scout box** ("message N paces this way") could be her one sign that points at a secret, something she left for E. Use it only if the secret-items lore gives her a reason.
- **In blackouts the tell is a sound:** a silent cistern, a hollow tile, an odd hum. It comes from the prop, starts silent and plays only within earshot (house rules).

### 5.4 Counts and tests

**Counts per level** **[est]**:

| | Building (1–3) | Lobby (4) | Rooms (5+) |
|---|---|---|---|
| Caches with something in them | 2–3 (cells, landings, radiators, chutes) | 0–1 (emptiness is the content) | 2–4 |
| Of which signed | 1 | 0–1 | 1–2 |
| Crossed signs at empty caches | 1 on level 1 or 2 (the lesson) | 0 | 0–1 |
| Kinds of her mark on a level | 1–2 | 2 | 3–4 (none below 13) |

That is below *Doom*'s mean of about 5 and near *Half-Life 2*'s three a chapter, because each search here costs time, light and safety.

**Tests:**
1. **Determinism.** New salts, and her trail's existing draws unchanged: reference levels differ only by the new objects. Nothing reads the save.
2. **Sight by kind.** Every mark is visible from the previous one at its own kind's distance. A sheet as now; a jamb arrow up to about 8 m; a skirting tick about 6 m; a tape strip anywhere in the flashlight's reach **[est]**. Make these constants, each checked by a headless view.
3. **Size on screen.** Headless views of each kind at 4, 8 and 12 m in its zone's light. Record pixel area and contrast against the surface as bands, like STYLE's: a floor ("a looking player sees it") and a ceiling ("an unwary one takes it for litter").
4. **Personas:**
   - a *reader* bot that follows only her hand reaches the door on every level kind;
   - a *credulous* bot that follows any mark reaches a false dead end at the expected rate;
   - a *searcher* bot that visits every tell finds every cache, and no prop without a tell holds one.
5. **Separation.** No mark of hers lies on a false trail, and none of theirs on her walk. No cache lies on her walk or at a false trail's end. Each sign is within one room of its cache.
6. **Playtest questions** for PLAYTEST.md:
   - When did they first notice a mark that was not a sheet, and did they read it as hers?
   - Did they tell the flat sheets from hers, and by what?
   - Did they find a cache with a sign? Without one? Did they understand the crossed circle?
   - Did anyone start lifting every tile? That means the tell is too weak or the reward too strong.
   - Search time per cache, and a heatmap of searches from the recordings' telemetry.
