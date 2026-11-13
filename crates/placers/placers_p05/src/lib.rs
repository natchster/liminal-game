//! Prop placers: toys and nursery items (crayons, puzzle pieces, action figures,
//! rattles, dinosaurs, gumball dispensers, pacifiers, bottles, plush animals,
//! 8-track tapes, a wall thermometer), wall crafts and hand mirrors, and garden
//! props (hanging plants, seedlings, hand tools, seed packets, watering cans,
//! gloves, hoses).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod garden;
mod t06;
mod toys;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_crayons", toys::place_crayons));
    r.push(("place_puzzle_pieces", toys::place_puzzle_pieces));
    r.push(("place_action_figures", toys::place_action_figures));
    r.push(("place_baby_rattles", toys::place_baby_rattles));
    r.push(("place_toy_dinosaurs", toys::place_toy_dinosaurs));
    r.push(("place_gum_ball_dispensers", toys::place_gum_ball_dispensers));
    r.push(("place_pacifiers", toys::place_pacifiers));
    r.push(("place_sippy_cups", toys::place_sippy_cups));
    r.push(("place_thermometers", toys::place_thermometers));
    r.push(("place_baby_bottles", toys::place_baby_bottles));
    r.push(("place_8_track_tapes", toys::place_8_track_tapes));
    r.push(("place_stuffed_rabbits", toys::place_stuffed_rabbits));
    r.push(("place_stuffed_elephants", toys::place_stuffed_elephants));
    r.push(("place_cross_stitches", garden::place_cross_stitches));
    r.push(("place_hand_mirrors", garden::place_hand_mirrors));
    r.push(("place_watercolors", garden::place_watercolors));
    r.push(("place_hanging_plants", garden::place_hanging_plants));
    r.push(("place_seedlings", garden::place_seedlings));
    r.push(("place_trowels", garden::place_trowels));
    r.push(("place_seed_packets", garden::place_seed_packets));
    r.push(("place_watering_cans", garden::place_watering_cans));
    r.push(("place_gardening_gloves", garden::place_gardening_gloves));
    r.push(("place_pruning_shears", garden::place_pruning_shears));
    r.push(("place_garden_hoes", garden::place_garden_hoes));
    r.push(("place_garden_rakes", garden::place_garden_rakes));
    r.push(("place_garden_hoses", garden::place_garden_hoses));
    r.push(("place_garden_shovels", garden::place_garden_shovels));
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

/// `o.rotation.set(x, y, z, order)`
pub(crate) fn set_rot_order(o: &Obj, x: f64, y: f64, z: f64, order: EulerOrder) {
    o.set_rotation_order(order);
    o.rotation().set(x, y, z);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 27);
        game::debug::testkit::smoke_placers(&list);
    }
}
