# Interactive props: what exists, the rules, and what to add

Research for [`../design/PROPS.md`](../design/PROPS.md) (2026-10-31), on the request "make more props interactive in some way without breaking the lore": every interaction the game has, the rules any new one must keep (each with its source line), which common props have none, and fifteen proposals ranked. Measured on 54 floors (`--dump-level`, levels 4–12 of six seeds) and 27 census floors; the dumps were working files and are not kept; the code is cited by file and line as it was at `9df0d3b`.

## 0. Summary

- The game has **about 26 verbs per level** (mean 25.9, range 10–39 over the 54 dumped levels), from 20
  `interact(` call sites. Nearly all of them are on **core props** and on systems: drawers, light switches,
  the payphone, the vacuum, elevators, the door, her page, secrets, vending, blinds. **No themed placer
  except `place_gameplay_r1_trigger` has a verb**, out of 649.
- Interactions come in two styles:
  - **The house style** is used by the newer systems: the vending machine, the blinds, the secrets and the
    Building's cells. Each has rules rather than dice, keeps its effects unsaid, plays its sounds from the
    prop with `play_from` only within earshot, charges noise that creatures hear, and draws no
    randomness from the build's stream.
  - **Legacy interactions** (older than these rules) break several of those rules: the vacuum, the
    payphone's roll, the shaft peer, the "(bad wiring)" switch, candle relighting, the R1 button and the
    safe-room toast. They are listed in §1.3. Fixing them is the cheapest way to stop the lore breaking.
- The **common, non-interactive props are the core props**. They are on 22–27 of 27 levels, often 2–4 per
  level: cardboard stacks, lockers, the CRT TV, the dead elevator, book stacks, chairs, trash cans, coat
  racks, extinguishers, wheelchairs. Themed placers are individually rare: the most common one with a
  plausible verb, wall vents, is on 32 of 54 levels, and most are on 3–10 of 54.
- **Top proposals:**
  1. `OPEN THE BOX` on cardboard stacks.
  2. `OPEN THE LOCKER` on lockers whose padlock hangs open.
  3. `TURN IT OFF / ON` on the CRT TV.
  4. `PRESS THE BUTTON` on the dead elevator (a car passes and never stops).
  5. `TEAR OUT PAGES` from book stacks.
  6. `SIT DOWN` / `SET IT UPRIGHT` on chairs.
  7. `LOOK INSIDE` trash cans.

  These are ranked in §4, with 8 more and a "do not" list.

---

## 1. Every interaction the game has now

### 1.1 Verbs (E / left click on what you face)

`Game::interact(pos, r, prompt, enabled, on_use)` is defined at `crates/game/src/base/game.rs:1229-1238`, and
the `Interactable` record at `crates/game/src/base/state.rs:12-29`. Each frame the HUD picks the faced target
(`interact_target`, `crates/game/src/ui/hud.rs:1637-1660`: the one you face most squarely, ties to the
nearer; one within 0.3 m counts whichever way you face). `try_interact` runs it (`hud.rs:1600-1632`). The
binding is `Act::Interact` = `KeyE`, `Mouse0` (`crates/game/src/session/input.rs:62`). Prompts are written
`[E] VERB …` and painted in sentence case ("E  Open the door", `crates/game/src/ui/rest.rs:197`).

"Levels" means the 27-level census or the 54-level dump.

