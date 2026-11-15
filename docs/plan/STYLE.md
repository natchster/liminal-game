# The work on the look

The plan behind [`../design/STYLE.md`](../design/STYLE.md), which says what the game looks like and why: the brief,
the analysis, milestones, progress. What is still open is queued in [`MILESTONES.md`](MILESTONES.md).

## The brief (2026-11-16): the noise taken out

GitHub issue #14, *Remove noise* (a maintainer's): "Change look & feel from noisy/grained to more photorealistic.
Do not change light level outside the environment lighting. Focus on visual feeling more photorealistically
abandoned than bad-camera-recording".

Read as: take out what makes the screen a bad camera's recording of the place (noise over it, a camera's colour
faults) and leave every light level as it was: the grade, the lens's falloff, the vignettes, the fades and
flashes, the nerve's darkening. Only the place's own lighting (the light map, its fixtures) may change how light
it is. The gate for the light: each frame's mean luminance as before, within what the noise itself moved it.

## The analysis

Read from the code (`ui/hud.rs`, `ui/draw.rs`, `visuals/scene_fx.rs`, `entities/ai.rs`,
`engine/src/composite.metal`) and from frames rendered before and after.

| What | Where | A camera's? | Its light |
|---|---|---|---|
| The film grain: a 480×300 canvas of random greys (each row mixed 30% with the one above) stretched over the view, overlay-blended at 0.08 + 0.28 × the nerve lost, repainted every 55 ms (144,000 draws from the frame stream each time) | `update_grain`, the `#grain` layer, `Settings::grain` | yes: the issue's "noisy/grained" | none: overlay over a mean grey of one half keeps each pixel's mean |
| The aberration layer: two colour blotches (red at 68% 58%, cyan at 32% 42%, multiply) and a red edge, eased in as the nerve goes (at most 0.95), spiked by a whisper (0.98) and a glitch (0.85) | `update_grain` (its opacity), `#aberration` | its fringes, named for a lens's | the red edge darkens (the nerve's) |
| The nerve's hue drift: below 20%, hue-rotate swinging with a 4.4 s period on the vignette layer | `entities/ai.rs` | a tape's colour drifting | none |
| The static's punch: saturation and contrast kicked for 0.35 s when the static crackles | `entities/ai.rs`, `audio/ambient/weather.rs` | an interference | contrast |
| The grade: a little less saturation, a gentle S-curve over lifted blacks, the lens's falloff | `composite.metal` `grade` | a photograph's too | is the light level: kept |
| The material wear (value noise at the rest-pose position) | `shaders.metal` | no: the surfaces' own, fixed to them | kept |
| The scene's sanity filter (saturation down, a little contrast) | `visuals/scene_fx.rs` | no: colour going out of the world | kept |

## Milestones

| # | Commit | What |
|---|---|---|
| N1 | (this commit) | The film grain taken out: its layer, canvas and refresh (and its 144,000 draws a refresh from the frame stream), the *Film grain* setting (a store that has it still loads), the anchor's two fields (a save that has them still continues). `update_grain` is `update_overlays`: the aberration layer's eased opacity and the sanity filter's classes, now every frame (they were swapped only on the grain's refreshes). The light: each frame's mean luminance the same within 0.0001 (seed 4242, levels 2, 5, 7: 0.0146, 0.0769, 0.0665 before; 0.0146, 0.0769, 0.0664 after), the neighbouring pixels' mean difference down 4% (level 2, dark), 29% (5) and 35% (7). `the_screen_lays_no_noise` (`tests/hud.rs`): at the lowest nerve the overlays draw nothing and two frames at rest are the same, pixel for pixel |
| N2 | | The camera's colour faults: the aberration layer's fringes, the hue drift, the static's punch; their light kept |
| N3 | | The review: a blind A/B of the goldens before N1 and after N2 ("which is a photograph of a real abandoned place, not a recording"), a code review, `PLAYTEST.md`'s question |
