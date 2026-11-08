# Barricades: how they work, how much they fill, real ones, and a proposal

Research for [`../PROPS.md`](../PROPS.md) (2026-10-31), on the request "May barricades more filling the space": how the barricades are chosen, built and collide, how much of the opening they fill (measured on seeds 4242, 77 and 202, levels 1–9, with `--dump-level` and `--views-at`), what real barricades in abandoned buildings are made of and how big their parts are, what the canon says, and a proposal. The renders and the measuring script were working files and are not kept; the code is cited by file and line as it was at `9df0d3b`.

## 0. The short version

1. **Barricades are chosen on the tile grid, which ignores most real walls.**
   `tail_barricades` picks a random open tile that has two open neighbours on
   opposite sides (`tail.rs:1367-1384`). That rule dates from when every wall
   was a solid 4 m tile. In the thin-walled zones (Lobby,
   Pillar Field, Office, Hall, Showroom, Poolrooms) a wall tile is only a
   partition along its centre line (`archgen.rs:1757-1799`). Half of the
   neighbouring tile is open floor, so blocking one tile only closes a real
   **doorway**.
   - In my sample, **28 of the 53 barricades stand in open floor**: beside a partition, or between lone pillars 7 m apart. A body walks round them. All 28 are in thin zones; 1 thin-zone barricade was a true doorway.
   - The 24 barricades in solid-walled zones (Maze, Utility Halls, Electrical) do seal their corridor.
2. **The collision is an invisible 4 × 4 m square.** The visible pile is about 1.25 m wide, 0.8 m deep and 1.8 m tall: one filing cabinet (0.6 × 0.55 × 1.4 m), a book stack, 2–4 cardboard boxes and a traffic cone.
   - The body is stopped over the whole tile (`world/mod.rs:270-318`), so you stop 1.5–2 m short of the pile, with clear floor on both sides that you cannot enter.
   - In a 4 m corridor the pile covers a median **32 %** of the width and **20 %** of the opening's face. In thin zones it covers 19 % of the width and 10 % of the face.
3. **Sight, sound and the creatures' paths all go straight through barricades.**
   - `los` reads only the solids (`world/mod.rs:246`).
   - `bfs` reads only the tiles (`world/mod.rs:203-241`). It drives the player flow field (`entities/ai.rs:249`), the creatures' hearing distance (`entities/creature.rs:351-362`) and the place's answer to a cry (`player.rs:345-372`).
   - Only her walk (`story/mod.rs:156-168`) and the test bot avoid barricades.
4. **Existing bugs:**
   - Duplicate tiles: there is no check, so 4 of the 18 rooms levels I dumped barricade the same tile twice. The core props' tail fixture has `[479, 479]` too.
   - Pickups locked away (`PICKUPS_BEHIND_BARRICADES`, `level_sweep.rs:72-77`).
   - The four pieces are separate, unprotected `place_props` objects, so placement pushes them apart (boxes and cone end up 0.6–0.7 m off).
5. **Canon:** no doc says what barricades are or who built them. There is a lore decision to make (§4).
6. **Proposal (§5):**
   - **Placement:** put barricades only in real openings (thin-partition doorways, or corridors between solid walls), measured on the solids.
   - **Kinds:** four, by purpose: boarded doorway / hoarding, office pile, storage stack, domestic. Each is built at real sizes to fill the opening face to face, up to the lintel or near the ceiling, with real-sized gaps.
   - **Collision:** a slab spanning the measured opening, not the whole tile.
   - **Sight and sound:** sight blocked; creatures' walking and hearing blocked.
   - **Light:** the light map stays as it is.
   - **Determinism:** dress from a salted SplitMix64. Keep today's `rnd()` draw count with a draw-only replica of the old code (or accept a full re-record).
   - **Assets:** a new Blender batch of real-size parts.

---

## 1. How barricades work now

### Where it runs

- `place_props_tail` calls `kind!(self, "barricades", self.tail_barricades(cx))` (`crates/game/src/core_props/tail.rs:125`).
  - Before it: the floor hole and the elevators.
  - After it: wall mirrors, tableaux and graffiti (`tail.rs:126-128`), then every themed placer (`world/build.rs:443-445`), then `settle_props` (`build.rs:450`) and `settle_placement` (`build.rs:464`).
- `kind!` (`core_props/mod.rs:11-18`) tags each new world child with `s["kind"] = "barricades"` (`world/build.rs:75-80`).
- `tag_placed` (`build.rs:430`) sets its placer to `place_props`.
- Levels 1–3 (the Building) had no barricades in any dump.

### How tiles are chosen (`tail.rs:1361-1394`)

- **Attempts:** up to 4 (`attempts = 4`).
- **Each attempt:** up to 30 tries of `irand(2, w-3)`, `irand(2, h-3)` (two level-stream draws per try).
- **A try is accepted** when the tile is open and has exactly 2 open 4-neighbours on opposite sides (`:1377-1380`).
- **Distance from spawn:** more than 6 m from `spawn_pos`. The spawn is tile (1,1) on rooms levels (`world/build.rs:322`).
- **Solvability:**
  - `s_alt_path_exists(self, 1, 1, exit tile, [candidate] + barricade_tiles)` (`tail.rs:1386-1393`, through `core_props/seam.rs:127-132`) must hold.
  - That calls `Game::alt_path_exists`, a 4-neighbour BFS over the tile grid (`world/mod.rs:321-350`).
  - If it fails, the attempt is spent (`continue`).
- **No duplicate check:** `barricade_tiles.push(blocked_idx)` (`:1394`) can push a tile that is already there.
  - Seen in the dumps on 202 L6, 202 L7, 4242 L8 and 77 L6.
  - The core props' tail fixture (now `crates/game/tests/fixtures/core_props_tail.json`) holds `"barricades": [479, 479]`, so the bug is as old as that recording.
- **No zone, purpose or archetype filter.**
- **Frequency:** across levels 4–9 of seeds 4242, 77 and 202, 57 entries (53 unique tiles) on 18 levels, about 3 a level.
  - Purposes: empty 13, living quarters 12, service tunnels 9, showroom 6, office wing 5, others 8.
  - Archetypes: maze 10, lobby 9, tunnels 8, pillars 8, store 6, electrical 6, pool 3, hall 2, office 1.
