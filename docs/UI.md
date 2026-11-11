# UI and controls: the hands and the frame

How the player moves, uses things and reads the screen, why, and the plan
that took it from "a column of spaced capitals and fourteen keys" to
controls you can use on the run and screens that belong to the place. The
pillars it serves are in [`PILLARS.md`](PILLARS.md); the research behind it
is in [`SOURCES.md`](SOURCES.md) section 9.

## The analysis (2026-10-26)

Read from the code and from the HUD test's frames
(`cargo test --release -p game --test hud`, `crates/target/hud-*.png`).

### Controls

| Key | Action | What was wrong |
|---|---|---|
| WASD / arrows | move | Ctrl+arrows is a macOS Mission Control shortcut |
| Shift (hold) | run | at zero stamina the run stuttered: three frames of running every 0.9 s while Shift was held (`stamina > 1.0`, no recovery threshold) |
| Ctrl / C (hold) | crouch | hold only |
| E | use | the prompt and E took the *first* usable thing in reach in the list, not the one you face |
| F, Q, G/1, R/2 | flashlight, paper, water, battery | — |
| H | cry for help | right hand: you let go of the mouse or of WASD to call |
| J | journal | right hand; opening it released the pointer (no looking) while the level ran on, and A/D both strafed and turned pages |
| K | debug panel | in the shipped game, beside J |
| Escape | "PAUSED, CLICK TO RESUME" | Escape again did nothing; the engine released the pointer on every Escape |
| mouse | look | buttons and wheel did nothing in play; no sensitivity, no invert |

Every action key re-fired with the OS key repeat (`key_down` ignored the
repeat flag on purpose, pinned by `keydown_actions`): holding F strobed the
flashlight, holding G drank every bottle.

### Screens

Every screen was one centred column of Courier New in letter-spaced
capitals over 88% black (`paint_overlay`): a title, a paragraph that
explained the premise ("Find the exit door before it finds you", the
objective the gameplay plan removed), a two-line wall of fourteen keys and
an explanation of the items in capitals (on the title, the pause *and* the
death screen), two boxed buttons, five medal pills with a shimmer, and a
blinking "CLICK TO ENTER". Level 1 is built behind the title and hidden.
The death screen was the same column with "IT FOUND YOU" and the seconds
survived; the ending card the same column again.

In play: LEVEL, the seed and the thread top left; three item lines with
their keys bottom left ("ALMOND WATER ×2 [G] DRINK"); labelled SANITY and
STAMINA bars bottom centre and FLASHLIGHT [F] bottom right, always on; up
to five medal boxes top right for the rest of the run; "— CROUCHING —";
"[E] OPEN THE DOOR"; toasts in spaced capitals. All Courier New, all
capitals.

| Where | What works against it |
|---|---|
| 4. You are never shown everything | The HUD shows everything all the time: three labelled meters, the inventory with its keys, the seed, the medals. |
| 1. Emptiness is the threat | The title is a black page of text; the empty lobby, already built behind it, is not shown. |
| 4. Rules are learned, not told | The title, pause and death screens explain the items and list fourteen keys, every time. |
| 5. Dread, then release | Death is a banner and a wall of keys; the retry is a letter in a blinking line. |
| Authored, not templated | Monospace for everything that is not code, everything centred, every string in tracked capitals, every element in a box: the defaults a template falls back to (section 9 of `SOURCES.md`, "AI-slop" catalogues). |

## What the research says

Section 9 of [`SOURCES.md`](SOURCES.md) has the sources; in short:

- **Layouts converge.** Shift run, Ctrl crouch, F light, E or left click to
  use, Tab for the inventory or notebook, R for batteries ("reload") in
  Amnesia, Outlast, Alien: Isolation, Phasmophobia and Lethal Company. None
  of them stops the game to use an item; full-screen inventories are for
  management. Scroll-to-select fails under pressure (Phasmophobia added
  direct keys in 2026 for that reason).
- **Reach.** With W under the middle finger and Shift under the little
  finger, the left hand reaches 1–4, Q, E, R, T, F, G, Z, X, C, V, Tab,
  Caps Lock, Ctrl and Space; J, H, M, P and 5+ need the hand to leave.
- **Holds tire.** Offer a toggle for run and crouch; never require a long
  hold or a chord (Game Accessibility Guidelines, Xbox guideline 107).
- **macOS.** No key-up arrives for a key pressed while Command is held;
  Ctrl+arrows belongs to Mission Control.
