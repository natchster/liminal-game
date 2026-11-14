//! The placers of plants, gardens and what grows or nests indoors (the `nature` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::base::state::PlacerFn;

mod acorns;
mod bird_feeders;
mod bird_nests;
mod botanicals;
mod butterfly_wings;
mod chestnuts;
mod dandelions;
mod dead_flower_vases;
mod dead_plants;
mod dead_potted_plants;
mod dirt_clods;
mod feathers;
mod ferns;
mod fertilizer_spikes;
mod flower_stems;
mod garden_aprons;
mod garden_gnomes;
mod garden_hoes;
mod garden_hoses;
mod garden_rakes;
mod garden_shovels;
mod gardening_gloves;
mod grass_clumps;
mod hanging_plants;
mod herb_bundles;
mod kneeling_pads;
mod leaf_litter;
mod moss_patches;
mod mushrooms;
mod pebbles;
mod pine_cones;
mod pine_needles;
mod potted_plants;
mod pressed_flowers;
mod pruning_shears;
mod seed_catalogs;
mod seed_packets;
mod seedlings;
mod shared;
mod snail_shells;
mod soil_bags;
mod spider_webs;
mod trowels;
mod twigs;
mod twine_balls;
mod watering_cans;

/// Every nature placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_acorns", acorns::place_acorns));
    r.push(("place_bird_feeders", bird_feeders::place_bird_feeders));
    r.push(("place_bird_nests", bird_nests::place_bird_nests));
    r.push(("place_botanicals", botanicals::place_botanicals));
    r.push(("place_butterfly_wings", butterfly_wings::place_butterfly_wings));
    r.push(("place_chestnuts", chestnuts::place_chestnuts));
    r.push(("place_dandelions", dandelions::place_dandelions));
    r.push(("place_dead_flower_vases", dead_flower_vases::place_dead_flower_vases));
    r.push(("place_dead_plants", dead_plants::place_dead_plants));
    r.push(("place_dead_potted_plants", dead_potted_plants::place_dead_potted_plants));
    r.push(("place_dirt_clods", dirt_clods::place_dirt_clods));
    r.push(("place_feathers", feathers::place_feathers));
    r.push(("place_ferns", ferns::place_ferns));
    r.push(("place_fertilizer_spikes", fertilizer_spikes::place_fertilizer_spikes));
    r.push(("place_flower_stems", flower_stems::place_flower_stems));
    r.push(("place_garden_aprons", garden_aprons::place_garden_aprons));
    r.push(("place_garden_gnomes", garden_gnomes::place_garden_gnomes));
    r.push(("place_garden_hoes", garden_hoes::place_garden_hoes));
    r.push(("place_garden_hoses", garden_hoses::place_garden_hoses));
    r.push(("place_garden_rakes", garden_rakes::place_garden_rakes));
    r.push(("place_garden_shovels", garden_shovels::place_garden_shovels));
    r.push(("place_gardening_gloves", gardening_gloves::place_gardening_gloves));
    r.push(("place_grass_clumps", grass_clumps::place_grass_clumps));
    r.push(("place_hanging_plants", hanging_plants::place_hanging_plants));
    r.push(("place_herb_bundles", herb_bundles::place_herb_bundles));
    r.push(("place_kneeling_pads", kneeling_pads::place_kneeling_pads));
    r.push(("place_leaf_litter", leaf_litter::place_leaf_litter));
    r.push(("place_moss_patches", moss_patches::place_moss_patches));
    r.push(("place_mushrooms", mushrooms::place_mushrooms));
    r.push(("place_pebbles", pebbles::place_pebbles));
    r.push(("place_pine_cones", pine_cones::place_pine_cones));
    r.push(("place_pine_needles", pine_needles::place_pine_needles));
    r.push(("place_potted_plants", potted_plants::place_potted_plants));
    r.push(("place_pressed_flowers", pressed_flowers::place_pressed_flowers));
    r.push(("place_pruning_shears", pruning_shears::place_pruning_shears));
    r.push(("place_seed_catalogs", seed_catalogs::place_seed_catalogs));
    r.push(("place_seed_packets", seed_packets::place_seed_packets));
    r.push(("place_seedlings", seedlings::place_seedlings));
    r.push(("place_snail_shells", snail_shells::place_snail_shells));
    r.push(("place_soil_bags", soil_bags::place_soil_bags));
    r.push(("place_spider_webs", spider_webs::place_spider_webs));
    r.push(("place_trowels", trowels::place_trowels));
    r.push(("place_twigs", twigs::place_twigs));
    r.push(("place_twine_balls", twine_balls::place_twine_balls));
    r.push(("place_watering_cans", watering_cans::place_watering_cans));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 45);
        game::debug::testkit::smoke_placers(&list);
    }
}