| # | Prop | Prompt | What it does | Where (file:line) | How often |
|---|---|---|---|---|---|
| 1 | Exit door | `[E] ENTER THE DOOR` / `[E] OPEN THE STEEL DOOR` (Stairwell's) | Next level / into the Stairwell (`use_exit_door`) | `core_props/mod.rs:683-689` | every rooms level |
| 2 | Filing cabinet drawers (3 per cabinet) | `[E] OPEN DRAWER` | Slides out 0.36 m; 28% a battery, 22% almond water, 18% a paper stack, 32% "EMPTY." (`builders.rs:142-152`); or a secret's thing taped under it | `core_props/mod.rs:322-409` (interact 345) | 3 cabinets/level, 27/27 (so about 9 drawers/level) |
| 3 | Light switch | `[E] LIGHT SWITCH` / `[E] LIGHT SWITCH  (bad wiring)` | Toggles the 4 nearest panels (light map: `switched_off`); 30% faulty ones flip by themselves when you come near ("THE SWITCH FLIPPED ON ITS OWN") | `core_props/tail.rs:135-229`, motion `player.rs:1428+` | 1–2/level (1.5 avg), 27/27 |
| 4 | Payphone | `[E] ANSWER` (while ringing; rings within 9 m, `hud.rs:1557`) | Roll: 40% whisper −15 nerve, 35% "where to look" (the lights / prints / door's call), 25% a scream that sets every creature to hunt, −25 nerve | `core_props/mod.rs:1027-1107` | 1/level, 27/27 |
| 5 | Robot vacuum | `[E] TAP THE VACUUM` | Roll: 60% "THE VACUUM LIKES YOU" (follows 15 s), 30% "NOTICED YOU" (chases 6 s), 10% "YOU KICKED IT — SOMETHING HEARD" (creatures within 20 m set to hunt) | `core_props/tail.rs:806-857` | 1/level, 27/27 |
| 6 | Broken elevator | `[E] PEER INTO THE SHAFT` | Once: chime, breath, shake, −12 nerve, one of "SOMETHING WAS DOWN THERE." / "IT LOOKED BACK." / … | `core_props/tail.rs:1164-1191` | 55% of levels (`tail.rs:980`); 17/27 |
| 7 | Working elevator | `[E] CALL THE ELEVATOR` | Next level at once (`next_level_via_elevator`, `session/flow.rs:370+`) | `core_props/tail.rs:1327-1346` (interact 1343) | the first of 1–2, so every level (27/27) |
| 8 | Haunted candles | `[E] BLOW OUT` / `[E] RELIGHT (3% battery)` | Out / relit for 3% of the lamp's battery (15% fizzle) | `core_props/weather_fx.rs:325-405` (391, 394) | "haunted" weather only, 2–4 candles |
| 9 | R1 wall button | `[E] TOGGLE SWITCH` | Red/green button; lights one far ceiling panel; a non-positional blip | `crates/placers/placers_p00/src/lib.rs:18-141` (interact 125) | **every level** (`PLACER_ALWAYS`, `world/tables.rs:1417`) |
| 10 | Vending machine | `[E] COIN RETURN` / `[E] INSERT COIN` / `[E] ROCK IT` / `[E] REACH IN` | Rules 3–9 of `docs/design/VENDING.md` (coin, silence, shove, fall, dead ones) | `core_props/vending/rules.rs:140-185`, `vending_use` 185+ | 18/27 levels (none on L4, Showroom or Borrowed House) |
| 11 | Window blind | `[E] LOWER THE BLIND` / `[E] RAISE THE BLIND` | Covers or uncovers the glass (the one behind the glass); its rattle is heard 10 m along the way | `core_props/window/blind.rs:21-28`, `blind_use` 49-60 | windows below the Lobby where a home allows |
| 12 | Secret hiding places | `[E] LIFT THE TILE` / `[E] PRISE THE GRILLE` / `[E] REACH BEHIND` / `[E] REACH IN` | Opens the place: what any floor holds, nothing, a thing, or the hole's exchange | `secrets/dress.rs:276-301`, `open_secret` 306-350 | 2–4/floor (0–1 on the Lobby); 38/54 levels |
| 13 | Things put away | `[E] EAT THEM` / `TAKE A SWEET` / `PUT IT ON` / `TAKE IT` / `EAT IT` / `DRINK IT` / `PICK IT UP` | Takes the thing; a lasting, unsaid effect (`secrets/keeps.rs`) | `secrets/dress.rs:298-300`, verbs 538-549, `take_secret` 352+ | 1–2 a floor on the first descent |
| 14 | Her page | `[E] TAKE HER PAGE` | Notebook page; her lamp gutters; wakes the place | `story/mod.rs:436-442` | one camp a floor, levels 1–13 |
| 15 | Building cell | `[E] TURN THE WHEEL` / `[E] OPEN THE DOOR` | Shuts or opens the hermetic door (the self-assembly) | `world/building_dress.rs:241-256` | levels 1–3 |
| 16 | Building sealed doors | `[E] KNOCK` | Three knocks; 8% something knocks back (`native_rnd`) | `world/building_dress.rs:414-431` | levels 1–3 |
| 17 | Borrowed House nest | `[E] PUT OUT THE LAMP` | One lamp out; tells the shadows where you are | `anomalies/house.rs:141-147` | the House only |
| 18 | Stairwell / Loop doors | `[E] OPEN THE DOOR` / `[E] OPEN THE EXIT DOOR` | Exit (prompt set directly, not an `Interactable`) | `modes/mod.rs:548, 871`; `ui/hud.rs:1604-1615` | those modes |

### 1.2 Proximity and step triggers (no verb)

| Prop | Trigger | Where | How often |
|---|---|---|---|
| Almond water / batteries on the floor | Walk within ~0.97 m: picked up, toast from `almondPickup` / `batteryPickup` (`ui/hud.rs:83-91`) | `player.rs:1029-1062`; placed `core_props/mod.rs:694-735` (3 + 3 a level), they bob and spin (`tail.rs:1488-1505`) | every level |
| Paper piles | Walk over: +3–8 sheets, "PAPER STACK — n SHEETS" | `player.rs:1073-1087`; `tail.rs:232-251` | 0–2/level (20/27) |
| Torn notes | Walk over: read into the notebook | `player.rs:1321+`; `tail.rs:301-354` | 2–4/level + an anomaly's own |
| Floor hole | Step in: fall a level | `player.rs:1064-1071`; `tail.rs:1350-1361` | 35% of levels (9/27) |
| Safe room | Stand in: nerve ×3.2 regen, toast "SAFE ROOM — SANITY MENDS HERE" | `player.rs:1096-1130`; `tail.rs:861-960` | every level |
| Faulty light switch | Come within 2 m: flickers, then flips | `player.rs:1428+` | 30% of switches |
| Payphone | Within 9 m: rings | `ui/hud.rs:1553-1566` | 1/level |
| Mannequin | Unseen within ~14 m: turns to you and steps 0.9 m closer | `core_props/mod.rs:1110-1176` | 1/level, 27/27 |
| Dead vending machine | Stand 2 m before its glass 1.5 s: the flap lifts, a hand print (rule 9) | `docs/design/VENDING.md:106` | half the dead ones |
| Showroom bed / wardrobe | Crouch under or in: hidden from the staff | `anomalies/mod.rs:21, 698` | Showroom only |

### 1.3 Existing interactions that break the house style (fixing these is part of "without breaking the lore")

| Where | What breaks | Rule (source) |
|---|---|---|
| `tail.rs:226` `"[E] LIGHT SWITCH  (bad wiring)"`; `tail.rs:153` "THE SWITCH FLIPPED ON ITS OWN" | The prompt tells you a hidden property; the toast narrates | Rules are learned, not told (`PILLARS.md:94-96`); no narrator (`LORE.md:63-64`) |
| `tail.rs:146-149` (switch), `core_props/mod.rs:359` drawer `sfx_clank()`, `placers_p00/src/lib.rs:136` blip, vacuum `tail.rs:819-852`, payphone `mod.rs:1075-1084`, shaft `tail.rs:1172` `sfx_chime` | One-shots played on the stereo mix, not from the prop (`play_from`) | Every sound comes from somewhere (`PILLARS.md:73-74`; `CLAUDE.md:84-86`; `.claude/rules/audio.md`) |
| Vacuum `tail.rs:816-855` | A dice table (60/30/10); "THE VACUUM LIKES YOU" states an effect; the kick sets creatures straight to `hunt` (`tail.rs:841-851`) instead of leaving a noise for the brain | Rules, not dice (`VENDING.md:71-72`); price in what the game has, warn before you punish (`VENDING.md:67-70`); fairness: two tells before a strike (`ENTITIES.md:435-437`) |
| Payphone `mod.rs:1047-1105` | Dice; "SOMEONE SCREAMED. THEY HEARD." states the effect; every creature set to `hunt` at once (`mod.rs:1092-1099`) | Same as above; the payphone's own channel is "a whisper, where to look, or worse; never why" (`LORE.md:58-59`) |
| Shaft peer `tail.rs:1185-1188` | "YOU SHOULD NOT HAVE LOOKED." and others are a narrator; a flat −12 nerve | `LORE.md:63-64`; nerve is spent by them, the dark only so far (`LORE.md:456-457`) |
| Corpse ID card `tail.rs:1266-1297` | "BACKROOMS STAFF ID" names the place; "LEVEL n"; "ELIAS", "M. RANDOLPH" | Never a name or a number (`LORE.md:60-61`); E. and M. never named in full (`LORE.md:250-254`) |
| Name tags: locker `builders.rs:376` ("ELIAS", "M. R.", "M.KANE"), vacuum `tail.rs:35` ("ELIAS", "M. RANDOLPH") | Same naming problem; blocks giving lockers a verb (players would expect his things) | `LORE.md:250-254` |
| Candles `weather_fx.rs:397` `"[E] RELIGHT (3% battery)"` | The prompt states a cost and a mechanic that is not physical (a candle lit from a torch battery) | `PILLARS.md:94-101`; the verb is a real hand's (`SECRETS.md:183`) |
| Safe room `player.rs:1103` "SAFE ROOM — SANITY MENDS HERE" | States the effect | `PILLARS.md:94-96, 102-106` |
| R1 button `placers_p00/src/lib.rs:72-140` | A red/green game button on every level, lighting a far panel: a game's signpost | `PILLARS.md:115-118`; props look real (`PILLARS.md:57-59`) |
| Mannequin `mod.rs:1142-1168` | "Moves only while unseen" is the Watcher's rule (`LORE.md:361-363`); LORE.md has no mannequin entry (only torn note 17, `ui/hud.rs:144`) | Sameness (`WINDOWS.md:116-122`; `QUALITY.md:38`) |
| Pickups bob and spin `tail.rs:1488-1505` | A game convention, not a real object | `PILLARS.md:57-59` (minor) |

---

## 2. The rules an interaction must keep (checklist)

Each line has its source. A new interaction should pass every box.

**Canon and channels**
- [ ] **It reaches the player through an allowed channel.** "Lore is never explained; it is noticed. A new piece
  of canon must use one of these channels" (`LORE.md:28-29`). For props that means *architecture and props*:
  "what a place was for, shown by what is left in it … never labelled beyond the zone caption" (`LORE.md:31-33`)
  and *sound*: "what is somewhere else, heard before it is seen" (`LORE.md:39-40`).
- [ ] **No narrator, tutorial or explanation.** "Not allowed: tutorials about creatures, maps that explain the
  place, named SCP numbers in the game, any narrator." (`LORE.md:63-64`); "no tutorials, maps or explanations
  of creatures or anomalies; notes hint, never explain" (`PILLARS.md:94-95`). A toast is not a voice of the
  place: UI U8's lore review removed death lines "that were tips after the fact" and "rules the game lacks"
  (`UI.md:150`).
- [ ] **Printing is only what a real one carries about itself**, word for word, never about the place
  (`LORE.md:33-38`; `PILLARS.md:96-101`).
- [ ] **No voice except the payphone.** "a voice on a ringing line, a few words … never why" (`LORE.md:58-59`);
  M.'s Rule 2: "Never answer the phone." (`ui/hud.rs:131`).
- [ ] **No new text channel.** Torn notes are the paper channel and lie only where the canon says
  (`LORE.md:42-46`). "What you take" lines are for things put away: "what it is and its taste or feel, never
  what it does, never why" (`LORE.md:55-57`).
- [ ] **E. and M. are never named in full; never a level number** (`LORE.md:250-254`, `60-61`).
- [ ] **What is unexplained stays unexplained** (for example "What fills them … no note, page or sign may answer
  it", `LORE.md:871-872`).
- [ ] **When it adds canon, `LORE.md` is updated in the same commit, written from the code's real gates**
  (`LORE.md:7-8`; memory "canon-from-the-code"), and a `lore-keeper` round reviews it
  (`.claude/agents/lore-keeper.md`).

**Pillars**
- [ ] **The Lobby and the `empty` purpose stay sparse.** "Clutter belongs to purposes deeper down"
  (`PILLARS.md:38-40`; `LORE.md:84-86, 209-213`). Vending, windows and secrets all skip level 4
  (`core_props/mod.rs:540-544`; `WINDOWS.md:194-200`; `SECRETS.md:208-210`), and so should new verbs.
  Core props do reach L4 when nothing else is at home (`Home.elsewhere`, `core_props/mod.rs:152-180`).
- [ ] **Not in the Showroom or the Borrowed House** (their light and rules are their own: `VENDING.md:90-93`,
  `WINDOWS.md:196-198`), nor in the Stairwell or the Loop.
- [ ] **Real objects behaving as real ones do** ("props look like real objects, not toys",
  `PILLARS.md:57-59`; "architecture follows real offices …", `PILLARS.md:53-56`). The verb is "a verb in the
  game's own style; looking takes a second or two and makes a sound where it is" (`SECRETS.md:183-184`).
- [ ] **The screen at rest is the place alone.** "a ring and a verb only on what can be used"
  (`PILLARS.md:102-106`; `UI.md:112-114`). Do not ring every piece of clutter: keep reach about 1.2 m or less
  and choose props where using them means something. There are about 26 verbs a level now.
- [ ] **Sound.** "every sound comes from somewhere: a prop, a fixed signature spot, a fixture" (`PILLARS.md:73-74`);
  "the mix holds its level; nothing piles up at a level's start" (`PILLARS.md:75-76`); the hush must quiet
  it (`PILLARS.md:77-81`; `.claude/rules/audio.md`).
- [ ] **Never a game's signpost.** "what guides you was made by someone or is the place itself, never a game's
  signpost … with no glow, chime or line" (`PILLARS.md:115-118`). No prop may point at the door.
- [ ] **Pacing is left alone.** "calm, build, peak, relief" (`PILLARS.md:129-130`); "the same scare never repeats
  soon" (`PILLARS.md:135-137`). A prop may leave a noise for creatures. It may not set them to hunt.
- [ ] **Wrongness accumulates.** Deeper floors may answer differently: "deeper levels rot" (`PILLARS.md:61-62`).

**House style (the newest designs: vending, windows, secrets)**
- [ ] **Rules, not dice.** "A random table cannot be learned; what varies is fixed per machine"
  (`VENDING.md:71-72`); "Its varying choices … are fixed per machine, from the level's seed and where it
  stands" (`VENDING.md:126-127`).
- [ ] **Price everything in what the game already has** (noise, light, nerve, time), and **warn before you
  punish** (`VENDING.md:67-70, 122-126`). Noise is given to creatures through the brain's memory
  (`vending_noise`, `core_props/vending/rules.rs:524-543`; `window_noise`, `core_props/window/blind.rs:90-111`:
  `mem.heard`, `awareness`, reach "along the way" × `hearing`), never by setting `state = "hunt"`.
- [ ] **Effects are never stated** ("never said … not the thing, not a meter's words, not the notebook",
  `LORE.md:759-761`; `SECRETS.md:217-225, 278-279`).
- [ ] **Bounded and fair.** A prop never comes after you and never kills; a throw-down leaves
  `KNOCKED_LEAVES` (`VENDING.md:122-124`). Nothing lowers what the dark leaves you or raises what a touch takes;
  a chase stays under your sprint; "Leaving a thing where it is is always a fine choice" (`LORE.md:799-807`;
  `ENTITIES.md:431-461`).
- [ ] **Supply discipline.** Most places hold "what any floor holds … or nothing but dust (never a coin: you carry
  one at a time, from the machines)" (`LORE.md:719-721`); snacks come only from machines (`LORE.md:684-685`).
  The supply is already generous: 3 + 3 pickups (`core_props/mod.rs:700, 719`) plus about 9 drawers a level at
  28% battery and 22% almond water (`builders.rs:142-152`).
- [ ] **No prop as a creature; no talking, no jump scares, no gore** (`VENDING.md:79-82`).
- [ ] **Avoid the canon's sameness list.** These are already overused: knocking (vending V5 removed "the canon's
  sixth knocking", `VENDING.md:157`); a tell that is a sound stopping; a counter that is stillness in the dark;
  "tall"; a turned back (`WINDOWS.md:116-122`; `QUALITY.md:38`); rules about your eyes; a hand at a glass
  (`QUALITY.md:57`); "wet", "hole", chalk (`QUALITY.md:66`); things that move while unseen (the Watcher);
  things that flip "on their own".
- [ ] **Do not dilute a secret's tell.** Hiding places have one tell all run: the askew tile, the grille on two
  screws, the drawer not quite shut, the rag behind a radiator, the hole (`LORE.md:697-717`;
  `SECRETS.md:186-194`). No verb on ordinary tiles, vents or Building radiators.

**Code invariants**
- [ ] **Determinism** (`CLAUDE.md:73-83`; `docs/engineering/DEVELOPMENT.md:76-103`; `.claude/rules/world.md`): build-time
  choices come from `rnd()` (and a new draw shifts every later placer, which means re-recording), or better,
  from a salted stream of its own (`with_own_stream` / `SplitMix64`; vending: "one draw from the build's stream
  and the rest from a stream of its own", `VENDING.md:152`; window wear "from a stream of its own (salted by
  where it stands)", `WINDOWS.md:141-142`). What happens in play draws from the native stream or its own
  seeded stream, never the build's (`VENDING.md:129-131`). Note that `rnd()` outside a build *is* native
  (`base/rng.rs:79-91`).
- [ ] **Sounds draw the frame stream.** `noise_burst` takes `native_rnd()`, so a prop must "begin as it is,
  silently" and play only within earshot (`HEARD_WITHIN` 30 m, `vending/rules.rs:58, 626`; `blind.rs:64-69`;
  `SECRETS.md:274-277`; memory "sounds-draw-the-frame-stream"). Use `play_from(x, y, z, ref, gain, occlude, f)`
  (`audio/ambient/mod.rs:436-456`). Held sounds need `Soundscape::note`, not `tone`; a listen test must fail
  with the sound at gain 0 (memory "audio-tone-is-a-pluck").
- [ ] **Light belongs to places** (`CLAUDE.md:84-86`; `.claude/rules/visuals.md`). A prop that lights its tile
  goes into `build_light_map` (`visuals/lighting.rs:324-400`: panels switched off, exit, vending, her lamp) and
  rebuilds it on change. No lamp cones; never tie light to the player.
- [ ] **Register verbs after settling.** An interactable pins wall faces within 1.35 m during `settle_props`
  (`world/settle.rs:315-318`); "a window must be finished after settling … and must not register its verb
  before then (it would pin its own wall)" (`WINDOWS.md:74-77`). The build order (`world/build.rs:445-466`)
  already has `vending_settled` and `windows_settled` hooks. A use point must follow its prop if the prop
  moves (vending moves its four every frame, `vending/rules.rs:172-181`).
- [ ] **One binding table.** Everything goes through E / left click (`session/input.rs:62`; `UI.md:95-100`).
  No new keys; "using anything never stops a run" (`UI.md:97`).
- [ ] **Gates and evidence** (`CLAUDE.md:105-112`): each rule held by a test that fails when broken (vending's
  and windows' pattern); `listen` / `loudness`; placer smoke tests exercise every interactable
  (`DEVELOPMENT.md:116-117`, `debug/testkit.rs:122-131`); goldens and reference levels re-recorded only for an
  intended change; a `docs/plan/content.json` entry (kind "prop with rules", as `props/vending`, `props/windows`).

---

## 3. Common props with no verb now

### 3.1 Core props (`place_props`): on nearly every level

Census over 27 levels (seeds 4242, 17, 777, levels 4–12). The count is groups a level where present.
Homes are from `core_props/mod.rs:163-181`, and scatter counts from `core_props/mod.rs:538-569`.

| Kind | Levels | Per level | Home (archetype / themes) | Builder | Verb now? |
|---|---|---|---|---|---|
| chairs (one on its side) | 27/27 | 3.0 | SEATING: office, lobby / office, domestic, party | `mod.rs:737-755`, `build_detailed_chair` 1180+ | no |
| cardboard stack | 27/27 | 4.0 | STORAGE: tunnels, electrical, hall / industrial (rows of 3) | `builders.rs:180-235` | no |
| filing cabinet | 27/27 | 3.0 | OFFICE | `builders.rs:34-178` | yes (drawers) |
| traffic cone | 27/27 | 3.0 | WORKS | `builders.rs:636` | no |
| wet floor sign | 27/27 | 2.0 | WET | `builders.rs:237` | no |
| pile (junk) | 27/27 | 2.0 | STORAGE | `builders.rs:1004` | no |
| balloon | 27/27 | 1 | anywhere | `mod.rs:833-853` | no |
| mannequin | 27/27 | 1 | anywhere (d > 14) | `mod.rs:1110-1176` | no (moves unseen) |
| elevators (1 works, 1 OUT OF SERVICE) | 27/27 | 2 | wall | `tail.rs:1327-1346`, `builders.rs:1267-1336` | the working one only |
| wall mirrors | 27/27 | 2 | wall | `tail.rs:1417-1429` | no |
| tableaux (doll in a wheelchair, cone row, box pyramid) | 27/27 | – | – | `tail.rs:1431-1467` | no |
| CRT TV (live static) | 26/27 | 1 | LIVING: domestic, office | `mod.rs:757-799` | no |
| wall clock (ticking) | 26/27 | 1 | CLOCK_WALL | `mod.rs:801-831` | no |
| locker (2–3 doors, padlocked, 35% ajar) | 26/27 | 2.0 | LOCKER_ROOM: office, hall, pool / office, bathroom, industrial | `builders.rs:325-426` | no |
| fire extinguisher | 26/27 | 2.0 | SERVICE | `builders.rs:428` | no |
| trash can | 26/27 | 2.9 | PUBLIC | `builders.rs:509-611` | no |
| suitcase (open, spilling) | 25/27 | 2.0 | LUGGAGE | `builders.rs:776` | no |
| book stack | 25/27 | 3.0 | READING: office / office, victorian, domestic | `builders.rs:895-944` | no |
| TV on floor (unlit) | 25/27 | 1 | LIVING | `builders.rs:828` | no |
| coat rack (coat on 70%) | 24/27 | 2.0 | CLOAKROOM | `builders.rs:946-971` | no |
| potted / floor plants | 24/27 | 3 + 1 | GREENERY | `builders.rs:1060, 1162` | no |
| fluorescent tube (dropped) | 23/27 | 3.0 | WORKS | `builders.rs:974` | no |
| wheelchair | 22/27 | 1 | HOME_CARE | `builders.rs:653` | no |
| baby doll | 22/27 | 1 | NURSERY | `builders.rs:741` | no |
| rubber duck | 18/27 | 2.9 | BATH_TOYS | `builders.rs:613` | no |

### 3.2 Themed placers (649 in 33 crates): individually rare

On how many of 54 dumped levels each placed something (`props/placer_freq.txt`). Only placers with a
plausible real verb are listed:

| Placer | Levels | Where | Note |
|---|---|---|---|
| `place_wall_vents` | 32/54 | `placers_p32/src/lib.rs:60` | **no verb**: it would dilute the vent secret's tell |
| `place_coffee_cups` | 28/54 | `placers_p31/src/floor.rs:104` | nothing a hand would do |
| `place_fire_extinguishers` | 23/54 | `placers_p31/src/wall.rs:539` | see P14 |
| `place_clipboards` | 17/54 | `placers_p31/src/floor.rs:6` | paper (P5 family) |
| `place_wall_calendars` | 17/54 | `placers_p27/src/lib.rs:702` | text: no |
| `place_mops` | 17/54 | `placers_p30/src/lib.rs:328` | – |
| `place_rusted_pipes` | 16/54 | `placers_p31/src/wall.rs:588` (40% when themed) | P12 (needs a valve part) |
| `place_trash_cans` | 14/54 | `placers_p31/src/floor.rs:162` | P7 |
| `place_bulletin_boards` | 14/54 | `placers_p31/src/wall.rs:328` | text: no |
| `place_cardboard_boxes` | 13/54 | `placers_p31/src/floor.rs:57` | P1 |
| `place_backpacks` | 11/54 | `placers_p01/src/lib.rs:252` | could share P2's "a person's things" |
| `place_table_lamps` | 11/54 | `placers_p28/src/lib.rs:946` | P15 |
| `place_wall_thermostats` | 11/54 | `placers_p27/src/lib.rs:764` | – |
| `place_alarm_clocks` | 9/54 | `placers_p27/src/lib.rs:217` (12%) | P9; already rings as ambience every 6–12 min (`audio/ambient/lobby.rs:238-246`) |
| `place_junction_boxes` | 9/54 | `placers_p31/src/wall.rs:241` | no (see "do not") |
| `place_walkie_talkies` | 7/54 | `placers_p29/src/lib.rs:220` | voice risk |
| `place_wooden_crates` | 7/54 | – | P1 |
| `place_wall_phones` | 6/54 | `placers_p30/src/lib.rs:222` (15%; rings to no one every 10–20 min, `lobby.rs:84-93`) | P10 |
| `place_rotary_phones` | 6/54 | `placers_p28/src/lib.rs:619` | P10 |
| `place_water_fountains` | 5/54 | `placers_p31/src/wall.rs:169` (15%; drips, `audio/ambient/fixtures.rs:15`) | P11 |
| `place_radiators` (rooms) | 5/54 | `placers_p31/src/wall.rs:495` (25%) | – |
| `place_typewriters` | 5/54 | `placers_p27/src/lib.rs:163` | text risk |
| `place_wall_doors` | 5/54 | `placers_p19/src/lib.rs:692` | "TRY THE DOOR": rare; the Loop's latch click is a sameness risk |
| `place_hand_dryers` | 4/54 | `placers_p26` (`toys::place_hand_dryers`) | P11 |
| `place_sink_faucets` | 6% | `placers_p18/src/lib.rs:1249` | P11 |
| `place_small_radios` / `place_clock_radios` | 3/54 each | `placers_p20/src/lib.rs:502`, `placers_p27/src/lib.rs:271` | static only, never a voice |

Conclusion: to change how most levels feel, put verbs on core props. Placer verbs touch a few levels in ten
unless the placer's home is widened.

---

## 4. Proposals, ranked

Every proposal plays its sound from the prop with `play_from`, makes it only within earshot, and makes nothing on
a level's first frame. Each registers its verb after settling, takes its fixed choices from a stream salted
by its tile (no new build draws, unless stated), skips L4 and the Showroom, the Borrowed House, the Loop and
the Stairwell, and ends in a test per rule (one that fails when the rule is broken) and a `content.json`
entry. Noise means a shared helper (factor `vending_noise` and `window_noise` into one
`prop_noise(at, reach, loud, tag)`) feeding `mem.heard` and `awareness`, with `director_note`.

### P1. Cardboard boxes: `[E] OPEN THE BOX` (rank 1)
- **Props:** the top box of each core cardboard stack (`builders.rs:180-235`; 4 a level in storage halls,
  27/27); also `place_cardboard_boxes` (`placers_p31/src/floor.rs:57`) and wooden crates.
- **What happens:** the packing tape rips (about 1 s), and the two flaps fold back (the "dropped flap" mesh
  exists, `builders.rs:200-204`). Inside is what its handwritten label says a box holds (labels at
  `builders.rs:184`):
  - OFFICE 3, HR, TAX: hanging files and loose paper.
  - BOOKS: paperbacks.
  - X-MAS: tinsel and a string of dead bulbs.
  - MOM, ATTIC: folded clothes.
  - FRAGILE: plates wrapped in newspaper.
  - DO NOT OPEN: the same ordinary things, nothing more ("a sign is not a promise", `LORE.md:452-453`).
- **Gives:** paper from the paper boxes (+2–4 sheets, once a box), otherwise nothing. In the Habitable Zone's
  storage halls (game level 5 is Backrooms Level 1; Backrooms Wikidot Level 1's crates, where "virtually
  anything can be found"), now and then one bottle of almond water, at most one a floor. Never a battery and
  never a coin (`LORE.md:719-721`).
- **Cost:** the rip is heard about 8 m along the way (awareness about 0.4); about 1 s spent.
- **Sound:** a broadband tape rip (stick-slip ticks over 2–6 kHz), cardboard flaps thumping, and paper shuffling.
- **Canon touched:** `LORE.md:31-33` (props show what a place was for), `239-244` (purposes: storage hall),
  `668-670` (paper marks your path; the Reader keeps the ones you drop), `393-396` (give it paper),
  `665-666` and torn note 1 "The bottles just... appear" (`ui/hud.rs:128`). **No LORE addition needed** if
  only paper. A line in *Survival* if almond water is added.
- **Code cost: low.**
  - One interactable a stack, registered after settle.
  - The label index has to be kept on the box: the build already chooses it with `choice` (`builders.rs:207`),
    so storing it adds no draw.
  - The flap tween runs through `schedule` or a small animator.
  - Contents can reuse `build_paper_pile` (`builders.rs:1338`) or nothing.
- **Flags:** none against §2 if DO NOT OPEN stays ordinary.

### P2. Lockers: `[E] OPEN THE LOCKER` (rank 2)
- **Props:** core lockers (`builders.rs:325-426`; 2 a level, 26/27); `place_metal_lockers`
  (`placers_p20/src/lib.rs:971`), whose slam already exists as ambience (`fixtures.rs:199-207`).
- **Tell:** every door has a padlock (`builders.rs:364-371`). Let about one door in three hang with its padlock
  open on the hasp. Those doors, and the ones already ajar (35%, `builders.rs:383-405`), take the verb.
  Locked doors get no verb, because a verb that always fails is a game's.
- **What happens:** the latch lifts and the door swings out on its hinge to about 100°, using the ajar code's
  leaf conversion (`builders.rs:383-400`, `attach` at `builders.rs:11`) at use time. Inside is what a staff
  locker holds: a coat on its hook (exists, `builders.rs:401-405`), a towel, a pair of shoes, a mug, a hanger
  still swinging, or nothing.
- **Gives:** about one in six, what any floor holds (a bottle or a battery); never a coin or a thing put away.
- **Cost:** sheet steel. The hinge's squeal and the door's boom are heard about 10 m along the way, the
  blind's reach (`blind.rs:13-14`).
- **Canon touched:** `LORE.md:31-33`, `719-721`.
- **Flags:**
  - Remove "ELIAS", "M. R." and "M.KANE" from the tags (`builders.rs:376`, against `LORE.md:250-254`), or
    players will look for his things.
  - **No hiding inside.** That is the Showroom's rule (`LORE.md:565-566`) and a stillness counter
    (`QUALITY.md:38`).
  - If a locker ever holds a thing put away, it becomes a new hiding-place kind and needs `LORE.md:697-717` and
    `SECRETS.md:186-194` additions.
- **Code cost: medium.** The open-padlock pose can be set after the build from a salted stream (no draw), the
  swing needs an animator, contents need meshes, and there is one verb a door.

### P3. CRT TV: `[E] TURN IT OFF` / `[E] TURN IT ON` (rank 3)
- **Props:** core CRT TV with live static (`core_props/mod.rs:757-799`; 26/27, 1 a level). Its light flickers
  `rand2(0, 2.6)` within 30 m and its static redraws every 0.09 s (`ui/hud.rs:1538-1551`). It is silent now.
- **What happens:** the power switch clicks, the picture collapses to a bright line, then a dot that fades, and
  the glow dies. On: the degauss thunk, and the static comes back. It **never** turns itself back on, to
  avoid the "on its own" sameness (`tail.rs:153`).
- **Gives or costs:** nothing stated. If its glow joins the light map like the vending machine's
  (`VEND_LIGHT`, `vending/rules.rs:18`; `visuals/lighting.rs:381-387`), turning it off darkens its corner:
  - the Smiler sees a body "only close, and only lit" (`LORE.md:332-333`);
  - the Reader sees only "where you are lit or moving" (`LORE.md:389-390`);
  - in the Borrowed House, lit means dangerous (`LORE.md:579-583`), though that level should be excluded anyway.

  All of this is learned, never told.
- **Sound:** while on and within earshot, a soft static hiss with a faint line whine (`Soundscape::note`; a
  listen test must fail at gain 0). Off: a relay click, then the whine falls away.
- **Canon touched:** `LORE.md:31-33`, `907-928` (add the TV's hiss to *The sound of the place*: a **small
  LORE addition**).
- **Code cost: low to medium.**
  - `tv_data` gains `on` (`base/records.rs`), the HUD block respects it, the screen material swaps, and one
    interactable is registered after settle.
  - A light-map entry with a rebuild on change is optional. It moves goldens, because the TV starts on.

### P4. The dead elevator: `[E] PRESS THE BUTTON` (rank 4)
- **Props:** the second elevator, which is dark with OUT OF SERVICE (`builders.rs:1305-1322`, buttons
  `0x3a3025` with no emissive, `builders.rs:1296-1299`); it has no verb now (`tail.rs:1340-1344`). On 27/27
  levels.
- **What happens:** the button clicks in and stays dark. On about one dead elevator in three (fixed per
  elevator by its seed), 5–12 s later a car comes down the shaft behind the doors: the cables sing, a seam of
  light slides down the gap between the doors, the car passes without slowing, and is gone below. Once per
  elevator. It never stops or opens.
- **Gives or costs:** nothing.
- **Canon touched:** "A sign is not a promise" (`LORE.md:452-453`); torn note 13 "Layer Notes": "The elevators
  know. Sometimes they take you to them." (`ui/hud.rs:140`). **A one-line LORE addition** (a new thing the
  place does).
- **Sound:** cable whir rising and falling, the counterweight's rumble, the doors rattling in their track.
  All from the doors, only within earshot.
- **Flags:**
  - It must not read as a way down or a guide (`PILLARS.md:115-118`).
  - It must not imitate the director's build-up signs (steps, failing lights, silence, `LORE.md:443-447`).
  - The light in the seam is the prop's own, not ambient.
- **Code cost: low.** One interactable, one sound, and an emissive strip animator.

### P5. Book stacks: `[E] TEAR OUT PAGES` (rank 5), and the paper family
- **Props:** core book stacks (`builders.rs:895-944`; 3 a level, 25/27); `place_clipboards`
  (`[E] TAKE THE SHEETS`, 17/54), floor books, paperbacks.
- **What happens:** you open the top book, its spine cracks, and a few pages come out with a rip. Deeper, the
  place rots (`PILLARS.md:61-62`): from about level 9, books are swollen with damp and the pages come away
  stuck together, so you get nothing. A toast in the house voice, "NOUN — PHRASE" (`ui/hud.rs:83-95`), for
  example "PULP — THE PAGES ARE STUCK TOGETHER".
- **Gives:** +2–3 sheets, once a stack.
- **Cost:** a quiet rip heard about 3 m, and about 1 s.
- **Canon touched:** `LORE.md:668-670` (paper), `393-396` and `462` (the Reader tears the sheets you feed it once
  it learns, which already bounds paper spam).
- **Flags:** no readable text, ever. That would be a new paper channel (`LORE.md:42-46`).
- **Code cost: low.**

### P6. Chairs: `[E] SIT DOWN` and `[E] SET IT UPRIGHT` (rank 6)
- **Props:** core chairs (`core_props/mod.rs:737-755`; 3 a level, one on its side, `mod.rs:748-751`; 27/27).
  Later, the safe room's bench (`tail.rs:861-960`) and placer chairs.
- **What happens when you sit:** the view drops to a seated eye (about 1.2 m), the chair creaks and its feet
  scrape. Any move key stands you up in about 0.5 s, so you are slow to rise. A knock-down stands you up.
- **What happens when you set it upright:** it is lifted, scrapes, and lands on four feet. It stays where you
  left it.
- **Gives:** recommend **nothing** at first; the room becomes real. A unsaid step of nerve while seated in
  light would sit within `LORE.md:799-807`, but it adds a rest counter.
- **Cost:** the scrape is heard about 6 m.
- **Flags:**
  - Sitting is stillness. Never make it hide you, because stillness is already the counter for two creatures
    (`QUALITY.md:38`).
  - `reset_hands` must clear the seated state on death and new runs (`UI.md:150`).
- **Canon touched:** `LORE.md:31-33`; no addition.
- **Code cost: medium.** A player state (eye height, no walk, settings for head bob), three verbs a level, and
  sounds; a chair-creak voice already exists for the rocking chairs (`audio/ambient/fixtures.rs:142-150`).

### P7. Trash cans: `[E] LOOK INSIDE` (rank 7)
- **Props:** core trash cans (`builders.rs:509-611`; 2.9 a level, 26/27; paper balls and a can on top);
  `place_trash_cans` (`placers_p31/src/floor.rs:162`).
- **What happens:** the paper balls are pushed aside (one is gone) and the can rattles. Mostly
  "WRAPPERS — NOTHING YOU WANT". About one bin in three gives a sheet you smooth out.
- **Gives:** +1 paper at most; never a bottle or a battery.
- **Cost:** a crumple, heard about 4 m.
- **Flags:** do not use "REACH IN" (the hole's and the vending bin's verb; "hole" sameness, `QUALITY.md:66`).
- **Code cost: low.**

### P8. Coat racks: `[E] GO THROUGH THE POCKETS` (rank 8)
- **Props:** core coat racks with a coat (`builders.rs:946-971`, 70% have one; 24/27); locker coats.
- **What happens:** the coat swings on its hook, and a toast names what is in the pockets: lint, a bus ticket,
  a receipt faded blank, a single glove, nothing.
- **Gives:** about one in eight, a battery. Never a coin. Never a key, which would promise a lock the game
  does not have.
- **Flags:**
  - Sameness with the tweed cap "on a coat hook in an office wing" (`LORE.md:717, 747`; `SECRETS.md:239`).
    Pockets must never hold a thing put away, or the cap's "where nobody would leave it" is lost.
  - Lines must never name a person or a place (`UI.md:150`).
- **Code cost: low.**

### P9. Wind-up alarm clocks: `[E] WIND IT` (rank 9; a real tool, but a rare prop)
- **Props:** `place_alarm_clocks` (`placers_p27/src/lib.rs:217`, 12% of levels when office-themed; 9/54).
  Clock radios.
- **What happens:** the key ratchets and the tick gets louder. After a delay fixed per clock by its seed
  (20–60 s, not when its hands say: M.'s Rule 3 "The clocks lie", `ui/hud.rs:132`), it rings: a hammer between
  two bells for about 8 s, rattling on the floor. Once a clock.
- **Gives or costs:** nothing to you. It is a sound you leave behind: what hears, within about 20 m along the
  way, comes to look where it rang (the bang's mechanism, `vending/rules.rs:524-543`). Against Hounds, which
  hear along the way (`LORE.md:345-347`), it is a decoy learned by playing.
- **Canon touched:** `LORE.md:39-40` (sound), `341-358`. A line in *The sound of the place*; otherwise
  canon-neutral.
- **Flags:**
  - Check that `pacing`, `personas` and `beatable` bands hold.
  - Ringing *by itself* already exists as ambience that creatures do not hear (`lobby.rs:238-246`). Keep that
    unheard by creatures, or the player cannot tell their clock from the place's.
  - Widen its home to office wings and living quarters to matter.
- **Code cost: low to medium.** It is a floor prop (no wall to pin), so it can register in its placer.

### P10. Phones: `[E] LIFT THE RECEIVER`, never a voice (rank 10)
- **Props:** the core payphone after its call (`answered`, `core_props/mod.rs:1041-1045`; 27/27); wall phones
  (`placers_p30/src/lib.rs:222`), rotary phones (`placers_p28/src/lib.rs:619`), wall pay phones (`placers_p20`).
- **What happens:**
  - A wall or rotary phone gives a dial tone (350 + 440 Hz, `sfx_dial_tone`, `audio/sys.rs:469-472`), or on a
    dead one a faint hiss. The cradle clacks when it goes back.
  - The payphone, after its call, gives **no tone at all**: it rang on a line it does not have.
- **Gives or costs:** nothing.
- **Flags:**
  - The payphone is the canon's one voice (`LORE.md:58-59`); any other voice is a narrator
    (`LORE.md:63-64`).
  - A ringing wall phone has no verb, because M.'s Rule 2 says "Never answer the phone" (`ui/hud.rs:131`).
- **Canon touched:** **a LORE addition** under the payphone: other phones are a tone or nothing.
- **Code cost: low.** A handset pose (`PhoneData.handset`, `core_props/mod.rs:1009-1022`) and sounds.

### P11. Washroom water: `[E] PRESS THE BUTTON` / `[E] TURN THE TAP`, and the hand dryer (rank 11)
- **Props:** water fountains (`placers_p31/src/wall.rs:169`, which already drips, `fixtures.rs:15`), sink
  faucets (`placers_p18/src/lib.rs:1249`), hand dryers (`placers_p26`). All are rare.
- **What happens:** a thin arc of water. Deeper, it coughs, spits rust-brown and gurgles dry while the pipe
  shudders in the wall (`PILLARS.md:61-62`). The hand dryer roars about 20 s and stops.
- **Gives:** nothing to drink. Almond water is the drink (`LORE.md:665-666`), and Elias wrote "Do not drink
  from it" of the pool (`ui/hud.rs:135`).
- **Cost:** the dryer's roar is heard about 20 m along the way (a decoy like P9); the pipe about 8 m.
- **Flags:** "wet" imagery is overused (`QUALITY.md:66`), so keep it to sound, with no new decal.
- **Code cost: low a prop.** Little reach unless washrooms get more of them.

### P12. Pipes: `[E] TURN THE VALVE` (rank 12)
- **Props:** `place_rusted_pipes` (`placers_p31/src/wall.rs:588-638`, 40% when themed; 16/54) with a new
  handwheel part, in service tunnels and maintenance.
- **What happens:** stiff, then a quarter turn. The pipe bangs along its length (water hammer) and hisses at a
  joint. Deeper, it is seized and a flake of rust falls.
- **Cost:** the bang carries about 20 m along the way.
- **Canon touched:** the survey note "Keep moving. Don't touch the pipes." (`ui/hud.rs:159`) becomes true by
  consequence: a hint, never explained (`LORE.md:42-46`). Backrooms Wikidot Level 2's pipes "burst, leak"
  (game level 6 is Level 2).
- **Flags:**
  - A bang, not a knock (`VENDING.md:157`).
  - Never on the Building's floors (the Works: "Everything here was switched off", `LORE.md:535-536`).
- **Code cost: medium.** A new Blender part and a placer change, which shifts draws and needs re-recording.

### P13. Wheelchair: `[E] PUSH IT` (rank 13)
- **Props:** core wheelchair (`builders.rs:653`, 22/27) and the doll-in-a-wheelchair tableau
  (`tail.rs:1431-1445`).
- **What happens:** it rolls about a metre and stops, and one wheel keeps turning with a squeak. It never
  moves by itself (mannequin and Watcher sameness).
- **Cost:** a squeak, heard about 5 m.
- **Code cost: low.** A roll animator with `circle_hits_wall`.

### P14. Fire extinguisher: `[E] PULL THE PIN` (rank 14, **flagged**)
- **Props:** core extinguishers (`builders.rs:428`, 2 a level, 26/27) and `place_fire_extinguishers` (23/54).
- **What happens:** a dry-chemical hiss for about 8 s and a white cloud filling a tile. Deeper, the gauge is in
  the red and it gives only a cough of powder.
- **Flag:** a cloud that blocks sight is a **new counter** to the Smiler, the Reader and the one behind the
  glass. It needs an `ENTITIES.md` design against the fairness invariants (`ENTITIES.md:431-461`) and a lore
  round. A rising cloud also echoes the self-assembly's fog (`LORE.md:486-499`) and the Loop's.
  Only "a cough of powder" is safe without that work.
- **Code cost: high** (LOS volume, particles, sound).

### P15. Table lamps: `[E] PULL THE CHAIN` (rank 15)
- **Props:** `place_table_lamps` (`placers_p28/src/lib.rs:946`, 11/54).
- **What happens:** the chain clicks. Most bulbs are dead (rot); a live one lights its tile. That goes into
  the light map with a rebuild, like the camp lamp (`lighting.rs:388-394`).
- **Flag:** never in a blackout zone ("The lights are dead", `LORE.md:233-234`).
- **Code cost: medium.**

### Do not (each breaks a rule in §2)
- **Verbs on ordinary vents, ceiling tiles or Building radiators.** They dilute the secrets' tells
  (`SECRETS.md:186-194`).
- **Hiding in lockers or wardrobes outside the Showroom.** That spreads an anomaly's rule (`LORE.md:565-566`)
  and adds another stillness counter (`QUALITY.md:38`).
- **Mirrors you look into.** Torn note 23 already has a four-second gaze rule (`ui/hud.rs:150`), and the canon's
  eye rules are "now many" (`QUALITY.md:57`).
- **A doll's tilt cry-box ("ma-ma").** It echoes the Stairwell's crying child (`LORE.md:644-646`).
- **Radios, walkie-talkies, typewriters, bulletin boards or calendars that give words.** That is a narrator or a
  new text channel (`LORE.md:42-46, 63-64`). Static only.
- **Breakers or fuse boxes that kill a region's hum.** Silence is the director's warning (`PILLARS.md:77-81`);
  a player-made silence muddies it. Blackout zones are dead, not switched off (`LORE.md:233-234`).
- **Squeezing the rubber duck.** It reads as a gag; the code already removed a gag stack for that reason
  (`core_props/mod.rs:309-311`).
- **Any verb that sets creatures to hunt directly, or that rolls dice per use** (§1.3).

---

## 5. Sources (only where they fit the canon)

- **Backrooms Wikidot, Level 1** (<http://backrooms-wiki.wikidot.com/level-1>, CC BY-SA 3.0): crates in
  which "virtually anything can be found", sometimes "with food or Almond Water". This fits P1 on the game's
  level 5 (Habitable Zone), already credited under *Backrooms* lore (`LORE.md:1045-1049`). Not taken: the
  crates' "disappear once not looked at", which is the Watcher's and the vending machine's territory.
- **Backrooms Wikidot, Level 2** (<http://backrooms-wiki.wikidot.com/level-2>): pipes that "burst, leak".
  It supports P12 next to the game's own survey note.
- **The game's own torn notes** carry most of the weight. No new canon is invented:
  - M.'s Rule 2, phones (`ui/hud.rs:131`);
  - Rule 3, the clocks lie (`:132`);
  - "Layer Notes", the elevators (`:140`);
  - "Paper Trail" (`:143`);
  - "Survey: Utility Halls", don't touch the pipes (`:159`);
  - "Almond Water", the bottles appear (`:128`).
- **The research in `docs/research/creature-ai-and-fear.md`** (lines 21, 133, 251: Outlast's and Alien
  Isolation's lockers) is the warning behind "no hiding in lockers".
- **No SCP is needed** for P1–P13. P14 would need its own research.

**LORE.md additions needed:**
- P3: one line in *The sound of the place*.
- P4: one line, the dead lift's passing car.
- P9: one line in *The sound of the place*.
- P10: one line under the payphone channel, other phones are a tone or nothing.
- P1: one line in *Survival*, if almond water is added.
- P2: an entry in *What is put away*, only if lockers ever hide things.
- P14: entries in `LORE.md` and `ENTITIES.md`.
- The §1.3 fixes: remove the names and the "BACKROOMS STAFF ID" text, and add a mannequin line or change its
  rule.
