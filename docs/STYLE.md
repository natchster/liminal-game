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

Bands per purpose for its zone view (colourfulness on the Hasler–Süsstrunk
scale, mean luminance 0–1), a colourfulness ceiling for every frame and a
luminance ceiling for every level view (creature close-ups, such as the
Watcher's sheet filling the frame, may be brighter). They were set
around the first golden of each purpose (2026-10-25, one view each), so they
are wide; the expressive-range report (`cargo xtask range`) measures many
views per purpose and is where they get tightened. Change them only when the
look of a purpose is meant to change, and say why in the commit.

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
      "colourfulness_max": 27,
      "luminance": [
        0.17,
        0.37
      ]
    },
    "office wing": {
      "colourfulness_max": 26,
      "luminance": [
        0.13,
        0.33
      ]
    },
    "showroom": {
      "colourfulness_max": 21,
      "luminance": [
        0.11,
        0.31
      ]
    },
    "living quarters": {
      "colourfulness_max": 18,
      "luminance": [
        0.0,
        0.18
      ]
    },
    "maintenance": {
      "colourfulness_max": 19,
      "luminance": [
        0.0,
        0.15
      ]
    },
    "overgrown atrium": {
      "colourfulness_max": 20,
      "luminance": [
        0.1,
        0.3
      ]
    },
    "party room": {
      "colourfulness_max": 31,
      "luminance": [
        0.08,
        0.28
      ]
    },
    "play area": {
      "colourfulness_max": 31,
      "luminance": [
        0.19,
        0.39
      ]
    },
    "pool hall": {
      "colourfulness_max": 16,
      "luminance": [
        0.05,
        0.25
      ]
    },
    "service tunnels": {
      "colourfulness_max": 15,
      "luminance": [
        0.0,
        0.17
      ]
    },
    "shrine": {
      "colourfulness_max": 32,
      "luminance": [
        0.19,
        0.39
      ]
    },
    "storage hall": {
      "colourfulness_max": 16,
      "luminance": [
        0.0,
        0.16
      ]
    },
    "washrooms": {
      "colourfulness_max": 21,
      "luminance": [
        0.06,
        0.26
      ]
    },
    "parlour": {
      "colourfulness_max": 25,
      "luminance": [
        0.13,
        0.33
      ]
    }
  }
}
```
<!-- style-bands:end -->