- **Minimal is not zero.** Players accept a HUD that helps; what changed is
  shown, then fades (Jørgensen). A held glance key (Phasmophobia's watch)
  instead of a permanent HUD.
- **Institutional type makes the uncanny routine** (Control's Helvetica /
  Akzidenz documents). Neutral sans for menus, typewriter or hand for
  in-world paper (Resident Evil). No capitals for sentences (NN/g).
- **Menus.** A title that is a place; a pause that really stops the world
  with the world visible behind it; resume quick and obvious; settings
  include brightness calibration, sensitivity, invert, FOV, a true off for
  head bob, hold/toggle. Deaths are low-key and the retry immediate
  (Frictional: repeated death kills fear).

## The design

**One sentence:** the hands never leave the run, and the frame is the
place; what the screen adds is small, set in the building's own voice, and
gone when it is not needed.

1. **Controls on the run.** One binding table (`session/input.rs`); every
   action a run needs under the left hand or on the mouse; a held key fires
   once; using anything never stops a run; a spent run comes back at 30
   stamina; Ctrl holds a crouch and C toggles it; E (or a left click) uses
   what you face, and an E a moment early still lands; Tab is the notebook,
   V calls out, Escape pauses and resumes.
2. **Three voices in type.** The building speaks Helvetica Neue in mixed
   case (menus, prompts, cards); M.'s notebook is her hand; the torn notes
   are typed (Courier New, where it is a document, not a style). Capitals
   only for a label of one to three words.
3. **The title is a place.** The lobby, live behind a short list at the
   lower left (Continue, New descent, Settings, Quit), keyboard and mouse.
   No paragraph, no keys, no medals. It stands a few metres out along the
   way from where you begin; New descent keeps its place and draws the
   camera back into your eyes, with no cut (`docs/CUTSCENES.md`, the way
   in).
4. **Pause stops the world.** The frozen frame dimmed behind the same
   list; Settings (look, invert, field of view, head bob, film grain, run
   hold/toggle, volume, brightness against a barely-visible mark, pause on
   focus loss) and a Controls page, the only place every key is listed.
5. **An empty HUD at rest.** Meters appear when they change and fade;
   the pockets show on change and while X is held; a dot only on what can
   be used, its verb beside it; medals live in the notebook.
6. **Low-key death and cards.** Black, the hum, one line in the world's
   voice; any key wakes you, once the card is up. The level card is small
   at the lower left; the ending holds.
7. **The notebook in your hands.** Opening it keeps the view and the
   mouse; it sits low on the screen as paper; her pages in her hand, the
   torn notes typed; pages turn with the wheel, the arrows or a click.
8. **A cutscene says your hands are taken** (`docs/CUTSCENES.md`): black
   bars slide in from the top and the bottom, the HUD goes, and a key
   pressed shows "Hold E to skip" small in the lower bar; play itself is
   never letterboxed.

## Milestones

Each ends in its tests and evidence and is committed on its own.

| # | Milestone | Done when |
|---|---|---|
| U0 | This plan; `SOURCES.md` section 9 | committed |
| U1 | Controls on the run (design 1); the engine leaves Escape to the game | `tests/controls.rs`: reach, a held key fires once, running and using things together, no stutter (fails on the old rule: stretches of `[299, 2, 2, 2, …]` frames), crouch hold/toggle, Escape pause/resume, E uses what you face, an early E lands, Command forgets held keys, the notebook pages without a strafe, the cards wait; pacing and personas in their bands |
| U2 | Faces: proportional type in the HUD's text engine; the three voices | a test that every face resolves to itself and that laid-out widths match CoreText's; the HUD frames |
| U3 | The title is a place | menu navigation by keys and mouse, Quit; the title PNG over the live lobby; no controls wall |
| U4 | Pause and settings | pause freezes the simulation and ducks the sound; settings persist and apply (look, invert, FOV, bob, grain, run toggle, volume, brightness, focus pause); the Controls page lists every binding from the table; PNGs |
| U5 | The HUD at rest | meters fade when idle; X shows the pockets; the dot and verb on a target; medals off the HUD; copy in mixed case; the HUD PNG and the goldens re-accepted |
| U6 | Death, level cards, the ending | no stats, any key after the card is up; level card; PNGs; the story and playthrough tests |
| U7 | The notebook in your hands | look and walk while reading; her hand / typed notes; paging keys; PNGs |
| U8 | Ledger, scorecard, playtest questions | `content.json` entries, `QUALITY.md`, `PLAYTEST.md`; the full check |

## Progress

| # | Commit | Notes |
|---|---|---|
| U0 | `718f11e` | the plan |
| U1 | `1951997` | `session/input.rs`: `BINDINGS` (17 actions), `LEFT_HAND`, `acts_for`, `sprint_ready` (a spent run comes back at 30), the C latch (a run stands you up), `interact_target` (the faced one; one under you or alone behind you still counts) and a 0.15 s early-E buffer, `toggle_pause` / `pause` / `resume`, `on_key_up` (Command forgets held keys). The engine no longer releases the pointer on Escape; the app routes mouse buttons as `Mouse0`–`Mouse2` keys while locked and the wheel to `wheel`; recordings skip key repeats and gain wheel events. The Stairwell / Loop recorded runs hold C to crouch: their harness now replays that bit as Ctrl held, and they match frame for frame. `tests/controls.rs` (11 tests); pacing, personas, the playthrough and the replays pass unchanged |
| U2 | `7776876` | `ui/faces.rs`: `Face` (Typed, Sans, SansLight, Hand), PostScript names, CoreText ascent / descent, advances measured per glyph (kerning and optional ligatures off, as drawn) and cached; the font code left `draw.rs` (2188 → 2096 lines). CoreText substitutes a missing face silently (ArialNarrowMT came back as Helvetica in the sampler), hence `every_face_is_the_one_asked_for`. CoreText puts pair kerning back on for any letter spacing but zero, so spaced proportional text is drawn glyph by glyph at its laid-out places. Courier New's metrics and attributes are unchanged: the goldens did not move. Her hand was chosen from ten candidates rendered side by side (Noteworthy Light: a pen, legible; Bradley Hand reads as a marker). `crates/target/faces.png` |
| U3 | `a4c2282` | `ui/menu.rs` (the lists, keys, hover, choices, `live_menu`: a list acts and draws only in its own state) and `ui/screens.rs` (the shade and the list, no panel). The title is the level-1 lobby, live: lights, dust, grain and, in the window, its sound (audio now starts at launch); the camera looks down the longest of sixteen sight lines from the spawn and turns slowly. Continue (with its level), New descent (asks first over a save: "The descent saved on level 4 is lost."), Quit; her last line in her hand under the title. No paragraph (its native draw is kept, so nothing after it moves), no keys, no medals, no buttons: the old title's elements are gone from the model. The first run of the check caught the title's shade over every golden (a headless start skipped the menu's close) and the title's list taking the ending card's E in a test that set the state by hand: both closed by `live_menu` and closing on any start. Recordings gain hover events. `tests/menus.rs` (3), a shared `tests/common` (the placer registry, once); the title PNG over the live lobby and the HUD test's title with a save |
| U4 | `5f08292` | Escape (or losing the window, unless Settings says not to) stops the world, drops the sound to a quarter and puts Paused over the frozen, darkened frame: Resume, Settings, Controls, Leave for the title (asks, on Back; keeps the notebook), a pause line in the world's voice, and the level and seed in a corner. `session/settings.rs`: look speed, invert, field of view (the walking one; running and crouching keep their +7 / −6), head bob (full, reduced, off), camera shake, film grain, the run key (hold, or press until you stop), volume, brightness (a lift or a dim over everything, no shader; a mark a hair above black to set it by), pause on leaving the window; kept under `settings`, apart from the save, clamped when read. Controls lists every binding from the table and rebinds by Enter, then the key; keys swap, Escape stays the pause key. The defaults are bitwise the game as it was (×1.0, 74 + 7): the goldens did not move. The draws under shake stay when it is off. Found on the way: the window reports a lost pointer twice on a focus loss (only a change counts now); "Shift / Shift" on the Controls page; questions opened on their unsafe answer. `tests/settings.rs` (5), `tests/menus.rs` (5); `--screen NAME` renders a list over a level |
| U5 | `4d5278c` | `ui/rest.rs` and a new in-play painter: at rest nothing but the place. Stamina shows while it is not full and a moment after, sanity while it is going or below 60, the light's cell (an outline, not a labelled bar) while it is on or changed, the pockets for 4 s after a change, her thread for 14 s when it is new (in her hand); X (a new binding, held) shows everything with small labels and the level. A ring and the verb only on what you face ("E  Open the door"; the key as bound). Help is offered with its key when it would help (water under 30 stamina, a battery under 20). Medals, the seed, "— CROUCHING —" and the achievement flash left the HUD (the medals live in the notebook). The model keeps its text (recorded fixtures pin it): `calm` sets its capitals in sentence case when painted, keeping initials, I and sign words. The goldens and look anchors were re-accepted (HUD-only diffs; the whole set moved alike: colourfulness −2.0, contrast −0.023), and the STYLE luminance floors re-measured by their rule (the HUD had lifted every view by about 0.01; one golden view fell under its old floor). `tests/rest.rs` (3), the HUD test's rest and glance frames |
| U6 | `dbe4241` | Death: the killer in your face, black, then one line in the world's voice ("The hum never stops.") and Wake up / Leave for the title; no title banner, no seconds survived, no keys. E, Enter, Space, R or a click anywhere wakes you, once the card is up (1.4 s); before, nothing skips it. Leaving after a death builds the floor again first, so the thing that found you is not standing in the title's view. The card's three native draws are kept. The level card: "Level 4" small at the lower left with the floor's line under it, 4 s, on every arrival (new descent, door, lift, hole, Continue, a retry, the Stairwell's and the Loop's way out, whose recorded trace of the old message is kept). Her last card: "The last camp" over black with the three paragraphs and Go on down; the chord dies, then the master fades to silence over 4 s, and comes back below. The old card painter (title, paragraph, fourteen keys, medals, blinking hint) is gone, and its words left the model. `--screen dead` / `ending`; `tests/menus.rs` (death), the story and playthrough tests read the new card; the HUD test's level-card, dead and ending frames |
| U7 | `ffa4014` | `ui/paper.rs`: the notebook is a page low on the screen, as tall as its words, the way ahead still in view; no dimming, no buttons. Opening it keeps the pointer: you turn and walk while reading (A/D strafe; the arrows, the wheel and the mouse buttons turn the pages); the ring and verb hide while you read; leaving the window or pausing puts it away. Her pages on ruled cream paper in her hand; the torn notes typed on a greyer photocopy with a torn top, keeping their capitals; the index, medals and epilogue in the building's voice on plain paper. The old parchment panel and its helpers left `draw.rs` (now 1,692 lines). The index lost "Press NEXT to browse individual entries", the empty notebook its "Press J". `--screen notebook` / `notebook-typed`; `reading_keeps_the_view_and_the_walk` |
| U8 | (this commit) | Two reviews of U1–U7, and what they found fixed. The `code-reviewer`: rebinding Step left / right did nothing (strafing read A/D directly); a trackpad swipe flipped through the whole notebook (the wheel now turns a page per notch of travel); a key's repeat after Command forgot it was a press the recordings missed; J opened the notebook unseen under the pause list (under a list only Mute reaches the game now); leaving for the title from a pause kept the Stairwell or the Loop (and anything chasing you) behind the title (the floor is built again); head bob, shake and press-to-run were not applied in the Stairwell and the Loop; a C crouch, a latched run, an early E or an open notebook carried over into the next life or run (`reset_hands`); the README's last two control rows had fallen out of the table; headless runs read the player's settings (`main.rs` drops them now). The `lore-keeper`: death lines that were tips after the fact ("You should have crouched.") or rules the game lacks ("It remembers you now."), pause lines that stated the Watcher's rule, "NEXT" and "story arcs" and a progress bar in the notebook, the debug key on the Controls page, three names for the light (it is "Light" on every screen), hints that repeated the pickup's noun, and LORE.md's "J". `content.json` gains `ui/*`; `QUALITY.md` grades UI and controls B; `PLAYTEST.md` asks about the hands and the screen. Full check passed |

## What only people can tell

Every gate above holds what can be measured: the keys are in reach, a run
survives using things, the lists answer keys and the mouse, the HUD is
empty at rest, the cards wait for their key, the words are the building's.
What the work is *for* is not measured by any of them, and is the next
playtest's focus (`PLAYTEST.md`, "Hands and screen"):

- Can a fresh player run, look and use things at once without thinking
  about their hands? Do they ever let go of WASD to use something?
- With the HUD empty at rest, do they miss something it no longer shows
  (their stamina, what they carry)? Do they find X on their own?
- Do the title, the pause and the death screen read as the game's, or as a
  menu? (One word each.)
- Is reading the notebook in real time too dangerous, or just right?
- Does anyone open Settings, and for what?
