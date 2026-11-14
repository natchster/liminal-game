# UI and controls: the hands and the frame

How the player moves, uses things and reads the screen, why, and the plan
that took it from "a column of spaced capitals and fourteen keys" to
controls you can use on the run and screens that belong to the place. The
pillars it serves are in [`PILLARS.md`](PILLARS.md); the research behind it
is in [`SOURCES.md`](../research/SOURCES.md) section 9.

## The work

How this was planned and done (the analysis, milestones, progress): [`../plan/UI.md`](../plan/UI.md).

## What the research says

Section 9 of [`SOURCES.md`](../research/SOURCES.md) has the sources; in short:

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
   camera back into your eyes, with no cut (`docs/design/CUTSCENES.md`, the way
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
8. **A cutscene says your hands are taken** (`docs/design/CUTSCENES.md`): black
   bars slide in from the top and the bottom, the HUD goes, and a key
   pressed shows "Hold E to skip" small in the lower bar; play itself is
   never letterboxed.
9. **A recording is yours to start and end.** F9 (on the Controls page,
   off the run's hand: it is not a thing a run needs) or the pause list's
   Record begins one from that moment, on any screen; a small red
   "● REC 1:23" stands at the top right while it runs (the HUD's one mark
   that is not the place's: it says something is kept), and Stop recording
   or F9 again ends it, its name shown a moment where the mark was. Nothing
   in play changes while it records (`docs/engineering/REPLAYS.md`).

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
