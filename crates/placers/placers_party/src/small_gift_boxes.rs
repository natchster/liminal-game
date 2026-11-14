//! `place_small_gift_boxes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small gift box: 10% chance per level.
pub(crate) fn place_small_gift_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let box_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0, 0xa020a0];
    let ribbon_colors = [0xf0e0d0, 0xd0d0d0, 0xd8b820, 0x2050d8];
    let box_mat = lambert!({ color: pick(&box_colors) });
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors) });
    let gift = group();
    let bs = 0.10 + rnd() * 0.04; // 10-14 cm cube
                                  // Body: cube.
    let body = mesh(pg("SmallGiftBoxes__body", box_geo!(bs, bs, bs)), &box_mat);
    body.position().y = bs / 2.0;
    gift.add(&body);
    // 2 ribbon strips crossing top + wrapping sides.
    let r_wide = 0.010;
    let r_top1 = mesh(pg("SmallGiftBoxes__ribbonX", box_geo!(bs + 0.002, 0.003, r_wide)), &ribbon_mat);
    r_top1.position().y = bs + 0.001;
    gift.add(&r_top1);
    let r_top2 = mesh(pg("SmallGiftBoxes__ribbonZ", box_geo!(r_wide, 0.003, bs + 0.002)), &ribbon_mat);
    r_top2.position().y = bs + 0.001;
    gift.add(&r_top2);
    // Side ribbon wraps (front + back + left + right).
    for (rw, rh, rp, px, py, pz) in [
        (bs + 0.002, bs, r_wide, 0.0, 0.0, bs / 2.0 + 0.001),
        (bs + 0.002, bs, r_wide, 0.0, 0.0, -bs / 2.0 - 0.001),
        (r_wide, bs, bs + 0.002, bs / 2.0 + 0.001, 0.0, 0.0),
        (r_wide, bs, bs + 0.002, -bs / 2.0 - 0.001, 0.0, 0.0),
    ] {
        let r_side = mesh(box_geo!(rw, rh, if rw == r_wide { rp } else { rw }), &ribbon_mat);
        // Scale the box to a 1 cm x 2 mm strip down the middle of its face,
        // continuing the top ribbon, and add it (it was never added).
        let gd = if rw == r_wide { rp } else { rw };
        r_side.scale().set(if px == 0.0 { r_wide / rw } else { 0.002 / rw }, 1, if px == 0.0 { 0.002 / gd } else { r_wide / gd });
        r_side.position().set(px, bs / 2.0 + py, pz);
        gift.add(&r_side);
    }
    // Bow: 2 small triangles at ribbon crossing.
    for bow_side in [-1.0f64, 1.0] {
        let bow_geo = Geo::new();
        let bow_verts: Vec<f32> = [0.0, bs + 0.005, 0.0, bow_side * 0.025, bs + 0.020, 0.0, bow_side * 0.020, bs + 0.005, bow_side * 0.010]
            .iter()
            .map(|&v| v as f32)
            .collect();
        bow_geo.set_position(bow_verts);
        bow_geo.set_index(vec![0, 1, 2]);
        bow_geo.compute_vertex_normals();
        let bow = mesh(bow_geo, &ribbon_mat);
        gift.add(&bow);
    }
    gift.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    gift.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gift);
}
