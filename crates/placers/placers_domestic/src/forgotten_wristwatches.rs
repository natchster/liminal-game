//! `place_forgotten_wristwatches`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Forgotten wristwatch: 6% chance per level. Watch face + leather band on floor.
pub(crate) fn place_forgotten_wristwatches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let band_colors = [0x2a1810, 0x603820, 0x404040, 0xd8b040];
    let band_mat = lambert!({ color: pick_hex(&band_colors) });
    let case_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: watch_face_texture(), side: DoubleSide });
    let watch = group();
    // Case disc.
    let case_ = mesh(pg("ForgottenWristwatches__case", cyl_geo!(0.020, 0.020, 0.006, 12)), &case_mat);
    case_.position().y = 0.003;
    watch.add(&case_);
    // Face.
    let face = mesh(circle_geo!(0.018, 12), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0065;
    watch.add(&face);
    // Band left half (curled); each strap starts at a lug on the case edge.
    let band_l = mesh(pg("ForgottenWristwatches__strapHoles", torus_geo!(0.030, 0.005, 4, 12, PI * 0.7)), &band_mat);
    band_l.rotation().x = -PI / 2.0;
    band_l.position().set(-0.030, 0.005, -0.020);
    watch.add(&band_l);
    // Band right half.
    let band_r = mesh(pg("ForgottenWristwatches__strapKeeper", torus_geo!(0.030, 0.005, 4, 12, PI * 0.7)), &band_mat);
    band_r.set_rotation_order(EulerOrder::YXZ); // lie flat first, then turn half round
    band_r.rotation().x = -PI / 2.0;
    band_r.rotation().y = PI;
    band_r.position().set(0.030, 0.005, 0.020);
    watch.add(&band_r);
    scatter_on_floor(g, &watch, x, z, 2.4);
    g.world.add(&watch);
}