- **Records:**
  - `barricade_tiles: Vec<usize>` (`base/game.rs:178-179`), cleared per build (`world/build.rs:190`).
  - Dumped as `"barricades"`, tile indices `z*W + x` (`debug/hooks.rs:736`).

### What it builds (`tail.rs:1395-1412`)

Four separate top-level groups at the tile centre:

| Piece | Builder | Size / parts | Placement |
|---|---|---|---|
| filing cabinet | `build_filing_cabinet` (`core_props/builders.rs:34-178`) | `Props__cabinet_body` 0.60 × 1.40 × 0.55 m, top, 4 feet, 3 drawers. Each drawer: front, pull, lock, label, a full box interior and random folders. 56 meshes in the dump. Materials `g.mats.metal`, `dark`, `chrome` plus lambert greys. | tile centre, yaw `rand1(2π)` (`:1399`) |
| book stack | `build_book_stack` (`builders.rs:895-947`) | 3–6 books, sometimes one standing (`Props__book_*`) | on the cabinet top at (+0.1, 1.42, +0.05) |
| cardboard stack | `build_cardboard_stack` (`builders.rs:180-235`) | 2–4 boxes, 0.4–0.6 m square × 0.32 m (`Props__cardboard_box/flap/tear`), hand-written labels | ±0.4 m from the centre |
| traffic cone | `build_traffic_cone` (`builders.rs:636-651`) | `Props__cone_*`, 0.34 m base, 0.58 m tall, orange | ±0.6 m, tilted ±0.3 rad |

- The cabinet's drawers are built with contents (battery, almond or paper rolls, `builders.rs:144-163`), but `cab.drawers` is dropped. They draw RNG and are never usable.
- **Settling** treats the pieces as ordinary clutter:
  - Each is its own world child with placer `place_props`. `settle_placement` protects only gameplay objects and live themed props (`world/settle.rs:145-151`; `protect` in `world/placement.rs:336-338`, step 3 `:393-...`).
  - So the boxes and cone are shoved out of the cabinet. In 4242 L6 at tile (18,37), the boxes ended 0.70 m off-centre and the cone was lifted and moved; the dump shows `dpos 0.7` and `-0.54, -0.24`.
- **The real-object test fails:**
  - The cabinet is oversized. A real 4-drawer letter cabinet is 0.38 W × 0.67 D × 1.32 H m; this one is 0.60 W × 0.55 D × 1.40 H with three tall drawers.
  - Every pile is the same four things, centred and randomly rotated in the middle of the passage, touching neither wall.
  - The cone is a marker, not a barricade (the same category as police tape).

### Collision, sight, sound, light

**Body.** `circle_hits_wall` (`world/mod.rs:270-318`) blocks the whole tile.
- With solids: `:290-302`; the grid path: `:305-316` ("treat barricade tiles as walls").
- Each test clamps the point into the full `CELL` square (`:271-275`; `CELL = 4.0`, `base/config.rs:2`).
- Users:
  - the player
  - the vacuum (`player.rs:673`)
  - creature movement (`entities/mod.rs:527-531`, `entities/ai.rs:33-44`, `watcher.rs:366`, `director.rs:444,766`)
  - placer clearance checks (`core_props/mod.rs:136-139`)
  - `free_buried_pickups`, which moves a pickup out of a barricade tile (`settle.rs:96-130`)
  - the level sweep

**Sight.** `los` (`world/mod.rs:246-266`) tests `archgen::segment_clear` at 1.5 m on the solids (or wall tiles). Barricades are not in the solids, so creatures (`e_los`, `entities/mod.rs:521-526`) see you through them, and the glass creature too (`glass.rs:211,224,608`).

**Sound and paths.** `bfs` (`world/mod.rs:203-241`) walks `tiles` only.
- The player flow field (`entities/ai.rs:245-250`) and `path_distance`, "how far your steps carry" (`entities/creature.rs:348-362`), go through barricades. Creatures hear you through one and steer into its invisible tile wall, because their steering uses the same field (`ai.rs:69-110`).
- The ambient reach (`audio/ambient/mod.rs:620`) uses `bfs`; muffling (`:382`) uses `los`.
- **The place's answer to a cry** (`way_to_the_door`, `player.rs:345-372`, used at `:404-470`) follows `bfs` from the exit. The lights or footprints can lead through a barricade.
- Things that do avoid barricades:
  - her walk: `story_open` refuses barricaded tiles (`story/mod.rs:156-168`)
  - the secrets: `story_walk_dist` (`secrets/mod.rs:331-332`)
  - the playthrough bot (`crates/backrooms/tests/playthrough.rs:143-147`)

**Light.** The light map has one value a tile (`visuals/lighting.rs:324-482`), and the corner shape reads only the solids (`visuals/occlusion.rs:58-80`). Barricades neither stop nor make light, which is correct under "light belongs to places".

### Rules and tests around barricades

- **`docs/DEVELOPMENT.md:204-218`:**
  - Barricades keep only spawn → exit solvable.
  - About 6 levels in 20 have rooms behind them.
  - Open issue: a barricade can lock a pickup away (about 1 level in 20).
  - The same is in `docs/QUALITY.md:36` and `docs/TOOLING_ROADMAP.md:22`.
- **`crates/backrooms/tests/level_sweep.rs`:**
  - Flood fill on a 0.25 m grid with `circle_hits_wall` (`:9-20`, `:107-157`).
  - `PICKUPS_BEHIND_BARRICADES = true` (`:72-77`).
  - A pocket is "behind a barricade (by design)" only if every sealing tile is a barricade tile (`:214-262`; `sealed_by` = open neighbour tiles with no free cell).
  - Pinned cases in `ALWAYS` (`:327-332`): 1161854898 L7 and 928196632 L4.
- **`crates/backrooms/tests/reference_levels.rs:137-139`** compares `barricades`; `:140-...` compares the placed objects placer by placer, including `place_props`.
- **`crates/game/tests/core_props_tail.rs:513-517`** compares `barricade_tiles` with its fixture.
  - The fixture had no rerecord function then (`rerecord` writes it now).
  - The test also replays the recorded builder log: name, arguments, draw index and draws used (`:1-15`).
- **`crates/game/tests/world_queries.rs:96,105,261,267`** loads the fixture barricades into the world queries.
- **`crates/game/tests/core_props_head.rs:75`** treats a barricade as blocked.
- **`docs/GAMEPLAY.md:107` (G1):** the build must not move barricades after a note is read.

