# Style

The art bible: how the Backrooms look, written so that most of it can be
checked. The canon is [`LORE.md`](LORE.md) and the pillars it serves are
[`PILLARS.md`](PILLARS.md) (2, *familiar but wrong*, above all). The numbers at
the end are read by `cargo xtask shots`: a zone view whose look leaves its
purpose's band fails, so this file and the gate cannot disagree.

## The look in one paragraph

Real buildings, photographed badly: worn materials under weak fluorescent
light, colour that has faded toward yellow-grey, grime gathering near the
floor, and real darkness between the lit parts. Nothing is clean, nothing is
saturated, nothing is lit from the player. A place is lit by its own fixtures
(the light map) and goes dark where they have died.

## Rules for every surface

- **Albedo** (linear, brightest channel) between 0.008 and 0.87 for
  non-metals; aim for 0.03–0.8 when authoring (black plastic ≈ 0.02, fresh
  concrete ≈ 0.5, fresh snow ≈ 0.81). Metals are dark in diffuse and carry
  their colour in the highlight. *Gate:* `tests/materials.rs`.
- **Saturation** at most 0.985 ((max − min) / max of the effective albedo);
  props pass through the look settle (`visuals/prop_look.rs`), which mutes
  chroma above 0.3 and keeps albedo in 0.02–0.8. Red paint and safety yellow
  are allowed to be strong; toy primaries are not.
- **Wear:** untextured lit surfaces get the shader's wear; grime gathers
  near the floor.
- **Light** comes from places: panels, tubes, lamps. Never ambient tied to
  the camera, never a lamp cone that follows the player except the
  flashlight itself.
- **The camera:** a mild grade and lens vignette, film grain rising as sanity
  falls. No bloom, no lens flares.

## Type (the screen's three voices)

What the screen says is set in one of three voices, each with a reason
(`crates/game/src/ui/faces.rs`, `docs/UI.md`):

- **The building:** Helvetica Neue, in mixed case, small, left-aligned.
  Menus, prompts, cards. Institutional type makes the uncanny routine.
- **M.:** her hand (Noteworthy Light), on her notebook pages and her
  thread line.
- **Typed paper:** Courier New, for the torn notes and the debug panel, and
  nowhere else.

Capitals only for a label of one to three words; no letter-spaced
sentences; no boxes, borders or glows around text. *Gate:*
`tests/faces.rs` (every face is the one asked for; a laid-out line is as
wide as CoreText draws it).

## Archetypes (the building)

| Archetype | Materials | Ceiling | Motif | Allowed deviation |
|---|---|---|---|---|
| Lobby | mono-yellow wallpaper, wet carpet | 2.7–3.05 m, tubes over every other tile | thin partitions full of gaps, lone pillars | the odd step in the ceiling |
| Pillar Field | lobby wallpaper | 3.2–3.7 m | a lattice of columns | none: monotony is the point |
| Abandoned Office | drywall, carpet tiles | 2.5–2.75 m | small rooms, one door each, chest-high cubicles | debris, a toppled chair |
| The Maze | 4 m walls | varies | braided corridors | collected things: toys, portraits, candles |
| Utility Halls | poured concrete | 2.35–2.65 m | shoulder-wide corridors, walls that bulge in | industrial debris |
| Electrical Station | old brick | low | tight, tangled, wiring behind every wall | sparks, dead panels |
| Habitable Zone | poured concrete, puddles | 5.4–7.6 m, fixtures on rods | wide column grid | long reverb, wet floors |
| Poolrooms | white tile everywhere | varies | too many pillars | water that is too still |
| Blackout | whatever was there | — | dead lights | only the flashlight |
| The Well (the Building, level 1) | panel concrete, a window in every panel | none: galleries over galleries, the void | walks round a well, sealed doors, a few lit windows | laundry across the void, a red-marked door |
| The Stair Halls (the Building, level 2) | rusticated stone, flagstones | none: the dark | piers, flights at every height, bridges | chains out of the dark, flights nobody reaches |
| The Works (the Building, level 3) | poured concrete, steel | a slab far up, its fittings dead | terraces stepping down past shafts, machine housings | gantries, pipes |

## Purposes (what the place was for)

Each area's purpose decides what is left in it. Its props tell one story;
the purpose's band (below) holds how bright and how colourful its view is.

| Purpose | What is left | Wayfinding | Threat |
|---|---|---|---|
| empty | nothing: the emptiness is the content | the lights, the exit sign | yes |
| office wing | desks, chairs, paper, screens | cubicle rows, corridors | yes |
| living quarters | beds, couches, kitchens, personal things | doors left open | yes |
| play area | toys, small furniture, drawings | colour, scattered toys | yes, sparingly |
| parlour | old furniture, frames, clocks | carpets, lamps | yes |
| washrooms | tile, stalls, sinks, mirrors | the tile, the drip | yes |
| overgrown atrium | plants grown through the building | green, light from above | yes, sparingly |
| maintenance | pipes, tools, machines | pipes and arrows | yes |
| shrine | candles, offerings, symbols | candlelight | the place itself is the threat |
| party room | balloons, streamers, a cake nobody ate | the mess | yes |
| storage hall | racking, boxes, pallets | aisles | yes |
| service tunnels | cables, pipes, maintenance doors | the hum | yes |
| pool hall | water, tile, ladders | the water | no creature needed (Pools has none) |
| showroom | room-sets, price tags, trolleys | floor arrows (lit only) | the staff, by night |

