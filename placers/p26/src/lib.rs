//! Prop placers ported from backrooms.js (chunk p26; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod party;
mod toys;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeSmallCupcakeWrappers", party::place_small_cupcake_wrappers));
    r.push(("placeSmallPaperNapkins", party::place_small_paper_napkins));
    r.push(("placeSmallPaperPlates", party::place_small_paper_plates));
    r.push(("placeSmallBirthdayCandles", party::place_small_birthday_candles));
    r.push(("placeConfettiScatters", party::place_confetti_scatters));
    r.push(("placeBirthdayHats", party::place_birthday_hats));
    r.push(("placeSmallRibbons", party::place_small_ribbons));
    r.push(("placeSmallBalloons", party::place_small_balloons));
    r.push(("placeSmallTambourines", party::place_small_tambourines));
    r.push(("placeSkippingRopes", party::place_skipping_ropes));
    r.push(("placeChildrensBooks", party::place_childrens_books));
    r.push(("placeSmallPacifiers", toys::place_small_pacifiers));
    r.push(("placeSmallMasks", toys::place_small_masks));
    r.push(("placeToyDrums", toys::place_toy_drums));
    r.push(("placeSmallHats", toys::place_small_hats));
    r.push(("placeSmallYoYos", toys::place_small_yo_yos));
    r.push(("placeSmallKites", toys::place_small_kites));
    r.push(("placeSmallPinwheels", toys::place_small_pinwheels));
    r.push(("placeSmallDolls", toys::place_small_dolls));
    r.push(("placeToyCars", toys::place_toy_cars));
    r.push(("placeMarbleScatters", toys::place_marble_scatters));
    r.push(("placeBuildingBlocks", toys::place_building_blocks));
    r.push(("placeChildrenShoes", toys::place_children_shoes));
    r.push(("placeSmallCrayons", toys::place_small_crayons));
    r.push(("placeDrawingPapers", toys::place_drawing_papers));
    r.push(("placeRubberBalls", toys::place_rubber_balls));
    r.push(("placePlushBears", toys::place_plush_bears));
    r.push(("placeHandbaskets", toys::place_handbaskets));
    r.push(("placeSuitcases", toys::place_suitcases));
    r.push(("placePaperTowelDispensers", toys::place_paper_towel_dispensers));
    r.push(("placeTissueBoxes", toys::place_tissue_boxes));
    r.push(("placeHandDryers", toys::place_hand_dryers));
}

/// The `openTiles` scan every placer starts with: open tiles, row-major.
pub(crate) fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// Wall-mount candidates `{ x, z, dx, dz }`: one per (open tile, side facing a
/// wall or the grid edge), in scan order.
pub(crate) fn wall_candidates(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 { continue; }
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

/// `arr[Math.floor(Math.random() * arr.length)]`
#[inline]
pub(crate) fn pick<T: Copy>(a: &[T]) -> T { a[(rnd() * a.len() as f64).floor() as usize] }

/// `new THREE.BufferGeometry()` + `position` (Float32Array) + `setIndex(idx)` + `computeVertexNormals()`
pub(crate) fn buf_geo(verts: &[f64], idx: &[u32]) -> Geo {
    let geo = Geo::new();
    geo.set_position(verts.iter().map(|&v| v as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}

/// `o.rotation.set(x, y, z, order)`
pub(crate) fn set_rot_order(o: &Obj, x: f64, y: f64, z: f64, order: EulerOrder) {
    let mut r = o.rotation();
    r.set(x, y, z);
    r.order = order;
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 32);
        game::testkit::smoke_placers(&list);
    }
}