---

## 2. How much of the opening a barricade fills now

### Openings by archetype

From `level/archgen.rs:171-345` and `build_solids_view`: thick bulges `:1738-1756`, thin partitions `:1757-1799`, jambs and lintels `:1818-1872`:
- The jamb is drawn per zone with `jamb_chance` (`:888`).
- `lintel_h = min(ceil − 0.25, lintel)` (`:889`).
- Jambs and a lintel exist only in thin zones whose zone has `jamb > 0` (`:1825`).

| Archetype | Walls | Opening across the passage | Top of the opening |
|---|---|---|---|
| Maze | solid 4 m tiles | 4.0 m | ceiling 2.95–3.3 m |
| Utility Halls | thick, bulging 0.55–1.15 m each side (`:1738-1756`) | 1.7–2.9 m (LORE says 1.6–2.8) | ceiling 2.35–2.65 m |
| Electrical | thick, bulging 0.3–0.8 m | 2.4–3.4 m | 2.6–3.1 m |
| Lobby doorway | thin 0.16–0.26 m | 4 − 2 × (0.8–1.35) = **1.3–2.4 m** in 55 % of zones; else 4 m | lintel 2.15–2.4 m; else ceiling 2.7–3.05 m |
| Office doorway | thin 0.12–0.18 m | 4 − 2 × (1.1–1.3) = **1.4–1.8 m** (90 %) | lintel 2.05–2.15 m |
| Hall doorway | thin 0.35–0.5 m | 2.0–2.8 m (40 %); else 4 m | lintel 3.2–3.6 m; else ceiling 5.4–7.6 m |
| Pool doorway | thin 0.3–0.4 m | 2.2–3.0 m (30 %); else 4 m | lintel 2.6–3.0 m; else 3.4–4.6 m |
| Pillar Field, Showroom | thin, no jambs | 4 m between run ends | ceiling 3.2–3.7 m / 4.4–4.9 m |

### Measured on real levels

Script: `analyse_barricades.py`. It replicates `build_solids` for the 5 × 5 tiles round each barricade, without anchors, protrusions or leaning walls, and floods a 0.25 m grid with r = 0.35 m round the blocked tile.

| | count | free width at the tile's centre line | pile width across (m) | pile height (m) | width filled | face filled (pile w × h ÷ opening w × ceiling) | body walks round? |
|---|---|---|---|---|---|---|---|
| corridor between solid/thick walls (maze, tunnels, electrical) | 24 | 4.0 m (an upper bound in tunnels and electrical, whose bulges narrow it) | 0.78–1.76 (all barricades) | 1.57–1.95 (all) | 25–44 %, median 32 % | median 20 % | no: sealed |
| thin zone, true doorway | 1 | 4.0 m | | | 32 % | 20 % | no |
| thin zone, not a doorway (beside a parallel partition, between pillars, at a corner) | 28 | 5.6–7.7 m | | | 13–27 %, median 19 % | median 10 % | **yes, all 28** |

All piles: width median 1.25 m, height median 1.79 m. 34 of 53 piles have 4 pieces.

Three tile-map examples (`B` barricade, `#` wall tile, `.` open):

```
4242 L4 (9,10), lobby          77 L4 (30,22), pillar field    202 L9 (47,40), lobby
  ..#...#                        #.#.#..                        .#...#.
  ..#...#                        .......                        .#.....
  #.#B##.   left: a partition    #.#B#..   lone pillars:       .##B##.   a true doorway:
  ..#...#   along z on x=8.5;    .......   7 m of free floor   .....#.   sealed
  #.#...#   1.9 m strip beside B #.#.#..                        .#...#.
```

**The collision against the pile.**
- The tile is 4 m deep along the passage, so the body stops 2.35 m from the tile centre.
- The pile's near face is 0.3–0.8 m from the centre, so you stand 1.5–2 m from it.
- Clear floor about 1 m wide shows on either side (see `now_closest_reach_…`).

### Rendered views

Views from `--views-at` at 3.5 m from the tile centre (1.5 m outside the tile), straight on or oblique:

| File | Level | What it shows |
|---|---|---|
| `now_lobby_4242L4_t9-10.png` | 4242 L4, tile (9,10), lobby / living quarters | a cabinet, books and boxes in a wide room; the partition end on the left, open floor on the right (a leak) |
| `now_lobby_open_4242L4_t31-29.png` | 4242 L4, tile (31,29) | the cabinet's back in an open room, the EXIT door visible beyond it |
| `now_maze_4242L5_t23-34.png` | 4242 L5, tile (23,34), maze / office wing | a 4 m corridor: about 1.3 m of pile, open floor and 1.4 m of air on both sides |
| `now_maze_4242L6_t18-37.png` | 4242 L6, tile (18,37), maze / party room | the same; a cone beside it |
| `now_closest_reach_maze_4242L6_t18-37.png` | same | from 2.36 m, the nearest a body can stand: about 1 m of clear floor to the right that you cannot enter |
| `now_tunnels_4242L6_t7-36.png` | 4242 L6, tile (7,36), utility hall | the narrow corridor is half filled; free above and to the right |
| `now_pillars_77L4_t30-22.png` | 77 L4, tile (30,22), pillar field | a pile standing alone among pillars |
| `now_office_77L8_t20-33.png` | 77 L8, tile (20,33), office wing | a pile in an open office floor |
| `now_tunnels_duplicate_202L6_t33-7.png`, `now_tunnels_duplicate_77L6_t9-27.png` | duplicated tiles | two cabinets in one spot |

The HUD's zone caption is visible in the shots ("The lobby — Level 0"); it is the zone caption, not part of the barricade.

---

## 3. What real barricades in abandoned buildings look like

The sources I read directly are marked (read). The rest are from search-result summaries and their figures should be checked before use.

### Boarded doorways and windows: the official vacant-building method

**USFA/FEMA *Board Up Procedures*, April 2018 (read):**
<https://www.usfa.fema.gov/downloads/pdf/publications/board-up-procedures.pdf>

- **Materials:**
  - **1/2-inch (4-ply) CDX exterior plywood.**
  - **2-by-4 × 8 ft braces.**
  - **3/8-inch × 12-inch carriage bolts**, the round head on the weather side, a 1/2-inch washer outside and a 3/8-inch washer inside.
  - 6d (1 5/8-inch) ring-shank nails.
  - No particle board or Masonite.