## Creatures

- **At a distance** a creature is a silhouette and its eyes; at 8 m it must
  read as *something standing there*, not as a model.
- **Close up** the detail is for horror: skin, teeth, the wrong joints.
- **Eyes** catch the light the way eyes do (eyeshine), never glow like
  lamps, except where the canon says so.
- *Gate:* the creature goldens at 2, 4 and 8 m and their death scenes.

## The numbers

Bands per purpose for its zone views (colourfulness on the Hasler–Süsstrunk
scale, mean luminance 0–1), a colourfulness ceiling for every frame and a
luminance ceiling for every level view (creature close-ups, such as the
Watcher's sheet filling the frame, may be brighter).

They are measured, not guessed: `cargo xtask range --look` rendered 631 zone
views (four runs, levels 1–6) and each band is the 2nd to 98th percentile of
its purpose's views, widened by 2 (colourfulness) and 0.02 (luminance). A
purpose with fewer than 12 views of its own takes the band of all views.
`cargo xtask shots` holds the golden zone views to them, and
`cargo xtask range --look` fails a purpose when fewer than 70% of its views
lie inside. One thing the measurement showed: **how dark a view is depends on
its region's light far more than on its purpose** (an office wing runs from
0.05 to 0.35); the purposes differ in colour and clutter, not in brightness,
except the showroom (always lit) and storage halls (mostly dark). Change the
bands only when the look of a purpose is meant to change, re-measure, and say
why in the commit. (The luminance floors were re-measured on 2026-10-27, by
the same rule rounded outward, when the HUD went to rest (`docs/UI.md` U5):
its always-on bars and capitals had lifted every view's mean by about 0.01.)
The Building's three places (`docs/MEGASTRUCTURE.md`) were measured by the
same rule when they came in (16 views each, B3; again when their lamps,
railings and own air came in, B4): far darker than the rooms
(their light is lamps and windows over a void, not fixtures on every other
tile), and greyer.

<!-- style-bands:begin -->
```json
{
  "all": {
    "colourfulness_max": 30
  },
  "levels": {
    "luminance_max": 0.42
  },
  "purposes": {
    "empty": {
      "colourfulness_max": 26,
      "luminance": [
        0.03,
        0.33
      ],
      "views": 220,
      "from": "own views"
    },
    "living quarters": {
      "colourfulness_max": 27,
      "luminance": [
        0.02,
        0.32
      ],
      "views": 59,
      "from": "own views"
    },
    "maintenance": {
      "colourfulness_max": 25,
      "luminance": [
        0.05,
        0.33
      ],
      "views": 25,
      "from": "own views"
    },
    "office wing": {
      "colourfulness_max": 25,
      "luminance": [
        0.04,
        0.35
      ],
      "views": 106,
      "from": "own views"
    },
    "overgrown atrium": {
      "colourfulness_max": 26,
      "luminance": [
        0.03,
        0.34
      ],
      "views": 10,
      "from": "all views (too few of its own)"
    },
    "parlour": {
      "colourfulness_max": 26,
      "luminance": [
        0.03,
        0.34
      ],
      "views": 3,
      "from": "all views (too few of its own)"
    },
    "party room": {
      "colourfulness_max": 26,
      "luminance": [
        0.03,
        0.34
      ],
      "views": 6,
      "from": "all views (too few of its own)"
    },
    "play area": {
      "colourfulness_max": 26,
      "luminance": [
        0.03,
        0.34
      ],
      "views": 7,
      "from": "all views (too few of its own)"
    },
    "pool hall": {
      "colourfulness_max": 22,
      "luminance": [
        0.09,
        0.42
      ],
      "views": 24,
      "from": "own views"
    },
    "service tunnels": {
      "colourfulness_max": 23,
      "luminance": [
        0.02,
        0.29
      ],
      "views": 74,
      "from": "own views"
    },
    "showroom": {
      "colourfulness_max": 19,
      "luminance": [
        0.19,
        0.27
      ],
      "views": 40,
      "from": "own views"
    },
    "shrine": {
      "colourfulness_max": 26,
      "luminance": [
        0.03,
        0.34
      ],
      "views": 1,
      "from": "all views (too few of its own)"
    },
    "stair halls": {
      "colourfulness_max": 13,
      "luminance": [
        0.06,
        0.19
      ],
      "views": 16,
      "from": "own views"
    },
    "storage hall": {
      "colourfulness_max": 21,
      "luminance": [
        0.02,
        0.26
      ],
      "views": 38,
      "from": "own views"
    },
    "washrooms": {
      "colourfulness_max": 25,
      "luminance": [
        0.05,
        0.33
      ],
      "views": 18,
      "from": "own views"
    },
    "well": {
      "colourfulness_max": 12,
      "luminance": [
        0.06,
        0.17
      ],
      "views": 16,
      "from": "own views"
    },
    "works": {
      "colourfulness_max": 12,
      "luminance": [
        0.07,
        0.16
      ],
      "views": 16,
      "from": "own views"
    }
  }
}
```
<!-- style-bands:end -->
