//! Prop placers: garden-shed props (soil bags, buckets, twine, seed catalogs,
//! pressed flowers, aprons, pegboards, bird feeders and baths, botanicals,
//! gnomes) and rustic-cabin props (antlers, signs, mounted fish, cuckoo clocks,
//! lanterns, horseshoes, cast iron, firewood, wagon wheels, maps, bellows).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod garden;
mod rustic;
mod t06;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_soil_bags", garden::place_soil_bags));
    r.push(("place_metal_buckets", garden::place_metal_buckets));
    r.push(("place_kneeling_pads", garden::place_kneeling_pads));
    r.push(("place_twine_balls", garden::place_twine_balls));
    r.push(("place_seed_catalogs", garden::place_seed_catalogs));
    r.push(("place_pressed_flowers", garden::place_pressed_flowers));
    r.push(("place_garden_aprons", garden::place_garden_aprons));
    r.push(("place_tote_bags", garden::place_tote_bags));
    r.push(("place_coat_hooks_pair", garden::place_coat_hooks_pair));
    r.push(("place_tool_pegboards", garden::place_tool_pegboards));
    r.push(("place_bird_feeders", garden::place_bird_feeders));
    r.push(("place_bird_baths", garden::place_bird_baths));
    r.push(("place_botanicals", garden::place_botanicals));
    r.push(("place_garden_gnomes", garden::place_garden_gnomes));
    r.push(("place_mounted_antlers", rustic::place_mounted_antlers));
    r.push(("place_fertilizer_spikes", rustic::place_fertilizer_spikes));
    r.push(("place_rustic_signs", rustic::place_rustic_signs));
    r.push(("place_mounted_fish", rustic::place_mounted_fish));
    r.push(("place_cuckoo_clocks", rustic::place_cuckoo_clocks));
    r.push(("place_candle_lanterns", rustic::place_candle_lanterns));
    r.push(("place_horseshoes", rustic::place_horseshoes));
    r.push(("place_cast_iron_pans", rustic::place_cast_iron_pans));
    r.push(("place_wooden_buckets", rustic::place_wooden_buckets));
    r.push(("place_cast_iron_kettles", rustic::place_cast_iron_kettles));
    r.push(("place_firewood_stacks", rustic::place_firewood_stacks));
    r.push(("place_wagon_wheels", rustic::place_wagon_wheels));
    r.push(("place_vintage_maps", rustic::place_vintage_maps));
    r.push(("place_bellows", rustic::place_bellows));
}

// ------------------------------------------------------------------ shared local helpers

pub(crate) use placers_kit::place::open_tiles;

pub(crate) use placers_kit::place::wall_tiles;

/// A uniformly random element (one `rnd()` draw).
#[inline]
pub(crate) fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

pub(crate) use placers_kit::place::face_yaw;

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 28);
        game::debug::testkit::smoke_placers(&list);
    }
}
