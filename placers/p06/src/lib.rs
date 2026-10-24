//! Prop placers ported from backrooms.js (chunk p06; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod garden;
mod rustic;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeSoilBags", garden::place_soil_bags));
    r.push(("placeMetalBuckets", garden::place_metal_buckets));
    r.push(("placeKneelingPads", garden::place_kneeling_pads));
    r.push(("placeTwineBalls", garden::place_twine_balls));
    r.push(("placeSeedCatalogs", garden::place_seed_catalogs));
    r.push(("placePressedFlowers", garden::place_pressed_flowers));
    r.push(("placeGardenAprons", garden::place_garden_aprons));
    r.push(("placeToteBags", garden::place_tote_bags));
    r.push(("placeCoatHooksPair", garden::place_coat_hooks_pair));
    r.push(("placeToolPegboards", garden::place_tool_pegboards));
    r.push(("placeBirdFeeders", garden::place_bird_feeders));
    r.push(("placeBirdBaths", garden::place_bird_baths));
    r.push(("placeBotanicals", garden::place_botanicals));
    r.push(("placeGardenGnomes", garden::place_garden_gnomes));
    r.push(("placeMountedAntlers", rustic::place_mounted_antlers));
    r.push(("placeFertilizerSpikes", rustic::place_fertilizer_spikes));
    r.push(("placeRusticSigns", rustic::place_rustic_signs));
    r.push(("placeMountedFish", rustic::place_mounted_fish));
    r.push(("placeCuckooClocks", rustic::place_cuckoo_clocks));
    r.push(("placeCandleLanterns", rustic::place_candle_lanterns));
    r.push(("placeHorseshoes", rustic::place_horseshoes));
    r.push(("placeCastIronPans", rustic::place_cast_iron_pans));
    r.push(("placeWoodenBuckets", rustic::place_wooden_buckets));
    r.push(("placeCastIronKettles", rustic::place_cast_iron_kettles));
    r.push(("placeFirewoodStacks", rustic::place_firewood_stacks));
    r.push(("placeWagonWheels", rustic::place_wagon_wheels));
    r.push(("placeVintageMaps", rustic::place_vintage_maps));
    r.push(("placeBellows", rustic::place_bellows));
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

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 28);
        game::testkit::smoke_placers(&list);
    }
}
