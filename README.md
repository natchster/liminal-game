# The Backrooms

A first-person exploration horror game for macOS on Apple Silicon, written in
Rust. You follow someone down an endless building that is different for
every run: first through the Building, three floors of vast, empty concrete
at many heights over a void, where you find the way down by stairs,
galleries and bridges (`docs/design/MEGASTRUCTURE.md`); then through the yellowed
office rooms inside its walls. M., who went in after E., left a trail of
folded paper, a camp on every floor and a page of her notebook at each.
Nothing points the way; the door is found by its light and the sound it
makes. Keep your sanity and your flashlight battery up, avoid what walks the
halls (taking her page wakes them), don't trust every trail, and go down —
through doors, holes in the floor, a stairwell that goes too deep, a shop
that closes at night, a house that is not yours, and a corridor that loops —
to her last camp on level 13, and on (`docs/design/GAMEPLAY.md`).

- **Rendering:** a Metal renderer with per-fragment Lambert / Phong / unlit
  shading, exponential fog, a place-based light map, floor occlusion and a
  soft highlight roll-off, procedural wear on props, a camera grade and lens
  vignette, skinned and morphing creatures, instancing, 4× MSAA and a
  compositor for the HUD's blend modes.
- **Creatures and pacing:** creatures that see ahead of them and search for
  what they only heard, and a pacing director that sends one after you,
  announces it through the place (its steps, failing lights, then silence)
  and backs everything off after a close call.
- **2D:** procedural textures and the HUD are drawn with CoreGraphics and
  CoreText.
- **Audio:** a fully synthesized soundscape on an audio node graph (oscillators,
  filters, convolution reverb, spatial panners) rendered through CoreAudio;
  every sound comes from somewhere: props from where they stand, a few
  signature sounds per level from fixed places.
- **Platform:** windowing and input through winit on AppKit.
- **Deterministic levels:** a run seed and a level number always build the
  same level, checked against recorded reference levels. Each area of a
  level has one purpose (an office wing, washrooms, a maintenance run, or
  nothing at all), and its wallpaper, grime and darkness change at its
  walls.

## Built with agents, and their tooling built with the game

The game is developed with coding agents (Claude Code), and the tooling
they work with is part of the project and grows with it (`docs/engineering/AGENTS.md`):

- **Custom skills** for each kind of content the game has (zones, props,
  creatures, anomalies, sounds, pacing, voices, re-recording, the weekly
  drift sweep), each ending in the game's own gates and evidence;
- **Custom reviewers**, read-only agents that see one kind of evidence
  each (an art critic on blind A/B frames, a lore keeper, a level analyst,
  an audio analyst, a code reviewer), calibrated on known answers and on
  people's verdicts;
- **Hooks and rules** that hold the game's invariants (determinism,
  recorded fixtures changed only by re-recording, edits kept inside the
  repository, the quick check before an agent stops);
- `CLAUDE.md`, the map an agent reads first.

When a mistake happens twice it becomes a rule, a lint, a test or a skill
step; a new system gets its skill when it is built.

## Build, play and work on it

| Read | For |
|---|---|
| [`docs/engineering/BUILD.md`](docs/engineering/BUILD.md) | the requirements, building and running, where things are kept, every command-line option |
| [`docs/design/UI.md`](docs/design/UI.md) | the keys (*The keys*) and the settings |
| [`docs/engineering/DEVELOPMENT.md`](docs/engineering/DEVELOPMENT.md) | the tests and reference data, the checks, the tools |
| [`docs/engineering/ARCHITECTURE.md`](docs/engineering/ARCHITECTURE.md) | the crates and folders, one frame, a level's build |
