//! The placers of plants, gardens and what grows or nests indoors (the `nature` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod dead_potted_plants;
mod flower_stems;
mod garden_aprons;
mod garden_hoses;
mod garden_shovels;
mod hanging_plants;
mod herb_bundles;
mod kneeling_pads;
mod moss_patches;
mod mushrooms;
mod potted_plants;
mod pressed_flowers;
mod shared;
mod spider_webs;
mod trowels;
mod twine_balls;
mod watering_cans;

/// Every nature placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_dead_potted_plants", dead_potted_plants::place_dead_potted_plants));
    r.push(("place_flower_stems", flower_stems::place_flower_stems));
    r.push(("place_garden_aprons", garden_aprons::place_garden_aprons));
    r.push(("place_garden_hoses", garden_hoses::place_garden_hoses));
    r.push(("place_garden_shovels", garden_shovels::place_garden_shovels));
    r.push(("place_hanging_plants", hanging_plants::place_hanging_plants));
    r.push(("place_herb_bundles", herb_bundles::place_herb_bundles));
    r.push(("place_kneeling_pads", kneeling_pads::place_kneeling_pads));
    r.push(("place_moss_patches", moss_patches::place_moss_patches));
    r.push(("place_mushrooms", mushrooms::place_mushrooms));
    r.push(("place_potted_plants", potted_plants::place_potted_plants));
    r.push(("place_pressed_flowers", pressed_flowers::place_pressed_flowers));
    r.push(("place_spider_webs", spider_webs::place_spider_webs));
    r.push(("place_trowels", trowels::place_trowels));
    r.push(("place_twine_balls", twine_balls::place_twine_balls));
    r.push(("place_watering_cans", watering_cans::place_watering_cans));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 16);
        game::debug::testkit::smoke_placers(&list);
    }
}
