//! Prop placers ported from backrooms.js (chunk p05; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod toys;
mod garden;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeCrayons", toys::place_crayons));
    r.push(("placePuzzlePieces", toys::place_puzzle_pieces));
    r.push(("placeActionFigures", toys::place_action_figures));
    r.push(("placeBabyRattles", toys::place_baby_rattles));
    r.push(("placeToyDinosaurs", toys::place_toy_dinosaurs));
    r.push(("placeGumBallDispensers", toys::place_gum_ball_dispensers));
    r.push(("placePacifiers", toys::place_pacifiers));
    r.push(("placeSippyCups", toys::place_sippy_cups));
    r.push(("placeThermometers", toys::place_thermometers));
    r.push(("placeBabyBottles", toys::place_baby_bottles));
    r.push(("place8TrackTapes", toys::place_8_track_tapes));
    r.push(("placeStuffedRabbits", toys::place_stuffed_rabbits));
    r.push(("placeStuffedElephants", toys::place_stuffed_elephants));
    r.push(("placeCrossStitches", garden::place_cross_stitches));
    r.push(("placeHandMirrors", garden::place_hand_mirrors));
    r.push(("placeWatercolors", garden::place_watercolors));
    r.push(("placeHangingPlants", garden::place_hanging_plants));
    r.push(("placeSeedlings", garden::place_seedlings));
    r.push(("placeTrowels", garden::place_trowels));
    r.push(("placeSeedPackets", garden::place_seed_packets));
    r.push(("placeWateringCans", garden::place_watering_cans));
    r.push(("placeGardeningGloves", garden::place_gardening_gloves));
    r.push(("placePruningShears", garden::place_pruning_shears));
    r.push(("placeGardenHoes", garden::place_garden_hoes));
    r.push(("placeGardenRakes", garden::place_garden_rakes));
    r.push(("placeGardenHoses", garden::place_garden_hoses));
    r.push(("placeGardenShovels", garden::place_garden_shovels));
}

// ------------------------------------------------------------------ shared local helpers

/// The `openTiles` scan every floor placer repeats: open interior tiles, row-major.
pub(crate) fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// The `wallTiles` scan: open interior tiles with the first wall neighbour
/// in `[[1,0],[-1,0],[0,1],[0,-1]]` order, as `(x, z, dx, dz)`.
pub(crate) fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    v
}

/// `arr[Math.floor(Math.random() * arr.length)]`
#[inline]
pub(crate) fn pick<T: Copy>(a: &[T]) -> T { a[(rnd() * a.len() as f64).floor() as usize] }

/// `Math.atan2(-dx, -dz)` with JS's signed zeros (`-0` for `dx == 0`).
#[inline]
pub(crate) fn face_yaw(dx: i32, dz: i32) -> f64 { (-(dx as f64)).atan2(-(dz as f64)) }

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
        game::testkit::smoke_placers(&list);
    }
}