- **Fitting:**
  - Plywood is cut to fit over the opening, flush with the outside of the molding or trimmer stud, and nailed every 12 inches round the perimeter.
  - **Door: the door is removed and stored inside.**
  - **Three brace sets per door:** one at the centre of the doorway, one halfway from the centre to the top, one halfway from the centre to the bottom. Each set is a brace outside and one inside, bolted through.
  - Bolt holes are at one third of the brace's length from the frame.
  - The exterior is painted the building's colour.
- **What this means for the look:** the two faces differ. The nailed face shows plywood, bolt heads and braces; the other face shows the second set of braces and the nuts. You can tell which side it was closed from.

### Construction hoarding and temporary walls

- CITB, *Temporary works: hoardings*: <https://www.citb.co.uk/media/xepdxp5u/temporary_works_hoardings.pdf>. Hoardings are "most commonly about 2.4 m" high and sheeted in plywood.
- A UK supplier specification (Four Seasons Fencing): 150 × 75 mm posts, three 100 × 47 mm rails and **2440 × 1220 × 18 mm** plywood. <https://fourseasonsfencing.co.uk/fencing-ashford/timber-stock/site-hoarding/>
- **What this means for a 4 m opening:** a stud wall of 2x4s with 3.3 sheets across and 2.44 m high, leaving a strip to the ceiling. Above it you see the far side's ceiling and light.

### Lumber

- A 2x4 is actually **1.5 × 3.5 in, 38 × 89 mm**: <https://www.inchcalculator.com/actual-size-of-dimensional-lumber/>
- A plywood sheet is 1220 × 2440 mm (4 × 8 ft), as in the hoarding specification above.

### Furniture barricades

**History.** Wikipedia, *Barricade* (read): <https://en.wikipedia.org/wiki/Barricade>. Barricades are improvised from what is at hand: "stones, furniture, and other debris" (Paris, 1830); "vehicles, furniture, and rubble" (Berlin, 1945). Interiors were defended floor by floor, with stairs ripped out (Warsaw).

**Lockdown guidance** for blocking a door with furniture (search summary):
- UConn: <https://emergencymanagement.universitysafety.uconn.edu/emergency/hazard-guide/lockdown>
- University of Houston: <https://uh.edu/police/safety-security/active-shooter/lockdown-deny>
- What it says:
  - Wedge the door, then pile the heaviest, largest items against it.
  - For a door opening outward, tie the handle to heavy furniture, then add more in front.
- **The pattern:** flat, heavy pieces go against the opening first, filling goes behind them, and wedges and ties hold it.

