---
paths:
  - "crates/game/src/cutscene/**"
  - "crates/game/src/debug/cutscene.rs"
  - "crates/game/src/ui/cinema.rs"
  - "assets/src/cutscenes/**"
---

# Cutscene rules

The design is `docs/CUTSCENES.md`; adding one is the `add-cutscene` skill.

- **It borrows your eyes, never your place.** Nothing in a cutscene moves
  your body (position, facing, pitch, crouch); the camera is the engine's
  alone (`cutscene_post`), never set by a cutscene's event. Its last frame
  is the gameplay camera exactly (`then = "back"`), or it holds behind the
  card (`then = "card"`).
- **Play never eases from a cutscene's pose**: `cutscene_pre` puts play's
  camera back before the player's update. A new per-frame camera effect in
  play must be computed from play's own state, not read back from the
  camera.
- **No random draw in the engine** (`cut_engine_draws` must stay 0): its
  sway, walk and a death's shake are functions of its clock
  (`curve::sway`, `curve::jitter`); what a cue draws is the cue's. A death
  keeps the death camera's three draws a frame (`death_world`).
- **Cues run with the cutscene in place**, each once; an event that ends
  it (a death, a new floor) ends the step. Events that must hold on a skip
  or a break-off are `on_skip`; clean-ups go in `finally`.
- **Hands taken means**: no strike or touch (`strike_allowed`,
  `creature_contact`, the staff's and shadows' kills), no tell counted, your
  asides wait, sounds heard from the camera. A cutscene that gives your
  hands back is refused in danger and breaks off at it
  (`cutscene_danger`); nothing breaks off a death.
- **The bundle is compiled**: change the TOML source and
  `cargo xtask cutscene write`; never edit `cutscenes.json` by hand.
- **Every cutscene has stages and keeps the contract on each**
  (`tests/cutscenes.rs`); a change that loosens the contract needs a known
  answer that still fails.
