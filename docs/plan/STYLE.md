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
| N1 | `d73a01a` | The film grain taken out: its layer, canvas and refresh (and its 144,000 draws a refresh from the frame stream), the *Film grain* setting (a store that has it still loads), the anchor's two fields (a save that has them still continues). `update_grain` is `update_overlays`: the aberration layer's eased opacity and the sanity filter's classes, now every frame (they were swapped only on the grain's refreshes). The light: each frame's mean luminance the same within 0.0001 (seed 4242, levels 2, 5, 7: 0.0146, 0.0769, 0.0665 before; 0.0146, 0.0769, 0.0664 after), the neighbouring pixels' mean difference down 4% (level 2, dark), 29% (5) and 35% (7). `the_screen_lays_no_noise` (`tests/hud.rs`): at the lowest nerve the overlays draw nothing and two frames at rest are the same, pixel for pixel |
| N2 | `eb41d48` | The camera's colour faults taken out. The aberration layer's red and cyan blotches (a lens's fringes) are gone, and the light they took is laid evenly (`NERVE_DIM`, a black fill at 0.016 × the layer's opacity, painted as `nerve`: the notebook's test reads a `fill` as a list's dim): at the lowest nerve over the HUD test's flat stand-in the mean luma 0.26051 with them, 0.26070 without (0.07% lighter; at 0.020 it was 0.4% darker), and the three points where they were and the middle alike, (105, 93, 51), where they were (97, 91, 51), (104, 91, 51), (106, 85, 48). The nerve's hue drift below 20% (hue-rotate swinging by up to 80° about a bias of up to 90°, on the vignette layer) and the static's saturation and contrast punch are gone with their tables (`sanity_hue_by_weather`, `static_chrom_by_weather`, `static_chromatic_t`): they coloured only the vignette's inset glows (a chime's flash, the electric surge's tint), which keep their colour now. Nothing of it drew a number: the frame stream is as N1 left it. `--screen nerve` (your nerve at 5, the overlay eased in): seed 4242 level 5's frame, mean luminance 0.0419 with the blotches (built with them put back), 0.0420 without; the cyan cast on the left wall and the pink on the right gone. `the_nerves_overlay_keeps_its_light_and_lays_no_colour` (fails on the parent: the cast) |
| N3 | (this commit) | The review. **Blind** (`visual-review`; a fresh reviewer given only the pairs and the question "Which of the two looks more like a photograph of a real place, abandoned, and less like a picture made by a camera or a screen?", not `STYLE.md`, which now names the grain): the goldens at 480×300 and full-nerve crops at the screen's own pixels hardly differ by the pairs' measure (no pixel moved by 12 levels: the grain at full nerve was 0.08), so the pairs were seed 4242's levels 5, 7 and 11 at the lowest nerve (`--screen nerve`, the parent of N1 built with only that screen added, against N2) and levels 5 and 7 cropped at full nerve: the new preferred 5 of 5, both orders agreeing ("sensor-style noise… a property of the camera or screen rather than of the surfaces"; at full nerve "a milder grain… still reads as camera noise"). At the lowest nerve the neighbouring pixels' mean difference fell 55–70% (4.17 → 1.41, 3.56 → 1.08, 2.73 → 1.22), the mean luma held (51.94 → 51.92, 45.68 → 45.64, 29.62 → 29.46 of 255), and the linear luminance fell 1.6–2.7% (strong noise's bright specks carry more light than its dark ones take: the grain had lifted it). Seen on both sides, so older: a near-black skirting band, a doorway's black sliver, the pool's water flat with no reflection. **Code** (a fresh `code-reviewer` on `d73a01a` and `eb41d48`): no bug; the every-frame class swap, the saves and anchors with old fields, the settings rows by id, the layer names and the goldens' untouched by N2 held. Fixed: eight stale lines (`SAVES.md`'s settings keys, `VISUAL_PLAYTEST.md`'s look render, the game loop's diagram, `CLAUDE.md`'s example of what a draw shifts, the `visual-playtest` and `visual-review` skills, the loudness reason's true peaks, `update_entity_pressure`'s doc); `the_screen_lays_no_noise` compared two frames between the grain's repaints (it would have passed with the grain back: now eight in a row, the draws counted over the layers' painting too); nothing ran the hue drift's code at a low nerve (`tests/entities_smoke.rs` now holds the vignette unfiltered on every frame, its nerve at 12 from frame 1500) |