**Chained doors** are a common code violation: padlocked panic hardware, chained pairs of exit doors (search summary, <https://idighardware.com/2013/09/ww-panic-hardware-protection/>). The rooms have no door leaves in their openings, so this kind needs a door first; I leave it out.

**Not barricades:** police tape and a lone traffic cone. Both are markers.

**Photo references.** I did not find a documented photo of an urbex hallway barricaded with office furniture. The searches returned AI-generated stock images. I give no composition from a photograph.

### Real dimensions for the pieces

| Thing | Size | Source |
|---|---|---|
| plywood sheet | 1.22 × 2.44 m; 12.7 mm (USFA) to 18 mm (hoarding) | USFA (read); Four Seasons |
| 2x4 | 38 × 89 mm, 2.44 m long | inchcalculator |
| interior door leaf | 0.61–0.91 × 2.03 m (24–36 × 80 in); hollow core 35 mm (1 3/8 in) | <https://www.bobvila.com/articles/standard-size-for-doors/>; slab example 28 × 80 × 1-3/8 in: <https://www.dkhardware.com/masonite-2-4-lau-28-in-w-x-80-in-h-lauan-wood-interior-hollow-core-door-slab-product-8316350.html> |
| 4-drawer letter filing cabinet | 0.38 W × 0.67 D × 1.32 H m (15 × 26.5 × 52 in); depth 0.48–0.67 m by model | <https://Quill.com/hon-310-series-4-drawer-vertical-file-cabinet-locking-letter-black-265d-h314pp-next2019-next2day/cbs/017255.html>; <https://www.findofficefurniture.com/guides/vertical-file-cabinets.html> |
| steel (tanker) office desk | 1.52 × 0.76 × 0.74 m (60 × 30 × 29 in) | <https://www.remodelista.com/products/classic-steel-tanker-desk>; <https://www.globalindustrial.com/p/open-desk-60-x-30-charcoal-top-black-base> |
| task chair | about 0.62 W × 0.64 D × 1.02–1.12 H m (24.5 × 25 × 40–44 in) | Herman Miller Viv: <https://www.hermanmiller.com/en_mx/resources/3d-models-and-planning-tools/product-models/individual/viv-high-back-side-chair-with-arms-4-star-swivel-base/> |
| stacking chair | 0.53 × 0.56 × 0.81 m (21 × 22 × 32 in) | <https://www.cymax.com/Full-Back-Contoured-Stacking-Chair-in-Black-SC-459754.htm> |
| GMA pallet | 1.219 × 1.016 m (48 × 40 in), 0.14–0.15 m high, 15–22 kg | <https://www.interlakemecalux.com/blog/gma-pallet>; <https://www.rehrigpacific.com/supply-chain/products-and-solutions/40-x-48-gma-rackable-2400-pallet> |
| boltless steel shelving | 0.91 × 0.46 × 1.83 m (36 × 18 × 72 in); 1.22 × 0.61 × 1.83 m (48 × 24 × 72 in) | <https://www.uline.com/Product/Detail/H-7115/Industrial-Shelving/Boltless-Shelving-36-x-18-x-72>; <https://www.uline.com/Product/Detail/H-8407/Industrial-Steel-Shelving/Heavy-Duty-Boltless-Shelving-48-x-24-x-72> |
| mattress | twin 0.97 × 1.91 m (38 × 75 in); full 1.37 × 1.91 m (54 × 75 in); 0.15–0.35 m thick | <https://www.tomsguide.com/features/twin-bed-size-how-big-is-a-twin-mattress> |
| cardboard archive box | about 0.38 × 0.31 × 0.25 m (15 × 12 × 10 in) | common size from memory, **not sourced** |

### Concrete compositions (real practice, scaled to the game's openings)

**A. Boarded doorway (USFA method).**
- Plywood cut to the opening plus 50 mm onto each jamb, nailed at 0.3 m centres.
- Three 2x4 braces across, at ¼, ½ and ¾ of the height, bolts at a third of their length.
- For an office doorway (1.6 m wide, lintel 2.1 m): one full sheet (1.22 m) plus a 0.48 m ripped strip, a 2.1 m cut height, and a gap of 3–6 mm at the joint.
- See-through: under 2 % (the joint, a split or kicked-in low corner of about 0.25 m).

**B. Hoarding across a 4 m gap.**
- 2x4 sole and head plates, studs at 0.61 m, four 2.44 m sheets (the last ripped).
- Under a 2.7–3.05 m ceiling: a 0.26–0.6 m strip of air above, or a ripped sheet to the ceiling grid.
- The back shows the studs.

**C. Office pile in a 1.4–1.8 m doorway under a 2.05–2.15 m lintel.**
- A steel desk stood on end (1.52 h × 0.76 w × 0.74 d) against one jamb.
- A 4-drawer cabinet upright (1.32) against the other jamb; a second cabinet on its back across the foot (0.38 high).
- A task chair upside down on top, a stacking chair wedged under the lintel, archive boxes in the low gaps.
- Depth 0.7–1.0 m. Gaps 5–15 cm between legs; 10–30 cm under the lintel.

**D. Storage stack in a 1.7–3.4 m tunnel.**
- A 0.91 × 0.46 × 1.83 m shelving unit across the tunnel (on its side when the tunnel is 1.7–1.9 m wide, or two upright).
- Two or three pallets on edge leaning on it (1.22 or 1.02 m tall).
- Archive boxes 4–6 high (1.0–1.5 m); a plywood sheet leaning over the top to close the upper part.

**E. Domestic.**
- Twin mattresses on edge (1.91 tall × 0.97 wide) across the opening, a dresser on its back at the foot.
- A door leaf taken off its hinges (0.81 × 2.03 m) nailed across with two 2x4s.
- Chairs seat-up on top.

---

## 4. Canon

**What the docs say about barricades.**
- Nothing about what they are or who made them. `docs/LORE.md`, `docs/PILLARS.md`, `docs/STYLE.md` and the canon parts of `docs/GAMEPLAY.md` never mention barricades or blocked ways.
- They appear only in engineering notes:
  - `GAMEPLAY.md:107` (G1, determinism)
  - `DEVELOPMENT.md:204-218`, `QUALITY.md:36`, `TOOLING_ROADMAP.md:22`, `ARCHITECTURE.md:53`

**What the canon says that bears on them:**

- **How lore may reach the player** (`LORE.md:26-64`). Props show what a place was for, "never labelled beyond the zone caption". Notices exist only on the Building's floors (`LORE.md:42-46`). Painted words on a barricade would need the lore-keeper.
- **Emptiness.**
  - "The Lobby … is rooms and nothing else. Clutter belongs deeper, where the place has started collecting things" (`LORE.md:84-87`).
  - "Thin partitions stand in open floor, full of gaps … almost nothing else. The emptiness is the content" (`LORE.md:209-213`).
  - PILLARS rule: "the Lobby and the `empty` purpose stay sparse", tested by the range report's prop density (`PILLARS.md:38`).
- **The Maze** "is where the Backrooms keeps what it has collected" (`LORE.md:219-222`).
- **Purposes decide what is left** (`LORE.md:239-244`; the table in `STYLE.md:75-96`): office wing is desks, chairs, paper; storage hall is racking, boxes, pallets; maintenance and service tunnels are pipes, tools, cables; living quarters are beds, couches; the showroom is "spotless" (`LORE.md:549-553`).
- **Real places, real doorways, real materials.** "Props look like real objects, not toys" (`PILLARS.md:57`).
- **People who came before.**
  - "Lore notes are torn pages left by those who came before: Elias, M., the field surveys, the Stairwell logs" (`LORE.md:671-674`).
  - "Earlier explorers left blue markers every ten landings" (the Stairwell, `LORE.md:656-657`).
  - The Building's residents are "gone behind their doors" (`LORE.md:24`, `:156`).
  - The code's graffiti pool also speaks for wanderers ("DAY 47", "I MADE IT TO LEVEL 3", `tail.rs:37-75`).
- **Her.**
  - In code, her walk goes round barricaded tiles (`story/mod.rs:156-168`).
  - The docs say she leaves marks and camps (`LORE.md:246-320`), nothing about building or meeting barricades.
- **Hounds** "hear your steps along the way they would walk to you, through doorways and not through walls" (`LORE.md:345-347`). The code's hearing ignores barricades (§1), which disagrees with this rule once a barricade truly closes a doorway.

**Who built them.** The docs do not say. Readings the canon allows, for the lore-keeper to choose (none is canon):

1. Those who came before, closing a room or a way they feared. This fits the furniture piles.
2. What the building was before the place took it: a floor closed off for refurbishment or vacancy. This fits plywood board-ups and hoarding ("familiar but wrong": real practice, no reason given).
3. The place's collecting (the Maze). Weak: nothing in the canon has the place build anything.

Nothing supports her as the builder. Whichever is chosen, it must be shown only by the objects, with no text.

---

## 5. Design proposal

### 5.1 Where barricades go

**Replace the tile test** (`tail.rs:1367-1384`) with a test for real openings:

- **Doorway in a thin partition:** the jamb condition of `build_solids` (`archgen.rs:1818-1872`).
  - An open tile between two wall tiles of thin zones, along one axis.
  - Pass-through on the other axis.
  - Neither side is a low partition (`TF_LOW`).
  - Both side tiles are the ends of one straight run.
- **Corridor tile between two solid or thick wall tiles.** The bulges count; the solids include them.
- **Measure the opening on the provisional solids** (built in `gen_layout_level`, `world/mod.rs:353-367`). They already include leaning walls.
  - Width: `archgen::ray_axis` (`archgen.rs:2247`) from the tile centre, both ways across the passage, at y = 0.3, 1.0 and 1.8 m (as `face_delta`/`settle_probe` do, `settle.rs:206-243`).
  - Top: the lintel box's y0 if the tile has a `BOX_LINTEL`, else `tile_ceil`.
  - Anchors are added later, in `settle_props`. They only fill a partition's side away from the doorway gap, so the measure holds.
- **Keep the existing rules and add four:**
  - Keep: spawn → exit stays solvable (`alt_path_exists`).
  - **No duplicate tile.**
  - **No pickup cut off**: every almond, battery, paper pile and lore note must stay reachable from (1,1) with the blocked tiles. All are placed before the barricades (`tail.rs:115-116` and the head). This closes `PICKUPS_BEHIND_BARRICADES`; set it to false in `level_sweep.rs:77`.
  - **Not within reach of the spawn** (keep > 6 m).
  - Prefer doorways into rooms or dead ends over through-routes, so "a room behind a barricade" is what the player meets. Today about 6 levels in 20 have rooms behind barricades.
- **Count:** keep "up to 4". Every rooms level in the sample has plenty of real openings: 5–32 thin doorways or 39–245 solid-corridor tiles.
- **No barricades:**
  - in the Showroom ("spotless", and an anomaly's own rules)
  - in `pool hall` purpose zones ("no creature needed")
  - in the Building (already true)
- **The Lobby (level 4, `lobby`/`empty`):** none, or at most one plain board-up in a doorway, to keep the sparseness rule. **Decide with the lore-keeper.**

### 5.2 Kinds

Built at real sizes, fitted to the measured opening. Each fills the opening to within 2 cm of the faces and from the floor to within 5–30 cm of the lintel or ceiling, but never above ceiling − 0.05 m (§5.3).

| Kind | Where (archetype / purpose) | Pieces (real sizes) | Fill and gaps |
|---|---|---|---|
| **K1 boarded doorway** | utility halls and electrical (maintenance, service tunnels); hall doorways; the one Lobby case if allowed | plywood 1.22 × 2.44 × 0.018 m cut to the opening plus 50 mm each side, on the room face of the partition (in front of the jambs, not in them); 3 × 2x4 braces 38 × 89 mm at ¼, ½, ¾ height; carriage-bolt heads; the far side shows the braces and nuts. Wide openings (4 m, or no jambs): 2x4 stud hoarding, studs at 0.61 m, sheets 2.44 m high plus a ripped strip to the lintel or ceiling | 98 % of the face; joints 3–6 mm; perhaps one kicked-in low corner (≤ 0.3 m); a strip above 2.44 m left open where the ceiling is over 3.05 m |
| **K2 office pile** | office wing; Abandoned Office (doorways 1.4–1.8 m under a 2.05–2.15 m lintel); maze office wing | steel desk on end (1.52 × 0.76 × 0.74); 4-drawer cabinet upright (0.38 × 0.67 × 1.32) plus one on its back at the foot; task chair upside down on top; stacking chair wedged under the lintel; archive boxes in the low gaps | about 90 %; gaps 5–15 cm between legs; 10–30 cm under the lintel; depth 0.7–1.0 m |
| **K3 storage stack** | storage hall, maintenance, service tunnels (tunnels 1.7–2.9 m, electrical 2.4–3.4 m) | shelving 0.91 × 0.46 × 1.83 on its side or upright; 2–3 GMA pallets on edge (1.22 × 1.02 × 0.14) leaning on it; archive boxes 4–6 high; a plywood sheet leaning over the top | about 85–90 %; slats leave 5–8 cm gaps; top gap ≤ 0.3 m |
| **K4 domestic** | living quarters, parlour, party room, shrine | 1–2 twin mattresses on edge (0.97 × 1.91 × 0.2); a dresser on its back (0.9 × 0.45 × 0.8); a door leaf off its hinges (0.81 × 2.03 × 0.035) nailed across with two 2x4s; chairs seat-up on top | about 90 %; soft gaps at the mattress edges |
| **K5 tall hall** | Habitable Zone (lintel 3.2–3.6 m, ceiling 5.4–7.6 m) | K1 hoarding with a second row of half sheets to the lintel; with no lintel, 2.44 m plus a strip, open above | full to the lintel; open above when there is no lintel |
| (empty, maze otherwise) | empty purpose in deeper levels; maze "collected things" | K1 for empty (no clutter); maze: K2 or K4 by the zone's purpose | |

**Real-looking rules for every pile:**
- Heavy, flat pieces against the opening first.
- Things upended: desks on end, chairs legs-up.
- Pieces lean 3–15° and interlock.
- None centred-and-spun; no cones; no neat pyramids.
- Debris at the foot on the near side.
- The far side looks different (the back of a pile; the braces of a board-up).

### 5.3 What it means for the code

1. **A record per barricade.** Add `Barricade { tile, kind, axis, span: [x0, z0, x1, z1] (world), top }` next to `barricade_tiles` (`base/game.rs:178-179`). Keep `barricade_tiles` for the tile-path users (`alt_path_exists`, `story_open`, the playthrough bot, the dump, the fixtures).

2. **Collision blocks the whole opening, and only it.**
   - In `circle_hits_wall` (`world/mod.rs:290-302` and `:305-316`), replace the whole-tile test with a span test (the pattern of `hits_cell_door`, `world/assembly.rs:317-323`).
   - The span runs face to face across the measured opening and is as deep as the pile (0.5–1.2 m). You can then walk up to the pile, and no gap of 2r = 0.7 m remains.
   - **Alternative:** append the span to the solids as a box of a new kind. Precedents: `BOX_VOID` "stops a body but not a look" and `BOX_FLOOR`, `archgen.rs:45-53`. It would be appended in `build_solids_leaning`, so collision, `los`, `ray_axis`, placement's `WallGrid` and the corner shape all see it.
     - Wall drawing only draws `BOX_WALL`/`BOX_ANCHOR` of wall tiles (`world/arch.rs:214-221`).
     - It is more consistent, but it changes what the archgen, world_queries and world_build fixtures read, and the solids must be rebuilt after `tail_barricades`.

3. **Settle and placement.**
   - Build each barricade as **one group** at the tile centre (not at the origin, or `settle_props` splits it, `settle.rs:252`), turned to the passage axis. At the centre no wall face is within 0.8 m, so nothing attaches (`settle.rs:322-361`).
   - Flag it `user.gameplay = true`:
     - It joins `gameplay_objects` (`settle.rs:84`) and becomes protected in placement: pushed out of walls at most 0.25 m (`placement.rs:336-338`), skipped by step 1b, and other props yield to it (step 3, `:393`).
     - Themed props that land in it are nudged out.
   - Keep pieces at least 2 cm off wall and jamb faces (`DEFAULT_TOL` = 0.015 m, `placement.rs:172`), or put boards on the room face of the partition.
   - Keep its top ≤ ceiling − 0.05 m. Otherwise `settle_props` raises the tile's ceiling to fit it (`settle.rs:369-395`, the `raised` stat), and a step would appear in the ceiling.
   - With `user.arch = true` instead, settle and placement skip it entirely (`settle.rs:249`, `:145-147`), but then nothing keeps themed props out of it.

4. **Sight.**
   - `los` (`world/mod.rs:246`) should fail across a barricade whose solid band covers 1.5 m. That is every kind; K5 is open only above 2.44 m.
   - Then creatures cannot see you through it, and ambient muffling (`audio/ambient/mod.rs:382`) treats it as a wall.
   - The visible gaps are dressing: light leaks through them, no gameplay sight.

5. **Sound and paths.** Follow the Hound canon ("along the way they would walk to you", `LORE.md:345-347`):
   - Make the player flow field barricade-aware: `e_bfs` in `entities/ai.rs:249` (`entities/mod.rs:515-520`). Creatures then neither path toward nor hear you through a barricade; `path_distance`, `creature.rs:351-362`, reads that field.
   - Make `way_to_the_door` (`player.rs:345-372`) avoid barricades too, so the place's answer to a cry never points into one.
   - Change only these runtime fields, not `Game::bfs`. The build calls `bfs` for creature ecology and placement (`entities/mod.rs:334`, `corner.rs:337-340`, `glass.rs:736-740`, `reader.rs:616-617`) and for the ambient reach (`ambient/mod.rs:620`). Changing it would shift levels.

6. **Light.** No change: the light map is per tile and place-based (CLAUDE.md invariant 2). A barricade added to the solids (the alternative in item 2) would darken its own corners through `visuals/occlusion.rs:58-80` if its kind counts as "full". That is optional and physically right.

7. **Level sweep** (`crates/backrooms/tests/level_sweep.rs`).
   - With a slab, the barricade tile has free cells on both sides. `pockets()` (`:214-262`) would then file the far side as "no way in on the tile grid" and fail.
   - Classify a pocket as behind a barricade when removing the barricade tiles from the tile grid is what disconnects it.
   - Add a per-barricade **seal check**: a local flood fill from one side of the slab must not reach the other side. Today's code would fail it on 28 of 53 sites.
   - Keep the exit-reached and pickup checks, and set `PICKUPS_BEHIND_BARRICADES = false` once the choice checks pickups.
   - Update `docs/DEVELOPMENT.md:204-218` and `docs/QUALITY.md:36`.

8. **Draws and determinism** (CLAUDE.md invariant 1; `.claude/rules/world.md`).
   - **Today's level-stream draws in `tail_barricades`:**
     - 2 per try (up to 30 a try loop, 4 loops).
     - Per built barricade: the filing cabinet (per drawer: `rnd()<0.6`, then the redrawn `irand(2,4)` bound, plus `irand` colour and 2 × `rand2` per folder, plus a content `rnd`; then dent `rnd` (+2), sticker `rnd` (+`irand` +2)), `rand1` yaw, the book stack, the cardboard stack, 2 × `rand2`, and the cone's 3 × `rand2`.
     - The count depends on the values drawn.
   - **What follows on the same stream:** wall mirrors, tableaux, graffiti, then every themed placer. A changed count re-rolls almost all of a level's dressing.
   - The box labels' `handwritten_tex` goes through `canvas_tex_opt`, which draws on its own stream (`placers/kit.rs:48-51`). It adds nothing to the count.
   - **Option A (recommended; keeps the rest of the level):**
     - Keep the old selection loop. Add a draw-only replica of the four old builders: the same `rnd/irand/rand2/rand1/choice` calls in the same order, no meshes, its own local tile list for the old `alt_path` checks. The level stream then leaves `tail_barricades` exactly where it did.
     - Choose the new sites and dress them from `SplitMix64::new(current_seed, BARRICADE_SALT ^ tile)` (`base/rng.rs:171-195`; precedents `SECRET_SALT`, `secrets/mod.rs:32`, and `STORY_SALT`, `story/mod.rs:39`).
     - Guard with a test: for many (seed, level), `seeded_draws()` after the replica equals the old code's count. The tail fixture's builder log also records the draws each builder used.
     - The diff is then confined to the barricades, plus props whose placers test collision near them, plus anything placement nudges. Review it in the re-recorded reference levels.
   - **Option B:** drop the old draws, re-record everything, and say so in the commit.
   - **Either way:**
     - The reference levels change: `barricades` and the `place_props` objects (`reference_levels.rs:137-160`).
     - `core_props_tail.rs` breaks: it replays the recorded builder log, and the barricade tiles and meshes are in the world subtree it compares. It needs either a rerecord path, which its fixture lacked then (it has `rerecord` now), or a seam that stubs `tail_barricades` and checks only its draw count.
     - `world_queries` loads the fixture barricades (`world_queries.rs:96,261`). Re-record it if the meaning of `circle_hits_wall` changes.
     - Goldens and the range baseline: re-accept them if a zone view shows a barricade.
     - New canvas textures (plywood grain) must use `canvas_tex` (its own stream).

9. **Cost.**
   - Today's cabinet alone is 56 meshes, most of them invisible drawer interiors.
   - Merge each barricade's static pieces by material into one mesh each, as the vending machine does (`core_props/vending/model.rs:1-10`): about 4–6 draws a barricade.
   - Use the existing materials (`g.mats.wood`, `metal`, `dark`, `chrome`, `cardboard`) and lambert colours. Add no texture reads to global shader patches (CLAUDE.md invariant 6).
   - New materials must pass `tests/materials.rs` (albedo 0.008–0.87, saturation ≤ 0.985; `STYLE.md:17-26`).

10. **Ledger, tools and docs.**
    - Add a `docs/content.json` entry `props/barricades` (kind "prop with rules", like `props/vending` and `props/windows`), with checks such as `test:backrooms/level_sweep`, `test:backrooms/materials` and a new `test:backrooms/barricades` (seals, fits, never over the ceiling, never on her walk).
    - For evidence, `--prop-census N` frames every kind, labelled `core_barricades` (`debug/hooks.rs:68-82`; the kind tag from `kind!`), along with `--views-at`.

### 5.4 The Blender parts needed

**How the pipeline works:**
- Batches are `assets/src/blender/props/*.py`. Each does `from propkit import *` and registers parts with `part(name, frame, build, smooth=35, budget=None)` (`assets/src/blender/props.py:38-45`).
- One part is one material. Hard edges: `rbox`, `bevel`, `cut`; soft forms: `subdivide`, `noisy`. The kit reference is in `props/README.md`.
- Budget by size: up to 1800 triangles past 1.2 m of diagonal; over 125 % is an error (`propkit.py:605-618`; checks in `props.py:120-140`).
- Randomness in a part is seeded by its name (`props.py`: `random.seed(zlib.crc32(name))`), so builds repeat.

**What a new batch should look like:**
- **Name:** e.g. `x01.py`, any unused prefix. Files starting with `zz_` are skipped in full builds (`xtask/src/assets.rs:277`).
- **Pattern:** follow `v01.py` (vending) and `w01.py` (windows).
  - No map file.
  - Parts modelled at their real size in their own frame and registered with `auto(name, build, budget)`, which uses `f_bounds(lo, hi, 'box')` from the shape's own bounds (`v01.py:209-216`).
  - The game places them at scale one with `prop_part_as_modelled(name)` (`crates/game/src/assets/props.rs:407-429`).
  - Cut-to-fit parts (plywood, studs) use `pg(name, box_geo!(w, h, d))`, which fits the part to any size (`props.rs:368`).
- **Build and check:**
  - `cargo xtask assets props --batch x01` builds in Blender and compares.
  - `cargo xtask assets props --write` (a full build) writes `crates/game/assets/prop_meshes.bin` and `MANIFEST.json` (`xtask/src/assets.rs:244-356`).
  - `REGEN=1 cargo test --release -p game --test asset_lint` regenerates `assets/parts.tsv` (`asset_lint.rs:100-110`).

**Parts.** Real sizes in metres; one material each:

| Part | Size | Notes |
|---|---|---|
| `Barricade__plywood` | 1.22 × 2.44 × 0.018 | chamfered edges, a slight bow, ply layers visible on the edge; fitted, so a cut sheet is a scale |
| `Barricade__plywoodBroken` | 1.22 × 1.22 × 0.018 | a split, kicked-in corner (for the low gap) |
| `Barricade__stud` | 0.038 × 0.089 × 2.44 | eased edges, end grain; fitted for length |
| `Barricade__boltHead` | Ø 0.035 × 0.012 | carriage-bolt dome on a 1/2-inch washer (the nailed face) |
| `Barricade__nut` | Ø 0.03 × 0.012 | nut and washer (the far face) |
| `Barricade__deskBody` | 1.52 × 0.74 × 0.76 | steel tanker desk: two pedestals, modesty panel, toe kick; painted steel |
| `Barricade__deskTop` | 1.52 × 0.03 × 0.76 | laminate top with a bullnose or T-moulding edge |
| `Barricade__deskDrawer` | 0.38 × 0.15 × 0.02 | drawer front with a recessed pull |
| `Barricade__fileBody` | 0.38 × 1.32 × 0.67 | 4-drawer letter cabinet shell. Or reuse `Props__cabinet_*` fitted to real size through a resized builder, which also drops the hidden drawer interiors |
| `Barricade__fileDrawer` | 0.36 × 0.30 × 0.02 | drawer front, label holder, pull |
| `Barricade__taskChairBase` | Ø 0.65 × 0.12 | 5-star nylon base with caster forks |
| `Barricade__taskChairColumn` | Ø 0.05 × 0.3 | gas column |
| `Barricade__taskChairSeat`, `__taskChairBack` | 0.5 × 0.1 × 0.48; 0.46 × 0.55 × 0.08 | upholstered, soft |
| `Barricade__caster` | Ø 0.05 | twin wheel |
| `Barricade__stackChairFrame`, `__stackChairShell` | 0.53 × 0.81 × 0.56 | steel tube frame; polypropylene shell |
| `Barricade__pallet` | 1.219 × 0.14 × 1.016 | GMA: 7 top deck boards (89 mm), 3 notched stringers (38 × 89), 3 bottom boards; worn edges, slight noise |
| `Barricade__shelfPost` | 0.035 × 1.83 × 0.035 | slotted angle post |
| `Barricade__shelfDeck` | 0.91 × 0.04 × 0.46 | steel shelf with lips |
| `Barricade__archiveBox`, `__archiveLid` | 0.38 × 0.25 × 0.31 | hand holes, dented; or reuse `Props__cardboard_box` fitted |
| `Barricade__mattress` | 0.97 × 0.2 × 1.91 | `slab` + `subdivide` + `noisy`, quilting, piped border, a sag |
| `Barricade__doorLeaf` | 0.81 × 2.03 × 0.035 | hollow-core leaf with a 54 mm lock bore and hinge mortises; or reuse `WallDoors__slab` fitted to real size |
| `Barricade__dresser` (later) | 0.9 × 0.8 × 0.45 | domestic |
| `Barricade__sofa` (later) | 2.0 × 0.85 × 0.9 | domestic |
| `Barricade__cableReel` (optional) | Ø 0.9 × 0.6 | electrical |

**Reusable now:**
- `Props__cardboard_box/flap/tear`
- `Props__cabinet_*` (resized)
- `Props__chair_*` (a wooden chair, used by `secrets/dress.rs:675`)
- `MetalLockers__body`
- `WoodenCrates__body`
- `WallDoors__slab`
- `SmallLadders__rail/rung`

The parts list has no office desk, task chair, pallet, plywood, 2x4, mattress or steel shelving (`crates/game/assets/parts.tsv`).

### 5.5 Decisions for the user or lore-keeper

1. **Who built them** (§4): those who came before, the building's last owners, or left unsaid.
2. **The Lobby:** no barricades, or one board-up at most.
3. **Determinism:** Option A (draw-preserving replica) or Option B (full re-record).
4. **Collision:** a span check in `circle_hits_wall` and `los` (smaller change), or barricade boxes in the solids (one source of truth for collision, sight, placement and corners, but more fixtures change).
