# The prop census (2026-10-31)

Which props still look like toys, cartoons or placeholders, found by
looking at every one of them, for [`../PROPS.md`](../PROPS.md).

## How

`backrooms --dump-level DIR --seed S --levels … --screenshot x
--prop-census N` (`Game::census_shots`) frames up to N of each kind of
placed prop on each level built: a core prop by its kind (`core_<kind>`,
tagged by `Game::tag_kind` as `place_props` runs), anything else by its
placer. The camera stands off the prop's face where it can (else round
it), clear of the walls and in sight of it, far enough for it to fill
about half the frame, its eye lowered for a small thing as one crouches
to it; in a dark place the lamp is on. `census.tsv` records each shot's
camera, and `--shots-at` renders the same cameras again, so a remodelled
prop is shot where the old one was.

Two runs: 10 seeds (4242, 77, 7, 202, 303, 1999, 11, 99, 555, 31337) over
levels 4–14, then 18 seeds (1–20 less 7 and 11) over levels 6–14 for the
themes that only the Maze welcomes (occult, victorian, nature): 272 floors
in all. They saw 652 kinds: every core prop, and 606 of the 649 themed
placers (the 43 never placed are listed below; most are victorian or
occult, the Maze's alone).

One shot of each kind (the one with the most detail, by PNG size) went
into contact sheets of six, and seven fresh reviewers, each given the
`art-critic` brief verbatim and this question
([`props-census/questions.md`](props-census/questions.md)), graded every
cell `real`, `plain`, `cartoon` or `unseen`, the evidence first. Their
reviews are kept in [`props-census/`](props-census/) (`review1.md` …
`review7.md`), and every kind's grade, how many of the 272 floors had it,
its theme and its sheet in [`props-census/grades.tsv`](props-census/grades.tsv).

## What they found

213 of 652 kinds are `cartoon` (a third), 412 `plain`, 21 `real` and 6
`unseen`. By how often a floor has one: 35 are everyday (on 40 or more of the
272 floors), 66 common (10–39) and 112 rare (under 10).

What makes them read as toys, in the reviewers' words, falls into a few
patterns:

- **Bodies of primitives.** Animals and dolls of a sphere on an egg with
  dot eyes (the crows, owls, black cats, bears, elephants, the deer's
  head), machines of a box and a cylinder (the sewing machine, the
  hairdryer, the vacuum cleaner), a pipe that is a box with a stub.
- **Icons where a picture or print should be.** A clock face with no
  numerals, a portrait that is a restroom-sign figure, a map of yellow
  blobs, a still life of glossy spheres, black bars for text, a
  biohazard sign of three rings, clip-art suns and flowers.
- **Toy primaries.** Rainbow xylophones, pinwheels, red-green-blue
  books, bottle caps like game counters.
- **Cloth as slabs.** Coats, aprons, towels and washcloths as flat
  boards, with no shoulders, folds or drape.
- **Hard-edged decals.** Stains, mould, rust streaks, chalk, peeled paint
  and water streaks as polygons and ellipses with hard edges; a cobweb
  as a flat triangle; a crack as one even black line.
- **Light from what is not a light.** A crystal skull and a cauldron that
  glow green and light the floor, cats and owls with lamps for eyes
  (`STYLE.md`: eyes catch the light, never glow).
- **Proportions and joins.** An umbrella with a short canopy on a long
  shaft, a desk on stilts, parts that do not meet (a mannequin's arms, a
  teddy's paws, a tricycle's wheel).

And over all of them: almost nothing is worn. Even the `plain` and
`real` props are clean (the shader's wear, `visuals/prop_look.rs`, a
mottle of ±14% and grime in the bottom half metre, is too faint to read).

They also found things broken that are not about the look: the floor
hole standing up out of the carpet as a black half-cylinder, a
mannequin's arms apart from its body, a TV's antenna floating, leaf chips
under a hanging pot, a pinwheel and a spinning top balanced on their
points, record sleeves standing on edge, the inside of an overturned
bucket flat black, text running off a poster and a diploma.

## The cartoon kinds

By how often a floor has one (of 272); `sheet` and `rev` find the
reviewer's evidence in `props-census/review<rev>.md`.

| Kind | Floors | Theme | Sheet | Rev |
|---|---|---|---|---|
| `place_scrap_papers` | 272 | core | sheet063 | 3 |
| `core_lore_notes` | 272 | core | sheet004 | 4 |
| `core_mannequin` | 272 | core | sheet004 | 4 |
| `place_wall_stains` | 272 | core | sheet094 | 4 |
| `core_plant_zone` | 272 | core | sheet005 | 5 |
| `core_wet_floor_sign` | 271 | core | sheet007 | 1 |
| `core_wall_mirrors` | 270 | core | sheet007 | 1 |
| `core_pile` | 264 | core | sheet005 | 5 |
| `core_wall_clock` | 261 | core | sheet007 | 1 |
| `core_book_stack` | 259 | core | sheet001 | 1 |
| `core_suitcase` | 253 | core | sheet006 | 6 |
| `core_coat_rack` | 251 | core | sheet002 | 2 |
| `core_baby_doll` | 247 | core | sheet001 | 1 |
| `core_weather_extras` | 226 | core | sheet007 | 1 |
| `place_secrets` | 213 | ? | sheet063 | 3 |
| `core_rubber_duck` | 201 | core | sheet005 | 5 |
| `place_ceiling_stains` | 198 | core | sheet019 | 1 |
| `place_chalk_marks` | 188 | core | sheet020 | 2 |
| `place_cracked_wall_holes` | 97 | core | sheet025 | 1 |
| `place_rusted_pipes` | 86 | industrial | sheet062 | 2 |
| `place_weathered_posters` | 85 | domestic | sheet096 | 6 |
| `core_floor_hole` | 83 | core | sheet003 | 3 |
| `place_fire_alarms` | 72 | core | sheet033 | 3 |
| `place_ceiling_cobwebs` | 70 | core | sheet018 | 6 |
| `place_trash_cans` | 67 | core | sheet087 | 3 |
| `place_ceiling_cracks` | 67 | core | sheet018 | 6 |
| `place_wall_water_streaks` | 58 | core | sheet095 | 5 |
| `place_umbrellas` | 55 | domestic | sheet089 | 5 |
| `place_single_socks` | 51 | core | sheet066 | 6 |
| `place_crushed_soda_cans` | 48 | core | sheet026 | 2 |
| `place_wooden_crates` | 46 | industrial | sheet098 | 2 |
| `place_bottle_caps` | 45 | core | sheet012 | 6 |
| `place_floor_puddles` | 43 | core | sheet035 | 5 |
| `place_broken_window_frames` | 42 | core | sheet015 | 3 |
| `place_coat_hook_rows` | 42 | domestic | sheet022 | 4 |
| `place_wall_mirrors` | 36 | domestic | sheet093 | 3 |
| `place_wall_phones` | 36 | office | sheet094 | 4 |
| `place_rolled_carpets` | 36 | industrial | sheet060 | 6 |
| `place_floor_rugs` | 32 | domestic | sheet035 | 5 |
| `place_extinguisher_cabinets` | 27 | core | sheet032 | 2 |
| `place_wall_coat_hooks` | 26 | domestic | sheet092 | 2 |
| `place_overturned_buckets` | 26 | industrial | sheet053 | 5 |
| `place_safety_glasses` | 23 | industrial | sheet062 | 2 |
| `place_alarm_clocks` | 23 | office | sheet009 | 3 |
| `place_dust_bunnies` | 23 | core | sheet030 | 6 |
| `place_water_puddles` | 23 | core | sheet096 | 6 |
| `place_tissue_boxes` | 22 | office | sheet085 | 1 |
| `place_small_ribbons` | 21 | domestic | sheet076 | 4 |
| `place_tape_rolls` | 21 | office | sheet084 | 6 |
| `place_small_card_stacks` | 20 | office | sheet067 | 1 |
| `place_dropped_gloves` | 20 | domestic | sheet030 | 6 |
| `place_single_boots` | 20 | domestic | sheet066 | 6 |
| `place_peeled_paint_patches` | 19 | core | sheet056 | 2 |
| `place_wall_handprints` | 19 | core | sheet093 | 3 |
| `place_rubber_balls` | 17 | toys | sheet061 | 1 |
| `place_keychain_keys` | 17 | office | sheet045 | 3 |
| `place_crumpled_receipts` | 16 | office | sheet025 | 1 |
| `place_wine_glasses` | 16 | domestic | sheet097 | 1 |
| `place_toy_cars` | 16 | toys | sheet086 | 2 |
| `place_wristwatches` | 16 | domestic | sheet098 | 2 |
| `place_paperback_books` | 15 | office | sheet055 | 1 |
| `place_wall_brooms` | 15 | domestic | sheet091 | 1 |
| `place_peeling_wallpapers` | 15 | core | sheet056 | 2 |
| `place_newspaper_stacks` | 15 | office | sheet052 | 4 |
| `place_rustic_signs` | 14 | industrial | sheet062 | 2 |
| `place_rust_stain_streaks` | 13 | industrial | sheet061 | 1 |
| `place_school_desks` | 13 | office | sheet063 | 3 |
| `place_firewood_stacks` | 13 | industrial | sheet034 | 4 |
| `place_small_sewing_machines` | 13 | domestic | sheet076 | 4 |
| `place_reading_glasses_and_books` | 13 | office | sheet059 | 5 |
| `place_single_gloves` | 13 | domestic | sheet066 | 6 |
| `place_cross_stitches` | 12 | domestic | sheet025 | 1 |
| `place_mold_patches` | 12 | core | sheet049 | 1 |
| `place_small_pinwheels` | 12 | toys | sheet074 | 2 |
| `place_wall_pipe_leaks` | 12 | industrial | sheet094 | 4 |
| `place_shoe_boxes` | 12 | domestic | sheet065 | 5 |
| `place_shoe_pairs` | 12 | domestic | sheet065 | 5 |
| `place_bread_loaves` | 11 | domestic | sheet013 | 1 |
| `place_fuse_boxes` | 11 | industrial | sheet037 | 1 |
| `place_paper_airplanes` | 11 | domestic | sheet055 | 1 |
| `place_wall_aprons` | 11 | domestic | sheet091 | 1 |
| `place_wall_chalkboards` | 11 | office | sheet092 | 2 |
| `place_dead_plants` | 11 | nature | sheet027 | 3 |
| `place_floor_kettles` | 11 | domestic | sheet034 | 4 |
| `place_puzzle_pieces` | 11 | toys | sheet059 | 5 |
| `place_washcloths` | 11 | bathroom | sheet095 | 5 |
| `place_cast_iron_pans` | 11 | domestic | sheet018 | 6 |
| `place_cracked_mirrors` | 11 | core | sheet024 | 6 |
| `place_broken_bottles` | 10 | domestic | sheet013 | 1 |
| `place_elevator_panels` | 10 | core | sheet031 | 1 |
| `place_emergency_phones` | 10 | core | sheet031 | 1 |
| `place_mop_buckets` | 10 | industrial | sheet049 | 1 |
| `place_small_paper_napkins` | 10 | party | sheet073 | 1 |
| `place_chalk_arrows` | 10 | core | sheet020 | 2 |
| `place_claw_marks` | 10 | core | sheet021 | 3 |
| `place_fire_alarm_pulls` | 10 | core | sheet033 | 3 |
| `place_wall_pay_phones` | 10 | office | sheet094 | 4 |
| `place_biohazard_signs` | 10 | industrial | sheet011 | 5 |
| `place_floss_containers` | 10 | bathroom | sheet035 | 5 |
| `place_small_nail_polishes` | 10 | bathroom | sheet072 | 6 |
| `place_magnetic_letters` | 10 | toys | sheet103 | 7 |
| `place_frosted_windows` | 9 | core | sheet037 | 1 |
| `place_small_paper_cups` | 9 | party | sheet073 | 1 |
| `place_deflated_balls` | 9 | toys | sheet028 | 4 |
| `place_makeup_compacts` | 9 | bathroom | sheet047 | 5 |
| `place_vacuum_cleaners` | 9 | domestic | sheet089 | 5 |
| `place_hanging_colanders` | 9 | domestic | sheet042 | 6 |
| `place_small_toy_microphones` | 9 | toys | sheet078 | 6 |
| `place_vintage_maps` | 9 | domestic | sheet090 | 6 |
| `place_perfume_bottles` | 8 | bathroom | sheet056 | 2 |
| `place_graduation_photos` | 8 | domestic | sheet039 | 3 |
| `place_nail_polish` | 8 | bathroom | sheet051 | 3 |
| `place_small_dolls` | 8 | toys | sheet069 | 3 |
| `place_baby_rattles` | 8 | toys | sheet010 | 4 |
| `place_spray_bottles` | 8 | bathroom | sheet082 | 4 |
| `place_small_skateboards` | 8 | toys | sheet077 | 5 |
| `place_bookshelves` | 8 | office | sheet012 | 6 |
| `place_metal_lockers` | 8 | office | sheet048 | 6 |
| `place_emergency_strobes` | 7 | core | sheet031 | 1 |
| `place_framed_photos` | 7 | office | sheet037 | 1 |
| `place_herb_sieves` | 7 | domestic | sheet043 | 1 |
| `place_skipping_ropes` | 7 | toys | sheet067 | 1 |
| `place_small_paper_plates` | 7 | party | sheet073 | 1 |
| `place_abstract_landscapes` | 7 | domestic | sheet008 | 2 |
| `place_small_perfumes` | 7 | bathroom | sheet074 | 2 |
| `place_smoking_ashtrays` | 7 | domestic | sheet080 | 2 |
| `place_coat_racks` | 7 | domestic | sheet022 | 4 |
| `place_makeup_sponges` | 7 | bathroom | sheet047 | 5 |
| `place_shampoo_shelves` | 7 | bathroom | sheet065 | 5 |
| `place_still_lifes` | 7 | domestic | sheet083 | 5 |
| `place_umbrella_stands` | 7 | domestic | sheet089 | 5 |
| `place_boot_trays` | 7 | domestic | sheet012 | 6 |
| `place_teddy_paws` | 6 | toys | sheet085 | 1 |
| `place_baby_photos` | 6 | domestic | sheet010 | 4 |
| `place_twigs` | 6 | nature | sheet088 | 4 |
| `place_doll_body_parts` | 6 | toys | sheet029 | 5 |
| `place_shower_caps` | 6 | bathroom | sheet065 | 5 |
| `place_framed_maps` | 6 | office | sheet036 | 6 |
| `place_teething_rings` | 5 | toys | sheet085 | 1 |
| `place_action_figures` | 5 | toys | sheet008 | 2 |
| `place_small_xylophones` | 5 | toys | sheet080 | 2 |
| `place_grass_clumps` | 5 | nature | sheet039 | 3 |
| `place_cake_stands` | 5 | party | sheet016 | 4 |
| `place_hairdryers` | 5 | bathroom | sheet040 | 4 |
| `place_plush_bears` | 5 | toys | sheet058 | 4 |
| `place_doll_heads` | 5 | toys | sheet029 | 5 |
| `place_opened_suitcases` | 5 | domestic | sheet053 | 5 |
| `place_propaganda_posters` | 5 | office | sheet059 | 5 |
| `place_small_kites` | 5 | toys | sheet072 | 6 |
| `place_bird_baths` | 5 | bathroom | sheet099 | 7 |
| `place_moss_patches` | 4 | nature | sheet050 | 2 |
| `place_overturned_tricycles` | 4 | toys | sheet054 | 6 |
| `place_hanging_streamers` | 4 | party | sheet102 | 7 |
| `place_religious_triptychs` | 4 | occult | sheet105 | 7 |
| `place_skull_stacks` | 4 | occult | sheet106 | 7 |
| `place_crayon_graffiti` | 3 | toys | sheet025 | 1 |
| `place_herb_bundles` | 3 | nature | sheet043 | 1 |
| `place_small_paper_crowns` | 3 | party | sheet073 | 1 |
| `place_garden_aprons` | 3 | nature | sheet038 | 2 |
| `place_small_pinatas` | 3 | party | sheet074 | 2 |
| `place_smudge_mirrors` | 3 | bathroom | sheet080 | 2 |
| `place_fertilizer_spikes` | 3 | nature | sheet033 | 3 |
| `place_pet_toys` | 3 | toys | sheet057 | 3 |
| `place_small_crows` | 3 | occult | sheet069 | 3 |
| `place_small_princess_wands` | 3 | toys | sheet075 | 3 |
| `place_snail_shells` | 3 | nature | sheet081 | 3 |
| `place_soil_bags` | 3 | nature | sheet081 | 3 |
| `place_spider_webs` | 3 | nature | sheet081 | 3 |
| `place_toy_dinosaurs` | 3 | toys | sheet087 | 3 |
| `place_birthday_hats` | 3 | party | sheet011 | 5 |
| `place_brandy_snifters` | 3 | victorian | sheet012 | 6 |
| `place_hanging_plants` | 3 | nature | sheet042 | 6 |
| `place_crystal_skulls` | 3 | occult | sheet101 | 7 |
| `place_deer_head_mounts` | 3 | victorian | sheet101 | 7 |
| `place_sink_faucets` | 3 | bathroom | sheet106 | 7 |
| `place_small_brooms` | 3 | occult | sheet106 | 7 |
| `place_small_folding_fans` | 3 | victorian | sheet106 | 7 |
| `place_small_pumpkins` | 3 | occult | sheet108 | 7 |
| `place_small_spinning_tops` | 3 | toys | sheet108 | 7 |
| `place_paper_garlands` | 2 | party | sheet055 | 1 |
| `place_garden_gnomes` | 2 | nature | sheet038 | 2 |
| `place_mounted_antlers` | 2 | victorian | sheet050 | 2 |
| `place_mounted_fish` | 2 | victorian | sheet050 | 2 |
| `place_small_party_horns` | 2 | party | sheet074 | 2 |
| `place_antique_maps` | 2 | victorian | sheet009 | 3 |
| `place_butterfly_wings` | 2 | nature | sheet015 | 3 |
| `place_hand_towel_bars` | 2 | bathroom | sheet041 | 5 |
| `place_small_toy_dinos` | 2 | toys | sheet078 | 6 |
| `place_teddy_bears` | 2 | toys | sheet084 | 6 |
| `place_ancient_portraits` | 2 | victorian | sheet099 | 7 |
| `place_cornbread_muffins` | 2 | party | sheet100 | 7 |
| `place_fireplaces` | 2 | victorian | sheet102 | 7 |
| `place_jump_ropes` | 2 | toys | sheet103 | 7 |
| `place_record_stacks` | 2 | victorian | sheet105 | 7 |
| `place_small_black_cats` | 2 | occult | sheet106 | 7 |
| `place_stuffed_elephants` | 2 | toys | sheet109 | 7 |
| `place_brass_candlesticks` | 1 | occult | sheet013 | 1 |
| `place_miniature_portrait_clusters` | 1 | victorian | sheet049 | 1 |
| `place_small_cauldrons` | 1 | occult | sheet068 | 2 |
| `place_anatomy_charts` | 1 | office | sheet009 | 3 |
| `place_seed_catalogs` | 1 | nature | sheet064 | 4 |
| `place_small_ghost_sheets` | 1 | occult | sheet070 | 4 |
| `place_bird_nests` | 1 | nature | sheet099 | 7 |
| `place_botanical_prints` | 1 | victorian | sheet100 | 7 |
| `place_family_portraits` | 1 | victorian | sheet101 | 7 |
| `place_hanging_bats` | 1 | occult | sheet102 | 7 |
| `place_leaf_litter` | 1 | nature | sheet103 | 7 |
| `place_ornate_cuckoo_clocks` | 1 | victorian | sheet104 | 7 |
| `place_ornate_wall_mirrors` | 1 | victorian | sheet104 | 7 |
| `place_peacock_feather_vases` | 1 | victorian | sheet104 | 7 |
| `place_small_masks` | 1 | occult | sheet107 | 7 |
| `place_small_owls` | 1 | occult | sheet107 | 7 |
| `place_small_skeleton_hands` | 1 | occult | sheet108 | 7 |

## Never placed in 272 floors

43 themed placers did not run anywhere the census looked; they are
judged from their code when their turn comes:
`place_acorns`, `place_aquariums`, `place_bellows`, `place_botanicals`, `place_candy_corn_scatters`, `place_chess_side_stands`, `place_chestnuts`, `place_cigarette_butts`, `place_cotton_swabs`, `place_empty_painting_outlines`, `place_ferns`, `place_floor_globes`, `place_garden_rakes`, `place_gardening_gloves`, `place_hanging_brass_lanterns`, `place_hanging_censers`, `place_heraldic_crests`, `place_humidors`, `place_memorial_shrines`, `place_orrery_models`, `place_ouija_boards`, `place_paperclip_clusters`, `place_pentagram_chalks`, `place_pine_cones`, `place_pine_needles`, `place_pocket_watches`, `place_reliquary_boxes`, `place_rubiks_cubes`, `place_secretary_desks`, `place_silhouette_portraits`, `place_skull_plaques`, `place_small_abacuses`, `place_small_chain_links`, `place_small_crystal_balls`, `place_small_feather_boas`, `place_small_maracas`, `place_small_music_boxes`, `place_small_witch_hats`, `place_stained_glass_windows`, `place_tarot_card_fans`, `place_wagon_wheels`, `place_wall_crucifixes`, `place_wet_floor_signs`.
