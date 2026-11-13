//! Prop placers: party remnants (cupcake wrappers, napkins, plates, birthday
//! candles and hats, confetti, ribbons, balloons), children's toys and items
//! (tambourines, ropes, books, masks, drums, kites, pinwheels, dolls, cars,
//! marbles, blocks, shoes, crayons, drawings, balls, plush bears), luggage, and
//! restroom fixtures (paper towel dispensers, tissue boxes, hand dryers).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod party;
mod toys;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_small_cupcake_wrappers", party::place_small_cupcake_wrappers));
    r.push(("place_small_paper_napkins", party::place_small_paper_napkins));
    r.push(("place_small_paper_plates", party::place_small_paper_plates));
    r.push(("place_small_birthday_candles", party::place_small_birthday_candles));
    r.push(("place_confetti_scatters", party::place_confetti_scatters));
    r.push(("place_birthday_hats", party::place_birthday_hats));
    r.push(("place_small_ribbons", party::place_small_ribbons));
    r.push(("place_small_balloons", party::place_small_balloons));
    r.push(("place_small_tambourines", party::place_small_tambourines));
    r.push(("place_skipping_ropes", party::place_skipping_ropes));
    r.push(("place_childrens_books", party::place_childrens_books));
    r.push(("place_small_pacifiers", toys::place_small_pacifiers));
    r.push(("place_small_masks", toys::place_small_masks));
    r.push(("place_toy_drums", toys::place_toy_drums));
    r.push(("place_small_hats", toys::place_small_hats));
    r.push(("place_small_yo_yos", toys::place_small_yo_yos));
    r.push(("place_small_kites", toys::place_small_kites));
    r.push(("place_small_pinwheels", toys::place_small_pinwheels));
    r.push(("place_small_dolls", toys::place_small_dolls));
    r.push(("place_toy_cars", toys::place_toy_cars));
    r.push(("place_marble_scatters", toys::place_marble_scatters));
    r.push(("place_building_blocks", toys::place_building_blocks));
    r.push(("place_children_shoes", toys::place_children_shoes));
    r.push(("place_small_crayons", toys::place_small_crayons));
    r.push(("place_drawing_papers", toys::place_drawing_papers));
    r.push(("place_rubber_balls", toys::place_rubber_balls));
    r.push(("place_plush_bears", toys::place_plush_bears));
    r.push(("place_handbaskets", toys::place_handbaskets));
    r.push(("place_suitcases", toys::place_suitcases));
    r.push(("place_paper_towel_dispensers", toys::place_paper_towel_dispensers));
    r.push(("place_tissue_boxes", toys::place_tissue_boxes));
    r.push(("place_hand_dryers", toys::place_hand_dryers));
}

pub(crate) use placers_kit::place::open_tiles;

/// Wall-mount candidates `{ x, z, dx, dz }`: one per (open tile, side facing a
/// wall or the grid edge), in scan order.
pub(crate) fn wall_candidates(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push((x, z, dx, dz));
                }
            }
        }
    }
    v
}

/// A uniformly random element (one `rnd()` draw).
#[inline]
pub(crate) fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

pub(crate) use placers_kit::geometry::buf_geo;

pub(crate) use placers_kit::geometry::set_rot_order;

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 32);
        game::debug::testkit::smoke_placers(&list);
    }
}
